use nightshade::ecs::ui::state::UiStateTrait;
use nightshade::prelude::*;

use crate::data::{get_item_definition, get_npc_definition, get_shop_keeper_name};
use crate::ecs::{ShopMode, TreeState, World as GameWorld};
use crate::game::GamePhase;
use crate::systems::social;
use crate::types::{ToolType, format_time};

const SHOP_ITEM_COUNT: usize = 6;
const HOTBAR_SLOT_COUNT: usize = 10;

fn get_short_item_name(name: &str) -> &str {
    if name.contains("Parsnip") {
        "Pars"
    } else if name.contains("Cauliflower") {
        "Caul"
    } else if name.contains("Potato") {
        "Pota"
    } else if name.contains("Tomato") {
        "Toma"
    } else if name.contains("Corn") {
        "Corn"
    } else if name.contains("Pumpkin") {
        "Pump"
    } else if name.len() > 4 {
        &name[..4]
    } else {
        name
    }
}

pub struct FarmingUi {
    main_menu_screen: Entity,
    start_button: Entity,

    pause_screen: Entity,
    resume_button: Entity,

    hud_screen: Entity,
    day_time_text: Entity,
    weather_text: Entity,
    stamina_bar: Entity,
    stamina_bar_fill: Entity,
    gold_text: Entity,
    tool_text: Entity,

    hotbar_slots: [Entity; HOTBAR_SLOT_COUNT],
    hotbar_name_texts: [Entity; HOTBAR_SLOT_COUNT],
    hotbar_qty_texts: [Entity; HOTBAR_SLOT_COUNT],

    tree_health_container: Entity,
    tree_health_bar: Entity,
    tree_health_bar_fill: Entity,

    shop_screen: Entity,
    buy_tab: Entity,
    sell_tab: Entity,
    shop_item_rows: [Entity; SHOP_ITEM_COUNT],
    shop_item_name_texts: [Entity; SHOP_ITEM_COUNT],
    shop_item_owned_texts: [Entity; SHOP_ITEM_COUNT],
    shop_item_price_texts: [Entity; SHOP_ITEM_COUNT],
    shop_gold_text: Entity,
    shop_controls_text: Entity,

    dialogue_container: Entity,
    dialogue_header: Entity,
    dialogue_name_entity: Entity,
    dialogue_name_text: Entity,
    dialogue_text_text: Entity,

    hint_container: Entity,
    hint_text: Entity,
}

impl Default for FarmingUi {
    fn default() -> Self {
        let placeholder = Entity {
            id: 0,
            generation: 0,
        };
        Self {
            main_menu_screen: placeholder,
            start_button: placeholder,
            pause_screen: placeholder,
            resume_button: placeholder,
            hud_screen: placeholder,
            day_time_text: placeholder,
            weather_text: placeholder,
            stamina_bar: placeholder,
            stamina_bar_fill: placeholder,
            gold_text: placeholder,
            tool_text: placeholder,
            hotbar_slots: [placeholder; HOTBAR_SLOT_COUNT],
            hotbar_name_texts: [placeholder; HOTBAR_SLOT_COUNT],
            hotbar_qty_texts: [placeholder; HOTBAR_SLOT_COUNT],
            tree_health_container: placeholder,
            tree_health_bar: placeholder,
            tree_health_bar_fill: placeholder,
            shop_screen: placeholder,
            buy_tab: placeholder,
            sell_tab: placeholder,
            shop_item_rows: [placeholder; SHOP_ITEM_COUNT],
            shop_item_name_texts: [placeholder; SHOP_ITEM_COUNT],
            shop_item_owned_texts: [placeholder; SHOP_ITEM_COUNT],
            shop_item_price_texts: [placeholder; SHOP_ITEM_COUNT],
            shop_gold_text: placeholder,
            shop_controls_text: placeholder,
            dialogue_container: placeholder,
            dialogue_header: placeholder,
            dialogue_name_entity: placeholder,
            dialogue_name_text: placeholder,
            dialogue_text_text: placeholder,
            hint_container: placeholder,
            hint_text: placeholder,
        }
    }
}

