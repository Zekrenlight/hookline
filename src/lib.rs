//! Hookline 1.0.0 — a Sunrise client fishing helper.
//!
//! The launcher compiles this crate against the pinned game build.
//! Logic tests under `tests/` do not need a running client.

pub mod logic;
mod panel;
mod session;
mod world;

#[cfg(test)]
mod rules_tests;

pub use logic::*;
pub use logic::listing;

use crate::world::{Game, HasHost, View};
use crate::world::SETTING_KEYS;
use mod_api::{
    ClientHost, ClientTickPreEvent, Command, CommandArgumentValue, Flow, GuiLayerPlacement, Host,
    HostSide, Hooks, Identifier, KeyRegistration, PlayLevelSoundAtEntityEvent,
    PlayLevelSoundAtPositionEvent, RegisterClientCommandsEvent, RegisterGuiLayersEvent, Registrar,
    RegistrationContext, ScreenCharTypedPreEvent, ScreenKeyPressedPreEvent,
    ScreenMouseButtonPressedPreEvent, ScreenMouseButtonReleasedPreEvent,
    ScreenMouseDraggedPreEvent, ScreenMouseScrolledPreEvent, ScreenRenderPostEvent,
    SettingChangedEvent, SettingKey, SettingSpec, SettingValue,
};

const OVERLAY_LAYER: &str = "hookline:overlay";

#[derive(Default)]
struct Hookline;

#[mod_api::sunrise_mod]
impl Hooks for Hookline {
    fn register(&self, registrar: &mut dyn Registrar, _cx: &RegistrationContext) {
        registrar.register_key_mapping(
            KeyRegistration::new("toggle", "key.keyboard.v").label("hookline.key.toggle"),
        );
        registrar.register_key_mapping(
            KeyRegistration::new("panel", "key.keyboard.o").label("hookline.key.panel"),
        );
        // Defaults match Prefs::default(). SettingKind has no button, so opening
        // the panel stays a keybind or `/hookline panel` (not an Options click).
        let mut habit = std::collections::HashMap::<&str, Option<SettingKey>>::new();
        for (key, default) in [
            ("pace", true),
            ("wander", false),
            ("meals", false),
            ("worn_rod", false),
            ("bail_hurt", true),
            ("rest_after_lull", true),
            ("overlay", false),
        ] {
            let k = registrar
                .register_setting(SettingSpec::toggle(
                    key,
                    format!("hookline.setting.{key}"),
                    default,
                ))
                .ok();
            habit.insert(key, k);
        }
        // The accent colour of hookline's cards (Sunrise orange by default).
        let accent = registrar
            .register_setting(
                mod_api::SettingSpec::color("accent", "hookline.setting.accent", 0xF9801D)
                    .tooltip("hookline.setting.accent.tooltip"),
            )
            .ok();
        let _ = SETTING_KEYS.set(crate::world::SettingKeys {
            pace: habit.get("pace").copied().flatten(),
            wander: habit.get("wander").copied().flatten(),
            meals: habit.get("meals").copied().flatten(),
            worn_rod: habit.get("worn_rod").copied().flatten(),
            bail_hurt: habit.get("bail_hurt").copied().flatten(),
            rest_after_lull: habit.get("rest_after_lull").copied().flatten(),
            overlay: habit.get("overlay").copied().flatten(),
            accent,
        });
    }

    fn register_client_commands(
        &self,
        _host: &mut dyn ClientHost,
        event: &mut RegisterClientCommandsEvent<'_>,
    ) {
        event.commands.push(
            Command::new("hookline [sub:text]").executes_client(|host, inv| {
                let rest = match inv.argument("sub") {
                    Some(CommandArgumentValue::String(s)) => s.clone(),
                    _ => String::new(),
                };
                let mut game = Game::new(host);
                if !game.native_on() {
                    return Some(0);
                }
                session::ensure_ready(&game);
                let mut ran = false;
                let cmd = if rest.is_empty() {
                    "hookline".to_string()
                } else {
                    format!("hookline {rest}")
                };
                session::handle_command(&mut game, &cmd, &mut ran);
                session::apply_native_settings(&mut game);
                Some(if ran { 1 } else { 0 })
            }),
        );
    }

    fn register_gui_layers(
        &self,
        _context: &RegistrationContext,
        event: &mut RegisterGuiLayersEvent<'_>,
    ) {
        let _ = event.layers.register(
            OVERLAY_LAYER.into(),
            GuiLayerPlacement::Above("minecraft:overlay_message".into()),
            Box::new(|host, draw| {
                let game = View::new(host);
                if !game.native_on() {
                    return;
                }
                if let Some(line) = session::overlay_line() {
                    draw.text(&line, [4.0, 4.0], 0xFFE8DCC8, true);
                }
            }),
        );
    }

    fn client_tick_pre(&self, host: &mut dyn ClientHost, _event: &mut ClientTickPreEvent) {
        {
            let game = Game::new(host);
            if !game.native_on() {
                return;
            }
            session::ensure_ready(&game);
            session::revise(session::tick_flash);
        }
        let playing = host.playing();
        let flip = playing && consume_mod_click(host, session::MOD_ID, "toggle");
        let panel = playing && consume_mod_click(host, session::MOD_ID, "panel");
        let mut game = Game::new(host);
        if flip {
            session::flip_from_key(&mut game);
        }
        if panel {
            if game.native_mod_screen() == Some(session::MOD_ID) {
                game.native_mod_screen_close(session::MOD_ID);
            } else {
                crate::panel::open(&mut game);
            }
        }
        session::drive_tick(&mut game);
        session::apply_native_settings(&mut game);
        crate::panel::close_if_asked(&mut game);
    }

