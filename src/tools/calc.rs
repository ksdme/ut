use crate::tool::{Output, Tool};
use anyhow::{Result, anyhow};
use clap::{Command, CommandFactory, Parser};
use nom::{
    IResult,
    branch::alt,
    bytes::complete::{tag, tag_no_case, take_while1},
    character::complete::{char, multispace0},
    combinator::{map, map_res, opt, recognize},
    multi::separated_list0,
    sequence::{delimited, pair, preceded, tuple},
};
use rust_decimal::MathematicalOps;
use rust_decimal::prelude::*;
use serde_json::json;

/// Calculator and number base converter.
#[derive(Parser, Debug)]
#[command(name = "calc", about = "Expression calculator with math functions")]
pub struct CalcTool {
    /// Expression to evaluate
    /// Supports arithmetic, functions, constants, and multiple number formats
    expression: String,
}

impl Tool for CalcTool {
    /// Returns the CLI command definition for this tool
    fn cli() -> Command {
        CalcTool::command()
    }

    /// Executes the calculator tool with the provided expression
    /// Returns the result formatted in decimal, binary, and hexadecimal
    fn execute(&self) -> Result<Option<Output>> {
        // Parse and evaluate the mathematical expression
        let result = evaluate_expression(&self.expression)?;

        // Format the result in multiple number bases
        let output = json!({
            "decimal": result.to_string(),
            "hex": format_hex(result),
            "binary": format_binary(result),
        });

        Ok(Some(Output::JsonValue(output)))
    }
}

/// Main entry point for expression evaluation using nom parser
fn evaluate_expression(input: &str) -> Result<Decimal> {
    match parse_expression(input.trim()) {
        Ok((remaining, result)) => {
            if remaining.is_empty() {
                Ok(result)
            } else {
                Err(anyhow!(
                    "Unexpected characters after expression: '{}'",
                    remaining
                ))
            }
        }
        Err(e) => Err(anyhow!("Parse error: {}", e)),
    }
}

/// Parses a complete mathematical expression with proper precedence
fn parse_expression(input: &str) -> IResult<&str, Decimal> {
    delimited(multispace0, parse_bitwise_or, multispace0)(input)
}

/// Validates operands for bitwise operations and converts them to u64
/// Returns an error if operands are not non-negative integers within u64 range
fn validate_bitwise_operands(a: Decimal, b: Decimal) -> Result<(u64, u64)> {
    if a.fract() != Decimal::ZERO || b.fract() != Decimal::ZERO {
        Err(anyhow!("Bitwise operations require integer operands"))
    } else if a.is_sign_negative() || b.is_sign_negative() {
        Err(anyhow!("Bitwise operations require non-negative integers"))
    } else if a > Decimal::from(u64::MAX) || b > Decimal::from(u64::MAX) {
        Err(anyhow!(
            "Bitwise operations require values within u64 range"
        ))
    } else {
        Ok((a.to_u64().unwrap(), b.to_u64().unwrap()))
    }
}

/// Handles bitwise OR (lower precedence than AND)
fn parse_bitwise_or(input: &str) -> IResult<&str, Decimal> {
    let (input, init) = parse_bitwise_and(input)?;

    let (input, ops) = nom::multi::many0(pair(
        delimited(multispace0, char('|'), multispace0),
        parse_bitwise_and,
    ))(input)?;

    let mut result = init;
    for (_, val) in ops {
        match validate_bitwise_operands(result, val) {
            Ok((a, b)) => result = Decimal::from(a | b),
            Err(_) => {
                return Err(nom::Err::Failure(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::Verify,
                )));
            }
        }
    }

    Ok((input, result))
}

/// Handles bitwise AND (higher precedence than OR, lower than addition)
fn parse_bitwise_and(input: &str) -> IResult<&str, Decimal> {
    let (input, init) = parse_additive(input)?;

    let (input, ops) = nom::multi::many0(pair(
        delimited(multispace0, char('&'), multispace0),
        parse_additive,
    ))(input)?;

    let mut result = init;
    for (_, val) in ops {
        match validate_bitwise_operands(result, val) {
            Ok((a, b)) => result = Decimal::from(a & b),
            Err(_) => {
                return Err(nom::Err::Failure(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::Verify,
                )));
            }
        }
    }

    Ok((input, result))
}

