//! Websockets used by the server to communicate with the clients.

use crate::base::*;

/// Creates a websocket server at some address.
fn new_server(addr: &str, port: usize) -> Res<Server> {
    let server = Server::bind(&format!("{}:{}", addr, port))
        .chain_err(|| format!("while binding websocket server at `{}:{}`", addr, port))?;
    Ok(server)
}

fn handle_requests(log: bool, server: Server) -> Res<()> {
    for request in server.filter_map(Result::ok) {
        let mut handler =
            Handler::new(log, request).chain_err(|| "while creating request handler")?;
        std::thread::spawn(move || handler.run());
        ()
    }
    Ok(())
}

pub fn spawn_server(addr: &str, port: usize, log: bool) -> Res<()> {
    let server = new_server(addr, port)?;
    std::thread::spawn(move || handle_requests(log, server));
    Ok(())
}

pub struct Com {
    ip: IpAddr,
    sender: Sender,
    receiver: Receiver,
    log: Option<std::fs::File>,
    /// Ping message use for acknowledgments.
    ping_msg: websocket::message::OwnedMessage,
}
impl Com {
    pub fn new(
        log: bool,
        ping_label: Vec<u8>,
        ip: IpAddr,
        sender: Sender,
        receiver: Receiver,
    ) -> Res<Self> {
        let ping_msg = websocket::message::OwnedMessage::Ping(ping_label);

        let log = if log {
            use std::fs::OpenOptions;
            let path = format!("log_{}", ip);
            let file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(&path)
                .chain_err(|| format!("while opening log file {:?}", path))?;
            Some(file)
        } else {
            None
        };

        Ok(Self {
            log,
            ip,
            sender,
            receiver,
            ping_msg,
        })
    }

    pub fn ip(&self) -> &IpAddr {
        &self.ip
    }

    pub fn send(&mut self, msg: impl Into<msg::to_client::Msg>) -> Res<()> {
        use websocket::message::OwnedMessage;

        let content = msg
            .into()
            .as_json()
            .chain_err(|| "while encoding message as json")?;

        if let Some(log) = self.log.as_mut() {
            use std::io::Write;
            writeln!(
                log,
                "[{}] sending message to client {{",
                alloc_data::time::now()
            )
            .chain_err(|| "while writing to log file")?;
            for line in content.to_string().lines() {
                writeln!(log, "    {}", line).chain_err(|| "while writing to log file")?;
            }
            writeln!(log, "}}\n").chain_err(|| "while writing to log file")?;
        }

        let msg = OwnedMessage::Binary(content.into_bytes());
        self.sender
            .send_message(&msg)
            .chain_err(|| format!("while sending message to client {}", self.ip))?;
        Ok(())
    }

    pub fn send_ping(&mut self) -> Res<()> {
        if let Some(log) = self.log.as_mut() {
            use std::io::Write;
            writeln!(
                log,
                "[{}] sending ping message to client\n",
                alloc_data::time::now(),
            )
            .chain_err(|| "while writing to log file")?;
        }

        self.sender
            .send_message(&self.ping_msg)
            .chain_err(|| format!("while sending message to client {}", self.ip))?;
        Ok(())
    }

    pub fn receiver(&self) -> &Receiver {
        &self.receiver
    }
    pub fn receiver_mut(&mut self) -> &mut Receiver {
        &mut self.receiver
    }
    fn log_receive_msg(log: &mut std::fs::File, msg: &Res<websocket::OwnedMessage>) -> Res<()> {
        use std::io::Write;
        use websocket::OwnedMessage::*;

        match msg {
            Ok(Text(txt)) => {
                writeln!(
                    log,
                    "[{}] received text message from client {{",
                    alloc_data::time::now(),
                )?;
                for line in txt.lines() {
                    writeln!(log, "    {}", line)?
                }
                writeln!(log, "}}\n")?
            }
            Ok(Binary(data)) => {
                let msg = msg::from_client::Msg::from_json_bytes(&data)
                    .chain_err(|| "while parsing message from client")?;
                writeln!(
                    log,
                    "[{}] received binary message from client {{",
                    alloc_data::time::now(),
                )?;
                for line in msg.as_json() {
                    writeln!(log, "    {}", line)?
                }
                writeln!(log, "}}\n")?
            }
            Ok(Ping(_)) => writeln!(
                log,
                "[{}] received ping message from client\n",
                alloc_data::time::now()
            )?,
            Ok(Pong(_)) => writeln!(
                log,
                "[{}] received pong message from client\n",
                alloc_data::time::now()
            )?,
            Ok(Close(_)) => writeln!(
                log,
                "[{}] received close message from client\n",
                alloc_data::time::now()
            )?,
            Err(e) => {
                writeln!(
                    log,
                    "[{}] received error from client {{    {}    }}\n",
                    alloc_data::time::now(),
                    e
                )?;
            }
        }
        Ok(())
    }

