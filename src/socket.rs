//! Websockets used by the server to communicate with the clients.

use std::{ffi::OsString, path::Path, thread::sleep, time::SystemTime};

use websocket::message::OwnedMessage;

use crate::base::*;

/// A websocket server.
pub type Server = websocket::sync::Server<websocket::server::NoTlsAcceptor>;

/// A request.
pub type Request = websocket::server::upgrade::WsUpgrade<
    std::net::TcpStream,
    Option<websocket::server::upgrade::sync::Buffer>,
>;

/// An IP address.
pub type IpAddr = std::net::SocketAddr;
/// A receiver for a request.
pub type Receiver = websocket::receiver::Reader<std::net::TcpStream>;
/// A sender for a request.
pub type Sender = websocket::sender::Writer<std::net::TcpStream>;

/// Creates a websocket server at some address.
fn new_server(addr: &str, port: usize) -> Res<Server> {
    let server = Server::bind(&format!("{}:{}", addr, port))
        .chain_err(|| format!("while binding websocket server at `{}:{}`", addr, port))?;
    Ok(server)
}

fn handle_requests(server: Server, dump_dir: String) -> Res<()> {
    for request in server.filter_map(Result::ok) {
        let mut handler =
            Handler::new(request, &dump_dir).chain_err(|| "while creating request handler")?;
        std::thread::spawn(move || handler.run().unwrap());
        ()
    }
    Ok(())
}

pub fn spawn_server<Str: Into<String>>(addr: &str, port: usize, dump_dir: Str) -> Res<()> {
    let dump_dir = dump_dir.into();
    let server = new_server(addr, port)?;
    std::thread::spawn(move || handle_requests(server, dump_dir));
    Ok(())
}

pub struct Handler {
    /// Ip address of the client.
    ip: IpAddr,
    /// Receives messages from the client.
    recver: Receiver,
    /// Sends messages to the client.
    sender: Sender,
    /// Directory containing memthol files.
    dump_dir: String,
    /// Temporary file used by memthol to write dumps.
    tmp_file: &'static str,
    /// Init file.
    init_file: &'static str,
    /// Last date of modification of the init file.
    ///
    /// This is used to detect new runs by checking whether the init file has been modified.
    init_last_modified: Option<SystemTime>,
    /// Files that have already been sent to the client and must be ignored.
    ///
    /// **Always** contains `self.tmp_file` and `self.init_file`.
    known_files: Set<OsString>,
    /// Stores the new diff files to send to the client.
    new_diffs: Vec<std::fs::DirEntry>,
    /// Label sent in ping (acknowledgment) messages to the client.
    ping_label: Vec<u8>,
}
impl Handler {
    /// Constructor from a request and a dump directory.
    pub fn new<Str>(request: Request, dump_dir: Str) -> Res<Self>
    where
        Str: Into<String>,
    {
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

        let dump_dir = dump_dir.into();
        let tmp_file = "tmp.memthol";
        let init_file = "init.memthol";

        let ping_label = vec![6u8, 6u8, 6u8];

        let known_files = Set::new();

        let mut handler = Self {
            ip,
            recver,
            sender,
            dump_dir,
            tmp_file,
            init_file,
            init_last_modified: None,
            known_files,
            new_diffs: Vec::with_capacity(103),
            ping_label,
        };
        handler.reset();
        Ok(handler)
    }

    /// Resets the state of the handler.
    ///
    /// - clears `self.known_files`
    /// - adds `self.tmp_file` and `self.init_file` to `self.known_files`
    /// - clears `self.new_diffs`
    fn reset(&mut self) {
        log!(self.ip => "resetting request handler");
        self.known_files.clear();
        let is_new = self.known_files.insert((&self.tmp_file).into());
        debug_assert! { is_new }
        let is_new = self.known_files.insert((&self.init_file).into());
        debug_assert! { is_new }
        self.new_diffs.clear()
    }

    /// Reads the content of a file.
    fn read_content<P>(path: P, target: &mut Vec<u8>) -> Res<()>
    where
        P: AsRef<Path>,
    {
        use std::{fs::OpenOptions, io::Read};
        let path = path.as_ref();
        let mut file_reader = OpenOptions::new().read(true).write(false).open(path)?;
        file_reader.read_to_end(target)?;
        Ok(())
    }

