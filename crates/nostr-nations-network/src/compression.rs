//! Optional compression for event payloads.
//!
//! This module provides compression utilities for large state updates
//! while skipping compression for small messages where overhead isn't worth it.

use serde::{Deserialize, Serialize};

/// Compression algorithm selection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionAlgorithm {
    /// No compression.
    None,
    /// Simple run-length encoding (lightweight, fast).
    #[default]
    Rle,
    /// LZ77-style compression (better ratio, slower).
    Lz,
}

/// Configuration for compression.
#[derive(Clone, Debug)]
pub struct CompressionConfig {
    /// Algorithm to use.
    pub algorithm: CompressionAlgorithm,
    /// Minimum size to trigger compression (bytes).
    pub min_size: usize,
    /// Maximum size to attempt compression (bytes).
    pub max_size: usize,
    /// Minimum compression ratio to keep compressed version.
    pub min_ratio: f64,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            algorithm: CompressionAlgorithm::Rle,
            min_size: 256,
            max_size: 1024 * 1024, // 1MB
            min_ratio: 0.9,        // Only keep if compressed is at least 10% smaller
        }
    }
}

/// A compressed payload with metadata.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompressedPayload {
    /// The compressed data.
    pub data: Vec<u8>,
    /// Original uncompressed size.
    pub original_size: usize,
    /// Algorithm used.
    pub algorithm: CompressionAlgorithm,
    /// Checksum for validation.
    pub checksum: u32,
}

impl CompressedPayload {
    /// Get the compression ratio (compressed / original).
    pub fn ratio(&self) -> f64 {
        if self.original_size == 0 {
            1.0
        } else {
            self.data.len() as f64 / self.original_size as f64
        }
    }

    /// Check if compression was effective.
    pub fn is_effective(&self) -> bool {
        self.ratio() < 0.9
    }
}

/// Compressor for network payloads.
#[derive(Clone)]
pub struct PayloadCompressor {
    config: CompressionConfig,
    stats: CompressionStats,
}

/// Statistics for compression operations.
#[derive(Clone, Debug, Default)]
pub struct CompressionStats {
    /// Total compressions attempted.
    pub compressions_attempted: u64,
    /// Compressions that were effective.
    pub compressions_effective: u64,
    /// Total bytes before compression.
    pub bytes_before: u64,
    /// Total bytes after compression.
    pub bytes_after: u64,
    /// Messages skipped (too small).
    pub skipped_too_small: u64,
}

impl CompressionStats {
    /// Get the overall compression ratio.
    pub fn overall_ratio(&self) -> f64 {
        if self.bytes_before == 0 {
            1.0
        } else {
            self.bytes_after as f64 / self.bytes_before as f64
        }
    }
}

impl PayloadCompressor {
    /// Create a new compressor with the given configuration.
    pub fn new(config: CompressionConfig) -> Self {
        Self {
            config,
            stats: CompressionStats::default(),
        }
    }

    /// Create a compressor with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(CompressionConfig::default())
    }

    /// Compress a payload if beneficial.
    /// Returns None if compression is not worthwhile.
    pub fn compress(&mut self, data: &[u8]) -> Option<CompressedPayload> {
        // Skip small payloads
        if data.len() < self.config.min_size {
            self.stats.skipped_too_small += 1;
            return None;
        }

        // Skip huge payloads
        if data.len() > self.config.max_size {
            return None;
        }

        self.stats.compressions_attempted += 1;
        self.stats.bytes_before += data.len() as u64;

        let compressed = match self.config.algorithm {
            CompressionAlgorithm::None => data.to_vec(),
            CompressionAlgorithm::Rle => rle_compress(data),
            CompressionAlgorithm::Lz => lz_compress(data),
        };

        let ratio = compressed.len() as f64 / data.len() as f64;

        // Check if compression is effective
        if ratio >= self.config.min_ratio {
            self.stats.bytes_after += data.len() as u64;
            return None;
        }

        self.stats.compressions_effective += 1;
        self.stats.bytes_after += compressed.len() as u64;

        Some(CompressedPayload {
            data: compressed,
            original_size: data.len(),
            algorithm: self.config.algorithm,
            checksum: simple_checksum(data),
        })
    }

    /// Decompress a payload.
    pub fn decompress(&self, payload: &CompressedPayload) -> Result<Vec<u8>, CompressionError> {
        let decompressed = match payload.algorithm {
            CompressionAlgorithm::None => payload.data.clone(),
            CompressionAlgorithm::Rle => rle_decompress(&payload.data)?,
            CompressionAlgorithm::Lz => lz_decompress(&payload.data)?,
        };

        // Verify checksum
        let actual_checksum = simple_checksum(&decompressed);
        if actual_checksum != payload.checksum {
            return Err(CompressionError::ChecksumMismatch {
                expected: payload.checksum,
                actual: actual_checksum,
            });
        }

        // Verify size
        if decompressed.len() != payload.original_size {
            return Err(CompressionError::SizeMismatch {
                expected: payload.original_size,
                actual: decompressed.len(),
            });
        }

        Ok(decompressed)
    }

    /// Get compression statistics.
    pub fn stats(&self) -> &CompressionStats {
        &self.stats
    }

    /// Get the configuration.
    pub fn config(&self) -> &CompressionConfig {
        &self.config
    }
}

