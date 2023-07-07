use unitytelebuild::unitybuild::process::{BuildPlatform, LogBehaviour, UnityProcess};
use std::fs;
use std::path::Path;
use std::path::PathBuf;

pub fn get_projects() -> Vec<PathBuf> {
    fs::read_dir(
        dotenv::var("PROJECTS_LOCATION")
            .expect("Environment variable PROJECTS_LOCATION should be set in '.env'"),
    )
    .expect("Cannot read PROJECTS_LOCATION directory")
    .filter(|f| f.as_ref().unwrap().path().is_dir())
    .map(|f| f.unwrap().path())
    .collect::<Vec<_>>()
}

pub fn unity_build() {
    let paths = fs::read_dir(
        dotenv::var("PROJECTS_LOCATION")
            .expect("Environment variable PROJECTS_LOCATION should be set in '.env'"),
    )
    .expect("Cannot read PROJECTS_LOCATION directory")
    .filter(|f| f.as_ref().unwrap().path().is_dir())
    .map(|f| f.unwrap().path())
    .collect::<Vec<_>>();

    let project_path = paths.last().unwrap().to_str().unwrap();
    println!("{:?}", project_path);

    let default_log_path = Path::new(project_path)
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
        .set_project_path(project_path.into());

    let output = process.build().unwrap_or_else(|_| {
        panic!(
            "Failed to execute Unity process. See logs at {0}",
            default_log_path.to_str().unwrap()
        )
    });

    println!("Process status: {}", output.status);
    /* println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("stderr: {}", String::from_utf8_lossy(&output.stderr)); */
}