/// Handles addition and subtraction (medium precedence)
fn parse_additive(input: &str) -> IResult<&str, Decimal> {
    let (input, init) = parse_multiplicative(input)?;

    let (input, ops) = nom::multi::many0(pair(
        delimited(multispace0, alt((char('+'), char('-'))), multispace0),
        parse_multiplicative,
    ))(input)?;

    let result = ops.into_iter().fold(init, |acc, (op, val)| match op {
        '+' => acc + val,
        '-' => acc - val,
        _ => unreachable!(),
    });

    Ok((input, result))
}

/// Handles multiplication, division, and modulo (medium precedence)
fn parse_multiplicative(input: &str) -> IResult<&str, Decimal> {
    let (input, init) = parse_power(input)?;

    let (input, ops) = nom::multi::many0(pair(
        delimited(
            multispace0,
            alt((char('*'), char('/'), char('%'))),
            multispace0,
        ),
        parse_power,
    ))(input)?;

    let mut result = init;
    for (op, val) in ops {
        match op {
            '*' => result = result * val,
            '/' => {
                if val.is_zero() {
                    return Err(nom::Err::Failure(nom::error::Error::new(
                        input,
                        nom::error::ErrorKind::Verify,
                    )));
                }
                result = result / val;
            }
            '%' => result = result % val,
            _ => unreachable!(),
        }
    }

    Ok((input, result))
}

/// Handles exponentiation (high precedence, right-associative)
fn parse_power(input: &str) -> IResult<&str, Decimal> {
    let (input, base) = parse_unary(input)?;

    let (input, exponent) = opt(preceded(
        delimited(multispace0, char('^'), multispace0),
        parse_power, // Right associative recursion
    ))(input)?;

    match exponent {
        Some(exp) => Ok((input, base.powd(exp))),
        None => Ok((input, base)),
    }
}

/// Handles unary operators (+ and -)
fn parse_unary(input: &str) -> IResult<&str, Decimal> {
    alt((
        map(preceded(char('-'), parse_unary), |val| -val),
        map(preceded(char('+'), parse_unary), |val| val),
        parse_primary,
    ))(input)
}

/// Handles primary expressions (numbers, functions, parentheses)
fn parse_primary(input: &str) -> IResult<&str, Decimal> {
    delimited(
        multispace0,
        alt((
            parse_function,
            parse_constant,
            parse_number,
            delimited(char('('), parse_expression, char(')')),
        )),
        multispace0,
    )(input)
}

/// Parses mathematical functions with arguments
fn parse_function(input: &str) -> IResult<&str, Decimal> {
    let (input, name) = parse_identifier(input)?;

    let (input, _) = char('(')(input)?;
    let (input, args) = separated_list0(
        delimited(multispace0, char(','), multispace0),
        parse_expression,
    )(input)?;
    let (input, _) = char(')')(input)?;

    match apply_function(&name, args) {
        Ok(result) => Ok((input, result)),
        Err(_) => Err(nom::Err::Failure(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Verify,
        ))),
    }
}

/// Parses mathematical constants (pi, e)
fn parse_constant(input: &str) -> IResult<&str, Decimal> {
    alt((
        map(tag_no_case("pi"), |_| {
            Decimal::from_str("3.1415926535897932384626433832795").unwrap()
        }),
        map(tag_no_case("e"), |_| {
            Decimal::from_str("2.7182818284590452353602874713527").unwrap()
        }),
    ))(input)
}

/// Parses numbers in various formats (decimal, hex, binary)
fn parse_number(input: &str) -> IResult<&str, Decimal> {
    alt((parse_hex_number, parse_binary_number, parse_decimal_number))(input)
}

/// Parses hexadecimal numbers (0x prefix)
fn parse_hex_number(input: &str) -> IResult<&str, Decimal> {
    let (input, _) = tag_no_case("0x")(input)?;
    let (input, hex_str) = take_while1(|c: char| c.is_ascii_hexdigit())(input)?;

    match u64::from_str_radix(hex_str, 16) {
        Ok(value) => Ok((input, Decimal::from(value))),
        Err(_) => Err(nom::Err::Failure(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Verify,
        ))),
    }
}

