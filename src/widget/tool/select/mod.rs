pub mod rename_pin;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum SubtoolState {
    Idle,
    Active,
}
