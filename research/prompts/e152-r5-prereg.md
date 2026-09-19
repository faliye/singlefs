# E152（按里程碑对比六家文件系统的文件性能） 第五次正式跑：重跑登记

写于 2026-09-19 10:57 JST（01:57 UTC），装置改动（原登记第十四节）之后、这一次产物之前。不占新号。
判据照原登记 `research/prompts/e152-preregistration.md` 第十二、十三、十四节写死的，这里不改一个字；这份只把它们落到问题单的每一行上，补上阳性对照、几何取样点、停机条款与逐量的算法。

## 一、问题

主 agent 给的问题单 `research/prompts/e152-r5-questions.md` 逐字（不是为岔路建的，是问题单）：

| 编号 | 问题 | 够判条件 | 状态 |
|---|---|---|---|
| Q1 | 改过的装置下，singlefs 一臂每一轮的外层第二个事务挂钟包不包得住子进程自己计的数 | 5 轮每轮都有 `second_transaction_outer_contains_inner` 且取值确定；过半为 false / NA 照第十四节整格不报 | 开着 |
| Q2 | 子进程自己计的第二个事务挂钟（只罩发布 B）在 5 轮里的中位与离散 | 汇总有 `singlefs_second_transaction_inner_milliseconds`，5 轮都收 | 开着 |
| Q3 | 发布 B 的写数与段序列是否与第三、第四次正式跑逐字相同（两盘合计 21 次写请求、344 576 字节、4 次屏障、1 次 FUA，段序列 `16+2+1+2`），冷恢复是否读回第二版、根 1:4 | 5 轮每轮的 `name=second_transaction` 与 `recover_cold` 行都在 | 开着 |
| Q4 | 第一段（mkfs + 取号 + 暖机 + 第一个事务）里子进程连着打出的几行，到达时刻跨度是否都在 1 ms 以内 | 5 轮每轮都能从产物里数出这几行的跨度 | 开着 |

问法只有一种读法（判问法时逐字读了原登记第十四节与装置、`crates/` 的实现，见第三节）。读法写死如下：

- **取哪一次**：产物里每个 `(configuration=singlefs, round)` 可能有两次尝试（`E152RUN … attempt=1/2`）；只取 `vm_exit=0` 的第一次，与汇总器 `summarize_product` 同一个取法（装置第 1627 行起）。两次都失败的轮记「跑不起来」，这一轮对 Q1–Q4 都算缺一轮。
- **Q1**：取值 = 那一次的 `name=singlefs_timing` 行里 `second_transaction_outer_contains_inner=` 的原样值。「取值确定」= `true` 或 `false`；`NA`、字段不存在、不是布尔值都算不确定。答案是三个数：true 的轮数、false 的轮数、不确定的轮数。false + 不确定 ≥ 3（5 轮里过半）⇒ 外层「第二个事务（覆盖写 B）ms」一格整格不报（第十四节原文）。
- **Q2**：取 `E7RESULT name=summary configuration=singlefs metric=singlefs_second_transaction_inner_milliseconds` 那一行的 `rounds`、`median`、`minimum`、`maximum`、`spread_percent`、`stability`、`values`。单位毫秒 = 子进程 `nanoseconds=` ÷ 10⁶；离散 = (最大 − 最小) ÷ 中位；≤ 15% 记稳定（原登记第七节，装置第 36 行 `STABLE_SPREAD_PERCENT`）。`rounds=5` 才算「5 轮都收」。
- **Q3**：每一轮查十样，逐样各判（第六节 C1–C10）：`inner=second_transaction` 行的 `segments=`、`operations=`；`singlefs_timing` 行的四个 `second_transaction_*_both_devices`；`inner=recover_cold` 行的 `root=`、`content_matches=`；另把 `inner=second_transaction` 行的八个逐盘字段（`device_{0,1}_{writes,written_bytes,force_unit_access_writes,barriers}`）与 `segments=`、`operations=` 逐字比第三、第四次正式跑两份产物（`research/results/e152-file-system-benchmark-second-transaction-2026-09-17.out`、`research/results/e152-file-system-benchmark-second-transaction-final-2026-09-17.out`）里每一轮的同名字段。「逐字相同」= 这十个字段的字符串在新旧 15 轮里全等。`root_txg=`、`transaction=`、`released=`、`closed_form=` 照报、不判（问题单没问）。
- **Q4**：「这几行」= 同一轮 `singlefs_inner` 行里，从第一条 `inner=segments ` 行到 `inner=transaction ` 行（含两端）之间的全部行；子进程在这两行之间只做内存里的切段与格式化、不碰盘（第三节）。跨度 = `transaction` 行的 `at_nanoseconds` − 第一条 `segments` 行的 `at_nanoseconds`，单位纳秒。「在 1 ms 以内」= 跨度 ≤ 1 000 000 ns。另报这之间的行数 N 与相邻两行间隔的最大值（附带）。
  原登记第十四节点名的是 `segments`、`device_calls`、`transaction` 三种行；从第一条 `segments` 到 `transaction` 的跨度就是这三种行的跨度（`segments` 行在最前、`transaction` 行在最后，第三节），夹在中间的 `publish_writes`、`publish_writes_against_device` 行不改变两端，所以不是第二种读法。

## 二、被测条款与它引的定义

被测的是原登记第十四节写死的判据（连同第十二、十三节的写数判据与第七节的轮数、稳定性口径），与 kb 里定这种计时做法的那一节。下面四段由 `research/scripts/quote-kb.py` 整段抄出（出口 `/tmp/claude-1000/e152-r5-designer/quotes.md`，脚本回读逐字节一致、退出码 0），原样追加。原登记第十四节里引的实验页「这几个数说明什么（第二个事务，与两盘镜像的六家比）」一节是结论，没读、没抄。

**出处 `research/prompts/e152-preregistration.md:261-282`（整段抄，未转述）**

```markdown
## 十四、装置改动：读子进程输出与计时分开（2026-09-17 JST 写，改动之后只有一次冒烟跑，没有正式跑）

**为什么改**：第三、第四次正式跑之后发现，来宾程序 `run_singlefs` 在读子进程输出的循环里先给一行打时间戳、再把它转打到串口、才读下一行，子进程写管道不被挡，所以时间戳里混进前面几行的打印积压（最多 25.8 ms）。第四节与第十二节第 2 条「按结果行到达的时刻切三段挂钟」因此在那四次跑里都带着积压：两次第二个事务的跑 10 轮里 9 轮外层切出的挂钟小于子进程自己计的纳秒，物理上不可能。逐轮数与证据行在 `.claude/kb/experiments/152-按里程碑对比六家文件系统的文件性能.md`「这几个数说明什么（第二个事务，与两盘镜像的六家比）」一节第 4 条。第四到第十三节的判据与已有四份产物不改、不重算。

**改了什么**（`research/e7-index-bench/src/bin/e152_file_system_benchmark.rs`、`research/scripts/e152-tables.py`）：

1. `read_lines_with_arrival_times` 把子进程输出读到 EOF、每行记下读出来那一刻，签名里没有输出句柄；读完之后 `run_singlefs` 才按原次序、原格式转打 `singlefs_inner` 行、再切三段。三段挂钟的定义不变，仍是行到达时刻之差。
2. `singlefs_timing` 行末尾加两个字段：`second_transaction_inner_nanoseconds`（子进程 `name=second_transaction` 行的 `nanoseconds=`）与 `second_transaction_outer_contains_inner`（外层切出的第二个事务那一段 ≥ 子进程自己计的数为 true，缺值 NA）。
3. 宿主汇总：`singlefs_second_transaction_milliseconds` 只收 `second_transaction_outer_contains_inner=true` 的轮，false、NA、字段不存在、不是布尔值四种都报一行 `name=summary_excluded` 带理由、不进中位；新增指标 `singlefs_second_transaction_inner_milliseconds`，5 轮都收。
4. `e152-tables.py` 的 singlefs 表加一列「发布 B（二进制自己计）ms」，产物里没有这个指标写「—」。
5. 单测 22 → 26、变异 16 → 26，26 条全抓、0 条无效、0 条没红（`research/mutations/e152_file_system_benchmark.tsv` M17–M26）。

**从下一次正式跑起的判据（跑前写死）**：

- 每轮 `second_transaction_outer_contains_inner=true`。**触发的观测**：任一轮 false 或 NA ⇒ 那一轮的外层挂钟不进中位（汇总自动排除并报 `summary_excluded`），如实记下，回查 `run_singlefs` 是不是又在读的循环里转打、或子进程的计时点挪了位置；5 轮里过半被排除 ⇒ 外层第二个事务那一格整格不报。
- 和六家比每次持久化的挂钟时，singlefs 取 `singlefs_second_transaction_inner_milliseconds`：它只罩发布 B，与六家「每次单独计时、只罩那一次操作」同口径；外层那一格多罩从分配记录重建分配器、切段与打印结果行，只作包含自检。
- 第一段（mkfs + 取号 + 暖机 + 第一个事务）与冷恢复那一段没有子进程自己计的数可比，只靠第 1 条的读与转打分开；它们仍是行到达时刻之差。**触发的观测**：子进程连着打出、中间没有计算的几行（`segments`、`device_calls`、`transaction`）到达时刻跨度超过 1 ms ⇒ 读的循环里又有了阻塞，那一轮第一段不进中位、回查。

**冒烟跑**（改动之后、这一节写之前，产物不留存，放在会话暂存目录）：`E152_ROUNDS=1 E152_CONFIGURATIONS=singlefs bash research/scripts/e152-run.sh <暂存目录>/e152-smoke-relay-timing.out`，一轮一次过，`vm_exit=0`，宿主负载 1.99。从第一行 `segments` 到 `transaction` 行 9 行跨 24 µs（改之前的跑里最多 25.8 ms）；外层第二个事务 6 632 915 ns，子进程自己计的 6 575 144 ns，`second_transaction_outer_contains_inner=true`；`content_matches=true`，根 1:4。

**它答不了的**：包含自检只查外层包不包住里层，两边一起偏查不出；子进程自己计的纳秒只有这一条路，没有第二条路核过。

```

