use std::process::Command;

use chrono::{DateTime, Utc};

#[derive(thiserror::Error, Debug)]
pub enum TmuxError {
    #[error("tmux server is not running")]
    NoServer,
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    TmuxError(String),
}

#[derive(Debug)]
pub struct TmuxSession {
    name: String,
    created: DateTime<Utc>,
    attached: u8,
}

impl TmuxSession {
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl TmuxSession {
    fn from_line(line: &str) -> Option<Self> {
        let mut line = line.split(";");
        let name = line.next()?.into();
        let created_timestamp = line.next()?.parse::<i64>().ok()?;
        let attached = line.next()?.parse::<u8>().ok()?;

        Some(Self {
            name,
            created: DateTime::from_timestamp_secs(created_timestamp)?,
            attached,
        })
    }
}

pub fn list_sessions() -> Result<Vec<TmuxSession>, TmuxError> {
    let output = Command::new("tmux")
        .arg("list-sessions")
        .arg("-F")
        .arg("#{session_name};#{session_created};#{session_attached}")
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("no server running") {
            return Err(TmuxError::NoServer);
        }
        return Err(TmuxError::TmuxError(format!(
            "tmux list-sessions: {}",
            stderr.trim()
        )));
    }

    let stdout = String::from_utf8(output.stdout).unwrap();
    Ok(stdout.lines().filter_map(TmuxSession::from_line).collect())
}
