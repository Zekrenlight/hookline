//! Host-facing adapter used by the session and panel.
//!
//! Keeps Hookline's inventory layout (slots 0-8 the hotbar, 9-35 the main bag)
//! and talks to Sunrise through `ClientHost` only. No mixin, no raw play packets.

use mod_api::{
    ClientHost, ConfigPath, DrawContext, EquipmentSlot, Hand, Identifier, ItemKey, ItemStack,
    JavaString, LivingRef, MenuClick, MenuRef, PlayerRef, SoundPlay, SoundSource, Tag,
};

pub const MOD_ID: &str = "hookline";

/// Shared read surface for `&dyn ClientHost` and `&mut dyn ClientHost` wrappers.
pub trait HasHost {
    fn host(&self) -> &dyn ClientHost;

    fn native_on(&self) -> bool {
        self.host().on(MOD_ID)
    }

    fn native_window(&self) -> bool {
        self.host().window_focused()
    }


    fn native_mod_screen(&self) -> Option<&str> {
        self.host().mod_screen()
    }

    fn native_setting(&self, _mod_id: &str, key: &str) -> Option<String> {
        self.host().setting_bool(key).map(|on| {
            if on {
                "yes".into()
            } else {
                "no".into()
            }
        })
    }

    fn native_set_setting(&self, mod_id: &str, key: &str, value: &str) {
        self.host().set_setting(mod_id, key, value);
    }

    fn native_mapping(&self, name: &str) -> Option<(bool, String)> {
        match self.host().mapping(name) {
            Ok(Some(m)) => Some((m.down, String::new())),
            _ => None,
        }
    }


    fn write_config(&self, file: &str, text: &str) {
        if let Ok(path) = ConfigPath::new(file) {
            let _ = self.host().data_files().write(&path, text.as_bytes());
        }
    }

    fn read_config(&self, file: &str) -> String {
        let Ok(path) = ConfigPath::new(file) else {
            return String::new();
        };
        self.host()
            .data_files()
            .read(&path)
            .ok()
            .and_then(|b| String::from_utf8(b).ok())
            .unwrap_or_default()
    }

    fn native_play_local_sound(
        &self,
        id: &str,
        pos: [f64; 3],
        volume: f32,
        pitch: f32,
        _delay: u32,
    ) {
        let sound = SoundPlay::at(id, SoundSource::Players, pos)
            .with_volume(volume)
            .with_pitch(pitch);
        let _ = self.host().play_local_sound(&sound, 0);
    }

    fn yaw(&self) -> f32 {
        self.host().camera().map(|c| c.yaw).unwrap_or(0.0)
    }

    fn pitch(&self) -> f32 {
        self.host().camera().map(|c| c.pitch).unwrap_or(0.0)
    }

    fn selected(&self) -> usize {
        self.host()
            .local_player()
            .and_then(|p| p.selected_slot(self.host()))
            .unwrap_or(0)
    }

    fn using(&self) -> Option<(i32, i32)> {
        None
    }

    fn offhand(&self) -> Option<Stack> {
        let living = local_living(self.host())?;
        let stack = living.hand_item(self.host(), Hand::Off)?;
        Some(to_stack(self.host(), &stack))
    }

    fn inventory(&self) -> Vec<Option<Stack>> {
        inventory_of(self.host())
    }

    fn player(&self) -> Option<PlayerView> {
        player_of(self.host())
    }

    fn native_player(&self) -> Option<PlayerView> {
        self.player()
    }

    fn hook(&self) -> Option<Hook> {
        hook_of(self.host())
    }

    fn on_ground(&self) -> bool {
        self.host()
            .local_player()
            .and_then(|p| p.entity().on_ground(self.host()))
            .unwrap_or(false)
    }

    fn block_id(&self, pos: [i32; 3]) -> String {
        let Some(player) = self.host().local_player() else {
            return String::new();
        };
        let Some(level) = player.level(self.host()) else {
            return String::new();
        };
        let Some(key) = level.block(self.host(), pos) else {
            return String::new();
        };
        key.id(self.host())
    }

    fn max_damage(&self, item: &str) -> u32 {
        let Some(key) = item_key_of(self.host(), item) else {
            return 0;
        };
        match self.host().item_default_component(key, "minecraft:max_damage") {
            Some(Tag::Int(v)) if v > 0 => v as u32,
            Some(Tag::Short(v)) if v > 0 => v as u32,
            _ => 0,
        }
    }