    pub fn incoming_messages<'a>(
        &'a mut self,
    ) -> impl Iterator<Item = Res<websocket::OwnedMessage>> + 'a {
        let log = &mut self.log;
        self.receiver.incoming_messages().map(move |res| {
            let res = res.map_err(|e| e.to_string().into());

            let log_res = if let Some(log) = log {
                Self::log_receive_msg(log, &res)
            } else {
                Ok(())
            };

            if res.is_ok() {
                if let Err(e) = log_res {
                    return Err(e);
                }
            }

            res
        })
    }
}

pub struct Handler {
    /// Sends/receives messages to/from the client.
    com: Com,
    /// The charts of the client.
    charts: Charts,
    /// Stores the result of receiving messages from the client.
    from_client: FromClient,
    /// Time at which we last sent points to render.
    last_frame: Instant,
    /// Minimum time between two rendering steps.
    frame_span: Duration,
    /// Label for ping messages.
    ping_label: Vec<u8>,
}

impl Handler {
    /// Constructor from a request and a dump directory.
    pub fn new(log: bool, request: Request) -> Res<Self> {
        let client = request
            .accept()
            .map_err(|(_, e)| e)
            .chain_err(|| "while accepting websocket connection")?;
        let ip = client
            .peer_addr()
            .chain_err(|| "while retrieving client's IP address")?;

        let (receiver, sender) = client
            .split()
            .chain_err(|| "while splitting the client into receive/send pair")?;

        let ping_label = vec![6u8, 6u8, 6u8];
        let mut com = Com::new(log, ping_label.clone(), ip, sender, receiver)
            .chain_err(|| "during communicator construction")?;

        // Wait until data has been loaded.
        if let Some(mut info) = charts::data::progress::get()? {
            macro_rules! send {
                () => {
                    com.send(msg::to_client::Msg::load_progress(info.clone()))?
                };
            }

            send!();

            while let Some(nu_info) = charts::data::progress::get()? {
                if nu_info != info {
                    info = nu_info;
                    send!();
                }
                std::thread::sleep(std::time::Duration::from_millis(200))
            }
        }
        com.send(msg::to_client::Msg::DoneLoading)?;

        let mut charts = Charts::new();

        charts
            .auto_gen(charts::filter::FilterGen::default())
            .chain_err(|| "during default filter generation")?;

        let slf = Handler {
            com,
            charts,
            from_client: FromClient::new(),
            last_frame: Instant::now(),
            frame_span: Duration::from_millis(500),
            ping_label,
        };

        log!(slf.ip() => "connection successful");

        Ok(slf)
    }

    pub fn ip(&self) -> &IpAddr {
        self.com.ip()
    }

    /// Runs the handler.
    pub fn run(&mut self) {
        unwrap!(self.internal_run())
    }

    /// Sets the time of the last frame to now.
    fn set_last_frame(&mut self) {
        self.last_frame = Instant::now()
    }

