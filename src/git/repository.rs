use anyhow::{Context, Result};
use git2::Repository;
use std::path::{Path, PathBuf};

pub struct RepoCache {
    repo: Repository,
    workdir: PathBuf,
}

impl RepoCache {
    pub fn open(path: &Path) -> Result<Self> {
        let repo =
            Repository::discover(path).context("Not a git repository (or any parent directory)")?;
        let workdir = repo
            .workdir()
            .context("Bare repositories are not supported")?
            .to_path_buf();
        Ok(Self { repo, workdir })
    }

    pub fn repo(&self) -> &Repository {
        &self.repo
    }

    pub fn workdir(&self) -> &Path {
        &self.workdir
    }
}
