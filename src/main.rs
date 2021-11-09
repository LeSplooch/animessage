use std::{cell::RefCell, io::Read, mem::MaybeUninit, path};

use anyhow::bail;
use envmnt::{exists, get_list};
use inquire::Confirm;
use log::info;
use term_table::{row::Row, Table};

// #![no_mangle]

use {
    /* parking_lot::{
        Mutex
    }, */
    /* lazy_static::lazy_static, */
    anyhow::Result as AnyResult,
    args::{duration_from_arg, Args},
    asciify::AsciiBuilder,
    crossterm::{
        self, /* Command, */
        /* execute, ExecutableCommand,  */ cursor,
        terminal::{self, ClearType},
    },
    device_query::{DeviceQuery, DeviceState, Keycode},
    image::{self /* GenericImageView */},
    log::{debug /*,  info */, error, warn},
    read_input::prelude::*,
    rodio::{self, Source},
    simple_logger::SimpleLogger,
    std::{
        /* borrow::Cow, */
        /* env, */
        collections::BTreeMap,
        fs::{read_to_string, /* self, */ File},
        io::{stdout, BufReader, Write},
        path::{Path, PathBuf},
        str::FromStr,
        thread::sleep,
        time::Duration,
    },
    structopt::StructOpt,
    variable::Variable,
};

mod args;
mod variable;

// #[derive(ThisError, Debug)]
// pub enum AnimessageError {
//     #[error("Can't set `{0}` as current working directory.")]
//     RelativePathError(String),
// }

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Animessage",
    about = "Create animated messages in the terminal."
)]
struct Opts {
    /// Path to the file you want to open.
    #[structopt(short, long)]
    file: Option<PathBuf>,

    /// Enables debug mode to show executed lines as they get interpreted.
    #[structopt(short, long)]
    debug: bool,

    /// Reads the tutorial instead of a file.
    #[structopt(short, long)]
    tutorial: bool,

    /// Doesn't execute most of the functions for faster debugging. Enables debug mode if not enabled.
    #[structopt(short, long)]
    no_exec: bool,

    /// Searches for a marker in the animessage and starts at its line if a corresponding marker is found. Will have no effect if no such marker has been found.
    #[structopt(short, long)]
    marker: Option<String>,

    /// Prints a summary of all the markers in the animessage showing their name and line number. Will have no effect if no marker has been found.
    #[structopt(short, long)]
    summary: bool,
}

// Functions
const PRINT: &str = "--[PRINT]--"; // Prints your previous lines to the console. 1st arg : Delay between each character.
const PRINT_LINE: &str = "--[PRINT_LINE]--"; // Prints your previous lines to the console with a newline at the end. 1st arg : Delay between each character.
const VAR: &str = "--[VAR]--"; // BETA. DOESN'T WORK YET.
const GOTO: &str = "--[GOTO]--"; // Go to a line. 1st arg : line to go to.
const MARKER: &str = "--[MARKER]--"; // Sets a marker to easily go to a specified line of the animessage via the command parameter '-m'. 1st arg : Marker name. 2nd arg : line to start from.
const WAIT: &str = "--[WAIT]--"; // Wait for some duration before continuing. 1st arg : Duration in ms.
const REPLACE: &str = "--[REPLACE]--"; // Replace text at a given line. 1st arg : line. 2nd arg : Replace from. 3rd arg : Replace to.
const DEL_LINE: &str = "--[DEL_LINE]--"; // Deletes a line, therefore offsetting the following lines by -1. 1st arg : line number.
const WAIT_FOR_INPUT: &str = "--[WAIT_FOR_INPUT]--"; // Wait for a keyboard key to be input before continuing. 1st arg : Key.
const OPEN_URL: &str = "--[OPEN_URL]--"; // Opens a given URL if the user allows it. 1st arg : URL.
const AUDIO: &str = "--[AUDIO]--"; // Plays a sound in the background. 1st arg : Sound path.
const TTS: &str = "--[TTS]--"; // NOT IMPLEMENTED YET. Reads some text using the default text-to-speech voice from your operating system.
const DRAW: &str = "--[DRAW]--"; // NOT IMPLEMENTED YET. Draws forms and presets and puts them in the print buffer. Will use args.
const IMAGE: &str = "--[IMAGE]--"; // Transforms an image into ASCII and then prints it to the console. 1st arg : Image path.
const VIDEO: &str = "--[VIDEO]--"; // NOT IMPLEMENTED YET. Transforms a video into ASCII and then prints it to the console. 1st arg : Video path.
const TITLE: &str = "--[TITLE]--"; // Sets the title of the terminal. 1st arg : title.
const CLEAR: &str = "--[CLEAR]--"; // Clears the terminal, leaving the terminal empty. Often used before print to seperate steps in your animessage.
const RESIZE: &str = "--[RESIZE]--"; // Resizes the terminal. 1st arg : columns. 2nd arg : rows.
const MOVE_CURSOR: &str = "--[MOVE_CURSOR]--"; // Moves the cursor to the specified location in columns * rows. 1st arg : columns. 2nd arg : rows.
const HIDE_CURSOR: &str = "--[HIDE_CURSOR]--"; // Hides the cursor. DUH !
const SHOW_CURSOR: &str = "--[SHOW_CURSOR]--"; // Shows the cursor. DUH !
const EMPTY: &str = "--[EMPTY]--"; // Inserts an empty line. By default, empty lines in your code have no effect to allow better formatting of your code.
const INCLUDE: &str = "--[INCLUDE]--"; // Includes another text file at the location. 1st arg : path to animessage to include.
const ESCAPE: &str = "--[ESCAPE]--"; // Disables functions in this line, entering the line in the print buffer as it is without this function in it.
const NOTE: &str = "--[NOTE]--"; // Used to write a comment. This has no effect.
const EXIT: &str = "--[EXIT]--"; // Close Animessage prematurely.