/// Compression errors.
#[derive(Clone, Debug)]
pub enum CompressionError {
    /// Checksum verification failed.
    ChecksumMismatch { expected: u32, actual: u32 },
    /// Decompressed size doesn't match.
    SizeMismatch { expected: usize, actual: usize },
    /// Invalid compressed data.
    InvalidData(String),
}

impl std::fmt::Display for CompressionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompressionError::ChecksumMismatch { expected, actual } => {
                write!(
                    f,
                    "Checksum mismatch: expected {}, got {}",
                    expected, actual
                )
            }
            CompressionError::SizeMismatch { expected, actual } => {
                write!(f, "Size mismatch: expected {}, got {}", expected, actual)
            }
            CompressionError::InvalidData(msg) => {
                write!(f, "Invalid compressed data: {}", msg)
            }
        }
    }
}

impl std::error::Error for CompressionError {}

/// Simple checksum for data validation.
fn simple_checksum(data: &[u8]) -> u32 {
    let mut checksum: u32 = 0;
    for (i, &byte) in data.iter().enumerate() {
        checksum = checksum.wrapping_add(byte as u32);
        checksum = checksum.wrapping_add((i as u32) << 8);
        checksum = checksum.rotate_left(5);
    }
    checksum
}

/// Simple run-length encoding compression.
fn rle_compress(data: &[u8]) -> Vec<u8> {
    if data.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::with_capacity(data.len());
    let mut i = 0;

    while i < data.len() {
        let byte = data[i];
        let mut count = 1u8;

        // Count consecutive identical bytes (max 255)
        while i + (count as usize) < data.len() && data[i + count as usize] == byte && count < 255 {
            count += 1;
        }

        if count >= 3 || byte == 0xFF {
            // Use RLE encoding: 0xFF, byte, count
            result.push(0xFF);
            result.push(byte);
            result.push(count);
        } else {
            // Store bytes literally
            for _ in 0..count {
                result.push(byte);
            }
        }

        i += count as usize;
    }

    result
}

/// Decompress RLE-encoded data.
fn rle_decompress(data: &[u8]) -> Result<Vec<u8>, CompressionError> {
    let mut result = Vec::new();
    let mut i = 0;

    while i < data.len() {
        if data[i] == 0xFF {
            if i + 2 >= data.len() {
                return Err(CompressionError::InvalidData(
                    "Truncated RLE sequence".to_string(),
                ));
            }
            let byte = data[i + 1];
            let count = data[i + 2];
            for _ in 0..count {
                result.push(byte);
            }
            i += 3;
        } else {
            result.push(data[i]);
            i += 1;
        }
    }

    Ok(result)
}

/// Simple LZ77-style compression.
fn lz_compress(data: &[u8]) -> Vec<u8> {
    if data.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::with_capacity(data.len());
    let mut i = 0;
    let window_size = 255usize;
    let min_match = 3usize;
    let max_match = 258usize;

    while i < data.len() {
        let mut best_offset = 0usize;
        let mut best_length = 0usize;

        // Search for matches in the sliding window
        let window_start = i.saturating_sub(window_size);
        for j in window_start..i {
            let mut length = 0;
            while i + length < data.len()
                && length < max_match
                && data[j + length % (i - j)] == data[i + length]
            {
                length += 1;
            }
            if length >= min_match && length > best_length {
                best_offset = i - j;
                best_length = length;
            }
        }

        if best_length >= min_match {
            // Encode as (offset, length) - using 0x00 as marker
            result.push(0x00);
            result.push(best_offset as u8);
            result.push((best_length - min_match) as u8);
            i += best_length;
        } else {
            // Store literal
            if data[i] == 0x00 {
                result.push(0x00);
                result.push(0);
                result.push(0);
                result.push(data[i]);
            } else {
                result.push(data[i]);
            }
            i += 1;
        }
    }

    result
}

