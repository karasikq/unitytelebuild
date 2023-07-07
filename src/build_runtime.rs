use std::fs;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use unitytelebuild::unitybuild::process::{BuildPlatform, LogBehaviour, UnityProcess};

pub fn projects_root() -> PathBuf {
    dotenv::var("PROJECTS_LOCATION")
        .expect("Environment variable PROJECTS_LOCATION should be set in '.env'")
        .into()
}

pub fn get_projects() -> Vec<PathBuf> {
    fs::read_dir(projects_root())
        .expect("Cannot read PROJECTS_LOCATION directory")
        .filter(|f| f.as_ref().unwrap().path().is_dir())
        .map(|f| f.unwrap().path())
        .collect::<Vec<_>>()
}

pub fn unity_build(project_name: &String) {
    let project_path = projects_root().join(project_name);
    let default_log_path = project_path
        .join(
            dotenv::var("UNITY_LOG_PATH")
                .expect("Environment variable UNITY_LOG_PATH should be set in '.env'"),
        )
        .join("androind_build.log");

    let log_to_stdout = dotenv::var("LOG_TO_STDOUT")
        .expect("Environment variable LOG_TO_STDOUT should be set in '.env'")
        .parse::<bool>()
        .unwrap();

    let mut unity_process = UnityProcess::new();
    let process = unity_process.set_bin(dotenv::var("UNITY_BIN").unwrap().into());
    if log_to_stdout {
        process.set_log_behavior(LogBehaviour::StdoutFile);
    } else {
        process
            .set_log_behavior(LogBehaviour::File)
            .set_log_path(default_log_path.to_str().unwrap().into());
    }
    process
        .set_platform(BuildPlatform::AndroidDevelopment)
        .set_project_path(project_path);

    let output = process.build().unwrap_or_else(|_| {
        panic!(
            "Failed to execute Unity process. See logs at {0}",
            default_log_path.to_str().unwrap()
        )
    });

    println!("{}", default_log_path.to_str().unwrap());
    println!("Process status: {}", output.status);
    match process.log_behavior.as_ref().unwrap() {
        LogBehaviour::Stdout => {}
        LogBehaviour::StdoutFile => {
            stdout_to_file(default_log_path, &output.stdout);
        }
        LogBehaviour::File => {
            stdout_to_file(default_log_path, &output.stdout);
        }
    };
}

fn stdout_to_file(file_path: PathBuf, stream: &[u8]) {
    let mut file = fs::File::create(file_path).expect("Cannot create log file");
    file.write_all(stream).unwrap();
}