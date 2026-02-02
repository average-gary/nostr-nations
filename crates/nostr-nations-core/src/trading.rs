//! Resource trading system between players.
//!
//! This module provides a comprehensive trading system for:
//! - Gold and gold-per-turn deals
//! - Strategic and luxury resources
//! - Cities
//! - Technologies
//! - Diplomatic agreements (open borders, defensive pacts)

use crate::game_state::GameState;
use crate::terrain::{Resource, ResourceCategory};
use crate::types::{CityId, PlayerId, TechId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A trade offer between two players.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TradeOffer {
    /// Unique identifier for this trade offer.
    pub id: u64,
    /// Player proposing the trade.
    pub from_player: PlayerId,
    /// Player receiving the trade proposal.
    pub to_player: PlayerId,
    /// Items being offered by from_player.
    pub offer: TradeItems,
    /// Items requested from to_player.
    pub request: TradeItems,
    /// Current status of the trade.
    pub status: TradeStatus,
    /// Turn when the trade was proposed.
    pub turn_proposed: u32,
    /// Turn when the offer expires (None = no expiration).
    pub expires_turn: Option<u32>,
}

impl TradeOffer {
    /// Create a new trade offer.
    pub fn new(
        id: u64,
        from_player: PlayerId,
        to_player: PlayerId,
        offer: TradeItems,
        request: TradeItems,
        turn_proposed: u32,
        expires_turn: Option<u32>,
    ) -> Self {
        Self {
            id,
            from_player,
            to_player,
            offer,
            request,
            status: TradeStatus::Pending,
            turn_proposed,
            expires_turn,
        }
    }

    /// Check if this trade offer has expired.
    pub fn is_expired(&self, current_turn: u32) -> bool {
        if let Some(expires) = self.expires_turn {
            current_turn >= expires
        } else {
            false
        }
    }

    /// Check if this trade involves per-turn payments.
    pub fn has_per_turn_payments(&self) -> bool {
        self.offer.gold_per_turn != 0 || self.request.gold_per_turn != 0
    }
}

/// Items that can be traded between players.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TradeItems {
    /// Lump sum of gold.
    pub gold: i32,
    /// Gold paid each turn (for treaty duration).
    pub gold_per_turn: i32,
    /// Resources and their quantities.
    pub resources: HashMap<Resource, u32>,
    /// Cities to transfer.
    pub cities: Vec<CityId>,
    /// Technologies to share.
    pub technologies: Vec<TechId>,
    /// Grant open borders access.
    pub open_borders: bool,
    /// Agree to defensive pact.
    pub defensive_pact: bool,
}

impl TradeItems {
    /// Create an empty set of trade items.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add gold to the trade.
    pub fn with_gold(mut self, amount: i32) -> Self {
        self.gold = amount;
        self
    }

    /// Add gold per turn to the trade.
    pub fn with_gold_per_turn(mut self, amount: i32) -> Self {
        self.gold_per_turn = amount;
        self
    }

    /// Add a resource to the trade.
    pub fn with_resource(mut self, resource: Resource, quantity: u32) -> Self {
        self.resources.insert(resource, quantity);
        self
    }

    /// Add a city to the trade.
    pub fn with_city(mut self, city_id: CityId) -> Self {
        self.cities.push(city_id);
        self
    }

    /// Add a technology to the trade.
    pub fn with_technology(mut self, tech_id: TechId) -> Self {
        self.technologies.push(tech_id);
        self
    }

    /// Add open borders to the trade.
    pub fn with_open_borders(mut self) -> Self {
        self.open_borders = true;
        self
    }

    /// Add defensive pact to the trade.
    pub fn with_defensive_pact(mut self) -> Self {
        self.defensive_pact = true;
        self
    }

    /// Check if the trade items are empty.
    pub fn is_empty(&self) -> bool {
        self.gold == 0
            && self.gold_per_turn == 0
            && self.resources.is_empty()
            && self.cities.is_empty()
            && self.technologies.is_empty()
            && !self.open_borders
            && !self.defensive_pact
    }

