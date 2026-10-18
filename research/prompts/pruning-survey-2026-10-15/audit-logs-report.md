# 重型门禁复用键审计（路子三：按真实日志核实际复用了多少）

日期：2026-10-15（`TZ=Asia/Tokyo date +%F`）。批次：records/2026-10-13-复用键与全仓改名.md#2026-10-15-批2。
只读：没起 cargo、门禁、测试二进制，没动判定库；只读日志、grep、git show / diff、python3 解析。
标注：每条结论标「量的（指到哪一行）」或「推的」。日志里的私有主机名照原样出现处，本报告一律写成「第二台」。

## 0 结论摘要

1. 近一周七道里真跑过、留下计数行的只有四道：崩溃点重放 --full（连同崩溃放量全量）、crates 变异整表（一趟）、形式化验证（一趟）、
   研究侧 mutation-tables-compile（四趟）。QEMU、LKMM 只有整道全绿标记的「复用 / 重跑」两态。广谱变异近一周没真跑（最后一趟 10-01）。
   experiment-replay 近一周没查到一趟日志，它的判定库目录 `.git/singlefs-replay-verdicts` 不存在（量的，§2.8）。
2. 复用掉成 0 的那几趟，原因全是「整库级」的键变了，没有一趟是「只作废受影响的那部分」：
   - 崩溃放量全量三次掉成 0（10-11、10-12、10-13 各一次），每次都是判法版本换了（7fa0adbf → f0b9512b → dca6156d），
     而判器闭包里收了流水线 `crash_amplification.rs`、存储 `verdict_store.rs`、`crash_library.rs`、几乎整个 `singlefs-core`，
     还按原文收 wgsl（注释也算）（量的，§2.2）。
   - crates 变异那一趟分片段「复用抓到 0 条」，判法版本是新的一格目录（量的，§2.5）。
   - 形式化 pinned-red-examples-replay「现判 360、复用 0（装置 变了 360）」，装置键是整个 checker 档 bin 的 cargo 依赖闭包按原文取（量的，§2.6）。
   - mutation-tables-compile 一趟「现判 164 张、复用 0 张」，原因「mutate.sh、判法版本 变了」；这一格的库放在 /tmp，现在整个库目录已经不在了（量的，§2.8）。
3. 另有三处与键无关、同样让复用变 0 的：判定库目录随改名换了（`layer0-enumeration-library` → `crash-state-enumeration-library`，量的）；
   崩溃点重放的用例级指纹从 10-12 的「185 个文件，减去用例读不到的 176 个」变成 10-14 的「421 个文件，减去用例读不到的 0 个」（量的，原因推的）；
   崩溃放量与崩溃点重放把状态数相同的两条流在两个库里各判一遍（量的状态数相等，同一批状态推的）。
4. 最该先改的三条见 §3 末尾：①判器闭包收进流水线与整个 core、wgsl 按原文；②各判定库的判法版本把「阶段脚本、内存包装、判定库共用部分」整份收进去，改一行脚本整库作废；
   ③判定库放在会被清掉或会随改名换掉的目录。

## 1 找过的目录（全部列出）

| 目录 | 找到什么 |
|---|---|
| `~/.local/state/singlefs/detached-runs/*/log`（116 个，10-08 以后的逐个看过 command） | full-2026-10-11 … full-2026-10-11r、full-2026-10-15 的 log 只有几百字节（真输出在 commit-2026-10-11/ 下）；owed-c795/c798 四趟 mutation-tables-compile；broad-mutation-local（10-01） |
| `~/.local/share/singlefs/work/commit-2026-10-11/` | `f-amplification.log`（10-13 那一趟，之前各趟被 `>` 覆盖）、`f-layer0-full.log`（10-12 那一趟）、`f-layer0-full.l.log`（10-12 那一趟）、`checker-tier-*.log` 与 `.marker`、`summary.txt`、`full-summary.txt` |
| `~/.local/share/singlefs/work/commit-2026-10-11/full-2026-10-14/f1.log` | 正在跑的 full-2026-10-15 的崩溃点重放 --full（读的时候还在跑，59 行） |
| `~/.local/share/singlefs/work/commit-2026-10-11/merge-2026-10-14/*.log`、`two-stage-validate/*` | 只有 --quick 档（tier=quick）与库内单测，属轻型，不在射程；只拿来取判器闭包的文件清单 |
| `~/.local/share/singlefs/crash-amplification/runs/`（434 个子目录） | `entry-*/<分项>.log`（崩溃放量逐分项）、`layer0-layer0-*`、`crash-point-replay-*`（双机驱动的 share/import/record 日志） |
| `/home/fy5090/code/singlefs/.git/singlefs-layer0-logs/*`、`.git/singlefs-crash-point-replay-logs/*` | 崩溃点重放 --full 各用例的全量日志 |
| `.git/singlefs-mutation-shard-logs/`、`.git/singlefs-mutation-rows/` | crates 变异双机分片日志与判定库 |
| `~/.local/share/singlefs/broad-mutation/` | 广谱变异库（versions/、conclusions.tsv） |
| 全盘 `find / -maxdepth 6 -name singlefs-mutation-tables-compile-verdicts` | 没找到（§2.8） |
| `grep -rl '^REPLAY_TOTAL dispatched'` 于 research/results/、~/.local/share/singlefs/、~/.local/state/singlefs/、/tmp/claude-1000 | 没找到一份 |

整理用的表与脚本都在草稿目录：`entry-table.tsv`（崩溃放量逐趟逐分项）、`cpr-table.tsv`（崩溃点重放逐用例）、`closure-files-68.txt` / `closure-files-53.txt`（判器闭包）、`closure-diffs.txt`（相邻暂存树之间闭包文件的 diff）、`tools/*.py`。

## 2 逐道

### 2.1 checker-tier-crash-point-replay（--full）

| 格 | 库在哪 | 键的组成 | 作废粒度 | 合理与否 | 依据 |
|---|---|---|---|---|---|
| 用例级全绿标记 | `.git/singlefs-crash-case-green.<用例>.<指纹>` | 整个 `crates/` 与 Cargo 清单、锁下 git 列得出的文件，减去用例读不到的三类；加判法摘要、工具链、构建环境、登记行；双机用例另加四份驱动脚本 | 一条用例整条 | 键过宽：10-14 那一趟每条用例 421 个文件、减去 0 个，`crates/mutations.tsv`、各 `src/bin/` 实验装置改了也让每条用例失标记 | `scripts/gate/admission.py:1617`–1630；f1.log「421 个文件，减去用例读不到的 0 个」（量的） |
| 按块判定库（单机用例）与进度文件 | `~/.local/share/singlefs/crash-amplification/<类>-library`、`.git/singlefs-layer0-progress` | 块键 = 输入指纹 ‖ 流名 ‖ 枚举计划哈希 ‖ 块起点；另一列判法版本 = 判器闭包摘要 | 判法版本一变整库 | 键过宽（判器闭包，见 §2.2） | `crates/singlefs-checker-tier/src/verdict_store.rs:16`–24（量的） |
| 双机用例的判定库 | 10-12 白天 `library-by-conclusion-kind`，10-12 夜里起 `layer0-enumeration-library`，10-13 改名后 `crash-state-enumeration-library` | 同上 | 库目录一换整库没了 | 不合理：改名、换库路径不迁移 | record.log 的 `CRASH_LIBRARY_RECORDED … library=` 行；`stat` 生日（量的，下面） |

