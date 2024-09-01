use nom::bytes::complete::take_while;
use nom::character::complete::char;
use nom::multi::{many0, separated_list0};
use nom::sequence::tuple;
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag},
    character::complete::{digit1, multispace0, multispace1},
    combinator::{map, opt},
    multi::{fold_many0, many1},
    sequence::{delimited, preceded, separated_pair, terminated},
    IResult,
};
use std::collections::HashMap;
use crate::ast::pattern_action_fun::PatternActionFunc;
use crate::parser::VariableValue::Action;

#[derive(Debug, PartialEq)]
enum Function {
    FunctionWithArgs(Vec<(String, Vec<String>)>),
}

#[derive(Debug, PartialEq)]
enum VariableValue {
    String(String),
    Integer(i32),
    PatternAction { pattern: String, command: Function },
    Action { command: Function },
    Case {
        pattern: String,
        cases: HashMap<String, VariableValue>,
        default: Option<Function>,
    },
}

#[derive(Debug, PartialEq)]
struct Variables {
    variables: HashMap<String, VariableValue>,
}

#[derive(Debug, PartialEq)]
struct ShireFile {
    variables: Variables,
    body: Vec<String>, // This represents the body where `$var1` is located.
}

fn parse_quote_string(input: &str) -> IResult<&str, String> {
    // let esc = escaped(none_of("\\\'"), '\\', tag("'"));
    // let esc_or_empty = alt((esc, tag("")));
    // let res = delimited(tag("'"), esc_or_empty, tag("'"))(input)?;
    //
    // Ok(res)
    map(is_not("|\n"), |s: &str| s.to_string())(input)
}

fn parse_function(input: &str) -> IResult<&str, (String, Vec<String>)> {
    let (input, cmd) = take_while(|c: char| c.is_alphanumeric())(input)?;
    let (input, args) = opt(preceded(
        multispace0,
        delimited(
            char('('),
            separated_list0(char(','), delimited(char('"'), is_not("\""), char('"'))),
            char(')'),
        ),
    ))(input)?;

    let opt_args: Vec<String> = match args {
        Some(args) => args.into_iter().map(|s| s.to_string()).collect(),
        None => vec![],
    };

    Ok((input, (cmd.to_string(), opt_args)))
}

/// Parser for pattern action
/// for example: `/.*.java/ { grep("error.log") | sort | xargs("rm") }`
fn parse_pattern_actions(input: &str) -> IResult<&str, VariableValue> {
    let (input, pattern) = delimited(tag("/"), is_not("/"), tag("/"))(input)?;

    let (input, command) = delimited(
        preceded(multispace0, tag("{")),
        many0(parse_function),
        preceded(multispace0, tag("}")),
    )(input)?;

    Ok((input, VariableValue::PatternAction {
        pattern: pattern.to_string(),
        command: Function::FunctionWithArgs(command),
    }))
}

// Parser for case blocks
fn parse_case_block(input: &str) -> IResult<&str, VariableValue> {
    let (input, pattern) = delimited(tag("/"), is_not("/"), tag("/"))(input)?;
    let (input, _) = delimited(multispace0, tag("{"), multispace0)(input)?;

    let mut cases: HashMap<String, VariableValue> = HashMap::new();
    let mut default = None;
    let (mut input, _) = fold_many0(
        terminated(
            separated_pair(
                delimited(tag("\""), is_not("\""), tag("\"")),
                multispace1,
                parse_function,
            ),
            multispace0,
        ),
        || (),
        |_, (key, value)| {
            cases.insert(key.to_string(), Action { command: Function::FunctionWithArgs(vec![value]) });
        },
    )(input)?;

    let (mut input, _) = opt(terminated(tag("default"), multispace1))(input)?;

    if let Ok((remaining_input, cmd)) = parse_function(input) {
        default = Some(
            Function::FunctionWithArgs(vec![cmd])
        );
        input = remaining_input;
    }

    let (input, _) = delimited(multispace0, tag("}"), multispace0)(input)?;

    Ok((input, VariableValue::Case {
        pattern: pattern.to_string(),
        cases: cases,
        default,
    }))
}

// Parser for integer
fn parse_integer(input: &str) -> IResult<&str, VariableValue> {
    let (input, digits) = digit1(input)?;
    let value = digits.parse::<i32>().unwrap();
    Ok((input, VariableValue::Integer(value)))
}