**出处 `research/prompts/e152-preregistration.md:236-259`（整段抄，未转述）**

```markdown
## 十二、里程碑「第二个事务」的 singlefs 重跑（2026-09-17 JST 写，装置改动之后、一格产物都还没跑）

只重跑 singlefs 那一臂（`E152_CONFIGURATIONS=singlefs bash research/scripts/e152-run.sh <产物>`），六家的基线不重跑（内核、测试台、负载都没变）。这一轮改了三样，都在跑之前写死：

1. 真设备二进制多一个模式 `second-transaction`（`crates/singlefs-harness/src/bin/first_transaction_on_device.rs`）：第一个事务之后同一个进程里再覆盖写一次（里程碑「第二个事务」步 1 的发布 B，4100 字节的第二版），分配器从第一个事务的分配记录重建（与可写挂载同一条路），冷重开读回的是第二版；它自己报一行 `name=second_transaction`：B 的 txg、事务号、释放的落点数、挂钟纳秒、录制流里 B 的操作数与段序列、逐盘写请求 / 写字节 / FUA 写 / 屏障（程序计数，门禁 55 号已证录制流与设备侧日志逐项相等，不另量）。来宾二进制的 singlefs 臂从此跑这个模式（第四节那段的 `direct` 换成 `second-transaction`）。
2. 挂钟从两段切成三段：起跑到 `name=transaction` = 第一个事务的写路（mkfs + 取号 + 暖机 + 第一个事务，仍拆不开，记 P 的上界）；`name=transaction` 到 `name=second_transaction` = 第二个事务的挂钟（**这一格才是稳态的每次代价**，六家拿来比的是刚格式化之后第一次写，singlefs 这一格与它们的「第二次写」同口径）；`name=second_transaction` 到 `name=recover_cold` = 冷恢复。
3. 报四个新指标：B 两盘合计的写请求、写字节、屏障、FUA 写（`singlefs_second_transaction_*`），表由 `research/scripts/e152-tables.py` 从 `name=summary` 行生成。

**判据（跑前写死）**：

- B 的段序列必须是 `16+2+1+2`、操作数 21（与 E142（第一个事务的干跑） 的 `path=transaction` 同型：8 个单元 × 2 盘 + 记录 × 2 + 根 FUA + 超级块 × 2）；两盘合计写请求 21、写字节 = 2 × 172 032 + 512（根槽 512 只落一块盘）= 344 576；屏障两盘合计 4、FUA 1。**触发的观测**：`name=second_transaction` 行的 `segments=` 不是 `16+2+1+2`，或 `singlefs_timing` 行的四个 `second_transaction_*_both_devices` 与上面的数不等 ⇒ 那一轮的 singlefs 格作废，先查二进制再重跑。
- 冷恢复读回第二版（`content_matches=true`），根是 1:4。**触发的观测**：`recover_cold` 的 `root=` 不是 `1:4` 或 `content_matches=false` ⇒ 作废。
- 5 轮，格子取中位、离散 =（最大 − 最小）÷ 中位，超过 15% 标 ⚠ 照报（第七节不变）；第二个事务的挂钟这一格离散多半会大（几毫秒量级的挂钟被虚机调度吃掉），照报、不删。

**它答不了的**：大文件顺序读写、4K 随机读写、元数据、格式化的挂钟与写量（真设备整环写 0 还没做）——这四维在这一轮的表里仍写「不能跑」，缺的能力照 `research/perf-by-milestone.md`「以后的里程碑怎么往下接」那张表。冷恢复仍读整环，这一轮没动它。

## 十三、里程碑「第二个事务」收尾时的 singlefs 再跑一次（2026-09-17 JST 写，一格产物都还没跑）

用户 2026-09-17 在里程碑收尾时要求「完成后参照实验 152 进行一轮 benchmark」。第十二节那次跑（产物 `research/results/e152-file-system-benchmark-second-transaction-2026-09-17.out`）之后 `crates/` 又改了四处：抬 F 回收的槽扣住到生效、提交内生块的 bump 游标绕开开段之后才置的隔离位与扣住位（`crates/singlefs-core/src/allocator.rs` 的 `allocate_commit_generated` 多一个跳过循环，发布 B 走这条路）、抬 F 接住读不出计数、checker 多判 I-7.4（近 K 代块未被复用） 与 I-4.8（近 K 代根校验和自洽）（checker 不在真设备二进制的路径上）。真设备二进制 `second-transaction` 模式的负载、判据、指标一个字不改。

- 命令：`E152_CONFIGURATIONS=singlefs bash research/scripts/e152-run.sh research/results/e152-file-system-benchmark-second-transaction-final-2026-09-17.out`，5 轮。六家的基线不重跑（内核、测试台、负载都没变）。
- 判据与作废条款逐条照第十二节：B 的段序列 `16+2+1+2`、两盘合计 21 次写请求 / 344 576 字节 / 4 次屏障 / 1 次 FUA，冷恢复读回第二版、根 1:4。**触发的观测**：任一轮 `name=second_transaction` 行的 `segments=` 或四个计数与之不同 ⇒ bump 路的跳过循环改变了发布 B 的落点或写数，那一轮作废、如实记下并回查 `allocate_commit_generated`；`content_matches=false` ⇒ 作废。
- 与第十二节那次比：两次的写数必须逐字相同（跳过循环在发布 B 上不应碰到任何被挡的槽）；挂钟两次各报 5 轮中位与离散，不合并、不取平均，差异只写观测，不下「变快 / 变慢」的结论——第十二节那次第二个事务挂钟的离散已是 186%。**触发的观测**：写数不同 ⇒ 上一条作废条款；挂钟中位差得比两次各自的离散还大 ⇒ 照报，写明两次宿主负载。
- 宿主上可能同时在跑门禁 54 号（层 0 全量，`nice -n 19`，一个核）：照第七节逐轮记 1 分钟负载，照跑照记。
```

**出处 `research/prompts/e152-preregistration.md:94-99`（整段抄，未转述）**

```markdown
## 七、轮数与稳定性

- 每个配置 5 轮，每轮一个新虚机、一块新盘；八个配置轮流跑（第 1 轮八个全跑完再跑第 2 轮），宿主上的漂移平摊到每一家。
- 报中位，另报最小、最大与离散 = (最大 − 最小) ÷ 中位；离散 ≤ 15% 记「稳定」，> 15% 记「不稳定」，照报不删。
- 每轮开跑前记宿主 1 分钟负载（同一台机器上可能有别的会话在跑）；负载高的那一轮照跑照记、不删，汇总时逐轮列出负载。

```

**出处 `.claude/kb/vm-harness.md:147-163`（整段抄，未转述）**

