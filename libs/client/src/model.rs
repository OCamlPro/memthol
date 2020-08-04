//! Model of the client.

prelude! {}

/// Model of the client.
pub struct Model {
    /// Component link.
    pub link: ComponentLink<Self>,
    /// Socket task for receiving/sending messages from/to the server.
    pub socket_task: Option<WebSocketTask>,
    /// Errors.
    pub errors: Vec<err::Err>,
    /// Collection of charts.
    pub charts: Charts,
    /// Allocation filters.
    pub filters: filter::Filters,

    /// Footer DOM element.
    pub footer: footer::Footer,

    /// If not `None`, then the server is currently loading the dumps.
    pub progress: Option<LoadInfo>,
}

impl Model {
    pub fn filters(&self) -> &filter::ReferenceFilters {
        self.filters.reference_filters()
    }
    pub fn footer_filters(&self) -> &filter::Filters {
        &self.filters
    }
    pub fn charts(&self) -> &Charts {
        &self.charts
    }
}

impl Model {
    /// Activates the websocket to receive data from the server.
    fn activate_ws(link: &mut ComponentLink<Self>) -> Res<WebSocketTask> {
        info!("fetching server's websocket info");
        let (addr, port) = js::server::address()?;
        let addr = format!("ws://{}:{}", addr, port + 1);
        info!("websocket: {:?}", addr);
        let callback = link.callback(|msg| Msg::FromServer(msg));
        let notification = link.callback(|status| Msg::ConnectionStatus(status));
        let task = WebSocketService::connect(&addr, callback, notification)?;
        info!("connection established successfully");
        Ok(task)
    }
}

/// # Communication with the server
impl Model {
    /// Sends a message to the server.
    pub fn server_send(&mut self, msg: msg::to_server::Msg) {
        self.socket_task.as_mut().map(|socket| socket.send(msg));
    }

    /// Handles a message from the server.
    pub fn handle_server_msg(&mut self, msg: Res<msg::from_server::Msg>) -> Res<ShouldRender> {
        use msg::from_server::*;
        let msg = msg?;
        match msg {
            Msg::Info => Ok(false),
            Msg::Alert { msg } => {
                alert!("{}", msg);
                Ok(false)
            }
            Msg::Charts(msg) => self.charts.server_update(&self.filters, msg),
            Msg::Filters(msg) => self.filters.server_update(msg),
            Msg::LoadProgress { loaded, total } => {
                self.progress = if loaded == total {
                    None
                } else {
                    Some(LoadInfo { loaded, total })
                };
                Ok(true)
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

    fn create(_: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        let (socket_task, errors) = match Self::activate_ws(&mut link) {
            Ok(res) => (Some(res), vec![]),
            Err(e) => (None, vec![e]),
        };
        let charts = Charts::new(link.callback(|msg: Msg| msg));
        let filters = filter::Filters::new(link.callback(|msg: Msg| msg));
        Model {
            link,
            socket_task,
            errors,
            charts,
            filters,
            footer: footer::Footer::new(),
            progress: None,
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
                self.server_send(msg);
                false
            }

            // Dealing with status changes in the connection with the server.
            Msg::ConnectionStatus(status) => {
                use WebSocketStatus::*;
                match status {
                    Opened => debug!("successfully established connection with the server"),
                    Closed => warn!("connection with the server was closed"),
                    Error => alert!("failed to connect with the server"),
                }
                false
            }

            // Internal operations.
            Msg::Charts(msg) => unwrap_or_send_err!(
                self.charts.update(&self.filters, msg) => self default false
            ),
            Msg::Footer(msg) => unwrap_or_send_err!(
                self.footer.update(msg) => self default false
            ),
            Msg::Filter(msg) => unwrap_or_send_err!(
                self.filters.update(msg) => self default false
            ),

            // Basic communication messages.
            Msg::Msg(s) => {
                debug!("{}", s);
                false
            }
            Msg::Warn(s) => {
                warn!("{}", s);
                false
            }
            Msg::Err(e) => {
                alert!("{}", e.pretty());
                true
            }

            Msg::Noop => false,
        }
    }
    fn view(&self) -> Html {
        layout::render(self)
    }

    fn rendered(&mut self, _first_render: bool) {
        self.charts.rendered(self.filters.reference_filters())
    }

    fn change(&mut self, _props: ()) -> bool {
        false
    }
}
