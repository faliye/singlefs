# 回扫员 judge-S38：知识腐烂治理第二轮逐行判（批 6，机械探针）

日期：2026-10-17。分到的组：`awk -F'\t' '$1=="S38"{print $2}' segments2.tsv`，183 个组（P10 路径探针 14 行、P30 代码名探针 169 行）。没建副本、没起编译。草稿：`/home/fy5090/.local/share/singlefs/work/rot-2026-10-17-S38/`（`gen.py` 是按我逐类判完之后批量写表的脚本）。

做法：每个 P30 名字先在 `crates scripts .claude/gate.d .claude/hooks .claude/scripts litmus Cargo.toml` 里 `git grep -F` 与 `grep -rF` 各核一遍（148 个不同名字，除 `checker/src/walk.rs`、`scripts/gate.sh` 这类路径子串外全部零命中），再另搜 `research/`（命中的多在 `research/prompts` 冻结证据与 `research/results` 产物里）。大类：别家实现的名字（内核、btrfs、bcachefs、ZFS、XFS、UBI 等）判不相干；引产物字段、当时测试名、当时现查结果的判事件句不改；把已删或改名的名字当现有代码写的判要改，共 8 行（P30002、P30014、P30015、P30020、P30021、P30022、P30025、P30031）。

注意：工作区里有别的会话暂存的大批 `crates/` 改动（`git status` 里 `crates/singlefs-checker-tier/src/crash_identity.rs` 等为 `M `）。P30020、P30021、P30022 的「要改」依据是暂存之后的工作区；这几处在 HEAD 里名字还在（`git grep -F RANDOM_HISTORY_THROUGH_LIBRARY_MODULES_ALLOWED HEAD -- crates` 命中 `crash_identity.rs:309`），那批暂存提交之前改这几句要先问那个会话。

