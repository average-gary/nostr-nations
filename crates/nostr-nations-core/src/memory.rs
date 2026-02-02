//! Memory optimization utilities for reducing allocations and memory footprint.
//!
//! This module provides:
//! - Object pools for reusing allocations
//! - Arena allocator for temporary bump allocations
//! - String interning for deduplicating repeated strings
//! - Memory statistics tracking
//! - Compact data types for space efficiency

use std::collections::HashMap;
use std::mem;
use std::ptr;

// ============================================================================
// Object Pool
// ============================================================================

/// A reusable object pool to reduce allocations.
///
/// Objects are borrowed from the pool and returned when no longer needed,
/// avoiding repeated heap allocations for frequently created/destroyed objects.
///
/// # Example
/// ```
/// use nostr_nations_core::memory::ObjectPool;
///
/// let mut pool: ObjectPool<Vec<u8>> = ObjectPool::new(|| Vec::with_capacity(1024), 4, 16);
/// let mut buf = pool.acquire();
/// buf.push(42);
/// pool.release(buf);
/// ```
pub struct ObjectPool<T> {
    available: Vec<T>,
    factory: Box<dyn Fn() -> T>,
    max_size: usize,
}

impl<T> ObjectPool<T> {
    /// Create a new object pool.
    ///
    /// # Arguments
    /// * `factory` - Function to create new objects when pool is empty
    /// * `initial_size` - Number of objects to pre-allocate
    /// * `max_size` - Maximum objects to keep in the pool
    pub fn new(factory: impl Fn() -> T + 'static, initial_size: usize, max_size: usize) -> Self {
        let factory = Box::new(factory);
        let mut available = Vec::with_capacity(max_size);

        for _ in 0..initial_size.min(max_size) {
            available.push(factory());
        }

        Self {
            available,
            factory,
            max_size,
        }
    }

    /// Acquire an object from the pool.
    ///
    /// Returns an existing object if available, otherwise creates a new one.
    pub fn acquire(&mut self) -> T {
        self.available.pop().unwrap_or_else(|| (self.factory)())
    }

    /// Release an object back to the pool.
    ///
    /// If the pool is at capacity, the object is dropped.
    pub fn release(&mut self, item: T) {
        if self.available.len() < self.max_size {
            self.available.push(item);
        }
        // Otherwise item is dropped
    }

    /// Get the number of available objects in the pool.
    pub fn available_count(&self) -> usize {
        self.available.len()
    }

    /// Clear all objects from the pool.
    pub fn clear(&mut self) {
        self.available.clear();
    }

    /// Get the maximum pool size.
    pub fn max_size(&self) -> usize {
        self.max_size
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for ObjectPool<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ObjectPool")
            .field("available", &self.available.len())
            .field("max_size", &self.max_size)
            .finish()
    }
}

// ============================================================================
// Arena Allocator
// ============================================================================

/// A bump allocator for temporary allocations.
///
/// Allocations are made by bumping a pointer forward in a chunk of memory.
/// All allocations can be freed at once by resetting the arena, which is
/// much faster than individual deallocations.
///
/// # Safety
/// References returned by `alloc` and `alloc_slice` are valid until `reset`
/// is called. The caller must ensure no references outlive a reset.
///
/// # Example
/// ```
/// use nostr_nations_core::memory::Arena;
///
/// let mut arena = Arena::new(4096);
/// {
///     let x = arena.alloc(42u32);
///     assert_eq!(*x, 42);
/// }
/// {
///     let slice = arena.alloc_slice(&[1, 2, 3, 4]);
///     assert_eq!(slice, &[1, 2, 3, 4]);
/// }
/// arena.reset();
/// ```
pub struct Arena {
    chunks: Vec<Vec<u8>>,
    current_chunk: usize,
    offset: usize,
    chunk_size: usize,
}

