# 实分一交回：层 0 崩溃重放按双机分片（里程碑三第六项）

日期：2026-09-27。实现员，线程上限 8，内存包装 8G（replay.sh 默认）。

## 结论（出口 1–5 各到哪）

| 出口 | 状态 | 依据 |
|---|---|---|
| 1 分片开关与 merge | crates 侧做完 | `SINGLEFS_LAYER0_SHARD=<i>/<n>` 只跑 `slice_index % n == i` 的切片、写账本；`merge/<n>` 读 n 份、核齐、按切片序号并。合并之后用例的计数行与不分片逐字相同：新测试文件 `two_and_three_shards_…` 比 `count_line`，驱动脚本自证 ① 比 LAYER0_SHARDED 行 |
| 2 不带 layer0 的测试文件 | 做完 | `crates/singlefs-harness/tests/crash_enumeration_sharded_across_processes.rs`，10 条全绿，每条都证过红 |
| 3 驱动脚本 | 成品在 deliver/，没进仓（写范围外） | `--selftest` 在草稿目录的仓拷贝上 10 格全过；非 `--selftest` 那条路没跑（不许） |
| 4 变异表 | 做完 | `crates/mutations.tsv` 末尾追加 13 行，13 行全部由我证过红（见「变异证红」） |
| 5 kb | 成品在 deliver/，没进仓（写范围外） | `.claude/kb/milestone/03-third-txn.md` 第六节的 diff |
| 默认不分片（2026-09-27 加） | 做完 | golden 用例 `unsharded_enumeration_prints_and_writes_byte_for_byte_what_it_did_before_sharding`：不设开关时打印行（38 行，时刻与目录抹掉）与进度文件（名字连计划哈希、30 行逐字节）和加分片之前那一版代码跑出来的逐项相同 |

什么现象会推翻「合并之后与单机逐字相同」：真的两台上 merge 那一趟的 LAYER0 / LAYER0B / CHECKER 行与单机 `--full` 记进标记的那几行对不上；或者换一条流（小流只罩了第一条流甲二 29 个状态）时 `two_and_three_shards_…` 的比对红。

## 这一轮写过的文件（crates 侧，直接改主工作区）

- `crates/singlefs-harness/src/layer0_progress.rs`（未跟踪文件）：分片开关 `LAYER0_SHARD_ENVIRONMENT_VARIABLE`（第 42 行）、`Layer0ShardOfShards`、`Layer0ToolchainIdentity::of_the_cargo_running_this_test`（第 194 行）、`Layer0ShardRun` / `Layer0ShardMerge`、`Layer0Resume` 多两个成员、`shard_switch_from_text`（第 292 行）、`Layer0ProgressPlan.shard`、进度文件名与文件头带 `shard-<i>-of-<n>`（不分片时不带，与改前逐字相同）、片行逐行核抽成 `restored_slice_lines`（第 856 行，原循环逐字保留，另加「不归这一片」一判）、账本 `shard_ledger_path` / `write_shard_ledger` / `read_shard_ledgers_for_merge`（第 1078、1184、1441 行）；库内用例改两处调用、加两条。
- `crates/singlefs-harness/src/crash.rs`：只动了这几段（与开工时 HEAD + 那一刻工作区的差，`diff -U0` 的块头）：第 27–30 行 `use crate::layer0_progress` 那一块；第 1874–1912 行 `slicing_of_the_run` 与新的 `SlicingRule`；第 2100–2131 行 `layer0_plan_hash`（多一个 `shard` 参数，不分片时消息不变）；第 2167–2645 行枚举本体（`enumerate_layer0_in_state_slices` 变成外壳、新 `enumerate_layer0_in_state_slices_or_one_shard`、`Layer0EnumerationOutcome`、`Layer0ShardLedgerWritten`、`MergedSlicesOfThisRun`、`merge_the_shard_ledgers`）；第 2777 行起的 `mod tests` 加一条切法用例 `a_shard_run_and_a_merge_slice_independently_of_the_worker_threads`。记录核对器（`check_records_against` 附近，第 630–930 行）一行没碰。开工前 `git diff --stat` 记在 `/tmp/claude-1000/impl-shard-1/crash-diffstat-before.txt`。
- `crates/singlefs-harness/tests/crash_enumeration_sharded_across_processes.rs`（新，未跟踪）。
- `crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs`：全量用例与共用的 `enumerate_counting_allocation_generation_read_sets` 改调 `_or_one_shard`，分片跑一片时交回 `None`、用例收工；快档那一条 `.expect`。
- `crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs`：全量用例改调 `_or_one_shard`，分片跑一片时收工。
- `crates/mutations.tsv`：末尾追加 13 行（insert-row.py，逐行 `--absent` 挡重复），变异名：
  1. 实分一 双机分片：merge 按切片序号倒着并（「第一处」取了序号最大的那一片的）
  2. 实分一 双机分片：merge 缺账本不报（缺的那一片的切片悄悄不并）
  3. 实分一 双机分片：merge 不核账本文件头（输入指纹不同的账本照并）
  4. 实分一 双机分片：merge 不核账本文件头（切片方案不同的账本报不出是切法那一处）
  5. 实分一 双机分片：merge 不核账本第二行记的是第几片（两份账本同一片报不出）
  6. 实分一 双机分片：账本文件头不记 rustc -Vv（工具链不同的账本照并）
  7. 实分一 双机分片：计划哈希不带片（分片的进度文件与单机的同一个哈希）
  8. 实分一 双机分片：甲二少算单元写全落那一半（驱动脚本自证那条小流钉死的 29 个状态要红）
  9. 实分一 默认不分片：不设分片开关也在 LAYER0_PARALLEL_* 行末打 shard=（与加分片之前的打印行不再逐字相同）
  10. 实分一 双机分片：分片开关不认 merge/<n>
  11. 实分一 双机分片：分片跑的进度文件收不归这一片的切片
  12. 实分一 双机分片：分片跑与 merge 仍按线程数定片长（两台线程数不同，切法对不上）
  13. 实分一 默认不分片：留给测试看的进度文件跑完也删掉（golden 比对取不到进度文件）

