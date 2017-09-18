extern crate iron;
extern crate time;

use std::process::Command;
use std::path::PathBuf;
use std::fs;
use std::env;
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

    chunk(&time::now().tm_nsec.to_string());
}

fn write_chunks() {
    //    println!("Starting writes to S3: {}", canonical_filename.to_string_lossy());


    //    println!("Finishing writes to S3: {}", canonical_filename.to_string_lossy());
}

fn chunk(output_directory: &str) {
    let args = parse_args();

    let output_dir = env::current_dir()
        .expect("Can not determine current directory")
        .join(output_directory);

    fs::create_dir(
        &output_dir
    ).expect("Can not create tmp working directory");

    let canonical_filename = fs::canonicalize(
        args.file_name()
            .expect("No input filename found"))
        .expect("Can't canonicalize input argument");

    let file_ext = &args.extension()
        .expect("No input file extension found")
        .to_string_lossy()
        .into_owned();

        println!("Starting chunking: {}", canonical_filename.to_string_lossy());

    let mut output_dir_formatted = output_dir
        .join("output-%03d");

    output_dir_formatted.set_extension(file_ext);

    let output = Command::new("ffmpeg")
        .args(&["-i", &canonical_filename.to_string_lossy().into_owned()])
        .args(&["-c", "copy"])
        .args(&["-f", "segment"])
        .args(&["-segment_time", "20"])
        .args(&["-reset_timestamps", "1"])
        .args(&["-map", "0"])
        .arg(output_dir_formatted.to_string_lossy().into_owned())
        .output()
        .expect("failed to executse process");

    let package = OutputPackage {
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        status: output.status
    };

    println!("Finishing chunking: {}", canonical_filename.to_string_lossy());
}

fn parse_args() -> PathBuf {
    let args: Vec<String> = env::args().collect();

    return PathBuf::from(args[1].to_owned());
}