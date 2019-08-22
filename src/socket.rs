//! Websockets used by the server to communicate with the clients.

use websocket::message::OwnedMessage;

use crate::base::*;

/// A websocket server.
type Server = websocket::sync::Server<websocket::server::NoTlsAcceptor>;

/// A request.
type Request = websocket::server::upgrade::WsUpgrade<
    std::net::TcpStream,
    Option<websocket::server::upgrade::sync::Buffer>,
>;

/// Creates a websocket server at some address.
fn new_server(addr: &str, port: usize) -> Res<Server> {
    let server = Server::bind(&format!("{}:{}", addr, port))
        .chain_err(|| format!("while binding websocket server at `{}:{}`", addr, port))?;
    Ok(server)
}

/// Handles a client's incoming messages.
///
/// Returns `should_stop`, `true` if the client requested to close the connection.
fn drain_messages(
    receiver: &mut websocket::receiver::Reader<std::net::TcpStream>,
    sender: &mut websocket::sender::Writer<std::net::TcpStream>,
    ping_label: Option<&[u8]>,
) -> Res<bool> {
    let mut should_stop = false;

    println!("retrieving incoming messages");

    for message in receiver.incoming_messages() {
        let message = message.chain_err(|| "while retrieving message")?;
        match message {
            OwnedMessage::Close(_) => {
                should_stop = true;
                break;
            }
            OwnedMessage::Ping(label) => {
                let message = OwnedMessage::Pong(label);
                sender
                    .send_message(&message)
                    .chain_err(|| "while sending `Pong` message to client")?
            }
            OwnedMessage::Pong(label) => {
                if let Some(ping_label) = ping_label {
                    if label == ping_label {
                        break;
                    }
                }
                bail!("unexpected pong message from client")
            }

            OwnedMessage::Text(blah) => bail!("unexpected text message from client: `{}`", blah),
            OwnedMessage::Binary(_) => bail!("unexpected binary message from client"),
        }
    }

    println!("done with incoming messages");

    Ok(should_stop)
}

/// Handles a connection request.
fn handle_request(request: Request, dump_dir: String) -> Res<()> {
    println!("handling websocket connection request");
    let client = request
        .accept()
        .map_err(|(_, e)| e)
        .chain_err(|| "while accepting websocket connection")?;
    let ip = client
        .peer_addr()
        .chain_err(|| "while retrieving client's IP address")?;

    println!("accepted ip connection from `{}`", ip);

    let (mut receiver, mut sender) = client
        .split()
        .chain_err(|| "while splitting the client into receive/send pair")?;

    let ping_label = vec![6u8, 6u8, 6u8];

    // Path to the diff of files.
    let diff_dir = std::path::Path::new(&dump_dir);
    // Set of all the files sent so far.
    let mut known_files: Set<std::ffi::OsString> = Set::new();
    // Add memthol's tmp file.
    {
        let is_new = known_files.insert("tmp.memthol".into());
        assert! { is_new }
    }

    // Send the init file.
    {
        let mut init_file = diff_dir.to_path_buf();
        init_file.push("init.memthol");
        let is_new = known_files.insert("init.memthol".into());
        assert! { is_new }

        use std::{fs::OpenOptions, io::Read};
        let mut content = vec![];
        let mut file_reader = OpenOptions::new()
            .read(true)
            .write(false)
            .open(&init_file)
            .chain_err(|| format!("while opening file `{}`", init_file.to_string_lossy()))?;
        file_reader.read_to_end(&mut content).chain_err(|| {
            format!(
                "while reading the content of file `{}`",
                init_file.to_string_lossy()
            )
        })?;
        let msg = OwnedMessage::Binary(content);
        println!("sending content of init file");
        sender.send_message(&msg).chain_err(|| {
            format!(
                "while sending content of init file `{}`",
                init_file.to_string_lossy()
            )
        })?;
    }

    // New files discovered during one iteration of the loop below.
    let mut new_files = vec![];

    loop {
        // The following is guaranteed by the fact that we drain `new_files` in the loop if it's not
        // empty.
        debug_assert! { new_files.is_empty() }

        let dir = std::fs::read_dir(diff_dir).chain_err(|| {
            format!(
                "while reading dump directory `{}`",
                diff_dir.to_string_lossy()
            )
        })?;

        for file in dir {
            let file = file.chain_err(|| {
                format!(
                    "while reading dump directory `{}`",
                    diff_dir.to_string_lossy()
                )
            })?;
            let file_type = file.file_type().chain_err(|| {
                format!(
                    "failed to retrieve file/dir information for `{}`",
                    file.file_name().to_string_lossy()
                )
            })?;

            if !file_type.is_file() {
                continue;
            }

            let is_new = known_files.insert(file.file_name());

            // File is
            if is_new {
                new_files.push(file)
            }
        }

        // If there was no new file, sleep for a bit and continue.
        if new_files.is_empty() {
            std::thread::sleep(std::time::Duration::new(2, 0));
            continue;
        }

        // Sort files so that they're in lexicographical order.
        new_files.sort_by(|f_1, f_2| f_1.file_name().cmp(&f_2.file_name()));

        // Number of diffs to send.
        let len = new_files.len();
        // Counts diff sent (from `1` to `len`) so that we know when we reach the last one.
        let mut cnt = 0;

        for file in new_files.drain(0..) {
            use std::{fs::OpenOptions, io::Read};

            cnt += 1;

            let is_last = cnt == len;
            let content_start = if is_last {
                "1".to_string()
            } else {
                "0".to_string()
            };

            let mut content = content_start.into_bytes();
            let mut file_reader = OpenOptions::new()
                .read(true)
                .write(false)
                .open(file.path())
                .chain_err(|| format!("while opening file `{}`", file.path().to_string_lossy()))?;
            file_reader.read_to_end(&mut content).chain_err(|| {
                format!(
                    "while reading the content of file `{}`",
                    file.path().to_string_lossy()
                )
            })?;

            let msg = OwnedMessage::Binary(content);
            println!(
                "sending content of file `{}`",
                file.file_name().to_string_lossy()
            );
            sender.send_message(&msg).chain_err(|| {
                format!(
                    "while sending content of file `{}`",
                    file.path().to_string_lossy()
                )
            })?;
            // std::thread::sleep(std::time::Duration::new(0, 500_000_000));
        }

        let message = OwnedMessage::Ping(ping_label.clone());
        sender
            .send_message(&message)
            .chain_err(|| "while sending ping message")?;

        let is_dead = drain_messages(&mut receiver, &mut sender, Some(&ping_label))?;

        if is_dead {
            break;
        }
    }

    Ok(())
}

fn handle_requests(server: Server, dump_dir: String) -> Res<()> {
    for request in server.filter_map(Result::ok) {
        let dump_dir = dump_dir.clone();
        std::thread::spawn(move || handle_request(request, dump_dir).unwrap());
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