impl Arena {
    /// Create a new arena with the given chunk size.
    pub fn new(chunk_size: usize) -> Self {
        let chunk_size = chunk_size.max(64); // Minimum chunk size
        Self {
            chunks: vec![vec![0u8; chunk_size]],
            current_chunk: 0,
            offset: 0,
            chunk_size,
        }
    }

    /// Allocate a value in the arena.
    ///
    /// Returns a mutable reference to the allocated value.
    ///
    /// # Panics
    /// Panics if the value size exceeds the chunk size.
    pub fn alloc<T>(&mut self, value: T) -> &mut T {
        let size = mem::size_of::<T>();
        let align = mem::align_of::<T>();

        assert!(
            size <= self.chunk_size,
            "Value size {} exceeds chunk size {}",
            size,
            self.chunk_size
        );

        let ptr = self.alloc_raw(size, align);

        // Safety: ptr is properly aligned and has enough space
        unsafe {
            let typed_ptr = ptr as *mut T;
            ptr::write(typed_ptr, value);
            &mut *typed_ptr
        }
    }

    /// Allocate a slice in the arena by cloning values.
    ///
    /// Returns a mutable reference to the allocated slice.
    pub fn alloc_slice<T: Clone>(&mut self, values: &[T]) -> &mut [T] {
        if values.is_empty() {
            return &mut [];
        }

        let size = std::mem::size_of_val(values);
        let align = mem::align_of::<T>();

        assert!(
            size <= self.chunk_size,
            "Slice size {} exceeds chunk size {}",
            size,
            self.chunk_size
        );

        let ptr = self.alloc_raw(size, align);

        // Safety: ptr is properly aligned and has enough space
        unsafe {
            let typed_ptr = ptr as *mut T;
            for (i, v) in values.iter().enumerate() {
                ptr::write(typed_ptr.add(i), v.clone());
            }
            std::slice::from_raw_parts_mut(typed_ptr, values.len())
        }
    }

    /// Allocate raw bytes with the given alignment.
    fn alloc_raw(&mut self, size: usize, align: usize) -> *mut u8 {
        // Align the offset
        let aligned_offset = (self.offset + align - 1) & !(align - 1);

        // Check if we have space in current chunk
        if aligned_offset + size <= self.chunk_size {
            let chunk = &mut self.chunks[self.current_chunk];
            let ptr = chunk.as_mut_ptr();
            self.offset = aligned_offset + size;
            // Safety: aligned_offset is within bounds
            unsafe { ptr.add(aligned_offset) }
        } else {
            // Need a new chunk
            self.current_chunk += 1;

            if self.current_chunk >= self.chunks.len() {
                self.chunks.push(vec![0u8; self.chunk_size]);
            }

            // Align within new chunk (starts at 0)
            let aligned_offset = (align - 1) & !(align - 1);
            let chunk = &mut self.chunks[self.current_chunk];
            let ptr = chunk.as_mut_ptr();
            self.offset = aligned_offset + size;
            // Safety: aligned_offset is 0 or a small alignment value
            unsafe { ptr.add(aligned_offset) }
        }
    }

    /// Reset the arena for reuse without deallocating memory.
    ///
    /// # Safety
    /// All references returned by previous `alloc` calls become invalid.
    /// The caller must ensure no references to arena-allocated data are used
    /// after calling reset.
    pub fn reset(&mut self) {
        self.current_chunk = 0;
        self.offset = 0;

        // Zero out chunks to avoid undefined behavior if types have drop
        // Note: We don't run destructors - this is intentional for POD types
        for chunk in &mut self.chunks {
            chunk.fill(0);
        }
    }

    /// Get the total memory used by the arena.
    pub fn memory_used(&self) -> usize {
        if self.current_chunk == 0 {
            self.offset
        } else {
            self.current_chunk * self.chunk_size + self.offset
        }
    }

    /// Get the total memory allocated (including unused space).
    pub fn memory_allocated(&self) -> usize {
        self.chunks.len() * self.chunk_size
    }

    /// Get the chunk size.
    pub fn chunk_size(&self) -> usize {
        self.chunk_size
    }
}