`git diff --stat -- crates litmus` 原样在 `/tmp/claude-1000/impl-shard-1/diffstat-final.txt`（83 行，末行 ` 82 files changed, 19732 insertions(+), 10253 deletions(-)`；其中 `crates/singlefs-harness/src/crash.rs | 1496 +++-`、`crates/mutations.tsv | 507 +-`）；它不含未跟踪的 `layer0_progress.rs` 与新测试文件，且混着别的会话的改动，分不出谁改的，以上面的清单为准。

## 成品（写范围外，放在 `/tmp/claude-1000/impl-shard-1/deliver/`，保持仓内相对路径）

| deliver/ 里的 | 放到仓里 | 怎么放 |
|---|---|---|
| `research/scripts/layer0-shard-run.sh` | 同路径（新） | 拷过去，权限 775 |
| `research/scripts/layer0-shard-run-selftest.sh` | 同路径（新） | 拷过去，775；`layer0-shard-run.sh --selftest` 转给它 |
| `research/scripts/layer0-shard-configuration-check.sh` | 同路径（新） | 拷过去，775；54 号与驱动脚本的运行条件共用这一份判法 |
| `multi-host.env.example` | 仓根（新） | 拷过去；7 个键，占位值，不写主机名 |
| `research/scripts/admission.py.diff`（另附改完的整份） | `research/scripts/admission.py` | `patch -p1`（对今天主工作区的那一份 dry-run 过，能打上） |
| `.claude/gate.d/54-layer0-replay.sh.diff`（另附整份） | `.claude/gate.d/54-layer0-replay.sh` | 同上 |
| `.claude/gate.d/stage-inputs.tsv.diff`（另附整份） | `.claude/gate.d/stage-inputs.tsv` | 同上；两条流的全量行第三列加 `shard=across-machines`（会改这两条用例的输入指纹，它们的全绿标记本来就因 crates 改了而失效） |
| `.claude/hooks/lib_heavy_tests.py.diff`（另附整份） | `.claude/hooks/lib_heavy_tests.py` | 同上；`KNOWN_SCRIPT_LOCATIONS` 加驱动脚本，`classify` 认它是 `crash-case-cargo`（`--selftest` 不算） |
| `.claude/kb/milestone/03-third-txn.md.diff`（另附整份） | `.claude/kb/milestone/03-third-txn.md` | 同上；第六节「仓里已有的」改成现状、「开工前要先定的」换成「定案」五条、历史版本加 2026-09-27 一条 |