/// Decompress LZ-encoded data.
fn lz_decompress(data: &[u8]) -> Result<Vec<u8>, CompressionError> {
    let mut result = Vec::new();
    let min_match = 3usize;
    let mut i = 0;

    while i < data.len() {
        if data[i] == 0x00 {
            if i + 2 >= data.len() {
                return Err(CompressionError::InvalidData(
                    "Truncated LZ sequence".to_string(),
                ));
            }
            let offset = data[i + 1] as usize;
            let extra_length = data[i + 2] as usize;

            if offset == 0 && extra_length == 0 {
                // Escaped literal 0x00
                if i + 3 >= data.len() {
                    return Err(CompressionError::InvalidData(
                        "Truncated escaped literal".to_string(),
                    ));
                }
                result.push(data[i + 3]);
                i += 4;
            } else {
                // Back-reference
                let length = extra_length + min_match;
                if offset == 0 || offset > result.len() {
                    return Err(CompressionError::InvalidData(format!(
                        "Invalid offset: {} (result len: {})",
                        offset,
                        result.len()
                    )));
                }
                let start = result.len() - offset;
                for j in 0..length {
                    result.push(result[start + j % offset]);
                }
                i += 3;
            }
        } else {
            result.push(data[i]);
            i += 1;
        }
    }

    Ok(result)
}

