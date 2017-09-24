use SavedFile;
use std::path::PathBuf;
use std;

struct OutputPackage {
    stdout: String,
    stderr: String,
    status: std::process::ExitStatus
}

pub struct Job {
    pub original_upload_dir: PathBuf,
    pub upload_id: String,
    pub output_dir: PathBuf,
    pub canonical_name: PathBuf,
    pub file_ext: String,
    pub original_upload_file: PathBuf
}

pub fn chunk(job: &Job) {
    println!("[{}] Starting chunking: {}", &job.upload_id, job.canonical_name.to_string_lossy());

    let mut output_dir_formatted = job.output_dir
        .join("output-%03d");

    output_dir_formatted.set_extension(&job.file_ext);

    let output = std::process::Command::new("ffmpeg")
        .args(&["-i", &job.canonical_name.to_string_lossy().into_owned()])
        .args(&["-c", "copy"])
        .args(&["-f", "segment"])
        .args(&["-segment_time", "20"])
        .args(&["-reset_timestamps", "1"])
        .args(&["-map", "0"])
        .arg(&output_dir_formatted)
        .output()
        .expect("failed to execute process");

    println!("[{}] {} -> {:?}",
             &job.upload_id,
             &job.canonical_name.to_string_lossy().into_owned(),
             &output_dir_formatted
    );

    let package = OutputPackage {
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        status: output.status
    };

    println!("[{}] Finishing chunking: {}", &job.upload_id, job.canonical_name.to_string_lossy());
}

fn parse_args() -> PathBuf {
    let args: Vec<String> = std::env::args().collect();

    return PathBuf::from(args[1].to_owned());
}

pub fn create_working_dirs(upload_id: &str, uploaded_file: SavedFile) -> Job {
    let job_working_dir = std::env::current_dir()
        .expect("Can not determine current directory").join(upload_id);

    let original_upload_dir = job_working_dir.join("original");
    let output_dir = job_working_dir.join("chunks");

    std::fs::create_dir_all(
        &original_upload_dir
    ).expect("Can not create tmp working directory");

    std::fs::create_dir(
        &output_dir
    ).expect("Can not create tmp working directory");

    let original_upload_filename = uploaded_file
        .filename
        .expect("Unable to deduce original filename")
        .to_owned();

    std::fs::copy(&uploaded_file.path, &original_upload_dir
        .join(&original_upload_filename)
    ).expect("failed copying uploaded file to working dir");

    let original_upload_file = original_upload_dir
        .join(&original_upload_filename);

    let canonical_name = std::fs::canonicalize(
        original_upload_file.file_name()
            .expect("No input filename found"))
        .expect("Can't canonicalize input argument");

    let file_ext = original_upload_file.extension()
        .expect("No input file extension found")
        .to_string_lossy()
        .into_owned();

    return Job {
        original_upload_dir,
        output_dir,
        canonical_name,
        file_ext,
        original_upload_file,
        upload_id: String::from(upload_id)
    };
}