use std::path::Path;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Child;
use tokio::signal::ctrl_c;
use unitytelebuild::unitybuild::process::{BuildPlatform, LogBehaviour, UnityProcess};

#[derive(Clone, Serialize, Deserialize)]
struct BuildSettings {
    platform: BuildPlatform,
    keystore_password: String,
    build_path: String,
}

pub fn projects_root() -> PathBuf {
    dotenv::var("PROJECTS_LOCATION")
        .expect("Environment variable PROJECTS_LOCATION should be set in '.env'")
        .into()
}

pub fn projects_root_unity() -> Result<String, dotenv::Error> {
    dotenv::var("PROJECTS_LOCATION_UNITY")
}

pub fn log_path() -> String {
    dotenv::var("UNITY_LOG_PATH")
        .expect("Environment variable UNITY_LOG_PATH should be set in '.env'")
}

pub fn build_path(platform: &BuildPlatform) -> String {
    match platform {
        BuildPlatform::AndroidDevelopment => dotenv::var("UNITY_ANDROID_BUILD_PATH_DEV")
            .expect("Environment variable UNITY_ANDROID_BUILD_PATH_DEV should be set in '.env'"),
        BuildPlatform::AndroidRelease => dotenv::var("UNITY_ANDROID_BUILD_PATH_REL")
            .expect("Environment variable UNITY_ANDROID_BUILD_PATH_REL should be set in '.env'"),
    }
}

pub fn get_projects() -> Vec<PathBuf> {
    fs::read_dir(projects_root())
        .expect("Cannot read PROJECTS_LOCATION directory")
        .filter(|f| f.as_ref().unwrap().path().is_dir())
        .map(|f| f.unwrap().path())
        .collect::<Vec<_>>()
}

pub async fn unity_build(project_name: &String) {
    let mut unity_process = UnityProcess::new();

    let project_path = projects_root().join(project_name);
    let telebuild_root = PathBuf::from(dotenv::var("TELEBUILD_ROOT").unwrap())
        .join(format!("{}", unity_process.uuid));
    println!(
        "Telebuild project root: {}",
        telebuild_root.to_str().unwrap()
    );
    let process_root = project_path.join(telebuild_root.clone());
    let log_directory = process_root.join(log_path());

    let _ = std::fs::create_dir_all(log_directory.to_str().unwrap());
    let default_log_path = log_directory.join("androind_build.log");

    log::info!("{}", default_log_path.to_str().unwrap());

    let log_to_stdout = dotenv::var("LOG_TO_STDOUT")
        .expect("Environment variable LOG_TO_STDOUT should be set in '.env'")
        .parse::<bool>()
        .unwrap();

    let process = unity_process.set_bin(dotenv::var("UNITY_BIN").unwrap().into());
    if log_to_stdout {
        process.set_log_behavior(LogBehaviour::StdoutFile);
    } else {
        process.set_log_behavior(LogBehaviour::File);
    }

    process
        .set_platform(BuildPlatform::AndroidDevelopment)
        .set_project_path(match projects_root_unity() {
            Ok(path) => PathBuf::from(path).join(project_name),
            Err(_) => project_path,
        })
        .set_telebuild_root(telebuild_root);
    let platform = *process.platform.as_ref().unwrap();
    let settings = BuildSettings {
        platform,
        keystore_password: dotenv::var("KEYSTORE_PASSWORD").unwrap(),
        build_path: build_path(&platform),
    };
    create_build_settings(&process_root, &settings);

    let mut child = process.build().await.unwrap();

    tokio::select! {
        _ = handle_output(&default_log_path, process, &mut child) => { }
        _ = ctrl_c() => {

            log::warn!("Ctrl-C received. Terminating unity process...");
            child.kill().await.unwrap();
            log::info!("Unity process has been terminated.");
        }
    }
}

fn create_build_settings(root: &Path, settings: &BuildSettings) {
    let file_path = root.join("settings.json");
    let mut file = fs::File::create(file_path).expect("Cannot create 'settings.json' file");
    let _ = file.write_all(serde_json::to_string(settings).unwrap().as_bytes());
}

async fn handle_output(log_path: &PathBuf, unity_process: &mut UnityProcess, process: &mut Child) {
    let stdout = process.stdout.take().expect("Failed to get stdout");
    let mut reader = BufReader::new(stdout);
    let mut file = fs::File::create(log_path).expect("Cannot create log file");
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
