**出处 `.claude/kb/experiments/202-GPU与CPU两种模式GC对照组.md:1-8`（整段抄，未转述）**

```markdown
## E202 GPU与CPU两种模式GC对照组 —— 跑前登记已写、装置未跑

**中文** · [English](202-gpu-vs-cpu-gc-comparison.en.md) · [日本語](202-GPUとCPUの2方式GC対照群.ja.md)

跑前登记 `research/prompts/e202-preregistration.md`。回答问题单 `research/prompts/m4-gpugc-compare-questions.md`：比较同一负载、同一池下 `background_compute=cpu` 与 `background_compute=gpu` 两种模式的 GC 挂钟、占用的主机 CPU 时间与省下的时间、前台 fsync 延迟分布，以及省下的 CPU 时间随盘容量的变化。

装置 `crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/main.rs`（入库装置，住在产品 GPU 的测试包 background-gpu-tests 里）。**这一页只记跑前登记与装置骨架已写，产物一次没跑。**

```

**出处 `.claude/kb/experiments/202-GPU与CPU两种模式GC对照组.md:9-16`（整段抄，未转述）**

```markdown
### 现状

- 跑前登记已写（2026-11-03），装置骨架已立，产物一次没跑。
- 第一段的几何核起过一次、没跑完、没留产物：造起步池太慢，照量到的速度过不了任务服务的四小时上限。量到的：2288MiB 档顺序写那一半约 34 分钟，交错写那一半约 60 秒做 95 步、共 18304 步；8GiB 档 2100 秒做到顺序写 18292 步、共 65536 步；三档同时造，常驻内存 2100 秒时 9.4 GiB。用户定这一次不跑，等造池提速做完再跑。
- 盘容量采用定占用读法（容量涨，活数据占单元区比例固定 50%）。
- 装置约束：GPU 模式每张卡上限 1 GiB、按卡的顺序填满；一张卡与多张卡各跑；判定逐条与 CPU 参照相同才计一组数。
- 多张卡那一臂用几张卡由 `lockstep --cards <张数>` 定（给 0 或 1、或多于服务交来的张数退 2；不给就用服务交来的全部卡）。产物里有两种记卡的行：`name=every_card_arm_cards` 记 `cards_named`、它是怎么定的（`named_by=cards_option` 或 `every_handed_over_card`）与每张卡的 PCI 地址与型号；`name=one_card_arm_card` 记点名一张卡那条臂那一张卡的 PCI 地址与型号，交替跑与并发跑都打。续跑时这几样与要续的产物里已有的同种行有一处不同，或产物里有这条臂跑出的结果行（`name=run`、`name=concurrent_run`）而没有这种行，打 `name=stopped clause=cards_differ_from_the_product_being_resumed` 退 2；一条结果行都没跑出来的那一趟不挡以后的续跑。

```

**出处 `.claude/kb/experiments/202-GPU与CPU两种模式GC对照组.md:17-23`（整段抄，未转述）**

````markdown
### 复跑

```bash
# 入库装置编译与运行归主 agent；经 task-server 调度
task-server/task submit -- cargo run --release -p singlefs-background-gpu-tests --features tests-on-a-real-gpu --bin e202_gpu_cpu_gc_comparison
```

````

**出处 `.claude/kb/experiments/202-GPU与CPU两种模式GC对照组.md:24-32`（整段抄，未转述）**

```markdown
### 口径

- 原始输出未留存：实验未跑。
- 一次整理：一次跑里 GC 线程从第一次调后台入口到最后一次调用返回为止做的全部整理。
- GC 挂钟：GC 线程上每一次后台入口调用前后读 `std::time::Instant` 之差相加。
- GC 的主机 CPU 时间：每次 GC 调用前后读 `clock_gettime(CLOCK_PROCESS_CPUTIME_ID)` 之差减去同一窗口前台线程增量。
- 省下的 CPU 时间：同一档同一次重复里 CPU 臂的进程口径减 GPU 臂的进程口径。
- 重复次数：每档每臂 5 次，报中位、最小、最大与中位绝对偏差。

```

**出处 `.claude/kb/experiments/202-GPU与CPU两种模式GC对照组.md:33-37`（整段抄，未转述）**

```markdown
### 判据与失败条款

- 判据：每一档容量上「CPU 模式减 GPU 模式」的差有数、有正负、有离散度；阳性对照下省下的 CPU 时间必须少于正常对照。
- 失败条款：若 GPU 模式省下的主机 CPU 时间在各容量档位均恒 ≤ 0，或前台 fsync 延迟劣化超限，则触发失败条款。

```

**出处 `.claude/kb/experiments/202-GPU与CPU两种模式GC对照组.md:38-43`（整段抄，未转述）**

```markdown
### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D26（后台整理与放置回收） 已定项 13 | 备料 | 2026-11-18 不受影响：这一次只把多张卡那一条的说法照登记修订第 12 条回写（两种记卡的行、续跑核卡的停机分句名），判据、臂与口径没改，产物仍一次没跑。2026-11-14 那一次改的是装置搬进测试包与起法、多张卡那一臂的张数定法，判据、臂与口径没改，产物一次没跑，仍是跑前登记已写、装置未跑 |

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

- 装置现在是 sha256 `d312b9aa71d0518976fd383b48300705f4598d2e79d4a3d59488657e684a51eb`，26562 行，`grep -c '#\[test\]'` 对它 = 240；三份起跑脚本的 `registered_apparatus_sha256` 都是它。第 2 次重跑那三趟用的是 `0bd2274383f12e88beab3f96f354148698cfcd2043263b5d5709e32e1946784d`，13187 行、118 条（改前基线是第 1 次重跑第三段用的 `eadb929d…`，10226 行、87 条）。三趟产物的 `name=apparatus` 行里，编进二进制的源码哈希与起跑脚本设给装置的登记哈希都是 `0bd22743…`（停机条款第 7 条与修订 R2-8 没触发），单点核两份与第二段同形，第一段如下（第一段第 2 行）：

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

**出处 `.claude/kb/decisions/13-验证路线.md:1-6`（整段抄，未转述）**

```markdown
## D13 验证路线 —— 已定

**中文** · [English](13-verification-route.en.md) · [日本語](13-検証ルート.ja.md)

D13（验证路线） 管本工程拿什么验正确性：三类现成办法全做——自写参照实现做差分对拍、有界穷举崩溃测试、crash refinement 形式验证（调查见 [prior-art.md](../prior-art.md) 7.4）——并且不照抄，针对本工程特有的四样（COW 每事务发新根、Merkle 自验证、整卷 AEAD 的 nonce 唯一性、没有参照实现）自己设计验证手段。它定 oracle 怎么分、自研哪两样、崩溃点重放怎么分层与抽样、崩溃状态集合怎么定义、checker 与实现共享什么。不管的：每条不变量写什么在 [invariants/README.md](../../../crates/singlefs-checker/invariants/README.md)；三样验证手段怎么落地、谁挡着谁在 [verification-build.md](../verification-build.md)；提交步骤登记在哪、结构等价类怎么分、有几个，归 D17（实现分层与第三方管道） 已定项 2。

```

**出处 `.claude/kb/decisions/13-验证路线.md:30-93`（整段抄，未转述）**

```markdown
#### 已定项 1：crash refinement 先验「一次发布 + 一次崩溃 + 一次恢复」

**实现边界**：M1–M4 在定案点名的位置都在（`crates/singlefs-replay-prefix/src/lib.rs`、E165、I-8.4）；I-8.3 的 checker 侧欠 C869（重放前缀严格连续没有会红的判定）。

**定案**：crash refinement 的第一个验证对象是恢复函数 `Recover` 的关系性质，最小封闭单元是「一次发布 + 一次崩溃 + 一次恢复」。四个里程碑的主语 `Recover` 指 `crates/singlefs-core` 里以 `verify_named_units = true` 调 `replay_journal` 的三处（`recover` 的 `Consult` 策略、可写挂载读阶段、只读挂载），三处的 `records` 都取 `scan_journal` 的输出；`Ignore` 与 `ConsultWithoutNamedVerification` 是测试与对照用的臂，写进可信部分；`crates/singlefs-implementation-experiments/src/bin/e158_root_choice_repair/main.rs` 不是产品调用方。四个里程碑依次是：

| 里程碑 | 义务 | 落点 |
|---|---|---|
| **M1 全性** | `Recover` 在 D13（验证路线） 已定项 4 枚举的**每一个**崩溃状态上有定义、终止、输出一个能过 I-7.1（根槽有效集非空） 与 I-8.3（重放前缀严格连续） 的镜像 | 脚手架。**完成判据不是「证出来了」，是「M2 能在它上面写出来」**——崩溃点重放本就全量枚举（[verification-build.md](../verification-build.md)「崩溃点重放不抽样、全量枚举」），恢复卡住当场暴露，单独证它没有边际价值。取前缀、择根、扫环三截已进 Verus，都在 `crates/singlefs-replay-prefix/src/lib.rs`：取前缀核 `replay_core`（终止与不溢出在核内证到，循环都带 `decreases`；`requires` 两条——记录表按 (实例代号, 计数器) 升序、计数器有后继——由胶水 `replay_journal` 里的断言兜），择根核 `choose_root_slot`（自证过的槽里取 (`checkpoint_txg`, 实例代号) 最大、键相同取槽序最前），扫环核 `first_copy_of_each_record_key`（同一 (实例代号, 计数器) 先到先得），择根与取前缀的合判写成推论 `corollary_recovery_replays_from_the_newest_self_verified_root`；门禁 formal-verification 判证明，登记数 56 verified（`.claude/gate.d/verus-pins.tsv`）。E165（恢复重放有界穷举与变异杀伤界） 当轻量工具，它第 3 次重跑第二段的择根合判族在内存盘上调真 `choose_root`、`scan_journal`、`replay_journal`。择根序的四个改坏变体在合法历史 K 族里只有「取第一个有效者」红，另三种（比较不看实例代号、txg 平局取低、实例代号为主）由 Verus 必红世界（`crates/singlefs-replay-prefix/verus-must-red-worlds.tsv` 择根那一节）与对抗根环 KA 罩（第 3 次重跑第三段）。核之外：择系统配置（`choose_system_configuration`）与两截胶水（择根的读槽、自证校验和、fsid、区域到盘的映射、槽的遍历次序；扫环的偏移枚举、提示、解析与 CRC、盘的遍历次序）不在 Verus 里，崩溃点重放与崩溃放量的 CPU 判器在每个崩溃状态上调 `recover` 罩它们（GPU 判器用 WGSL 另写一份 `replay_journal`，验不到 core） |
| **M2 前缀不切发布** | **任一 `jsn` 连续前缀，其重放结果等于某个整数个已发布的 checkpoint 的结果——不存在被前缀切开的发布；前缀停在一次发布中间时，那次发布整体不施加**（D16（发布语义） 已定项 4「一次发布整体施加或整体不施加」）。分两层。**句法层**（对任意自证过的记录表，无前提）：施加的记录是若干整次发布，在飞上限与提交标记逐条判过，施加的计数器从链首起逐一连续（锚点读得出时从锚点加一起；读不出时首条 `checkpoint_txg` = 所选根 + 1、序号 1）；反向链与点名单元是读盘回调的结果，写成「被施加的每条记录，回调交回了真」。**语义层**（前提：盘上每条记录出自这一版的合法写侧，即 E165（恢复重放有界穷举与变异杀伤界） 的合法历史一族；同一 (实例代号, 计数器) 在全部时间线上只对应一份内容）：「结果」是根的投影加盘，不是整条根——返回的根的 (实例代号, `checkpoint_txg`) 与新根段四个字段等于所选根那条时间线上的第 j 版（j 不早于所选根），其余字段等于所选根，卸载记号除外（由记录重建的根取「不是卸载那一串写下的」）；盘上 (所选根, j] 每一版的记录读得出、点名单元两份都验过、记录条数之和不超在飞上限 | 落点是 I-8.3（重放前缀严格连续）——它的判据补句逐字就是这一条（「前缀不得停在一次发布的中间——一次发布整体施加或整体不施加」），不要在 [invariants/03-journal.md](../../../crates/singlefs-checker/invariants/03-journal.md) 里另立一条。句法层在 Verus 核 `replay_core` 的 `ensures`（`replay_follows_the_prefix_rules`）里，必红世界登记在 `crates/singlefs-replay-prefix/verus-must-red-worlds.tsv`，其中「改判定」行只改可执行判定、不碰 spec；语义层靠 E165（恢复重放有界穷举与变异杀伤界） 的合法历史一族在界内判，Verus 不证。核之外：胶水（摊表、包回调、拼报告与新根）由 E165（恢复重放有界穷举与变异杀伤界） 直接调真 `replay_journal`、harness 用例与 `crates/mutations.tsv` 的变异行罩，拼根里投影之外的字段（卸载记号、实例表指针、分配记录树根）没有变异行、E165（恢复重放有界穷举与变异杀伤界） 的生成器把它们写成定值；两个读盘回调（反向链、点名单元闭包）由 E165（恢复重放有界穷举与变异杀伤界） 里真回调、内存盘的读数、`crates/mutations.tsv` 的变异行与只读挂载几种形状的读盘次数用例罩 |
| **M3 幂等与单调** | **幂等**：`Recover(Recover(c)) = Recover(c)`，两读都证——记录级（同一条记录施加两次与施加一次，得到的根相同，I-8.4（重放幂等） 的字面）与不动点（把重建出的根当所选根、同一张记录表、同一个读者、同一个开关再重放一次，返回的根不变；前提是所选根之上的记录不超在飞上限，且记录表满足 I-8.12（同实例记录的 txg 不减）：同一实例里 journal 记录的 txg 随计数器不减）；第二次因所选根 `checkpoint_txg` 顶格报 `CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR` 且一条没施加的，算相等。「就是 I-8.4（重放幂等）」不成立：I-8.4（重放幂等） 是记录级一读，不动点是另一条性质。**单调**（M2 的推论，不单独证）：串行提交（已定项 2）、D16（发布语义） 已定项 7 的持久顺序、所选根取根环里最新的有效根之下，同一次运行里 D13（验证路线） 已定项 4 的两个不带故障的崩溃状态 `c ⊆ c'`，`Recover(c)` 落到的版本不晚于 `Recover(c')` 落到的版本，版本按 (`checkpoint_txg`, 实例代号) 比，与 D22（单元原子性怎么合成） 已定项 7 的择新序同一个序；开并行提交或把域放宽到带故障时重开。「施加的事务集包含于」那一读法撤：它对理想恢复也为假 | Verus 核 `replay_core` 的 ensures 证两读：记录级与不动点（不动点的前件含 I-8.12（同实例记录的 txg 不减））；单调写成推论 `corollary_the_landed_version_does_not_go_back`，择根与写侧那两条前提写在它的 requires 里。E165（恢复重放有界穷举与变异杀伤界） 的对抗一族在界内判不动点，违例全在 I-8.12（同实例记录的 txg 不减） 不成立的记录表上（第 3 次重跑第一段）。checker 那一半（I-8.4（重放幂等））已实现，取在崩溃注入的二次崩溃上判、不进池级 checker 的清单：`crates/singlefs-verification/src/crash_injection.rs` 的 `REPLAY_IDEMPOTENCE_INVARIANT` 与 `judge_the_remount_after_a_second_crash`。O2（独立解析器 + checker） 判不了幂等：I-4.5（已停用·需两镜像） 退役理由逐字「幂等要比对两个镜像，单镜像判不了」 |
| **M4 不少施加** | 前提与对象同 M2 的语义层。`Recover` 返回的根恰是 j\*：从所选根起每一版都完整在盘上（记录任一份自证过、点名单元两份都验过）、记录条数之和不超在飞上限的最远一版。不加这一条，M1–M3 全部证出来也排除不了「已确认的完整发布被恢复丢掉」：一条都不施加的恢复过 M1–M3 的全部字面。「不少施加」按「多出违例」读：变体多出违例才算被它挡住 | E165（恢复重放有界穷举与变异杀伤界） 的最远一版判据在界内判；Verus 的必红世界里「第一条水位之上的记录就停、一条都不施加」那一行红；M4 自己不单独证 |

**挂载透明**（独立的第四条性质，不是 M3 的第三读）：在崩溃镜像上只读恢复一次得 R1；在同一份镜像上做一个动作——只挂载（取号、写行、暖机、挂载续做与挂载自己推的抬 F），或挂载加一次发布——之后再只读恢复一次得 R2，逐字段比 R1 与 R2。比的那一版：只挂载比恢复落到的那一版，挂载加一次发布比挂载后那次发布之前的那一版。挂载途中三个相位的二次崩溃镜像 img2 上也判：C(recover(img2)) = C(v1)（v1 是 R1 落到的那一版，C 取 POSIX 内容与存在性），内容与字段两半都判；字段一半里，落回 v1 那一版自己的各字段与 R1 相同，落到这次挂载写的某一版的照逐字段表判，扫描报告不判，抬 F 第一个写起 F 不判、在它之前（含暖机）F 照「挂载没推抬 F」那一格判；抬 F 第一个写的位置按挂载交回的抬 F 那几串（准入推的与续做里推的）在挂载写表末尾的位置取，取不出（`NotLocatedInTheMountWrites`）时这一次挂载的二次崩溃整段 F 记不判。逐字段表（R2 对 R1）：

| 字段 | 判法 |
|---|---|
| POSIX 内容与存在性 | 必须相同 |
| 池身份与系统配置的不可变配置（本盘设备号除外） | 必须相同；不可变档 37 行除本盘设备号一律按原字节比 |
| 系统配置的节点大小 | 必须相同 |
| 系统配置的运行配置 36 字节（T_time、T_dirty、三水位） | R2 等于内置默认值（没有挂载选项通道，`to_slot` 总写默认） |
| 卸载记号 | R2 恒为「不是卸载写的」，R1 的记号不进比较 |
| F 与 F 生效值 | 挂载没推抬 F：R2 的 F 生效值与 R2 根里的 F 都等于 R1 的 F 生效值；推了（准入推的与续做里为空间推的都算）：R2 的 F 生效值等于挂载交回的最后一串抬到的 F |
| 实例代号 | max(各盘两槽自证系统配置的实例代号, 根环全部可读根的实例代号) + 1，两个量都从崩溃镜像读 |
| 实例表行 | 照 D18（块里携带什么信息） 已定项 11 恢复写的行 |
| `checkpoint_txg` | 只判下界：≥ max(根环全部根的 txg, 环里全部自证记录的 `checkpoint_txg`) + 1（D23（journal 的角色与格式） 已定项 14 的下界，允许跳号） |
| 根槽位置 | 不另判，由 `checkpoint_txg` 定 |
| 树 ID 水位 | 挂载不发第一个文件时相同 |
| 树表里 inode 树、extent 树、墓碑、谱系、deleted_inodes 的根指针（后三样是树表条目，种类 9、10、11，不是根记录字段） | 相同；挂载续做过时，按续做之后那一版判：等于挂载交回的续做各步里 txg 不大于 R2 那一版的最近一步的各根指针；树表单元自己的落点不比 |
| 树表种类 6、7、8、12 的根，诞生 txg、头 ID、树 ID、种类 | 必须相同（种类 6、7、8、12 的根恒全零）；诞生 txg 跨根另有 I-9.14（树表条目的诞生 txg 跨根不变），头 ID 另有 I-9.27（树表条目的头 ID 取值），单镜像判 |
| 实例表与树表的单元指针 | 不比字节，与「树表单元自己的落点不比」同 |
| 扫描报告 | 施加 0 条、水位之上 0 条 |
| 回收门槛 t 与钉槽 p（根槽偏移 428..437） | 不比：每次带单元的发布按这个进程的根环与钉现算、零单元发布照抄上一版，挂载做的发布会改它们（与 F、卸载记号同类）；单镜像性质，归池级 checker 的 I-7.18（根记录的回收门槛等于现算的门槛） 与 I-7.19（钉着的槽不住本实例写的根） 判 |
| 根槽偏移 437..466（算法、nonce、MAC） | 必须为 0；单镜像性质，归池级 checker 的 I-2.4（头校验和覆盖范围）判，挂载透明不比 |
| magic、flags 其余位、自证校验和、整槽校验和 | 不比（结构性，读槽时判） |
| 系统配置槽世代号 | 不比，交 I-7.16（系统配置槽世代号与槽号的关系） 在 R2 的镜像上判 |
| 系统配置的 journal_tail | R1 ≤ R2 ≤ R2 环里自证记录的最大计数器，挂载做完取等 |
| 系统配置的 journal 实例代号 | 挂载做完等于 R2 根的实例代号；二次崩溃时取 R1 的实例代号或新实例号两者之一 |
| 分配记录（含根记录的 `mapping_root`、`allocation_record_tree_root` 与树表种类 3、4 的根） | 不比整表：POSIX 内容相同，且池级 checker 对 R2 的镜像判绿；`mapping_root`：R2 全零 ⇔ R1 全零；`allocation_record_tree_root`：R1 带树 ⇒ R2 全零，R1 树表 0 条 ⇒ （R2 非零 ⇔ 新实例号 ≥ 2） |

系统配置的运行量（槽世代号、journal_tail、journal 实例代号）落回 v1 时照它们自己那一行判，不按 v1 相同判。

**射程**：refinement 的边际价值只在崩溃点重放枚举取不到的那几维——载荷字节、记录条数 / 段数 / txg 数（含环回绕）、上一条时间线的残留、槽数与设备数；崩溃点那一维崩溃点重放已全量枚举。「校验和过了」在模型里是一个布尔标志，**不许读成 refinement 验了完整性**——与已定项 14 里 D4（校验和位置） 那一行「不许读成『验了 Merkle』」同形，也与 D13（验证路线） 已定项 4 对 CRC32C 碰撞概率的「知情接受」是同一笔账。D13（验证路线） 已定项 4 是 refinement 左边的 `Crash`（模型层的枚举域），不是右边的 `Spec`，拿它当 spec 会让 refinement 恒真。I-6.5（nonce 水位单调） 的形式化撤下；D18（块里携带什么信息） 已定项 14 定了、且 nonce 分配规则（单调计数器 + 预支水位、水位在使用前持久）被立成一条分项之后，这一格重开——重开时它唯一的真优势是 nonce 对 POSIX 不可见，它的 spec 与 O1（内存模型对拍（HashMap 理想文件系统）） 的 model 结构上不可能同源，不受冲突 2（D13（验证路线） 已定项 13） 影响。

M1–M4 的前提与域：语义层只挂「同一 (实例代号, 计数器) 在全部时间线上只对应一份内容」，由 D23（journal 的角色与格式） 已定项 14「这一版的失败处置」的原样重发担保，根槽写失败推进 `checkpoint_txg` 再发那条路落地时重核；前提域取 E165（恢复重放有界穷举与变异杀伤界） 的合法历史一族（合法写侧写出的历史的任一落盘子集，较新的根槽可以读不出）。M2 的「其余字段等于所选根」里，由记录重建的根的卸载记号是 `NotWrittenByTheUnmountSequence`（D22（单元原子性怎么合成） 已定项 7「其余根写 0」），与所选根的记号可以不同，所以写「卸载记号除外」。M3 的单调推论只在串行提交、不带故障的崩溃状态里成立。M4 连同择根才排除「已确认的」完整发布被恢复丢掉，前提要全：不带故障的崩溃状态（已定项 4 的枚举域）、所选根按 D22（单元原子性怎么合成） 已定项 7 的择根取、链不跨实例边界（D23（journal 的角色与格式） 已定项 14 注 1）、新实例本实例写成的根覆盖两块盘之前不让任何 fsync 返回（D16（发布语义） 已定项 8 的暖机）；带故障的域里不排除——在飞上限会截掉记录完整的发布，一块盘整块掉了时点名单元少一份的那一版不施加（C519（丢一整块盘时恢复丢掉刚确认的那一版））。

挂载透明的射程：只管不带故障、可写挂载做成的崩溃状态。可写挂载被拒的状态不判透明，判拒绝本身合不合法：拒绝合法 ⇔ 拒因在 D23（journal 的角色与格式） 已定项 14 那张单子上，且照那一条的判据在崩溃镜像上自己复算成立（「读阶段判完之后系统配置又读不出」这一拒因带补写例外：D23（journal 的角色与格式） 已定项 14 的单子上这一拒只在某块盘一份自证过的系统配置都没有、又过不了 D22（单元原子性怎么合成） 已定项 8 的补写闸时成立；判官只复算「一份都没有」那一半，补写闸五件不复算，补写闸放过的盘面挂上了不拒，归崩溃点重放的观察者判（D13（验证路线） 已定项 6）；判槽自证读池级 checker 的逐盘数，不用核心层的解析，实现在实现批），且盘上逐字节不变，且只读读出在允许集合里；拒因不在单子上的照崩溃注入第二截的写法报红。带故障的状态不在射程：取号那一刻有一槽读不出与 N-配置报成同一个拒因，接进带读故障的档之前要分开（C664（挂载拒绝合法复算把两种拒因报成一个））。逐字段表里树 ID 水位、两棵树的根指针、`checkpoint_txg`、扫描报告四格是推的（三方判决取辩方的式子，没量），F 那一格「推了」的一半没量；根记录的 `mapping_root`、`allocation_record_tree_root`，墓碑、谱系、deleted_inodes 三根指针，算法、nonce、MAC，系统配置的槽世代号与 journal_tail 各归哪一类，照 C655（挂载透明逐字段表有字段没归类）、C687（从 mkfs 水位建树那一版落点判不了的几格） 判决 Y1、Y2 写在逐字段表里。

