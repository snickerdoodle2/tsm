use nom::{
    IResult, Parser, branch,
    bytes::complete::{tag, take_until1},
    character::complete,
    combinator::opt,
    multi::separated_list0,
    sequence::{delimited, separated_pair},
};

use anyhow::{Result, anyhow};

#[derive(Debug, Clone)]
enum LayoutType {
    Horizontal(Vec<Layout>),
    Vertical(Vec<Layout>),
    Pane(usize),
}

#[derive(Debug, Clone)]
pub struct Layout {
    pub(in crate::tmux) width: u16,
    pub(in crate::tmux) height: u16,
    pub(in crate::tmux) offset_x: u16,
    pub(in crate::tmux) offset_y: u16,
    pub(in crate::tmux) layout_type: LayoutType,
}

fn parse_checksum(input: &str) -> IResult<&str, u32> {
    fn from_hex(input: &str) -> Result<u32, std::num::ParseIntError> {
        u32::from_str_radix(input, 16)
    }

    let (input, res) = take_until1(",").map_res(from_hex).parse(input)?;
    let (input, _) = tag(",").parse(input)?;
    Ok((input, res))
}

fn parse_dimensions(input: &str) -> IResult<&str, (u16, u16)> {
    let (input, res) = separated_pair(complete::u16, tag("x"), complete::u16).parse(input)?;
    let (input, _) = tag(",").parse(input)?;
    Ok((input, res))
}

fn parse_offset(input: &str, terminated: bool) -> IResult<&str, u16> {
    let (input, res) = complete::u16(input)?;

    let input = if terminated {
        let (input, _) = tag(",").parse(input)?;
        input
    } else {
        let (input, _) = opt(tag(",")).parse(input)?;
        input
    };

    Ok((input, res))
}

fn parse_pane_id(input: &str) -> IResult<&str, Option<usize>> {
    opt(complete::usize).parse(input)
}

fn parse_children(input: &str) -> IResult<&str, LayoutType> {
    let delim = input.chars().next();

    let (input, res) = branch::alt((
        delimited(
            tag("["),
            separated_list0(tag(","), parse_layout_inner),
            tag("]"),
        ),
        delimited(
            tag("{"),
            separated_list0(tag(","), parse_layout_inner),
            tag("}"),
        ),
    ))
    .parse(input)?;

    let res = match delim.unwrap() {
        '[' => LayoutType::Vertical(res),
        '{' => LayoutType::Horizontal(res),
        _ => unreachable!(),
    };

    Ok((input, res))
}

fn parse_layout_inner(input: &str) -> IResult<&str, Layout> {
    let (input, (width, height)) = parse_dimensions(input)?;
    let (input, offset_x) = parse_offset(input, true)?;
    let (input, offset_y) = parse_offset(input, false)?;
    let (input, pane_id) = parse_pane_id(input)?;

    if let Some(pane_id) = pane_id {
        return Ok((
            input,
            Layout {
                width,
                height,
                offset_x,
                offset_y,
                layout_type: LayoutType::Pane(pane_id),
            },
        ));
    }

    let (input, layout_type) = parse_children(input)?;

    Ok((
        input,
        Layout {
            width,
            height,
            offset_x,
            offset_y,
            layout_type,
        },
    ))
}

pub fn parse_layout(input: &str) -> Result<Layout> {
    let (input, _) = parse_checksum(input).map_err(|_| anyhow!("checksum"))?;
    parse_layout_inner(input)
        .map(|(_, l)| l)
        .map_err(|_| anyhow!("layout"))
}

#[cfg(test)]
mod tests {
    use claim::{assert_matches, assert_ok, assert_some};

    use super::*;

    #[test]
    fn parses_simple_layout() {
        let data = "6230,168x64,0,0,11";

        let layout = assert_ok!(parse_layout(data));

        assert_eq!(layout.width, 168);
        assert_eq!(layout.height, 64);
        assert_eq!(layout.offset_x, 0);
        assert_eq!(layout.offset_y, 0);
        assert_matches!(layout.layout_type, LayoutType::Pane(11));
    }

    #[test]
    fn parses_complicated_layout() {
        // ecbe,168x64,0,0 {
        //   84x64,0,0,11,
        //   83x64,85,0 [
        //     83x32,85,0,12,
        //     83x31,85,33 {
        //       41x31,85,33,13,
        //       41x31,127,33,14
        //     }
        //   ]
        // }
        let data = "ecbe,168x64,0,0{84x64,0,0,11,83x64,85,0[83x32,85,0,12,83x31,85,33{41x31,85,33,13,41x31,127,33,14}]}";

        let layout = assert_ok!(parse_layout(data));

        assert_eq!(layout.width, 168);
        assert_eq!(layout.height, 64);
        assert_eq!(layout.offset_x, 0);
        assert_eq!(layout.offset_y, 0);

        let LayoutType::Horizontal(children) = layout.layout_type else {
            panic!("Expected horizontal, got {:?}", layout.layout_type);
        };

        let lhs = assert_some!(children.first().cloned());
        assert_eq!(lhs.width, 84);
        assert_eq!(lhs.height, 64);
        assert_eq!(lhs.offset_x, 0);
        assert_eq!(lhs.offset_y, 0);
        assert_matches!(&lhs.layout_type, LayoutType::Pane(11));

        let rhs = assert_some!(children.get(1).cloned());
        assert_eq!(rhs.width, 83);
        assert_eq!(rhs.height, 64);
        assert_eq!(rhs.offset_x, 85);
        assert_eq!(rhs.offset_y, 0);

        let LayoutType::Vertical(children) = rhs.layout_type else {
            panic!("Expected vertical, got {:?}", rhs.layout_type);
        };

        let lhs = assert_some!(children.first().cloned());
        assert_eq!(lhs.width, 83);
        assert_eq!(lhs.height, 32);
        assert_eq!(lhs.offset_x, 85);
        assert_eq!(lhs.offset_y, 0);
        assert_matches!(&lhs.layout_type, LayoutType::Pane(12));

        let rhs = assert_some!(children.get(1).cloned());
        assert_eq!(rhs.width, 83);
        assert_eq!(rhs.height, 31);
        assert_eq!(rhs.offset_x, 85);
        assert_eq!(rhs.offset_y, 33);

        let LayoutType::Horizontal(children) = rhs.layout_type else {
            panic!("Expected horizontal, got {:?}", rhs.layout_type);
        };

        let lhs = assert_some!(children.first().cloned());
        assert_eq!(lhs.width, 41);
        assert_eq!(lhs.height, 31);
        assert_eq!(lhs.offset_x, 85);
        assert_eq!(lhs.offset_y, 33);
        assert_matches!(&lhs.layout_type, LayoutType::Pane(13));

        let rhs = assert_some!(children.get(1).cloned());
        assert_eq!(rhs.width, 41);
        assert_eq!(rhs.height, 31);
        assert_eq!(rhs.offset_x, 127);
        assert_eq!(rhs.offset_y, 33);
        assert_matches!(&rhs.layout_type, LayoutType::Pane(14));
    }
}
