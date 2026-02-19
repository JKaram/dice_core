use nom::bytes::complete::tag;
use nom::character::complete::one_of;
use nom::combinator::map;
use nom::multi::many0;
use nom::number::complete::double;
use nom::sequence::tuple;
use nom::{IResult, Parser};

#[derive(Debug, Clone, PartialEq)]
pub enum DropKeep {
    KeepHighest(u32),
    KeepLowest(u32),
    DropHighest(u32),
    DropLowest(u32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiceTerm {
    pub quantity: f64,
    pub sides: f64,
    pub is_subtracted: bool,
    pub drop_keep: Option<DropKeep>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiceRequest {
    pub terms: Vec<DiceTerm>,
    pub modifier: f64,
}

fn parse_d(input: &str) -> IResult<&str, &str> {
    tag("d").or(tag("D")).parse(input)
}

fn ws(input: &str) -> IResult<&str, &str> {
    let chars = input.chars().take_while(|c| c.is_whitespace()).collect::<String>();
    let len = chars.len();
    if len > 0 {
        Ok((&input[len..], &input[..len]))
    } else {
        Ok((input, ""))
    }
}

fn looks_like_dice(input: &str) -> bool {
    let trimmed = input.trim_start();
    
    let first_char = trimmed.chars().next();
    if first_char == Some('d') || first_char == Some('D') {
        return true;
    }
    
    let non_digit_pos = trimmed.find(|c: char| !c.is_ascii_digit() && !c.is_whitespace());
    if let Some(pos) = non_digit_pos {
        let c = trimmed.chars().nth(pos);
        if c == Some('d') || c == Some('D') {
            return true;
        }
    }
    false
}

fn parse_drop_keep(input: &str) -> IResult<&str, Option<DropKeep>> {
    let (input, _) = ws(input)?;
    
    if input.is_empty() {
        return Ok((input, None));
    }
    
    let mut chars = input.chars();
    let first = chars.next().unwrap();
    let second = chars.next();
    
    let kind = match (first, second) {
        ('k', Some('h')) => Some("kh"),
        ('k', Some('l')) => Some("kl"),
        ('d', Some('h')) => Some("dh"),
        ('d', Some('l')) => Some("dl"),
        _ => None,
    };
    
    let kind = match kind {
        Some(k) => k,
        None => return Ok((input, None)),
    };
    
    let input = &input[2..];
    let (input, _) = ws(input)?;
    
    let (input, count) = double.parse(input)?;
    let count = count as u32;
    
    let result = match kind {
        "kh" => DropKeep::KeepHighest(count),
        "kl" => DropKeep::KeepLowest(count),
        "dh" => DropKeep::DropHighest(count),
        "dl" => DropKeep::DropLowest(count),
        _ => unreachable!(),
    };
    
    Ok((input, Some(result)))
}

fn parse_dice_term(input: &str) -> IResult<&str, (f64, f64, Option<DropKeep>)> {
    let (remaining, (_, qty, _, _d, _, sides)) = tuple((
        ws,
        nom::combinator::opt(double),
        ws,
        parse_d,
        ws,
        double,
    )).parse(input)?;

    let (remaining, drop_keep) = parse_drop_keep(remaining)?;

    Ok((remaining, (qty.unwrap_or(1.0), sides, drop_keep)))
}

fn parse_subsequent(input: &str) -> IResult<&str, (Option<(f64, f64, bool, Option<DropKeep>)>, Option<f64>)> {
    let (input, _) = ws(input)?;
    
    let sign_char = input.chars().next();
    if sign_char != Some('+') && sign_char != Some('-') {
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Char)));
    }
    let is_subtracted = sign_char == Some('-');
    let input = &input[1..];
    
    let (input, _) = ws(input)?;
    
    if looks_like_dice(input) {
        let (remaining, (qty, sides, drop_keep)) = parse_dice_term(input)?;
        return Ok((remaining, (Some((qty, sides, is_subtracted, drop_keep)), None)));
    } else {
        let (remaining, val) = double.parse(input)?;
        let signed_val = if is_subtracted { -val } else { val };
        return Ok((remaining, (None, Some(signed_val))));
    }
}

