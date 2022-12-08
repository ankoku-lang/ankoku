use std::{
    error::Error,
    fmt::{Debug, Display},
};

pub trait AnkokuError: Error + Debug + Display {
    fn msg(&self) -> &str;
    fn code(&self) -> u32;
    fn line_col(&self) -> Option<(u32, usize, &str)>;
    fn length(&self) -> Option<usize>;
    fn filename(&self) -> Option<&str>;
}

pub trait ErrorReporter {
    fn report<E: AnkokuError>(&self, err: E);
}
