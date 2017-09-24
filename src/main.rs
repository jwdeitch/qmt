extern crate iron;
extern crate time;
extern crate rusoto_core;
extern crate rusoto_s3;
extern crate multipart;

mod chunking;

use std::fs;
use std::thread;
use rusoto_s3::{S3, S3Client, PutObjectRequest};
use rusoto_core::{DefaultCredentialsProvider, Region};
use rusoto_core::default_tls_client;
use std::io::Read;
use multipart::server::{Multipart, Entries, SaveResult, SavedFile};
use iron::prelude::*;
use iron::status;

fn main() {
    let _server = Iron::new(process_request).http("localhost:3000").unwrap();
    println!("On 3000");
}

fn process_request(request: &mut Request) -> IronResult<Response> {
    let upload_id = &time::now().tm_nsec.to_string();
    match Multipart::from_request(request) {
        Ok(mut multipart) => {
            match multipart.save().temp() {
                SaveResult::Full(entries) => process_entries(entries, upload_id),
                SaveResult::Partial(entries, reason) => {
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

fn process_entries(entries: Entries, upload_id: &str) -> IronResult<Response> {
    for (name, field) in entries.fields {
        println!("[{}] Field {:?}: {:?}", upload_id, name, field);
    }

    for (name, files) in entries.files {
        println!("[{}] Field {:?} has {} files:", upload_id, name, files.len());

        for file in files {
            let job = chunking::create_working_dirs(upload_id, file);
            println!("[{}] {:?}", upload_id, job.original_upload_dir);
            chunking::chunk(&job);
            write_chunks(&job);
        }
    }

    Ok(Response::with((status::Ok, "Multipart data is processed")))
}


pub fn write_chunks(job: &chunking::Job) {
    println!("Starting writes to S3: {}", job.cononical_name.to_string_lossy());
    let mut children = vec![];
    for chunk in fs::read_dir(&job.output_dir).expect("cannot read chunk directory") {
        let chunk_path = chunk.expect("cannot enumerate chunk path").path();
        println!("Uploading: {}", chunk_path.display());
        let mut f = fs::File::open(chunk_path.display().to_string()).unwrap();
        let mut contents: Vec<u8> = Vec::new();
        match f.read_to_end(&mut contents) {
            Err(why) => panic!("Error opening file to send to S3: {}", why),
            Ok(_) => {
                let req = PutObjectRequest {
                    bucket: String::from("cdn.rsa.pub"),
                    key: "t/".to_string() + &job.upload_id + "/" + &chunk_path.file_name().expect("Cannot deduce chunk filename").to_string_lossy().to_owned(),
                    body: Some(contents),
                    ..Default::default()
                };
                children.push(thread::spawn(move || {
                    let provider = DefaultCredentialsProvider::new().unwrap();
                    let client = S3Client::new(
                        default_tls_client().expect("Unable to retrieve default TLS client"),
                        DefaultCredentialsProvider::new().expect("Unable to retrieve AWS credentials"),
                        Region::UsEast1
                    );
                    client.put_object(&req)
                }));
            }
        }
    }
    for child in children {
        // Wait for the thread to finish. Returns a result.
        let _ = child.join();
    }
    println!("Finishing writes to S3: {}", job.cononical_name.to_string_lossy());
}
