extern crate iron;

use std::process::Command;
use std::env;
use std::path::PathBuf;
use std::fs;
//use iron::prelude::*;
//use iron::status;

struct OutputPackage {
    stdout: String,
    stderr: String,
    status: std::process::ExitStatus
}

fn main() {
    //    fn hello_world(_: &mut Request) -> IronResult<Response> {
    //        Ok(Response::with((status::Ok, "Hello World!")))
    //    }
    //
    //    let _server = Iron::new(hello_world).http("localhost:3000").unwrap();
    //    println!("On 3000");
    chunk()
}

fn chunk() {
    let args = parse_args();

    let canonical_filename = fs::canonicalize(
        args.file_name()
            .expect("No input filename found"))
        .expect("Can't canonicalize input argument");

    let file_ext = &args.extension()
        .expect("No input file extension found")
        .to_string_lossy()
        .into_owned();
    println!("Starting: {}", canonical_filename.to_string_lossy());

    assert!(1 == 2);

    let output = Command::new("ffmpeg")
        .args(&["-i", &canonical_filename.to_string_lossy().into_owned()])
        .args(&["-c", "copy"])
        .args(&["-f", "segment"])
        .args(&["-segment_time", "20"])
        .args(&["-reset_timestamps", "1"])
        .args(&["-map", "0"])
        .arg(&(String::from("output-%03d.") + &file_ext))
        .output()
        .expect("failed to execute process");

    let package = OutputPackage {
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        status: output.status
    };

    println!("Finishing: {}", canonical_filename.to_string_lossy());
}

fn parse_args() -> PathBuf {
    let args: Vec<String> = env::args().collect();

    return PathBuf::from(args[1].to_owned());
}

