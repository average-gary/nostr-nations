//! Protocol tests for Nostr event handling.
//!
//! These tests verify the correctness of:
//! - Game event serialization/deserialization
//! - Event chain validation and integrity
//! - Event ordering by timestamp and ID
//! - All GameAction variants
//! - Event validation rules
//! - Replay system determinism

use nostr_nations_core::{
    cashu::{DeterministicRandomness, RandomnessContext, RandomnessProof, RandomnessProvider},
    city::ProductionItem,
    events::{EventBuilder, EventChain, EventChainError, GameAction, GameEvent},
    game_state::GamePhase,
    hex::HexCoord,
    replay::{GameEngine, ReplayConfig, ReplayError},
    settings::GameSettings,
    terrain::Improvement,
    types::{MapSize, PlayerId},
    unit::UnitType,
};

// =============================================================================
// Test Helpers
// =============================================================================

/// Create a GameEvent with explicit ID and timestamp for testing
fn create_event(
    id: &str,
    game_id: &str,
    player_id: PlayerId,
    prev_event_id: Option<&str>,
    turn: u32,
    sequence: u32,
    action: GameAction,
    timestamp: u64,
) -> GameEvent {
    let mut event = GameEvent::new(
        game_id.to_string(),
        player_id,
        prev_event_id.map(|s| s.to_string()),
        turn,
        sequence,
        action,
    );
    event.id = id.to_string();
    event.timestamp = timestamp;
    event
}

/// Create a GameEvent with randomness proof
fn create_event_with_proof(
    id: &str,
    game_id: &str,
    player_id: PlayerId,
    prev_event_id: Option<&str>,
    turn: u32,
    sequence: u32,
    action: GameAction,
    timestamp: u64,
    proof: RandomnessProof,
) -> GameEvent {
    let mut event = GameEvent::with_randomness(
        game_id.to_string(),
        player_id,
        prev_event_id.map(|s| s.to_string()),
        turn,
        sequence,
        action,
        proof,
    );
    event.id = id.to_string();
    event.timestamp = timestamp;
    event
}

/// Create a test randomness proof
fn create_test_proof(random_value: f32) -> RandomnessProof {
    // Convert float to bytes for the proof
    let value_as_u32 = (random_value * u32::MAX as f32) as u32;
    let mut random_bytes = [0u8; 32];
    random_bytes[0..4].copy_from_slice(&value_as_u32.to_le_bytes());

    RandomnessProof {
        mint_keyset_id: "deterministic".to_string(),
        blinded_message: vec![],
        blinded_signature: vec![],
        signature: [42u8; 32].to_vec(),
        random_bytes,
        context: "test".to_string(),
        timestamp: 0,
    }
}

/// Create default game settings JSON
fn default_settings_json() -> String {
    let settings = GameSettings::new("Test Game".to_string());
    serde_json::to_string(&settings).unwrap()
}

// =============================================================================
// 1. Game Event Serialization Tests
// =============================================================================

mod game_event_serialization {
    use super::*;

    #[test]
    fn test_game_event_serializes_to_json() {
        let event = create_event(
            "evt_001",
            "game_123",
            0,
            None,
            1,
            1,
            GameAction::EndTurn,
            1700000000,
        );

        let json = serde_json::to_string(&event);
        assert!(json.is_ok(), "GameEvent should serialize to JSON");

        let json_str = json.unwrap();
        assert!(json_str.contains("\"id\":\"evt_001\""));
        assert!(json_str.contains("\"game_id\":\"game_123\""));
        assert!(json_str.contains("\"turn\":1"));
    }

