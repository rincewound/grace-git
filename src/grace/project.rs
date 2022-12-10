use std::{path::PathBuf, io::BufReader};
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
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


impl Project
{

    pub fn init(path: PathBuf) -> Self
    {
        // create .grace dir
        // create .grace-config
        // create .grace file

        let mut grace_dir = path.clone();
        grace_dir.push(".grace");
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
        cfg_file.push(".grace-config");
        let mut file = File::create(cfg_file.to_str().unwrap()).expect("Failed to create config file");
        let _= file.write_all(serde_json::to_string(&result).unwrap().as_bytes());

        result
    }

    pub fn open(path: PathBuf) -> Self
    {
        let mut grace_dir = path.clone();
        grace_dir.push(".grace");
        if !grace_dir.exists()
        {
            panic!("This is not a grace project.")
        }

        let mut gpath = grace_dir.clone();
        gpath.push(".grace-config");
        
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
        cfg_file.push(".grace");        // folder
        cfg_file.push(".grace-config"); //file
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
        registry_dir.push(".grace");

        // sanitize path:
        let reg_dir = r.uri.clone()
                                .replace(":", "_")
                                .replace("/", "_")
                                .replace("\\", "_");

        registry_dir.push(reg_dir);

        if !registry_dir.exists()
        {
            println!("this is a new registry.");
            let _= std::fs::create_dir(registry_dir.clone());
            let git = git::GitClient::create();
            git.cwd(registry_dir.clone().to_str().unwrap().to_string()).clone(r.uri.clone());
        }

        let registry_content = std::fs::read_dir(registry_dir.clone()).unwrap();
        for content in registry_content
        {
            if let Ok(entry) = content
            {
                if entry.file_type().unwrap().is_dir()
                {
                    let git = git::GitClient::create();
                    // slightly wrong, git will checkout into a subfolder!                    
                    git.cwd(entry.path().to_str().unwrap().to_string()).checkout("master".to_string());
                    break;
                }
            }
        }

    }

    pub(crate) fn resolve_package(&self, package_name: String, package_version: String) -> Result<PackageVersion, bool> {        
        println!("Checking registries for package {}", package_name);
        for r in self.registries.iter()
        {
            println!("...{}", r.uri);
            let mut registry_dir = self.project_dir.clone();
            registry_dir.push(".grace");
            registry_dir.push(r.uri.clone());       // ToDo: This should be sanitized!    

            if registry_dir.exists()
            {                                
                registry_dir.push("index.json");
                let file  = File::open(registry_dir).expect("index.json is missing");
                let reader = BufReader::new(file);
                let packages: Vec<Package> = serde_json::from_reader(reader).expect("Malformed index.json");
                
                for package in packages.iter()
                {
                    if package.name == package_name
                    {
                        for version in package.versions.iter()
                        {
                            // ToDo: Chheck for SemVer compatibility!
                            if version.name == package_version
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

