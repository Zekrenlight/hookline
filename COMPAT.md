# Compatibility

Hookline 1.0.0 is a Sunrise native client mod (`.smod`).

| Piece | Pin |
| --- | --- |
| Minecraft Java | 26.3 |
| Sunrise API | 0.2 |
| Game build | 26.3 (`[native].game` in `sunrise-mod.toml`) |
| Mod SDK (`[native].sunrise`) | `94256510f74e6843754e0a93fd9e4da7e45b1056` (`9425651`, 19 Nov 2026) |
| Mixins (`[native].mixins`) | `e59da9094f0c2fdda6c627fe9eff22dec6a906f0` |

Fishing, the panel and the commands use the typed Host / Hooks API only. The settings card
is one small native module (`src/native/card.rs`) with cancellable `HEAD` hooks on the Options
screen's draw, mouse, drag, scroll and key handlers. They act only while Hookline's own
Options page is open, so Hookline works on its own and alongside other Sunrise mods (including
SunnyMods) without conflicting replacements.

To move to a newer SDK, run `cargo sunrise-mod update --sunrise <build>` (it rewrites the
`sunrise` and `mixins` lines in `sunrise-mod.toml`), then `cargo sunrise-mod check`,
`cargo test --lib` and `cargo sunrise-mod build`.
