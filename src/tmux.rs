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

#[derive(Debug, Clone)]
pub struct TmuxSession {
    id: usize,
    name: String,
    created: DateTime<Utc>,
    attached: u8,
}

impl TmuxSession {
    pub fn id(&self) -> usize {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn created(&self) -> &DateTime<Utc> {
        &self.created
    }

    pub fn attached(&self) -> u8 {
        self.attached
    }

    pub fn list() -> Result<Vec<Self>, TmuxError> {
        let output = Command::new("tmux")
            .arg("list-sessions")
            .arg("-F")
            .arg("#{session_id};#{session_name};#{session_created};#{session_attached}")
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

    pub fn create(name: &str) -> Result<(), TmuxError> {
        // TODO: starting directory
        let output = Command::new("tmux")
            .arg("new-session")
            .arg("-d")
            .arg("-s")
            .arg(name)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(TmuxError::TmuxError(format!(
                "tmux new-session: {}",
                stderr.trim()
            )))
        } else {
            Ok(())
        }
    }

    pub fn rename(&mut self, new_name: &str) -> Result<(), TmuxError> {
        let output = Command::new("tmux")
            .arg("rename-session")
            .arg("-t")
            .arg(&self.name)
            .arg(new_name)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(TmuxError::TmuxError(format!(
                "tmux rename-session: {}",
                stderr.trim()
            )))
        } else {
            self.name.replace_range(.., new_name);
            Ok(())
        }
    }

    pub fn select(&self) -> Result<(), TmuxError> {
        let output = Command::new("tmux")
            .arg("switch-client")
            .arg("-t")
            .arg(&self.name)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(TmuxError::TmuxError(format!(
                "tmux switch-client: {}",
                stderr.trim()
            )))
        } else {
            Ok(())
        }
    }

    pub fn delete(&self) -> Result<(), TmuxError> {
        let output = Command::new("tmux")
            .arg("kill-session")
            .arg("-t")
            .arg(&self.name)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(TmuxError::TmuxError(format!(
                "tmux kill-session: {}",
                stderr.trim()
            )))
        } else {
            Ok(())
        }
    }

    fn from_line(line: &str) -> Option<Self> {
        let mut line = line.split(";");
        let id = (&line.next()?[1..]).parse().ok()?;
        let name = line.next()?.into();
        let created_timestamp = line.next()?.parse::<i64>().ok()?;
        let attached = line.next()?.parse::<u8>().ok()?;

        Some(Self {
            id,
            name,
            created: DateTime::from_timestamp_secs(created_timestamp)?,
            attached,
        })
    }
}
