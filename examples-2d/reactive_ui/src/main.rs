use nightshade::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(ReactiveUiDemo::default())
}

#[derive(Default)]
struct ReactiveUiDemo {
    fps_label: Entity,
}

impl State for ReactiveUiDemo {
    fn title(&self) -> &str {
        "Reactive UI"
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
            ui.heading("Reactive UI Demo");
            ui.separator();

            self.fps_label = ui.label("FPS: --");

            ui.spacing(4.0);
            ui.label("Brightness:");
            ui.slider("brightness", 0.0, 100.0, 50.0);
            ui.react("brightness", |val: f32, world: &mut World| {
                let brightness = val / 100.0;
                let value = 0.05 + brightness * 0.1;
                world.resources.retained_ui.background_color =
                    Some(nalgebra_glm::Vec4::new(value, value, value + 0.03, 1.0));
            });

            ui.spacing(4.0);
            ui.label("Reduced Motion:");
            ui.toggle("reduced_motion", false);
            ui.react("reduced_motion", |val: bool, world: &mut World| {
                world.ui_set_reduced_motion(val);
            });

            ui.spacing(4.0);
            let toast_btn = ui.button("Show Toast");
            ui.react_clicked(toast_btn, |world: &mut World| {
                world.ui_show_toast("Hello from reactive UI!", ToastSeverity::Info, 3.0);
            });

            ui.spacing(4.0);
            ui.label("Submit (press Enter):");
            let input = ui.text_input("submit_input", "Type a message...");
            let submit_label = ui.label("No submission yet");
            ui.react_submitted(input, move |text: String, world: &mut World| {
                world.ui_set_text(submit_label, &format!("Submitted: \"{text}\""));
            });

            ui.spacing(4.0);
            ui.label("Radio group:");
            ui.radio("Low", 0, 0);
            ui.radio("Medium", 0, 1);
            ui.radio("High", 0, 2);
            ui.radio_group("quality", 0);
            let quality_label = ui.label("Quality: Low");
            ui.react("quality", move |val: usize, world: &mut World| {
                let name = ["Low", "Medium", "High"][val.min(2)];
                world.ui_set_text(quality_label, &format!("Quality: {name}"));
            });

            ui.spacing(8.0);
            ui.separator();
            ui.label_colored(
                "All state above is managed via named properties and reactions.",
                nalgebra_glm::Vec4::new(0.5, 0.5, 0.6, 1.0),
            );
        });

        tree.finish();
    }

    fn run_systems(&mut self, world: &mut World) {
        let fps = world.resources.window.timing.frames_per_second;
        world.ui_set_text(self.fps_label, &format!("FPS: {fps:.0}"));
    }
}