```markdown
## 计时不在转发输出的循环里打时间戳

来宾里常由一个包装程序把被测程序起成子进程（`Stdio::piped()`），读它的结果行、转打到自己的标准输出，宿主从串口抓。**不许拿「读一行、打时间戳、转打这一行、再读下一行」那个循环里行到达的时刻当分段挂钟。**
来宾的标准输出是串口 ttyS0（`research/scripts/vm-bench.sh` 的 `-serial mon:stdio` 与 `console=ttyS0`），转打一行要阻塞几毫秒；子进程往管道里写不被挡，于是后面每一行的时间戳都带着前面几行的打印积压，而且积压忽大忽小，看着像虚机调度的抖动。

实测（2026-09-17，E152（按里程碑对比六家文件系统的文件性能） 第三、第四次正式跑）：子进程连着打出、中间只有几微秒计算的几行，时间戳跨了 21.7–25.8 ms；两次跑 10 轮里 9 轮外层切出的第二个事务挂钟小于子进程自己计的纳秒，物理上不可能；前两次跑第一段挂钟「分两群」也是这个积压。逐轮数在 E152（按里程碑对比六家文件系统的文件性能） 实验页「这几个数说明什么（第二个事务，与两盘镜像的六家比）」一节第 4 条。
装置改成先读完再转打之后，同一台机器上冒烟跑一轮，那几行 9 行跨 24 µs，外层 6 632 915 ns 包住子进程自己计的 6 575 144 ns（`research/prompts/e152-preregistration.md` 第十四节）。

| 做法 | 例子 |
|---|---|
| 在被测进程里自己计时，把纳秒写进结果行 | 真设备二进制 `name=second_transaction` 行的 `nanoseconds=`；和别家比每次操作的挂钟时取这个数，它只罩那一次操作 |
| 真要按到达时刻切，先把子进程输出整份读到 EOF、每行记下到达时刻，读完再转打 | E152（按里程碑对比六家文件系统的文件性能） 装置的 `read_lines_with_arrival_times`：签名里不给输出句柄，读的循环里写不出转打 |
| 外层与里层两个数都有时，报一个包含自检，外层不包住里层的轮不进中位 | E152（按里程碑对比六家文件系统的文件性能） 的 `second_transaction_outer_contains_inner`，汇总时 false、NA、缺字段的轮报 `summary_excluded` |

谁在拦：门禁 65 号（`research/scripts/relay-timing-lint.py`）扫 `research/` 与 `crates/` 下的 Rust 与 Python 装置，读子进程输出的循环里同时取时间与输出就判红；它认得出读循环被挪进同一个文件里的另一个函数（跨一层）；输出藏进自己写的函数、子进程输出跨文件或隔两层以上传递、经通道或结构体字段传出去、shell 的 `while read` 配 `date` 这几种写法它够不着，逐条列在脚本头。不扫 `research/prompts/` 与 `research/results/`（冻结证据）。
⚠️ 包含自检只查外层包不包住里层，两边一起偏查不出；子进程自己计的纳秒只有这一条路。

```

## 三、实现今天的样子

登记时的快照（2026-09-19 01:57 UTC）：主仓 HEAD `5efaf79191c0f22af41f1ffef88ba8421aa5f8a8`；`git status --short -- crates/` 零行（派发提示说的别的会话在 `crates/singlefs-harness/src/crash.rs`、`lib.rs`、`history.rs` 的改动，这时已不在工作区里；以后再出现也不带进 worktree，见第五节）。
被测装置三份文件在主仓工作区里是改过、没提交的（`git diff --stat` 共 284 行增、19 行删），登记时的 sha256：

| 文件 | sha256 |
|---|---|
| `research/e7-index-bench/src/bin/e152_file_system_benchmark.rs` | `e1a88dc0f57e66744506c1f2928aba36623865ac523e373ff1431b3954143d4b` |
| `research/scripts/e152-tables.py` | `8f57f3d97c14aecc57f87b3d72751533a390bb4bb55dfc27726c96d5e9233e9f` |
| `research/mutations/e152_file_system_benchmark.tsv` | `9c4ae959d13a43cd13f7461c1b8c30e1a2f6e61bd796ac1ef440bea10f553620` |

HEAD 里的那一版装置跑的是 `direct` 模式、在读的循环里转打（`git show HEAD:research/e7-index-bench/src/bin/e152_file_system_benchmark.rs` 的 `run_singlefs`：`.arg("direct")`，`for line in BufReader::new(child_output).lines()` 里先 `started.elapsed()` 再 `emitter.emit`）；原登记第十二节的 `second-transaction` 模式与第十四节的改动都只在工作区里。所以 worktree 必须拷入工作区这三份，只取 HEAD 会跑成旧装置。

**子进程**（`crates/singlefs-harness/src/bin/first_transaction_on_device.rs`，HEAD 与工作区相同）：

- 第 247–250 行 `Emitter::emit` 用 `println!` 打每一行；Rust 的标准输出按行冲刷，每行写完就进管道，行到达时刻能反映子进程打它的时刻。
- 第 585 行打 `name=geometry`；第 597 行 `run_first_transaction`（mkfs + 取号 + 暖机 + 第一个事务，唯一的大段计算与盘 I/O）；之后第 635 行起的 `paths` 是 5 个元素的数组，第 648/650 行每个打一条 `name=segments`（`mkfs`、`instance_acquisition`、`warm_up`、`transaction`、`post_mkfs_stream`）；第 658/660 行两块盘各打一条 `name=device_calls`（块层计数用的是第 597 行前后已经读好的两份，不再读盘）；第 672/673 行每次暖机打一条 `name=publish_writes`（暖机次数 = `crates/singlefs-format/src/lib.rs` 第 204 行 `WARM_UP_EMPTY_PUBLISHES: u64 = 2`，经 `crates/singlefs-core/src/transaction.rs` 第 459 行 `for txg_number in 1..=WARM_UP_EMPTY_PUBLISHES`）；第 679 行第一个事务的 `publish_writes`；第 690、699 行两条 `publish_writes_against_device`；第 701 行 `name=transaction`。
  从第一条 `segments` 到 `transaction` 共 5 + 2 + 2 + 1 + 2 + 1 = 13 行，中间只有内存里的切段与格式化，不碰盘。
- 第 701 行打 `transaction` 之后，第 723 行 `let started = Instant::now();`，第 741 行 `let nanoseconds = started.elapsed().as_nanos();`，第 754 行才打 `name=second_transaction … nanoseconds={nanoseconds} … segments=… {per_device}`；逐盘字段由第 99–104 行 `describe` 写成 `device_{i}_writes`、`device_{i}_written_bytes`、`device_{i}_force_unit_access_writes`、`device_{i}_barriers`。
  所以两行到达的间隔物理上 ≥ 子进程自己计的数，前提是两行的到达时刻都不带额外的偏差。计时罩的只有发布 B（第 723–741 行之间的 `publish_overwrite`）；分配器重建（第 721–722 行 `PoolAllocator::rebuild_from_records`）与切段、打印都在表外。
- 第 853 行打 `name=recover_cold … root=… content_matches=…`。

**来宾装置**（`research/e7-index-bench/src/bin/e152_file_system_benchmark.rs`，工作区版）：

- 第 1071–1079 行 `read_lines_with_arrival_times`：`for line in reader.lines()` 里只做 `started.elapsed()` 与 `push`（第 1075 行），签名里没有输出句柄；读到 EOF 才返回。
- 第 1116 行 `run_singlefs`：第 1124 行 `.arg("second-transaction")`；第 1129 行读完全部输出；之后 `child.wait()`、读块层计数；第 1134–1167 行按原次序转打 `singlefs_inner`（第 1166 行），并记下 `geometry`、`transaction`、`second_transaction`、`recover_cold` 四行的到达时刻。
- 第 1195–1211 行打 `singlefs_timing`：`write_path_nanoseconds` = `transaction` − `geometry`（第 1198 行；注意起点是 `geometry` 行，不是起子进程那一刻，原登记第四、十二节写的是「从起跑到」——不在 Q1–Q4 里，照报给主 agent）；`second_transaction_nanoseconds` = `second_transaction` − `transaction`；末尾两个字段来自第 1098 行 `second_transaction_containment_fields`（里层取子进程那一行的 `nanoseconds=`，外层 ≥ 里层为 true，缺一个为 NA）。
- 第 1494 行 `second_transaction_wall_clock_if_contained` 与第 1551 行起的 `singlefs_timing` 读数：外层那一格只收 true 的轮；`singlefs_second_transaction_inner_milliseconds` 五轮都收。
- **原登记第十四节第 3 条（Q4 触发时「那一轮第一段不进中位」）装置里没有实现**：汇总器对 `singlefs_write_path_milliseconds` 不看跨度，照收每一轮。所以 Q4 触发时这一步由执行员手算（第六节 D4）。
- `research/scripts/e152-run.sh`（HEAD 与工作区相同）：每轮用 `VM_CPUS=4 VM_MEM=4096`，每次尝试前读 `/proc/loadavg` 第一列写进 `E152RUN … host_load1=`（原登记第七节的逐轮负载），全部跑完才把 `.partial` 改名成产物；它按自己所在的仓编译来宾二进制、汇总二进制与 `first_transaction_on_device`（`REPOSITORY="$(cd "$RESEARCH/.." && pwd)"`），所以在 worktree 里跑就用 worktree 的 `crates/`。

