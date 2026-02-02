//! Priority queue for network events.
//!
//! This module provides priority-based event scheduling to ensure
//! critical events (combat, turn end) are processed before cosmetic updates.

use nostr_nations_core::events::{GameAction, GameEvent};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, VecDeque};

/// Priority levels for events.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventPriority {
    /// Highest priority - system critical (game end, disconnect).
    Critical = 0,
    /// High priority - gameplay affecting (combat, turn end).
    High = 1,
    /// Normal priority - standard game actions.
    #[default]
    Normal = 2,
    /// Low priority - non-critical updates.
    Low = 3,
    /// Lowest priority - cosmetic/visual updates.
    Cosmetic = 4,
}

impl EventPriority {
    /// Get the numeric value (lower = higher priority).
    pub fn value(&self) -> u8 {
        *self as u8
    }

    /// Check if this priority is higher than another.
    pub fn is_higher_than(&self, other: &EventPriority) -> bool {
        self.value() < other.value()
    }
}

impl Ord for EventPriority {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering so lower value = higher priority
        other.value().cmp(&self.value())
    }
}

impl PartialOrd for EventPriority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Determine the priority of a game event based on its action type.
pub fn event_priority(event: &GameEvent) -> EventPriority {
    match &event.action {
        // High priority - game state changes and combat
        GameAction::EndTurn => EventPriority::High,
        GameAction::EndGame { .. } => EventPriority::Critical,
        GameAction::StartGame => EventPriority::High,
        GameAction::AttackUnit { .. } => EventPriority::High,
        GameAction::AttackCity { .. } => EventPriority::High,

        // Normal priority - standard game actions
        GameAction::CreateGame { .. } => EventPriority::Normal,
        GameAction::JoinGame { .. } => EventPriority::Normal,
        GameAction::MoveUnit { .. } => EventPriority::Normal,
        GameAction::FoundCity { .. } => EventPriority::Normal,
        GameAction::SetProduction { .. } => EventPriority::Normal,
        GameAction::BuyItem { .. } => EventPriority::Normal,
        GameAction::SetResearch { .. } => EventPriority::Normal,

        // Low priority - unit state changes
        GameAction::FortifyUnit { .. } => EventPriority::Low,
        GameAction::SleepUnit { .. } => EventPriority::Low,
        GameAction::WakeUnit { .. } => EventPriority::Low,
        GameAction::DeleteUnit { .. } => EventPriority::Normal,
        GameAction::UpgradeUnit { .. } => EventPriority::Normal,

        // Worker actions
        GameAction::BuildImprovement { .. } => EventPriority::Normal,
        GameAction::BuildRoad { .. } => EventPriority::Normal,
        GameAction::RemoveFeature { .. } => EventPriority::Normal,

        // City management
        GameAction::AssignCitizen { .. } => EventPriority::Low,
        GameAction::UnassignCitizen { .. } => EventPriority::Low,
        GameAction::SellBuilding { .. } => EventPriority::Normal,

        // Diplomacy
        GameAction::DeclareWar { .. } => EventPriority::High,
        GameAction::ProposePeace { .. } => EventPriority::Normal,
        GameAction::AcceptPeace { .. } => EventPriority::Normal,
        GameAction::RejectPeace { .. } => EventPriority::Normal,

        // Randomness
        GameAction::RequestRandom { .. } => EventPriority::Normal,
        GameAction::ProvideRandom { .. } => EventPriority::Normal,
    }
}

/// A queued event with priority and metadata.
#[derive(Clone, Debug)]
pub struct PrioritizedEvent {
    /// The event itself.
    pub event: GameEvent,
    /// Priority level.
    pub priority: EventPriority,
    /// Sequence number for FIFO ordering within same priority.
    pub sequence: u64,
    /// Time the event was enqueued.
    pub enqueued_at: u64,
}

impl PrioritizedEvent {
    /// Create a new prioritized event.
    pub fn new(event: GameEvent, priority: EventPriority, sequence: u64) -> Self {
        Self {
            event,
            priority,
            sequence,
            enqueued_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
        }
    }

    /// Create with automatic priority detection.
    pub fn auto(event: GameEvent, sequence: u64) -> Self {
        let priority = event_priority(&event);
        Self::new(event, priority, sequence)
    }
}

impl Eq for PrioritizedEvent {}

impl PartialEq for PrioritizedEvent {
    fn eq(&self, other: &Self) -> bool {
        self.event.id == other.event.id
    }
}