**依据**：
- E42（一事务几条记录）：为了让「被切开 ⇒ 非法」成为恒等式而钉死了载荷，「被切开却碰巧合法」这一类它结构上看不见——M2 把载荷符号化补的正是这一格。
- 三方：材料 `_d13-item1-background.md`，三条腿的产出 `d13-item1-opus-forward-output.md`、`d13-item1-sonnet-reverse-output.md`、`d13-item1-local-attack-v2-output.md`；三腿一致把原候选排最后，两条云端腿各自收敛到「前缀判定与提交原子性」这道接缝，主 agent 按封闭性把择新与发布原子性收进同一个单元（判决与撤下原候选的三条门槛原文在变更史）。
- E165（恢复重放有界穷举与变异杀伤界）：在界内对真 `replay_journal` 判 M2 的语义层、M4 与 M3 两读，并量变异杀伤界；用户 2026-10-01 定它当轻量工具；C 域合法崩溃状态数按两种切法分写（甲切法是 D16（发布语义） 已定项 7 的屏障位置）；对抗一族里不动点的违例全在 I-8.12（同实例记录的 txg 不减） 不成立的记录表上，谓词成立那一边 0 违例（第 3 次重跑第一段）；择根合判族调真 `choose_root`、`scan_journal`、`replay_journal`，界内、合法、不带故障时已确认的完整发布不丢（第 3 次重跑第二段）；择根序的四个改坏变体在合法历史 K、K9、KW 三族界内只有取第一个有效者红，另三种在对抗根环 KA 上红（第 3 次重跑第三段）。
- 三方判决（M1–M4 的措辞与落点）：`research/prompts/m3-formal-r1-main-verification.md`、`research/prompts/m3-formal-r2-main-verification.md` 第二节、`research/prompts/m3-formal-r3-main-verification.md` 第二、三节；M3 单调推论的四步推导在 `m3-formal-r2-opus-output.md`「3.2 推的：串行提交下，j1 与 j2 推得出落到的版本单调」一节。
- 三方判决（M3 不动点前提补 I-8.12（同实例记录的 txg 不减） 与落点现状）：`research/prompts/m3-formal-code-r1-main-verification.md` 第二节不动点前件那一格（打中条款、不打代码）与第三节第 5 条；装置的前提比 Verus 的前件宽、违例全落在该谓词不成立一边的对账在调度记录 `records/2026-10-01-里程碑三形式化接入.md` 第 10 批「10-5 交回」那一条。
- 用户定案（2026-10-01 弹窗，都照推荐）：卸载记号除外、M4 射程句的前提写全、单调的版本序取 (`checkpoint_txg`, 实例代号)、txg 顶格取「报 no_successor 且一条没施加算相等」；写回等 K10 判完一起做，原话在变更史。
- 挂载透明：无实验：判法在崩溃注入里（`crates/singlefs-verification/src/mount_transparency.rs`，崩溃注入调它），不是实验；三轮判决里攻方的数在副本装置上量，代码轮第二轮攻方在入库代码上量的第一次崩溃 147 806 态、二次崩溃 234 773 态 0 红主 agent 没复跑。
- 三方判决（挂载透明）：`research/prompts/m3-formal-r4-main-verification.md` 第二节「挂载后再恢复：模掉什么」「比的对象与射程」两行与第四节第 2、3 条（独立的第四条、比的那一版、射程）；`research/prompts/m3-formal-r5-main-verification.md` 第二节 U5、U6 两行与第四节第 5、6 条（逐字段表、二次崩溃比 v1、拒绝合法的条件）；`research/prompts/m3-formal-r6-main-verification.md` 第二节 U5、U6 两行（txg 只判下界、拒的判据在崩溃镜像上自己复算）与「逐字段表没碰的字段」一行；`research/prompts/m3-formal-code-r2-main-verification.md` 第二节 K2 各行（实现照判、站住；暖机与挂载推的抬 F 拆段）。
- 用户定案（挂载透明加二次崩溃，2026-10-02 弹窗，原话在变更史）：`research/prompts/m3-formal-r4-main-verification.md`「用户定案（2026-10-02）」二次崩溃那一行。
- 择根序变体那一句：调度记录 `records/2026-10-01-里程碑三形式化接入.md` 第 12 批 E165（恢复重放有界穷举与变异杀伤界） 第 3 次重跑第三段判 Q2 交回那一条（主 agent 判补这一句）。
- 挂载续做算挂载的一部分、F 的界按挂载交回的抬 F 在写表末尾的位置取：用户定案 2026-10-03「续做算挂载的一部分」（原话在变更史）；代码轮第三轮判决 `research/prompts/m3-formal-code-r3-main-verification.md` 第三节第 3 条（界按写流位置取，找不到不 panic）；实现 `crates/singlefs-verification/src/mount_transparency.rs`。
- 拒绝合法那一句里「读阶段判完之后系统配置又读不出」这一拒因的复算带补写例外、判槽自证读池级 checker 的逐盘数：三方判决 `research/prompts/m3-close-g4-rewrite-r3-main-verification.md` 第二节 Y1、Y3 两行与第三节交用户表「条文」「G4 拒绝那一臂的判槽自证」两行；用户定案 2026-10-22「改读池级 checker」，零轮，原话在变更史。

**欠**：C77（重放起点未定义）：在择新与前缀判定那道接缝上；C649（Verus 判不到的两样回调结果）：已还清（M2 语义层的反向链与点名单元仍只能写成后置条件，Verus 一侧不判，两个读盘回调改由 `crates/mutations.tsv` 的变异行罩着，见欠账表已还清表）；C650（择根与重放没有合判）：M4 的射程句要择根与重放合起来才成立；C653（扫环胶水内存随盘数涨）；C654（必红世界期望条数随求解环境漂）；C655（挂载透明逐字段表有字段没归类）：已还清（逐字段表照判决 Y1、Y2 补齐，见欠账表已还清表）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:94-107`（整段抄，未转述）**

```markdown
#### 已定项 2：先串行提交

**定案**：事务层先承诺串行提交——`.claude/rules/fs-design.md`「判据写不出来时，先只做一条，而且**先做慢而正确的那条**」落到实处，串行是本工程明令的默认值，不是在两条路里挑了一条。越过交点再开并行，而不是在交点之前就承诺并行。对将来并行提交的一条前置约束：记录按事务号顺序追加，事务号 N 的 fsync 在返回前必须等 < N 的事务全部已追加 ∨ 已作废 ∨ 已确定实例切换或转只读（作废是提交管道的内存状态，对并发后继可见）；串行提交下这条规则空转。

**射程**：串行的天花板在一次 checkpoint 约 1.1 万条条目（单线程合并耗时追上一次设备同步的交点，算式在 C58（串行提交撞天花板没人报警））：低于交点设备同步是瓶颈、串行不吃亏，高于交点单线程合并变成天花板，没有安全余量。已定项 2 不决定记录头的字段清单、不改变新池新建文件写出的字节，它决定性能天花板与 crash refinement 的范围。并行提交仍然会动崩溃语义：「前缀」是全序上的概念，并行下一个前缀可能同时含有完整的事务 B 与残缺的事务 A；记录头已带事务边界字段（D23（journal 的角色与格式） 已定项 7：事务号 8 + 提交标记 1）。

**依据**：
- E17（write buffer 合并上限）：单线程 sort-merge 的上限是交点的一半；按叶分区的无锁并行是越过交点之后的逃生口，但不许默认它够。
- E44（序号位宽的本机实测与代价）：本机 fsync 率是交点的另一半。
- E42（一事务几条记录）：串行下一个事务跨多条记录同样会被前缀切开——决定记录头字段清单的是「一个事务恰不恰好产生一条 journal 记录」，不是串行 / 并行。
- 用户定案，记在变更史；将来并行提交那条前置约束来自 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P1。没走三方论证：这是确认一条已经写明的默认值，真正要走三方的是何时改成并行，那一步没做。

**欠**：C58（串行提交撞天花板没人报警）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:108-140`（整段抄，未转述）**

```markdown
#### 已定项 3：RefFS 做到极致，自己处理持久化与崩溃语义

**定案**：不做只管功能的内存模型，做一个自己处理持久化与崩溃语义的参照实现（RefFS），把 RefFS 从「储备候选」升为**要做的东西**。「做到极致」取语义级：RefFS 只在内存里按 POSIX 操作走，自己定义每次 fsync 之后什么算持久、崩溃之后允许看到哪几种 POSIX 状态，没有盘上格式；带自己盘上格式、自己写盘与恢复的那一档（工程量接近再写一个文件系统）不做。RefFS 与实现不许共用崩溃点枚举的那段代码：做到极致之后「三 oracle 合谋测试」（已定项 6）防的那个共同误解，最危险的落点是「RefFS 与实现是否共享同一个崩溃语义的错误理解」。

RefFS 的来处与边界：

| 项 | 定案 |
|---|---|
| 来处 | 只照 kb 条文另写：不拷 O1（内存模型对拍（HashMap 理想文件系统）） 的代码，不读录制流，不调切段、状态计划与实现的崩溃点枚举 |
| 放哪 | 单独一个包 `crates/singlefs-reference-filesystem`：依赖闭包（dev-dependencies 也算）⊆ {它自己, `singlefs-format`}，能只用 std 就只用 std；源码不引 `singlefs_core`、`singlefs_checker`、`singlefs_verification`（经改名的别名同样算）；`singlefs-core` 与 `singlefs-checker` 的依赖闭包里没有它。三条由门禁 checker-independence-and-sync 的 checker-implementation-disjoint 格判 |
| 输入白名单 | 都由调用方交入，只收五样：每个操作的结局（成、在任何写之前拒、写了之后报错、原样重发）；崩溃落在第几个操作里（hi）与它的归属规则；回退目标 m；每次恢复读出的内容；显式发布点 |
| 第一版的操作 | 第一个文件、覆盖写、发布调用、fsync、回退 |
| 调用点 | 崩溃注入与故障注入的调用点上，RefFS 的判定与 O1（内存模型对拍（HashMap 理想文件系统）） 崩溃判官的全部四条判据合取，任一红即红 |

崩溃语义（串行提交、不带故障的崩溃状态）：

1. **fsync**：fsync 返回成功之后，这次 fsync 等到的那次发布带走的全部操作必须在（串行下就是调用开始之前已取号成功的全部操作，按单元计）；按状态下标读，即允许集合的下界 lo 抬到那次发布的末尾。fsync 报错时不抬 lo，报错那次发布带的操作可在可不在；之后的操作接在下一次恢复读出的状态上判。
2. **丢法：按发布边界的前缀**。崩溃落在第 hi 个操作里时，恢复允许读出的是 {S_j : j = lo，或 j 恰是某次显式发布的末尾；lo ≤ j ≤ hi}，S_j 是上一次恢复之后前 j 个操作施加出来的状态，S_0 是上一次恢复读出的内容。丢掉任意子集的那种丢法在串行提交下出局（与 D16（发布语义） 已定项 12 不相容），开并行提交时重开。
3. **抬 lo**：返回成功的发布调用、fsync、回退都抬 lo（D16（发布语义） 已定项 7：回退与 fsync 同一个返回条件）；回退算一次已确认发布，它的下标算发布末尾。
4. **回退**：回退是 RefFS 的一个操作，回退之后的内容等于整段历史里第 m 次已确认发布之后的内容；m 由调用方从实现报的回退根翻出来交入，RefFS 不判目标选得对不对。m 的编号覆盖每一个写根的入口（发布调用、回退、写行与暖机、抬 F、卸载），映到写它时的现行版本，现行版本含每次恢复读出、重新起头的那一份。
5. **崩溃时刻的归属**：崩溃落在第几个操作里，按「切开当前段的那道屏障由哪一步发出」归属，不按第一个被扣下的写；这条只给 RefFS 用，O1（内存模型对拍（HashMap 理想文件系统）） 的崩溃判官不动。
6. **恢复之后**：每次恢复之后，RefFS 先判读出的状态在允许集合里，再以它为起点接着走。

**射程**：只定做到什么程度与 RefFS 的形态，不定排期；不许读成「先做 RefFS 再做实现」——RefFS 是验证手段不是里程碑（`.claude/rules/format-evolution.md`「从写开始，不从读开始」）。显式发布点只取历史里的发布调用与 fsync：自动触发的攒批落地之前 RefFS 只接显式发布点，自动触发的发布不在 RefFS 射程，归已定项 1 的 M2 与记录核对器；崩溃恢复抛弃最新根的那一次，RefFS 以读出内容重新起头、不判那一步。带故障的崩溃状态（说谎的设备、丢整块盘 C519（丢一整块盘时恢复丢掉刚确认的那一版））不适用。独立性：RefFS 不读录制流、不调切段与状态计划，但它的 (lo, hi) 与 O1（内存模型对拍（HashMap 理想文件系统）） 崩溃判官盘级下界那一条判据的入参输入同源，记在 C191（同源性计数器只盖了一对） 那一类同源账上。D17（实现分层与第三方管道） 已定项 4 据此把「入口」的可执行定义交给 RefFS 的操作面，fsync 的持久性承诺由 RefFS 的崩溃语义定义；O3（独立规约执行器） 仍是崩溃点集合的独立执行器，职责不变。RefFS 的第一版在 `crates/singlefs-reference-filesystem`，崩溃注入两截与故障注入重开的调用点已接：验证包经翻译层 `crates/singlefs-verification/src/reference_filesystem_translation.rs` 依赖它，调用点判 RefFS 与 O1（内存模型对拍（HashMap 理想文件系统）） 崩溃判官两票、任一红即红。

**依据**：
- 无实验：做到什么程度、RefFS 怎么来与收什么是验证投入与边界的取舍；三轮判决里攻方的数都是副本装置上量的，入库装置上还没重做（`research/prompts/m3-formal-r5-main-verification.md` 第五节第 3 条），这里不引。
- 用户定案，原话在变更史：「做到极致」（原定案）；做到语义级（2026-10-02，岔路单 `m3-formal-7-forks.md` 里 RefFS 做到哪一档那一行，是对原定案换了读法）；按发布边界的前缀、调用点合取、另起、单独一个包（2026-10-02，`research/prompts/m3-formal-r4-main-verification.md`「用户定案」）；合取收严成全四条、回退目标作输入（2026-10-02，`research/prompts/m3-formal-r5-main-verification.md`「用户定案」）；回退抬 lo、崩溃时刻按屏障归属、m 覆盖每一个写根的入口、白名单加原样重发（2026-10-02，`research/prompts/m3-formal-r6-main-verification.md`「用户定案」）。
- 三方判决：`research/prompts/m3-formal-r4-main-verification.md`（第一轮：fsync、丢法、独立性三格与六条岔路）、`research/prompts/m3-formal-r5-main-verification.md`（第二轮：fsync 合成句、按发布边界补三句、合取收严、另起与单独一个包、任意子集出局限串行）、`research/prompts/m3-formal-r6-main-verification.md`（第三轮最终轮：回退与 lo、射程两句、白名单、fsync 按状态下标读）。
- D17（实现分层与第三方管道） 已定项 4：O1（内存模型对拍（HashMap 理想文件系统））明写「不管崩溃」，定义不了 fsync 的持久性承诺，而第一个可运行目标恰恰是「正确提交一个事务」——做到极致的 RefFS 补的正是这一半。

**欠**：C447（规约执行器没接进崩溃点重放与 RefFS 比对）；C191（同源性计数器只盖了一对）；C658（崩溃注入第三截放过丢掉的一步复活）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:141-156`（整段抄，未转述）**

```markdown
#### 已定项 4：崩溃点重放只枚举整写子集

**定案**：崩溃点重放要枚举的崩溃状态集合定义为：录下来的写请求流**按设备**切成段——一次写只被它自己那块盘上之后的屏障（或它那块盘上的 FUA 写）排在之后的写前面；当前段里每块有写的盘都被自己的屏障或 FUA 放行时这一段才关上，有一块盘还没放行，这道屏障（或 FUA）不关段、前后的写同段。一个崩溃状态是「前若干段全部持久，当前段每次写各取它的几态、任意组合，之后各段全部没持久」；枚举的单位是**一次整写**。带 FUA 的写只放行它自己那块盘：它与那块盘上前面没被屏障隔开的写同段，别的盘上的写不因它持久。一次写取两态（持久 / 没持久）；**原地覆写**（不是单元写、长于一个扇区、罩住的范围里原来有东西的写，系统配置槽写与覆盖旧记录的 journal 写属于这一类）多取第三态「新旧都读不出」，一律枚举；第三态的镜像罩那次写的整个范围，新旧不同的那一截前一半是新字节、后一半是旧字节（按字节撕，不按扇区）；其余写的撕裂态一律并进「这次写没持久」。「没持久」的位置上，镜像里放的是**崩溃前那个位置的旧字节**，不是零，而且这些镜像（第三态的在内）要真的生成出来喂给 checker 与记录核对器，不许只当计数。镜像双写（D2（RAID 条带策略） 已定项 6 的 w ≥ 2、D22（单元原子性怎么合成） 已定项 8 的 journal 镜像）按**设备**各算一次写，harness 的状态数按自己的写流另算闭式。**同一块盘上连着的屏障并成一道**：录制流里一道屏障与这块盘上一道屏障之间没有任何一次写（别的盘的屏障不算写）时，这一道不记（`crates/singlefs-verification/src/lib.rs` 的 `push`），独立模型照同一条规矩记；中间没有写，这一段不产生崩溃状态，合并与否不改变崩溃状态集合，只改变操作数与种类串。

**射程**：它定义的是模型层的枚举域。真实设备的 FLUSH / FUA 是否如宣称生效归 C6（块层语义假设写错），要在 QEMU 里用真设备验。按设备切段多出实际走不到的状态（一块盘没放行时，别的盘上已放行的写也留在段里取任意组合），不漏走得到的。撕裂分两类：原地覆写之外的写（单元写、不长于一个扇区的写、罩住的范围里原来没东西的写）撕裂态与「没持久」的差别只有 CRC32C 的碰撞概率（32 位，D19（块指针的结构与宽度预算） 已定项 2、D23（journal 的角色与格式） 已定项 11），**知情接受**，撕裂粒度不进模型，与 D20（承重面：单元的原子性与自包含）「有父指针的单元不依赖任何宽度」一致；原地覆写撕裂时那一处的旧内容也没了，「没持久」却留着完整的旧内容，两者不是同一个状态，所以单独枚举。崩溃点重放的扇区取 512 字节；两条崩溃点重放流上取第三态的只有系统配置槽写。已知边角：第三态的旧字节取「这次写之前的写全落了」那一版，同段更早、与它重叠的写没落的组合里旧的那一半多带那次写的字节（两条崩溃点重放流上这种组合一次都没有）；根槽写长于一个扇区又罩住旧内容时第一版不支持第三态（harness 断言；崩溃点重放的池都按 512 字节物理块建，走不到）。设备把一次写撕成恰好撞上校验和碰撞的形态不在模型内。D13（验证路线） 已定项 5 与它正交：一个定枚举域、一个定 crate 边界。

**依据**：
- E77（发布的持久顺序）：段模型在四种屏障摆法下的状态数逐臂等于闭式——这套枚举域数得对。
- E142（新池新建文件的干跑）：第十七次跑第一段的几何敏感性取样点 G7（阳性对照同一条写流、只把 FUA 当边界）实测 4097 个状态、2046 个违例——没有屏障时单元、journal 记录与根槽合成一段 `[12]`；装置的切法与 `crates/singlefs-verification/src/segments.rs:91`「FUA 写关掉自己所在的那一段（FUA 不替前面的普通写做持久，所以它们同段、任意子集）」一致，「FUA 自成一段」那一读法预言的 2050 / 0 对不上——FUA 那一句的字面照实现的切法写。
- 三方对抗与判决：[verification-build.md](../verification-build.md)「三方对抗（2026-09-03）」判决第 3 条——每一类单元都有整单元校验（有父指针的靠父指针里的校验和，D4（校验和位置） 已定项 1、D16（发布语义） 已定项 7、D22（单元原子性怎么合成） 已定项 19；自证单元靠整单元校验和加被实际检查的代号，D22（单元原子性怎么合成） 已定项 21；journal 记录靠头校验和与头内的载荷校验和，D23（journal 的角色与格式） 已定项 13），撕裂态与「没持久」在校验和眼里是同一件事。
- 用户定案（按 E77（发布的持久顺序） 与三方对抗定；FUA 写算段边界那一句是 C313（FUA 写算不算崩溃段的边界） 收口时的用户定案，2026-09-26 用户定照实现改成「关掉它所在的那一段，与前面没被屏障隔开的写同段」），记在变更史。
- 按设备切段（有一块盘没放行就不关段）、FUA 只放行它那块盘、原地覆写多取「新旧都读不出」：用户定案（代码审阅第 1、3、4 条；2026-09-27 弹窗选「A 合并，按设备记屏障」「照补，全量也跑」与「原地覆写补第三态」），第三态按字节撕是主 agent 同日定的、被攻过零轮，记在变更史；实现与判别力（只吞盘 1 一道屏障，段尾 `26,2,1,2` → `26,5`，崩溃点重放小流里记录核对器报出来）在 `m2-rev-b3a2-implementer-report.md` 第一、三节。
- 同一块盘上连着的屏障并成一道：用户 2026-09-28 定「并，跟今天的录制器」（E142（新池新建文件的干跑） 第十九次跑撞上登记的停机 S19-crates：独立模型两道都记、录制器并掉第二道，暖机与整条流四格的操作数与种类串不同、崩溃状态数相同，报告 `e142-r19-runner3-report.md` 结论段，登记 `e142-r19-prereg.md` 的 R40），原话在变更史。

**欠**：C6（块层语义假设写错）：拿虚机的设备侧日志重建崩溃状态、崩溃测试必须变红的那一半。

```

**出处 `.claude/kb/decisions/13-验证路线.md:157-175`（整段抄，未转述）**

```markdown
#### 已定项 5：只共享一份从 kb 生成的常量

**实现边界**：共享的常量模块 `crates/singlefs-format` 是手写的，没有从 kb 生成的生成器；宽度由 checker 自己探测那一半已有。

**定案**：checker 与实现之间只共享一样东西：一份由 kb 的字段表生成的常量模块，生成器从 kb 读、两边都只消费、任何人不许手改（`.claude/rules/machine-first.md`：重复要生成，不能手抄）。**其余一律不共享**：地址空间的 newtype 各自声明、格式解析各写一份、校验和各用一份独立实现、遍历与记账代码交集为空（C12（增量语义共用） 由门禁 checker-independence-and-sync 的 checker-implementation-disjoint 格定的判据，形态是 crate 粒度的依赖闭包交集、不是符号级，「格式解析与常量除外」那条例外就是这一项在用）。

- **判定宽度不许从别处拿**：I-7.5（根槽按判定宽度对齐） 与 I-8.2（记录头不跨原子单元） 的判据是挂载时探测的 `physical_block_size`，它不在镜像里。checker 自己探一次，再与系统配置记录的 mkfs 时取值比对（D22（单元原子性怎么合成） 已定项 2「mkfs 按池内最大值划、挂载时逐设备复算比对」的 checker 侧形态）。**探不到时报「声明值，未探测」，不许当成探到的**——文件当设备的镜像属于这一类，崩溃点重放的绝大多数镜像都是文件，所以这条不是边角。异构池里判定宽度是一个集合不是一个数（C17（设备几何当全局参数）），checker 要复算 mkfs 那步取最大值的聚合，不然健康的异构池上会假红。
- **生成器拒绝发射没有 kb 落点的常量**，也只发射标量值：它一旦开始发射「按字段表算出来的偏移函数」，那就是 D13（验证路线） 明令不许共享的格式解析，而不再是常量；判据是发射物里有没有分支与算术。
- **CRC32C 各写一份的前提是参数钉死**（多项式、初值、输入输出反转、异或输出）：参数不钉死，两份独立实现会在健康镜像上给出不同结果，那是假红而不是独立性。

**射程**：管 checker 与实现的 crate 边界；管不到 kb 里就写错的值——共享一份生成常量与两边各抄一份都抓不住那一类，它要一条正交的门禁（C94（登记的格式常量与后来的定案对不上））。实现与它的差距：`crates/singlefs-format` 是手写的，不是生成的（值由门禁 code-source-discipline 的 format-constants 格按 kb 里的 `format-const` 标记绑住，只绑标了的那些），生成器没有；`crates/singlefs-checker` 只依赖 `singlefs-format`，CRC-32C 另写了一份按位的；checker 取的是系统配置声明的 `physical_block_size` 与 `io_min`（`crates/singlefs-checker/src/image.rs` 的 `geometry_of`），「探到 / 声明」两种来源的类型是 `crates/singlefs-checker/src/lib.rs` 的 `DecisionWidth`：声明值那一种（`DeclaredOnly`）已接到 I-7.5（根槽按判定宽度对齐） 的判定上（已实现，`decision_width_declared_by_the_chosen_system_configurations` 取各盘所选系统配置声明的 max(`physical_block_size`, `io_min`) 里最大的），探到那一种（`Probed`）只有类型定义（`crates/singlefs-checker/src/lib.rs`），全仓没有构造它的代码、没有接到任何判定上；I-8.2（记录头不跨原子单元） 未实现。系统配置里 mkfs 时的 `physical_block_size` 字段在 D22（单元原子性怎么合成） 已定项 9 的几何段（4 字节）。

**依据**：
- 无实验：共享边界是保验证独立性的政策，没有可量的量；它的判别力靠两边各写一份的检查逐条红给人看。
- 三方对抗：材料 `_checker-sharing-vehicle-background.md`，腿的产出 `research/prompts/checker-sharing-vehicle-*`，腿况在 [verification-build.md](../verification-build.md)——正推腿从 D13（验证路线）、C12（增量语义共用） 推出「常量在例外内、其余禁止」；替代反推腿与本地腿各自构造出同一条：kb 里写错的值共享与各抄都抓不住。
- 用户定案，记在变更史。

**欠**：C17（设备几何当全局参数）；C387（常量模块手写，不是从 kb 生成）；C388（checker 的判定宽度不自己探测）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:176-206`（整段抄，未转述）**

```markdown
#### 已定项 6：三个互相独立的 oracle

**定案**：三个 oracle 各判一样东西，独立性各有来处：

<!-- doc-lint:registry name-col=2 -->

| # | oracle | 判什么 | 独立性来自哪里 | 验不到什么 |
|---|---|---|---|---|
| O1 | 内存模型对拍（HashMap 理想文件系统） | POSIX 可见语义：内容、大小、目录结构；无崩溃处的内容期望归它，崩溃之后允许看到的集合归 RefFS（已定项 3） | 不碰盘、不解析磁盘格式 | 磁盘结构是否自洽；崩溃语义；**两份实现对同一条 POSIX 语义的共同误解** |
| O2 | 独立解析器 + checker | 磁盘字节是否满足 I-1（块自描述）~I-6（加密不变量） | **checker 不得调用被测 crate 的任何反序列化/遍历代码**，必须重新实现一份解析，直接读裸设备 | 只判结构自洽：一个块被覆盖成另一份**合法**内容，Merkle 链照样自洽、记账照样对 |
| O3 | 独立规约执行器 | 崩溃点截断后的每个磁盘状态是否落在规约允许的集合里 | **用另一种语言写**（不用 Rust），只照 decisions 与 invariants 的文字重新推导；第一版用 Python 单写（E170（O3规约执行器第一版）） | 规约本身可能写错——**必须配阳性对照才有意义**；被投影的六样的取值与六样之外的根记录字段、实现自己的恢复与读者代码、系统配置槽的见证值（journal_tail）、一次发布内各写的先后 |