/// Parses binary numbers (0b prefix)
fn parse_binary_number(input: &str) -> IResult<&str, Decimal> {
    let (input, _) = tag_no_case("0b")(input)?;
    let (input, bin_str) = take_while1(|c: char| c == '0' || c == '1')(input)?;

    match u64::from_str_radix(bin_str, 2) {
        Ok(value) => Ok((input, Decimal::from(value))),
        Err(_) => Err(nom::Err::Failure(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Verify,
        ))),
    }
}

/// Parses decimal numbers (including floating point)
fn parse_decimal_number(input: &str) -> IResult<&str, Decimal> {
    map_res(
        recognize(tuple((
            opt(alt((char('+'), char('-')))),
            alt((
                recognize(tuple((
                    nom::character::complete::digit1,
                    opt(tuple((char('.'), opt(nom::character::complete::digit1)))),
                ))),
                recognize(tuple((char('.'), nom::character::complete::digit1))),
            )),
        ))),
        |s: &str| Decimal::from_str(s),
    )(input)
}

/// Parses function and constant identifiers
fn parse_identifier(input: &str) -> IResult<&str, String> {
    map(
        recognize(tuple((
            alt((nom::character::complete::alpha1, tag("_"))),
            nom::multi::many0(alt((nom::character::complete::alphanumeric1, tag("_")))),
        ))),
        |s: &str| s.to_string(),
    )(input)
}

/// Applies mathematical functions to their arguments
/// Supports trigonometric, logarithmic, exponential, and utility functions
fn apply_function(name: &str, args: Vec<Decimal>) -> Result<Decimal> {
    match name {
        "sin" => {
            if args.len() != 1 {
                return Err(anyhow!("sin() expects 1 argument"));
            }
            // Calculate sine (input in radians)
            Ok(args[0].sin())
        }
        "cos" => {
            if args.len() != 1 {
                return Err(anyhow!("cos() expects 1 argument"));
            }
            // Calculate cosine (input in radians)
            Ok(args[0].cos())
        }
        "tan" => {
            if args.len() != 1 {
                return Err(anyhow!("tan() expects 1 argument"));
            }
            // Calculate tangent (input in radians)
            Ok(args[0].tan())
        }
        "log" => {
            if args.len() != 1 {
                return Err(anyhow!("log() expects 1 argument"));
            }
            if args[0] <= Decimal::ZERO {
                return Err(anyhow!("log() argument must be positive"));
            }
            // Calculate natural logarithm (base e)
            Ok(args[0].ln())
        }
        "exp" => {
            if args.len() != 1 {
                return Err(anyhow!("exp() expects 1 argument"));
            }
            // Calculate e raised to the power of the argument
            Ok(args[0].exp())
        }
        "sqrt" => {
            if args.len() != 1 {
                return Err(anyhow!("sqrt() expects 1 argument"));
            }
            if args[0] < Decimal::ZERO {
                return Err(anyhow!("sqrt() argument must be non-negative"));
            }
            // Calculate square root
            Ok(args[0]
                .sqrt()
                .ok_or_else(|| anyhow!("Invalid sqrt operation"))?)
        }
        "abs" => {
            if args.len() != 1 {
                return Err(anyhow!("abs() expects 1 argument"));
            }
            // Calculate absolute value (distance from zero)
            Ok(args[0].abs())
        }
        "floor" => {
            if args.len() != 1 {
                return Err(anyhow!("floor() expects 1 argument"));
            }
            // Round down to the nearest integer
            Ok(args[0].floor())
        }
        "ceil" => {
            if args.len() != 1 {
                return Err(anyhow!("ceil() expects 1 argument"));
            }
            // Round up to the nearest integer
            Ok(args[0].ceil())
        }
        "round" => match args.len() {
            1 => {
                // Round to nearest integer
                Ok(args[0].round())
            }
            2 => {
                // Round to specified number of decimal places
                let decimal_places = args[1].to_u32().unwrap_or(0);
                Ok(args[0].round_dp(decimal_places))
            }
            _ => Err(anyhow!("round() expects 1 or 2 arguments")),
        },
        _ => Err(anyhow!("Unknown function: {}", name)),
    }
}

/// Formats a decimal value as a binary string
/// Returns None for non-integer, negative, or values too large for u64
fn format_binary(value: Decimal) -> Option<String> {
    // Only format integers that fit in u64 range
    if value.fract() != Decimal::ZERO || value.is_sign_negative() || value > Decimal::from(u64::MAX)
    {
        None
    } else {
        let int_val = value.to_u64().unwrap_or(0);
        Some(format!("0b{:b}", int_val))
    }
}

