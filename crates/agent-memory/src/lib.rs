//! Intelligent memory layer for specialized agents.
//!
//! This crate provides a modular 4-layer memory architecture:
//! 1) short-term conversation context
//! 2) interaction summary cards (condensed session snapshots)
//! 3) learning memory (errors/corrections patterns)
//! 4) permanent memory (long-term source of truth)

use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Role of a message in an interaction.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

/// Atomic message item.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
}

/// Rich interaction envelope used by short-term and summary layers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InteractionRecord {
    pub interaction_id: Uuid,
    pub objective: String,
    pub messages: Vec<Message>,
    pub final_answer: String,
    pub tags: Vec<String>,
}

impl InteractionRecord {
    pub fn new(objective: impl Into<String>, final_answer: impl Into<String>) -> Self {
        Self {
            interaction_id: Uuid::new_v4(),
            objective: objective.into(),
            messages: Vec::new(),
            final_answer: final_answer.into(),
            tags: Vec::new(),
        }
    }
}

/// Recent context memory with bounded size.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortTermMemory {
    capacity: usize,
    recent_interactions: VecDeque<InteractionRecord>,
}

impl ShortTermMemory {
    pub fn with_capacity(capacity: usize) -> Result<Self, MemoryError> {
        if capacity == 0 {
            return Err(MemoryError::InvalidCapacity);
        }

        Ok(Self {
            capacity,
            recent_interactions: VecDeque::with_capacity(capacity),
        })
    }

    pub fn push(&mut self, interaction: InteractionRecord) {
        if self.recent_interactions.len() == self.capacity {
            self.recent_interactions.pop_front();
        }
        self.recent_interactions.push_back(interaction);
    }

    pub fn recent(&self) -> impl Iterator<Item = &InteractionRecord> {
        self.recent_interactions.iter()
    }
}

/// Condensed card optimized for retrieval and later reuse.
///
/// Unlike permanent memory, this is a lossy summary of one interaction.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InteractionSummaryCard {
    pub interaction_id: Uuid,
    pub objective: String,
    pub user_intent: String,
    pub answer_summary: String,
    pub key_points: Vec<String>,
    pub follow_up_actions: Vec<String>,
    pub tags: Vec<String>,
}

/// Strategy contract to summarize an interaction.
pub trait InteractionSummarizer: Send + Sync {
    fn summarize(&self, interaction: &InteractionRecord) -> InteractionSummaryCard;
}

/// Baseline summarizer with deterministic heuristics.
#[derive(Debug, Default, Clone)]
pub struct HeuristicSummarizer;

impl InteractionSummarizer for HeuristicSummarizer {
    fn summarize(&self, interaction: &InteractionRecord) -> InteractionSummaryCard {
        let user_intent = interaction
            .messages
            .iter()
            .find(|m| m.role == MessageRole::User)
            .map(|m| truncate(&m.content, 200))
            .unwrap_or_else(|| truncate(&interaction.objective, 200));

        let answer_summary = truncate(&interaction.final_answer, 240);

        let key_points = if interaction.final_answer.is_empty() {
            vec!["No answer generated".to_string()]
        } else {
            interaction
                .final_answer
                .split('.')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .take(3)
                .map(ToString::to_string)
                .collect()
        };

        InteractionSummaryCard {
            interaction_id: interaction.interaction_id,
            objective: interaction.objective.clone(),
            user_intent,
            answer_summary,
            key_points,
            follow_up_actions: Vec::new(),
            tags: interaction.tags.clone(),
        }
    }
}

/// Error and correction pair used to improve future runs.
///
/// Unlike permanent memory, this stores operational lessons and patterns.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ErrorCorrection {
    pub error_signature: String,
    pub root_cause: String,
    pub correction: String,
    pub prevention_rule: String,
}

/// Learning entry persisted in learning memory.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LearningEntry {
    pub id: Uuid,
    pub interaction_id: Option<Uuid>,
    pub correction: ErrorCorrection,
    pub confidence_gain: f32,
}

