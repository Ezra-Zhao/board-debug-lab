//! CLI: board-log-parser <logfile> [--json] [--failures-only]
//! Exit code: 0 = no FAIL/ERROR results, 1 = failures present, 2 = usage/IO error.

use std::env;
use std::fs;
use std::process::ExitCode;

use board_log_parser::parse_log;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut json = false;
    let mut failures_only = false;
    let mut path: Option<String> = None;
    for a in &args {
        match a.as_str() {
            "--json" => json = true,
            "--failures-only" => failures_only = true,
            "-h" | "--help" => {
                println!("usage: board-log-parser <logfile> [--json] [--failures-only]");
                return ExitCode::SUCCESS;
            }
            other => {
                if path.is_some() {
                    eprintln!("unexpected extra argument: {other}");
                    return ExitCode::from(2);
                }
                path = Some(other.to_string());
            }
        }
    }
    let Some(path) = path else {
        eprintln!("usage: board-log-parser <logfile> [--json] [--failures-only]");
        return ExitCode::from(2);
    };
    let text = match fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("cannot read {path}: {e}");
            return ExitCode::from(2);
        }
    };
    let (results, summary) = parse_log(&text);
    if json {
        print!("{}", summary.to_json(&results));
    } else if failures_only {
        for r in summary.failure_results(&results) {
            println!(
                "line {}: {} {} codes=[{}]",
                r.line_no,
                r.test_name,
                r.status.as_str(),
                r.error_codes.join(", ")
            );
        }
    } else {
        print!("{}", summary.to_text(&results));
    }
    if summary.failed + summary.errors > 0 {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}
