// TODO: Contain positon where error occured?
#[derive(Debug, Clone)]
pub enum ParseError {
    WrongToken { message: String },
    EndOfStream,
}
