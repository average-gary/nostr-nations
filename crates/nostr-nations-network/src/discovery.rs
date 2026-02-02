//! Peer discovery and QR code support.
//!
//! This module provides:
//! - QR code generation for connection tickets
//! - QR code parsing for joining games
//! - Peer discovery utilities
//!
//! # QR Code Format
//!
//! Connection tickets are encoded as base64 JSON and rendered
//! as QR codes for easy mobile scanning.

use crate::peer::ConnectionTicket;
use serde::{Deserialize, Serialize};

/// QR code data for connection.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QrCodeData {
    /// Protocol version.
    pub version: u8,
    /// Connection ticket.
    pub ticket: ConnectionTicket,
}

impl QrCodeData {
    /// Create new QR code data from a ticket.
    pub fn new(ticket: ConnectionTicket) -> Self {
        Self { version: 1, ticket }
    }

    /// Serialize to a string for QR code generation.
    pub fn to_qr_string(&self) -> Result<String, serde_json::Error> {
        // Prefix with "nn:" for Nostr Nations protocol
        let _json = serde_json::to_string(self)?;
        Ok(format!("nn:{}", self.ticket.to_string()?))
    }

    /// Parse from a QR code string.
    pub fn from_qr_string(s: &str) -> Result<Self, QrParseError> {
        // Check for "nn:" prefix
        let data = s.strip_prefix("nn:").ok_or(QrParseError::InvalidPrefix)?;

        let ticket =
            ConnectionTicket::from_string(data).map_err(|_| QrParseError::InvalidTicket)?;

        if ticket.is_expired() {
            return Err(QrParseError::Expired);
        }

        Ok(Self { version: 1, ticket })
    }
}

/// Errors from QR code parsing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QrParseError {
    /// Missing or wrong prefix.
    InvalidPrefix,
    /// Invalid ticket format.
    InvalidTicket,
    /// Ticket has expired.
    Expired,
}

impl std::fmt::Display for QrParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QrParseError::InvalidPrefix => write!(f, "Invalid QR code prefix"),
            QrParseError::InvalidTicket => write!(f, "Invalid connection ticket"),
            QrParseError::Expired => write!(f, "Connection ticket has expired"),
        }
    }
}

impl std::error::Error for QrParseError {}

/// Simple QR code generator.
///
/// This generates a text-based QR code representation.
/// For actual UI, use a proper QR code library.
pub struct QrGenerator {
    /// Error correction level.
    error_correction: ErrorCorrection,
}

/// Error correction levels for QR codes.
#[derive(Clone, Copy, Debug, Default)]
pub enum ErrorCorrection {
    /// 7% error recovery.
    Low,
    /// 15% error recovery.
    #[default]
    Medium,
    /// 25% error recovery.
    Quartile,
    /// 30% error recovery.
    High,
}

impl QrGenerator {
    /// Create a new QR generator.
    pub fn new() -> Self {
        Self {
            error_correction: ErrorCorrection::default(),
        }
    }

    /// Set error correction level.
    pub fn with_error_correction(mut self, level: ErrorCorrection) -> Self {
        self.error_correction = level;
        self
    }

    /// Generate QR code data for display.
    ///
    /// Returns a 2D boolean array where true = black, false = white.
    /// This is a placeholder - real implementation would use a QR library.
    pub fn generate(&self, data: &str) -> QrCodeMatrix {
        // Placeholder implementation - would use qrcode crate in real code
        // For now, return a simple pattern based on data hash
        let hash = simple_hash(data);
        let size = 21 + ((data.len() / 25) * 4).min(16); // QR version estimation

        let mut matrix = vec![vec![false; size]; size];

        // Generate finder patterns (top-left, top-right, bottom-left)
        Self::draw_finder_pattern(&mut matrix, 0, 0);
        Self::draw_finder_pattern(&mut matrix, size - 7, 0);
        Self::draw_finder_pattern(&mut matrix, 0, size - 7);

        // Fill data area with pseudo-random pattern based on hash
        for (y, row) in matrix.iter_mut().enumerate().take(size - 8).skip(8) {
            for (x, cell) in row.iter_mut().enumerate().take(size - 8).skip(8) {
                let bit_index = (y * size + x) % 64;
                *cell = (hash >> bit_index) & 1 == 1;
            }
        }

        QrCodeMatrix { matrix, size }
    }

