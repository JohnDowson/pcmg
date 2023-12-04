use slotmap::SecondaryMap;

use super::{
    DeviceId,
    InputId,
    OutputId,
};

#[derive(Debug)]
pub struct Node {
    pub input_to_param: SecondaryMap<InputId, (DeviceId, usize)>,
    pub output_to_param: SecondaryMap<OutputId, (DeviceId, usize)>,
}

impl Node {
    pub fn empty() -> Self {
        Self {
            input_to_param: Default::default(),
            output_to_param: Default::default(),
        }
    }
}