impl std::fmt::Debug for Arena {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Arena")
            .field("chunks", &self.chunks.len())
            .field("chunk_size", &self.chunk_size)
            .field("memory_used", &self.memory_used())
            .finish()
    }
}

// ============================================================================
// String Interning
// ============================================================================

/// A handle to an interned string.
///
/// This is a lightweight copy type that can be used instead of `String`
/// when the same strings are used repeatedly.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct InternedString(usize);

impl InternedString {
    /// Get the index of this interned string.
    pub fn index(&self) -> usize {
        self.0
    }
}

/// A pool for string interning.
///
/// Interning ensures that identical strings share the same storage,
/// reducing memory usage when the same strings appear many times.
///
/// # Example
/// ```
/// use nostr_nations_core::memory::InternPool;
///
/// let mut pool = InternPool::new();
/// let a = pool.intern("hello");
/// let b = pool.intern("hello");
/// assert_eq!(a, b); // Same interned handle
/// assert_eq!(pool.get(a), "hello");
/// ```
pub struct InternPool {
    strings: HashMap<String, usize>,
    indexed: Vec<String>,
}

impl InternPool {
    /// Create a new intern pool.
    pub fn new() -> Self {
        Self {
            strings: HashMap::new(),
            indexed: Vec::new(),
        }
    }

    /// Create an intern pool with pre-allocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            strings: HashMap::with_capacity(capacity),
            indexed: Vec::with_capacity(capacity),
        }
    }

    /// Intern a string, returning a handle.
    ///
    /// If the string is already interned, returns the existing handle.
    pub fn intern(&mut self, s: &str) -> InternedString {
        if let Some(&idx) = self.strings.get(s) {
            return InternedString(idx);
        }

        let idx = self.indexed.len();
        let owned = s.to_string();
        self.strings.insert(owned.clone(), idx);
        self.indexed.push(owned);
        InternedString(idx)
    }

    /// Get the string for an interned handle.
    ///
    /// # Panics
    /// Panics if the handle is invalid (from a different pool).
    pub fn get(&self, interned: InternedString) -> &str {
        &self.indexed[interned.0]
    }

    /// Try to get the string for an interned handle.
    pub fn try_get(&self, interned: InternedString) -> Option<&str> {
        self.indexed.get(interned.0).map(|s| s.as_str())
    }

    /// Get the number of interned strings.
    pub fn len(&self) -> usize {
        self.indexed.len()
    }

    /// Check if the pool is empty.
    pub fn is_empty(&self) -> bool {
        self.indexed.is_empty()
    }

    /// Clear the pool.
    pub fn clear(&mut self) {
        self.strings.clear();
        self.indexed.clear();
    }

    /// Get total memory used by interned strings.
    pub fn memory_used(&self) -> usize {
        self.indexed.iter().map(|s| s.len()).sum::<usize>()
            + self.indexed.capacity() * mem::size_of::<String>()
            + self.strings.capacity()
                * (mem::size_of::<String>() + mem::size_of::<usize>() + mem::size_of::<u64>())
    }
}

impl Default for InternPool {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for InternPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InternPool")
            .field("count", &self.indexed.len())
            .field("memory_used", &self.memory_used())
            .finish()
    }
}

// ============================================================================
// Memory Statistics
// ============================================================================

/// Statistics about memory usage in a game.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MemoryStats {
    /// Bytes used by game state metadata.
    pub game_state_bytes: usize,
    /// Bytes used by the map.
    pub map_bytes: usize,
    /// Bytes used by units.
    pub units_bytes: usize,
    /// Bytes used by cities.
    pub cities_bytes: usize,
    /// Bytes used by caches and other data.
    pub cache_bytes: usize,
}

