//! Default item behavior implementation.

use crate::behavior::{InteractionResult, ItemBehavior, UseOnContext};

/// Vanilla's base item behavior.
///
/// Shared component-driven behavior belongs on [`ItemBehavior`]'s default
/// methods so specialized items inherit it too.
pub struct DefaultItemBehavior;

impl ItemBehavior for DefaultItemBehavior {
    /// Vanilla `Item.useOn`: component-driven block transformations apply to
    /// every plain item, including tools.
    fn use_on(&self, context: &mut UseOnContext) -> InteractionResult {
        super::block_transformer::use_on(context)
    }
}
