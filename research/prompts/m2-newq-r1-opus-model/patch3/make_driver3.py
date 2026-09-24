#!/usr/bin/env python3
"""从 patch/m2_newq_attack.rs 派生第三版驱动 m2_newq_attack3.rs（接手腿加的故障 F8–F10、N2 拒绝臂、复用检测列）。"""
import pathlib
here = pathlib.Path(__file__).parent
src = (here.parent / "patch" / "m2_newq_attack.rs").read_text()
def rep(old, new):
    global src
    n = src.count(old); assert n == 1, (n, old[:90]); src = src.replace(old, new)

rep("""//! m2-newq-r1 云端攻方腿的驱动（只在副本里跑）：N1 / N2 / N3 的候选臂 × 故障 × 用户后缀全扫。""",
"""//! m2-newq-r1 云端攻方腿的驱动第三版（接手腿，只在副本 repo3 里跑）：在第一版之上加 F8–F10、N2 拒绝臂、复用检测列。""")
rep("""use singlefs_core::transaction::{
    publish_overwrite, FirstFile,""", """use singlefs_core::transaction::{
    publish_new_inodes, publish_overwrite, FirstFile,""")
rep("""use singlefs_core::recovery::{recover""", """use singlefs_core::inode_tree::InodeLeafContainerIndexInTree;
use singlefs_core::recovery::{recover""")
rep("""enum Persistence {
    Once,
    UntilWritten,
}""", """enum Persistence {
    Once,
    UntilWritten,
    /// 读写都失败、永远（坏介质：写不进去，写也报错）。
    Forever,
}""")
rep("""    fn write_at(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8], durability: WriteDurability) -> Result<(), BlockDeviceError> {
        let start = offset.0;
        let end = start + bytes.len() as u64;
        for fault in &mut self.plan.borrow_mut().faults {""", """    fn write_at(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8], durability: WriteDurability) -> Result<(), BlockDeviceError> {
        let start = offset.0;
        let end = start + bytes.len() as u64;
        if self.plan.borrow().faults.iter().any(|fault| fault.active && fault.persistence == Persistence::Forever
            && fault.device == self.identity && fault.start < end && start < fault.end) {
            self.plan.borrow_mut().writes_refused += 1;
            return Err(BlockDeviceError::InputOutput(std::io::Error::other("injected write error")));
        }
        for fault in &mut self.plan.borrow_mut().faults {""")
rep("""struct FaultPlan {
    faults: Vec<ReadFault>,
    reads_refused: u64,
}""", """struct FaultPlan {
    faults: Vec<ReadFault>,
    reads_refused: u64,
    writes_refused: u64,
}""")
rep("""    MappingPointsAtLeafContainerReleasedInTheSamePublish,
}""", """    MappingPointsAtLeafContainerReleasedInTheSamePublish,
    /// F8 写者错（落盘）：注入那次发布把映射里数据单元那条的槽号写成现行实例表的槽，节点照常封、照常落盘。
    WriterBugOnDiskToInstanceTable,
    /// F9 坏介质：上一版数据单元那个槽两块盘都读写皆失败、永远。
    BadMediaReadAndWriteForever,
    /// F10 写者错（落盘）：池里建 240 个 inode 分成两片叶，注入那次发布把数据单元那条的槽号写成右边那片叶
    /// （覆盖写只重写最左那片，右边那片一直照抄、活着，跨度同为 2）。
    WriterBugOnDiskToCarriedLeaf,
}""")
rep("""    const ALL: [Fault; 7] = [""", """    const ALL: [Fault; 10] = [""")
rep("""        Fault::MappingPointsAtLeafContainerReleasedInTheSamePublish,
    ];""", """        Fault::MappingPointsAtLeafContainerReleasedInTheSamePublish,
        Fault::WriterBugOnDiskToInstanceTable,
        Fault::BadMediaReadAndWriteForever,
        Fault::WriterBugOnDiskToCarriedLeaf,
    ];""")
rep("""            Fault::MappingPointsAtLeafContainerReleasedInTheSamePublish => "F7-map-bug-leaf-same-publish",
        }""", """            Fault::MappingPointsAtLeafContainerReleasedInTheSamePublish => "F7-map-bug-leaf-same-publish",
            Fault::WriterBugOnDiskToInstanceTable => "F8-writer-bug-disk-instance-table",
            Fault::BadMediaReadAndWriteForever => "F9-bad-media-rw-forever",
            Fault::WriterBugOnDiskToCarriedLeaf => "F10-writer-bug-disk-carried-leaf",
        }""")
