# m3-prune-gpu-r1 事实表甲：崩溃验证全景与可共享的前缀

调度记录 `records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 任务甲。只读代码与 kb，没编译、没跑任何测试。
行号都是 2026-09-28 这一次现查的（`grep -n` / `awk 'NR==n'`）；标「推的」的是从代码读出来的推论，没有跑出来的数撑。
状态数一律照用例里钉死的常量或门禁登记行原样引，另标出处；百分比是拿那几个常量现算的（`python3`）。

## 一　今天仓里的崩溃验证路径（问题 1）

### 1.1 总表

| # | 路径 | 入口（文件:行 函数） | 枚举什么 | 每个状态上依次调 | 结果累计 | 谁跑 |
|---|---|---|---|---|---|---|
| P1 | 层 0 第一条流（新池新建文件：取号 → 暖机 × 2 → A） | `crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs:498` `layer0_enumerates_every_crash_state_of_the_settled_stream_with_zero_violations`，经同文件 `:402` `enumerate_counting_allocation_generation_read_sets` → `crash.rs:2583` `enumerate_layer0_in_state_slices_or_one_shard` | 段模型全量（见 1.2），`FULL_STATES = 16_777_260`（同文件 `:446`），快档 `QUICK_TIER_STATES = 46`（`:450`） | 见 1.3 的层 0 流水线；另带观察者：每个状态交给 `allocation_generation_read_set_in`（`:416`–`:425`）记 I-3.10 那一类计数 | `Layer0Tally`（`crash.rs:743`），观察者计数进 `Layer0ObserverCounts`（`crash.rs:784`） | 54 号 `--full`，`crash-case:layer0-first-stream`（`.claude/gate.d/stage-inputs.tsv:36`），可双机分片 |
| P2 | 层 0 第二条流（固定脚本到 E 再正常卸载） | `crash_enumeration_fixed_script_stream.rs:797` `full_enumeration_of_the_fixed_script_stream_is_exhaustive_and_clean` → `crash.rs:2583` | `FULL_STATES_THROUGH_THE_UNMOUNT = 1_662_648_564`（同文件 `:100`），两态口径 `1_662_648_449`（`:96`），快档 278（`:114`） | 层 0 流水线，无观察者（`:826` 传 `None`） | `Layer0Tally` | 54 号 `--full`，`stage-inputs.tsv:37`，可双机分片 |
| P3 | 层 0 并行线一流（多记录发布） | `crash_enumeration_multi_record_publish_stream.rs` 全量用例（`:394` 起调 `enumerate_layer0_in_state_slices_or_one_shard`） | 文件头 `:21` 写「层 0 的枚举域 1358954634 个状态」 | 层 0 流水线 | `Layer0Tally` | 54 号 `--full`，`stage-inputs.tsv:40`，可双机分片 |
| P4 | 树分裂七条流 | `crash_enumeration_tree_split_streams.rs` 全量用例，经同文件 `enumerate`（`:356` 调 `enumerate_layer0_selecting_versions`） | 每条只录分裂那一次发布，段 `[2u, 2, 1, 2]`（`:307`–`:308`）；七条合计 78905428（`stage-inputs.tsv:41` 注释） | 层 0 流水线 | 每条一份 `Layer0Tally` | 54 号 `--full`，`stage-inputs.tsv:41`，不认分片、不续跑 |
| P5 | 位置寻址树流 | `crash_enumeration_position_addressed_trees.rs:480` `full_enumeration_of_every_enumerable_position_addressed_tree_stream_is_exhaustive_and_clean`，经 `:427` `enumerate_layer0_selecting_versions` | 只录被判那一次发布或挂载；用例不钉状态数，只断言等于闭式（`:435`–`:450` 那一条 `assert_eq!`）；注释写「状态数上亿级」（`:476`，没钉数） | 层 0 流水线 | `Layer0Tally` | 54 号 `--full`，`stage-inputs.tsv:42`，不认分片、不续跑 |
| P6 | 会话推的抬 F 串 | `crash_enumeration_floor_raise_pushed_by_admission.rs`（`:153` 调 `enumerate_layer0_in_state_slices`） | 一条历史里只录抬 F 那一串，`STATES_AT_MOST = 1_000_000`（`:43`） | 层 0 流水线，外加每个状态恢复之后与再挂载之后各跑池级 checker（文件头 `:3`–`:5`） | `Layer0Tally` + 用例自己的断言 | 54 号 `--full`，`stage-inputs.tsv:38` |
| P7 | C561 σ 段全量 | `record_checker_judges_absence_by_the_persisted_set.rs:719` → 同文件 `:581` `enumerate_every_subset_of_one_segment` | σ 一段的全部子集，`assert_eq!(enumerated.states, 65_536)`（`:731`–`:733`） | 只有一遍看 journal 的恢复 + 记录核对器（`:369`–`:380` `judge`：`recover(..Consult)`、`check_records`）；不跑不看 journal 那一遍、不跑 oracle、不跑池级 checker | 用例自己的计数（`states`、`reported_missing`，`:649`–`:660`） | 54 号 `--full`，`stage-inputs.tsv:39` |
| P8 | 崩溃注入（随机历史上抽样） | `crates/singlefs-checker-tier/src/crash_injection.rs:1979` `run_crash_injection_campaign` → `:722` `inject_crashes_into_history`；快档用例 `tests/crash_injection_campaign.rs:135` | 24 段历史 × 每段 24 步 × 每段抽 4 个崩溃点（`crash_injection_campaign.rs:41`–`:45`）；域同层 0 的段模型，但**不取第三态**（`crash_injection.rs` 里 `Tearable\|torn` 0 处，模块头 `:5` 写「撕裂并进没持久」） | 见 1.4 | `CrashInjectionTally`（`crash_injection.rs:376`） | 54 号 `--full`，`stage-inputs.tsv:43`；大档 `:963` 不登记 |
| P9 | 故障注入 | `crates/singlefs-harness/src/fault_injection.rs:2723` `run_fault_injection_campaign` → `:1842` `inject_faults_into_history` → `:2173` `inject_one_fault` | 不是崩溃状态枚举：按种子在读 / 写 / 刷盘调用里摆注入点，每个注入点**整段历史重跑一遍**（`:1940`–`:1972`）；快档 24 段 × 20 步 × 4 个注入点（`tests/fault_injection_fast_tier.rs:49`–`:51`） | 重跑到注入点 → 录制流整份重建镜像（`:2263`–`:2267` `MemoryPool::with_devices` + `apply`）→ `recover(..Consult)`（`:2273`）→ 模型判定 → 池级 checker（`:2377`）；被吞的写插回去的那份镜像再跑一次 checker（`:2395` 起） | `FaultInjectionTally`（`:1364`） | harness 档，随时跑 |
| P10 | 坏盘输入 | `crates/singlefs-checker-tier/src/bad_disk_input.rs:2694` `run_bad_disk_campaign` → `:950` `feed_bad_disk_inputs_from_history` → `:1156` `feed_a_damaged_image` | 不是崩溃状态：把合法镜像按种子改坏 | 恢复（`:1163`）、可写挂载（`:1184`）、池级 checker（`:1198`），三者都包在 panic 捕获里 | `BadDiskTally`（`:556`） | checker 档快档（普通用例）；大档 `tests/bad_disk_input_campaign.rs:1035` 不登记 |
| P11 | QEMU 真设备 | `.claude/gate.d/55-qemu-device-streams.sh`，装置 `crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs` 与 `new_pool_file_creation_device_log_check.rs` | 不枚举崩溃状态：六次虚机跑，设备侧 blklogwrites 日志与程序录制流逐项比（55 号文件头 `:8`–`:21`）；文件头 `:5`–`:6` 写明「今天只有真实负载与设备侧录制、没有崩溃注入」 | 冷重开读回 | 阶段自己判 | 55 号，提交时 |

P1–P7 共用同一个枚举器（`crates/singlefs-checker-tier/src/crash.rs`），P7 例外（用例自己按掩码切片，`record_checker_judges_absence_by_the_persisted_set.rs:565` `mask_slices`）。
P8、P10 共用种子切片 `crash_injection.rs:2116` `seed_slices`（`bad_disk_input.rs:2703` 调它）；P9 在 harness 档另有一份 `fault_injection.rs:2836` `seed_slices`。

另有住在 checker 档、不登记 `crash-case:`、归 54 号快档（`cargo test --release -p singlefs-checker-tier --lib --tests`）的崩溃状态用例：
按文件数的现查（`grep -c '#\[test\]'` 与 `grep -c '#\[ignore'`，逐文件）里 `ignored=0` 的那几份，例如
`crash_enumeration_acquisition_barrier.rs`（只展开两段）、`crash_enumeration_record_spill_over_stream.rs`（单元段 136 写不展开，25 个状态，文件头 `:14`–`:16`）、
`crash_enumeration_writable_mount_of_a_formatted_pool.rs`、`crash_points_of_the_rollback_publish.rs`、`crash_points_tree_identifier_watermark_orphans.rs`（后两份自己逐个造崩溃状态，不经 `crash.rs` 枚举器）。
研究侧另有一条独立实现：E142 装置（`research/e7-index-bench/src/bin/e142_new_pool_file_creation_dry_run.rs`，`stage-inputs.tsv` 的 `E142/layer0` 行），手写模型、不读 `crates/`；以及 E161 装置 `crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs`（文件头 `:10`–`:14`：第一条流经 `build_pool`、第二条流脚本照抄 fixed_script 用例，枚举计划自己一份、与 crates 逐状态对拍）。

### 1.2 段模型与闭式（P1–P6 共用）

| 事实 | 出处 |
|---|---|
| 切段：一段录制流拆成写表与段，「当前段里每块有写的盘都被自己的屏障或 FUA 放行了才关段」 | `crates/singlefs-harness/src/memory_pool.rs:514`–`:515` 文档、`:547` `writes_and_segments_with_stream_indexes_and_entries`、关段在 `:597`–`:599` |
| 状态 = 前面的段全持久 + 当前段每次写各取几态的任意组合（去掉整段全持久）+ 最后一个「全部持久」 | `crash.rs:2177`–`:2179` 文档；`crash.rs:1813`–`:1815` |
| 每次写几态：原地覆写（不是单元写、长于一个扇区、罩住的范围原来有东西）取三态「没持久 / 新旧都读不出 / 持久」，其余两态 | `crash.rs:1484`–`:1488`，判法 `crash.rs:1566` `is_tearable_in_place_overwrite` |
| 全量一段的状态数 `3^m · 2^(n−m) − 1`；甲二快档 `3^m · 2^(k−m) ·（c>0 时 2，否则 1）− 1` | `crash.rs:1397`、`:1400`–`:1401`；实现 `crash.rs:1412` `Layer0SegmentExpansion::state_count` |
| 整条流闭式 = 各段之和 + 1 | `crash.rs:1785` `state_count_of_the_segments`（`try_fold(1u64, …)`，`:1797`） |
| 两态口径的闭式 `1 + Σ(2^|段| − 1)` | `memory_pool.rs:611` `closed_form_state_count` |
| 状态序号 → 段：各段序号区间首尾相接，`partition_point` 定段 | `crash.rs:1849`–`:1859`、`:1898` `segment_of_state` |
| 段内序号按混合进制拆到段内每次写，第一次写是最低位 | `crash.rs:1875`–`:1895` `assign_landings` |
| 撕裂态的字节：枚举开始时一次性算好，每个原地覆写的写在写表末尾接一条撕裂镜像，再接同段里在它之后、同盘重叠的写各一份重放 | `crash.rs:1631`–`:1634` 文档、`:1648` `WritesWithTornImages::of`（在 `:2634` 调一次） |

### 1.3 层 0 一个状态上依次调了什么（P1–P6）

本体 `crash.rs:1131` `evaluate_state_recording_findings`，每个状态恰好这一串：

| 次序 | 调用 | 行 |
|---|---|---|
| 1 | 由计划给出持久集合 `Vec<bool>`（工作线程里调） | `crash.rs:2143` `plan.persisted_writes_of_state(ordinal)` |
| 2 | `newest_persisted_root(writes, &persisted)`（线性扫写表） | `crash.rs:1141`；定义 `memory_pool.rs:812` |
| 3 | 建 `CrashImage { base, writes, persisted }`（不物化，见第五节） | `crash.rs:1142`–`:1146` |
| 4 | 恢复两遍：`recover(&image, JournalPolicy::Consult)`、`recover(&image, JournalPolicy::Ignore)` | `crash.rs:1147`、`:1148` |
| 5 | oracle 两遍：看 journal 那一遍判 `violations`，不看的判 `ignored_violations` | `crash.rs:1168`、`:1190`（`classified_oracle_violation_for_versions`，`:968`） |
| 6 | 池级 checker 一遍：`check_pool_image(&image)`，逐条不变量记评估 / 违例 / 不适用 | `crash.rs:1210`–`:1231` |
| 7 | 记录核对器一遍：`check_records(&image, consulted.effective_root)` | `crash.rs:1247`（→ `:105` `check_records_against`） |
| 8 | 判红的记进发现表（按签名去重） | `crash.rs:1182`–`:1266`（`Layer0Findings::record_red_state`，`:615`） |
| 9 | 有观察者时，状态的 `persisted` 与恢复报告带回调用线程、按序号交观察者 | `crash.rs:2144`–`:2156`、`:2861`–`:2874` |

累计：每片一份 `Layer0Tally`，调用线程按片号从小到大 `absorb_following_slice`（`crash.rs:859`：计数相加，「第一处」取序号最小的一片的）。字段表 `crash.rs:743`–`:779`。

P7 只做第 3、4（只 Consult）、7 步（`record_checker_judges_absence_by_the_persisted_set.rs:369`–`:380`），注释 `stage-inputs.tsv:39` 写明「不经 enumerate_layer0_in_state_slices（那一路每个状态多跑一遍不看 journal 的恢复与池级 checker、状态集合差一个）」。

### 1.4 崩溃注入（P8）一个崩溃点上依次调了什么

入口 `crash_injection.rs:722` `inject_crashes_into_history`：先整段跑一遍历史录流（`:735`），切段（`:767`–`:768`，与层 0 同一个切段函数），按种子抽崩溃点（`:774` → `:1673` `draw_crash_points`，快档 `Sampled { crash_points_per_history: 4 }`，`tests/crash_injection_campaign.rs:43`–`:45`）。崩溃点按段号排好之后逐个：

| 次序 | 调用 | 行 |
|---|---|---|
| 1 | 基线只往前叠：更早的段整段 `base.apply_writes(...)`，不为每个崩溃点从头重建 | `crash_injection.rs:800`–`:802` |
| 2 | 建整条流长度的持久集合 `vec![false; writes.len()]`，更早的段填 true，当前段按子集 | `:803`–`:809` |
| 3 | `CrashImage { base: &base, writes: 当前段那几次写, persisted: 段内子集 }`（叠加表只有当前段） | `:810`–`:815` |
| 4 | `recover(&image, JournalPolicy::Consult)` 一遍（没有 Ignore 那一遍） | `:846` |
| 5 | 模型判定 `crash_recovery_disagreement` | `:857`–`:858` |
| 6 | 记录核对器 `check_records_against(&image, &image, &writes, &persisted, …)`（整条历史的写表与持久集合） | `:872`–`:879` |
| 7 | 池级 checker `checker_violations_on(&image, …)` | `:887` |
| 8 | 第二截：`materialized_crash_image(&image)` 把崩溃镜像物化成一个池（`base.clone()` + 叠持久的写，`:1049`–`:1057`），在上面可写挂载（`:1123`）、发一次布（`:1139`），记录核对器（`:940`–`:941`）、池级 checker（`:1374`） | `:915`–`:950` |
| 9 | 第三截：挂载那一段录制流切段，取号 / 写行 / 暖机各按种子摆一个二次崩溃（`:1477` `draw_second_crash_points`）；每个二次崩溃：第二份基线同样只往前叠（`:1574`–`:1585`），`recover(..Consult)`（`:1603`）、模型（`:1615`）、记录核对器（`:1621`）、池级 checker（`:1627`） | `:951`–`:965`、`:1543` |

累计：每段历史一份 `CrashInjectionTally`，调用线程按片次序收齐后逐段 `tally.absorb`（`:2081`–`:2089`）。

## 二　共享前缀（问题 2）

### 2.1 三条从 mkfs 起的整条流，段序列逐段对照

三条流都从 `build_pool(tag)`（`crates/singlefs-harness/tests/common/mod.rs:372`）起：mkfs → 取号 → 暖机 × 2 → A（新池新建文件），基线都是 `memory_pool_after_mkfs()`（`common/mod.rs:194`，只施加 mkfs 那几步），写表都从 `mkfs_operation_count` 之后切。

| 流 | 用例里钉的段长数组 | 出处 | 段数 | 写数 |
|---|---|---|---|---|
| 第一条（P1） | `[2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2]` | `crash_enumeration_new_pool_file_creation_stream.rs:466` | 11 | 41（`:470`） |
| 第二条（P2） | `[2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 2, 18, …, 16, 2, 1, 2]` | `crash_enumeration_fixed_script_stream.rs:439`–`:442` | 78（`:95` 注释） | `191 + 5 * 33 + (2 + 2 * 21) + 33 + (2 + 2 * 21)` = 477（`:448`） |
| 并行线一（P3） | `[2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 28, 4, 1, 2, 30, 6, 1, 2]` | `crash_enumeration_multi_record_publish_stream.rs:185` | 19 | 用例不钉写数；段长之和 115（现算） |
| 只做过 mkfs 的池（不登记，不另枚举） | 段序列与第一条逐项相同 | `crash_enumeration_writable_mount_of_a_formatted_pool.rs:214`–`:237`（基线、写表连同内容、段序列逐项 `assert`） | 11 | 同第一条 |

**相同到第几段**：三条的前 11 段（第 0–10 段：取号 2、暖机一 `2,1,2`、暖机二 `2,1,2`、A 的单元 24、A 的记录 2、A 的根槽 1、A 的系统配置轮换 2）逐段同长；第 11 段起分开（第二条是 B 的 24 个单元写，并行线一是 B 的 28 个单元写）。第二条与并行线一在第 11 段就不同，两两之间没有更长的公共前缀。

**逐字节相同**：只有「只做过 mkfs 的池」那条与第一条之间有用例逐字节核（上表末行）。第一条与第二条、并行线一之间**没有用例核写表逐字节相同**；推的：三条都由同一个 `build_pool` 走同一串确定性调用（固定内容 `file_content()`、固定写时 `FIXED_WRITE_TIME_SECONDS`、固定 `filesystem_identifier: E142_FILESYSTEM_IDENTIFIER`，`common/mod.rs:48`），`tag` 只进镜像文件名（`common/mod.rs:38`–`:41`），前 41 次写应逐字节相同；「只做过 mkfs 的池」那条用例拿两个不同 `tag` 建的池比出逐项相同，是这条推论的一个旁证。

### 2.2 公共前缀那 11 段各有几个状态

逐段分解按第一条流用例里的算式（`crash_enumeration_new_pool_file_creation_stream.rs:443`–`:445`：「取三态的只有 8 次系统配置槽写，四个系统配置槽段……各 3² − 1、A 的单元段 2²⁴ − 1，其余同两态，再加全部持久那一个：1 + 4 × 8 + 3 × 3 + 3 × 1 + (2²⁴ − 1) = 16777260」）；哪一段是哪一种写照用例自己的说明（`crash_enumeration_new_pool_file_creation_stream.rs:472`「取号 2 + 暖机两次各 5 + A 29（24 个单元写、两份记录、根槽、两块盘的系统配置槽轮换）」，`crash_enumeration_fixed_script_stream.rs:388`–`:393`）；`.claude/kb/layout/01-first-txn.md:410` 的种类串是 C577 之前的形状，今天对不上（见第六节）。每段的数是按算式拆的（推的），合计与用例钉死的 `FULL_STATES`、`QUICK_TIER_STATES` 相等。

| 段 | 写数 | 内容 | 全量状态数 | 甲二快档状态数 | 归哪次发布 |
|---|---|---|---|---|---|
| 0 | 2 | 取号两块盘的系统配置槽（三态） | 8 | 8 | txg 1 |
| 1 | 2 | 暖机一的两份记录 | 3 | 3 | txg 1 |
| 2 | 1 | 暖机一的根槽 FUA | 1 | 1 | txg 1 |
| 3 | 2 | 暖机一的系统配置槽轮换（三态） | 8 | 8 | txg 2 |
| 4 | 2 | 暖机二的两份记录 | 3 | 3 | txg 2 |
| 5 | 1 | 暖机二的根槽 | 1 | 1 | txg 2 |
| 6 | 2 | 暖机二的系统配置槽轮换（三态） | 8 | 8 | txg 3 |
| 7 | 24 | A 的 12 个单元 × 2 盘 | 2²⁴ − 1 = 16777215 | 1 | txg 3 |
| 8 | 2 | A 的两份记录 | 3 | 3 | txg 3 |
| 9 | 1 | A 的根槽 | 1 | 1 | txg 3 |
| 10 | 2 | A 的系统配置槽轮换（三态） | 8 | 8 | 第一条流里是 `after_the_last_root`；第二条流里归 txg 4 |
| — | — | 第一条流末尾「全部持久」 | 1 | 1 | `every_write_persisted` |
| 合计 | 41 | | **16777260**（= `FULL_STATES`，`:446`） | **46**（= `QUICK_TIER_STATES`，`:450`） | 第一条流按发布分钉在 `:565`：`instance1_txg1=12 instance1_txg2=12 instance1_txg3=16777227 after_the_last_root=8 every_write_persisted=1` |

第二条流用例钉的按发布分（`crash_enumeration_fixed_script_stream.rs:105`–`:110`）前三格同为 `instance1_txg1=12 instance1_txg2=12 instance1_txg3=16777227`，与上表一致。

**第一条流的整个枚举域都含在另两条流里**：第一条流最后那个「全部持久」状态，在第二条流与并行线一里就是第 11 段的空子集（段内序号 0，`crash.rs:1917`–`:1936`：前面的段全持久、当前段按序号拆，序号 0 各写都取 0 = 没持久）。所以另两条流的状态序号 `[0, 16777260)` 与第一条流的全部状态一一对应、崩溃镜像相同（镜像相同以 2.1「逐字节相同」那条推论为前提）。快档同理：第二条流第 11 段只有单元写，甲二只取「全不落」那一个（`crash.rs:1947`–`:1957`：有单元写时序号除以 2 拆到原地写、余数 1 才让单元写全落；这一段没有原地写，序号 0 就是单元写全不落），第一条流的 46 个快档状态也含在第二条流的 278 个里（推的）。

| 流 | 全量状态数 | 其中与第一条流共享的 | 占比（现算） |
|---|---|---|---|
| 第一条 | 16777260 | 16777260（它自己） | 100% |
| 第二条 | 1662648564（`crash_enumeration_fixed_script_stream.rs:100`） | 16777260 | 1.009% |
| 并行线一 | 1358954634（`crash_enumeration_multi_record_publish_stream.rs:21` 文件头） | 16777260 | 1.235% |
| 三条合计 | 3038380458 | 重复枚举 2 × 16777260 = 33554520 | 1.104% |

前缀里真正贵的是第 7 段（A 的 24 个单元写，16777215 个状态，占前缀的 99.9998%）；去掉它，前缀其余 10 段加末尾一个只有 45 个状态（现算：16777260 − 16777215）。

### 2.3 这些前缀今天是不是每条流各枚举一遍

是。依据：

| 事实 | 出处 |
|---|---|
| 每条用例各自 `prepare`：各自 `build_pool` 重跑整条写路径、各自切段、各自调枚举器 | P1 `crash_enumeration_new_pool_file_creation_stream.rs:458`–`:489`；P2 `crash_enumeration_fixed_script_stream.rs:218`–`:493`；P3 `crash_enumeration_multi_record_publish_stream.rs:147`–`:185` |
| 枚举器每一趟从状态 0 数到状态总数，没有「从某个序号起」「跳过某段」的入参；能跳过的只有 `NotExpanded`（整段不展开，只以全持久进入后面） | `crash.rs:1393`–`:1404` `Layer0SegmentExpansion`；`crash.rs:2644`–`:2647` 切片从 `plan.state_count` 起 |
| 断点续跑与分片账本按「输入指纹 + 流名 + 计划哈希」分格，计划哈希含整张写表、段、版本表、有没有观察者；不同流的格互不相认，没有跨流复用 | `layer0_progress.rs:4`–`:6` 文件头；`crash.rs:2286`–`:2290` `layer0_plan_hash` 文档 |
| 全绿标记按用例分格（`singlefs-crash-case-green.<用例名>.<输入指纹>`） | `.claude/gate.d/54-layer0-replay.sh:19` |
| 唯一「相同就不另枚举」的先例：只做过 mkfs 的池那条流，用例逐项核「基线、写表连同内容、段序列」与第一条流相同，54 号文件头写「由 cargo test 里的快用例钉住，不另枚举」 | `crash_enumeration_writable_mount_of_a_formatted_pool.rs:210`–`:237`；`.claude/gate.d/54-layer0-replay.sh:4` |
| 另一种已在用的写法「起点镜像不枚举，只录被判那一次发布」：树分裂、位置寻址、记录跨条三类流把之前那一大截施加成基线（`memory_pool_before_the_current_version`，`crates/singlefs-harness/tests/common_tree_split/mod.rs:401`–`:406`），只枚举最后那一次发布的段；它们的起点前缀（mkfs、取号、暖机、A、铺垫的发布，节点容量用只供测试的开关压小）没有哪条流枚举过 | `crash_enumeration_tree_split_streams.rs:3`–`:5`、`:298`；`crash_enumeration_position_addressed_trees.rs:3`–`:4`、`:148`；`crash_enumeration_record_spill_over_stream.rs` 文件头 |

## 三　每条流「走到哪些代码模块」有没有机械登记（问题 3）

没有。今天能找到的登记只到「整个 `crates/`」或「checker 档自己的模块」这一粒度：

| 登记 | 粒度 | 出处 |
|---|---|---|
| 8 条 `crash-case:` 行的路径列一律是 `crates/ Cargo.toml Cargo.lock`，不按用例列文件 | 整个 crates 目录 | `.claude/gate.d/stage-inputs.tsv:36`–`:43`（逐行现查，`grep -c '^crash-case:'` = 8） |
| 算一条用例的输入指纹时自动减去三类读不到的文件：① 别的测试目标独占的测试文件；② `crates/mutations.tsv`；③ 没有代码读 `CARGO_BIN_EXE_` 时各包 `src/bin/` 下的文件；其余全留，另加判法摘要、工具链、构建环境、登记行本身，分片用例再加驱动脚本 | 文件级减法，不做模块可达性 | `research/scripts/admission.py:1187`–`:1199` |
| 注释原话「登记只写整个 crates/，不按用例手列它读哪些文件：新加的共用文件（tests/common.rs、build.rs、新拆出的 crate）默认就在输入里」 | — | `research/scripts/admission.py:1200` |
| 改成整份 `crates/` 的来由：层 0 规模第二轮攻方量过按流拆的清单会漏新加的 `tests/common.rs`、`build.rs`、新拆出的 crate | — | `research/prompts/m2-layer0-scale-r2-main-verification.md:23`（M3 那一行） |
| 54 号整道阶段的路径列同样是 `crates/ Cargo.toml Cargo.lock` 加三份研究脚本 | 整个 crates | `.claude/gate.d/stage-inputs.tsv:28` |
| checker 档测试文件第一行声明 `//! checker 档模块：…`，登记的是它从 `singlefs_checker_tier::` 导入了哪几个 checker 档模块（crash、layer0_progress、crash_injection、bad_disk_input、device_log、on_device_modes），不是它走到 `singlefs-core` 的哪些模块 | checker 档模块 | `.claude/rules/verification.md:40`；判法 `research/scripts/crash-case-check.py:157`、`:194`–`:199` |
| 57 号的 litmus 锚点把一条 litmus 绑到代码里的锚点（今天指 `crates/singlefs-core/src/transaction.rs` 与 `recovery.rs`），管的是内存序模型，不是崩溃流 | litmus ↔ 代码锚点 | `.claude/gate.d/stage-inputs.tsv:30` 注释 |
| 覆盖率工具：仓里 `cargo llvm-cov` 只出现在重型测试闸的命令分类里（把它认成起测试的子命令），没有任何脚本或门禁拿它量「一条流走到哪些函数」 | — | `.claude/hooks/lib_heavy_tests.py:229`、`:372`、`:381` |