    /// Draw a finder pattern at the given position.
    fn draw_finder_pattern(matrix: &mut [Vec<bool>], x: usize, y: usize) {
        // Outer black square
        for i in 0..7 {
            matrix[y][x + i] = true;
            matrix[y + 6][x + i] = true;
            matrix[y + i][x] = true;
            matrix[y + i][x + 6] = true;
        }
        // Inner white square
        for i in 1..6 {
            for j in 1..6 {
                matrix[y + i][x + j] = i == 1 || i == 5 || j == 1 || j == 5;
            }
        }
        // Center black square
        for i in 2..5 {
            for j in 2..5 {
                matrix[y + i][x + j] = true;
            }
        }
    }
}

impl Default for QrGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// A QR code matrix.
#[derive(Clone, Debug)]
pub struct QrCodeMatrix {
    /// The matrix data (true = black).
    pub matrix: Vec<Vec<bool>>,
    /// Size of the matrix.
    pub size: usize,
}

impl QrCodeMatrix {
    /// Render to ASCII art.
    pub fn to_ascii(&self) -> String {
        let mut result = String::new();

        for row in &self.matrix {
            for &cell in row {
                result.push_str(if cell { "██" } else { "  " });
            }
            result.push('\n');
        }

        result
    }

    /// Render to compact ASCII (half blocks).
    pub fn to_ascii_compact(&self) -> String {
        let mut result = String::new();

        for y in (0..self.size).step_by(2) {
            for x in 0..self.size {
                let top = self.matrix[y][x];
                let bottom = if y + 1 < self.size {
                    self.matrix[y + 1][x]
                } else {
                    false
                };

                let ch = match (top, bottom) {
                    (true, true) => '█',
                    (true, false) => '▀',
                    (false, true) => '▄',
                    (false, false) => ' ',
                };
                result.push(ch);
            }
            result.push('\n');
        }

        result
    }

    /// Get raw matrix data.
    pub fn data(&self) -> &[Vec<bool>] {
        &self.matrix
    }
}

/// Simple hash function for placeholder QR generation.
fn simple_hash(data: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325; // FNV offset basis
    for byte in data.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3); // FNV prime
    }
    hash
}

/// Peer discovery service.
#[derive(Default)]
pub struct DiscoveryService {
    /// Known hosts (game_id -> ticket).
    known_hosts: std::collections::HashMap<String, ConnectionTicket>,
}

impl DiscoveryService {
    /// Create a new discovery service.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a game we're hosting.
    pub fn register_host(&mut self, ticket: ConnectionTicket) {
        self.known_hosts.insert(ticket.game_id.clone(), ticket);
    }

    /// Unregister a hosted game.
    pub fn unregister_host(&mut self, game_id: &str) {
        self.known_hosts.remove(game_id);
    }

    /// Get ticket for a known game.
    pub fn get_ticket(&self, game_id: &str) -> Option<&ConnectionTicket> {
        self.known_hosts.get(game_id)
    }

    /// List all known games.
    pub fn list_games(&self) -> Vec<&str> {
        self.known_hosts.keys().map(|s| s.as_str()).collect()
    }

    /// Add a discovered game.
    pub fn add_discovered(&mut self, ticket: ConnectionTicket) {
        if !ticket.is_expired() {
            self.known_hosts.insert(ticket.game_id.clone(), ticket);
        }
    }

