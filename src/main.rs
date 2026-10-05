mod app;
mod appearance;
mod files;
mod macos;
mod menu;
mod protocol;
mod render;

use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).filter(|arg| !arg.starts_with("-psn_")).collect();
    let result = match args.first().map(String::as_str) {
        Some("--version") => {
            println!("mdview {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Some("--render") => print_page(&args[1..]),
        _ => app::run(args.into_iter().map(PathBuf::from).collect()),
    };
    let Err(error) = result else {
        return ExitCode::SUCCESS;
    };
    eprintln!("mdview: {error:#}");
    ExitCode::FAILURE
}

fn print_page(paths: &[String]) -> anyhow::Result<()> {
    let Some(path) = paths.first() else {
        anyhow::bail!("usage: mdview --render <file.md>");
    };
    let markdown = std::fs::read_to_string(path)?;
    println!("{}", render::renderer().page(&markdown, path));
    Ok(())
}
