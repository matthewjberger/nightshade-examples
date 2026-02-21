use nightshade::prelude::egui;
use std::fmt::Write as FmtWrite;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SdfShape {
    Sphere,
    Box,
    RoundBox,
    BoxFrame,
    Torus,
    CappedTorus,
    Link,
    Cone,
    HexPrism,
    TriPrism,
    Capsule,
    CappedCylinder,
    RoundedCylinder,
    CappedCone,
    SolidAngle,
    CutSphere,
    CutHollowSphere,
    DeathStar,
    RoundCone,
    Ellipsoid,
    Rhombus,
    Octahedron,
    Pyramid,
    VerticalCapsule,
}

impl SdfShape {
    pub const ALL: &[Self] = &[
        Self::Sphere,
        Self::Box,
        Self::RoundBox,
        Self::BoxFrame,
        Self::Torus,
        Self::CappedTorus,
        Self::Link,
        Self::Cone,
        Self::HexPrism,
        Self::TriPrism,
        Self::Capsule,
        Self::CappedCylinder,
        Self::RoundedCylinder,
        Self::CappedCone,
        Self::SolidAngle,
        Self::CutSphere,
        Self::CutHollowSphere,
        Self::DeathStar,
        Self::RoundCone,
        Self::Ellipsoid,
        Self::Rhombus,
        Self::Octahedron,
        Self::Pyramid,
        Self::VerticalCapsule,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Self::Sphere => "Sphere",
            Self::Box => "Box",
            Self::RoundBox => "Round Box",
            Self::BoxFrame => "Box Frame",
            Self::Torus => "Torus",
            Self::CappedTorus => "Capped Torus",
            Self::Link => "Link",
            Self::Cone => "Cone",
            Self::HexPrism => "Hex Prism",
            Self::TriPrism => "Tri Prism",
            Self::Capsule => "Capsule",
            Self::CappedCylinder => "Capped Cylinder",
            Self::RoundedCylinder => "Rounded Cylinder",
            Self::CappedCone => "Capped Cone",
            Self::SolidAngle => "Solid Angle",
            Self::CutSphere => "Cut Sphere",
            Self::CutHollowSphere => "Cut Hollow Sphere",
            Self::DeathStar => "Death Star",
            Self::RoundCone => "Round Cone",
            Self::Ellipsoid => "Ellipsoid",
            Self::Rhombus => "Rhombus",
            Self::Octahedron => "Octahedron",
            Self::Pyramid => "Pyramid",
            Self::VerticalCapsule => "Vertical Capsule",
        }
    }

    pub fn param_names(&self) -> &'static [&'static str] {
        match self {
            Self::Sphere => &["Radius"],
            Self::Box => &["Width", "Height", "Depth"],
            Self::RoundBox => &["Width", "Height", "Depth", "Rounding"],
            Self::BoxFrame => &["Width", "Height", "Depth", "Edge"],
            Self::Torus => &["Major R", "Minor R"],
            Self::CappedTorus => &["Arc (rad)", "Major R", "Minor R"],
            Self::Link => &["Length", "R1", "R2"],
            Self::Cone => &["Angle (rad)", "Height"],
            Self::HexPrism => &["Radius", "Depth"],
            Self::TriPrism => &["Radius", "Depth"],
            Self::Capsule => &["Length", "Radius"],
            Self::CappedCylinder => &["Height", "Radius"],
            Self::RoundedCylinder => &["Radius", "Rounding", "Height"],
            Self::CappedCone => &["Height", "R Bottom", "R Top"],
            Self::SolidAngle => &["Angle (rad)", "Radius"],
            Self::CutSphere => &["Radius", "Cut Height"],
            Self::CutHollowSphere => &["Radius", "Cut Height", "Thickness"],
            Self::DeathStar => &["R Outer", "R Inner", "Distance"],
            Self::RoundCone => &["R Bottom", "R Top", "Height"],
            Self::Ellipsoid => &["Rx", "Ry", "Rz"],
            Self::Rhombus => &["La", "Lb", "Height", "Rounding"],
            Self::Octahedron => &["Size"],
            Self::Pyramid => &["Height"],
            Self::VerticalCapsule => &["Height", "Radius"],
        }
    }

    pub fn default_params(&self) -> [f32; 6] {
        match self {
            Self::Sphere => [0.8, 0.0, 0.0, 0.0, 0.0, 0.0],
            Self::Box => [0.6, 0.6, 0.6, 0.0, 0.0, 0.0],
            Self::RoundBox => [0.5, 0.5, 0.5, 0.1, 0.0, 0.0],
            Self::BoxFrame => [0.6, 0.6, 0.6, 0.08, 0.0, 0.0],
            Self::Torus => [0.5, 0.15, 0.0, 0.0, 0.0, 0.0],
            Self::CappedTorus => [2.0, 0.5, 0.12, 0.0, 0.0, 0.0],
            Self::Link => [0.2, 0.4, 0.12, 0.0, 0.0, 0.0],
            Self::Cone => [0.5, 0.8, 0.0, 0.0, 0.0, 0.0],
            Self::HexPrism => [0.5, 0.3, 0.0, 0.0, 0.0, 0.0],
            Self::TriPrism => [0.7, 0.3, 0.0, 0.0, 0.0, 0.0],
            Self::Capsule => [0.8, 0.2, 0.0, 0.0, 0.0, 0.0],
            Self::CappedCylinder => [0.5, 0.3, 0.0, 0.0, 0.0, 0.0],
            Self::RoundedCylinder => [0.3, 0.08, 0.4, 0.0, 0.0, 0.0],
            Self::CappedCone => [0.7, 0.5, 0.15, 0.0, 0.0, 0.0],
            Self::SolidAngle => [0.8, 0.8, 0.0, 0.0, 0.0, 0.0],
            Self::CutSphere => [0.8, 0.2, 0.0, 0.0, 0.0, 0.0],
            Self::CutHollowSphere => [0.8, 0.3, 0.05, 0.0, 0.0, 0.0],
            Self::DeathStar => [0.8, 0.6, 0.5, 0.0, 0.0, 0.0],
            Self::RoundCone => [0.4, 0.15, 0.8, 0.0, 0.0, 0.0],
            Self::Ellipsoid => [0.6, 0.4, 0.3, 0.0, 0.0, 0.0],
            Self::Rhombus => [0.6, 0.3, 0.08, 0.04, 0.0, 0.0],
            Self::Octahedron => [0.7, 0.0, 0.0, 0.0, 0.0, 0.0],
            Self::Pyramid => [1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            Self::VerticalCapsule => [0.8, 0.2, 0.0, 0.0, 0.0, 0.0],
        }
    }

    pub fn generate_call(&self, pos_var: &str, params: &[f32; 6]) -> String {
        match self {
            Self::Sphere => format!("sd_sphere({pos_var}, {:.4})", params[0]),
            Self::Box => format!(
                "sd_box({pos_var}, vec3<f32>({:.4}, {:.4}, {:.4}))",
                params[0], params[1], params[2]
            ),
            Self::RoundBox => format!(
                "sd_round_box({pos_var}, vec3<f32>({:.4}, {:.4}, {:.4}), {:.4})",
                params[0], params[1], params[2], params[3]
            ),
            Self::BoxFrame => format!(
                "sd_box_frame({pos_var}, vec3<f32>({:.4}, {:.4}, {:.4}), {:.4})",
                params[0], params[1], params[2], params[3]
            ),
            Self::Torus => format!(
                "sd_torus({pos_var}, vec2<f32>({:.4}, {:.4}))",
                params[0], params[1]
            ),
            Self::CappedTorus => format!(
                "sd_capped_torus({pos_var}, vec2<f32>(sin({:.4}), cos({:.4})), {:.4}, {:.4})",
                params[0], params[0], params[1], params[2]
            ),
            Self::Link => format!(
                "sd_link({pos_var}, {:.4}, {:.4}, {:.4})",
                params[0], params[1], params[2]
            ),
            Self::Cone => format!(
                "sd_cone({pos_var}, vec2<f32>(sin({:.4}), cos({:.4})), {:.4})",
                params[0], params[0], params[1]
            ),
            Self::HexPrism => format!(
                "sd_hex_prism({pos_var}, vec2<f32>({:.4}, {:.4}))",
                params[0], params[1]
            ),
            Self::TriPrism => format!(
                "sd_tri_prism({pos_var}, vec2<f32>({:.4}, {:.4}))",
                params[0], params[1]
            ),
            Self::Capsule => {
                let half_len = params[0] / 2.0;
                format!(
                    "sd_capsule({pos_var}, vec3<f32>(0.0, {:.4}, 0.0), vec3<f32>(0.0, {:.4}, 0.0), {:.4})",
                    -half_len, half_len, params[1]
                )
            }
            Self::CappedCylinder => format!(
                "sd_capped_cylinder({pos_var}, {:.4}, {:.4})",
                params[0], params[1]
            ),
            Self::RoundedCylinder => format!(
                "sd_rounded_cylinder({pos_var}, {:.4}, {:.4}, {:.4})",
                params[0], params[1], params[2]
            ),
            Self::CappedCone => format!(
                "sd_capped_cone({pos_var}, {:.4}, {:.4}, {:.4})",
                params[0], params[1], params[2]
            ),
            Self::SolidAngle => format!(
                "sd_solid_angle({pos_var}, vec2<f32>(sin({:.4}), cos({:.4})), {:.4})",
                params[0], params[0], params[1]
            ),
            Self::CutSphere => format!(
                "sd_cut_sphere({pos_var}, {:.4}, {:.4})",
                params[0], params[1]
            ),
            Self::CutHollowSphere => format!(
                "sd_cut_hollow_sphere({pos_var}, {:.4}, {:.4}, {:.4})",
                params[0], params[1], params[2]
            ),
            Self::DeathStar => format!(
                "sd_death_star({pos_var}, {:.4}, {:.4}, {:.4})",
                params[0], params[1], params[2]
            ),
            Self::RoundCone => format!(
                "sd_round_cone({pos_var}, {:.4}, {:.4}, {:.4})",
                params[0], params[1], params[2]
            ),
            Self::Ellipsoid => format!(
                "sd_ellipsoid({pos_var}, vec3<f32>({:.4}, {:.4}, {:.4}))",
                params[0], params[1], params[2]
            ),
            Self::Rhombus => format!(
                "sd_rhombus({pos_var}, {:.4}, {:.4}, {:.4}, {:.4})",
                params[0], params[1], params[2], params[3]
            ),
            Self::Octahedron => format!("sd_octahedron({pos_var}, {:.4})", params[0]),
            Self::Pyramid => format!("sd_pyramid({pos_var}, {:.4})", params[0]),
            Self::VerticalCapsule => format!(
                "sd_vertical_capsule({pos_var}, {:.4}, {:.4})",
                params[0], params[1]
            ),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SdfCombine {
    Replace,
    Union,
    Subtraction,
    Intersection,
    Xor,
    SmoothUnion,
    SmoothSubtraction,
    SmoothIntersection,
}

impl SdfCombine {
    pub const ALL: &[Self] = &[
        Self::Replace,
        Self::Union,
        Self::Subtraction,
        Self::Intersection,
        Self::Xor,
        Self::SmoothUnion,
        Self::SmoothSubtraction,
        Self::SmoothIntersection,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Self::Replace => "Replace",
            Self::Union => "Union",
            Self::Subtraction => "Subtraction",
            Self::Intersection => "Intersection",
            Self::Xor => "XOR",
            Self::SmoothUnion => "Smooth Union",
            Self::SmoothSubtraction => "Smooth Sub",
            Self::SmoothIntersection => "Smooth Intersect",
        }
    }

    pub fn has_parameter(&self) -> bool {
        matches!(
            self,
            Self::SmoothUnion | Self::SmoothSubtraction | Self::SmoothIntersection
        )
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SdfModifierType {
    Twist,
    Bend,
    Round,
    Onion,
    Elongate,
    InfiniteRep,
    FiniteRep,
    SymmetryX,
    SymmetryXZ,
}

impl SdfModifierType {
    pub const ALL: &[Self] = &[
        Self::Twist,
        Self::Bend,
        Self::Round,
        Self::Onion,
        Self::Elongate,
        Self::InfiniteRep,
        Self::FiniteRep,
        Self::SymmetryX,
        Self::SymmetryXZ,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Self::Twist => "Twist",
            Self::Bend => "Bend",
            Self::Round => "Round",
            Self::Onion => "Onion",
            Self::Elongate => "Elongate",
            Self::InfiniteRep => "Infinite Rep",
            Self::FiniteRep => "Finite Rep",
            Self::SymmetryX => "Symmetry X",
            Self::SymmetryXZ => "Symmetry XZ",
        }
    }

    pub fn param_names(&self) -> &'static [&'static str] {
        match self {
            Self::Twist => &["Strength"],
            Self::Bend => &["Strength"],
            Self::Round => &["Radius"],
            Self::Onion => &["Thickness"],
            Self::Elongate => &["X", "Y", "Z"],
            Self::InfiniteRep => &["X Spacing", "Y Spacing", "Z Spacing"],
            Self::FiniteRep => &["Spacing", "X Limit", "Y Limit", "Z Limit"],
            Self::SymmetryX | Self::SymmetryXZ => &[],
        }
    }

    pub fn default_params(&self) -> [f32; 4] {
        match self {
            Self::Twist => [2.0, 0.0, 0.0, 0.0],
            Self::Bend => [1.0, 0.0, 0.0, 0.0],
            Self::Round => [0.05, 0.0, 0.0, 0.0],
            Self::Onion => [0.1, 0.0, 0.0, 0.0],
            Self::Elongate => [0.2, 0.0, 0.2, 0.0],
            Self::InfiniteRep => [3.0, 3.0, 3.0, 0.0],
            Self::FiniteRep => [2.0, 2.0, 0.0, 2.0],
            Self::SymmetryX | Self::SymmetryXZ => [0.0; 4],
        }
    }

    pub fn is_pre_modifier(&self) -> bool {
        matches!(
            self,
            Self::Twist
                | Self::Bend
                | Self::Elongate
                | Self::InfiniteRep
                | Self::FiniteRep
                | Self::SymmetryX
                | Self::SymmetryXZ
        )
    }
}

#[derive(Clone)]
pub struct SdfModifier {
    pub modifier_type: SdfModifierType,
    pub params: [f32; 4],
}

#[derive(Clone)]
pub struct SdfEntry {
    pub shape: SdfShape,
    pub combine: SdfCombine,
    pub combine_k: f32,
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub scale: f32,
    pub params: [f32; 6],
    pub modifiers: Vec<SdfModifier>,
    pub material_id: u32,
    pub enabled: bool,
    pub expanded: bool,
}

impl SdfEntry {
    pub fn new(shape: SdfShape) -> Self {
        Self {
            shape,
            combine: SdfCombine::Union,
            combine_k: 0.3,
            position: [0.0; 3],
            rotation: [0.0; 3],
            scale: 1.0,
            params: shape.default_params(),
            modifiers: Vec::new(),
            material_id: 1,
            enabled: true,
            expanded: true,
        }
    }
}

pub struct SdfEditor {
    pub entries: Vec<SdfEntry>,
    pub show_ground: bool,
    pub ground_height: f32,
    pub auto_rotate: bool,
    pub dirty: bool,
    generated_scene: String,
    generated_full: String,
}

impl Default for SdfEditor {
    fn default() -> Self {
        let mut editor = Self {
            entries: Vec::new(),
            show_ground: true,
            ground_height: 1.5,
            auto_rotate: true,
            dirty: true,
            generated_scene: String::new(),
            generated_full: String::new(),
        };

        let mut sphere = SdfEntry::new(SdfShape::Sphere);
        sphere.combine = SdfCombine::Replace;
        sphere.material_id = 1;
        editor.entries.push(sphere);

        let mut box_entry = SdfEntry::new(SdfShape::RoundBox);
        box_entry.position = [1.8, 0.0, 0.0];
        box_entry.combine = SdfCombine::SmoothUnion;
        box_entry.combine_k = 0.3;
        box_entry.material_id = 2;
        editor.entries.push(box_entry);

        let mut torus = SdfEntry::new(SdfShape::Torus);
        torus.position = [-1.8, 0.0, 0.0];
        torus.combine = SdfCombine::Union;
        torus.material_id = 3;
        editor.entries.push(torus);

        editor
    }
}

impl SdfEditor {
    pub fn generate_scene_function(&mut self) -> &str {
        if !self.dirty {
            return &self.generated_scene;
        }

        let mut code = String::with_capacity(4096);
        let _ = writeln!(code, "fn sdf_scene(p: vec3<f32>) -> vec2<f32> {{");

        if self.auto_rotate {
            let _ = writeln!(code, "    let time = uniforms.time;");
        }

        let _ = writeln!(code, "    var d = 1e10;");
        let _ = writeln!(code, "    var mat_id = 0.0;");
        let _ = writeln!(code);

        let mut first_enabled = true;
        for (index, entry) in self.entries.iter().enumerate() {
            if !entry.enabled {
                continue;
            }

            let _ = writeln!(code, "    {{");
            let _ = writeln!(
                code,
                "        var q = p - vec3<f32>({:.4}, {:.4}, {:.4});",
                entry.position[0], entry.position[1], entry.position[2]
            );

            if entry.rotation[0].abs() > 0.001 {
                let _ = writeln!(code, "        q = rot_x(q, {:.4});", entry.rotation[0]);
            }
            if entry.rotation[1].abs() > 0.001 {
                let _ = writeln!(code, "        q = rot_y(q, {:.4});", entry.rotation[1]);
            }
            if entry.rotation[2].abs() > 0.001 {
                let _ = writeln!(code, "        q = rot_z(q, {:.4});", entry.rotation[2]);
            }

            if self.auto_rotate {
                let _ = writeln!(
                    code,
                    "        q = rot_y(q, time * {:.4});",
                    0.3 + index as f32 * 0.1
                );
            }

            if (entry.scale - 1.0).abs() > 0.001 {
                let _ = writeln!(code, "        q = q / {:.4};", entry.scale);
            }

            for modifier in &entry.modifiers {
                if modifier.modifier_type.is_pre_modifier() {
                    let modifier_code =
                        generate_pre_modifier("q", &modifier.modifier_type, &modifier.params);
                    if !modifier_code.is_empty() {
                        let _ = writeln!(code, "        {modifier_code}");
                    }
                }
            }

            let shape_call = entry.shape.generate_call("q", &entry.params);
            let _ = writeln!(code, "        var shape_d = {shape_call};");

            if (entry.scale - 1.0).abs() > 0.001 {
                let _ = writeln!(code, "        shape_d = shape_d * {:.4};", entry.scale);
            }

            for modifier in &entry.modifiers {
                if !modifier.modifier_type.is_pre_modifier() {
                    let modifier_code = generate_post_modifier(
                        "shape_d",
                        &modifier.modifier_type,
                        &modifier.params,
                    );
                    if !modifier_code.is_empty() {
                        let _ = writeln!(code, "        {modifier_code}");
                    }
                }
            }

            if first_enabled {
                let _ = writeln!(code, "        d = shape_d;");
                let _ = writeln!(code, "        mat_id = {:.1};", entry.material_id as f32);
                first_enabled = false;
            } else {
                let combine_code = generate_combine(&entry.combine, entry.combine_k);
                let _ = writeln!(code, "        let prev_d = d;");
                let _ = writeln!(code, "        {combine_code}");
                let _ = writeln!(
                    code,
                    "        if d != prev_d {{ mat_id = {:.1}; }}",
                    entry.material_id as f32
                );
            }

            let _ = writeln!(code, "    }}");
            let _ = writeln!(code);
        }

        if self.show_ground {
            let _ = writeln!(code, "    let ground = p.y + {:.4};", self.ground_height);
            let _ = writeln!(code, "    if ground < d {{ d = ground; mat_id = 0.0; }}");
            let _ = writeln!(code);
        }

        let _ = writeln!(code, "    return vec2<f32>(d, mat_id);");
        let _ = writeln!(code, "}}");

        self.generated_scene = code;
        self.dirty = false;
        &self.generated_scene
    }

    pub fn generate_full_shader(&mut self) -> &str {
        let scene = self.generate_scene_function().to_string();
        self.generated_full = format!(
            "{}\n{}\n{}",
            crate::presets::SDF_EDITOR_HEADER,
            scene,
            crate::presets::SDF_EDITOR_FOOTER,
        );
        &self.generated_full
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;

        ui.horizontal(|ui| {
            if ui.button("+ Add Shape").clicked() {
                let mut entry = SdfEntry::new(SdfShape::Sphere);
                entry.material_id = (self.entries.len() as u32 % 8) + 1;
                if !self.entries.is_empty() {
                    entry.position[0] = self.entries.len() as f32 * 1.5;
                }
                self.entries.push(entry);
                self.dirty = true;
                changed = true;
            }
        });

        ui.horizontal(|ui| {
            if ui.checkbox(&mut self.show_ground, "Ground").changed() {
                self.dirty = true;
                changed = true;
            }
            if ui.checkbox(&mut self.auto_rotate, "Auto-Rotate").changed() {
                self.dirty = true;
                changed = true;
            }
        });

        if self.show_ground {
            ui.horizontal(|ui| {
                ui.label("Ground H:");
                if ui
                    .add(
                        egui::DragValue::new(&mut self.ground_height)
                            .speed(0.05)
                            .range(0.0..=10.0),
                    )
                    .changed()
                {
                    self.dirty = true;
                    changed = true;
                }
            });
        }

        ui.separator();

        let mut remove_index = None;
        let mut swap_pair = None;
        let mut duplicate_index = None;

        egui::ScrollArea::vertical()
            .id_salt("sdf_editor_scroll")
            .show(ui, |ui| {
                let entry_count = self.entries.len();
                for index in 0..entry_count {
                    let entry_id = egui::Id::new(("sdf_entry", index));
                    let header_text = format!(
                        "{}. {} ({})",
                        index + 1,
                        self.entries[index].shape.name(),
                        self.entries[index].combine.name()
                    );

                    let mut is_expanded = self.entries[index].expanded;
                    let header_response =
                        egui::CollapsingHeader::new(egui::RichText::new(&header_text).strong())
                            .id_salt(entry_id)
                            .default_open(is_expanded)
                            .show(ui, |ui| {
                                let entry = &mut self.entries[index];

                                ui.horizontal(|ui| {
                                    if ui.checkbox(&mut entry.enabled, "Enabled").changed() {
                                        self.dirty = true;
                                        changed = true;
                                    }
                                });

                                ui.horizontal(|ui| {
                                    ui.label("Shape:");
                                    egui::ComboBox::from_id_salt(("shape_combo", index))
                                        .selected_text(entry.shape.name())
                                        .show_ui(ui, |ui| {
                                            for shape in SdfShape::ALL {
                                                if ui
                                                    .selectable_value(
                                                        &mut entry.shape,
                                                        *shape,
                                                        shape.name(),
                                                    )
                                                    .changed()
                                                {
                                                    entry.params = shape.default_params();
                                                    self.dirty = true;
                                                    changed = true;
                                                }
                                            }
                                        });
                                });

                                if index > 0 {
                                    ui.horizontal(|ui| {
                                        ui.label("Combine:");
                                        egui::ComboBox::from_id_salt(("combine_combo", index))
                                            .selected_text(entry.combine.name())
                                            .show_ui(ui, |ui| {
                                                for combine in SdfCombine::ALL {
                                                    if ui
                                                        .selectable_value(
                                                            &mut entry.combine,
                                                            *combine,
                                                            combine.name(),
                                                        )
                                                        .changed()
                                                    {
                                                        self.dirty = true;
                                                        changed = true;
                                                    }
                                                }
                                            });
                                    });

                                    if entry.combine.has_parameter() {
                                        ui.horizontal(|ui| {
                                            ui.label("  K:");
                                            if ui
                                                .add(
                                                    egui::DragValue::new(&mut entry.combine_k)
                                                        .speed(0.01)
                                                        .range(0.01..=2.0),
                                                )
                                                .changed()
                                            {
                                                self.dirty = true;
                                                changed = true;
                                            }
                                        });
                                    }
                                }

                                ui.horizontal(|ui| {
                                    ui.label("Pos:");
                                    for axis in 0..3 {
                                        if ui
                                            .add(
                                                egui::DragValue::new(&mut entry.position[axis])
                                                    .speed(0.05)
                                                    .range(-50.0..=50.0),
                                            )
                                            .changed()
                                        {
                                            self.dirty = true;
                                            changed = true;
                                        }
                                    }
                                });

                                ui.horizontal(|ui| {
                                    ui.label("Rot:");
                                    for axis in 0..3 {
                                        if ui
                                            .add(
                                                egui::DragValue::new(&mut entry.rotation[axis])
                                                    .speed(0.02)
                                                    .range(-6.3..=6.3),
                                            )
                                            .changed()
                                        {
                                            self.dirty = true;
                                            changed = true;
                                        }
                                    }
                                });

                                ui.horizontal(|ui| {
                                    ui.label("Scale:");
                                    if ui
                                        .add(
                                            egui::DragValue::new(&mut entry.scale)
                                                .speed(0.02)
                                                .range(0.01..=10.0),
                                        )
                                        .changed()
                                    {
                                        self.dirty = true;
                                        changed = true;
                                    }
                                    ui.label("Mat:");
                                    let mut mat = entry.material_id as i32;
                                    if ui
                                        .add(egui::DragValue::new(&mut mat).range(0..=8))
                                        .changed()
                                    {
                                        entry.material_id = mat as u32;
                                        self.dirty = true;
                                        changed = true;
                                    }
                                });

                                let param_names = entry.shape.param_names();
                                for (param_index, param_name) in param_names.iter().enumerate() {
                                    ui.horizontal(|ui| {
                                        ui.label(format!("  {param_name}:"));
                                        if ui
                                            .add(
                                                egui::DragValue::new(
                                                    &mut entry.params[param_index],
                                                )
                                                .speed(0.01)
                                                .range(-10.0..=10.0),
                                            )
                                            .changed()
                                        {
                                            self.dirty = true;
                                            changed = true;
                                        }
                                    });
                                }

                                ui.label("Modifiers:");
                                let mut mod_remove = None;
                                for (mod_index, modifier) in entry.modifiers.iter_mut().enumerate()
                                {
                                    ui.horizontal(|ui| {
                                        ui.label(format!("  {}", modifier.modifier_type.name()));
                                        let mod_param_names = modifier.modifier_type.param_names();
                                        for (pi, pname) in mod_param_names.iter().enumerate() {
                                            ui.label(*pname);
                                            if ui
                                                .add(
                                                    egui::DragValue::new(&mut modifier.params[pi])
                                                        .speed(0.02)
                                                        .range(-20.0..=20.0),
                                                )
                                                .changed()
                                            {
                                                self.dirty = true;
                                                changed = true;
                                            }
                                        }
                                        if ui.small_button("x").clicked() {
                                            mod_remove = Some(mod_index);
                                            self.dirty = true;
                                            changed = true;
                                        }
                                    });
                                }
                                if let Some(mi) = mod_remove {
                                    entry.modifiers.remove(mi);
                                }

                                ui.horizontal(|ui| {
                                    ui.label("  +Mod:");
                                    egui::ComboBox::from_id_salt(("add_mod", index))
                                        .selected_text("Add...")
                                        .width(90.0)
                                        .show_ui(ui, |ui| {
                                            for mod_type in SdfModifierType::ALL {
                                                if ui
                                                    .selectable_label(false, mod_type.name())
                                                    .clicked()
                                                {
                                                    entry.modifiers.push(SdfModifier {
                                                        modifier_type: *mod_type,
                                                        params: mod_type.default_params(),
                                                    });
                                                    self.dirty = true;
                                                    changed = true;
                                                }
                                            }
                                        });
                                });

                                ui.horizontal(|ui| {
                                    if index > 0 && ui.small_button("Up").clicked() {
                                        swap_pair = Some((index - 1, index));
                                    }
                                    if index < entry_count - 1 && ui.small_button("Down").clicked()
                                    {
                                        swap_pair = Some((index, index + 1));
                                    }
                                    if ui.small_button("Remove").clicked() {
                                        remove_index = Some(index);
                                    }
                                    if ui.small_button("Duplicate").clicked() {
                                        duplicate_index = Some(index);
                                    }
                                });
                            });

                    is_expanded = header_response.body_returned.is_some();
                    self.entries[index].expanded = is_expanded;
                }
            });

        if let Some(index) = remove_index {
            self.entries.remove(index);
            self.dirty = true;
            changed = true;
        }

        if let Some((a, b)) = swap_pair {
            self.entries.swap(a, b);
            self.dirty = true;
            changed = true;
        }

        if let Some(index) = duplicate_index {
            let clone = self.entries[index].clone();
            self.entries.insert(index + 1, clone);
            self.dirty = true;
            changed = true;
        }

        changed
    }
}

fn generate_pre_modifier(var: &str, modifier_type: &SdfModifierType, params: &[f32; 4]) -> String {
    match modifier_type {
        SdfModifierType::Twist => format!("{var} = op_twist({var}, {:.4});", params[0]),
        SdfModifierType::Bend => format!("{var} = op_cheap_bend({var}, {:.4});", params[0]),
        SdfModifierType::SymmetryX => format!("{var} = op_sym_x({var});"),
        SdfModifierType::SymmetryXZ => format!("{var} = op_sym_xz({var});"),
        SdfModifierType::Elongate => format!(
            "{var} = op_elongate({var}, vec3<f32>({:.4}, {:.4}, {:.4}));",
            params[0], params[1], params[2]
        ),
        SdfModifierType::InfiniteRep => format!(
            "{var} = op_rep({var}, vec3<f32>({:.4}, {:.4}, {:.4}));",
            params[0], params[1], params[2]
        ),
        SdfModifierType::FiniteRep => format!(
            "{var} = op_rep_lim({var}, {:.4}, vec3<f32>({:.4}, {:.4}, {:.4}));",
            params[0], params[1], params[2], params[3]
        ),
        _ => String::new(),
    }
}

fn generate_post_modifier(var: &str, modifier_type: &SdfModifierType, params: &[f32; 4]) -> String {
    match modifier_type {
        SdfModifierType::Round => format!("{var} = op_round({var}, {:.4});", params[0]),
        SdfModifierType::Onion => format!("{var} = op_onion({var}, {:.4});", params[0]),
        _ => String::new(),
    }
}

fn generate_combine(combine: &SdfCombine, k: f32) -> String {
    match combine {
        SdfCombine::Replace => "d = shape_d;".to_string(),
        SdfCombine::Union => "d = op_union(d, shape_d);".to_string(),
        SdfCombine::Subtraction => "d = op_subtraction(shape_d, d);".to_string(),
        SdfCombine::Intersection => "d = op_intersection(d, shape_d);".to_string(),
        SdfCombine::Xor => "d = op_xor(d, shape_d);".to_string(),
        SdfCombine::SmoothUnion => format!("d = op_smooth_union(d, shape_d, {k:.4});"),
        SdfCombine::SmoothSubtraction => {
            format!("d = op_smooth_subtraction(shape_d, d, {k:.4});")
        }
        SdfCombine::SmoothIntersection => {
            format!("d = op_smooth_intersection(d, shape_d, {k:.4});")
        }
    }
}