sha256（`find . -type f | sort | xargs sha256sum`，deliver/ 里现算）：
```
e7aaa247f421414272da9edba6115fc62d02a9dc84e5a922be8a05f98b30800d  ./.claude/gate.d/54-layer0-replay.sh
d29c2ec18e34fbecd3cb12827d8fc13e41c8579e375576a4f7e2d6bbd389c8a8  ./.claude/gate.d/54-layer0-replay.sh.diff
f0a04099b2e948f7f14e145c190dd6eecf757bafe829c848d804bcc58e534d84  ./.claude/gate.d/stage-inputs.tsv
c61d0b48c2cf3cc608b29781fb06ee6e604274acf719122ea40ff2fd94b3458a  ./.claude/gate.d/stage-inputs.tsv.diff
6973c12f19f207fb49950bd171e181131141e719ebb3229c1c79964900fd16a0  ./.claude/hooks/lib_heavy_tests.py
bbd837ff23b0120a81c47246de78b964cb7c7844c819b54207a819fed8471c8f  ./.claude/hooks/lib_heavy_tests.py.diff
830c0e8d853684d39ec960158c0a49634ab0323594dc2f4751b148e1e40d9d33  ./.claude/kb/milestone/03-third-txn.md
98cf3000dd278c0115372dd58e918c63bd8b705ee0962dde6b586aa4718919ab  ./.claude/kb/milestone/03-third-txn.md.diff
72ae03d4a01a49ccc069dc27c62a03e31d42dc420e52a879fa67bd4c2c458bd4  ./multi-host.env.example
dea4f615ccd26c484641e2950d61243fe4496e7d5a5bd8a5607b8a12f369af96  ./research/scripts/admission.py
71a3a162127988538c0fe656bb64579f26cd6f701235b2cecc704a9d111ed028  ./research/scripts/admission.py.diff
18862ecdc45f37685b2f9231169184a5c87f16c4b6185bddf7b9193aa97a4096  ./research/scripts/layer0-shard-configuration-check.sh
8d8684134687ae674ef953c2899cd3fe8dd1b5d389b9701cf35e1f64c7c31114  ./research/scripts/layer0-shard-run-selftest.sh
cce3a0434fab12ba0c2a27a06d4c83474c8b7b34f2c50ad44b508a21dd48f855  ./research/scripts/layer0-shard-run.sh
```

放进仓之后要跑的自证（主 agent）：`python3 research/scripts/admission.py --selftest`（我在草稿拷贝上 181 格全过）、`python3 .claude/hooks/lib_heavy_tests.py --selftest`（113 种全过）、`bash research/scripts/layer0-shard-run.sh --selftest`（10 格全过，约 5 分钟、临时目录 1.1G、跑完删）、`bash .claude/gate.d/73-research-gate-lint.sh`（草稿拷贝上全绿）。

### 驱动脚本怎么走（与规格的步骤对照）

- `layer0-shard-run.sh <crash-case:键> <树根>`：① 两台各 `rustc -Vv` 前三行与 host 行、`cargo -V` 比，不同就拒；各取 `nproc`。② 树 `rsync -a`（不带 `--delete`，排除 `target`、`.git`）到第二台 `<PEER_REPOSITORY_DIRECTORY>/runs/<用例>-<时刻>-<pid>/`（新建的空目录），在那里 `git init`（`crash-case-manifest` 要 git 列文件）。③ 两台各跑 `admission.py crash-case-manifest`（参数与 54 号 `--full` 逐字相同：`--extra-file` 54 号与准入模块、`--toolchain --build-environment`），指纹不同就拒；清场（配置写了才做）并回读，复原挂在 `trap … EXIT`。④ 本机 `SINGLEFS_LAYER0_SHARD=0/2`、第二台 `1/2` 同时跑（各记 pid、各 `wait "$pid"`、各写日志、`LAYER0_PROGRESS` 边跑边转出来），两片日志里都要有 `LAYER0_SHARD mode=run shard=<i>/2`。⑤ 第二台的账本 rsync 回本机进度目录，两片账本要各恰好一份。⑥ 本机 `merge/2`，日志交 `crash-case-judge`，跑完再算一次指纹、相同才 `crash-case-record`（与 54 号 `--full` 同一组标记）。⑦ 删第二台这一趟的 runs/ 子目录（连编译目录，删前 `du -sh`），两片的进度目录留着（被杀之后下一趟接着跑）。跑过之后判红删这批输入那一格标记（与 54 号同）。
- `--merged-log <键> <树根> <指纹> <日志>`：给 54 号用，只做 ①–⑥ 的跑，merge 那一趟整段输出写进日志，判与写标记归 54 号。
- 进度目录：本机 `<git common-dir>/singlefs-layer0-progress/<指纹>`（与 54 号相同），第二台 `<PEER_REPOSITORY_DIRECTORY>/progress/<指纹>`。
- 54 号 `--full`：开跑时 `bash research/scripts/layer0-shard-configuration-check.sh <根>` 判得过就打「双机分片：开（…）」，否则打「双机分片：关（原因）」；开着时登记了 `shard=across-machines` 的用例经 `--merged-log` 跑，别的单机跑；日志照单机的判法判、写同一格。