所以今天改 `crates/` 下任何一个进指纹的文件（包括与某条流无关的 `singlefs-core` 模块），8 条崩溃枚举用例的指纹都变、全绿标记都不再作数（按 `admission.py` 那段减法推的；没有逐文件量过）。
里程碑三第一项把「节点走到哪些代码模块怎么机械地登记（不靠手列清单）」列为开工前要定的（`.claude/kb/milestone/03-third-txn.md:35`）。

## 四　状态调度与 `Vec<bool>`（问题 4）

### 4.1 `Vec<bool>` 与 `[bool]` 的每一处（非测试代码）

数的命令：以 `#[cfg(test)]` 所在行为界（`crash.rs:3160`、`crash_injection.rs:2131`），`awk` 数界之前含 `Vec<bool>` 的行：crash.rs 7 行、crash_injection.rs 9 行；`vec![false|true; n]` crash.rs 2 行、crash_injection.rs 3 行；参数 `&[bool]` crash.rs 2 行、crash_injection.rs 3 行。`crates/singlefs-checker/src/` 四个文件 `Vec<bool>|[bool]` 都是 0。

| 文件:行 | 写法 | 表示什么 | 长度 | 每个状态建一次吗 |
|---|---|---|---|---|
| `crash.rs:1043` | `evaluate_state` 参数 `persisted: Vec<bool>` | 一个崩溃状态的持久集合（与写表逐条对应），单版本入口 | 写表长 | 调用方给 |
| `crash.rs:1062` | `evaluate_state_for_versions` 参数 | 同上，多版本入口（手摆状态用） | 写表长 | 调用方给 |
| `crash.rs:1134` | `evaluate_state_recording_findings` 参数 | 同上，本体；进 `CrashImage.persisted`（`:1142`–`:1146`） | 写表长 | 是（按值移进） |
| `crash.rs:1493` | `TearableInPlaceOverwrites.is_tearable_by_write` | 录制流里第 i 次写是不是原地覆写（取三态） | 录制流写数 | 否，一趟枚举建一次（`:2633`） |
| `crash.rs:1504` | `TearableInPlaceOverwrites::of` 里的局部 | 同上的构造 | 录制流写数 | 否 |
| `crash.rs:1525` | `vec![false; write_count]` | `TearableInPlaceOverwrites::none`：全部只取两态 | 录制流写数 | 否 |
| `crash.rs:1917` | `persisted_writes_of_state` 的返回值 | 状态序号 → 持久集合，按**枚举用的写表**（录制流的写 + 撕裂镜像 + 重放） | 枚举写表长 | **是**，每个状态现建一份 |
| `crash.rs:1962` | `vec![false; self.writes_with_torn_images.writes.len()]` | 上一行的本体：先全 false，再按落法填 true，撕裂态另把撕裂镜像与重放那几格填上（`:1963`–`:1983`）；它之前还有一份 `landings: Vec<WriteLandingInCrashState>`（`:1920`，三值枚举，不是 bool） | 枚举写表长 | **是** |
| `crash.rs:2115` | `FinishedSlice.observed_states: Vec<(Vec<bool>, RecoveryReport)>` | 有观察者时一片里每个状态的持久集合与恢复报告，带回调用线程 | 片长 × 枚举写表长 | 有观察者时每个状态 `clone` 一份（`:2149`） |
| `crash.rs:112` | `check_records_against` 参数 `persisted: &[bool]` | 被核记录流的持久集合 | 记录流长 | — |
| `crash.rs:194` | `unit_copy_is_missing_under_the_persisted_set` 参数 | 同上，判一份单元副本缺不缺席 | — | — |
| `memory_pool.rs:434` | `CrashImage.persisted: Vec<bool>` | 崩溃镜像 = 基线 + 这些持久了的写（按写表次序叠） | 与 `CrashImage.writes` 同长 | 随状态 |
| `memory_pool.rs:796`、`:814`、`:963` | `some_publish_persisted_without_its_root`、`newest_persisted_root`、`publishes_in` 的 `&[bool]` 参数 | 同一个持久集合，各扫一遍 | — | — |
| `crash_injection.rs:203` | `CrashPoint.persisted_within_the_segment` | 崩溃点所在那一段里第 k 个写持久没有 | 段长 | 每个崩溃点一份 |
| `crash_injection.rs:803` | `vec![false; writes.len()]` 局部 `persisted` | 整条历史流的持久集合（更早段 true、当前段按子集、更晚段 false），交记录核对器与 `newest_persisted_root` | 整条历史写表长 | 是，每个崩溃点 |
| `crash_injection.rs:1192` | `HistoryThenWritableMountRecords.persisted_in_the_history` | 第一次崩溃时历史那一段的持久集合（接缝前） | 历史写表长 | 每个崩溃点 |
| `crash_injection.rs:1248` | `persisted_with_the_mount` 返回值 | 历史 + 挂载段 + 之后那次发布拼起来的持久集合 | 三段之和 | 每次核对 |
| `crash_injection.rs:1294` | `vec![true; self.writes_of_the_mount]` | 第二截：挂载那一段全落 | 挂载写数 | 每个崩溃点 |
| `crash_injection.rs:1471` | `SecondCrashPoint.persisted_within_the_segment` | 二次崩溃所在挂载段里的子集 | 段长 | 每个二次崩溃 |
| `crash_injection.rs:1509` | 局部，`draw_second_crash_points` 里造上一行 | w 之前 true、w false、之后按种子 | 段长 | 同上 |
| `crash_injection.rs:1587` | `vec![false; mount_writes.len()]` | 整条挂载流的持久集合（二次崩溃） | 挂载写数 | 每个二次崩溃 |
| `crash_injection.rs:1693` | `BTreeSet<(usize, Vec<bool>)>` | 抽到的（段号, 段内子集）去重集合 | — | 每段历史一份 |
| `crash_injection.rs:1752` | `bits_of` 返回值 | 掩码低 n 位摊成逐写的 bool | 段长 | 每个抽样 |
| `crash_injection.rs:1758`、`:1763` | `draw_a_proper_subset` 返回值与局部 | 按种子抽一个真子集（段长 ≤ 63 抽掩码，否则逐写抽） | 段长 | 每个抽样 |

