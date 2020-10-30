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
    /// Link to the model, to send messages.
    link: ComponentLink<Model>,
    /// Duration of the run.
    run_duration: time::SinceStart,

    /// Global charts settings.
    charts_settings: Memory<charts::chart::settings::Charts>,
}

impl Settings {
    /// Constructor.
    pub fn new(link: ComponentLink<Model>) -> Self {
        Self {
            display_mode: DisplayMode::new(),
            charts_settings: Memory::default(),
            link,
            run_duration: time::SinceStart::zero(),
        }
    }

    /// Update the current time since the run started.
    pub fn set_run_duration(&mut self, run_duration: time::SinceStart) {
        self.run_duration = run_duration
    }

    /// Renders the settings menu.
    pub fn render(&self, model: &Model) -> Html {
        html! {
            <>
                {self.time_window_line(model)}
            </>
        }
    }

    /// True if the current settings are different form the server ones.
    pub fn has_changed(&self) -> bool {
        // Exhaustive deconstruction so that this breaks when new fields are added to `Self`.
        let Self {
            display_mode: _,
            link: _,
            run_duration: _,

            charts_settings,
        } = self;
        charts_settings.has_changed()
    }

    /// Generates the save/undo buttons, if needed.
    pub fn buttons(&self) -> Html {
        define_style! {
            RIGHT = {
                float(right),
            };
        }

        const BUTTON_SIZE: usize =
            layout::header::HEADER_LINE_HEIGHT_PX - layout::header::HEADER_LINE_HEIGHT_PX / 10;

        if self.has_changed() {
            html! {
                <>
                    <div
                        style = RIGHT
                    >
                        { layout::button::img::undo(
                            Some(BUTTON_SIZE),
                            "header_settings_undo",
                            Some(self.link.callback(move |_| msg::Msg::from(Msg::Revert))),
                            "revert changes",
                        ) }
                    </div>
                    <div
                        style = RIGHT
                    >
                        { layout::button::img::check(
                            Some(BUTTON_SIZE),
                            "header_settings_apply",
                            Some(self.link.callback(move |_| msg::Msg::from(Msg::Save))),
                            "apply changes"
                        ) }
                    </div>
                </>
            }
        } else {
            html! {}
        }
    }

    /// Generates the time-window line.
    pub fn time_window_line(&self, model: &Model) -> Html {
        define_style! {
            LEFT = {
                float(left),
            };
            RIGHT = {
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
                        { layout::header::emph("time window") }
                        { " (seconds) " }
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
            Msg::TimeWindowLb(mut lb) => {
                lb = match lb {
                    Some(lb) if lb.is_zero() => None,
                    Some(lb) if lb >= self.run_duration => Some(self.run_duration),
                    lb => lb,
                };

                let lbound = &mut self.charts_settings.get_mut().time_windopt_mut().lbound;
                if lbound != &lb {
                    *lbound = lb;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Msg::TimeWindowUb(mut ub) => {
                ub = match ub {
                    Some(ub) if ub >= self.run_duration => None,
                    ub => ub,
                };

                let ubound = &mut self.charts_settings.get_mut().time_windopt_mut().ubound;
                if ubound != &ub {
                    *ubound = ub;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Msg::Revert => {
                self.charts_settings.reset();
                Ok(true)
            }
            Msg::Save => {
                if self.has_changed() {
                    self.link.send_message(msg::Msg::ToServer(
                        msg::to_server::ChartsMsg::settings(self.charts_settings.get().clone())
                            .into(),
                    ));
                    self.charts_settings.overwrite_reference();
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
    /// Reverts the settings.
    Revert,
    /// Saves the current settings.
    Save,
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
                Self::Revert => write!(fmt, "revert"),
                Self::Save => write!(fmt, "save"),
            }
        }
    }
}
