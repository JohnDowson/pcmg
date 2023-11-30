use std::{
    collections::{
        BTreeMap,
        VecDeque,
    },
    fs,
    path::Path,
    sync::Arc,
};

use devices::description::DeviceKind;
use egui::{
    Context,
    DragValue,
    Pos2,
    Ui,
    Vec2,
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

pub fn ser_device_description<S>(dd: &DeviceKind, ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match dd {
        DeviceKind::Control => "control",
        DeviceKind::MidiControl => "midicontrol",
        DeviceKind::Audio(dd) => devices::DEVICES[*dd].name,
        DeviceKind::Output => "output",
    }
    .serialize(ser)
}

pub fn de_device_description<'de, D>(de: D) -> Result<DeviceKind, D::Error>
where
    D: Deserializer<'de>,
    D::Error: serde::de::Error,
{
    let s = String::deserialize(de)?;

    Ok(match &*s {
        "control" => DeviceKind::Control,
        "midicontrol" => DeviceKind::MidiControl,
        "output" => DeviceKind::Output,
        s => devices::DEVICES
            .iter()
            .enumerate()
            .find_map(|(id, dd)| {
                if dd.name == s {
                    Some(DeviceKind::Audio(id))
                } else {
                    None
                }
            })
            .ok_or(serde::de::Error::custom(format!(
                "{s} is not a known device name, I only know these devices: {:?}",
                devices::DEVICES
            )))?,
    })
}
pub struct STQueue<T> {
    inner: Arc<eframe::epaint::mutex::Mutex<VecDeque<T>>>,
}

impl<T> Clone for STQueue<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T> STQueue<T> {
    pub fn new() -> Self {
        Self {
            inner: Default::default(),
        }
    }

    pub fn put(&self, msg: T) {
        self.inner.lock().push_front(msg)
    }

    pub fn get(&self) -> Option<T> {
        self.inner.lock().pop_back()
    }
}

impl<T> Default for STQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}
