//! Bevy plugins for Nostr Nations.
//!
//! Plugins bundle related systems, resources, and configuration
//! for modular initialization of the game.

use bevy::prelude::*;
use nostr_nations_core::GameSettings;

use crate::resources::{
    CameraState, CityEntityMap, CurrentTurn, GameSettingsResource, GameStateResource,
    PendingAction, SelectedEntity, TileEntityMap, UiState, UnitEntityMap,
};
use crate::systems::{
    despawn_removed_entities_system, game_tick_system, movement_animation_system,
    pending_action_system, selection_changed_system, selection_system, spawn_new_entities_system,
    sync_game_state_system, turn_system, visibility_system, GameSystemSet,
};

/// Main plugin for Nostr Nations game.
///
/// This plugin initializes all game systems and resources required
/// to run the Nostr Nations game in Bevy.
///
/// # Usage
///
/// ```ignore
/// use bevy::prelude::*;
/// use nostr_nations_bevy::plugins::NostrNationsPlugin;
///
/// fn main() {
///     App::new()
///         .add_plugins(DefaultPlugins)
///         .add_plugins(NostrNationsPlugin::default())
///         .run();
/// }
/// ```
#[derive(Default)]
pub struct NostrNationsPlugin {
    /// Game settings to use when initializing.
    pub settings: GameSettings,
    /// Random seed for deterministic game initialization.
    pub seed: [u8; 32],
    /// Local player's ID.
    pub local_player_id: u8,
    /// Whether this is a networked game.
    pub is_networked: bool,
}

impl NostrNationsPlugin {
    /// Create a new plugin with custom settings.
    pub fn new(settings: GameSettings, seed: [u8; 32], local_player_id: u8) -> Self {
        Self {
            settings,
            seed,
            local_player_id,
            is_networked: false,
        }
    }

    /// Create a plugin for a networked game.
    pub fn networked(settings: GameSettings, seed: [u8; 32], local_player_id: u8) -> Self {
        Self {
            settings,
            seed,
            local_player_id,
            is_networked: true,
        }
    }

    /// Create a plugin for a local (offline) game.
    pub fn local(settings: GameSettings, seed: [u8; 32]) -> Self {
        Self {
            settings,
            seed,
            local_player_id: 0,
            is_networked: false,
        }
    }
}

impl Plugin for NostrNationsPlugin {
    fn build(&self, app: &mut App) {
        // Add sub-plugins
        app.add_plugins((
            GameStatePlugin {
                settings: self.settings.clone(),
                seed: self.seed,
                local_player_id: self.local_player_id,
                is_networked: self.is_networked,
            },
            SelectionPlugin,
            VisibilityPlugin,
            AnimationPlugin,
        ));

        // Configure system sets
        app.configure_sets(
            Update,
            (
                GameSystemSet::Input,
                GameSystemSet::Update,
                GameSystemSet::Sync,
                GameSystemSet::Animation,
            )
                .chain(),
        );

        // Add game logic systems
        app.add_systems(
            Update,
            (game_tick_system, turn_system, pending_action_system)
                .chain()
                .in_set(GameSystemSet::Update),
        );

        // Add synchronization systems
        app.add_systems(
            Update,
            (
                sync_game_state_system,
                spawn_new_entities_system,
                despawn_removed_entities_system,
            )
                .chain()
                .in_set(GameSystemSet::Sync),
        );
    }
}

/// Plugin for game state management.
///
/// This plugin initializes the core game state resources and
/// provides the foundation for game logic.
#[derive(Default)]
pub struct GameStatePlugin {
    /// Game settings.
    pub settings: GameSettings,
    /// Random seed.
    pub seed: [u8; 32],
    /// Local player ID.
    pub local_player_id: u8,
    /// Whether networked.
    pub is_networked: bool,
}

impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        // Initialize game state resource
        let game_state = GameStateResource::new(self.settings.clone(), self.seed);

        // Initialize game settings resource
        let game_settings = if self.is_networked {
            GameSettingsResource::networked(self.settings.clone(), self.local_player_id)
        } else {
            GameSettingsResource::local(self.settings.clone(), self.local_player_id)
        };

        // Initialize turn state
        let current_turn = CurrentTurn::new(0, 0);

        // Add resources
        app.insert_resource(game_state)
            .insert_resource(game_settings)
            .insert_resource(current_turn)
            .insert_resource(PendingAction::default())
            .insert_resource(TileEntityMap::default())
            .insert_resource(UnitEntityMap::default())
            .insert_resource(CityEntityMap::default());

        // Add game state events
        app.add_event::<GameStateEvent>();
    }
}

