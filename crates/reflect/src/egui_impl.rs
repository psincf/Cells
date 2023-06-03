use crate::*;

use std::any::TypeId;
use std::collections::HashMap;

use egui::Widget;

pub struct DefaultValueSetting {
    general: Option<Value>,
    special: HashMap<String, Value>,
}

impl DefaultValueSetting {
    fn get_default(&self, field_name: Option<&String>) -> Option<Value> {
        if let Some(field_name) = field_name {
            if let Some(value) = self.special.get(field_name) {
                return Some(value.clone());
            }
        }
        return self.general.clone();
    }
}


#[derive(Default)]
pub struct InspectSettings {
    default_struct: HashMap<TypeId, StructValue>,
    default_enum: HashMap<TypeId, EnumValue>,
    default_value: HashMap<Type, DefaultValueSetting>,
}

impl InspectSettings {
    pub fn add_default_enum<T: Reflect + 'static>(&mut self, value: T) {
        let value = value.to_value();
        if let Value::Enum(enum_value) = value {
            self.default_enum.insert(TypeId::of::<T>(), *enum_value);
        } else {
            panic!()
        }
    }

    pub fn add_default_struct<T: Reflect + 'static>(&mut self, value: T) {
        let value = value.to_value();
        if let Value::Struct(struct_value) = value {
            self.default_struct.insert(TypeId::of::<T>(), struct_value);
        } else {
            panic!()
        }
    }

    pub fn add_default_value<T: Reflect + 'static>(&mut self, value: T) {
        let value = value.to_value();
        let _type = T::to_type();
        if let Some(value_settings) = self.default_value.get_mut(&_type) {
            value_settings.general = Some(value);
        } else {
            self.default_value.insert(_type, DefaultValueSetting {
                general: Some(value),
                special: HashMap::new(),
            });
        }
    }

    pub fn add_default_value_special<T: Reflect + 'static>(&mut self, value: T, field_name: &'static str) {
        let value = value.to_value();
        let _type = T::to_type();
        if let Some(value_settings) = self.default_value.get_mut(&_type) {
            value_settings.special.insert(field_name.to_string(), value);
        } else {
            let mut hashmap = HashMap::new(); hashmap.insert(field_name.to_string(), value);
            self.default_value.insert(_type, DefaultValueSetting {
                general: None,
                special: hashmap,
            });
        }
    }

    fn try_get_default_from(&self, _type: Type) -> Option<Value> {
        match _type {
            Type::Enum(enum_type) => {
                return self.default_enum.get(&enum_type.id).map(|v| Value::Enum(Box::new(v.clone())));
            }
            Type::Struct(struct_type) => {
                return self.default_struct.get(&struct_type.id).map(|v| Value::Struct(v.clone()));
            }
            _ => {
                return self.default_value.get(&_type).map(|s| s.get_default(None)).flatten();
            }
            //_ => { return None }
        }
    }
}

pub fn inspect(ui: &mut egui::Ui, data: &mut Value, name_field: Option<String>, settings: Option<InspectSettings>) {
    let mut state = InspectState::default();
    if let Some(s) = settings { state.settings = s; }
    
    //ui.style_mut().animation_time = 0.1;
    ui.set_min_height(500.0);
    ui.style_mut().body_text_style = egui::TextStyle::Monospace;
    //default_fonts.font_data.insert("Ubuntu-Light".to_owned(), default_fonts.font_data.get("ProggyClean").unwrap().clone());
    ui.ctx().clear_animations();
    let frame = egui::Frame::default();
    //frame.fill = egui::Color32::from_rgb(16, 16, 16);
    frame.show(ui, |ui| {
        ui.style_mut().spacing.item_spacing.y = 1.0;
        state.inspect_intern(ui, data, name_field);
    });
}

#[derive(Default)]
struct InspectState {
    field_count: usize,
    labels: HashMap<String, usize>,
    name_parent: Option<String>,
    settings: InspectSettings,
}