近一周各趟计数行（原样）：

- 10-12 那一趟（`.git/singlefs-layer0-logs/20261012-3532157/log.crash-injection-fast-tier`），现判、复用 0：
  `CRASH_POINT_REPLAY_GIVEN_STATES item=crash-injection-fast-tier mode=two_hosts_cpu_with_gpus judged_by=gpu_judge tables_judged_here=116 tables_reused=0 states_judged_by_the_gpu_here=369 states_judged_by_the_cpu_here=0 states_reused=0 states_the_gpu_could_not_judge=0 share=0/1 imported_blocks=0 tables_left_to_the_other_hosts=0 stopped_early=false`
- 10-12 那一趟（同目录 `20261012-1997314`），用例标记全换了指纹，但按块全复用：
  `CRASH_POINT_REPLAY_GIVEN_STATES item=crash-injection-fast-tier mode=two_hosts_cpu_with_gpus judged_by=gpu_judge tables_judged_here=0 tables_reused=116 states_judged_by_the_gpu_here=0 states_judged_by_the_cpu_here=0 states_reused=369 states_the_gpu_could_not_judge=0 share=0/1 imported_blocks=0 tables_left_to_the_other_hosts=0 stopped_early=false`
  `LAYER0_PARALLEL_FINISHED states=268435480 slices=4098 worker_threads=32 configured_worker_threads=32 worker_threads_source=environment_variable resumed_slices=4098 freshly_run_slices=0 progress_file_after_completion=none elapsed_seconds=9.4 judge=gpu`
- 10-12 那一趟（`commit-2026-10-11/f-layer0-full.log` 第 108 行），用例级 9 条复用、3 条现跑：
  `  ✓ --full 跑完（开跑 2026-10-12，跑完 2026-10-13）：12 条崩溃枚举用例，这一趟跑了判绿 3 条（crash-case:layer0-first-stream crash-case:layer0-second-stream crash-case:layer0-multi-record-publish-stream），复用 9 条（crash-case:floor-raise-pushed-by-the-session crash-case:c561-sigma-full crash-case:layer0-tree-split-streams crash-case:layer0-position-addressed-tree-streams crash-case:writable-mount-of-an-io-min-8192-pool crash-case:crash-injection-fast-tier crash-case:read-faulted-states-never-hold-the-witness-criterion crash-case:instance-table-sweep-mount crash-case:instance-table-sweep-phases-remount）；日志在 /home/fy5090/code/singlefs/.git/singlefs-layer0-logs/20261012-2501999/`
  那 3 条双机用例在驱动里两片全现判、复用 0（`runs/layer0-layer0-second-stream-20261012-2515435/share-0.log`）：
  `LAYER0_PARALLEL_FINISHED states=1678377204 slices=12806 worker_threads=31 configured_worker_threads=31 worker_threads_source=environment_variable resumed_slices=0 freshly_run_slices=12806 progress_file_after_completion=none elapsed_seconds=9286.2 judge=gpu shard=0/2 all_blocks=25612 blocks_left_to_other_hosts=12806`
  （这一片 9286 秒，量的；同一批 1678377204 个状态，崩溃放量 10-12 那一趟已在它自己的库里判过、全复用，见 §2.2）
- 10-14 起的 full-2026-10-15（`full-2026-10-14/f1.log` 与 `.git/singlefs-crash-point-replay-logs/20261014-3463989/log.floor-raise-pushed-by-the-session`），复用 0：
  `CRASH_POINT_REPLAY_PARALLEL_FINISHED states=8223 slices=2 worker_threads=31 configured_worker_threads=31 worker_threads_source=environment_variable resumed_slices=0 freshly_run_slices=2 progress_file_after_completion=none elapsed_seconds=633.6 judge=gpu`
  （同一条用例 10-12 那一趟是 `resumed=2,fresh=0`；两条双机用例这一趟卡在第二台清场回读、退 78，没判到）

复用为 0 的那几趟，前面改了什么：

- 10-12 22:58 三条双机用例全现判：前一趟（10-12 13:37）三条的分片都在 `crates/singlefs-checker-tier/src/layer0_enumeration.rs:328:31` panic，没判出东西
  （`runs/layer0-layer0-first-stream-20261012-1997653/share-0.log`，量的）；再前两趟只到录入段。库路径同时换了：前几趟 record.log 写
  `library=/home/fy5090/.local/share/singlefs/crash-amplification/library-by-conclusion-kind`，那一趟写 `library=…/layer0-enumeration-library`（量的）。
  按用户口径：这一次没有可复用的旧判定，现判本身不算浪费；库路径一趟一换才是浪费的来源（推的）。
- 10-14 floor-raise 现判：暂存树之间判器闭包有改动（§2.2），且库目录改名：`crash-state-enumeration-library` 生日 2026-10-13、旧库 `layer0-enumeration-library`
  （624M）最后改动 2026-10-13，新库 333M（`stat`、`du -sh`，量的）。用例指纹与 10-12 的标记「不同的共 409 处」，前 20 处是判法摘要、登记行、
  `crates/mutations.tsv`、`crates/singlefs-checker-tier/Cargo.toml`、`build.rs`、十几份 `src/bin/*.rs` 实验装置（f1.log，量的）。
  按用户口径：`mutations.tsv` 与 `src/bin/` 实验装置这类用例读不到的文件不该让用例失标记；改名不该让库清零。

用例级指纹「减去 0 个」的原因（推的，没量）：`admission.py:1447`–1448 在包的 `src/` 里见到参数是算出来的 include 就 `return None`，整包不减；
10-14 的提交 d6dd6da4 新加了 `crates/singlefs-harness/src/heavy_threads.rs:22` `include_str!(concat!(`（`git log -1 -- 该文件` 是 d6dd6da4 2026-10-14，量的），
每条用例都依赖 harness。推翻条件：在副本里把那一处换成字面量路径，`python3 scripts/gate/admission.py paths <根> crash-case:<用例>` 列出的文件数仍是 421、减去仍是 0。
另一个候选是 `crates/singlefs-checker-tier/src/crash_identity.rs:661` 的 `include_str!(concat!(env!("OUT_DIR"), …))`，但它在 10-12 的树（4fc0bcda）里已经有、那时仍减了 176 个，所以排在后面（量的：`git grep` 于 4fc0bcda 命中第 639 行）。

### 2.2 崩溃放量全量（`scripts/crash/crash-amplification.sh`，展开 30）

