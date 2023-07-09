use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Child;
use tokio::process::Command;
use uuid::Uuid;

#[derive(Clone, Copy)]
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
    command: Option<Command>,
    pub project_path: Option<PathBuf>,
    pub log_path: Option<PathBuf>,
    pub uuid: Uuid,
}

impl UnityProcess {
    pub fn new() -> Self {
        UnityProcess {
            platform: None,
            log_behavior: None,
            command: None,
            project_path: None,
            log_path: None,
            uuid: Uuid::now_v1(&[1, 2, 3, 4, 5, 6]),
        }
    }

    pub fn set_bin(&mut self, path: PathBuf) -> &mut Self {
        self.command = Some(Command::new(path));
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

    pub fn set_log_path(&mut self, log_path: PathBuf) -> &mut Self {
        self.log_path = Some(log_path);
        self
    }

    pub async fn build(&mut self) -> Result<Child, std::io::Error> {
        let command = self.command.as_mut().unwrap();
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
            .arg("Build.BuildActions.AndroidDevelopment")
            .arg("-buildTarget")
            .arg("android")
            .arg("-logFile")
            .arg("-")
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
