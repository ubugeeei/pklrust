use proc_macro2::{Delimiter, Spacing, TokenTree};

// ---------------------------------------------------------------------------
// Token → PKL string conversion
//
// Rust's tokenizer handles most PKL tokens. The main challenge is that
// newlines (statement separators in PKL) are stripped. We reconstruct them
// by tracking the last emitted token and inserting newlines at statement
// boundaries in brace-delimited contexts.
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq)]
enum Context {
    Brace,
    Paren,
    Bracket,
}

/// What we last emitted, for newline and spacing decisions.
#[derive(Clone, Copy, PartialEq)]
enum Last {
    /// Beginning of output or after a newline
    Start,
    /// An identifier. `continuation` is true for PKL keywords that bind to
    /// the next token on the same line (class, function, local, else, etc.)
    Ident { continuation: bool },
    /// A string/number literal
    Literal,
    /// `)`, `]`, or `}`
    CloseDelim,
    /// A punct with Joint spacing — next punct is part of the same operator
    JointPunct,
    /// A punct with Alone spacing, or after `,`/`:` (which add their own space)
    AlonePunct,
    /// After a punct that suppresses space on the following token (`.`)
    GluePunct,
}

impl Last {
    /// Could a complete expression have just ended?
    fn is_value_like(self) -> bool {
        matches!(
            self,
            Last::Ident { continuation: false } | Last::Literal | Last::CloseDelim
        )
    }
}

/// PKL keywords that bind the following token(s) onto the same line.
/// When the PREVIOUS ident is one of these, no newline before the current token.
/// When the CURRENT ident is one of these and the previous was value-like,
/// keywords marked as "infix" still don't get a newline.
const CONTINUATION_KEYWORDS: &[&str] = &[
    // Modifiers
    "local", "hidden", "fixed", "const", "open", "abstract", "external",
    // Declarations
    "class", "typealias", "function", "module",
    // Expressions / operators
    "new", "as", "is", "in", "not",
    // Control flow
    "if", "else", "for", "when", "let",
    // Module header
    "amends", "extends", "import",
    // Built-ins that take an expression
    "throw", "trace", "read",
];

/// Keywords that can appear *after* a value-like token without starting
/// a new statement. These are infix/postfix operators or continuations.
const INFIX_KEYWORDS: &[&str] = &["else", "is", "as", "in"];

fn is_continuation(s: &str) -> bool {
    CONTINUATION_KEYWORDS.contains(&s)
}

fn is_infix(s: &str) -> bool {
    INFIX_KEYWORDS.contains(&s)
}

pub(crate) fn tokens_to_pkl(trees: &[TokenTree]) -> String {
    emit_tokens(trees, Context::Brace)
}

