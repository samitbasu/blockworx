#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum ToolName {
    Move,
    NewBlock,
    NewPin,
}

impl std::fmt::Display for ToolName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolName::Move => write!(f, "Move"),
            ToolName::NewBlock => write!(f, "New Block"),
            ToolName::NewPin => write!(f, "New Pin"),
        }
    }
}

pub const TOOL_NAMES: &[ToolName] = &[ToolName::Move, ToolName::NewBlock, ToolName::NewPin];
