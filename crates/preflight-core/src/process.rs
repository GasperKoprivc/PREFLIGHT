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
        if std::env::var_os("PATH").is_none_or(|p| p.is_empty()) {
            return Err(ProcessError::NotFound);
        }
        let (program, args) = invocation.command();
        let mut command = Command::new(program);
        command.args(args);
        #[cfg(windows)]
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW
        execute(command, Duration::from_secs(5), 32 * 1024).await
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