| 格 | 库在哪 | 键的组成 | 作废粒度 | 合理与否 | 依据 |
|---|---|---|---|---|---|
| 判定块 | 崩溃放量长期库（RocksDB；full-run.sh 文件头写 `library-by-conclusion-kind`） | 块键 = 输入指纹 ‖ 流名 ‖ 枚举计划哈希 ‖ 块起点；判法版本（32 字节）另一列 | 判法版本一变整库；块键只随流程变，按块作废 | 块键合理；判法版本键过宽（下面三条） | `verdict_store.rs:16`–24；`crash_identity.rs:6`–8 |
| 判法版本 | 同上，`09` 表记文件清单 | 判器闭包：从 `crash.rs`、`crash_judge_gpu.rs` 出发顺着 `use` 收文件，`include_str!` 读的按原文、其余去注释取词法；另拼 `Cargo.lock`、各 crate 的 `Cargo.toml` 原文与工具链 | 整库 | 键过宽：①10-14 两份日志里闭包收 68 个文件（含 `crash_amplification.rs`、`verdict_store.rs`、`crash_library.rs`、`crash_point_replay_progress.rs`、`gpu_card_configuration.rs`），10-14 合入之后 53 个，53 个里仍有 `singlefs-core` 的 32 份（`crates/singlefs-core/src/` 共 34 份 .rs）（admission、allocator、transaction、make_filesystem、unlink、write_request_split…写路径）；②wgsl 按原文进，注释改一个字也换版本；③`Cargo.lock` 整份进，任何依赖升级整库作废 | `crash_identity.rs:512`；`two-stage-validate/lib-nofeature.log` 的 `JUDGING_CODE entry=judges files=68 raw=8 stopped=2`；`merge-2026-10-14/lib-gpu-diag-2.log` 等的 `files=53 raw=5 stopped=2`（量的） |
| 跨类复用 | 崩溃放量与崩溃点重放各一个库 | 块键带流名（`multi-record-publish` 对 `multi_record_publish::full`） | — | 键过窄（重复判）：同一批状态两库各判一遍 | 状态数相等：放量 `fixed-script-through-unmount` states=1678377204 与崩溃点重放 `layer0-second-stream` states=1678377204；放量 `multi-record-publish` reused=1358954634 与崩溃点重放 `layer0-multi-record-publish-stream` states=1358954634（量的）；是同一批状态（推的） |

近一周展开 30 各趟（从 `runs/entry-*/<分项>.log` 抽，全表在草稿目录 `entry-table.tsv`；pending=这一趟要现判的状态，reused=库里复用的状态，量的）：

| 那一趟（entry 目录） | 暂存树 | 判法版本 | 分项：pending / reused |
|---|---|---|---|
| 20261011 | 0da47fde | 没打出（中途停） | rollback 67502198 / 0 |
| 20261011 | 663bb8b6 | 没打出 | rollback 3997700 / 63504498 |
| 20261011 | 663bb8b6 | cedd6ed84834daee | rollback 0 / 67502198 |
| **20261011** | b5a35b44 | **7fa0adbf10708a36** | **rollback 67502198 / 0**；unmount 1610875015 / 67502189；multi-record-publish 1342177375 / 16777259 |
| **20261012** | 3acf5002 | **f0b9512b88d5bbfe** | **rollback 67502198 / 0** |
| 20261012 | 3acf5002 | f0b9512b | rollback 6422539 / 61079659 |
| 20261012 | cac8ab25 | f0b9512b | rollback 0 / 67502198；unmount 1478885503 / 199491701；multi-record-publish 1342177375 / 16777259；unlink 104857653 / 1358954625；position-addressed-trees 268435480 / 0 |
| 20261012 | cac8ab25 | f0b9512b | 八项全部 pending 0（例：unmount 0 / 1678377204） |
| **20261013** | 9da32924 | **dca6156d6e736c21** | **rollback 67502198 / 0**；unmount 1610875015 / 67502189 |
| 20261013 | a07c22e2 | dca6156d | rollback 0 / 67502198；unmount 1422786677 / 255590527 |
| 20261013 | fae5b14c | dca6156d | unmount 165412906 / 1512964298（这一趟被停：用户定先全仓改名） |

原样计数行（各取 `CRASH_AMPLIFICATION_PLACEMENT host=local` 第一行）：

- 全复用的一趟，`runs/entry-20261012-3490497/fixed-script-through-unmount.log`：
  `CRASH_AMPLIFICATION_PLACEMENT host=local judge=gpu pending_blocks=0 pending_states=0 states_reused=1678377204 tables_mebibytes=49 cards=local:0:18413:0,peer:0:8602:0,local:1:8268:0,peer:1:8602:0,peer:2:8602:0 states_placed_on_local=0 states_placed_on_peer=0 states_per_second_on_this_host=0 estimated_seconds_on_this_host=0`
- 掉成 0 的第一次，`runs/entry-20261011-1990665/fixed-script-through-rollback.log`：
  `CRASH_AMPLIFICATION_PLACEMENT host=local judge=gpu pending_blocks=1060 pending_states=67502198 states_reused=0 tables_mebibytes=19 cards=local:0:18432:18432,peer:0:8621:8621,local:1:8287:8287,peer:1:8621:8621,peer:2:8621:8621 states_placed_on_local=26719 states_placed_on_peer=25863 states_per_second_on_this_host=193697 estimated_seconds_on_this_host=348`
- 掉成 0 的第二次，`runs/entry-20261012-2782527/fixed-script-through-rollback.log`：
  `CRASH_AMPLIFICATION_PLACEMENT host=local judge=gpu pending_blocks=1060 pending_states=67502198 states_reused=0 tables_mebibytes=19 cards=local:0:18432:18432,peer:0:8621:8621,local:1:8287:8287,peer:1:8621:8621,peer:2:8621:8621 states_placed_on_local=26719 states_placed_on_peer=25863 states_per_second_on_this_host=163094 estimated_seconds_on_this_host=414`
- 掉成 0 的第三次，`runs/entry-20261013-3168619/fixed-script-through-rollback.log`：
  `CRASH_AMPLIFICATION_PLACEMENT host=local judge=gpu pending_blocks=1060 pending_states=67502198 states_reused=0 tables_mebibytes=19 cards=local:0:18432:18432,peer:0:8621:8621,local:1:8287:8287,peer:1:8621:8621,peer:2:8621:8621 states_placed_on_local=26719 states_placed_on_peer=25863 states_per_second_on_this_host=165949 estimated_seconds_on_this_host=407`
- 10-13 那一趟入口的分项总结（`commit-2026-10-11/f-amplification.log`）：
  `CRASH_AMPLIFICATION_ITEM item=fixed-script-through-rollback exit=0 states=0 reused=67502198 red=0 lines=3 expand_up_to=30 exhaustive=true verdict=green seconds=143 ignored_points=0 states_skipped_by_ignore=0 registered_segments=- unregistered_segments_beyond_expansion=- registered_segment_domains=- registered_segment_progress=-`

续跑本身是好的：同一判法版本下，中途停掉的那几趟判过的块下一趟都复用了（10-11、10-12、10-13 三行，量的）。复用掉成 0 只发生在判法版本换了的那三趟。

三次掉成 0，前面改了哪些判器闭包里的文件（`git diff --stat <前一棵暂存树> <这一棵> -- $(cat closure-files-68.txt)`，全文在 `closure-diffs.txt`，量的）：

