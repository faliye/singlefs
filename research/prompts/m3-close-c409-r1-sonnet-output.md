# m3-close-c409-r1 正推腿（Sonnet）报告（2026-10-18）

腿：正推（K1 候选与重叠、K2 代价缺口）。背景材料 `research/prompts/_m3-close-c409-r1-body.md`，方案按 `crates/` 今天的实现（主工作区现状）。
没编译、没跑重型；下面每个数标来源：「量的」（这一轮命令现算或产物里的原值）、「推的」（由量的常量算出）、「旧数」（早先产物或记录里的，只当参考，没在今天的树上重量）、「登记自述」（登记行自己写的，没复核）。
草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-close-c409-r1-sonnet/`（`segment_sums.py`、`segment_sums.out`、`commands.out`）。

## 一、先读到的事实（后面各节引用，不重复）

### 1.1 层表与「写请求」的口径

- 三层定案：「固定种子、固定脚本的负载与登记的崩溃枚举用例，不按写请求数限规模」（`.claude/kb/decisions/13-验证路线.md:246`）；「中等规模负载（写请求数上千），精确全量，枚举不完靠精确剪枝，一时跑不了的节点标 ignore，覆盖报告与门禁逐个列」（`.claude/kb/decisions/13-验证路线.md:247`，该行末尾还有「本次未跑」，被引号嵌套截在引文之外）；「常规层 负载 + 大规模负载 + 多设备档位交叉，N≥5 轮，精确全量」（`.claude/kb/decisions/13-验证路线.md:248`）。
- 「写请求」没有单独的定义。已定项 4 把枚举的单位定成「枚举的单位是一次整写」（`.claude/kb/decisions/13-验证路线.md:136`），实现里一次整写 = 录制流里一条 `RetainedWrite`，现成测试用 `writes.len()` 的常量钉它（例：`WRITES` 常量 `crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_fixed_script_stream.rs:90`）。下面「上千」一律按录制流写条数数。「规则没说」：上千是总写条数，还是每条流、每个节点的写条数，没写。
- 欠块：「常规层、大负载层在门禁里没有阶段」（`.claude/kb/decisions/13-验证路线.md:261`）；已定项 11 写「量不出 T1（常规层 挂钟预算） / C / 每格耗时」（`.claude/kb/decisions/13-验证路线.md:327`）并禁止「T1（常规层 挂钟预算） 这三个数一个都不许现在写进 kb」（`.claude/kb/decisions/13-验证路线.md:328`）。所以下面的数只进本报告，不进 kb。
- 用户 2026-10-18 弹窗定「立」，负载形态先开三方定：记录里的写法是「常规层、大负载层在门禁里立阶段；负载形态先开三方定」（`records/2026-10-18-里程碑三收尾与近期欠账.md:49`，主 agent 记的，不是用户逐字）。

### 1.2 状态数怎么算、今天已登记的流多大（量的）

- 展开方式三种：`NotExpanded`、`EveryProperSubset`（任意真子集，m 个原地覆写时 3^m·2^(n−m)−1）、甲二 `InPlaceSubsetsWithCopyOnWriteNoneOrAll`（`crates/singlefs-checker-tier/src/crash.rs:1965`、`:1970`、`:1975`）。每次写只取两态时一条流的全量态数 = 1 + Σ(2^段写数 − 1)，闭式在 `crates/singlefs-harness/src/memory_pool.rs:659` 的 `closed_form_state_count`。
- 十一份固定写表常量的写数、最长段、两态闭式（命令 `python3 -I segment_sums.py <仓根>`，原样输出在草稿目录 `segment_sums.out`）：

| 流（tests/crash_enumeration_ 后面的名字） | 段数 | 写数 | 最长段 | 两态闭式态数 |
|---|---|---|---|---|
| lifecycle_closure_stream | 139 | 824 | 26 | 288751838 |
| unlink_after_the_fixed_script_stream | 78 | 503 | 28 | 2013790337 |
| compaction_relocating_a_block_shared_with_a_snapshot_stream | 48 | 272 | 26 | 123338833 |
| compaction_relocation_stream | 40 | 238 | 30 | 1377370253 |
| truncate_under_a_snapshot_stream | 35 | 223 | 30 | 2438004868 |
| truncate_into_a_unit_then_extend_stream | 31 | 204 | 30 | 1695547518 |
| snapshot_destruction_resumed_after_a_crash_stream | 39 | 204 | 26 | 84869186 |
| truncate_to_zero_stream | 31 | 196 | 30 | 1628700798 |
| overwrite_and_unlink_under_a_snapshot_stream | 27 | 153 | 26 | 167837744 |
| unlink_under_a_snapshot_stream | 23 | 126 | 28 | 352387114 |
| snapshot_creation_stream | 15 | 62 | 24 | 16842782 |

- 最长的已登记冒烟流 lifecycle_closure 已有 824 条写（`crates/singlefs-checker-tier/tests/crash_enumeration_lifecycle_closure_stream.rs:68`，态数常量 `:77`），不到上千的 0.83 倍；固定脚本到卸载 `FULL_STATES_THROUGH_THE_UNMOUNT: u64 = 1_678_377_204`（`crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs:104`）。「常规层 = 上千写」与冒烟层最长流只差一个小因子，这是 K3 的靶子，K1 只记数。
- 十一份里 ≥10 写的段共 105 条，取值只有 16/18/20/22/24/26/28/30 八种（同一命令输出末行 `segments_of_10_or_more_writes_by_size`）。一次带单元的普通发布的态数 = 2^n + 11：「n = 24：16777227，n = 16：65547，n = 28：268435467」（`crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs:107`）。同文件注释写「n = 24 三处、16 四处、20 一处、28 六处」并点名这些发布是 A、B、C、写行之后的两次暖机、回退 D、五次覆盖写、抬 F 与卸载各第二次空发布、E（`crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs:95`）；按个数对上，n = 16 的四处是写行之后的两次暖机与抬 F、卸载各第二次空发布（对应是推的，注释没逐个写数）。也就是**没有用户单元的发布也要改写 16 个单元写**。
- 发布一次的写条数 = n + 5（上一次轮换 2、单元写 n、记录 2、根槽 1），由上面的段形推的（`segment_sums.out` 里 `2,24,2,1` 一类的段序列，如 snapshot_creation 的 `[2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 16, 2, 1, 2]`，`crates/singlefs-checker-tier/tests/crash_enumeration_snapshot_creation_stream.rs:48`）。

### 1.3 上千写的态数上下限（推的，由 1.2 的 n 值算，算式与原样输出见第九节）

| 每次发布的单元写数 n | 每次发布写条数 | 凑够 1000 条写要几次发布 | 全部发布的态数合计 |
|---|---|---|---|
| 16（空发布、暖机一类的最小发布） | 21 | 48 | 3146256 |
| 20 | 25 | 40 | 41943480 |
| 24 | 29 | 35 | 587202945 |
| 28（固定脚本里覆盖写的取值） | 33 | 31 | 8321499477 |
| 30 | 35 | 29 | 31138513215 |

读法：上千写的态数跨 4 个量级（3.1×10^6 到 3.1×10^10），决定因素是每次发布改写多少个单元，不是写条数。这是 K2 里「状态数」一栏的主轴。

### 1.4 「精确剪枝」「ignore」今天实际是什么（量的）

- 键只由内容定：块键 =「块键 = 这一步带屏障、FUA、盘、偏移、段号的写表与它起步的镜像（扇区终值）的摘要」加段展开、判器另读的输入与段内序号（`crates/singlefs-checker-tier/src/crash_identity.rs:5`；总则 `:4`）。所以今天的精确剪枝只有三样：同键块直接复用（前缀共享：E161 量的两条流共用前 11 段（取号、暖机、第一个文件）逐字节相同，snapshot_creation 与 unlink_after_the_fixed_script 的前 11 段写数同形（2,2,1,2,2,1,2,24,2,1,2，见上表常量），`research/results/e161-crash-state-dedup-and-time-split-feasibility-r2-2026-10-05.out:5`）、恢复出逐字节相同镜像的状态观察者只看一次（`crates/singlefs-checker-tier/src/crash_amplification.rs:9958` 一带的单测）、按段登记的分片（见下）。**全新内容的块没有任何东西替它剪**：一条新的上千写流，除共用前缀外，每个块都要现判。
- ignore：`CrashPointSpan` 的 `ignored` 字段，注释是「测试代码里预设的 ignore：这个点的段一律不展开（只留全持久那一个状态），路径照样往下走」（`crates/singlefs-checker-tier/src/crash_amplification.rs:239`，字段 `:240`，生效处 `:540`）。仓里 3 处写 `ignored: true`（`commands.out` 里 `grep -rn "ignored: true" crates --include=*.rs | wc -l` 输出 3）：1 处在 `crash_amplification.rs` 的单测，**2 处在一条真流上**——放量分项 `sigma-of-the-misaligned-reuse`（合表里对应登记行 `c561-sigma-full`）：「σ 之前的段与 σ 那次发布之后的段各包成一个 ignore 的点（只留全持久那一个状态：是剪枝，也是别的流可复用的前缀）」（`crates/singlefs-checker-tier/tests/record_checker_judges_absence_by_the_persisted_set.rs:1530`），只把 σ 那次发布展开；它的判器写作「判器是流水线的：两遍恢复、oracle、池级 checker、」加记录核对器（`crates/singlefs-checker-tier/tests/record_checker_judges_absence_by_the_persisted_set.rs:1531`）。这是 ignore 用在手写动作串上的唯一现成先例，也是下面「覆盖写链」「种子随机历史整流枚举」的做法模板：长历史录一遍，只展开够得着的发布，其余包成 ignore 的点。驱动行只报点数与态数：「驱动行只报点数与态数、不报类名」（`scripts/crash/crash-amplification.sh:163`）。已定项 9 要求的覆盖报告与门禁逐个列「本次未跑」，今天报不到节点名，这是缺口。
- 按段登记的段不是精确全量：「登记了的段，引擎在重型里只按轻型那两个域的并展开」（`.claude/gate.d/crash-point-replay-segments.tsv:2`）。位置寻址树第三条流的 322 写与 290 写两段就是这样登的（同文件 `:25`）。**所以「精确全量」今天只对最长段不超过展开上限的流成立**：全量展开深度 `FULL_EXPAND_UP_TO=30`（`scripts/crash/crash-amplification.sh:336`），轻型 `LIGHT_EXPAND_UP_TO=6`（`scripts/crash/crash-point-replay.sh:246`）。

### 1.5 引擎怎么接一条新流（量的）

- 一条流 = `CrashFlow`（写表、段、被判根下标、版本表、展开函数、崩溃点列表，`crates/singlefs-checker-tier/src/crash_amplification.rs:234` 一带）；崩溃点是命名的写区间，几条流里同一个函数录出逐字节相同的一段就是同一个节点、共享判定（`crates/singlefs-checker-tier/tests/common_crash_points/mod.rs:1`）。手写流的步骤函数：`overwrite_step`（`:419`）、`content_needing`（`:44`）、把整条写表当一个崩溃点的 `one_crash_point_over_every_write`（`:848`）。
- 只数态数不判：`plan_crash_points`（`crates/singlefs-checker-tier/src/crash_amplification.rs:505`）按展开函数算出每点的态数区间；`SINGLEFS_CRASH_AMPLIFICATION_PLACEMENT_ONLY=1` 只排派活计划（`:12812`）。所以**状态数不用判就能数**，挂钟与库增量才要真判。
- 手写动作串 → `CrashFlow` 有先例：`record_history(tag, actions)` 把一串 `UserAction`（覆盖写、小文件、重开挂载、卸载再重开）建成池、录写表、切段，并按发布记每一版 `PublishedVersion { instance, checkpoint_txg, content }`（函数在 `crates/singlefs-checker-tier/tests/record_checker_judges_absence_by_the_persisted_set.rs:284`）；版本表的用途写作「崩溃放量的 oracle 按它认恢复落到的是哪一版」（同文件 `:191`）。随机历史**生成器** → `CrashFlow` 没有：`grep -rn CrashFlow crates/singlefs-checker-tier/src/crash_injection.rs crates/singlefs-harness/src | wc -l` 输出 0；`grep -rn generate_history` 在 `tests/crash_enumeration_*.rs` 与 `tests/crash_amplification_*.rs` 里 0 命中（`commands.out`）。版本表的类型是 `PublishedVersion`（`crates/singlefs-harness/src/memory_pool.rs:843`）。

### 1.6 随机历史一侧与设备一侧（量的）

- 随机历史：操作种类 17 种（`crates/singlefs-harness/src/history.rs:719`）；种子基永久固定（`:123`）；盘宽四档（`:128`，成员 `:148` 收尾）；`HistoryDeviceWidth::parameters` 对四档都取 `e142_parameters(512, 512)`（`crates/singlefs-harness/src/history.rs:222`），**物理块/io_min 这一轴历史取样不暴露**，io_min 8192 只有一条手写流（`crates/singlefs-checker-tier/tests/crash_enumeration_writable_mount_of_an_io_min_8192_pool.rs:46`）。
- 给定状态那条（登记行 `random-histories-and-fault-injection-given-states`，仓里登记的崩溃枚举用例中唯一在提交时真起的一条）：七段随机历史，其中三段是 32 种子×150 步（`crates/singlefs-checker-tier/tests/crash_given_states_of_random_histories_and_fault_injection_judged_through_the_library.rs:162`、`:163`），判的是每一步之后的镜像与注入点重开的盘，**不是段内崩溃子集**；登记行自述「标了 ignore，release 跑，单跑以十分钟计」（`.claude/gate.d/stage-inputs.tsv:71`，登记自述，未复核）。
- 崩溃注入（抽样）：`CrashPointDraw` 三种取法含 `EveryProperSubsetOfShortSegments`（短段整段枚举、长段抽样，`crates/singlefs-checker-tier/src/crash_injection.rs:239`、`:244`）；每个状态判三截（崩溃镜像、崩溃后可写挂载加发布、挂载途中二次崩溃）加理想模型「恢复到的这一版允不允许」（同文件头注释）；大档规模从环境变量取（`crates/singlefs-checker-tier/tests/crash_injection_campaign.rs:1557`），快档登记注释自述「debug 下单跑一百来秒」（`:431`）。它是用户已否过的抽样路线（「我们不是抽样 我们是剪枝 加 初期ignore」，`.claude/kb/decisions/13-验证路线.md:267`），但判法里的模型与可写挂载两截是引擎那一路没有的。
- 设备数：`make_filesystem` 硬断言两块（「第一版跑 2 块盘（D2（RAID 条带策略） 已定项 9）」，`crates/singlefs-core/src/make_filesystem.rs:479`）；两块时区域布局只许一种（`crates/singlefs-core/src/make_filesystem.rs:463`，常量 `:63`）。

### 1.7 提交时今天怎么处理崩溃枚举用例（量的）

- 阶段 `checker-tier-crash-point-replay` 两格：`crash-case-replay`（`.claude/gate.d/checker-tier-crash-point-replay.sh:9`）、`feature-gated-tests`（`:10`）。快档对登记的 29 条崩溃枚举用例（`commands.out`：`grep -c '^crash-case:' .claude/gate.d/stage-inputs.tsv` 输出 29）「快档逐条带 SINGLEFS_CRASH_CASE_BLOCKS_QUERY=1 起用例（只算块键、查库，一块都不判）」（`.claude/gate.d/checker-tier-crash-point-replay.sh:21`）：库里键相同且不红才算罩住，没判过的报本次未跑、退 0。**提交时不判任何新块**。
- 唯一例外是登记了 `red-blocks=judged-by-test` 的用例，快档照全量起（`.claude/gate.d/checker-tier-crash-point-replay.sh:24`）；今天只有 1 条带它（`grep '^crash-case:' … | grep -c 'red-blocks=judged-by-test'` 输出 1，就是上面那条随机历史给定状态用例）。
- 入口合表 `.claude/gate.d/crash-point-replay-items.tsv` 28 项四列（读法 `scripts/crash/lib_crash_point_replay_items.py:115` 的 `if len(fields) != 4:`），没有层这一列（该脚本里「层」字 0 处，`grep -c` 输出 0）；`crash-amplification.sh --list-items` 列 20 项（`commands.out`）。入口不带参数是剪枝跑全部项（`scripts/crash/crash-point-replay.sh` 文件头用法段），已按块键复用。

### 1.8 旧数（只当参考，没在今天的树上重量）

- 全量冷跑：`CRASH_AMPLIFICATION_TOTAL items=7 failed=0 states=2563727926 red=0 seconds=27288`（两台五张卡、展开 28；`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md:35`），同一行：「到 E 那一项平均每秒约 9.9 万态」。折成吞吐约 10^5 态每秒，之后判器、键、库都改过，**今天的吞吐没量**。
- 录入段是瓶颈：「录入那几个小时五张卡全空、推的约占整趟 78%」（`records/2026-10-13-复用键与全仓改名.md:1001`，原文标「推的」）；本机一份判段：「录入 508 秒、判段 663 秒」（`records/2026-10-13-复用键与全仓改名.md:942`）。
- CPU 逐态判：E161 第二条流段头 4096 态 `state_elapsed_ns=30213684776`（`research/results/e161-crash-state-dedup-and-time-split-feasibility-r2-2026-10-05.out:162`），约 7.4 毫秒每态，池级 checker 占九成。
- 轻型整趟（单机，W13–W16 合入后）：`CRASH_POINT_REPLAY_TOTAL tier=quick dispatched=17 collected=17 green=16 red=0 not_run=1`，同一行 `seconds=1856`（`records/2026-10-13-代码欠账批.md:207`）。提交时崩溃侧已有这一块基线，常规层要加在它上面。
- 库：`du -sh ~/.local/share/singlefs/crash-amplification/crash-state-verdicts` 输出 7.0G（量的，`commands.out`）；上面十一份闭式合计加固定脚本与多记录流约 1.2×10^10 态（推的，没扣共享前缀），粗约每态不到 1 字节，块数没读过（读法见 K2）。
- 提交改动频度（量的，`commands.out`）：2026-09-15 以来 196 次提交，碰 `crates` 的 35 次。判器单元键只在碰到判器闭包里的函数时变，这 35 是上界，不是变的次数。

## 二、Q1　常规层的负载取什么（K1 + K2）

四条候选。共同约束来自 1.2、1.3：上千写的态数由每次发布改写的单元数定，不由写条数定；来自 1.4：全新内容的块没有任何东西替它剪，常规层要「精确全量」就得让最长段落在展开上限之内，段太长的节点只能标 ignore 并报名字（今天报不了名字）。

### 候选「覆盖写链」

- 负载怎么生成：沿用标准前缀（mkfs、取号、暖机、第一个文件），之后循环调 `overwrite_step(…, content_needing(单元数, 种子), …)`（`crates/singlefs-checker-tier/tests/common_crash_points/mod.rs:419`、`:44`），每隔若干次插一次可写挂载步骤；内容单元数与种子逐次变，手写，不随机。
- 规模怎么控：录制器写条数 ≥ 1000 为止；每次覆盖写设一个崩溃点，n 大于阈值的发布包成 `ignored: true` 的点、其余全展开（模板是 1.4 的 σ 用例）；循环次数写成常量，用例里 `assert_eq!(writes.len(), WRITES)` 钉住，做法同 `crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_fixed_script_stream.rs:90`。
- 与已登记用例的重叠：前 11 段与固定脚本一类的流同形，同函数同数据的节点键相同、全复用；循环里的每次覆盖写与登记行 `overwrite-fixed-script-stream` 的五次回退后覆盖写是同一种发布，但内容、txg、起步镜像不同，块键不同，**不复用**；所以重叠的是写表形状，不是块。
- 判法：引擎一路（两遍恢复、oracle 按版本表认恢复落到哪一版、池级 checker、记录核对器，按 D13 已定项 17 判器单元键复用；组成见 1.4 引的 σ 用例注释）；没有观察者（`OBSERVERS_ON_THE_STANDARD_PREFIX` 只登了 acquire_instance_and_warm_up、publish_first_file、sequential_write 三个节点名，`crates/singlefs-checker-tier/tests/common_crash_points/mod.rs:282`，覆盖写节点不在内）。
- 要动 `crates/`：新增一个测试文件（`#[ignore]` 全量用例加一条快档小规模用例）；`common_crash_points/mod.rs` 可能加「循环 N 次覆盖写」步骤；不动 `src/`。新增判别力锚点要动 `crates/mutations.tsv`。
- 要动 `scripts/` 与登记：`.claude/gate.d/stage-inputs.tsv` 加一行 `crash-case:`；`.claude/gate.d/crash-point-replay-items.tsv` 加一项；放量一侧要的话在 `scripts/crash/crash-amplification.sh` 的 ITEMS 加一个分项。
- K2 缺口：态数——**今天没有长链的录制**。按 1.3，覆盖写取固定脚本里的 n = 28 时合计 8.3×10^9 态（推的），取 n = 16 时 3.1×10^6（推的）；链里的覆盖写落在哪一档要录一次才知道。

