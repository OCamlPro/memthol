//! Websockets used by the server to communicate with the clients.

use crate::base::*;

/// Creates a websocket server at some address.
fn new_server(addr: &str, port: usize) -> Res<Server> {
    let server = Server::bind(&format!("{}:{}", addr, port))
        .chain_err(|| format!("while binding websocket server at `{}:{}`", addr, port))?;
    Ok(server)
}

fn handle_requests(server: Server) -> Res<()> {
    for request in server.filter_map(Result::ok) {
        let mut handler = Handler::new(request).chain_err(|| "while creating request handler")?;
        std::thread::spawn(move || handler.run());
        ()
    }
    Ok(())
}

pub fn spawn_server(addr: &str, port: usize) -> Res<()> {
    let server = new_server(addr, port)?;
    std::thread::spawn(move || handle_requests(server));
    Ok(())
}

pub struct Handler {
    /// Ip address of the client.
    ip: IpAddr,
    /// Receives messages from the client.
    recver: Receiver,
    /// Sends messages to the client.
    sender: Sender,
    /// Label sent in ping (acknowledgment) messages to the client.
    ping_label: Vec<u8>,
    /// The charts of the client.
    charts: Charts,
}

impl Handler {
    /// Constructor from a request and a dump directory.
    pub fn new(request: Request) -> Res<Self> {
        let client = request
            .accept()
            .map_err(|(_, e)| e)
            .chain_err(|| "while accepting websocket connection")?;
        let ip = client
            .peer_addr()
            .chain_err(|| "while retrieving client's IP address")?;

        let (recver, sender) = client
            .split()
            .chain_err(|| "while splitting the client into receive/send pair")?;

        let charts = Charts::new();

        let ping_label = vec![6u8, 6u8, 6u8];

        let slf = Handler {
            ip,
            recver,
            sender,
            ping_label,
            charts,
        };

        Ok(slf)
    }

    /// Runs the handler.
    pub fn run(&mut self) {
        unwrap!(self.internal_run())
    }

    /// Runs the handler, can fail.
    fn internal_run(&mut self) -> Res<()> {
        self.init()?;

        Ok(())
    }

    /// Initializes a client.
    pub fn init(&mut self) -> Res<()> {
        let points = self
            .charts
            .new_points(true)
            .chain_err(|| "while constructing points for client init")?;
        log!(self.ip => "sending points to client");
        self.send(msg::to_client::ChartsMsg::new_points(points))
            .chain_err(|| "while sending points for client init")?;
        Ok(())
    }

    /// Sends a message to the client.
    pub fn send<Msg>(&mut self, msg: Msg) -> Res<()>
    where
        Msg: Into<msg::to_client::Msg>,
    {
        use websocket::message::OwnedMessage;

        let content = msg
            .into()
            .as_json()
            .chain_err(|| "while encoding message as toml")?
            .into_bytes();
        let msg = OwnedMessage::Binary(content);
        self.sender
            .send_message(&msg)
            .chain_err(|| format!("while sending message to client {}", self.ip))?;
        Ok(())
    }
}