1. 663bb8b6 → b5a35b44（换成 7fa0adbf）：`crash_amplification.rs` 3742 行、`crash.rs` 2779 行、`crash_library.rs` +980、`verdict_store.rs` +151、`crash_judge_dispatch.rs` 151、
   `crash_facts.rs` 42、`crash_given_states.rs` 29、`crash_judge_conclusions.rs` 16、`crash_identity.rs` 10、`crash_verify_gpu.rs` 6、`gpu_memory_ledger.rs` 2。
   这是「CPU 录入写库、GPU 读库判」两段化的流水线与存储改写（full-summary.txt「先改代码：CPU 跑数据入 RocksDB → 从 RocksDB 取 → … 两段分开」）。
   那时的 GPU 判器 `crash_judge_gpu.rs` 引 `crate::crash_amplification` 与 `crate::crash_judge_dispatch`（`git show 663bb8b6:…/crash_judge_gpu.rs | grep -o 'crate::[a-z_]*'`，量的），流水线因此在闭包里。
   按用户口径：流水线与存储换了写法、判对错的代码没动的部分不该让判定作废；只有 `crash_facts.rs`、`crash_judge_conclusions.rs` 那几行要看是不是改了判法（推的）。
2. b5a35b44 → f1985f9f（换成 f0b9512b）：29 个闭包文件；我逐行看过的 `admission.rs`、`allocator.rs`、`pointer.rs`、`crash_judge_gpu.rs` 与三份 wgsl，diff 全是注释里「层 0」改「崩溃点重放」、删「实审 A…」字样（量的；其余 22 份没逐行看，是不是也只改注释没量）；真改代码的是 journal 提示（`block_device.rs` +11 加 `journal_record_offsets_hint` 默认返回 `None`、`recovery.rs` 里转发、
   `memory_pool.rs` +28）。按用户口径：注释改动不该换版本（Rust 那边已去注释，wgsl 按原文是漏洞）；journal 提示只是给观察者提速，判定结果照理不变（推的），可以用新旧两版在同一批块上抽样对拍来证，证过就不该整库重判。
3. 4fc0bcda → 9da32924（换成 dca6156d；中间 cac8ab25 → d83fe646 只动了 `verdict_store.rs`，d83fe646 → 4fc0bcda 没动 crates）：`crash_amplification.rs` 4029、`crash.rs` 847、
   `crash_facts.rs` 446、`verdict_store.rs` 322、`crash_judge_tables.rs` 149、`crash_verify_gpu.rs` 125、`crash_judge_common.wgsl` 74、`crash_library.rs` 23。
   wgsl 里真改的是按段切片的进位与 `UNDECIDABLE_SLICE_PAST_ITS_DOMAIN`，只在 `slice_digit_count > 0`（登记了切片的段）时走到（量的，diff）；
   这一趟八个分项的 `registered_segments=-`（f-amplification.log，量的），即不走切片。按用户口径：只改切片分支的判法改动，不该让没登记切片的流整库重判（推的：要证明这几条流的判定向量不随它变）。

旁证（轻型 --quick 档，不在射程，只说明闭包变动的频度）：10-14 一天里三趟 --quick 各是一把新版本 `gpu_judge:3b773890a855e719`、`gpu_judge:c17fb1e853b0c86c`、`gpu_judge:4183272c362039b1`（`entry-20261014`、`T045216Z`、`T072437Z`，量的）。

### 2.3 checker-tier-qemu-device-streams

| 格 | 库在哪 | 键的组成 | 作废粒度 | 合理与否 | 依据 |
|---|---|---|---|---|---|
| device-streams（唯一一格） | 没有判定库，只有整道全绿标记 `.git/singlefs-stage-green.checker-tier-qemu-device-streams.sh.<指纹>` | `crates/ Cargo.toml Cargo.lock scripts/ research/results/ .claude/kb/layout/01-first-txn.md`，加 `stage-inputs.tsv` 整份 | 整道 | 没有库、键过宽：`scripts/` 与 `research/results/` 下任何一份文件变了都整道重跑；6 次虚机跑各自的输入没分开 | `.claude/gate.d/stage-inputs.tsv:37`、`:30`（量的） |

近一周（`commit-2026-10-11/summary.txt`，原样）：
`checker-tier-qemu-device-streams rc=1 秒=9`、`checker-tier-qemu-device-streams rc=1 秒=3`（两次）、`checker-tier-qemu-device-streams rc=0 秒=133`；没有一行「复用」。
最后一次判绿的输出（`checker-tier-qemu-device-streams.log`）：`  ✓ QEMU 真设备：6 次虚机跑（direct skip-new-pool-file-creation-barrier page-cache file-overwrite second-instance raise-rollback-floor）、123 项检查全过；direct file-overwrite second-instance raise-rollback-floor 设备侧逐项对得上（宿主照模式重跑到这一档最后那次发布），2 个对照（skip-new-pool-file-creation-barrier page-cache）都红在该红的地方`
复用为 0：前三趟是红的（红不写标记，重跑应当）；那一趟（暂存树 d83fe646）之前近一周没有一次绿，标记查不到也应当；10-13 那一趟的提交阶段没跑到 QEMU 就停了。
所以近一周没观测到「键过宽导致白跑」的一趟（量的：summary.txt 只有上面四行）。键过宽是按登记推的：例 cac8ab25 → d83fe646 只动了 crates 里 3 份文件（其中 `verdict_store.rs`），照登记 QEMU 也要重跑，这类改动与虚机设备流无关（推的）。整道 133 秒，代价小。

### 2.4 checker-tier-lkmm

| 格 | 库在哪 | 键的组成 | 作废粒度 | 合理与否 | 依据 |
|---|---|---|---|---|---|
| litmus-verdicts（唯一一格） | 没有判定库，只有整道全绿标记 | `litmus/ .claude/scripts/lkmm.sh .claude/scripts/fetch-deps.sh .claude/singlefs-ai-sop/scripts/lib.sh crates/`，加 herd7 版本、`stage-inputs.tsv` | 整道 | 没有库、键过宽（整个 `crates/`）；8 份 litmus 各判各的却不分开记 | `stage-inputs.tsv:38`（量的） |

近一周（`summary.txt`，原样）：
`checker-tier-lkmm 复用：ok /home/fy5090/code/singlefs/.git/singlefs-stage-green.checker-tier-lkmm.sh.1849ded8b957d9fe5876373a82d8cea0a728d49b75e8e5b114c7a3a4db0ca767 2026-10-12`（三次），
`checker-tier-lkmm rc=0 秒=1`（三次）。复用掉成 0 的两趟前面：暂存树 74022d17 → d83fe646 只动了 crates 下 3 份文件，`litmus/`、`lkmm.sh`、`fetch-deps.sh` 0 份；d83fe646 → 1722e788 动了 crates 下 2 份与 `stage-inputs.tsv`，litmus 那三样 0 份（`git diff --name-only <树> <树> -- …` 数的，树名取自 `fingerprints.txt` 的 tree 列，量的）。即两次重跑都不是 litmus 或判法变了。
按用户口径：不合理但不痛，整道 1 秒。

### 2.5 checker-tier-crates-mutation-replay

| 格 | 库在哪 | 键的组成 | 作废粒度 | 合理与否 | 依据 |
|---|---|---|---|---|---|
| crates-mutation-rows | `.git/singlefs-mutation-rows/<判法版本>/`，一条一个文件 | 判法版本 = 登记行 + 阶段脚本 + `crates-mutation-rows.py` + `mutation-prune.py` + `lib_libtest_result_lines.py` + `lib_verdict_library.py` + 内存包装 `run-with-memory-cap.sh` + 构建环境；行键 = 这一行六段 + 每条内存上限 + 每条限时 + 工具链；再按快照与覆盖记录比「被变异的函数、执行过的函数」 | 判法版本一变整库；行级按覆盖剪枝 | 判法版本键过宽：阶段脚本与内存包装（全仓共用）改一行整库作废，且只留最新 3 个版本目录，改回去也找不回；行键带内存上限与限时，换一个上限整表重判 | `scripts/mutation/crates-mutation-rows.py:58`–61、`:84`（量的） |

