# 第二轮复查（路子 verify）：在副本上造改动，核底稿 29 条机制的作废面

日期 2026-10-15。底稿 `~/.local/share/singlefs/work/survey-pruning-ledger/ledger-r1.md`（读的是 88 行那一版，已含 gaps 路补的 L30、L31；这一轮只核 L1–L29）。
只读主工作区：没改仓里任何文件，没动判定库，没起门禁、测试二进制、cargo。全部造改动都在草稿目录的仓副本 `copy/`（`rsync -a --exclude target --exclude .git`，另排除 `research/prompts`、`multi-host.env`、`qemu.env`、`target-*`、`.claude/settings.local.json`；副本里 `git init`，源码摘要缓存与造的 `refs/sop/staged-green` 都只在副本的 `.git` 里）。
标注：「量的」= 这一轮在副本上跑出来的原样输出（文件在 `out/`），或现读的那一行代码；「推的」= 由代码推出、没观测过。

## 0 结论

- 29 条里 27 条核过（在副本上造了现象或读码现查）、2 条只核了一半（L3 块键那一半、L6 整条：要跑 checker 档 Rust 引擎，这一轮不许编不许跑）。没有一条整条判错；要改写的 6 处见第 3 节。
- 新找到、底稿里没有的四样（都是量的）：
  1. `singlefs-harness` 整个 crate 273 份 `.rs` 在门禁源码摘要里一律按原文算（注释也算），起因是 `crates/singlefs-harness/src/heavy_threads.rs:22` 的 `include_str!(concat!(…))` 触发 `scripts/gate/lib_source_digest.py:516-517` 把整个 crate 记成读原文。只往 `crates/singlefs-harness/src/memory_pool.rs` 末尾加一行注释，18 把门禁键里 14 把换掉；把那一处换成字面量路径后同一改动 0 把换。底稿 L14 只写了「文件级」，要补「harness 整 crate 原文」与这一处落点。
  2. 广谱变异「抓到」沿用：不止测试调的辅助函数改了照沿用（B8），**被变异函数自己调到的函数**改了也照沿用（`index_node_entry_capacity` 调的 `index_node_header_bytes` 改了，`carry_decision` 交 `carried`）。变异自己的指纹只罩它所在那个 fn 加文件的函数外部分。L22 重做方向里「被变异的 fn 单元指纹」要写成「被变异 fn 与它调到的单元」，或明说靠「抓它的测试执行过的单元集合」罩住。
  3. 双机分片驱动（K7）：只改注释**不**作废（shell 摘要去注释），加一个没人调的 shell 函数才作废；登记了 `shard=across-machines` 的是 4 条用例，不是 3 条。底稿 L17 引 K7 的那半句要改。
  4. 实验准入指纹（L26）按原文取，不按源码摘要：只往 core 的 `transaction.rs` 加一行注释，E142 与 E142/p8g-crates-dump 两把都换，同一改动门禁 18 把一把不换。
- 主工作区此刻 `crates-mutation-rows.py judge-version` 打 `c6d72979…`，盘上 `.git/singlefs-mutation-rows/` 三个目录（`37ab6ce3…`、`3a63d840…`、`d0ef8d8c…`）里没有它：下一趟 crates 变异复用 0 条（量的）。

## 1 副本、工具与跑法

- 副本：`/home/fy5090/.local/share/singlefs/work/survey-pruning-r2-verify/copy`（571M），交回前删，大小见第 8 节。改动一律经 `tools/edit2.py`（备份放副本外的 `backups/`），每格之后还原；收尾 `rsync -an --checksum --itemize-changes` 对主工作区：30 行，全是时间戳行（`.f..t`），内容差只有 `.claude/kb/checks-owed.md`（别的会话在主工作区改的，`>fcst`），即副本里没有残留的造改动（量的，`out/copy-vs-main.txt`）。
- 工具（都在 `tools/`）：`fp_all.sh`（副本上每道 `admission.py stage-fingerprint`、两条 `crash-case-manifest`、E142 两把 `experiment_fingerprint`）、`closure_port.py`（crates 路的移植闭包算法，sha256 `8c0cd2be…`，与 r1 那份逐字节同）、`closure_scenarios*.sh`、`carry_probe.py` / `prune_probe.py`（只 import 副本的 `mutation-prune.py`）、`selection_probe.py`（驱动 `run_items` 的选条段，Runner / Pruner 换替身，`select_items` 交回就停，不起 cargo、不碰真库目录）、`judge_version_probe.sh`、`keys_probe.py`、`reuse_probe.sh`、`replay_scenarios.sh`、`shard_cases.sh`、`judging_digest_cases.sh`、`e165_device_probe.py`、`digest_kinds.py`。
- 起法：除第 7 节列的几条外，都经 `nice -n 19 bash scripts/hosts/run-with-memory-cap.sh 8G …`；没起重活，没用到线程上限。开跑前 `ps` 看到 224 个 cargo / rustc 进程（别的会话在编），没有 `qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`。
- 移植闭包算法这一轮在主工作区（只读）重核了一遍：打出的表与 `target/release/build/singlefs-checker-tier-d6a124019c061d7a/out/conclusion_code_digests.tsv`（表头 `# singlefs conclusion code digest table 1`）各 137 行，按（种类，摘要）比 `diff_rc=0`、差 0 行（量的，`out/table-diff.txt` 空）。
- 判定复用的基线稳不稳：`fp_all.sh` 在没改的副本上连跑两次逐字相同（`base_stable`），`git add` 进副本索引之后再跑一次仍逐字相同（`base_same_after_index`）。

## 2 逐条：底稿 29 条每条一格

「输出」列指第 4 节的原样输出块（块名即 `out/` 下的文件名）。「底稿」列：对 / 错 / 要补，要补的写成什么。

