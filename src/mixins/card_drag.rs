// SPDX-License-Identifier: MIT
// Dragging on Hookline's settings card: its sliders and scrollbar.
fn hookline_card_options_drag(game: &mut crate::game::Game<'_>) -> ::core::ops::ControlFlow<Result<()>> {
    if !crate::game::hookline_card::active(game) {
        return ::core::ops::ControlFlow::Continue(());
    }
    ::core::ops::ControlFlow::Break(crate::game::hookline_card::drag(game))
}
