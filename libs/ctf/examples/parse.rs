use ctf::prelude::*;

fn main() {
    match run() {
        Ok(()) => std::process::exit(0),
        Err(e) => {
            eprintln!("|===| Error");
            for e in e.iter() {
                let mut pref = "| - ";
                for line in e.to_string().lines() {
                    eprintln!("{}{}", pref, line);
                    pref = "|   "
                }
            }
            eprintln!("|===|");
            std::process::exit(2)
        }
    }
}

fn run() -> Res<()> {
    let path = get_path()?;

    println!("running on {:?}", path);

    let data = read_file(&path)?;

    let mut last_packet_id = None;
    let mut event_count = 0;

    ctf::parse(
        &data,
        |packet_header: &ctf::ast::header::Packet, event_time, event: ctf::ast::event::Event| {
            let packet_id = packet_header.id;
            if Some(packet_id) != last_packet_id {
                event_count = 0;
                if packet_id > 0 {
                    println!("}}")
                }
                last_packet_id = Some(packet_id);
                println!(
                    "packet #{} {} {{",
                    packet_id,
                    packet_header.header.timestamp.pretty_time(),
                )
            }
            println!(
                "    event #{} @{}: {}",
                event_count,
                event_time,
                event.desc()
            );
            event_count += 1;
            Ok(())
        },
    )?;
    if last_packet_id.is_some() {
        println!("}}")
    }

    Ok(())
}

fn get_path() -> Res<String> {
    let mut args = std::env::args();
    ignore(args.next());
    args.next()
        .ok_or_else(|| "expected file path as argument, found nothing".into())
}
fn read_file(path: impl AsRef<std::path::Path>) -> Res<Vec<u8>> {
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    let mut buf = Vec::with_capacity(2048);
    {
        use std::io::Read;
        let _bytes_read = file.read_to_end(&mut buf).map_err(|e| e.to_string())?;
    }
    Ok(buf)
}
