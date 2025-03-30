mod settings;

use anyhow::Result;
use settings::Settings;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

// Repository represents a git repository
#[derive(Debug)]
pub struct Repository {
    pub worktree: PathBuf,
    pub gitdir: PathBuf,
    pub settings: Settings,
}

impl Repository {
    pub fn new(path: &Path, force: bool) -> Result<Repository> {
        let worktree = path.to_path_buf();
        let gitdir = worktree.join(".git");
        let mut settings_path = None;

        if !force {
            if !gitdir.exists() {
                anyhow::bail!("Not a git repository");
            }

            settings_path = Some(gitdir.join("config"));
        }

        let settings = Settings::new(settings_path.as_deref())?;
        let version = settings.core.repositoryformatversion;

        if !force && version != 0 {
            anyhow::bail!("Unsupported repositoryformatversion: {}", version);
        }

        Ok(Repository {
            worktree,
            gitdir,
            settings,
        })
    }

    pub fn create(path: &Path) -> Result<Repository> {
        let repo = Repository::new(path, true)?;

        if !repo.worktree.exists() {
            fs::create_dir_all(&repo.worktree)?;
        }

        if repo.gitdir.exists() {
            anyhow::bail!("Directory is already a git repository");
        }

        fs::create_dir_all(&repo.gitdir)?;

        let dirs = ["branches", "objects", "refs/tags", "refs/heads"];
        for dir in dirs.iter() {
            fs::create_dir_all(repo.gitdir.join(dir))?;
        }

        let description = repo.gitdir.join("description");
        fs::write(
            description,
            "Unnamed repository; edit this file 'description' to name the repository.",
        )?;

        let head = repo.gitdir.join("HEAD");
        fs::write(head, "ref: refs/heads/master\n")?;

        let config = repo.gitdir.join("config");
        let config_content = toml::to_string(&repo.settings)?;
        fs::write(config, config_content)?;

        Ok(repo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_repository_new_no_gitdir() {
        let tempdir = TempDir::new().unwrap();
        let repo = Repository::new(tempdir.path(), false);
        assert_eq!(repo.unwrap_err().to_string(), "Not a git repository");
    }

    #[test]
    fn test_repository_new_no_gitdir_force() {
        let tempdir = TempDir::new().unwrap();
        let repo = Repository::new(tempdir.path(), true);
        assert!(repo.is_ok());
    }

    #[test]
    fn test_repository_new_is_gitdir() {
        let tempdir = TempDir::new().unwrap();
        let gitdir = tempdir.path().join(".git");
        let config = gitdir.join("config.ini");
        fs::create_dir(gitdir).unwrap();
        fs::File::create(config).unwrap();
        let repo = Repository::new(tempdir.path(), false);
        assert!(repo.is_ok());
    }

    #[test]
    fn test_create() {
        let tempdir = TempDir::new().unwrap();
        let repo = Repository::create(tempdir.path());
        assert!(repo.is_ok());

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
}
