use crate::*;

use std::any::TypeId;
use std::collections::HashMap;

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
    behaviour_enum: HashMap<TypeId, Box<dyn Fn(&imgui::Ui, &mut EnumValue)>>,
    behaviour_struct: HashMap<TypeId, Box<dyn Fn(&imgui::Ui, &mut StructValue)>>,
    behaviour_field_enum: HashMap<String, Box<dyn Fn(&imgui::Ui, &mut EnumValue)>>,
    behaviour_field_struct: HashMap<String, Box<dyn Fn(&imgui::Ui, &mut StructValue)>>,
    behaviour_field_value: HashMap<String, Box<dyn Fn(&imgui::Ui, &mut Value)>>,
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
    
    pub fn add_behaviour_enum<T: Reflect + 'static, F: Fn(&imgui::Ui, &mut EnumValue) + 'static>(&mut self, _value: T, behaviour: F) {
        self.behaviour_enum.insert(TypeId::of::<T>(), Box::new(behaviour));
    }

    pub fn add_behaviour_struct<T: Reflect + 'static, F: Fn(&imgui::Ui, &mut StructValue) + 'static>(&mut self, _value: T, behaviour: F) {
        self.behaviour_struct.insert(TypeId::of::<T>(), Box::new(behaviour));
    }

    pub fn add_behaviour_field_enum<T: Reflect + 'static, F: Fn(&imgui::Ui, &mut EnumValue) + 'static>(&mut self, _value: T, field_name: String, behaviour: F) {
        self.behaviour_field_enum.insert(field_name, Box::new(behaviour));
    }

    pub fn add_behaviour_field_struct<T: Reflect + 'static, F: Fn(&imgui::Ui, &mut StructValue) + 'static>(&mut self, _value: T, field_name: String, behaviour: F) {
        self.behaviour_field_struct.insert(field_name, Box::new(behaviour));
    }

    pub fn add_behaviour_field_value<T: Reflect + 'static, F: Fn(&imgui::Ui, &mut Value) + 'static>(&mut self, _value: T, field_name: String, behaviour: F) {
        self.behaviour_field_value.insert(field_name, Box::new(behaviour));
    }

    fn behaviour_enum(&self, ui: &imgui::Ui, value: &mut EnumValue) -> bool {
        if let Some(func) = self.behaviour_enum.get(&value.info.id) {
            func(ui, value);
            return true;
        }
        return false;
    }

    fn behaviour_struct(&self, ui: &imgui::Ui, value: &mut StructValue) -> bool {
        if let Some(func) = self.behaviour_struct.get(&value.info.id) {
            func(ui, value);
            return true;
        }
        return false;
    }

    fn behaviour_field_enum(&self, ui: &imgui::Ui, field_name: &String, value: &mut EnumValue) -> bool {
        if let Some(func) = self.behaviour_field_enum.get(field_name) {
            func(ui, value);
            return true;
        }
        return false;
    }

    fn behaviour_field_struct(&self, ui: &imgui::Ui, field_name: &String, value: &mut StructValue) -> bool {
        if let Some(func) = self.behaviour_field_struct.get(field_name) {
            func(ui, value);
            return true;
        }
        return false;
    }

    fn behaviour_field_value(&self, ui: &imgui::Ui, field_name: &String, value: &mut Value) -> bool {
        if let Some(func) = self.behaviour_field_value.get(field_name) {
            func(ui, value);
            return true;
        }
        return false;
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

pub fn inspect(ui: &imgui::Ui, data: &mut Value, name_field: Option<String>, settings: Option<InspectSettings>) {
    let mut state = InspectState::default();
    if let Some(s) = settings { state.settings = s; }
    let style = ui.push_style_var(imgui::StyleVar::ItemSpacing([0.0, 1.0]));
    state.inspect_intern(ui, data, name_field);
    style.pop(ui);
}

#[derive(Default)]
pub struct InspectState {
    labels: HashMap<String, usize>,
    name_parent: Option<String>,
    settings: InspectSettings,
}

impl InspectState {
    pub fn get_nocollidable_label(&mut self, label_old: String) -> String {
        if let Some(state) = self.labels.get_mut(&label_old.clone()) {
            *state += 1;
            return label_old + "#" + &state.to_string();
        } else {
            self.labels.insert(label_old.clone(), 0);
            return label_old;
        }
    }

    pub fn get_nocollidable_label_hide(&mut self, label_old: String) -> String {
        let mut new_label = label_old.clone();
        new_label.insert_str(0, "##");
        if let Some(state) = self.labels.get_mut(&new_label.clone()) {
            *state += 1;
            return new_label + "#" + &state.to_string();
        } else {
            self.labels.insert(new_label.clone(), 0);
            return new_label;
        }
    }
}

impl InspectState {
    fn inspect_intern(&mut self, ui: &imgui::Ui , data: &mut Value, name_field: Option<String>) {
        if let Value::Unit = data { return }
        if let Some(name_field) = name_field.as_ref() { if self.settings.behaviour_field_value(ui, name_field, data) { return } }

        let indent = ui.push_style_var(imgui::StyleVar::IndentSpacing(20.0));
        //let style_color = ui.push_style_color(imgui::StyleColor::FrameBg, [0.5, 0.5, 0.5, 1.0]);
        ui.align_text_to_frame_padding();
        let name_field_label = imgui::ImString::from(self.get_nocollidable_label(name_field.clone().unwrap_or(String::from("unknown"))));
        let pos = ui.cursor_pos();
        let mut new_pos = pos; new_pos[0] += 100.0;

        let label_fn = || {
            if name_field.is_some() {
                let mut new_name_field = name_field.as_ref().unwrap().clone();
                if new_name_field.chars().count() > 14 {
                    let mut name_field_trunc = new_name_field.split_at(11).0.to_string();
                    name_field_trunc.push_str("...");
                    new_name_field = name_field_trunc;
                }
                ui.text(new_name_field);
                if ui.is_item_hovered() {
                    ui.tooltip_text(name_field.as_ref().unwrap());
                }
                ui.set_cursor_pos(new_pos);
            }
            //ui.set_cursor_pos(new_pos);
            let mut name_field_label = name_field_label.clone().to_string();
            name_field_label.insert_str(0, "##");
            let name_field_label = imgui::ImString::new(name_field_label);
            ui.align_text_to_frame_padding();
            return name_field_label;
        };
        //if name_field.is_some() { ui.text(&name_field_label); ui.same_line_with_spacing(0.0, 0.0); ui.text(": "); ui.same_line_with_spacing(0.0, 0.0);}
        match data {
            Value::Bool(v) => {
                let name_field_label = label_fn();
                ui.checkbox(&name_field_label, v);
            }
            Value::U8(v) => {
                let name_field_label = label_fn();
                let mut v_i32 = *v as i32;
                ui.input_int(&name_field_label, &mut v_i32).enter_returns_true(true).build();
                *v = v_i32 as u8;
            }
            Value::U64(v) => {
                let name_field_label = label_fn();
                let mut v_i32 = *v as i32;
                ui.input_int(&name_field_label, &mut v_i32).enter_returns_true(true).build();
                *v = v_i32 as u64;
            }
            Value::I32(v) => {
                let name_field_label = label_fn();
                let mut v_i32 = *v as i32;
                ui.input_int(&name_field_label, &mut v_i32).enter_returns_true(true).build();
                *v = v_i32;
            }
            Value::I64(v) => {
                let name_field_label = label_fn();
                let mut v_string = imgui::ImString::new(v.to_string());
                if ui.input_text(&name_field_label, &mut v_string).chars_decimal(true).enter_returns_true(true).resize_buffer(true).build() {
                    let v_string = v_string.to_string();
                    let v_string: String = v_string
                        .chars()
                        .enumerate()
                        .filter(|(i, c)| c.is_numeric() || *i == 0 && *c == '-')
                        .map(|(_i, c)| c)
                        .collect();
                    *v = v_string.parse().unwrap_or_default();
                }
            }
            Value::F32(v) => {
                let name_field_label = label_fn();
                let mut v_f32 = *v as f32;
                ui.input_float(&name_field_label, &mut v_f32).enter_returns_true(true).build();
                *v = v_f32;
            }
            Value::String(string) => {
                let name_field_label = label_fn();
                let mut imgui_string = imgui::ImString::from(string.clone());
                ui.input_text(&name_field_label, &mut imgui_string).enter_returns_true(true).build();
                *string = imgui_string.to_string();
            }
            Value::Enum(v) => {
                let _name_field_label = label_fn();
                self.inspect_enum(ui, v, name_field);
            }
            Value::Option(v) => {
                let _name_field_label = label_fn();
                self.inspect_option(ui, v, name_field);
            }
            Value::Seq(v) => {
                let _name_field_label = label_fn();
                self.inspect_seq(ui, v, name_field);
            }
            Value::Tuple(v) => {
                let _name_field_label = label_fn();
                self.inspect_tuple(ui, v, name_field);
            }
            Value::Struct(struct_value) => {
                let _name_field_label = label_fn();
                self.inspect_struct(ui, struct_value, name_field);
                //ui.separator();
            }
            _ => {}
        }
        indent.pop(ui);
    }

    fn inspect_enum(&mut self, ui: &imgui::Ui, enum_value: &mut Box<EnumValue>, name_field: Option<String>) {
        if let Some(name_field) = name_field.as_ref() { if self.settings.behaviour_field_enum(ui, name_field, enum_value) { return } }
        if self.settings.behaviour_enum(ui, enum_value) { return }
        //if name_field.is_some() { ui.text(name_field.as_ref().unwrap()); ui.same_line_with_spacing(0.0, 10.0); };

        //if let Some(enum_settings) = self.settings.special_enum.get(enum_value.info.enum_name) {
            imgui::ComboBox::new(&imgui::ImString::from(self.get_nocollidable_label_hide(enum_value.info.enum_name.to_string()))).preview_value(&imgui::ImString::from(enum_value.variant.info.variant_name.to_string())).build(ui, || {
            //imgui::ComboBox::new(&imgui::ImString::from(name_field.clone().unwrap_or("unknown".to_string()))).preview_value(&imgui::ImString::from(enum_value.info.enum_name.to_string() + "::" + &enum_value.variant.info.variant_name.to_string())).build(ui, || {
                for variant in enum_value.info.variants.iter() {
                    if imgui::Selectable::new(&imgui::ImString::from(variant.variant_name.to_string())).build(ui) {
                        if variant.variant_name != enum_value.variant.info.variant_name {
                            enum_value.variant.info = variant.clone();
                            //enum_value.variant.value = variant.variant_type.get().default_value().unwrap();
                            enum_value.variant.value = self.settings.try_get_default_from(variant.variant_type.get()).or(variant.variant_type.get().default_value()).unwrap();
                        }
                    }
                }
            });
        //}

        ui.indent();
        self.inspect_intern(ui, &mut enum_value.variant.value, None);
        ui.unindent();
    }

    fn inspect_option(&mut self, ui: &imgui::Ui, option: &mut Box<OptionValue>, _name_field: Option<String>) {
        self.name_parent = None;

        //ui.same_line_with_spacing(0.0, 10.0);
        //ui.set_next_item_width(100.0);
        imgui::ComboBox::new(&imgui::ImString::from(self.get_nocollidable_label_hide("Option".to_string()))).preview_value(&imgui::ImString::from(option.value.as_ref().map_or("None", |_o| "Some" ).to_string())).build(ui, || {
            if imgui::Selectable::new(&imgui::ImString::from(self.get_nocollidable_label("None".to_string()))).build(ui) {
                if option.value.is_some() {
                    option.value = None;
                }
            }
            if imgui::Selectable::new(&imgui::ImString::from(self.get_nocollidable_label("Some".to_string()))).build(ui) {
                if option.value.is_none() {
                    option.value = self.settings.try_get_default_from(option.info.some.get()).or(option.info.some.get().default_value());
                    //option.value = option.info.some.get().default_value();
                }
            }
        });

        if let Some(value) = option.value.as_mut() {
            ui.indent();
            self.inspect_intern(ui, value, None);
            ui.unindent();
        }
    }

    fn inspect_struct(&mut self, ui: &imgui::Ui, struct_value: &mut StructValue, name_field: Option<String>) {
        if let Some(name_field) = name_field.as_ref() { if self.settings.behaviour_field_struct(ui, name_field, struct_value) { return } }
        if self.settings.behaviour_struct(ui, struct_value) { return }

        let name = struct_value.info.name;
        self.name_parent = Some(name.to_string());
        imgui::TreeNode::new(&imgui::ImString::from(self.get_nocollidable_label(name.to_string()))).build(ui, || {
            for field in struct_value.fields.iter_mut() {
                self.inspect_intern(ui, &mut field.value, Some(field.info.name.to_owned()))
            }
        });
    }

    fn inspect_tuple(&mut self, ui: &imgui::Ui, fields: &mut Vec<Value>, name_field: Option<String>) {
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

    fn inspect_seq(&mut self, ui: &imgui::Ui, seq_value: &mut SeqValue, name_field: Option<String>) {
        //if name_field.is_some() { ui.text(name_field.as_ref().unwrap()); ui.same_line_with_spacing(0.0, 10.0); };
        self.name_parent = None;

        if seq_value.info.fixed_size == Some(4) && seq_value.info.seq_type.get() == Type::U8 {
            let mut color = [0.0, 0.0, 0.0, 0.0];
            for (index, value) in seq_value.values.iter().enumerate() {
                if let Value::U8(a) = value {
                    color[index] = *a as f32 / 256.0;
                }
            }
            ui.set_next_item_width(200.0);
            imgui::ColorEdit::new(&imgui::ImString::from(self.get_nocollidable_label_hide("color".to_string())), &mut color).build(ui);
            for (index, value) in seq_value.values.iter_mut().enumerate() {
                if let Value::U8(a) = value {
                    *a = (color[index] * 256.0) as u8;
                }
            }
        } else {
            let mut to_delete = Vec::new();
            imgui::TreeNode::new(&imgui::ImString::from(self.get_nocollidable_label(name_field.clone().unwrap_or("List".to_string())))).build(ui, || {
                //ui.indent();
                for (index, field) in seq_value.values.iter_mut().enumerate() {
                    match field {
                        _ => {
                            self.inspect_intern(ui, field, name_field.clone());
                            if seq_value.info.fixed_size.is_none() {
                                ui.same_line_with_spacing(0.0, 10.0);
                                if ui.small_button(&imgui::ImString::from(self.get_nocollidable_label("Remove".to_string()))) {
                                    to_delete.push(index);
                                }
                            }
                        }
                    }
                }
                if seq_value.info.fixed_size.is_none() {
                    if ui.small_button(&imgui::ImString::from(self.get_nocollidable_label("Add".to_string()))) {
                        let new_value = seq_value.info.seq_type.get().default_value();
                        seq_value.values.push(new_value.unwrap());
                    }
                }
                to_delete.iter().rev().for_each(|&i| { seq_value.values.remove(i); });
                //ui.unindent();
            });
        }

        //if name_field.is_some() { ui.unindent() };
    }
}