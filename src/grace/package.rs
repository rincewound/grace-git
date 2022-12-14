use std::{
    fs::File,
    io::{self, BufRead, BufReader, Lines, Write},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

use super::{
    git,
    project::{Project, GRACE_PACKAGE_FILE_NAME, GRACE_PACKAGE_LOCK_FILE_NAME},
    semver::SemanticVersion,
};

#[derive(PartialEq, Copy, Clone)]
pub enum VersionSelector {
    /// Versions must match each other
    StrictEquals, // =
    /// found version must be larger or same than requested one
    LargerEquals, // >=
    /// found version must have same minor and major as requested one
    Compatible, // ~=
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PackageVersion {
    pub id: String,
    pub commit_hash: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Package {
    pub name: String,
    pub uri: String,
    pub versions: Vec<PackageVersion>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PackageList {
    pub packagelist: Vec<Package>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PackageDependency {
    pub name: String,
    pub version: String,
    pub uri: String,
    pub commit_hash: String,
}

impl PackageVersion {
    pub fn as_semver(&self) -> SemanticVersion {
        return SemanticVersion::from_string(self.id.clone());
    }
}

impl PackageDependency {
    pub fn get_package_list(path: PathBuf) -> Vec<Self> {
        let mut grace_file = path.clone();
        grace_file.push(GRACE_PACKAGE_LOCK_FILE_NAME);
        let file: File;
        if !grace_file.exists() {
            return vec![];
        } else {
            file = File::open(grace_file).expect(".grace file is missing");
        }

        let reader = BufReader::new(file);
        let data: Vec<Self> =
            serde_json::from_reader(reader).expect("Project configuration is corrupt");
        data
    }

    pub fn store_package_list(path: PathBuf, data: Vec<Self>) {
        let mut package_file = path.clone();
        package_file.push(GRACE_PACKAGE_LOCK_FILE_NAME);
        let mut file =
            File::create(package_file.to_str().unwrap()).expect("Failed to create config file");
        let _ = file.write_all(serde_json::to_string(&data).unwrap().as_bytes());
    }

    /// Process grace-package.txt and install all packages found there
    pub(crate) fn install(path: PathBuf) {
        let mut cfg_file = path.clone();
        cfg_file.push(GRACE_PACKAGE_FILE_NAME);
        let file = File::open(cfg_file).expect(
            format!(
                "No package file ({}) is available.",
                GRACE_PACKAGE_FILE_NAME
            )
            .as_str(),
        );

        for line in io::BufReader::new(file).lines() {
            if let Ok(ln) = line {
                let items: Vec<&str> = ln.split(&[' ']).collect();
                if items.len() != 3 {
                    panic!("Expected a version of form <packag> <selector> <version> (note the whitespace!). Got: {}", ln);
                }

                let version = SemanticVersion::from_string(items[2].to_string());

                let version_selector = match items[1] {
                    ">=" => VersionSelector::LargerEquals,
                    "=" => VersionSelector::StrictEquals,
                    "~=" => VersionSelector::Compatible,
                    _ => panic!("Invalid version selector!"),
                };

                println!("Installing package {}", items[0].to_string());
                let dep = Self::add_package(
                    path.clone(),
                    items[0].to_string(),
                    version_selector,
                    version,
                );

                Self::install_single_dependency(path.clone(), dep);
            }
        }
    }

    fn install_single_dependency(path: PathBuf, dep: PackageDependency) {
        let repo = dep.uri;
        let commit = dep.commit_hash;
        let mut target_dir = path.clone();
        target_dir.push("packages");
        let mut package_dir = target_dir.clone();
        package_dir.push(dep.name);

        let git = git::GitClient::create();
        git.cwd(target_dir.to_str().unwrap().to_string())
            .clone(repo)
            .cwd(package_dir.to_str().unwrap().to_string())
            .fetch()
            .checkout(commit);
    }

    pub fn add_package(
        path: PathBuf,
        package_name: String,
        version_selector: VersionSelector,
        package_version: SemanticVersion,
    ) -> PackageDependency {
        let p = Project::open(path.clone());
        if let Some(package) = p.resolve_package(
            package_name.clone(),
            package_version.clone(),
            version_selector,
        ) {
            let this = PackageDependency::get_package_list(path.clone());

            let mut actual_list: Vec<PackageDependency> = this
                .into_iter()
                .filter(|x| x.name != package_name)
                .collect();

            let dep = PackageDependency {
                name: package_name,
                version: package.0.id,
                uri: package.1,
                commit_hash: package.0.commit_hash,
            };

            actual_list.push(dep.clone());
            Self::store_package_list(path.clone(), actual_list);
            return dep;
        } else {
            panic!(
                "The package {} in version {} is not available in your registries ",
                package_name, package_version
            );
        }
    }
}