### 候选「种子随机历史整流枚举」

- 负载怎么生成：`generate_history_with_weights(种子, 步数, 比重)`（`crates/singlefs-harness/src/history.rs:1860`）按固定种子区间出历史，带录制地执行，录制流整条切段后包成 `CrashFlow`；崩溃点按操作切（名字取操作种类加下标，使两条历史在共同前缀上是同一个节点）；版本表来自理想模型提交的各版。
- 规模怎么控：每个种子的步数调到录制写条数 ≥ 1000，把每个种子的写条数钉成表（种子基永久固定 `crates/singlefs-harness/src/history.rs:123`，表才不漂）；给定状态那条已有 150 步×32 种子的现成取样（`crates/singlefs-checker-tier/tests/crash_given_states_of_random_histories_and_fault_injection_judged_through_the_library.rs:162`、`:163`）。
- 与已登记用例的重叠：**同生成器、同种子基、同比重**的有两条——给定状态那条（判每一步之后的镜像，本候选判步内崩溃子集，判的对象不同）、崩溃注入（同样是随机历史，抽样 vs 全枚举）；与手写流只共用前缀。
- 判法：引擎一路。oracle 认版本表，版本表由历史执行时理想模型提交的各版内容出（`record_history` 的做法，1.5），所以模型那一半不丢；丢的只有崩溃后可写挂载与二次崩溃那两截（`crates/singlefs-checker-tier/src/crash_injection.rs:18`：「崩溃点重放不跑可写挂载，这两截只在这里有」）。
- 要动 `crates/`：随机历史生成器 → `CrashFlow` 转换 crates 里没有（1.5，两条零命中），模板是 `record_history`（手写动作串），新写在 `crates/singlefs-checker-tier/src/crash_injection.rs`（或新模块）；新增测试文件；段长于展开上限的发布包成 `ignored: true` 的点（σ 用例的做法，1.4），并让驱动行报节点名（`crates/singlefs-checker-tier/src/crash_amplification.rs:12492` 一带的驱动行格式）。
- 要动 `scripts/` 与登记：同上，另加：`scripts/crash/crash-amplification.sh:163` 一段「类名取不出」改成报名字。
- K2 缺口：态数、挂钟、库增量**三样都没有数**。随机历史的段形（最长段、各段写数）今天没有任何产物；17 种操作里卸载、整理、截断、删除类会出 n = 28、30 的段（固定脚本的取值，`segment_sums.out`），含它们的历史会被 ignore 吃掉一大块，ignore 的占比决定「精确全量」还剩多少。

