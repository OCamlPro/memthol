pub extern crate log;

#[macro_use]
mod macros;

pub const VERSION: u16 = 2;

pub use base::err;

#[macro_use]
pub mod prelude;

pub mod ast;
pub mod btrace;
pub mod loc;
pub mod parse;

prelude! {}

/// Activates verbose parsing, only active in debug and test.
#[cfg(any(test, not(release)))]
const VERB: bool = false;
/// Activates debug parsing, only active in debug and test.
#[cfg(any(test, not(release)))]
const DEBUG_VERB: bool = false;

use ast::{event::Event, *};

pub trait EventAction<'data>:
    FnMut(Option<&ast::header::Packet>, Clock, Event<'data>) -> err::Res<()>
{
}
impl<'data, T> EventAction<'data> for T where
    T: FnMut(Option<&ast::header::Packet>, Clock, Event<'data>) -> err::Res<()>
{
}

pub use diff_parse::parse;

mod diff_parse {
    use alloc_data::prelude::*;

    type Encoded = usize;
    type LocMap = Map<Encoded, Loc>;

    fn build_trace(loc_map: &LocMap, trace: SVec16<usize>) -> Res<Trace> {
        let mut res = Vec::with_capacity(trace.len());
        let mut trace = trace.into_iter();
        let mut next = trace.next();

        while let Some(encoded) = next {
            let loc = if let Some(loc) = loc_map.get(&encoded).cloned() {
                loc
            } else {
                bail!("[ctf parser] unknown encoded location {:?}", encoded)
            };

            let mut cnt = 1;

            'while_same_encoded: loop {
                match trace.next() {
                    Some(enc) if encoded == enc => cnt += 1,
                    new_next => {
                        next = new_next;
                        break 'while_same_encoded;
                    }
                }
            }

            let cloc = CLoc::new(loc, cnt);
            res.push(cloc)
        }

        res.shrink_to_fit();
        Ok(Trace::new(res))
    }

    fn date_from_millis(date: crate::prelude::Clock) -> Date {
        Date::from_millis(convert(date, "date_from_millis"))
    }

    pub fn parse(bytes: &[u8]) -> Res<(Init, Vec<Diff>)> {
        let mut factory = mem::Factory::new(false);

        // Maps location encoded identifiers to actual locations.
        let mut loc_id_to_loc = LocMap::new();

        // Registers a bunch of locations.
        macro_rules! register_locs {
        ($locs:expr) => {
            for loc in $locs {
                register_locs!(@one loc)
            }
        };
        (@one $loc:expr) => {{
            let encoded = $loc.encoded;
            let file = factory.register_str($loc.file_path);
            let line = $loc.line;
            let col = $loc.col;

            let loc = Loc::new(
                file,
                line,
                Span {
                    start: col.begin,
                    end: col.end,
                },
            );

            let prev = loc_id_to_loc.insert(encoded, loc);

            if prev.is_some() {
                bail!("[ctf parser] trying to register encoded location #{} twice", encoded)
            }
        }};
    }

        // Maps allocation UIDs to the index of the diff it appears in (as a new allocation), and the
        // index of that allocation.
        let mut alloc_uid_to_diff_idx: Map<Uid, (usize, usize)> = Map::new();

        parse! {
            bytes => |mut parser| {
                let header = parser.header();

                // Start time of the run, used for init and to compute the time-since-start of all
                // events.
                let start_time = date_from_millis(header.header.timestamp.begin);

                // Init info.
                let init = {
                    let trace_info = parser.trace_info();
                    Init::new(start_time, trace_info.word_size as usize, false)
                };

                // List of diffs yielded by the function. One diff per packet.
                let mut diffs = Vec::with_capacity(123);

                // Iterate over the packet of the trace.
                while let Some(mut packet_parser) = parser.next_packet()? {
                    let header = packet_parser.header();
                    // Header gives us the number of new allocation, used to set the capacity of the
                    // `new` list in the diff to avoid reallocation.
                    let new_len = (header.header.alloc_id.end - header.header.alloc_id.begin) as usize;

                    // Index of the current diff in `diffs`.
                    let diff_idx = diffs.len();
                    // Empty diff we will fill as we parse the packet.
                    let diff = Diff {
                        time: date_from_millis(packet_parser.header().timestamp.begin).sub(start_time)?,
                        new: Vec::with_capacity(new_len),
                        dead: Vec::with_capacity(0),
                    };
                    diffs.push(diff);

                    // Iterate over the events of the packet.
                    while let Some((clock, event)) = packet_parser.next_event()? {
                        use crate::ast::event::Event;

                        match event {
                            Event::Alloc(alloc) => {
                                let uid = Uid::from(alloc.id);

                                // Remember the index of the allocation in `diff.new` to propagate the
                                // time of death later if needed.
                                let prev = alloc_uid_to_diff_idx.insert(
                                    uid.clone(), (diff_idx, diffs[diff_idx].new.len())
                                );
                                if prev.is_some() {
                                    bail!("[ctf parser] trying to register allocation #{} twice", uid)
                                }

                                // Build the allocation.
                                let alloc = {
                                    let time_since_start = date_from_millis(clock).sub(start_time)?;
                                    let trace = build_trace(&loc_id_to_loc, alloc.backtrace)?;
                                    Alloc::new(
                                        uid,
                                        AllocKind::Minor,
                                        convert(alloc.len, "ctf parser: alloc size"),
                                        trace,
                                        Labels::new(vec![]),
                                        time_since_start,
                                        None
                                    )
                                };

                                // Push as a new allocation.
                                diffs[diff_idx].new.push(alloc)
                            },

                            Event::Collection(alloc_uid) => {
                                let uid = Uid::from(alloc_uid);
                                let time_since_start = date_from_millis(clock).sub(start_time)?;

                                if let Some((diff_idx, alloc_idx)) = alloc_uid_to_diff_idx.get(&uid) {
                                    diffs[*diff_idx].new[*alloc_idx].set_tod(time_since_start)?
                                } else {
                                    bail!("[ctf parser] collection for unknown allocation #{}", uid)
                                }
                            },
                            Event::Locs(locs) => register_locs!(locs.locs),
                            Event::Promotion(_) => (),
                        }
                    }

                    debug_assert_eq!(diffs[diff_idx].new.len(), new_len);
                    debug_assert_eq!(diffs[diff_idx].new.len(), diffs[diff_idx].new.capacity());
                    debug_assert!(diffs[diff_idx].dead.is_empty());

                    if diffs[diff_idx].new.is_empty() {
                        let _ = diffs.pop();
                    }
                }

                diffs.shrink_to_fit();

                Ok((init, diffs))
            }
        }
    }
}
