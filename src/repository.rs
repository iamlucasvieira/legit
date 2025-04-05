use crate::settings::Settings;
use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

// Repository represents a git repository
#[derive(Debug)]
pub struct Repository {
    pub worktree: PathBuf,
    pub gitdir: PathBuf,
    pub settings: Settings,
}

impl Repository {
    /// Find the git directory and returns the Repository instance
    pub fn find(path: Option<&Path>) -> Result<Repository> {
        let path = match path {
            Some(p) => p.to_owned(),
            None => {
                let current_dir = env::current_dir().context("Failed to get current directory")?;
                current_dir.to_owned()
            }
        };

        let gitdir = path.join(".git");
        if !gitdir.exists() {
            let parent = path
                .parent()
                .ok_or_else(|| anyhow::anyhow!("No parent directory"))?;
            return Repository::find(Some(parent));
        }
        let settings = Settings::new()?;
        Ok(Repository {
            worktree: path.to_owned(),
            gitdir,
            settings,
        })
    }

    /// Create a new Repository instance
    pub fn new(path: &Path) -> Result<Repository> {
        let worktree = path.to_owned();
        let gitdir = worktree.join(".git");
        let settings = Settings::new()?;

        Ok(Repository {
            worktree,
            gitdir,
            settings,
        })
    }

    /// Populate the git directory with the necessary files and directories
    pub fn create(&self) -> Result<()> {
        let version = self.settings.core.repositoryformatversion;
        if version != 0 {
            anyhow::bail!("Unsupported repositoryformatversion: {}", version);
        }

        if !self.worktree.exists() {
            fs::create_dir_all(&self.worktree)?;
        }

        if self.gitdir.exists() {
            anyhow::bail!("Directory is already a git repository");
        }

        fs::create_dir_all(&self.gitdir)?;

        let dirs = ["branches", "objects", "refs/tags", "refs/heads"];
        for dir in dirs.iter() {
            fs::create_dir_all(self.gitdir.join(dir))?;
        }

        let description = self.gitdir.join("description");
        fs::write(
            description,
            "Unnamed repository; edit this file 'description' to name the repository.",
        )?;

        let head = self.gitdir.join("HEAD");
        fs::write(head, "ref: refs/heads/master\n")?;

        let config = self.gitdir.join("config");
        let config_content = toml::to_string(&self.settings)?;
        fs::write(config, config_content)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_new() {
        let tempdir = TempDir::new().unwrap();
        let repo = Repository::new(tempdir.path());
        assert!(repo.is_ok());
    }

    #[test]
    fn test_create() {
        let tempdir = TempDir::new().unwrap();
        let repo = Repository::new(tempdir.path()).unwrap();
        repo.create().unwrap();
        let expected_dirs = [
            ".git",
            ".git/branches",
            ".git/objects",
            ".git/refs/tags",
            ".git/refs/heads",
        ];

        for dir in expected_dirs.iter() {
            assert!(tempdir.path().join(dir).exists());
        }

        let expected_files = [".git/description", ".git/HEAD", ".git/config"];
        for file in expected_files.iter() {
            assert!(tempdir.path().join(file).exists());
        }
    }

    #[test]
    fn test_find() {
        let tempdir = TempDir::new().unwrap();
        let gitdir = tempdir.path().join(".git");
        fs::create_dir_all(gitdir).unwrap();
        let repo = Repository::find(Some(tempdir.path())).unwrap();
        assert_eq!(repo.worktree, tempdir.path());
    }

    #[test]
    fn test_find_parent() {
        let tempdir = TempDir::new().unwrap();
        let subdir = tempdir.path().join("subdir");
        let gitdir = tempdir.path().join(".git");
        fs::create_dir_all(gitdir).unwrap();
        fs::create_dir_all(&subdir).unwrap();

        let repo = Repository::find(Some(&subdir)).unwrap();
        assert_eq!(repo.worktree, tempdir.path());
    }

    #[test]
    /// Test that finds the git directory of the project itself
    fn test_find_self() {
        let repo = Repository::find(None).unwrap();
        assert!(repo.worktree.ends_with("legit"));
    }
}
