//! `dfr` — tools of the dsh-locale-fr language pack (replaces the three Python scripts).

use dfr_outils::{build_client, harvest_cjk, harvest_dictionaries};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "usage:
  dfr build-client [<pack-root>]                       generate lib/client.js (default: current directory)
  dfr harvest-dictionaries <dir-of-@deepseek-ai> <out.json>
  dfr harvest-cjk <client.js> [<client.js> ...] > out.json";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match executer(&args) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("dfr: {e:#}");
            ExitCode::from(1)
        }
    }
}

fn executer(args: &[String]) -> anyhow::Result<ExitCode> {
    match args.first().map(String::as_str) {
        Some("build-client") if args.len() <= 2 => {
            let racine = args
                .get(1)
                .map_or_else(|| PathBuf::from("."), PathBuf::from);
            println!("{}", build_client::executer(&racine)?);
        }
        Some("harvest-dictionaries") if args.len() == 3 => {
            let (json, resume) = harvest_dictionaries::executer(Path::new(&args[1]))?;
            std::fs::write(&args[2], json)?;
            print!("{resume}");
        }
        Some("harvest-cjk") if args.len() >= 2 => {
            let fichiers: Vec<&Path> = args[1..].iter().map(Path::new).collect();
            let (json, n) = harvest_cjk::executer(&fichiers)?;
            println!("{json}");
            eprintln!("{n} runs");
        }
        _ => {
            eprintln!("{USAGE}");
            return Ok(ExitCode::from(2));
        }
    }
    Ok(ExitCode::SUCCESS)
}
