#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum ToolName {
    Move,
    NewBlock,
    NewPin,
    MovePin,
    RenamePin,
}

impl std::fmt::Display for ToolName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolName::Move => write!(f, "Move"),
            ToolName::NewBlock => write!(f, "New Block"),
            ToolName::NewPin => write!(f, "New Pin"),
            ToolName::MovePin => write!(f, "Move Pin"),
            ToolName::RenamePin => write!(f, "Rename Pin"),
        }
    }
}

pub const TOOLBAR_TOOLS: &[ToolName] = &[
    ToolName::Move,
    ToolName::NewBlock,
    ToolName::NewPin,
    ToolName::MovePin,
    ToolName::RenamePin,
];
