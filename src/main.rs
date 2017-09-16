extern crate iron;

use std::process::Command;
use std::env;
use iron::prelude::*;
use iron::status;

struct Arguments {
    pub input_file_name: String,
    pub input_file_extension: String
}

struct OutputPackage {
    stdout: String,
    stderr: String,
    status: std::process::ExitStatus
}

fn main() {



}

fn chunk(file_name: String, file_ext: String) {
    let args = parse_args();

    println!("Starting to chunk: {}.{}", args.input_file_name, args.input_file_extension);

    assert!(1 == 2);

    let output = Command::new("ffmpeg")
        .args(&["-i", "/Users/jordan1/IdeaProjects/untitled6/input.mp4"])
        .args(&["-c", "copy"])
        .args(&["-f", "segment"])
        .args(&["-segment_time", "20"])
        .args(&["-reset_timestamps", "1"])
        .args(&["-map", "0"])
        .arg("output-%03d.mp4")
        .output()
        .expect("failed to execute process");

    let package = OutputPackage{
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        status: output.status
    };

    println!("Finished chunking on: {}.{}", args.input_file_name, args.input_file_extension);
}

fn parse_args() -> Arguments {
    let args: Vec<String> = env::args().collect();
    assert_eq!(args.len(), 2);
    let input_file: Vec<String> = args[1].split(".").map(|s| s.to_string()).collect();
    assert_eq!(input_file.len(), 2);

    return Arguments{
        input_file_name: input_file[0].to_owned(),
        input_file_extension: input_file[1].to_owned()
    }
}