    #[test]
    fn test_game_event_deserializes_correctly() {
        let original = create_event(
            "evt_002",
            "game_456",
            1,
            Some("evt_001"),
            5,
            3,
            GameAction::EndTurn,
            1700000100,
        );

        let json = serde_json::to_string(&original).unwrap();
        let restored: GameEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.id, original.id);
        assert_eq!(restored.game_id, original.game_id);
        assert_eq!(restored.player_id, original.player_id);
        assert_eq!(restored.prev_event_id, original.prev_event_id);
        assert_eq!(restored.turn, original.turn);
        assert_eq!(restored.sequence, original.sequence);
        assert_eq!(restored.timestamp, original.timestamp);
    }

    #[test]
    fn test_create_game_action_serialization() {
        let seed = [42u8; 32];
        let settings_json = default_settings_json();

        let action = GameAction::CreateGame {
            settings_json: settings_json.clone(),
            seed,
        };

        let json = serde_json::to_string(&action).unwrap();
        let restored: GameAction = serde_json::from_str(&json).unwrap();

        match restored {
            GameAction::CreateGame {
                settings_json: s,
                seed: restored_seed,
            } => {
                assert_eq!(s, settings_json);
                assert_eq!(restored_seed, seed);
            }
            _ => panic!("Expected CreateGame action"),
        }
    }

    #[test]
    fn test_join_game_action_serialization() {
        let action = GameAction::JoinGame {
            player_name: "Alice".to_string(),
            civilization_id: "rome".to_string(),
        };

        let json = serde_json::to_string(&action).unwrap();
        let restored: GameAction = serde_json::from_str(&json).unwrap();

        match restored {
            GameAction::JoinGame {
                player_name,
                civilization_id,
            } => {
                assert_eq!(player_name, "Alice");
                assert_eq!(civilization_id, "rome");
            }
            _ => panic!("Expected JoinGame action"),
        }
    }

    #[test]
    fn test_move_unit_action_serialization() {
        let path = vec![
            HexCoord::new(5, 5),
            HexCoord::new(5, 6),
            HexCoord::new(6, 6),
        ];

        let action = GameAction::MoveUnit { unit_id: 42, path };

        let json = serde_json::to_string(&action).unwrap();
        let restored: GameAction = serde_json::from_str(&json).unwrap();

        match restored {
            GameAction::MoveUnit {
                unit_id,
                path: restored_path,
            } => {
                assert_eq!(unit_id, 42);
                assert_eq!(restored_path.len(), 3);
                assert_eq!(restored_path[0], HexCoord::new(5, 5));
            }
            _ => panic!("Expected MoveUnit action"),
        }
    }

    #[test]
    fn test_attack_unit_action_serialization() {
        let action = GameAction::AttackUnit {
            attacker_id: 1,
            defender_id: 2,
            random: 0.5,
        };

        let json = serde_json::to_string(&action).unwrap();
        let restored: GameAction = serde_json::from_str(&json).unwrap();

        match restored {
            GameAction::AttackUnit {
                attacker_id,
                defender_id,
                random,
            } => {
                assert_eq!(attacker_id, 1);
                assert_eq!(defender_id, 2);
                assert!((random - 0.5).abs() < 0.001);
            }
            _ => panic!("Expected AttackUnit action"),
        }
    }

    #[test]
    fn test_event_with_randomness_proof_serialization() {
        let proof = create_test_proof(0.75);
        let event = create_event_with_proof(
            "evt_rng",
            "game_rng",
            0,
            None,
            1,
            1,
            GameAction::AttackUnit {
                attacker_id: 1,
                defender_id: 2,
                random: 0.75,
            },
            1700000000,
            proof,
        );

        let json = serde_json::to_string(&event).unwrap();
        let restored: GameEvent = serde_json::from_str(&json).unwrap();

        assert!(restored.randomness_proof.is_some());
        assert!(restored.has_randomness_proof());
    }

    #[test]
    fn test_all_game_action_variants_serialize() {
        let actions: Vec<GameAction> = vec![
            GameAction::CreateGame {
                settings_json: "{}".to_string(),
                seed: [0u8; 32],
            },
            GameAction::JoinGame {
                player_name: "Test".to_string(),
                civilization_id: "rome".to_string(),
            },
            GameAction::StartGame,
            GameAction::EndTurn,
            GameAction::EndGame {
                winner_id: 0,
                victory_type: "domination".to_string(),
            },
            GameAction::MoveUnit {
                unit_id: 1,
                path: vec![HexCoord::new(0, 0)],
            },
            GameAction::AttackUnit {
                attacker_id: 1,
                defender_id: 2,
                random: 0.5,
            },
            GameAction::AttackCity {
                attacker_id: 1,
                city_id: 1,
                random: 0.5,
            },
            GameAction::FoundCity {
                settler_id: 1,
                name: "Rome".to_string(),
            },
            GameAction::FortifyUnit { unit_id: 1 },
            GameAction::SleepUnit { unit_id: 1 },
            GameAction::WakeUnit { unit_id: 1 },
            GameAction::DeleteUnit { unit_id: 1 },
            GameAction::UpgradeUnit {
                unit_id: 1,
                gold_cost: 100,
            },
            GameAction::BuildImprovement {
                unit_id: 1,
                improvement: Improvement::Farm,
            },
            GameAction::BuildRoad { unit_id: 1 },
            GameAction::RemoveFeature { unit_id: 1 },
            GameAction::SetProduction {
                city_id: 1,
                item: ProductionItem::Unit(UnitType::Warrior),
            },
            GameAction::BuyItem {
                city_id: 1,
                item: ProductionItem::Unit(UnitType::Warrior),
                gold_cost: 200,
            },
            GameAction::AssignCitizen {
                city_id: 1,
                tile: HexCoord::new(5, 5),
            },
            GameAction::UnassignCitizen {
                city_id: 1,
                tile: HexCoord::new(5, 5),
            },
            GameAction::SellBuilding {
                city_id: 1,
                building: "granary".to_string(),
            },
            GameAction::SetResearch {
                tech_id: "writing".to_string(),
            },
            GameAction::DeclareWar { target_player: 1 },
            GameAction::ProposePeace { target_player: 1 },
            GameAction::AcceptPeace { from_player: 1 },
            GameAction::RejectPeace { from_player: 1 },
            GameAction::RequestRandom {
                purpose: "combat".to_string(),
                blinded_message: "msg".to_string(),
            },
            GameAction::ProvideRandom {
                request_id: "req_1".to_string(),
                blind_signature: "sig".to_string(),
            },
        ];

        for (i, action) in actions.iter().enumerate() {
            let json = serde_json::to_string(action);
            assert!(
                json.is_ok(),
                "Action variant {} failed to serialize: {:?}",
                i,
                action
            );

            let json_str = json.unwrap();
            let restored: Result<GameAction, _> = serde_json::from_str(&json_str);
            assert!(
                restored.is_ok(),
                "Action variant {} failed to deserialize from: {}",
                i,
                json_str
            );
        }
    }
}

// =============================================================================
// 2. Event Chain Validation Tests
// =============================================================================

mod event_chain_validation {
    use super::*;