impl Ord for PrioritizedEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        // First compare by priority (higher priority first)
        match self.priority.cmp(&other.priority) {
            Ordering::Equal => {
                // Same priority: FIFO by sequence (lower sequence first)
                other.sequence.cmp(&self.sequence)
            }
            other_ordering => other_ordering,
        }
    }
}

impl PartialOrd for PrioritizedEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Configuration for the priority queue.
#[derive(Clone, Debug)]
pub struct PriorityQueueConfig {
    /// Maximum events in the queue.
    pub max_size: usize,
    /// Enable fair scheduling (prevents starvation).
    pub fair_scheduling: bool,
    /// Maximum consecutive events from same priority before yielding.
    pub max_consecutive: usize,
    /// Age-based promotion threshold (ms).
    pub age_promotion_threshold: u64,
}

impl Default for PriorityQueueConfig {
    fn default() -> Self {
        Self {
            max_size: 10000,
            fair_scheduling: true,
            max_consecutive: 10,
            age_promotion_threshold: 5000, // 5 seconds
        }
    }
}

/// Priority queue for network events with fair scheduling.
pub struct EventPriorityQueue {
    /// Configuration.
    config: PriorityQueueConfig,
    /// Priority heap for efficient retrieval.
    heap: BinaryHeap<PrioritizedEvent>,
    /// Per-priority queues for fair scheduling.
    priority_queues: HashMap<EventPriority, VecDeque<PrioritizedEvent>>,
    /// Next sequence number.
    next_sequence: u64,
    /// Consecutive events from current priority.
    consecutive_count: usize,
    /// Last served priority.
    last_priority: Option<EventPriority>,
    /// Statistics.
    stats: PriorityQueueStats,
}

/// Statistics for the priority queue.
#[derive(Clone, Debug, Default)]
pub struct PriorityQueueStats {
    /// Total events enqueued.
    pub events_enqueued: u64,
    /// Total events dequeued.
    pub events_dequeued: u64,
    /// Events dropped due to capacity.
    pub events_dropped: u64,
    /// Events by priority level.
    pub by_priority: HashMap<EventPriority, u64>,
    /// Priority promotions (due to aging).
    pub promotions: u64,
}

impl EventPriorityQueue {
    /// Create a new priority queue.
    pub fn new(config: PriorityQueueConfig) -> Self {
        let mut priority_queues = HashMap::new();
        priority_queues.insert(EventPriority::Critical, VecDeque::new());
        priority_queues.insert(EventPriority::High, VecDeque::new());
        priority_queues.insert(EventPriority::Normal, VecDeque::new());
        priority_queues.insert(EventPriority::Low, VecDeque::new());
        priority_queues.insert(EventPriority::Cosmetic, VecDeque::new());

        Self {
            config,
            heap: BinaryHeap::new(),
            priority_queues,
            next_sequence: 0,
            consecutive_count: 0,
            last_priority: None,
            stats: PriorityQueueStats::default(),
        }
    }