impl InspectState {
    fn get_nocollidable_label(&mut self, label_old: String) -> String {
        if let Some(state) = self.labels.get_mut(&label_old.clone()) {
            *state += 1;
            return label_old + "#" + &state.to_string();
        } else {
            self.labels.insert(label_old.clone(), 0);
            return label_old;
        }
    }
}

impl InspectState {
    fn inspect_intern(&mut self, ui: &mut egui::Ui , data: &mut Value, name_field: Option<String>) {
        if let Value::Unit = data { return }
        //ui.separator();

        //ui.style_mut().spacing.item_spacing.y = 0.0;
        self.field_count += 1;
        let color = if self.field_count % 2 == 0 { egui::Color32::from_rgb(128, 128, 128) } else { egui::Color32::from_rgb(64, 64, 64) };
        let _frame = egui::Frame::default().fill(color);
        
        ui.horizontal( |ui| {
            let space = ui.available_width();
            if name_field.is_some() {
                let mut name_field_new = name_field.as_ref().unwrap().clone();
                if name_field_new.chars().count() > 11 {
                    name_field_new.truncate(11);
                    name_field_new.push_str("...");
                }
                if ui.label(name_field_new.clone() + ": ").hovered() {
                    egui::show_tooltip_text(ui.ctx(), egui::Id::new(1), name_field.clone().unwrap());
                }
                ui.allocate_space( egui::Vec2::new((100.0 - (space - ui.available_width())).max(-1_000_000.0), 0.0));
            }
            match data {
                Value::Bool(v) => {
                    ui.checkbox(v, "");
                    //ui.checkbox(v, &name_field_label);
                }
                Value::U8(v) => {
                    ui.add(egui::widgets::DragValue::u8(v));
                }
                Value::U64(v) => {
                    let mut v_i32 = *v as i32;
                    ui.add(egui::widgets::DragValue::i32(&mut v_i32));
                    *v = v_i32 as u64;
                }
                Value::I32(v) => {
                    let mut v_i32 = *v as i32;
                    ui.add(egui::widgets::DragValue::i32(&mut v_i32));
                    *v = v_i32;
                }
                Value::I64(v) => {
                    let mut v_f32 = *v as f32;
                    ui.add(egui::widgets::DragValue::f32(&mut v_f32));
                    *v = v_f32 as i64;
                }
                Value::F32(v) => {
                    let mut v_f32 = *v as f32;
                    ui.add(egui::widgets::DragValue::f32(&mut v_f32));
                    *v = v_f32;
                }
                Value::String(string) => {
                    ui.add(egui::widgets::TextEdit::singleline(string));
                }
                Value::Enum(v) => {
                    self.inspect_enum(ui, v, name_field);
                }
                Value::Option(v) => {
                    self.inspect_option(ui, v, name_field);
                }
                Value::Seq(v) => {
                    self.inspect_seq(ui, v, name_field);
                }
                Value::Tuple(v) => {
                    self.inspect_tuple(ui, v, name_field);
                }
                Value::Struct(struct_value) => {
                    self.inspect_struct(ui, struct_value.info.name, &mut struct_value.fields, name_field);
                }
                _ => {}
            }
        });
    }

    fn inspect_enum(&mut self, ui: &mut egui::Ui, enum_value: &mut Box<EnumValue>, _name_field: Option<String>) {
        let width_available = ui.available_width();

        egui::containers::combo_box(ui, egui::Id::new(enum_value.info.enum_name.to_string()), enum_value.variant.info.variant_name.to_string(), | ui | {
            for variant in enum_value.info.variants.iter() {
                let selected = variant.variant_name == enum_value.variant.info.variant_name;
                //if ui.selectable_label(selected, variant.variant_name.to_string()).clicked {
                if egui::widgets::SelectableLabel::new(selected, variant.variant_name.to_string()).ui(ui).clicked() {
                    enum_value.variant.info = variant.clone();
                    enum_value.variant.value = self.settings.try_get_default_from(variant.variant_type.get()).or(variant.variant_type.get().default_value()).unwrap();
                }
            }
        });

        if enum_value.variant.value != Value::Unit {
            ui.advance_cursor(20.0 - (width_available - ui.available_width()));
            ui.with_layout(egui::Layout::default(), |ui| {
                ui.advance_cursor(24.0);
                ui.indent(egui::Id::new(0), |ui| {
                    self.inspect_intern(ui, &mut enum_value.variant.value, None);
                });
            });
        }
    }

