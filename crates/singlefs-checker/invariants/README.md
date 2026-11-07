# 不变量清单

**checker 是这份清单的可执行形式。** 这份清单每加一条，checker 就要加一个检查。
两者不同步的 commit 一律不收（见 [format-evolution.md](../../../.claude/rules/format-evolution.md)「硬约束」）。

这份清单按类拆成本目录下的 9 份文件，README.md 只放开头说明、状态、条数登记与「待补」；
每条不变量必须写成**可判定**的形式——能对着一个镜像回答"成立/不成立"。池级 checker 对一条不变量的判定有四种：成立、不成立、不适用（必须带理由，不许静默算成立）、部分没判（`InvariantVerdict::PartiallyJudged`，`crates/singlefs-checker/src/image.rs`）。部分没判是判到的那一部分都对得上、射程里另有一部分这张镜像上判不了，说明逐条列出没判到的是什么；它不是红，也不是成立：崩溃点重放与崩溃放量把它算成不红，崩溃点重放计数并进「不适用」那一格。GPU 判器每条记两种位：违例位，与判到位（`crates/singlefs-verification/src/crash_judge_checker_common.wgsl` 的 `judge` 记「评估过」、`mark_part_of_the_range_not_judged` 记「射程里有一部分判不了」，写回时判到位 = 违例位 ∪（评估过 ∖ 有一部分判不了），CPU 一侧是 `crates/singlefs-verification/src/crash_judge_conclusions.rs` 的 `pool_checker_judged_bits`），对拍两种位都逐位比：部分没判在 GPU 那一侧是违例位 0、判到位 0，成立是违例位 0、判到位 1。判到位只在当次的 GPU 结论与对拍字里，长期判定库存不存它以代码为准（今天崩溃放量从库里拼回对拍字时判到位读回是 0，`crates/singlefs-verification/src/crash_amplification.rs` 的「判到位不存，读回是 0」那一句）。报它的不止一条，逐条见各行（例：I-5.4（分配记录罩住的槽互不相交）、I-5.5（从 mkfs 水位建树那一版的 t1 落点）、I-9.28（全部可读根里同一树 ID 的诞生 txg 相同））。
写不成可判定形式的，说明还没想清楚，不许进这份清单。

## 哪类放哪

| 文件 | 类 |
|---|---|
| [01-块自描述.md](01-块自描述.md) | I-1（块自描述） |
| [02-根环.md](02-根环.md) | I-7（根环） |
| [03-journal.md](03-journal.md) | I-8（journal） |
| [04-校验和.md](04-校验和.md) | I-2（校验和） |
| [05-空间记账.md](05-空间记账.md) | I-3（空间记账） |
| [06-事务与崩溃一致性.md](06-事务与崩溃一致性.md) | I-4（事务与崩溃一致性） |
| [07-分配器.md](07-分配器.md) | I-5（分配器） |
| [08-加密不变量.md](08-加密不变量.md) | I-6（加密不变量） |
| [09-inode树结构.md](09-inode树结构.md) | I-9（inode 树结构） |

没有编号、或跨类的引用都指 README.md。

## 状态

**当前里程碑见项目根 `CLAUDE.md` 的「当前里程碑」那一句，这里不另记。** 磁盘格式已落成字节：mkfs、新池新建文件的写路径与冷启动恢复在 `crates/singlefs-core`；checker 在 `crates/singlefs-checker`，与实现只共享常量模块（D13（验证路线） 已定项 5）。
池级 checker（`walk::check_pool_image`）判 114 条，状态列写「已实现」的就是这 114 条（I-3.5（引用区间的精确性） 的出现规则与 ③ ④ 连同逆向都算在内）：每条配一份改坏了正好触发它的镜像（`crates/singlefs-verification/tests/checker_known_bad_images.rs`，删除线那几份的搭建与改法在 `crates/singlefs-verification/tests/common_deletion_in_progress/mod.rs`，I-4.4（意图只描述目标态） 与里程碑四自建 GC 那四条在 `crates/singlefs-verification/tests/checker_compaction_intent_and_relocation_must_red_worlds.rs`），崩溃点重放的每个崩溃状态都跑它。
I-7.7（系统配置实例代号不低于根环） 按 C322（取号那一步的屏障怎么放没有条款） 判成 ① ②，崩溃点重放里 0 个状态判红；I-3.1（已分配统计对得上） 与 I-7.8（根记录树 ID 水位不低于全池最大树 ID） 的 checker 读法写在各自那一行。
其余条目仍写「未实现」：它们要覆盖写、释放、回退与复用（释放、重用、残留）、journal 一侧的那一类检查、快照或加密，新池新建文件的镜像上判不出东西。

