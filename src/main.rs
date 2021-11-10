use std::{cell::RefCell, io::Read, mem::MaybeUninit, path};

use anyhow::bail;
use envmnt::{exists, get_list};
use inquire::Confirm;
use log::info;
use term_table::{row::Row, Table};

mod parser;
use parser::*;

mod term;
use term::*;

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

#[cfg(windows)]
const TUTORIAL: &str = include_str!(r#"..\animessages\tutorial\tutorial_new.txt"#);
#[cfg(not(windows))]
const TUTORIAL: &str = include_str!(r#"../animessages/tutorial/tutorial_new.txt"#);

fn check_relative_path_ok(path: &Path, relative_paths_ok: bool) {
    if path.is_relative() && !relative_paths_ok {
        error!("PATH ERROR : Can't process a relative path because of an error. Check the first \"WARN\" messages for more details.");
        std::process::exit(0)
    }
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
        let app_dir_path_string: String = app_dir_path.as_os_str().to_str().unwrap().to_string();

        let mut is_in_path = false;
        for value in path_list.clone() {
            if value == app_dir_path_string {
                is_in_path = true;
                if debug {
                    debug!("Animessage is in the Path (env vars).")
                }
                break;
            }
        }
        if !is_in_path {
            path_list.push(app_dir_path_string.clone());
            envmnt::set_list("Path", &path_list);
            if debug {
                debug!("Animessage has been added to the Path (env vars).")
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
                error!("Couldn't open default 'run.anim' file in this folder. Falling back to opening the tutorial / a given file...\nError details :\n{:?}", err)
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
