mod pilot;

use captrs::*;

use winapi::um::winuser::*;

use std::collections::HashMap;
use std::time::Instant;
use std::net::UdpSocket;


const LAMPS_PORT: &str = "38899";

fn main() {
    // Initialize lamps IPs
    let lamps_ips:Vec<&str> = vec![
        "Your IPs here"
    ];


    // Initialize UDP Socket
    println!("Initializing UDP Socket...");
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();

    // Get previous lamps state
    println!("Getting previous lamps state...");
    let mut lamps_state: HashMap<String, String> = HashMap::new();
    for ip in lamps_ips.iter() {
        let msg = pilot::get_pilot_message();

        socket.send_to(msg.as_bytes(), format!("{}:{}", ip, LAMPS_PORT)).unwrap();

        let mut buf = [0; 1024];
        let (amt, _) = socket.recv_from(&mut buf).unwrap();

        let response = String::from_utf8_lossy(&buf[..amt]);
        
        lamps_state.insert(ip.to_string(), response.to_string());
    }

    let hola = lamps_state.iter().nth(0).unwrap();
    pilot::set_pilot_message_from_previous_state(hola.1.to_string());

    // Initialize capture
    println!("Initializing capture...");
    let mut capturer = Capturer::new(0).unwrap();

    let mut previous_frame = capturer.capture_frame().unwrap();

    loop {
        // Start timer
        let start = Instant::now();

        // Capture frame or fallback to previous frame
        let frame = capturer.capture_frame().unwrap_or(previous_frame);
        previous_frame = frame.clone();

        // Get most common color
        // let selected_color = get_most_common_color(frame);

        // Get average color
        let selected_color = get_average_color(frame);

        
        // Send color to lamps
        for ip in lamps_ips.iter() {
            let msg = pilot::set_pilot_message(
                selected_color.0.into(),
                selected_color.1.into(),
                selected_color.2.into(), 
                100,
                true
            );

            let address = format!("{}:{}", ip, LAMPS_PORT);
            
            socket.send_to(&msg.as_bytes(), address).expect("Could not send message");
        }

        println!("Color set to: {:?} - {}ms", selected_color, start.elapsed().as_millis());

        // If ESC is pressed, exit
        unsafe {if GetKeyState(VK_ESCAPE) != 0 {
            break;
        }}
    }

    // Restore previous lamps state
    println!("Restoring previous lamps state...");
    for (ip, state) in lamps_state.iter() {
        let msg = pilot::set_pilot_message_from_previous_state(state.to_string());

        let address = format!("{}:{}", ip, LAMPS_PORT);
        
        socket.send_to(&msg.as_bytes(), address).expect("Could not send message");
    }

    println!("Byebye!");
}



fn get_average_color(pixels: Vec<Bgr8>) -> (u64, u64, u64) {    
    let mut r: u64 = 0;
    let mut g: u64 = 0;
    let mut b: u64 = 0;

    // Filter (0,0,0) pixels
    let filtered_pixels = pixels.iter().filter(|pixel| pixel.r != 0 || pixel.g != 0 || pixel.b != 0);
    let pixel_count = filtered_pixels.clone().count() as u64;

    if pixel_count == 0 {
        return (0, 0, 0);
    }
    
    for pixel in filtered_pixels {
        r += pixel.r as u64;
        g += pixel.g as u64;
        b += pixel.b as u64;
    }

    return (
        (r / pixel_count),
        (g / pixel_count),
        (b / pixel_count),
    );
}


fn _get_most_common_color(pixels: Vec<Bgr8>) -> (u8, u8, u8) {
    let mut colors: HashMap<(u8, u8, u8), u32> = HashMap::new();

    for pixel in pixels {
        let color = (pixel.r, pixel.g, pixel.b);
        let count = colors.entry(color).or_insert(0);
        *count += 1;
    }
    
    let mut most_common_color = *colors.keys().nth(0).unwrap();

    if colors.len() > 0 {
        for (color, count) in colors.iter() {
            if count > colors.get(&most_common_color).unwrap() {
                most_common_color = *color;
            }
        }
    }

    return most_common_color;
}