/// Learning memory indexed by error signature.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LearningMemory {
    entries: Vec<LearningEntry>,
    by_signature: HashMap<String, Vec<usize>>,
}

impl LearningMemory {
    pub fn add(&mut self, entry: LearningEntry) {
        let idx = self.entries.len();
        self.by_signature
            .entry(entry.correction.error_signature.clone())
            .or_default()
            .push(idx);
        self.entries.push(entry);
    }

    pub fn by_error_signature(&self, signature: &str) -> Vec<&LearningEntry> {
        self.by_signature
            .get(signature)
            .into_iter()
            .flat_map(|indexes| indexes.iter().filter_map(|i| self.entries.get(*i)))
            .collect()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn entries(&self) -> &[LearningEntry] {
        &self.entries
    }
}

/// Importance level for permanent memory records.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum MemoryPriority {
    Normal,
    High,
    Critical,
}

/// Semantic category for durable knowledge.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PermanentMemoryKind {
    UserPreference,
    LongTermObjective,
    ArchitectureDecision,
    SystemRule,
    BusinessConstraint,
    DevelopmentConvention,
    Other,
}

/// Durable source-of-truth entry.
///
/// Unlike summary cards and learning entries, this record is designed for durable
/// facts/rules and supports explicit, revisioned updates.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PermanentMemoryEntry {
    pub key: String,
    pub kind: PermanentMemoryKind,
    pub value: String,
    pub priority: MemoryPriority,
    pub source: Option<String>,
    pub revision: u64,
    pub updated_at_unix: u64,
}

impl PermanentMemoryEntry {
    pub fn new(
        key: impl Into<String>,
        kind: PermanentMemoryKind,
        value: impl Into<String>,
        priority: MemoryPriority,
    ) -> Self {
        Self {
            key: key.into(),
            kind,
            value: value.into(),
            priority,
            source: None,
            revision: 1,
            updated_at_unix: unix_now(),
        }
    }
}

/// Explicit safe update command for permanent entries.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PermanentMemoryUpdate {
    pub key: String,
    pub new_value: String,
    pub new_priority: Option<MemoryPriority>,
    pub reason: String,
    pub expected_revision: Option<u64>,
}

/// Mutation policy to protect critical permanent records.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverwritePolicy {
    Safe,
    Force,
}

/// Backend abstraction for permanent memory.
///
/// This trait allows future adapters for file, SQLite, PostgreSQL, or KV stores,
/// while keeping `MemoryEngine` decoupled from storage implementations.
pub trait PermanentMemoryBackend: Send + Sync {
    fn insert(&mut self, entry: PermanentMemoryEntry) -> Result<(), MemoryError>;
    fn update(
        &mut self,
        update: PermanentMemoryUpdate,
        policy: OverwritePolicy,
    ) -> Result<(), MemoryError>;
    fn get(&self, key: &str) -> Option<&PermanentMemoryEntry>;
    fn list(&self) -> Vec<&PermanentMemoryEntry>;
}

/// In-memory backend used as default and for tests.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InMemoryPermanentBackend {
    records: BTreeMap<String, PermanentMemoryEntry>,
}

impl PermanentMemoryBackend for InMemoryPermanentBackend {
    fn insert(&mut self, mut entry: PermanentMemoryEntry) -> Result<(), MemoryError> {
        if self.records.contains_key(&entry.key) {
            return Err(MemoryError::EntryAlreadyExists {
                key: entry.key.clone(),
            });
        }

        entry.updated_at_unix = unix_now();
        self.records.insert(entry.key.clone(), entry);
        Ok(())
    }

    fn update(
        &mut self,
        update: PermanentMemoryUpdate,
        policy: OverwritePolicy,
    ) -> Result<(), MemoryError> {
        let existing =
            self.records
                .get_mut(&update.key)
                .ok_or_else(|| MemoryError::EntryNotFound {
                    key: update.key.clone(),
                })?;

        if existing.priority == MemoryPriority::Critical && policy != OverwritePolicy::Force {
            return Err(MemoryError::CriticalOverwriteBlocked {
                key: update.key.clone(),
            });
        }

        if let Some(expected) = update.expected_revision {
            if existing.revision != expected {
                return Err(MemoryError::RevisionConflict {
                    key: update.key,
                    expected,
                    actual: existing.revision,
                });
            }
        }

        existing.value = update.new_value;
        if let Some(priority) = update.new_priority {
            existing.priority = priority;
        }
        existing.revision += 1;
        existing.updated_at_unix = unix_now();
        Ok(())
    }