fn emit_tokens(trees: &[TokenTree], ctx: Context) -> String {
    let mut out = String::new();
    let mut last = Last::Start;

    for tree in trees {
        match tree {
            TokenTree::Ident(id) => {
                let name = id.to_string();

                // Newline between statements in brace context
                if ctx == Context::Brace && last.is_value_like() && !is_infix(&name) {
                    out.push('\n');
                } else if needs_space(last) {
                    out.push(' ');
                }

                out.push_str(&name);
                last = Last::Ident {
                    continuation: is_continuation(&name),
                };
            }

            TokenTree::Literal(lit) => {
                if needs_space(last) {
                    out.push(' ');
                }
                out.push_str(&lit.to_string());
                last = Last::Literal;
            }

            TokenTree::Punct(p) => {
                let ch = p.as_char();
                let spacing = p.spacing();

                if ch == ';' {
                    out.push('\n');
                    last = Last::Start;
                    continue;
                }

                // Space before punct?
                // No space if:
                //  - Previous was Joint (multi-char operator: |>, ==, etc.)
                //  - Previous was Glue (after `.`)
                //  - Start of output
                //  - Current is `.` or `,` (always attach to left)
                let no_space = last == Last::JointPunct
                    || last == Last::GluePunct
                    || last == Last::Start
                    || matches!(ch, '.' | ',' | ':' | '?');

                if !no_space && needs_space(last) {
                    out.push(' ');
                }

                out.push(ch);

                // Determine `last` for the next token
                match ch {
                    ',' => {
                        out.push(' ');
                        last = Last::Start;
                    }
                    ':' => {
                        out.push(' ');
                        last = Last::Start;
                    }
                    '.' => {
                        // `.` glues to the next token (no space after)
                        last = Last::GluePunct;
                    }
                    _ => {
                        last = if spacing == Spacing::Joint {
                            Last::JointPunct
                        } else {
                            Last::AlonePunct
                        };
                    }
                }
            }

            TokenTree::Group(g) => {
                let inner_trees: Vec<TokenTree> = g.stream().into_iter().collect();

                match g.delimiter() {
                    Delimiter::Brace => {
                        if ctx == Context::Brace && last == Last::CloseDelim {
                            out.push('\n');
                        }

                        let inner = emit_tokens(&inner_trees, Context::Brace);
                        out.push_str(" {\n");
                        out.push_str(&inner);
                        if !inner.is_empty() && !inner.ends_with('\n') {
                            out.push('\n');
                        }
                        out.push('}');
                        last = Last::CloseDelim;
                    }
                    Delimiter::Parenthesis => {
                        let inner = emit_tokens(&inner_trees, Context::Paren);
                        out.push('(');
                        out.push_str(&inner);
                        out.push(')');
                        last = Last::CloseDelim;
                    }
                    Delimiter::Bracket => {
                        let inner = emit_tokens(&inner_trees, Context::Bracket);
                        out.push('[');
                        out.push_str(&inner);
                        out.push(']');
                        last = Last::CloseDelim;
                    }
                    Delimiter::None => {
                        let inner = emit_tokens(&inner_trees, ctx);
                        out.push_str(&inner);
                        last = Last::Literal; // approximate
                    }
                }
            }
        }
    }

    out
}

