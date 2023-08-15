#[derive(Debug)]
pub enum RepoCreateError {
    InvalidData(String),
    Unknown(String),
}

#[derive(Debug)]
pub enum RepoSelectError {
    SelectParamInvalid(String),
    Unknown(String),
}

#[derive(Debug)]
pub enum RepoFindAllError {
    NotFound,
    Unknown(String),
}

#[derive(Debug)]
pub enum RepoUpdateError {
    NotFound,
    Unknown(String),
}

#[derive(Debug)]
pub enum RepoDeleteError {
    NotFound,
    InvalidData(String),
    Unknown(String),
}
