use super::*;

// Functions
pub(crate) const PRINT: &str = "--[PRINT]--"; // Prints your previous lines to the console. 1st arg : Delay between each character.
pub(crate) const PRINT_LINE: &str = "--[PRINT_LINE]--"; // Prints your previous lines to the console with a newline at the end. 1st arg : Delay between each character.
pub(crate) const VAR: &str = "--[VAR]--"; // BETA. DOESN'T WORK YET.
pub(crate) const GOTO: &str = "--[GOTO]--"; // Go to a line. 1st arg : line to go to.
pub(crate) const MARKER: &str = "--[MARKER]--"; // Sets a marker to easily go to a specified line of the animessage via the command parameter '-m'. 1st arg : Marker name. 2nd arg : line to start from.
pub(crate) const WAIT: &str = "--[WAIT]--"; // Wait for some duration before continuing. 1st arg : Duration in ms.
pub(crate) const REPLACE: &str = "--[REPLACE]--"; // Replace text at a given line. 1st arg : line. 2nd arg : Replace from. 3rd arg : Replace to.
pub(crate) const DEL_LINE: &str = "--[DEL_LINE]--"; // Deletes a line, therefore offsetting the following lines by -1. 1st arg : line number.
pub(crate) const WAIT_FOR_INPUT: &str = "--[WAIT_FOR_INPUT]--"; // Wait for a keyboard key to be input before continuing. 1st arg : Key.
pub(crate) const OPEN_URL: &str = "--[OPEN_URL]--"; // Opens a given URL if the user allows it. 1st arg : URL.
pub(crate) const AUDIO: &str = "--[AUDIO]--"; // Plays a sound in the background. 1st arg : Sound path.
pub(crate) const TTS: &str = "--[TTS]--"; // NOT IMPLEMENTED YET. Reads some text using the default text-to-speech voice from your operating system.
pub(crate) const DRAW: &str = "--[DRAW]--"; // NOT IMPLEMENTED YET. Draws forms and presets and puts them in the print buffer. Will use args.
pub(crate) const IMAGE: &str = "--[IMAGE]--"; // Transforms an image into ASCII and then prints it to the console. 1st arg : Image path.
pub(crate) const VIDEO: &str = "--[VIDEO]--"; // NOT IMPLEMENTED YET. Transforms a video into ASCII and then prints it to the console. 1st arg : Video path.
pub(crate) const TITLE: &str = "--[TITLE]--"; // Sets the title of the terminal. 1st arg : title.
pub(crate) const CLEAR: &str = "--[CLEAR]--"; // Clears the terminal, leaving the terminal empty. Often used before print to seperate steps in your animessage.
pub(crate) const RESIZE: &str = "--[RESIZE]--"; // Resizes the terminal. 1st arg : columns. 2nd arg : rows.
pub(crate) const MOVE_CURSOR: &str = "--[MOVE_CURSOR]--"; // Moves the cursor to the specified location in columns * rows. 1st arg : columns. 2nd arg : rows.
pub(crate) const HIDE_CURSOR: &str = "--[HIDE_CURSOR]--"; // Hides the cursor. DUH !
pub(crate) const SHOW_CURSOR: &str = "--[SHOW_CURSOR]--"; // Shows the cursor. DUH !
pub(crate) const EMPTY: &str = "--[EMPTY]--"; // Inserts an empty line. By default, empty lines in your code have no effect to allow better formatting of your code.
pub(crate) const INCLUDE: &str = "--[INCLUDE]--"; // Includes another text file at the location. 1st arg : path to animessage to include.
pub(crate) const ESCAPE: &str = "--[ESCAPE]--"; // Disables functions in this line, entering the line in the print buffer as it is without this function in it.
pub(crate) const NOTE: &str = "--[NOTE]--"; // Used to write a comment. This has no effect.
pub(crate) const EXIT: &str = "--[EXIT]--"; // Close Animessage prematurely.

