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

Verified with:

- `cargo fmt --all --check`
- `cargo check -p steel-registry`
- `cargo check -p steel-core`

## Remaining Plan

1. Verify structure processor rule-test changes from the diff, especially
   `blockstate_match`, `random_block_match`, and `random_blockstate_match`.
2. Add `dimension_origin` structure placement and verify abandoned camp data
   against generated structure-set and template-pool output.
3. Audit direct-feature behavior changes for selectors, vegetation patch,
   underwater magma, twisting vines, and large dripstone against
   `minecraft-src/net/...`.
4. Run broader worldgen tests/checks once the remaining structure pieces are in.

## Rules

- Do not edit generated Rust under `src/generated/` directly.
- If extracted JSON is missing or malformed, update SteelExtractor or request
  the exact output instead of hand-authoring vanilla data.
- Verify gameplay-affecting behavior against local Vanilla source before
  implementing it.
