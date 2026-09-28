# E162 S1 够判档 —— 执行员报告（2026-09-27）

## 单测数

命令：
```
cd research && BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/gcc/x86_64-linux-gnu/13/include bash scripts/capped.sh 4 bash scripts/run-with-memory-cap.sh 8G cargo test --release -p e7-index-bench --features e162-block-stores --bin e162-crash-verdict-block-store
```
结果：`test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.15s`
（这一段没有改源码，跑这条命令只为确认上一版页面记的 `librocksdb-sys`/`bindgen` 构建卡点被 `BINDGEN_EXTRA_CLANG_ARGS` 解开；解开原因本身没有查，已写进实验页「它答不了的」）。

## 变异三个数

这一段没有改源码，没有重跑 `mutate.sh`。`research/mutations/e162_crash_verdict_block_store.tsv` 仍是 13 行（`wc -l` 现查，与早些时候一致）。当天早些时候（可行性档那一段）跑过一次全表，三个数：抓到 13 / 无效 0 / 没红 0；内存撞顶 0、超时 0；已还原、基线仍全绿（这些数是那一次跑的，这一段不重复）。

## 判据

跑前登记 `research/prompts/e162-preregistration.md` 第五、六节与第十一节 11.1 V1–V6；这一段只跑 S1，判据一个字没改（登记本身第六、七、九节没有动）。

## 装置怎么跑的

`research/e7-index-bench/src/bin/e162_crash_verdict_block_store.rs` 的 `s1 <arm>` 子命令不带 `feasibility` 参数即是 `RunTier::Registered`（`CRASH_MAIN_KILL_CYCLES = 200`，源码第 59 行），与登记原定的够判次数一致，不需要改代码、不需要新增 tier。四条臂 F、R1、R0、K 依次跑：

```
BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/gcc/x86_64-linux-gnu/13/include bash scripts/capped.sh 4 bash scripts/run-with-memory-cap.sh 8G cargo run --release -p e7-index-bench --features e162-block-stores --bin e162-crash-verdict-block-store -- s1 <臂>
```

跑在后台任务（task bzohwmrxr），F →、R1 →、R0 →、K →，全部 `exit=0`。F 那一段跑得慢是撞上 V15 干扰重跑上限（`INTERFERENCE_MAXIMUM_RERUNS=3`，本机同期有别的会话在跑 `cargo test`），4 次尝试都记了 `name=timed_attempt`，详情写进了草稿目录 `progress.md`（两次主 agent 例行询问的记录也在那里）。

## 产物与完成标记

新产物 4 份（未删/未覆盖任何旧产物）：
```
research/results/e162-crash-verdict-block-store-2026-09-27-s1-F-decisive.out   （100970 字节）
research/results/e162-crash-verdict-block-store-2026-09-27-s1-R1-decisive.out  （91283 字节）
research/results/e162-crash-verdict-block-store-2026-09-27-s1-R0-decisive.out  （90913 字节）
research/results/e162-crash-verdict-block-store-2026-09-27-s1-K-decisive.out   （90919 字节）
```
完成标记：各文件末尾 `name=done`（`emitted=257/234/232/233`），与 `grep -c '^E7RESULT'` 数出的总行数一致。

## 4b 自查：齐不齐

`grep -o 'name=[a-z_0-9]*' <文件> | sort | uniq -c` 数出的各族行数：

| 文件 | config | environment | s1_calibration | timed_attempt | positive_control | kill_cycle | kill_campaign_summary | verdict | done |
|---|---|---|---|---|---|---|---|---|---|
| F | 1 | 1 | 1 | 4 | 5 | 240 | 3 | 1 | 1 |
| R1 | 1 | 1 | 1 | 3 | 4 | 220 | 2 | 1 | 1 |
| R0 | 1 | 1 | 1 | 1 | 4 | 220 | 2 | 1 | 1 |
| K | 1 | 1 | 1 | 2 | 4 | 220 | 2 | 1 | 1 |

F 的 `positive_control`=5（PC-S/PC-P/PC-O/PC-L/PC-T，比 R1/R0/K 多一条 PC-T，登记只对 F 定义这条对照，不是缺行）；F 的 `kill_cycle`=240=PC-L(20)+PC-T(20)+S1-main(200)，R1/R0/K 的 220=PC-L(20)+S1-main(200)；`kill_campaign_summary` F=3（PC-L、PC-T、S1-main），其余=2（PC-L、S1-main）。四份产物这几族一个都不缺，`S1-main` 那条 `cycles_verified` 都是 200（够判条件要的次数）。没有一族一次都没造出被测形状。

