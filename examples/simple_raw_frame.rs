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
    println!("Parsing raw frames from chunks:");
    // A transport may return chunks ending anywhere within a frame.
    for chunk in raw_settings_v20.chunks(4) {
        let mut input = chunk;
        loop {
            match parser.next_frame(&mut input) {
                Ok(Some(frame)) => println!("{:#?}", frame),
                Ok(None) => break,
                Err(error) => eprintln!("Error parsing packet: {:?}", error),
            }
        }
    }
}
