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
    let root = skills_root(&source_path)?;
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
    let id_path = skill_id_path(id)?;
    let source_path = PathBuf::from(source);
    let root = skills_root(&source_path)?;
    let source_skill = root.join(&id_path);
    let skill_file = source_skill.join("SKILL.md");
    if !skill_file.exists() {
        anyhow::bail!("Skill not found in source: {id}");
    }
    ensure_inside_root(&root, &source_skill, id)?;

    let target = dest.join(&id_path);
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

fn skill_id_path(id: &str) -> Result<PathBuf> {
    let parts = id.split('/').collect::<Vec<_>>();
    if parts.len() != 2
        || parts
            .iter()
            .any(|part| part.is_empty() || *part == "." || *part == ".." || part.contains('\\'))
    {
        anyhow::bail!("Invalid skill id: {id}. Expected <namespace>/<skill>.");
    }

    Ok(PathBuf::from(parts[0]).join(parts[1]))
}

fn skills_root(source: &Path) -> Result<PathBuf> {
    reject_path_symlink(source)?;
    let nested = source.join("skills");

    match fs::symlink_metadata(&nested) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() {
                anyhow::bail!(
                    "Skill source contains unsupported symlink: {}",
                    nested.display()
                );
            }
            if metadata.is_dir() {
                Ok(nested)
            } else {
                Ok(source.to_path_buf())
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(source.to_path_buf()),
        Err(error) => Err(error).with_context(|| format!("failed to inspect {}", nested.display())),
    }
}

fn reject_path_symlink(path: &Path) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            anyhow::bail!(
                "Skill source contains unsupported symlink: {}",
                path.display()
            );
        }
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error).with_context(|| format!("failed to inspect {}", path.display())),
    }
}

fn ensure_inside_root(root: &Path, source_skill: &Path, id: &str) -> Result<()> {
    let root = root
        .canonicalize()
        .with_context(|| format!("failed to resolve {}", root.display()))?;
    let source_skill = source_skill
        .canonicalize()
        .with_context(|| format!("failed to resolve {}", source_skill.display()))?;

    if !source_skill.starts_with(root) {
        anyhow::bail!("Skill source escapes skills root: {id}");
    }

    Ok(())
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
    let metadata = fs::symlink_metadata(source)
        .with_context(|| format!("failed to inspect {}", source.display()))?;
    let file_type = metadata.file_type();
    if file_type.is_symlink() {
        anyhow::bail!(
            "Skill source contains unsupported symlink: {}",
            source.display()
        );
    }
    if !file_type.is_dir() {
        anyhow::bail!(
            "Skill source contains unsupported file type: {}",
            source.display()
        );
    }

    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            anyhow::bail!(
                "Skill source contains unsupported symlink: {}",
                source_path.display()
            );
        }
        if file_type.is_dir() {
            copy_dir(&source_path, &target_path)?;
        } else if file_type.is_file() {
            fs::copy(&source_path, &target_path)
                .with_context(|| format!("failed to copy {}", source_path.display()))?;
        } else {
            anyhow::bail!(
                "Skill source contains unsupported file type: {}",
                source_path.display()
            );
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