**独立性要被验证，不许自称**：设计一个「三 oracle 合谋测试」——拿一个精心构造的坏实现
（例如 nonce 永远为 0）去过三关，若三者全部放行，说明存在共同盲区，回头补 oracle。
功能正确性不只押在这三个 oracle 上：O1（内存模型对拍（HashMap 理想文件系统）） 那一行已把崩溃之后允许看到的集合交给 RefFS（已定项 3），恢复函数的 M2–M4 落在 Verus 与 E165（恢复重放有界穷举与变异杀伤界）（已定项 1），需要第二个输入的核对归记录核对器（已定项 7）；三个 oracle 之间的共同盲区靠「三 oracle 合谋测试」找。

**射程**：三个 oracle 各自验不到什么写在表的最后一列；O2（独立解析器 + checker） 的定义域归已定项 7，需要第二个输入的核对归记录核对器。**实现今天的样子**：O2（独立解析器 + checker） 在 `crates/singlefs-checker`（只依赖格式常量 crate）；O1（内存模型对拍（HashMap 理想文件系统）） 的形态是 `crates/singlefs-verification/src/model.rs` 的理想模型（只住内存、只用 `singlefs_format` 的常量、不建崩溃），门禁 model-differential-and-scenarios 拿随机历史逐步对拍；O3（独立规约执行器） 是只照 kb 文字写的 Python 执行器，现行那一份是 E171（三oracle合谋测试） 第 1 次重跑用的 `scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates_journal_header_487/`（E170（O3规约执行器第一版） 第 2 次重跑从第 1 次重跑冻结的 `scripts/experiment-apparatus/e170_second_run_o3_spec_executor/` 拷出 `scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates/` 加谓词 W、T，再跟上 487 字节 journal 记录头；第 1 次重跑冻结的那一份与第一次那一份 `scripts/experiment-apparatus/e170_o3_spec_executor/` 留档）：严格跑法判得出（第一次导出的批 1、批 2 上 0 态撞「规约欠一句」），与实现逐态一致（两边全放行，不当它判得对的证据；「择到的根不是自证根里 txg 最大的那条」「③ 收到不少于 2 槽」两条路径批里 0 态，C661（规约执行器批里两条路径没有态）），坏状态表照改正的期望重判有判别力（阳性对照与盲测都判对），还没接进崩溃点重放；判一个崩溃态的做法：照规约的恢复规则（择系统配置、择根、前缀按 D23（journal 的角色与格式） 已定项 14 的五条口径）从镜像自己恢复出一版，恢复不出（整池拒或没有自证根）判不在；恢复出的这一版的投影（实例代号、checkpoint_txg、树表单元指针、中央映射树根指针、树 ID 水位、F）要等于版本表里某一版的投影，且这一版的下标在 [已确认, 已开始] 之间。版本表是基镜像上自己恢复出的现行那一版，加录制流里每一次根槽写按流序各一版，每版的投影取实现写进根槽的字节；已确认取崩溃态里落了的最晚一条写之前已返回的最晚一版（只从落了的写推），已开始取落了的写所属的最晚一版。系统配置槽只用来择槽、取根环与 journal 环的几何、判池级字段越界整池拒。所以恢复规则那一半是照 kb 文字另写的，允许集合那一半取自实现自己写出的根；RefFS（已定项 3）在 `crates/singlefs-reference-filesystem`，崩溃注入与故障注入的调用点已接；「三 oracle 合谋测试」是 E171（三oracle合谋测试），读数取自一棵冻住的树（快照摘要见实验页，与现行 `crates/` 不是同一份）：O1（内存模型对拍（HashMap 理想文件系统）） 一列是模型对拍加模型的崩溃判官，不接 RefFS，与表里 O1（内存模型对拍（HashMap 理想文件系统）） 那一行不同（崩溃之后的集合那一半 E171（三oracle合谋测试） 没用 RefFS）；O3（独立规约执行器） 一列用 E170（O3规约执行器第一版） 第 1 次重跑冻结的那一份；手造坏镜像那一族只喂 O2（独立解析器 + checker）、O3（独立规约执行器），期望出自 E170（O3规约执行器第一版） 的设计员，Q2 照登记字面判「有」（4 张全是 P8），按登记 F3 判成 E170（O3规约执行器第一版） 推期望推错、这四张不是坏镜像，真坏镜像里 O2（独立解析器 + checker）、O3（独立规约执行器） 两条全放行的 0 张；三条的源头是否独立（表第 4 列）E171（三oracle合谋测试） 只在 Q2 量过 O2（独立解析器 + checker）、O3（独立规约执行器） 一对，那一对的补丁与期望同出 E170（O3规约执行器第一版） 的改法代码，不当独立读数，O1（内存模型对拍（HashMap 理想文件系统）） 不在 Q2 里；没有量到的不写成「互相独立」被验证过。

**三条 oracle 都不判的两样**（第一样由崩溃注入与崩溃点重放两侧判，崩溃点重放那一侧的观察者在实现批）：一是无读故障的崩溃态上，D23（journal 的角色与格式） 已定项 14 可写挂载那条见证判据为真（系统配置槽里的见证值超前于盘上读得出的记录）：崩溃注入第二截判拒绝合不合法时只在崩溃镜像上复算这条判据，为真就判合法（挂载透明的射程只管不带故障的状态，已定项 1）；要有判官问「这个崩溃态没注入读故障，判据凭什么为真」：崩溃注入第二、三截在没注入读故障的崩溃态上复算这条判据，为真即判拒得不合法（`crates/singlefs-verification/src/crash_injection.rs` 的 `RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults`，C670（无读故障的崩溃态上可写挂载见证判据为真没有判官） 已还清，见欠账表已还清表）。「无读故障的崩溃态上每块盘至少一份自证过的系统配置」这一性质由崩溃点重放 mkfs 返回之后各条流上的观察者判：观察者按流登记，记录 mkfs 的流不挂，只记相对基镜像新出现的、一份自证过的系统配置都没有的盘，撕裂域在内，判槽自证读池级 checker 的逐盘数（`crates/singlefs-checker/src/image.rs` 的 `system_configuration_redundancy_report`），另有一条登记完整性用例判含系统配置槽写段的流都挂了它（实现在实现批）。随机历史里补写闸（D22（单元原子性怎么合成） 已定项 8）不过的那一形，由崩溃注入第二、三截的 G4 拒绝那一臂判；补写闸过了的那一形，随机历史里没人判，正确实现留不下这种盘面（两次抽样都没打中）。mkfs 返回之前的崩溃态不在观察者与 G4 拒绝那一臂的射程：mkfs 逐盘两槽在同一个循环里写、之后才逐盘屏障，崩溃注入的候选段与 E171（三oracle合谋测试） 的基镜像都取在 mkfs 之后；G4 拒绝那一臂与 C670（无读故障的崩溃态上可写挂载见证判据为真没有判官） 那条还有一处不同形：择不到系统配置时那条的复算交 `None`、G4 按真，这时读阶段本身已拒、到不了这一格。二是一次发布内各写的先后：O2（独立解析器 + checker） 的 I-7.12（系统配置 F 不低于同盘根上的 F） 判抬 F 那一串先写系统配置这一步；变异点名的用例只判 E171（三oracle合谋测试） 第 270 行那一种（发布返回之后每个崩溃点的见证值），第 242 行点名的用例判的是失败路径冻结；`layout/01-first-txn.md` 第八节把根槽 FUA 与系统配置槽轮换登记成两段，只是登记在册；其余写序不在三条之内。

**依据**：
- E170（O3规约执行器第一版）：O3（独立规约执行器） 照「另一种语言、只照 kb 文字重新推导、配阳性对照」写得出第一版，判得动单个崩溃状态、一态够快、每趟都能跑；第 1 次重跑（照 2026-10-03 的 kb 改过一次的执行器）严格跑法判得出、与实现在取样的批态上一致、阳性对照有判别力（第一次跑判不出与判别力不足那两笔留在实验页）。
- G4（无读故障的崩溃态上每块盘至少一份自证过的系统配置）：代码轮判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 j2-b 立、第 29 批实现，射程与 mkfs 那一段见 `research/prompts/m3-formal-code-b29-r1-main-verification.md` 第二节 L3 与第三节第 3 条。
- 两槽全废盘面改由崩溃点重放 mkfs 返回之后各条流上的观察者判、G4 拒绝那一臂留着：三方判决 `research/prompts/m3-close-g4-rewrite-r3-main-verification.md` 第二节 Y2 各行与第三节（前两轮 `research/prompts/m3-close-g4-rewrite-r1-main-verification.md`、`research/prompts/m3-close-g4-rewrite-r2-main-verification.md`）；观察者的接法（按流登记、相对基镜像、登记完整性用例）是用户定案 2026-10-22「按流登记+相对基镜像」，零轮，原话在变更史。数在三方腿的模型与副本原型上量，没立实验号。
- 三个 oracle 的分工与各自独立性的来处是验证体系的结构设计；独立性靠「三 oracle 合谋测试」观测（E171（三oracle合谋测试））。
- E171（三oracle合谋测试）：在一棵冻住的树上，坏实现表里有三条 oracle 全部放行的行、有只靠一条才抓到的行，O3（独立规约执行器） 在坏实现表上一行没红；手造的伪造根、伪造续接镜像里有 O2（独立解析器 + checker） 放行而 O3（独立规约执行器） 判不在的（读数与限定在实验页 Q1、Q2、Q3 三行）。
- 三方论证：D13（验证路线） 新立那一轮（本地 LLM / Sonnet / Opus 各一条腿，立场不重复），材料没有留存，经过在变更史。
- O1（内存模型对拍（HashMap 理想文件系统）） 与 RefFS 的分工句：三方判决 `research/prompts/m3-formal-r5-main-verification.md` 第二节另起与单独一个包那一格（两腿同向）。

**欠**：C445（模型对拍的取样点不随写路径增长）；C447（规约执行器没接进崩溃点重放与 RefFS 比对）；C407（三 oracle 合谋测试的判官没进门禁）：E171（三oracle合谋测试） 已量过一次，没有一格门禁读它的产物；C672（交回值或内存状态错而写流不变的一类）；C679（卸载记号没进模型对拍）

```

**出处 `.claude/kb/decisions/13-验证路线.md:207-225`（整段抄，未转述）**

```markdown
#### 已定项 7：O2（独立解析器 + checker） 判的是**一个**镜像

**定案**：

> **定案：O2（独立解析器 + checker） 的定义域是单个镜像**——一个一元谓词 `P(镜像) → 成立/不成立`，
> 只判集合成员，永不判 identity。**任何需要第二个输入（另一个镜像、或一段记录流）的检查都不属于它。**

**需要第二个输入的核对归记录核对器**，入参 `(崩溃前镜像, 记录流, 崩溃后镜像, 持久集合)`——持久集合是枚举器给出的这个崩溃状态里哪些写落了盘（崩溃点重放的段内整写子集）；只看盘上字节分不开「写没落、而原地内容恰好相同」与「写落了」，判复用与缺席要拿持久集合判；它与自研第 1 项「Merkle 根链差分」（已定项 8）同属二元检查。⚠️ **故意不给它编号**——O2（独立解析器 + checker） 那场混乱就是编号当称呼造成的，再加一个 O4 只会给下一次漂移多一个落脚点。⚠️ **「崩溃后镜像」是两份，不是一份**：harness 的顺序是先跑实现自己的恢复、再跑记录核对器，而恢复本身会改盘（实例切换写行、重发在飞 checkpoint，D23（journal 的角色与格式） 已定项 14）⇒ 择根与前缀判定看**崩溃态镜像**（「记录流」就是它环里的 journal 记录），比对的对象是**实现恢复后的镜像**。

**射程**：指针层目标态下核对一条记录只需格式解析 + 校验和 + 根环择新；逻辑意图记录不点名任何盘上对象，核对方要自行实现树语义 + 记账 + 分配合法性——D23（journal 的角色与格式） 否掉逻辑意图日志的第二条理由据此改写成第一条的一条封堵，写在 D23（journal 的角色与格式） 已定项 5。记录核对器的代码在 `crates/singlefs-verification/src/crash.rs`，接在崩溃点重放的每个崩溃状态上（门禁 crash-point-replay）。记录核对器的判定因此不只是镜像的函数：崩溃后镜像逐字节相同、持久集合不同的两个状态，它可以判得不同，按镜像去重的提速对它不适用（O2（独立解析器 + checker） 与 oracle 照旧只看镜像）。 崩溃点重放不覆盖可写挂载：崩溃点重放在每个崩溃状态上只跑只读恢复（看 journal 与不看各一遍），取号、写行、暖机在崩溃点重放的任何崩溃状态上都不跑。可写挂载由崩溃注入覆盖（`crates/singlefs-verification/src/crash_injection.rs`）：每个抽到的崩溃状态上，只读恢复与判定之后在同一份崩溃后镜像上起一次可写挂载、再发一次布、跑池级 checker；挂载途中再崩一次，取号、写行、暖机三段各摆一个二次崩溃状态，每个上只读恢复、问模型、池级 checker、记录核对器：第三截交给记录核对器的写表从「挂载那一段到当前段为止的前缀」换成整条历史录制流 + 整条挂载流 + 挂载之后那次发布，持久集合逐段对应（历史那一段取第一次崩溃的持久集合，挂载那一段按二次崩溃取子集，之后那次发布全没落）。「比对的对象是实现恢复后的镜像」那一半：记录核对器改收两份镜像，崩溃态镜像判根在而记录在不在（择根与前缀），恢复后的镜像判单元在不在、读落到那一版的实例表；第二截接的恢复后镜像是挂载与那次发布之后的池，恢复自称的那一版取挂载与那次发布写出的最新那条根，挂载一条根都没写出时退回第一次崩溃落到的那一版。

**依据**：
- 无实验：**依据是类型，不是成本**：「逐字节比对」是二元关系 `R(镜像₁, 镜像₂)`，**没有任何投入能把一元谓词变成二元关系**；I-4.5（已停用·需两镜像） 因同一条门槛退役。
- 「崩溃后镜像」是两份：随 D16（发布语义） 已定项 4 收窄问法那一轮三方论证（材料 `_d16-items134-background.md`）由用户定案，记在变更史。
- 持久集合那一样入参：用户定案，原话在变更史；判别子观测不到的实据在 `research/prompts/m2-layer0-scale-r3-main-verification.md` 第二节按扇区豁免那一行。
- 崩溃点重放不覆盖可写挂载、可写挂载与二次崩溃由崩溃注入覆盖：用户定案（代码审阅第 2 条，2026-09-27 弹窗选「崩溃注入层补可写挂载」），记在变更史；崩溃注入那一侧的实现在 `m2-rev-b3b-implementer-report.md`。

**欠**：无。

```

**出处 `.claude/kb/decisions/13-验证路线.md:226-249`（整段抄，未转述）**

```markdown
#### 已定项 8：自研的两样：Merkle 根链差分与代际增量 scrub

**定案**：现成工具没有、本工程自研的两样：

**1. Merkle 根链差分 —— COW + 每事务发新根独有的杠杆。**
历史根在被回收前物理存在过，构成一条可回放的链。对相邻两代根递归比较「子树指针 + 校验和/MAC」，
**指针没变的地方直接跳过不下钻**，复杂度只正比于两代之间实际改动的部分。
它能检出常规遍历式 checker 检不出的一类错误：**同步篡改后自洽的原地写**——
某次写绕过 COW、原地改了块内容并同步更新了校验和，单点看它是自洽的（I-2.1（校验和与内容匹配） 全绿），
而根链差分会发现矛盾：**没走 COW 就不该有新分配，父指针理应没变，校验和却变了**。
⚠️ **必须配阴性对照**：正常 COW 提交后跑同一检查必须全绿，否则它会把每次正常更新都当成篡改。

**2. 代际增量 scrub。** 靠 I-1.2（块头写序已发布） 与 D5（快照 / 空间记账机制） 的 birth txg
维护一个「上次 scrub 扫到的 generation 水位」，只遍历 `birth generation > 水位` 的块——
老子树的校验和链没变就必然还自洽。**它让「每次提交都全量验证本次改动」在挂钟上可行，不必抽样。**

**射程**：被搬过的副本 `birth` 不变、会永远落在水位之下，代际增量 scrub 看不见它们。`crates/` 里没有根链差分（`grep -rn 根链 crates` 零命中），也没有代际增量 scrub（`grep -rn scrub crates` 零命中）。

**依据**：
- 无实验：两样都靠 COW 每事务发新根与 birth txg 的结构性质成立，都还没有实现，没有可量的量。
- 三方论证：D13（验证路线） 新立那一轮（本地 LLM / Sonnet / Opus 各一条腿，立场不重复），材料没有留存，经过在变更史。

**欠**：C389（Merkle 根链差分零代码）；C194（代际增量 scrub 看不见被搬过的副本）；C408（代际增量 scrub 零代码）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:250-279`（整段抄，未转述）**

```markdown
#### 已定项 9：崩溃点重放要分三层，否则挂钟必炸

**定案**：

| 层 | 范围 | 触发 |
|---|---|---|
| 冒烟层 | 固定种子、固定脚本的负载与登记的崩溃枚举用例（登记行第三列带 `separate-run=` 的用例没被点名时不跑、报本次未跑），不按写请求数限规模（规模见射程），精确全量 | 每次提交（门禁不指定范围那一趟） |
| 常规层 | 中等规模负载（写请求数上千），精确全量，枚举不完靠精确剪枝，一时跑不了的节点标 ignore，覆盖报告与门禁逐个列「本次未跑」 | 每次提交 |
| 大负载层 | 常规层 负载 + 大规模负载 + 多设备档位交叉，N≥5 轮，精确全量 | 用户要求时 / 发布前 |

**剪枝加 ignore 之后敢说什么**：敢说「开着的节点在全部崩溃状态上通过」；
**不敢说**「ignore 的节点通过」。

**提交前 = 不指定范围那一趟全绿（必要条件）；红了照门禁通则修好再提交。**
**指定范围（`--item`、`--check` 这类参数点名）可以缩崩溃域：设了展开深度，放量分项与崩溃枚举用例都只展开到它（用例取自己的展开与设的深度里较小的那个）；缩过域的那一趟结果行写出实际展开，不算门禁通过。**
「不指定范围那一趟」指两样：门禁 crash-point-replay 不带选条参数的那一趟，与崩溃放量（`scripts/crash/crash-amplification.sh`）不带选条参数的那一趟。
**不指定范围那一趟必须优先保证「失败可复现」而不是「覆盖率最大」**——修一个它判出的 bug 的挂钟 ≈ `log2(N) × 跑一趟的挂钟`，一个不可复现的失败价值接近 0，
甚至为负，它训练人忽略红灯。

**射程**：只有冒烟层（门禁 crash-point-replay）；冒烟层只有一条路：门禁不指定范围那一趟照单元键判全部崩溃枚举用例（登记行第三列带 `separate-run=` 的用例没被点名时不跑、报本次未跑），崩溃放量入口 `scripts/crash/crash-amplification.sh` 不指定范围的那一趟（每次提交都跑）判全部放量分项，判过的块照块键复用、没判过的当场判，带断点续跑；登记在 `.claude/gate.d/stage-inputs.tsv` 的崩溃枚举用例已经长到并行线一那条流 12230590578 个状态，不再是「写请求数两位数」的冒烟体量，挂钟长不是收窄它的理由。常规层、大负载层在门禁里没有阶段。每个崩溃状态怎么定义归已定项 4；节点怎么精确剪枝、ignore 节点什么时候开归已定项 10、11。

**依据**：
- 无实验：分层是门禁挂钟的设计约束；常规层、大负载层在门禁里的负载还没有定，E184（常规层与大负载层的代价计数） 第一段已量候选负载的规模与态数区间重叠，只备料；第二到四段（作废面、冷吞吐、两种落法的固定开销）未跑。
- 三方论证：D13（验证路线） 新立那一轮（本地 LLM / Sonnet / Opus 各一条腿，立场不重复），材料没有留存，经过在变更史。
- 用户定案：「我们不是抽样 我们是剪枝 加 初期ignore」（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 第三节「用户定案：不抽样，剪枝加初期 ignore」那一行）、「枚举可以设置 ignore 标记 然后慢慢开启」（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 第三节「用户定：ignore 标记慢慢开」那一行）、「1 该枚举该设计的设计完 设计全」「2 枚举不完 正是剪枝的用处」（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 第三节里材料员被门禁 doc-process-records 的 implementation-premise 格挡下、用户定『设计全、枚举不完靠剪枝、粒度合适』的那一行）；第一轮判决 `research/prompts/m3-prune-gpu-r1-main-verification.md`「回看决策」一节：已定项 9 推翻。
- 用户定案（2026-11-05）：不分轻重快慢、只有一个门禁，门禁分指定范围与不指定范围两种跑法，原话在变更史与 `records/2026-11-05-删轻型并入重型.md` 第 108 行；崩溃域红了挡提交，是主 agent 照它判的（同一份记录第 122 行 ①、第 128 行审定点）。
- 用户定案（2026-11-05，弹窗选「允许缩域」）：指定范围可缩崩溃域，缩过域那一趟不算门禁通过（主 agent 照它改的审定点），原话在变更史与 `records/2026-11-05-删轻型并入重型.md` 第 133 行。

**欠**：C409（常规层、大负载层在门禁里没有阶段）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:280-309`（整段抄，未转述）**

```markdown
#### 已定项 10：真正必须精确覆盖的 N 不是「布局套数」，是「提交协议结构的等价类数」

**定案**：常规层 精确覆盖先缩问题，再谈剪枝。崩溃点的定义域是**事务层状态机**，而 `.claude/rules/fs-design.md` 有两句顶层规则：
「一个事务层，所有结构共用」与「唯一不许多样化的是事务层」。所以布局能改的只有两类：

| 差异 | 改变的是 | 判定 |
|---|---|---|
| 节点大小、条带宽度 | 一步写多少字节、某步重复几次 | **参数**，不新增步骤种类 |
| 旋转 vs SSD | seek 代价、IO 调度 | 落在设备层，**不落在事务层协议里** |
| **zoned（ZNS）** | **能不能原地覆盖根指针** | **结构性**：只能追加、根位置漂移，且多出 zone finish/reset |

**落地手段是一个封闭枚举 + 穷尽 match**：`crates/singlefs-core/src/transaction.rs` 的 `CommitStep`，步骤种类的登记位是 [layout/01-first-txn.md](../layout/01-first-txn.md) 八的段序列登记表。
任何布局要引入新的结构性步骤**必须改这个共享定义 ⇒ 编译器强制所有引用处补分支 ⇒
这个 diff 门禁 grep 得到**，可以自动打标「本次引入了新的结构等价类，需为它新开一轮独立大负载层 重放」，
**不靠人记得申报**（按 diff 算门禁范围欠在 C8（门禁范围判不出来））。再加 newtype 隔离（`RotationalPtr` / `ZonedPtr`），
把 D12（目标介质） 已定项 2 里「不许共用」那一桶（记账事实的判定路径）变成编译期约束。
⚠️ **它有第二个职责（D17（实现分层与第三方管道） 加的）：它是第三方管道的接入点。**
协议归本工程所有，一条管道要引入新的结构性提交步骤必须往本仓发这个 diff。

**射程**：旋转与 SSD 在提交协议层面同构，由 D12（目标介质） 那一轮三方论证承担；zoned 那一行只说明它至少多出一个等价类。等价关系按录制流的段序列同构、有几个，归 D17（实现分层与第三方管道） 已定项 2。按种类串认节点够不够，第一轮判决打中（挂起，待第二、三轮攻）：`research/prompts/m3-prune-gpu-r1-main-verification.md`「逐格判决」一节。
⚠️ **它钉住的只是「有哪些步骤」，钉不住「每一步的失败原子性宽度」**——
持久内存与对象存储改变的是每一步的失败原子性宽度，枚举接不住（缺口与补法见 D17（实现分层与第三方管道））。
`RotationalPtr` / `ZonedPtr` 这层 newtype 隔离 `crates/` 里没有（`grep -rn "RotationalPtr\|ZonedPtr" crates` 零命中）。

**依据**：
- 无实验：N 是什么由 `.claude/rules/fs-design.md` 那两句顶层规则推出，没有可量的量；等价关系怎么判、有几个的依据在 D17（实现分层与第三方管道） 已定项 2。
- 三方论证：常规层 抽样那一轮（「三方论证 + 实测验算」），材料没有留存，经过在变更史。

**欠**：C8（门禁范围判不出来）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:310-344`（整段抄，未转述）**

```markdown
#### 已定项 11：ignore 的节点什么时候开：否决按欠账轮转的账本，留下三条

**定案**：

**「按欠账轮转的账本」调度：否决。** 格子 = `(布局, 子阶段桶)`、按欠账降序取队首、声称任一格子最多 `K_max = ⌈N×M/C⌉` 次提交必被覆盖的那个调度被否决：它保证不了的恰恰是它要防的那一类——慢的、罕见的分支（实测验算与另外四条否决理由在变更史）。

**留下的三条**：

1. **方向对**：要一个**确定性的、能被门禁判红的覆盖记录**，而不是概率论证——
   概率保证在长期维护里等于没有保证。

<!-- doc-lint:registry name-col=2 -->

| 符号 | 简称 | 含义 |
|---|---|---|
| T1 | 常规层 挂钟预算 | 常规层 本地挂钟预算，是输入不是输出 |

2. **`T1`（常规层 挂钟预算） 是输入，不是输出**（`.claude/rules/fs-design.md`：门禁运行时长本身是设计约束）。
   正确顺序是「先声明预算 → 用它约束一次提交能开几个 ignore 节点」，不是「量出耗时 → 推出能开多少」。
   **这一条不需要任何测量，现在就该定。**
