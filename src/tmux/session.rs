use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Default)]
pub struct Session {
    pub(in crate::tmux) id: usize,
    pub(in crate::tmux) name: String,
    pub(in crate::tmux) created: DateTime<Utc>,
    pub(in crate::tmux) last_activity: DateTime<Utc>,
    pub(in crate::tmux) attached: u8,
}

impl Session {
    pub fn id(&self) -> usize {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn attached(&self) -> u8 {
        self.attached
    }

    pub fn created(&self) -> String {
        chrono_humanize::HumanTime::from(self.created).to_string()
    }

    pub fn last_activity(&self) -> String {
        chrono_humanize::HumanTime::from(self.last_activity).to_string()
    }
}
