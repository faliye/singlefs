# m3-conc-r1 正推腿报告（Sonnet，第一轮，2026-10-01）

腿：three-way-forward，立场正推：从已定条款、E166 / E167 / E169 的产物行与冻结副本推出每一格「现在定 / 转里程碑四」。
背景材料：`research/prompts/_m3-conc-r1-background.md`；问题单 `research/prompts/m3-concurrency-questions.md`；冻结副本（路径见第 0 节）。
接手：前两条腿被停掉时没写报告；它们留在模型目录的十个脚本我全部在 `run-with-memory-cap.sh` 下复跑过（下面「复跑」一节），报告里的数以这一次复跑为准。

## 0　读法

- 引用写法：「原文」（`路径:行号`）。**代码行号取自冻结副本**（`crates/…` 在 `/tmp/claude-1000/m3-conc-r1/tree/` 下），kb 与产物行号取主仓现查。
  事实表里的行号和今天对不上（kb 与副本都改过）：例如 D13 已定项 2 的定案句现在在第 51 行（事实表写 45），`persist_the_publish_or_freeze_it` 现在在 `transaction.rs:1144`（事实表写 1139）。引用请只认本报告的行号。
- 「推的」指没有产物行、没有命令输出的推导；「模型」指模型目录里的独立小模型（不与 `crates/` 共用代码）。
- 每格给：判、定案句（或转过去时带什么）、依据、要改的条款与 `crates/` 落点、什么现象会推翻它。被攻轮数一律「零轮」（第一轮，没人攻过）。

## 1　实现今天的样子（我读过的 crates/ 路径与看到的事实，主 agent 可复核）

