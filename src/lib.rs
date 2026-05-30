use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Materials available for building.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Material {
    Stone,
    Wood,
    Glass,
    Crystal,
    Metal,
    Glow,
    Water,
    Lava,
    Grass,
    Sand,
}

impl Material {
    /// Vibe cost per block of this material.
    pub fn vibe_cost(&self) -> f64 {
        match self {
            Material::Stone => 1.0,
            Material::Wood => 0.5,
            Material::Glass => 2.0,
            Material::Crystal => 5.0,
            Material::Metal => 3.0,
            Material::Glow => 4.0,
            Material::Water => 1.5,
            Material::Lava => 6.0,
            Material::Grass => 0.3,
            Material::Sand => 0.4,
        }
    }
}

/// A single block within a blueprint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlueprintBlock {
    pub position: (i32, i32, i32),
    pub material: Material,
    /// Rotation in quarter-turns (0–3).
    pub rotation: u8,
}

impl BlueprintBlock {
    pub fn new(position: (i32, i32, i32), material: Material, rotation: u8) -> Self {
        let rotation = rotation % 4;
        Self { position, material, rotation }
    }
}

/// A named building plan composed of blocks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blueprint {
    pub name: String,
    pub blocks: Vec<BlueprintBlock>,
    pub author: String,
    pub version: u32,
    pub tags: Vec<String>,
}

impl Blueprint {
    pub fn new(name: impl Into<String>, author: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            blocks: Vec::new(),
            author: author.into(),
            version: 1,
            tags: Vec::new(),
        }
    }

    pub fn add_block(&mut self, block: BlueprintBlock) {
        self.blocks.push(block);
    }

    pub fn remove_block(&mut self, pos: (i32, i32, i32)) -> bool {
        let before = self.blocks.len();
        self.blocks.retain(|b| b.position != pos);
        self.blocks.len() < before
    }

    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }

    /// Returns (min_corner, max_corner) of the bounding box. Panics if empty.
    pub fn bounds(&self) -> ((i32, i32, i32), (i32, i32, i32)) {
        assert!(!self.blocks.is_empty(), "bounds() on empty blueprint");
        let mut min = self.blocks[0].position;
        let mut max = self.blocks[0].position;
        for b in &self.blocks {
            min.0 = min.0.min(b.position.0);
            min.1 = min.1.min(b.position.1);
            min.2 = min.2.min(b.position.2);
            max.0 = max.0.max(b.position.0);
            max.1 = max.1.max(b.position.1);
            max.2 = max.2.max(b.position.2);
        }
        (min, max)
    }

    pub fn center_of_mass(&self) -> (f64, f64, f64) {
        if self.blocks.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        let n = self.blocks.len() as f64;
        let sum = self.blocks.iter().fold((0i64, 0i64, 0i64), |acc, b| {
            (acc.0 + b.position.0 as i64, acc.1 + b.position.1 as i64, acc.2 + b.position.2 as i64)
        });
        (sum.0 as f64 / n, sum.1 as f64 / n, sum.2 as f64 / n)
    }

    pub fn material_count(&self) -> HashMap<Material, usize> {
        let mut map = HashMap::new();
        for b in &self.blocks {
            *map.entry(b.material).or_insert(0) += 1;
        }
        map
    }

    pub fn total_vibe_cost(&self) -> f64 {
        self.blocks.iter().map(|b| b.material.vibe_cost()).sum()
    }
}

/// A library of named blueprints.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BlueprintLib {
    pub blueprints: HashMap<String, Blueprint>,
}

impl BlueprintLib {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn save(&mut self, bp: Blueprint) {
        self.blueprints.insert(bp.name.clone(), bp);
    }

    pub fn load(&self, name: &str) -> Option<&Blueprint> {
        self.blueprints.get(name)
    }

    pub fn search_by_tag(&self, tag: &str) -> Vec<&Blueprint> {
        self.blueprints
            .values()
            .filter(|bp| bp.tags.iter().any(|t| t.eq_ignore_ascii_case(tag)))
            .collect()
    }

    pub fn search_by_material(&self, mat: Material) -> Vec<&Blueprint> {
        self.blueprints
            .values()
            .filter(|bp| bp.blocks.iter().any(|b| b.material == mat))
            .collect()
    }

