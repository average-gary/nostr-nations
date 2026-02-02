//! Bevy ECS systems for Nostr Nations.
//!
//! Systems are the core logic that processes game state each frame.
//! They query for entities with specific components and update them.

use bevy::prelude::*;
use nostr_nations_core::{events::GameAction, replay::ActionEffect, HexCoord};

use crate::components::{
    CityComponent, LocalPlayerOwned, MovementAnimation, PositionComponent, SelectionComponent,
    TileComponent, UnitComponent, VisibleComponent,
};
use crate::resources::{
    CityEntityMap, CurrentTurn, GameSettingsResource, GameStateResource, PendingAction,
    PendingActionType, SelectedEntity, TileEntityMap, UnitEntityMap,
};

/// System that processes game tick updates.
///
/// This system updates the game state based on elapsed time and
/// processes any queued actions.
pub fn game_tick_system(
    time: Res<Time>,
    mut game_state: ResMut<GameStateResource>,
    mut current_turn: ResMut<CurrentTurn>,
    settings: Res<GameSettingsResource>,
) {
    // Update turn timer if enabled
    if current_turn.time_remaining.is_some() {
        let delta = time.delta_seconds();
        if current_turn.update_timer(delta) {
            // Timer expired - auto end turn
            if settings.local_player_id == current_turn.current_player {
                // End turn for local player
                let _ = game_state
                    .engine
                    .apply_action(settings.local_player_id, &GameAction::EndTurn);
            }
        }
    }

    // Sync current turn state from game engine
    let state = game_state.state();
    if current_turn.turn != state.turn || current_turn.current_player != state.current_player {
        current_turn.next_turn(state.turn, state.current_player);
    }
}

/// System that handles entity selection.
///
/// This system manages the selection state when entities are clicked
/// and updates the SelectionComponent markers accordingly.
#[allow(clippy::too_many_arguments)]
pub fn selection_system(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    _selected: ResMut<SelectedEntity>,
    current_turn: Res<CurrentTurn>,
    settings: Res<GameSettingsResource>,
    _units_query: Query<(Entity, &UnitComponent, &PositionComponent), Without<SelectionComponent>>,
    _cities_query: Query<(Entity, &CityComponent, &PositionComponent), Without<SelectionComponent>>,
    _tiles_query: Query<(Entity, &TileComponent, &PositionComponent), Without<SelectionComponent>>,
    selected_query: Query<Entity, With<SelectionComponent>>,
    _tile_map: Res<TileEntityMap>,
    // Note: In a real implementation, you'd need to convert mouse position to hex coordinates
    // This is simplified for the example
) {
    // Only process selection on left click
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    // Check if it's the local player's turn
    let can_select = current_turn.is_player_turn(settings.local_player_id);
    if !can_select {
        return;
    }

    // Clear previous selection
    for entity in selected_query.iter() {
        commands.entity(entity).remove::<SelectionComponent>();
    }

    // In a real implementation, you would:
    // 1. Convert mouse screen position to world position
    // 2. Convert world position to hex coordinate
    // 3. Find entities at that coordinate
    // 4. Prioritize units > cities > tiles
    // 5. Select the appropriate entity

    // For now, this is a placeholder that would be connected to actual input handling
}

/// System that processes selection changes and updates UI.
///
/// This runs when the SelectedEntity resource changes.
pub fn selection_changed_system(
    selected: Res<SelectedEntity>,
    units_query: Query<&UnitComponent>,
    cities_query: Query<&CityComponent>,
) {
    if !selected.is_changed() {
        return;
    }

    if let Some(entity) = selected.entity {
        match selected.selection_type {
            crate::resources::SelectionType::Unit => {
                if let Ok(unit) = units_query.get(entity) {
                    // Log or update UI with unit info
                    info!("Selected unit {} at {:?}", unit.id(), unit.position());
                }
            }
            crate::resources::SelectionType::City => {
                if let Ok(city) = cities_query.get(entity) {
                    // Log or update UI with city info
                    info!("Selected city {} at {:?}", city.name(), city.position());
                }
            }
            crate::resources::SelectionType::Tile => {
                if let Some(coord) = selected.coord {
                    info!("Selected tile at {:?}", coord);
                }
            }
            crate::resources::SelectionType::None => {}
        }
    }
}

