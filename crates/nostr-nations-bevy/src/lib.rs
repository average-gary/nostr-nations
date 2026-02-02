//! Nostr Nations Bevy Integration
//!
//! This crate provides Bevy ECS components, systems, resources, and plugins
//! for rendering the Nostr Nations game using the Bevy game engine.
//!
//! # Architecture
//!
//! The crate is organized into four main modules:
//!
//! - **[`components`]**: ECS components that wrap core game data types (units, cities, tiles)
//! - **[`resources`]**: Singleton resources for game-wide state (game engine, selection, settings)
//! - **[`systems`]**: Game loop systems (input, update, render, animation)
//! - **[`plugins`]**: Bevy plugins for modular initialization
//!
//! # Quick Start
//!
//! ```ignore
//! use bevy::prelude::*;
//! use nostr_nations_bevy::prelude::*;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugins(NostrNationsPlugin::default())
//!         .run();
//! }
//! ```
//!
//! # Custom Game Setup
//!
//! For more control over game initialization:
//!
//! ```ignore
//! use bevy::prelude::*;
//! use nostr_nations_bevy::prelude::*;
//! use nostr_nations_core::GameSettings;
//!
//! fn main() {
//!     let settings = GameSettings::new("My Game".to_string());
//!     let seed = [42u8; 32]; // Or use random seed
//!
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugins(NostrNationsPlugin::local(settings, seed))
//!         .add_systems(Startup, setup_game)
//!         .run();
//! }
//!
//! fn setup_game(mut commands: Commands) {
//!     // Add custom game setup here
//! }
//! ```
//!
//! # Component Overview
//!
//! The main components for game entities are:
//!
//! - [`TileComponent`](components::TileComponent) - Hex map tiles with terrain, resources, improvements
//! - [`UnitComponent`](components::UnitComponent) - Military and civilian units
//! - [`CityComponent`](components::CityComponent) - Cities with production, population, buildings
//! - [`PlayerComponent`](components::PlayerComponent) - Player data with gold, tech, diplomacy
//! - [`SelectionComponent`](components::SelectionComponent) - Marker for selected entities
//! - [`PositionComponent`](components::PositionComponent) - Hex coordinate position
//! - [`VisibleComponent`](components::VisibleComponent) - Fog of war visibility state
//!
//! # Resource Overview
//!
//! Key resources available to systems:
//!
//! - [`GameStateResource`](resources::GameStateResource) - The core game engine
//! - [`SelectedEntity`](resources::SelectedEntity) - Currently selected entity
//! - [`CurrentTurn`](resources::CurrentTurn) - Turn state and timing
//! - [`GameSettingsResource`](resources::GameSettingsResource) - Game configuration
//!
//! # System Sets
//!
//! Systems are organized into sets for ordering:
//!
//! 1. `GameSystemSet::Input` - Selection and input handling
//! 2. `GameSystemSet::Update` - Game logic and turn processing
//! 3. `GameSystemSet::Sync` - ECS/core state synchronization
//! 4. `GameSystemSet::Animation` - Visual animations

pub mod components;
pub mod plugins;
pub mod resources;
pub mod systems;

// Re-export core types for convenience
pub use nostr_nations_core;

/// Prelude module for convenient imports.
///
/// Import all commonly used types with:
/// ```ignore
/// use nostr_nations_bevy::prelude::*;
/// ```
pub mod prelude {
    // Components
    pub use crate::components::{
        CityBundle, CityComponent, CombatAnimation, LocalPlayerOwned, MovementAnimation,
        OtherPlayerOwned, PlayerComponent, PositionComponent, SelectionComponent, TileBundle,
        TileComponent, UnitBundle, UnitComponent, VisibleComponent,
    };

    // Resources
    pub use crate::resources::{
        CameraState, CityEntityMap, CurrentTurn, GameSettingsResource, GameStateResource,
        PendingAction, PendingActionType, SelectedEntity, SelectionType, TileEntityMap, UiState,
        UnitEntityMap,
    };

    // Systems
    pub use crate::systems::GameSystemSet;

    // Plugins
    pub use crate::plugins::{
        AnimationPlugin, CameraPlugin, GameStateEvent, GameStatePlugin, NostrNationsPlugin,
        SelectionEvent, SelectionPlugin, UiPlugin, VisibilityPlugin,
    };

    // Re-export commonly used core types
    pub use nostr_nations_core::{
        City, GameEngine, GameSettings, GameState, HexCoord, Map, Player, Tile, Unit,
    };
}