    /// Handler's entry point.
    pub fn run(&mut self) -> Res<()> {
        self.send_first_init()?;

        log!(self.ip => "starting sending diffs");

        // Main loop. Look for new diffs, send them, check if init file was changed, again.
        //
        // The loop does two thing: find *all* the new diff files, and then send all these files.
        // Now, at any point while it's doing the profiler can be relaunched, meaning the diff files
        // will disappear and a new init file will be created. Most of the boiling plate in this
        // loop takes care of this "relaunch" case.
        'send_diffs: loop {
            if self.reset_if_init_changed()? {
                continue 'send_diffs;
            }

            // Gather the new diff.
            //
            // In case of an error, check whether the init file has changed. If it has, discard the
            // error and start sending diffs again.
            let new_stuff = match self.gather_new_diffs() {
                Ok(new_stuff) => new_stuff,
                Err(e) => {
                    if self.reset_if_init_changed()? {
                        continue 'send_diffs;
                    } else {
                        bail!(e)
                    }
                }
            };

            if new_stuff {
                // Same deal as diff gathering. If `send_new_diffs` errors but the init file has
                // changed, discard the error and start sending diffs again.
                match self.send_new_diffs() {
                    Ok(()) => (),
                    Err(e) => {
                        if self.reset_if_init_changed()? {
                            continue 'send_diffs;
                        } else {
                            bail!(e)
                        }
                    }
                }

                // Sending ping (acknowledgment) message.
                //
                // This will cause the client to send a corresponding `Pong` message that will be
                // handled by `self.drain_messages` bellow, indicating that diff-sending should
                // resume.
                let message = OwnedMessage::Ping(self.ping_label.clone());
                self.sender
                    .send_message(&message)
                    .chain_err(|| "while sending ping message")?;

                let should_stop = self.drain_messages()?;
                if should_stop {
                    break 'send_diffs;
                }

                if self.reset_if_init_changed()? {
                    continue 'send_diffs;
                }
            }
        }

        Ok(())
    }

    /// Sends the new diff files in `self.new_diffs`.
    ///
    /// - assumes `!self.new_diffs.is_empty()`
    /// - guarantees `self.new_diffs.is_empty()`
    pub fn send_new_diffs(&mut self) -> Res<()> {
        debug_assert! { !self.new_diffs.is_empty() }

        // Sort files so that they're in lexicographical order.
        self.new_diffs
            .sort_by(|f_1, f_2| f_1.file_name().cmp(&f_2.file_name()));

        // Number of diffs to send.
        let len = self.new_diffs.len();
        // Counts diff sent (from `1` to `len`) so that we know when we reach the last one.
        let mut cnt = 0;

        log!(self.ip => "sending {} diffs", self.new_diffs.len());

        for file in self.new_diffs.drain(0..) {
            cnt += 1;

            let is_last = cnt == len;
            let content_start = if is_last {
                "1".to_string()
            } else {
                "0".to_string()
            };

            let mut content = Vec::with_capacity(500);

            'content_might_be_empty: loop {
                content.extend(content_start.clone().into_bytes());
                Self::read_content(file.path(), &mut content)?;
                if content == content_start.as_bytes() {
                    content.clear();
                    continue 'content_might_be_empty;
                } else {
                    break 'content_might_be_empty;
                }
            }

            log!(self.ip => "sending content of diff file `{}`", file.path().to_string_lossy());

            let msg = OwnedMessage::Binary(content);
            self.sender.send_message(&msg).chain_err(|| {
                format!(
                    "while sending content of file `{}`",
                    file.path().to_string_lossy()
                )
            })?;
        }

        Ok(())
    }

    /// Gathers the new diff files.
    ///
    /// - diff files to send will be in `self.new_diffs`.
    /// - assumes `self.new_diffs.is_empty()`.
    /// - returns `true` if there was at list one new diff found (equivalent to
    ///     `!self.new_diffs.is_empty()`)
    pub fn gather_new_diffs(&mut self) -> Res<bool> {
        debug_assert! { self.new_diffs.is_empty() }

        let dir = std::fs::read_dir(&self.dump_dir)
            .chain_err(|| format!("while reading dump directory `{}`", self.dump_dir))?;

        for file in dir {
            let file =
                file.chain_err(|| format!("while reading dump directory `{}`", self.dump_dir))?;
            let file_type = file.file_type().chain_err(|| {
                format!(
                    "failed to retrieve file/dir information for `{}`",
                    file.file_name().to_string_lossy()
                )
            })?;

            if !file_type.is_file() {
                continue;
            }

            let is_new = self.known_files.insert(file.file_name());

            // File is
            if is_new {
                self.new_diffs.push(file)
            }
        }
        Ok(!self.new_diffs.is_empty())
    }
}