### 4.2 层 0 的状态怎么分给线程

| 事实 | 出处 |
|---|---|
| 单位是「片」= 状态序号的连续区间 `Range<u64>`，首尾相接覆盖 `[0, 状态数)` | `crash.rs:2003` `state_slices` |
| 不续跑时片长按线程数定：片数 `max(64, 16 × 线程数)`，每片至少 16 个状态 | `crash.rs:1293`–`:1297` 三个常量；`:2005`–`:2013` |
| 续跑、分片、merge 时片长只看状态数：片数取 65536，每片 `max(⌈状态数 / 65536⌉, 16)` 个状态（换线程数续跑片方案不变） | `crash.rs:2029` `LAYER0_RESUMABLE_SLICE_COUNT`、`:2032` `states_per_slice_independent_of_worker_threads`、`:2043` `slicing_of_the_run` |
| 按这个切法现算三条整条流的片长：第一条 257 个状态一片、65282 片；第二条 25371 个一片、65534 片；并行线一 20737 个一片、65533 片（`python3` 现算，没跑） | — |
| 线程：`std::thread::scope` 里起 `min(线程数, 待跑片数)` 个工作线程，共用一个 `AtomicUsize` 游标 `fetch_add(1)` 领下一片（动态领片，不预分） | `crash.rs:2742`、`:2774`–`:2811`（`:2776` scope，`:2792` fetch_add） |
| 线程数：环境变量 `SINGLEFS_LAYER0_THREADS`，没设取 `available_parallelism` | `crash.rs:1290`、`:1346` `Layer0Parallelism::from_environment` |
| 一片在一个线程上按序号逐个评：`for ordinal in slice` → `persisted_writes_of_state` → `evaluate_state_recording_findings` | `crash.rs:2124` `evaluate_state_slice`、`:2136`–`:2168` |
| 交回：`mpsc::channel` 发 `FinishedSlice`；调用线程按片号次序并（先到的暂存进 `BTreeMap`），每收一片打一行 `LAYER0_PROGRESS` | `crash.rs:2778`、`:2837`–`:2913` |
| 基线 `MemoryPool`、枚举写表、计划只读、各线程共用；每个状态自己的 `Vec<bool>` 与 `CrashImage` 在工作线程上现建 | `crash.rs:2118`–`:2119` 文档 |
| P7（C561 σ）另有一份切片：掩码区间，片数 `max(64, 16 × 线程数)`，同样 scope + 游标 | `record_checker_judges_absence_by_the_persisted_set.rs:565`–`:573`、`:592`–`:623` |
| P8 / P10：种子区间切片，片数 `min(种子数, 4 × 线程数)`，同样 scope + 游标；一片 = 若干段历史，一段历史的全部崩溃点在同一个线程上串行跑 | `crash_injection.rs:2116`–`:2129`、`:2006`–`:2037`；`bad_disk_input.rs:2703`、`:2714` |
| P9：harness 档自己一份种子切片与 scope | `fault_injection.rs:2744`、`:2836` |