近一周只有一趟整表（10-12，`commit-2026-10-11/checker-tier-crates-mutation-replay.log`，原样）：
- 本机那一片：`        … 只判了选中的 1977 行，不是整道结论（GATE_MUTATION_ROW_SELECTION=/tmp/tmp.pxdlYCf5yk/local.selection；有红，见上面；判定库记录写进了 /home/fy5090/code/singlefs/.git/singlefs-mutation-rows/37ab6ce31a0dbf86b9cbc5e41eeb75424a14223e3a733d3865ff4313eb395730）；这一趟现判 1977 条（一律现跑 0、行新加或改了 0、没有覆盖记录 1977、被变异的函数变了 0、执行过的函数变了 0、函数外有改动 0）、按覆盖复用 0 条（其中源码与上次一字未动 0 条）`
- 第二台那一片：`        … 判定库记录：复用抓到 0 条、复用基线 ok 0 行；现跑基线 239 批 1976 行、现跑变异至多 1976 条、补记覆盖 0 组；前移 0 条（上一次用时 ≥ 300 秒或上一次超时的先派，按用时从长到短）；判法版本 37ab6ce31a0dbf86…，记录在 第二台的目录/mutation-kept/records/singlefs-mutation-rows/37ab6ce31a0dbf86b9cbc5e41eeb75424a14223e3a733d3865ff4313eb395730`
  （这一片随后「记覆盖要的 llvm 工具不对 … 一条变异都没跑」，量的）
- 本机整张那一趟：`  … 判定库记录：复用抓到 1693 条、复用基线 ok 66 行；现跑基线 243 批 2194 行、现跑变异至多 2260 条、补记覆盖 13 组；前移 93 条（上一次用时 ≥ 300 秒或上一次超时的先派，按用时从长到短）；判法版本 37ab6ce31a0dbf86…，记录在 /home/fy5090/code/singlefs/.git/singlefs-mutation-rows/37ab6ce31a0dbf86b9cbc5e41eeb75424a14223e3a733d3865ff4313eb395730`
  这 1693 条是同一趟本机那一片刚判的（推的：分片段之前这一格目录里没有新格式记录，见「没有覆盖记录 1977」与文件头 `:87`–89「目录里一条新格式记录都没有时记「没有覆盖记录」」）。跨趟复用 = 0。

复用为 0 的原因：判法版本 37ab6ce3 是新开的一格（量的：这一趟之前的版本目录已被「只留 3 个」删掉，看不到上一版名）。近一周改动判法版本输入的提交：
`git log --since 2026-10-08 -- .claude/gate.d/checker-tier-crates-mutation-replay.sh scripts/mutation/crates-mutation-rows.py scripts/mutation/mutation-prune.py scripts/gate/lib_libtest_result_lines.py scripts/crash/lib_verdict_library.py scripts/hosts/run-with-memory-cap.sh`
列出 021b4db0、1b6f9865（10-14）、e0bddcba、8c07d1f6、503b80e2（10-11）、53d1154a（10-10）共 6 次（量的）；这几次是门禁改名、脚本搬家、加自证格，被测的 crates 源码与变异表的行不一定变（推的）。
按用户口径：阶段脚本、内存包装换了写法不改变「这条变异被哪条测试抓到」，不该整库作废。

另记（量的，原因没查）：`.git/singlefs-mutation-rows/37ab6ce3…/` 现在只有 246 个 `mutation.*` 文件，日期都是 10-13；那一趟日志说复用 1693、现跑至多 2260。三个版本目录：37ab6ce3（246 条）、3a63d840（391 条）、d0ef8d8c（185 条）。
旁证（轻型 --quick，不在射程）：10-13、10-14 四趟 --quick 的判法版本各是 f8ab002220d76b3a、3a63d8405e4ce1fb、1993741381afb10e、3a63d8405e4ce1fb（`detached-runs/owed-c456-quick-2/log`、`-3/log`、`mutation-light-2026-10-13.log`、`two-stage-validate/round3/v3.log`），两天四把（量的）。

### 2.6 checker-tier-formal-verification

| 格 | 库在哪 | 键的组成 | 作废粒度 | 合理与否 | 依据 |
|---|---|---|---|---|---|
| verus-toolchain-pinned / verus-proof / verus-vacuity / verus-must-red-worlds | 格级全绿标记 `.git/singlefs-stage-green.checker-tier-formal-verification.sh:<格>.<指纹>`（另有 `.git/singlefs-formal-verdicts/{proof,vacuity,must-red-worlds}`） | 各格登记的 10–11 个文件（`crates/singlefs-replay-prefix/`、verus 钉表、判法脚本、内存包装、`lib_verdict_library.py`…）+ verus 版本 | 一格整格 | 基本合理（输入窄）；但内存包装与 `lib_verdict_library.py` 在每格键里，改它们四格全重跑 | `stage-inputs.tsv:39`–42（量的） |
| pinned-red-examples-complete | 无库 | — | 每趟现判 | 按设计（便宜） | 日志「每趟现判、不复用」（量的） |
| pinned-red-examples-replay | `.git/singlefs-formal-verdicts/pinned-red-examples-replay/` 按条 | 判法版本 = `e165-pin-red-examples.py` + `lib_verdict_library.py`；输入指纹共用一样「装置」= e165 装置 bin 的 cargo 依赖闭包（按 crate 目录、逐文件原文）+ `Cargo.toml` + `Cargo.lock` + 工具链；每行另加这一行、档、引的变异行 | 「装置」一变整格 360 行 | 键过宽：依赖闭包按 crate 目录收、按原文取（注释也算），闭包里任何 crate 的任何源文件变了 360 行全现判 | `scripts/experiments/e165-pin-red-examples.py:92`–97（量的） |

近一周只有一趟留下日志（10-13，`commit-2026-10-11/checker-tier-formal-verification.log`，原样）：
- 第 1 行：`── 按格复用判定：6 格里 4 格复用、2 格重跑（每格的输入登记在 .claude/gate.d/stage-inputs.tsv，键 checker-tier-formal-verification.sh:<格名>）`
- 第 377 行：`  pinned-red-examples-replay：现判 360、复用 0（现判的原因：装置 变了 360）`
- 第 378 行：`── 按条（1 格有逐项判定；verus-toolchain-pinned 与 pinned-red-examples-complete 每趟现判、不复用）：现判 360、复用 0`
- summary.txt：`checker-tier-formal-verification rc=0 秒=225`、`checker-tier-formal-verification rc=0 秒=55`、`checker-tier-formal-verification rc=0 秒=307`（前两趟的日志被覆盖，复用数没留下）

复用为 0 的那一趟前面改了什么：上一趟（暂存树 d83fe646）→ 这一趟（1722e788），crates 下只动了 `crates/singlefs-harness/src/read_tally.rs`（+112）与 `crates/singlefs-harness/tests/presumed_clause_checks.rs`
（`git diff --name-only d83fe646 1722e788 -- crates`，量的）。`git grep -l read_tally 1722e788 -- crates` 命中的全在 singlefs-core 与 singlefs-harness，checker 档包里没有一份引它（量的）。
按用户口径：E165 装置读不到的 harness 新模块，不该让 360 行全现判。代价小（每块 1.7 秒、整趟 307 秒），不痛。