rep("""    Arm { name, arms: rc::Arms { enabled, on_read_failure: read, mirror_rule: mirror, scope, on_mismatch: mismatch, recompute_at_mount: recompute, rebuild_tolerates_unreadable_data: false }, persisted }""",
"""    Arm { name, arms: rc::Arms { enabled, on_read_failure: read, mirror_rule: mirror, scope, on_mismatch: mismatch, recompute_at_mount: recompute, rebuild_tolerates_unreadable_data: false, rebuild_refuses_on_mapping_mismatch: false }, persisted }""")
rep("""            same.arms.rebuild_tolerates_unreadable_data = true;
            same
        },
    ]""", """            same.arms.rebuild_tolerates_unreadable_data = true;
            same
        },
        {
            let mut refuse = arm("N2-rebuild-checks-mapping-refuses", true, A, Any, All, Iso, false, false);
            refuse.arms.rebuild_refuses_on_mapping_mismatch = true;
            refuse
        },
    ]""")
# Run 多两项：故障目标（复用检测用）与复用记录
rep("""    arm: Arm,
    switches: u64,
}""", """    arm: Arm,
    switches: u64,
    /// 故障目标：槽号区间与原主人（身份、槽、字节 CRC）；之后哪一步有别的单元落进这个区间就记一笔。
    target: Option<((u64, u64), (TransactionUnit, u64, u32))>,
}""")
rep("""            writable: true, panicked: false, seed: 1, persisted_isolation: Vec::new(), arm, switches: 0 };""",
"""            writable: true, panicked: false, seed: 1, persisted_isolation: Vec::new(), arm, switches: 0, target: None };""")
rep("""    fn start(tag: &str, arm: Arm) -> Run {
        rc::set_arms(rc::Arms::OFF);
        rc::reset_counters();""", """    fn start(tag: &str, arm: Arm, fault: Fault) -> Run {
        rc::set_arms(rc::Arms::OFF);
        rc::reset_counters();
        rc::WRITER_BUG_DATA_ENTRY_TO.with(|bug| *bug.borrow_mut() = None);
        rc::REBUILD_MAPPING_MISMATCH.with(|count| *count.borrow_mut() = 0);""")
rep("""        let o = run.overwrite();
        assert_eq!(o, "ok", "前缀 O");
        rc::reset_counters();
        run
    }""", """        let o = run.overwrite();
        assert_eq!(o, "ok", "前缀 O");
        if fault == Fault::WriterBugOnDiskToCarriedLeaf {
            // 前缀多两步：建 240 个 inode（第一片叶满 233 条在末尾分裂成两片），再覆盖写一次（只重写最左那片）。
            let previous = run.current.clone();
            let publish_parameters = parameters();
            let devices = run.devices.as_mut().expect("开着");
            let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
            run.current = publish_new_inodes(&mut writer, &mut run.allocator, &previous, 240, FIXED_WRITE_TIME_SECONDS + 30, previous.root.instance)
                .expect("建 240 个 inode");
            assert!(run.current.inode_leaf_containers.len() >= 2, "分裂成两片叶");
            let o = run.overwrite();
            assert_eq!(o, "ok", "前缀第二个 O");
        }
        rc::reset_counters();
        rc::REBUILD_MAPPING_MISMATCH.with(|count| *count.borrow_mut() = 0);
        run
    }

    fn note_target(&mut self, identity: TransactionUnit, slot: SlotNumber) {
        let owner = self.current.units.iter().find(|unit| unit.identity == identity && unit.slot == slot).expect("原主人在这一版里");
        self.target = Some(((slot.0, slot.0 + identity.span_slots()), (identity, slot.0, crc32_castagnoli(&owner.bytes))));
    }

    /// 这一版里有没有别的单元落进故障目标那个区间（原主人照抄不算）。
    fn foreign_unit_on_target(&self) -> bool {
        let Some(((start, end), owner)) = self.target else { return false };
        self.current.units.iter().any(|unit| {
            let unit_end = unit.slot.0 + unit.identity.span_slots();
            unit.slot.0 < end && start < unit_end && (unit.identity, unit.slot.0, crc32_castagnoli(&unit.bytes)) != owner
        })
    }""")
