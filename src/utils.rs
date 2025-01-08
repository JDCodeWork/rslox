pub fn error(line: isize, error_type: ErrorType) -> ErrorType {
    let message = error_type.to_string();
    report(line, "", message);
    error_type
}

fn report(line: isize, where_is: &str, message: String) {
    eprintln!("[ line {} ] Error {}: {}", line, where_is, message);
}

#[derive(thiserror::Error, Debug)]
pub enum ErrorType {
    #[error("Unexpected character.")]
    UnexpectedChar,
}

pub type Result = std::result::Result<(), ErrorType>;
