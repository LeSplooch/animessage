use super::*;
use crossterm::terminal;

pub(crate) fn clear_terminal() {
    crossterm::execute!(stdout(), terminal::Clear(ClearType::All)).unwrap_or_else(|_| {
        error!("Can't clear lines in this terminal. Animessage can't work properly.");
        std::process::exit(0)
    });
}

pub(crate) fn save_cursor_position() {
    crossterm::execute!(stdout(), cursor::SavePosition).unwrap_or_else(|_| {
        error!("Can't save the position of the cursor.");
        std::process::exit(0)
    });
}

pub(crate) fn restore_cursor_position() {
    crossterm::execute!(stdout(), cursor::RestorePosition).unwrap_or_else(|_| {
        error!("Can't restore the position of the cursor..");
        std::process::exit(0)
    });
}

pub(crate) fn move_cursor(columns: u16, rows: u16) {
    crossterm::execute!(stdout(), cursor::MoveTo(columns, rows)).unwrap_or_else(|_| {
        error!("Can't move the cursor in this terminal. Use another terminal such as Alacritty.");
        std::process::exit(0)
    });
    flush_stdout();
}

pub(crate) fn flush_stdout() {
    if let Err(err) = stdout().flush() {
        warn!(
            "PRINT ERROR : Can't flush stdout. Error details below : \n{:#?}",
            err
        )
    }
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Animessage",
    about = "Create animated messages for the terminal."
)]
pub(crate) struct Opts {
    /// Path to the file you want to open.
    #[structopt(short, long)]
    pub(crate) file: Option<PathBuf>,

    /// Enables debug mode to show executed lines as they get interpreted.
    #[structopt(short, long)]
    pub(crate) debug: bool,

    /// Reads the tutorial instead of a file.
    #[structopt(short, long)]
    pub(crate) tutorial: bool,

    /// Doesn't execute most of the functions for faster debugging. Enables debug mode if not enabled.
    #[structopt(short, long)]
    pub(crate) no_exec: bool,

    /// Searches for a marker in the animessage and starts at its line if a corresponding marker is found. Will have no effect if no such marker has been found.
    #[structopt(short, long)]
    pub(crate) marker: Option<String>,

    /// Prints a summary of all the markers in the animessage showing their name and line number. Will have no effect if no marker has been found.
    #[structopt(short, long)]
    pub(crate) summary: bool,
}
