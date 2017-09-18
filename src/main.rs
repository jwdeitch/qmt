extern crate iron;
extern crate time;
extern crate multipart;

use std::process::Command;
use std::path::PathBuf;
use std::fs;
//use mime::Mime;
use std::env;

use std::io::Read;
use multipart::server::{Multipart, Entries, SaveResult, SavedFile};
use iron::prelude::*;
use iron::status;

struct OutputPackage {
    stdout: String,
    stderr: String,
    status: std::process::ExitStatus
}

fn main() {
        let _server = Iron::new(process_request).http("localhost:3000").unwrap();
        println!("On 3000");
}

fn process_request(request: &mut Request) -> IronResult<Response> {
    let upload_id = &time::now().tm_nsec.to_string();
    // Getting a multipart reader wrapper
    match Multipart::from_request(request) {
        Ok(mut multipart) => {
            // Fetching all data and processing it.
            // save().temp() reads the request fully, parsing all fields and saving all files
            // in a new temporary directory under the OS temporary directory.
            match multipart.save().temp() {
                SaveResult::Full(entries) => process_entries(entries, upload_id),
                SaveResult::Partial(entries, reason) => {
                    process_entries(entries.keep_partial(), upload_id)?;
                    Err(IronError::new(reason.unwrap_err(), status::InternalServerError))
                }
                SaveResult::Error(error) => Err(IronError::new(error, status::InternalServerError)),
            }
        }
        Err(_) => {
            Ok(Response::with((status::BadRequest, "The request is not multipart")))
        }
    }
}

/// Processes saved entries from multipart request.
/// Returns an OK response or an error.
fn process_entries(entries: Entries, upload_id: &str) -> IronResult<Response> {
    for (name, field) in entries.fields {
        println!("[{}] Field {:?}: {:?}", upload_id, name, field);
    }

    for (name, files) in entries.files {
        println!("[{}] Field {:?} has {} files:", upload_id, name, files.len());

        for file in files {
            let working_dir = create_working_dirs(upload_id);
            let original_upload = working_dir.join("original").join(&file.filename.expect("Unable to find original filename"));
            fs::copy(file.path, original_upload.clone());

            println!("[{}] {:?}", upload_id, original_upload);
            chunk(original_upload, working_dir.join("chuncks"));
        }
    }

    Ok(Response::with((status::Ok, "Multipart data is processed")))
}


fn write_chunks() {
//        println!("Starting writes to S3: {}", canonical_filename.to_string_lossy());


//        println!("Finishing writes to S3: {}", canonical_filename.to_string_lossy());
}

fn chunk(original_upload: PathBuf, output_dir: PathBuf) {
    let canonical_filename = fs::canonicalize(
        original_upload.file_name()
            .expect("No input filename found"))
        .expect("Can't canonicalize input argument");

    let file_ext = &original_upload.extension()
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
        .expect("failed to execute process");

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

fn create_working_dirs(upload_id: &str) -> PathBuf {

    let original_upload_dir = env::current_dir()
        .expect("Can not determine current directory").join(upload_id);

    fs::create_dir(
        &original_upload_dir
    ).expect("Can not create tmp working directory");

    fs::create_dir(
        &original_upload_dir.join("original")
    ).expect("Can not create tmp working directory");

    fs::create_dir(
        &original_upload_dir.join("chuncks")
    ).expect("Can not create tmp working directory");

    return original_upload_dir;

}