impl MemoryStats {
    /// Create new memory stats with all zeros.
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate memory stats from a game state.
    ///
    /// This provides an estimate of memory usage based on struct sizes
    /// and collection lengths.
    pub fn calculate(game: &crate::game_state::GameState) -> Self {
        use std::mem::size_of;

        // Base game state (excluding collections)
        let game_state_bytes = size_of::<crate::game_state::GameState>();

        // Map memory
        let tile_size = size_of::<crate::map::Tile>();
        let map_bytes = size_of::<crate::map::Map>()
            + game.map.tiles.len() * (tile_size + size_of::<crate::hex::HexCoord>() + 8);

        // Units memory
        let unit_size = size_of::<crate::unit::Unit>();
        let units_bytes = game.units.len() * (unit_size + size_of::<crate::types::UnitId>() + 8);

        // Cities memory
        let city_size = size_of::<crate::city::City>();
        let cities_bytes = game.cities.len() * (city_size + size_of::<crate::types::CityId>() + 8)
            + game
                .cities
                .values()
                .map(|c| {
                    c.territory.len() * size_of::<crate::hex::HexCoord>()
                        + c.worked_tiles.len() * size_of::<crate::hex::HexCoord>()
                        + c.buildings.len() * size_of::<crate::city::BuildingType>()
                })
                .sum::<usize>();

        // Caches and other data
        let cache_bytes = game.event_chain.len() * size_of::<crate::types::EventId>()
            + game.players.len() * size_of::<crate::player::Player>();

        Self {
            game_state_bytes,
            map_bytes,
            units_bytes,
            cities_bytes,
            cache_bytes,
        }
    }

    /// Get total memory usage.
    pub fn total(&self) -> usize {
        self.game_state_bytes
            + self.map_bytes
            + self.units_bytes
            + self.cities_bytes
            + self.cache_bytes
    }

    /// Format memory stats as human-readable string.
    pub fn format_human_readable(&self) -> String {
        format!(
            "Memory Usage:\n  Game State: {}\n  Map: {}\n  Units: {}\n  Cities: {}\n  Cache: {}\n  Total: {}",
            format_bytes(self.game_state_bytes),
            format_bytes(self.map_bytes),
            format_bytes(self.units_bytes),
            format_bytes(self.cities_bytes),
            format_bytes(self.cache_bytes),
            format_bytes(self.total()),
        )
    }
}