| 号 | 核 | 造了什么（命令见第 4 节） | 看到的 | 底稿那一行 |
|---|---|---|---|---|
| L1 | 核过（量的，移植算法） | `crash.rs` 那个 `#[cfg(test)]` 模块里已有单测 `sparse_device_reads_zero_for_holes_and_flip_byte_only_touches_one_sector` 体内加一句 `let _probe = 0;`；core `unit.rs:576` 的 `#[cfg(test)]` 里单测体内加同一句；对照：同一处只加一行 `//` 注释 | 两处改单测，GPU、CPU、观察者三把全换；只加注释三把都不换（`p6-closure`） | 对。补一句：不止「加空单测」，改已有单测体、改 core 里的单测同样三把全换 |
| L2 | 核过（量的，移植算法） | `crash_judge_recovery.wgsl` 末尾加 `// probe comment` | 只 GPU 那把换（`d177…`→`f7e6…`），CPU、观察者不换（`p6-closure`） | 对 |
| L3 | 核过一半 | 闭包摘要那一半：`crates/singlefs-format` 整个目录换名、内容不变 | 三把全换，137 行的表里 134 行换（`p6b-closure`）。块键节点名那一半（`crash_amplification.rs:399` `block_key_under`、`:12126` 给定状态节点名）只读码，没实跑 Rust | 对。块键那一半盘上的数由 disk 路量过（4100 块、9679 块），这一轮没重量 |
| L4 | 核过（量的，移植算法） | `Cargo.lock` 末尾加 `# probe comment`；core 的 `Cargo.toml` 末尾加同样一行 | lock：三把全换；core `Cargo.toml`：三把全换、表 134 行换（`p6-closure`、`p6b-closure`） | 对 |
| L5 | 核过（量的，移植算法） | 设点文件 `tests/crash_enumeration_tree_split_streams.rs` 末尾加一个没人调的 fn；对照：另一份测试文件 `tests/crash_enumeration_fixed_script_stream.rs` 加同样的 fn | 前者：CPU、观察者换，GPU 不换，表 2 行换；后者：tree_split 那三把都不换，表 2 行换（那一份自己的两行）（`p6b-closure`） | 对 |
| L6 | 没核（实跑） | 要跑 checker 档引擎算块键（Rust，派发不许编不许跑）。读码现查：给定状态输入串 `crash_amplification.rs:12094-12100` 把 `item=`、`seeds_and_scale=`、`table=` 原样拼进被 sha256 的串；`tests/crash_injection_campaign.rs:81` `seeds_and_scale_of` 把 `seed_count`（`:85`）写进串 | 读码：同一批里多加一个种子，`seed_count` 变，这一批**每一张**表的输入指纹都变（推的，format! 原样拼入、sha256 无例外）；整条流一节点盘上 disk 路量过 | 对。补一句：`seed_count` 进每张表的键，加一个种子整批全换，比「内容没变的表全换键」更宽 |
| L7 | 核过（读码 + 量的） | `grep` 列手写号；移植算法往停止文件 `crash_amplification.rs`、排除文件 `crash_judge_dispatch.rs` 各加一个没人调的 fn | 手写号原样在：`crash_judge_gpu.rs:504` `GPU_JUDGE_REVISION: u32 = 1`、`:537` `gpu-crash-judge-v7`、`crash_amplification.rs:14060` `= 2`、`:3937` `cpu-facts-verifier-v2`、`:12095`、`:12136`、`:14387`、`crash_identity.rs:604`、`judging_closure.rs:783`、`crash_judge_tables.rs:54` `= 9`、`crash_facts.rs:35` `= 2`、`crash_verify_gpu.rs:364` `gpu-facts-verifier-v4`；两处加 fn 三把都不换、表 0 行换（`p6b-closure`）；`gpu_judge_digest` 只在测试 `:2714` 断言非零，`tables.digest()` 只在测试 `:247` 断言非零 | 对。行号更正：`crash_verify_gpu.rs:361` 改 `:364`（那一行才是手写前缀） |
| L8 | 核过（读码） | 现读 `crash_amplification.rs:3934-3941`、`:4043-4050`、`crash_facts.rs:20` | 核对键只拼 `cpu-facts-verifier-v2` 与 `include_str!("crash_facts.rs")`；`crash_facts.rs:20` `use singlefs_checker::{check_root_slot, check_unit, crc32_castagnoli_table};` 不进键；读库只看 `.is_some()` | 对。没造实例（要跑 Rust） |
| L9 | 核过（读码 + 量的） | 现读 `crash_identity.rs:771-783`；移植算法看「改一份无关测试文件」表换几行 | 比的是两边全部入口的并集（`every_entry`），任一行不等就交非空、`crash_amplification.rs:12951-12957` 整趟不写（`:11861` 给定状态引擎同一调用）；无关测试文件加 fn 表换 2 行（`p6b-closure`）⇒ 这一趟所有项都不写（推的，Rust 没跑） | 对 |
| L10 | 核过（读码） | 现读 `verdict_store.rs:16-24`、`crash_amplification.rs:1644-1647` | 表 `01`/`04`/`09` 键带「判法版本」；`TheJudgeVersion` 注释「直接调流水线各段的对拍用例与单测用它；run_from_environment 不用」 | 对。没实跑 |
| L11 | 核过（读码） | 现读 `scripts/crash/crash-amplification-item.sh:762-765` | `:764` `crash_amplification_record_on_this_host "$local_library" "$run_directory/record.log"; record_exit=$?` 前面没有任何条件 | 对 |
| L12 | 核过（ls / du，只读） | `ls`、`du -sh ~/.local/share/singlefs/crash-amplification/*` | 12 份库目录在（`library` 272M、`layer0-enumeration-library` 624M、`given-states-library.polluted-…` 104M 等，见第 4 节 `ls-libraries`）；另有 `runs/` 436 个目录共 89G | 对。补：`runs/` 89G（K5 每趟建一份没人读的库的累计） |
| L13 | 核过（读码 + 量的） | `grep include_str!(concat!`；`digest_kinds.py` 把 `heavy_threads.rs:22` 换成字面量前后各数一次 | 两处：`crash_identity.rs:661`、`crates/singlefs-harness/src/heavy_threads.rs:22`；后者让门禁源码摘要把 harness 整 crate 记成读原文（273 份 raw → 换掉后 272 份 rust、1 份 raw）（`digest-kinds`） | 对。补：门禁侧 `lib_source_digest.py` 对算出来的 include 是「整 crate 退原文」（过宽，不是漏），与 crates 侧闭包「不进闭包」（漏）两种口径相反，合成一份算法时要定一种 |
| L14 | 核过（量的） | 副本 `.claude/gate.d/stage-inputs.tsv` 末尾加一行 `#` 注释；core `transaction.rs` 只加注释；harness `model.rs` 加一个没人调的 fn；harness `memory_pool.rs`、checker-tier `crash_facts.rs` 只加注释；`admission.py` 末尾加一个没人调的 def | 登记表加注释：8 把整道键全换，6 格形式化、`mutation-tables-compile` 格、两条崩溃用例、两把实验都不换；core 只加注释：18 把门禁键 0 把换；harness 加 fn：14 把换；harness 或 `crash_facts.rs` 只加注释：同样 14 把换；`admission.py` 加 def：崩溃点重放整道键换、用例键不换（`p1-tsv-comment`、`p3-table`、`p-judging`） | 要补：「`.rs` 去注释」只对没有算出来的 include 的 crate 成立；harness 整 crate 与被 `include_str!` 字面读的 `.rs`（`crash_facts.rs`、`crash_amplification.rs`、`gpu_unit_checks.rs`、`model.rs`）按原文进键。落点加 `lib_source_digest.py:504-517`（`DigestContext.add_includer`）与 `heavy_threads.rs:22` |
| L15 | 核过（量的） | 副本里把同一棵树既当 `refs/sop/staged-green` 又当 `SINGLEFS_STAGED_TREE`，PATH 前插一个 `rustc -V` 打 `9.99.0` 的假 rustc，问 `admission.py gate-reuse` | 没登记 `environment=` 的三道（模型对拍、崩溃点重放、红例 replay 格）真假 rustc 都退 10（可跳过），而 `stage-fingerprint` 随假 rustc 换；登记了 `environment=` 的 LKMM 退 0（环境没记下，要跑）（`p-reuse`）；`change-touches-crates.sh:57` `printf 'HEAD'` | 对 |
| L16 | 核过（量的） | 副本 `.cargo/config.toml` 末尾加 `# probe comment`；树对比那一路换 rustc（同 L15） | 18 把门禁键全换（配置取原文）；树对比那一路不看工具链（`p3-table`、`p-reuse`） | 对 |
| L17 | 核过（量的） | 判法函数 `judge_worker_threads` 体内加一行注释、对照 `experiment_fingerprint` 体内加注释；四份双机驱动之一 `crash-amplification-lib.sh` 加注释、加一个没人调的 shell 函数 | 判法函数内注释：用例键 `b536…`→`d856…`，对照不换；驱动加注释：4 条分片用例都不换；驱动加 shell 函数：查的两条分片用例换，不分片的 tree-split 不换（`p-judging`、`p-shard`）；harness 里加 fn 或只加注释，两条用例键都换（`p3-table`） | 要改：L17 引的 K7「驱动脚本改一句注释…三条用例的标记全作废」不对，改成「四份驱动里任一份有函数级改动（哪怕加一个没人调的函数）就作废全部 4 条分片用例；只改注释不作废」；分片用例是 4 条（new-pool-file-creation-stream、overwrite-fixed-script-stream、multi-record-publish-stream、unlink-after-the-multi-record-publish-stream） |
| L18 | 核过（grep） | `grep -c stage-marker` 三道阶段脚本 | 三份都是 0（`grep-anchors`） | 对 |
| L19 | 核过（量的） | `research/results/` 下新建一份文件；PATH 前插一个 `--version` 打 `99.0.0` 的假 `qemu-system-x86_64`；登记表加注释 | 新文件：QEMU、研究复跑整道、`mutation-tables-compile` 格换；假 QEMU：18 把一把不换；登记表注释：QEMU 换（`p3-table`）；`checker-tier-qemu-device-streams.sh:452-454` 点名几档就不写标记 | 对 |
| L20 | 核过（量的） | 把 `tools/memory-model` 拷进草稿 `ktree/`，`SINGLEFS_KERNEL_TREE` 指过去，再往 `linux-kernel.cat` 加一行注释；harness 加 fn | 默认树、拷贝树、改过 `.cat` 的拷贝树三次 LKMM 键同为 `3f76dfdbc95a6b9c`；harness 加 fn LKMM 键换（`p-lkmm`、`p3-table`） | 对 |
| L21 | 核过（量的） | `crates-mutation-rows.py judge-version` 在副本上，逐份加注释：本模块、`run-with-memory-cap.sh`、`mutation-prune.py`、`lib_verdict_library.py`、登记表；`row_key` 只改限时、只改上限、只改变异名 | 前四份各让判法版本换，登记表注释不换（它只取本道那几行）；行键三种改法都换（`p2-judge-version`、`p-keys`）；`BASE_DIRECTORIES_KEPT 3`；主工作区此刻的判法版本不在盘上三个目录里（`main-judge-version`） | 对。补：`mutation-prune.py` 注释也作废这里（广谱变异那边改切法连带） |
| L22 | 核过（量的） | B3：驱动 `run_items` 换替身量 `Pruner.carry` 调用数；B4：旧树里被变异函数是旧样、结论表有此刻指纹的「抓到」；B5：同文件末尾加一个没用的 `use`、文件挪目录；B7：定义宏的 `admission.rs` 里改一个无关函数，对照：不定义宏的 `unit.rs`；B8：测试 fn、测试调的辅助函数、测试路径上的 `publish_version`、被变异函数调到的 `index_node_header_bytes` 各改一处；B9：`singlefs-format/Cargo.toml` 加注释 | B3：不带参数 `carry_calls=0`、三条全选去现判，对照 `--milestone-full` `carry_calls=3`；B4：`carry_decision=True/carried changed_since=function-changed plan=judge:function-changed`；B5：`function-or-outside-changed`、键第一段换；B7：`macro-or-include-changed`，对照 `carried`；B8：测试 fn 改 `catching-test-changed`，其余三种都 `carried`；B9：全局键全文变，唯一变了的是 `file crates/singlefs-format/Cargo.toml <sha256>` 那一行（摘要换成 `310b1ff6…`）（`p-selection`、`p-prune`、`p4-carry`、`p-keys`） | 要补：重做方向「结论键 = 被变异的 fn 单元指纹 + …」漏了被变异函数调到的单元（量的：callee 改了照沿用）；写成「被变异 fn 单元与它调到的单元的指纹，或抓它的测试执行过的单元集合（已含）」二选一明说 |
| L23 | 核过（grep） | `scripts/mutation/mutate.sh` 文件头与 `grep -c sha256` | `:2` `# admission: always …`；`sha256` 3 处 | 对 |
| L24 | 核过（量的） | 照 `verus-proof-check.py:793`、`e165-pin-red-examples.py:1168` 的式子用 `lib_verdict_library.digest_file` 算两把判法版本（不 import 两份判法脚本）；逐份加注释：`verus-proof-check.py`、`lib_verdict_library.py`、`e165-pin-red-examples.py`、核 crate `lib.rs`；`lib.rs` 文件头插一行注释后 `verus-proof-check.py list-items`；e165 装置那一样照 `:1171-1187` 的式子在 core `unit.rs` 加注释前后各算一次 | 副本基线形式化判法版本 `3eecb6661fb0c839`，与主仓 `.git/singlefs-formal-verdicts/proof/keys/` 里 10-14 那条记录的 `judge_version` 前 16 位相同（换一条路的校验）；各份注释各换各的那几把（`lib_verdict_library.py` 一份注释换形式化、红例、crates 变异三把）；插一行注释：33 个空真项名 33 个全换（`vacuity:…lib.rs:1197` 起）；装置那一样 `f29981ca…`→`8b5c3412…`（419 份文件）；库里 vacuity 记录的 components 有「限时」一样（`p2-judge-version`、`verus-items`、`p-e165`、`stored-components`） | 对 |
| L25 | 核过（量的） | `admission.py replay-row-inputs <副本>`（247 行）：设 `ENABLE_GPU=1 CHECKER_TIER_FEATURES=gpu`；`replay.sh` 的 `replay_one` 里一行代码行尾加 `# probe inline comment`；`scripts/experiments/lib.rs` 加一个没人调的 fn | GPU 开关：247 行 0 行换；行尾注释：247 行全换；`lib.rs` 加 fn：247 行全换（`p-replay`）；`checker-tier-research-build-and-replay.sh:458` 库在 `${GATE_CROSS_RUN_TMPDIR:-${TMPDIR:-/tmp}}` | 对 |
| L26 | 核过（量的） | 副本 `.claude/kb/decisions/13-验证路线.md` 末尾加一行 HTML 注释；core `transaction.rs` 只加注释；全仓 grep `manifest` 子命令的调用 | kb 注释：E142 换、门禁 18 把不换；core 注释：E142、E142/p8g-crates-dump 两把都换（`p3-table`）；`manifest` 只有 `admission.py:4347`、`:4892` 两处自证在调，别处没有（`grep-anchors`） | 要补：实验指纹走 `input_manifest` 不带 `source_digests`（`admission.py:868-873`），是逐文件原文 sha256，只改注释也换，口径比门禁还宽一档 |
| L27 | 核过（ls / 读码） | `ls ~/.local/share/singlefs/reuse/`；现读 `reuse_library.py:88-89` | 只有 `e171-r1`；`:88` `DEFAULT_LIBRARY_ROOT = …/reuse/e171-r1`、`:89` `KEY_SCHEMA_LINE = "schema e171-r1-reuse-key 1"`。装置没跑 | 对 |
| L28 | 核过（抽查 5 处） | `awk 'NR==行号'` 现取 | `crash_amplification.rs:272`「路径不进块键（改名不使判定失效）」、`crash_judge_gpu.rs:21`「进判定表的键」、`crash_judge_tables.rs:11`「键带 JUDGE_TABLES_VERSION」、`verdict_store.rs:88`「门禁…给每条崩溃枚举用例算的输入指纹」、`crash_identity.rs:6-8`「复用键罩三样…判法代码」而 `:103-115` `reuse_key` 只拼写表与父输出——五处仍与代码不符 | 对。只抽查 5 处，其余没逐条核 |
| L29 | 核过（抽查计数） | `grep -c 'staged-green\|--changed-since\|整道\|全绿标记\|判法版本'` 五份文档 | `commit-stages.md` 6、`verification.md` 5、`implementation-workflow.md` 4、`staged-gate.md` 3、`main-agent.md` 1 | 对。只核了「旧口径还在」，没逐条核 docs 路 D1–D97 |