## 变异证红（每条新测试一行能让它红的变异；在草稿仓副本 `red-copy` 上跑，自己的 target，基线先跑一遍）

基线红集：空（`red-baseline-sharded.log`：`test result: ok. 9 passed; 0 failed; 1 ignored`——那时 golden 子进程用例还标着 ignore；`red-baseline-lib.log`：`test result: ok. 81 passed; 0 failed`）。每条跑完从主工作区拷回原文件并 `touch`。13 行变异都由我证过红，没有留给 59 号「只追加没证」的行。

| 变异（改坏哪一行） | 证的是哪条新测试 | 红在哪条断言 | 同时红了 |
|---|---|---|---|
| m01 `crash.rs` `for (_slice_index, slice_tally) in ledgers.slice_tallies {` → `.into_iter().rev()` | `two_and_three_shards_merge_into_the_same_tally_and_count_line_as_one_unsharded_run` | 测试文件第 386 行 `assert_eq!(merged, unsharded, "分 2 片再 merge：计数、观察者计数与每一处「第一处」都与不分片逐项相同")` | 无 |
| m02 `layer0_progress.rs` `if !missing.is_empty() {` → `if false {` | `merge_stops_and_names_the_shard_whose_ledger_is_missing` | merge 落到 `read_shard_ledgers_for_merge` 末尾的 `assert_eq!`（「n 份账本各自恰好是归它那一片的切片…」），用例第 414 行找不到「缺 1 份账本」 | 无 |
| m03 `layer0_progress.rs` `if header_body != expected_header_body {` → `if false {` | `merge_stops_when_a_ledger_was_run_on_another_input_fingerprint`（也证 `…_sliced_another_way`） | 指纹那条：`panic_message_of_merge` 的 `expect_err("merge 核不齐要 panic")`（merge 照并了）；切法那条：报成「记了 7 个切片归它」，用例第 512 行找不到 `slices 账本写的` | `…_built_by_another_toolchain` |
| m04 `layer0_progress.rs` `if written_shard_index != shard.shard_index() {` → `if false {` | `merge_stops_when_two_ledgers_are_the_same_shard` | 报成「记了 15 个切片归它」，用例第 479 行找不到「记的是第 0 片」 | 无 |
| m06 `layer0_progress.rs` `hexadecimal_text(toolchain.rustc_version_lines.as_bytes()),` → `hexadecimal_text(b""),` | `merge_stops_when_a_ledger_was_built_by_another_toolchain` | `expect_err("merge 核不齐要 panic")`（merge 照并了） | 无 |
| m07 `crash.rs` `if let Some(shard) = shard {` → `shard.filter(\|_\| false)` | `a_shard_cut_short_resumes_from_its_own_progress_file_to_the_same_ledger` | 第 620 行 `assert_ne!(…, "分片的计划哈希带着这一片，与单机的不同")`，两边都是 `a1d502a7…` | 无 |
| m08 `crash.rs` `(1u64 << (in_place_writes.len() + 1)) - 1` → `(1u64 << in_place_writes.len()) - 1` | `the_first_stream_quick_tier_sharded_by_the_environment_keeps_its_pinned_counts` | 第 692 行 `assert_eq!(tally.states, QUICK_TIER_STATES)`（25 ≠ 29） | `two_and_three_shards_…`、`…_sliced_another_way`、golden |
| m09 `crash.rs` `let shard_fields = shard.map_or_else(String::new, …` → 不分片也打 ` shard=none` | `unsharded_enumeration_prints_and_writes_byte_for_byte_what_it_did_before_sharding` | 第 837 行（行号是那一版的）打印行的 SHA-256 与 golden 不同，消息里第一行带 `shard=none` | 无 |
| m10 `layer0_progress.rs` `if first == "merge" {` → `"merged"` | 库内 `the_shard_switch_decides_one_shard_or_merge_and_misconfiguration_stops` | `from_environment_values` panic「SINGLEFS_LAYER0_SHARD 解不开：不是十进制数："merge"」 | 无 |
| m11 `layer0_progress.rs` `if !shard.owns_slice(slice_index) {` → `if false {` | 库内 `a_shard_progress_file_is_named_after_its_shard_and_holds_only_its_slices` | `expect_err("第 1 个切片不归第 0/2 片")` | 无 |
| m12 `crash.rs` `\| Layer0Resume::MergeShardLedgers(_) => SlicingRule::IndependentOfWorkerThreads,` → `AsGiven` | 库内 `a_shard_run_and_a_merge_slice_independently_of_the_worker_threads` | 那条用例的 `assert!(slices_on(resume, worker_threads) == single_machine_slices, …)` | 库内 `a_resumable_run_slices_independently_of_the_worker_threads`（替换把续跑也换成 AsGiven） |
| m13 `layer0_progress.rs` `KeptForTheTestThatInspectsIt => {}` → 也删文件 | `print_the_unsharded_enumerations_for_the_golden_comparison` | 第 785 行 `expect("留下恰好一个进度文件")`（读到 `[]`） | golden、`a_shard_cut_short_…` |

