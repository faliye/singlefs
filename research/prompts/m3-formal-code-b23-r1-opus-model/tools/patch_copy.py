#!/usr/bin/env python3
"""在副本里打开关（m3-formal-code-b23-r1 opus）。不设环境变量时副本行为与冻结副本相同。
用法：patch_copy.py <副本根>。每处原文要恰好命中一次，否则退 3。"""
import sys
root = sys.argv[1]

def edit(path, pairs):
    p = f"{root}/{path}"
    s = open(p, encoding="utf-8").read()
    for old, new in pairs:
        n = s.count(old)
        if n != 1:
            print(f"{path}: 原文命中 {n} 次：{old[:80]!r}", file=sys.stderr)
            sys.exit(3)
        s = s.replace(old, new)
    open(p + ".opus-new", "w", encoding="utf-8").write(s)
    import os
    os.replace(p + ".opus-new", p)

CI = "crates/singlefs-checker-tier/src/crash_injection.rs"
edit(CI, [
    # K3 接线：OPUS_K3_WIRING=first 时第三截交第一次崩溃那一版（与 mutations.tsv 23-1 K3-a 那一行同义）
    ("                crash_image_materialized: &second_crash_image_materialized,\n                version_landed_on_after_the_crash: report.effective_root,",
     "                crash_image_materialized: &second_crash_image_materialized,\n                version_landed_on_after_the_crash: if std::env::var(\"OPUS_K3_WIRING\").as_deref() == Ok(\"first\") { fields_read_after_the_first_crash.report.effective_root } else { report.effective_root },"),
    # 第二截带文件而第一个文件不是活记录那一支：只在这一支的调用里打开 ARM_ACTIVE；OPUS_ARM=time-zero / count-two / count-zero 改调用点参数
    ("                PoolVersion::WithFile(current) => publish_new_inodes(\n                    &mut writer,\n                    &mut allocator,\n                    current,\n                    1,\n                    FIXED_WRITE_TIME_SECONDS,\n                    mounted.output.instance,\n                )\n                .map(|_| ()),",
     "                PoolVersion::WithFile(current) => { singlefs_core::transaction::opus_probe::ARM_ACTIVE.with(|c| c.set(true)); let opus_arm = std::env::var(\"OPUS_ARM\").unwrap_or_default(); let opus_result = publish_new_inodes(\n                    &mut writer,\n                    &mut allocator,\n                    current,\n                    match opus_arm.as_str() { \"count-zero\" => 0, \"count-two\" => 2, _ => 1 },\n                    if opus_arm == \"time-zero\" { 0 } else { FIXED_WRITE_TIME_SECONDS },\n                    if opus_arm == \"old-instance\" { InstanceGeneration(mounted.output.instance.0 - 1) } else { mounted.output.instance },\n                )\n                .map(|_| ()); singlefs_core::transaction::opus_probe::ARM_ACTIVE.with(|c| c.set(false)); opus_result }"),
    # 判法只看 Some（与 mutations.tsv 改法 b 那一行同义）
    ("fn first_file_is_a_live_record(version: &TransactionOutput) -> bool {\n",
     "fn first_file_is_a_live_record(version: &TransactionOutput) -> bool {\n    if std::env::var(\"OPUS_K2B_SOME\").is_ok() {\n        return version.inode_record.is_some();\n    }\n"),
])
s = open(f"{root}/{CI}", encoding="utf-8").read()
if "pub mod opus_probe;" not in s:
    open(f"{root}/{CI}", "a", encoding="utf-8").write("\n#[path = \"opus_probe.rs\"]\npub mod opus_probe;\n")

TX = "crates/singlefs-core/src/transaction.rs"
edit(TX, [
    ("    let txg = checkpoint_txg_of_the_publish_after(previous.root.checkpoint_txg)?;\n    let first_new_inode_number = previous.inode_number_watermark();",
     "    let txg = checkpoint_txg_of_the_publish_after(previous.root.checkpoint_txg)?;\n    let opus_arm_variant = opus_probe::variant();\n    let opus_core_variant = std::env::var(\"OPUS_CORE_MUT\").ok();\n    let txg = if opus_arm_variant.as_deref() == Some(\"txg-skip\") { CheckpointTxg(txg.0 + 1) } else { txg };\n    let first_new_inode_number = previous.inode_number_watermark();"),
    ("            change_count,\n            write_time_seconds,\n        })\n        .collect();\n    publish_version(\n        pool,\n        allocator,\n        PublishPlan {\n            txg,\n            counter: previous.record.counter + 1,\n            transaction: transaction_number,",
     "            change_count,\n            write_time_seconds,\n        })\n        .map(|mut record| {\n            // 23-1 那三行 core 变异（每个调用都生效）\n            match opus_core_variant.as_deref() {\n                Some(\"nlink\") => record.nlink = NLINK_OF_AN_INODE_BEING_DELETED,\n                Some(\"birth\") => record.object_birth = previous.root.checkpoint_txg,\n                Some(\"size\") => record.size = 4096,\n                _ => {}\n            }\n            // 只在第二截那一支的调用里生效的坏发布\n            match opus_arm_variant.as_deref() {\n                Some(\"being-deleted\") => record.nlink = NLINK_OF_AN_INODE_BEING_DELETED,\n                Some(\"old-birth\") => record.object_birth = previous.root.checkpoint_txg,\n                Some(\"size\") => record.size = 4096,\n                Some(\"change-previous\") => record.change_count = previous.root.checkpoint_txg.0,\n                Some(\"birth-next\") => record.object_birth = CheckpointTxg(txg.0 + 1),\n                _ => {}\n            }\n            record\n        })\n        .collect();\n    publish_version(\n        pool,\n        allocator,\n        PublishPlan {\n            txg,\n            counter: previous.record.counter + 1,\n            transaction: transaction_number,"),
])
s = open(f"{root}/{TX}", encoding="utf-8").read()
if "pub mod opus_probe" not in s:
    open(f"{root}/{TX}", "a", encoding="utf-8").write('''
/// 攻方副本探针用（m3-formal-code-b23-r1 opus）：只在崩溃注入第二截那一支的调用里生效。
pub mod opus_probe {
    use std::cell::Cell;
    thread_local! {
        pub static ARM_ACTIVE: Cell<bool> = const { Cell::new(false) };
    }
    #[must_use]
    pub fn variant() -> Option<String> {
        if ARM_ACTIVE.with(Cell::get) {
            std::env::var("OPUS_K2A_VARIANT").ok()
        } else {
            None
        }
    }
}
''')
print("patched")