## 3 要改写的底稿行（照抄就能进派发提示）

- L14 现状的键一格补：「`.rs` 按整文件去注释词法；有算出来的 include（`include_str!(concat!…)` 这类）的 crate 整个取原文——此刻 `singlefs-harness` 273 份全部原文（`heavy_threads.rs:22`），`include_str!` 字面读到的 `.rs`（`crash_facts.rs`、`crash_amplification.rs`、`gpu_unit_checks.rs`、`model.rs`）取原文」；落点加 `scripts/gate/lib_source_digest.py:504-517`。
- L17 引 K7 那半句改：「四份双机驱动脚本按 shell 摘要进 4 条分片用例的键：只改注释不作废，任一份里有函数级改动（哪怕是没人调的函数）就作废全部 4 条」。
- L22 重做方向「结论键 = 被变异的 fn 单元指纹 + …」改成「被变异 fn 单元与它调到的单元各自的指纹 + …」，或明写「被变异 fn 的被调方由『抓它的测试执行过的单元集合』罩住」；阳性对照用这一轮造的那一格（`index_node_header_bytes` 改了，结论必须作废）。
- L26 现状的键一格补：「逐文件原文 sha256（`input_manifest` 不带 `source_digests`），只改注释也换」。
- L6 现状的键一格补：「给定状态输入串带 `seeds_and_scale=`，其中 `seed_count` 让同一批每张表的键都随种子个数变」。
- L12 盘上的数补：「`crash-amplification/runs/` 436 个目录 89G」。
- L7 落点行号：`crash_verify_gpu.rs:361` 改 `:364`。

