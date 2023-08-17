use std::fmt::Display;

#[derive(Debug)]
pub enum RepoCreateError {
    InvalidData(String),
    Unknown(String),
}

impl Display for RepoCreateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RepoCreateError::InvalidData(msg) => write!(f, "InvalidData: {}", msg),
            RepoCreateError::Unknown(msg) => write!(f, "Unknown: {}", msg),
        }
    }
}

#[derive(Debug)]
pub enum RepoSelectError {
    SelectParamInvalid(String),
    Unknown(String),
}

impl Display for RepoSelectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RepoSelectError::SelectParamInvalid(msg) => write!(f, "SelectParamInvalid: {}", msg),
            RepoSelectError::Unknown(msg) => write!(f, "Unknown: {}", msg),
        }
    }
}

#[derive(Debug)]
pub enum RepoFindAllError {
    NotFound,
    Unknown(String),
}

impl Display for RepoFindAllError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RepoFindAllError::NotFound => write!(f, "NotFound"),
            RepoFindAllError::Unknown(msg) => write!(f, "Unknown: {}", msg),
        }
    }
}

#[derive(Debug)]
pub enum RepoUpdateError {
    NotFound,
    Unknown(String),
}

impl Display for RepoUpdateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RepoUpdateError::NotFound => write!(f, "NotFound"),
            RepoUpdateError::Unknown(msg) => write!(f, "Unknown: {}", msg),
        }
    }
}

#[derive(Debug)]
pub enum RepoDeleteError {
    NotFound,
    InvalidData(String),
    Unknown(String),
}

impl Display for RepoDeleteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RepoDeleteError::NotFound => write!(f, "NotFound"),
            RepoDeleteError::Unknown(msg) => write!(f, "Unknown: {}", msg),
            RepoDeleteError::InvalidData(msg) => write!(f, "InvalidData: {}", msg),
        }
    }
}
