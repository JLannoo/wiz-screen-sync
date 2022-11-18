use std::net::UdpSocket;
use std::collections::HashMap;

use serde_json::*;

use crate::exit_with_error;

const LAMPS_PORT: &str = "38899";

/// This struct is used to communicate with the lamps
pub struct LightCommunication {
    /// List of lamps IPs
    lights: Vec<String>,
    /// List of lamps initial states.
    /// Has to be initialized with `get_initial_states()`
    lights_initial_state: HashMap<String, String>,
    /// Socket used to communicate with the lamps
    socket: UdpSocket,
}

impl LightCommunication {
    /// Create a new LightCommunication struct
    pub fn new(lights: Vec<String>) -> Self {
        let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
        socket.set_read_timeout(Some(std::time::Duration::from_millis(400))).unwrap();
        Self { lights, socket, lights_initial_state: HashMap::new() }
    }

    /// Set the color of a specific lamp
    /// 
    /// You have to set either rgb or temp
    /// 
    /// # Arguments
    /// * `ip` - The IP of the lamp
    /// * `rgb` - The RGB color to set
    /// * `temp` - The temperature to set
    /// * `dimming` - The dimming to set
    /// * `is_on` - If the lamp should be turned on or off
    fn set_color(&self, ip: &str, rgb: (u64, u64, u64), temp: u64, dimming: u64, is_on: bool) {
        let msg = self.set_pilot_message(rgb, temp, dimming, is_on);

        self.send_message_to_light(msg, ip);
    }

    /// Set the color of all the lamps
    /// 
    /// You have to set either rgb or temp
    /// 
    /// # Arguments
    /// * `ip` - The IP of the lamp
    /// * `rgb` - The RGB color to set
    /// * `temp` - The temperature to set
    /// * `dimming` - The dimming to set
    /// * `is_on` - If the lamp should be turned on or off
    pub fn set_color_all(&self, rgb: (u64, u64, u64), temp: u64, dimming: u64, is_on: bool) {
        for ip in self.lights.iter() {
            self.set_color(ip, rgb, temp, dimming, is_on)
        }
    }

    /// Set the dimming time of a specific lamp
    /// 
    /// # Arguments
    /// * `ip` - The IP of the lamp
    /// * `fade_in` - The fade in time
    /// * `fade_out` - The fade out time
    fn set_fade_speed(&self, ip: &str, fade_in: u64, fade_out: u64) {
        let msg = self.set_user_config_message(fade_in, fade_out);

        self.send_message_to_light(msg, ip);
    }

    /// Set the dimming time of all lamps
    /// 
    /// # Arguments
    /// * `ip` - The IP of the lamp
    /// * `fade_in` - The fade in time
    /// * `fade_out` - The fade out time
    pub fn set_fade_speed_all(&self, fade_in: u64, fade_out: u64) {
        for ip in self.lights.iter() {
            self.set_fade_speed(ip, fade_in, fade_out)
        }
    }

    /// Get the initial state of all the lamps
    /// 
    /// Store the initial state in `lights_initial_state`
    /// 
    /// This function has to be called before `restore_initial_states()`
    pub fn get_initial_states(&mut self) {
        for ip in self.lights.iter() {
            // Send getPilot message
            let get_pilot_reponse = self.send_message_to_light(self.get_pilot_message(), ip);

            // Send getUserConfig message
            let get_user_config_reponse = self.send_message_to_light(self.get_user_config_message(), ip);

            // Parse response
            let mut parsed_pilot: Value = serde_json::from_str(&get_pilot_reponse).unwrap();
            let parsed_user_config: Value = serde_json::from_str(&get_user_config_reponse).unwrap();

            // Add fadeIn and fadeOut from userConfig to pilot[result]
            parsed_pilot["result"]["fadeIn"] = parsed_user_config["result"]["fadeIn"].clone();
            parsed_pilot["result"]["fadeOut"] = parsed_user_config["result"]["fadeOut"].clone();

            self.lights_initial_state
                .insert(ip.to_string(), parsed_pilot.to_string());
        }
    }

    /// Restore the initial state of all the lamps
    pub fn restore_initial_states(&self) {
        for (ip, is_on) in self.lights_initial_state.iter() {
            let is_on: Value = serde_json::from_str(is_on).unwrap();

            let result = is_on["result"].as_object().unwrap();

            if result.contains_key("temp") {
                let temp = result["temp"].as_u64().unwrap();
                let dimming = result["dimming"].as_u64().unwrap();
                let is_on: bool = result["state"].as_bool().unwrap();
                let fade_in = result["fadeIn"].as_u64().unwrap();
                let fade_out = result["fadeOut"].as_u64().unwrap();

                self.set_color(ip, (0, 0, 0), temp, dimming, is_on);
                self.set_fade_speed(ip, fade_in, fade_out)
            } else {
                let r = result["r"].as_u64().unwrap();
                let g = result["g"].as_u64().unwrap();
                let b = result["b"].as_u64().unwrap();
                let dimming = result["dimming"].as_u64().unwrap();
                let is_on: bool = result["state"].as_bool().unwrap();
                let fade_in = result["fadeIn"].as_u64().unwrap();
                let fade_out = result["fadeOut"].as_u64().unwrap();

                self.set_color(ip, (r, g, b), 0, dimming, is_on);
                self.set_fade_speed(ip, fade_in, fade_out)
            }
        }
    }

    /// Send a message to a lamp and return the response
    fn send_message_to_light(&self, msg: String, ip: &str) -> String {
        match self.socket.send_to(msg.as_bytes(), format!("{}:{}", ip, LAMPS_PORT)) {
            Ok(_) => {},
            Err(_) => {
                exit_with_error(&format!("Error communicating with {} \nPlease make sure the IP is correct, the lamp is turned on and connected to the same network as this computer", ip));
            }
        }

        let mut buf = [0; 1024];
        let amt:usize;

        // Receive response
        match self.socket.recv_from(&mut buf) {
            Ok((a, _)) => {
                amt = a;
            }
            Err(_) => {
                amt = 0;
                exit_with_error(&format!("Error communicating with {} \nPlease make sure the IP is correct, the lamp is turned on and connected to the same network as this computer", ip));
            }
        }

        String::from_utf8_lossy(&buf[..amt]).to_string()
    }

    /// Create the message to get the pilot state
    fn set_pilot_message(&self, rgb: (u64, u64, u64), temp: u64, dimming: u64, is_on: bool) -> String {
        if temp != 0 {
            let msg = json!({
                "method": "setPilot",
                "params": {
                    "temp": temp,
                    "dimming": dimming,
                    "state": is_on
                }
            });
            return msg.to_string();
        } else {
            let msg = json!({
                "method": "setPilot",
                "params": {
                    "r": rgb.0,
                    "g": rgb.1,
                    "b": rgb.2,
                    "dimming": dimming,
                    "state": is_on
                }
            });
            return msg.to_string();
        }
    }

    /// Create the message to get the pilot state
    fn get_pilot_message(&self) -> String {
        let msg = json!({
            "method": "getPilot",
            "params": {}
        });
        return msg.to_string();
    }

    /// Create the message to get the user config
    fn get_user_config_message(&self) -> String {
        let msg = json!({
            "method": "getUserConfig",
            "params": {}
        });
        return msg.to_string();
    }

    /// Create the message to set the dimming time
    fn set_user_config_message(&self, fade_in: u64, fade_out: u64) -> String {
        let msg = json!({
            "method": "setUserConfig",
            "params": {
                "fadeIn": fade_in,
                "fadeOut": fade_out
            }
        });
        return msg.to_string();
    }
}