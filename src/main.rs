mod light_communication;

use dxgcap::*;
use winapi::um::winuser::*;
use crossterm::{queue , terminal , cursor};

use std::collections::HashMap;
use std::time::Instant;
use std::fs;

/// Improves performance by skipping pixels. Reduces color accuracy.
/// 1 = no skipping, 2 = skip every other pixel, etc.
const PIXEL_SKIPPING: usize = 1;

/// If the color variation between iterations is lower than this value, 
/// the program will not send a new color to the lamps
const COLOR_VARIATION_THRESHOLD: u64 = 20;  // 0 = no variation, 255 = max variation

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
            exit_with_error("Error reading ips.txt");
        }
    }
    if lamps_ips.len() == 0 {
        exit_with_error("No lamps found in ips.txt");
    }
            

    // Initialize LightCommunication
    let mut light_communication = light_communication::LightCommunication::new(lamps_ips);

    // Get initial states
    println!("Getting initial states...");
    light_communication.get_initial_states();

    // Initialize capture
    println!("Initializing capture...");

    let mut capturer = DXGIManager::new(300).unwrap();
    let (mut previous_frame, (width , height)) = capturer.capture_frame().unwrap();

    // Get this window
    let this_window = unsafe { GetForegroundWindow() };
    
    // Clear terminal
    queue!(std::io::stdout(), terminal::Clear(terminal::ClearType::All)).unwrap();
    
    let mut previous_color = (0, 0, 0);
    loop {
        // Start timer
        let start = Instant::now();
 
        // Capture frame or fallback to previous frame
        let (frame, (_,_)) = capturer.capture_frame().unwrap_or((previous_frame, (width, height)));
        previous_frame = frame.clone();

        // Get most common color
        // let selected_color = get_most_common_color(frame);

        // Get average color
        let selected_color = get_average_color(frame);
        
        // Send color to lamps
        if calculate_color_variation(selected_color, previous_color) > COLOR_VARIATION_THRESHOLD {
            light_communication.set_color_all(selected_color, 0, 100, true);

            print_color_and_instructions(selected_color, start);

            previous_color = selected_color;
        }

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


fn calculate_color_variation(rgb: (u64, u64, u64), previous_rgb: (u64, u64, u64)) -> u64 {
    let r = (rgb.0 as i64 - previous_rgb.0 as i64).abs() as u64;
    let g = (rgb.1 as i64 - previous_rgb.1 as i64).abs() as u64;
    let b = (rgb.2 as i64 - previous_rgb.2 as i64).abs() as u64;

    return r + g + b;
}

fn get_average_color(pixels: Vec<BGRA8>) -> (u64, u64, u64) {    
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
    
    for pixel in filtered_pixels.step_by(PIXEL_SKIPPING) {
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

fn _get_most_common_color(pixels: Vec<BGRA8>) -> (u8, u8, u8) {
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


fn exit_with_error(error: &str) {
    println!("{}", error);
    println!("");
    println!("Press enter to exit...");

    std::io::stdin().read_line(&mut String::new()).unwrap();

    std::process::exit(1);
}

fn print_color_and_instructions(rgb: (u64, u64, u64), time_start: Instant) {
    // set cursor to 0,0
    queue!(std::io::stdout(), cursor::MoveTo(0, 0)).unwrap();
    // clear line
    queue!(std::io::stdout(), terminal::Clear(terminal::ClearType::CurrentLine)).unwrap();
    println!("Color set to: {:?} - {}ms", rgb, time_start.elapsed().as_millis());
    println!();
    println!("Press 'ESC' to quit");
}