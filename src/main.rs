use serde::{Deserialize, Serialize};
use std::error::Error;
use std::any;
use std::fs;
use std::io;


fn main() {
    println!("wave2text  Copyright (C) 2023  Tom Su
This program comes with ABSOLUTELY NO WARRANTY.
This is free software, and you are welcome to redistribute it under certain
conditions.
See LICENSE.txt for details.\n");

    // Load settings from JSON file and get the pulse shape
    let settings_filepath: &str = "settings.json";
    let mut settings: Settings = match get_settings(settings_filepath) {
        Ok(json) => {
            println!("Settings loaded from {}", settings_filepath);
            json
        },
        Err(error) => {
            println!("error loading settings from {}: {}", settings_filepath, error);
            println!("Loading default settings...");
            let default_settings = Settings {
                pulse_path: String::from("pulse.txt"),
                sample_rate_hz: 100000.0,
                presets_pulsefreq_duration_filler: Vec::new()
            };
            if confirm("Do you want to create a settings file from default settings? Enter [Y] to confirm, or any other key to continue.") {
                save_settings(settings_filepath, &default_settings);
            };
            default_settings
        }
    };

    let pulse: Vec<f64> = match get_pulse_shape(&settings.pulse_path) {
        Ok(vec) => {
            println!("Pulse shape loaded from {}", settings.pulse_path);
            vec
        },
        Err(error) => {
            println!("error loading pulse shape from {}: {}", settings.pulse_path, error);
            println!("Default pulse shape loaded");
            Vec::from([-1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0])
        }
    };

    let mut sample_rate_hz: f64 = settings.sample_rate_hz;
    println!("Sampling rate loaded: {} Hz", sample_rate_hz);

    let mut presets: Vec<WaveDescription> = settings.presets_pulsefreq_duration_filler
        .iter().map(|x| WaveDescription {
            pulse_frequency_hz: x.0,
            duration_sec: x.1,
            filler: x.2,
        }).collect();
    if presets.is_empty() {
        println!("No presets loaded");
    } else {
        println!("{} presets loaded:", presets.len());
        for (n, preset) in presets.iter().enumerate() {
            println!("
    {}
    Pulse frequency: {} Hz
    Duration:        {} s
    Filler:          {}", n, preset.pulse_frequency_hz, preset.duration_sec, preset.filler);
        };
    };

    println!(r#"

    _          _   ___       _____   _____
   / \        / | |   |    _|     | |     \
 -'   \  /\  /  |_|   |   |       | |      `-
       \/  \/         |___|       |_|
 "#);


    // Define vectors to store waveform in
    let mut waveform: Vec<f64> = Vec::new();
    let mut wave_history: Vec<String> = Vec::new();


    // Main program loop
    loop {
        let main_menu: String = format!("
    [1] Add to waveform manually.
    [2] Add presets to waveform.
    [3] View waveform history.
    [4] Clear waveform.
    [5] Export waveform.
    [6] View/edit presets.
    [7] Edit sampling rate ({} Hz).
    [8] Exit program.", sample_rate_hz);
        match menu(&main_menu, 8) {
            // TODO 1 Add checks that inputs are valid (i.e. floats that must be positive are positive)
            1 => {
                edit_waveform_manually(&mut waveform, &mut wave_history, &pulse, &sample_rate_hz);
            },


            2 => {
                display_presets(&presets);
                
                let preset_selection: Vec<usize> = loop {
                    let mut input = String::new();
                    println!("Please enter the preset(s) you would like to add.\nYou can specify more than one by inserting a space between each number.");
                    match io::stdin().read_line(&mut input) {
                        Ok(_) => (),
                        Err(error) => {
                            println!("error: {}", error);
                            continue;
                        }
                    };
                    match input.trim().split(' ').map(|x| x.parse::<usize>()).collect() {
                        Ok(vec) => break vec,
                        Err(error) => {
                            println!("error: {}", error);
                            continue;
                        }
                    };
                };

                for preset_index in preset_selection {
                    let preset_add: &WaveDescription = &presets[preset_index];
                    let preset_name: String = format!("Preset {}", preset_index);
                    wave_gen(&mut waveform, &mut wave_history, &preset_name, &pulse, &sample_rate_hz, preset_add);
                };
            },


            3 => {
                let wave_history_temp: Vec<String> = wave_history.to_vec();
                if wave_history_temp.is_empty() {
                    println!("Waveform is empty. Returning to menu...");
                    continue;
                }

                for item in wave_history_temp {
                    println!("{}", item);
                };
                println!("\nPress [Enter] to return to menu.");
                if io::stdin().read_line(&mut String::new()).is_ok() {};
            },


            4 => {
                if confirm("Are you sure you want to clear the current waveform? Enter [Y] to confirm, or any other key to return to menu without clearing.") {
                    waveform.clear();
                    wave_history.clear();
                    println!("Waveform cleared");
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
                let save_name: &str = save_name.trim();

                let waveform_string: String = waveform.iter().map(|x| x.to_string()).collect::<Vec<String>>().join("\n");
                let waveform_string: &str = waveform_string.trim();
                match fs::write(format!("saved/{}.txt", save_name), waveform_string) {
                    Ok(_) => println!("Waveform saved"),
                    Err(error) => {
                        println!("error: {}", error);
                        println!("Waveform was not saved");
                    }
                };
                
                let wave_history_string: String = wave_history.join("\n");
                let wave_history_string: &str = wave_history_string.trim();
                match fs::write(format!("saved/{}_history.txt", save_name), wave_history_string) {
                    Ok(_) => println!("Waveform history saved"),
                    Err(error) => {
                        println!("error: {}", error);
                        println!("Waveform history was not saved");
                    }
                };

                if confirm("Do you want to clear the current waveform? Enter [Y] to confirm, or any other key to return to menu without clearing.") {
                    waveform.clear();
                    wave_history.clear();
                    println!("Waveform cleared");
                };
            },


            // TODO 2 Allow user to edit presets
            // TODO 3 Allow user to save presets
            6 => {
                loop {
                    match menu("
    [1] Create new preset.
    [2] Edit existing preset.
    [3] Remove existing preset.
    [4] Save presets as future default.
    [5] Return to menu.", 5) {
                      1 => println!("Create new preset (WiP)"),


                      2 => println!("Edit existing preset (WiP)"),


                      3 => println!("Remove existing preset (WiP)"),

                      4 => {
                        display_presets(&presets);
                        if confirm("Are you sure you want to save these presets as the future default? This will overwrite the existing settings file. Enter [Y] to confirm, or any other key to go back without saving.") {
                            settings.presets_pulsefreq_duration_filler = presets.iter().map(|x| (x.pulse_frequency_hz, x.duration_sec, x.filler)).collect();
                            save_settings(settings_filepath, &settings);
                        };
                      },


                      5 => break,
                      _ => (),
                    };
                };
            },


            7 => {
                println!("Please enter new sampling rate.");
                sample_rate_hz = get_user_num("Please enter a positive number.");
                if confirm("Do you want to save this sampling rate as the future default? This will overwrite the existing settings file. Enter [Y] to confirm, or any other key to return to menu without saving.") {
                    settings.sample_rate_hz = sample_rate_hz;
                    save_settings(settings_filepath, &settings);
                };
            },


            8 => if confirm("Are you sure you want to exit the program? Enter [Y] to confirm, or any other key to return to menu.") {
                let presets_check: Vec<(f64, f64, f64)> = presets.iter().map(|x| (x.pulse_frequency_hz, x.duration_sec, x.filler)).collect();
                if (settings.sample_rate_hz != sample_rate_hz || settings.presets_pulsefreq_duration_filler != presets_check) && confirm("Do you want to save current settings as the future default? This will overwrite the existing settings file. Enter [Y] to confirm, or any other key to exit without saving.") {
                    settings.sample_rate_hz = sample_rate_hz;
                    settings.presets_pulsefreq_duration_filler = presets_check;
                    save_settings(settings_filepath, &settings);
                };
                println!("Exiting...");
                break;
            },


            _ => ()
        };
    };
}


/// Loads saved settings from JSON file.
fn get_settings(file_path: &str) -> Result<Settings, Box<dyn Error>> {
    let json_data: String = fs::read_to_string(file_path)?;
    let settings: Settings = serde_json::from_str(&json_data)?;
    Ok(settings)
}

/// Saves current settings from JSON file.
fn save_settings(file_path: &str, json_data: &Settings) {
    match fs::File::create(file_path) {
        Ok(buffer) => {
            match serde_json::to_writer(buffer, json_data) {
                Ok(_) => println!("{} updated", file_path),
                Err(error) => {
                    println!("error: {}", error);
                    println!("{} was not updated", file_path);
                }
            };
        },
        Err(error) => {
            println!("error: {}", error);
            println!("{} was not updated", file_path);
        }
    };
}

/// Loads pulse shape from TXT file.
fn get_pulse_shape(file_path: &str) -> Result<Vec<f64>, Box<dyn Error>> {
    let pulse_string = fs::read_to_string(file_path)?;
    let pulse_string: Vec<&str> = pulse_string.split("\r\n").collect();
    let mut pulse_shape: Vec<f64> = Vec::new();
    for sample in pulse_string {
        pulse_shape.push(sample.parse::<f64>()?);
    }
    Ok(pulse_shape)
}

/// Gets user input and returns a number if valid
fn get_user_num<T: std::str::FromStr>(prompt: &str) -> T {
    loop {
        let mut input = String::new();
        println!("{}", prompt);
        match io::stdin().read_line(&mut input) {
            Ok(_) => (),
            Err(error) => {
                println!("error: {}", error);
                continue;
            }
        };

        match input.trim().parse::<T>() {
            Ok(num) => return num,
            Err(_) => {
                println!("error: input cannot be parsed as {}", any::type_name::<T>());
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

    input == "y" || input == "yes"
}

/// Brings up the menu and returns the input if valid.
fn menu(menu_text: &str, max_input: u8) -> u8 {
    // Print menu and get user input
    println!("\n\nWhat would you like to do?\n{}\n\n", menu_text);
    loop {
        let input: u8 = get_user_num("Please enter a number 1-8.");

        if input > 0 && input <= max_input {
            return input;
        } else {
            println!("error: number outside valid range");
            continue;
        };
    };
}

/// Displays list of current presets.
fn display_presets(presets: &Vec<WaveDescription>) {
    if presets.is_empty() {
        println!("No presets found. Returning to menu...");
        return;
    }

    for (n, preset) in presets.iter().enumerate() {
        println!("
    {}
    Pulse frequency: {} Hz
    Duration:        {} s
    Filler:          {}", n, preset.pulse_frequency_hz, preset.duration_sec, preset.filler);
    };
}

/// User menu for manually editing the waveform.
fn edit_waveform_manually(waveform: &mut Vec<f64>, wave_history: &mut Vec<String>, pulse_shape: &Vec<f64>, sample_rate_hz: &f64) {
    let prompt: &str = "Please enter a positive number.";
    let manual_wave: WaveDescription = loop {
        println!("\nPulse phase (Hz)");
        let pulse_frequency_hz_temp: f64 = get_user_num(prompt);
        println!("Duration (s)");
        let duration_sec_temp: f64 = get_user_num(prompt);
        println!("Filler value");
        let filler_temp: f64 = get_user_num(prompt);

        println!("\nPulse frequency: {} Hz", pulse_frequency_hz_temp);
        println!("Duration: {} s", duration_sec_temp);
        println!("Filler: {}", filler_temp);
        if confirm("Are these parameters correct? Enter [Y] to confirm, or any other key to re-enter them.") {
            break WaveDescription {
                pulse_frequency_hz: pulse_frequency_hz_temp,
                duration_sec: duration_sec_temp,
                filler: filler_temp,
            };
        } else {
            continue;
        };
    };

    wave_gen(waveform, wave_history, "Manual", pulse_shape, sample_rate_hz, &manual_wave)
}

/// Constructs a new waveform in which the provided pulse repeats as specified, and appends it to the end of the existing waveform.
fn wave_gen(waveform: &mut Vec<f64>, wave_history: &mut Vec<String>, history_name: &str, pulse_shape: &Vec<f64>, sample_rate_hz: &f64, wave_desc: &WaveDescription) {
    let mut waveform_new: Vec<f64> = Vec::new();

    if wave_desc.pulse_frequency_hz == 0.0 {
        waveform_new = vec![wave_desc.filler; (sample_rate_hz * wave_desc.duration_sec) as usize];
    } else {
        let period_sec: f64 = 1.0/wave_desc.pulse_frequency_hz;
        let pulse_count_final = (wave_desc.pulse_frequency_hz * wave_desc.duration_sec).ceil() as usize;
        let wave_len_final: f64 = (sample_rate_hz * wave_desc.duration_sec).round();
        for pulse_count in 0..pulse_count_final {
            waveform_new.extend(pulse_shape);

            let wave_len_target = f64::min(wave_len_final, (period_sec * sample_rate_hz * (pulse_count as f64 + 1.0)).round()) as usize;
            let fill: Vec<f64> = vec![wave_desc.filler; wave_len_target - waveform_new.len()];
            waveform_new.extend(fill)
        };
    };

    waveform.extend(waveform_new);

    let wave_history_new: String = format!("{}
    Sampling rate:   {} Hz
    Pulse frequency: {} Hz
    Duration:        {} s
    Filler:          {}
", history_name, sample_rate_hz, wave_desc.pulse_frequency_hz, wave_desc.duration_sec, wave_desc.filler);
    wave_history.push(wave_history_new);
}


/// Format to store/read settings in JSON file.
#[derive(Serialize, Deserialize)]
struct Settings {
    pulse_path: String,
    sample_rate_hz: f64,
    presets_pulsefreq_duration_filler: Vec<(f64, f64, f64)>,
}

/// Parameters for wave generation.
struct WaveDescription {
    pulse_frequency_hz: f64,
    duration_sec: f64,
    filler: f64,
}