## 4 原样输出

### p1-tsv-comment（副本 `stage-inputs.tsv` 末尾加一行注释前后，`tools/fp_all.sh`；左基线、右改后，前 16 位）

```
stage=checker-tier-crash-point-replay.sh 23d40aee1f654419 41c23db8fdb30323
stage=checker-tier-qemu-device-streams.sh f26628c26bc16862 f9eca7e493fec1f5
stage=checker-tier-lkmm.sh 3f76dfdbc95a6b9c 93b91748f29362d2
stage=checker-tier-formal-verification.sh:verus-toolchain-pinned 0b6f8d1f3b1acb8b 0b6f8d1f3b1acb8b
stage=checker-tier-formal-verification.sh:verus-proof 8eedfcb924f084d1 8eedfcb924f084d1
stage=checker-tier-formal-verification.sh:verus-vacuity 8df10175d563b498 8df10175d563b498
stage=checker-tier-formal-verification.sh:verus-must-red-worlds 2095c98355d59e0d 2095c98355d59e0d
stage=checker-tier-formal-verification.sh:pinned-red-examples-complete ce76c6a40178e19c ce76c6a40178e19c
stage=checker-tier-formal-verification.sh:pinned-red-examples-replay 4cade18cd786bcbb 4cade18cd786bcbb
stage=checker-tier-crates-mutation-replay.sh d9b2021f78649360 583c66d8bb01c4d4
stage=checker-tier-broad-mutation.sh fb4427061bbbcc61 b63380ef0a646ff8
stage=harness-model-differential-and-scenarios.sh 518aa24229e35066 8b19feb632ff0571
stage=checker-tier-research-build-and-replay.sh 16e769e2970086b2 1f17f5c5bee71199
stage=checker-tier-research-build-and-replay.sh:mutation-tables-compile 40bd924c95c47033 40bd924c95c47033
case=crash-case:tree-split-streams b536c28697db7d8a b536c28697db7d8a
case=crash-case:crash-injection-fast-tier 98c098743a3a4475 98c098743a3a4475
experiment=E142 6ef7410296c022e2 6ef7410296c022e2
experiment=E142/p8g-crates-dump fbeef81d3730b655 fbeef81d3730b655
```

### p3-table（`tools/cmp.py` 由 `out/p1-base.txt` 与 `out/p3a2.txt`、`out/p3b2.txt`、`out/p3c.txt`、`out/p1-tsv-comment.txt` 逐格比出来的；「换」= 那一把与基线不同）

| 造的改动 | crash-point-replay | qemu-device-streams | lkmm | fv:verus-toolchain-pinned | fv:verus-proof | fv:verus-vacuity | fv:verus-must-red-worlds | fv:pinned-red-examples-complete | fv:pinned-red-examples-replay | crates-mutation-replay | broad-mutation | harness-model-differential-and-scenarios | research-build-and-replay | research-build-and-replay:mutation-tables-compile | case:tree-split-streams | case:crash-injection-fast-tier | E142 | E142/p8g-crates-dump |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| comment_only_in_core_transaction_rs | 同 | 同 | 同 | 同 | 同 | 同 | 同 | 同 | 同 | 同 | 同 | 同 | 同 | 同 | 同 | 同 | 换 | 换 |
| unused_fn_appended_to_harness_model_rs | 换 | 换 | 换 | 同 | 同 | 同 | 同 | 换 | 换 | 换 | 换 | 换 | 换 | 同 | 换 | 换 | 换 | 换 |
| comment_in_crash_amplification_lib_sh | 同 | 同 | 同 | 同 | 同 | 同 | 同 | 同 | 同 | 同 | 同 | 同 | 同 | 同 | 同 | 同 | 同 | 同 |
| comment_in_cargo_config_toml | 换 | 换 | 换 | 换 | 换 | 换 | 换 | 换 | 换 | 换 | 换 | 换 | 换 | 换 | 换 | 换 | 换 | 换 |
| comment_in_kb_decision_13 | 同 | 同 | 同 | 同 | 同 | 同 | 同 | 同 | 同 | 同 | 同 | 同 | 同 | 同 | 同 | 同 | 换 | 同 |
| new_file_under_research_results | 同 | 换 | 同 | 同 | 同 | 同 | 同 | 同 | 同 | 同 | 同 | 同 | 换 | 换 | 同 | 同 | 同 | 同 |
| fake_qemu_version | 同 | 同 | 同 | 同 | 同 | 同 | 同 | 同 | 同 | 同 | 同 | 同 | 同 | 同 | 同 | 同 | 同 | 同 |
| stage_inputs_tsv_comment | 换 | 换 | 换 | 同 | 同 | 同 | 同 | 同 | 同 | 换 | 换 | 换 | 换 | 同 | 同 | 同 | 同 | 同 |
| comment_only_in_harness_memory_pool_rs | 换 | 换 | 换 | 同 | 同 | 同 | 同 | 换 | 换 | 换 | 换 | 换 | 换 | 同 | 换 | 换 | 换 | 换 |
| comment_only_in_checker_tier_crash_facts_rs | 换 | 换 | 换 | 同 | 同 | 同 | 同 | 换 | 换 | 换 | 换 | 换 | 换 | 同 | 换 | 换 | 换 | 换 |
| unused_shell_function_in_crash_amplification_lib_sh | 换 | 换 | 同 | 同 | 同 | 同 | 同 | 同 | 同 | 同 | 同 | 同 | 换 | 换 | 同 | 同 | 同 | 同 |

