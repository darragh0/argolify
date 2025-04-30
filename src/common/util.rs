use colored::Colorize;
use std::process::exit;

pub fn print_syntax_err(msg: &str, err: Option<String>, line: usize, col: usize, path: &str) {
    let prefix = "[syntax error]".red().bold();

    let loc = format!("({line}:{col})").yellow().bold();
    let file = format!("in `{path}`").bright_yellow().dimmed();

    let msg = if let Some(e) = err {
        format!("{msg}: {e}")
    } else {
        msg.to_string()
    };

    println!("{prefix} {loc} {file}\n {} {msg}", "⤷".red());
    exit(1);
}
