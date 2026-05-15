#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum ToolName {
    Move,
    NewBlock,
    NewPin,
    MovePin,
    RenamePin,
    Route,
    MoveBlock,
    ResizeBlock,
}

impl std::fmt::Display for ToolName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolName::Move => write!(f, "Move"),
            ToolName::NewBlock => write!(f, "New Block"),
            ToolName::NewPin => write!(f, "New Pin"),
            ToolName::MovePin => write!(f, "Move Pin"),
            ToolName::RenamePin => write!(f, "Rename Pin"),
            ToolName::Route => write!(f, "Route"),
            ToolName::MoveBlock => write!(f, "Move Block"),
            ToolName::ResizeBlock => write!(f, "Resize Block"),
        }
    }
}

pub const TOOLBAR_TOOLS: &[ToolName] = &[
    ToolName::Move,
    ToolName::NewBlock,
    ToolName::NewPin,
    ToolName::MovePin,
    ToolName::RenamePin,
    ToolName::Route,
    ToolName::MoveBlock,
    ToolName::ResizeBlock,
];
