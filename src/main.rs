#[macro_use]
extern crate lazy_static;

mod ansi_colors;
mod bad_names;
mod error;
mod list;
mod lpr;
mod print;
mod ssh;
mod upload;

use crate::error::Error;
use crate::list::PrinterInput;
use crate::print::print;
use regex::Regex;
use std::collections::HashMap;
use std::ops::RangeInclusive;
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[derive(Debug)]
pub enum ColorMode {
    Color,
    BlackWhite,
}

pub struct Printer {
    pub name: String,
    pub description: String,
    pub location: String,
    pub cost: HashMap<ColorMode, f32>,
    pub size: String,
    pub support: String,
}

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct PrintOpt {
    /// Number of copies to print
    #[structopt(short, long, default_value = "1")]
    copies: u32,

    /// Force grayscale
    #[structopt(short, long, conflicts_with = "color")]
    grayscale: bool,

    /// Force color
    #[structopt(long, conflicts_with = "grayscale")]
    color: bool,

    /// Set page size (e.g. "a4").
    #[structopt(short, long)]
    media: Option<String>,

    /// Print on one side of the paper
    #[structopt(short, long)]
    one_sided: bool,

    /// A range of pages to print, e.g. "2-14" or "3". Applies to all files.
    #[structopt(short, long, parse(try_from_str = parse_range))]
    range: Vec<RangeInclusive<u32>>,
}

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct Opt {
    #[structopt(short, long)]
    debug: bool,

    #[structopt(flatten)]
    print_opts: PrintOpt,

    /// The specific printer to print from
    #[structopt(short, long)]
    printer: Option<String>,

    /// Files to print
    #[structopt(name = "FILE", parse(from_os_str))]
    files: Vec<PathBuf>,
}

fn parse_range(part: &str) -> Result<RangeInclusive<u32>, Error> {
    lazy_static! {
        static ref DIGIT_RE: Regex = Regex::new(r"^\d+$").unwrap();
        static ref RANGE_RE: Regex = Regex::new(r"^(?P<d1>\d+)\s*-\s*(?P<d2>\d+)$").unwrap();
    }

    let part = part.trim();

    if DIGIT_RE.is_match(part) {
        let d: u32 = part
            .parse()
            .map_err(|_| Error::user("failed to parse int"))?;
        Ok(RangeInclusive::new(d, d))
    } else if let Some((d1, d2)) = RANGE_RE.captures(part).and_then(|r| {
        r.name("d1")
            .into_iter()
            .zip(r.name("d2").into_iter())
            .next()
    }) {
        let d1: u32 = d1
            .as_str()
            .parse()
            .map_err(|_| Error::user("failed to parse int"))?;
        let d2: u32 = d2
            .as_str()
            .parse()
            .map_err(|_| Error::user("failed to parse int"))?;
        Ok(RangeInclusive::new(d1, d2))
    } else {
        Err(Error::user(format!(
            "could not parse \"{}\" as range.",
            part
        )))
    }
}

async fn load_file(path: &Path) -> Result<Vec<u8>, Error> {
    let mut data = vec![];
    let mut file = File::open(path).await?;
    file.read_to_end(&mut data).await?;
    Ok(data)
}

async fn ask_for_printer() -> Result<String, Error> {
    let mut rl = rustyline::Editor::with_config(
        rustyline::Config::builder()
            .completion_type(rustyline::CompletionType::Fuzzy)
            .build(),
    );
    rl.set_helper(Some(PrinterInput));

    let readline = rl.readline("choose your printer: ");
    match readline {
        Ok(line) => Ok(line),
        Err(_) => Err(Error::user("no printer specified")),
    }
}

async fn run(opt: &Opt, remote: &str) -> Result<(), Error> {
    if opt.files.is_empty() {
        return Err(Error::user("no files specified"));
    }

    let printer = match &opt.printer {
        Some(p) => p.to_string(),
        None => ask_for_printer().await?,
    };

    if printer.is_empty() {
        return Err(Error::user("no printer specified"));
    }

    let mut session = ssh::connect(remote)?;

    let cmd_string = lpr::build_cmd(&printer, &opt.print_opts);

    for file_path in opt.files.iter() {
        let data = load_file(file_path).await?;

        eprintln!("uploading {:?}", file_path);
        let remote_file_name = upload::upload_print_file(&mut session, &data)?;
        let job_name = bad_names::generate();
        let cmd_string = format!("{} -J '{}' {}", cmd_string, job_name, remote_file_name);

        eprintln!("printing {:?}", file_path);
        if opt.debug {
            eprintln!("[DEBUG] skipping print");
            eprintln!("[DEBUG] cmd: {}", cmd_string);
        } else {
            print(&mut session, &cmd_string)?;
        }

        eprintln!("file printed successfully.");
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();

    let mut remotes = ssh::DOMAINS.iter().peekable();
    while let Some(remote) = remotes.next() {
        eprintln!("trying {}", remote);
        if let Err(e) = run(&opt, remote).await {
            eprintln!("{}", e);

            // if there was an error with the SSH connection, ask to try again with another remote
            // TODO: detect password errors
            match e {
                Error::SshError(_) => {
                    if let Some(next_remote) = remotes.peek() {
                        let question = format!(r#"try again with "{}"? (y/n) "#, next_remote);
                        let mut rl = rustyline::Editor::<()>::new();
                        match rl.readline(&question) {
                            Ok(answer) if answer.trim() == "y" => continue,
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        break;
    }
}