    fn is_food_item(&self, item: &str) -> bool {
        let Some(key) = item_key_of(self.host(), item) else {
            return false;
        };
        self.host()
            .item_default_component(key, "minecraft:food")
            .is_some()
    }

    fn item_key(&self, id: &str) -> Option<ItemKey> {
        item_key_of(self.host(), id)
    }

    fn draw_item(&self, draw: &mut dyn DrawContext, id: &str, at: [f32; 2]) {
        if let Some(key) = self.item_key(id) {
            let _ = draw.item(key, at, 16.0);
        }
    }


    fn container_open(&self) -> bool {
        match open_menu(self.host()) {
            Some(menu) => !is_player_inventory_menu(self.host(), menu),
            None => false,
        }
    }

}

/// Read-only wrapper, for hooks that only lend `&dyn ClientHost`.
pub struct View<'a> {
    host: &'a dyn ClientHost,
}

impl<'a> View<'a> {
    pub fn new(host: &'a dyn ClientHost) -> Self {
        Self { host }
    }
}

impl HasHost for View<'_> {
    fn host(&self) -> &dyn ClientHost {
        self.host
    }
}

/// Mutable wrapper used by the session tick and commands.
pub struct Game<'a> {
    host: &'a mut dyn ClientHost,
    using_ticks: Option<(i32, i32)>,
}

impl<'a> Game<'a> {
    pub fn new(host: &'a mut dyn ClientHost) -> Self {
        Self {
            host,
            using_ticks: None,
        }
    }


    pub fn native_mod_screen_open(&mut self, mod_id: &str) -> bool {
        self.host.mod_screen_open(mod_id)
    }

    pub fn native_mod_screen_close(&mut self, mod_id: &str) {
        self.host.mod_screen_close(mod_id)
    }

    pub fn native_chat(&mut self, text: &str) {
        let _ = self.host.chat_line(mod_api::Text::literal(text.to_string()));
    }

    pub fn native_client_action_bar(&mut self, text: mod_api::Text, _overlay: bool) {
        // Action-bar writes are a server-host method in this API pin; on the
        // client the same line goes through chat (and the HUD overlay still
        // shows Hookline's own flash / overlay).
        let _ = self.host.chat_line(text);
    }

    pub fn set_look(&mut self, yaw: f32, pitch: f32) {
        let _ = self.host.set_local_rotation(yaw, pitch);
    }

    pub fn add_yaw(&mut self, delta: f32) {
        let yaw = self.yaw() + delta;
        let pitch = self.pitch();
        self.set_look(yaw, pitch);
    }

    pub fn select(&mut self, slot: usize) {
        if slot < 9 {
            let _ = self.host.select_hotbar_slot(slot as u8);
        }
    }

    pub fn set_right(&mut self, down: bool) {
        let _ = self.host.set_mapping_down("key.use", down);
    }

    pub fn click_use(&mut self) -> bool {
        self.host.click_mapping("key.use").is_ok()
    }

    pub fn click_jump(&mut self) -> bool {
        self.host.click_mapping("key.jump").is_ok()
    }


    pub fn leave_server(&mut self) {
        let _ = self.host.disconnect(Some(mod_api::Text::literal(
            "Hookline: health dropped below three hearts.",
        )));
    }

    pub fn close_container(&mut self) {
        if let Some(menu) = open_menu(self.host) {
            if !is_player_inventory_menu(self.host, menu) {
                let _ = menu.close(self.host);
                return;
            }
        }
        let _ = self.host.close_screen();
    }

    pub fn click_jump_if_grounded(&mut self) {
        if self.on_ground() {
            let _ = self.click_jump();
        }
    }

    pub fn use_fishing_rod(&mut self) -> bool {
        self.click_use()
    }

    pub fn use_hand(&mut self, _hand: i32) -> bool {
        self.click_use()
    }

    pub fn start_using_item(&mut self, _hand: i32) {
        let _ = self.host.set_mapping_down("key.use", true);
        if self.using_ticks.is_none() {
            self.using_ticks = Some((40, 0));
        }
    }

    pub fn drop_held(&mut self, whole_stack: bool) {
        if let Some(menu) = open_menu(self.host) {
            let slot = inv_to_menu(self.host, menu, self.selected());
            let _ = menu.click(
                self.host,
                &[MenuClick::Throw {
                    slot,
                    whole_stack,
                }],
            );
        }
    }

