use std::{
    error::Error,
    fmt::{Debug, Display},
};

use owo_colors::OwoColorize;

pub trait AnkokuError: Clone + Error + Debug + Display {
    fn msg(&self) -> &str;
    fn code(&self) -> u32;
    fn line_col(&self) -> Option<(u32, usize, &str)>;
    fn length(&self) -> Option<usize>;
    fn filename(&self) -> Option<&str>;
}

pub trait ErrorReporter {
    fn report<E: AnkokuError>(&self, err: E);
}

pub struct CLIErrorReporter;

impl ErrorReporter for CLIErrorReporter {
    fn report<E: AnkokuError>(&self, err: E) {
        if let Some((line, col, content)) = err.line_col() {
            println!(
                "{} {:04}: {}",
                "error".bright_red().bold(),
                format!("ESC{}", err.code()).bold(),
                err.msg()
            );
            // println!("{} todo filename", "-->".bold().bright_cyan());

            let bottom_highlight = || {
                format!(
                    "{}{}",
                    " ".repeat(col - 1),
                    "^".repeat(err.length().unwrap_or(1)).bold().yellow(),
                )
            };
            // 4 digits ought to be enough for anyone
            if line < 100 {
                println!("{}", "    |".bold().bright_cyan());
                println!(
                    "{} {}",
                    format!(" {:2} |", line).bold().bright_cyan(),
                    content
                );
                println!("{} {}", "    |".bold().bright_cyan(), bottom_highlight());
            } else if line < 1000 {
                println!("{}", "     |".bold().bright_cyan());
                println!(
                    "{} {}",
                    format!(" {:3} |", line).bold().bright_cyan(),
                    content
                );
                println!("{} {}", "     |".bold().bright_cyan(), bottom_highlight());
            } else if line < 10000 {
                println!("{}", "      |".bold().bright_cyan());
                println!(
                    "{} {}",
                    format!(" {:4} |", line).bold().bright_cyan(),
                    content
                );
                println!("{} {}", "      |".bold().bright_cyan(), bottom_highlight());
            }
        }
    }
}
