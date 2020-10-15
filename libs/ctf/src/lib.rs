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
#[cfg(debug_assertions)]
const VERB: bool = false;
/// Activates debug parsing, only active in debug and test.
#[cfg(debug_assertions)]
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
        base::new_time_stats! {
            struct Prof {
                pub total => "total",
                pub basic_parsing => "basic parsing",
                pub event_parsing => "event parsing",
                pub packet_parsing => "packet parsing",
                pub trace_building => "building traces",
                pub locations => "registering locations",
                pub dead => "handling collections",
                pub alloc => "handling allocations",
            }
        }
        let mut prof = Prof::new();
        prof.total.start();

        // Maps location encoded identifiers to actual locations.
        let mut loc_id_to_loc = LocMap::with_capacity(1001);

        parse! {
            bytes => |mut parser| {
                prof.basic_parsing.start();

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
                prof.basic_parsing.stop();

                // Iterate over the packet of the trace.
                while let Some(mut packet_parser) = prof.packet_parsing.time(
                    || parser.next_packet()
                )? {
                    bytes_progress(packet_parser.real_position().0);

                    // Iterate over the events of the packet.
                    while let Some((clock, event)) = prof.event_parsing.time(
                        || packet_parser.next_event()
                    )? {
                        use crate::ast::event::Event;

                        match event {
                            Event::Alloc(crate::ast::event::Alloc { id: uid, backtrace, backtrace_len, len, .. }) => {

                                let trace = {
                                    prof.trace_building.time(|| build_trace(
                                        factory,
                                        &loc_id_to_loc,
                                        backtrace.into_iter().take(backtrace_len),
                                    ))?
                                };

                                prof.alloc.start();

                                // Build the allocation.
                                let alloc = {
                                    let time_since_start =
                                        date_from_microsecs(clock).sub(start_time)?;
                                    let labels = factory.register_labels(SVec32::new());
                                    let alloc = Alloc::new(
                                        uid,
                                        AllocKind::Minor,
                                        convert(len, "ctf parser: alloc size"),
                                        trace,
                                        labels,
                                        time_since_start,
                                        None
                                    );
                                    alloc
                                };

                                new_action(factory, alloc);

                                prof.alloc.stop()
                            },

                            Event::Collection(alloc_uid) => {
                                prof.dead.start();

                                let uid = Uid::from(alloc_uid);
                                let timestamp = date_from_microsecs(clock).sub(start_time)?;

                                dead_action(&mut factory, timestamp, uid);

                                prof.dead.stop();
                            },
                            Event::Locs(crate::ast::Locs { id, locs }) => {
                                prof.locations.start();

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
                                prof.locations.stop();
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

                prof.all_do(
                    || base::log::info!("done parsing"),
                    |desc, sw| base::log::info!("| {:>25}: {}", desc, sw),
                );

                Ok(())
            }
        }
    }
}
