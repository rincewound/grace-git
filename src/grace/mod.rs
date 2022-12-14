use serde::{Deserialize, Serialize};

mod git;
pub mod package;
pub mod project;
pub mod semver;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Registry {
    pub uri: String,
}
