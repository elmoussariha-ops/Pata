use crate::history;
use crate::memory_engine::compress_task_memory;
use std::fs;
use std::path::Path;

pub fn optimization_tick(root: &Path) -> Result<(), String> {
    compress_task_memory(root)?;
    if matches!(
        std::env::var("PATA_LOW_POWER").ok().as_deref(),
        Some("1") | Some("true") | Some("on")
    ) {
        compress_task_memory(root)?;
    }
    fs::create_dir_all(root.join(".pata/optimizer")).map_err(|e| e.to_string())?;
    let marker = root.join(".pata/optimizer/last_tick.txt");
    fs::write(&marker, "optimizer tick complete\n").map_err(|e| e.to_string())?;
    history::log(root, "optimizer", "tick completed")?;
    Ok(())
}
