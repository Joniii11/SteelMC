//! Handler for the "difficulty" command.
use crate::command::commands::{
    CommandExecutor, CommandHandlerDyn, CommandParserLeafExecutor, DynCommandHandler, literal,
};
use crate::command::context::CommandContext;
use crate::command::error::CommandError;
use std::marker::PhantomData;
use steel_protocol::packets::game::CChangeDifficulty;
use steel_utils::translations;
use steel_utils::types::Difficulty;
use text_components::TextComponent;
use text_components::translation::Translation;

/// Handler for the "difficulty" command.
#[must_use]
pub fn command_handler() -> impl CommandHandlerDyn {
    let mut handler = DynCommandHandler::new(
        &["difficulty"],
        "Gets or sets the world difficulty.",
        "minecraft:command.difficulty",
    );

    for difficulty in [
        Difficulty::Peaceful,
        Difficulty::Easy,
        Difficulty::Normal,
        Difficulty::Hard,
    ] {
        handler = handler
            .then(literal(difficulty_key(difficulty)).executes(SetDifficultyExecutor(difficulty)));
    }

    handler = handler.then(CommandParserLeafExecutor {
        executor: QueryDifficultyExecutor,
        _source: PhantomData::<()>,
    });

    handler
}

/// Returns the string value of a Difficulty
const fn difficulty_key(difficulty: Difficulty) -> &'static str {
    match difficulty {
        Difficulty::Peaceful => "peaceful",
        Difficulty::Easy => "easy",
        Difficulty::Normal => "normal",
        Difficulty::Hard => "hard",
    }
}

/// Translation key helper fn
fn difficulty_display_name(difficulty: Difficulty) -> &'static Translation<0> {
    match difficulty {
        Difficulty::Peaceful => &translations::OPTIONS_DIFFICULTY_PEACEFUL,
        Difficulty::Easy => &translations::OPTIONS_DIFFICULTY_EASY,
        Difficulty::Normal => &translations::OPTIONS_DIFFICULTY_NORMAL,
        Difficulty::Hard => &translations::OPTIONS_DIFFICULTY_HARD,
    }
}

/// The query for the current difficulty
struct QueryDifficultyExecutor;

impl CommandExecutor<()> for QueryDifficultyExecutor {
    fn execute(&self, _args: (), context: &mut CommandContext) -> Result<(), CommandError> {
        let difficulty = context.world.get_difficulty();
        let display_name = difficulty_display_name(difficulty);

        context.sender.send_message(
            &translations::COMMANDS_DIFFICULTY_QUERY
                .message([TextComponent::from(display_name)])
                .into(),
        );

        Ok(())
    }
}

/// Sets the world difficulty to the specified value
struct SetDifficultyExecutor(Difficulty);

impl CommandExecutor<()> for SetDifficultyExecutor {
    fn execute(&self, _args: (), context: &mut CommandContext) -> Result<(), CommandError> {
        let difficulty = self.0;
        let current = context.world.get_difficulty();

        if current == difficulty {
            return Err(CommandError::CommandFailed(Box::new(
                translations::COMMANDS_DIFFICULTY_FAILURE
                    .message([TextComponent::plain(difficulty_key(difficulty))])
                    .into(),
            )));
        }

        context.world.set_difficulty(difficulty);

        let locked = context.world.is_difficulty_locked();
        context
            .world
            .broadcast_to_all(CChangeDifficulty { difficulty, locked });

        let display_name = difficulty_display_name(difficulty);
        context.sender.send_message(
            &translations::COMMANDS_DIFFICULTY_SUCCESS
                .message([TextComponent::from(display_name)])
                .into(),
        );

        Ok(())
    }
}
