use std::error::Error;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Child;
use tokio::signal::ctrl_c;
use unitytelebuild::unitybuild::process::{BuildPlatform, LogBehaviour, UnityOutput, UnityProcess};

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

pub fn get_projects() -> Vec<PathBuf> {
    fs::read_dir(projects_root())
        .expect("Cannot read PROJECTS_LOCATION directory")
        .filter(|f| f.as_ref().unwrap().path().is_dir())
        .map(|f| f.unwrap().path())
        .collect::<Vec<_>>()
}

pub async fn unity_build(project_name: &String) -> Result<UnityOutput, Box<dyn Error + Send + Sync>> {
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
    let default_log_path = log_directory.join("android_build.log");

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
        .set_project_path(project_path.clone())
        .set_project_path_unity(match projects_root_unity() {
            Ok(path) => PathBuf::from(path).join(project_name),
            Err(_) => project_path,
        })
        .set_platform(BuildPlatform::AndroidDevelopment)
        .set_telebuild_root(telebuild_root)
        .set_build_entry(dotenv::var("BUILD_ENTRY").unwrap())
        .set_keystore_password(dotenv::var("KEYSTORE_PASSWORD").unwrap())
        .create_build_settings();

    let mut child = process.build().await.unwrap();

    tokio::select! {
        result = handle_output(&default_log_path, process, &mut child) => { result }
        _ = ctrl_c() => {
            child.kill().await.unwrap();
            log::info!("Unity process has been terminated.");
            Err("Ctrl-C received. Terminating unity process...")?
        }
    }
}

async fn handle_output(
    log_path: &PathBuf,
    unity_process: &mut UnityProcess,
    process: &mut Child,
) -> Result<UnityOutput, Box<dyn Error + Send + Sync>> {
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
    let result = process.wait().await.unwrap();
    log::info!(
        "Unity process exit code: {}",
        result.code().unwrap()
    );
    let mut output = unity_process.load_output().unwrap();
    output.log_path = Some(log_path.clone().into_os_string().into_string().unwrap());
    Ok(output)
}
