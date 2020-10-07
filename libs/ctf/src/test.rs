#[test]
fn event_header() {
    fn u32_to_le(num: u32) -> String {
        format!("{:0>32b}", num).chars().rev().collect()
    }
    fn u8_to_le(num: u8) -> String {
        format!("{:0>8b}", num).chars().rev().collect()
    }

    let timestamp = 1_524_611u32;
    let id = 11u8;

    println!("timestamp: {}, id: {}", timestamp, id);

    let mut le_bytes = 0u32;
    let mut offset = 0;

    println!("working on timestamp");
    for le_idx in 0..25 {
        use bitlab::*;

        if timestamp.get_bit(31 - le_idx).unwrap() {
            le_bytes = le_bytes.set_bit(offset + le_idx).unwrap()
        }
    }

    offset = 25;

    println!("working on id");
    for le_idx in 0..7 {
        use bitlab::*;

        if id.get_bit(7 - le_idx).unwrap() {
            le_bytes = le_bytes.set_bit(offset + le_idx).unwrap()
        }
    }

    println!("timestamp | {}", format!("{:0>32b}", timestamp));
    println!("timestamp | {} (le)", u32_to_le(timestamp));
    println!("       id | {}", format!("{:0>8b}", id));
    println!("       id | {} (le)", u8_to_le(id));
    println!(
        "timestamp | {:}",
        format!("{:0<32b}", timestamp.swap_bytes())
    );
    println!("       id | {:>32}", format!("{:0<8b}", id.swap_bytes()));
    println!(" le_bytes | {:b}", le_bytes);

    let bytes = le_bytes.to_be_bytes();
    let mut parser = crate::RawParser::new(&bytes);

    let header = parser.event_header().unwrap();

    println!("header {{ {}, {} }}", header.timestamp, header.id)
}
