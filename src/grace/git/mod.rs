use std::{
    io::Write,
    process::{Command, ExitStatus, Output},
};

pub struct GitClient {
    cwd: String,
    silent: bool,
    err: bool,
}

impl GitClient {
    pub fn create() -> Self {
        Self {
            cwd: ".".to_string(),
            silent: false,
            err: false,
        }
    }

    fn process_output(&self, o: Output) -> bool {
        if !self.silent {
            std::io::stdout().write_all(&o.stdout).unwrap();
            std::io::stderr().write_all(&o.stderr).unwrap();
        }
        return o.status.success();
    }

    pub fn cwd(mut self, cwd: String) -> Self {
        self.cwd = cwd;
        self
    }

    pub fn silent(mut self) -> Self {
        self.silent = true;
        self
    }

    pub fn fetch(mut self) -> Self {
        let mut git = Command::new("git");

        self.err = self.process_output(
            git.current_dir(self.cwd.clone())
                .args(["fetch".to_string()])
                .output()
                .expect("FETCH failed"),
        );
        self
    }

    pub fn init(mut self) -> Self {
        let mut git = Command::new("git");

        self.err = self.process_output(
            git.current_dir(self.cwd.clone())
                .args(["init".to_string()])
                .output()
                .expect("INIT failed"),
        );
        self
    }

    pub fn remote(mut self, remote: String) -> Self {
        let mut git = Command::new("git");

        self.err = self.process_output(
            git.current_dir(self.cwd.clone())
                .args([
                    "remote".to_string(),
                    "add".to_string(),
                    "origin".to_string(),
                    remote,
                ])
                .output()
                .expect("INIT failed"),
        );
        self
    }

    pub fn clone(mut self, uri: String, bare: bool) -> Self {
        let mut git = Command::new("git");
        let args = if !bare {
            vec!["clone".to_string(), uri]
        } else {
            vec!["clone".to_string(), uri, ".".to_string()]
        };

        self.err = self.process_output(
            git.current_dir(self.cwd.clone())
                .args(args)
                .output()
                .expect("CLONE failed"),
        );
        self
    }

    pub fn pull(mut self) -> Self {
        let mut git = Command::new("git");

        self.err = self.process_output(
            git.current_dir(self.cwd.clone())
                .args(["pull", "origin", "master"])
                .output()
                .expect("PULL failed"),
        );
        self
    }

    pub fn checkout(mut self, commit_hash: String) -> Self {
        let mut git = Command::new("git");

        self.err = self.process_output(
            git.current_dir(self.cwd.clone())
                .args(["checkout".to_string(), commit_hash])
                .output()
                .expect("CHECKOUT failed"),
        );
        self
    }

    pub(crate) fn err(&self) -> bool {
        self.err
    }
}
