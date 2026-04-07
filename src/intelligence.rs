#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntelligenceMode {
    Safe,
    Fast,
    Deep,
    LowPower,
}

impl IntelligenceMode {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "safe" => Some(Self::Safe),
            "fast" => Some(Self::Fast),
            "deep" => Some(Self::Deep),
            "low-power" => Some(Self::LowPower),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Safe => "safe",
            Self::Fast => "fast",
            Self::Deep => "deep",
            Self::LowPower => "low-power",
        }
    }

    pub fn retrieval_n(self, base: usize) -> usize {
        match self {
            Self::Safe => base.min(5),
            Self::Fast => base.min(4),
            Self::Deep => base.saturating_add(2),
            Self::LowPower => base.min(3),
        }
    }

    pub fn run_full_validation(self) -> bool {
        !matches!(self, Self::Fast)
    }
}

pub fn mode_from_env() -> IntelligenceMode {
    std::env::var("PATA_MODE")
        .ok()
        .and_then(|v| IntelligenceMode::from_str(&v))
        .unwrap_or(IntelligenceMode::Safe)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_parsing_works() {
        assert_eq!(IntelligenceMode::from_str("safe"), Some(IntelligenceMode::Safe));
        assert_eq!(IntelligenceMode::from_str("bad"), None);
    }
}
