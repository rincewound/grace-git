use std::{path::PathBuf, io::BufReader};
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use crate::grace::package::PackageList;

use super::{Registry, git};
use super::package::{Package, PackageVersion};
use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Debug)]
pub struct Project
{
    pub registries: Vec<Registry>,

    #[serde(skip)] 
    project_dir: PathBuf
}

// This folder is used to holde checked out registry data.
pub const GRACE_ROOT_FOLDER : &str = ".grace";
pub const GRACE_PROJECT_FILE_NAME: &str = "grace-config.json";
pub const GRACE_PACKAGE_FILE_NAME: &str = "grace.json";
pub const GRACE_PACKAGE_LOCK_FILE_NAME: &str = "grace-lock.json";


impl Project
{

    fn uri_to_directory(uri: String) -> String
    {
        let reg_dir = uri.clone()
        .replace(":", "_")
        .replace("/", "_")
        .replace("\\", "_");

        reg_dir
    }

    pub fn init(path: PathBuf) -> Self
    {
        let mut grace_dir = path.clone();
        grace_dir.push(GRACE_ROOT_FOLDER);
        if grace_dir.exists()
        {
            panic!("This already seems to be a grace project.")
        }

        std::fs::create_dir(grace_dir.clone()).expect("Cannot create .grace dir");
    
        let result = Self
        {
            registries: vec![],
            project_dir: path.clone()
        };

        let mut cfg_file = grace_dir.clone();
        cfg_file.push(GRACE_PROJECT_FILE_NAME);
        let mut file = File::create(cfg_file.to_str().unwrap()).expect("Failed to create config file");
        let _= file.write_all(serde_json::to_string(&result).unwrap().as_bytes());

        result
    }

    pub fn open(path: PathBuf) -> Self
    {
        let mut grace_dir = path.clone();
        grace_dir.push(GRACE_ROOT_FOLDER);
        if !grace_dir.exists()
        {
            panic!("This is not a grace project.")
        }

        let mut gpath = grace_dir.clone();
        gpath.push(GRACE_PROJECT_FILE_NAME);
        
        let file  = File::open(gpath).expect(".grace-config file is missing");
        let reader = BufReader::new(file);
        let mut data: Project = serde_json::from_reader(reader).expect("Project configuration is corrupt");
        data.project_dir = path.clone();
        data

    }

    pub fn add_registry(mut self, registry: String)
    {
        let r = Registry {uri: registry};
        self.registries.push(r.clone());
        self.update_registry(&r);

        let mut cfg_file = self.project_dir.clone();
        cfg_file.push(GRACE_ROOT_FOLDER);
        cfg_file.push(GRACE_PROJECT_FILE_NAME);
        let mut file = OpenOptions::new().write(true).open(cfg_file).expect("Failed to open config file");
        let _= file.write_all(serde_json::to_string(&self).unwrap().as_bytes());                 
    }


    pub fn update_registries(&self)
    {
        // checkout all registries
        for r in self.registries.iter()
        {
            self.update_registry(r);
        }

    }

    fn update_registry(&self, r: &Registry) {
        println!("updating registry {}", r.uri.clone());

        let mut registry_dir = self.project_dir.clone();
        registry_dir.push(GRACE_ROOT_FOLDER);

        // sanitize path:
        let reg_dir = Self::uri_to_directory(r.uri.clone());

        registry_dir.push(reg_dir);

        if !registry_dir.exists()
        {
            println!("this is a new registry.");
            let _= std::fs::create_dir(registry_dir.clone());
            let git = git::GitClient::create();
            git.cwd(registry_dir.clone().to_str().unwrap().to_string()).clone(r.uri.clone());
        }

        let registry_content = std::fs::read_dir(registry_dir.clone()).unwrap();
        let mut ok = false;
        for content in registry_content
        {
            if let Ok(entry) = content
            {
                if entry.file_type().unwrap().is_dir()
                {
                    let git = git::GitClient::create();
                    // slightly wrong, git will checkout into a subfolder!                    
                    
                    git.cwd(entry.path().to_str().unwrap().to_string())
                        .fetch()
                        .checkout("master".to_string())
                        .pull();
                    ok = true;
                    break;
                }
            }
        }

        if !ok
        {
            println!("..failed.")
        }

    }

    pub(crate) fn resolve_package(&self, package_name: String, package_version: String) -> Result<PackageVersion, bool> {        
        println!("Checking registries for package {}", package_name);
        for r in self.registries.iter()
        {
            println!("...{}", r.uri);
            let mut registry_dir = self.project_dir.clone();
            registry_dir.push(GRACE_ROOT_FOLDER);
            registry_dir.push(Self::uri_to_directory(r.uri.clone()));       // ToDo: This should be sanitized!    

            if registry_dir.exists()
            {                                
                registry_dir.push("index.json");                
                let file  = File::open(registry_dir);
                
                if file.is_err()
                {
                    continue;
                }

                let reader = BufReader::new(file.unwrap());
                let packages: PackageList = serde_json::from_reader(reader).expect("Malformed index.json");
                
                for package in packages.packagelist.iter()
                {
                    if package.name == package_name
                    {
                        for version in package.versions.iter()
                        {
                            // ToDo: Chheck for SemVer compatibility!
                            if version.id == package_version
                            {
                                return Ok(version.clone());
                            }
                        }
                    }
                }

            }
        }

        Err(false)
    }

}

