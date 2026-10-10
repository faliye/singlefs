# 攻方原型：给两份 checker 拷贝（b36-checker-probe = 今天的 R1，b36-checker-pre36 = 反打 36-1b 的 FL）加只记不判的探针。
# 探针只往线程局部变量里写，不改任何判定（原型每张镜像核一遍探针拷贝与 singlefs-checker 的判定逐条相同）。
import sys
path, flavor = sys.argv[1], sys.argv[2]
s = open(path).read()

def once(old, new):
    global s
    assert s.count(old) == 1, (old[:80], s.count(old))
    s = s.replace(old, new)

if flavor == "new":
    old = """    let version_the_next_mount_applies_first = version_the_next_mount_applies_first(
        reader,
        &journal_records_by_device,
        &roots[newest_index].2,
        geometry.journal_ring_bytes,
    );
"""
    once(old, old + """    PROBE.with(|p| {
        let mut p = p.borrow_mut();
        p.next_mount = version_the_next_mount_applies_first.as_ref().map(|v| (v.instance, v.checkpoint_txg, v.new_root_segment.clone()));
        p.newest = (roots[newest_index].2.instance, roots[newest_index].2.checkpoint_txg);
        let records_the_replay_reads = records_the_replay_reads(&journal_records_by_device);
        p.replay_kind = match replay_the_next_mount_would_do(
            reader,
            &records_the_replay_reads,
            &roots[newest_index].2,
            in_flight_record_limit_of_the_journal_ring(geometry.journal_ring_bytes),
        ) {
            ReplayTheNextMountWouldDo::AnchorUndecided => "anchor-undecided".to_string(),
            ReplayTheNextMountWouldDo::ChosenRootTxgHasNoSuccessor => "no-successor".to_string(),
            ReplayTheNextMountWouldDo::NothingApplied => "nothing".to_string(),
            ReplayTheNextMountWouldDo::StoppedAtANamedUnitThatDoesNotVerify { last_record_of_the_first_publish_applied_before_it } =>
                format!("stopped first={}", if last_record_of_the_first_publish_applied_before_it.is_some() { "some" } else { "none" }),
            ReplayTheNextMountWouldDo::Applied { publishes, .. } => format!("applied publishes={}", publishes.min(3)),
        };
    });
""")
else:
    old = """    let version_the_next_mount_applies_first =
        version_the_next_mount_applies_first(&journal_records_by_device, &roots, newest_index);
"""
    once(old, old + """    PROBE.with(|p| {
        let mut p = p.borrow_mut();
        p.next_mount = version_the_next_mount_applies_first.as_ref().map(|v| (v.instance, v.checkpoint_txg, v.new_root_segment.clone()));
        p.newest = (roots[newest_index].2.instance, roots[newest_index].2.checkpoint_txg);
    });
""")
old = """            let replayed_version = VersionAppliedOnlyByRecords {
                instance: last_record.instance,
                checkpoint_txg: last_record.checkpoint_txg,
                new_root_segment: last_record.new_root_segment.clone(),
            };
"""
once(old, old + """            PROBE.with(|p| p.borrow_mut().replayed = Some((last_record.instance, last_record.checkpoint_txg, last_record.new_root_segment.clone(), publishes)));
""")
s += """
/// 攻方原型的探针（只在这份拷贝里）：最近一次 check_pool_image 的最新根、认出的下一次挂载先施加的那一版（实例、txg、新根段）、
/// I-4.1 重放施加出来的那一版（实例、txg、新根段、施加几次发布），与（只 R1 那份）重放的结局种类。
#[derive(Default, Clone, Debug)]
pub struct Probe {
    pub newest: (u32, u64),
    pub next_mount: Option<(u32, u64, Vec<u8>)>,
    pub replayed: Option<(u32, u64, Vec<u8>, u64)>,
    pub replay_kind: String,
}
thread_local! {
    pub static PROBE: std::cell::RefCell<Probe> = std::cell::RefCell::new(Probe::default());
}
"""
open(path, "w").write(s)