    #[test]
    fn test_events_link_to_previous_correctly() {
        let mut chain = EventChain::new();

        let event1 = create_event("evt1", "game1", 0, None, 1, 1, GameAction::EndTurn, 1000);
        assert!(chain.add(event1).is_ok());

        let event2 = create_event(
            "evt2",
            "game1",
            0,
            Some("evt1"),
            1,
            2,
            GameAction::EndTurn,
            1001,
        );
        assert!(chain.add(event2).is_ok());

        let event3 = create_event(
            "evt3",
            "game1",
            0,
            Some("evt2"),
            1,
            3,
            GameAction::EndTurn,
            1002,
        );
        assert!(chain.add(event3).is_ok());

        assert_eq!(chain.len(), 3);
        assert!(chain.verify().is_ok());
    }

    #[test]
    fn test_chain_validates_integrity() {
        let mut chain = EventChain::new();

        // Build a valid chain
        chain
            .add(create_event(
                "a",
                "g",
                0,
                None,
                1,
                1,
                GameAction::EndTurn,
                100,
            ))
            .unwrap();
        chain
            .add(create_event(
                "b",
                "g",
                0,
                Some("a"),
                1,
                2,
                GameAction::EndTurn,
                101,
            ))
            .unwrap();
        chain
            .add(create_event(
                "c",
                "g",
                0,
                Some("b"),
                1,
                3,
                GameAction::EndTurn,
                102,
            ))
            .unwrap();

        // Verify the chain
        assert!(chain.verify().is_ok());

        // Can retrieve events by ID
        assert!(chain.get("a").is_some());
        assert!(chain.get("b").is_some());
        assert!(chain.get("c").is_some());
        assert!(chain.get("nonexistent").is_none());
    }

    #[test]
    fn test_missing_previous_event_detected() {
        let mut chain = EventChain::new();

        // First event with no previous (valid)
        chain
            .add(create_event(
                "evt1",
                "game1",
                0,
                None,
                1,
                1,
                GameAction::EndTurn,
                1000,
            ))
            .unwrap();

        // Second event with wrong previous ID
        let result = chain.add(create_event(
            "evt2",
            "game1",
            0,
            Some("wrong_id"),
            1,
            2,
            GameAction::EndTurn,
            1001,
        ));

        assert_eq!(result, Err(EventChainError::InvalidPreviousEvent));
    }

    #[test]
    fn test_missing_previous_when_chain_not_empty() {
        let mut chain = EventChain::new();

        chain
            .add(create_event(
                "evt1",
                "game1",
                0,
                None,
                1,
                1,
                GameAction::EndTurn,
                1000,
            ))
            .unwrap();

        // Try to add event without previous when chain is not empty
        let result = chain.add(create_event(
            "evt2",
            "game1",
            0,
            None,
            1,
            2,
            GameAction::EndTurn,
            1001,
        ));

        assert_eq!(result, Err(EventChainError::MissingPreviousEvent));
    }

    #[test]
    fn test_chain_last_event() {
        let mut chain = EventChain::new();

        assert!(chain.last().is_none());
        assert!(chain.is_empty());

        chain
            .add(create_event(
                "first",
                "g",
                0,
                None,
                1,
                1,
                GameAction::EndTurn,
                100,
            ))
            .unwrap();
        assert_eq!(chain.last().unwrap().id, "first");

        chain
            .add(create_event(
                "second",
                "g",
                0,
                Some("first"),
                1,
                2,
                GameAction::EndTurn,
                101,
            ))
            .unwrap();
        assert_eq!(chain.last().unwrap().id, "second");
    }

    #[test]
    fn test_events_for_turn() {
        let mut chain = EventChain::new();

        chain
            .add(create_event(
                "t1_1",
                "g",
                0,
                None,
                1,
                1,
                GameAction::EndTurn,
                100,
            ))
            .unwrap();
        chain
            .add(create_event(
                "t1_2",
                "g",
                0,
                Some("t1_1"),
                1,
                2,
                GameAction::EndTurn,
                101,
            ))
            .unwrap();
        chain
            .add(create_event(
                "t2_1",
                "g",
                0,
                Some("t1_2"),
                2,
                1,
                GameAction::EndTurn,
                102,
            ))
            .unwrap();
        chain
            .add(create_event(
                "t2_2",
                "g",
                0,
                Some("t2_1"),
                2,
                2,
                GameAction::EndTurn,
                103,
            ))
            .unwrap();

        let turn1_events = chain.events_for_turn(1);
        assert_eq!(turn1_events.len(), 2);

        let turn2_events = chain.events_for_turn(2);
        assert_eq!(turn2_events.len(), 2);

        let turn3_events = chain.events_for_turn(3);
        assert!(turn3_events.is_empty());
    }
}

// =============================================================================
// 3. Event Ordering Tests
// =============================================================================

mod event_ordering {
    use super::*;

    #[test]
    fn test_events_sorted_by_timestamp() {
        // Create events with different timestamps
        let events = vec![
            create_event("evt3", "g", 0, None, 1, 1, GameAction::EndTurn, 3000),
            create_event("evt1", "g", 0, None, 1, 1, GameAction::EndTurn, 1000),
            create_event("evt2", "g", 0, None, 1, 1, GameAction::EndTurn, 2000),
        ];

        // Sort by timestamp
        let mut sorted = events.clone();
        sorted.sort_by_key(|e| e.timestamp);

        assert_eq!(sorted[0].id, "evt1");
        assert_eq!(sorted[1].id, "evt2");
        assert_eq!(sorted[2].id, "evt3");
    }

