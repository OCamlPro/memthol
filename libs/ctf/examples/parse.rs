use ctf::prelude::*;

fn main() {
    match run() {
        Ok(()) => std::process::exit(0),
        Err(e) => {
            eprintln!("|===| Error");
            for line in e.to_string().lines() {
                eprintln!("| {}", line)
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

    let mut parser = ctf::Parser::new(&data)?;
    parser.work()?;

    Ok(())
}

fn get_path() -> Res<String> {
    let mut args = std::env::args();
    ignore(args.next());
    args.next()
        .ok_or_else(|| "expected file path as argument, found nothing".to_string())
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