## 四、跑之前已经存在的数

写这份之前已经读到或算出的数，照实列；没读实验页的结果与结论小节，没读 `research/results/` 下任何产物的内容（只 `ls` 了文件名与大小）。

| 数 | 从哪读到 | 对判据的影响 |
|---|---|---|
| 改之前的读循环里，时间戳混进的打印积压最多 25.8 ms | 原登记第十四节「为什么改」 | 阳性对照 P1 的预期方向由它而来（第五节），P1 因此不是盲的；Q4 的门槛 1 ms 是原登记第十四节写死的，不由它定 |
| 第三、第四次正式跑 10 轮里 9 轮外层第二个事务挂钟小于子进程自己计的纳秒 | 原登记第十四节；`.claude/kb/vm-harness.md` 第 152 行 | P1 对 Q1 那一半只是「重算能不能复现 9 / 10」的公式核对，不算盲的阳性对照（第五节、第十节 F6） |
| 那几次跑里子进程连着打出的几行，时间戳跨 21.7–25.8 ms | `.claude/kb/vm-harness.md` 第 152 行 | 同上，P1 对 Q4 那一半的预期；不知道它数的是哪几行（见下面「9 行」） |
| 冒烟跑一轮：那几行「9 行跨 24 µs」；外层 6 632 915 ns、子进程自己计的 6 575 144 ns（差 57 771 ns，命令核：`python3 -c 'print(6632915-6575144)'` 输出 `57771`），`second_transaction_outer_contains_inner=true`，`content_matches=true`，根 1:4，宿主负载 1.99 | 原登记第十四节；`.claude/kb/vm-harness.md` 第 153 行 | 一轮、不留存，不进这次结论；它是改动之后唯一一次外层包住里层的观测，判据不照它定 |
| **我按 `crates/` 数出的行数是 13，与冒烟跑说的「9 行」对不上** | 第三节逐行数；`python3 -c 'print(5+2+2+1+2+1)'` 输出 `13` | 列成钉绝对值的断言与停机条款（第七节 B3、第十一节 S4）：产物里数出来不是 13 就两边都查 |
| 第三次正式跑第二个事务挂钟离散 186% | 原登记第十三节 | 不影响 Q1–Q4 的判据；Q2 的离散照第七节 15% 判稳定与否 |
| 发布 B 两盘合计 21 次写、344 576 字节、4 次屏障、1 次 FUA，段序列 `16+2+1+2`，操作数 21，根 1:4，读回第二版 | 原登记第十二、十三节（条款本身写死的） | 就是 Q3 的锚点（第七节 A 类） |
| 发布 B 逐盘：盘 0 程序 10 次写、2 道屏障；盘 1 程序 11 次写、2 道屏障、1 次 FUA；块层 12 / 14 | `.claude/kb/vm-harness.md` 第 167 行 | 与条款的合计相容（10 + 11 = 21，2 + 2 = 4，0 + 1 = 1）；Q3 逐盘比的是新旧产物的原样字段，不照这几个数判 |
| 装置单测的样本：外层 1 991 507 ns 对里层 6 608 505 ns（注明「2026-09-17 第二次正式跑第 2 轮」），外层 16 327 481 ns 对里层 6 368 614 ns | 装置第 2056–2095 行 | 取自旧产物的真实读数，与第二行「9 / 10」同源；只当单测输入，不影响判据 |

阳性对照 P1、P2 的门槛（第六节 E1–E4）是读过上面几行之后写的，写的时候知道旧产物大概会过；这一点就是 P2 要跑的理由（P2 在新装置上现跑，写门槛时没有它的数）。

## 五、臂、阳性对照、真实基线

**被测的一条臂：singlefs，改过的装置（先读到 EOF、再转打）。**

- 怎么做：来宾装置以第三节登记的 sha256 那一版，起 `first_transaction_on_device <盘 0> <盘 1> second-transaction`，把它的输出读到 EOF、每行记下读出来那一刻，读完再转打、再按行到达时刻切三段；`crates/` 取主仓 HEAD `5efaf79`（执行时 HEAD 若已前移，取执行那一刻的 HEAD，写进第十二节并照第十一节 S1 核 `crates/` 与本登记的第三节逐项相同）。
- 做完会怎样（待证命题，不是定义）：读的循环里不再写串口，行到达时刻里没有打印积压，于是 Q1 每轮外层 ≥ 里层、Q4 每轮跨度 ≤ 1 ms。
  从「怎么做」推得出的只有「积压这一个来源没了」；读的进程被调度晚醒、宿主抢占 vCPU 这两个来源还在，所以后一句要量，不能当定义。
- 对面那条臂，照支持它的人认的样子写：**边读边转打**（HEAD 那一版的写法）。支持它的说法是「转打一行只有几微秒到几毫秒，比几毫秒的第二个事务、几百毫秒的第一段都小，按到达时刻切出来的数够用」。它今天不在正式跑里（原登记第十四节已换掉它），只在阳性对照里出现。

**真实基线：不适用。** Q1–Q4 问的都是装置自己量得准不准、发布 B 的写数变没变，没有「这个项目打算实现的方案比现有的多换来什么」这一问；六家的基线不重跑，也不进 Q1–Q4 的判据（拿 `singlefs_second_transaction_inner_milliseconds` 与六家比是原登记第十四节的下游用法，不在这张问题单里）。

**阳性对照（每条被测的判据都要有一个，只有一条臂，所以就是这一条臂的对照）：**

| 编号 | 做什么 | 证明什么 | 盲不盲 |
|---|---|---|---|
| P1 | 不跑虚机。拿第三、第四次正式跑两份产物（边读边转打那一版装置跑的，10 轮 singlefs），用第六节 D1 的同一段脚本算每轮跨度，用 A1 的同一个比较算每轮「外层 ≥ 里层」（外层取旧 `singlefs_timing` 行的 `second_transaction_nanoseconds=`，里层取同一轮 `inner=second_transaction` 行的 `nanoseconds=`） | Q4、Q1 的判定脚本在有积压的产物上会判出触发 | 不盲：预期方向在第四节已知 |
| P2 | 跑虚机，第二段。worktree 里的来宾装置只改一处：第 1075 行 `        arrived_lines.push(ArrivedLine { at_nanoseconds: started.elapsed().as_nanos(), text });` 换成先取时刻、再 `println!("relay_control {text}");`、再 push（行首不是 `E7RESULT`，vm-bench.sh 不收它，完整性闸的条数不变），让读的循环里重新有串口写。`E152_ROUNDS=3 E152_CONFIGURATIONS=singlefs`，产物 `research/results/e152-file-system-benchmark-second-transaction-relay-in-loop-control-2026-09-19.out`；跑完把那一处换回来，`sha256sum` 回到第三节的值 | 同一台机器、同一份 `crates/`、同一个模式下，把积压放回去，Q4、Q1 的判定会不会触发 | 盲：写门槛时没有它的数 |

P2 读循环里转打的是 `relay_control` 加上子进程原行，每行比原来 `singlefs_inner` 的转打短一截（少了 `configuration=… round=… at_nanoseconds=…`），积压只会偏小；它触发了就说明判据有牙，没触发不等于旧装置没有积压。

**执行步骤（第一段）：**

1. 主 agent 建 worktree（共用约束不许 subagent 做 `git worktree`）：`git worktree add --detach /tmp/claude-1000/e152-r5-wt HEAD`。执行员从主仓把第三节三份文件拷进 worktree 同一相对路径，`sha256sum` 三份须等于第三节的值（不等 ⇒ 装置在登记之后又改过，停下交回，第十一节 S1）；`git -C /tmp/claude-1000/e152-r5-wt status --short` 恰好这三行 ` M`；`git -C /tmp/claude-1000/e152-r5-wt diff --quiet HEAD -- crates/` 退出码 0。
2. 在 worktree 里：`cd research && nice -n 19 cargo test --release --bin e152-file-system-benchmark`（26 个单测全过）；`nice -n 19 bash research/scripts/mutate.sh e152-file-system-benchmark e7-index-bench/src/bin/e152_file_system_benchmark.rs mutations/e152_file_system_benchmark.tsv`（参数形态以脚本头为准，26 条全抓、0 无效）；`python3 research/scripts/e152-tables.py --selftest` 通过。
3. 第十一节 S2–S3 的两处对拍（worktree 里的 `crates/` 与第三节的行一致）。
4. 开跑前 `ps -o pid,args -u "$(id -u)"`，有 `qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio` 在跑就等；记下 `cat /proc/loadavg` 与 `nproc`。
5. 在 worktree 根：`E152_CONFIGURATIONS=singlefs bash research/scripts/e152-run.sh research/results/e152-file-system-benchmark-second-transaction-relay-timing-2026-09-19.out`（5 轮是脚本默认）。跑完拷回主仓：主仓 `research/results/` 下先确认同名文件不存在，`cp` 后两边 `sha256sum` 相等。门禁 69 号的提示还要 `research/scripts/replay.sh` 里这个实验的登记行指到新产物；登记时 `grep -n -i 152 research/scripts/replay.sh` 零命中，E152 在那里有没有、要不要加登记行，由执行员照它的定义与门禁 69 号办。
6. 照第六节逐量算、照第七节核锚点、照第八节报轨迹；P1 在同一段里做（不跑虚机）。判定脚本先过第九节的自证再用在产物上。