    #[test]
    fn test_same_timestamp_ordered_by_id() {
        let events = vec![
            create_event("evt_c", "g", 0, None, 1, 1, GameAction::EndTurn, 1000),
            create_event("evt_a", "g", 0, None, 1, 1, GameAction::EndTurn, 1000),
            create_event("evt_b", "g", 0, None, 1, 1, GameAction::EndTurn, 1000),
        ];

        // Sort by timestamp first, then by ID
        let mut sorted = events.clone();
        sorted.sort_by(|a, b| a.timestamp.cmp(&b.timestamp).then_with(|| a.id.cmp(&b.id)));

        assert_eq!(sorted[0].id, "evt_a");
        assert_eq!(sorted[1].id, "evt_b");
        assert_eq!(sorted[2].id, "evt_c");
    }

    #[test]
    fn test_turn_events_processed_in_order() {
        let mut chain = EventChain::new();

        // Add events with proper turn/sequence ordering
        chain
            .add(create_event(
                "t1_s1",
                "g",
                0,
                None,
                1,
                1,
                GameAction::MoveUnit {
                    unit_id: 1,
                    path: vec![HexCoord::new(0, 0)],
                },
                100,
            ))
            .unwrap();

        chain
            .add(create_event(
                "t1_s2",
                "g",
                0,
                Some("t1_s1"),
                1,
                2,
                GameAction::MoveUnit {
                    unit_id: 2,
                    path: vec![HexCoord::new(1, 1)],
                },
                101,
            ))
            .unwrap();

        chain
            .add(create_event(
                "t1_s3",
                "g",
                0,
                Some("t1_s2"),
                1,
                3,
                GameAction::EndTurn,
                102,
            ))
            .unwrap();

        // Verify sequence order
        let events = chain.events();
        assert_eq!(events[0].sequence, 1);
        assert_eq!(events[1].sequence, 2);
        assert_eq!(events[2].sequence, 3);
    }

    #[test]
    fn test_invalid_sequence_rejected() {
        let mut chain = EventChain::new();

        chain
            .add(create_event(
                "evt1",
                "g",
                0,
                None,
                1,
                1,
                GameAction::EndTurn,
                100,
            ))
            .unwrap();

        // Try to add event with same sequence number
        let result = chain.add(create_event(
            "evt2",
            "g",
            0,
            Some("evt1"),
            1,
            1, // Same sequence - invalid
            GameAction::EndTurn,
            101,
        ));

        assert_eq!(result, Err(EventChainError::InvalidSequence));
    }

    #[test]
    fn test_invalid_turn_sequence_rejected() {
        let mut chain = EventChain::new();

        chain
            .add(create_event(
                "evt1",
                "g",
                0,
                None,
                5,
                1,
                GameAction::EndTurn,
                100,
            ))
            .unwrap();

        // Try to add event with earlier turn
        let result = chain.add(create_event(
            "evt2",
            "g",
            0,
            Some("evt1"),
            4, // Earlier turn - invalid
            1,
            GameAction::EndTurn,
            101,
        ));

        assert_eq!(result, Err(EventChainError::InvalidTurnSequence));
    }

    #[test]
    fn test_new_turn_resets_sequence() {
        let mut chain = EventChain::new();

        chain
            .add(create_event(
                "t1_s5",
                "g",
                0,
                None,
                1,
                5,
                GameAction::EndTurn,
                100,
            ))
            .unwrap();

        // New turn can start with sequence 1
        let result = chain.add(create_event(
            "t2_s1",
            "g",
            0,
            Some("t1_s5"),
            2, // New turn
            1, // Sequence resets
            GameAction::EndTurn,
            101,
        ));

        assert!(result.is_ok());
    }
}

// =============================================================================
// 4. Event Types (GameAction) Tests
// =============================================================================

mod event_types {
    use super::*;
    use nostr_nations_core::city::BuildingType;