    pub fn throw_inv_slot(&mut self, inv: usize, whole_stack: bool) -> bool {
        let Some(menu) = open_menu(self.host) else {
            return false;
        };
        let slot = inv_to_menu(self.host, menu, inv);
        menu.click(
            self.host,
            &[MenuClick::Throw {
                slot,
                whole_stack,
            }],
        )
    }

    pub fn inventory_quick_move(&mut self, inv: usize) -> bool {
        let Some(menu) = open_menu(self.host) else {
            return false;
        };
        let slot = inv_to_menu(self.host, menu, inv);
        menu.click(self.host, &[MenuClick::QuickMove { slot }])
    }

    pub fn click_crosshair_use(&mut self) -> bool {
        self.click_use()
    }
}

impl HasHost for Game<'_> {
    fn host(&self) -> &dyn ClientHost {
        self.host
    }

    fn using(&self) -> Option<(i32, i32)> {
        self.using_ticks
    }
}

/// One inventory stack in Hookline's old `item` / `count` / damage shape.
#[derive(Clone, Debug)]
pub struct Stack {
    pub item: String,
    pub count: u32,
    pub damage: u32,
    charms: Vec<(String, i32)>,
}

impl Stack {
    pub fn crafting_enchantments(&self) -> &[(String, i32)] {
        &self.charms
    }
}

#[derive(Clone, Debug)]
pub struct HealthFood {
    pub food: i32,
}

#[derive(Clone, Debug)]
pub struct Health {
    inner: f32,
    pub food: HealthFood,
}

impl Health {
    pub fn health(&self) -> f32 {
        self.inner
    }
}

/// The local player as the session sees it. Some fields are kept for the overlay and future
/// habits even when no habit reads them yet.
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct PlayerView {
    pub health: Health,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub jumping: bool,
    max_health: f32,
}


#[derive(Clone, Copy, Debug)]
pub struct Hook {
    pub position: Pos,
}

