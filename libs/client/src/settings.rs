//! Settings part of the client.

prelude! {}

/// Linear display mode.
#[derive(Debug, Clone, Copy)]
pub enum DisplayMode {
    /// Collapsed.
    Collapsed,
    /// Expanded for some *depth*.
    Expanded(u8),
}
impl DisplayMode {
    const MAX: u8 = 0;

    /// Constructor.
    pub fn new() -> Self {
        Self::Collapsed
    }

    /// Augments the display mode.
    pub fn inc(&mut self) {
        *self = match *self {
            Self::Collapsed => Self::Expanded(0),
            Self::Expanded(mut n) => {
                if n < Self::MAX {
                    n += 1
                }
                Self::Expanded(n)
            }
        }
    }

    /// Decreases the display mode.
    pub fn dec(&mut self) {
        *self = match *self {
            Self::Collapsed | Self::Expanded(0) => Self::Collapsed,
            Self::Expanded(mut n) => {
                debug_assert!(n > 0);
                n -= 1;
                Self::Expanded(n)
            }
        }
    }
}

/// Stores the settings state.
#[derive(Debug, Clone)]
pub struct Settings {
    /// Current display mode.
    display_mode: DisplayMode,

    /// Global charts settings.
    charts_settings: Memory<charts::chart::settings::Charts>,
}

impl Settings {
    /// Constructor.
    pub fn new() -> Self {
        Self {
            display_mode: DisplayMode::new(),
            charts_settings: Memory::default(),
        }
    }

    /// Renders the settings menu.
    pub fn render(&self, model: &Model) -> Html {
        html! {
            <>
                {self.time_window_line(model)}
            </>
        }
    }

    /// Generates the time-window line.
    pub fn time_window_line(&self, model: &Model) -> Html {
        define_style! {
            LEFT = {
                float(left),
            };
            INPUT_CONTAINER = {
                extends_style(&*LEFT),
                width(10%),
                height(80%),
            };
        }

        layout::header::three_part_line(
            html! {},
            layout::header::center(html! {
                <div>
                    <div
                        style = LEFT
                    >
                        { "time window (seconds) " }
                        { layout::header::code("[ ") }
                    </div>

                    <div
                        style = INPUT_CONTAINER
                    >
                        { layout::input::since_start_opt_input(
                            model,
                            "1",
                            &self.charts_settings.get().time_windopt().lbound,
                            |since_start_opt| msg_of_res(
                                since_start_opt.map(|lb| Msg::TimeWindowLb(lb).into())
                            )
                        ) }
                    </div>

                    <div
                        style = LEFT
                    >
                        { layout::header::code(", ") }
                    </div>

                    <div
                        style = INPUT_CONTAINER
                    >
                        { layout::input::since_start_opt_input(
                            model,
                            "1",
                            &self.charts_settings.get().time_windopt().ubound,
                            |since_start_opt| msg_of_res(
                                since_start_opt.map(|ub| Msg::TimeWindowUb(ub).into())
                            )
                        ) }
                    </div>

                    <div
                        style = LEFT
                    >
                        { layout::header::code(" ]") }
                    </div>
                </div>
            }),
            html! {},
        )
    }

    /// Updates itself given a settings message.
    pub fn update(&mut self, msg: Msg) -> Res<ShouldRender> {
        log::info!(
            "updating settings: [{:?}, {:?}]",
            self.charts_settings.get().time_windopt().lbound,
            self.charts_settings.get().time_windopt().ubound,
        );
        let res = match msg {
            Msg::TimeWindowLb(lb) => {
                let lbound = &mut self.charts_settings.get_mut().time_windopt_mut().lbound;
                if lbound != &lb {
                    *lbound = lb;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Msg::TimeWindowUb(ub) => {
                let ubound = &mut self.charts_settings.get_mut().time_windopt_mut().ubound;
                if ubound != &ub {
                    *ubound = ub;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        };
        log::info!(
            "done updating settings: [{:?}, {:?}]",
            self.charts_settings.get().time_windopt().lbound,
            self.charts_settings.get().time_windopt().ubound,
        );
        res
    }
}

/// Messages acting on the global charts settings.
#[derive(Clone, Debug)]
pub enum Msg {
    /// Updates the time window's lower bound.
    TimeWindowLb(Option<time::SinceStart>),
    /// Updates the time window's upper bound.
    TimeWindowUb(Option<time::SinceStart>),
}
base::implement! {
    impl Msg {
        Display {
            |&self, fmt| match self {
                Self::TimeWindowLb(lb) => write!(
                    fmt,
                    "time window lb: {}",
                    lb
                        .map(|lb| lb.to_string())
                        .unwrap_or("_".into()),
                ),
                Self::TimeWindowUb(ub) => write!(
                    fmt,
                    "time window ub: {}",
                    ub
                        .map(|ub| ub.to_string())
                        .unwrap_or("_".into()),
                ),
            }
        }
    }
}
