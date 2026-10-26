use vstd::prelude::*;
verus! {
pub trait BlockDevice { fn write(&mut self, slot: u64, bytes: &[u8]) -> Result<(), u8>; }
pub struct PoolWriter<'a, D: BlockDevice> { pub devices: &'a mut D }
// persist_publish_writes 的形状：泛型设备 + &mut 借用 + 逐项写
pub fn persist<D: BlockDevice>(w: &mut PoolWriter<'_, D>, units: &Vec<(u64, Vec<u8>)>) -> Result<(), u8> {
    for unit in units.iter() {
        w.devices.write(unit.0, &unit.1)?;
    }
    Ok(())
}
}
fn main() {}