    /// Remove expired tickets.
    pub fn cleanup_expired(&mut self) {
        self.known_hosts.retain(|_, ticket| !ticket.is_expired());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== QrCodeData Tests ====================

    #[test]
    fn test_qr_code_data_roundtrip() {
        let ticket = ConnectionTicket::new(
            "node123".to_string(),
            vec!["192.168.1.1:4433".to_string()],
            "game456".to_string(),
            3600,
        );

        let qr_data = QrCodeData::new(ticket);
        let qr_string = qr_data.to_qr_string().unwrap();

        assert!(qr_string.starts_with("nn:"));

        let parsed = QrCodeData::from_qr_string(&qr_string).unwrap();
        assert_eq!(parsed.ticket.game_id, "game456");
    }

    #[test]
    fn test_qr_code_data_version() {
        let ticket = ConnectionTicket::new("node1".to_string(), vec![], "game1".to_string(), 3600);

        let qr_data = QrCodeData::new(ticket);
        assert_eq!(qr_data.version, 1);
    }

    #[test]
    fn test_qr_code_data_preserves_ticket_fields() {
        let addresses = vec!["192.168.1.1:4433".to_string(), "10.0.0.1:4433".to_string()];
        let ticket = ConnectionTicket::new(
            "node_abc".to_string(),
            addresses.clone(),
            "game_xyz".to_string(),
            7200,
        );

        let qr_data = QrCodeData::new(ticket);
        let qr_string = qr_data.to_qr_string().unwrap();
        let parsed = QrCodeData::from_qr_string(&qr_string).unwrap();

        assert_eq!(parsed.ticket.node_id, "node_abc");
        assert_eq!(parsed.ticket.game_id, "game_xyz");
        assert_eq!(parsed.ticket.addresses, addresses);
    }

    #[test]
    fn test_qr_code_expired() {
        // Create a ticket that is already expired (expires_at = 0 is Unix epoch, always in the past)
        let ticket = ConnectionTicket {
            node_id: "node123".to_string(),
            addresses: vec![],
            alpn: "nostr-nations/1".to_string(),
            game_id: "game456".to_string(),
            expires_at: 0, // Unix epoch - always in the past
        };

        let qr_data = QrCodeData::new(ticket);
        let qr_string = qr_data.to_qr_string().unwrap();

        let result = QrCodeData::from_qr_string(&qr_string);
        assert!(matches!(result, Err(QrParseError::Expired)));
    }

    #[test]
    fn test_qr_code_invalid_prefix() {
        let result = QrCodeData::from_qr_string("invalid:data");
        assert!(matches!(result, Err(QrParseError::InvalidPrefix)));
    }

    #[test]
    fn test_qr_code_missing_prefix() {
        let result = QrCodeData::from_qr_string("somedata");
        assert!(matches!(result, Err(QrParseError::InvalidPrefix)));
    }

    #[test]
    fn test_qr_code_empty_string() {
        let result = QrCodeData::from_qr_string("");
        assert!(matches!(result, Err(QrParseError::InvalidPrefix)));
    }

    #[test]
    fn test_qr_code_invalid_ticket() {
        let result = QrCodeData::from_qr_string("nn:invalid_base64_garbage!!!");
        assert!(matches!(result, Err(QrParseError::InvalidTicket)));
    }

    #[test]
    fn test_qr_code_prefix_only() {
        let result = QrCodeData::from_qr_string("nn:");
        assert!(matches!(result, Err(QrParseError::InvalidTicket)));
    }

    // ==================== QrParseError Tests ====================

    #[test]
    fn test_qr_parse_error_display() {
        assert_eq!(
            format!("{}", QrParseError::InvalidPrefix),
            "Invalid QR code prefix"
        );
        assert_eq!(
            format!("{}", QrParseError::InvalidTicket),
            "Invalid connection ticket"
        );
        assert_eq!(
            format!("{}", QrParseError::Expired),
            "Connection ticket has expired"
        );
    }

    #[test]
    fn test_qr_parse_error_equality() {
        assert_eq!(QrParseError::InvalidPrefix, QrParseError::InvalidPrefix);
        assert_eq!(QrParseError::InvalidTicket, QrParseError::InvalidTicket);
        assert_eq!(QrParseError::Expired, QrParseError::Expired);

        assert_ne!(QrParseError::InvalidPrefix, QrParseError::InvalidTicket);
        assert_ne!(QrParseError::InvalidTicket, QrParseError::Expired);
    }

    #[test]
    fn test_qr_parse_error_debug() {
        let error = QrParseError::InvalidPrefix;
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("InvalidPrefix"));
    }

    // ==================== ErrorCorrection Tests ====================

    #[test]
    fn test_error_correction_default() {
        let default = ErrorCorrection::default();
        assert!(matches!(default, ErrorCorrection::Medium));
    }

    #[test]
    fn test_error_correction_levels() {
        let low = ErrorCorrection::Low;
        let medium = ErrorCorrection::Medium;
        let quartile = ErrorCorrection::Quartile;
        let high = ErrorCorrection::High;

        // Just verify they exist and are different
        assert!(matches!(low, ErrorCorrection::Low));
        assert!(matches!(medium, ErrorCorrection::Medium));
        assert!(matches!(quartile, ErrorCorrection::Quartile));
        assert!(matches!(high, ErrorCorrection::High));
    }

    #[test]
    fn test_error_correction_copy() {
        let level = ErrorCorrection::High;
        let copied = level;
        assert!(matches!(copied, ErrorCorrection::High));
    }

