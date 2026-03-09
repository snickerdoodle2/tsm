use std::rc::Rc;

use anyhow::anyhow;
use chrono::{DateTime, Utc};

use crate::tmux::session::Session;

pub enum Field {
    ID,
    Name,
    Created,
    Activity,
    Attached,
}

impl Field {
    pub fn format(&self) -> &'static str {
        match self {
            Field::ID => "session_id",
            Field::Name => "session_name",
            Field::Created => "session_created",
            Field::Activity => "session_activity",
            Field::Attached => "session_attached",
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("Failed to parse {0}: {1}")]
    Parsing(&'static str, anyhow::Error),
    #[error("{0} is defined more than once")]
    DuplicateField(&'static str),
    #[error("{0} is out of range")]
    OutOfRange(&'static str),
    #[error("{0} is missing")]
    Missing(&'static str),
}

impl Default for Fieldset {
    fn default() -> Self {
        Self {
            fields: Box::new([
                Field::ID,
                Field::Name,
                Field::Created,
                Field::Activity,
                Field::Attached,
            ]),
            separator: ";".into(),
        }
    }
}

#[derive(Default, Debug)]
pub struct SessionBuilder {
    pub id: Option<usize>,
    pub name: Option<String>,
    pub created: Option<DateTime<Utc>>,
    pub last_activity: Option<DateTime<Utc>>,
    pub attached: Option<u8>,
}

impl SessionBuilder {
    fn id(&mut self, data: &str) -> Result<(), ParseError> {
        const NAME: &str = "id";

        if self.id.is_some() {
            return Err(ParseError::DuplicateField(NAME));
        }

        if data.is_empty() {
            return Err(ParseError::Parsing(NAME, anyhow!("empty data")));
        }

        if &data[0..1] != "$" {
            return Err(ParseError::Parsing(
                NAME,
                anyhow!("expected id to start with $"),
            ));
        }

        self.id = Some(
            data[1..]
                .parse()
                .map_err(|e| ParseError::Parsing(NAME, anyhow::Error::new(e)))?,
        );

        Ok(())
    }

    fn name(&mut self, data: &str) -> Result<(), ParseError> {
        if self.name.is_some() {
            return Err(ParseError::DuplicateField("name"));
        }

        self.name = Some(data.to_string());

        Ok(())
    }

    fn created(&mut self, data: &str) -> Result<(), ParseError> {
        const NAME: &str = "created";

        if self.created.is_some() {
            return Err(ParseError::DuplicateField(NAME));
        }

        let timestamp: i64 = data
            .parse()
            .map_err(|e| ParseError::Parsing(NAME, anyhow::Error::new(e)))?;
        let created =
            DateTime::from_timestamp_secs(timestamp).ok_or(ParseError::OutOfRange(NAME))?;

        self.created = Some(created);

        Ok(())
    }

    fn last_activity(&mut self, data: &str) -> Result<(), ParseError> {
        const NAME: &str = "last activity";

        if self.last_activity.is_some() {
            return Err(ParseError::DuplicateField(NAME));
        }

        let timestamp: i64 = data
            .parse()
            .map_err(|e| ParseError::Parsing(NAME, anyhow::Error::new(e)))?;
        let last_activity =
            DateTime::from_timestamp_secs(timestamp).ok_or(ParseError::OutOfRange(NAME))?;

        self.last_activity = Some(last_activity);

        Ok(())
    }

    fn attached(&mut self, data: &str) -> Result<(), ParseError> {
        const NAME: &str = "attached";

        if self.attached.is_some() {
            return Err(ParseError::DuplicateField(NAME));
        }

        self.attached = Some(
            data.parse()
                .map_err(|e| ParseError::Parsing(NAME, anyhow::Error::new(e)))?,
        );

        Ok(())
    }

    pub fn build(self) -> Result<Session, ParseError> {
        Ok(Session {
            id: self.id.ok_or(ParseError::Missing("id"))?,
            name: self.name.ok_or(ParseError::Missing("name"))?,
            created: self.created.ok_or(ParseError::Missing("created"))?,
            last_activity: self
                .last_activity
                .ok_or(ParseError::Missing("last activity"))?,
            attached: self.attached.ok_or(ParseError::Missing("attached"))?,
        })
    }
}

pub struct Fieldset {
    fields: Box<[Field]>,
    separator: Rc<str>,
}

impl Fieldset {
    pub fn new(fields: Box<[Field]>, separator: Rc<str>) -> Self {
        Self { fields, separator }
    }

    pub fn new_separator(separator: Rc<str>) -> Self {
        Self {
            separator,
            ..Default::default()
        }
    }

    pub fn format(&self) -> String {
        let fields: Vec<_> = self
            .fields
            .iter()
            .map(Field::format)
            .map(|s| format!("#{{{s}}}"))
            .collect();

        fields.join(&self.separator)
    }

    pub fn parse_session(&self, line: &str) -> Result<SessionBuilder, ParseError> {
        let mut builder = SessionBuilder::default();

        for (data, field) in line.split(self.separator.as_ref()).zip(self.fields.iter()) {
            match field {
                Field::ID => builder.id(data)?,
                Field::Name => builder.name(data)?,
                Field::Created => builder.created(data)?,
                Field::Activity => builder.last_activity(data)?,
                Field::Attached => builder.attached(data)?,
            }
        }

        Ok(builder)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use claim::*;

    #[test]
    fn formats_default_string() {
        let fieldset = Fieldset::default();
        let expected = "#{session_id};#{session_name};#{session_created};#{session_activity};#{session_attached}";

        assert_eq!(fieldset.format(), expected);
    }

    #[test]
    fn formats_custom_separator() {
        let fieldset = Fieldset {
            separator: ":)".into(),
            ..Default::default()
        };

        let expected = "#{session_id}:)#{session_name}:)#{session_created}:)#{session_activity}:)#{session_attached}";

        assert_eq!(fieldset.format(), expected);
    }

    #[test]
    fn parses_valid_data() {
        let fieldset = Fieldset::default();
        let line = "$42;foo;2281580400;2281623600;67";

        let builder = assert_ok!(fieldset.parse_session(line));
        let session = assert_ok!(builder.build());

        assert_eq!(session.id, 42);
        assert_eq!(session.name, "foo");
        assert_eq!(
            session.created,
            Utc.with_ymd_and_hms(2042, 4, 20, 4, 20, 0).unwrap()
        );
        assert_eq!(
            session.last_activity,
            Utc.with_ymd_and_hms(2042, 4, 20, 16, 20, 0).unwrap()
        );
        assert_eq!(session.attached, 67);
    }

    #[test]
    fn fails_to_parse_duplicates() {
        let fieldset = Fieldset {
            fields: Box::new([
                Field::ID,
                Field::ID,
                Field::Name,
                Field::Created,
                Field::Activity,
                Field::Attached,
            ]),
            ..Default::default()
        };
        let line = "$42;$42;foo;2281580400;2281623600;67";

        let error = assert_err!(fieldset.parse_session(line));
        assert_matches!(error, ParseError::DuplicateField("id"));
    }

    #[test]
    fn fails_to_parse_invalid_fields() {
        let fieldset = Fieldset::default();
        let line = "42;foo;2281580400;2281623600;67";

        let error = assert_err!(fieldset.parse_session(line));
        assert_matches!(error, ParseError::Parsing("id", _));
    }

    #[test]
    fn fails_to_parse_missing_fields() {
        let fieldset = Fieldset {
            fields: Box::new([
                Field::Name,
                Field::Created,
                Field::Activity,
                Field::Attached,
            ]),
            ..Default::default()
        };
        let line = "foo;2281580400;2281623600;67";

        let builder = assert_ok!(fieldset.parse_session(line));
        let error = assert_err!(builder.build());
        assert_matches!(error, ParseError::Missing("id"));
    }
}
