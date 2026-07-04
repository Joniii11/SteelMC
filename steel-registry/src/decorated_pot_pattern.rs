use rustc_hash::FxHashMap;
use simdnbt::ToNbtTag;
use simdnbt::owned::NbtTag;
use steel_utils::Identifier;

#[derive(Debug)]
pub struct DecoratedPotPattern {
    pub key: Identifier,
    pub asset_id: Identifier,
}

impl ToNbtTag for &DecoratedPotPattern {
    fn to_nbt_tag(self) -> NbtTag {
        use simdnbt::owned::NbtCompound;
        let mut compound = NbtCompound::new();
        let asset_id = self.asset_id.to_string();
        compound.insert("asset_id", asset_id.as_str());
        NbtTag::Compound(compound)
    }
}

pub type DecoratedPotPatternRef = &'static DecoratedPotPattern;

pub struct DecoratedPotPatternRegistry {
    decorated_pot_patterns_by_id: Vec<DecoratedPotPatternRef>,
    decorated_pot_patterns_by_key: FxHashMap<Identifier, usize>,
    allows_registering: bool,
}

impl DecoratedPotPatternRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            decorated_pot_patterns_by_id: Vec::new(),
            decorated_pot_patterns_by_key: FxHashMap::default(),
            allows_registering: true,
        }
    }
}

crate::impl_standard_methods!(
    DecoratedPotPatternRegistry,
    DecoratedPotPatternRef,
    decorated_pot_patterns_by_id,
    decorated_pot_patterns_by_key,
    allows_registering
);

crate::impl_registry!(
    DecoratedPotPatternRegistry,
    DecoratedPotPattern,
    decorated_pot_patterns_by_id,
    decorated_pot_patterns_by_key,
    decorated_pot_patterns
);