    /// Create with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(PriorityQueueConfig::default())
    }

    /// Enqueue an event with automatic priority detection.
    pub fn enqueue(&mut self, event: GameEvent) -> Result<(), QueueError> {
        let priority = event_priority(&event);
        self.enqueue_with_priority(event, priority)
    }

    /// Enqueue an event with explicit priority.
    pub fn enqueue_with_priority(
        &mut self,
        event: GameEvent,
        priority: EventPriority,
    ) -> Result<(), QueueError> {
        // Check capacity
        if self.len() >= self.config.max_size {
            self.stats.events_dropped += 1;
            return Err(QueueError::QueueFull);
        }

        let sequence = self.next_sequence;
        self.next_sequence += 1;

        let prioritized = PrioritizedEvent::new(event, priority, sequence);

        // Add to appropriate structures
        if self.config.fair_scheduling {
            if let Some(queue) = self.priority_queues.get_mut(&priority) {
                queue.push_back(prioritized);
            }
        } else {
            self.heap.push(prioritized);
        }

        // Update stats
        self.stats.events_enqueued += 1;
        *self.stats.by_priority.entry(priority).or_insert(0) += 1;

        Ok(())
    }

    /// Dequeue the next event based on priority and fair scheduling.
    pub fn dequeue(&mut self) -> Option<GameEvent> {
        if self.config.fair_scheduling {
            self.dequeue_fair()
        } else {
            self.dequeue_strict()
        }
    }

    /// Strict priority dequeue (always highest priority first).
    fn dequeue_strict(&mut self) -> Option<GameEvent> {
        let prioritized = self.heap.pop()?;
        self.stats.events_dequeued += 1;
        Some(prioritized.event)
    }

    /// Fair scheduling dequeue (prevents starvation).
    fn dequeue_fair(&mut self) -> Option<GameEvent> {
        // Check if we should yield to lower priorities
        let should_yield = self.consecutive_count >= self.config.max_consecutive;

        // Try priorities in order
        let priorities = [
            EventPriority::Critical,
            EventPriority::High,
            EventPriority::Normal,
            EventPriority::Low,
            EventPriority::Cosmetic,
        ];

        for priority in priorities {
            // Skip if we should yield and this was the last priority
            if should_yield
                && Some(priority) == self.last_priority
                && priority != EventPriority::Critical
            {
                continue;
            }

            if let Some(queue) = self.priority_queues.get_mut(&priority) {
                if let Some(prioritized) = queue.pop_front() {
                    // Update consecutive tracking
                    if Some(priority) == self.last_priority {
                        self.consecutive_count += 1;
                    } else {
                        self.consecutive_count = 1;
                        self.last_priority = Some(priority);
                    }

                    self.stats.events_dequeued += 1;
                    return Some(prioritized.event);
                }
            }
        }

        // If we yielded but found nothing, try again without yielding
        if should_yield {
            self.consecutive_count = 0;
            return self.dequeue_fair();
        }

        None
    }

    /// Peek at the next event without removing it.
    pub fn peek(&self) -> Option<&GameEvent> {
        if self.config.fair_scheduling {
            // Check priorities in order
            for priority in [
                EventPriority::Critical,
                EventPriority::High,
                EventPriority::Normal,
                EventPriority::Low,
                EventPriority::Cosmetic,
            ] {
                if let Some(queue) = self.priority_queues.get(&priority) {
                    if let Some(prioritized) = queue.front() {
                        return Some(&prioritized.event);
                    }
                }
            }
            None
        } else {
            self.heap.peek().map(|p| &p.event)
        }
    }

    /// Get the number of events in the queue.
    pub fn len(&self) -> usize {
        if self.config.fair_scheduling {
            self.priority_queues.values().map(|q| q.len()).sum()
        } else {
            self.heap.len()
        }
    }

    /// Check if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the count for a specific priority level.
    pub fn count_by_priority(&self, priority: EventPriority) -> usize {
        if self.config.fair_scheduling {
            self.priority_queues
                .get(&priority)
                .map(|q| q.len())
                .unwrap_or(0)
        } else {
            self.heap.iter().filter(|p| p.priority == priority).count()
        }
    }

    /// Clear all events from the queue.
    pub fn clear(&mut self) {
        self.heap.clear();
        for queue in self.priority_queues.values_mut() {
            queue.clear();
        }
        self.consecutive_count = 0;
        self.last_priority = None;
    }

    /// Get queue statistics.
    pub fn stats(&self) -> &PriorityQueueStats {
        &self.stats
    }

    /// Get the configuration.
    pub fn config(&self) -> &PriorityQueueConfig {
        &self.config
    }

    /// Drain all events of a specific priority.
    pub fn drain_priority(&mut self, priority: EventPriority) -> Vec<GameEvent> {
        if self.config.fair_scheduling {
            self.priority_queues
                .get_mut(&priority)
                .map(|q| q.drain(..).map(|p| p.event).collect())
                .unwrap_or_default()
        } else {
            let (matching, remaining): (Vec<_>, Vec<_>) =
                self.heap.drain().partition(|p| p.priority == priority);
            self.heap = remaining.into_iter().collect();
            matching.into_iter().map(|p| p.event).collect()
        }
    }
}

/// Queue errors.
#[derive(Clone, Debug)]
pub enum QueueError {
    /// Queue is at capacity.
    QueueFull,
    /// Invalid priority.
    InvalidPriority,
}

impl std::fmt::Display for QueueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueueError::QueueFull => write!(f, "Priority queue is full"),
            QueueError::InvalidPriority => write!(f, "Invalid priority level"),
        }
    }
}