3. **判别力必须与覆盖分开检查**：每个 ignore 节点被打开的那次提交里，额外对一份「已知坏的镜像」跑它自己的 checker，
   不报错即判红——这是 D13（验证路线）已有的阳性对照纪律的直接复用。见 C13（checker 判定失效）。

门禁里没有「常规层 崩溃点重放」这个阶段，量不出 `T1`（常规层 挂钟预算） / `C` / 每格耗时。
⚠️ **因此 `C`、`K_max`、`T1`（常规层 挂钟预算） 这三个数一个都不许现在写进 kb。**

**射程**：只管 ignore 的节点什么时候开；冒烟层全量枚举不需要 `T1`（常规层 挂钟预算）。第 2 条的「现在就该定」与末句「一个都不许现在写进 kb」说反话，记在 [verification-build.md](../verification-build.md)「待定案」第 6 问，没判。节点登记表带状态字段（开着 / ignore / 等实现），另加三条——每个 ignore 行点名一个会变的对象、那个对象变绿而节点仍 ignore 时判红、覆盖报告列 ignore 的状态数与占比：这三条被攻过零轮（`research/prompts/m3-prune-gpu-r1-main-verification.md`「逐格判决」一节），不写成定案，等第二、三轮再攻。

**依据**：
- 无实验：否决账本靠的是一次调度规则的模拟验算（200 轮、取后半程稳态最大值、严格前缀贪心，三行数在变更史），没有登记成实验；留下的三条是门禁设计原则，没有可量的量。
- 三方论证：常规层 抽样那一轮（「三方论证 + 实测验算」），材料没有留存，经过在变更史。
- 用户定案：「枚举可以设置 ignore 标记 然后慢慢开启」（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 第三节「用户定：ignore 标记慢慢开」那一行）；第一轮判决 `research/prompts/m3-prune-gpu-r1-main-verification.md`「回看决策」一节：已定项 11 推翻，「逐格判决」一节给节点登记表带状态字段的形态与三条待攻的补充条件。

**欠**：C8（门禁范围判不出来）；C13（checker 判定失效）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:345-362`（整段抄，未转述）**

```markdown
#### 已定项 12：冲突 1：`checker 即规范` 与 `校验的两条路径不许共享工具`

**定案**：**冲突 1：`checker 即规范` 与 `校验的两条路径不许共享工具` 直接打架。**
在用的不变量（条数的权威登记位在 [invariants/README.md](../../../crates/singlefs-checker/invariants/README.md) 文件头）若既是 checker 源码又是形式验证的 spec，就成了单点。
**可行的折中，而且本工程已经付过这笔钱**：checker 做两个方向——**遍历式**（从系统配置走下去）
与**扫描式**（全盘扫，靠「元数据块必须自描述」的块头重建），两者对同一条不变量独立判断，
共享的只有格式解析。两个方向往同一个判定累加器（`crates/singlefs-checker/src/image.rs` 的 `Judgements::judge`）报判定，这条报告通道不算判定共享（用户 2026-10-26 定）。

**射程**：管 checker 内部两个方向之间共享什么；checker 与实现之间共享什么归已定项 5。池级 checker 两个方向都有（`crates/singlefs-checker/src/walk.rs` 文件头）。扫描式方向连同 `walk.rs` 里扫单元头、按已发布谓词滤、喂 I-1.8（归并后版本全序） 与扫描重建的那几个函数一起算；已发布谓词在扫描式那一侧另写一份（`PublishedPredicateOfTheScanDirection`），实例表照旧由遍历式读出、当扫描重建的输入传进来（C113（扫描重建时多版单元的现行版本判定无输入） 的定案）；`lib.rs` 里的 `check_unit` 与 `data_unit_bytes` 算格式解析，登记为允许共享（`.claude/gate.d/checker-direction-entries.tsv`）。

**依据**：
- 无实验：这是「checker 即规范」与校验路径独立之间的取舍，没有可量的量。
- 三方论证：D13（验证路线） 新立那一轮（本地 LLM / Sonnet / Opus 各一条腿，立场不重复），材料没有留存，经过在变更史。
- 用户定案 2026-10-26（两次弹窗：扫描式方向算哪些函数、工装另建调用图；越界的六项怎么处置），原话在变更史。
- 门禁 checker-independence-and-sync 的 checker-direction-overlap 格（`scripts/gate/checker-direction-overlap.py`）：两个方向各从登记的入口收函数单元闭包，交集里格式解析、常量与报告通道之外的函数单元判红，真树上越界 0 项。

**欠**：C410（checker 两个方向共享什么没有检查）：已还清（门禁 checker-independence-and-sync 的 checker-direction-overlap 格，见欠账表已还清表）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:363-383`（整段抄，未转述）**

```markdown
#### 已定项 13：冲突 2：crash refinement 的 spec 与模型对拍的 model

**定案**：**冲突 2：crash refinement 的 spec 与模型对拍的 model 是同一个东西的两种写法。**
合并成一份 → 省一半工作量，但**直接违反独立性要求**；各写一份 → 名义独立，
但通常是同一个人同一周写的。缓解（只降低相关性、不消除）：让两份**在表述层面就不同**——
spec 写成「对镜像的状态谓词」，model 写成「op → 状态转移」，不一致时至少能互相定位。
**同源性要被观测**：统计「spec 与 model 不一致」的事件数及每次判定哪一方错；
若累计 ≥ 5 次不一致而判定「spec 错」的次数为 0，说明两者实际同源，应把两条腿合并、预算移给穷举。
判定分三格：spec 错 / model 错 / 条款欠一句（两边各照条款的一种读法，条款没写谁对）。一次不一致先在前提成立的域里重判，还在的才记；「条款欠一句」的事件先记账，不计入「累计 ≥ 5 次」，也不计入「spec 错」，条款补了那一句之后按补的句子再判一次、归到前两格。
这里的 model 指 O1（内存模型对拍（HashMap 理想文件系统））。O3（独立规约执行器） 能对同一（历史、崩溃点）交出判定行之前（第一版 Python 执行器 E170（O3规约执行器第一版） 已有，没接进崩溃点重放，C447（规约执行器没接进崩溃点重放与 RefFS 比对）），O3（独立规约执行器） × RefFS 这一对登记为空格、不计事件；O1（内存模型对拍（HashMap 理想文件系统）） 的崩溃判官不当替身；没立的对报「未立」，不报绿、不计入任何一格。

**射程**：缓解只降低相关性、不消除。已定项 1 撤下的那条原候选不受这条影响，理由在已定项 1 的射程。crash refinement 的代码是 `crates/singlefs-replay-prefix/src/lib.rs` 里的 Verus 核与它的 spec（取前缀、择根、扫环三截与择根和取前缀的合判推论，覆盖已定项 1 的 M1、M2 句法层与 M3 的两读和单调推论；M2 语义层与 M4 靠 E165（恢复重放有界穷举与变异杀伤界） 在界内判）；spec 不能对具体状态求值，「spec 与 model 不一致」的事件从哪来没定，计数器没有代码（C191（同源性计数器只盖了一对））。

**依据**：
- 无实验：同源性要靠将来 spec 与 model 的不一致事件计数观测，spec 与计数器都还没有，没有可量的量。
- 三方论证：D13（验证路线） 新立那一轮（本地 LLM / Sonnet / Opus 各一条腿，立场不重复），材料没有留存，经过在变更史。
- 第三格「条款欠一句」：三方判决 `research/prompts/m3-formal-r1-main-verification.md` 第二节「F2 同源性」一行（攻方打中，推的）、`research/prompts/m3-formal-r2-main-verification.md` 第二节第 9 行、`research/prompts/m3-formal-r3-main-verification.md` 第二节同源性计数器一行，三轮站住；无实验：计数器仍没有事件来源。
- 「model 指 O1（内存模型对拍（HashMap 理想文件系统））」与没立的对登记为空格：三方判决 `research/prompts/m3-formal-r5-main-verification.md` 第二节同源性计数器那一格（两腿同向的部分）；`research/prompts/m3-formal-r4-main-verification.md`「用户定案」里替身对那一行（不算替身对）。

**欠**：C191（同源性计数器只盖了一对）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:384-402`（整段抄，未转述）**

```markdown
#### 已定项 14：与其他决策的连锁

**定案**：

| 决策 | 连锁点 |
|---|---|
| D4（校验和位置） | 根链差分把 Merkle 从「要付的代价」翻转成验证杠杆；但形式验证里哈希是 uninterpreted function，**碰撞抗性完全在验证范围之外，不许读成「验了 Merkle」** |
| D8（核心索引结构） | 日志结构节点、write buffer、key cache 三层都会绑架符号执行（循环上界、滞后窗口、缓存一致性），使「Yggdrasil 一分钟内」的口径在本工程不成立 |
| D9（加密） | AEAD 在 SMT 里不可解，只能建模成 uninterpreted function，**「nonce 重用泄漏明文」这个后果验不出来**，只剩「nonce 不重复」这条纯组合性质——而这条纯组合性质恰是这条腿最值钱的产出 |
| D11（索引节点要不要留消息缓冲区） | 已定项 1 取「留」；D8（核心索引结构） 已定项 1 取幂等完整值之后 checker 的 I-3.1（已分配统计对得上） 不必解析消息语义（D11（索引节点要不要留消息缓冲区） 据此判它的代价 1 在本工程不成立），冲突 1 不因此变成「无解」 |

**射程**：连锁的是形式验证与 checker 碰到这几条决策时的边界，不改那几条决策的条款；根链差分与冲突 1 本身归已定项 8、12。

**依据**：
- 无实验：连锁是从那几条决策的已定条款推出来的边界，没有可量的量。
- 三方论证：D13（验证路线） 新立那一轮（本地 LLM / Sonnet / Opus 各一条腿，立场不重复），材料没有留存，经过在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/13-验证路线.md:403-414`（整段抄，未转述）**

```markdown
#### 已定项 15：一个验证包 singlefs-verification、一个门禁、两种跑法

**定案**：`crates/` 下的验证代码只住一个包：验证包 `crates/singlefs-verification`（库名 `singlefs_verification`），与池级 checker 一共两个名字。验证包里是单元测试与集成测试加上它们用的脚手架库（录制器、内存池、理想模型、随机历史、故障注入设备、场景），崩溃态枚举引擎与记录核对器、崩溃枚举用例经判定库的两段与它的计数、发现日志与工具链身份、一份长期判定库（块键只由内容定，几类测试各自要哪些块只当索引）与两段协议、崩溃注入与坏盘输入两场战役、真设备一侧的设备日志比对、实验与真设备装置二进制，以及它们的全部用例；一条用例一个场景，测试文件按测什么起名、不按里程碑起名。门禁只有一个，两种跑法：不指定范围（不带选条参数，门禁照单元键自己选要判的、判全部项）在提交时跑；指定范围（`--item`、`--check`、点名测试目标）由主 agent 随时经 `task-server/task submit` 交（内存额度、卡与清场由服务定）。验证包的测试、QEMU（门禁 qemu-device-streams）、herd7（门禁 lkmm）、crates 变异整表（门禁 crates-mutation-replay）、全部实验复跑（门禁 research-build-and-replay 的 experiment-replay 格）与崩溃点重放（门禁 crash-point-replay）都只分这两种跑法，不分快慢。门禁 crash-point-replay 判两路：按条那一路是 `cargo test --release -p singlefs-verification --lib --tests`（`--tests` 带上全部 `test = true` 的目标，装置二进制的内联单测也在里面），再一趟带 `tests/` 里 `cfg` 点到的特性（`--features <特性> --lib --test <挂特性的测试文件>…`，是它的 feature-gated-tests 格），每条测试函数照单元键复用，登记在 `.claude/gate.d/stage-inputs.tsv` 键为 `crash-case:` 的崩溃枚举用例用 `--skip` 排掉；登记的用例交按块那一路（登记行第三列带 `separate-run=` 的用例没被点名时不跑、报本次未跑），块照块键复用、没判过的当场判；分层按已定项 9。崩溃放量判器整套上了 GPU，罩全部池级 checker 不变量（`crates/singlefs-verification/src/crash_judge_gpu.rs`）。**池级 checker** 是 `crates/singlefs-checker`，判决器库，被验证包调用。崩溃枚举用例一律写在 `crates/singlefs-verification/tests/` 里（`scripts/crash/crash-case-check.py` 判）。

**射程**：管 `crates/` 下验证代码与测试住哪个包、门禁怎么跑、验证包与池级 checker 怎么称呼（`.claude/rules/verification.md`「定义与名字」）；不管每条不变量判什么（[invariants/README.md](../../../crates/singlefs-checker/invariants/README.md)）、不管枚举域怎么定（已定项 4）、不管池级 checker 与实现共享什么（已定项 5：池级 checker 仍只依赖 `singlefs-format`）。上游 SOP 不再写这个项目的验证手段：崩溃点重放、模型对拍、checker 即规范、文件系统特有的反推缺口四节与 `gate.sh` 未实现清单里的项目键都收进本仓（`.claude/rules/verification.md`、`.claude/gate-not-implemented.tsv`），上游那一批已同步进副本、未发版。

**依据**：
- 无实验：住哪个包、门禁怎么跑是流程政策，没有可量的量；验证包的判别力由它的用例与变异表证。
- 用户定案（2026-11-05），原话在变更史：不分轻重快慢、只有一个门禁、门禁分指定范围与不指定范围两种跑法（`records/2026-11-05-删轻型并入重型.md` 第 108 行）；两个验证代码包并成一个、不留残留（同一份记录第 286、287 行）；新包名 `singlefs-verification`、七道门禁去掉档位前缀（同一份记录第 307 行）；需要改的彻底改掉（同一份记录第 311 行）。未经三方攻过。

**欠**：无。

```

**出处 `.claude/kb/decisions/13-验证路线.md:415-431`（整段抄，未转述）**

```markdown
#### 已定项 16：按段登记的段整域装不进 u64 就分片判，态数用 1280 位定宽整数

**定案**：
- 分片：崩溃放量里按段登记的段（`.claude/gate.d/crash-point-replay-segments.tsv`，一行一段五列：项名、流名、段号、段写数、段写表摘要），整域态数装不进 u64 时按固定大小的片依次判；片大小由环境变量 `SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES` 定，没设取 2^32 态；片起点是判定库里的续点（续点按段写表摘要与判器单元键寻址，项名、流名、段号只作显示）。整域装得进 u64 的登记段不分片，整域一次判。代码落点：`crates/singlefs-verification/src/crash_amplification.rs:21713` `REGISTERED_SEGMENT_SLICE_STATES_ENVIRONMENT_VARIABLE`、`crates/singlefs-verification/src/crash_amplification.rs:21885` `fn segment_slices_of_the_recording_from_the_environment_and_the_library`。
- 位宽：登记段的总态数、片起点、片尾与进度一律用 1280 位（10 个 u128）定宽无符号整数 `StateCount1280Bits`（`crates/singlefs-verification/src/crash.rs:1640`，肢数是同一文件的 `STATE_COUNT_LIMBS`），算术全部 checked，不饱和、不截断；驱动行 `registered_segment_progress=` 的已判态数与总态数都写精确十进制，不写幂的形态。
- 登记表怎么交给引擎：引擎只经环境变量 `SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY` 读按段登记表（`crates/singlefs-verification/src/crash_amplification.rs:21676`，在 `fn registered_segments_of_the_flow_from_the_environment` 里），没设就当一段都没登，没有别的退路。144 个单元的文件顺序写成 145 个、extent 下段长成两层的那条流（引擎流名 `position_addressed_ExtentLowerSegmentGrowsToTwoLevels`）的单元写段（322 次单元写）与记录段（290 次 journal 记录原地写）两段都登。

**射程**：管崩溃放量引擎对按段登记的段怎么分片、态数用什么整数记、登记表从哪里读；不管哪几段该登（由用户点头，写在登记表文件头）、不管登记段的域本身怎么展开（判决 `research/prompts/m3-crash-enumeration-merge-r3-main-verification.md` 的 J10；登记段的域现行是每段写数不超过这一趟展开上限的取全部真子集、超过的取甲二，即每段原地写取任意子集、COW 写只取全落或全不落，不取 J10 定的「甲二 ∪ 展开 6」）、不管每个崩溃状态怎么定义（已定项 4）。没登记的段的态数、与整条流合起来的态数仍要装得进 u64，装不下在写库之前拒跑；登记段整域越过 1280 位同样在写库之前拒跑。片大小只许 1 到 2^32：两个 GPU 内核把片内序号当 u32 加到片起点上（`crates/singlefs-verification/src/crash_amplification.rs:21720` `LARGEST_REGISTERED_SEGMENT_SLICE_STATES`）。展开上限盖不住又没登记的段，放量入口 `scripts/crash/crash-amplification.sh` 判红；崩溃点重放入口、放量入口与分项驱动开跑前核这份表再送给引擎。

**依据**：
- E161（崩溃放量的去重与分段耗时）「第 2 次重跑：取样域按 11 段的流重定（2026-10-05）」一节：段的全量态数与闭式、装置逐个数、crates 计划逐段相等，态数随段写数按幂长，撑的只是「长段整域装不进 u64」这个前提；片大小、位宽取值与登记表的读法是工程参数，用户定，它不撑。
- 用户定案，原话在变更史：分片与每片 2^32 态（`records/2026-10-11-崩溃放量两段与脚本审计.md` 第 96 行）、位宽 1280 位（同一份记录第 98、99 行）、两段都登（同一份记录第 92 行）。
- 分片与位宽是对判决 `research/prompts/m3-crash-enumeration-merge-r3-main-verification.md` 的 J10（按段登记）的补充，未经三方攻过。
- 登记段的域照这一趟的展开上限展开：用户定案（2026-11-05），原话在变更史与 `records/2026-11-05-删轻型并入重型.md` 第 102 行；主 agent 采用条款审定点「登记段的域」取的那一边（同一份记录第 128 行），未经三方攻过。

**欠**：无。

```

**出处 `.claude/kb/decisions/13-验证路线.md:432-460`（整段抄，未转述）**

```markdown
#### 已定项 17：判定复用的单元是函数，键只由函数单元、工具链、依赖版本组成

**定案**：
- 门禁、崩溃判定库与实验准入的每一份判定（崩溃状态块、按条复用的每条测试函数、crates 变异每行、广谱变异每条、形式化每条、QEMU 每档、litmus 每份、研究复跑每行、实验变异表每条、实验准入）沿不沿用，只看它的单元键：它依赖的函数单元各自去注释后的指纹，加闭包里各单元（含 mod 头）的 `#[cfg]`、`#[cfg_attr]`、`cfg!` 真引到的 cfg 原子、各带这一次目标下的真值（闭包没引到的原子不进键：带不带 `gpu` 特性编的两趟，闭包不提 `gpu` 的判定键相同、共用一份库），加工具链（含 cargo 解析后的生效构建环境与外部工具版本）、从 `Cargo.lock` 解析出的闭包真引到的依赖版本，加这份判定自己的内容输入（段写表与起步镜像摘要、litmus 内容、变异原文与替换文这类）。函数单元从这份判定的入口沿调用与引用收，连同引用的常量、类型、静态项与宏展开一起算；wgsl 按 fn 切。
- 一律不进键、也不许当复用依据：按提交、HEAD 或树算的改动范围；整文件原文；文件级、包级、整表级的指纹或闭包；手写的版本常量；路径（仓根也算）、名字、种子串、行号、资源上限、限时；只有整道或整格的标记而没有按判定的库。键过窄（引用的常量、类型、宏漏收）、库放在会丢的位置、改名丢库、两份库重复判同一批，同样算违反。
- 收不全（动态分派、宏、`include_str!(concat!…)` 认不出）的那一份拒算键、照跑，不静默少收；每把键连同组成它的单元清单落盘，作废时报得出是哪几个单元变了。
- 三种排除不进键，理由不同、分开记。文件级：`scripts/crash/lib_verdict_library.py` 的 `NOT_JUDGING_FILES` 那两份（算键的库自己与单元算法 `scripts/gate/lib_function_units.py`）按角色排，管怎么存、怎么算键，不管判对错，它们的单元不进任何一份判定的键，crates 变异判法（`scripts/mutation/crates-mutation-rows.py`、`scripts/mutation/lib_mutation_units.py`）除外：那里这两份照进判法单元指纹，覆盖号也带判法单元指纹。边级：一条只打提示、不改判定结局的子进程边，按实测排，不沿它收；边的清单以 `scripts/gate/lib_function_units.py` 的 `EXCLUDED_PROGRAM_EDGES` 登记为准，登记了的都只出提示，按这些边排出键。边级排除要配三格自证：这条路上的任何异常不改包装退出码；判法读包装输出前去掉这条路打的行；切边前后少掉的单元只经这条边可达。另有一格「键不变而退出码变」必红。单元级：Rust 一侧按判器组、按单元（限定名）登记「不沿它收」，收闭包时够到登记的单元，它不进这一组的闭包、不从它往下收（经别的边够得到的单元照进），这一趟真够到的几个另外交出；登记放在判器组入口登记表旁（`crates/singlefs-verification/src/judging_units_entries.rs` 的 `units_not_followed_of_the_group`），今天只有 GPU 判器那一组有登记，分四张：显存记账那一张（`UNITS_OF_THE_MEMORY_LEDGER_THE_GPU_JUDGE_GROUP_DOES_NOT_FOLLOW`）、只经包装设备那一张（`UNITS_REACHED_ONLY_THROUGH_WRAPPER_DEVICES_THE_GPU_JUDGE_GROUP_DOES_NOT_FOLLOW`）、只打 GPU 阶段现场那一行的那一张（`UNITS_THAT_ONLY_PRINT_THE_GPU_LIFECYCLE_LINE_THE_GPU_JUDGE_GROUP_DOES_NOT_FOLLOW`）、只经只打 GPU 阶段现场那一行的单元才够得到的那一张（`UNITS_REACHED_ONLY_THROUGH_THE_UNITS_THAT_PRINT_THE_GPU_LIFECYCLE_LINE_THE_GPU_JUDGE_GROUP_DOES_NOT_FOLLOW`），别的组是空的；收法在 `crates/singlefs-unit-graph/src/closure.rs` 的 `compute_closure_of_entries_not_following_the_listed_units` 与 `crates/singlefs-unit-graph/src/entry_groups.rs` 的 `entry_group_key_not_following_the_listed_units`，登记是空的，键与不带登记的逐字相同。登记配这几组测试（`crates/singlefs-verification/src/crash_identity.rs`）：四张登记共用两条，不沿它收少掉的恰好是登记的那些（陈旧的登记与跟着被切掉、没逐个判过的单元都红），登记的每个名字在工作区里恰好一个单元；显存记账那一张另有两条，不带登记收时显存记账里够到的恰好等于登记的（陈旧的登记与新进来的单元都红），改其中一个登记了的单元，这一组的键不变；只经包装设备那一张另有一条，留下的单元不写登记里的包装类型名；只打 GPU 阶段现场那一行的那一张另有五条，留下的单元读不到它们交的值，它们不会 panic、也不改调用方手里的东西，分别改 `gpu_lifecycle_trace`、`gpu_work_trace`、`gpu_lifecycle_line`，这一组的键都不变；只经只打 GPU 阶段现场那一行的单元才够得到的那一张另有一条，别的三张照不沿它收、只沿这一张收时，这一张里至少有一个被真够到，带全部登记收时它们一个都没被真够到。python 模块有单元进闭包时，它在 import 时执行的模块级语句一起收进来：import、赋值、类头本身（不连方法）、带装饰器或缺省值的 def 头。
- 判定库只有一份：prune-server 那一份 RocksDB（`.claude/rules/verification.md`「判定库只有一份：全仓统一写进 prune-server 的那份 RocksDB」），各类判定按各自的键前缀分，不随一趟、不随判法换；仓根也不是库的输入；两台判出的都直接写进这一份，不拷库、不导入。
- 红的也照键记：门禁、崩溃判定库与实验准入的每一份判定判红时，照同一把键写一条红记录、撤掉同一把键已有的绿；键相同而库里记着红的，下一趟现判、不复用；只有判绿的可以复用（按条复用的测试函数、崩溃放量的块、变异、形式化同一个规矩）。
- 改键让旧库作废不算代价：不做兼容迁移、不为旧行补签，改完重跑。
- Verus 的规格与证明块：判产品行为的那几类判定，单元键算指纹时剥掉 Verus 的规格与证明块，只改证明、产品代码不变时键不变；剥法用 Verus 自己的语法树（`verus_syn`），不用词法。形式化判定的键保留规格，剥掉会假绿。

**射程**：管门禁、崩溃判定库与实验准入的复用键；不管静态门禁「只判新加行」的改动范围（`scripts/gate/changed-paths.sh`，那是判据语义）与 preflight 的 `admission: inputs-changed`；不管枚举域（已定项 4）、分层与提交时跑什么（已定项 9）。镜像判定与给定状态判定的版本是判定定义那一组（CPU 参照判器 `crash::judge_crash_image`、`crash_given_states::judge_the_given_states_on_the_cpu` 的闭包）连同 GPU 判器那一组（含 WGSL 内核）的单元一起算出的键，不论这一块在 CPU 还是 GPU 上判的，键相同，改 GPU 判器那一组的任一单元，这一类的每一块（CPU 判过的也在内）都作废、现判：GPU 判器是加速路径，它的准入靠与定义那一组的对拍（GPU 与 CPU 对拍用例），块不按设备分种类；显卡驱动、卡型、几张卡怎么分活留作块的属性或跑的参数，不进键。键算法里不设「包装类型的 impl 等类型被点名」这类通用规则：三轮三方都被打中，已撤回，键算法本身没动；GPU 判器那一组要切的几类（故障注入、读计数、核心层挂载这类包装设备）走上面的单元级登记。

