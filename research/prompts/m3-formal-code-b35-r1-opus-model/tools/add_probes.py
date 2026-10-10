# 攻方原型：给两份 checker 拷贝（b35-checker-pre35、b35-checker-probe）加只记不判的探针。
import sys
path, flavor = sys.argv[1], sys.argv[2]
s = open(path).read()

def once(old, new):
    global s
    assert s.count(old) == 1, (old[:80], s.count(old))
    s = s.replace(old, new)

old = """    let versions_applied_only_by_records = versions_applied_only_by_records(
        &journal_records_by_device,
        &roots,
        &instance_table_rows,
        effective_rollback_floor,
    );
"""
once(old, old + """    PROBE.with(|p| p.borrow_mut().versions = versions_applied_only_by_records.iter().map(|v| (v.instance, v.checkpoint_txg, v.new_root_segment.clone())).collect());
""")
old = """    let version_the_next_mount_applies_first =
        version_the_next_mount_applies_first(&journal_records_by_device, &roots, newest_index);
"""
once(old, old + """    PROBE.with(|p| p.borrow_mut().next_mount = version_the_next_mount_applies_first.as_ref().map(|v| (v.instance, v.checkpoint_txg, v.new_root_segment.clone())));
""")
old = """            let replayed_version = VersionAppliedOnlyByRecords {
                instance: last_record.instance,
                checkpoint_txg: last_record.checkpoint_txg,
                new_root_segment: last_record.new_root_segment.clone(),
            };
"""
once(old, old + """            PROBE.with(|p| p.borrow_mut().replayed = Some((last_record.instance, last_record.checkpoint_txg, last_record.new_root_segment.clone(), publishes)));
""")
if flavor == "probe":
    old = """    versions
        .into_iter()
        .map(
            |((instance, checkpoint_txg), taken)| VersionAppliedOnlyByRecords {"""
    once(old, """    PROBE.with(|p| p.borrow_mut().from_last = versions.iter().map(|(k, t)| (*k, matches!(t, SegmentTakenForTheVersion::FromTheLastRecordOfThePublish(_)))).collect());
""" + old)
s += """
/// 攻方原型的探针（只在这份拷贝里）：最近一次 check_pool_image 认出的由记录施加出来的版本（实例、txg、新根段）、
/// 下一次挂载先施加的那一版、I-4.1 重放施加出来的那一版（实例、txg、新根段、施加几次发布），与每一版的新根段是不是取自末条。
#[derive(Default, Clone, Debug)]
pub struct Probe {
    pub versions: Vec<(u32, u64, Vec<u8>)>,
    pub from_last: Vec<((u32, u64), bool)>,
    pub next_mount: Option<(u32, u64, Vec<u8>)>,
    pub replayed: Option<(u32, u64, Vec<u8>, u64)>,
}
thread_local! {
    pub static PROBE: std::cell::RefCell<Probe> = std::cell::RefCell::new(Probe::default());
}
"""
open(path, "w").write(s)