### 4.3 断点续跑（`layer0_progress.rs`）的单位

| 事实 | 出处 |
|---|---|
| 单位是片：一片跑完、观察者看完、没 panic 之后，把这一片的整份 `Layer0Tally` 写成一行追加进进度文件并 `sync_data` | `layer0_progress.rs:1186` `append_finished_slice`；调用 `crash.rs:2875`–`:2881` |
| 被杀时最多丢每个线程手上那一片；读回时核过的片不再跑，末尾半行丢掉、那一片重跑 | `crash.rs:2027`–`:2028` 注释；`layer0_progress.rs:8`–`:9` |
| 进度文件按「输入指纹 + 流名 + 枚举计划哈希」分格，三样都进文件名 | `layer0_progress.rs:4`–`:6` |
| 计划哈希拼的是：基线每个扇区、枚举写表（含撕裂镜像与重放）、段与每段展开方式、被判的根、版本表、状态数、片方案、有没有观察者（分片时另拼 `shard <i>/<n>`） | `crash.rs:2286`–`:2290` |
| 进度目录由 54 号设：`<git common-dir>/singlefs-layer0-progress/<这条用例的输入指纹>` | `.claude/gate.d/54-layer0-replay.sh:23` |

### 4.4 双机分片（`research/scripts/layer0-shard-run.sh`）的单位

