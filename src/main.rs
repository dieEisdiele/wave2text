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
                filler: 0.0,
                phase_duration_presets: Vec::new()
            };
            // TODO Move settings display after match statement
            // TODO Enable proper preset display
            println!(r#"
    Pulse shape file: "/pulse.txt"
    Sampling rate:    100000 Hz
    Filler:           0.0
    No presets"#);
            default
        }
    };
    let mut sample_rate_hz: f64 = settings.sample_rate_hz;
    let mut filler: f64 = settings.filler;
    let pulse: Vec<f64> = match get_pulse_shape(&settings.pulse_path) {
        Ok(vec) => vec,
        Err(error) => {
            println!("error loading pulse shape from file: {}", error);
            println!("Loading default pulse shape...");
            Vec::from([-1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0])
        }
    };

    // Define vectors to store waveform in
    let mut waveform: Vec<f64> = Vec::new();
    let mut wave_history: Vec<String> = Vec::new();


    // Main program loop
    loop {
        match terminal_menu() {
            // TODO 2 Add special case for 0Hz (don't insert any pulses)
            // TODO 3 Allow user to use presets
            // TODO 4 Allow user to save presets
            1 => {
                let pulse_temp: Vec<f64> = pulse.to_vec();
                (waveform, wave_history) = edit_waveform(wave_history, waveform, pulse_temp, sample_rate_hz, filler);
            },


            2 => {
                let wave_history_temp: Vec<String> = wave_history.to_vec();
                if wave_history_temp.len() == 0 {
                    println!("Waveform is empty. Returning to menu...");
                } else {
                    for item in wave_history_temp {
                    println!("{}", item);
                    };

                    println!("\nPress any key to return to menu.");
                    let mut input = String::new();
                    match io::stdin().read_line(&mut input) {
                        Ok(_) => (),
                        Err(_) => ()
                    };
                };
            },


            3 => if confirm("Are you sure you want to clear the current waveform? Enter [Y] to confirm, or press any other key to return to menu.") {
                waveform.clear();
                wave_history.clear();
                println!("Waveform cleared.");
            } else {continue},


            // TODO 1 Allow user to save waveform to .TXT file
            4 => println!("Export waveform."),


            // TODO 5 Query user to save new settings
            5 => {
                sample_rate_hz = loop {
                    println!("\nSampling rate (Hz): {}", sample_rate_hz);
                    if confirm("Is this correct? Enter [Y] to confirm, or press any other key to enter a different sampling rate.") {
                        break sample_rate_hz
                    } else {
                        println!("Please enter new sampling rate.");
                        break get_user_float()
                    };
                };

                filler = loop {
                    println!("\nFiller value: {}", filler);
                    if confirm("Is this correct? Enter [Y] to confirm, or press any other key to enter a different filler.") {
                        break filler
                    } else {
                        println!("Please enter new filler.");
                        break get_user_float()
                    };
                };
            },


            6 => if confirm("Are you sure you want to exit the program? Enter [Y] to confirm, or press any other key to return to menu.") {
                println!("Exiting...");
                break
            } else {continue},


            _ => ()
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
    [2]. View waveform history.
    [3]. Clear waveform.
    [4]. Export waveform.
    [5]. View/edit settings.
    [6]. Exit program.
    "#;
    let input_prompt: &str = "Please enter a number 1-6.";

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
            }
        };

        match input.trim().parse::<u8>() {
            Ok(num) => if num > 0 && num < 7 {
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
            }
        };
    }
}

/// User menu for editing the current waveform.
fn edit_waveform(wave_history_pre: Vec<String>, waveform_pre: Vec<f64>, pulse_shape: Vec<f64>, sample_rate_hz: f64, filler: f64) -> (Vec<f64>, Vec<String>) {
    let (pulse_frequency_hz, duration_sec): (f64, f64) = loop {
        let (pulse_frequency_hz_temp, duration_sec_temp): (f64, f64) = get_wave_variables();
        println!("\nPulse frequency: {} Hz", pulse_frequency_hz_temp);
        println!("Duration: {} s", duration_sec_temp);
        if confirm("Are these parameters correct? Enter [Y] to confirm, or press any other key to re-enter them.") {
            break (pulse_frequency_hz_temp, duration_sec_temp)
        } else {
            continue
        };
    };

    let waveform_new: Vec<f64> = wave_gen(waveform_pre, pulse_shape, sample_rate_hz, filler, pulse_frequency_hz, duration_sec);

    let mut wave_history: Vec<String> = Vec::from(wave_history_pre);
    let wave_history_new: String = format!("
    Sampling rate:   {} Hz
    Pulse frequency: {} Hz
    Duration:        {} s
    Filler:          {}", sample_rate_hz, pulse_frequency_hz, duration_sec, filler);
    wave_history.push(wave_history_new);

    (waveform_new, wave_history)
}

/// Get user-defined variables
fn get_wave_variables() -> (f64, f64) {
    println!("\nPulse phase (Hz)");
    let pulse_frequency_hz: f64 = get_user_float();

    println!("Duration (s)");
    let duration_sec: f64 = get_user_float();

    (pulse_frequency_hz, duration_sec)
}

/// Gets user input and returns a float if valid
fn get_user_float() -> f64 {
    let input_prompt: &str = "Please enter a positive number.";
    loop {
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => (),
            Err(error) => {
                println!("error: {}", error);
                println!("{}", input_prompt);
                continue;
            }
        };

        match input.trim().parse::<f64>() {
            Ok(num) => return num,
            Err(error) => {
                println!("error: {}", error);
                println!("{}", input_prompt);
                continue;
            }
        };
    };
}

/// Constructs a new waveform in which the provided pulse repeats as specified, and appends it to the end of the existing waveform.
fn wave_gen(waveform_pre: Vec<f64>, pulse_shape: Vec<f64>, sample_rate_hz: f64, filler: f64, pulse_frequency_hz: f64, duration_sec: f64) -> Vec<f64> {
    let mut waveform: Vec<f64> = Vec::from(waveform_pre);

    let period_sec: f64 = 1.0/pulse_frequency_hz;
    let pulse_count_final = (pulse_frequency_hz * duration_sec).ceil() as u32;
    let wave_len_final: f64 = (sample_rate_hz * duration_sec).round();

    let mut waveform_new: Vec<f64> = Vec::new();
    for pulse_count in 0..pulse_count_final {
        waveform_new.extend(&pulse_shape);

        let wave_len_target = f64::min(wave_len_final, (period_sec * sample_rate_hz * (pulse_count as f64 + 1.0)).round()) as usize;
        let fill: Vec<f64> = vec![filler; wave_len_target - waveform_new.len()];
        waveform_new.extend(fill)
    };

    waveform.extend(waveform_new);
    waveform
}

/// Prompts user to confirm action before proceeding.
fn confirm(query: &str) -> bool {
    println!("{}", query);

    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => (),
        Err(_) => return false
        };
    input = input.trim().to_lowercase();

    if input == "y" || input == "yes" {
        return true
    } else {
        return false
    };
}


/// Format to store/read settings in JSON file.
#[derive(Serialize, Deserialize)]
struct Settings {
    pulse_path: String,
    sample_rate_hz: f64,
    filler: f64,
    phase_duration_presets: Vec<(f64, f64)>,
}