**分段：**

- 第一段（上面 1–6）：补 Q1、Q2、Q3、Q4 四行。按问题单的够判条件，四行在第一段交回时都够判；P1 给 Q1、Q4 的判定脚本一个不盲的「会触发」证据。
- 第二段（主 agent 认了才跑）：P2（3 轮）与第八节的几何取样点 G1（`VM_CPUS=1`，3 轮）。它们不补问题单的哪一行，补的是 Q1、Q4 的答案能不能写成「装置改好了」而不只是「在 4 vCPU、这一天的宿主负载下没触发」。第一段交回后主 agent 对着问题单判：四行都够判就停，这两项逐条标「够判后未跑」，Q1、Q4 的答案照第十节 F7 带上限定语。
- 实现量：1 臂 × 5 轮（第二段另加 2 × 3 轮），虚机单轮只跑 singlefs 一个子进程；要另写的只有一份读产物的判定脚本（Q1–Q4 与 P1 共用）和第九节的造数自证。一个执行员一次做得完第一段。

**主 agent 在装置跑之前定的（2026-09-19，读过这份登记之后、一格产物都还没跑）**：
- worktree 由主 agent 建（`git worktree add --detach /tmp/claude-1000/e152-r5-wt HEAD`，拷入第三节登记的三份文件并核 sha256），执行员在里面编译与跑，不改主仓的这三份文件。
- 第二段 P2（把转打放回读的循环，3 轮）与 G1（`VM_CPUS=1`，3 轮）都跑：P2 是 Q1、Q4 判据唯一的阳性对照，G1 是方向相反的几何取样点。
- 正式 5 轮与第二段都**不加** `nice -n 19`：这一跑量的是挂钟，降优先级会让虚机在宿主有负载时让出 CPU、改掉被量的东西；逐轮照第七节记宿主 1 分钟负载代替。编译与出表照共用约束加 `nice -n 19`。

## 六、报哪些量与各自的判据

每个量各占一行、各报各的判定；「轮」一律按第一节的取法选那一次尝试。门槛除注明「原登记」的之外都在这里写死；被测臂的定义里没有任何一个门槛能直接推出来（臂只定义了读法，没定义跨度与包含关系的取值）。

| 量 | 怎么算 | 门槛 | 判定 | 对应哪一行、什么值让答案翻面 |
|---|---|---|---|---|
| A1 包含关系 | 每轮 `singlefs_timing` 的 `second_transaction_outer_contains_inner=` 原样 | 原登记第十四节：`true` | 逐轮报 true / false / NA / 缺 | Q1。任一轮不是 `true` ⇒ 答案从「每轮都包住」翻成「第 k 轮没包住」；false + 不确定 ≥ 3 ⇒ 外层一格整格不报 |
| A2 包含余量 | 每轮 `second_transaction_nanoseconds − second_transaction_inner_nanoseconds`（有符号整数，纳秒） | 不另设门槛，是 A1 的数值形态 | 只报，不单独判 | Q1 的轨迹（第八节） |
| A3 外层、里层复算 | 外层 = 同一轮 `inner=second_transaction` 行的 `at_nanoseconds` − `inner=transaction` 行的 `at_nanoseconds`；里层 = `inner=second_transaction` 行的 `nanoseconds=` | 两数分别与 `singlefs_timing` 行的 `second_transaction_nanoseconds`、`second_transaction_inner_nanoseconds` 逐位相等 | 逐轮相等 / 不等 | Q1 能不能判：任一轮不等 ⇒ 装置抄错字段或切错行，Q1 不够判，走第十一节 S5 |
| B1 收进的轮数 | `metric=singlefs_second_transaction_inner_milliseconds` 汇总行的 `rounds=` | `5` | 等 / 不等 | Q2 够判条件：`rounds` < 5 ⇒ Q2 不够判 |
| B2 中位与离散 | 同一行的 `median`、`minimum`、`maximum`、`spread_percent`、`stability` | 原登记第七节：离散 ≤ 15% 稳定 | stable / unstable | Q2。离散 > 15% ⇒ 答案是「不稳定」，照报不删 |
| B3 汇总复算 | 另写脚本从 5 轮 `second_transaction_inner_nanoseconds` ÷ 10⁶ 算中位（5 个取第 3 个）、最小、最大、离散 | 与 B2 的三位小数（离散一位小数）逐字相等 | 相等 / 不等 | Q2 能不能判：不等 ⇒ 汇总器错，走第十一节 S5 |
| C1 段序列 | 每轮 `inner=second_transaction` 行 `segments=` | 原登记第十二节：`16+2+1+2` | 逐轮 | Q3。任一轮不等 ⇒ 「不同」，那一轮作废（第十一节 V2） |
| C2 操作数 | 同一行 `operations=` | `21` | 逐轮 | Q3，同上 |
| C3 写请求 | `singlefs_timing` 的 `second_transaction_writes_both_devices=` | `21` | 逐轮 | Q3，同上 |
| C4 写字节 | `second_transaction_written_bytes_both_devices=` | `344576` | 逐轮 | Q3，同上 |
| C5 屏障 | `second_transaction_barriers_both_devices=` | `4` | 逐轮 | Q3，同上 |
| C6 FUA 写 | `second_transaction_force_unit_access_writes_both_devices=` | `1` | 逐轮 | Q3，同上 |
| C7 冷恢复的根 | `inner=recover_cold` 行 `root=` | `1:4` | 逐轮 | Q3。不等 ⇒ 「不同」，那一轮作废 |
| C8 读回第二版 | 同一行 `content_matches=` | `true` | 逐轮 | Q3。`false` ⇒ 那一轮作废 |
| C9 与第三、第四次逐字比 | `inner=second_transaction` 行的 `segments`、`operations` 与八个逐盘字段，共 10 个字段；新 5 轮每轮每字段与旧两份产物 10 轮的同名字段比字符串 | 全等 | 10 字段 × 5 轮各一格 | Q3「逐字相同」。任一格不等 ⇒ 答案翻成「第 k 轮的某字段不同」；C3–C6 相等而 C9 的逐盘字段不等 ⇒ 「合计同、两盘之间的分配不同」 |
| C10 旧两份之间是否同 | 旧 10 轮的上述 10 个字段彼此比 | 全等 | 等 / 不等 | Q3 的比较对象是否唯一：不等 ⇒ C9 分别对第三次、第四次各报一列，不合成一个「相同」 |
| D1 第一段那几行的跨度 | 第一节 Q4 的定义 | 原登记第十四节：≤ 1 000 000 ns | 逐轮 | Q4。任一轮 > 1 ms ⇒ 答案翻成「第 k 轮超」，那一轮第一段不进中位（D4），回查读循环 |
| D2 那几行的行数 N | 从第一条 `inner=segments` 到 `inner=transaction`（含两端）的 `singlefs_inner` 行数 | `13`（第七节 B3） | 逐轮 | Q4 能不能判：不等 ⇒ 第十一节 S4 |
| D3 相邻两行最大间隔 | 那 N 行里相邻两行 `at_nanoseconds` 之差的最大值 | 无 | 只报 | 附带，产物里现成，不另跑 |
| D4 第一段中位（剔除 D1 触发的轮） | 只在 D1 有轮触发时算：其余轮的 `write_path_nanoseconds` ÷ 10⁶ 的中位、最小、最大、离散，与汇总行并排报 | 无 | 只报 | Q4 触发之后原登记第十四节要的「不进中位」；装置没实现这一步（第三节） |
| E1 P1 跨度 | 旧两份产物 10 轮，用 D1 的同一段脚本 | > 1 ms 的轮 ≥ 6 | 有牙 / 未证 | Q4 的判定有没有判别力：< 6 ⇒ Q4 的答案写「判别力未证」，第二段 P2 变成必跑 |
| E2 P1 包含 | 旧两份产物 10 轮，用 A1 的比较（外层 ≥ 里层）现算 | false 的轮 = 9（第四节已知数的复现） | 复现 / 不复现 | Q1 的判定脚本对不对：≠ 9 ⇒ 脚本或 kb 那个数有一个错，第十节 F6 |
| E3 P2 跨度 | 第二段 P2 产物 3 轮，D1 的同一段脚本 | > 1 ms 的轮 ≥ 2 | 有牙 / 没牙 | Q4 的判据在新装置、同一台机器上有没有牙：没牙 ⇒ Q4 的「都在 1 ms 内」不能当「积压没了」的证据 |
| E4 P2 包含 | P2 产物 3 轮的 A1 | false 的轮 ≥ 1 | 有牙 / 没牙 | Q1 同理；没牙 ⇒ 包含检查的判别力只剩单测（M17、M18） |
| G1 几何取样点 | 第八节 | 见第八节 | 翻 / 不翻 | Q1、Q4 |
| H1 宿主负载 | 每轮 `E152RUN … host_load1=` | 无（原登记第七节：照跑照记） | 只报 | 与 A2、D1 逐轮并排，Q1、Q4 的上下文 |

