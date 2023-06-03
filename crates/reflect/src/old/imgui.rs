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


pub fn inspect(ui: &imgui::Ui, data: &mut ReflectType, name_field: Option<String>, settings: Option<InspectSettings>) {
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
    fn inspect_intern(&mut self, ui: &imgui::Ui , data: &mut ReflectType, name_field: Option<String>) {
        ui.set_next_item_width(200.0);
        ui.align_text_to_frame_padding();
        let name_field_label = imgui::ImString::from(self.get_nocollidable_label(name_field.clone().unwrap_or(String::from("unknown"))));
        //if name_field.is_some() { ui.text(&name_field_label); ui.same_line_with_spacing(0.0, 0.0); ui.text(": "); ui.same_line_with_spacing(0.0, 0.0);}
        match data {
            ReflectType::bool(v) => {
                ui.checkbox(&name_field_label, v);
            }
            ReflectType::u8(v) => {
                let mut v_i32 = *v as i32;
                ui.input_int(&name_field_label, &mut v_i32).enter_returns_true(true).build();
                *v = v_i32 as u8;
            }
            ReflectType::u64(v) => {
                let mut v_i32 = *v as i32;
                ui.input_int(&name_field_label, &mut v_i32).enter_returns_true(true).build();
                *v = v_i32 as u64;
            }
            ReflectType::i32(v) => {
                let mut v_i32 = *v as i32;
                ui.input_int(&name_field_label, &mut v_i32).enter_returns_true(true).build();
                *v = v_i32;
            }
            ReflectType::i64(v) => {
                let mut v_f32 = *v as f32;
                ui.input_float(&name_field_label, &mut v_f32).enter_returns_true(true).build();
                *v = v_f32 as i64;
            }
            ReflectType::f32(v) => {
                let mut v_f32 = *v as f32;
                ui.input_float(&name_field_label, &mut v_f32).enter_returns_true(true).build();
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
    }

    fn inspect_enum(&mut self, ui: &imgui::Ui, enum_variant: &mut Box<EnumVariant>, name_field: Option<String>) {
        if name_field.is_some() { ui.text(name_field.as_ref().unwrap()); ui.same_line_with_spacing(0.0, 10.0); };

        if let Some(enum_settings) = self.settings.special_enum.get(&enum_variant.enum_name) {
            //imgui::ComboBox::new(&imgui::ImString::from(enum_variant.enum_name.clone())).preview_value(&imgui::ImString::from(enum_variant.enum_name.clone() + "::" + &enum_variant.variant_name.clone())).build(ui, || {
            imgui::ComboBox::new(&imgui::ImString::from(enum_variant.enum_name.clone())).preview_value(&imgui::ImString::from(enum_variant.variant_name.clone())).build(ui, || {
                for variant in enum_settings {
                    //if imgui::Selectable::new(&imgui::ImString::from(enum_variant.enum_name.clone() + "::" + &variant.variant_name.clone())).build(ui) {
                    if imgui::Selectable::new(&imgui::ImString::from(variant.variant_name.clone())).build(ui) {
                        if variant.variant_name != enum_variant.variant_name {
                            *enum_variant = variant.clone();
                        }
                    }
                }
            });
        } else {
            /*
            ui.text(enum_variant.variant_name.clone());
            if name_field.is_some() { ui.same_line_with_spacing(0.0, 10.0); ui.text(name_field.as_ref().unwrap()); };
            */
            
            imgui::ComboBox::new(&imgui::ImString::from(enum_variant.enum_name.clone())).preview_value(&imgui::ImString::from(enum_variant.variant_name.clone())).build(ui, || {
                if imgui::Selectable::new(&imgui::ImString::from(enum_variant.variant_name.clone())).build(ui) {

                }
            });
        }

        ui.indent();
        self.inspect_intern(ui, &mut enum_variant.value, None);
        ui.unindent();
    }

    fn inspect_option(&mut self, ui: &imgui::Ui, option: &mut Option<Box<ReflectType>>, name_field: Option<String>) {
        if name_field.is_some() { ui.text(name_field.as_ref().unwrap()) };
        self.name_parent = None;

        ui.indent();

        if let Some(field) = option {
            ui.same_line_with_spacing(0.0, 10.0);
            ui.text("Some: ");
            self.inspect_intern(ui, field, None)
        } else {
            ui.same_line_with_spacing(0.0, 10.0);
            ui.text("None");
        }

        ui.unindent();
    }

    fn inspect_struct(&mut self, ui: &imgui::Ui, name: &mut String, fields: &mut Vec<StructField>, name_field: Option<String>) {
        if name_field.is_some() { ui.text(name_field.as_ref().unwrap()); ui.same_line_with_spacing(0.0, 10.0); };
        self.name_parent = Some(name.clone());
        imgui::TreeNode::new(&imgui::ImString::from(self.get_nocollidable_label(name.clone()))).build(ui, || {
        //if imgui::CollapsingHeader::new(&imgui::ImString::from(self.get_nocollidable_label(name.clone()))).build(ui) {
            ui.indent();

            //ui.text(name);
            for field in fields {
                self.inspect_intern(ui, &mut field.value, Some(field.name.to_owned()))
            }
            ui.unindent();
        });
    }

    fn inspect_tuple(&mut self, ui: &imgui::Ui, fields: &mut Vec<ReflectType>, name_field: Option<String>) {
        //if name_field.is_some() { ui.text(name_field.as_ref().unwrap()); ui.indent() };
        self.name_parent = None;

        for field in fields.iter_mut() {
            match field {
                _ => {
                    self.inspect_intern(ui, field, name_field.clone())
                }
            }
        }

        //if name_field.is_some() { ui.unindent() };
    }

    fn inspect_seq(&mut self, ui: &imgui::Ui, fields: &mut Vec<ReflectType>, name_field: Option<String>) {
        if name_field.is_some() { ui.text(name_field.as_ref().unwrap()); ui.same_line_with_spacing(0.0, 10.0); };
        self.name_parent = None;

        imgui::TreeNode::new(&imgui::ImString::from(self.get_nocollidable_label(name_field.clone().unwrap_or("List".to_string())))).build(ui, || {
            if name_field.is_some() { ui.same_line_with_spacing(0.0, 10.0); ui.text(name_field.as_ref().unwrap()); };
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

        if name_field.is_some() { ui.unindent() };
    }
}