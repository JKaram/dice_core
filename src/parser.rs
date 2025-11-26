use nom::bytes::complete::tag;
use nom::character::complete::{one_of, space0};
use nom::combinator::{map, opt};
use nom::error::context;
use nom::number::complete::double;
use nom::sequence::{pair, preceded};
use nom::{IResult, Parser, branch::alt};

pub struct DiceRequest {
    pub quantity: f64,
    pub sides: f64,
    pub modifier: f64,
}

fn parse_quantity(input: &str) -> IResult<&str, f64> {
    map(opt(double), |q| q.unwrap_or(1.0)).parse(input)
}

fn parse_sides(input: &str) -> IResult<&str, f64> {
    double(input)
}

fn parse_d(input: &str) -> IResult<&str, &str> {
    alt((tag("d"), tag("D"))).parse(input)
}

fn parse_modifier(input: &str) -> IResult<&str, f64> {
    map(pair(one_of("+-"), double), |(sign, val)| {
        if sign == '-' { -val } else { val }
    })
    .parse(input)
}

pub fn dice_result(expression: &str) -> IResult<&str, DiceRequest> {
    let (remaining, (quantity, _d, sides, modifier)) = (
        context("quantity", preceded(space0, parse_quantity)),
        context("separator", preceded(space0, parse_d)),
        context("sides", preceded(space0, parse_sides)),
        context("modifier", opt(preceded(space0, parse_modifier))),
    )
        .parse(expression)?;

    Ok((
        remaining,
        DiceRequest {
            quantity,
            sides,
            modifier: modifier.unwrap_or(0.0),
        },
    ))
}
