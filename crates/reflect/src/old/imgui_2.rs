use serde::Deserialize;

use serde_reflection::Value;
use serde_reflection::ContainerFormat;
use serde_reflection::Named;
use serde_reflection::Format;
use serde_reflection::VariantFormat;
use serde_reflection::Registry;


pub fn inspect<'de, T: Deserialize<'de>>(ui: &imgui::Ui , data: &mut Value, registry: Registry, name_field: Option<String>) {
    let mut state = InspectState::default();
    let data_name = serde_name::trace_name::<T>().unwrap();
    state.registry = registry;
    state.inspect_type_name(ui, data, data_name.to_string(), Some(data_name.to_owned()));
}

#[derive(Default)]
struct InspectState {
    labels: std::collections::HashSet<String>,
    registry: Registry,
}

impl InspectState {
    fn get_nocollidable_label(&mut self, label_old: String) -> String {
        if !self.labels.insert(label_old.clone()) {
            let mut new_label = label_old.clone();
            if let Some(number) = new_label.chars().last().unwrap().to_digit(10) {
                if number < 9 {
                    new_label.pop();
                    new_label.push(std::char::from_digit(number + 1, 10).unwrap());
                    return self.get_nocollidable_label(new_label);
                } else {
                    new_label.push('0');
                    return self.get_nocollidable_label(new_label);
                }
            } else {
                new_label.push('0');
                return self.get_nocollidable_label(new_label);
            }
        } else {
            return label_old
        }
    }
}

impl InspectState {
    fn inspect_type_name(&mut self, ui: &imgui::Ui, data: &mut Value, data_name: String, name_field: Option<String>) {
        let data_format = self.registry.get(&data_name).unwrap().clone();
        let name_field_label = imgui::ImString::from(self.get_nocollidable_label(name_field.clone().unwrap_or(String::from("unknown"))));
        match data_format {
            ContainerFormat::UnitStruct => {
                assert!(*data == Value::Unit);
            }
            ContainerFormat::NewTypeStruct(format) => {
                self.inspect_intern(ui, data, format.as_ref(), name_field);
            }
            ContainerFormat::TupleStruct(formats) => {
                if let Value::Seq(values) = data {
                    self.inspect_tuple(ui, values, &formats, name_field);
                } else {
                    panic!()
                }
            }
            ContainerFormat::Struct(named_formats) => {
                if let Value::Seq(values) = data {
                    self.inspect_struct(ui, &data_name, values, &named_formats, name_field);
                } else {
                    panic!()
                }
            }
            ContainerFormat::Enum(enum_variants) => {
                if let Value::Variant(id, value) = data {
                    self.inspect_variant(ui, id, value, &enum_variants, name_field);
                } else {
                    panic!()
                }
            }
        }
    }
    
    fn inspect_intern(&mut self, ui: &imgui::Ui , data: &mut Value, data_format: &Format, name_field: Option<String>) {
        let name_field_label = imgui::ImString::from(self.get_nocollidable_label(name_field.clone().unwrap_or(String::from("unknown"))));
        match data {
            Value::Bool(v) => {
                ui.checkbox(&name_field_label, v);
            }
            Value::U8(v) => {
                let mut v_i32 = *v as i32;
                ui.input_int(&name_field_label, &mut v_i32).enter_returns_true(true).build();
                *v = v_i32 as u8;
            }
            Value::U64(v) => {
                let mut v_i32 = *v as i32;
                ui.input_int(&name_field_label, &mut v_i32).enter_returns_true(true).build();
                *v = v_i32 as u64;
            }
            Value::I32(v) => {
                let mut v_i32 = *v as i32;
                ui.input_int(&name_field_label, &mut v_i32).enter_returns_true(true).build();
                *v = v_i32;
            }
            Value::I64(v) => {
                let mut v_i32 = *v as f32;
                ui.input_float(&name_field_label, &mut v_i32).enter_returns_true(true).build();
                *v = v_i32 as i64;
            }
            Value::F32(v) => {
                let mut v_f32 = *v as f32;
                ui.input_float(&name_field_label, &mut v_f32).enter_returns_true(true).build();
                *v = v_f32;
            }
            Value::Variant(i, v) => {
                panic!()
            }
            Value::Option(v) => {
                self.inspect_option(ui, v, data_format, name_field);
            }
            _ => {}
        }
    }

    fn inspect_variant(&mut self, ui: &imgui::Ui, variant_id: &mut u32, variant: &mut Box<Value>, data_format: &std::collections::BTreeMap<u32, Named<VariantFormat>>, name_field: Option<String>) {
        if name_field.is_some() { ui.text(name_field.as_ref().unwrap()) };
        let variant_format = data_format.get(variant_id).unwrap();
        ui.indent();
        ui.text(variant_format.name.clone());
        ui.indent();

        match &variant_format.value {
            VariantFormat::Variable(_) => {
                panic!()
            }
            VariantFormat::Unit => {

            }
            VariantFormat::NewType(format) => {
                self.inspect_intern(ui, variant, format.as_ref(), name_field);
            }
            VariantFormat::Tuple(formats) => {
                if let Value::Seq(values) = variant.as_mut() {
                    self.inspect_tuple(ui, values, &formats, name_field);
                } else {
                    panic!()
                }
            }
            VariantFormat::Struct(named_formats) => {

            }
        }

        ui.unindent();
        ui.unindent();
    }

    fn inspect_option(&mut self, ui: &imgui::Ui, option: &mut Option<Box<Value>>, format: &Format, name_field: Option<String>) {
        if name_field.is_some() { ui.text(name_field.as_ref().unwrap()) };
        ui.indent();

        if let Some(field) = option {
            self.inspect_intern(ui, field, format, name_field);
        } else {
            ui.text("None");
        }

        ui.unindent();
    }

    fn inspect_struct(&mut self, ui: &imgui::Ui, name: &String, fields: &mut Vec<Value>, fields_format: &Vec<Named<Format>>, name_field: Option<String>) {
        if name_field.is_some() { ui.text(name_field.as_ref().unwrap()) };
        ui.indent();

        for (field, format) in fields.iter_mut().zip(fields_format) {
            self.inspect_intern(ui, field, &format.value, Some(format.name.to_owned()));
        }

        ui.unindent();
    }

    fn inspect_tuple(&mut self, ui: &imgui::Ui , fields: &mut Vec<Value>, formats: &Vec<Format>, name_field: Option<String>) {
        if name_field.is_some() { ui.indent() };

        for (field, format) in fields.iter_mut().zip(formats) {
            match field {
                _ => {
                    self.inspect_intern(ui, field, format, name_field.clone())
                }
            }
        }

        if name_field.is_some() { ui.unindent() };
    }
}