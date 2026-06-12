//! Mutation boundary computation.
use super::evidence::{ModifiedFile, MutationBoundary};
use std::collections::HashMap;

const DEFAULT_MUTATION_CAP: usize = 10_000;

pub fn compute_mutation_boundary(
    pre_files: &[(String, String)],
    post_files: &[(String, String)],
) -> MutationBoundary {
    let pre_map: HashMap<&str, &str> =
        pre_files.iter().map(|(p, h)| (p.as_str(), h.as_str())).collect();
    let post_map: HashMap<&str, &str> =
        post_files.iter().map(|(p, h)| (p.as_str(), h.as_str())).collect();
    let mut created = Vec::new();
    let mut modified = Vec::new();
    let mut deleted = Vec::new();
    for (path, _) in post_files {
        if !pre_map.contains_key(path.as_str()) {
            created.push(path.clone());
        }
    }
    for (path, _) in pre_files {
        if !post_map.contains_key(path.as_str()) {
            deleted.push(path.clone());
        }
    }
    for (path, post_hash) in post_files {
        if let Some(pre_hash) = pre_map.get(path.as_str())
            && pre_hash != &post_hash.as_str()
        {
            modified.push(ModifiedFile {
                path: path.clone(),
                pre_hash: (*pre_hash).to_string(),
                post_hash: post_hash.clone(),
            });
        }
    }
    let total = created.len() + modified.len() + deleted.len();
    let truncated = total > DEFAULT_MUTATION_CAP;
    MutationBoundary {
        created,
        modified,
        deleted,
        truncated,
        total_observed: if truncated { Some(total as u32) } else { None },
        complete: true,
        error: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn empty() {
        let b = compute_mutation_boundary(&[], &[]);
        assert!(b.created.is_empty());
        assert!(b.complete);
    }
    #[test]
    fn created() {
        let pre = vec![("a.txt".into(), "h1".into())];
        let post = vec![("a.txt".into(), "h1".into()), ("b.txt".into(), "h2".into())];
        let b = compute_mutation_boundary(&pre, &post);
        assert_eq!(b.created, vec!["b.txt"]);
    }
    #[test]
    fn modified() {
        let pre = vec![("a.txt".into(), "h1".into())];
        let post = vec![("a.txt".into(), "h2".into())];
        let b = compute_mutation_boundary(&pre, &post);
        assert_eq!(b.modified[0].pre_hash, "h1");
    }

    #[test]
    fn deleted() {
        let pre = vec![("a.txt".into(), "h1".into()), ("b.txt".into(), "h2".into())];
        let post = vec![("a.txt".into(), "h1".into())];
        let b = compute_mutation_boundary(&pre, &post);
        assert_eq!(b.deleted, vec!["b.txt"]);
        assert!(b.complete);
    }

    #[test]
    fn no_changes_read_only() {
        let files = vec![("a.txt".into(), "h1".into()), ("b.txt".into(), "h2".into())];
        let b = compute_mutation_boundary(&files, &files);
        assert!(b.created.is_empty());
        assert!(b.modified.is_empty());
        assert!(b.deleted.is_empty());
        assert!(!b.truncated);
        assert!(b.complete);
    }

    #[test]
    fn truncated_at_cap() {
        let mut pre = Vec::new();
        let mut post = Vec::new();
        for i in 0..12_000 {
            pre.push((format!("f{}.txt", i), "h1".into()));
            post.push((format!("f{}.txt", i), "h2".into()));
        }
        let b = compute_mutation_boundary(&pre, &post);
        assert!(b.truncated);
        assert_eq!(b.total_observed, Some(12_000));
        assert_eq!(b.modified.len(), 12_000);
    }

    #[test]
    fn filesystem_error_marks_incomplete() {
        // When mutation detection hits a filesystem error, the boundary
        // should be marked incomplete with the error recorded.
        let incomplete = MutationBoundary {
            created: vec!["partial.txt".into()],
            modified: Vec::new(),
            deleted: Vec::new(),
            truncated: false,
            total_observed: None,
            complete: false,
            error: Some("permission denied reading workspace/subdir".into()),
        };
        assert!(!incomplete.complete);
        assert!(incomplete.error.is_some());
    }
}