### 2.7 checker-tier-broad-mutation（驱动 `scripts/mutation/broad-mutation.py`、`mutation-prune.py`）

| 格 | 库在哪 | 键的组成 | 作废粒度 | 合理与否 | 依据 |
|---|---|---|---|---|---|
| 结论表 | `~/.local/share/singlefs/broad-mutation/conclusions.tsv`（跨版本） | 键 =（文件、限定名、函数内相对起止、替换文）；另存抓到它的测试 fn 指纹、fn 项与函数外部分的词法指纹、宏与 include 依赖指纹、全局键 | 按函数剪枝（这一样合理） | 全局键过宽：`Cargo.lock`、每份 `Cargo.toml`、`.cargo/`、构建环境、工具版本、排除表、限时参数整份进；判法版本 `JUDGE_RULES_VERSION` 一换「一条都不沿用，全部重判」 | `broad-mutation.py:55`–66、`:180`–185（量的） |
| 这一版的结果表 | `versions/<指纹>/` | 这一版的指纹 = sha256(源码指纹、rustc -Vv、判法版本、全局键) | 源码一变就是新一版，靠结论表沿用 | 同上 | `broad-mutation.py:55`（量的） |

近一周：没查到日志。找过：`~/.local/state/singlefs/detached-runs/broad-*`（最后是 10-01 的 broad-mutation-local / -peer）、`commit-2026-10-11/` 下没有 `broad-marker.log` 与 `broad-mutation-run.log`（run.sh 第 57、71 行写这两份，量的：`ls` 没有）、`~/.local/share/singlefs/broad-mutation/`。
最近一趟（10-01，`detached-runs/broad-mutation-local/log`）的计数（`grep -o 'outcome=[a-z_]*' | sort | uniq -c`，量的）：caught 131、unviable 6、timeout 6、stage 3，共 146 行 `BROAD_MUTATION_ITEM`，末尾 `  ✗ 被 TERM / INT 停了：在途那一条没落行，在途那一步已一起停掉`。
库里现状（量的）：`conclusions.tsv` 只有 2 行数据；两个版本目录 19c45e02…（rules prune-r3-e1-e6）与 e1ba2574…（rules prune-r3-e1-e6-fresh-build），都建于 10-01；
而驱动此刻的判法版本是 `prune-r3-e1-e6-fresh-build-mac`（`:185`），即下一趟起这两版的结论一条都不沿用（量的：文件头原话「它之前落的结论 … 一条都不沿用，全部重判」）。
按用户口径：换判法版本（加校验列 mac）的理由是防手写行，不是判法变了；结论本身的对错没变，整版作废不合理（推的）。

### 2.8 checker-tier-research-build-and-replay：experiment-replay 与 mutation-tables-compile

| 格 | 库在哪 | 键的组成 | 作废粒度 | 合理与否 | 依据 |
|---|---|---|---|---|---|
| experiment-replay | `${REPLAY_VERDICT_LIBRARY:-<git common-dir>/singlefs-replay-verdicts}/<键>/` | 每行：装置源码（源码摘要，注释不算）、复跑表这一行、判一行的函数、留存产物、工具链 | 按行 | 设计上是按行的；但库从没写过（目录不存在） | `scripts/experiments/replay.sh:33`–39（量的） |
| mutation-tables-compile | `${GATE_CROSS_RUN_TMPDIR:-${TMPDIR:-/tmp}}/singlefs-mutation-tables-compile-verdicts/` | 每张表：表、源文件、它 include 的文件、`scripts/experiments/` 的 Cargo.toml、Cargo.lock、lib.rs、`mutate.sh` 各自的 sha256 + 工具链；判法版本 = 这份阶段脚本 + `lib_verdict_library.py` | 判法版本或 `mutate.sh` 一变 164 张全作废 | ①库在 /tmp，会被清掉；②阶段脚本（五格共一份）进判法版本，改别的格也整格作废；③`mutate.sh` 每张表都带 | `.claude/gate.d/checker-tier-research-build-and-replay.sh:48`–51、`:458`（量的） |

experiment-replay 近一周：没查到日志。找过：`grep -rl '^REPLAY_TOTAL dispatched'` 于 `research/results/`、`~/.local/share/singlefs/`、`~/.local/state/singlefs/`、`/tmp/claude-1000`，0 份；detached-runs 里没有一趟起 experiment-replay；
`ls -d /home/fy5090/code/singlefs/.git/singlefs-replay-verdicts` 回 `No such file or directory`（量的）。即这一格的按行库一次都没落过盘，下一趟全量必定全部现跑（推的）。

mutation-tables-compile 近一周四趟（`detached-runs/owed-c79*-full-cell*/log`，原样）：
- owed-c798-full-cell（10-13）：`  ✗ 研究侧变异表有 2 条替换文编不过、0 张表没判完（现判 164 张、复用 0 张）`，原因分栏「现判（无记录）」130 张、「现判（mutate.sh、判法版本 变了）」34 张
- owed-c798-full-cell-2（10-13）：`  ✓ 研究侧变异表逐条只编译全编得过：判了 164 张表，现判 2 张、复用 162 张（现判的共 8 条）；没跑 20 张（上面「本次未跑」逐张列名）`
- owed-c798-full-cell-3（10-13）：`  ✓ 研究侧变异表逐条只编译全编得过：判了 164 张表，现判 164 张、复用 0 张（现判的共 2089 条）；没跑 20 张（上面「本次未跑」逐张列名）`，164 张原因全是「现判（mutate.sh、判法版本 变了）」
- owed-c795-full-cell（10-13）：没有计数行，`  mutation-tables-compile：红（退出 1）`

复用为 0 的那一趟（c798-3）前面改了什么：原因栏写明 `mutate.sh` 与判法版本（阶段脚本或 `lib_verdict_library.py`）变了（量的）；两趟之间是 owed-c798 那一批修 E142 表与 `mutate.sh` 的 cap-lints（`detached-runs/owed-c798-e142-table/command` 写 `research/results/e142-mutate-cap-lints-2026-10-13.log`，量的；改动是在工作区里、没单独提交，具体哪几行没量）。
按用户口径：`mutate.sh` 加 `--cap-lints` 对「替换文编不编得过」可能有影响，重判一次说得过去（推的）；但阶段脚本改别的格也让这 164 张全作废，不合理。
库现状（量的）：`find / -maxdepth 6 -type d -name singlefs-mutation-tables-compile-verdicts` 一个都没找到；`commit-2026-10-11/clean-build-residue-local.log` 第 208 行在 10-14 还列着 `   0.0G /tmp/singlefs-mutation-tables-compile-verdicts`。下一趟这一格从 0 开始（推的）。

## 3 不合理清单

