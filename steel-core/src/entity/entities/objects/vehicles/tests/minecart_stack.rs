//! Compare the optimized normal manager tick with its original ordered contact loop.

use std::sync::Arc;

use glam::DVec3;
use steel_registry::blocks::block_state_ext::BlockStateExt as _;
use steel_registry::blocks::properties::{BlockStateProperties, RailShape};
use steel_registry::entity_type::EntityTypeRef;
use steel_registry::{vanilla_blocks, vanilla_entities};
use steel_utils::locks::SyncMutex;
use steel_utils::{BlockPos, ChunkPos, Downcast as _};

use crate::entity::entities::TntMinecartEntity;
use crate::entity::entities::objects::vehicles::{
    minecart_behavior::measure_cart_stack_skips,
    test_support::{CART_BLOCK_POS, set_block, test_world},
};
use crate::entity::tests::multipart::MultipartTestEntity;
use crate::entity::{
    Entity, EntityBase, EntityEventSource, EntityVisibility, RemovalReason, SharedEntity,
    create_entity_instance, next_entity_id,
};
use crate::test_support::TestWorld;
use crate::world::World;

const TNT_STACK: [EntityTypeRef; 4] = [&vanilla_entities::TNT_MINECART; 4];
const RIDEABLE_STACK: [EntityTypeRef; 4] = [&vanilla_entities::MINECART; 4];
const MIXED_STACK: [EntityTypeRef; 4] = [
    &vanilla_entities::MINECART,
    &vanilla_entities::TNT_MINECART,
    &vanilla_entities::MINECART,
    &vanilla_entities::TNT_MINECART,
];

fn spawn_at(world: &Arc<World>, kind: EntityTypeRef, position: DVec3) -> SharedEntity {
    let entity = create_entity_instance(world, kind, position).expect("registered entity type");
    world
        .try_add_entity(Arc::clone(&entity))
        .expect("loaded chunk");
    entity
}

struct Stack {
    fixture: TestWorld,
    carts: Vec<SharedEntity>,
}

impl Stack {
    fn new(kinds: &[EntityTypeRef]) -> Self {
        let chunk = ChunkPos::from_block_pos(CART_BLOCK_POS);
        let fixture = test_world("minecart_stack_replay", chunk);
        let world = &fixture.world;
        world.update_entity_chunk_visibility(chunk, EntityVisibility::Ticking);
        world.set_tick_runs_normally(true);
        for x in 4..=12 {
            let pos = BlockPos::new(x, CART_BLOCK_POS.y(), CART_BLOCK_POS.z());
            set_block(world, pos.below(), vanilla_blocks::STONE.default_state());
            set_block(
                world,
                pos,
                vanilla_blocks::RAIL
                    .default_state()
                    .set_value(&BlockStateProperties::RAIL_SHAPE, RailShape::EastWest),
            );
        }
        let carts = kinds
            .iter()
            .map(|&kind| spawn_at(world, kind, CART_BLOCK_POS.get_bottom_center()))
            .collect();
        Self { fixture, carts }
    }

    fn tick_and_compare(&self, reference: &Self, tick: i32) -> usize {
        let (dirty, skips) = measure_cart_stack_skips(true, || {
            self.fixture
                .world
                .entity_manager()
                .tick_entities(tick, true)
        });
        let (expected, reference_skips) = measure_cart_stack_skips(false, || {
            reference
                .fixture
                .world
                .entity_manager()
                .tick_entities(tick, true)
        });
        assert_eq!(reference_skips, 0);
        assert_eq!(dirty, expected);
        for (a, b) in self.carts.iter().zip(&reference.carts) {
            assert_eq!(a.tick_count(), tick);
            assert_eq!(
                a.position().to_array().map(f64::to_bits),
                b.position().to_array().map(f64::to_bits)
            );
            assert_eq!(
                a.velocity().to_array().map(f64::to_bits),
                b.velocity().to_array().map(f64::to_bits)
            );
            assert_eq!(a.rotation(), b.rotation());
            assert_eq!(a.needs_velocity_sync(), b.needs_velocity_sync());
            assert_eq!(a.horizontal_collision(), b.horizontal_collision());
            assert_eq!(a.entity_type(), b.entity_type());
            let fuse = |cart: &SharedEntity| {
                cart.downcast_ref::<TntMinecartEntity>()
                    .and_then(TntMinecartEntity::fuse)
            };
            assert_eq!(fuse(a), fuse(b));
            assert_eq!(a.is_on_rails(), b.is_on_rails());
            assert_eq!(a.removal_reason(), b.removal_reason());
        }
        skips
    }
}

