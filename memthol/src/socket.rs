//! Websockets used by the server to communicate with the clients.

use crate::prelude::*;

/// Creates a websocket server at some address.
fn new_server(addr: &str, port: usize) -> Res<net::TcpListener> {
    let server = net::TcpListener::bind(&format!("{}:{}", addr, port))
        .chain_err(|| format!("while binding websocket server at `{}:{}`", addr, port))?;
    Ok(server)
}

/// Spawns a `Handler` for each incoming connection request.
fn handle_requests(log: bool, server: net::TcpListener) {
    for stream in server.incoming().filter_map(Result::ok) {
        let mut handler = base::unwrap_or! {
            Handler::new(log, stream).chain_err(|| "while creating request handler"),
            {
                log::error!("failed to start request handler");
                return ()
            }
        };
        std::thread::spawn(move || handler.run());
        ()
    }
}

/// Spawns the server that listens for connection requests.
pub fn spawn_server(addr: &str, port: usize, log: bool) -> Res<()> {
    let server = new_server(addr, port)?;
    std::thread::spawn(move || handle_requests(log, server));
    Ok(())
}

base::new_time_stats! {
    struct Prof {
        total => "total",
        bytes => "bytes-ser",
        log => "logging",
        send => "sending",
    }
}

/// Maintains a socket to a client and some information such as the client's IP.
pub struct Com {
    /// IP addresse.
    ip: net::IpAddr,
    /// Socket used for communicating with the client.
    socket: net::WebSocket,
    /// Optional log file.
    log: Option<std::fs::File>,
    /// Ping message use for acknowledgments.
    ping_msg: tungstenite::Message,
    /// Time statistics.
    prof: Prof,
    /// Error context.
    err_cxt: err::ErrorCxt,
}
impl Com {
    /// Constructor.
    ///
    /// The `log` flag, if `true`, makes the constructor create a log file in the current directory.
    /// It will contain a log of all the interactions with this client.
    pub fn new(log: bool, ping_label: Vec<u8>, socket: net::WebSocket) -> Res<Self> {
        let ping_msg = tungstenite::Message::Ping(ping_label);

        let ip = socket
            .get_ref()
            .peer_addr()
            .map_err(|e| format!("failed to retrieve client IP: {}", e))?;

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
            socket,
            ping_msg,
            prof: Prof::new(),
            err_cxt: err::ErrorCxt::new(),
        })
    }

    /// IP address of the client.
    pub fn ip(&self) -> &net::IpAddr {
        &self.ip
    }

    /// Sends a message to the client.
    pub fn send(&mut self, msg: impl Into<msg::to_client::Msg>) -> Res<()> {
        use tungstenite::Message;

        self.prof.reset();
        self.prof.total.start();

        let msg = msg.into();
        let is_interesting = !msg.is_minor();

        if Prof::TIME_STATS_ACTIVE && is_interesting {
            log::info!("sending message to client: {}", msg);
        } else {
            log::trace!("sending message to client: {}", msg);
        }

        time! {
            > self.prof.log,

            if let Some(log) = self.log.as_mut() {
                use std::io::Write;
                writeln!(
                    log,
                    "[{}] sending message to client: {}",
                    time::Date::now(),
                    msg
                )
                .chain_err(|| "while writing to log file")?;
            }
        }

        let bytes = time! {
            > self.prof.bytes,
            msg.to_bytes()
        }?;
        log::trace!("sending binary message ({} bytes)", bytes.len());
        let msg = Message::Binary(bytes);

        time! {
            > self.prof.send,
            self.socket
                .write_message(msg)
                .chain_err(|| format!("while sending message to client {}", self.ip))?
        };

        self.prof.total.stop();

        if is_interesting {
            self.prof.all_do(
                || log::info!("message sent"),
                |desc, sw| log::info!("| {:>16}: {}", desc, sw),
            );
        }

        Ok(())
    }

    /// Sends a ping message to the client.
    pub fn send_ping(&mut self) -> Res<()> {
        if let Some(log) = self.log.as_mut() {
            use std::io::Write;
            writeln!(
                log,
                "[{}] sending ping message to client\n",
                time::Date::now(),
            )
            .chain_err(|| "while writing to log file")?;
        }

        self.socket
            .write_message(self.ping_msg.clone())
            .chain_err(|| format!("while sending message to client {}", self.ip))?;
        Ok(())
    }

    /// Sends chart statistics to the client.
    fn send_stats(&mut self, charts: &Charts) -> Res<()> {
        use charts::prelude::AllocStats;
        if let Some(stats) = AllocStats::get()? {
            if let Some(log) = self.log.as_mut() {
                use std::io::Write;
                writeln!(
                    log,
                    "[{}] sending stats message to client\n",
                    time::Date::now(),
                )
                .chain_err(|| "while writing to log file")?;
            }

            self.send(msg::to_client::Msg::alloc_stats(stats))?;
            self.send(msg::to_client::Msg::filter_stats(
                charts.filters().filter_stats()?,
            ))?
        }

        Ok(())
    }

    /// Send charts-related errors to the client.
    fn send_errors(&mut self) -> Res<()> {
        let mut err_cxt = self.err_cxt.clone();
        err_cxt
            .new_errors_try(|err, is_fatal| self.send(msg::to_client::Msg::alert(err, is_fatal)))?;
        self.err_cxt = err_cxt;
        Ok(())
    }

    /// Logs the reception of a message.
    fn log_receive_msg(&mut self, msg: Either<&msg::from_client::Msg, &str>) -> Res<()> {
        if let Some(log) = self.log.as_mut() {
            use std::io::Write;

            match msg {
                Either::Left(msg) => writeln!(
                    log,
                    "[{}] received message from client {}",
                    time::Date::now(),
                    msg,
                )?,
                Either::Right(desc) => writeln!(
                    log,
                    "[{}] received message from client {}",
                    time::Date::now(),
                    desc,
                )?,
            }
        }

        Ok(())
    }

    /// Retrieves a message from the client.
    pub fn incoming_message<'a>(&'a mut self) -> Res<net::Msg> {
        self.socket.read_message().map_err(|e| {
            err::Error::from(format!("failed to receive message from {}: {}", self.ip, e))
        })
    }
}