违例位是 128 位（4 个 u32 字，`crates/singlefs-verification/src/crash_judge_gpu.rs` 的 `POOL_CHECKER_VIOLATED_BIT_CAPACITY`），位 i 对 `IMPLEMENTED_INVARIANTS` 的第 i 条（`pool_checker_violated_bits` 读成 u128）；用了 114 位；`POOL_CHECKER_COVERED_INVARIANT_INDEXES` 列 114 个下标，就是 `IMPLEMENTED_INVARIANTS` 的全部 114 条（「覆盖表就是池级 checker 的每一条」那条编译期断言守着；下标 106 的 I-1.5（系统配置记管道身份） 由走读内核判，下标 107 的 I-3.18（分配记录的跨度等于它罩住的单元那一类的槽数） 由扫描内核判；这一句只说覆盖表与内核源码，挂 `gpu` 特性那一趟编不编得过、GPU 与 CPU 对拍过不过看提交时门禁不指定范围那一趟的输出），GPU 一侧的内核与下标（下标 65 的 I-3.5（引用区间的精确性） 走读内核在回退候选集的版本都走完之后逐对判；66 是 I-9.27（树表条目的头 ID 取值）、67 是 I-5.5（从 mkfs 水位建树那一版的 t1 落点）、68 是 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同）；69 是 I-9.5（删除队列里的号仍在树里）、70 是 I-3.4（可用空间扣待删占用）、71 是 I-4.7（删除中 inode 有账）、72 是 I-4.6（中间态其余不变量成立），这四条由扫描内核的删除态扫描在回退候选集的每一版上判，读不全的版不置位；73 是 I-4.4（意图只描述目标态），由扫描内核判 logged_ops 条目；84 是 I-9.30（inode 号水位的台阶行与水位、记录、谱系对得上），由扫描内核在最新根上判；85 是 I-9.31（退役树条目身份与子头相符），由走读内核在退役树的每条内部条目上判；86 到 89 是 I-9.32（退役记录合法）、I-9.33（谱系上的退役记录按身份不重复）、I-9.34（谱系上的退役记录不指本版活叶） 与 I-9.35（退役完备），由扫描内核在墓碑与谱系那一遍的每一版上判，I-9.35（退役完备） 扫完各版再跨相邻两版比，读不全的那一部分不置位；90 是 I-9.36（产品镜像里非最右谱系与退役容器装满），由扫描内核在墓碑与谱系那一遍判；91 是 I-9.37（测试镜像里谱系与退役容器不超旋钮值），编进旋钮值的那一份内核才判、没装旋钮的那一份恒不置位；92 是 I-1.13（扫描重建与遍历方向的索引相同），由扫描内核的第三个入口（`crash_judge_checker_scan_rebuild.wgsl` 的 `scan_rebuild_main`）判；93 是 I-3.6（deadlist 紧界）、94 是 I-3.7（活快照可枚举）、95 是 I-3.15（未释放记录罩住的单元由映射指在这一槽）、96 是 I-5.6（活整理意图的区域里没有签发之后分配的记录）、97 是 I-3.16（碎片度与全空段统计对得上）、98 是 I-1.14（同一映射 key 在各版里的单元逐字节相同），由扫描内核判；内核不置位、对拍不比的那张表（`crash_judge_conclusions.rs` 的 `POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS`）是空表；I-3.5（引用区间的精确性） ③ ④ 与逆向并进 65，扫描内核的 `s_judge_the_deadlist_against_the_head_and_the_live_snapshots` 判 deadlist 那几格、置的也是下标 65 那一位；出现规则 (b) 的谱系那一半 GPU 内核还没判，见 05-空间记账.md I-3.5 那一行），没有「内核还没判」的那张表；I-9.6（水位大于两处最大号） 墓碑那一半也在扫描内核里判、并进下标 41 那一位；编译期断言覆盖表按下标升序、一条不缺一条不重（第 k 项是 k）。

<!-- invariant-count -->
现共 134 条在用（编号至 I-9.40（extent 上段形状由有条目的 inode 号集合定），另有六条退役，编号不回收）。这一句是当前条数的权威登记位。

## 待补

这几类不变量还没写全。立账时卡着它们的决策现在的状态（以 `.claude/kb/decisions.md` 索引表为准）见各条括注；每一类现在写得出多少、清单里已有哪几条相关的，没有逐类核过：

- 反向索引一致性（D1（数据可移动性 / 反向索引） 已定：正向引用与反向索引必须双向可达且一致）
- 条带 / 多副本布局（D2（RAID 条带策略） 已定 21 项、未定 1 项）
- 快照可见性（D6（快照实现模型） 已定）
- 索引结构自身的结构不变量，如树的有序性、平衡性（D8（核心索引结构） 已定；inode 树与树表的有序性已有 I-9.12（分隔 key 落在孩子区间之外） 与 I-9.16（树表条目按树 ID 严格升序且合发号次序））
- 密钥层级与换钥（主密钥槽 80 与 KDF 标识 4 的宽度与落点由 D22（单元原子性怎么合成） 已定项 9 定死；还剩主密钥槽的内部构成与换钥不重写数据）