## 逐行判定
| 组 | 载体 | 判定 | 改后的句子或理由 |
|---|---|---|---|
| P10001 | .claude/kb/checks-owed.md:22 | 事件句不改 | 原话「EAD` 比的 `scripts/gate/change-touches-crates.sh` 两条路已删。*」说的是这两条路「已删」这件事本身，不把路径当现役文件引；`ls scripts/gate/stage-must-run.sh scripts/gate/change-touches-crates.sh` 两个都不存在，与句子一致。 |
| P10002 | .claude/kb/checks-owed.md:22 | 事件句不改 | 原话「上的；按树比的 `scripts/gate/stage-must-run.sh` 与按 `HEA」说的是这两条路「已删」这件事本身，不把路径当现役文件引；`ls scripts/gate/stage-must-run.sh scripts/gate/change-touches-crates.sh` 两个都不存在，与句子一致。 |
| P10006 | .claude/kb/checks-owed.md:200 | 事件句不改 | 原话「c io_min .claude/kb/invariants.md` = 0 与 `」是 2026-09-10 那一轮反推腿当时现查的 grep，句内已写明「当时的单文件清单，现已拆到 `crates/singlefs-checker/invariants/`」，是搬迁说明与事件，不是把旧路径当现役引。 |
| P10007 | .claude/kb/checks-owed.md:200 | 事件句不改 | 原话「c io_min .claude/kb/invariants.md` = 0 与 `」是 2026-09-10 那一轮反推腿当时现查的 grep，句内已写明「当时的单文件清单，现已拆到 `crates/singlefs-checker/invariants/`」，是搬迁说明与事件，不是把旧路径当现役引。 |
| P10015 | .claude/kb/checks-owed.md:645 | 不相干 | 原话「不说  上游 `scripts/gate.sh` 的 `--st」：「上游 `scripts/gate.sh`」指上游 SOP 自己仓里的路径（本仓拷贝在 `.claude/singlefs-ai-sop/scripts/gate.sh`，`ls` 存在），不是本仓的路径；该行是给上游的欠账，不受 `.claude/scripts/gate.sh` 这个本仓薄包装影响。 |
| P10016 | .claude/kb/checks-owed.md:645 | 不相干 | 原话「不说  上游 `scripts/gate.sh` 的 `--st」：「上游 `scripts/gate.sh`」指上游 SOP 自己仓里的路径（本仓拷贝在 `.claude/singlefs-ai-sop/scripts/gate.sh`，`ls` 存在），不是本仓的路径；该行是给上游的欠账，不受 `.claude/scripts/gate.sh` 这个本仓薄包装影响。 |
| P10017 | .claude/kb/checks-owed.md:652 | 不相干 | 原话「.0.52 的 `scripts/shell-lint.sh` 修了两处假红（」：「上游 SOP 0.0.52 的 `scripts/shell-lint.sh`」指上游 SOP 自己仓里的路径（本仓拷贝在 `.claude/singlefs-ai-sop/scripts/shell-lint.sh`，`ls` 存在），不是本仓路径。 |
| P10034 | .claude/kb/experiments/155-每次持久化的写量三种fsync形态与反事实上界.md:16 | 事件句不改 | 原话「olume.rs mutations/e155_fsync_write_volume.tsv` 报「基线：全绿」是那一轮跑变异表时敲的命令（「报基线：全绿 → 全部红」），说的是当时的跑法；同一行已用现路径 `scripts/mutation/tables/e155_*.tsv`（`ls scripts/mutation/tables` 两份都在）。 |
| P10038 | .claude/kb/experiments/155-每次持久化的写量三种fsync形态与反事实上界.md:89 | 事件句不改 | 原话「scade.rs mutations/e155_third_run_release_cascade.tsv` 报「基线：全绿」是那一轮跑变异表时敲的命令（「报基线：全绿 → 全部红」），说的是当时的跑法；同一行已用现路径 `scripts/mutation/tables/e155_*.tsv`（`ls scripts/mutation/tables` 两份都在）。 |
| P10040 | .claude/kb/experiments/158-择根与修复四岔路.md:1027 | 事件句不改 | 原话「ery.rs`/`checker/walk.rs`。逐一现查本登记」说的是那一轮开工重核时另一条并行线正在改哪几个文件，是事件；`checker/walk.rs` 与同句 `mount.rs`、`recovery.rs` 一样是不带目录的简写，仓里从没有过这个路径。 |
| P10070 | .claude/kb/milestone/02-second-txn.md:391 | 不相干 | 原话「.rs` 三处、`checker/image.rs` 的区域数下标，」是 `crates/singlefs-checker/src/image.rs` 的简写（同句 `bytes.rs`、`allocator.rs` 也不带目录），`git log --all --name-only -- "*checker/image.rs"` 零行，仓里从没有过这个路径，不是搬迁后的旧路径。 |
| P10071 | .claude/kb/milestone/02-second-txn.md:391 | 不相干 | 原话「自己的错误成员，`checker/src/walk.rs` 的中央映射那一」是 `crates/singlefs-checker/src/walk.rs` 的后缀简写（`git grep -h -o -E ".(0, 12)checker/src/walk\.rs" -- crates scripts .claude` 的 589 处里 587 处是这个全路径里的子串，其余两处就是这一行这类简写），文件在原处，没搬过。 |
| P10072 | .claude/kb/milestone/02-second-txn.md:391 | 不相干 | 原话「自己的错误成员，`checker/src/walk.rs` 的中央映射那一」是 `crates/singlefs-checker/src/walk.rs` 的后缀简写（`git grep -h -o -E ".(0, 12)checker/src/walk\.rs" -- crates scripts .claude` 的 589 处里 587 处是这个全路径里的子串，其余两处就是这一行这类简写），文件在原处，没搬过。 |
| P10074 | .claude/kb/milestone/03-third-txn.md:202 | 不相干 | 原话「查员从产物量的，`m3-batch60/exponential-stage/report.md`）  导入随库」是草稿目录 `~/.local/share/singlefs/work/` 下的相对路径（`records/2026-10-01-删除与并发事务.md` 1527、1529 行写全了），不是仓内路径，也不会进仓。 |
| P30001 | .claude/agent-common.md:47 | 不相干 | 原话「layer0`、`stream_0`、`first_」是禁用写法的反例（「不写 `layer0`、`stream_0`、`first_stream`」），本来就不该出现在代码里。 |
| P30002 | .claude/agent-common.md:62 | 要改 | 原话「自己的线程变量（`KS_THREADS` 这类）同样设成」已过时，改后：实验装置自己的线程变量（`E168_THREADS` 这类）同样设成 N。——`KS_THREADS` 在 `crates/`、`scripts/` 零命中（只剩 research/prompts 冻结证据里有），现役同类变量是 `crates/singlefs-checker-tier/src/bin/e168_delete_ring_window_borrow.rs` 里读的 `E168_THREADS`（`git grep -c E168_THREADS -- crates` 5 处）。 |
| P30003 | .claude/kb/checks-owed.md:25 | 不相干 | 原话「SUPP` 的 `REQ_HARDBARRIER` 在 2.6.3」里的名字是Linux 块层请求标志，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30004 | .claude/kb/checks-owed.md:25 | 不相干 | 原话「块层自己合成的 `REQ_PREFLUSH`/`REQ_FU」里的名字是Linux 块层请求标志，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30005 | .claude/kb/checks-owed.md:25 | 不相干 | 原话「eads` / `drop_writes` / `erro」里的名字是Linux dm-flakey 的选项，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30006 | .claude/kb/checks-owed.md:25 | 不相干 | 原话「它的全部能力是 `error_reads` / `drop」里的名字是Linux dm-flakey 的选项，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30007 | .claude/kb/checks-owed.md:25 | 不相干 | 原话「ites` / `error_writes` / `corr」里的名字是Linux dm-flakey 的选项，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30008 | .claude/kb/checks-owed.md:26 | 不相干 | 原话「x 6.17）：`zone_sectors` 挂在 `dis」里的名字是Linux 块层 queue limits 字段，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30009 | .claude/kb/checks-owed.md:73 | 不相干 | 原话「buffer`、`WriteBuffer` 三个词在 `g」说这三个词在 `crates/*/src` 里零命中，现查 `git grep -i -E "write.buffer|write_buffer|WriteBuffer" -- "crates/*/src"` 仍零行，句子现在仍是真话。 |
| P30010 | .claude/kb/checks-owed.md:173 | 不相干 | 原话「  **一次 `ZoneReset` 把已提交的一整」里的名字是zoned 设备的区重置命令（E125 产物里的名字），说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30011 | .claude/kb/checks-owed.md:210 | 不相干 | 原话「 源码对应的是 `ds_num_children`（`dsl_de」里的名字是ZFS 源码字段，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30012 | .claude/kb/checks-owed.md:211 | 不相干 | 原话「，就把出生晚于 `dd_origin_txg` 的块追加进 l」里的名字是ZFS 源码字段，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30013 | .claude/kb/checks-owed.md:254 | 事件句不改 | 原话「.txt` 的 `i99_legal_red`：perunit」引的是当时那份产物 `output.txt` 里的字段读数（perunit 232、copy 360、clone 39256），是对那次产物的引用，不是在说仓里现有代码。 |
| P30014 | .claude/kb/checks-owed.md:314 | 要改 | 原话「的链接，而它的 `mask_code()` 会把反引号」已过时，改后：共享门禁「链接指向」阶段（上游 `link-targets.py`）只判 `[文字](路径)` 形态的链接，而它的 `mask_code_spans()` 会把反引号片段整段涂空（`.claude/singlefs-ai-sop/scripts/link-targets.py:42` 现名；`git log -S` 在 `.claude/singlefs-ai-sop` 与 `scripts` 里找不到 `mask_code(` 这个名字，行为没变，只是名字写错或上游改过）。 |
| P30015 | .claude/kb/checks-owed.md:321 | 要改 | 原话「s.rs` 的 `worker_threads_by_default()` 返回 `m」已过时，改后：而 `crates/singlefs-harness/tests/random_histories.rs` 的 `worker_threads_from_environment()` 现在读 `SINGLEFS_RANDOM_HISTORY_THREADS`，没设取 `SINGLEFS_THREAD_CAP`，再没设取登记的每台默认重活线程数（不取本机核数，也不再夹到 16；该文件 1984–2012 行）——`worker_threads_by_default()` 已不存在，「返回 `min(available_parallelism(), 16)`」不再是真话，C457 的超订阅前提要按现在的取法重核。 |
| P30016 | .claude/kb/checks-owed.md:330 | 事件句不改 | 原话「 那条路径上把 `corrupted_check_holds` 填成「G27 」描述 stage2 产物里字段名填反的 bug，同一行后文写明主 agent 当天已修（新产物 `stage3.out`，`stage2.out` 原样留着当证据），说的是那一次发生的事；`git grep -i corrupted_check -- crates scripts` 零行与「已改」一致。 |
| P30017 | .claude/kb/checks-owed.md:330 | 事件句不改 | 原话「名填成它的反面（`corrupted_check_is_red`）。产物 `e1」描述 stage2 产物里字段名填反的 bug，同一行后文写明主 agent 当天已修（新产物 `stage3.out`，`stage2.out` 原样留着当证据），说的是那一次发生的事；`git grep -i corrupted_check -- crates scripts` 零行与「已改」一致。 |
| P30018 | .claude/kb/checks-owed.md:364 | 事件句不改 | 原话「 个判定全择中（`before_row_root` 248、`be」是「批 17 立账时量到」的产物字段读数（248、216），说的是那一次量到的数，不是现有代码。 |
| P30019 | .claude/kb/checks-owed.md:364 | 事件句不改 | 原话「ot` 248、`before_warm_up_covers` 216），其中」是「批 17 立账时量到」的产物字段读数（248、216），说的是那一次量到的数，不是现有代码。 |
| P30020 | .claude/kb/checks-owed.md:426 | 要改 | 原话「`m5_`（例 `lexical_digest_matches_the_e164_anchors`）。测试文件头里」已过时，改后：另 7 条带实验号或 `m1_`、`m3_`、`m5_`（例 `eight_byte_flip_probes_recover_as_the_e142_product_records`）——例子换成现存的 `crates/singlefs-harness/tests/recovery_reads_the_created_file_after_reopen.rs:132`；`lexical_digest_matches_the_e164_anchors` 随 `crash_identity.rs` 的暂存重写已不在工作区（HEAD 的 `crash_identity.rs:802` 还有），「7 条」「21 条」「3231 条」要在提交后重数。 |
| P30021 | .claude/kb/checks-owed.md:446 | 要改 | 原话「用例的白名单表（`RANDOM_HISTORY_THROUGH_LIBRARY_MODULES_ALLOWED`）逐个引入者判谁」已过时，改后：对象已不在（工作区）：`crates/singlefs-checker-tier/src/crash_identity.rs` 的白名单表 `RANDOM_HISTORY_THROUGH_LIBRARY_MODULES_ALLOWED` 与判引入者的 `random_history_scaffolding_introducers_judged` 在暂存的重写里已删（HEAD 的 `crash_identity.rs:309`、`:1775` 还有），别名绕过这一条随之没有对象；提交后销账或改写成新机制的同类问题。 |
| P30022 | .claude/kb/checks-owed.md:446 | 要改 | 原话「，没接到真正的 `random_history_scaffolding_introducers_judged` 上（那要编 c」已过时，改后：对象已不在（工作区）：`crates/singlefs-checker-tier/src/crash_identity.rs` 的白名单表与判引入者的 `random_history_scaffolding_introducers_judged` 在暂存的重写里已删（HEAD 的 `crash_identity.rs:1775` 还有），「攻方没接到真正的 `random_history_scaffolding_introducers_judged` 上」这一句随之没有对象；提交后销账或改写。 |
| P30023 | .claude/kb/checks-owed.md:502 | 不相干 | 原话「TEPS=1`，`OPUS_R3_ARM` 取空与 `re」是攻方装置变量，装置在 `research/prompts/m3-batch10-code-r3-opus-model/rerun.sh`（`git grep -l -F OPUS_R3_ARM -- research/prompts/m3-batch10-code-r3-opus-model` 命中 main.rs、mount-arms.patch、rerun.sh），不在 `crates/` 是本来如此。 |
| P30024 | .claude/kb/checks-owed.md:560 | 不相干 | 原话「g.rs` 的 `txg_is_inside_the_one_revolution_window` 已由 W4 删」说的就是函数已删、零命中：`git grep -l -F txg_is_inside_the_one_revolution_window -- . ":!research/prompts"` 只剩 checks-owed.md 与一份 records，代码里零行，句子仍是真话。 |
| P30025 | .claude/kb/checks-owed.md:649 | 要改 | 原话「353 行那个 `must_hold` 列表的第一项就」已过时，改后：由 `crates/mutations.tsv` 里「步 2 变异：已分配行不随分配更新（I-3.1 要红；树分裂换锚点）」那一行承担（要红的是 I-3.1（已分配统计对得上）；点名的 `cold_start_reads_the_second_content_and_the_pool_checker_stays_green` 现在是 checker 档同名用例，在 `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs:1310`，不再有 `must_hold` 列表，改法以变异表该行第三、四列为准），由门禁 checker-tier-crates-mutation-replay 复跑——立这一条的那一轮没跑这道门禁。 |
| P30026 | .claude/kb/checks-owed.md:710 | 不相干 | 原话「5），树里没有 `statement_line` 这个函数了（`」说树里没有这个函数；现查 `git grep -n -F statement_line -- scripts .claude` 只命中 `.claude/kb/checks-owed.md:710`，句子仍是真话。 |
| P30027 | .claude/kb/checks-owed.md:825 | 事件句不改 | 原话「；已还的那一半（`ACRO_DUP`）随脚本一起退役」说检测器 2026-10-11 从仓里删掉、已还的一半随脚本退役，是退役这件事本身，零命中与之一致。 |
| P30028 | .claude/kb/decisions/02-RAID条带策略.md:41 | 不相干 | 原话「btrfs 把 `num_stripes` / `type」里的名字是btrfs 的 chunk 字段，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30029 | .claude/kb/decisions/02-RAID条带策略.md:41 | 不相干 | 原话「type` / `stripe_len` 挂在 chun」里的名字是btrfs 的 chunk 字段，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30030 | .claude/kb/decisions/02-RAID条带策略.md:276 | 不相干 | 原话「re.c` 的 `nvme_update_disk_info` 里算 `phy」里的名字是Linux NVMe 驱动函数，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30031 | .claude/kb/decisions/02-RAID条带策略.md:315 | 要改 | 原话「n.rs` 的 `FormatTimeGeometry::slot_sp」已过时，改后：`crates/` 写出了系统配置里那两个字段与槽距算式（`crates/singlefs-core/src/system_configuration.rs` 的 `SystemImmutableSizes::slot_spacing_for`，单测钉住「io_min = 3072 时槽距 6144」）——`FormatTimeGeometry` 在 3cff909b 系统配置拆四类可改性时已改成 `SystemImmutableSizes`（`git grep -F FormatTimeGeometry -- crates` 零命中）。 |
| P30032 | .claude/kb/decisions/05-快照-空间记账机制.md:309 | 不相干 | 原话「y 提交时维护、`check_allocations` 用全量遍历重建」里的名字是bcachefs 的函数，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30033 | .claude/kb/decisions/05-快照-空间记账机制.md:358 | 不相干 | 原话「销毁级联的目标 `dsl_deadlist_merge(&ds_next」里的名字是ZFS 的函数，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30034 | .claude/kb/decisions/05-快照-空间记账机制.md:398 | 不相干 | 原话「m.)」，以及 `dsl_destroy_snapshot_check_impl`：`/* Can」里的名字是ZFS 的函数，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30035 | .claude/kb/decisions/08-核心索引结构.md:379 | 事件句不改 | 原话「only` / `G3_attempt` / `G3_p」引的是 `c161-r3-model-output.txt` 里三种断言的产物行（合法历史上误报），是当时模型输出的名字，不是现有代码。 |
| P30036 | .claude/kb/decisions/08-核心索引结构.md:379 | 事件句不改 | 原话「empt` / `G3_physical` 行）。」引的是 `c161-r3-model-output.txt` 里三种断言的产物行（合法历史上误报），是当时模型输出的名字，不是现有代码。 |
| P30037 | .claude/kb/decisions/08-核心索引结构.md:379 | 事件句不改 | 原话「.txt` 的 `G3_publish_only` / `G3_a」引的是 `c161-r3-model-output.txt` 里三种断言的产物行（合法历史上误报），是当时模型输出的名字，不是现有代码。 |
| P30038 | .claude/kb/decisions/08-核心索引结构.md:445 | 不相干 | 原话「x(` 行与带 `BTREE_IS_write_buffer` 的行，源码固定」里的名字是bcachefs 的标志，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30039 | .claude/kb/decisions/08-核心索引结构.md:445 | 不相干 | 原话「odes` / `rebalance_work` / `acco」里的名字是bcachefs 的 btree 名，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30040 | .claude/kb/decisions/09-加密.md:50 | 不相干 | 原话「中；Linux `chachapoly_setauthsize` 要求 auth」里的名字是Linux crypto 函数，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30041 | .claude/kb/decisions/09-加密.md:50 | 不相干 | 原话「为 16 字节，`crypto_gcm_check_authsize` 拒 10 字节」里的名字是Linux crypto 函数，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30042 | .claude/kb/decisions/09-加密.md:53 | 不相干 | 原话「的默认值：那是 `bch_extent_crc64` 位域挤出来的余」里的名字是bcachefs 的结构，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30043 | .claude/kb/decisions/09-加密.md:209 | 不相干 | 原话「ty.c` 的 `dm_integrity_disable_recalculate`，[prior-」里的名字是Linux dm-integrity 函数，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30044 | .claude/kb/decisions/11-索引节点要不要留消息缓冲区.md:21 | 不相干 | 原话「成独立的一棵树（`pending_ops`）这条出路已否。」是已否出路的示意名（「把消息做成独立的一棵树（`pending_ops`）这条出路已否」），本来就没有代码。 |
| P30045 | .claude/kb/decisions/12-目标介质.md:56 | 不相干 | 原话「btrfs 的 `btrfs_dev_item` 里有 `sec」里的名字是btrfs 的结构，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30046 | .claude/kb/decisions/12-目标介质.md:56 | 不相干 | 原话「容量**带**（`btrfs_stripe` 每条写明 `d」里的名字是btrfs 的结构，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30047 | .claude/kb/decisions/12-目标介质.md:56 | 不相干 | 原话「size` / `io_align` / `io_w」里的名字是btrfs 的字段，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30048 | .claude/kb/decisions/12-目标介质.md:56 | 不相干 | 原话「lign` / `io_width` 三个每设备槽位」里的名字是btrfs 的字段，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30049 | .claude/kb/decisions/13-验证路线.md:288 | 不相干 | 原话「type 隔离（`RotationalPtr` / `Zone」是设计里的 newtype 名，同卷 296 行明说「`crates/` 里没有（零命中）」，现查仍零命中，没有句子把它当现有代码写。 |
| P30050 | .claude/kb/decisions/13-验证路线.md:288 | 不相干 | 原话「lPtr` / `ZonedPtr`），」是设计里的 newtype 名，同卷 296 行明说「`crates/` 里没有（零命中）」，现查仍零命中，没有句子把它当现有代码写。 |
| P30051 | .claude/kb/decisions/13-验证路线.md:296 | 不相干 | 原话「`RotationalPtr` / `Zone」是设计里的 newtype 名，同卷 296 行明说「`crates/` 里没有（零命中）」，现查仍零命中，没有句子把它当现有代码写。 |
| P30052 | .claude/kb/decisions/13-验证路线.md:296 | 不相干 | 原话「lPtr` / `ZonedPtr` 这层 newt」是设计里的 newtype 名，同卷 296 行明说「`crates/` 里没有（零命中）」，现查仍零命中，没有句子把它当现有代码写。 |
| P30053 | .claude/kb/decisions/15-格式冻结政策.md:96 | 不相干 | 原话「ersion`、`frozen_at_commit`、`spec_h」是将来冻结文件的字段，同一决策表第 5 行写着「进度：未实现」，`.claude/kb/format-frozen/` 目录不存在，与句子一致。 |
| P30054 | .claude/kb/decisions/16-发布语义.md:124 | 不相干 | 原话「照：XFS 有 `XFS_SB_FEAT_INCOMPAT_LOG_ALL`，在主系统配置里」里的名字是XFS 的特性位，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30055 | .claude/kb/decisions/16-发布语义.md:124 | 不相干 | 原话「nt.c` 的 `xfs_add_incompat_log_feature` 与 `xfs_」里的名字是XFS 的函数，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30056 | .claude/kb/decisions/16-发布语义.md:124 | 不相干 | 原话「ture` 与 `xfs_clear_incompat_log_features`），**未在本项」里的名字是XFS 的函数，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30057 | .claude/kb/decisions/16-发布语义.md:262 | 不相干 | 原话「ckpoint（`zfs_txg_timeout` 默认 5 秒，」里的名字是ZFS 的参数，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30058 | .claude/kb/decisions/17-实现分层与第三方管道.md:22 | 不相干 | 原话「按用户计费。与 `REQ_PREFLUSH` / `REQ_」里的名字是Linux 块层请求标志，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30059 | .claude/kb/decisions/17-实现分层与第三方管道.md:29 | 不相干 | 原话「SUPP` 的 `REQ_HARDBARRIER` 在 2.6.3」里的名字是Linux 块层请求标志，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30060 | .claude/kb/decisions/17-实现分层与第三方管道.md:29 | 不相干 | 原话「块层自己合成的 `REQ_PREFLUSH` / `REQ_」里的名字是Linux 块层请求标志，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30061 | .claude/kb/decisions/17-实现分层与第三方管道.md:88 | 不相干 | 原话「标志且内核开了 `CONFIG_TRANSPARENT_HUGEPAGE`，而本工程的单元」里的名字是Linux 内核配置项，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30062 | .claude/kb/decisions/17-实现分层与第三方管道.md:88 | 不相干 | 原话「ev.c` 的 `sb_set_blocksize` 对大于 `PA」里的名字是Linux 内核函数，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30063 | .claude/kb/decisions/17-实现分层与第三方管道.md:93 | 不相干 | 原话「17 源码给出 `sb_set_blocksize` 那笔账；用户定」里的名字是Linux 内核函数，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30064 | .claude/kb/decisions/17-实现分层与第三方管道.md:103 | 不相干 | 原话「p.h` 里的 `validity_cookie` 也是因为共享层」里的名字是Linux iomap 的字段，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30065 | .claude/kb/decisions/17-实现分层与第三方管道.md:108 | 不相干 | 原话「txg 脏链表、`vdev_ops` 里四个方法越过」里的名字是ZFS 的结构，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30066 | .claude/kb/decisions/18-块里携带什么信息.md:492 | 不相干 | 原话「**常规读路径 `btrfs_validate_extent_buffer` 对它 0 命中」里的名字是btrfs 的函数，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30067 | .claude/kb/decisions/18-块里携带什么信息.md:492 | 不相干 | 原话「btrfs 的 `chunk_tree_uuid`，16 字节/块」里的名字是btrfs 的字段，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30068 | .claude/kb/decisions/18-块里携带什么信息.md:498 | 不相干 | 原话「位；UBI 的 `image_seq` 解决同类问题）」里的名字是UBI 的字段，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30069 | .claude/kb/decisions/18-块里携带什么信息.md:499 | 不相干 | 原话「」；它的解法是 `copy_flag` + 只有副本才」里的名字是UBI 的字段，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30070 | .claude/kb/decisions/18-块里携带什么信息.md:499 | 不相干 | 原话「只有副本才付的 `data_crc`  **未定*」里的名字是UBI 的字段，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30071 | .claude/kb/decisions/18-块里携带什么信息.md:503 | 不相干 | 原话「销（btrfs `btrfs_header`、bcachef」里的名字是btrfs 的结构，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30072 | .claude/kb/decisions/18-块里携带什么信息.md:503 | 不相干 | 原话「btrfs 的 `chunk_tree_uuid` 常规读路径 0」里的名字是btrfs 的字段，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30073 | .claude/kb/decisions/18-块里携带什么信息.md:561 | 不相干 | 原话「ck.c` 里 `ocfs2_hamming_fix_block` 那一处纠 1 」里的名字是OCFS2 的函数，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30074 | .claude/kb/decisions/19-块指针的结构与宽度预算.md:41 | 不相干 | 原话「p()`），靠 `BTREE_ID_backpointers` 才降到 O(该」里的名字是bcachefs 的 btree 名，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30075 | .claude/kb/decisions/19-块指针的结构与宽度预算.md:41 | 不相干 | 原话「一棵带指针的树（`bch2_dev_usrdata_drop()`），靠 `B」里的名字是bcachefs 的函数，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30076 | .claude/kb/decisions/19-块指针的结构与宽度预算.md:41 | 不相干 | 原话「一层永不消失的 `vdev_indirect_mapping`（只能 cond」里的名字是ZFS 的结构，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30077 | .claude/kb/decisions/19-块指针的结构与宽度预算.md:276 | 不相干 | 原话「um.h` 的 `extent_nonce()` 由 `bv」里的名字是bcachefs 的函数，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30078 | .claude/kb/decisions/19-块指针的结构与宽度预算.md:289 | 不相干 | 原话「Z 全部列位置（`BP_GET_ASIZE` 把 3 个 D」里的名字是ZFS 的宏，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30079 | .claude/kb/decisions/20-承重面单元的原子性与自包含.md:72 | 不相干 | 原话「re.c` 的 `nvme_configure_atomic_write()`）：」里的名字是Linux NVMe 驱动函数，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30080 | .claude/kb/decisions/20-承重面单元的原子性与自包含.md:86 | 不相干 | 原话「re.c` 的 `nvme_configure_atomic_write()`，未在本项目」里的名字是Linux NVMe 驱动函数，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30081 | .claude/kb/decisions/20-承重面单元的原子性与自包含.md:119 | 不相干 | 原话「key` 只有 `drop_writes` / `erro」里的名字是Linux dm-flakey 的选项，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30082 | .claude/kb/decisions/20-承重面单元的原子性与自包含.md:119 | 不相干 | 原话「ites` / `error_writes` / `corr」里的名字是Linux dm-flakey 的选项，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30083 | .claude/kb/decisions/20-承重面单元的原子性与自包含.md:135 | 不相干 | 原话「零引用**  `nvme_configure_atomic_write()` 」里的名字是Linux NVMe 驱动函数，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30084 | .claude/kb/decisions/20-承重面单元的原子性与自包含.md:136 | 不相干 | 原话「re.c` 的 `nvme_configure_atomic_write()`：设备不声明」里的名字是Linux NVMe 驱动函数，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30085 | .claude/kb/decisions/20-承重面单元的原子性与自包含.md:137 | 不相干 | 原话「tear**——`REQ_PREFLUSH`/`REQ_FU」里的名字是Linux 块层请求标志，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30086 | .claude/kb/decisions/20-承重面单元的原子性与自包含.md:146 | 不相干 | 原话「log**  `xlog_pack_data()`：**每 5」里的名字是XFS 日志函数，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30087 | .claude/kb/decisions/20-承重面单元的原子性与自包含.md:147 | 不相干 | 原话「、其余为零」的 `verify_partial` 」里的名字是jbd2（ext4 日志）的函数，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30088 | .claude/kb/decisions/20-承重面单元的原子性与自包含.md:149 | 不相干 | 原话「是 XFS 的 `RWF_ATOMIC` **用户数据*」里的名字是Linux 的写标志，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30089 | .claude/kb/decisions/21-权威态与派生态的分界.md:374 | 不相干 | 原话「rfs.h`）；`btrfs_file_extent_item` = 53；`b」里的名字是btrfs 的结构，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30090 | .claude/kb/decisions/21-权威态与派生态的分界.md:374 | 不相干 | 原话「-08-28）：`btrfs_header` = 32+16」里的名字是btrfs 的结构，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30091 | .claude/kb/decisions/21-权威态与派生态的分界.md:374 | 不相干 | 原话「m` = 53；`btrfs_item` = 25；`S」里的名字是btrfs 的结构，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30092 | .claude/kb/decisions/22-单元原子性怎么合成.md:391 | 不相干 | 原话「key` 只有 `drop_writes` / `erro」里的名字是Linux dm-flakey 的选项，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30093 | .claude/kb/decisions/22-单元原子性怎么合成.md:391 | 不相干 | 原话「ites` / `error_writes` / `corr」里的名字是Linux dm-flakey 的选项，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30094 | .claude/kb/decisions/22-单元原子性怎么合成.md:393 | 不相干 | 原话「tr_t` 有 `blk_cksum`（256 位），」里的名字是ZFS 的字段，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30095 | .claude/kb/decisions/22-单元原子性怎么合成.md:521 | 不相干 | 原话「.h` 只登记 `NVME_ZONE_TYPE_SEQWRITE_REQ`），适用面缩到有」里的名字是Linux NVMe 头文件常量，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30096 | .claude/kb/decisions/23-journal的角色与格式.md:682 | 不相干 | 原话「 CRC，背着 `XLOG_REC_SIZE` / `XLOG」里的名字是XFS 的常量，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30097 | .claude/kb/decisions/23-journal的角色与格式.md:682 | 不相干 | 原话「SIZE` / `XLOG_REC_SIZE_OTHER` 两个常量和两遍」里的名字是XFS 的常量，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30098 | .claude/kb/decisions/23-journal的角色与格式.md:682 | 不相干 | 原话「G2 从块内读 `zc_nused` 决定校验范围，」里的名字是ZFS 的字段，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30099 | .claude/kb/experiments/105-extent叶改码3的代价.md:59 | 事件句不改 | 原话「的时候它临时叫 `PACKED_HDR_WITH_WSEQ`，为的是不让门禁」说的是 2026-09-05 跑的时候它临时叫这个名字，是那一次跑的事。 |
| P30100 | .claude/kb/experiments/107-条带表对主负载的写放大.md:48 | 事件句不改 | 原话「乙臂 `injected_sleep_is_recovered_within_tolerance`（注入 5 ms」在「钉绝对值的断言（跑前写死）」清单里，是跑前登记的断言名；装置 `scripts/experiments/e107_stripe_table_wa.rs` 里乙臂改成了 `injected_recovered_ns` 字段，登记清单是当时写死的，不回改。 |
| P30101 | .claude/kb/experiments/112-旧写者遇到不认识的树.md:104 | 事件句不改 | 原话「自己就写着**：`Refuse_Carry` / `Refu」引的是 E112 产物里的行名（「产物自己就写着」），是那份产物当时的内容。 |
| P30102 | .claude/kb/experiments/112-旧写者遇到不认识的树.md:104 | 事件句不改 | 原话「arry` / `Refuse_NoCarry` 两行同样 `s」引的是 E112 产物里的行名（「产物自己就写着」），是那份产物当时的内容。 |
| P30103 | .claude/kb/experiments/125-顺序区写指针之上读到什么.md:10 | 不相干 | 原话「`zone_capacity` / `zone」里的名字是QEMU 设备参数 / Linux sysfs 项，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30104 | .claude/kb/experiments/125-顺序区写指针之上读到什么.md:10 | 不相干 | 原话「conv` / `zone_max_open` / `zone」里的名字是QEMU 设备参数 / Linux sysfs 项，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30105 | .claude/kb/experiments/128-指针带出生树txg之后的点查代价.md:57 | 事件句不改 | 原话「 未留存（去掉 `tree_per_level` 之前那版源码）」说的是「去掉 `tree_per_level` 之前」的源码与未留存的前两轮，是那几轮的事。 |
| P30106 | .claude/kb/experiments/128-指针带出生树txg之后的点查代价.md:155 | 事件句不改 | 原话「两轮跑的是去掉 `tree_per_level` 之前的源码，*」说的是「去掉 `tree_per_level` 之前」的源码与未留存的前两轮，是那几轮的事。 |
| P30107 | .claude/kb/experiments/129-小于iomin的写在真设备上怎么出事.md:29 | 不相干 | 原话「byte` / `random_write_corrupt` 造的是」里的名字是Linux dm-flakey 的选项，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30108 | .claude/kb/experiments/139-按盘回退下界的收严形态.md:44 | 事件句不改 | 原话「ss` 8 行、`skip_then_raise_r1` / `r2` 」是 E139 产物里的世界名与行数（`git grep -l -F skip_then_raise_r2 -- research/results` 2 份产物里有），说的是那几次重跑读到的数，不是现有代码。 |
| P30109 | .claude/kb/experiments/139-按盘回退下界的收严形态.md:45 | 事件句不改 | 原话「3 行 > 0：`skip_then_raise_r2` S = 4 丢」是 E139 产物里的世界名与行数（`git grep -l -F skip_then_raise_r2 -- research/results` 2 份产物里有），说的是那几次重跑读到的数，不是现有代码。 |
| P30110 | .claude/kb/experiments/139-按盘回退下界的收严形态.md:45 | 事件句不改 | 原话「2 次（16）、`skip_then_raise_r3` S = 4 丢」是 E139 产物里的世界名与行数（`git grep -l -F skip_then_raise_r2 -- research/results` 2 份产物里有），说的是那几次重跑读到的数，不是现有代码。 |
| P30111 | .claude/kb/experiments/139-按盘回退下界的收严形态.md:186 | 事件句不改 | 原话「里确实起了作用（`verdict_window_gate_reach` 的 `brok」引的是 E139 产物里 `verdict_window_gate_reach` 行的读数（`research/results` 里 2 份产物有），说的是那次重跑的结果。 |
| P30112 | .claude/kb/experiments/139-按盘回退下界的收严形态.md:221 | 事件句不改 | 原话「产物里 `verdict_window_gate_reach` 的七类不注入根」引的是 E139 产物里 `verdict_window_gate_reach` 行的读数（`research/results` 里 2 份产物有），说的是那次重跑的结果。 |
| P30113 | .claude/kb/experiments/139-按盘回退下界的收严形态.md:249 | 事件句不改 | 原话「6 行 > 0（`skip_then_raise_r2` / `r3` 」是 E139 产物里的世界名与行数（`git grep -l -F skip_then_raise_r2 -- research/results` 2 份产物里有），说的是那几次重跑读到的数，不是现有代码。 |
| P30114 | .claude/kb/experiments/139-按盘回退下界的收严形态.md:311 | 事件句不改 | 原话「 > 0，都在 `skip_then_raise_r2` / `r3`、」是 E139 产物里的世界名与行数（`git grep -l -F skip_then_raise_r2 -- research/results` 2 份产物里有），说的是那几次重跑读到的数，不是现有代码。 |
| P30115 | .claude/kb/experiments/139-按盘回退下界的收严形态.md:329 | 事件句不改 | 原话「至多活两圈  `skip_then_raise_r3`、S = 4 的」是 E139 产物里的世界名与行数（`git grep -l -F skip_then_raise_r2 -- research/results` 2 份产物里有），说的是那几次重跑读到的数，不是现有代码。 |
| P30116 | .claude/kb/experiments/139-按盘回退下界的收严形态.md:505 | 事件句不改 | 原话「第一条命令算出（`max_x_per_h_x100` 除以 100；」是那一轮第一条命令输出里的字段名，说的是表是怎么算出来的，是那一次的事。 |
| P30117 | .claude/kb/experiments/139-按盘回退下界的收严形态.md:507 | 事件句不改 | 原话「；F10 触发（`skip_then_raise_r3`、S = 4 的」是 E139 产物里的世界名与行数（`git grep -l -F skip_then_raise_r2 -- research/results` 2 份产物里有），说的是那几次重跑读到的数，不是现有代码。 |
| P30118 | .claude/kb/experiments/139-按盘回退下界的收严形态.md:525 | 事件句不改 | 原话「；余下 6 行（`skip_then_raise_r2` / `r3`、」是 E139 产物里的世界名与行数（`git grep -l -F skip_then_raise_r2 -- research/results` 2 份产物里有），说的是那几次重跑读到的数，不是现有代码。 |
| P30119 | .claude/kb/experiments/139-按盘回退下界的收严形态.md:533 | 事件句不改 | 原话「4. `skip_then_raise_r3`、S = 4 这」是 E139 产物里的世界名与行数（`git grep -l -F skip_then_raise_r2 -- research/results` 2 份产物里有），说的是那几次重跑读到的数，不是现有代码。 |
| P30120 | .claude/kb/experiments/142-新池新建文件的干跑.md:42 | 事件句不改 | 原话「umbers`、`system_configuration_is_481_bytes_and_the_slot_checksum_covers_all_4096` 三条测试，源码」说变异 M29 单独复跑时红在哪三条测试，是那一次复跑的事。 |
| P30121 | .claude/kb/experiments/142-新池新建文件的干跑.md:253 | 事件句不改 | 原话「el`（①⑧）；`header311_write_digest` 48 行的 `」是 E142 产物里的行名与字段（`research/results` 里 2 份产物有），说的是产物怎么比对，不是现有代码。 |
| P30122 | .claude/kb/experiments/15-异构设备几何的模拟是否成立.md:49 | 不相干 | 原话「，QEMU 的 `rotation_rate` 只改声明不改延」里的名字是QEMU 设备参数，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30123 | .claude/kb/experiments/153-账本形态与环上有洞的代价.md:29 | 事件句不改 | 原话「s`、循环变量 `g12_index`、测试名里的 `」说这几处被命名纪律点名后「已改名」，是改名这件事本身，旧名零命中正是改名的结果。 |
| P30124 | .claude/kb/experiments/153-账本形态与环上有洞的代价.md:29 | 事件句不改 | 原话「点名的 3 处（`snapshot_q9_reads`、循环变量 `g」说这几处被命名纪律点名后「已改名」，是改名这件事本身，旧名零命中正是改名的结果。 |
| P30125 | .claude/kb/experiments/153-账本形态与环上有洞的代价.md:32 | 事件句不改 | 原话「ly` 行追加 `q7_g8prime_red_states`。」是那一轮往 `q_family` 行追加的字段（记录当时做了什么），之后命名纪律改名了；说的是那一次的改动。 |
| P30126 | .claude/kb/experiments/153-账本形态与环上有洞的代价.md:33 | 事件句不改 | 原话「ly` 行追加 `q10_interval_inference_mismatches`。」是那一轮往 `q_family` 行追加的字段（记录当时做了什么），之后命名纪律改名了；说的是那一次的改动。 |
| P30127 | .claude/kb/experiments/153-账本形态与环上有洞的代价.md:36 | 事件句不改 | 原话「母/字母加数字（`max_value`、`bound_」说命名纪律点名的名字「已全部改名」，是改名这件事本身。 |
| P30128 | .claude/kb/experiments/155-每次持久化的写量三种fsync形态与反事实上界.md:26 | 事件句不改 | 原话「算格逐字节相同（`q3_1_general_implementation_matches_the_hand_calculated_cell_at_file_count_1`）。Q4.1 是」是那一轮写了什么、钉了什么的记录（Q3.1 在 P=1 一格与手算格逐字节相同、门槛出现在 P=14→15 之间），后来测试改名后旧名零命中；说的是那一轮的事。 |
| P30129 | .claude/kb/experiments/155-每次持久化的写量三种fsync形态与反事实上界.md:26 | 事件句不改 | 原话「取整字节」串成 `q3_1_tree_savings`，六棵树各自的 」是那一轮写了什么、钉了什么的记录（Q3.1 在 P=1 一格与手算格逐字节相同、门槛出现在 P=14→15 之间），后来测试改名后旧名零命中；说的是那一轮的事。 |
| P30130 | .claude/kb/experiments/155-每次持久化的写量三种fsync形态与反事实上界.md:99 | 事件句不改 | 原话「s_tests::release_append_only_reaches_two_layers_at_p_star_fifteen` 只钉「门槛出现」是那一轮写了什么、钉了什么的记录（Q3.1 在 P=1 一格与手算格逐字节相同、门槛出现在 P=14→15 之间），后来测试改名后旧名零命中；说的是那一轮的事。 |
| P30131 | .claude/kb/experiments/155-每次持久化的写量三种fsync形态与反事实上界.md:120 | 不相干 | 原话「6×、1024=`nr_requests`=1023 向上」里的名字是Linux 块层 sysfs 项，说的是别家实现或设备，本仓 `crates/`、`scripts/` 零命中是本来如此，不是被删或改名。 |
| P30132 | .claude/kb/experiments/156-alloc-basis四条岔路的代价数.md:109 | 事件句不改 | 原话「rSource::WithoutRootRingPositiveControl`、`alloca」说那一轮装置怎么改的（「删 `HhAllocatorSource::WithoutRootRingPositiveControl`」），删除是那一轮的事。 |
| P30133 | .claude/kb/experiments/156-alloc-basis四条岔路的代价数.md:150 | 事件句不改 | 原话「sult`  `consistent_on_12_sampling_points`  `unst」是 E156 产物里 `result` 字段的两个读数（`research/results` 里 5 份产物有），是那份产物的内容。 |
| P30134 | .claude/kb/experiments/157-并行线一两条条款的计数模型.md:6 | 事件句不改 | 原话「ataUnit::ExtentLeafKeyOffsetUnit`，即 C490（」说跑前登记时被测的是哪条未定条款，是登记当时的事。 |
| P30135 | .claude/kb/experiments/157-并行线一两条条款的计数模型.md:6 | 事件句不改 | 原话「未定条款之一——`UndecidedClauseBlockingMoreThanOneDataUnit::ExtentL」说跑前登记时被测的是哪条未定条款，是登记当时的事。 |
| P30136 | .claude/kb/experiments/158-择根与修复四岔路.md:457 | 事件句不改 | 原话「didate`、`RollbackToVersionWithoutFileWhileTheRingStillHoldsAFileVersion`），逐条现查确认」说那一轮「逐条现查确认」剪枝分支只有两类，是那一轮现查的结果。 |
| P30137 | .claude/kb/experiments/158-择根与修复四岔路.md:479 | 事件句不改 | 原话「gory`  `RollbackToVersionWithoutFileWhileTheRingStillHoldsAFileVersion`（`crates」自己写着「一个已删除变体」，与现查零命中一致，说的是装置当时怎么分类。 |
| P30138 | .claude/kb/experiments/158-择根与修复四岔路.md:581 | 事件句不改 | 原话「Φ1 故障注入（`attempt_op1`：按 (设备, 」是那一轮装置里写了哪些件的流水（H1 全族枚举、Φ1 故障注入、PC1-a/PC1-b），是那一轮的事。 |
| P30139 | .claude/kb/experiments/158-择根与修复四岔路.md:584 | 事件句不改 | 原话「a/PC1-b（`run_pc1_a_and_b`：按固定次序找第」是那一轮装置里写了哪些件的流水（H1 全族枚举、Φ1 故障注入、PC1-a/PC1-b），是那一轮的事。 |
| P30140 | .claude/kb/experiments/158-择根与修复四岔路.md:854 | 事件句不改 | 原话「，只把它已有的 `subset_enumeration_cap` 参数」说那一轮复用既有搜索函数的哪个参数，后文写明 session s9 已改名为 `weight_ceiling`，是带日期的过程记录。 |
| P30141 | .claude/kb/experiments/158-择根与修复四岔路.md:856 | 事件句不改 | 原话「把 `subset_enumeration_cap` 参数改名为 `」说那一轮复用既有搜索函数的哪个参数，后文写明 session s9 已改名为 `weight_ceiling`，是带日期的过程记录。 |
| P30142 | .claude/kb/experiments/158-择根与修复四岔路.md:942 | 事件句不改 | 原话「—比乙族多一个 `published_txg_to_carry_over` 方法 + 取号」说三份副本是怎么建的、开工时快照的哪几个读数，是那一轮的事。 |
| P30143 | .claude/kb/experiments/158-择根与修复四岔路.md:943 | 事件句不改 | 原话「_lines`/`crates_sha256_of_sha256s` 与」说三份副本是怎么建的、开工时快照的哪几个读数，是那一轮的事。 |
| P30144 | .claude/kb/experiments/158-择根与修复四岔路.md:943 | 事件句不改 | 原话「t_head`/`git_status_crates_lines`/`crates」说三份副本是怎么建的、开工时快照的哪几个读数，是那一轮的事。 |
| P30145 | .claude/kb/experiments/161-崩溃放量的去重与分段耗时.md:41 | 事件句不改 | 原话「mall` 3、`first_quick` 3，K4-un」引的是 E161 产物里的格名或判定字段（`research/results` 里 2–4 份产物有），说的是那两次跑读到的数，不是现有代码。 |
| P30146 | .claude/kb/experiments/161-崩溃放量的去重与分段耗时.md:41 | 事件句不改 | 原话「uick` 6、`second_quick` 3，其余格 0」引的是 E161 产物里的格名或判定字段（`research/results` 里 2–4 份产物有），说的是那两次跑读到的数，不是现有代码。 |
| P30147 | .claude/kb/experiments/161-崩溃放量的去重与分段耗时.md:50 | 事件句不改 | 原话「mall` 与 `first_quick` 上 Q3f 大」引的是 E161 产物里的格名或判定字段（`research/results` 里 2–4 份产物有），说的是那两次跑读到的数，不是现有代码。 |
| P30148 | .claude/kb/experiments/161-崩溃放量的去重与分段耗时.md:54 | 事件句不改 | 原话「_small`、`first_quick`、`second」引的是 E161 产物里的格名或判定字段（`research/results` 里 2–4 份产物有），说的是那两次跑读到的数，不是现有代码。 |
| P30149 | .claude/kb/experiments/161-崩溃放量的去重与分段耗时.md:54 | 事件句不改 | 原话「_quick`、`second_quick` 上都是 `tr」引的是 E161 产物里的格名或判定字段（`research/results` 里 2–4 份产物有），说的是那两次跑读到的数，不是现有代码。 |
| P30150 | .claude/kb/experiments/161-崩溃放量的去重与分段耗时.md:183 | 事件句不改 | 原话「- `first_segment_head_k2_consult_constant_key_separates`、`first_」引的是 E161 产物里的格名或判定字段（`research/results` 里 2–4 份产物有），说的是那两次跑读到的数，不是现有代码。 |
| P30151 | .claude/kb/experiments/161-崩溃放量的去重与分段耗时.md:183 | 事件句不改 | 原话「arates`、`first_segment_head_k2_ignore_constant_key_separates`、`first_」引的是 E161 产物里的格名或判定字段（`research/results` 里 2–4 份产物有），说的是那两次跑读到的数，不是现有代码。 |
| P30152 | .claude/kb/experiments/161-崩溃放量的去重与分段耗时.md:183 | 事件句不改 | 原话「arates`、`first_segment_head_k4_full_constant_key_separates`、`second」引的是 E161 产物里的格名或判定字段（`research/results` 里 2–4 份产物有），说的是那两次跑读到的数，不是现有代码。 |
| P30153 | .claude/kb/experiments/161-崩溃放量的去重与分段耗时.md:183 | 事件句不改 | 原话「arates`、`second_segment_head_k2_consult_constant_key_separates`、`second」引的是 E161 产物里的格名或判定字段（`research/results` 里 2–4 份产物有），说的是那两次跑读到的数，不是现有代码。 |
| P30154 | .claude/kb/experiments/161-崩溃放量的去重与分段耗时.md:183 | 事件句不改 | 原话「arates`、`second_segment_head_k2_ignore_constant_key_separates`、`second」引的是 E161 产物里的格名或判定字段（`research/results` 里 2–4 份产物有），说的是那两次跑读到的数，不是现有代码。 |
| P30155 | .claude/kb/experiments/161-崩溃放量的去重与分段耗时.md:183 | 事件句不改 | 原话「arates`、`second_segment_head_k4_full_constant_key_separates` 六个：两个段头」引的是 E161 产物里的格名或判定字段（`research/results` 里 2–4 份产物有），说的是那两次跑读到的数，不是现有代码。 |
| P30156 | .claude/kb/experiments/161-崩溃放量的去重与分段耗时.md:184 | 事件句不改 | 原话「- `second_k2_consult_pc2c_pc4c_found_a_position`、`second」引的是 E161 产物里的格名或判定字段（`research/results` 里 2–4 份产物有），说的是那两次跑读到的数，不是现有代码。 |
| P30157 | .claude/kb/experiments/161-崩溃放量的去重与分段耗时.md:184 | 事件句不改 | 原话「sition`、`second_k2_ignore_pc2c_pc4c_found_a_position`、`second」引的是 E161 产物里的格名或判定字段（`research/results` 里 2–4 份产物有），说的是那两次跑读到的数，不是现有代码。 |
| P30158 | .claude/kb/experiments/161-崩溃放量的去重与分段耗时.md:184 | 事件句不改 | 原话「sition`、`second_k4_full_pc2c_pc4c_found_a_position` 五个：覆盖写固」引的是 E161 产物里的格名或判定字段（`research/results` 里 2–4 份产物有），说的是那两次跑读到的数，不是现有代码。 |
| P30159 | .claude/kb/experiments/161-崩溃放量的去重与分段耗时.md:184 | 事件句不改 | 原话「sition`、`second_k4_units_pc2c_pc4c_found_a_position`、`second」引的是 E161 产物里的格名或判定字段（`research/results` 里 2–4 份产物有），说的是那两次跑读到的数，不是现有代码。 |
| P30160 | .claude/kb/experiments/161-崩溃放量的去重与分段耗时.md:184 | 事件句不改 | 原话「sition`、`second_k4_walk_pc2c_pc4c_found_a_position`、`second」引的是 E161 产物里的格名或判定字段（`research/results` 里 2–4 份产物有），说的是那两次跑读到的数，不是现有代码。 |
| P30161 | .claude/kb/experiments/161-崩溃放量的去重与分段耗时.md:185 | 事件句不改 | 原话「- `first_k4_full_pc2c_pc4c_found_a_position`：第 1 次是 」引的是 E161 产物里的格名或判定字段（`research/results` 里 2–4 份产物有），说的是那两次跑读到的数，不是现有代码。 |
| P30162 | .claude/kb/experiments/161-崩溃放量的去重与分段耗时.md:188 | 事件句不改 | 原话「mall` 与 `first_quick` 的 K4-wa」引的是 E161 产物里的格名或判定字段（`research/results` 里 2–4 份产物有），说的是那两次跑读到的数，不是现有代码。 |
| P30163 | .claude/kb/experiments/161-崩溃放量的去重与分段耗时.md:188 | 事件句不改 | 原话「 次 3、6）、`second_quick` 的 K4-un」引的是 E161 产物里的格名或判定字段（`research/results` 里 2–4 份产物有），说的是那两次跑读到的数，不是现有代码。 |
| P30164 | .claude/kb/experiments/161-崩溃放量的去重与分段耗时.md:223 | 事件句不改 | 原话「_small`、`first_quick` 两格 V3 触」引的是 E161 产物里的格名或判定字段（`research/results` 里 2–4 份产物有），说的是那两次跑读到的数，不是现有代码。 |
| P30165 | .claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:339 | 事件句不改 | 原话「out` 三行 `r2_table_both_profile_check` 都是 `ide」引的是 E161 产物里的格名或判定字段（`research/results` 里 2–4 份产物有），说的是那两次跑读到的数，不是现有代码。 |
| P30166 | .claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:341 | 事件句不改 | 原话「out` 三行 `r2_profile_check` 的 `iden」引的是 E161 产物里的格名或判定字段（`research/results` 里 2–4 份产物有），说的是那两次跑读到的数，不是现有代码。 |
| P30167 | .claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:414 | 事件句不改 | 原话「out` 四行 `old_product_check`，`identi」引的是 E161 产物里的格名或判定字段（`research/results` 里 2–4 份产物有），说的是那两次跑读到的数，不是现有代码。 |
| P30168 | .claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:414 | 事件句不改 | 原话「相同（同一文件 `r3_against_r2_check` 一行，`rem」引的是 E161 产物里的格名或判定字段（`research/results` 里 2–4 份产物有），说的是那两次跑读到的数，不是现有代码。 |
| P30169 | .claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:414 | 事件句不改 | 原话「整份逐字节相同（`threads_check` 一行，`sha」引的是 E161 产物里的格名或判定字段（`research/results` 里 2–4 份产物有），说的是那两次跑读到的数，不是现有代码。 |

## 要主 agent 现查

1. P30021、P30022（`.claude/kb/checks-owed.md:446` C668）与 P30020（`:426` C638）：依据是别的会话暂存的 `crash_identity.rs` 重写（白名单表与 `random_history_scaffolding_introducers_judged` 已删、`lexical_digest_matches_the_e164_anchors` 已不在）。暂存若回退，这三行改回不判；若提交，C668 要销账或改成新机制的同类问题，C638 的「3231 / 21 / 7 条」要重数（`#[test]` 函数数我没重数）。
2. P30015（`:321` C457）：`worker_threads_by_default()` 已换成 `worker_threads_from_environment()`，现取登记的每台默认重活线程数、不再夹到 16。整条 C457「叠加超订阅」的前提（随机历史测试自己起最多 16 个线程）要按现在的取法重核，我只判了函数名与取法那一句，没重判整条欠账是否仍成立。
3. P30025（`:649` C12）：点名的测试已搬到 checker 档同名用例，`must_hold` 在 `crates/` 零命中；`crates/mutations.tsv:537` 那一行的改法列我只看了第一、二、五列，没逐字对「`allocated_slots() * SLOT_BYTES` 换成 0」。
4. P30014（`:314` C436）：`mask_code()` 在 `.claude/singlefs-ai-sop` 与 `scripts` 的 git 历史里从没出现过（`git log --all -S'mask_code(' -- .claude/singlefs-ai-sop scripts` 空），现名是 `mask_code_spans`（`link-targets.py:42`）；当时上游版本是否叫 `mask_code` 我无从核，改前请看上游。
5. P30002（`agent-common.md:62`）：改动在 `.claude/agent-common.md`，该文件我不碰，也不是 kb-scribe 范围，主 agent 自己改。
6. P30024（`:560` C842）：句里写「全仓 0 命中」，实际 `checks-owed.md`、一份 records 里还有这个名字（说它已删的话）；指的是代码，未判要改。
7. 搜索范围的一点：P1 探针对 `checker/image.rs`、`checker/walk.rs`、`checker/src/walk.rs` 三个简写（P10040、P10070–P10072）判不相干；若要全仓统一写全路径，是风格治理，不是搬迁腐烂。

## 汇总

| 判定 | 行数 |
|---|---|
| 要改 | 8 |
| 要补 | 0 |
| 事件句不改 | 82 |
| 不相干 | 93 |
| 要人看 | 0 |
| 合计 | 183 |

数行命令：`grep -c '^| P[0-9]* |' <本报告>` 应为 183（表头、分隔行不算）。

## 没做什么

- 没改任何仓内文件；没建副本、没起编译、没跑门禁；没读 `~/.claude/`。
- 别家实现的名字（内核、btrfs、bcachefs、ZFS、XFS、UBI、OCFS2、dm-flakey 等）只判了「说的是别家」，没有逐个核对它们在别家源码里是否仍叫这个名字。
- 事件句不改的行里，引产物行名的（E139、E142、E156、E161、E165 等），只核了 `research/results` 或 `research/prompts` 里这个名字在，没逐份核对产物里的读数。
- 推翻条件：任何一行「事件句不改」的句子若放在「现状」口吻里（例如被当成复跑指令），则判错；任何一行「要改」的名字若在工作区重新出现，则那一行判错。

## 校验

`--check-report` 输出末行（退出码 0）：

    ✓ 报告判全了：183 行候选都有逐行判定