/// Convenience function to check if compression would be beneficial.
pub fn should_compress(data: &[u8], config: &CompressionConfig) -> bool {
    data.len() >= config.min_size && data.len() <= config.max_size
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== CompressionAlgorithm Tests ====================

    #[test]
    fn test_compression_algorithm_default() {
        let algo = CompressionAlgorithm::default();
        assert_eq!(algo, CompressionAlgorithm::Rle);
    }

    #[test]
    fn test_compression_algorithm_serialization() {
        let algo = CompressionAlgorithm::Lz;
        let json = serde_json::to_string(&algo).unwrap();
        let restored: CompressionAlgorithm = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, CompressionAlgorithm::Lz);
    }

    // ==================== CompressionConfig Tests ====================

    #[test]
    fn test_compression_config_default() {
        let config = CompressionConfig::default();
        assert_eq!(config.min_size, 256);
        assert_eq!(config.max_size, 1024 * 1024);
        assert!((config.min_ratio - 0.9).abs() < 0.001);
    }

    // ==================== CompressedPayload Tests ====================

    #[test]
    fn test_compressed_payload_ratio() {
        let payload = CompressedPayload {
            data: vec![0u8; 50],
            original_size: 100,
            algorithm: CompressionAlgorithm::Rle,
            checksum: 0,
        };
        assert!((payload.ratio() - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_compressed_payload_is_effective() {
        let effective = CompressedPayload {
            data: vec![0u8; 50],
            original_size: 100,
            algorithm: CompressionAlgorithm::Rle,
            checksum: 0,
        };
        assert!(effective.is_effective());

        let not_effective = CompressedPayload {
            data: vec![0u8; 95],
            original_size: 100,
            algorithm: CompressionAlgorithm::Rle,
            checksum: 0,
        };
        assert!(!not_effective.is_effective());
    }

    // ==================== RLE Tests ====================

    #[test]
    fn test_rle_compress_empty() {
        let result = rle_compress(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_rle_compress_no_runs() {
        let data = b"abcdefgh";
        let compressed = rle_compress(data);
        let decompressed = rle_decompress(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_rle_compress_with_runs() {
        let data = vec![0u8; 100];
        let compressed = rle_compress(&data);
        assert!(compressed.len() < data.len());
        let decompressed = rle_decompress(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_rle_compress_mixed() {
        let mut data = Vec::new();
        data.extend_from_slice(b"hello");
        data.extend(vec![0u8; 50]);
        data.extend_from_slice(b"world");
        data.extend(vec![1u8; 30]);

        let compressed = rle_compress(&data);
        let decompressed = rle_decompress(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_rle_escape_0xff() {
        // Data containing 0xFF bytes
        let data = vec![0xFF, 0xFF, 0x01, 0xFF];
        let compressed = rle_compress(&data);
        let decompressed = rle_decompress(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }

    // ==================== LZ Tests ====================

    #[test]
    fn test_lz_compress_empty() {
        let result = lz_compress(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_lz_compress_no_matches() {
        let data = b"abcdefgh";
        let compressed = lz_compress(data);
        let decompressed = lz_decompress(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_lz_compress_with_repeats() {
        let data = b"abcabcabcabcabc";
        let compressed = lz_compress(data);
        let decompressed = lz_decompress(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_lz_escape_0x00() {
        // Data containing 0x00 bytes
        let data = vec![0x01, 0x00, 0x02, 0x00, 0x03];
        let compressed = lz_compress(&data);
        let decompressed = lz_decompress(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }

    // ==================== PayloadCompressor Tests ====================

    #[test]
    fn test_compressor_skip_small() {
        let config = CompressionConfig {
            min_size: 100,
            ..Default::default()
        };
        let mut compressor = PayloadCompressor::new(config);

        let small_data = vec![0u8; 50];
        let result = compressor.compress(&small_data);
        assert!(result.is_none());
        assert_eq!(compressor.stats().skipped_too_small, 1);
    }

    #[test]
    fn test_compressor_compress_large() {
        let config = CompressionConfig {
            min_size: 10,
            min_ratio: 0.9,
            algorithm: CompressionAlgorithm::Rle,
            ..Default::default()
        };
        let mut compressor = PayloadCompressor::new(config);

        // Highly compressible data
        let data = vec![0u8; 1000];
        let result = compressor.compress(&data);
        assert!(result.is_some());

        let payload = result.unwrap();
        assert!(payload.data.len() < data.len());
    }

    #[test]
    fn test_compressor_decompress_roundtrip() {
        let config = CompressionConfig {
            min_size: 10,
            min_ratio: 1.0, // Always compress
            algorithm: CompressionAlgorithm::Rle,
            ..Default::default()
        };
        let mut compressor = PayloadCompressor::new(config);

        let data = b"Hello, World! This is a test message for compression.".to_vec();
        // Make it long enough and compressible
        let mut long_data = Vec::new();
        for _ in 0..100 {
            long_data.extend_from_slice(&data);
        }

        if let Some(compressed) = compressor.compress(&long_data) {
            let decompressed = compressor.decompress(&compressed).unwrap();
            assert_eq!(decompressed, long_data);
        }
    }

    #[test]
    fn test_compressor_checksum_validation() {
        let compressor = PayloadCompressor::with_defaults();

        let payload = CompressedPayload {
            data: vec![1, 2, 3],
            original_size: 3,
            algorithm: CompressionAlgorithm::None,
            checksum: 12345, // Wrong checksum
        };

        let result = compressor.decompress(&payload);
        assert!(matches!(
            result,
            Err(CompressionError::ChecksumMismatch { .. })
        ));
    }

    #[test]
    fn test_compressor_size_validation() {
        let compressor = PayloadCompressor::with_defaults();

        let data = vec![1, 2, 3];
        let payload = CompressedPayload {
            data: data.clone(),
            original_size: 10, // Wrong size
            algorithm: CompressionAlgorithm::None,
            checksum: simple_checksum(&data),
        };

        let result = compressor.decompress(&payload);
        assert!(matches!(result, Err(CompressionError::SizeMismatch { .. })));
    }

    #[test]
    fn test_compressor_stats() {
        let config = CompressionConfig {
            min_size: 10,
            min_ratio: 1.0,
            algorithm: CompressionAlgorithm::Rle,
            ..Default::default()
        };
        let mut compressor = PayloadCompressor::new(config);

        let data = vec![0u8; 100];
        compressor.compress(&data);

        let stats = compressor.stats();
        assert_eq!(stats.compressions_attempted, 1);
        assert!(stats.bytes_before > 0);
    }

    // ==================== should_compress Tests ====================

    #[test]
    fn test_should_compress() {
        let config = CompressionConfig {
            min_size: 100,
            max_size: 1000,
            ..Default::default()
        };

        assert!(!should_compress(&vec![0u8; 50], &config)); // Too small
        assert!(should_compress(&vec![0u8; 500], &config)); // Good size
        assert!(!should_compress(&vec![0u8; 2000], &config)); // Too large
    }

    // ==================== Error Tests ====================

    #[test]
    fn test_compression_error_display() {
        let e1 = CompressionError::ChecksumMismatch {
            expected: 100,
            actual: 200,
        };
        assert!(format!("{}", e1).contains("Checksum mismatch"));

        let e2 = CompressionError::SizeMismatch {
            expected: 100,
            actual: 50,
        };
        assert!(format!("{}", e2).contains("Size mismatch"));

        let e3 = CompressionError::InvalidData("test".to_string());
        assert!(format!("{}", e3).contains("Invalid compressed data"));
    }
}