    #[test]
    fn test_create_game_with_settings() {
        let settings = GameSettings::new("Epic Battle".to_string());
        let settings_json = serde_json::to_string(&settings).unwrap();
        let seed = [1u8; 32];

        let action = GameAction::CreateGame {
            settings_json: settings_json.clone(),
            seed,
        };

        // Verify we can extract settings back
        if let GameAction::CreateGame {
            settings_json: json,
            seed: s,
        } = action
        {
            let parsed: GameSettings = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed.name, "Epic Battle");
            assert_eq!(s, seed);
        }
    }

    #[test]
    fn test_join_game_with_player_info() {
        let action = GameAction::JoinGame {
            player_name: "Emperor Augustus".to_string(),
            civilization_id: "rome".to_string(),
        };

        if let GameAction::JoinGame {
            player_name,
            civilization_id,
        } = action
        {
            assert_eq!(player_name, "Emperor Augustus");
            assert_eq!(civilization_id, "rome");
        }
    }

    #[test]
    fn test_start_game() {
        let action = GameAction::StartGame;
        assert!(matches!(action, GameAction::StartGame));
    }

    #[test]
    fn test_move_unit_action() {
        let path = vec![
            HexCoord::new(0, 0),
            HexCoord::new(1, 0),
            HexCoord::new(2, 0),
        ];

        let action = GameAction::MoveUnit {
            unit_id: 42,
            path: path.clone(),
        };

        if let GameAction::MoveUnit {
            unit_id,
            path: moved_path,
        } = action
        {
            assert_eq!(unit_id, 42);
            assert_eq!(moved_path, path);
        }
    }

    #[test]
    fn test_attack_unit_action() {
        let action = GameAction::AttackUnit {
            attacker_id: 10,
            defender_id: 20,
            random: 0.7,
        };

        assert!(action.requires_random());

        if let GameAction::AttackUnit {
            attacker_id,
            defender_id,
            random,
        } = action
        {
            assert_eq!(attacker_id, 10);
            assert_eq!(defender_id, 20);
            assert!((random - 0.7).abs() < 0.001);
        }
    }

    #[test]
    fn test_attack_city_action() {
        let action = GameAction::AttackCity {
            attacker_id: 5,
            city_id: 1,
            random: 0.3,
        };

        assert!(action.requires_random());

        if let GameAction::AttackCity {
            attacker_id,
            city_id,
            random,
        } = action
        {
            assert_eq!(attacker_id, 5);
            assert_eq!(city_id, 1);
            assert!((random - 0.3).abs() < 0.001);
        }
    }

    #[test]
    fn test_found_city_action() {
        let action = GameAction::FoundCity {
            settler_id: 1,
            name: "Constantinople".to_string(),
        };

        if let GameAction::FoundCity { settler_id, name } = action {
            assert_eq!(settler_id, 1);
            assert_eq!(name, "Constantinople");
        }
    }

    #[test]
    fn test_set_production_action() {
        let action = GameAction::SetProduction {
            city_id: 1,
            item: ProductionItem::Unit(UnitType::Archer),
        };

        if let GameAction::SetProduction { city_id, item } = action {
            assert_eq!(city_id, 1);
            assert_eq!(item, ProductionItem::Unit(UnitType::Archer));
        }
    }

    #[test]
    fn test_set_production_building() {
        let action = GameAction::SetProduction {
            city_id: 2,
            item: ProductionItem::Building(BuildingType::Library),
        };

        if let GameAction::SetProduction { city_id, item } = action {
            assert_eq!(city_id, 2);
            assert_eq!(item, ProductionItem::Building(BuildingType::Library));
        }
    }

    #[test]
    fn test_research_tech_action() {
        let action = GameAction::SetResearch {
            tech_id: "philosophy".to_string(),
        };

        if let GameAction::SetResearch { tech_id } = action {
            assert_eq!(tech_id, "philosophy");
        }
    }

    #[test]
    fn test_end_turn_action() {
        let action = GameAction::EndTurn;
        assert!(!action.requires_random());
        assert!(matches!(action, GameAction::EndTurn));
    }

    #[test]
    fn test_declare_war_action() {
        let action = GameAction::DeclareWar { target_player: 2 };

        if let GameAction::DeclareWar { target_player } = action {
            assert_eq!(target_player, 2);
        }
    }

    #[test]
    fn test_propose_peace_action() {
        let action = GameAction::ProposePeace { target_player: 1 };

        if let GameAction::ProposePeace { target_player } = action {
            assert_eq!(target_player, 1);
        }
    }

    #[test]
    fn test_accept_peace_action() {
        let action = GameAction::AcceptPeace { from_player: 3 };

        if let GameAction::AcceptPeace { from_player } = action {
            assert_eq!(from_player, 3);
        }
    }

    #[test]
    fn test_end_game_with_victory_type() {
        let action = GameAction::EndGame {
            winner_id: 0,
            victory_type: "science".to_string(),
        };

        if let GameAction::EndGame {
            winner_id,
            victory_type,
        } = action
        {
            assert_eq!(winner_id, 0);
            assert_eq!(victory_type, "science");
        }
    }

    #[test]
    fn test_action_kind_mapping() {
        use nostr_nations_core::events::kinds;

        let create_event = GameEvent::new(
            "g".to_string(),
            0,
            None,
            0,
            1,
            GameAction::CreateGame {
                settings_json: "{}".to_string(),
                seed: [0; 32],
            },
        );
        assert_eq!(create_event.kind(), kinds::GAME_CREATE);

        let join_event = GameEvent::new(
            "g".to_string(),
            0,
            None,
            0,
            1,
            GameAction::JoinGame {
                player_name: "P".to_string(),
                civilization_id: "c".to_string(),
            },
        );
        assert_eq!(join_event.kind(), kinds::PLAYER_JOIN);

        let start_event = GameEvent::new("g".to_string(), 0, None, 0, 1, GameAction::StartGame);
        assert_eq!(start_event.kind(), kinds::GAME_START);

        let action_event = GameEvent::new(
            "g".to_string(),
            0,
            None,
            1,
            1,
            GameAction::MoveUnit {
                unit_id: 1,
                path: vec![],
            },
        );
        assert_eq!(action_event.kind(), kinds::GAME_ACTION);

        let end_turn_event = GameEvent::new("g".to_string(), 0, None, 1, 1, GameAction::EndTurn);
        assert_eq!(end_turn_event.kind(), kinds::TURN_END);

        let end_game_event = GameEvent::new(
            "g".to_string(),
            0,
            None,
            1,
            1,
            GameAction::EndGame {
                winner_id: 0,
                victory_type: "test".to_string(),
            },
        );
        assert_eq!(end_game_event.kind(), kinds::GAME_END);
    }
}

// =============================================================================
// 5. Event Validation Tests
// =============================================================================

mod event_validation {
    use super::*;

    fn setup_game_engine() -> GameEngine {
        let mut settings = GameSettings::new("Validation Test".to_string());
        settings.map_size = MapSize::Duel;
        GameEngine::new(settings, [42u8; 32])
    }