/// System that updates visibility (fog of war).
///
/// This system calculates which tiles are visible to the current player
/// based on unit positions and updates VisibleComponent accordingly.
pub fn visibility_system(
    game_state: Res<GameStateResource>,
    settings: Res<GameSettingsResource>,
    mut tiles_query: Query<(&PositionComponent, &mut VisibleComponent)>,
    units_query: Query<(&UnitComponent, &PositionComponent), With<LocalPlayerOwned>>,
    cities_query: Query<(&CityComponent, &PositionComponent), With<LocalPlayerOwned>>,
) {
    // Skip if fog of war is disabled
    if !settings.has_fog_of_war() {
        // Make all tiles visible
        for (_, mut visible) in tiles_query.iter_mut() {
            visible.in_sight = true;
            visible.explored = true;
        }
        return;
    }

    let local_player = settings.local_player_id;

    // First, reset all tiles to not in sight (but keep explored state)
    for (_, mut visible) in tiles_query.iter_mut() {
        visible.in_sight = false;
    }

    // Calculate sight range from units
    let sight_range = 2u32; // Base sight range
    let mut visible_coords: std::collections::HashSet<HexCoord> = std::collections::HashSet::new();

    // Add tiles visible from units
    for (unit, pos) in units_query.iter() {
        if unit.owner() == local_player {
            for coord in pos.coord.hexes_in_radius(sight_range) {
                visible_coords.insert(coord);
            }
        }
    }

    // Add tiles visible from cities
    for (city, pos) in cities_query.iter() {
        if city.owner() == local_player {
            // Cities have extended sight range
            for coord in pos.coord.hexes_in_radius(sight_range + 1) {
                visible_coords.insert(coord);
            }
        }
    }

    // Update tile visibility
    for (pos, mut visible) in tiles_query.iter_mut() {
        if visible_coords.contains(&pos.coord) {
            visible.in_sight = true;
            visible.explored = true;
        }
    }

    // Also check player's explored tiles from game state
    if let Some(player) = game_state.state().get_player(local_player) {
        for (pos, mut visible) in tiles_query.iter_mut() {
            if player.has_explored(&pos.coord) {
                visible.explored = true;
            }
        }
    }
}

/// System that manages turn transitions.
///
/// This system handles end turn actions and transitions between players.
pub fn turn_system(
    mut game_state: ResMut<GameStateResource>,
    mut current_turn: ResMut<CurrentTurn>,
    settings: Res<GameSettingsResource>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut units_query: Query<&mut UnitComponent, With<LocalPlayerOwned>>,
) {
    // Check for end turn key press (Enter or E)
    let end_turn_pressed =
        keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::KeyE);

    if !end_turn_pressed {
        return;
    }

    // Only allow ending turn on local player's turn
    if !current_turn.is_player_turn(settings.local_player_id) {
        return;
    }

    // Apply end turn action
    let result = game_state
        .engine
        .apply_action(settings.local_player_id, &GameAction::EndTurn);

    match result {
        Ok(action_result) => {
            // Process action effects
            for effect in action_result.effects {
                match effect {
                    ActionEffect::TurnStarted { player_id, turn } => {
                        info!("Turn {} started for player {}", turn, player_id);
                        current_turn.next_turn(turn, player_id);

                        // Reset unit movement for the new player
                        if player_id == settings.local_player_id {
                            for mut unit in units_query.iter_mut() {
                                unit.unit.new_turn();
                            }
                        }
                    }
                    ActionEffect::GameEnded {
                        winner_id,
                        victory_type,
                    } => {
                        info!("Game ended! Winner: {} by {}", winner_id, victory_type);
                    }
                    _ => {}
                }
            }
        }
        Err(e) => {
            error!("Failed to end turn: {:?}", e);
        }
    }
}

/// System that processes pending actions.
///
/// This system executes actions that have been queued up and confirmed.
pub fn pending_action_system(
    mut game_state: ResMut<GameStateResource>,
    mut pending: ResMut<PendingAction>,
    settings: Res<GameSettingsResource>,
    current_turn: Res<CurrentTurn>,
    mut unit_map: ResMut<UnitEntityMap>,
    mut commands: Commands,
) {
    // Only process actions on local player's turn
    if !current_turn.is_player_turn(settings.local_player_id) {
        pending.clear();
        return;
    }

    let action = match pending.action.take() {
        Some(a) => a,
        None => return,
    };

    let game_action = match action {
        PendingActionType::MoveUnit { unit_id, path } => GameAction::MoveUnit { unit_id, path },
        PendingActionType::AttackUnit {
            attacker_id,
            defender_id,
        } => {
            // In a real implementation, you'd get the randomness from Cashu
            // For now, use a deterministic value
            GameAction::AttackUnit {
                attacker_id,
                defender_id,
                random: 0.5,
            }
        }
        PendingActionType::FoundCity { settler_id } => {
            // In a real implementation, you'd prompt for city name
            GameAction::FoundCity {
                settler_id,
                name: "New City".to_string(),
            }
        }
        PendingActionType::AttackCity {
            attacker_id,
            city_id,
        } => GameAction::AttackCity {
            attacker_id,
            city_id,
            random: 0.5,
        },
    };

    let result = game_state
        .engine
        .apply_action(settings.local_player_id, &game_action);

    match result {
        Ok(action_result) => {
            // Process action effects
            for effect in action_result.effects {
                process_action_effect(&effect, &mut commands, &mut unit_map);
            }
        }
        Err(e) => {
            error!("Action failed: {:?}", e);
        }
    }

    pending.clear();
}