    fn inspect_option(&mut self, ui: &mut egui::Ui, option: &mut Box<OptionValue>, _name_field: Option<String>) {
        self.name_parent = None;

        let width_available = ui.available_width();

        egui::containers::combo_box(ui, egui::Id::new(self.get_nocollidable_label("Option".to_string())), self.get_nocollidable_label(option.value.as_ref().map_or("None", |_o| "Some" ).to_string()), |ui| {
            let is_some = option.value.is_some();
            if egui::widgets::SelectableLabel::new(!is_some, self.get_nocollidable_label("None".to_string())).ui(ui).clicked() {
                option.value = None;
            }
            if egui::widgets::SelectableLabel::new(is_some, self.get_nocollidable_label("Some".to_string())).ui(ui).clicked() {
                option.value = self.settings.try_get_default_from(option.info.some.get()).or(option.info.some.get().default_value());
            }
        });

        if let Some(value) = option.value.as_mut() {
            ui.advance_cursor(20.0 - (width_available - ui.available_width()));
            ui.with_layout(egui::Layout::default(), |ui| {
                ui.advance_cursor(24.0);
                ui.indent(egui::Id::new(0), |ui| {
                    self.inspect_intern(ui, value, None)
                });
            });
        }
    }

    fn inspect_struct(&mut self, ui: &mut egui::Ui, name: &'static str, fields: &mut Vec<StructFieldValue>, _name_field: Option<String>) {
        self.name_parent = Some(name.to_string());
        egui::containers::CollapsingHeader::new(self.get_nocollidable_label(name.to_string())).show(ui, |ui| {
            for field in fields {
                self.inspect_intern(ui, &mut field.value, Some(field.info.name.to_owned()))
            }
        });
    }

    fn inspect_tuple(&mut self, ui: &mut egui::Ui, fields: &mut Vec<Value>, name_field: Option<String>) {
        //if name_field.is_some() { ui.text(name_field.as_ref().unwrap()); ui.indent() };
        self.name_parent = None;

        ui.with_layout(egui::Layout::default(), |ui| {
            for field in fields.iter_mut() {
                match field {
                    _ => {
                        self.inspect_intern(ui, field, name_field.clone())
                    }
                }
            }
        });
    }

    fn inspect_seq(&mut self, ui: &mut egui::Ui, seq_value: &mut SeqValue, name_field: Option<String>) {
        self.name_parent = None;

        if seq_value.info.fixed_size == Some(4) && seq_value.info.seq_type.get() == Type::U8 {
            let mut color = [0.0, 0.0, 0.0, 0.0];
            for (index, value) in seq_value.values.iter().enumerate() {
                if let Value::U8(a) = value {
                    color[index] = *a as f32 / 256.0;
                }
            }
            ui.color_edit_button_rgba_unmultiplied(&mut color);
            for (index, value) in seq_value.values.iter_mut().enumerate() {
                if let Value::U8(a) = value {
                    *a = (color[index] * 256.0) as u8;
                }
            }
        } else {
            egui::containers::CollapsingHeader::new(self.get_nocollidable_label(name_field.clone().unwrap_or("List".to_string()))).show(ui, |ui| {
                for field in seq_value.values.iter_mut() {
                    match field {
                        _ => {
                            self.inspect_intern(ui, field, name_field.clone())
                        }
                    }
                }
            });
        }
    }
}