/// Format bytes as human-readable string (KB, MB, etc.).
fn format_bytes(bytes: usize) -> String {
    const KB: usize = 1024;
    const MB: usize = KB * 1024;
    const GB: usize = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

// ============================================================================
// Compact Types
// ============================================================================

/// A packed coordinate using 2 bytes instead of 8.
///
/// Stores q and r as i8 values, supporting coordinates from -128 to 127.
/// This is sufficient for maps up to 256x256.
///
/// # Example
/// ```
/// use nostr_nations_core::memory::PackedCoord;
///
/// let packed = PackedCoord::new(10, -5);
/// let (q, r) = packed.unpack();
/// assert_eq!(q, 10);
/// assert_eq!(r, -5);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PackedCoord(u16);

impl PackedCoord {
    /// Create a new packed coordinate.
    ///
    /// # Panics
    /// Panics if q or r are outside the range -128..=127.
    pub fn new(q: i8, r: i8) -> Self {
        // Pack q in high byte, r in low byte
        let q_bits = (q as u8) as u16;
        let r_bits = (r as u8) as u16;
        Self((q_bits << 8) | r_bits)
    }

    /// Create a packed coordinate from i32 values.
    ///
    /// # Panics
    /// Panics if values are outside i8 range.
    pub fn from_i32(q: i32, r: i32) -> Self {
        assert!(
            q >= i8::MIN as i32 && q <= i8::MAX as i32,
            "q value {} out of i8 range",
            q
        );
        assert!(
            r >= i8::MIN as i32 && r <= i8::MAX as i32,
            "r value {} out of i8 range",
            r
        );
        Self::new(q as i8, r as i8)
    }

    /// Try to create a packed coordinate from i32 values.
    pub fn try_from_i32(q: i32, r: i32) -> Option<Self> {
        if q >= i8::MIN as i32 && q <= i8::MAX as i32 && r >= i8::MIN as i32 && r <= i8::MAX as i32
        {
            Some(Self::new(q as i8, r as i8))
        } else {
            None
        }
    }

    /// Unpack the coordinate into q and r values.
    pub fn unpack(&self) -> (i8, i8) {
        let q = (self.0 >> 8) as u8 as i8;
        let r = (self.0 & 0xFF) as u8 as i8;
        (q, r)
    }

    /// Get the q (column) coordinate.
    pub fn q(&self) -> i8 {
        (self.0 >> 8) as u8 as i8
    }

    /// Get the r (row) coordinate.
    pub fn r(&self) -> i8 {
        (self.0 & 0xFF) as u8 as i8
    }

    /// Convert to a full HexCoord.
    pub fn to_hex_coord(&self) -> crate::hex::HexCoord {
        let (q, r) = self.unpack();
        crate::hex::HexCoord::new(q as i32, r as i32)
    }

    /// Create from a HexCoord.
    ///
    /// # Panics
    /// Panics if the HexCoord values are outside i8 range.
    pub fn from_hex_coord(coord: &crate::hex::HexCoord) -> Self {
        Self::from_i32(coord.q, coord.r)
    }

    /// Try to create from a HexCoord.
    pub fn try_from_hex_coord(coord: &crate::hex::HexCoord) -> Option<Self> {
        Self::try_from_i32(coord.q, coord.r)
    }

    /// Get the raw packed value.
    pub fn raw(&self) -> u16 {
        self.0
    }
}

impl From<PackedCoord> for crate::hex::HexCoord {
    fn from(packed: PackedCoord) -> Self {
        packed.to_hex_coord()
    }
}

impl std::fmt::Display for PackedCoord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (q, r) = self.unpack();
        write!(f, "({}, {})", q, r)
    }
}

// ============================================================================
// Additional Compact Types
// ============================================================================

/// A compact player ID using a single byte.
///
/// Supports up to 256 players, which is more than enough for any game.
pub type CompactPlayerId = u8;

/// A compact unit ID using 2 bytes.
///
/// Supports up to 65536 units, sufficient for most games.
pub type CompactUnitId = u16;

