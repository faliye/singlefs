# 里程碑三第十一项事实调查：崩溃放量的过程与结果怎么存（KV 存储）

2026-09-28。调度记录 `records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 任务丁。只读代码、kb 与产物；没编译、没跑任何测试与实验、没改仓里别的文件。行号都是现取的（`grep -n` / `awk 'NR==…'`）；引产物整行抄；没量过的写「推的」。

## 一、E162（崩溃放量判定块存储选型）全貌

### 1.1 文件在哪

| 物 | 路径 | 现状 |
|---|---|---|
| 实验页 | `.claude/kb/experiments/162-崩溃放量判定块存储选型.md`（203 行） | 标题 :1「## E162 崩溃放量判定块存储选型 —— S1 够判档已交，S2 / S3 还没跑（2026-09-27）」；S3 回环半份、S4 装置都还没写进页 |
| 跑前登记 | `research/prompts/e162-preregistration.md`（1296 行） | 第一至十一节（S1–S3 原登记）；第十二节修订一至六（:426–:431）、补登 S3 跨机与 S4 掉电（:433 起，补 1–补 13）、补 14 主 agent 认定（:1038）、S4-修订一至十（:1053）、S3-修订一至十（:1070） |
| 问题单 | `research/prompts/m2-crash-store-r1-forks.md`（12 行） | S1–S4 在 :9–:12 |
| 报告 | `research/prompts/e162-s1-decisive-runner-report.md`、`e162-s3-s4-designer-report.md`、`e162-s4-apparatus-runner-report.md`（证据目录 `e162-s4-apparatus-evidence/`） | S3 回环执行员没有入库报告（`ls research/prompts \| grep -i -E 's3-loop\|e162'` 只列出这四样加登记） |
| 装置 | `research/e7-index-bench/src/bin/e162_crash_verdict_block_store.rs`（S1、S2）、`e162_verdict_store_sender.rs`、`e162_verdict_store_network.rs`（S3）、`e162_verdict_store_power_cut.rs`（S4） | `research/e7-index-bench/Cargo.toml:471`、`:495`、`:501`、`:507` 四个 `[[bin]]`；除 sender 外都 `required-features = ["e162-block-stores"]` |
| 版本 | `research/e7-index-bench/Cargo.toml:457–:463` `[dependencies.redb] version = "=4.3.0"`、`[dependencies.rocksdb] version = "=0.25.0"`，都 `optional = true`；`:466` `e162-block-stores = ["dep:redb", "dep:rocksdb"]` | `research/Cargo.lock:885–:886` redb 4.3.0、`:938–:939` rocksdb 0.25.0、`:544–:545` librocksdb-sys `0.19.0+11.8.1` |

构建注意（`research/e7-index-bench/Cargo.toml:453–:455` 注释）：`librocksdb-sys` 现编 C++，本机 bindgen 找不到 `stdbool.h`，要 `BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/gcc/x86_64-linux-gnu/13/include` 才编得过；为什么以前不设也编过没查（实验页 :187）。

### 1.2 四条臂

实验页 :7 整行：

> 四条被测臂：F（加固的文件，临时文件 + fsync + 改名 + 目录 fsync）、R1（redb，`set_quick_repair(true)`）、R0（redb，默认修复）、K（RocksDB，同步写 + WAL）。

完整定义在登记 :149（R1）、:150（R0）、:151（K）、:152（F）：R1 每块一个写事务、`Durability::Immediate`、`set_quick_repair(true)`；R0 同 R1 不开 quick-repair；K `put_opt` 带 `set_sync(true)`、WAL 开、Lz4 压缩，打不开时调一次 `DB::repair`；F 一块一个文件（48 字节头含块键与 CRC-32C）、临时文件 → `sync_all` → 改名 → 父目录 `sync_all`。块键 32 字节 =（节点 u32 大端，段号 u32 大端，序号区间起点 u64 大端，输入指纹 16 字节），块值 65 536 字节、每状态 1 字节判定编号（登记「1.1 读法写死」「块」那一行）。没建的形态：RocksDB BlobDB（登记 :154）。

登记第四节 4.10：F 在 S1 的 Q1a–Q1f 上「由它的定义加内核语义推得出为 0」，对 F 不算判据；F 真正的风险（掉电）SIGKILL 量不到。

### 1.3 各段现状

| 行 | 跑没跑 | 够没够判 | 出处 |
|---|---|---|---|
| S1 主格（每臂杀 200 次） | 跑了（可行性档 20 次、够判档 200 次，2026-09-27） | **够判：四臂都没翻** | 页 :105–:151；产物 `…-s1-{F,R1,R0,K}-decisive.out` |
| S1-large（10⁵ 块大库上杀 20 次） | 没跑 | — | 页 :180、:182 |
| S2 主格（10⁵ 块） | 只跑可行性档 10⁴ 块 | 未判；Q2a 标 `pending_e161` | 页 :50–:68、:180 |
| 几何敏感性 G-bs10、G-batch16、G-sparse、G-random | 没跑 | — | 页 :182；登记 8.2（:329 起） |
| S3 回环 | 只跑了 R1 的一部分（2026-09-28），执行员按用户定停下；按调度记录这半份作废 | 未判 | 产物 `research/results/e162-verdict-store-network-2026-09-28-s3-loopback-R1.out`（168 行，没有 `name=done`，没有 S3-T 的 `name=verdict`） |
| S3 跨机 | 没写装置、没跑（S3-修订九：network bin 只有回环角色） | — | 登记 :1070 起 S3-修订九那一行 |
| S4 掉电 | 装置、单测 23 个、变异 11 条、干跑写完；没起 QEMU | 未判 | `research/prompts/e162-s4-apparatus-runner-report.md` 第 4–8 行、第 70 行 |

**S1 够判档判决行**（`grep -n 'name=verdict' research/results/e162-crash-verdict-block-store-2026-09-27-s1-{F,R1,R0,K}-decisive.out`，整行）：

```
research/results/e162-crash-verdict-block-store-2026-09-27-s1-F-decisive.out:258:E7RESULT name=verdict part=s1 arm=F tier=registered judgement=judged_by_registration kill_cycles=200 kills_inside_commit=180 pc_l_ok=true pc_s_ok=true pc_p_ok=true pc_o_ok=true pc_t_ok=true commit_waits_for_device=true calibration_interfered=true v6_kills_inside_commit_ok=true planned_confirmations=6178 a_s1_ok=true q1a=0 q1b=0 q1c=0 q1d=0 q1e=0 q1f=0 repaired_openings=0 s1_positive_quantities=none
research/results/e162-crash-verdict-block-store-2026-09-27-s1-R1-decisive.out:235:E7RESULT name=verdict part=s1 arm=R1 tier=registered judgement=judged_by_registration kill_cycles=200 kills_inside_commit=179 pc_l_ok=true pc_s_ok=true pc_p_ok=true pc_o_ok=true pc_t_ok=not_applicable commit_waits_for_device=true calibration_interfered=false v6_kills_inside_commit_ok=true planned_confirmations=6178 a_s1_ok=true q1a=0 q1b=0 q1c=0 q1d=0 q1e=0 q1f=0 repaired_openings=0 s1_positive_quantities=none
research/results/e162-crash-verdict-block-store-2026-09-27-s1-R0-decisive.out:233:E7RESULT name=verdict part=s1 arm=R0 tier=registered judgement=judged_by_registration kill_cycles=200 kills_inside_commit=178 pc_l_ok=true pc_s_ok=true pc_p_ok=true pc_o_ok=true pc_t_ok=not_applicable commit_waits_for_device=true calibration_interfered=false v6_kills_inside_commit_ok=true planned_confirmations=6178 a_s1_ok=true q1a=0 q1b=0 q1c=0 q1d=0 q1e=0 q1f=0 repaired_openings=0 s1_positive_quantities=none
research/results/e162-crash-verdict-block-store-2026-09-27-s1-K-decisive.out:234:E7RESULT name=verdict part=s1 arm=K tier=registered judgement=judged_by_registration kill_cycles=200 kills_inside_commit=180 pc_l_ok=true pc_s_ok=true pc_p_ok=true pc_o_ok=true pc_t_ok=not_applicable commit_waits_for_device=true calibration_interfered=false v6_kills_inside_commit_ok=true planned_confirmations=6178 a_s1_ok=true q1a=0 q1b=0 q1c=0 q1d=0 q1e=0 q1f=0 repaired_openings=0 s1_positive_quantities=none
```

S1 的结论句，实验页 :151 原句（节选到句号为止的那一句整句）：「**S1 够判：redb（quick-repair 与默认修复两种形态）、RocksDB（同步写 + WAL）、加固的文件，在这次 SIGKILL 模型下四条臂都没有丢已确认的块、没有静默坏块、没有幽灵块、没有打不开**。」页上同一段接着写「这里写成『四个候选在这个 SIGKILL 模型下都没有翻』，不写成『都一样安全』」。

S1 重开用时（Q1h，轨迹，不判翻面）四臂的汇总行在页 :130–:133；中位 F 1100 µs、R1 1015 µs、R0 124 691 µs、K 11 411 µs，峰值 K 3 271 087 µs（页 :138 说是撞上同期别的会话 `cargo test` 的极端值）。

**S2 可行性档判决行**（10⁴ 块，页 :53–:56 整行）：

```
research/results/e162-crash-verdict-block-store-2026-09-27-s2-F-feasibility.out:41:E7RESULT name=verdict part=s2 arm=F tier=feasibility judgement=not_judged_feasibility_tier_only blocks=10000 pc_rate_ok=true pc_read_ok=true pc_space_ok=true s2_main_write_states_per_second=33530296.2 q2a_verdict=pending_e161 s1_large=not_run_feasibility_tier
research/results/e162-crash-verdict-block-store-2026-09-27-s2-R1-feasibility.out:41:E7RESULT name=verdict part=s2 arm=R1 tier=feasibility judgement=not_judged_feasibility_tier_only blocks=10000 pc_rate_ok=true pc_read_ok=true pc_space_ok=true s2_main_write_states_per_second=33593904.5 q2a_verdict=pending_e161 s1_large=not_run_feasibility_tier
research/results/e162-crash-verdict-block-store-2026-09-27-s2-R0-feasibility.out:41:E7RESULT name=verdict part=s2 arm=R0 tier=feasibility judgement=not_judged_feasibility_tier_only blocks=10000 pc_rate_ok=true pc_read_ok=true pc_space_ok=true s2_main_write_states_per_second=19339984.1 q2a_verdict=pending_e161 s1_large=not_run_feasibility_tier
research/results/e162-crash-verdict-block-store-2026-09-27-s2-K-feasibility.out:41:E7RESULT name=verdict part=s2 arm=K tier=feasibility judgement=not_judged_feasibility_tier_only blocks=10000 pc_rate_ok=true pc_read_ok=true pc_space_ok=true s2_main_write_states_per_second=48638773.4 q2a_verdict=pending_e161 s1_large=not_run_feasibility_tier
```

S2 可行性档的时延与占用（页 :61–:66 的表，页 :68 写明「单次裸读数，未重复跑、未判定，不当最终结论」）：

| 臂 | 持续写入（状态/秒） | 提交时延中位/p99/最大（µs） | 冷读中位/p99（µs） | 热读中位/p99（µs） | `peak_ratio_high` | `after_close_ratio` |
|---|---|---|---|---|---|---|
| F | 33 530 296.2 | 1529 / 5036 / 583066 | 321 / 399 | 123 / 144 | 1.0639 | 1.0625 |
| R1 | 33 593 904.5 | 1550 / 4352 / 115139 | 230 / 352 | 5 / 8 | 2.0029 | 2.0021 |
| R0 | 19 339 984.1 | 903 / 8299 / 2558476 | 229 / 366 | 5 / 11 | 2.0039 | 2.0017 |
| K | 48 638 773.4 | 804 / 1293 / 1733112 | 157 / 367 | 12 / 21 | 2.1495 | 1.0013 |

**S3 回环 R1 那半份**（`research/results/e162-verdict-store-network-2026-09-28-s3-loopback-R1.out`；已进暂存区 `git status --short` 报 `A`）：三个阳性对照的判决行整行：

```
30:E7RESULT name=verdict part=pc3_drop arm=R1 lost_confirmed=50 library_controlled_drops=50 positive_control_passed=true
55:E7RESULT name=verdict part=pc3_rate arm=R1 fast_blocks_per_second=579.634 slow_blocks_per_second=154.459 positive_control_passed=true timing_undisturbed=true
162:E7RESULT name=verdict part=pc3_torn arm=R1 interruptions=50 escalated_to_200=false silently_corrupted=28 positive_control_passed=true
```

S3-T 只写出了 sender / library / verification / rates 四行，没有判决行；rates 行开头（第 168 行，截到 `q3k_windows` 之前）：`E7RESULT name=rates arm=R1 cell=S3-T attempt=1 q3a_blocks_per_second=274.933 q3a_states_per_second=18017998.3 q3b_blocks_per_second=565.217 q3b_states_per_second=37042085.9 q3c_states_per_second=55060084.2`。**这半份按调度记录作废**（下面 1.5），数只当量级参考，不能引作结论。同批 anchors 产物 `research/results/e162-verdict-store-network-2026-09-28-anchors.out:21` 整行 `E7RESULT name=verdict part=anchors anchors_checked=20 anchor_mismatches=0`。

### 1.4 哪几格还等 E161（崩溃放量的去重与分段耗时） 的正式每状态耗时

| 格 | 门槛怎么用 t_state | 出处 |
|---|---|---|
| S2 Q2a 持续写入速率 | R_AB = (P_A + P_B) ÷ t_state；四臂判决行都是 `q2a_verdict=pending_e161` | 登记 6.0（:225 起）、:256；页 :80、:183 |
| S2 Q2b 分窗速率、Q2c 缓冲 | 都按 R_AB | 登记 6.2 表 Q2b、Q2c 两行；修订四（:429）让 Q2c 在 t_state 没到时按实测 Q2a 的 50/75/90/100% 报 |
| S2 Q2d 冷读 p99 | 门槛 = 每块状态数 × t_state（设计员加的，登记写明要主 agent 认或删） | 登记 6.2 表 Q2d 那一行 |
| 几何敏感性「门槛本身」 | t_state 与 t_state_1 各判一次 | 登记 8.2（:329 起）第二行 |
| S3 Q3a（回环与跨机） | R_B = P_B ÷ t_state | 登记 1.1「那一台 CPU 的判定产出速率（S3）」、8.2 末行 |

E161 今天只有可行性档：页 `.claude/kb/experiments/161-崩溃放量的去重与分段耗时.md:1` 标题「可行性档已跑（2026-09-27）」，:81「**G1–G5 每一行都未判**…第八节几何敏感性…G3 的加权估计与 G5 的外推行这一趟不跑」。E162 页 :183 引过 E161 一格粗数：`research/results/e161-crash-state-dedup-and-time-split-feasibility-2026-09-27.out:37` 的 `states=37 state_elapsed_ns=77898270`，约 2.11 ms/状态，页上写明「只当量级参考，不代入本页任何判定」。另一种不依赖 E161 的速率（门禁 54 号实跑的整流挂钟）在第五节第 4 行。

### 1.5 第三段为什么停

`records/2026-09-24-里程碑二收尾调度.md:318` 整行：

> | 每 65 分钟查一次巡视看门狗；E162 第三段现在停（2026-09-28） | 用户：「再设置一个 每1小时零五分的定时循环任务 用来检查 1小时的定时循环任务是否在进行」——主 agent 起后台等待 65 分钟、查本会话的看门狗进程在不在再退出（退出即叫醒），每次醒来核完再起下一轮；cron 排不出每 65 分钟，不用。E162 第三段执行员报：跑前登记要求计时格开跑前等本机没有 cargo / rustc / qemu（最多 20 分钟、受干扰重跑最多三次），层 0 全量一跑计时读数多半作废（机器空闲时四条臂约 3.5–4 小时，撞车时约 35–40 小时，都是推的）；弹窗三选一，用户答「现在停，层 0 跑完再从头重跑」——执行员停掉产物任务、交回；已写出的 anchors 与 R1 半份原样留着、写明作废；层 0 跑完另派执行员从头重跑四条臂，产物另起 `-r2` | 层 0 跑完 → E162 重跑 |

「写明作废」这句在仓里只见于这一行：产物文件里没有作废标记（`grep -c 'name=verdict part=s3'` 那份产物为 0、`name=done` 为 0），实验页也还没提 S3。

之前还有一处与 S2、S3、S4 排期有关的用户定案，`records/2026-09-24-里程碑二收尾调度.md:311` 那一行里：「② KV 与 GPU 弹窗答「先跑第五步，KV 接入放提交之后（推荐）」——全量跑期间并行做 E163 GPU 重跑与 E162 的 S3 跨机、S4 断电、几何敏感性，选型定了之后 KV 接入作为里程碑二提交后的剩余任务、接完再跑一趟全量」（整行在第三节）。

### 1.6 装置住哪、为什么不进 `crates/`

- 登记 1.3（:42）：「装置走 `research/e7-index-bench/src/bin/e162_crash_verdict_block_store.rs`（独立手写模型，不与 `crates/` 共用代码，`.claude/rules/implementation-first.md` 第 4 条）」；:44「**不碰 `crates/` 下任何文件，不给 `crates/` 加依赖。**」
- `research/e7-index-bench` 是独立 Cargo 工作区：根 `Cargo.toml:2`「research/ 是实验 workspace，自己有 Cargo.toml，这里显式排除。」、`:12` `exclude = ["research"]`；`research/Cargo.toml:1–:3` `[workspace]`、`members = ["e7-index-bench"]`；锁文件各一份（`Cargo.lock` 与 `research/Cargo.lock`）。
- 为什么放那里：`records/2026-09-24-里程碑二收尾调度.md:325` 末句「实验放 `research/e7-index-bench`（独立的 Cargo 工作区），不给 `crates/` 加依赖——加依赖会改 `Cargo.lock`，而 `.claude/gate.d/stage-inputs.tsv` 里每条崩溃枚举用例都登记了它，全部复用标记会失效；接进 harness 等选型定了、随里程碑二提交一起跑全量。」（整行在第三节）
- 登记第三节（:90 起）引的 `crates/singlefs-harness/src/crash.rs`、`crates/singlefs-harness/src/layer0_progress.rs` 路径已过时：两档拆分（提交 `973e1f58`）之后这两份在 `crates/singlefs-checker-tier/src/`。登记是冻结证据，不改；引它时换成新路径（第二节的行号都是新路径现取的）。

## 二、今天崩溃枚举落不落盘

### 2.1 每个状态算完留下什么

| 面 | 在哪 | 事实 |
|---|---|---|
| 一个状态怎么评 | `crates/singlefs-checker-tier/src/crash.rs:1131` `evaluate_state_recording_findings`（`:1059` 的 `evaluate_state_for_versions` 转给它，`position = None`） | 用 `persisted: Vec<bool>` 造 `CrashImage`，跑两遍恢复（`JournalPolicy::Consult` / `Ignore`，:1147–:1148），再判 oracle、池级 checker、记录核对器；结果只做 `tally.<计数> += 1` 与记第一处违例原文 |
| 累计进什么 | `crash.rs:743` `pub struct Layer0Tally` | 字段：`states`、`states_by_publish`、`violations`、`root_persisted_states`、`no_file_states`、`file_read_states`、`failed_states`、`journal_differing_states`、`verification_ran_states`、`verification_failed_states`、`first_violation`、`ignored_violations`、`first_ignored_violation`、`record_root_without_record`、`record_claimed_state_missing_unit`、`checker_evaluated_states` / `checker_violated_states` / `checker_first_violation` / `checker_not_applicable_states`（按不变量的 BTreeMap）、`observed_states`、`observer_counts`、`findings`（:745–:779） |
| 判红的状态留什么 | `crash.rs:590` `Layer0Findings`、`:549` `Layer0FindingSignature`、`:567` `Layer0Finding`、`:557` `LAYER0_FINDING_SAMPLES_KEPT: usize = 3` | 签名 =（判红的那一遍 `Layer0RedPass`（:473，四种：看 journal 的 oracle / 不看 journal 的 oracle / 池级 checker 带违了哪几条 / 记录核对器），段号或 `all_persisted`，发布类）；每签名记状态数与最先 3 个样本（状态序号 + 违例原文）；另记 `red_states` |
| 判绿的状态留什么 | 同上 | 什么都不留，只进计数。没有「每个状态的判定」这一份东西，全仓没有按状态存的判定（`grep -rln '判定向量\|判定编号'` 只命中里程碑三 03 号文件、E162 登记与问题单、E162 装置、一份无关的 runlog） |
| 状态怎么编号 | `crash.rs:1816` `Layer0StatePlan`（文档注释 :1813–:1815）、`:1917` `persisted_writes_of_state` | 一条流一个**全局序号**：各段的状态区间首尾相接、随段号递增，最后加「全部持久」那一个；段内序号 = 序号 − 该段起点，按混合进制拆到段里每次写（原地覆写三态、其余两态）。持久集合由序号现算，不存 |

### 2.2 断点续跑的进度文件（`crates/singlefs-checker-tier/src/layer0_progress.rs`）

| 项 | 事实 | 出处 |
|---|---|---|
| 开不开 | 设了环境变量 `SINGLEFS_LAYER0_PROGRESS_DIRECTORY` 才留；设了就要同时设 `SINGLEFS_LAYER0_INPUT_FINGERPRINT` | :44–:47 |
| 文件名 | `layer0-progress-<流名>-<输入指纹>-<计划哈希>[-shard-<i>-of-<n>].txt` | :464–:477 |
| 文件头 | `layer0_progress_file format=2 input_fingerprint=… stream=… plan=… states=… slices=… states_per_slice=… has_observer=…[ shard=i/n]`，行尾 ` checksum=<CRC-32C 8 位十六进制>` | :58、:480–:500 |
| 单位 | **片**：一行一片，片行 25 个字段（`slice`、`first`、`end`、`states` … `findings`），就是那一片的 `Layer0Tally` 全部计数，不按状态 | :502–:528、:567 `slice_line` |
| 片多大 | 续跑、分片、merge 时片长只看状态数：`ceil(状态数 / 65 536)`，至少 16（`LAYER0_RESUMABLE_SLICE_COUNT = 65_536`、`LAYER0_MINIMUM_STATES_PER_SLICE = 16`）；不续跑时按线程数切（每线程 16 片、至少 64 片） | `crash.rs:2029`、`:2032–:2039`、`:1293–:1297`、`:2003` |
| 怎么写 | 打开时把核过的片重写成新文件：建 `.rewriting` → `write_all` → `sync_all` → `rename` → 目录 `sync_all`；之后每跑完一片追加一行、`sync_data` | :1139–:1159、:1186–:1197 |
| 读回 | 每行自带 CRC-32C；末尾没换行的半行丢掉；整行坏、字段缺多、解不开，整份作废；同一片出现两次计数相同去重、不同整份作废 | 模块文档 :1–:17 |
| 跑完 | 默认删掉（`Layer0ProgressFileAfterCompletion::Deleted`：「进度文件只在『没跑完』时存在」） | :102–:109、:1204–:1215 |
| 54 号放哪 | `<git common-dir>/singlefs-layer0-progress/<这条用例的输入指纹>` | `.claude/gate.d/54-layer0-replay.sh` 文件头「断点续跑」那几行 |

具体的数（按片长公式现算，算术）：并行线一那条流 12 230 590 578 个状态（`.claude/gate.d/stage-inputs.tsv:40` 注释）→ 每片 186 625 个状态；`crash.rs:2028` 注释「第二条流全量五十多亿个状态时每片约八万五千个（32 线程每秒约两万多个时一片一分钟上下，推的）」。

### 2.3 发现日志（判红那一份，用户 2026-09-27 定「全量和错误双份」）

- 环境变量 `SINGLEFS_LAYER0_FINDINGS_FILE`（`layer0_progress.rs:55`）；格式号 `FINDINGS_LOG_FORMAT = 1`（:1682）。
- 每趟枚举追加一节：`layer0_findings_begin`（format、stream、states，分片时加 shard、shard_states，:1923–:1940）；跑的过程中新签名一行、跨 10/100/1000… 台阶一行，每行 `sync_data`（:1795–:1800）；跑完把这一节截回起点重写成定稿：每签名一行 `layer0_finding`（号、签名字段、states、sample_states）加一行 `layer0_findings_summary`（:2005–:2030、:1808–:1817）。没有 summary 行 = 没跑完。
- 54 号读定稿：没 summary 判红、`red_states > 0` 判红、定稿行数与 `signatures=` 对不上判红；打 ≤ 30 行发现表（54 号文件头「发现日志」那一段）。

### 2.4 双机分片与合并（`research/scripts/layer0-shard-run.sh`，511 行）

| 步 | 事实 | 出处 |
|---|---|---|
| 怎么分 | 第 i 片跑切片序号 `slice_index % n == i` 的那些片（交错分） | `layer0_progress.rs:184–:189` `shard_owning_slice`；:121–:122 |
| 各自写什么 | 每片一份**账本** `layer0-shard-<流名>-shard-<i>-of-<n>.tally`：文件头（format=2、输入指纹、流名、计划哈希、状态数、片数、片长、shards、rustc、cargo、target）、这一片的线程行、按切片序号的片行（与进度文件同一种）、末行；临时文件 `.writing` → `sync_all` → 改名 → 目录 `sync_all` | :1241–:1256、:1274–:1300、:1352、:1402–:1421 |
| 进度 | 两片各自有进度文件（文件名带 `shard-<i>-of-<n>`），本机在 `<git common-dir>/singlefs-layer0-progress/<指纹>`，第二台在 `<PEER_REPOSITORY_DIRECTORY>/progress/<指纹>`；驱动收尾不删进度目录 | 驱动 :14–:30 |
| 合并 | ⑤ 第二台的账本 rsync 回本机进度目录（只拷 `layer0-shard-*-shard-1-of-2.tally`）；⑥ 本机 `SINGLEFS_LAYER0_SHARD=merge/2` 再跑同一条用例：不枚举，读 n 份账本、核文件头逐字段相同、核切片 0..S 每个恰好一次，按切片序号交回计数，再走用例钉死的计数断言 | 驱动 :463–:471、:473–:482；`layer0_progress.rs:1427–:1436` `Layer0MergedShardLedgers`、:1600–:1612 `read_shard_ledgers_for_merge` |
| 发现日志 | 三趟各一份（`<日志>.findings.tsv`）：第二台那一片的拷回本机；merge 那一趟重写出整流的发现表，54 号只读 merge 那一份 | 驱动 :48–:52、:352–:356、:448–:453、:476–:486 |
| 这种形态叫什么 | 「两台各跑一片、各写一份账本，本机 merge：正是用户 2026-09-27 否掉的『各写各的再合并』那一形」 | E162 登记第三节（:90 起）「两台机器今天怎么分工」那一行 |

### 2.5 门禁 54 号的复用判定

| 项 | 事实 | 出处 |
|---|---|---|
| 单位 | **一条崩溃枚举用例 × 这条用例这批输入的指纹**；一条用例整条流一个标记，不按片、不按段 | `.claude/gate.d/54-layer0-replay.sh` 文件头 `--full` 那一段；`research/scripts/admission.py:1757–:1762` |
| 指纹怎么算 | `admission.py crash-case-manifest`：登记路径（`crates/ Cargo.toml Cargo.lock`）下的文件减去这条用例读不到的（别的测试目标独占的测试文件、`crates/mutations.tsv`、没有代码读 `CARGO_BIN_EXE_` 时的 `src/bin/`），加准入模块里崩溃枚举用例的判法摘要、工具链、构建环境、这条用例的登记行；登记了 `shard=across-machines` 的再按内容加驱动与配置判法两份（`SHARD_DRIVER_FILES`，:220） | `admission.py` 文件头 :82–:85；`.claude/gate.d/stage-inputs.tsv:14–:20` 注释 |
| 标记写在哪 | git common-dir 里 `singlefs-crash-case-green.<用例名>.<输入指纹>`，不进工作树，各 worktree 共用 | `admission.py:225`、:1757–:1762 |
| 标记里有什么、何时作数 | `input_hash` 等于这一次指纹、`case` 对、`test_result` 是「1 passed; 0 failed」、登记的每个计数行恰好一行、要 `exhaustive=true` 的带着；没有发现表 | `admission.py:1931–:1947` `crash_case_marker_problems`；54 号文件头「全绿标记不带发现表」 |
| 判红 | 删这批输入那一格；「跑的过程中输入变了」判红不删 | 54 号文件头 |
| 门禁阶段的标记（55、57、59 号） | `singlefs-stage-green.<阶段文件名>.<指纹>`，过 `SINGLEFS_REUSE_HOURS`（默认 24 小时）不作数；`stage-must-run.sh` 先看它、再比 `refs/sop/staged-green` 那棵树与这一次暂存树在登记路径上变没变 | `research/scripts/stage-must-run.sh:16–:19`；`admission.py:2222–:2250` |

小结（从上面几张表读出来的，不是新的量）：今天落盘的三样——进度文件（按片的计数，跑完就删）、分片账本（按片的计数）、发现日志（只有判红的签名与每签名 3 个样本）——都不带「每个状态的判定」；门禁 54 号的复用粒度是整条流。

## 三、用户对日志与存储提过的要求（原文整行）

`records/2026-09-24-里程碑二收尾调度.md:245`（层 0 放量的发现日志）：

> | 层 0 放量的发现日志（用户 2026-09-27） | 用户原话「我觉得还有个问题 我们到时候的 崩溃放量日志 只写出错日志或者需要全量和错误双份日志 不然你读不过来了」。主 agent 现查今天的样子：`Layer0Tally`（`crash.rs:1171`）只记各类计数与每类一条 `first_violation`；54 号 `run_crash_case`（第 225–236 行）整段 cargo 输出 `tee` 进一份日志、只转 `LAYER0_PROGRESS` 行；红了只在末尾看到一条，分不出红在哪几段哪几类，中途读不到。定：**两份日志**——全量日志照旧，另加发现日志（按签名去重：不变量集合 + 段号 + 发布类 + 判红的那一遍，每签名记状态数与最先 3 个状态序号，新签名或跨 1/10/100… 台阶时追加并 fsync，跑完写汇总行；标准输出每个新签名打一行 `LAYER0_FINDING`）；断点续跑与分片 merge 合发现表；54 号读汇总与发现日志、把发现表（≤ 30 行）打进阶段输出，全量日志只给路径；双机驱动 merge 两片的发现日志。派实现员改 crates 那一半（规格 `/tmp/claude-1000/impl-layer0-findings/spec.md`，先交环境变量名与行格式），工具实现员改 54 号与驱动那一半等它交第一节后派；三方进代码轮第二轮与门禁批第三轮。 |

（这一行里的 `crash.rs:1171` 是当时 `crates/singlefs-harness/src/crash.rs` 的行号；今天 `Layer0Tally` 在 `crates/singlefs-checker-tier/src/crash.rs:743`。落地形态见第二节 2.3，交回记录在同一份调度记录 :250、:255。）

`records/2026-09-24-里程碑二收尾调度.md:325`（崩溃放量的判定存储进里程碑二）：

> | 崩溃放量的判定存储进里程碑二（2026-09-27，用户定） | 用户定：「这种规模文件的读写和持久化放在文件中 不靠谱。需要解决」「KV 存储这个功能也放入里程碑 2 做」。主 agent 定形态（推的）：嵌入式事务 KV，按块存（一块 2¹⁶ 个状态），不用向量数据库（查找全是精确匹配）；选哪个库先做计数实验（问题单 `research/prompts/m2-crash-store-r1-forks.md`：redb / RocksDB / 加固的文件三臂，杀进程重开核丢块、持续写入速率、双机合并）。实验放 `research/e7-index-bench`（独立的 Cargo 工作区），不给 `crates/` 加依赖——加依赖会改 `Cargo.lock`，而 `.claude/gate.d/stage-inputs.tsv` 里每条崩溃枚举用例都登记了它，全部复用标记会失效；接进 harness 等选型定了、随里程碑二提交一起跑全量。 |

`records/2026-09-24-里程碑二收尾调度.md:311`（分片可用性真测一次；KV 接入放提交之后）：

> | 分片可用性真测一次、红在第二台算不出指纹；用户定 KV 接入放提交之后、两台清场扩大、提交时测试次序（2026-09-28） | 用户：「没有越权 分片你要测试一下 测试可用性 然后告诉我大约预计的时间」。驱动自检 13 格绿（假 cargo、第二台是本机目录）；主 agent 用第三轮快照那棵树带 `SINGLEFS_HEAVY_TESTS=user-request` 真跑 `crash-case:layer0-first-stream`：两台工具链相同、各 32 核，树拷过去之后第二台算不出输入指纹（规范副本被 `.gitignore` 第 6 行忽略、不随树走；本机与自检都能回退到主仓，所以自检没抓到），失败在清场之前，本机本地模型服务一直 active；派 tooling-writer 修（`records/2026-09-16-subagent拆分提案.md` 第 54 行）。全量时间先报推的：三条大流合计 30.4 亿状态，按 2026-09-25 量的 13,880 个/秒（32 线程、第二条流，C577 与 49 条不变量之前）单机约 61 小时、双机约 30 小时，修好后真跑量两台速率再换成量的数。用户接着定：①「32核 至少可以跑31核」——本机那一片 `SINGLEFS_LAYER0_THREADS=31`；② KV 与 GPU 弹窗答「先跑第五步，KV 接入放提交之后（推荐）」——全量跑期间并行做 E163 GPU 重跑与 E162 的 S3 跨机、S4 断电、几何敏感性，选型定了之后 KV 接入作为里程碑二提交后的剩余任务、接完再跑一趟全量；③「两台都停 包括tts」「配置写在本地 不要公开」「要明确指定cot服务 防止错误清理systemctl 导致系统问题」——主 agent 改私有配置（仓根 `multi-host.env` 与它指向的三份私有清场脚本，单元逐个写死、两张白名单、带通配字符就拒，只停开跑前 active 的、复原只起开跑前 active 的；清场单元里原来经别的仓的脚本间接停的那几个也改成直接写死），仓里不写单元名与主机；不是服务单元的 java / node 开发服务器开跑时按进程号逐个停、起法记私有文件交用户；④「提交时候 先跑 harness 然后跑 checker 最后跑checker-tier」——第五步派崩溃验证员照这个次序。另：singlefs-8b 提交 `e5253e8a` 之后主 agent 把第一段打进主工作区（`crates/mutations.tsv` 1400 行、33 号绿，`crates/` `litmus/` 与第三轮快照逐字相同，差的只是第一段的变异表行）；singlefs-8b 转来 55 号判别力样本缺 raise-rollback-floor 档（stage-selftest 判错 2 个），用户定下次提交崩溃验证员跑 55 号带 `VM_KEEP=1` 补样本，并进第五步派发提示。第十步素材 ㊳：用户「你遇到的这些问题 稍后都要按照脚本的方式处理 修bug」——看门狗把驱动自检里一趟趟起的小分片认成没带前缀的重型测试（按 pid 确认压不住）；交回后留在后台的门禁进程没人停；派发闸要「重型测试：不跑」单独一行；内存准入排队时主 agent 看不出要等多久 | 驱动修好 → 真跑第一条流量速率 → 报用户；第五步照 ④ 的次序 |

里程碑三第十一项的原话，`.claude/kb/milestone/03-third-txn.md:93` 整行：

> - **原话**：「既然说到150亿文件的读写，这里就要给里程碑3 增加一个任务任务 快速的读取和写入这150亿文件。我觉得我们需要使用数据库，KV 存储似乎是一个比较合适的方法。」「这种规模文件的读写和持久化放在文件中 不靠谱。需要解决」（用户 2026-09-27）

### 3.1 第十一项「主 agent 的理解」（`03-third-txn.md:95`，整段标「待用户核，推的，没量过」）逐句对今天的仓

| # | 原句（逐字） | 推的还是有数 | 今天仓里对得上的事实 |
|---|---|---|---|
| a | 「要读写的不是 150 亿个文件，是 150 亿个崩溃状态」 | 推的（读法） | 状态总量仓里有三个互不相同的数：150 亿（原话）、问题单 :5 的「约 1.9 × 10¹⁰」、调度记录 :311 的「三条大流合计 30.4 亿状态」（推的）；另有一条流单独就是 12 230 590 578（`stage-inputs.tsv:40` 注释）。几个数的口径（哪几条流、快档还是全量）没有一处对齐过 |
| b | 「一个状态由（节点，段号，段内序号）唯一定出，镜像与持久集合都能从序号现算」 | 一半有代码 | 今天是（流，全局序号）→ 持久集合现算（`crash.rs:1917` `persisted_writes_of_state`），镜像由 `CrashImage{base, writes, persisted}` 现造（:1142–:1146）；「节点」今天不存在（第一项的节点树没实现） |
| c | 「真要落盘的只有每个状态的判定与去重用的键」 | 推的 | 今天什么判定都不按状态落盘（第二节）；E161 可行性档报 K4-walk、K4-units 在小域上不一致（E161 页 :82），去重键的形态没定 |
| d | 「裸文件在这个规模上缺五样：一批写一半的原子性、坏字节的校验、多机多卡写入的事务、数据与「哪个输入算出来的」元数据的绑定、按违例查询」 | 推的 | E162 S1 在 SIGKILL 下 F 臂没翻，但 F 在 Q1a–Q1f 上没有判别力（登记 4.10）；掉电（S4）没跑；多机写（S3）没有有效产物 |
| e | 「嵌入式事务 KV 存储（纯 Rust 的 redb 或 RocksDB，只进 harness）」 | 推的；「只进 harness」与今天的包划分对不上 | RocksDB 不是纯 Rust（`librocksdb-sys` 现编 C++，`research/e7-index-bench/Cargo.toml:453–:455`）；崩溃枚举引擎今天在 checker 档包（D13（验证路线） 已定项 15），见第四节 |
| f | 「值 = 区间里每个状态的判定指纹编号（不同的判定向量只有几十种，另存一张表）」 | 推的，没量过 | 仓里没有任何数量过「判定向量有几种」（第二节 2.1 那条 grep）；E162 只按 1 字节编号造合成数据（登记 4.1「本实验照问题单用 1 字节，不量种数」） |
| g | 「一块取 2¹⁶ 个状态，150 亿状态约 23 万个键、15–30 GB 值」 | 算术 + 推的 | 1.5 × 10¹⁰ ÷ 2¹⁶ = 228 881.8（登记 4.1、锚点 A-C2）；每状态 1 字节时 15 GB，「30 GB」要每状态 2 字节，出处没写（推的）；按 1.9 × 10¹⁰ 是 289 917 块、17.70 GiB（登记 4.7） |
| h | 「一块正好是 GPU 的一个批、双机分片的一个单位、门禁 54 号复用判定的一个单位」 | 推的；与今天三处的单位都不同 | 今天双机分片的单位是**片**，片长 ceil(状态数 ÷ 65 536)（并行线一那条流 186 625 个状态/片，不是 2¹⁶）；54 号复用单位是整条用例 × 输入指纹（第二节 2.5）；GPU 批今天由 E163 装置自己定（本调查没查） |
| i | 「违例另存一张小表」 | 已有类似物 | 发现日志（第二节 2.3）：按签名去重、每签名 3 个样本，已落地 |
| j | 「向量数据库答的是近似最近邻，这里全是精确匹配，近似命中会把两个状态当成一个，不适用。」 | 推的（读法） | 仓里没有向量库依赖（`grep -rn 'redb\|rocksdb\|sqlite\|lmdb\|sled\|heed' --include=Cargo.toml crates Cargo.toml \| wc -l` → `0`） |

同一段「仓里已有的」（:94）里「仓里没有任何数据库依赖（2026-09-27 现查各 `Cargo.toml`，sqlite / rocksdb / lmdb / 向量库都 0 命中）」今天只在 `crates/` 与根工作区成立：研究工作区已经有 redb 与 rocksdb（`research/e7-index-bench/Cargo.toml:457–:463`，E162 加的）。根 `Cargo.lock` 5 个包、`grep -c 'source = "registry' Cargo.lock` → `0`，即 `crates/` 今天一个外部依赖都没有。

## 四、接入约束

### 4.1 KV 库住哪个包，门禁 94 号怎么判

门禁 94 号（`.claude/gate.d/94-checker-implementation-disjoint.sh`）四条判据在文件头 :9–:23：① `singlefs-checker` 与 `singlefs-core` 各取传递闭包求交、减去 `singlefs-format` 必须为空，「外部 crate 在图上是叶子，仍然进闭包、仍然参与求交」（:12–:13），外部 crate 之间的传递依赖罩不到（:13–:14）；② checker 源码零处引 `singlefs_core`；③ 常量模块不许有分支循环；④ `singlefs-harness` 的闭包（含 dev-dependencies）里没有 `singlefs-checker-tier`。今天各包依赖（各 `crates/*/Cargo.toml` 现读）：format 无依赖；core → format；checker → format；harness → checker、core、format；checker-tier → checker、core、format、harness。

| 放进哪个包 | 94 号红不红（按判据推的，没跑 94 号） | 别的条款 |
|---|---|---|
| `singlefs-checker`（池级 checker） | 只要 core 的闭包里没有同一个外部 crate，① 不红 | D13（验证路线） 已定项 15 射程「不管池级 checker 与实现共享什么（已定项 5：池级 checker 仍只依赖 `singlefs-format`）」（`.claude/kb/decisions/13-验证路线.md:320`）；`.claude/rules/verification.md:45`「它今天只依赖 `singlefs-format`、没有 `[dev-dependencies]`，这一句 94 号不单判」。门禁不拦，条款不许 |
| `singlefs-core` | 若 checker 也用同一个 KV crate，① 红 | 实现层不该带验证存储（推的）；不是候选 |
| `singlefs-harness` | ①④ 都不红（harness 不在 checker、core 的闭包里） | 里程碑三 :95 与调度记录 :325 写的是「只进 harness」「接进 harness」；但崩溃枚举引擎、进度文件、分片账本今天都在 checker 档包（`crates/singlefs-checker-tier/src/crash.rs`、`layer0_progress.rs`），harness 档「改了代码随时跑」（`.claude/rules/verification.md` 定义表），RocksDB 进 harness 会让每次 harness 构建都现编 C++（推的） |
| `singlefs-checker-tier` | ①④ 都不红（它不在 checker、core、harness 任何一个的闭包里） | 消费者（`Layer0Tally`、片、账本、发现日志）都在这个包；D13（验证路线） 已定项 15 定案把「断点续跑与双机分片」列在 checker 档（`13-验证路线.md:318`） |

「只进 harness」这句写于两档拆分（提交 `973e1f58`）之前的认识；拆分之后它与包划分的对应要重判（推的，没三方）。

### 4.2 加依赖动 `Cargo.lock`：哪些登记行会失效

`.claude/gate.d/stage-inputs.tsv`（43 行，16 条登记）里路径列含根 `Cargo.lock` 的 13 行（awk 按制表符切第二列逐词比 `Cargo.lock`）：

| 行 | 键 |
|---|---|
| :28 | `54-layer0-replay.sh` |
| :29 | `55-qemu-device-streams.sh` |
| :31 | `59-crates-mutation-replay.sh` |
| :32 | `74-model-differential.sh` |
| :33 | `87-replay.sh` |
| :34 | `E142`（另含 `research/Cargo.lock` 与 `research/e7-index-bench/Cargo.toml`） |
| :36–:43 | 八条崩溃枚举用例：`crash-case:layer0-first-stream`、`layer0-second-stream`、`floor-raise-pushed-by-the-session`、`c561-sigma-full`、`layer0-multi-record-publish-stream`、`layer0-tree-split-streams`、`layer0-position-addressed-tree-streams`、`crash-injection-fast-tier` |

这八条崩溃枚举用例的路径列都是 `crates/ Cargo.toml Cargo.lock`，所以改任何 `crates/*/Cargo.toml` 同样让它们全部失效；准入模块只减去「用例读不到的文件」（别的测试目标独占的测试文件、`crates/mutations.tsv`、`src/bin/`），不减清单与锁（`admission.py` 文件头 :82–:85、`stage-inputs.tsv:14–:18` 注释）。在研究工作区加依赖只动 `research/Cargo.lock`，登记它的只有 `E142` 那一行；`87-replay.sh` 那一行登记了 `research/e7-index-bench/` 整个目录。

构建环境进不进指纹：`admission.py` 的构建环境只收 `RUSTFLAGS`、`CARGO_*`、`RUSTC*` 这几类（文件头「输入指纹里的构建环境」一段）；`grep -c BINDGEN research/scripts/admission.py` → `0`。RocksDB 的 `BINDGEN_EXTRA_CLANG_ARGS` 这一类不进指纹；双机驱动核两台的 rustc 与 cargo 版本，`grep -c 'clang\|g++\|BINDGEN' research/scripts/layer0-shard-run.sh` → `0`，不核 C++ 工具链与 libclang（事实；它会不会造成两台构建不同是推的）。

### 4.3 「独立工作区」写在哪

| 处 | 原文 |
|---|---|
| 根 `Cargo.toml:2`、`:12` | 「research/ 是实验 workspace，自己有 Cargo.toml，这里显式排除。」、`exclude = ["research"]` |
| `research/Cargo.toml:1–:3` | `[workspace]`、`resolver = "2"`、`members = ["e7-index-bench"]` |
| `research/e7-index-bench/Cargo.toml:452–:456` | E162 两个存储候选的注释：可选依赖、挂在不默认打开的特性上，理由是 `librocksdb-sys` 现编 C++ 与 bindgen 找不到 `stdbool.h` |
| 登记 1.3（:40–:46） | 「不碰 `crates/` 下任何文件，不给 `crates/` 加依赖。」 |
| 调度记录 :325 | 「实验放 `research/e7-index-bench`（独立的 Cargo 工作区），不给 `crates/` 加依赖——加依赖会改 `Cargo.lock`……」（整行在第三节） |

## 五、设计上还开着的问题（按 `03-third-txn.md:96`「开工前要先定的」逐条）

`03-third-txn.md:96` 整行：

> - **开工前要先定的**：选型挪到里程碑二先做（用户 2026-09-27 定；问题单 `research/prompts/m2-crash-store-r1-forks.md`）；里程碑三接着定块多大、块键里带哪些指纹；两台机器各写各的库还是一个库，怎么合并；违例表与判定向量表的形态；与第一项、第四项一起定。

| # | 问题 | 今天仓里有没有能回答它的数 | 出处 |
|---|---|---|---|
| 1 | 选型：redb（R1 / R0）、RocksDB（K）、加固的文件（F） | **S1 有，够判**：四臂 200 次 SIGKILL 全 0（对 F 无判别力）。**S2 只有可行性档**（10⁴ 块，写入 19 339 984.1–48 638 773.4 状态/秒，冷读 p99 352–399 µs，占用峰值 1.0639–2.1495），Q2a 等 E161。**S3 没有有效数**（R1 半份作废）。**S4 没有**（装置写完没起虚机）。**几何敏感性、S1-large 没有** | 第一节 1.3–1.5 |
| 1a | S2 门槛要的「两机 CPU 合计判定产出速率」 | 登记的正式口径（E161 t_state × 两机核数）没有数。另有一种口径的实测：门禁 54 号第二条流「本机 23,700 个/秒、第二台 24,800 个/秒」（调度记录 :317，写「量过」）；登记 4.9 写明不用这个口径。拿它与 S2 可行性档的最低写入速率比，19 339 984.1 ÷ (23 700 + 24 800) ≈ 399 倍（只是算术，两个口径不同、S2 未判，不是判定） | 调度记录 :317；登记 :120（4.9） |
| 2 | 块多大 | 没有量过块大小的取舍。只有：E162 主取样点 2¹⁶、反向点 G-bs10（2¹⁰）没跑；两条层 0 流段长（登记 4.12 引 E161 登记 :545、:550）：写数少的段只出一个不满的块，只有 18、26 写的段满得了 2¹⁶；今天片长 ceil(状态数 ÷ 65 536) 与块大小无关（并行线一那条流 186 625 个状态/片） | 第一节 1.3；登记 :123（4.12）；第二节 2.2 |
| 3 | 块键里带哪些指纹 | 没有数。今天已有的指纹：54 号每条用例的输入指纹（sha256，`admission.py crash-case-manifest`）、进度文件名里的计划哈希、分片账本文件头的工具链（rustc、cargo、target）；E162 块键只留 16 字节指纹。相关约束：记录核对器的判定依赖持久集合，按镜像去重对它不适用（`13-验证路线.md:135`）；E161 可行性档 K4-walk、K4-units 在小域上不一致（E161 页 :82），按走到的单元复用判定出错 | 第二节 2.2、2.5 |
| 4 | 两台一个库还是各写各的、怎么合并 | 用户已定「两机公用」（问题单 :11 S3 行、登记 :9–:11）。今天的双机分片正是「各写一份账本、本机 merge」那一形（第二节 2.4）。一个库时另一台送块的吞吐：没有有效数（S3 回环 R1 半份作废，网络那一路 `q3a_states_per_second=18017998.3` 只当量级）；跨机格没写装置。要求的量级：第二台 24 800 状态/秒（调度记录 :317），按每状态 1 字节约 24.8 KB/秒（算术） | 第一节 1.3；调度记录 :317 |
| 5 | 违例表的形态 | 已有可用的现成形态：发现日志（按签名去重、每签名 3 个样本、定稿与切法无关），54 号已在读 | 第二节 2.3 |
| 6 | 判定向量表的形态 | 没有。仓里没有量过判定向量有几种；「几十种」是推的（`03-third-txn.md:95`）。今天一个状态的「判定」散在 `Layer0Tally` 的几类计数里（恢复结局三选一、两遍恢复是否不同、验证跑没跑与失败、根槽是否持久、oracle 与不看 journal 的 oracle、记录核对器两条、每条不变量评估 / 违例 / 不适用），哪几样进向量没定（推的映射） | `crash.rs:743–:779` |
| 7 | 与第一项（节点树）一起定 | 没有：节点、按节点共享前缀都没实现；状态今天按（流，全局序号）编号 | `crash.rs:1813–:1816` |
| 8 | 与第四项（GPU）一起定 | E163（GPU多卡算单元校验和） 可行性：单机两卡 2560/2560 与 CPU 相同，双机五卡里一张起不来（`.claude/kb/experiments.md:186`）；显存预算「10¹⁰ 个状态时单卡每状态 3.419 字节、五卡每状态 10.26 字节」（`research/prompts/e161-preregistration.md:174`）。块怎么对上 GPU 批没有数 | 同左 |
| 9 | 状态总量口径 | 150 亿、1.9 × 10¹⁰、30.4 亿（推的）、单条流 12 230 590 578 四个数并存，没有一处说清各指哪几条流、快档还是全量 | 第三节 3.1 a 行 |
| 10 | KV 库住哪个包 | 没有定；按判据推只有 checker 档包与 harness 过得了 94 号，池级 checker 被 D13（验证路线） 已定项 15 射程挡着（第四节 4.1） | 第四节 |
| 11 | 掉电之后块还在不在 | 没有（S4 没跑）；装置自己报的风险：验盘把设备侧日志塞进 initramfs 可能超 QEMU initrd 上限（推的，没量过） | `research/prompts/e162-s4-apparatus-runner-report.md:34` |
| 12 | 空间 | 可行性档占用比 × 1.9 × 10¹⁰ 字节都落在当时 `df -B1` 可用字节内（约 1.98 × 10¹² 字节）；1.9 × 10¹⁰ 字节块值 = 17.70 GiB（登记 4.7） | 实验页 :68；登记 :118（4.7） |

## 六、没做什么

- 没编译、没跑任何测试、实验、门禁（派发「重型测试：不跑」；另一会话的层 0 全量占着两台机器）；第四节 4.1 的「94 号红不红」是按它文件头的判据推的，没跑 94 号。
- 没读 E163 的实验页与装置（第五节第 8 行只引了索引行）；没查 GPU 批今天怎么切。
- 没读 E161 跑前登记 :545、:550 原文，段长数只经 E162 登记 4.12 转引。
- 没核调度记录 :311、:317 里速率与状态总量是怎么量出来的（照原文标「量过」或「推的」）。
- 发现两处与本任务无关、没改：`crates/singlefs-checker-tier/src/layer0_progress.rs:14` 与驱动文件头仍把双机分片叫「里程碑三第六项」，而 `03-third-txn.md` 2026-09-27 已重新编号（第六项今天是「多次 COW 下的读写一致」）；E162 登记第三节的 `crates/singlefs-harness/src/…` 路径已过时（冻结证据，不改）。
- 草稿目录 `/tmp/claude-1000/m3-facts-kv/` 里只有从仓里抽出来比对引文用的几行文本；没有编译目录、工作树或仓副本。
