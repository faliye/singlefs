# 意图记录的形态能不能装下第二个使用者（整理 / GC）：第一轮云端正推腿报告（2026-10-01）

<!-- doc-lint:not-numbers I1 I2 I3 I4 I5 I6 L1 L4 A B C D M1 K-U1 -->

轮名 m3-intent-r1，腿 = 云端正推（Sonnet），格 = 正文第三节 I1–I6。方案按 `crates/` 今天的实现来谈：冻结副本 `/tmp/claude-1000/m3-delete-r2/tree/crates/`，引代码写冻结副本里的行号（引文里的路径写成冻结副本的绝对路径 `/tmp/claude-1000/m3-delete-r2/tree/crates/…`，正文里不带前缀的 `crates/…` 行号同指冻结副本），引 kb 写 kb 文件自己的行号（2026-10-01 现查）。

## 零、前提核对

| 项 | 命令 | 原样输出 |
|---|---|---|
| 冻结副本没动 | `cd /tmp/claude-1000/m3-delete-r2/tree; sha256sum -c <快照>/crates-sha256.txt \| grep -vc ': OK$'` | `0`（`grep -c ': OK$'` 为 `210`） |
| 实现里没有意图机制 | `cd /tmp/claude-1000/m3-delete-r2/tree; grep -rn 'logged_ops\|LoggedOp\|deleted_inodes' crates --include=*.rs` | 两行，都是注释：`crates/singlefs-checker/src/walk.rs:6133` 与 `crates/singlefs-core/src/inode_tree.rs:16`；`grep -rn 'compat_ro' crates --include=*.rs \| wc -l` 输出 `0` |
| kb 与开工快照 | `sha256sum -c <快照>/kb-sha256.txt` | 13 份里 4 份 FAILED：`checks-owed.md`、`invariants.md`、`decisions/13-验证路线.md`、`decisions/26-后台整理与放置回收.md`，是同仓里别人在我开工之后改的（`git diff HEAD -U0` 的改动行：26 号只有第 146、227、235 行三处行号指引，不在我引的条款里；`checks-owed.md` 的改动删了两行，第 84 行以后的行号会移）。**我引 `checks-owed.md` 与 `invariants.md` 的行号以交回前最后一次 `cite-check` 为准**，报告末尾记那几份文件交回时的 sha256 |

我没有整读背景材料的 1.6 MB 附录：D8 / D26 / D3 / D23 / D28 / D15 / D21 / D13 / D18 / D24 的条款、`fs-design.md`、`format-evolution.md`、`layout/03-third-txn.md`、`invariants.md` 的相关行都是回 kb 原文读的。

一个词的用法：**「活 GC 意图」**指住在 logged_ops 里的整理意图；**「删除队列」**指 `deleted_inodes`（D8 已定项 16）。`CommitStep` 与 `TransactionUnit` 是两样东西：前者是封闭的提交步骤枚举（`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-core/src/transaction.rs:117`），后者是单元的角色（`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-core/src/transaction.rs:2006`），本报告里「不新增提交步骤」说的是前者。

## 一、六格推荐一览

| 格 | 推荐 | 一句话 |
|---|---|---|
| I1 | 一条整理意图 = 键（种类 2 + R 10）+ 值（c0 8 + segs0 8 + 进度提示 6 + 预留）；触发三量不进记录；全池同时至多 1 条活 GC 意图；R 取「设备 + 对齐的聚簇段起点槽」；进度只是提示、不是续做的依据 | 记录小、字段全由条款逼出；字节宽度都能从已有坐标（分配记录 key、checkpoint 号、统计量值）借来 |
| I2 | 推荐新名字的候选 D：logged_ops 树现在 day-1 零根注册（与三·2 的三棵树同一次基线重做），字段表留到整理落地；其次 A；B 的「现在定字段表」买不到字节；C 的三种回退写法各有代价 | 候选之间的差只在「新池新建文件字节再重做一次」「已写回的删除条款改几处」「回退时一棵树要不要按行种类拆」 |
| I3 | 按树分入口、共用发布路径：挂载交回之前只把 `deleted_inodes` 走到空；整理意图在挂载时只读出来并装进分配器的排他集合，续做放在会话的后台一步；整理走常规准入，式子要加一项 | `mount_writable` 里分配器先于 `establish_instance` 建好，活意图必须在重建那一版时读出来 |
| I4 | logged_ops 随现行那一版（D23 已定项 14 的表加一行，与墓碑树、谱系树同）；搬迁与删除批共用中央映射、串行提交下没有「同窗口重落」；R 排他对删除批落点的威胁靠 N₀ 与式子新项挡 | 整理意图丢了读者只拿保守答案（D21 已定项 10 的判据），所以回退取哪一版没有正确性代价，只有性能代价 |
| I5 | 意图记录不为 FTL / ZNS / GPU 留种类或字段 | 设备线由 mkfs 写死的 feature bit 分流，记录只描述「要清空哪段」，与谁来做无关 |
| I6 | 现在必须定死的只有两条：确认 K-U1 的「不立」只管 unlink 的意图载体；logged_ops 要不要与删除那三棵树同批 day-1 零根注册；其余留到落地 | 见第七节两张表 |

本报告里新冒出来的「事实」（前人没写的）一并列在这里，各自在对应节展开、各带推翻条件：
1. 全空聚簇段数在代码里只在**回收**时才涨（`allocator.rs:671`），释放只进 defer（`allocator.rs:556`）⇒ 产出谓词「全空聚簇段数(现在) − segs0 ≥ 1」在意图删掉那一刻几乎不可能为真（I1 第 2.5 节）。
2. 进度字段每更新一次 = 该次发布多写一个单元 = 崩溃状态数 ×4（I1 第 2.4 节）。
3. R 只有取「对齐的整段」，准入式子里「R 内被排除的空槽数」才是 O(1)（I1 第 2.2 节、I3）。
4. 码 2 节点只有一个条目宽（`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-core/src/unit.rs:166`），所以一棵树装多种定长条目要按最宽的补零；候选 C 因此逼出「现在就把整理的值宽定下来」（I2）。

## 二、I1　整理意图记录装什么

### 2.1 条款逼出了哪几样（原文，不改写）

| 条款 | 原文 | 逼出什么 |
|---|---|---|
| D26 已定项 1 第 2 条 | 「`segs0` 是写下这条整理意图那一刻的值」（`.claude/kb/decisions/26-后台整理与放置回收.md:26`） | `segs0` 一个值，住记录里 |
| D26 已定项 1 第 2 条 | 「无除法、无累加器」（`.claude/kb/decisions/26-后台整理与放置回收.md:26`） | 记录里不许有计数器 / 累加量；进度只能是位置，不能是「已搬 N 个」 |
| D26 已定项 2 | 「`c0` = 写下这条意图那一刻的分配代」（`.claude/kb/decisions/26-后台整理与放置回收.md:48`） | `c0` 一个值 |
| D26 已定项 1 第 3 条 | 「O(活意图数) 的点查，不是遍历」（`.claude/kb/decisions/26-后台整理与放置回收.md:27`） | 分配器每次候选落点都要读 R，所以 R 必须能放进内存里的一个小集合 |
| D26 已定项 1 射程 | 「这个状态在盘上没有独立落点，靠读活意图推出来」（`.claude/kb/decisions/26-后台整理与放置回收.md:31`） | R 排他的唯一落点就是意图记录 |
| D26 已定项 6 | 「整理进度 | 意图记录 | 它就是 D8（核心索引结构） 已定项 5 的续做点」（`.claude/kb/decisions/26-后台整理与放置回收.md:127`） | 进度住记录里（用户已定「存」） |
| D26 已定项 6 | 「runs、全空聚簇段数 | 记账树」（`.claude/kb/decisions/26-后台整理与放置回收.md:125`） | 触发用的三个量住**记账树**，不住意图记录 |
| C364 | 「四样的宽度与编码全仓一处都没有」（`.claude/kb/checks-owed.md:301`） | 字段表是缺的：R、c0、segs0、续做点 |
| D8 已定项 5 | 「意图声明目标状态，绝不声明增量。」（`.claude/kb/decisions/08-核心索引结构.md:122`） | I-4.4 的字面：进度不能是计数 |

**与正文第一节的一处不一致（冲突，两句并排）**：

| 来源 | 原文 |
|---|---|
| 正文 | 「触发用的三个观测量持久化进记账树与意图记录（已定项 6）」（`research/prompts/_m3-intent-r1-body.md:13`） |
| D26 已定项 6 | 「整理进度 | 意图记录 | 它就是 D8（核心索引结构） 已定项 5 的续做点」（`.claude/kb/decisions/26-后台整理与放置回收.md:127`） |
| D26 已定项 6 | 「runs、全空聚簇段数 | 记账树」（`.claude/kb/decisions/26-后台整理与放置回收.md:125`） |

条款里住进意图记录的是**整理进度**与（已定项 1）**segs0**，触发三量（runs、全空聚簇段数、不可回收字节）住记账树。正文那句把两件事并成了一句。判定：冲突；按条款读。

### 2.2 字段表（推荐形态；宽度「推的」＝借已有坐标，没有条款定过）

键与值：键 = 种类 + R，值 = c0 + segs0 + 进度 + 预留。键放 R 的好处：同一段至多一条意图（树里键唯一），不用另写「R 互不重叠」的检查。

| # | 字段 | 宽 | 必带吗 | 谁读 | 什么时候读 | 宽度从哪借 |
|---|---|---|---|---|---|---|
| 1 | 种类 | 2 | 只在这棵树装多种意图时才必带；整理独占一棵树时可省 | 挂载重建那一版时分派；checker 走树 | 每次挂载（`recovery.rs` 按种类取树的同一步）；checker 全扫 | 树表条目的「树的种类」也是 2（`.claude/kb/decisions/08-核心索引结构.md:281` 登记表） |
| 2 | R = 设备 4 + 聚簇段起点槽号 6 | 10 | **必带**（已定项 1 第 3 条） | ① 分配器每次取候选落点；② 准入式子里「R 内被排除的空槽数」；③ 续做每一批；④ checker | ① 每次分配；② 每次发布前；③ 每批；④ 全扫 | 分配记录的 key（设备 4 + 槽号 6，`.claude/kb/decisions/03-空间分配.md:120`），同一个坐标系，不许各定各的（`.claude/kb/decisions/03-空间分配.md:124`） |
| 3 | c0 | 8 | **必带** | 续做每批算目标态谓词；删意图那次核一遍；checker | 每批、末批 | 分配代是完整 checkpoint 号 8（`.claude/kb/decisions/03-空间分配.md:120`），与 `checkpoint_txg` 同宽同值域 |
| 4 | segs0 | 8 | **必带**（已定项 1 第 2 条「必须以值写进」） | 停机判断；观测 | 读法取决于 C373（见第 2.5 节），推荐：每条意图删掉那一刻读一次、决定签不签下一条 | 记账统计量的 value 8（`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-format/src/lib.rs:128`） |
| 5 | 进度提示 | 6 | 条款定了「存」，但**不是续做正确性的输入**（第 2.4 节） | 续做起点：从哪个槽开始扫 R | 每次挂载装入内存；每批开头 | 槽号 6，同分配记录 key |
| 6 | 触发观测量的值 | 0 | **不带** | — | — | 触发三量住记账树；起跑时读、停机时不读；停机只读 segs0 一个量 |
| 7 | 预留 | 落地时取 | 恒 0，读者遇非 0 拒收该条 | 读者 | 解条目时 | 同 I-9.7 / 树表条目 flags 的政策（`.claude/kb/decisions/08-核心索引结构.md:246`） |

