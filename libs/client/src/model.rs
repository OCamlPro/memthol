//! Model of the client.

prelude! {}

/// Model of the client.
pub struct Model {
    /// Component link.
    pub link: Link,
    /// Socket task for receiving/sending messages from/to the server.
    pub socket_task: Option<WebSocketTask>,
    /// Errors.
    pub errors: Vec<err::Error>,
    /// Collection of charts.
    pub charts: Charts,

    /// Allocation filters.
    pub filters: filter::FilterInfo,

    /// Footer DOM element.
    pub footer: footer::Footer,
    /// Header DOM element.
    pub header: header::Header,

    /// If not `None`, then the server is currently loading the dumps.
    pub progress: Option<LoadInfo>,
    /// Allocation statistics, for the header.
    pub alloc_stats: Option<AllocStats>,

    /// Global chart settings.
    pub settings: settings::Settings,
}

impl Model {
    /// Reference filters accessor.
    pub fn filters(&self) -> filter::Reference {
        self.filters.reference()
    }
    /// Client-side filters accessor.
    pub fn footer_filters(&self) -> filter::Current {
        self.filters.current()
    }
    /// Charts accessor.
    pub fn charts(&self) -> &Charts {
        &self.charts
    }

    /// True if the `catch_all` filter does not catch any allocation.
    pub fn is_catch_all_empty(&self) -> bool {
        self.filters
            .ref_stats()
            .get(uid::Line::CatchAll)
            .map(|stats| stats.alloc_count == 0)
            .unwrap_or(true)
    }
}

impl Model {
    /// Activates the websocket to receive data from the server.
    fn activate_ws(link: &mut Link) -> Res<WebSocketTask> {
        log::info!("fetching server's websocket info");
        let (addr, port) = js::server::address()?;
        let addr = format!("ws://{}:{}", addr, port + 1);
        log::info!("websocket: {:?}", addr);
        let callback = link.callback(|msg| Msg::FromServer(msg));
        let notification = link.callback(|status| Msg::ConnectionStatus(status));
        let task = WebSocketService::connect(&addr, callback, notification)?;
        log::info!("connection established successfully");
        Ok(task)
    }
}

/// # Communication with the server
impl Model {
    /// Sends a message to the server.
    pub fn server_send(&mut self, msg: msg::to_server::Msg) {
        if let Some(socket_task) = self.socket_task.as_mut() {
            socket_task.send_binary(msg)
        } else {
            log::warn!("no socket task available, failed to send message {}", msg)
        }
    }

    /// Handles a message from the server.
    pub fn handle_server_msg(&mut self, msg: Res<msg::from_server::Msg>) -> Res<ShouldRender> {
        use msg::from_server::*;
        let msg = msg?;
        log::info!("received message from server: {}", msg);
        match msg {
            Msg::Info => Ok(false),
            Msg::Alert { msg, fatal } => {
                alert!("{}{}", if fatal { "[fatal] " } else { "" }, msg);
                Ok(false)
            }
            Msg::Charts(msg) => {
                self.charts
                    .server_update(self.filters.reference(), self.filters.ref_stats(), msg)
            }
            Msg::Filters(msg) => self.filters.server_update(msg),

            Msg::AllocStats(stats) => {
                let redraw = self
                    .alloc_stats
                    .as_ref()
                    .map(|s| s != &stats)
                    .unwrap_or(true);
                self.settings.set_run_duration(stats.duration);
                self.alloc_stats = Some(stats);
                Ok(redraw)
            }
            Msg::FilterStats(stats) => {
                log::info!("updating filter stats");
                self.filters.update_ref_stats(stats);
                Ok(true)
            }

            Msg::LoadProgress(info) => {
                let redraw = self.progress.as_ref().map(|s| s != &info).unwrap_or(true);
                self.progress = Some(info);
                Ok(redraw)
            }
            Msg::DoneLoading => {
                let redraw = self.progress.is_some();
                self.progress = None;
                Ok(redraw)
            }
        }
    }
}

macro_rules! unwrap_or_send_err {
    ($e:expr => $slf:ident default $default:expr ) => {
        match $e {
            Ok(res) => res,
            Err(e) => {
                $slf.link.send_message(e);
                $default
            }
        }
    };
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, mut link: Link) -> Self {
        let (socket_task, errors) = match Self::activate_ws(&mut link) {
            Ok(res) => (Some(res), vec![]),
            Err(e) => (None, vec![e]),
        };
        let charts = Charts::new(link.clone());
        let filters = filter::FilterInfo::new(link.clone());
        let settings = settings::Settings::new(link.clone());
        let header = header::Header::new(link.clone());
        Model {
            link,
            socket_task,
            errors,
            charts,

            filters,

            footer: footer::Footer::new(),
            header,

            progress: Some(LoadInfo::unknown()),
            alloc_stats: None,
            settings,
        }
    }

    fn update(&mut self, msg: Msg) -> ShouldRender {
        match msg {
            // Messages to/from the server.
            Msg::FromServer(msg) => {
                let msg: Res<charts::msg::to_client::Msg> = msg.into();
                unwrap_or_send_err!(self.handle_server_msg(msg) => self default false)
            }
            Msg::ToServer(msg) => {
                log::info!("propagating message to server {}", msg);
                self.server_send(msg);
                false
            }

            // Dealing with status changes in the connection with the server.
            Msg::ConnectionStatus(status) => {
                use WebSocketStatus::*;
                match status {
                    Opened => log::debug!("successfully established connection with the server"),
                    Closed => log::warn!("connection with the server was closed"),
                    Error => alert!("failed to connect with the server"),
                }
                false
            }

            // Internal operations.
            Msg::Charts(msg) => unwrap_or_send_err!(
                self.charts.update(self.filters.reference(), msg) => self default false
            ),
            Msg::Footer(msg) => unwrap_or_send_err!(
                self.footer.update(msg) => self default false
            ),
            Msg::Filter(msg) => unwrap_or_send_err!(
                self.filters.update(msg) => self default false
            ),
            Msg::Settings(msg) => unwrap_or_send_err!(
                self.settings.update(msg) => self default false
            ),

            // Basic communication messages.
            Msg::Msg(s) => {
                log::debug!("{}", s);
                false
            }
            Msg::Warn(s) => {
                log::warn!("{}", s);
                false
            }
            Msg::Err(e) => {
                alert!("{}", e.to_pretty());
                true
            }

            Msg::Noop => false,
        }
    }
    fn view(&self) -> Html {
        layout::render(self)
    }

    fn rendered(&mut self, _first_render: bool) {
        self.charts
            .rendered(self.filters.reference(), self.filters.ref_stats())
    }

    fn change(&mut self, _props: ()) -> bool {
        false
    }
}