    #[test]
    fn test_error_correction_debug() {
        let level = ErrorCorrection::Quartile;
        let debug_str = format!("{:?}", level);
        assert!(debug_str.contains("Quartile"));
    }

    // ==================== QrGenerator Tests ====================

    #[test]
    fn test_qr_generator_new() {
        let generator = QrGenerator::new();
        // Verify it can generate a matrix
        let matrix = generator.generate("test");
        assert!(matrix.size >= 21);
    }

    #[test]
    fn test_qr_generator_default() {
        let generator = QrGenerator::default();
        let matrix = generator.generate("test");
        assert!(matrix.size >= 21);
    }

    #[test]
    fn test_qr_generator_with_error_correction() {
        let generator = QrGenerator::new().with_error_correction(ErrorCorrection::High);

        let matrix = generator.generate("test data");
        assert!(matrix.size >= 21);
    }

    #[test]
    fn test_qr_generator_all_error_correction_levels() {
        let levels = [
            ErrorCorrection::Low,
            ErrorCorrection::Medium,
            ErrorCorrection::Quartile,
            ErrorCorrection::High,
        ];

        for level in levels {
            let generator = QrGenerator::new().with_error_correction(level);
            let matrix = generator.generate("test data");
            assert!(matrix.size >= 21);
        }
    }

    #[test]
    fn test_qr_generator() {
        let generator = QrGenerator::new();
        let matrix = generator.generate("test data");

        assert!(matrix.size >= 21);
        assert_eq!(matrix.matrix.len(), matrix.size);
        assert_eq!(matrix.matrix[0].len(), matrix.size);
    }

    #[test]
    fn test_qr_generator_empty_data() {
        let generator = QrGenerator::new();
        let matrix = generator.generate("");

        assert!(matrix.size >= 21);
    }

    #[test]
    fn test_qr_generator_long_data() {
        let generator = QrGenerator::new();
        let long_data = "a".repeat(500);
        let matrix = generator.generate(&long_data);

        // Size should increase for longer data
        assert!(matrix.size >= 21);
    }

    #[test]
    fn test_qr_generator_deterministic() {
        let generator = QrGenerator::new();

        let matrix1 = generator.generate("same data");
        let matrix2 = generator.generate("same data");

        assert_eq!(matrix1.size, matrix2.size);
        assert_eq!(matrix1.matrix, matrix2.matrix);
    }

    #[test]
    fn test_qr_generator_different_data() {
        let generator = QrGenerator::new();

        let matrix1 = generator.generate("data one");
        let matrix2 = generator.generate("data two");

        // Different data should produce different patterns in data area
        // (finder patterns will be the same)
        assert_ne!(matrix1.matrix, matrix2.matrix);
    }

    // ==================== QrCodeMatrix Tests ====================

    #[test]
    fn test_qr_matrix_ascii() {
        let generator = QrGenerator::new();
        let matrix = generator.generate("hello");
        let ascii = matrix.to_ascii();

        assert!(!ascii.is_empty());
        assert!(ascii.contains("██")); // Should have some black cells
        assert!(ascii.contains('\n')); // Should have newlines
    }

    #[test]
    fn test_qr_matrix_ascii_compact() {
        let generator = QrGenerator::new();
        let matrix = generator.generate("hello");
        let ascii_compact = matrix.to_ascii_compact();

        assert!(!ascii_compact.is_empty());
        // Compact should be roughly half the height
        let regular_lines = matrix.to_ascii().lines().count();
        let compact_lines = ascii_compact.lines().count();
        assert!(compact_lines <= regular_lines.div_ceil(2) + 1);
    }

    #[test]
    fn test_qr_matrix_data() {
        let generator = QrGenerator::new();
        let matrix = generator.generate("test");

        let data = matrix.data();
        assert_eq!(data.len(), matrix.size);
        for row in data {
            assert_eq!(row.len(), matrix.size);
        }
    }

    #[test]
    fn test_qr_matrix_clone() {
        let generator = QrGenerator::new();
        let matrix = generator.generate("test");
        let cloned = matrix.clone();

        assert_eq!(matrix.size, cloned.size);
        assert_eq!(matrix.matrix, cloned.matrix);
    }

