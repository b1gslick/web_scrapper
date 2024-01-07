#[derive(Debug)]
pub enum Error {
    ParseUrl,
    NewNotFound,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self {
            Error::ParseUrl => {
                write!(f, "Cannot parse url")
            }
            Error::NewNotFound => {
                write!(f, "News for keywords not found",)
            }
        }
    }
}
