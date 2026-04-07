use crate::fssec;
use crate::lock;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Mission {
    pub id: String,
    pub objective: String,
    pub mode: String,
    pub status: String,
    pub steps: Vec<String>,
    pub last_patch: String,
    pub last_validation: String,
}

fn missions_dir(root: &Path) -> PathBuf {
    root.join(".pata/missions")
}

pub fn save_active(root: &Path, mission: &Mission) -> Result<(), String> {
    let _guard = lock::acquire(root, "mission-write")?;
    let dir = missions_dir(root);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    fssec::set_secure_dir(&dir)?;
    let path = dir.join("active.txt");
    let data = format!(
        "id={}\nobjective={}\nmode={}\nstatus={}\nlast_patch={}\nlast_validation={}\nsteps={}\n",
        mission.id,
        sanitize(&mission.objective),
        mission.mode,
        mission.status,
        mission.last_patch,
        sanitize(&mission.last_validation),
        mission.steps.join(" | ")
    );
    fs::write(&path, data).map_err(|e| e.to_string())?;
    fssec::set_secure_file(&path)
}

pub fn load_active(root: &Path) -> Result<Mission, String> {
    let path = missions_dir(root).join("active.txt");
    let txt = fs::read_to_string(&path).map_err(|e| format!("cannot read mission: {e}"))?;
    let get = |k: &str| {
        txt.lines()
            .find_map(|l| l.strip_prefix(&format!("{k}=")))
            .unwrap_or("")
            .to_string()
    };
    let steps = get("steps")
        .split(" | ")
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    Ok(Mission {
        id: get("id"),
        objective: get("objective"),
        mode: get("mode"),
        status: get("status"),
        steps,
        last_patch: get("last_patch"),
        last_validation: get("last_validation"),
    })
}

fn sanitize(s: &str) -> String {
    s.replace(['\n', '\r', '\t'], " ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn mission_roundtrip() {
        let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let root = std::env::temp_dir().join(format!("pata-mission-{ts}"));
        fs::create_dir_all(&root).unwrap();
        let m = Mission {
            id: "m1".to_string(),
            objective: "Improve scanner".to_string(),
            mode: "safe".to_string(),
            status: "planning".to_string(),
            steps: vec!["a".to_string(), "b".to_string()],
            last_patch: "none".to_string(),
            last_validation: "unknown".to_string(),
        };
        save_active(&root, &m).unwrap();
        let m2 = load_active(&root).unwrap();
        assert_eq!(m2.id, "m1");
        assert_eq!(m2.steps.len(), 2);
    }
}
