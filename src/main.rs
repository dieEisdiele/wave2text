use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::io;


fn main() {
    // Splash screen
    let notice: &str = r#"wave2text  Copyright (C) 2022  Tom Su
    This program comes with ABSOLUTELY NO WARRANTY.
    This is free software, and you are welcome to redistribute it under certain
    conditions.
    See LICENSE.txt for details.
    
    "#;
    let logo: &str = r#"
       _          _   ___       _____   _____
      / \        / | |   |    _|     | |     \
    -'   \  /\  /  |_|   |   |       | |      `-
          \/  \/         |___|       |_|
    
    "#;
    println!("{}{}", notice, logo);

    // Load settings from JSON file and get the pulse shape
    let settings: Settings = match get_settings("settings.json") {
        Ok(json) => json,
        Err(error) => {
            println!("error loading ini.json: {}", error);
            println!("Loading default settings...");
            let default = Settings {
                pulse_path: String::from("pulse.txt"),
                sample_rate_hz: 100000.0,
                phase_duration_presets: Vec::new()
            };
            default
        },
    };
    let pulse: Vec<f64> = match get_pulse_shape(&settings.pulse_path) {
        Ok(vec) => vec,
        Err(error) => panic!("error loading pulse shape from file: {}", error),
    };

    // Define vectors to store waveform in
    let waveform: Vec<f64> = Vec::new();
    let wave_history: Vec<&str> = Vec::new();

    // Main program loop
    loop {
        match terminal_menu() {
            1 => println!("Edit waveform."),
            2 => println!("Clear waveform."),
            3 => println!("Save waveform."),
            4 => if confirm_exit() == true {
                break
            } else {continue},
            _ => (),
        };
    };
}


/// Loads saved settings from JSON file.
fn get_settings(file_path: &str) -> Result<Settings, Box<dyn Error>> {
    let ini_data: String = fs::read_to_string(file_path)?;
    let settings: Settings = serde_json::from_str(&ini_data)?;

    Ok(settings)
}

/// Loads pulse shape from TXT file.
fn get_pulse_shape(file_path: &str) -> Result<Vec<f64>, Box<dyn Error>> {
    let pulse_string = fs::read_to_string(file_path)?;
    let pulse_string: Vec<&str> = pulse_string.split("\r\n").collect();
    let mut pulse_shape: Vec<f64> = Vec::new();
    for sample in 0..pulse_string.len() {
        pulse_shape.push(pulse_string[sample].parse::<f64>()?);
    }
    Ok(pulse_shape)
}

/// Brings up the menu and returns the input if valid.
fn terminal_menu() -> u8 {
    // Define strings for showing user options and how to call them.
    let menu: &str = r#"
What would you like to do?
    
    [1]. Edit waveform.
    [2]. Clear waveform.
    [3]. Save waveform.
    [4]. Exit program.
    "#;
    let input_prompt: &str = "Please enter a number 1-4.";

    // Print menu and get user input
    println!("{}\n{}", menu, input_prompt);
    loop {
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => (),
            Err(error) => {
                println!("error: {}", error);
                println!("{}", input_prompt);
                continue;
            },
        }

        match input.trim().parse::<u8>() {
            Ok(num) => if num > 0 && num < 5 {
                return num
            } else {
                    println!("error: number outside valid range");
                    println!("{}", input_prompt);
                    continue;
            },
            Err(error) => {
                println!("error: {}", error);
                println!("{}", input_prompt);
                continue;
            },
        };
    }
}

/// Constructs a new waveform in which the provided pulse repeats as specified, and appends it to the end of the existing waveform.
fn wave_gen(waveform_pre: Vec<f64>, pulse_shape: Vec<f64>, sample_rate_hz: f64, phase_hz: f64, duration_sec: f64) -> Vec<f64> {
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

/// Confirms that the user wants to exit.
fn confirm_exit() -> bool {
    println!("Are you sure you want to quit? Enter [Y] to confirm, or press any other button to return to the menu.");

    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => (),
        Err(_) => return false
        }
    input = input.trim().to_lowercase();
    
    if input == "y" || input == "yes" {
        return true
    } else {
        return false
    }
}


/// Format to store/read settings in JSON file.
#[derive(Serialize, Deserialize)]
struct Settings {
    pulse_path: String,
    sample_rate_hz: f64,
    phase_duration_presets: Vec<(f64, f64)>,
}