条目宽 = 键 12（种类 2 + R 10）+ 值 22（c0 8 + segs0 8 + 进度 6）= 34，再加预留，预留取多少落地时定。一条记录用一个码 2 单元（16 KiB）装，树高 1。字节数都是推的。

字段 2 取「聚簇段、整段对齐」而不是任意槽区间，依据在代码里：

| 事实 | 位置 | 对 R 的含义 |
|---|---|---|
| 聚簇段长 64 槽、槽 16384 字节（1 MiB） | 「pub const CLUSTER_SEGMENT_SLOTS: u64 = 64;」（`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-format/src/lib.rs:271`） | R 取整段时长度是格式常量，不用写进记录 |
| 分配器已经把「不许分到哪几个段」作为一个段起点集合传进取落点函数 | 「cluster_segments: &BTreeSet<SlotNumber>,」（`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-core/src/allocator.rs:734`） | R 的排他可以是同类型的第二个集合，点查 O(活意图数) |
| 每段已分配槽数有增量计数 | 「self.used_per_segment[segment] -= 1;」（`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-core/src/allocator.rs:671`） | 取整段时「R 内空槽数」= 64 − 已分配 − 隔离 − 扣住，O(1)；任意槽区间要逐槽数 |
| 取落点的回落路径早已为 R 留了口子，现在恒空 | 「第一版 `R` 为空」（`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-core/src/allocator.rs:775`） | 不用新开一条排他路径 |
| 单元区起点对齐在聚簇段边界上，是类型不变量 | 「单元区起点在段边界上」（`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-core/src/allocator.rs:279`） | 段起点槽号 ÷ 64 取整即得段号 |

C364 第二轮三方判决推荐的 R 就是「一个聚簇段、按设备」（`.claude/kb/checks-owed.md:301`），与我从代码推出来的一致；但同一行写着「整张字段表交用户定案」（`.claude/kb/checks-owed.md:301`），我在 kb 里没找到用户对 R 的定案，所以字段 2 的**形态**是推荐、不是已定；同一行还写「第一版两盘同号时两者逐字节等价」（`.claude/kb/checks-owed.md:301`），所以第一版推荐与「全池同号」读法在字节上不分家。

### 2.3 活意图数的上界

推荐：**全池同时至多 1 条活 GC 意图。** 这是一条写者守、checker 数的策略，不是格式位——键里带 R，同一段至多一条已由键唯一保证；放宽到「每盘至多 1 条」时记录字节不变。

| 理由 | 依据 |
|---|---|
| 条款要的是有界，没有给数：「O(活意图数) 的点查，不是遍历」 | 「O(活意图数) 的点查，不是遍历」（`.claude/kb/decisions/26-后台整理与放置回收.md:27`） |
| 同类的先例取 1：活让渡区同时至多一个 | 「活让渡区同时至多一个 ⇒ O(1) 点查不是遍历」（`.claude/kb/decisions/03-空间分配.md:139`） |
| 提交本来串行，多条意图没有吞吐可买 | 「事务层先承诺串行提交」（`.claude/kb/decisions/13-验证路线.md:45`）；我没有量过并行意图的吞吐，这一格是推的 |
| segs0 的读法在多条意图下互相干扰：每条意图签发时的全空段数不同，一条腾出的空段会被另一条的停机谓词计入 | C373 第二轮在攻方腿模型里零命中的读法只有一种（`.claude/kb/checks-owed.md:308`）；那些模型是不是单条意图我没核，多条下的干扰是我的推理，没有模型 |
| 第一版两盘取同一个槽（分配器只发「两盘同槽」的落点），各盘各一条意图与一条等价 | 「let placement = Placement { slot, span: 2 };」（`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-core/src/allocator.rs:1410`）；各盘聚簇段要不要对齐是 D3 已定项 8 的待办 ① |

放宽的代价（备选「每盘至多 1 条」）：准入式子的新项按盘各一份、分配器排他集合多到盘数个；等第一版「各盘落点必须相同」的限制放开时才有意义。

### 2.4 生命周期

一条意图的每一次状态变化都是一次**普通发布**（D26 已定项 2：「不新增 `CommitStep` 成员；意图的目标态写成」（`.claude/kb/decisions/26-后台整理与放置回收.md:12`））。

| 步 | 条件 | 这次发布写什么 | 崩在它前 / 后 |
|---|---|---|---|
| 签发 | 三水位任一越线 ∧ 无活 GC 意图 ∧ 常规准入过 ∧ 全空聚簇段数 ≥ N₀（`.claude/kb/decisions/03-空间分配.md:143`） | logged_ops 加一键（种类, R），值 c0 = 签发发布的 txg − 1（2.5 c 推荐的定义）、segs0 = 这次发布前的全空聚簇段数、进度 = R 起点 | 前：没有意图，什么都没发生；后：意图在，R 排他在挂载时装进分配器 |
| 批 | 意图在；对 R 的分配记录取「分配代 ≤ c0 且未释放」的落点 | 目的地分配（不在 R）、中央映射改、源释放（进 defer）、分配记录树 / 记账 / 映射树 COW；**可选**更新进度提示 | 每批是原子的；重做的是**状态谓词**，不是「再搬 N 个」，所以幂等 |
| 末批 | 对整个 R 全量核目标态为真（C364 第二轮：「删意图前对整个 R 全量核目标态」（`.claude/kb/checks-owed.md:301`）） | 同一次发布里删键；树空则根指针置零（照 `deleted_inodes` 删空的先例，`.claude/kb/decisions/08-核心索引结构.md:454`） | 与 D8 已定项 5「删除意图必须与最后一批工作在同一个事务里提交」同形（`.claude/kb/decisions/08-核心索引结构.md:120`） |
| 删后 | 读 segs0，决定签不签下一条（C373 的读法，见 2.5 f） | 无 | — |
| 放弃（卡住时删意图而目标态未达成） | **条款没说** | — | 与 D8 已定项 5 同一句冲突，见 2.5 g |

进度提示的写入成本：一次发布里每多写一个单元（每盘一份、两盘），该次发布的崩溃状态数 ×4——

| 事实 | 位置 |
|---|---|
| 一次带单元的普通发布的状态数闭式 | 「上一次轮换段 3² − 1 = 8 + 单元段 2^n − 1 + 记录段 3 + 根槽段 1 = 2^n + 11」（`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs:102`）；只读引，没编译、没跑 |
| 取值例 | 「n = 24：16777227，n = 16：65547，n = 20：1048587，n = 28：268435467」（`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs:103`） |
| 我算的：`python3` 输出 `n=20 states=2^20+11=1048587`、`n=24 states=2^24+11=16777227`；`ratio n=22 over n=20: 3.9999685290777016` | 本次命令输出 |

所以：**进度提示要不要每批都写，是一个运行时旋钮，不进格式**——续做的正确性靠状态谓词（对 R 的分配记录现算），提示丢了或滞后只会让续做从 R 起点多扫几十条记录（R 一共 64 槽）。推荐第一版只在签发和删除两次写它，批间不写；这样批发布不多写 logged_ops 那一个单元。这与 D26 已定项 6 的「整理进度 | 意图记录」不冲突：字段在，写的频度不进条款。

### 2.5 从条款与代码推出来的七件事（各带推翻条件）

**a. 触发三量不进记录。** 起跑读三水位（记账树里的统计量，「触发用的三个量持久化，不靠挂载后重算」（`.claude/kb/decisions/26-后台整理与放置回收.md:121`）），停机只读 segs0 一个量。把 runs 一类的值也抄进记录，只会重新引入 C91 ② 说的增量量漂移。推翻：出现一条条款让停机读 runs 或不可回收字节。

