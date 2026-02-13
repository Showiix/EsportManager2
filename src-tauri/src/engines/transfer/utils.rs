use rand::Rng;

pub fn normalize_position(pos: &str) -> String {
    match pos.to_lowercase().as_str() {
        "jungle" => "Jug".to_string(),
        "bot" => "Adc".to_string(),
        "support" => "Sup".to_string(),
        "top" => "Top".to_string(),
        "jug" => "Jug".to_string(),
        "mid" => "Mid".to_string(),
        "adc" => "Adc".to_string(),
        "sup" => "Sup".to_string(),
        _ => pos.to_string(),
    }
}

#[allow(dead_code)] // Used in tests; retained as a tested utility
pub(crate) fn probabilistic_round(value: f64, rng: &mut impl Rng) -> i64 {
    let floor = value.floor() as i64;
    let frac = value - value.floor();
    if frac > 0.0 && rng.gen::<f64>() < frac {
        floor + 1
    } else {
        floor
    }
}
