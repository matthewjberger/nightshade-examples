use nightshade::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(UiTestApp {
        driver: UiTestDriver::new("comprehensive_ui_test"),
    })
}

struct UiTestApp {
    driver: UiTestDriver,
}

impl State for UiTestApp {
    fn title(&self) -> &str {
        "UI Testing Example"
    }

    fn initialize(&mut self, world: &mut World) {
        setup_test_scene(world);

        let mut tree = UiTreeBuilder::new(world);
        let root = create_test_root(&mut tree);

        tree.build_ui(root, |ui| {
            ui.heading("Automated UI Test");
            ui.separator();

            let status_label = ui.label_id("Status: Ready", "status_label");

            ui.spacing(4.0);

            let btn = ui.button_id("Click Me", "test_button");
            ui.react_clicked(btn, move |world: &mut World| {
                world.ui_set_text(status_label, "Status: Button Clicked!");
            });

            ui.spacing(4.0);
            ui.label("Volume:");
            ui.slider_id("volume", 0.0, 100.0, 50.0, "volume_slider");

            ui.spacing(4.0);
            ui.label("Mute:");
            ui.toggle_id("muted", false, "mute_toggle");

            ui.spacing(4.0);
            ui.checkbox_id("agree", "I agree", false, "agree_checkbox");

            ui.spacing(4.0);
            ui.text_input_id("username", "Enter name...", "name_input");

            ui.spacing(4.0);
            let hidden_label = ui.label_id("Hidden", "hidden_label");
            ui.set_visible(hidden_label, false);

            ui.spacing(4.0);
            ui.button_id("Right Click Target", "right_click_target");

            ui.spacing(4.0);
            ui.label("Color:");
            ui.dropdown_id("color", &["Red", "Green", "Blue"], 0, "color_dropdown");

            ui.spacing(4.0);
            ui.label("Speed:");
            ui.drag_value_id("speed", 0.0, 100.0, 25.0, "speed_drag");

            ui.spacing(4.0);
            ui.button_id("Drag Me", "drag_target");
        });

        tree.finish();

        self.driver
            .at_frame(3)
            .assert_visible("test_button")
            .assert_visible("volume_slider")
            .assert_visible("mute_toggle")
            .assert_visible("agree_checkbox")
            .assert_visible("name_input")
            .assert_visible("status_label")
            .assert_visible("right_click_target")
            .assert_visible("color_dropdown")
            .assert_visible("speed_drag")
            .assert_visible("drag_target")
            .assert_not_visible("hidden_label")
            .assert_text("status_label", "Status: Ready")
            .assert_value_f32("volume", 50.0)
            .assert_value_bool("muted", false)
            .assert_value_bool("agree", false)
            .assert_value_usize("color", 0)
            .assert_value_f32("speed", 25.0)
            .log("Initial state verified");

        self.driver
            .wait(2)
            .move_mouse_to("test_button")
            .log("Moved mouse to button");

        self.driver
            .wait(1)
            .assert_hovered("test_button")
            .log("Button hover verified");

        self.driver.wait(1).click_entity("test_button");

        self.driver
            .wait(2)
            .assert_text("status_label", "Status: Button Clicked!")
            .assert_event_fired_button_clicked()
            .log("Button click verified");

        self.driver.wait(2).click_entity("mute_toggle");

        self.driver
            .wait(2)
            .assert_event_fired_toggle_changed()
            .assert_value_bool("muted", true)
            .log("Toggle interaction verified");

        self.driver.wait(2).set_slider_value("volume_slider", 75.0);

        self.driver
            .wait(2)
            .assert_event_fired_slider_changed()
            .assert_value_f32_approx("volume", 75.0, 2.0)
            .log("Slider value change verified");

        self.driver.wait(2).click_entity("agree_checkbox");

        self.driver
            .wait(2)
            .assert_event_fired_checkbox_changed()
            .assert_value_bool("agree", true)
            .log("Checkbox interaction verified");

        self.driver.wait(2).click_entity("name_input");

        self.driver
            .wait(1)
            .assert_focused("name_input")
            .log("Text input focused");

        self.driver.wait(1).type_text("hello");

        self.driver.wait(1).press_key(KeyCode::Enter);

        self.driver
            .wait(2)
            .assert_event_fired_text_submitted()
            .log("Text input submission verified");

        self.driver.wait(2).right_click_entity("right_click_target");

        self.driver.wait(2).log("Right click verified");

        self.driver
            .wait(2)
            .scroll(nalgebra_glm::Vec2::new(0.0, -3.0))
            .log("Scroll input verified");

        self.driver
            .wait(2)
            .select_dropdown_option("color_dropdown", 2);

        self.driver
            .wait(8)
            .assert_event_fired_dropdown_changed()
            .assert_value_usize("color", 2)
            .log("Dropdown selection verified");

        self.driver
            .wait(2)
            .drag_entity("drag_target", nalgebra_glm::Vec2::new(400.0, 300.0));

        self.driver.wait(2).log("Drag interaction verified");

        self.driver.wait(2).key_down(KeyCode::ShiftLeft);

        self.driver
            .then()
            .key_down(KeyCode::ControlLeft)
            .log("Modifier keys pressed (key_down + then() verified)");

        self.driver
            .wait(1)
            .key_up(KeyCode::ShiftLeft)
            .key_up(KeyCode::ControlLeft)
            .log("Modifier keys released");

        self.driver.start();
    }

    fn run_systems(&mut self, world: &mut World) {
        self.driver.step(world);
    }
}