#[derive(Clone, Copy, Debug)]
pub struct Pos {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl From<[f64; 3]> for Pos {
    fn from(p: [f64; 3]) -> Self {
        Self {
            x: p[0],
            y: p[1],
            z: p[2],
        }
    }
}

fn item_key_of(host: &dyn ClientHost, id: &str) -> Option<ItemKey> {
    let ident = Identifier::parse(id).ok()?;
    host.item_key(&ident)
}

fn local_living(host: &dyn ClientHost) -> Option<LivingRef> {
    let player = host.local_player()?;
    host.living(player.entity())
}

fn open_menu(host: &dyn ClientHost) -> Option<MenuRef> {
    let player = host.local_player()?;
    player.container_menu(host)
}

fn is_player_inventory_menu(host: &dyn ClientHost, menu: MenuRef) -> bool {
    match menu.kind(host) {
        Some(None) | None => true,
        Some(Some(_)) => false,
    }
}

fn inv_to_menu(host: &dyn ClientHost, menu: MenuRef, inv: usize) -> usize {
    let total = menu.slot_count(host).unwrap_or(36);
    let base = total.saturating_sub(36);
    if inv < 9 {
        base + 27 + inv
    } else {
        base + (inv - 9)
    }
}

fn to_stack(host: &dyn ClientHost, stack: &ItemStack) -> Stack {
    let item = stack.item().id(host);
    let count = stack.count();
    let damage = stack.damage();
    let charms = stack_enchantments(stack);
    Stack {
        item,
        count,
        damage,
        charms,
    }
}

fn inventory_of(host: &dyn ClientHost) -> Vec<Option<Stack>> {
    let mut slots: Vec<Option<Stack>> = vec![None; 36];
    if let Some(menu) = open_menu(host) {
        if let Some(total) = menu.slot_count(host) {
            if total >= 36 {
                let base = total - 36;
                for i in 0..27 {
                    if let Some(stack) = menu.slot(host, base + i).and_then(|s| s) {
                        slots[9 + i] = Some(to_stack(host, &stack));
                    }
                }
                for i in 0..9 {
                    if let Some(stack) = menu.slot(host, base + 27 + i).and_then(|s| s) {
                        slots[i] = Some(to_stack(host, &stack));
                    }
                }
                return slots;
            }
        }
    }
    if let Some(player) = host.local_player() {
        if let Some(stacks) = player.inventory(host) {
            let sel = player.selected_slot(host).unwrap_or(0).min(8);
            for (i, stack) in stacks.into_iter().take(36).enumerate() {
                if i < slots.len() {
                    slots[i] = Some(to_stack(host, &stack));
                }
            }
            if slots[sel].is_none() {
                if let Some(living) = local_living(host) {
                    if let Some(stack) = living.hand_item(host, Hand::Main) {
                        slots[sel] = Some(to_stack(host, &stack));
                    }
                }
            }
        }
    }
    slots
}

fn player_of(host: &dyn ClientHost) -> Option<PlayerView> {
    let player: PlayerRef = host.local_player()?;
    let entity = player.entity();
    let living = host.living(entity)?;
    let pos = entity
        .position(host)
        .or_else(|| host.camera().map(|c| c.position))?;
    let food = player.food(host).map(|f| f.level).unwrap_or(20);
    let health = living.health(host).unwrap_or(20.0);
    let max_health = living.max_health(host).unwrap_or(20.0);
    let yaw = host.camera().map(|c| c.yaw).unwrap_or(0.0);
    let pitch = host.camera().map(|c| c.pitch).unwrap_or(0.0);
    Some(PlayerView {
        health: Health {
            inner: health,
            food: HealthFood { food },
        },
        x: pos[0],
        y: pos[1],
        z: pos[2],
        yaw,
        pitch,
        jumping: false,
        max_health,
    })
}

fn hook_of(host: &dyn ClientHost) -> Option<Hook> {
    let player = host.local_player()?;
    let pos = player.entity().position(host)?;
    let level = player.level(host)?;
    let reach = 64.0;
    let min = [pos[0] - reach, pos[1] - reach, pos[2] - reach];
    let max = [pos[0] + reach, pos[1] + reach, pos[2] + reach];
    let ents = level.entities_in(host, min, max).unwrap_or_default();
    // fishing_hook() is a server-host method in this API pin and refuses on a
    // client. Identify bobbers by entity type instead and take the nearest.
    let mut nearest: Option<(f64, [f64; 3])> = None;
    for entity in ents {
        let Some(kind) = entity.kind(host) else { continue };
        if kind.id(host) != "minecraft:fishing_bobber" {
            continue;
        }
        let Some(p) = entity.position(host) else { continue };
        let dx = p[0] - pos[0];
        let dy = p[1] - pos[1];
        let dz = p[2] - pos[2];
        let d2 = dx * dx + dy * dy + dz * dz;
        if nearest.is_none_or(|(best, _)| d2 < best) {
            nearest = Some((d2, p));
        }
    }
    nearest.map(|(_, p)| Hook { position: p.into() })
}

fn stack_enchantments(stack: &ItemStack) -> Vec<(String, i32)> {
    let Some(components) = stack.components() else {
        return Vec::new();
    };
    let nbt = components.as_nbt();
    for key in ["minecraft:enchantments", "minecraft:stored_enchantments"] {
        if let Some(tag) = nbt.get(&JavaString::from(key)) {
            let found = charms_from_tag(tag);
            if !found.is_empty() {
                return found;
            }
        }
    }
    Vec::new()
}

fn charms_from_tag(tag: &Tag) -> Vec<(String, i32)> {
    let levels = match tag {
        Tag::Compound(c) => c
            .get(&JavaString::from("levels"))
            .unwrap_or(tag),
        other => other,
    };
    match levels {
        Tag::Compound(map) => map
            .iter()
            .filter_map(|(key, value)| Some((key.to_string_lossy(), tag_int(value)?)))
            .collect(),
        Tag::List(list) => list
            .iter()
            .filter_map(|t| {
                let id = match t {
                    Tag::Compound(c) => c
                        .get(&JavaString::from("id"))
                        .and_then(tag_string)?,
                    _ => return None,
                };
                let lvl = match t {
                    Tag::Compound(c) => c
                        .get(&JavaString::from("lvl"))
                        .or_else(|| c.get(&JavaString::from("level")))
                        .and_then(tag_int)?,
                    _ => return None,
                };
                Some((id, lvl))
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn tag_int(tag: &Tag) -> Option<i32> {
    match tag {
        Tag::Byte(v) => Some(*v as i32),
        Tag::Short(v) => Some(*v as i32),
        Tag::Int(v) => Some(*v),
        Tag::Long(v) => Some(*v as i32),
        _ => None,
    }
}

fn tag_string(tag: &Tag) -> Option<String> {
    match tag {
        Tag::String(s) => Some(s.to_string_lossy()),
        _ => None,
    }
}

#[allow(dead_code)]
fn _equip_offhand() -> EquipmentSlot {
    EquipmentSlot::OffHand
}
