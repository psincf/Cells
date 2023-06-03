use crate::ReflectType;
use crate::EnumVariant;
use crate::StructField;
use crate::ReflectedStructSerializer;

use serde::Serialize;

use std::collections::HashMap;

#[derive(Default)]
pub struct InspectSettings {
    special_struct: HashMap<String, Vec<StructField>>,
    special_enum: HashMap<String, Vec<Box<EnumVariant>>>,
    special_tuple: HashMap<(String, String), Vec<ReflectType>>,
    special_option: HashMap<(String, String), ReflectType>,
}

impl InspectSettings {
    pub fn add_struct<T: Serialize>(&mut self, _struct: T) {
        let reflect_type = ReflectedStructSerializer::serialize(&_struct);
        if let ReflectType::Struct(name, fields) = reflect_type {
            assert!( self.special_struct.insert(name, fields).is_none() );
        } else {
            panic!()
        }
    }
    
    pub fn add_enum<T: Serialize>(&mut self, _enum_variant: T) {
        let reflect_type = ReflectedStructSerializer::serialize(&_enum_variant);
        if let ReflectType::Enum(variant) = reflect_type {
            let name = variant.enum_name.clone();
            let mut enum_special = self.special_enum.get_mut(&name);
            if enum_special.is_none() { self.special_enum.insert(name.clone(), Vec::new()); enum_special = self.special_enum.get_mut(&name); }
            enum_special.unwrap().push(variant);
        } else {
            panic!()
        }
    }
    
    pub fn add_tuple<T: Serialize>(&mut self, name_parent: String, name_field: String, _tuple: T) {
        let reflect_type = ReflectedStructSerializer::serialize(&_tuple);
        if let ReflectType::Tuple(types) = reflect_type {
            
        } else {
            panic!()
        }
    }
    
    pub fn add_option(&mut self, name_parent: String, name_field: String, reflect_type: ReflectType) {
        if let ReflectType::Option(types) = reflect_type {

        } else {
            panic!()
        }
    }
}


pub fn inspect(ui: &mut egui::Ui, data: &mut ReflectType, name_field: Option<String>, settings: Option<InspectSettings>) {
    let mut state = InspectState::default();
    if let Some(s) = settings { state.settings = s; }
    state.inspect_intern(ui, data, name_field);
}

#[derive(Default)]
struct InspectState {
    labels: HashMap<String, usize>,
    name_parent: Option<String>,
    settings: InspectSettings,
}

impl InspectState {
    fn get_nocollidable_label(&mut self, label_old: String) -> String {
        if let Some(state) = self.labels.get_mut(&label_old.clone()) {
            *state += 1;
            return label_old + &state.to_string();
        } else {
            self.labels.insert(label_old.clone(), 0);
            return label_old;
        }
    }
}

impl InspectState {
    fn inspect_intern(&mut self, ui: &mut egui::Ui, data: &mut ReflectType, name_field: Option<String>) {
        let name_field_label = self.get_nocollidable_label(name_field.clone().unwrap_or(String::from("unknown")));
        //if name_field.is_some() { ui.text(&name_field_label); ui.same_line_with_spacing(0.0, 0.0); ui.text(": "); ui.same_line_with_spacing(0.0, 0.0);}
        match data {
            ReflectType::bool(v) => {
                ui.checkbox(v, &name_field_label);
            }
            ReflectType::u8(v) => {
                let mut v_i32 = *v as i32;
                ui.add(egui::widgets::DragValue::i32(&mut v_i32));
                *v = v_i32 as u8;
            }
            ReflectType::u64(v) => {
                let mut v_i32 = *v as i32;
                ui.add(egui::widgets::DragValue::i32(&mut v_i32));
                *v = v_i32 as u64;
            }
            ReflectType::i32(v) => {
                let mut v_i32 = *v as i32;
                ui.add(egui::widgets::DragValue::i32(&mut v_i32));
                *v = v_i32;
            }
            ReflectType::i64(v) => {
                let mut v_f32 = *v as f32;
                ui.add(egui::widgets::DragValue::f32(&mut v_f32));
                *v = v_f32 as i64;
            }
            ReflectType::f32(v) => {
                let mut v_f32 = *v as f32;
                ui.add(egui::widgets::DragValue::f32(&mut v_f32));
                *v = v_f32;
            }
            ReflectType::Enum(v) => {
                self.inspect_enum(ui, v, name_field);
            }
            ReflectType::Option(v) => {
                self.inspect_option(ui, v, name_field);
            }
            ReflectType::Seq(v) => {
                self.inspect_seq(ui, v, name_field);
            }
            ReflectType::Tuple(v) => {
                self.inspect_tuple(ui, v, name_field);
            }
            ReflectType::Struct(name, fields) => {
                self.inspect_struct(ui, name, fields, name_field);
            }
            _ => {}
        }
        ui.separator();
    }

