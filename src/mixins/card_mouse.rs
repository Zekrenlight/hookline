// SPDX-License-Identifier: MIT
// The mouse buttons on Hookline's settings card.
fn hookline_card_options_mouse(
    game: &mut crate::game::Game<'_>,
    pressed: bool,
    right: bool,
) -> ::core::ops::ControlFlow<Result<()>> {
    if !crate::game::hookline_card::active(game) {
        return ::core::ops::ControlFlow::Continue(());
    }
    ::core::ops::ControlFlow::Break(crate::game::hookline_card::mouse(game, pressed, right))
}
