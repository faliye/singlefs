**出处 `.claude/kb/experiments/162-崩溃放量判定块存储选型.md:1-10`（整段抄，未转述）**

```markdown
## E162 崩溃放量判定块存储选型 —— S1 够判档已交，S2 / S3 还没跑

**中文** · [English](162-storage-selection-for-crash-verdict-blocks.en.md) · [日本語](162-クラッシュ増幅判定ブロックのストレージ選択.ja.md)

<!-- doc-lint:not-numbers V15 -->

跑前登记 `e162-preregistration.md`；问题单 `m2-crash-store-r1-forks.md`（S1、S2、S3）。按用户定案（登记第十二节修订六），先跑了一版只验能跑、能用的可行性档（S1 每条臂杀 20 次、S2 每条臂写 10⁴ 块），S1、S2 都记「未判（只跑了可行性档）」，S1-large、四个几何取样点（G-bs10、G-batch16、G-sparse、G-random）与 S3 未跑。同一天随后续跑了 S1 的够判档（第二段）：四条臂各按登记原定的 200 次杀主格全跑完，够判条件（第六节 6.1、11.1 V1–V6）逐条过闸，**S1 这一行够判：四条臂在这 200 次杀里都没有翻**（丢块、坏块、确认/未确认读报错、幽灵块、打不开全部 0 次）。S2、S3 这一段仍未跑，见「S1 够判档（2026-09-27，第二段）」一节。装置写在 `scripts/experiment-apparatus/e162_crash_verdict_block_store.rs`（独立手写模型，不进 `crates/`，不碰 `crates/singlefs-core`、`crates/singlefs-checker`），同一天早些时候 23 单测全绿（三遍，经 `run-with-memory-cap.sh` 4G）；第二段续跑 S1 够判档之前又重跑了一次同一条单测命令，仍 23 passed / 0 failed（`BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/gcc/x86_64-linux-gnu/13/include` 解开了上一版页面写的 `librocksdb-sys` 构建卡点，见「它答不了的」）；变异表 `scripts/mutation/tables/e162_crash_verdict_block_store.tsv` 13 条（`wc -l` 现查，行数未变，源码本身这一段没有改动），同一天早些时候跑过 `mutate.sh` 全部**抓到**（无效 0、没红 0、内存撞顶 0、超时 0），已还原、基线仍全绿；这一段没有再改源码，没有重跑变异表。

四条被测臂：F（加固的文件，临时文件 + fsync + 改名 + 目录 fsync）、R1（redb，`set_quick_repair(true)`）、R0（redb，默认修复）、K（RocksDB，同步写 + WAL）。

```

**出处 `.claude/kb/experiments/162-崩溃放量判定块存储选型.md:11-16`（整段抄，未转述）**

```markdown
### 这一段做了什么

- **S1**：每条臂各跑杀点计划前 20 行（18 次「写中途」+ 2 次「打开中途」，种子 `seed_kill=0xe162000000000003` 写死），每次杀后重开、逐块核（Q1a–Q1f）。阳性对照 PC-L、PC-S、PC-P、PC-O 四条臂都跑；PC-T（半截文件）按登记 5.3 只定义给 F。
- **S2**：每条臂各写 10⁴ 块（`states_per_block=65536`），报持续写入速率（Q2a）、关库重开后按块键随机读的冷/热时延（Q2d）、占用与写入字节之比（Q2e）。阳性对照 PC-rate、PC-read、PC-space 都跑。S1-large、G-bs10、G-batch16、G-sparse、G-random、S3 按修订六这次可行性档不跑。
- **anchors**：登记第七节 7.2 的 22 个独立算出的锚点（覆盖 SplitMix64 参考值、CRC-32C 标准校验值、块值生成、块键编码、置换、杀点计划、算术锚点等，逐项按跑前登记 13.3 锚点脚本的原样输出钉死）逐项与装置启动自检比对。

```

**出处 `.claude/kb/experiments/162-崩溃放量判定块存储选型.md:17-73`（整段抄，未转述）**

````markdown
### 结果整行抄自产物

**S1 判决行**（`grep -n 'name=verdict' research/results/e162-crash-verdict-block-store-2026-09-27-s1-{F,R1,R0,K}-feasibility.out`）：

```
research/results/e162-crash-verdict-block-store-2026-09-27-s1-F-feasibility.out:73:E7RESULT name=verdict part=s1 arm=F tier=feasibility judgement=not_judged_feasibility_tier_only kill_cycles=20 kills_inside_commit=18 pc_l_ok=true pc_s_ok=true pc_p_ok=true pc_o_ok=true pc_t_ok=true commit_waits_for_device=true calibration_interfered=true v6_kills_inside_commit_ok=not_applicable planned_confirmations=747 a_s1_ok=true q1a=0 q1b=0 q1c=0 q1d=0 q1e=0 q1f=0 repaired_openings=0 s1_positive_quantities=none
research/results/e162-crash-verdict-block-store-2026-09-27-s1-R1-feasibility.out:51:E7RESULT name=verdict part=s1 arm=R1 tier=feasibility judgement=not_judged_feasibility_tier_only kill_cycles=20 kills_inside_commit=18 pc_l_ok=true pc_s_ok=true pc_p_ok=true pc_o_ok=true pc_t_ok=not_applicable commit_waits_for_device=true calibration_interfered=true v6_kills_inside_commit_ok=not_applicable planned_confirmations=747 a_s1_ok=true q1a=0 q1b=0 q1c=0 q1d=0 q1e=0 q1f=0 repaired_openings=0 s1_positive_quantities=none
research/results/e162-crash-verdict-block-store-2026-09-27-s1-R0-feasibility.out:51:E7RESULT name=verdict part=s1 arm=R0 tier=feasibility judgement=not_judged_feasibility_tier_only kill_cycles=20 kills_inside_commit=18 pc_l_ok=true pc_s_ok=true pc_p_ok=true pc_o_ok=true pc_t_ok=not_applicable commit_waits_for_device=true calibration_interfered=true v6_kills_inside_commit_ok=not_applicable planned_confirmations=747 a_s1_ok=true q1a=0 q1b=0 q1c=0 q1d=0 q1e=0 q1f=0 repaired_openings=0 s1_positive_quantities=none
research/results/e162-crash-verdict-block-store-2026-09-27-s1-K-feasibility.out:51:E7RESULT name=verdict part=s1 arm=K tier=feasibility judgement=not_judged_feasibility_tier_only kill_cycles=20 kills_inside_commit=18 pc_l_ok=true pc_s_ok=true pc_p_ok=true pc_o_ok=true pc_t_ok=not_applicable commit_waits_for_device=true calibration_interfered=true v6_kills_inside_commit_ok=not_applicable planned_confirmations=747 a_s1_ok=true q1a=0 q1b=0 q1c=0 q1d=0 q1e=0 q1f=0 repaired_openings=0 s1_positive_quantities=none
```

**S1 阳性对照行**（四条臂各自的 PC-S / PC-P / PC-O / PC-L；F 另有 PC-T）：

```
research/results/e162-crash-verdict-block-store-2026-09-27-s1-F-feasibility.out:5:E7RESULT name=positive_control control=PC-S arm=F open_failed=false opened_after_repair=not_applicable q1a=0 q1b=1 q1b_blocks=7 q1c=0 q1d=0 q1e=0 enumerated_keys=64 ok=true
research/results/e162-crash-verdict-block-store-2026-09-27-s1-F-feasibility.out:6:E7RESULT name=positive_control control=PC-P arm=F open_failed=false opened_after_repair=not_applicable q1a=0 q1b=0 q1b_blocks=- q1c=0 q1d=0 q1e=1 enumerated_keys=65 ok=true
research/results/e162-crash-verdict-block-store-2026-09-27-s1-F-feasibility.out:7:E7RESULT name=positive_control control=PC-O arm=F open_failed=true opened_after_repair=not_applicable q1a=0 q1b=0 q1b_blocks=- q1c=0 q1d=0 q1e=0 enumerated_keys=0 ok=true open_error=file:_FORMAT_不对
research/results/e162-crash-verdict-block-store-2026-09-27-s1-F-feasibility.out:29:E7RESULT name=positive_control control=PC-L arm=F write_path=file-lazy-rename write_kills=18 write_kills_with_q1a=18 write_kills_with_new_q1a=14 ok=true
research/results/e162-crash-verdict-block-store-2026-09-27-s1-F-feasibility.out:51:E7RESULT name=positive_control control=PC-T arm=F write_kills=18 write_kills_with_read_error=16 q1b=0 extended_to_200=false ok=true
research/results/e162-crash-verdict-block-store-2026-09-27-s1-R1-feasibility.out:5:E7RESULT name=positive_control control=PC-S arm=R1 open_failed=false opened_after_repair=not_applicable q1a=0 q1b=1 q1b_blocks=7 q1c=0 q1d=0 q1e=0 enumerated_keys=64 ok=true
research/results/e162-crash-verdict-block-store-2026-09-27-s1-R1-feasibility.out:6:E7RESULT name=positive_control control=PC-P arm=R1 open_failed=false opened_after_repair=not_applicable q1a=0 q1b=0 q1b_blocks=- q1c=0 q1d=0 q1e=1 enumerated_keys=65 ok=true
research/results/e162-crash-verdict-block-store-2026-09-27-s1-R1-feasibility.out:7:E7RESULT name=positive_control control=PC-O arm=R1 open_failed=true opened_after_repair=not_applicable q1a=0 q1b=0 q1b_blocks=- q1c=0 q1d=0 q1e=0 enumerated_keys=0 ok=true open_error=redb:_I/O_error:_Not_a_redb_database:_magic_number_mismatch
research/results/e162-crash-verdict-block-store-2026-09-27-s1-R1-feasibility.out:29:E7RESULT name=positive_control control=PC-L arm=R1 write_path=redb-durability-none write_kills=18 write_kills_with_q1a=18 write_kills_with_new_q1a=18 ok=true
research/results/e162-crash-verdict-block-store-2026-09-27-s1-R0-feasibility.out:5:E7RESULT name=positive_control control=PC-S arm=R0 open_failed=false opened_after_repair=not_applicable q1a=0 q1b=1 q1b_blocks=7 q1c=0 q1d=0 q1e=0 enumerated_keys=64 ok=true
research/results/e162-crash-verdict-block-store-2026-09-27-s1-R0-feasibility.out:6:E7RESULT name=positive_control control=PC-P arm=R0 open_failed=false opened_after_repair=not_applicable q1a=0 q1b=0 q1b_blocks=- q1c=0 q1d=0 q1e=1 enumerated_keys=65 ok=true
research/results/e162-crash-verdict-block-store-2026-09-27-s1-R0-feasibility.out:7:E7RESULT name=positive_control control=PC-O arm=R0 open_failed=true opened_after_repair=not_applicable q1a=0 q1b=0 q1b_blocks=- q1c=0 q1d=0 q1e=0 enumerated_keys=0 ok=true open_error=redb:_I/O_error:_Not_a_redb_database:_magic_number_mismatch
research/results/e162-crash-verdict-block-store-2026-09-27-s1-R0-feasibility.out:29:E7RESULT name=positive_control control=PC-L arm=R0 write_path=redb-durability-none write_kills=18 write_kills_with_q1a=18 write_kills_with_new_q1a=18 ok=true
research/results/e162-crash-verdict-block-store-2026-09-27-s1-K-feasibility.out:5:E7RESULT name=positive_control control=PC-S arm=K open_failed=false opened_after_repair=not_applicable q1a=0 q1b=1 q1b_blocks=7 q1c=0 q1d=0 q1e=0 enumerated_keys=64 ok=true
research/results/e162-crash-verdict-block-store-2026-09-27-s1-K-feasibility.out:6:E7RESULT name=positive_control control=PC-P arm=K open_failed=false opened_after_repair=not_applicable q1a=0 q1b=0 q1b_blocks=- q1c=0 q1d=0 q1e=1 enumerated_keys=65 ok=true
research/results/e162-crash-verdict-block-store-2026-09-27-s1-K-feasibility.out:7:E7RESULT name=positive_control control=PC-O arm=K open_failed=true opened_after_repair=yes q1a=0 q1b=0 q1b_blocks=- q1c=0 q1d=0 q1e=0 enumerated_keys=64 ok=true open_error=rocksdb:_IO_error:_No_such_file_or_directory:_While_opening_a_file_for_sequentially_reading:_/tmp/e162-3127058/pc-o/MANIFEST-999999:_No_such_file_or_directory
research/results/e162-crash-verdict-block-store-2026-09-27-s1-K-feasibility.out:29:E7RESULT name=positive_control control=PC-L arm=K write_path=rocksdb-no-wal write_kills=18 write_kills_with_q1a=18 write_kills_with_new_q1a=18 ok=true
```

四条臂的 PC-S/PC-P/PC-O 除 `open_error` 文案不同外逐字段相同，都 `ok=true`；K 的 PC-O 另报 `opened_after_repair=yes`（`DB::repair` 修复后能重新打开），F/R1/R0 没有这个字段（登记只对 K 定义了修复动作）。

**S2 判决行**：

```
research/results/e162-crash-verdict-block-store-2026-09-27-s2-F-feasibility.out:41:E7RESULT name=verdict part=s2 arm=F tier=feasibility judgement=not_judged_feasibility_tier_only blocks=10000 pc_rate_ok=true pc_read_ok=true pc_space_ok=true s2_main_write_states_per_second=33530296.2 q2a_verdict=pending_e161 s1_large=not_run_feasibility_tier
research/results/e162-crash-verdict-block-store-2026-09-27-s2-R1-feasibility.out:41:E7RESULT name=verdict part=s2 arm=R1 tier=feasibility judgement=not_judged_feasibility_tier_only blocks=10000 pc_rate_ok=true pc_read_ok=true pc_space_ok=true s2_main_write_states_per_second=33593904.5 q2a_verdict=pending_e161 s1_large=not_run_feasibility_tier
research/results/e162-crash-verdict-block-store-2026-09-27-s2-R0-feasibility.out:41:E7RESULT name=verdict part=s2 arm=R0 tier=feasibility judgement=not_judged_feasibility_tier_only blocks=10000 pc_rate_ok=true pc_read_ok=true pc_space_ok=true s2_main_write_states_per_second=19339984.1 q2a_verdict=pending_e161 s1_large=not_run_feasibility_tier
research/results/e162-crash-verdict-block-store-2026-09-27-s2-K-feasibility.out:41:E7RESULT name=verdict part=s2 arm=K tier=feasibility judgement=not_judged_feasibility_tier_only blocks=10000 pc_rate_ok=true pc_read_ok=true pc_space_ok=true s2_main_write_states_per_second=48638773.4 q2a_verdict=pending_e161 s1_large=not_run_feasibility_tier
```

**S2 持续写入速率、随机读时延与占用比**（`name=throughput`、`name=random_read`、`name=occupancy_summary`，各产物第 11、39、40、37 行）：

| 臂 | 持续写入速率（状态/秒） | 提交时延中位/p99/最大（µs） | 冷读中位/p99（µs） | 热读中位/p99（µs） | 占用峰值比 `peak_ratio_high` | 关库后占用比 `after_close_ratio` |
|---|---|---|---|---|---|---|
| F | 33 530 296.2 | 1529 / 5036 / 583066 | 321 / 399 | 123 / 144 | 1.0639 | 1.0625 |
| R1 | 33 593 904.5 | 1550 / 4352 / 115139 | 230 / 352 | 5 / 8 | 2.0029 | 2.0021 |
| R0 | 19 339 984.1 | 903 / 8299 / 2558476 | 229 / 366 | 5 / 11 | 2.0039 | 2.0017 |
| K | 48 638 773.4 | 804 / 1293 / 1733112 | 157 / 367 | 12 / 21 | 2.1495 | 1.0013 |

四条臂的 `q2e_flips_at_1_9e10`、`q2e_flips_at_1_5e10`、`q2e_flips_at_1_9e10_using_low` 都是 `false`：按这次可行性档的占用比乘 1.9×10¹⁰ 或 1.5×10¹⁰ 字节都还落在开跑时 `df -B1` 报的可用字节内（F 可用 1 979 726 131 200，R1 1 979 601 149 952，R0 1 979 590 852 608，K 1 980 708 077 568）。R1、R0 关库时占用约是写入字节的 2 倍（redb 两条形态接近，`peak_ratio_high` 2.0029 / 2.0039）；K 写入过程中峰值到 2.1495，但关库后掉到 1.0013（RocksDB 的合并把占用压回接近写入字节）；F 全程稳定在约 1.06。这些数只是这次可行性档 10⁴ 块、`samples=10` 的单次裸读数，未重复跑、未判定，不当最终结论；持续写入速率之间的差异（R0 明显低于其余三臂）同理。

**anchors 判决**：`research/results/e162-crash-verdict-block-store-2026-09-27-anchors-2026-09-28.out` 里 `E7RESULT name=verdict part=anchors anchor_mismatches=0` 与 `E7RESULT name=done emitted=27` 两行；22 个 `name=anchor` 行全部 `matches_registration=true`（2026-09-28 路径搬迁重跑，与改名前的 `e162-crash-verdict-block-store-2026-09-27-anchors.out` 逐字节一致，`cmp` 确认）。

````

**出处 `.claude/kb/experiments/162-崩溃放量判定块存储选型.md:74-85`（整段抄，未转述）**

```markdown
### 判决行点名

`grep -n 'name=verdict' <产物>` 列出的判决行里，没有任何字段取值是布尔 `false`；出现的非 `true`、非零字段全部逐个如下（值全部落在登记第十二节修订六划定的这次可行性档范围内，是预期收窄，不是装置坏）：

- `judgement=not_judged_feasibility_tier_only`（8 份 S1/S2 产物全部）：这是登记修订六要求的措辞——用户 2026-09-27 定这次只验能跑、能用，S1、S2 都不据这次可行性档的数据下够判结论，字段名字面像「没有判定」，实际是设计如此。
- `pc_t_ok=not_applicable`（S1 的 R1、R0、K 三份）：PC-T（半截文件对照）按登记 5.3 只定义给 F-direct 这条候选，R1/R0/K 没有这个对照本来就该是 `not_applicable`，不是没跑。
- `v6_kills_inside_commit_ok=not_applicable`（S1 全部 4 份）：V6 的门槛是对 180 次「写中途」的杀写的（登记 11.1 作废条款 V6），可行性档只跑 20 次杀（18 次写中途），源码按 tier 直接给 `not_applicable`，与登记修订六「V6…可行性档只报杀在提交中间的次数、不判」一致。
- `s1_large=not_run_feasibility_tier`（S2 全部 4 份）：登记修订六写明这次可行性档 S1-large 不跑，字段照实报「未跑」，预期内。
- `q2a_verdict=pending_e161`（S2 全部 4 份）：Q2a 的门槛 R_AB 要用 E161（崩溃放量的去重与分段耗时） 交回的 t_state，E161（崩溃放量的去重与分段耗时） 同一天也只交了可行性档（见「它答不了的」），门槛还没有正式值可代入，字段照实报「等 E161（崩溃放量的去重与分段耗时）」。

`q1a`–`q1f`（S1，8 份中的 4 份 verdict 行）全部为 0：这次可行性档 20 次杀里没有丢块、静默坏块、确认/未确认读报错、幽灵块、打不开。`anchor_mismatches=0`：22 个锚点全部对上登记。判决行里没有任何名字表示违例、不匹配、歧义或失败的整数计数取正值。

```

**出处 `.claude/kb/experiments/162-崩溃放量判定块存储选型.md:86-110`（整段抄，未转述）**

````markdown
### 复跑

登记的那一行比对 `research/results/e162-verdict-store-power-cut-2026-09-27-selftest-r2.out`（49 行，末行前一行 `E7RESULT name=verdict part=anchors anchor_mismatches=0`）。

`replay.sh` 登记确定性的 anchors（`exact`，逐字节比），另登记三行全部臂（`timing`，按结构比）：`driver_e162_s1_all_arms`、`driver_e162_s2_all_arms` 对臂 R1、R0、K、F 按次序各起一次登记档、输出接成一份，`driver_e162_s4_all_arms` 起 `s4-drive`、不给 `--arms` 就是四臂全跑（起 QEMU 虚机）；`research/results/` 下还没有这三行的产物。可行性档 `feasibility` 是缩域，复跑表不跑：

```
bash scripts/experiment-apparatus/replay.sh E162
```
```
E162  @driver_e162_anchors     字节一致 e162-crash-verdict-block-store-2026-09-27-anchors-2026-09-28.out
```

（2026-09-28 路径搬迁：`replay.sh` 的 E162（崩溃放量判定块存储选型） 登记行改指这份新产物；与改名前的 `e162-crash-verdict-block-store-2026-09-27-anchors.out` 逐字节一致，见「历史版本」。）

直接重跑 anchors、S1、S2 的可行性档（`feasibility` 是缩域：产物的判定格写 `not_judged_feasibility_tier_only`，不当正式产物；不写这个参数就是登记档；各自独立一条命令，`<臂>` 取 `F`、`R1`、`R0`、`K` 之一；S1、S2 每次都会新起子进程杀、写，产物带时序噪声，与已入库的产物不逐字节比）：

```
task-server/task submit -- cargo run --release -p e7-index-bench --features e162-block-stores --bin e162-crash-verdict-block-store -- anchors
task-server/task submit -- cargo run --release -p e7-index-bench --features e162-block-stores --bin e162-crash-verdict-block-store -- s1 <臂> feasibility
task-server/task submit -- cargo run --release -p e7-index-bench --features e162-block-stores --bin e162-crash-verdict-block-store -- s2 <臂> feasibility
```

单测：`task-server/task submit --cwd scripts/experiment-apparatus -- cargo test --release -p e7-index-bench --features e162-block-stores --bin e162-crash-verdict-block-store`（同一天早些时候 23 passed / 0 failed；本页写作时重跑这条命令被 `librocksdb-sys` 的构建环境挡住，见「它答不了的」）。

````

**出处 `.claude/kb/experiments/162-崩溃放量判定块存储选型.md:111-183`（整段抄，未转述）**

````markdown
### S1 够判档（2026-09-27，第二段）

跑前登记第十二节修订六只批了可行性档；这一段按主 agent 派发（继承用户定案「可以跑 可以用 然后把 验证交给 Milestone 2 remaining tasks 去」，来历 `records/2026-09-24-里程碑二收尾调度.md` 第三节）把 S1 从 20 次杀补到登记原定的 200 次，S1 的够判条件（第五节 5.5、第六节 6.1、第十一节 11.1 V1–V6）没有改一个字；S2、S3 这一段没有跑，留给机器空闲的另一段。

**⚠ 登记补记缺口（执行员的疏漏）**：这一段跑产物之前没有先往登记（`e162-preregistration.md` 第十二节）追加「修订七」记录这一段的收窄范围（按纪律该在装置跑之前写、写完再跑）。这一条只把可行性档的样本量放大到登记原定的值、第六、七、九节的判据一个字没动，不影响下面的判定；产物已经跑出来之后按纪律不许再改登记，这处缺口交主 agent 处置（补记还是接受缺口）。

#### 这一段做了什么

- **S1（够判档）**：四条臂各按登记原定的杀点计划跑满 200 次（`registered` 档，不带 `feasibility` 参数，与源码 `CRASH_MAIN_KILL_CYCLES=200` 对应），逐次杀后重开、逐块核（Q1a–Q1f）；阳性对照 PC-L、PC-S、PC-P、PC-O（F 另有 PC-T）全部重新跑过一遍新库。跑之前重跑了一次单测（见「复跑」），没有改动源码，没有重跑变异表。
- **S2、S3**：这一段未跑。

#### 结果整行抄自产物

**S1 判决行**（`grep -n 'name=verdict' research/results/e162-crash-verdict-block-store-2026-09-27-s1-{F,R1,R0,K}-decisive.out`）：

```
research/results/e162-crash-verdict-block-store-2026-09-27-s1-F-decisive.out:258:E7RESULT name=verdict part=s1 arm=F tier=registered judgement=judged_by_registration kill_cycles=200 kills_inside_commit=180 pc_l_ok=true pc_s_ok=true pc_p_ok=true pc_o_ok=true pc_t_ok=true commit_waits_for_device=true calibration_interfered=true v6_kills_inside_commit_ok=true planned_confirmations=6178 a_s1_ok=true q1a=0 q1b=0 q1c=0 q1d=0 q1e=0 q1f=0 repaired_openings=0 s1_positive_quantities=none
research/results/e162-crash-verdict-block-store-2026-09-27-s1-R1-decisive.out:235:E7RESULT name=verdict part=s1 arm=R1 tier=registered judgement=judged_by_registration kill_cycles=200 kills_inside_commit=179 pc_l_ok=true pc_s_ok=true pc_p_ok=true pc_o_ok=true pc_t_ok=not_applicable commit_waits_for_device=true calibration_interfered=false v6_kills_inside_commit_ok=true planned_confirmations=6178 a_s1_ok=true q1a=0 q1b=0 q1c=0 q1d=0 q1e=0 q1f=0 repaired_openings=0 s1_positive_quantities=none
research/results/e162-crash-verdict-block-store-2026-09-27-s1-R0-decisive.out:233:E7RESULT name=verdict part=s1 arm=R0 tier=registered judgement=judged_by_registration kill_cycles=200 kills_inside_commit=178 pc_l_ok=true pc_s_ok=true pc_p_ok=true pc_o_ok=true pc_t_ok=not_applicable commit_waits_for_device=true calibration_interfered=false v6_kills_inside_commit_ok=true planned_confirmations=6178 a_s1_ok=true q1a=0 q1b=0 q1c=0 q1d=0 q1e=0 q1f=0 repaired_openings=0 s1_positive_quantities=none
research/results/e162-crash-verdict-block-store-2026-09-27-s1-K-decisive.out:234:E7RESULT name=verdict part=s1 arm=K tier=registered judgement=judged_by_registration kill_cycles=200 kills_inside_commit=180 pc_l_ok=true pc_s_ok=true pc_p_ok=true pc_o_ok=true pc_t_ok=not_applicable commit_waits_for_device=true calibration_interfered=false v6_kills_inside_commit_ok=true planned_confirmations=6178 a_s1_ok=true q1a=0 q1b=0 q1c=0 q1d=0 q1e=0 q1f=0 repaired_openings=0 s1_positive_quantities=none
```

**S1 主格逐项汇总行**（`grep -n 'name=kill_campaign_summary campaign=S1-main' research/results/e162-crash-verdict-block-store-2026-09-27-s1-{F,R1,R0,K}-decisive.out`，200 次杀的累计计数与重开用时）：

```
research/results/e162-crash-verdict-block-store-2026-09-27-s1-F-decisive.out:257:E7RESULT name=kill_campaign_summary campaign=S1-main arm=F write_path=registered cycles_verified=200 q1a_lost=0 q1b_corrupted=0 q1c_confirmed_read_errors=0 q1d_unconfirmed_read_errors=0 q1e_phantoms=0 q1f_open_failures=0 q1f_timeouts=0 repaired_openings=0 q1g_kills_inside_commit=180 write_phase_kills=180 confirmations_before_write_kills=6178 write_kills_with_loss=0 write_kills_with_new_loss=0 write_kills_with_read_error=0 reopen_peak_microseconds=2171 reopen_median_microseconds=1100 reopen_last_microseconds=2171 reopen_over_60s=0 rises=- first_rise_cycle=- stopped_after_unopenable=false assertion_failures=-
research/results/e162-crash-verdict-block-store-2026-09-27-s1-R1-decisive.out:234:E7RESULT name=kill_campaign_summary campaign=S1-main arm=R1 write_path=registered cycles_verified=200 q1a_lost=0 q1b_corrupted=0 q1c_confirmed_read_errors=0 q1d_unconfirmed_read_errors=0 q1e_phantoms=0 q1f_open_failures=0 q1f_timeouts=0 repaired_openings=0 q1g_kills_inside_commit=179 write_phase_kills=180 confirmations_before_write_kills=6178 write_kills_with_loss=0 write_kills_with_new_loss=0 write_kills_with_read_error=0 reopen_peak_microseconds=1407 reopen_median_microseconds=1015 reopen_last_microseconds=439 reopen_over_60s=0 rises=- first_rise_cycle=- stopped_after_unopenable=false assertion_failures=-
research/results/e162-crash-verdict-block-store-2026-09-27-s1-R0-decisive.out:232:E7RESULT name=kill_campaign_summary campaign=S1-main arm=R0 write_path=registered cycles_verified=200 q1a_lost=0 q1b_corrupted=0 q1c_confirmed_read_errors=0 q1d_unconfirmed_read_errors=0 q1e_phantoms=0 q1f_open_failures=0 q1f_timeouts=0 repaired_openings=0 q1g_kills_inside_commit=178 write_phase_kills=180 confirmations_before_write_kills=6178 write_kills_with_loss=0 write_kills_with_new_loss=0 write_kills_with_read_error=0 reopen_peak_microseconds=399403 reopen_median_microseconds=124691 reopen_last_microseconds=472 reopen_over_60s=0 rises=- first_rise_cycle=- stopped_after_unopenable=false assertion_failures=-
research/results/e162-crash-verdict-block-store-2026-09-27-s1-K-decisive.out:233:E7RESULT name=kill_campaign_summary campaign=S1-main arm=K write_path=registered cycles_verified=200 q1a_lost=0 q1b_corrupted=0 q1c_confirmed_read_errors=0 q1d_unconfirmed_read_errors=0 q1e_phantoms=0 q1f_open_failures=0 q1f_timeouts=0 repaired_openings=0 q1g_kills_inside_commit=180 write_phase_kills=180 confirmations_before_write_kills=6178 write_kills_with_loss=0 write_kills_with_new_loss=0 write_kills_with_read_error=0 reopen_peak_microseconds=3271087 reopen_median_microseconds=11411 reopen_last_microseconds=5818 reopen_over_60s=0 rises=- first_rise_cycle=- stopped_after_unopenable=false assertion_failures=-
```

四条臂的 `cycles_verified=200`（够判条件要的次数）、`q1a_lost`–`q1f_timeouts` 与 `repaired_openings` 全部 0；`q1g_kills_inside_commit`（180/179/178/180）都 ≥ 100（V6 门槛，180 次写中途的杀）、`write_phase_kills=180`、`confirmations_before_write_kills=6178`，与登记 A8 锚点「写中途那 180 行 n_i 之和 6178」一致。阳性对照四条臂的 PC-S、PC-P、PC-O、PC-L（F 另有 PC-T）都 `ok=true`：`q1b=1` 落在第 7 块、`q1e=1`、`open_failed=true`（K 另报 `opened_after_repair=yes`）、PC-T 这次 `write_kills_with_read_error=18`（可行性档 20 次杀里是 16，这次 200 次杀的前 20 行种子相同，`q1d`/`q1c` 累计口径不同，18/18 与登记 V5「18 次写中途的杀里 Q1c+Q1d ≥ 1」一致），与登记的钉死值都对得上、都过（`grep -n 'name=positive_control' research/results/e162-crash-verdict-block-store-2026-09-27-s1-{F,R1,R0,K}-decisive.out` 现查，不逐行重抄）。

重开用时（Q1h，轨迹，登记不拿它判翻面）四条臂差异明显：F 中位 1100 µs、峰值 2171 µs；R1 中位 1015 µs、峰值 1407 µs；R0 中位 124 691 µs（约 125 ms）、峰值 399 403 µs（约 0.4 秒）；K 中位 11 411 µs、峰值 3 271 087 µs（约 3.27 秒：K 的 `calibration_interfered=false`，这个峰值不是校准阶段撞干扰的产物，是 S1-main 200 次杀主格里单次重开撞上本机同期别的会话在跑 `cargo test` 的极端值，属于 Q1h 轨迹的一个高点，不代表这条臂常态的重开时延）。R0（redb 默认修复、不开 quick-repair）比 R1（quick-repair）重开慢两个量级，方向与 redb 4.3.0 文档注释「默认模式崩溃后重开要把整个库走一遍，quick-repair 几乎立刻完成」（登记第四节 4.6）一致；这是轨迹观察，不是 S1 的判据。

#### 判决行点名

`grep -n 'name=verdict'` 列出的这四行判决行里：

- 没有任何字段取值是布尔 `false`。
- 没有任何字段取值是 `not_run`。
- `pc_t_ok=not_applicable`（R1、R0、K 三份）：PC-T 只定义给 F，其余三条臂本来就该是 `not_applicable`，不是没跑；与可行性档同一读法。
- `calibration_interfered=true`（仅 F 一份；R1、R0、K 三份都是 `false`）：F 的 S1-calibration 撞上 V15 的干扰重跑上限（`INTERFERENCE_MAXIMUM_RERUNS=3`，四次尝试都撞见别的会话的 `cargo test`），按登记 11.1 V15「重跑，最多三次…还有就带受干扰标记照报」处理，不作废 S1（V15 只作废「那一趟的计时」，S1 的判据看 Q1a–Q1f，不看 calibration 的计时）。
- 名字表示违例、不匹配、歧义或失败的整数计数（`q1a`、`q1b`、`q1c`、`q1d`、`q1e`、`q1f`、`repaired_openings`）四条臂全部为 `0`。
- 其余字段（`judgement=judged_by_registration`、`pc_l_ok`/`pc_s_ok`/`pc_p_ok`/`pc_o_ok=true`、`commit_waits_for_device=true`、`v6_kills_inside_commit_ok=true`、`a_s1_ok=true`、`s1_positive_quantities=none`）都是预期的「过」，不点名。

按第六节 6.1 与第十一节 11.1 V1–V6：四条臂的阳性对照都过（V1–V5 不作废），V6 的门槛（180 次写中途的杀里 Q1g ≥ 100）四条臂都过（178–180），200 次主格里 Q1a–Q1f 全部为 0——**S1 够判：redb（quick-repair 与默认修复两种形态）、RocksDB（同步写 + WAL）、加固的文件，在这次 SIGKILL 模型下四条臂都没有丢已确认的块、没有静默坏块、没有幽灵块、没有打不开**。按第十节末尾「结果反过来我接不接受」写死的读法：F 在 S1 上不翻不能说它比数据库更安全（4.10 的注：F 这几格由定义加内核语义直接推得出为 0，不是它自己的恢复代码扛住的），这里写成「四个候选在这个 SIGKILL 模型下都没有翻」，不写成「都一样安全」。

#### 复跑

单测（这一段重跑一次，确认这份改动过环境变量的构建仍然过闸）：

```
task-server/task submit --cwd scripts/experiment-apparatus -- env BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/gcc/x86_64-linux-gnu/13/include cargo test --release -p e7-index-bench --features e162-block-stores --bin e162-crash-verdict-block-store
```

23 passed / 0 failed（上一版页面记的 `librocksdb-sys` / `bindgen` 卡点由 `BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/gcc/x86_64-linux-gnu/13/include` 解开，见「它答不了的」）。

这四份决判档产物不进 `replay.sh` 逐字节比对，原因与可行性档相同：S1 每次都新起子进程按真实挂钟杀、真实盘上持久时延重开，两次跑不是同一批字节（这一段亲眼见到：F 这次撞上四次干扰重跑，可行性档那次一次没撞上，同一份代码、同一批种子，产物就是不同字节）。直接重跑（`<臂>` 取 `F`、`R1`、`R0`、`K` 之一；不带 `feasibility` 参数即登记原定的 `registered` 档，200 次杀）：

```
task-server/task submit -- env BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/gcc/x86_64-linux-gnu/13/include cargo run --release -p e7-index-bench --features e162-block-stores --bin e162-crash-verdict-block-store -- s1 <臂>
```

这一段登记未变（2026-09-28 路径搬迁前），只有 `driver_e162_anchors` 一行；复跑命令跑出来的行与「结果整行抄自产物」一节抄的可行性档那次输出逐字一致：

```
bash scripts/experiment-apparatus/replay.sh E162
```
```
E162  @driver_e162_anchors     字节一致 e162-crash-verdict-block-store-2026-09-27-anchors-2026-09-28.out
```

````

**出处 `.claude/kb/experiments/162-崩溃放量判定块存储选型.md:184-196`（整段抄，未转述）**

```markdown
### 它答不了的

装置 `e162_verdict_store_power_cut.rs` 在 2026-11-09 改过（自证里取装置源码、`preflight.py` 与 `vm-bench.sh` 路径的写法改成 `concat!(env!("CARGO_MANIFEST_DIR"), …)`，`vm-bench.sh` 的路径改到 `scripts/devices/vm-bench.sh`；读数的算术没动），之后没有正式跑；要数的时候等用户说跑。

- **S1 够判（2026-09-27 第二段补跑），S2 仍未判**：S1 四条臂各跑满登记原定的 200 次主格杀，阳性对照与 V6 门槛都过，Q1a–Q1f 全部 0，见「S1 够判档」一节；S2 仍只跑了可行性档的 10⁴ 块（登记要 10⁵ 块才够判），且 Q2a 的判据要代入 E161（崩溃放量的去重与分段耗时） 的正式 t_state，E161（崩溃放量的去重与分段耗时） 同一天也只交了可行性档（下面 Q2a 那条）。S1-large、四个几何取样点也仍未跑，见「S1-large、G-bs10、G-batch16、G-sparse、G-random 四个几何取样点未跑」一条。
- **S3 完全未跑**：经网络送块的持续吞吐、网络中断重连后的一致性、跨机那一格，登记第十二节修订六写明可行性档不跑；这一段（2026-09-27 第二段）按派发只补 S1，S3 仍未跑，等机器空闲的另一段。
- **S1-large、G-bs10、G-batch16、G-sparse、G-random 四个几何取样点未跑**：占用比、随机读、写入速率在这些取样点上会不会与主取样点反向，还没有数据。
- **Q2a 的门槛还没有正式值**：R_AB = (P_A + P_B) ÷ t_state 需要 E161（崩溃放量的去重与分段耗时） 的正式 t_state，E161（崩溃放量的去重与分段耗时） 同一天也只交了可行性档（实验页 [161-崩溃放量的去重与分段耗时.md](161-崩溃放量的去重与分段耗时.md)，g1–g5 未判）。E161（崩溃放量的去重与分段耗时） 那份产物的 `name=g3 cell=first_small` 行报 `states=37 state_elapsed_ns=77898270`（`research/results/e161-crash-state-dedup-and-time-split-feasibility-2026-09-27.out` 里那一行），除以状态数得约 2 105 359 ns/状态（约 2.11 ms/状态）——**这是单个取样格、有干扰（该行同一产物里 `calibration_interfered` 类字段与 E161（崩溃放量的去重与分段耗时） 实验页正文标注同批跑的机器上一直有编译与别的装置在跑）的粗数，只当量级参考，不代入 E162（崩溃放量判定块存储选型） 任何判定**；正式 t_state 要等 E161（崩溃放量的去重与分段耗时） 按登记 6.0 的加权闭式跑完全域才有。
- **占用比只跑了一次、`samples=10` 一趟**：R1、R0 的关库占用约 2 倍写入字节、K 峰值到 2.1495 再靠合并压回 1.0013，这三个数有没有随写入量变化、会不会随几何取样点（G-bs10 的小块、G-sparse 的可压缩值）反向，这次可行性档没跑。
- **13 条变异是这份源码改动之前那一版跑出来的**（2026-09-27 早些时候；`scripts/mutation/tables/e162_crash_verdict_block_store.tsv` 的抓到/无效/没红三个数是那次改动之后跑的；2026-09-27 第二段没有再改源码，没有重跑变异表——三个数仍是当天早些时候那一次的）。
- **登记第十二节缺「修订七」**：2026-09-27 第二段跑 S1 够判档之前没有先补记登记修订，见「S1 够判档」一节开头的补记缺口。
- **`librocksdb-sys` 这一次能编过，上一版遇到的卡点原因仍没查**：这一页早些时候写作时重跑单测撞到 `bindgen` 找不到 `stdbool.h`（本机只有 `libclang1-18` 运行库，没有 `clang` 可执行文件，缺 resource-dir 里的标准头）；2026-09-27 第二段设 `BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/gcc/x86_64-linux-gnu/13/include` 之后单测与四条臂的 `cargo build`/`cargo run` 都过了，说明这个环境变量能绕开那处卡点，但两次之间本机环境具体起了什么变化（为什么不设这个变量就找不到 `stdbool.h`）没有查；下一次要动这份源码，先照这段的复跑命令设这个变量。

```

**出处 `.claude/kb/experiments/162-崩溃放量判定块存储选型.md:197-202`（整段抄，未转述）**

```markdown
### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 4 | 备料 | 2026-11-14 不受影响：装置自证里取装置源码、`preflight.py` 与 `vm-bench.sh` 的路径写法改了，读数的算术没动，之后没有正式跑，结论、数与产物不变；该分项只备料，分项正文没点名这次改的东西；此前 2026-11-05 不受影响：只改用法与复跑说明（一个门禁、两种跑法；验证包改名），结论、数与产物不变；此前 2026-11-01 不受影响：只改复跑起法；此前 2026-10-24 不受影响：装置经 replay.sh 现跑复核，产物与留存产物逐字节完全一致，结论与数据未变；此前 2026-10-18 不受影响：复跑一节补点名 replay.sh 登记的那份产物，结论、数不变 |

```

**出处 `.claude/kb/experiments/174-借据丢掉时等待与显存回还.md:1-25`（整段抄，未转述）**

````markdown
## E174 借据丢掉时等待与显存回还 —— 两段跑完、够判

**中文** · [English](174-wait-and-gpu-memory-return-on-lost-iou.en.md) · [日本語](174-借用証書喪失時の待機とGPUメモリ返却.ja.md)

<!-- doc-lint:not-numbers G1 G2 S1 S2 Q1 Q1b Q2a Q2b Q2c Q3 Q3b Q3c K1 K2 K3 K4 K5 K6 K7 K8 K9 B1 B2 B3 B4 B5 B6 B7 F1 F2 F3 F4 F5 F6 F7 F8 V1 V2 V3 V4 V5 V6 V7 V8 V9 V10 P1 P2 M1 M2 M3 M4 M5 M6 M7 M8 M9 M10 M11 M12 M13 M14 M15 M16 M17 M18 M19 M20 M21 M22 M23 M24 M25 M26 M27 M28 -->

```text
本页的本地臂名与记号写小写（j1、j2 ……）：跑前登记、装置、产物与命令里写作对应的大写，与全局登记的编号同形；kb 正文里改成小写只为与那些编号分开，读产物时按大写对。
```

跑前登记 `research/prompts/e174-preregistration.md`（装置与单测写完、真卡产物一次没跑之前的 sha256 `b2d5cd0c56552ec1ad322d5d8cca22f1fcb705617cb876d3726635058edc0d92`；执行员在第十二节「修订」写了 1 条：补一条臂「在飞·中」、钉死几处登记没写死的读法，原判据一字没动）；岔路单 `m3-batch10z-a-code-r1-forks.md` 的 G2 一行里"乙"的两个数：借据丢掉时先等设备把这笔借据提交的活做完再销账（`device.poll` 用等待那一种），等一次要多久（G2-乙-等待）；等完、缓冲丢掉之后，下一次 `nvidia-smi -q -d MEMORY,PIDS` 读到这个进程占着的是否已经不含那笔缓冲，含多少、隔多久才不含（G2-乙-读数）。问法里"再丢掉缓冲"按登记的读法：缓冲对象在借据活着时先丢，借据丢掉时再 poll，poll 返回那一刻算销账。

装置是独立手写的，不是入库装置：`scripts/experiment-apparatus/e174_lease_drop_wait_memory_return.rs`（bin 名 `e174-lease-drop-wait-memory-return`，特性 `e174-gpu`，依赖沿用 E163（GPU多卡算单元校验和） 那份钉死的 `wgpu =30.0.1`、`pollster =1.0.1`），不依赖 `crates/`、不与实现共用代码。设备怎么起（Vulkan、`required_limits: adapter.limits()`、`MemoryHints::Performance`）、读数口径（`nvidia-smi -q -d MEMORY,PIDS` 按进程号把 `Used GPU Memory` 加起来）、`PollType::Poll` 的写法都抄自跑时 `crates/` 的取法，开跑前用登记第十一节那 7 条 `grep -c` 回比过：输出依次 2、1、1、1、1、1、1，与登记写的值逐条相同；驱动版本现读是 595.84（`nvidia-smi --query-gpu=index,name,pci.bus_id,memory.total,driver_version --format=csv` 两张卡都报 595.84，Total 32607 MiB 与 16311 MiB）。

**现状**：两段真卡产物都跑完、`judge` 判过，G2-乙 的两个数在两段上都量到了，登记第十一节的够判条件满足（状态「两段跑完、够判」）。

- **第一段**（设备上只有这一笔借据的活，不含在飞·轻）：88 格、446 个子进程，430 个完整、16 个失败；16 个失败全在 5060 Ti 的 8 GiB、不等待臂，wgpu 报 Out of Memory，那两格（在飞·中、在飞·重各一格）作废，别的 86 格、168 栏都凑满 5 个有效轮。
- **第二段**（同一设备两条线程共用，不含在飞·轻）：12 格、60 个子进程全完整，24 栏。
- **在飞·轻**两段都没跑，记「够判后未跑」：登记第九节 K9 在登记自己的数下必红（读一次显存的时长大于那一档活的 0.8 倍），修订 1 补的「在飞·中」一条臂在每个用在飞·轻的组里顶了它。
- **G2-乙-读数**：等待臂（第二段还有 `Wait` 带自己最后一次提交号的那一条臂）在两段里每一栏（第一段的阳性对照「扣住不丢」除外，它按设计含），t0 之后的第一份 `nvidia-smi -q -d MEMORY,PIDS` 读数都不含那笔缓冲，窗口末也不留；不等待臂：第一段里活还在飞、S 不小于 512 MiB 时第一份读数含整笔、2 秒窗口里一直不还，第二段 S = 2048 MiB 的 4 栏第一份读数含整笔、0.7 到 1.15 秒后还回去。读数与计数命令在「读数：第一段」「读数：第二段」两节。
- **G2-乙-等待**：已做完档的等待 27 微秒以内；活在飞的两档，等待比剩下的活本身多出来的部分从负几百微秒到 213.6 毫秒不等；同一设备另有一条线程一直在提交 400 毫秒的活时，等一次要 0.3 到 0.69 秒。数在两节读数里。
- 用户已照两段的数在岔路单 `m3-batch10z-a-code-r1-forks.md` G2 一行定「改用乙」，实现在第 10z 批第六块，不在这一页。

不起卡的部分（装置、单测 19 个、变异表 28 条全抓、自检、模型卡上的父子流水线）的读数留在「读数：不起卡的几份」「单测与变异」两节。

````

**出处 `.claude/kb/experiments/174-借据丢掉时等待与显存回还.md:26-33`（整段抄，未转述）**

```markdown
### 做了什么

- **一笔借据的步骤**（登记第五节）写成对着 `CardHost` 这个 trait 的一份代码，有两份实现：真卡（`WgpuCardHost`）与模型卡（`SimulatedCardHost`，虚拟时钟，读一份报告记 50 毫秒，小于 256 MiB 的缓冲共用一块 256 MiB 的块、块用过就留着，更大的单独拿，释放要等缓冲被丢掉、提交做完、之后有一次 poll）。登记第九节的 M1–M7 都改在对着 trait 的那份代码上，所以不起卡也有单测可红。模型卡只复现步骤的算术，不是 wgpu 与驱动的行为，不拿来当 G2-乙 的读数。
- **子命令**：`run`（父进程，按清单一个接一个起子进程，等它退出后整份读、不在转发输出的循环里打时间戳；续跑、补轮、原文压成 `.raw.gz`、扣住不丢组出现 F1 就停）、`child`（一个装置进程）、`judge`（读产物判，确定性）、`checks`（不起卡的自检，确定性）、`plan`（清单各阶段的格数与子进程数，确定性）、`simulate`（在模型卡上跑某一格几轮，试 `judge` 用）；`run --simulated` 让每个子进程跑在模型卡上，`--limit-jobs N` 冒烟，`--skip-light` 先不跑在飞·轻（这两样是缩域，产物 config 行写出实际值；不给就是整段）。
- **清单**（`e174-lease-drop-wait-memory-return-plan-first-2026-10-08.out`、`…-plan-second-2026-10-08.out`）：第一段 128 格、首轮 640 个子进程，第二段 24 格、首轮 120 个子进程；"扣住不丢"最先，在飞·轻最后。
- **修订 1**：登记第九节 K9 要 e < 0.8 W，而 e 至少含 U_peak 那一次 -q 读与 B5 另读的一次 compute-apps（各约 0.05 秒），在飞·轻 W = 50 毫秒的 0.8 倍只有 40 毫秒，K9 在登记自己的数下必红。在飞·轻照登记保留、K9 照登记判，另补一条臂「在飞·中」（空转目标 250 毫秒）给每个用在飞·轻的组；已经作废的借据不再读窗口。其余是读法钉死：一轮有效的定义、补轮从 6 起至多 3 次、次序、B5 字面读法、F1 停机、判别力自证怎么取新 τ。
- **判法**：`judge --input 产物` 对每格按"第 1 笔"与"第 2、3 笔"两栏、取轮号最小的 5 个有效轮，逐轮列出 Q1（等多久）、Q2a / Q2b（t0 之后第一份是否含）、Q2c（隔多久不含）、Q3（窗口末留着）、Q3b（按块多拿）、Q3c（wgpu 分配器），再给一行 `name=verdict` 汇总 F1–F8 与几何敏感性；汇总里"…_ok"类字段写成通过的样子，`false` 就是那条没过（F2、F3、F4 打中是登记认的正当结果，不是装置有错）。

```

**出处 `.claude/kb/experiments/174-借据丢掉时等待与显存回还.md:34-59`（整段抄，未转述）**

````markdown
### 复跑

```
# 仓根；装置在 scripts/experiment-apparatus/（不依赖 crates/）。真卡上的命令也整条经 task-server 交，内存额度、线程由任务服务定，卡与清场归 GPU 服务、任务不领卡不清场
cd /home/fy5090/code/singlefs/scripts/experiment-apparatus
# 不起卡：单测 19 条、变异表 28 条、自检、清单
../../task-server/task submit -- cargo test --release -p e7-index-bench --features e174-gpu --bin e174-lease-drop-wait-memory-return
../../task-server/task submit -- bash ../mutation/mutate.sh e174-lease-drop-wait-memory-return e174_lease_drop_wait_memory_return.rs ../mutation/tables/e174_lease_drop_wait_memory_return.tsv
../../task-server/task submit -- cargo run --release -p e7-index-bench --features e174-gpu --bin e174-lease-drop-wait-memory-return -- checks
../../task-server/task submit -- cargo run --release -p e7-index-bench --features e174-gpu --bin e174-lease-drop-wait-memory-return -- plan --segment first
# 上卡：先冒烟两个子进程，再第一段（不含在飞·轻），再在飞·轻，再第二段；同一条命令重起就是续跑；一律经 scripts/experiment-apparatus/device-code-key.py run 在仓根起
cd ../..
task-server/task submit -- python3 scripts/experiment-apparatus/device-code-key.py run . --binary e174-lease-drop-wait-memory-return --feature e174-gpu --entry e7_index_bench::run_child_on_the_card --entry e7_index_bench::ChildJob::child_arguments -- run --segment first --skip-light --limit-jobs 2 --output research/results/e174-lease-drop-wait-memory-return-segment-one-<日期>.out
# 判（确定性，读产物）
task-server/task submit -- cargo run --release --manifest-path scripts/experiment-apparatus/Cargo.toml -p e7-index-bench --features e174-gpu --bin e174-lease-drop-wait-memory-return -- judge --input research/results/e174-lease-drop-wait-memory-return-segment-one-<日期>.out --scope first
```

两段真卡产物是 `run --segment first --skip-light` 与 `run --segment second --skip-light` 写的（产物第 1 行 `config` 的 `skip_light=true` 为证），命令形态是冒烟那条命令去掉 `--limit-jobs`（`gpu-quiesce-run.sh -- … run --segment first --skip-light --output …`，第二段换 `--segment second`）。真卡产物含计时与显存读数，两次跑不同：复跑表的 `driver_e174_run_segment_one`、`driver_e174_run_segment_two` 两行经 `device-code-key.py run` 不带 `--skip-light`、`--limit-jobs` 整段重跑两段，按 `timing` 比结构；逐字节比的是它们的 `judge` 输出（确定性，读产物）。判的命令，仓根下跑：

```
task-server/task submit -- cargo run --release --manifest-path scripts/experiment-apparatus/Cargo.toml -p e7-index-bench --features e174-gpu --bin e174-lease-drop-wait-memory-return -- judge --input research/results/e174-lease-drop-wait-memory-return-segment-one-2026-10-08.out --scope first
task-server/task submit -- cargo run --release --manifest-path scripts/experiment-apparatus/Cargo.toml -p e7-index-bench --features e174-gpu --bin e174-lease-drop-wait-memory-return -- judge --input research/results/e174-lease-drop-wait-memory-return-segment-two-2026-10-08.out --scope second
```

`scripts/experiment-apparatus/replay.sh` 登记六行，整段重跑的两行 E174（借据丢掉时等待与显存回还） `|@driver_e174_run_segment_one`、E174（借据丢掉时等待与显存回还） `|@driver_e174_run_segment_two`（`timing`）之外是这四行：E174（借据丢掉时等待与显存回还） `|@driver_e174_checks`、E174（借据丢掉时等待与显存回还） `|@driver_e174_simulated_judge`（不起卡、不含计时字段的两份）与 E174（借据丢掉时等待与显存回还） `|@driver_e174_segment_one_judge`、E174（借据丢掉时等待与显存回还） `|@driver_e174_segment_two_judge`（真卡两段产物的 `judge` 输出，产物 `…-segment-one-judge-2026-10-08.out`、`…-segment-two-judge-2026-10-08.out`）。replay.sh 整份目前一开跑就在准入表检查上退 2（`bash scripts/experiment-apparatus/replay.sh --admission-only` 报「E142（新池新建文件的干跑） 登记了准入，而这张复跑表里它有 2 行」），与这四行无关，所以这四行没经 replay.sh 复跑，是手工跑同一条命令，输出与登记的产物 `cmp` 逐字节相同：两段的 judge 命令都退 0、stderr 为空，`cmp` 无输出。

````

**出处 `.claude/kb/experiments/174-借据丢掉时等待与显存回还.md:60-68`（整段抄，未转述）**

```markdown
### 产物清单

`research/results/` 下 E174（借据丢掉时等待与显存回还） 的全部产物：
- 自检与清单：`e174-lease-drop-wait-memory-return-checks-2026-10-08.out`、`e174-lease-drop-wait-memory-return-plan-first-2026-10-08.out`、`e174-lease-drop-wait-memory-return-plan-second-2026-10-08.out`；
- 单测与变异：`e174-lease-drop-wait-memory-return-unit-tests-2026-10-08-r2.out`、`e174-lease-drop-wait-memory-return-mutate-2026-10-08-r2.log` 是现行的；`e174-lease-drop-wait-memory-return-unit-tests-2026-10-08.out`、`e174-lease-drop-wait-memory-return-mutate-2026-10-08.log` 是被 r2 取代的第一次；
- 模型卡流水线：`e174-lease-drop-wait-memory-return-simulated-second-2026-10-08.out`、原文 `e174-lease-drop-wait-memory-return-simulated-second-2026-10-08.out.raw.gz`、判的输出 `e174-lease-drop-wait-memory-return-simulated-second-judge-2026-10-08.out`；
- 真卡第一段：`e174-lease-drop-wait-memory-return-segment-one-2026-10-08.out`、原文 `e174-lease-drop-wait-memory-return-segment-one-2026-10-08.out.raw.gz`、判的输出 `e174-lease-drop-wait-memory-return-segment-one-judge-2026-10-08.out`；
- 真卡第二段：`e174-lease-drop-wait-memory-return-segment-two-2026-10-08.out`、原文 `e174-lease-drop-wait-memory-return-segment-two-2026-10-08.out.raw.gz`、判的输出 `e174-lease-drop-wait-memory-return-segment-two-judge-2026-10-08.out`。

```

**出处 `.claude/kb/experiments/174-借据丢掉时等待与显存回还.md:69-75`（整段抄，未转述）**

```markdown
### 读数：不起卡的几份（模型卡数据，不是 G2-乙 的读数）

- **自检**（`research/results/e174-lease-drop-wait-memory-return-checks-2026-10-08.out`，33 行，31 条自检）：第 32 行 `E7RESULT name=verdict scope=checks self_checks_total=31 self_checks_failed=0 void_conditions=0`。检查本身分得出差别：第 16–18 行三条负对照（模型卡被注入别的进程号、compute-apps 比 -q 多 3 MiB、标定不在带内）各自打出对应的作废理由。第 12 行 `E7RESULT name=self_check id=simulated_light_trips_k9_at_the_registered_numbers passed=true`：模型卡上在飞·轻按登记的数 K9 必红，复现的是修订 1 的算术。
- **清单**：`…-plan-first-2026-10-08.out` 里 `E7RESULT name=verdict scope=plan segment=first cells_total=128 first_pass_children_total=640 void_conditions=0`；`…-plan-second-2026-10-08.out` 里 `E7RESULT name=verdict scope=plan segment=second cells_total=24 first_pass_children_total=120 void_conditions=0`。
- **父子流水线走通**（`run --simulated --segment second --skip-light`，子进程都在模型卡上）：`…-simulated-second-2026-10-08.out`（243 行）第 242 行 `E7RESULT name=verdict scope=run segment=second cells_total=12 cells_incomplete=0 children_total=60 children_complete=60 children_failed=0 children_void=0 stopped_by=none`，原文压缩包 `…-simulated-second-2026-10-08.out.raw.gz`（`zcat` 读得出，每个子进程一段）。同一条命令用 `--limit-jobs 7`、`--limit-jobs 20` 分三次续跑，与一次跑完的产物和原文 `cmp` 逐字节相同（草稿读数，续跑文件不入库）。
- **判法在模型卡产物上**：`…-simulated-second-judge-2026-10-08.out`（26 行）第 25 行 `E7RESULT name=verdict scope=second columns_judged=24 cells_void=0 hold_control_holds=not_run wait_arm_next_reading_absent_everywhere=false wait_arm_end_of_window_not_kept_everywhere=false wait_always_returns=true poll_arm_also_absent_where_wait_arm_absent=false in_flight_cells_void_for_work_not_in_flight=0 long_window_clear_within_window=not_run geometry_same_as_main_grid=not_run done_wait_expensive_candidate_columns=0 discrimination_real_data_flipped_columns=12 discrimination_synthetic_flips=true`。这几个 `false` 来自模型卡自己的规则（小于 256 MiB 的借据让进程多占一块 256 MiB 的块、块用过就留着），模型卡上的数不是 wgpu 与驱动的数；`not_run` 是这份产物里没有对应的组。判别力自证：真数据里 τ 挪到含着的读数的中位数时 12 栏翻面，另用模型卡产物自证 `discrimination_synthetic_flips=true`。

```

**出处 `.claude/kb/experiments/174-借据丢掉时等待与显存回还.md:76-274`（整段抄，未转述）**

````markdown
### 读数：第一段（真卡，设备上只有这一笔借据的活）

产物 `research/results/e174-lease-drop-wait-memory-return-segment-one-2026-10-08.out`（1699 行，原文压缩包是同名加 `.raw.gz`），判的输出 `research/results/e174-lease-drop-wait-memory-return-segment-one-judge-2026-10-08.out`（188 行）。命令都在 `research/results/` 下跑，先设四个变量（`S2`、`j2` 是第二段两份）：

```
cd /home/fy5090/code/singlefs/research/results
S1=e174-lease-drop-wait-memory-return-segment-one-2026-10-08.out
J1=e174-lease-drop-wait-memory-return-segment-one-judge-2026-10-08.out
S2=e174-lease-drop-wait-memory-return-segment-two-2026-10-08.out
J2=e174-lease-drop-wait-memory-return-segment-two-judge-2026-10-08.out
```

产物第 1 行与第 1698 行（`grep -n 'name=config' $S1`、`grep -n 'name=verdict' $S1`，原样带行号；真卡产物含计时与显存读数，再跑要上卡，不登记进 `scripts/experiment-apparatus/replay.sh`，登记的是它的判的输出）：

```
1:E7RESULT name=config segment=first seed=174 driver_version=595.84 skip_light=true cells=88 rounds_required=5 refills_at_most=3 simulated=false
1698:E7RESULT name=verdict scope=run segment=first cells_total=88 cells_incomplete=2 children_total=446 children_complete=430 children_failed=16 children_void=0 stopped_by=none
```

判的输出第 187 行（`grep -n 'name=verdict' $j1`）：

```
E7RESULT name=verdict scope=first columns_judged=168 cells_void=2 hold_control_holds=true wait_arm_next_reading_absent_everywhere=true wait_arm_end_of_window_not_kept_everywhere=true wait_always_returns=true poll_arm_also_absent_where_wait_arm_absent=false in_flight_cells_void_for_work_not_in_flight=0 long_window_clear_within_window=true geometry_same_as_main_grid=false done_wait_expensive_candidate_columns=0 discrimination_real_data_flipped_columns=40 discrimination_synthetic_flips=true
```

**栏数**（组 × 臂；每格分「第 1 笔」「第 2、3 笔」两栏，长窗一格一栏）与每栏的轮数：

```
grep 'name=cell_verdict' $J1 | awk '{print $3,$8}' | sort | uniq -c
      8 group=coexisting_long_lease arm=poll
      8 group=coexisting_long_lease arm=wait
      8 group=hold_control arm=poll
      8 group=hold_control arm=wait
      4 group=long_window arm=wait
     56 group=main arm=poll
     60 group=main arm=wait
      8 group=pass_by_pass arm=poll
      8 group=pass_by_pass arm=wait
grep -h -o 'rounds_used=[0-9]*' $J1 $J2 | sort | uniq -c
    192 rounds_used=5
```

主格子不等待臂 56 栏而不是 60：5060 Ti 上 8 GiB 的两格作废（最后一条命令的 192 是两段 168 + 24 合计）。

**阳性对照「扣住不丢」**（登记 F1）：缓冲对象在 t0 之后还活着，读数必须含它。两条臂、两张卡、512 MiB 与 2 GiB、在飞·中，16 栏 R_next 都含整笔，判的输出里 `hold_control_holds=true`：

```
grep 'name=cell_verdict' $J1 | grep ' group=hold_control ' | grep -o ' q2_next_reading=[a-z_]*' | sort | uniq -c
     16  q2_next_reading=present
```

**G2-乙-读数，等待臂**（登记 Q2a、Q2c、Q3）：除扣住不丢之外的 80 栏等待臂（主格子 60、已做完逐趟 8、并存的长借据 8、长窗 4），t0 之后的第一份读数都不含，隔 0 毫秒就不含，窗口末（2 秒；长窗 30 秒）也不留：

```
grep 'name=cell_verdict' $J1 | grep -v ' group=hold_control ' | grep ' arm=wait ' | grep -o ' q2_next_reading=[a-z_]*\| q2c_time_to_clear=[a-z_]*\| q3_end_of_window=[a-z_]*' | sort | uniq -c
     80  q2c_time_to_clear=next_reading_already_absent
     80  q2_next_reading=absent
     80  q3_end_of_window=absent
```

**不等待臂**（登记 Q2b，跑时 `crates/` 的做法）：活已经做完的档（已做完、逐趟）下一次读数不含；活还在飞（中、重）的档，S 不小于 512 MiB 时下一次读数含整笔，窗口里一直不还（`not_absent_within_window`）；S = 1、64 MiB 不含。8 GiB 在 5060 Ti 上没有读数（「作废的两格」一段）：

```
grep 'name=cell_verdict' $J1 | grep -v ' group=hold_control ' | grep ' arm=poll ' | grep -o ' q2_next_reading=[a-z_]*\| q2c_time_to_clear=[a-z_]*' | paste - - | sort | uniq -c
     48  q2_next_reading=absent	 q2c_time_to_clear=next_reading_already_absent
     24  q2_next_reading=present	 q2c_time_to_clear=not_absent_within_window
grep 'name=cell_verdict' $J1 | grep ' group=main ' | grep ' arm=poll ' | grep -v ' liveness=done ' | awk '{for(i=1;i<=NF;i++){split($i,a,"=");d[a[1]]=a[2]} print d["card"],d["size_mib"],d["liveness"],d["q2_next_reading"]}' | sort | uniq -c
      2 rtx5060ti 1 heavy absent
      2 rtx5060ti 1 medium absent
      2 rtx5060ti 2048 heavy present
      2 rtx5060ti 2048 medium present
      2 rtx5060ti 512 heavy present
      2 rtx5060ti 512 medium present
      2 rtx5060ti 64 heavy absent
      2 rtx5060ti 64 medium absent
      2 rtx5090 1 heavy absent
      2 rtx5090 1 medium absent
      2 rtx5090 2048 heavy present
      2 rtx5090 2048 medium present
      2 rtx5090 512 heavy present
      2 rtx5090 512 medium present
      2 rtx5090 64 heavy absent
      2 rtx5090 64 medium absent
      2 rtx5090 8192 heavy present
      2 rtx5090 8192 medium present
```

第一条命令的 24 个「含」是主格子 20 栏（5090 的 512、2048、8192 MiB 与 5060 Ti 的 512、2048 MiB，各在飞·中、在飞·重两档，每格两栏）加并存的长借据 64 MiB 的 4 栏；并存的长借据那一组写在同名的一段。

**G2-乙-等待**（登记 Q1、Q1b）：每栏 5 轮的 T_wait 与 `judged`（已做完档就是 T_wait；在飞档是 T_wait 超出剩下的活的部分 E，负数是等待比剩下的活短）按组、档、臂取全部轮的最小与最大（微秒）。已做完档的等待最长 27 微秒，在飞·中 175–392 毫秒、在飞·重 325–571 毫秒；不等待臂都在 24 微秒以内：

```
awk '/name=cell_verdict/ && !/ group=hold_control /{for(i=1;i<=NF;i++){split($i,a,"=");d[a[1]]=a[2]} n=split(d["q1_wait_microseconds"],v,";"); m=split(d["q1_judged_microseconds"],w,";"); k=d["group"]" "d["liveness"]" "d["arm"]; for(j=1;j<=n;j++){x=v[j]+0; if(!(k in lo)||x<lo[k])lo[k]=x; if(!(k in hi)||x>hi[k])hi[k]=x} for(j=1;j<=m;j++){y=w[j]+0; if(!(k in jlo)||y<jlo[k])jlo[k]=y; if(!(k in jhi)||y>jhi[k])jhi[k]=y}} END{for(k in lo)print k, "wait_us", lo[k], hi[k], "judged_us", jlo[k], jhi[k]}' $J1 | sort
coexisting_long_lease medium poll wait_us 1 12 judged_us 1 12
coexisting_long_lease medium wait wait_us 175786 254725 judged_us -175 17137
long_window medium wait wait_us 161848 248056 judged_us -124 33292
main done poll wait_us 1 22 judged_us 1 22
main done wait wait_us 1 27 judged_us 1 27
main heavy poll wait_us 2 24 judged_us 2 24
main heavy wait wait_us 325083 571300 judged_us -196 213567
main medium poll wait_us 1 20 judged_us 1 20
main medium wait wait_us 175290 392336 judged_us -185 165674
pass_by_pass done poll wait_us 2 11 judged_us 2 11
pass_by_pass done wait wait_us 2 13 judged_us 2 13
```

每栏的判定（登记 Q1 的门槛：已做完档 5 轮都 ≤ 5 毫秒记「零头」；在飞档 5 轮都 ≤ 20 毫秒记「只等活本身」，都 > 20 毫秒记「等得比活长」，否则「不稳定」）。已做完档 28 栏全是「零头」，`done_wait_expensive_candidate_columns=0`（没有「乙的代价翻面候选」）；在飞档 34 栏「只等活本身」、13 栏「等得比活长」、5 栏「不稳定」，没有一栏「等不回来」（判的输出里 `wait_always_returns=true`）：

```
grep 'name=cell_verdict' $J1 | grep -v ' group=hold_control ' | grep ' arm=wait ' | awk '{for(i=1;i<=NF;i++){split($i,a,"=");d[a[1]]=a[2]} print d["liveness"],d["q1_wait"]}' | sort | uniq -c
     28 done zero_overhead
      6 heavy longer_than_the_work
     12 heavy only_the_work_itself
      2 heavy unstable
      7 medium longer_than_the_work
     22 medium only_the_work_itself
      3 medium unstable
grep -h -c 'q1_wait_never_returned=true' $J1 $J2
0
0
```

在飞档里不是「只等活本身」的 18 栏列在这里：5090 的 8192 MiB、5060 Ti 的 2048 与 8192 MiB 是「等得比活长」，别的是 5 轮不都在 20 毫秒同一侧的「不稳定」：

```
grep 'name=cell_verdict' $J1 | grep ' arm=wait ' | grep -v ' liveness=done ' | grep -v ' group=hold_control ' | awk '{for(i=1;i<=NF;i++){split($i,a,"=");d[a[1]]=a[2]} if(d["q1_wait"]!="only_the_work_itself") print d["group"],d["card"],d["size_mib"],d["liveness"],d["column"],d["q1_wait"]}' | sort
long_window rtx5060ti 2048 medium first longer_than_the_work
long_window rtx5090 2048 medium first unstable
main rtx5060ti 2048 heavy first longer_than_the_work
main rtx5060ti 2048 heavy later longer_than_the_work
main rtx5060ti 2048 medium first longer_than_the_work
main rtx5060ti 2048 medium later longer_than_the_work
main rtx5060ti 512 heavy later unstable
main rtx5060ti 512 medium later unstable
main rtx5060ti 8192 heavy first longer_than_the_work
main rtx5060ti 8192 heavy later longer_than_the_work
main rtx5060ti 8192 medium first longer_than_the_work
main rtx5060ti 8192 medium later longer_than_the_work
main rtx5090 2048 heavy later unstable
main rtx5090 2048 medium later unstable
main rtx5090 8192 heavy first longer_than_the_work
main rtx5090 8192 heavy later longer_than_the_work
main rtx5090 8192 medium first longer_than_the_work
main rtx5090 8192 medium later longer_than_the_work
```

**Q3、Q3b、Q3c**（窗口末留着、按块多拿、wgpu 分配器）：等待臂窗口末都不留（Q3 80 栏 `absent`，见 Q2a 那条命令的第三行）；wgpu 分配器在等待臂 80 栏都「不留块」。不等待臂有 24 栏「wgpu 留着块」，这 24 栏都是第一份读数含的那 24 栏。并存的长借据 64 MiB 的两条臂各 4 栏 Q3b 判「按块多拿」（缓冲活着时进程多占的比这笔多），等待臂等完之后仍不含：

```
grep 'name=cell_verdict' $J1 | grep -v ' group=hold_control ' | awk '{for(i=1;i<=NF;i++){split($i,a,"=");d[a[1]]=a[2]} print d["arm"],"q3b="d["q3b_block_overtake"],"q3c="d["q3c_wgpu_blocks"]}' | sort | uniq -c
     48 poll q3b=absent q3c=wgpu_does_not_keep_blocks
     20 poll q3b=absent q3c=wgpu_keeps_blocks
      4 poll q3b=present q3c=wgpu_keeps_blocks
     76 wait q3b=absent q3c=wgpu_does_not_keep_blocks
      4 wait q3b=present q3c=wgpu_does_not_keep_blocks
grep 'name=cell_verdict' $J1 | grep -v ' group=hold_control ' | grep ' arm=poll ' | grep ' q2_next_reading=present ' | grep -c 'q3c_wgpu_blocks=wgpu_keeps_blocks'
24
```

**并存的长借据**（登记第八节几何，F8）：子进程里另有一张 64 MiB 缓冲活到子进程末尾，借据 S 取 1、64 MiB，在飞·中。判的输出逐对比主格子同一格，等待臂 8 对都相同，不等待臂的 S = 64 MiB 四对不同——并存的长借据下第一份读数含 256 MiB（一整块），主格子里不含。`geometry_same_as_main_grid=false` 来自这四对：

```
grep 'name=geometry_pair' $J1 | grep -c 'same=true'
12
grep 'name=geometry_pair' $J1 | grep ' arm=wait ' | grep -c 'same=true'
8
grep -n 'name=geometry_pair' $J1 | grep 'same=false'
177:E7RESULT name=geometry_pair group=coexisting_long_lease card=rtx5090 size_mib=64 liveness=medium method=one_build arm=poll column=first q2_main_grid=absent q2_coexisting=present q3_main_grid=absent q3_coexisting=present same=false
178:E7RESULT name=geometry_pair group=coexisting_long_lease card=rtx5090 size_mib=64 liveness=medium method=one_build arm=poll column=later q2_main_grid=absent q2_coexisting=present q3_main_grid=absent q3_coexisting=present same=false
185:E7RESULT name=geometry_pair group=coexisting_long_lease card=rtx5060ti size_mib=64 liveness=medium method=one_build arm=poll column=first q2_main_grid=absent q2_coexisting=present q3_main_grid=absent q3_coexisting=present same=false
186:E7RESULT name=geometry_pair group=coexisting_long_lease card=rtx5060ti size_mib=64 liveness=medium method=one_build arm=poll column=later q2_main_grid=absent q2_coexisting=present q3_main_grid=absent q3_coexisting=present same=false
```

登记 F8 写的是「Q2a、Q3 在那几格记『不稳定』，乙在这一形上不够判」，触发观测是「第八节逐格对，主格子在飞·轻与『并存的长借据』组判定不同」；第八节同时写「被判的是『等待臂读数不含』与『留不留着』两件事」。按字面逐格对，F8 在不等待臂的四对上打中（比的是在飞·中，不是在飞·轻）；只看被判的等待臂，八对都相同，F8 没打中。登记没写不等待臂上的差别算不算打中，两种读法并列在这里，哪种承重交主 agent。

**作废的两格**（登记 V8：同一格有 2 轮子进程报 Out of Memory 或非 0 退出，那一格作废并交回原文）：5060 Ti 上 S = 8 GiB 的不等待臂，在飞·中与在飞·重各一格，轮 1 至 8（首轮 5 个加补轮 3 个）的子进程都以 wgpu 的 Out of Memory 退出；同一张卡同一个 S 的等待臂 6 栏（三档 × 两栏）正常，不等待臂的已做完档 2 栏也正常。16 个 `child_failed` 行的原文（stderr 末尾）在产物里；第几笔借据报的错没有留下，因为失败的子进程没有 `lease` 行，原文压缩包里也没有它们的段：

```
grep -c 'name=child_failed' $S1
16
grep 'name=child_failed' $S1 | grep -c 'card=rtx5060ti size_mib=8192 .* arm=poll'
16
grep 'name=child_failed' $S1 | grep -c 'Out_of_Memory'
16
grep -c 'name=cell_void' $J1
2
grep 'name=cell_verdict' $J1 | grep ' card=rtx5060ti size_mib=8192 ' | awk '{print $6,$8}' | sort | uniq -c
      2 liveness=done arm=poll
      2 liveness=done arm=wait
      2 liveness=heavy arm=wait
      2 liveness=medium arm=wait
grep 'name=lease ' $S1 | grep -c 'card=rtx5060ti size_mib=8192 liveness=\(medium\|heavy\) .*arm=poll'
0
zcat $S1.raw.gz | grep -c '^######## .*card=rtx5060ti size_mib=8192 liveness=\(medium\|heavy\) .*arm=poll'
0
```

**判别力自证**（登记第八节）：把 τ 换成含着的读数的中位数，真数据上 40 栏判定翻面（`discrimination_real_data_flipped_columns=40`），模型卡产物另证 `discrimination_synthetic_flips=true`（两项都在判的输出第 187 行）。

````

**出处 `.claude/kb/experiments/174-借据丢掉时等待与显存回还.md:275-340`（整段抄，未转述）**

````markdown
### 读数：第二段（真卡，同一设备几条线程共用）

产物 `research/results/e174-lease-drop-wait-memory-return-segment-two-2026-10-08.out`（243 行，原文压缩包是同名加 `.raw.gz`），判的输出 `research/results/e174-lease-drop-wait-memory-return-segment-two-judge-2026-10-08.out`（26 行）。一个子进程两条线程：线程 B 在同一个设备上一次接一次提交 400 毫秒的活，线程 A 做一笔借据（S = 64 MiB、2 GiB，在飞·中，空转目标 250 毫秒，一次建）；臂三条：`Wait { submission_index: None }`（下记 wait）、`Wait { submission_index: Some(线程 A 这笔借据最后一次提交) }`（wait_own）、`Poll`（poll）。两张卡，每格 5 轮。变量同第一段那一节。

产物第 1 行与第 242 行（`grep -n 'name=config' $S2`、`grep -n 'name=verdict' $S2`，原样带行号；不登记进 `scripts/experiment-apparatus/replay.sh`，理由同第一段）：

```
1:E7RESULT name=config segment=second seed=174 driver_version=595.84 skip_light=true cells=12 rounds_required=5 refills_at_most=3 simulated=false
242:E7RESULT name=verdict scope=run segment=second cells_total=12 cells_incomplete=0 children_total=60 children_complete=60 children_failed=0 children_void=0 stopped_by=none
```

判的输出第 25 行（`grep -n 'name=verdict' $j2`）：

```
E7RESULT name=verdict scope=second columns_judged=24 cells_void=0 hold_control_holds=not_run wait_arm_next_reading_absent_everywhere=true wait_arm_end_of_window_not_kept_everywhere=true wait_always_returns=true poll_arm_also_absent_where_wait_arm_absent=false in_flight_cells_void_for_work_not_in_flight=0 long_window_clear_within_window=not_run geometry_same_as_main_grid=not_run done_wait_expensive_candidate_columns=0 discrimination_real_data_flipped_columns=4 discrimination_synthetic_flips=true
```

**G2-乙-读数**（Q2a、Q2c、Q3、Q3c）：wait 与 wait_own 各 8 栏，t0 之后第一份读数都不含、隔 0 毫秒就不含、窗口末不留、wgpu 不留块；poll 在 S = 2048 MiB 的 4 栏第一份读数含整笔 2048 MiB，之后在窗口（2 秒）里还回去，T_clear 的 20 个值（两张卡 × 两栏 × 5 轮）在 0.71 到 1.15 秒之间（判定名 `absent_after_wait`：每轮都有限，不是都为 0）；S = 64 MiB 的 4 栏不含。第一段的不等待臂在同样是 S = 2048 MiB、活在飞的格子里 2 秒窗口里一直不还（`not_absent_within_window`），第二段的 poll 会还。两段的差别是第二段有线程 B 一直在设备上提交活，读数还回去与它有没有关系，这些数分不出来，页上不下结论：

```
grep 'name=cell_verdict' $J2 | awk '{for(i=1;i<=NF;i++){split($i,a,"=");d[a[1]]=a[2]} print d["arm"],d["q1_wait"],"q2="d["q2_next_reading"],"q2c="d["q2c_time_to_clear"],"q3="d["q3_end_of_window"],"q3c="d["q3c_wgpu_blocks"]}' | sort | uniq -c
      4 poll does_not_wait q2=absent q2c=next_reading_already_absent q3=absent q3c=wgpu_does_not_keep_blocks
      4 poll does_not_wait q2=present q2c=absent_after_wait q3=absent q3c=wgpu_does_not_keep_blocks
      8 wait longer_than_the_work q2=absent q2c=next_reading_already_absent q3=absent q3c=wgpu_does_not_keep_blocks
      8 wait_own longer_than_the_work q2=absent q2c=next_reading_already_absent q3=absent q3c=wgpu_does_not_keep_blocks
grep 'name=cell_verdict' $J2 | grep ' arm=poll ' | awk '{for(i=1;i<=NF;i++){split($i,a,"=");d[a[1]]=a[2]} print d["card"],d["size_mib"],d["column"],"q2="d["q2_next_reading"],d["q2_remainders_mebibytes"]}' | sort
rtx5060ti 2048 first q2=present 2048;2048;2048;2048;2048
rtx5060ti 2048 later q2=present 2048;2048;2048;2048;2048
rtx5060ti 64 first q2=absent 0;0;0;0;0
rtx5060ti 64 later q2=absent 0;0;0;0;0
rtx5090 2048 first q2=present 2048;2048;2048;2048;2048
rtx5090 2048 later q2=present 2048;2048;2048;2048;2048
rtx5090 64 first q2=absent 0;0;0;0;0
rtx5090 64 later q2=absent 0;0;0;0;0
grep 'name=cell_verdict' $J2 | grep ' arm=poll ' | grep ' size_mib=2048 ' | grep -o ' q2c_values_microseconds=[0-9;]*'
 q2c_values_microseconds=803316;707802;743818;732683;719862
 q2c_values_microseconds=1025177;903407;897814;1016805;793476
 q2c_values_microseconds=797972;860259;821002;1026236;851788
 q2c_values_microseconds=932552;854468;1015238;1146156;986349
```

**G2-乙-等待**（Q1、Q1b）：按臂、栏取全部轮的 T_wait（`wait_us`）与超出剩下的活的部分（`judged_us`）的最小与最大（微秒）。wait 与 wait_own 的区间大幅重叠（wait 306–691 毫秒、wait_own 301–639 毫秒），两条臂的 16 栏都判「等得比活长」；poll 在 19 微秒以内。第 2、3 笔那一栏（`later`）的下界比第 1 笔高（wait 418 对 306 毫秒，wait_own 393 对 301 毫秒）：

```
awk '/name=cell_verdict/{for(i=1;i<=NF;i++){split($i,a,"=");d[a[1]]=a[2]} n=split(d["q1_wait_microseconds"],v,";"); m=split(d["q1_judged_microseconds"],w,";"); k=d["arm"]" "d["column"]; for(j=1;j<=n;j++){x=v[j]+0; if(!(k in lo)||x<lo[k])lo[k]=x; if(!(k in hi)||x>hi[k])hi[k]=x} for(j=1;j<=m;j++){y=w[j]+0; if(!(k in jlo)||y<jlo[k])jlo[k]=y; if(!(k in jhi)||y>jhi[k])jhi[k]=y}} END{for(k in lo)print k, "wait_us", lo[k], hi[k], "judged_us", jlo[k], jhi[k]}' $J2 | sort
poll first wait_us 2 13 judged_us 2 13
poll later wait_us 6 19 judged_us 6 19
wait first wait_us 306115 682744 judged_us 122617 508382
wait later wait_us 418499 691320 judged_us 225340 497541
wait_own first wait_us 300820 532027 judged_us 100692 317773
wait_own later wait_us 393284 638682 judged_us 207968 417178
```

wait_own 的区间整体略低于 wait（first 栏上界 532 毫秒对 683 毫秒），但两条臂的 5 轮值没有按轮配对比过，页上不下「Some 比 None 省」的结论。产物里 `child_done` 60 个、`child_failed` 0 个、`lease` 行 180 条：

```
grep -c 'name=child_done' $S2
60
grep -c 'name=child_failed' $S2
0
grep -c 'name=lease ' $S2
180
```

**判别力自证**：真数据上 4 栏判定翻面，模型卡产物另证翻面（判的输出第 25 行 `discrimination_real_data_flipped_columns=4`、`discrimination_synthetic_flips=true`）。

````

**出处 `.claude/kb/experiments/174-借据丢掉时等待与显存回还.md:341-356`（整段抄，未转述）**

```markdown
### 判决行的点名

四份产物的判决行（`grep -n 'name=verdict' $S1 $j1 $S2 $j2`）逐个点名没过的字段；字段名表示「通过」的取值（`true`、`zero`、0）不点名：

| 产物与行 | 字段 | 取值 | 为什么，是不是登记预期内的 |
|---|---|---|---|
| `$S1` 第 1698 行 | `cells_incomplete` | 2 | 5060 Ti、8 GiB、不等待臂的两格（在飞·中、在飞·重）补满 3 次仍 0 个有效轮，原因是 16 个子进程 Out of Memory（登记 V8，同一格 2 轮这样的就作废并交回原文）。V8 预期内，Out of Memory 本身登记没有预料到是哪一格 |
| `$S1` 第 1698 行 | `children_failed` | 16 | 5060 Ti、8 GiB、不等待臂那两格的 16 个子进程 Out of Memory，16 个 `child_failed` 行 |
| `$j1` 第 187 行 | `cells_void` | 2 | 5060 Ti、8 GiB、不等待臂那两格，判的输出里是 `name=cell_void` 两行 |
| `$j1` 第 187 行 | `poll_arm_also_absent_where_wait_arm_absent` | false | 登记 F5（不等待臂在每个等待臂判「不含」的格上也判「不含」）的反面：不等待臂在 24 栏含着，F5 没打中，登记预期内 |
| `$j1` 第 187 行 | `geometry_same_as_main_grid` | false | 登记 F8，不等待臂四对不同、等待臂八对相同，「并存的长借据」一段两种读法并列，哪种承重交主 agent |
| `$j2` 第 25 行 | `poll_arm_also_absent_where_wait_arm_absent` | false | 同 F5 的反面，poll 在 S = 2048 MiB 的 4 栏含着，登记预期内 |
| `$j2` 第 25 行 | `hold_control_holds`、`long_window_clear_within_window`、`geometry_same_as_main_grid` | not_run | 第二段只有 `shared_device` 一个组，没有扣住不丢、长窗、并存的长借据三个组，登记第五节第二段只写了共用设备一形，预期内 |

别的判决字段：`$S2` 第 242 行 `cells_incomplete=0 children_failed=0 children_void=0`；`$j1` 与 `$j2` 的 `wait_arm_next_reading_absent_everywhere=true`、`wait_arm_end_of_window_not_kept_everywhere=true`、`wait_always_returns=true`、`in_flight_cells_void_for_work_not_in_flight=0`、`done_wait_expensive_candidate_columns=0`、`discrimination_synthetic_flips=true`；`$j1` 的 `hold_control_holds=true`、`long_window_clear_within_window=true`。`discrimination_real_data_flipped_columns` 是 40（第一段）与 4（第二段），是判别力自证的栏数、不是「没过」。

```

**出处 `.claude/kb/experiments/174-借据丢掉时等待与显存回还.md:357-364`（整段抄，未转述）**

```markdown
### 单测与变异

续跑键（装置代码键）的样本与复跑（2026-10-15）：`e174-lease-drop-wait-memory-return-resume-key-samples-2026-10-15.out` 判决行 `key_samples=8 key_samples_holding=8 f_k1_missed=0 f_k2_extra=0 unit_test_samples=2`；同日整表变异 `e174-lease-drop-wait-memory-return-mutate-2026-10-15.log` 收尾「计数：内存撞顶 0 条（上限 6G）、超时 0 条、排不上 0 条、被总上限挤掉 0 条、显存不够 0 条、现判 34 条、复用 0 条、拒算 0 条」「已还原，基线仍全绿」。

单测 19 个：`grep -c '^test .* \.\.\. ok$' e174-lease-drop-wait-memory-return-unit-tests-2026-10-08-r2.out` 输出 19，同一份文件里 `test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`。变异表 `scripts/mutation/tables/e174_lease_drop_wait_memory_return.tsv` 28 行（`grep -c '' scripts/mutation/tables/e174_lease_drop_wait_memory_return.tsv` 输出 28），`mutate.sh` 输出 `e174-lease-drop-wait-memory-return-mutate-2026-10-08-r2.log`：`grep -c '^✅' …` 输出 28，`grep -c '^⏭' …` 与 `grep -c '^❌' …` 都输出 0；第 30、31 行 `计数：内存撞顶 0 条（上限 6G）、超时 0 条、排不上 0 条、被总上限挤掉 0 条、显存不够 0 条` 与 `已还原，基线仍全绿`。单测与变异各有一份更早的输出（`…-unit-tests-2026-10-08.out`、`…-mutate-2026-10-08.log`，同样 19 个、28 条全抓），对的是之后又改了 `run` 续跑语义（已有 `name=done` 行但要补轮或换一组格时接着跑并追加新的 verdict 与 done）之前的源码，留着，正文引的数以 r2 为准。

变异表的行（页内编号）：M1–M7 是登记第九节的七条（M1 等待臂的 `Wait` 换成 `Poll`、M2 拆缓冲张数向上取整换成向下取整、M3 把 BAR1 下的 `Used` 当成进程占着的、M4 建缓冲时每张取一半、M5 扣住不丢组在第 6 步照样丢缓冲、M6 t0 记在 poll 开始之前、M7 在飞档第 4 步也等完）；M8–M28 是执行员补的二十一条，改判定一侧的门槛与比较（M8–M13、M25）、读数解析与分配器比较（M15–M17）、外层计时与 K7、K9、扣住不丢组的窗口比较（M18、M22–M24）、续跑与清单（M14、M19–M21、M26–M28）。登记第九节说"判定脚本那几处门槛的变异由第八节的判别力自证代替"，这里另补了变异，不算替代。M5 登记写的取样点（"等待臂某一格 R_next 已经 ≤ τ 的 S"）在模型卡上就有（等待臂 2048 MiB 的 R_next 为 0），变异被抓，真卡上有没有这样的格要看产物。

```

**出处 `.claude/kb/experiments/174-借据丢掉时等待与显存回还.md:365-377`（整段抄，未转述）**

```markdown
### 岔路表

登记第一节的岔路单只有 G2 一行，拆成乙的两个数；第二段把每个数再按「设备上只有这一笔借据的活」（第一段）与「同一设备几条线程共用」（第二段）分成两形。每一行的依据是「读数：第一段」「读数：第二段」两节里对应的命令输出。

| 岔路行 | 状态 | 依据 | 还差什么，剩下的量能不能让它翻面 |
|---|---|---|---|
| G2-乙-等待，设备上只有这一笔借据的活 | 已够判 | 第一段 Q1 的 `awk` 区间命令与 `q1_wait` 计数命令：已做完档 28 栏「零头」（最长 27 微秒），在飞档 34 栏「只等活本身」、13 栏「等得比活长」（超出最多 213.6 毫秒）、5 栏「不稳定」，0 栏「等不回来」 | 无。剩下的量只有在飞·轻，改变不了这一行 |
| G2-乙-读数，设备上只有这一笔借据的活 | 已够判 | 第一段 Q2a、Q2c、Q3 的计数命令：等待臂 80 栏全部「不含」、T_clear 为 0、窗口末不留；阳性对照 16 栏全含（F1 没打中）；不等待臂 24 栏含 | 不等待臂在 5060 Ti、8 GiB 没有读数（两格作废），不影响等待臂这一半；并存的长借据一组 F8 的两种读法交主 agent。剩下的量翻不了面 |
| G2-乙-等待，同一设备几条线程共用 | 已够判 | 第二段 Q1 的 `awk` 区间命令：wait 306–691 毫秒、wait_own 301–639 毫秒，16 栏「等得比活长」，0 栏等不回来 | 无 |
| G2-乙-读数，同一设备几条线程共用 | 已够判 | 第二段 `uniq -c` 计数命令：wait 与 wait_own 各 8 栏「不含」、T_clear 为 0、窗口末不留；poll 的 2048 MiB 4 栏含、0.7 到 1.15 秒后还 | 无 |
| 在飞·轻那一档（上一段岔路表里还差） | 够判后未跑 | 登记第九节 K9 在登记自己的数下必红（模型卡自检 `simulated_light_trips_k9_at_the_registered_numbers`，`research/results/e174-lease-drop-wait-memory-return-checks-2026-10-08.out` 里 `id=simulated_light_trips_k9_at_the_registered_numbers` 那一行）；修订 1 补的在飞·中在两段里 `in_flight_cells_void_for_work_not_in_flight=0`，没有一格因「活没在飞」作废 | 登记 F6 预计在飞·轻凑不够有效轮；G2-乙 的四行已经够判，剩下的量翻不了面，续不续由主 agent 定 |
| G2 甲、丙 | 不在这一次实验里 | 登记第一节：「甲、丙两格不在这一次实验里」 | 甲要真跑产品路径数少借次数，丙要在模型上跑补法并被攻一轮，都不是这个装置能量的 |

```

**出处 `.claude/kb/experiments/174-借据丢掉时等待与显存回还.md:378-390`（整段抄，未转述）**

```markdown
### 它答不了的

装置 `e174_lease_drop_wait_memory_return.rs` 与变异表 `e174_lease_drop_wait_memory_return.tsv` 在 2026-11-12 改过（卡改成只开服务给的：环境里有 `GPU_CARDS_BY_PRIORITY` 照它，没有而有领卡通道时到开设备前一刻经 `task-server/task lease take` 领、子进程退出时还，从给这一台的卡里取头一张与这一格同型号的，不再按写死的总线号开；轮键里「驱动与卡那一行」按型号取；文件头加了 `task-server-gpu-arguments` 一行；变异表加了 M35 至 M39 五条），之后没有正式跑；要数的时候等用户说跑。

- 两段只在登记的取样点上：S 五档（第二段只有 64 MiB 与 2 GiB）、在飞·中与在飞·重（第二段只有在飞·中）、借据连做 3 笔、窗口 2 秒（长窗 30 秒）。长窗那 4 栏只在 64 MiB、2 GiB 与在飞·中上做，没有更大的 S。
- 5060 Ti 上 8 GiB 的不等待臂两格作废：没有这一格的第一份读数，不等待臂在 8 GiB 上是否含、是否还，只有 5090 的 8192 MiB 那 4 栏有读数（含，`not_absent_within_window`）。Out of Memory 发生在哪一笔借据没有记下来，页上不下原因的结论。
- 第二段的线程 B 只提交 400 毫秒的活、不建缓冲；线程 B 的活更短、更长、或也建缓冲时的等待时长与读数没量。wait 与 wait_own 的 5 轮值没有按轮配对比过，页上只报了区间。
- 不等待臂在两段里表现不同（第一段 2048 MiB 的在飞格 2 秒里一直不还，第二段 0.7 到 1.15 秒后还回去），差别来自什么没有量。
- 没量的旋钮照登记：`MemoryHints` 只取 `Performance`，后端只取 Vulkan，驱动只取 595.84，一个进程里借据只连做 3 笔。
- 登记第四节自己推出的两条（gpu-allocator 的块大小、"被还在飞的提交用着的缓冲要等下一次 maintain"）是假设，装置没有按它们写任何门槛。
- B5（`-q` 与 `--query-compute-apps` 逐 MiB 相等）在两段里没有让任何子进程作废（两份 `run` 的 verdict 都是 `children_void=0`），本机驱动 595.84 把 Vulkan 进程列进了 compute-apps。
- 什么现象会推翻页上的读数：同一装置、同一驱动重跑时等待臂任一栏第一份读数含着缓冲，或不等待臂在 S 不小于 512 MiB、活在飞的格子里第一份读数不含；换驱动版本、换 `MemoryHints`、换后端，这些读数都不保证。

```

**出处 `.claude/kb/experiments/174-借据丢掉时等待与显存回还.md:391-396`（整段抄，未转述）**

```markdown
### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D24（后台重活能不能卸给 GPU） 已定项 2 | 备料 | 2026-11-19 不受影响：只改复跑一节开头注释里任务服务分卡、清场的旧说法（现在卡与清场归 GPU 服务），结论、数与产物不变；此前 2026-11-14 不受影响：装置改了取哪张卡的办法（只开服务给的卡）并给变异表加了五条，之后没有正式跑，页里的结论、数与产物不变；该分项只备料，分项正文没点名这次改的东西；此前 2026-11-05 不受影响：只改用法与复跑说明（一个门禁、两种跑法；验证包改名），结论、数与产物不变；此前 2026-11-03 不受影响：只改复跑写法，结论、数与产物都不变；此前 2026-11-01 不受影响：只改复跑起法；此前 2026-10-23 不受影响：只改了产物名的写法，结论、数与产物不变；此前 2026-10-21 不受影响：只加了三语切换行；此前 2026-10-18 不受影响：单测与变异一节补点名 10-15 续跑键样本与整表变异的产物，结论、数不变 |

```

**出处 `.claude/kb/experiments/184-常规层与大负载层的代价计数.md:1-23`（整段抄，未转述）**

```markdown
## E184 常规层与大负载层的代价计数 —— 部分已跑（第一段「负载规模与重叠」已跑并在新段形上整份重跑过一次，G1 够判；第二到四段未跑）

**中文** · [English](184-cost-counts-for-the-regular-and-large-workload-tiers.en.md) · [日本語](184-通常層と大規模負荷層のコスト計数.ja.md)

<!-- doc-lint:not-numbers J2 J3 G1 G2 G3 G4 F1 F2 F3 F4 F5 F6 F7 X1 X2 X3 X4 X5 X6 X7 T0 T2 T3 T4 T5 T7 A1 A3 A4 A5 A8 A9 A10 A11 A12 B1 B2 B3 B4 V1 V2 V3 V4 V5 V6 V7 V9 V10 V11 V12 V13 V14 V15 V16 V17 V18 V19 V20 V21 V22 V23 V24 V25 V26 V27 R4 FR1 FR2 FR3 XR1 XR2 XR3 XR4 XR5 -->

跑前登记 `research/prompts/e184-preregistration.md`（第十二节修订十条，修订 6 至 10 写在装置单测之后、产物之前）。回答问题单 `research/prompts/m3-close-c409-cost-questions.md` 的四行（「负载规模与重叠」：问题单写的是「各候选负载有多大、和冒烟层重叠多少」，登记里分规模、态数区间重叠、共用块键、长期库同键块四项；J2 冷吞吐与库增量；J3 块键作废面；「两种落法的固定开销」。对应岔路单 `research/prompts/m3-close-c409-r1-forks.md` 的 G1 至 G3；G4 登记不罩），分四段，第一段只答「负载规模与重叠」（对应 G1）：D13（验证路线） 已定项 9 层表里的常规层，该收哪些负载、这些负载的态数区间与冒烟层已登记的流重不重叠、块键共用多少。被测条款还有 D13（验证路线） 已定项 17 的块键（块键只由内容算出）与 D13（验证路线） 已定项 4 的切段口径（装置照它切段）。C409（常规层、大负载层在门禁里没有阶段） 是这一件活的欠账。

装置是两份入库装置：`crates/singlefs-implementation-experiments/src/bin/e184_regular_layer_cost_counts/main.rs`（全部子命令与内联单测）与 `crates/singlefs-implementation-experiments/src/bin/e184_regular_layer_cost_counts_write_dump.rs`（只录写流、写转储）。块键一律调 crates 的 `crash_points_of_the_steps`、`plan_crash_points`、`PlannedCrashPoint::block_key` 算，不另写一份。第一段由长跑脚本 `scripts/experiment-apparatus/e184-segment-one-long-run.sh` 分 18 件跑（`--build-here`，产物头记两份二进制的 sha256；现跑 17 件、沿用 0，第 18 件 `library` 的登记条件没成立，产物里写 `state=not_run`）。负载两族：臂 R（固定种子随机历史，种子基起连续 20 个种子）与臂 F（覆盖写链，每次覆盖写的数据单元数 u 取 1、4、16），基线是冒烟层四条重录流（登记里叫 R4）。

**现状**：

- 第 2 次重跑（2026-10-24，重跑登记 `research/prompts/e184-r2-prereg.md`）：第一段整份产物在新段形（新池 12 段、建快照 16 段、多记录 20 段）上用今天的主工作区重跑，产物 `research/results/e184-regular-layer-cost-counts-segment-one-2026-10-24.out`（959 行，末行 `E7RESULT name=done segments=18 emitted=959`，`sha256sum` 为 `5c5749effbda6daed7d8f329c6b2fc4fecd71074ff2dff794a996a0f3cfdac0c`），逐格比输出 `research/results/e184-regular-layer-cost-counts-segment-one-r2-verdict-comparison-2026-10-24.out`（83 行）。问题单两行都够判：锚 A1、锚 2、A5、锚 7 与 X3 的新数（16842785；16777243 与 43 写；16777268 与 54；1678377220 与 294；16842815）全部「新数定」，10 格现算与现录 `same=true`，`selfcheck` 件退 0；窄读 40 格、宽读 62 格、4 条推出的条款换词 0 格，记「不翻」。区间 S 变了（下界 16777243、上界 5722079403，多两条 2026-10-21 进仓的冒烟流），各范围的占比在动、判定词没动，R 400 步档 inside 占比 0.100000 正好在门槛端点上；B1 现录 1358954642，F6 第一条照旧触发。读数与条款逐条见「第 2 次重跑」一节。
- 第 1 次跑的产物（2026-10-19，取号写一轮那一版的树；这一项起到「单测 82 个」那一项，读数都是这份产物的）`research/results/e184-regular-layer-cost-counts-segment-one-2026-10-19.out`：950 行，末行 `E7RESULT name=done segments=18 emitted=950`，`sha256sum` 为 `7eacaffaddd44d34bf4a67afe29214ccbbf7ffbd63663d61ef7489124e858c06`（键名、标签、流名已换成不带缩写的名字，数没变；换名之前是 `0704bc652549aa54477335b893627ada8ad04bedd76972a0a76ac14db4f39867`）。127 条流：随机历史 120 条（BROAD 比重、两块 4 GiB 盘，50、100、200、400 步各 20 条；盘宽 240 槽 100 步 20 条；带整理比重 100 步 20 条，其中 BROAD 四档 80 条是主取样、后两组是 8.2 的方向相反取样点）、覆盖写链 3 条（u = 1、4、16）、冒烟层重录 4 条。随机历史 120 条的 `ending` 全是 `Completed`。
- 「态数区间重叠」（态数区间与冒烟层重不重叠，门槛 inside ≥ 0.5 记重叠、≤ 0.1 记分开）：随机历史按步数档走 50 步 overlap（0.583333）、100 步 overlap（0.500000，正好在门槛上）、200 步 partial（0.200000）、400 步 separate（0.000000）；四档合并 partial（0.291667）。覆盖写链三档都是 separate（态数全在区间上界之上）。区间 S 取 14 条钉过的冒烟流，上界 2438004868，下界 16777240。
- 「共用块键」（与冒烟层四条重录流共用的块键占比，门槛族最大 ≤ 0.05 记独立、族中位 ≥ 0.5 记换名）：随机历史六组和覆盖写链 u = 1、4 全是 independent；共用块占比最大 0.002761（覆盖写链 u = 1），随机历史最大 0.001985（50 步）。随机历史 120 条里 8 条（200 步 3 条、400 步 5 条）和覆盖写链 u = 16 块数上界超过 2^26，不排计划，「共用块键」对它们取不出（`not_planned`），不写 0。
- 8.2 几何敏感性：随机历史 100 步的两个方向相反取样点（盘宽 240 槽、带整理比重）「态数区间重叠」都是 overlap，与主取样相同；步数那一行（合并的 72 条对 50 步档）partial 对 overlap，记「不稳定」。覆盖写链「态数区间重叠」在 u = 1、4、16 上相同（separate）；「共用块键」在 u = 16 上取不出。
- 条款这一格：锚 B1（D13（验证路线） 已定项 9 射程写的多记录流 12230590578 个状态）装置重录得 1358954634（第一段产物的读数；取号写两轮之后 `selfcheck` 现录 1358954642，仍是 `match=false`），是登记第四节预计的结果，触发失败条款 F6，不作废（第 2 次重跑产物第 77 行同：现录 1358954642，`match=false`）。
- 阳性对照 174 条 `name=control` 全部 `passed=true`；随机历史每档种子 0、1 各录两遍写表摘要 12 对全部相同（X2 不触发）；停机检查 28 条全过；`name=anchor` 10 行里 9 行 `match=true`，B1 一行 `match=false`（`selfcheck_done` 的 `anchors_mismatching` 不数 B1，`b1_matches=false` 另报）。
- 单测 82 个（主 agent 在 release 下跑的读数：82 passed、0 failed；`grep -c '#\[test\]'` 对主装置得 61、对转储 bin 得 21，转储 bin 经 `#[path]` 引进主装置，61 加 21 等于 82）；变异 26 行追加进 `crates/mutations.tsv`，三个数待提交时门禁 crates-mutation-replay 跑出，这一页没有变异结果。上游写序变化之后只修了钉值，这 82 个单测在改过的装置上还没重跑。
- 上游写序变化（取号写两轮：mkfs 之后多一个 2 写的系统配置槽段）之后只修钉值，结论不变：主装置照四份冒烟测试文件现钉的数重抄测试钉值与段形（新池新建文件流 两态 16777243、全量 16777268、甲二 54、12 段、43 写；固定脚本流 1678377095、1678377220、294、80 段；多记录流 1358954607、1358954642、20 段、117 写；建快照流 16842785、16842815、16 段、64 写；中项全量 167837792），锚 A1、锚 2 照新段形现算（16842785、16777243、43 写），`selfcheck` 重出 `research/results/e184-regular-layer-cost-counts-selfcheck-2026-10-23.out`（40 行，退 0）：停机检查 28 条全过，`name=anchor` 10 行里 9 行 `match=true`，B1 一行 `match=false`（登记预计）。复跑登记改指这一份；2026-10-19 那份 `research/results/e184-regular-layer-cost-counts-selfcheck-2026-10-19.out` 留着，与新的一份差 18 行（第 5、7、12、13、17、18、22、23、27 至 31、34 至 38 行：照抄段在测试文件里的起止行号、各流两态与全量、块数、A1 与锚 2 及写数锚、四条全量锚与 B1 的现录值）。第一段整份产物 `research/results/e184-regular-layer-cost-counts-segment-one-2026-10-19.out` 里的态数是取号写一轮那一版的读数；2026-10-24 在新段形上整份重跑过（「第 2 次重跑」一节），判定词逐格同这一份。X3 写死的 16842807 与 J2 小项那一格的本地常量（装置 `:2913`）没改，登记的新值是 16842815，第三段开跑前要改。
- 第二段（J3 作废面）、第三段（J2 冷吞吐）、第四段（两种落法的固定开销）没写外层脚本、没跑；J2、J3 与「两种落法的固定开销」三行仍开着，G2、G3 没有读数。G4 登记不罩。

```

**出处 `.claude/kb/experiments/184-常规层与大负载层的代价计数.md:24-33`（整段抄，未转述）**

```markdown
### 做了什么

1. 停机条款「随机历史到 `CrashFlow` 的转换 crates 里没有」（`grep -rn "CrashFlow" crates/singlefs-verification/src/crash_injection.rs crates/singlefs-verification/src` 零命中）由长跑脚本的 `repo-reads` 件现读，产物第 3 行 `result=pass hits=0 files_read=18`。
2. 长跑脚本 18 件按序做完，每件装置的原输出整段并进产物，前面加一行 `name=segment`（件名、跑了几次、累计秒数、行数、二进制 sha256）；各件累计秒数加起来是 5462 秒（求和命令在「复跑」一节）。
3. 转储由确定性的种子重现：随机历史六档各取种子 0、1 录两遍比写表摘要（产物 12 条 `name=determinism`，全部 `equal=true`）；120 条历史的转储在写出时都核过「重建的基镜像等于把录制流 mkfs 那几步叠到空盘上的基镜像」（`base_matches_the_stream=true`，六个 `dump_done` 的 `base_mismatches=0`）。
4. 覆盖写链 u = 4、16 走 `publish_sequential_write`（顺序写，每次覆盖写是一次发布里的 u 个一单元事务），u = 1 走 `publish_overwrite`：crates 里没有单事务的多数据单元覆盖写，依据与退路写在登记修订 6；覆盖写次数 K 由装置数出来：u = 1、4、16 分别 30、23、11 次（产物第 71、73、75 行），写数 1003、1042、1050。
5. 覆盖写链 u = 16 的全量态数 2954361402816462969，块数上界远超 2^26，登记修订 10 规定不排计划；这一条链上三个要全量计划的阳性对照（基镜像改一扇区、名字不进键、两种展开）没跑，转储往返对照只比基镜像（`keys_equal=not_planned`，产物第 76 行 `kind=dump_round_trip_direct_base`）。
6. 2026-10-23 只修钉值：取号写两轮落地之后装置单测 8 条红在钉值上（`selfcheck` 退 3：停机检查 6 条 fail、锚 4 行对不上）。在仓副本（主工作区当天的 `crates/`，不带 `target/`）里先用原装置跑 `selfcheck` 现量，读数与调查员在另一份副本里量的逐行相同；四份冒烟测试文件里已是两轮的数，装置照它们重抄 `PINNED_BY_THE_TESTS`、三个段形常量、中项全量常量，A1、锚 2 与新池写数锚照新段形用 `python3 -c` 现算（`s=[2,2,2,1,2,2,1,2,24,2,1,2,16,2,1,2]` 得 16 段、64 写、16842785；前 12 段得 43 写、16777243；多记录流 20 段得 117 写、1358954607），内联单测里跟着钉值走的 12 处一起改；release 编出装置跑 `selfcheck`，退 0，产物 `research/results/e184-regular-layer-cost-counts-selfcheck-2026-10-23.out`；复跑登记改指它。
7. 2026-10-24 第 2 次重跑：核登记第七节的新数（两条现算路与装置现文逐个相同），在今天的主工作区上用长跑脚本把第一段 18 件从全新的工作目录整份重跑（现跑 17、沿用 0；第一趟在 `repo-reads` 件退 3、没出产物，登记第十二节修订 2 补一类钉值认法后换新工作目录重起），再用 `scripts/experiment-apparatus/e184_r2_verdict_compare.py` 把重跑产物与 2026-10-19 产物逐格比判定词（窄读 40 格、宽读 62 格、4 条推出的条款）；逐格比脚本的单测 16 项本段重跑全绿。读数与条款逐条见「第 2 次重跑」一节。

```

**出处 `.claude/kb/experiments/184-常规层与大负载层的代价计数.md:34-88`（整段抄，未转述）**

````markdown
### 复跑

```bash
# 仓根；整份产物是长跑（分 18 件，续跑沿用做完的件，起法在长跑脚本文件头），带挂钟字段，不登记逐字节复跑。
# 复跑表登记两行：确定性的那一片 selfcheck（停机 T1 至 T4 与锚点，产物里这一件 4 秒，exact），与整份第一段（driver_e184_segment_one：长跑脚本不带参数、装置用这棵树里编的，timing 比结构）；入库装置，这两行在门禁 research-build-and-replay 的 experiment-replay 格里跑
bash scripts/experiment-apparatus/replay.sh E184

# 整份产物的起法（预计超过 2 小时，长活声明在脚本文件头；这一次各件累计 5462 秒，求和命令在后一段）
SINGLEFS_RUN_SCOPE='种类=whole; 跑什么=常规层与大负载层代价计数第一段的整份产物; 范围=18 件; 为什么=…' task-server/task submit -- bash scripts/experiment-apparatus/e184-segment-one-long-run.sh --build-here

# 各件累计秒数相加
grep 'name=segment segment=' research/results/e184-regular-layer-cost-counts-segment-one-2026-10-19.out | grep -o 'seconds_total=[0-9]*' | cut -d= -f2 | paste -sd+ | bc

# 第 2 次重跑（登记 research/prompts/e184-r2-prereg.md 5.3）：起法同上，只多一个全新的工作目录；产物名脚本自己取，当天已有的加 -r2
SINGLEFS_RUN_SCOPE='种类=whole; 跑什么=常规层与大负载层代价计数第一段的整份产物，第 2 次重跑; 范围=18 件; 为什么=…' task-server/task submit -- env E184_WORK_DIRECTORY="$HOME/.local/share/singlefs/work/e184-segment-one-r2" bash scripts/experiment-apparatus/e184-segment-one-long-run.sh --build-here

# 逐格比（只读两份产物，不起装置；单测先跑）
python3 scripts/experiment-apparatus/e184_r2_verdict_compare_test.py
python3 scripts/experiment-apparatus/e184_r2_verdict_compare.py research/results/e184-regular-layer-cost-counts-segment-one-2026-10-19.out research/results/e184-regular-layer-cost-counts-segment-one-2026-10-24.out
```

复跑表登记行在 `scripts/experiment-apparatus/replay.sh`：E184（常规层与大负载层的代价计数） 名下的 `@driver_e184||e184-regular-layer-cost-counts-selfcheck-2026-10-23.out|exact` 与 `@driver_e184_segment_one||e184-regular-layer-cost-counts-segment-one-2026-10-24.out|timing` 两行。登记的留存产物 `research/results/e184-regular-layer-cost-counts-selfcheck-2026-10-23.out` 是只修钉值之后的装置跑 `selfcheck` 的 39 行加一行 `E7RESULT name=done emitted=40`（驱动照先例 E175（产品路径上 inode 号水位退回与重发号） 补收尾行）；它与整份产物第 31 至 69 行（`selfcheck` 件在取号写一轮那一版上的原输出）差 18 行，`diff <(sed -n 31,69p <整份产物>) <(head -n 39 <留存产物>) | grep -c '^>'` 输出 `18`；原来登记的 `research/results/e184-regular-layer-cost-counts-selfcheck-2026-10-19.out` 与那 39 行逐字相同（同一条 `diff` 无输出）。其余各件（`keys` 带每条流的 `keys_seconds`，`record-fixed-chains` 带 `seconds`，`progress` 行带用时）含挂钟，两次跑本来就不同，不登记；长段跑法见脚本文件头。

产物里每一类结果行的条数（`awk '{for(i=1;i<=NF;i++) if($i ~ /^name=/){print $i; break}}' research/results/e184-regular-layer-cost-counts-segment-one-2026-10-19.out | sort | uniq -c` 的原样输出）：

```text
     10 name=anchor
      7 name=chain_recorded
      1 name=chains_done
    174 name=control
     12 name=determinism
      1 name=done
    120 name=dump
      6 name=dump_done
     13 name=family_summary
    127 name=flow
      7 name=keys_done
     11 name=overlap
     10 name=pinned_smoke
      1 name=pinned_smoke_done
     15 name=pinned_smoke_missing
    247 name=progress
     18 name=segment
      1 name=segment_one_begin
      1 name=selfcheck_done
    114 name=share
      1 name=share_done
     11 name=share_group
      4 name=share_ignored
      9 name=share_not_planned
     29 name=stop_check
```

登记的「臂 × 历史 × 几何」数一遍：127 条 `name=flow` = 覆盖写链 3 + 冒烟 4 + 随机历史 6 组 × 20；120 条 `name=dump` = 随机历史 6 组 × 20；`name=family_summary` 13 条 = 3（覆盖写链各 u）+ 4（冒烟各流）+ 6（随机历史各组）；`name=share` 114 条 = 123 条候选 − 9 条 `share_not_planned`（8 条随机历史与覆盖写链 u = 16）；`name=control` 174 条里 `self_share` 118 条 = 114 条排了计划的候选 + 4 条冒烟；`base_sector_changed`、`name_not_in_key`、`two_expansions` 各 12 条 = 随机历史每档种子 0 共 6 + 覆盖写链 u = 1、4 共 2 + 冒烟 4（覆盖写链 u = 16 因没排全量计划缺这三类，登记修订 10 第 2 条）；`dump_round_trip` 13 条 = 随机历史每档种子 0 共 6 + 覆盖写链与冒烟 7；`dump_round_trip_direct_base` 7 条。没有缺行、缺字段。

````

**出处 `.claude/kb/experiments/184-常规层与大负载层的代价计数.md:89-271`（整段抄，未转述）**

````markdown
### 第 2 次重跑（2026-10-24）：新段形上第一段整份重跑

重跑登记 `research/prompts/e184-r2-prereg.md`（读法写死为宽读；第十二节修订两条，都在产物之前：逐格比脚本四处读法写死、长跑脚本补一类钉值认法）。回答问题单 `research/prompts/m3-close-e184-r2-questions.md` 两行：第一行，登记第七节锚 A1、锚 2、A5、锚 7 与第十一节 X3 在新段形（新池 12 段、建快照 16 段、多记录 20 段）下各取什么；第二行，第一段整份产物在新段形上判定词翻不翻。装置两份源文件原样（`sha256sum` 得 `8f529e12…`、`320822c4…`，与登记文件头相同），长跑脚本是修订 2 之后的 `c1f9176e…`；这一趟新写的只有逐格比脚本 `scripts/experiment-apparatus/e184_r2_verdict_compare.py`、它的单测 `e184_r2_verdict_compare_test.py` 与变异表 `e184_r2_verdict_compare.mutations.tsv`。树是当日的主工作区：2026-10-19 以来 `crates/` 里别的改动同样进了这一趟，所以判定词翻不翻只报「今天的树上翻没翻」，不归因到段形（登记 1.1）。

**产物与完整性**：

- 重跑产物 `research/results/e184-regular-layer-cost-counts-segment-one-2026-10-24.out`：959 行，`sha256sum` 为 `5c5749effbda6daed7d8f329c6b2fc4fecd71074ff2dff794a996a0f3cfdac0c`，末行 `E7RESULT name=done segments=18 emitted=959`，`emitted` 等于行数。工作目录全新，18 件现跑 17、沿用 0；`library` 件写 `state=not_run reason=no_family_share_verdict_partial`（第 958 行，登记算完整）；各件累计 8403 秒（改前 5462 秒），其中 `share` 件 6273 秒。同一天先起过一趟，在 `repo-reads` 件退 3、没出任何产物：冒烟层新进的 `large_file_overwritten_across_lower_leaves_stream` 钉的是展开段的另一个量，长跑脚本两种钉法都认不出，登记第十二节修订 2 补了「段形在、钉法认不出的流记缺钉值」这一类认法；换新工作目录重起的这一趟才是这份产物。长跑日志在仓外，不入库。
- 逐格比输出 `research/results/e184-regular-layer-cost-counts-segment-one-r2-verdict-comparison-2026-10-24.out`：83 行，`sha256sum` 为 `a3f4d71226fed5c9d092bf753d5f2d6bd653647b96d2df369bf221b37bea2aaf`，末行 `E7RESULT name=done emitted=83`。本段把同一条命令又跑了一遍，输出与入库的这份 `cmp` 逐字节相同。逐格比脚本单测 16 项全绿（末行「自证 16 项，红 0 项」，本段又跑了一遍）；变异表 9 行，写脚本那一段（登记 5.4 分段表第 1 步）交回的读数是抓到 9、无效 0、没红 0（`mutate.sh` 只管 Rust，照 `e185_r1_compare.mutations.tsv` 的先例在副本上逐行改坏跑单测），本段没有重跑变异。
- 每类结果行的条数与臂 × 历史 × 几何（一条命令数出来；与改前产物相比只有 `name=pinned_smoke` 10 变 12、`name=pinned_smoke_missing` 15 变 22 不同，其余各类条数相同，没有缺行、缺字段）：

```bash
P=research/results/e184-regular-layer-cost-counts-segment-one-2026-10-24.out
awk '{for(i=1;i<=NF;i++) if($i ~ /^name=/){print $i; break}}' $P | sort | uniq -c
grep 'name=flow ' $P | grep -o 'family=[a-z_]*' | sort | uniq -c
grep 'name=chain_recorded' $P | grep -o 'label=overwrite_chain-[^ ]* \|overwrites=[0-9]* writes=[0-9]*' | paste - -
```

```text
     10 name=anchor
      7 name=chain_recorded
      1 name=chains_done
    174 name=control
     12 name=determinism
      1 name=done
    120 name=dump
      6 name=dump_done
     13 name=family_summary
    127 name=flow
      7 name=keys_done
     11 name=overlap
     12 name=pinned_smoke
      1 name=pinned_smoke_done
     22 name=pinned_smoke_missing
    247 name=progress
     18 name=segment
      1 name=segment_one_begin
      1 name=selfcheck_done
    114 name=share
      1 name=share_done
     11 name=share_group
      4 name=share_ignored
      9 name=share_not_planned
     29 name=stop_check
      3 family=overwrite_chain
    120 family=random_history
      4 family=smoke
label=overwrite_chain-01_units_per_overwrite 	overwrites=30 writes=1005
label=overwrite_chain-04_units_per_overwrite 	overwrites=22 writes=1001
label=overwrite_chain-16_units_per_overwrite 	overwrites=11 writes=1052
```

覆盖写链 u = 4 的覆盖写次数由 23 变 22（写数 1042 变 1001），u = 1、16 仍是 30、11；覆盖写次数由装置数出来（「做了什么」第 4 条），不是改过的参数；u = 4 的数跟着新段形变了。

**问题单第一行：锚 A1、锚 2、A5、锚 7 与 X3 在新段形下各取什么**（旧数抄自原登记第七节、第十一节，原登记一个字没改；新数是登记第七节 7.2 现算的，本段把那份现算脚本又跑了一遍，`sha256sum` 与登记所记相同，输出与登记所贴逐字相同。整式路：1 加各段态数之和，系统配置槽 2 写段取三态；增量路：旧数加新段形多出来的系统配置槽段的增量）：

| 锚 | 旧数 → 新数 | 现算两条路 | 重跑产物（`expected=` / `actual=` / `match=`） | 逐格比 |
|---|---|---|---|---|
| A1 建快照流两态 `snapshot_creation_two_state` | 16842782 → 16842785（15 段 62 写 → 16 段 64 写） | 整式 16842785；增量 16842782 加 3 | 第 68 行 16842785 / 16842785 / true | 第 67 行 `same=true` |
| 锚 2 新池两态 `new_pool_two_state` | 16777240 → 16777243 | 整式 16777243；增量加 3 | 第 69 行 16777243 / 16777243 / true | 第 68 行 `same=true` |
| 锚 2 新池写数 `new_pool_writes` | 41 → 43 | 整式 2 × 8 + 3 + 24 = 43；增量加 2 | 第 70 行 43 / 43 / true | 第 69 行 `same=true` |
| A5 新池全量 `full_states_smoke-new_pool_file_creation_stream` | 16777260 → 16777268（甲二 46 → 54） | 整式 16777268；增量加 8 | 第 73 行 16777268 / 16777268 / true；甲二 54 在 `name=flow` 第 120 行 `quick_tier=54` | 第 72 行 `same=true` |
| A5 固定脚本全量 `full_states_smoke-overwrite_fixed_script_stream` | 1678377204 → 1678377220（两态 1678377089 → 1678377095，甲二 278 → 294，段数 78 → 80） | 整式路现算不出（80 段的角色测试文件与装置都没钉）；增量加 16；另一条 1678377095 加 25 × 5 = 1678377220 | 第 74 行 1678377220 / 1678377220 / true；两态、甲二、段数在 `name=flow` 第 125 行（`two_state=1678377095`、`quick_tier=294`、`segments=80`） | 第 73 行 `same=true` |
| A5 建快照流全量 `full_states_smoke-snapshot_creation_stream` | 16842807 → 16842815 | 整式 1 + 6 × 8 + 4 × 3 + 4 + (2^24 − 1) + (2^16 − 1) = 16842815；增量加 8 | 第 76 行 16842815 / 16842815 / true | 第 75 行 `same=true` |
| 锚 6 多记录流全量（附带）`full_states_smoke-multi_record_publish_stream` | 1358954634 → 1358954642（两态 1358954604 → 1358954607，19 段 115 写 → 20 段 117 写） | 整式（角色是推的）1358954642；增量加 8 | 第 75 行 1358954642 / 1358954642 / true；两态与段数在 `name=flow` 第 115 行 | 第 74 行 `same=true` |
| 锚 7 的后半与 X3 小项 `states` | 16842807 → 16842815 | 同 A5 建快照流全量，两条路相同 | 后半就是 A5 建快照流全量那一格（第 76 行）；锚 7 前半要 J2 冷跑，这一趟不跑 | 不适用 |
| B1 条款 `multi_record_full_named_by_the_clause` | 条款 12230590578 不变；现录 1358954634 → 1358954642 | 整式 1358954642 | 第 77 行 12230590578 / 1358954642 / false（登记预计内） | 第 76 行 `same=true`（只比 `actual=`，登记第十二节修订 1 第 2 条） |

- 自检退出码：`selfcheck` 件第 78 行 `E7RESULT name=selfcheck_done stop_checks=28 failed=0 anchors=10 anchors_mismatching=0 clause_anchor_matches=false emitted=39`，记「退 0」。`selfcheck` 件的 39 行（产物第 40 至 78 行）与 `research/results/e184-regular-layer-cost-counts-selfcheck-2026-10-23.out` 前 39 行逐字相同（命令见本节末「复跑登记」）。
- X3 的新值：小项 `states` 不等于 16842815 即作废（原登记写 16842807），其余三条（`reused`、包含自检、单调）不变。装置常量 `SNAPSHOT_CREATION_FULL_STATES_PINNED_BY_THE_TEST`（`crates/singlefs-implementation-experiments/src/bin/e184_regular_layer_cost_counts/compose_command.rs:469`）这一趟没改，仍是 16842807，用在 `:3035` 的 `voided_cold_runs`；只有第三段的 `compose` 用到它，所以第三段开跑之前要改成 16842815，连同那两条手造读数的用例（`cold_run_that_is_not_cold_or_not_monotone_or_not_the_pinned_item_is_void`、`the_compose_lines_give_the_pinned_cells_for_a_hand_made_load`），谁改缺主 agent 定。
- 候选甲（照新数改原登记，修订记录写改了什么、依据、时点）与乙（原数不动，另立重跑登记只换数）是记法的选择，不由读数翻面；重跑登记第七节那张表就是乙，同时备好了甲要追加进原登记第十二节的文字；主 agent 定不追（乙已成）。

**问题单第二行：第一段判定词翻不翻**（宽读；逐格比脚本只读两份产物）：

| 读法 | 格 | 格数 | 换词的格数 | 判定 | 逐格比行 |
|---|---|---|---|---|---|
| 窄读 | `stop_check` 29 格（`result=`，全部 pass → pass）、`anchor` 10 格（`match=`，9 格 true、B1 一格 false，改前改后相同）、`selfcheck_done` 1 格（`clause_anchor_matches=` false → false） | 40 | 0 | not_flipped | 第 81 行 |
| 宽读 | 窄读 40 格加 `overlap` 11 格、`share_group` 11 格 | 62 | 0 | not_flipped | 第 82 行 |
| 推出的条款 | F1、F2、F3、F6 第二条，改前改后都是 false | 4 | 0 | 同 | 第 63 至 66 行 |

`overlap` 与 `share_group` 22 格逐范围（判定词改前改后全同；占比抄自两份产物同名行，改前的行号在 2026-10-19 产物的各读数节里，改后的行号在最后一列）：

| 范围 | 区间重叠 inside 占比 改前 → 改后 | 乙判定 | 共用块占比最大 改前 → 改后 | 丙判定 | 改后行（`share_group` / `overlap`） |
|---|---|---|---|---|---|
| R 主取样合并（四档共 80 条流） | 0.291667 → 0.438356 | partial | 0.001985 → 0.002261 | independent | 937 / 938 |
| R 50 步 | 0.583333 → 0.923077 | overlap | 0.001985 → 0.002261 | independent | 945 / 946 |
| R 100 步（主取样） | 0.500000 → 0.550000 | overlap | 0.000864 → 0.000986 | independent | 947 / 948 |
| R 100 步 盘宽 240 槽 | 0.894737 → 0.894737 | overlap | 0.000000 → 0.000000 | independent | 953 / 954 |
| R 100 步 带整理比重 | 0.526316 → 0.789474 | overlap | 0.000664 → 0.000757 | independent | 955 / 956 |
| R 200 步 | 0.200000 → 0.350000 | partial | 0.000548 → 0.000637 | independent | 949 / 950 |
| R 400 步 | 0.000000 → 0.100000 | separate | 0.000153 → 0.000174 | independent | 951 / 952 |
| F 主取样（u = 4） | 0.000000 → 0.000000 | separate | 0.000060 → 0.000061 | independent | 935 / 936 |
| F u = 1 | 0.000000 → 0.000000 | separate | 0.002761 → 0.002771 | independent | 939 / 940 |
| F u = 4 | 0.000000 → 0.000000 | separate | 0.000060 → 0.000061 | independent | 941 / 942 |
| F u = 16 | 0.000000 → 0.000000 | separate | 取不出 → 取不出 | not_planned | 943 / 944 |

- 占比在动，词没动：区间 S 变了。改前 S 下界 16777240、上界 2438004868、最长段区间 24 至 30（14 条钉过的冒烟流）；改后下界 16777243、上界 5722079403、最长段区间 24 至 32（产物第 936 行，`name=pinned_smoke` 12 条加四条重录流共 16 条）。多出来的两条是 `one_file_overwritten_across_two_writable_remounts_stream`（产物第 8 行，两态 5675286699）与 `several_inodes_overwritten_in_turn_stream`（第 10 行，两态 5722079403，定了新上界），两条文件 2026-10-21 才进仓（`git log --diff-filter=A` 查两份测试文件，提交 `0b926bd3f`），改前产物里没有。候选流自己的写数与态数也变了（`name=family_summary` 的写数中位：R 100 步 2261.5 → 2281.5，R 400 步 8580.5 → 8682.5），所以占比的变动混着 S 变宽与候选流变化，这一趟分不开。
- 离门槛的距离（登记的「离门槛的距离」量够判后不跑；这几句是读产物行的算术，推的）：R 100 步 11 / 20 = 0.55，门槛 0.5 含端点，少一条流在 S 里仍是 overlap（10 / 20），少两条才是 partial；R 400 步 2 / 20 = 0.1，正好在「≤ 0.1 记分开」的端点上，多一条流落进 S（3 / 20 = 0.15）就是 partial；R 主取样合并 32 / 73 = 0.438356，离 0.5 差 0.061644。
- F6 第二条（随机历史写数 ≥ 1000 的流里两态闭式不超过 S 上界的占比 ≥ 0.5）：合并范围 `at_most_smoke_interval_maximum_fraction=0.438356`（第 938 行），不触发，同改前；逐组读 50 步 0.923077、100 步 0.550000、盘宽 240 槽 1.000000、带整理比重 0.789474 四组 ≥ 0.5，200 步 0.350000、400 步 0.100000 不到，同改前的四组。
- 随机历史按步数档，区间重叠仍单调：50 步 overlap、100 步 overlap、200 步 partial、400 步 separate（inside 占比 0.923077、0.550000、0.350000、0.100000）。
- 共用块键六组随机历史与覆盖写链 u = 1、4 仍全是 independent，共用块占比最大 0.002261（R 50 步，第 945 行）；排不了计划的 9 条流同改前（200 步 3 条、400 步 5 条、覆盖写链 u = 16，`name=share_not_planned` 在第 698、847、848、857、872、883、884、885、892 行）。

**失败、作废、停机条款逐条**（登记第十、十一节；第 N 行指重跑产物，「比」指逐格比输出）：

| 条款 | 这一趟的读数 | 结论 |
|---|---|---|
| FR1 推的「不翻」被推翻 | 比第 81、82 行 `differing=0`、`verdict=not_flipped`；`grep -c 'same=false' <比>` 得 0 | 不触发 |
| FR2 B1 不再触发 F6 | 第 77 行 `multi_record_full_named_by_the_clause` `match=false`，第 78 行 `clause_anchor_matches=false` | 不触发（F6 第一条照旧触发，登记预计内） |
| FR3 F1、F2、F3、F6 第二条换边 | 比第 63 至 66 行四条 `same=true` | 不触发 |
| XR1 阳性对照没过 | 比第 77 行 `name=void_check family=control cells=174 failing=0`；产物 `passed=false` 0 条；17 件现跑件的 `name=segment` 都是 `state=done runs=1`；长跑退出码文件写 0 | 不触发 |
| XR2 随机历史确定性 | 比第 78 行 `family=determinism cells=12 failing=0`；12 条 `equal=true`（产物第 184、185、275、276、366、367、457、458、548、549、639、640 行） | 不触发 |
| XR3 转储重建基镜像 | 比第 79 行 `family=dump_done cells=6 failing=0`；六个 `dump_done` 都是 `base_mismatches=0`（第 186、277、368、459、550、641 行） | 不触发 |
| 冒烟层钉值 | 比第 80 行 `family=pinned_smoke cells=12 failing=0`；第 38 行 `pinned_smoke_done pinned=12 missing=22 mismatches=0 t6_hits=0` | 不触发 |
| XR4 产物不完整 | 末行 `name=done segments=18 emitted=959` 等于 `wc -l` 的 959；18 条 `name=segment` 里 17 条 `state=done`，`library` 是 `not_run`（登记算完整） | 不触发 |
| XR5 逐格比单测没过 | `python3 scripts/experiment-apparatus/e184_r2_verdict_compare_test.py` 末行「自证 16 项，红 0 项」 | 不触发 |
| X6、X7 | 没有写 0 的取不出值（9 条 `not_planned` 与 `library` 的 `not_run` 照原样报）；17 件现跑件都是 `runs=1`、没有重起，长跑日志里没有内存包装退 245 或 250 至 254 的字样 | 不触发 |
| S1 起跑前五样 | 本段现查：两份装置源文件的 sha256 与登记文件头相同；长跑脚本 `c1f9176e…` 是登记第十二节修订 2 记下的值；改前产物 `sha256sum` 得 `7eacaffa…`；停机项 6 在产物第 3 行 `result=pass hits=0 files_read=19`。工作目录全新与 `--list-segments` 的输出是主 agent 起跑时看的，本段没见到原样输出 | 本段能核的都成立；两样没见到 |
| S2 长跑退 3 或 4 | 产物里停机项 1 至 T4 的 29 格 `stop_check` 全是 `result=pass`（`grep -c 'result=fail'` 得 0），`selfcheck_done failed=0 anchors_mismatching=0` | 不触发 |
| S3 现算与现录不符 | 10 格 `anchor_recompute` 全是 `same=true`（比第 67 至 76 行） | 不触发 |
| S4 两份产物格对不齐 | 比第 81、82 行 `verdict=not_flipped`，不是 `keys_differ`；输出里没有 `absent`、`duplicate` 的格（`grep -c absent <比>` 与 `grep -c duplicate <比>` 都得 0），脚本退 0 | 不触发 |

两行够判：第一行，登记第七节的数全部「新数定」，10 格 `same=true`，自检退 0；第二行，XR1 至 XR5 都没触发，窄读与宽读都记「不翻」。登记的「离门槛的距离」标「够判后未跑」，原登记第二到四段这一趟不罩。

**判决行与没过的字段**：

```bash
C=research/results/e184-regular-layer-cost-counts-segment-one-r2-verdict-comparison-2026-10-24.out
P=research/results/e184-regular-layer-cost-counts-segment-one-2026-10-24.out
grep -n 'name=verdict' $C
for pat in 'name=verdict' 'passed=false' 'result=fail' 'equal=false'; do echo "$pat $(grep -c "$pat" $P)"; done
grep -cE 'failed=[1-9]|controls_failed=[1-9]|count_disagreements=[1-9]|base_mismatches=[1-9]|anchors_mismatching=[1-9]|mismatches=[1-9]' $P
grep -n 'match=false' $P
```

```text
81:E7RESULT name=verdict_comparison reading=narrow cells=40 differing=0 verdict=not_flipped
82:E7RESULT name=verdict_comparison reading=wide cells=62 differing=0 verdict=not_flipped
name=verdict 0
passed=false 0
result=fail 0
equal=false 0
0
77:E7RESULT name=anchor identifier=multi_record_full_named_by_the_clause expected=12230590578 actual=1358954642 match=false
```

逐格比输出的判决行 2 条（`name=verdict_comparison`），取值都是 `not_flipped`，`differing=0`。重跑产物不打 `name=verdict` 行；没有 `result=fail`、`passed=false`、`equal=false`，违例、不匹配、失败类的计数都是 0。没过或取不出的字段逐个点名：

| 字段 | 位置 | 取值 | 为什么是这个值 | 登记预期内吗 |
|---|---|---|---|---|
| `match=false`（B1）与 `clause_anchor_matches=false` | 产物第 77、78 行；比第 39、40、76 行（`before=false after=false same=true`） | false | D13（验证路线） 已定项 9 射程写的 12230590578，装置重录多记录流得 1358954642（条款是改段形之前的数） | 预计内（登记第四节、7.1），F6 第一条照旧触发，FR2 不触发 |
| 推出的条款 F1、F2、F3、F6 第二条的 `before=false after=false` | 比第 63 至 66 行 | false | 值 false 是「条款没触发」，不是没过 | 预计内 |
| `blocks_full=not_planned`（`name=flow` 9 条）、`name=share_not_planned`（9 条）、`verdict=not_planned`、`keys_equal=not_planned` | 产物第 113、588、589、594、673、679、680、681、685 行；第 698、847、848、857、872、883、884、885、892 行；第 943 行；第 85 行 | not_planned | 全量计划的块数上界超过 2^26，不排计划（原登记修订 10） | 预计内 |
| `state=not_run`（`library` 件） | 产物第 958 行 | not_run | 长期库那一项只在「共用块键」有一族判 partial 时跑，六组随机历史与覆盖写链都是 independent | 预计内 |

**岔路表**（问题单两行）：

| 问题单行 | 状态 | 还差什么 | 剩下的量能不能让它翻面 | 算出它的命令 |
|---|---|---|---|---|
| 第一行：锚 A1、锚 2、A5、锚 7 与 X3 在新段形下取什么 | 够判：「新数定」全部成立，10 格 `same=true`，自检退 0 | X3 新值 16842815 写进装置常量与两条手造读数用例（第三段开跑前，谁改缺主 agent 定）；候选甲（往原登记第十二节追加）不追，乙已成 | 不能：只有某格 `same=false` 才翻，没发生；登记里没有别的量碰这一行 | `grep -n 'name=anchor_recompute' <比>`；`grep -n 'name=selfcheck_done' <重跑产物>` |
| 第二行：第一段判定词翻不翻 | 够判：不翻（窄读 40 格、宽读 62 格、4 条推出的条款，换词 0） | 没有 | 不能：剩下的只有「离门槛的距离」（附带，够判后未跑）与原登记第二到四段（这一趟不罩）；翻不翻只看两份产物同一格的词换没换 | `grep -n 'name=verdict_comparison' <比>`；`grep -c 'same=false' <比>`（得 0） |

**它答不了的**：

- 判定词不翻，只说今天的树上同改前一样；占比的变动有多少来自取号写两轮与欠账 C755（恢复与两种挂载触发退回的失败不同）、多少来自 2026-10-19 以来 `crates/` 里别的改动与冒烟层新进的流，这一趟分不开（登记 1.1）。
- 判定词是门槛的字面结果，不稳固：R 400 步档 `inside_fraction=0.100000` 正好在「≤ 0.1 记分开」的端点上（第 952 行），R 100 步档 0.550000（11 / 20）离 0.5 只差一条流（第 948 行）；换一批种子或再多一条冒烟流进 S，词就可能换。
- 区间 S 仍只含钉过两态闭式的冒烟流（12 条加 4 条重录），不含没钉的：`name=pinned_smoke_missing` 由 15 条变 22 条（产物第 6 至 37 行之间），其中 `large-file-overwritten-across-lower-leaves-stream` 写数 1962、最长段 324（产物第 6 行，两态闭式 98 位十进制数），在候选流的任何一族之上；`above` 因此仍是相对 S 说的，不是相对冒烟层真实上界。冒烟层钉住常量的流写数最大由 824 变 828，写数 ≥ 1000 的候选流仍都在它之上。
- X3 在新段形下是否真按 16842815 判、锚 7 前半，要等第三段的 J2 冷跑；这一趟只把数定下来。
- 2026-10-19 那份产物「它答不了的」各条里没被这一趟改变的（版本表取法、每档 20 个种子、覆盖写链顺序写、9 条取不出的算术上界、判别力自证没核到）对这份产物同样成立，用其中的数时要按这份产物重算。
- 各件累计 8403 秒与改前 5462 秒不可比：两趟的机器负载与线程取法没有对齐，挂钟类的数不判。

**复跑登记**：整份第一段产物仍不登记逐字节复跑（带挂钟字段），`scripts/experiment-apparatus/replay.sh` 没改。复跑表里 E184（常规层与大负载层的代价计数） 那一行 `@driver_e184` 只跑 `selfcheck`，期望产物 `research/results/e184-regular-layer-cost-counts-selfcheck-2026-10-23.out`；这一趟的 `selfcheck` 件输出与它逐字相同，所以这一行不用改指：

```bash
diff <(sed -n 40,78p research/results/e184-regular-layer-cost-counts-segment-one-2026-10-24.out) <(head -n 39 research/results/e184-regular-layer-cost-counts-selfcheck-2026-10-23.out); echo rc=$?
```

```text
rc=0
```

`.claude/gate.d/replay-comparison-units.tsv` 里这个实验两个比对对象的「不登记」两行（文件头写着欠账号和理由）不动：比对对象 1 是改前整份产物，由逐格比脚本比；比对对象 2 是 2026-10-23 的 `selfcheck` 留存产物，这一节的 `diff` 与 10 格 `anchor_recompute` 已逐行比过。

2026-10-19 产物的读数各节（「读数：规模」「读数：态数区间重叠与共用块键」「8.2 几何敏感性：翻不翻面」「判决行与没过的字段」「作废、失败、停机条款逐条（第一段）」）里的行号与数都是改前那份产物的，没有改；这一趟的数以本节为准。

````

**出处 `.claude/kb/experiments/184-常规层与大负载层的代价计数.md:272-331`（整段抄，未转述）**

````markdown
### 读数：规模

`research/results/e184-regular-layer-cost-counts-segment-one-2026-10-19.out` 第 126、127、128、129、130、131、132、223、314、405、496、587、678 行（`name=family_summary`，每组一条，最小 / 中位 / 最大三个数在行内；这份产物带挂钟字段、不在复跑登记里，贴的是命令与原样输出）：

```text
$ grep -n 'name=family_summary' research/results/e184-regular-layer-cost-counts-segment-one-2026-10-19.out
126:E7RESULT name=family_summary family=overwrite_chain group=overwrite_chain-01_units_per_overwrite flows=1 flows_with_writes_at_least_1000=1 flows_beyond_u64=0 writes=[minimum=1003 median=1003 maximum=1003] segments=[minimum=131 median=131 maximum=131] longest_segment=[minimum=28 median=28 maximum=28] log2_two_state=[minimum=32.554588898332504 median=32.554588898332504 maximum=32.554588898332504] log2_full=[minimum=32.55458893721156 median=32.55458893721156 maximum=32.55458893721156] log2_quick_tier=[minimum=8.768184324776927 median=8.768184324776927 maximum=8.768184324776927] blocks_full=[minimum=96357 median=96357 maximum=96357]
127:E7RESULT name=family_summary family=overwrite_chain group=overwrite_chain-04_units_per_overwrite flows=1 flows_with_writes_at_least_1000=1 flows_beyond_u64=0 writes=[minimum=1042 median=1042 maximum=1042] segments=[minimum=103 median=103 maximum=103] longest_segment=[minimum=36 median=36 maximum=36] log2_two_state=[minimum=38.087545743703906 median=38.087545743703906 maximum=38.087545743703906] log2_full=[minimum=38.087545744370736 median=38.087545744370736 maximum=38.087545744370736] log2_quick_tier=[minimum=12.584257887744567 median=12.584257887744567 maximum=12.584257887744567] blocks_full=[minimum=4456784 median=4456784 maximum=4456784]
128:E7RESULT name=family_summary family=overwrite_chain group=overwrite_chain-16_units_per_overwrite flows=1 flows_with_writes_at_least_1000=1 flows_beyond_u64=0 writes=[minimum=1050 median=1050 maximum=1050] segments=[minimum=55 median=55 maximum=55] longest_segment=[minimum=60 median=60 maximum=60] log2_two_state=[minimum=61.35755202769712 median=61.35755202769712 maximum=61.35755202769712] log2_full=[minimum=61.35755202769712 median=61.35755202769712 maximum=61.35755202769712] log2_quick_tier=[minimum=35.459431623065115 median=35.459431623065115 maximum=35.459431623065115] blocks_full=[minimum=none median=none maximum=none]
129:E7RESULT name=family_summary family=smoke group=smoke-multi_record_publish_stream flows=1 flows_with_writes_at_least_1000=0 flows_beyond_u64=0 writes=[minimum=115 median=115 maximum=115] segments=[minimum=19 median=19 maximum=19] longest_segment=[minimum=30 median=30 maximum=30] log2_two_state=[minimum=30.339850117539722 median=30.339850117539722 maximum=30.339850117539722] log2_full=[minimum=30.339850149388358 median=30.339850149388358 maximum=30.339850149388358] log2_quick_tier=[minimum=7.169925001442312 median=7.169925001442312 maximum=7.169925001442312] blocks_full=[minimum=20753 median=20753 maximum=20753]
130:E7RESULT name=family_summary family=smoke group=smoke-new_pool_file_creation_stream flows=1 flows_with_writes_at_least_1000=0 flows_beyond_u64=0 writes=[minimum=41 median=41 maximum=41] segments=[minimum=11 median=11 maximum=11] longest_segment=[minimum=24 median=24 maximum=24] log2_two_state=[minimum=24.000002063790333 median=24.000002063790333 maximum=24.000002063790333] log2_full=[minimum=24.00000378361336 median=24.00000378361336 maximum=24.00000378361336] log2_quick_tier=[minimum=5.523561956057013 median=5.523561956057013 maximum=5.523561956057013] blocks_full=[minimum=267 median=267 maximum=267]
131:E7RESULT name=family_summary family=smoke group=smoke-overwrite_fixed_script_stream flows=1 flows_with_writes_at_least_1000=0 flows_beyond_u64=0 writes=[minimum=481 median=481 maximum=481] segments=[minimum=78 median=78 maximum=78] longest_segment=[minimum=28 median=28 maximum=28] log2_two_state=[minimum=30.64441974337032 median=30.64441974337032 maximum=30.64441974337032] log2_full=[minimum=30.644419842221723 median=30.644419842221723 maximum=30.644419842221723] log2_quick_tier=[minimum=8.118941072723507 median=8.118941072723507 maximum=8.118941072723507] blocks_full=[minimum=25672 median=25672 maximum=25672]
132:E7RESULT name=family_summary family=smoke group=smoke-snapshot_creation_stream flows=1 flows_with_writes_at_least_1000=0 flows_beyond_u64=0 writes=[minimum=62 median=62 maximum=62] segments=[minimum=15 median=15 maximum=15] longest_segment=[minimum=24 median=24 maximum=24] log2_two_state=[minimum=24.005627118893454 median=24.005627118893454 maximum=24.005627118893454] log2_full=[minimum=24.00562926030627 median=24.00562926030627 maximum=24.00562926030627] log2_quick_tier=[minimum=5.882643049361842 median=5.882643049361842 maximum=5.882643049361842] blocks_full=[minimum=271 median=271 maximum=271]
223:E7RESULT name=family_summary family=random_history group=random_history-broad-FourGibibytes-steps0050 flows=20 flows_with_writes_at_least_1000=12 flows_beyond_u64=0 writes=[minimum=382 median=1039.5 maximum=1464] segments=[minimum=82 median=170 maximum=229] longest_segment=[minimum=26 median=28 maximum=30] log2_two_state=[minimum=27.674636274596462 median=30.37566688831223 maximum=32.425363506393886] log2_full=[minimum=27.709519939008434 median=30.421581409516403 maximum=33.95203027866784] log2_quick_tier=[minimum=8.285402218862249 median=9.363596285096868 maximum=12.81117397072132] blocks_full=[minimum=3527 median=22108 maximum=253700]
314:E7RESULT name=family_summary family=random_history group=random_history-broad-FourGibibytes-steps0100 flows=20 flows_with_writes_at_least_1000=20 flows_beyond_u64=0 writes=[minimum=1008 median=2261.5 maximum=2752] segments=[minimum=195 median=366.5 maximum=474] longest_segment=[minimum=26 median=30 maximum=36] log2_two_state=[minimum=28.599915012358327 median=31.084452500385645 maximum=36.21589557685162] log2_full=[minimum=28.930740613687696 median=33.9700202398081 maximum=40.70491949311818] log2_quick_tier=[minimum=9.49585502688717 median=11.9788464410077 maximum=16.960827787028194] blocks_full=[minimum=8105 median=260561.5 maximum=27348049]
405:E7RESULT name=family_summary family=random_history group=random_history-broad-UnitAreaOf240Slots-steps0100 flows=20 flows_with_writes_at_least_1000=19 flows_beyond_u64=0 writes=[minimum=835 median=3571.5 maximum=5888] segments=[minimum=175 median=825.5 maximum=1391] longest_segment=[minimum=20 median=24 maximum=26] log2_two_state=[minimum=22.96464898372668 median=25.891453207134532 maximum=27.486779525829036] log2_full=[minimum=22.96481112358067 median=25.891474243909215 maximum=28.048113272159032] log2_quick_tier=[minimum=9.467605550082997 median=11.5413876819633 maximum=12.296916206879288] blocks_full=[minimum=688 median=1653 maximum=5148]
496:E7RESULT name=family_summary family=random_history group=random_history-broad_with_compaction_while_mounted-FourGibibytes-steps0100 flows=20 flows_with_writes_at_least_1000=19 flows_beyond_u64=0 writes=[minimum=936 median=2166 maximum=2583] segments=[minimum=190 median=367 maximum=437] longest_segment=[minimum=26 median=30 maximum=34] log2_two_state=[minimum=28.92240039275743 median=31.136029108832314 maximum=34.47804282673022] log2_full=[minimum=29.32235347425211 median=31.814965400572902 maximum=39.872878591274734] log2_quick_tier=[minimum=9.535275376620802 median=11.60110735828604 maximum=16.87758370515652] blocks_full=[minimum=10544 median=57976 maximum=15362357]
587:E7RESULT name=family_summary family=random_history group=random_history-broad-FourGibibytes-steps0200 flows=20 flows_with_writes_at_least_1000=20 flows_beyond_u64=0 writes=[minimum=2912 median=4327.5 maximum=5509] segments=[minimum=528 median=719.5 maximum=959] longest_segment=[minimum=28 median=32 maximum=36] log2_two_state=[minimum=29.359356900603476 median=33.43418243131835 maximum=36.81238419050781] log2_full=[minimum=29.35935876490942 median=37.32983738795073 maximum=43.40711311699381] log2_quick_tier=[minimum=10.891024190149498 median=16.724180842104673 maximum=23.212240026662197] blocks_full=[minimum=10956 median=1389411 maximum=27594991]
678:E7RESULT name=family_summary family=random_history group=random_history-broad-FourGibibytes-steps0400 flows=20 flows_with_writes_at_least_1000=20 flows_beyond_u64=0 writes=[minimum=4817 median=8580.5 maximum=10707] segments=[minimum=896 median=1430.5 maximum=1836] longest_segment=[minimum=28 median=34 maximum=40] log2_two_state=[minimum=31.460781202958813 median=35.062177803280576 maximum=40.108211687236725] log2_full=[minimum=31.460813602239426 median=40.64768052197316 maximum=49.37876086360342] log2_quick_tier=[minimum=13.613098794209224 median=19.315901203760895 maximum=26.39985199419267] blocks_full=[minimum=45798 median=6454063 maximum=55846164]
```
两张表是从这 13 行现算的（脚本从产物读，不读 crates；块数一栏只统计排了计划的流，排不了计划的流不在里面）。

规模：

| 组 | 流数 | 写数 ≥ 1000 的流数 | 写数 最小 / 中位 / 最大 | 段数 最小 / 中位 / 最大 | 最长段 最小 / 中位 / 最大 | 产物行 |
|---|---|---|---|---|---|---|
| F u=1 | 1 | 1 | 1003 / 1003 / 1003 | 131 / 131 / 131 | 28 / 28 / 28 | 126 |
| F u=4 | 1 | 1 | 1042 / 1042 / 1042 | 103 / 103 / 103 | 36 / 36 / 36 | 127 |
| F u=16 | 1 | 1 | 1050 / 1050 / 1050 | 55 / 55 / 55 | 60 / 60 / 60 | 128 |
| R 50 步 | 20 | 12 | 382 / 1039.50 / 1464 | 82 / 170 / 229 | 26 / 28 / 30 | 223 |
| R 100 步（主取样） | 20 | 20 | 1008 / 2261.50 / 2752 | 195 / 366.50 / 474 | 26 / 30 / 36 | 314 |
| R 100 步 盘宽 240 槽 | 20 | 19 | 835 / 3571.50 / 5888 | 175 / 825.50 / 1391 | 20 / 24 / 26 | 405 |
| R 100 步 带整理比重 | 20 | 19 | 936 / 2166 / 2583 | 190 / 367 / 437 | 26 / 30 / 34 | 496 |
| R 200 步 | 20 | 20 | 2912 / 4327.50 / 5509 | 528 / 719.50 / 959 | 28 / 32 / 36 | 587 |
| R 400 步 | 20 | 20 | 4817 / 8580.50 / 10707 | 896 / 1430.50 / 1836 | 28 / 34 / 40 | 678 |
| 冒烟 新池新建文件流 | 1 | 0 | 41 / 41 / 41 | 11 / 11 / 11 | 24 / 24 / 24 | 130 |
| 冒烟 建快照流 | 1 | 0 | 62 / 62 / 62 | 15 / 15 / 15 | 24 / 24 / 24 | 132 |
| 冒烟 多记录发布流 | 1 | 0 | 115 / 115 / 115 | 19 / 19 / 19 | 30 / 30 / 30 | 129 |
| 冒烟 覆盖写固定脚本流 | 1 | 0 | 481 / 481 / 481 | 78 / 78 / 78 | 28 / 28 / 28 | 131 |
态数（对数以 2 为底）：

| 组 | log2 两态闭式 最小 / 中位 / 最大 | log2 全量态数 最小 / 中位 / 最大 | log2 甲二态数 最小 / 中位 / 最大 | 全量块数 最小 / 中位 / 最大 |
|---|---|---|---|---|
| F u=1 | 32.55 / 32.55 / 32.55 | 32.55 / 32.55 / 32.55 | 8.77 / 8.77 / 8.77 | 96357 / 96357 / 96357 |
| F u=4 | 38.09 / 38.09 / 38.09 | 38.09 / 38.09 / 38.09 | 12.58 / 12.58 / 12.58 | 4456784 / 4456784 / 4456784 |
| F u=16 | 61.36 / 61.36 / 61.36 | 61.36 / 61.36 / 61.36 | 35.46 / 35.46 / 35.46 | 取不出 / 取不出 / 取不出 |
| R 50 步 | 27.67 / 30.38 / 32.43 | 27.71 / 30.42 / 33.95 | 8.29 / 9.36 / 12.81 | 3527 / 22108 / 253700 |
| R 100 步（主取样） | 28.60 / 31.08 / 36.22 | 28.93 / 33.97 / 40.70 | 9.50 / 11.98 / 16.96 | 8105 / 260561.50 / 27348049 |
| R 100 步 盘宽 240 槽 | 22.96 / 25.89 / 27.49 | 22.96 / 25.89 / 28.05 | 9.47 / 11.54 / 12.30 | 688 / 1653 / 5148 |
| R 100 步 带整理比重 | 28.92 / 31.14 / 34.48 | 29.32 / 31.81 / 39.87 | 9.54 / 11.60 / 16.88 | 10544 / 57976 / 15362357 |
| R 200 步 | 29.36 / 33.43 / 36.81 | 29.36 / 37.33 / 43.41 | 10.89 / 16.72 / 23.21 | 10956 / 1389411 / 27594991 |
| R 400 步 | 31.46 / 35.06 / 40.11 | 31.46 / 40.65 / 49.38 | 13.61 / 19.32 / 26.40 | 45798 / 6454063 / 55846164 |
| 冒烟 新池新建文件流 | 24.00 / 24.00 / 24.00 | 24.00 / 24.00 / 24.00 | 5.52 / 5.52 / 5.52 | 267 / 267 / 267 |
| 冒烟 建快照流 | 24.01 / 24.01 / 24.01 | 24.01 / 24.01 / 24.01 | 5.88 / 5.88 / 5.88 | 271 / 271 / 271 |
| 冒烟 多记录发布流 | 30.34 / 30.34 / 30.34 | 30.34 / 30.34 / 30.34 | 7.17 / 7.17 / 7.17 | 20753 / 20753 / 20753 |
| 冒烟 覆盖写固定脚本流 | 30.64 / 30.64 / 30.64 | 30.64 / 30.64 / 30.64 | 8.12 / 8.12 / 8.12 | 25672 / 25672 / 25672 |
- 写数随步数档涨：随机历史 50、100、200、400 步档的写数中位是 1039.5、2261.5、4327.5、8580.5；两态闭式的 log2 中位是 30.38、31.08、33.43、35.06，全量态数的 log2 中位是 30.42、33.97、37.33、40.65。四档连起来看，全量态数的增长比两态闭式快：400 步档最大一条全量态数到 2^49.38，两态闭式 2^40.11。
- 没有一条流的两态闭式装不进 u64（13 条 `family_summary` 的 `flows_beyond_u64` 都是 0）；覆盖写链 u = 16 的两态闭式 2954361402816462894（log2 61.36）仍装得进 u64，最长段 60 写，没到 64 写。
- 「写请求数上千」在写数这根轴上分得开：冒烟层 14 条已钉流的写数最大 824（十条钉住常量的流写数 126 至 824，四条重录流 41 至 481），随机历史与覆盖写链里写数 ≥ 1000 的流都在它之上；但 50 步档 20 条里 8 条写数不到 1000，盘宽 240 槽与带整理比重两组各有 1 条不到 1000。在态数这根轴上分不开，见「读数：态数区间重叠与共用块键」一节。

````

**出处 `.claude/kb/experiments/184-常规层与大负载层的代价计数.md:332-407`（整段抄，未转述）**

````markdown
### 读数：态数区间重叠与共用块键

区间 S 是冒烟层 14 条已钉流的两态闭式：产物第 4 至 13 行的十条钉住常量的流（`name=pinned_smoke`，十条都 `match=true`）加四条重录流（产物第 129 至 132 行的 `family_summary`）；下界 16777240（新池新建文件流），上界 2438004868（`truncate_under_a_snapshot`），最长段区间 24 至 30。登记里钉不住的 15 条流写在产物第 14 至 28 行（`name=pinned_smoke_missing`，原因都是 `no_segment_shape_constant_with_a_pinned_closed_form`），不在 S 里。

「态数区间重叠」（`inside` 占比按写数 ≥ 1000 的流算；最长段一栏是这些流的最长段落在 24 至 30 里的占比，登记要求另报，不参与判定）：

| 范围 | 写数 ≥ 1000 的流 | 在 S 里 / 高于 S 上界 / 低于 S 下界 | inside 占比 | 最长段落在冒烟层最长段区间里的占比 | 乙判定 | 产物行 |
|---|---|---|---|---|---|---|
| R 主取样合并（四档步数共 80 条流） | 72 | 21 / 51 / 0 | 0.291667 | 0.486111 | partial | 929 |
| R 50 步 | 12 | 7 / 5 / 0 | 0.583333 | 1.000000 | overlap | 937 |
| R 100 步（主取样） | 20 | 10 / 10 / 0 | 0.500000 | 0.600000 | overlap | 939 |
| R 100 步 盘宽 240 槽 | 19 | 17 / 0 / 2 | 0.894737 | 0.631579 | overlap | 945 |
| R 100 步 带整理比重 | 19 | 10 / 9 / 0 | 0.526316 | 0.842105 | overlap | 947 |
| R 200 步 | 20 | 4 / 16 / 0 | 0.200000 | 0.450000 | partial | 941 |
| R 400 步 | 20 | 0 / 20 / 0 | 0.000000 | 0.100000 | separate | 943 |
| F 主取样（u = 4） | 1 | 0 / 1 / 0 | 0.000000 | 0.000000 | separate | 927 |
| F u = 1 | 1 | 0 / 1 / 0 | 0.000000 | 1.000000 | separate | 931 |
| F u = 4 | 1 | 0 / 1 / 0 | 0.000000 | 0.000000 | separate | 933 |
| F u = 16 | 1 | 0 / 1 / 0 | 0.000000 | 0.000000 | separate | 935 |
「共用块键」（共用块占比按块算，判定用；复用键占比是不含计划哈希与版本表的上界，另报，不参与判定）：

| 范围 | 排得了计划的流 / 总流数 | 共用块占比 中位 / 最大 | 复用键占比（上界）中位 / 最大 | 按态数加权占比 中位 | 丙判定 | 产物行 |
|---|---|---|---|---|---|---|
| R 主取样合并（四档步数共 80 条流） | 72 / 80 | 0.000024 / 0.001985 | 0.000161 / 0.053737 | 0.000000 | independent | 928 |
| R 50 步 | 20 / 20 | 0.000284 / 0.001985 | 0.001355 / 0.053737 | 0.000000 | independent | 936 |
| R 100 步（主取样） | 20 / 20 | 0.000028 / 0.000864 | 0.000222 / 0.007968 | 0.000000 | independent | 938 |
| R 100 步 盘宽 240 槽 | 20 / 20 | 0.000000 / 0.000000 | 0.000000 / 0.000000 | 0.000000 | independent | 944 |
| R 100 步 带整理比重 | 20 / 20 | 0.000121 / 0.000664 | 0.000172 / 0.025228 | 0.000000 | independent | 946 |
| R 200 步 | 17 / 20 | 0.000005 / 0.000548 | 0.000062 / 0.005095 | 0.000000 | independent | 940 |
| R 400 步 | 15 / 20 | 0.000001 / 0.000153 | 0.000004 / 0.000153 | 0.000000 | independent | 942 |
| F 主取样（u = 4） | 1 / 1 | 0.000060 / 0.000060 | 0.000060 / 0.000060 | 0.000057 | independent | 926 |
| F u = 1 | 1 / 1 | 0.002761 / 0.002761 | 0.002761 / 0.002761 | 0.002660 | independent | 930 |
| F u = 4 | 1 / 1 | 0.000060 / 0.000060 | 0.000060 / 0.000060 | 0.000057 | independent | 932 |
| F u = 16 | 0 / 1 | 取不出 | 取不出 | 取不出 | not_planned（取不出） | 934 |
两张表的判定词来自 22 条 `name=share_group` 与 `name=overlap`，在产物第 926、927、928、929、930、931、932、933、934、935、936、937、938、939、940、941、942、943、944、945、946、947 行（表里「产物行」一栏是各范围在其中的行号；产物不在复跑登记里，贴的是命令与原样输出）：

```text
$ grep -nE 'name=(share_group|overlap) ' research/results/e184-regular-layer-cost-counts-segment-one-2026-10-19.out
926:E7RESULT name=share_group scope=family:overwrite_chain_main flows=1 planned_flows=1 median_share_fraction=0.000060 maximum_share_fraction=0.000060 median_weighted_fraction=0.000057 median_reuse_fraction=0.000060 maximum_reuse_fraction=0.000060 verdict=independent
927:E7RESULT name=overlap scope=family:overwrite_chain_main smoke_interval_two_state_minimum=16777240 smoke_interval_two_state_maximum=2438004868 smoke_interval_longest_segment_minimum=24 smoke_interval_longest_segment_maximum=30 flows=1 flows_with_writes_at_least_1000=1 inside=0 above=1 below=0 inside_fraction=0.000000 above_fraction=1.000000 at_most_smoke_interval_maximum_fraction=0.000000 longest_segment_inside_fraction=0.000000 verdict=separate
928:E7RESULT name=share_group scope=family:random_history_main flows=80 planned_flows=72 median_share_fraction=0.000024 maximum_share_fraction=0.001985 median_weighted_fraction=0.000000 median_reuse_fraction=0.000161 maximum_reuse_fraction=0.053737 verdict=independent
929:E7RESULT name=overlap scope=family:random_history_main smoke_interval_two_state_minimum=16777240 smoke_interval_two_state_maximum=2438004868 smoke_interval_longest_segment_minimum=24 smoke_interval_longest_segment_maximum=30 flows=80 flows_with_writes_at_least_1000=72 inside=21 above=51 below=0 inside_fraction=0.291667 above_fraction=0.708333 at_most_smoke_interval_maximum_fraction=0.291667 longest_segment_inside_fraction=0.486111 verdict=partial
930:E7RESULT name=share_group scope=group:overwrite_chain-01_units_per_overwrite flows=1 planned_flows=1 median_share_fraction=0.002761 maximum_share_fraction=0.002761 median_weighted_fraction=0.002660 median_reuse_fraction=0.002761 maximum_reuse_fraction=0.002761 verdict=independent
931:E7RESULT name=overlap scope=group:overwrite_chain-01_units_per_overwrite smoke_interval_two_state_minimum=16777240 smoke_interval_two_state_maximum=2438004868 smoke_interval_longest_segment_minimum=24 smoke_interval_longest_segment_maximum=30 flows=1 flows_with_writes_at_least_1000=1 inside=0 above=1 below=0 inside_fraction=0.000000 above_fraction=1.000000 at_most_smoke_interval_maximum_fraction=0.000000 longest_segment_inside_fraction=1.000000 verdict=separate
932:E7RESULT name=share_group scope=group:overwrite_chain-04_units_per_overwrite flows=1 planned_flows=1 median_share_fraction=0.000060 maximum_share_fraction=0.000060 median_weighted_fraction=0.000057 median_reuse_fraction=0.000060 maximum_reuse_fraction=0.000060 verdict=independent
933:E7RESULT name=overlap scope=group:overwrite_chain-04_units_per_overwrite smoke_interval_two_state_minimum=16777240 smoke_interval_two_state_maximum=2438004868 smoke_interval_longest_segment_minimum=24 smoke_interval_longest_segment_maximum=30 flows=1 flows_with_writes_at_least_1000=1 inside=0 above=1 below=0 inside_fraction=0.000000 above_fraction=1.000000 at_most_smoke_interval_maximum_fraction=0.000000 longest_segment_inside_fraction=0.000000 verdict=separate
934:E7RESULT name=share_group scope=group:overwrite_chain-16_units_per_overwrite flows=1 planned_flows=0 verdict=not_planned
935:E7RESULT name=overlap scope=group:overwrite_chain-16_units_per_overwrite smoke_interval_two_state_minimum=16777240 smoke_interval_two_state_maximum=2438004868 smoke_interval_longest_segment_minimum=24 smoke_interval_longest_segment_maximum=30 flows=1 flows_with_writes_at_least_1000=1 inside=0 above=1 below=0 inside_fraction=0.000000 above_fraction=1.000000 at_most_smoke_interval_maximum_fraction=0.000000 longest_segment_inside_fraction=0.000000 verdict=separate
936:E7RESULT name=share_group scope=group:random_history-broad-FourGibibytes-steps0050 flows=20 planned_flows=20 median_share_fraction=0.000284 maximum_share_fraction=0.001985 median_weighted_fraction=0.000000 median_reuse_fraction=0.001355 maximum_reuse_fraction=0.053737 verdict=independent
937:E7RESULT name=overlap scope=group:random_history-broad-FourGibibytes-steps0050 smoke_interval_two_state_minimum=16777240 smoke_interval_two_state_maximum=2438004868 smoke_interval_longest_segment_minimum=24 smoke_interval_longest_segment_maximum=30 flows=20 flows_with_writes_at_least_1000=12 inside=7 above=5 below=0 inside_fraction=0.583333 above_fraction=0.416667 at_most_smoke_interval_maximum_fraction=0.583333 longest_segment_inside_fraction=1.000000 verdict=overlap
938:E7RESULT name=share_group scope=group:random_history-broad-FourGibibytes-steps0100 flows=20 planned_flows=20 median_share_fraction=0.000028 maximum_share_fraction=0.000864 median_weighted_fraction=0.000000 median_reuse_fraction=0.000222 maximum_reuse_fraction=0.007968 verdict=independent
939:E7RESULT name=overlap scope=group:random_history-broad-FourGibibytes-steps0100 smoke_interval_two_state_minimum=16777240 smoke_interval_two_state_maximum=2438004868 smoke_interval_longest_segment_minimum=24 smoke_interval_longest_segment_maximum=30 flows=20 flows_with_writes_at_least_1000=20 inside=10 above=10 below=0 inside_fraction=0.500000 above_fraction=0.500000 at_most_smoke_interval_maximum_fraction=0.500000 longest_segment_inside_fraction=0.600000 verdict=overlap
940:E7RESULT name=share_group scope=group:random_history-broad-FourGibibytes-steps0200 flows=20 planned_flows=17 median_share_fraction=0.000005 maximum_share_fraction=0.000548 median_weighted_fraction=0.000000 median_reuse_fraction=0.000062 maximum_reuse_fraction=0.005095 verdict=independent
941:E7RESULT name=overlap scope=group:random_history-broad-FourGibibytes-steps0200 smoke_interval_two_state_minimum=16777240 smoke_interval_two_state_maximum=2438004868 smoke_interval_longest_segment_minimum=24 smoke_interval_longest_segment_maximum=30 flows=20 flows_with_writes_at_least_1000=20 inside=4 above=16 below=0 inside_fraction=0.200000 above_fraction=0.800000 at_most_smoke_interval_maximum_fraction=0.200000 longest_segment_inside_fraction=0.450000 verdict=partial
942:E7RESULT name=share_group scope=group:random_history-broad-FourGibibytes-steps0400 flows=20 planned_flows=15 median_share_fraction=0.000001 maximum_share_fraction=0.000153 median_weighted_fraction=0.000000 median_reuse_fraction=0.000004 maximum_reuse_fraction=0.000153 verdict=independent
943:E7RESULT name=overlap scope=group:random_history-broad-FourGibibytes-steps0400 smoke_interval_two_state_minimum=16777240 smoke_interval_two_state_maximum=2438004868 smoke_interval_longest_segment_minimum=24 smoke_interval_longest_segment_maximum=30 flows=20 flows_with_writes_at_least_1000=20 inside=0 above=20 below=0 inside_fraction=0.000000 above_fraction=1.000000 at_most_smoke_interval_maximum_fraction=0.000000 longest_segment_inside_fraction=0.100000 verdict=separate
944:E7RESULT name=share_group scope=group:random_history-broad-UnitAreaOf240Slots-steps0100 flows=20 planned_flows=20 median_share_fraction=0.000000 maximum_share_fraction=0.000000 median_weighted_fraction=0.000000 median_reuse_fraction=0.000000 maximum_reuse_fraction=0.000000 verdict=independent
945:E7RESULT name=overlap scope=group:random_history-broad-UnitAreaOf240Slots-steps0100 smoke_interval_two_state_minimum=16777240 smoke_interval_two_state_maximum=2438004868 smoke_interval_longest_segment_minimum=24 smoke_interval_longest_segment_maximum=30 flows=20 flows_with_writes_at_least_1000=19 inside=17 above=0 below=2 inside_fraction=0.894737 above_fraction=0.000000 at_most_smoke_interval_maximum_fraction=1.000000 longest_segment_inside_fraction=0.631579 verdict=overlap
946:E7RESULT name=share_group scope=group:random_history-broad_with_compaction_while_mounted-FourGibibytes-steps0100 flows=20 planned_flows=20 median_share_fraction=0.000121 maximum_share_fraction=0.000664 median_weighted_fraction=0.000000 median_reuse_fraction=0.000172 maximum_reuse_fraction=0.025228 verdict=independent
947:E7RESULT name=overlap scope=group:random_history-broad_with_compaction_while_mounted-FourGibibytes-steps0100 smoke_interval_two_state_minimum=16777240 smoke_interval_two_state_maximum=2438004868 smoke_interval_longest_segment_minimum=24 smoke_interval_longest_segment_maximum=30 flows=20 flows_with_writes_at_least_1000=19 inside=10 above=9 below=0 inside_fraction=0.526316 above_fraction=0.473684 at_most_smoke_interval_maximum_fraction=0.526316 longest_segment_inside_fraction=0.842105 verdict=overlap
```
- 随机历史按步数档，「态数区间重叠」从 50 步档的 overlap 到 400 步档的 separate 单调走：`inside` 占比 0.583333、0.500000、0.200000、0.000000。100 步档正好 10 条在 S 里、10 条在上界之上，占比恰为门槛 0.5，判 overlap（`inside_fraction >= 0.5`）；这一档 20 条流的两态闭式从小到大排是命令输出的 20 个数，第 10 个 2092827143 在 S 里，第 11 个 2477261445 在上界 2438004868 之上，差 1.6%：

```bash
grep 'name=flow family=random_history' research/results/e184-regular-layer-cost-counts-segment-one-2026-10-19.out | grep 'label=random_history-broad-FourGibibytes-steps0100-' | grep -o ' two_state=[0-9]*' | cut -d= -f2 | sort -n | paste -sd' '
```

```text
406848100 604381514 617218498 1149051398 1545666995 1576928306 1617625859 1788805901 1827996579 2092827143 2477261445 7110525483 8232043213 9730458182 9900261778 18592432719 18648662610 22181118834 76742721965 79812495196
```

  只要有一条在 S 里的流换到上界之上，这一档就是 partial（9 条在里面，0.45）；带整理比重一组同样靠近门槛（10 条 / 19 条 = 0.526316，少一条在里面就是 9 / 19 = 0.473684，partial）；盘宽 240 槽一组 17 / 19，离门槛远。
- 「共用块键」全部范围都是 independent，共用块很少：覆盖写链 u = 1 的 96357 块里 266 块与冒烟层共用（产物第 686 行，共用态数 16777259；冒烟层新池新建文件流有 267 块、全量态数 16777260，共用的应是它的 266 块，覆盖写链和它从同一个标准前缀起，这是推的，装置没按块标出处），u = 4 的 4456784 块里同样 266 块；盘宽 240 槽一组 20 条流的 `shared_blocks` 全是 0。
- 复用键占比是块键共用的上界：它在 50 步档种子 06 一条流上到 0.053737（共用 7 块、总块数 4950，产物第 747 行），越过了 0.05 的门槛，但判定按块占比（0.001414），这一条不改判定；其余流的复用键占比最大 0.025228（带整理比重一组种子 09，产物第 905 行）。
- 排不了计划的流（块数上界超过 2^26）：随机历史 200 步档 3 条（种子 12、13、18）、400 步档 5 条（种子 06、12、13、14、18），加覆盖写链 u = 16，共 9 条。它们的「共用块键」取不出，按登记修订 10 不写 0、不写独立。共用块数不会超过冒烟层四条重录流的块数之和 46963（产物 `name=flow` 里四条冒烟流的 `blocks_full` 267、271、20753、25672 相加），每条排不了计划的流至少有「全量态数 ÷ 65536」块，所以共用占比的上界是 46963 除以这个块数：9 条里最大的一条是 200 步档种子 18，上界 0.000494（推的，从产物 `full=` 现算，装置没量），u = 16 的上界是 1.04e-9；这个算术上界不是装置量到的数，不替代 `not_planned`。命令与逐条输出在「它答不了的」一节。

````

**出处 `.claude/kb/experiments/184-常规层与大负载层的代价计数.md:408-421`（整段抄，未转述）**

```markdown
### 8.2 几何敏感性：翻不翻面

登记第 8.2 节的几何敏感性：同一门槛在方向相反的取样点上重判，判定与主取样不同记「不稳定」，相同记两点：

| 判决 | 主取样 | 方向相反的取样点 | 记法 |
|---|---|---|---|
| 态数区间重叠（随机历史） | BROAD、4 GiB、100 步：overlap（0.500000，产物第 939 行） | 盘宽 240 槽：overlap（0.894737，第 945 行）；带整理比重：overlap（0.526316，第 947 行） | 三点相同，记两点 |
| 共用块键（随机历史） | independent（第 938 行） | 盘宽 240 槽 independent（第 944 行）；带整理比重 independent（第 946 行） | 三点相同，记两点 |
| 态数区间重叠（覆盖写链） | u = 4：separate（第 933 行） | u = 1 separate（第 931 行）；u = 16 separate（第 935 行） | 三点相同，记两点 |
| 共用块键（覆盖写链） | u = 4：independent（第 932 行） | u = 1 independent（第 930 行）；u = 16 取不出（第 934 行 `verdict=not_planned`） | u = 16 取不出，不记两点，也不记稳定 |
| 态数区间重叠（步数） | 写数 ≥ 1000 的全部流合并：partial（0.291667，第 929 行） | 只取 50 步档：overlap（0.583333，第 937 行） | 判定不同，记「不稳定」 |

另外 100 步以外的步数档也翻面（第 941 行 200 步 partial、第 943 行 400 步 separate）：「态数区间重叠」对随机历史不是一个与步数无关的判定，它随步数档从 overlap 走到 separate。登记第八节末句要求的判别力自证（把门槛挪到两个取样点的值之间，判定必须换边）：装置单测里只找到门槛包含端点与 partial 带的两条（`the_share_verdict_has_inclusive_thresholds_and_a_partial_band`、`the_overlap_verdict_has_inclusive_thresholds_and_a_partial_band`），没有找到以两个取样点的值为输入的换边测试；这一格记「没核到」，交主 agent 判是否补。

```

**出处 `.claude/kb/experiments/184-常规层与大负载层的代价计数.md:422-481`（整段抄，未转述）**

````markdown
### 判决行与没过的字段

产物不打 `name=verdict` 行；装置的判定词写在 `name=share_group` 与 `name=overlap` 的 `verdict=` 字段上，22 条的命令与原样输出在「读数：态数区间重叠与共用块键」一节。逐项数：

```bash
P=research/results/e184-regular-layer-cost-counts-segment-one-2026-10-19.out
grep -c 'name=verdict' $P
grep -c 'passed=false' $P
grep -c 'result=fail' $P
grep -c 'equal=false' $P
grep -cE 'failed=[1-9]|controls_failed=[1-9]|count_disagreements=[1-9]|base_mismatches=[1-9]|anchors_mismatching=[1-9]|mismatches=[1-9]' $P
grep -n 'match=false' $P
```

```text
0
0
0
0
0
68:E7RESULT name=anchor id=B1_multi_record_full_named_by_the_clause expected=12230590578 actual=1358954634 match=false
```

整行抄自 2026-10-19 的 `selfcheck` 留存产物 `research/results/e184-regular-layer-cost-counts-selfcheck-2026-10-19.out` 第 38、39 行（整份产物第 68、69 行；只修钉值之后的 `research/results/e184-regular-layer-cost-counts-selfcheck-2026-10-23.out` 第 38、39 行是 `actual=1358954642 match=false` 与同一条 `selfcheck_done`）：

```text
E7RESULT name=anchor identifier=multi_record_full_named_by_the_clause expected=12230590578 actual=1358954634 match=false
E7RESULT name=selfcheck_done stop_checks=28 failed=0 anchors=10 anchors_mismatching=0 clause_anchor_matches=false emitted=39
```
没过或取不出的字段逐个点名（`verdict=` 取值 overlap、separate、partial、independent 是判定词，不算没过）：

| 字段 | 产物行 | 取值 | 为什么是这个值 | 登记预期内吗 |
|---|---|---|---|---|
| `match=false`（B1）与 `b1_matches=false` | 68、69 | false | D13（验证路线） 已定项 9 射程写的多记录流 12230590578 个状态，装置重录这条流得 1358954634；多记录流测试文件头（`crates/singlefs-verification/tests/crash_enumeration_multi_record_publish_stream.rs` 第 22 行）写着 C577（系统配置没见证到的最新根，乙罩不到） 改段形之前、轮换并进单元段时的两个数 12230590578 与 5435818083；条款句里的数是改段形之前的数 | 预计内（登记第四节第 2 条），触发失败条款 F6 的第一条观测，不作废 |
| `blocks_full=not_planned`（9 条 `name=flow`）、`name=share_not_planned`（9 条）、`verdict=not_planned`（第 934 行）、`keys_equal=not_planned`（第 76 行） | 104、579、580、585、664、670、671、672、676、689、838、839、848、863、874、875、876、883、934、76 | not_planned | 全量计划的块数上界超过 2^26，不排计划（登记修订 10）；「共用块键」对这 9 条取不出 | 预计内（修订 10），覆盖写链 u = 16 预期，随机历史 8 条登记里没有预期过，由修订 10 的上界规则落到这里 |
| `state=not_run`（`library` 件） | 949 | not_run | 长期库那一项只在「共用块键」有一族判 partial 时跑，六组随机历史与覆盖写链都是 independent | 预计内（登记第六节长期库那一行） |

`name=flow` 与 `name=share_not_planned` 在产物第 104、579、580、585、664、670、671、672、676、689、838、839、848、863、874、875、876、883 行（产物不在复跑登记里，贴的是命令与原样输出）：

```text
$ grep -nE 'blocks_full=not_planned|name=share_not_planned' research/results/e184-regular-layer-cost-counts-segment-one-2026-10-19.out
104:E7RESULT name=flow family=overwrite_chain tier=candidate label=overwrite_chain-16_units_per_overwrite seed=none ending=NotAHistory:overwrites=11 writes=1050 segments=55 longest_segment=60 two_state=2954361402816462894 log2_two_state=61.3576 full=2954361402816462969 quick_tier=47244640401 blocks_full=not_planned blocks_quick=720941 points=56 keys_seconds=1.3 two_state_agrees_with_crates=true reused=false
579:E7RESULT name=flow family=random_history tier=candidate label=random_history-broad-FourGibibytes-steps0200-seed12 seed=7463871032432355125 ending=Completed writes=4619 segments=759 longest_segment=36 two_state=88704156932 log2_two_state=36.3683 full=9921447726162 quick_tier=1203896 blocks_full=not_planned blocks_quick=777 points=760 keys_seconds=29.1 two_state_agrees_with_crates=true reused=false
580:E7RESULT name=flow family=random_history tier=candidate label=random_history-broad-FourGibibytes-steps0200-seed13 seed=7463871032432355126 ending=Completed writes=5470 segments=892 longest_segment=36 two_state=120679041817 log2_two_state=36.8124 full=11663880818977 quick_tier=2580113 blocks_full=not_planned blocks_quick=928 points=893 keys_seconds=121.7 two_state_agrees_with_crates=true reused=false
585:E7RESULT name=flow family=random_history tier=candidate label=random_history-broad-FourGibibytes-steps0200-seed18 seed=7463871032432355131 ending=Completed writes=5084 segments=857 longest_segment=34 two_state=47264499128 log2_two_state=35.4600 full=6233485413873 quick_tier=9718081 blocks_full=not_planned blocks_quick=1004 points=858 keys_seconds=29.0 two_state_agrees_with_crates=true reused=false
664:E7RESULT name=flow family=random_history tier=candidate label=random_history-broad-FourGibibytes-steps0400-seed06 seed=7463871032432355119 ending=Completed writes=10707 segments=1836 longest_segment=36 two_state=157541863687 log2_two_state=37.1969 full=18690168524082 quick_tier=3347048 blocks_full=not_planned blocks_quick=1885 points=1837 keys_seconds=108.4 two_state_agrees_with_crates=true reused=false
670:E7RESULT name=flow family=random_history tier=candidate label=random_history-broad-FourGibibytes-steps0400-seed12 seed=7463871032432355125 ending=Completed writes=9317 segments=1564 longest_segment=36 two_state=98945404591 log2_two_state=36.5259 full=10165605368686 quick_tier=1338332 blocks_full=not_planned blocks_quick=1582 points=1565 keys_seconds=55.7 two_state_agrees_with_crates=true reused=false
671:E7RESULT name=flow family=random_history tier=candidate label=random_history-broad-FourGibibytes-steps0400-seed13 seed=7463871032432355126 ending=Completed writes=10422 segments=1730 longest_segment=36 two_state=164590854855 log2_two_state=37.2601 full=18429129377335 quick_tier=56989467 blocks_full=not_planned blocks_quick=2587 points=1731 keys_seconds=57.2 two_state_agrees_with_crates=true reused=false
672:E7RESULT name=flow family=random_history tier=candidate label=random_history-broad-FourGibibytes-steps0400-seed14 seed=7463871032432355127 ending=Completed writes=8931 segments=1486 longest_segment=40 two_state=1185154009993 log2_two_state=40.1082 full=731961382353473 quick_tier=88541593 blocks_full=not_planned blocks_quick=2834 points=1487 keys_seconds=43.6 two_state_agrees_with_crates=true reused=false
676:E7RESULT name=flow family=random_history tier=candidate label=random_history-broad-FourGibibytes-steps0400-seed18 seed=7463871032432355131 ending=Completed writes=9607 segments=1606 longest_segment=36 two_state=127927716645 log2_two_state=36.8965 full=15453626179150 quick_tier=10948412 blocks_full=not_planned blocks_quick=1770 points=1607 keys_seconds=124.1 two_state_agrees_with_crates=true reused=false
689:E7RESULT name=share_not_planned flow=overwrite_chain-16_units_per_overwrite reason=no_blocks_planned
838:E7RESULT name=share_not_planned flow=random_history-broad-FourGibibytes-steps0200-seed12 reason=no_blocks_planned
839:E7RESULT name=share_not_planned flow=random_history-broad-FourGibibytes-steps0200-seed13 reason=no_blocks_planned
848:E7RESULT name=share_not_planned flow=random_history-broad-FourGibibytes-steps0200-seed18 reason=no_blocks_planned
863:E7RESULT name=share_not_planned flow=random_history-broad-FourGibibytes-steps0400-seed06 reason=no_blocks_planned
874:E7RESULT name=share_not_planned flow=random_history-broad-FourGibibytes-steps0400-seed12 reason=no_blocks_planned
875:E7RESULT name=share_not_planned flow=random_history-broad-FourGibibytes-steps0400-seed13 reason=no_blocks_planned
876:E7RESULT name=share_not_planned flow=random_history-broad-FourGibibytes-steps0400-seed14 reason=no_blocks_planned
883:E7RESULT name=share_not_planned flow=random_history-broad-FourGibibytes-steps0400-seed18 reason=no_blocks_planned
```
````

**出处 `.claude/kb/experiments/184-常规层与大负载层的代价计数.md:482-503`（整段抄，未转述）**

```markdown
### 作废、失败、停机条款逐条（第一段）

| 条款 | 这一段的读数 | 结论 |
|---|---|---|
| X1 阳性对照没过 | `name=control` 174 条全部 `passed=true`（`grep -c 'passed=false'` 为 0）；各 `*_done` 行 `controls_failed=0`；覆盖写链 u = 16 上三类要全量计划的对照没跑（修订 10 第 2 条），转储往返照跑 | 不触发 |
| X2 随机历史确定性 | 12 条 `name=determinism`（六档各种子 0、1 录两遍），`equal=true` 12 条（第 175、176、266、267、357、358、448、449、539、540、630、631 行） | 不触发 |
| X3、X4、X5 | 属于第二、三、四段 | 这一段不判 |
| X6 读不到不等于读到 0 | 排不了计划的 9 条流写 `not_planned`，没有写成 0；`library` 件写 `state=not_run` | 按条款处置 |
| X7 内存包装退 245、250 至 254 | 17 件现跑的 `name=segment` 行都是 `state=done runs=1`（没有重起），没有件因包装退出码留下输出 | 不触发 |
| 停机：常量回比 | 第 31 至 33 行三条 `result=pass` | 过 |
| T2 照抄的段逐字 | 第 34 至 38 行五条 `result=pass`（四段加 `segments=4_expected=4`） | 过 |
| T3 重录值等于测试钉的常量 | 第 39、40、44、45、49、50、54、55 行八条 `result=pass`；十条钉住常量的流 `match=true`、`pinned_smoke_done` `mismatches=0`（第 29 行） | 过 |
| T4 两条数全量态数的路径相等 | 第 41、46、51、56 行 `result=pass`；127 条 `name=flow` 所在各 `keys_done` 行 `count_disagreements=0` | 过 |
| T5 | 属于第三段 | 这一段不判 |
| 停机：随机历史到 `CrashFlow` 的转换 crates 里没有 | 第 3 行 `result=pass hits=0 files_read=18` | 过 |
| T7 切段撞断言或 panic | 随机历史 120 条 `ending=Completed`，六个 `dump_done` 的 `failed=0`，`keys_done` 的 `failed=0` | 不触发 |
| F1 甲分不了层 | 触发需随机历史与覆盖写链两族「态数区间重叠」都 overlap 且 `above` 都小于 0.5：随机历史合并 partial（第 929 行）、覆盖写链 separate 且 `above_fraction=1.000000`（第 927 行） | 不触发 |
| F2 乙名不副实 | 随机历史「共用块键」判 renamed：合并范围 independent，中位 0.000024（第 928 行） | 不触发 |
| F3 覆盖写链只是换名 | 覆盖写链「共用块键」判 renamed：u = 4 independent（第 932 行 0.000060），u = 1 independent（第 930 行 0.002761），u = 16 取不出 | 不触发（u = 16 一档没有读数） |
| F4、F5、F7 | 属于第二、三、四段 | 这一段不判 |
| F6 条款这一格可能错 | 第一条观测：B1 的多记录流全量态数 1358954634 不等于 12230590578（第 68 行）。第二条观测（随机历史写数 ≥ 1000 的流里两态闭式不超过 S 上界的占比 ≥ 0.5）：登记没写按哪个范围读，合并范围 `at_most_s_max_fraction=0.291667`（第 929 行）不触发；逐组读，50 步 0.583333（第 937 行）、100 步 0.500000（第 939 行）、盘宽 240 槽 1.000000（第 945 行）、带整理比重 0.526316（第 947 行）四组 ≥ 0.5，200 步 0.200000、400 步 0.000000 不到 | 第一条触发（预计内）；第二条两种读法给出不同的结果，两种都列，不挑 |

```

**出处 `.claude/kb/experiments/184-常规层与大负载层的代价计数.md:504-508`（整段抄，未转述）**

```markdown
### 单测与变异

- 单测：主 agent 在 release 下跑的读数 82 passed、0 failed（日志在仓外，不入库）；`grep -c '#\[test\]' crates/singlefs-verification/src/bin/e184_regular_layer_cost_counts.rs` 为 61，`grep -c '#\[test\]' crates/singlefs-verification/src/bin/e184_regular_layer_cost_counts_write_dump.rs` 为 21，转储 bin 经 `#[path]` 当模块引进主装置，两个加起来是 82。覆盖写链 u = 4、16 两档、块数上界与 `preflight` 的单测在登记修订 6、8、10 各写了验收。
- 变异：26 行追加进 `crates/mutations.tsv`（`grep -c '^[E]184 ' crates/mutations.tsv` 得 26，名字 V1 至 V7 与 V9 至 V27；V8 随修订 1 删去）。三个数（抓到 / 无效 / 没红）待提交时门禁 crates-mutation-replay 跑出，这一页没有变异结果。

```

**出处 `.claude/kb/experiments/184-常规层与大负载层的代价计数.md:509-525`（整段抄，未转述）**

```markdown
### 岔路表（问题单「负载规模与重叠」一行，对应岔路单 G1）

| 岔路 | 状态 | 还差什么 | 剩下的量能不能让它翻面 | 算出它的命令 |
|---|---|---|---|---|
| G1 常规层负载与冒烟层怎么分 | 够判：登记第十一节 G1 够判（规模、态数区间重叠、共用块键三项在随机历史与覆盖写链的主取样与 8.2 取样点上都有判定词，「不稳定」也算）这一段满足；覆盖写链 u = 16 的「共用块键」一格是 `not_planned`（取不出） | 没有必需的；长期库那一项登记条件没成立，没跑；点级复用键上界没做（对 9 条排不了计划的流） | 能：S 只含 14 条钉过的冒烟流，15 条没钉的流（含 `position-addressed-trees`，两段登记段 322 与 290 写）若算进来 S 上界会抬高，「态数区间重叠」的 200、400 步档和覆盖写链会从 partial、separate 向 overlap 动（推的）；100 步主取样本来就在门槛上；「共用块键」的 9 条取不出的流有算术上界（最大 0.000494），翻不了 independent | `grep -n 'name=overlap' <产物>`、`grep -n 'name=share_group' <产物>`、`grep -n 'blocks_full=not_planned' <产物>`、`grep -n 'name=share_not_planned' <产物>` |
| G2 | 这一段不罩（第二、三段） | J3 作废面、J2 冷吞吐与库增量 | — | — |
| G3 | 这一段不罩（第四段，等 G2） | 两种落法的固定开销 | — | — |
| G4 | 登记不罩 | — | — | — |

岔路单 G1 三个候选对应的登记翻面观测，这一段的读数（只摆读数，不判哪个候选站得住，那是推论）：

| 候选 | 登记的翻面观测 | 这一段的读数 |
|---|---|---|
| 甲：每条负载登记全量态数与最长段写数，层表规模分句写成区间 | 态数区间重叠：两族都重叠且 `above` 都小于 0.5，甲按数分层分不开（F1）；有一族 `above` 不小于 0.5，甲可以把常规层写成「高于冒烟层上界」那一截 | F1 不触发。`above` 占比：覆盖写链三档都是 1.000000，随机历史 50 步 0.416667、100 步 0.500000、200 步 0.800000、400 步 1.000000、盘宽 240 槽 0.000000、带整理比重 0.473684（第 927、931、933、935、937、939、941、943、945、947 行）；都是相对 S（14 条钉过的冒烟流）说的 |
| 乙：按负载来源分，冒烟层是手写脚本与登记的崩溃枚举用例，常规层是固定种子随机历史整段全量展开 | 共用块键（随机历史）：换名则乙名不副实；独立则乙加的是真的新块 | 随机历史六组都是 independent，共用块占比最大 0.001985；排得了计划的 112 条之外，8 条取不出（算术上界最大 0.000494，推的） |
| 丙：不分，常规层并进冒烟层 | 共用块键（覆盖写链）：换名则覆盖写链只是冒烟层换名；独立则并进冒烟层不只是换名，冒烟层要多判这些块 | u = 1、4 independent（0.002761、0.000060）；u = 16 取不出（算术上界 1.04e-9，推的） |

```

**出处 `.claude/kb/experiments/184-常规层与大负载层的代价计数.md:526-566`（整段抄，未转述）**

````markdown
### 它答不了的

- 区间 S 只含冒烟层 14 条钉过的流（十条钉住常量加四条重录），上界 2438004868。冒烟层登记流里还有没钉的 15 条（产物第 14 至 28 行），其中 `position-addressed-trees` 在 `.claude/gate.d/crash-point-replay-segments.tsv` 第 26、27 行登了两段，段写数 322 与 290，两态闭式至少 2^322 − 1（推的：一段 322 次两态写的闭式是 2^322 − 1），远在随机历史与覆盖写链全部流之上。「态数区间重叠」的 `above` 因此是相对 S 说的，不是相对冒烟层真实上界说的；S 算不算这条流、要不要补一份把按段登记的流也算进区间的读法，登记没写，这一页不定。
- 冒烟层其余 25 条登记流与候选共用多少块没量：只重录了四条，`library`（长期库里已录的同键块）登记条件没成立，没跑。
- 块键含版本表。随机历史 120 条里 78 条的最后一版没有文件（`versions_with_a_file=0 versions_without_a_file=24`），版本表取法换了，共用占比会变；复用键占比不含版本表与计划哈希，是上界，最大 0.053737。常规层真要落成时版本表另有取法（登记射程）。统计命令与输出：

  ```bash
  grep 'name=dump ' research/results/e184-regular-layer-cost-counts-segment-one-2026-10-19.out | grep -o 'versions_with_a_file=[0-9]* versions_without_a_file=[0-9]*' | sort | uniq -c | sort -rn | head -3
  ```

  ```text
       78 versions_with_a_file=0 versions_without_a_file=24
       28 versions_with_a_file=24 versions_without_a_file=0
        3 versions_with_a_file=12 versions_without_a_file=12
  ```
- 随机历史每档只有 20 个种子，覆盖写链每档只有 1 条流，没有分布：「族中位」「族最大」对覆盖写链就是那一条流；BROAD 比重、两块 4 GiB 盘、盘宽 240 槽、带整理比重、步数 50 至 400 之外的几何没量。
- 100 步主取样的「态数区间重叠」恰好站在门槛上（10 / 20），换一条流就是 partial；判定词 overlap 是登记门槛的字面结果，不是稳固的结果。
- 覆盖写链 u = 4、16 走顺序写（每次覆盖写是 u 个一单元事务），不是登记原文写的单事务多单元覆盖写（crates 里没有）；u = 16 的全量计划没排，要全量计划的三类对照没跑。
- 9 条排不了计划的流「共用块键」取不出。算术上界（推的，装置没量）：共用块数不超过冒烟层四条重录流的块数之和 46963，每条流至少有全量态数除以 65536 块：

```bash
grep 'name=flow ' research/results/e184-regular-layer-cost-counts-segment-one-2026-10-19.out | grep 'blocks_full=not_planned' | awk '{for(i=1;i<=NF;i++){if($i~/^label=/)l=substr($i,7); if($i~/^full=/)f=substr($i,6)} printf "%s full=%s blocks_at_least=%.0f share_at_most=%.3g\n", l, f, f/65536, 46963/(f/65536)}'
```

```text
overwrite_chain-u16 full=2954361402816462969 blocks_at_least=45079977459968 share_at_most=1.04e-09
random_history-broad-FourGibibytes-steps0200-seed12 full=9921447726162 blocks_at_least=151389278 share_at_most=0.00031
random_history-broad-FourGibibytes-steps0200-seed13 full=11663880818977 blocks_at_least=177976697 share_at_most=0.000264
random_history-broad-FourGibibytes-steps0200-seed18 full=6233485413873 blocks_at_least=95115439 share_at_most=0.000494
random_history-broad-FourGibibytes-steps0400-seed06 full=18690168524082 blocks_at_least=285189339 share_at_most=0.000165
random_history-broad-FourGibibytes-steps0400-seed12 full=10165605368686 blocks_at_least=155114828 share_at_most=0.000303
random_history-broad-FourGibibytes-steps0400-seed13 full=18429129377335 blocks_at_least=281206198 share_at_most=0.000167
random_history-broad-FourGibibytes-steps0400-seed14 full=731961382353473 blocks_at_least=11168844335 share_at_most=4.2e-06
random_history-broad-FourGibibytes-steps0400-seed18 full=15453626179150 blocks_at_least=235803622 share_at_most=0.000199
```

- 登记第八节末句的判别力自证（把门槛挪到两个取样点的值之间，判定必须换边）：装置单测里没有找到以两个取样点的值为输入的换边测试，这一格记「没核到」。
- 多机没用：第一段只在一台上跑，没有第二台的片；块键文件合计的磁盘量随步数档涨（100 步档 20 条流合计 15G），是单台跑的原因之一。
- 没有一个崩溃状态被判：这一段只数态数、块与键，不说常规层会不会红。
- 挂钟类的数（G2 的冷吞吐、库增量）这一段没量；产物里的 `keys_seconds` 是逐条流算计划与块键的用时，不是冷吞吐。

````

**出处 `.claude/kb/experiments/184-常规层与大负载层的代价计数.md:567-581`（整段抄，未转述）**

```markdown
### 路径与结论登记

结论是「装置里数出的态数，与 crates 里另一条路径、测试文件钉的常量、手算的式子一致」这一类：同一个量用几条互不共享代码的路径各算一遍再比对。

| # | 路径 | 怎么算 | 源码落点 | 读了哪些共用项 |
|---|---|---|---|---|
| 1 | 本地复算两态闭式 | 装置按 1 + Σ(2^段写数 − 1) 自己算，一段不少于 64 写或总和装不进 u64 交 `None`，每条流报 `two_state_agrees_with_crates` | `crates/singlefs-implementation-experiments/src/bin/e184_regular_layer_cost_counts/crash_point_plans_and_block_keys.rs:94 two_state_closed_form` | D13（验证路线） 已定项 4 的段模型与闭式写法；段由 crates 的切段函数切出，两条路径读同一份段 |
| 2 | crates 的两态闭式 | 调 harness 的 `closed_form_state_count`，有一段 64 写以上 panic | `crates/singlefs-verification/src/memory_pool.rs:659 closed_form_state_count` | 同一份切出来的段；同一条式子 |
| 3 | 全量态数：计划末节点序号尾 | `plan_crash_points` 排计划，取最后一个点的 `ordinals.end` | `crates/singlefs-verification/src/crash_amplification.rs:505 plan_crash_points` | 节点切法 `crash_points_of_the_steps`、`full_expansion`（原地覆写取第三态）、同一份写表与段 |
| 4 | 全量态数：按段区间计数 | `crash_state_count_with_torn_in_place_overwrites` 配 `full_expansion` 数 | `crates/singlefs-verification/src/crash.rs:2418 crash_state_count_with_torn_in_place_overwrites` | 同一个展开函数与原地覆写第三态规则；路径 3 与 4 错在同一条展开规则上会一起错 |
| 5 | 测试文件钉的常量 | 冒烟层四条重录流的两态闭式、全量、甲二与测试文件里钉的数比（停机 T3） | `crates/singlefs-verification/tests/crash_enumeration_new_pool_file_creation_stream.rs:774 FULL_STATES_WITH_TWO_STATES_PER_WRITE` | 同一份被测流；常量是写测试的人钉下的，不独立于当时的实现 |
| 6 | 手算 | 按段形 `[2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 16, 2, 1, 2]` 手写 python 一行算 A1 的 16842785、前 12 段得新池新建文件流的两态闭式 16777243（取号写一轮时段形少开头那一个 2，A1 得 16842782；「做了什么」第 6 条） | 无实现：登记第十三节第 13.4 条第 3 点的 `python3 -c` 一行换上取号写两轮之后的段形，没有存进仓的脚本 | 只读 D13（验证路线） 已定项 4 的闭式写法与段形，不读 crates |

跨路径的比对断言钉在源码和产物里：路径 1 对 2 是每条 `name=flow` 的 `two_state_agrees_with_crates=true`（产物 127 条）；路径 3 对 4 是 `T4_two_counting_routes_*` 与各 `keys_done` 的 `count_disagreements=0`；路径 5 是 `T3_recorded_values_*`；路径 6 对 1 是建快照流两态闭式 16842785 与新池新建文件流两态闭式 16777243 两个锚（第 2 次重跑产物的第 68、69 行 `match=true`，现算与现录相同；2026-10-19 的第一段产物里是取号写一轮那一版的 16842782、16777240）。结论的取值是枚举点，不是扫遍定义域：127 条流、四条冒烟流、一条建快照流手算，理由是登记只要求这几族。路径 1、2、3、4 共读同一份切段与展开规则，那两项错了它们会一起错，看不出来。

```

**出处 `.claude/kb/experiments/184-常规层与大负载层的代价计数.md:582-590`（整段抄，未转述）**

```markdown
### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 9 | 备料 | 2026-11-05 不受影响：只改用法与复跑说明（一个门禁、两种跑法；验证包改名），结论、数与产物不变；此前 2026-11-03 不受影响：只改复跑写法，结论、数与产物都不变；此前 2026-11-01 不受影响：只改复跑命令写法，结论、数与产物不变；此前 2026-10-26 不受影响：改的是过时的现状句与指路，结论与产物不变；此前 2026-10-24 不受影响：新段形上第一段整份重跑，判定词逐格同改前（窄读 40 格、宽读 62 格换词 0，见「第 2 次重跑」一节）；B1 现录 1358954642（重跑产物第 77 行），仍与射程里的 12230590578 对不上，F6 第一条照旧触发；该分项依据段第 269 行写着「无实验……E184（常规层与大负载层的代价计数） 第一段已量候选负载的规模与态数区间重叠，只备料」，关系维持备料。层表常规层「写请求数上千」：写数 ≥ 1000 的候选流仍都在冒烟层已钉流（最大 828 写）之上，在态数轴上随步数档仍从 overlap 走到 separate（inside 占比 0.923077、0.550000、0.350000、0.100000，400 步档正好在门槛端点上）。这几点交主 agent 判是否升成支撑或推翻、是否改射程句，决策正文这一次没有改。2026-10-23 不受影响：上游写序变化（取号写两轮）之后只修钉值，`selfcheck` 现录四条冒烟流与测试钉的数逐项相等、停机检查全过，第一段整份产物没重跑、判定词推的不翻；B1 现录 1358954642，与射程里的 12230590578 仍对不上；2026-10-19 不受影响：该分项的依据段没引 E184（常规层与大负载层的代价计数），这一页只备料；层表常规层行的「写请求数上千」在写数轴上分得开冒烟层 14 条已钉流（最大 824 写），在态数轴上随步数档从 overlap 走到 separate，射程里 12230590578 与今天的多记录流 1358954634 对不上（失败条款 F6 第一条）。这几点交主 agent 判是否升成支撑或推翻、是否改射程句，决策正文这一次没有改 |
| D13（验证路线） 已定项 17 | 备料 | 2026-11-05 不受影响：只改用法与复跑说明（一个门禁、两种跑法；验证包改名），结论、数与产物不变；此前 2026-11-03 不受影响：只改复跑写法，结论、数与产物都不变；此前 2026-11-01 不受影响：只改复跑命令写法，结论、数与产物不变；此前 2026-10-26 不受影响：改的是过时的现状句与指路，结论与产物不变；此前 2026-10-24 不受影响：重跑产物里 12 条 `name_not_in_key` 全部 `passed=true`，随机历史与冒烟层四条重录流共用块占比最大 0.002261（R 50 步，重跑产物第 945 行）；该分项依据段没引 E184（常规层与大负载层的代价计数），这一页只备料；这两点交主 agent 判是否升成支撑，决策正文这一次没有改。2026-10-23 不受影响：上游写序变化（取号写两轮）之后只修钉值，`selfcheck` 现录四条冒烟流与测试钉的数逐项相等、停机检查全过，第一段整份产物没重跑、判定词推的不翻；2026-10-19 不受影响：该分项的依据段没引 E184（常规层与大负载层的代价计数），这一页只备料；「名字不进键」在装置这一路上成立（12 条 `name_not_in_key` 全部 `passed=true`），随机历史与冒烟层四条重录流共用块占比最大 0.001985，这两点交主 agent 判是否升成支撑，决策正文这一次没有改 |
| D13（验证路线） 已定项 4 | 不影响 | 2026-11-05 不受影响：只改用法与复跑说明（一个门禁、两种跑法；验证包改名），结论、数与产物不变；此前 2026-11-03 不受影响：只改复跑写法，结论、数与产物都不变；此前 2026-11-01 不受影响：只改复跑命令写法，结论、数与产物不变；此前 2026-10-26 不受影响：改的是过时的现状句与指路，结论与产物不变；此前 2026-10-24 不受影响：重跑产物里四条重录冒烟流的两态闭式与全量态数和测试钉的常量逐项相等（停机 T3 八格 `result=pass`，`pinned_smoke_done` 的 `mismatches=0`），没有新结论。2026-10-23 不受影响：上游写序变化（取号写两轮）之后只修钉值，`selfcheck` 现录四条冒烟流与测试钉的数逐项相等、停机检查全过，第一段整份产物没重跑、判定词推的不翻；2026-10-19 不受影响：装置照这个分项的切段口径切段，冒烟层四条重录流的两态闭式与全量态数和测试钉的常量逐项相等（停机 T3 八条 pass），没有新结论 |
| D13（验证路线） 已定项 16 | 不影响 | 2026-11-05 不受影响：只改用法与复跑说明（一个门禁、两种跑法；验证包改名），结论、数与产物不变；此前 2026-11-03 不受影响：只改复跑写法，结论、数与产物都不变；此前 2026-11-01 不受影响：只改复跑命令写法，结论、数与产物不变；此前 2026-10-26 不受影响：改的是过时的现状句与指路，结论与产物不变；此前 2026-10-24 不受影响：重跑产物 13 条 `family_summary` 的 `flows_beyond_u64` 都是 0，没有一条流的两态闭式装不进 u64，按段登记与分片这一路没被碰到。2026-10-23 不受影响：上游写序变化（取号写两轮）之后只修钉值，`selfcheck` 现录四条冒烟流与测试钉的数逐项相等、停机检查全过，第一段整份产物没重跑、判定词推的不翻；2026-10-19 不受影响：13 条 `family_summary` 的 `flows_beyond_u64` 都是 0，没有一条流的两态闭式装不进 u64，按段登记与分片这一路在这一段没有被碰到 |

```

**出处 `.claude/kb/experiments/53-丢一整块盘之后根环还挂不挂得上.md:1-9`（整段抄，未转述）**

```markdown
## E53 丢一整块盘之后根环还挂不挂得上 —— 已跑（2026-08-30，虚机真盘，N=5 轮字节一致，6 单测 / 6 条变异全抓）：**判据全过，最坏回退 1 代**

**中文** · [English](53-can-root-ring-mount-after-losing-an-entire-disk.en.md) · [日本語](53-丸ごと1台のディスク喪失後にルートリングがマウントできるか.ja.md)

**这是本工程第一个真的碰设备的失败域实验。** E46（根环区域间距与失败域） / E47（根环失败域的损失） /
E48（根环几何的可行点） / E50（根环槽数的上下界） 四个全是纯算术模型
（**文件操作 0 处、故障注入 0 处**），它们判的是几何；E53（丢一整块盘之后根环还挂不挂得上）在**虚机里的真块设备**上写根、
**真的抹掉一整块盘**、再走一遍恢复的择根路径。

```

**出处 `.claude/kb/experiments/53-丢一整块盘之后根环还挂不挂得上.md:10-17`（整段抄，未转述）**

```markdown
### 它验的是哪一条，不验哪一条

| 验 | 不验 |
|---|---|
| E48（根环几何的可行点） 的可行点结论在真设备上复不复现：**丢一整块盘之后仍读得到有效根，且最坏回退 1 代** | **素数步长把区域摊到不同盘上**那一半——虚机里是 4 块独立裸盘、没有条带，区域与盘一一对应，那正是素数步长在条带阵列上要达成的理想放置 |
| 择根算法（先逐个验证全部候选、再按代号择新）在真 I/O 上的行为 | **物理失败域**（控制器 / 背板 / 电源 / 固件批次）——要真硬件，本机做不到 |
| 校验和能不能把被抹掉的槽判掉 | **EIO 形态的盘失效**：E53（丢一整块盘之后根环还挂不挂得上）的注入是「整盘内容被抹」，不是「读返回 EIO」。最小 initramfs 里没有 `dm-error` |

```

**出处 `.claude/kb/experiments/53-丢一整块盘之后根环还挂不挂得上.md:18-27`（整段抄，未转述）**

```markdown
### 要测什么

4 块盘 × 每盘一个区域 × 每区 2 槽 = 8 个根槽，**跨区轮转**（第 t 代写区域 `t mod 4` 的槽 `(t / 4) mod 2`）。

1. **阶段 A**：用 O_DIRECT 往 4 块真盘写 24 代根记录（转 3 圈），每条含 magic + 代号 + 槽号 + CRC32。
2. **阶段 B（真注入）**：把第 k 块盘承载的那个区域**整段写零**——真写，走同一条 O_DIRECT 路径。
3. **阶段 C**：重读全部 4 盘 8 槽，逐个验 magic 与 CRC32，在有效者中按代号择新。

报三个数：`survivors`（验得过的槽数）、`chosen_gen`（择出的代号）、`rollback = 最新代 − chosen_gen`。

```

**出处 `.claude/kb/experiments/53-丢一整块盘之后根环还挂不挂得上.md:28-35`（整段抄，未转述）**

```markdown
### 判据（跑前写死）

| # | 判据 |
|---|---|
| 1 | **丢一块盘之后 `survivors ≥ 1`**（挂得上）|
| 2 | **`rollback ≤ 1`**——E48（根环几何的可行点） 模型给的就是这个数，E53（丢一整块盘之后根环还挂不挂得上）看它在真设备上成不成立 |
| 3 | 四块盘**逐块各丢一次**，四次都要满足 1 与 2（不许只丢一块就下结论）|

```

**出处 `.claude/kb/experiments/53-丢一整块盘之后根环还挂不挂得上.md:36-44`（整段抄，未转述）**

```markdown
### 怎样才算失败（跑前写死，跑完不许改）

| 条款 | 触发时怎么判 |
|---|---|
| **阳性对照**：不注入时 `rollback` 必须为 **0**、`survivors` 必须为 **8** | 不是 ⇒ 写或读的路径有问题，**整轮作废** |
| **阴性对照**：把**四块盘全抹** ⇒ `survivors` 必须为 **0** | 非 0 ⇒ 它没在读盘（多半在读缓存），**整轮作废** |
| `rollback > 1` | **如实记录**——那是 E48（根环几何的可行点） 的模型结论在真设备上不成立，比「符合预期」有价值得多 |
| 虚机里读不到 `E7RESULT` 收尾行 | 按 `.claude/rules/command-safety-by-review.md`「结果抓取要有完整性闸」**整轮作废**，不许当成零条 |

```

**出处 `.claude/kb/experiments/53-丢一整块盘之后根环还挂不挂得上.md:45-52`（整段抄，未转述）**

```markdown
### 前置

- 虚机 harness（`scripts/devices/vm-bench.sh`），**E53（丢一整块盘之后根环还挂不挂得上）为它加了多盘支持**（`VM_DISKS=4`）；
  加完先跑 `--selftest`，通过之后才算数。
- 二进制必须是静态的（musl 目标）。
- ⚠️ **本机唯一可读的内核带 lockdep** ⇒ 虚机里的**时间不可与宿主比**；
  E53（丢一整块盘之后根环还挂不挂得上）只报**计数**（幸存槽数、代号），不报时间，不受此影响。

```

**出处 `.claude/kb/experiments/53-丢一整块盘之后根环还挂不挂得上.md:53-62`（整段抄，未转述）**

````markdown
### 口径与复跑

- 代码 `scripts/experiment-apparatus/e53_ring_failure.rs`。
- 复跑：
  ```
  task-server/task submit -- env VM_DISKS=4 bash scripts/experiment-apparatus/e53-vm-bench-run.sh <输出文件>
  ```
- 原始输出 `e53-ring-failure-2026-08-30.out`。
- 变异表 `scripts/mutation/tables/e53_ring_failure.tsv`。

````

**出处 `.claude/kb/experiments/53-丢一整块盘之后根环还挂不挂得上.md:63-88`（整段抄，未转述）**

```markdown
### 实测（2026-08-30，QEMU 4 块 virtio 真盘，全程 O_DIRECT，N=5 轮逐字节一致）

| 场景 | 幸存槽 | 择出的代号 | 回退 | 挂得上 |
|---|---|---|---|---|
| **阳性对照：不注入** | **8 / 8** | 23 | **0** | 是 |
| 抹掉 /dev/vda（区域 0） | 6 | 23 | **0** | 是 |
| 抹掉 /dev/vdb（区域 1） | 6 | 23 | **0** | 是 |
| 抹掉 /dev/vdc（区域 2） | 6 | 23 | **0** | 是 |
| **抹掉 /dev/vdd（区域 3，最新一代所在）** | 6 | **22** | **1** | 是 |
| **阴性对照：四块全抹** | **0** | NA | NA | 否 |

**判据逐条判定**：

| # | 判据 | 判定 |
|---|---|---|
| 1 | 丢一块盘后 `survivors ≥ 1` | **过**（四次都是 6）|
| 2 | `rollback ≤ 1` | **过**（最坏 1，出现在「最新一代恰好落在被抹的那个区域」那一次）|
| 3 | 四块盘逐块各丢一次都要满足 | **过**（四次全过）|
| 阳性对照 | 不注入 ⇒ 幸存 8、回退 0 | **过** |
| 阴性对照 | 全抹 ⇒ 幸存 0 | **过**——⇒ **它真的在读盘，不是在读缓存** |

⇒ **E48（根环几何的可行点） 的模型结论在真设备上复现**：跨区轮转 + 每区落在不同盘上，
丢一整块盘之后恒挂得上、最坏回退 1 代。
**回退 1 代只在「最新一代恰好落在被抹的那个区域」时出现**，
与 E47（根环失败域的损失） 模型里「对相位取最坏」算出的结果一致。

```

**出处 `.claude/kb/experiments/53-丢一整块盘之后根环还挂不挂得上.md:89-96`（整段抄，未转述）**

```markdown
### 2026-08-30 那次跑补的一条闸（变异测试打出来的）

`M3_magic不参与判定` **第一轮一个测试都没红**：被抹的槽是全零，
**CRC 就把它拦了** ⇒ 全零那条测试**考不出 magic 这道闸**。
补了一条造「**magic 错、而 CRC 自洽**」的记录的测试——只有 magic 拦得住它，补完 6 条全抓。
⚠️ **这不是纸面问题**：真实场景里一个陌生块（别的结构、上一次 mkfs 的残留）
完全可能自带一个自洽的校验和，**那时判定靠的就是 magic**。

```

**出处 `.claude/kb/experiments/53-丢一整块盘之后根环还挂不挂得上.md:97-116`（整段抄，未转述）**

````markdown
### 口径与已知局限

- 代码 `scripts/experiment-apparatus/e53_ring_failure.rs`，
  原始输出 `e53-ring-failure-2026-08-30.out`（9 条，收尾行 `emitted=9`）。
  变异表 `scripts/mutation/tables/e53_ring_failure.tsv`（6 条，全抓）。
- 复跑（E53（丢一整块盘之后根环还挂不挂得上）要虚机，不在 `replay.sh` 的快档里）：
  ```
  task-server/task submit -- env VM_DISKS=4 VM_DISK_MB=64 bash scripts/experiment-apparatus/e53-vm-bench-run.sh <输出文件>
  ```
- ⚠️ **注入形态是「整段内容被抹」，不是「读返回 EIO」**。最小 initramfs 里没有 `dm-error`，
  真正的设备消失（`device_del`）也没做。⇒ **E53（丢一整块盘之后根环还挂不挂得上）证明的是「内容没了还能不能择出根」**，
  不是「设备摘掉之后驱动层会怎样」。
- ⚠️ **区域与盘一一对应，虚机里没有条带**：E53（丢一整块盘之后根环还挂不挂得上）只验「区域确实在不同盘上」之后的后果。
  ⚠️ **那个前提已由 E55（条带映射的算术在真设备上对不对） 在真 dm-stripe 设备上补验**
  （`(p/chunk)%devs` 72/72 格全对；素数步长六组全部落在互不相同的盘上；
  非素数步长的阳性对照全部撞同盘）⇒ **前提与后果各自有了真设备证据。**
- ⚠️ **物理失败域**（控制器 / 背板 / 电源 / 固件批次）**仍然零覆盖**——要真硬件。
- 4 块盘 × 每盘 64 MiB；根区域起点 1 MiB、每槽 4096 字节、每区 2 槽、共 24 代（转 3 圈）。
- 虚机内核带 lockdep ⇒ **时间不可与宿主比**；E53（丢一整块盘之后根环还挂不挂得上）只报计数，不受影响。

````

**出处 `.claude/kb/experiments/53-丢一整块盘之后根环还挂不挂得上.md:117-124`（整段抄，未转述）**

```markdown
### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D5（快照 / 空间记账机制） 已定项 2 | 备料 | 2026-11-19 不受影响：只改复跑命令的写法（换成入口脚本），结论、数与产物不变；此前 2026-11-17 不受影响：只改指向规则小节的指针，结论、数与产物不变；此前 2026-10-21 不受影响：只加了三语切换行；此前 2026-10-11 不受影响：只改措辞（时间词、位置指代与自称），结论、数与产物不变 |
| D22（单元原子性怎么合成） 已定项 2 | 支撑 | 2026-11-19 不受影响：只改复跑命令的写法（换成入口脚本），结论、数与产物不变；此前 2026-11-17 不受影响：只改指向规则小节的指针，结论、数与产物不变；此前 2026-10-21 不受影响：只加了三语切换行；此前 2026-10-11 不受影响：只改措辞（时间词、位置指代与自称），结论、数与产物不变 |
| D16（发布语义） 已定项 6 | 支撑 | 2026-11-19 不受影响：只改复跑命令的写法（换成入口脚本），结论、数与产物不变；此前 2026-11-17 不受影响：只改指向规则小节的指针，结论、数与产物不变；此前 2026-10-21 不受影响：只加了三语切换行；此前 2026-10-11 不受影响：只改措辞（时间词、位置指代与自称），结论、数与产物不变 |

```

**出处 `.claude/kb/experiments/159-fsync等待时间随并发数组提交与wal两臂.md:1-15`（整段抄，未转述）**

```markdown
## E159 fsync 等待时间随并发数：组提交与 WAL 两臂 —— 装置已写、计时未跑（2026-09-24，接续会话；纯计数部分确定性、55 单测 / 17 条变异全抓；正式计时的主网格、PC1–PC3、AA、G8、G1、G6 五轮尚未跑；装置的 `RECORD_HEADER_BYTES` 仍是 311、点名项钉 67，头 496、点名项 64 之后没跟着重跑，欠账 C776（E155、E159 装置停在头 311 与点名项 67））

**中文** · [English](159-fsync-wait-time-vs-concurrency-group-commit-and-wal-arms.en.md) · [日本語](159-並行数に伴うfsync待機時間-グループコミットとWALの2アーム.ja.md)

<!-- doc-lint:registry name-col=2 -->

| 编号 | 简称 |
|---|---|
| J1 | 甲在某个 k 越过 M |
| J4 | p99 在某个 k 越过 M |

**问题**：跑前登记 `e159-preregistration.md`——岔路单 `m2-fsync-wait-forks.md` 第 1 行「甲（今天的做法：每次 fsync 写脏叶加全部祖先，按 D16（发布语义） 已定项 10 攒批、组提交）的等待时间可不可以接受」，量「一次 fsync 从发出到返回等了多久」在甲与两条 WAL 臂（wal_full-K10、乙-M-K10，两条都留、各判各的）上随并发数 k 的走法。装置写在 `scripts/experiment-apparatus/e159_fsync_wait_group_commit.rs`（独立手写模型，不是入库装置——`crates/` 里没有组提交，第三节 2.3 现查零命中）。量延迟用真跑（真机、真 O_DIRECT 设备、真线程），不用计数模型换算，理由见跑前登记「五、5.1」。

**这一段（接续一个撞了会话限额而中断的执行员）只做到「装置与冒烟」**：正式计时（本机口径下的 5 轮主网格、PC1–PC3、AA、G8、G1、G6）没有跑——机器上这会儿有并发编译，计时会被污染（`.claude/kb/tooling.md`「仓里的性能数都只在本机成立」同一类顾虑），等机器空闲另派。

```

**出处 `.claude/kb/experiments/159-fsync等待时间随并发数组提交与wal两臂.md:16-27`（整段抄，未转述）**

```markdown
### 这一段做了什么

1. **装置**：从 `scripts/experiment-apparatus/e155_fourth_run_group_commit_concurrency.rs` 第 36–2058 行原样拷来常量、K1–K4、5.3 组公式与三条臂的单发/批量函数（跑前登记停机条款 S2 要求这段与源文件逐个 diff 为空，已核，见「复跑」）；新写组提交队列（`SharedQueue`、`drain_batch_from_queue`）、WAL checkpoint 线程（`checkpoint_loop`、`should_trigger_checkpoint`）、闭环调用方（`caller_loop`）、O_DIRECT I/O（`RealBlockDevice`）、结构性单测用的假设备（`RecordingBlockDevice`、`SleepingBlockDevice`）、统计（分位数、Mann–Whitney A、规则 R、J1（甲在某个 k 越过 M）/J2a/J4（p99 在某个 k 越过 M） 判定）、PC1 忙等注入、`cell`/`probe-device`/`anchors` 三个子命令。
2. **停机条款 S1（锚点格与 `crates/` 逐项对拍）**：单测钉死甲在 F1、P=1、k=1 的单盘总字节 172544、写调用 11 次（`jia_single_fsync_batch_matches_crates_test_table`，与第三节 `crates/` 测试表逐项加总一致），发布次序单元→屏障→记录→屏障→根槽 FUA→系统配置槽（先读两次、再写一次）逐步核对（`jia_single_fsync_batch_step_sequence_matches_crates_order`），根槽持久时刻不早于根槽 FUA（`jia_durable_instant_is_not_earlier_than_the_root_slot_fua`）。S1 不触发。
3. **停机条款 S2（拷贝块 diff 为空）**：`diff <(sed -n '39,2061p' e159_...) <(sed -n '36,2058p' e155_...)` 退出码 0。
4. **停机条款 S3（来宾 `logical_block_size`）**：真机 VM 探针 `probe-device` 子命令读回 `logical_block_size=512`，与主几何 `physical_block_size_bytes` 常量一致，不触发。
5. **单测与变异**：54 个单测全绿（`cargo test --release -p e7-index-bench --bin e159-fsync-wait-group-commit`）；变异表 `scripts/mutation/tables/e159_fsync_wait_group_commit.tsv` 14 条（M1–M14，覆盖第九节全部目标：甲发布去掉第二道屏障、第一道屏障后就放行、WAL 臂写混规划函数、组提交开也把批封顶为 1、`t_issue` 漏排队段、PC1 忙等挪到放行之后、随机取整改成向下取整、`T_time` 常量读错、系统配置槽写前不读、根槽宽度改错、J1（甲在某个 k 越过 M） 边界比较符改错、五轮合并改成多数轮说了算、开环 `issued_at` 改取实际发出时刻、`release_to_wake` 漏唤醒段），`mutate.sh` 跑两遍（源码改动前后各一遍）都 14/14 全抓、收尾「已还原，基线仍全绿」。M3–M6（组提交队列与真并发时序相关的四条）原本没有单测覆盖真并发引擎，这一段新增 6 条单测补齐：`plan_batch_for_arm_routes_each_arm_to_its_own_solver_not_a_different_one`（M3）、`drain_batch_from_queue_collects_everything_when_group_commit_is_on` / `_collects_only_one_when_group_commit_is_off`（M4）、`closed_loop_queueing_delay_is_counted_when_group_commit_is_off`（M5，用 `SleepingBlockDevice` 固定屏障延迟逼出排队，不需要真磁盘）、`pc1_injected_wait_lands_before_release_not_after`（M6，同样用 `SleepingBlockDevice`）、`checkpoint_time_threshold_constant_is_five_seconds`（M8，把 `T_time` 常量提成 `CHECKPOINT_TIME_THRESHOLD` 具名常量，`run_cell_subcommand` 与单测共用同一个落点，不再各写一份字面量）。
6. **真机冒烟撞见一个真 bug，已修复**：`RealBlockDevice::read_at` 原实现把数据读进一块普通 `vec![0u8; length]`（不对齐）再传给 O_DIRECT 的 `read_exact_at`，第一次真跑（`--arm jia --k 1024`）当场 `panicked ... EINVAL`；这条路径在 2026-09-24 修复之前 0 个单测覆盖到（全部旧单测只用 `RecordingBlockDevice`/`SleepingBlockDevice`）。改法：直接把数据读进按 `ALIGNMENT_BYTES` 对齐的 `AlignedBuffer`（新增 `AlignedBuffer::as_mut`），读完再拷出返回值；修复后同一条命令换成 k=4 不再 panic。产物见「结果整行抄自产物」。
7. **真机冒烟：三条臂各跑通一格**（`VM_DISK_PREALLOCATE=1`，短 duration，只为验证装置真跑不崩、不为出正式数）：甲 k=1、wal_full-K10 k=2（duration 4s，未到 `T_time`=5s 与 `T_dirty`=256MiB，checkpoint_count=0，符合预期）、乙-M-K10 k=2（duration 6s，触发一次 checkpoint，checkpoint 机制真跑通过）。三条臂、组提交引擎、WAL checkpoint 线程都验证过端到端能跑通，没有再撞到新的 panic。
8. **`print_anchors` 重构**：改成经 `Emitter` 发 `E7RESULT` 行、带 `name=done emitted=N` 收尾（2026-09-24 重构之前是裸 `println!`，不满足 `scripts/experiment-apparatus/replay.sh` 的完整性闸），并把甲那一行的单盘字节算法从 `outcome.total_bytes() / DEVICE_COUNT`（对根槽字段是错的近似，会把单盘专属的 512 字节根槽也腰斩）改成与单测共用的 `jia_outcome_single_device_bytes`（挪成模块顶层函数），输出从 172288 改回正确的 172544，与 B4 锚点一致。
9. **命名纪律**：`naming-lint.sh` 对这个文件全程通过（唯一一条是 `naming-lint:external` 标注过的 `O_DIRECT` 外部标志名，不算违规）。

```

**出处 `.claude/kb/experiments/159-fsync等待时间随并发数组提交与wal两臂.md:28-48`（整段抄，未转述）**

````markdown
### 结果整行抄自产物（产物已移出工作区的行写成一行指针）

`e159-fsync-wait-group-commit-2026-09-24-anchors.out`（纯算术、逐字节可复跑，见「复跑」）：

```text
E7RESULT name=anchor label=jia_p1_f1_k1_single_device_bytes value=172544
E7RESULT name=anchor label=write_ahead_log_full_k10_p1_f1_k1_single_device_bytes value=102400
E7RESULT name=anchor label=yi_m_k10_p1_f1_k1_single_device_bytes value=86016
E7RESULT name=anchor label=write_ahead_log_full_k10_f8a_p10000_k1_single_device_expected_bytes value=365346.000
E7RESULT name=anchor label=write_ahead_log_full_k10_f8a_p10000_k16_single_device_expected_bytes value=4313362.000
E7RESULT name=done emitted=6
```

`e159-fsync-wait-group-commit-2026-09-24-smoke.out`（真机冒烟存证，不参与 `replay.sh` 的逐字节比对——真延迟每次跑都不同——只证明装置真跑得通；完整上下文见该文件）：修复之后三条臂各一格的 `E159RESULT` 行，例如乙-M-K10 那一格：

```text
产物行见 git show 5d0b7e7c1667:research/results/e159-fsync-wait-group-commit-2026-09-24-smoke.out 第 175 行（E7RESULT E159RESULT arm=yi-M-K10）
```

`containment_violations=0`（V5 包含自检没有违反）、`checkpoint_count=1`（WAL checkpoint 机制真触发过一次）。

````

**出处 `.claude/kb/experiments/159-fsync等待时间随并发数组提交与wal两臂.md:49-65`（整段抄，未转述）**

````markdown
### 复跑

```
bash scripts/experiment-apparatus/replay.sh E159
```

只登记了 `anchors` 子命令这一行（纯算术、不碰真设备、逐字节可复跑）：`scripts/experiment-apparatus/replay.sh` 里 E159（fsync 等待时间随并发数：组提交与 WAL 两臂） 的 `anchors` 那一行，产物 `e159-fsync-wait-group-commit-2026-09-24-anchors.out`、判法 `exact`，本次交回前跑过、判定「字节一致」。真机冒烟（S3 探针与三条臂各一格）不登记进 `replay.sh`——延迟数每次跑都不同，没有能钉住的逐字节判据；复跑命令原样存进 `e159-fsync-wait-group-commit-2026-09-24-smoke.out` 文件头，需要重跑时手动执行，例如：

```
task-server/task submit -- bash scripts/experiment-apparatus/e159-vm-bench-run.sh <输出文件> probe-device --device /dev/vda
task-server/task submit -- env VM_DISK_PREALLOCATE=1 bash scripts/experiment-apparatus/e159-vm-bench-run.sh <输出文件> cell --device /dev/vda \
  --arm yi-m --k 2 --group-commit on --warmup-seconds 0.3 --duration-seconds 6 --seed 5 \
  --file-count 10000 --family f8a
```

单测：`cargo test --release -p e7-index-bench --bin e159-fsync-wait-group-commit` → `55 passed; 0 failed`（2026-09-24 头 307 → 311 重跑新增 1 条）。变异：`cd scripts/experiment-apparatus && bash ../mutation/mutate.sh e159-fsync-wait-group-commit e159_fsync_wait_group_commit.rs ../mutations/e159_fsync_wait_group_commit.tsv` → 17/17 全抓（新增 M15/M16/M17），收尾「已还原，基线仍全绿」（跑了两遍，源码改动前后各一遍，结论不变）。

````

**出处 `.claude/kb/experiments/159-fsync等待时间随并发数组提交与wal两臂.md:66-73`（整段抄，未转述）**

```markdown
### 它答不了的

- **岔路单第 1 行的翻面观测还没有答案**：正式计时（5 轮主网格 × 3 臂 × 8 个 k、PC1–PC3、AA、G8、G1、G6）一格都没跑，J1（甲在某个 k 越过 M）/J2a/J4（p99 在某个 k 越过 M） 三句判据都还没有值。这一段只证明了装置能真跑通、字节口径与 `crates/` 对得上、变异表全抓。
- **S4 写量预算（跑前登记 5.7）没有算清**：只有零散的冒烟读数（例如甲 k=4、duration 3s 约写 58.5 MB，折合约 17.7 MB/s，但这是在稀疏盘首次触碰扇区、没做「测量之前整盘写一遍」那一步的读数下量到的，真实预热之后的吞吐可能明显更高、真实字节总量因此可能被这个数低估）；没有按第一段全部格子（主网格 24 格 + PC1/PC2/PC3/AA/锚点 + G8 24 格 + G1 12 格 + G6 8 格，每格 11s 或 2s，5 轮）逐格累计算出精确的预算总量，也就没有确认 1 TiB 那道停机线会不会触发。下一段跑正式计时之前要先把这笔账算清、或者先做一次预热盘上的代表格实测再推算。
- **「测量之前整盘写一遍」（跑前登记 5.3）这一步还没有实现进装置**：本段冒烟只用 `VM_DISK_PREALLOCATE=1`（`fallocate` 预分配，不是整盘写满），第一次不带这个参数的冒烟因此撞见 samples=0（不是 bug，是稀疏盘首次触碰扇区的宿主 ext4 元数据开销），修复 read_at 之后没有验证过「整盘写满一遍」这一步本身。
- **G8 开环泊松到达、G1 互不共享、G6 `T_time=200ms` 三个反向取样点都还没跑**，第八节的「方向相反的取样点」判别力自证因此也没做。
- **PC1/PC2/PC3/AA 四个阳性对照与噪声地板都没有正式跑过**（只有非正式的冒烟：乙-M-K10 那一格触发过一次真 checkpoint，证明 checkpoint 机制通，不是 PC 系列的正式判定）。

```

**出处 `.claude/kb/experiments/159-fsync等待时间随并发数组提交与wal两臂.md:74-87`（整段抄，未转述）**

```markdown
### 影响的决策

这一段只交装置与冒烟，没有计时结论；「影响的决策」表里每条分项的「依据」段都没有引用 E159（fsync 等待时间随并发数：组提交与 WAL 两臂）（逐条 `grep -n` 查各条决策文件，零命中），按格式演进纪律都记「备料」，等正式计时跑完再回看要不要升级成支撑或推翻。

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D23（journal 的角色与格式） 已定项 1 | 备料 | 2026-11-19 不受影响：只改复跑命令的写法（换成入口脚本），结论、数与产物不变；此前 2026-10-22 不受影响：只改现状句里的格式宽度；此前 2026-10-15 不受影响：整行抄的产物行换成指向产物的指针，数、结论与产物不变 |
| D16（发布语义） 已定项 2 | 备料 | 2026-11-19 不受影响：只改复跑命令的写法（换成入口脚本），结论、数与产物不变；此前 2026-10-22 不受影响：只改现状句里的格式宽度；此前 2026-10-15 不受影响：整行抄的产物行换成指向产物的指针，数、结论与产物不变 |
| D16（发布语义） 已定项 5 | 备料 | 2026-11-19 不受影响：只改复跑命令的写法（换成入口脚本），结论、数与产物不变；此前 2026-10-22 不受影响：只改现状句里的格式宽度；此前 2026-10-15 不受影响：整行抄的产物行换成指向产物的指针，数、结论与产物不变 |
| D16（发布语义） 已定项 7 | 备料 | 2026-11-19 不受影响：只改复跑命令的写法（换成入口脚本），结论、数与产物不变；此前 2026-10-22 不受影响：只改现状句里的格式宽度；此前 2026-10-15 不受影响：整行抄的产物行换成指向产物的指针，数、结论与产物不变 |
| D16（发布语义） 已定项 10 | 备料 | 2026-11-19 不受影响：只改复跑命令的写法（换成入口脚本），结论、数与产物不变；此前 2026-10-22 不受影响：只改现状句里的格式宽度；此前 2026-10-15 不受影响：整行抄的产物行换成指向产物的指针，数、结论与产物不变 |
| D16（发布语义） 已定项 11 | 备料 | 2026-11-19 不受影响：只改复跑命令的写法（换成入口脚本），结论、数与产物不变；此前 2026-10-22 不受影响：只改现状句里的格式宽度；此前 2026-10-15 不受影响：整行抄的产物行换成指向产物的指针，数、结论与产物不变 |
| D25（目标负载优先级） 已定项 7 | 备料 | 2026-11-19 不受影响：只改复跑命令的写法（换成入口脚本），结论、数与产物不变；此前 2026-10-22 不受影响：只改现状句里的格式宽度；此前 2026-10-15 不受影响：整行抄的产物行换成指向产物的指针，数、结论与产物不变 |

```

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:1-51`（整段抄，未转述）**

````markdown
## E185 GC一批搬迁的挂钟拆分 —— 第 2 次重跑已跑（占比 0.25 的几何改负载后造成了，单点核与第一、二段跑完；问题单四行都够判：第 1、2 行读这一趟，第 3、4 行沿用第 1 次重跑的产物；第三、四段够判后不跑）

**中文** · [English](185-wall-clock-breakdown-of-a-batch-of-gc-migrations.en.md) · [日本語](185-GC-一バッチ分の移動の壁時計分解.ja.md)

<!-- doc-lint:not-numbers Q1 Q2 Q3 Q4 Q5 Q6 Q7 Q8 Q9 Q10 Q11 Q12 Q13 Q14 Q15 Q16 Q17 G1 G2 G3 G4 G5 G6 V1 V15 V16 V21 V22 V23 V26 V27 V30 V31 V33 V34 V55 V56 V94 F1 F2 F3 F4 F5 F7 F8 N4 N5 N6 N7 N8 N9 N10 N11 N12 N13 N14 N15 H1 H2 H3 H4 H5 H6 H7 H8 R2 -->

跑前登记 `research/prompts/e185-preregistration.md`（四段：第一段 `wall-clock`、第二段 `stage-split`、第三段 `gpu-submission`、第四段 `foreground-fsync`，第四段附带、够判后不跑）。回答问题单 `research/prompts/m4-gpugc-measure-forks.md` 的四行：一批搬迁的挂钟受盘还是 CPU 限、GPU 上算 GC 的哪几段（甲 只算校验和核对 ／ 乙 加区域扫描里的解头与映射反查 ／ 丙 再加选段打分）、攒批形态（一批一交 ／ 几批攒起来一次交 ／ 放宽 R）、定时兜底间隔与写带宽上限的默认值从什么量取。被测的是 `crates/` 今天的 GC 路径 `MountedSession::run_compaction`；背景是欠账 C412（整理的挂钟代价没量过），被测条款是 D26（后台整理与放置回收） 已定项 13、已定项 1 与 D24（后台重活能不能卸给 GPU） 已定项 3 的判据形式。

正文是第 2 次重跑的结果：重跑登记 `research/prompts/e185-r2-prereg.md`（sha256 前缀 `7d74c089`，写于 2026-10-23，装置改之前、这一趟的产物一份都还没跑）。起因：第 1 次重跑里 f = 0.25 两点几何没造成，问题单第 1、2 行按字面不够判；第一段里还有一段连成一片的慢段（同一份活 CPU 涨到 1.13–2.5 倍，作废后当场重试的那条 2.316 倍被收下，登记第四节第 5 行，第 161 行）。登记改了三样：负载（改法甲：只换造池第 2 步覆盖写的次序，让占比 0.25 的几何造得出）、装置的读数（每条意图前后的校准、线程与机器状态读数，造池之后逐段打成分）、作废与重试的做法（校准比接进「机器被别的活挤」，作废后先等机器回到常态再重试，第十一节修订 R2-1 至 R2-4，原判据原样保留）；另有执行员在单测之前、产物之前写的修订一（R2-5 至 R2-9，只收严或补量，登记第 415–425 行）。Q1–Q17 每一行的算法、门槛、判定照原登记（含第十二节 12.1–12.11）一个字不改。问题单第 3、4 行第 1 次重跑已够判，这一趟不再判，沿用第 1 次重跑的产物（岔路表写明哪几份）；第三、四段这一趟不跑。第 1 次重跑的数与七·6 前后的比法（重跑登记 `research/prompts/e185-r1-prereg.md`）已挪进变更史，正文只写这一趟。

装置是入库装置 `crates/singlefs-implementation-experiments/src/bin/e185_gc_batch_wall_clock_split/main.rs`（一个二进制、五个子命令：`wall-clock`、`stage-split`、`gpu-submission`、`foreground-fsync` 与这一趟新加的 `geometry-check`，只读驱动 `singlefs-core` 的公开入口，不改 `crates/` 里别的文件）。决定工作量、几何与判据门槛的常量都是本地常量，值抄自登记第五节常量表，每个子命令开跑先与 `crates/` 同名常量回比（停机条款第 1 条）；这一趟新加一个本地常量 `LOCAL_ROOT_RING_ENTRIES = 24`，回比 `singlefs_format::ROOT_RING_REGIONS × ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM`（装置第 131 行、第 178–180 行），只用来打机理核对那一栏。

**现状**：

- 装置现在是 sha256 `d312b9aa71d0518976fd383b48300705f4598d2e79d4a3d59488657e684a51eb`，26562 行，`grep -c '#\[test\]'` 对它 = 240；它在登记之后改过，登记第十二节没有修订行；三份起跑脚本的 `registered_apparatus_sha256` 仍各是自己登记第十二节写的值（第 2 次重跑 `0bd22743…`、第 3 次重跑 `4368c0e7…`、第 4 次重跑 `9e813cbd…`），与装置现算的不等：再起第 4 次重跑照停机 K-S6、第 3 次重跑照停机 J-S7 退 3，第 2 次重跑由装置里的停机条款第 7 条（修订 R2-8）退 3。第 2 次重跑那三趟用的是 `0bd2274383f12e88beab3f96f354148698cfcd2043263b5d5709e32e1946784d`，13187 行、118 条（改前基线是第 1 次重跑第三段用的 `eadb929d…`，10226 行、87 条）。三趟产物的 `name=apparatus` 行里，编进二进制的源码哈希与起跑脚本设给装置的登记哈希都是 `0bd22743…`（停机条款第 7 条与修订 R2-8 没触发），单点核两份与第二段同形，第一段如下（第一段第 2 行）：

```text
E7RESULT name=apparatus source_sha256=0bd2274383f12e88beab3f96f354148698cfcd2043263b5d5709e32e1946784d registered_sha256=0bd2274383f12e88beab3f96f354148698cfcd2043263b5d5709e32e1946784d
```

- 单测与变异（登记第十二节第 427–431 行，量的是主 agent 在一次性编译目录里跑的，这一页没重跑）：单测 `118 passed; 0 failed`，clippy 退 0；`crates/mutations.tsv` 里 E185（GC一批搬迁的挂钟拆分） `V` 起头的行现在 94 条（在 `crates/mutations.tsv` 里按该前缀 `grep -c` 输出 94）：V1–V15 照原登记第九节，V16–V55 是第 1 次重跑执行员补的，V56–V94 共 39 行是这一趟补的（V56–V76 登记第九节，V77–V94 修订一 R2-9），这 39 行的轻型变异全部抓到（登记第 431 行：计数「抓到 314 条、没红 0 条、无效 0 条」，39 行都在抓到里）；整表的三个数待提交时门禁 crates-mutation-replay。
- 这一趟的产物（都在 `research/results/`，行数用 `grep -c ''` 数，末行都是 `name=done`）：
  - 单点核：`research/results/e185-gc-batch-wall-clock-split-geometry-check-r2-2026-10-23.out`（125 行，sha256 `c47db8bf3a7841d65aa209c3a3fb4ba941e9c30ec68672cde1b0fc480c206e00`，末行 `E7RESULT name=done emitted=125`），对照 `research/results/e185-gc-batch-wall-clock-split-geometry-check-slot-order-r2-2026-10-23.out`（125 行，sha256 `0add58dbc58f0948d0b115d6575173494aca801550508c2d5d732ad38885a5c2`，末行 `E7RESULT name=done emitted=125`），输入清单 `research/results/e185-r2-inputs-geometry-check-2026-10-23.txt`，清场记录 `research/results/e185-r2-quiesce-geometry-check-2026-10-23.txt`。
  - 第一段：`research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-2026-10-23.out`（2103 行，sha256 `18191ec3a8afbb27405987f99d3e36e5ae2538d3c9ab4f2717738495e767096b`，末行 `E7RESULT name=done emitted=2103`），输入清单 `research/results/e185-r2-inputs-wall-clock-2026-10-23.txt`，清场记录 `research/results/e185-r2-quiesce-wall-clock-2026-10-23.txt`。
  - 第二段：`research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out`（1803 行，sha256 `f70eb21b8a7081489dd8fc3209080f3ab83f16ac40beee3042c03f4756f1e07a`，末行 `E7RESULT name=done emitted=1803`），输入清单 `research/results/e185-r2-inputs-stage-split-2026-10-23.txt`，清场记录 `research/results/e185-r2-quiesce-stage-split-2026-10-23.txt`。
  - 沿用的（第 3、4 行，不重算）：第三段 `research/results/e185-gc-batch-wall-clock-split-gpu-submission-r1-2026-10-22.out`（sha256 `4041a1cac83e7d62ef8bbe1580468aa6b4e43d69d67e8e9e044dd885ab063408`）与它读的第 1 次重跑第一段 `research/results/e185-gc-batch-wall-clock-split-wall-clock-r1-2026-10-22.out`（sha256 `978ae7b5fe60ab8f5a87338b982e8989c4c0d0ba8f19994bfc8e1c39b3d437fd`），输入清单 `research/results/e185-r1-inputs-gpu-submission-2026-10-22.txt`、`research/results/e185-r1-inputs-2026-10-22.txt`。
  - 存档不引（第 1 次重跑与 2026-10-20 那一趟的产物，数在变更史）：`research/results/e185-gc-batch-wall-clock-split-stage-split-r1-2026-10-22.out`、`research/results/e185-r1-comparison-2026-10-22.out`（比法判决行 `output_void=false`，即那一份没作废）、`research/results/e185-gc-batch-wall-clock-split-wall-clock-2026-10-20.out`、`research/results/e185-gc-batch-wall-clock-split-stage-split-2026-10-20.out`。
  - 2026-10-24 第 3 次与第 4 次重跑产物（三方论证备料，出处见 `research/prompts/e185-r3-prereg.md` 与 `research/prompts/e185-r4-prereg.md`）：`research/results/e185-gc-batch-wall-clock-split-cpu-writings-r4-2026-10-24.out`、`research/results/e185-gc-batch-wall-clock-split-cpu-writings-r4-2026-10-24.out.err`、`research/results/e185-gc-batch-wall-clock-split-function-split-r3-2026-10-24.out`、`research/results/e185-gc-batch-wall-clock-split-function-split-r3-2026-10-24.out.err`、`research/results/e185-gc-batch-wall-clock-split-host-cpu-drift-control-r4-2026-10-24.out`、`research/results/e185-gc-batch-wall-clock-split-host-cpu-drift-control-r4-2026-10-24.out.err`、`research/results/e185-gc-batch-wall-clock-split-host-cpu-per-submission-r3-2026-10-24-2.out`、`research/results/e185-gc-batch-wall-clock-split-host-cpu-per-submission-r3-2026-10-24-2.out.err`、`research/results/e185-gc-batch-wall-clock-split-host-cpu-per-submission-r3-2026-10-24.out`、`research/results/e185-gc-batch-wall-clock-split-host-cpu-per-submission-r3-2026-10-24.out.err`、`research/results/e185-gc-batch-wall-clock-split-recompute-disagreement-r3-2026-10-24.out`、`research/results/e185-gc-batch-wall-clock-split-recompute-disagreement-r3-2026-10-24.out.err`、`research/results/e185-r3-inputs-function-split-2026-10-24.txt`、`research/results/e185-r3-inputs-host-cpu-per-submission-2026-10-24-2.txt`、`research/results/e185-r3-inputs-host-cpu-per-submission-2026-10-24.txt`、`research/results/e185-r3-inputs-recompute-disagreement-2026-10-24.txt`、`research/results/e185-r3-quiesce-host-cpu-per-submission-2026-10-24-2.txt`、`research/results/e185-r3-quiesce-host-cpu-per-submission-2026-10-24.txt`、`research/results/e185-r3-quiesce-recompute-disagreement-2026-10-24.txt`、`research/results/e185-r4-baseline-rejudge-2026-10-24.out.gz`、`research/results/e185-r4-inputs-cpu-writings-2026-10-24.txt`、`research/results/e185-r4-inputs-host-cpu-drift-control-2026-10-24.txt`、`research/results/e185-r4-quiesce-host-cpu-drift-control-2026-10-24.txt`、`research/results/e185-r4-serial-cpu-writings-2026-10-24.txt`、`research/results/e185-r4-serial-host-cpu-drift-control-2026-10-24.txt`。
- 跑在哪：单点核、第一段、第二段在第二台先后跑，不同时跑，真盘臂的镜像放在同一块盘的 ext4 上（输入清单第 22–26 行：型号 CT2000T500SSD8，逻辑块 512，文件系统 ext4；第一段配置行 `image_directory_filesystem=ext4 image_directory_is_cleared_at_boot=false image_device=64512`），不带 `gpu` 特性编（三份输入清单里 `grep -c lookup-work-counters` 都输出 0），外层内存上限 46G、`E185_MEMORY_CAP=40G`、装置线程 31（第一段配置行 `worker_threads=31`）。三份输入清单第 1–11 行（装置、`transaction.rs`、`mounted_session.rs`、`region_scan.rs`、`compaction_plan.rs`、`unit.rs`、`checksum.rs`、`block_device.rs`、`allocator.rs`、`singlefs-format/src/lib.rs`、`Cargo.lock` 的 sha256）逐行相同（`diff` 核过），第 12 行是各自拷树后的提交号。每趟整条命令经 `scripts/crash/gpu-quiesce-run.sh` 起，清场记录原样：

```text
GPU_QUIESCE step=quiesce result=stopped-and-read-back holders=0
GPU_QUIESCE step=command result=exit-0
GPU_QUIESCE step=restore result=not-restored holders=1
GPU_QUIESCE step=exit code=0
```

  单点核那一份 `step=quiesce result=stopped-and-read-back holders=0`、`step=restore result=not-restored holders=0`；第二段那一份第一行是 `step=quiesce result=joined-and-read-back holders=1`（别的会话已登记同用卡，加入而不是新停）；三份都是 `step=command result=exit-0`、`step=exit code=0`。清场回读没过会退 78、命令不起，三趟都没有。
- 判决行（单点核两份、第一段、第二段，整行抄）：

```text
E7RESULT name=verdict segment=geometry_check constant_mismatches=0 overwrite_order=highest_full_segment_last points=2 geometry_formed_points=2
E7RESULT name=verdict segment=geometry_check constant_mismatches=0 overwrite_order=slot_order points=2 geometry_formed_points=1
E7RESULT name=verdict segment=wall_clock constant_or_anchor_mismatches=0 stop_clause_findings=0 positive_control_failures=0 positive_control_failure_items=none discrimination_failures=0 clause_wall_clock_not_rising_with_the_batch_triggered=0 clause_two_cpu_paths_disagree_points=0 gc_to_sequential_write_bandwidth_maximum_ratio=0.015977 gc_to_sequential_write_bandwidth_judgement=single_run_enough stored_intents=140
E7RESULT name=verdict segment=stage_split constant_or_anchor_mismatches=0 stop_clause_findings=0 positive_control_failures=0 positive_control_failure_items=none
```

  四行里取值 `false` 的字段没有，也没有 `not_run`，名字表示违例、不匹配、失败的计数（`constant_mismatches`、`constant_or_anchor_mismatches`、`stop_clause_findings`、`positive_control_failures`、`discrimination_failures`、两条失败条款的触发数）都是 0；大于 0 的整数只有 `stored_intents=140` 与 `points`/`geometry_formed_points`：旧造法对照那一份 `geometry_formed_points=1`（两点里只造成了占比 0.75 那一点）是对照的预期，它重现了第 1 次重跑的「没造成」，改法甲那一份是 `geometry_formed_points=2`。`gc_to_sequential_write_bandwidth_maximum_ratio=0.015977` 是 Q16 的值不是失败计数。
- 起跑脚本：三趟当时跑的是仓根 `e185-r2-tools/` 下的 `run-geometry-check.sh`、`run-wall-clock.sh`、`run-stage-split.sh` 与它们共用的 `common.sh`；之后照用户要求，合成了 `scripts/experiment-apparatus/e185-r2-segment-run.sh`（登记第五节「跑法」已改写，第十二节「起法」一行写明），三趟跑的是合并之前那几份，每段做的事相同（写输入清单 → 在拷过去的树里编一次装置 → 跑这一段 → 产物写进 `research/results/`）。
- `scripts/experiment-apparatus/replay.sh` 还没有 E185（GC一批搬迁的挂钟拆分） 的登记行（在 `scripts/experiment-apparatus/replay.sh` 里按该实验号 `grep -c` 输出 0；照 E184（常规层与大负载层的代价计数） 的先例只登记确定性的一片），三份测量产物都带挂钟字段。

````

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:52-128`（整段抄，未转述）**

````markdown
### 几何：f = 0.25 两点造成了（改法甲）

改法甲只换造池第 2 步覆盖写的次序（「做了什么」第 2 条），占比 0.75 的次序不变。单点核（登记第五节「单点核」，只核 P = 1024）有两份：改法甲 `research/results/e185-gc-batch-wall-clock-split-geometry-check-r2-2026-10-23.out`，对照 `research/results/e185-gc-batch-wall-clock-split-geometry-check-slot-order-r2-2026-10-23.out`（`--overwrite-order slot-order`，第 1 次重跑那种造法）。登记三格的读数：

| 格 | 读哪 | 读到的 | 判 |
|---|---|---|---|
| 甲造成 | 改法甲那一份第 64 行 `name=geometry`、第 55 行 `name=segment_composition` | `formed=true`；那一段 `live_slots=16 slots_of_this_workload=16 other_slots=0` | 成立，停机条款第 8 条（第十条 H1）不中 |
| 占比 0.75 没被同一棵树里的改动带歪 | 两份第 123 行 `name=pool fraction=0.75 pool_data_units=1024` 的 `digests=` 整值 | 相同（紧接的代码块里 `diff` 退 0） | 成立，停机条款第 9 条不中 |
| 对照：旧造法在今天的树上重现「没造成」 | 对照那一份第 3 行 `name=geometry_attempt`、第 64 行 `name=geometry` | `geometry_formed_segments=0`、`choice=none … formed=false` | 与登记推的 0 相同 |

```text
E7RESULT name=geometry fraction=0.25 pool_data_units=1024 choice=forced_fallback fallback_region_start_slot=53824 formed=true reason=greedy_segment_holds_units_other_than_this_workload_no_registered_workload_change
E7RESULT name=segment_composition fraction=0.25 pool_data_units=1024 overwrite_order=highest_full_segment_last workload_changes=0 region_start_slot=53824 live_slots=16 slots_of_this_workload=16 other_slots=0 formed=true
E7RESULT name=geometry_attempt fraction=0.25 pool_data_units=1024 workload_changes=0 construction_seconds=20.697 overwritten_units=776 filler_units=0 floor_raises_pushed_by_the_session=0 release_rounds=4 every_overwritten_slot_released=true mapped_units=1056 geometry_formed_segments=1 greedy_region_start_slot=51008 greedy_live_slots=1 greedy_slots_of_this_workload=0 greedy_other_slots=1 greedy_formed=false
E7RESULT name=geometry_attempt fraction=0.25 pool_data_units=1024 workload_changes=0 construction_seconds=20.776 overwritten_units=776 filler_units=0 floor_raises_pushed_by_the_session=0 release_rounds=4 every_overwritten_slot_released=true mapped_units=1058 geometry_formed_segments=0 greedy_region_start_slot=50368 greedy_live_slots=1 greedy_slots_of_this_workload=0 greedy_other_slots=1 greedy_formed=false
E7RESULT name=geometry fraction=0.25 pool_data_units=1024 choice=none fallback_region_start_slot=none formed=false reason=greedy_segment_holds_units_other_than_this_workload_no_registered_workload_change
```

改法甲下 greedy 自己选中的那一段（第 3 行 `greedy_region_start_slot=51008 greedy_live_slots=1 greedy_slots_of_this_workload=0 greedy_other_slots=1 greedy_formed=false`）仍是装着别的单元的段，f = 0.25 的点跟占比 0.75 一样走登记好的后备（`choice=forced_fallback`，只供测试的强制选段点名那一个 f 段，第 64 行 `reason=greedy_segment_holds_units_other_than_this_workload_no_registered_workload_change`）。所以第一、二段量的是被强制选定的那一段，不是 greedy 自己选中的段。第一、二段八个 `name=geometry` 行（四个点 × 两段）全是 `choice=forced_fallback … formed=true`：

```text
$ grep -h 'name=geometry ' research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-2026-10-23.out research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out | grep -c 'choice=forced_fallback .* formed=true'
8
```

```text
E7RESULT name=geometry fraction=0.25 pool_data_units=1024 choice=forced_fallback fallback_region_start_slot=53824 formed=true reason=greedy_segment_holds_units_other_than_this_workload_no_registered_workload_change
E7RESULT name=geometry fraction=0.25 pool_data_units=8192 choice=forced_fallback fallback_region_start_slot=78464 formed=true reason=greedy_segment_holds_units_other_than_this_workload_no_registered_workload_change
```

占比 0.75 的池与第 1 次重跑是不是逐字节相同，登记第五节两道，各报各的：

```text
$ diff <(awk 'NR==237||NR==515' research/results/e185-gc-batch-wall-clock-split-wall-clock-r1-2026-10-22.out | sed -E 's/.* pool_data_units=([0-9]+) .* digests=([^ ]+).*/\1 \2/') <(grep '^E7RESULT name=pool fraction=0.75 ' research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-2026-10-23.out | sed -E 's/.* pool_data_units=([0-9]+) .* digests=([^ ]+).*/\1 \2/'); echo "exit=$?"
1,2c1,2
< 1024 423aa5bc004b3247287ae29cf0bd80de0508b33ec2724f654640e0475fdbc216,ec8a4ecb79383a2f66c6828c4ce64bc0234a708f10c09b7566a5d58b79200ca3
< 8192 e0910aa57c7de2b42dfd1026123435f3e84244105ef0067fb6b0aa44edc3cbb6,d177e4fc925f9c701acdd6e9a879bfff3017615db4c034e1d0c9e55ea6902e26
---
> 1024 a77b38d298bf0f30358b2011867845734bea60928bcb4b1e98b2ab13563d30c3,c43dcaf3fa8b68f1cf7b1d498569f1fa96a6220d07b428d8b5ceecb8b5ebc5d0
> 8192 7d26930931d792bf5859dfc1ceaa83a87ae4a2328ffce19af576986880642c3e,72de4e03f439421f3e56933997b33246b24351994fb457965e6d157778679ede
exit=1
$ diff <(grep '^E7RESULT name=pool fraction=0.75 pool_data_units=1024 ' research/results/e185-gc-batch-wall-clock-split-geometry-check-r2-2026-10-23.out | sed -E 's/.* pool_data_units=([0-9]+) .* digests=([^ ]+).*/\1 \2/') <(grep '^E7RESULT name=pool fraction=0.75 pool_data_units=1024 ' research/results/e185-gc-batch-wall-clock-split-geometry-check-slot-order-r2-2026-10-23.out | sed -E 's/.* pool_data_units=([0-9]+) .* digests=([^ ]+).*/\1 \2/'); echo "exit=$?"
exit=0
$ diff <(grep '^E7RESULT name=pool fraction=0.75 pool_data_units=1024 ' research/results/e185-gc-batch-wall-clock-split-geometry-check-r2-2026-10-23.out | sed -E 's/.* pool_data_units=([0-9]+) .* digests=([^ ]+).*/\1 \2/') <(grep '^E7RESULT name=pool fraction=0.75 pool_data_units=1024 ' research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-2026-10-23.out | sed -E 's/.* pool_data_units=([0-9]+) .* digests=([^ ]+).*/\1 \2/'); echo "exit=$?"
exit=0
```

第 1 道（与第 1 次重跑第一段第 237、515 行比）不同，第 2 道（同一棵树里：单点核两份、再加第一段第 912 行）相同。照登记第五节判法记：占比 0.75 的池与第 1 次重跑不同，同一棵树里改动没有带歪它；差在两棵树之间（核心层五份源文件与 `Cargo.lock` 的 sha256 与第 1 次重跑输入清单不同，登记第三节第 134 行）。不停，占比 0.75 的点照量，这一趟的数不与第 1 次重跑的数混用，所以岔路表与各量的判定只引这一趟。

机理核对（登记第八节，第十条 H2；只报，不是锚点，对不上不作废）：覆盖写副本落进这次造池打出的旧洞的份数，装置逐次数出来的与登记推的 `overwritten_units − 24` 并列，造成段数两条路数并列，紧接的两条命令从三份产物里数：

```text
$ grep -h 'name=overwrite_landing' research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-2026-10-23.out research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out research/results/e185-gc-batch-wall-clock-split-geometry-check-r2-2026-10-23.out | grep 'status=counted' | sed -E 's/.*fraction=([0-9.]+) pool_data_units=([0-9]+) .*copies_landing_in_holes_of_this_construction=([0-9]+) .*predicted_copies_landing_in_holes=([0-9]+).*/\1 \2 \3 \4/' | sort | uniq -c
      3 0.25 1024 752 752
      2 0.25 8192 6128 6128
      3 0.75 1024 256 256
      2 0.75 8192 2048 2048
$ grep -h 'name=segment_composition_summary' research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-2026-10-23.out research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out | sed -E 's/.*fraction=([0-9.]+) pool_data_units=([0-9]+) .* formed_segments=([0-9]+) geo.*/\1 \2 \3/' | sort | uniq -c
      2 0.25 1024 1
      2 0.25 8192 1
      2 0.75 1024 3
      2 0.75 8192 3
```

三样都对上：每个点落进旧洞的副本数 = 登记的预测（752、6128、256、2048）；改法甲下 f = 0.25 的 `formed_segments` 是 1（登记预测 1，与 P 无关），f = 0.75 是 3（预测 3）；对照那一份 f = 0.25 的 `geometry_formed_segments=0`（预测 0，上面对照格）。`name=segment_composition_summary` 共 12 行（第一、二段各 4、单点核两份各 2），`counts_agree=true` 12 行、`counts_agree=false` 0 行（`grep -h 'name=segment_composition_summary' <四份产物> | grep -c 'counts_agree=true'` 输出 12，`grep -c 'counts_agree=false'` 输出 0）。记「调查员推的机理在这几个点上对上了」；这一句只说这四个点与单点核那两点，不外推到别的 f、P。

```text
E7RESULT name=segment_composition_summary fraction=0.25 pool_data_units=1024 overwrite_order=highest_full_segment_last workload_changes=0 segments=58 formed_segments=1 geometry_formed_segments_of_the_attempt=1 counts_agree=true
E7RESULT name=overwrite_landing fraction=0.25 pool_data_units=1024 overwrite_order=highest_full_segment_last workload_changes=0 status=counted overwritten_units=776 copies_landing_in_holes_of_this_construction=752 copies_landing_elsewhere=24 predicted_copies_landing_in_holes=752 root_ring_entries=24
E7RESULT name=segment_composition_summary fraction=0.25 pool_data_units=8192 overwrite_order=highest_full_segment_last workload_changes=0 segments=472 formed_segments=1 geometry_formed_segments_of_the_attempt=1 counts_agree=true
E7RESULT name=overwrite_landing fraction=0.25 pool_data_units=8192 overwrite_order=highest_full_segment_last workload_changes=0 status=counted overwritten_units=6152 copies_landing_in_holes_of_this_construction=6128 copies_landing_elsewhere=24 predicted_copies_landing_in_holes=6128 root_ring_entries=24
E7RESULT name=segment_composition_summary fraction=0.75 pool_data_units=1024 overwrite_order=highest_full_segment_last workload_changes=0 segments=53 formed_segments=3 geometry_formed_segments_of_the_attempt=3 counts_agree=true
E7RESULT name=overwrite_landing fraction=0.75 pool_data_units=1024 overwrite_order=highest_full_segment_last workload_changes=0 status=counted overwritten_units=280 copies_landing_in_holes_of_this_construction=256 copies_landing_elsewhere=24 predicted_copies_landing_in_holes=256 root_ring_entries=24
E7RESULT name=segment_composition_summary fraction=0.75 pool_data_units=8192 overwrite_order=highest_full_segment_last workload_changes=0 segments=418 formed_segments=3 geometry_formed_segments_of_the_attempt=3 counts_agree=true
E7RESULT name=overwrite_landing fraction=0.75 pool_data_units=8192 overwrite_order=highest_full_segment_last workload_changes=0 status=counted overwritten_units=2072 copies_landing_in_holes_of_this_construction=2048 copies_landing_elsewhere=24 predicted_copies_landing_in_holes=2048 root_ring_entries=24
```

````

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:129-139`（整段抄，未转述）**

````markdown
### 问题单第 1 行够判的集合 S

登记第一节「第 1 行够判」的读法（设计员写的，主 agent 认下，调度记录 `records/2026-10-19-里程碑四GPU垃圾回收开工.md` 第 381 行）：S = 第一段里（f、P、K）三者的格，那一点 `name=geometry` 写 `formed=true`，且那一格 `name=q3` 写 `judgement=compatible`；两个 f 在 S 里各有至少两个 K 就够判；Q1、Q2 在 S 的格上报的判定就是第 1 行的读数。

```text
$ grep -c 'name=q3 .*judgement=compatible row_one_judgeable_here=true' research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-2026-10-23.out
12
```

第一段 `name=geometry` 四行全 `formed=true`（上面），`name=q3` 12 格全是 `judgement=compatible row_one_judgeable_here=true`（第 1879、1883、1887、1891、1895、1899、1903、1907、1911、1915、1919、1923 行），所以 S = 全部 12 格：f = 0.25 与 f = 0.75 各 6 格（P = 1024、8192 × K = 4、8、16）。第二段 12 格同一批点。F5 没触发（判决行 `clause_two_cpu_paths_disagree_points=0`），F4 没有（每格 5 轮判定一致）。

````

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:140-192`（整段抄，未转述）**

````markdown
### 各量的判定（第 2 次重跑）

照原登记第六节各报各的判定，5 轮判定不全相同写「不稳定」。产物：第一段 = `research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-2026-10-23.out`（这一页里行号叫「第一段第 N 行」），第二段 = `research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out`（「第二段第 N 行」）。区间是对那一组行取最小与最大，两端都是产物里的原值。第一段 Q1、Q2 合计 24 格的判定数：

```text
$ W=research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-2026-10-23.out
$ grep -E 'name=q[12] ' $W | sed -E 's/.* judgement=([a-z_]+)$/\1/' | sort | uniq -c
     12 both
     12 cpu
```

第一段 12 个（f、P、K）格，每格在 `name=q1`、`name=q2`、`name=q3`、`name=q14` 四行里各一行，四行连着排（第 1877–1880 行起，每格 4 行，到第 1924 行）。

| 量 | 读哪 | 这一趟的数 | 判定 |
|---|---|---|---|
| Q1 ρ_真盘 | 第一段 `name=q1` 12 行（第 1877、1881、1885、1889、1893、1897、1901、1905、1909、1913、1917、1921 行） | `rho_median`：P = 1024 六格 1.221978–1.493617，P = 8192 六格 8.441898–9.848870 | P = 1024 六格 `both`，P = 8192 六格 `cpu`；每格 5 轮判定全相同（`judgement_per_round`），没有 `disk`、没有 `unstable`；f = 0.25 的六格与 f = 0.75 的六格同判。随 P 翻面（第 1926 行六对全翻），不随 f（第 1925 行 `flips=0`）、不随 K（第 1927 行 `flips=0`） |
| Q2 挂钟比 | 第一段 `name=q2` 12 行（第 1878、1882、…、1922 行，每隔 4 行） | `intent_wall_ratio_median`：P = 1024 六格 1.723812–1.906832，P = 8192 六格 1.105487–1.118963 | 与 Q1 同判：P = 1024 六格 `both`，P = 8192 六格 `cpu`，每格 5 轮相同；随 P 翻面（第 1929 行 `flips=6`），不随 f（第 1928 行）、不随 K（第 1930 行）`flips=0` |
| Q3 两路对账 | 第一段 `name=q3` 12 行（第 1879、1883、…、1923 行） | 每轮 `reconciliation_per_round` 最大 0.103088（f = 0.75、P = 1024、K = 8 第 5 轮，第 1907 行）；f = 0.25 的格最大 0.055696（P = 1024、K = 16 第 2 轮，第 1887 行） | 12 格全 `compatible`、`row_one_judgeable_here=true`；F5 没触发（判决行 `clause_two_cpu_paths_disagree_points=0`）。离 20% 的门槛最近的一轮是 0.103088 |
| Q4 批大小翻倍 | 第一段 `name=q4` 12 对（第 1933–1944 行） | 每对 5 轮中位 1.015102–1.074115 | 内存臂 6 对、真盘臂 6 对全 `rises`，12 对的 `copy_bytes_double=true`、`clause_wall_clock_not_rising_with_the_batch_triggered=false`；判决行 F1 触发数 0。判据只要 > 1（且每轮 > 1），这一趟涨得不多：批大小翻倍，第一批挂钟涨 1.5%–7.4% |
| Q14 GC 单跑写带宽 | 第一段 `name=q14` 12 行 | `write_bytes_per_second_median` 7229989.048161–50509463.337147 | 不单判 |
| Q15 设备顺序写带宽 | 第一段第 1945 行 | `E7RESULT name=q15 direct_with_fdatasync_bytes_per_second_median=3161349388.1 page_cache_without_sync_bytes_per_second_median=13066869439.3 positive_control_row_four_holds=true` | 不单判 |
| Q16 带宽比 | 第一段第 1958 行 | `maximum_ratio=0.015977`（f = 0.75、P = 1024、K = 16，第 1954 行 `q16_point`） | 「单跑够」；三个旋钮都不翻（第 1959–1961 行 `flips=0`） |
| Q17 前台 fsync 延迟比 | 第四段 | 没跑 | 够判后未跑（第 4 行，Q16 ≤ 10%，原登记第十一节够判停机） |
| Q5 甲段份额 | 第二段 `name=stage` 批行（`kind` 是 `batch` 或 `final_batch`）的 `crc_share`；`name=stage_judgement` 30 行 | f = 0.75 最大 0.020433（第 947 行），f = 0.25 最大 0.006863（第 116 行） | 30 行 `crc_share_judgement=not_worth`，每行 5 轮都是 |
| Q6 乙段增量份额 | 同上的 `scan_increment_share` | f = 0.75 最大 0.006521（第 1065 行），f = 0.25 最大 0.002601（第 216 行） | 30 行 `scan_increment_share_judgement=not_worth`，每行 5 轮都是 |
| Q7 丙段增量份额 | 第二段 `name=choose_judgement` 12 行（第 248–250、869–871、1148–1150、1793–1795 行）的 `choose_share_per_batch_per_round` | f = 0.25 最大 0.028846（第 250 行），f = 0.75 最大 0.014279（第 1150 行）；签发那一次的原值 `choose_share_of_the_issue` 最大 f = 0.25 是 0.03894、f = 0.75 是 0.038388，也都低于 10% | 12 行 `choose_share_judgement=not_worth`，每行 5 轮都是。`name=stage_judgement` 里的丙段那一格 30 行都是 `not_run`，是登记第六节写明的（丙段改读 `name=choose_judgement`），不是漏判 |
| Q8 单线程吞吐 | 第二段批行 `crc_bytes_per_second`、`scan_increment_bytes_per_second`；签发行 `choose_record_bytes_per_second` | 甲 2875331797.740485–3189332554.68769 B/s；乙增量 1046057094.516405–12147544022.242819 B/s；丙 112157777.007656–892197239.242721 记录字节/秒 | 不单判，喂 Q9 |
| 饱和吞吐（Q9 的 CPU 一侧） | 第二段 `name=saturation` 108 行，`distinct_threads_holds=true` 108 行 | 16 线程：甲 48832371460.2–49032300970.9，乙 14532706506.7–37784969513.8，丙 1641866090.6–13741047928.6 B/s | 不单判 |

第二段几条数的命令与输出（`S` 是第二段产物路径）：

```text
$ S=research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out
$ for f in 0.25 0.75; do for k in crc_share scan_increment_share; do echo -n "batch f=$f $k max: "; grep 'name=stage ' $S | grep -E "kind=(batch|final_batch)" | grep "fraction=$f " | grep -o " $k=[0-9.]*" | sed 's/.*=//' | sort -g | tail -1; done; done
batch f=0.25 crc_share max: 0.006863
batch f=0.25 scan_increment_share max: 0.002601
batch f=0.75 crc_share max: 0.020433
batch f=0.75 scan_increment_share max: 0.006521
$ for f in 0.25 0.75; do echo -n "choose_judgement f=$f max per_round: "; grep 'name=choose_judgement ' $S | grep "fraction=$f " | grep -o 'choose_share_per_batch_per_round=[^ ]*' | sed 's/.*=//' | tr ',' '\n' | sort -g | tail -1; done
choose_judgement f=0.25 max per_round: 0.028846
choose_judgement f=0.75 max per_round: 0.014279
$ grep 'name=stage_judgement ' $S | sed -E 's/.* (crc_share_judgement=[a-z_]+) .* (scan_increment_share_judgement=[a-z_]+) .* (choose_share_judgement=[a-z_]+)$/\1 \2 \3/' | sort | uniq -c
     30 crc_share_judgement=not_worth scan_increment_share_judgement=not_worth choose_share_judgement=not_run
$ grep 'name=choose_judgement ' $S | sed -E 's/.* (choose_share_judgement=[a-z_]+)$/\1/' | sort | uniq -c
     12 choose_share_judgement=not_worth
```

最大的那一行的第二段第 250 行（丙段）整行抄：

```text
E7RESULT name=choose_judgement fraction=0.25 pool_data_units=1024 batch_units=16 rounds=5 choose_share_per_batch_per_round=0.028676,0.028423,0.028614,0.028705,0.028846 choose_share_judgement_per_round=not_worth,not_worth,not_worth,not_worth,not_worth choose_share_judgement=not_worth
```

````

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:193-205`（整段抄，未转述）**

```markdown
### 沿用第 1 次重跑的量（问题单第 3、4 行；这一趟第三、四段不跑）

数取自第 1 次重跑第三段 `research/results/e185-gc-batch-wall-clock-split-gpu-submission-r1-2026-10-22.out`（这一页叫「r1 第三段第 N 行」）与它读的第 1 次重跑第一段 `research/results/e185-gc-batch-wall-clock-split-wall-clock-r1-2026-10-22.out`。两个范围都含 f = 0.25 的两点，那两点在第 1 次重跑里几何没造成（量的是 greedy 选中的段），这一趟没有重算。

| 量 | 读哪 | 第 1 次重跑的数 | 判定 |
|---|---|---|---|
| Q9 16 线程对 GPU | r1 第三段 `name=q9` 72 行（两张卡 × 三段 × 12 格） | `ratio` 最大 0.007746（卡 0、乙增量、f = 0.75、P = 8192、K = 8） | 72 行 `judgement=gpu_loses`，单线程版 `judgement_one_thread=gpu_loses`，`clause_sixteen_thread_form_misfits_the_gc_triggered=false`（F2 没触发）。这一趟在 f = 0.25 造成了的点上没有 Q9。Q9 算不算问题单第 2 行的翻面量，交主 agent，并进七·2 设计轮定（登记第一节、调度记录第 393 行） |
| Q10 B_批 | r1 第三段 `name=q13` 的 `batch_bytes`（读第 1 次重跑第一段 `name=call`） | f = 0.75：131072–786432 字节；f = 0.25：16384 字节（几何没造成的点） | 不单判 |
| Q11 a、Q12 b | r1 第三段第 54、136 行 | 卡 0 `E7RESULT name=q11_q12 index=0 card_valid=true intercept_seconds=0.004492 slope_seconds_per_byte=4.117289e-10 one_range_median_seconds=0.005244 largest_two_points_slope_seconds_per_byte=4.172332e-10`；卡 1 `E7RESULT name=q11_q12 index=1 card_valid=true intercept_seconds=0.010300 slope_seconds_per_byte=1.266004e-9 one_range_median_seconds=0.005393 largest_two_points_slope_seconds_per_byte=1.191920e-9` | 不单判 |
| Q13 φ 与 n* | r1 第三段 `name=q13` 56 行 | 卡 0 `fixed_overhead_share` 0.932762–0.998500、`batches_to_merge` 125–5993；卡 1 0.911860–0.997990、94–4470 | 56 行 `judgement=merge_batches`（φ ≥ 50%，「要攒批」），两张卡同判（F8 没触发）。判别力第 201 行 `E7RESULT name=discrimination quantity=Q13 lower_value=0.911860 higher_value=0.998500 moved_threshold=0.955180 lower_judgement=one_batch_per_submission higher_judgement=merging_optional holds=true` |

推的（没量）：f = 0.25 造成之后，它的 B_批 ≤ 签发那一次的 262144 字节（原登记第七节乙类第 356 行），比 f = 0.75 的小；φ = a ÷ (a + b·B_批) 随 B_批变小只会变大，翻面要 φ 落到 50% 以下，第 1 次重跑 φ 最小的那一格是 0.911860。

```

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:206-221`（整段抄，未转述）**

````markdown
### 阳性对照、自检与失败条款

- 第一段第 1 行阳性对照四格（第 1963–1966 行）：盘臂内存盘、真盘两格 `holds=true`，CPU 臂内存盘、真盘两格 `cpu_clause_holds=true write_wait_clause_holds=true`；CPU 臂内存盘那一格原判据那一句 `wait_clause_under_the_original_criterion_holds=false`，照 12.9 只报不判。整行抄：

```text
E7RESULT name=positive_control row=1 arm_of_the_question=disk device=memory injection=sleep_2_milliseconds_per_write baseline_rounds=5 control_rounds=5 write_calls_median=176.0 injected_nanoseconds=352000000.0 cpu_increment_nanoseconds=1795862.0 wait_increment_nanoseconds=362017597.0 write_wait_increment_nanoseconds=362030328.0 cpu_change=0.036631 wait_change=-3020.110262 paired_rounds=5 cpu_increment_paired_median_nanoseconds=1701172.000000 write_wait_increment_paired_median_nanoseconds=362030518.000000 holds=true
E7RESULT name=positive_control row=1 arm_of_the_question=cpu device=memory injection=spin_2_milliseconds_per_write baseline_rounds=5 control_rounds=5 write_calls_median=176.0 injected_nanoseconds=352000000.0 cpu_increment_nanoseconds=352496564.0 wait_increment_nanoseconds=68029.0 write_wait_increment_nanoseconds=67282.0 cpu_change=7.189988 wait_change=-0.567528 cpu_increment_over_injected=1.001411 cpu_clause_holds=true write_wait_increment_over_injected=0.000191 write_wait_clause_holds=true wait_clause_under_the_original_criterion_holds=false original_criterion_is_reported_only=true paired_rounds=5 cpu_increment_paired_median_nanoseconds=352445233.000000 write_wait_increment_paired_median_nanoseconds=67472.000000 holds=true
E7RESULT name=positive_control row=1 arm_of_the_question=disk device=real_disk injection=sleep_2_milliseconds_per_write baseline_rounds=5 control_rounds=5 write_calls_median=176.0 injected_nanoseconds=352000000.0 cpu_increment_nanoseconds=1782904.0 wait_increment_nanoseconds=361903509.0 write_wait_increment_nanoseconds=362318961.0 cpu_change=0.033885 wait_change=7.108353 paired_rounds=5 cpu_increment_paired_median_nanoseconds=1272218.000000 write_wait_increment_paired_median_nanoseconds=362373072.000000 holds=true
E7RESULT name=positive_control row=1 arm_of_the_question=cpu device=real_disk injection=spin_2_milliseconds_per_write baseline_rounds=5 control_rounds=5 write_calls_median=176.0 injected_nanoseconds=352000000.0 cpu_increment_nanoseconds=353173767.0 wait_increment_nanoseconds=4108344.0 write_wait_increment_nanoseconds=1711955.0 cpu_change=6.712162 wait_change=0.080694 cpu_increment_over_injected=1.003335 cpu_clause_holds=true write_wait_increment_over_injected=0.004864 write_wait_clause_holds=true wait_clause_under_the_original_criterion_holds=true original_criterion_is_reported_only=true paired_rounds=5 cpu_increment_paired_median_nanoseconds=352961316.000000 write_wait_increment_paired_median_nanoseconds=1684671.000000 holds=true
```

- 第二段第 2 行阳性对照四格（第 123–126 行）`holds=true`；两段 `name=clock_resolution_check` 都 `holds=true`（两段第 3 行）；第一段 `name=discrimination` 三行（Q1、Q2、Q16，第 1931、1932、1962 行）`holds=true`；两段 `name=anchor` 各 34 行全 `holds=true`（`grep 'name=anchor' <产物> | grep -c 'holds=true'` 两份都输出 34，`grep -vc` 输出 0）。
- 作废：第一段 `name=intent` 140 行全是 `valid=true invalid_reason=none`，第二段 60 行同；两段都没有 `name=round_invalid` 行（`grep -c 'name=round_invalid'` 输出 0），`stored_intents=140`。第 1 次重跑第一段作废过 1 轮，这一趟一轮都没作废、没重试。
- 失败条款：F1 没触发（Q4 12 对全 `rises`）；F2 这一趟不适用（Q9 沿用）；F3 打中：Q1 在 P = 1024 的六格落在 (0.5, 2)（1.221978–1.493617）、Q2 在 P = 1024 的六格落在 (1.5, 3)（1.723812–1.906832），照实记「两边都占」，是正当结果；F4 没有；F5 没触发；F7 打中：第二段 Q5、Q6、Q7 都 < 10%（最大分别 0.020433、0.006521、0.028846），照原登记记「三段都不到一批 CPU 的 10%，CPU 的大头在其余（读这一版、发布）」，交主 agent 判第 2 行的候选是否要补一个「发布那一半」的臂；F6、F8 沿用第 1 次重跑（F6：Q13 没有落在 (10%, 50%) 的格；F8：两张卡同判），这一趟没重算。
- 这一趟新加的失败条款（登记第十条）：H1（甲造不出）没触发；H2（机理对不上）没触发，见「几何」一节；H3、H4 以 F5 打中为前提，没打中，不适用；H5（轮数不足）没有，`name=round_invalid` 0 行；H6（第 1 行仍不够判）没有，S 里 f = 0.25 与 f = 0.75 各 6 格；H7（第 4 行单跑那一半翻面）没触发，`name=q16` 是 `single_run_enough`；H8（校准基线漂移）没触发，两段 `drifted=false`。

````

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:222-241`（整段抄，未转述）**

````markdown
### 校准与机器状态读数（登记第六节新加的量，读数 1–N15）

校准比（读数 1）是测量线程每条意图前后对一块 16 MiB 缓冲连做 3 遍 CRC-32C 取中位，除以段开头 21 遍的中位；> 1.10 作废（修订 R2-1）；作废后先等机器回到常态再重试（R2-2）。其余读数只报不判。

| 读数 | 第一段 | 第二段 |
|---|---|---|
| 段开头 / 段末基线中位（读数 2） | 5243296.0 / 5227887.0 纳秒，末 ÷ 头 = 0.997061，`drifted=false`（第 5、2099 行） | 5224477.0 / 5222377.0 纳秒，0.999598，`drifted=false`（第 5、1799 行） |
| 校准自己的阳性对照（R2-3，加倍的活） | `ratio=2.003004 crc_holds=true holds=true`（第 6 行） | `ratio=2.003983 crc_holds=true holds=true`（第 6 行） |
| 前后两个校准比里大的那个的最大值（读数 1） | 1.015958；> 1.10 的意图 0 条，> 1.05 的 0 条，> 1.20 的 0 条；等常态 0 次（第 2100 行） | 1.077323；> 1.10 的 0 条，> 1.05 的 3 条，> 1.20 的 0 条；等常态 0 次（第 1800 行） |
| 追指针校准比（N15）的范围 | 1.226963–1.534854，每条意图都高于 1 | 1.279786–1.514878，同 |
| 整机其余部分占的忙比例（N10 `others_share`）最大 | 0.002502（第 4 行，段开头） | 0.001875（第 932 行） |
| 线程读数（读数 3–N9）最大 | 排队等 CPU 7508078 纳秒，非自愿切换 151，`major_faults` 全 0，CPU 号 4 个 | 排队等 CPU 914484 纳秒，非自愿切换 29，CPU 号 2 个 |

```text
E7RESULT name=calibration_trajectory segment=wall_clock intents=140 intents_with_a_readable_ratio=140 larger_ratio_peak=1.015958 intents_above_the_voiding_edge=0 voiding_edge=1.10 intents_above_the_lower_reported_edge=0 lower_reported_edge=1.05 intents_above_the_upper_reported_edge=0 upper_reported_edge=1.20 larger_ratio_final=0.998801 wait_longest_seconds=0 waits=0 wait_total_seconds=0
E7RESULT name=calibration_trajectory segment=stage_split intents=60 intents_with_a_readable_ratio=60 larger_ratio_peak=1.077323 intents_above_the_voiding_edge=0 voiding_edge=1.10 intents_above_the_lower_reported_edge=3 lower_reported_edge=1.05 intents_above_the_upper_reported_edge=0 upper_reported_edge=1.20 larger_ratio_final=1.002871 wait_longest_seconds=0 waits=0 wait_total_seconds=0
```

追指针校准比（N15）全部高于 1（1.23–1.53）是装置报出来的，登记只要求报、不判，这一页没查原因；它没有用来作废任何意图。N14（照原作废规则的并列判定）两段各 12 行，`differs_from_the_registered=true` 0 行（`grep -c 'differs_from_the_registered=true'` 对两份产物都输出 0）：没有一条意图因校准比被作废，R2-1 在这一趟没有动任何一格的进入与否。

````

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:242-275`（整段抄，未转述）**

````markdown
### 问题单逐行（岔路表）

问题单 `research/prompts/m4-gpugc-measure-forks.md`（四行的「状态」一格已由主 agent 改成「够判」，调度记录 `records/2026-10-19-里程碑四GPU垃圾回收开工.md` 第 394 行）；够判条件照问题单原文，判定照这一趟的产物；S 的读法见「各量的判定」一节。

| 行 | 已够判 / 还差什么 | 这一趟读得出的 | 剩下的量能不能让它翻面 |
|---|---|---|---|
| 1 盘 ／ CPU | **已够判**。问题单够判条件：至少两种批大小、两种段内活槽占比，在内存设备与真盘上各量到一批的挂钟与按阶段拆开的 CPU 时间。f = 0.25 与 f = 0.75 的几何都造成了（第一段 `name=geometry` 四行 `formed=true`），S = 12 格（`grep -c 'name=q3 .*judgement=compatible row_one_judgeable_here=true'` 输出 12），两个 f 各 6 格、K = 4、8、16 都在，Q1、Q2 逐格 5 轮判定一致 | P = 1024 六格 Q1、Q2 都判 `both`（`rho_median` 1.221978–1.493617，`intent_wall_ratio_median` 1.723812–1.906832）；P = 8192 六格都判 `cpu`（8.441898–9.848870；1.105487–1.118963）。判定随 P 翻面（第一段第 1926、1929 行 `flips=6`），不随 f、不随 K 翻（第 1925、1927、1928、1930 行 `flips=0`）；按 f、P 报，不按 K 报。F3 打中（P = 1024 两边都占）。整行见紧接的第一块 | 登记里没有剩下没跑的量。能动它的只有 P 之外的点：装置只收 P = 1024、8192 两个池大小，两点之间与更大的池没量。推的（没量）：翻面轴是 P，池更大时 CPU 一侧只会更重 |
| 2 GPU 算哪几段 | **已够判**（第 1 行已够判）。登记第一节「第 2 行够判」：S 里每一格第二段 `name=stage_judgement` 的甲、乙判定与 `name=choose_judgement` 的丙判定都不是 `not_run`，且 `crc_bytes`、`records_read_per_choice` 大于 0：甲、乙 30 行批判定全 `not_worth`，丙 12 行全 `not_worth`，批行 `crc_bytes` 最小 131072，`records_read_per_choice` 最小 1068（命令见「各量的判定」一节） | 甲、乙、丙三段都 `not_worth`：甲份额最大 0.020433，乙增量最大 0.006521，丙每批最大 0.028846（签发那一次的原值最大 0.03894），都不到一批 CPU 的 10%。原登记 F7 打中：CPU 的大头在其余（读这一版、发布），交主 agent 判第 2 行的候选要不要补「发布那一半」的臂。Q9 沿用第 1 次重跑 72 行 `gpu_loses`，算不算翻面量交主 agent，并进七·2 设计轮定（Q9 是登记第一节写明的一格）。整行见紧接的第二块 | 登记里没有剩下没跑的量。Q9 在 f = 0.25 的点上没有（第三段不跑）；它不论算不算翻面量，都与 Q5–Q7 同向（第 1 次重跑 72 行全 `gpu_loses`，这一趟三段份额没有一格越过 10%） |
| 3 攒批形态 | **已够判**（沿用第 1 次重跑，这一趟第三段不跑）。依据：第 1 次重跑第三段 `research/results/e185-gc-batch-wall-clock-split-gpu-submission-r1-2026-10-22.out`（sha256 `4041a1ca…`）与它读的第 1 次重跑第一段 `research/results/e185-gc-batch-wall-clock-split-wall-clock-r1-2026-10-22.out`（`978ae7b5…`）。第三段装置 `eadb929d…` 比重跑登记写死的 `b3a50d50…` 多 3 行（`quarantine_mark`），主 agent 已认（本次登记第一节派发提示第 5 条，登记第 19 行） | 同第 1 次重跑：Q10 f = 0.75 每批 131072–786432 字节；两张卡 a、b 量到；Q13 56 行全 `merge_batches`，φ 最小 0.911860，两张卡同判：一批一交（甲）出局，选择在乙、丙之间，乙丙之间七·1 只给 n* 与每批字节，R 的语义与批间重扫由七·2 设计轮判（原登记第五节第 3 行臂表） | 没有剩下的量。翻面要 φ 落到 50% 以下，φ 最小的那一格也在 0.91 以上；f = 0.25 造成之后 B_批更小，φ 只会更大（推的，没量） |
| 4 兜底间隔与写带宽上限 | **单跑那一半已够判**（沿用第 1 次重跑 `name=q16`，这一趟第一段并列报、不重判）；第四段与 Q17 够判后未跑 | 第 1 次重跑第一段第 786 行 `E7RESULT name=q16 maximum_ratio=0.011019 judgement=single_run_enough …`；这一趟第一段第 1958 行 `maximum_ratio=0.015977 judgement=single_run_enough`，H7 没触发 | Q17 只在 Q16 > 10% 时跑，这一趟不跑 |

第一块（第 1 行的读数，第一段整行）：

```text
E7RESULT name=q1 fraction=0.25 pool_data_units=1024 batch_units=8 geometry_formed=true rounds=5 rho_per_round=1.456034,1.438745,1.464884,1.404448,1.458415 rho_median=1.456034 judgement_per_round=both,both,both,both,both judgement=both
E7RESULT name=q2 fraction=0.25 pool_data_units=1024 batch_units=8 geometry_formed=true rounds=5 intent_wall_ratio_per_round=1.772596,1.749635,1.720118,1.741935,1.760248 intent_wall_ratio_median=1.749635 per_batch_median_ratio=1.749635 judgement_per_round=both,both,both,both,both judgement=both
E7RESULT name=q3 fraction=0.25 pool_data_units=1024 batch_units=8 rounds=5 reconciliation_per_round=0.050004,0.031328,0.021439,0.016639,0.043347 judgement_per_round=compatible,compatible,compatible,compatible,compatible judgement=compatible row_one_judgeable_here=true
E7RESULT name=q1 fraction=0.75 pool_data_units=8192 batch_units=8 geometry_formed=true rounds=5 rho_per_round=9.819379,9.125596,9.851385,8.752512,9.908914 rho_median=9.819379 judgement_per_round=cpu,cpu,cpu,cpu,cpu judgement=cpu
E7RESULT name=q2 fraction=0.75 pool_data_units=8192 batch_units=8 geometry_formed=true rounds=5 intent_wall_ratio_per_round=1.108970,1.117088,1.108237,1.122220,1.114470 intent_wall_ratio_median=1.114470 per_batch_median_ratio=1.120025,1.116781,1.108746 judgement_per_round=cpu,cpu,cpu,cpu,cpu judgement=cpu
E7RESULT name=q3 fraction=0.75 pool_data_units=8192 batch_units=8 rounds=5 reconciliation_per_round=0.006159,0.006354,0.005763,0.006425,0.012056 judgement_per_round=compatible,compatible,compatible,compatible,compatible judgement=compatible row_one_judgeable_here=true
E7RESULT name=sensitivity quantity=Q1 knob=fraction pairs=6 flips=0 statement=does_not_flip_at_these_points flipped_pairs=none
E7RESULT name=sensitivity quantity=Q1 knob=pool_data_units pairs=6 flips=6 statement=flips_with_this_knob flipped_pairs=0.25:1024:4:both->0.25:8192:4:cpu,0.25:1024:8:both->0.25:8192:8:cpu,0.25:1024:16:both->0.25:8192:16:cpu,0.75:1024:4:both->0.75:8192:4:cpu,0.75:1024:8:both->0.75:8192:8:cpu,0.75:1024:16:both->0.75:8192:16:cpu
E7RESULT name=sensitivity quantity=Q2 knob=pool_data_units pairs=6 flips=6 statement=flips_with_this_knob flipped_pairs=0.25:1024:4:both->0.25:8192:4:cpu,0.25:1024:8:both->0.25:8192:8:cpu,0.25:1024:16:both->0.25:8192:16:cpu,0.75:1024:4:both->0.75:8192:4:cpu,0.75:1024:8:both->0.75:8192:8:cpu,0.75:1024:16:both->0.75:8192:16:cpu
```

第二块（第 2 行的读数，第二段整行：丙段判定、批判定的一行，份额最大的两行批行）：

```text
E7RESULT name=choose_judgement fraction=0.25 pool_data_units=1024 batch_units=4 rounds=5 choose_share_per_batch_per_round=0.014847,0.014637,0.014529,0.014800,0.014522 choose_share_judgement_per_round=not_worth,not_worth,not_worth,not_worth,not_worth choose_share_judgement=not_worth
E7RESULT name=stage_judgement fraction=0.25 pool_data_units=1024 batch_units=4 batch_index=0 rounds=5 crc_share_judgement_per_round=not_worth,not_worth,not_worth,not_worth,not_worth crc_share_judgement=not_worth scan_increment_share_judgement_per_round=not_worth,not_worth,not_worth,not_worth,not_worth scan_increment_share_judgement=not_worth choose_share_judgement_per_round=not_run,not_run,not_run,not_run,not_run choose_share_judgement=not_run
E7RESULT name=stage fraction=0.75 pool_data_units=1024 batch_units=4 round=1 call_index=1 kind=batch call_cpu_nanoseconds=12578975 crc_cpu_nanoseconds=257032.000000 crc_wall_nanoseconds=257271.000000 crc_repetitions=1 scan_cpu_nanoseconds=332311.000000 scan_increment_cpu_nanoseconds=75279.000000 choose_cpu_nanoseconds=none choose_calls=0 crc_bytes=786432 records_read_per_choice=1069 full_segments=3312 crc_share=0.020433 scan_increment_share=0.005985 choose_share_per_batch=none choose_share_of_the_issue=none crc_bytes_per_second=3059665722.555947 scan_increment_bytes_per_second=10446897541.146935 choose_record_bytes_per_second=none choose_records_per_second=none choose_full_segments_per_second=none snapshot_branch_bytes=0 snapshot_branch_note=the_pools_have_no_snapshot release_check_crc_cpu_nanoseconds=84920.000000 release_check_crc_wall_nanoseconds=85020.000000 release_check_crc_repetitions=1 release_check_crc_bytes=262144 crc_with_the_release_check_cpu_nanoseconds=341952.000000 crc_with_the_release_check_bytes=1048576
E7RESULT name=stage fraction=0.75 pool_data_units=1024 batch_units=16 round=3 call_index=1 kind=batch call_cpu_nanoseconds=13777990 crc_cpu_nanoseconds=247692.000000 crc_wall_nanoseconds=247821.000000 crc_repetitions=1 scan_cpu_nanoseconds=337532.000000 scan_increment_cpu_nanoseconds=89840.000000 choose_cpu_nanoseconds=none choose_calls=0 crc_bytes=786432 records_read_per_choice=1069 full_segments=3312 crc_share=0.017977 scan_increment_share=0.006521 choose_share_per_batch=none choose_share_of_the_issue=none crc_bytes_per_second=3175039968.993751 scan_increment_bytes_per_second=8753695458.593056 choose_record_bytes_per_second=none choose_records_per_second=none choose_full_segments_per_second=none snapshot_branch_bytes=0 snapshot_branch_note=the_pools_have_no_snapshot release_check_crc_cpu_nanoseconds=330301.000000 release_check_crc_wall_nanoseconds=330562.000000 release_check_crc_repetitions=1 release_check_crc_bytes=1048576 crc_with_the_release_check_cpu_nanoseconds=577993.000000 crc_with_the_release_check_bytes=1835008
```

````

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:276-285`（整段抄，未转述）**

```markdown
### 做了什么

1. 计时包装设备：包住内存盘（`SparseBlockDevice`）或真盘（ext4 上的 `O_DIRECT` 镜像文件，`DirectInputOutputBlockDevice`），`BlockDevice` 的每一种调用前后各读一次 `Instant` 与本线程 `clock_gettime(CLOCK_THREAD_CPUTIME_ID)`（CPU 读数包着挂钟读数），按调用种类累计挂钟、CPU、等盘（挂钟减 CPU）、次数与字节；读按「这个计时窗口里第一次写之前 / 之后」分开，第一次写之前的读再分三栏：落在 R 那块盘 R 里的、扣掉搬迁源槽上的再读，归扫描读；这一批搬迁源的槽上（两块盘同号槽）扣掉 R 那块盘每个槽第一次读之后剩下的，归释放前读盘核（`release_check_read_bytes`，登记第十二节 12.5）；其余归别处；写按「落在这一批目的地 / 别的」分开。阳性对照的注入（每次 `write_at` 里睡 2 ms，或按线程 CPU 时钟自旋 2 ms）也在这里。
2. 造池：harness 默认参数 mkfs 两块 4 GiB 内存盘、第一个文件、可写挂载；建 P + 64 个空 inode，前 P 个各顺序写一个数据单元；整段都是这次写的段按槽序每 4 个覆盖写 3 个（f = 0.25）或 1 个（f = 0.75），混住的段这次写的单元全覆盖写掉；用多建的空 inode 把最高那一段补满；再「抬 F 到准入上限、建一个 inode」转到被覆盖写的旧落点都已释放。**覆盖写的次序（改法甲，装置 `overwrite_order_of`，第 2131 行）**：f = 0.75 仍全部按旧槽号升序；f = 0.25 改成三组依次：① 不在「整段 32 个单元都是这次写的」段里的单元（混住段的全部，与配方累计点名的），组内按旧槽号升序；② 整段里要覆盖写的单元，除槽号最大的那一整段之外，按旧槽号升序；③ 槽号最大的那一整段里要覆盖写的 24 个，按旧槽号升序。覆盖写哪些单元、写几次、写的内容不变，只换次序。机理（登记第七节「推的预测」）：每次覆盖写是一次发布、写一条根，根环 24 条根，旧落点在覆盖写那一次发布就记成已释放，覆盖写到第 25 次起每份副本都落进 24 次之前打出的洞，所以最后 24 次覆盖写打出的洞（槽号最大那一整段的 24 个）造完时还空着，那一段剩 16 个活槽、全是这次写的。
3. 几何核：在一份拷贝上可写挂载、照会话签发那一步的选段那一圈重放（`choose_region` 与「先扫」交替），看 greedy 将选哪一段；那一段全是这次写的数据单元、活槽 = 64f 就用 greedy，不是而它全是这次写的数据单元就把它们并进配方重造（至多三次），装着别的单元就走登记好的后备（只供测试的强制选段点名槽号最小的 f 段）。造池之后逐段打成分（`name=segment_composition`，N11）、造成段数两条路数并列（`name=segment_composition_summary` 的 `counts_agree`）、覆盖写副本的落点（`name=overwrite_landing`，N12）；这几行只打每个点头一次造池，第 2–5 轮的重造不打。
4. 量一条意图：可写挂载（`mount_writable_with_compaction_switches`，产品开关只把批大小换成只供测试的 `ForcedForTests(K)`），每次调用之前把会话「已经过去的发布数」设成 1000（发布预算落到 1），一次一次调 `run_compaction` 到最后一批删意图；每次调用前后读 `Instant` 与线程 CPU 时钟（挂钟读数包着 CPU 读数，与设备调用那一层反过来），返回之后在计时窗口外按发成的那一版判是签发、批还是最后一批。每条意图前后还在计时窗口外做一遍校准（读数 1、N15）和线程与机器读数（读数 3–N9，`name=intent_machine_readings`）：校准比 > 1.10 的那一条作废，作废之后先每 10 秒做一次校准、连着两次 ≤ 1.10 才重量，单次至多等 300 秒、一段里累计至多等 600 秒（修订 R2-1、R2-2）；每段开头做校准基线（读数 2，21 遍）与加倍活的自检（R2-3），段末再做一遍基线（R2-4）。段开头、每个取样点开头、段末各一行 `name=machine_state`（N10）。
5. 第二段的重放：调用之前取这一版的分配记录、映射条目（照会话的私有取法另写一份）、钉住与隔离的槽，取扫描读到的单元字节；三个阶段（甲 每个被读单元整单元 CRC-32C（另一个子项是释放前读盘核那几份副本的整单元 CRC-32C，副本字节在调用之前从两块盘同号槽各读一份）、乙 整个 `scan_region`、丙 签发那一步每次 `choose_region`）每次重放先不计时跑一遍热身、再计时跑一遍，5 次重放取中位；饱和吞吐 1、16、N 线程各 2 秒。丙段的判定（改动六）：每轮取那一轮有效意图签发那一行已经算出的 `choose_share_per_batch`，按原门槛（10%）判，五轮收成一行 `name=choose_judgement`（`name=stage_judgement` 行照旧打，它的丙段那一格仍是 `not_run`）。
6. 第三段（沿用，这一趟不跑）：每张卡用造池时写的一个真数据单元（32768 字节）、n 取 1 到 16384 段，`GpuCard::crc32c_of_ranges` 先丢一次再计 5 次取中位，核每段 CRC 与 CPU 相同、每次不快过 PCIe 链路峰值；最小二乘拟合 a、b，用第一段的 B_批 算 φ 与 n*，用第二段的 16 线程饱和吞吐算 D24（后台重活能不能卸给 GPU） 已定项 3 形式的比（Q9）。
7. 单点核（`geometry-check` 子命令，改动二）：只造池、核几何，打 `name=geometry_attempt`、`name=geometry`、`name=segment_composition`、`name=segment_composition_summary`、`name=overwrite_landing`、`name=pool` 几行与末行，不量；`--overwrite-order slot-order` 照第 1 次重跑那一版造（全部按旧槽号升序，不数副本落点）。

```

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:286-315`（整段抄，未转述）**

````markdown
### 复跑（第 2 次重跑；命令照重跑登记第五节「跑法」，起法在 `scripts/experiment-apparatus/e185-r2-segment-run.sh` 文件头）

```bash
# 仓根。装置在验证包里，编与单测归主 agent 经 task-server 交（主工作区那一版 118 条）
SINGLEFS_RUN_SCOPE='…' task-server/task submit -- cargo test --release -p singlefs-verification --bin e185_gc_batch_wall_clock_split
# 三趟（单点核、第一段、第二段）各一条命令，主 agent 在本机仓根经 task-server 交；派哪台由服务定，两台在同一棵共享的树里跑，产物直接写进这棵树的 `research/results/`、两台都读得到；`--disk-model` 填重跑登记第五节点名的那块盘的型号，派到的那一台没有那块盘时入口退 78、一步都不编不跑；
# <段> 取 geometry-check、wall-clock、stage-split，<日期> 是那一趟起跑当天，产物名见上面「这一趟的产物」
SINGLEFS_RUN_SCOPE='…' task-server/task submit -- env E185_RUN_DATE=<日期> bash scripts/experiment-apparatus/e185-r2-segment-run.sh <段> --disk-model <盘的型号>
```

`e185-r2-segment-run.sh` 在共享的树里、同一个绝对路径上做四步（编译目录取 `CARGO_TARGET_DIR`：派到第二台时是统一任务服务把这一趟派过去时在第二台本地盘上新建、跑完删的那一个（服务不设、只记下它在哪，`task-server/task status` 看得见），没设是树里的 `target`）：写输入清单 `research/results/e185-r2-inputs-<段>-<日期>.txt` → `cargo build --release -p singlefs-verification --bin e185_gc_batch_wall_clock_split`（不带 `gpu` 特性，不带 `--all-targets`、`--tests`，免得并进 `lookup-work-counters`）→ 跑这一段（装置线程不设，取登记值 `.claude/heavy-thread-default`，环境 `E185_REGISTERED_APPARATUS_SHA256`、`E185_MEMORY_CAP=40G`）→ 产物写进 `research/results/`：

```bash
# geometry-check 段（单点核，两条）
${CARGO_TARGET_DIR:-target}/release/e185_gc_batch_wall_clock_split geometry-check --point 0.25:1024 --point 0.75:1024 > research/results/e185-gc-batch-wall-clock-split-geometry-check-r2-<日期>.out
${CARGO_TARGET_DIR:-target}/release/e185_gc_batch_wall_clock_split geometry-check --point 0.25:1024 --point 0.75:1024 --overwrite-order slot-order > research/results/e185-gc-batch-wall-clock-split-geometry-check-slot-order-r2-<日期>.out
# wall-clock 段（第一段；真盘臂的镜像目录取 `E185_IMAGE_DIRECTORY`，没设取 `<MULTI_HOST_RUN_PEER_SCRATCH>/e185-images`：派到第二台时是统一任务服务把这一趟派过去时在第二台本地给的目录，不在 NFS 上、不在开机清空的目录里，跑完连目录删；没有这个变量（在本机跑）取 `${CARGO_TARGET_DIR:-target}/e185-images`；镜像目录要落在 `--disk-model` 点名的那块盘上）
${CARGO_TARGET_DIR:-target}/release/e185_gc_batch_wall_clock_split wall-clock --image-directory <镜像目录> > research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-<日期>.out
# stage-split 段（第二段；只用内存盘，与第一段同一台、先后跑）
${CARGO_TARGET_DIR:-target}/release/e185_gc_batch_wall_clock_split stage-split > research/results/e185-gc-batch-wall-clock-split-stage-split-r2-<日期>.out
```

三趟在第二台先后跑、不同时跑；装置线程不设，取登记值 `.claude/heavy-thread-default`（跑的那一刻是 31）；产物文件名的日期是那一趟起跑当天，同一天再跑加 `-2`、`-3`，不盖已有的。三趟当时跑的是合并之前仓根的 `e185-r2-tools/run-<段>.sh`，每段做的事与上面相同。

第三段与第四段这一趟不跑。主 agent 定要 f = 0.25 造成了的点上的 Q9 时另跑第三段（条件照重跑登记第五节「第三段跑不跑」，三丁换成「主 agent 定要」；命令形态照第 1 次重跑，`--wall-clock-product`、`--stage-split-product` 换成这一趟的两份 `-r2-` 产物，带 `gpu` 特性另编）：

```bash
task-server/task submit -- bash -c 'cargo run --release -p singlefs-verification --features gpu --bin e185_gc_batch_wall_clock_split -- gpu-submission --wall-clock-product research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-2026-10-23.out --stage-split-product research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out --cards 0,1 > research/results/e185-gc-batch-wall-clock-split-gpu-submission-r2-<日期>.out'
```

````

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:316-330`（整段抄，未转述）**

```markdown
### 口径

- CPU 时间是本线程 `clock_gettime(CLOCK_THREAD_CPUTIME_ID)` 两次读数之差（登记第一节，第十二节 12.2 修订；原来的 schedstat 读法对正在跑的线程约 1 ms 才前进一次，实测数在 12.2）。`wall-clock`、`stage-split`、`foreground-fsync` 开跑先在本线程上自旋 1 ms、连读这个钟，前进不到 100 次就停机退 3（产物行 `name=clock_resolution_check`）。设备调用那一层 CPU 读数包着挂钟读数，等盘 = 挂钟 − CPU，纯 CPU 的设备调用里可以是负的（差读钟的工夫）；一次 `run_compaction` 调用那一层挂钟读数包着 CPU 读数。这样嵌套，两个钟走得一样快时「挂钟 ≥ CPU + 等盘」不会因为读钟的先后被打破。
- 一批的挂钟是包住那一次 `run_compaction` 调用（含调用末尾的 discard）的 `Instant`；discard、屏障、写零各按调用种类单列。
- 真盘臂：每次测量之前把上一次改过的 4 MiB 块从起始件拷回工作件，再核整份镜像的分块摘要（每 4 MiB 一块的 SHA-256 接起来再算一次 SHA-256）与起始件相同；内存臂每轮按同一份配方重造、核两块盘的分块摘要与第一次造的相同。挂载时 journal 记录偏移的提示照抄单元区之前写过的扇区（同内存盘的答法），不在 `O_DIRECT` 镜像上把整个环逐槽读一遍；计时窗口里不照抄。
- 设备：内存臂是 harness 的稀疏内存盘；真盘臂是两个镜像文件，放在第二台一块 NVMe（型号 CT2000T500SSD8，逻辑块 512，`research/results/e185-r2-inputs-wall-clock-2026-10-23.txt` 第 22–26 行）上的 ext4 里，屏障是 `fdatasync`，结论只说「第二台那块盘上 ext4 的 `O_DIRECT` 镜像文件」（重跑登记第一节「真盘」）；原登记写的本机那块盘没用。
- 批：一次调用里搬了单元的那一次（与会话 `observations.batches_published` 同一口径）；签发单列，不算批。
- Q4 的 `rises` 只要求第一批挂钟的比 > 1（5 轮中位，且每轮 > 1）；这一趟 12 对的中位在 1.015102–1.074115，涨幅小，判据不看涨了多少。
- Q7 的「一批 CPU」取这条意图批 CPU 的中位（装置从原登记起的读法，登记第六节「丙段怎么有读数」写死）；丙段的值是签发那一次 `name=stage` 行的 `choose_share_per_batch`，每轮取那一轮有效意图的一个，所以 `name=choose_judgement` 每格只有 5 个值。
- 第一行阳性对照（登记第五节阳性对照表第 1 行，CPU 臂等盘那一句照第十二节 12.9）：取样点 f = 0.75、P = 1024、K = 8，每次 `write_at` 里睡 2 ms（盘臂）或按线程 CPU 时钟自旋 2 ms（CPU 臂）。增量一律是对照各轮的中位 − 基线各轮的中位。盘臂：整条意图等盘增量落在写调用次数 × 2 ms 的 [1.0, 1.5] 倍里、CPU 变化在 ±10% 以内（没改）。CPU 臂两句各自报：CPU 增量落在写调用次数 × 2 ms 的 ±10% 里（没改）；写调用（注入所在的那一种调用）的等盘增量的绝对值 ≤ 写调用次数 × 2 ms × 10%（12.9 的新判法，分母是注入量）。原判据「整条意图等盘相对基线的变化在 ±10% 以内」照算照报（`wait_change`、`wait_clause_under_the_original_criterion_holds`），不作判定用。对照点每一轮里，每个设备臂挨着量基线、盘臂对照、CPU 臂对照各一次，次序按轮号轮换（第 1、4 轮 基线 → 盘臂 → CPU 臂，第 2、5 轮 盘臂 → CPU 臂 → 基线，第 3 轮 CPU 臂 → 基线 → 盘臂），轮数 5；按轮号配对的增量中位另报、不判。
- 作废与停机（登记第十一节）：原登记八条作废、12.10 第 ③ 条与第 1 次重跑的「计数特性被并进来」照沿用；修订 R2-1 加第三样触发（这条意图前后的校准比有一个 > 1.10，`invalid_reason=calibration_ratio_above_the_registered_edge`），R2-2 作废后先等常态再重试，R2-3 校准自己的阳性对照不过就停（退 4），R2-4 基线漂移只打标记；停机条款第 7 条（装置变了，输入清单第一行与登记的哈希对不上，另有修订 R2-8 在装置里也核一遍）至第 10 条（校准锚点不符）与修订 R2-5、R2-7 把基线读不到、锚点不符的遍数累计超过 3 遍收严到停（退 3）；修订 R2-6 校准读不到那一条意图记 `invalid_reason=calibration_unreadable`。这一趟这些都没触发（两段判决行 `stop_clause_findings=0`，140 + 60 条意图 `invalid_reason=none`）。
- 第三段的 B_批（沿用）取第 1 次重跑第一段内存臂、第 1 轮、`kind` 为 `batch` 或 `final_batch` 那几行的 `scan_read_bytes_in_the_region`；φ 与 n* 按两张卡各自拟合的 a、b 算。
- 产物不带执行那台机器的路径：第一段 `name=configuration` 不打 `--image-directory` 的值，只打 `image_directory_filesystem`（目录所在设备在 `/proc/self/mountinfo` 里的文件系统种类）、`image_directory_is_cleared_at_boot`（tmpfs / ramfs，或真路径落在 `/tmp` 下为 `true`）与 `image_device`（设备号）；镜像建不成的报错只带文件名（登记第十二节 12.6）；输入清单里第二台的目录换成「〔第二台的目录〕」。
- 键名写全名，登记里的提问与条款编号只当索引：第二段甲、乙、丙三段的份额是 `name=stage` 的 `crc_share`、`scan_increment_share`、`choose_share_per_batch`（签发那一次另报 `choose_share_of_the_issue`，丙段判定读 `name=choose_judgement`）；Q16 是第一段判决行的 `gc_to_sequential_write_bandwidth_maximum_ratio` 与 `gc_to_sequential_write_bandwidth_judgement`；F1、F2、F5 是 `clause_wall_clock_not_rising_with_the_batch_triggered`、`clause_sixteen_thread_form_misfits_the_gc_triggered`、`clause_two_cpu_paths_disagree_points`；Q17 是第四段判决行的 `foreground_fsync_latency_judgement`。新旧对照表在登记 12.6。

```

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:331-348`（整段抄，未转述）**

```markdown
### 它答不了的

- f = 0.25 量到的是被强制选定的那一段，不是 greedy 自己选中的段：改法甲下 greedy 选中的仍是装着别的单元的段（单点核第 3 行 `greedy_formed=false`），f = 0.25 与 f = 0.75 一样走只供测试的强制选段后备。这一页的第 1 行读数只对「被强制选定的、全是这次写的数据单元的段」成立，对 greedy 自己会选中的段没有读数。
- 改法甲造成的几何靠一条机理（根环 24 条根、旧落点在覆盖写那一次发布就记成已释放），机理核对只在四个点（f = 0.25、0.75 × P = 1024、8192）与单点核上对上；别的 f、P、别的覆盖写次序没试，也没试登记里的改法乙（快照留旧单元，装置没实现）。
- 占比 0.75 的池与第 1 次重跑不同（差在两棵树，「几何」一节），所以这一趟的数不与第 1 次重跑的数逐格比；七·6 前后的比法这一趟没有。
- 第 3、4 行沿用第 1 次重跑：第三段的 Q9–Q13 在 f = 0.25 的两点上量的是几何没造成的段（`batch_bytes=16384`、`merged_buffer_bytes=16` 不是显存占用：B_批 16384 字节不满一个 32768 字节的槽，装置把每批段数整除成 0 再算缓冲），f = 0.25 造成之后的 φ 与 n* 没量；第三段装置 `eadb929d…` 与重跑登记写死的 `b3a50d50…` 差的 3 行不在第三段路径上这一句是读 `git diff` 推的，没另量，主 agent 已认。
- Q9 照原登记算、照原样报；今天的 D26（后台整理与放置回收） 已定项 13 第 7 条写 D24（后台重活能不能卸给 GPU） 已定项 3 按挂钟比的判据不管它罩着的后台任务，Q9 在第 2 行还算不算翻面量，这一页不判，交主 agent，并进七·2 设计轮定（重跑登记第六节、第一节）。
- 第 1 行读数的 P 轴只有 1024、8192 两点（装置只收这两个池大小）：翻面发生在两点之间的哪里、更大的池上是什么，没量。
- Q4 的 `rises` 涨幅小（1.5%–7.4%）：「批大小是不是挂钟的主旋钮」这个问法这一页不下判断，F1 没触发只说明每一对都 > 1。
- 校准门槛 1.10 的依据只有本机上一个代用量（登记第四节第 11 行，zlib 的 CRC-32 在本机、负载 18 时的分布），第二台与装置自己的 CRC-32C 没量过；门槛在跑前写死，没看这一趟的数再改。这一趟没有一条意图越过它（最大 1.077323），所以它实际没动任何一格（N14 并列判定 24 行无一差异），也就不能说明门槛放在哪合适。追指针校准比（N15）每条意图都高于 1（1.23–1.53），没查原因。
- 校准只量测量线程此刻跑同一份活要多久，不排除对象自身（GC 路径）在个别意图里变慢；H3、H4 以 F5 打中为前提，这一趟没打中，两条没用上。F5 没触发也不能反推第 1 次重跑的 F5 是机器状态：这一趟的机器状态读数（`others_share` 最大 0.002502）与第 1 次重跑没有同一口径的对照读数。
- 第二段清场记录第一行是 `joined-and-read-back holders=1`：别的会话已登记同用卡，本趟加入而不是新停；计时窗口里别的会话有没有占第二台的 CPU，只能读 `name=machine_state` 的 `others_share`（最大 0.001875）与每条意图的排队等 CPU 读数，那是读数不是隔离。
- 释放前读盘核那一栏（登记第十二节 12.5）只认这一批搬迁源的槽。同一次发布对被换下的旧单元（重写的树节点这类）做的释放前读盘核走同一个函数、也在第一次写之前，落在 R 外，仍记在 `scan_read_bytes_elsewhere`：这一条是读 `transaction.rs` 推的，没量。分栏靠「区域扫描在释放前读盘核之前、扫描对 R 那块盘每个槽只读一遍」这个次序；调查员只在 f = 0.75、P = 128、K = 8 一个点上、只对 R 那块盘抓过调用方，另一块盘那一份是推的，单测端到端那一条在同一个点上钉住两块盘合计 2 × 搬的单元数 × 32768。甲段第二个子项不进 Q5、Q8、Q10 与第三段的 B_批，只报数。
- 几何后备：greedy 将选的段装着节点或打包单元时，登记只举了「多覆盖写那几段、让节点段的旧节点也被引用着」两种改法，装置只实现了前一种；竞争段装着别的单元时直接走后备、不试三次（登记第十二节 12.3 写死了这个读法），`name=geometry` 行写明原因；后一种改法没有试过。
- 第二段的饱和吞吐量的是重放（同一份输入反复跑），不是 GC 自己在多线程上跑：今天的 GC 整条路在调用线程上同步跑。
- 第一行阳性对照照 12.9 判「中位之差」，按轮配对的增量只报不判（这一趟 CPU 臂真盘格 `write_wait_increment_paired_median_nanoseconds=1684671.000000`，内存格 `67472.000000`）；交错之后配对判不判、怎么判没有定。
- 停机条款第 6 条（包含自检反过来）照登记字面判，但两个钟走得不一样快：线程 CPU 时钟不受校时调速，`Instant`（`CLOCK_MONOTONIC`）受。2026-10-19 执行员在本机用 python 量过（忙等、挂钟读数包着 CPU 读数）：200 ms 的窗口里 CPU 读数最多比挂钟多 1526 纳秒（比值最大 1.000007630），`CLOCK_MONOTONIC` 比 `CLOCK_MONOTONIC_RAW` 慢 7.036 ppm；挂钟换成 `CLOCK_MONOTONIC_RAW` 时 CPU 读数没有一次大过挂钟（200 ms 窗口最多 −194 纳秒）。几乎全在 CPU 上、又长到几百毫秒的调用（内存臂）因此可能让第 6 条触发，读钟那几十纳秒的余量盖不住；与线程 CPU 读数配对的挂钟已照 12.4 换成 `CLOCK_MONOTONIC_RAW`，这一趟两段判决行都是 `stop_clause_findings=0`，第 6 条没触发。

```

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:349-368`（整段抄，未转述）**

```markdown
### 路径与结论登记

同一个量两条或以上互不共享代码的路径各算一遍再比对的，有五样：

| # | 路径 | 怎么算 | 源码落点 | 读了哪些共用项 |
|---|---|---|---|---|
| 1 | 扫描读字节：设备计数 | 计时包装记下的、第一次写之前落在 R 那块盘 R 里的读，扣掉这一批搬迁源槽上的再读（释放前读盘核，登记第十二节 12.5），长度相加 | `crates/singlefs-implementation-experiments/src/bin/e185_gc_batch_wall_clock_split/sampling_point_pool_and_geometry.rs:81 split_the_reads_of_a_call` | 区域 R（会话签发的那一条意图）、槽字节 16384、这一批搬迁源的槽（会话交回的 `units_relocated_by_this_publish` 的 `source`）、读的次序（扫描在释放前读盘核之前） |
| 2 | 扫描读字节：分配记录 | 调用之前那一版 R 那块盘上未释放记录的 Σ 跨度 × 16384 | `crates/singlefs-implementation-experiments/src/bin/e185_gc_batch_wall_clock_split/timed_block_device.rs:160 expected_scan_read_bytes` | 同一个 R、同一个槽字节常量 |
| 3 | 批数：装置分类 | 每次调用发成的那一版里 logged_ops 还有没有意图、搬了几个单元 | `crates/singlefs-implementation-experiments/src/bin/e185_gc_batch_wall_clock_split/timed_block_device.rs:100 kind_of_the_call` | 会话交回的那一版 |
| 4 | 批数：会话计数 | 会话 `observations.batches_published` 的增量 | `crates/singlefs-implementation-experiments/src/bin/e185_gc_batch_wall_clock_split/intent_voiding_and_result_lines.rs:33 stop_findings_of` | 无（会话自己数） |
| 5 | 批数：闭式 | ⌈m ÷ K⌉ | `crates/singlefs-implementation-experiments/src/bin/e185_gc_batch_wall_clock_split/call_kinds_and_byte_anchors.rs:69 batches_of_one_intent` | 活槽占比定的 m |
| 6 | 一批的 CPU：真盘臂线程 CPU 时钟 | 真盘上一批的本线程 CPU | `crates/singlefs-implementation-experiments/src/bin/e185_gc_batch_wall_clock_split/main.rs:759 thread_cpu_nanoseconds` | 本线程 `clock_gettime(CLOCK_THREAD_CPUTIME_ID)`（clock id 3） |
| 7 | 一批的 CPU：内存臂挂钟 | 内存盘上同一批的挂钟（等盘近 0） | `crates/singlefs-implementation-experiments/src/bin/e185_gc_batch_wall_clock_split/timed_block_device.rs:296 cpu_reconciliation` | 同一份配方造的池；`Instant`（`CLOCK_MONOTONIC`），与路径 6 的钟走得不一样快（停机条款第 6 条那一条，口径里） |
| 8 | 造成段数：逐段成分 | 单元区里每一段（有未释放记录的段）逐段数成分，数「造成」为真的段（`name=segment_composition_summary` 的 `formed_segments`） | `crates/singlefs-implementation-experiments/src/bin/e185_gc_batch_wall_clock_split/pool_construction.rs:451 construction_reading_lines` | `is_the_geometry_formed`（活槽 = 64f 且全是这次写的数据单元，第 2784 行）、`composition_of_the_segment`、同一份分配记录 |
| 9 | 造成段数：几何核那条路数 | 对着可写挂载后的那一版，按造池记下的数据单元槽所在的段起点筛「造成」为真的段（`geometry_formed_segments`） | `crates/singlefs-implementation-experiments/src/bin/e185_gc_batch_wall_clock_split/sampling_point_pool_and_geometry.rs:166 predict_the_greedy_choice` | 同路径 8：`is_the_geometry_formed`、`composition_of_the_segment`、同一份分配记录 |
| 10 | 落进旧洞的副本：逐次数 | 造池第 2 步每次覆盖写之后，看这一份新副本的槽是不是前面某次覆盖写空出来的旧槽（`copies_landing_in_holes_of_this_construction`） | `crates/singlefs-implementation-experiments/src/bin/e185_gc_batch_wall_clock_split/pool_construction.rs:134 landings_of_the_copies` | 覆盖写的次序（改法甲的 `overwrite_order_of`）、每次覆盖写前后的槽号 |
| 11 | 落进旧洞的副本：闭式推的预测 | `overwritten_units − 24`（24 = 根环条数） | `crates/singlefs-implementation-experiments/src/bin/e185_gc_batch_wall_clock_split/pool_construction.rs:158 predicted_copies_landing_in_holes` | 本地常量 `LOCAL_ROOT_RING_ENTRIES`（开跑与 `singlefs_format` 的两个常量之积回比）；机理前提：每次覆盖写一次发布、一条根，洞从第 j + 24 次发布起分得到，用户数据取最低的空槽对 |

源码落点的行号按主工作区那一份装置（sha256 前 16 位 `0bd2274383f12e88`）现查。跨路径的比对：1 与 2 由停机条款第 2 条逐批连起来（`stop_findings_of`，对不上就停）；3 与 4 由停机条款第 3 条连起来，3 与 5 只在几何造成了的点上连（同一函数）；6 与 7 是 Q3，> 20% 记第十条 F5，不停；8 与 9 由 `counts_agree` 连起来（`construction_reading_lines` 里 `formed_segments == prediction.geometry_formed_segments.len()`，单测 `construction_reading_lines_compare_the_two_counts_of_formed_segments_and_report_the_landing`，第 13124 行），对不上写 `counts_agree=false`、不停机，8 与 9 共用同一个判成造的函数，这个函数错了两边一起错，由「甲造成」那一格 `live_slots=16 slots_of_this_workload=16 other_slots=0` 的绝对值兜；10 与 11 只在 `name=overwrite_landing` 里并列打出，对不上记第十条 H2、不停机不作废，单测钉了闭式的四个值（第 12344–12347 行），没有一处断言把逐次数的与闭式的连起来，还欠这一条。取值是枚举点（两个 f、两个 P、三个 K、两个设备；8 至 11 是四个 f、P 点），理由是登记第八节只要求这几档；1 与 2 共用同一个 R，R 取错时两边一起错，由第七节「签发那一步扫 R 读到的字节」的绝对值锚点兜。

```

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:369-377`（整段抄，未转述）**

```markdown
### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D26（后台整理与放置回收） 已定项 13 | 支撑 | 2026-11-19 不受影响：只把页首现状那一行的装置 sha256、行数与测试条数改成现值，没有重跑，结论、数与产物不变；此前 2026-11-05 不受影响：改名只动措辞，结论、数与产物不变；此前 2026-11-03 不受影响：只改复跑写法，结论、数与产物都不变；此前 2026-11-02 不受影响：只改复跑一节的写法（派哪台、编译目录与镜像目录照共享的树改写，命令补上 `--disk-model`），结论、数与产物不变；此前 2026-11-01 不受影响：只改复跑命令写法，结论、数与产物不变；此前 2026-10-26 改了：第 3、4 次重跑的读数（一次 GPU 提交的主机 CPU，按 CPU 写法、卡、提交间隔拆开的不赔格数，主 agent 现数的表在 `research/prompts/m4-gpugc-design-r3-main-verification.md` 第二节）进了该分项依据段，撑第 1 条放上 GPU 的三段与射程里快写法下赔的卡与间隔；此前 2026-10-24 不受影响：补上第 3、4 次重跑产物点名，数与结论不变；此前 2026-10-23 不受影响：第 2 次重跑给出的是里程碑四设计轮要的读数（问题单第 1 行：池小（P = 1024）两边都占、池大（P = 8192）CPU 限；第 2 行：甲、乙、丙三段份额都不到 10%，F7 打中；第 3 行沿用第 1 次重跑 Q13 全判要攒批；第 4 行单跑够，比值 0.015977），该分项射程把「GPU 上算 GC 的哪几段」与定时兜底间隔、写带宽上限的默认值留给设计轮与 C412（整理的挂钟代价没量过），依据段是「无实验」、没引 E185（GC一批搬迁的挂钟拆分）；Q9 的用法与第 7 条是否相左，这一趟没重算（第三段沿用），Q9 算不算翻面量交主 agent，并进七·2 设计轮定；决策正文这一次没有改 |
| D26（后台整理与放置回收） 已定项 1 | 备料 | 2026-11-19 不受影响：只把页首现状那一行的装置 sha256、行数与测试条数改成现值，没有重跑，结论、数与产物不变；此前 2026-11-05 不受影响：改名只动措辞，结论、数与产物不变；此前 2026-11-03 不受影响：只改复跑写法，结论、数与产物都不变；此前 2026-11-02 不受影响：只改复跑一节的写法（派哪台、编译目录与镜像目录照共享的树改写，命令补上 `--disk-model`），结论、数与产物不变；此前 2026-11-01 不受影响：只改复跑命令写法，结论、数与产物不变；此前 2026-10-24 不受影响：补上第 3、4 次重跑产物点名，数与结论不变；此前 2026-10-23 不受影响：这一趟装置照旧读它的停机谓词（R 按记录全空）判最后一批，不量三水位阈值；f = 0.25 几何这一趟用改法甲造成了，是负载次序的事，与触发判据无关；该分项依据段没引 E185（GC一批搬迁的挂钟拆分），决策正文这一次没有改 |
| D24（后台重活能不能卸给 GPU） 已定项 3 | 备料 | 2026-11-19 不受影响：只把页首现状那一行的装置 sha256、行数与测试条数改成现值，没有重跑，结论、数与产物不变；此前 2026-11-05 不受影响：改名只动措辞，结论、数与产物不变；此前 2026-11-03 不受影响：只改复跑写法，结论、数与产物都不变；此前 2026-11-02 不受影响：只改复跑一节的写法（派哪台、编译目录与镜像目录照共享的树改写，命令补上 `--disk-model`），结论、数与产物不变；此前 2026-11-01 不受影响：只改复跑命令写法，结论、数与产物不变；此前 2026-10-24 不受影响：补上第 3、4 次重跑产物点名，数与结论不变；此前 2026-10-23 不受影响：Q9 沿用第 1 次重跑第三段（72 行全判 GPU 输、F2 没触发），这一趟没重算；第二段又量了 CPU 一侧的 16 线程饱和吞吐（甲 48832371460.2–49032300970.9 B/s），没有代进该分项的形式；该分项依据段没引 E185（GC一批搬迁的挂钟拆分）；Q9 算不算问题单第 2 行的翻面量交主 agent；决策正文这一次没有改 |
| D24（后台重活能不能卸给 GPU） 已定项 2 | 不影响 | 2026-11-19 不受影响：只把页首现状那一行的装置 sha256、行数与测试条数改成现值，没有重跑，结论、数与产物不变；此前 2026-11-05 不受影响：改名只动措辞，结论、数与产物不变；此前 2026-11-03 不受影响：只改复跑写法，结论、数与产物都不变；此前 2026-11-02 不受影响：只改复跑一节的写法（派哪台、编译目录与镜像目录照共享的树改写，命令补上 `--disk-model`），结论、数与产物不变；此前 2026-11-01 不受影响：只改复跑命令写法，结论、数与产物不变；此前 2026-10-26 不受影响：该分项射程补了一句一次 GPU 提交的实测挂钟量级（第 4 次重跑约 5.3 ms），依据段没引 E185（GC一批搬迁的挂钟拆分），条款判据不改；此前 2026-10-24 不受影响：补上第 3、4 次重跑产物点名，数与结论不变；此前 2026-10-23 不受影响：只作定义引用（GPU 结果要复核、GPU 与 CPU 两份实现）：第三段逐段对拍 CRC 的 32 行 `crcs_agree=true` 是第 1 次重跑第三段的装置作废检查，这一趟第三段不跑，不量运行时两份 GPU 核各算一次比对的复核政策，结论不变 |

```

**出处 `.claude/kb/tooling.md:1-7`（整段抄，未转述）**

```markdown
# 工具与环境事实

**中文** · [English](tooling.en.md) · [日本語](tooling.ja.md)

**tooling.md 放「本工程用到的工具现在是什么状态、用的时候怎么做」**。
规则不进 tooling.md，进 [CLAUDE.md](../../CLAUDE.md) 或 `.claude/rules/`。

```

**出处 `.claude/kb/tooling.md:8-26`（整段抄，未转述）**

```markdown
## 本地 LLM（三方论证不派它当腿）

三方论证只派两条云端腿加主 agent 判决，不派本地模型的腿。仓里调网关的脚本与字词损坏检测器（`ask-local.sh`、`ask-local-selftest.sh`、`local-repro.sh`、`local-repro-tuned.sh`、`corruption-check.py`、`corruption-summary.py`、`oov-check.py`、`en-words.txt`）2026-10-11 删掉了；要再用，`git log --diff-filter=D -- scripts/agents/ask-local.sh` 找到删它的那次提交，从它的上一版取回。

| 项 | 值 |
|---|---|
| 入口 | 本机的本地模型网关（OpenAI 兼容，地址与所在目录不进仓） |
| 鉴权 | 请求头 `X-Api-Key`，取值在本机私有配置里，不进仓 |
| 模型 | `local` → `Qwen3-Next-80B-A3B-Thinking-AWQ-4bit`，`max_model_len` 262144 |
| 调用脚本 | 仓里没有了（取回办法见本节开头那一段） |
| 预算 | 不传 `max_tokens` 与 `thinking_token_budget` |

用它的时候：

- 提示一律用英文，不用 markdown 强调（`**粗体**:`）；不靠 `presence_penalty`、`repetition_penalty` 修中文复读。
- 它给出的具体字段名、结构名、机制描述一律当待核线索；核不动整条丢掉。
- 提示里把最容易上钩的类别错误写出来并禁止：「同一系统在两种介质上」不许答成「两个不同系统」；「宽度在别处定」写明「推迟的是宽度，不是字段本身」。
- 它答「举不出来」时跑阳性对照：同一份提示只把被问对象换成已知答案是「有」的对象，这次举得出来，那个「举不出来」才算数。

```

**出处 `.claude/kb/tooling.md:27-38`（整段抄，未转述）**

```markdown
## 仓里的性能数都只在本机成立，引用要带机器口径

**射程**：挂钟、吞吐、每次发布多少毫秒、门禁跑多久、冷挂载多少秒、并行加速比，凡是时间与速率。

**怎么做**：

- 引一个性能数时把**机器口径**一起带（哪台机、什么盘、什么负载、空不空闲），不带口径的数不许进判据。
- 拿本机的数去定**阈值、上界、超时**时，写明「这是本机值，别的机器上要重量」。
- 需要一个跨机器成立的数时，要么量两台以上，要么写成条件式（「按 X 算是 Y，那一档本机测不到，仍是假设」）。

门禁那一档整环清零的耗时、冷重开恢复的耗时、崩溃点重放两条流的挂钟、E152（按里程碑对比六家文件系统的文件性能） 与六家的全部对比数都是本机数（C486（整环清零的 55 秒与冷重开的 17 秒都没人管））。

```

**出处 `.claude/kb/tooling.md:39-45`（整段抄，未转述）**

```markdown
## 跑项目代码的命令一律经 task-server 交

- 统一任务服务（`task-server/`）是起跑项目代码的任务（编译、测试、门禁、实验、自证；CPU、GPU 都算）的唯一入口：挑机器、内存额度、线程、排队次序都由服务定，提交时不收。服务只调配 CPU 与内存，不分卡、不领卡、不清场：卡归 GPU 批处理验证服务（`gpu-verification-server/`）。用法、判法与拒绝清单在 `task-server/README.md`，不抄第二份。
- 调用方用客户端 `task-server/task` 的七个子命令：`submit [--cwd <目录>] -- <命令…>` 交、打出任务号；`confirm <号> --items <N> --tasks <M>` 核对确认单之后确认（交了的任务先待确认，确认了才进队，不确认的满 30 分钟服务取消），`confirm <号> --why '<…>'` 确认一条跑满两小时的任务确是要跑这么久；`wait <号>` 等到结局（末行 `TASK_SERVER_RESULT …`，退出码是任务的）；`status [<号>]` 看在排、在跑与排队原因；`log <号>` 看最后一次尝试的日志；`cancel <号>` 停；`clean <号>` 删这条任务留下的编译目录（第二台那一趟的编译目录由 `multi-host-run.sh` 跑完自己删，`clean` 不管）；任务自己在跑的那一次尝试里按需领卡、还卡用 `lease take`、`lease return`（只认文件头声明了 `# task-server-card-lease: on-demand` 的入口），调用方不起。命令里不套包装、不写资源参数，写了当场拒（`task-server/README.md`「提交写法」）。
- 子 agent 一律不交任务：`submit`、`wait`、`cancel`、`clean` 只由主 agent 起，子 agent 要跑的写进报告交主 agent（用户 2026-11-01 定，`records/2026-11-17-subagent账本自上而下清欠.md` 第一百七十四节）。
- 出了问题看哪里：结局看 `wait` 末行的 `type=`（错误类型与退出码的表在 `task-server/task_server_lib.py` 文件头「错误类型」）；排着不起看 `task-server/task status <号>` 的「在等：…」；跑挂了看 `task-server/task log <号>`（`--all` 整份）；`wait` 退 78 是连不上服务，先 `task-server/task status <号>` 看任务还在不在，不重交。

```

**出处 `.claude/kb/tooling.md:46-50`（整段抄，未转述）**

```markdown
## 变异工具：并发不超过核数，不限时，卡住由统一任务服务判

- 并发度乘以一条被测单位自己占的核数，不超过核数。cargo-mutants 跑本仓用 `--jobs 1`。
- `scripts/mutation/mutate.sh`、`scripts/mutation/broad-mutation.py`、`scripts/mutation/mutation-prune.py` 不给一条变异或一步设限时，也没有超时这一种结局：整条命令是统一任务服务（`task-server/`）的一个任务，卡住与死循环由服务判、停掉，记 `stalled`、打回提交它的会话（判法在 `task-server/README.md`）。

```

**出处 `.claude/kb/tooling.md:51-61`（整段抄，未转述）**

````markdown
## 变异条数会随源码漂，不许在 kb 里钉死

正文里写数它的命令，不只写数：

```
cargo mutants --list --package singlefs-checker | wc -l
cargo mutants --list --package singlefs-core | wc -l
```

要引一个具体的数，连同量它的日期写成事件句（「<日期> 数出 N 条」），不写成现状句。

````

**出处 `.claude/kb/tooling.md:62-65`（整段抄，未转述）**

```markdown
## 改了引用方，要 grep 全仓改被引用方

改正一处措辞之后，说「已全部改完」之前，先 `grep -rn` 全仓数一遍。范围含 `kb/` 全部子目录、`research/prompts/`（喂给三方论证的背景材料）、`scripts/` 与 `research/` 下的源码注释、`records/`、门禁脚本。数不清就写「改了这几处，没有全仓核过」。

```

**出处 `.claude/kb/tooling.md:66-70`（整段抄，未转述）**

```markdown
## 一个数不许从「它自己判为非法」的配置里算出来

- 算一个上限或下限之前，先确认那个配置在本工程的判定下是合法的。
- 撤回一个数时，作废理由写成真实的那一条（缺输入、配置非法、口径错），不写成它保留下来的别的档同样有的性质。

```

**出处 `.claude/kb/tooling.md:71-74`（整段抄，未转述）**

```markdown
## 未定项一律写成编号条目，标题一律声明条数

按 `^\d+\.` 提取的脚本只认编号条目；`.claude/gate.d/doc-decisions.sh` 的 kb-shape 格的条数检查只在标题声明了「N 项未定」时启动。

```

**出处 `.claude/kb/tooling.md:75-87`（整段抄，未转述）**

```markdown
## 验证产物放哪：四个位置，各管一段

| 位置 | 放什么 | 谁读 | 进仓库 |
|---|---|---|---|
| `.claude/kb/experiments.md` | 结论、口径、判据、复现命令、**判决性的那几行数字** | 模型检索 | 进 |
| `records/` | 每一轮的叙述：定了什么、推翻了什么、依据 | 人通读 | 进 |
| `scripts/` | 实验代码与脚本 | 跑的人 | 进（`.gitignore` 只排除 `target/` 与个别超大文件） |
| `research/results/<实验>-<日期>.out` | 原始输出全文 | 下一轮复跑时逐行比对 | 进；早于最近一轮的在下一次提交时归档进 git 历史（门禁 doc-experiments 的 archive-past-rounds 格） |

- 判决性的数字贴进 kb 正文。
- 每份原始输出要能自证是产出它的那一次运行留下的：收尾行 `E7RESULT name=done emitted=N` 里的 N 等于文件里 `E7RESULT` 的行数，对不上整轮作废。归档时核一遍：
  `for f in research/results/*.out; do echo "$(grep -c '^E7RESULT' $f) vs $(grep 'name=done' $f)"; done`

```

**出处 `.claude/kb/tooling.md:88-100`（整段抄，未转述）**

```markdown
## 内核源码：本机有完整树，不必 WebFetch

| 项 | 值 |
|---|---|
| 路径 | `~/kbuild/linux-om`（1.8 GB） |
| 版本 | **Linux 7.2.0**（`Makefile` 的 `VERSION = 7` / `PATCHLEVEL = 2` / `SUBLEVEL = 0`） |
| 有什么 | 完整 `.c`/`.h` 正文，含 `fs/jbd2/`、`fs/xfs/`、`fs/btrfs/`、`fs/erofs/`、`fs/ext4/`、`fs/f2fs/`、`drivers/nvme/`、`drivers/md/`、`crypto/`、`Documentation/` |
| **没有什么** | **`fs/bcachefs/` 不在这棵树里**（见 [decisions.md](decisions.md) D7（是否进 Linux 主线））。`/usr/src/linux-headers-6.17.0-1028-oem/fs/bcachefs/` 只有 `Kconfig` + `Makefile`，**没有源码** |

- 查源码用 `~/kbuild/linux-om`，不用 `/usr/src/linux-*-headers-*`（那里只有 `Kconfig` 与 `Makefile`）。
- 引哪棵树就标哪个版本号，两个版本的结论不混着引。kb 里标「现查 Linux 6.17」的引用来自 GitHub 的 `v6.17` tag，本机没有那个版本。
- OpenZFS 本机没有树，现取 `https://raw.githubusercontent.com/openzfs/zfs/master/...`，引用时记下取回日期，行号不是永久的。

```

**出处 `.claude/kb/tooling.md:101-106`（整段抄，未转述）**

```markdown
## 到点自己开工：用 remind.sh 登记

用户要会话到点自己开工时，只用 `bash scripts/agents/remind.sh start --every <分钟> --message <提醒内容> --once`（Bash 的 `run_in_background` 起，到点退出叫醒会话；规则 `.claude/rules/timer-only-remind.md`，写法 `.claude/skills/remind/SKILL.md`）；不用 CronCreate，也不在 Bash 里只为定时而写 `sleep`。提醒内容写开工指令所在的路径，第一步用 `set -o noclobber` 排他建「已开工」标记，标记已在就什么都不做（`remind.sh` 重起时可能重复提醒）。

- 提醒只活在当前会话里，告诉用户会话不能关、机器不能休眠。

```

**出处 `.claude/kb/tooling.md:107-117`（整段抄，未转述）**

```markdown
## Rust 工具链

| 项 | 值 |
|---|---|
| 版本 | rustc 1.98.0 / cargo 1.98.0 |
| 安装方式 | rustup，`--profile minimal --default-toolchain stable`，装在宿主 `~/.cargo` |
| 额外目标 | `x86_64-unknown-linux-musl`，实验二进制（`scripts/experiment-apparatus/` 下的包）编成静态的在虚机的 busybox initramfs 里跑；验证包的真设备二进制 `new_pool_file_creation_on_device` 不走它（验证包默认编 RocksDB 的 C++，本机没有 musl 的 C/C++ 编译器），改用 `--target x86_64-unknown-linux-gnu` 加 `+crt-static` 编成 static-pie，命令在 `.claude/gate.d/qemu-device-streams.sh` |
| 组件 | `rustfmt` + `clippy`（`--profile minimal` 不含它们，要另装） |
| **PATH** | rustup 的 shim 软链在 PATH 上的用户级可执行文件目录里（cargo / rustc / rustup / rustdoc / rustfmt / cargo-fmt / cargo-clippy）；门禁与脚本用 `bash -c` 起，不读 shell 的启动文件，PATH 不靠改 dotfile |
| release 溢出检查 | 树根 `Cargo.toml` 与 `scripts/experiment-apparatus/Cargo.toml` 的 `[profile.release]` 都设了 `overflow-checks = true`；profile 只认工作区根，写在成员 manifest 里不生效 |

```

**出处 `.claude/kb/tooling.md:118-145`（整段抄，未转述）**

```markdown
## RocksDB 与 snappy 的预编库：每台主机编一次，取值只走进程环境

librocksdb-sys 的 C++（`librocksdb.a` 与 `libsnappy.a`）每台主机只编一次，各编译目录经环境变量链这一份，不在每个编译目录里现编。工具是 `scripts/hosts/rocksdb-prebuilt.py`，用法写在它的文件头（它不认 `--help`，退出码 2）。子命令都在仓根跑，`--root R` 不给就是这份脚本所在的仓。

| 子命令 | 做什么 |
|---|---|
| `key` | 打这台主机此刻的键（各组成）与预编库目录，不编 |
| `env` | 预编库在、对得上时打四行 `export`（给 `eval` 用）；不在退 1、标准输出为空 |
| `build` | 预编库不在就编一次，持锁：几个进程同时来只编一次，后来的等锁之后看到编好了直接退 0；起真 cargo，是重活，经 `task-server/task submit -- python3 scripts/hosts/rocksdb-prebuilt.py build` 交（`task-server/README.md`） |
| `install` | `build`，再写这台主机的三处环境来源、改用户级 systemd 管理器此刻的环境，最后跑 `check`；同样是重活 |
| `check` | 判这台主机：两个工作区（`crates/singlefs-verification/Cargo.toml` 与 `scripts/experiment-apparatus/Cargo.toml`）的依赖对得上、预编库在且对得上、三处环境来源与此刻进程环境的取值都与预编库一致、cargo 配置里没写这四个变量 |
| `audit-target <编译目录>…` | 判编译目录：librocksdb-sys 构建脚本那一次拿到的取值是不是这台的预编库、`out/` 里有没有现编过的 C++、同一把键下见过几种取值；一份构建脚本记录都没找到退 77（本次未判） |
| `entry [--cargo C]` | 起 cargo 的入口开头调的轻判（谁调、取值缺了怎么办写在「起 cargo 的入口」那一段） |

**预编库放哪**：`${SINGLEFS_ROCKSDB_PREBUILT_STORE:-~/.local/share/singlefs/rocksdb-prebuilt}/librocksdb-sys-<版本>/<键的前 16 位>/lib/{librocksdb.a,libsnappy.a}`；同目录 `manifest.json` 记键的各组成与两份库的 sha256，`complete` 最后写（有它才算编好）；不放开机清空的目录。`SINGLEFS_ROCKSDB_PREBUILT_STORE` 只给自证换临时目录用。键由两个工作区的 rocksdb 依赖写法、两份 `Cargo.lock` 里相关包的版本与 checksum、`rustc -vV`、c++ / cc / ar 的真实路径与版本、交给 C++ 的指令集、编法组成，哪一样变了键就变、要重编。

**取值从哪来**：四个变量 `ROCKSDB_LIB_DIR`、`ROCKSDB_STATIC`、`SNAPPY_LIB_DIR`、`SNAPPY_STATIC` 的取值写在三处环境来源里，都由 `install` 写：

| 位置 | 作用 |
|---|---|
| `~/.config/singlefs/rocksdb-prebuilt.env` | 唯一的取值来源：四行 `export`，外加 `unset ROCKSDB_COMPILE`、`unset SNAPPY_COMPILE`；别手改，改了重跑 `install` |
| `~/.bashrc` 最前面一段（三行，`# >>> singlefs rocksdb-prebuilt >>>` 到 `# <<< singlefs rocksdb-prebuilt <<<`） | `source` 那份 env 文件；放在交互判断之前，所以交互 shell、从它起的会话与 ssh 起的非交互命令都读得到 |
| `~/.config/environment.d/60-singlefs-rocksdb-prebuilt.conf` | 用户级 systemd 管理器起的单元读（task-server 为每一次尝试起的单元，`task-server/README.md`）；`install` 另跑 `systemctl --user set-environment`，让此刻在跑的管理器当场换上 |

**四个变量只走进程环境**：不写进 `.cargo/config.toml` 或别的哪一层 cargo 配置的 `[env]`（写了 `check` 判红；理由在 `.cargo/config.toml` 开头的注释）。`ROCKSDB_COMPILE`、`SNAPPY_COMPILE` 一律不设，设成 1 就在编译目录里现编。

**起 cargo 的入口**：门禁阶段、崩溃与变异驱动、`merge-and-verify.sh`、`admission.py` 交的命令、第二台那一侧，开头都调 `entry`。它只判三样：这台装了（预编库在、对得上，不算两份库的 sha256）、此刻进程环境的四个取值与预编库一致、没设 `ROCKSDB_COMPILE` 与 `SNAPPY_COMPILE`；不判三处环境来源（那是 `check` 的事）。取值缺了或对不上，入口当场拒、一个 cargo 都不起，逐项写出出路。出路：此刻进程没有取值的，开新会话（`install` 之后起的登录 shell 读 `~/.bashrc` 最前面那一段），或在命令前 `eval "$(python3 scripts/hosts/rocksdb-prebuilt.py env)"`；这台还没装的先交 `task-server/task submit -- python3 scripts/hosts/rocksdb-prebuilt.py install`（`task-server/README.md`）。要编的那棵树里没有依赖 rocksdb 的工作区时（编不到 librocksdb-sys，门禁样本与自证的小仓多是这样），`entry` 不判、退 0。

```

**出处 `.claude/kb/tooling.md:146-160`（整段抄，未转述）**

```markdown
## 虚机 benchmark harness（E7（离线索引 harness） 用）

| 项 | 值 |
|---|---|
| 脚本 | `bash scripts/devices/vm-bench.sh <静态二进制>` / `--selftest` |
| 它是什么 | 项目自己的虚机装置：给虚机挂 virtio 盘（稀疏文件落 `TMPDIR`）并把二进制塞进 initramfs |
| 完整性闸 | 被测程序收尾行报 `name=done emitted=N`，宿主抓到的条数必须等于 N，否则整轮作废 |
| 默认规格 | 2048M 内存 / 4 vCPU / 4096M 盘 / 900s 超时，均可用环境变量覆盖 |

- 抓结果行不锚定行首（控制台首行前面有 BIOS 的转义序列），条数靠完整性闸核。
- 虚机里裸设备的顺序写吞吐不稳定，不作为任何判断的输入。
- 跑批脚本作废一轮时，把该轮输出的尾部一并记下。

---

```

**出处 `.claude/kb/tooling.md:161-197`（整段抄，未转述）**

````markdown
## 造几何量不同的设备：不需要真硬件

本机具备造异构设备池的全部手段，[decisions.md](decisions.md) D12（目标介质） 已定项 3「设备级几何量该放哪、异构池支不支持」可以实测。

### QEMU 路线（推荐：无需 root，直接接现有 harness）

`zoned.zone_size` 与 `logical_block_size` 挂在 **`nvme-ns`（命名空间）** 上，不是挂在控制器上，同一个控制器下的多个命名空间可以有不同的几何量。

已实测通过（QEMU 8.2.2，退出码 0，三个设备全部构造成功）：

```
-device nvme,serial=A,id=nv0
-device nvme-ns,drive=d1,bus=nv0,nsid=1,zoned=true,zoned.zone_size=64M,logical_block_size=4096,physical_block_size=4096
-device nvme-ns,drive=d2,bus=nv0,nsid=2,zoned=true,zoned.zone_size=128M,logical_block_size=4096,physical_block_size=4096
-device nvme-ns,drive=d3,bus=nv0,nsid=3,logical_block_size=512,physical_block_size=512
```

一条命令同时造出：**异构 zoned 池**（64M vs 128M zone）、**512e/4Kn 混用**、**zoned 与常规混用**。

- `logical_block_size` 必须配 `physical_block_size`，只给前者会被拒：`logical_block_size > physical_block_size not supported`。
- 来宾内核看不看得到三份不同的几何没有验过：要进来宾读 `/sys/block/*/queue/{logical_block_size,chunk_sectors}` 与 `blkzone report`。

### 宿主侧路线（要 root，胜在不用启虚机）

| 手段 | 造什么 | 关键参数 |
|---|---|---|
| `null_blk` | zoned 设备，内存后端 | `zoned=1 zone_size=<MB> zone_capacity zone_nr_conv zone_max_open`。模块参数是全局的，**每设备各自的几何要走 configfs**（`/sys/kernel/config` 本机已挂载） |
| `zloop` | **文件后端**的 zoned 设备 | 模块在 `drivers/block/zloop.ko.zst`，本机有 |
| `scsi_debug` | 任意扇区大小 + ZBC | `sector_size=<bytes> zbc=host-managed zone_cap_mb zone_nr_conv` |

运行内核 6.17.0-1028-oem 的相关配置：`CONFIG_BLK_DEV_ZONED=y`、
`CONFIG_BLK_DEV_NULL_BLK=m`、`CONFIG_BLK_DEV_ZONED_LOOP=m`、`CONFIG_SCSI_DEBUG=m`。

⚠️ 本机唯一的真实块设备是 `nvme0n1`（3.6T，512/512，非旋转）——
**没有真的旋转盘、没有真的 4Kn 盘、没有真的 zoned 盘**，所有相关数字都只能是模拟出来的，
引用时必须带这个口径。

````

**出处 `.claude/kb/tooling.md:198-203`（整段抄，未转述）**

```markdown
## `scripts/mutation/mutate.sh` 的结局分类

- 结局：`✅ [抓到]`（红了）/ `❌ [没红]`（测试盲区）/ `⏭ 变异导致编译失败（无效变异，不计入盲区也不算命中）`；另有 `💥`（测试进程没跑完）、`⚠️`（测试进程报了失败，却一个失败的测试名都没抓到：抓名字的规则有盲区）与锚点失效（`⚓ [锚点腐化]`，原文命中不是恰好一次的那一条不跑、不入判定库，别的条照跑，整轮退 3）几类，各自单列；收尾「计数：」一行报抓到、没红、无效、没跑完、抓名字有盲区、锚点失效六栏（全部结局见 `scripts/mutation/mutate.sh` 文件头与 `--selftest`）。
- 上限与限时：每次 cargo test（基线、每条变异、还原之后那一次）直接起，不设内存上限、不限时；整条 `mutate.sh` 交统一任务服务（`task-server/`），一整条一个任务：内存上限由服务按记录给、撞了放大重排；死循环的变异由服务停掉整条任务（用时远超记录而日志又满 10 分钟不涨就停掉、打回）（`scripts/mutation/mutate.sh` 文件头「上限、限时与卡住」）。
- 改了判定工具，拿已知答案的整套语料回归一遍，不只跑新加的那几条。

```

**出处 `.claude/kb/tooling.md:204-217`（整段抄，未转述）**

```markdown
## 几个会话共用一个仓时怎么暂存与提交

门禁红了先分清红的是谁的改动，再动手；不为了让门禁变绿去改别人在飞的文件。规矩在上游 `session-wrapup.md` 第 4 节，本仓的做法：

| 事 | 怎么做 |
|---|---|
| 只暂存自己的块 | `scripts/gate/stage-mine.py --match 我的标记 --foreign 别人的标记 文件…`，先 `--dry-run` 看它挑了哪几块 |
| 暂存区也是共用的 | 暂存前先确认 `git diff --cached --name-only` 为空；暂存完、提交前再核一遍名单与逐文件 cached diff，然后不带路径地 `git commit` |
| 不要 `git commit -- <路径>` | 带路径时 git 提交的是这些路径在工作区里的内容，不是暂存区 |
| 同一行两边都改了 | stage-mine 报「进暂存区的一块里有命中 --foreign 的行」时一行切不开：对方在那份文件里只做了可逆替换（改名）的，拿工作区版本把对方的替换反做一遍、与 `git show HEAD:文件` 比 diff 确认只剩自己的，再 `git hash-object -w` + `git update-index --cacheinfo 100644,<sha>,<路径>` 放进暂存区，工作区不动 |
| 在 worktree 上提交 | 在 worktree（HEAD 上）用脚本重放这次要提交的全部改动 → 与主工作区逐文件比对（不同的部分应当全是对方的；比对时 `git -c core.quotePath=false`，否则中文路径被转义、`cmp` 全报不同）→ 在 worktree 跑整道门禁并提交 → 主仓里 `git update-ref refs/heads/master <新提交> <旧 HEAD>` 再 `git read-tree <新提交>`，工作区不动、对方的改动原样留着 |
| 看清哪些不是这次要提交的 | `git -c core.quotepath=false status --short`，中文路径默认被转义成八进制，grep 中文关键词会漏 |
| 两边改动交织在同一文件 | 一个提交带全部，消息里写明（拆开提交会让门禁 doc-registries 的 experiment-refs 格与门禁 doc-experiments 的 experiment-orphans 格判红） |

```

**出处 `.claude/kb/tooling.md:218-223`（整段抄，未转述）**

```markdown
## 复跑与变异脚本要在 `scripts/experiment-apparatus/` 下跑

- `scripts/mutation/mutate.sh` 在 `scripts/experiment-apparatus/` 下跑：`cd scripts/experiment-apparatus && bash ../mutation/mutate.sh <bin 名> <源文件> ../mutation/tables/<变异表>`。
- `replay.sh` 只认带 E 的实验号（`E` 加数字），裸数字退出码 2。
- `replay.sh` 每一趟编进它在 `${TMPDIR:-/tmp}/singlefs-replay-build-<进程号>-XXXXXX/target/` 自己建的编译目录、跑完删（调用方设了 `CARGO_TARGET_DIR` 的照它编、不删），研究二进制从 `$REPLAY_RELEASE_DIRECTORY`（`$CARGO_TARGET_DIR/release/`）起；`scripts/experiment-apparatus/target/` 不再有复跑编出来的二进制。

```

**出处 `.claude/kb/tooling.md:224-243`（整段抄，未转述）**

```markdown
## 批量立 / 定分项时门禁会红的形态

一次立或定一批分项之后，提交前门禁里这几格会按下列形态红，写的时候照「写法」一列写：

| 门禁 | 会红的形态 | 写法 |
|---|---|---|
| doc-registries 的 first-txn-hooks 格（三份文件挂钩） | 按 `\| N \|` 行切未定项条目，最后一行把后面所有 `#### 未定项 N` 正文吞进自己那一条；正文里写「改新池新建文件的字节：是（」，判「否」的最后一行也被当成「是」 | 索引行用规范形态 `改新池新建文件的字节：是（日期，依据：…）`，正文小节写成 `**改新池新建文件的字节**：是（…）` |
| doc-decisions 的 item-ref-status 格（引用归属） | 行内裸的「已定项 k」归属给同一行最近的 `D<n>（简称）` | 新立分项之后跑 doc-decisions 的 item-ref-status 格，把翻掉的裸引用补上决策名；决策正文里引别的决策的分项，同一行里要重复写自己的决策名 |
| doc-lint | `F` 加数字这类审核编号被当成没有登记位的编号；简称后面加逗号注判「名字不符」 | 写「发现 1」「提议 2」；括注另起一对括号，写成「编号（简称）（条款已写）」，不塞进简称的括号里 |
| doc-registries 的 table-shape 格（欠账表形状） | 新的六列欠账行接在文件末尾，而那里是「已还清」那张 4 列表；欠着那张里一格开头写「已还（…）」或「销账」；欠着那张中间夹一个空行 | 接在第一张表最后一行后面：第一个五竖线的 `\|---\|` 之前、最后一行以 `\| C` 开头且竖线 ≥ 7 的行；还清了就按四列挪进已还清表，只还一部分的开头写「已还一半」「决策已定、检查仍欠」「前置已还」；表中间不留空行 |
| doc-registries 的 segment-registry 格（段序列登记表） | `--selftest` 要「mkfs 种根」那一行的段序列串在整份 `layout/01-first-txn.md` 里只出现一次，同一串在别处再写一遍就红 | 别处引用写成「4 + 1 + 1 + 1 + 4」带空格的写法 |
| 门禁 doc-decisions 的变更史三格（entry-added、shape、history-loss） | 「（其N）」、快查两行与节顶现状行 | 写法与判法在 `.claude/rules/changelog-format.md`；数与编号有没有丢由 history-loss 格判，整理变更史时暂存前自己先跑 `scripts/kb/history-consolidate.py loss-check` |
| code-source-discipline 的 format-constants 格（格式常量） | kb 标记改值后，`scripts/experiment-apparatus/**/*.rs` 与 `crates/**/*.rs` 里每一份同名 `const` 都要跟（`research/` 下只剩冻结证据，扫它但由 `.claude/format-constants-exclude` 逐份放行）；`stale=` 列的旧值串不许再出现在 kb 正文 | 别把仍在别处合法出现的串（`base = 95`）加进 `stale=` |
| clippy `shadow_unrelated` | 同一个测试函数里第二次 `let reader = …` / `let segments = …` 判红，rustfmt 重排之后用正则替换还会漏改后半段 | 每个读者 / 段 / 报告起不同的名字（`extent_reader`、`warm_up_segments`、`control_report`） |
| naming-lint | 「字母 + 数字」当单字母：`read_u48` 判红；结尾的 `a` 与只有一段的 `a` 判红（`side_a`），打头与中间的 `a` 后面还有别的段时当英语冠词、不判（`a_mount_that_stays_short`）；它扫全仓的 .rs，连 `research/prompts` 里云端腿写的模型源码也扫 | 写 `read_six_byte_unsigned`；`research/prompts` 在 `.claude/naming-lint-exclude`（证据目录，不回收），腿的模型写到别的目录照查 |
| crash-point-replay（崩溃点重放） | 崩溃枚举用例住 `crates/singlefs-verification/tests/`、登记成 `.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行、不标 `#[ignore]`（D13（验证路线） 已定项 15） | 交门禁 crash-point-replay：按条那一趟经 `--skip` 排掉登记的用例，登记的用例逐条按崩溃状态块的单元键判（块键相同的复用、缺的当场判；登记行第三列带 `separate-run=` 的用例没被点名时不跑、报本次未跑）；在跑时换 `CARGO_TARGET_DIR` 就能同时编别的测试 |
| 变异证明 | rustfmt 之后多行调用的整串匹配「没命中」 | 按「找到含关键串的行、删到以 `);` 收尾的那一行」做；改完复原并 `assert` 文件逐字节等于原文 |
| checker 新判定 | 只在对象已经过了同一关之后才判的判定永远判不红 | 每条新判定配一份坏镜像（`checker_known_bad_images.rs` 的形态），不许只看干净镜像全绿 |
| 批量定点替换 | 逐条写盘会在一处锚点不中时留下半套改动 | 用 `scripts/kb/replace-batch.py`：全部替换先在内存里核「恰好命中 1 次」，全过了才写盘 |

```

**出处 `.claude/kb/tooling.md:244-254`（整段抄，未转述）**

```markdown
## 本机 Claude Code 会话里的 `grep` 与 `find` 不是 GNU 那两个

`type grep`、`type find` 显示两个都是 shell 函数：`grep` 底下是 ugrep，`find` 底下是 bfs（报错分别以 `ugrep: error`、`bfs: error` 开头）。两处行为与 GNU 不同：

| 写法 | 结果 | 改成 |
|---|---|---|
| `grep -o` 带 `.\{0,90\}` 这种有界重复，作用在中文行上 | `exceeds complexity limits`，退出码 2，一行都不输出 | 用 python 按下标切：`line[max(0, i - 90):i + 90]` |
| `find 目录 -newermt '-40 minutes'` | `Invalid timestamp`，不认相对时间 | 写 ISO 日期：`-newermt '<YYYY-MM-DD>'` |

两种失败的样子都是「这条命令没给出结果」，接在管道或 `||` 后面时不要读成「没有命中」。

```