表里的测试文件行号是证红那一版的（之后 golden 子进程用例改成不标 ignore，行号往后挪了）。记录：`research/prompts/impl-shard-1-tmp-evidence/red-batch-1.log`（m01–m12 汇总行）、`red-m*.log`（逐条整段）。m03 同一行变异在表里登记了两行（指纹那条与切法那条各为必须红的测试）。

草稿之外的几样也各证过一次会红：
- `admission.py` 新判法：弄坏开关 `threads-ignore-shards` 那一格（「第 1 片只起 1 个线程跑了 32 片」在只看合计时判绿）写进了自证；54 号分片那两格：把草稿拷贝里 54 号的 `layer0_sharded=1` 改成 `0` 再跑自证，`✗ 54 号 --full 分片开着：…驱动脚本被调 []` 与 `✗ 54 号 --full 分片开着：驱动脚本退非 0 …` 两格红（`admission-selftest-red54.log`：`✗ admission.py 自证没过：2 格判错（共 181 格）`），改回之后 181 格全过。
- `lib_heavy_tests.py`：把 `classify` 里认驱动脚本的那一支改名废掉，`✗ lib_heavy_tests 自检：bash 起双机分片的驱动脚本 应当是 crash-case-cargo，实际 None` 与 `--merged-log` 那一格红（同一次里另有三格红，是把文件拷到仓外跑、找不到登记表的缘故，与这次改动无关）。
- 驱动脚本自证：把驱动脚本里比工具链的 `[[ "$local_toolchain" == "$peer_toolchain" ]]` 与数账本的 `[[ -f "${fetched_ledgers[0]}" … ]]` 都换成 `true` 再跑自证，③ ④ 两格红、其余 8 格绿（`driver-selftest-red.log`：`✗ layer0-shard-run.sh 自证没过：2 格判错（共 10 格）`）；改回之后 10 格全过。

## 第 4 步那几样的末尾原样输出（主工作区，2026-09-27）

开跑前看负载：`ps` 看不到 qemu / vm-bench / e152 / fio；中途有别的会话的 `cargo build --release … --bin e158_root_choice_repair` 与 `cargo run … --bin e156_allocation_basis_counts` 在跑，我的命令都加了 `nice -n 19`，没见到等锁。

