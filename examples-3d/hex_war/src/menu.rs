use crate::ecs::{Difficulty, Faction};
use nightshade::ecs::ui::state::UiStateTrait;
use nightshade::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum MenuState {
    #[default]
    MainMenu,
    MapSetup,
    Playing,
    Paused,
    GameOver,
    Replay,
}

pub struct MenuUi {
    pub main_menu_screen: Entity,
    pub new_game_button: Entity,
    pub quit_button: Entity,

    pub map_setup_screen: Entity,
    pub easy_button: Entity,
    pub normal_button: Entity,
    pub hard_button: Entity,
    pub difficulty_label: Entity,
    pub new_map_button: Entity,
    pub start_button: Entity,
    pub setup_back_button: Entity,

    pub pause_screen: Entity,
    pub resume_button: Entity,
    pub pause_main_menu_button: Entity,

    pub game_over_screen: Entity,
    pub game_over_title: Entity,
    pub game_over_subtitle: Entity,
    pub game_over_new_game_button: Entity,
    pub game_over_main_menu_button: Entity,
    pub game_over_replay_button: Entity,

    pub replay_screen: Entity,
    pub replay_back_button: Entity,
}

pub fn build_menu_ui(world: &mut World) -> MenuUi {
    let placeholder = Entity {
        id: 0,
        generation: 0,
    };

    let title_font = 36.0;
    let heading_font = 24.0;
    let label_font = 14.0;

    let gold = Vec4::new(1.0, 0.8, 0.2, 1.0);
    let white = Vec4::new(1.0, 1.0, 1.0, 1.0);
    let dim = Vec4::new(0.5, 0.5, 0.55, 1.0);
    let panel_bg = Vec4::new(0.08, 0.08, 0.12, 0.9);
    let panel_border = Vec4::new(0.25, 0.22, 0.15, 0.5);
    let screen_bg = Vec4::new(0.0, 0.0, 0.0, 0.6);

    let mut tree = UiTreeBuilder::new(world);

    let mut new_game_button = placeholder;
    let mut quit_button = placeholder;

    let main_menu_screen = tree
        .add_node()
        .boundary(Rl(Vec2::new(0.0, 0.0)), Rl(Vec2::new(100.0, 100.0)))
        .with_rect(0.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
        .with_color::<UiBase>(screen_bg)
        .with_layer(UiLayer::FloatingPanels)
        .without_pointer_events()
        .with_children(|tree| {
            tree.add_node()
                .window(
                    Rl(Vec2::new(50.0, 45.0)),
                    Ab(Vec2::new(340.0, 300.0)),
                    Anchor::Center,
                )
                .with_rect(10.0, 1.0, panel_border)
                .with_color::<UiBase>(panel_bg)
                .flow(FlowDirection::Vertical, 24.0, 6.0)
                .without_pointer_events()
                .with_children(|tree| {
                    tree.add_spacing(8.0);

                    tree.add_node()
                        .flow_child(
                            Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, title_font * 1.6)),
                        )
                        .with_text("HEX WAR", title_font)
                        .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                        .with_color::<UiBase>(gold)
                        .without_pointer_events()
                        .done();

                    tree.add_spacing(16.0);

                    new_game_button =
                        tree.add_button_colored("NEW GAME", Vec4::new(0.3, 0.5, 0.3, 1.0));
                    quit_button = tree.add_button("QUIT");
                })
                .done();
        })
        .done();

    let mut easy_button = placeholder;
    let mut normal_button = placeholder;
    let mut hard_button = placeholder;
    let mut difficulty_label = placeholder;
    let mut new_map_button = placeholder;
    let mut start_button = placeholder;
    let mut setup_back_button = placeholder;

    let map_setup_screen = tree
        .add_node()
        .boundary(Rl(Vec2::new(0.0, 0.0)), Rl(Vec2::new(100.0, 100.0)))
        .with_layer(UiLayer::FloatingPanels)
        .with_visible(false)
        .without_pointer_events()
        .with_children(|tree| {
            tree.add_node()
                .window(
                    Rl(Vec2::new(50.0, 12.0)),
                    Ab(Vec2::new(440.0, 320.0)),
                    Anchor::TopCenter,
                )
                .with_rect(10.0, 1.0, panel_border)
                .with_color::<UiBase>(panel_bg)
                .flow(FlowDirection::Vertical, 16.0, 6.0)
                .without_pointer_events()
                .with_children(|tree| {
                    tree.add_spacing(4.0);

                    tree.add_node()
                        .flow_child(
                            Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, heading_font * 1.6)),
                        )
                        .with_text("MAP SETUP", heading_font)
                        .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                        .with_color::<UiBase>(Vec4::new(0.8, 1.0, 0.8, 1.0))
                        .without_pointer_events()
                        .done();

                    difficulty_label = tree
                        .add_node()
                        .flow_child(
                            Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, label_font * 1.5)),
                        )
                        .with_text("Difficulty: Normal", label_font)
                        .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                        .with_color::<UiBase>(dim)
                        .without_pointer_events()
                        .done();

                    let diff_row = tree
                        .add_node()
                        .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, 36.0)))
                        .flow(FlowDirection::Horizontal, 0.0, 6.0)
                        .without_pointer_events()
                        .entity();
                    tree.push_parent(diff_row);
                    tree.add_spring();

                    let easy_wrap = tree
                        .add_node()
                        .flow_child(Ab(Vec2::new(110.0, 36.0)))
                        .without_pointer_events()
                        .entity();
                    tree.push_parent(easy_wrap);
                    easy_button = tree.add_button("EASY");
                    tree.pop_parent();

                    let normal_wrap = tree
                        .add_node()
                        .flow_child(Ab(Vec2::new(110.0, 36.0)))
                        .without_pointer_events()
                        .entity();
                    tree.push_parent(normal_wrap);
                    normal_button =
                        tree.add_button_colored("NORMAL", Vec4::new(0.35, 0.35, 0.2, 1.0));
                    tree.pop_parent();

                    let hard_wrap = tree
                        .add_node()
                        .flow_child(Ab(Vec2::new(110.0, 36.0)))
                        .without_pointer_events()
                        .entity();
                    tree.push_parent(hard_wrap);
                    hard_button = tree.add_button("HARD");
                    tree.pop_parent();

                    tree.add_spring();
                    tree.pop_parent();

                    tree.add_spacing(4.0);

                    new_map_button = tree.add_button("NEW MAP");
                    start_button =
                        tree.add_button_colored("START GAME", Vec4::new(0.3, 0.5, 0.3, 1.0));
                    setup_back_button = tree.add_button("BACK");
                })
                .done();
        })
        .done();

    let mut resume_button = placeholder;
    let mut pause_main_menu_button = placeholder;

    let pause_screen = tree
        .add_node()
        .boundary(Rl(Vec2::new(0.0, 0.0)), Rl(Vec2::new(100.0, 100.0)))
        .with_rect(0.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
        .with_color::<UiBase>(Vec4::new(0.0, 0.0, 0.0, 0.6))
        .with_layer(UiLayer::FloatingPanels)
        .with_visible(false)
        .without_pointer_events()
        .with_children(|tree| {
            tree.add_node()
                .window(
                    Rl(Vec2::new(50.0, 45.0)),
                    Ab(Vec2::new(300.0, 250.0)),
                    Anchor::Center,
                )
                .with_rect(10.0, 1.0, panel_border)
                .with_color::<UiBase>(panel_bg)
                .flow(FlowDirection::Vertical, 24.0, 6.0)
                .without_pointer_events()
                .with_children(|tree| {
                    tree.add_spacing(4.0);

                    tree.add_node()
                        .flow_child(
                            Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, heading_font * 1.6)),
                        )
                        .with_text("PAUSED", heading_font)
                        .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                        .with_color::<UiBase>(Vec4::new(1.0, 0.5, 0.2, 1.0))
                        .without_pointer_events()
                        .done();

                    tree.add_spacing(8.0);

                    resume_button = tree.add_button("RESUME");
                    pause_main_menu_button = tree.add_button("MAIN MENU");
                })
                .done();
        })
        .done();

    let mut game_over_title = placeholder;
    let mut game_over_subtitle = placeholder;
    let mut game_over_new_game_button = placeholder;
    let mut game_over_main_menu_button = placeholder;
    let mut game_over_replay_button = placeholder;
    let mut replay_back_button = placeholder;

    let game_over_screen = tree
        .add_node()
        .boundary(Rl(Vec2::new(0.0, 0.0)), Rl(Vec2::new(100.0, 100.0)))
        .with_layer(UiLayer::FloatingPanels)
        .with_visible(false)
        .without_pointer_events()
        .with_children(|tree| {
            tree.add_node()
                .window(
                    Rl(Vec2::new(50.0, 15.0)),
                    Ab(Vec2::new(380.0, 300.0)),
                    Anchor::TopCenter,
                )
                .with_rect(10.0, 1.0, panel_border)
                .with_color::<UiBase>(panel_bg)
                .flow(FlowDirection::Vertical, 24.0, 6.0)
                .without_pointer_events()
                .with_children(|tree| {
                    tree.add_spacing(8.0);

                    game_over_title = tree
                        .add_node()
                        .flow_child(
                            Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, title_font * 1.6)),
                        )
                        .with_text("VICTORY!", title_font)
                        .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                        .with_color::<UiBase>(gold)
                        .without_pointer_events()
                        .done();

                    game_over_subtitle = tree
                        .add_node()
                        .flow_child(
                            Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, label_font * 1.5)),
                        )
                        .with_text("", label_font)
                        .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                        .with_color::<UiBase>(white)
                        .without_pointer_events()
                        .done();

                    tree.add_spacing(8.0);

                    game_over_new_game_button =
                        tree.add_button_colored("NEW GAME", Vec4::new(0.3, 0.5, 0.3, 1.0));
                    game_over_replay_button =
                        tree.add_button_colored("WATCH REPLAY", Vec4::new(0.2, 0.3, 0.5, 1.0));
                    game_over_main_menu_button = tree.add_button("MAIN MENU");
                })
                .done();
        })
        .done();

    let replay_screen = tree
        .add_node()
        .boundary(Rl(Vec2::new(0.0, 0.0)), Rl(Vec2::new(100.0, 100.0)))
        .with_layer(UiLayer::FloatingPanels)
        .with_visible(false)
        .without_pointer_events()
        .with_children(|tree| {
            tree.add_node()
                .window(
                    Rl(Vec2::new(50.0, 4.0)),
                    Ab(Vec2::new(360.0, 100.0)),
                    Anchor::TopCenter,
                )
                .with_rect(8.0, 1.0, panel_border)
                .with_color::<UiBase>(panel_bg)
                .flow(FlowDirection::Vertical, 8.0, 6.0)
                .without_pointer_events()
                .with_children(|tree| {
                    tree.add_node()
                        .flow_child(
                            Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, heading_font * 1.4)),
                        )
                        .with_text("GAME REPLAY", heading_font)
                        .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                        .with_color::<UiBase>(gold)
                        .without_pointer_events()
                        .done();

                    tree.add_node()
                        .flow_child(
                            Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, label_font * 1.3)),
                        )
                        .with_text("Scroll the event log to review the game", label_font)
                        .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                        .with_color::<UiBase>(dim)
                        .without_pointer_events()
                        .done();

                    replay_back_button = tree.add_button("BACK TO RESULTS");
                })
                .done();
        })
        .done();

    tree.finish();

    MenuUi {
        main_menu_screen,
        new_game_button,
        quit_button,

        map_setup_screen,
        easy_button,
        normal_button,
        hard_button,
        difficulty_label,
        new_map_button,
        start_button,
        setup_back_button,

        pause_screen,
        resume_button,
        pause_main_menu_button,

        game_over_screen,
        game_over_title,
        game_over_subtitle,
        game_over_new_game_button,
        game_over_main_menu_button,
        game_over_replay_button,

        replay_screen,
        replay_back_button,
    }
}

