use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

pub struct GitCli {
    workdir: std::path::PathBuf,
}

impl GitCli {
    pub fn new(workdir: &Path) -> Self {
        Self {
            workdir: workdir.to_path_buf(),
        }
    }

    pub fn stage_file(&self, path: &Path) -> Result<()> {
        let output = Command::new("git")
            .args(["add", "--"])
            .arg(path)
            .current_dir(&self.workdir)
            .output()
            .context("Failed to run git add")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("git add failed: {stderr}");
        }
        Ok(())
    }

    pub fn unstage_file(&self, path: &Path) -> Result<()> {
        let output = Command::new("git")
            .args(["reset", "HEAD", "--"])
            .arg(path)
            .current_dir(&self.workdir)
            .output()
            .context("Failed to run git reset")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("git reset failed: {stderr}");
        }
        Ok(())
    }

    pub fn restore_file(&self, path: &Path) -> Result<()> {
        let output = Command::new("git")
            .args(["checkout", "--"])
            .arg(path)
            .current_dir(&self.workdir)
            .output()
            .context("Failed to run git checkout")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("git checkout failed: {stderr}");
        }
        Ok(())
    }

    pub fn commit(&self, message: &str) -> Result<()> {
        let output = Command::new("git")
            .args(["commit", "-m", message])
            .current_dir(&self.workdir)
            .output()
            .context("Failed to run git commit")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("git commit failed: {stderr}");
        }
        Ok(())
    }

    pub fn stage_all(&self) -> Result<()> {
        let output = Command::new("git")
            .args(["add", "-A"])
            .current_dir(&self.workdir)
            .output()
            .context("Failed to run git add -A")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("git add -A failed: {stderr}");
        }
        Ok(())
    }
}
