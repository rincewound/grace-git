
use std::{process::{Command, ExitStatus, Output}, io::Write};

pub struct GitClient
{
    cwd: String
}


impl GitClient
{
    pub fn create() -> Self
    {
        Self
        {
            cwd: ".".to_string()
        }
    }

    fn process_output(&self, o: Output) -> bool
    {        
        std::io::stdout().write_all(&o.stdout).unwrap();
        std::io::stderr().write_all(&o.stderr).unwrap();
        return o.status.success();
    }

    pub fn cwd(mut self, cwd: String) -> Self
    {
        self.cwd = cwd;
        self
    }

    pub fn fetch(self) -> Self
    {
        let mut git = Command::new("git");

        self.process_output(git.current_dir(self.cwd.clone())
            .args(["fetch".to_string()])
            .output()
            .expect("FETCH failed"));
        self
    }


    pub fn clone(self, uri: String) -> Self
    {
        let mut git = Command::new("git");

        self.process_output(git.current_dir(self.cwd.clone())
            .args(["clone".to_string(), uri, ".".to_string()])
            .output()
            .expect("CLONE failed"));
        self        
    }

    pub fn pull(self) -> Self
    {
        let mut git = Command::new("git");

        self.process_output(git.current_dir(self.cwd.clone())
            .arg("pull")
            .output()
            .expect("PULL failed"));
        self
            
    }

    pub fn checkout(self, commit_hash: String) -> Self
    {
        let mut git = Command::new("git");

        self.process_output(git.current_dir(self.cwd.clone())
            .args(["checkout".to_string(), commit_hash])
            .output()
            .expect("CHECKOUT failed"));
        self
    }
}