    fn get(&self, key: &str) -> Option<&PermanentMemoryEntry> {
        self.records.get(key)
    }

    fn list(&self) -> Vec<&PermanentMemoryEntry> {
        self.records.values().collect()
    }
}

/// Fourth memory layer: source of truth for critical durable knowledge.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PermanentMemory<B = InMemoryPermanentBackend>
where
    B: PermanentMemoryBackend,
{
    backend: B,
}

impl<B> PermanentMemory<B>
where
    B: PermanentMemoryBackend,
{
    pub fn new(backend: B) -> Self {
        Self { backend }
    }

    pub fn insert(&mut self, entry: PermanentMemoryEntry) -> Result<(), MemoryError> {
        self.backend.insert(entry)
    }

    pub fn update_safe(&mut self, update: PermanentMemoryUpdate) -> Result<(), MemoryError> {
        self.backend.update(update, OverwritePolicy::Safe)
    }

    pub fn update_with_policy(
        &mut self,
        update: PermanentMemoryUpdate,
        policy: OverwritePolicy,
    ) -> Result<(), MemoryError> {
        self.backend.update(update, policy)
    }

    pub fn get(&self, key: &str) -> Option<&PermanentMemoryEntry> {
        self.backend.get(key)
    }

    pub fn all(&self) -> Vec<&PermanentMemoryEntry> {
        self.backend.list()
    }
}

/// Unified 4-layer memory service.
#[derive(Debug, Clone)]
pub struct MemoryEngine<S = HeuristicSummarizer>
where
    S: InteractionSummarizer,
{
    /// Ephemeral conversation context.
    pub short_term: ShortTermMemory,
    /// Condensed interaction cards.
    pub summaries: Vec<InteractionSummaryCard>,
    /// Error/correction learnings.
    pub learning: LearningMemory,
    /// Durable source-of-truth memory.
    pub permanent: PermanentMemory,
    /// Buffered errors pending deterministic consolidation into learning memory.
    pub pending_errors: Vec<ErrorCorrection>,
    summarizer: S,
}

impl<S> MemoryEngine<S>
where
    S: InteractionSummarizer,
{
    pub fn new(short_term_capacity: usize, summarizer: S) -> Result<Self, MemoryError> {
        Ok(Self {
            short_term: ShortTermMemory::with_capacity(short_term_capacity)?,
            summaries: Vec::new(),
            learning: LearningMemory::default(),
            permanent: PermanentMemory::default(),
            pending_errors: Vec::new(),
            summarizer,
        })
    }

    /// Ingest a full interaction in short-term and summary layers.
    pub fn ingest_interaction(&mut self, interaction: InteractionRecord) {
        let card = self.summarizer.summarize(&interaction);
        self.short_term.push(interaction);
        self.summaries.push(card);
    }

    /// Record short-term context only (without immediate summarization).
    ///
    /// Useful when consolidation is deferred by policy.
    pub fn record_short_term_only(&mut self, interaction: InteractionRecord) {
        self.short_term.push(interaction);
    }

    /// Enrich learning memory with an error/correction pattern.
    pub fn ingest_learning(
        &mut self,
        interaction_id: Option<Uuid>,
        correction: ErrorCorrection,
        confidence_gain: f32,
    ) {
        self.learning.add(LearningEntry {
            id: Uuid::new_v4(),
            interaction_id,
            correction,
            confidence_gain,
        });
    }

    /// Buffer an error/correction for later deterministic consolidation.
    pub fn buffer_error_for_learning(&mut self, correction: ErrorCorrection) {
        self.pending_errors.push(correction);
    }
}

