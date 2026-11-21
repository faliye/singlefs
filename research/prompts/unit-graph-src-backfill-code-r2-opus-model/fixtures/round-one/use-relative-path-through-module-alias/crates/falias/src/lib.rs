pub mod deep {
    pub mod real {
        pub fn helper_total() -> u32 {
            7
        }
    }
}

pub mod other {
    pub mod short {
        pub fn helper_total() -> u32 {
            8
        }
    }
}

use crate::deep::real as short;
use short::helper_total;

pub fn entry_point() -> u32 {
    helper_total()
}