### p6-closure（`tools/closure_scenarios.sh`：移植闭包算法，GPU 判器与 tree_split 设点文件的 CPU、观察者三把，前 16 位）

```
scenario=baseline gpu=d17732ad426081b7 cpu_of_tree_split=fe35be42de6dccdd observer_of_tree_split=e72cadd60a623a12
scenario=cfg_test_unit_test_body_changed_in_crash_rs gpu=32a013f79b9bee5f cpu_of_tree_split=3baed90ed9492910 observer_of_tree_split=807739583ffdacb7
scenario=cfg_test_unit_test_body_changed_in_core_unit_rs gpu=38db630a22e2991e cpu_of_tree_split=31516bb7af2e03da observer_of_tree_split=07ad8dd42e19f4ea
scenario=comment_only_in_crash_rs_unit_test_control gpu=d17732ad426081b7 cpu_of_tree_split=fe35be42de6dccdd observer_of_tree_split=e72cadd60a623a12
scenario=comment_appended_to_crash_judge_recovery_wgsl gpu=f7e6698620da1193 cpu_of_tree_split=fe35be42de6dccdd observer_of_tree_split=e72cadd60a623a12
scenario=comment_appended_to_cargo_lock gpu=6b7258e79e6736d9 cpu_of_tree_split=052225929dec7e73 observer_of_tree_split=09e34a1b19e1e394
```

### p6b-closure（`tools/closure_scenarios2.sh`：同上，另数 137 行的表换了几行）

```
scenario=baseline gpu=d17732ad426081b7 cpu_of_tree_split=fe35be42de6dccdd observer_of_tree_split=e72cadd60a623a12 table_rows_changed=0
scenario=unused_fn_in_the_setting_file gpu=d17732ad426081b7 cpu_of_tree_split=ffe63b84da5cc5e3 observer_of_tree_split=2231d2874a4866d0 table_rows_changed=2
scenario=unused_fn_in_another_test_file gpu=d17732ad426081b7 cpu_of_tree_split=fe35be42de6dccdd observer_of_tree_split=e72cadd60a623a12 table_rows_changed=2
scenario=comment_in_core_cargo_toml gpu=a0f141cba2ce6599 cpu_of_tree_split=f948dae485f7d968 observer_of_tree_split=93ba629cf20182b0 table_rows_changed=134
scenario=crate_directory_singlefs_format_moved_same_content gpu=669fbdb7bd2813d5 cpu_of_tree_split=5fc57b6a027e8cc3 observer_of_tree_split=7913a3671337db29 table_rows_changed=134
scenario=unused_fn_in_crash_amplification_stop_file gpu=d17732ad426081b7 cpu_of_tree_split=fe35be42de6dccdd observer_of_tree_split=e72cadd60a623a12 table_rows_changed=0
scenario=unused_fn_in_crash_judge_dispatch_excluded_file gpu=d17732ad426081b7 cpu_of_tree_split=fe35be42de6dccdd observer_of_tree_split=e72cadd60a623a12 table_rows_changed=0
```

### p-reuse（`tools/reuse_probe.sh`：副本里同一棵树当 staged-green 与暂存树，真假 rustc 各问一次 gate-reuse；退 10 = 可跳过，退 0 = 要跑）

```
stage=harness-model-differential-and-scenarios.sh rustc=real gate_reuse_rc=10 fingerprint=518aa24229e35066 reason=与上次整轮全绿那棵树（96c5872c1e5c）在这几条路径上逐文件源码摘要相同（注释与空白不算）：crates/ Cargo.toml Cargo.lock scri
stage=harness-model-differential-and-scenarios.sh rustc=fake gate_reuse_rc=10 fingerprint=e1c59510e73214b0 reason=与上次整轮全绿那棵树（96c5872c1e5c）在这几条路径上逐文件源码摘要相同（注释与空白不算）：crates/ Cargo.toml Cargo.lock scri
stage=checker-tier-crash-point-replay.sh rustc=real gate_reuse_rc=10 fingerprint=23d40aee1f654419 reason=与上次整轮全绿那棵树（96c5872c1e5c）在这几条路径上逐文件源码摘要相同（注释与空白不算）：crates/ Cargo.toml Cargo.lock scri
stage=checker-tier-crash-point-replay.sh rustc=fake gate_reuse_rc=10 fingerprint=2a4908af989b7b18 reason=与上次整轮全绿那棵树（96c5872c1e5c）在这几条路径上逐文件源码摘要相同（注释与空白不算）：crates/ Cargo.toml Cargo.lock scri
stage=checker-tier-formal-verification.sh:pinned-red-examples-replay rustc=real gate_reuse_rc=10 fingerprint=4cade18cd786bcbb reason=与上次整轮全绿那棵树（96c5872c1e5c）在这几条路径上逐文件源码摘要相同（注释与空白不算）：Cargo.toml Cargo.lock crates/mutat
stage=checker-tier-formal-verification.sh:pinned-red-examples-replay rustc=fake gate_reuse_rc=10 fingerprint=476d7332837830ea reason=与上次整轮全绿那棵树（96c5872c1e5c）在这几条路径上逐文件源码摘要相同（注释与空白不算）：Cargo.toml Cargo.lock crates/mutat
stage=checker-tier-lkmm.sh rustc=real gate_reuse_rc=0 fingerprint=3f76dfdbc95a6b9c reason=输入与上次整轮全绿那棵树相同，但这一道上一次判绿时的环境没记下（/home/fy5090/.local/share/singlefs/work/survey-pruning-r2-verify
stage=checker-tier-lkmm.sh rustc=fake gate_reuse_rc=0 fingerprint=d7dd8bf737ad5c6c reason=输入与上次整轮全绿那棵树相同，但这一道上一次判绿时的环境没记下（/home/fy5090/.local/share/singlefs/work/survey-pruning-r2-verify
```

### p2-judge-version（`tools/judge_version_probe.sh`：crates 变异 `judge-version` 实跑；形式化、红例两把照 `:793`、`:1168` 的式子算；核 crate 是 `digest_tree` 整目录）