1. 提交串行靠 `&mut` 借用，不靠锁：`publish_the_change_once` 每次现开一个写入口（「let mut writer = PoolWriter::new(parameters, devices);」（`crates/singlefs-core/src/mounted_session.rs:188`）），成功后会话换 current（「*current_file_version = published;」（`crates/singlefs-core/src/mounted_session.rs:314`））。
2. txg、jsn 计数器、事务号都从上一版 `previous` 现算，没有发号器：「let txg = checkpoint_txg_of_the_publish_after(previous.root.checkpoint_txg)?;」（`crates/singlefs-core/src/transaction.rs:4754`）、「counter: previous.record.counter + 1,」（`crates/singlefs-core/src/transaction.rs:4760`）、「transaction: previous.highest_transaction_number_in_this_instance + 1,」（`crates/singlefs-core/src/transaction.rs:4761`）。
3. 准备阶段整段在号序里：`prepare_the_version_publish` 收「allocator: &mut PoolAllocator,」（`crates/singlefs-core/src/transaction.rs:5105`）与 `previous`，`publish_admitted` 里「// 落点先于内容：分配记录树要装自己那条（D3（空间分配） 已定项 5），所以这次重写的落点在装任何单元之前全部取定。」（`crates/singlefs-core/src/transaction.rs:6153`）。
4. 分配器按根环顺序记账并断言：「装了根环表的进程写的根 txg 逐个加一、每一条都经分配器记」（`crates/singlefs-core/src/allocator.rs:1644`）；同一时刻至多冻结一次发布：「frozen_publish: Option<Box<FrozenPublish>>,」（`crates/singlefs-core/src/allocator.rs:1059`），落盘中途失败时「allocator.freeze_publish(FrozenPublish {」（`crates/singlefs-core/src/transaction.rs:1156`）。
5. 一次发布内部已经有多个事务（顺序写切成 N 个事务共用一个 txg）：「文件版本有 N 个数据单元时这次发布是 N 个事务（切分纪律一事务一单元），事务号 `transaction .. transaction + N` 连号，」（`crates/singlefs-core/src/transaction.rs:3583`）。
6. 提交步骤只有五种：「pub enum CommitStep<'publish> {」（`crates/singlefs-core/src/transaction.rs:122`）；一次发布的落盘顺序在 `persist_publish_writes`（`crates/singlefs-core/src/transaction.rs:1057`）。
7. 读侧：`core/src` 里 Mutex / RwLock / Arc / thread / Send / Sync 零命中（命令与输出见第 12 节）；只读挂载态「observation_since_open: Cell<ReadPathObservation>,」（`crates/singlefs-core/src/mounted_read.rs:327`）；设备读是 `&self`、写是 `&mut self`（「fn read_at(」（`crates/singlefs-core/src/block_device.rs:116`）、「fn write_at(」（`crates/singlefs-core/src/block_device.rs:121`））。
8. 准入读的是分配器此刻的计数，在飞 overlay 没有实现：「allocated: BytesOnOneDevice::of_slots(device_map.allocated_slots()),」（`crates/singlefs-core/src/admission.rs:329`）、「（在飞 overlay 没有实现，C87（准入读的合成值无定义））」（`crates/singlefs-core/src/admission.rs:314`）；准入调用在「&admission_reading_before_a_publish(allocator, previous),」（`crates/singlefs-core/src/transaction.rs:5183`）。
9. 写入口已经拒删除中的 inode（plan 时看 `previous`）：「fn refuse_writing_the_file_while_its_inode_is_being_deleted(」（`crates/singlefs-core/src/transaction.rs:4915`），覆盖写与顺序写各调一次（`transaction.rs:4753`、`transaction.rs:4807`）。
10. 前缀核的 spec 不要求相邻两次发布 txg 连号：上一条是末条时只要求「record.ordinal_within_publish.0 == 1」（`crates/singlefs-replay-prefix/src/lib.rs:230`），换 txg 的约束只在「一次发布之内」（「&&& record.checkpoint_txg == previous_record.checkpoint_txg」（`crates/singlefs-replay-prefix/src/lib.rs:232`））。
11. C58 的计数器与报警零实现：`grep -rnE 'checkpoint_entr|crossover|\bC58\b|write.?buffer|sort.?merge|WriteBuffer' crates --include=*.rs` 命中 0 行（第 12 节）。E17 的「条目」是 write buffer 的 sort-merge 输入，这个实现里没有 write buffer。

## 2　C1　Q1「并行的是哪一段」

**判：转里程碑四。** 问题单 Q1 的够判条件有两项：吞吐与每次 fsync 的等待（C488），各候选要改的提交步骤枚举成员。第一项在仓里没有一个本机数，所以不能在里程碑三记成定案。

**已有读数，按候选分（E166 / E167 / E169 的行，抄自产物）**

| 量 | 甲 准备并行落盘串行 | 乙 一次发布装多个事务 | 丙 多次发布同时在飞 |
|---|---|---|---|
| 崩溃枚举照旧按根槽写切会不会漏（E167） | 不漏，K1 漏数 0..0（一盘 `g1_serial`、两盘 `g4a_serial` 同） | 不漏，K1 漏数 0..0（`g1_one_publish`、`g4a_one_publish`） | 漏：一盘每条流都漏（`g1_free` 12870 / 12870 条流，最小 27），两盘 155115151 / 155117520 条流漏、最小 0（模型 `a4_e167_digest.py`） |
| fsync 等待 ρ = W_max(64) ÷ W_max(8)（E169 第三段，抽象单位） | 7.09 至 8.00，48 / 48 行「长」 | 1.29 至 5.76，36 行「长」、12 行「中间」 | 5.29 至 8.00，两个子形态各 48 / 48 行「长」 |
| 主取样点 W_max(64)（等速、P1、r = 1，甲整段） | 269.120 | 10.505 | 149.805（根序）、97.805（根乱） |
| E166 的共享状态 | 事务号 X 与树 ID T 都要罩 | 只有 X（T 由批发布者发，批发布者是单一持有者） | X 与 T 都要罩 |

出处行：E167 页「真实基线」V1 行与产物 `research/results/e167-multi-inflight-crash-cut-2026-10-01-r2.out:57`（g1_free）、`research/results/e167-multi-inflight-crash-cut-second-2026-10-01-r3.out:12`（g4a_one_publish）；E169 第三段 「quantity=Q2-3 q1=serial_publishes verdict_lines=48 grows=48 bounded=0 between_not_judged=0」（`research/results/e169-parallel-commit-append-and-prefix-recovery-segment-three-2026-10-01.out:2035`）、「quantity=Q2-3 q1=shared_publish verdict_lines=48 grows=36 bounded=0 between_not_judged=12」（`research/results/e169-parallel-commit-append-and-prefix-recovery-segment-three-2026-10-01.out:2036`）；主取样点三行：「quantity=Q2-3 q1=serial_publishes q2=jia_whole profile=uniform point=p1_device_dominated r=1 reading=r2_covered w_max_k8=33.640 w_max_k64=269.120 rho=8.0000 judgement=grows」（`research/results/e169-parallel-commit-append-and-prefix-recovery-segment-three-2026-10-01.out:1447`）、「quantity=Q2-3 q1=shared_publish q2=jia_whole profile=uniform point=p1_device_dominated r=1 reading=r2_covered w_max_k8=4.905 w_max_k64=10.505 rho=2.1417 judgement=grows」（`research/results/e169-parallel-commit-append-and-prefix-recovery-segment-three-2026-10-01.out:1543`）、「quantity=Q2-3 q1=multi_flight_roots_ordered q2=jia_whole profile=uniform point=p1_device_dominated r=1 reading=r2_covered w_max_k8=19.605 w_max_k64=149.805 rho=7.6412 judgement=grows」（`research/results/e169-parallel-commit-append-and-prefix-recovery-segment-three-2026-10-01.out:1639`）。E166 的 X、T 重复：「failure=7 clause=literal_lock_misses_shared_state triggered=true cells_with_x_or_t_duplicates=24」（`research/results/e166-parallel-commit-shared-state-partition-2026-10-01.out:1173`）；`a3_e166_digest.py` 的表里 A字面 在 Q1乙 的 Q-4（树 ID）为 0、Q1甲 与 Q1丙 为 46944。

**E166 的数够不够判 Q1：** 不够。E166 把 Q1 三个候选只当「阶段」标签，里面没有吞吐；它的 Q-9 只判到不比串行多；页自己写：「只是模型层的计数：不计时，没有吞吐。」（`.claude/kb/experiments/166-并行提交的共享状态分法.md:166`）它够判的是 Q4 的重复数那一半（见 C4）。

**甲的并行度上界，从代码推（推的，占比没量过）：** 准备阶段里碰分配器与 `previous` 的部分都在号序里（第 1 节第 3 条），所以甲真正能并行的只有取完落点之后装数据单元字节与校验和（`build_file_version_units`，`transaction.rs:6183` 起）。其余是「准备与上一个事务落盘重叠」的两级流水线，上限是 `1 + c ÷ S`（c 为准备耗时、S 为一次发布的设备同步）。

**模型 `c1_throughput_forms.py`（代数，取样点，不是本机的数）：** 乙靠摊薄固定项与同步，丙靠设备 flush 并发度 q，甲只抬 c。c ÷ S = 0.3 时甲 ≥ 0.769；乙在 m ≥ 12（q = 4）、m ≥ 14（q ≥ 8）以后不输丙；m = 2 时丙更好。排序翻面的四个参数 q、m、c ÷ S、f 全没量过。

**缺的数怎么量（C488 那一半，两件不需要并行代码就能做）：**
1. 现有串行路径上分段计时（计时器放在被测进程里，`.claude/singlefs-ai-sop/rules/test-discipline.md`「分段计时在被测进程里计」）：同一设备上「N 次 1 单元发布」对「1 次 N 单元发布」（后者就是今天顺序写的形态），各记 `prepare_the_version_publish` 与 `persist_publish_writes` 两段耗时，斜率给 c_p，截距给 c_f，persist 段给 S_pub，`build_file_version_units` 占比给 f。阳性对照沿用 E44 的 nosync（「| `sync_1`（每块一次 fdatasync） | **2785** | 0.42% |」（`.claude/kb/experiments/44-序号位宽的本机实测与代价.md:64`））。
2. 设备 flush 并发曲线：同一设备 k ∈ {1, 2, 4, 8, 16, 32, 64} 个线程各自 4 KiB O_DIRECT 写加 fdatasync 的合计次数 / 秒。E44 只量了 sync_1 与 sync_8（8 块一次），没量并发线程数。判据跑前写死（我拍的，主 agent 可改）：q(8) ÷ q(1) ≥ 2 才值得做丙，否则丙的上限等于乙。
本机口径要带上：机器是「机器：KINGSTON SNV3S4000G」（`.claude/kb/experiments/44-序号位宽的本机实测与代价.md:60`）；E152 虚机里发布 B 每次 6.6 ms 没拆（「7. singlefs 每次持久化的 6.6 ms 在编码与校验、21 次写、5 次刷新之间怎么分，没拆。」（`.claude/kb/experiments/152-按里程碑对比六家文件系统的文件性能.md:640`））。

**若必须现在表态（推的）：第一版取乙。** 理由四条都是仓里已有的：① 它就是今天顺序写「切成 N 个事务、共用一个 txg」的形态（第 1 节第 5 条）；② D16 已定项 10 已定「攒够一批脏节点（或到时间）再发布一次根，不是每个事务发一次」（`.claude/kb/decisions/16-发布语义.md:233`）；③ E167 里它的 K1 漏数 0，崩溃枚举不用换切法；④ E169 里它的等待最低。丙 在真实发布形上两种切法都跑不动（见 C7），所以丙要先过规模这一关。

**要改的条款与落点（转过去时）：** 问题单 Q1 行的「状态」；D13 已定项 2 射程补一句「候选的已有读数」；里程碑四开工时落点是 `crates/singlefs-core/src/mounted_session.rs` 的 `publish_user_change`（`mounted_session.rs:243`）前面加提交管道，里程碑三不动代码。

**什么现象会推翻「转」：** 上面第 2 件量出 q(8) ÷ q(1) ≥ 2 **并且** 第 1 件量出 S_pub 占 T0 的大头而 c ÷ S < 0.3 ——那时丙值得做，Q1 的取舍就够判，可以升成定案。**推翻「倾向乙」：** 同发的批量 m 受 fsync 延迟预算封顶在 q 个以下（D25 已定项 7 的并发 fsync 下限 16 是 C488 行引的，延迟预算本身仓里没有数）。

## 3　C2　Q2「记录按事务号顺序追加怎么实现」

**判：纪律现在定，实现选择（单一追加者 / 预留槽位）转里程碑四。**

**读数（E169 第一到三段，R1 与 R2 两种读法逐行相同，下面抄 R2）：**
- 号连续而内容属于两个半截事务（Q2-1 的 N2a）：串落下四条臂全 0；同发下甲交错 125、对照丁 666 打中——「quantity=Q2-1 q1=shared_publish q2=jia_interleaved reading=r2_covered legal_cells=44 legal_cells_positive=11 legal_states=125 judgement=hit」（`research/results/e169-parallel-commit-append-and-prefix-recovery-2026-10-01-r3.out:5053`）、「quantity=Q2-1 q1=shared_publish q2=ding_control reading=r2_covered legal_cells=44 legal_cells_positive=11 legal_states=666 judgement=hit」（`research/results/e169-parallel-commit-append-and-prefix-recovery-2026-10-01-r3.out:5101`）；多飞下两个子形态甲交错各 3、对照丁各 3——「quantity=Q2-1 q1=multi_flight_roots_ordered q2=jia_interleaved reading=r2_covered legal_cells=20 legal_cells_positive=1 legal_states=3 judgement=hit」（`research/results/e169-parallel-commit-append-and-prefix-recovery-segment-two-2026-10-01-r2.out:1981`）。
- 甲整段、乙整段：四个 Q1 条件下 0（「quantity=Q2-1 q1=shared_publish q2=jia_whole reading=r2_covered legal_cells=44 legal_cells_positive=0 legal_states=0 judgement=not_hit」（`research/results/e169-parallel-commit-append-and-prefix-recovery-2026-10-01-r3.out:5029`））。**这个 0 由臂的定义推出，不是量出的**：登记写「甲整段、乙整段在 H0、H1 的每一格（四个 Q1 条件）N2a = N2b = 0」（`research/prompts/e169-preregistration.md:560`），只当钉绝对值断言。所以第一分句真正量出来的只有「甲交错与对照丁会翻」。
- 臂的定义（登记）：甲整段「一个追加者逐个事务追加：Ti 的全部记录连着发出；发出 Ti 的第一条之前要求每个 j < i 的 Tj 记录全部已发出或已作废」（`research/prompts/e169-preregistration.md:443`）；甲交错「只要求 Ti 的第一条在每个 j < i 的 Tj 的第一条之后发出（或 Tj 已作废）」（`research/prompts/e169-preregistration.md:444`）。问题单 Q2 甲的原句「事务号小的没就绪时后面的等」没说就绪指整个事务还是一条记录，所以两读都对得上原文。
- 等待（Q2 第二分句，抽象单位）：四个 Q1 条件各 48 行 r1+r2 判决，「有界」0 行，F10（一臂长、一臂有界）0 配对（`a2_e169_wait_digest.py`：r2 读法 96 行，grows 90，bounded 0，其余 6 行「中间」；配对 60 行 f9_both_grow、4 行 neither）。长的是两臂共有的分量（每个事务自己的屏障与 FUA、设备队列、同发 P2 下各自的一个 CPU 单服务台），翻不了任何一臂。这一点是 D13 已定项 2 那句「事务号 N 的 fsync 在返回前必须等 < N 的事务全部已追加……」（`.claude/kb/decisions/13-验证路线.md:51`）自己推出来的：k 个同时 fsync 的调用方，最后一个必须等 k 次追加，除非批量化；这与追加者怎么实现无关。
- 今天的实现就是同发 × 甲整段 × 丙0：「今天一次发布里装多个事务（顺序写切成 N 个事务）是 Q1「同发」在 Q2 甲整段下的样子」（`research/prompts/e169-preregistration.md:398`）；串落 × 甲整段 × 丙0 的真实基线 14 条序列与 `crates/` 的 `replay_core` 逐条相同：「name=t1_summary sequences=14 mismatches=0」（`research/results/e169-parallel-commit-append-and-prefix-recovery-2026-10-01-r3.out:16`）。

**定案句（建议写入 D23 已定项 7 第二条要点与 D13 已定项 2 的并行前置约束）：**

> 并行提交下，journal 记录按事务号升序、以事务为单位整段追加：一个事务的全部记录 jsn 连号，jsn 次序等于事务号次序（记录什么时候发出去不限，只要落在自己的号段里：甲整段按号序一段一段发，乙整段取号时预留号段、各事务自己发）。事务号 N 的 fsync 返回前要等 < N 的事务全部已追加或已作废（D13 已定项 2 的前两个析取支，E169 四个 Q1 条件下 V_fsync 全 0）。「已确定实例切换或转只读」那一支 E169 没量（登记不含：「不在这份登记（修订 1 第 5 条）」（`.claude/kb/experiments/169-并行提交的追加纪律与前缀恢复.md:7`）），本条不对它作任何断言，D13 已定项 2 原句照旧。

这样写不越位的三处：① 只说「jsn 次序等于事务号次序、一个事务的记录连号」，排除甲交错与无纪律，不说「甲整段优于乙整段」（登记里两条臂的被测量就是这一句：「jsn 次序等于事务号次序，每个事务的记录 jsn 连号」（`research/prompts/e169-preregistration.md:443`）乙整段那一行写「jsn 次序等于事务号次序，每个事务连号；没有永久的洞」（`research/prompts/e169-preregistration.md:445`），两条臂都满足）；② 只说前两个析取支无违例，第三支原文不动；③ 「作废」只在 E169 的 H1（作废·窄）上量过，H2（作废·宽，D23 已定项 7 不许的历史）不进判定。

**转过去的条件（甲整段 vs 乙整段）：**
1. **乙整段需要在取号时知道记录条数**：登记自己写「条数取号时未知时要按上界预留、多余的写占位，那一形态不建模」（`research/prompts/e169-preregistration.md:448`）。代码里条数在准入之后才算出：「let records_in_this_publish =」（`crates/singlefs-core/src/transaction.rs:6115`），所以要么取号挪到 resolve 之后（那就和甲没有区别），要么按上界预留加占位（E169 没建模）。E169 的乙整段读数对应「取号即知条数」的读法。
2. 根乱子形态下乙整段 V_dangling 552、甲整段 0：「quantity=Q3-3 q1=multi_flight_roots_unordered q2=yi_reserved_whole reading=r2_covered bing0=552 bing1=552 bing2=552 bing3=552」（`research/results/e169-parallel-commit-append-and-prefix-recovery-segment-two-2026-10-01-r2.out:2111`）。四条恢复臂同值，E169 记成 Q1 条件的共有前提（F8）、不判 Q2；分开甲和乙的读数一共两处：这一处（只在 Q1 取多飞·根乱时相关）和 Q3-4（只对丙2 有意义，C3 不采丙2）。
3. 两臂在本机 fsync 等待上的差，等 C488 的计时（C1 第 1 件）。

**要改的条款与落点：** D23 已定项 7 第二条要点（「记录按事务号顺序追加，一个事务在发出单元写之后失败即实例切换、号更大的记录一条都不追加」（`.claude/kb/decisions/23-journal的角色与格式.md:210`））补「整段」半句；D13 已定项 2（`.claude/kb/decisions/13-验证路线.md:51`）同；问题单 Q2 行状态。`crates/` 里程碑三不动：今天 `persist_publish_writes` 里「for record in &writes.records {」（`crates/singlefs-core/src/transaction.rs:1069`）本来就是一个事务一整段、按计数器升序；里程碑四的追加者落在这一处。

**什么现象会推翻：** ① C488 的计时量出两臂关键路径分量不同（例如甲的追加者每条记录耗 h 随 k 长，乙的预留 g 不长）——F10 打中，第二分句能翻臂，Q2 的选择就够判；② H2 加进判定后整段臂出非零；③ 有人把「按事务号顺序追加」读成只约束首条（甲交错读法）——那时纪律句本身要重写。

## 4　C3　Q3「前缀里同时有完整的 B 与残缺的 A 时恢复取什么」

**判：恢复算法现在定（不换，照 D23 已定项 14 五条口径，即丙0 = 今天的 `replay_core`）；Verus 规约补一条 txg 连号转里程碑四（与 Q2 的实现同一批），今天不改。**

**读数（E169 第一、二段）：** 对三条候选追加纪律（甲整段、甲交错、乙整段），丙0、丙1、丙3 的 V_fsync（Q3-1）、V_dep（Q3-2）、部分施加（Q3-4）全 0；丙0 与丙1 在甲整段、乙整段的每一格恢复出的版本逐状态相同，不同的全在丙2 一条臂上（`a1_e169_digest.py` 的「按恢复臂汇总」：除 Q3-3 外，bing0 / bing1 / bing3 在三条候选追加臂上非零格 0 个，bing2 非零格第一段 4 个、第二段 2 个）。丙2 的打中行：
- V_dep：「quantity=Q3-2 q1=shared_publish q2=jia_interleaved reading=r2_covered q3=bing2_drop_a_keep_b legal_cells=44 legal_cells_positive=20 legal_states=69 judgement=hit」（`research/results/e169-parallel-commit-append-and-prefix-recovery-2026-10-01-r3.out:5060`）。
- 部分施加：同发下四条追加臂全打中，例：「quantity=Q3-4 q1=shared_publish q2=jia_whole reading=r2_covered q3=bing2_drop_a_keep_b legal_cells=44 legal_cells_positive=36 legal_states=120 judgement=hit」（`research/results/e169-parallel-commit-append-and-prefix-recovery-2026-10-01-r3.out:5040`）；多飞 × 乙整段 54：「quantity=Q3-4 q1=multi_flight_roots_ordered q2=yi_reserved_whole reading=r2_covered q3=bing2_drop_a_keep_b legal_cells=20 legal_cells_positive=10 legal_states=54 judgement=hit」（`research/results/e169-parallel-commit-append-and-prefix-recovery-segment-two-2026-10-01-r2.out:2016`）。

问题单 Q3 的两条翻面观测：① 「B 依赖 A 写下的单元」——只有丙2 打中；② 「fsync 已向 B 的调用方返回而恢复丢掉了 B」——四条恢复臂 V_fsync 全 0。②为 0 的原因在 D13 已定项 2：fsync(B) 要等 < B 的事务已追加（`.claude/kb/decisions/13-验证路线.md:51`），B 返回时 A 已完整，所以「截到 A 之前」丢掉 B 时 B 的调用方还没拿到返回。

**丙2 为什么出局（推到条款）：** 丙2 在同发、串落 × 乙整段、多飞 × 乙整段下只施加一次发布的一部分，与「**定案**：「施加一条记录在指针层上做什么」那个二选一取「一次发布整体施加或整体不施加」」（`.claude/kb/decisions/16-发布语义.md:111`）直接冲突；E169 登记把这条记成 F6「交主 agent」。我的推导是出局，不重开 D16 已定项 4。**丙3（遇到即判坏）** 与整段臂的配对站住（N2b = 0），与甲交错、对照丁的配对出局；它不需要作为恢复侧新增检查，由 C2 的整段纪律保证前缀里不会出现这种形状。

**定案句（建议写入 D23 已定项 14 的注，与 C2 的纪律句同批）：**

> 并行提交下恢复不换算法：取前缀仍是五条口径（jsn 严格连续、(实例代号, checkpoint_txg) 大于所选根水位、提交标记齐全的事务才施加、逐项验点名单元、施加的单位是一次发布）。「只丢残缺的事务、保留后面完整的事务」不采，因为它让一次发布被部分施加（D16 已定项 4）。

**「越过没施加的 txg」是不是今天前缀核在并行下的缺陷：**
- 它是**规约缺口加无纪律历史上的真行为**，不是候选纪律下的行为。规约缺口：前缀核的 spec 在「上一条是末条」时只要求「record.ordinal_within_publish.0 == 1」（`crates/singlefs-replay-prefix/src/lib.rs:230`），换 txg 的约束只在一次发布之内（「&&& record.checkpoint_txg == previous_record.checkpoint_txg」（`crates/singlefs-replay-prefix/src/lib.rs:232`）），M2 只说「若干整次发布的拼接」（「// 若干整次发布的拼接（M2：重放结果等于整数个发布，不存在被前缀切开的发布）。」（`crates/singlefs-replay-prefix/src/lib.rs:475`））——相邻两次发布 txg 不连号 spec 也放行。
- 真行为：对照丁 k = 3 的 H0 格上四条恢复臂 V_dep 同为 42：「check=A5_v_dep q1=multi_flight_roots_ordered q2=all reading=r2_covered bing0_today_v_dep=42 bing1_truncate_before_a_v_dep=42 bing3_forbid_v_dep=42」（`research/results/e169-parallel-commit-append-and-prefix-recovery-segment-two-2026-10-01-r2.out:2167`）；转写没写错，直接对拍把 88560 个崩溃状态的恢复输入喂给 `replay_core`：「input=all_durable distinct_inputs=6192 states=88560 mismatched_inputs=0 mismatched_states=0」（`research/results/e169-parallel-commit-append-and-prefix-recovery-replay-core-direct-check-2026-10-01.out:1`）。
- 串行下走不到：txg 从上一版现算（`checkpoint_txg_of_the_publish_after`，`transaction.rs:3446`；调用见第 1 节第 2 条）。三条候选追加纪律下 V_dep = 0（上面的读数）。
- **我复算了「施加的记录跳过 txg」的输入数**（`c3_txg_gap_recount.py`，只读 E169 的恢复输入产物，口径：施加的记录里相邻两条（含所选根）txg 之差 > 1）：`all_durable` 7 份 64 个状态、`strict_segment` 6 份 48 个状态，合计 13 份 112 个状态。E169 页写的是 10 份 208 个状态（`.claude/kb/experiments/169-并行提交的追加纪律与前缀恢复.md:254`），数法在执行员报告里，我没有，**没复现**；换三个口径数（`c3b_txg_gap_variants.py`）：含降序 35 份 / 1050 个状态，集合不连续 31 份 / 1034 个状态，有更小的 txg 没被施加 5 份 / 166 个状态；四个口径没有一个是 10 份 208 个状态。数随口径变，结论（非零）不变。回归输入就用这个产物里这一类输入。

**要不要现在改 Verus 规约：不改。** 四条理由：① 触发它的是 Q2 之外的无纪律历史，候选纪律下 0；② 规约是为编进产品的那份 `lib.rs` 写的（文件头「Verus 验的就是编进产品、被恢复路径调用的这一份」（`crates/singlefs-replay-prefix/src/lib.rs:2`）），补一条 txg 连号等于在 `obeys_the_reader_rules_and_the_commit_markers` 的「上一条是末条」分支（`lib.rs:229`-`lib.rs:231`）加一句并在 `replay_core` 的逐条判定里实现，重证 Verus、`verus-mutations.tsv` 补一个必红世界，都是 checker 档（形式化验证），子 agent 不跑；③ 它是 D23 已定项 14「五条，缺一不可」的第六条，加它要走三方并改条款；④ 它依赖 Q2 的实现细节：txg 与 jsn 必须在同一个临界区里按事务号同序发（C6 的 S1），这是里程碑四的设计，不是现在的。

**转过去的条件：** Q2 的整段纪律落地的同一批里做：D23 已定项 14 加「相邻两次施加的发布 txg 连号，不连号即止」；`lib.rs` 规约与判定；`verus-mutations.tsv` 加「跳一个 txg」必红世界；回归输入取 E169 恢复输入产物里上面那类输入。**更早触发：** 任何一版 Q2 实现允许 jsn 次序与 txg 次序不一致。

**什么现象会推翻：** 在甲整段、乙整段任一臂的合法历史上量到 V_dep > 0（E169 第一、二段全 0，若扩到 k ≥ 4 或 r ≥ 3 出现非零，规约缺口就是候选纪律下的缺陷，要提前补）。

## 5　C4　Q4「共享状态怎么分给多个在飞事务」

**判：「整体独占」的读法现在定（写成 A 读法）；六条臂里选哪一条转里程碑四（随 Q1 走）。**

**E166 六条臂的读数（`a3_e166_digest.py`，抄自产物；重复数是 duplicate_total，Q-1 至 Q-7 依次为 U、G、I、T 树 ID、X 事务号、J / JS、TXG / RS）：**

| 臂 | 重复数 | 冲突重算 Q-8 | 同步轮次 Q-9 | 页上的判 |
|---|---|---|---|---|
| A 整体独占（锁罩住从读到写回整段） | 三个 Q1 阶段全 0 | 0 | 不报 | 定义推出，不判 |
| A字面（候选定义逐字，锁只罩分配器、根环、journal 写指针、inode 号水位） | T：Q1甲 与 Q1丙 46944（8 格里 4 格），Q1乙 0；X：三个阶段都 11952（8 / 8 格） | 0 | 不报 | 判；失败 7、8 触发 |
| P区 分区（按区域） | 全 0 | 0 | 8 / 8 格比串行少（R_n = 1） | 定义推出 |
| P设 分区（按设备） | 全 0（加锁） | 12094527412260，截断 311170212792 | 8 / 8 格比串行少 | 失败 6 触发（D = 2 重算上限 2 内不收敛） |
| O整 乐观（整版校验） | 全 0 | 4576 | 0 格少于串行（R_n = n） | 定义推出 |
| O读 乐观（按读集校验） | 全 0 | 4576（Q1乙 2512） | Q1甲、Q1丙 0 格少；Q1乙 4 / 8 格少（不相交形状） | 判 |

出处行：「failure=6 clause=device_partition_does_not_converge triggered=true cells_with_truncation=18」（`research/results/e166-parallel-commit-shared-state-partition-2026-10-01.out:1172`）、「failure=7 clause=literal_lock_misses_shared_state triggered=true cells_with_x_or_t_duplicates=24」（`research/results/e166-parallel-commit-shared-state-partition-2026-10-01.out:1173`）、「failure=8 clause=judgement_flips_with_q1_candidate triggered=true flipping_columns=A字面:T」（`research/results/e166-parallel-commit-shared-state-partition-2026-10-01.out:1174`）、「question=Q-4 arm=A字面 stage=Q1甲 judgement=judged cells=8 cells_with_duplicates=4 duplicate_total=46944 no_duplicate_in_any_cell=false」（`research/results/e166-parallel-commit-shared-state-partition-2026-10-01.out:1064`）、「question=Q-5 arm=A字面 stage=Q1乙 judgement=judged cells=8 cells_with_duplicates=8 duplicate_total=11952 no_duplicate_in_any_cell=false」（`research/results/e166-parallel-commit-shared-state-partition-2026-10-01.out:1083`）、「question=Q-9 arm=O整 stage=Q1甲 judgement=definition_implied cells=8 cells_with_fewer_rounds_than_serial=0 cells_without_round_count=0 rounds_equal_serial_in_every_cell=true」（`research/results/e166-parallel-commit-shared-state-partition-2026-10-01.out:1163`）、「question=Q-9 arm=P区 stage=Q1甲 judgement=definition_implied cells=8 cells_with_fewer_rounds_than_serial=8」（`research/results/e166-parallel-commit-shared-state-partition-2026-10-01.out:1157`）。

**从读数和代码推出的四件事：**
1. **候选定义逐字不自洽，不是整体独占这条路不通。** A字面 漏的是事务号 X 与树 ID T，因为今天这两样和 txg、jsn、反向链、inode 号水位一样都从上一版 `previous` 现算（第 1 节第 2 条；inode 号水位「Some(previous_version) => previous_version.inode_number_watermark(),」（`crates/singlefs-core/src/transaction.rs:6313`））。共享状态的真清单 = 一次发布从 `previous` 读、再写回新一版的全部量，不是 Q4 候选里点名的四样。E166 页自己写了这条限定：「哪个读法是「整体独占」的定义归主 agent。」（`.claude/kb/experiments/166-并行提交的共享状态分法.md:168`）
2. **分配器这一项不是可选项：** 「装了根环表的进程写的根 txg 逐个加一、每一条都经分配器记」（`crates/singlefs-core/src/allocator.rs:1644`），准入读分配器此刻的计数（C5），发布 N+1 的取落点要读 N 之后的分配器。分区臂（P区、P设）要在分配器上另写一套「各区合并」的规则，E166 没建模这一步（推的）。
3. **没有一条臂被 E166 证到「逻辑内容不丢更新」：** 页自己写「记账 ACC 的丢更新与 BASE 的丢更新（Q-10）没建模」（`.claude/kb/experiments/166-并行提交的共享状态分法.md:172`）。C9 的模型里并行盲写 28 条交错 10 条违例（复活、重复释放、悬空），说明这一缺口有实际后果。
4. **吞吐那一半只有 Q-9：** 乐观臂的同步轮次不少于串行（O整 全部格 R_n = n），分区臂少。它们能抬的是 CPU 那一项，不是设备同步那一项；这和 C1 的代数同向。P区 的 FALLBACK 与死锁路径没走到：「区数 = 事务数、预留够用的默认几何：P区 的 FALLBACK 与死锁路径在默认几何上走不到」（`.claude/kb/experiments/166-并行提交的共享状态分法.md:171`）。

**定案句（现在定，写入问题单 Q4 候选的定义与 D13 已定项 2 的射程）：**

> 「整体独占」指一把提交锁罩住一次发布从读上一版 `previous`（txg、jsn 计数器、事务号、反向链、树 ID 水位、inode 号水位、分配器与它的根环占用表、记账行）到把 current 换成新一版为止的整段。只罩分配器、根环、journal 写指针、inode 号水位四样的写法漏掉事务号与树 ID 水位（E166 的 A字面，重复数 X 11952、T 46944），不叫整体独占。

「归谁定」：这是一句定义修补（没有它，候选的描述和 E166 自己的失败条款 7 打架），归主 agent 在判决里定；「第一版取整体独占」是设计取舍，随 Q1 转里程碑四，到时该交用户。

**转过去时带：** 上表六行；Q-10 没建模（ACC、BASE）；P设 在 D = 2 不收敛（页自己说不等于「分区按设备」做不到，换形态要走新实验）；P区 的 K = n − 1 取样点没跑；吞吐（C1 的四个量）。
**要改的条款与落点：** 问题单 Q4 行；D13 已定项 2 射程（`.claude/kb/decisions/13-验证路线.md:53` 附近加一行共享状态清单）。`crates/` 现在不动；里程碑四的锁落点是 `publish_user_change`（`crates/singlefs-core/src/mounted_session.rs:243`）到 `*current_file_version = published;`（`mounted_session.rs:314`）这一段。
**什么现象会推翻：** E166 第二段（K = n − 1、多 inode 预留不够、S = 1、环槽 2）量出 P区 的 FALLBACK 或死锁路径有违例；或 C1 量出 f（准备里能并行的占比）很大而 A 的锁段占大头——那时 A 近似串行，分区与乐观值得再比。

## 6　C5　Q5「准入的在飞合成怎么落到实现（C87）」

**判：「谁持有哪一份占用」的规则现在定；overlay 的实现与可强制清零的开关转里程碑四（第一版串行没有在飞，没有东西可清）。**

**代码今天的样子：** 准入读分配器此刻的计数（「allocated: BytesOnOneDevice::of_slots(device_map.allocated_slots()),」（`crates/singlefs-core/src/admission.rs:329`）），取落点一发生计数就加；在飞部分「在飞已批准第一版恒为零」（`crates/singlefs-core/src/admission.rs:313`）；已承诺预留（含墓碑）今天恒 0（「pub const COMMITTED_RESERVATION_OF_THE_FIRST_VERSION: BytesSummedOverAllReplicas =」（`crates/singlefs-core/src/admission.rs:156`）），环窗口项冻结副本里没有（`grep -rn 'ring_window\|RingWindow\|环窗口' crates/singlefs-core` 零命中；E168 装置里有，在 checker-tier）。条款：式子带「环窗口(d)」（`.claude/kb/decisions/28-挂载期承诺量.md:19`），环窗口是「**环窗口**（常驻扣项）」（`.claude/kb/decisions/28-挂载期承诺量.md:29`）；在飞合成「准入读数 = 已发布统计量 − 在飞已批准的预留」（`.claude/kb/decisions/28-挂载期承诺量.md:71`）；删除类发布不判式子：「删除类发布（意图、各批、挂载里的续做批）与管理员回退也不判式子」（`.claude/kb/decisions/28-挂载期承诺量.md:36`）。

**模型（`b3_c5_admission_overlap.py`、`b3b_c5_split_clear.py`，纯算术，事件 A 准入 → L 取落点 → P 发布，3 个事务，d + t = 4 + 1，R = 2，容量 C = 11 至 14，全部交错 1680 条，A 与 L 相邻即整体独占 90 条）。** 格内是「超卖交错数 / 假拒交错数」：

| 读数写法 | 任意交错 C = 11 | 任意交错 C ≥ 12 | 整体独占 C = 11 | 整体独占 C ≥ 12 |
|---|---|---|---|---|
| P 只读已发布统计，无 overlay | 1620 / 0 | 1620 / 0 | 72 / 0 | 72 / 0 |
| L 读分配器计数，无 overlay（今天的读法去掉恒 0 的 overlay） | 1620 / 0 | 1224 / 0 | 72 / 0 | 0 / 0 |
| LO 分配器计数 + 在飞预留（d + t 到发布才清）——问题单 Q5 候选二的字面 | 0 / 0 | 0 / 360 | 0 / 0 | 0 / 72 |
| PO 已发布统计 + 在飞预留（发布那一刻同步清）——D28 已定项 2 的字面 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| PO早 / PO晚（清的时点早一拍 / 晚一拍） | 738 / 0；0 / 0 | 936 / 0；0 / 60 | 72 / 0；0 / 0 | 72 / 0；0 / 18 |
| LS 分配器计数 + 在飞预留分两段清（d 在取落点清、t 在发布清） | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| L0 = LS 的 overlay 强制清零 | 1620 / 0 | 1224 / 0 | 72 / 0 | 0 / 0 |

**读法：**
1. **重扣是读数基与清零时点没配对，不是环窗口或墓碑预付自己的毛病。** 今天读的是分配器计数（取落点即加 d），问题单候选二写的「发布后结清」若原样套上来就是 LO：d 被分配器计数和在飞预留各扣一次，假拒 360 / 1680（任意交错）、72 / 90（整体独占）。配对只有两种：(i) 已发布统计 + 在飞预留 d + t 到发布才清（PO，D28 已定项 2 的字面；读数要从上一版记账行取，不从分配器计数取——推的：今天的 `of_allocator` 读分配器，改读上一版 `accounting_entries` 是实现上要加的一路）；(ii) 分配器计数 + 在飞预留，d 在取落点那一刻清、t 在发布那一刻清（LS）。混着用（LO）重扣，早一拍晚一拍（PO早、PO晚）超卖或假拒。
2. **环窗口 R 是几何常量，在每个变体里都只扣一次，不进在飞预留**：删除类发布不判式子（上面引的 `28-挂载期承诺量.md:36`），所以它们不领在飞预留，也就不会和常驻的 R 重扣。唯一的前提是不要给删除类发布补一份「最坏需求」预留。
3. **墓碑预付 t 的持有者在发布那一刻从在飞预留换成已承诺预留**；两边同一拍都算（PO早 / PO晚的一拍之差）就超卖或假拒。
4. 与 Q4 的关系：L 在 A、L 分开的任意交错下超卖 1620 / 1680（C = 11），整体独占（A 与 L 同一临界区）把它压到 72 / 90，剩下的 72 条全是 t 还没进计数。所以 Q4 取整体独占时 Q5 只需要 t 那一份 overlay；取分区或乐观时 d 那一份也要。

**定案句（写进 D28 已定项 2 的射程）：**

> 准入读数的每一份占用在任一时刻只有一个持有者：事务自己的新写槽 d，从准入到取落点由在飞预留持有，取落点那一刻起由分配器计数持有（预留同时清零）；墓碑预付 t，从准入到发布由在飞预留持有，发布那一刻起由已承诺预留持有（预留同时清零）。环窗口是几何常量，不进在飞预留；删除类发布不领在飞预留。准入与取落点不在同一个临界区时，准入就要按 d + t 预留。

**可强制清零的开关怎么造（C87 欠的那一半，里程碑四）：** `AdmissionReading::of_allocator`（`crates/singlefs-core/src/admission.rs:318`）加一个只供测试的在飞项参数（照 `PoolWriter` 上 `journal_record_named_entry_capacity` 的写法：「journal_record_named_entry_capacity: JournalRecordNamedEntryCapacity,」（`crates/singlefs-core/src/transaction.rs:200`），开关枚举带 `branch_name()` 让走哪一臂可观测）。**自证的容量要紧**：L0 在整体独占下只有 C = 11（= R + 两个事务的 d + t − 1）才超卖，C ≥ 12 清零不超卖（72 → 0），容量取松了自证白过。判据：LS 在 1680 条交错上 0 / 0，L0 在 C = 11 上超卖 > 0 且检查红。
**要改的条款与落点：** D28 已定项 2 射程；C87 行「仍欠」一句换成这一条；`admission.rs` 与 `transaction.rs:5183` 的调用点在里程碑四动。
**什么现象会推翻：** 在 k = 4 或 d、t 取别的值时 LS 出现超卖或假拒（模型里 k = 3、一组 d + t）；或删除类发布在实现里被补了最坏需求预留（那时 R 被重扣，要回来改规则 2）。

## 7　C6　Q6「litmus 与形式化」

**判：四·1 现在不加 litmus，Verus 扩展转里程碑四（与 C3 同批）。**

**同步点清单与现有罩（`c6_sync_points.py`，清单是我列的，出处在下面各条；脚本只对账计数）：**

| 候选 | 需要的同步点 | 现有 litmus 罩 | Verus M2 罩 | 入库计数模型罩 | 没有罩 |
|---|---|---|---|---|---|
| 甲 | 11 | 3（S4 S5 S6） | 1（S11） | 5（S1 S2 S3 S7 S8） | 2（S10 写失败的级联、S12 txg 连号） |
| 乙 | 10 | 3 | 1 | 5 | 1（S12） |
| 丙 | 12 | 3 | 1 | 5 | 3（S9 跨发布根次序、S10、S12） |

S4 至 S6 是单次发布内的三段落盘顺序，对应现有三条 Never：「单元 -> 屏障 -> journal 记录 -> 屏障 -> 根槽 FUA。」（`litmus/new-pool-file-creation-journal-implies-units.litmus:5`），第三条「坏结果 = 发布已经返回，之后读系统配置槽却看不到这次轮换写的值」（`litmus/publish-returns-after-the-rotation-barrier.litmus:6`）。S1 取号、S2 准入与取落点同临界区、S3 追加次序、S7 fsync 等待条件、S8 作废可见性是进程内的先后；S9 是丙才有的设备序（后一次发布的根不先于前一次发布的单元与记录持久，E169 的根序 / 根乱）；S10 是写失败时冻结的级联（`frozen_publish` 同时至多一个，第 1 节第 4 条；甲、丙里 N+1 已经在 N 之后的分配器上准备好了，N 失败它也得作废）；S11 是恢复取前缀整发布施加（Verus M2）；S12 是 C3 的 txg 连号。

**为什么现在不加 litmus：**
1. **甲、乙的设备序点一个都没新增**（上表：设备序 3 个、现有 litmus 罩 3 个）。新增的点全是进程内的锁与通道。`core/src` 今天零 Mutex / RwLock / Arc / thread / Send / Sync（第 12 节命令；Atomic 只在测试模块里给镜像文件起名：「use std::sync::atomic::{AtomicU64, Ordering};」（`crates/singlefs-core/src/block_device.rs:741`），上一行是「#[cfg(test)]」（`crates/singlefs-core/src/block_device.rs:738`）），crate 带 `forbid(unsafe_code)`，进程内先后由 std 给；LKMM 的 `smp_wmb` 在这些 litmus 里是块层 FLUSH 的代理：「P0 = 写者（提交事务）；P1 = 读者（挂载或并发读）。」（`litmus/commit-publish.litmus:10`），建模不了 Rust std 的锁。进程内这五个点现在由 E166 / E169 的状态机模型罩，没有代码绑定。
2. **丙才有新增的设备序点 S9**，它是一个三进程形态（两个写者加恢复读者）。但门禁 lkmm.sh 要求每条 Never 绑代码：「4. 每条 Never 都要绑到代码。herd7 只判 litmus 写下的那个形态」（`.claude/scripts/lkmm.sh:39`），绑不到时只能写 `none —— <理由>`，像「singlefs-models: none —— COW 发布序的通用形态」（`litmus/commit-publish.litmus:13`）那样成一条悬空的抽象测试。Q1 没选，现在写它的形态是猜。
3. **候选三项各罩几个：** 加两写者 litmus 只罩 S9（1 / 12，且只对丙）；只扩形式化（Verus 补 txg 连号）只罩 S12（1 / 12）；两样都扩罩 2 个。进程内五到六个点、S10 谁也罩不到，需要的是带运行期断言的、可注入调度的测试（`allocator.rs:1644` 那条 txg 逐个加一的断言是现成的先例）。这是问题单没有的第四个候选，列入「问题单以外的空白」。

**转过去的条件与落点：** Q1 取丙时，在 `litmus/` 加一对（Never + 去掉等待的 Sometimes）三进程形态并绑到实现里那个等前一发布根的函数；Q2 / Q3 的纪律落地时 Verus 补 S12（C3）。`crates/` 现在不动。
**什么现象会推翻「现在不加」：** 实现里自己写原子变量（`AtomicU64` 加 `Ordering`）做取号器——那时进程内的点有了内存序语义，要立一对 litmus；或 Q1 在里程碑三就取了丙。

## 8　C7　Q7「崩溃枚举怎么切多在飞的发布」

**判：转里程碑四；一句条件句现在记。** E167 的三条结论里，只有第一条量足了，另两条各差一格，且三条都只在 Q1 取多飞时才用得上。

**三条结论各自量到哪一步（`a4_e167_digest.py`，抄自产物）：**

| 结论 | 量到的 | 没量到的 |
|---|---|---|
| 按根槽写切（K1）在多在飞下会漏 | **多飞才漏**：一盘 `g1_free` 12870 / 12870 条流都漏（「name=arm_by_root_publish cell=g1_free streams=12870 states_minimum=15 states_maximum=29」（`research/results/e167-multi-inflight-crash-cut-2026-10-01-r2.out:57`），漏数最小 27）；两盘 155115151 / 155117520 条流漏、最小 0（「cross_misses_minimum=0 cross_misses_maximum=1324774 streams_with_cross_misses=155115151」（`research/results/e167-multi-inflight-crash-cut-second-2026-10-01-r3.out:25`）），对走得到的跨事务子集最大漏 702 / 729（「true_cross_misses_minimum=0 true_cross_misses_maximum=702 streams_with_true_cross_misses=155115151」（`research/results/e167-multi-inflight-crash-cut-third-2026-10-01.out:3`））。串落与同发恒 0（`g1_serial`、`g1_one_publish`、`g4a_one_publish`：states_minimum = states_maximum，漏数 0..0） | 没有「哪条流会被 harness 录到」的概率：页自己写「不按概率加权」 |
| 按事务切（K2）不漏 | 两盘 8 个格（含 P1、P2）上对走得到的跨事务落盘子集漏 0（一盘 12 格按定义推得 0）：「name=arm_by_transaction cell=g4a_free states=784 cross_misses=1324072 true_cross_misses=0 non_cross_misses=2248 states_not_in_truth=0 identity_ok=true」（`research/results/e167-multi-inflight-crash-cut-second-2026-10-01-r3.out:26`）；那 1324072 个「漏」是对模型全集（两盘上等于全部状态向量）的漏，精确可达集只有 784 个 | **真实发布形没跑**：k ≤ 2 个事务、每个事务 u ≤ 2 个单元、≤ 18 个写 |
| 状态数超出崩溃放量装置能力 | 算术：K1 在真实发布形上存在一条把 58 个写并成一段的发出流，「segment_states=1459166279268040703 exceeds_u32=true within_u64=true」（`research/results/e167-multi-inflight-crash-cut-second-2026-10-01-r3.out:108`）；K2 两个事务的积「quantity=real_shape_arm_by_transaction_product_states expected=281475379363984 actual=281475379363984 exceeds_u32=true within_u64=true ok=true」（`research/results/e167-multi-inflight-crash-cut-2026-10-01-r2.out:35`）。对照：GPU 判器一段的状态数要小于 u32：「segment.states.end - segment.states.start < u64::from(u32::MAX),」（`crates/singlefs-checker-tier/src/crash_verify_gpu.rs:304`），崩溃放量到 E 全域一共 1 662 648 564 态（`git log` 提交 4e8b0370 的标题） | 没跑：K1 最大一段的动态规划只在小几何上对拍过 |

K2 积 2.81×10¹⁴ 对 1.66×10⁹，差 1.7×10⁵ 倍（`python3 -c 'print(281475379363984/1662648564)'` 得 169293.37…，第 12 节）。所以**在真实发布形上两种切法都跑不动**，不是「K2 比 K1 小」的问题。

**代码落点：** 今天的切法是「一次根槽写之前、上一次根槽写之后的单元写与 journal 记录写都归它」（`crates/singlefs-harness/src/memory_pool.rs:951`）：`publishes_of_one_recording`（`crates/singlefs-harness/src/memory_pool.rs:993`）按「StepKind::RootRecordFua => {」（`crates/singlefs-harness/src/memory_pool.rs:1005`）一遇根槽写就收一次发布；单次提交最多枚举 2048 个写：「pub const MAXIMUM_ENUMERATED_WRITES: usize = 2048;」（`crates/singlefs-checker-tier/src/crash_facts.rs:129`）。

**条件句（现在记，写进问题单 Q7 行与 D13 已定项 4 射程）：**

> 崩溃枚举的切法随 Q1：Q1 取串落或同发时沿用按根槽写切（E167 里 P1、P2 的 K1 漏数在一盘 6 格与两盘 4 格共 10 格全为 0）。Q1 取多飞时，按根槽写切在一盘的每条录制流上、两盘的绝大多数录制流上漏走得到的跨事务落盘子集；按事务切在小几何上不漏，但在真实发布形上两个事务的总状态数 2.8×10¹⁴，是现有崩溃放量装置总量的约 1.7×10⁵ 倍。多飞要先有能在真实发布形上落地的枚举方案，才能进里程碑。

**转过去的条件：** Q1 选了丙。带：上表三行；D13 已定项 10 的「新结构等价类」门禁（`.claude/kb/decisions/13-验证路线.md:221` 一带）会因多飞的录制流形状触发（推的）；需要一个能把真实发布形压下来的方案（例如按独立写的等价类归并）及其必红世界。
**要改的条款与落点：** 问题单 Q7 状态；`memory_pool.rs:993` 的切法与 `crash_verify_gpu.rs:304` 的断言是里程碑四丙的改动面，里程碑三不动。
**什么现象会推翻「多飞跑不动」：** 一个可靠的归并（把真实发布形的 58 个写按独立性压到 10⁹ 量级以内，并在小几何上与 E167 的两盘 8 格逐状态相同）；**推翻「串落 / 同发沿用 K1」：** 在真实发布形的同发流（m 个事务各若干单元）上枚举出 K1 漏的状态——E167 的同发格只到 u ≤ 2。

## 9　C8　Q8「C58 的交点报警立在哪」（四·1 点名，判到能实现）

**判：现在定。交点取运行时配置（候选二），计数器接在会话发布成功之后，口径是本实现的「版本整片建树输入记录数」，第一步是在本实现上标定交点——不是拿 11200。**

### 9.1　先说单位：11200 的单位和本实现对不上

C58 行的交点是「交点约 **1.1 万条条目/checkpoint**（31.2M 条目/秒 × 359 µs）」（`.claude/kb/checks-owed.md:59`）。31.2M 来自 E17：「**29.9–31.8M 条目/秒（685–727 MiB/s，八轮独立运行）**」（`.claude/kb/experiments/17-writebuffer合并上限.md:8`），是 write buffer 里 sort-merge 的输入条目；359 µs 是 E44 一次 fdatasync：「| `sync_1`（每块一次 fdatasync） | **2785** | 0.42% |」（`.claude/kb/experiments/44-序号位宽的本机实测与代价.md:64`）的倒数。本实现**没有 write buffer**（第 12 节 grep，零命中），每次发布把每棵树按整个版本重建：`allocation_records` 每次整份拷出再排序（「let mut allocation_records: Vec<AllocationRecord> = allocator.records().to_vec();」（`crates/singlefs-core/src/transaction.rs:6267`）、「allocation_records.sort_by_key(AllocationRecord::sort_key);」（`crates/singlefs-core/src/transaction.rs:6268`）），映射树对文件每个数据单元一条（「let mut mapped_units_with_locations: Vec<(TransactionUnit, Vec<u8>, [LocationEntry; 2])> =」（`crates/singlefs-core/src/transaction.rs:6415`）起）。所以 CPU 随**版本整片**的记录数走，不随这次事务的增量走。另一头，一次发布的设备同步不是一次 fdatasync，是四个序点：D16 已定项 7 的屏障口径（`.claude/kb/decisions/16-发布语义.md:184`）；E152 的虚机里发布 B 每次 6.6 ms（`.claude/kb/experiments/152-按里程碑对比六家文件系统的文件性能.md:640`）。交点 N* = r_impl × S_pub 两项都得在本实现上量。

### 9.2　三个交点来源（`b1_c8_crossover.py`，S 档里除 E44、E152 两个已量值外都是取样点）

N* = r × S，r 取 E17 区间 29.9M / 31.2M / 31.8M；下表取 r = 31.2M，常量取 11200：

| S 档 | N* | 编译期常量 11200 的错位 |
|---|---|---|
| 0.1 × E44（取样点：快 10 倍的盘） | 1120 | **永不响带** (1120, 11200]，宽 10080，占常量 90% |
| 1 × E44（一次 fdatasync，C58 用的） | 11203 | 无错位（差 3 条，取整） |
| 4 × E44（四个序点各等一次 fdatasync，取样点） | 44811 | **总在响带** (11200, 44811]，常量的 3.00 倍 |
| 6.6 ms（E152 发布 B，虚机，含 CPU） | 205920 | 总在响带，17.39 倍 |
| 10 × E44（取样点：慢 10 倍的盘） | 112029 | 总在响带，9.00 倍 |

方向：S 比本机一次 fdatasync 小，报警**永不响**；S 大，**总在响**；而本机自己一次发布就有四个序点，常量在本机上就已经偏到「总在响」那一侧。

| | a mkfs 时实测写进系统配置 | b 运行时配置（调用方给，不进盘） | c 编译期常量（取本机已量的数） |
|---|---|---|---|
| 格式代价 | 系统配置 489 字节（「pub const SYSTEM_CONFIGURATION_BYTES: u64 = 489;」（`crates/singlefs-format/src/lib.rs:232`））再加一个 8 字节字段：新池写出的字节变，而 D13 已定项 2 刻意写了「已定项 2 不决定记录头的字段清单、不改变新池新建文件写出的字节」（`.claude/kb/decisions/13-验证路线.md:53`）；系统配置每加一个字段都是一次用户定案（「2026-09-26 用户定案在系统运行量末尾加回退下界 F 8 字节，偏移 481 起，481 → 489」（`crates/singlefs-format/src/lib.rs:231`）） | 零 | 零 |
| 复跑性 | 差（推的）：mkfs 在哪台机器、什么负载下计时就写什么值；同一块盘 mkfs 两次得两个数；测试要固定值就得另开注入口，那就是候选二 | 好：值是显式输入 | 好：值编进二进制 |
| 换机器 | 盘搬走，盘里是 mkfs 那台机器的 N*，错位和常量是同一张表（`b1_c8_crossover.py`：本机 ↔ 10 × E44 互搬各是 100800 条宽的带） | 调用方在新机器上重给 | 同 a |

**取候选二。** 理由三条：① a 的格式代价买不到 b 没有的东西——盘一搬就和常量一样错位，唯一多出来的是「mkfs 那台机器上是对的」；② 判别力自证要能把门槛挪动，只有 b 有现成的注入口（`PoolWriter` 上 `set_journal_record_named_entry_capacity` 那类开关的先例，第 6 节）；③ C58 原文这件事「不报错、只是变慢」，报警不需要持久。缺省值（调用方没给）用编译期常量，**常量取标定的结果**，写进 kb 一行带机器口径（照 `.claude/kb/tooling.md` 里「仓里的性能数都只在本机成立」那一节）。

### 9.3　计数器与报警接在哪（实现员照着写）

**口径（`checkpoint_entries`）：** 一次发布的建树输入记录数，分六项，都已经是 `TransactionOutput` 的字段（「pub struct TransactionOutput {」（`crates/singlefs-core/src/transaction.rs:2488`））：
1. extent 叶记录 = `data_pointers.len()`（「pub data_pointers: Vec<DataPointer>,」（`crates/singlefs-core/src/transaction.rs:2504`））；
2. 中央映射条目 = `mapping_keys.len()`（「pub mapping_keys: Vec<Vec<u8>>,」（`crates/singlefs-core/src/transaction.rs:2505`），构造在 `transaction.rs:6502`）；
3. 分配记录 = `allocation_records.len()`（「pub allocation_records: Vec<AllocationRecord>,」（`crates/singlefs-core/src/transaction.rs:2507`））——全版本、不是增量，这一项随池里已分配的槽数涨，正是 CPU 的大头之一；
4. 记账行 = `accounting_entries.len()`（「pub accounting_entries: Vec<AccountingEntry>,」（`crates/singlefs-core/src/transaction.rs:2513`））；
5. 树表行 = `tree_table_entries.len()`（「pub tree_table_entries: Vec<TreeTableEntry>,」（`crates/singlefs-core/src/transaction.rs:2518`））；
6. inode 记录 = `inode_leaf_containers` 里每片 `contents.records.len()` 之和（「pub inode_leaf_containers: Vec<InodeLeafContainerVersion>,」（`crates/singlefs-core/src/transaction.rs:2526`））。

**接线：**
1. `TransactionOutput::checkpoint_entries(&self) -> CheckpointEntries`：纯方法，返回六项与它们的和（先例：C319 的 `count_key_order_mismatches`，纯函数加单测，「pub fn count_key_order_mismatches(extent_keys_in_issue_order: &[[u8; 24]]) -> u64 {」（`crates/singlefs-core/src/transaction.rs:3493`），断言写法见「assert_eq!(count_key_order_mismatches(&ascending), 0);」（`crates/singlefs-core/src/transaction.rs:7104`））。**用方法、不加字段**：`TransactionOutput` 在 `transaction.rs` 的发布路径、`mount.rs:5187`、`recovery.rs:2764` 三处按字段构造，加字段要改三处；方法从现有字段现算，从盘上重建的版本同样算得出。
2. 新模块 `crates/singlefs-core/src/ceiling.rs`：`pub struct CeilingCrossover(pub u64)`；`pub struct CeilingAlarm { pub entries: u64, pub crossover: u64 }`；`pub fn judge(entries: u64, crossover: CeilingCrossover) -> Option<CeilingAlarm>`，**严格大于**才报（`entries > crossover.0`）。纯函数，不碰设备。
3. 发布成功之后判：`MountedSession::publish_user_change`（`crates/singlefs-core/src/mounted_session.rs:243`）里 `Ok(published)` 那一支：`*current_file_version = published;`（`crates/singlefs-core/src/mounted_session.rs:314`）之后、`return Ok(UserChangePublished {…})`（`crates/singlefs-core/src/mounted_session.rs:315`）之前，对新一版算 `checkpoint_entries()` 再 `judge`。**判在落盘成功之后**：失败的发布冻结在分配器上（`transaction.rs:1156`），不算「本次 checkpoint」。三条用户发布路径（覆盖写、顺序写、建 inode）都经 `publish_the_change_once`（`crates/singlefs-core/src/mounted_session.rs:180`）汇到这里，所以放这一处三条都罩住；放进某一条路径的包装里只罩一条。零单元发布（`publish_without_units`，挂载暖机与写行）不经这里，也不判：D28 已定项 1 同样不判没有普通分配的发布。
4. 结果放进 `UserChangePublished` 新字段 `ceiling_alarm: Option<CeilingAlarm>`（`crates/singlefs-core/src/mounted_session.rs:63` 定义处）；**报警不让发布失败**（C58 原文：不报错、只是变慢）。`MountedSession`（`crates/singlefs-core/src/mounted_session.rs:39`）加一个 pub 字段 `ceiling_crossover: CeilingCrossover`，`of_the_mount` 里取缺省常量，调用方可改。
5. 「红」落在 harness 档：固定基线脚本每次发布断言 `ceiling_alarm.is_none()`，T1 造的翻倍负载断言 `is_some()`。门禁不用新开阶段，随 harness 的测试走。
6. 独立数一遍（校验路径）：`crates/singlefs-checker/src/walk.rs` 里在 `check_pool_image` 旁加一个只数条数的函数（「pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, InvariantVerdict)> {」（`crates/singlefs-checker/src/walk.rs:5737`）），从盘上镜像逐棵读叶数六项，**不引用 core**；T3 逐项比。证据纪律：「⚠️ **独立计数只对上了总数，印证不了拆法。**」（`.claude/singlefs-ai-sop/rules/evidence-discipline.md:19`），所以比六项各自的数，不只比和。

### 9.4　标定交点（实现员第一步，harness 档，不是重型测试）

1. 在被测进程里用 `std::time::Instant` 包住 `prepare_the_version_publish`（`crates/singlefs-core/src/transaction.rs:5103`）与 `persist_publish_writes`（`crates/singlefs-core/src/transaction.rs:1057`）两段，对多种规模各发若干次：写 u ∈ {1, 16, 144, 1000} 个数据单元，池里已有的分配记录数取几档（覆盖「分配记录随池涨」这一项）。
2. 对 `checkpoint_entries().total` 与 prepare 耗时做回归：斜率 = 1 ÷ r_impl，截距 = 固定项；persist 段耗时 = S_pub，设备用真盘后端（文件后端 O_DIRECT），阳性对照照 E44 的 nosync。N* = r_impl × S_pub，缺省常量取它，在 kb 里记一行带机器口径。
3. 回归若不线性（allocation_records 的排序是 n log n），改成分项回归并把阈值改成「预计 prepare 耗时 ≥ S_pub」的两项式；那时 `judge` 的第二个参数不是一个数而是一对系数。**这一步的结果能推翻候选二以外的东西，见「什么现象会推翻」。**

### 9.5　判别力自证怎么造（用例，每条先证会红）

| 用例 | 做法 | 抓的接线故障 |
|---|---|---|
| T1 翻倍 | 同一个池上造负载 w 与 2w（顺序写 n 个与 2n 个单元；另一组建 m 个与 2m 个 inode），各发一次，取 E_w、E_2w，门槛取两者中点；断言 w 不报、2w 报 | 计数器没接（恒 0）、漏项、只取最后一个事务 |
| T2 门槛挪动 | 同一负载，门槛 = E_w − 1 必须报，= E_w 必须不报（严格大于）；挪回必须由红转绿 | 门槛恒无穷、比较取反、读到上一次发布的计数 |
| T3 独立点数 | `checkpoint_entries()` 六项与 checker 从镜像数出的六项逐项相等 | 漏项、少数、单位错 |
| T4 三条路径 | 覆盖写、顺序写、建 inode 各一次有负载的发布，每条的计数都大于同一路径空负载的计数 | 只接了一条发布路径 |

**模型核对（用例集够不够，`b2_c8_selfproof.py`、`c8b_selfproof_paths.py`）：** 七种接线故障（F1 没接、F2 漏 inode 项、F3 漏 extent 项、F4 只取最后一个事务、F5 门槛恒无穷、F6 比较取反、F7 读上一次的计数）加 F8 只接一条路径，T1 至 T4 里每个故障至少被一条抓到、每条用例至少抓到一个（b2 的输出：「漏网的故障： 无」；F8 被 T4 抓到、b2 的四个用例不分路径抓不到）。**T1 的门槛取实测两点的中点，不依赖 11200，所以 T1 在任何机器上都有判别力**；只有生产缺省值取决于标定。b2 里各项的大小是取样点，用例通过所需的真实数要实现员在第 9.4 步之后现取。

### 9.6　要改的条款与落点（汇总）与推翻条件

- `.claude/kb/checks-owed.md` C58 行：「交点由 mkfs 时算……写进系统配置或运行时配置」改成「运行时配置（调用方给）加本实现单位的标定」，条目口径写成第 9.3 的六项；D13 已定项 2 射程「约 1.1 万条条目」加一句单位注（E17 的条目是 write buffer 的 sort-merge 输入，本实现按版本整片重建，交点要在本实现上标定）。
- `crates/`：`transaction.rs`（`checkpoint_entries` 方法）、新 `ceiling.rs`（`judge`）、`mounted_session.rs`（`UserChangePublished.ceiling_alarm`、`MountedSession.ceiling_crossover`）、`singlefs-checker/src/walk.rs`（只数条数的函数）、harness 档测试 T1 至 T4。里程碑三内全部做得了，不依赖并行提交。
- **什么现象会推翻：** ① 标定量出 prepare 耗时与 `checkpoint_entries` 不线性（排序的 n log n 或固定项占大头）——单一阈值不成立，改两项式；② 量出的 N* 大到造不出负载（例如 > 10⁷ 条）——报警形同虚设，C58 应降成「登记、不做」；③ 实现里出现不经 `publish_admitted` 的第二个发布汇入点——`grep -n 'publish_admitted(' crates/singlefs-core/src/transaction.rs` 现在只有一个调用点（`transaction.rs:5209`，第 12 节），多一个就要回头把计数挪到更靠下的汇入点；④ b 的使用者在生产里没人配值、缺省常量永远用——那就退回候选三的错位表，缺省常量必须随标定结果更新。

## 10　C9　Q9「删除的续做与用户事务能不能同时在飞」

**判：串行下现在定（没有剩余的并发问题）；并行下转里程碑四，带一条条件。**

**串行下还剩什么：** 问题单 Q9 的两条翻面观测，① 「用户事务写进正在删的 inode」、② 「删除的最后一批与意图不在同一事务里」，串行下都被已定项挡住：
- ① 有两道保险，各自足够：「写入口拒 nlink = 0 的 inode」（`.claude/kb/decisions/08-核心索引结构.md:461`）已在代码里（「fn refuse_writing_the_file_while_its_inode_is_being_deleted(」（`crates/singlefs-core/src/transaction.rs:4915`），覆盖写与顺序写各调一次：`transaction.rs:4753`、`transaction.rs:4807`）；「会话里每一次用户改动（含 unlink、写、建 inode）先把现行那一版的 `deleted_inodes` 队列走空」（`.claude/kb/decisions/08-核心索引结构.md:463`）。`c9_serial_subset.py`：用户事务 W 插进删除三次发布（意图 I、批 B1、末批 B2）的 4 个位置，只有第 4 条 → 违例 0；只有第 6 条 → 违例 0；两条都没有 → 2 种位置违例（I-B1-W-B2、I-W-B1-B2）。所以串行下两条互为冗余保险，第 6 条的独立用处是队列长度界（回退到意图那一版会把号带回队列，已定项 16 依据里的 m3-blanks-r1 一行），与并发无关。
- ② D8 已定项 5 要的是「删除意图必须与最后一批工作在同一个事务里提交」（`.claude/kb/decisions/08-核心索引结构.md:120`），已定项 16 把它落成「最后一批释放剩下全部单元、删 inode 记录、删 `deleted_inodes` 那条 key，同一次发布」（`.claude/kb/decisions/08-核心索引结构.md:459`）：一次发布就是一个原子单位，串行下没有「同时在飞」。

**并行下（`b4_c9_delete_interleave.py`、`b4b_c9_atomic_and_exclusive.py`，D 的六步 PI CI P1 C1 P2 C2 与 W 的两步 PW CW，全部交错 28 条）：**

| 提交纪律 | 规则放行的交错 | 违例交错 | 备注 |
|---|---|---|---|
| 并行、提交盲写 | 28 | 10 {复活 2；重复释放 + 泄漏 + 复活 4；重复释放 + 复活 2；重复释放 + 悬空 2} | W 计划时已看到 nlink = 0 而被第 4 条拒的 15 条交错，第 4 条只挡了这一半 |
| 并行、提交时校验读集（X 的 inode 版本） | 28 | 0 | 重新计划合计 12 次 |
| 续做独占·计划闸（队列非空时 W 不许开始计划） | 14 | **10**（同盲写那几类） | 计划闸不够：W 在意图提交之前计划、之后提交 |
| 续做独占·计划闸 + 提交闸（提交时队列非空或 X 版本变了就作废） | 14 | 0 | 提交闸作废 W 10 条 |
| 整体独占（Q4 的 A，每次发布计划加提交不可分） | W 插进 D 三次发布的 4 种位置 | 0 | |

**读法：** 「先走空」与「写入口拒 nlink = 0」都只在**计划时**看现行版（代码里 `refuse_writing_the_file_while_its_inode_is_being_deleted(previous)` 的 `previous` 就是计划时的上一版，第 1 节第 9 条）。并行下 W 可以在意图提交之前计划、之后提交，只有计划闸挡不住。能挡住的三种写法（提交时重判、读集校验、整体独占）都是 Q4 里已有的臂，所以并行下 Q9 不是新问题，是 Q4 的写写冲突类；E166 的分区臂（P区、P设）不管 inode 记录的写写冲突（页自己写 Q-10 没建模，第 5 节）。

**定案句（写入 D8 已定项 16 射程）：**

> 串行提交下删除的续做与用户事务不并发：会话里每一次用户改动先走空队列、写入口拒 nlink = 0 的 inode，两条各自足以挡住用户事务写进正在删的 inode；最后一批与意图不同事务由第 2 条同一次发布钉住。并行提交打开时，这两条只在计划时看现行版，不够：要么计划之后在提交时重判（队列非空或 X 的 inode 版本变了就作废这次事务），要么用户事务的读集校验 X 的 inode 记录版本，要么把计划与提交放进同一个临界区。

**转过去时带：** 上表；条件是 Q4 选定（取分区臂的话 inode 记录的写写冲突要另有机制）。**落点（里程碑四）：** 把 `refuse_writing_the_file_while_its_inode_is_being_deleted`（`transaction.rs:4915`）的调用从两个计划点（`transaction.rs:4753`、`transaction.rs:4807`）加到提交点，对象是提交队列队首那一版而不是计划时的 `previous`；删除合入后的「走空」点放在 `publish_user_change` 里（`crates/singlefs-core/src/mounted_session.rs:243`）。里程碑三不动。
**什么现象会推翻：** 模型里只有一个覆盖写 W 与一条删除链 D，别的用户事务类型（两个 unlink 打同一个 inode、truncate、rename）没建；扩进去后串行下的两条保险不够或并行下的三种写法漏，这一格要重判。

## 11　C10　Q10「读者与写者同时跑」

**判：转里程碑四。**

**依据：**
1. 「并发」的口径：用户 2026-10-01 定为「多个事务并行提交」（`records/2026-10-01-删除与并发事务.md:41`：「多个事务并行提交」）；读写并发不在这一句里。里程碑三项四的原话「并发读写稳定」（`.claude/kb/milestone/03-third-txn.md:94`）与它的作业点四·1 只做前置（「只做前置：C58」（`.claude/kb/milestone/03-third-txn.md:100`））。
2. **今天做不了读写并发，要先动三样东西**（都是里程碑四并行提交本来就要动的）：① 只读挂载态持有 `Cell`（「observation_since_open: Cell<ReadPathObservation>,」（`crates/singlefs-core/src/mounted_read.rs:327`）），`Cell` 不是 `Sync`，同一个挂载态不能被两个线程同时读（推的，没编译验过）；② 设备读是 `&self`、写是 `&mut self`（「fn read_at(」（`crates/singlefs-core/src/block_device.rs:116`）、「fn write_at(」（`crates/singlefs-core/src/block_device.rs:121`）），发布期间盘表被 `&mut` 独占，同一份盘表不能同时 `&` 借给读者；③ `core/src` 里没有任何共享可变状态的原语（第 12 节命令零命中）。这三样改动就是 Q1 / Q4 要引入的第一批锁与 `Send` / `Sync` 界，在里程碑三单独做会抢在 Q1、Q4 前面定了设备所有权的形状。
3. **C214 的窗口在读写并发下才出现，并且今天没有地方放那个共享计数**：「⚠️ **但那条余量没有任何东西在守**」（`.claude/kb/checks-owed.md:184`）。可判形式是「运行期断言「任一遍历的相邻两跳之间，已过去的发布数 < K」，越界即判红并中止那次遍历」（`.claude/kb/checks-owed.md:184`）：断言要读者看得到写者的发布计数，串行下读与写交替、不可能越界，断言没有地方触发，接不上。
4. **模型（`c10_reader_writer_window.py`，一个读者 3 跳、写者 6 次发布，全部交错 924 条）：** 无保护时「Rp 与 Rc 之间过去的发布数 ≥ K」的暴露交错：K = 1 → 840（90.9%）、K = 2 → 547（59.2%）、K = 3 → 249（26.9%）、K = 6 → 3（0.3%）。K 的口径有 ±1 的出入，只取方向：读者的一跳一旦被写者的 K 次发布超过就违例，没有保护只能靠没人守的余量。两种保护——读者钉住版本（写者不重用被钉住版本之后换下的块）与运行期断言——按定义清零或中止这批违例，没有在模型里重复数。

**转过去的条件：** 里程碑四的并行提交把设备所有权改成能共享的形状（Q1、Q4 定了之后）；带：C214 的单跳耗时实测（今天能做的一半）、钉住版本与断言二选一、模型的暴露表。**要改的条款与落点：** 问题单 Q10 状态；里程碑 03 项四·1 的「开工前要先定的」补一句「读写并发归并行提交之后」。`crates/` 里程碑三不动。
**什么现象会推翻：** 用户把「读写并发」单独提到里程碑三（那一句 2026-10-01 的话要改）；或有人证明只读读者可以不经设备所有权改动就跑（例如读者各开一份自己的盘表句柄）——那一条路要另估代价，C214 的窗口问题仍然在。

## 12　复跑、命令与输出

### 12.1　模型复跑（接手核对）

前两条腿留在模型目录的十个脚本（`a1` 至 `a4`、`b1`、`b2`、`b3`、`b3b`、`b4`、`b4b`，外加共用的 `a_common.py`）我全部重跑过；新写 `c1_throughput_forms.py`、`c3_txg_gap_recount.py`、`c3b_txg_gap_variants.py`、`c6_sync_points.py`、`c8b_selfproof_paths.py`、`c9_serial_subset.py`、`c10_reader_writer_window.py`。每一个都经 `bash research/scripts/capped.sh 8 bash research/scripts/run-with-memory-cap.sh <上限> python3 <脚本>` 跑，退出码全 0；输出落在 `research/prompts/m3-conc-r1-sonnet-model/out/`（每份第一行可能是包装打印的「run-with-memory-cap: …」，不属于脚本输出）。两个包装提示我设了理由变量：峰值表里 `python3 …` 键的实测峰值只有 5 至 18 M、上限写 512M 超过 2 倍，设 `RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON`；同一会话里已有别的腿的包装在跑，设 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON`。

前一次留存输出可比的八份（`a1`、`a2`、`a3`、`a4`、`b1`、`b2`、`b3`、`b4`），去掉包装头一行之后与这一次逐行相同（`diff` 无输出）；`b3b`、`b4b` 之前没留输出，这一次是第一次落盘。

| 脚本 | 答哪格 | 它读什么 |
|---|---|---|
| `a1_e169_digest.py`、`a2_e169_wait_digest.py` | C2、C3 | E169 第一、二、三段产物 |
| `a3_e166_digest.py` | C4 | E166 产物 |
| `a4_e167_digest.py` | C7 | E167 三段产物 |
| `b1_c8_crossover.py`、`b2_c8_selfproof.py`、`c8b_selfproof_paths.py` | C8 | 页上的已量数；纯算术 |
| `b3_c5_admission_overlap.py`、`b3b_c5_split_clear.py` | C5 | 纯算术状态机，不读 `crates/` |
| `b4_c9_delete_interleave.py`、`b4b_c9_atomic_and_exclusive.py`、`c9_serial_subset.py` | C9 | 纯状态机，不读 `crates/` |
| `c1_throughput_forms.py` | C1 | 代数，取样点 |
| `c3_txg_gap_recount.py`、`c3b_txg_gap_variants.py` | C3 | E169 恢复输入产物（只读） |
| `c6_sync_points.py` | C6 | 人列的同步点表 |
| `c10_reader_writer_window.py` | C10 | 纯算术 |

### 12.2　命令与原样输出（冻结副本里跑）

```
$ grep -rnE 'Mutex|RwLock|Arc<|thread::|\bSend\b|\bSync\b' crates/singlefs-core/src | wc -l
0
$ grep -rnE 'checkpoint_entr|crossover|\bC58\b|write.?buffer|sort.?merge|WriteBuffer' crates --include=*.rs | wc -l
0
$ grep -rn 'ring_window\|RingWindow\|环窗口' crates/singlefs-core | wc -l
0
$ grep -n 'publish_admitted(' crates/singlefs-core/src/transaction.rs
5209:    publish_admitted(
$ grep -c '^P2' litmus/*.litmus
litmus/commit-publish-nofence.litmus:0
litmus/new-pool-file-creation-journal-implies-units-nofence.litmus:0
litmus/new-pool-file-creation-journal-implies-units.litmus:0
litmus/new-pool-file-creation-root-implies-units-nofence.litmus:0
litmus/commit-publish.litmus:0
litmus/new-pool-file-creation-root-implies-units.litmus:0
litmus/publish-returns-after-the-rotation-barrier.litmus:0
litmus/publish-returns-after-the-rotation-barrier-nofence.litmus:0
$ grep -rn 'TransactionOutput {' crates/singlefs-core/src --include=*.rs | grep -v 'impl\|struct\|fn '
crates/singlefs-core/src/transaction.rs:6888:    let output = TransactionOutput {
crates/singlefs-core/src/recovery.rs:2764:    let rebuilt = TransactionOutput {
crates/singlefs-core/src/mount.rs:5125:) -> TransactionOutput {
crates/singlefs-core/src/mount.rs:5187:    TransactionOutput {
$ python3 -c "print(281475379363984/1662648564)"
169293.3705044742
```

（`mount.rs:5125` 是函数签名，不是构造点；构造点三处：`transaction.rs:6888`、`recovery.rs:2764`、`mount.rs:5187`。）

### 12.3　手算核三个数（独立于脚本）

整体独占下 L 读法、C = 11 的「72 / 90」：3 个事务各「AL 相邻、P 在其后」，全部交错 6! ÷ 2³ = 90。C − R = 9，每个事务 d + t = 5。第二个事务准入时：若第一个已经发布，计数 = d + t = 5，需要 5 ≤ 9 − 5 = 4，不成立，被拒、不超卖；若第一个还没发布，计数只有取落点的 d = 4（t 要发布才进计数），需要 5 ≤ 9 − 4 = 5，成立，被准入，两个事务合计 10 > 9，超卖。不超卖的交错 = 先选第一个事务 x（3 种）、其 P 紧接其后、其余四个事件任意排（4! ÷ (2·2) = 6 种，后两个全被拒）= 18。90 − 18 = 72，与脚本一致。

另两个数同样手算对过：
- C9「并行盲写 28 条里 10 条违例」：D 的六步次序固定（PI CI P1 C1 P2 C2），W 的计划 PW 排在 CI 之前（看得到 nlink = 1）的交错数 = PW 落在「PI 之前」「PI 与 CI 之间」两个空隙、CW 落在它之后任一空隙：7 + 6 = 13（与脚本的「28 − 15 = 13」一致）；这 13 条里 W 整个在意图提交之前提交的（PW CW 都在 CI 之前）只有 3 条（两个都在 PI 之前、PW 在 PI 之前 CW 在 PI 与 CI 之间、两个都在 PI 与 CI 之间），所以违例 = 13 − 3 = 10。计划闸放行的 14 条 = 这 13 条加上 PW 在 C2 之后的 1 条，违例同样是这 10 条。
- C10「K = 6 暴露 3 条」：要有一跳的 Rp 与 Rc 之间塞满全部 6 次发布，3 个跳各一种排法，3 条。

### 12.4　与已有文字对不上的几处（并排整行抄）

| 项 | 一边 | 另一边 |
|---|---|---|
| E169 的「跳 txg 的输入数」 | 「输入里施加的记录跳过了一个 txg 的有 10 份（208 个状态」（`.claude/kb/experiments/169-并行提交的追加纪律与前缀恢复.md:254`） | 我复算（口径：施加的记录里相邻两条 txg 之差 > 1）：`合计跳 txg 的输入（两类各算一次，E169 页的数法）：份数 13，状态 112`（`research/prompts/m3-conc-r1-sonnet-model/out/c3_txg_gap_recount.out:6`）。口径不同，没复现 |
| D28 已定项 2「欠」里的 grep | 「`crates/` 里今天没有 overlay（`grep -rn overlay crates --include=*.rs` 零命中）」（`.claude/kb/decisions/28-挂载期承诺量.md:83`） | 同一条命令在冻结副本命中 6 行：`crash_judge_tables.rs`、`crash_identity.rs`、`crash_amplification.rs` 里的字节镜像 overlay，加 `admission.rs:314` 一行注释；准入的在飞 overlay 仍然没有实现，只是这条 grep 命令已经不准 |
| C58 的计数单位 | 「提交路径记一个「本次 checkpoint 的条目数」计数器，超过交点即判红」（`.claude/kb/checks-owed.md:59`） | 冻结副本没有 write buffer（第 12.2 节第二条命令零命中），按版本整片重建（第 9.1 节） |
| 事实表的行号 | 事实表：「.claude/kb/decisions/13-验证路线.md:45: **定案**：事务层先承诺串行提交」（`research/prompts/m3-concurrency-facts.md:23`） | 今天：「**定案**：事务层先承诺串行提交」（`.claude/kb/decisions/13-验证路线.md:51`） |


### 12.5　模型目录文件清单与 sha256（`sha256sum` 原样输出，路径相对 `research/prompts/m3-conc-r1-sonnet-model/`）

```
b8b8579f2d72636a3d6eaba22eec7863f9544eb7238e8f8c23b4a143f11c911b  a1_e169_digest.py
1b968333939fb2a803a4521f243d939a4c5bc2b30d7b4b708abc56f1218c59ec  a2_e169_wait_digest.py
c15478650c98c2e5d2b9399ca428051f6380dad025b203eb6d30eb6b9d1c8e67  a3_e166_digest.py
deb4e917fb03829dfa38ea0c5aa12ff878eae41b0b47f948301cbb715f9cdd65  a4_e167_digest.py
b540a3211a119cc3e168bf616bf2cec448e7bc1f24e87c1fb7fa55d703b8b20c  a_common.py
5b9ea004cf4d44b03ced3f7be8755526b8eec0270cd43653f3b038d3b81a1505  b1_c8_crossover.py
5167809ce6a3ad81bf45b95fa7c27cc1678ef8eaf2b023c9fb169b86533a99a6  b2_c8_selfproof.py
52a85fed088732cfb6cde000bce8864f42a0c29ff1f249b33597eee251d43ca2  b3b_c5_split_clear.py
f4c86dcf342ce2dd0e9b9efd999bde187d424edb71f24a799488d027f168335b  b3_c5_admission_overlap.py
fccc52310f472a965a2f444c3e2dfea16a43e2f6721f1f1515f69d7ae9d7fb2f  b4b_c9_atomic_and_exclusive.py
68f04ab4ec731788e317584fed2dc8714408c410877dc999d618aa8d3d2b1ba3  b4_c9_delete_interleave.py
320bb0bcafc20fc4c83391886b57ccdd9761d5639d924873fb26effe6d0dec33  c10_reader_writer_window.py
dc4fd5f8fc1ca08f2ba4ad1673356c916b51369215a759e7171d2a99dccbf795  c1_throughput_forms.py
a0d4ec390c290ef5a69a260b296064c80510471cec9a8d13f4df5bd6b9902c22  c3b_txg_gap_variants.py
7ac716115536a183ed97b96d041ca75e81eed92fb9e129fb9702f8cf998f5321  c3_txg_gap_recount.py
06a2c276b410854643c5174089dcafe6437d38e0293219622dc7c34bc7b07d74  c6_sync_points.py
090a3bdb16e009f4fca66109d346e633f9644e3a7bd8c817d684c093d2602f73  c8b_selfproof_paths.py
77e0f1eedfc2f80c4e29cc99114fef42bf7b815daceaf1773705c15bb8392d82  c9_serial_subset.py
b6bedaf630b91abf80b75c1b09a424748593c692b5d02f47b83d43199ff77248  out/a1_e169_digest.out
865fb3e92b0df4e57526e66a6b0669a88127ee2efa2436236c84f39a4d3160bd  out/a2_e169_wait_digest.out
a20a01a3585583733ed789a204debdfe2dc7253ddaed9b51d5256f3199562fd8  out/a3_e166_digest.out
4f522a79bf1224bb5baa088ebd892ad8ce18138e90a6f28262fbe4d0986f2424  out/a4_e167_digest.out
9c2f06cb4d6b871868510d8678ceb277b5cffcd6ed9520c4d079e3050344235e  out/b1_c8_crossover.out
e1be835392ef358c65e6b7a5feac8a830f5a221407c9c628ced95f8b34f4d3fb  out/b2_c8_selfproof.out
499d5e2d07c896df4b335e382c07f226eecb00e6c2e8762c0616a7b8d2bd71a4  out/b3b_c5_split_clear.out
cc7fb52d0142a21aa685a9ccefd9187a237d5ca98610fe95f2c4c7a67b22aa95  out/b3_c5_admission_overlap.out
b1f3c7d70ab15027bd63b63aacde1bba39a1258d710c90e1f7541fb4a6d14210  out/b4b_c9_atomic_and_exclusive.out
ccb6e6c65bf2f05a1fc0312b3c0f1d5d4239b2110ad4fc28bf664d10aea1f4fe  out/b4_c9_delete_interleave.out
00df3b7849637980a41e435ae1ea2b0a769fd830b5d0c14a05926f8caf50d2ba  out/c10_reader_writer_window.out
86642419da24836e98dab4ef6555ddc0d72218753e4ea12dd6612123015b00ed  out/c1_throughput_forms.out
2e80d397d702cf5858d9a1242fa0d479e5896eab9427802d77e23f0962effedf  out/c3b_txg_gap_variants.out
b363f7a85b36f4ecf01a1b3e7f60d6c9d95a250396b58dd1ffdcd49634d07784  out/c3_txg_gap_recount.out
7d8ac8c15de1cb66d6e6b87438bfcb030ee7a80315e71136e18ce43858d0ca35  out/c6_sync_points.out
5968ca84fc248e8f9cbdf0496b0599224b537bd01e1b99a731f34d618fcdcd05  out/c8b_selfproof_paths.out
afa89fe0b478406a159c62905b1f573bc49032bad8e3159a10d81f641055ca08  out/c9_serial_subset.out
```

## 13　问题单以外的空白

1. **写失败的级联（S10，建议另立 Q11）：** 分配器同一时刻至多冻结一次发布（「frozen_publish: Option<Box<FrozenPublish>>,」（`crates/singlefs-core/src/allocator.rs:1059`）），落盘失败时分配器换回备份并冻结（「allocator.freeze_publish(FrozenPublish {」（`crates/singlefs-core/src/transaction.rs:1156`）），后续发布「冻结着一次没重发的发布就在任何读写之前拒」（`crates/singlefs-core/src/transaction.rs:5048`），拒的函数是 `refuse_while_a_publish_is_frozen`（`transaction.rs:1126`）。甲（流水线）与丙里 N+1 已经在 N 之后的分配器上准备好了，N 失败它也得作废、号与 txg 重发；D13 已定项 2 只写了「作废是提交管道的内存状态，对并发后继可见」（`.claude/kb/decisions/13-验证路线.md:51`），说的是事务，没说发布层的冻结。
2. **乙整段取号时要知道记录条数（C2）：** 条数在准入之后才算出（`transaction.rs:6115`），E169 登记自己写「条数取号时未知时要按上界预留、多余的写占位，那一形态不建模」（`research/prompts/e169-preregistration.md:448`）。
3. **C58 的计数单位（C8）：** E17 的条目与本实现的建树输入记录不是一个单位，11200 要在本实现上重标。
4. **前缀核规约缺 txg 连号（C3）：** 「record.ordinal_within_publish.0 == 1」（`crates/singlefs-replay-prefix/src/lib.rs:230`）；D23 已定项 14 的「五条，缺一不可」要变六条。
5. **Q6 的第四个候选（C6）：** 进程内同步点（取号、准入与取落点同临界区、追加次序、fsync 等待、作废可见、写失败级联）既不是 litmus 也不是 Verus 能罩的，需要带运行期断言、可注入调度的测试；问题单三个候选都不含。
6. **E166 没建模逻辑内容的丢更新（C4、C9）：** 「记账 ACC 的丢更新与 BASE 的丢更新（Q-10）没建模」（`.claude/kb/experiments/166-并行提交的共享状态分法.md:172`）；C9 的模型里并行盲写 28 条交错 10 条违例，这一格是 Q4 任何一条臂的前提。
7. **环窗口的 W 与常量 R 在多个删除类发布同时在飞时够不够（C5，推的）：** 条款写的是「W 是一次删除类发布每盘新占槽数的上界」（`.claude/kb/decisions/28-挂载期承诺量.md:29`），R 是几何常量；多飞下 m 个删除类发布各占至多 W 时一个 R 罩不罩得住，D28 与 E168 都按一条删除发布在飞写，没写 m 个。

## 14　判定一览

被攻轮数一律零轮。「倾向」「推的」的内容不是判决，主 agent 采不采由主 agent 定。

| 格 | 判定 | 一句话（定案句或转过去的条件） | 依据 | 要改的条款与 `crates/` 落点 |
|---|---|---|---|---|
| C1 / Q1 | 转里程碑四 | 缺 C488 的本机吞吐与 fsync 等待数；带三候选的已有读数与四个待量参数（q、m、c ÷ S、f）；若必须表态倾向乙（今天顺序写就是同发形态，K1 不漏，等待最低）；「转」的推翻条件：本机量出 q(8) ÷ q(1) ≥ 2 且 S_pub 占一次发布耗时的大头 | E166 `…out:1173`；E167 `…r2.out:57`、`…second…r3.out:12`；E169 第三段 `…out:2035`、`:2036`；模型 c1 | 问题单 Q1；D13 已定项 2 射程；里程碑四在 `mounted_session.rs:243` 前加提交管道 |
| C2 / Q2 | 纪律现在定，实现选择转里程碑四 | 记录按事务号升序、以事务为单位整段追加；前两个析取支无违例；第三支没量、原句不动；甲整段对乙整段留给实现（乙要取号时知条数） | E169 `…r3.out:5053`、`:5101`、`:5029`；登记 `:443`、`:444`、`:448`、`:560`；第三段 `:2035` | D23 已定项 7 `:210`；D13 已定项 2 `:51`；里程碑四落在 `transaction.rs:1069` |
| C3 / Q3 | 恢复现在定（丙0 不换，丙2 出局）；Verus 补 txg 连号转里程碑四 | 取前缀仍是五条口径；「越过没施加的 txg」是规约缺口加无纪律历史上的真行为，候选纪律下 0，今天不改规约 | E169 `…r3.out:5060`、`:5040`；第二段 `…r2.out:2167`；对拍 `…direct-check…out:1`；`lib.rs:230`、`:232`、`:475` | D23 已定项 14 的注；里程碑四：`lib.rs` 规约与判定、`verus-mutations.tsv` |
| C4 / Q4 | 定义现在定，选臂转里程碑四 | 整体独占 = 锁罩住从读 `previous` 到换 current 的整段（A 读法），A字面漏 X 与 T 不算；六臂选哪个随 Q1 | E166 `…out:1172`、`:1173`、`:1174`、`:1064`、`:1083`、`:1163`、`:1157`；页 `:172` | 问题单 Q4；D13 已定项 2 射程；里程碑四锁段 `mounted_session.rs:243` 至 `:314` |
| C5 / Q5 | 持有规则现在定，实现与清零开关转里程碑四 | d 在取落点那一刻、t 在发布那一刻由在飞预留交给分配器计数与已承诺预留；环窗口常量不进预留；自证容量取 R + Σ(d + t) − 1 | 模型 b3、b3b；`admission.rs:329`、`:313`、`:314`；D28 `:19`、`:29`、`:36`、`:71` | D28 已定项 2 射程；C87 行；里程碑四 `admission.rs:318` |
| C6 / Q6 | 转里程碑四（四·1 现在不加 litmus） | 甲、乙设备序点全被现有三条 litmus 罩，新增点全在进程内；丙才有 S9，要三进程 litmus 且绑不到代码；Verus 补 txg 连号随 C3 | 模型 c6；`lkmm.sh:39`；`litmus/commit-publish.litmus:10`、`:13` | 里程碑四丙时加 litmus；Verus 与 C3 同批 |
| C7 / Q7 | 转里程碑四，条件句现在记 | Q1 取串落、同发沿用按根槽写切（K1 漏 0，10 格）；取多飞时 K1 漏、K2 小几何不漏但真实发布形 2.8×10¹⁴ 态，约为现有放量的 1.7×10⁵ 倍 | E167 `…r2.out:57`、`:35`；`…second…r3.out:25`、`:26`、`:108`；`…third….out:3`；`crash_verify_gpu.rs:304` | 问题单 Q7；D13 已定项 4 射程；里程碑四丙：`memory_pool.rs:993`、`crash_verify_gpu.rs:304` |
| C8 / Q8 | 现在定（四·1 的出口） | 交点取运行时配置，计数器接在会话发布成功之后（`checkpoint_entries` 六项，`judge` 严格大于），先在本实现上标定交点；T1 至 T4 自证 | 模型 b1、b2、c8b；`transaction.rs:2488`、`:6267`、`:6415`；`mounted_session.rs:314`；`checks-owed.md:59` | C58 行；D13 已定项 2 射程单位注；`transaction.rs`、新 `ceiling.rs`、`mounted_session.rs`、`walk.rs:5737`、harness 测试 |
| C9 / Q9 | 串行下现在定；并行转里程碑四 | 串行下第 4 条与第 6 条互为冗余保险（4 个位置：有一条违例 0，两条都没有 2 种）；并行下计划闸不够（违例 10 / 14），要提交时重判、读集校验或整体独占 | 模型 b4、b4b、c9；D8 已定项 16 `:459`、`:461`、`:463`；`transaction.rs:4915` | D8 已定项 16 射程；里程碑四把 `transaction.rs:4915` 的调用加到提交点 |
| C10 / Q10 | 转里程碑四 | 用户定「并发」= 多个事务并行提交；读写并发要先改设备所有权（`Cell`、`&mut`、零同步原语），C214 的断言串行下接不上；无保护暴露 K = 2 时 547 / 924 | 模型 c10；`mounted_read.rs:327`、`block_device.rs:116`、`:121`；`checks-owed.md:184`；`records/2026-10-01-删除与并发事务.md:41` | 问题单 Q10；里程碑 03 项四·1「开工前要先定的」补一句 |

## 15　没做什么

- **不判别的腿的格：** 反推（攻方腿）与校验（主 agent 判决时逐条现查）没做；上面所有「倾向」与「推的」都不是判决，我不替主 agent 采纳。
- **没跑重型测试与门禁：** 没跑 checker 档、`gate.sh`、Verus；没编 Rust。`c3_txg_gap_recount.py` 等只读产物或做纯算术。
- **没量任何本机吞吐与延迟数：** C1 要的分段计时与设备 flush 并发曲线、C8 的交点标定，我只给了怎么量；`Cell` 不是 `Sync` 是推的，没编译验过。
- **产物只读不复算：** E166、E167、E169 的产物行我只读、数按产物与页；`c3_txg_gap_recount.py` 是独立读恢复输入产物的复算，与页上 10 份 208 个状态没对上（第 12.4 节）。E167 第二段、第三段在两盘 G4 上的数没有独立 Python 再算（页自己写 155117520 条流数不动）。
- **模型的限度：** b2、c8b 里各项大小、c1 的参数是取样点，只看方向；c6 的同步点清单是我列的、没有第二条路径核；c10 的 K 口径有 ±1 的出入；b3 的 k = 3、一组 d + t 之外没扫；b4 只有一个覆盖写 W 与一条删除链。
- **没读 E168 里「(环长 + 1) × W」的推导**，所以第 13 节第 7 条只引条款原句、标「推的」。
- **没读禁读清单：** `research/prompts/m3-conc-r1-opus-output.md`、`research/prompts/m3-conc-r1-opus-model/`、`/tmp/claude-1000/m3-handover/conc-opus.md` 与攻方腿的提示都没读。读了接手摘要 `/tmp/claude-1000/m3-handover/conc-sonnet.md` 与 `conc-sonnet-2.md`。
- **写法偏差：** 报告按段追加写成（每段不超过 150 行），其后做过两次整份改写：一次把节号与「第 N 节」引用重编号、一次改正几处数与措辞，改的都是已写内容，没有新增内容；其余改动用 `research/scripts/replace-once.py` 定点替换。
- **没建编译目录、工作树、仓副本：** 草稿目录里只有 `cite.py`（取引文行号的小脚本）与中间输出，不作依据，不入库。

## 16　交回前 cite-check

命令：`python3 research/scripts/cite-check.py research/prompts/m3-conc-r1-sonnet-output.md --root <冻结副本根> --root <仓根> --background research/prompts/_m3-conc-r1-background.md`；末行：`✓ 核了 138 处引文，对上 138 处，没判 0 处（逐处列在上面）`。
