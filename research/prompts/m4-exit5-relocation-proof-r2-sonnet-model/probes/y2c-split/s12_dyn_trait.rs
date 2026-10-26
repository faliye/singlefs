use vstd::prelude::*;
verus! {
pub trait Device {
    fn read(&self, offset: u64, length: usize) -> (r: Option<Vec<u8>>);
}
pub fn s12_dyn_trait(d: &dyn Device) -> (r: usize)
{
    match d.read(0, 4) {
        Some(v) => v.len(),
        None => 0,
    }
}
}
fn main() {}
