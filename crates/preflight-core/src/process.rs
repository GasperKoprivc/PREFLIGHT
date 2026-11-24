use std::{future::Future, process::Stdio, time::Duration};
use tokio::{
    io::{AsyncRead, AsyncReadExt},
    process::Command,
};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Invocation {
    Python,
    Python3,
    Py3,
    Git,
    Node,
    Java,
    Javac,
    CodeExtensions,
}
impl Invocation {
    fn command(self) -> (&'static str, &'static [&'static str]) {
        match self {
            Self::Python => ("python", &["--version"]),
            Self::Python3 => ("python3", &["--version"]),
            Self::Py3 => ("py", &["-3", "--version"]),
            Self::Git => ("git", &["--version"]),
            Self::Node => ("node", &["--version"]),
            Self::Java => ("java", &["-version"]),
            Self::Javac => ("javac", &["-version"]),
            Self::CodeExtensions => ("code", &["--list-extensions"]),
        }
    }
}
#[derive(Debug, Clone)]
pub struct Output {
    pub stdout: String,
    pub stderr: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessError {
    NotFound,
    Timeout,
    OutputLimit,
    Failed,
}
pub trait ProcessRunner: Sync {
    fn run(
        &self,
        invocation: Invocation,
    ) -> impl Future<Output = Result<Output, ProcessError>> + Send;
}
pub struct SystemRunner;
impl ProcessRunner for SystemRunner {
    async fn run(&self, invocation: Invocation) -> Result<Output, ProcessError> {
        #[cfg(windows)]
        if invocation == Invocation::CodeExtensions {
            return execute(windows_code()?, Duration::from_secs(5), 32 * 1024).await;
        }
        if std::env::var_os("PATH").is_none_or(|p| p.is_empty()) {
            return Err(ProcessError::NotFound);
        }
        let (program, args) = invocation.command();
        let mut command = Command::new(program);
        command.args(args);
        #[cfg(windows)]
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW
        let result = execute(command, Duration::from_secs(5), 32 * 1024).await;
        #[cfg(target_os = "macos")]
        if invocation == Invocation::CodeExtensions && matches!(result, Err(ProcessError::NotFound))
        {
            let mut fallback = Command::new(
                "/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code",
            );
            fallback.arg("--list-extensions");
            return execute(fallback, Duration::from_secs(5), 32 * 1024).await;
        }
        result
    }
}
async fn read_bounded(
    reader: impl AsyncRead + Unpin,
    limit: usize,
) -> Result<Vec<u8>, ProcessError> {
    let mut bytes = Vec::new();
    reader
        .take((limit + 1) as u64)
        .read_to_end(&mut bytes)
        .await
        .map_err(|_| ProcessError::Failed)?;
    if bytes.len() > limit {
        Err(ProcessError::OutputLimit)
    } else {
        Ok(bytes)
    }
}
async fn execute(
    mut command: Command,
    timeout: Duration,
    limit: usize,
) -> Result<Output, ProcessError> {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    let mut child = command.spawn().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            ProcessError::NotFound
        } else {
            ProcessError::Failed
        }
    })?;
    let stdout = child.stdout.take().ok_or(ProcessError::Failed)?;
    let stderr = child.stderr.take().ok_or(ProcessError::Failed)?;
    let result = tokio::time::timeout(timeout, async {
        let (out, err, status) = tokio::try_join!(
            read_bounded(stdout, limit),
            read_bounded(stderr, limit),
            async { child.wait().await.map_err(|_| ProcessError::Failed) }
        )?;
        if out.len() + err.len() > limit {
            return Err(ProcessError::OutputLimit);
        }
        if !status.success() {
            return Err(ProcessError::Failed);
        }
        Ok(Output {
            stdout: String::from_utf8_lossy(&out).into_owned(),
            stderr: String::from_utf8_lossy(&err).into_owned(),
        })
    })
    .await
    .unwrap_or(Err(ProcessError::Timeout));
    if result.is_err() {
        let _ = child.kill().await;
        let _ = child.wait().await;
    }
    result
}

