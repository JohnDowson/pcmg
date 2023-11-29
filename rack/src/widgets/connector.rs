mod in_port;
mod out_port;

pub use in_port::InPort;
pub use out_port::OutPort;

use crate::graph::ModuleId;

pub struct Waddr {
    module: ModuleId,
    widget: u16,
}

pub struct Cable {
    pub a_id: Waddr,
    pub b_id: Waddr,
}
