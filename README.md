# lau-blueprint

A voxel building-plan system with materials, structural validation, cost accounting, and a blueprint library — designed for game worlds where players construct structures from blocks with different materials, costs, and physical constraints.

> **28 tests** · depends on `serde` + `serde_json`

---

## What This Does

This crate provides:

1. **Materials** — 10 building materials (Stone, Wood, Glass, Crystal, Metal, Glow, Water, Lava, Grass, Sand), each with a per-block "vibe cost."
2. **Blueprints** — Named building plans composed of positioned, rotated blocks with metadata (author, version, tags).
3. **Structural Validation** — Checks that blueprints have foundations, are connected to ground, and don't exceed size limits. Warns about hazardous material combinations (lava + water, wood + lava, fragile glass).
4. **Blueprint Library** — A save/load/search/fork registry for managing named blueprints.
5. **Pre-built Blueprints** — Small House (20 blocks), Tower (15), Bridge (12), Crystal Garden (18), Castle Wall (25).

---

## Key Idea

A **blueprint** is a collection of voxel blocks in 3D space. Each block has a position `(x, y, z)`, a material, and a rotation (0–3 quarter-turns). The system validates that structures are physically plausible:

- **Foundation check**: at least one block at y=0 (ground level).
- **Connectivity check**: every block must be reachable from ground blocks via 6-adjacency (BFS flood fill).
- **Size limit**: max 200 blocks per blueprint.
- **Material warnings**: lava + water = steam explosion risk; wood + lava = fire hazard; majority glass = fragile.

The validation uses BFS from all ground blocks, checking that every block in the blueprint is reachable through face-adjacent neighbors.

---

## Install

```toml
[dependencies]
lau-blueprint = { git = "https://github.com/SuperInstance/lau-blueprint" }
```

Dependencies: `serde`, `serde_json`.

---

## Quick Start

```rust
use lau_blueprint::*;

// Create a blueprint
let mut bp = Blueprint::new("My Tower", "Alice");
bp.tags = vec!["tall".into(), "fortress".into()];

// Add blocks
bp.add_block(BlueprintBlock::new((0, 0, 0), Material::Stone, 0));
bp.add_block(BlueprintBlock::new((0, 1, 0), Material::Stone, 0));
bp.add_block(BlueprintBlock::new((0, 2, 0), Material::Crystal, 0));

// Inspect
println!("blocks: {}", bp.block_count());              // 3
println!("bounds: {:?}", bp.bounds());                  // ((0,0,0), (0,2,0))
println!("center of mass: {:?}", bp.center_of_mass());  // (0.0, 1.0, 0.0)
println!("vibe cost: {:.1}", bp.total_vibe_cost());     // 1.0 + 1.0 + 5.0 = 7.0

// Validate
let result = StructureValidator::new().validate(&bp);
assert!(result.valid);

// Save to library
let mut lib = BlueprintLib::new();
lib.save(bp);
let loaded = lib.load("My Tower").unwrap();

// Search
let magical = lib.search_by_tag("magical");
let crystal = lib.search_by_material(Material::Crystal);

// Fork
let forked = lib.fork("My Tower", "Bob's Tower", "Bob").unwrap();
```

### Using Pre-built Blueprints

```rust
use lau_blueprint::*;

let house = small_house();       // 20 blocks
let twr = tower();               // 15 blocks
let brg = bridge();              // 12 blocks
let garden = crystal_garden();   // 18 blocks
let wall = castle_wall();        // 25 blocks

// All pass validation
assert!(StructureValidator::new().validate(&house).valid);
```

---

## API Reference

### Material

```rust
pub enum Material {
    Stone, Wood, Glass, Crystal, Metal, Glow, Water, Lava, Grass, Sand,
}
```

| Method | Description |
|---|---|
| `vibe_cost()` | Per-block cost (Grass=0.3, Sand=0.4, Wood=0.5, Stone=1.0, Water=1.5, Glass=2.0, Metal=3.0, Glow=4.0, Crystal=5.0, Lava=6.0) |

### BlueprintBlock

| Field | Type | Description |
|---|---|---|
| `position` | `(i32, i32, i32)` | 3D coordinates |
| `material` | `Material` | Block material |
| `rotation` | `u8` | Quarter-turns (0–3, clamped mod 4) |

| Method | Description |
|---|---|
| `new(position, material, rotation)` | Create block (rotation clamped to 0–3) |

### Blueprint

| Field | Type | Description |
|---|---|---|
| `name` | `String` | Blueprint name (used as library key) |
| `blocks` | `Vec<BlueprintBlock>` | All blocks |
| `author` | `String` | Creator name |
| `version` | `u32` | Version number |
| `tags` | `Vec<String>` | Searchable tags |

