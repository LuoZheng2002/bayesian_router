use nom::{
    IResult, Parser,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::{char, digit1, multispace0, multispace1},
    combinator::{map, map_res},
    multi::many0,
    sequence::{delimited, preceded, separated_pair},
};

use crate::s_expr::SExpr;

fn is_atom_char(c: char) -> bool {
    !c.is_whitespace() && c != '(' && c != ')'
}

fn parse_atom(input: &str) -> IResult<&str, SExpr> {
    let (input, atom) = take_while1(is_atom_char)(input)?;
    Ok((input, SExpr::Atom(atom.to_string())))
}

fn parse_quoted_string(input: &str) -> IResult<&str, SExpr> {
    let (input, s) = delimited(char('"'), take_while(|c| c != '"'), char('"')).parse(input)?;
    Ok((input, SExpr::Atom(s.to_string())))
}

fn parse_list(input: &str) -> IResult<&str, SExpr> {
    let (input, _) = char('(')(input)?;
    let (input, _) = multispace0(input)?;
    let (input, items) = many0(preceded(multispace0, parse_expr)).parse(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = char(')')(input)?;
    Ok((input, SExpr::List(items)))
}

fn parse_expr(input: &str) -> IResult<&str, SExpr> {
    preceded(
        multispace0,
        nom::branch::alt((parse_list, parse_quoted_string, parse_atom)),
    )
    .parse(input)
}

pub fn parse_dsn_to_s_expr(input: &str) -> Result<SExpr, nom::Err<nom::error::Error<&str>>> {
    let (remaining, expr) = parse_expr(input)?;
    if !remaining.trim().is_empty() {
        eprintln!("Warning: leftover input: {:?}", remaining);
    }
    Ok(expr)
}