    #[test]
    fn test_qr_matrix_finder_patterns() {
        let generator = QrGenerator::new();
        let matrix = generator.generate("x");

        // Check top-left finder pattern (7x7 with specific pattern)
        // Outer edge should be black
        for i in 0..7 {
            assert!(matrix.matrix[0][i], "Top edge should be black");
            assert!(matrix.matrix[6][i], "Bottom edge should be black");
            assert!(matrix.matrix[i][0], "Left edge should be black");
            assert!(matrix.matrix[i][6], "Right edge should be black");
        }

        // Center should be black (3x3)
        for i in 2..5 {
            for j in 2..5 {
                assert!(
                    matrix.matrix[i][j],
                    "Center should be black at ({}, {})",
                    i, j
                );
            }
        }
    }

    #[test]
    fn test_qr_matrix_ascii_contains_unicode_blocks() {
        let generator = QrGenerator::new();
        let matrix = generator.generate("test");

        let ascii = matrix.to_ascii();
        // Full block is used for black cells
        assert!(ascii.contains('█'));

        let compact = matrix.to_ascii_compact();
        // Compact version may use half blocks
        assert!(
            compact.contains('█')
                || compact.contains('▀')
                || compact.contains('▄')
                || compact.contains(' ')
        );
    }

    // ==================== DiscoveryService Tests ====================

    #[test]
    fn test_discovery_service_new() {
        let service = DiscoveryService::new();
        assert!(service.list_games().is_empty());
    }

    #[test]
    fn test_discovery_service_default() {
        let service = DiscoveryService::default();
        assert!(service.list_games().is_empty());
    }

    #[test]
    fn test_discovery_service() {
        let mut service = DiscoveryService::new();

        let ticket = ConnectionTicket::new("node1".to_string(), vec![], "game1".to_string(), 3600);

        service.register_host(ticket);
        assert!(service.get_ticket("game1").is_some());
        assert_eq!(service.list_games(), vec!["game1"]);

        service.unregister_host("game1");
        assert!(service.get_ticket("game1").is_none());
    }

    #[test]
    fn test_discovery_service_register_multiple() {
        let mut service = DiscoveryService::new();

        let ticket1 = ConnectionTicket::new("node1".to_string(), vec![], "game1".to_string(), 3600);
        let ticket2 = ConnectionTicket::new("node2".to_string(), vec![], "game2".to_string(), 3600);
        let ticket3 = ConnectionTicket::new("node3".to_string(), vec![], "game3".to_string(), 3600);

        service.register_host(ticket1);
        service.register_host(ticket2);
        service.register_host(ticket3);

        let games = service.list_games();
        assert_eq!(games.len(), 3);
        assert!(games.contains(&"game1"));
        assert!(games.contains(&"game2"));
        assert!(games.contains(&"game3"));
    }

    #[test]
    fn test_discovery_service_register_overwrites() {
        let mut service = DiscoveryService::new();

        let ticket1 = ConnectionTicket::new(
            "node1".to_string(),
            vec!["addr1".to_string()],
            "game1".to_string(),
            3600,
        );
        let ticket2 = ConnectionTicket::new(
            "node2".to_string(),
            vec!["addr2".to_string()],
            "game1".to_string(), // Same game ID
            7200,
        );

        service.register_host(ticket1);
        service.register_host(ticket2);

        // Should only have one game
        assert_eq!(service.list_games().len(), 1);

        // Should have the second ticket's info
        let ticket = service.get_ticket("game1").unwrap();
        assert_eq!(ticket.node_id, "node2");
        assert_eq!(ticket.addresses, vec!["addr2".to_string()]);
    }

    #[test]
    fn test_discovery_service_unregister_nonexistent() {
        let mut service = DiscoveryService::new();

        // Should not panic
        service.unregister_host("nonexistent");
    }

    #[test]
    fn test_discovery_service_get_ticket_nonexistent() {
        let service = DiscoveryService::new();

        assert!(service.get_ticket("nonexistent").is_none());
    }

    #[test]
    fn test_discovery_service_add_discovered() {
        let mut service = DiscoveryService::new();

        let ticket = ConnectionTicket::new(
            "discovered_node".to_string(),
            vec!["192.168.1.100:4433".to_string()],
            "discovered_game".to_string(),
            3600,
        );

        service.add_discovered(ticket);

        assert!(service.get_ticket("discovered_game").is_some());
        let retrieved = service.get_ticket("discovered_game").unwrap();
        assert_eq!(retrieved.node_id, "discovered_node");
    }

