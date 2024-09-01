// 使用derive宏自动生成调试信息
#[derive(Debug)]
pub enum PatternActionFunc {
    /// Prompt variant for displaying a message prompt.
    Prompt { message: String },

    /// Grep variant for searching with one or more patterns.
    Grep { patterns: Vec<String> },

    /// Sed variant for find and replace operations.
    Sed {
        pattern: String,
        replacements: String,
        is_regex: bool,
    },

    /// Sort variant for sorting with one or more arguments.
    Sort { arguments: Vec<String> },

    /// Uniq variant for removing duplicates based on one or more arguments.
    Uniq { texts: Vec<String> },

    /// Head variant for retrieving the first few lines.
    Head { number: usize },

    /// Tail variant for retrieving the last few lines.
    Tail { number: usize },

    /// Xargs variant for processing one or more variables.
    Xargs { variables: Vec<String> },

    /// Print variant for printing one or more texts.
    Print { texts: Vec<String> },

    /// Cat variant for concatenating one or more files.
    Cat { paths: Vec<String> },

    /// From variant for selecting one or more elements.
    From { variables: Vec<VariableElement> },

    /// Where variant for filtering elements.
    Where { statement: Statement },

    /// Select variant for ordering elements.
    Select { statements: Vec<Statement> },

    /// Execute a shire script
    ExecuteShire {
        filename: String,
        variable_names: Vec<String>,
    },

    /// Use IDE Notify
    Notify { message: String },

    /// Case Match
    CaseMatch { key_value: Vec<CaseKeyValue> },

    /// Splitting
    Splitting { paths: Vec<String> },

    /// Embedding text
    Embedding { entries: Vec<String> },

    /// Searching text
    Searching {
        text: String,
        threshold: f64,
    },

    /// Caching semantic
    Caching { text: String },

    /// Reranking the result
    Reranking { r#type: String },

    /// The Redact variant for handling sensitive data by applying a specified redaction strategy.
    Redact { strategy: String },

    /// The Crawl variant is used to crawl a list of URLs, get markdown from HTML and save it to a file.
    Crawl { urls: Vec<String> },

    /// The Capture variant used to capture file by NodeType
    Capture {
        file_name: String,
        node_type: String,
    },

    /// The Thread variant will run the function in a new thread
    Thread {
        file_name: String,
        variable_names: Vec<String>,
    },

    /// The JsonPath variant will parse the JSON and get the value by JSONPath
    JsonPath {
        obj: Option<String>,
        path: String,
    },

    /// User Custom Functions
    ToolchainFunction {
        func_name: String,
        args: Vec<String>,
    },
}

impl std::fmt::Display for PatternActionFunc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PatternActionFunc::ToolchainFunction { func_name, args } => {
                write!(f, "{}({})", func_name, args.join(", "))
            }
            _ => write!(f, "{:?}", self),
        }
    }
}

// Placeholder structs to match the Kotlin code
#[derive(Debug)]
pub struct VariableElement; // Replace with actual implementation

#[derive(Debug)]
pub struct Statement; // Replace with actual implementation

#[derive(Debug)]
pub struct CaseKeyValue; // Replace with actual implementation