/// Plugin for entity selection functionality.
///
/// This plugin provides the selection system and related resources
/// for tracking which entities the player has selected.
pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        // Add selection resource
        app.insert_resource(SelectedEntity::default());

        // Add selection systems
        app.add_systems(
            Update,
            (selection_system, selection_changed_system)
                .chain()
                .in_set(GameSystemSet::Input),
        );

        // Add selection events
        app.add_event::<SelectionEvent>();
    }
}

/// Plugin for fog of war and visibility.
///
/// This plugin manages which tiles are visible to the player
/// based on unit positions and exploration.
pub struct VisibilityPlugin;

impl Plugin for VisibilityPlugin {
    fn build(&self, app: &mut App) {
        // Add visibility system
        app.add_systems(Update, visibility_system.in_set(GameSystemSet::Update));
    }
}

/// Plugin for animations.
///
/// This plugin handles smooth animations for unit movement,
/// combat effects, and other visual feedback.
pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        // Add animation systems
        app.add_systems(
            Update,
            movement_animation_system.in_set(GameSystemSet::Animation),
        );
    }
}

/// Plugin for camera controls.
///
/// This plugin provides camera movement, zoom, and panning
/// functionality for the game map.
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        // Add camera state resource
        app.insert_resource(CameraState::default());

        // Camera systems would be added here
        // (camera_movement_system, camera_zoom_system, etc.)
    }
}

/// Plugin for UI systems.
///
/// This plugin manages the user interface including
/// panels, menus, and HUD elements.
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        // Add UI state resource
        app.insert_resource(UiState::default());

        // UI systems would be added here
        // (ui_panel_system, tooltip_system, etc.)
    }
}

/// Event fired when game state changes significantly.
#[derive(Event, Clone, Debug)]
pub enum GameStateEvent {
    /// Game has started.
    GameStarted,
    /// A new turn has begun.
    TurnStarted {
        /// Turn number.
        turn: u32,
        /// Current player.
        player_id: u8,
    },
    /// A unit was created.
    UnitCreated {
        /// Unit ID.
        unit_id: u64,
    },
    /// A unit was destroyed.
    UnitDestroyed {
        /// Unit ID.
        unit_id: u64,
    },
    /// A city was founded.
    CityFounded {
        /// City ID.
        city_id: u64,
        /// City name.
        name: String,
    },
    /// A city was captured.
    CityCaptured {
        /// City ID.
        city_id: u64,
        /// New owner.
        new_owner: u8,
    },
    /// A technology was researched.
    TechResearched {
        /// Player ID.
        player_id: u8,
        /// Technology ID.
        tech_id: String,
    },
    /// Game has ended.
    GameEnded {
        /// Winner player ID.
        winner_id: u8,
        /// Victory type.
        victory_type: String,
    },
}

/// Event fired when selection changes.
#[derive(Event, Clone, Debug)]
pub enum SelectionEvent {
    /// An entity was selected.
    Selected {
        /// The selected entity.
        entity: Entity,
    },
    /// Selection was cleared.
    Cleared,
}