/// Retrieval intent guides which memory layers are queried first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetrievalIntent {
    ImmediateContext,
    SessionRecall,
    ErrorAvoidance,
    DurableRules,
    Balanced,
}

/// Memory retrieval request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetrievalQuery {
    pub intent: RetrievalIntent,
    pub text: String,
    pub max_results: usize,
}

impl RetrievalQuery {
    pub fn new(intent: RetrievalIntent, text: impl Into<String>, max_results: usize) -> Self {
        Self {
            intent,
            text: text.into(),
            max_results: max_results.max(1),
        }
    }
}

/// Layer that produced a retrieval hit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetrievalLayer {
    ShortTerm,
    Summary,
    Learning,
    Permanent,
}

/// Retrieval result with deterministic score and source layer.
#[derive(Debug, Clone, PartialEq)]
pub struct RetrievedMemory {
    pub layer: RetrievalLayer,
    pub score: f32,
    pub value: String,
}

/// Explicit retrieval interface for integration with external orchestrators.
pub trait MemoryRetriever {
    fn retrieve(&self, query: &RetrievalQuery) -> Vec<RetrievedMemory>;
}

impl<S> MemoryRetriever for MemoryEngine<S>
where
    S: InteractionSummarizer,
{
    fn retrieve(&self, query: &RetrievalQuery) -> Vec<RetrievedMemory> {
        let tokens = tokenize(&query.text);
        let mut hits = Vec::new();

        let include_short = matches!(
            query.intent,
            RetrievalIntent::ImmediateContext
                | RetrievalIntent::SessionRecall
                | RetrievalIntent::Balanced
        );
        let include_summary = matches!(
            query.intent,
            RetrievalIntent::SessionRecall
                | RetrievalIntent::DurableRules
                | RetrievalIntent::Balanced
        );
        let include_learning = matches!(
            query.intent,
            RetrievalIntent::ErrorAvoidance | RetrievalIntent::Balanced
        );
        let include_permanent = matches!(
            query.intent,
            RetrievalIntent::ImmediateContext
                | RetrievalIntent::DurableRules
                | RetrievalIntent::ErrorAvoidance
                | RetrievalIntent::Balanced
        );

        if include_short {
            for interaction in self.short_term.recent() {
                let haystack = format!("{} {}", interaction.objective, interaction.final_answer);
                let score = overlap_score(&tokens, &haystack) + 0.15;
                if score > 0.0 {
                    hits.push(RetrievedMemory {
                        layer: RetrievalLayer::ShortTerm,
                        score,
                        value: haystack,
                    });
                }
            }
        }

        if include_summary {
            for card in &self.summaries {
                let haystack = format!(
                    "{} {} {}",
                    card.objective, card.user_intent, card.answer_summary
                );
                let score = overlap_score(&tokens, &haystack) + 0.10;
                if score > 0.0 {
                    hits.push(RetrievedMemory {
                        layer: RetrievalLayer::Summary,
                        score,
                        value: haystack,
                    });
                }
            }
        }

        if include_learning {
            for learned in self.learning.entries() {
                let haystack = format!(
                    "{} {} {}",
                    learned.correction.error_signature,
                    learned.correction.root_cause,
                    learned.correction.correction
                );
                let score = overlap_score(&tokens, &haystack) + 0.20;
                if score > 0.0 {
                    hits.push(RetrievedMemory {
                        layer: RetrievalLayer::Learning,
                        score,
                        value: haystack,
                    });
                }
            }
        }

        if include_permanent {
            for entry in self.permanent.all() {
                let haystack = format!("{} {}", entry.key, entry.value);
                let priority_boost = match entry.priority {
                    MemoryPriority::Critical => 0.35,
                    MemoryPriority::High => 0.20,
                    MemoryPriority::Normal => 0.05,
                };
                let score = overlap_score(&tokens, &haystack) + priority_boost;
                if score > 0.0 {
                    hits.push(RetrievedMemory {
                        layer: RetrievalLayer::Permanent,
                        score,
                        value: haystack,
                    });
                }
            }
        }

        hits.sort_by(|a, b| b.score.total_cmp(&a.score));
        hits.truncate(query.max_results);
        hits
    }
}