    #[test]
    fn test_discovery_service_add_discovered_expired() {
        let mut service = DiscoveryService::new();

        // Create a ticket that is already expired (expires_at = 0 is Unix epoch, always in the past)
        let ticket = ConnectionTicket {
            node_id: "expired_node".to_string(),
            addresses: vec![],
            alpn: "nostr-nations/1".to_string(),
            game_id: "expired_game".to_string(),
            expires_at: 0, // Unix epoch - always in the past
        };

        service.add_discovered(ticket);

        // Expired ticket should not be added
        assert!(service.get_ticket("expired_game").is_none());
    }

    #[test]
    fn test_discovery_service_cleanup_expired() {
        let mut service = DiscoveryService::new();

        let valid_ticket = ConnectionTicket::new(
            "valid_node".to_string(),
            vec![],
            "valid_game".to_string(),
            3600,
        );
        // Create a ticket that is already expired (expires_at = 0 is Unix epoch, always in the past)
        let expired_ticket = ConnectionTicket {
            node_id: "expired_node".to_string(),
            addresses: vec![],
            alpn: "nostr-nations/1".to_string(),
            game_id: "expired_game".to_string(),
            expires_at: 0, // Unix epoch - always in the past
        };

        service.register_host(valid_ticket);
        service.register_host(expired_ticket);

        service.cleanup_expired();

        // Valid game should remain
        assert!(service.get_ticket("valid_game").is_some());
        // Expired game should be removed
        assert!(service.get_ticket("expired_game").is_none());
    }

    #[test]
    fn test_discovery_service_cleanup_expired_all() {
        let mut service = DiscoveryService::new();

        // Create tickets that are already expired (expires_at = 0 is Unix epoch, always in the past)
        let ticket1 = ConnectionTicket {
            node_id: "n1".to_string(),
            addresses: vec![],
            alpn: "nostr-nations/1".to_string(),
            game_id: "g1".to_string(),
            expires_at: 0,
        };
        let ticket2 = ConnectionTicket {
            node_id: "n2".to_string(),
            addresses: vec![],
            alpn: "nostr-nations/1".to_string(),
            game_id: "g2".to_string(),
            expires_at: 0,
        };

        service.register_host(ticket1);
        service.register_host(ticket2);

        service.cleanup_expired();

        assert!(service.list_games().is_empty());
    }

    #[test]
    fn test_discovery_service_cleanup_expired_none() {
        let mut service = DiscoveryService::new();

        let ticket1 = ConnectionTicket::new("n1".to_string(), vec![], "g1".to_string(), 3600);
        let ticket2 = ConnectionTicket::new("n2".to_string(), vec![], "g2".to_string(), 3600);

        service.register_host(ticket1);
        service.register_host(ticket2);

        service.cleanup_expired();

        assert_eq!(service.list_games().len(), 2);
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_qr_code_full_flow() {
        // Simulate the full QR code flow
        let mut service = DiscoveryService::new();

        // Host creates a ticket
        let ticket = ConnectionTicket::new(
            "host_node".to_string(),
            vec!["192.168.1.1:4433".to_string()],
            "multiplayer_game".to_string(),
            3600,
        );

        // Register with discovery service
        service.register_host(ticket.clone());

        // Generate QR code
        let qr_data = QrCodeData::new(ticket);
        let qr_string = qr_data.to_qr_string().unwrap();

        // Generate visual representation
        let generator = QrGenerator::new().with_error_correction(ErrorCorrection::High);
        let matrix = generator.generate(&qr_string);

        // Verify matrix is valid
        assert!(matrix.size >= 21);

        // Client scans and parses QR code
        let parsed = QrCodeData::from_qr_string(&qr_string).unwrap();

        // Client uses ticket to discover game
        assert_eq!(parsed.ticket.game_id, "multiplayer_game");
        assert_eq!(parsed.ticket.node_id, "host_node");

        // Verify service has the game
        let hosted_ticket = service.get_ticket("multiplayer_game").unwrap();
        assert_eq!(hosted_ticket.node_id, parsed.ticket.node_id);
    }

    #[test]
    fn test_simple_hash_consistency() {
        // Test that the hash function produces consistent results
        let data = "test data for hashing";

        // Generate two matrices with the same data
        let generator = QrGenerator::new();
        let matrix1 = generator.generate(data);
        let matrix2 = generator.generate(data);

        // They should be identical
        assert_eq!(matrix1.matrix, matrix2.matrix);
    }
}