base::new_time_stats! {
    struct HandlerProf {
        point_extraction => "point extraction",
        point_sending => "sending point messages",
    }
}

/// Handles communications with a client, maintains the client's state.
pub struct Handler {
    /// Sends/receives messages to/from the client.
    com: Com,
    /// The charts of the client.
    charts: Charts,
    /// Stores the result of receiving messages from the client.
    from_client: FromClient,
    /// Time at which we last sent points to render.
    last_frame: time::Instant,
    /// Minimum time between two rendering steps.
    frame_span: time::Duration,
    /// Label for ping messages.
    ping_label: Vec<u8>,

    instance_prof: HandlerProf,
    total_prof: HandlerProf,
}

impl Handler {
    /// Constructor from a request and a dump directory.
    pub fn new(log: bool, stream: std::net::TcpStream) -> Res<Self> {
        let socket = tungstenite::server::accept(stream).map_err(|e| e.to_string())?;

        let instance_prof = HandlerProf::new();
        let total_prof = HandlerProf::new();

        // let (receiver, sender) = client
        //     .split()
        //     .chain_err(|| "while splitting the client into receive/send pair")?;

        let ping_label = vec![6u8, 6u8, 6u8];
        let mut com = Com::new(log, ping_label.clone(), socket)
            .chain_err(|| "during communicator construction")?;

        com.send_errors()?;

        // Wait until data has been loaded.
        if let Some(mut info) = charts::data::progress::get()? {
            macro_rules! send {
                () => {
                    com.send(msg::to_client::Msg::load_progress(info.clone()))?
                };
            }

            send!();

            while let Some(nu_info) = charts::data::progress::get()? {
                com.send_errors()?;
                if nu_info != info {
                    info = nu_info;
                    send!();
                }
                std::thread::sleep(std::time::Duration::from_millis(200))
            }
        }

        com.send(msg::to_client::Msg::DoneLoading)?;

        let mut charts = Charts::new();

        time! {
            charts
                .auto_gen()
                .chain_err(|| "during default filter generation")?,
            |time| log::info!("done with filter generation in {}", time)
        };

        let slf = Handler {
            com,
            charts,
            from_client: FromClient::new(),
            last_frame: time::Instant::now(),
            frame_span: time::Duration::from_millis(500),
            ping_label,

            instance_prof,
            total_prof,
        };

        log::info!("successfully connected to {}", slf.ip());

        Ok(slf)
    }

    /// The client's IP address.
    pub fn ip(&self) -> &net::IpAddr {
        self.com.ip()
    }

    /// Display time statistics.
    #[inline]
    pub fn show_time_stats(&self, msg: &'static str) {
        self.show_instance_time_stats(msg);
        self.show_total_time_stats("total:");
    }
    /// Displays total time statistics.
    #[inline]
    pub fn show_total_time_stats(&self, msg: &'static str) {
        self.total_prof.all_do(
            || log::info!("{}", msg),
            |desc, sw| log::info!("| {:>20}: {}", desc, sw),
        );
    }
    /// Displays instance time statistics.
    #[inline]
    pub fn show_instance_time_stats(&self, msg: &'static str) {
        self.instance_prof.all_do(
            || log::info!("{}", msg),
            |desc, sw| log::info!("| {:>20}: {}", desc, sw),
        );
    }

