use nom::bytes::complete::tag;
use nom::character::complete::{digit0, digit1, one_of, space0};
use nom::combinator::{not, opt, peek, recognize};
use nom::sequence::{pair, preceded, terminated};
use nom::{IResult, Parser, branch::alt, combinator::map_res};
use std::num::ParseIntError;

pub struct DiceRequest {
    pub quantity: i32,
    pub sides: i32,
    pub modifier: i32,
}

fn str_to_i32(str: &str) -> Result<i32, ParseIntError> {
    str.parse::<i32>()
}

fn str_to_i32_or_one(s: &str) -> Result<i32, ParseIntError> {
    if s.is_empty() { Ok(1) } else { str_to_i32(s) }
}

fn parse_quantity(input: &str) -> IResult<&str, i32> {
    let strict_digits = terminated(digit0, peek(not(tag("."))));
    map_res(strict_digits, str_to_i32_or_one).parse(input)
}

fn parse_sides(input: &str) -> IResult<&str, i32> {
    let strict_digits = terminated(digit1, peek(not(tag("."))));

    map_res(strict_digits, str_to_i32).parse(input)
}

fn parse_d(input: &str) -> IResult<&str, &str> {
    alt((tag("d"), tag("D"))).parse(input)
}

fn parse_modifier(input: &str) -> IResult<&str, i32> {
    let strict_digits = terminated(digit1, peek(not(tag("."))));

    map_res(
        pair(one_of("+-"), strict_digits),
        |(sign, s)| -> Result<i32, ParseIntError> {
            let val = str_to_i32(s)?;
            Ok(if sign == '-' { -val } else { val })
        },
    )
    .parse(input)
}

pub fn dice_result(expression: &str) -> IResult<&str, DiceRequest> {
    let (remaining, (quantity, _d, sides, modifier)) = (
        preceded(space0, parse_quantity),
        preceded(space0, parse_d),
        preceded(space0, parse_sides),
        opt(preceded(space0, parse_modifier)),
    )
        .parse(expression)?;

    Ok((
        remaining,
        DiceRequest {
            quantity,
            sides,
            modifier: modifier.unwrap_or(0),
        },
    ))
}
