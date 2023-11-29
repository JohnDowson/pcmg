use std::{
    collections::BTreeMap,
    fs,
    path::Path,
};

use egui::{
    Context,
    Window,
};
use serde::{
    Deserialize,
    Deserializer,
    Serialize,
    Serializer,
};
use uuid::Uuid;
use widget_description::WidgetDescription;

pub mod container;
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