#[test]
fn same_cell_offset_then_restacking_matches_ordered_tick() {
    for kinds in [&TNT_STACK, &RIDEABLE_STACK, &MIXED_STACK] {
        check_restacking(kinds);
    }
}

fn check_restacking(kinds: &[EntityTypeRef]) {
    let actual = Stack::new(kinds);
    let reference = Stack::new(kinds);
    for stack in [&actual, &reference] {
        for cart in &stack.carts {
            if let Some(tnt) = cart.downcast_ref::<TntMinecartEntity>() {
                tnt.prime_fuse(None);
                assert!(tnt.is_primed());
            }
        }
    }
    for tick in 1..=2 {
        let skips = actual.tick_and_compare(&reference, tick);
        if tick == 2 {
            assert_eq!(skips, kinds.len());
        }
    }
    for stack in [&actual, &reference] {
        let cart = &stack.carts[3];
        cart.try_set_position(cart.position() + DVec3::X * 0.375)
            .expect("same-cell offset");
        cart.set_velocity(-DVec3::X * 0.125);
    }
    actual.tick_and_compare(&reference, 3);
    assert!(
        actual.carts[..3]
            .iter()
            .any(|cart| cart.velocity().x != 0.0),
        "previously stationary cart received an impulse"
    );
    actual.tick_and_compare(&reference, 4);

    for stack in [&actual, &reference] {
        let target = stack.carts[0].position();
        for cart in &stack.carts {
            cart.try_set_position(target).expect("restack");
            cart.set_velocity(DVec3::ZERO);
        }
    }
    actual.tick_and_compare(&reference, 5);
    assert!(actual.tick_and_compare(&reference, 6) > 0);
}

// A custom implementation may store semantic effects in push_entity even when
// both centers are exactly equal. Its registry ID alone cannot certify a stack.
struct ForeignCart {
    base: EntityBase,
    contacts: SyncMutex<Vec<i32>>,
    next_position: SyncMutex<Option<DVec3>>,
}

crate::entity::impl_test_downcast_type!(ForeignCart);

impl Entity for ForeignCart {
    fn base(&self) -> &EntityBase {
        &self.base
    }
    fn entity_type(&self) -> EntityTypeRef {
        &vanilla_entities::TNT_MINECART
    }
    fn is_pushable(&self) -> bool {
        true
    }
    fn push_entity(&self, target: &dyn Entity) {
        self.contacts.lock().push(target.id());
    }
    fn tick(self: Arc<Self>) {
        let next_position = self.next_position.lock().take();
        if let Some(position) = next_position {
            self.try_set_position(position).expect("move foreign cart");
        }
    }
}

fn add_foreign(stack: &Stack, position: DVec3) -> Arc<ForeignCart> {
    let world = &stack.fixture.world;
    let cart = Arc::new(ForeignCart {
        base: EntityBase::new(
            next_entity_id(),
            position,
            vanilla_entities::TNT_MINECART.dimensions,
            Arc::downgrade(world),
        ),
        contacts: SyncMutex::new(Vec::new()),
        next_position: SyncMutex::new(None),
    });
    world
        .try_add_entity(Arc::clone(&cart).into_shared_entity())
        .expect("live foreign cart");
    cart
}

#[test]
fn foreign_same_type_spawn_and_removal_restore_shortcut() {
    let actual = Stack::new(&MIXED_STACK);
    let reference = Stack::new(&MIXED_STACK);
    actual.tick_and_compare(&reference, 1);
    assert!(actual.tick_and_compare(&reference, 2) > 0);
    let a = add_foreign(&actual, actual.carts[0].position());
    let b = add_foreign(&reference, reference.carts[0].position());
    assert_eq!(actual.tick_and_compare(&reference, 3), 0);
    for (stack, foreign) in [(&actual, &a), (&reference, &b)] {
        let ids: Vec<_> = stack.carts.iter().map(|cart| cart.id()).collect();
        assert_eq!(*foreign.contacts.lock(), ids);
    }
    for foreign in [&a, &b] {
        foreign.set_removed(RemovalReason::Discarded);
    }
    actual.tick_and_compare(&reference, 4);
    assert!(actual.tick_and_compare(&reference, 5) > 0);
    assert_eq!(a.contacts.lock().len(), 4);
}

