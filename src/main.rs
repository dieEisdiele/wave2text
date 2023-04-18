use serde::{Deserialize, Serialize};
use std::error::Error;
use std::any;
use std::fs;
use std::io;


fn main() {
    // Splash screen
    let notice: &str = r#"wave2text  Copyright (C) 2023  Tom Su
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
            println!("\n\nerror loading ini.json: {}", error);
            println!("Loading default settings...");
            let default = Settings {
                pulse_path: String::from("pulse.txt"),
                sample_rate_hz: 100000.0,
                presets_phase_duration_filler: Vec::new()
            };
            // TODO 6 Move settings display after match statement
            // TODO 7 Enable proper preset display
            println!(r#"
    Pulse shape file: /pulse.txt
    Sampling rate:    100000 Hz
    Filler:           0
    No presets
"#);
            default
        }
    };
    let mut presets: Vec<(f64, f64, f64)> = settings.presets_phase_duration_filler;
    let mut sample_rate_hz: f64 = settings.sample_rate_hz;
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
        match terminal_menu(sample_rate_hz) {
            // TODO 1 Add checks that inputs are valid (i.e. floats that must be positive are positive)
            // TODO 2 Allow user to use presets
            // TODO 3 Allow user to save presets
            1 => {
                let pulse_temp: Vec<f64> = pulse.to_vec();
                (waveform, wave_history) = edit_waveform(waveform, pulse_temp, sample_rate_hz, wave_history);
            },


            2 => {
                let presets_temp: Vec<(f64, f64, f64)> = presets.to_vec();
                if presets_temp.len() == 0 {
                    println!("No presets found. Returning to menu...");
                    continue;
                }

                let input_prompt: &str = "Please enter the preset(s) you would like to add.\nYou can specify more than one by inserting a space between each number.";
                for (n, preset) in presets_temp.iter().enumerate() {
                    println!("
    {}
    Pulse frequency: {} Hz
    Duration:        {} s
    Filler:          {}", n, preset.0, preset.1, preset.2);
                };
                println!("{}", input_prompt);
                
                let preset_selection: Vec<u32> = loop {
                    let mut input = String::new();
                    match io::stdin().read_line(&mut input) {
                        Ok(_) => (),
                        Err(error) => {
                            println!("error: {}", error);
                            println!("{}", input_prompt);
                            continue;
                        }
                    };

                    match input.trim().split(" ").map(|x| x.parse::<u32>()).collect() {
                        Ok(vec) => break vec,
                        Err(error) => {
                            println!("error: {}", error);
                            println!("{}", input_prompt);
                            continue;
                        }
                    };
                };

                for preset_n in preset_selection {
                    let pulse_temp: Vec<f64> = pulse.to_vec();
                    let preset_add: (f64, f64, f64) = presets_temp[(preset_n) as usize];
                    let preset_name: String = format!("Preset {}", preset_n);
                    (waveform, wave_history) = wave_gen(waveform, pulse_temp, sample_rate_hz, preset_add.0, preset_add.1, preset_add.2, wave_history, &preset_name);
                };
            },


            3 => {
                let wave_history_temp: Vec<String> = wave_history.to_vec();
                if wave_history_temp.len() == 0 {
                    println!("Waveform is empty. Returning to menu...");
                    continue;
                }

                for item in wave_history_temp {
                    println!("{}", item);
                };
                println!("\nPress any key to return to menu.");
                let mut input = String::new();
                match io::stdin().read_line(&mut input) {
                    Ok(_) => (),
                    Err(_) => ()
                };
            },


            4 => {
                if confirm("Are you sure you want to clear the current waveform? Enter [Y] to confirm, or press any other key to return to menu without clearing.") {
                    waveform.clear();
                    wave_history.clear();
                    println!("Waveform cleared.");
                };
            },


            5 => {
                let mut save_name = String::new();
                println!("Please enter a file name for the current waveform.");
                loop {
                    match io::stdin().read_line(&mut save_name) {
                        Ok(_) => break,
                        Err(error) => {
                            println!("error: {}", error);
                            println!("Please enter a valid UTF-8 string.");
                            continue;
                        }
                    };
                };

                let waveform_string: String = waveform.iter().map(|x| x.to_string()).collect::<Vec<String>>().join("\n");
                let wave_history_string: String = wave_history.join("\n");
                let waveform_string: &str = waveform_string.trim();
                let wave_history_string: &str = wave_history_string.trim();
                let save_name: &str = save_name.trim();
                match fs::write(format!("saved/{}.txt", save_name), waveform_string) {
                    Ok(_) => println!("Waveform saved."),
                    Err(error) => {
                        println!("error: {}", error);
                        println!("Waveform was not saved.");
                    }
                };
                match fs::write(format!("saved/{}_history.txt", save_name), wave_history_string) {
                    Ok(_) => println!("Waveform history saved."),
                    Err(error) => {
                        println!("error: {}", error);
                        println!("Waveform history was not saved");
                    }
                };

                if confirm("Do you want to clear the current waveform? Enter [Y] to confirm, or press any other key to return to menu without clearing.") {
                    waveform.clear();
                    wave_history.clear();
                    println!("Waveform cleared.");
                };
            },


            // TODO 5 Allow user to edit presets.
            6 => println!("Edit presets (WiP)."),


            // TODO 4 Query user to save new sampling rate
            7 => {
                println!("Please enter new sampling rate.");
                sample_rate_hz = get_user_num("Please enter a positive number.");
            },


            8 => if confirm("Are you sure you want to exit the program? Enter [Y] to confirm, or press any other key to return to menu.") {
                println!("Exiting...");
                break;
            },


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

/// Gets user input and returns a number if valid
fn get_user_num<T: std::str::FromStr>(prompt: &str) -> T {
    loop {
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => (),
            Err(error) => {
                println!("error: {}", error);
                println!("{}", prompt);
                continue;
            }
        };

        match input.trim().parse::<T>() {
            Ok(num) => return num,
            Err(_) => {
                println!("error: input cannot be parsed as {}", any::type_name::<T>());
                println!("{}", prompt);
                continue;
            }
        };
    };
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
        return true;
    } else {
        return false;
    };
}