/// Helper function to create a minimal Bevy app for testing.
#[cfg(test)]
pub fn create_test_app() -> App {
    use bevy::input::keyboard::KeyCode;
    use bevy::input::mouse::MouseButton;
    use bevy::input::ButtonInput;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    // Add input resources required by systems (selection_system, turn_system)
    app.init_resource::<ButtonInput<MouseButton>>();
    app.init_resource::<ButtonInput<KeyCode>>();
    app.add_plugins(NostrNationsPlugin::default());
    app
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================
    // NostrNationsPlugin Tests
    // ============================================

    #[test]
    fn test_nostr_nations_plugin_default() {
        let plugin = NostrNationsPlugin::default();
        assert_eq!(plugin.local_player_id, 0);
        assert!(!plugin.is_networked);
        assert_eq!(plugin.seed, [0u8; 32]);
    }

    #[test]
    fn test_nostr_nations_plugin_new() {
        let settings = GameSettings::new("Custom Game".to_string());
        let seed = [42u8; 32];
        let plugin = NostrNationsPlugin::new(settings, seed, 2);

        assert_eq!(plugin.local_player_id, 2);
        assert!(!plugin.is_networked);
        assert_eq!(plugin.seed, seed);
    }

    #[test]
    fn test_nostr_nations_plugin_local() {
        let settings = GameSettings::new("Local Test".to_string());
        let plugin = NostrNationsPlugin::local(settings, [42u8; 32]);

        assert_eq!(plugin.local_player_id, 0);
        assert!(!plugin.is_networked);
    }

    #[test]
    fn test_nostr_nations_plugin_networked() {
        let settings = GameSettings::new("Networked Test".to_string());
        let plugin = NostrNationsPlugin::networked(settings, [42u8; 32], 3);

        assert_eq!(plugin.local_player_id, 3);
        assert!(plugin.is_networked);
    }

    #[test]
    fn test_nostr_nations_plugin_builds() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(NostrNationsPlugin::default());

        // Verify all expected resources are present
        assert!(app.world().contains_resource::<GameStateResource>());
        assert!(app.world().contains_resource::<GameSettingsResource>());
        assert!(app.world().contains_resource::<CurrentTurn>());
        assert!(app.world().contains_resource::<SelectedEntity>());
        assert!(app.world().contains_resource::<TileEntityMap>());
        assert!(app.world().contains_resource::<UnitEntityMap>());
        assert!(app.world().contains_resource::<CityEntityMap>());
        assert!(app.world().contains_resource::<PendingAction>());
    }

    #[test]
    fn test_nostr_nations_plugin_with_custom_settings() {
        let settings = GameSettings::duel("Duel Match".to_string());
        let seed = [1u8; 32];
        let plugin = NostrNationsPlugin::local(settings, seed);

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(plugin);

        // Verify plugin was built
        assert!(app.world().contains_resource::<GameStateResource>());
    }

    #[test]
    fn test_nostr_nations_plugin_networked_settings() {
        let settings = GameSettings::new("Multiplayer".to_string());
        let seed = [5u8; 32];
        let plugin = NostrNationsPlugin::networked(settings, seed, 1);

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(plugin);

        // Verify networked settings
        let game_settings = app.world().resource::<GameSettingsResource>();
        assert!(game_settings.is_networked);
        assert_eq!(game_settings.local_player_id, 1);
    }

    // ============================================
    // GameStatePlugin Tests
    // ============================================

    #[test]
    fn test_game_state_plugin_default() {
        let plugin = GameStatePlugin::default();
        assert_eq!(plugin.local_player_id, 0);
        assert!(!plugin.is_networked);
        assert_eq!(plugin.seed, [0u8; 32]);
    }

    #[test]
    fn test_game_state_plugin_builds() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(GameStatePlugin::default());

        // Check all expected resources
        assert!(app.world().contains_resource::<GameStateResource>());
        assert!(app.world().contains_resource::<GameSettingsResource>());
        assert!(app.world().contains_resource::<CurrentTurn>());
        assert!(app.world().contains_resource::<PendingAction>());
        assert!(app.world().contains_resource::<TileEntityMap>());
        assert!(app.world().contains_resource::<UnitEntityMap>());
        assert!(app.world().contains_resource::<CityEntityMap>());
    }

    #[test]
    fn test_game_state_plugin_with_custom_seed() {
        let settings = GameSettings::new("Seeded Game".to_string());
        let seed = [42u8; 32];

        let plugin = GameStatePlugin {
            settings,
            seed,
            local_player_id: 0,
            is_networked: false,
        };

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(plugin);

        assert!(app.world().contains_resource::<GameStateResource>());
    }

    #[test]
    fn test_game_state_plugin_local_settings() {
        let settings = GameSettings::new("Local".to_string());

        let plugin = GameStatePlugin {
            settings: settings.clone(),
            seed: [0u8; 32],
            local_player_id: 0,
            is_networked: false,
        };

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(plugin);

        let game_settings = app.world().resource::<GameSettingsResource>();
        assert!(!game_settings.is_networked);
    }

    #[test]
    fn test_game_state_plugin_networked_settings() {
        let settings = GameSettings::new("Networked".to_string());

        let plugin = GameStatePlugin {
            settings: settings.clone(),
            seed: [0u8; 32],
            local_player_id: 2,
            is_networked: true,
        };

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(plugin);

        let game_settings = app.world().resource::<GameSettingsResource>();
        assert!(game_settings.is_networked);
        assert_eq!(game_settings.local_player_id, 2);
    }

    #[test]
    fn test_game_state_plugin_registers_event() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(GameStatePlugin::default());

        // The event type should be registered
        // We can verify by checking the app can handle the event
        app.update();
    }

    // ============================================
    // SelectionPlugin Tests
    // ============================================

    #[test]
    fn test_selection_plugin_builds() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(SelectionPlugin);

        assert!(app.world().contains_resource::<SelectedEntity>());
    }

    #[test]
    fn test_selection_plugin_resource_initialized() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(SelectionPlugin);

        let selected = app.world().resource::<SelectedEntity>();
        assert!(!selected.has_selection());
    }

    // ============================================
    // VisibilityPlugin Tests
    // ============================================

    #[test]
    fn test_visibility_plugin_builds() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // VisibilityPlugin needs GameStateResource and GameSettingsResource
        app.insert_resource(GameStateResource::default());
        app.insert_resource(GameSettingsResource::default());

        app.add_plugins(VisibilityPlugin);

        // Plugin should build without panic
    }

    // ============================================
    // AnimationPlugin Tests
    // ============================================

    #[test]
    fn test_animation_plugin_builds() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(AnimationPlugin);

        // Plugin should build without panic
    }

    // ============================================
    // CameraPlugin Tests
    // ============================================

    #[test]
    fn test_camera_plugin_builds() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(CameraPlugin);

        assert!(app.world().contains_resource::<CameraState>());
    }

    #[test]
    fn test_camera_plugin_default_state() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(CameraPlugin);

        let camera_state = app.world().resource::<CameraState>();
        assert_eq!(camera_state.zoom, 1.0);
        assert_eq!(camera_state.position, Vec2::ZERO);
    }

    // ============================================
    // UiPlugin Tests
    // ============================================

    #[test]
    fn test_ui_plugin_builds() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(UiPlugin);

        assert!(app.world().contains_resource::<UiState>());
    }

    #[test]
    fn test_ui_plugin_default_state() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(UiPlugin);

        let ui_state = app.world().resource::<UiState>();
        assert!(!ui_state.menu_open);
        assert!(!ui_state.production_panel_open);
        assert!(!ui_state.tech_tree_open);
    }

    // ============================================
    // GameStateEvent Tests
    // ============================================

    #[test]
    fn test_game_state_event_game_started() {
        let event = GameStateEvent::GameStarted;
        match event {
            GameStateEvent::GameStarted => { /* pass */ }
            _ => panic!("Expected GameStarted event"),
        }
    }

    #[test]
    fn test_game_state_event_turn_started() {
        let event = GameStateEvent::TurnStarted {
            turn: 5,
            player_id: 1,
        };
        match event {
            GameStateEvent::TurnStarted { turn, player_id } => {
                assert_eq!(turn, 5);
                assert_eq!(player_id, 1);
            }
            _ => panic!("Expected TurnStarted event"),
        }
    }

    #[test]
    fn test_game_state_event_unit_created() {
        let event = GameStateEvent::UnitCreated { unit_id: 42 };
        match event {
            GameStateEvent::UnitCreated { unit_id } => {
                assert_eq!(unit_id, 42);
            }
            _ => panic!("Expected UnitCreated event"),
        }
    }

    #[test]
    fn test_game_state_event_unit_destroyed() {
        let event = GameStateEvent::UnitDestroyed { unit_id: 100 };
        match event {
            GameStateEvent::UnitDestroyed { unit_id } => {
                assert_eq!(unit_id, 100);
            }
            _ => panic!("Expected UnitDestroyed event"),
        }
    }

    #[test]
    fn test_game_state_event_city_founded() {
        let event = GameStateEvent::CityFounded {
            city_id: 1,
            name: "Rome".to_string(),
        };
        match event {
            GameStateEvent::CityFounded { city_id, name } => {
                assert_eq!(city_id, 1);
                assert_eq!(name, "Rome");
            }
            _ => panic!("Expected CityFounded event"),
        }
    }

    #[test]
    fn test_game_state_event_city_captured() {
        let event = GameStateEvent::CityCaptured {
            city_id: 5,
            new_owner: 2,
        };
        match event {
            GameStateEvent::CityCaptured { city_id, new_owner } => {
                assert_eq!(city_id, 5);
                assert_eq!(new_owner, 2);
            }
            _ => panic!("Expected CityCaptured event"),
        }
    }

    #[test]
    fn test_game_state_event_tech_researched() {
        let event = GameStateEvent::TechResearched {
            player_id: 0,
            tech_id: "writing".to_string(),
        };
        match event {
            GameStateEvent::TechResearched { player_id, tech_id } => {
                assert_eq!(player_id, 0);
                assert_eq!(tech_id, "writing");
            }
            _ => panic!("Expected TechResearched event"),
        }
    }

    #[test]
    fn test_game_state_event_game_ended() {
        let event = GameStateEvent::GameEnded {
            winner_id: 1,
            victory_type: "domination".to_string(),
        };
        match event {
            GameStateEvent::GameEnded {
                winner_id,
                victory_type,
            } => {
                assert_eq!(winner_id, 1);
                assert_eq!(victory_type, "domination");
            }
            _ => panic!("Expected GameEnded event"),
        }
    }

    #[test]
    fn test_game_state_event_clone() {
        let event = GameStateEvent::TurnStarted {
            turn: 3,
            player_id: 0,
        };
        let cloned = event.clone();
        match cloned {
            GameStateEvent::TurnStarted { turn, player_id } => {
                assert_eq!(turn, 3);
                assert_eq!(player_id, 0);
            }
            _ => panic!("Clone failed"),
        }
    }

    #[test]
    fn test_game_state_event_debug() {
        let event = GameStateEvent::GameStarted;
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("GameStarted"));
    }

    // ============================================
    // SelectionEvent Tests
    // ============================================

    #[test]
    fn test_selection_event_selected() {
        let entity = Entity::from_raw(42);
        let event = SelectionEvent::Selected { entity };
        match event {
            SelectionEvent::Selected { entity: e } => {
                assert_eq!(e, entity);
            }
            _ => panic!("Expected Selected event"),
        }
    }

    #[test]
    fn test_selection_event_cleared() {
        let event = SelectionEvent::Cleared;
        match event {
            SelectionEvent::Cleared => { /* pass */ }
            _ => panic!("Expected Cleared event"),
        }
    }

    #[test]
    fn test_selection_event_clone() {
        let entity = Entity::from_raw(10);
        let event = SelectionEvent::Selected { entity };
        let cloned = event.clone();
        match cloned {
            SelectionEvent::Selected { entity: e } => {
                assert_eq!(e, entity);
            }
            _ => panic!("Clone failed"),
        }
    }

    #[test]
    fn test_selection_event_debug() {
        let event = SelectionEvent::Cleared;
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("Cleared"));
    }

    // ============================================
    // create_test_app Tests
    // ============================================

    #[test]
    fn test_create_test_app() {
        let app = create_test_app();

        // Verify all expected resources
        assert!(app.world().contains_resource::<GameStateResource>());
        assert!(app.world().contains_resource::<SelectedEntity>());
        assert!(app.world().contains_resource::<CurrentTurn>());
    }

    // ============================================
    // Plugin Combinations Tests
    // ============================================

    #[test]
    fn test_multiple_plugins_together() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Add multiple plugins
        app.add_plugins(GameStatePlugin::default());
        app.add_plugins(SelectionPlugin);
        app.add_plugins(CameraPlugin);
        app.add_plugins(UiPlugin);

        // All resources should be present
        assert!(app.world().contains_resource::<GameStateResource>());
        assert!(app.world().contains_resource::<SelectedEntity>());
        assert!(app.world().contains_resource::<CameraState>());
        assert!(app.world().contains_resource::<UiState>());
    }

    #[test]
    fn test_full_plugin_with_update() {
        use bevy::input::keyboard::KeyCode;
        use bevy::input::mouse::MouseButton;
        use bevy::input::ButtonInput;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        // Add input resources required by systems (selection_system, turn_system)
        app.init_resource::<ButtonInput<MouseButton>>();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.add_plugins(NostrNationsPlugin::default());

        // Run one update cycle
        app.update();

        // App should still have all resources after update
        assert!(app.world().contains_resource::<GameStateResource>());
        assert!(app.world().contains_resource::<CurrentTurn>());
    }

    #[test]
    fn test_plugin_multiple_updates() {
        use bevy::input::keyboard::KeyCode;
        use bevy::input::mouse::MouseButton;
        use bevy::input::ButtonInput;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        // Add input resources required by systems (selection_system, turn_system)
        app.init_resource::<ButtonInput<MouseButton>>();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.add_plugins(NostrNationsPlugin::default());

        // Run multiple update cycles
        for _ in 0..10 {
            app.update();
        }

        // Resources should persist
        assert!(app.world().contains_resource::<GameStateResource>());
    }

    // ============================================
    // Resource Initialization Value Tests
    // ============================================

    #[test]
    fn test_current_turn_initialization() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(GameStatePlugin::default());

        let current_turn = app.world().resource::<CurrentTurn>();
        assert_eq!(current_turn.turn, 0);
        assert_eq!(current_turn.current_player, 0);
        assert!(!current_turn.turn_ended);
    }

    #[test]
    fn test_entity_maps_initialization() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(GameStatePlugin::default());

        let tile_map = app.world().resource::<TileEntityMap>();
        let unit_map = app.world().resource::<UnitEntityMap>();
        let city_map = app.world().resource::<CityEntityMap>();

        assert!(tile_map.is_empty());
        assert!(unit_map.units.is_empty());
        assert!(city_map.cities.is_empty());
    }

    #[test]
    fn test_pending_action_initialization() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(GameStatePlugin::default());

        let pending = app.world().resource::<PendingAction>();
        assert!(!pending.has_pending());
    }
}
