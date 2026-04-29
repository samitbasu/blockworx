#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LabelSide {
    East,
    West,
}

#[derive(Clone)]
pub struct Label {
    pub text: String,
    pub side: LabelSide,
    pub offset: f32,
}
