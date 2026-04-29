use ::kdl::{KdlDocument, KdlNode};
use hexcolor::HexColor;
use miette::{NamedSource, SourceSpan};

use crate::document::*;

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum KdlParseError {
    #[error("missing required property '{prop}' on '{node}' node")]
    MissingProp {
        prop: &'static str,
        node: &'static str,
        #[label("here")]
        span: SourceSpan,
        #[source_code]
        src: NamedSource<String>,
    },

    #[error("wrong value type for property '{prop}' on '{node}' node")]
    WrongType {
        prop: &'static str,
        node: &'static str,
        #[label("here")]
        span: SourceSpan,
        #[source_code]
        src: NamedSource<String>,
    },

    #[error("missing required argument at position {index} on '{node}' node")]
    MissingArg {
        index: usize,
        node: &'static str,
        #[label("here")]
        span: SourceSpan,
        #[source_code]
        src: NamedSource<String>,
    },

    #[error("invalid side '{value}'; expected north/south/east/west (or top/bottom/left/right)")]
    InvalidSide {
        value: String,
        #[label("invalid side")]
        span: SourceSpan,
        #[source_code]
        src: NamedSource<String>,
    },

    #[error("expected a top-level 'document' node")]
    NoDocumentNode,

    #[error("expected a 'block' child inside 'document' node")]
    NoBlockNode {
        #[label("here")]
        span: SourceSpan,
        #[source_code]
        src: NamedSource<String>,
    },

    #[error("invalid color '{value}' on '{node}' node: {reason}")]
    InvalidColor {
        value: String,
        node: &'static str,
        reason: &'static str,
        #[label("here")]
        span: SourceSpan,
        #[source_code]
        src: NamedSource<String>,
    },
}

fn named_source(src: &str) -> NamedSource<String> {
    NamedSource::new("input.kdl", src.to_string())
}

fn get_required_i64(
    node: &KdlNode,
    key: &'static str,
    node_name: &'static str,
    src: &str,
) -> miette::Result<i64> {
    let entry = node.entry(key).ok_or_else(|| KdlParseError::MissingProp {
        prop: key,
        node: node_name,
        span: node.span(),
        src: named_source(src),
    })?;
    entry
        .value()
        .as_integer()
        .and_then(|n| i64::try_from(n).ok())
        .ok_or_else(|| {
            KdlParseError::WrongType {
                prop: key,
                node: node_name,
                span: entry.span(),
                src: named_source(src),
            }
            .into()
        })
}

fn get_required_str<'a>(
    node: &'a KdlNode,
    key: &'static str,
    node_name: &'static str,
    src: &str,
) -> miette::Result<&'a str> {
    let entry = node.entry(key).ok_or_else(|| KdlParseError::MissingProp {
        prop: key,
        node: node_name,
        span: node.span(),
        src: named_source(src),
    })?;
    entry.value().as_string().ok_or_else(|| {
        KdlParseError::WrongType {
            prop: key,
            node: node_name,
            span: entry.span(),
            src: named_source(src),
        }
        .into()
    })
}

fn get_required_arg_str<'a>(
    node: &'a KdlNode,
    index: usize,
    node_name: &'static str,
    src: &str,
) -> miette::Result<&'a str> {
    let entry = node.entry(index).ok_or_else(|| KdlParseError::MissingArg {
        index,
        node: node_name,
        span: node.span(),
        src: named_source(src),
    })?;
    entry.value().as_string().ok_or_else(|| {
        KdlParseError::WrongType {
            prop: "positional",
            node: node_name,
            span: entry.span(),
            src: named_source(src),
        }
        .into()
    })
}

fn get_opt_color(
    node: &KdlNode,
    node_name: &'static str,
    src: &str,
) -> miette::Result<Option<HexColor>> {
    node.entry("color")
        .map(|entry| {
            let s = entry.value().as_string().ok_or_else(|| KdlParseError::WrongType {
                prop: "color",
                node: node_name,
                span: entry.span(),
                src: named_source(src),
            })?;
            s.parse::<HexColor>().map_err(|reason| {
                KdlParseError::InvalidColor {
                    value: s.to_string(),
                    node: node_name,
                    reason,
                    span: entry.span(),
                    src: named_source(src),
                }
                .into()
            })
        })
        .transpose()
}

pub fn parse_document(src: &str) -> miette::Result<Document> {
    let kdl_doc: KdlDocument = src.parse()?;

    let doc_node = kdl_doc
        .nodes()
        .iter()
        .find(|n| n.name().value() == "document")
        .ok_or(KdlParseError::NoDocumentNode)?;

    let name = get_required_arg_str(doc_node, 0, "document", src)?.to_string();

    let block_node = doc_node
        .children()
        .and_then(|c| c.get("block"))
        .ok_or_else(|| KdlParseError::NoBlockNode {
            span: doc_node.span(),
            src: named_source(src),
        })?;

    let top = parse_block(block_node, src)?;
    Ok(Document { name, top })
}

fn parse_block(node: &KdlNode, src: &str) -> miette::Result<Block> {
    let name = get_required_arg_str(node, 0, "block", src)?.to_string();

    let pins = node
        .children()
        .and_then(|c| c.get("pins"))
        .map(|n| parse_pins(n, src))
        .transpose()?
        .unwrap_or_default();

    let visual_node = node
        .children()
        .and_then(|c| c.get("visual"))
        .ok_or_else(|| KdlParseError::MissingProp {
            prop: "visual",
            node: "block",
            span: node.span(),
            src: named_source(src),
        })?;
    let visual = parse_visual(visual_node, src)?;

    let definition = node
        .children()
        .and_then(|c| c.get("definition"))
        .map(|n| parse_definition(n, src))
        .transpose()?
        .unwrap_or(Definition {
            blocks: vec![],
            nets: vec![],
        });

    Ok(Block {
        name,
        pins,
        definition,
        visual,
    })
}