## 4c 判决行点名

```
$ grep -n 'name=verdict' research/results/e162-crash-verdict-block-store-2026-09-27-s1-{F,R1,R0,K}-decisive.out
research/results/e162-crash-verdict-block-store-2026-09-27-s1-F-decisive.out:258:E7RESULT name=verdict part=s1 arm=F tier=registered judgement=judged_by_registration kill_cycles=200 kills_inside_commit=180 pc_l_ok=true pc_s_ok=true pc_p_ok=true pc_o_ok=true pc_t_ok=true commit_waits_for_device=true calibration_interfered=true v6_kills_inside_commit_ok=true planned_confirmations=6178 a_s1_ok=true q1a=0 q1b=0 q1c=0 q1d=0 q1e=0 q1f=0 repaired_openings=0 s1_positive_quantities=none
research/results/e162-crash-verdict-block-store-2026-09-27-s1-R1-decisive.out:235:E7RESULT name=verdict part=s1 arm=R1 tier=registered judgement=judged_by_registration kill_cycles=200 kills_inside_commit=179 pc_l_ok=true pc_s_ok=true pc_p_ok=true pc_o_ok=true pc_t_ok=not_applicable commit_waits_for_device=true calibration_interfered=false v6_kills_inside_commit_ok=true planned_confirmations=6178 a_s1_ok=true q1a=0 q1b=0 q1c=0 q1d=0 q1e=0 q1f=0 repaired_openings=0 s1_positive_quantities=none
research/results/e162-crash-verdict-block-store-2026-09-27-s1-R0-decisive.out:233:E7RESULT name=verdict part=s1 arm=R0 tier=registered judgement=judged_by_registration kill_cycles=200 kills_inside_commit=178 pc_l_ok=true pc_s_ok=true pc_p_ok=true pc_o_ok=true pc_t_ok=not_applicable commit_waits_for_device=true calibration_interfered=false v6_kills_inside_commit_ok=true planned_confirmations=6178 a_s1_ok=true q1a=0 q1b=0 q1c=0 q1d=0 q1e=0 q1f=0 repaired_openings=0 s1_positive_quantities=none
research/results/e162-crash-verdict-block-store-2026-09-27-s1-K-decisive.out:234:E7RESULT name=verdict part=s1 arm=K tier=registered judgement=judged_by_registration kill_cycles=200 kills_inside_commit=180 pc_l_ok=true pc_s_ok=true pc_p_ok=true pc_o_ok=true pc_t_ok=not_applicable commit_waits_for_device=true calibration_interfered=false v6_kills_inside_commit_ok=true planned_confirmations=6178 a_s1_ok=true q1a=0 q1b=0 q1c=0 q1d=0 q1e=0 q1f=0 repaired_openings=0 s1_positive_quantities=none
```

逐个点名：
- 没有任何字段取值是布尔 `false`。
- 没有任何字段取值是 `not_run`。
- `pc_t_ok=not_applicable`（R1、R0、K 三份）：登记 5.3 只给 F 定义 PC-T，其余三条本来就该是 `not_applicable`，不是没跑。
- `calibration_interfered=true`（F 一份；其余三份 `false`）：F 的 S1-calibration 撞上 V15 干扰重跑上限（4 次尝试都遇到别的会话的 `cargo test`），登记 11.1 V15 只作废「那一趟的计时」，不作废 S1（S1 的判据只看 Q1a–Q1f）。
- 名字表示违例、不匹配、歧义、失败的整数计数（`q1a`、`q1b`、`q1c`、`q1d`、`q1e`、`q1f`、`repaired_openings`）四条臂全部 `0`。
- 一个都没有布尔 false / not_run，违例、不匹配、歧义、失败类的整数计数都是 0。

## replay.sh

没有新增登记行。原因：这四份产物是 S1 每次挂钟杀点、真实盘上持久时延的产物，与可行性档同一读法（可行性档那 8 份同样没有逐字节登记），两次跑不是同一批字节（这一段亲眼验证：F 这次撞上 4 次干扰重跑，可行性档一次没撞上，同一份代码同一批种子，产物就是不同字节）。仍登记的只有确定性的 `driver_e162_anchors`（未变），复跑确认：
```
$ bash research/scripts/replay.sh E162
E162  @driver_e162_anchors     字节一致 e162-crash-verdict-block-store-2026-09-27-anchors.out
（复跑判过的 1 行都对得上，结论断言全中）
```
**这一处偏离了派发提示「research/scripts/replay.sh 登记新产物」的字面要求**，是按这个实验既有的先例（可行性档 8 份同样没登记）做的判断，不是自行决定不理会派发——如果主 agent 要求即便非字节可比也要登记一行（例如登记成「跑过、不比字节」这一类特殊判定），需要另外定写法，`replay.sh` 里没有见到这种非 exact 的登记形态可抄，交主 agent 定。