| 事实 | 出处 |
|---|---|
| 单位也是片（续跑那一套切法）：第 i 台只跑 `slice_index % n == i` 的片（交错分，不是切成两大段） | `crash.rs:2680`–`:2683`；`layer0_progress.rs:167` `owns_slice`、`:186` `shard_owning_slice` |
| 驱动固定两台：本机跑 0/2、第二台跑 1/2，第二台的账本拷回本机，本机 `merge/2` 读两份账本按片号并 | `research/scripts/layer0-shard-run.sh:2`–`:4` |
| 账本与进度文件同一种片行，另加文件头（工具链、输入指纹、片方案）；merge 核两份账本的文件头相同、片不重不漏 | `layer0_progress.rs:14`–`:17`；`crash.rs:2567`–`:2569` |
| 只接登记了 `shard=across-machines` 的用例：今天是第一条、第二条、并行线一三条（P1–P3）；树分裂、位置寻址不认分片开关 | `stage-inputs.tsv:36`、`:37`、`:40`；`:41`、`:42` 注释 |
| 推的：因为交错分片，第二条流里与第一条流共享的那 661 片左右（16777260 ÷ 25371 ≈ 661.3）两台各跑一半 | 现算 |

## 五　相邻状态之间差什么、镜像怎么造（问题 5）