### 候选「生命周期闭环加长」

- 负载怎么生成：取 `lifecycle-closure` 的步骤序列（合表注释：分配 → 覆盖写 → 建快照 → 再覆盖写 → 删文件 → 整理搬共享块 → 删快照 → 回收 → 再分配，139 段，`.claude/gate.d/crash-point-replay-items.tsv` 的 lifecycle-closure 行）连跑 N 轮，轮与轮之间换内容种子。
- 规模怎么控：一轮 824 写（`crates/singlefs-checker-tier/tests/crash_enumeration_lifecycle_closure_stream.rs:68` 的段和），两轮 1648 写，写条数用常量钉住。
- 与已登记用例的重叠：**第一轮与 lifecycle-closure 逐字节相同**，块键相同、全部复用，等于把冒烟流再跑一遍；真正新增只从第二轮起（起步镜像不同，键不同）。若只取第二轮起算，这条是冒烟流的延长，不是另一类负载（K3 的靶子）。
- 判法：引擎一路加观察者（快照类、整理类观察者在 `tests/common_snapshot_crash_point_observers`、`tests/common_compaction_crash_point_observers`，随各步骤挂在节点上）。
- 要动 `crates/`：新增测试文件，调现成的 `common_compaction_prefix`、`common_snapshot_prefix` 步骤；不动 `src/`。`scripts/` 与登记同「覆盖写链」。
- K2 缺口：态数——一轮已知 2.89×10^8（`:77`），N 轮按段形相同推约 N×3×10^8，但第二轮起 n 取值未录；挂钟要吞吐（见下表）；库增量要读块数。

