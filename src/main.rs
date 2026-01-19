mod apply;

use std::path::PathBuf;
use std::process::exit;
use crate::apply::{apply, Config};

fn main() {
    let (config, path) = parse_args();

    if let Err(e) = apply(config, path) {
        eprintln!("Encountered an error: {}", e);
        exit(1);
    }
}

fn parse_args() -> (Config, PathBuf) {
    let mut config = Config::default();
    let mut positional = Vec::new();

    let args : Vec<String> = std::env::args().skip(1).collect();

    for arg in &args {
        match arg.as_str() {
            "--recursive" | "-R" => config.recursive = true,
            "--verbose" | "-v" => config.verbose = true,
            "--xattr" | "-x" => config.with_xattr = true,
            "--no-permissions" => config.no_permissions = true,
            "--dry-run" => config.dry_run = true,
            "--help" | "-h" => print_usage(),
            a => positional.push(a),
        }
    }

    if positional.len() != 2 {
        eprintln!("Error: Missing mandatory arguments.");
        print_usage();
    }

    config.difference = positional[0].parse::<i32>().unwrap_or_else(|_| {
        eprintln!("Error: <difference> must be an integer.");
        exit(2);
    });

    let path = PathBuf::from(&positional[1]);

    (config, path)
}

fn print_usage() -> ! {
    println!("Usage: chownshift <difference> <path> [Optional Arguments]");
    println!();
    println!("Mandatory Arguments:");
    println!("  {:<20} A positive or negative integer denoting the amount of UIDs and GIDs", "<difference>:");
    println!("  {:<20} the files in <path> are to be shifted by", "");
    println!("  {:<20} The target path to a file or directory", "<path>:");
    println!();
    println!("Optional Arguments:");
    println!("  {:<4}{:<20} Recurse through the provided path", "-R,", "--recursive");
    println!("  {:<4}{:<20} Verbose output", "-v,", "--verbose");
    println!("  {:<4}{:<20} Preserve security capability sets (security.capability extended file attribute)", "-x,", "--xattr");
    println!("  {:<4}{:<20} Do not preserve mode permission bits", "", "--no-permissions");
    println!("  {:<4}{:<20} Simulate only", "", "--dry-run");
    println!("  {:<4}{:<20} Show this message", "-h,", "--help");
    println!();
    println!("Exit codes:");
    println!("  0 - Success");
    println!("  1 - Error");
    println!("  2 - Argument error");

    exit(2);
}