#[test]
fn entering_the_stack_invalidates_the_shortcut_within_the_same_tick() {
    let mut actual = Stack::new(&MIXED_STACK);
    let mut reference = Stack::new(&MIXED_STACK);
    let [a, b] = [&mut actual, &mut reference].map(|stack| {
        let foreign = add_foreign(stack, stack.carts[0].position() + 6.0 * DVec3::X);
        // The final native cart ticks immediately after the foreign cart moves.
        stack.carts.push(spawn_at(
            &stack.fixture.world,
            &vanilla_entities::MINECART,
            CART_BLOCK_POS.get_bottom_center(),
        ));
        foreign
    });
    actual.tick_and_compare(&reference, 1);
    assert!(actual.tick_and_compare(&reference, 2) > 0);
    for (stack, foreign) in [(&actual, &a), (&reference, &b)] {
        assert!(foreign.contacts.lock().is_empty());
        *foreign.next_position.lock() = Some(stack.carts[0].position());
    }
    assert_eq!(
        actual.tick_and_compare(&reference, 3),
        actual.carts.len() - 1
    );
    for (stack, foreign) in [(&actual, &a), (&reference, &b)] {
        let last = stack.carts.last().expect("cart after move");
        assert_eq!(foreign.contacts.lock().as_slice(), [last.id()]);
    }
}

#[test]
fn registered_part_in_stack_with_distant_parent_falls_back() {
    let actual = Stack::new(&MIXED_STACK);
    let reference = Stack::new(&MIXED_STACK);
    actual.tick_and_compare(&reference, 1);
    assert!(actual.tick_and_compare(&reference, 2) > 0);
    let parents = [&actual, &reference].map(|stack| {
        let world = &stack.fixture.world;
        let center = stack.carts[0].position();
        let parent = MultipartTestEntity::shared(center + 6.0 * DVec3::X, Arc::downgrade(world));
        parent.parts()[0].set_part_position(center);
        world
            .try_add_entity(Arc::clone(&parent))
            .expect("multipart parent in the same chunk");
        let hitbox = stack.carts[0].bounding_box();
        assert!(!hitbox.intersects(parent.bounding_box()));
        assert!(hitbox.intersects(parent.parts()[0].bounding_box()));
        parent
    });
    assert_eq!(actual.tick_and_compare(&reference, 3), 0);
    for parent in parents {
        parent.set_removed(RemovalReason::Discarded);
    }
    assert!(actual.tick_and_compare(&reference, 4) > 0);
}

#[test]
fn moving_rideable_cart_picks_up_a_chicken_during_its_tick() {
    let pair = [&vanilla_entities::MINECART; 2];
    let actual = Stack::new(&pair);
    let reference = Stack::new(&pair);
    actual.tick_and_compare(&reference, 1);
    assert_eq!(actual.tick_and_compare(&reference, 2), pair.len());

    let [a, b] = [&actual, &reference].map(|stack| {
        let cart = &stack.carts[0];
        let chicken = spawn_at(
            &stack.fixture.world,
            &vanilla_entities::CHICKEN,
            cart.position() + 0.5 * DVec3::X,
        );
        chicken.as_mob().expect("chicken").set_no_ai(true);
        cart.set_velocity(0.25 * DVec3::X);
        chicken
    });
    for tick in 3..=4 {
        actual.tick_and_compare(&reference, tick);
        for (stack, chicken) in [(&actual, &a), (&reference, &b)] {
            let cart = &stack.carts[0];
            assert!(cart.has_passenger(chicken.as_ref()));
            assert_eq!(chicken.vehicle().expect("mounted chicken").id(), cart.id());
        }
        assert_eq!(a.position(), b.position());
        assert_eq!(a.velocity(), b.velocity());
        assert_eq!(a.tick_count(), b.tick_count());
    }
}