    pub fn fork(&self, name: &str, new_name: &str, new_author: &str) -> Option<Blueprint> {
        self.blueprints.get(name).map(|bp| Blueprint {
            name: new_name.to_string(),
            blocks: bp.blocks.clone(),
            author: new_author.to_string(),
            version: 1,
            tags: bp.tags.clone(),
        })
    }
}

/// Result of validating a blueprint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

const MAX_BLOCKS: usize = 200;

/// Checks that a blueprint is structurally sound.
#[derive(Debug, Clone, Default)]
pub struct StructureValidator;

impl StructureValidator {
    pub fn new() -> Self {
        Self
    }

    pub fn validate(&self, bp: &Blueprint) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        if bp.blocks.is_empty() {
            errors.push("Blueprint has no blocks".to_string());
            return ValidationResult { valid: false, errors, warnings };
        }

        if bp.block_count() > MAX_BLOCKS {
            errors.push(format!("Blueprint has {} blocks, max is {}", bp.block_count(), MAX_BLOCKS));
        }

        let has_foundation = bp.blocks.iter().any(|b| b.position.1 == 0);
        if !has_foundation {
            errors.push("Blueprint has no foundation (no blocks at y=0)".to_string());
        }

        // Connectivity: every block must be reachable from ground blocks via 6-adjacency
        if !bp.blocks.is_empty() {
            let positions: std::collections::HashSet<(i32, i32, i32)> =
                bp.blocks.iter().map(|b| b.position).collect();
            let ground: Vec<_> = positions.iter().filter(|p| p.1 == 0).copied().collect();

            if !ground.is_empty() {
                let mut visited = std::collections::HashSet::new();
                let mut queue = std::collections::VecDeque::new();
                for g in &ground {
                    visited.insert(*g);
                    queue.push_back(*g);
                }
                while let Some(p) = queue.pop_front() {
                    for nb in [
                        (p.0 + 1, p.1, p.2), (p.0 - 1, p.1, p.2),
                        (p.0, p.1 + 1, p.2), (p.0, p.1 - 1, p.2),
                        (p.0, p.1, p.2 + 1), (p.0, p.1, p.2 - 1),
                    ] {
                        if positions.contains(&nb) && !visited.contains(&nb) {
                            visited.insert(nb);
                            queue.push_back(nb);
                        }
                    }
                }
                if visited.len() != positions.len() {
                    errors.push(format!(
                        "Blueprint has {} floating blocks (not connected to ground)",
                        positions.len() - visited.len()
                    ));
                }
            }
        }

        let mc = bp.material_count();
        if mc.contains_key(&Material::Lava) && mc.contains_key(&Material::Water) {
            warnings.push("Lava and Water in the same blueprint may cause steam explosions".to_string());
        }
        if mc.contains_key(&Material::Wood) && mc.contains_key(&Material::Lava) {
            warnings.push("Wood and Lava adjacent — fire hazard!".to_string());
        }
        if let Some(&gc) = mc.get(&Material::Glass) {
            if gc > bp.block_count() / 2 {
                warnings.push("More than half the blocks are Glass — fragile!".to_string());
            }
        }

        let valid = errors.is_empty();
        ValidationResult { valid, errors, warnings }
    }
}

// --- Pre-built blueprints ---

/// Small House: 20 blocks. A simple 5×1×4 vertical structure (wall-like house).
pub fn small_house() -> Blueprint {
    let mut bp = Blueprint::new("Small House", "System");
    bp.tags = vec!["starter".into(), "house".into()];
    // 5-long, 1-wide, 4-tall = 20 blocks
    // Floor y=0 (5 stone)
    for x in 0..5 { bp.add_block(BlueprintBlock::new((x, 0, 0), Material::Stone, 0)); }
    // Walls y=1 (5 wood)
    for x in 0..5 { bp.add_block(BlueprintBlock::new((x, 1, 0), Material::Wood, 0)); }
    // Windows y=2 (5: 4 wood + 1 glass)
    for x in 0..5 {
        let m = if x == 2 { Material::Glass } else { Material::Wood };
        bp.add_block(BlueprintBlock::new((x, 2, 0), m, 0));
    }
    // Roof y=3 (5 wood)
    for x in 0..5 { bp.add_block(BlueprintBlock::new((x, 3, 0), Material::Wood, 0)); }
    bp
}

