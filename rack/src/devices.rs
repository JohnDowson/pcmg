use fusebox::FuseBox;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DeviceDescription {
    pub name: &'static str,
    pub kind: DeviceKind,
    pub params: &'static [Param],
    pub make: fn(&mut FuseBox<dyn Device + Send + Sync + 'static>) -> usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeviceKind {
    Foo,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Param {
    In(&'static str),
    Out(&'static str),
}

impl std::fmt::Display for Param {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self, f)
    }
}

pub trait Device {
    fn output(&mut self) -> f32;
    fn set_param_indexed(&mut self, idx: u8, val: f32);
}

const fn dd(
    name: &'static str,
    kind: DeviceKind,
    params: &'static [Param],
    make: fn(&mut FuseBox<dyn Device + Send + Sync>) -> usize,
) -> DeviceDescription {
    DeviceDescription {
        name,
        kind,
        params,
        make,
    }
}

use DeviceKind::*;
use Param::*;
pub static DEVICES: &[DeviceDescription] = &[dd("PLACEHOLDER", Foo, &[In("PARAM")], |_| 0)];