    fn inspect_enum(&mut self, ui: &mut egui::Ui, enum_variant: &mut Box<EnumVariant>, name_field: Option<String>) {
        if name_field.is_some() { ui.label(name_field.as_ref().unwrap()); };

        if let Some(enum_settings) = self.settings.special_enum.get(&enum_variant.enum_name) {
            egui::combo_box(ui, egui::Id::new(enum_variant.enum_name.clone()), enum_variant.variant_name.clone(), |ui| {
                for variant in enum_settings {
                    let selected = variant.variant_name == enum_variant.variant_name;
                    if ui.selectable_label(selected, variant.variant_name.clone()).clicked {
                        if variant.variant_name != enum_variant.variant_name {
                            *enum_variant = variant.clone();
                        }
                    }
                }
            });
        } else {
            ui.label(enum_variant.variant_name.clone());
        }

        ui.indent("aaa", |ui| {
            self.inspect_intern(ui, &mut enum_variant.value, None);
        });
    }

    fn inspect_option(&mut self, ui: &mut egui::Ui, option: &mut Option<Box<ReflectType>>, name_field: Option<String>) {
        if name_field.is_some() { ui.label(name_field.as_ref().unwrap()); }
        self.name_parent = None;

        ui.indent("bbb", |ui| {
            if let Some(field) = option {
                ui.label("Some: ");
                self.inspect_intern(ui, field, None)
            } else {
                ui.label("None");
            }
        });
    }

    fn inspect_struct(&mut self, ui: &mut egui::Ui, name: &mut String, fields: &mut Vec<StructField>, name_field: Option<String>) {
        if name_field.is_some() { ui.label(name_field.as_ref().unwrap()); }
        self.name_parent = Some(name.clone());
        
        ui.collapsing(self.get_nocollidable_label(name.clone()), |ui| {
            for field in fields {
                self.inspect_intern(ui, &mut field.value, Some(field.name.to_owned()))
            }
        });
    }

    fn inspect_tuple(&mut self, ui: &mut egui::Ui, fields: &mut Vec<ReflectType>, name_field: Option<String>) {
        if name_field.is_some() { ui.label(name_field.as_ref().unwrap()); }
        self.name_parent = None;

        for field in fields.iter_mut() {
            match field {
                _ => {
                    self.inspect_intern(ui, field, name_field.clone())
                }
            }
        }
    }

    fn inspect_seq(&mut self, ui: &mut egui::Ui, fields: &mut Vec<ReflectType>, name_field: Option<String>) {
        if name_field.is_some() { ui.label(name_field.as_ref().unwrap()); }
        self.name_parent = None;

        ui.collapsing(self.get_nocollidable_label(name_field.clone().unwrap_or("List".to_string())), |ui| {
            //ui.indent();
            for field in fields.iter_mut() {
                match field {
                    _ => {
                        self.inspect_intern(ui, field, name_field.clone())
                    }
                }
            }
            //ui.unindent();
        });
    }
}