    /// Runs the handler.
    pub fn run(&mut self) {
        base::unwrap_or!(
            self.internal_run(),
            log::info!("lost connection with {}", self.ip())
        )
    }

    /// Sets the time of the last frame to now.
    fn set_last_frame(&mut self) {
        self.last_frame = time::Instant::now()
    }

    /// Runs the handler, can fail.
    fn internal_run(&mut self) -> Res<()> {
        self.init()?;

        // Let's do this.
        loop {
            self.com.send_errors()?;
            self.set_last_frame();
            self.send_ping()?;

            // Receive new messages.
            self.receive_messages()?;

            // Connection closed?
            if self.from_client.is_closed() {
                let close_data = self
                    .from_client
                    .close_data()
                    .map(|net::CloseFrame { code, reason }| {
                        let mut blah = format!("status code `{}`", code);
                        if !reason.is_empty() {
                            blah.push_str(": ");
                            blah.push_str(&reason)
                        }
                        blah
                    })
                    .unwrap_or_else(|| "no information".into());
                log::debug!(
                    "client {} closed the connection with {}",
                    self.ip(),
                    close_data
                );
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
            let now = time::Instant::now();
            if now <= self.last_frame + self.frame_span {
                std::thread::sleep((self.last_frame + self.frame_span) - now)
            }

            // Render.
            let (points, overwrite) = time! {
                > self.instance_prof.point_extraction,
                > self.total_prof.point_extraction,
                self
                    .charts
                    .new_points(false)
                    .chain_err(|| "while constructing points for the client")?
            };

            if overwrite || !points.is_empty() {
                time! {
                    > self.instance_prof.point_sending,
                    > self.total_prof.point_sending,

                    self.send(msg::to_client::ChartsMsg::points(points, overwrite))
                        .chain_err(|| "while sending points to the client")?
                }

                self.send_stats()?;

                self.show_time_stats("done extracting/sending points");
            }
        }

        Ok(())
    }

    /// Sends chart-related statistics to the client.
    fn send_stats(&mut self) -> Res<()> {
        self.com.send_stats(&self.charts)
    }

    /// Sends all charts to the client.
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
    /// Sends all the filters to the client.
    fn send_filters(&mut self) -> Res<()> {
        let msg = msg::to_client::FiltersMsg::revert(
            self.charts.filters().everything().clone(),
            self.charts.filters().filters().clone(),
            self.charts.filters().catch_all().clone(),
        );
        self.send(msg)
    }
    /// Sends all the points in all the charts to the client.
    fn send_all_points(&mut self) -> Res<()> {
        let (points, overwrite) = time! {
            > self.instance_prof.point_extraction.start(),
            > self.total_prof.point_extraction.start(),

            self.charts.new_points(true)?
        };

        if !points.is_empty() {
            time! {
                > self.instance_prof.point_sending,
                > self.total_prof.point_sending,

                self.send(msg::to_client::ChartsMsg::points(points, overwrite))?
            }

            self.show_time_stats("done extracting/sending points");

            self.send_stats()?
        }

        self.instance_prof.reset();

        Ok(())
    }

    /// Initializes a client.
    pub fn init(&mut self) -> Res<()> {
        use charts::chart::{
            axis::{XAxis, YAxis},
            Chart,
        };

        self.send_stats()?;

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
        for message in self.com.incoming_message() {
            // Let's do this.
            match message {
                // Normal message(s) from the client.
                net::Msg::Text(_) => bail!(
                    "trying to receive a message in text format, \
                        only binary format is supported"
                ),
                net::Msg::Binary(data) => {
                    let msg = msg::from_client::Msg::from_bytes(&data)
                        .chain_err(|| "while parsing message from client")?;
                    self.com.log_receive_msg(Either::Left(&msg))?;
                    self.from_client.push(msg)?
                }

                // The client is telling us to stop listening for messages and render.
                net::Msg::Pong(label) => {
                    if self.ping_label == label {
                        self.com.log_receive_msg(Either::Right("pong"))?;
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
                net::Msg::Close(close_data) => {
                    self.com
                        .log_receive_msg(Either::Right("close connection"))?;
                    self.from_client.close()?;
                    self.from_client.set_close_data(close_data)?;
                    break;
                }

                // Unexpected mesage(s).
                net::Msg::Ping(label) => bail!(
                    "unexpected `Ping({})` message",
                    String::from_utf8_lossy(&label)
                ),
            }
        }

        Ok(())
    }
}