/// Helper function to process action effects and update ECS state.
fn process_action_effect(
    effect: &ActionEffect,
    commands: &mut Commands,
    unit_map: &mut UnitEntityMap,
) {
    match effect {
        ActionEffect::UnitMoved { unit_id, from, to } => {
            info!("Unit {} moved from {:?} to {:?}", unit_id, from, to);
            // The actual position update happens through component sync
        }
        ActionEffect::UnitDamaged {
            unit_id,
            damage,
            new_health,
        } => {
            info!(
                "Unit {} took {} damage, health: {}",
                unit_id, damage, new_health
            );
        }
        ActionEffect::UnitDestroyed { unit_id } => {
            info!("Unit {} destroyed", unit_id);
            // Remove the entity
            if let Some(entity) = unit_map.remove(*unit_id) {
                commands.entity(entity).despawn();
            }
        }
        ActionEffect::UnitCreated {
            unit_id,
            unit_type,
            position,
        } => {
            info!(
                "Unit {} ({:?}) created at {:?}",
                unit_id, unit_type, position
            );
            // Spawning handled elsewhere
        }
        ActionEffect::CityFounded {
            city_id,
            name,
            position,
        } => {
            info!("City {} '{}' founded at {:?}", city_id, name, position);
        }
        ActionEffect::CityDamaged { city_id, damage } => {
            info!("City {} took {} damage", city_id, damage);
        }
        ActionEffect::CityGrew {
            city_id,
            new_population,
        } => {
            info!("City {} grew to population {}", city_id, new_population);
        }
        ActionEffect::TechResearched { player_id, tech_id } => {
            info!("Player {} researched {}", player_id, tech_id);
        }
        ActionEffect::TurnStarted { player_id, turn } => {
            info!("Turn {} started for player {}", turn, player_id);
        }
        ActionEffect::GameEnded {
            winner_id,
            victory_type,
        } => {
            info!("Game ended! Winner: {} by {}", winner_id, victory_type);
        }
    }
}

/// System that updates movement animations.
///
/// This system smoothly animates units moving along paths.
pub fn movement_animation_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut MovementAnimation,
        &mut PositionComponent,
        &mut Transform,
    )>,
) {
    let delta = time.delta_seconds();

    for (entity, mut anim, mut position, mut transform) in query.iter_mut() {
        if anim.is_complete() {
            // Remove animation component when done
            commands.entity(entity).remove::<MovementAnimation>();
            continue;
        }

        // Update progress
        anim.progress += delta * anim.speed * 2.0; // 2.0 = moves per second

        if anim.progress >= 1.0 {
            // Move to next segment
            anim.progress = 0.0;
            anim.current_index += 1;

            // Update logical position
            if let Some(coord) = anim.current_position() {
                position.coord = coord;
            }
        }

        // Interpolate visual position
        if let (Some(from), Some(to)) = (anim.current_position(), anim.next_position()) {
            let from_world = hex_to_world(from);
            let to_world = hex_to_world(to);
            let interpolated = from_world.lerp(to_world, anim.progress);
            transform.translation.x = interpolated.x;
            transform.translation.y = interpolated.y;
        }
    }
}