### 5.1 层 0：一个状态的镜像不物化，是「基线 + 持久集合」的叠加视图

| 事实 | 出处 |
|---|---|
| 状态的镜像就是 `CrashImage { base, writes, persisted }`，只借用基线与写表，自己只持有 `persisted: Vec<bool>`；没有一份按状态物化的盘面 | `crates/singlefs-harness/src/memory_pool.rs:428`–`:435`；建在 `crash.rs:1142`–`:1146` |
| 每一次读（恢复与 checker 都经它读）：先从基线读（稀疏盘 `BTreeMap` 按扇区区间查，`memory_pool.rs:49`–`:66`），再**逐条扫整张写表**，持久且同盘、与读区间重叠的写把重叠那段拷上去 | `memory_pool.rs:445`–`:473`（循环在 `:454`） |
| journal 记录槽的提示、checker 的候选单元槽与候选 journal 槽，同样每次调用逐条扫整张写表 | `memory_pool.rs:478`–`:511`、`:741`–`:751`、`:752`–`:770` |
| P1–P3 的基线是 mkfs 之后那一版（`memory_pool_after_mkfs`，`common/mod.rs:194`–`:196`），不随段前移：第二条流第 60 段上的状态，每次读都要把前面几百次持久写逐条叠一遍 | 同上；写表长 477（`crash_enumeration_fixed_script_stream.rs:448`），枚举写表再接 46 条系统配置槽写各自的撕裂镜像与重放（`:97`–`:99` 注释说取三态的是 46 次系统配置槽写；重放条数没现算） |
| 每个状态的持久集合从头现建：先 `landings = vec![NotPersisted; 录制流写数]`，把当前段之前每一段的每次写标成持久（逐写循环），再拆当前段，最后新建 `vec![false; 枚举写表长]` 按落法填 | `crash.rs:1917`–`:1985` |
| 撕裂镜像的字节不在每个状态上算：枚举开始时 `WritesWithTornImages::of` 一次性算好——克隆基线一次、按次序叠写到每个原地覆写之前取旧字节 | `crash.rs:1669`–`:1682`；调用 `crash.rs:2634` |
| 造一个状态「镜像」花的函数：`Layer0StatePlan::persisted_writes_of_state`（`crash.rs:1917`）+ 结构体字面量（`:1142`）；之后的代价摊在每一次读上：`<CrashImage as PoolReader>::read`（`memory_pool.rs:445`）→ `<MemoryPool as PoolReader>::read`（`:396`）→ `SparseDevice::read_into`（`:49`）+ 逐写 `WrittenContents::copy_range_into`（`:261`）；`ImageReader` 那一侧 `memory_pool.rs:726`–`:771` 转回同一个 `read` | 同左 |
| 一个状态上这套叠加视图被读几遍：两遍恢复、一遍池级 checker、一遍记录核对器（1.3 第 4、6、7 步）各自独立读，互不共享读出来的字节 | `crash.rs:1147`、`:1148`、`:1210`、`:1247` |