/// Formats a decimal value as a hexadecimal string
/// Returns None for non-integer, negative, or values too large for u64
fn format_hex(value: Decimal) -> Option<String> {
    // Only format integers that fit in u64 range
    if value.fract() != Decimal::ZERO || value.is_sign_negative() || value > Decimal::from(u64::MAX)
    {
        None
    } else {
        let int_val = value.to_u64().unwrap_or(0);
        Some(format!("0x{:x}", int_val))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addition() {
        let tool = CalcTool {
            expression: "2 + 3".to_string(),
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        assert_eq!(val["decimal"].as_str().unwrap(), "5");
        assert_eq!(val["hex"].as_str().unwrap(), "0x5");
        assert_eq!(val["binary"].as_str().unwrap(), "0b101");
    }

    #[test]
    fn test_subtraction() {
        let tool = CalcTool {
            expression: "10 - 7".to_string(),
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        assert_eq!(val["decimal"].as_str().unwrap(), "3");
        assert_eq!(val["hex"].as_str().unwrap(), "0x3");
        assert_eq!(val["binary"].as_str().unwrap(), "0b11");
    }

    #[test]
    fn test_multiplication() {
        let tool = CalcTool {
            expression: "4 * 5".to_string(),
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        assert_eq!(val["decimal"].as_str().unwrap(), "20");
        assert_eq!(val["hex"].as_str().unwrap(), "0x14");
        assert_eq!(val["binary"].as_str().unwrap(), "0b10100");
    }

    #[test]
    fn test_division() {
        let tool = CalcTool {
            expression: "20 / 4".to_string(),
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        assert_eq!(val["decimal"].as_str().unwrap(), "5");
        assert_eq!(val["hex"].as_str().unwrap(), "0x5");
        assert_eq!(val["binary"].as_str().unwrap(), "0b101");
    }

    #[test]
    fn test_float_division() {
        let tool = CalcTool {
            expression: "7 / 2".to_string(),
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        assert!(val["decimal"].as_str().unwrap().starts_with("3.5"));
        assert!(val["hex"].is_null());
        assert!(val["binary"].is_null());
    }

    #[test]
    fn test_modulo() {
        let tool = CalcTool {
            expression: "10 % 3".to_string(),
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        assert_eq!(val["decimal"].as_str().unwrap(), "1");
        assert_eq!(val["hex"].as_str().unwrap(), "0x1");
        assert_eq!(val["binary"].as_str().unwrap(), "0b1");
    }

    #[test]
    fn test_exponentiation() {
        let tool = CalcTool {
            expression: "2 ^ 8".to_string(),
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        assert_eq!(val["decimal"].as_str().unwrap(), "256");
        assert_eq!(val["hex"].as_str().unwrap(), "0x100");
        assert_eq!(val["binary"].as_str().unwrap(), "0b100000000");
    }

    #[test]
    fn test_complex_expression() {
        let tool = CalcTool {
            expression: "(2 + 3) * 4 - 6 / 2".to_string(),
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        assert_eq!(val["decimal"].as_str().unwrap(), "17");
        assert_eq!(val["hex"].as_str().unwrap(), "0x11");
        assert_eq!(val["binary"].as_str().unwrap(), "0b10001");
    }

    #[test]
    fn test_nested_parentheses() {
        let tool = CalcTool {
            expression: "((10 + 5) * 2) / 3".to_string(),
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        assert_eq!(val["decimal"].as_str().unwrap(), "10");
        assert_eq!(val["hex"].as_str().unwrap(), "0xa");
        assert_eq!(val["binary"].as_str().unwrap(), "0b1010");
    }

    #[test]
    fn test_negative_numbers() {
        let tool = CalcTool {
            expression: "-5 + 10".to_string(),
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        assert_eq!(val["decimal"].as_str().unwrap(), "5");
        assert_eq!(val["hex"].as_str().unwrap(), "0x5");
        assert_eq!(val["binary"].as_str().unwrap(), "0b101");
    }

    #[test]
    fn test_decimal_numbers() {
        let tool = CalcTool {
            expression: "3.14 * 2".to_string(),
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        assert_eq!(val["decimal"].as_str().unwrap(), "6.28");
        assert!(val["hex"].is_null());
        assert!(val["binary"].is_null());
    }

    #[test]
    fn test_hex_input() {
        let tool = CalcTool {
            expression: "0xFF + 1".to_string(),
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        assert_eq!(val["decimal"].as_str().unwrap(), "256");
        assert_eq!(val["hex"].as_str().unwrap(), "0x100");
        assert_eq!(val["binary"].as_str().unwrap(), "0b100000000");
    }

    #[test]
    fn test_binary_input() {
        let tool = CalcTool {
            expression: "0b1010 + 0b0101".to_string(),
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        assert_eq!(val["decimal"].as_str().unwrap(), "15");
        assert_eq!(val["hex"].as_str().unwrap(), "0xf");
        assert_eq!(val["binary"].as_str().unwrap(), "0b1111");
    }

    #[test]
    fn test_hex_output() {
        let tool = CalcTool {
            expression: "255".to_string(),
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        assert_eq!(val["decimal"].as_str().unwrap(), "255");
        assert_eq!(val["hex"].as_str().unwrap(), "0xff");
        assert_eq!(val["binary"].as_str().unwrap(), "0b11111111");
    }

    #[test]
    fn test_binary_output() {
        let tool = CalcTool {
            expression: "15".to_string(),
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        assert_eq!(val["decimal"].as_str().unwrap(), "15");
        assert_eq!(val["hex"].as_str().unwrap(), "0xf");
        assert_eq!(val["binary"].as_str().unwrap(), "0b1111");
    }

    #[test]
    fn test_sqrt_function() {
        let tool = CalcTool {
            expression: "sqrt(16)".to_string(),
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        assert!(val["decimal"].as_str().unwrap().starts_with("4"));
        assert_eq!(val["hex"].as_str().unwrap(), "0x4");
        assert_eq!(val["binary"].as_str().unwrap(), "0b100");
    }

    #[test]
    fn test_abs_function() {
        let tool = CalcTool {
            expression: "abs(-42)".to_string(),
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        assert_eq!(val["decimal"].as_str().unwrap(), "42");
        assert_eq!(val["hex"].as_str().unwrap(), "0x2a");
        assert_eq!(val["binary"].as_str().unwrap(), "0b101010");
    }

    #[test]
    fn test_floor_function() {
        let tool = CalcTool {
            expression: "floor(3.7)".to_string(),
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        assert_eq!(val["decimal"].as_str().unwrap(), "3");
        assert_eq!(val["hex"].as_str().unwrap(), "0x3");
        assert_eq!(val["binary"].as_str().unwrap(), "0b11");
    }

    #[test]
    fn test_ceil_function() {
        let tool = CalcTool {
            expression: "ceil(3.2)".to_string(),
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        assert_eq!(val["decimal"].as_str().unwrap(), "4");
        assert_eq!(val["hex"].as_str().unwrap(), "0x4");
        assert_eq!(val["binary"].as_str().unwrap(), "0b100");
    }

    #[test]
    fn test_round_function() {
        let tool = CalcTool {
            expression: "round(3.6)".to_string(),
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        assert_eq!(val["decimal"].as_str().unwrap(), "4");
        assert_eq!(val["hex"].as_str().unwrap(), "0x4");
        assert_eq!(val["binary"].as_str().unwrap(), "0b100");
    }

    #[test]
    fn test_pi_constant() {
        let tool = CalcTool {
            expression: "pi * 2".to_string(),
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        let decimal_val = val["decimal"].as_str().unwrap();
        assert!(decimal_val.starts_with("6.28318"));
        assert!(val["hex"].is_null());
        assert!(val["binary"].is_null());
    }

    #[test]
    fn test_e_constant() {
        let tool = CalcTool {
            expression: "e".to_string(),
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        let decimal_val = val["decimal"].as_str().unwrap();
        assert!(decimal_val.starts_with("2.71828"));
        assert!(val["hex"].is_null());
        assert!(val["binary"].is_null());
    }

    #[test]
    fn test_invalid_expression() {
        let tool = CalcTool {
            expression: "2 + * 3".to_string(),
        };
        let result = tool.execute();

        assert!(result.is_err());
    }

    #[test]
    fn test_division_by_zero() {
        let tool = CalcTool {
            expression: "5 / 0".to_string(),
        };
        let result = tool.execute();

        assert!(result.is_err());
    }

    #[test]
    fn test_sqrt_negative() {
        let tool = CalcTool {
            expression: "sqrt(-1)".to_string(),
        };
        let result = tool.execute();

        assert!(result.is_err());
    }

    #[test]
    fn test_bitwise_and() {
        let tool = CalcTool {
            expression: "12 & 10".to_string(),
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        // 12 = 0b1100, 10 = 0b1010, 12 & 10 = 0b1000 = 8
        assert_eq!(val["decimal"].as_str().unwrap(), "8");
        assert_eq!(val["hex"].as_str().unwrap(), "0x8");
        assert_eq!(val["binary"].as_str().unwrap(), "0b1000");
    }

    #[test]
    fn test_bitwise_or() {
        let tool = CalcTool {
            expression: "12 | 10".to_string(),
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        // 12 = 0b1100, 10 = 0b1010, 12 | 10 = 0b1110 = 14
        assert_eq!(val["decimal"].as_str().unwrap(), "14");
        assert_eq!(val["hex"].as_str().unwrap(), "0xe");
        assert_eq!(val["binary"].as_str().unwrap(), "0b1110");
    }

    #[test]
    fn test_bitwise_with_hex() {
        let tool = CalcTool {
            expression: "0xFF & 0x0F".to_string(),
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        assert_eq!(val["decimal"].as_str().unwrap(), "15");
        assert_eq!(val["hex"].as_str().unwrap(), "0xf");
        assert_eq!(val["binary"].as_str().unwrap(), "0b1111");
    }

    #[test]
    fn test_bitwise_with_binary() {
        let tool = CalcTool {
            expression: "0b1111 | 0b1000".to_string(),
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        assert_eq!(val["decimal"].as_str().unwrap(), "15");
        assert_eq!(val["hex"].as_str().unwrap(), "0xf");
        assert_eq!(val["binary"].as_str().unwrap(), "0b1111");
    }

    #[test]
    fn test_bitwise_precedence() {
        let tool = CalcTool {
            expression: "8 | 4 & 12".to_string(),
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        // & has higher precedence than |
        // 4 & 12 = 4, then 8 | 4 = 12
        assert_eq!(val["decimal"].as_str().unwrap(), "12");
        assert_eq!(val["hex"].as_str().unwrap(), "0xc");
        assert_eq!(val["binary"].as_str().unwrap(), "0b1100");
    }

    #[test]
    fn test_bitwise_with_parentheses() {
        let tool = CalcTool {
            expression: "(8 | 4) & 12".to_string(),
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        // (8 | 4) = 12, then 12 & 12 = 12
        assert_eq!(val["decimal"].as_str().unwrap(), "12");
        assert_eq!(val["hex"].as_str().unwrap(), "0xc");
        assert_eq!(val["binary"].as_str().unwrap(), "0b1100");
    }

    #[test]
    fn test_bitwise_and_float_error() {
        let tool = CalcTool {
            expression: "3.5 & 2".to_string(),
        };
        let result = tool.execute();

        assert!(result.is_err());
    }

    #[test]
    fn test_bitwise_or_float_error() {
        let tool = CalcTool {
            expression: "4 | 2.5".to_string(),
        };
        let result = tool.execute();

        assert!(result.is_err());
    }

    #[test]
    fn test_bitwise_and_negative_error() {
        let tool = CalcTool {
            expression: "-5 & 3".to_string(),
        };
        let result = tool.execute();

        assert!(result.is_err());
    }

    #[test]
    fn test_bitwise_or_negative_error() {
        let tool = CalcTool {
            expression: "5 | -3".to_string(),
        };
        let result = tool.execute();

        assert!(result.is_err());
    }

    #[test]
    fn test_bitwise_complex_expression() {
        let tool = CalcTool {
            expression: "(0xFF & 0x0F) | (0x10 & 0x10)".to_string(),
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        // (255 & 15) | (16 & 16) = 15 | 16 = 31
        assert_eq!(val["decimal"].as_str().unwrap(), "31");
        assert_eq!(val["hex"].as_str().unwrap(), "0x1f");
        assert_eq!(val["binary"].as_str().unwrap(), "0b11111");
    }

    #[test]
    fn test_arithmetic_with_bitwise_and() {
        let tool = CalcTool {
            expression: "10 + 5 & 12".to_string(),
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        // 10 + 5 = 15, then 15 & 12 = 12
        assert_eq!(val["decimal"].as_str().unwrap(), "12");
        assert_eq!(val["hex"].as_str().unwrap(), "0xc");
        assert_eq!(val["binary"].as_str().unwrap(), "0b1100");
    }

    #[test]
    fn test_arithmetic_with_bitwise_or() {
        let tool = CalcTool {
            expression: "8 - 4 | 2".to_string(),
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        // 8 - 4 = 4, then 4 | 2 = 6
        assert_eq!(val["decimal"].as_str().unwrap(), "6");
        assert_eq!(val["hex"].as_str().unwrap(), "0x6");
        assert_eq!(val["binary"].as_str().unwrap(), "0b110");
    }

    #[test]
    fn test_multiplication_with_bitwise() {
        let tool = CalcTool {
            expression: "2 * 4 & 7".to_string(),
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        // 2 * 4 = 8, then 8 & 7 = 0
        assert_eq!(val["decimal"].as_str().unwrap(), "0");
        assert_eq!(val["hex"].as_str().unwrap(), "0x0");
        assert_eq!(val["binary"].as_str().unwrap(), "0b0");
    }

    #[test]
    fn test_division_with_bitwise() {
        let tool = CalcTool {
            expression: "16 / 2 | 3".to_string(),
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        // 16 / 2 = 8, then 8 | 3 = 11
        assert_eq!(val["decimal"].as_str().unwrap(), "11");
        assert_eq!(val["hex"].as_str().unwrap(), "0xb");
        assert_eq!(val["binary"].as_str().unwrap(), "0b1011");
    }

    #[test]
    fn test_bitwise_with_parentheses_arithmetic() {
        let tool = CalcTool {
            expression: "(10 | 5) + 2".to_string(),
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        // (10 | 5) = 15, then 15 + 2 = 17
        assert_eq!(val["decimal"].as_str().unwrap(), "17");
        assert_eq!(val["hex"].as_str().unwrap(), "0x11");
        assert_eq!(val["binary"].as_str().unwrap(), "0b10001");
    }

    #[test]
    fn test_mixed_bitwise_arithmetic() {
        let tool = CalcTool {
            expression: "3 + 4 & 5 | 2".to_string(),
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        // 3 + 4 = 7, 7 & 5 = 5, 5 | 2 = 7
        assert_eq!(val["decimal"].as_str().unwrap(), "7");
        assert_eq!(val["hex"].as_str().unwrap(), "0x7");
        assert_eq!(val["binary"].as_str().unwrap(), "0b111");
    }

    #[test]
    fn test_bitwise_or_chain() {
        let tool = CalcTool {
            expression: "1 | 2 | 4 | 8".to_string(),
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        // 1 | 2 | 4 | 8 = 15
        assert_eq!(val["decimal"].as_str().unwrap(), "15");
        assert_eq!(val["hex"].as_str().unwrap(), "0xf");
        assert_eq!(val["binary"].as_str().unwrap(), "0b1111");
    }

    #[test]
    fn test_bitwise_and_chain() {
        let tool = CalcTool {
            expression: "255 & 127 & 63".to_string(),
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        // 255 & 127 & 63 = 63
        assert_eq!(val["decimal"].as_str().unwrap(), "63");
        assert_eq!(val["hex"].as_str().unwrap(), "0x3f");
        assert_eq!(val["binary"].as_str().unwrap(), "0b111111");
    }

    #[test]
    fn test_bitwise_zero_operands() {
        let tool = CalcTool {
            expression: "0 & 255".to_string(),
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        assert_eq!(val["decimal"].as_str().unwrap(), "0");
        assert_eq!(val["hex"].as_str().unwrap(), "0x0");
        assert_eq!(val["binary"].as_str().unwrap(), "0b0");
    }

    #[test]
    fn test_bitwise_with_modulo() {
        let tool = CalcTool {
            expression: "17 % 5 & 3".to_string(),
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        // 17 % 5 = 2, 2 & 3 = 2
        assert_eq!(val["decimal"].as_str().unwrap(), "2");
        assert_eq!(val["hex"].as_str().unwrap(), "0x2");
        assert_eq!(val["binary"].as_str().unwrap(), "0b10");
    }
}
