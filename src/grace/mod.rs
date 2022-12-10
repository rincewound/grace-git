use serde::{Serialize, Deserialize};

pub mod project;
pub mod package;
mod git;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Registry
{
    pub uri: String
}
