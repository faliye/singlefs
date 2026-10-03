#!/usr/bin/env python3
"""把冻结副本的一份拷贝改成原型要的样子（只动拷贝，不动冻结副本与主仓）。
用法：patch-tree.py <拷贝根> <臂>，臂 = c1（丙1，原样）或 held（把 mount.rs 第 2259 行改回 HeldUntilFloorTakesEffect）。
两臂都做的：mount.rs 末尾加一个只读的可见性口子（probe_s2_placements_referenced_by_root）；工作区成员加 crates/probe-s2。"""
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
arm = sys.argv[2]
mount = root / "crates/singlefs-core/src/mount.rs"
lines = mount.read_text().split("\n")
target = "            RaiseFloorEntry::Unmount => ReclaimedReuse::Immediately,"
assert lines[2258] == target, f"第 2259 行不是预期的样子：{lines[2258]!r}"
if arm == "held":
    lines[2258] = "            RaiseFloorEntry::Unmount => ReclaimedReuse::HeldUntilFloorTakesEffect,"
elif arm == "perdisk":
    # 攻方自提的改法（零轮）：卸载那一格回收时，释放代不高于「每块盘上最新的有效根」最小者的那一截立即可用，其余照准入那一格扣到生效。
    text = "\n".join(lines)
    old = """    let reclaimed = allocator.reclaim_released_up_to(
        reclaim_floor(new_floor, oldest_valid_root_in_the_window),
        entry.reuse_of_the_reclaimed_slots(),
    );"""
    new = """    let reclaimed = {
        let floor_of_the_reclaim = reclaim_floor(new_floor, oldest_valid_root_in_the_window);
        match entry {
            RaiseFloorEntry::Unmount => {
                let newest_root_on_each_device_at_least = all_devices
                    .iter()
                    .map(|device| {
                        roots
                            .iter()
                            .filter(|root| !abandoned_by_table(root, &table) && device_of_txg(root.checkpoint_txg) == *device)
                            .map(|root| root.checkpoint_txg)
                            .max()
                            .unwrap_or(CheckpointTxg(0))
                    })
                    .min()
                    .unwrap_or(CheckpointTxg(0));
                let mut reclaimed = allocator.reclaim_released_up_to(
                    floor_of_the_reclaim.min(newest_root_on_each_device_at_least),
                    ReclaimedReuse::Immediately,
                );
                reclaimed.extend(allocator.reclaim_released_up_to(
                    floor_of_the_reclaim,
                    ReclaimedReuse::HeldUntilFloorTakesEffect,
                ));
                reclaimed
            }
            RaiseFloorEntry::Admission => allocator.reclaim_released_up_to(
                floor_of_the_reclaim,
                entry.reuse_of_the_reclaimed_slots(),
            ),
        }
    };"""
    assert text.count(old) == 1, "回收那一段不在预期的样子"
    lines = text.replace(old, new).split("\n")
elif arm != "c1":
    sys.exit(f"臂只认 c1 / held / perdisk：{arm}")
helper = '''
/// probe-s2（攻方原型）：只在冻结副本的拷贝里加的可见性口子，不改行为。交回 (按账取的, 按映射现算的) 两份。
#[allow(clippy::ptr_arg, clippy::type_complexity)]
pub fn probe_s2_placements_referenced_by_root<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    root: &RootRecord,
) -> (
    Option<Vec<(DeviceIdentity, SlotNumber, u64)>>,
    Option<Vec<(DeviceIdentity, SlotNumber, u64)>>,
) {
    let system_configuration =
        crate::recovery::choose_system_configuration(devices).expect("probe-s2：系统配置读得出");
    let start = unit_area_start_of_the_chosen_system_configuration(&system_configuration);
    (
        placements_referenced_by_root(devices, start, root),
        placements_referenced_by_root_computed_from_the_mapping(devices, start, root),
    )
}
'''
mount.write_text("\n".join(lines) + helper)
cargo = root / "Cargo.toml"
text = cargo.read_text()
anchor = '    "crates/singlefs-reference-filesystem",\n'
assert anchor in text
cargo.write_text(text.replace(anchor, anchor + '    "crates/probe-s2",\n', 1))
print(f"patched {root} arm={arm}")
