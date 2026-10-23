use super::*;
use steel_registry::data_components::vanilla_components::SwingAnimationType;
use steel_utils::translations::ARGUMENT_SWING_ANIMATION_INVALID;

#[test]
fn swing_animation_retains_typed_values_through_keyed_erasure() {
    let dispatcher = resource_dispatcher(SteelArgumentType::swing_animation());
    for (name, expected) in [
        ("none", SwingAnimationType::None),
        ("whack", SwingAnimationType::Whack),
        ("stab", SwingAnimationType::Stab),
    ] {
        let input = format!("resource {name}");
        let chain = dispatcher
            .context_chain(dispatcher.parse(&input, TestSource::new()))
            .expect("animation should parse");
        assert_eq!(chain.top_context().swing_animation("value"), Ok(expected));
        assert!(chain.top_context().time("value").is_err());
    }
}

#[test]
fn swing_animation_rejects_unknown_names_and_preserves_vanilla_error_context() {
    let dispatcher = resource_dispatcher(SteelArgumentType::swing_animation());
    for name in ["unknown", "WHACK", "\"stab\""] {
        let input = format!("resource {name}");
        let Err(error) = dispatcher.context_chain(dispatcher.parse(&input, TestSource::new()))
        else {
            panic!("invalid animation should fail");
        };
        assert_eq!(
            error.cursor(),
            Some(if name.starts_with('"') {
                9
            } else {
                input.len()
            })
        );
        let raw = if name.starts_with('"') { "" } else { name };
        assert_eq!(
            error.message_component(),
            ARGUMENT_SWING_ANIMATION_INVALID
                .message([raw.to_owned()])
                .component()
        );
    }
}

#[test]
fn swing_animation_suggestions_filter_case_insensitively() {
    let dispatcher = resource_dispatcher(SteelArgumentType::swing_animation());
    for (input, expected) in [
        ("resource ", vec!["none", "stab", "whack"]),
        ("resource S", vec!["stab"]),
        ("resource W", vec!["whack"]),
        ("resource invalid", vec![]),
    ] {
        let parse = dispatcher.parse(input, TestSource::new());
        let suggestions = dispatcher
            .completion_suggestions(&parse)
            .expect("suggestions should build");
        assert_eq!(
            suggestions
                .list()
                .iter()
                .map(Suggestion::text)
                .collect::<Vec<_>>(),
            expected
        );
    }
}
