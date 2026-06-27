use rustc_hash::FxHashMap;
use simdnbt::ToNbtTag;
use simdnbt::owned::NbtTag;
use steel_utils::Identifier;

/// Represents an armor trim material definition from the data packs.
#[derive(Debug)]
pub struct TrimMaterial {
    pub key: Identifier,
    pub palette_id: Identifier,
    pub description: StyledTextComponent,
}

/// Represents a translatable text component that can also include styling.
#[derive(Debug)]
pub struct StyledTextComponent {
    pub translate: String,
    pub color: Option<String>,
}

impl ToNbtTag for &TrimMaterial {
    fn to_nbt_tag(self) -> NbtTag {
        use simdnbt::owned::NbtCompound;
        let mut compound = NbtCompound::new();
        let palette_id = self.palette_id.to_string();
        compound.insert("palette_id", palette_id.as_str());
        let mut desc = NbtCompound::new();
        desc.insert("translate", self.description.translate.as_str());
        if let Some(color) = &self.description.color {
            desc.insert("color", color.as_str());
        }
        compound.insert("description", NbtTag::Compound(desc));
        NbtTag::Compound(compound)
    }
}

pub type TrimMaterialRef = &'static TrimMaterial;

pub struct TrimMaterialRegistry {
    trim_materials_by_id: Vec<TrimMaterialRef>,
    trim_materials_by_key: FxHashMap<Identifier, usize>,
    allows_registering: bool,
}

impl TrimMaterialRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            trim_materials_by_id: Vec::new(),
            trim_materials_by_key: FxHashMap::default(),
            allows_registering: true,
        }
    }
}

crate::impl_standard_methods!(
    TrimMaterialRegistry,
    TrimMaterialRef,
    trim_materials_by_id,
    trim_materials_by_key,
    allows_registering
);

crate::impl_registry!(
    TrimMaterialRegistry,
    TrimMaterial,
    trim_materials_by_id,
    trim_materials_by_key,
    trim_materials
);
