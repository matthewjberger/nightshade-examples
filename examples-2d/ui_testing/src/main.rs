use nightshade::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(UiTestApp {
        driver: UiTestDriver::new("widget_interactions"),
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
        world.resources.retained_ui.enabled = true;
        world.resources.retained_ui.background_color =
            Some(nalgebra_glm::Vec4::new(0.05, 0.05, 0.08, 1.0));

        let camera = spawn_ortho_camera(world, nalgebra_glm::Vec2::new(0.0, 0.0));
        world.resources.active_camera = Some(camera);

        let mut tree = UiTreeBuilder::new(world);

        let root = tree
            .add_node()
            .boundary(
                Ab(nalgebra_glm::Vec2::new(20.0, 20.0)),
                Ab(nalgebra_glm::Vec2::new(-20.0, -20.0))
                    + Rl(nalgebra_glm::Vec2::new(100.0, 100.0)),
            )
            .flow(FlowDirection::Vertical, 8.0, 8.0)
            .entity();

        tree.build_ui(root, |ui| {
            ui.heading("Automated UI Test");
            ui.separator();

            let status_label = ui.label("Status: Ready");
            ui.set_test_id(status_label, "status_label");

            ui.spacing(4.0);

            let btn = ui.button("Click Me");
            ui.set_test_id(btn, "test_button");
            ui.react_clicked(btn, move |world: &mut World| {
                world.ui_set_text(status_label, "Status: Button Clicked!");
            });

            ui.spacing(4.0);
            ui.label("Volume:");
            let slider = ui.slider("volume", 0.0, 100.0, 50.0);
            ui.set_test_id(slider, "volume_slider");

            ui.spacing(4.0);
            ui.label("Mute:");
            let toggle = ui.toggle("muted", false);
            ui.set_test_id(toggle, "mute_toggle");
        });

        tree.finish();

        self.driver
            .at_frame(3)
            .assert_visible("test_button")
            .assert_visible("volume_slider")
            .assert_visible("mute_toggle")
            .assert_visible("status_label")
            .assert_text("status_label", "Status: Ready")
            .assert_value_f32("volume", 50.0)
            .assert_value_bool("muted", false)
            .log("Initial state verified");

        self.driver.at_frame(5).click_entity("test_button");

        self.driver
            .at_frame(7)
            .assert_text("status_label", "Status: Button Clicked!")
            .assert_event_fired_button_clicked()
            .log("Button click verified");

        self.driver.at_frame(10).click_entity("mute_toggle");

        self.driver
            .at_frame(12)
            .assert_event_fired_toggle_changed()
            .assert_value_bool("muted", true)
            .log("Toggle interaction verified");

        self.driver.start();
    }

    fn run_systems(&mut self, world: &mut World) {
        self.driver.step(world);
    }
}
