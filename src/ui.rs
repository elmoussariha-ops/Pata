use std::io::{self, Write};

pub struct UiState {
    pub active_view: &'static str,
    pub files: Vec<String>,
    pub chat: Vec<String>,
    pub terminal: Vec<String>,
    pub logs: Vec<String>,
    pub patch_review: String,
    pub memory: Vec<String>,
    pub status: Vec<String>,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            active_view: "chat",
            files: vec![],
            chat: vec!["Bienvenue dans Pata MVP".to_string()],
            terminal: vec!["Shortcuts: f c t l p h m s q".to_string()],
            logs: vec![],
            patch_review: "Aucun patch".to_string(),
            memory: vec!["No memory snapshot".to_string()],
            status: vec!["Status unavailable".to_string()],
        }
    }
}

pub fn render(state: &UiState) {
    println!("{}", render_string(state));
}

pub fn render_string(state: &UiState) -> String {
    let mut out = String::new();
    out.push_str("\n================ PATA TUI =================\n");
    out.push_str(&format!("Active view: {}\n", state.active_view));
    out.push_str("--------------------------------------------\n");

    out.push_str("Files\n");
    for v in state.files.iter().take(6) {
        out.push_str(&format!("  - {v}\n"));
    }

    out.push_str("Chat\n");
    for v in state.chat.iter().rev().take(4).rev() {
        out.push_str(&format!("  {v}\n"));
    }

    out.push_str("Terminal\n");
    for v in state.terminal.iter().rev().take(4).rev() {
        out.push_str(&format!("  {v}\n"));
    }

    out.push_str("Logs\n");
    for v in state.logs.iter().rev().take(4).rev() {
        out.push_str(&format!("  {v}\n"));
    }

    out.push_str("Patch review\n");
    out.push_str(&format!("  {}\n", state.patch_review));

    out.push_str("Memory\n");
    for v in state.memory.iter().rev().take(4).rev() {
        out.push_str(&format!("  {v}\n"));
    }

    out.push_str("Status\n");
    for v in state.status.iter().rev().take(8).rev() {
        out.push_str(&format!("  {v}\n"));
    }

    out.push_str("--------------------------------------------\n");
    out.push_str(
        "Keys: [f]iles [c]hat [t]erminal [l]ogs [p]atch [h]istory [m]emory [s]tatus [q]uit\n",
    );
    out
}

pub fn read_key() -> Option<char> {
    print!("ui> ");
    let _ = io::stdout().flush();
    let mut s = String::new();
    io::stdin().read_line(&mut s).ok()?;
    s.chars().next()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_panel_is_rendered() {
        let mut state = UiState::new();
        state.status = vec!["git=clean".to_string(), "ollama=ok".to_string()];
        let out = render_string(&state);
        assert!(out.contains("Status"));
        assert!(out.contains("ollama=ok"));
    }
}