/// Init-related functions.
impl Handler {
    /// Waits for an init file to exist and sends it to the client.
    pub fn send_first_init(&mut self) -> Res<()> {
        log!(self.ip => "watching for first init file");
        debug_assert_eq! { self.init_last_modified, None }
        loop {
            let was_sent = self
                .try_send_init()
                .chain_err(|| "while sending the first init message")?;
            if was_sent {
                log!(self.ip => "first init file sent");
                return Ok(());
            }
            sleep(std::time::Duration::from_millis(200))
        }
    }

    /// If init file was changed since first init, reset the handler.
    pub fn reset_if_init_changed(&mut self) -> Res<bool> {
        let init_changed = self.try_send_init()?;
        if init_changed {
            log!(self.ip => "init file was modified recently");
            self.reset();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Reads the init file and sends it to the client.
    ///
    /// Returns `was_sent`: `true` if an init file was actually sent.
    pub fn try_send_init(&mut self) -> Res<bool> {
        if let Some(init_content) = self
            .try_read_init()
            .chain_err(|| "on first read of init file")?
        {
            let init_msg = OwnedMessage::Binary(init_content);
            self.sender
                .send_message(&init_msg)
                .chain_err(|| format!("while sending content of init file `{}`", self.init_file))?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Reads the init file in the dump directory.
    ///
    /// Returns `None` when
    ///
    /// - init file does not exist, **or**
    /// - `init_last_modified = Some(t)` and the init file was modified more recently than `t`.
    ///
    /// This function is used *i)* during initialization to read the init file, and *ii)* to check
    /// whether it was overwritten by a new run to relaunch everything.
    ///
    /// If the result isn't `None`, `self.init_last_modified` is updated to the date of last
    /// modification of the init file.
    pub fn try_read_init(&mut self) -> Res<Option<Vec<u8>>> {
        use std::path::PathBuf;
        let mut init_path = PathBuf::new();
        init_path.push(&self.dump_dir);
        init_path.push(&self.init_file);

        if !(init_path.exists() && init_path.is_file()) {
            return Ok(None);
        }

        // Time of last modification of init file.
        let last_modified = init_path
            .metadata()
            .chain_err(|| {
                format!(
                    "could not retrieve metadata of init file `{}`",
                    init_path.to_string_lossy()
                )
            })?
            .modified()
            .chain_err(|| {
                format!(
                    "could not retrieve time of last modification of init file`{}`",
                    init_path.to_string_lossy()
                )
            })?;

        // Is it our first time loading the init file?
        if let Some(lm) = self.init_last_modified.as_mut() {
            // Not the first time, has the init file changed?
            if last_modified != *lm {
                // Yes, update
                debug_assert! { last_modified <= *lm }
                *lm = last_modified
            } else {
                // No, no need to load the file.
                return Ok(None);
            }
        } else {
            // First time, update time of last modification.
            self.init_last_modified = Some(last_modified)
        }

        let mut init_content = Vec::with_capacity(600);
        Self::read_content(init_path, &mut init_content)
            .chain_err(|| format!("while reading content of init file `{}`", self.init_file))?;

        if init_content.is_empty() {
            return Ok(None);
        } else {
            Ok(Some(init_content))
        }
    }

    /// Handles the client's incoming messages.
    ///
    /// Returns `should_stop`: `true` if the client requested to close the connection.
    fn drain_messages(&mut self) -> Res<bool> {
        let mut should_stop = false;

        for message in self.recver.incoming_messages() {
            let message = message.chain_err(|| "while retrieving message")?;
            match message {
                OwnedMessage::Close(_) => {
                    should_stop = true;
                    break;
                }
                OwnedMessage::Ping(label) => {
                    let message = OwnedMessage::Pong(label);
                    self.sender
                        .send_message(&message)
                        .chain_err(|| "while sending `Pong` message to client")?
                }
                OwnedMessage::Pong(pong_label) => {
                    if pong_label == self.ping_label {
                        break;
                    }
                    bail!("unexpected pong message from client")
                }

                OwnedMessage::Text(blah) => {
                    bail!("unexpected text message from client: `{}`", blah)
                }
                OwnedMessage::Binary(_) => bail!("unexpected binary message from client"),
            }
        }

        Ok(should_stop)
    }
}
