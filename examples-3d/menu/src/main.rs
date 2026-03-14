use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Anchor {
    Center,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(MenuDemoState::default())
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
enum GameState {
    #[default]
    MainMenu,
    Settings,
    GraphicsSettings,
    AudioSettings,
    Playing,
    Paused,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
enum SettingsSource {
    #[default]
    MainMenu,
    Pause,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
enum TransitionState {
    #[default]
    None,
    FadingOut {
        target: GameState,
        timer: f32,
    },
    FadingOutToDialog {
        timer: f32,
    },
    FadingIn {
        timer: f32,
    },
}

const TRANSITION_DURATION: f32 = 0.25;

struct Button {
    entity: Entity,
    position: nalgebra_glm::Vec2,
    anchor: Anchor,
    width: f32,
    height: f32,
    base_color: nalgebra_glm::Vec4,
    hover_color: nalgebra_glm::Vec4,
    pressed_color: nalgebra_glm::Vec4,
    is_hovered: bool,
    is_pressed: bool,
}

struct Toggle {
    label_entity: Entity,
    value_entity: Entity,
    value_text_index: usize,
    position: nalgebra_glm::Vec2,
    anchor: Anchor,
    value: bool,
    height: f32,
    is_hovered: bool,
}

struct SliderRange {
    initial: f32,
    min: f32,
    max: f32,
}

struct Slider {
    label_entity: Entity,
    value_entity: Entity,
    value_text_index: usize,
    position: nalgebra_glm::Vec2,
    anchor: Anchor,
    value: f32,
    min_value: f32,
    max_value: f32,
    height: f32,
    is_hovered: bool,
    is_dragging: bool,
}

struct Dropdown {
    label_entity: Entity,
    value_entity: Entity,
    value_text_index: usize,
    position: nalgebra_glm::Vec2,
    anchor: Anchor,
    options: Vec<String>,
    selected_index: usize,
    height: f32,
    is_hovered: bool,
}

struct ConfirmDialog {
    title_entity: Entity,
    message_entity: Entity,
    yes_button: Button,
    no_button: Button,
    on_confirm: DialogAction,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum DialogAction {
    Quit,
    MainMenu,
}

struct GameSettings {
    sound_enabled: bool,
    music_enabled: bool,
    master_volume: f32,
    music_volume: f32,
    sfx_volume: f32,
    resolution_index: usize,
    fullscreen: bool,
    vsync: bool,
    quality_index: usize,
    game_speed: f32,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            sound_enabled: true,
            music_enabled: true,
            master_volume: 0.8,
            music_volume: 0.7,
            sfx_volume: 1.0,
            resolution_index: 2,
            fullscreen: false,
            vsync: true,
            quality_index: 2,
            game_speed: 1.0,
        }
    }
}

#[derive(Default)]
struct MenuDemoState {
    game_state: GameState,
    settings_source: SettingsSource,
    settings: GameSettings,
    transition: TransitionState,
    global_alpha: f32,

    camera_entity: Option<Entity>,
    title_entity: Option<Entity>,
    subtitle_entity: Option<Entity>,

    main_menu_buttons: Vec<Button>,
    settings_buttons: Vec<Button>,
    graphics_toggles: Vec<Toggle>,
    graphics_dropdowns: Vec<Dropdown>,
    audio_sliders: Vec<Slider>,
    audio_toggles: Vec<Toggle>,
    back_button: Option<Button>,
    pause_buttons: Vec<Button>,

    confirm_dialog: Option<ConfirmDialog>,
    pending_dialog: Option<(String, String, DialogAction)>,

    game_entities: Vec<Entity>,
    game_rotation: f32,
    paused_text_entity: Option<Entity>,

    active_slider: Option<usize>,
    screen_width: f32,
    screen_height: f32,
}

fn get_element_screen_bounds(
    position: nalgebra_glm::Vec2,
    anchor: Anchor,
    width: f32,
    height: f32,
    screen_width: f32,
    screen_height: f32,
) -> (f32, f32, f32, f32) {
    let base_x = match anchor {
        Anchor::Center => screen_width * 0.5,
    };

    let base_y = match anchor {
        Anchor::Center => screen_height * 0.5,
    };

    let screen_x = base_x + position.x;
    let screen_y = base_y + position.y;

    let left = screen_x - width * 0.5;
    let right = screen_x + width * 0.5;
    let top = screen_y - height * 0.5;
    let bottom = screen_y + height * 0.5;

    (left, right, top, bottom)
}

fn is_point_in_bounds(
    position: nalgebra_glm::Vec2,
    anchor: Anchor,
    width: f32,
    height: f32,
    mouse_pos: nalgebra_glm::Vec2,
    screen_size: nalgebra_glm::Vec2,
) -> bool {
    let (left, right, top, bottom) = get_element_screen_bounds(
        position,
        anchor,
        width,
        height,
        screen_size.x,
        screen_size.y,
    );
    mouse_pos.x >= left && mouse_pos.x <= right && mouse_pos.y >= top && mouse_pos.y <= bottom
}

fn apply_alpha(color: nalgebra_glm::Vec4, alpha: f32) -> nalgebra_glm::Vec4 {
    nalgebra_glm::vec4(color.x, color.y, color.z, color.w * alpha)
}

fn update_button_visuals(world: &mut World, button: &Button, global_alpha: f32) {
    let color = if button.is_pressed {
        button.pressed_color
    } else if button.is_hovered {
        button.hover_color
    } else {
        button.base_color
    };

    if let Some(hud_text) = world.core.get_text_mut(button.entity) {
        hud_text.properties.color = apply_alpha(color, global_alpha);
        hud_text.dirty = true;
    }
}

fn update_toggle_visuals(world: &mut World, toggle: &Toggle, global_alpha: f32) {
    let label_color = if toggle.is_hovered {
        nalgebra_glm::vec4(1.0, 0.9, 0.3, 1.0)
    } else {
        nalgebra_glm::vec4(0.7, 0.7, 0.7, 1.0)
    };

    if let Some(hud_text) = world.core.get_text_mut(toggle.label_entity) {
        hud_text.properties.color = apply_alpha(label_color, global_alpha);
        hud_text.dirty = true;
    }

    let value_color = if toggle.value {
        nalgebra_glm::vec4(0.3, 1.0, 0.3, 1.0)
    } else {
        nalgebra_glm::vec4(0.6, 0.6, 0.6, 1.0)
    };

    if let Some(hud_text) = world.core.get_text_mut(toggle.value_entity) {
        hud_text.properties.color = apply_alpha(value_color, global_alpha);
        hud_text.dirty = true;
    }
}

fn update_slider_visuals(world: &mut World, slider: &Slider, global_alpha: f32) {
    let label_color = if slider.is_hovered || slider.is_dragging {
        nalgebra_glm::vec4(1.0, 0.9, 0.3, 1.0)
    } else {
        nalgebra_glm::vec4(0.7, 0.7, 0.7, 1.0)
    };

    if let Some(hud_text) = world.core.get_text_mut(slider.label_entity) {
        hud_text.properties.color = apply_alpha(label_color, global_alpha);
        hud_text.dirty = true;
    }

    let value_color = if slider.is_dragging {
        nalgebra_glm::vec4(1.0, 0.9, 0.3, 1.0)
    } else {
        nalgebra_glm::vec4(0.5, 0.8, 1.0, 1.0)
    };

    if let Some(hud_text) = world.core.get_text_mut(slider.value_entity) {
        hud_text.properties.color = apply_alpha(value_color, global_alpha);
        hud_text.dirty = true;
    }
}

fn update_dropdown_visuals(world: &mut World, dropdown: &Dropdown, global_alpha: f32) {
    let label_color = if dropdown.is_hovered {
        nalgebra_glm::vec4(1.0, 0.9, 0.3, 1.0)
    } else {
        nalgebra_glm::vec4(0.7, 0.7, 0.7, 1.0)
    };

    if let Some(hud_text) = world.core.get_text_mut(dropdown.label_entity) {
        hud_text.properties.color = apply_alpha(label_color, global_alpha);
        hud_text.dirty = true;
    }
}

impl MenuDemoState {
    fn start_transition(&mut self, target: GameState) {
        self.transition = TransitionState::FadingOut {
            target,
            timer: TRANSITION_DURATION,
        };
    }

    fn start_dialog_transition(&mut self, title: &str, message: &str, action: DialogAction) {
        self.pending_dialog = Some((title.to_string(), message.to_string(), action));
        self.transition = TransitionState::FadingOutToDialog {
            timer: TRANSITION_DURATION,
        };
    }

    fn create_button(
        &self,
        world: &mut World,
        label: &str,
        position: nalgebra_glm::Vec2,
        anchor: Anchor,
        font_size: f32,
    ) -> Button {
        let base_color = nalgebra_glm::vec4(0.8, 0.8, 0.8, 1.0);
        let hover_color = nalgebra_glm::vec4(1.0, 0.9, 0.3, 1.0);
        let pressed_color = nalgebra_glm::vec4(0.9, 0.7, 0.1, 1.0);

        let props = TextProperties {
            font_size,
            color: apply_alpha(base_color, self.global_alpha),
            alignment: TextAlignment::Center,
            outline_width: 0.05,
            outline_color: apply_alpha(nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0), self.global_alpha),
            ..Default::default()
        };

        let entity = spawn_ui_text_with_properties(world, label, nalgebra_glm::Vec2::zeros(), props);

        let char_width = font_size * 0.55;
        let width = label.len() as f32 * char_width;
        let height = font_size * 1.2;

        Button {
            entity,
            position,
            anchor,
            width,
            height,
            base_color,
            hover_color,
            pressed_color,
            is_hovered: false,
            is_pressed: false,
        }
    }

    fn create_toggle(
        &self,
        world: &mut World,
        label: &str,
        position: nalgebra_glm::Vec2,
        anchor: Anchor,
        initial_value: bool,
    ) -> Toggle {
        let label_props = TextProperties {
            font_size: 24.0,
            color: apply_alpha(nalgebra_glm::vec4(0.7, 0.7, 0.7, 1.0), self.global_alpha),
            alignment: TextAlignment::Left,
            outline_width: 0.03,
            outline_color: apply_alpha(nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0), self.global_alpha),
            ..Default::default()
        };

        let label_entity =
            spawn_ui_text_with_properties(world, label, nalgebra_glm::Vec2::zeros(), label_props);

        let value_color = if initial_value {
            nalgebra_glm::vec4(0.3, 1.0, 0.3, 1.0)
        } else {
            nalgebra_glm::vec4(0.6, 0.6, 0.6, 1.0)
        };

        let value_props = TextProperties {
            font_size: 24.0,
            color: apply_alpha(value_color, self.global_alpha),
            alignment: TextAlignment::Right,
            outline_width: 0.03,
            outline_color: apply_alpha(nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0), self.global_alpha),
            ..Default::default()
        };

        let value_text = if initial_value { "[ON]" } else { "[OFF]" };
        let value_entity =
            spawn_ui_text_with_properties(world, value_text, nalgebra_glm::Vec2::zeros(), value_props);

        let value_text_index = world
            .core.get_text(value_entity)
            .map(|t| t.text_index)
            .unwrap_or(0);

        Toggle {
            label_entity,
            value_entity,
            value_text_index,
            position,
            anchor,
            value: initial_value,
            height: 30.0,
            is_hovered: false,
        }
    }

    fn create_slider(
        &self,
        world: &mut World,
        label: &str,
        position: nalgebra_glm::Vec2,
        anchor: Anchor,
        range: SliderRange,
    ) -> Slider {
        let label_props = TextProperties {
            font_size: 24.0,
            color: apply_alpha(nalgebra_glm::vec4(0.7, 0.7, 0.7, 1.0), self.global_alpha),
            alignment: TextAlignment::Left,
            outline_width: 0.03,
            outline_color: apply_alpha(nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0), self.global_alpha),
            ..Default::default()
        };

        let label_entity =
            spawn_ui_text_with_properties(world, label, nalgebra_glm::Vec2::zeros(), label_props);

        let value_props = TextProperties {
            font_size: 24.0,
            color: apply_alpha(nalgebra_glm::vec4(0.5, 0.8, 1.0, 1.0), self.global_alpha),
            alignment: TextAlignment::Right,
            outline_width: 0.03,
            outline_color: apply_alpha(nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0), self.global_alpha),
            ..Default::default()
        };

        let percentage = ((range.initial - range.min) / (range.max - range.min) * 100.0) as i32;
        let bar = create_slider_bar(percentage);
        let value_entity =
            spawn_ui_text_with_properties(world, &bar, nalgebra_glm::Vec2::zeros(), value_props);

        let value_text_index = world
            .core.get_text(value_entity)
            .map(|t| t.text_index)
            .unwrap_or(0);

        Slider {
            label_entity,
            value_entity,
            value_text_index,
            position,
            anchor,
            value: range.initial,
            min_value: range.min,
            max_value: range.max,
            height: 30.0,
            is_hovered: false,
            is_dragging: false,
        }
    }

    fn create_dropdown(
        &self,
        world: &mut World,
        label: &str,
        position: nalgebra_glm::Vec2,
        anchor: Anchor,
        options: Vec<String>,
        selected_index: usize,
    ) -> Dropdown {
        let label_props = TextProperties {
            font_size: 24.0,
            color: apply_alpha(nalgebra_glm::vec4(0.7, 0.7, 0.7, 1.0), self.global_alpha),
            alignment: TextAlignment::Left,
            outline_width: 0.03,
            outline_color: apply_alpha(nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0), self.global_alpha),
            ..Default::default()
        };

        let label_entity =
            spawn_ui_text_with_properties(world, label, nalgebra_glm::Vec2::zeros(), label_props);

        let value_props = TextProperties {
            font_size: 24.0,
            color: apply_alpha(nalgebra_glm::vec4(0.5, 0.8, 1.0, 1.0), self.global_alpha),
            alignment: TextAlignment::Right,
            outline_width: 0.03,
            outline_color: apply_alpha(nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0), self.global_alpha),
            ..Default::default()
        };

        let value_text = format!("< {} >", &options[selected_index]);

        let value_entity =
            spawn_ui_text_with_properties(world, &value_text, nalgebra_glm::Vec2::zeros(), value_props);

        let value_text_index = world
            .core.get_text(value_entity)
            .map(|t| t.text_index)
            .unwrap_or(0);

        Dropdown {
            label_entity,
            value_entity,
            value_text_index,
            position,
            anchor,
            options,
            selected_index,
            height: 30.0,
            is_hovered: false,
        }
    }

    fn create_confirm_dialog(
        &self,
        world: &mut World,
        title: &str,
        message: &str,
        action: DialogAction,
    ) -> ConfirmDialog {
        let title_props = TextProperties {
            font_size: 36.0,
            color: nalgebra_glm::vec4(1.0, 0.8, 0.2, 1.0),
            alignment: TextAlignment::Center,
            outline_width: 0.06,
            outline_color: nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0),
            ..Default::default()
        };

        let title_entity = spawn_ui_text_with_properties(
            world,
            title,
            nalgebra_glm::Vec2::zeros(),
            title_props,
        );

        let message_props = TextProperties {
            font_size: 24.0,
            color: nalgebra_glm::vec4(0.9, 0.9, 0.9, 1.0),
            alignment: TextAlignment::Center,
            outline_width: 0.03,
            outline_color: nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0),
            ..Default::default()
        };

        let message_entity = spawn_ui_text_with_properties(
            world,
            message,
            nalgebra_glm::Vec2::zeros(),
            message_props,
        );

        let yes_button = self.create_button(
            world,
            "YES",
            nalgebra_glm::vec2(-60.0, 50.0),
            Anchor::Center,
            32.0,
        );

        let no_button = self.create_button(
            world,
            "NO",
            nalgebra_glm::vec2(60.0, 50.0),
            Anchor::Center,
            32.0,
        );

        ConfirmDialog {
            title_entity,
            message_entity,
            yes_button,
            no_button,
            on_confirm: action,
        }
    }

    fn despawn_confirm_dialog(&mut self, world: &mut World, restore_background: bool) {
        if let Some(dialog) = self.confirm_dialog.take() {
            world.despawn_entities(&[
                dialog.title_entity,
                dialog.message_entity,
                dialog.yes_button.entity,
                dialog.no_button.entity,
            ]);
        }
        if restore_background {
            self.set_background_ui_alpha(world, self.global_alpha);
        }
    }

    fn set_background_ui_alpha(&self, world: &mut World, alpha: f32) {
        if let Some(entity) = self.title_entity
            && let Some(hud_text) = world.core.get_text_mut(entity)
        {
            hud_text.properties.color.w = alpha;
            hud_text.properties.outline_color.w = alpha;
            hud_text.dirty = true;
        }
        if let Some(entity) = self.subtitle_entity
            && let Some(hud_text) = world.core.get_text_mut(entity)
        {
            hud_text.properties.color.w = alpha;
            hud_text.properties.outline_color.w = alpha;
            hud_text.dirty = true;
        }
        if let Some(entity) = self.paused_text_entity
            && let Some(hud_text) = world.core.get_text_mut(entity)
        {
            hud_text.properties.color.w = alpha;
            hud_text.properties.outline_color.w = alpha;
            hud_text.dirty = true;
        }
        for button in &self.main_menu_buttons {
            if let Some(hud_text) = world.core.get_text_mut(button.entity) {
                hud_text.properties.color.w = alpha;
                hud_text.properties.outline_color.w = alpha;
                hud_text.dirty = true;
            }
        }
        for button in &self.pause_buttons {
            if let Some(hud_text) = world.core.get_text_mut(button.entity) {
                hud_text.properties.color.w = alpha;
                hud_text.properties.outline_color.w = alpha;
                hud_text.dirty = true;
            }
        }
    }

    fn despawn_ui_elements(&mut self, world: &mut World) {
        if let Some(entity) = self.title_entity.take() {
            world.despawn_entities(&[entity]);
        }
        if let Some(entity) = self.subtitle_entity.take() {
            world.despawn_entities(&[entity]);
        }
        if let Some(entity) = self.paused_text_entity.take() {
            world.despawn_entities(&[entity]);
        }

        for button in self.main_menu_buttons.drain(..) {
            world.despawn_entities(&[button.entity]);
        }

        for button in self.settings_buttons.drain(..) {
            world.despawn_entities(&[button.entity]);
        }

        for toggle in self.graphics_toggles.drain(..) {
            world.despawn_entities(&[toggle.label_entity, toggle.value_entity]);
        }

        for dropdown in self.graphics_dropdowns.drain(..) {
            world.despawn_entities(&[dropdown.label_entity, dropdown.value_entity]);
        }

        for slider in self.audio_sliders.drain(..) {
            world.despawn_entities(&[slider.label_entity, slider.value_entity]);
        }

        for toggle in self.audio_toggles.drain(..) {
            world.despawn_entities(&[toggle.label_entity, toggle.value_entity]);
        }

        if let Some(button) = self.back_button.take() {
            world.despawn_entities(&[button.entity]);
        }

        for button in self.pause_buttons.drain(..) {
            world.despawn_entities(&[button.entity]);
        }

        self.despawn_confirm_dialog(world, false);
    }

    fn despawn_game_entities(&mut self, world: &mut World) {
        world.resources.graphics.show_grid = false;

        if let Some(camera) = self.camera_entity
            && let Some(pan_orbit) = world.core.get_pan_orbit_camera_mut(camera)
        {
            pan_orbit.enabled = false;
        }

        for entity in self.game_entities.drain(..) {
            world.queue_command(WorldCommand::DespawnRecursive { entity });
        }
    }

    fn setup_main_menu(&mut self, world: &mut World) {
        self.despawn_ui_elements(world);
        self.despawn_game_entities(world);

        let title_props = TextProperties {
            font_size: 72.0,
            color: apply_alpha(nalgebra_glm::vec4(1.0, 0.8, 0.2, 1.0), self.global_alpha),
            alignment: TextAlignment::Center,
            outline_width: 0.08,
            outline_color: apply_alpha(nalgebra_glm::vec4(0.3, 0.1, 0.0, 1.0), self.global_alpha),
            ..Default::default()
        };

        self.title_entity = Some(spawn_ui_text_with_properties(
            world,
            "NIGHTSHADE",
            nalgebra_glm::Vec2::zeros(),
            title_props,
        ));

        let subtitle_props = TextProperties {
            font_size: 24.0,
            color: apply_alpha(nalgebra_glm::vec4(0.7, 0.7, 0.7, 1.0), self.global_alpha),
            alignment: TextAlignment::Center,
            outline_width: 0.03,
            outline_color: apply_alpha(nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0), self.global_alpha),
            ..Default::default()
        };

        self.subtitle_entity = Some(spawn_ui_text_with_properties(
            world,
            "Menu Demo",
            nalgebra_glm::Vec2::zeros(),
            subtitle_props,
        ));

        self.main_menu_buttons.push(self.create_button(
            world,
            "PLAY",
            nalgebra_glm::vec2(0.0, 0.0),
            Anchor::Center,
            48.0,
        ));

        self.main_menu_buttons.push(self.create_button(
            world,
            "SETTINGS",
            nalgebra_glm::vec2(0.0, 60.0),
            Anchor::Center,
            48.0,
        ));

        self.main_menu_buttons.push(self.create_button(
            world,
            "QUIT",
            nalgebra_glm::vec2(0.0, 120.0),
            Anchor::Center,
            48.0,
        ));
    }

    fn setup_settings_menu(&mut self, world: &mut World) {
        self.despawn_ui_elements(world);

        let title_props = TextProperties {
            font_size: 48.0,
            color: apply_alpha(nalgebra_glm::vec4(1.0, 0.8, 0.2, 1.0), self.global_alpha),
            alignment: TextAlignment::Center,
            outline_width: 0.06,
            outline_color: apply_alpha(nalgebra_glm::vec4(0.3, 0.1, 0.0, 1.0), self.global_alpha),
            ..Default::default()
        };

        self.title_entity = Some(spawn_ui_text_with_properties(
            world,
            "SETTINGS",
            nalgebra_glm::Vec2::zeros(),
            title_props,
        ));

        self.settings_buttons.push(self.create_button(
            world,
            "GRAPHICS",
            nalgebra_glm::vec2(0.0, -50.0),
            Anchor::Center,
            40.0,
        ));

        self.settings_buttons.push(self.create_button(
            world,
            "AUDIO",
            nalgebra_glm::vec2(0.0, 10.0),
            Anchor::Center,
            40.0,
        ));

        self.back_button = Some(self.create_button(
            world,
            "BACK",
            nalgebra_glm::vec2(0.0, 100.0),
            Anchor::Center,
            32.0,
        ));
    }

    fn setup_graphics_settings(&mut self, world: &mut World) {
        self.despawn_ui_elements(world);

        let title_props = TextProperties {
            font_size: 48.0,
            color: apply_alpha(nalgebra_glm::vec4(1.0, 0.8, 0.2, 1.0), self.global_alpha),
            alignment: TextAlignment::Center,
            outline_width: 0.06,
            outline_color: apply_alpha(nalgebra_glm::vec4(0.3, 0.1, 0.0, 1.0), self.global_alpha),
            ..Default::default()
        };

        self.title_entity = Some(spawn_ui_text_with_properties(
            world,
            "GRAPHICS",
            nalgebra_glm::Vec2::zeros(),
            title_props,
        ));

        let resolutions = vec![
            "1280x720".to_string(),
            "1600x900".to_string(),
            "1920x1080".to_string(),
            "2560x1440".to_string(),
            "3840x2160".to_string(),
        ];

        self.graphics_dropdowns.push(self.create_dropdown(
            world,
            "Resolution",
            nalgebra_glm::vec2(-140.0, -120.0),
            Anchor::Center,
            resolutions,
            self.settings.resolution_index,
        ));

        let qualities = vec![
            "Low".to_string(),
            "Medium".to_string(),
            "High".to_string(),
            "Ultra".to_string(),
        ];

        self.graphics_dropdowns.push(self.create_dropdown(
            world,
            "Quality",
            nalgebra_glm::vec2(-140.0, -70.0),
            Anchor::Center,
            qualities,
            self.settings.quality_index,
        ));

        self.graphics_toggles.push(self.create_toggle(
            world,
            "Fullscreen",
            nalgebra_glm::vec2(-140.0, -20.0),
            Anchor::Center,
            self.settings.fullscreen,
        ));

        self.graphics_toggles.push(self.create_toggle(
            world,
            "V-Sync",
            nalgebra_glm::vec2(-140.0, 30.0),
            Anchor::Center,
            self.settings.vsync,
        ));

        self.back_button = Some(self.create_button(
            world,
            "BACK",
            nalgebra_glm::vec2(0.0, 120.0),
            Anchor::Center,
            32.0,
        ));
    }

    fn setup_audio_settings(&mut self, world: &mut World) {
        self.despawn_ui_elements(world);

        let title_props = TextProperties {
            font_size: 48.0,
            color: apply_alpha(nalgebra_glm::vec4(1.0, 0.8, 0.2, 1.0), self.global_alpha),
            alignment: TextAlignment::Center,
            outline_width: 0.06,
            outline_color: apply_alpha(nalgebra_glm::vec4(0.3, 0.1, 0.0, 1.0), self.global_alpha),
            ..Default::default()
        };

        self.title_entity = Some(spawn_ui_text_with_properties(
            world,
            "AUDIO",
            nalgebra_glm::Vec2::zeros(),
            title_props,
        ));

        self.audio_sliders.push(self.create_slider(
            world,
            "Master Volume",
            nalgebra_glm::vec2(-140.0, -120.0),
            Anchor::Center,
            SliderRange {
                initial: self.settings.master_volume,
                min: 0.0,
                max: 1.0,
            },
        ));

        self.audio_sliders.push(self.create_slider(
            world,
            "Music Volume",
            nalgebra_glm::vec2(-140.0, -70.0),
            Anchor::Center,
            SliderRange {
                initial: self.settings.music_volume,
                min: 0.0,
                max: 1.0,
            },
        ));

        self.audio_sliders.push(self.create_slider(
            world,
            "SFX Volume",
            nalgebra_glm::vec2(-140.0, -20.0),
            Anchor::Center,
            SliderRange {
                initial: self.settings.sfx_volume,
                min: 0.0,
                max: 1.0,
            },
        ));

        self.audio_toggles.push(self.create_toggle(
            world,
            "Sound Enabled",
            nalgebra_glm::vec2(-140.0, 30.0),
            Anchor::Center,
            self.settings.sound_enabled,
        ));

        self.audio_toggles.push(self.create_toggle(
            world,
            "Music Enabled",
            nalgebra_glm::vec2(-140.0, 80.0),
            Anchor::Center,
            self.settings.music_enabled,
        ));

        self.back_button = Some(self.create_button(
            world,
            "BACK",
            nalgebra_glm::vec2(0.0, 150.0),
            Anchor::Center,
            32.0,
        ));
    }

    fn setup_playing_state(&mut self, world: &mut World) {
        self.despawn_ui_elements(world);

        world.resources.graphics.show_grid = true;

        if let Some(camera) = self.camera_entity {
            let transform_data = if let Some(pan_orbit) = world.core.get_pan_orbit_camera_mut(camera) {
                pan_orbit.enabled = true;
                Some(pan_orbit.compute_camera_transform())
            } else {
                None
            };

            if let Some((position, rotation)) = transform_data
                && let Some(local_transform) = world.core.get_local_transform_mut(camera)
            {
                local_transform.translation = position;
                local_transform.rotation = rotation;
            }
            world.core.set_local_transform_dirty(camera, LocalTransformDirty);
        }

        if self.game_entities.is_empty() {
            spawn_sun(world);

            let cube_entity = spawn_mesh(
                world,
                "Cube",
                Vec3::new(0.0, 0.5, 0.0),
                Vec3::new(1.0, 1.0, 1.0),
            );

            let cube_material = format!("GameCube_{}", cube_entity.id);
            material_registry_insert(
                &mut world.resources.material_registry,
                cube_material.clone(),
                Material {
                    base_color: [0.4, 0.6, 0.9, 1.0],
                    ..Default::default()
                },
            );
            if let Some(&index) = world
                .resources
                .material_registry
                .registry
                .name_to_index
                .get(&cube_material)
            {
                world
                    .resources
                    .material_registry
                    .registry
                    .add_reference(index);
            };
            world.core.set_material_ref(cube_entity, MaterialRef::new(cube_material));

            self.game_entities.push(cube_entity);

            self.game_rotation = 0.0;
        }

        let hint_props = TextProperties {
            font_size: 20.0,
            color: apply_alpha(nalgebra_glm::vec4(0.7, 0.7, 0.7, 1.0), self.global_alpha),
            alignment: TextAlignment::Center,
            outline_width: 0.03,
            outline_color: apply_alpha(nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0), self.global_alpha),
            ..Default::default()
        };

        self.subtitle_entity = Some(spawn_ui_text_with_properties(
            world,
            "Press P to pause",
            nalgebra_glm::Vec2::zeros(),
            hint_props,
        ));
    }

    fn setup_pause_menu(&mut self, world: &mut World) {
        self.despawn_ui_elements(world);

        let title_props = TextProperties {
            font_size: 56.0,
            color: apply_alpha(nalgebra_glm::vec4(1.0, 0.5, 0.2, 1.0), self.global_alpha),
            alignment: TextAlignment::Center,
            outline_width: 0.06,
            outline_color: apply_alpha(nalgebra_glm::vec4(0.3, 0.1, 0.0, 1.0), self.global_alpha),
            ..Default::default()
        };

        self.paused_text_entity = Some(spawn_ui_text_with_properties(
            world,
            "PAUSED",
            nalgebra_glm::Vec2::zeros(),
            title_props,
        ));

        self.pause_buttons.push(self.create_button(
            world,
            "RESUME",
            nalgebra_glm::vec2(0.0, -40.0),
            Anchor::Center,
            40.0,
        ));

        self.pause_buttons.push(self.create_button(
            world,
            "SETTINGS",
            nalgebra_glm::vec2(0.0, 20.0),
            Anchor::Center,
            40.0,
        ));

        self.pause_buttons.push(self.create_button(
            world,
            "MAIN MENU",
            nalgebra_glm::vec2(0.0, 80.0),
            Anchor::Center,
            40.0,
        ));
    }

    fn apply_state(&mut self, world: &mut World, state: GameState) {
        self.game_state = state;
        match state {
            GameState::MainMenu => self.setup_main_menu(world),
            GameState::Settings => self.setup_settings_menu(world),
            GameState::GraphicsSettings => self.setup_graphics_settings(world),
            GameState::AudioSettings => self.setup_audio_settings(world),
            GameState::Playing => self.setup_playing_state(world),
            GameState::Paused => self.setup_pause_menu(world),
        }
    }

    fn update_screen_size(&mut self, world: &World) {
        if let Some(window_handle) = &world.resources.window.handle {
            let size = window_handle.inner_size();
            self.screen_width = size.width as f32;
            self.screen_height = size.height as f32;
        }
    }

    fn update_transitions(&mut self, world: &mut World, delta_time: f32) {
        match self.transition {
            TransitionState::FadingOut { target, timer } => {
                let new_timer = timer - delta_time;
                if new_timer <= 0.0 {
                    self.global_alpha = 0.0;
                    self.apply_state(world, target);
                    self.transition = TransitionState::FadingIn {
                        timer: TRANSITION_DURATION,
                    };
                } else {
                    self.global_alpha = new_timer / TRANSITION_DURATION;
                    self.transition = TransitionState::FadingOut {
                        target,
                        timer: new_timer,
                    };
                }
                self.update_all_alphas(world);
            }
            TransitionState::FadingOutToDialog { timer } => {
                let new_timer = timer - delta_time;
                if new_timer <= 0.0 {
                    self.global_alpha = 0.0;
                    self.set_background_ui_alpha(world, 0.0);
                    if let Some((title, message, action)) = self.pending_dialog.take() {
                        self.confirm_dialog =
                            Some(self.create_confirm_dialog(world, &title, &message, action));
                    }
                    self.transition = TransitionState::FadingIn {
                        timer: TRANSITION_DURATION,
                    };
                } else {
                    self.global_alpha = new_timer / TRANSITION_DURATION;
                    self.transition = TransitionState::FadingOutToDialog { timer: new_timer };
                }
                self.update_all_alphas(world);
            }
            TransitionState::FadingIn { timer } => {
                let new_timer = timer - delta_time;
                if new_timer <= 0.0 {
                    self.global_alpha = 1.0;
                    self.transition = TransitionState::None;
                } else {
                    self.global_alpha = 1.0 - (new_timer / TRANSITION_DURATION);
                    self.transition = TransitionState::FadingIn { timer: new_timer };
                }
                self.update_all_alphas(world);
            }
            TransitionState::None => {}
        }
    }

    fn update_all_alphas(&mut self, world: &mut World) {
        let dialog_active = self.confirm_dialog.is_some();

        if !dialog_active {
            if let Some(entity) = self.title_entity
                && let Some(hud_text) = world.core.get_text_mut(entity)
            {
                hud_text.properties.color.w = self.global_alpha;
                hud_text.properties.outline_color.w = self.global_alpha;
                hud_text.dirty = true;
            }
            if let Some(entity) = self.subtitle_entity
                && let Some(hud_text) = world.core.get_text_mut(entity)
            {
                hud_text.properties.color.w = self.global_alpha;
                hud_text.properties.outline_color.w = self.global_alpha;
                hud_text.dirty = true;
            }
            if let Some(entity) = self.paused_text_entity
                && let Some(hud_text) = world.core.get_text_mut(entity)
            {
                hud_text.properties.color.w = self.global_alpha;
                hud_text.properties.outline_color.w = self.global_alpha;
                hud_text.dirty = true;
            }
            for button in &self.main_menu_buttons {
                if let Some(hud_text) = world.core.get_text_mut(button.entity) {
                    hud_text.properties.color.w = self.global_alpha;
                    hud_text.properties.outline_color.w = self.global_alpha;
                    hud_text.dirty = true;
                }
            }
            for button in &self.settings_buttons {
                if let Some(hud_text) = world.core.get_text_mut(button.entity) {
                    hud_text.properties.color.w = self.global_alpha;
                    hud_text.properties.outline_color.w = self.global_alpha;
                    hud_text.dirty = true;
                }
            }
            for button in &self.pause_buttons {
                if let Some(hud_text) = world.core.get_text_mut(button.entity) {
                    hud_text.properties.color.w = self.global_alpha;
                    hud_text.properties.outline_color.w = self.global_alpha;
                    hud_text.dirty = true;
                }
            }
            if let Some(ref button) = self.back_button
                && let Some(hud_text) = world.core.get_text_mut(button.entity)
            {
                hud_text.properties.color.w = self.global_alpha;
                hud_text.properties.outline_color.w = self.global_alpha;
                hud_text.dirty = true;
            }
            for toggle in &self.graphics_toggles {
                for entity in [toggle.label_entity, toggle.value_entity] {
                    if let Some(hud_text) = world.core.get_text_mut(entity) {
                        hud_text.properties.color.w = self.global_alpha;
                        hud_text.properties.outline_color.w = self.global_alpha;
                        hud_text.dirty = true;
                    }
                }
            }
            for toggle in &self.audio_toggles {
                for entity in [toggle.label_entity, toggle.value_entity] {
                    if let Some(hud_text) = world.core.get_text_mut(entity) {
                        hud_text.properties.color.w = self.global_alpha;
                        hud_text.properties.outline_color.w = self.global_alpha;
                        hud_text.dirty = true;
                    }
                }
            }
            for slider in &self.audio_sliders {
                for entity in [slider.label_entity, slider.value_entity] {
                    if let Some(hud_text) = world.core.get_text_mut(entity) {
                        hud_text.properties.color.w = self.global_alpha;
                        hud_text.properties.outline_color.w = self.global_alpha;
                        hud_text.dirty = true;
                    }
                }
            }
            for dropdown in &self.graphics_dropdowns {
                for entity in [dropdown.label_entity, dropdown.value_entity] {
                    if let Some(hud_text) = world.core.get_text_mut(entity) {
                        hud_text.properties.color.w = self.global_alpha;
                        hud_text.properties.outline_color.w = self.global_alpha;
                        hud_text.dirty = true;
                    }
                }
            }
        }
        if let Some(ref dialog) = self.confirm_dialog {
            for entity in [
                dialog.title_entity,
                dialog.message_entity,
                dialog.yes_button.entity,
                dialog.no_button.entity,
            ] {
                if let Some(hud_text) = world.core.get_text_mut(entity) {
                    hud_text.properties.color.w = self.global_alpha;
                    hud_text.properties.outline_color.w = self.global_alpha;
                    hud_text.dirty = true;
                }
            }
        }
    }

    fn process_buttons(
        &mut self,
        world: &mut World,
        buttons: &mut [Button],
        mouse_pos: nalgebra_glm::Vec2,
        mouse_down: bool,
        clicked: bool,
    ) -> Option<usize> {
        let screen_size = nalgebra_glm::vec2(self.screen_width, self.screen_height);
        let mut clicked_index = None;

        for (index, button) in buttons.iter_mut().enumerate() {
            let is_hovered = is_point_in_bounds(
                button.position,
                button.anchor,
                button.width,
                button.height,
                mouse_pos,
                screen_size,
            );

            let was_hovered = button.is_hovered;
            let was_pressed = button.is_pressed;

            button.is_hovered = is_hovered;
            button.is_pressed = is_hovered && mouse_down;

            if button.is_hovered != was_hovered || button.is_pressed != was_pressed {
                update_button_visuals(world, button, self.global_alpha);
            }

            if clicked && is_hovered {
                clicked_index = Some(index);
            }
        }

        clicked_index
    }

    fn run_main_menu(&mut self, world: &mut World) {
        if self.confirm_dialog.is_some() {
            self.run_confirm_dialog(world);
            return;
        }

        let mouse_pos = nalgebra_glm::vec2(
            world.resources.input.mouse.position.x,
            world.resources.input.mouse.position.y,
        );
        let clicked = world
            .resources
            .input
            .mouse
            .state
            .contains(MouseState::LEFT_JUST_RELEASED);
        let mouse_down = world
            .resources
            .input
            .mouse
            .state
            .contains(MouseState::LEFT_CLICKED);

        let mut buttons = std::mem::take(&mut self.main_menu_buttons);
        let clicked_index =
            self.process_buttons(world, &mut buttons, mouse_pos, mouse_down, clicked);
        self.main_menu_buttons = buttons;

        if let Some(index) = clicked_index {
            match index {
                0 => self.start_transition(GameState::Playing),
                1 => {
                    self.settings_source = SettingsSource::MainMenu;
                    self.start_transition(GameState::Settings);
                }
                2 => {
                    self.start_dialog_transition(
                        "QUIT GAME",
                        "Are you sure you want to quit?",
                        DialogAction::Quit,
                    );
                }
                _ => {}
            }
        }
    }

    fn run_confirm_dialog(&mut self, world: &mut World) {
        let mouse_pos = nalgebra_glm::vec2(
            world.resources.input.mouse.position.x,
            world.resources.input.mouse.position.y,
        );
        let screen_size = nalgebra_glm::vec2(self.screen_width, self.screen_height);
        let clicked = world
            .resources
            .input
            .mouse
            .state
            .contains(MouseState::LEFT_JUST_RELEASED);
        let mouse_down = world
            .resources
            .input
            .mouse
            .state
            .contains(MouseState::LEFT_CLICKED);

        if let Some(ref mut dialog) = self.confirm_dialog {
            let yes_hovered = is_point_in_bounds(
                dialog.yes_button.position,
                dialog.yes_button.anchor,
                dialog.yes_button.width,
                dialog.yes_button.height,
                mouse_pos,
                screen_size,
            );

            let no_hovered = is_point_in_bounds(
                dialog.no_button.position,
                dialog.no_button.anchor,
                dialog.no_button.width,
                dialog.no_button.height,
                mouse_pos,
                screen_size,
            );

            dialog.yes_button.is_hovered = yes_hovered;
            dialog.yes_button.is_pressed = yes_hovered && mouse_down;
            dialog.no_button.is_hovered = no_hovered;
            dialog.no_button.is_pressed = no_hovered && mouse_down;

            update_button_visuals(world, &dialog.yes_button, self.global_alpha);
            update_button_visuals(world, &dialog.no_button, self.global_alpha);

            if clicked {
                if yes_hovered {
                    let action = dialog.on_confirm;
                    self.despawn_confirm_dialog(world, false);
                    match action {
                        DialogAction::Quit => {
                            world.resources.window.should_exit = true;
                        }
                        DialogAction::MainMenu => {
                            self.start_transition(GameState::MainMenu);
                        }
                    }
                } else if no_hovered {
                    self.despawn_confirm_dialog(world, true);
                }
            }
        }
    }

    fn run_settings_menu(&mut self, world: &mut World) {
        let mouse_pos = nalgebra_glm::vec2(
            world.resources.input.mouse.position.x,
            world.resources.input.mouse.position.y,
        );
        let screen_size = nalgebra_glm::vec2(self.screen_width, self.screen_height);
        let clicked = world
            .resources
            .input
            .mouse
            .state
            .contains(MouseState::LEFT_JUST_RELEASED);
        let mouse_down = world
            .resources
            .input
            .mouse
            .state
            .contains(MouseState::LEFT_CLICKED);

        let mut buttons = std::mem::take(&mut self.settings_buttons);
        let clicked_index =
            self.process_buttons(world, &mut buttons, mouse_pos, mouse_down, clicked);
        self.settings_buttons = buttons;

        if let Some(ref mut back_button) = self.back_button {
            let is_hovered = is_point_in_bounds(
                back_button.position,
                back_button.anchor,
                back_button.width,
                back_button.height,
                mouse_pos,
                screen_size,
            );

            let was_hovered = back_button.is_hovered;
            let was_pressed = back_button.is_pressed;

            back_button.is_hovered = is_hovered;
            back_button.is_pressed = is_hovered && mouse_down;

            if back_button.is_hovered != was_hovered || back_button.is_pressed != was_pressed {
                update_button_visuals(world, back_button, self.global_alpha);
            }

            if clicked && is_hovered {
                match self.settings_source {
                    SettingsSource::MainMenu => self.start_transition(GameState::MainMenu),
                    SettingsSource::Pause => self.start_transition(GameState::Paused),
                }
            }
        }

        if let Some(index) = clicked_index {
            match index {
                0 => self.start_transition(GameState::GraphicsSettings),
                1 => self.start_transition(GameState::AudioSettings),
                _ => {}
            }
        }
    }

    fn run_graphics_settings(&mut self, world: &mut World) {
        let mouse_pos = nalgebra_glm::vec2(
            world.resources.input.mouse.position.x,
            world.resources.input.mouse.position.y,
        );
        let screen_size = nalgebra_glm::vec2(self.screen_width, self.screen_height);
        let clicked = world
            .resources
            .input
            .mouse
            .state
            .contains(MouseState::LEFT_JUST_RELEASED);
        let mouse_down = world
            .resources
            .input
            .mouse
            .state
            .contains(MouseState::LEFT_CLICKED);

        for (index, toggle) in self.graphics_toggles.iter_mut().enumerate() {
            let value_center = nalgebra_glm::vec2(toggle.position.x + 280.0, toggle.position.y);
            let is_hovered = is_point_in_bounds(
                value_center,
                toggle.anchor,
                80.0,
                toggle.height,
                mouse_pos,
                screen_size,
            );

            let was_hovered = toggle.is_hovered;
            toggle.is_hovered = is_hovered;

            if toggle.is_hovered != was_hovered {
                update_toggle_visuals(world, toggle, self.global_alpha);
            }

            if clicked && is_hovered {
                toggle.value = !toggle.value;
                let value_text = if toggle.value { "[ON]" } else { "[OFF]" };
                world
                    .resources
                    .text_cache
                    .set_text(toggle.value_text_index, value_text);
                update_toggle_visuals(world, toggle, self.global_alpha);

                match index {
                    0 => self.settings.fullscreen = toggle.value,
                    1 => self.settings.vsync = toggle.value,
                    _ => {}
                }
            }
        }

        for (index, dropdown) in self.graphics_dropdowns.iter_mut().enumerate() {
            let value_center = nalgebra_glm::vec2(dropdown.position.x + 280.0, dropdown.position.y);
            let is_hovered = is_point_in_bounds(
                value_center,
                dropdown.anchor,
                160.0,
                dropdown.height,
                mouse_pos,
                screen_size,
            );

            let was_hovered = dropdown.is_hovered;
            dropdown.is_hovered = is_hovered;

            if dropdown.is_hovered != was_hovered {
                update_dropdown_visuals(world, dropdown, self.global_alpha);
            }

            if clicked && is_hovered {
                dropdown.selected_index = (dropdown.selected_index + 1) % dropdown.options.len();
                let value_text = format!("< {} >", &dropdown.options[dropdown.selected_index]);
                world
                    .resources
                    .text_cache
                    .set_text(dropdown.value_text_index, value_text);
                if let Some(hud_text) = world.core.get_text_mut(dropdown.value_entity) {
                    hud_text.dirty = true;
                }

                match index {
                    0 => self.settings.resolution_index = dropdown.selected_index,
                    1 => self.settings.quality_index = dropdown.selected_index,
                    _ => {}
                }
            }
        }

        if let Some(ref mut back_button) = self.back_button {
            let is_hovered = is_point_in_bounds(
                back_button.position,
                back_button.anchor,
                back_button.width,
                back_button.height,
                mouse_pos,
                screen_size,
            );

            let was_hovered = back_button.is_hovered;
            let was_pressed = back_button.is_pressed;

            back_button.is_hovered = is_hovered;
            back_button.is_pressed = is_hovered && mouse_down;

            if back_button.is_hovered != was_hovered || back_button.is_pressed != was_pressed {
                update_button_visuals(world, back_button, self.global_alpha);
            }

            if clicked && is_hovered {
                self.start_transition(GameState::Settings);
            }
        }
    }

    fn run_audio_settings(&mut self, world: &mut World) {
        let mouse_pos = nalgebra_glm::vec2(
            world.resources.input.mouse.position.x,
            world.resources.input.mouse.position.y,
        );
        let screen_size = nalgebra_glm::vec2(self.screen_width, self.screen_height);
        let clicked = world
            .resources
            .input
            .mouse
            .state
            .contains(MouseState::LEFT_JUST_RELEASED);
        let mouse_down = world
            .resources
            .input
            .mouse
            .state
            .contains(MouseState::LEFT_CLICKED);

        for (index, toggle) in self.audio_toggles.iter_mut().enumerate() {
            let value_center = nalgebra_glm::vec2(toggle.position.x + 280.0, toggle.position.y);
            let is_hovered = is_point_in_bounds(
                value_center,
                toggle.anchor,
                80.0,
                toggle.height,
                mouse_pos,
                screen_size,
            );

            let was_hovered = toggle.is_hovered;
            toggle.is_hovered = is_hovered;

            if toggle.is_hovered != was_hovered {
                update_toggle_visuals(world, toggle, self.global_alpha);
            }

            if clicked && is_hovered {
                toggle.value = !toggle.value;
                let value_text = if toggle.value { "[ON]" } else { "[OFF]" };
                world
                    .resources
                    .text_cache
                    .set_text(toggle.value_text_index, value_text);
                update_toggle_visuals(world, toggle, self.global_alpha);

                match index {
                    0 => self.settings.sound_enabled = toggle.value,
                    1 => self.settings.music_enabled = toggle.value,
                    _ => {}
                }
            }
        }

        for slider in &mut self.audio_sliders {
            let value_center = nalgebra_glm::vec2(slider.position.x + 280.0, slider.position.y);
            let is_hovered = is_point_in_bounds(
                value_center,
                slider.anchor,
                160.0,
                slider.height,
                mouse_pos,
                screen_size,
            );

            let was_hovered = slider.is_hovered;
            slider.is_hovered = is_hovered;

            if slider.is_hovered != was_hovered && !slider.is_dragging {
                update_slider_visuals(world, slider, self.global_alpha);
            }
        }

        if mouse_down {
            if self.active_slider.is_none() {
                for (index, slider) in self.audio_sliders.iter().enumerate() {
                    if slider.is_hovered {
                        self.active_slider = Some(index);
                        break;
                    }
                }
            }

            if let Some(active_index) = self.active_slider
                && let Some(slider) = self.audio_sliders.get_mut(active_index)
            {
                slider.is_dragging = true;

                let value_center = nalgebra_glm::vec2(slider.position.x + 280.0, slider.position.y);
                let (left, right, _, _) = get_element_screen_bounds(
                    value_center,
                    slider.anchor,
                    160.0,
                    slider.height,
                    self.screen_width,
                    self.screen_height,
                );

                let normalized = ((mouse_pos.x - left) / (right - left)).clamp(0.0, 1.0);
                slider.value =
                    slider.min_value + normalized * (slider.max_value - slider.min_value);

                let percentage = ((slider.value - slider.min_value)
                    / (slider.max_value - slider.min_value)
                    * 100.0) as i32;
                let bar = create_slider_bar(percentage);

                world
                    .resources
                    .text_cache
                    .set_text(slider.value_text_index, bar);
                if let Some(hud_text) = world.core.get_text_mut(slider.value_entity) {
                    hud_text.dirty = true;
                }
                update_slider_visuals(world, slider, self.global_alpha);

                match active_index {
                    0 => self.settings.master_volume = slider.value,
                    1 => self.settings.music_volume = slider.value,
                    2 => self.settings.sfx_volume = slider.value,
                    _ => {}
                }
            }
        } else if let Some(active_index) = self.active_slider.take()
            && let Some(slider) = self.audio_sliders.get_mut(active_index)
        {
            slider.is_dragging = false;
            update_slider_visuals(world, slider, self.global_alpha);
        }

        if let Some(ref mut back_button) = self.back_button {
            let is_hovered = is_point_in_bounds(
                back_button.position,
                back_button.anchor,
                back_button.width,
                back_button.height,
                mouse_pos,
                screen_size,
            );

            let was_hovered = back_button.is_hovered;
            let was_pressed = back_button.is_pressed;

            back_button.is_hovered = is_hovered;
            back_button.is_pressed = is_hovered && mouse_down;

            if back_button.is_hovered != was_hovered || back_button.is_pressed != was_pressed {
                update_button_visuals(world, back_button, self.global_alpha);
            }

            if clicked && is_hovered {
                self.start_transition(GameState::Settings);
            }
        }
    }

    fn run_playing_state(&mut self, world: &mut World) {
        let delta_time = world.resources.window.timing.delta_time;
        self.game_rotation += delta_time * self.settings.game_speed;

        for &entity in &self.game_entities {
            if let Some(transform) = world.core.get_local_transform_mut(entity) {
                transform.rotation =
                    nalgebra_glm::quat_angle_axis(self.game_rotation, &Vec3::new(0.0, 1.0, 0.0))
                        * nalgebra_glm::quat_angle_axis(
                            self.game_rotation * 0.7,
                            &Vec3::new(1.0, 0.0, 0.0),
                        );
            }
            world.core.set_local_transform_dirty(entity, LocalTransformDirty);
        }
    }

    fn run_pause_menu(&mut self, world: &mut World) {
        if self.confirm_dialog.is_some() {
            self.run_confirm_dialog(world);
            return;
        }

        let mouse_pos = nalgebra_glm::vec2(
            world.resources.input.mouse.position.x,
            world.resources.input.mouse.position.y,
        );
        let clicked = world
            .resources
            .input
            .mouse
            .state
            .contains(MouseState::LEFT_JUST_RELEASED);
        let mouse_down = world
            .resources
            .input
            .mouse
            .state
            .contains(MouseState::LEFT_CLICKED);

        let mut buttons = std::mem::take(&mut self.pause_buttons);
        let clicked_index =
            self.process_buttons(world, &mut buttons, mouse_pos, mouse_down, clicked);
        self.pause_buttons = buttons;

        if let Some(index) = clicked_index {
            match index {
                0 => self.start_transition(GameState::Playing),
                1 => {
                    self.settings_source = SettingsSource::Pause;
                    self.start_transition(GameState::Settings);
                }
                2 => {
                    self.start_dialog_transition(
                        "RETURN TO MENU",
                        "Are you sure? Progress will be lost.",
                        DialogAction::MainMenu,
                    );
                }
                _ => {}
            }
        }
    }
}