**依据**：
- 用户定案（2026-10-15），原话在变更史与 `records/2026-10-13-复用键与全仓改名.md`「2026-10-15 用户定：提交与重跑无关，重跑只跟代码变化走」一节、「2026-10-15 批3」那几段。
- 用户定案（2026-10-17）：判定不与硬件、不与 CPU 还是 GPU 绑定，复用与剪枝不看设备；原话在变更史与 `records/2026-10-13-复用键与全仓改名.md` 2026-10-17 叫停全量那几段。
- 用户定案（2026-10-29）：判红的记红、撤掉同一把键的绿、记着红的下一趟现判，按条复用的测试函数、放量、变异、形式化四处同一个规矩；GPU 判器的单元进块键（主 agent 定的口径：CPU 判与 GPU 判的块键仍相同）；原话在变更史，核查与现查在 `records/2026-10-29-崩溃轻型与重型同一套逻辑.md`「批次 4：复用逻辑独立核查」「批次 5：照核查修复用（用户 2026-10-29 弹窗定案）」两节。
- 主 agent 定案（2026-10-15）：仓根不进键与库、cfg 成分只取闭包真引到的原子，出处同一份记录「2026-10-15 验证段第一步判红、两条定案、4-3 四批派出」定案第 1、2 条。
- 用户定案（2026-11-02）：判定库只有 prune-server 那一份、不留兼容开关，原话在变更史；现行规则 `.claude/rules/verification.md`「判定库只有一份：全仓统一写进 prune-server 的那份 RocksDB」。
- 反证：E164（崩溃放量新设计的小范围功能测试） F4 量到只按函数取指纹、不连引用的常量一起收，漏判 V5 与 V11（V11 只改常量 `DATA_POINTER_BYTES`）：函数单元要连同引用的常量、类型、静态项与宏一起算。
- 盘上的数：`ledger.md`「一之二、盘上量到的」一节——只差名字的块各存各判 4100 块与 9679 块；整库一把的判法摘要一换累计多判 3 935 242 934 态，那次闭包里改动的 29 份文件有 24 份去注释后逐字相同：键带名字、按文件或整版取指纹，同一份判定会重判。
- 三方判决 `research/prompts/m3-close-reuse-closure-r1-main-verification.md`、`research/prompts/m3-close-reuse-closure-r2-main-verification.md`、`research/prompts/m3-close-reuse-closure-r3-main-verification.md`（两种排除并列、模块级语句随模块进闭包、第二台那一路只出提示按边排出键；三轮定，零轮改法交用户定）；用户定案 2026-10-22「丙加甲 要走三方」、2026-10-23 弹窗四题，原话在变更史。
- 没有腿况记录：除复用键闭包三轮（两种排除、模块级语句、第二台那一路）攻过的几处外，未经三方攻过；用户定改完代码开三轮攻方，只攻「剪枝是不是原子化的、键是不是只由函数单元组成」。
- 三方判决 `research/prompts/m4-exit5-relocation-proof-r2-main-verification.md` 第二节判法与门禁那一格（判器闭包键去掉 Verus 的规格与证明块）、`research/prompts/m4-exit5-relocation-proof-r3-main-verification.md` 第二节判器键两格（只剥判产品行为的那几类、形式化判定保留规格、用 `verus_syn` 剥；辩方提，零轮）：不这样剥时，改一次证明就让判定库整库作废，一次约 3.9×10⁹ 到 3.8×10¹⁰ 态（夹出来的，推的）。
- 三方判决 `research/prompts/m3-close-unit-graph-wrapper-impls-r1-main-verification.md`、`research/prompts/m3-close-unit-graph-wrapper-impls-r2-main-verification.md`、`research/prompts/m3-close-unit-graph-wrapper-impls-r3-main-verification.md`（按组按单元登记不沿它收，机制与四条测试两轮站住；在键算法里做「包装类型等被点名」的通用规则三轮都被打中，整类盖住时 GPU 判器那一组的闭包比不改的还大）；用户定案 2026-11-13 弹窗，原话在变更史。

**欠**：C8（门禁范围判不出来）、C489（singlefs-core 的广谱变异对今天的代码没有结论）、C528（两条门禁路判的改动窗口不同）、C564（另外 6 条崩溃点重放的全域还没判过）、C605（崩溃放量的复用与剪枝没有关掉再判的对照）、C630（提交时形式化那一道第 1 步停下的频率没量）、C645（崩溃放量不指定范围那一趟没进提交流程）、C836（staged-green 挂的树不一定是被判的树）、C1031（全具体类型 impl 的两形不收）、C1032（只经包装设备的登记没被攻过）、C1033（两个显存记账单元没有会红的检查）、C1034（十六类写法的真仓扫描没立）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:461-481`（整段抄，未转述）**

```markdown
#### 已定项 18：GC 恒等性质与 core 直验

**定案**：
- 性质：整理不改现行内容；回退候选集是不整理那份的后缀、至多少 k 个，抬 F 的上限恰多 k（k 为这一段整理占的发布数），按版本号点名。这是判决 `research/prompts/m4-gc-r1-main-verification.md` 里恒等变换那一格性质句的现行写法，那份判决原文冻结不改。
- 层：在 `crates/singlefs-core` 上直接用 Verus 证，连胶水一起证；抽象模型的证明加模型对拍不算证出。
- 证到哪：发布、回退、崩溃后读映射三个闭包，加抬 F 一路、挂载重建分配器之后的回收、丢弃（含 `discard_at` 的 trait 默认实现）、实例表清扫抹头四块；第二轮正推腿给闭包里的函数单元分的 A、B、C、D 四类都在里程碑四做完，分批只排次序，不减射程。
- 抽象状态：`gc_crash_proofs` 的 `logical` 补回退目标视图、多逻辑版本与「内存 F / 持久 F」两层；根环协议按区间、映射值带跨度；目的地前提取按区间不重叠的那一版。
- 另进证明义务的三条前提：只供测试的开关挪进默认关的 cargo 特性（`set_reuse_window` 拨到 `ForcedToZero` 会绕过 F）；「先内存抬 F、持久后才放开复用」由协议的两层 F 写出；`gc_crash_proofs.rs` 里「门槛蕴涵没被引用」那一处写成引理。
- 门禁：去掉 `--no-cheating`，换成登记式信任清单（每一处 `assume_specification`、`external_body`、`uninterp` 点名 std 函数、记规格原文 sha256、配一条对 std 实际行为的差分测试，没登记的判红；vstd 自带的那批按 vstd 版本整体登一行）；`axiom` 一律不许；被验区域里任何 cfg 原子都判红（信任清单登记的除外）；被验类型的 trait impl 一律写在 `verus!` 里；产品构建与 `verus --compile` 构建的 MIR 逐函数比；Verus 与产品的 rustc 版本差进工具链钉表；按函数单元的全路径（含 impl 与 trait）逐项验。

**射程**：管整理的恒等性质怎么证、证到 core 的哪几条路径、门禁怎么判。恢复函数 `Recover` 的四个里程碑归已定项 1，不在这一条；checker 与实现共享什么归已定项 5（core 加 vstd 不进池级 checker 的依赖闭包）；校验和照旧当未解释函数（已定项 14），进信任清单。罩不到的那十五项（设备、只能当未解释函数的校验和这一类）各由什么罩，在第二轮正推腿报告 `research/prompts/m4-exit5-relocation-proof-r2-sonnet-output.md`「罩不到的」那一节。代价是推的：按这一形的射程，证明代码约 3.9 万到 20.8 万行，倍数随类别差一个数量级；实现第一批量出 A 类的实际倍数后回写。

**依据**：
- 无实验：三轮的数出自两条腿各自模型目录里的原型与复跑（各份判决第五节），没有立实验号。
- 三方判决 `research/prompts/m4-exit5-relocation-proof-r1-main-verification.md` 第二节：抽象状态取 `logical` 并补回退目标视图，性质句要连回退候选一起说，目的地前提改成按区间不重叠。
- 三方判决 `research/prompts/m4-exit5-relocation-proof-r2-main-verification.md` 第二节：接法、信任清单替 `--no-cheating`、射程加崩溃后读映射那一路、判法改十一条与按全路径逐项验、罩不到的十五项。
- 三方判决 `research/prompts/m4-exit5-relocation-proof-r3-main-verification.md` 第二节与第三节：根环协议按区间（两腿各自得到，零轮）、性质句照辩方改写（零轮）、三条前提进证明义务、门禁收严三条、射程加四块（辩方打中）、代价区间（推的）。
- 用户定案 2026-10-26：层「core 直验，连胶水一起证」、「里程碑四全验」，原话在变更史。

**欠**：C921（只供测试的开关在发布构建里够得着）。

```

**出处 `.claude/kb/experiments/152-按里程碑对比六家文件系统的文件性能.md:1-33`（整段抄，未转述）**

````markdown
## E152 按里程碑对比六家文件系统的文件性能 —— 已跑（七次正式跑，第七次 2026-09-29：两盘镜像七家 + singlefs 各 5 轮，40 次一次过；2026-11-17 又跑一次同一组，40 次一次过，产物头被入口标成缩过、用户定照用；逐次见「各次跑（新的在前）」；26 单测 / 26 条变异全抓）

**中文** · [English](152-file-performance-of-six-file-systems-compared-by-milestone.en.md) · [日本語](152-マイルストーン別に-6-ファイルシステムのファイル性能を比較.ja.md)

问的是：同一台虚机、同一种虚拟盘、同一组负载下，XFS、ext4、F2FS、Bcachefs、Btrfs、OpenZFS 六家与 singlefs 在文件性能上各在什么位置、singlefs 差在哪里。
用户定向，维度是大文件顺序写、大文件顺序读、4K 随机读、4K 随机写、元数据 / 小文件，主 agent 加了小文件持久化、冷挂载、格式化与可用容量三维；
按里程碑对比的结果写在 `scripts/experiment-apparatus/perf-by-milestone.md`。**只在用户说跑时才跑**（登记在 `scripts/experiment-apparatus/on-request-experiments.tsv`，2026-09-19 用户定）：装置改了不重跑，门禁 doc-experiments 的 evidence-in-repo 格改查这一页写没写「没有正式跑」。只出观测；「差距说明 singlefs 该怎么改」是推论，要另走三方论证。

**备料**：不支撑任何一条决策。要用它的时候是 D23（journal 的角色与格式） 那一侧判冷恢复的读量要不要降、怎么降，以及 D17（实现分层与第三方管道） 已定项 5 那一侧判什么时候加 FUSE（新池新建文件上八维里只比得了三维）。

**被测对象与参数**：六家取各家默认参数（ext4 只加挂载选项 `noinit_itable`），各一块 16 GiB virtio 盘；另有裸盘一行当天花板。
singlefs 在里程碑「新池新建文件」上没有文件接口（D17（实现分层与第三方管道） 已定项 5：第一版是纯用户态库，不挂载），
只能跑门禁 qemu-device-streams 那个真设备二进制，两块盘（D2（RAID 条带策略） 已定项 9）。版本、命令与次序逐字在跑前登记 `e152-preregistration.md`。

**测试台**：本机 QEMU 8.2.2 + KVM，4 vCPU、4 GiB，虚机内核 6.17.0-1028-oem（与宿主同版本、不带 lockdep，用 `scripts/devices/vm-kernel.sh --release` 取）；
盘是 TMPDIR 里 fallocate 预分配的 raw 文件，`cache=none,aio=native`。量出来的是这个测试台上的相对位置，绝对值不与外面的公开数比。

**复跑命令**：

```bash
bash scripts/devices/e152-run.sh research/results/e152-file-system-benchmark-<日期>.out   # 单盘：八个配置按轮交错跑 5 轮
E152_CONFIGURATIONS="raw-md ext4-md xfs-md f2fs-md btrfs-raid1 bcachefs-replicas2 zfs-mirror singlefs" \
  bash scripts/devices/e152-run.sh research/results/e152-file-system-benchmark-mirror-<日期>.out   # 两盘镜像
E152_ROUNDS=1 E152_CONFIGURATIONS="ext4" bash scripts/devices/e152-run.sh <产物>              # 冒烟：指定范围加缩域，产物头接 reduced=yes … not_e152_product=yes，不当 E152（按里程碑对比六家文件系统的文件性能） 的产物
E152_CONFIGURATIONS=singlefs bash scripts/devices/e152-run.sh research/results/e152-file-system-benchmark-file-overwrite-<日期>.out   # 里程碑之后只重跑 singlefs 一臂
```

`E152_CONFIGURATIONS` 是指定范围，`E152_ROUNDS` 是缩域；与登记的轮数或配置不同的那一趟，产物头 `E152RUN_HEADER` 接 `reduced=yes registered_rounds=… registered_configurations=… not_e152_product=yes`；入口的登记值是单盘那八个配置名，上面两盘镜像那条命令的配置名与它不同，所以 2026-11-17 那份产物头也带这一段（更早的没有，它们跑的时候入口还没有 `reduced_note`）。正式跑八个配置 5 轮约 45 分钟，这段时间别在本机编译、别跑门禁（crash-point-replay、qemu-device-streams、research-build-and-replay 吃 CPU 或起虚机，会弄脏计时）。

代码 `scripts/experiment-apparatus/e152_file_system_benchmark.rs`（26 单测；来宾一轮负载与宿主汇总两个模式），
变异表 `scripts/mutation/tables/e152_file_system_benchmark.tsv`（26 条全抓），附加根 `scripts/devices/e152-stage-root.sh`，驱动 `scripts/devices/e152-run.sh`，出表 `scripts/experiment-apparatus/e152-tables.py`（`--selftest`）。
原始输出 `e152-file-system-benchmark-2026-09-15.out`（单盘，890 行，末行 `emitted=174`）与 `e152-file-system-benchmark-mirror-2026-09-15.out`（两盘镜像，941 行，末行 `emitted=174`），跑前登记 `e152-preregistration.md`（第十节是正式跑之前的六次修订与四次冒烟跑）。

````

**出处 `.claude/kb/experiments/152-按里程碑对比六家文件系统的文件性能.md:34-46`（整段抄，未转述）**

```markdown
### 各次跑（新的在前）

| 日期 | 跑了什么 | 轮 / 虚机跑 | 产物（`research/results/`） | 结果在哪 |
|---|---|---|---|---|
| 2026-11-17 | 工作区改动之上，两盘镜像七家 + singlefs（产物头带 `reduced=yes … not_e152_product=yes`，用户定照用） | 5 / 40，全 `vm_exit=0` | `e152-file-system-benchmark-mirror-2026-11-17.out` | 「2026-11-17：两盘镜像七家 + singlefs」一节；与 2026-09-29 那次逐格比在 `scripts/experiment-apparatus/perf-by-milestone.md` 三·五 |
| 2026-09-29 | 崩溃放量全量之后清场，两盘镜像七家 + singlefs | 5 / 40，全 `vm_exit=0` | `e152-file-system-benchmark-mirror-2026-09-29.out` | 「2026-09-29：两盘镜像七家 + singlefs」一节；与 2026-09-28 那次逐格比在 `scripts/experiment-apparatus/perf-by-milestone.md` 三·四 |
| 2026-09-28 | 里程碑二收尾第 8 步，两盘镜像七家 + singlefs | 5 / 40，全 `vm_exit=0` | `e152-file-system-benchmark-mirror-2026-09-28.out` | 「2026-09-28：两盘镜像七家 + singlefs」一节；singlefs 与 2026-09-17 逐行比在 `scripts/experiment-apparatus/perf-by-milestone.md` 三·三 |
| 2026-09-19 | 只跑 singlefs 一臂（装置改为先读完再转打），另跑 P1 复算、G1 单 vCPU 3 轮 | 5 + 3 | `e152-file-system-benchmark-second-transaction-relay-timing-2026-09-19.out` 等 | 「2026-09-19：装置改为先读完再转打之后的 singlefs 一臂」一节 |
| 2026-09-17 收尾 | 只跑 singlefs 一臂（里程碑收尾时） | 5 | `e152-file-system-benchmark-second-transaction-final-2026-09-17.out` | 「2026-09-17 收尾」一节；与镜像六家比在「这几个数说明什么（覆盖写、释放、回退与复用，与两盘镜像的六家比）」一节 |
| 2026-09-17 步 7 | 只跑 singlefs 一臂（`file-overwrite` 模式） | 5 | `e152-file-system-benchmark-second-transaction-2026-09-17.out` | 「2026-09-17 收尾」一节里的「步 7」那一段 |
| 2026-09-15 两盘镜像 | 七个镜像配置 + singlefs | 5 / 41（1 次重跑） | `e152-file-system-benchmark-mirror-2026-09-15.out`（已归档） | 「2026-09-15 两盘镜像」一节；`scripts/experiment-apparatus/perf-by-milestone.md` 二·二 |
| 2026-09-15 单盘 | 八个配置各一块盘 | 5 / 40 | `e152-file-system-benchmark-2026-09-15.out`（已归档） | 「2026-09-15 两盘镜像」一节里的「单盘」那一段；`scripts/experiment-apparatus/perf-by-milestone.md` 二 |

```

**出处 `.claude/kb/experiments/152-按里程碑对比六家文件系统的文件性能.md:47-148`（整段抄，未转述）**

````markdown
### 2026-11-17：两盘镜像七家 + singlefs

代码是提交 `8ed2ea4845ef0023fcf9a41e3ac09fece4423c0d`（`git rev-parse HEAD` 现查）之上带未提交改动的工作区（`git diff HEAD --shortstat -- crates/singlefs-core` 数出 35 个文件有改动），与 2026-09-29 那次（dbcc0d2f，核心代码工作区没有改动）不是同一份。命令（本机跑，用时 2 410 秒）：
`env 'E152_CONFIGURATIONS=raw-md ext4-md xfs-md f2fs-md btrfs-raid1 bcachefs-replicas2 zfs-mirror singlefs' bash scripts/devices/e152-run.sh research/results/e152-file-system-benchmark-mirror-2026-11-17.out`。
产物 1026 行，末行 `E7RESULT name=summary_done emitted=180`；40 次虚机跑全部 `vm_exit=0`、没有重试（`grep -c '^E152RUN .*vm_exit=0' <产物>` 数出 40，`grep -c 'attempt=2' <产物>` 数出 0）。

宿主 1 分钟负载 1.61–27.19（2026-09-29 是 0.50–9.61），高的四次整行（`grep -n '^E152RUN ' <产物>` 取）：

```
101:E152RUN configuration=bcachefs-replicas2 round=1 attempt=1 host_load1=10.10 vm_exit=0
121:E152RUN configuration=zfs-mirror round=1 attempt=1 host_load1=27.19 vm_exit=0
140:E152RUN configuration=singlefs round=1 attempt=1 host_load1=10.02 vm_exit=0
171:E152RUN configuration=raw-md round=2 attempt=1 host_load1=6.29 vm_exit=0
```

其余 36 次低于 6。第 1 轮那三次期间本机有一条别的任务在编译；这一份产物经用户看过这几行之后定为照用，负载高的格由表里的离散标记照报、不删。
产物头第 1 行带 `reduced=yes registered_rounds=5 registered_configurations=raw,ext4,xfs,f2fs,btrfs,bcachefs,zfs,singlefs not_e152_product=yes`（整行 `E152RUN_HEADER kernel=6.17.0-1028-oem rounds=5 configurations=raw-md,ext4-md,xfs-md,f2fs-md,btrfs-raid1,bcachefs-replicas2,zfs-mirror,singlefs …`，`started_jst=` 那一段是开跑钟点，不抄）：入口 `scripts/devices/e152-run.sh` 的 `reduced_note` 只认单盘那八个配置名为登记值，两盘镜像这一组名字不同就被标成缩过，轮数与配置其实是登记的（2026-09-29 那份产物头没有这一段，`reduced_note` 是那之后才有的）。

与 2026-09-29 逐格比（`scripts/experiment-apparatus/perf-by-milestone.md` 三·五）：207 格里变了 15% 以上的 25 格（六家 22、singlefs 3），每一格两天里至少有一天带 ⚠；带 ⚠ 的格 34 → 69 个，除裸 md 持平（1 个）外每家都增多。singlefs 一臂：发布 B 的写请求、字节、屏障、FUA 与 2026-09-29 逐字相同（29 / 475 648 / 6 / 1），二进制自己计的中位 19.1 → 15.4 ms；第一段与冷恢复做的活变了（每盘写 805 959 680 → 807 012 352 字节，盘 1 写请求 1 596 → 832，冷恢复每盘多读 114 688 与 40 960 字节），冷恢复中位 9 914 → 13 918 ms（第 1 轮 35 696 ms、第 2–5 轮 13 367–14 561 ms），第一段中位 348 → 363 ms；冷恢复变慢的原因没查。

六个维度的表与 singlefs 那一张（`python3 scripts/experiment-apparatus/e152-tables.py research/results/e152-file-system-benchmark-mirror-2026-11-17.out`，整份原样取前七张；块层放大倍数与对照两张在同一份输出里，抄在 `perf-by-milestone.md` 三·五）：

#### 大文件顺序写与顺序读（MiB/s，越大越好）

| 配置 | 顺序写 | 顺序读 |
|---|---|---|
| 裸 md raid1（镜像天花板） | 2 677 | 5 126 |
| ext4（md raid1） | 2 504 ⚠19% | 4 935 |
| XFS（md raid1） | 2 251 ⚠24% | 4 870 |
| F2FS（md raid1） | 2 579 | 5 107 ⚠16% |
| Btrfs raid1 | 1 363 | 4 746 |
| Bcachefs 双副本 | 2 299 ⚠47% | 4 713 ⚠53% |
| OpenZFS mirror（vdev 是分区） | 519 ⚠19% | 1 852 |
| singlefs（两盘） | — | — |

#### 4K 随机读写（次/秒越大越好；p99 是完成延迟，µs，越小越好）

| 配置 | 随机读 次/秒 | 随机读 p99 | 随机写 次/秒 | 随机写 p99 |
|---|---|---|---|---|
| 裸 md raid1（镜像天花板） | 64 940 | 700 ⚠27% | 37 533 | 905 |
| ext4（md raid1） | 59 966 ⚠18% | 823 ⚠136% | 34 803 | 1 089 |
| XFS（md raid1） | 59 277 ⚠28% | 741 ⚠161% | 34 465 | 1 089 ⚠42% |
| F2FS（md raid1） | 60 514 | 626 ⚠45% | 35 367 | 1 056 |
| Btrfs raid1 | 50 236 | 667 ⚠36% | 10 791 ⚠45% | 3 489 ⚠575% |
| Bcachefs 双副本 | 28 177 ⚠71% | 1 564 ⚠416% | 34 945 ⚠72% | 1 679 ⚠742% |
| OpenZFS mirror（vdev 是分区） | 2 881 | 11 993 ⚠28% | 3 114 ⚠23% | 49 020 ⚠52% |
| singlefs（两盘） | — | — | — | — |

#### 元数据 / 小文件（个/秒，越大越好）

| 配置 | 建空文件 | stat | 删文件 | 4 KiB 写 + fsync |
|---|---|---|---|---|
| ext4（md raid1） | 232 558 ⚠29% | 909 090 ⚠46% | 294 117 ⚠20% | 881 |
| XFS（md raid1） | 178 571 ⚠55% | 909 090 ⚠22% | 196 078 | 981 |
| F2FS（md raid1） | 166 666 ⚠54% | 909 090 | 232 558 ⚠21% | 1 228 |
| Btrfs raid1 | 227 272 ⚠39% | 909 090 | 126 582 | 747 ⚠70% |
| Bcachefs 双副本 | 135 135 ⚠76% | 833 333 ⚠131% | 128 205 ⚠68% | 750 ⚠70% |
| OpenZFS mirror（vdev 是分区） | 51 020 ⚠167% | 666 666 ⚠21% | 30 674 ⚠57% | 1 086 ⚠57% |
| singlefs（两盘） | — | — | — | — |

#### 小文件持久化：新建 + 写 3000 字节 + fsync 文件 + fsync 目录（刚格式化的空文件系统上 100 次）

| 配置 | 延迟中位 µs | 延迟 p99 µs | 每次写请求 | 每次写字节 | 每次 FLUSH | 第一次写字节 | 第一次写请求 | 第一次 FLUSH |
|---|---|---|---|---|---|---|---|---|
| ext4（md raid1） | 1 050 | 1 354 ⚠48% | 8.00 | 72 581 | 4.00 | 73 728 | 8.00 | 4.00 |
| XFS（md raid1） | 986 | 1 330 ⚠32% | 6.00 | 19 927 | 4.00 | 13 312 | 6.00 | 4.00 |
| F2FS（md raid1） | 1 013 | 1 108 ⚠37% | 12.0 | 41 123 | 6.00 | 40 960 | 12.0 | 6.00 |
| Btrfs raid1 | 2 409 | 3 355 ⚠961% | 14.0 | 201 195 | 8.00 | 155 648 | 14.0 | 8.00 |
| Bcachefs 双副本 | 1 642 | 2 675 ⚠180% | 6.30 | 14 673 | 4.00 | 36 864 | 14.0 | 4.00 |
| OpenZFS mirror（vdev 是分区） | 909 ⚠63% | 1 005 ⚠197% | 4.00 | 8 192 ⚠16% | 2.00 | 8 192 ⚠288% | 4.00 ⚠350% | 2.00 |
| singlefs（两盘） | 15 368 ⚠19% | — | 36.0 | 475 648 | 7.00 | — | — | — |

#### 冷挂载（卸载、清缓存之后再挂）

| 配置 | 挂钟 ms | 读字节 | 读请求 |
|---|---|---|---|
| ext4（md raid1） | 1.60 ⚠26% | 21 504 | 6.00 |
| XFS（md raid1） | 7.59 ⚠79% | 4 235 776 | 45.0 |
| F2FS（md raid1） | 3.28 ⚠19% | 692 224 | 14.0 |
| Btrfs raid1 | 4.32 ⚠323% | 331 776 | 24.0 |
| Bcachefs 双副本 | 41.0 ⚠96% | 8 243 200 ⚠22% | 59.0 |
| OpenZFS mirror（vdev 是分区） | 6 113 | 10 785 280 ⚠23% | 241 |
| singlefs（两盘） | 13 918 ⚠160% | 1 612 242 944 | 393 694 |

#### 格式化与可用容量（16 GiB 盘）

| 配置 | 格式化 ms | 格式化写字节 | 格式化 FLUSH | 总容量 % | 可用 % |
|---|---|---|---|---|---|
| ext4（md raid1） | 78.3 ⚠30% | 143 197 184 | 14.0 | 48.8 | 46.3 |
| XFS（md raid1） | 90.3 ⚠55% | 135 160 832 | 10.0 | 49.8 | 48.7 |
| F2FS（md raid1） | 107 ⚠184% | 79 815 680 | 8.00 | 49.9 | 49.0 |
| Btrfs raid1 | 36.0 ⚠529% | 9 216 000 | 23.0 | 50.0 | 49.2 |
| Bcachefs 双副本 | 27.4 ⚠53% | 32 768 | 2.00 | 92.0 | 90.6 |
| OpenZFS mirror（vdev 是分区） | 543 ⚠40% | 3 580 928 | 26.0 | 46.9 | 46.9 |
| singlefs（两盘） | — | — | — | — | — |

#### singlefs（两块 16 GiB 盘，门禁 qemu-device-streams 的真设备二进制）

| 配置 | mkfs + 取号 + 暖机 + 新池新建文件 ms | 覆盖写、释放、回退与复用（覆盖写 B）ms | 发布 B（二进制自己计）ms | 冷恢复 ms | 两盘读字节 | 两盘读请求 | B 两盘写请求 | B 两盘写字节 | B 两盘屏障 | B 两盘 FUA 写 |
|---|---|---|---|---|---|---|---|---|---|---|
| singlefs（两盘） | 363 ⚠107% | 16.4 ⚠39% | 15.4 ⚠19% | 13 918 ⚠160% | 1 612 242 944 | 393 694 | 29.0 | 475 648 | 6.00 | 1.00 |

````

**出处 `.claude/kb/experiments/152-按里程碑对比六家文件系统的文件性能.md:149-235`（整段抄，未转述）**

```markdown
### 2026-09-29：两盘镜像七家 + singlefs

