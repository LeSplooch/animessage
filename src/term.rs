use super::*;

pub(crate) fn clear_terminal(stdout: &Term) -> anyhow::Result<()> {
    match stdout.clear_screen() {
        Ok(_exec_ref) => Ok(()),
        Err(_err) => {
            anyhow::bail!("Can't clear lines in this terminal. Animessage can't work properly.")
        }
    }
}

// pub(crate) fn save_cursor_position(stdout: &Term) -> anyhow::Result<()> {
//     match stdout().execute(cursor::SavePosition) {
//         Ok(_exec_ref) => Ok(()),
//         Err(_err) => anyhow::bail!("Can't save the position of the cursor."),
//     }
// }

// pub(crate) fn restore_cursor_position(stdout: &Term) -> anyhow::Result<()> {
//     match stdout. {
//         Ok(_exec_ref) => Ok(()),
//         Err(_err) => anyhow::bail!("Can't restore the position of the cursor."),
//     }
// }

pub(crate) fn move_cursor(stdout: &Term, columns: usize, rows: usize) -> anyhow::Result<()> {
    match stdout.move_cursor_to(columns, rows) {
        Ok(_exec_ref) => {
            flush_stdout();
            Ok(())
        }
        Err(_err) => anyhow::bail!(
            "Can't move the cursor in this terminal. Use another terminal such as Alacritty."
        ),
    }
}

pub(crate) fn move_to_previous_line(stdout: &Term, lines_n: usize) -> anyhow::Result<()> {
    match stdout.move_cursor_up(lines_n) {
        Ok(_exec_ref) => Ok(()),
        Err(_err) => anyhow::bail!(
            "Can't move the cursor in this terminal. Use another terminal such as Alacritty."
        ),
    }
}

pub(crate) fn flush_stdout() {
    if let Err(err) = io::stdout().flush() {
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
