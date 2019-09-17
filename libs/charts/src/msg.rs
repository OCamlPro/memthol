//! Client/server messages.

use crate::base::*;
use filter::*;

/// Messages from the client to the server.
pub mod to_server {
    use super::*;

    /// Messages from the client to the server.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum Msg {
        /// Operation over filters.
        Filters {
            /// The operation.
            msg: FiltersMsg,
        },
    }

    /// Operations over filters.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum FiltersMsg {
        /// Adds a new filter.
        Add {
            /// Filter to add.
            filter: Filter,
        },
        /// Removes a filter.
        Rm {
            /// Index of the filter to remove.
            index: index::Filter,
        },

        /// Operation over a filter.
        Filter {
            /// Index of the filter.
            index: index::Filter,
            /// The operation.
            msg: FilterMsg,
        },
    }

    /// Operations over a filter.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum FilterMsg {
        /// Adds a new sub-filter.
        Add {
            /// Sub-filter to add.
            filter: SubFilter,
        },
        /// Removes a sub-filter.
        Rm {
            /// Index of the sub-filter.
            index: index::SubFilter,
        },
        /// Updates a sub-filter.
        Update {
            /// Index of the sub-filter to replace.
            index: index::SubFilter,
            /// New sub-filter.
            filter: SubFilter,
        },
    }
}

/// Messages from the server to the client.
pub mod to_client {
    use super::*;

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
        Charts {
            /// Chart message.
            msg: ChartsMsg,
        },
    }
    impl Msg {
        /// Constructor for `Info`.
        pub fn info() -> Self {
            Self::Info
        }
        /// Constructor for `Alert`.
        pub fn alert<S>(msg: S) -> Self
        where
            S: Into<String>,
        {
            Self::Alert { msg: msg.into() }
        }
        /// Constructor for `Charts`.
        pub fn charts(msg: ChartsMsg) -> Self {
            Self::Charts { msg }
        }
    }

    /// Messages for the charts of the client.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum ChartsMsg {
        /// Messages for each of the charts.
        Chart(Vec<ChartMsg>),
        /// A new collection of points, overwrites existing points.
        NewPoints {
            /// New points for each chart.
            points: Vec<Points>,
        },
        /// Some points to append to existing points.
        AddPoints {
            /// New points to append.
            points: Vec<Points>,
        },
    }
    impl ChartsMsg {
        /// Constructor for `NewPoints`.
        pub fn new_points(points: Vec<Points>) -> Msg {
            Msg::Charts {
                msg: ChartsMsg::NewPoints { points },
            }
        }
        /// Constructor for `AddPoints`.
        pub fn add_points(points: Vec<Points>) -> Msg {
            Msg::charts(ChartsMsg::AddPoints { points })
        }
    }

    /// Messages for a specific chart in the client.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum ChartMsg {
        /// A brand new list of points.
        ///
        /// Replaces all the points in a chart.
        NewPoints(Points),
        /// Some points to append.
        Points(Points),
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