/// System that synchronizes ECS components with core game state.
///
/// This ensures the ECS world stays in sync with the authoritative
/// game state in GameStateResource.
pub fn sync_game_state_system(
    game_state: Res<GameStateResource>,
    mut units_query: Query<(&mut UnitComponent, &mut PositionComponent), Without<CityComponent>>,
    mut cities_query: Query<(&mut CityComponent, &mut PositionComponent), Without<UnitComponent>>,
    unit_map: Res<UnitEntityMap>,
    city_map: Res<CityEntityMap>,
) {
    // Sync unit data
    for (unit_id, unit_data) in game_state.state().units.iter() {
        if let Some(entity) = unit_map.get(*unit_id) {
            if let Ok((mut unit_comp, mut pos_comp)) = units_query.get_mut(entity) {
                // Update unit data
                unit_comp.unit = unit_data.clone();
                pos_comp.coord = unit_data.position;
            }
        }
    }

    // Sync city data
    for (city_id, city_data) in game_state.state().cities.iter() {
        if let Some(entity) = city_map.get(*city_id) {
            if let Ok((mut city_comp, mut pos_comp)) = cities_query.get_mut(entity) {
                // Update city data
                city_comp.city = city_data.clone();
                pos_comp.coord = city_data.position;
            }
        }
    }
}

/// System that spawns entities for new game objects.
///
/// This detects when new units/cities are added to the game state
/// and creates corresponding ECS entities.
pub fn spawn_new_entities_system(
    mut commands: Commands,
    game_state: Res<GameStateResource>,
    settings: Res<GameSettingsResource>,
    mut unit_map: ResMut<UnitEntityMap>,
    mut city_map: ResMut<CityEntityMap>,
) {
    let state = game_state.state();
    let local_player = settings.local_player_id;

    // Check for new units
    for (unit_id, unit_data) in state.units.iter() {
        if unit_map.get(*unit_id).is_none() {
            // Spawn new unit entity
            let mut entity_commands = commands.spawn((
                crate::components::UnitBundle::new(unit_data.clone()),
                Name::new(format!("Unit_{}", unit_id)),
            ));

            // Add ownership marker
            if unit_data.owner == local_player {
                entity_commands.insert(LocalPlayerOwned);
            } else {
                entity_commands.insert(crate::components::OtherPlayerOwned::new(unit_data.owner));
            }

            let entity = entity_commands.id();
            unit_map.insert(*unit_id, entity);

            info!("Spawned unit entity for unit {}", unit_id);
        }
    }

    // Check for new cities
    for (city_id, city_data) in state.cities.iter() {
        if city_map.get(*city_id).is_none() {
            // Spawn new city entity
            let mut entity_commands = commands.spawn((
                crate::components::CityBundle::new(city_data.clone()),
                Name::new(format!("City_{}", city_data.name)),
            ));

            // Add ownership marker
            if city_data.owner == local_player {
                entity_commands.insert(LocalPlayerOwned);
            } else {
                entity_commands.insert(crate::components::OtherPlayerOwned::new(city_data.owner));
            }

            let entity = entity_commands.id();
            city_map.insert(*city_id, entity);

            info!(
                "Spawned city entity for city {} ({})",
                city_id, city_data.name
            );
        }
    }
}

/// System that removes entities for destroyed game objects.
///
/// This detects when units/cities are removed from the game state
/// and despawns the corresponding ECS entities.
pub fn despawn_removed_entities_system(
    mut commands: Commands,
    game_state: Res<GameStateResource>,
    mut unit_map: ResMut<UnitEntityMap>,
    mut city_map: ResMut<CityEntityMap>,
) {
    let state = game_state.state();

    // Check for removed units
    let current_unit_ids: std::collections::HashSet<_> = state.units.keys().copied().collect();
    let tracked_unit_ids: Vec<_> = unit_map.units.keys().copied().collect();

    for unit_id in tracked_unit_ids {
        if !current_unit_ids.contains(&unit_id) {
            if let Some(entity) = unit_map.remove(unit_id) {
                commands.entity(entity).despawn();
                info!("Despawned unit entity for unit {}", unit_id);
            }
        }
    }

    // Check for removed cities
    let current_city_ids: std::collections::HashSet<_> = state.cities.keys().copied().collect();
    let tracked_city_ids: Vec<_> = city_map.cities.keys().copied().collect();

    for city_id in tracked_city_ids {
        if !current_city_ids.contains(&city_id) {
            if let Some(entity) = city_map.remove(city_id) {
                commands.entity(entity).despawn();
                info!("Despawned city entity for city {}", city_id);
            }
        }
    }
}

/// Convert a hex coordinate to world position.
///
/// Uses pointy-top hex layout with odd-q offset coordinates.
fn hex_to_world(coord: HexCoord) -> Vec2 {
    // Hex dimensions (these would typically come from a config)
    let hex_width = 64.0f32;
    let hex_height = 74.0f32; // height = width * sqrt(3) / 2 * 2 for pointy-top

    let x = coord.q as f32 * hex_width * 0.75;
    let y = coord.r as f32 * hex_height
        + if coord.q % 2 != 0 {
            hex_height * 0.5
        } else {
            0.0
        };

    Vec2::new(x, -y) // Negative y because screen coordinates go down
}

