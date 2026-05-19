use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct SkillSummary {
    pub id: String,
    pub namespace: String,
    pub name: String,
    pub path: String,
    pub description: Option<String>,
    pub installed: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct SkillList {
    pub source: Option<String>,
    pub resolved_ref: Option<String>,
    pub skills: Vec<SkillSummary>,
}

#[derive(Debug, Serialize)]
pub struct SkillInstall {
    pub id: String,
    pub source: String,
    pub resolved_ref: Option<String>,
    pub installed_path: String,
}

pub fn list(source: Option<&str>, dest: Option<&Path>) -> Result<SkillList> {
    let Some(source) = source else {
        return Ok(SkillList {
            source: None,
            resolved_ref: None,
            skills: Vec::new(),
        });
    };
    reject_remote_source(source)?;
    let source_path = PathBuf::from(source);
    let root = skills_root(&source_path);
    let mut skills = Vec::new();

    if root.exists() {
        for namespace in read_dirs(&root)? {
            for skill in read_dirs(&namespace)? {
                let skill_file = skill.join("SKILL.md");
                if !skill_file.exists() {
                    continue;
                }
                let namespace_name = file_name(&namespace)?;
                let skill_name = file_name(&skill)?;
                let id = format!("{namespace_name}/{skill_name}");
                skills.push(SkillSummary {
                    id: id.clone(),
                    namespace: namespace_name,
                    name: skill_name,
                    path: skill.display().to_string(),
                    description: read_description(&skill_file)?,
                    installed: dest.map(|dest| dest.join(&id).join("SKILL.md").exists()),
                });
            }
        }
    }

    skills.sort_by(|left, right| left.id.cmp(&right.id));

    Ok(SkillList {
        source: Some(source_path.display().to_string()),
        resolved_ref: git_head(&source_path),
        skills,
    })
}

pub fn install(source: &str, dest: &Path, id: &str, force: bool) -> Result<SkillInstall> {
    reject_remote_source(source)?;
    let source_path = PathBuf::from(source);
    let root = skills_root(&source_path);
    let source_skill = root.join(id);
    let skill_file = source_skill.join("SKILL.md");
    if !skill_file.exists() {
        anyhow::bail!("Skill not found in source: {id}");
    }

    let target = dest.join(id);
    if target.exists() {
        if force {
            fs::remove_dir_all(&target)?;
        } else {
            anyhow::bail!("Skill already exists at {}", target.display());
        }
    }

    copy_dir(&source_skill, &target)?;

    Ok(SkillInstall {
        id: id.to_string(),
        source: source_path.display().to_string(),
        resolved_ref: git_head(&source_path),
        installed_path: target.display().to_string(),
    })
}

fn skills_root(source: &Path) -> PathBuf {
    let nested = source.join("skills");
    if nested.exists() {
        nested
    } else {
        source.to_path_buf()
    }
}

fn read_dirs(path: &Path) -> Result<Vec<PathBuf>> {
    let mut dirs = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            dirs.push(entry.path());
        }
    }
    dirs.sort();
    Ok(dirs)
}

fn file_name(path: &Path) -> Result<String> {
    Ok(path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid skill path: {}", path.display()))?
        .to_string())
}

fn read_description(path: &Path) -> Result<Option<String>> {
    let body = fs::read_to_string(path)?;
    let mut in_frontmatter = false;
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed == "---" {
            if in_frontmatter {
                break;
            }
            in_frontmatter = true;
            continue;
        }
        if in_frontmatter {
            if let Some(description) = trimmed.strip_prefix("description:") {
                return Ok(Some(description.trim().trim_matches('"').to_string()));
            }
        }
    }
    Ok(None)
}

fn copy_dir(source: &Path, target: &Path) -> Result<()> {
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&source_path, &target_path)?;
        } else {
            fs::copy(&source_path, &target_path)
                .with_context(|| format!("failed to copy {}", source_path.display()))?;
        }
    }
    Ok(())
}

fn reject_remote_source(source: &str) -> Result<()> {
    if source.starts_with("https://") || source.starts_with("http://") || source.starts_with("git@")
    {
        anyhow::bail!(
            "Remote skill sources are not fetched by this command. Clone and pin the source locally, then pass the local path."
        );
    }
    Ok(())
}

fn git_head(path: &Path) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}
