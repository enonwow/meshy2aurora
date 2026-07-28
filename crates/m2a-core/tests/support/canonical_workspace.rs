use std::path::PathBuf;

/// Resolves read-only local test data from the canonical repository when the
/// test binary is compiled from an approved Git worktree nested below
/// `.worktrees`.
///
/// This never creates or copies an asset root. Ordinary clones keep using
/// their own repository root; only the canonical nested-worktree layout
/// projects back to its owning repository for Git-ignored fixtures.
pub fn canonical_repository_root() -> PathBuf {
    let crate_directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repository_root = crate_directory
        .parent()
        .and_then(std::path::Path::parent)
        .expect("m2a-core must be nested at crates/m2a-core");
    let worktrees_directory = repository_root.parent();
    if worktrees_directory
        .and_then(std::path::Path::file_name)
        .is_some_and(|name| name.eq_ignore_ascii_case(".worktrees"))
    {
        return worktrees_directory
            .and_then(std::path::Path::parent)
            .expect(".worktrees must have the canonical repository parent")
            .to_path_buf();
    }
    repository_root.to_path_buf()
}

#[test]
fn nested_worktree_projection_never_creates_a_second_asset_root() {
    let root = canonical_repository_root();
    assert!(root.join("Cargo.toml").is_file());
    assert!(root.join("documentation/CANONICAL_WORKSPACE.md").is_file());
}
