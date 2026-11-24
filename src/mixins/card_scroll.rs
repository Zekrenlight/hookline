// SPDX-License-Identifier: MIT
// The wheel on Hookline's settings card.
fn hookline_card_options_scroll(
    game: &mut crate::game::Game<'_>,
    amount: f32,
) -> ::core::ops::ControlFlow<Result<bool>> {
    if !crate::game::hookline_card::active(game) {
        return ::core::ops::ControlFlow::Continue(());
    }
    ::core::ops::ControlFlow::Break(crate::game::hookline_card::scroll(game, amount).map(|()| true))
}