- `cargo test --offline -p singlefs-harness --test crash_enumeration_sharded_across_processes`（经 run-with-memory-cap.sh 8G、capped.sh 8）：`test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.69s`
- `cargo test --offline -p singlefs-harness --lib`：`test result: ok. 81 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 35.77s`
- `cargo test --offline -p singlefs-harness --test crash_enumeration_resumes_from_its_progress_file`（没改它，改了它走的枚举本体）：`test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.95s`
- `cargo test --offline -p singlefs-harness --test second_transaction_crash_inside_the_floor_raise_pushed_by_the_session`（不带 `--include-ignored`）：`test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s`
- `cargo fmt --check -p singlefs-harness`：退 0、无输出
- `cargo clippy --offline -p singlefs-harness --lib --test crash_enumeration_sharded_across_processes --test first_transaction_step_seven_layer0 --test second_transaction_step_zero_layer0 --test crash_enumeration_resumes_from_its_progress_file --test second_transaction_crash_inside_the_floor_raise_pushed_by_the_session --all-features -- -D warnings`＋check.sh 那 7 条 `-D clippy::…`：退 0，末行 `    Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 0.10s`。`--all-targets` 那一趟红在别人的 `src/bin/e158_root_choice_repair.rs:7244/7245/9345/9498` 与 `src/bin/e156_allocation_basis_counts.rs:3309/3877/3879`（`shadow_unrelated`），不在我的改动里，没碰
- `cargo build --offline --all-targets`（工作区根）：退 0，末行 `    Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 0.33s`
- 登记给我的门禁阶段（`nice -n 19 bash research/scripts/capped.sh 8 bash .claude/gate.d/<阶段>`）：33 号退 0（`✓ … crates/mutations.tsv 859 条的原文各命中源码一次 …`）——追加第 13 行之后重跑退 1，红在第 301、690、692 行（别人的行，`crates/singlefs-core/src/mount.rs` 里命中 0 次，那个文件别的会话正在改），我的 13 行另用 `anchor-check.py` 核过：`checked 63 rows, bad 0`（crash.rs 与 layer0_progress.rs 两个文件上的全部行）；53 号退 0；74 号退 0（末行「随机历史：小盘上逼近单元区墙的取样点…」）；92 号退 0；94 号退 0；93 号退 0；89 号退 77（本次未跑，不是通过）。
- 草稿拷贝上的成品自证：`admission.py --selftest` `✓ admission.py 自证通过：181 格都对（…threads-by-worker-count、threads-ignore-shards 下各自那一格转红…）`；`lib_heavy_tests.py --selftest` `✓ lib_heavy_tests 自检通过（查了 113 种 …）`；`layer0-shard-run.sh --selftest` `✓ layer0-shard-run.sh 自证通过：10 格都对（第二台是本机上的另一个目录，没碰真的第二台）`；`73-research-gate-lint.sh <草稿拷贝>` 退 0（`✓ 门禁自检通过：87 个脚本（.sh 与 .py）、272 条拒绝都带了出路`、shell 纪律三行都过）。自证那一趟用的 crates 是交回前一版（golden 子进程用例改成不标 ignore、两处变量改名之前），两处都不改行为。

## 停下交主 agent 的设计问题与偏离规格的地方（都照做了，都是推的、没被攻过）

