# lau-blueprint

> Blueprint/building plan system for kids to design structures

## What This Does

Blueprint/building plan system for kids to design structures. Part of the PLATO/LAU ecosystem — a mathematically rigorous framework for building educational agents that learn, teach, and evolve.

## The Key Idea

This crate implements the core abstractions needed for its domain, with a focus on correctness, composability, and conservation guarantees. Every public type is serializable (serde), every algorithm is tested, and every invariant is verified.

## Install

```bash
cargo add lau-blueprint
```

## Quick Start

See the API Reference below for complete usage. Key entry points:

```rust
use lau_blueprint::*;
// See types and methods below for complete usage
```

## API Reference

```rust
pub enum Material 
    pub fn vibe_cost(&self) -> f64 
pub struct BlueprintBlock 
    pub fn new(position: (i32, i32, i32), material: Material, rotation: u8) -> Self 
pub struct Blueprint 
    pub fn new(name: impl Into<String>, author: impl Into<String>) -> Self 
    pub fn add_block(&mut self, block: BlueprintBlock) 
    pub fn remove_block(&mut self, pos: (i32, i32, i32)) -> bool 
    pub fn block_count(&self) -> usize 
    pub fn bounds(&self) -> ((i32, i32, i32), (i32, i32, i32)) 
    pub fn center_of_mass(&self) -> (f64, f64, f64) 
    pub fn material_count(&self) -> HashMap<Material, usize> 
    pub fn total_vibe_cost(&self) -> f64 
pub struct BlueprintLib 
    pub fn new() -> Self 
    pub fn save(&mut self, bp: Blueprint) 
    pub fn load(&self, name: &str) -> Option<&Blueprint> 
    pub fn search_by_tag(&self, tag: &str) -> Vec<&Blueprint> 
    pub fn search_by_material(&self, mat: Material) -> Vec<&Blueprint> 
    pub fn fork(&self, name: &str, new_name: &str, new_author: &str) -> Option<Blueprint> 
pub struct ValidationResult 
pub struct StructureValidator;
    pub fn new() -> Self 
    pub fn validate(&self, bp: &Blueprint) -> ValidationResult 
pub fn small_house() -> Blueprint 
pub fn tower() -> Blueprint 
pub fn bridge() -> Blueprint 
pub fn crystal_garden() -> Blueprint 
pub fn castle_wall() -> Blueprint 
```

## How It Works

Read the source in `src/` for full implementation details. All algorithms are documented with inline comments explaining the mathematical foundations.

## The Math

This crate implements formal mathematical constructs. See the source documentation for theorem statements and proofs of correctness.

## Testing

**28 tests** covering construction, serialization, correctness properties, edge cases, and composability with other lau-* crates.

## License

MIT
