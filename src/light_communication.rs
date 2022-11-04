use std::net::UdpSocket;
use std::collections::HashMap;

use serde_json::*;

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
        Self { lights, socket, lights_initial_state: HashMap::new() }
    }

    /// Set the color of all the lamps
    pub fn set_color_all(&self, rgb: (u64, u64, u64), temp: u64, dimming: u64, is_on: bool) {
        for ip in self.lights.iter() {
            self.set_color(ip, rgb, temp, dimming, is_on)
        }
    }

    /// Set the color of a specific lamp
    fn set_color(&self, ip: &str, rgb: (u64, u64, u64), temp: u64, dimming: u64, is_on: bool) {
        let msg = self.set_pilot_message(rgb, temp, dimming, is_on);

        self.socket
            .send_to(msg.as_bytes(), format!("{}:{}", ip, LAMPS_PORT))
            .unwrap();
    }

    ///
    pub fn get_initial_states(&mut self) {
        for ip in self.lights.iter() {
            let msg = self.get_pilot_message();

            self.socket
                .send_to(msg.as_bytes(), format!("{}:{}", ip, LAMPS_PORT))
                .unwrap();

            let mut buf = [0; 1024];
            let (amt, _) = self.socket.recv_from(&mut buf).unwrap();

            let response = String::from_utf8_lossy(&buf[..amt]);

            self.lights_initial_state
                .insert(ip.to_string(), response.to_string());
        }
    }

    pub fn restore_initial_states(&self) {
        for (ip, is_on) in self.lights_initial_state.iter() {
            let is_on: Value = serde_json::from_str(is_on).unwrap();

            let result = is_on["result"].as_object().unwrap();

            if result.contains_key("temp") {
                let temp = result["temp"].as_u64().unwrap();
                let dimming = result["dimming"].as_u64().unwrap();
                let is_on: bool = result["state"].as_bool().unwrap();

                self.set_color(ip, (0, 0, 0), temp, dimming, is_on);
            } else {
                let r = result["r"].as_u64().unwrap();
                let g = result["g"].as_u64().unwrap();
                let b = result["b"].as_u64().unwrap();
                let dimming = result["dimming"].as_u64().unwrap();
                let is_on: bool = result["state"].as_bool().unwrap();

                self.set_color(ip, (r, g, b), 0, dimming, is_on);
            }
        }
    }

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

    fn get_pilot_message(&self) -> String {
        let msg = json!({
            "method": "getPilot",
            "params": {}
        });
        return msg.to_string();
    }
}