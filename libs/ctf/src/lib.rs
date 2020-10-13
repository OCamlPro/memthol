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

    type Encoded = u64;
    type LocMap = std::collections::HashMap<Encoded, Vec<Loc>>;

    #[inline]
    fn build_trace<I>(factory: &mut mem::Factory, loc_map: &LocMap, trace: I) -> Res<Trace>
    where
        I: Iterator<Item = usize> + ExactSizeIterator,
    {
        let mut res: SVec32<CLoc> = SVec32::with_capacity(trace.len());

        for code in trace {
            let sub_trace = match loc_map.get(&(code as u64)) {
                Some(sub_trace) => {
                    if sub_trace.is_empty() {
                        continue;
                    } else {
                        sub_trace
                    }
                }
                None => bail!("[ctf parser] unknown location code `{}`", code),
            };

            for loc in sub_trace {
                if let Some(cloc) = res.last_mut() {
                    if loc == &cloc.loc {
                        cloc.cnt += 1
                    } else {
                        res.push(CLoc::new(loc.clone(), 1));
                    }
                } else {
                    res.push(CLoc::new(loc.clone(), 1))
                };
            }
        }

        res.shrink_to_fit();

        Ok(factory.register_trace(res))
    }

    fn date_from_microsecs(date: crate::prelude::Clock) -> Date {
        Date::from_microsecs(convert(date, "date_from_microsecs"))
    }

    // pub fn parse(bytes: &[u8], mut bytes_progress: impl FnMut(usize)) -> Res<(Init, Vec<Diff>)> {
    //     let mut factory = mem::Factory::new(false);

    //     // Maps location encoded identifiers to actual locations.
    //     let mut loc_id_to_loc = LocMap::new();

    //     // Maps allocation UIDs to the index of the diff it appears in (as a new allocation), and
    //     // the index of that allocation.
    //     let mut alloc_uid_to_diff_idx: Map<Uid, (usize, usize)> = Map::new();

    //     parse! {
    //         bytes => |mut parser| {
    //             let header = parser.header();

    //             // Start time of the run, used for init and to compute the time-since-start of all
    //             // events.
    //             let start_time = date_from_microsecs(header.header.timestamp.begin);
    //             let end_time = date_from_microsecs(header.header.timestamp.end).sub(start_time)?;

    //             // Init info.
    //             let init = {
    //                 let trace_info = parser.trace_info();
    //                 Init::new(start_time, Some(end_time), trace_info.word_size as usize, false)
    //             };

    //             // List of diffs yielded by the function. One diff per packet.
    //             let mut diffs = Vec::with_capacity(123);

    //             // Iterate over the packet of the trace.
    //             while let Some(mut packet_parser) = parser.next_packet()? {
    //                 bytes_progress(packet_parser.real_position().0);
    //                 let header = packet_parser.header();
    //                 // Header gives us the number of new allocation, used to set the capacity of the
    //                 // `new` list in the diff to avoid reallocation.
    //                 let new_len = (header.header.alloc_id.end - header.header.alloc_id.begin) as usize;

    //                 // Index of the current diff in `diffs`.
    //                 let diff_idx = diffs.len();
    //                 // Empty diff we will fill as we parse the packet.
    //                 let diff = Diff {
    //                     time: date_from_microsecs(packet_parser.header().timestamp.begin)
    //                         .sub(start_time)?,
    //                     new: Vec::with_capacity(new_len),
    //                     dead: Vec::with_capacity(0),
    //                 };
    //                 diffs.push(diff);

    //                 // Iterate over the events of the packet.
    //                 while let Some((clock, event)) = packet_parser.next_event()? {
    //                     use crate::ast::event::Event;

    //                     match event {
    //                         Event::Alloc(alloc) => {
    //                             let uid = Uid::from(alloc.id);

    //                             // Remember the index of the allocation in `diff.new` to propagate
    //                             // the time of death later if needed.
    //                             let prev = alloc_uid_to_diff_idx.insert(
    //                                 uid.clone(), (diff_idx, diffs[diff_idx].new.len())
    //                             );
    //                             if prev.is_some() {
    //                                 bail!(
    //                                     "[ctf parser] trying to register allocation #{} twice",
    //                                     uid,
    //                                 )
    //                             }

    //                             // Build the allocation.
    //                             let alloc = {
    //                                 let time_since_start = date_from_microsecs(clock).sub(start_time)?;
    //                                 let trace = build_trace(
    //                                     &mut factory,
    //                                     &loc_id_to_loc,
    //                                     alloc.backtrace.into_iter().take(alloc.backtrace_len),
    //                                 )?;
    //                                 let labels = factory.register_labels(SVec32::new());
    //                                 Alloc::new(
    //                                     uid,
    //                                     AllocKind::Minor,
    //                                     convert(alloc.len, "ctf parser: alloc size"),
    //                                     trace,
    //                                     labels,
    //                                     time_since_start,
    //                                     None
    //                                 )
    //                             };

    //                             // Push as a new allocation.
    //                             diffs[diff_idx].new.push(alloc)
    //                         },

    //                         Event::Collection(alloc_uid) => {
    //                             let uid = Uid::from(alloc_uid);
    //                             let time_since_start = date_from_microsecs(clock).sub(start_time)?;

    //                             if let Some((diff_idx, alloc_idx)) = alloc_uid_to_diff_idx.get(&uid) {
    //                                 diffs[*diff_idx].new[*alloc_idx].set_tod(time_since_start)?
    //                             } else {
    //                                 bail!("[ctf parser] collection for unknown allocation #{}", uid)
    //                             }
    //                         },
    //                         Event::Locs(crate::ast::Locs { id, locs }) => {
    //                             let locs = locs.into_iter().map(|loc| {
    //                                 let file = factory.register_str(loc.file_path);
    //                                 let line = loc.line;
    //                                 let col = loc.col;

    //                                 Loc::new(
    //                                     file,
    //                                     line,
    //                                     Span {
    //                                         start: col.begin,
    //                                         end: col.end,
    //                                     },
    //                                 )
    //                             }).collect();

    //                             let prev = loc_id_to_loc.insert(id, locs);
    //                             if prev.is_some() && prev.as_ref() != loc_id_to_loc.get(&id) {
    //                                 bail!("[ctf parser] trying to register locations #{} twice", id)
    //                             }
    //                         },
    //                         Event::Promotion(_) => {
    //                             ()
    //                         },
    //                     }
    //                 }

    //                 debug_assert_eq!(diffs[diff_idx].new.len(), new_len);
    //                 debug_assert_eq!(diffs[diff_idx].new.len(), diffs[diff_idx].new.capacity());
    //                 debug_assert!(diffs[diff_idx].dead.is_empty());
    //             }

    //             diffs.shrink_to_fit();

    //             Ok((init, diffs))
    //         }
    //     }
    // }

    pub fn parse<'a, F>(
        bytes: &[u8],
        mut factory: &mut F,
        mut bytes_progress: impl FnMut(usize),
        init_action: impl FnOnce(&mut F, Init),
        mut new_action: impl FnMut(&mut F, Alloc),
        mut dead_action: impl FnMut(&mut F, SinceStart, Uid),
    ) -> Res<()>
    where
        F: std::ops::DerefMut<Target = mem::Factory<'a>>,
    {
        // Maps location encoded identifiers to actual locations.
        let mut loc_id_to_loc = LocMap::with_capacity(1001);

        parse! {
            bytes => |mut parser| {
                let header = parser.header();

                // Start time of the run, used for init and to compute the time-since-start of all
                // events.
                let start_time = date_from_microsecs(header.header.timestamp.begin);
                // let end_time = date_from_microsecs(header.header.timestamp.end).sub(start_time)?;

                // Init info.
                let init = {
                    let trace_info = parser.trace_info();
                    Init::new(
                        start_time,
                        None,
                        convert(trace_info.word_size, "ctf parser: word_size"),
                        false,
                    )
                };

                init_action(factory, init);

                // Iterate over the packet of the trace.
                while let Some(mut packet_parser) = parser.next_packet()? {
                    bytes_progress(packet_parser.real_position().0);
                    let _header = packet_parser.header();

                    // Iterate over the events of the packet.
                    while let Some((clock, event)) = packet_parser.next_event()? {
                        use crate::ast::event::Event;

                        match event {
                            Event::Alloc(alloc) => {
                                let uid = Uid::from(alloc.id);

                                // Build the allocation.
                                let alloc = {
                                    let time_since_start = date_from_microsecs(clock).sub(start_time)?;
                                    let trace = build_trace(
                                        factory,
                                        &loc_id_to_loc,
                                        alloc.backtrace.into_iter().take(alloc.backtrace_len),
                                    )?;
                                    let labels = factory.register_labels(SVec32::new());
                                    let alloc = Alloc::new(
                                        uid,
                                        AllocKind::Minor,
                                        convert(alloc.len, "ctf parser: alloc size"),
                                        trace,
                                        labels,
                                        time_since_start,
                                        None
                                    );
                                    alloc
                                };

                                new_action(factory, alloc)
                            },

                            Event::Collection(alloc_uid) => {
                                let uid = Uid::from(alloc_uid);
                                let timestamp = date_from_microsecs(clock).sub(start_time)?;

                                dead_action(&mut factory, timestamp, uid)
                            },
                            Event::Locs(crate::ast::Locs { id, locs }) => {
                                let locs = locs.into_iter().map(|loc| {
                                    let file = factory.register_str(loc.file_path);
                                    let line = loc.line;
                                    let col = loc.col;

                                    Loc::new(
                                        file,
                                        line,
                                        Span {
                                            start: col.begin,
                                            end: col.end,
                                        },
                                    )
                                }).collect();

                                let prev = loc_id_to_loc.insert(id, locs);
                                if prev.is_some() && prev.as_ref() != loc_id_to_loc.get(&id) {
                                    bail!("[ctf parser] trying to register locations #{} twice", id)
                                }
                            },
                            Event::Promotion(_) => {
                                ()
                            },
                        }
                    }
                }

                Ok(())
            }
        }
    }
}