#[cfg(windows)]
const TUTORIAL: &str = include_str!(r#"..\animessages\tutorial\tutorial_new.txt"#);
#[cfg(not(windows))]
const TUTORIAL: &str = include_str!(r#"../animessages/tutorial/tutorial_new.txt"#);

// fn check_relative_paths(path: &Path, relative_paths_ok: bool) -> anyhow::Result<()> {
//     if path.is_relative() && !relative_paths_ok {
//         return Err("Can't")
//     } else {
//         Ok(())
//     }
// }

fn clear_terminal() {
    crossterm::execute!(stdout(), terminal::Clear(ClearType::All)).unwrap_or_else(|_| {
        error!("Can't clear lines in this terminal. Animessage can't work properly.")
    });
}

fn save_cursor_position() {
    crossterm::execute!(stdout(), cursor::SavePosition).unwrap_or_else(|_| {
        error!("Can't save the position of the cursor.");
        std::process::exit(0)
    });
}

fn restore_cursor_position() {
    crossterm::execute!(stdout(), cursor::RestorePosition).unwrap_or_else(|_| {
        error!("Can't restore the position of the cursor..");
        std::process::exit(0)
    });
}

fn move_cursor(columns: u16, rows: u16) {
    crossterm::execute!(stdout(), cursor::MoveTo(columns, rows)).unwrap_or_else(|_| {
        error!("Can't move the cursor in this terminal. Use another terminal such as Alacritty.");
        std::process::exit(0)
    });
    flush_stdout();
}

fn flush_stdout() {
    if let Err(err) = stdout().flush() {
        warn!(
            "PRINT ERROR : Can't flush stdout. Error details below : \n{:#?}",
            err
        )
    }
}

fn check_relative_path_ok(path: &Path, relative_paths_ok: bool) {
    if path.is_relative() && !relative_paths_ok {
        error!("PATH ERROR : Can't process a relative path because of an error. Check the first \"WARN\" messages for more details.");
        std::process::exit(0)
    }
}

#[allow(unreachable_code)]
fn display_animessage(
    animessage_str: &str,
    relative_paths_ok: bool,
    debug: bool,
    no_exec: bool,
    start_index: usize,
) -> AnyResult<()> {
    let mut current_step = String::with_capacity(1024 * 5);
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
    let mut line_index: usize = start_index;

    if !debug {
        clear_terminal();
        move_cursor(0, 0);
        save_cursor_position();
    }

    'main_loop: while line_index + 1 <= lines.len() {
        let line = lines[line_index].clone();
        let line_trimmed = line.trim();
        let line_number = line_index + 1;

        if debug {
            println!("{} | {}", line_number, line);
        }

        match line {
            // PRINT
            _ if line_trimmed.starts_with(PRINT) => {
                let args = Args::parse(line_trimmed, 1, debug)?;
                let (print_interval_f32, print_interval) = duration_from_arg(args.get(0)); // We have verified that the number of args is correct so we can index as we please.

                if !current_step.is_empty() {
                    let current_step_trimmed = current_step.trim_end();

                    if print_interval_f32 == 0.0 {
                        if debug {
                            debug!("Printing this step all at once.");
                        }
                        if !no_exec {
                            print!("{}", current_step_trimmed);
                            flush_stdout();
                        }
                    } else {
                        if debug {
                            debug!("Printing this step character by character with an interval of {:?}.", print_interval);
                        }
                        if !no_exec {
                            for line_string in current_step_trimmed.lines() {
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
                    let current_step_trimmed = current_step.trim_end();

                    if print_interval_f32 == 0.0 {
                        if debug {
                            debug!("Printing this step all at once.");
                        }
                        if !no_exec {
                            println!("{}", current_step_trimmed);
                            flush_stdout();
                        }
                    } else {
                        if debug {
                            debug!("Printing this step character by character with an interval of {:?}.", print_interval);
                        }
                        if !no_exec {
                            for line_string in current_step_trimmed.lines() {
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
                        r#"Open the following URL with your default browser ? {:?} (Press "y" and then "Enter" on your keyboard to accept, or leave blank to refuse.) "#,
                        &url
                    );
                    let yes = Confirm::new(&prompt_msg)
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
                let audio_path: PathBuf = args
                    .get(0)
                    .into();

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

                    let terminal_size = terminal::size().unwrap();
                    let image = image::open(&image_path).unwrap_or_else(|img_error| {
                        error!("Image Error : {:#?}", img_error);
                        std::process::exit(0)
                    });
                    // let image_dimensions = image.dimensions();

                    if debug {
                        debug!(
                            "Converting image from path to ASCII : {:?} ...",
                            &image_path
                        );
                        debug!(
                            "Resizing ASCII to terminal size (columns * rows) : {} * {} ...",
                            terminal_size.0, terminal_size.1
                        );
                    }
                    if !no_exec {
                        AsciiBuilder::new_from_image(image)
                            .set_deep(true)
                            .set_invert(false)
                            .set_resize((terminal_size.0 as u32 - 1, terminal_size.1 as u32))
                            .to_std_out(true);

                        crossterm::execute!(
                            stdout(),
                            crossterm::style::SetAttribute(crossterm::style::Attribute::Reset)
                        )
                        .unwrap();
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
                let line = format!("{}{}", line, "\n");
                current_step.push_str(&line);
            }
        }

        line_index += 1;
    }

    Ok(())
}

fn print_title() {
    println!(
        "

    ╔═══════════════════════════════════════════════════════════════════════╗
    ║   ----|| Animessage, an application by github.com/LeSplooch ||----    ║
    ╚═══════════════════════════════════════════════════════════════════════╝
    "
    );
    flush_stdout();

    crossterm::execute!(stdout(), cursor::Show).unwrap_or_else(|_| {
        error!("Can't show the cursor on this terminal with its current settings.");
        std::process::exit(0)
    });
}

enum MarkerMode {
    Find,
    Summary,
}

impl FromStr for MarkerMode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "find" => Ok(MarkerMode::Find),
            "summary" => Ok(MarkerMode::Summary),
            _ => anyhow::bail!("No marker mode with this name."),
        }
    }
}

fn process_markers(
    animessage_str: &str,
    marker: &str,
    debug: bool,
    mode: MarkerMode,
) -> AnyResult<usize> {
    if debug {
        match mode {
            MarkerMode::Find => debug!("Searching for markers..."),
            MarkerMode::Summary => (),
        }
    }

    let anim_lines_iter = animessage_str.lines().enumerate();
    match mode {
        MarkerMode::Find => {
            info!("Searching for markers...");
            for (index, l) in anim_lines_iter {
                if l.starts_with(MARKER) {
                    let args = Args::parse(l, 1, debug)?;
                    // TODO : Add one more arg to set an optional line to end with. -1 would be the last line.
                    if args.get(0) == marker {
                        if debug {
                            debug!(
                                "Found a corresponding marker to {:?} at line {}",
                                marker,
                                index + 1
                            )
                        }
                        return Ok(index);
                    }
                }
            }
        }
        MarkerMode::Summary => {
            println!("Markers summary :");
            let mut table = Table::new();
            for (index, l) in anim_lines_iter {
                if l.starts_with(MARKER) {
                    let args = Args::parse(l, 1, debug)?;
                    let marker_name = args.get(0);
                    let row = Row::new(vec![marker_name, &format!("{}", index + 1)]);
                    table.add_row(row);
                }
            }
            println!("{}", table.render());
            // table.unwrap()
            // .add_row(
            //     Row::new(vec!["Name", "Line number"])
            // );
        }
    }

    if let MarkerMode::Find = mode {
        error!("No marker corresponding to {:?}. Exiting...", marker);
        print_title();
        std::process::exit(0);
    }

    Ok(0)
}

fn main() -> AnyResult<()> {
    SimpleLogger::new().init().unwrap();

    ctrlc::set_handler(move || {
        std::process::exit({
            print_title();
            warn!("Animessage terminated by user. (Ctrl + C)");
            0
        });
    })?;

    // Get cmd args
    let options = Opts::from_args();
    let file = options.file;
    let tutorial: bool = options.tutorial;
    let no_exec = options.no_exec;
    let debug = if options.no_exec {
        debug!("Debug mode has been enabled by default because no_exec is enabled.");
        true
    } else {
        options.debug
    };
    let marker: Option<String> = options.marker;
    let markers_summary: bool = options.summary;

    // Put Animessage in the Path
    let path_list = envmnt::get_list("Path");
    if let Some(mut path_list) = path_list {
        let app_dir_path = std::env::current_dir()?;
        let app_dir_path_string: &String = &app_dir_path
            .as_os_str()
            .to_str()
            .unwrap()
            .split("/")
            .collect();

        for value in path_list.clone() {
            if value == "Path" {
                if !path_list.contains(app_dir_path_string) {
                    path_list.push(app_dir_path_string.clone());
                    envmnt::set_list("Path", &path_list);
                } else {
                    debug!("Animessage is in the Path.")
                }
                break;
            }
        }
    }

    // Open either default file or tutorial or specified file, in this order.
    let default_file = PathBuf::from("run.anim");
    if default_file.exists() && file.is_none() {
        match File::open(default_file) {
            Ok(mut file) => {
                let mut buf = String::new();
                let _bytes_read = file.read_to_string(&mut buf);
                display_animessage(&buf, true, false, false, 0)?;
                print_title();
                return Ok(());
            }
            Err(err) => {
                error!("Couldn't open default 'run.anim' file in this folder. Falling back to opening the tutorial / a given file...")
            }
        }
    }
    if tutorial || file.is_none() {
        if markers_summary {
            let marker_mode = MarkerMode::Summary;
            process_markers(TUTORIAL, "", debug, marker_mode)?;
            return Ok(());
        } else {
            let marker_mode = MarkerMode::Find;
            let start_index = if marker.is_some() {
                process_markers(TUTORIAL, &marker.unwrap(), debug, marker_mode)?
            } else {
                0
            };

            display_animessage(TUTORIAL, false, debug, no_exec, start_index)?;
        }
    } else {
        let file: PathBuf = file.unwrap();
        let animessage_absolute_path = file
            .as_path()
            .canonicalize()?
            .to_str()
            .unwrap()
            .replace("\\\\?\\", "");

        let animessage_absolute_path: PathBuf = animessage_absolute_path.into();

        let animessage_dir_absolute_path = animessage_absolute_path.parent().unwrap();

        let animessage_string = read_to_string(&file).unwrap();

        if markers_summary {
            let marker_mode = MarkerMode::Summary;
            process_markers(&animessage_string, "", debug, marker_mode)?;
            return Ok(());
        }

        let start_index = if let Some(marker) = marker {
            process_markers(&animessage_string, &marker, debug, MarkerMode::Find)?
        } else {
            0
        };

        let mut relative_paths_ok = false;

        if debug {
            debug!(
                "Setting current working directory to {path:?} ...",
                path = &animessage_dir_absolute_path
            )
        }

        match std::env::set_current_dir(&animessage_dir_absolute_path) {
            Ok(_) => {
                relative_paths_ok = true;
                if debug {
                    debug!("Current directory set to {path:?}. Relative paths will work in your functions arguments.", path = &animessage_dir_absolute_path)
                }
            }
            Err(err) => {
                if debug {
                    warn!("WARNING : Can't set current working directory to {path:?}. Relative paths won't work in your functions arguments. \nError details : {err:#?}",
                    path = &animessage_dir_absolute_path,
                    err = err
                )
                }
            }
        }

        display_animessage(
            &animessage_string,
            relative_paths_ok,
            debug,
            no_exec,
            start_index,
        )?;
    }

    if debug {
        debug!("--- END --- ");
    }
    print_title();

    // if let Ok((columns, rows)) = terminal::size() {
    //     move_cursor(columns, rows);
    // }

    Ok(())
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer scope.
    use super::*;

    #[test]
    fn syntax_test() {
        let res = display_animessage(TUTORIAL, true, true, true, 0);
        crossterm::execute!(stdout(), cursor::Show).unwrap_or_else(|_| {
            error!("Can't show the cursor. Use another terminal such as Alacritty.");
        });
        assert!(res.is_ok());
    }
}