附带、不判的：每轮 `child_exit`、`recovery_nanoseconds`、`root_txg`、`transaction`、`released`、`closed_form`——产物里现成，照报，不为它们多跑一次。

## 七、钉绝对值的断言

Q1、Q4 本身就是绝对判据（外层 ≥ 里层；跨度 ≤ 1 ms），不是臂间互比；这一节钉的是「所有轮一起错」时能抓住它的那几个数。

**A 类：出自被测条款本身（原登记第十二、十三节）。不符 ⇒ 走第十节 F4「条款可能错，或 `crates/` 改了发布 B 的路」，不是作废装置：**

| 断言 | 值 | 管哪一格 |
|---|---|---|
| A-1 段序列 | `16+2+1+2` | C1 |
| A-2 操作数 | 21 | C2 |
| A-3 两盘写请求（程序计数） | 21 | C3 |
| A-4 两盘写字节 | 344 576 | C4 |
| A-5 两盘屏障 | 4 | C5 |
| A-6 两盘 FUA 写 | 1 | C6 |
| A-7 冷恢复的根 | `1:4` | C7 |
| A-8 读回第二版 | `content_matches=true` | C8 |

**B 类：独立算出、用命令核过的。不符 ⇒ 第十一节的停机条款（两边都查），不当结果：**

| 断言 | 怎么算的 | 命令与原样输出 | 管哪一格 |
|---|---|---|---|
| B-1 A-4 的算术 | 2 × 172 032 + 512 | `python3 -c "print('2*172032+512 =', 2*172032+512)"` → `2*172032+512 = 344576` | 核条款自己的加法；不核 172 032 这个数本身（它出自 E142 的形状，这份登记没有独立推它） |
| B-2 A-2 的构成 | 8 × 2 + 2 + 1 + 2 | `python3 -c "print('16+2+1+2 =', 16+2+1+2, '; 8*2+2+1+2 =', 8*2+2+1+2)"` → `16+2+1+2 = 21 ; 8*2+2+1+2 = 21` | C1 与 C2 自洽 |
| B-3 第一段那几行的行数 | 5（`paths` 数组，子进程第 635 行）+ 2（盘数）+ 2（`WARM_UP_EMPTY_PUBLISHES`，`crates/singlefs-format/src/lib.rs` 第 204 行）+ 1 + 2 + 1 | `python3 -c "print('lines segments..transaction =', 5+2+2+1+2+1)"` → `lines segments..transaction = 13` | D2 |
| B-4 外层、里层复算 | 由产物里另两处字段现算（A3） | 产物出来后跑 | A3；B3 同理核 Q2 的汇总 |

B-3 的运行时值只能等产物；冒烟跑说的「9 行」与它不符（第四节），产物数出来是 13 则登记这一步对、「9 行」是另一种数法，是 9 则我读 `crates/` 读错了——两种都先查清再判 Q4。

## 八、轨迹与几何敏感性

**轨迹。** 被谓词消费的量有两个：A1（原登记第十四节「过半被排除 ⇒ 整格不报」消费它）与 D1（「超过 1 ms ⇒ 那一轮第一段不进中位」消费它）。这里没有「初始供给」可吃，轮次就是 5 轮，所以三样写成：

| 量 | 峰值（最坏的一轮） | 「为正」轮数 | 期末值 |
|---|---|---|---|
| A2 包含余量 | 5 轮里最小的余量与它是第几轮 | 余量 < 0 的轮数（= A1 为 false 的轮数） | 第 5 轮的余量 |
| D1 跨度 | 5 轮里最大的跨度与它是第几轮 | 跨度 > 1 ms 的轮数 | 第 5 轮的跨度 |

5 个值逐轮全报，旁边并排 H1（那一轮的宿主 1 分钟负载）。只报「全部 true」「全部 ≤ 1 ms」而不报最小余量、最大跨度的，正文里不许写「总是」「从不」。

**几何敏感性。** Q1、Q4 的判定只在一个取样点上量：来宾 4 vCPU（`e152-run.sh` 写死 `VM_CPUS=4`）、宿主负载是那一天的。读的进程能不能在子进程写完一行后马上醒来，是这两个判定依赖的旋钮，而这个旋钮不是任何条款给的。方向相反的取样点：

| 编号 | 取样点 | 为什么方向相反 | 怎么跑 | 报什么 |
|---|---|---|---|---|
| G1 | `VM_CPUS=1`（其余不变），3 轮 | 子进程与读的进程抢同一个 vCPU：读的进程更可能晚醒，`transaction` 行的到达时刻可能比 `second_transaction` 行晚醒得更多，外层就可能 < 里层；跨度也可能变大 | 第二段。worktree 里 `research/scripts/e152-run.sh` 那一行的 `VM_CPUS=4` 换成 `VM_CPUS=1`（跑完换回），`E152_ROUNDS=3 E152_CONFIGURATIONS=singlefs`，产物 `research/results/e152-file-system-benchmark-second-transaction-relay-timing-vcpu1-2026-09-19.out` | A1、A2、D1 逐轮，与 4 vCPU 的 5 轮并列 |
| G2 | 边读边转打（P2） | 把积压放回去，是这次装置改动本身的反方向 | 第五节 P2 | E3、E4 |

判定：G1 的 3 轮里 A1 全 true 且 D1 全 ≤ 1 ms ⇒ 记「两点（4 vCPU 与 1 vCPU）都不触发」；有一轮触发 ⇒ 记「不稳定：1 vCPU 下第 k 轮触发」，Q1 / Q4 的答案限定在 4 vCPU。
判别力自证：把 D1 的门槛挪到正式 5 轮的最大跨度与 P2（或 P1）的最小跨度之间，D1 对正式跑的判定不变、对 P2（P1）必须全判触发；挪不进去（两组有交叠）就照报交叠，不写「门槛分得开两种装置」。
第二段不跑的，G1、G2 两行照写「够判后未跑」，Q1、Q4 的答案照第十节 F7 带限定语。

## 九、变异

**装置自己的变异表**（`research/mutations/e152_file_system_benchmark.tsv`，26 条；第五节第 2 步在 worktree 里重跑一遍，要 26 条全抓、0 条无效）。与 Q1–Q4 有关的几条，写明它在哪个取样点上改变输出：

| 变异 | 在哪个取样点上改变输出 |
|---|---|
| M17 `>=` 换 `<=` | 外层 ≠ 里层的每一点：单测里外层 16 327 481 / 里层 6 608 505 由 true 翻 false，外层 1 991 507 由 false 翻 true；相等那一点不变 |
| M18 `>=` 换 `>` | 只在外层与里层逐纳秒相等那一点（单测第 2056 行起那个测试里的相等样本）；正式跑的轮几乎不会落在这一点上，它只由单测盯 |
| M19 里层读 `root_txg` | 每一轮（`root_txg` 是个小整数，里层变得极小、包含恒真） |
| M20 / M21 / M22 汇总照收 false / 缺字段 / NA 的轮 | 只在有那种轮的点上：单测第 2101 行起的造数第 2、4 轮等；正式跑若 5 轮全 true，这三条在产物上不改变任何输出 |
| M23 里层指标读外层字段 | 外层 ≠ 里层的每一轮 |
| M24 读的循环丢掉第一行 | 每一轮（第一行是 `geometry`，`write_path_nanoseconds` 变 NA） |
| M25 时间戳倒着数 | 每一轮 |
| M26 一段的长度取后一行的绝对时刻 | 每一轮 |
| M5 稳定门槛放宽十倍 | Q2 离散落在 15%–150% 之间的点 |
| M6 偶数个取上中位 | 只在收进的轮数为偶数时（Q2 若有一轮作废只剩 4 轮）；5 轮时不改变输出 |
| M7 失败的尝试也收 | 只在某一轮第一次 `vm_exit≠0` 时 |

