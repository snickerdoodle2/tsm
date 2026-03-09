use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};

use crate::tmux;

#[derive(Default)]
pub struct Sessions {
    sessions: Option<Vec<tmux::Session>>,
    filtered: Vec<usize>,
    current_session: Option<usize>,
    matcher: SkimMatcherV2,
}

impl Sessions {
    pub fn set(&mut self, sessions: Option<Vec<tmux::Session>>, pattern: &str) {
        // TODO: probably should migrate current session
        self.sessions = sessions;
        self.update_filter(pattern);
    }

    /// Returns sessions after filtering
    pub fn sessions(&self) -> Option<impl Iterator<Item = &tmux::Session>> {
        let sessions = self.sessions.as_ref()?;
        Some(self.filtered.iter().filter_map(move |&i| sessions.get(i)))
    }

    pub fn current(&self) -> Option<&tmux::Session> {
        self.sessions
            .as_ref()
            .and_then(|s| s.get(self.current_session?))
    }

    pub fn current_mut(&mut self) -> Option<&mut tmux::Session> {
        self.sessions
            .as_mut()
            .and_then(|s| s.get_mut(self.current_session?))
    }

    pub fn current_idx(&self) -> Option<usize> {
        self.current_session
    }

    pub fn update_filter(&mut self, pattern: &str) {
        let Some(sessions) = self.sessions.as_ref() else {
            return;
        };

        let mut indexes: Vec<_> = sessions
            .iter()
            .enumerate()
            .filter_map(|(i, s)| {
                self.matcher
                    .fuzzy_match(s.name(), pattern)
                    .map(|score| (i, score))
            })
            .collect();

        indexes.sort_by_key(|(_, score)| *score);

        self.filtered = indexes.into_iter().map(|(i, _)| i).collect();

        self.current_session = match self.current_session {
            None => self.filtered.get(0).copied(),
            Some(x) if !self.filtered.contains(&x) => self.filtered.get(0).copied(),
            Some(x) => Some(x),
        };
    }

    // TODO: support repeat
    pub fn cycle_next(&mut self) {
        let Some(cur) = self.current_session else {
            self.current_session = self.filtered.get(0).copied();
            return;
        };

        let Some(pos) = self.filtered.iter().position(|&x| x == cur) else {
            self.current_session = self.filtered.get(0).copied();
            return;
        };

        self.current_session = self
            .filtered
            .get((pos + 1).min(self.filtered.len()))
            .copied();
    }

    // TODO: support repeat
    pub fn cycle_prev(&mut self) {
        let Some(cur) = self.current_session else {
            self.current_session = self.filtered.last().copied();
            return;
        };

        let Some(pos) = self.filtered.iter().position(|&x| x == cur) else {
            self.current_session = self.filtered.get(0).copied();
            return;
        };

        self.current_session = self.filtered.get(pos.saturating_sub(1)).copied();
    }
}