| 编号 | 哪道 | 哪一种 | 现象（量的 / 推的） | 最小改法（方向与落点） |
|---|---|---|---|---|
| U1 | 崩溃点重放、崩溃放量 | 键过宽 | 判器闭包在 10-11 到 10-14 收进流水线 `crash_amplification.rs`、`crash_judge_dispatch.rs`、存储 `verdict_store.rs`、`crash_library.rs`（68 个文件）；近一周三次整库掉成 0 都由这类改动触发（量的，§2.2） | 闭包停在流水线与存储之前（10-14 的 53 份已部分做到，要在日志里核下一次全量不再随流水线换版本）；落点 `crates/singlefs-checker-tier/src/judging_closure.rs` 的 `JUDGE_CLOSURE_STOP_FILES` / `FILES_THE_JUDGES_MUST_NOT_REACH` |
| U2 | 同上 | 键过宽 | 53 份里仍有 `singlefs-core` 32 份（`src/` 共 34 份 .rs，量的：`ls … | wc -l` 与 `grep -c` 数的），含写路径（admission、allocator、transaction、make_filesystem、unlink、write_request_split）；闭包按文件、顺 `use` 收，不按判器真调到的函数（量的清单，判器调不调这些函数是推的） | 判器那一侧按 fn 项收（照 `mutation-prune.py` 的 fn 项切法），只收恢复、checker、oracle 真调到的函数；落点 `judging_closure.rs`、`crash_identity.rs:512` |
| U3 | 同上 | 键过宽 | wgsl 按原文进摘要，b5a35b44 → f1985f9f 三份 wgsl 只改了注释（量的） | wgsl 也去注释、去空白再摘要；落点 `crash_identity.rs`（`include_str!` 原文那一支）与 `crates/singlefs-checker-tier/build.rs` |
| U4 | 同上 | 键过宽 | 判法版本另拼整份 `Cargo.lock` 与各 crate `Cargo.toml` 原文（量的，`crash_identity.rs:512`）；依赖升级一次整库作废（推的，近一周没观测到） | 只拼闭包里各 crate 用到的那几个依赖条目；落点同上 |
| U5 | 同上 | 整库作废 | 判法版本一变整库不复用（`verdict_store.rs:24`「复用只认同一判法版本」）；切片分支的改动让没登记切片的八个分项全重判（量的 diff 与 `registered_segments=-`，影响面推的） | 换版本时先在新旧两版上对拍一批块，判定向量逐字节相同就把旧行标成新版本可用、记一行对拍证据，不同的那几类块才重判；落点 `verdict_store.rs`、`crash_library.rs` |
| U6 | 崩溃点重放 × 崩溃放量 | 键过窄（重复判） | 同样 1678377204、1358954634 个状态两库各判一遍；崩溃点重放 10-12 22:58 那一片单片 9286 秒（量的） | 块键去掉流名与类名，只留流程内容（起步镜像、写表、枚举计划），两类共一个库；落点 `verdict_store.rs` 块键、`crash_library.rs:132` 的类与目录 |
| U7 | 崩溃点重放 | 库位置不稳 | 一周里库目录换了三次：`library-by-conclusion-kind` → `layer0-enumeration-library` → `crash-state-enumeration-library`，旧库 624M 留在原处没人读（量的） | 改名时搬库（或库名不带类名、不随术语改名）；落点 `crash_library.rs:132`、`scripts/crash/crash-point-replay-shard-run.sh:85` |
| U8 | 崩溃点重放 | 键过宽 | 用例级标记键 = 整个 `crates/`；10-14 起每条用例「421 个文件，减去用例读不到的 0 个」，`mutations.tsv` 与 `src/bin/` 也在里面（量的；原因推的，见 §2.1） | 用例输入改成测试目标的 cargo 依赖闭包（`stage-inputs.tsv` 已有的 `closure=` 那一套），算出来的 include 认 `concat!(env!("CARGO_MANIFEST_DIR"|"OUT_DIR"), "<字面>")` 这种可解析的写法；落点 `scripts/gate/admission.py:1405`–1460、`crates/singlefs-harness/src/heavy_threads.rs:22` |
| U9 | crates 变异整表 | 键过宽 | 判法版本收阶段脚本、`crates-mutation-rows.py` 整份、内存包装、`lib_verdict_library.py`；10-12 那一趟跨趟复用 0（量的）；整表按用时估约 98368 + 98365 秒（日志 ③ 的估算值，不是实测挂钟） | 判法版本只收决定「抓没抓到」的那几段（libtest 结果行解析、切 fn 规则），内存包装、存储共用库、阶段脚本的调度部分不进；落点 `scripts/mutation/crates-mutation-rows.py:602`–620 |
| U10 | crates 变异整表 | 键过宽、删得太早 | 行键带每条内存上限与限时（`:61`），换一个上限整表重判；只留最新 3 个版本目录（`:84`），改回去找不回 | 「抓到」记录的复用不看上限与限时（它们只影响超时 / 撞顶那两档，这两档本来就不复用）；版本目录按大小或日期留，不按个数；落点同上 `row_key`、清理那一段 |
| U11 | 形式化 pinned-red-examples-replay | 键过宽 | 装置键 = bin 的 cargo 依赖闭包按 crate 目录、逐文件原文；harness 新模块（checker 档没有一份引它）让 360 行全现判（量的） | 装置键改成 bin 的模块闭包、去注释取词法（与判器闭包同一套）；落点 `scripts/experiments/e165-pin-red-examples.py` 的 `device_inputs_digest`、`admission.py` 的 `cargo_closure_input_paths` |
| U12 | mutation-tables-compile | 库会丢 | 库在 `${TMPDIR:-/tmp}`，10-14 还在、现在全盘找不到（量的） | 库挪到 git common-dir 或 `~/.local/share/singlefs/` 下；落点 `.claude/gate.d/checker-tier-research-build-and-replay.sh:458` |
| U13 | mutation-tables-compile | 键过宽 | 判法版本 = 整份阶段脚本（五格共用）+ `lib_verdict_library.py`；c798-3 一趟 164 张全现判（量的） | 判法版本只收这一格的判法函数原文；落点同文件 `:48`–50 与它算判法版本的那段 |
| U14 | experiment-replay | 没有库（实际上） | 按行库目录 `.git/singlefs-replay-verdicts` 不存在，近一周没跑过（量的） | 不是键的问题：下一次全量跑完要核库落了盘、`REPLAY_TOTAL` 的 reused 能大于 0；落点 `scripts/experiments/replay.sh` |
| U15 | 四道共用 | 键过宽 | `lib_verdict_library.py` 同时进 crates 变异、形式化（verus 四格与 E165）、mutation-tables-compile 的判法版本（量的，三处文件头）；改一次存储小工具，三道的库一起作废（推的） | 存储共用库不进任何判法版本（它管怎么存，不管判对错）；落点 `crates-mutation-rows.py:239`、`e165-pin-red-examples.py` 的 `judge_version`、研究阶段脚本 `:49`、`stage-inputs.tsv:39`–42 |
| U16 | 广谱变异 | 整版作废 | 判法版本换成 `prune-r3-e1-e6-fresh-build-mac`（加校验列），旧结论「一条都不沿用」（量的，`broad-mutation.py:185`）；全局键整份收 `Cargo.lock` | 校验列与判法版本分开：旧行用密钥补签后沿用；全局键只收被测包的依赖条目；落点 `broad-mutation.py` 的 `JUDGE_RULES_VERSION` 与 `global_key_lines` |
| U17 | QEMU、LKMM | 没有库、整道级 | 只有整道全绿标记；键含整个 `crates/`（QEMU 另含 `scripts/`、`research/results/`）；LKMM 两次重跑前 litmus 与判法 0 处改动（量的） | 按格拆：QEMU 每次虚机跑一格、键取它那个装置的闭包；LKMM 每份 litmus 一格、键取 litmus 文件与它绑到的代码文件；落点 `.claude/gate.d/stage-inputs.tsv:37`–38 与两道阶段脚本 |
| U18 | 所有整道键 | 键过宽 | `stage-inputs.tsv` 整份无条件进每一道整道键（`stage-inputs.tsv:30`–31 原话），加一个实验行也让整道重跑（量的条文；近一周 d83fe646 → 1722e788 之间它变过，LKMM 那一趟同时也有 crates 改动，分不开是谁触发的） | 整道键只收这一道自己的登记行（格级已经这样做）；落点 `scripts/gate/stage-must-run.sh`、`admission.py` 的 `stage_fingerprint` |
| U19 | crates 变异分片 | 不是键的问题 | 第二台缺 llvm-tools，1976 行一条没跑，整张又在本机跑一遍（量的，§2.5） | 分片前先在第二台查 llvm-tools，查不过就不分给它；落点双机分片驱动 |

