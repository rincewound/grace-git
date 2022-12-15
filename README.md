# Grace - Your Git Nanny

Grace is a Git based package manager that allows you to
* Specify versions of packages you need
* Have an easily usable registry
* Is language agnostic
* Allows in project editing of packages

Think Cargo + VCPKG

Registry Format
* The registry is a git repository that contains data about all packages available
* The registry does not host packages itself, instead it only contains meta informations such as
    * Versions
    * URLs (..from where to get the actual package)
    * Git Commit Ids

At its core the registry is just a large JSON File, with the following format
```json
{
    "packagelist": [
        {
            "name": "APackage",
            "uri": "E:/code/grace/SomePackage",
            "versions": [
                {
                    "id": "0.1.0",
                    "commit_hash": "a116c8f923485c34c6f1fa9ad7a2ba459b46212a"
                },
                {
                    "id": "1.0.1",
                    "commit_hash": "a116c8f923485c34c6f1fa9ad7a2ba459b46212a"
                }
            ]
        }
    ]
}
```

The local file ".grace-config" contains registry URLs
The local fille ".grace" contains the packages used in the project. Syntax:
<PackageName><Operator><SemVer>

* Where PackageName is the namee of the package as written in the registry
* Operator is one of:
    * =
    * > =
    * <=
    * ~=
    (see https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html for details)  


## The Grace CLI

### Project Commands
#### init
Setup the current directory as root of a Grace enabled project.

### Registry Commands
#### add
Adds a new registry URL to the project. Example
`grace registry add https://foo.bar` 

Local, windows
`grace registry add c:/blah/blubb` 
Local linux
`grace registry add ~/blah/blubb` 

#### update
Fetches the indexfile from all registries (or updates it!)
Example
`grace registry update`

#### remove
Remove a registry
`grace registry remove https://foo.bar` 

-> Removing a registry will not remove packages associated with the registry

### Package Commands

#### install
Processes the .grace file and proceed to clone all packages, that are
mentioned there. The update is recursive, i.e. if packages in turn
contain .grace files the update command is applied there as well.

Example:
`grace package install`

Note that Grace will only update if the package version in the .grace file
changed.

#### publish
Will publish a package to the registry it originated from, IF the commithash
changed.
Example
`grace package publish APackage/1.0.1`

This will:
* Check the contents of APackage and get the commit id via git status
* If the commit id has changed since the last "grace update" it will:
    * Check, from which registry the package originated
    * Update the registry index (i.e. git pull!)
    * Open the index and check if that version already exists
        * If yes, the operation is aborted
        * If no, Grace will add the new version to the index, commit and push it.
    * Publish is always an atomic operation. If a conflict occurs, grace will restart the
      process by repulling the index.

If the package was not yet found in any registry an additional parameter is required, e.g.
`grace package publish APackage/1.0.0 https://i-am-a-registry.com`

#### clean
#### remove

## Anatomy of a Grace Project

```
<root>
 +-----[.grace]
 |          +-----.grace-config
 |          +-----[registry1]
 |          |        +-----index.json
 |          +-----[registry2]
 |                   +-----index.json
 +-----[cache-dir]
 |          +----[package1]
 |          +----[package2]
 +-----grace-packages.txt
 +-----grace-lock.json
```

The cache dir contains all downloaded packages. Grace allows to configure the location of the cache dir on a per-project basis by setting the "cache-dir" property in the .grace-config file. Note that subprojects will not inherit this property but instead use their own setting.


## Using Grace
Grace makes no assumptions as to the kind of projects it is used with. It just uses git to checkout
packages and place them into the package directory.