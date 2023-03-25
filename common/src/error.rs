use std::fmt::Display;

// TODO: Contain positon where error occured?
#[derive(Debug, Clone)]
pub enum ParseError {
    WrongToken { message: String },
    EndOfStream,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::WrongToken { message } => f.write_str(message),
            ParseError::EndOfStream => f.write_str("End of stream"),
        }
    }
}