    /// Count the number of distinct items in this trade.
    pub fn item_count(&self) -> usize {
        let mut count = 0;
        if self.gold != 0 {
            count += 1;
        }
        if self.gold_per_turn != 0 {
            count += 1;
        }
        count += self.resources.len();
        count += self.cities.len();
        count += self.technologies.len();
        if self.open_borders {
            count += 1;
        }
        if self.defensive_pact {
            count += 1;
        }
        count
    }
}

/// Status of a trade offer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TradeStatus {
    /// Waiting for response from target player.
    Pending,
    /// Trade was accepted and executed.
    Accepted,
    /// Trade was rejected by target player.
    Rejected,
    /// Trade was cancelled by proposing player.
    Cancelled,
    /// Trade expired before a response.
    Expired,
}

/// Errors that can occur during trade operations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TradeError {
    /// Trade offer not found.
    OfferNotFound,
    /// Trade is not in pending status.
    NotPending,
    /// Player is not authorized to perform this action.
    NotAuthorized,
    /// Insufficient gold for the trade.
    InsufficientGold,
    /// Player doesn't have the required resource.
    InsufficientResources,
    /// City doesn't exist or isn't owned by the player.
    InvalidCity,
    /// Technology doesn't exist or isn't owned by the player.
    InvalidTechnology,
    /// Players are at war and cannot trade.
    AtWar,
    /// Cannot trade with yourself.
    SelfTrade,
    /// Trade offer has expired.
    Expired,
}

impl std::fmt::Display for TradeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TradeError::OfferNotFound => write!(f, "Trade offer not found"),
            TradeError::NotPending => write!(f, "Trade is not in pending status"),
            TradeError::NotAuthorized => write!(f, "Not authorized to perform this action"),
            TradeError::InsufficientGold => write!(f, "Insufficient gold"),
            TradeError::InsufficientResources => write!(f, "Insufficient resources"),
            TradeError::InvalidCity => write!(f, "Invalid city"),
            TradeError::InvalidTechnology => write!(f, "Invalid technology"),
            TradeError::AtWar => write!(f, "Cannot trade while at war"),
            TradeError::SelfTrade => write!(f, "Cannot trade with yourself"),
            TradeError::Expired => write!(f, "Trade offer has expired"),
        }
    }
}

impl std::error::Error for TradeError {}

/// Fairness assessment of a trade offer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TradeFairness {
    /// Trade values are roughly equal.
    Fair,
    /// One side gets slightly more value.
    SlightlyUnfair,
    /// Trade is significantly one-sided.
    VeryUnfair,
    /// One side gets everything, other gets nothing.
    OneWay,
}

impl TradeFairness {
    /// Get a numerical representation of fairness.
    pub fn value(&self) -> i32 {
        match self {
            TradeFairness::Fair => 0,
            TradeFairness::SlightlyUnfair => 1,
            TradeFairness::VeryUnfair => 2,
            TradeFairness::OneWay => 3,
        }
    }
}

/// Manages all trade offers in the game.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TradeManager {
    /// All trade offers indexed by ID.
    offers: HashMap<u64, TradeOffer>,
    /// Next available trade offer ID.
    next_id: u64,
    /// Active per-turn trade agreements (offer_id -> turns_remaining).
    active_agreements: HashMap<u64, u32>,
}

impl TradeManager {
    /// Create a new trade manager.
    pub fn new() -> Self {
        Self {
            offers: HashMap::new(),
            next_id: 1,
            active_agreements: HashMap::new(),
        }
    }

