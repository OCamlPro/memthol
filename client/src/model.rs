//! Model of the client.

use crate::base::*;

/// Model of the client.
pub struct Model {
    /// Component link.
    pub link: ComponentLink<Self>,
    /// Socket service with the server.
    pub socket: WebSocketService,
    /// Socket task for receiving/sending messages from/to the server.
    pub socket_task: WebSocketTask,
    /// Collection of charts.
    pub charts: Charts,
    /// Allocation filters.
    pub filters: filter::Filters,

    /// Footer DOM element.
    pub footer: footer::Footer,
}

impl Model {
    /// Activates the websocket to receive data from the server.
    fn activate_ws(link: &mut ComponentLink<Self>, socket: &mut WebSocketService) -> WebSocketTask {
        let (addr, port) = get_server_addr();
        let addr = format!("ws://{}:{}", addr, port + 1);
        let callback = link.send_back(|msg| Msg::FromServer(msg));
        let notification = link.send_back(|status| Msg::ConnectionStatus(status));
        socket.connect(&addr, callback, notification)
    }
}

/// # Communication with the server
impl Model {
    /// Sends a message to the server.
    pub fn server_send(&mut self, msg: msg::to_server::Msg) {
        self.socket_task.send(msg)
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
        }
    }
}

macro_rules! unwrap_or_send_err {
    ($e:expr => $slf:ident default $default:expr ) => {
        match $e {
            Ok(res) => res,
            Err(e) => {
                $slf.link.send_self(e.into());
                $default
            }
        }
    };
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        let mut socket = WebSocketService::new();
        let socket_task = Self::activate_ws(&mut link, &mut socket);
        let charts = Charts::new(link.send_back(|msg: Msg| msg));
        let filters = filter::Filters::new(link.send_back(|msg: Msg| msg));
        Model {
            link,
            socket,
            socket_task,
            charts,
            filters,
            footer: footer::Footer::new(),
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
                    Opened => info!("successfully established connection with the server"),
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
                info!("{}", s);
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
}

impl Renderable<Model> for Model {
    fn view(&self) -> Html {
        html! {
            <>
                <div class=style::class::FULL_BODY>
                    { self.charts.render() }
                    { self.footer.render(&self.filters) }
                </div>
            </>
        }
    }
}
