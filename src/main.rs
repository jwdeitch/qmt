extern crate iron;
extern crate time;
extern crate rusoto_core;
extern crate rusoto_s3;
extern crate multipart;

use std::process::Command;
use std::path::PathBuf;
use std::fs;
use std::env;
use rusoto_s3::{S3, S3Client, PutObjectRequest};
use rusoto_core::{DefaultCredentialsProvider, Region};
use rusoto_core::default_tls_client;
use std::io::Read;
use multipart::server::{Multipart, Entries, SaveResult, SavedFile};
use iron::prelude::*;
use iron::status;

struct Job {
    original_upload_dir: PathBuf,
    upload_id: String,
    output_dir: PathBuf,
    cononical_name: PathBuf,
    file_ext: String,
    original_upload_file: PathBuf
}

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
            let job = create_working_dirs(upload_id, file);
            println!("[{}] {:?}", upload_id, job.original_upload_dir);
            chunk(job);
        }
    }

    Ok(Response::with((status::Ok, "Multipart data is processed")))
}


fn write_chunks(job: Job) {
            println!("Starting writes to S3: {}", job.cononical_name.to_string_lossy());
        let provider = DefaultCredentialsProvider::new().unwrap();
        let client = S3Client::new(
            default_tls_client().expect("Unable to retrieve default TLS client"),
            DefaultCredentialsProvider::new().expect("Unable to retrieve AWS credentials"),
            Region::UsEast1
        );

        for chunk in fs::read_dir(job.output_dir).expect("cannot read chunk directory") {
            let chunk_path = chunk.expect("cannot enumerate chunk path").path().display();
            println!("Uploading: {}", chunk_path);
            let mut f = fs::File::open(chunk_path.to_string()).unwrap();
            let mut contents: Vec<u8> = Vec::new();
            match f.read_to_end(&mut contents) {
                Err(why) => panic!("Error opening file to send to S3: {}", why),
                Ok(_) => {
                    let req = PutObjectRequest {
                        bucket: String::from("cdn.rsa.pub"),
                        key: "t/" + job.upload_id.to_owned() + "/" + chunk.path().expect("Cannot deduce chunk path").file_name(),
                        body: Some(contents),
                        ..Default::default()
                    };
                    let result = client.put_object(&req);
                    println!("{:#?}", result);
                }
            }
        }

    //        println!("Finishing writes to S3: {}", canonical_filename.to_string_lossy());
}

fn chunk(job: Job) {
    println!("Starting chunking: {}", job.cononical_name.to_string_lossy());

    let mut output_dir_formatted = job.output_dir
        .join("output-%03d");

    output_dir_formatted.set_extension(&job.file_ext);

    let output = Command::new("ffmpeg")
        .args(&["-i", &job.cononical_name.to_string_lossy().into_owned()])
        .args(&["-c", "copy"])
        .args(&["-f", "segment"])
        .args(&["-segment_time", "20"])
        .args(&["-reset_timestamps", "1"])
        .args(&["-map", "0"])
        .arg(job.output_dir.to_string_lossy().into_owned())
        .output()
        .expect("failed to execute process");

    let package = OutputPackage {
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        status: output.status
    };

    write_chunks(job);
    println!("Finishing chunking: {}", job.cononical_name.to_string_lossy());
}

fn parse_args() -> PathBuf {
    let args: Vec<String> = env::args().collect();

    return PathBuf::from(args[1].to_owned());
}

fn create_working_dirs(upload_id: &str, uploaded_file: SavedFile) -> Job {
    let job_working_dir = env::current_dir()
        .expect("Can not determine current directory").join(upload_id);

    let original_dir = job_working_dir.join("original");
    let chunk_dir = job_working_dir.join("chunks");

    fs::create_dir_all(
        &original_dir
    ).expect("Can not create tmp working directory");

    fs::create_dir(
        &chunk_dir
    ).expect("Can not create tmp working directory");

    fs::copy(&uploaded_file.path, &original_dir);

    let original_uploaded_file = original_dir
        .join(uploaded_file.filename.expect("Unable to find original filename"));

    let cononical_name = fs::canonicalize(
        original_uploaded_file.file_name()
            .expect("No input filename found"))
        .expect("Can't canonicalize input argument");

    let file_ext = original_uploaded_file.extension()
        .expect("No input file extension found")
        .to_string_lossy()
        .into_owned();

    return Job {
        original_upload_dir: original_dir,
        output_dir: chunk_dir,
        cononical_name: cononical_name,
        file_ext: file_ext,
        original_upload_file: original_uploaded_file,
        upload_id: String::from(upload_id)
    };
}