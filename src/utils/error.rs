use std::fmt;
use std::error::Error;
use std::path::PathBuf;

pub type GtrResult<T> = std::result::Result<T, GtrError>;

#[derive(Debug, Clone)]
pub struct GtrError {
    message: String
}

impl GtrError {
    fn new(message: String) -> Self {
        GtrError { message }
    }
}

impl Error for GtrError {}

impl fmt::Display for GtrError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

pub trait GitError {
    fn not_git_repo(dir: &PathBuf) -> Self;
    fn command_failed(e: Box<dyn Error>) -> Self;
    fn ignore_failed(e: Box<dyn Error>) -> Self;
    fn pack_read_failed(e: Box<dyn Error>) -> Self;
    fn pack_write_failed(e: Box<dyn Error>) -> Self;
}

impl GitError for GtrError {
    fn not_git_repo(dir: &PathBuf) -> GtrError {
        GtrError::new(format!("{} is not a git repository", dir.as_path().display()))
    }

    fn command_failed(e: Box<dyn Error>) -> GtrError {
        GtrError::new(format!("Error running git command, {e:#?}"))
    }

    fn ignore_failed(e: Box<dyn Error>) -> Self {
        GtrError::new(format!("Error persisting git ignore, {e:#?}"))
    }

    fn pack_read_failed(e: Box<dyn Error>) -> Self {
        GtrError::new(format!("Error requesting pack file: {:?}", e))
    }

    fn pack_write_failed(e: Box<dyn Error>) -> Self {
        GtrError::new(format!("Error reading pack file content: {:?}", e))
    }
 }