#[allow(unreachable_code)]
pub(crate) fn display_animessage(
    animessage_str: &str,
    relative_paths_ok: bool,
    debug: bool,
    no_exec: bool,
    start_index: usize,
) -> AnyResult<()> {
    let mut current_step = String::with_capacity(1024);
    // let mut expected_steps_n: u64 = 0;

    let mut gotos_cache: BTreeMap<usize, u64> = BTreeMap::new(); // K: goto line / V: goto iters number
                                                                 // let mut goto_iters_n: u64 = 0;
    let mut replaces_cache: BTreeMap<usize, [String; 2]> = BTreeMap::new();
    let mut vars: BTreeMap<String, Variable> = BTreeMap::new();

    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    // let mut audio_controller: Option<rodio::Sink> = None;

    let mut lines: Vec<String> = animessage_str // IDEA : Change to a BTreeMap<usize, String> if keeping lines number/index in place becomes necessary.
        .lines()
        .map(|s| s.to_string())
        .collect();
    let _orig_lines = lines.clone();

    let lines_number_count = lines.len().to_string().chars().count();

    if !debug {
        // clear_terminal();
        // move_cursor(0, 0);
        save_cursor_position();
    }

    let mut line_index: usize = start_index;
    'main_loop: while line_index + 1 <= lines.len() {
        let line = lines[line_index].clone();
        let line_trimmed = line.trim();
        let line_number = line_index + 1;

        if debug {
            println!(
                "{line_number:0fill$} | {line}",
                fill = lines_number_count,
                line_number = line_number,
                line = line
            );
        }

        match line {
            // PRINT
            _ if line_trimmed.starts_with(PRINT) => {
                let args = Args::parse(line_trimmed, 1, debug)?;
                let (print_interval_f32, print_interval) = duration_from_arg(args.get(0)); // We have verified that the number of args is correct so we can index as we please.

                if !current_step.is_empty() {

                    if print_interval_f32 == 0.0 {
                        if debug {
                            debug!("Printing this step all at once.");
                        }
                        if !no_exec {
                            print!("{}", &current_step);
                            flush_stdout();
                        }
                    } else {
                        if debug {
                            debug!("Printing this step character by character with an interval of {:?}.", print_interval);
                        }
                        if !no_exec {
                            for line_string in current_step.lines() {
                                for c in line_string.chars() {
                                    print!("{}", c);
                                    flush_stdout();
                                    sleep(print_interval);
                                }
                            }
                        }
                    }

                    current_step.clear();
                    if debug {
                        debug!("Current print buffer has been cleared.");
                    }
                }
            }

            // PRINT_LINE
            _ if line_trimmed.starts_with(PRINT_LINE) => {
                let args = Args::parse(line_trimmed, 1, debug)?;
                let (print_interval_f32, print_interval) = duration_from_arg(args.get(0)); // We have verified that the number of args is correct so we can index as we please.

                if !current_step.is_empty() {

                    if print_interval_f32 == 0.0 {
                        if debug {
                            debug!("Printing this step all at once.");
                        }
                        if !no_exec {
                            println!("{}", &current_step);
                            flush_stdout();
                        }
                    } else {
                        if debug {
                            debug!("Printing this step character by character with an interval of {:?}.", print_interval);
                        }
                        if !no_exec {
                            for line_string in current_step.lines() {
                                for c in line_string.chars() {
                                    print!("{}", c);
                                    flush_stdout();
                                    sleep(print_interval);
                                }
                                println!();
                            }
                        }
                    }

                    current_step.clear();
                    if debug {
                        debug!("Current print buffer has been cleared.");
                    }
                }
            }

            // VAR
            _ if line_trimmed.starts_with(VAR) => {
                // TODO: Changer la fonction pour aussi prendre en charge le mode GET.
                error!("UNSTABLE FUNCTION. Do not use this function.");
                return Ok(());

                let get_set_values = ["GET", "SET"];
                let get_set_error_msg =
                    r#"VAR functions' 1st arg must define a "GET" or "SET" mode."#;

                let args = Args::parse(line_trimmed, 4, debug)?;
                let mode = args.get(0);

                if !get_set_values.contains(&mode) {
                    error!("{}", get_set_error_msg);
                    std::process::exit(0);
                }

                let var_name = args.get(1);
                let var_type = args.get(2);
                let var_unparsed = args.get(3);

                if !no_exec {
                    Variable::new(var_name, var_type, &var_unparsed, Some(&mut vars), debug);
                }
            }

            // GOTO
            _ if line_trimmed.starts_with(GOTO) => {
                let args = Args::parse(line_trimmed, 1, debug)?;
                let goto_line_number: usize = args.get(0)
                    .parse::<usize>()
                    .unwrap_or_else(|_| {
                        error!("Can't convert arg into zero or a positive number, or your integer is too big.");
                        std::process::exit(0);
                    });

                if !gotos_cache.contains_key(&line_number) {
                    if debug {
                        debug!("Going to line {}", &goto_line_number);
                    }

                    line_index = goto_line_number - 2; // - 2 because we increment it by 1 afterwards and line_number == line_index + 1.
                    gotos_cache.insert(line_number, 1); // we don't care about the value, it's not processed yet
                } else {
                    if debug {
                        debug!(
                            "Not going to line {} : this GOTO function has already been executed.",
                            &goto_line_number
                        )
                    }
                }
            }

            // WAIT
            _ if line_trimmed.starts_with(WAIT) => {
                let args = Args::parse(line_trimmed, 1, debug)?;
                let wait_time_str = args.get(0);

                let (_, duration) = duration_from_arg(&wait_time_str);

                if debug {
                    debug!("Waiting for {:?} before continuing...", &duration);
                }

                if !no_exec {
                    sleep(duration);
                }
            }

            // REPLACE
            _ if line_trimmed.starts_with(REPLACE) => {
                let args = Args::parse(line_trimmed, 3, debug)?;
                let line_replace_number = args.get(0)
                    .parse::<usize>()
                    .unwrap_or_else(|_| {
                        error!("Can't convert arg into zero or a positive integer, or your integer is too big.");
                        std::process::exit(0)
                    });

                let replace_from = args.get(1);
                let replace_with = args.get(2);

                let line_to_modify = line_replace_number - 1;
                let replaces_cache_entry = replaces_cache.get(&line_to_modify);
                let array_replace: [String; 2] =
                    [replace_from.to_string(), replace_with.to_string()];

                if replaces_cache_entry.is_none() || replaces_cache_entry.unwrap() != &array_replace
                {
                    if debug {
                        debug!(
                            "Replacing {:?} with {:?} at line {:?}",
                            replace_from, replace_with, &line_replace_number
                        );
                    }
                    replaces_cache.insert(line_to_modify, array_replace);
                    lines[line_to_modify] =
                        lines[line_to_modify].replace(replace_from, replace_with);
                } else {
                    if debug {
                        debug!("Not replacing text at line {:?} : text has already been replaced with the same arguments.", &line_replace_number);
                    }
                }
            }

            // DEL_LINE
            _ if line_trimmed.starts_with(DEL_LINE) => {
                let args = Args::parse(line_trimmed, 1, debug)?;
                let del_line_number_str = args.get(0);
                let mut del_line_index = del_line_number_str // Still a line number at this point.
                    .parse::<usize>()
                    .unwrap_or_else(|_| {
                        error!("Can't convert arg into zero or a positive integer, or your integer is too big.");
                        std::process::exit(0)
                    });
                del_line_index -= 1; // Now it's actually a line index.

                lines.remove(del_line_index);

                if debug {
                    debug!("Deleted line {}", del_line_number_str)
                }
            }

            // WAIT_FOR_INPUT
            _ if line_trimmed.starts_with(WAIT_FOR_INPUT) => {
                let args = Args::parse(line_trimmed, 1, debug)?;
                let mut expected_key = args.get(0).to_string();

                if expected_key.chars().count() == 1 {
                    expected_key = expected_key.to_uppercase();
                } else {
                    expected_key = expected_key.to_string();
                }

                if !no_exec {
                    sleep(Duration::from_millis(250)); // To avoid chaining events unwillingly if expected_key is pressed for too long.
                }

                if debug {
                    debug!("Expecting key {:?} ...", &expected_key);
                }

                if !no_exec {
                    let device_state = DeviceState::new();
                    'key_loop: loop {
                        let keys = device_state.get_keys();
                        if !keys.is_empty() {
                            let expected_key_clone = expected_key.clone();
                            if debug {
                                debug!("Received keys {:?}", &keys);
                            }
                            let keycode_from_str = match Keycode::from_str(&expected_key_clone) {
                                Ok(key) => key,
                                Err(_) => {
                                    error!("Key {:?} isn't supported or isn't a correct key. Please replace the key in your animessage with an alphanumeric key, or a special common key (such as LControl for example) instead.", &expected_key_clone);
                                    std::process::exit(0)
                                }
                            };
                            if keys.contains(&keycode_from_str) {
                                if debug {
                                    debug!(
                                        "Key {:?} triggered this --[WAIT_FOR_INPUT]-- .",
                                        &expected_key_clone
                                    );
                                }
                                break 'key_loop;
                            }
                        }
                        sleep(Duration::from_millis(30));
                    }
                }
            }

            // OPEN_URL
            _ if line_trimmed.starts_with(OPEN_URL) => {
                let args = Args::parse(line_trimmed, 1, debug)?;
                let url = args.get(0);

                if url.contains(" ") {
                    error!("Your URL must not contain whitespaces because it can open several links. Remove all whitespaces. If your link contains whitespaces, replace them with %20 instead.");
                    std::process::exit(0)
                }

                if !url.is_empty() {
                    println!();
                    let prompt_msg = format!(
                        "Open the following URL with your default internet browser ? {}",
                        url
                    );
                    let yes = Confirm::new(&prompt_msg)
                        .with_help_message(
                            "Type \"y\" to accept or \"n\" to refuse, and then press \"Enter\".",
                        )
                        .prompt();
                    if let Ok(true) = yes {
                        if !no_exec {
                            let webbrowser_result = webbrowser::open(&url);
                            if debug {
                                match webbrowser_result {
                                    Ok(_) => debug!("Successfully opened URL {:?}.", &url),
                                    Err(err) => warn!(
                                        "URL has not been opened {:?}. Error details :\n{:#?}",
                                        &url, &err
                                    ),
                                }
                            }
                        }
                    } else {
                        if debug {
                            debug!("Refused opening URL {:?}.", &url);
                        }
                    }
                } else {
                    error!("URL is empty. Please enter an URL as the 1st argument.");
                    std::process::exit(0)
                }
            }

            // AUDIO
            _ if line_trimmed.starts_with(AUDIO) => {
                let args = Args::parse(line_trimmed, 1, debug)?;
                let audio_path: PathBuf = args.get(0).into();

                if !audio_path.as_os_str().is_empty() {
                    check_relative_path_ok(&audio_path, relative_paths_ok);

                    if debug {
                        debug!("Playing audio file {:?} ...", &audio_path);
                    }

                    match File::open(&audio_path) {
                        Ok(file) => {
                            let buf_reader = BufReader::new(file);
                            match rodio::Decoder::new(buf_reader) {
                                Ok(source) => {
                                    if !no_exec {
                                        if let Err(err) =
                                            stream_handle.play_raw(source.convert_samples())
                                        {
                                            warn!("Can't read audio using your default output device (details below) :\n{:#?}", err)
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("AUDIO ERROR : Can't read audio from file {:?} . Error : \n{}", &audio_path, e.to_string());
                                    std::process::exit(0)
                                }
                            }
                        }
                        Err(e) => {
                            error!(
                                "FILE ERROR : Can't open audio file. Error : \n{}",
                                e.to_string()
                            );
                            std::process::exit(0)
                        }
                    }
                } else {
                    error!("ARG ERROR : Please specify a path as 1st argument of --[AUDIO]-- :\n--[AUDIO]-- path/to/file.mp3");
                    std::process::exit(0)
                }
            }

            // ASCII_IMAGE
            _ if line_trimmed.starts_with(IMAGE) => {
                let args = Args::parse(line_trimmed, 1, debug)?;
                let image_path: PathBuf = args.get(0).into();

                if !image_path.as_os_str().is_empty() {
                    check_relative_path_ok(&image_path, relative_paths_ok);
                    if debug {
                        debug!(
                            "Converting image from path to ASCII : {:?} ...",
                            &image_path
                        );
                    }
                    if !no_exec {
                        let (x, y) = match cursor::position() {
                            Ok(pos) => pos,
                            Err(err) => {
                                error!("Can't obtain cursor position : {:?}", err);
                                return Ok(());
                            }
                        };

                        let conf = viuer::Config {
                            // set offset
                            x,
                            y: y as i16,
                            // set dimensions
                            width: None,
                            height: None,
                            ..Default::default()
                        };
                        if let Err(err) = viuer::print_from_file(image_path, &conf) {
                            error!("Printing image failed : {:?}", err);
                            return Ok(());
                        }

                        // crossterm::execute!(
                        //     stdout(),
                        //     crossterm::style::SetAttribute(crossterm::style::Attribute::Reset)
                        // )
                        // .unwrap();
                    }
                } else {
                    error!("ARG ERROR : Please specify a path as 1st argument of --[ASCII_IMAGE]-- :\n--[ASCII_IMAGE]-- path/to/file.jpg");
                    std::process::exit(0)
                }
            }

            // VIDEO
            // _ if line_trimmed.starts_with(VIDEO) => {
            //     let args = Args::parse(line_trimmed, 1, debug)?;
            // }

            // TITLE
            _ if line_trimmed.starts_with(TITLE) => {
                let args = Args::parse(line_trimmed, 1, debug)?;
                let title = args.get(0);

                crossterm::execute!(
                    stdout(),
                    terminal::SetTitle(&title)
                ).unwrap_or_else(|_| {
                    error!("Can't set terminal's title. Please use a terminal that supports title changes, such as Alacritty 0.5 or above.");
                    std::process::exit(0)
                });

                if debug {
                    debug!("Terminal title set to {:?}", title);
                }
            }

            // CLEAR
            _ if line_trimmed == CLEAR => {
                let _ = Args::parse(line_trimmed, 0, debug);
                if debug {
                    debug!("Clearing terminal. This function has no effect in debug mode.");
                } else {
                    clear_terminal()
                }
            }

            // RESIZE
            _ if line_trimmed.starts_with(RESIZE) => {
                let args = Args::parse(line_trimmed, 2, debug)?;

                let columns = args.get(0).parse::<u16>().unwrap_or_else(|_| {
                    error!("Can't convert arg to an integer between 0 and 65535 included.");
                    std::process::exit(0)
                });
                let rows = args.get(1).parse::<u16>().unwrap_or_else(|_| {
                    error!("Can't convert arg to an integer between 0 and 65535 included.");
                    std::process::exit(0)
                });

                if debug {
                    let current_terminal_size = terminal::size().unwrap();
                    let new_terminal_size = (columns, rows);
                    debug!("Resizing the terminal from {:?} to {:?} (columns, rows). This function has no effect in debug mode.", current_terminal_size, new_terminal_size);
                }

                if !no_exec {
                    crossterm::execute!(
                        stdout(),
                        terminal::SetSize(columns, rows)
                    ).unwrap_or_else(|_| {
                        error!("Can't resize this terminal. Use another terminal such as Alacritty.");
                        std::process::exit(0)
                    });
                }
            }

            // MOVE_CURSOR
            _ if line_trimmed.starts_with(MOVE_CURSOR) => {
                let args = Args::parse(line_trimmed, 2, debug)?;

                let columns = args.get(0).parse::<u16>().unwrap_or_else(|_| {
                    error!("Can't convert arg to an integer between 0 and 65535 included.");
                    std::process::exit(0)
                });
                let rows = args.get(1).parse::<u16>().unwrap_or_else(|_| {
                    error!("Can't convert arg to an integer between 0 and 65535 included.");
                    std::process::exit(0)
                });

                if debug {
                    debug!(
                        "Moving the cursor to position {} * {} (columns * rows) ...",
                        &columns, &rows
                    );
                }

                if !debug {
                    move_cursor(columns, rows);
                }
            }

            // HIDE_CURSOR
            _ if line_trimmed == HIDE_CURSOR => {
                if !no_exec {
                    crossterm::execute!(stdout(), cursor::Hide).unwrap_or_else(|_| {
                        error!(
                            "Can't resize this terminal. Use another terminal such as Alacritty."
                        );
                        std::process::exit(0)
                    });
                }

                if debug {
                    debug!(
                        "The cursor has been hidden. This function has no effect in debug mode."
                    );
                }
            }

            // SHOW_CURSOR
            _ if line_trimmed == SHOW_CURSOR => {
                if !no_exec {
                    crossterm::execute!(stdout(), cursor::Show).unwrap_or_else(|_| {
                        error!(
                            "Can't resize this terminal. Use another terminal such as Alacritty."
                        );
                        std::process::exit(0)
                    });
                }

                if debug {
                    debug!("The cursor is now shown. This function has no effect in debug mode.");
                }
            }

            // INCLUDE
            _ if line_trimmed.starts_with(INCLUDE) => {
                let args = Args::parse(line_trimmed, 1, debug)?;
                let s_path: PathBuf = args.get(0).into();

                if !s_path.as_os_str().is_empty() {
                    check_relative_path_ok(&s_path, relative_paths_ok);

                    match read_to_string(&s_path) {
                        Ok(s) => {
                            if debug {
                                debug!("Including file {:?} ...", &s_path)
                            }

                            let mut base_index = line_index;
                            lines.remove(base_index); // Remove the --[INCLUDE]-- line.
                            for l in s.lines() {
                                // Replace the old --[INCLUDE]-- line with text from file.
                                lines.insert(base_index, l.to_string());
                                base_index += 1;
                            }
                            line_index -= 1; // Go back to the line where the --[INCLUDE]-- was.
                        }
                        Err(_) => {
                            error!("FILE ERROR : Can't read file as text");
                            std::process::exit(0)
                        }
                    }
                } else {
                    error!("ARG ERROR : Please specify a path as 1st argument of --[INCLUDE]-- :\n--[INCLUDE]-- path/to/file.txt");
                    std::process::exit(0)
                }
            }

            // ESCAPE
            _ if line_trimmed.starts_with(ESCAPE) => {
                if debug {
                    debug!("Escaping this line. Functions won't be executed.")
                }

                let skipped_string = &line_trimmed[ESCAPE.chars().count() + 1..]; // Escaped text is not considered as an arg. Do NOT Args::parse.
                let escaped_string = format!("{}{}", skipped_string, "\n");
                current_step.push_str(&escaped_string);
            }

            // EMPTY
            _ if line_trimmed == EMPTY => current_step.push_str("\n"),

            // NOTE, MARKER, REAL, FORMATTING EMPTY LINE
            _ if line_trimmed.starts_with(NOTE)
                || line_trimmed.starts_with(MARKER)
                || line_trimmed.is_empty() =>
            {
                ()
            }

            _ if line_trimmed == EXIT => return Ok(()),

            // Anything else
            _ => {
                let line = format!("{}\n", line);
                current_step.push_str(&line);
            }
        }

        line_index += 1;
    }

    Ok(())
}