    /// Propose a new trade offer.
    /// Returns the ID of the created offer.
    pub fn propose_trade(&mut self, mut offer: TradeOffer) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        offer.id = id;
        offer.status = TradeStatus::Pending;
        self.offers.insert(id, offer);
        id
    }

    /// Accept a pending trade offer.
    /// Returns the accepted offer for execution.
    pub fn accept_trade(&mut self, offer_id: u64) -> Result<TradeOffer, TradeError> {
        let offer = self
            .offers
            .get_mut(&offer_id)
            .ok_or(TradeError::OfferNotFound)?;

        if offer.status != TradeStatus::Pending {
            return Err(TradeError::NotPending);
        }

        offer.status = TradeStatus::Accepted;

        // If this has per-turn payments, track it as an active agreement
        if offer.has_per_turn_payments() {
            // Default duration of 30 turns for per-turn agreements
            self.active_agreements.insert(offer_id, 30);
        }

        Ok(offer.clone())
    }

    /// Reject a pending trade offer.
    pub fn reject_trade(&mut self, offer_id: u64) -> Result<(), TradeError> {
        let offer = self
            .offers
            .get_mut(&offer_id)
            .ok_or(TradeError::OfferNotFound)?;

        if offer.status != TradeStatus::Pending {
            return Err(TradeError::NotPending);
        }

        offer.status = TradeStatus::Rejected;
        Ok(())
    }

    /// Cancel a pending trade offer (by the proposing player).
    pub fn cancel_trade(&mut self, offer_id: u64, player: PlayerId) -> Result<(), TradeError> {
        let offer = self
            .offers
            .get_mut(&offer_id)
            .ok_or(TradeError::OfferNotFound)?;

        if offer.status != TradeStatus::Pending {
            return Err(TradeError::NotPending);
        }

        if offer.from_player != player {
            return Err(TradeError::NotAuthorized);
        }

        offer.status = TradeStatus::Cancelled;
        Ok(())
    }

    /// Get a trade offer by ID.
    pub fn get_offer(&self, offer_id: u64) -> Option<&TradeOffer> {
        self.offers.get(&offer_id)
    }

    /// Get all pending offers sent to a player.
    pub fn get_offers_for_player(&self, player: PlayerId) -> Vec<&TradeOffer> {
        self.offers
            .values()
            .filter(|o| o.to_player == player && o.status == TradeStatus::Pending)
            .collect()
    }

    /// Get all pending offers sent by a player.
    pub fn get_offers_from_player(&self, player: PlayerId) -> Vec<&TradeOffer> {
        self.offers
            .values()
            .filter(|o| o.from_player == player && o.status == TradeStatus::Pending)
            .collect()
    }

    /// Get all offers involving a player (either direction).
    pub fn get_all_offers_for_player(&self, player: PlayerId) -> Vec<&TradeOffer> {
        self.offers
            .values()
            .filter(|o| o.from_player == player || o.to_player == player)
            .collect()
    }

    /// Expire old offers based on current turn.
    pub fn expire_old_offers(&mut self, current_turn: u32) {
        for offer in self.offers.values_mut() {
            if offer.status == TradeStatus::Pending && offer.is_expired(current_turn) {
                offer.status = TradeStatus::Expired;
            }
        }
    }

    /// Process active per-turn agreements for a new turn.
    /// Returns a list of (offer_id, from_player, to_player) for agreements that need gold transfers.
    pub fn process_turn(&mut self) -> Vec<(u64, PlayerId, PlayerId, i32, i32)> {
        let mut transfers = Vec::new();
        let mut expired = Vec::new();

        for (&offer_id, turns) in &mut self.active_agreements {
            if *turns > 0 {
                *turns -= 1;
                if let Some(offer) = self.offers.get(&offer_id) {
                    // Return the gold per turn amounts for both directions
                    transfers.push((
                        offer_id,
                        offer.from_player,
                        offer.to_player,
                        offer.offer.gold_per_turn,
                        offer.request.gold_per_turn,
                    ));
                }
            }
            if *turns == 0 {
                expired.push(offer_id);
            }
        }

        // Remove expired agreements
        for id in expired {
            self.active_agreements.remove(&id);
        }

        transfers
    }

    /// Evaluate the fairness of a trade offer.
    pub fn evaluate_fairness(&self, offer: &TradeOffer, game: &GameState) -> TradeFairness {
        let offer_value = calculate_trade_value(&offer.offer, game, offer.from_player);
        let request_value = calculate_trade_value(&offer.request, game, offer.to_player);

        // Handle one-way trades
        if offer_value == 0 && request_value > 0 {
            return TradeFairness::OneWay;
        }
        if request_value == 0 && offer_value > 0 {
            return TradeFairness::OneWay;
        }

        // Calculate ratio
        let (higher, lower) = if offer_value > request_value {
            (offer_value, request_value)
        } else {
            (request_value, offer_value)
        };

        if lower == 0 {
            return TradeFairness::OneWay;
        }

        let ratio = higher as f32 / lower as f32;

        if ratio <= 1.2 {
            TradeFairness::Fair
        } else if ratio <= 1.5 {
            TradeFairness::SlightlyUnfair
        } else {
            TradeFairness::VeryUnfair
        }
    }
}

