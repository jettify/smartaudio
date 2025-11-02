use smartaudio::SmartAudioParser;

fn main() {
    let mut parser = SmartAudioParser::new();

    let raw_settings_v20: [u8; 10] = [
        0xAA, 0x55, // Headers
        0x09, // Version/Command
        0x06, // Length
        0x01, // Channel
        0x00, // Power Level
        0x1A, // Operation/Mode
        0x16, 0xE9, // Current Frequency 5865
        0x0A, // CRC8
    ];
    println!("Parssing packet:");
    for b in &raw_settings_v20[0..raw_settings_v20.len()] {
        let reult = parser.push_byte_raw(*b);
        match reult {
            Ok(Some(f)) => println!("{:?}", f),
            Err(e) => eprintln!("Error parsing packet: {:?}", e),
            _ => (),
        }
    }
}