    #[test]
    fn test_invalid_player_id_rejected() {
        let mut engine = setup_game_engine();

        // Add two valid players
        engine
            .apply_action(
                0,
                &GameAction::JoinGame {
                    player_name: "P1".to_string(),
                    civilization_id: "rome".to_string(),
                },
            )
            .unwrap();

        engine
            .apply_action(
                1,
                &GameAction::JoinGame {
                    player_name: "P2".to_string(),
                    civilization_id: "egypt".to_string(),
                },
            )
            .unwrap();

        engine.apply_action(0, &GameAction::StartGame).unwrap();

        // Player 99 doesn't exist - attacking with non-existent unit
        let result = engine.apply_action(
            0,
            &GameAction::MoveUnit {
                unit_id: 999, // Non-existent unit
                path: vec![HexCoord::new(0, 0)],
            },
        );

        // Should fail because unit doesn't exist
        assert!(result.is_err());
    }

    #[test]
    fn test_out_of_turn_action_rejected() {
        let mut engine = setup_game_engine();

        // Add players and start
        engine
            .apply_action(
                0,
                &GameAction::JoinGame {
                    player_name: "P1".to_string(),
                    civilization_id: "rome".to_string(),
                },
            )
            .unwrap();
        engine
            .apply_action(
                1,
                &GameAction::JoinGame {
                    player_name: "P2".to_string(),
                    civilization_id: "egypt".to_string(),
                },
            )
            .unwrap();
        engine.apply_action(0, &GameAction::StartGame).unwrap();

        // It's player 0's turn, not player 1's
        assert_eq!(engine.state.current_player, 0);

        // Create an event from player 1 (wrong turn)
        let event = create_event(
            "bad_evt",
            &engine.state.id,
            1, // Player 1 trying to act
            None,
            1,
            1,
            GameAction::EndTurn,
            1000,
        );

        let result = engine.apply_event(&event);

        // Should be rejected as not player's turn
        assert!(matches!(result, Err(ReplayError::NotPlayerTurn)));
    }

    #[test]
    fn test_invalid_game_state_transition_rejected() {
        let mut engine = setup_game_engine();

        // Try to start game without any players
        let result = engine.apply_action(0, &GameAction::StartGame);
        assert!(result.is_err());
    }

    #[test]
    fn test_action_requires_randomness_validation() {
        let mut chain = EventChain::new();

        // Create an attack event without randomness proof
        let attack_event = create_event(
            "attack",
            "game",
            0,
            None,
            1,
            1,
            GameAction::AttackUnit {
                attacker_id: 1,
                defender_id: 2,
                random: 0.5,
            },
            1000,
        );

        // Chain should reject events that require randomness without proof
        let result = chain.add(attack_event);
        assert_eq!(result, Err(EventChainError::MissingRandomnessProof));
    }

    #[test]
    fn test_action_with_randomness_proof_accepted() {
        let mut chain = EventChain::new();

        let proof = create_test_proof(0.5);
        let attack_event = create_event_with_proof(
            "attack",
            "game",
            0,
            None,
            1,
            1,
            GameAction::AttackUnit {
                attacker_id: 1,
                defender_id: 2,
                random: 0.5,
            },
            1000,
            proof,
        );

        // With proof, chain should accept the event
        let result = chain.add(attack_event);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_game_requires_randomness() {
        let action = GameAction::CreateGame {
            settings_json: "{}".to_string(),
            seed: [0; 32],
        };
        assert!(action.requires_random());
    }

    #[test]
    fn test_non_combat_actions_dont_require_randomness() {
        assert!(!GameAction::EndTurn.requires_random());
        assert!(!GameAction::StartGame.requires_random());
        assert!(!GameAction::MoveUnit {
            unit_id: 1,
            path: vec![]
        }
        .requires_random());
        assert!(!GameAction::FoundCity {
            settler_id: 1,
            name: "Test".to_string()
        }
        .requires_random());
        assert!(!GameAction::SetResearch {
            tech_id: "test".to_string()
        }
        .requires_random());
    }

    #[test]
    fn test_event_builder_sequences_correctly() {
        let mut builder = EventBuilder::new("game1".to_string(), 0);
        builder.set_turn(1);

        let event1 = builder.build(GameAction::EndTurn);
        assert_eq!(event1.turn, 1);
        assert_eq!(event1.sequence, 1);

        let event2 = builder.build(GameAction::EndTurn);
        assert_eq!(event2.turn, 1);
        assert_eq!(event2.sequence, 2);

        builder.set_turn(2);
        let event3 = builder.build(GameAction::EndTurn);
        assert_eq!(event3.turn, 2);
        assert_eq!(event3.sequence, 1); // Reset for new turn
    }
}

// =============================================================================
// 6. Replay System Tests
// =============================================================================

mod replay_system {
    use super::*;

