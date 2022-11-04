use serde_json::*;

pub fn set_pilot_message(r: u64, g: u64, b: u64, dimming: u64, state: bool) -> String{
    let msg = json!({
        "method": "setPilot",
        "params": {
            "r": r,
            "g": g,
            "b": b,
            "dimming": dimming,
            "state": state
        }
    });
    return msg.to_string();
}

pub fn set_pilot_message_from_previous_state(previous_state: String) -> String {
    let previous_json: Value = serde_json::from_str(&previous_state).unwrap();

    let result = previous_json["result"].as_object().unwrap();

    let msg: String;

    if result.contains_key("temp"){
        let temp = result["temp"].as_u64().unwrap();
        let dimming = result["dimming"].as_u64().unwrap();
        let state: bool = result["state"].as_bool().unwrap();

        msg = json!({
            "method": "setPilot",
            "params": {
                "temp": temp,
                "dimming": dimming,
                "state": state
            }
        }).to_string();
    } else {
        let r = result["r"].as_u64().unwrap();
        let g = result["g"].as_u64().unwrap();
        let b = result["b"].as_u64().unwrap();
        let dimming = result["dimming"].as_u64().unwrap();
        let state: bool = result["state"].as_bool().unwrap();

        msg = set_pilot_message(r, g, b, dimming, state);
    }

    return msg;
}

pub fn get_pilot_message() -> String{
    let msg = json!({
        "method": "getPilot",
        "params": {}
    });
    return msg.to_string();
}