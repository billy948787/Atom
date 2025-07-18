use egui_winit_vulkano::{self, egui};

use crate::{
    graphics::light::Light,
    reader::{FileType, obj_reader},
};

pub mod command;
#[derive(Debug)]
pub struct Editor {
    pub scene: crate::graphics::scene::Scene,
    selected_file_type: FileType,
}

impl Default for Editor {
    fn default() -> Self {
        let scene = crate::graphics::scene::Scene::default();
        

        Editor {
            scene,
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
            for (i, object) in self.scene.objects.iter_mut().enumerate() {
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

        // camera properties
        egui::Window::new("Camera Properties").show(ctx, |ui| {
            let camera = &mut self.scene.cameras[self.scene.main_camera_index];
            ui.label("Camera Properties");
            ui.horizontal(|ui| {
                ui.label("Position:");
                ui.add(egui::DragValue::new(&mut camera.position.x).speed(0.1));
                ui.add(egui::DragValue::new(&mut camera.position.y).speed(0.1));
                ui.add(egui::DragValue::new(&mut camera.position.z).speed(0.1));
            });
            ui.horizontal(|ui| {
                ui.label("Target:");
                ui.add(egui::DragValue::new(&mut camera.target.x).speed(0.1));
                ui.add(egui::DragValue::new(&mut camera.target.y).speed(0.1));
                ui.add(egui::DragValue::new(&mut camera.target.z).speed(0.1));
            });
            ui.horizontal(|ui| {
                ui.label("Up Vector:");
                ui.add(egui::DragValue::new(&mut camera.up.x).speed(0.1));
                ui.add(egui::DragValue::new(&mut camera.up.y).speed(0.1));
                ui.add(egui::DragValue::new(&mut camera.up.z).speed(0.1));
            });
            ui.horizontal(|ui| {
                ui.label("FOV (degrees):");
                ui.add(egui::DragValue::new(&mut camera.fov).speed(0.1));
            });
            ui.horizontal(|ui| {
                ui.label("Near Plane:");
                ui.add(egui::DragValue::new(&mut camera.near_plane).speed(0.1));
            });
            ui.horizontal(|ui| {
                ui.label("Far Plane:");
                ui.add(egui::DragValue::new(&mut camera.far_plane).speed(0.1));
            });
        });

        //light controls
        egui::Window::new("Light Controls").show(ctx, |ui| {
            for (i, light) in self.scene.lights.iter_mut().enumerate() {
                match light {
                    Light::Directional(directional) => {
                        ui.label(format!("Directional Light {}", i));
                        ui.horizontal(|ui| {
                            ui.label("Direction:");
                            ui.add(egui::DragValue::new(&mut directional.direction.x).speed(0.1));
                            ui.add(egui::DragValue::new(&mut directional.direction.y).speed(0.1));
                            ui.add(egui::DragValue::new(&mut directional.direction.z).speed(0.1));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Color:");
                            ui.color_edit_button_rgb(&mut directional.color.into());
                        });
                    }
                    Light::Point(point) => {
                        ui.label(format!("Point Light {}", i));
                        ui.horizontal(|ui| {
                            ui.label("Position:");
                            ui.add(egui::DragValue::new(&mut point.position.x).speed(0.1));
                            ui.add(egui::DragValue::new(&mut point.position.y).speed(0.1));
                            ui.add(egui::DragValue::new(&mut point.position.z).speed(0.1));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Color:");
                            let mut color_arr = point.color.to_array();
                            ui.color_edit_button_rgb(&mut color_arr);
                            point.color = glam::Vec3::from_array(color_arr);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Intensity:");
                            ui.add(egui::DragValue::new(&mut point.intensity).speed(0.1));
                        });
                    }
                }
            }
        });
    }
}