### 5.2 同一段内相邻两个状态差什么

| 事实 | 出处 |
|---|---|
| 段内序号按混合进制拆，第一次写是最低位、每位的进制是它能取几态（2 或 3） | `crash.rs:1875`–`:1895` `assign_landings` |
| 所以序号 r → r + 1 就是这个混合进制数加一：最低位那次写换一态，进位时连着换后面几次写。两个相邻状态的持久集合只在这几次写上不同，镜像只在这几次写罩住的字节区间上不同，每一处都是**整次写**（或整次写的撕裂镜像及其重放），不会差半次写（推的，从 `assign_landings` 与 `persisted_writes_of_state` 读出；没有用例量过「相邻状态平均差几次写」） | 同上；`crash.rs:1962`–`:1983` |
| 次序不是格雷码：全两态的段上，r 到 r + 1 平均要翻约 2 次写（二进制加一翻动的位数期望，推的） | — |
| 跨段：段 k 的最后一个序号是「最低位取次大、其余取最大」（全两态时 = 段内第一次写没持久、其余持久），下一个序号是段 k + 1 的序号 0（段 k 全持久、段 k + 1 全不持久），两者只差段 k 的第一次写（推的） | `crash.rs:1420`（段内状态数 = 组合数 − 1）、`:1898`–`:1901` |
| 甲二快档：有单元写的段里序号的奇偶决定「单元写全不落 / 全落」，相邻两个状态差这一段的**全部**单元写 | `crash.rs:1947`–`:1957` |
| 今天没有任何增量：每个状态的持久集合、叠加视图、两遍恢复、checker、记录核对器都从头来，不看上一个状态 | 1.3 那张表；`crash.rs:2136`–`:2168` 的循环体里没有跨状态携带的变量（只有 `tally` 与 `observed_states`） |

