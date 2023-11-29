mod in_port;
mod out_port;

pub use in_port::InPort;
pub use out_port::OutPort;

use crate::widget_description::WidFull;

pub struct Cable {
    pub a_id: WidFull,
    pub b_id: WidFull,
}