/// A compact city ID using 2 bytes.
///
/// Supports up to 65536 cities, sufficient for most games.
pub type CompactCityId = u16;

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== ObjectPool Tests ====================

    #[test]
    fn test_object_pool_creation() {
        let pool: ObjectPool<Vec<u8>> = ObjectPool::new(|| Vec::with_capacity(100), 5, 10);
        assert_eq!(pool.available_count(), 5);
        assert_eq!(pool.max_size(), 10);
    }

    #[test]
    fn test_object_pool_acquire_release() {
        let mut pool: ObjectPool<Vec<u8>> = ObjectPool::new(Vec::new, 2, 4);
        assert_eq!(pool.available_count(), 2);

        let obj1 = pool.acquire();
        assert_eq!(pool.available_count(), 1);

        let obj2 = pool.acquire();
        assert_eq!(pool.available_count(), 0);

        // Acquire when empty - creates new
        let _obj3 = pool.acquire();
        assert_eq!(pool.available_count(), 0);

        // Release objects
        pool.release(obj1);
        assert_eq!(pool.available_count(), 1);

        pool.release(obj2);
        assert_eq!(pool.available_count(), 2);
    }

    #[test]
    fn test_object_pool_max_size() {
        let mut pool: ObjectPool<u32> = ObjectPool::new(|| 0, 0, 2);

        pool.release(1);
        pool.release(2);
        pool.release(3); // Should be dropped (exceeds max)

        assert_eq!(pool.available_count(), 2);
    }

    #[test]
    fn test_object_pool_clear() {
        let mut pool: ObjectPool<u32> = ObjectPool::new(|| 0, 5, 10);
        assert_eq!(pool.available_count(), 5);

        pool.clear();
        assert_eq!(pool.available_count(), 0);
    }

    // ==================== Arena Tests ====================

    #[test]
    fn test_arena_creation() {
        let arena = Arena::new(1024);
        assert_eq!(arena.chunk_size(), 1024);
        assert_eq!(arena.memory_used(), 0);
    }

    #[test]
    fn test_arena_alloc_primitives() {
        let mut arena = Arena::new(1024);

        // Test u32 allocation
        {
            let x = arena.alloc(42u32);
            assert_eq!(*x, 42);
            *x = 100;
            assert_eq!(*x, 100);
        }

        arena.reset();

        // Test f64 allocation
        {
            let y = arena.alloc(std::f64::consts::PI);
            assert!((*y - std::f64::consts::PI).abs() < 0.001);
        }
    }

    #[test]
    fn test_arena_alloc_slice() {
        let mut arena = Arena::new(1024);

        let slice = arena.alloc_slice(&[1, 2, 3, 4, 5]);
        assert_eq!(slice, &[1, 2, 3, 4, 5]);

        slice[0] = 10;
        assert_eq!(slice[0], 10);
    }

    #[test]
    fn test_arena_alloc_empty_slice() {
        let mut arena = Arena::new(1024);
        let slice: &mut [u32] = arena.alloc_slice(&[]);
        assert!(slice.is_empty());
    }

    #[test]
    fn test_arena_reset() {
        let mut arena = Arena::new(1024);

        let _ = arena.alloc(42u32);
        let _ = arena.alloc(100u64);
        assert!(arena.memory_used() > 0);

        arena.reset();
        assert_eq!(arena.memory_used(), 0);
    }

    #[test]
    fn test_arena_multiple_chunks() {
        let mut arena = Arena::new(64); // Small chunks

        // Allocate more than one chunk's worth
        for i in 0..20 {
            let _ = arena.alloc(i as u64);
        }

        assert!(arena.memory_used() > 64);
    }

    // ==================== InternPool Tests ====================

    #[test]
    fn test_intern_pool_creation() {
        let pool = InternPool::new();
        assert_eq!(pool.len(), 0);
        assert!(pool.is_empty());
    }

    #[test]
    fn test_intern_pool_intern() {
        let mut pool = InternPool::new();

        let a = pool.intern("hello");
        let b = pool.intern("world");
        let c = pool.intern("hello"); // Same as a

        assert_eq!(a, c);
        assert_ne!(a, b);
        assert_eq!(pool.len(), 2);
    }

    #[test]
    fn test_intern_pool_get() {
        let mut pool = InternPool::new();

        let handle = pool.intern("test string");
        assert_eq!(pool.get(handle), "test string");
    }

    #[test]
    fn test_intern_pool_try_get() {
        let mut pool = InternPool::new();
        let handle = pool.intern("valid");

        assert_eq!(pool.try_get(handle), Some("valid"));
        assert_eq!(pool.try_get(InternedString(999)), None);
    }

    #[test]
    fn test_intern_pool_clear() {
        let mut pool = InternPool::new();
        pool.intern("a");
        pool.intern("b");
        assert_eq!(pool.len(), 2);

        pool.clear();
        assert_eq!(pool.len(), 0);
        assert!(pool.is_empty());
    }

    #[test]
    fn test_interned_string_index() {
        let mut pool = InternPool::new();
        let handle = pool.intern("test");
        assert_eq!(handle.index(), 0);

        let handle2 = pool.intern("test2");
        assert_eq!(handle2.index(), 1);
    }

    // ==================== MemoryStats Tests ====================

    #[test]
    fn test_memory_stats_new() {
        let stats = MemoryStats::new();
        assert_eq!(stats.total(), 0);
    }

    #[test]
    fn test_memory_stats_total() {
        let stats = MemoryStats {
            game_state_bytes: 100,
            map_bytes: 200,
            units_bytes: 50,
            cities_bytes: 75,
            cache_bytes: 25,
        };
        assert_eq!(stats.total(), 450);
    }

    #[test]
    fn test_memory_stats_format() {
        let stats = MemoryStats {
            game_state_bytes: 1024,
            map_bytes: 2048,
            units_bytes: 512,
            cities_bytes: 256,
            cache_bytes: 128,
        };

        let formatted = stats.format_human_readable();
        assert!(formatted.contains("Memory Usage:"));
        assert!(formatted.contains("Game State:"));
        assert!(formatted.contains("Map:"));
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1536), "1.50 KB");
        assert_eq!(format_bytes(1048576), "1.00 MB");
        assert_eq!(format_bytes(1073741824), "1.00 GB");
    }

    // ==================== PackedCoord Tests ====================

    #[test]
    fn test_packed_coord_creation() {
        let coord = PackedCoord::new(10, -5);
        let (q, r) = coord.unpack();
        assert_eq!(q, 10);
        assert_eq!(r, -5);
    }

    #[test]
    fn test_packed_coord_accessors() {
        let coord = PackedCoord::new(50, -30);
        assert_eq!(coord.q(), 50);
        assert_eq!(coord.r(), -30);
    }

    #[test]
    fn test_packed_coord_from_i32() {
        let coord = PackedCoord::from_i32(100, -100);
        assert_eq!(coord.q(), 100);
        assert_eq!(coord.r(), -100);
    }

    #[test]
    fn test_packed_coord_try_from_i32() {
        assert!(PackedCoord::try_from_i32(50, 50).is_some());
        assert!(PackedCoord::try_from_i32(200, 50).is_none()); // Out of range
        assert!(PackedCoord::try_from_i32(50, -200).is_none()); // Out of range
    }

    #[test]
    fn test_packed_coord_extreme_values() {
        let coord = PackedCoord::new(127, -128);
        assert_eq!(coord.q(), 127);
        assert_eq!(coord.r(), -128);
    }

    #[test]
    fn test_packed_coord_to_hex_coord() {
        let packed = PackedCoord::new(15, 20);
        let hex = packed.to_hex_coord();
        assert_eq!(hex.q, 15);
        assert_eq!(hex.r, 20);
    }

    #[test]
    fn test_packed_coord_from_hex_coord() {
        use crate::hex::HexCoord;
        let hex = HexCoord::new(25, -10);
        let packed = PackedCoord::from_hex_coord(&hex);
        assert_eq!(packed.q(), 25);
        assert_eq!(packed.r(), -10);
    }

    #[test]
    fn test_packed_coord_try_from_hex_coord() {
        use crate::hex::HexCoord;

        let hex = HexCoord::new(50, 50);
        assert!(PackedCoord::try_from_hex_coord(&hex).is_some());

        let hex_large = HexCoord::new(1000, 50);
        assert!(PackedCoord::try_from_hex_coord(&hex_large).is_none());
    }

    #[test]
    fn test_packed_coord_display() {
        let coord = PackedCoord::new(5, -3);
        assert_eq!(format!("{}", coord), "(5, -3)");
    }

    #[test]
    fn test_packed_coord_equality() {
        let a = PackedCoord::new(10, 20);
        let b = PackedCoord::new(10, 20);
        let c = PackedCoord::new(10, 21);

        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_packed_coord_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(PackedCoord::new(1, 2));
        set.insert(PackedCoord::new(3, 4));
        set.insert(PackedCoord::new(1, 2)); // Duplicate

        assert_eq!(set.len(), 2);
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_memory_stats_calculate() {
        use crate::game_state::GameState;
        use crate::settings::GameSettings;

        let settings = GameSettings::new("Test".to_string());
        let game = GameState::new("test_game".to_string(), settings, [0u8; 32]);

        let stats = MemoryStats::calculate(&game);
        assert!(stats.game_state_bytes > 0);
        // Map bytes depends on map size - could be 0 if no tiles
        assert!(stats.total() > 0);
    }
}
