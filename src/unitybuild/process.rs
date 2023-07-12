use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::io::{BufReader, Write};
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Child;
use tokio::process::Command;
use uuid::Uuid;

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum BuildPlatform {
    AndroidDevelopment,
    AndroidRelease,
}

#[derive(Clone, Copy)]
pub enum LogBehaviour {
    Stdout,
    StdoutFile,
    File,
}

pub struct UnityProcess {
    pub platform: Option<BuildPlatform>,
    pub log_behavior: Option<LogBehaviour>,
    pub project_path: Option<PathBuf>,
    pub bin_path: Option<PathBuf>,
    pub uuid: Uuid,
    keystore_password: Option<String>,
    project_path_unity: Option<PathBuf>,
    script_build_entry: Option<String>,
    telebuild_root: Option<PathBuf>,
    envs: HashMap<String, String>,
}

#[derive(Clone, Serialize, Deserialize)]
struct BuildSettings {
    platform: BuildPlatform,
    keystore_password: String,
    build_path: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UnityOutput {
    pub build_path: String,
    pub platform: BuildPlatform,
    #[serde(skip_serializing)]
    pub log_path: Option<String>,
    #[serde(skip_serializing)]
    pub exit_code: Option<i32>,
}

impl UnityProcess {
    pub fn new() -> Self {
        UnityProcess {
            platform: None,
            log_behavior: None,
            project_path: None,
            bin_path: None,
            uuid: Uuid::now_v1(&[1, 2, 3, 4, 5, 6]),
            keystore_password: None,
            project_path_unity: None,
            script_build_entry: None,
            telebuild_root: None,
            envs: HashMap::new(),
        }
    }

    pub fn set_bin(&mut self, path: PathBuf) -> &mut Self {
        self.bin_path = Some(path);
        self
    }

    pub fn set_platform(&mut self, platform: BuildPlatform) -> &mut Self {
        self.platform = Some(platform);
        self
    }

    pub fn set_log_behavior(&mut self, log_behavior: LogBehaviour) -> &mut Self {
        self.log_behavior = Some(log_behavior);
        self
    }

    pub fn set_project_path(&mut self, project_path: PathBuf) -> &mut Self {
        self.project_path = Some(project_path);
        self
    }

    pub fn set_project_path_unity(&mut self, project_path: PathBuf) -> &mut Self {
        self.project_path_unity = Some(project_path);
        self
    }

    pub fn set_build_entry(&mut self, function_full_name: String) -> &mut Self {
        self.script_build_entry = Some(function_full_name);
        self
    }

    pub fn set_keystore_password(&mut self, password: String) -> &mut Self {
        self.keystore_password = Some(password);
        self
    }

    pub fn set_telebuild_root(&mut self, telebuild_root: PathBuf) -> &mut Self {
        self.telebuild_root = Some(telebuild_root);
        self
    }

    pub fn build_path(&self) -> String {
        match self.platform.as_ref().unwrap() {
            BuildPlatform::AndroidDevelopment => dotenv::var("UNITY_ANDROID_BUILD_PATH_DEV")
                .expect(
                    "Environment variable UNITY_ANDROID_BUILD_PATH_DEV should be set in '.env'",
                ),
            BuildPlatform::AndroidRelease => dotenv::var("UNITY_ANDROID_BUILD_PATH_REL").expect(
                "Environment variable UNITY_ANDROID_BUILD_PATH_REL should be set in '.env'",
            ),
        }
    }

    pub fn load_output(&self) -> Result<UnityOutput, Box<dyn Error + Send + Sync>> {
        let file_path = self
            .project_path
            .as_ref()
            .unwrap()
            .join(self.telebuild_root.as_ref().unwrap())
            .join("output.json");
        let file = fs::File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut output: UnityOutput = serde_json::from_reader(reader)?;
        let path = PathBuf::from(self.telebuild_root.as_ref().unwrap()).join(output.build_path);
        output.build_path = path.into_os_string().into_string().unwrap();
        Ok(output)
    }

    pub fn create_build_settings(&self) {
        let root = self
            .project_path
            .as_ref()
            .unwrap()
            .join(self.telebuild_root.as_ref().unwrap());
        let file_path = root.join("settings.json");
        let settings = BuildSettings {
            platform: *self.platform.as_ref().unwrap(),
            keystore_password: self.keystore_password.as_ref().unwrap().to_string(),
            build_path: self.build_path(),
        };
        let _ = std::fs::create_dir_all(root.to_str().unwrap());
        let mut file = fs::File::create(file_path).expect("Cannot create 'settings.json' file");
        let _ = file.write_all(serde_json::to_string(&settings).unwrap().as_bytes());
    }

    pub fn set_env<K, V>(&mut self, key: K, val: V) -> &mut Self
    where
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        self.envs.insert(
            key.as_ref().to_string_lossy().into_owned(),
            val.as_ref().to_string_lossy().into_owned(),
        );
        self
    }

    pub async fn build(&mut self) -> Result<Child, std::io::Error> {
        let mut command = Command::new(self.bin_path.as_ref().unwrap());
        command.envs(&self.envs);
        command
            .arg("-batchmode")
            .arg("-quit")
            .arg("-projectPath")
            .arg(
                self.project_path_unity
                    .as_ref()
                    .expect("Need to specify a project path"),
            )
            .arg("-executeMethod")
            .arg(
                self.script_build_entry
                    .as_ref()
                    .expect("Need to specify build entry function"),
            )
            .arg("-buildTarget")
            .arg("android")
            .arg("-logFile")
            .arg("-")
            .arg("-teleroot")
            .arg(
                self.telebuild_root
                    .as_ref()
                    .expect("Need to specify a telebuild root"),
            )
            .kill_on_drop(true)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        command.spawn()
    }
}

impl Default for UnityProcess {
    fn default() -> Self {
        Self::new()
    }
}
