//! Client/server messages.

prelude! {}
use filter::*;

/// Chart settings message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChartSettingsMsg {
    /// Toggles a chart's visibility.
    ToggleVisible,
    /// Changes the title of a chart.
    ChangeTitle(String),
    /// Changes the display mode of a chart.
    SetDisplayMode(chart::settings::DisplayMode),
    /// Changes the resolution of a chart.
    SetResolution(chart::settings::Resolution),
}

impl ChartSettingsMsg {
    /// Toggles a chart's visibility.
    pub fn toggle_visible<Res>(uid: uid::Chart) -> Res
    where
        (uid::Chart, Self): Into<Res>,
    {
        (uid, Self::ToggleVisible).into()
    }

    /// Changes the display mode of a chart.
    pub fn set_display_mode<Res>(uid: uid::Chart, mode: chart::settings::DisplayMode) -> Res
    where
        (uid::Chart, Self): Into<Res>,
    {
        (uid, Self::SetDisplayMode(mode)).into()
    }

    /// Changes the title of a chart.
    pub fn change_title<Res>(uid: uid::Chart, title: impl Into<String>) -> Res
    where
        (uid::Chart, Self): Into<Res>,
    {
        (uid, Self::ChangeTitle(title.into())).into()
    }

    /// Changes the resolution of a chart.
    pub fn set_resolution<Res>(
        uid: uid::Chart,
        resolution: impl Into<chart::settings::Resolution>,
    ) -> Res
    where
        (uid::Chart, Self): Into<Res>,
    {
        (uid, Self::SetResolution(resolution.into())).into()
    }
}

impl fmt::Display for ChartSettingsMsg {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ToggleVisible => write!(fmt, "toggle visible"),
            Self::SetDisplayMode(mode) => write!(fmt, "set display mode: {}", mode.desc()),
            Self::ChangeTitle(title) => write!(fmt, "change title: {}", title),
            Self::SetResolution(resolution) => write!(fmt, "set resolution: {}", resolution),
        }
    }
}

/// Messages from the client to the server.
pub mod to_server {
    use super::*;

    /// A list of messages from the client to the server.
    pub type Msgs = Vec<Msg>;

    /// Messages from the client to the server.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum Msg {
        /// Operations over charts.
        Charts(ChartsMsg),