```
scenario=baseline crates_mutation=c6d72979bd38cc79 items=8 verus_judge_version=3eecb6661fb0c839 e165_judge_version=df506a683c7b50f5 verus_core_crate=b09cd400d32c3ce2
scenario=comment_in_crates_mutation_rows_py crates_mutation=29ef2906518ecb7b items=8 verus_judge_version=3eecb6661fb0c839 e165_judge_version=df506a683c7b50f5 verus_core_crate=b09cd400d32c3ce2
scenario=comment_in_run_with_memory_cap_sh crates_mutation=4af0ae0a92c475b3 items=8 verus_judge_version=3eecb6661fb0c839 e165_judge_version=df506a683c7b50f5 verus_core_crate=b09cd400d32c3ce2
scenario=comment_in_mutation_prune_py crates_mutation=e697ca24b2d8b5d1 items=8 verus_judge_version=3eecb6661fb0c839 e165_judge_version=df506a683c7b50f5 verus_core_crate=b09cd400d32c3ce2
scenario=comment_in_verus_proof_check_py crates_mutation=c6d72979bd38cc79 items=8 verus_judge_version=b0c1f786ac91cc83 e165_judge_version=df506a683c7b50f5 verus_core_crate=b09cd400d32c3ce2
scenario=comment_in_lib_verdict_library_py crates_mutation=b95fa5458d19e0fc items=8 verus_judge_version=e4c2f83ec59aaef5 e165_judge_version=8b7ddd82b36f7a43 verus_core_crate=b09cd400d32c3ce2
scenario=comment_in_e165_pin_red_examples_py crates_mutation=c6d72979bd38cc79 items=8 verus_judge_version=3eecb6661fb0c839 e165_judge_version=398d8d09b1e79062 verus_core_crate=b09cd400d32c3ce2
scenario=comment_in_replay_prefix_lib_rs crates_mutation=c6d72979bd38cc79 items=8 verus_judge_version=3eecb6661fb0c839 e165_judge_version=df506a683c7b50f5 verus_core_crate=8fa9215eefaa1ae0
scenario=comment_in_stage_inputs_tsv crates_mutation=c6d72979bd38cc79 items=8 verus_judge_version=3eecb6661fb0c839 e165_judge_version=df506a683c7b50f5 verus_core_crate=b09cd400d32c3ce2
```

### p-keys（`tools/keys_probe.py`：广谱变异 `global_key_lines`；crates 变异 `row_key`）

```
B9 global_key_same=False changed_lines=['file crates/singlefs-format/Cargo.toml 310b1ff66dd7f810cee472b87ff4799d96b59d2a1c144822d4731bb6777cbcbd']
C3 timeout_1800_vs_3600_same=False memory_8G_vs_16G_same=False renamed_same=False
BASE_DIRECTORIES_KEPT 3 JUDGE_RULES_VERSION prune-r3-e1-e6-fresh-build-mac
```

### p4-carry（`tools/carry_probe.py`：变异 `unit.rs:153` 的 `/`→`*`，造一条被 `seventy_nine_devices_fill_the_accounting_node_exactly_and_still_publish` 抓到的记录，逐格造改动看 `carry_decision`）

```
item ['crates/singlefs-core/src/unit.rs', 'index_node_entry_capacity', '45.0-45.1', '*'] function
test_fingerprint_is_function_level True
scenario=baseline_no_change carry=True reason=carried test_fp_same=True mutation_fp_same=True
scenario=test_function_body_changed carry=False reason=catching-test-changed test_fp_same=False mutation_fp_same=True
scenario=helper_called_by_the_test_changed carry=True reason=carried test_fp_same=True mutation_fp_same=True
scenario=callee_of_the_mutated_function_changed_same_file carry=True reason=carried test_fp_same=True mutation_fp_same=True
scenario=publish_version_on_the_test_path_changed carry=True reason=carried test_fp_same=True mutation_fp_same=True
scenario=mutated_function_itself_changed carry=False reason=function-or-outside-changed test_fp_same=True mutation_fp_same=False
```

### p-prune（`tools/prune_probe.py`：B4、B5、B7）

```
B4 carry_decision=True/carried changed_since=function-changed plan=judge:function-changed
B5_use_added carry=(False, 'function-or-outside-changed') key_same=True
B5_file_moved key_before=['crates/singlefs-core/src/unit.rs', 'index_node_entry_capacity'] key_after=['crates/singlefs-core/src/moved/unit.rs', 'index_node_entry_capacity'] same_key=False fingerprint_same=True
admission_item container=live_objects_of_the_inode_tree invoked_macro_dependency_nonempty=True
B7_unrelated_function_in_macro_defining_file carry=(False, 'macro-or-include-changed') fingerprint_same=True
B7_control_unrelated_function_in_file_without_macro carry=(True, 'carried')
```

### p-selection（`tools/selection_probe.py`：驱动 `run_items` 换替身；结论表替身对每条都交「可沿用」）

```
run=no_arguments pruner_built=1 carry_calls=0 selected_for_fresh_judging=[1, 2, 3]
  · 里程碑全量：开跑时会沿用的结论 3 条记进 milestone-carry.tsv，全部现判之后逐条比
run=milestone_full_control pruner_built=1 carry_calls=3 selected_for_fresh_judging=[1, 2, 3]
```

### p-shard（`tools/shard_cases.sh`：前两条登记了 shard=across-machines，tree-split 没登记）

```
baseline crash-case:new-pool-file-creation-stream 51dbadea3c48d7b0
baseline crash-case:multi-record-publish-stream a4b98875bc987c6b
baseline crash-case:tree-split-streams b536c28697db7d8a
comment_in_crash_amplification_lib_sh crash-case:new-pool-file-creation-stream 51dbadea3c48d7b0
comment_in_crash_amplification_lib_sh crash-case:multi-record-publish-stream a4b98875bc987c6b
comment_in_crash_amplification_lib_sh crash-case:tree-split-streams b536c28697db7d8a
unused_shell_function_in_crash_amplification_lib_sh crash-case:new-pool-file-creation-stream f5e7951d0986c97f
unused_shell_function_in_crash_amplification_lib_sh crash-case:multi-record-publish-stream 0e22bb8eda0e8b51
unused_shell_function_in_crash_amplification_lib_sh crash-case:tree-split-streams b536c28697db7d8a
```

### p-judging（`tools/judging_digest_cases.sh`：tree-split 用例键（前 16 位）与崩溃点重放整道键）

```
scenario=baseline b536c28697db7d8a stage=23d40aee1f654419
scenario=comment_inside_judging_function d8565b6e38a01c6d stage=23d40aee1f654419
scenario=comment_inside_experiment_fingerprint_control b536c28697db7d8a stage=23d40aee1f654419
scenario=unused_function_appended_to_admission_py b536c28697db7d8a stage=ae156086a7ce7319
```

### p-replay（`tools/replay_scenarios.sh`：`admission.py replay-row-inputs` 247 行，按（行，键）与基线比）

```
scenario=gpu_switch_on rows=247 changed_rows=0
scenario=inline_comment_in_replay_one rows=247 changed_rows=247
scenario=unused_fn_in_research_lib_rs rows=247 changed_rows=247
```

