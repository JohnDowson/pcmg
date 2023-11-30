use std::{
    collections::BTreeMap,
    fs,
    path::Path,
};

use devices::{
    DeviceDescription,
    DEVICES,
};
use egui::{
    Context,
    DragValue,
    Pos2,
    Ui,
    Vec2,
    Window,
};
use emath::pos2;
use serde::{
    Deserialize,
    Deserializer,
    Serialize,
    Serializer,
};
use uuid::Uuid;
use widget_description::WidgetDescription;

pub mod container;
pub mod devices;
pub mod graph;
pub mod widget_description;
pub mod widgets;

pub fn widget_prefabs(
    path: impl AsRef<Path>,
) -> Result<BTreeMap<Uuid, WidgetDescription>, Box<dyn std::error::Error>> {
    fs::read_to_string(path).map_err(Into::into).and_then(|s| {
        let prefabs = serde_yaml::from_str(&s)?;
        Ok(prefabs)
    })
}

pub fn widget_name(uuid: Option<Uuid>, widgets: &BTreeMap<Uuid, WidgetDescription>) -> String {
    if let Some(uuid) = uuid {
        format!("{}: {}", uuid, widgets[&uuid].name)
    } else {
        "None".into()
    }
}

pub fn pos_drag_value(ui: &mut Ui, label: impl AsRef<str>, pos: &mut Pos2) {
    two_drag_value(ui, label.as_ref(), "X", "Y", &mut pos.x, &mut pos.y)
}

pub fn vec_drag_value(ui: &mut Ui, label: impl AsRef<str>, vec: &mut Vec2) {
    two_drag_value(ui, label.as_ref(), "X", "Y", &mut vec.x, &mut vec.y)
}

pub fn two_drag_value(
    ui: &mut Ui,
    label: impl AsRef<str>,
    l1: impl AsRef<str>,
    l2: impl AsRef<str>,
    v1: &mut f32,
    v2: &mut f32,
) {
    ui.horizontal(|ui| {
        ui.label(label.as_ref());
        ui.label(l1.as_ref());
        ui.add(DragValue::new(v1));
        ui.label(l2.as_ref());
        ui.add(DragValue::new(v2));
    });
}

pub fn error_window(error: &mut Option<Box<dyn std::error::Error>>, ctx: &Context) {
    Window::new("Error").resizable(false).show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.label(error.as_ref().unwrap().to_string());
            if ui.button("Ok").clicked() {
                error.take();
            }
        })
    });
}

fn ser_btree_as_vec<S, T>(map: &BTreeMap<usize, T>, ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Serialize,
{
    map.values().collect::<Vec<_>>().serialize(ser)
}

fn de_vec_as_btree<'de, D, T>(de: D) -> Result<BTreeMap<usize, T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    let vec: Vec<T> = Vec::deserialize(de)?;
    Ok(vec.into_iter().enumerate().collect())
}

pub fn ser_device_description<S>(dd: &DeviceDescription, ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    dd.name.serialize(ser)
}

pub fn de_device_description<'de, D>(de: D) -> Result<DeviceDescription, D::Error>
where
    D: Deserializer<'de>,
    D::Error: serde::de::Error,
{
    let s = String::deserialize(de)?;
    DEVICES
        .iter()
        .find(|d| d.name == s)
        .cloned()
        .ok_or(serde::de::Error::custom(format!(
            "{s} is not a known device name, I only know these devices: {DEVICES:?}"
        )))
}
