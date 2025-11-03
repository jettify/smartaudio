use smartaudio::Response;
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
    println!("Parssing raw packet:");
    for b in &raw_settings_v20[0..raw_settings_v20.len()] {
        let reult = parser.push_byte(*b);
        match reult {
            Ok(Some(Response::GetSettings(settings))) => {
                println!("{:#?}", settings);
                println!("\n");
                println!("Version:     {:?}", settings.version);
                println!("Channel:     {:?}", settings.channel);
                println!("Power Level: {:?}", settings.power_level);
                println!("Unlocked:    {:?}", settings.unlocked);
            }
            Err(e) => eprintln!("Error parsing packet: {:?}", e),
            _ => (),
        }
    }
}