### p-lkmm（副本上 `admission.py stage-fingerprint <副本> checker-tier-lkmm.sh`，`SINGLEFS_KERNEL_TREE` 指向草稿里拷的 `tools/memory-model`）

```
default_tree=3f76dfdbc95a6b9c7bbdfe0f4fd5cb1d67a2b10166227ddebc0263c0912ce03f 431
copied_tree=3f76dfdbc95a6b9c7bbdfe0f4fd5cb1d67a2b10166227ddebc0263c0912ce03f 431
copied_tree_with_linux_kernel_cat_edited=3f76dfdbc95a6b9c7bbdfe0f4fd5cb1d67a2b10166227ddebc0263c0912ce03f 431
```

### verus-items（`verus-proof-check.py list-items <副本>`：核 crate `lib.rs` 文件头插一行注释前后的项名）

```
base_rc=0 items=188 vacuity=33
comment_rc=0 items=188
item_names_changed=33
3,35c3,35
< vacuity:crates/singlefs-replay-prefix/src/lib.rs:1197
< vacuity:crates/singlefs-replay-prefix/src/lib.rs:1230
---
> vacuity:crates/singlefs-replay-prefix/src/lib.rs:1198
> vacuity:crates/singlefs-replay-prefix/src/lib.rs:1231
```

### p-e165（`tools/e165_device_probe.py`）

```
E2 device_files=419 before=f29981ca11489598 after_comment_in_core_unit_rs=8b5c34128c9d94a0 same=False
```

### digest-kinds（`tools/digest_kinds.py`：副本 `crates/`、`Cargo.toml`、`Cargo.lock` 下 git 列得出的文件交 `lib_source_digest.digests_of`，数 `.rs` 各 crate 的写法；再往 `memory_pool.rs` 末尾加一行注释看哪几份摘要变；第二段是把 `heavy_threads.rs:22` 换成字面量路径之后）

```
rs_files crate=singlefs-checker kind=rust count=4
rs_files crate=singlefs-checker-tier kind=raw count=3
rs_files crate=singlefs-checker-tier kind=rust count=88
rs_files crate=singlefs-core kind=rust count=34
rs_files crate=singlefs-format kind=rust count=1
rs_files crate=singlefs-harness kind=raw count=273
rs_files crate=singlefs-reference-filesystem kind=rust count=1
rs_files crate=singlefs-replay-prefix kind=rust count=1
comment_appended_to=crates/singlefs-harness/src/memory_pool.rs kind=raw changed_entries=['crates/singlefs-harness/src/memory_pool.rs']
--- heavy_threads.rs:22 换成字面量路径之后
rs_files crate=singlefs-checker kind=rust count=4
rs_files crate=singlefs-checker-tier kind=raw count=3
rs_files crate=singlefs-checker-tier kind=rust count=88
rs_files crate=singlefs-core kind=rust count=34
rs_files crate=singlefs-format kind=rust count=1
rs_files crate=singlefs-harness kind=raw count=1
rs_files crate=singlefs-harness kind=rust count=272
rs_files crate=singlefs-reference-filesystem kind=rust count=1
rs_files crate=singlefs-replay-prefix kind=rust count=1
comment_appended_to=crates/singlefs-harness/src/memory_pool.rs kind=rust changed_entries=[]
```

### stored-components（只读主仓 `.git/singlefs-formal-verdicts/*/keys/*.json`：各条记录 `judge_version` 前 16 位计数与 components 名字并集）

```
proof judge_version_prefixes {'3eecb6661fb0c839': 1, '': 1, '6dacce535664bf93': 2, '54114142e1b199d3': 1} component_names ['verus-pins.tsv', '判法', '工具链', '核 crate', '限时']
vacuity judge_version_prefixes {'': 33, '6dacce535664bf93': 33} component_names ['verus-pins.tsv', '判法', '工具链', '核 crate', '这一处', '限时']
pinned-red-examples-replay judge_version_prefixes {'': 753, '4c3137d190b03f01': 1080} component_names ['判法', '引的变异行', '档', '装置', '这一行']
```

### main-judge-version（主工作区只读：`python3 -I scripts/mutation/crates-mutation-rows.py judge-version <主工作区>` 与 `ls .git/singlefs-mutation-rows/`）

```
== main-judge-version
c6d72979bd38cc79bcc1118bc0e7f5c438525504d19235c974696fd68b36aba7 8
37ab6ce31a0dbf86b9cbc5e41eeb75424a14223e3a733d3865ff4313eb395730
3a63d8405e4ce1fbf82bb0043274a37c2481324c983d921997d5e852ad5828c8
d0ef8d8c93edbb15ba55d9c7d9c8faff4b374bf3118c550c11a1a5a98fee87d6
```

### grep-anchors（现查的几处 grep）