fn parse_pins(node: &KdlNode, src: &str) -> miette::Result<Vec<Pin>> {
    node.children()
        .map(|doc| doc.nodes())
        .unwrap_or_default()
        .iter()
        .filter(|n| n.name().value() == "pin")
        .map(|n| parse_pin(n, src))
        .collect()
}

fn parse_pin(node: &KdlNode, src: &str) -> miette::Result<Pin> {
    let name = get_required_arg_str(node, 0, "pin", src)?.to_string();
    let side_entry = node
        .entry("side")
        .ok_or_else(|| KdlParseError::MissingProp {
            prop: "side",
            node: "pin",
            span: node.span(),
            src: named_source(src),
        })?;
    let side_str = side_entry
        .value()
        .as_string()
        .ok_or_else(|| KdlParseError::WrongType {
            prop: "side",
            node: "pin",
            span: side_entry.span(),
            src: named_source(src),
        })?;
    let side = parse_side(side_str, side_entry.span(), src)?;
    let offset = get_required_i64(node, "at", "pin", src)?;
    Ok(Pin { name, side, offset })
}

fn parse_side(value: &str, span: SourceSpan, src: &str) -> miette::Result<Side> {
    match value {
        "north" | "top" => Ok(Side::North),
        "south" | "bottom" => Ok(Side::South),
        "east" | "right" => Ok(Side::East),
        "west" | "left" => Ok(Side::West),
        other => Err(KdlParseError::InvalidSide {
            value: other.to_string(),
            span,
            src: named_source(src),
        }
        .into()),
    }
}

fn parse_visual(node: &KdlNode, src: &str) -> miette::Result<Rect> {
    let x = get_required_i64(node, "x", "visual", src)?;
    let y = get_required_i64(node, "y", "visual", src)?;
    let width = get_required_i64(node, "w", "visual", src)?;
    let height = get_required_i64(node, "h", "visual", src)?;
    let color = get_opt_color(node, "visual", src)?;
    Ok(Rect {
        x,
        y,
        width,
        height,
        color,
    })
}

fn parse_definition(node: &KdlNode, src: &str) -> miette::Result<Definition> {
    let children = node.children().map(|c| c.nodes()).unwrap_or_default();

    let blocks = children
        .iter()
        .filter(|n| n.name().value() == "block")
        .map(|n| parse_block(n, src))
        .collect::<miette::Result<Vec<_>>>()?;

    let nets = children
        .iter()
        .find(|n| n.name().value() == "nets")
        .map(|n| parse_nets_node(n, src))
        .transpose()?
        .unwrap_or_default();

    Ok(Definition { blocks, nets })
}

fn parse_nets_node(node: &KdlNode, src: &str) -> miette::Result<Vec<Net>> {
    node.children()
        .map(|doc| doc.nodes())
        .unwrap_or_default()
        .iter()
        .filter(|n| n.name().value() == "net")
        .map(|n| parse_net(n, src))
        .collect()
}

fn parse_net(node: &KdlNode, src: &str) -> miette::Result<Net> {
    let name = get_required_arg_str(node, 0, "net", src)?.to_string();
    let from = get_required_str(node, "from", "net", src)?.to_string();
    let to = get_required_str(node, "to", "net", src)?.to_string();
    let route = node
        .children()
        .and_then(|c| c.get("route"))
        .map(|n| parse_route(n, src))
        .transpose()?;
    Ok(Net {
        name,
        from,
        to,
        route,
    })
}

fn parse_route(node: &KdlNode, src: &str) -> miette::Result<Route> {
    let color = get_opt_color(node, "route", src)?;
    let children = node.children().map(|c| c.nodes()).unwrap_or_default();
    let points = children
        .iter()
        .filter(|n| n.name().value() == "pt")
        .map(|n| parse_point(n, src))
        .collect::<miette::Result<Vec<_>>>()?;
    let labels = children
        .iter()
        .filter(|n| n.name().value() == "label")
        .map(|n| parse_label(n, src))
        .collect::<miette::Result<Vec<_>>>()?;
    Ok(Route {
        points,
        labels,
        color,
    })
}

fn parse_point(node: &KdlNode, src: &str) -> miette::Result<Point> {
    let x = get_required_i64(node, "x", "pt", src)?;
    let y = get_required_i64(node, "y", "pt", src)?;
    Ok(Point { x, y })
}

fn parse_label(node: &KdlNode, src: &str) -> miette::Result<Label> {
    let text = get_required_arg_str(node, 0, "label", src)?.to_string();
    let linear_distance = get_required_i64(node, "at", "label", src)?;
    Ok(Label {
        text,
        linear_distance,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_sample() {
        let src = include_str!("../../sample.kdl");
        let doc = parse_document(src).expect("parse failed");
        assert_eq!(doc.name, "MyDesign");
        assert_eq!(doc.top.name, "A");
        assert_eq!(doc.top.pins.len(), 2);
        assert_eq!(doc.top.visual.color, Some(HexColor::new(0xcc, 0xcc, 0xee)));
        assert_eq!(doc.top.definition.blocks.len(), 2);
        assert_eq!(doc.top.definition.nets.len(), 2);
        let n1 = doc
            .top
            .definition
            .nets
            .iter()
            .find(|n| n.name == "n1")
            .unwrap();
        let route = n1.route.as_ref().unwrap();
        assert_eq!(route.points.len(), 2);
        assert_eq!(route.color, Some(HexColor::new(0x00, 0xff, 0x00)));
    }
}
