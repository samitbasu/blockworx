//! This module defines the on-disk representation of a BW document.
//! It also contains the KDL parser.  The data structure used here
//! is not meant to hold all state used internally by the tool.
use hexcolor::HexColor;
use serde::{Deserialize, Serialize};
pub mod parse_kdl;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// The name of the document.  Doesn't have to match the file name.
    pub name: String,
    /// The top level block.
    pub top: Block,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub name: String,
    pub pins: Vec<Pin>,
    pub definition: Definition,
    pub visual: Rect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rect {
    pub x: i64,
    pub y: i64,
    pub width: i64,
    pub height: i64,
    pub color: Option<HexColor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Definition {
    pub blocks: Vec<Block>,
    pub nets: Vec<Net>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Side {
    North,
    South,
    East,
    West,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pin {
    pub name: String,
    pub side: Side,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Net {
    pub name: String,
    pub from: String,
    pub to: String,
    pub route: Option<Route>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point {
    pub x: i64,
    pub y: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    pub points: Vec<Point>,
    pub labels: Vec<Label>,
    pub color: Option<HexColor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    pub text: String,
    pub linear_distance: i64,
}
