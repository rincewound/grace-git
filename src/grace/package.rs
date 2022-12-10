use std::{path::PathBuf, fs::File, io::BufReader};

use serde::{Serialize, Deserialize};

use super::project::Project;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PackageVersion
{
    pub name: String,
    pub commit_hash: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Package
{
    pub name: String,
    pub versions: Vec<PackageVersion>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PackageDependency
{
    pub name: String,
    pub version: String
}


impl PackageDependency
{
    pub fn get_package_list(path: PathBuf) -> Vec<Self>
    {

        let mut grace_file = path.clone();
        grace_file.push(".grace");
        let file  = File::open(grace_file).expect(".grace file is missing");
        let reader = BufReader::new(file);
        let data: Vec<Self> = serde_json::from_reader(reader).expect("Project configuration is corrupt");
        data
    }

    pub fn store_package_list(data: Vec<Self>)
    {

    }

    pub fn add_package(path: PathBuf, package_name: String, package_version: String)
    {
        let p = Project::open(path.clone());
        if let Ok(package) = p.resolve_package(package_name.clone(), package_version.clone())
        {
            
            let mut this = PackageDependency::get_package_list(path);
            this.push(PackageDependency {
                name: package_name,
                version: package_version
            });

            Self::store_package_list(this);
            
        }
        else {
            panic!("The package {} in version {} is not available in your registries ", package_name, package_version);
        }
    }
}