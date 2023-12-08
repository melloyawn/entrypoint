//! example w/ user impls/overrides

use entrypoint::prelude::*;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, num_args = 1..)]
    pub dotenv_files: Vec<std::path::PathBuf>,

    #[arg(short, long)]
    verbose: bool,
}

impl DotEnvParser for Args {
    /// pull in user provided dotenv files
    fn dotenv_files(&self) -> Option<Vec<std::path::PathBuf>> {
        Some(self.dotenv_files.clone())
    }
}

impl Logger for Args {
    /// select log level based on user input
    fn log_level(&self) -> Level {
        if self.verbose {
            Level::TRACE
        } else {
            Level::DEBUG
        }
    }
}

#[entrypoint::entrypoint]
#[test]
fn entrypoint(args: Args) -> entrypoint::Result<()> {
    entrypoint::tracing::info!("in entrypoint({:?})", args);

    Ok(())
}