M1–M4、M8–M16 管六家与镜像那一半，不在 Q1–Q4 的路径上，照表重跑、不单列。
变异表里没有一条把转打放回读的循环（那要真虚机、串口才分得出），这一处由 P2 在虚机上做（第五节），由门禁 65 号（`research/scripts/relay-timing-lint.py`）在源码形状上拦。

**执行员判定脚本的自证**（脚本是执行员另写的，读产物算 A–E 各量）。先拿一份造数的产物跑，造数里放齐下面这些点；每条变异注入脚本后，造数上的判定必须翻：

| 变异 | 造数里让它翻的那一点 |
|---|---|
| J1 跨度的起点取 `inner=geometry` 而不是第一条 `inner=segments` | 一轮 `geometry` 比第一条 `segments` 早 50 ms、跨度本身 0.3 ms：原判 ≤ 1 ms，变异判 > 1 ms |
| J2 跨度的终点取最后一条 `device_calls` 而不是 `transaction` | 一轮 `transaction` 比前一行晚 0.8 ms、整段 1.1 ms：原判 > 1 ms，变异判 ≤ 1 ms |
| J3 门槛写成 `< 1 000 000` | 一轮跨度恰好 1 000 000 ns：原判「在以内」，变异判超 |
| J4 包含写成 `>` | 一轮外层、里层逐纳秒相等：原判 true，变异判 false |
| J5 C4 的锚点写成 344 577 | 每一轮都翻 |
| J6 每轮取第一次尝试而不看 `vm_exit` | 一轮第一次 `vm_exit=2`、第二次 0，两次的数不同：原取第二次，变异取第一次 |
| J7 B3 的中位取平均 | 一组 5 个偏斜的值（1、1、1、1、10）：中位 1，平均 2.8 |
| J8 C9 只比新产物内部、不比旧产物 | 新 5 轮彼此相同而与旧产物某一字段不同：原判不同，变异判相同 |

脚本、造数与每条变异的结果放执行员自己的草稿目录，运行记录进 `research/results/`（执行员的定义管）。

## 十、失败条款

每条后面紧跟「什么观测会让它触发」。触发了都是正当结果，照报，不改判据。

- **F1（Q1）：「读与转打分开之后外层一定包住里层」这句错了。** 触发的观测：任一轮 A1 = `false`（A2 < 0）。
  处置照原登记第十四节：那一轮外层挂钟不进中位（汇总自动排除），回查 `run_singlefs` 读的循环里有没有写输出、子进程计时点有没有挪；A1 不是 true 的轮 ≥ 3 ⇒ 外层那一格整格不报。结论写「装置改动之后仍有第 k 轮包不住」，不写成「装置坏了、整轮作废」。
- **F2（Q1）：包含关系判不了。** 触发的观测：任一轮 A1 = `NA`、字段不存在或不是布尔值。回查那一轮的 `inner=second_transaction` 行在不在、`nanoseconds=` 能不能解析；那一行不在同时让 Q3 那一轮缺数。
- **F3（Q4）：「读与转打分开之后第一段那几行跨度 ≤ 1 ms」这句错了。** 触发的观测：任一轮 D1 > 1 000 000 ns。处置照原登记第十四节：那一轮第一段不进中位（D4 手算，装置没实现），回查读的循环；答案写「第 k 轮跨度 x ms」。
- **F4（Q3）：原登记第十二、十三节的写数锚点不再成立。** 触发的观测：任一轮 C1–C6 有一格与第七节 A 类不等，或 C7 ≠ `1:4`、C8 = `false`。
  两种可能都要查：条款的锚点本身错，或第四次正式跑之后 `crates/` 改了发布 B 的路（`git log --since='2026-09-17 06:40' -- crates/singlefs-core/src/transaction.rs crates/singlefs-core/src/allocator.rs` 列出改动）。那一轮按原登记第十二节作废（第十一节 V2），停下交回，不自己改锚点、不盲目重跑。
- **F5（Q3）：合计同、两盘之间的分配变了。** 触发的观测：C3–C6 全等而 C9 的某个逐盘字段与旧产物不等。原登记只判合计，不作废；Q3 的答案写成「合计逐字相同，第 k 轮盘 i 的某字段由 a 变 b」。
- **F6（P1）：判定脚本复现不出已知的 9 / 10。** 触发的观测：E2 算出的 false 轮数 ≠ 9。要么脚本的取行、取字段、比较方向错，要么 kb 那个数错；查清之前 Q1 的判定脚本不可信，Q1 不够判。
- **F7（Q1、Q4 的判别力）：「没触发」分不清是积压没了还是判据没牙。** 触发的观测：E1 < 6；或第二段没跑；或第二段跑了而 E3 < 2（E4 = 0 同理管 Q1）。
  处置：Q4 的答案只能写「4 vCPU、宿主负载 x–y 下 5 轮最大跨度 z」，不写「积压没了」；E1 < 6 时第二段 P2 由可选变成必跑，交主 agent 定。
- **Q2 没有单独的失败条款**：Q2 问的是一个量的中位与离散，没有「应当是多少」的主张可错；离散 > 15% 是答案（「不稳定」）不是失败。它能不能判由 B1、B3 管（第十一节 S5）。
- **结果反过来我接不接受**：5 轮全 true、全 ≤ 1 ms，结论是「在这一个取样点上改动生效」；出现 false 或 > 1 ms，结论是「改动去掉了积压这一个来源，还有别的来源（读的进程晚醒、宿主抢占）」，照 F1、F3 写，不回头改门槛。

## 十一、作废条款与停机条款

**作废：**

- **V1** 原登记第八节第 1 条：`vm_exit≠0` ⇒ 脚本同参数自动重跑一次；两次都不过 ⇒ 那一轮「跑不起来」，附来宾 dmesg 末尾。触发：格式化、子进程或控制台出错。那一轮对 Q1–Q4 都缺，问题单的够判条件（5 轮）不满足，停下交回。
- **V2** 原登记第十二节：C1–C6 任一格不等 ⇒ 那一轮 singlefs 格作废，先查二进制再重跑（F4）。触发：`segments=` 不是 `16+2+1+2`，或四个 `_both_devices` 与锚点不等。
- **V3** 原登记第十二节：C7 ≠ `1:4` 或 C8 = `false` ⇒ 那一轮作废。触发：冷恢复读回的不是第二版。
- **V4** 装置不是登记的那一版：第五节第 1 步三份文件的 `sha256sum` 与第三节不等，或 worktree 里 `crates/` 与 HEAD 有差 ⇒ 整份产物不是这份登记的实验，作废。触发：拷错文件、别的会话在登记之后又改了装置、worktree 带进了没提交的 `crates/` 改动。
- **V5** 判定脚本的自证（第九节 J1–J8）有一条没翻 ⇒ 脚本出的判定作废，改脚本重判；产物不作废、不重跑。
- 宿主负载高、同时有别的会话在编译或跑门禁：不作废（原登记第七节照跑照记），逐轮 H1 并排报；开跑前看到别的性能测量在跑则先等（第五节第 4 步）。

**停机（装置与 `crates/` 对不上，`.claude/rules/implementation-first.md` 第 4 条：两边都查，既不作废也不当结果，停下交回）：**

- **S1** 第五节第 1 步的三项检查任一不过（与 V4 同一个触发，先停下查清是哪一种，再按 V4 判）。
- **S2** worktree 里子进程源码的次序不是：打 `name=transaction`（今天第 701 行）→ `Instant::now()`（第 723 行）→ `elapsed()`（第 741 行）→ 打 `name=second_transaction`（第 754 行）。触发：`grep -n '"name=transaction \|let started = Instant::now();\|let nanoseconds = started.elapsed().as_nanos();\|"name=second_transaction ' crates/singlefs-harness/src/bin/first_transaction_on_device.rs` 不是恰好四行、行号递增（登记时是 702、723、741、755——前后两个是格式串所在行，比 `emitter.emit(` 那一行晚一行）。次序变了，A1 的「物理上外层 ≥ 里层」这个前提就不成立。
- **S3** 子进程的标准输出不再按行冲刷：`Emitter::emit` 不再是 `println!`，或文件里出现包住标准输出的 `BufWriter`。触发：`grep -n 'println!\|BufWriter\|stdout()' crates/singlefs-harness/src/bin/first_transaction_on_device.rs` 与第三节不符。行不按写的时刻进管道，Q4 的跨度就量不到子进程打印的时刻。
- **S4** D2 ≠ 13（第七节 B-3）。
- **S5** A3 或 B3 任一轮不等：装置把字段抄错、行切错，或汇总器算错。

