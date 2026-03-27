use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use crate::config::EvolutionSettings;
use crate::provider::{GenerationRequest, LlmProvider};

const RESPONSE_FORMAT_MARKER: &str = "EVOLVER_CANDIDATE_BUNDLE";
const MAX_FILE_BYTES: usize = 24 * 1024;
const ALLOWED_EXTENSIONS: &[&str] = &["md", "rs", "toml", "json", "yaml", "yml"];

#[derive(Debug, Clone)]
pub struct EvolutionEngine {
    repo_root: PathBuf,
    workspace_root: PathBuf,
}

#[derive(Debug, Clone)]
pub struct StagedCandidate {
    pub id: String,
    pub path: PathBuf,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CandidateBundle {
    summary: String,
    files: Vec<CandidateFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CandidateFile {
    path: String,
    content: String,
}

impl EvolutionEngine {
    pub fn new(repo_root: &Path, settings: EvolutionSettings) -> Self {
        Self {
            repo_root: repo_root.to_path_buf(),
            workspace_root: repo_root.join(settings.workspace_dir),
        }
    }

    pub fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }

    pub fn count_candidates(&self) -> Result<usize> {
        let candidates_dir = self.workspace_root.join("candidates");
        if !candidates_dir.exists() {
            return Ok(0);
        }

        let mut count = 0usize;
        for entry in fs::read_dir(&candidates_dir)
            .with_context(|| format!("failed to read {}", candidates_dir.display()))?
        {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                count += 1;
            }
        }

        Ok(count)
    }

    pub async fn stage(&self, provider: &dyn LlmProvider, goal: &str) -> Result<StagedCandidate> {
        fs::create_dir_all(self.workspace_root.join("candidates")).with_context(|| {
            format!(
                "failed to create workspace {}",
                self.workspace_root.join("candidates").display()
            )
        })?;

        let snapshot = self.build_snapshot()?;
        let prompt = self.render_prompt(goal, &snapshot);
        let raw_response = provider
            .generate(GenerationRequest {
                system_prompt:
                    "You revise the local Rust CLI by returning a safe staged candidate bundle."
                        .to_string(),
                user_prompt: prompt.clone(),
            })
            .await?;

        let bundle = parse_candidate_bundle(&raw_response)?;
        if bundle.files.is_empty() {
            bail!("candidate bundle did not include any files");
        }

        let candidate_id = current_timestamp()?;
        let candidate_root = self.workspace_root.join("candidates").join(&candidate_id);
        fs::create_dir_all(&candidate_root)
            .with_context(|| format!("failed to create {}", candidate_root.display()))?;

        fs::write(candidate_root.join("prompt.txt"), prompt)
            .with_context(|| format!("failed to write {}", candidate_root.display()))?;
        fs::write(candidate_root.join("snapshot.txt"), snapshot)
            .with_context(|| format!("failed to write {}", candidate_root.display()))?;
        fs::write(candidate_root.join("response.json"), &raw_response)
            .with_context(|| format!("failed to write {}", candidate_root.display()))?;
        fs::write(
            candidate_root.join("candidate.json"),
            serde_json::to_string_pretty(&bundle)?,
        )
        .with_context(|| format!("failed to write {}", candidate_root.display()))?;

        for file in &bundle.files {
            let relative = safe_relative_path(&file.path)?;
            let destination = candidate_root.join(relative);
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("failed to create {}", parent.display()))?;
            }
            fs::write(&destination, &file.content)
                .with_context(|| format!("failed to write {}", destination.display()))?;
        }

        Ok(StagedCandidate {
            id: candidate_id,
            path: candidate_root,
            summary: bundle.summary,
        })
    }

    pub fn apply(&self, candidate_id: &str) -> Result<PathBuf> {
        let candidate_root = self.workspace_root.join("candidates").join(candidate_id);
        let bundle_path = candidate_root.join("candidate.json");
        let raw = fs::read_to_string(&bundle_path)
            .with_context(|| format!("failed to read {}", bundle_path.display()))?;
        let bundle: CandidateBundle =
            serde_json::from_str(&raw).with_context(|| "candidate.json is invalid")?;

        for file in bundle.files {
            let relative = safe_relative_path(&file.path)?;
            let destination = self.repo_root.join(relative);
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("failed to create {}", parent.display()))?;
            }
            fs::write(&destination, file.content)
                .with_context(|| format!("failed to write {}", destination.display()))?;
        }

        Ok(self.repo_root.clone())
    }

    fn build_snapshot(&self) -> Result<String> {
        let mut files = Vec::new();
        self.collect_snapshot_files(&self.repo_root, &mut files)?;
        files.sort();

        if files.is_empty() {
            return Ok("Repository is empty.".to_string());
        }

        let mut out = String::new();
        for path in files {
            let absolute = self.repo_root.join(&path);
            let content = fs::read_to_string(&absolute)
                .with_context(|| format!("failed to read {}", absolute.display()))?;
            let trimmed = truncate_to_boundary(&content, MAX_FILE_BYTES);
            out.push_str(&format!("=== {} ===\n{}\n\n", path.display(), trimmed));
        }

        Ok(out)
    }

    fn collect_snapshot_files(&self, dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
        for entry in fs::read_dir(dir)
            .with_context(|| format!("failed to read directory {}", dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            let relative = match path.strip_prefix(&self.repo_root) {
                Ok(path) => path,
                Err(_) => continue,
            };

            if is_ignored(relative) {
                continue;
            }

            let file_type = entry.file_type()?;
            if file_type.is_dir() {
                self.collect_snapshot_files(&path, out)?;
                continue;
            }

            if !file_type.is_file() {
                continue;
            }

            if should_include_in_snapshot(relative) {
                out.push(relative.to_path_buf());
            }
        }

        Ok(())
    }

    fn render_prompt(&self, goal: &str, snapshot: &str) -> String {
        format!(
            "You are revising the Rust CLI project named evolver.\n\
Goal:\n{goal}\n\n\
Return only a JSON object marked as {RESPONSE_FORMAT_MARKER}.\n\
The JSON schema is:\n\
{{\"summary\":\"short summary\",\"files\":[{{\"path\":\"Cargo.toml\",\"content\":\"...\"}},{{\"path\":\"src/main.rs\",\"content\":\"...\"}}]}}\n\n\
Rules:\n\
- Return full file contents for every file you want to replace.\n\
- Keep paths relative to the repository root.\n\
- Do not write inside .git, target, or .evolver.\n\
- Preserve the staged evolution workflow rather than editing the live binary at runtime.\n\
- Keep the program as a Rust CLI.\n\n\
Current repository snapshot:\n{snapshot}"
        )
    }
}

