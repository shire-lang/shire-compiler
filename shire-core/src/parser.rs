use crate::parser::VariableTransform::Action;
use nom::bytes::complete::take_while;
use nom::character::complete::char;
use nom::multi::separated_list0;
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

#[derive(Debug, PartialEq)]
enum Function {
    Functions(
        Vec<(String, Vec<String>)>,
    ),
}

#[derive(Debug, PartialEq)]
enum VariableTransform {
    String(String),
    Integer(i32),
    PatternAction { pattern: String, command: Function },
    Action { command: Function },
    Case {
        pattern: String,
        cases: HashMap<String, VariableTransform>,
        default: Option<Function>,
    },
}

#[derive(Debug)]
pub enum ConfigValue {
    String(String),
    Number(i64),
    Object(Vec<(String, ConfigValue)>),
}

#[derive(Debug, PartialEq)]
enum InteractionType {
    AppendCursor,
    AppendCursorStream,
    OutputFile,
    ReplaceSelection,
    ReplaceCurrentFile,
    InsertBeforeSelection,
    RunPanel,
    OnPaste,
}

/// TODO: implement the description trait for InteractionType
trait description {
    fn description(&self) -> &str;
}

impl InteractionType {
    fn description(&self) -> &str {
        match self {
            InteractionType::AppendCursor => "Append content at the current cursor position",
            InteractionType::AppendCursorStream => "Append content at the current cursor position, stream output",
            InteractionType::OutputFile => "Output to a new file",
            InteractionType::ReplaceSelection => "Replace the currently selected content",
            InteractionType::ReplaceCurrentFile => "Replace the content of the current file",
            InteractionType::InsertBeforeSelection => "Insert content before the currently selected content",
            InteractionType::RunPanel => "Show Result in Run panel which is the bottom of the IDE",
            InteractionType::OnPaste => "Copy the content to the clipboard",
        }
    }

    fn from(interaction: &str) -> InteractionType {
        match interaction.to_lowercase().as_str() {
            "appendcursor" => InteractionType::AppendCursor,
            "appendcursorstream" => InteractionType::AppendCursorStream,
            "outputfile" => InteractionType::OutputFile,
            "replaceselection" => InteractionType::ReplaceSelection,
            "replacecurrentfile" => InteractionType::ReplaceCurrentFile,
            "insertbeforeselection" => InteractionType::InsertBeforeSelection,
            "runpanel" => InteractionType::RunPanel,
            "onpaste" => InteractionType::OnPaste,
            _ => InteractionType::RunPanel,
        }
    }
}

#[derive(Debug, PartialEq)]
enum ShireActionLocation {
    ContextMenu,
    IntentionMenu,
    TerminalMenu,
    CommitMenu,
    RunPanel,
    InputBox,
}

impl ShireActionLocation {
    fn location(&self) -> &str {
        match self {
            ShireActionLocation::ContextMenu => "ContextMenu",
            ShireActionLocation::IntentionMenu => "IntentionMenu",
            ShireActionLocation::TerminalMenu => "TerminalMenu",
            ShireActionLocation::CommitMenu => "CommitMenu",
            ShireActionLocation::RunPanel => "RunPanel",
            ShireActionLocation::InputBox => "InputBox",
        }
    }

    fn description(&self) -> &str {
        match self {
            ShireActionLocation::ContextMenu => "Show in Context Menu by Right Click",
            ShireActionLocation::IntentionMenu => "Show in Intention Menu by Alt+Enter",
            ShireActionLocation::TerminalMenu => "Show in Terminal panel menu bar",
            ShireActionLocation::CommitMenu => "Show in Commit panel menu bar",
            ShireActionLocation::RunPanel => "Show in Run panel which is the bottom of the IDE",
            ShireActionLocation::InputBox => "Show in Input Box",
        }
    }

