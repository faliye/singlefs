#!/usr/bin/env python3
"""攻方变异（m3-batch10z-xy-code-r1，opus 腿）：mutants.py <仓副本根> <变异名> 改那份副本；mutants.py x --list 列名字。
原文在被改文件里恰好命中一次才改，否则退 3。只改副本，不碰冻结副本与仓。ARM- 开头的不是变异，是 K1-2 另一条臂（推环入口也先比次序）的造法。"""
import pathlib
import sys

MS = "crates/singlefs-core/src/mounted_session.rs"
TABLE_CHECK = "        if !self\n            .parameters_and_device_table\n            .is_the_device_table_of_the_mount(devices)\n        {"
TABLE_REFUSAL = """        if !self.parameters_and_device_table.is_the_device_table_of_the_mount(devices) {
            return Err(WaysOutOfTheWideProtectionRefused::DeviceTableOtherThanTheOneOfTheMount {
                device_table_of_the_mount: self.parameters_and_device_table.device_identities_in_table_order().to_vec(),
                device_table_handed_in: devices.iter().map(|(identity, _)| *identity).collect(),
            });
        }
"""
REHEARSE = ("        let _ = rehearse_the_push_then_the_remainder_of_the_change(self.parameters_and_device_table.parameters_on_disk(), devices, "
            "&self.allocator, version_refused_on, self.instance, self.shadow_ledger, stuck.remainder_of_the_change.as_remainder());\n")
PUSH_ON = "        Self::push_the_root_ring_on(\n            parameters_and_device_table.parameters_on_disk(),\n            devices,\n            allocator,\n            current,\n            *instance,\n        )"

MUTANTS = {
    # 上一轮两腿都活下来的那一个（与变异表第二十四块那一行同一原文）
    "walkSkipped": (MS, "        RemainderOfTheUserChange::WalkTheQueue => {\n            let mut version_to_walk",
                    "        RemainderOfTheUserChange::WalkTheQueue if true => Ok(()),\n        RemainderOfTheUserChange::WalkTheQueue => {\n            let mut version_to_walk"),
    # 这一轮新加的盘表那一道
    "N1-table-check-before-the-version-check": (MS, "    ) -> Result<WaysOutOfTheWideProtection, WaysOutOfTheWideProtectionRefused> {\n        let is_the_version_refused_on",
                    "    ) -> Result<WaysOutOfTheWideProtection, WaysOutOfTheWideProtectionRefused> {\n" + TABLE_REFUSAL + "        let is_the_version_refused_on"),
    "N2-table-check-ignores-the-order": (MS, TABLE_CHECK,
                    "        if !{\n            let mut handed: Vec<DeviceIdentity> = devices.iter().map(|(identity, _)| *identity).collect();\n            handed.sort();\n"
                    "            let mut mount = self.parameters_and_device_table.device_identities_in_table_order().to_vec();\n            mount.sort();\n            handed == mount\n        }\n        {"),
    "N3-table-check-only-the-count": (MS, TABLE_CHECK,
                    "        if devices.len() != self.parameters_and_device_table.device_identities_in_table_order().len()\n        {"),
    "N4-refusal-records-the-mount-table-as-handed-in": (MS, "                    device_table_handed_in: devices.iter().map(|(identity, _)| *identity).collect(),\n                },\n            );\n        }\n        let pushing_the_root_ring",
                    "                    device_table_handed_in: self.parameters_and_device_table.device_identities_in_table_order().to_vec(),\n                },\n            );\n        }\n        let pushing_the_root_ring"),
    "N5-rehearse-once-before-the-table-check": (MS, TABLE_CHECK, REHEARSE + TABLE_CHECK),
    # 报里拷下的改动读回、留下
    "N6-as-remainder-drops-the-change": (MS, "            RemainderOfTheUserChangeKeptByTheRefusal::WalkTheQueueThenPublishTheChange(change) => {\n                RemainderOfTheUserChange::WalkTheQueueThenPublishTheChange(change.as_user_change())\n            }",
                    "            RemainderOfTheUserChangeKeptByTheRefusal::WalkTheQueueThenPublishTheChange(_change) => {\n                RemainderOfTheUserChange::WalkTheQueue\n            }"),
    "N7-kept-from-keeps-only-the-walk": (MS, "            RemainderOfTheUserChange::WalkTheQueueThenPublishTheChange(change) => {\n                RemainderOfTheUserChangeKeptByTheRefusal::WalkTheQueueThenPublishTheChange(\n                    UserChangeKeptByTheRefusal::kept_from(change),\n                )\n            }",
                    "            RemainderOfTheUserChange::WalkTheQueueThenPublishTheChange(_change) => {\n                RemainderOfTheUserChangeKeptByTheRefusal::WalkTheQueue\n            }"),
    # 只进内存的盘：物理块写死 512 的另两处
    "N8-block-bytes-512": (MS, "    fn block_bytes(&self) -> u64 {\n        u64::from(self.physical_block_size.0)\n    }",
                    "    fn block_bytes(&self) -> u64 {\n        512\n    }"),
    "N9-over-the-disk-takes-512": (MS, "            physical_block_size: disk.probe_physical_block_size(),\n            written_blocks: BTreeMap::new(),",
                    "            physical_block_size: { let _ = disk.probe_physical_block_size(); PhysicalBlockSizeInBytes(512) },\n            written_blocks: BTreeMap::new(),"),
    # K1-2 另一条臂：推环入口也先照 publish_user_change 那样比盘表次序，不符在任何读写之前拒（借 DevicesHandedIn 那一种装，臂的造法，不是提案的报法）
    "ARM-push-checks-the-table-order": (MS, PUSH_ON,
                    "        if !parameters_and_device_table.is_the_device_table_of_the_mount(devices) {\n"
                    "            return Err(RootRingPushRefused::DevicesHandedIn(Box::new(\n"
                    "                MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration { disagreeing_fields: Vec::new(), disagreeing_device_table: Vec::new() },\n"
                    "            )));\n        }\n" + PUSH_ON),
}


def main():
    if len(sys.argv) >= 3 and sys.argv[2] == "--list":
        print("\n".join(name for name in MUTANTS if not name.startswith("ARM-")))
        return
    root = pathlib.Path(sys.argv[1])
    name = sys.argv[2]
    rel, old, new = MUTANTS[name]
    path = root / rel
    text = path.read_text()
    count = text.count(old)
    if count != 1:
        print(f"{name}: 原文命中 {count} 次（要恰好一次）", file=sys.stderr)
        sys.exit(3)
    path.write_text(text.replace(old, new))
    print(f"{name}: 改了 {rel}")


if __name__ == "__main__":
    main()