### 候选「崩溃注入精确抽取」

- 负载怎么生成：随机历史（同上），不换引擎：给 `CrashPointDraw` 加一种「展开上限内的段全部真子集，更长段登记 ignore」（现有 `EveryProperSubsetOfShortSegments` 对更长段是抽样，`crates/singlefs-checker-tier/src/crash_injection.rs:244`），仍判三截加理想模型。
- 规模怎么控：每个种子的操作数调到录制写条数 ≥ 1000；战役的环境变量已有种子数、步数、点数（`crates/singlefs-checker-tier/tests/crash_injection_campaign.rs:1557`）。
- 与已登记用例的重叠：与登记行 `crash-injection-fast-tier`、`read-faulted-states-never-hold-the-witness-criterion` 同生成器同判法，只是取法从抽样换成枚举；与给定状态那条同种子基。
- 判法：这是唯一带崩溃后可写挂载与二次崩溃两截的候选（`crates/singlefs-checker-tier/src/crash_injection.rs:18`：「崩溃点重放不跑可写挂载，这两截只在这里有」，每个崩溃状态判三截见同文件 `:15`）；快档自述 debug 下单跑一百来秒（`crates/singlefs-checker-tier/tests/crash_injection_campaign.rs:431`）。与引擎一路逐态成本的比没量；可写挂载一截在 CPU 上做是推的。
- 要动 `crates/`：`crash_injection.rs` 加取法并让战役认它；`crash_injection_campaign.rs` 加用例；`scripts/` 与登记同上。
- K2 缺口：逐态成本（可写挂载那一截）没有分段计时；态数同上没有。

### Q1 的 K2 一览（三样里哪样今天没有数）

| 候选 | 状态数 | 挂钟 | 判定库增量 |
|---|---|---|---|
| 覆盖写链 | 没有（要录一次长链；推的区间 3.1×10^6 到 8.3×10^9，1.3） | 没有（要今天的吞吐，见下） | 没有 |
| 种子随机历史整流枚举 | 没有（随机历史没有任何段形产物） | 没有 | 没有 |
| 生命周期闭环加长 | 一轮有（2.89×10^8，量的）；加长的轮没有 | 没有 | 没有 |
| 崩溃注入精确抽取 | 没有 | 没有；逐态成本比引擎一路高（可写挂载一截，无分段计时） | 没有 |

四条候选的挂钟都卡在「今天的冷吞吐没量」（1.8 只有 2026-09-30 的约 10^5 态每秒，之后判器与键都改过）。要建的计数与实测，按从便宜到贵：

- 计数探针（本机秒到分钟，不判）：对每条候选的流录一遍，打写条数、段数、最长段、两态闭式态数与 `plan_crash_points` 给的每点态数。落脚点：`closed_form_state_count`（`crates/singlefs-harness/src/memory_pool.rs:659`）与 `plan_crash_points`（`crates/singlefs-checker-tier/src/crash_amplification.rs:505`）都是现成的；探针本身是新写的 `#[ignore]` 测试或 `src/bin/` 小程序，`crates/` 里没有（`grep` 零命中见 1.5）。对随机历史还要打每个种子的写条数、最长段分布、段长超过展开上限的节点占比。这是各候选里**必须最先建**的一样，因为它决定「精确全量」还剩多少、ignore 吃掉多少。
- 吞吐与分段计时（重型，归主 agent）：在今天的树上对一个已有小项冷库跑，读驱动打的 `CRASH_AMPLIFICATION_ITEM … states=… seconds=…` 与录入段、判段各自用时。可用的小项：`snapshot-creation`（一次 1.68×10^7 态，上表）、`multi-record-publish`（2.68×10^8 态，`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md:35` 里 268435552）。
- 库增量：同一趟结束后读库块数与字节。读库程序 `crates/singlefs-checker-tier/src/bin/crash_library_reader.rs` 已有，`crash-point-replay.sh --list-items --with-keys` 会起它（文件头用法段）；对一个新建的库目录跑前后各读一次，得字节每态。

