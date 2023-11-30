use fusebox::FuseBox;

use self::description::{
    DeviceDescription,
    Param,
};

pub mod description;

pub mod impls;

pub trait Device {
    fn output(&mut self) -> f32;
    fn set_param_indexed(&mut self, idx: u8, val: f32);
}

const fn dd(
    name: &'static str,
    params: &'static [Param],
    make: fn(&mut FuseBox<dyn Device + Send + Sync>) -> usize,
) -> DeviceDescription {
    DeviceDescription { name, params, make }
}

pub static DEVICES: &[DeviceDescription] = {
    use Param::*;

    &[dd("PLACEHOLDER", &[In("PARAM")], |_| 0)]
};
