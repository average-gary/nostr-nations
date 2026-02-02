//! Hex coordinate system for the game map.
//!
//! Uses offset "odd-q" coordinates where odd columns are shifted down.
//! This is common for hex grids displayed with pointy-top hexagons.

use serde::{Deserialize, Serialize};

/// Axial coordinates for hex grid (offset odd-q).
///
/// In this coordinate system:
/// - `q` is the column (x-axis)
/// - `r` is the row (y-axis)
/// - Odd columns are shifted down by half a hex
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, Serialize, Deserialize)]
pub struct HexCoord {
    /// Column coordinate
    pub q: i32,
    /// Row coordinate
    pub r: i32,
}

impl PartialOrd for HexCoord {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HexCoord {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Row-major ordering for deterministic iteration
        (self.r, self.q).cmp(&(other.r, other.q))
    }
}

impl HexCoord {
    /// Create a new hex coordinate.
    #[inline]
    pub const fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }

    /// Get all 6 neighboring hexes in clockwise order starting from northeast.
    ///
    /// Returns neighbors in order: NE, E, SE, SW, W, NW
    pub fn neighbors(&self) -> [HexCoord; 6] {
        // Offset depends on whether we're in an odd or even column
        let offset = if self.q & 1 == 0 { 0 } else { 1 };

        [
            // NE
            HexCoord::new(self.q + 1, self.r - 1 + offset),
            // E (same row, next column)
            HexCoord::new(self.q + 1, self.r + offset),
            // SE
            HexCoord::new(self.q, self.r + 1),
            // SW
            HexCoord::new(self.q - 1, self.r + offset),
            // W
            HexCoord::new(self.q - 1, self.r - 1 + offset),
            // NW
            HexCoord::new(self.q, self.r - 1),
        ]
    }

    /// Calculate the distance to another hex (in hex steps).
    ///
    /// Uses cube coordinate conversion for accurate distance calculation.
    pub fn distance(&self, other: &HexCoord) -> u32 {
        let (x1, y1, z1) = self.to_cube();
        let (x2, y2, z2) = other.to_cube();

        // In cube coordinates, distance is max of absolute differences
        let dx = (x1 - x2).abs();
        let dy = (y1 - y2).abs();
        let dz = (z1 - z2).abs();

        // All three should sum to equal amounts, so max gives distance
        dx.max(dy).max(dz) as u32
    }

    /// Convert offset coordinates to cube coordinates.
    ///
    /// Cube coordinates satisfy x + y + z = 0 and are useful for
    /// distance calculations and line drawing.
    pub fn to_cube(&self) -> (i32, i32, i32) {
        let x = self.q;
        let z = self.r - (self.q - (self.q & 1)) / 2;
        let y = -x - z;
        (x, y, z)
    }

    /// Create a HexCoord from cube coordinates.
    ///
    /// Note: Input must satisfy x + y + z = 0
    pub fn from_cube(x: i32, _y: i32, z: i32) -> Self {
        let q = x;
        let r = z + (x - (x & 1)) / 2;
        Self { q, r }
    }

    /// Check if this coordinate is within bounds of a rectangular map.
    pub fn in_bounds(&self, width: u32, height: u32) -> bool {
        self.q >= 0 && self.r >= 0 && (self.q as u32) < width && (self.r as u32) < height
    }

    /// Get all hexes within a given radius (inclusive).
    ///
    /// Returns a Vec of all hexes that are at most `radius` steps away.
    pub fn hexes_in_radius(&self, radius: u32) -> Vec<HexCoord> {
        let mut result = Vec::new();
        let r = radius as i32;

        for dq in -r..=r {
            for dr in -r..=r {
                let candidate = HexCoord::new(self.q + dq, self.r + dr);
                if self.distance(&candidate) <= radius {
                    result.push(candidate);
                }
            }
        }

        result
    }

    /// Get a ring of hexes at exactly the given distance.
    pub fn hex_ring(&self, radius: u32) -> Vec<HexCoord> {
        if radius == 0 {
            return vec![*self];
        }

        self.hexes_in_radius(radius)
            .into_iter()
            .filter(|h| self.distance(h) == radius)
            .collect()
    }
}

impl std::fmt::Display for HexCoord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.q, self.r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let coord = HexCoord::new(3, 5);
        assert_eq!(coord.q, 3);
        assert_eq!(coord.r, 5);
    }

    #[test]
    fn test_distance_same_hex() {
        let coord = HexCoord::new(5, 5);
        assert_eq!(coord.distance(&coord), 0);
    }

    #[test]
    fn test_distance_neighbors() {
        let coord = HexCoord::new(5, 5);
        for neighbor in coord.neighbors() {
            assert_eq!(coord.distance(&neighbor), 1);
        }
    }

    #[test]
    fn test_neighbors_count() {
        let coord = HexCoord::new(3, 3);
        assert_eq!(coord.neighbors().len(), 6);
    }

    #[test]
    fn test_in_bounds() {
        let coord = HexCoord::new(5, 5);
        assert!(coord.in_bounds(10, 10));
        assert!(!coord.in_bounds(5, 5));
        assert!(!HexCoord::new(-1, 0).in_bounds(10, 10));
    }

    #[test]
    fn test_hexes_in_radius() {
        let center = HexCoord::new(5, 5);
        let radius_0 = center.hexes_in_radius(0);
        assert_eq!(radius_0.len(), 1);
        assert!(radius_0.contains(&center));

        let radius_1 = center.hexes_in_radius(1);
        assert_eq!(radius_1.len(), 7); // center + 6 neighbors
    }

    #[test]
    fn test_cube_roundtrip() {
        let original = HexCoord::new(7, 3);
        let (x, y, z) = original.to_cube();
        let recovered = HexCoord::from_cube(x, y, z);
        assert_eq!(original, recovered);
    }

    #[test]
    fn test_display() {
        let coord = HexCoord::new(3, 7);
        assert_eq!(format!("{}", coord), "(3, 7)");
    }
}
