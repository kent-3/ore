pub mod consts;
pub mod error;
pub mod event;
pub mod instruction;
pub mod sdk;
pub mod state;

pub mod prelude {
    pub use crate::consts::*;
    pub use crate::error::*;
    pub use crate::event::*;
    pub use crate::instruction::*;
    pub use crate::sdk::*;
    pub use crate::state::*;
}

use steel::*;

declare_id!("CWyZM2fBZUJzxM7rgw2Q8sXAsBfj3LNQVMmWWiox1r3F");