| Method | Description |
|---|---|
| `new(name, author)` | Create empty blueprint |
| `add_block(block)` | Add a block |
| `remove_block(position)` | Remove block at position → true if found |
| `block_count()` | Number of blocks |
| `bounds()` | (min_corner, max_corner) — panics if empty |
| `center_of_mass()` | Average (x, y, z) of all blocks |
| `material_count()` | HashMap<Material, usize> counts |
| `total_vibe_cost()` | Sum of all block vibe costs |

### BlueprintLib

| Method | Description |
|---|---|
| `new()` | Create empty library |
| `save(bp)` | Insert/update blueprint by name |
| `load(name)` | Get reference to blueprint by name |
| `search_by_tag(tag)` | Find blueprints matching a tag (case-insensitive) |
| `search_by_material(mat)` | Find blueprints containing a material |
| `fork(name, new_name, new_author)` | Clone a blueprint with new name/author |

### StructureValidator

| Method | Description |
|---|---|
| `new()` | Create validator |
| `validate(bp)` | Returns `ValidationResult` |

### ValidationResult

| Field | Description |
|---|---|
| `valid` | true if no errors |
| `errors` | List of error strings |
| `warnings` | List of warning strings |

**Errors**: empty blueprint, no foundation, no connectivity (floating blocks), exceeds 200-block limit.
**Warnings**: lava + water (steam), wood + lava (fire), >50% glass (fragile).

### Pre-built Blueprints

| Function | Blocks | Tags | Description |
|---|---|---|---|
| `small_house()` | 20 | starter, house | 5×1×4 vertical structure |
| `tower()` | 15 | tall, medieval | 2×2 base + 7-column + 2×2 top |
| `bridge()` | 12 | path, crossing | 6×1×2 path |
| `crystal_garden()` | 18 | magical, garden | Grass base + crystal pillars + glow accents |
| `castle_wall()` | 25 | medieval, defense | 5×5×1 stone wall |

---

## How It Works

### Structural Validation

The validator runs a series of checks:

1. **Empty check**: At least one block must exist.
2. **Size check**: Block count ≤ 200.
3. **Foundation check**: At least one block at y=0.
4. **Connectivity check**: BFS from all ground blocks (y=0) using 6-adjacency. If any block is unreachable, it's "floating."
5. **Material warnings**: Cross-checks material combinations.

### BFS Connectivity

```
ground_blocks = { p ∈ blocks | p.y == 0 }
visited = BFS from ground_blocks via 6-neighbors (±x, ±y, ±z)
if |visited| < |blocks|:
    error: "floating blocks"
```

This ensures structures are physically connected to the ground — no floating islands.

### Bounding Box

Finds min and max coordinates across all blocks. Used for spatial reasoning (e.g., "does this fit in a chunk?").

### Center of Mass

Arithmetic mean of all block positions:

$$(x_{cm}, y_{cm}, z_{cm}) = \frac{1}{n} \sum_{i=1}^{n} (x_i, y_i, z_i)$$

Currently unweighted (all materials have the same mass). Could be extended with per-material density.

### Vibe Cost

Each material has a fixed cost per block. The total cost of a blueprint is:

$$C = \sum_{i=1}^{n} \text{vibe\_cost}(\text{material}_i)$$

This represents the "energy" or "resources" required to manifest the structure.

### Blueprint Library

A `HashMap<String, Blueprint>` keyed by name. Supports:
- **Save/Load**: insert and lookup by name.
- **Search by tag**: linear scan, case-insensitive tag matching.
- **Search by material**: linear scan, checks if any block uses the material.
- **Fork**: clone a blueprint with new name, author, and version=1.

---

## The Math

### Vibe Cost Table

| Material | Cost |
|---|---|
| Grass | 0.3 |
| Sand | 0.4 |
| Wood | 0.5 |
| Stone | 1.0 |
| Water | 1.5 |
| Glass | 2.0 |
| Metal | 3.0 |
| Glow | 4.0 |
| Crystal | 5.0 |
| Lava | 6.0 |

### Center of Mass

$$\mathbf{r}_{cm} = \frac{1}{n} \sum_{i=1}^{n} \mathbf{r}_i$$

### Connectivity (Graph Theory)

Blocks form a graph G = (V, E) where edges connect 6-adjacent voxels. The validator checks that all blocks are in the connected component containing ground blocks:

$$V_{ground} = \{v \in V : v_y = 0\}$$
$$CC(V_{ground}) = V \implies \text{valid}$$

### Bounding Box

$$\mathbf{b}_{min} = \left(\min_i x_i,\; \min_i y_i,\; \min_i z_i\right)$$
$$\mathbf{b}_{max} = \left(\max_i x_i,\; \max_i y_i,\; \max_i z_i\right)$$

---

## License

MIT or Apache-2.0 (at your option).
