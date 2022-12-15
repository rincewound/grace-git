use crate::grace::package::PackageList;
use crate::grace::semver::SemanticVersion;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::{io::BufReader, path::PathBuf};

use super::package::{Package, PackageVersion, VersionSelector};
use super::{git, Registry};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Project {
    pub registries: Vec<Registry>,

    #[serde(skip)]
    project_dir: PathBuf,
}

// This folder is used to holde checked out registry data.
pub const GRACE_ROOT_FOLDER: &str = ".grace";
pub const GRACE_PROJECT_FILE_NAME: &str = "grace-config.json";
pub const GRACE_PACKAGE_FILE_NAME: &str = "grace-packages.txt";
pub const GRACE_PACKAGE_LOCK_FILE_NAME: &str = "grace-lock.json";

impl Project {
    fn uri_to_directory(uri: String) -> String {
        let reg_dir = uri
            .clone()
            .replace(":", "_")
            .replace("/", "_")
            .replace("\\", "_");

        reg_dir
    }

    pub fn init(path: PathBuf) -> Self {
        let mut grace_dir = path.clone();
        grace_dir.push(GRACE_ROOT_FOLDER);
        if grace_dir.exists() {
            panic!("This already seems to be a grace project.")
        }

        std::fs::create_dir(grace_dir.clone()).expect("Cannot create .grace dir");

        let mut package_dir = path.clone();
        package_dir.push("packages");
        std::fs::create_dir(package_dir.clone()).expect("Cannot create package dir");

        let result = Self {
            registries: vec![],
            project_dir: path.clone(),
        };

        let mut cfg_file = grace_dir.clone();
        cfg_file.push(GRACE_PROJECT_FILE_NAME);
        let mut file =
            File::create(cfg_file.to_str().unwrap()).expect("Failed to create config file");
        let _ = file.write_all(serde_json::to_string(&result).unwrap().as_bytes());

        result
    }

    pub fn open(path: PathBuf) -> Self {
        let mut grace_dir = path.clone();
        grace_dir.push(GRACE_ROOT_FOLDER);
        if !grace_dir.exists() {
            panic!("This is not a grace project.")
        }

        let mut gpath = grace_dir.clone();
        gpath.push(GRACE_PROJECT_FILE_NAME);

        let file = File::open(gpath).expect(".grace-config file is missing");
        let reader = BufReader::new(file);
        let mut data: Project =
            serde_json::from_reader(reader).expect("Project configuration is corrupt");
        data.project_dir = path.clone();
        data
    }

    pub fn add_registry(mut self, registry: String) {
        let r = Registry { uri: registry };
        self.registries.push(r.clone());
        self.update_registry(&r);

        let mut cfg_file = self.project_dir.clone();
        cfg_file.push(GRACE_ROOT_FOLDER);
        cfg_file.push(GRACE_PROJECT_FILE_NAME);
        let mut file = OpenOptions::new()
            .write(true)
            .open(cfg_file)
            .expect("Failed to open config file");
        let _ = file.write_all(serde_json::to_string(&self).unwrap().as_bytes());
    }

    pub fn update_registries(&self) {
        // checkout all registries
        for r in self.registries.iter() {
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

        if !registry_dir.exists() {
            println!("this is a new registry.");
            let _ = std::fs::create_dir(registry_dir.clone());
            let git = git::GitClient::create();
            git.cwd(registry_dir.clone().to_str().unwrap().to_string())
                .clone(r.uri.clone(), true);
        }

        let git = git::GitClient::create();
        let ok = git.cwd(registry_dir.to_str().unwrap().to_string())
            .silent()
            .init()
            .remote(r.uri.clone())
            .pull()
            .fetch()
            .err();

        if !ok {
            println!("..failed.")
        }
    }

    fn fetch_packages(&self, registry: &Registry) -> Option<PackageList> {
        let mut registry_dir = self.project_dir.clone();
        registry_dir.push(GRACE_ROOT_FOLDER);
        registry_dir.push(Self::uri_to_directory(registry.uri.clone()));

        if !registry_dir.exists() {
            return None;
        }

        registry_dir.push("index.json");
        let file = File::open(registry_dir);

        if file.is_err() {
            return None;
        }

        let reader = BufReader::new(file.unwrap());
        let packages: PackageList = serde_json::from_reader(reader).expect("Malformed index.json");

        return Some(packages);
    }

    pub(crate) fn resolve_package(
        &self,
        package_name: String,
        package_version: SemanticVersion,
        selector: VersionSelector,
    ) -> Option<(PackageVersion, String)> {
        println!("  Checking registries for package {}", package_name);

        let mut found_package: Option<(PackageVersion, String)> = None;
        let sought_version = package_version;

        for r in self.registries.iter() {
            println!("  ...{}", r.uri);

            if let Some(packages) = self.fetch_packages(r) {
                for package in packages
                    .packagelist
                    .iter()
                    .filter(|x| x.name == package_name)
                {
                    for version in package.versions.iter() {
                        if let None = found_package {
                            // this should only ever happen if the found package's version is actually
                            // compatible to the one we've passed in
                            if is_usable_for(&version.as_semver(), &sought_version, selector) {
                                found_package = Some((version.clone(), package.uri.clone()));
                            }
                        }

                        if let Some(p) = found_package.clone() {
                            /*
                                Check if this package version is:
                                * newer than the previously selected one
                                * and still compatible with the input version.
                            */
                            found_package = select_package(
                                package,
                                version,
                                p.0,
                                &sought_version,
                                found_package,
                                selector,
                            )
                        }
                    }
                }
            }
        }
        found_package
    }
}

fn is_usable_for(
    version_a: &SemanticVersion,
    version_b: &SemanticVersion,
    selector: VersionSelector,
) -> bool {
    let compat = version_a.match_to(version_b);
    match compat {
        crate::grace::semver::Compatibility::Breaking => return false,
        crate::grace::semver::Compatibility::Exact => {
            return true;
        }
        crate::grace::semver::Compatibility::Partial => {
            if selector == VersionSelector::LargerEquals || selector == VersionSelector::Compatible
            {
                return true;
            }
        }
        crate::grace::semver::Compatibility::Compatible => {
            if selector == VersionSelector::LargerEquals {
                return true;
            }
        }
    }
    false
}

fn select_package(
    package: &Package,
    new_selected_version: &PackageVersion,
    last_selected_version: PackageVersion,
    sought_version: &SemanticVersion,
    found_package: Option<(PackageVersion, String)>,
    selector: VersionSelector,
) -> Option<(PackageVersion, String)> {
    let found_version = new_selected_version.as_semver();
    let current_version = last_selected_version.as_semver();
    if current_version < found_version {
        let compat = sought_version.match_to(&found_version);

        match compat {
            crate::grace::semver::Compatibility::Breaking => return found_package,
            crate::grace::semver::Compatibility::Exact => {
                return Some((new_selected_version.clone(), package.uri.clone()));
            }
            crate::grace::semver::Compatibility::Partial => {
                if selector == VersionSelector::LargerEquals
                    || selector == VersionSelector::Compatible
                {
                    return Some((new_selected_version.clone(), package.uri.clone()));
                }
            }
            crate::grace::semver::Compatibility::Compatible => {
                if selector == VersionSelector::LargerEquals {
                    return Some((new_selected_version.clone(), package.uri.clone()));
                }
            }
        }
    }

    found_package
}
