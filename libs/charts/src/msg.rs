//! Client/server messages.

prelude! {}
use filter::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChartSettingsMsg {
    ToggleVisible,
    ChangeTitle(String),
    SetDisplayMode(chart::settings::DisplayMode),
}

impl ChartSettingsMsg {
    pub fn toggle_visible<Res>(uid: uid::ChartUid) -> Res
    where
        (uid::ChartUid, Self): Into<Res>,
    {
        (uid, Self::ToggleVisible).into()
    }
    pub fn set_display_mode<Res>(uid: uid::ChartUid, mode: chart::settings::DisplayMode) -> Res
    where
        (uid::ChartUid, Self): Into<Res>,
    {
        (uid, Self::SetDisplayMode(mode)).into()
    }
    pub fn change_title<Res>(uid: uid::ChartUid, title: impl Into<String>) -> Res
    where
        (uid::ChartUid, Self): Into<Res>,
    {
        (uid, Self::ChangeTitle(title.into())).into()
    }
}

impl fmt::Display for ChartSettingsMsg {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ToggleVisible => write!(fmt, "toggle visible"),
            Self::SetDisplayMode(mode) => write!(fmt, "set display mode: {}", mode.desc()),
            Self::ChangeTitle(title) => write!(fmt, "title: {}", title),
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
        pub fn to_bytes(&self) -> Res<Vec<u8>> {
            Ok(bincode::serialize(self)?)
        }

        pub fn from_bytes(bytes: &[u8]) -> Res<Self> {
            Ok(bincode::deserialize(bytes)?)
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
        ChartUpdate { uid: uid::ChartUid, msg: ChartMsg },
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

    impl From<(uid::ChartUid, ChartMsg)> for ChartsMsg {
        fn from((uid, msg): (uid::ChartUid, ChartMsg)) -> Self {
            Self::ChartUpdate { uid, msg }
        }
    }
    impl From<(uid::ChartUid, ChartSettingsMsg)> for ChartsMsg {
        fn from((uid, msg): (uid::ChartUid, ChartSettingsMsg)) -> Self {
            Self::ChartUpdate {
                uid,
                msg: msg.into(),
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum ChartMsg {
        SettingsUpdate(ChartSettingsMsg),
    }
    impl fmt::Display for ChartMsg {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Self::SettingsUpdate(_) => write!(fmt, "settings update"),
            }
        }
    }

    impl From<ChartSettingsMsg> for ChartMsg {
        fn from(msg: ChartSettingsMsg) -> Self {
            Self::SettingsUpdate(msg)
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

    impl Into<yew::format::Text> for Msg {
        fn into(self) -> yew::format::Text {
            anyhow::bail!("trying to encode a message as text, only binary is supported")
        }
    }
    impl Into<yew::format::Binary> for Msg {
        fn into(self) -> yew::format::Binary {
            match self.to_bytes() {
                Ok(bytes) => Ok(bytes),
                Err(e) => anyhow::bail!("{}", e),
            }
        }
    }

    impl From<FiltersMsg> for Msg {
        fn from(msg: FiltersMsg) -> Self {
            Self::Filters(msg)
        }
    }
    impl From<ChartsMsg> for Msg {
        fn from(msg: ChartsMsg) -> Self {
            Self::Charts(msg)
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
        pub fn alert(msg: impl Into<String>) -> Self {
            Self::Alert { msg: msg.into() }
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

        pub fn to_bytes(&self) -> Res<Vec<u8>> {
            Ok(bincode::serialize(self)?)
        }

        pub fn from_bytes(bytes: &[u8]) -> Res<Self> {
            Ok(bincode::deserialize(bytes)?)
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

    impl From<ChartsMsg> for Msg {
        fn from(msg: ChartsMsg) -> Self {
            Self::Charts(msg)
        }
    }
    impl From<FiltersMsg> for Msg {
        fn from(msg: FiltersMsg) -> Self {
            Self::Filters(msg)
        }
    }

    /// Messages for the charts of the client.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum ChartsMsg {
        /// Creates a new chart.
        NewChart(chart::ChartSpec, chart::ChartSettings),
        /// Message for a specific chart.
        Chart { uid: uid::ChartUid, msg: ChartMsg },
        /// A new collection of points, overwrites existing points.
        NewPoints {
            points: point::ChartPoints,
            refresh_filters: bool,
        },
        /// Some points to append to existing points.
        AddPoints(point::ChartPoints),
    }
    impl ChartsMsg {
        /// Constructor for `NewChart`.
        pub fn new_chart(spec: chart::ChartSpec, settings: chart::ChartSettings) -> Msg {
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
        pub fn new_points(uid: uid::ChartUid, points: point::Points) -> Msg {
            Msg::charts(ChartsMsg::Chart {
                uid,
                msg: Self::NewPoints(points),
            })
        }
        /// List of points to append.
        pub fn points(uid: uid::ChartUid, points: point::Points) -> Msg {
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
            everything: FilterSpec,
            filters: Vec<Filter>,
            catch_all: FilterSpec,
        },

        /// Updates all the specs.
        UpdateSpecs(Map<uid::LineUid, FilterSpec>),
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
        pub fn update_specs(specs: Map<uid::LineUid, FilterSpec>) -> Msg {
            Self::UpdateSpecs(specs).into()
        }
    }

    impl Into<Res<Msg>> for RawMsg {
        fn into(self) -> Res<Msg> {
            let res = match self {
                RawMsg::Binary(res_bytes) => {
                    let bytes = res_bytes
                        .map_err(err::Err::from)
                        .chain_err(|| "while retrieving message from the server")?;
                    Msg::from_bytes(&bytes)
                }
                RawMsg::Text(res_string) => {
                    let _ = res_string
                        .map_err(err::Err::from)
                        .chain_err(|| "while retrieving message from the server")?;
                    bail!(
                        "trying to build message from text representation, \
                        only binary format is supported"
                    )
                }
            };
            res.chain_err(|| "while parsing a message from the server")
        }
    }

    /// A raw message from the server.
    #[derive(Debug, Clone)]
    pub enum RawMsg {
        /// Binary version.
        Binary(Result<Vec<u8>, String>),
        /// String version.
        Text(Result<String, String>),
    }
    impl From<yew::format::Binary> for RawMsg {
        fn from(data: yew::format::Binary) -> Self {
            RawMsg::Binary(data.map_err(|e| e.to_string()))
        }
    }
    impl From<yew::format::Text> for RawMsg {
        fn from(data: yew::format::Text) -> Self {
            RawMsg::Text(data.map_err(|e| e.to_string()))
        }
    }
}
