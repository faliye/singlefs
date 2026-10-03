#!/usr/bin/env python3
"""给 E165 装置的副本打两处只加计数、不改判据的补丁：
S6：judge_the_fixed_point 里按 (前提真假, 第一次是否落在 u64 最大值, 第二次的结局种类) 数；每个域的 verdict 行之后打一行 S6SPLIT 并清零。
S1：环境变量 S1_MARKER=1 时时间线上每一版的根带卸载记号；在 judge_a_legal_state 里数「施加了至少一条的态」里返回的根与所选根在非新根段字段上的差。
"""
import sys, re, pathlib
path = pathlib.Path(sys.argv[1])
s = path.read_text()

def once(old, new):
    global s
    assert s.count(old) == 1, (s.count(old), old[:80])
    s = s.replace(old, new)

# 计数器与打印
once("use std::sync::{Mutex, OnceLock};\n",
"""use std::sync::{Mutex, OnceLock};

// ── 辩方腿加的计数（只加计数，不改任何判据）
static S6_COUNTS: [AtomicU64; 32] = [const { AtomicU64::new(0) }; 32];
static S1_COUNTS: [AtomicU64; 8] = [const { AtomicU64::new(0) }; 8];

fn s6_second_kind(second: &ReplayOutcome, first_root: &RootRecord) -> usize {
    match second {
        ReplayOutcome::Returned(returned) if returned.root == *first_root => 0,
        ReplayOutcome::Returned(_) => 1,
        ReplayOutcome::Failed(FailureKind::ChosenRootTxgHasNoSuccessor) => 2,
        ReplayOutcome::Failed(FailureKind::RootPublishCarriesMoreThanOneLastRecordFlag) => 3,
        ReplayOutcome::Failed(FailureKind::Other) => 4,
        ReplayOutcome::Panicked => 5,
    }
}

fn s6_dump_and_reset(domain_text: &str) {
    let labels = ["equal", "returned_different", "failed_no_successor", "failed_more_than_one_last_flag", "failed_other", "panicked"];
    for premise in 0..2usize {
        for top in 0..2usize {
            for (kind, label) in labels.iter().enumerate() {
                let index = (premise * 2 + top) * 6 + kind;
                let value = S6_COUNTS[index].swap(0, Ordering::SeqCst);
                if value != 0 {
                    println!("S6SPLIT {domain_text} premise_holds={} first_landed_at_u64_max={} second={} states={value}", premise == 1, top == 1, label);
                }
            }
        }
    }
    let names = ["applied_ge1", "marker_differs", "fsid_differs", "instance_table_differs", "alloc_root_differs", "segment_or_ids_equal_chosen"];
    for (index, name) in names.iter().enumerate() {
        let value = S1_COUNTS[index].swap(0, Ordering::SeqCst);
        if value != 0 {
            println!("S1SPLIT {domain_text} {name}={value}");
        }
    }
}
""")

# judge_the_fixed_point: 在 `if fixed_point_holds {` 之前数
once("""    let fixed_point_holds = matches!(&second, ReplayOutcome::Returned(second_returned) if second_returned.root == first.root);
""",
"""    let fixed_point_holds = matches!(&second, ReplayOutcome::Returned(second_returned) if second_returned.root == first.root);
    if matches!(implementation, Implementation::TodayReplayJournal) {
        let top = usize::from(first.root.checkpoint_txg.0 == u64::MAX);
        let index = (usize::from(premise_holds) * 2 + top) * 6 + s6_second_kind(&second, &first.root);
        S6_COUNTS[index].fetch_add(1, Ordering::SeqCst);
    }
""")

# verdict 行之后打印
once("""    fn result(&mut self, text: &str) {
        println!("E7RESULT {text}");
        self.emitted += 1;
    }
""",
"""    fn result(&mut self, text: &str) {
        println!("E7RESULT {text}");
        self.emitted += 1;
        if text.starts_with("name=verdict domain=") {
            let domain_text: String = text.split_whitespace().take(2).collect::<Vec<_>>().join(" ");
            s6_dump_and_reset(&domain_text);
        }
    }
""")

# S1：to_root 的卸载记号
once("""            unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
            instance: InstanceGeneration(self.instance),""",
"""            unmount_marker: if env::var("S1_MARKER").is_ok() { UnmountMarker::WrittenByTheUnmountSequence } else { UnmountMarker::NotWrittenByTheUnmountSequence },
            instance: InstanceGeneration(self.instance),""")

# S1：judge_a_legal_state 里数
once("""    tally.add(Counter::States);
    if chosen_version > 0 {
        tally.add(Counter::ChosenRootIsALaterVersion);
    }
""",
"""    tally.add(Counter::States);
    if matches!(implementation, Implementation::TodayReplayJournal) {
        if let ReplayOutcome::Returned(returned) = &first {
            if returned.prefix_applied >= 1 {
                S1_COUNTS[0].fetch_add(1, Ordering::SeqCst);
                let r = &returned.root;
                let c = &chosen_root;
                if r.unmount_marker != c.unmount_marker { S1_COUNTS[1].fetch_add(1, Ordering::SeqCst); }
                if r.filesystem_identifier != c.filesystem_identifier { S1_COUNTS[2].fetch_add(1, Ordering::SeqCst); }
                if r.instance_table != c.instance_table { S1_COUNTS[3].fetch_add(1, Ordering::SeqCst); }
                if r.allocation_record_tree_root != c.allocation_record_tree_root { S1_COUNTS[4].fetch_add(1, Ordering::SeqCst); }
                if r.unmount_marker == c.unmount_marker && r.filesystem_identifier == c.filesystem_identifier && r.instance_table == c.instance_table && r.allocation_record_tree_root == c.allocation_record_tree_root { S1_COUNTS[5].fetch_add(1, Ordering::SeqCst); }
            }
        }
    }
    if chosen_version > 0 {
        tally.add(Counter::ChosenRootIsALaterVersion);
    }
""")
path.write_text(s)
print("patched", path)
