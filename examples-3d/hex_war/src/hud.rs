use crate::ecs::{Faction, GameWorld, HudSnapshot, faction_color, faction_name};
use crate::turn_phase::TurnPhase;
use nightshade::ecs::ui::state::UiStateTrait;
use nightshade::prelude::*;

pub struct HudUi {
    pub screen: Entity,
    pub turn_text: Entity,
    pub faction_text: Entity,
    pub actions_text: Entity,
    pub instructions_text: Entity,
    pub speed_text: Entity,
}

pub fn build_hud_ui(world: &mut World) -> HudUi {
    let placeholder = Entity {
        id: 0,
        generation: 0,
    };

    let label_font = 14.0;
    let heading_font = 16.0;
    let white = Vec4::new(1.0, 1.0, 1.0, 1.0);
    let dim = Vec4::new(0.7, 0.7, 0.7, 1.0);
    let speed_color = Vec4::new(0.9, 0.9, 0.5, 1.0);
    let panel_bg = Vec4::new(0.0, 0.0, 0.0, 0.5);

    let mut tree = UiTreeBuilder::new(world);

    let mut turn_text = placeholder;
    let mut faction_text = placeholder;
    let mut actions_text = placeholder;
    let mut instructions_text = placeholder;
    let mut speed_text = placeholder;

    let screen = tree
        .add_node()
        .boundary(Rl(Vec2::new(0.0, 0.0)), Rl(Vec2::new(100.0, 100.0)))
        .with_visible(false)
        .without_pointer_events()
        .with_children(|tree| {
            tree.add_node()
                .window(
                    Ab(Vec2::new(10.0, 10.0)),
                    Ab(Vec2::new(280.0, 130.0)),
                    Anchor::TopLeft,
                )
                .with_rect(6.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
                .with_color::<UiBase>(panel_bg)
                .flow(FlowDirection::Vertical, 8.0, 4.0)
                .without_pointer_events()
                .with_children(|tree| {
                    turn_text = tree
                        .add_node()
                        .flow_child(
                            Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, heading_font * 1.4)),
                        )
                        .with_text("Turn 1", heading_font)
                        .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                        .with_color::<UiBase>(white)
                        .without_pointer_events()
                        .done();

                    faction_text = tree
                        .add_node()
                        .flow_child(
                            Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, heading_font * 1.4)),
                        )
                        .with_text("Redosia", heading_font)
                        .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                        .with_color::<UiBase>(white)
                        .without_pointer_events()
                        .done();

                    actions_text = tree
                        .add_node()
                        .flow_child(
                            Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, label_font * 1.4)),
                        )
                        .with_text("Actions: 5", label_font)
                        .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                        .with_color::<UiBase>(Vec4::new(0.9, 0.9, 0.7, 1.0))
                        .without_pointer_events()
                        .done();

                    speed_text = tree
                        .add_node()
                        .flow_child(
                            Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, label_font * 1.4)),
                        )
                        .with_text("Speed: 1x", label_font)
                        .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                        .with_color::<UiBase>(speed_color)
                        .without_pointer_events()
                        .done();
                })
                .done();

            instructions_text = tree
                .add_node()
                .window(
                    Rl(Vec2::new(50.0, 98.0)),
                    Ab(Vec2::new(500.0, 24.0)),
                    Anchor::BottomCenter,
                )
                .with_text(
                    "[SPACE] End Turn  [S] Speech  [P] Pause  [+/-] Speed",
                    label_font,
                )
                .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                .with_color::<UiBase>(dim)
                .without_pointer_events()
                .done();
        })
        .done();

    tree.finish();

    HudUi {
        screen,
        turn_text,
        faction_text,
        actions_text,
        instructions_text,
        speed_text,
    }
}

pub fn update_hud(
    hud: &HudUi,
    game_world: &mut GameWorld,
    world: &mut World,
    player_faction: Faction,
) {
    let current = HudSnapshot {
        turn: game_world.resources.turn_number,
        faction: game_world.resources.current_faction,
        actions: game_world.resources.actions_remaining,
        speed_bits: game_world.resources.game_speed.to_bits(),
        is_player_turn: game_world.resources.current_faction == player_faction,
        turn_phase: game_world.resources.turn_phase,
    };

    if current == game_world.resources.previous_hud {
        return;
    }

    game_world.resources.previous_hud = current;

    let turn = game_world.resources.turn_number;
    let faction = game_world.resources.current_faction;
    let actions = game_world.resources.actions_remaining;
    let is_player_turn = faction == player_faction;

    world.ui_set_text(hud.turn_text, &format!("Turn {}", turn));

    let name = faction_name(faction);
    let fc = faction_color(faction);
    world.ui_set_text(hud.faction_text, name);
    if let Some(color) = world.ui.get_ui_node_color_mut(hud.faction_text) {
        color.colors[UiBase::INDEX] = Some(Vec4::new(fc[0], fc[1], fc[2], 1.0));
    }

    let phase_label = match game_world.resources.turn_phase {
        TurnPhase::Reinforcement => "Reinforcement",
        TurnPhase::Action => "Action",
        TurnPhase::End => "End",
    };
    world.ui_set_text(
        hud.actions_text,
        &format!("Actions: {} ({})", actions, phase_label),
    );

    let instructions = if is_player_turn {
        "[SPACE] End Turn  [S] Speech  [P] Pause  [+/-] Speed"
    } else {
        "[P] Pause  [+/-] Speed"
    };
    world.ui_set_text(hud.instructions_text, instructions);

    let speed = game_world.resources.game_speed;
    let speed_str = if speed >= 1.0 {
        format!("Speed: {}x", speed as i32)
    } else {
        format!("Speed: {:.2}x", speed)
    };
    world.ui_set_text(hud.speed_text, &speed_str);
}
