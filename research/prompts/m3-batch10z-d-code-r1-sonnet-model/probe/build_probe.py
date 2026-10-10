import re,sys
src=open('/home/fy5090/.local/share/singlefs/work/m3-batch10z-d-code-r1-sonnet/repo/crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs').read().split('\n')
head='\n'.join(src[21:481])   # lines 22..481
# fault with zero mode
head=head.replace("""struct ReadFaultOnChosenSlots {
    ranges: Rc<Vec<(DeviceIdentity, u64, u64)>>,
    is_active: Rc<Cell<bool>>,
}""","""struct ReadFaultOnChosenSlots {
    ranges: Rc<Vec<(DeviceIdentity, u64, u64)>>,
    is_active: Rc<Cell<bool>>,
    zero_mode: Rc<Cell<bool>>,
}""")
head=head.replace("""            is_active: Rc::new(Cell::new(false)),
        }""","""            is_active: Rc::new(Cell::new(false)),
            zero_mode: Rc::new(Cell::new(false)),
        }""")
head=head.replace("""        if self.fault.refuses(self.identity, offset.0, length_in_bytes) {
            return Err(""","""        if self.fault.refuses(self.identity, offset.0, length_in_bytes) {
            if self.fault.zero_mode.get() {
                buffer.fill(0);
                return Ok(());
            }
            return Err(""")
assert head.count("zero_mode: Rc::new")==2, head.count("zero_mode: Rc::new")
head=head.replace("use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};","use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, SlotNumber};")
head=head.replace("""    record_one_publish_after_overwrites, DeviceOverASparseImage""","""    record_one_publish_after_overwrites, root_is_in_the_ring, units_of_r_overwritten_on, DeviceOverASparseImage""")
open('head.rs','w').write("//! 草稿探测（K2/K3 正推腿），不入库。\n#![allow(dead_code, unused_imports, clippy::all)]\n"+head+"\n")