/// Deterministic consolidation actions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsolidationAction {
    ShortTermToSummary {
        interaction_id: Uuid,
    },
    SummaryToPermanent {
        key: String,
        kind: PermanentMemoryKind,
        priority: MemoryPriority,
        value: String,
    },
    PendingErrorToLearning {
        error_signature: String,
    },
}

/// Full consolidation plan with explainable actions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsolidationPlan {
    pub actions: Vec<ConsolidationAction>,
}

/// Explicit consolidation interface for future scheduler integration.
pub trait MemoryConsolidator {
    fn plan_consolidation(&self) -> ConsolidationPlan;
    fn apply_consolidation(&mut self) -> Result<ConsolidationPlan, MemoryError>;
}

impl<S> MemoryConsolidator for MemoryEngine<S>
where
    S: InteractionSummarizer,
{
    fn plan_consolidation(&self) -> ConsolidationPlan {
        let summarized_ids: HashSet<Uuid> =
            self.summaries.iter().map(|s| s.interaction_id).collect();
        let mut actions = Vec::new();

        for interaction in self.short_term.recent() {
            if !summarized_ids.contains(&interaction.interaction_id) {
                actions.push(ConsolidationAction::ShortTermToSummary {
                    interaction_id: interaction.interaction_id,
                });
            }
        }

        for card in &self.summaries {
            if let Some((key, kind, priority)) = summary_to_permanent_hint(card) {
                let already_exists = self.permanent.get(&key).is_some();
                if !already_exists {
                    actions.push(ConsolidationAction::SummaryToPermanent {
                        key,
                        kind,
                        priority,
                        value: card.answer_summary.clone(),
                    });
                }
            }
        }

        for correction in &self.pending_errors {
            actions.push(ConsolidationAction::PendingErrorToLearning {
                error_signature: correction.error_signature.clone(),
            });
        }

        ConsolidationPlan { actions }
    }

    fn apply_consolidation(&mut self) -> Result<ConsolidationPlan, MemoryError> {
        let plan = self.plan_consolidation();

        for action in &plan.actions {
            match action {
                ConsolidationAction::ShortTermToSummary { interaction_id } => {
                    if let Some(interaction) = self
                        .short_term
                        .recent()
                        .find(|x| &x.interaction_id == interaction_id)
                        .cloned()
                    {
                        let card = self.summarizer.summarize(&interaction);
                        self.summaries.push(card);
                    }
                }
                ConsolidationAction::SummaryToPermanent {
                    key,
                    kind,
                    priority,
                    value,
                } => {
                    let entry =
                        PermanentMemoryEntry::new(key.clone(), *kind, value.clone(), *priority);
                    if self.permanent.get(key).is_none() {
                        self.permanent.insert(entry)?;
                    }
                }
                ConsolidationAction::PendingErrorToLearning { error_signature } => {
                    if let Some(pos) = self
                        .pending_errors
                        .iter()
                        .position(|err| &err.error_signature == error_signature)
                    {
                        let correction = self.pending_errors.remove(pos);
                        self.ingest_learning(None, correction, 0.1);
                    }
                }
            }
        }

        Ok(plan)
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum MemoryError {
    #[error("short term memory capacity must be greater than zero")]
    InvalidCapacity,
    #[error("permanent memory entry already exists: {key}")]
    EntryAlreadyExists { key: String },
    #[error("permanent memory entry not found: {key}")]
    EntryNotFound { key: String },
    #[error("critical permanent entry cannot be overwritten in safe mode: {key}")]
    CriticalOverwriteBlocked { key: String },
    #[error("revision conflict on key '{key}', expected {expected} but found {actual}")]
    RevisionConflict {
        key: String,
        expected: u64,
        actual: u64,
    },
}

fn tokenize(input: &str) -> Vec<String> {
    input
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|t| !t.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn overlap_score(query_tokens: &[String], haystack: &str) -> f32 {
    if query_tokens.is_empty() {
        return 0.0;
    }

    let hay_tokens: HashSet<String> = tokenize(haystack).into_iter().collect();
    let overlap = query_tokens
        .iter()
        .filter(|token| hay_tokens.contains((*token).as_str()))
        .count();

    overlap as f32 / query_tokens.len() as f32
}

fn summary_to_permanent_hint(
    card: &InteractionSummaryCard,
) -> Option<(String, PermanentMemoryKind, MemoryPriority)> {
    let tags: HashSet<String> = card.tags.iter().map(|t| t.to_lowercase()).collect();

    let mapping = [
        (
            "architecture",
            PermanentMemoryKind::ArchitectureDecision,
            MemoryPriority::Critical,
        ),
        (
            "rule",
            PermanentMemoryKind::SystemRule,
            MemoryPriority::Critical,
        ),
        (
            "constraint",
            PermanentMemoryKind::BusinessConstraint,
            MemoryPriority::High,
        ),
        (
            "convention",
            PermanentMemoryKind::DevelopmentConvention,
            MemoryPriority::High,
        ),
        (
            "objective",
            PermanentMemoryKind::LongTermObjective,
            MemoryPriority::High,
        ),
        (
            "preference",
            PermanentMemoryKind::UserPreference,
            MemoryPriority::Normal,
        ),
    ];

    for (needle, kind, priority) in mapping {
        if tags.contains(needle) {
            let key = format!("summary.{}.{}", needle, card.interaction_id);
            return Some((key, kind, priority));
        }
    }

    None
}

fn truncate(input: &str, limit: usize) -> String {
    if input.chars().count() <= limit {
        return input.to_string();
    }

    let truncated: String = input.chars().take(limit).collect();
    format!("{truncated}...")
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_term_memory_evicts_oldest_interaction() {
        let mut stm = ShortTermMemory::with_capacity(2).expect("valid capacity");
        let first = InteractionRecord::new("a", "answer-a");
        let second = InteractionRecord::new("b", "answer-b");
        let third = InteractionRecord::new("c", "answer-c");

        stm.push(first);
        stm.push(second.clone());
        stm.push(third.clone());

        let recent: Vec<_> = stm.recent().collect();
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].objective, second.objective);
        assert_eq!(recent[1].objective, third.objective);
    }

    #[test]
    fn engine_auto_generates_summary_card() {
        let mut engine =
            MemoryEngine::new(4, HeuristicSummarizer).expect("memory engine should initialize");

        let mut interaction = InteractionRecord::new(
            "Explain Rust ownership",
            "Ownership ensures memory safety. Borrowing avoids copies.",
        );
        interaction.messages.push(Message {
            role: MessageRole::User,
            content: "Can you explain ownership quickly?".to_string(),
        });
        interaction.tags = vec!["rust".to_string(), "teaching".to_string()];

        engine.ingest_interaction(interaction.clone());

        assert_eq!(engine.short_term.recent().count(), 1);
        assert_eq!(engine.summaries.len(), 1);
        assert_eq!(
            engine.summaries[0].interaction_id,
            interaction.interaction_id
        );
        assert!(engine.summaries[0]
            .answer_summary
            .contains("Ownership ensures"));
        assert_eq!(engine.summaries[0].tags, vec!["rust", "teaching"]);
    }

    #[test]
    fn learning_memory_indexes_entries_by_error_signature() {
        let mut learning = LearningMemory::default();

        learning.add(LearningEntry {
            id: Uuid::new_v4(),
            interaction_id: None,
            correction: ErrorCorrection {
                error_signature: "E0502".to_string(),
                root_cause: "mutable and immutable borrow overlap".to_string(),
                correction: "split immutable borrow scope".to_string(),
                prevention_rule: "limit lifetimes and use scoped blocks".to_string(),
            },
            confidence_gain: 0.2,
        });

        let found = learning.by_error_signature("E0502");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].correction.error_signature, "E0502");
    }

    #[test]
    fn permanent_memory_insertion_and_reading_work() {
        let mut permanent: PermanentMemory<InMemoryPermanentBackend> = PermanentMemory::default();

        permanent
            .insert(PermanentMemoryEntry::new(
                "user.pref.style",
                PermanentMemoryKind::UserPreference,
                "concise",
                MemoryPriority::Normal,
            ))
            .expect("insert should work");

        let entry = permanent
            .get("user.pref.style")
            .expect("entry must be retrievable");
        assert_eq!(entry.value, "concise");
        assert_eq!(entry.revision, 1);
    }

    #[test]
    fn permanent_memory_update_is_explicit_and_safe() {
        let mut permanent: PermanentMemory<InMemoryPermanentBackend> = PermanentMemory::default();

        permanent
            .insert(PermanentMemoryEntry::new(
                "objective.platform",
                PermanentMemoryKind::LongTermObjective,
                "Build modular OSS reference",
                MemoryPriority::High,
            ))
            .expect("insert should work");

        permanent
            .update_safe(PermanentMemoryUpdate {
                key: "objective.platform".to_string(),
                new_value: "Build modular OSS Rust reference".to_string(),
                new_priority: Some(MemoryPriority::Critical),
                reason: "objective refined".to_string(),
                expected_revision: Some(1),
            })
            .expect("safe update should work for non-critical existing entry");

        let entry = permanent
            .get("objective.platform")
            .expect("entry must exist");
        assert_eq!(entry.value, "Build modular OSS Rust reference");
        assert_eq!(entry.priority, MemoryPriority::Critical);
        assert_eq!(entry.revision, 2);
    }

    #[test]
    fn permanent_memory_protects_critical_entries_from_safe_overwrite() {
        let mut permanent: PermanentMemory<InMemoryPermanentBackend> = PermanentMemory::default();
        let key = "architecture.rule.decoupling";

        permanent
            .insert(PermanentMemoryEntry::new(
                key,
                PermanentMemoryKind::ArchitectureDecision,
                "agent-core must not depend on persona crates",
                MemoryPriority::Critical,
            ))
            .expect("insert should work");

        let err = permanent
            .update_safe(PermanentMemoryUpdate {
                key: key.to_string(),
                new_value: "overwrite attempt".to_string(),
                new_priority: None,
                reason: "accidental".to_string(),
                expected_revision: Some(1),
            })
            .expect_err("safe update must be blocked for critical entry");

        assert_eq!(
            err,
            MemoryError::CriticalOverwriteBlocked {
                key: key.to_string(),
            }
        );
    }

    #[test]
    fn permanent_memory_force_update_allows_controlled_override() {
        let mut permanent: PermanentMemory<InMemoryPermanentBackend> = PermanentMemory::default();
        let key = "system.rule.audit";

        permanent
            .insert(PermanentMemoryEntry::new(
                key,
                PermanentMemoryKind::SystemRule,
                "Every tool call must be logged",
                MemoryPriority::Critical,
            ))
            .expect("insert should work");

        permanent
            .update_with_policy(
                PermanentMemoryUpdate {
                    key: key.to_string(),
                    new_value: "Every tool call and result must be logged".to_string(),
                    new_priority: None,
                    reason: "policy strengthening".to_string(),
                    expected_revision: Some(1),
                },
                OverwritePolicy::Force,
            )
            .expect("force update should work");

        let entry = permanent.get(key).expect("entry should exist");
        assert_eq!(entry.value, "Every tool call and result must be logged");
        assert_eq!(entry.revision, 2);
    }

    #[test]
    fn memory_engine_keeps_layers_logically_separated() {
        let mut engine =
            MemoryEngine::new(3, HeuristicSummarizer).expect("memory engine should initialize");

        engine.ingest_interaction(InteractionRecord::new("Draft README", "README drafted"));

        assert_eq!(engine.short_term.recent().count(), 1);
        assert_eq!(engine.summaries.len(), 1);
        assert!(engine.learning.is_empty());
        assert_eq!(engine.permanent.all().len(), 0);
    }

    #[test]
    fn retrieval_targets_expected_layers_by_intent() {
        let mut engine =
            MemoryEngine::new(3, HeuristicSummarizer).expect("memory engine should initialize");

        let mut interaction =
            InteractionRecord::new("Set architecture decision", "Use strict crate boundaries");
        interaction.tags = vec!["architecture".to_string()];
        engine.ingest_interaction(interaction);

        engine
            .permanent
            .insert(PermanentMemoryEntry::new(
                "system.rule.audit",
                PermanentMemoryKind::SystemRule,
                "All tool calls must be logged",
                MemoryPriority::Critical,
            ))
            .expect("insert should work");

        let short_hits = engine.retrieve(&RetrievalQuery::new(
            RetrievalIntent::SessionRecall,
            "architecture decision",
            5,
        ));
        assert!(short_hits
            .iter()
            .any(|hit| hit.layer == RetrievalLayer::ShortTerm));
        assert!(short_hits
            .iter()
            .any(|hit| hit.layer == RetrievalLayer::Summary));
        assert!(!short_hits
            .iter()
            .any(|hit| hit.layer == RetrievalLayer::Learning));

        let durable_hits = engine.retrieve(&RetrievalQuery::new(
            RetrievalIntent::DurableRules,
            "audit rule",
            5,
        ));
        assert!(durable_hits
            .iter()
            .any(|hit| hit.layer == RetrievalLayer::Permanent));
    }

    #[test]
    fn retrieval_prioritizes_critical_permanent_entries() {
        let mut engine =
            MemoryEngine::new(3, HeuristicSummarizer).expect("memory engine should initialize");

        engine
            .permanent
            .insert(PermanentMemoryEntry::new(
                "rule.normal",
                PermanentMemoryKind::SystemRule,
                "Audit logs retained",
                MemoryPriority::Normal,
            ))
            .expect("insert should work");
        engine
            .permanent
            .insert(PermanentMemoryEntry::new(
                "rule.critical",
                PermanentMemoryKind::SystemRule,
                "Audit logs retained",
                MemoryPriority::Critical,
            ))
            .expect("insert should work");

        let hits = engine.retrieve(&RetrievalQuery::new(
            RetrievalIntent::DurableRules,
            "audit logs retained",
            2,
        ));
        assert_eq!(hits.len(), 2);
        assert!(hits[0].value.contains("rule.critical"));
    }

    #[test]
    fn consolidation_moves_data_across_layers_deterministically() {
        let mut engine =
            MemoryEngine::new(3, HeuristicSummarizer).expect("memory engine should initialize");

        let mut interaction = InteractionRecord::new(
            "Define architecture constraints",
            "Core must remain decoupled from persona crates",
        );
        interaction.tags = vec!["architecture".to_string()];
        engine.record_short_term_only(interaction.clone());

        engine.buffer_error_for_learning(ErrorCorrection {
            error_signature: "E0502".to_string(),
            root_cause: "borrow overlap".to_string(),
            correction: "shorten borrow scope".to_string(),
            prevention_rule: "keep mutable scopes tight".to_string(),
        });

        let plan = engine
            .apply_consolidation()
            .expect("consolidation should work");
        assert!(!plan.actions.is_empty());
        assert!(engine
            .summaries
            .iter()
            .any(|s| s.interaction_id == interaction.interaction_id));
        assert_eq!(engine.learning.len(), 1);
    }

    #[test]
    fn consolidation_preserves_layer_responsibilities() {
        let mut engine =
            MemoryEngine::new(3, HeuristicSummarizer).expect("memory engine should initialize");

        engine.record_short_term_only(InteractionRecord::new(
            "Keep a temporary note",
            "Temporary answer",
        ));

        let _ = engine
            .apply_consolidation()
            .expect("consolidation should work");

        // Consolidation can create summaries, but should not invent learning entries
        // or permanent records without explicit triggers (tags/errors).
        assert_eq!(engine.learning.len(), 0);
        assert_eq!(engine.permanent.all().len(), 0);
    }
}
