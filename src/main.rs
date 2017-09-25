extern crate iron;
extern crate time;
extern crate rusoto_core;
extern crate rusoto_s3;
extern crate multipart;

mod ffmpeg;
mod s3;

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
            let job = ffmpeg::create_working_dirs(upload_id, file);
            println!("[{}] {:?}", upload_id, job.original_upload_dir);
            ffmpeg::chunk(&job);
            s3::write_chunks(&job);
        }
    }

    Ok(Response::with((status::Ok, "Multipart data is processed")))
}