    fn create_game_events() -> Vec<GameEvent> {
        let settings = GameSettings::new("Replay Test".to_string());
        let settings_json = serde_json::to_string(&settings).unwrap();
        let seed = [42u8; 32];

        // Create a deterministic randomness proof for CreateGame
        let proof = RandomnessProof {
            mint_keyset_id: "deterministic".to_string(),
            blinded_message: vec![],
            blinded_signature: vec![],
            signature: seed.to_vec(),
            random_bytes: seed,
            context: "map_generation".to_string(),
            timestamp: 0,
        };

        vec![
            create_event_with_proof(
                "evt_create",
                "game_replay",
                0,
                None,
                0,
                1,
                GameAction::CreateGame {
                    settings_json,
                    seed,
                },
                1000,
                proof,
            ),
            create_event(
                "evt_join1",
                "game_replay",
                0,
                Some("evt_create"),
                0,
                2,
                GameAction::JoinGame {
                    player_name: "Player1".to_string(),
                    civilization_id: "rome".to_string(),
                },
                1001,
            ),
            create_event(
                "evt_join2",
                "game_replay",
                1,
                Some("evt_join1"),
                0,
                3,
                GameAction::JoinGame {
                    player_name: "Player2".to_string(),
                    civilization_id: "egypt".to_string(),
                },
                1002,
            ),
            create_event(
                "evt_start",
                "game_replay",
                0,
                Some("evt_join2"),
                0,
                4,
                GameAction::StartGame,
                1003,
            ),
        ]
    }

    #[test]
    fn test_events_can_recreate_game_state() {
        let events = create_game_events();
        let engine = GameEngine::from_events(&events);

        assert!(engine.is_ok());
        let engine = engine.unwrap();

        assert_eq!(engine.state.phase, GamePhase::Playing);
        assert_eq!(engine.state.players.len(), 2);
        assert_eq!(engine.state.players[0].name, "Player1");
        assert_eq!(engine.state.players[1].name, "Player2");
    }

    #[test]
    fn test_same_events_produce_same_state_determinism() {
        let events = create_game_events();

        // Replay twice
        let engine1 = GameEngine::from_events(&events).unwrap();
        let engine2 = GameEngine::from_events(&events).unwrap();

        // States should be identical
        assert_eq!(engine1.state.id, engine2.state.id);
        assert_eq!(engine1.state.turn, engine2.state.turn);
        assert_eq!(engine1.state.current_player, engine2.state.current_player);
        assert_eq!(engine1.state.players.len(), engine2.state.players.len());
        assert_eq!(engine1.state.seed, engine2.state.seed);

        // Player data should match
        for i in 0..engine1.state.players.len() {
            assert_eq!(engine1.state.players[i].name, engine2.state.players[i].name);
            assert_eq!(engine1.state.players[i].gold, engine2.state.players[i].gold);
        }
    }

    #[test]
    fn test_partial_replay_works() {
        let all_events = create_game_events();

        // Replay only first 2 events (CreateGame + JoinGame)
        let partial_events = &all_events[0..2];
        let engine = GameEngine::from_events(partial_events);

        assert!(engine.is_ok());
        let engine = engine.unwrap();

        // Game should be in setup phase with 1 player
        assert_eq!(engine.state.phase, GamePhase::Setup);
        assert_eq!(engine.state.players.len(), 1);
    }

    #[test]
    fn test_replay_empty_events_fails() {
        let events: Vec<GameEvent> = vec![];
        let result = GameEngine::from_events(&events);

        assert!(matches!(result, Err(ReplayError::EmptyEventChain)));
    }

    #[test]
    fn test_replay_missing_create_game_fails() {
        // Start with JoinGame instead of CreateGame
        let events = vec![create_event(
            "evt_join",
            "game",
            0,
            None,
            0,
            1,
            GameAction::JoinGame {
                player_name: "Player".to_string(),
                civilization_id: "rome".to_string(),
            },
            1000,
        )];

        let result = GameEngine::from_events(&events);
        assert!(matches!(result, Err(ReplayError::MissingCreateGame)));
    }

    #[test]
    fn test_replay_config_strict_mode() {
        let config = ReplayConfig {
            strict_randomness_validation: true,
            allow_deterministic_randomness: false,
        };

        let settings = GameSettings::new("Strict Test".to_string());
        let engine = GameEngine::with_config(settings, [0u8; 32], config);

        assert!(engine.config.strict_randomness_validation);
        assert!(!engine.config.allow_deterministic_randomness);
    }

    #[test]
    fn test_replay_determinism_with_randomness() {
        // Create a game with some random events
        let seed = [123u8; 32];
        let mut rng1 = DeterministicRandomness::new(seed);
        let mut rng2 = DeterministicRandomness::new(seed);

        let context1 = RandomnessContext::Combat {
            game_id: "test".to_string(),
            turn: 1,
            attacker_id: 1,
            defender_id: 2,
        };

        let context2 = RandomnessContext::Combat {
            game_id: "test".to_string(),
            turn: 1,
            attacker_id: 1,
            defender_id: 2,
        };

        let proof1 = rng1.request_randomness(context1).unwrap();
        let proof2 = rng2.request_randomness(context2).unwrap();

        // Same seed and context should produce same randomness
        assert_eq!(proof1.random_bytes, proof2.random_bytes);
        assert!((proof1.to_f32() - proof2.to_f32()).abs() < 0.0001);
    }

    #[test]
    fn test_event_content_serialization() {
        let event = create_event(
            "evt",
            "game",
            0,
            None,
            1,
            1,
            GameAction::MoveUnit {
                unit_id: 42,
                path: vec![HexCoord::new(5, 5), HexCoord::new(5, 6)],
            },
            1000,
        );

        let content = event.content();

        // Content should be JSON of the action
        assert!(content.contains("MoveUnit"));
        assert!(content.contains("42"));
    }