    fn from(action_location: &str) -> ShireActionLocation {
        match action_location {
            "ContextMenu" => ShireActionLocation::ContextMenu,
            "IntentionMenu" => ShireActionLocation::IntentionMenu,
            "TerminalMenu" => ShireActionLocation::TerminalMenu,
            "CommitMenu" => ShireActionLocation::CommitMenu,
            "RunPanel" => ShireActionLocation::RunPanel,
            "InputBox" => ShireActionLocation::InputBox,
            _ => ShireActionLocation::RunPanel,
        }
    }

    fn all() -> Vec<ShireActionLocation> {
        vec![
            ShireActionLocation::ContextMenu,
            ShireActionLocation::IntentionMenu,
            ShireActionLocation::TerminalMenu,
            ShireActionLocation::CommitMenu,
            ShireActionLocation::RunPanel,
            ShireActionLocation::InputBox,
        ]
    }

    fn default() -> &'static str {
        ShireActionLocation::ContextMenu.location()
    }
}

#[derive(Debug, PartialEq)]
pub struct HobbitHole {
    name: String,
    description: Option<String>,
    interaction: Option<InteractionType>,
    action_location: Option<ShireActionLocation>,
    variables: HashMap<String, VariableTransform>,
}

impl Default for HobbitHole {
    fn default() -> Self {
        HobbitHole {
            name: "".to_string(),
            description: None,
            interaction: None,
            action_location: None,
            variables: HashMap::new(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum HobbitHoleKey {
    Name,
    Description,
    Interaction,
    ActionLocation,
    Variables,
}

impl From<&str> for HobbitHoleKey {
    fn from(key: &str) -> Self {
        match key {
            "name" => HobbitHoleKey::Name,
            "description" => HobbitHoleKey::Description,
            "interaction" => HobbitHoleKey::Interaction,
            "actionLocation" => HobbitHoleKey::ActionLocation,
            "variables" => HobbitHoleKey::Variables,
            _ => HobbitHoleKey::Name,
        }
    }
}

#[derive(Debug, PartialEq)]
struct ShireFile {
    hobbit: HobbitHole,
    body: Vec<String>, // This represents the body where `$var1` is located.
}

fn parse_string(input: &str) -> IResult<&str, String> {
    map(is_not("|\n"), |s: &str| s.to_string())(input)
}

fn parse_quoted_string(input: &str) -> IResult<&str, String> {
    delimited(
        char('"'),                             // opening quote
        map(is_not("\""), |s: &str| s.to_string()), // content of the string
        char('"'),                             // closing quote
    )(input)
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
fn parse_pattern_actions(input: &str) -> IResult<&str, VariableTransform> {
    let (input, pattern) = delimited(tag("/"), is_not("/"), tag("/"))(input)?;

    let (input, functions) = delimited(
        tuple((multispace0, tag("{"), multispace0)),
        separated_list0(
            delimited(multispace0, tag("|"), multispace0),
            parse_function,
        ),
        tuple((multispace0, tag("}"), multispace0)),
    )(input)?;


    Ok((input, VariableTransform::PatternAction {
        pattern: pattern.to_string(),
        command: Function::Functions(functions),
    }))
}

// Parser for case blocks
fn parse_case_block(input: &str) -> IResult<&str, VariableTransform> {
    let (input, pattern) = delimited(tag("/"), is_not("/"), tag("/"))(input)?;
    let (input, _) = delimited(multispace0, tag("{"), multispace0)(input)?;

    let mut cases: HashMap<String, VariableTransform> = HashMap::new();
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
            cases.insert(key.to_string(), Action { command: Function::Functions(vec![value]) });
        },
    )(input)?;

    let (mut input, _) = opt(terminated(tag("default"), multispace1))(input)?;

    if let Ok((remaining_input, cmd)) = parse_function(input) {
        default = Some(
            Function::Functions(vec![cmd])
        );
        input = remaining_input;
    }

    let (input, _) = delimited(multispace0, tag("}"), multispace0)(input)?;

    Ok((input, VariableTransform::Case {
        pattern: pattern.to_string(),
        cases: cases,
        default,
    }))
}

fn parse_integer(input: &str) -> IResult<&str, i32> {
    let (input, digits) = digit1(input)?;
    let value = digits.parse::<i32>().unwrap();
    Ok((input, value))
}

fn parse_variable_value(input: &str) -> IResult<&str, VariableTransform> {
    alt((
        parse_pattern_actions,
        parse_case_block,
        map(parse_quoted_string, VariableTransform::String),
        map(parse_integer, VariableTransform::Integer),
    ))(input)
}

///
/// parse for key value pair value
/// for example: `"var1": "demo"`
///
fn parse_variable(input: &str) -> IResult<&str, (String, VariableTransform)> {
    let (input, (key, value)) = tuple((
        preceded(multispace0, delimited(tag("\""), is_not("\""), tag("\""))),
        preceded(
            delimited(multispace0, tag(":"), multispace0),
            parse_variable_value,
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

fn parse_frontmatter_end(input: &str) -> IResult<&str, ()> {
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("---")(input)?;
    let (input, _) = multispace0(input)?;
    Ok((input, ()))
}

fn parse_hobbit_hole(input: &str) -> IResult<&str, HobbitHole> {
    let (mut cond_input, _) = parse_frontmatter_start(input)?;
    let mut hole = HobbitHole::default();
    while !cond_input.is_empty() {
        let (input, key) = take_while(|c: char| c.is_alphanumeric())(cond_input)?;
        let (value_input, _) = delimited(multispace0, tag(":"), multispace0)(input)?;
        cond_input = value_input;

        match HobbitHoleKey::from(key) {
            HobbitHoleKey::Name => {
                let (new, name) = parse_quoted_string(value_input)?;
                hole.name = name;
                cond_input = new
            }
            HobbitHoleKey::Description => {
                let (new, description) = parse_string(value_input)?;
                hole.description = Some(description);
                cond_input = new;
            }
            HobbitHoleKey::Interaction => {
                let (new, interaction) = parse_string(value_input)?;
                hole.interaction = Some(InteractionType::from(&interaction));
                cond_input = new;
            }
            HobbitHoleKey::ActionLocation => {
                let (new, action_location) = parse_string(value_input)?;
                hole.action_location = Some(ShireActionLocation::from(&action_location));
                cond_input = new;
            }
            HobbitHoleKey::Variables => {
                let (new, vars) = fold_many0(
                    terminated(parse_variable, multispace0),
                    HashMap::new,
                    |mut acc, (k, v)| {
                        acc.insert(k, v);
                        acc
                    },
                )(value_input)?;

                hole.variables = vars;
                cond_input = new;
            }
        }

        let (new_input, _) = multispace0(cond_input)?;
        cond_input = new_input;

        // 检查 input 是否以 `---` 开头
        if cond_input.starts_with("---") || cond_input.is_empty() {
            break;
        }
    }

    let (input, _) = parse_frontmatter_end(cond_input)?;
    Ok((input, hole))
}

// Parser for the entire file
fn parse_file(input: &str) -> IResult<&str, ShireFile> {
    let (input, variables) = parse_hobbit_hole(input)?;
    let (input, body) = many1(parse_string)(input)?; // Simplified for demonstration
    /// collect Variable Table in body
    Ok((input, ShireFile { hobbit: variables, body }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_parse_multiple_frontmatter_config() {
        let input = r#"
---
name: "Summary"
description: "Generate Summary"
interaction: AppendCursor
actionLocation: ContextMenu
variables:
  "var1": "demo"
---
"#;

        let result = parse_hobbit_hole(input);
        assert_eq!(
            result,
            Ok((
                "",
                HobbitHole {
                    name: "Summary".to_string(),
                    description: Some("\"Generate Summary\"".to_string()),
                    interaction: Some(InteractionType::AppendCursor),
                    action_location: Some(ShireActionLocation::ContextMenu),
                    variables: vec![("var1".to_string(), VariableTransform::String("demo".to_string()))]
                        .into_iter()
                        .collect()
                }
            ))
        );
    }

    #[test]
    fn test_parse_regex_block() {
        assert_eq!(
            parse_pattern_actions("/.*.java/ { grep(\"error.log\") | sort | xargs(\"rm\") }"),
            Ok((
                "",
                VariableTransform::PatternAction {
                    pattern: ".*.java".to_string(),
                    command: Function::Functions(vec![
                        ("grep".to_string(), vec!["error.log".to_string()]),
                        ("sort".to_string(), vec![]),
                        ("xargs".to_string(), vec!["rm".to_string()])
                    ])
                }
            ))
        );
    }

    #[test]
    fn multiple_vars() {
        let input = r#"
---
variables:
  "var1": "demo"
  "var1": 42
  "var2": /.*.java/ { grep("error.log") | sort | xargs("rm")}
---
"#
            ;

        assert_eq!(
            parse_hobbit_hole(input),
            Ok((
                "",
                HobbitHole {
                    name: "".to_string(),
                    description: None,
                    interaction: None,
                    action_location: None,
                    variables: vec![
                        ("var1".to_string(), VariableTransform::String("demo".to_string())),
                        ("var1".to_string(), VariableTransform::Integer(42)),
                        ("var2".to_string(), VariableTransform::PatternAction {
                            pattern: ".*.java".to_string(),
                            command: Function::Functions(vec![
                                ("grep".to_string(), vec!["error.log".to_string()]),
                                ("sort".to_string(), vec![]),
                                ("xargs".to_string(), vec!["rm".to_string()])
                            ])
                        })
                    ].into_iter().collect()
                }
            ))
        );
    }

    #[test]
    fn parse_block() {
        let input = r#"
---
variables:
  "var2": /.*.java/ { grep("error.log") | sort | xargs("rm")}
---

$var1
"#;

        assert_eq!(
            parse_file(input),
            Ok((
                "\n",
                ShireFile {
                    hobbit: HobbitHole {
                        name: "".to_string(),
                        description: None,
                        interaction: None,
                        action_location: None,
                        variables: vec![
                            ("var2".to_string(), VariableTransform::PatternAction {
                                pattern: ".*.java".to_string(),
                                command: Function::Functions(vec![
                                    ("grep".to_string(), vec!["error.log".to_string()]),
                                    ("sort".to_string(), vec![]),
                                    ("xargs".to_string(), vec!["rm".to_string()])
                                ])
                            })
                        ].into_iter().collect()
                    },
                    body: vec!["$var1".to_string()]
                }
            ))
        );
    }

    /// ```shire
    /// ---
    /// variables:
    ///   "var1": "demo"
    ///   "var2": /.*.java/ { grep("error.log") | sort | xargs("rm")}
    ///   "var3": /.*.log/ {
    ///     case "$0" {
    ///       "error" { grep("ERROR") | sort | xargs("notify_admin") }
    ///       "warn" { grep("WARN") | sort | xargs("notify_admin") }
    ///       "info" { grep("INFO") | sort | xargs("notify_user") }
    ///       default  { grep("ERROR") | sort | xargs("notify_admin") }
    ///     }
    ///   }
    ///   "var4": 42
    /// ---
    /// ```
    #[test]
    fn test_parse_case_block() {
        let input = r#"
---
variables:
  "log": /.*.log/ {
    case "$0" {
      "error" { grep("ERROR") | sort | xargs("notify_admin") }
      "warn" { grep("WARN") | sort | xargs("notify_admin") }
      "info" { grep("INFO") | sort | xargs("notify_user") }
      default  { grep("ERROR") | sort | xargs("notify_admin") }
    }
  }
---
"#;

        let result = parse_file(input);
        println!("{:?}", result);
    }
}