# inject：新故障与目标登记
rep("""    fn inject(&mut self, fault: Fault) {
        let data = self.current.data_pointer.locations;""", """    fn inject(&mut self, fault: Fault) {
        let data = self.current.data_pointer.locations;
        match fault {
            Fault::MappingPointsAtLiveInstanceTable | Fault::WriterBugOnDiskToInstanceTable => {
                let slot = self.current.root.instance_table.locations[0].slot;
                self.note_target(TransactionUnit::InstanceTable, slot);
            }
            Fault::MappingPointsAtLeafContainerReleasedInTheSamePublish => {
                let slot = self.current.inode_leaf_containers[0].pointer.locations[0].slot;
                self.note_target(TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::of_position(0)), slot);
            }
            Fault::WriterBugOnDiskToCarriedLeaf => {
                let slot = self.current.inode_leaf_containers[1].pointer.locations[0].slot;
                self.note_target(TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::of_position(1)), slot);
            }
            _ => self.note_target(TransactionUnit::Data, data[0].slot),
        }
        if matches!(fault, Fault::WriterBugOnDiskToInstanceTable | Fault::WriterBugOnDiskToCarriedLeaf) {
            let target = self.target.expect("刚登记").0 .0;
            rc::WRITER_BUG_DATA_ENTRY_TO.with(|bug| *bug.borrow_mut() = Some(SlotNumber(target)));
            let o = self.overwrite();
            assert_eq!(o, "ok", "注入那次发布");
            assert!(rc::WRITER_BUG_DATA_ENTRY_TO.with(|bug| bug.borrow().is_none()), "注入用掉了");
            rc::reset_counters();
            return;
        }
        if fault == Fault::BadMediaReadAndWriteForever {
            let (start, end) = (data[0].slot.0 * SLOT, (data[0].slot.0 + 2) * SLOT);
            for device in [DeviceIdentity(0), DeviceIdentity(1)] {
                self.plan.borrow_mut().faults.push(ReadFault { device, start, end, persistence: Persistence::Forever, active: true });
            }
            return;
        }""")
rep("""            Fault::MappingPointsAtLiveInstanceTable | Fault::MappingPointsAtLeafContainerReleasedInTheSamePublish => {
                let target =""", """            Fault::WriterBugOnDiskToInstanceTable | Fault::BadMediaReadAndWriteForever | Fault::WriterBugOnDiskToCarriedLeaf => unreachable!("上面处置过"),
            Fault::MappingPointsAtLiveInstanceTable | Fault::MappingPointsAtLeafContainerReleasedInTheSamePublish => {
                let target =""")
rep("""    let mut run = Run::start(&format!("newq{index}"), arm);
    run.inject(fault);
    let mut outcomes = Vec::new();
    let mut isolated_by_check_total = 0u64;
    for op in suffix.chars() {
        let outcome = run.step(op);""", """    let mut run = Run::start(&format!("newq{index}"), arm, fault);
    run.inject(fault);
    let mut outcomes = Vec::new();
    let mut reuse_steps = Vec::new();
    let mut isolated_by_check_total = 0u64;
    for (step_index, op) in suffix.chars().enumerate() {
        let mut outcome = run.step(op);
        if !run.panicked && run.foreign_unit_on_target() {
            reuse_steps.push(step_index);
            outcome.push_str("[reuse]");
        }""")
rep("""    let mut line = String::new();
    write!(line, "{}\\t{}\\t{}\\t{}\\t{}\\t{}\\t{}\\t{:?}\\t{}\\t{}\\t{:?}\\t{}\\t{}\\t{}\\t{}\\t{}",""",
"""    let rebuild_mapping_mismatch = rc::REBUILD_MAPPING_MISMATCH.with(|count| *count.borrow());
    let writes_refused = run.plan.borrow().writes_refused;
    let mut line = String::new();
    write!(line, "{}\\t{}\\t{}\\t{}\\t{}\\t{}\\t{}\\t{:?}\\t{}\\t{}\\t{:?}\\t{}\\t{}\\t{}\\t{}\\t{}\\t{:?}\\t{}\\t{}",""")
rep("""        if events.is_empty() { "-".to_string() } else { events.join(";") }).expect("写");""",
"""        if events.is_empty() { "-".to_string() } else { events.join(";") }, reuse_steps, rebuild_mapping_mismatch, writes_refused).expect("写");""")
rep("""\\tswitches\\treads_refused\\tcheck_events").unwrap();""", """\\tswitches\\treads_refused\\tcheck_events\\treuse_steps\\trebuild_mapping_mismatch\\twrites_refused").unwrap();""")
(here / "m2_newq_attack3.rs").write_text(src)
print("driver3 written")