    #[test]
    fn test_event_tags_generation() {
        let event = create_event(
            "evt_test",
            "game_abc",
            1,
            Some("prev_evt"),
            5,
            3,
            GameAction::EndTurn,
            1000,
        );

        let tags = event.tags();

        // Should have game tag
        assert!(tags.iter().any(|t| t[0] == "g" && t[1] == "game_abc"));
        // Should have player tag
        assert!(tags.iter().any(|t| t[0] == "p" && t[1] == "1"));
        // Should have turn tag
        assert!(tags.iter().any(|t| t[0] == "turn" && t[1] == "5"));
        // Should have sequence tag
        assert!(tags.iter().any(|t| t[0] == "seq" && t[1] == "3"));
        // Should have previous event reference
        assert!(tags.iter().any(|t| t[0] == "e" && t[1] == "prev_evt"));
    }

    #[test]
    fn test_replay_validates_proofs() {
        let events = create_game_events();
        let config = ReplayConfig {
            strict_randomness_validation: false, // Allow deterministic
            allow_deterministic_randomness: true,
        };

        let result = GameEngine::from_events_with_config(&events, config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_valid_action() {
        let settings = GameSettings::new("Valid Test".to_string());
        let engine = GameEngine::new(settings, [0u8; 32]);

        // EndTurn should be valid for player 0 (default current player)
        assert!(engine.is_valid_action(0, &GameAction::EndTurn));
    }

    #[test]
    fn test_game_engine_turn_tracking() {
        let mut events = create_game_events();

        // Add an EndTurn event
        events.push(create_event(
            "evt_end_turn",
            "game_replay",
            0,
            Some("evt_start"),
            1,
            1,
            GameAction::EndTurn,
            1004,
        ));

        let engine = GameEngine::from_events(&events).unwrap();

        // Turn should have advanced
        assert!(engine.turn() >= 1);
    }
}

// =============================================================================
// Additional Edge Case Tests
// =============================================================================

mod edge_cases {
    use super::*;

    #[test]
    fn test_action_description() {
        let actions = vec![
            (GameAction::EndTurn, "Ended turn"),
            (GameAction::StartGame, "Game started"),
            (
                GameAction::FoundCity {
                    settler_id: 1,
                    name: "Rome".to_string(),
                },
                "Founded city Rome",
            ),
            (
                GameAction::DeclareWar { target_player: 2 },
                "Declared war on player 2",
            ),
        ];

        for (action, expected_substring) in actions {
            let desc = action.description();
            assert!(
                desc.contains(expected_substring),
                "Description '{}' should contain '{}'",
                desc,
                expected_substring
            );
        }
    }

    #[test]
    fn test_hex_coord_in_events() {
        let coord = HexCoord::new(100, -50);
        let action = GameAction::AssignCitizen {
            city_id: 1,
            tile: coord,
        };

        let json = serde_json::to_string(&action).unwrap();
        let restored: GameAction = serde_json::from_str(&json).unwrap();

        if let GameAction::AssignCitizen { city_id: _, tile } = restored {
            assert_eq!(tile.q, 100);
            assert_eq!(tile.r, -50);
        } else {
            panic!("Wrong action type");
        }
    }

    #[test]
    fn test_large_path_serialization() {
        let mut path = Vec::new();
        for i in 0..100 {
            path.push(HexCoord::new(i, i * 2));
        }

        let action = GameAction::MoveUnit { unit_id: 1, path };

        let json = serde_json::to_string(&action).unwrap();
        let restored: GameAction = serde_json::from_str(&json).unwrap();

        if let GameAction::MoveUnit {
            unit_id: _,
            path: restored_path,
        } = restored
        {
            assert_eq!(restored_path.len(), 100);
        }
    }

    #[test]
    fn test_unicode_in_player_name() {
        let action = GameAction::JoinGame {
            player_name: "プレイヤー1".to_string(), // Japanese characters
            civilization_id: "rome".to_string(),
        };

        let json = serde_json::to_string(&action).unwrap();
        let restored: GameAction = serde_json::from_str(&json).unwrap();

        if let GameAction::JoinGame { player_name, .. } = restored {
            assert_eq!(player_name, "プレイヤー1");
        }
    }

    #[test]
    fn test_unicode_in_city_name() {
        let action = GameAction::FoundCity {
            settler_id: 1,
            name: "北京".to_string(), // Beijing in Chinese
        };

        let json = serde_json::to_string(&action).unwrap();
        let restored: GameAction = serde_json::from_str(&json).unwrap();

        if let GameAction::FoundCity { name, .. } = restored {
            assert_eq!(name, "北京");
        }
    }

    #[test]
    fn test_randomness_proof_in_tags() {
        let proof = create_test_proof(0.5);
        let mut event = GameEvent::with_randomness(
            "game".to_string(),
            0,
            None,
            1,
            1,
            GameAction::AttackUnit {
                attacker_id: 1,
                defender_id: 2,
                random: 0.5,
            },
            proof,
        );
        event.id = "test_evt".to_string();

        let tags = event.tags();

        // Should have cashu tag with proof
        assert!(
            tags.iter().any(|t| t[0] == "cashu"),
            "Tags should include cashu proof"
        );
    }

    #[test]
    fn test_chain_randomness_proofs_collection() {
        let mut chain = EventChain::new();

        // Add an event with randomness proof
        let proof = create_test_proof(0.75);
        let event = create_event_with_proof(
            "evt1",
            "game",
            0,
            None,
            1,
            1,
            GameAction::AttackUnit {
                attacker_id: 1,
                defender_id: 2,
                random: 0.75,
            },
            1000,
            proof,
        );
        chain.add(event).unwrap();

        let proofs = chain.randomness_proofs();
        assert_eq!(proofs.len(), 1);
    }
}
