use std::{
    fmt::Write as FmtWrite,
    fs, io,
    io::{Read, Write},
    process,
};

use clap::{crate_version, Parser};
use glob::glob;
use sqlformat::{format, FormatOptions, Indent, QueryParams};
use thiserror::Error;

fn main() {
    let options = Options::parse();
    let Options {
        indent_spaces,
        uppercase,
        lines_between_queries,
        ..
    } = options;
    let format_options = FormatOptions {
        indent: Indent::Spaces(indent_spaces),
        uppercase: Some(uppercase),
        lines_between_queries,
        ..Default::default()
    };

    let result = || -> Result<(), Error> {
        match options.file_paths.is_empty() {
            // If no file paths are provided, read from stdin
            true => {
                let mut input = String::new();
                io::stdin().read_to_string(&mut input)?;
                let formatted = format(&input, &QueryParams::default(), &format_options);
                io::stdout().write_all(formatted.as_bytes())?;
            }
            false => {
                for file_path in options.file_paths {
                    let entries = glob(&file_path)?;
                    for entry in entries {
                        let path = entry?;

                        let mut input = String::new();
                        fs::File::open(&path)?.read_to_string(&mut input)?;

                        let mut formatted =
                            format(&input, &QueryParams::default(), &format_options);

                        if options.omit_newline && !formatted.ends_with('\n') {
                            writeln!(&mut formatted)?;
                        }
                        fs::File::create(&path)?.write_all(formatted.as_bytes())?;
                    }
                }
            }
        }
        Ok(())
    };

    process::exit(match result() {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("{}", e);
            1
        }
    })
}

#[derive(Error, Debug)]
enum Error {
    #[error("Failed to read from stdin: {0}")]
    Io(#[from] io::Error),
    #[error("Failed to read glob pattern: {0}")]
    Glob(#[from] glob::GlobError),
    #[error("Failed to read glob pattern: {0}")]
    Patter(#[from] glob::PatternError),
    #[error("Failed to append a trailing newline to the formatted SQL.")]
    Format(#[from] std::fmt::Error),
}

#[derive(Parser)]
#[clap(version = crate_version!())]
struct Options {
    /// File path(s) to format, supports glob patterns.
    /// If no file paths are provided, reads from stdin.
    file_paths: Vec<String>,
    /// Set the number of spaces to use for indentation
    #[clap(short, long, default_value = "4")]
    indent_spaces: u8,
    /// Change reserved keywords to ALL CAPS
    #[clap(long, default_value = "false")]
    uppercase: bool,
    /// Set the number of line breaks after a query
    #[clap(short, long, default_value = "2")]
    lines_between_queries: u8,
    /// Enforce a tailing newline at the end of the file
    #[clap(short = 'n', long, default_value = "false")]
    omit_newline: bool,
}
