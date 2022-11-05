mod light_communication;

use captrs::*;
use winapi::um::winuser::*;

use std::collections::HashMap;
use std::time::Instant;
use std::fs;

fn main() {
    // Initialize lamps IPs
    let mut lamps_ips = Vec::new();
    match fs::read_to_string("ips.txt") {
        Ok(lamps) => {
            for lamp in lamps.lines() {
                lamps_ips.push(lamp.to_string());
            }
        }
        Err(_) => {
            println!("No ips.txt file found, please create one with the ips of the lamps you want to control");
            return;
        }
    }
    if lamps_ips.len() == 0 {
        println!("No ips found in ips.txt, please add the ips of the lamps you want to control");
        return;
    }
            

    // Initialize LightCommunication
    let mut light_communication = light_communication::LightCommunication::new(lamps_ips);

    // Get initial states
    println!("Getting initial states...");
    light_communication.get_initial_states();

    // Initialize capture
    println!("Initializing capture...");

    let mut capturer = Capturer::new(0).unwrap();
    let mut previous_frame = capturer.capture_frame().unwrap();

    // Get this window
    let this_window = unsafe { GetForegroundWindow() };

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
        light_communication.set_color_all(selected_color, 0, 100, true);

        println!("Color set to: {:?} - {}ms", selected_color, start.elapsed().as_millis());

        // If ESC is pressed (high order bit is set)
        // and active window is this window
        unsafe {
            if GetKeyState(VK_ESCAPE) & 0x1000 != 0{
                let current_window = GetForegroundWindow();
                if current_window == this_window {
                    break;
                }
            }  
        } 
    }

    // Restore previous lamps state
    println!("Restoring previous lamps state...");
    light_communication.restore_initial_states();

    println!("Byebye!");
}



fn get_average_color(pixels: Vec<Bgr8>) -> (u64, u64, u64) {    
    let mut r: u64 = 0;
    let mut g: u64 = 0;
    let mut b: u64 = 0;

    // Filter (0,0,0) pixels
    let filtered_pixels = pixels.iter().filter(|pixel| pixel.r != 0 || pixel.g != 0 || pixel.b != 0);
    let pixel_count = filtered_pixels.clone().count() as u64;

    // If amount of pixels after filtering out black is less than 10%, return (1,1,1).
    // (0,0,0) is not accepted by the lamps
    if pixel_count < pixels.len() as u64 * 10 / 100 {
        return (1, 1, 1);
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