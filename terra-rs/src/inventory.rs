use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

use crate::consts::*;
use crate::world::tile_color;

#[derive(Clone, Copy, Default)]
pub struct ItemSlot {
    pub id: u8,
    pub count: u16,
}

#[derive(Resource)]
pub struct Inventory {
    pub slots: [ItemSlot; INV_SLOTS],
    pub selected: usize,
}

impl Default for Inventory {
    fn default() -> Self {
        let mut slots = [ItemSlot::default(); INV_SLOTS];
        slots[0] = ItemSlot { id: ITEM_SWORD, count: 1 };
        slots[1] = ItemSlot { id: ITEM_BOW, count: 1 };
        Self { slots, selected: 0 }
    }
}

impl Inventory {
    /// false if the inventory is full
    pub fn add(&mut self, item: u8, count: u16) -> bool {
        let mut count = count;
        for s in &mut self.slots {
            if s.id == item && s.count < STACK_MAX {
                let add = count.min(STACK_MAX - s.count);
                s.count += add;
                count -= add;
                if count == 0 {
                    return true;
                }
            }
        }
        for s in &mut self.slots {
            if s.id == ITEM_NONE {
                s.id = item;
                s.count = count.min(STACK_MAX);
                return true;
            }
        }
        false
    }
}

pub fn item_color(item: u8) -> Color {
    match item {
        ITEM_SWORD => Color::srgb_u8(200, 200, 215),
        ITEM_BOW => Color::srgb_u8(190, 140, 80),
        1..=6 => tile_color(item),
        _ => Color::srgb(1.0, 0.0, 1.0),
    }
}

// ---- Hotbar UI ---------------------------------------------------------
// TODO(parity-notes): plain with_children spawn; revisit with 0.19 bsn! once
// its syntax is battle-tested.

#[derive(Component)]
struct HotbarSlot(usize);

#[derive(Component)]
struct HotbarCount;

#[derive(Component)]
struct HotbarIcon;

const SLOT_BG: Color = Color::srgba(0.0, 0.0, 0.0, 0.47);
const SLOT_BORDER: Color = Color::srgba(0.86, 0.86, 0.86, 0.63);
const SLOT_BORDER_SELECTED: Color = Color::srgb(0.99, 0.98, 0.0);

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Inventory>()
            .add_systems(Startup, spawn_hotbar)
            .add_systems(
                Update,
                (
                    select_slot,
                    refresh_hotbar.run_if(resource_changed::<Inventory>),
                )
                    .chain(),
            );
    }
}

fn select_slot(
    keys: Res<ButtonInput<KeyCode>>,
    mut wheel: MessageReader<MouseWheel>,
    mut inv: ResMut<Inventory>,
) {
    const DIGITS: [KeyCode; INV_SLOTS] = [
        KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4,
        KeyCode::Digit5, KeyCode::Digit6, KeyCode::Digit7, KeyCode::Digit8,
    ];
    for (i, k) in DIGITS.iter().enumerate() {
        if keys.just_pressed(*k) {
            inv.selected = i;
        }
    }
    let scroll: f32 = wheel.read().map(|w| w.y).sum();
    if scroll > 0.0 {
        inv.selected = (inv.selected + INV_SLOTS - 1) % INV_SLOTS;
    } else if scroll < 0.0 {
        inv.selected = (inv.selected + 1) % INV_SLOTS;
    }
}

fn spawn_hotbar(mut commands: Commands) {
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(12.0),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            justify_content: JustifyContent::Center,
            column_gap: Val::Px(4.0),
            ..default()
        })
        .with_children(|row| {
            for i in 0..INV_SLOTS {
                row.spawn((
                    HotbarSlot(i),
                    Node {
                        width: Val::Px(40.0),
                        height: Val::Px(40.0),
                        border: UiRect::all(Val::Px(2.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(SLOT_BG),
                    BorderColor::all(SLOT_BORDER),
                ))
                .with_children(|slot| {
                    // Default ImageNode = 1x1 white pixel; tinting it gives the
                    // colored-square fallback, setting image+rect gives a sprite
                    slot.spawn((
                        HotbarIcon,
                        ImageNode::default(),
                        Node {
                            position_type: PositionType::Absolute,
                            width: Val::Px(24.0),
                            height: Val::Px(24.0),
                            ..default()
                        },
                    ));
                    slot.spawn((
                        HotbarCount,
                        Text::new(""),
                        // font_size is the FontSize enum since 0.19
                        TextFont { font_size: FontSize::Px(11.0), ..default() },
                    ));
                });
            }
        });
}

fn refresh_hotbar(
    inv: Res<Inventory>,
    assets: Res<crate::assets::GameAssets>,
    mut slots: Query<(&HotbarSlot, &mut BorderColor, &Children)>,
    mut texts: Query<&mut Text, With<HotbarCount>>,
    mut icons: Query<&mut ImageNode, With<HotbarIcon>>,
) {
    for (slot, mut border, children) in &mut slots {
        let s = inv.slots[slot.0];
        *border = BorderColor::all(if slot.0 == inv.selected {
            SLOT_BORDER_SELECTED
        } else {
            SLOT_BORDER
        });
        for child in children.iter() {
            if let Ok(mut text) = texts.get_mut(child) {
                text.0 = if s.id != ITEM_NONE && s.count > 1 {
                    s.count.to_string()
                } else {
                    String::new()
                };
            }
            if let Ok(mut icon) = icons.get_mut(child) {
                let name = match s.id {
                    ITEM_SWORD => Some("item_sword"),
                    1..=6 => Some(
                        ["", "tile_dirt", "tile_grass", "tile_stone",
                         "tile_wood", "tile_leaves", "tile_ore"][s.id as usize],
                    ),
                    _ => None,
                };
                let sprite = name.and_then(|n| assets.sprite(n, Vec2::splat(24.0)));
                *icon = match (s.id, sprite) {
                    (ITEM_NONE, _) => ImageNode::default().with_color(Color::NONE),
                    (_, Some(sp)) => ImageNode { image: sp.image, rect: sp.rect, ..default() },
                    (id, None) => ImageNode::default().with_color(item_color(id)),
                };
            }
        }
    }
}
