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

impl Default for DisplayMode {
    fn default() -> Self {
        Self::Expanded(0)
    }
}

impl DisplayMode {
    const MAX: u8 = 0;

    /// Constructor.
    pub fn new() -> Self {
        Self::Collapsed
    }

    /// Number of header lines for this display mode.
    pub fn line_count(&self) -> usize {
        match self {
            Self::Collapsed => 0,
            Self::Expanded(_) => 1,
        }
    }

    /// True if the display mode can be augmented.
    pub fn can_inc(self) -> bool {
        match self {
            Self::Expanded(n) if n >= Self::MAX => false,
            Self::Collapsed | Self::Expanded(_) => true,
        }
    }

    /// True if the display mode can be reduced.
    pub fn can_dec(self) -> bool {
        match self {
            Self::Collapsed => false,
            Self::Expanded(_) => true,
        }
    }

    /// Augments the display mode.
    pub fn inc(&mut self) -> bool {
        match self {
            Self::Collapsed => {
                *self = Self::Expanded(0);
                true
            }
            Self::Expanded(ref mut n) => {
                if *n < Self::MAX {
                    *n = *n + 1;
                    true
                } else {
                    false
                }
            }
        }
    }

    /// Decreases the display mode.
    pub fn dec(&mut self) -> bool {
        match self {
            Self::Collapsed => false,
            Self::Expanded(0) => {
                *self = Self::Collapsed;
                true
            }
            Self::Expanded(ref mut n) => {
                debug_assert!(*n > 0);
                *n = *n + 1;
                true
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
    link: Link,
    /// Duration of the run.
    run_duration: time::SinceStart,

    /// Global charts settings.
    charts_settings: Memory<charts::chart::settings::Charts>,
}

impl Settings {
    /// Constructor.
    pub fn new(link: Link) -> Self {
        Self {
            display_mode: DisplayMode::default(),
            charts_settings: Memory::default(),
            link,
            run_duration: time::SinceStart::zero(),
        }
    }

    /// True if the settings menu can be expanded.
    pub fn can_expand(&self) -> bool {
        self.display_mode.can_inc()
    }
    /// True if the settings menu can be collapsed.
    pub fn can_collapse(&self) -> bool {
        self.display_mode.can_dec()
    }

    /// Number of header lines for the current display mode.
    pub fn line_count(&self) -> usize {
        self.display_mode.line_count()
    }

    /// Update the current time since the run started.
    pub fn set_run_duration(&mut self, run_duration: time::SinceStart) {
        self.run_duration = run_duration
    }

    /// Renders the settings menu.
    pub fn render(&self, model: &Model) -> Html {
        match self.display_mode {
            DisplayMode::Collapsed => html! {},
            DisplayMode::Expanded(_) => self.render_0(model),
        }
    }

    /// Renders the settings menu in display mode 0.
    pub fn render_0(&self, model: &Model) -> Html {
        html! {
            <>
                {self.time_window_line(model)}
            </>
        }
    }

    /// True if the current settings are different form the server ones.
    pub fn has_changed(&self) -> bool {
        // Exhaustive deconstruction so that this breaks when new fields are added to `Self`.
        //
        // DO NOT USE `..` HERE.
        let Self {
            display_mode: _,
            link: _,
            run_duration: _,

            charts_settings,
        } = self;
        charts_settings.has_changed()
    }

    /// True if the current settings are legal.
    pub fn is_legal(&self) -> Option<String> {
        // Exhaustive deconstruction so that this breaks when new fields are added to `Self`.
        //
        // DO NOT USE `..` HERE.
        let Self {
            display_mode: _,
            link: _,
            run_duration: _,

            charts_settings,
        } = self;

        charts_settings.get().is_legal()
    }

    /// Generates the save/undo buttons, if needed.
    pub fn buttons(&self) -> Html {
        define_style! {
            RIGHT = {
                float(right),
            };
        }

        if self.has_changed() {
            html! {
                <>
                    <div
                        style = RIGHT
                    >
                        { layout::button::img::undo(
                            Some(header::HEADER_INFO_LINE_BUTTON_HEIGHT_PX),
                            "header_settings_undo",
                            Some(self.link.callback(move |_| msg::Msg::from(Msg::Revert))),
                            "revert changes",
                        ) }
                    </div>
                    <div
                        style = RIGHT
                    >
                        { layout::button::img::check(
                            Some(header::HEADER_INFO_LINE_BUTTON_HEIGHT_PX),
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
        const BORDER_HEIGHT_PX: usize = 2;
        const LINE_HEIGHT_PX: usize = header::HEADER_LINE_HEIGHT_PX - BORDER_HEIGHT_PX;
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
            SETTINGS_LINE = {
                border(bottom, {BORDER_HEIGHT_PX}px, {layout::LIGHT_BLUE_FG}),
                height({LINE_HEIGHT_PX}px),
            };
        }

        let (lb, ub) = (
            self.charts_settings
                .get()
                .time_windopt()
                .lbound
                .unwrap_or_else(time::SinceStart::zero),
            self.charts_settings
                .get()
                .time_windopt()
                .ubound
                .unwrap_or_else(|| self.run_duration),
        );
        let step = self.run_duration / 10;

        header::Header::three_part_line_with(
            &*SETTINGS_LINE,
            html! {},
            header::Header::center(html! {
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
                            step,
                            Some(lb),
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
                            step,
                            Some(ub),
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
            Msg::Expand => {
                let changed = self.display_mode.inc();
                Ok(changed)
            }
            Msg::Collapse => {
                let changed = self.display_mode.dec();
                Ok(changed)
            }
            Msg::Revert => {
                self.charts_settings.reset();
                Ok(true)
            }
            Msg::Save => {
                if self.has_changed() {
                    if let Some(mut errors) = self.is_legal() {
                        errors.push_str("\n😿 cannot apply these settings , please fix them");
                        self.link.send_message(msg::Msg::err(errors));
                        Ok(false)
                    } else {
                        self.link.send_message(msg::Msg::ToServer(
                            msg::to_server::ChartsMsg::settings(self.charts_settings.get().clone())
                                .into(),
                        ));
                        self.charts_settings.overwrite_reference();
                        Ok(true)
                    }
                } else {
                    Ok(false)
                }
            }
        };
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
    /// Expands the settings.
    Expand,
    /// Collapses the settings.
    Collapse,
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
                Self::Expand => write!(fmt, "expand"),
                Self::Collapse => write!(fmt, "collapse"),
            }
        }
    }
}
