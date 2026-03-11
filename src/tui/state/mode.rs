#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Normal,
    Details,
    #[cfg(feature = "debug")]
    Debug,
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
            #[cfg(feature = "debug")]
            Mode::Debug => ModeType::Normal,
        }
    }

    pub fn is_modal(&self) -> bool {
        match self {
            Mode::Normal | Mode::Details | Mode::Search => false,
            Mode::Rename | Mode::Create | Mode::Delete => true,
            #[cfg(feature = "debug")]
            Mode::Debug => false,
        }
    }
}

pub enum ModeType {
    Normal,
    Input,
    Confirm,
}
