use std::collections::HashMap;
use std::fmt::Debug;

// FrontMatterType 枚举定义
#[derive(Debug)]
pub enum FrontMatterType {
    STRING(String),
    NUMBER(i32),
    DATE(String),
    BOOLEAN(bool),
    ERROR(String),
    EMPTY,
    ARRAY(Vec<FrontMatterType>),
    OBJECT(HashMap<String, FrontMatterType>),
    PATTERN(RuleBasedPatternAction),
    CASE_MATCH(HashMap<String, FrontMatterType>),
    VARIABLE(String),
    EXPRESSION(Box<dyn Statement>),
    IDENTIFIER(String),
    QUERY_STATEMENT(ShirePsiQueryStatement),
}

impl FrontMatterType {
    // display 方法实现
    pub fn display(&self) -> String {
        match self {
            FrontMatterType::STRING(value) => format!("\"{}\"", value),
            FrontMatterType::NUMBER(value) => value.to_string(),
            FrontMatterType::DATE(value) => value.to_string(),
            FrontMatterType::BOOLEAN(value) => value.to_string(),
            FrontMatterType::ERROR(value) => value.to_string(),
            FrontMatterType::EMPTY => "".to_string(),
            FrontMatterType::ARRAY(value) => {
                let elements: Vec<String> = value.iter().map(|e| e.display()).collect();
                format!("[{}]", elements.join(", "))
            }
            FrontMatterType::OBJECT(value) => {
                let elements: Vec<String> = value.iter()
                    .map(|(k, v)| format!("\"{}\": {}", k, v.display()))
                    .collect();
                format!("{{{}}}", elements.join(", "))
            }
            FrontMatterType::PATTERN(value) => format!("{} -> {}", value.pattern, value.processors.iter().map(|p| p.func_name.clone()).collect::<Vec<_>>().join(", ")),
            FrontMatterType::CASE_MATCH(value) => {
                let elements: Vec<String> = value.iter().map(|(k, v)| {
                    let pattern = if let FrontMatterType::PATTERN(pattern) = v {
                        pattern.pattern.clone()
                    } else {
                        "".to_string()
                    };
                    let processors = if let FrontMatterType::PATTERN(pattern) = v {
                        pattern.processors.iter().map(|p| p.func_name.clone()).collect::<Vec<_>>().join(" | ")
                    } else {
                        "".to_string()
                    };
                    format!("\"{}\" {{ {} }}", k, processors)
                }).collect();
                format!("case \"$0\" {{\n{}\n}}", elements.join("\n"))
            }
            FrontMatterType::VARIABLE(value) => format!("${}", value),
            FrontMatterType::EXPRESSION(statement) => statement.display(),
            FrontMatterType::IDENTIFIER(value) => value.to_string(),
            FrontMatterType::QUERY_STATEMENT(query_statement) => query_statement.to_string(),
        }
    }

    // to_value 方法实现
    pub fn to_value(&self) -> &dyn std::any::Any {
        match self {
            FrontMatterType::STRING(value) => value,
            FrontMatterType::NUMBER(value) => value,
            FrontMatterType::DATE(value) => value,
            FrontMatterType::BOOLEAN(value) => value,
            FrontMatterType::ERROR(value) => value,
            FrontMatterType::EMPTY => &"",
            FrontMatterType::ARRAY(value) => value,
            FrontMatterType::OBJECT(value) => value,
            FrontMatterType::PATTERN(value) => value,
            FrontMatterType::CASE_MATCH(value) => value,
            FrontMatterType::VARIABLE(value) => value,
            FrontMatterType::EXPRESSION(statement) => statement,
            FrontMatterType::IDENTIFIER(value) => value,
            FrontMatterType::QUERY_STATEMENT(query_statement) => query_statement,
        }
    }
}

#[derive(Debug)]
pub struct RuleBasedPatternAction {
    pattern: String,
    processors: Vec<Processor>,
}

#[derive(Debug)]
pub struct Processor {
    func_name: String,
}

pub trait Statement {
    fn display(&self) -> String;
}

impl Debug for dyn Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Statement Debug")
    }
}

#[derive(Debug)]
pub struct ShirePsiQueryStatement;

impl std::fmt::Display for ShirePsiQueryStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "QueryStatement Display")
    }
}