pub fn show_menu_screen(world: &mut World, ui: &MenuUi, state: MenuState) {
    world.ui_set_visible(ui.main_menu_screen, state == MenuState::MainMenu);
    world.ui_set_visible(ui.map_setup_screen, state == MenuState::MapSetup);
    world.ui_set_visible(ui.pause_screen, state == MenuState::Paused);
    world.ui_set_visible(ui.game_over_screen, state == MenuState::GameOver);
    world.ui_set_visible(ui.replay_screen, state == MenuState::Replay);
}

pub fn update_difficulty_display(world: &mut World, ui: &MenuUi, difficulty: Difficulty) {
    let label = match difficulty {
        Difficulty::Easy => "Difficulty: Easy",
        Difficulty::Normal => "Difficulty: Normal",
        Difficulty::Hard => "Difficulty: Hard",
    };
    world.ui_set_text(ui.difficulty_label, label);

    let easy_color = if difficulty == Difficulty::Easy {
        Vec4::new(0.35, 0.35, 0.2, 1.0)
    } else {
        Vec4::new(0.15, 0.15, 0.15, 1.0)
    };
    let normal_color = if difficulty == Difficulty::Normal {
        Vec4::new(0.35, 0.35, 0.2, 1.0)
    } else {
        Vec4::new(0.15, 0.15, 0.15, 1.0)
    };
    let hard_color = if difficulty == Difficulty::Hard {
        Vec4::new(0.35, 0.35, 0.2, 1.0)
    } else {
        Vec4::new(0.15, 0.15, 0.15, 1.0)
    };

    if let Some(color) = world.ui.get_ui_node_color_mut(ui.easy_button) {
        color.colors[UiBase::INDEX] = Some(easy_color);
    }
    if let Some(color) = world.ui.get_ui_node_color_mut(ui.normal_button) {
        color.colors[UiBase::INDEX] = Some(normal_color);
    }
    if let Some(color) = world.ui.get_ui_node_color_mut(ui.hard_button) {
        color.colors[UiBase::INDEX] = Some(hard_color);
    }
}

pub fn setup_game_over_display(
    world: &mut World,
    ui: &MenuUi,
    winner: Faction,
    is_player_winner: bool,
) {
    let (title_text, title_color) = if is_player_winner {
        ("VICTORY!", Vec4::new(1.0, 0.85, 0.2, 1.0))
    } else {
        ("DEFEAT", Vec4::new(0.8, 0.2, 0.2, 1.0))
    };

    world.ui_set_text(ui.game_over_title, title_text);
    if let Some(color) = world.ui.get_ui_node_color_mut(ui.game_over_title) {
        color.colors[UiBase::INDEX] = Some(title_color);
    }

    let name = winner.name();
    let subtitle = if is_player_winner {
        format!("{} conquers all!", name)
    } else {
        format!("{} has conquered the world!", name)
    };

    world.ui_set_text(ui.game_over_subtitle, &subtitle);
    let fc = winner.color();
    if let Some(color) = world.ui.get_ui_node_color_mut(ui.game_over_subtitle) {
        color.colors[UiBase::INDEX] = Some(Vec4::new(fc[0], fc[1], fc[2], 1.0));
    }
}