/// Create a default Bevy app configured for Nostr Nations.
///
/// This is a convenience function that creates a fully configured app
/// with default settings. For more control, use the plugin directly.
///
/// # Example
///
/// ```ignore
/// use nostr_nations_bevy::create_app;
///
/// fn main() {
///     create_app().run();
/// }
/// ```
pub fn create_app() -> bevy::app::App {
    use bevy::prelude::*;

    let mut app = App::new();

    // Add default Bevy plugins
    app.add_plugins(DefaultPlugins);

    // Add Nostr Nations plugin
    app.add_plugins(plugins::NostrNationsPlugin::default());

    app
}

/// Create a Bevy app with custom game settings.
///
/// # Arguments
///
/// * `settings` - Game configuration settings
/// * `seed` - Random seed for deterministic game generation
///
/// # Example
///
/// ```ignore
/// use nostr_nations_bevy::create_app_with_settings;
/// use nostr_nations_core::GameSettings;
///
/// fn main() {
///     let settings = GameSettings::duel("Quick Match".to_string());
///     let seed = [42u8; 32];
///     create_app_with_settings(settings, seed).run();
/// }
/// ```
pub fn create_app_with_settings(
    settings: nostr_nations_core::GameSettings,
    seed: [u8; 32],
) -> bevy::app::App {
    use bevy::prelude::*;

    let mut app = App::new();

    // Add default Bevy plugins
    app.add_plugins(DefaultPlugins);

    // Add Nostr Nations plugin with custom settings
    app.add_plugins(plugins::NostrNationsPlugin::local(settings, seed));

    app
}