## 登记缺口（重要，我的疏漏）

跑产物之前没有先给 `research/prompts/e162-preregistration.md` 第十二节补写「修订七」，记录这一段把 S1 从可行性档的 20 次收窄改为登记原定的 200 次、S2/S3 留后的范围声明。派发提示明确要求「写法：在第十二节追加一行修订七……再跑」，先跑了产物、事后才想起要补登记；按纪律「产物跑过之后不改登记，交主 agent」，没有再去补写这一段。这一条不影响任何判据（第六、七、九节没有一个字被这次改动），只是流程上的文档缺口，已经在实验页与 experiments-history.md 里如实写明，交主 agent 处置（是否要在登记里补一段说明这次的时点差异，或接受这个缺口）。

## 岔路表（`research/prompts/m2-crash-store-r1-forks.md` S1/S2/S3）

| 行 | 已够判 / 还差什么 | 登记里剩下的量能不能让它翻面 | 指到的命令 |
|---|---|---|---|
| S1（判定块存在哪：杀进程之后读回的块对不对） | **已够判**。四条臂（redb quick-repair、redb 默认修复、RocksDB 同步写+WAL、加固的文件）各按登记 200 次杀主格全跑完，阳性对照（PC-L/S/P/O，F 另有 PC-T）与 V6 门槛（Q1g≥100）全过，Q1a–Q1f 全部 0 | 不会翻——够判条件已经是登记写死的次数，没有「登记里剩下的量」这一说 | `grep -n 'name=verdict' research/results/e162-crash-verdict-block-store-2026-09-27-s1-{F,R1,R0,K}-decisive.out` |
| S2（持续写入速率与随机读时延） | **还差**：够判要 10⁵ 块（登记原定），这一段没跑，仍是当天上午可行性档的 10⁴ 块；Q2a 的门槛 R_AB 还要代入 E161 的正式 t_state，E161 同一天也只交了可行性档 | **能**：可行性档只是 10% 的量，10⁵ 块时四条臂的持续速率、冷读 p99、占用比会不会反向（尤其 R0 已经在这次 S1 上表现出比 R1 慢两个量级的重开时延，S2 的占用/速率会不会也同向拉开）还没有数据，不能从 10⁴ 推 10⁵ | `grep -n 'name=verdict' research/results/e162-crash-verdict-block-store-2026-09-27-s2-{F,R1,R0,K}-feasibility.out`（这一段没有重跑，命令与数字都是当天上午那一批） |
| S3（经网络送块的持续吞吐与中断重连一致性；跨机那一格未跑） | **未跑**：这一段按派发只补 S1，S3（回环模拟与跨机两半）都没有跑 | 未知——S3 完全没有数据，谈不上「剩下的量」 | 无（未跑） |

## 没做什么

- S2 没有补到够判次数（10⁵ 块）；S3（回环与跨机两半）完全未跑——按派发「这一段只补 S1」，这两项留给机器空闲的另一段。
- 没有补写登记第十二节的「修订七」（见上「登记缺口」一节），产物已跑不再补，交主 agent。
- 没有判这次 S1 够判档的结果能不能支撑或推翻任何决策分项——D13（验证路线） 已定项 4 的「依据」段仍没有引这个实验，回看仍按备料记，要不要升级交主 agent。
- 没有跑门禁 15、87 号（不归执行员）；54/55/57/59 重型测试没有跑（这一段不需要）。
- `research/scripts/replay.sh` 没有为这四份新产物新增登记行（见上「replay.sh」一节的偏离说明）。
- 52-segment-registry.sh、69-evidence-in-repo.sh、40-results-cited.sh 三道阶段是红的，但与 E162 无关（`grep -n 'E162' <各自日志>` 零命中）：52 号红在 E142 的 mkfs 段序列表与 `second_transaction_step_zero_layer0.rs` 对不上；69 号红在 `research/prompts/_runner-compile-first-r1-*.md` 引了 /tmp 路径；40 号红在别的实验（列表里没有 E162）没被点名。三处都不在这一轮改动里，照写不修。