        /// Operation over filters.
        Filters(FiltersMsg),
    }
    impl fmt::Display for Msg {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Self::Charts(msg) => write!(fmt, "charts({})", msg),
                Self::Filters(msg) => write!(fmt, "filters({})", msg),
            }
        }
    }

    impl Msg {
        /// Encodes the message as bytes.
        pub fn to_bytes(&self) -> Res<Vec<u8>> {
            Ok(base::bincode::serialize(self)?)
        }

        /// Decodes the message from bytes.
        pub fn from_bytes(bytes: &[u8]) -> Res<Self> {
            Ok(base::bincode::deserialize(bytes)?)
        }
    }

    base::implement! {
        impl Msg {
            From {
                from (uid::Chart, ChartMsg) => |pair| Self::Charts(ChartsMsg::from(pair)),
                from (uid::Chart, ChartSettingsMsg) => |pair| Self::Charts(ChartsMsg::from(pair)),
                from FiltersMsg => |msg| Self::Filters(msg),
                from ChartsMsg => |msg| Self::Charts(msg),
            }

            Into {
                to yew::format::Text => |self| anyhow::bail!(
                    "trying to encode a message as text, only binary is supported"
                ),
                to yew::format::Binary => |self| match self.to_bytes() {
                    Ok(bytes) => Ok(bytes),
                    Err(e) => anyhow::bail!("{}", e),
                }
            }
        }
    }

    /// Operations over charts.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum ChartsMsg {
        /// Creates a new chart.
        New(chart::axis::XAxis, chart::axis::YAxis),
        /// Reloads all charts.
        Reload,
        /// An update for a specific chart.
        ChartUpdate {
            /// UID of the chart the message is for.
            uid: uid::Chart,
            /// Actual message.
            msg: ChartMsg,
        },
    }
    impl fmt::Display for ChartsMsg {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Self::New(_, _) => write!(fmt, "new chart"),
                Self::Reload => write!(fmt, "reload"),
                Self::ChartUpdate { uid, msg } => write!(fmt, "update({}, {})", uid, msg),
            }
        }
    }
    impl ChartsMsg {
        /// Constructs a chart creation message.
        pub fn new(x: chart::axis::XAxis, y: chart::axis::YAxis) -> Msg {
            Self::New(x, y).into()
        }
        /// Reloads all charts.
        pub fn reload() -> Msg {
            Self::Reload.into()
        }
    }

    base::implement! {
        impl ChartsMsg {
            From {
                from (uid::Chart, ChartMsg) => |(uid, msg)| Self::ChartUpdate { uid, msg },
                from (uid::Chart, ChartSettingsMsg) => |(uid, msg)| Self::ChartUpdate {
                    uid,
                    msg: msg.into(),
                }
            }
        }
    }

    /// A message for a specific chart.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum ChartMsg {
        /// Settings update.
        SettingsUpdate(ChartSettingsMsg),
    }
    impl fmt::Display for ChartMsg {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Self::SettingsUpdate(_) => write!(fmt, "settings update"),
            }
        }
    }

    base::implement! {
        impl ChartMsg {
            From {
                from ChartSettingsMsg => |msg| Self::SettingsUpdate(msg)
            }
        }
    }

    /// Operations over filters.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum FiltersMsg {
        /// Requests a new filter.
        ///
        /// This will cause the server to generate a new filter to send to the client (*via*
        /// [`FiltersMsg::Add`]). The server will **not** register the filter in any way, this will
        /// happen when/if the user saves the modifications.
        ///
        /// [`FiltersMsg::Add`]: ../to_client/enum.FiltersMsg.html#variant.Add
        /// (The Add message)
        RequestNew,

        /// Requests the current server-side list of filters.
        Revert,

        /// Updates all the filters.
        UpdateAll {
            /// New specificationfor the "everything" filter.
            everything: filter::FilterSpec,
            /// New filters.
            filters: Vec<Filter>,
            /// New specification for the "catch-all" filter.
            catch_all: filter::FilterSpec,
        },
    }
    impl fmt::Display for FiltersMsg {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Self::RequestNew => write!(fmt, "request new"),
                Self::Revert => write!(fmt, "revert"),
                Self::UpdateAll { .. } => write!(fmt, "update all"),
            }
        }
    }

    impl FiltersMsg {
        /// Requests a new filter.
        pub fn request_new() -> Msg {
            Self::RequestNew.into()
        }
        /// Requests the current server-side list of filters.
        pub fn revert() -> Msg {
            Self::Revert.into()
        }

        /// Updates all the filters.
        pub fn update_all(
            everything: filter::FilterSpec,
            filters: Vec<Filter>,
            catch_all: filter::FilterSpec,
        ) -> Msg {
            Self::UpdateAll {
                everything,
                filters,
                catch_all,
            }
            .into()
        }
    }
}

/// Messages from the server to the client.
pub mod to_client {
    use super::*;

    /// A list of messages from the server to the client.
    pub type Msgs = Vec<Msg>;

