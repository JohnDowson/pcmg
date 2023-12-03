use slotmap::{
    SecondaryMap,
    SlotMap,
};

use crate::devices::description::DeviceKind;

use super::{
    DeviceId,
    InputId,
    OutputId,
};

#[derive(Debug)]
pub struct Node {
    pub devices: SlotMap<DeviceId, DeviceKind>,
    pub input_to_param: SecondaryMap<InputId, (DeviceId, usize)>,
    pub output_to_param: SecondaryMap<OutputId, (DeviceId, usize)>,
}

impl Node {
    pub fn empty() -> Self {
        Self {
            devices: Default::default(),
            input_to_param: Default::default(),
            output_to_param: Default::default(),
        }
    }
}
