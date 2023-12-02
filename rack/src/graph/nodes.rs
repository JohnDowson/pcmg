use crate::{
    devices::description::DeviceDescription,
    widgets::SlotWidget,
};

pub struct Node {
    device: DeviceDescription,
    visual: Option<Box<dyn SlotWidget>>,
}
