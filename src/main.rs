use std::process::Command;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    assert_eq!(args.len(), 2);
    let input_file: Vec<String> = args[1].split(".").map(|s| s.to_string()).collect();
    assert_eq!(input_file.len(), 2);
    let input_file_name = &input_file[0];
    let input_file_extension = &input_file[1];

    println!("Starting to chunk: {}.{}", input_file_name, input_file_extension);

    assert!(1 == 2);

    struct OutputPackage {
        stdout: String,
        stderr: String,
        status: std::process::ExitStatus
    }

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

    println!("Finished chunking on: {}.{}", input_file_name, input_file_extension);

}


