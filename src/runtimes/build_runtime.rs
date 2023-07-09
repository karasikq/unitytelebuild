use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Child;
use tokio::signal::ctrl_c;
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

pub async fn unity_build(project_name: &String) {
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
        process.set_log_behavior(LogBehaviour::File);
    }
    process
        .set_log_path(default_log_path.to_str().unwrap().into())
        .set_platform(BuildPlatform::AndroidDevelopment)
        .set_project_path(project_path);

    let mut child = process.build().await.unwrap();

    tokio::select! {
        _ = handle_output(process, &mut child) => { }
        _ = ctrl_c() => {
        
            log::warn!("Ctrl-C received. Terminating unity process...");
            child.kill().await.unwrap();
            log::info!("Unity process has been terminated.");
        }
    }
}

async fn handle_output(unity_process: &mut UnityProcess, process: &mut Child) {
    let stdout = process.stdout.take().expect("Failed to get stdout");
    let mut reader = BufReader::new(stdout);
    let mut file =
        fs::File::create(unity_process.log_path.as_ref().unwrap()).expect("Cannot create log file");
    let mut line = String::new();
    while let Ok(n) = reader.read_line(&mut line).await {
        if n == 0 {
            break;
        }
        match unity_process.log_behavior.as_ref().unwrap() {
            LogBehaviour::Stdout => {
                print!("{}", line);
            }
            LogBehaviour::StdoutFile => {
                print!("{}", line);
                let _ = file.write_all(line.as_bytes());
            }
            LogBehaviour::File => {
                let _ = file.write_all(line.as_bytes());
            }
        };
        line.clear();
    }
    log::info!(
        "Unity process exit code: {}",
        process.wait().await.unwrap().code().unwrap()
    );
}
