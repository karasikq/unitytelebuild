use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::OsStr;
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
    telebuild_root: Option<PathBuf>,
    envs: HashMap<String, String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UnityOutput {
    pub build_path: String,
    pub platform: BuildPlatform,
}

impl UnityProcess {
    pub fn new() -> Self {
        UnityProcess {
            platform: None,
            log_behavior: None,
            project_path: None,
            uuid: Uuid::now_v1(&[1, 2, 3, 4, 5, 6]),
            envs: HashMap::new(),
            bin_path: None,
            telebuild_root: None,
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

    pub fn set_telebuild_root(&mut self, telebuild_root: PathBuf) -> &mut Self {
        self.telebuild_root = Some(telebuild_root);
        self
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
                self.project_path
                    .as_ref()
                    .expect("Need to specify a project path"),
            )
            .arg("-executeMethod")
            .arg("Builds.BuildSystem.Build")
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
