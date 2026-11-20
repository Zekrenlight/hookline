# Hookline

Version 1.0.0 · Minecraft Java 26.3 · Sunrise API 0.2 · Mod SDK `9425651` · MIT licence

Hookline is written entirely in Rust for Sunrise, a Rust client for Minecraft: Java Edition 26.3. No Java or Fabric required. It reels when **your** bobber splashes, recasts, and gives you a keep/toss list, enchant rules, optional meals, and a deposit pass into a chest you let it open.

This is a **client-side AFK fishing helper**. It drives ordinary client actions (the use key, hotbar selection, and inventory clicks) to reel, recast, throw discarded catch, deposit kept catch, and eat. It does not use a custom server protocol. Use only on servers and worlds where AFK fishing and this kind of automation are allowed.

This is an independent implementation of autofishing.

<img width="919" height="636" alt="image" src="https://github.com/user-attachments/assets/6f76b98a-db80-4306-af1b-bd603116f41d" />
<img width="919" height="636" alt="image" src="https://github.com/user-attachments/assets/ba02b446-3da5-40de-8435-56d0dfa0847a" />
<img width="919" height="636" alt="image" src="https://github.com/user-attachments/assets/761afff7-6e8f-4bb1-a100-65c77a481100" />
<img width="919" height="636" alt="image" src="https://github.com/user-attachments/assets/b027f09e-26b9-4c92-95ad-fe63bd785833" />


## Features

- Toggle with **V**; open the panel with **O** (rebindable under Options > Controls). The toggle shows a short action-bar message.
- Bite detection uses your own bobber splash, with a short settle window after a cast.
- Pace variance (on by default) waits a little before reeling and stretches recast and meal pauses with non-uniform jitter.
- Catch list order: Enchanted Book, Fishing Rod, Bow, Salmon, Cod, Tropical Fish, Pufferfish, then the rest of the usual catch. Each row is Keep or Toss. The in-use rod is never thrown.
- Books, rods, and bows also use per-enchant rules. Mending wins if it is present; otherwise the closest-to-max vote wins; a tie keeps the item. Unlisted enchants do not vote. The item row is the master switch.
- Tosses go through inventory-click actions, including items in the main inventory.
- When the bag is almost full, Hookline can deposit **kept catch only** into a chest or barrel it opened, then cool down.
- Optional meals: looks in the off-hand, then the hotbar only, and eats one item at a time until hunger is full, with a pause between bites. After the last bite it waits before grabbing the rod and again before recasting. Swap and use never share a tick.
- Optional idle rest after five minutes with no bite, reel, or recast. Meals do not reset or count as idle time.
- Optional look-around, switch or stop on a worn rod, and leave the server below three hearts.
- Session ledger and per-item catch chimes.

Habits also appear as toggles in Hookline's settings card (SunnyMods' **Settings** button, Sunrise's Mods list, or Options > Mods). The fishing panel itself opens with **O** or `/hookline panel`.

## Screenshots

<img width="1381" height="959" alt="image" src="https://github.com/user-attachments/assets/2f0dd17d-aae5-48d3-9899-e9a07fe06620" />
<img width="1381" height="959" alt="image" src="https://github.com/user-attachments/assets/0bd11d12-4ce6-44bc-bcf7-b7e7f2268d8e" />
<img width="1381" height="959" alt="image" src="https://github.com/user-attachments/assets/aa82d264-6917-449a-b1fc-1bd6ea71f41b" />
<img width="1381" height="959" alt="image" src="https://github.com/user-attachments/assets/88e8c7d7-e675-436d-9970-95a92ae4a859" />

## Install (local `.smod`)

1. Build or download `hookline-1.0.0.smod`.
2. In **sunrise-launcher**, use **Add from file** and pick that file, or use **Add from GitHub** (`sunrise-launcher mods add-github`) and point it at this repository; the launcher compiles the source against the pin in `sunrise-mod.toml`.
3. Start Sunrise 26.3, turn Hookline on in the mods list if it is not already on, join a world, press **V** to start fishing and **O** to open the panel.

Sunrise native mods are compiled and loaded by sunrise-launcher. Dropping the `.smod` into a vanilla `mods/` folder of another launcher will not load it; add it with sunrise-launcher first, then start that Sunrise install however you like. Native mods run with full access to your computer; read the source first.

## Keybinds

| Action | Default |
| --- | --- |
| Turn Hookline on or off | V |
| Open or close the panel | O |

Chat commands use the prefix `/hookline`: `toggle`, `pace`, `panel`, `ledger`, `clear-ledger`, `toss`, `chime`, and `help`.

Settings are stored in `hookline.cfg` under the mod data directory (format version 1). There is no automatic import of another mod's config.

## Menu

The panel is a centered tabbed card:

- **Loot** — the twenty-one catch items, each with Keep/Toss and Chime/Quiet chips, a search field, and a scrollbar
- **Enchants** — per-enchant Toss / Any / minimum-level chips, with a scrollbar
- **Behavior** — on/off options (pace variance, look-around, meals, worn rod, leave if hurt, idle auto-off, overlay)
- **Stats** — the session ledger and a clear-ledger control

The panel uses the shared Sunrise dark look: a `#0F1118` card with a 2 px accent edge, `#161A22` tabs with the selected one in a darkened accent, charcoal chips, and warm `#E8DCC8` text. Keep and Toss stay green and red so they read at a glance. The accent is Sunrise orange `#F9801D` at first; change it with **Accent colour** in Hookline's settings card (eight presets, or Reset). **Key binds › Rebind** in that card switches to Sunrise's classic page to change keys.

## Compatibility

| Piece | Pin |
| --- | --- |
| Minecraft Java | 26.3 |
| Sunrise API | 0.2 |
| Mod SDK (`[native].sunrise`) | `94256510f74e6843754e0a93fd9e4da7e45b1056` (`9425651`, 19 Nov 2026) |
| Mixins (`[native].mixins`) | `e59da9094f0c2fdda6c627fe9eff22dec6a906f0` |

See also [COMPAT.md](COMPAT.md).

Hookline works on its own and alongside the other Sunrise mods by Zekrenlight. Its settings-card
hooks act only on its own Options page, so several mods can be installed together without any
conflicting replacements. A mod with native hooks works only on the exact SDK build it was made for.

## Building

```sh
cargo sunrise-mod           # fetches the pinned SDK into .sunrise/ (gitignored)
cargo sunrise-mod check     # type-checks against the pinned SDK
cargo test --lib            # unit tests, no game needed
cargo sunrise-mod build     # target/sunrise-mod/hookline.smod
```

Rename the output to `hookline-1.0.0.smod` for a release.

To move to a newer SDK, run `cargo sunrise-mod update --sunrise <build>` (it rewrites the
`sunrise` and `mixins` lines in `sunrise-mod.toml`), then check and rebuild.

Install the `.smod` with sunrise-launcher (**Add from file**), or point **Add from GitHub** at
this repository. Native mods run with full access to your computer; read the source first.

## License

MIT © 2026 Zekrenlight. See [LICENSE](LICENSE).
