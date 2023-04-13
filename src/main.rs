use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;

#[derive(Serialize, Deserialize)]
struct Settings {
    pulse_path: String,
    sample_rate_hz: f64,
}

fn get_settings(file_path: &str) -> Result<Settings, Box<dyn Error>> {
    let ini_data: String = fs::read_to_string(file_path)?;
    let settings: Settings = serde_json::from_str(&ini_data)?;

    Ok(settings)
}

fn wave_gen(waveform_pre: Vec<f64>, pulse_shape: Vec<f64>, phase_hz: f64, sample_rate_hz: f64, duration_sec: f64) -> Vec<f64> {
    let mut waveform: Vec<f64> = Vec::from(waveform_pre);

    let period_sec: f64 = 1.0/phase_hz;
    let pulse_count_final = (phase_hz * duration_sec).ceil() as u32;
    let wave_len_final: f64 = (sample_rate_hz * duration_sec).round();

    let mut waveform_new: Vec<f64> = Vec::new();
    for pulse_count in 0..pulse_count_final {
        waveform_new.extend(&pulse_shape);

        let wave_len_target = f64::min(wave_len_final, (period_sec * sample_rate_hz * (pulse_count as f64)).round()) as usize;
        let zeros: Vec<f64> = vec![0.0; wave_len_target - waveform_new.len()];
        waveform_new.extend(zeros)
    }

    waveform.extend(waveform_new);
    waveform
}

fn main() {
    let ini_path: &str = "ini.json";
    let settings: Settings = match get_settings(ini_path) {
        Ok(settings) => settings,
        Err(error) => panic!("Problem loading ini.json: {}", error)
    };
    let pulse: String = fs::read_to_string(settings.pulse_path)
        .expect("Error reading pulse shape");
    println!("{}", pulse)
}
