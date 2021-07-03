mod interpreter;

use clap::{App, Arg};
use interpreter::Interpreter;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

struct Cli
{
    path: PathBuf,
}

impl Cli
{
    fn new() -> Self
    {
        let app = App::new(NAME)
            .version(VERSION)
            .arg(Arg::with_name("filepath").takes_value(true).required(true));
        let matches = app.get_matches();
        let path = Path::new(matches.value_of("filepath").unwrap()).to_owned();
        Self { path }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>>
{
    let cli = Cli::new();
    let mut file = fs::File::open(&cli.path)?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let mut intptr = Interpreter::new(&contents);
    
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    
    intptr.run(&stdin, &mut stdout)?;

    Ok(())
}