崩溃放量全量之后清场（两台上的推理与语音服务停了、两台显卡全空），本机没有别的编译与测试在跑；40 次虚机跑全部 `vm_exit=0`、没有重试，宿主 1 分钟负载 0.50–9.61（高的那几次在 OpenZFS 那几轮）。singlefs 一臂的代码与 2026-09-28 那次同一个提交（dbcc0d2f）。
singlefs 写读的字节与请求和 2026-09-28 那次逐字相同（每盘写 805 959 680 字节、1 598 次，冷恢复每盘读 806 371 328 字节、196 874 次），第一段中位 1 874 → 348 ms、冷恢复 5 轮 9 886–9 985 ms 没有离群，是宿主状态；两份表 207 格里变了 15% 以上的 13 格，多数在两天都标着不稳定的顺序写上（逐格比在 `scripts/experiment-apparatus/perf-by-milestone.md` 三·四）。

六个维度的表与 singlefs 那一张（`python3 scripts/experiment-apparatus/e152-tables.py e152-file-system-benchmark-mirror-2026-09-29.out`，整份原样取前七张；块层放大倍数与对照两张在同一份输出里）：

#### 大文件顺序写与顺序读（MiB/s，越大越好）

| 配置 | 顺序写 | 顺序读 |
|---|---|---|
| 裸 md raid1（镜像天花板） | 2 823 ⚠42% | 5 278 |
| ext4（md raid1） | 1 524 ⚠166% | 5 272 |
| XFS（md raid1） | 2 091 ⚠120% | 5 299 |
| F2FS（md raid1） | 1 853 ⚠98% | 5 238 |
| Btrfs raid1 | 1 287 ⚠37% | 4 947 |
| Bcachefs 双副本 | 1 736 ⚠95% | 4 836 |
| OpenZFS mirror（vdev 是分区） | 406 ⚠108% | 1 959 |
| singlefs（两盘） | — | — |

#### 4K 随机读写（次/秒越大越好；p99 是完成延迟，µs，越小越好）

| 配置 | 随机读 次/秒 | 随机读 p99 | 随机写 次/秒 | 随机写 p99 |
|---|---|---|---|---|
| 裸 md raid1（镜像天花板） | 63 319 | 522 | 37 538 | 888 |
| ext4（md raid1） | 59 637 | 552 | 33 444 ⚠23% | 1 056 ⚠28% |
| XFS（md raid1） | 59 653 | 552 | 34 434 ⚠35% | 1 036 ⚠100% |
| F2FS（md raid1） | 59 651 | 552 | 34 509 | 1 056 |
| Btrfs raid1 | 50 963 | 651 | 11 218 | 3 162 |
| Bcachefs 双副本 | 34 775 | 1 204 | 30 667 ⚠58% | 1 613 ⚠148% |
| OpenZFS mirror（vdev 是分区） | 3 076 | 10 944 | 4 718 ⚠91% | 17 694 ⚠4394% |
| singlefs（两盘） | — | — | — | — |

#### 元数据 / 小文件（个/秒，越大越好）

| 配置 | 建空文件 | stat | 删文件 | 4 KiB 写 + fsync |
|---|---|---|---|---|
| ext4（md raid1） | 243 902 | 909 090 | 303 030 | 821 ⚠22% |
| XFS（md raid1） | 227 272 | 909 090 | 196 078 | 910 |
| F2FS（md raid1） | 188 679 | 909 090 | 256 410 | 1 220 |
| Btrfs raid1 | 238 095 | 909 090 | 131 578 | 691 ⚠23% |
| Bcachefs 双副本 | 142 857 ⚠60% | 833 333 | 135 135 | 732 |
| OpenZFS mirror（vdev 是分区） | 126 582 | 666 666 | 88 495 | 1 640 ⚠60% |
| singlefs（两盘） | — | — | — | — |

#### 小文件持久化：新建 + 写 3000 字节 + fsync 文件 + fsync 目录（刚格式化的空文件系统上 100 次）

| 配置 | 延迟中位 µs | 延迟 p99 µs | 每次写请求 | 每次写字节 | 每次 FLUSH | 第一次写字节 | 第一次写请求 | 第一次 FLUSH |
|---|---|---|---|---|---|---|---|---|
| ext4（md raid1） | 1 157 | 1 284 ⚠15% | 8.00 | 72 581 | 4.00 | 73 728 | 8.00 | 4.00 |
| XFS（md raid1） | 1 079 | 1 271 | 6.00 | 19 927 | 4.00 | 13 312 | 6.00 | 4.00 |
| F2FS（md raid1） | 1 048 | 1 138 | 12.0 | 41 123 | 6.00 | 40 960 | 12.0 | 6.00 |
| Btrfs raid1 | 2 555 | 2 868 ⚠16% | 14.0 | 201 195 | 8.00 | 155 648 | 14.0 | 8.00 |
| Bcachefs 双副本 | 1 616 | 2 437 ⚠21% | 6.30 | 14 602 | 4.00 | 36 864 | 14.0 | 4.00 |
| OpenZFS mirror（vdev 是分区） | 941 | 1 042 ⚠46% | 4.58 | 9 308 | 2.03 | 31 744 ⚠229% | 18.0 ⚠133% | 2.00 |
| singlefs（两盘） | 19 075 | — | 36.0 | 475 648 | 7.00 | — | — | — |

#### 冷挂载（卸载、清缓存之后再挂）

| 配置 | 挂钟 ms | 读字节 | 读请求 |
|---|---|---|---|
| ext4（md raid1） | 1.38 | 21 504 | 6.00 |
| XFS（md raid1） | 8.07 ⚠61% | 4 235 776 | 45.0 |
| F2FS（md raid1） | 2.90 ⚠17% | 692 224 | 14.0 |
| Btrfs raid1 | 3.90 | 315 392 | 23.0 |
| Bcachefs 双副本 | 28.5 | 7 718 912 ⚠17% | 57.0 |
| OpenZFS mirror（vdev 是分区） | 6 110 | 11 534 336 ⚠17% | 240 |
| singlefs（两盘） | 9 914 | 1 612 087 296 | 393 628 |

#### 格式化与可用容量（16 GiB 盘）

| 配置 | 格式化 ms | 格式化写字节 | 格式化 FLUSH | 总容量 % | 可用 % |
|---|---|---|---|---|---|
| ext4（md raid1） | 90.4 ⚠31% | 143 197 184 | 14.0 | 48.8 | 46.3 |
| XFS（md raid1） | 82.9 | 135 160 832 | 10.0 | 49.8 | 48.7 |
| F2FS（md raid1） | 63.0 ⚠53% | 79 815 680 | 8.00 | 49.9 | 49.0 |
| Btrfs raid1 | 20.0 | 9 216 000 | 23.0 | 50.0 | 49.2 |
| Bcachefs 双副本 | 28.4 ⚠53% | 32 768 | 2.00 | 92.0 | 90.6 |
| OpenZFS mirror（vdev 是分区） | 544 | 3 574 272 ⚠111% | 26.0 | 46.9 | 46.9 |
| singlefs（两盘） | — | — | — | — | — |

#### singlefs（两块 16 GiB 盘，门禁 qemu-device-streams 的真设备二进制）

| 配置 | mkfs + 取号 + 暖机 + 新池新建文件 ms | 覆盖写、释放、回退与复用（覆盖写 B）ms | 发布 B（二进制自己计）ms | 冷恢复 ms | 两盘读字节 | 两盘读请求 | B 两盘写请求 | B 两盘写字节 | B 两盘屏障 | B 两盘 FUA 写 |
|---|---|---|---|---|---|---|---|---|---|---|
| singlefs（两盘） | 348 ⚠83% | 19.3 | 19.1 | 9 914 | 1 612 087 296 | 393 628 | 29.0 | 475 648 | 6.00 | 1.00 |

```

**出处 `.claude/kb/experiments/152-按里程碑对比六家文件系统的文件性能.md:236-321`（整段抄，未转述）**

```markdown
### 2026-09-28：两盘镜像七家 + singlefs

里程碑二收尾第 8 步，40 次虚机跑全部 `vm_exit=0`、没有重试。singlefs 第一段中位 1 874 ms（mkfs 真写整条 journal 环，每盘 805 959 680 字节）、发布 B 29 次写 6 道屏障中位 19.4 ms、冷恢复中位 10.0 s（第 3 轮 72.6 s 离群）；与 2026-09-17 那次 singlefs 一臂的逐行比在 `scripts/experiment-apparatus/perf-by-milestone.md` 三·三。

六个维度的表与 singlefs 那一张（`python3 scripts/experiment-apparatus/e152-tables.py e152-file-system-benchmark-mirror-2026-09-28.out`，整份原样取前七张；块层放大倍数与对照两张在同一份输出里）：

#### 大文件顺序写与顺序读（MiB/s，越大越好）

| 配置 | 顺序写 | 顺序读 |
|---|---|---|
| 裸 md raid1（镜像天花板） | 2 819 | 5 265 |
| ext4（md raid1） | 2 713 ⚠91% | 5 211 ⚠63% |
| XFS（md raid1） | 2 614 ⚠94% | 5 191 |
| F2FS（md raid1） | 2 707 ⚠89% | 5 265 |
| Btrfs raid1 | 1 375 ⚠94% | 4 905 |
| Bcachefs 双副本 | 1 208 ⚠143% | 4 824 |
| OpenZFS mirror（vdev 是分区） | 421 ⚠110% | 1 934 |
| singlefs（两盘） | — | — |

#### 4K 随机读写（次/秒越大越好；p99 是完成延迟，µs，越小越好）

| 配置 | 随机读 次/秒 | 随机读 p99 | 随机写 次/秒 | 随机写 p99 |
|---|---|---|---|---|
| 裸 md raid1（镜像天花板） | 64 564 | 610 | 37 037 ⚠26% | 897 |
| ext4（md raid1） | 59 591 ⚠29% | 643 ⚠764% | 35 156 ⚠69% | 1 056 ⚠462% |
| XFS（md raid1） | 59 019 | 561 | 34 959 | 1 044 |
| F2FS（md raid1） | 60 200 | 643 | 35 301 | 1 056 |
| Btrfs raid1 | 50 472 | 659 | 11 127 ⚠20% | 3 194 |
| Bcachefs 双副本 | 31 571 | 1 400 | 23 586 ⚠119% | 1 810 ⚠6430% |
| OpenZFS mirror（vdev 是分区） | 2 934 | 11 337 | 4 153 ⚠113% | 28 442 ⚠1013% |
| singlefs（两盘） | — | — | — | — |

#### 元数据 / 小文件（个/秒，越大越好）

| 配置 | 建空文件 | stat | 删文件 | 4 KiB 写 + fsync |
|---|---|---|---|---|
| ext4（md raid1） | 243 902 | 909 090 | 303 030 | 815 ⚠79% |
| XFS（md raid1） | 227 272 | 909 090 | 188 679 | 903 |
| F2FS（md raid1） | 185 185 ⚠18% | 909 090 | 256 410 | 1 212 |
| Btrfs raid1 | 238 095 | 909 090 | 128 205 | 680 ⚠66% |
| Bcachefs 双副本 | 138 888 ⚠48% | 909 090 | 113 636 ⚠22% | 723 ⚠66% |
| OpenZFS mirror（vdev 是分区） | 128 205 | 666 666 | 87 719 | 1 472 ⚠92% |
| singlefs（两盘） | — | — | — | — |

#### 小文件持久化：新建 + 写 3000 字节 + fsync 文件 + fsync 目录（刚格式化的空文件系统上 100 次）

| 配置 | 延迟中位 µs | 延迟 p99 µs | 每次写请求 | 每次写字节 | 每次 FLUSH | 第一次写字节 | 第一次写请求 | 第一次 FLUSH |
|---|---|---|---|---|---|---|---|---|
| ext4（md raid1） | 1 152 | 1 309 ⚠21% | 8.00 | 72 581 | 4.00 | 73 728 | 8.00 | 4.00 |
| XFS（md raid1） | 1 072 | 1 226 | 6.00 | 19 927 | 4.00 | 13 312 | 6.00 | 4.00 |
| F2FS（md raid1） | 1 062 | 1 172 ⚠25% | 12.0 | 41 123 | 6.00 | 40 960 | 12.0 | 6.00 |
| Btrfs raid1 | 2 574 | 2 791 | 14.0 | 201 195 | 8.00 | 155 648 | 14.0 | 8.00 |
| Bcachefs 双副本 | 1 688 | 2 402 | 6.30 | 14 632 | 4.00 | 36 864 | 14.0 | 4.00 |
| OpenZFS mirror（vdev 是分区） | 921 | 1 185 ⚠26% | 4.00 | 8 192 | 2.00 | 8 192 ⚠288% | 4.00 ⚠350% | 2.00 |
| singlefs（两盘） | 19 233 ⚠198% | — | 36.0 | 475 648 | 7.00 | — | — | — |

#### 冷挂载（卸载、清缓存之后再挂）

| 配置 | 挂钟 ms | 读字节 | 读请求 |
|---|---|---|---|
| ext4（md raid1） | 1.39 | 21 504 | 6.00 |
| XFS（md raid1） | 10.5 ⚠46% | 4 235 776 | 45.0 |
| F2FS（md raid1） | 3.08 ⚠16% | 692 224 | 14.0 |
| Btrfs raid1 | 4.23 ⚠165% | 315 392 | 23.0 |
| Bcachefs 双副本 | 31.2 | 7 718 912 ⚠31% | 57.0 ⚠16% |
| OpenZFS mirror（vdev 是分区） | 6 107 | 11 207 680 ⚠21% | 245 |
| singlefs（两盘） | 10 017 ⚠626% | 1 612 087 296 | 393 628 |

#### 格式化与可用容量（16 GiB 盘）

| 配置 | 格式化 ms | 格式化写字节 | 格式化 FLUSH | 总容量 % | 可用 % |
|---|---|---|---|---|---|
| ext4（md raid1） | 90.0 ⚠22% | 143 197 184 | 14.0 | 48.8 | 46.3 |
| XFS（md raid1） | 86.7 | 135 160 832 | 10.0 | 49.8 | 48.7 |
| F2FS（md raid1） | 79.1 ⚠35% | 79 815 680 | 8.00 | 49.9 | 49.0 |
| Btrfs raid1 | 21.2 | 9 216 000 | 23.0 | 50.0 | 49.2 |
| Bcachefs 双副本 | 40.9 ⚠31% | 32 768 | 2.00 | 92.0 | 90.6 |
| OpenZFS mirror（vdev 是分区） | 543 | 3 586 560 | 26.0 | 46.9 | 46.9 |
| singlefs（两盘） | — | — | — | — | — |

#### singlefs（两块 16 GiB 盘，门禁 qemu-device-streams 的真设备二进制）

| 配置 | mkfs + 取号 + 暖机 + 新池新建文件 ms | 覆盖写、释放、回退与复用（覆盖写 B）ms | 发布 B（二进制自己计）ms | 冷恢复 ms | 两盘读字节 | 两盘读请求 | B 两盘写请求 | B 两盘写字节 | B 两盘屏障 | B 两盘 FUA 写 |
|---|---|---|---|---|---|---|---|---|---|---|
| singlefs（两盘） | 1 874 ⚠230% | 19.4 ⚠197% | 19.2 ⚠198% | 10 017 ⚠626% | 1 612 087 296 | 393 628 | 29.0 | 475 648 | 6.00 | 1.00 |

