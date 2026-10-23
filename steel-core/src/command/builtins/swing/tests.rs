use std::sync::{Arc, Weak};

use glam::DVec3;
use steel_registry::{
    data_components::vanilla_components::SwingAnimationType, init_vanilla_registry,
    vanilla_entities,
};

use super::*;
use crate::{
    command::builtins::create_dispatcher, entity::entities::PigEntity, test_support::TestEntity,
};

fn pig(id: i32) -> SharedEntity {
    Arc::new(PigEntity::new(
        &vanilla_entities::PIG,
        id,
        DVec3::ZERO,
        Weak::new(),
    ))
}

#[test]
fn mixed_targets_only_count_living_entities_and_keep_single_target_feedback() {
    init_vanilla_registry();
    let pig = pig(1);
    let nonliving = TestEntity::shared(2, DVec3::ZERO, Weak::new(), &vanilla_entities::ITEM);
    let animation = SwingAnimation::new(SwingAnimationType::Stab, 13);
    let (count, message) = swing_entities(
        &[nonliving, Arc::clone(&pig)],
        InteractionHand::OffHand,
        animation,
    )
    .expect("living target should swing");
    assert_eq!(count, 1);
    assert_eq!(
        message,
        translations::COMMANDS_SWING_SUCCESS_SINGLE
            .message([pig.display_name()])
            .component()
    );
    let living = pig.as_living_entity().expect("pig is living");
    assert_eq!(
        living.living_swing_state().swinging_arm(),
        Some(InteractionHand::OffHand)
    );
    for _ in 0..=animation.duration {
        living.living_base().tick_swing_state();
    }
    assert!(living.living_swing_state().swinging());
    living.living_base().tick_swing_state();
    assert!(!living.living_swing_state().swinging());
}

#[test]
fn ongoing_swings_still_count_towards_command_success() {
    init_vanilla_registry();
    let targets = [pig(1), pig(2)];
    let animation = SwingAnimation::DEFAULT;
    for target in &targets {
        assert!(target.as_living_entity().expect("pig is living").swing(
            InteractionHand::MainHand,
            animation,
            false
        ));
        target
            .as_living_entity()
            .expect("pig is living")
            .living_base()
            .tick_swing_state();
    }
    let (count, message) = swing_entities(&targets, InteractionHand::OffHand, animation)
        .expect("ongoing swings count");
    assert_eq!(count, 2);
    assert_eq!(
        message,
        translations::COMMANDS_SWING_SUCCESS_MULTIPLE
            .message(["2"])
            .component()
    );
    for target in targets {
        assert_eq!(
            target
                .as_living_entity()
                .expect("pig is living")
                .living_swing_state()
                .swinging_arm(),
            Some(InteractionHand::MainHand)
        );
    }
}

#[test]
fn exclusively_nonliving_targets_fail_with_vanilla_error() {
    init_vanilla_registry();
    let target = TestEntity::shared(1, DVec3::ZERO, Weak::new(), &vanilla_entities::ITEM);
    let error = swing_entities(
        &[target],
        InteractionHand::MainHand,
        SwingAnimation::DEFAULT,
    )
    .expect_err("nonliving targets cannot swing");
    assert_eq!(
        error.message_component(),
        TextComponent::from(&translations::COMMANDS_SWING_FAILED_NOTLIVING)
    );
}

#[test]
fn command_graph_supports_optional_animation_and_positive_time_for_both_hands() {
    init_vanilla_registry();
    let dispatcher = create_dispatcher().expect("built-ins should register");
    for hand in ["mainhand", "offhand"] {
        let mut parent = dispatcher.root();
        for (name, expected) in [
            ("swing", None),
            ("targets", Some(SteelArgumentType::entities())),
            (hand, None),
            ("animation", Some(SteelArgumentType::swing_animation())),
            ("duration", Some(SteelArgumentType::time(1))),
        ] {
            let child = dispatcher
                .children(parent)
                .expect("parent exists")
                .iter()
                .copied()
                .find(|child| {
                    dispatcher
                        .node(*child)
                        .is_some_and(|node| node.name() == name)
                })
                .expect("command path exists");
            let node = dispatcher.node(child).expect("node exists");
            assert!(node.is_executable());
            assert_eq!(node.argument_type(), expected.as_ref());
            parent = child;
        }
    }
}