**什么会推翻上面的缺口判断**：如果计数探针在 `grep` 里能在现有产物里找到（`research/results/` 里已有随机历史的段形），则「态数没有」不成立；今天的 `grep -rn generate_history` 对 `tests/crash_enumeration_*.rs` 与 `tests/crash_amplification_*.rs` 0 命中，对 `research/results/` 我没逐份查，**只查了 e161 一份产物的字段名**（`awk '{print $2}'` 列出的 `name=` 种类里没有随机历史段形）。

## 三、Q2　大负载层的负载与「多设备档位交叉」

先把「档位」在今天实现上能取的轴列出来（量的，1.6）：

| 轴 | 今天取得到的档 | 出处 | 取不到的 |
|---|---|---|---|
| 设备块数 | 恒 2 块 | `crates/singlefs-core/src/make_filesystem.rs:479` 硬断言 | 1 块、3 块以上；三块以上已挪出里程碑三（`.claude/kb/milestone/03-third-txn.md:109`） |
| 盘宽 | 4 档，两块恒等大 | `crates/singlefs-harness/src/history.rs:126`、`:128` | 两块不等大（注释明说模型只有一个盘大小） |
| 物理块 / io_min | 512/512（历史取样）；8192 只有一条手写流 | `crates/singlefs-harness/src/history.rs:222`、`crates/singlefs-checker-tier/tests/crash_enumeration_writable_mount_of_an_io_min_8192_pool.rs:46` | 历史取样里要加参数才取得到别的值 |
| 区域布局 | 一种 | `crates/singlefs-core/src/make_filesystem.rs:463` | 其余布局 |
| 盘间不对称 | 一盘最新系统配置槽读回全 0、掉电落在第一个 journal 记录写 | 操作种类 `crates/singlefs-harness/src/history.rs:536`、`:542` | 只在这两种操作的比重组里出现，其余比重组不触发 |
| 介质 | 无 | `grep -rn "RotationalPtr\|ZonedPtr" crates \| wc -l` 输出 0（`commands.out`，D13 已定项 10 自己也这样写） | zoned、旋转 |

### 候选「盘宽乘 io_min 交叉」

- 负载：Q1 选定的常规层负载，原样重放在 盘宽 4 档 × io_min {512, 8192} = 8 格上；N≥5 轮读成五个不同种子或内容。
- 规模：写条数不变，但小盘（单元区 384/256/240 槽）上空槽少、回收更早、落点拒绝走得到，同一条脚本的段形（n）会变——今天没有数。
- 重叠：4 GiB 一档与 Q1 候选逐字相同（重复）；小盘三档与登记行里的小盘用例（给定状态那条的 `toward-the-unit-area-wall` 两段，在 `crates/singlefs-checker-tier/tests/crash_given_states_of_random_histories_and_fault_injection_judged_through_the_library.rs:160` 一带）在生成器上重叠，判的对象不同。io_min 8192 与登记行 `writable-mount-of-an-io-min-8192-pool` 重叠（该流 33 态，`crates/singlefs-checker-tier/tests/crash_enumeration_writable_mount_of_an_io_min_8192_pool.rs:50`）。
- 判法：引擎一路。
- 要动 `crates/`：`HistoryDeviceWidth::parameters` 或新增一个「几何档」参数让历史取样带上 io_min（`crates/singlefs-harness/src/history.rs:222`）；手写链则在 `tests/common` 的 `parameters()` 处加档。`scripts/`：登记行按「负载 × 档」展开，合表要有「档」这一列（合表今天 4 列，`scripts/crash/lib_crash_point_replay_items.py:115`）。
- K2 缺口：8 格各自的态数都没有；格数 8 倍乘上每格态数，是大负载层挂钟的第二个乘数。

### 候选「盘间不对称故障档」

- 负载：Q1 负载 × 故障档 {无、一盘最新系统配置槽读回全 0、掉电落在第一个 journal 记录写}，直接用现成的两种操作（`crates/singlefs-harness/src/history.rs:536`、`:542`）与 `FaultInjectingBlockDevice`。
- 规模：写条数同 Q1；故障只在注入点上，状态数是 Q1 的态数加注入点数，不乘。
- 重叠：与登记行 `read-faulted-states-never-hold-the-witness-criterion`、`random-histories-and-fault-injection-given-states` 的故障部分重叠（同种操作、同比重组）。
- 判法：故障后的红由用例分进「已知红」「说谎设备」两张清单（`.claude/gate.d/stage-inputs.tsv:71`），**不是引擎一路的「0 红」**，所以这一档的「精确全量」判不出（K3 靶子）。
- 要动 `crates/`：新增测试文件，沿用 `classify_the_pool_checker_verdicts_of_the_images_after_a_fault`（给定状态那条文件头 import）；`scripts/` 与登记同上。
- K2 缺口：态数小、挂钟小，但可写挂载一类的逐态成本没有分段计时（同 Q1 最后一条）。

### 候选「放大到上万写」

- 负载：把 Q1 负载放大到 ≥ 10^4 条写（大规模取 10 倍，**规则没说大规模是几倍**）。
- 规模：n = 16 的最小发布也要 477 次发布、31265919 态；n = 28 要 304 次发布、81604381968 态（推的，算式与输出见第九节 `chain_states.out`）。一次发布 n ≥ 24 就是 10^8 量级，吞吐按旧数 10^5 态每秒算是 10^3 秒量级每次发布。
- 重叠：包含 Q1 负载（前缀与已判块全复用）。
- 判法：引擎一路；最长段超过展开上限的节点只能 ignore 或按段登记，登记段**不是精确全量**（1.4）。
- 要动：同 Q1，另加分片枚举的登记（`.claude/gate.d/crash-point-replay-segments.tsv` 五列，需用户点头，文件头明写）。
- K2 缺口：态数取决于 Q1 的计数探针；挂钟在不 ignore 时以天计（推的）。

### 候选「取不到的档位登记为等实现」

- 不是负载，是登记：对 1 块、3 块以上、其余区域布局、zoned 四个取不到的档，在大负载层的登记里各写一行，状态写「等实现」，前置写它挡在哪一道（make_filesystem 的断言、`RotationalPtr`/`ZonedPtr` 零命中）。已定项 11 已给过形态「节点登记表带状态字段（开着 / ignore / 等实现）」（`.claude/kb/decisions/13-验证路线.md:330`），同一行说明这几条「被攻过零轮」「不写成定案」。
- 重叠：无。判法：无（不跑）。
- 要动：登记表加一列状态；门禁一格把「等实现」逐行列为本次未跑。`crates/` 不动。
- K2：无代价；缺口是「谁来保证等实现的行不被遗忘」，已定项 11 的三条补充条件（每个 ignore 行点名一个会变的对象、那个对象变绿而节点仍 ignore 时判红、覆盖报告列 ignore 的状态数与占比）今天一条都没落地。

### 「N≥5 轮」的读法（规则没说，并排）

