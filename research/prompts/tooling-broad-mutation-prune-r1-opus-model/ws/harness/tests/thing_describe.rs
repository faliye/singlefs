//! S4a / S4b：方法语法叫 describe，方法解析在固有方法与 trait 方法之间选。
#[allow(unused_imports)]
use model_core::thing::{Describe, Gadget, Thing};

#[test]
fn a_thing_describes_itself_with_the_next_number() {
    assert_eq!(Thing(1).describe(), "thing-2");
}

#[test]
fn a_gadget_describes_itself_with_the_next_number() {
    assert_eq!(Gadget(1).describe(), "gadget-2");
}