键过窄会漏判的：这一轮按日志没找到实例。核过的一处：experiment-replay 的装置键只收 `scripts/experiments/` 里的源码，`scripts/experiments/Cargo.toml` 没有一条 `singlefs-` 依赖（`grep -n singlefs- scripts/experiments/Cargo.toml` 只命中两行注释，量的），所以不会因为 crates 改了而漏跑。别的门禁没逐个找漏判。

### 最该先改的三条

1. **U1–U5（判器闭包与整库作废）**：受影响的是最贵的两道。崩溃放量一次整库作废要重判 16 亿 + 13 亿 + 14 亿级的状态，10-12 那一趟四个多小时（`full-summary.txt` 两行时间，量的）。
   近一周三次掉成 0，全是这把键触发的，里面至少有一次（f0b9512b）主要是注释改动加一个提速提示。先做 U3（wgsl 去注释）与 U1（停在流水线之前）两样，代价最小。
2. **U9、U10、U15（变异库的判法版本收了脚本与内存包装）**：整表按用时估约 19.7 万秒（日志 ③ 两片的估算值相加，不是实测），而判法版本随任何一次门禁脚本、内存包装、存储共用库的修改整库换；
   近一周这几份文件在 6 个提交里被改过（量的）。这几份文件改得最勤，按用户口径又都不改「抓没抓到」。
3. **U6、U7、U12（库放错地方、两库重复判）**：不用动判法就能拿回来的复用。改名把 624M 的库丢在一边、/tmp 的库已经没了、同一批状态两边各判一遍（单片 9286 秒），
   改的都是路径与块键里的一个名字字段，影响面小。

## 4 推翻条件

- 「三次掉成 0 都是判法版本换了」：同一判法版本下出现 `states_reused=0` 而 pending 等于全量的一趟，就推翻。`entry-table.tsv` 里展开 30 的各趟没有这样的行（量的）。
- 「判器闭包在 10-11 到 10-13 收了 `crash_amplification.rs`」：那几棵暂存树里 `crash_judge_gpu.rs` 不引 `crate::crash_amplification`、或闭包算法会把它停在门外，就推翻。
  我量的只是 `git show <树>:crates/singlefs-checker-tier/src/crash_judge_gpu.rs` 里有这条引用，闭包清单 68 份那一份是 10-14 的日志，不是当时的；当时的清单没量，所以这一条是推的。
- 「用例级指纹减 0 个是 `heavy_threads.rs:22` 引起的」：在副本里把它换成字面量路径后 `admission.py paths` 仍减 0 个，就推翻（命令见 §5）。
- 「mutation-tables-compile 的库丢了」：在任何一个 `TMPDIR` 下找到 `singlefs-mutation-tables-compile-verdicts`，就推翻；我只查了 `find / -maxdepth 6`。
- 「崩溃放量与崩溃点重放判的是同一批状态」：只量了状态数相等。两边块键里的输入指纹与枚举计划哈希不同（例如崩溃点重放的流多一步或少一步），就推翻。

没在副本里造推翻现象：这一轮派发写的是只读、不起 cargo 与门禁，造不了（例如改闭包后看版本变不变要编译）。

## 5 要主 agent 跑的命令（核 U8 的原因；不是重型，但会写源码摘要缓存，所以这一轮没跑）

工作目录：`~/.local/share/singlefs/work/audit-reuse-keys-by-logs/`（草稿目录），不设环境变量前缀，不起 cargo，内存与线程不涉及。

```
rsync -a --exclude target --exclude .git /home/fy5090/code/singlefs/ copy-a/ && git -C copy-a init -q && git -C copy-a add -A
python3 /home/fy5090/code/singlefs/scripts/gate/admission.py crash-case-manifest copy-a crash-case:floor-raise-pushed-by-the-session manifest-a.txt --judging-digest --toolchain --build-environment
# 再把 copy-a/crates/singlefs-harness/src/heavy_threads.rs 第 22 行起的 include_str!(concat!(…)) 换成 include_str!("../../../.claude/heavy-thread-default")，git add 后重跑同一条，输出写 manifest-b.txt
```

看哪几行：stdout 那一行「<指纹> <文件数> <减去的文件数>」。第一次减去 0、第二次减去大于 0 ⇒ U8 的原因坐实；两次都是 0 ⇒ 推翻，原因在别处（下一个候选 `crash_identity.rs:661`）。
（`crash-case-manifest` 的用法照 `scripts/gate/admission.py` 文件头第 137–140 行；副本要是 git 工作树，因为输入按 git 列文件。）

## 6 没做什么

- 没跑任何重型与轻型测试、门禁阶段、cargo、admission.py 的子命令；没在副本里造推翻现象（派发写明只读）。
- 判器闭包在 10-11 到 10-13 各棵暂存树上的文件清单没算（要编译或跑闭包算法），只拿 10-14 两份日志的 68 / 53 份清单与那几棵树里的 `crate::` 引用对照，§4 已标推的。
- b5a35b44 → f1985f9f 的 29 份闭包文件只逐行看了 7 份，另 3 份（`recovery.rs`、`block_device.rs`、`walk.rs`）只看了非注释行。
- 判定库内容（RocksDB 的 `09` 表、各版本的块数）没读，只看了目录、大小、生日与日志里的计数。
- crates 变异库 37ab6ce3 现在只有 246 条记录、与日志里的 1693 / 2260 对不上，原因没查。
- QEMU、LKMM、形式化前两趟、崩溃放量 10-13 之前各趟的入口汇总被 `>` 覆盖了，复用数只能从 `runs/entry-*`、`summary.txt` 取；崩溃放量 10-11 之前的各趟（10-08、10-10）只在 `entry-table.tsv` 里，正文没逐趟写。
- 轻型 --quick 的各趟（two-stage-validate、crash-point-replay-quick-after-merge、owed-c456-quick*）不在射程，只当旁证引了判法版本。
- 正在跑的 full-2026-10-15（pid 3461100 起的那一条）读的是它此刻的半截日志，跑完之后的数没有。
- 仓里的文件一处没改；草稿目录里只有日志摘录、两个表、闭包清单与三份解析脚本，没有仓副本与编译目录。