#[cfg(windows)]
fn windows_code() -> Result<Command, ProcessError> {
    use std::path::PathBuf;
    let mut roots = Vec::new();
    for key in ["LOCALAPPDATA", "ProgramFiles", "ProgramFiles(x86)"] {
        if let Some(base) = std::env::var_os(key) {
            let base = PathBuf::from(base);
            roots.push(if key == "LOCALAPPDATA" {
                base.join("Programs/Microsoft VS Code")
            } else {
                base.join("Microsoft VS Code")
            });
        }
    }
    if let Some(path) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path) {
            if dir
                .file_name()
                .is_some_and(|n| n.eq_ignore_ascii_case("bin"))
            {
                if let Some(parent) = dir.parent() {
                    roots.push(parent.to_path_buf())
                }
            }
        }
    }
    for root in roots {
        let exe = root.join("Code.exe");
        let cli = root.join("resources/app/out/cli.js");
        if exe.is_file() && cli.is_file() {
            let mut cmd = Command::new(exe);
            cmd.arg(cli)
                .arg("--list-extensions")
                .env("ELECTRON_RUN_AS_NODE", "1")
                .creation_flags(0x08000000);
            return Ok(cmd);
        }
    }
    Err(ProcessError::NotFound)
}

impl crate::install::InstallRunner for SystemRunner {
    async fn run_install(
        &self,
        invocation: crate::install::InstallInvocation,
    ) -> Result<(), crate::install::InstallError> {
        use crate::install::{InstallError, InstallInvocation};
        if !cfg!(all(target_os = "windows", target_arch = "x86_64")) {
            return Err(InstallError::ManagerUnavailable);
        }
        let (program, args) = invocation.command();
        #[cfg(windows)]
        let executable = std::env::var_os("LOCALAPPDATA")
            .map(std::path::PathBuf::from)
            .map(|p| p.join("Microsoft/WindowsApps/winget.exe"))
            .filter(|p| p.is_file())
            .ok_or(InstallError::ManagerUnavailable)?;
        #[cfg(windows)]
        let mut command = Command::new(executable);
        #[cfg(not(windows))]
        let mut command = Command::new(program);
        #[cfg(windows)]
        let _ = program;
        command.args(args);
        #[cfg(windows)]
        command.creation_flags(0x08000000);
        let timeout = if invocation == InstallInvocation::WingetVersion {
            Duration::from_secs(5)
        } else {
            Duration::from_secs(300)
        };
        execute(command, timeout, 1024 * 1024)
            .await
            .map(|_| ())
            .map_err(|e| match e {
                ProcessError::NotFound => InstallError::ManagerUnavailable,
                ProcessError::Timeout => InstallError::Timeout,
                ProcessError::OutputLimit => InstallError::OutputLimit,
                ProcessError::Failed => InstallError::Failed,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn registry_is_static() {
        assert_eq!(Invocation::Py3.command(), ("py", &["-3", "--version"][..]));
    }
    #[test]
    fn helper_process() {
        match std::env::var("PREFLIGHT_TEST_HELPER").as_deref() {
            Ok("sleep") => std::thread::sleep(Duration::from_secs(10)),
            Ok("flood") => {
                use std::io::Write;
                let _ = std::io::stdout().write_all(&vec![b'x'; 100_000]);
            }
            _ => {}
        }
    }
    fn helper(mode: &str) -> Command {
        let mut c = Command::new(std::env::current_exe().unwrap());
        c.args(["--exact", "process::tests::helper_process", "--nocapture"])
            .env("PREFLIGHT_TEST_HELPER", mode);
        c
    }
    #[tokio::test]
    async fn enforces_timeout() {
        let start = std::time::Instant::now();
        assert_eq!(
            execute(helper("sleep"), Duration::from_millis(150), 1024)
                .await
                .unwrap_err(),
            ProcessError::Timeout
        );
        assert!(start.elapsed() < Duration::from_secs(3));
    }
    #[tokio::test]
    async fn enforces_output_limit() {
        assert_eq!(
            execute(helper("flood"), Duration::from_secs(3), 1024)
                .await
                .unwrap_err(),
            ProcessError::OutputLimit
        );
    }
}