**够判停机：** 第一段交回时逐行对问题单：Q1 五轮 A1 都有确定值（或已照 F1 数出过半）、Q2 有 `rounds=5` 的汇总行、Q3 五轮十格齐、Q4 五轮 D1 齐 ⇒ 四行都够判，停。第二段的 P2（E3、E4）与 G1 逐条标「够判后未跑」，除非主 agent 认了要跑；E1 < 6 时 P2 改为必跑（F7），这一条交主 agent 定。

## 十二、修订

（留空。装置写之后、产物之前由执行员写，只许收严或补臂。）

## 十三、读过的文件与跑过的命令

**读过的文件（行号区间；grep 命中行也列）：**

| 文件 | 读了哪些行 |
|---|---|
| `.claude/agent-common.md` | 全文 |
| `research/prompts/e152-r5-questions.md` | 全文 |
| `.claude/singlefs-ai-sop/rules/test-discipline.md` | 第 35–134 行；节标题（`grep -n '^#'`） |
| `.claude/singlefs-ai-sop/rules/evidence-discipline.md` | 第 108–194 行；节标题 |
| `.claude/rules/three-way-inference.md` | 第 224–254 行（`grep -n '交岔路时写岔路单' -A30`）；节标题 |
| `.claude/rules/mutation-sampling.md` | 第 46–81 行；节标题 |
| `.claude/rules/implementation-first.md` | 第 22–40 行；节标题 |
| `research/prompts/e152-preregistration.md` | 全文 1–281 行 |
| `.claude/kb/vm-harness.md` | 第 147–175 行；`grep -n '^## \|^### '` 的节标题；第 164–168 行 |
| `research/e7-index-bench/src/bin/e152_file_system_benchmark.rs`（工作区版） | 第 320–352、1050–1230、1489–1730、2056–2072 行；`grep -n 'fn \|singlefs_timing\|…'` 的命中行（第 36、62–1729 行里的函数签名、第 1745–2127 行的测试名）；`git diff --stat` |
| 同一文件的 HEAD 版（`git show HEAD:…`） | `run_singlefs` 前 45 行；`grep -n 'second-transaction\|fn run_singlefs\|lines()\|elapsed'` 命中的第 749、752、782、908、949、960、963、994、1004、1030 行 |
| `crates/singlefs-harness/src/bin/first_transaction_on_device.rs` | 第 74–106、126–180、240–256、514–880 行；`grep -n` 命中的第 3、5、10、16、45、52、57、64、80–1171 行里的函数签名、第 249、253、303、517、528、554、585、597、613、635、648、650、658、660、672、673、679、690、699、701、702、722、723、726、738、741、743、749、754、755、764、774、804、808、842、853、854 行 |
| `crates/singlefs-core/src/transaction.rs` | 第 420–448 行；`grep` 命中第 20、459 行 |
| `crates/singlefs-format/src/lib.rs` | `grep` 命中第 203–204 行 |
| `crates/singlefs-core/src/mount.rs` | `grep` 命中第 100、102、165、846、862、869、878、880、881、909 行 |
| `crates/singlefs-harness/src/scenario.rs` | `grep` 命中第 12、57、100、104、120、125 行 |
| `research/scripts/e152-run.sh` | 全文 |
| `research/scripts/e152-tables.py` | `git diff`（第 72–84、205–235 行一带）；`grep -n 'argv\|selftest'` 命中第 5、9、202、235、240、245、246、248、249、250 行 |
| `research/mutations/e152_file_system_benchmark.tsv` | 全文 26 行 |
| `research/scripts/quote-kb.py` | 第 1–40 行 |
| `research/scripts/mutate.sh` | 第 1–45 行 |
| `research/scripts/vm-kernel.sh`、`research/scripts/e152-stage-root.sh`、`research/scripts/vm-bench.sh` | `grep -n 'CACHE\|cache_dir\|TMPDIR\|\$HOME\|/home'` 命中行（e152-stage-root.sh 第 8、16、30、37、38 行；vm-kernel.sh 第 6、14、18 行；vm-bench.sh 第 10、47、147、148、199 行） |
| `research/scripts/replace-once.py` | `--help` 输出 |
| `research/prompts/e151-r6-prereg.md` | 前 6 行与节标题（看重跑登记的文件头形态） |
| `.claude/gate.d/69-evidence-in-repo.sh` | 第 1–60 行里 `grep -n 'tmp\|prompts'` 的命中行、第 17–24 行、`grep -n` 命中的第 142、194、199、203 行 |
| `.claude/gate.d/stage-owners.tsv` | 按共用约束的 awk 列登记给实验设计员的阶段：零行 |
| `research/results/` | 只 `ls -la | grep -i e152`：四份 E152 产物的文件名、大小、时刻，没读内容 |

没读：实验页 `.claude/kb/experiments/152-按里程碑对比六家文件系统的文件性能.md`（任何一节）、`research/results/` 下任何产物的内容。

**跑过的命令（原样）：**

```
cat .claude/agent-common.md; cat research/prompts/e152-r5-questions.md; ls research/prompts/ | grep e152
for f in <六份规则与原登记>; do grep -n '^#' $f; done; wc -l research/prompts/e152-preregistration.md
sed -n '35,134p' .claude/singlefs-ai-sop/rules/test-discipline.md; sed -n '108,194p' .claude/singlefs-ai-sop/rules/evidence-discipline.md
grep -n '交岔路时写岔路单' -A30 .claude/rules/three-way-inference.md | head -60; sed -n '46,81p' .claude/rules/mutation-sampling.md; sed -n '22,40p' .claude/rules/implementation-first.md
sed -n '1,116p' research/prompts/e152-preregistration.md; sed -n '117,281p' research/prompts/e152-preregistration.md
git status --short -- crates/ research/; git diff --stat HEAD -- crates/singlefs-harness/src/bin/first_transaction_on_device.rs
date -u; git log --oneline -3; git status --short; git worktree list
git show HEAD:research/e7-index-bench/src/bin/e152_file_system_benchmark.rs | grep -n 'second-transaction\|fn run_singlefs\|lines()\|elapsed'
python3 -c "print('2*172032+512 =', 2*172032+512)"   # → 2*172032+512 = 344576
python3 -c "print('16+2+1+2 =', 16+2+1+2, '; 8*2+2+1+2 =', 8*2+2+1+2)"   # → 16+2+1+2 = 21 ; 8*2+2+1+2 = 21
python3 -c "print('lines segments..transaction =', 5+2+2+1+2+1)"   # → lines segments..transaction = 13
python3 -c 'print(6632915-6575144)'   # → 57771
sha256sum research/e7-index-bench/src/bin/e152_file_system_benchmark.rs research/scripts/e152-tables.py research/mutations/e152_file_system_benchmark.tsv
grep -n 'println!\|BufWriter\|stdout()' crates/singlefs-harness/src/bin/first_transaction_on_device.rs   # → 第 249、253 行 println!，其余 eprintln!，零 BufWriter
grep -n '"name=transaction \|let started = Instant::now();\|let nanoseconds = started.elapsed().as_nanos();\|"name=second_transaction ' crates/singlefs-harness/src/bin/first_transaction_on_device.rs   # → 702、723、741、755
nice -n 19 python3 research/scripts/quote-kb.py <草稿目录>/quotes.md '<原登记>@## 十四、…' '<原登记>:236-259' '<原登记>@## 七、轮数与稳定性' '.claude/kb/vm-harness.md@## 计时不在转发输出的循环里打时间戳'   # → ✓ 4 段整抄，回读逐字节一致，exit=0
diff <(awk 'NR>=30' research/prompts/e152-r5-prereg.md) <草稿目录>/quotes.md   # → same
python3 research/scripts/replace-once.py research/prompts/e152-r5-prereg.md … （四次定点替换：第三节一处行号、第四节两处、第十一节 S2 的触发命令）
```

上面四条 `python3 -c` 先在一个命令里合跑过一次，之后又逐条单跑，单跑的输出就是注释里写的那四行。

之后又跑的：`nice -n 19 bash .claude/gate.d/69-evidence-in-repo.sh`（退出码 1，红的是装置源码与变异表比最新 E152 产物新——这次正式跑要消掉的那一条；没有点名这份登记）；`grep -n 'e152' research/scripts/replay.sh` 与 `grep -n -i '152' research/scripts/replay.sh` 都零命中；另两次定点替换（第五节第 5 步末尾的 replay.sh 一句）。读过 `research/scripts/replay.sh` 的只有这两次 grep。