/// Calculate the estimated value of trade items from a player's perspective.
pub fn calculate_trade_value(items: &TradeItems, game: &GameState, player: PlayerId) -> i32 {
    let mut value = 0;

    // Gold is worth its face value
    value += items.gold;

    // Gold per turn is worth roughly 20x its value (assuming 30 turn agreement, discounted)
    value += items.gold_per_turn * 20;

    // Resources have varying values based on category
    for (&resource, &quantity) in &items.resources {
        let base_value = match resource.category() {
            ResourceCategory::Strategic => 50,
            ResourceCategory::Luxury => 40,
            ResourceCategory::Bonus => 15,
        };
        value += base_value * quantity as i32;
    }

    // Cities are very valuable - base value plus population
    for &city_id in &items.cities {
        if let Some(city) = game.cities.get(&city_id) {
            // Base city value + 20 per population
            value += 200 + (city.population as i32 * 20);

            // Capital is worth more
            if city.is_capital {
                value += 200;
            }

            // Wonders add value
            value += city.buildings.len() as i32 * 10;
        }
    }

    // Technologies have significant value
    for tech_id in &items.technologies {
        // Check if the receiving player already has this tech
        if let Some(receiving_player) = game.get_player(player) {
            if !receiving_player.has_tech(tech_id) {
                // Technology value based on era (approximated by position in tree)
                value += 100;
            }
        }
    }

    // Open borders has modest value
    if items.open_borders {
        value += 30;
    }

    // Defensive pact has significant value
    if items.defensive_pact {
        value += 80;
    }

    value
}

/// Execute a trade, transferring items between players.
pub fn execute_trade(game: &mut GameState, offer: &TradeOffer) -> Result<(), TradeError> {
    // Validate the trade can be executed
    validate_trade(game, offer)?;

    // Transfer items from proposer to receiver
    transfer_items(game, offer.from_player, offer.to_player, &offer.offer)?;

    // Transfer items from receiver to proposer
    transfer_items(game, offer.to_player, offer.from_player, &offer.request)?;

    Ok(())
}

/// Validate that a trade can be executed.
fn validate_trade(game: &GameState, offer: &TradeOffer) -> Result<(), TradeError> {
    // Check not trading with self
    if offer.from_player == offer.to_player {
        return Err(TradeError::SelfTrade);
    }

    // Check not at war
    if game
        .diplomacy
        .are_at_war(offer.from_player, offer.to_player)
    {
        return Err(TradeError::AtWar);
    }

    // Validate proposer can fulfill their offer
    validate_player_can_provide(game, offer.from_player, &offer.offer)?;

    // Validate receiver can fulfill their request
    validate_player_can_provide(game, offer.to_player, &offer.request)?;

    Ok(())
}

/// Validate that a player can provide the specified trade items.
fn validate_player_can_provide(
    game: &GameState,
    player: PlayerId,
    items: &TradeItems,
) -> Result<(), TradeError> {
    let player_data = game.get_player(player).ok_or(TradeError::NotAuthorized)?;

    // Check gold
    if items.gold > 0 && player_data.gold < items.gold {
        return Err(TradeError::InsufficientGold);
    }

    // Check cities
    for &city_id in &items.cities {
        if let Some(city) = game.cities.get(&city_id) {
            if city.owner != player {
                return Err(TradeError::InvalidCity);
            }
        } else {
            return Err(TradeError::InvalidCity);
        }
    }

    // Check technologies
    for tech_id in &items.technologies {
        if !player_data.has_tech(tech_id) {
            return Err(TradeError::InvalidTechnology);
        }
    }

    // Note: Resource validation would require a resource inventory system
    // which isn't fully implemented in the current codebase

    Ok(())
}

