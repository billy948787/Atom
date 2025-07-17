use egui_winit_vulkano::{self, egui};

use crate::reader::{FileType, obj_reader};
#[derive(Debug)]
pub struct Editor {
    pub scene: crate::graphics::scene::Scene,
    selected_file_type: FileType,
}

impl Default for Editor {
    fn default() -> Self {
        Editor {
            scene: obj_reader::read_file("test_model/Forklift/Forklift.obj").unwrap(),
            selected_file_type: FileType::default(),
        }
    }
}

impl Editor {
    pub fn ui(&mut self, ctx: &egui::Context) {
        // load file window
        egui::Window::new("Editor").show(ctx, |ui| {
            let selected_text = self.selected_file_type.to_string();
            egui::ComboBox::new("file_type", "File Type")
                .selected_text(selected_text.clone())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.selected_file_type, FileType::Obj, "OBJ");
                    ui.selectable_value(&mut self.selected_file_type, FileType::Fbx, "FBX");
                });
            if ui.button("Load Scene").clicked() {
                // Logic to load a scene

                let path = rfd::FileDialog::new()
                    .add_filter(selected_text, &[self.selected_file_type.to_string()])
                    .pick_file();

                if let Some(path) = path {
                    let scene = obj_reader::read_file(path.to_str().unwrap()).unwrap();
                    self.scene.merge(scene);
                    println!("Scene loaded from: {:?}", path);
                }
            }
        });

        egui::Window::new("object properties").show(ctx, |ui| {
            for (i, mut object) in self.scene.objects.iter_mut().enumerate() {
                ui.label(format!("Object {}", i));

                let (mut scale, mut rotation, mut translation) =
                    object.world_transform.to_scale_rotation_translation();

                // translation
                ui.horizontal(|ui| {
                    ui.label("Position:");
                    ui.add(egui::DragValue::new(&mut translation.x).speed(0.1));
                    ui.add(egui::DragValue::new(&mut translation.y).speed(0.1));
                    ui.add(egui::DragValue::new(&mut translation.z).speed(0.1));
                });

                let mut euler_degrees = rotation.to_euler(glam::EulerRot::XYZ);
                euler_degrees.0 = euler_degrees.0.to_degrees();
                euler_degrees.1 = euler_degrees.1.to_degrees();
                euler_degrees.2 = euler_degrees.2.to_degrees();

                // rotation
                ui.horizontal(|ui| {
                    ui.label("Rotation:");
                    ui.add(egui::DragValue::new(&mut euler_degrees.0).speed(0.1));
                    ui.add(egui::DragValue::new(&mut euler_degrees.1).speed(0.1));
                    ui.add(egui::DragValue::new(&mut euler_degrees.2).speed(0.1));
                });
                rotation = glam::Quat::from_euler(
                    glam::EulerRot::XYZ,
                    euler_degrees.0.to_radians(),
                    euler_degrees.1.to_radians(),
                    euler_degrees.2.to_radians(),
                );

                // scale
                ui.horizontal(|ui| {
                    ui.label("Scale:");
                    ui.add(egui::DragValue::new(&mut scale.x).speed(0.1));
                    ui.add(egui::DragValue::new(&mut scale.y).speed(0.1));
                    ui.add(egui::DragValue::new(&mut scale.z).speed(0.1));
                });

                object.world_transform =
                    glam::Mat4::from_scale_rotation_translation(scale, rotation, translation);
            }
        });
    }
}