/// Brings up the menu and returns the input if valid.
fn terminal_menu(sample_rate_hz: f64) -> u8 {
    let input_prompt: &str = "Please enter a number 1-8.";

    // Print menu and get user input
    println!("\n\nWhat would you like to do?\n
    [1]. Add to waveform manually.
    [2]. Add presets to waveform.
    [3]. View waveform history.
    [4]. Clear waveform.
    [5]. Export waveform.
    [6]. View/edit presets.
    [7]. Edit sampling rate ({} Hz).
    [8]. Exit program.\n\n{}", sample_rate_hz, input_prompt);
    loop {
        let input: u8 = get_user_num(input_prompt);

        if input > 0 && input < 9 {
            return input;
        } else {
            println!("error: number outside valid range");
            println!("{}", input_prompt);
            continue;
        };
    };
}

/// User menu for manually editing the waveform.
fn edit_waveform(waveform_pre: Vec<f64>, pulse_shape: Vec<f64>, sample_rate_hz: f64, wave_history_pre: Vec<String>) -> (Vec<f64>, Vec<String>) {
    let input_prompt: &str = "Please enter a positive number.";
    let (pulse_frequency_hz, duration_sec, filler): (f64, f64, f64) = loop {
        println!("\nPulse phase (Hz)");
        let pulse_frequency_hz_temp: f64 = get_user_num(input_prompt);
        println!("Duration (s)");
        let duration_sec_temp: f64 = get_user_num(input_prompt);
        println!("Filler value");
        let filler_temp: f64 = get_user_num(input_prompt);

        println!("\nPulse frequency: {} Hz", pulse_frequency_hz_temp);
        println!("Duration: {} s", duration_sec_temp);
        println!("Filler: {}", filler_temp);
        if confirm("Are these parameters correct? Enter [Y] to confirm, or press any other key to re-enter them.") {
            break (pulse_frequency_hz_temp, duration_sec_temp, filler_temp);
        } else {
            continue;
        };
    };

    return wave_gen(waveform_pre, pulse_shape, sample_rate_hz, pulse_frequency_hz, duration_sec, filler, wave_history_pre, "Manual");
}

/// Constructs a new waveform in which the provided pulse repeats as specified, and appends it to the end of the existing waveform.
fn wave_gen(waveform_pre: Vec<f64>, pulse_shape: Vec<f64>, sample_rate_hz: f64, pulse_frequency_hz: f64, duration_sec: f64, filler: f64, wave_history_pre: Vec<String>, history_name: &str) -> (Vec<f64>, Vec<String>) {
    let mut waveform: Vec<f64> = Vec::from(waveform_pre);
    let mut waveform_new: Vec<f64> = Vec::new();

    if pulse_frequency_hz == 0.0 {
        waveform_new = vec![filler; (sample_rate_hz * duration_sec) as usize];
    } else {
        let period_sec: f64 = 1.0/pulse_frequency_hz;
        let pulse_count_final = (pulse_frequency_hz * duration_sec).ceil() as u32;
        let wave_len_final: f64 = (sample_rate_hz * duration_sec).round();
    
        for pulse_count in 0..pulse_count_final {
            waveform_new.extend(&pulse_shape);
    
            let wave_len_target = f64::min(wave_len_final, (period_sec * sample_rate_hz * (pulse_count as f64 + 1.0)).round()) as usize;
            let fill: Vec<f64> = vec![filler; wave_len_target - waveform_new.len()];
            waveform_new.extend(fill)
        };
    };

    waveform.extend(waveform_new);

    let mut wave_history: Vec<String> = Vec::from(wave_history_pre);
    let wave_history_new: String = format!("{}
    Sampling rate:   {} Hz
    Pulse frequency: {} Hz
    Duration:        {} s
    Filler:          {}
", history_name, sample_rate_hz, pulse_frequency_hz, duration_sec, filler);
    wave_history.push(wave_history_new);

    (waveform, wave_history)
}


/// Format to store/read settings in JSON file.
#[derive(Serialize, Deserialize)]
struct Settings {
    pulse_path: String,
    sample_rate_hz: f64,
    presets_phase_duration_filler: Vec<(f64, f64, f64)>,
}