fn create_slider_bar(percentage: i32) -> String {
    let filled = (percentage as f32 / 10.0).round() as usize;
    let empty = 10 - filled;
    format!(
        "[{}{}] {}%",
        "|".repeat(filled),
        "-".repeat(empty),
        percentage
    )
}

impl State for MenuDemoState {
    fn title(&self) -> &str {
        "Menu Demo - Nightshade"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = false;
        world.resources.graphics.show_grid = false;
        world.resources.graphics.atmosphere = Atmosphere::None;

        let camera = world.spawn_entities(
            NAME | LOCAL_TRANSFORM
                | LOCAL_TRANSFORM_DIRTY
                | GLOBAL_TRANSFORM
                | CAMERA
                | PAN_ORBIT_CAMERA,
            1,
        )[0];

        if let Some(name) = world.core.get_name_mut(camera) {
            *name = Name("Main Camera".to_string());
        }

        world.core.set_local_transform(
            camera,
            LocalTransform {
                translation: Vec3::new(0.0, 2.0, 8.0),
                rotation: Quat::identity(),
                scale: Vec3::new(1.0, 1.0, 1.0),
            },
        );
        world.core.set_local_transform_dirty(camera, LocalTransformDirty);
        world.core.set_global_transform(camera, GlobalTransform::default());

        world.core.set_camera(
            camera,
            Camera {
                projection: Projection::Perspective(PerspectiveCamera {
                    aspect_ratio: None,
                    y_fov_rad: 45.0_f32.to_radians(),
                    z_far: None,
                    z_near: 0.01,
                }),
                smoothing: None,
            },
        );

        world.core.set_pan_orbit_camera(
            camera,
            PanOrbitCamera {
                focus: Vec3::new(0.0, 0.5, 0.0),
                target_focus: Vec3::new(0.0, 0.5, 0.0),
                radius: 10.0,
                target_radius: 10.0,
                pitch: 0.25,
                target_pitch: 0.25,
                yaw: 0.0,
                target_yaw: 0.0,
                enabled: false,
                ..Default::default()
            },
        );

        self.camera_entity = Some(camera);
        world.resources.active_camera = Some(camera);

        self.global_alpha = 1.0;
        self.update_screen_size(world);
        self.setup_main_menu(world);
    }

