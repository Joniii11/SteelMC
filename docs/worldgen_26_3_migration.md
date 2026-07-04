# Worldgen 26.3 Migration

Steel targets `0.12.1+mc26.3-snapshot-1`. Vanilla 26.3 moves worldgen feature
data from configured-feature entries toward direct `Feature` registry entries.
Keep this migration aligned with `minecraft-src/net/...` and
`minecraft-src/data/minecraft/worldgen/...`; the older
`minecraft-src/minecraft/src/...` tree may still contain stable-version code.

## Completed In This Pass

1. Replaced Steel's `ConfiguredFeature*` worldgen registry model with
   `Feature*` names and generated `vanilla_features`.
2. Moved feature generation to
   `steel-utils/build_assets/builtin_datapacks/minecraft/worldgen/feature`.
3. Kept `PlacedFeatureData` as the placement modifier chain plus direct
   `FeatureRef`.
4. Added 26.3 ore `RuleTest` support for `all_of` and `height_match`.
5. Added new poplar tree components:
   `poplar_trunk_placer`, `poplar_foliage_placer`, and `shelf_mushroom`.
6. Added `end_podium` feature data/runtime support.
7. Updated trim material parsing/NBT to Vanilla 26.3 `palette_id`.
8. Verified and added structure processor rule-test changes for
   `blockstate_match`, `random_block_match`, and `random_blockstate_match`.
9. Added `dimension_origin` structure placement and verified abandoned camp
   structure-set/template-pool data against `minecraft-src/data` and extracted
   assets.
10. Audited direct-feature runtime behavior for selectors, vegetation patch,
    underwater magma, twisting vines, and large dripstone against
    `minecraft-src/net/...`.
11. Made generated vanilla template-pool/template constructors stack-friendly
    so default test-thread stacks can load vanilla structure assets.

Verified with:

- `cargo fmt --all --check`
- `cargo check -p steel-registry`
- `cargo check -p steel-core`
- `cargo check -p steel-worldgen`
- targeted `steel-worldgen` tests for `dimension_origin` and
  `load_vanilla_structure_sets`
- targeted `steel-worldgen` test
  `structure::generator::tests::vanilla_assets_cover_vanilla_structure_sets`
- targeted `steel-core` test
  `worldgen::registry::tests::default_flat_config_matches_vanilla_superflat`

## Remaining Plan

No known open 26.3 worldgen migration items remain in this document.

## Rules

- Do not edit generated Rust under `src/generated/` directly.
- If extracted JSON is missing or malformed, update SteelExtractor or request
  the exact output instead of hand-authoring vanilla data.
- Verify gameplay-affecting behavior against local Vanilla source before
  implementing it.
