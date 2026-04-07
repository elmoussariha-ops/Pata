use crate::io_fast;
use crate::types::{ProjectIndex, RetrievalHit};
use std::path::Path;

pub fn top_n(index: &ProjectIndex, query: &str, n: usize, root: &Path) -> Vec<RetrievalHit> {
    let terms = query
        .split_whitespace()
        .map(|s| s.to_lowercase())
        .collect::<Vec<_>>();
    let mut hits = Vec::new();

    for f in &index.file_summaries {
        let mut score = 0usize;
        let file_name = f.path.to_string_lossy().to_lowercase();
        for t in &terms {
            if file_name.contains(t) {
                score += 5;
            }
            if f.module_summary.to_lowercase().contains(t) {
                score += 2;
            }
            if f.symbols.iter().any(|x| x.to_lowercase().contains(t)) {
                score += 3;
            }
        }
        if score > 0 {
            let text = io_fast::read_text(&root.join(&f.path));
            let excerpt = text.lines().take(10).collect::<Vec<_>>().join("\n");
            hits.push(RetrievalHit {
                path: f.path.clone(),
                score,
                excerpt,
            });
        }
    }
    hits.sort_by(|a, b| b.score.cmp(&a.score));
    hits.truncate(n);
    hits
}