/// Transfer trade items from one player to another.
fn transfer_items(
    game: &mut GameState,
    from: PlayerId,
    to: PlayerId,
    items: &TradeItems,
) -> Result<(), TradeError> {
    // Transfer gold
    if items.gold != 0 {
        if let Some(from_player) = game.get_player_mut(from) {
            from_player.gold -= items.gold;
        }
        if let Some(to_player) = game.get_player_mut(to) {
            to_player.gold += items.gold;
        }
    }

    // Transfer cities
    for &city_id in &items.cities {
        if let Some(city) = game.cities.get_mut(&city_id) {
            city.owner = to;
            // Reset capital status if transferring a capital
            if city.is_capital {
                city.is_capital = false;
            }
        }
    }

    // Transfer technologies
    for tech_id in &items.technologies {
        if let Some(to_player) = game.get_player_mut(to) {
            if !to_player.has_tech(tech_id) {
                to_player.add_tech(tech_id.clone());
            }
        }
    }

    // Handle diplomatic agreements
    if items.open_borders {
        game.diplomacy.propose_treaty(
            from,
            to,
            crate::game_state::TreatyType::OpenBorders,
            game.turn,
        );
    }

    if items.defensive_pact {
        // Set relationship to allied if not already
        if let Some(rel) = game.diplomacy.get_mut(from, to) {
            rel.status = crate::game_state::DiplomaticStatus::Allied;
        }
        game.diplomacy.propose_treaty(
            from,
            to,
            crate::game_state::TreatyType::DefensivePact,
            game.turn,
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::{Civilization, Player};
    use crate::settings::GameSettings;

    fn create_test_game() -> GameState {
        let settings = GameSettings::new("Test".to_string());
        let mut game = GameState::new("game1".to_string(), settings, [0u8; 32]);

        let p1 = Player::new(
            0,
            "npub0".to_string(),
            "Player1".to_string(),
            Civilization::generic(),
        );
        let p2 = Player::new(
            1,
            "npub1".to_string(),
            "Player2".to_string(),
            Civilization::generic(),
        );

        game.add_player(p1).unwrap();
        game.add_player(p2).unwrap();
        game.start().unwrap();

        // Give players some gold
        game.get_player_mut(0).unwrap().gold = 1000;
        game.get_player_mut(1).unwrap().gold = 1000;

        game
    }

    #[test]
    fn test_trade_manager_creation() {
        let manager = TradeManager::new();
        assert_eq!(manager.next_id, 1);
        assert!(manager.offers.is_empty());
    }

    #[test]
    fn test_propose_trade() {
        let mut manager = TradeManager::new();

        let offer = TradeOffer::new(
            0,
            0,
            1,
            TradeItems::new().with_gold(100),
            TradeItems::new().with_gold(50),
            1,
            Some(10),
        );

        let id = manager.propose_trade(offer);
        assert_eq!(id, 1);

        let stored = manager.get_offer(id).unwrap();
        assert_eq!(stored.status, TradeStatus::Pending);
        assert_eq!(stored.offer.gold, 100);
        assert_eq!(stored.request.gold, 50);
    }

    #[test]
    fn test_accept_trade() {
        let mut manager = TradeManager::new();

        let offer = TradeOffer::new(
            0,
            0,
            1,
            TradeItems::new().with_gold(100),
            TradeItems::new().with_gold(50),
            1,
            None,
        );

        let id = manager.propose_trade(offer);
        let accepted = manager.accept_trade(id).unwrap();

        assert_eq!(accepted.status, TradeStatus::Accepted);
        assert_eq!(manager.get_offer(id).unwrap().status, TradeStatus::Accepted);
    }

    #[test]
    fn test_reject_trade() {
        let mut manager = TradeManager::new();

        let offer = TradeOffer::new(
            0,
            0,
            1,
            TradeItems::new().with_gold(100),
            TradeItems::new(),
            1,
            None,
        );

        let id = manager.propose_trade(offer);
        manager.reject_trade(id).unwrap();

        assert_eq!(manager.get_offer(id).unwrap().status, TradeStatus::Rejected);
    }

    #[test]
    fn test_cancel_trade() {
        let mut manager = TradeManager::new();

        let offer = TradeOffer::new(
            0,
            0,
            1,
            TradeItems::new().with_gold(100),
            TradeItems::new(),
            1,
            None,
        );

        let id = manager.propose_trade(offer);

        // Wrong player cannot cancel
        assert_eq!(manager.cancel_trade(id, 1), Err(TradeError::NotAuthorized));

        // Correct player can cancel
        manager.cancel_trade(id, 0).unwrap();
        assert_eq!(
            manager.get_offer(id).unwrap().status,
            TradeStatus::Cancelled
        );
    }

    #[test]
    fn test_cannot_accept_non_pending() {
        let mut manager = TradeManager::new();

        let offer = TradeOffer::new(
            0,
            0,
            1,
            TradeItems::new().with_gold(100),
            TradeItems::new(),
            1,
            None,
        );

        let id = manager.propose_trade(offer);
        manager.reject_trade(id).unwrap();

        assert_eq!(manager.accept_trade(id), Err(TradeError::NotPending));
    }

    #[test]
    fn test_get_offers_for_player() {
        let mut manager = TradeManager::new();

        // Player 0 sends to player 1
        let offer1 = TradeOffer::new(0, 0, 1, TradeItems::new(), TradeItems::new(), 1, None);
        manager.propose_trade(offer1);

        // Player 2 sends to player 1
        let offer2 = TradeOffer::new(0, 2, 1, TradeItems::new(), TradeItems::new(), 1, None);
        manager.propose_trade(offer2);

        // Player 0 sends to player 2
        let offer3 = TradeOffer::new(0, 0, 2, TradeItems::new(), TradeItems::new(), 1, None);
        manager.propose_trade(offer3);

        let offers_to_1 = manager.get_offers_for_player(1);
        assert_eq!(offers_to_1.len(), 2);

        let offers_from_0 = manager.get_offers_from_player(0);
        assert_eq!(offers_from_0.len(), 2);
    }

    #[test]
    fn test_expire_old_offers() {
        let mut manager = TradeManager::new();

        let offer = TradeOffer::new(
            0,
            0,
            1,
            TradeItems::new(),
            TradeItems::new(),
            1,
            Some(5), // Expires on turn 5
        );

        let id = manager.propose_trade(offer);

        manager.expire_old_offers(4);
        assert_eq!(manager.get_offer(id).unwrap().status, TradeStatus::Pending);

        manager.expire_old_offers(5);
        assert_eq!(manager.get_offer(id).unwrap().status, TradeStatus::Expired);
    }

    #[test]
    fn test_trade_items_builder() {
        let items = TradeItems::new()
            .with_gold(100)
            .with_gold_per_turn(10)
            .with_resource(Resource::Iron, 2)
            .with_technology("writing".to_string())
            .with_open_borders()
            .with_defensive_pact();

        assert_eq!(items.gold, 100);
        assert_eq!(items.gold_per_turn, 10);
        assert_eq!(items.resources.get(&Resource::Iron), Some(&2));
        assert!(items.technologies.contains(&"writing".to_string()));
        assert!(items.open_borders);
        assert!(items.defensive_pact);
    }

    #[test]
    fn test_trade_items_is_empty() {
        let empty = TradeItems::new();
        assert!(empty.is_empty());

        let with_gold = TradeItems::new().with_gold(100);
        assert!(!with_gold.is_empty());
    }

    #[test]
    fn test_trade_items_count() {
        let items = TradeItems::new()
            .with_gold(100)
            .with_gold_per_turn(10)
            .with_resource(Resource::Iron, 2)
            .with_resource(Resource::Horses, 1)
            .with_open_borders();

        assert_eq!(items.item_count(), 5);
    }

    #[test]
    fn test_trade_fairness_evaluation() {
        let game = create_test_game();
        let manager = TradeManager::new();

        // Fair trade: equal gold
        let fair_offer = TradeOffer::new(
            1,
            0,
            1,
            TradeItems::new().with_gold(100),
            TradeItems::new().with_gold(100),
            1,
            None,
        );
        assert_eq!(
            manager.evaluate_fairness(&fair_offer, &game),
            TradeFairness::Fair
        );

        // One-way trade
        let one_way = TradeOffer::new(
            2,
            0,
            1,
            TradeItems::new().with_gold(100),
            TradeItems::new(),
            1,
            None,
        );
        assert_eq!(
            manager.evaluate_fairness(&one_way, &game),
            TradeFairness::OneWay
        );
    }

    #[test]
    fn test_calculate_trade_value() {
        let game = create_test_game();

        // Gold value
        let gold_items = TradeItems::new().with_gold(100);
        assert_eq!(calculate_trade_value(&gold_items, &game, 0), 100);

        // Gold per turn value
        let gpt_items = TradeItems::new().with_gold_per_turn(10);
        assert_eq!(calculate_trade_value(&gpt_items, &game, 0), 200); // 10 * 20

        // Strategic resource value
        let resource_items = TradeItems::new().with_resource(Resource::Iron, 2);
        assert_eq!(calculate_trade_value(&resource_items, &game, 0), 100); // 50 * 2

        // Luxury resource value
        let luxury_items = TradeItems::new().with_resource(Resource::Gold, 1);
        assert_eq!(calculate_trade_value(&luxury_items, &game, 0), 40);

        // Open borders value
        let borders_items = TradeItems::new().with_open_borders();
        assert_eq!(calculate_trade_value(&borders_items, &game, 0), 30);

        // Combined value
        let combined = TradeItems::new()
            .with_gold(100)
            .with_resource(Resource::Iron, 1)
            .with_open_borders();
        assert_eq!(calculate_trade_value(&combined, &game, 0), 180); // 100 + 50 + 30
    }

    #[test]
    fn test_execute_trade_gold() {
        let mut game = create_test_game();

        let offer = TradeOffer::new(
            1,
            0,
            1,
            TradeItems::new().with_gold(100),
            TradeItems::new().with_gold(50),
            1,
            None,
        );

        execute_trade(&mut game, &offer).unwrap();

        // Player 0: started with 1000, gave 100, received 50 = 950
        assert_eq!(game.get_player(0).unwrap().gold, 950);
        // Player 1: started with 1000, gave 50, received 100 = 1050
        assert_eq!(game.get_player(1).unwrap().gold, 1050);
    }

    #[test]
    fn test_execute_trade_insufficient_gold() {
        let mut game = create_test_game();
        game.get_player_mut(0).unwrap().gold = 50;

        let offer = TradeOffer::new(
            1,
            0,
            1,
            TradeItems::new().with_gold(100),
            TradeItems::new(),
            1,
            None,
        );

        assert_eq!(
            execute_trade(&mut game, &offer),
            Err(TradeError::InsufficientGold)
        );
    }

    #[test]
    fn test_execute_trade_at_war() {
        let mut game = create_test_game();
        game.diplomacy.declare_war(0, 1, 1);

        let offer = TradeOffer::new(
            1,
            0,
            1,
            TradeItems::new().with_gold(100),
            TradeItems::new(),
            1,
            None,
        );

        assert_eq!(execute_trade(&mut game, &offer), Err(TradeError::AtWar));
    }

    #[test]
    fn test_execute_trade_self_trade() {
        let mut game = create_test_game();

        let offer = TradeOffer::new(
            1,
            0,
            0, // Same player
            TradeItems::new().with_gold(100),
            TradeItems::new(),
            1,
            None,
        );

        assert_eq!(execute_trade(&mut game, &offer), Err(TradeError::SelfTrade));
    }

    #[test]
    fn test_execute_trade_technology() {
        let mut game = create_test_game();

        // Give player 0 a technology
        game.get_player_mut(0)
            .unwrap()
            .add_tech("writing".to_string());

        let offer = TradeOffer::new(
            1,
            0,
            1,
            TradeItems::new().with_technology("writing".to_string()),
            TradeItems::new().with_gold(100),
            1,
            None,
        );

        execute_trade(&mut game, &offer).unwrap();

        // Player 1 should now have the technology
        assert!(game.get_player(1).unwrap().has_tech(&"writing".to_string()));
    }

    #[test]
    fn test_execute_trade_invalid_technology() {
        let mut game = create_test_game();

        // Player 0 does NOT have the technology
        let offer = TradeOffer::new(
            1,
            0,
            1,
            TradeItems::new().with_technology("writing".to_string()),
            TradeItems::new(),
            1,
            None,
        );

        assert_eq!(
            execute_trade(&mut game, &offer),
            Err(TradeError::InvalidTechnology)
        );
    }

    #[test]
    fn test_per_turn_agreement_tracking() {
        let mut manager = TradeManager::new();

        let offer = TradeOffer::new(
            0,
            0,
            1,
            TradeItems::new().with_gold_per_turn(10),
            TradeItems::new(),
            1,
            None,
        );

        let id = manager.propose_trade(offer);
        manager.accept_trade(id).unwrap();

        // Should have an active agreement
        assert!(manager.active_agreements.contains_key(&id));

        // Process turn should return transfer info
        let transfers = manager.process_turn();
        assert_eq!(transfers.len(), 1);
        assert_eq!(transfers[0].0, id); // offer_id
        assert_eq!(transfers[0].3, 10); // gold_per_turn from offer
    }

    #[test]
    fn test_trade_offer_expiration() {
        let offer = TradeOffer::new(1, 0, 1, TradeItems::new(), TradeItems::new(), 1, Some(5));

        assert!(!offer.is_expired(4));
        assert!(offer.is_expired(5));
        assert!(offer.is_expired(6));
    }

    #[test]
    fn test_trade_offer_no_expiration() {
        let offer = TradeOffer::new(
            1,
            0,
            1,
            TradeItems::new(),
            TradeItems::new(),
            1,
            None, // No expiration
        );

        assert!(!offer.is_expired(100));
        assert!(!offer.is_expired(1000));
    }

    #[test]
    fn test_trade_error_display() {
        assert_eq!(
            format!("{}", TradeError::OfferNotFound),
            "Trade offer not found"
        );
        assert_eq!(
            format!("{}", TradeError::AtWar),
            "Cannot trade while at war"
        );
        assert_eq!(
            format!("{}", TradeError::SelfTrade),
            "Cannot trade with yourself"
        );
    }

    #[test]
    fn test_trade_fairness_values() {
        assert_eq!(TradeFairness::Fair.value(), 0);
        assert_eq!(TradeFairness::SlightlyUnfair.value(), 1);
        assert_eq!(TradeFairness::VeryUnfair.value(), 2);
        assert_eq!(TradeFairness::OneWay.value(), 3);
    }

    #[test]
    fn test_trade_manager_serialization() {
        let mut manager = TradeManager::new();

        let offer = TradeOffer::new(
            0,
            0,
            1,
            TradeItems::new()
                .with_gold(100)
                .with_resource(Resource::Iron, 2),
            TradeItems::new().with_gold(50),
            1,
            Some(10),
        );

        manager.propose_trade(offer);

        let json = serde_json::to_string(&manager).unwrap();
        let restored: TradeManager = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.next_id, manager.next_id);
        assert_eq!(restored.offers.len(), manager.offers.len());

        let restored_offer = restored.get_offer(1).unwrap();
        assert_eq!(restored_offer.offer.gold, 100);
        assert_eq!(
            restored_offer.offer.resources.get(&Resource::Iron),
            Some(&2)
        );
    }
}