- 「常规层 负载 + 大规模负载 + 多设备档位交叉，N≥5 轮，精确全量」（`.claude/kb/decisions/13-验证路线.md:248`）。
- 已定项 10 标题：真正必须精确覆盖的 N 不是「布局套数」，是「提交协议结构的等价类数」（`.claude/kb/decisions/13-验证路线.md:274`），并写「需为它新开一轮独立大负载层 重放」（`.claude/kb/decisions/13-验证路线.md:287`）。
- D17 已定项 2：「等价类数 = 已开线的结构等价类数 × 失败模型数，现值 1 × 1 = 1，每开一条线重算」（`.claude/kb/decisions/17-实现分层与第三方管道.md:39`）。
- 两读法：N 是轮数（五个种子或五组内容）；N 是等价类数（今天 1，达不到 5）。**两读法给的门槛不同，规则没说取哪一个**。顺带：D17 写步骤种类「六种」，`StepKind` 今天 7 个成员（`commands.out` 里 awk 数得 7；`crates/singlefs-harness/src/segments.rs:25`），`CommitStep` 5 个（`crates/singlefs-core/src/transaction.rs:186`），三处数字互不相同。

**什么会推翻 Q2 的轴表**：如果有别处的建池入口接受 ≠ 2 块，则「恒 2 块」不成立。我跑了 `grep -rn 'fn make_filesystem' crates/singlefs-core/src`，建池入口只有 `make_filesystem` 一个（`crates/singlefs-core/src/make_filesystem.rs:472`）；`grep -rn 'devices.len()' crates/singlefs-core/src` 另命中的 `transaction.rs`、`mount.rs`、`admission.rs` 是按盘数循环或取个数，没有一处是对盘数的断言，别的入口没追。

## 四、Q3　落在哪

登记位事实：待优化第 13 行写的改法是「新装置并进已有的那一套当它的加速或扩展，不另起名字、不另立阶段」（`.claude/kb/milestone/03-third-txn.md:193`）。「立阶段」指新建一份 `.claude/gate.d/*.sh`，还是在已有阶段里加一格，2026-10-18 的记录（`records/2026-10-18-里程碑三收尾与近期欠账.md:49`，主 agent 记的，不是用户逐字）写的是「常规层、大负载层在门禁里立阶段」，**规则没说**这两种算不算。

### 候选「并进现有入口，门禁里加格」

- 做法：合表加一列「层」（冒烟 / 常规 / 大负载）；`crash-point-replay.sh` 加按层选项（今天的用法段有 `--item`、`--full`、`--quick`、`--list-items`、`--local-only`，没有层）；门禁阶段 `checker-tier-crash-point-replay` 在文件头的两行 `# gate-cell:`（`.claude/gate.d/checker-tier-crash-point-replay.sh:9`、`:10`）之后加一格，格名表同步（`:212`）。
- 与待优化 13 的关系：一个入口、一个阶段，合拍。
- 要动：`scripts/crash/lib_crash_point_replay_items.py`（`:115` 的 `!= 4` 改五列，核法与自证）、`scripts/crash/crash-point-replay.sh`、`scripts/crash/crash-case-check.py`（placement）、`.claude/gate.d/checker-tier-crash-point-replay.sh`（格、样本档）、`.claude/gate.d/crash-point-replay-items.tsv`、`.claude/gate.d/stage-inputs.tsv`；`.claude/skills/crash-test/commit-stages.md` 的描述句。`crates/` 只动 Q1 选定的那一份。
- K2 缺口：加一格不加 cargo 起跑次数，但格内的录制与块键计算每次提交都做（见 Q4）；一格的固定开销没有数。

### 候选「另立阶段」

- 做法：新建 `.claude/gate.d/checker-tier-crash-regular-layer.sh`（头部要写 `admission:`、`gate-stage:`、`gate-fixture:`、`gate-category:`、`gate-covers:`、`gate-cell:`，仿 `.claude/gate.d/checker-tier-crash-point-replay.sh:1` 起的文件头），阶段内调同一个入口脚本的 `--layer` 选项。
- 与待优化 13 的关系：这是字面上的「另立阶段」；只要阶段不带自己的驱动、只转调入口，装置仍是一套，但文字上撞第 13 行，需要用户明说例外。
- 要动：新阶段文件、样本档、`.claude/gate.d/stage-inputs.tsv` 的阶段行、`gate-similar` 说明（`.claude/gate.d/checker-tier-crash-point-replay.sh:7` 那样的查重声明）、`.claude/skills/crash-test/commit-stages.md` 的「六道」改写；入口与合表同上一条。
- K2 缺口：多一个阶段多一次门禁整轮的启动与 cargo 复用判定（按条复用要先 `test-plan`），这一项固定开销没有数。

### 候选「只并进入口，门禁一格不加」

- 做法：合表加层列、入口加 `--layer`，提交时门禁不变（崩溃枚举用例仍只查库，1.7），常规层由主 agent 在提交前起入口。
- 与「立」的关系：门禁里没有这一层的格，C409 的欠块（「常规层、大负载层在门禁里没有阶段」，`.claude/kb/decisions/13-验证路线.md:261`）不会因此消掉，只能算候选「并进现有入口，门禁里加格」的第一步。
- 要动：入口与合表，不动门禁阶段。
- K2：同 Q4。

**什么会推翻 Q3 的触点清单**：合表读法只有 `lib_crash_point_replay_items.py` 一处（`grep -rn "crash-point-replay-items" scripts .claude/gate.d` 我没逐个核有无第二处读它的脚本）。有第二处读者的话，加列要同步改它。

## 五、Q4　触发与挂钟

事实（1.7、1.8）：提交时崩溃枚举用例**不判任何新块**，只查库（唯一例外 1 条带 `red-blocks=judged-by-test` 的用例，登记自述单跑以十分钟计）；已定项 9 写「全量档提交时不默认跑、不限时刻，由用户要求时跑；提交前 = 快档全绿（必要条件）。」（`.claude/kb/decisions/13-验证路线.md:253`），常规层那一行写「每次提交」（`.claude/kb/decisions/13-验证路线.md:247`）。**两处并排**：「每次提交」今天对任何登记用例的实现意思是「查库、没判过的报本次未跑」，并不是「判」。规则没说常规层的「每次提交」是哪一种。

### 候选「提交时按键增量判」

- 做法：常规层各行按全量的方式起（库照键读写、不带块查询），与那一条 `judged-by-test` 用例同一条路径（`.claude/gate.d/checker-tier-crash-point-replay.sh:24`）；键对不上的块现判。
- 与全量并存：`--full` 是只写不读（`--start-over`，`.claude/gate.d/checker-tier-crash-point-replay.sh:23` 一带）把所有行（含常规层）重判；常规层的提交时跑法与全量写同一份库（D13 已定项 17：每类判定一份库），所以全量跑过的块提交时全复用。大负载层的行只在 `--full` 与用户要求时起。
- 挂钟：冷（第一次）= 态数 ÷ 今天的吞吐 + 录入段（旧数：10^5 态每秒量级、录入占大头，1.8，**今天没量**）；热（每次提交）= 重新录制所有常规层流的时间（块键要写表，每次都要录，旧数每条大流约 480–520 秒，`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md:35`）+ 查库 + 键对不上那几块的判。
- 键对不上的频度：判器闭包里的函数一变，该判器下所有块作废，热等于冷。提交频度上界：2026-09-15 以来 196 次提交碰 `crates` 的 35 次（量的）；真正动到判器闭包的次数没数。

### 候选「提交时只查库，作废了再请主 agent 起」