    /// Messages from the server to the client.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum Msg {
        /// Info about the current allocation data.
        Info,
        /// An alert.
        Alert {
            /// Alert message.
            msg: String,
            /// True if the error is fatal.
            fatal: bool,
        },
        /// Loading progress.
        ///
        /// Sent by the server when it is loading data, *i.e.* not ready to actually produce charts
        /// yet.
        LoadProgress(LoadInfo),
        /// Allocation statistics.
        AllocStats(AllocStats),
        /// Sent by the server when it is done loading dumps.
        DoneLoading,
        /// A message for the charts.
        Charts(ChartsMsg),
        /// A filter operation.
        Filters(FiltersMsg),
        /// Some filter statistics.
        FilterStats(filter::stats::AllFilterStats),
    }
    impl Msg {
        /// Constructor for `Info`.
        pub fn info() -> Self {
            Self::Info
        }
        /// Constructor for `Alert`.
        pub fn alert(msg: impl Into<String>, fatal: bool) -> Self {
            Self::Alert {
                msg: msg.into(),
                fatal,
            }
        }
        /// Constructor for chart messages.
        pub fn charts(msg: ChartsMsg) -> Self {
            Self::Charts(msg)
        }
        /// Constructor for a load progress message.
        pub fn load_progress(info: LoadInfo) -> Self {
            Self::LoadProgress(info)
        }
        /// Constructor for an allocation-statistics message.
        pub fn alloc_stats(stats: AllocStats) -> Self {
            Self::AllocStats(stats)
        }
        /// Constructor for a filter-statistics message.
        pub fn filter_stats(stats: filter::stats::AllFilterStats) -> Self {
            Self::FilterStats(stats)
        }

        /// Encodes the message as bytes.
        pub fn to_bytes(&self) -> Res<Vec<u8>> {
            Ok(base::bincode::serialize(self)?)
        }

        /// Decodes the message from bytes.
        pub fn from_bytes(bytes: &[u8]) -> Res<Self> {
            Ok(base::bincode::deserialize(bytes)?)
        }

        /// True if the message is a minor message.
        ///
        /// *Minor messages* are all messages that do not act on charts or filters directly.
        pub fn is_minor(&self) -> bool {
            match self {
                Self::Charts(_) | Self::Filters(_) => false,
                Self::Info
                | Self::Alert { .. }
                | Self::LoadProgress(_)
                | Self::AllocStats(_)
                | Self::DoneLoading
                | Self::FilterStats(_) => true,
            }
        }
    }

    impl fmt::Display for Msg {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Self::Info => "info".fmt(fmt),
                Self::Alert { .. } => "alert".fmt(fmt),
                Self::Charts(msg) => write!(fmt, "charts({})", msg),
                Self::LoadProgress(_) => "load progress".fmt(fmt),
                Self::AllocStats(_) => "alloc stats".fmt(fmt),
                Self::FilterStats(_) => "filter stats".fmt(fmt),
                Self::DoneLoading => "done loading".fmt(fmt),
                Self::Filters(_) => "filter".fmt(fmt),
            }
        }
    }

    base::implement! {
        impl Msg {
            From {
                from ChartsMsg => |msg| Self::Charts(msg),
                from FiltersMsg => |msg| Self::Filters(msg),
            }
        }
    }

    /// Messages for the charts of the client.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum ChartsMsg {
        /// Creates a new chart.
        NewChart(chart::ChartSpec, settings::Chart),
        /// Message for a specific chart.
        Chart {
            /// UID of the chart this message is for.
            uid: uid::Chart,
            /// Actual chart message.
            msg: ChartMsg,
        },
        /// A new collection of points, overwrites existing points.
        NewPoints {
            /// New points.
            points: point::ChartPoints,
            /// If true, refresh all filters.
            refresh_filters: bool,
        },
        /// Some points to append to existing points.
        AddPoints(point::ChartPoints),
    }
    impl ChartsMsg {
        /// Constructor for `NewChart`.
        pub fn new_chart(spec: chart::ChartSpec, settings: settings::Chart) -> Msg {
            Msg::charts(Self::NewChart(spec, settings))
        }
        /// Constructor for `NewPoints`.
        pub fn new_points(points: point::ChartPoints, refresh_filters: bool) -> Msg {
            Msg::charts(Self::NewPoints {
                points,
                refresh_filters,
            })
        }
        /// Constructor for `AddPoints`.
        pub fn add_points(points: point::ChartPoints) -> Msg {
            Msg::charts(Self::AddPoints(points))
        }

        /// Constructs a `NewPoints` if `overwrite`, and a `AddPoints` otherwise.
        pub fn points(points: point::ChartPoints, overwrite: bool) -> Msg {
            if overwrite {
                Self::new_points(points, false)
            } else {
                Self::add_points(points)
            }
        }
    }

    impl fmt::Display for ChartsMsg {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Self::NewChart(_, _) => "new chart".fmt(fmt),
                Self::Chart { uid, msg } => write!(fmt, "chart({}, {})", uid, msg),
                Self::NewPoints { points, .. } => {
                    "new points:".fmt(fmt)?;
                    for (idx, (uid, points)) in points.iter().enumerate() {
                        write!(fmt, " ")?;
                        if idx > 0 {
                            write!(fmt, "| ")?
                        }
                        write!(fmt, "{}: {}, {}", uid, points.len(), points.point_count())?
                    }
                    Ok(())
                }
                Self::AddPoints(points) => {
                    "add points:".fmt(fmt)?;
                    for (idx, (uid, points)) in points.iter().enumerate() {
                        write!(fmt, " ")?;
                        if idx > 0 {
                            write!(fmt, "| ")?
                        }
                        write!(fmt, "{}: {}, {}", uid, points.len(), points.point_count())?
                    }
                    Ok(())
                }
            }
        }
    }

    /// Messages for a specific chart in the client.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum ChartMsg {
        /// A brand new list of points.
        ///
        /// Replaces all the points in a chart.
        NewPoints(point::Points),
        /// Some points to append.
        Points(point::Points),
    }

    impl ChartMsg {
        /// List of points overwriting the existing points.
        pub fn new_points(uid: uid::Chart, points: point::Points) -> Msg {
            Msg::charts(ChartsMsg::Chart {
                uid,
                msg: Self::NewPoints(points),
            })
        }
        /// List of points to append.
        pub fn points(uid: uid::Chart, points: point::Points) -> Msg {
            Msg::charts(ChartsMsg::Chart {
                uid,
                msg: Self::Points(points),
            })
        }
    }

    impl fmt::Display for ChartMsg {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Self::NewPoints(points) => write!(fmt, "{} new points", points.len()),
                Self::Points(points) => write!(fmt, "add {} points", points.len()),
            }
        }
    }

    /// Filter operations.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum FiltersMsg {
        /// Adds a filter.
        ///
        /// This message always comes in response to a [`FiltersMsg::RequestNew`] message for the
        /// server.
        ///
        /// [`FiltersMsg::RequestNew`]: ../to_server/enum.FiltersMsg.html#variant.RequestNew
        /// (The RequestNew message)
        Add(filter::Filter),

        /// Orders the client to revert all its filters.
        Revert {
            /// Specification for the `everything` filter.
            everything: FilterSpec,
            /// Specification for custom filters.
            filters: Vec<Filter>,
            /// Specification for the `catch_all` filter.
            catch_all: FilterSpec,
        },

        /// Updates all the specs.
        UpdateSpecs(BTMap<uid::Line, FilterSpec>),
    }
    impl FiltersMsg {
        /// Adds a filter.
        pub fn add(filter: filter::Filter) -> Msg {
            Self::Add(filter).into()
        }

        /// Orders the client to revert all its filters.
        pub fn revert(everything: FilterSpec, filters: Vec<Filter>, catch_all: FilterSpec) -> Msg {
            Self::Revert {
                everything,
                filters,
                catch_all,
            }
            .into()
        }

        /// Updates all the specs.
        pub fn update_specs(specs: BTMap<uid::Line, FilterSpec>) -> Msg {
            Self::UpdateSpecs(specs).into()
        }
    }

    /// A raw message from the server.
    #[derive(Debug, Clone)]
    pub enum RawMsg {
        /// Binary version.
        Binary(Result<Vec<u8>, String>),
    }

    base::implement! {
        impl RawMsg {
            From {
                from yew::format::Binary => |data| RawMsg::Binary(data.map_err(|e| e.to_string())),
                from yew::format::Text => |_| panic!(
                    "trying to decode a message from text, \
                    but only decoding from binary is supported"
                ),
            }

            Into {
                to Res<Msg> => |self| {
                    let res = match self {
                        RawMsg::Binary(res_bytes) => {
                            let bytes = res_bytes
                                .map_err(err::Error::from)
                                .chain_err(|| "while retrieving message from the server")?;
                            Msg::from_bytes(&bytes)
                        }
                    };
                    res.chain_err(|| "while parsing a message from the server")
                }
            }
        }
    }
}
