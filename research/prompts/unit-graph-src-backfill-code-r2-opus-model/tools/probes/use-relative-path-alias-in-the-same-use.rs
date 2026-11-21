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

macro_rules! bring_in_the_real_module_as_short {
    () => {
        use crate::deep::real as short;
    };
}

pub mod through_two_uses {
    use crate::deep::real as short;
    use short::helper_total;

    pub fn entry_point() -> u32 {
        helper_total()
    }
}

pub mod through_one_use {
    use {crate::deep::real as short, short::helper_total};

    pub fn entry_point() -> u32 {
        helper_total()
    }
}

pub mod through_a_macro {
    bring_in_the_real_module_as_short!();
    use short::helper_total;

    pub fn entry_point() -> u32 {
        helper_total()
    }
}

fn main() {
    println!("runtime through_two_uses={}", through_two_uses::entry_point());
    println!("runtime through_one_use={}", through_one_use::entry_point());
    println!("runtime through_a_macro={}", through_a_macro::entry_point());
}