    fn run_systems(&mut self, world: &mut World) {
        self.update_screen_size(world);
        pan_orbit_camera_system(world);

        let delta_time = world.resources.window.timing.delta_time;
        self.update_transitions(world, delta_time);

        if !matches!(self.transition, TransitionState::None) {
            return;
        }

        match self.game_state {
            GameState::MainMenu => self.run_main_menu(world),
            GameState::Settings => self.run_settings_menu(world),
            GameState::GraphicsSettings => self.run_graphics_settings(world),
            GameState::AudioSettings => self.run_audio_settings(world),
            GameState::Playing => self.run_playing_state(world),
            GameState::Paused => self.run_pause_menu(world),
        }
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, state: KeyState) {
        if state == KeyState::Pressed && matches!(self.transition, TransitionState::None) {
            match key {
                KeyCode::KeyP => match self.game_state {
                    GameState::Playing => self.start_transition(GameState::Paused),
                    GameState::Paused => self.start_transition(GameState::Playing),
                    _ => {}
                },
                KeyCode::Escape => {
                    if self.confirm_dialog.is_some() {
                        self.despawn_confirm_dialog(world, true);
                    } else {
                        match self.game_state {
                            GameState::Settings => match self.settings_source {
                                SettingsSource::MainMenu => {
                                    self.start_transition(GameState::MainMenu)
                                }
                                SettingsSource::Pause => self.start_transition(GameState::Paused),
                            },
                            GameState::GraphicsSettings | GameState::AudioSettings => {
                                self.start_transition(GameState::Settings);
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
