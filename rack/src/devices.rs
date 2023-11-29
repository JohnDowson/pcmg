use fusebox::FuseBox;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DeviceDescription {
    pub name: &'static str,
    pub params: &'static [&'static str],
    pub make: fn(&mut FuseBox<dyn Device + Send + Sync + 'static>) -> usize,
}

pub trait Device {
    fn output(&mut self) -> f32;
    fn set_param_indexed(&mut self, idx: u8, val: f32);
}

const fn dd(
    name: &'static str,
    params: &'static [&'static str],
    make: fn(&mut FuseBox<dyn Device + Send + Sync>) -> usize,
) -> DeviceDescription {
    DeviceDescription { name, params, make }
}

pub static DEVICES: &[DeviceDescription] = &[dd("PLACEHOLDER", &["PARAM"], |_| 0)];