    fn play_level_sound_at_position(
        &self,
        host: HostSide<'_>,
        event: &mut PlayLevelSoundAtPositionEvent<'_>,
    ) -> Flow {
        maybe_splash(host, event.sound, event.position);
        Flow::Continue
    }

    fn play_level_sound_at_entity(
        &self,
        host: HostSide<'_>,
        event: &mut PlayLevelSoundAtEntityEvent<'_>,
    ) -> Flow {
        let pos = match &host {
            HostSide::Client(h) => event.entity.position(*h),
            HostSide::Server(h) => event.entity.position(*h),
        }
        .unwrap_or([0.0, 0.0, 0.0]);
        maybe_splash(host, event.sound, pos);
        Flow::Continue
    }


    fn screen_render_post(&self, host: &dyn ClientHost, event: &mut ScreenRenderPostEvent<'_>) {
        let game = View::new(host);
        if !game.native_on() {
            return;
        }
        session::ensure_ready(&game);
        crate::panel::set_accent(
            SETTING_KEYS
                .get()
                .and_then(|k| k.accent)
                .and_then(|k| host.setting_color(k).ok())
                .unwrap_or(0xF9801D),
        );
        crate::panel::paint(&game, event.draw, event.screen);
    }

    fn screen_mouse_button_pressed_pre(
        &self,
        host: &dyn ClientHost,
        event: &mut ScreenMouseButtonPressedPreEvent<'_>,
    ) -> Flow {
        let game = View::new(host);
        cancel_if(panel::click(&game, event.screen, event.button), event)
    }

    fn screen_mouse_button_released_pre(
        &self,
        host: &dyn ClientHost,
        event: &mut ScreenMouseButtonReleasedPreEvent<'_>,
    ) -> Flow {
        let game = View::new(host);
        cancel_if(panel::release(&game, event.screen, event.button), event)
    }

    fn screen_mouse_scrolled_pre(
        &self,
        host: &dyn ClientHost,
        event: &mut ScreenMouseScrolledPreEvent<'_>,
    ) -> Flow {
        let game = View::new(host);
        cancel_if(panel::wheel(&game, event.screen, event.scroll), event)
    }

    fn screen_mouse_dragged_pre(
        &self,
        host: &dyn ClientHost,
        event: &mut ScreenMouseDraggedPreEvent<'_>,
    ) -> Flow {
        let game = View::new(host);
        cancel_if(panel::drag(&game, event.screen, event.button), event)
    }

    fn screen_key_pressed_pre(
        &self,
        host: &dyn ClientHost,
        event: &mut ScreenKeyPressedPreEvent<'_>,
    ) -> Flow {
        let game = View::new(host);
        cancel_if(panel::key(&game, event.screen, event.key), event)
    }

    fn screen_char_typed_pre(
        &self,
        host: &dyn ClientHost,
        event: &mut ScreenCharTypedPreEvent<'_>,
    ) -> Flow {
        let game = View::new(host);
        cancel_if(panel::typed(&game, event.screen, event.c), event)
    }

    fn setting_changed(&self, host: &dyn Host, event: &mut SettingChangedEvent<'_>) {
        if event.mod_id != session::MOD_ID {
            return;
        }
        if event.key == "accent" {
            if let SettingValue::Color(rgb) = event.value {
                crate::panel::set_accent(rgb);
            }
            return;
        }
        // Load the file first if this is the first notice, using Host I/O only
        // (this hook no longer lends a client host).
        session::ensure_ready_host(host);
        session::revise(|s| {
            if let SettingValue::Toggle(on) = event.value {
                session::set_habit(s, event.key, on);
            }
            session::write_prefs_host(host, s);
        });
    }
}


fn consume_mod_click(host: &mut dyn ClientHost, owner: &str, id: &str) -> bool {
    host.mod_key_mapping(owner, id)
        .and_then(|mapping| mapping.consume_click(host))
        .unwrap_or(false)
}

fn cancel_if(handled: bool, event: &mut impl Cancel) -> Flow {
    if handled {
        event.cancel();
        Flow::Cancel
    } else {
        Flow::Continue
    }
}

trait Cancel {
    fn cancel(&mut self);
}

macro_rules! impl_cancel {
    ($($t:ty),+) => {$(
        impl Cancel for $t {
            fn cancel(&mut self) {
                self.cancelled = true;
            }
        }
    )+};
}

impl_cancel!(
    ScreenMouseButtonPressedPreEvent<'_>,
    ScreenMouseButtonReleasedPreEvent<'_>,
    ScreenMouseScrolledPreEvent<'_>,
    ScreenMouseDraggedPreEvent<'_>,
    ScreenKeyPressedPreEvent<'_>,
    ScreenCharTypedPreEvent<'_>
);

fn maybe_splash(host: HostSide<'_>, sound: &Identifier, position: [f64; 3]) {
    if !crate::logic::splash_sound(&sound.to_string()) {
        return;
    }
    let HostSide::Client(host) = host else {
        return;
    };
    let mut game = Game::new(host);
    session::on_splash_at(&mut game, position);
}
