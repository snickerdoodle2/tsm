use std::{cmp::Reverse, mem};

use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};

use crate::tmux;

#[derive(Default)]
pub struct Sessions {
    sessions: Option<Vec<tmux::Session>>,
    filtered: Vec<usize>,

    current_session: Option<usize>,
    current_filtered: Option<usize>,

    created_cell: Option<Box<str>>,
    deleted_cell: Option<usize>,

    matcher: SkimMatcherV2,
}

impl Sessions {
    pub fn set(&mut self, sessions: Option<Vec<tmux::Session>>, pattern: &str) {
        let prev_id = self
            .sessions
            .as_ref()
            .zip(self.current_session)
            .and_then(|(s, i)| s.get(i))
            .map(tmux::Session::id);

        self.sessions = sessions;
        self.update_filter(pattern);
        self.restore_current(prev_id);
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
        self.current_filtered
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

        indexes.sort_by_key(|(_, score)| Reverse(*score));

        self.filtered = indexes.into_iter().map(|(i, _)| i).collect();

        self.current_filtered = Some(0);
        self.current_session = self.filtered.first().copied();
    }

    // TODO: support repeat
    pub fn cycle_next(&mut self) {
        self.current_filtered = Some(
            self.current_filtered
                .map(|x| x.saturating_add(1).min(self.filtered.len() - 1))
                .unwrap_or_default(),
        );

        self.update_current();
    }

    // TODO: support repeat
    pub fn cycle_prev(&mut self) {
        self.current_filtered = Some(
            self.current_filtered
                .map(|x| x.saturating_sub(1))
                .unwrap_or_else(|| self.filtered.len() - 1),
        );

        self.update_current();
    }

    pub fn set_created(&mut self, name: &str) {
        self.created_cell = Some(name.into());
    }

    pub fn set_deleted(&mut self) {
        self.deleted_cell = self.current_filtered;
    }

    fn update_current(&mut self) {
        self.current_session = self
            .current_filtered
            .and_then(|i| self.filtered.get(i).copied());
    }

    // FIXME: wtf is this xdd
    fn restore_current(&mut self, prev_id: Option<usize>) {
        if let Some(deleted) = mem::take(&mut self.deleted_cell)
            && deleted < self.filtered.len()
        {
            self.current_filtered = Some(deleted);
            self.update_current();
            return;
        }

        if let Some(created) = mem::take(&mut self.created_cell)
            && let Some(sessions) = self.sessions.as_ref()
            && let Some(idx) = sessions.iter().position(|s| s.name() == created.as_ref())
            && let Some(filtered_idx) = self.filtered.iter().position(|&i| i == idx)
        {
            self.current_session = Some(idx);
            self.current_filtered = Some(filtered_idx);
            return;
        }

        if let Some(prev_id) = prev_id
            && let Some(sessions) = self.sessions.as_ref()
            && let Some(idx) = sessions.iter().position(|s| s.id() == prev_id)
            && let Some(filtered_idx) = self.filtered.iter().position(|&i| i == idx)
        {
            self.current_session = Some(idx);
            self.current_filtered = Some(filtered_idx);
            return;
        }

        self.current_session = self.filtered.first().copied();
        self.current_filtered = Some(0);
    }
}
