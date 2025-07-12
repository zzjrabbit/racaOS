#[rustfmt::skip]
use alloc::{vec::Vec, vec, string::{String, ToString}};

peg::parser! {
    pub grammar command_parser() for str {
        rule _() = [' ' | '\t']+
            rule new_line_raw() = "\r"? "\n"
            rule comment() = "#" [^'\r' | '\n']*

            rule string() -> String
            = "\"" s:[^'"']* "\"" { s.into_iter().collect() }

        rule non_label_arg() -> String
            = n:$([^' ' | '\t' | '\r' | '\n' | '<' | '>' | '&' | '|' | ';' | '"' | '\'']+) { n.into() }
        / s: string() { s }

        rule label_arg() -> String
            = "-" n:non_label_arg() { alloc::format!("-{}", n) }
            / "--" n:non_label_arg() { alloc::format!("--{}", n) }
            / "--" { alloc::format!("--") }
            / "-" { alloc::format!("-") }

            rule arg() -> String = non_label_arg() / label_arg()

                rule arguments() -> Vec<String>
            = a:(arg() ** _) { a }

        rule ws_arg<T: Default>(r: rule<T>) -> T
            = a:(_ a:r() {a})? _? { a.unwrap_or_default() }

            rule ws_args() -> Vec<String> = ws_arg(<arguments()>)

        rule path() -> String
            = p:$(['a'..='z' | 'A'..='Z' | '0'..='9' | '/' | '.' | '_' | '-' | '~' | ':' | '@' | '&' | '=' | '+' | '$' | '%' | '!' | '*' | '?' | '#' | '^' | '|']+) { p.to_string() }
        / p: string() { p }

        pub rule command() -> Command
        = "echo" a:ws_args() { Command::Echo(a.join(" ")) }
        / a:path() b:ws_args() { Command::Spawn(a, b) }
        / { Command::Empty }
    }
}

#[derive(Debug)]
pub enum Command {
    Echo(String),
    Spawn(String, Vec<String>),
    Empty,
}
