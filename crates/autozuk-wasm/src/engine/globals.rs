use std::cell::RefCell;

use crate::engine::{Region, SimContext};

thread_local! {
    pub(crate) static REGION_CACHE: RefCell<Vec<(u8, Region)>> = const { RefCell::new(Vec::new()) };
    pub(crate) static SIM_CONTEXT: RefCell<Option<SimContext>> = const { RefCell::new(None) };
}