/// Create a Bevy app for a networked multiplayer game.
///
/// # Arguments
///
/// * `settings` - Game configuration settings
/// * `seed` - Random seed (must match across all players)
/// * `local_player_id` - The ID of the local player
///
/// # Example
///
/// ```ignore
/// use nostr_nations_bevy::create_networked_app;
/// use nostr_nations_core::GameSettings;
///
/// fn main() {
///     let settings = GameSettings::new("Multiplayer Match".to_string());
///     let seed = [42u8; 32]; // Received from game host
///     let local_player_id = 1; // Assigned by matchmaking
///     create_networked_app(settings, seed, local_player_id).run();
/// }
/// ```
pub fn create_networked_app(
    settings: nostr_nations_core::GameSettings,
    seed: [u8; 32],
    local_player_id: u8,
) -> bevy::app::App {
    use bevy::prelude::*;

    let mut app = App::new();

    // Add default Bevy plugins
    app.add_plugins(DefaultPlugins);

    // Add Nostr Nations plugin for networked play
    app.add_plugins(plugins::NostrNationsPlugin::networked(
        settings,
        seed,
        local_player_id,
    ));

    app
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use nostr_nations_core::{unit::UnitType, City, HexCoord, Terrain, Tile, Unit};

    // ============================================
    // Prelude Import Tests
    // ============================================

    #[test]
    fn test_prelude_imports() {
        // Test that prelude exports compile correctly
        use crate::prelude::*;

        let _settings = GameSettings::default();
        let _coord = HexCoord::new(0, 0);
    }

    #[test]
    fn test_prelude_game_settings() {
        use crate::prelude::*;

        let settings = GameSettings::new("Test Game".to_string());
        assert_eq!(settings.name, "Test Game");
    }

    #[test]
    fn test_prelude_hex_coord() {
        use crate::prelude::*;

        let coord = HexCoord::new(5, 10);
        assert_eq!(coord.q, 5);
        assert_eq!(coord.r, 10);
    }

    #[test]
    fn test_prelude_components() {
        use crate::prelude::*;

        let pos = PositionComponent::at(3, 4);
        assert_eq!(pos.q(), 3);
        assert_eq!(pos.r(), 4);

        let visible = VisibleComponent::visible();
        assert!(visible.in_sight);
    }

    #[test]
    fn test_prelude_resources() {
        use crate::prelude::*;

        let selected = SelectedEntity::none();
        assert!(!selected.has_selection());

        let turn = CurrentTurn::new(1, 0);
        assert_eq!(turn.turn, 1);
    }

    #[test]
    fn test_prelude_system_set() {
        use crate::prelude::*;

        let _input = GameSystemSet::Input;
        let _update = GameSystemSet::Update;
        let _sync = GameSystemSet::Sync;
        let _animation = GameSystemSet::Animation;
    }

    // ============================================
    // Plugin Build Tests
    // ============================================

    #[test]
    fn test_plugin_builds() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(plugins::NostrNationsPlugin::default());

        // Verify resources are present
        assert!(app
            .world()
            .contains_resource::<resources::GameStateResource>());
        assert!(app.world().contains_resource::<resources::SelectedEntity>());
        assert!(app.world().contains_resource::<resources::CurrentTurn>());
    }

    #[test]
    fn test_plugin_builds_with_local_settings() {
        let settings = nostr_nations_core::GameSettings::new("Local Game".to_string());
        let plugin = plugins::NostrNationsPlugin::local(settings, [42u8; 32]);

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(plugin);

        assert!(app
            .world()
            .contains_resource::<resources::GameStateResource>());
        assert!(app
            .world()
            .contains_resource::<resources::GameSettingsResource>());
    }

    #[test]
    fn test_plugin_builds_with_networked_settings() {
        let settings = nostr_nations_core::GameSettings::new("Networked Game".to_string());
        let plugin = plugins::NostrNationsPlugin::networked(settings, [1u8; 32], 2);

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(plugin);

        let game_settings = app.world().resource::<resources::GameSettingsResource>();
        assert!(game_settings.is_networked);
        assert_eq!(game_settings.local_player_id, 2);
    }

    // ============================================
    // create_app_with_settings Tests
    // ============================================

    #[test]
    #[ignore] // Requires main thread / GUI event loop (DefaultPlugins)
    fn test_create_app_with_settings() {
        let settings = nostr_nations_core::GameSettings::duel("Test".to_string());
        let _app = create_app_with_settings(settings, [0u8; 32]);
        // App creation should not panic
    }

    #[test]
    #[ignore] // Requires main thread / GUI event loop (DefaultPlugins)
    fn test_create_app_with_different_seeds() {
        let settings1 = nostr_nations_core::GameSettings::new("Game 1".to_string());
        let settings2 = nostr_nations_core::GameSettings::new("Game 2".to_string());

        let _app1 = create_app_with_settings(settings1, [1u8; 32]);
        let _app2 = create_app_with_settings(settings2, [2u8; 32]);
        // Both should create without issues
    }

    #[test]
    #[ignore] // Requires main thread / GUI event loop (DefaultPlugins)
    fn test_create_app_with_settings_has_resources() {
        let settings = nostr_nations_core::GameSettings::new("Resource Test".to_string());
        let app = create_app_with_settings(settings, [5u8; 32]);

        // Note: create_app_with_settings adds DefaultPlugins which requires a windowed environment
        // In CI/headless tests, this might need adjustment
        // For now, we verify the function doesn't panic
        let _ = app;
    }

    // ============================================
    // create_networked_app Tests
    // ============================================

    #[test]
    #[ignore] // Requires main thread / GUI event loop (DefaultPlugins)
    fn test_create_networked_app_signature() {
        // Test that the function exists and has the right signature
        let settings = nostr_nations_core::GameSettings::new("Multiplayer".to_string());
        let seed = [42u8; 32];
        let local_player_id = 1;

        // This will create the app (may need windowed environment)
        let _app = create_networked_app(settings, seed, local_player_id);
    }

    // ============================================
    // Resource Verification Tests
    // ============================================

    #[test]
    fn test_app_contains_game_state_resource() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(plugins::NostrNationsPlugin::default());

        let game_state = app.world().resource::<resources::GameStateResource>();
        assert_eq!(game_state.turn(), 0);
        assert!(!game_state.is_ended());
    }

    #[test]
    fn test_app_contains_selected_entity_resource() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(plugins::NostrNationsPlugin::default());

        let selected = app.world().resource::<resources::SelectedEntity>();
        assert!(!selected.has_selection());
    }

    #[test]
    fn test_app_contains_current_turn_resource() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(plugins::NostrNationsPlugin::default());

        let turn = app.world().resource::<resources::CurrentTurn>();
        assert_eq!(turn.turn, 0);
        assert_eq!(turn.current_player, 0);
    }

    #[test]
    fn test_app_contains_entity_maps() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(plugins::NostrNationsPlugin::default());

        assert!(app.world().contains_resource::<resources::TileEntityMap>());
        assert!(app.world().contains_resource::<resources::UnitEntityMap>());
        assert!(app.world().contains_resource::<resources::CityEntityMap>());
    }

    #[test]
    fn test_app_contains_pending_action_resource() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(plugins::NostrNationsPlugin::default());

        let pending = app.world().resource::<resources::PendingAction>();
        assert!(!pending.has_pending());
    }

    // ============================================
    // Integration Tests - Entity Spawning
    // ============================================

    #[test]
    fn test_spawn_tile_in_app() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(plugins::NostrNationsPlugin::default());

        // Spawn a tile
        let tile = Tile::new(HexCoord::new(0, 0), Terrain::Grassland);
        let entity = app
            .world_mut()
            .spawn(components::TileBundle::new(tile))
            .id();

        // Verify tile exists
        assert!(app
            .world()
            .get::<components::TileComponent>(entity)
            .is_some());
        assert!(app
            .world()
            .get::<components::PositionComponent>(entity)
            .is_some());
    }

    #[test]
    fn test_spawn_unit_in_app() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(plugins::NostrNationsPlugin::default());

        // Spawn a unit
        let unit = Unit::new(1, 0, UnitType::Warrior, HexCoord::new(5, 5));
        let entity = app
            .world_mut()
            .spawn(components::UnitBundle::new(unit))
            .id();

        // Verify unit exists
        assert!(app
            .world()
            .get::<components::UnitComponent>(entity)
            .is_some());
        assert!(app
            .world()
            .get::<components::PositionComponent>(entity)
            .is_some());

        // Track in entity map
        app.world_mut()
            .resource_mut::<resources::UnitEntityMap>()
            .insert(1, entity);

        // Verify can retrieve
        let unit_map = app.world().resource::<resources::UnitEntityMap>();
        assert_eq!(unit_map.get(1), Some(entity));
    }

    #[test]
    fn test_spawn_city_in_app() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(plugins::NostrNationsPlugin::default());

        // Spawn a city
        let city = City::new(1, 0, "Test City".to_string(), HexCoord::new(10, 10), true);
        let entity = app
            .world_mut()
            .spawn(components::CityBundle::new(city))
            .id();

        // Verify city exists
        let city_comp = app
            .world()
            .get::<components::CityComponent>(entity)
            .unwrap();
        assert_eq!(city_comp.name(), "Test City");
        assert!(city_comp.is_capital());
    }

    // ============================================
    // Integration Tests - Resource Modification
    // ============================================

    #[test]
    fn test_modify_selection_in_app() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(plugins::NostrNationsPlugin::default());

        // Spawn a unit
        let unit = Unit::new(1, 0, UnitType::Warrior, HexCoord::new(0, 0));
        let entity = app
            .world_mut()
            .spawn(components::UnitBundle::new(unit))
            .id();

        // Select the unit
        {
            let mut selected = app.world_mut().resource_mut::<resources::SelectedEntity>();
            *selected = resources::SelectedEntity::unit(entity, 1, HexCoord::new(0, 0));
        }

        // Verify selection
        let selected = app.world().resource::<resources::SelectedEntity>();
        assert!(selected.has_selection());
        assert!(selected.is_unit());
        assert_eq!(selected.entity, Some(entity));
    }

    #[test]
    fn test_modify_current_turn_in_app() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(plugins::NostrNationsPlugin::default());

        // End turn
        {
            let mut turn = app.world_mut().resource_mut::<resources::CurrentTurn>();
            turn.end_turn();
        }

        // Verify turn ended
        let turn = app.world().resource::<resources::CurrentTurn>();
        assert!(turn.turn_ended);
    }

    #[test]
    fn test_modify_game_state_in_app() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(plugins::NostrNationsPlugin::default());

        // Modify game state
        {
            let mut game_state = app
                .world_mut()
                .resource_mut::<resources::GameStateResource>();
            game_state.state_mut().turn = 5;
        }

        // Verify change
        let game_state = app.world().resource::<resources::GameStateResource>();
        assert_eq!(game_state.turn(), 5);
    }

    // ============================================
    // Integration Tests - App Update Cycle
    // ============================================

    #[test]
    fn test_app_update_cycle() {
        use bevy::input::keyboard::KeyCode;
        use bevy::input::mouse::MouseButton;
        use bevy::input::ButtonInput;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        // Add input resources required by systems (selection_system, turn_system)
        app.init_resource::<ButtonInput<MouseButton>>();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.add_plugins(plugins::NostrNationsPlugin::default());

        // Run update
        app.update();

        // Resources should still exist
        assert!(app
            .world()
            .contains_resource::<resources::GameStateResource>());
        assert!(app.world().contains_resource::<resources::CurrentTurn>());
    }

    #[test]
    fn test_app_multiple_update_cycles() {
        use bevy::input::keyboard::KeyCode;
        use bevy::input::mouse::MouseButton;
        use bevy::input::ButtonInput;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        // Add input resources required by systems (selection_system, turn_system)
        app.init_resource::<ButtonInput<MouseButton>>();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.add_plugins(plugins::NostrNationsPlugin::default());

        // Run multiple updates
        for _ in 0..5 {
            app.update();
        }

        // App should still be functional
        assert!(app
            .world()
            .contains_resource::<resources::GameStateResource>());
    }

    #[test]
    fn test_app_update_with_entities() {
        use bevy::input::keyboard::KeyCode;
        use bevy::input::mouse::MouseButton;
        use bevy::input::ButtonInput;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        // Add input resources required by systems (selection_system, turn_system)
        app.init_resource::<ButtonInput<MouseButton>>();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.add_plugins(plugins::NostrNationsPlugin::default());

        // Spawn entities
        let unit = Unit::new(1, 0, UnitType::Warrior, HexCoord::new(0, 0));
        let entity = app
            .world_mut()
            .spawn(components::UnitBundle::new(unit))
            .id();

        // Run update
        app.update();

        // Entity should still exist
        assert!(app.world().get_entity(entity).is_some());
    }

    // ============================================
    // Module Re-export Tests
    // ============================================

    #[test]
    fn test_nostr_nations_core_reexport() {
        // Verify core types are re-exported
        let _settings = nostr_nations_core::GameSettings::default();
        let _coord = nostr_nations_core::HexCoord::new(0, 0);
    }

    #[test]
    fn test_components_module_accessible() {
        let tile = Tile::new(HexCoord::new(0, 0), Terrain::Grassland);
        let _component = components::TileComponent::new(tile);
    }

    #[test]
    fn test_resources_module_accessible() {
        let _selected = resources::SelectedEntity::none();
        let _turn = resources::CurrentTurn::default();
    }

    #[test]
    fn test_systems_module_accessible() {
        let _set = systems::GameSystemSet::Input;
    }

    #[test]
    fn test_plugins_module_accessible() {
        let _plugin = plugins::NostrNationsPlugin::default();
    }

    // ============================================
    // Component-Resource Interaction Tests
    // ============================================

    #[test]
    fn test_entity_map_integration() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(plugins::NostrNationsPlugin::default());

        // Spawn multiple units
        let mut entities = Vec::new();
        for i in 0..3 {
            let unit = Unit::new(i, 0, UnitType::Warrior, HexCoord::new(i as i32, 0));
            let entity = app
                .world_mut()
                .spawn(components::UnitBundle::new(unit))
                .id();
            entities.push((i, entity));
        }

        // Track in entity map
        {
            let mut unit_map = app.world_mut().resource_mut::<resources::UnitEntityMap>();
            for (id, entity) in &entities {
                unit_map.insert(*id, *entity);
            }
        }

        // Verify all can be retrieved
        let unit_map = app.world().resource::<resources::UnitEntityMap>();
        for (id, entity) in &entities {
            assert_eq!(unit_map.get(*id), Some(*entity));
        }
    }

    #[test]
    fn test_selection_with_query() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(plugins::NostrNationsPlugin::default());

        // Spawn and select unit
        let unit = Unit::new(1, 0, UnitType::Warrior, HexCoord::new(5, 5));
        let entity = app
            .world_mut()
            .spawn((
                components::UnitBundle::new(unit),
                components::SelectionComponent::primary(),
            ))
            .id();

        // Query selected entities
        let mut query = app
            .world_mut()
            .query::<(Entity, &components::SelectionComponent)>();
        let selected: Vec<_> = query.iter(app.world()).collect();

        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].0, entity);
    }

    #[test]
    fn test_visibility_with_ownership() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(plugins::NostrNationsPlugin::default());

        // Spawn local player's unit
        let unit1 = Unit::new(1, 0, UnitType::Warrior, HexCoord::new(0, 0));
        let entity1 = app
            .world_mut()
            .spawn((
                components::UnitBundle::new(unit1),
                components::LocalPlayerOwned,
            ))
            .id();

        // Spawn enemy unit
        let unit2 = Unit::new(2, 1, UnitType::Warrior, HexCoord::new(1, 1));
        let entity2 = app
            .world_mut()
            .spawn((
                components::UnitBundle::new(unit2),
                components::OtherPlayerOwned::new(1),
            ))
            .id();

        // Query local player's units
        let mut local_query = app
            .world_mut()
            .query_filtered::<Entity, With<components::LocalPlayerOwned>>();
        let local_units: Vec<_> = local_query.iter(app.world()).collect();

        // Query enemy units
        let mut enemy_query = app
            .world_mut()
            .query_filtered::<Entity, With<components::OtherPlayerOwned>>();
        let enemy_units: Vec<_> = enemy_query.iter(app.world()).collect();

        assert_eq!(local_units.len(), 1);
        assert_eq!(local_units[0], entity1);
        assert_eq!(enemy_units.len(), 1);
        assert_eq!(enemy_units[0], entity2);
    }
}