impl std::error::Error for QueueError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_event(id: &str, action: GameAction) -> GameEvent {
        let mut event = GameEvent::new("test_game".to_string(), 0, None, 1, 1, action);
        event.id = id.to_string();
        event
    }

    // ==================== EventPriority Tests ====================

    #[test]
    fn test_event_priority_ordering() {
        assert!(EventPriority::Critical > EventPriority::High);
        assert!(EventPriority::High > EventPriority::Normal);
        assert!(EventPriority::Normal > EventPriority::Low);
        assert!(EventPriority::Low > EventPriority::Cosmetic);
    }

    #[test]
    fn test_event_priority_is_higher_than() {
        assert!(EventPriority::Critical.is_higher_than(&EventPriority::High));
        assert!(!EventPriority::Low.is_higher_than(&EventPriority::Normal));
    }

    #[test]
    fn test_event_priority_default() {
        assert_eq!(EventPriority::default(), EventPriority::Normal);
    }

    // ==================== event_priority Tests ====================

    #[test]
    fn test_event_priority_detection() {
        let attack = create_event(
            "e1",
            GameAction::AttackUnit {
                attacker_id: 1,
                defender_id: 2,
                random: 0.5,
            },
        );
        assert_eq!(event_priority(&attack), EventPriority::High);

        let end_turn = create_event("e2", GameAction::EndTurn);
        assert_eq!(event_priority(&end_turn), EventPriority::High);

        let move_unit = create_event(
            "e3",
            GameAction::MoveUnit {
                unit_id: 1,
                path: vec![],
            },
        );
        assert_eq!(event_priority(&move_unit), EventPriority::Normal);
    }

    // ==================== PrioritizedEvent Tests ====================

    #[test]
    fn test_prioritized_event_ordering() {
        let high = PrioritizedEvent::new(
            create_event("e1", GameAction::EndTurn),
            EventPriority::High,
            1,
        );
        let normal = PrioritizedEvent::new(
            create_event("e2", GameAction::EndTurn),
            EventPriority::Normal,
            2,
        );

        assert!(high > normal);
    }

    #[test]
    fn test_prioritized_event_fifo_within_priority() {
        let first = PrioritizedEvent::new(
            create_event("e1", GameAction::EndTurn),
            EventPriority::Normal,
            1,
        );
        let second = PrioritizedEvent::new(
            create_event("e2", GameAction::EndTurn),
            EventPriority::Normal,
            2,
        );

        // Lower sequence should come first (be "greater" in heap terms)
        assert!(first > second);
    }

    #[test]
    fn test_prioritized_event_auto() {
        let attack = create_event(
            "e1",
            GameAction::AttackUnit {
                attacker_id: 1,
                defender_id: 2,
                random: 0.5,
            },
        );
        let prioritized = PrioritizedEvent::auto(attack, 1);

        assert_eq!(prioritized.priority, EventPriority::High);
    }

    // ==================== EventPriorityQueue Tests ====================

    #[test]
    fn test_priority_queue_new() {
        let queue = EventPriorityQueue::with_defaults();
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_priority_queue_enqueue() {
        let mut queue = EventPriorityQueue::with_defaults();

        queue
            .enqueue(create_event("e1", GameAction::EndTurn))
            .unwrap();
        assert_eq!(queue.len(), 1);
    }

    #[test]
    fn test_priority_queue_dequeue_strict() {
        let config = PriorityQueueConfig {
            fair_scheduling: false,
            ..Default::default()
        };
        let mut queue = EventPriorityQueue::new(config);

        // Enqueue in reverse priority order
        queue
            .enqueue_with_priority(create_event("low", GameAction::EndTurn), EventPriority::Low)
            .unwrap();
        queue
            .enqueue_with_priority(
                create_event("high", GameAction::EndTurn),
                EventPriority::High,
            )
            .unwrap();
        queue
            .enqueue_with_priority(
                create_event("normal", GameAction::EndTurn),
                EventPriority::Normal,
            )
            .unwrap();

        // Should dequeue in priority order
        assert_eq!(queue.dequeue().unwrap().id, "high");
        assert_eq!(queue.dequeue().unwrap().id, "normal");
        assert_eq!(queue.dequeue().unwrap().id, "low");
    }

    #[test]
    fn test_priority_queue_dequeue_fair() {
        let config = PriorityQueueConfig {
            fair_scheduling: true,
            max_consecutive: 2,
            ..Default::default()
        };
        let mut queue = EventPriorityQueue::new(config);

        // Enqueue several high priority
        for i in 0..5 {
            queue
                .enqueue_with_priority(
                    create_event(&format!("high{}", i), GameAction::EndTurn),
                    EventPriority::High,
                )
                .unwrap();
        }

        // Enqueue one normal priority
        queue
            .enqueue_with_priority(
                create_event("normal", GameAction::EndTurn),
                EventPriority::Normal,
            )
            .unwrap();

        // First two should be high
        let e1 = queue.dequeue().unwrap();
        let e2 = queue.dequeue().unwrap();
        assert!(e1.id.starts_with("high"));
        assert!(e2.id.starts_with("high"));

        // Third should be normal (fair scheduling kicks in)
        let e3 = queue.dequeue().unwrap();
        assert_eq!(e3.id, "normal");
    }

    #[test]
    fn test_priority_queue_peek() {
        let mut queue = EventPriorityQueue::with_defaults();

        queue
            .enqueue(create_event("e1", GameAction::EndTurn))
            .unwrap();

        let peeked = queue.peek().unwrap();
        assert_eq!(peeked.id, "e1");

        // Still in queue
        assert_eq!(queue.len(), 1);
    }

    #[test]
    fn test_priority_queue_count_by_priority() {
        let mut queue = EventPriorityQueue::with_defaults();

        queue
            .enqueue_with_priority(create_event("h1", GameAction::EndTurn), EventPriority::High)
            .unwrap();
        queue
            .enqueue_with_priority(create_event("h2", GameAction::EndTurn), EventPriority::High)
            .unwrap();
        queue
            .enqueue_with_priority(
                create_event("n1", GameAction::EndTurn),
                EventPriority::Normal,
            )
            .unwrap();

        assert_eq!(queue.count_by_priority(EventPriority::High), 2);
        assert_eq!(queue.count_by_priority(EventPriority::Normal), 1);
        assert_eq!(queue.count_by_priority(EventPriority::Low), 0);
    }

    #[test]
    fn test_priority_queue_clear() {
        let mut queue = EventPriorityQueue::with_defaults();

        queue
            .enqueue(create_event("e1", GameAction::EndTurn))
            .unwrap();
        queue
            .enqueue(create_event("e2", GameAction::EndTurn))
            .unwrap();

        queue.clear();
        assert!(queue.is_empty());
    }

    #[test]
    fn test_priority_queue_full() {
        let config = PriorityQueueConfig {
            max_size: 2,
            ..Default::default()
        };
        let mut queue = EventPriorityQueue::new(config);

        queue
            .enqueue(create_event("e1", GameAction::EndTurn))
            .unwrap();
        queue
            .enqueue(create_event("e2", GameAction::EndTurn))
            .unwrap();

        let result = queue.enqueue(create_event("e3", GameAction::EndTurn));
        assert!(matches!(result, Err(QueueError::QueueFull)));
        assert_eq!(queue.stats().events_dropped, 1);
    }

    #[test]
    fn test_priority_queue_drain_priority() {
        let mut queue = EventPriorityQueue::with_defaults();

        queue
            .enqueue_with_priority(create_event("h1", GameAction::EndTurn), EventPriority::High)
            .unwrap();
        queue
            .enqueue_with_priority(
                create_event("n1", GameAction::EndTurn),
                EventPriority::Normal,
            )
            .unwrap();
        queue
            .enqueue_with_priority(create_event("h2", GameAction::EndTurn), EventPriority::High)
            .unwrap();

        let high_events = queue.drain_priority(EventPriority::High);
        assert_eq!(high_events.len(), 2);
        assert_eq!(queue.count_by_priority(EventPriority::High), 0);
        assert_eq!(queue.count_by_priority(EventPriority::Normal), 1);
    }

    #[test]
    fn test_priority_queue_stats() {
        let mut queue = EventPriorityQueue::with_defaults();

        queue
            .enqueue_with_priority(create_event("h1", GameAction::EndTurn), EventPriority::High)
            .unwrap();
        queue.dequeue();

        let stats = queue.stats();
        assert_eq!(stats.events_enqueued, 1);
        assert_eq!(stats.events_dequeued, 1);
        assert_eq!(stats.by_priority.get(&EventPriority::High), Some(&1));
    }

    // ==================== QueueError Tests ====================

    #[test]
    fn test_queue_error_display() {
        assert!(format!("{}", QueueError::QueueFull).contains("full"));
        assert!(format!("{}", QueueError::InvalidPriority).contains("Invalid"));
    }
}
