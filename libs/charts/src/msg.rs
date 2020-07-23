//! Client/server messages.

prelude! {}
use filter::*;

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

    /// Operations over charts.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum ChartsMsg {
        /// Creates a new chart.
        New(chart::axis::XAxis, chart::axis::YAxis),
        /// Reloads all charts.
        Reload,
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
            match self.as_json() {
                Ok(res) => Ok(res),
                Err(e) => anyhow::bail!("{}", e.pretty()),
            }
        }
    }
    impl Into<yew::format::Binary> for Msg {
        fn into(self) -> yew::format::Binary {
            match self.as_json() {
                Ok(res) => Ok(res.into_bytes()),
                Err(e) => anyhow::bail!("{}", e.pretty()),
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
        /// A message for the charts.
        Charts(ChartsMsg),
        /// A filter operation.
        Filters(FiltersMsg),
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
        /// Constructor for `Charts`.
        pub fn charts(msg: ChartsMsg) -> Self {
            Self::Charts(msg)
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
                    let bytes: Res<Vec<u8>> = res_bytes.map_err(|e| e.into());
                    let bytes = bytes.chain_err(|| "while retrieving message from the server")?;
                    Msg::from_json_bytes(&bytes)
                }
                RawMsg::Text(res_string) => {
                    let string: Res<String> = res_string.map_err(|e| e.into());
                    let string = string.chain_err(|| "while retrieving message from the server")?;
                    Msg::from_json(&string)
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
