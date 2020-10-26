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

    ctf::parse! {
        &data => |mut parser| {
            let (header, trace_info) = (parser.header(), parser.trace_info());
            println!("ctf header {{");
            println!("    time span {}", header.timestamp);
            println!("}}\n\ntrace info {{");
            println!("    sample rate: {}", trace_info.sample_rate);
            println!("    word size: {}", trace_info.word_size);
            println!("    exe name: {:?}", trace_info.exe_name);
            println!("    pid: {:?}", trace_info.pid);
            println!("}}\n");

            println!("parsing packets...\n");

            while let Some(mut packet_parser) = parser.next_packet()? {
                let header = packet_parser.header();
                println!("packet {}", header.id());
                println!("    time span: {}", header.timestamp);
                println!("    alloc span: {}", header.alloc_id);
                println!("{{");

                while let Some((clock, event)) = packet_parser.next_event()? {
                    println!("    {} @ {}", event.desc(), clock)
                }

                println!("}}\n")
            }
        }
    };

    Ok(())
}

fn get_path() -> Res<String> {
    let mut args = std::env::args();
    args.next();
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
