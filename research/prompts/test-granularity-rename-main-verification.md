# 用例粒度与按内容起名：判决（2026-09-27）

这一批（提交 bc57af7a）**没有派三方的腿**：用户 2026-09-27 JST 00:5x 定「要么你先把你改动了都提交了」「提交了之后再慢慢验证」。下面逐个点名这一批改过的 `crates/` 源文件与定义，写明每一类改的是什么；被攻过零轮。

## 按路径点名（门禁 56、72 号）

改名（标识符里的 `first_transaction` 一族换成 `new_pool_file_creation`、`second_transaction` 一族换成 `file_overwrite`，文件与装置按内容改名，不改行为）、
harness 用例拆成一场景一条与重档标记、checker 档测试文件第一行的模块声明，落在这 40 个文件：

- `crates/singlefs-checker/src/walk.rs`
- `crates/singlefs-checker-tier/src/bad_disk_input.rs`
- `crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump_one_device.rs`
- `crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs`
- `crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs`
- `crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs`
- `crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs`
- `crates/singlefs-checker-tier/src/bin/new_pool_file_creation_device_log_check.rs`
- `crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs`
- `crates/singlefs-checker-tier/src/bin/new_pool_file_creation_region_bytes.rs`
- `crates/singlefs-checker-tier/src/crash_injection.rs`
- `crates/singlefs-checker-tier/src/crash.rs`
- `crates/singlefs-checker-tier/src/on_device_modes.rs`
- `crates/singlefs-core/src/address.rs`
- `crates/singlefs-core/src/admission.rs`
- `crates/singlefs-core/src/allocator.rs`
- `crates/singlefs-core/src/inode_tree.rs`
- `crates/singlefs-core/src/journal.rs`
- `crates/singlefs-core/src/make_filesystem.rs`
- `crates/singlefs-core/src/mounted_read.rs`
- `crates/singlefs-core/src/mount.rs`
- `crates/singlefs-core/src/records.rs`
- `crates/singlefs-core/src/recovery.rs`
- `crates/singlefs-core/src/root_ring.rs`
- `crates/singlefs-core/src/transaction.rs`
- `crates/singlefs-core/src/write_accounting.rs`
- `crates/singlefs-core/src/write_request_split.rs`
- `crates/singlefs-format/src/lib.rs`
- `crates/singlefs-harness/src/fault_injection.rs`
- `crates/singlefs-harness/src/hexadecimal.rs`
- `crates/singlefs-harness/src/history.rs`
- `crates/singlefs-harness/src/lib.rs`
- `crates/singlefs-harness/src/memory_pool.rs`
- `crates/singlefs-harness/src/model_comparison.rs`
- `crates/singlefs-harness/src/model.rs`
- `crates/singlefs-harness/src/new_pool_file_creation_regions.rs`
- `crates/singlefs-harness/src/read_tally.rs`
- `crates/singlefs-harness/src/scenario.rs`
- `crates/singlefs-harness/src/segments.rs`
- `crates/singlefs-harness/src/sha256.rs`

定义：`.claude/main-agent.md`（合入后验证改成跑 harness 档的测试二进制、红了只重跑红的那几条）、`.claude/agents/crash-verifier.md`（改名跟着换的装置名与阶段名）、`.claude/agents/implementation-writer.md`（第 7a 条认自己逐个造崩溃状态的写法、开工先读加 `.claude/rules/verification.md` 三节、第 1c 条红了只重跑红的那几条）、`.claude/agents/kb-scribe.md`（改名跟着换的文档与术语）。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 15 | 改了 | 2026-09-27 改了：harness 档的轻重由单线程耗时表定、一条用例一个场景、测试文件按测什么起名（变更史 2026-09 其二十三） |