/// Tower: 15 blocks. 2×2 base + 7-column + 2×2 top.
pub fn tower() -> Blueprint {
    let mut bp = Blueprint::new("Tower", "System");
    bp.tags = vec!["tall".into(), "medieval".into()];
    // Base 2×2 = 4
    for x in 0..2 { for z in 0..2 { bp.add_block(BlueprintBlock::new((x, 0, z), Material::Stone, 0)); } }
    // Column from y=1..7 = 7
    for y in 1..8 { bp.add_block(BlueprintBlock::new((0, y, 0), Material::Stone, 0)); }
    // Top 2×2 = 4
    for x in 0..2 { for z in 0..2 { bp.add_block(BlueprintBlock::new((x, 8, z), Material::Wood, 0)); } }
    // 4 + 7 + 4 = 15
    bp
}

/// Bridge: 12 blocks. 6×1×2 path.
pub fn bridge() -> Blueprint {
    let mut bp = Blueprint::new("Bridge", "System");
    bp.tags = vec!["path".into(), "crossing".into()];
    for x in 0..6 {
        bp.add_block(BlueprintBlock::new((x, 0, 0), Material::Stone, 0));
        bp.add_block(BlueprintBlock::new((x, 0, 1), Material::Wood, 0));
    }
    bp
}

/// Crystal Garden: 18 blocks.
pub fn crystal_garden() -> Blueprint {
    let mut bp = Blueprint::new("Crystal Garden", "System");
    bp.tags = vec!["magical".into(), "garden".into()];
    // Grass base 3×2 = 6
    for x in 0..3 { for z in 0..2 { bp.add_block(BlueprintBlock::new((x, 0, z), Material::Grass, 0)); } }
    // Crystal pillars: 2 at col(0) + 3 at col(2,z=1) = 5
    bp.add_block(BlueprintBlock::new((0, 1, 0), Material::Crystal, 0));
    bp.add_block(BlueprintBlock::new((0, 2, 0), Material::Crystal, 0));
    bp.add_block(BlueprintBlock::new((2, 1, 1), Material::Crystal, 0));
    bp.add_block(BlueprintBlock::new((2, 2, 1), Material::Crystal, 0));
    bp.add_block(BlueprintBlock::new((2, 3, 1), Material::Crystal, 0));
    // Glow accents: 4
    bp.add_block(BlueprintBlock::new((1, 1, 0), Material::Glow, 0));
    bp.add_block(BlueprintBlock::new((1, 1, 1), Material::Glow, 0));
    bp.add_block(BlueprintBlock::new((0, 1, 1), Material::Glow, 0));
    bp.add_block(BlueprintBlock::new((2, 0, 1), Material::Glow, 0));
    // Sand detail: 3 more (connected to ground via glow/adjacency)
    // Need 18 total: 6+5+4=15, need 3 more
    bp.add_block(BlueprintBlock::new((2, 4, 1), Material::Crystal, 0));
    bp.add_block(BlueprintBlock::new((0, 3, 0), Material::Glow, 0));
    bp.add_block(BlueprintBlock::new((1, 2, 1), Material::Glow, 0));
    // 6+5+4+3 = 18
    bp
}