impl FarmingUi {
    pub fn build(&mut self, world: &mut World) {
        let font_size = 14.0;
        let small_font = 12.0;
        let title_font = 24.0;
        let dim_text = Vec4::new(0.5, 0.5, 0.5, 1.0);
        let white = Vec4::new(1.0, 1.0, 1.0, 1.0);
        let green_title = Vec4::new(0.4, 0.8, 0.4, 1.0);
        let gold = Vec4::new(1.0, 0.84, 0.0, 1.0);

        let mut tree = UiTreeBuilder::new(world);
        let placeholder = Entity {
            id: 0,
            generation: 0,
        };

        let mut start_button = placeholder;
        self.main_menu_screen = tree
            .add_node()
            .boundary(Rl(Vec2::new(0.0, 0.0)), Rl(Vec2::new(100.0, 100.0)))
            .with_rect(0.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
            .with_color::<UiBase>(Vec4::new(0.0, 0.0, 0.0, 0.7))
            .with_layer(UiLayer::FloatingPanels)
            .without_pointer_events()
            .with_children(|tree| {
                tree.add_node()
                    .window(
                        Rl(Vec2::new(50.0, 50.0)),
                        Ab(Vec2::new(400.0, 500.0)),
                        Anchor::Center,
                    )
                    .flow(FlowDirection::Vertical, 0.0, 4.0)
                    .with_children(|tree| {
                        tree.add_node()
                            .flow_child(
                                Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, title_font * 1.5)),
                            )
                            .with_text("MEADOW FIELDS", title_font)
                            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                            .with_color::<UiBase>(green_title)
                            .without_pointer_events()
                            .done();

                        tree.add_spacing(6.0);

                        tree.add_node()
                            .flow_child(
                                Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, font_size * 1.5)),
                            )
                            .with_text("A Farming Simulation Game", font_size)
                            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                            .with_color::<UiBase>(dim_text)
                            .without_pointer_events()
                            .done();

                        tree.add_spacing(30.0);

                        start_button =
                            tree.add_button_colored("START GAME", Vec4::new(0.24, 0.47, 0.31, 1.0));

                        tree.add_spacing(20.0);

                        tree.add_label("Controls:");
                        tree.add_label_colored("WASD / Arrow Keys - Move", dim_text);
                        tree.add_label_colored("1-6 - Select Tool", dim_text);
                        tree.add_label_colored("7-0 - Select Seeds", dim_text);
                        tree.add_label_colored("Left Click - Use Tool / Plant", dim_text);
                        tree.add_label_colored("E - Interact / Talk", dim_text);
                        tree.add_label_colored("Tab - Toggle Camera", dim_text);
                        tree.add_label_colored("ESC - Pause", dim_text);

                        tree.add_spacing(10.0);

                        tree.add_node()
                            .flow_child(
                                Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, font_size * 1.5)),
                            )
                            .with_text("Press Enter or click START to begin", font_size)
                            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                            .with_color::<UiBase>(Vec4::new(0.59, 0.59, 0.59, 1.0))
                            .without_pointer_events()
                            .done();
                    })
                    .done();
            })
            .done();
        self.start_button = start_button;

        let mut resume_button = placeholder;
        self.pause_screen = tree
            .add_node()
            .boundary(Rl(Vec2::new(0.0, 0.0)), Rl(Vec2::new(100.0, 100.0)))
            .with_rect(0.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
            .with_color::<UiBase>(Vec4::new(0.0, 0.0, 0.0, 0.78))
            .with_layer(UiLayer::FloatingPanels)
            .with_visible(false)
            .without_pointer_events()
            .with_children(|tree| {
                tree.add_node()
                    .window(
                        Rl(Vec2::new(50.0, 50.0)),
                        Ab(Vec2::new(300.0, 200.0)),
                        Anchor::Center,
                    )
                    .flow(FlowDirection::Vertical, 0.0, 4.0)
                    .with_children(|tree| {
                        tree.add_node()
                            .flow_child(
                                Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, title_font * 1.5)),
                            )
                            .with_text("PAUSED", title_font)
                            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                            .with_color::<UiBase>(white)
                            .without_pointer_events()
                            .done();

                        tree.add_spacing(30.0);

                        resume_button =
                            tree.add_button_colored("Resume", Vec4::new(0.24, 0.39, 0.24, 1.0));

                        tree.add_spacing(16.0);

                        tree.add_node()
                            .flow_child(
                                Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, font_size * 1.5)),
                            )
                            .with_text("Press ESC to resume", font_size)
                            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                            .with_color::<UiBase>(dim_text)
                            .without_pointer_events()
                            .done();
                    })
                    .done();
            })
            .done();
        self.resume_button = resume_button;

        self.hud_screen = tree
            .add_node()
            .boundary(Rl(Vec2::new(0.0, 0.0)), Rl(Vec2::new(100.0, 100.0)))
            .with_visible(false)
            .without_pointer_events()
            .with_children(|tree| {
                let bar_height = 12.0;
                let bar_width = 150.0;

                let mut stamina_bar = placeholder;
                let mut day_time_text = placeholder;
                let mut weather_text = placeholder;
                let mut gold_text = placeholder;
                let mut tool_text = placeholder;
                tree.add_node()
                    .window(
                        Ab(Vec2::new(10.0, 10.0)),
                        Ab(Vec2::new(280.0, 180.0)),
                        Anchor::TopLeft,
                    )
                    .with_rect(4.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
                    .with_color::<UiBase>(Vec4::new(0.0, 0.0, 0.0, 0.6))
                    .without_pointer_events()
                    .with_children(|tree| {
                        tree.add_node()
                            .boundary(
                                Ab(Vec2::new(10.0, 10.0)),
                                Rl(Vec2::new(100.0, 100.0)) + Ab(Vec2::new(-10.0, -10.0)),
                            )
                            .flow(FlowDirection::Vertical, 0.0, 5.0)
                            .auto_size(AutoSizeMode::Height)
                            .with_children(|tree| {
                                day_time_text = tree
                                    .add_node()
                                    .flow_child(
                                        Rl(Vec2::new(100.0, 0.0))
                                            + Ab(Vec2::new(0.0, font_size * 1.5)),
                                    )
                                    .with_text("Day 1 - Spring - 6:00 AM", font_size)
                                    .with_text_alignment(
                                        TextAlignment::Left,
                                        VerticalAlignment::Middle,
                                    )
                                    .with_color::<UiBase>(white)
                                    .without_pointer_events()
                                    .done();

                                weather_text = tree
                                    .add_node()
                                    .flow_child(
                                        Rl(Vec2::new(100.0, 0.0))
                                            + Ab(Vec2::new(0.0, font_size * 1.5)),
                                    )
                                    .with_text("Weather: Sunny", font_size)
                                    .with_text_alignment(
                                        TextAlignment::Left,
                                        VerticalAlignment::Middle,
                                    )
                                    .with_color::<UiBase>(Vec4::new(0.7, 0.8, 0.9, 1.0))
                                    .without_pointer_events()
                                    .done();

                                tree.add_node()
                                    .flow_child(
                                        Rl(Vec2::new(100.0, 0.0))
                                            + Ab(Vec2::new(0.0, bar_height + 6.0)),
                                    )
                                    .flow(FlowDirection::Horizontal, 0.0, 6.0)
                                    .with_children(|tree| {
                                        tree.add_node()
                                            .flow_child(Ab(Vec2::new(60.0, bar_height + 6.0)))
                                            .with_text("Stamina:", small_font)
                                            .with_text_alignment(
                                                TextAlignment::Left,
                                                VerticalAlignment::Middle,
                                            )
                                            .with_color::<UiBase>(white)
                                            .without_pointer_events()
                                            .done();

                                        stamina_bar = tree.add_progress_bar(1.0);
                                        if let Some(node) =
                                            tree.world_mut().get_ui_layout_node_mut(stamina_bar)
                                        {
                                            node.flow_child_size =
                                                Some(Ab(Vec2::new(bar_width, bar_height)).into());
                                        }
                                    })
                                    .done();

                                gold_text = tree
                                    .add_node()
                                    .flow_child(
                                        Rl(Vec2::new(100.0, 0.0))
                                            + Ab(Vec2::new(0.0, font_size * 1.5)),
                                    )
                                    .with_text("Gold: 500 G", font_size)
                                    .with_text_alignment(
                                        TextAlignment::Left,
                                        VerticalAlignment::Middle,
                                    )
                                    .with_color::<UiBase>(gold)
                                    .without_pointer_events()
                                    .done();

                                tool_text = tree
                                    .add_node()
                                    .flow_child(
                                        Rl(Vec2::new(100.0, 0.0))
                                            + Ab(Vec2::new(0.0, font_size * 1.5)),
                                    )
                                    .with_text("Tool: Hand", font_size)
                                    .with_text_alignment(
                                        TextAlignment::Left,
                                        VerticalAlignment::Middle,
                                    )
                                    .with_color::<UiBase>(white)
                                    .without_pointer_events()
                                    .done();
                            })
                            .done();
                    })
                    .done();

                self.stamina_bar = stamina_bar;
                self.day_time_text = day_time_text;
                self.weather_text = weather_text;
                self.gold_text = gold_text;
                self.tool_text = tool_text;
                if let Some(UiWidgetState::ProgressBar(data)) =
                    tree.world_mut().get_ui_widget_state(stamina_bar)
                {
                    self.stamina_bar_fill = data.fill_entity;
                }

                let slot_size = 50.0;
                let slot_spacing = 5.0;
                let hotbar_width =
                    HOTBAR_SLOT_COUNT as f32 * (slot_size + slot_spacing) - slot_spacing;

                let mut hotbar_slots = [placeholder; HOTBAR_SLOT_COUNT];
                let mut hotbar_name_texts = [placeholder; HOTBAR_SLOT_COUNT];
                let mut hotbar_qty_texts = [placeholder; HOTBAR_SLOT_COUNT];
                tree.add_node()
                    .window(
                        Rl(Vec2::new(50.0, 100.0)) + Ab(Vec2::new(0.0, -20.0 - slot_size)),
                        Ab(Vec2::new(hotbar_width + 20.0, slot_size + 20.0)),
                        Anchor::BottomCenter,
                    )
                    .with_rect(4.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
                    .with_color::<UiBase>(Vec4::new(0.0, 0.0, 0.0, 0.5))
                    .without_pointer_events()
                    .with_children(|tree| {
                        tree.add_node()
                            .boundary(
                                Ab(Vec2::new(10.0, 10.0)),
                                Ab(Vec2::new(hotbar_width, slot_size)),
                            )
                            .flow(FlowDirection::Horizontal, 0.0, slot_spacing)
                            .with_children(|tree| {
                                for index in 0..HOTBAR_SLOT_COUNT {
                                    let key_label = if index < 9 {
                                        format!("{}", index + 1)
                                    } else {
                                        "0".to_string()
                                    };
                                    hotbar_slots[index] = tree
                                        .add_node()
                                        .flow_child(Ab(Vec2::new(slot_size, slot_size)))
                                        .with_rect(2.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
                                        .with_color::<UiBase>(Vec4::new(0.2, 0.2, 0.2, 0.8))
                                        .without_pointer_events()
                                        .with_children(|tree| {
                                            tree.add_node()
                                                .boundary(
                                                    Ab(Vec2::new(3.0, 2.0)),
                                                    Ab(Vec2::new(
                                                        slot_size - 6.0,
                                                        small_font * 1.2,
                                                    )),
                                                )
                                                .with_text(&key_label, small_font)
                                                .with_text_alignment(
                                                    TextAlignment::Left,
                                                    VerticalAlignment::Top,
                                                )
                                                .with_color::<UiBase>(Vec4::new(0.6, 0.6, 0.6, 1.0))
                                                .without_pointer_events()
                                                .done();

                                            hotbar_name_texts[index] = tree
                                                .add_node()
                                                .boundary(
                                                    Ab(Vec2::new(3.0, 16.0)),
                                                    Ab(Vec2::new(
                                                        slot_size - 6.0,
                                                        small_font * 1.2,
                                                    )),
                                                )
                                                .with_text("", small_font)
                                                .with_text_alignment(
                                                    TextAlignment::Left,
                                                    VerticalAlignment::Top,
                                                )
                                                .with_color::<UiBase>(Vec4::new(0.9, 0.9, 0.9, 1.0))
                                                .without_pointer_events()
                                                .done();

                                            hotbar_qty_texts[index] = tree
                                                .add_node()
                                                .boundary(
                                                    Ab(Vec2::new(3.0, 32.0)),
                                                    Ab(Vec2::new(
                                                        slot_size - 6.0,
                                                        small_font * 1.2,
                                                    )),
                                                )
                                                .with_text("", small_font)
                                                .with_text_alignment(
                                                    TextAlignment::Left,
                                                    VerticalAlignment::Top,
                                                )
                                                .with_color::<UiBase>(Vec4::new(0.8, 0.8, 0.8, 1.0))
                                                .without_pointer_events()
                                                .done();
                                        })
                                        .done();
                                }
                            })
                            .done();
                    })
                    .done();

                self.hotbar_slots = hotbar_slots;
                self.hotbar_name_texts = hotbar_name_texts;
                self.hotbar_qty_texts = hotbar_qty_texts;
            })
            .done();

        let mut tree_health_bar = placeholder;
        self.tree_health_container = tree
            .add_node()
            .window(
                Rl(Vec2::new(50.0, 0.0)) + Ab(Vec2::new(0.0, 60.0)),
                Ab(Vec2::new(220.0, 60.0)),
                Anchor::TopCenter,
            )
            .with_visible(false)
            .without_pointer_events()
            .flow(FlowDirection::Vertical, 0.0, 5.0)
            .with_children(|tree| {
                tree.add_node()
                    .flow_child(Ab(Vec2::new(208.0, 28.0)))
                    .with_rect(0.0, 2.0, Vec4::new(0.3, 0.2, 0.1, 1.0))
                    .with_color::<UiBase>(Vec4::new(0.0, 0.0, 0.0, 0.9))
                    .without_pointer_events()
                    .flow(FlowDirection::Vertical, 4.0, 0.0)
                    .with_children(|tree| {
                        tree_health_bar = tree.add_progress_bar(1.0);
                        if let Some(node) = tree.world_mut().get_ui_layout_node_mut(tree_health_bar)
                        {
                            node.flow_child_size = Some(Ab(Vec2::new(200.0, 20.0)).into());
                        }
                    })
                    .done();

                tree.add_node()
                    .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, font_size * 1.5)))
                    .with_text("Tree", font_size)
                    .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                    .with_color::<UiBase>(Vec4::new(0.8, 0.6, 0.4, 1.0))
                    .without_pointer_events()
                    .done();
            })
            .done();

        self.tree_health_bar = tree_health_bar;
        if let Some(UiWidgetState::ProgressBar(data)) =
            tree.world_mut().get_ui_widget_state(tree_health_bar)
        {
            self.tree_health_bar_fill = data.fill_entity;
        }

        let panel_width = 400.0;
        let panel_height = 350.0;
        let mut buy_tab = placeholder;
        let mut sell_tab = placeholder;
        let mut shop_item_rows = [placeholder; SHOP_ITEM_COUNT];
        let mut shop_item_name_texts = [placeholder; SHOP_ITEM_COUNT];
        let mut shop_item_owned_texts = [placeholder; SHOP_ITEM_COUNT];
        let mut shop_item_price_texts = [placeholder; SHOP_ITEM_COUNT];
        let mut shop_gold_text = placeholder;
        let mut shop_controls_text = placeholder;

        self.shop_screen = tree
            .add_node()
            .window(
                Rl(Vec2::new(50.0, 50.0)),
                Ab(Vec2::new(panel_width, panel_height)),
                Anchor::Center,
            )
            .with_rect(0.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
            .with_color::<UiBase>(Vec4::new(0.1, 0.1, 0.1, 0.95))
            .with_layer(UiLayer::FloatingPanels)
            .with_visible(false)
            .without_pointer_events()
            .with_children(|tree| {
                tree.add_node()
                    .boundary(Ab(Vec2::new(0.0, 0.0)), Ab(Vec2::new(panel_width, 40.0)))
                    .with_rect(0.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
                    .with_color::<UiBase>(Vec4::new(0.2, 0.4, 0.3, 1.0))
                    .with_children(|tree| {
                        tree.add_node()
                            .boundary(
                                Ab(Vec2::new(15.0, 10.0)),
                                Ab(Vec2::new(panel_width - 30.0, 20.0)),
                            )
                            .with_text(&format!("{}'s Shop", get_shop_keeper_name()), font_size)
                            .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                            .with_color::<UiBase>(white)
                            .without_pointer_events()
                            .done();
                    })
                    .done();

                let tab_width = (panel_width - 20.0) / 2.0;
                buy_tab = tree
                    .add_node()
                    .boundary(Ab(Vec2::new(10.0, 50.0)), Ab(Vec2::new(tab_width, 30.0)))
                    .with_rect(0.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
                    .with_color::<UiBase>(Vec4::new(0.3, 0.5, 0.3, 1.0))
                    .without_pointer_events()
                    .with_children(|tree| {
                        tree.add_node()
                            .boundary(Ab(Vec2::new(0.0, 5.0)), Ab(Vec2::new(tab_width, 20.0)))
                            .with_text("BUY", font_size)
                            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                            .with_color::<UiBase>(white)
                            .without_pointer_events()
                            .done();
                    })
                    .done();

                sell_tab = tree
                    .add_node()
                    .boundary(
                        Ab(Vec2::new(10.0 + tab_width, 50.0)),
                        Ab(Vec2::new(tab_width, 30.0)),
                    )
                    .with_rect(0.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
                    .with_color::<UiBase>(Vec4::new(0.2, 0.2, 0.2, 1.0))
                    .without_pointer_events()
                    .with_children(|tree| {
                        tree.add_node()
                            .boundary(Ab(Vec2::new(0.0, 5.0)), Ab(Vec2::new(tab_width, 20.0)))
                            .with_text("SELL", font_size)
                            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                            .with_color::<UiBase>(Vec4::new(0.6, 0.6, 0.6, 1.0))
                            .without_pointer_events()
                            .done();
                    })
                    .done();

                tree.add_node()
                    .boundary(
                        Ab(Vec2::new(10.0, 90.0)),
                        Ab(Vec2::new(panel_width - 20.0, panel_height - 140.0)),
                    )
                    .with_rect(0.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
                    .with_color::<UiBase>(Vec4::new(0.15, 0.15, 0.15, 1.0))
                    .without_pointer_events()
                    .with_children(|tree| {
                        let item_height = 35.0;
                        for index in 0..SHOP_ITEM_COUNT {
                            let item_y = 5.0 + index as f32 * item_height;
                            let item_width = panel_width - 24.0;

                            shop_item_rows[index] = tree
                                .add_node()
                                .boundary(
                                    Ab(Vec2::new(2.0, item_y)),
                                    Ab(Vec2::new(item_width, item_height - 2.0)),
                                )
                                .with_rect(0.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
                                .with_color::<UiBase>(Vec4::new(0.0, 0.0, 0.0, 0.0))
                                .without_pointer_events()
                                .with_children(|tree| {
                                    shop_item_name_texts[index] = tree
                                        .add_node()
                                        .boundary(
                                            Ab(Vec2::new(8.0, 8.0)),
                                            Ab(Vec2::new(item_width - 140.0, 20.0)),
                                        )
                                        .with_text("", font_size)
                                        .with_text_alignment(
                                            TextAlignment::Left,
                                            VerticalAlignment::Middle,
                                        )
                                        .with_color::<UiBase>(white)
                                        .without_pointer_events()
                                        .done();

                                    shop_item_owned_texts[index] = tree
                                        .add_node()
                                        .boundary(
                                            Ab(Vec2::new(item_width - 120.0, 8.0)),
                                            Ab(Vec2::new(50.0, 20.0)),
                                        )
                                        .with_text("", font_size)
                                        .with_text_alignment(
                                            TextAlignment::Right,
                                            VerticalAlignment::Middle,
                                        )
                                        .with_color::<UiBase>(Vec4::new(0.7, 0.7, 0.7, 1.0))
                                        .without_pointer_events()
                                        .done();

                                    shop_item_price_texts[index] = tree
                                        .add_node()
                                        .boundary(
                                            Ab(Vec2::new(item_width - 60.0, 8.0)),
                                            Ab(Vec2::new(52.0, 20.0)),
                                        )
                                        .with_text("", font_size)
                                        .with_text_alignment(
                                            TextAlignment::Right,
                                            VerticalAlignment::Middle,
                                        )
                                        .with_color::<UiBase>(Vec4::new(1.0, 0.9, 0.4, 1.0))
                                        .without_pointer_events()
                                        .done();
                                })
                                .done();
                        }
                    })
                    .done();

                tree.add_node()
                    .boundary(
                        Ab(Vec2::new(0.0, panel_height - 50.0)),
                        Ab(Vec2::new(panel_width, 50.0)),
                    )
                    .with_rect(0.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
                    .with_color::<UiBase>(Vec4::new(0.15, 0.15, 0.15, 1.0))
                    .without_pointer_events()
                    .with_children(|tree| {
                        shop_gold_text = tree
                            .add_node()
                            .boundary(
                                Ab(Vec2::new(15.0, 8.0)),
                                Ab(Vec2::new(panel_width - 30.0, 16.0)),
                            )
                            .with_text("Your Gold: 0 G", font_size)
                            .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                            .with_color::<UiBase>(gold)
                            .without_pointer_events()
                            .done();

                        shop_controls_text = tree
                            .add_node()
                            .boundary(
                                Ab(Vec2::new(15.0, 28.0)),
                                Ab(Vec2::new(panel_width - 30.0, 16.0)),
                            )
                            .with_text(
                                "[W/S] Select  [E] Buy  [Q] Switch Tab  [ESC] Close",
                                small_font,
                            )
                            .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                            .with_color::<UiBase>(Vec4::new(0.6, 0.6, 0.6, 1.0))
                            .without_pointer_events()
                            .done();
                    })
                    .done();
            })
            .done();

        self.buy_tab = buy_tab;
        self.sell_tab = sell_tab;
        self.shop_item_rows = shop_item_rows;
        self.shop_item_name_texts = shop_item_name_texts;
        self.shop_item_owned_texts = shop_item_owned_texts;
        self.shop_item_price_texts = shop_item_price_texts;
        self.shop_gold_text = shop_gold_text;
        self.shop_controls_text = shop_controls_text;

        let box_height = 120.0;
        let mut dialogue_header = placeholder;
        let mut dialogue_name_entity = placeholder;
        let mut dialogue_text_text = placeholder;
        self.dialogue_container = tree
            .add_node()
            .window(
                Rl(Vec2::new(50.0, 100.0)) + Ab(Vec2::new(0.0, -80.0 - box_height)),
                Rl(Vec2::new(80.0, 0.0)) + Ab(Vec2::new(0.0, box_height)),
                Anchor::BottomCenter,
            )
            .with_rect(0.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
            .with_color::<UiBase>(Vec4::new(0.0, 0.0, 0.0, 0.85))
            .with_layer(UiLayer::FloatingPanels)
            .with_visible(false)
            .without_pointer_events()
            .with_children(|tree| {
                dialogue_header = tree
                    .add_node()
                    .boundary(
                        Ab(Vec2::new(0.0, 0.0)),
                        Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, 30.0)),
                    )
                    .with_rect(0.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
                    .with_color::<UiBase>(Vec4::new(0.35, 0.2, 0.15, 0.9))
                    .without_pointer_events()
                    .with_children(|tree| {
                        dialogue_name_entity = tree
                            .add_node()
                            .boundary(
                                Ab(Vec2::new(15.0, 5.0)),
                                Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(-30.0, 20.0)),
                            )
                            .with_text("", font_size)
                            .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                            .with_color::<UiBase>(Vec4::new(0.7, 0.4, 0.3, 1.0))
                            .without_pointer_events()
                            .done();
                    })
                    .done();

                dialogue_text_text = tree
                    .add_node()
                    .boundary(
                        Ab(Vec2::new(15.0, 40.0)),
                        Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(-30.0, 40.0)),
                    )
                    .with_text("", font_size)
                    .with_text_alignment(TextAlignment::Left, VerticalAlignment::Top)
                    .with_color::<UiBase>(white)
                    .without_pointer_events()
                    .done();

                tree.add_node()
                    .boundary(
                        Ab(Vec2::new(15.0, 85.0)),
                        Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(-30.0, 20.0)),
                    )
                    .with_text("[Press E to continue]", font_size)
                    .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                    .with_color::<UiBase>(dim_text)
                    .without_pointer_events()
                    .done();
            })
            .done();
        self.dialogue_header = dialogue_header;
        self.dialogue_name_entity = dialogue_name_entity;
        self.dialogue_name_text = dialogue_name_entity;
        self.dialogue_text_text = dialogue_text_text;

        let mut hint_text = placeholder;
        self.hint_container = tree
            .add_node()
            .window(
                Rl(Vec2::new(50.0, 100.0)) + Ab(Vec2::new(0.0, -200.0)),
                Ab(Vec2::new(220.0, 30.0)),
                Anchor::BottomCenter,
            )
            .with_rect(4.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
            .with_color::<UiBase>(Vec4::new(0.0, 0.0, 0.0, 0.7))
            .with_visible(false)
            .without_pointer_events()
            .with_children(|tree| {
                hint_text = tree
                    .add_node()
                    .boundary(Ab(Vec2::new(0.0, 0.0)), Rl(Vec2::new(100.0, 100.0)))
                    .with_text("", font_size)
                    .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                    .with_color::<UiBase>(Vec4::new(1.0, 1.0, 0.8, 1.0))
                    .without_pointer_events()
                    .done();
            })
            .done();
        self.hint_text = hint_text;

        tree.finish();
    }

    pub fn update(
        &mut self,
        game: &GameWorld,
        phase: GamePhase,
        world: &mut World,
    ) -> Option<GamePhase> {
        let mut new_phase = None;

        let show_main_menu = phase == GamePhase::MainMenu;
        let show_paused = phase == GamePhase::Paused;
        let show_playing = phase == GamePhase::Playing;

        world.ui_set_visible(self.main_menu_screen, show_main_menu);
        world.ui_set_visible(self.pause_screen, show_paused);
        world.ui_set_visible(self.hud_screen, show_playing);

        if show_main_menu
            && world
                .widget::<UiButtonData>(self.start_button)
                .is_some_and(|d| d.clicked)
        {
            new_phase = Some(GamePhase::Playing);
            return new_phase;
        }

        if show_paused
            && world
                .widget::<UiButtonData>(self.resume_button)
                .is_some_and(|d| d.clicked)
        {
            new_phase = Some(GamePhase::Playing);
            return new_phase;
        }

        if show_playing {
            self.update_hud(game, world);
            self.update_hotbar(game, world);
            self.update_tree_health(game, world);
            self.update_shop(game, world);
            self.update_dialogue(game, world);
            self.update_hints(game, world);
        } else {
            world.ui_set_visible(self.tree_health_container, false);
            world.ui_set_visible(self.shop_screen, false);
            world.ui_set_visible(self.dialogue_container, false);
            world.ui_set_visible(self.hint_container, false);
        }

        new_phase
    }

    fn update_hud(&self, game: &GameWorld, world: &mut World) {
        let time_str = format_time(game.resources.hour);
        world.ui_set_text(
            self.day_time_text,
            &format!(
                "Day {} - {} - {}",
                game.resources.day,
                game.resources.season.name(),
                time_str
            ),
        );

        world.ui_set_text(
            self.weather_text,
            &format!("Weather: {}", game.resources.weather.name()),
        );

        let (stamina, max_stamina, equipped_tool) = game
            .resources
            .player_entity
            .and_then(|entity| game.get_player(entity))
            .map(|player| (player.stamina, player.max_stamina, player.equipped_tool))
            .unwrap_or((100.0, 100.0, ToolType::Hand));

        let stamina_pct = stamina / max_stamina;
        world.ui_progress_bar_set_value(self.stamina_bar, stamina_pct);

        let stamina_color = if stamina_pct > 0.5 {
            Vec4::new(0.2, 0.6, 0.9, 1.0)
        } else if stamina_pct > 0.25 {
            Vec4::new(0.9, 0.7, 0.2, 1.0)
        } else {
            Vec4::new(0.9, 0.3, 0.2, 1.0)
        };
        if let Some(color) = world.get_ui_node_color_mut(self.stamina_bar_fill) {
            color.colors[UiBase::INDEX] = Some(stamina_color);
        }

        world.ui_set_text(self.gold_text, &format!("Gold: {} G", game.resources.money));

        world.ui_set_text(self.tool_text, &format!("Tool: {}", equipped_tool.name()));
    }

    fn update_hotbar(&self, game: &GameWorld, world: &mut World) {
        let tools = [
            Some(ToolType::Hoe),
            Some(ToolType::WateringCan),
            Some(ToolType::Axe),
            Some(ToolType::Pickaxe),
            Some(ToolType::Scythe),
            Some(ToolType::Sword),
            None,
            None,
            None,
            None,
        ];

        for (index, tool) in tools.iter().enumerate() {
            let is_selected = game.resources.inventory.selected_slot == index;
            let bg_color = if is_selected {
                Vec4::new(0.3, 0.5, 0.8, 0.8)
            } else {
                Vec4::new(0.2, 0.2, 0.2, 0.8)
            };

            if let Some(color) = world.get_ui_node_color_mut(self.hotbar_slots[index]) {
                color.colors[UiBase::INDEX] = Some(bg_color);
            }

            if let Some(tool) = tool {
                let tool_name = match tool {
                    ToolType::Hand => "",
                    ToolType::Hoe => "Hoe",
                    ToolType::WateringCan => "Can",
                    ToolType::Axe => "Axe",
                    ToolType::Pickaxe => "Pick",
                    ToolType::Scythe => "Syth",
                    ToolType::Sword => "Swrd",
                };
                world.ui_set_text(self.hotbar_name_texts[index], tool_name);
                world.ui_set_text(self.hotbar_qty_texts[index], "");
            } else {
                let slot = &game.resources.inventory.hotbar[index];
                if let Some(item_id) = slot.item_id
                    && let Some(definition) = get_item_definition(item_id)
                {
                    let short_name = get_short_item_name(definition.name);
                    world.ui_set_text(self.hotbar_name_texts[index], short_name);
                    if slot.quantity > 1 {
                        world.ui_set_text(
                            self.hotbar_qty_texts[index],
                            &format!("x{}", slot.quantity),
                        );
                    } else {
                        world.ui_set_text(self.hotbar_qty_texts[index], "");
                    }
                } else {
                    world.ui_set_text(self.hotbar_name_texts[index], "");
                    world.ui_set_text(self.hotbar_qty_texts[index], "");
                }
            }
        }
    }

    fn update_tree_health(&self, game: &GameWorld, world: &mut World) {
        let show = game
            .resources
            .targeted_tree
            .and_then(|entity| game.get_tree(entity))
            .is_some_and(|tree| tree.state == TreeState::Standing);

        world.ui_set_visible(self.tree_health_container, show);

        if show
            && let Some(tree_entity) = game.resources.targeted_tree
            && let Some(tree) = game.get_tree(tree_entity)
        {
            let health_pct = tree.health / tree.max_health;
            world.ui_progress_bar_set_value(self.tree_health_bar, health_pct);

            let health_color = if health_pct > 0.66 {
                Vec4::new(0.2, 0.8, 0.2, 1.0)
            } else if health_pct > 0.33 {
                Vec4::new(0.9, 0.7, 0.1, 1.0)
            } else {
                Vec4::new(0.9, 0.2, 0.1, 1.0)
            };
            if let Some(color) = world.get_ui_node_color_mut(self.tree_health_bar_fill) {
                color.colors[UiBase::INDEX] = Some(health_color);
            }
        }
    }

    fn update_shop(&self, game: &GameWorld, world: &mut World) {
        let show = game.resources.shop.is_some();
        world.ui_set_visible(self.shop_screen, show);

        if let Some(shop) = &game.resources.shop {
            let buy_selected = shop.mode == ShopMode::Buy;

            let buy_bg = if buy_selected {
                Vec4::new(0.3, 0.5, 0.3, 1.0)
            } else {
                Vec4::new(0.2, 0.2, 0.2, 1.0)
            };
            let sell_bg = if !buy_selected {
                Vec4::new(0.3, 0.5, 0.3, 1.0)
            } else {
                Vec4::new(0.2, 0.2, 0.2, 1.0)
            };

            if let Some(color) = world.get_ui_node_color_mut(self.buy_tab) {
                color.colors[UiBase::INDEX] = Some(buy_bg);
            }
            if let Some(color) = world.get_ui_node_color_mut(self.sell_tab) {
                color.colors[UiBase::INDEX] = Some(sell_bg);
            }

            for (index, shop_item) in game.resources.shop_items.iter().enumerate() {
                if index >= SHOP_ITEM_COUNT {
                    break;
                }

                let is_item_selected = index == shop.selected;
                let row_bg = if is_item_selected {
                    Vec4::new(0.3, 0.4, 0.3, 1.0)
                } else {
                    Vec4::new(0.0, 0.0, 0.0, 0.0)
                };

                if let Some(color) = world.get_ui_node_color_mut(self.shop_item_rows[index]) {
                    color.colors[UiBase::INDEX] = Some(row_bg);
                }

                let name = get_item_definition(shop_item.item_id)
                    .map(|definition| definition.name)
                    .unwrap_or("Unknown");

                let price = if shop.mode == ShopMode::Buy {
                    shop_item.buy_price
                } else {
                    shop_item.sell_price
                };

                let owned = game.resources.inventory.count_item(shop_item.item_id);

                world.ui_set_text(self.shop_item_name_texts[index], name);
                world.ui_set_text(self.shop_item_owned_texts[index], &format!("x{}", owned));
                world.ui_set_text(self.shop_item_price_texts[index], &format!("{} G", price));
            }

            world.ui_set_text(
                self.shop_gold_text,
                &format!("Your Gold: {} G", game.resources.money),
            );

            let action_str = if shop.mode == ShopMode::Buy {
                "[E] Buy"
            } else {
                "[E] Sell"
            };
            world.ui_set_text(
                self.shop_controls_text,
                &format!("[W/S] Select  {}  [Q] Switch Tab  [ESC] Close", action_str),
            );
        }
    }

    fn update_dialogue(&self, game: &GameWorld, world: &mut World) {
        let show = game.resources.dialogue.is_some() && game.resources.shop.is_none();
        world.ui_set_visible(self.dialogue_container, show);

        if let Some(dialogue) = &game.resources.dialogue
            && let Some(npc) = game.get_npc(dialogue.npc)
            && let Some(definition) = get_npc_definition(npc.npc_type)
        {
            world.ui_set_text(self.dialogue_name_text, definition.name);

            if let Some(color) = world.get_ui_node_color_mut(self.dialogue_header) {
                color.colors[UiBase::INDEX] = Some(Vec4::new(
                    definition.color[0] * 0.5,
                    definition.color[1] * 0.5,
                    definition.color[2] * 0.5,
                    0.9,
                ));
            }

            if let Some(color) = world.get_ui_node_color_mut(self.dialogue_name_entity) {
                color.colors[UiBase::INDEX] = Some(Vec4::new(
                    definition.color[0],
                    definition.color[1],
                    definition.color[2],
                    1.0,
                ));
            }

            if dialogue.line_index < definition.dialogue.len() {
                world.ui_set_text(
                    self.dialogue_text_text,
                    definition.dialogue[dialogue.line_index],
                );
            }
        }
    }

    fn update_hints(&self, game: &GameWorld, world: &mut World) {
        if game.resources.shop.is_some() || game.resources.dialogue.is_some() {
            world.ui_set_visible(self.hint_container, false);
            return;
        }

        if social::should_show_shop_hint(game) {
            world.ui_set_visible(self.hint_container, true);
            world.ui_set_text(self.hint_text, "[E] Open Shop");
            return;
        }

        if let Some(npc_entity) = social::get_nearest_npc(game)
            && let Some(npc) = game.get_npc(npc_entity)
            && let Some(definition) = get_npc_definition(npc.npc_type)
        {
            world.ui_set_visible(self.hint_container, true);
            world.ui_set_text(self.hint_text, &format!("[E] Talk to {}", definition.name));
            return;
        }

        world.ui_set_visible(self.hint_container, false);
    }
}