### 5.3 另几条路径怎么造镜像

| 路径 | 怎么造 | 出处 |
|---|---|---|
| P8 崩溃注入 | 基线随崩溃点段号只往前叠（增量），叠加视图里只放当前段那几次写，读的代价只随段长涨；但每个崩溃点为可写挂载**整份克隆**一次物化镜像（`materialized_crash_image`：`base.clone()` + 叠持久的写），二次崩溃的第二份基线再克隆一次 | `crash_injection.rs:800`–`:815`、`:915`、`:1049`–`:1057`、`:1574` |
| P7 σ | 与层 0 同一个 `CrashImage`，写表是整条历史，每个状态 `persisted.to_vec()` | `record_checker_judges_absence_by_the_persisted_set.rs:373`–`:377`、`:607`–`:610` |
| P9 故障注入 | 每个注入点整段历史重跑一遍，再从录制流整份重建镜像（`MemoryPool::with_devices` + `apply`） | `fault_injection.rs:2263`–`:2267` |
| P4、P5、spill-over | 基线是被录那一次发布之前的整份镜像（整段 `apply` 物化一次），写表只有那一次发布 | `common_tree_split/mod.rs:401`–`:406` |

## 六　顺带看到的数对不上（只记，不改）

C577（发布返回之前加屏障，轮换自成一段）之后用例里的常量改了，下面几处文字还是改之前的数。没跑 52 号，不知道它今天判不判红。

| 位置 | 写的 | 代码今天钉的 |
|---|---|---|
| `.claude/kb/layout/01-first-txn.md:410` | 第一条流 `2+2+1+2+2+1+26+2+1+2`、67108885 个状态，种类串里 `[unit_write×24,system_configuration_slot×2,barrier]` 同段 | `[2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2]`（`crash_enumeration_new_pool_file_creation_stream.rs:466`），两态闭式 16777240（`:442`），三态 16777260（`:446`） |
| `.claude/kb/layout/01-first-txn.md:412` | 第二条流 61 段、`closed_form=6649413746`、快档 232 | 78 段（`crash_enumeration_fixed_script_stream.rs:439`–`:442`，现数 78 个数）、两态 1662648449（`:96`）、三态 1662648564（`:100`）、快档 278（`:114`） |
| `.claude/gate.d/stage-inputs.tsv:39` 注释 | σ「2^18」 | 65536（`record_checker_judges_absence_by_the_persisted_set.rs:731`–`:733`，`:712` 注释「C577 之前 σ 带上一次的轮换、2^18 个」） |
| `.claude/gate.d/stage-inputs.tsv:40` 注释 | 并行线一「枚举域 12230590578 个状态」 | 1358954634（`crash_enumeration_multi_record_publish_stream.rs:21`，同行写 12230590578 是 C577 之前的） |
| `crash_enumeration_new_pool_file_creation_stream.rs:497` ignore 串 | 「全量六千七百多万个状态」 | 16777260 |
| `crash_enumeration_fixed_script_stream.rs:796` ignore 串 | 「全量五十多亿个状态」 | 1662648564 |
| `crash.rs:2028` 注释 | 「第二条流全量五十多亿个状态时每片约八万五千个」 | 按 `:2029`–`:2037` 现算 25371 个一片 |
| `.claude/kb/milestone/03-third-txn.md:46` | `Vec<bool>` 在 crash.rs「7 处」、crash_injection.rs「5 处」（2026-09-26 现查） | 这一次按「非测试区含 `Vec<bool>` 的行」数：crash.rs 7、crash_injection.rs 9（口径见 4.1；那一次的口径没写，不知道差在口径还是代码） |

## 七　什么现象会推翻这份事实表里的结论

| 结论 | 推翻它的观测 |
|---|---|
| 三条从 mkfs 起的流前 11 段逐字节相同、第一条流的全部 16777260 个状态在另两条里各重复一遍 | 在同一次编译里分别 `prepare` 三条流，比 `writes[..41]`（连同内容）与基线：有一条不等就不成立（今天没有这条用例；只做过 mkfs 的池那条用例是唯一的逐字节比对） |
| 共享前缀上三条流的判定相同 | 同一个前缀状态在第一条与第二条流里 oracle / checker / 记录核对器判得不同。已知会不同的只有计数口径：`root_persisted_states` 按各自的被判根（第一条是 A 的根，第二条是 txg 19 的根）数，版本表不同（第一条单版本 `single_version`，`:391`），第一条多一个观察者；oracle 的 `find` 按 (txg, 实例) 精确找、`NoFile` 那一支只找更旧的版本（`crash.rs:993`–`:1019`），推的：前缀状态上结论相同 |
| 流走到哪些模块没有机械登记 | 仓里找到一份按流列 `singlefs-core` 模块或函数的登记、或一份覆盖率产物；这一次搜了 `stage-inputs.tsv`、`admission.py`、`crash-case-check.py`、`llvm-cov`/`tarpaulin`/`grcov` 全仓字面 |
| 层 0 每个状态从头建持久集合、不物化镜像、读的时候逐写叠加 | `persisted_writes_of_state` 或 `CrashImage::read` 里出现跨状态的缓存或增量；今天两处都没有（`crash.rs:1917`–`:1985`、`memory_pool.rs:445`–`:473`） |

## 八　没做什么

- 没编译、没跑任何测试或门禁（派发要求；另一会话的层 0 全量占着两台机器），所以文中所有状态数都是用例常量或现算，不是这一次跑出来的。
- 没有写一条「三条流前缀逐字节相同」的比对去跑，只做了推论（第七节第一行是它的推翻办法）。
- 没现算枚举写表里撕裂镜像之后的重放条数（第二条流 46 条撕裂镜像之外还接几条重放）。
- 没数位置寻址那几条流的状态数（用例不钉数，只断言等于闭式）。
- 相邻状态平均差几次写、每个状态花在哪一步，没有量（后者归任务乙与 E161）。
- 第六节列的不一致没改，也没判 52 号今天红不红。