/// Should we add a space before the current token given what came before?
fn needs_space(last: Last) -> bool {
    match last {
        Last::Start | Last::JointPunct | Last::GluePunct => false,
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pkl(input: &str) -> String {
        let stream: proc_macro2::TokenStream = input.parse().unwrap();
        let trees: Vec<TokenTree> = stream.into_iter().collect();
        tokens_to_pkl(&trees)
    }

    #[test]
    fn simple_properties() {
        let result = pkl(r#"host = "localhost"  port = 8080"#);
        assert_eq!(result, "host = \"localhost\"\nport = 8080");
    }

    #[test]
    fn nested_object() {
        let result = pkl(r#"database { url = "pg://localhost" maxConn = 10 }"#);
        assert_eq!(
            result,
            "database {\nurl = \"pg://localhost\"\nmaxConn = 10\n}"
        );
    }

    #[test]
    fn class_definition() {
        let result = pkl(r#"class Server { host: String  port: UInt16 }"#);
        assert_eq!(result, "class Server {\nhost: String\nport: UInt16\n}");
    }

    #[test]
    fn function_definition() {
        let result = pkl(r#"function add(a, b) = a + b"#);
        assert_eq!(result, "function add(a, b) = a + b");
    }

    #[test]
    fn local_modifier() {
        let result = pkl(r#"local basePort = 8080  port = basePort + 1"#);
        assert_eq!(result, "local basePort = 8080\nport = basePort + 1");
    }

    #[test]
    fn import_statement() {
        let result = pkl(r#"import("pkl:json")  name = "test""#);
        assert_eq!(result, "import(\"pkl:json\")\nname = \"test\"");
    }

    #[test]
    fn pipe_operator() {
        let result = pkl(r#"result = items |> filter(it > 0)"#);
        assert_eq!(result, "result = items |> filter(it > 0)");
    }

    #[test]
    fn for_generator() {
        let result = pkl(r#"items { for (x in xs) { name = x } }"#);
        assert!(result.contains("for(x in xs)"), "got: {result}");
    }

    #[test]
    fn when_generator() {
        let result = pkl(r#"when (enabled) { port = 8080 }"#);
        assert!(result.contains("when(enabled)"), "got: {result}");
    }

    #[test]
    fn semicolons_as_newlines() {
        let result = pkl(r#"a = 1; b = 2; c = 3"#);
        assert_eq!(result, "a = 1\nb = 2\nc = 3");
    }

    #[test]
    fn if_else_expression() {
        let result = pkl(r#"port = if (debug) 3000 else 8080"#);
        assert_eq!(result, "port = if(debug) 3000 else 8080");
    }

    #[test]
    fn new_expression() {
        let result = pkl(r#"server = new Server { host = "localhost" }"#);
        assert!(result.contains("new Server"), "got: {result}");
        assert!(result.contains("host = \"localhost\""), "got: {result}");
    }

    #[test]
    fn dot_access() {
        let result = pkl(r#"port = config.server.port"#);
        assert_eq!(result, "port = config.server.port");
    }

    #[test]
    fn null_coalescing() {
        // In Rust tokenizer, `??` → `?` (Joint) `?` (Alone)
        // `?` attaches to the left, so `input??` with space after
        let result = pkl(r#"name = input ?? "default""#);
        assert_eq!(result, "name = input?? \"default\"");
    }

    #[test]
    fn type_annotation() {
        let result = pkl(r#"port: UInt16 = 8080"#);
        assert_eq!(result, "port: UInt16 = 8080");
    }

    #[test]
    fn comparison_operators() {
        let result = pkl(r#"valid = port >= 1024 && port <= 65535"#);
        assert_eq!(result, "valid = port >= 1024 && port <= 65535");
    }

    #[test]
    fn not_equal() {
        let result = pkl(r#"changed = old != new"#);
        assert_eq!(result, "changed = old != new");
    }

    #[test]
    fn multiple_classes() {
        let result = pkl(
            r#"class Server { host: String }  class Database { url: String }"#,
        );
        assert!(result.contains("}\nclass Database"), "got: {result}");
    }

    #[test]
    fn extends_clause() {
        let result = pkl(r#"extends "base.pkl"  name = "child""#);
        assert!(result.starts_with("extends \"base.pkl\""), "got: {result}");
        assert!(result.contains("\nname = \"child\""), "got: {result}");
    }

    #[test]
    fn negative_number() {
        // Rust tokenizer: `-` (Alone) `1` → gets space between. PKL handles `- 1`.
        let result = pkl(r#"offset = -1"#);
        // `-1` or `- 1` are both valid PKL; Rust tokenizer splits them
        assert!(
            result == "offset = -1" || result == "offset = - 1",
            "got: {result}"
        );
    }

    #[test]
    fn list_literal() {
        let result = pkl(r#"items = new Listing { "a" "b" "c" }"#);
        assert!(result.contains("new Listing"), "got: {result}");
    }

    #[test]
    fn is_operator() {
        let result = pkl(r#"check = value is String"#);
        assert_eq!(result, "check = value is String");
    }

    #[test]
    fn as_operator() {
        let result = pkl(r#"num = value as Int"#);
        assert_eq!(result, "num = value as Int");
    }

    #[test]
    fn duration_unit() {
        let result = pkl(r#"timeout = 30.ms"#);
        assert_eq!(result, "timeout = 30.ms");
    }

    #[test]
    fn duration_hour() {
        let result = pkl(r#"ttl = 1.h"#);
        assert_eq!(result, "ttl = 1.h");
    }

    #[test]
    fn data_size_unit() {
        let result = pkl(r#"maxSize = 512.mb"#);
        assert_eq!(result, "maxSize = 512.mb");
    }

    #[test]
    fn duration_in_object() {
        let result = pkl(r#"server { timeout = 30.s  retryDelay = 500.ms }"#);
        assert!(result.contains("timeout = 30.s"), "got: {result}");
        assert!(result.contains("retryDelay = 500.ms"), "got: {result}");
    }

    #[test]
    fn null_safe_access() {
        // `?.` → `?` (Joint) `.` (Alone)
        let result = pkl(r#"name = user?.name"#);
        assert_eq!(result, "name = user?.name");
    }
}
