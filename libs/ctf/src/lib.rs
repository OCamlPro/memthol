//! Frontend for memtrace's CTF format.

#![deny(missing_docs)]

#[macro_use]
mod macros;

/// Memtrace version of the parser realized by this crate.
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

/// Shorthand trait for the signature of event-handling functions.
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

    /// Type of an encoded location.
    type EncodedLoc = u64;
    /// Maps encoded locations to vectors of locations.
    type LocMap = HMap<EncodedLoc, Vec<Loc>>;

    pub struct TraceBuilder {
        last_trace: Vec<CLoc>,
        last_trace_len: usize,
        last_trace_cached: Option<Trace>,
        cursor: usize,
        cursor_count_minus: usize,
    }
    impl TraceBuilder {
        fn new() -> Self {
            Self {
                last_trace: Vec::with_capacity(32),
                last_trace_cached: None,
                last_trace_len: 0,
                cursor: 0,
                cursor_count_minus: 0,
            }
        }
        #[inline]
        fn reset(&mut self) {
            self.cursor = 0;
            self.cursor_count_minus = 0;
        }

        #[inline]
        fn build_trace(
            &mut self,
            factory: &mut mem::Factory,
            loc_map: &LocMap,
            common_pref_len: usize,
            trace: Vec<usize>,
        ) -> Res<Trace> {
            debug_assert_eq!(self.cursor, 0);
            debug_assert_eq!(self.cursor_count_minus, 0);

            let trace_len = trace.len();

            let trace = if common_pref_len == trace_len && trace_len == self.last_trace_len {
                if let Some(trace) = self.last_trace_cached.clone() {
                    trace
                } else if common_pref_len == 0 {
                    let mut trace = self.last_trace.clone();
                    trace.shrink_to_fit();
                    let trace = factory.register_trace(trace);
                    self.last_trace_cached = Some(trace.clone());
                    trace
                } else {
                    bail!("[build_trace] illegal internal state: no previous trace exists")
                }
            } else {
                'drain_trace: for (idx, code) in trace.into_iter().enumerate() {
                    let sub_trace = loc_map
                        .get(&(code as u64))
                        .ok_or_else(|| format!("[ctf parser] unknown location code `{}`", code))?;

                    match idx.cmp(&common_pref_len) {
                        std::cmp::Ordering::Less => {
                            let mut to_skip = sub_trace.len();
                            'update_cursor: loop {
                                let cursor_cnt = self.last_trace[self.cursor].cnt;
                                let left_at_cursor = cursor_cnt - self.cursor_count_minus;
                                match left_at_cursor.cmp(&to_skip) {
                                    std::cmp::Ordering::Less => {
                                        to_skip -= left_at_cursor;
                                        self.cursor += 1;
                                        self.cursor_count_minus = 0;
                                        continue 'update_cursor;
                                    }
                                    std::cmp::Ordering::Equal => {
                                        self.cursor += 1;
                                        self.cursor_count_minus = 0;
                                        continue 'drain_trace;
                                    }
                                    std::cmp::Ordering::Greater => {
                                        self.cursor_count_minus += to_skip;
                                        continue 'drain_trace;
                                    }
                                }
                            }
                        }
                        std::cmp::Ordering::Equal => {
                            if self.cursor_count_minus == 0 {
                                self.last_trace.truncate(self.cursor)
                            } else {
                                self.last_trace.truncate(self.cursor + 1);
                                let last = self.last_trace.last_mut().ok_or_else(|| {
                                    format!("[build_trace] illegal internal state")
                                })?;
                                debug_assert!(last.cnt >= self.cursor_count_minus);
                                last.cnt = self.cursor_count_minus
                            }
                        }
                        std::cmp::Ordering::Greater => (),
                    }

                    for loc in sub_trace {
                        if let Some(cloc) = self.last_trace.last_mut() {
                            if loc == &cloc.loc {
                                cloc.cnt += 1
                            } else {
                                self.last_trace.push(CLoc::new(loc.clone(), 1));
                            }
                        } else {
                            self.last_trace.push(CLoc::new(loc.clone(), 1))
                        };
                    }
                }

                self.reset();

                self.last_trace_len = trace_len;
                let mut trace = self.last_trace.clone();
                trace.shrink_to_fit();
                let trace = factory.register_trace(trace);
                self.last_trace_cached = Some(trace.clone());
                trace
            };

            Ok(trace)
        }
    }

    fn date_from_microsecs(date: crate::prelude::Clock) -> time::Date {
        time::Date::from_micros(convert(date, "date_from_microsecs"))
    }

    /// Parses a CTF file (memtrace format).
    pub fn parse<'a, F>(
        bytes: &[u8],
        mut factory: &mut F,
        mut bytes_progress: impl FnMut(usize),
        init_action: impl FnOnce(&mut F, Init),
        mut new_action: impl FnMut(&mut F, Alloc),
        mut dead_action: impl FnMut(&mut F, time::SinceStart, uid::Alloc),
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
                pub alloc_action => "allocation action",
            }
        }
        let mut prof = Prof::new();
        prof.total.start();

        let mut trace_builder = TraceBuilder::new();

        // Maps location encoded identifiers to actual locations.
        let mut loc_id_to_loc = LocMap::with_capacity(1001);

        parse! {
            bytes => |mut parser| {
                prof.basic_parsing.start();

                let header = parser.header();

                // Start time of the run, used for init and to compute the time-since-start of all
                // events.
                let start_time = date_from_microsecs(header.timestamp.begin);
                // let end_time = date_from_microsecs(header.header.timestamp.end).sub(start_time)?;

                // Init info.
                let init = parser.trace_info().to_init(start_time);

                init_action(factory, init);
                prof.basic_parsing.stop();

                // Iterate over the packet of the trace.
                while let Some(mut packet_parser) = prof.packet_parsing.time(
                    || parser.next_packet()
                )? {
                    if packet_parser.header().id() % 10 == 9 {
                        bytes_progress(packet_parser.real_position().0);
                    }

                    // Iterate over the events of the packet.
                    while let Some((clock, event)) = prof.event_parsing.time(
                        || packet_parser.next_event()
                    )? {
                        use crate::ast::event::Event;

                        match event {
                            Event::Alloc(crate::ast::event::Alloc {
                                id: uid, backtrace, len, common_pref_len, nsamples, ..
                            }) => {
                                let trace = {
                                    prof.trace_building.time(|| trace_builder.build_trace(
                                        factory,
                                        &loc_id_to_loc,
                                        common_pref_len,
                                        backtrace,
                                    ))?
                                };

                                prof.alloc.start();

                                // Build the allocation.
                                let alloc = {
                                    let time_since_start =
                                        date_from_microsecs(clock) - start_time;
                                    let labels = factory.empty_labels();
                                    let alloc = Alloc::new(
                                        uid,
                                        AllocKind::Minor,
                                        convert(len, "ctf parser: alloc size"),
                                        trace,
                                        labels,
                                        time_since_start,
                                        None
                                    ).nsamples(nsamples as u32);
                                    alloc
                                };

                                prof.alloc.stop();

                                prof.alloc_action.time(|| new_action(factory, alloc))
                            },

                            Event::Collection(alloc_uid) => {
                                prof.dead.start();

                                let uid = uid::Alloc::from(alloc_uid);
                                let timestamp = date_from_microsecs(clock) - start_time;

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