// Parser for a variable value
fn parse_variable_value(input: &str) -> IResult<&str, VariableValue> {
    alt((
        parse_pattern_actions,
        parse_case_block,
        map(parse_quote_string, VariableValue::String),
        parse_integer,
    ))(input)
}

///
/// parse for key value pair value
/// for example: `"var1": "demo"`
///
fn parse_variable(input: &str) -> IResult<&str, (String, VariableValue)> {
    let (input, (key, value)) = tuple((
        // for string
        preceded(multispace0, delimited(tag("\""), is_not("\""), tag("\""))),
        // for patter action
        preceded(
            delimited(multispace0, tag(":"), multispace0),
            parse_variable_value
        ),
    ))(input)?;

    Ok((input, (key.to_string(), value)))
}

// 前置的 `---` 标记
fn parse_frontmatter_start(input: &str) -> IResult<&str, ()> {
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("---")(input)?;
    let (input, _) = multispace0(input)?;
    Ok((input, ()))
}

// 结束的 `---` 标记
fn parse_frontmatter_end(input: &str) -> IResult<&str, ()> {
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("---")(input)?;
    let (input, _) = multispace0(input)?;
    Ok((input, ()))
}

// 解析变量块
fn parse_hobbit_hole(input: &str) -> IResult<&str, Variables> {
    let (input, _) = parse_frontmatter_start(input)?;
    let (input, _) = tuple((multispace0, tag("variables"), multispace0, tag(":")))(input)?;

    let (input, vars) = fold_many0(
        terminated(parse_variable, multispace0),
        HashMap::new,
        |mut acc, (k, v)| {
            acc.insert(k, v);
            acc
        },
    )(input)?;

    let (input, _) = parse_frontmatter_end(input)?;
    Ok((input, Variables { variables: vars }))
}

// Parser for the entire file
fn parse_file(input: &str) -> IResult<&str, ShireFile> {
    let (input, variables) = parse_hobbit_hole(input)?;
    let (input, body) = many1(parse_quote_string)(input)?; // Simplified for demonstration
    Ok((input, ShireFile { variables, body }))
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_parse_regex_block() {
    //     assert_eq!(
    //         parse_pattern_actions("/.*.java/ { grep(\"error.log\") | sort | xargs(\"rm\") }" ),
    //         Ok((
    //             "",
    //             VariableValue::PatternAction {
    //                 pattern: ".*.java".to_string(),
    //                 command: Function::FunctionWithArgs(vec![
    //                     ("grep".to_string(), vec!["error.log".to_string()]),
    //                     ("sort".to_string(), vec![]),
    //                     ("xargs".to_string(), vec!["rm".to_string()])
    //                 ])
    //             }
    //         ))
    //     );
    // }

    #[test]
    fn parse_block() {
        let input = r#"
---
variables:
  "var2": /.*.java/ { grep("error.log") | sort | xargs("rm")}
---

$var1
"#;

        println!("{:?}", parse_file(input));
    }

    // /// ```shire
    // /// ---
    // /// variables:
    // ///   "var1": "demo"
    // ///   "var2": /.*.java/ { grep("error.log") | sort | xargs("rm")}
    // ///   "var3": /.*.log/ {
    // ///     case "$0" {
    // ///       "error" { grep("ERROR") | sort | xargs("notify_admin") }
    // ///       "warn" { grep("WARN") | sort | xargs("notify_admin") }
    // ///       "info" { grep("INFO") | sort | xargs("notify_user") }
    // ///       default  { grep("ERROR") | sort | xargs("notify_admin") }
    // ///     }
    // ///   }
    // ///   "var4": 42
    // /// ---
    // /// ```
    // #[test]
    // fn test_parse_case_block() {
    //     let input = r#""error" { grep("ERROR") | sort | xargs("notify_admin") }"#;
    //     assert_eq!(
    //         parse_case_block(input),
    //         Ok((
    //             "",
    //             VariableValue::Case {
    //                 pattern: "error".to_string(),
    //                 cases: vec![
    //                     ("error".to_string(), Command::Pipeline(vec![
    //                         ("grep".to_string(), vec!["ERROR".to_string()]),
    //                         ("sort".to_string(), vec![]),
    //                         ("xargs".to_string(), vec!["notify_admin".to_string()])
    //                     ]))
    //                 ].into_iter().collect(),
    //                 default: None
    //             }
    //         ))
    //     );
    // }
}
