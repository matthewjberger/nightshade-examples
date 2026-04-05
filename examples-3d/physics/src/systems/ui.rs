use crate::ecs::GameWorld;
use nightshade::ecs::text::components::{TextAlignment, VerticalAlignment};
use nightshade::prelude::*;

pub fn build_crosshair(world: &mut World) -> (Entity, Vec<Entity>) {
    let mut tree = UiTreeBuilder::new(world);
    let center = nalgebra_glm::Vec2::new(50.0, 50.0);
    let color = nalgebra_glm::Vec4::new(1.0, 1.0, 1.0, 0.7);

    let container = tree
        .add_node()
        .boundary(
            Vp(center) + Ab(nalgebra_glm::Vec2::new(-10.0, -10.0)),
            Vp(center) + Ab(nalgebra_glm::Vec2::new(10.0, 10.0)),
        )
        .without_pointer_events()
        .entity();

    tree.push_parent(container);

    let left = tree
        .add_node()
        .boundary(
            Ab(nalgebra_glm::Vec2::new(2.0, 9.0)),
            Ab(nalgebra_glm::Vec2::new(7.0, 11.0)),
        )
        .with_rect(0.0, 0.0, nalgebra_glm::Vec4::zeros())
        .with_color::<UiBase>(color)
        .without_pointer_events()
        .done();

    let right = tree
        .add_node()
        .boundary(
            Ab(nalgebra_glm::Vec2::new(13.0, 9.0)),
            Ab(nalgebra_glm::Vec2::new(18.0, 11.0)),
        )
        .with_rect(0.0, 0.0, nalgebra_glm::Vec4::zeros())
        .with_color::<UiBase>(color)
        .without_pointer_events()
        .done();

    let top = tree
        .add_node()
        .boundary(
            Ab(nalgebra_glm::Vec2::new(9.0, 2.0)),
            Ab(nalgebra_glm::Vec2::new(11.0, 7.0)),
        )
        .with_rect(0.0, 0.0, nalgebra_glm::Vec4::zeros())
        .with_color::<UiBase>(color)
        .without_pointer_events()
        .done();

    let bottom = tree
        .add_node()
        .boundary(
            Ab(nalgebra_glm::Vec2::new(9.0, 13.0)),
            Ab(nalgebra_glm::Vec2::new(11.0, 18.0)),
        )
        .with_rect(0.0, 0.0, nalgebra_glm::Vec4::zeros())
        .with_color::<UiBase>(color)
        .without_pointer_events()
        .done();

    tree.pop_parent();
    tree.finish();

    (container, vec![left, right, top, bottom])
}

pub fn build_note_overlay(world: &mut World) -> (Entity, Entity, Entity) {
    let mut tree = UiTreeBuilder::new(world);

    let panel_width = 500.0;
    let panel_height = 400.0;

    let overlay = tree
        .add_node()
        .boundary(
            Vp(nalgebra_glm::Vec2::new(50.0, 50.0))
                + Ab(nalgebra_glm::Vec2::new(
                    -panel_width / 2.0,
                    -panel_height / 2.0,
                )),
            Vp(nalgebra_glm::Vec2::new(50.0, 50.0))
                + Ab(nalgebra_glm::Vec2::new(
                    panel_width / 2.0,
                    panel_height / 2.0,
                )),
        )
        .with_rect(6.0, 2.0, nalgebra_glm::Vec4::new(0.471, 0.392, 0.275, 1.0))
        .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.961, 0.922, 0.824, 0.98))
        .with_visible(false)
        .without_pointer_events()
        .with_clip()
        .entity();

    tree.push_parent(overlay);

    let title_entity = tree
        .add_node()
        .boundary(
            Ab(nalgebra_glm::Vec2::new(20.0, 20.0)),
            Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(-20.0, 50.0)),
        )
        .with_text("", 20.0)
        .with_text_wrap()
        .with_text_alignment(TextAlignment::Center, VerticalAlignment::Top)
        .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.196, 0.157, 0.118, 1.0))
        .without_pointer_events()
        .done();

    tree.add_node()
        .boundary(
            Ab(nalgebra_glm::Vec2::new(20.0, 56.0)),
            Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(-20.0, 57.0)),
        )
        .with_rect(0.0, 0.0, nalgebra_glm::Vec4::zeros())
        .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.471, 0.392, 0.275, 0.5))
        .without_pointer_events();

    let content_entity = tree
        .add_node()
        .boundary(
            Ab(nalgebra_glm::Vec2::new(20.0, 70.0)),
            Rl(nalgebra_glm::Vec2::new(100.0, 100.0)) + Ab(nalgebra_glm::Vec2::new(-20.0, -20.0)),
        )
        .with_text("", 16.0)
        .with_text_wrap()
        .with_text_alignment(TextAlignment::Left, VerticalAlignment::Top)
        .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.157, 0.137, 0.098, 1.0))
        .without_pointer_events()
        .done();

    tree.pop_parent();
    tree.finish();

    (overlay, title_entity, content_entity)
}

pub fn spawn_label(
    world: &mut World,
    text: &str,
    position: Vec3,
    properties: nightshade::ecs::text::components::TextProperties,
) {
    nightshade::ecs::world::commands::spawn_3d_billboard_text_with_properties(
        world, text, position, properties,
    );
}

pub fn spawn_wall_label(
    world: &mut World,
    text: &str,
    position: Vec3,
    properties: nightshade::ecs::text::components::TextProperties,
) {
    let entity = nightshade::ecs::world::commands::spawn_3d_text_with_properties(
        world, text, position, properties,
    );
    if let Some(transform) = world.core.get_local_transform_mut(entity) {
        transform.rotate(std::f32::consts::PI, &nalgebra_glm::vec3(0.0, 1.0, 0.0));
    }
    nightshade::ecs::transform::commands::mark_local_transform_dirty(world, entity);
}

pub fn update_note_overlay(game_world: &mut GameWorld, world: &mut World) {
    if let Some(crosshair) = game_world.resources.crosshair_entity {
        world.ui_set_visible(crosshair, game_world.resources.reading_note.is_none());
    }

    if let Some(overlay) = game_world.resources.note_overlay_entity {
        if let Some(note_game_entity) = game_world.resources.reading_note {
            world.ui_set_visible(overlay, true);
            if game_world.resources.last_shown_note != Some(note_game_entity) {
                if let Some(note) = game_world.get_note(note_game_entity) {
                    let title = note.title.clone();
                    let content = note.content.clone();
                    if let Some(entity) = game_world.resources.note_title_entity {
                        world.ui_set_text(entity, &title);
                    }
                    if let Some(entity) = game_world.resources.note_content_entity {
                        world.ui_set_text(entity, &content);
                    }
                }
                game_world.resources.last_shown_note = Some(note_game_entity);
            }
        } else {
            world.ui_set_visible(overlay, false);
            if game_world.resources.last_shown_note.is_some() {
                game_world.resources.last_shown_note = None;
            }
        }
    }
}

pub fn debug_toggle_system(game_world: &mut GameWorld, world: &mut World) {
    let keyboard = &world.resources.input.keyboard;
    let key4_pressed = keyboard.is_key_pressed(KeyCode::Digit4);

    if key4_pressed && !game_world.resources.key4_was_pressed {
        game_world.resources.show_physics_debug = !game_world.resources.show_physics_debug;
        world.resources.physics.debug_draw = game_world.resources.show_physics_debug;
    }

    game_world.resources.key4_was_pressed = key4_pressed;
}