```
$ grep -c stage-marker （三道阶段脚本）
.claude/gate.d/checker-tier-research-build-and-replay.sh:0
.claude/gate.d/checker-tier-crash-point-replay.sh:0
.claude/gate.d/harness-model-differential-and-scenarios.sh:0
$ grep -rn admission.py 的 manifest 子命令调用（排除 research/results 与 crash-case-manifest）
records/2026-09-16-subagent拆分提案.md:950:| 39 | 门禁 54、55、57、59 号接准入模块（`scripts/gate/admiss
scripts/gate/admission.py:2996:        say("  ✗ 用法：admission.py manifest <项目根> <键> <清单文件> [--ext
scripts/gate/admission.py:4347:        exit_code, output, messages = run_quietly([sys.executable, module, "manifest", wo
scripts/gate/admission.py:4892:                [sys.executable, module, "manifest", work, "59-demo.sh", manifest_file, "
$ grep include_str!(concat!
crates/singlefs-checker-tier/src/crash_identity.rs:661:    include_str!(concat!(env!("OUT_DIR"), "/conclusion_
crates/singlefs-harness/src/heavy_threads.rs:22:pub const REGISTRATION_FILE_TEXT: &str = include_str!(concat!(
$ mutate.sh 第 2 行与 sha256 计数
# admission: always 每一次调都判此刻的仓与这一…
3
```

### ls-libraries（`du -sh ~/.local/share/singlefs/crash-amplification/*/`，只读）

```
== ls-libraries
12K	/home/fy5090/.local/share/singlefs/crash-amplification/crash-point-replay-fast-tier/
564K	/home/fy5090/.local/share/singlefs/crash-amplification/crash-point-replay-quick/
2.8M	/home/fy5090/.local/share/singlefs/crash-amplification/facts-pairing-library/
104M	/home/fy5090/.local/share/singlefs/crash-amplification/given-states-library.polluted-by-gpu-fault-tests-2026-10-11/
159M	/home/fy5090/.local/share/singlefs/crash-amplification/judge-tables-pairing-library/
251M	/home/fy5090/.local/share/singlefs/crash-amplification/crash-state-enumeration-library/
272M	/home/fy5090/.local/share/singlefs/crash-amplification/library/
624M	/home/fy5090/.local/share/singlefs/crash-amplification/layer0-enumeration-library/
918M	/home/fy5090/.local/share/singlefs/crash-amplification/amplification-unit-tests-library/
981M	/home/fy5090/.local/share/singlefs/crash-amplification/mutation-replay/
1.3G	/home/fy5090/.local/share/singlefs/crash-amplification/library-by-conclusion-kind/
1.8G	/home/fy5090/.local/share/singlefs/crash-amplification/given-states-library-of-harness-scenario-images-judged-afresh/
2.2G	/home/fy5090/.local/share/singlefs/crash-amplification/given-states-library/
89G	/home/fy5090/.local/share/singlefs/crash-amplification/runs/
runs_directories=436
```

## 5 推翻条件，与这一轮造过的那一次

| 结论 | 什么现象会推翻 | 这一轮造的对照（证明探针分得出两边） |
|---|---|---|
| harness 整 crate 按原文进门禁键（L14 补） | 只往 harness 某份没被 `include_str!` 字面读的 `.rs` 加注释，`stage-fingerprint` 不变 | 把 `heavy_threads.rs:22` 换成字面量路径后，同一改动 `changed_entries=[]`（`digest-kinds` 第二段）；core `transaction.rs` 只加注释 18 把 0 把换（`p3-table`） |
| 被变异函数调到的函数改了，「抓到」照沿用（L22 补） | 只改 `index_node_header_bytes`，`carry_decision` 不交 `carried` | 改被变异函数自己 → `function-or-outside-changed`；改测试 fn → `catching-test-changed`（`p4-carry`） |
| 驱动脚本只改注释不作废分片用例（L17 改） | 只给 `crash-amplification-lib.sh` 加注释，分片用例键变 | 同一份加一个没人调的 shell 函数，两条分片用例键都变、不分片的不变（`p-shard`） |
| 实验指纹按原文（L26 补） | core 只加注释，E142 键不变 | 同一改动门禁 18 把 0 把换（`p3-table` 第一行），即这条路分得出「去注释」与「原文」 |
| 树对比不比工具链（L15、L16） | 换了 rustc，没登记 `environment=` 的道 `gate-reuse` 退 0 | 登记了 `environment=` 的 LKMM 退 0；三道的 `stage-fingerprint` 随假 rustc 换（`p-reuse`） |
| 广谱变异不带参数不读结论表（L22 B3） | 不带参数时 `Pruner.carry` 被调 | `--milestone-full` 那一支 `carry_calls=3`（`p-selection`） |
| QEMU 版本、`tools/memory-model` 不进键（L19、L20） | 换版本串 / 改 `.cat`，键变 | `research/results/` 加文件 QEMU 键变；harness 加 fn LKMM 键变（`p3-table`） |
| 复跑行键不含 GPU 开关（L25） | 设 `ENABLE_GPU=1` 有行变 | `replay_one` 行尾注释 247 行全变（`p-replay`） |
| 移植闭包算法与 Rust 同算法（L1–L5、L7、L9 的前提） | 任一行与编译表不等 | 137/137 相等，`diff_rc=0`（第 1 节） |
| 形式化判法版本的复算式子（L24） | 复算值与库里记录不等 | 复算 `3eecb6661fb0c839` 与 `proof/keys` 里一条记录相同（`stored-components`） |

## 6 排除掉的解释与凭的观测

- 「只改 `.rs` 注释，门禁键整片都换」：第一次跑的 `out/p3a.txt`、`out/p3b.txt` 是这样，排除。原因是改动工具把备份 `*.probe-orig` 放在副本里，`git ls-files --others` 把它当新文件列进清单；凭 `digest_kinds.py` 量到 `transaction.rs` 加注释 `changed_entries=[]` 认出来，改用备份放副本外的 `edit2.py` 重跑成 `p3a2`、`p3b2`。两份作废的原样留在 `out/`。同期用旧工具的几格不受影响：移植闭包只顺 `use` 收 `.rs`（同批的注释对照三把不换），`mutation-prune.py` 的 `rust_files` 只列 `.rs`，空真项名只看 `lib.rs` 内容（之后用 `edit2.py` 重跑仍是 33 个全换）。
- 「副本与主工作区不同，量的是副本的特殊情形」：收尾 `rsync -an --checksum` 只有时间戳行与别的会话改的 `checks-owed.md`，排除。
- 「副本里 `git add` 进了索引，键因此变」：`base_same_after_index`，排除。
- 「移植算法已与编译表脱节」：主工作区重核 137/137，排除。
- 「`crates-mutation-rows.py` 判法版本里有整张登记表」：登记表加注释判法版本不换（`p2-judge-version` 最后一行），排除；它只取本道那几行。
- 「K7 的『改注释就作废』在别的分片用例上成立」：两条分片用例都不换，排除到「驱动脚本有函数级改动」。

## 7 没做什么

- 没跑 Rust（不编、不跑 checker 档测试）：L3 块键那一半、L6 整条、L8 核对复用的实例、L9 引擎退 78 的那一下、L10、L11 的运行期、scripts 路 K1、crates 路 C26 都只读码。要坐实只能跑 checker 档测试或引擎，是重型，交主 agent 去问用户；这一轮不写命令。
- scripts 路推翻条件里没造的：B6（切不开退整文件）、C4–C8（crates 变异按覆盖复用那几条要覆盖记录与快照）、E4、X1、X5（要起装置）、K1（要跑放量）。
- 盘上的数（disk 路）没重量；底稿第三节 8 问不归这一路，只顺手留了两样线索：门禁源码摘要对算出来的 include 是整 crate 退原文（第三节第 1 问）；`admission.py manifest` 只有自己的两格自证在调（`admission.py:4347`、`:4892`，第三节第 6 问）。L30、L31 不在这一轮的 29 条里，没核。
- 没经内存包装起的几条（都是前台几秒到几十秒的只读 python）：最早两次 `verus-proof-check.py list-items`（之后经包装重跑，`verus-items`）、`replay-row-inputs` 基线那一次（46 秒）、`shard_probe.py` 两次排错、LKMM 第一次（之后经包装重跑，`p-lkmm`）、主工作区 `crates-mutation-rows.py judge-version` 两次。
- 在副本自己的 `.git` 里做过 `git init`、`git add`、`write-tree`、`commit-tree`、`update-ref refs/sop/staged-green`（造树对比那一格用）；主仓的 git 目录一样没动。
- 没修、没判该怎么改；副本里的改动全部还原、不回主工作区。

## 要主 agent 跑的命令

没有。

## 8 副本与草稿的去留

- 交回前删掉：仓副本 `copy/`（删前 `du -sh` 603M，含它自己的 `.git` 与源码摘要缓存）、拷的内核内存模型 `ktree/`（580K）、假 rustc `fakebin/`（8.0K）、假 QEMU `fakeqemu/`（8.0K）、选条探针的假库 `fake-library/`（12K）、空的 `backups/`（4.0K）。
- 留下：`tools/`（全部探针脚本）、`out/`（全部原样输出，含第 6 节说的两份作废输出）、这份报告。删完整个草稿目录 792K。
- 第 4 节 `grep-anchors` 块里 `mutate.sh` 那一行被 `cut -c1-60` 从一个汉字中间截断，报告里那个残字节换成了「…」；`out/grep-anchors.txt` 是原样。
