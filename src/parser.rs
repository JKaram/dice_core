use nom::bytes::complete::tag;
use nom::character::complete::{digit0, digit1, one_of};
use nom::combinator::recognize;
use nom::sequence::pair;
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
    map_res(digit0, str_to_i32_or_one).parse(input)
}

fn parse_sides(input: &str) -> IResult<&str, i32> {
    map_res(digit1, str_to_i32).parse(input)
}

fn parse_d(input: &str) -> IResult<&str, &str> {
    alt((tag("d"), tag("D"))).parse(input)
}

fn parse_modifier(input: &str) -> IResult<&str, i32> {
    map_res(recognize(pair(one_of("+-"), digit1)), str_to_i32).parse(input)
}

pub fn dice_result(expression: &str) -> IResult<&str, DiceRequest> {
    let (remaining, (quantity, _d, sides, modifier)) =
        (parse_quantity, parse_d, parse_sides, parse_modifier).parse(expression)?;

    Ok((
        remaining,
        DiceRequest {
            quantity,
            sides,
            modifier,
        },
    ))
}
