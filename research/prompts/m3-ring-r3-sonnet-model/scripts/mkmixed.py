import os
OLD_START='        match self.ranges.read_back() {\n            UnreadableRangeReadBack::DeviceError => {'
NEW='''        let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
        let mixed = std::env::var("R3_MIXED_SLOT_READ_BACK").is_ok();
        let kind_of = |range: &UnreadableRange| -> UnreadableRangeReadBack {
            if mixed && (range.offset_in_bytes == 0 || range.offset_in_bytes == spacing) {
                if range.device.0 == 0 {
                    UnreadableRangeReadBack::DeviceError
                } else {
                    UnreadableRangeReadBack::Zeros
                }
            } else {
                self.ranges.read_back()
            }
        };
        if failing.iter().any(|range| kind_of(range) == UnreadableRangeReadBack::DeviceError) {
            return Err(singlefs_core::block_device::BlockDeviceError::InputOutput(
                std::io::Error::other("用例点名的这一段读坏"),
            ));
        }
        match UnreadableRangeReadBack::Zeros {
            UnreadableRangeReadBack::DeviceError => {'''
for d in ('g3','g3-eio','g3-dev'):
    p=f'/tmp/claude-1000/m3-ring-r3-sonnet/{d}/crates/singlefs-harness/tests/common/mod.rs'
    t=open(p).read()
    assert t.count(OLD_START)==1
    open(p,'w').write(t.replace(OLD_START,NEW))
print('ok')
