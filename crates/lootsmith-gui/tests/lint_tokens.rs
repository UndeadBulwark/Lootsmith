//! Lint: no legacy `Tokens::` colour references may remain in the GUI crate
//! once the shell migration (step 6) is done. Ignored until then.

use std::path::Path;

/// Fails while any source file under `src/` still mentions `Tokens::`.
#[test]
#[ignore = "step 6 (shell migration) has not landed yet; enable in the last shell commit"]
fn no_token_references_remain() {
    if std::env::var_os("LOOTSMITH_NO_GPU").is_some() {
        eprintln!("skipped: no GPU adapter in this environment");
        return;
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut hits = Vec::new();
    for entry in walk(&root) {
        let text = std::fs::read_to_string(&entry).expect("read source");
        for (line_no, line) in text.lines().enumerate() {
            if line.contains("Tokens::") {
                hits.push(format!("{}:{}", entry.display(), line_no + 1));
            }
        }
    }
    assert!(
        hits.is_empty(),
        "legacy Tokens:: references remain ({}):\n{}",
        hits.len(),
        hits.join("\n")
    );
}

fn walk(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                out.extend(walk(&path));
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }
    out
}