```

**出处 `.claude/kb/experiments/152-按里程碑对比六家文件系统的文件性能.md:322-364`（整段抄，未转述）**

````markdown
### 2026-09-19：装置改为先读完再转打之后的 singlefs 一臂（整行抄自产物（产物已移出工作区的行写成一行指针））

起，重跑登记 `e152-r5-prereg.md`，只答问题单 `e152-r5-questions.md` 的 Q1–Q4，不加 `nice -n 19`（登记第五节：这一跑量的是挂钟）。5 轮一次过（`vm_exit=0`），宿主 1 分钟负载 6.88→4.15（单调降）。产物 `e152-file-system-benchmark-second-transaction-relay-timing-2026-09-19.out`（142 行，末行 `emitted=26`）：

```text
E7RESULT name=summary configuration=singlefs metric=singlefs_second_transaction_inner_milliseconds rounds=2 median=6.576 minimum=6.360 maximum=6.792 spread_percent=6.6 stability=stable values=4:6.792,5:6.360
E7RESULT name=summary configuration=singlefs metric=singlefs_second_transaction_milliseconds rounds=2 median=6.629 minimum=6.409 maximum=6.850 spread_percent=6.7 stability=stable values=4:6.850,5:6.409
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-second-transaction-relay-timing-2026-09-19.out 第 139 行（name=summary）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-second-transaction-relay-timing-2026-09-19.out 第 140 行（name=summary）
E7RESULT name=summary configuration=singlefs metric=singlefs_write_path_milliseconds rounds=5 median=19.071 minimum=18.642 maximum=19.867 spread_percent=6.4 stability=stable values=1:19.019,2:19.071,3:19.218,4:19.867,5:18.642
```

**Q1（外层包不包住里层）不够按「5 轮都取到确定值」的路径判，按「过半不确定」的路径够判**：第 1–3 轮的 `singlefs_timing` 行本身被虚机关机时内核打印的 `[    8.423276] reboot: Power down`（3 轮各一次，时刻不同）从中间截断（整行抄，第 1 轮）：

```text
E7RESULT name=singlefs_timing configuration=singlefs round=1 write_path_nanoseconds=19018861 second_transaction_nanoseconds=6424304 recovery_nanoseconds=7797794145 second_transaction_writes_both_devices=21 second_transaction_written_bytes_both_devices=344576 second_transaction_barriers_both_devices=4 second_transaction_[    8.423276] reboot: Power down
```

`file_overwrite_outer_contains_inner` 字段落在截断点之后，字段不存在（登记第一节 Q1 读法：字段不存在算不确定）；第 4、5 轮该行完整，两轮都是 `true`。5 轮里 3 轮不确定、0 轮 false、2 轮 true，过半（3/5）不确定 ⇒ 按登记第十四节，外层「覆盖写、释放、回退与复用 ms」这一格整格不报（汇总行 `rounds=2`，只收 4、5 轮，与上面第一段引文一致）。用 A3（同一轮 `inner=file_overwrite` 行的 `at_nanoseconds` 与 `nanoseconds=` 独立复算）逐轮核，5 轮外层减里层分别是 52 540、65 020、65 211、57 621、48 330 ns，全部为正——**物理上包含关系在 5 轮里都成立，只是第 1–3 轮那一个字段被截断读不到，不是关系不成立**；这一条不进 Q1 的正式答案（登记的读法只认 `singlefs_timing` 行原样字段），列在这里做区分。

**这处截断之前没有记录**：`.claude/kb/vm-harness.md`「计时不在转发输出的循环里打时间戳」管的是转打阻塞导致的时间戳积压，装置改了读法（先读到 EOF 再转打）之后那个来源确实没有了（`E152RUN` 时刻与 `singlefs_inner` 行本身没有再出现几十毫秒级的积压，见 Q4）；这次截断是另一个来源：guest 内核在虚机关机（`reboot()`）那一刻自己往同一个串口写 `dmesg`，与用户态最后一次 `println!` 竞争同一个 tty，物理上把字节交错打断——**读法分开这一步没有、也管不了这个来源**。3/5 出现、2/5 没出现，样本太小说不清与宿主负载的关系（出现的三轮负载 6.88/5.98/5.21，没出现的两轮 4.64/4.15，同一次跑单调降，不能排除只是「越晚跑负载越低」的巧合）。

**Q2 不够判**：够判条件是 `singlefs_file_overwrite_inner_milliseconds` 的汇总行 `rounds=5`，实际 `rounds=2`（同一个截断，`file_overwrite_inner_nanoseconds` 字段也在截断点之后）。只能报这 2 轮：中位 6.576 ms、离散 6.6%（稳定），不能代表 5 轮。

**Q3 够判（问题单的够判条件只要求 `name=file_overwrite` 与 `name=recover_cold` 两行都在，这两行不受截断影响，来自子进程原始输出）**：c1（段序列）、c7（根 1:4）、c8（`content_matches=true`）、c9（逐盘八个字段）5 轮与2026-09-17 两次正式跑的 10 轮逐字相同；c3–c5（写请求 21、字节 344576、屏障 4）5 轮都在截断点之前，同样相同；c6（FUA 写）仅第 4、5 轮可读，值 1，与旧产物相同，第 1–3 轮因截断缺字段。**c2（操作数）5 轮均为 23，不是登记第十二节引文与第六节 A-2 写的锚点 21**——这不是这一次新出现的：2026-09-17 两次正式跑的旧产物里操作数同样是 23（`e152-file-system-benchmark-second-transaction-2026-09-17.out`、`…-final-2026-09-17.out` 逐轮皆 23），「这几个数说明什么」一节已有过解释（「操作数 23（21 次写加 2 次屏障计数的口径见二进制）」），`git log --since='2026-09-17' -- crates/singlefs-core/src/transaction.rs crates/singlefs-core/src/allocator.rs` 有一条改动（`cae5092`），但操作数 23 在这条改动之前的 2026-09-17 收尾那次产物里已经存在，不是它带来的回归。锚点 21 与实现 23 的差异交主 agent 定要不要改登记的锚点。

**Q4 够判、5 轮全部满足**：d1（第一条 `segments` 到 `transaction` 行的跨度，不受截断影响）5 轮分别为 81 240、78 121、86 351、86 810、80 301 ns，全部 ≤ 1 ms；d2（行数）5 轮均为 13，与登记 B-3 的新锚点一致（旧产物是 8 行，源码在两次跑之间新增了 `instance_acquisition`/`post_mkfs_stream` 等 segments 行与两条 `device_calls`，登记已预判并接受的 13）；d3（相邻最大间隔）5 轮 21 200–23 540 ns，与十四节冒烟跑的「24 µs」量级一致。

**P1（旧产物复算，阳性对照）**：对2026-09-17 两次正式跑共 10 轮用同一套判定复算，`file_overwrite_outer_contains_inner` 为 false 的轮数 = 9（与 `.claude/kb/vm-harness.md` 已记的「9/10」逐轮复现，`.claude/kb/vm-harness.md` 那条数没有错）；d1 > 1 ms 的轮数 = 8/10（≥ 6 门槛），两条判定在旧产物上都判「有牙」，说明这次装置改动之后 Q1、Q4 在 5/5、5/5 轮上过关不是判定本身没有分辨力。

**P2（把转打放回读的循环，阳性对照）不可行，0/6 次尝试成功**：按登记第五节把 `read_lines_with_arrival_times` 循环里加一行 `println!("relay_control {text}")`（登记设想它「行首不是 E7RESULT，vm-bench.sh 不收它」），3 轮 × 2 次尝试全部被 `scripts/devices/vm-bench.sh` 的完整性闸判「结果条数对不上：程序声称发了 22 条，宿主只抓到 41 条」而作废（`vm_exit=1`）。根因：`vm-bench.sh` 里 `grep -ao 'E7RESULT .*'` 那一行不锚定行首（该行自己的注释也写着「本脚本抓 E7RESULT 已经不锚定行首」），子进程原始行本身以 `E7RESULT ` 开头，`relay_control E7RESULT name=... ` 这行仍会被从 `E7RESULT` 处截出来当成一条独立结果，与稍后 wrapper 正式转打的同一行重复计数——登记对 vm-bench.sh 抓取方式的假设与实现不符，产物 `e152-file-system-benchmark-second-transaction-relay-in-loop-control-2026-09-19.out`（257 行，全部作废）留档，不产出 e3/e4 的数据。已把注入行改回原样、`sha256sum` 核对与登记第三节一致。

**G1（几何敏感性，`VM_CPUS=1`，3 轮）不稳定，与登记的方向相反预期一致**：产物 `e152-file-system-benchmark-second-transaction-relay-timing-vcpu1-2026-09-19.out`（83 行，末行 `emitted=13`）：

```text
E7RESULT name=summary configuration=singlefs metric=singlefs_second_transaction_inner_milliseconds rounds=3 median=6.260 minimum=6.132 maximum=6.643 spread_percent=8.2 stability=stable values=1:6.132,2:6.260,3:6.643
E7RESULT name=summary configuration=singlefs metric=singlefs_second_transaction_milliseconds rounds=1 median=6.302 minimum=6.302 maximum=6.302 spread_percent=0.0 stability=stable values=2:6.302
```

3 轮里 `file_overwrite_outer_contains_inner` 是 false、true、false（第 1、3 轮为 false，`file_overwrite_nanoseconds` 比 `file_overwrite_inner_nanoseconds` 分别少 636 471 ns、458 921 ns）——单 vCPU 下，读子进程输出的宿主进程与子进程抢同一个核，即使读法已经改成先读完再转打，`transaction` 行与 `file_overwrite` 行各自的到达时刻仍可能因为调度延迟而颠倒，3 轮里 2 轮触发。d1（第一段跨度）3 轮为 726 152、70 590、559 782 ns，仍全部 ≤ 1 ms（比 4 vCPU 时大一个数量级，但没有超阈值）。按登记第八节判定：**A1 在两个几何点上不稳定（4 vCPU 因截断只 2/5 轮确定但复算全部为真，1 vCPU 有 2/3 轮确定为 false）**，Q1「读与转打分开之后外层包住里层」这句结论要限定在「消除了转打阻塞这一个来源」，不能扩大到「任何几何下都包住」；d1 在两个点上都没有触发（≤1 ms），Q4 的判定在这次跑到的两个几何点上没有分出「触发/不触发」两种，暂不能判定它对更极端的调度延迟有没有牙。

跑前登记第十一节「够判停机」判：Q1、Q3、Q4 三行第一段交回时已够判（Q1 走「过半不确定」路径），Q2 不够判（`rounds=2 < 5`）；第二段 P2、G1 均已跑（P2 因方法本身不可行未产出数据，G1 产出数据但确认了不稳定）。

````

**出处 `.claude/kb/experiments/152-按里程碑对比六家文件系统的文件性能.md:365-400`（整段抄，未转述）**

````markdown
### 2026-09-17 收尾：里程碑「覆盖写、释放、回退与复用」收尾时再跑 singlefs 一臂（整行抄自产物（产物已移出工作区的行写成一行指针））

起，用户在里程碑收尾时要求再跑一轮；步 7 那次跑之后 `crates/` 改了四处（跑前登记第十三节逐条列），负载与判据不变。5 轮一次过，宿主 1 分钟负载 2.27–2.45（门禁 crash-point-replay 的崩溃点重放全量同时在跑，`nice -n 19`）。产物 `e152-file-system-benchmark-second-transaction-final-2026-09-17.out`（91 行，末行 `emitted=10`）。

```text
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-second-transaction-final-2026-09-17.out 第 82 行（name=summary）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-second-transaction-final-2026-09-17.out 第 83 行（name=summary）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-second-transaction-final-2026-09-17.out 第 84 行（name=summary）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-second-transaction-relay-timing-2026-09-19.out 第 135 行（name=summary）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-second-transaction-final-2026-09-17.out 第 86 行（name=summary）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-second-transaction-final-2026-09-17.out 第 87 行（name=summary）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-second-transaction-relay-timing-2026-09-19.out 第 139 行（name=summary）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-second-transaction-relay-timing-2026-09-19.out 第 140 行（name=summary）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-second-transaction-final-2026-09-17.out 第 90 行（name=summary）
```

跑前判据（第十三节）逐条：5 轮 B 的段序列 `16+2+1+2`、两盘合计写请求 21、写字节 344 576、屏障 4、FUA 1，与第三次逐字相同；冷恢复 5 轮 `content_matches=true`、根 1:4。挂钟两次并列在 `scripts/experiment-apparatus/perf-by-milestone.md` 第三·二节「收尾时再跑一次」，不比快慢。

**步 7：里程碑「覆盖写、释放、回退与复用」的 singlefs 一臂（整行抄自产物（产物已移出工作区的行写成一行指针））**

起，只跑 singlefs（两块 16 GiB 盘，`file-overwrite` 模式：新池新建文件之后同一个进程里再覆盖写一次），5 轮一次过，宿主 1 分钟负载 1.28–1.69（同一台机器上另有一份全量门禁在跑 research-build-and-replay 的 experiment-replay 格复跑，跑前登记第七节：照跑照记）。配置与判据在跑前登记第十二节；六家的基线不重跑。产物 `e152-file-system-benchmark-second-transaction-2026-09-17.out`（91 行，末行 `emitted=10`）。

```text
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-second-transaction-final-2026-09-17.out 第 82 行（name=summary）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-second-transaction-final-2026-09-17.out 第 83 行（name=summary）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-second-transaction-2026-09-17.out 第 84 行（name=summary）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-second-transaction-relay-timing-2026-09-19.out 第 135 行（name=summary）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-second-transaction-final-2026-09-17.out 第 86 行（name=summary）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-second-transaction-2026-09-17.out 第 87 行（name=summary）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-second-transaction-relay-timing-2026-09-19.out 第 139 行（name=summary）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-second-transaction-relay-timing-2026-09-19.out 第 140 行（name=summary）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-second-transaction-2026-09-17.out 第 90 行（name=summary）
```

跑前判据（第十二节）逐条：B 的段序列 5 轮都是 `16+2+1+2`、操作数 23（21 次写加 2 次屏障计数的口径见二进制）、两盘合计写请求 21、写字节 344 576、屏障 4、FUA 1，与登记的数相同；冷恢复 5 轮 `content_matches=true`、根 1:4。覆盖写、释放、回退与复用的挂钟离散 186%、第一段 156%，登记里写明照报不删。这几个数说明什么写在「这几个数说明什么（覆盖写、释放、回退与复用，与两盘镜像的六家比）」一节，给人读的版本在 `scripts/experiment-apparatus/perf-by-milestone.md` 第三·二节。

````

**出处 `.claude/kb/experiments/152-按里程碑对比六家文件系统的文件性能.md:401-469`（整段抄，未转述）**

````markdown
### 这几个数说明什么（覆盖写、释放、回退与复用，与两盘镜像的六家比）

2026-09-17 写。singlefs 一格取2026-09-17 两次正式跑的发布 B（同一个进程里对已有文件覆盖写一次，4100 字节的第二版）；六家一格取2026-09-15 两盘镜像那次正式跑（2026-09-15，两盘镜像，刚格式化的空文件系统上 100 次「新建 + 写 3000 字节 + fsync 文件 + fsync 目录」），六家没有重跑。这里只有观测与算术；差距该怎么缩是设计推论，没走三方论证。

**1. 七家排位**（六家一格：写字节、写请求、FLUSH 是 100 次的平均，挂钟是 100 次的中位；singlefs 一格：两次跑 10 轮逐字相同的计数，挂钟的取法在第 4 条）

| 维 | singlefs 发布 B | 镜像六家：最轻或最快 … 最重或最慢 | singlefs 在七家里 | 是最轻那家的几倍 | 是最重那家的几倍 |
|---|---|---|---|---|---|
| 每次写字节（两盘合计） | 344 576 | OpenZFS mirror 8 192 … Btrfs raid1 201 195 | 最多 | 42.1 | 1.71 |
| 每次写请求（两盘合计，来宾块层记法） | 26（二进制记 21 次写调用，第 5 条） | OpenZFS mirror 4.00 … Btrfs raid1 14.0 | 最多 | 6.5 | 1.86 |
| 每次 FLUSH（两盘合计） | 5 | OpenZFS mirror 2.00 … Btrfs raid1 8.00 | 居中：少于 F2FS（md raid1）6.00 与 Btrfs raid1，多于另外四家 | 2.5 | 0.63 |
| 每次挂钟 | 2026-09-17 收尾那次 5 轮中位 6.63 ms（离散 11.8%）；两次跑 10 次 5.95–12.61 ms | OpenZFS mirror 850 µs … Btrfs raid1 2 199 µs | 最慢：10 次里最快的 5.95 ms 是六家 p99 里最慢那一格（Bcachefs 双副本 2 559 µs）的 2.3 倍 | 7.8 | 3.0 |
| 冷挂载挂钟 | 7 893 / 7 738 ms（2026-09-17 步 7 那次 / 收尾那次的中位） | ext4（md raid1）1.66 ms … OpenZFS mirror 6 125 ms | 最慢 | 约 4 700 | 1.26–1.29 |
| 冷挂载读字节（两盘合计） | 1 610 981 376 | ext4（md raid1）21 504 … OpenZFS mirror 11 878 400 | 最多 | 约 75 000 | 136 |
| 能跑的维 | 八维里三维（小文件持久化、冷挂载、格式化与容量） | 八维 | — | — | — |

**2. 差距在哪几格、多大（判断，依据是第 1 条的表）**

- 小文件持久化在里程碑「覆盖写、释放、回退与复用」上第一次有稳态的数：写字节、写请求、挂钟三项七家最多或最慢，连六家里最重、最慢的一家也比不过（1.71、1.86、3.0 倍）；FLUSH 居中，不是差距所在。
- 写得多不是「所有树第一次落盘」的代价：发布 B 与新池新建文件写的字节、写调用、屏障、FUA 逐个相同（两者的段序列都是 16+2+1+2），singlefs 每次持久化的代价与是不是第一次无关。
- 冷挂载是绝对量最大的差距：七家最慢、秒级。两盘读的 1 610 981 376 字节里，按 `scan_journal` 逐槽读整环的是 1 610 612 736（99.98%），这一部分只随环的大小变；环以外的读随盘上内容变，覆盖写、释放、回退与复用比新池新建文件多 5 次请求、49 152 字节。这个拆法来自代码：块层分不出哪次请求落在环上，读字节也不等于读请求数 × 4 KiB（393 302 × 4 096 比实测少 16 384）。按请求摊（冷恢复挂钟 ÷ 读请求数，含读以外的时间），singlefs 每次 19.7–20.1 µs、共 393 302 次；OpenZFS mirror 只读 260 次、每次摊 23.6 ms——七家里最慢的两家，一家读的次数多，一家读得少而每次摊得长。
- 六家里 Btrfs、Bcachefs、OpenZFS 也是写时复制（外部事实，未在本项目验证），同一台测试台上后两家每次只写 14 592 与 8 192 字节；由此推：写时复制这个做法本身不要求每次 fsync 写几十万字节（推论，没走三方论证）。它们这 8–15 KiB 写的是什么，E152（按里程碑对比六家文件系统的文件性能） 到 2026-09-17 为止没量。

**3. 差距从哪来（算术）**：发布 B 换下第一个文件版本的八个单元（[layout/02-second-txn.md](../layout/02-second-txn.md) 「已释放的分配记录」那一行），新写的八个与新池新建文件那八个同种同宽（[layout/01-first-txn.md](../layout/01-first-txn.md) 的单元表：数据单元、extent 树根兼叶、inode 树叶容器、inode 树根、分配记录树根兼叶、记账树根兼叶、中央映射树根兼叶、树表单元），两盘各一份：

| 部分 | 两盘合计字节 | 占发布 B | 单独拿出来与六家整次比 |
|---|---|---|---|
| 数据单元 1 个（4100 字节的内容；数据单元占盘恒 32 768 含头，D4（校验和位置） 已定项 5） | 65 536 | 19.0% | 已比 OpenZFS mirror、Bcachefs 双副本、XFS（md raid1）、F2FS（md raid1）四家的整次重：是 OpenZFS mirror 的 8.0 倍、XFS（md raid1）的 3.3 倍 |
| 元数据单元 7 个（inode 树叶容器 32 768，其余六个各 16 384） | 262 144 | 76.1% | 比六家里最重的 Btrfs raid1 整次还重，是它的 1.30 倍 |
| journal 记录 2 个、根槽 1 个、系统配置槽 2 个 | 16 896 | 4.9% | 比 XFS（md raid1）整次的 19 927 还少，不是差距所在 |

- 两块各自就足够构成差距：只算数据单元，已比六家里四家整次重；只算元数据单元，已比六家全部整次重。
- 写请求同理：8 个单元 × 2 盘的 16 次写，已多于六家里最多的 Btrfs raid1 每次 14.0 次。
- 2026-09-17 那两次跑里每棵树只有一个节点，一次覆盖写碰到的七棵树各重写一个单元；树长高、文件变多之后每次写多少，2026-09-17 两次跑量不到。

**4. 计时的口径：汇总行里覆盖写、释放、回退与复用那一格不是发布 B 的时间**（装置的问题，不是设计、不是数据）

- 汇总行 `singlefs_file_overwrite_milliseconds`（2026-09-17 步 7 那次中位 5.078、收尾那次 5.529）按跑前登记第十二节第 2 条，取外层包装程序给 `name=transaction` 与 `name=file_overwrite` 两行打的时间戳之差。外层（`scripts/experiment-apparatus/e152_file_system_benchmark.rs` 的 `run_singlefs`）每读到子进程的一行，先把它转打到自己的标准输出，才读下一行、打下一个时间戳，而子进程往管道里写不被挡；所以一行的时间戳里带着它前面几行的打印积压。
- 反推：二进制先打 `name=transaction` 行，再开始计时、做发布 B、停表，最后打 `name=file_overwrite` 行，外层两行的真实间隔必然包住二进制自己计的那一段。两次跑 10 轮里有 9 轮外层数比二进制自己计的小。
- 积压有多大：二进制连着打 `segments`、`device_calls`、`transaction` 这几行，中间只有几微秒的内存计算；2026-09-17 收尾那次第 2 轮这几行的时间戳跨了 21.7 ms（整行抄自 `e152-file-system-benchmark-second-transaction-final-2026-09-17.out` 里第 2 轮 `inner=segments path=mkfs`、两行 `inner=device_calls`、`inner=transaction`、`inner=second_transaction` 与 `name=singlefs_timing` 那六行）：

```text
E7RESULT name=singlefs_inner configuration=singlefs round=2 at_nanoseconds=19555763 inner=segments path=mkfs operations=13 segments=4+1+1+1+4 closed_form=34 kinds=[unit_write×4,barrier]|[root_record_fua]|[root_record_fua]|[root_record_fua]|[superblock_slot×4,barrier]
E7RESULT name=singlefs_inner configuration=singlefs round=2 at_nanoseconds=35650082 inner=device_calls device=0 path=/dev/vda writes=23 written_bytes=251904 force_unit_access_writes=4 barriers=8 skipped_barriers=0 write_ios=35 write_sectors=492 flush_ios=12
E7RESULT name=singlefs_inner configuration=singlefs round=2 at_nanoseconds=38414092 inner=device_calls device=1 path=/dev/vdb writes=21 written_bytes=250880 force_unit_access_writes=2 barriers=8 skipped_barriers=0 write_ios=31 write_sectors=490 flush_ios=10
E7RESULT name=singlefs_inner configuration=singlefs round=2 at_nanoseconds=41239814 inner=transaction policy_mismatches=0 key_order_mismatches=0 root_txg=3 back_chain=628216162
E7RESULT name=singlefs_inner configuration=singlefs round=2 at_nanoseconds=43231321 inner=second_transaction root_txg=4 transaction=2 released=8 nanoseconds=6608505 operations=23 segments=16+2+1+2 closed_form=65543 device_0_writes=10 device_0_written_bytes=172032 device_0_force_unit_access_writes=0 device_0_barriers=2 device_1_writes=11 device_1_written_bytes=172544 device_1_force_unit_access_writes=1 device_1_barriers=2
E7RESULT name=singlefs_timing configuration=singlefs round=2 write_path_nanoseconds=41040214 second_transaction_nanoseconds=1991507 recovery_nanoseconds=7738420894 second_transaction_writes_both_devices=21 second_transaction_written_bytes_both_devices=344576 second_transaction_barriers_both_devices=4 second_transaction_force_unit_access_writes_both_devices=1 child_exit=0 content_matches=true device_0_read_bytes=805609472 device_0_read_requests=196664 device_0_write_bytes=423936 device_0_write_requests=47 device_0_flush_requests=14 device_1_read_bytes=805371904 device_1_read_requests=196638 device_1_write_bytes=423424 device_1_write_requests=45 device_1_flush_requests=13
```

- 同一个积压解释了第一段挂钟的「两群」：四次跑 20 轮里，从第一行 `segments` 到 `transaction` 行（中间只有打印）占 0.71–25.83 ms；40 ms 上下的那 5 轮（2026-09-15 单盘那次第 1、3 轮，2026-09-17 步 7 那次第 2 轮，收尾那次第 2、3 轮）占 21.68–25.83 ms。冷恢复那一段 7.7–7.9 s，这个量级的积压影响不到 0.4%。
- 第 1 条的挂钟因此取二进制在 `name=file_overwrite` 行里自己报的 `nanoseconds=`（跑前登记第十二节第 1 条登记过的字段）：2026-09-17 步 7 那次 12.611 / 8.576 / 5.948 / 6.135 / 6.093 ms（中位 6.135，离散 109%），收尾那次 6.369 / 6.609 / 6.634 / 7.153 / 6.643 ms（中位 6.634，离散 11.8%）。⚠️ 这是看了产物之后换的取法，理由是外层数物理上不可能，不是它不合意；二进制自己计的数只有这一条路，没有第二条路核过。
- 装置已改（跑前登记第十四节）：来宾程序先把子进程输出读到 EOF 再转打，`singlefs_timing` 行末尾多报子进程自己计的纳秒（`file_overwrite_inner_nanoseconds`）与包含自检（`file_overwrite_outer_contains_inner`），汇总时外层不包住里层的轮报 `summary_excluded`、不进中位，新增指标 `singlefs_file_overwrite_inner_milliseconds`。改完冒烟跑一轮：那几行 9 行跨 24 µs，外层 6 632 915 ns 包住子进程自己计的 6 575 144 ns。已有四份产物不重算；这条坑怎么避、谁在拦写在 `.claude/kb/vm-harness.md`「计时不在转发输出的循环里打时间戳」。

**5. 写请求的口径：六家是块层数，汇总行里 singlefs 是二进制的写调用数**

- 六家的「每次写请求」是来宾 `/sys/block/*/stat` 写请求数那个字段在每次操作前后的差；汇总行 `singlefs_file_overwrite_writes_both_devices` 是二进制的写调用数（跑前登记第十二节第 1 条：程序计数，不另量）。两者不同口径。
- 块层差分（2026-09-17 两次跑 10 轮逐字相同）：整程块层写请求盘 0 / 盘 1 是 47 / 45（第 4 条抄录里 `singlefs_timing` 行的 `device_*_write_requests=`）；二进制在新池新建文件之后、发布 B 之前读的块层数是 35 / 31（同一抄录里两行 `device_calls` 的 `write_ios=`）；冷恢复不写（2026-09-15 两盘镜像那次正式跑的 direct 模式没有发布 B，整程块层数就是 35 / 31）。所以发布 B 在块层是 12 / 14 次写请求、172 032 / 172 544 字节、2 / 3 次 FLUSH；字节与 FLUSH 与二进制的计数逐个相同，写请求比二进制的 10 / 11 各多 2 / 3。
- 多出来的正好是屏障数加 FUA 写数，四组逐盘读数都对上：发布 B 盘 0 两道屏障（10 + 2 = 12）、盘 1 两道屏障加一次 FUA（11 + 3 = 14）；发布 B 之前盘 0 八道屏障加四次 FUA（23 + 12 = 35）、盘 1 八道屏障加两次 FUA（21 + 10 = 31）。内核为什么这样记没查；比法按同口径走，singlefs 取块层记法的 26。
- 新池新建文件的段序列与发布 B 同型，按同一记法也是 26；`scripts/experiment-apparatus/perf-by-milestone.md` 第三节拿 21 与六家比的倍数按 26 算。

**6. 负载不一样的地方**

| 不一样 | 对比受不受影响 |
|---|---|
| 六家新建文件并 fsync 目录；singlefs 覆盖写已有文件，没有目录 | singlefs 新建（新池新建文件，3000 字节）与覆盖写（发布 B，4100 字节）写的字节、写调用、屏障、FUA 逐个相同，换成新建差距不变；六家覆盖写一次写多少没量 |
| 六家是 100 次的平均或中位（含第一次）；singlefs 每轮 1 次 | 从汇总中位近似算，六家去掉第一次之后每次写字节的变化在 −1.5% 到 +0.3% |
| 六家是内核里的文件系统，挂钟从 create 到目录 fsync 返回；singlefs 是用户态库对裸盘做 O_DIRECT 写与 fdatasync，挂钟只罩发布 B，不含从分配记录重建分配器 | 两边罩的都是「发起到持久返回」；singlefs 的 6.6 ms 在编码与校验、写、刷新之间怎么分，没拆 |

````

**出处 `.claude/kb/experiments/152-按里程碑对比六家文件系统的文件性能.md:470-727`（整段抄，未转述）**

````markdown
### 2026-09-15 两盘镜像：七个镜像配置 + singlefs

六个维度的表与 singlefs 那一张（`python3 scripts/experiment-apparatus/e152-tables.py research/results/e152-file-system-benchmark-mirror-2026-09-15.out`，整份原样取前七张；块层放大倍数与对照两张在同一份输出里）。产物已归档，从版本库 `3cff909b^` 取：

#### 大文件顺序写与顺序读（MiB/s，越大越好）

| 配置 | 顺序写 | 顺序读 |
|---|---|---|
| 裸 md raid1（镜像天花板） | 2 707 ⚠18% | 5 404 |
| ext4（md raid1） | 2 725 | 5 404 |
| XFS（md raid1） | 2 684 | 5 368 |
| F2FS（md raid1） | 2 718 ⚠44% | 5 389 |
| Btrfs raid1 | 1 280 ⚠30% | 5 044 |
| Bcachefs 双副本 | 2 547 ⚠38% | 4 905 |
| OpenZFS mirror（vdev 是分区） | 500 ⚠41% | 1 982 |
| singlefs（两盘） | — | — |

#### 4K 随机读写（次/秒越大越好；p99 是完成延迟，µs，越小越好）

| 配置 | 随机读 次/秒 | 随机读 p99 | 随机写 次/秒 | 随机写 p99 |
|---|---|---|---|---|
| 裸 md raid1（镜像天花板） | 57 893 | 626 | 33 019 ⚠44% | 1 028 ⚠16% |
| ext4（md raid1） | 53 276 | 659 | 34 441 ⚠34% | 1 368 ⚠16% |
| XFS（md raid1） | 53 356 | 634 ⚠16% | 33 253 ⚠20% | 1 269 ⚠30% |
| F2FS（md raid1） | 55 985 | 593 ⚠33% | 31 928 ⚠23% | 1 089 ⚠33% |
| Btrfs raid1 | 48 047 | 823 ⚠25% | 10 948 ⚠25% | 3 686 |
| Bcachefs 双副本 | 31 599 | 1 286 | 33 924 | 1 843 ⚠16% |
| OpenZFS mirror（vdev 是分区） | 3 122 | 10 682 | 5 237 ⚠22% | 15 925 ⚠77% |
| singlefs（两盘） | — | — | — | — |

#### 元数据 / 小文件（个/秒，越大越好）

| 配置 | 建空文件 | stat | 删文件 | 4 KiB 写 + fsync |
|---|---|---|---|---|
| ext4（md raid1） | 208 333 ⚠17% | 833 333 ⚠49% | 263 157 ⚠19% | 1 002 ⚠23% |
| XFS（md raid1） | 217 391 | 909 090 ⚠69% | 185 185 | 1 061 |
| F2FS（md raid1） | 172 413 | 909 090 ⚠88% | 227 272 ⚠21% | 1 365 |
| Btrfs raid1 | 222 222 ⚠15% | 833 333 ⚠75% | 120 481 | 809 |
| Bcachefs 双副本 | 125 000 ⚠32% | 714 285 ⚠101% | 113 636 ⚠18% | 815 |
| OpenZFS mirror（vdev 是分区） | 111 111 ⚠22% | 666 666 ⚠48% | 77 519 ⚠22% | 1 719 ⚠38% |
| singlefs（两盘） | — | — | — | — |

#### 小文件持久化：新建 + 写 3000 字节 + fsync 文件 + fsync 目录（刚格式化的空文件系统上 100 次）

| 配置 | 延迟中位 µs | 延迟 p99 µs | 每次写请求 | 每次写字节 | 每次 FLUSH | 第一次写字节 | 第一次写请求 | 第一次 FLUSH |
|---|---|---|---|---|---|---|---|---|
| ext4（md raid1） | 980 | 1 120 | 8.00 | 72 581 | 4.00 | 73 728 | 8.00 | 4.00 |
| XFS（md raid1） | 858 | 1 087 ⚠19% | 6.00 | 19 927 | 4.00 | 13 312 | 6.00 | 4.00 |
| F2FS（md raid1） | 928 | 1 418 ⚠31% | 12.0 | 41 123 | 6.00 | 40 960 | 12.0 | 6.00 |
| Btrfs raid1 | 2 199 | 2 531 | 14.0 | 201 195 | 8.00 | 155 648 | 14.0 | 8.00 |
| Bcachefs 双副本 | 1 520 | 2 559 ⚠26% | 6.30 | 14 592 | 4.00 | 36 864 | 14.0 | 4.00 |
| OpenZFS mirror（vdev 是分区） | 850 | 1 121 ⚠29% | 4.00 | 8 192 | 2.00 | 8 192 ⚠288% | 4.00 ⚠350% | 2.00 |
| singlefs（两盘） | — | — | — | — | — | — | — | — |

#### 冷挂载（卸载、清缓存之后再挂）

| 配置 | 挂钟 ms | 读字节 | 读请求 |
|---|---|---|---|
| ext4（md raid1） | 1.66 ⚠22% | 21 504 | 6.00 |
| XFS（md raid1） | 7.29 ⚠42% | 4 235 776 | 45.0 |
| F2FS（md raid1） | 3.02 | 692 224 | 14.0 |
| Btrfs raid1 | 4.21 ⚠24% | 315 392 | 23.0 |
| Bcachefs 双副本 | 29.7 | 7 456 768 | 56.0 |
| OpenZFS mirror（vdev 是分区） | 6 125 | 11 878 400 | 260 ⚠18% |
| singlefs（两盘） | 8 293 | 1 610 932 224 | 393 297 |

#### 格式化与可用容量（16 GiB 盘）

| 配置 | 格式化 ms | 格式化写字节 | 格式化 FLUSH | 总容量 % | 可用 % |
|---|---|---|---|---|---|
| ext4（md raid1） | 79.3 ⚠29% | 143 197 184 | 14.0 | 48.8 | 46.3 |
| XFS（md raid1） | 81.0 ⚠41% | 135 160 832 | 10.0 | 49.8 | 48.7 |
| F2FS（md raid1） | 62.0 ⚠47% | 79 815 680 | 8.00 | 49.9 | 49.0 |
| Btrfs raid1 | 18.2 | 9 216 000 | 23.0 | 50.0 | 49.2 |
| Bcachefs 双副本 | 23.4 ⚠43% | 32 768 | 2.00 | 92.0 | 90.6 |
| OpenZFS mirror（vdev 是分区） | 546 | 3 586 048 | 26.0 | 46.9 | 46.9 |
| singlefs（两盘） | — | — | — | — | — |

#### singlefs（两块 16 GiB 盘，门禁 qemu-device-streams 的真设备二进制）

| 配置 | mkfs + 取号 + 暖机 + 新池新建文件 ms | 覆盖写、释放、回退与复用（覆盖写 B）ms | 发布 B（二进制自己计）ms | 冷恢复 ms | 两盘读字节 | 两盘读请求 | B 两盘写请求 | B 两盘写字节 | B 两盘屏障 | B 两盘 FUA 写 |
|---|---|---|---|---|---|---|---|---|---|---|
| singlefs（两盘） | 17.3 | — | — | 8 293 | 1 610 932 224 | 393 297 | — | — | — | — |

整行抄自产物（产物已移出工作区的行写成一行指针）：

2026-09-15 起，七个镜像配置加 singlefs 各两块 16 GiB 盘，按轮交错跑 5 轮（配置与口径在跑前登记第十一节）。41 次虚机跑里失败一次，是别的会话在跑的途中改了 vm-bench.sh，同参数重跑成功：

```text
E152RUN configuration=ext4-md round=1 attempt=1 host_load1=2.26 vm_exit=2
```

七个带对照的配置每一行都是 `rounds_with_teeth=5`；四个 md 配置的 40 行 `software_raid_state` 都是 `synchronizing=false`：

```text
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-mirror-2026-09-15.out 第 768 行（name=summary_control）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-mirror-2026-09-15.out 第 769 行（name=summary_control）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-mirror-2026-09-15.out 第 770 行（name=summary_control）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-mirror-2026-09-15.out 第 771 行（name=summary_control）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-mirror-2026-09-15.out 第 772 行（name=summary_control）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-mirror-2026-09-15.out 第 773 行（name=summary_control）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-mirror-2026-09-15.out 第 774 行（name=summary_control）
```

```text
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-mirror-2026-09-15.out 第 887 行（name=summary）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-mirror-2026-09-15.out 第 888 行（name=summary）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-mirror-2026-09-15.out 第 917 行（name=summary）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-mirror-2026-09-15.out 第 829 行（name=summary）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-mirror-2026-09-15.out 第 901 行（name=summary）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-mirror-2026-09-15.out 第 813 行（name=summary）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-mirror-2026-09-15.out 第 927 行（name=summary）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-mirror-2026-09-15.out 第 776 行（name=summary）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-mirror-2026-09-15.out 第 775 行（name=summary）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-mirror-2026-09-15.out 第 941 行（name=summary_done）
```

单盘那次 Bcachefs 的两格（用来对照镜像下的容量读数），抄自 `e152-file-system-benchmark-2026-09-15.out`：

```text
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-2026-09-15.out 第 725 行（name=summary）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-2026-09-15.out 第 724 行（name=summary）
```

**两盘镜像：这几个数说明什么**

1. **同配置比，冷挂载仍差得最多**：singlefs 冷恢复 8 293 ms，OpenZFS mirror 的 import 6 125 ms（singlefs 是它的 1.35 倍），ext4（md raid1）1.66 ms；singlefs 两盘读的字节与单盘那次逐字相同，还是整环扫描。
2. **小文件持久化**：singlefs 新池新建文件两盘合计 344 576 字节，是 XFS（md raid1）第一次 13 312 字节的 25.9 倍、Btrfs raid1 第一次 155 648 字节的 2.2 倍；OpenZFS mirror 那一格 5 轮离散 288%，不拿来比。
3. **Bcachefs 双副本的容量格不能比**：它的 statfs 报的是两块盘的原始容量，没按两个副本折半——镜像下的总容量 92.0%、可用 90.6%，与单盘那次逐字相同。
4. 镜像下写那两列的放大在 2.00 附近是镜像本身；OpenZFS mirror 的 4K 随机写放大 66.45。每一格的表在 `scripts/experiment-apparatus/perf-by-milestone.md` 第二·二节。

**单盘：八个配置各一块盘**

六个维度的表与 singlefs 那一张（`python3 scripts/experiment-apparatus/e152-tables.py research/results/e152-file-system-benchmark-2026-09-15.out`，整份原样取前七张；块层放大倍数与对照两张在同一份输出里）。产物已归档，从版本库 `3cff909b^` 取：

#### 大文件顺序写与顺序读（MiB/s，越大越好）

| 配置 | 顺序写 | 顺序读 |
|---|---|---|
| 裸盘（天花板） | 5 498 ⚠26% | 5 911 |
| ext4 | 5 505 | 5 894 |
| XFS | 5 573 | 5 902 |
| F2FS | 5 491 | 5 894 |
| Btrfs | 1 720 | 5 411 |
| Bcachefs | 5 211 | 5 793 |
| OpenZFS（vdev 是分区） | 736 | 2 216 |
| singlefs（两盘） | — | — |

#### 4K 随机读写（次/秒越大越好；p99 是完成延迟，µs，越小越好）

| 配置 | 随机读 次/秒 | 随机读 p99 | 随机写 次/秒 | 随机写 p99 |
|---|---|---|---|---|
| 裸盘（天花板） | 71 052 | 528 ⚠16% | 71 488 | 485 ⚠21% |
| ext4 | 64 626 | 518 ⚠21% | 52 739 | 684 ⚠23% |
| XFS | 64 742 ⚠15% | 518 ⚠22% | 52 324 | 675 ⚠26% |
| F2FS | 65 139 | 514 ⚠20% | 52 970 | 675 ⚠24% |
| Btrfs | 48 987 | 684 ⚠23% | 14 375 | 2 899 |
| Bcachefs | 33 922 | 1 155 | 51 574 | 978 |
| OpenZFS（vdev 是分区） | 3 213 | 10 420 | 6 151 | 12 517 |
| singlefs（两盘） | — | — | — | — |

#### 元数据 / 小文件（个/秒，越大越好）

| 配置 | 建空文件 | stat | 删文件 | 4 KiB 写 + fsync |
|---|---|---|---|---|
| ext4 | 238 095 ⚠25% | 1 000 000 ⚠48% | 303 030 ⚠22% | 708 |
| XFS | 222 222 | 1 250 000 ⚠42% | 192 307 ⚠20% | 706 |
| F2FS | 178 571 | 1 000 000 ⚠52% | 243 902 ⚠25% | 1 371 |
| Btrfs | 232 558 | 1 111 111 ⚠25% | 133 333 | 849 |
| Bcachefs | 140 845 | 1 111 111 ⚠31% | 126 582 ⚠22% | 1 008 |
| OpenZFS（vdev 是分区） | 123 456 | 909 090 ⚠41% | 82 644 ⚠38% | 1 970 ⚠22% |
| singlefs（两盘） | — | — | — | — |

#### 小文件持久化：新建 + 写 3000 字节 + fsync 文件 + fsync 目录（刚格式化的空文件系统上 100 次）

| 配置 | 延迟中位 µs | 延迟 p99 µs | 每次写请求 | 每次写字节 | 每次 FLUSH | 第一次写字节 | 第一次写请求 | 第一次 FLUSH |
|---|---|---|---|---|---|---|---|---|
| ext4 | 1 395 | 1 567 ⚠52% | 3.00 | 36 290 | 2.00 | 36 864 | 3.00 | 2.00 |
| XFS | 1 161 | 1 296 | 2.00 | 9 963 | 2.00 | 6 656 | 2.00 | 2.00 |
| F2FS | 1 083 | 1 178 | 5.02 | 20 561 | 3.00 | 20 480 | 5.00 | 3.00 |
| Btrfs | 2 183 | 2 467 | 15.8 | 188 907 | 4.00 | 143 360 | 13.0 | 4.00 |
| Bcachefs | 1 461 | 2 077 | 2.15 | 6 656 | 2.00 | 14 848 | 6.00 | 2.00 |
| OpenZFS（vdev 是分区） | 853 ⚠22% | 923 ⚠42% | 2.27 | 4 613 | 1.02 | 14 336 ⚠82% | 8.00 ⚠88% | 1.00 |
| singlefs（两盘） | — | — | — | — | — | — | — | — |

#### 冷挂载（卸载、清缓存之后再挂）

| 配置 | 挂钟 ms | 读字节 | 读请求 |
|---|---|---|---|
| ext4 | 1.41 ⚠19% | 21 504 | 6.00 |
| XFS | 6.76 ⚠31% | 4 235 776 | 45.0 |
| F2FS | 2.89 | 692 224 | 14.0 |
| Btrfs | 4.54 | 339 968 | 23.0 |
| Bcachefs | 26.5 ⚠37% | 5 039 104 | 33.0 |
| OpenZFS（vdev 是分区） | 3 102 | 6 688 768 | 170 |
| singlefs（两盘） | 7 898 | 1 610 932 224 | 393 297 |

#### 格式化与可用容量（16 GiB 盘）

| 配置 | 格式化 ms | 格式化写字节 | 格式化 FLUSH | 总容量 % | 可用 % |
|---|---|---|---|---|---|
| ext4 | 5.33 ⚠17% | 138 702 848 | 4.00 | 97.4 | 92.3 |
| XFS | 19.4 | 67 575 808 | 2.00 | 99.6 | 97.5 |
| F2FS | 14.2 | 39 903 232 | 1.00 | 100.0 | 98.1 |
| Btrfs | 12.7 | 4 984 832 | 14.0 | 100 | 96.8 |
| Bcachefs | 31.1 ⚠25% | 16 384 | 1.00 | 92.0 | 90.6 |
| OpenZFS（vdev 是分区） | 278 | 1 791 488 | 13.0 | 93.8 | 93.8 |
| singlefs（两盘） | — | — | — | — | — |

#### singlefs（两块 16 GiB 盘，门禁 qemu-device-streams 的真设备二进制）

| 配置 | mkfs + 取号 + 暖机 + 新池新建文件 ms | 覆盖写、释放、回退与复用（覆盖写 B）ms | 发布 B（二进制自己计）ms | 冷恢复 ms | 两盘读字节 | 两盘读请求 | B 两盘写请求 | B 两盘写字节 | B 两盘屏障 | B 两盘 FUA 写 |
|---|---|---|---|---|---|---|---|---|---|---|
| singlefs（两盘） | 17.8 ⚠131% | — | — | 7 898 | 1 610 932 224 | 393 297 | — | — | — | — |

整行抄自产物（产物已移出工作区的行写成一行指针）：

2026-09-15，40 次虚机跑全部一次跑通，没有格子被排除；七个带对照的配置每一行都是 `rounds_with_teeth=5`：

```text
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-2026-09-15.out 第 717 行（name=summary_control）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-2026-09-15.out 第 718 行（name=summary_control）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-2026-09-15.out 第 719 行（name=summary_control）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-2026-09-15.out 第 720 行（name=summary_control）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-2026-09-15.out 第 721 行（name=summary_control）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-2026-09-15.out 第 722 行（name=summary_control）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-2026-09-15.out 第 723 行（name=summary_control）
```

```text
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-2026-09-15.out 第 836 行（name=summary）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-mirror-2026-09-15.out 第 885 行（name=summary）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-mirror-2026-09-15.out 第 886 行（name=summary）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-2026-09-15.out 第 837 行（name=summary）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-2026-09-15.out 第 778 行（name=summary）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-2026-09-15.out 第 866 行（name=summary）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-2026-09-15.out 第 850 行（name=summary）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-2026-09-15.out 第 788 行（name=summary）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-2026-09-15.out 第 762 行（name=summary）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-2026-09-15.out 第 828 行（name=summary）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-2026-09-15.out 第 883 行（name=summary）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-2026-09-15.out 第 786 行（name=summary）
产物行见 git show 0ec0be214cdc:research/results/e152-file-system-benchmark-mirror-2026-09-15.out 第 941 行（name=summary_done）
```

singlefs 那一列的写入量不是量的，是 E142（新池新建文件的干跑） 产物 `e142-first-txn-dry-run-2026-09-14-round2-slot4096.out` 的录制流算出来的：

```text
产物行见 git show 0ec0be214cdc:research/results/e142-first-txn-dry-run-2026-09-18-change-count-three.out 第 38 行（name=write_list）
```

**单盘：这几个数说明什么**

1. **冷恢复读的量 5 轮是一个常数，绝大部分是整环扫描**：5 轮两盘读都是 1 610 932 224 字节、393 297 次请求（盘 0 805 568 512 字节 / 196 661 次，盘 1 805 363 712 字节 / 196 636 次）。`crates/singlefs-core/src/recovery.rs` 的 `scan_journal` 对每块盘把 768 MiB 的 journal 环按 4096 字节一槽逐槽读一遍，每盘 196 608 次、805 306 368 字节，两盘占读字节的 99.98%。块层计数与这个读法相容，但印证不了它：请求数比 2 × 196 608 多的 81 次只带 319 488 字节（盘 0 53 次 262 144 字节，盘 1 28 次 57 344 字节），读字节也不等于请求数 × 4 KiB（393 297 × 4 096 = 1 610 944 512，比实测多 12 288）；块层只数请求与字节，分不出哪次请求落在环上，要分得靠设备侧的读日志或二进制自己数读调用，拆法来自代码。挂钟 7 898 ms，是 ext4 冷挂载 1.406 ms 的约 5 600 倍、OpenZFS 3 102 ms 的 2.5 倍。
2. **小文件持久化**：singlefs 新池新建文件每块盘写 172 544 字节（盘 0：单元 8 次、journal 记录、根槽、系统配置槽各 1 次），六家在空文件系统上第一次做同样的事写 6 656 字节（XFS）到 143 360 字节（Btrfs）。singlefs 的挂钟只量到整段上界，5 轮分成 16.955–17.811 ms 与 40.123–40.277 ms 两群，不拿来比；40 ms 那两轮多出的 24.1 与 25.8 ms 是外层包装程序的打印积压（「这几个数说明什么（覆盖写、释放、回退与复用，与两盘镜像的六家比）」一节第 4 条）。
3. **六家之间**：OpenZFS 在默认参数下 4K 随机读是裸盘的 4.5%，块层每读 4 KiB 实际读 128 KiB（放大 32 倍，与默认 recordsize 128 KiB 对得上）；Bcachefs 随机读放大 16 倍；ext4、XFS、F2FS 四个数据作业的放大都是 1.00。每一格的表在 `scripts/experiment-apparatus/perf-by-milestone.md` 第二节。
4. 这些数不替任何决策定案：冷恢复的读量要不要降、怎么降，是设计推论，要另走三方论证。

````

**出处 `.claude/kb/experiments/152-按里程碑对比六家文件系统的文件性能.md:728-736`（整段抄，未转述）**

```markdown
### 口径

- 每个配置 5 轮，每轮一个新虚机、一块新盘，八个配置按轮交错；报中位、最小、最大与离散，离散 ≤ 15% 记「稳定」。
- 校验路径：每个 fio 作业前后读来宾 `/sys/block/vda/stat`；带 direct 的读（写）作业块层字节不到 fio 报的 90%，那一格记「被缓存顶替」（「写被吸收」），不进中位。
  自证：每轮两遍不带 direct 的读，第二遍必须被同一个判定函数判成「被缓存顶替」，判不出来那一轮的读格作废。
- singlefs 的小文件持久化第一次只量得到上界：它的真设备二进制把挂钟切成三段（起跑到新池新建文件、新池新建文件到覆盖写、释放、回退与复用、冷恢复），第一段仍拆不开 mkfs、取号、暖机；第二段（2026-09-17 起）是外层收到输出行的时刻之差：2026-09-17 两次跑的汇总行里混进了打印积压，不是发布 B 的时间（「这几个数说明什么（覆盖写、释放、回退与复用，与两盘镜像的六家比）」一节第 4 条）；跑前登记第十四节起装置先读完再转打、报包含自检。和六家比每次持久化的挂钟取二进制在 `name=file_overwrite` 行里自己报的 `nanoseconds=`（汇总指标 `singlefs_file_overwrite_inner_milliseconds`），它只罩发布 B。
- 写请求：六家是来宾块层数；singlefs 汇总行是二进制的写调用数，块层把每道屏障与每次 FUA 各多记一次，同口径比取块层数（「这几个数说明什么（覆盖写、释放、回退与复用，与两盘镜像的六家比）」一节第 5 条）。
- 六家那六张表的最后一行是 singlefs（`scripts/experiment-apparatus/e152-tables.py` 的 `SINGLEFS_FROM_SUMMARY` 与 `SINGLEFS_FROM_BLOCK_LAYER`），同一个量才填、没有的写「—」：小文件持久化的延迟中位取发布 B 二进制自己计的挂钟（µs）、每次写字节取发布 B 两盘写字节，每次写请求与每次 FLUSH 取每轮块层计数在发布 B 前后的差（两盘合计，`singlefs_timing` 行的整程计数减发布 B 之前那两行 `device_calls` 的 `write_ios`、`flush_ios`；冷恢复不写）；冷挂载三格取冷恢复的挂钟与两盘读字节、读请求。延迟 p99 与「第一次」三格、大文件、4K 随机、元数据、格式化与容量 singlefs 没有同一个量（没有 POSIX 接口；mkfs 没单独计）。负载差在哪见「这几个数说明什么（覆盖写、释放、回退与复用，与两盘镜像的六家比）」一节第 6 条。2026-09-29 那次块层写请求 36 = 写调用 29 + 屏障 6 + FUA 1、FLUSH 7 = 屏障 6 + FUA 1，与第 5 条的记法两条路对上。

```

**出处 `.claude/kb/experiments/152-按里程碑对比六家文件系统的文件性能.md:737-750`（整段抄，未转述）**

```markdown
### 它答不了的

1. 虚拟盘上的相对位置，不是真 NVMe 上的绝对性能。
2. 镜像的做法各家不同：ext4 / XFS / F2FS 走 md raid1（冷挂载不含阵列组装），Btrfs、Bcachefs、OpenZFS 用自己的镜像；Bcachefs 的 statfs 不按副本折半，它在镜像下的容量格不能和别家比。
3. 参数全取默认、一种尺寸、不老化、不并发。
4. singlefs 在新池新建文件上只有小文件持久化、冷挂载、格式化三维能跑。
5. 元数据的 stat 一列被 fio 按毫秒记的运行时间量化了：1 万个文件 stat 一遍，六家的中位对应的只有 8–11 ms，读数只能是「1 万 ÷ 整数毫秒」，分不出六家；要比得把文件数加到十万以上，另开一次正式跑。
6. 放大倍数（块层字节 ÷ fio 字节）是跑完之后从产物里的块层读数算的，跑前没登记成指标。
7. singlefs 每次持久化的 6.6 ms 在编码与校验、21 次写、5 次刷新之间怎么分，没拆。
8. singlefs 每棵树在 2026-09-17 那两次跑里只有一个节点：树长高、文件变多之后每次持久化写多少，没量。
9. 六家每次 8–200 KiB 写的是日志、意图日志还是树，没量；要用得派调研员现查，标未在本项目验证。
10. 差距怎么缩（数据单元的粒度、一次覆盖写碰到的七棵树各重写一个单元、冷恢复读整环）是设计推论，要走三方论证。
11. 2026-11-17 那次：singlefs 冷恢复中位比 2026-09-29 长 40%（9 914 → 13 918 ms）、每盘读写的字节与请求也变了，代码不是同一份，变在哪一处改动没查；第 1 轮三次宿主负载 10.02–27.19，负载与离散的关系只看了表里的 ⚠，没做对照轮。

```

**出处 `.claude/kb/experiments/152-按里程碑对比六家文件系统的文件性能.md:751-759`（整段抄，未转述）**

```markdown
### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D23（journal 的角色与格式） | 备料 | 2026-11-17 不受影响：新增 2026-11-17 那一节，冷恢复读量 1 612 242 944 字节、393 694 次仍是整环扫描的量级，冷恢复中位 13 918 ms，备料用法不变，没有分项的依据引这一页；此前 2026-11-05 不受影响：只改用法与复跑说明（一个门禁、两种跑法；验证包改名），结论、数与产物不变；此前 2026-10-15 不受影响：整行抄的产物行换成指向产物的指针，数、结论与产物不变 |
| D17（实现分层与第三方管道） 已定项 5 | 备料 | 2026-11-17 不受影响：新增 2026-11-17 那一节，singlefs 仍只有三张表能与六家同量比，没有 POSIX 接口，备料用法不变；此前 2026-11-05 不受影响：只改用法与复跑说明（一个门禁、两种跑法；验证包改名），结论、数与产物不变；此前 2026-10-15 不受影响：整行抄的产物行换成指向产物的指针，数、结论与产物不变 |
| D2（RAID 条带策略） 已定项 9 | 不影响 | 2026-11-17 不受影响：新增 2026-11-17 那一节，singlefs 仍跑两块盘，条带策略没有被量；此前 2026-11-05 不受影响：只改用法与复跑说明（一个门禁、两种跑法；验证包改名），结论、数与产物不变；此前 2026-10-15 不受影响：整行抄的产物行换成指向产物的指针，数、结论与产物不变 |
| D4（校验和位置） 已定项 5 | 不影响 | 2026-11-17 不受影响：新增 2026-11-17 那一节，没有量校验和位置；此前 2026-11-05 不受影响：只改用法与复跑说明（一个门禁、两种跑法；验证包改名），结论、数与产物不变；此前 2026-10-15 不受影响：整行抄的产物行换成指向产物的指针，数、结论与产物不变 |

```

**出处 `.claude/kb/decisions/13-验证路线.md:8-29`（整段抄，未转述）**

```markdown

| # | 分项 | 定案 |
|---|---|---|
| 1 | **crash refinement 先验哪一块** | 先验「一次发布 + 一次崩溃 + 一次恢复」这个封闭单元，第一个里程碑是任一 `jsn` 连续前缀不切开发布；挂载透明另立第四条性质；原候选 I-6.5（nonce 水位单调） 撤下 **状态：已定。** **进度：已实现。** |
| 2 | **事务层承不承诺串行提交** | 先串行，越过交点再开并行；将来并行提交时记录按事务号顺序追加 **状态：已定。** **进度：已实现。** |
| 3 | **自写参照实现（RefFS 形态）做到什么程度** | 做到极致取语义级：只在内存里按 POSIX 操作走，自己定义持久与崩溃后允许的状态，没有盘上格式；照 kb 另起、单独一个包、只收白名单输入 **状态：已定。** **进度：已实现。** |
| 4 | **崩溃点重放的崩溃状态集合怎么定义** | 写流按设备切段，段里每块有写的盘都被自己的屏障或 FUA 放行才关段；段内每次整写各取几态、任意组合，原地覆写多一态「新旧都读不出」；没持久放旧字节且真生成镜像；双写按设备各算 **状态：已定。** **进度：已实现。** |
| 5 | **checker 与被测实现允许共享什么** | 只共享一份从 kb 生成的常量模块，其余各写一份；判定宽度由 checker 自己探测再与系统配置比对，探不到报「声明值，未探测」 **状态：已定。** **进度：未实现。** |
| 6 | **三个互相独立的 oracle** | 内存模型对拍、独立解析器 + checker、独立规约执行器三个 oracle 各判一样；独立性要靠「三 oracle 合谋测试」验证，不许自称 **状态：已定。** **进度：未实现。** |
| 7 | **O2（独立解析器 + checker） 判什么** | O2（独立解析器 + checker） 的定义域是单个镜像，只判集合成员；需要第二个输入的检查归记录核对器（入参含枚举器给的持久集合），故意不给编号 **状态：已定。** **进度：已实现。** |
| 8 | **自研的两样** | Merkle 根链差分（相邻两代根递归比较，指针没变处不下钻）与代际增量 scrub（只遍历 birth generation 高于水位的块） **状态：已定。** **进度：未实现。** |
| 9 | **崩溃点重放怎么分层** | 三层都精确全量，枚举不完靠精确剪枝，一时跑不了的节点标 ignore 慢慢开；冒烟层就是门禁不指定范围的那一趟，提交时照单元键判全部；指定范围可缩域、缩过域不算门禁通过；先保失败可复现 **状态：已定。** **进度：已实现。** |
| 10 | **真正必须精确覆盖的 N 是什么** | 提交协议结构的等价类数，不是布局套数；落地是一个封闭的提交步骤枚举加穷尽 match **状态：已定。** **进度：已实现。** newtype 隔离（`RotationalPtr` / `ZonedPtr`）没有。 |
| 11 | **ignore 的节点什么时候开** | 否决按欠账轮转的账本；留下三条改管开的时机：确定性的覆盖记录、`T1`（常规层 挂钟预算） 是输入、判别力与覆盖分开检查 **状态：已定。** **进度：无对象。** |
| 12 | **冲突 1：checker 即规范与校验路径不许共享工具** | checker 做遍历与扫描两个方向，对同一条不变量各自独立判，共享的只有格式解析 **状态：已定。** **进度：已实现。** |
| 13 | **冲突 2：refinement 的 spec 与对拍的 model** | spec 与 model 在表述层面就写得不同；同源性要观测，≥ 5 次不一致而判 spec 错 0 次就合并、预算移给穷举 **状态：已定。** **进度：无对象。** |
| 14 | **与其他决策的连锁** | 哈希与 AEAD 在形式验证里只能当 uninterpreted function；日志结构三层绑架符号执行；checker 不必解析消息语义 **状态：已定。** **进度：无对象。** |
| 15 | **一个验证包、一个门禁、两种跑法** | 验证代码都在验证包 `singlefs-verification` 里；门禁只有一个，两种跑法：提交时不指定范围、照单元键判全部，平时主 agent 经任务服务交指定范围 **状态：已定。** **进度：已实现。** |
| 16 | **按段登记的段怎么分片、态数用多宽** | 登记段整域装不进 u64 就按片依次判，片大小默认 2^32 态、片起点是库里的续点；态数一律用 1280 位定宽整数、算术全 checked；登记表只经环境变量读 **状态：已定。** **进度：已实现。** |
| 17 | **判定复用的单元是什么** | 剪枝的单元是函数：一份判定沿不沿用只看它依赖的函数单元各自去注释后的指纹、工具链、解析出的依赖版本；按提交或树算改动、整文件原文、文件级与包级指纹、手写版本号、路径名字进键、整道标记一律不许 **状态：已定。** **进度：已实现。** |
| 18 | **GC 恒等性质怎么证、证到哪一层** | 在 core 上直接用 Verus 证、连胶水一起：发布、回退、崩溃后读映射三个闭包加四块，里程碑四全验；整理不改现行内容，回退候选至多少 k 个 **状态：已定。** **进度：未实现。** |

```
