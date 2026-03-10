#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Normal,
    Details,
    Search,
    Rename,
    Create,
    Delete,
}

impl Mode {
    pub fn mode_type(&self) -> ModeType {
        match self {
            Mode::Normal | Mode::Details => ModeType::Normal,
            Mode::Search | Mode::Rename | Mode::Create => ModeType::Input,
            Mode::Delete => ModeType::Confirm,
        }
    }
}

pub enum ModeType {
    Normal,
    Input,
    Confirm,
}