/// Convert a world position to the nearest hex coordinate.
#[allow(dead_code)]
fn world_to_hex(world_pos: Vec2) -> HexCoord {
    let hex_width = 64.0f32;
    let hex_height = 74.0f32;

    // Approximate q coordinate
    let q = (world_pos.x / (hex_width * 0.75)).round() as i32;

    // Adjust y for odd columns
    let adjusted_y = -world_pos.y - if q % 2 != 0 { hex_height * 0.5 } else { 0.0 };
    let r = (adjusted_y / hex_height).round() as i32;

    HexCoord::new(q, r)
}

/// System set labels for organizing system execution order.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameSystemSet {
    /// Input handling systems.
    Input,
    /// Game logic update systems.
    Update,
    /// Post-update synchronization.
    Sync,
    /// Animation systems.
    Animation,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{CityBundle, TileBundle, TileComponent, UnitBundle};
    use nostr_nations_core::{unit::UnitType, City, Terrain, Tile, Unit};

    // ============================================
    // Hex Coordinate Conversion Tests
    // ============================================

    #[test]
    fn test_hex_to_world_origin() {
        let origin = hex_to_world(HexCoord::new(0, 0));
        assert_eq!(origin, Vec2::ZERO);
    }

    #[test]
    fn test_hex_to_world_positive_q() {
        let right = hex_to_world(HexCoord::new(1, 0));
        assert!(right.x > 0.0);
    }

    #[test]
    fn test_hex_to_world_negative_q() {
        let left = hex_to_world(HexCoord::new(-1, 0));
        assert!(left.x < 0.0);
    }

    #[test]
    fn test_hex_to_world_positive_r() {
        let down = hex_to_world(HexCoord::new(0, 1));
        // Y is negated in hex_to_world, so positive r means negative y
        assert!(down.y < 0.0);
    }

    #[test]
    fn test_hex_to_world_odd_q_offset() {
        // Odd q columns should have an offset in y
        let even = hex_to_world(HexCoord::new(0, 0));
        let odd = hex_to_world(HexCoord::new(1, 0));

        // The y offset should be different due to odd-q layout
        // Even column at r=0 vs odd column at r=0
        assert_ne!(even.y, odd.y);
    }

    #[test]
    fn test_world_to_hex_roundtrip() {
        let original = HexCoord::new(5, 5);
        let world = hex_to_world(original);
        let recovered = world_to_hex(world);
        assert_eq!(original, recovered);
    }

    #[test]
    fn test_world_to_hex_roundtrip_negative() {
        let original = HexCoord::new(-3, -4);
        let world = hex_to_world(original);
        let recovered = world_to_hex(world);
        assert_eq!(original, recovered);
    }

    #[test]
    fn test_world_to_hex_roundtrip_mixed() {
        let original = HexCoord::new(-2, 5);
        let world = hex_to_world(original);
        let recovered = world_to_hex(world);
        assert_eq!(original, recovered);
    }

    #[test]
    fn test_hex_to_world_distance_consistency() {
        // Adjacent hexes should have similar distances in world space
        let center = hex_to_world(HexCoord::new(5, 5));
        let neighbor = hex_to_world(HexCoord::new(6, 5));

        let distance = center.distance(neighbor);
        assert!(distance > 0.0);
        assert!(distance < 100.0); // Reasonable distance for our hex size
    }

    // ============================================
    // GameSystemSet Tests
    // ============================================

    #[test]
    fn test_game_system_set_variants() {
        // Test that all variants can be created
        let input = GameSystemSet::Input;
        let update = GameSystemSet::Update;
        let sync = GameSystemSet::Sync;
        let animation = GameSystemSet::Animation;

        // Test equality
        assert_eq!(input, GameSystemSet::Input);
        assert_ne!(input, update);
        assert_ne!(update, sync);
        assert_ne!(sync, animation);
    }

    #[test]
    fn test_game_system_set_clone() {
        let original = GameSystemSet::Update;
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_game_system_set_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(GameSystemSet::Input);
        set.insert(GameSystemSet::Update);
        set.insert(GameSystemSet::Sync);
        set.insert(GameSystemSet::Animation);

        assert_eq!(set.len(), 4);
        assert!(set.contains(&GameSystemSet::Input));
    }

    #[test]
    fn test_game_system_set_debug() {
        let set = GameSystemSet::Input;
        let debug_str = format!("{:?}", set);
        assert!(debug_str.contains("Input"));
    }

    // ============================================
    // World and System Integration Tests
    // ============================================

    #[test]
    fn test_create_minimal_test_world() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Add required resources
        app.insert_resource(GameStateResource::default());
        app.insert_resource(GameSettingsResource::default());
        app.insert_resource(CurrentTurn::default());
        app.insert_resource(SelectedEntity::default());
        app.insert_resource(TileEntityMap::default());
        app.insert_resource(UnitEntityMap::default());
        app.insert_resource(CityEntityMap::default());
        app.insert_resource(PendingAction::default());

        // Verify resources are present
        assert!(app.world().contains_resource::<GameStateResource>());
        assert!(app.world().contains_resource::<CurrentTurn>());
    }

    #[test]
    fn test_spawn_entities_in_world() {
        let mut world = World::new();

        // Spawn a tile
        let tile = Tile::new(HexCoord::new(0, 0), Terrain::Grassland);
        let tile_entity = world.spawn(TileBundle::new(tile)).id();

        // Spawn a unit
        let unit = Unit::new(1, 0, UnitType::Warrior, HexCoord::new(0, 0));
        let unit_entity = world.spawn(UnitBundle::new(unit)).id();

        // Verify entities exist
        assert!(world.get_entity(tile_entity).is_some());
        assert!(world.get_entity(unit_entity).is_some());

        // Verify components
        assert!(world.get::<TileComponent>(tile_entity).is_some());
        assert!(world
            .get::<crate::components::UnitComponent>(unit_entity)
            .is_some());
    }

    #[test]
    fn test_query_units_by_position() {
        let mut world = World::new();

        // Spawn multiple units at different positions
        for i in 0..5 {
            let unit = Unit::new(i, 0, UnitType::Warrior, HexCoord::new(i as i32, 0));
            world.spawn(UnitBundle::new(unit));
        }

        // Query units
        let mut query = world.query::<(&crate::components::UnitComponent, &PositionComponent)>();
        let units: Vec<_> = query.iter(&world).collect();

        assert_eq!(units.len(), 5);
    }

    #[test]
    fn test_query_with_local_player_marker() {
        let mut world = World::new();

        // Spawn player-owned unit
        let unit1 = Unit::new(1, 0, UnitType::Warrior, HexCoord::new(0, 0));
        world.spawn((UnitBundle::new(unit1), LocalPlayerOwned));

        // Spawn enemy unit
        let unit2 = Unit::new(2, 1, UnitType::Warrior, HexCoord::new(1, 1));
        world.spawn(UnitBundle::new(unit2));

        // Query only local player's units
        let mut query =
            world.query_filtered::<&crate::components::UnitComponent, With<LocalPlayerOwned>>();
        let local_units: Vec<_> = query.iter(&world).collect();

        assert_eq!(local_units.len(), 1);
    }

    #[test]
    fn test_despawn_entity() {
        let mut world = World::new();

        let unit = Unit::new(1, 0, UnitType::Warrior, HexCoord::new(0, 0));
        let entity = world.spawn(UnitBundle::new(unit)).id();

        // Verify exists
        assert!(world.get_entity(entity).is_some());

        // Despawn
        world.despawn(entity);

        // Verify removed
        assert!(world.get_entity(entity).is_none());
    }

    // ============================================
    // Resource State Tests
    // ============================================

    #[test]
    fn test_modify_game_state_resource() {
        let mut world = World::new();
        world.insert_resource(GameStateResource::default());

        // Modify turn
        {
            let mut game_state = world.resource_mut::<GameStateResource>();
            game_state.state_mut().turn = 10;
        }

        // Verify change
        let game_state = world.resource::<GameStateResource>();
        assert_eq!(game_state.turn(), 10);
    }

    #[test]
    fn test_modify_current_turn_resource() {
        let mut world = World::new();
        world.insert_resource(CurrentTurn::new(1, 0));

        // End turn
        {
            let mut current_turn = world.resource_mut::<CurrentTurn>();
            current_turn.end_turn();
        }

        // Verify change
        let current_turn = world.resource::<CurrentTurn>();
        assert!(current_turn.turn_ended);
    }

    #[test]
    fn test_entity_map_synchronization() {
        let mut world = World::new();
        world.insert_resource(UnitEntityMap::default());

        // Spawn unit and track in map
        let unit = Unit::new(42, 0, UnitType::Warrior, HexCoord::new(0, 0));
        let entity = world.spawn(UnitBundle::new(unit)).id();

        {
            let mut unit_map = world.resource_mut::<UnitEntityMap>();
            unit_map.insert(42, entity);
        }

        // Verify can retrieve
        let unit_map = world.resource::<UnitEntityMap>();
        assert_eq!(unit_map.get(42), Some(entity));
    }

    #[test]
    fn test_selection_resource_state() {
        let mut world = World::new();
        world.insert_resource(SelectedEntity::none());

        // Select unit
        let unit = Unit::new(1, 0, UnitType::Warrior, HexCoord::new(0, 0));
        let entity = world.spawn(UnitBundle::new(unit)).id();

        {
            let mut selection = world.resource_mut::<SelectedEntity>();
            *selection = SelectedEntity::unit(entity, 1, HexCoord::new(0, 0));
        }

        // Verify selection
        let selection = world.resource::<SelectedEntity>();
        assert!(selection.has_selection());
        assert!(selection.is_unit());
        assert_eq!(selection.entity, Some(entity));
    }

    // ============================================
    // Component Modification Tests
    // ============================================

    #[test]
    fn test_modify_position_component() {
        let mut world = World::new();

        let unit = Unit::new(1, 0, UnitType::Warrior, HexCoord::new(0, 0));
        let entity = world.spawn(UnitBundle::new(unit)).id();

        // Modify position
        if let Some(mut pos) = world.get_mut::<PositionComponent>(entity) {
            pos.coord = HexCoord::new(5, 5);
        }

        // Verify
        let pos = world.get::<PositionComponent>(entity).unwrap();
        assert_eq!(pos.coord, HexCoord::new(5, 5));
    }

    #[test]
    fn test_modify_visible_component() {
        let mut world = World::new();

        let tile = Tile::new(HexCoord::new(0, 0), Terrain::Grassland);
        let entity = world.spawn(TileBundle::new(tile)).id();

        // Initially hidden
        {
            let visible = world
                .get::<crate::components::VisibleComponent>(entity)
                .unwrap();
            assert!(!visible.in_sight);
            assert!(!visible.explored);
        }

        // Make visible
        if let Some(mut visible) = world.get_mut::<crate::components::VisibleComponent>(entity) {
            visible.in_sight = true;
            visible.explored = true;
        }

        // Verify
        let visible = world
            .get::<crate::components::VisibleComponent>(entity)
            .unwrap();
        assert!(visible.in_sight);
        assert!(visible.explored);
    }

    #[test]
    fn test_add_component_to_entity() {
        let mut world = World::new();

        let unit = Unit::new(1, 0, UnitType::Warrior, HexCoord::new(0, 0));
        let entity = world.spawn(UnitBundle::new(unit)).id();

        // Initially no selection component
        assert!(world.get::<SelectionComponent>(entity).is_none());

        // Add selection component
        world
            .entity_mut(entity)
            .insert(SelectionComponent::primary());

        // Verify added
        let selection = world.get::<SelectionComponent>(entity).unwrap();
        assert!(selection.is_primary);
    }

    #[test]
    fn test_remove_component_from_entity() {
        let mut world = World::new();

        let unit = Unit::new(1, 0, UnitType::Warrior, HexCoord::new(0, 0));
        let entity = world
            .spawn((UnitBundle::new(unit), SelectionComponent::primary()))
            .id();

        // Verify component exists
        assert!(world.get::<SelectionComponent>(entity).is_some());

        // Remove component
        world.entity_mut(entity).remove::<SelectionComponent>();

        // Verify removed
        assert!(world.get::<SelectionComponent>(entity).is_none());
    }

    // ============================================
    // Movement Animation State Tests
    // ============================================

    #[test]
    fn test_movement_animation_component_added() {
        let mut world = World::new();

        let unit = Unit::new(1, 0, UnitType::Warrior, HexCoord::new(0, 0));
        let entity = world.spawn(UnitBundle::new(unit)).id();

        // Add movement animation
        let path = vec![
            HexCoord::new(0, 0),
            HexCoord::new(1, 0),
            HexCoord::new(2, 0),
        ];
        world
            .entity_mut(entity)
            .insert(crate::components::MovementAnimation::new(path));

        // Verify
        let anim = world
            .get::<crate::components::MovementAnimation>(entity)
            .unwrap();
        assert!(!anim.is_complete());
        assert_eq!(anim.current_position(), Some(HexCoord::new(0, 0)));
    }

    #[test]
    fn test_movement_animation_progress() {
        let path = vec![HexCoord::new(0, 0), HexCoord::new(1, 0)];
        let mut anim = crate::components::MovementAnimation::new(path);

        // Initially at start
        assert_eq!(anim.current_index, 0);
        assert_eq!(anim.progress, 0.0);

        // Simulate progress
        anim.progress = 0.5;
        assert!(!anim.is_complete());

        // Complete first segment
        anim.progress = 1.0;
        anim.current_index = 1;
        assert!(anim.is_complete());
    }

    // ============================================
    // Pending Action Tests
    // ============================================

    #[test]
    fn test_pending_action_resource() {
        let mut world = World::new();
        world.insert_resource(PendingAction::default());

        // No pending action initially
        {
            let pending = world.resource::<PendingAction>();
            assert!(!pending.has_pending());
        }

        // Set pending action
        {
            let mut pending = world.resource_mut::<PendingAction>();
            pending.action = Some(PendingActionType::MoveUnit {
                unit_id: 1,
                path: vec![HexCoord::new(0, 0), HexCoord::new(1, 0)],
            });
            pending.target = Some(HexCoord::new(1, 0));
        }

        // Verify
        let pending = world.resource::<PendingAction>();
        assert!(pending.has_pending());
    }

    #[test]
    fn test_pending_action_clear() {
        let mut world = World::new();
        world.insert_resource(PendingAction {
            action: Some(PendingActionType::FoundCity { settler_id: 1 }),
            target: Some(HexCoord::new(5, 5)),
        });

        // Clear
        {
            let mut pending = world.resource_mut::<PendingAction>();
            pending.clear();
        }

        // Verify cleared
        let pending = world.resource::<PendingAction>();
        assert!(!pending.has_pending());
    }

    // ============================================
    // Multi-Entity Query Tests
    // ============================================

    #[test]
    fn test_query_multiple_entity_types() {
        let mut world = World::new();

        // Spawn tiles
        for i in 0..3 {
            let tile = Tile::new(HexCoord::new(i, 0), Terrain::Grassland);
            world.spawn(TileBundle::new(tile));
        }

        // Spawn units
        for i in 0..2 {
            let unit = Unit::new(i, 0, UnitType::Warrior, HexCoord::new(i as i32, 0));
            world.spawn(UnitBundle::new(unit));
        }

        // Spawn city
        let city = City::new(1, 0, "Test".to_string(), HexCoord::new(1, 0), true);
        world.spawn(CityBundle::new(city));

        // Query each type
        let tile_count = world.query::<&TileComponent>().iter(&world).count();
        let unit_count = world
            .query::<&crate::components::UnitComponent>()
            .iter(&world)
            .count();
        let city_count = world
            .query::<&crate::components::CityComponent>()
            .iter(&world)
            .count();

        assert_eq!(tile_count, 3);
        assert_eq!(unit_count, 2);
        assert_eq!(city_count, 1);
    }

    #[test]
    fn test_query_entities_at_position() {
        let mut world = World::new();

        let target_coord = HexCoord::new(5, 5);

        // Spawn entities at various positions
        for i in 0..5 {
            let unit = Unit::new(i, 0, UnitType::Warrior, HexCoord::new(i as i32, 0));
            world.spawn(UnitBundle::new(unit));
        }

        // Spawn unit at target position
        let unit = Unit::new(100, 0, UnitType::Warrior, target_coord);
        world.spawn(UnitBundle::new(unit));

        // Query and filter by position
        let mut query = world.query::<(&crate::components::UnitComponent, &PositionComponent)>();
        let at_target: Vec<_> = query
            .iter(&world)
            .filter(|(_, pos)| pos.coord == target_coord)
            .collect();

        assert_eq!(at_target.len(), 1);
    }

    // ============================================
    // System Set Configuration Tests
    // ============================================

    #[test]
    fn test_system_set_ordering_configurable() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Configure system sets (this tests that the API works)
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

        // If we got here without panic, configuration succeeded
    }

    #[test]
    fn test_add_system_to_set() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Configure sets
        app.configure_sets(
            Update,
            (GameSystemSet::Input, GameSystemSet::Update).chain(),
        );

        // Add a simple system to a set
        fn test_system() {}

        app.add_systems(Update, test_system.in_set(GameSystemSet::Input));

        // If we got here without panic, system was added successfully
    }
}