/// Castle Wall: 25 blocks. 5-long × 5-tall × 1-thick.
pub fn castle_wall() -> Blueprint {
    let mut bp = Blueprint::new("Castle Wall", "System");
    bp.tags = vec!["medieval".into(), "defense".into()];
    for x in 0..5 { for y in 0..5 { bp.add_block(BlueprintBlock::new((x, y, 0), Material::Stone, 0)); } }
    bp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_material_vibe_costs() {
        assert_eq!(Material::Grass.vibe_cost(), 0.3);
        assert_eq!(Material::Crystal.vibe_cost(), 5.0);
        assert_eq!(Material::Lava.vibe_cost(), 6.0);
        assert_eq!(Material::Wood.vibe_cost(), 0.5);
    }

    #[test]
    fn test_block_new_clamps_rotation() {
        let b = BlueprintBlock::new((0, 0, 0), Material::Stone, 7);
        assert_eq!(b.rotation, 3);
        let b2 = BlueprintBlock::new((1, 1, 1), Material::Wood, 2);
        assert_eq!(b2.rotation, 2);
    }

    #[test]
    fn test_blueprint_add_remove() {
        let mut bp = Blueprint::new("Test", "Alice");
        bp.add_block(BlueprintBlock::new((0, 0, 0), Material::Stone, 0));
        bp.add_block(BlueprintBlock::new((1, 0, 0), Material::Wood, 1));
        assert_eq!(bp.block_count(), 2);
        assert!(bp.remove_block((1, 0, 0)));
        assert_eq!(bp.block_count(), 1);
        assert!(!bp.remove_block((9, 9, 9)));
    }

    #[test]
    fn test_blueprint_bounds() {
        let mut bp = Blueprint::new("B", "A");
        bp.add_block(BlueprintBlock::new((2, 3, 5), Material::Stone, 0));
        bp.add_block(BlueprintBlock::new((-1, 0, 4), Material::Wood, 0));
        let (min, max) = bp.bounds();
        assert_eq!(min, (-1, 0, 4));
        assert_eq!(max, (2, 3, 5));
    }

    #[test]
    fn test_center_of_mass() {
        let mut bp = Blueprint::new("C", "A");
        bp.add_block(BlueprintBlock::new((0, 0, 0), Material::Stone, 0));
        bp.add_block(BlueprintBlock::new((2, 2, 2), Material::Stone, 0));
        let com = bp.center_of_mass();
        assert!((com.0 - 1.0).abs() < f64::EPSILON);
        assert!((com.1 - 1.0).abs() < f64::EPSILON);
        assert!((com.2 - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_center_of_mass_empty() {
        let bp = Blueprint::new("E", "A");
        assert_eq!(bp.center_of_mass(), (0.0, 0.0, 0.0));
    }

    #[test]
    fn test_material_count() {
        let mut bp = Blueprint::new("M", "A");
        bp.add_block(BlueprintBlock::new((0, 0, 0), Material::Stone, 0));
        bp.add_block(BlueprintBlock::new((1, 0, 0), Material::Stone, 0));
        bp.add_block(BlueprintBlock::new((2, 0, 0), Material::Wood, 0));
        let mc = bp.material_count();
        assert_eq!(mc[&Material::Stone], 2);
        assert_eq!(mc[&Material::Wood], 1);
        assert_eq!(mc.len(), 2);
    }

    #[test]
    fn test_total_vibe_cost() {
        let mut bp = Blueprint::new("V", "A");
        bp.add_block(BlueprintBlock::new((0, 0, 0), Material::Grass, 0));
        bp.add_block(BlueprintBlock::new((1, 0, 0), Material::Stone, 0));
        assert!((bp.total_vibe_cost() - 1.3).abs() < 1e-9);
    }

    #[test]
    fn test_lib_save_load() {
        let mut lib = BlueprintLib::new();
        lib.save(small_house());
        assert!(lib.load("Small House").is_some());
        assert!(lib.load("nope").is_none());
    }

    #[test]
    fn test_lib_search_by_tag() {
        let mut lib = BlueprintLib::new();
        lib.save(small_house());
        lib.save(tower());
        lib.save(bridge());
        lib.save(castle_wall());
        let medieval = lib.search_by_tag("medieval");
        assert_eq!(medieval.len(), 2);
        let starter = lib.search_by_tag("starter");
        assert_eq!(starter.len(), 1);
        assert_eq!(starter[0].name, "Small House");
    }

    #[test]
    fn test_lib_search_by_material() {
        let mut lib = BlueprintLib::new();
        lib.save(small_house());
        lib.save(crystal_garden());
        let crystal = lib.search_by_material(Material::Crystal);
        assert_eq!(crystal.len(), 1);
        assert_eq!(crystal[0].name, "Crystal Garden");
    }

    #[test]
    fn test_lib_fork() {
        let mut lib = BlueprintLib::new();
        lib.save(small_house());
        let forked = lib.fork("Small House", "My House", "Bob").unwrap();
        assert_eq!(forked.name, "My House");
        assert_eq!(forked.author, "Bob");
        assert_eq!(forked.version, 1);
        assert_eq!(forked.block_count(), 20);
    }

    #[test]
    fn test_lib_fork_missing() {
        let lib = BlueprintLib::new();
        assert!(lib.fork("ghost", "new", "me").is_none());
    }

    #[test]
    fn test_validator_empty_blueprint() {
        let bp = Blueprint::new("E", "A");
        let result = StructureValidator::new().validate(&bp);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("no blocks")));
    }

    #[test]
    fn test_validator_no_foundation() {
        let mut bp = Blueprint::new("F", "A");
        bp.add_block(BlueprintBlock::new((0, 1, 0), Material::Stone, 0));
        let result = StructureValidator::new().validate(&bp);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("foundation")));
    }

    #[test]
    fn test_validator_valid_blueprint() {
        let bp = small_house();
        let result = StructureValidator::new().validate(&bp);
        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validator_lava_water_warning() {
        let mut bp = Blueprint::new("Danger", "A");
        bp.add_block(BlueprintBlock::new((0, 0, 0), Material::Stone, 0));
        bp.add_block(BlueprintBlock::new((1, 0, 0), Material::Lava, 0));
        bp.add_block(BlueprintBlock::new((2, 0, 0), Material::Water, 0));
        let result = StructureValidator::new().validate(&bp);
        assert!(result.warnings.iter().any(|w| w.contains("steam")));
    }

    #[test]
    fn test_validator_floating_blocks() {
        let mut bp = Blueprint::new("Float", "A");
        bp.add_block(BlueprintBlock::new((0, 0, 0), Material::Stone, 0));
        // Block at y=3 not connected to ground (gap at y=1, y=2)
        bp.add_block(BlueprintBlock::new((0, 3, 0), Material::Stone, 0));
        let result = StructureValidator::new().validate(&bp);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("floating")));
    }

    #[test]
    fn test_validator_size_limit() {
        let mut bp = Blueprint::new("Big", "A");
        for i in 0..=200_i32 {
            bp.add_block(BlueprintBlock::new((i, 0, 0), Material::Stone, 0));
        }
        let result = StructureValidator::new().validate(&bp);
        assert!(result.errors.iter().any(|e| e.contains("max")));
    }

    #[test]
    fn test_serde_roundtrip() {
        let bp = small_house();
        let json = serde_json::to_string(&bp).unwrap();
        let bp2: Blueprint = serde_json::from_str(&json).unwrap();
        assert_eq!(bp.name, bp2.name);
        assert_eq!(bp.blocks, bp2.blocks);
    }

    #[test]
    fn test_serde_material() {
        let mat = Material::Crystal;
        let json = serde_json::to_string(&mat).unwrap();
        let mat2: Material = serde_json::from_str(&json).unwrap();
        assert_eq!(mat, mat2);
    }

    #[test]
    fn test_small_house_block_count() {
        assert_eq!(small_house().block_count(), 20);
    }

    #[test]
    fn test_tower_block_count() {
        assert_eq!(tower().block_count(), 15);
    }

    #[test]
    fn test_bridge_block_count() {
        assert_eq!(bridge().block_count(), 12);
    }

    #[test]
    fn test_crystal_garden_block_count() {
        assert_eq!(crystal_garden().block_count(), 18);
    }

    #[test]
    fn test_castle_wall_block_count() {
        assert_eq!(castle_wall().block_count(), 25);
    }

    #[test]
    fn test_prebuilt_blueprints_validate() {
        let v = StructureValidator::new();
        assert!(v.validate(&small_house()).valid);
        assert!(v.validate(&tower()).valid);
        assert!(v.validate(&bridge()).valid);
        assert!(v.validate(&crystal_garden()).valid);
        assert!(v.validate(&castle_wall()).valid);
    }

    #[test]
    fn test_validation_result_serde() {
        let vr = ValidationResult {
            valid: true,
            errors: vec![],
            warnings: vec!["fragile".into()],
        };
        let json = serde_json::to_string(&vr).unwrap();
        let vr2: ValidationResult = serde_json::from_str(&json).unwrap();
        assert!(vr2.valid);
        assert_eq!(vr2.warnings.len(), 1);
    }
}
