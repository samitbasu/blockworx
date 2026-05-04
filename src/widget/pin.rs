#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PinSide {
    East,
    West,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BoxKind {
    Normal,
    Port,
}

#[derive(Clone)]
pub struct Pin {
    pub text: String,
    pub side: PinSide,
    pub offset: f32,
}
