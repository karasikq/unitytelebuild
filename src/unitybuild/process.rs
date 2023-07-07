use std::path::PathBuf;
use std::process::Command;
use std::process::Output;
use std::process::Stdio;

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
    platform: Option<BuildPlatform>,
    log_behavior: Option<LogBehaviour>,
    command: Option<Command>,
    project_path: Option<PathBuf>,
    log_path: Option<PathBuf>,
}

impl UnityProcess {
    pub fn new() -> Self {
        UnityProcess {
            platform: None,
            log_behavior: None,
            command: None,
            project_path: None,
            log_path: None,
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

    pub fn build(&mut self) -> Result<Output, std::io::Error> {
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
            .arg("-logFile");

        let log_behavior = self.log_behavior.unwrap();
        match log_behavior {
            LogBehaviour::Stdout => {
                command
                    .arg("-")
                    .stdin(Stdio::inherit())
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit());
            }
            LogBehaviour::StdoutFile => {
                command
                    .arg("-")
                    .stdin(Stdio::inherit())
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit());
            }
            LogBehaviour::File => {
                command
                    .arg(
                        self.log_path
                            .as_ref()
                            .expect("Need to specify a log path")
                            .to_str()
                            .unwrap(),
                    )
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped());
            }
        };
        command.output()
    }
}

impl Default for UnityProcess {
    fn default() -> Self {
        Self::new()
    }
}
