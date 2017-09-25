use ffmpeg::Job;

use rusoto_s3::{S3, S3Client, PutObjectRequest};
use rusoto_core::{DefaultCredentialsProvider, Region};
use rusoto_core::default_tls_client;
use std::thread;
use std;
use std::fs;
use std::io::Read;

pub fn write_chunks(job: &Job) {
    println!("Starting writes to S3: {}", job.canonical_name.to_string_lossy());
    let mut put_children = vec![];
    let mut paths: Vec<_> = fs::read_dir(&job.output_dir)
        .expect("cannot read chunk directory")
        .map(|r| r.expect("cannot enumerate chunk path"))
        .collect();
    paths.sort_by_key(|dir| dir.path());
    for chunk in paths {
        let chunk_path = chunk.path();
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
                put_children.push(thread::spawn(move || {
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
    for PutChild in put_children {
        let _ = PutChild.join();
    }


    println!("Finishing writes to S3: {}", job.canonical_name.to_string_lossy());
}

pub fn wait_for_chunking_finish() {}