1. **账本文件名**写成 `<进度目录>/layer0-shard-<流名>-shard-<i>-of-<n>.tally`，不是规格的 `shard-<i>-of-<n>.tally`：多带流名，两条流的账本同放一个目录也不撞；不带输入指纹与计划哈希，两台对不上时 merge 读得到那一份、报出是文件头哪一处不同（规格要的「说清哪一处不同」），而不是报缺账本。
2. **账本文件头里的计划哈希**记整条流的（不带片），n 份才能逐字相同；分片跑的进度文件用带片的那个哈希。
3. **merge 那一趟的两行 `LAYER0_PARALLEL_*`**：`worker_threads=` / `configured_worker_threads=` / `resumed_slices=` / `freshly_run_slices=` 是 n 片之和（读回 + 跑过 = 总片数，今天的 `judge_worker_threads` 不改也认），`worker_threads_source=shard_ledgers`，另带 `shards=` 与逐片的 `shard_worker_threads=`、`shard_configured_worker_threads=`、`shard_worker_threads_sources=`、`shard_available_parallelism=`、`shard_resumed_slices=`、`shard_freshly_run_slices=`、`shard_elapsed_milliseconds=`。`admission.py` 的改法见 deliver 里的 diff：见到 `shards=` 就逐片判，机器核数与「显式设成 1」取那一片账本里记的。**没打补丁之前**今天的判法只看合计，某一片只起 1 个线程而另一片起了 16 个时判不出（弄坏开关那一格就是这个样子）。
4. **分片跑一片那一趟的两行 `LAYER0_PARALLEL_*`**：`states=` / `slices=` 是这一片的，另带 ` shard=<i>/<n> all_states= all_slices=`；这两行不交 54 号判（54 号只判 merge 那一趟）。
5. **配置多了两个键** `QUIESCE_STOPPED_CHECK_COMMAND`、`QUIESCE_STARTED_CHECK_COMMAND`（清场、复原之后回读用，退 0 才算做成），规格只列了五个键；写了 STOP 就要写 STOPPED_CHECK 与 START，写了 START 就要写 STARTED_CHECK。
6. **登记表多一种写法** `shard=across-machines`（`admission.py` 解析、自查、新子命令 `crash-case-shardable`），54 号靠它认哪几条用例能分片；只有两条流的全量用例的枚举经 `Layer0Resume::from_environment` 认开关，另两条（`floor-raise-pushed-by-the-session`、`c561-sigma-full`）不认，54 号照单机跑它们。
7. **驱动脚本多一种调法** `--merged-log`（给 54 号用，判与写标记留在 54 号，不抄第二份）。
8. **第二台上的目录**：每一趟在 `PEER_REPOSITORY_DIRECTORY/runs/` 下新建子目录、跑完删；两片的进度目录在 `PEER_REPOSITORY_DIRECTORY/progress/<指纹>/`。主 agent 在第二台上的仓副本目录已放了一份仓副本，驱动脚本不用那份副本，只在它下面建 `runs/` 与 `progress/`；要另指一个空目录就改配置。
9. **第二台那一片的线程数**取第二台的 `nproc`，不跟本机的 `SINGLEFS_LAYER0_THREADS`（本机那一片照 54 号的取法）。
10. **工具链怎么取**：`Layer0ToolchainIdentity::of_the_cargo_running_this_test` 在分片开关设了时起子进程问 `$CARGO -V` 与它旁边的 `rustc -Vv`（旁边没有就取 PATH 上的）；target triple 取 `rustc -Vv` 的 `host:` 行（几条流都不交叉编译）。环境里没有 `CARGO`（不是经 cargo test 起的）就 panic。
11. **merge 带 `SINGLEFS_LAYER0_START_OVER=1` 就 panic**（只读账本，没有进度文件可丢）；设了分片开关而没设进度目录也 panic。驱动脚本 merge 那一趟清掉 START_OVER。
12. **`enumerate_layer0_in_state_slices` 收到分片跑一片的 `resume` 就 panic**（这个入口要整条流的计数）；分片的用例改调 `_or_one_shard`。外壳里另有一处 `unreachable!`（`crash.rs` 第 2257 行）：走不到的理由是同一个函数开头第 2236 行的 `match resume` 已经在 `RunOneShardKeepingProgressFile` 上 panic，而 `_or_one_shard` 只在那个成员上交回 `OneShardWrittenToItsLedger`（它唯一的构造点在 `_or_one_shard` 末尾，`ledger_written` 只由 `shard_run` 来）。
13. **`--selftest` 的分派写在 `preflight` 之前**（照 `research/scripts/gate-staged.sh` 的先例）：运行条件要配置在、第二台连得上，`--selftest` 不该被它拒。preflight 那几行文件头在这个仓里目前不被门禁查（没有 `.claude/preflight-dirs`），写法照 `preflight-discipline.md`，`run-condition: check` 的拒绝经自证 ⑤ 核过（退 78、出路指到模板）。
14. **只供测试的开关**两个：`SINGLEFS_LAYER0_SHARD_PEER_IS_THIS_MACHINE=1`（「第二台」是本机上的目录，不 ssh）；`SINGLEFS_LAYER0_SHARD_INCLUDE_IGNORED=0`（三趟 cargo test 不带 `--include-ignored`：自证那条小流用例不标 ignore；主 agent 转来的看门狗报告之后加的，之前那几趟自证带过 `--include-ignored --exact <小流用例>`，`--exact` 之下没跑到标 ignore 的用例）。
15. `.claude/rules/implementation-workflow.md`「重型测试只在提交时跑」的清单里没有这个驱动脚本；`lib_heavy_tests.py` 的 diff 让闸认它，规则那一行要不要加归主 agent。

## README 那一节要写的配置项（主 agent 写 README）

配置文件：`${SINGLEFS_MULTI_HOST_CONFIG:-<主工作树的根>/multi-host.env}`，git 忽略；模板 `multi-host.env.example`（仓根）。一行一个 `KEY=值`，值不做 shell 展开，七个键都要写。没有这份配置（或判不过）时门禁 54 号 `--full` 照单机跑。