fn parse_candidate_bundle(raw: &str) -> Result<CandidateBundle> {
    let trimmed = raw.trim();
    let attempt = strip_code_fence(trimmed).unwrap_or(trimmed);

    match serde_json::from_str::<CandidateBundle>(attempt) {
        Ok(bundle) => Ok(bundle),
        Err(_) => {
            let start = attempt
                .find('{')
                .context("provider response did not contain JSON")?;
            let end = attempt
                .rfind('}')
                .context("provider response did not contain JSON")?;
            serde_json::from_str(&attempt[start..=end])
                .context("provider response was not valid candidate JSON")
        }
    }
}

fn strip_code_fence(raw: &str) -> Option<&str> {
    if !raw.starts_with("```") {
        return None;
    }

    let without_prefix = raw
        .strip_prefix("```json\n")
        .or_else(|| raw.strip_prefix("```\n"))?;
    without_prefix.strip_suffix("\n```")
}

fn safe_relative_path(path: &str) -> Result<PathBuf> {
    let candidate = PathBuf::from(path);
    if candidate.is_absolute() {
        bail!("absolute paths are not allowed in candidate bundles");
    }

    let mut normalized = PathBuf::new();
    for component in candidate.components() {
        match component {
            Component::Normal(part) => normalized.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                bail!("unsafe relative path in candidate bundle: {path}")
            }
        }
    }

    if normalized.as_os_str().is_empty() {
        bail!("candidate bundle contained an empty path");
    }

    let first = normalized
        .components()
        .next()
        .context("candidate path had no first component")?;
    if let Component::Normal(part) = first {
        let forbidden = [".git", ".evolver", "target"];
        let part = part.to_string_lossy();
        if forbidden.iter().any(|prefix| *prefix == part) {
            bail!("candidate path targets a forbidden location: {path}");
        }
    }

    Ok(normalized)
}

fn should_include_in_snapshot(path: &Path) -> bool {
    match path.extension().and_then(|value| value.to_str()) {
        Some(extension) => ALLOWED_EXTENSIONS.contains(&extension),
        None => path == Path::new("Cargo.toml"),
    }
}

fn is_ignored(path: &Path) -> bool {
    let Some(first) = path.components().next() else {
        return false;
    };

    matches!(
        first,
        Component::Normal(part)
            if part == ".git" || part == ".evolver" || part == "target"
    )
}

fn truncate_to_boundary(input: &str, limit: usize) -> &str {
    if input.len() <= limit {
        return input;
    }

    let mut boundary = 0usize;
    for (index, _) in input.char_indices() {
        if index > limit {
            break;
        }
        boundary = index;
    }

    &input[..boundary]
}

fn current_timestamp() -> Result<String> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock moved backwards")?;
    Ok(duration.as_secs().to_string())
}

#[cfg(test)]
mod tests {
    use super::{parse_candidate_bundle, safe_relative_path};

    #[test]
    fn parses_fenced_candidate_bundle() {
        let raw = "```json\n{\"summary\":\"ok\",\"files\":[{\"path\":\"src/main.rs\",\"content\":\"fn main() {}\"}]}\n```";
        let bundle = parse_candidate_bundle(raw).expect("bundle should parse");
        assert_eq!(bundle.summary, "ok");
        assert_eq!(bundle.files.len(), 1);
    }

    #[test]
    fn rejects_parent_segments() {
        assert!(safe_relative_path("../src/main.rs").is_err());
    }
}