pub fn dice_result(input: &str) -> IResult<&str, DiceRequest> {
    let (remaining, (first_qty, first_sides, first_dk)) = parse_dice_term(input)?;

    let mut terms = vec![DiceTerm {
        quantity: first_qty,
        sides: first_sides,
        is_subtracted: false,
        drop_keep: first_dk,
    }];
    let mut modifier = 0.0;

    let (remaining, results) = many0(parse_subsequent).parse(remaining)?;

    for (dice, mod_val) in results {
        if let Some((qty, sides, is_subtracted, drop_keep)) = dice {
            terms.push(DiceTerm {
                quantity: qty,
                sides,
                is_subtracted,
                drop_keep,
            });
        }
        if let Some(m) = mod_val {
            modifier += m;
        }
    }

    Ok((remaining, DiceRequest { terms, modifier }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let (_, req) = dice_result("2d6").unwrap();
        assert_eq!(req.terms.len(), 1);
        assert_eq!(req.terms[0].quantity, 2.0);
        assert_eq!(req.terms[0].sides, 6.0);
        assert!(!req.terms[0].is_subtracted);
        assert!(req.terms[0].drop_keep.is_none());
        assert_eq!(req.modifier, 0.0);
    }

    #[test]
    fn test_two_terms() {
        let (_, req) = dice_result("1d20+1d4").unwrap();
        assert_eq!(req.terms.len(), 2);
        assert_eq!(req.terms[0].quantity, 1.0);
        assert_eq!(req.terms[0].sides, 20.0);
        assert_eq!(req.terms[1].quantity, 1.0);
        assert_eq!(req.terms[1].sides, 4.0);
        assert_eq!(req.modifier, 0.0);
    }

    #[test]
    fn test_with_modifier() {
        let (_, req) = dice_result("2d6+3").unwrap();
        assert_eq!(req.terms.len(), 1);
        assert_eq!(req.modifier, 3.0);
    }

    #[test]
    fn test_subtract_dice() {
        let (_, req) = dice_result("2d6-1d4").unwrap();
        assert_eq!(req.terms.len(), 2);
        assert!(req.terms[1].is_subtracted);
    }

    #[test]
    fn test_mixed() {
        let (_, req) = dice_result("1d20+2d6-5").unwrap();
        assert_eq!(req.terms.len(), 2);
        assert_eq!(req.modifier, -5.0);
    }

    #[test]
    fn test_with_spaces() {
        let (_, req) = dice_result(" 2d6 + 1d4 ").unwrap();
        assert_eq!(req.terms.len(), 2);
    }

    #[test]
    fn test_three_terms() {
        let (_, req) = dice_result("1d20+1d20+1d20").unwrap();
        assert_eq!(req.terms.len(), 3);
    }

    #[test]
    fn test_keep_highest() {
        let (_, req) = dice_result("4d6kh3").unwrap();
        assert_eq!(req.terms.len(), 1);
        assert_eq!(req.terms[0].quantity, 4.0);
        assert_eq!(req.terms[0].sides, 6.0);
        assert_eq!(req.terms[0].drop_keep, Some(DropKeep::KeepHighest(3)));
    }

    #[test]
    fn test_keep_lowest() {
        let (_, req) = dice_result("4d6kl2").unwrap();
        assert_eq!(req.terms[0].drop_keep, Some(DropKeep::KeepLowest(2)));
    }

    #[test]
    fn test_drop_highest() {
        let (_, req) = dice_result("5d6dh2").unwrap();
        assert_eq!(req.terms[0].drop_keep, Some(DropKeep::DropHighest(2)));
    }

    #[test]
    fn test_drop_lowest() {
        let (_, req) = dice_result("5d6dl1").unwrap();
        assert_eq!(req.terms[0].drop_keep, Some(DropKeep::DropLowest(1)));
    }

    #[test]
    fn test_keep_highest_with_spaces() {
        let (_, req) = dice_result("4d6 kh 3").unwrap();
        assert_eq!(req.terms[0].drop_keep, Some(DropKeep::KeepHighest(3)));
    }

    #[test]
    fn test_keep_highest_in_expression() {
        let (_, req) = dice_result("4d6kh3+2d8").unwrap();
        assert_eq!(req.terms.len(), 2);
        assert_eq!(req.terms[0].drop_keep, Some(DropKeep::KeepHighest(3)));
        assert!(req.terms[1].drop_keep.is_none());
    }
}
