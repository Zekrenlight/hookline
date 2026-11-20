// SPDX-License-Identifier: MIT
// Keys on Hookline's settings card; Escape (search let go) is Sunrise's, to leave.
fn hookline_card_options_key(
    game: &mut crate::game::Game<'_>,
    key: KeyCode,
    text: Option<&str>,
) -> ::core::ops::ControlFlow<Result<()>> {
    if !crate::game::hookline_card::active(game) {
        return ::core::ops::ControlFlow::Continue(());
    }
    match crate::game::hookline_card::key(game, key, text) {
        Some(done) => ::core::ops::ControlFlow::Break(done),
        None => ::core::ops::ControlFlow::Continue(()),
    }
}
