#[allow(unused_imports)]
use model_core::thing::{Describe, Thing};

#[test]
fn thing_describes_itself() {
    assert_eq!(Thing::new(1).describe(), "thing-2");
}