**b. 产出谓词在意图删掉那一刻几乎不可能为真。** 条款：「D16（发布语义） 的 defer 段不算空」（`.claude/kb/decisions/26-后台整理与放置回收.md:25`）。代码：释放只做「defer 队列加」「self.deferred_slots += span;」（`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-core/src/allocator.rs:556`），每段已分配计数只在回收里减「self.used_per_segment[segment] -= 1;」（`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-core/src/allocator.rs:671`），全空段数数的是计数为 0 的段「pub fn empty_segments(&self) -> u64 {」（`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-core/src/allocator.rs:683`），回收要释放代落到下界之下「回收：一个已释放、释放代 ≤ max(F_生效, 环里最旧有效根) 的落点回到空闲」（`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-core/src/allocator.rs:655`）。⇒ 一批搬迁释放源落点的那一刻，全空聚簇段数不动；要等回收下界越过那次释放的 txg。D3 已定项 9 给的界是「界是 3 次改变用户可见状态的发布」（`.claude/kb/decisions/03-空间分配.md:187`），自己推的空发布不算这 3 次「文件系统自己推的空发布不算这 3 次」（`.claude/kb/decisions/03-空间分配.md:187`）。这不是与 D26 的冲突（条款本来就说 defer 不算空），是它的后果：C140 说的「意图完成不等于空段被腾出」在这里多一层——连意图完成之后立刻读也读不到产出（`.claude/kb/checks-owed.md:125`）。含义：停机谓词若原样读「全空聚簇段数(现在)」，删意图那一刻判不真，下一条会立刻签出去，直到回收追上；要么把读数口径改成「全空段数 + 整段都是空闲或 defer 的段数」（改 D26 已定项 1 第 1 条），要么接受多签几条。**这一格我不替定，列进 7.3 节「可以留到整理落地时定的」，但它是 segs0 读法的前提。** 推翻：我没有跑出实际序列；若 `reclaim_released_up_to` 在一次 GC 批发布里就能越过那次释放（例如 F 在同一次发布里被抬起），「几乎」要改成「有时」。

**c. c0 的定义缺一句，缺在「签发那次发布自己的落点」上。** 分配记录里「分配代」就是发那一块的 txg（「仍分配时是分配代；已释放时是释放代」（`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-core/src/allocator.rs:39`）；落点取出来就用调用方给的 `generation` 记一条，`crates/singlefs-core/src/allocator.rs:1411` 的 `self.record(placement, generation);`）。条款写「`c0` = 写下这条意图那一刻的分配代」（`.claude/kb/decisions/26-后台整理与放置回收.md:48`），C364 已经点过缺一句「等于意图事务被分进的那个 checkpoint 的 txg」的定义（`.claude/kb/checks-owed.md:301`）。如果 c0 取签发那次发布的 txg，那么那次发布自己新分的提交内生块（分配代也是这个 txg）若落进 R，就是「分配代 ≤ c0 且未释放」，一出生就违反目标态；R 排他要到那次发布生效才有。两种补法：① 定义 c0 = 签发发布的 txg − 1（所有此前已发布的代）；② 让排他从这次发布的分配起就生效（把待签的意图装进分配器的在飞合成，形态照 D28 已定项 2）。①是一句定义、零字节、零代码路径，推荐；留到落地写，但那一句必须写。推翻：若分配器的取落点在签发发布里天然绕开 R（例如意图签发永远在开了新段之后），则不需要。

**d. 整理意图按 D21 已定项 10 的判据落在派生态。** 判据：「重建不出这个值时，读者拿到的是保守答案还是错答案」（`.claude/kb/decisions/21-权威态与派生态的分界.md:216`）。意图丢了：分配器不再排他 R（D3 已定项 8 射程说这类分配器政策「违反它的实现写出的镜像完全合法」（`.claude/kb/decisions/03-空间分配.md:169`））、下一轮触发再签一条，读者拿到的是保守答案（少排他、重新整理），不是错答案。而派生态「丢了只是慢。不参与格式冻结」（`.claude/kb/decisions/21-权威态与派生态的分界.md:189`）。对照：unlink 那一支，权威事实是 inode 记录里的 nlink = 0，`deleted_inodes` 里的队列是派生的「队列 = inode 树里 nlink = 0 的号集合」（`.claude/kb/decisions/08-核心索引结构.md:452`）；整理这一支**连权威事实都没有**，意图本身就是全部。两件事的后果：① 回退时 logged_ops 取哪一版没有正确性代价，只有性能代价（I4）；② 整理意图的节点格式不进永久契约，只有树表条目（根，D21 已定项 9 把根算权威态）进，这是「字段表可以留到落地」的依据。推翻：若有人把整理意图判成权威态（比如要求 R 排他是正确性条款），②就不成立，字段表要在冻结前定。

**e. 码 2 节点只有一个条目宽。** 代码：节点头里条目宽只有一个 `u16`（「entry_width: u16,」（`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-core/src/unit.rs:166`）），读时核条目数 × 条目宽 = 声明长度（「if entry_count * entry_width != declared_length {」（`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-core/src/unit.rs:412`））；条款：D8 已定项 11 射程已经写过「条目宽字段只有一个 u16，按类两宽时这两句顶不住」（`.claude/kb/decisions/08-核心索引结构.md:334`）。⇒ 一棵 logged_ops 树装多种意图、各种意图值宽不同，只能按最宽补零；每加一种更宽的意图要么改条目宽（已写出的行全变），要么另起一棵树。墓碑树、谱系树、`deleted_inodes` 都是各自专用的树（「**墓碑树**：一棵专用树，day-1 零根注册进树表」（`.claude/kb/decisions/18-块里携带什么信息.md:565`）；「**谱系记录**：一棵专用谱系树，day-1 零根注册，随现行那一版、只增不删」（`.claude/kb/decisions/18-块里携带什么信息.md:569`）），这是项目里已经有的做法。推翻：若出现一种不要求定宽的码 2 变体。

**f. segs0 的读法（C373）。** C373 的现状是「三种读法三种结果，全仓没有一句选」（`.claude/kb/checks-owed.md:308`）；第二轮在两个攻方模型里零命中的是「谓词只在意图照目标态删掉那一刻判、决定签不签下一条」（`.claude/kb/checks-owed.md:308`）。我的推荐与它一致，理由是它不让谓词成为删意图的条件（删意图的唯一条件是目标态达成，与 D8 已定项 5 的「删除意图必须与最后一批工作在同一个事务里提交」同向，`.claude/kb/decisions/08-核心索引结构.md:120`），因此意图总能在有界批数内删掉。但它与 b 叠加：要同时定 b 的读数口径。

**g. 放弃路径条款没说。** D8 已定项 5：「删除意图必须与最后一批工作在同一个事务里提交」（`.claude/kb/decisions/08-核心索引结构.md:120`）。整理意图卡住（准入拒、设备错）时，R 一直排他：R 里的空槽既不能分配、也要从式子的可用里减掉（见 I3 的式子新项）；若要允许「删意图而目标态未达成」，这句要为整理另开一条例外，改的是概念模型的一句定义（D8 已定项 5 那一句）。三个候选都同样要改，所以不是候选之间的差。D3 已定项 9 第 1 条「不许有假性 ENOSPC：只要 `df` 报出的空闲 ≥ s，写 s 字节就必须成功」（`.claude/kb/decisions/03-空间分配.md:186`）还要求 `df` 也扣掉 R 里的空槽，否则 `df` 报的空闲里有分配器不肯发的槽——与 C318（影子账隔离的单元没进准入不等式）同形，那一条的修法已在代码里：「abandoned_root_exclusive: BytesOnOneDevice::of_slots(device_map.isolated_slots()),」（`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-core/src/admission.rs:332`）。

### 2.6 I1 判定与推翻条件

判定：**规则没说**（字段表整张缺，C364 开着）；**一处冲突**（正文 vs D26 已定项 6，2.1 节并排）。推荐形态见 2.2 节表。

什么现象会推翻 I1 的推荐：
- 条款要求停机谓词读触发三量里的别的量 ⇒ 字段 6 要加（2.5 a）。
- 用户把 R 定成全池同号、或任意槽区间 ⇒ 字段 2 的宽与对齐要重算；R 取任意区间时准入式子里的 O(1) 项落空（2.2 末表第三行）。
- 一次 GC 批发布里回收下界就能越过那次释放 ⇒ 2.5 b 的「几乎」改「有时」。
- 整理意图被判成权威态 ⇒ 2.5 d ②不成立。

## 三、I2　载体候选与各自的返工量

### 3.1 先分出「与载体无关」的整理落地返工（A / B / C / D 都要做，不算候选之间的差）

| # | 项 | 改什么 | 读到的位置 |
|---|---|---|---|
| 1 | 准入式子加一项「R 内被排除的空槽数」 | D28 已定项 1 的式子由九项变十项（式子带门禁标记 `gate:admission-formula`，`.claude/kb/decisions/28-挂载期承诺量.md:16`）；`admission.rs` 里每盘多一项（今天的「被抛弃根独占量」一项就是读分配器的隔离槽数：「abandoned_root_exclusive: BytesOnOneDevice::of_slots(device_map.isolated_slots()),」（`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-core/src/admission.rs:332`））；D3 已定项 7 合取 5 由恒真变活「整理意图期内那批槽不许当分配目的地」（`.claude/kb/decisions/03-空间分配.md:141`） | 同左 |
| 2 | 分配器排他 R | 取落点函数多一个段起点集合参数（今天已有 `cluster_segments`，`crates/singlefs-core/src/allocator.rs:734`），或第四种挡位（今天三种：「enum SlotBlockingBit {」（`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-core/src/allocator.rs:288`））；回落路径的口子已留：「第一版 `R` 为空」（`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-core/src/allocator.rs:775`） | 同左 |
| 3 | 整理引擎 | 批规划（扫 R 的分配记录取「分配代 ≤ c0 且未释放」的落点）、目的地分配、映射条目改、源释放；每批施加时先核（D8 已定项 5 第 4 条，`.claude/kb/decisions/08-核心索引结构.md:125`） | 无条款也无实现，推的 |
| 4 | 概念句与欠账 | 2.5 b / c / f / g 四处要写的句子；C364、C372、C373、C140、C91 还清；D26 已定项 3 底额占式子哪一项「底额自己占的空间算准入不等式的哪一项没定」（`.claude/kb/decisions/26-后台整理与放置回收.md:72`） | 同左 |
| 5 | 续做入口与 C1 的范围 | 第四节 | — |
| 6 | 崩溃流 | 新一条：签发 → 若干批 → 末批；另一条：回退跨过活意图。两态闭式 2^n + 11 | 2.4 节的表 |
| 7 | checker | 新不变量（3.4 节）；`IMPLEMENTED_INVARIANTS` 数组是定长 49「pub const IMPLEMENTED_INVARIANTS: [&str; 49] = [」（`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-checker/src/image.rs:68`），加几条就改几处 | 同左 |

### 3.2 候选的定义

| 名 | 定义 | 出处 |
|---|---|---|
| A | 维持现状：unlink 用 `deleted_inodes`；logged_ops 树等整理落地时再定字段表、再注册 | 正文第三节 |
| B | 现在就定 logged_ops 树：字段表、树 ID 与种类码、day-1 零根注册现在定死；unlink 不用它 | 正文第三节 |
| C | 合一：unlink 的意图也写成 logged_ops 的一行，`deleted_inodes` 改成派生队列或去掉 | 正文第三节 |
| D（我加） | **只做 B 的注册那一半**：logged_ops 树现在 day-1 零根注册（种类码、树 ID、树表一条），字段表与节点格式留到整理落地；unlink 不用它 | 本报告；起因 2.5 d：整理意图是派生态，节点格式不进永久契约，只有树表条目进 |
| C 随现行 | C 的回退写法一：回退时 logged_ops 取现行那一版（与墓碑树、谱系树同） | 本报告 |
| C 取 R_old | C 的回退写法二：回退时 logged_ops 取 R_old 那一版（与 `deleted_inodes` 同） | 本报告 |
| C 按种类合并 | C 的回退写法三：回退时 unlink 行取 R_old、整理行取现行，新树是两者之并 | 本报告 |

**D 与 K-U1 字面的张力（要交用户）**：K-U1 的状态写「用户已定（2026-10-01）：L4，只立 `deleted_inodes`，不立 logged_ops」（`research/prompts/m3-delete-r1-forks.md:9`），D8 已定项 16 第 1 条写「不立 logged_ops（K-U1 L4）」（`.claude/kb/decisions/08-核心索引结构.md:450`）。这两句的主语是 **unlink 的意图载体**（L4 的定义是「不另立意图记录」），D 并不让 unlink 写 logged_ops 的任何一行，所以我读成不冲突；但若「不立」按字面管到「不在树表里登记」，D 与 B 就碰了它，要用户确认「立」的射程。碰了就退到 A。

### 3.3 逐格对照

数据来源：行 1–3 读自冻结副本的代码与 `layout/03-third-txn.md`；行 4 的行数是 `grep -c` 现数；行 8 之外的数见 3.5 节的原型。

| # | 项 | A | B | C（三种写法共有部分） | D |
|---|---|---|---|---|---|
| 1 | **现在**（三·2 之前）多改的新池新建文件字节 | 无 | 树表条目 10 → 11（一条 200 字节，`TREE_TABLE_ENTRY_BYTES` = 200，`crates/singlefs-format/src/lib.rs:169`）、树 ID 水位 22 → 23、登记表 D8 已定项 9 加种类码 9；**0 个单元**（零根） | 条目数不变（10），`deleted_inodes` 那条换成 logged_ops 那条；水位不变 | 同 B 的树表部分 |
| 2 | **整理落地那天**多改的新池新建文件字节 | 同 B 现在的那一份，但这是对「树表 10 条 / 水位 22」那套已定字节的第二次重做 | 无 | 无 | 无 |
| 3 | 落地那天要不要改动已发布的意图字节 | 否：落地之前没有任何写者，logged_ops 没有行 | 否：字段表只在纸上，没有写者 | **看现在定不定整理的值宽**：unlink 行落盘时条目宽（码 2 节点只有一个条目宽，2.5 e）已固定；整理值比它宽则落地日已发布的 unlink 行全变宽。避免办法是现在就把整理值宽定下来，即把 C364 的全表现在还清 | 否 |
| 4 | **已写回的条款**要改几处 | 落地日：D8 已定项 9 登记表加一行；D8 已定项 8 ② 的发号次序句「八棵树的号从水位起连号发，次序照格式常量 11..18 那一组」（`.claude/kb/decisions/08-核心索引结构.md:242`）；D18 已定项 24 第 6 项的树表条数「**登记**：树表 7 → 10 条」（`.claude/kb/decisions/18-块里携带什么信息.md:570`）；`layout/03-third-txn.md` 树表那一行（第 15 行，「树表 7 → 10 条，树 ID 水位 19 → 22」）；`layout/01-first-txn.md` 树表条数几行（第 49、78、379、383 行） | 同 A 的那几处，**现在改** | 现在：`deleted_inodes` 出现在 6 份 kb 的 23 行里（第九节的命令现数）：D8 `08-核心索引结构.md` 第 15、26、108、172、204、446–456、466 行（第 418 行是 bcachefs 树清单，无关）、D23 第 376、385 行、D18 第 570 行、`layout/03` 第 9、15、40、42 行、`layout/01` 第 268 行、`invariants.md` 第 279 行（I-9.5）与第 292 行（I-9.18）；另有 I-4.7（第 172 行）的主语要换；回退那一句按写法重写 | 同 B |
| 5 | 不变量与 checker | 落地日：`TreeKindForReferenceScan` 加成员（今天它的注释就写了「day-1 注册的那三棵根指针为零、…走到这里就是它们真有了根节点，那时这条根的引用集合数不全」（`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-checker/src/walk.rs:2698-2699`））、发号次序数组 `[u16; 7]` 变长（`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-checker/src/walk.rs:52` 与 `crates/singlefs-core/src/recovery.rs:2188`）、新不变量 | 现在：发号次序数组；落地日：引用集合那一支 + 新不变量 | 现在：I-9.5、I-9.18、I-4.7 的主语由 `deleted_inodes` 换成 logged_ops 的 unlink 行；**C 随现行写法下 I-9.5 在合法历史上红**（3.4 节） | 同 B |
| 6 | 崩溃流增量 | 落地日：整理那两条流；树表条数不动流的状态数（零根不增单元） | 同 A，但注册那一步在现在 | **多一个耦合**：logged_ops 非空（有活整理意图）时，unlink 末批不能把根置零、要多重写一个节点（每盘 +1 单元，该次发布状态数 ×4），unlink 末批因此有「有 / 无活整理意图」两种形状，各要一条流（五条硬要求第 3 条「每条分支单独验，包括崩溃点重放」（`.claude/rules/fs-design.md:149`）） | 同 A |
| 7 | 回退（D23 已定项 14） | 落地日加一行：logged_ops 随现行 | 同 A，现在加 | 写法一：随现行；写法二：取 R_old；写法三：按种类合并（见 3.4） | 同 A |
| 8 | 与「一个事务层」「封闭提交步骤枚举」 | 都不新增 `CommitStep`；新树的节点要一个 `TransactionUnit` 角色（`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-core/src/transaction.rs:2006` 的穷举 `match` 要逐处补臂），角色不是提交步骤 | 同 A | 同 A | 同 A |
| 9 | 代价数（多几棵树 / 多几个单元 / 两态闭式 / 记录宽） | 落地日 +1 棵（总 11）；签发那次 +1 单元；批 0 或 +1；末批 0（零根）；记录宽落地日定 | 同 A，+1 棵在现在 | 总树数 10（少一棵 `deleted_inodes`）；unlink 行宽 = 整理最宽值（补零）；整理行宽同；末批 +1 单元的耦合形状 | 同 A，+1 棵在现在；记录宽落地日定 |
| 10 | 有节点的新树要接几棵 | 两棵：三·2 接 `deleted_inodes`，整理落地日再接 logged_ops | 同 A，logged_ops 那一棵的接线在落地日 | **一棵**（unlink 与整理共用）——这是 C 唯一省下来的东西 | 同 A |

行 10 的量：一棵有节点的树要接多少，按记账树这棵现成的树数它在冻结副本里被引用的行数（是上界，不是改动数）：核心里 `grep -rn 'TransactionUnit::AccountingTree\b'` 在 `transaction.rs` 17 行、`admission.rs` 3 行、`write_accounting.rs` 2 行、`mount.rs` 1 行，共 23 行；`grep -rn 'TREE_KIND_ACCOUNTING' singlefs-core/src` 在 `recovery.rs` 7 行、`transaction.rs` 2 行、`records.rs` 1 行；checker 的 `walk.rs` 8 行（`TREE_KIND_ACCOUNTING` 与 `TreeKindForReferenceScan::Accounting` 的命中）。C 省的就是第二棵这一份；它换来的代价在行 3、5、6、7 与 3.4 节。

### 3.4 回退写法与合法历史（C 的三种写法要分开判）

三段合法历史（都能由管理员回退造出，回退候选集见 D23 已定项 14）：
- H1「删 → 回退到删之前」：文件 F 的 unlink 已发出意图（inode F 的 nlink = 0、队列里有 F），管理员回退到意图之前的根 R_old。
- H2「整理在途 → 回退」：现行那一版有一条活 GC 意图，R_old 没有。
- H3「回退跨过一条已完成的整理」：R_old 那一代有一条活 GC 意图，现行那一版没有。

D23 已定项 14 的回退写法是：「新根的用户可见树（inode 树、extent 树）与 `deleted_inodes` 的根指针取 R_old 那一版的；墓碑树、谱系树随现行那一版」（`.claude/kb/decisions/23-journal的角色与格式.md:376`）；代码里按种类挑树就是这一句：「root: if entry.kind == TREE_KIND_EXTENT || entry.kind == TREE_KIND_INODE {」（`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-core/src/mount.rs:4972`），别的种类走 `else` 照现行那一版。所以 **A / B / D 里的 logged_ops 不用动回退代码也自然随现行**；`deleted_inodes` 要往那个 `if` 里加一个种类（三·2 本来就要做）。

| 候选与写法 | H1 | H2 | H3 | 要改的不变量 | 判（正文第五节判据） |
|---|---|---|---|---|---|
| A / B / D：`deleted_inodes` 取 R_old，logged_ops 随现行 | 队列取 R_old = 空，inode 取 R_old 里 nlink = 1，I-9.5 成立 | 意图随现行留着，R 排他不变；复活的单元分配代取回 R_old 那一版账里的值（「分配代写回 R_old 那一版账里的分配代」（`.claude/kb/decisions/23-journal的角色与格式.md:382`）），可能 ≤ c0 且落进 R，目标态又假，续做重扫 R，有限步内收敛（回退是有限次的管理员动作） | 现行没有意图，R_old 的那一条不复活；无事发生 | 不动 I-9.5「`deleted_inodes` 里的每个 inode 号在同一棵树的 inode 树里存在且记录的 nlink = 0」（`.claude/kb/invariants.md:279`） | 收 |
| C 随现行 | **意图行留着、inode F 回到 nlink = 1**：I-9.5 的 logged_ops 版在合法状态上红；续做若信行就会按「删 F」去释放一个活文件的单元（D8 已定项 5 第 4 条只要求先核「要删的东西还在」，不核 nlink） | 同上 | 同上 | I-9.5 要么削弱成「行可以指向 nlink = 1 的 inode」（失去它要抓的那一类错：行在而 nlink ≠ 0），要么续做加一道「先核 nlink = 0」的守卫 | **不收**：「续做判不出该做什么、或做错」（正文第五节第 2 条）在 H1 上成立，除非削弱不变量并加守卫 |
| C 取 R_old | 同 A | 现行的活意图随回退消失，分配器的 R 排他集合要在回退发布之后重装（多一个状态迁移）；之后触发再签一条 | R_old 的旧意图复活，带着陈旧的 c0 / segs0 / R；续做按状态谓词重判，收敛；segs0 判的是过期的值，结果只是多签或少签一条 | 无 | 收；写明：整理意图的时间线被绑在 inode 树上，回退一次丢掉在途进度 |
| C 按种类合并 | 同 A | 同 A | 同 A | 「回退后 unlink 行集 = R_old 的 unlink 行集」要两个镜像才判得出，单镜像 checker 判不了，与 C614 同组「回退时墓碑树取错与删完对象无墓碑单镜像判不了」（`.claude/kb/checks-owed.md:485`） | 收；最贵：回退发布多重写一个 logged_ops 节点（+1 单元，且要进 ckpt_cost 的 Σ 名单，照谱系容器的先例：「谱系容器归固定点角色、进 ckpt_cost 的 Σ 名单」（`.claude/kb/decisions/23-journal的角色与格式.md:385`））、一段按种类合并的代码 |

新增的单镜像可判不变量（A / B / C / D 共有，落地时进 `IMPLEMENTED_INVARIANTS`）：① logged_ops 里每个键的 R 对齐（槽号是 64 的整数倍）、设备在池里；② 活 GC 意图至多 1 条；③ c0 小于本根的 txg；④ 预留段恒 0（同 I-9.7 的政策，`.claude/kb/invariants.md:281`）。**判不了的**（要两个镜像）：进度单调、segs0 不随意图推进而变。I-4.4「意图只描述目标态」与 I-4.6「中间态其余不变量成立」今天是「未实现」（`.claude/kb/invariants.md:168`、`.claude/kb/invariants.md:171`），整理落地时一起实现；I-4.7「删除中 inode 有账」是 unlink 专用（`.claude/kb/invariants.md:172`），整理没有对应物。

### 3.5 原型：给树表多注册一棵零根的树要碰多少

**目的**：量 A 落地日那一次、B 与 D 现在那一次「新池新建文件的树表多一条」到底碰多少代码与测试。方法：冻结副本上加一棵 logged_ops（种类码 9、树 ID 19、水位 20、零根），`cargo check -p singlefs-format -p singlefs-core -p singlefs-checker -p singlefs-harness --all-targets`，再跑 harness 档看红的（全量中途按第十一节停了，改跑点名的）。没碰 `singlefs-checker-tier`。

**静态足迹**：今天一棵 day-1 零根树（deadlist）在冻结副本里出现的行数，`grep -rn 'TREE_KIND_DEADLIST\|TREE_IDENTIFIER_DEADLIST' crates --include=*.rs | wc -l` 输出 `19`；按文件：`singlefs-format/src/lib.rs` 3、`singlefs-core/src/recovery.rs` 5、`singlefs-core/src/transaction.rs` 4、`singlefs-core/src/records.rs` 1、`singlefs-checker/src/walk.rs` 2、`singlefs-checker-tier/src/bin/e158_root_choice_repair.rs` 2、`singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs` 2。

**原型结果**（补丁与复跑脚本：`research/prompts/m3-intent-r1-sonnet-model/proto-day1-logged-ops-tree.patch`、`research/prompts/m3-intent-r1-sonnet-model/rerun.sh`；我跑的两批的摘录在同目录 `results.log`）：

- 补丁：`grep -c '^[-+][^-+]' proto-day1-logged-ops-tree.patch` 输出 `45`，碰 5 个文件：`crates/singlefs-format/src/lib.rs`、`crates/singlefs-core/src/records.rs`、`crates/singlefs-core/src/recovery.rs`、`crates/singlefs-core/src/transaction.rs`、`crates/singlefs-checker/src/walk.rs`；`patch -p1 --dry-run` 在冻结副本上全部命中。
- `cargo check --offline -p singlefs-format -p singlefs-core -p singlefs-checker -p singlefs-harness --all-targets`：`Finished`，零 error（harness 的测试源码不改也编得过）。
- format、core、checker 三个包的 `--lib` 用例：都是 `rc=0`。
- harness 点名的 7 个用例二进制（静态 grep 找出有字面 7 / 19 的文件，加一条树表次序用例）：

| 二进制 | 结果 | 红在哪 |
|---|---|---|
| `tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker` | 全过 | — |
| `overwrite_in_one_instance` | 10 过 2 红 | `:107` 水位「left: 20 right: 19」；`:321` 树表条数「left: 8 right: 7」 |
| `new_pool_file_creation_publish` | 6 过 2 红 | `:540` 水位「left: (20, 0) right: (19, 0)」；`:1201` 树表条数「left: 8 right: 7」（用例名就叫 `tree_table_holds_seven_entries_keyed_by_tree_identifier`） |
| `unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over` | 22 过 1 红 | `:328` 钉死的两块盘整份镜像 CRC32：「left: 443711748 right: 1783297687」（常量 `DEFAULT_RING_NEW_POOL_FILE_CREATION_IMAGES_CRC32`，`:296`） |
| `writable_mount_of_a_formatted_pool` | 14 过 1 红 | `:1425` 最高树 ID「left: 19 right: 18」 |
| `corrupt_on_disk_content_is_refused_instead_of_panicking` | 30 过 1 红 | `:443`：用例把「deadlist 条目」的树 ID 改成水位去撞水位不变量，原型里 deadlist 不再是最后一条，撞到的是 I-9.16 |
| `rollback_by_a_forward_publish` | 0 过 22 被 ignore | 字面 19 在 `:1271`，落在被 ignore 的用例里，没跑到 |

基线对照：未打补丁的冻结副本上跑 `unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over` 与 `corrupt_on_disk_content_is_refused_instead_of_panicking` 两个二进制，各 `23 passed; 0 failed` 与 `31 passed; 0 failed`（`results.log` 末尾）；另外三个红的二进制我没在基线上跑，红的断言本身是 7 / 19 / 18 的字面数，基线值与它们一致是读源码看的。

读法：**7 个用例红，没有一个是逻辑错**：5 个是树表条数 / 水位 / 最高号的字面数，1 个是整份镜像的 CRC 钉死值，1 个是「deadlist 是树表最后一条」的隐含前提。所以「再重做一次新池新建文件的基线」的实际内容 = 改约 7 处断言、重钉 1 个 CRC、改 `layout/01-first-txn.md` 里树表条数那几行、重跑 E142 干跑（E142 我没碰）。三·2 本来就要把这批断言从 7 条改成 10 条、水位 19 改成 22、重钉同一个 CRC，所以候选 D / B 在那一次里多的只是把 10 写成 11、22 写成 23；候选 A 则要在落地日对同一批地方再改一遍。崩溃枚举的状态数不动是推的（零根不增单元），没有量。**没跑到的**：其余 44 个提到树表 / 水位 / 种类常量的 harness 用例二进制（在原型副本里 `grep -lE 'tree_table|TreeTable|TREE_TABLE|WATERMARK|树表|tree_identifier|TREE_KIND|TREE_IDENTIFIER' tests/*.rs | wc -l` 输出 `51`，减去点名的 7 个；全量跑到一半按第十一节停了）、`checker_known_bad_images`（跑了十几分钟被后台时限停掉，没出结果）。

### 3.6 I2 判定

| 候选 | 判 | 一句话 |
|---|---|---|
| A | 收 | 落地日多一次树表重做（3.5 节的数），其余同 D；符合 K-U1 字面 |
| B | 收，不推荐 | 注册那一半同 D；字段表现在定死买不到字节（没有写者），却要现在替用户定 C364 里还开着的几格（R 的形态、segs0 的读法、c0 的定义）；定错的代价是 kb 与检查，不是字节 |
| C 随现行 | **不收** | H1 上续做做错或 I-9.5 在合法状态上红 |
| C 取 R_old | 收（不出局，交用户） | 推翻 K-U1 L4 的依据我没找到；它换来少接一棵有节点的树（3.3 节表行 10），代价见该表行 3、4、5、6、7 与 3.4 节的表 |
| C 按种类合并 | 收（不出局，交用户） | 最贵 |
| D | 收，**推荐** | 注册与三·2 的三棵树同一次基线重做；字段表等有写者时再定；与 K-U1 字面的张力要用户确认 |

推翻「推荐 D」的现象：用户确认 K-U1 的「不立」管到树表登记（则退 A）；或 3.5 节量出的注册成本比落地日多做一次的成本还大（则 A）；或整理意图被判成权威态、字段表要在冻结前定（则 B 的字段表要在冻结提交前补，不必现在）。

对照正文第五节的五条跑前判据（`research/prompts/_m3-intent-r1-body.md` 第五节）：
1. 「整理落地那一天要改动已发布的意图字节」：A、B、D 不会（落地之前没有写者）；C 取决于现在定不定整理的值宽（3.3 表行 3），不定则会，返工量 = 已发布的 unlink 行全部变宽。
2. 「某段合法历史上续做判不出或做错」：C 随现行在 H1 上打中（3.4 节）；A、B、D、C 取 R_old、C 按种类合并没打中。
3. 「为 FTL / ZNS / GPU 立买不到失败模式改善的格式分支」：四个候选都没有（第六节）。
4. 「C 推翻 K-U1：写明推翻依据与返工清单交用户」：C 的返工清单在 3.3 表行 4、5、6、7 与 3.4 节；**推翻依据我没找到**——C 唯一的好处是少接一棵有节点的树（行 10），而它带来的三样代价（值宽、回退、末批形状耦合）A / B / D 都没有。
5. 「腿提的新形态算被攻过零轮」：候选 D 与「C 的三种回退写法」是我加的，零轮。

## 四、I3　续做语义两类

### 4.1 今天的可写挂载顺序（冻结副本）

读阶段之后依次是：重建上一版「let previous = rebuild_previous_version(devices, unit_area_start, &effective_root, own_record)?;」（`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-core/src/mount.rs:4487`）→ 用 `&previous` 重建分配器「let rebuilt = rebuilt_allocator(」（`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-core/src/mount.rs:4502`）→ 把分配器交给 `establish_instance`「establish_instance(」（`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-core/src/mount.rs:4521`）；其中暖机循环起头是「let mut current = row_publish.clone();」（`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-core/src/mount.rs:3938`），之后是推抬 F 的一支「push_floor_raises_after_the_row_publish(」（`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-core/src/mount.rs:4005`），最后交回「Ok(Mounted {」（`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-core/src/mount.rs:4018`）。

两条推论（读代码推的）：
1. **分配器先于 `establish_instance` 建好，且只吃 `previous`** ⇒ 活 GC 意图必须在「重建上一版」那一步就读出来、带进 `previous`，分配器重建时才装得上 R 排他；取号之后的写行发布与暖机发布本身就要分配，它们之前排他必须已在。这要求 logged_ops 是树表里登记的树（挂载重建那一步已经按种类取每棵树：「deadlist: tree_of(TREE_KIND_DEADLIST)?,」（`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-core/src/recovery.rs:2374`）），放在 journal 或系统配置里都读不到（C364 第二轮同一句：「logged_ops 只有放在根下的一棵树时站得住」（`.claude/kb/checks-owed.md:301`））。
2. **排他状态必须住在 `PoolAllocator` 里，不能住在会话或挂载输出里**：取号前的预演拿的是分配器的拷贝（「fn dry_run_of_the_publishes_after_acquisition<Device: BlockDevice>(」（`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-core/src/mount.rs:2874`），参数 `allocator: &PoolAllocator`），预演取的落点与真发的落点逐次相同由一条断言钉住「&placements_taken, planned,」（`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-core/src/mount.rs:3974`）；排他若在拷贝之外，预演会取到 R 里的落点而真发不会，这条断言就红。

### 4.2 按树分入口，不按种类分派

| 项 | unlink（`deleted_inodes`） | 整理（logged_ops） | 为什么分开 |
|---|---|---|---|
| 读哪棵树 | `deleted_inodes` | logged_ops | 两棵树，各取各的根（2.5 e：一棵树一个条目宽） |
| 什么时候读 | 重建上一版那一步 | 同 | 分配器要用 |
| 谁驱动 | `mount_writable` 在写行与暖机之后、交回之前，D8 已定项 16 第 6 条写：「在写行与暖机之后、交回之前把队列走到空；不进取号前预演；会话里的队列每次从现行那一版读、不缓存；崩溃注入加」（`.claude/kb/decisions/08-核心索引结构.md:455`） | 会话的后台一步（`MountedSession` 一类，今天的会话入口是「pub fn publish_user_change<Device: BlockDevice>(」（`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-core/src/mounted_session.rs:238`），还没有后台入口） | 整理是第三格、无戒律：「整理属第三格，无戒律——可以遍历、可以慢、不参与任何即时判定」（`.claude/kb/decisions/26-后台整理与放置回收.md:178`），不该挡挂载 |
| 挂载交回时队列 | 必须空 | 可以非空；R 排他必须已装 | 同上 |
| 空间准入 | 删除类不判式子，落点照判（4.4） | 常规准入（「整理不是释放路径，它走常规准入，批次最坏占用计入」（`.claude/kb/decisions/26-后台整理与放置回收.md:67`），见该行） | 释放不能申请空间（D3 已定项 17），整理要新落点 |
| 崩溃注入 | 新阶段 `WritableMountPhase::Continuation`（D8 已定项 16 第 6 条） | 会话 tick 的新入口；重挂载之后再 tick | 整理没有「挂载里续做」这一阶段 |

共用的是**发布路径**（同一个 `CommitStep` 枚举、同一套准入函数，只是准入类不同），不共用「续做循环」：不写一个「走完所有意图」的泛化循环，免得 unlink 的「挂载交回前必须空」被带到整理上。这与 `.claude/rules/fs-design.md` 的「一个事务层，所有结构共用」同向：被约束的是事务层（共享的状态机与封闭枚举），不是调度（`.claude/rules/fs-design.md:98`：「一个共享的事务抽象状态机 + 一个封闭的提交步骤枚举」）。

### 4.3 C1 的字面与整理冲突（两句并排）

| 来源 | 原文 |
|---|---|
| C1 | 「崩溃点重放里，任一含未完成意图的镜像正常挂载后没跑续做即判红」（`.claude/kb/checks-owed.md:17`） |
| D26 已定项 12 | 「整理属第三格，无戒律——可以遍历、可以慢、不参与任何即时判定」（`.claude/kb/decisions/26-后台整理与放置回收.md:178`） |

C1 字面读：含活整理意图的镜像挂载之后没把意图续做完就判红。整理意图本来就可以挂在盘上很久，C1 的字面把整理拖进挂载。D8 已定项 5 第 2 条的原意是「续做是每次恢复都跑的常规动作，不是修复手段」（`.claude/kb/decisions/08-核心索引结构.md:123`），重点是**每次恢复都经过续做那段代码**，没写「必须在交回之前做完」。判定：C1 字面与 D26 已定项 12 **冲突**；按原意读两者相容。建议 C1 的判红谓词按意图种类写：对 `deleted_inodes`：交回时非空判红；对 logged_ops：交回时「R 排他已装进分配器，且会话有一条路径能继续这条意图」，后一半在 harness 层判（会话不是盘上状态，checker 判不了）。五条硬要求第 2 条还要一个只供测试的开关能强制走进整理续做那条路（`.claude/rules/fs-design.md:146`：「每条分支必须能被测试强制进入」）。

### 4.4 空间准入：三类发布

| 类 | 例 | 式子 | 落点那一判 | 出处 |
|---|---|---|---|---|
| 删除类 | 意图、各批、挂载里的续做批 | 不判 | 判 | 「删除类发布（意图、各批、挂载里的续做批）与管理员回退也不判式子」（`.claude/kb/decisions/28-挂载期承诺量.md:36`） |
| 写入类 | 建 inode、顺序写、覆盖写 | 判（按发布之后的标签） | 判 | `research/prompts/m3-delete-r3-main-verification.md` 的 G1-写入类发布一行（零轮） |
| 整理类 | 签发、批、末批 | **判**（常规准入） | 判 | `.claude/kb/decisions/26-后台整理与放置回收.md:67` |

整理类的需求取「这次发布新写的全部槽」「需求 = 这次发布新写的全部槽」（`.claude/kb/decisions/28-挂载期承诺量.md:36`）：目的地新槽 + 树节点；源落点的释放进 defer、不发信用「窗口内产生的释放不发信用」（`.claude/kb/decisions/03-空间分配.md:110`）。所以满池上整理会被准入拒，这是对的：整理不是释放路径。两处要在整理落地时写下来、与候选无关：**① 式子要扣 R 内被排除的空槽**（3.1 表第 1 行，2.5 g）；**② 整理被拒时能不能请求抬 F**：整理的产出要等回收下界越过释放代（2.5 b），而抬 F 缩短回退深度，这是一个用户可见的代价，条款没说整理有没有这个权利（规则没说）。

### 4.5 I3 判定与推翻条件

判定：**规则没说**（调度从哪读、两类怎么分入口没有条款）；**一处冲突**（C1 字面 vs D26 已定项 12，4.3 节并排）。

什么现象会推翻：
- 推论 1：若有人提出分配器在取号之后才装 R（那么写行与暖机会落进 R），4.1 第 1 条要改；
- 推论 2：若预演的断言（`mount.rs:3974`）被放宽成「不同也行」，排他可以放进拷贝之外；
- 4.2 的「不泛化循环」：若里程碑要求「所有意图类型共用一个交回前必须空的闸」，则 C1 不分种类，整理要在挂载里续做到空——那等于把整理拖进第一格，与 D26 已定项 12 冲突。

## 五、I4　与删除、回退的交互

### 5.1 整理搬迁一个正在删除的文件的单元

提交串行（「事务层先承诺串行提交」（`.claude/kb/decisions/13-验证路线.md:45`）），不存在两次发布「同一窗口」交错；窗口只能是相邻两次发布。三种次序，逐个看：

| 次序 | 发生什么 | 靠什么保证对 |
|---|---|---|
| 删除批在前（txg t），整理批在后（t+1） | 删除批把尾部 K 个单元释放（分配记录改成「已释放 + 释放代」），整理批对 R 现读分配记录，已释放的不再是「分配代 ≤ c0 且未释放」，跳过 | 整理的目标态谓词就是读分配记录现算，不带旧计划 |
| 整理批在前（t），删除批在后（t+1） | 整理批把单元 u 搬到 R 外的新槽、改中央映射；删除批释放 u 时按**上一版的映射**查位置：「上一版里这些角色的落点从上一版的映射节点按 key 查出来」（`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-core/src/transaction.rs:2742`），查到的是新槽，释放新槽 | 释放一律经映射、不经提示（D19 已定项 5）；搬迁走映射、不改引用者（「搬迁走映射不改引用者」（`.claude/kb/decisions/26-后台整理与放置回收.md:93`）） |
| 两者各按旧视图算好计划再施加（并行提交才会出现，今天不会） | 整理批的源落点已被删除批释放 | 今天的释放入口会报错而不是静默减两次：「某块盘的条目已是已释放 ⇒ `ReleaseTargetAlreadyReleased`」（`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-core/src/transaction.rs:2756`）；整理批施加时也要先核源仍是「已分配」（D8 已定项 5 第 4 条，`.claude/kb/decisions/08-核心索引结构.md:125`） |

**整理搬走的单元，提示过期是常态**：D8 已定项 14 已经量过「提示过期、经中央映射回退时每跳多 2 次设备读」（`.claude/kb/decisions/08-核心索引结构.md:411`）。整理批因此只改映射 / 分配记录 / 记账 / 意图四样，不重写 extent 树与 inode 树，与删除批只重写被截掉那一片下段叶与它的祖先（D8 已定项 16 第 2 条）互不碰同一棵树；两者共改的只有分配记录树、记账树、中央映射树（每次发布都改）。判定：**一致**，没有找到「同窗口重落」的合法历史会判错。

**一处例外要记**：映射树自己的节点与树表、实例表「豁免映射」「映射树与树表豁免映射，各从根记录里指着它们的那条指针取」（`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-core/src/transaction.rs:2744`），位置由父条目 / 根记录里的指针说了算。R 里若躺着这几类单元，整理不能靠改映射搬走它们，只能让它们在某次普通发布里被重写（COW）。这与意图记录的格式无关，是整理引擎的事，列在这里免得落地时漏。

### 5.2 R 排他与删除批的落点

删除批自己也要新写元数据（被截掉那片下段叶与祖先、inode 记录、`deleted_inodes`、墓碑容器、分配记录与记账树节点），这些提交内生块从开放段 bump，段耗尽时回落到槽号最小的空槽，**回落同样排除 R**（「提交内生块的段耗尽时回落到该设备内槽号最小的空槽，同样排除 `R`」（`.claude/kb/decisions/03-空间分配.md:162`））。威胁：池近满时删除批（不判式子、只判落点）要的落点只剩 R 里的洞，被拒，而 D3 已定项 17 要求「释放空间这个操作本身不需要申请空间」（`.claude/kb/decisions/03-空间分配.md:304`）。

挡法：式子新增项「R 内被排除的空槽」（3.1 表第 1 行）。有了它，「可用 ≥ 需求」的合取和环窗口常驻扣项（`.claude/kb/decisions/28-挂载期承诺量.md:29`）一起保证：任何一次判过式子的发布完成之后，R 之外仍有不少于环窗口的空槽，删除批的落点那一判过得去。**这一项依赖 R 取整段对齐**（O(1) 得出 R 里空槽数，2.2 节末表第三行），所以它反过来支持字段 2 的形态。不需要意图记录再多一个字段。判定：**规则没说**（D3 已定项 8 第 2 条只写了回落排除 R，没写 R 的洞对式子的影响）；与 C318（影子账隔离的单元没进准入不等式）同形，那一条已经走过一遍。

### 5.3 管理员回退与 logged_ops

推荐：**logged_ops 随现行那一版**，D23 已定项 14 的那一句改成「……与 `deleted_inodes` 的根指针取 R_old 那一版的；墓碑树、谱系树、logged_ops 随现行那一版」。代码侧不用动：回退新根的树表只对 extent、inode 两种取目标那一版（「root: if entry.kind == TREE_KIND_EXTENT || entry.kind == TREE_KIND_INODE {」（`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-core/src/mount.rs:4972`）），其余种类自动随现行。

理由（三条独立）：
1. **整理意图丢了只给读者保守答案**（2.5 d，D21 已定项 10 的判据）⇒ 取哪一版没有正确性代价，选实现最省、状态迁移最少的：随现行，回退发布前后分配器里的排他集合不变。
2. **回退发布自己要分配**（谱系容器、固定点节点），这些分配必须仍排除 R；若回退把 logged_ops 换成 R_old 的，同一次发布里分配器用现行的 R、盘上写的是 R_old 的，两者要等发布后重装才一致。
3. 与已定的墓碑树、谱系树同一条纪律（「墓碑树、谱系树随现行那一版」（`.claude/kb/decisions/23-journal的角色与格式.md:376`））；`deleted_inodes` 取 R_old 是因为它和 inode 树捆在一起（I-9.5），整理意图没有这层捆绑。

**回退跨过一条整理意图之后 c0 / segs0 还成不成立**：
- c0 是 txg，新根的 txg 照常加一（「checkpoint_txg 照常加一」（`.claude/kb/decisions/23-journal的角色与格式.md:376`）），不会倒退；回退之后新分配的落点分配代仍 > c0。
- 复活的单元分配代取回 R_old 那一版账里的值（「分配代写回 R_old 那一版账里的分配代」（`.claude/kb/decisions/23-journal的角色与格式.md:382`）），可能 ≤ c0 且落在 R 里，目标态重新变假；续做按状态谓词重扫，有限步收敛（回退是有限次的管理员动作）。D26 已定项 2 的「单调」只对**新分配**成立（「前台在 `R` 内新分配的落点，其分配代必然 > `c0`」（`.claude/kb/decisions/26-后台整理与放置回收.md:50`）），对复活不成立；那一句的用法（「前台写不再阻止意图完成」（`.claude/kb/decisions/26-后台整理与放置回收.md:50`））要加一个前提：意图期内没有回退。**规则没说**，列进「留到落地」。
- segs0 是签发时的全空段数，回退后现行全空段数可以上下任意变（回退发布自己的分配可能开新段、复活单元占着的段回不了空），谓词只会多签或少签一条，不会错删意图（2.5 f 的读法下它只管签不签下一条）。

### 5.4 搬迁不改 birth 与墓碑、谱系

墓碑记录是 (对象 ID, 对象出生代, 区间起, 区间止, 死亡 txg)（`.claude/kb/layout/03-third-txn.md` 第二节），谱系记录是 (T_rb, T_old)（同文件第三节），都是逻辑身份，不含落点；D26 已定项 5 的第 1 条「搬迁不改 `birth`」（`.claude/kb/decisions/26-后台整理与放置回收.md:102`）保证 birth 不变。整理批不写墓碑树与谱系树（它们是 unlink 与回退的写物）。判定：**一致**。唯一的接口是：墓碑容器与谱系容器本身是进中央映射的单元（D18 已定项 24 第 1 条：「容器与根进中央映射，条目里的指针是位置提示」（`.claude/kb/decisions/18-块里携带什么信息.md:565`）），整理可以像搬别的单元一样搬它们，不涉及记录内容。

### 5.5 I4 判定与推翻条件

判定：搬迁 vs 删除 **一致**；R 排他 vs 删除批落点 **规则没说**（有挡法）；回退 **规则没说**（有推荐，需 D23 已定项 14 加一行）。

什么现象会推翻：
- 若并行提交进里程碑，5.1 第三行（旧视图计划）从「今天不会」变成常态，整理批的「先核源仍已分配」要变成硬检查并进崩溃流；
- 若有人给整理意图加了权威事实（例如 R 排他变成正确性条款）⇒ 5.3 的第 1 条理由失效，取哪一版就要重判；
- 5.2 的挡法若发现式子加了那一项仍有删除批被落点拒的合法历史，则要加 N₀ 一类的形状底额（D3 已定项 7 合取 7 已经在等它）。

## 六、I5　FTL / ZNS 与 GPU 只是优化

### 6.1 条款

| 条款 | 原文 |
|---|---|
| D26 已定项 7 | 「回收走哪条线按系统配置里的 feature bit 处理，每块盘自带」（`.claude/kb/decisions/26-后台整理与放置回收.md:144`）；「GC 的设计里要预留这条分支，这个功能以后一定会加；近期不论证、不实现、不验证」（`.claude/kb/decisions/26-后台整理与放置回收.md:144`） |
| D26 未定项 10 | 「两条线写到盘上的字节若相同、只差主机搬不搬数据与发不发 discard，不认识这一位的实现本来能正确读写这块盘，incompat 挡住它换不来安全性，该用 compat」（`.claude/kb/decisions/26-后台整理与放置回收.md:233`） |
| D26 未定项 10 | 「feature bit 在 mkfs 时写死，满足未定项 8 引的」（`.claude/kb/decisions/26-后台整理与放置回收.md:233`）——后接「分支变量不许在操作中途改变」 |
| D26 已定项 11 | 「把碎片的答案押在设备接口上不可靠」（`.claude/kb/decisions/26-后台整理与放置回收.md:161`）；FDP / ZNS 降为「有就用的加速」 |
| D24 已定项 2 | 「GPU 是可选加速器，一个比特都不许进格式」（`.claude/kb/decisions/24-后台重活能不能卸给GPU.md:32`）；「卸出去的结果必须能被 CPU 独立复核」（`.claude/kb/decisions/24-后台重活能不能卸给GPU.md:34`） |
| D24 范围 | D24 管的三个候选是批量压缩、全盘 scrub、大规模纠删码重建；「后台整理与回收的队列与预算在 D26（后台整理与放置回收）」（`.claude/kb/decisions/24-后台重活能不能卸给GPU.md:3`），整理不在 D24 的候选里 |
| 格式分支判据 | 「判据里不许出现「这样更快」」（`.claude/rules/fs-design.md:208`）；分支值不值得看它能不能把「用错了」变成「挂不上」（`.claude/rules/fs-design.md:199`） |

### 6.2 逐条优化对意图记录的要求

| 优化 | 意图记录要为它留种类 / 字段吗 | 理由 | 买到失败模式改善吗 |
|---|---|---|---|
| **设备线（盘的 FTL 自己 GC）** | 不留 | ① 线由 mkfs 写死的 feature bit 选，记录里再加一个「线」字段就是第二个、可能与前者不一致的开关，违反五条硬要求第 5 条（分支变量不许在操作中途改变，`.claude/rules/fs-design.md:153`）。② 交给盘的线上本工程「只打标记、发指令」：标记就是已回收落点的 discard，来自分配记录 / 空闲图，丢了只是盘晚知道，重发幂等，不是「无界且必须做完」的操作，用不着 D8 已定项 5 的意图（推的）。③ 腿一在该线上哪几件活停下还没定（未定项 9）；整理若仍要跑（为 N₀ 那条链），记录与主机线逐字节相同；若停，记录根本不产生。两种都不需要线字段 | 否：读者遇到「线」与 feature bit 不一致时没有比「feature bit 挂不上」更好的失败模式 |
| **ZNS** | 不留；zoned 不是意图记录的优化，是另一条布局线 | D13 已定项 10：zoned 是「结构性」差异，要改共享枚举「任何布局要引入新的结构性步骤必须改这个共享定义」（`.claude/kb/decisions/13-验证路线.md:212`），自带 incompat 位与新的提交步骤，不在这张记录的射程里。若 ZNS 仅作放置加速，是运行时分配器提示，不落盘 | 否 |
| **FDP** | 不留 | 放置提示，运行时 | 否 |
| **GPU** | 不留 | D24 已定项 2 第 1 条：一个比特都不许进格式。GPU 若用来选段，结果以 R 的值进记录，记录长得和 CPU 选的一样；第 3 条的 CPU 复核恰好是意图本来就有的状态谓词（对 R 的分配记录现算），不要新字段 | 否 |

### 6.3 前向兼容里唯一值得记的一格：R 的长度

R 取「对齐的整段」时长度是格式常量 64 槽（`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-format/src/lib.rs:271`）。ZNS 作为布局线时一个 zone 远大于 64 槽，R 的长度由那条布局线定，仍是 mkfs 写死的，**不需要**记录自带长度；显式写 2 字节长度是另一种选择，好处是记录自描述（`.claude/rules/fs-design.md` 的「不为省空间牺牲自包含」一节），代价是多一个会和布局线不一致的量。没有写者之前两种都零字节，所以**留到落地**；落地时与 C364 的 R 形态一起定。

### 6.4 I5 判定与推翻条件

判定：**一致**（四样优化都不逼出格式分支，与「CPU 整理是底线、另外三样是优化」相容）。

什么现象会推翻：
- 用户把设备线定成 incompat 且两线写到盘上的字节**不同**（未定项 10 的「动」）⇒ 记录是否随线变要重判；
- ZNS 路线定成「同一条布局线上的加速」且要求 R = zone ⇒ 6.3 的长度要进记录；
- 出现要求 GPU 结果以别的形式（不是 R 的值）进记录的方案 ⇒ 与 D24 已定项 2 第 1 条冲突，先过那一条。

## 七、I6　现在定什么、留什么

### 7.1 三·2（unlink 实现）之前必须定死的

标准：晚定会让**已写出的格式字节**或**概念模型的一句定义**返工。

| # | 要定的 | 理由 | 晚定的代价 |
|---|---|---|---|
| 1 | 确认 K-U1 的「不立 logged_ops」只管 unlink 的意图载体（则 D / B 可行），还是管到树表登记（则退 A） | 3.2 节：字面有张力；L4 的定义是「不另立意图记录」，不是「不登记树」 | 不确认就无法决定第 2 条 |
| 2 | logged_ops 要不要现在与 `deleted_inodes`、墓碑树、谱系树同批 day-1 零根注册（候选 D / B 的注册半，推荐是） | 这是**唯一**碰「新池新建文件」字节的一项：树表 10 → 11 条、水位 22 → 23；与删除那三棵同一次基线重做（E142 干跑、`layout/01` 几行、harness 与 checker 里钉树表条数的断言），0 个单元。晚定 = 第二次重做（3.5 节的量） | 落地日再做一次同样的重做；若落地晚于冻结提交，还要为这棵树选一档 feature bit（D15 已定项 9 对「新增一棵 keyspace」写 compat，对「新增记录类型」写 compat_ro，两行读法不同；`compat_ro` 在代码里零命中） |

其余都不碰已写出的字节，也不改概念模型的现有句子（只新增），留到落地。

### 7.2 三·2 实现时顺手做、不算定案的三件事

| # | 做法 | 为什么 |
|---|---|---|
| a | 回退新根挑树：把 `if entry.kind == TREE_KIND_EXTENT \|\| entry.kind == TREE_KIND_INODE`（`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-core/src/mount.rs:4972`）改成按一张「取 R_old 的种类表」查（extent、inode、`deleted_inodes`），别的种类默认随现行 | 新树默认随现行，不用每加一棵树就改一次回退 |
| b | 挂载里的续做入口只续 `deleted_inodes`，不写成「走完所有意图」的泛化循环 | 4.2 节；免得 unlink 的交回前必须空被带到整理 |
| c | 第三成员（「有树、文件集合为空」）的判定句里点名「不看 logged_ops」，照它已经「不看墓碑树与谱系树」的写法 | 整理意图可以在没有文件的池上活着；判定句不点名，将来有人会顺手加上 |

### 7.3 可以留到整理落地时定的

「最晚」一列：这一项最晚要在哪个时点之前定。

| # | 项 | 为什么可以留 | 最晚 |
|---|---|---|---|
| 1 | 字段表（R 的形态、键 / 值宽、预留） | 没有写者就没有已发布的字节；整理意图是派生态（2.5 d），节点格式不进永久契约；且 C364 里还有用户未定的几格（R 的形态） | 整理落地；若整理意图被判成权威态，则在 Layer 3 / 4 冻结提交之前 |
| 2 | c0 的定义（签发 txg − 1，还是排他从签发发布的分配起生效） | 一句定义、零字节（2.5 c） | 整理落地 |
| 3 | segs0 的读法与产出谓词的读数口径 | 取决于 C372 × C373 与 2.5 b 的口径选择 | 整理落地 |
| 4 | 放弃路径（D8 已定项 5 一句例外）、整理能否请求抬 F | 概念句，三个候选都同样要写（2.5 g、4.4） | 整理落地 |
| 5 | 活意图数上界（推荐全池 1 条） | 写者策略，不是格式位；放宽到每盘 1 条记录字节不变 | 整理落地 |
| 6 | 准入式子第十项、D26 已定项 3 底额占哪项 | 式子是准入判定的形式，不改写出的字节 | 整理落地 |
| 7 | C1 的范围按意图种类写；整理续做的强制进入开关 | 4.3 节 | 三·3 实现 `deleted_inodes` 的 C1 时顺带写成按种类，整理落地时补整理那一支 |
| 8 | D23 已定项 14 加一行「logged_ops 随现行」 | 5.3 节；代码默认就随现行 | 整理落地 |
| 9 | R 的长度隐含还是显式 | 6.3 节 | 整理落地 |
| 10 | 进度提示的写入频度 | 运行时旋钮（2.4 节：每多写一次该次发布状态数 ×4） | 整理落地 |
| 11 | 整理意图的态别写进 D21 | 2.5 d | 与第 1 项同 |

### 7.4 I6 判定与推翻条件

判定：**规则没说**（两张清单都是我从条款推的）。

推翻「只有两条必须现在定」的现象：若有人在 unlink 的实现里让 `deleted_inodes` 的条目带了种类码或宽到会和整理的条目宽共用一个节点（那就回到候选 C），则第 2 条之外要加「条目宽」；或者 3.5 节原型量到注册一棵零根树远大于预期（则第 2 条改成「可以留」）。

## 八、我去看了什么来反推（假设结论错，本该看到什么）

| 结论 | 若错，本该看到 | 我看了 | 看到 |
|---|---|---|---|
| 码 2 节点只有一个条目宽（2.5 e） | 节点头里有按种类的宽度表 | `crates/singlefs-core/src/unit.rs:166`、`:410-419` 解节点头 | 只有 `entry_width: u16` 一个，读时核「条目数 × 条目宽 = 声明长度」，没有第二个宽 |
| 全空段数只在回收时涨（2.5 b） | `mark_released` 里减每段已分配计数 | `crates/singlefs-core/src/allocator.rs:549-557` | 只有 `self.deferred_slots += span;`，每段计数的减在 `mark_reclaimed`（`:671`） |
| 回退默认随现行（5.3） | 回退新根的树表对未登记种类有别的分支 | `crates/singlefs-core/src/mount.rs:4966-4977` | 只有 extent、inode 两个种类取目标那一版，其余 `entry.root` 照现行 |
| R 要整段对齐才能 O(1)（2.2） | 分配器有按槽区间的计数 | `allocator.rs` 里 `used_per_segment`、`isolated_per_segment`、`held_per_segment` 三个每段计数 | 只有每段计数，没有区间计数 |
| 活意图要在重建那一步读出来（4.1） | 分配器重建不依赖 `previous` | `crates/singlefs-core/src/mount.rs:4487-4526` | `rebuilt_allocator(... &previous ...)` 先于 `establish_instance(...)` |
| 进度提示每写一次崩溃状态 ×4（2.4） | 状态数闭式与单元数无关 | `crash_enumeration_fixed_script_stream.rs:102-103` 与我算的 `n=22` 比 `n=20` | 比值 3.99997，闭式 2^n + 11 |
| 注册一棵零根树只碰五个文件（3.5） | 编译或点名用例红在别处 | 原型 | 见 3.5 |


## 九、复核命令与原样输出

以下命令都在 2026-10-01 真跑，输出不截断；最后一条是我引的 kb 与规则文件交回时的 sha256，行号随同仓别人的改动会移，对不上时以 `cite-check` 为准。

```text
$ cd /tmp/claude-1000/m3-delete-r2/tree; sha256sum -c /home/fy5090/code/singlefs/research/prompts/m3-intent-r1-snapshot/crates-sha256.txt | grep -vc ': OK$'
0

$ cd /tmp/claude-1000/m3-delete-r2/tree; sha256sum -c /home/fy5090/code/singlefs/research/prompts/m3-intent-r1-snapshot/crates-sha256.txt | grep -c ': OK$'
210

$ cd /tmp/claude-1000/m3-delete-r2/tree; grep -rn 'logged_ops\|LoggedOp\|deleted_inodes' crates --include=*.rs
crates/singlefs-checker/src/walk.rs:6133:    // （没有删除，C118（`deleted_inodes` 树的形态无落点） 未定形态）。
crates/singlefs-core/src/inode_tree.rs:16://! **合并不在这个模块里**：这一版没有删除（`deleted_inodes` 的形态无落点，C118（`deleted_inodes` 树的形态无落点）），

$ cd /tmp/claude-1000/m3-delete-r2/tree; grep -rn 'compat_ro' crates --include=*.rs | wc -l
0

$ cd /home/fy5090/code/singlefs/.claude/kb; for f in decisions/08-核心索引结构.md layout/03-third-txn.md invariants.md decisions/23-journal的角色与格式.md decisions/18-块里携带什么信息.md layout/01-first-txn.md; do printf '%s %s\n' "$f" "$(grep -c 'deleted_inodes' $f)"; done
decisions/08-核心索引结构.md 13
layout/03-third-txn.md 4
invariants.md 2
decisions/23-journal的角色与格式.md 2
decisions/18-块里携带什么信息.md 1
layout/01-first-txn.md 1

$ cd /tmp/claude-1000/m3-delete-r2/tree; grep -rn 'TREE_KIND_DEADLIST\|TREE_IDENTIFIER_DEADLIST' crates --include=*.rs | wc -l
19

$ cd /tmp/claude-1000/m3-delete-r2/tree; grep -rn 'TransactionUnit::AccountingTree\b' crates/singlefs-core/src --include=*.rs | awk -F: '{print $1}' | sort | uniq -c
      3 crates/singlefs-core/src/admission.rs
      1 crates/singlefs-core/src/mount.rs
     17 crates/singlefs-core/src/transaction.rs
      2 crates/singlefs-core/src/write_accounting.rs

$ cd /tmp/claude-1000/m3-delete-r2/tree; grep -rn 'TREE_KIND_ACCOUNTING\|TreeKindForReferenceScan::Accounting' crates/singlefs-checker/src/walk.rs | wc -l
8

$ python3 -c "print('ratio n=22 over n=20:', (2**22+11)/(2**20+11)); print('n=20 states=2^20+11=%d' % (2**20+11)); print('n=24 states=2^24+11=%d' % (2**24+11))"
ratio n=22 over n=20: 3.9999685290777016
n=20 states=2^20+11=1048587
n=24 states=2^24+11=16777227

$ cd /home/fy5090/code/singlefs/research/prompts/m3-intent-r1-sonnet-model; grep -c '^[-+][^-+]' proto-day1-logged-ops-tree.patch
45

$ cd /home/fy5090/code/singlefs; sha256sum .claude/kb/checks-owed.md .claude/kb/invariants.md .claude/kb/decisions/08-核心索引结构.md .claude/kb/decisions/26-后台整理与放置回收.md .claude/kb/decisions/13-验证路线.md .claude/kb/decisions/03-空间分配.md .claude/kb/decisions/23-journal的角色与格式.md .claude/kb/decisions/28-挂载期承诺量.md .claude/kb/decisions/21-权威态与派生态的分界.md .claude/kb/decisions/18-块里携带什么信息.md .claude/kb/decisions/24-后台重活能不能卸给GPU.md .claude/kb/layout/03-third-txn.md .claude/rules/fs-design.md
eeef4b9df74952bf5bbaea430b5546acbf07f949b5fe1bba7a615f0bf972df08  .claude/kb/checks-owed.md
677f9fb875c7e461d6a5aeeb18aff701e84404b458458cf254784108bd8469d2  .claude/kb/invariants.md
dc5e2ac6ace5411b0c39272a7cd6f3ecf3b4ad090cf5a0440dea621b45c5fb58  .claude/kb/decisions/08-核心索引结构.md
a23b036518c152362ccff310de4e92e156bb0fe4e8ff2c3597553f25464a8bc5  .claude/kb/decisions/26-后台整理与放置回收.md
9ead76b1848dcff49f8ad8165272bc9ec035bb2fad01c2267685d9b52daec168  .claude/kb/decisions/13-验证路线.md
5cb9305fbf61c86026c22251d5ec148a0b0d94e9d007b5b4c3ce798f00bfa095  .claude/kb/decisions/03-空间分配.md
9c2fc930e9f2238bad034ad3ea402fa2a837fdfd03c8522e9825d56c64c76036  .claude/kb/decisions/23-journal的角色与格式.md
acdc3ea23590268bcbcb80010789374776c696eb4686d32ed2ed601e285c3fc6  .claude/kb/decisions/28-挂载期承诺量.md
2c07c54b234dd9d70bacbcff08f334a9a3e0839addccbbfe20a55f2cb9f986ae  .claude/kb/decisions/21-权威态与派生态的分界.md
e7fbe203e7745d9b2b11478f675b70e1f546bc511b8174054aa1c3d280787ef3  .claude/kb/decisions/18-块里携带什么信息.md
e71e6f87bbfb7784b9dec9a32dc84fd99038a639bd2faec7de152950124bbee7  .claude/kb/decisions/24-后台重活能不能卸给GPU.md
1cec823d3bd43b2b2338ac55e7fbdf4ee47ef6842a77a5e1347ea822f5078007  .claude/kb/layout/03-third-txn.md
768c24c589d0bb8496c9de6343ad5e525037a44f3934fe538ce0abf2867d7ef8  .claude/rules/fs-design.md

```
## 十、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| I1 | 规则没说；一处冲突（正文第一节 vs D26 已定项 6） | 字段表整张缺（C364）；推荐键（种类 2 + R 10）+ 值（c0 8 + segs0 8 + 进度提示 6 + 预留）；触发三量不进记录；全池至多 1 条；产出谓词受 defer 滞后影响，读数口径要与 segs0 读法一起定 |
| I2 候选 A | 收 | 落地日多一次「树表条数 / 水位」重做，其余同 D |
| I2 候选 B | 收，不推荐 | 字段表现在定死买不到字节，却要现在替用户定 C364 还开着的几格 |
| I2 候选 C 随现行 | 不收 | 「删 → 回退到删之前」上 I-9.5 红或续做做错 |
| I2 候选 C 取 R_old / 按种类合并 | 收，不出局，交用户 | 少接一棵有节点的树；换来整理时间线绑 inode 树，或回退多一段合并并多一个单镜像判不了的不变量；推翻 K-U1 的依据我没找到 |
| I2 候选 D（我加） | 收，推荐 | 只做注册；与 K-U1 字面「不立」有张力，用户确认后可行，否则退 A |
| I3 | 规则没说；一处冲突（C1 字面 vs D26 已定项 12） | 按树分入口、共用发布路径；挂载只续 `deleted_inodes`、整理意图只读出并装进分配器排他；整理走常规准入 + 式子新项 |
| I4 搬迁 vs 删除 | 一致 | 串行提交 + 释放经映射，没有合法历史判错 |
| I4 R 排他 vs 删除批落点 | 规则没说 | 式子新项挡，依赖 R 整段对齐 |
| I4 回退 | 规则没说 | logged_ops 随现行；D23 已定项 14 加一行；复活让 D26 已定项 2 的「单调」要加「意图期内没有回退」前提 |
| I5 | 一致 | 四样优化都不逼出格式分支；R 的长度隐含还是显式留到落地 |
| I6 | 规则没说 | 现在必须定的两条；实现时顺手的三件；可留的十一项 |

## 十一、没做什么

- 没跑 harness 档全量：全量命令从起到停 1034 秒（含编译，日志末行 `rc=143 seconds=1034`），当时机器负载平均 `load average: 80.46, 88.81, 89.48`（`uptime` 现看），一个用例二进制跑了 9 分钟还没完；我先停 cargo、再停它的子进程、再列一遍确认一个不剩，改跑点名的用例，点名的清单与结果见 3.5 节。**没跑过的用例里有没有因树表多一条而红的，我不知道。**
- 没碰、没编译 `singlefs-checker-tier`（按约束）；其中 `src/bin/e158_root_choice_repair.rs` 里引 `TREE_KIND_DEADLIST` 的两处（`:65`、`:2573`）按同形要加臂，没改、没编。
- 没验证整理在实现上的任何行为：实现里没有整理（`grep` 零命中）；2.5 b 的滞后只是读 `allocator.rs` 推的，没跑出序列。
- 没量并行意图的吞吐（2.3），没量进度写入的实际崩溃枚举耗时（只用了 `2^n + 11` 闭式与我算的比值）。
- 没把原型上的数当入库装置的数：原型在副本上，要引进 kb 须在入库装置上重做（`.claude/rules/three-way-inference.md`「判决由主 agent 做，不由投票做」一节）。
- 没读攻方腿这一轮的任何产出（禁读）；没整读背景材料的 1.6 MB 附录，引文都回 kb 与冻结副本原文取。
- 没替主 agent 采纳或出判决；「推荐」是我的推论，交主 agent 核。
- 没改仓内 `crates/`、`.claude/`、kb；只写了报告与模型目录。
- 草稿目录 `/tmp/claude-1000/m3-intent-r1-sonnet/` 里留着：原型与基线的原始日志（`proto-red.log`、`baseline.log`、`proto-harness.log` 等）、补丁、生成第九节的脚本与草稿，都是几百 KB 的纯文本；编译目录与副本已删（`target` 8.3G、`target-base` 458M、`tree` 11M、`orig` 11M）。原始日志没入库：原型数不进 kb、要引须在入库装置上重做，摘录在 `research/prompts/m3-intent-r1-sonnet-model/results.log`。