| 键 | 一句话 |
|---|---|
| `PEER_SSH_HOST` | 第二台的 ssh 别名，要能免密登录（`ssh -o BatchMode=yes <别名> true` 退 0） |
| `PEER_REPOSITORY_DIRECTORY` | 第二台上的专用目录（绝对路径）：每一趟在 `runs/` 下放树与编译目录、跑完删；两片的进度文件与账本在 `progress/<输入指纹>/` |
| `PEER_CARGO_BIN_DIRECTORY` | 第二台的 `~/.cargo/bin`（绝对路径；非登录 shell 的 PATH 里没有它） |
| `QUIESCE_STOP_COMMAND` | 开跑前在本机跑的清场命令，空串表示不清场 |
| `QUIESCE_STOPPED_CHECK_COMMAND` | 清场之后回读：退 0 才算清完；写了 STOP 就要写它 |
| `QUIESCE_START_COMMAND` | 跑完（跑红了也跑）在本机复原；写了 STOP 就要写它 |
| `QUIESCE_STARTED_CHECK_COMMAND` | 复原之后回读：退 0 才算复原了；写了 START 就要写它 |

另两个环境变量：`SINGLEFS_LAYER0_SHARD`（`<i>/<n>` 或 `merge/<n>`，驱动脚本自己设，一般不手设）；判配置的是 `bash research/scripts/layer0-shard-configuration-check.sh [<仓根>]`（退 0 能分片）。

## 没做什么

- 没走三方对抗；层 0、QEMU、herd7 与 crates 变异整表归 `crash-verifier`；没提交。
- 没登录第二台、没在它上面装东西、没跑驱动脚本的非 `--selftest` 形态；真的两台上的全量一次都没跑过，账本跨机拷贝、第二台的构建环境指纹与本机对不对得上（`~/.cargo/config*`、`RUSTFLAGS` 这类进指纹）都没验过——对不上时驱动脚本在第 ③ 步拒。
- 没跑任何名字带 layer0 的测试目标（`first_transaction_step_seven_layer0`、`second_transaction_step_zero_layer0` 只编过、clippy 过），没跑 54 号快档：规格说可以跑，但它跑的就是这两个 layer0 测试二进制，定义与派发都不许，重型闸也拒；「54 号快档不因我的改动变红」没验，只验了默认不分片时枚举的打印行与进度文件逐字节不变（golden 用例）。两条全量用例的分片分支（`OneShardWrittenToItsLedger` 就收工）没有任何跑过的测试罩着，要等 crash-verifier 在提交时跑全量、或主 agent 跑驱动脚本。
- 写范围外的东西都没进仓：驱动脚本三份、模板、`admission.py`、54 号、`stage-inputs.tsv`、`lib_heavy_tests.py`、kb 第六节，都在 `deliver/`。它们在草稿拷贝上的自证都过了，放进仓之后要重跑一遍。
- 门禁 59 号（变异整表复跑）没跑；33 号在我追加第 13 行之后红在别人的三行（mount.rs），没修。
- 仓根那个未跟踪的 `-.rej`（76800 字节）是我留下的：在仓根跑 `patch -o - … crates/singlefs-harness/src/crash.rs <开工时的 diff>` 核行数时，patch 把打不上的块写成了 `-.rej`（`-o -` 的输出名是 `-`）。看过开头三行（`--- crash.rs` / `+++ crash.rs` / `@@ -12,7 +12,7 @@ use std::sync::mpsc;`，29 块）确认是它，已删；那条命令用 `-o -` 输出到标准输出，没改 `crash.rs`（同目录没有 `.orig` / `.rej`）。

## 草稿与清理

删了（交回前）：`/tmp/claude-1000/impl-shard-1/golden-copy`（1.5G，拿加分片之前的代码跑 golden 的仓副本连 target）、`/tmp/claude-1000/impl-shard-1/red-copy`（2.0G，证红用的仓副本连 target）、`/tmp/claude-1000/impl-shard-1/deliver-copy`（161M，成品自证用的仓副本）、`redproof-heavy`（144K）、`cfgtest`（16K）、`golden-out`（32K）；我的测试红了时留在 `/tmp` 的 17 个 `singlefs-crash-enumeration-sharded-*` 目录（最大 44K）。驱动脚本自证的三个临时目录（各 1.1G）自证自己删了。
留着的：`/tmp/claude-1000/impl-shard-1/deliver/`（成品，主 agent 要拷）、`golden-child/`（golden 子进程用例的那几段拼成的测试文件，复核 golden 用）、`scripts/`（我的辅助脚本）、各 `*.log`、`*.orig`（成品 diff 的底）、`layer0-shard-run.good.sh`、`54-deliver-good.sh`（证红时改回用的底）、`crash-diff-before.patch` 与 `crash-diffstat-before.txt`（开工时 crash.rs 的差）、`diffstat-final.txt`。