- 做法：提交时仍是今天的查库；阶段打出作废块数与变了的判器单元清单（已定项 9 对全量已写了这个要求，`.claude/kb/decisions/13-验证路线.md:256`），作废数 > 0 时主 agent 起常规层，没动判器则零开销。
- 与全量并存：全量触发条件「动了崩溃点重放或判器」与这里的触发条件是同一个判据（作废块里有判器单元变了），常规层是更勤的一档。
- 挂钟：未动判器的提交只付录制 + 查库；动了判器的提交付冷成本。
- 代价：「每次提交」被降成「判器变了的提交」，与 D13 原文有出入，要用户点头。

### 候选「提交时固定预算，超出标 ignore」

- 做法：先声明挂钟预算（输入不是输出：「T1（常规层 挂钟预算） 是输入，不是输出」，`.claude/kb/decisions/13-验证路线.md:321`），常规层各节点登记状态「开着 / ignore / 等实现」，预算装得下的开着，装不下的 ignore 并**逐个报名字**；用户按「慢慢开」逐个开，每开一个记状态数（`.claude/kb/milestone/03-third-txn.md:145` 的六·2 行：「ignore 清单条数只减不增」）。已定项 11 否决了「按欠账轮转的账本」调度（`.claude/kb/decisions/13-验证路线.md:308`），这里**不轮转**，开关全由人手。
- 与全量并存：全量把 ignore 的节点也开着判（ignore 是常规层的预算，不是全量的）。
- 挂钟：上界 = 预算（一个声明值，不写进 kb，`.claude/kb/decisions/13-验证路线.md:328`）；代价是 ignore 占比，而 ignore 占比要等 Q1 的计数探针才有。
- 前提：驱动行报节点名（1.4 的缺口）、登记表加状态字段（已定项 11 称零轮被攻、不写成定案）。

### Q4 的 K2

| 量 | 今天有没有 | 要建什么 |
|---|---|---|
| 冷吞吐（态每秒，今天的树） | 没有（旧数 10^5 量级） | 在今天的树上冷库跑一个已有小项，读驱动打的 `states=`、`seconds=`、录入与判段各自用时（重型，归主 agent） |
| 热跑的固定开销（只录制 + 查库） | 没有 | 给新用例做完后用 `SINGLEFS_CRASH_CASE_BLOCKS_QUERY=1` 跑一遍，读挂钟；这不判任何块 |
| 判器闭包被提交碰到的频度 | 只有上界 35/196 | 对最近 K 次提交各算一遍判器单元键与库里键的对不上块数（`crash-point-replay.sh --list-items --with-keys` 已打「键对不上」块数，文件头用法段）；脚本把它按提交循环 |
| 轻型基线 | 有（旧数）：整趟 1856 秒 | 常规层挂在它之上，每次提交的总挂钟 ≥ 这个数 + 常规层增量（`records/2026-10-13-代码欠账批.md:207`） |

**什么会推翻 Q4 的「提交时不判新块」读法**：如果门禁实现里别处还有会判新块的路径（我只读了文件头的说明段与登记 1 行，没读 `admission.py` 的 `crash-case-command` 全文），则 1.7 要改。

## 六、Q5　C409 关账判据（一条）

先并排两处冲突的文字：

- C409 现行「怎么拦」写的是抽样与分桶：「按崩溃点所在子阶段分桶抽样」、「把常规层的抽样数设成 0，阶段必须判红而不是报绿」（`.claude/kb/checks-owed.md:294`）。
- 已定项 9 的用户定案是不抽样：「我们不是抽样 我们是剪枝 加 初期ignore」（`.claude/kb/decisions/13-验证路线.md:267`），常规层那一行写「精确全量，枚举不完靠精确剪枝，一时跑不了的节点标 ignore」（`.claude/kb/decisions/13-验证路线.md:247`）。

所以 C409 行里的判别力自证（抽样数设成 0）要换成下面第三条的写法；这是 C409 行自己要改的字，不是本轮的判据。

**判据**：一层算还清，当且仅当同时满足——

1. 负载登记：这一层至少一行 `crash-case:`（`.claude/gate.d/stage-inputs.tsv`）与合表一项（带层标记），`scripts/crash/crash-case-check.py` 的 placement 绿；常规层那一行的用例断言自己录出的写条数 ≥ 1000 并钉成常量；大负载层登记里「多设备档位」逐档一行，取不到的档写状态「等实现」。
2. 门禁一格：`.claude/gate.d/checker-tier-crash-point-replay.sh` 里（或另立阶段，取决于 Q3）有一格判这一层，成功行报「开着 / ignore / 等实现」各多少个节点和每个 ignore 节点的名字，没有登记行时退 77（本次未跑）而不是报绿，开着的节点有红退 1，ignore 数比上一次多退 1（沿用 `.claude/kb/milestone/03-third-txn.md:145` 的「ignore 清单条数只减不增」）。
3. 判别力样本各一：每层两个样本——门禁样本档里造一个「这一层的节点全标 ignore」或「没有登记行」，该格必须红（或退 77），不许绿；崩溃一侧造一条已知有违例的流（例：录制流里吞一道屏障，D13 已定项 4 的依据已有这样的对照，`.claude/kb/decisions/13-验证路线.md:145`），该层的判器必须报红。

K2：登记与格是纯登记工作，不需要吞吐；第 3 条的崩溃一侧样本要吞吐很小（一条小流），可以先做。

**什么会推翻这条判据**：如果「门禁一格」在用户的口径里必须是独立阶段文件（Q3 规则没说的那一问），第 2 条的落点要换；如果常规层的 ignore 占比在计数探针之后大到半数以上，第 1 条「精确全量」只剩名义——那时判据要加一条 ignore 占比上限，这个上限是「T1」一类数，已定项 11 禁止现在写进 kb。

## 七、并排的冲突与规则没说（汇总）

