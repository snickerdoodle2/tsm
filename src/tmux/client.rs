use std::{fmt::Display, process::Command, rc::Rc, string};

use crate::tmux::{Field, Fieldset, ParseError, Session};

#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("utf-8: {0}")]
    Utf8(#[from] string::FromUtf8Error),
    #[error("tmux server is not running")]
    NoServer,
    #[error("tmux {0}: {1}")]
    Tmux(Subcommand, String),
    #[error("parse: {0}")]
    Parse(#[from] ParseError),
}

#[derive(Debug, Clone, Copy)]
pub enum Subcommand {
    DisplayMessage,
    NewSession,
    RenameSession,
    KillSession,
    ListSessions,
    SwitchClient,
}

impl Display for Subcommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.arg())
    }
}

impl Subcommand {
    fn arg(&self) -> &'static str {
        use Subcommand::*;
        match self {
            DisplayMessage => "display-message",
            NewSession => "new-session",
            RenameSession => "rename-session",
            KillSession => "kill-session",
            ListSessions => "list-sessions",
            SwitchClient => "switch-client",
        }
    }

    fn format_arg(&self) -> &'static str {
        use Subcommand::*;
        match self {
            DisplayMessage => "-p",
            NewSession | ListSessions => "-F",
            RenameSession | KillSession | SwitchClient => unreachable!(),
        }
    }
}

fn run<'a, I>(
    subcommand: Subcommand,
    format: Option<&Fieldset>,
    args: I,
) -> Result<String, ClientError>
where
    I: IntoIterator<Item = &'a str>,
{
    let mut command = Command::new("tmux");
    command.arg(subcommand.arg());

    if let Some(format) = format {
        command.arg(subcommand.format_arg()).arg(format.format());
    }

    command.args(args);

    let output = command.output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("no server running") {
            return Err(ClientError::NoServer);
        }
        return Err(ClientError::Tmux(subcommand, stderr.trim().to_string()));
    }

    Ok(String::from_utf8(output.stdout)?)
}

#[derive(Default)]
pub struct Client {
    separator: Rc<str>,
}

impl Client {
    pub fn new(separator: Rc<str>) -> Self {
        Self { separator }
    }

    pub fn current_session(&self) -> Result<usize, ClientError> {
        let fieldset = Fieldset::new(Box::new([Field::ID]), self.separator.clone());

        let data = run(Subcommand::DisplayMessage, Some(&fieldset), [])?;
        dbg!(&data);
        let id = fieldset
            .parse_session(data.trim())?
            .id
            .ok_or(ParseError::Missing("id"))?;

        Ok(id)
    }

    pub fn create_session(&self, name: &str) -> Result<(), ClientError> {
        // TODO: starting directory
        let _ = run(
            Subcommand::NewSession,
            None,
            [
                "-d", // do not attach
                "-s", name,
            ],
        )?;

        Ok(())
    }

    pub fn rename_session(&self, session: &mut Session, name: &str) -> Result<(), ClientError> {
        let _ = run(Subcommand::RenameSession, None, ["-t", &session.name, name])?;

        session.name.replace_range(.., name);
        Ok(())
    }

    pub fn delete_session(&self, session: Session) -> Result<(), ClientError> {
        let _ = run(Subcommand::KillSession, None, ["-t", &session.name])?;
        Ok(())
    }

    pub fn list_sessions(&self) -> Result<Vec<Session>, ClientError> {
        let fieldset = Fieldset::new_separator(self.separator.clone());
        let data = run(Subcommand::ListSessions, Some(&fieldset), [])?;

        Ok(data
            .lines()
            .filter_map(|s| parse_line(s, &fieldset))
            .collect())
    }

    pub fn select_session(&self, session: &Session) -> Result<(), ClientError> {
        let _ = run(Subcommand::SwitchClient, None, ["-t", &session.name])?;

        Ok(())
    }
}

fn parse_line(line: &str, fieldset: &Fieldset) -> Option<Session> {
    fieldset.parse_session(line).ok()?.build().ok()
}

#[cfg(test)]
mod tests {
    // TODO: tests, idk how to do it yet tho
}