    /// Runs the handler, can fail.
    fn internal_run(&mut self) -> Res<()> {
        log!(self.ip() => "init...");
        self.init()?;

        log!(self.ip() => "running...");

        // Let's do this.
        loop {
            self.set_last_frame();
            self.send_ping()?;

            // Receive new messages.
            self.receive_messages()?;

            // Connection closed?
            if self.from_client.is_closed() {
                let close_data = self
                    .from_client
                    .close_data()
                    .map(
                        |CloseData {
                             status_code,
                             reason,
                         }| {
                            let mut blah = format!("status code `{}`", status_code);
                            if !reason.is_empty() {
                                blah.push_str(": ");
                                blah.push_str(&reason)
                            }
                            blah
                        },
                    )
                    .unwrap_or_else(|| "no information".into());
                log!(self.ip() => "client closed the connection with {}", close_data);
                break;
            }

            // List of messages to send to the client in response to the messages received from the
            // client.
            let mut to_client_msgs = vec![];

            // Handle the messages.
            for msg in self.from_client.drain() {
                let msgs = self.charts.handle_msg(msg)?;
                to_client_msgs.extend(msgs)
            }

            for msg in to_client_msgs {
                self.send(msg)?
            }

            // Wait before rendering if necessary.
            let now = Instant::now();
            if now <= self.last_frame + self.frame_span {
                std::thread::sleep((self.last_frame + self.frame_span) - now)
            }

            // Render.
            let (points, overwrite) = self
                .charts
                .new_points(false)
                .chain_err(|| "while constructing points for the client")?;
            if overwrite || !points.is_empty() {
                let msg = msg::to_client::ChartsMsg::points(points, overwrite);
                self.send(msg)
                    .chain_err(|| "while sending points to the client")?
            }
        }

        Ok(())
    }

    fn send_all_charts(&mut self) -> Res<()> {
        for chart in self.charts.charts() {
            let msg = msg::to_client::ChartsMsg::new_chart(
                chart.spec().clone(),
                chart.settings().clone(),
            );
            self.com.send(msg)?
        }
        Ok(())
    }
    fn send_filters(&mut self) -> Res<()> {
        let msg = msg::to_client::FiltersMsg::revert(
            self.charts.filters().everything().clone(),
            self.charts.filters().filters().clone(),
            self.charts.filters().catch_all().clone(),
        );
        self.send(msg)
    }
    fn send_all_points(&mut self) -> Res<()> {
        let (points, overwrite) = self.charts.new_points(true)?;
        let msg = msg::to_client::ChartsMsg::points(points, overwrite);
        self.send(msg)
    }

    /// Initializes a client.
    pub fn init(&mut self) -> Res<()> {
        use charts::chart::{
            axis::{XAxis, YAxis},
            Chart,
        };
        let chart = Chart::new(self.charts.filters(), XAxis::Time, YAxis::TotalSize)?;
        self.charts.push(chart);
        self.send_filters()
            .chain_err(|| "while sending filters for client init")?;
        self.send_all_charts()
            .chain_err(|| "while sending charts for client init")?;
        self.send_all_points()
            .chain_err(|| "while sending points for client init")?;

        Ok(())
    }

    /// Sends a ping message to the client.
    pub fn send_ping(&mut self) -> Res<()> {
        self.com
            .send_ping()
            .chain_err(|| format!("while sending ping message to client {}", self.ip()))
    }

    /// Sends a message to the client.
    pub fn send(&mut self, msg: impl Into<msg::to_client::Msg>) -> Res<()> {
        self.com.send(msg)
    }

    /// Retrieves actions to perform from the client before rendering.
    ///
    /// Returns `None` if the client requested to close
    fn receive_messages(&mut self) -> Res<()> {
        // Used in the `match` below.
        use websocket::message::OwnedMessage::*;

        for message in self.com.incoming_messages() {
            let message = message.chain_err(|| "while retrieving message")?;

            // Let's do this.
            match message {
                // Normal message(s) from the client.
                Text(text) => {
                    let msg = msg::from_client::Msg::from_json(&text)
                        .chain_err(|| "while parsing message from client")?;
                    self.from_client.push(msg)?
                }
                Binary(data) => {
                    let msg = msg::from_client::Msg::from_json_bytes(&data)
                        .chain_err(|| "while parsing message from client")?;
                    self.from_client.push(msg)?
                }

                // The client is telling us to stop listening for messages and render.
                Pong(label) => {
                    if self.ping_label == label {
                        break;
                    } else {
                        bail!(
                            "unexpected `Pong` label: expected {:?}, got {:?}",
                            self.ping_label,
                            label
                        )
                    }
                }

                // Client is closing the connection.
                Close(close_data) => {
                    self.from_client.close()?;
                    self.from_client.set_close_data(close_data)?;
                    break;
                }

                // Unexpected mesage(s).
                Ping(label) => bail!(
                    "unexpected `Ping({})` message",
                    String::from_utf8_lossy(&label)
                ),
            }
        }

        Ok(())
    }
}
