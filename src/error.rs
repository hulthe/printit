use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    SshError(#[from] ssh2::Error),

    #[error("I/O Error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("{0}")]
    UserError(String),
}

impl Error {
    pub fn user<M: ToString>(message: M) -> Self {
        Error::UserError(message.to_string())
    }
}
