use std::{
    error::Error,
    fmt::{Debug, Display, Formatter},
};

use owo_colors::OwoColorize;

pub trait EscuroError: Clone + Error + Debug + Display {
    fn msg(&self) -> &str;
    fn code(&self) -> u32;
    fn line(&self) -> Option<(u32, &str)>;
    fn filename(&self) -> Option<&str>;
}

pub trait ErrorReporter {
    fn report<E: EscuroError>(&self, err: E);
}

pub struct CLIErrorReporter;

impl ErrorReporter for CLIErrorReporter {
    fn report<E: EscuroError>(&self, err: E) {
        if let Some((n, l)) = err.line() {
            println!(
                "{} {:04}: {}",
                "error".bright_red().bold(),
                format!("ESC{}", err.code()).bold(),
                err.msg()
            );
            println!("{} todo filename", "-->".bold().bright_cyan());

            // 4 digits ought to be enough for anyone
            if n < 100 {
                println!("{}", "    |".bold().bright_cyan());
                println!("{} {}", format!(" {:02} |", n).bold().bright_cyan(), l);
                println!("{}", "    |".bold().bright_cyan());
            } else if n < 1000 {
                println!("{}", "     |".bold().bright_cyan());
                println!("{} {}", format!(" {:03} |", n).bold().bright_cyan(), l);
                println!("{}", "     |".bold().bright_cyan());
            } else if n < 10000 {
                println!("{}", "      |".bold().bright_cyan());
                println!("{} {}", format!(" {:04} |", n).bold().bright_cyan(), l);
                println!("{}", "      |".bold().bright_cyan());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tokenizer::{TokenizerError, TokenizerErrorType};

    use super::{CLIErrorReporter, ErrorReporter};
    fn get_reporter() -> impl ErrorReporter {
        CLIErrorReporter
    }
    #[test]
    fn err_report() {
        let err = TokenizerError::new(
            TokenizerErrorType::UnexpectedCharacter,
            "some text".into(),
            1,
        );

        let reporter = get_reporter();

        reporter.report(err);
    }
}
