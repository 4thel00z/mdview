use std::path::{Path, PathBuf};

use crate::protocol::is_markdown;

const MAX_DEPTH: usize = 4;
const MAX_FILES: usize = 400;
const SKIPPED: [&str; 6] = ["node_modules", "target", "dist", "build", "vendor", "Pods"];

pub fn search_root(document: &Path) -> Option<PathBuf> {
    let folder = document.parent()?;
    let repository = folder.ancestors().find(|candidate| candidate.join(".git").exists());
    Some(repository.unwrap_or(folder).to_path_buf())
}

pub fn markdown_files(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    collect(root, 0, &mut found);
    found.sort_by_key(|path| (path.components().count(), path.to_string_lossy().to_lowercase()));
    found
}

fn collect(directory: &Path, depth: usize, found: &mut Vec<PathBuf>) {
    if depth > MAX_DEPTH || found.len() >= MAX_FILES {
        return;
    }
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    let mut entries: Vec<_> = entries.filter_map(Result::ok).collect();
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        if found.len() >= MAX_FILES {
            return;
        }
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with('.') || SKIPPED.contains(&name.as_ref()) {
            continue;
        }
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        let path = entry.path();
        if kind.is_dir() {
            collect(&path, depth + 1, found);
            continue;
        }
        if kind.is_file() && is_markdown(&path) {
            found.push(path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_root_prefers_the_enclosing_repository() {
        let root = std::env::temp_dir().join(format!("mdview-root-{}", std::process::id()));
        std::fs::create_dir_all(root.join(".git")).ok();
        std::fs::create_dir_all(root.join("docs/deep")).ok();
        let inside = search_root(&root.join("docs/deep/a.md"));
        std::fs::remove_dir_all(root.join(".git")).ok();
        let outside = search_root(&root.join("docs/deep/a.md"));
        std::fs::remove_dir_all(&root).ok();
        assert_eq!(inside, Some(root.clone()));
        assert_eq!(outside, Some(root.join("docs/deep")));
    }

    #[test]
    fn finds_markdown_skipping_hidden_and_build_directories() {
        let root = std::env::temp_dir().join(format!("mdview-files-{}", std::process::id()));
        let make = |relative: &str| {
            let path = root.join(relative);
            std::fs::create_dir_all(path.parent().unwrap_or(&root)).ok();
            std::fs::write(&path, "# x").ok();
        };
        ["README.md", "docs/guide.markdown", "docs/deep/a.md", "notes.txt", ".git/x.md", "node_modules/pkg/README.md"]
            .into_iter()
            .for_each(make);
        let names: Vec<String> = markdown_files(&root)
            .iter()
            .filter_map(|path| path.strip_prefix(&root).ok())
            .map(|path| path.to_string_lossy().into_owned())
            .collect();
        std::fs::remove_dir_all(&root).ok();
        assert_eq!(names, ["README.md", "docs/guide.markdown", "docs/deep/a.md"]);
    }
}