| 项 | 一边 | 另一边 | 判定 |
|---|---|---|---|
| 常规层「每次提交」 | 「全量档提交时不默认跑、不限时刻，由用户要求时跑；提交前 = 快档全绿（必要条件）。」（`.claude/kb/decisions/13-验证路线.md:253`） | 「快档逐条带 SINGLEFS_CRASH_CASE_BLOCKS_QUERY=1 起用例（只算块键、查库，一块都不判）」（`.claude/gate.d/checker-tier-crash-point-replay.sh:21`） | 规则没说「每次提交」是判还是查库；实现今天对登记用例只查库 |
| 大负载层「多设备档位交叉」 | 「常规层 负载 + 大规模负载 + 多设备档位交叉，N≥5 轮，精确全量」（`.claude/kb/decisions/13-验证路线.md:248`） | 「第一版跑 2 块盘（D2（RAID 条带策略） 已定项 9）」（`crates/singlefs-core/src/make_filesystem.rs:479`） | 冲突：块数这一轴今天恒 2，取不到的档要登记为等实现 |
| 「N≥5」 | 同上 :248 | 「等价类数 = 已开线的结构等价类数 × 失败模型数，现值 1 × 1 = 1，每开一条线重算」（`.claude/kb/decisions/17-实现分层与第三方管道.md:39`） | 规则没说 N 是轮数还是等价类数；按后一种读，门槛今天满足不了 |
| 「精确全量」与长段 | 「常规层 负载 + 大规模负载 + 多设备档位交叉，N≥5 轮，精确全量」（`.claude/kb/decisions/13-验证路线.md:248`） | 「登记了的段，引擎在重型里只按轻型那两个域的并展开」（`.claude/gate.d/crash-point-replay-segments.tsv:2`） | 冲突：按段登记的段不是精确全量；上万写的负载几乎必出长段 |
| 覆盖报告逐个列本次未跑 | 常规层一行末尾「覆盖报告与门禁逐个列」（`.claude/kb/decisions/13-验证路线.md:247`） | 「驱动行只报点数与态数、不报类名」（`scripts/crash/crash-amplification.sh:163`） | 冲突：今天只报个数 |
| 抽样与剪枝 | C409 行：「按崩溃点所在子阶段分桶抽样」（`.claude/kb/checks-owed.md:294`） | 「我们不是抽样 我们是剪枝 加 初期ignore」（`.claude/kb/decisions/13-验证路线.md:267`） | 冲突（kb 内部）：C409 的判别力自证仍写抽样数 |
| 给 T1 定数 | 「T1（常规层 挂钟预算） 这三个数一个都不许现在写进 kb」（`.claude/kb/decisions/13-验证路线.md:328`） | K2 要量挂钟 | 不冲突：量的数写报告与产物，不写 kb |
| 步骤种类个数 | D17 已定项 2 写「六种」（`.claude/kb/decisions/17-实现分层与第三方管道.md:38`），03 里写 `StepKind` 6 个成员（`.claude/kb/milestone/03-third-txn.md:150`） | `StepKind` 今天 7 个（`crates/singlefs-harness/src/segments.rs:25`，`commands.out` 数得 7）、`CommitStep` 5 个（`crates/singlefs-core/src/transaction.rs:186`） | 冲突：三处数不同，D17 的数滞后；只在「N 读作等价类」那一读法里有用 |
| 立阶段 | 「常规层、大负载层在门禁里立阶段」（`records/2026-10-18-里程碑三收尾与近期欠账.md:49`） | 「新装置并进已有的那一套当它的加速或扩展，不另起名字、不另立阶段」（`.claude/kb/milestone/03-third-txn.md:193`） | 规则没说「立阶段」是新文件还是加格 |

## 八、要主 agent 跑的命令（草案，我没跑）

下面两样是重型，归主 agent、用户要求时跑；计数探针要先有新写的代码（Q1 选定之后由实现员建），所以这一轮没有可直接起的命令。

- 冷吞吐与分段计时：对一个已有小项在新建的空库目录上跑一趟。形态：`SINGLEFS_HEAVY_TESTS=user-request`、带范围声明、经 `bash scripts/hosts/run-with-memory-cap.sh <上限>` 与 `bash scripts/hosts/capped.sh 31`，命令 `bash scripts/crash/crash-amplification.sh --item snapshot-creation --library <新建的绝对路径目录> --label c409-throughput`（参数写法见 `scripts/crash/crash-amplification.sh` 文件头用法段）。注意这个入口会按多机配置清场（文件头「清场」一段），是否起由主 agent 与用户定。跑完看哪几行：`CRASH_AMPLIFICATION_ITEM item=snapshot-creation … states=… seconds=…`，再 `du -sh <目录>` 得库字节；`snapshot-creation` 是 1.68×10^7 态（上表），`multi-record-publish` 是 2.68×10^8 态，两点够出斜率。
- 每次提交判器闭包被碰到的频度：对最近若干次提交各算一遍键对不上的块数，用 `scripts/crash/crash-point-replay.sh --list-items --with-keys`（会起只读读库程序，`crates/singlefs-checker-tier/src/bin/crash_library_reader.rs`）。

## 九、命令与原样输出

- `python3 -I /home/fy5090/.local/share/singlefs/work/m3-close-c409-r1-sonnet/segment_sums.py /home/fy5090/code/singlefs`：输出在 `segment_sums.out`，1.2 的表原样取自它。
- `python3 -I /home/fy5090/.local/share/singlefs/work/m3-close-c409-r1-sonnet/chain_states.py`：输出在 `chain_states.out`。1000 条写的一段：n=16 → 48 次发布 3146256 态；n=20 → 40 次 41943480；n=24 → 35 次 587202945；n=28 → 31 次 8321499477；n=30 → 29 次 31138513215。10000 条写的一段：n=16 → 477 次 31265919 态；n=28 → 304 次 81604381968；n=30 → 286 次 307090164810。
- 其余命令（`grep -c`、`wc -l`、`git log`、`du`）的原样输出在 `commands.out`，1.4、1.5、1.7、1.8 引到的数都取自它。

## 十、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1 Q1 | 一致（四条候选） | 覆盖写链、种子随机历史整流枚举、生命周期闭环加长、崩溃注入精确抽取；上千写的态数跨 4 个量级，由每次发布改写的单元数定 |
| K2 Q1 | 规则没说 / 缺数 | 态数、挂钟、库增量三样对四条候选都没有今天的数；最先要建的是计数探针，不判、本机秒到分钟 |
| K1 Q2 | 冲突 | 块数轴恒 2 块（`crates/singlefs-core/src/make_filesystem.rs:479`），多设备档位只取得到盘宽×io_min 与盘间不对称两条；其余登记为等实现 |
| K2 Q2 | 缺数 | 8 格各自的态数没有；上万写在 n ≥ 24 时以天计（推的） |
| K1 Q3 | 规则没说 | 并进入口加格 / 另立阶段 / 只并进入口三条；「立阶段」是新文件还是加格没写 |
| K2 Q3 | 缺数 | 加一格或一个阶段的固定开销没有数 |
| K1 Q4 | 冲突 | 「每次提交」在实现里是查库不判；三条触发候选并排 |
| K2 Q4 | 缺数 | 冷吞吐（今天）、热跑固定开销、判器闭包被碰到的频度三样没有；只有上界 35/196 与轻型基线 1856 秒（旧数） |
| K1 Q5 | 冲突 | C409 现行判别力自证写抽样数，与不抽样的定案并排冲突；给了一条三合一的判据 |
| 附带 | 冲突 | 「精确全量」与按段登记的段不是同一回事；覆盖报告只报个数不报名；步骤种类个数三处不同 |

## 十一、没做什么

- 没判别的腿的格（K3、K4 归攻方腿）；没替主 agent 采纳候选、没选 Q3 的落点、没给用户的岔路单（`research/prompts/m3-close-c409-r1-forks.md` 归主 agent）。
- 没编译、没跑崩溃枚举、没起 GPU；所有「今天的吞吐」「今天的段形」都是缺数，不是我估的数。1.3、Q2、Q4 里的态数是由已入库常量算出的推的数，明写了「推的」。
- 没读禁读清单里的三份文件（`research/prompts/m3-close-c409-r1-opus-output.md`、`…-main-verification.md`、`…-forks.md`）。
- 背景材料 `_m3-close-c409-r1-background.md` 1.4 MB 没整读，kb 条款全部回到 `.claude/kb/` 里按行号现查；`_m3-close-c409-r1-body.md` 整读。
- 没逐份查 `research/results/` 里有没有随机历史的段形产物（只查了 e161 一份）；没读 `scripts/gate/admission.py` 的 `crash-case-command` 全文；没核合表读者是不是只有 `lib_crash_point_replay_items.py` 一处。
- 旧数（2026-09-30 全量冷跑、E161 的 CPU 逐态耗时、2026-10-13 的录入占比）只当参考，没在今天的树上重量。
