# E207 跑前登记：B8严格判据下的步数

写于 2026-11-09，装置写之前。

<!-- doc-lint:not-numbers B1 B4 B8 F1 F2 F3 F4 F5 F6 F7 F8 F9 P1 P2a W1 W2 W3 W4 W5 W6 W7 U Q1 Q2 Q3 Q4 Q5 Q6 Q7 Q8 Q9 Q10 Q11 M1 M2 M3 M4 M5 M6 M7 M8 M9 M10 M11 M12 M13 M14 C1 C2 C3 C4 K1 K2 K100 K1000 D100 D10 T1e5 FTB H1 H2 H3 S T P0 S1 S2 D8 D28 R_a K1000L Q10 M15 M16 -->

岔路单：`research/prompts/e203-b8-criterion-r1-forks.md`（sha256 `f84adae71af4d323` 起，全值见第十三节）；判决 `research/prompts/e203-b8-criterion-r1-main-verification.md` 第三节。本登记里不带别的说明的 `文件:行` 都指主工作区此刻的 `crates/`（`transaction.rs` sha256 `19c50519ed1ede3a96298feb646ef7aac38a810596b9f80797f0cb28ccd016a1`，`allocation_record_tree.rs` sha256 `991623b6a9b5028eb692b8e88a3e6acaab6c1471c5302b23dffeb6f42ea958ce`，`allocator.rs` sha256 `36fe6af3e544c9bd56126e525393c749c9dd0cdc81fb01297e8623895e3e7499`）；行号变了照函数名找，内容变了走停机条款第 3 条。

## 一、问题

英文名：b8_strict_criterion

主 agent 给的问题（派发提示「关的是哪几条」逐字，三条）：

> 1. B8 的新判据没有登记。验收：取新实验号写跑前登记，把判决第三节六样逐条写死：记一次输（整行抄 `research/results/e206-fixed-point-cost-report-2026-11-09.out:706`）；改动大小不变的发布读原步数；改动随轴涨的那一种读步数 ÷ 外生的用户单元数（`user_units`，不用实现算出来的 `Rs`、T，写明这一格比原判据松）；x 取量到的总量（记录数或活单元数，一个主读、另一个报）；阳性对照必须红、不红整段作废（`H_scan_every_publish`、B8 最大一行换回线性）；样本外新点。
> 2. 能分开「随改动」与「随总量」的那一格没有样本。验收：登记新点（攻方提的 `files_100_units_1000`、`files_10_units_10000`，`scripts/experiment-apparatus/e206_fixed_point_cost_histories.rs:111` 的 `Shape::SmallFilesOfUnits`；你核可达再定），连同 E206 已登记的 `files_then_big_1000`、`files_then_big_10000`，判读线跑前写死、两种结局各写怎么改结论。
> 3. 量窗第一次覆盖写的尖峰成因没查（`files_1000_units_100` W1 第 1 次 `final_roles=452`，`research/results/e206-fixed-point-cost-files_1000_units_100-p2a-2026-11-09.out:520`；W1 第 2 次 `final_roles=20`，同文件第 3485 行）。验收：登记里有一段量它：成因定到发布里的哪一步、重现条件。

被测条款（派发提示逐字）：`.claude/kb/decisions/08-核心索引结构.md`「已定项 16」射程那一句（第 516 行）；`.claude/rules/fs-design.md` 第 23 行运行时决策路径那一格。抄录在第二节。

**记一次输**（判决第二节 F2 字面，整行抄自 `research/results/e206-fixed-point-cost-report-2026-11-09.out:706`，登记写时 `sed -n 706p` 现读）：

```text
name=q8_peak arm=p2a axis=data_units low_end=3763 high_end=3231038 alpha=0.978 band=grows local_slopes=0.660,1.019,1.255
```

F2 是 E203 登记第十节的失败条款（`research/prompts/e203-preregistration.md:614`）：B8 行「每次发布准入之前扫类步数只随这一次碰到的单元、容器、记录数（加树高）涨」，任一根轴、任一种挂载状态、任一种被量的发布 α ≥ 0.5 即翻面。按最弱读法这一行把 B8 在原判据下记为**输**：B8 行翻面。本登记不撤回这一笔，新判据读数另列，两套都报。

### 读法（写死）

1. **「一次操作」**：量窗里 `Measured::measure` 的一次调用（E206 histories 的 `op` + `op_index`）；一次操作可含几次 `settle_call`，B8 步数取这些 `settle_call` 的 B8 合计（每份照 E206 模型 `by_delta_of_settle` 的口径，登记写时的 `scripts/experiment-apparatus/e206_fixed_point_cost.rs:1458`）。两种挂载状态 `same_mount`、`remounted` 一律分开判，不合并。
2. **`user_units`（外生）**：历史驱动按构造声明的、这一次用户改动**点名的数据单元数**，不读实现的任何输出：覆盖写 = 被换下的旧内容单元数（驱动自己记每个 inode 当前单元数 `units_of[inode]`，建时按构造，覆盖写后记新内容的 1 个单元）与新内容单元数的较大者；建 inode = 1；unlink = 被删文件的单元数，空 inode 记 1；截断 = |旧 − 新|，下限 1。**不用** `Rs`（各轮释放数之和）、T（第一轮释放加重写角色）、`final_roles`、`rounds`：它们由被判的固定点算出来（判决第二节「分母内生」）。
3. **`x`（量到的总量）**：主读 = 这次操作之前的 `allocation_records`（`name=state` 行那一项，操作之前取）；报 = `mapped_units`（活单元数，同一行）。另报 E206 原判据的旋钮 x，只作与旧判据对照，不进本登记的任何判定。
4. **增长阶与档**：两点 `α = ln(Q_hi ÷ Q_lo) ÷ ln(x_hi ÷ x_lo)`，Q 与 x 取这两点各自量到的值；档同 E203 登记第六节 Q6 的带：`α ≤ 0.25` 不随、`α ≥ 0.5` 随、其间介于（不当通过）。判决没有改带，本登记不动。三点以上另报相邻局部斜率。
5. **两种读数规则，按「比较集合」里 `user_units` 在两端是否相同逐格选，不由人挑**：两端 `user_units` 相同（比值 = 1）⇒ 读 **原步数**（÷ 常数不改 α，两种写法给同一个数）；两端不同 ⇒ 读 **步数 ÷ `user_units`**，并在输出行标 `looser=yes`；`looser=yes` 的格同时报原步数的 α。**这一条比原判据松**：同一次发布上 α(步 ÷ 分母) = α(步) − α(分母)，分母沿轴不降时新 α 不高于原步数的 α，门槛不变，没有一格会比原判据更严（判决第二节「题面写法」一格）。本登记不把它当收严：收严来自读法 3（x 取量到的总量）、读法 7（逐格先报第一次与后来的）与第五节的阳性对照，不来自除以分母。
6. **比较集合**（新点上，每个集合两点或三点，`x` 全取各点自己量到的）：K100 = {`files_100_units_100`, `files_1000_units_100`}（改动相同 100 个单元，总量差约十倍）；K1000 = {`files_10_units_1000`, `files_100_units_1000`}（改动 1000 个单元，总量差约十倍）；T1e5 = {`files_100_units_1000`, `files_10_units_10000`}（总量相近、改动差十倍，只报 `α_u = ln(Q_hi ÷ Q_lo) ÷ ln(u_hi ÷ u_lo)`，不按 x 判）；FTB = {`files_then_big_1000`, `files_then_big_10000`}（小文件数与大文件单元数一起涨，改动都是 1 个单元）；D100 = {`files_100_units_100`, `files_100_units_1000`}（F = 100 不变，K 与总量一起涨十倍：改动随轴涨，两端 `user_units` 不同 ⇒ `looser=yes`）；D10 = {`files_10_units_1000`, `files_10_units_10000`}（同上，F = 10）。K100、K1000、D100、D10 一起构成 2 × 2 的因子表（改动 × 总量），第六节 Q3 用它分开「随改动」与「随总量」。回扫集合 = E206 的轴（files、data_units、unlinks、unit_area 四根）上 40 份产物里的 p2a 臂，x 同样取记录数。**大点集合（只第四段）**：K1000L = {`files_10_units_1000_unit_area_2_pow_21`, `files_100_units_1000_unit_area_2_pow_21`, `files_1000_units_1000`}，三点 U 全是 2²¹，改动都是 1000 个单元（`uK` 档，K = 1000，三点 `user_units` 相同、`looser=no`），活单元数 10⁴、10⁵、10⁶（设计比 100.0；量到的 `allocation_records` 比与 `mapped_units` 比如实报，不够 100 倍的如实写，不改判据）。判读用两端（低端对大点）的 α，中点只给相邻两段的局部斜率；另报 {中点, 大点} 两点的 α（总量十倍，两点改动占总量都不超过 1%）。K1000L 的 `u1` 档同样读（三点都有 `user_units = 1` 的操作）。K1000L 与第一段 K1000（U = 2¹⁹，十倍跨度）是两个集合，不合并，U 与跨度同时不同，差在谁身上 Q10 不判，只报。
7. **「第一次」与「后来的」**：每个（比较集合、挂载状态、`user_units` 档、集合内的点）里按操作次序：`first` = 这一档的第一次操作，`later_peak` = 其余里步数最大的一次，`later_median` = 其余的中位数；三样各报一份，各判一档。`user_units` 档 = `u1`（`user_units = 1`）与 `uK`（`user_units` 等于该点声明的 K，K ≥ 100）；别的 `user_units` 值（如 D = 10 的大文件）报数不判。
8. **尖峰旗**：一次操作的 `final_nodes` ≥ max(28, 4 × 同点同状态量窗全部操作 `final_nodes` 的中位数)。每格另报「剔除旗上的操作」之后的档（`steady`），与不剔除的（`all`）并列；两样都报，不替它选。28 = 4 × 7，7 是已读到的历史每次发布 `final_nodes`（第四节第 8 条）。
9. **控制臂**（第五节）：每条在同一批读数上算出读数，与被测臂用同一把尺（读法 3–7）判一档。
10. **本实验没有 P1 臂**：主工作区 `crates/` 已带 P2a（`transaction.rs:13962` 的 `merge_into_the_rewrite_set_walking_the_release_chain`，两处调用 `:13918`、`:13945`），被测臂就是今天的实现；E206 的 `p1` 补丁在今天的树上锚点失效（第三节），P1 的逐轮读数 E206 页已作废，不用。

### 岔路单（`research/prompts/e203-b8-criterion-r1-forks.md` 第 7–11 行逐行照抄）

问题：用户 2026-11-09 定用严格形态（判决 `research/prompts/e203-b8-criterion-r1-main-verification.md` 第三节）：记一次输；改动大小不变的发布读原步数；改动随轴涨的那一种读步数 ÷ 外生的用户单元数；x 取量到的总量；阳性对照必须红；样本外跑新点。40 份 E206 产物只能回扫。

| 候选 | 翻面观测 | 够判条件 | 状态 |
|---|---|---|---|
| B8 站住：准入之前的步数只随用户这一次的改动涨 | 没见过的点上，同一种用户改动的步数按记录数的增长阶；改动随轴涨那一种的每用户单元步数按记录数的增长阶 | 含「改动至少 1000 个单元、改动远小于总量」的新点；每种发布两种挂载状态分开判；阳性对照（每次发布全表扫描、B8 最大一行换回线性）都判红 | 开着 |
| B8 翻面：步数还随总量涨，而且不只第一次发布 | 同上，加第二次及以后的发布的读数 | 同上 | 开着 |
| 尖峰另算：只有量窗第一次覆盖写随总量涨，来自固定点把攒下的角色一次带出 | 尖峰那次发布重写的角色由什么决定；挂载之后第几次发布、隔多少发布重现；能不能在别的操作后重现 | 把尖峰的成因定到发布里的具体一步，并写清它重现的条件 | 开着 |

第六节每个量写明它对应上表哪一行、取什么值让那一行翻面；对不上任何一行的量标「附带，够判后不跑」。下文把三行记作**行一**（站住）、**行二**（翻面）、**行三**（尖峰）。

未定项：无。原第 1 条用户已定（见下一条「已定项 1」；全文别处写「未定项第 1 条」处都指它）。

已定项 1（原未定项第 1 条，用户 2026-11-09 定「加大点，分段跑」，即原选项 (c)，不选 (a)、(b)）：加总量跨度 100 倍的大点，放进独立的最后一段（第四段）；不改 `C1` 的系数、不改判据。原问题的实情照留：`C1` 在第一至三段的 `uK` 档上预期不红（K = 1000 的 K1000 集合 α 约 0.32，推的，`$D/calc/arith.out`），按作废条款 1 那一档只报数，行一的「改动至少 1000 个单元、改动远小于总量」那一半在第一至三段不下结论；`u1` 档与 FTB 不受影响。大点补的就是这一半：K1000L 三点（第一节读法 6），`C1` 在它的 `uK` 档上的 α 推的结果（`$D2/calc/big.awk` → `big.out`，输入是猜的，不是量的）：每单元被测臂步数 b 取 150、178.4（第四节第 3 条攻方读数）时 0.52–0.61，b = 320 时 0.42–0.47；记录数按「约 2 倍活单元数加元数据」取低端 2.0–3.0 × 10⁴、高端 2.0–2.5 × 10⁶；b 的分界约 230（b ≤ 230 红、以上不红）；系数 4 的 `C2` 为 0.66–0.82。**贴线，不保证红**。红 ⇒ 第四段 `uK` 档行一、行二可判；不红 ⇒ 第四段 `uK` 档按作废条款 1 作废、只报数，`C2` 照报，此时回主 agent 定是否另立登记改强系数。`u1` 档上 `C1` 在 K1000L 的推的 α 约 0.96（`big.out`），能分开。第四段的作废、判读、阳性对照与第一至三段分开判，它的任何读数都不进第一至三段任何一格的判定。

## 二、被测条款与它引的定义

下面两段由 `scripts/kb/quote-kb.py` 整段抽出（命令在第十三节），回读逐字节一致。被测的是 D8（核心索引结构） 已定项 16 射程那一句（该段第 516 行，「记录总数」在那一行里）与 fs-design 第 23 行运行时决策路径那一格；整节抄，不转述。

**出处 `.claude/rules/fs-design.md:17-38`（整段抄，未转述）**

```markdown
## 记账是事务的副产品，不是事后的遍历

**线划在「谁消费这个数」，不划在「用什么机制算出来」。**

| 消费者 | 允不允许遍历 |
|---|---|
| **运行时决策路径**（分配、ENOSPC 准入、defer 窗口、生命周期判定） | **不许**，且代价不许随盘容量增长 |
| **checker / 审计** | **必须**遍历 |
| **后台、可续做、非决策路径**（销毁快照、整理、scrub） | **无戒律** |

**空间统计与配额落在第一格**：必须在事务提交时增量维护出来。

⚠️ **一个量落不落在第一格，按这一格自己的判据判：「被问到那一刻有没有答案」，不按名词清单判。**
「快照用量」按这条判的结果、将来做 per-snapshot 配额时怎么办，
见 `.claude/kb/decisions/05-快照-空间记账机制.md` D5（快照 / 空间记账机制） 已定项 4 统计量表第 8、9 行。

⚠️ **被禁止的不是「遍历」，是两件事**：
「在需要答案的那一刻还没有答案」，以及「审计与被审计用同一段代码」。
**运行时与 checker 必须用不同的算法。**

⚠️ **别拿第一格的禁令去否第三格的设计。**

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:503-535`（整段抄，未转述）**

```markdown
#### 已定项 16：第一版 unlink：只立 deleted_inodes、从尾部分批、挂载里续做到空

**定案**：

1. **意图**：意图那次发布把 inode N 的记录写成 nlink = 0（size 不变），`deleted_inodes` 加 key = N（8 字节、无 value、不带对象出生代），往墓碑树开放容器追加一条墓碑（D18（块里携带什么信息） 已定项 24）；不立 logged_ops（K-U1 L4）。
2. **分批**：对任一 inode N，从尾部按数据单元个数分批，每批释放 N 尾部 K 个单元、size 改成「剩下单元数 × 净荷容量」，剩下单元数取 N 在 extent 树里的单元数（没有的取 0，只有最后一批）；K 产品取 `EXTENT_TREE_LOWER_LEAF_DATA_UNITS`（144，自由选取），测试由只供测试的旋钮压小；删除批只重写 N 被截掉那一片下段叶与它的祖先，加上段里 N 那一串（K-R3-1，extent 写法的新分支，要过 `.claude/rules/fs-design.md` 五条硬要求）；最后一批释放 N 剩下全部单元、删 N 的 inode 记录、删 `deleted_inodes` 那条 key、把 N 的条目从上段去掉（不写标签 0），同一次发布；只有这一条路，不设一次做完的快路。意图那次、每一批与续做的每一批各占一个事务号，等于本实例的下一个（与普通发布同一套编号）。
3. **建树**：`deleted_inodes` 在第一个文件版本那次发布零根注册；它是派生态：队列 = inode 树里 nlink = 0 的号集合（`research/prompts/m3-delete-r2-main-verification.md` 第三节 V7）。`deleted_inodes` 根是单节点：码 2 头的层级字节写 0，条目就是队列里的号；池级 checker 的删除态扫描只按层级 0 读它（`crates/singlefs-checker/src/walk.rs` 的 `deleted_inodes_queue_without_judging`），层级不是 0 的根判不了。
4. **读者与写入口**：`InodeRecord` 的 nlink 由 `parse` 读、`to_bytes` 写结构里的值；现造记录的入口拒「nlink = 0」与「没有这条记录」；从意图提交起只读挂载、冷走读、模型读回答「没有文件」；写入口拒 nlink = 0 的 inode（`research/prompts/m3-delete-r2-main-verification.md` 第二节第 1 行、第三节 V6）。
5. **删空**：「有树、文件集合为空」是 `RebuiltVersion` / `PreviousVersion` / `PoolVersion` 的第三个成员，不借 `WithoutFile`；判定 = 树表有条目 ∧ inode 树零条目根（key 区间 `[0, 0]`，照已定项 6）∧ extent 树零条目根（key 区间按位置规定）∧ `deleted_inodes` 根指针置零；不看墓碑树、谱系树与退役树。删完之后只能做 `publish_new_inodes`（size 0）、写行与空发布（K-U5 X1）（`research/prompts/m3-delete-r3-main-verification.md` 第三节 G5）。删空的一版上覆盖写、顺序写、unlink 拒成已有的「没有文件版本可改」那一种拒绝（`UserChangeRefused::NoFileVersionToChange`）。删完第一个文件、别的文件还在的一版归 `WithFile`：inode 树还有任一记录或 extent 树还有条目的一版是带文件的一版，不管 inode 1 在不在；第三个成员只指文件集合真的空。
6. **续做**：`mount_writable` 在写行与暖机之后、交回之前把队列走到空；不进取号前预演；会话里的队列每次从现行那一版读、不缓存；崩溃注入加 `WritableMountPhase::Continuation`（`research/prompts/m3-delete-r2-main-verification.md` 第三节 V3）。会话里每一次用户改动（含 unlink、写、建 inode）先把现行那一版的 `deleted_inodes` 队列走空，再做它自己；挂载续做照旧；unlink 一次调用 = 走空 + 意图 + 全部批（与续做共用同一个走空）。会话走空时删除类发布（意图那次与分批的每一批）被落点那一判以空间拒，推一串抬 F、回收后再试；每一次被拒的删除类发布各算一次准入（一次准入最多 `PUBLISHES_PER_ADMISSION_AT_MOST` = 8 次发布，`crates/singlefs-core/src/mount.rs`）；推满仍放不下交回「抬 F 之后仍没空间」的拒绝（`UserChangeRefused::NoSpaceAfterRaisingTheFloor`），已发成的留在盘上。挂载续做走空时删除类发布被落点那一判以空间拒，同样推抬 F 再试，预算口径与会话走空相同；推满仍放不下时挂载照样交回可写、队列留着，之后会话里每次用户改动先走空、走不动报空间不够，正常卸载照常做成，靠的是 D16（发布语义） 已定项 1「正常卸载」那一行当场可用的那一截回收出来的槽（与 D16（发布语义） 已定项 1「准入」那一行同一定案）。任何一版里 `deleted_inodes` 至多一条 key；多于一个根装得下那一支拒成类型化错误、不是空间不够。
7. **幂等的判据五条**：① 计划相等；①b 同一份崩溃镜像各自重挂载续做同一批，两份盘逐字节相同（意图那次与每一批的改动计数是本次发布的 checkpoint_txg；意图那次的三个时间秒由调用方给、共一个值，各批照抄，续做的几批不另取时间）；② 删完再续做一个写都不发；③ 每个写点截断、重开续做到底，终态逻辑内容与不崩溃的终态相同、每个崩溃态 checker 全绿；④ 每个被判状态的记账标签 4、6 等于从这一版镜像现算的值。「逻辑内容」含 inode 每条（号、对象出生代、size、nlink）、extent 单元序号集与内容校验和、`deleted_inodes` key 集、墓碑记录多重集（含死亡 txg）、谱系记录多重集、标签 12 / 4 / 6、读者的答案；不含实例号、根与单元头里的 txg、改动计数、落点、单元头的诞生代号与写序、逐盘标签 1 / 2 / 5 / 10 / 11。另加 harness 层每种发布的重写角色集合钉绝对值（零轮）（`research/prompts/m3-delete-r2-main-verification.md` 第三节 V1；`research/prompts/m3-delete-r3-main-verification.md` 第三节 G2）。不同操作序列的终态不要求逻辑内容相同（例如不同删序的墓碑死亡 txg 不同），要求上段形状只由有条目 inode 的号集合定、每个崩溃态 checker 全绿。
8. **上段规范形**：extent 树上段形状只由有条目 inode 的号集合定，写与删共用一条规则：节点取各有条目 inode 的根路径（从它那片叶到根那一串）之并；根层级取罩得住最大号的最低一层；集合为空时取层级 0 第 0 格一个零条目根兼叶；非根节点不许零条目。一次发布里，进出集合或条目变了的号那一串上留在新形状里的节点与新出现的节点重写，别的照抄，上一版有而新形状没有的换下；长高那次旧根照抄成根之下的节点，降低那次上一版里降后层级的第 0 个节点照抄成根，旧根与脊线上方摘掉的节点换下（已定项 14 的 extent 树一条写同一个形状，`research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节「上段规范形」）。

**射程**：第一版按 inode 号删任一 inode、无目录（用户 2026-09-28 定的是单文件、按 inode 号、无目录、不带 truncate，任一号由多文件删除的判决放开）；截断与变长不走这一条，走已定项 5 的截断意图（判决 `research/prompts/m4-trunc-r1-main-verification.md`），照这一条的走空与幂等纪律；删完第一个文件之后（K-U5，记欠账）按 inode 号写数据只拒对象出生代晚于 inode 1 墓碑死亡 txg 的 inode，删之前就有的别的文件照常能写；取这个死亡 txg 只认这一版父链上的墓碑：「父链」照 I-9.18（删除队列里的号有谱系上的墓碑） 的定义，从这一版沿谱系记录往上走，只排掉被这条父链上的回退抛下的那几段 (T_old, T_rb) 开区间里的死亡 txg，不在父链上的谱系记录不排，再取最大；inode 1 的记录还在时不判；这一读法依赖墓碑第一版不回收（D18（块里携带什么信息） 已定项 24），墓碑回收落地时要换读法（C618（删完之后不能再建带数据的文件））；崩溃流接在 `crates/singlefs-verification/tests/crash_enumeration_multi_record_publish_stream.rs` 那条流后面 unlink 它的 3 单元文件（K-U6 F1），截断成 0 另立一条流。只改尾叶的 extent 分支、写入类发布按发布之后的标签判、重写角色集合钉绝对值被攻过零轮；第 6 条增补的会话里每次用户改动先走空、与任何一版至多一条 key，也被攻零轮。`.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」那张表里运行时决策路径「代价不许随盘容量增长」，在这一格读成只管准入之前（用户 2026-10-31 定，原话在变更史）：准入之前的步代价不随盘上容器总数、记录总数涨；准入之后装单元、拷字节不算运行时决策路径。

**依据**：

- 无实验：数都在三方腿的副本原型上量（三份删除判决），入库装置「删除借保留量的环窗口」第一段已在套补丁表的副本上跑完（2026-10-03），第二段要在真实现的删除批上重跑、还没跑。
- 三方判决 `research/prompts/m3-delete-r1-main-verification.md`、`research/prompts/m3-delete-r2-main-verification.md`、`research/prompts/m3-delete-r3-main-verification.md`。
- 用户定案 2026-10-01：K-U1、K-U5、K-U6、K-R3-1（原话在变更史）。
- 判决 `research/prompts/m3-blanks-r1-main-verification.md` 第二节队列上界与接口那一行：回退到意图那一版会把号带回队列，只有「会话里每次用户改动先走空」与「回退之后续做」两个臂在放开用户动作的历史上队列长度与有队列时的写次数两项都守住。
- 用户定案 2026-10-01：会话里每一次用户改动先走空（岔路单 `m3-blanks-r1-forks.md` 的「会话里的续做点」一行，原话在变更史）。
- 用户定案 2026-10-02：会话走空被空间拒推抬 F 再试；2026-10-03：每一次被拒的删除类发布各算一次准入、挂载续做被空间拒同样推抬 F 再试、推满仍放不下照 D16（发布语义） 已定项 1 做成可写、删除类发布各占一个事务号（原话在变更史）。
- 代码轮判决 `research/prompts/m3-unlink-code-r1-main-verification.md` 第二节 K2 那一行（续做、走空与推抬 F）与第四节（挂载续做推满仍放不下的结局交用户）；实现：会话走空推抬 F 是 `crates/singlefs-core/src/mounted_session.rs` 的 `publish_the_queue_until_it_is_empty_raising_the_floor_for_space`，事务号是 `crates/singlefs-core/src/transaction.rs` 的 `publish_deletion_step`。
- 主 agent 2026-10-02 定 Q-L 照收（调度记录 `records/2026-10-01-删除与并发事务.md`「十六　批 8」）：删空的一版上覆盖写、顺序写、unlink 拒成已有的那一种拒绝。
- 用户定案 2026-10-03：「卸载必须做成」，续做推满仍放不下之后正常卸载靠 D16（发布语义） 已定项 1「正常卸载」那一行放开的那一截回收出来的槽做成，单盘幸存那一问取「甲 GA（推荐）」（原话在变更史）；设计轮判决 `research/prompts/m3-unmount-after-short-continuation-r1-main-verification.md`，代码轮第三轮判决 `research/prompts/m3-unlink-code-r3-main-verification.md` 第二节 n1 各行。
- 用户定案 2026-10-03：删除步算改动（「算改动，改实现」），时间字段取「乙 意图带时间、三格同改」（原话在变更史）；代码轮第三轮判决 `research/prompts/m3-unlink-code-r3-main-verification.md` 第二节 n3 那一行。
- 判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节「删任一号」「上段规范形」「删第一个文件」与第五节用户定案：删任一 inode 的各步、上段规范形、K-U5 取「β 时序级」（用户定案原话在变更史）。
- 代码轮判决 `research/prompts/m3-close-w5-code-r1-main-verification.md` 第二节 W4 与第三节第 1 条：β 取墓碑死亡 txg 只认这一版父链上的墓碑（攻方量过：删 inode 1、建 inode 3、回退到删之前、再删 inode 1、回退到建 inode 3 那一版，写 inode 3 做成，池级 checker 零红）。
- 判决 `research/prompts/m4-close-code-r1-main-verification.md` 第二节「T7 容器小树代价」那一行（判决当时 `ancestors_of_containers` 按全部容器数开标记表）；用户定案 2026-10-31：代价条款「管，只管准入之前」（无实验：用户定案，原话在变更史）。

**欠**：C618（删完之后不能再建带数据的文件）、C619（墓碑树根长高那次发布没有崩溃流）、C614（回退时墓碑树取错与删完对象无墓碑单镜像判不了）；C118（`deleted_inodes` 树的形态无落点） 还清。

```

## 三、实现今天的样子

读过的事实（`crates/singlefs-core/src/` 下，行号为登记写时的；主 agent 复核时按行号看）：

- **B8 不是实现**：`grep -rln 'by_delta\|差量视图\|DeltaView' crates/` 零命中。B8 的步数是装置按 E203 登记第五节式子代入量到的输入算出来的列（E206 模型 `scripts/experiment-apparatus/e206_fixed_point_cost.rs:305` 的 `by_delta_rows`），不是在 B8 实现上量的；所以「B8 站住」说的是这份式子在今天的固定点、今天的发布上的读数，不是某段代码的性质。
- **今天的固定点已带 P2a**：`transaction.rs:13825` 的 `settle_the_allocation_record_tree`，循环变量 `round_index`（E206 的补丁锚点 `for _round in 0..…` 已不存在）；释放弄脏的节点并进重写集走 `:13918`，取落点后变了的节点走 `:13945`，两处都调 `:13962` 的 `merge_into_the_rewrite_set_walking_the_release_chain`——该函数从新并进的节点起，按上一版指针（`allocation_record_tree.rs:487` 的 `pointer_of`）找旧落点，经 `leaf_of_slot`（`:186`）、`parent_of`（`:200`）把落点所在的叶与祖先全并进来，直到不再多（`:13962`–`:13990`）。轮数上界 `ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT = 64`（`:13994`）。树表 0 条那一版的同形固定点 `:1942`，两处同名调用 `:2013`、`:2036`。
- **E206 测量补丁在今天的树上锚点对不上**（登记写时 `grep -cF` 对 `transaction.rs`，量的）：`rewritten.extend(changed_by_the_release);` 0 次（E206 脚本 `e206_fixed_point_cost_counters.py:725` 的 `RELEASE_MERGE`）；`        rewritten.extend(changed);` 0 次（`:726` 的 `PLACEMENT_MERGE`）；`for _round in 0..ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT {` 0 次（`:729` 的 `LIMIT_LOOP`，今天写成 `for round_index in …`，2 次，因为行版那一份也有）；其余五个锚点（`NODES_AFTER_DECLARED`、`ROUND_BEGIN_ANCHOR`、`REHEARSAL`、`CHANGED_BY_THE_RELEASE`、`HOLDING`）在整份文件里各 2 次，函数体内是否各恰好一次要执行员 `--check-anchors` 现核。E203 编辑表（被 E206 脚本 import 的那一份）在今天的树上是否全部命中**没核**，由停机条款第 1 条管。`transaction.rs` 自 E206 登记（sha256 `805afb9d…`）起已改，`allocator.rs`（`a07e50d8…` → `36fe6af3…`）也改了；`allocation_record_tree.rs` 没变（`991623b6…`）。
- **每叶槽数与扇出**：`crates/singlefs-format/src/lib.rs:163` 的 `ALLOCATION_RECORD_TREE_LEAF_SLOTS = 812`、`:168` 的 `ALLOCATION_RECORD_TREE_INTERNAL_FANOUT = 169`；节点按位置寻址（`allocation_record_tree.rs:186` 的 `leaf_of_slot` 用槽号除叶槽数），所以「同一个节点」在不同版本里是同一个位置，量「节点上次被重写是第几次固定点」有意义（读法见本节读数表第 4 组）。
- **覆盖写的入口**：`UserChange::OverwriteOfAnInode`（`mounted_session.rs:173` 起的文档：「按 inode 号覆盖写一个数据单元以内的内容……把 `inode` 那个文件整份换成 `content`」），实现 `transaction.rs:10051` 的 `publish_overwrite_of_an_inode`。整份换走是**一次发布**；这是判决第二节「第三条路」的前提，本实验不量分批。
- **历史驱动**：`scripts/experiment-apparatus/e206_fixed_point_cost_histories.rs`：点表 `point_named`（`:88` 起）、形状 `Shape`（`:54` 起）、`small_files`（`:307`）里 `Shape::SmallFilesOfUnits(n)` 每个小文件写 n 个数据单元（`fill_a_file`，每次写至多 400 个单元）、量窗 `Measured::window`（`:598` 起，W1 覆盖写第一个文件 32 次、W2 覆盖写冷文件 32 次、W3 建—写—删 8 次、W4 再写第一个文件 8 次，之后重挂载再做一遍）。W2 取冷文件 `live[share * live.len() / 32 % live.len()]`：文件少于 32 个时同一个文件会被取多次，第二次起它已是 1 个单元（`OVERWRITE_BYTES` 不到一个单元）。

### 每个读数的取样位置（写死；`T` = `transaction.rs`，`R` = `allocation_record_tree.rs`）

读数沿用 E206 登记第三节同名读数的定义（`research/prompts/e206-preregistration.md:185`–`:213`），取样位置按今天的行号重定；E206 的读数名与字段不改。本实验**新增**的只有 3、4 两组。

| 组 | 读数 | 取样位置 | 什么时候记 |
|---|---|---|---|
| 1 每次固定点 | `settle_call`、`rounds`、`outcome`、`final_roles`、`final_nodes`、`round_work`、`final_set_hash`、`slots_placed` | 调用处 `T:13296`；循环体 `T:13825` 起；定下处 `T:13942` | 全部发布，含历史 |
| 2 每轮 | `roles_rewritten`、`nodes_rewritten`、`placements_released`、`release_dirty_new_*`、`placement_dirty_new_*`、`round_exit`、`closure_hops`、`closure_nodes_added`、`closure_lookup_steps`、`release_chain_depth`、`release_chain_nodes` | `T:13915` 之后与 `T:13935` 之后；闭包三项在 `T:13962` 函数体里计 | 量窗 |
| 2 | 每个 `settle_call` 的 B8 输入（`name=round_row` 与 `name=state`） | 同 E206 | 量窗 |
| 3 新增：声明 | `E207RESULT name=declared item=… phase=window state=… op=… op_index=… user_units=… old_units=… new_units=…` | 历史驱动每次调 `Measured::measure` 之前，由驱动自己的 `units_of[inode]` 记账写 | 量窗每次操作 |
| 4 新增：节点年龄 | `pulled_age_min`、`pulled_age_median`、`pulled_age_max`、`seed_nodes_first_merge`、`pulled_nodes_first_merge`、`first_merge_site`（`release` 或 `placement`） | `T:13962` 函数体：入口时的 `rewritten` 集合之差 = 新并进的；节点年龄 = 当前 `settle_call` 序号 − 该节点上一次出现在**定下那一轮**的 `rewritten_by_the_plan` 里的 `settle_call` 序号（线程内一张 `节点 → 序号` 表，在 `T:13942` 定下处更新；没出现过的记序号 0）。被拒重判与抬 F 预演也各占一个序号（读法：年龄的单位是 `settle_call`，不是用户改动） | 量窗 |

## 四、跑之前已经存在的数

读派发提示点名的判决、岔路单、两条腿的报告，以及按派发提示读 `:706`、`:520`、`:3485` 三处时，已经知道或算出的数，全部照列。**这次判据是在知道这些数之后写的**：下面逐条写对判据的影响。腿的读数一律标「线索」（来源与该腿的标记照抄：「推的，待主 agent 跑」或「量的」）。我读产物时越过了派发提示点名的行，多读到的也列在这里（第 7、8 条）。

| # | 数 | 量的还是推的，出处 | 对判据的影响 |
|---|---|---|---|
| 1 | 记一次输那一行（第一节）：`q8_peak p2a data_units` 低端 3763、高端 3231038、α 0.978 | 量的，`research/results/e206-fixed-point-cost-report-2026-11-09.out:706`（`sed -n 706p`） | 第七节 ② 第 1 条钉回扫读数器；B8 在原判据下已记输，本登记不改这一笔 |
| 2 | 严格形态（x 取记录数）在 40 份产物上的读数：三轴 0.385、0.147、0.349；轴外 `files_30000` 0.591、`files_1000_units_100` 0.715；量窗第 2 次起 0.20–0.26；正推按活单元数端点 0.55 | 判决第二节「新形态判 B8（P2a）」一格，来自攻方（只一次采样）与正推（`out/offaxis.txt`），线索 | 说明 B8 在新判据下回扫已不是干净的站住（尖峰三臂共用，判决原话）。第六节 Q4 的回扫读数不是证实，本实验是在知道它们之后立的判据，回扫只用来校准读数器、阳性对照 |
| 3 | 改动随轴涨那一种（`data_units` 轴 W2 第 1 次，T 为 24、114、1028、10152）每用户单元步数：342.1、170.3、178.4、320.9（攻方）；156.8、150.8、174.9、318.3（正推，分母取第一轮释放加重写角色） | 攻方 `research/prompts/e203-b8-criterion-r1-opus-output.md` 第二节「改法」之前的 `out/user_units.txt` 摘录与判决第二节「分母写成哪个」一格，线索，推的待跑 | `uK` 档每单元步数约 150–340 是第 12 条推算 `C1` 判别力的输入；分母取外生的 `user_units` 而不取 T 是判决已定的 |
| 4 | 攻方的合成臂在旧形态下的读数（x 取记录数、不除分母、W1 与 W4 峰值）：`H_scan_every_publish` 三轴 0.710、0.614、0.625；真实 B8 同格 0.385、0.147、0.349；比值形态（除以 `Rs`）下 `H_scan_every_publish` 四列无一格随；`H_scan_only_big`（只在 `Rs ≥ 1000` 时多扫一遍）八列与真实同值或差 0.001；B8 的 s25 换回线性：原步数 0.788、0.549、0.470（旋钮 x，W1 第 1 次），除以 `Rs` 后 W2 第 1 次 0.462（介于） | 攻方第三节 3.2、3.3，线索，推的 | 阳性对照必须用 x = 记录数才红（判决第二节「x 取旋钮还是总量」）；比值形态下的对照可能不红，所以第五节对照臂在每格上用**与被测臂同一把尺**判，不红照作废条款走。`H_scan_only_big` 在 40 份产物里看不见，是新点要补的 |
| 5 | 40 份产物里（P2a 3336 次发布）满足「`Rs ≥ 1000` 且改动远小于总量（`Rs ÷ mapped < 0.5`）」的 0 次；「同一种改动、总量差 813 倍」只有一对（W2 换约 100 单元的文件：`data_units_100` 对 `files_1000_units_100`），α 0.141 | 攻方 3.3 的分桶与 `out/user_units.txt`，推的 | 新点必须造出这一格：K1000、K100 两个集合；攻方预估 `files_100_units_1000`、`files_10_units_10000` 的 W2 `Rs` 约 2000、30000（推的，线索） |
| 6 | 正推给的 `files_then_big_10000` 判读线：每种发布 B8 不超过 `files_10000` 同一种的 2 倍（`files_10000` 最大 15440）；超过的那一种按推翻记。2 倍是正推的取法（推的）。正推还说 `files_then_big_*` 的 files 与 data_units 同时涨，拆不出哪根轴 | `research/prompts/e203-b8-criterion-r1-sonnet-output.md` 3.5，线索 | 第六节 Q8 采用这一条并分开写：它是 FTB 集合的附带判读，不是行一的判据；其 2 倍我在 x 十倍时对应 α 约 0.30（`calc/arith2.out`），落在「介于」的带内，所以 Q8 另报 α 档，不单看 2 倍 |
| 7 | `files_1000_units_100` p2a 产物里历史每次发布：`rounds` 2 或 3、`final_roles` 约 120、`final_nodes` 7、`closure_nodes_added` 0、`seed_nodes` 0（产物 500–519 行摘到的 10 余条 `history_settle`；末条 `previous_nodes=551`） | 量的，`research/results/e206-fixed-point-cost-files_1000_units_100-p2a-2026-11-09.out` 500–519 行（越过派发提示点名的 520 行，多读） | 读法 8 的尖峰旗下限 28 = 4 × 7 取自这里；也说明历史里没有尖峰（至少这一段没有） |
| 8 | 同一产物 W1 第 1 次（520 行与它前后）：`rounds=3 final_roles=452 final_nodes=433 round_work=917 closure_hops=864 closure_nodes_added=410 previous_nodes=551 previous_records=238324 active_leaves_by_device=273,273 tree_height=3 release_chain_depth=85 release_chain_nodes=431`；第 1 轮 `roles_rewritten=15 placements_released=15 release_dirty_new_leaves=16 release_dirty_new_internal=5 round_exit=release_not_covered closure_nodes_added=410`；`state` 行 `mapped_units=101581 allocation_records=238324`。W1 第 2 次（3485 行）：`rounds=2 final_roles=20 final_nodes=7 round_work=32 closure_nodes_added=0 release_chain_depth=1 release_chain_nodes=7 previous_nodes=553 previous_records=239228` | 量的，同一产物 519–540、3484–3485 行（多读） | **尖峰成因的一半已经在手**：尖峰那次发布 433 个重写角色里约 410 个是第 1 轮 `merge_into_the_rewrite_set_walking_the_release_chain`（`transaction.rs:13918` 那一处）顺着释放链新并进来的，种子只有 16 + 5 = 21 个弄脏的叶与祖先，释放的落点只有 15 个。所以「角色由什么决定」的答案在读数里已经有；**第三行还开着的是**：这 410 个节点为什么「攒下」（年龄）、什么时候再攒够、不同操作能不能重现。第八节 Q9 把「第 1 轮首次合并的闭包」写成**待证伪的假设** H1，不当结论；探针（第五节）是为其余几问设的 |
| 9 | 第 8 条的数代入：原步数按记录数 α = ln(168373 ÷ 2102) ÷ ln(238324 ÷ 922) = 0.789 | 推的（算式输入：2102、922 来自攻方 1.3 节表；168373 来自攻方引的报告 379 行；`$D/calc/arith.awk`，输出 `arith.out`） | 与攻方在 `files_1000_units_100` 上「raw 按记录数 0.789」同值，算式对得上；说明回扫读数里这一格的高读数就是尖峰（W1 第 1 次） |
| 10 | 主工作区已带 P2a；E206 补丁三处锚点对不上（`transaction.rs` 0 命中：`rewritten.extend(changed_by_the_release);`、`rewritten.extend(changed);`、`for _round in 0..…LIMIT {`） | 量的（第三节 `grep -cF`） | 停机条款第 1 条的前件已成立：执行员必须重写测量补丁（第五节），不能直接跑 E206 脚本 |
| 11 | 回扫点上 P1 与 P2a 同一不动点；P1 逐轮读数 E206 页作废 | 判决第一节与 E206 页修订 2 | 本实验没有 P1 臂（读法 10） |
| 12 | 阳性对照判别力的推算（第 3a 步②）。`C1`（每次发布多扫一遍，1 步/记录）：`u1` 档代入 W1 第 2 次 5500 步（攻方）与记录数 239228，低端猜 4500 步、23923 记录：α 0.935，真实臂同格 0.087；`uK` 档 K = 1000：每单元 178.4 加 23.9 与 239.2：α 0.315；系数 4 的 `C2`：0.617；要 α ≥ 0.5 系数至少约 2.5；K = 10⁴：多 24 步对每单元 318 步，比 1.075。`C3`（最大行换线性，s25 的 `lg(记录数)` 换成记录数，calls = 4 × 释放，W1 第 2 次 `Rs = 32`）：`u1` 档 α 1.107，比真实 s25 大约 5.6 × 10³ 倍 | 推的（`calc/arith.out`、`calc/arith2.out`；低端值是猜的，不是量的；calls = 4 × 释放来自判决第二节「分母取各轮释放之和」一格，线索） | 写进未定项第 1 条：`C1` 在 `uK` 档预期不红（登记已预告，不是事后），`C1` 在 `u1` 档、`C3` 在两档预期红。读数全出来之后与这一条对：预期红的没红，走失败条款 F5 |
| 13 | 带边的分辨：x 比 10 倍时，α 0.25 对应 Q 比 1.778、α 0.5 对应 3.162；FTB 的 2 倍线对应 α 约 0.301 | 算的（`calc/arith.out`、`arith2.out`） | 步数是整数，分辨率 1，确定性历史无噪声；两点 α 的分辨在 Q 的千分之一以内，够分带；退化的是控制臂不是读数 |

第 3a 步②（基线符号与量级）：被测臂在 `u1` 档预期的 α 由第 2、9 条看是正的、量级在 0.1–0.4（真实）；`C1` 约 0.9，`C3` 约 1.1，差都在 0.5 以上，分得开；`uK` 档上 `C1` 分不开（第 12 条），登记不改判据，已写进未定项。
第 3a 步③（登记点上要触发的事件，只对触发是前提的做）：`uK` 档要在量窗里碰到仍是 K 个单元的文件，这是前提不是被问的量。算式：W2 取冷文件，`live` 个数为 F + 1（含大文件）；E206 的取法在 F = 10 时 32 次只碰到 11 个不同文件，第一次碰到时才是 K 个单元；所以新点的量窗把冷文件池按挂载状态劈成两半（第五节「量窗」），`same_mount` 取前半、`remounted` 取后半，F = 10 时每个状态至少碰到 5 个仍是 K 个单元的文件，F = 100 时 50 个，F = 1000 时 500 个。会发生，不用改几何。另一件前提：尖峰要一次释放落到很久没重写的叶上；E206 的历史在 W1 第 1 次上已触发（第 8 条），探针 `young_first`、`create_first` 就是不触发的对照。

## 五、臂、阳性对照、真实基线

### 装置写在哪（写死）

**走 `scripts/experiment-apparatus/`，不写入库装置，不改主工作区的 `crates/`。** 这条岔路问的是一份式子（B8 列）在今天固定点的读数上的增长阶，加固定点里一个节点年龄的成因；固定点是私有函数，逐轮读数外露不了，所以同 E206：拷来的实现上插计数，另写一个与实现不共用代码的模型。要读 `crates/` 常量的地方（812、169、上界 64）本地写一份常量，单测与产物头逐个比，不直接引。三份新文件，E206 的文件一个字不改：

1. **量的那一路**：`scripts/experiment-apparatus/e207_b8_strict_counters.py`（今天的树上重写测量补丁；import E203 编辑表与 E206 脚本里不依赖旧锚点的部分，不改它们）：把主工作区那一刻的 `crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo`、`.claude/heavy-thread-default` 拷进工作目录，先插 E203 计数，再插第三节的 E206 同名读数（锚点按今天的 `merge_into_the_rewrite_set_walking_the_release_chain` 重写）与第 3、4 组新读数；每个锚点在它那个函数体里恰好命中一次，否则退 3、一个字节不写。构建只有 `counts` 与 `plain`（只拷不插，给作废条款第 3 条比盘）两种；臂只有一个，就是今天的实现（环境变量 `E207_ARM=head|plain`）；探针由 `E207_PROBE=standard|cold_first|create_first|young_first` 在进程开头读一次。
2. **历史用例**：`scripts/experiment-apparatus/e207_b8_strict_histories.rs`（拷自 E206 的同名文件另写，不共用文件），拷进副本当 `crates/singlefs-verification/tests/e207_b8_strict_histories.rs`；一个 `<点>__<臂>` 一条用例；参数同 E206（`e142_parameters(512, 512)` 加 6 MiB 环，两块 `SparseBlockDevice`）。
3. **推的那一路**：`scripts/experiment-apparatus/e207_b8_strict_criterion.rs`（bin 名 `e207-b8-strict-criterion`，写进 `scripts/experiment-apparatus/Cargo.toml`），纯算术、不 `use` `crates/`；读第一路产物与 E206 的 40 份 p2a 产物（只读，回扫），算严格读数、控制臂、拆分表、尖峰旗；B8 列的式子是 E206 模型 `by_delta_rows`（`e206_fixed_point_cost.rs:305`）的本地重写，单测钉第七节 ② 与 ③；变异表 `scripts/mutation/tables/e207_b8_strict_criterion.tsv`；进 `scripts/experiment-apparatus/replay.sh`（对已落盘产物复跑，`exact`）。
4. 入口 `scripts/experiment-apparatus/e207-b8-strict-run.sh`（形态照 `e206-fixed-point-cost-run.sh`）与判定库脚本 `e207_b8_strict_library.py`（形态照 `e206_fixed_point_cost_library.py`）。

装置不调判段入口（`crash_amplification`、`GivenStatesEngine`、`crash_state_enumeration` 那一族一个都不调），所以不带 `--features gpu`，也不立 `grep -c NOT_RUN` 那一条判据。不量计时（计数实验，不读时钟）。

| 共用项 | 量的那一路 | 推的那一路 |
|---|---|---|
| 被测代码 | 就是它（拷来的实现，只多计数） | 不读；B8 列写成本地式子 |
| 常量 812、169、上界 64、两块盘 | 实现自己的（打进产物头） | 本地常量注明 `文件:行`，单测与产物头比，对不上走停机条款第 2 条 |
| `user_units` | 驱动按构造声明（读法 2），不读实现 | 吃声明行；与实现的 `mapped_units` 差另核（第七节 ② 第 4 条） |
| 控制臂 | 不在这一路里算 | 在模型里对同一批读数加项（下表） |

### 判定入库与按点单跑（写死）

同 E206 第五节同名一节，名字换成本实验：判定只经 prune-server 客户端写进那一份库（`.claude/rules/verification.md`「判定库只有一份」），族 `apparatus-items`，种类 `e207-b8-strict-run`；每个 `<点>__<臂>` 一条判定，键 = 副本里那条用例的单元闭包键加工具链行、闭包真引到的依赖行、臂、探针与跑法；键相同做完了的复用，每趟抽复用哨兵与一条强制现判；入口参数 `--list-items`、`--item <项>`（可多次）、`--segment first|second|third|fourth`、`--selftest`；每项结束打一行，历史段每 5 分钟打一行 `e207 progress item=<项> publishes_done=<n> of=<总数> peak_rounds=<r>`；退出码 0、1、2、77、78、3、75 含义照 `.claude/rules/test-script-discipline.md`。新脚本的登记（`.claude/gate.d/verdict-library-scripts.tsv`、`.claude/hooks/selftest-registry.tsv`、`.claude/hooks/selftest-temporary-service.tsv` 各一行）不在执行员写范围里，写进报告由主 agent 派。

### 取样点（写死）

一个点 = (U, F, K, D) 加形状：F 个小文件各 K 个数据单元，D = 10 的大文件一个，其余同 E206（u = 10 次空 inode 建删）。`U` 取 E206 对同量级点的取法（`files_1000_units_10` 在 2¹⁸、`files_1000_units_100` 在 2¹⁹）。「发布数」是历史里用户改动发出的发布，推的。

| 点 | U | F | K | 总单元约 | 形状 | 发布数约 | 段 | 用途 |
|---|---|---|---|---|---|---|---|---|
| `base`、`files_1000`、`files_10000` | 2¹⁷ | 10、10³、10⁴ | 1 | — | E206 同名点（`BigThenSmall`） | 50、1000、10000 | 第一段 | 与 E206 p2a 产物对账（第七节 ② 第 2 条）；`files_10000` 同时是 FTB 的对照 |
| `files_1000_units_100` | 2¹⁹ | 10³ | 100 | 10⁵ | `SmallFilesOfUnits(100)` | 1000 | 第一段（探针 `standard`） | 尖峰复现钉；K100 集合的高端 |
| `files_100_units_100` | 2¹⁹ | 10² | 100 | 10⁴ | `SmallFilesOfUnits(100)` | 约 200 | 第一段 | K100 集合的低端（我加的，补「改动相同、总量差十倍」） |
| `files_10_units_1000` | 2¹⁹ | 10 | 10³ | 10⁴ | `SmallFilesOfUnits(1000)` | 约 40 | 第一段 | K1000 集合的低端（我加的） |
| `files_100_units_1000` | 2¹⁹ | 10² | 10³ | 10⁵ | `SmallFilesOfUnits(1000)` | 约 300 | 第一段 | 攻方提的；K1000 高端；T1e5 |
| `files_10_units_10000` | 2¹⁹ | 10 | 10⁴ | 10⁵ | `SmallFilesOfUnits(10000)` | 约 250 | 第一段 | 攻方提的；T1e5；改动占总量约 10% |
| `files_then_big_1000`、`files_then_big_10000` | 2¹⁷ | 10³、10⁴ | 1 | D = F | E206 同名点（`SmallThenBig`） | 1000、10000 | 第一段 | FTB 集合；E206 已登记 |
| `files_1000_units_100` × 探针 `cold_first`、`create_first`、`young_first` | 2¹⁹ | 10³ | 100 | 10⁵ | 同上，量窗前多一次操作（见下） | 1000 | 第二段 | 行三的重现条件 |
| `files_1000_units_1000` | 2²¹ | 10³ | 10³ | 10⁶ | `SmallFilesOfUnits(1000)` | 约 3000–4000 | 第四段 | 大点：K1000L 高端；与低端活单元数差 100 倍 |
| `files_10_units_1000_unit_area_2_pow_21` | 2²¹ | 10 | 10³ | 10⁴ | `SmallFilesOfUnits(1000)` | 约 40 | 第四段 | K1000L 低端（与大点同 U，U 不混进 α） |
| `files_100_units_1000_unit_area_2_pow_21` | 2²¹ | 10² | 10³ | 10⁵ | `SmallFilesOfUnits(1000)` | 约 300 | 第四段 | K1000L 中点（两段局部斜率；{中点, 大点} 总量十倍） |

**可达核过**（第 3a 步①）：K 单元的小文件走 E206 已有的 `fill_a_file`（每次写至多 400 个单元，`e206_fixed_point_cost_histories.rs:238`），E206 的 `data_units_10000`（D = 10⁴）与 `files_1000_units_100` 已按同样写法跑过；新点只是 `SmallFilesOfUnits` 的参数取 1000、10000 与 F 取 10、100。单元区够不够：10⁵ 个数据单元的点在 E206 里放在 2¹⁹（`files_1000_units_100` 的 `state` 行 `units=102276`，每盘 2¹⁹ = 524288 槽），我沿用；10⁴ 个单元的点也取 2¹⁹，为的是每个比较集合（K100、K1000、D100、D10、T1e5）两端的 U 相同，U 不混进 α；FTB 两端都是 2¹⁷。单元区只写到过的槽才占内存（第十四节），不按 2¹⁹ 满额算。**几何敏感性的两个点**（第八节，第一段）：`files_10_units_1000_unit_area_2_pow_17`（F = 10、K = 10³，U = 2¹⁷：10⁴ 个单元放得下，E206 的 `data_units_10000` 在 2¹⁷ 放过 10⁴ 个单元）与 `files_100_units_1000_unit_area_2_pow_20`（F = 10²、K = 10³，U = 2²⁰）；它们是 K1000 集合两端换一个 U 的版本。**第三段（条件项，只在第二段之后还有一行开着才跑，否则记「够判后未跑」）**：`files_30000` 加扩展窗口（U = 2¹⁸，尖峰在另一种几何上重现不重现）。**第四段（用户 2026-11-09 定，独立的最后一段）**：上表最后三行，细节见下一段。

**大点可达核过**（第 3a 步①，第四段，数都是推的或量的，标在句里）：① 单元区：10⁶ 个活单元，元数据额外约 1.6%（第四节第 8 条：`files_1000_units_100` 的 `mapped_units` 101581 对 10⁵ 个数据单元，算的），约 1.02 × 10⁶ 槽，U = 2²¹ = 2097152 槽放得下（占 49%）；2²⁰ = 1048576 占 97%，太紧，所以取 2²¹。E206 点表另有 `data_units_1000000`（U = 2²²、发布约 2500，`e206_fixed_point_cost_histories.rs:128`），同量级单元数在点表里已有登记；它跑没跑过、读数是什么我没读。② 发布数：每个文件建一次、写 ⌈1000 ÷ 400⌉ = 3 次（`fill_a_file` 每次写至多 400 个单元），约 1000 × 4 = 4000，若建与首写同发布则约 3000，加 u = 10 次空 inode 建删。量窗 W1–W4 两种状态各 80 次操作，共 160 次；大点**不带**扩展窗口 W5–W7。③ 内存：写到过的槽约 1.02 × 10⁶ × 2 块盘 × 16 KiB ≈ 33 GiB；`run_point_under_arm` 在历史末取 `image_sha256` 要复制盘（`e206_fixed_point_cost_histories.rs:693` 的 `image_sha256(&pool.image())`），重挂载是否也复制一份我没查；最坏两份约 66 GiB。本机内存 60 GiB（`free -g`，量的；第二台没量）。所以**大点不取 `image_sha256`**（`plain` 对盘只在 `base`、`files_1000`，作废条款 3），重挂载是否复制由执行员先在 `files_100_units_1000_unit_area_2_pow_21`（10⁵ 个单元）上量峰值 RSS 再乘 10 写进报告；装不下就是大点「本次未跑」，不挡第一至三段，由主 agent 另议（哪台机器服务定）。④ 挂钟：4000 次发布，每次耗时有两个极端：不随总量涨（约 0.6 秒，取自 `files_1000_units_100` 的 E206 估计 10 分钟 ÷ 约 1000 次发布）→ 约 40 分钟；与总单元数成正比（十倍，约 6 秒）→ 约 6.7 小时；几何中点约 2 小时，加量窗 160 次操作约 10–20 分钟。都是推的，没量，区间 0.7–7 小时。超 2 小时很可能，处理见第十四节。⑤ 总量跨度：K1000L 两端活单元数设计比 100.0；`allocation_records` 的比由元数据占比定，低端元数据占比更大，量到的比可能略低于 100，如实报。

### 量窗（W1–W4 同 E206；W2 的挑选与扩展窗口是这一次改的）

1. **对账点**（`base`、`files_1000`、`files_10000`）：W1–W4 与 E206 逐字节相同的取法，包括 W2 的 `live[share * live.len() / 32 % live.len()]`；这样才对得上 E206 的 p2a 产物（第七节 ② 第 2 条）。`files_1000_units_100`（四个探针项）只钉 W1 第 1、2 次（第七节 ② 第 3 条），W2 起用下一条的劈开池：这样它在 `remounted` 里也有第一次碰、仍是 100 个单元的文件，K100 集合的 `uK` 档两种状态都有样本。
2. **新点**：W2 的冷文件池按挂载状态劈开：把 `live` 去掉大文件（SmallThenBig 里是最后一个，别的形状是第 0 个）得 n 个小文件，`same_mount` 取前 ⌈n ÷ 2⌉ 个、`remounted` 取其余，第 i 次取 `pool[(i − 1) × pool.len() ÷ 32 % pool.len()]`。这样两种状态里都至少碰到 5 个第一次碰、仍是 K 个单元的文件（第四节第 3a 步③）；E206 的取法在 `remounted` 里碰到的全是同挂载里已经覆盖写成 1 个单元的文件。
3. **扩展窗口**（只在 `files_1000_units_100` 的四个探针项里——`standard` 在第一段跑、另三个在第二段——和第三段的 `files_30000` 里；同一次挂载，接在 W4 之后、重挂载之前）：**W5** 覆盖写 200 个互不相同、W1–W4 没碰过的冷文件（把没碰过的按 inode 号升序，取等间隔的 200 个）；**W6** 建 100 个新 inode，各写 100 个单元（同历史第 3 步的写法，10 批每批 10 个）；**W7** 覆盖写第一个文件 1 次、W5 没碰过的冷文件 5 个、W6 新建的文件 5 个。每个 W5–W7 操作逐次全记。
4. **探针前缀**（`E207_PROBE`）：`standard` = 不加；`cold_first` = W1 之前加一次操作 `P1`：覆盖写 `live[len ÷ 2]`（中间的冷文件）；`create_first` = `P1` 建 1 个新 inode；`young_first` = `P1` 覆盖写 `live` 里最后一个小文件（历史里最后建的）。`P1` 也逐次全记；它之后 W1–W4 照旧。

### 阳性对照（每条控制臂对每一格都判）与真实基线

被测臂只有一个（今天的实现，第一节读法 10），所以「对每一条臂都跑」落在：**每一条控制臂对模型里枚举出的每一格（比较集合 × 挂载状态 × `user_units` 档 × `first` / `later_peak` / `later_median` × `all` / `steady`）都用同一把尺判一档**，模型的循环遍历全部格，不是拿第一格试一试。加格时连对照那一段一起改。

| 控制臂 | 怎么做 | 退化形态（做完会怎样，推得出） | 判过的条件 | 过不了 |
|---|---|---|---|---|
| `C1` = `H_scan_every_publish` | 每次操作的 B8 步数 + 操作之前的 `allocation_records`（每个记录 1 步，全表扫一遍） | 多出的一项对记录数的 α 恰为 1，所以它在被测臂占小头的格里把读数拉到「随」 | 在**被测臂判不随或介于**的每一格上，`C1` 用同一把尺判「随」 | 该格所在（段 × 档）作废（作废条款第 1 条）；`uK` 档预期不红，见未定项第 1 条 |
| `C2` = `C1` 系数 4（**附带**） | 同上，每个记录 4 步 | 同上，分辨力强 | 不作为作废条件；报它红不红，给未定项第 1 条的选项（b）看 | 不作废，只报 |
| `C3` = B8 最大行换回线性 | 每个点、每个状态：在 `by_delta_rows` 里**含 `lg(Z)` 因子**的行里，取该点该状态量窗全部操作里步数合计最大的一行，把它的 `lg(Z)` 换成 `Z`（Z 是该行吃的同一个量）；同时报这一行是不是 `allocator_release_linear_find`（s25） | 这一行的价格从对数变线性，对 Z 的 α 恰为 1 | 在被测臂判不随或介于的每一格上，`C3` 判「随」 | 同 `C1`（作废条款第 2 条） |

**真实基线**：被测臂本身。它回答行一、行二、行三；E206 的 `single_step`、`closure_off` 在本实验不做（P2a 已在今天的树里，两条是 P1 / P2a 补丁的退化）。

**阴性对照**（沿用 E206 的阴性对照点）：`base`、`files_1000`、`files_10000` 的量窗末尾，用 `truncate_to_the_same_size`（`new_size = OVERWRITE_BYTES`，E206 histories `:632`）发一次没有普通分配的发布；它的 `rounds ≤ 4`；过不了说明计数记错了发布，该点读数作废（作废条款第 4 条）。

### 实现量与分段（第 3b 步）

实现量（推的，按 E206 同形部件比）：测量补丁脚本约 700 行（含重锚点与第 3、4 组读数、自证）；历史用例约 900 行 Rust（拷 E206 的 1144 行，加点表、声明记账、W2 池劈开、探针与 W5–W7）；模型连单测约 1500 行（拷 `by_delta_rows` 与读取，加严格读数、控制臂、拆分表、旗）；判定库脚本与入口各约 300–400 行（照 E206 两份改）；变异 14 条。不另写检查器、触发式重算、崩溃枚举、坏镜像，崩溃支线 0。

- **第一段**：全部装置；对账点（`base`、`files_1000`、`files_10000`、`files_1000_units_100` 的 `standard`）加 plain 对照；新点 `files_100_units_100`、`files_10_units_1000`、`files_100_units_1000`、`files_10_units_10000`、`files_then_big_1000`、`files_then_big_10000`；两个几何敏感性点；模型与回扫。**补行一（站住）、行二（翻面）**；同时补行三的一半（尖峰在哪些点、哪个状态出现，`pulled_age`、首次合并发生在哪个站点）。补完行一、行二够不够判：`u1` 档与 FTB 在 `C1`、`C3` 红的条件下够；`uK` 档取决于未定项第 1 条。
- **第二段**：只有行三还开着才跑：`files_1000_units_100` 的 `cold_first`、`create_first`、`young_first` 三个探针项；`standard` 在第一段已有（也带扩展窗口 W5–W7，所以第一段里这一项也要跑扩展窗口）。补完行三（重现条件）。
- **第三段**（有条件）：第三段点 `files_30000` 加扩展窗口（见上）。只有第二段之后行三仍开着才跑。
- **第四段**（用户 2026-11-09 定，独立的最后一段，入口 `--segment fourth`）：`files_10_units_1000_unit_area_2_pow_21`、`files_100_units_1000_unit_area_2_pow_21`、`files_1000_units_1000` 三项，加模型里 K1000L 与 Q10 那一段。补岔路单行一的「改动至少 1000 个单元、改动远小于总量」`uK` 档那一半与行二的 `uK` 档。补完够不够判：`C1`、`C3` 在 K1000L 的 `uK` 档红（已定项 1：贴线）则够，不红则 `uK` 档仍只报数，交主 agent。实现增量：点表加三个名与 U 取 2²¹（约 10 行）、模型加 K1000L 集合与 Q10（约 150 行）、变异 2 条（M15、M16）。

每段交回附岔路表（已够判 / 还差什么 / 剩下的量能不能让它翻面），主 agent 判续不续。**段与段的依赖**（让执行员一段一段交）：

| 段 | 前置 | 读不读别的段的读数 | 补岔路单哪一行 | 它没跑完时别的段 |
|---|---|---|---|---|
| 第一段 | 无（装置全在这一段写完） | 否 | 行一、行二（`u1` 档与 FTB 全判；`uK` 档按作废条款 1 只报数）、行三一半 | 不依赖第二、三、四段 |
| 第二段 | 第一段 `standard` 项的复现钉过（第七节 ② 第 3 条） | 读第一段 `files_1000_units_100` 的 `standard` | 行三 | 不依赖第三、四段 |
| 第三段（条件） | 第二段交回后行三仍开 | 否 | 行三 | 不依赖第四段 |
| 第四段（用户定，无条件列入计划；起不起由主 agent 在第一段交回后对着岔路单判：行一的 `uK` 档那一半仍开则起） | 第一段的装置（构建、读数器、对账钉过，停机条款 1–4 未触发）；点表三个新名 | 否：不读第一至三段任何点的读数；`$D` 里 Q10 只吃第四段自己的产物 | 行一的 `uK` 档、行二的 `uK` 档 | 大点没跑完、装不下内存、被停或作废，第一至三段的结论照出；第四段另两个小点是独立任务，不等大点 |

## 六、报哪些量与各自的判据

每个量各占一行、各报各的判定，不合取；合取留给主 agent 对着岔路单做。计数是确定的（读法见 E206 登记读法第 5 条），不做多轮统计；证据靠第七节钉绝对值的断言与第九节变异。所有 α 的 x 默认取 `allocation_records`（读法 3）；每个 α 行同时报 x 取 `mapped_units` 的那一份（「另一个报」）。

| 量 | 怎么算 | 门槛与判定 | 对应岔路单哪一行、翻面取值 | 可跑性 / 分辨率 |
|---|---|---|---|---|
| Q1 `u1` 档的严格读数 | 集合 K100、K1000、FTB 各两点；每个（集合、挂载状态、`first` / `later_peak` / `later_median`、`all` / `steady`）一格：两点各取该格的原步数（两端 `user_units` 都是 1，`looser=no`）、各自量到的 `x`，算 α | 档同读法 4。判定逐格写：不随 / 介于 / 随 | **行一**：新点上全部 `u1` 格（`steady`，两种状态，`first` 与 `later_*`）都「不随」，且第五节 `C1`、`C3` 在这些格上都「随」⇒ 这一半站住。**行二**：任一 `later_peak` 或 `later_median` 的 `steady` 格「随」（α ≥ 0.5）且 `C1`、`C3` 在该格「随」⇒ 翻面（步数不只第一次发布随总量涨）。「介于」两行都不翻，如实报 | 整数步数，两点 α 的分辨在千分之一以内；x 跨度约十倍（K100、K1000）与约十倍（FTB），带边在 Q 比 1.78 与 3.16 |
| Q2 `uK` 档的严格读数 | 集合 K100（K = 100）、K1000（K = 1000）各两点，读法同 Q1，`user_units` 两端相同、`looser=no`；总量约差十倍、改动 100 与 1000 个单元（占总量 1%–10%）。FTB 没有 `uK` | 同 Q1 | **行一**的「改动至少 1000 个单元、改动远小于总量」那一半就是这一行（K1000，`files_100_units_1000` 占总量约 1%）；**行二**同 Q1。**已定项 1**：`C1` 在这一档（第一至三段）预期不红，若真不红，第一至三段这一档的行一、行二不下结论，只报数；这一档的判由第四段 Q10 补 | 同 Q1 |
| Q3 改动随轴涨的读数（拆「随改动」与「随总量」） | 集合 D100、D10（两端 `user_units` 不同，`looser=yes`）：Q = 步数 ÷ `user_units`，x = 记录数，算 α(Q)，同时报原步数的 α；集合 T1e5（总量相近）：只报 `α_u(Q) = ln(Q_hi ÷ Q_lo) ÷ ln(u_hi ÷ u_lo)`（记录数几乎不变，按 x 的 α 没有定义，**不算**）。另报 2 × 2 因子表：K100 / K1000 两个集合各自的 α(x)、D100 / D10 各自的 α(Q)、T1e5 的 α_u | 档同读法 4；T1e5 的 α_u 只报数与档，不进行一、行二 | **行一**第二个观测（改动随轴涨那一种的每用户单元步数按记录数的增长阶）：D100、D10 都「不随」且 `C1`、`C3` 红 ⇒ 站住；任一「随」⇒ 行二翻面（这一格比原判据松，翻面的写「`looser=yes` 下仍随」）。判读线：见本节末 | 同 Q1；`looser=yes` 的格同时报原步数的 α（读法 5） |
| Q4 回扫读数（40 份 E206 产物，p2a 臂；**不是证实**） | E206 的轴（files、data_units、unlinks、unit_area）上：同一把尺（读法 3–7）；x 取记录数，另报旋钮 x；逐格同 Q1 的格；`data_units` 轴 W2 第 1 次标 `looser=yes` | 档同读法 4；另报「旧判据（旋钮 x、原步数）」的 α 与 `:706` 那一行逐字符核（第七节 ② 第 1 条）。**附带，够判后不跑**的是旋钮 x 与 `mapped_units` 的那两份；回扫本身每次必跑（不起模拟，只读产物） | 只用来：（a）钉读数器；（b）判 `C1`、`C3` 在 40 份产物上红不红（第五节作废条款按段判，回扫段与新点段分开）；回扫上的「随」不进行二的判定（判据是看过这批产物才写的，判决第一节） | 40 份只读，秒级 |
| Q5 控制臂红不红 | 第五节每条控制臂，对 Q1–Q4 的每一格（被测臂判不随或介于的那些）用同一把尺判一档，逐格列：控制臂的 α、档；汇总（段 × 档）里红的格数 ÷ 应判的格数 | 要求：应判的格**全部**判「随」。`C2` 只报 | **行一、行二的够判条件**（岔路单第 1、2 行）：对照不红则该（段 × 档）作废（作废条款第 1、2 条）。翻面取值：任何一格应判而没红 | 控制臂的步数是被测臂的步数加解析式，同分辨率；第四节第 12 条列了预期（`u1` 档红、`uK` 档 `C1` 不红） |
| Q6 尖峰旗 | 读法 8：每个点、每个状态逐操作算 `final_nodes` 的中位数与旗；列旗上的操作（`op`、`op_index`、`final_nodes`、与中位数之比）；`all` 与 `steady` 两份 Q1 / Q2 的档并列 | 旗上操作的个数、落在哪个操作上、`all` 与 `steady` 的档是否不同，逐点逐状态照报，不判 | **行三**的第一问「尖峰那次发布重写的角色由什么决定」的入口，以及「只有量窗第一次覆盖写随总量涨」这一句：只有 `first` 的 `all` 格「随」而 `steady` 格与 `later_*` 都不随 ⇒ 这一句成立；`later_*` 的 `steady` 格也「随」⇒ 不成立（那是行二） | 整数 |
| Q7 尖峰成因读数 | 对每个旗上操作取：首次合并站点（`release` 或 `placement`）、`seed_nodes_first_merge`、`pulled_nodes_first_merge`、`pulled_age_min / median / max`、`release_chain_depth`、`rounds`；H1 = 「首次合并走在释放站点，新并进的节点占 `final_nodes` 的 80% 以上，且新并进节点的年龄中位数 ≥ 100 个 `settle_call`」 | 逐旗上操作判 H1 成立 / 不成立，列哪一条不满足。80%、100 取自第四节第 7、8 条的量级（`final_nodes` 约 7 对约 433；历史约 1043 次 `settle_call`），**知道这些数之后定的**，不是推出来的，主 agent 可以另判 | **行三**第一问：成因定到 `transaction.rs:13918` 的那次首次合并（站点 `release`）即 H1 成立；站点是 `placement`、或新并进占比不足、或年龄中位数 < 100 ⇒ H1 不成立，成因待另查，行三仍开 | 整数；年龄的单位是 `settle_call`（第三节第 4 组） |
| Q8 重现条件（探针） | 四个探针项（`standard`、`cold_first`、`create_first`、`young_first`）各列：`P1` 操作是否在旗上、`W1` 第 1 次是否在旗上、W1–W4 里第一个旗上操作、W5–W7 里的旗上操作及相邻两个旗上操作之间隔了几次发布；`remounted` 状态里 W1 第 1 次是否在旗上 | 判读线见本节末（H1 / H2 / H3 三种预期各对应的探针读数）。`standard` 项的 W1 第 1 次**必须**在旗上（复现钉，第七节 ② 第 3 条）；不在旗上，其余三项没有意义（停机条款第 3 条） | **行三**第二、三问（挂载之后第几次发布、隔多少发布重现、能不能在别的操作后重现） | 整数；每项一次历史加一次量窗 |
| Q9 FTB 对 `files_10000`（正推腿的判读线，**附带**） | `files_then_big_10000` 每个（状态、操作、次序）的 B8 步数 ÷ `files_10000` 同一（状态、操作、次序）的步数（后者取本次第一段在今天树上的产物）；取全部格的最大值 | 最大值 ≤ 2 记「不超」，> 2 记「超」，点名那一格。2 倍是正推腿的取法（第四节第 6 条），在 x 十倍时对应 α 约 0.30，所以 Q1 的 FTB 格另报 α 档；两样都报，不替它选 | 行二的旁证：「超」⇒ 总量翻一倍以上步数也翻（对 `u1` 档）。**附带，够判后不跑**：行一、行二已被 Q1 判出时不再看它 | 同 Q1 |
| Q10 大点集合 K1000L 的严格读数（第四段） | 集合 K1000L 三点（读法 6），同 Q1、Q2 的格：`u1` 档与 `uK` 档（K = 1000，`looser=no`）、两种挂载状态、`first` / `later_peak` / `later_median`、`all` / `steady`。判读用低端对大点的 α（活单元数差 100 倍）；{中点, 大点} 的 α 与两段局部斜率另报；每点报 `user_units ÷ mapped_units`（低端约 10%，中点约 1%，大点约 0.1%）与量到的 `allocation_records` 比、`mapped_units` 比；与第一段 K1000 同（状态、档、格）的 α 差只报。控制臂 `C1`、`C2`、`C3` 同尺同格 | 档同读法 4；判定逐格。判读线见本节末「大点结局」 | **行一**的「改动至少 1000 个单元、改动远小于总量」`uK` 档那一半（大点占 0.1%）与 `u1` 档；**行二**同 Q1：`later_*` 的 `steady` 格 α ≥ 0.5 且 `C1`、`C3` 在该格红 ⇒ 翻面。`C1` 在 `uK` 档贴线（已定项 1） | 整数步数；低端对大点 x 约差 100 倍，α 带边对应 Q 比 3.16 与 10，比第一段的十倍跨度松得多；内存与挂钟见第十四节 |

**判读线（跑前写死；两种结局各写怎么改结论）**

| 结局 | 观测 | 改什么结论 |
|---|---|---|
| 结局 S：随改动，不随总量 | Q2 的 K100、K1000 两个集合 `steady` 格（`first`、`later_peak`、`later_median`，两种状态）全是「不随」，Q3 的 D100、D10 也「不随」，且 Q5 在这些格上红 | 行一在「改动 ≥ 1000 个单元、改动远小于总量」这一格站住。判决第二节「fs-design 那句」一格里「D8 已定项 16 的『记录总数』要明写『同样的用户改动』」那条建议得到样本外支持；B8 在新判据下的剩余问题只剩尖峰（行三）。不改 E203 页的判，由主 agent 另报 |
| 结局 T：随总量 | Q2 任一集合任一 `later_peak` 或 `later_median` 的 `steady` 格 α ≥ 0.5，且 Q5 在该格红 | 行二翻面：步数不只第一次发布随总量涨，B8 的「只随这一次碰到的单元」在改动 ≥ 1000 个单元、改动远小于总量的格上不成立；fs-design 第一格的「代价不许随盘容量增长」在这一格被打中；D8 已定项 16 射程「记录总数」的措辞不够（单次绝对代价任何比值都罩不住，判决第二节同格） |
| 介于 | α 落在 0.25–0.5，或 Q5 在 `uK` 档不红（已定项 1：第一至三段预期如此；第四段若不红，该段 `uK` 档作废） | 两行都不翻，如实报，行一、行二在那一格保持「开着」；第四段 `uK` 档仍不红则交主 agent 定是否另立登记改强 `C1` 系数 |
| H1（行三） | Q7 对每个旗上操作 H1 成立；Q8：`cold_first` 里 `P1` 在旗上、`W1` 第 1 次不在旗上；`create_first` 与 `young_first` 里 `P1` 不在旗上、`W1` 第 1 次在旗上 | 成因 = 第一次碰到「很久没重写的叶」时的释放链闭包（站点 `transaction.rs:13918`）；重现条件 = 每次新碰到一片陈旧区域；`cold_first` 里「`W1` 不在旗上」表示同一条陈旧链被 `P1` 一次带出。尖峰「另算」成立，行三答 |
| H2（行三） | Q8：三个探针里 `P1` 都在旗上、其后的 `W1` 第 1 次都不在旗上 | 成因 = 量窗第一次操作（与操作种类无关）：状态相关，不是释放落点相关。H1 的站点结论保留与否看 Q7，行三另立 |
| H3（行三） | Q8：三个探针里 `P1` 都不在旗上、`W1` 第 1 次都在旗上 | 成因 = 与第一个文件的旧落点有关，与「第一次」无关；`cold_first` 里冷文件的闭包没有带出陈旧链，H1 的「一次带出整条链」不成立，行三另立 |
| 都不是 | 上面三种以外 | 记「不是这三种」，列探针读数；行三仍开 |

**大点结局（只对 Q10，第四段；与上表的 S、T 同一把尺，跑前写死）**

| 结局 | 观测 | 改什么结论 |
|---|---|---|
| 大点 S | Q10 `uK` 档与 `u1` 档的 `steady` 格（`first`、`later_peak`、`later_median`，两种状态）低端对大点 α 全 < 0.25，`C1`、`C3` 在这些格红 | 行一在「改动 1000 个单元、总量跨 100 倍、改动占大点 0.1%」这一格站住；第一至三段 `uK` 档只报的那一半由它补上 |
| 大点 T | Q10 任一 `later_peak` 或 `later_median` 的 `steady` 格低端对大点 α ≥ 0.5，且 `C1`、`C3` 在该格红 | 行二在 `uK` 档翻面，总量跨 100 倍步数仍随总量涨 |
| 随跨度 | 低端对大点 α ≥ 0.5 而 {中点, 大点} α < 0.25，或相反 | 记「增长阶随总量区间变」，两个 α 都报；不判站住也不判翻面，行一、行二该格开着，交主 agent |
| 大点作废 | `C1` 或 `C3` 在 Q10 应判格不红 | 该档按作废条款 1、2 只报数；`C1` 在 `uK` 档不红是贴线情形（已定项 1），不触发 F5，写 F10 |

## 七、钉绝对值的断言

每条互比（控制臂对被测臂、新点对回扫、`first` 对 `later`）旁边至少有一条钉绝对值的；要是被测臂和三条控制臂一起错（比如读数器把操作之前的记录数读成操作之后的），谁会发现：下面 ② 第 1–3 条（读数器对已落盘产物的复现）与 ③（独立算出的式子）。

### ① 出自被测条款与源码自述的（不符走第十节「条款可能错」，不作废）

1. `ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT = 64`（`transaction.rs:13994`）、每叶 812（`crates/singlefs-format/src/lib.rs:163`）、内部扇出 169（`:168`）：产物头打实现的值，模型本地常量，单测比；不符走停机条款第 2 条（常量是装置读的，不是条款）。
2. 重写集只增不减（`transaction.rs:13812` 的函数文档）：用例里断言每个固定点的 `rewritten_size_r` 不小于上一轮；不符走失败条款 F4。
3. 定下的那一轮 `release_dirty_new_*` 与 `placement_dirty_new_*` 为 0、`holding_differs = 0`（`transaction.rs:13936`–`:13942` 的判）：用例里断言。
4. D8 已定项 16 射程那一句与 fs-design 第一格：条款本身没有可钉的数字（「记录总数」「代价不许随盘容量增长」都是性质句），这一类锚点不适用；性质句由第六节 Q1–Q3 的 α 档读，不钉绝对值。

### ② 取已有产物或实现值的（不符走停机条款第 3 条，两边都查）

1. **回扫读数器钉**：模型对 E206 的 40 份产物里 p2a 的那些，按**旧判据**（旋钮 x、原步数）重算 `q8_peak` 的 `data_units` 轴，得到的整行与 `research/results/e206-fixed-point-cost-report-2026-11-09.out:706` 逐字符相同；`q8_first_publish` 的 `files` 轴（`arm=p2a axis=files low_end=2102 high_end=15440 alpha=0.289 band=between local_slopes=0.045,0.295,0.526`，同文件 `:695`）也逐字符相同。这两行是这一次许读的 E206 合并报告的行；别的合并报告行不读。
2. **今天的树对 E206 的 p2a 产物（逐操作）**：`base`、`files_1000`、`files_10000` 三个对账点上，量窗每个操作（两种状态）的 `rounds`、`final_roles`、`final_nodes`、`round_work`、`final_set_hash` 与 E206 的 `*-p2a-*.out` 对应行逐个相同。今天的树与 E206 登记时的树有差（第三节：`transaction.rs`、`allocator.rs` 的 sha256 都变了），不等时先比这段历史两边的第一处不同，不许默认哪一边对。
3. **尖峰复现钉**：`files_1000_units_100`（`standard`）同一次挂载 W1 第 1 次：`rounds = 3`、`final_roles = 452`、`final_nodes = 433`、`round_work = 917`、`closure_nodes_added = 410`；`state` 行 `mapped_units = 101581`、`allocation_records = 238324`；W1 第 2 次：`rounds = 2`、`final_roles = 20`、`final_nodes = 7`、`round_work = 32`（第四节第 8 条；来源 `…files_1000_units_100-p2a-2026-11-09.out:520`、`:3485` 及邻行）。读数器复现不了这几个数，走停机条款第 3 条。
4. **`user_units` 声明与实现的差**（独立算的与量的比，因子 2 的容差）：新点上 `same_mount` 与 `remounted` 里每个第一次碰、仍是 K 个单元的 W2 操作，驱动声明的 `old_units − new_units` 与相邻两个 `name=state` 行的 `mapped_units` 之差（操作之前减操作之后）之比落在 [0.5, 2]。因子 2 是因为 `mapped_units` 是整个文件版本映射的单元数，含元数据角色；它不是一个精确的相等。比值在区间外⇒停机条款第 3 条：声明记账与实现的单元数对不上，两边查（驱动的 `units_of` 记账；`mapped_units` 的口径 `e206_fixed_point_cost_histories.rs:1120` 的 `version.mapped_units.len()`）。
5. **plain 与带计数的构建同盘**：`base`、`files_1000` 两点上 `image_sha256`（历史末）两种构建相同。不同⇒全部量的数作废（作废条款第 3 条）。
6. **大点单元数钉**（独立算的：F × K 加元数据约 1.6%，来自第四节第 8 条的 101581 ÷ 10⁵）：第四段历史末 `name=state` 的 `mapped_units` 落在：`files_1000_units_1000` 在 [10⁶, 1.05 × 10⁶]；`files_100_units_1000_unit_area_2_pow_21` 在 [10⁵, 1.1 × 10⁵]；`files_10_units_1000_unit_area_2_pow_21` 在 [10⁴, 1.2 × 10⁴]（低端元数据占比大，取宽）。区间外 ⇒ 停机条款 4：驱动的 `units_of` 记账与实现的单元数两边查。

### ③ 独立算出、用命令核过的（不符先对读法与式子，两边各查；属第十一节作废的走作废）

命令：`awk -f $D/calc/arith.awk`、`awk -f $D/calc/arith2.awk`（输出 `arith.out`、`arith2.out`，第十三节）。模型单测钉下面这些（输入写进测试里，不读产物）：

| 输入 | 期望 | 命令核过的来源 |
|---|---|---|
| 两点 Q 比 1.778、x 比 10 | α = 0.25（刚好在「不随」边） | `arith.out` 第 1 行 |
| 两点 Q 比 3.162、x 比 10 | α = 0.5（刚好在「随」边：`α ≥ 0.5`） | 同上 |
| 步数 2102 → 168373，记录数 922 → 238324 | α = 0.789 | `arith.out` 末段 |
| 步数恒为常数 | α = 0，档「不随」 | 手推 |
| 步数 = 100 × `user_units`，记录数不变 | 步数 ÷ `user_units` 的 α = 0，原步数对 `user_units` 的 α_u = 1 | 手推 |
| 读法 5：两端 `user_units` 相同（1 对 1）；两端 1 对 10 | 前者读原步数、`looser=no`；后者读步数 ÷ `user_units`、`looser=yes` | 手推 |
| `C1` 在 `u1` 的构造输入（5500 步 / 239228 记录，4500 步 / 23923 记录） | `C1` 的 α = 0.935，真实臂同格 0.087 | `arith.out` |
| `C1` 在 `uK` 的构造输入（每单元 178.4，总量差十倍） | α = 0.315（介于）；系数 4 时 0.617 | `arith.out` |
| `C3` 在构造输入（calls = 4 × 32，记录 239228 与 23923） | α = 1.107 | `arith2.out` |
| 尖峰旗：一串 `final_nodes` = 7、7、7、433、7 | 中位数 7，旗线 max(28, 28) = 28，433 在旗上，其余不在 | 手推 |
| 2 × 2 因子表：构造 `R = c × u^b × x^a`（a = 0、b = 0） | 四个集合的 α 全 0 | 手推 |

## 八、轨迹与几何敏感性

**轨迹**（被谓词消费的量：`final_nodes` 被尖峰旗吃、B8 步数被 α 的带吃、`pulled_age` 被 H1 吃）。每个点、每种挂载状态、每个 `user_units` 档报：峰值与它落在哪个操作上（`op`、`op_index`）；旗上操作的个数（为正的轮数）；期末值（`remounted` 的 W4 最后一次；带扩展窗口的点另报 W7 最后一次）。B8 步数与 `final_nodes` 另按操作次序**整列**打出（量窗每个操作一行，不只报峰值），因为 `first` / `later_*` 是对次序的读法。正文不写「从来」「恒」，除非三样都支持。

**局部斜率**：三点以上的轴（回扫的四根轴）相邻两点报 `ln(Q_{i+1} ÷ Q_i) ÷ ln(x_{i+1} ÷ x_i)`；新点集合只有两点，没有局部斜率，写明「两点，无局部斜率」。

**几何敏感性**（`.claude/rules/mutation-sampling.md` 第六类；每个旋钮逐个说）：

| 旋钮 | 取样点 | 方向为什么相反 | 判决那一格 |
|---|---|---|---|
| 单元区 U | K1000 集合换 U：`files_10_units_1000_unit_area_2_pow_17`（比基线小 4 倍）与 `files_100_units_1000_unit_area_2_pow_20`（比基线大 2 倍）；E206 回扫的 `unit_area` 轴（2¹⁴ → 2²⁴）已有 | U 小到 2¹⁷ 时全树节点上界少一个数量级（E206 登记第四节第 9 条：2¹⁷ 331 个节点，2¹⁹ 1305 个，2²⁰ 2605 个，算的），闭包可走的链短；U 大，链长上界大 | Q2 的 K1000 格在基线 U 与换 U 两套上各判一档；档不同记「不稳定（随 U）」，两档都报，该格行一、行二不下结论 |
| 挂载状态 | 每个点 `same_mount`、`remounted` | 重挂载之后聚簇段集合从空起；W2 池在两种状态里是不同的文件（第五节「量窗」第 2 条） | 每格两种状态各判，不同档记「不稳定（随挂载状态）」两边都报 |
| 总量涨法 | 同总量 10⁵ 单元三种涨法：`files_1000_units_100`（文件多、单文件小）、`files_100_units_1000`、`files_10_units_10000`（文件少、单文件大） | 同样的活单元数，记录数、节点数、单文件改动大小各不同 | T1e5 集合的 α_u；Q1 的 `u1` 档在三点上的原步数（同一类改动、总量近似，应当近似） |
| 总量跨度 | 第一段 K1000（十倍，U = 2¹⁹）对第四段 K1000L（低端对大点一百倍、{中点, 大点} 十倍，U = 2²¹） | 跨度小时对数比的分母小，一个整数步的差就能移动 α；跨度大时 `C1` 的解析项（对记录数 α 恰为 1）占比升高，判别力随跨度涨（已定项 1） | Q10；同（状态、档、格）在两个跨度上档不同记「不稳定（随跨度）」，两档都报；跨度与 U 同时变，不判差在谁 |
| 次序 | `files_then_big_*`（先小后大）对 `files_*`（先大后小） | 两种历史次序 | Q9 |
| 改动占总量的比 | K1000（约 1%）、T1e5 里的 `files_10_units_10000`（约 10%） | 改动大到占总量一成时「远小于总量」不再成立 | Q2 报每个点的 `user_units ÷ mapped_units`，占比 > 5% 的点在行一的「远小于总量」上不计 |

**判别力自证**（照第六类）：模型单测把档的边 0.25 挪到两个构造集合的 α 之间、把 `C1` 的系数挪到 0、把旗线挪到 433 之上，判定必须由绿转红（第九节 M4、M5、M8 各对应一条）。

## 九、变异

三张表：模型 `scripts/mutation/tables/e207_b8_strict_criterion.tsv`（经 `scripts/mutation/mutate.sh`）、测量补丁脚本 `scripts/experiment-apparatus/e207_b8_strict_counters.mutations.tsv`（经它的 `--mutation-check`）、历史用例 `scripts/mutation/tables/e207_b8_strict_histories.tsv`。**历史用例那张表要证红**：`mutate.sh` 在 `scripts/experiment-apparatus` 里只起 `cargo test --release --bin`，跑不到副本里的历史用例（E206 修订 1 记过这个洞），所以入口须带 `E207_RUN_BREAK=<项>` 弄坏开关把改坏的起点带进副本的构建，每条在下表的取样点上证红；做不到的行写明「未证红」，不记「抓到」。每条写明在哪个取样点上改变输出；抓不到的先按 `mutation-sampling.md` 三分。

| # | 表 | 改什么 | 在哪个取样点上改变输出 |
|---|---|---|---|
| M1 | 模型 | 严格读数的 x 换成旋钮（F、D、u、K） | 构造 D100 读数（记录数比 10、旋钮 K 比 10、旋钮 F 不变）与构造 K100 读数（记录数比 10、旋钮 F 比 10）：x 换成旋钮后 D100 的 α 与 K100 的 α 都变，且 D100 变成与 F 无关 |
| M2 | 模型 | 读法 5 的「两端 `user_units` 相同」判成「不同」 | 构造 1 对 1 与 1 对 10：`looser` 标记与读的量都反过来 |
| M3 | 模型 | 分母用 `Rs`（各轮释放之和）代替声明的 `user_units` | 构造 D100：`Rs` 与 `user_units` 不同的构造读数，Q 变 |
| M4 | 模型 | 档边 `α ≥ 0.5` 改成 `α > 0.5` | 构造 Q 比 3.162、x 比 10（α 刚好 0.5）：「随」变「介于」 |
| M5 | 模型 | `C1` 系数 1 改 0 | 构造 `u1`：`C1` 由「随」变「不随」，Q5 判不红 |
| M6 | 模型 | `C3` 不把 `lg(Z)` 换成 `Z`（保持原价） | 构造 `u1`：`C3` α 由约 1.1 降到被测臂水平 |
| M7 | 模型 | `C3` 选量窗合计**最小**的含 `lg` 的行 | 构造两个含 `lg` 行合计不同的点：被换线性的行变 |
| M8 | 模型 | 尖峰旗线去掉 28 的下限，只留 4 × 中位数 | 构造序列 7、7、7、7、7、7、7、14、433：14 在旗上变成不在（4 × 7 = 28 对 14） |
| M9 | 模型 | 尖峰旗用均值代替中位数 | 构造序列 7、7、7、7、7、7、7、433、60：中位数 7（线 28，433 与 60 都在旗上），均值 56（线 224，60 不在） |
| M10 | 模型 | `first` 取这一档里步数最大的一次，不取次序第一次 | 构造序列首次不是最大：`first` 与 `later_peak` 对调 |
| M11 | 模型 | `steady` 把旗上操作留下、把不在旗上的剔掉（写反） | 构造含一个旗上操作的序列：`steady` 档变 |
| M12 | 模型 | H1 的占比线 80% 改 90% | 构造 `pulled ÷ final_nodes` = 0.85：成立变不成立 |
| M13 | 历史用例 | `units_of[inode]` 覆盖写后不刷新成 1 | `files_10_units_1000` 的 `same_mount` W2 同一文件第二次取到：声明的 `user_units` 由 1 变 1000（`E207_RUN_BREAK=units_of_not_refreshed`） |
| M14 | 测量补丁脚本 | 节点年龄表在每一轮更新，而不是只在定下那一轮 | 假树上一个节点在非终轮出现、终轮没出现：`pulled_age` 变 |
| M15 | 模型 | 三点集合的局部斜率用低端对大点的端点斜率代替相邻两段斜率 | 构造三点序列 Q = 1、2、100（x = 1、10、100）：相邻两段斜率 0.30 与 1.70，端点斜率 1.0 |
| M16 | 模型 | 作废条款 1 的「段」合并：第四段与第一至三段的 `uK` 档 `C1` 合并判 | 构造第一至三段 `uK` 档 `C1` 不红、第四段 `uK` 档 `C1` 红：第四段 `uK` 档的行一、行二由「可判」变「作废」 |

入口与判定库另按 `.claude/rules/test-script-discipline.md`「测试脚本自己也要被测试证明有效」写弄坏开关，形态照 `e206-fixed-point-cost-run.sh` 文件头。

## 十、失败条款

每条紧跟「什么观测会让它触发」；打中之后先照 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」问那四句（分不分辨臂、系统看不看得见、满足的是哪一个分句、改法碰不碰得到）。

| # | 条款 | 什么观测会让它触发 | 记什么 |
|---|---|---|---|
| F1 | B8 行「只随这一次碰到的单元涨」在新点上也翻面 | Q1 或 Q2 的 `later_peak` 或 `later_median`，`steady`，任一状态，α ≥ 0.5，且 Q5 在该格红 | 行二翻面；列贡献最大的三行（`by_delta_rows` 的行名）与它们吃的输入；不把 B8 的做法改强了再判。注意：判决已记 B8 在原判据下输一次（第一节），F1 说的是新判据下的第二次 |
| F2 | 「站住」被尖峰打断 | Q1 或 Q2 的 `first` 的 `all` 格 α ≥ 0.5 而同点同状态的 `steady`、`later_*` 全不到 0.5 | 记「只有量窗第一次随总量涨」成立；行一不判「干净的站住」，也不判行二；转 Q6–Q8 |
| F3 | 「改动随轴涨」那一格靠除以 `user_units` 才站住（`looser` 的实际作用） | Q3 的 D100 或 D10（或 Q4 的 `data_units` 轴 W2 第 1 次）原步数 α ≥ 0.5 而步数 ÷ `user_units` 的 α ≤ 0.25 | 记「这一格比原判据松」的实例，点名格与两个 α；不当收严，也不撤回判决已定的写法 |
| F4 | 固定点的前提不成立 | 任一固定点 `outcome=DidNotSettle` 或 `rounds = 64`，或 `rewritten_size_r` 下降，或定下那一轮 `release_dirty_new_*` 不为 0 | 记 `transaction.rs:13812` 函数文档「走得完」的前提在产品路径上不成立；**当场写进报告交主 agent**，不等实验收尾。该点读数照报，但这一点 B8 读数不进行一的判定 |
| F5 | 控制臂的判别力推算（第四节第 12 条）不符 | `C1` 在 `u1` 档任一应判格不红，或 `C3` 在 `u1`、`uK` 任一应判格不红 | 记第 12 条的推算错（报实际 α）；同时按作废条款 1、2 作废那（段 × 档）。`C1` 在 `uK` 档不红是**已预告**的，不触发 F5，走未定项第 1 条 |
| F6 | FTB 的 2 倍线（正推腿的判读线）被打破 | Q9 最大比值 > 2 | 记「推翻」，点名格；不替它判行二，Q1 的 FTB 格另判 |
| F7 | 假设 H1 被推翻 | Q7 对任一旗上操作 H1 不成立，或 Q8 的三个探针读数落在判读线「H2」「H3」「都不是」 | 记 H1 不成立；报是哪一条不满足；行三仍开，成因另查（新实验，不在本登记里补） |
| F8 | 历史走不完或 K 单元的冷文件池不够 | 任一点 `name=history_refused`，或新点任一状态里「第一次碰、仍是 K 个单元」的 W2 操作少于 5 个 | 该点这一状态的 `uK` 档记「样本不足」，不判；`history_refused` 照报，不算撞上界 |
| F9 | 探针的前提或结局落在「都不是」 | Q8 判读线上四种结局都不满足 | 记「不是这三种前缀单独决定的」，列四个探针项的旗上操作清单；行三仍开 |
| F10 | 第四段的阳性对照贴线预告落空，或第四段跑不出 | `C1` 在 Q10 `uK` 档应判格不红（已定项 1 预告过，贴线）；或大点内存装不下、被停、`history_refused`、超时未交 | 前者：记 b 的实测值与 `C1` 的 α，第四段 `uK` 档作废只报数，`C2` 照报，交主 agent 定改强系数；后者：大点记「本次未跑」，另两个小点照报，第一至三段结论不受影响。`C3` 在 `u1`、`uK` 不红仍走 F5；`u1` 档 `C1` 不红走 F5 |

本实验不量挂钟。**什么读数会让这一句重开**：行一或行二被判到要拿秒数交用户的程度（比如 `uK` 档的绝对代价「单次绝对代价任何比值都罩不住」那一格，判决第二节「fs-design 那句」），那是新一次登记，不在这一次里补。

结果反过来接不接受（`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「不许先有结论再建模型」）：要是 Q1、Q2 的 `steady` 与 `later_*` 全不随（含 `uK` 档、含 `C1`、`C3` 都红），我接受行一站住（尖峰另算）；要是 Q2 的 `steady` `later_*` 随，我接受行二翻面；这时第四节第 2 条的回扫读数（0.147–0.385）不能当作「B8 随总量涨」的证据，它们只是被尖峰托高。要是 Q8 落在 H2 或 H3，我接受第四节第 8 条看起来像的「释放站点闭包」只是描述，不是成因。

## 十一、作废条款与停机条款

### 作废条款

1. `C1` 在某（段 × 档）里**应判的格**没有全部判「随」（段 ∈ {新点（第一至三段）, 大点（第四段）, 回扫}，档 ∈ {`u1`, `uK`}；三个段各判各的，不合并，第四段的 `C1` 红不救第一至三段的 `uK` 档，反之亦然）：该（段 × 档）里被测臂的 B8 读数对行一、行二整段作废，只报数（登记原话：阳性对照不红整段作废）。`uK` 档预期如此，见未定项第 1 条；`u1` 档如此另走 F5。
2. `C3` 同上。
3. `plain` 与带计数的构建在 `base`、`files_1000` 上历史末 `image_sha256` 不同：插计数改了行为，全部量的数作废。
4. 阴性对照过不了（`rounds > 4`）：该点读数作废，先查计数记错发布。
5. 读不到不算读到 0：缺 `name=item_done`、缺 `E207RESULT name=declared` 行、声明行数不等于量窗操作数、某轮读数缺字段，这个点这一臂作废，当本次未跑报。
6. 对账（第七节 ② 第 2 条）逐操作不符且查明是计数记错了的：该点读数作废；查明是今天的树行为变了的，走停机条款第 3 条。
7. 第四段：大点（或另两点）缺 `item_done`、声明行数不等于量窗操作数（160 次）、或没跑完：Q10 缺这一点，K1000L 当本次未跑，不拿两点算 α 冒充三点；只剩低端和中点时 Q10 只报 {低端, 中点}（总量十倍），不当作跨 100 倍。

### 停机条款（装置与 `crates/` 对不上：两边都查，既不作废也不当结果，`.claude/rules/implementation-first.md` 第 4 条）

1. 测量补丁脚本的锚点命中不是恰好一次（退 3）。**已知前件**：E206 补丁在今天的树上三处锚点对不上（第三节），E203 编辑表在今天的树上是否全命中没核。执行员先对今天的树跑 `--check-anchors`，把每个失败的锚点逐个列出，再按今天的 `merge_into_the_rewrite_set_walking_the_release_chain` 重写；改的是谁就对谁，核对 `crates/` 是不是又变了。
2. 模型式子与计数读数对不上（`W` 不等于 `Σ_r A_r`，产物头常量不等），或第五节「对照」走不到：停，逐项对。
3. 第七节 ② 不符（任何一条），或登记开头写的三个文件的 sha256 与产物头记的不符：停，先比差在哪；E206 登记时的树在 `git log` 里，取同一提交的树比；比完把差与复现结果写进报告，由主 agent 判。
4. `E207RESULT name=declared` 的 `user_units` 与相邻 `name=state` 的 `mapped_units` 差对不上（第七节 ② 第 4 条区间外），或第四段 `mapped_units` 落在第七节 ② 第 6 条区间外：停，两边查。

### 够判停机

岔路单每一行都够判就停，没跑的量逐条标「够判后未跑」。第二段、第三段、第四段不是计划外的必跑：每段交回附岔路表（已够判 / 还差什么 / 剩下的量能不能让它翻面），主 agent 判续不续。行三只有在第一段之后仍开着才跑第二段。第四段是用户定的，列入计划，仍由主 agent 在第一段交回后对着岔路单判起不起；大点没跑完不挡第一至三段出结论。

## 十二、修订

设计员修订 1（登记写后、装置写前、产物一次没跑；依据：用户 2026-11-09 定「加大点，分段跑」，`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「跑产物之前修订臂或判据，是合法的，但要留下记录」）：
- 原未定项第 1 条（`C1` 在 `uK` 档判别力不足，三选项 (a)(b)(c)）改成已定项 1，选 (c)；原文的推算（K = 1000 的 K1000 集合 `C1` 的 α 约 0.32、要 α ≥ 0.5 系数至少约 2.5）保留在已定项 1 的句子里，没有改。
- 加三个点（`files_1000_units_1000` 与同 U = 2²¹ 的低端、中点）、集合 K1000L、量 Q10、「大点结局」判读线、第七节 ② 第 6 条、变异 M15、M16、失败条款 F10、作废条款 7；作废条款 1 改成按（段 × 档）三段分别判；原第三段里的 `files_1000_units_1000` 移出，第三段只剩 `files_30000`。
- 新增第四段与段间依赖表；第一至三段的臂、判据、门槛、判读线一个字没改（只收严，不放松）。
- 为什么加低端、中点同取 U = 2²¹：K1000L 两端 U 相同，U 不混进 α（读法 6 同样处理第一段各集合）。

执行员修订 1（2026-11-11，装置写完、单测与产物一次没跑；只补取样点，不动原判据与原取样点）：
- 第九节 M8 的取样点补一个敏感的。原取样点「7、7、7、7、7、7、7、14、433」照留；它在原式与变异式上同值（中位数 7，4 × 7 = 28 与下限 28 相等，14 两边都不在旗上），按 `.claude/rules/mutation-sampling.md`「没红的三类，判据不同」第三类不当等价留档。补的取样点：「3、3、3、3、20」——原式旗线 max(28, 12) = 28、20 不在旗上；去掉下限后旗线 12、20 在旗上。依据是读法第 8 条的式子本身（手推），不是单测读数；落在模型单测 `the_spike_line_takes_the_floor_and_four_times_the_lower_median`。

执行员修订 2（2026-11-11，装置补齐用例之时，单测与产物一次没跑；判据、门槛、取样点与第六、七、九节各条的去留与报告方式一个字没动；两件都是主 agent 在派发提示里定的，依据照抄那一句）：
- **停机条款第 3 条的前件（第 7 行记的 sha256 与今天不同）怎么办**。依据：派发提示「主 agent 判：以今天主工作区为被测的树，差异来自之后合入的产品改动（里程碑四收口第二批，判决 research/prompts/m4-close-batch2-code-r1/r2/r3-main-verification.md 点名的文件），不比旧版；产物头照记今天三份文件的 sha256。」照此：被测的树是跑产物那一刻的主工作区；第 7 行记的 `transaction.rs`、`allocator.rs` 两个 sha256 与它不同，不当停机条款第 3 条的触发，不去比登记时那一版（git 里取不到）；`research/prompts/m4-close-batch2-code-r1-main-verification.md` 第 51、55 行与 `research/prompts/m4-close-batch2-code-r2-main-verification.md` 第 44、49 行点名的就是这两份文件。产物头 `name=source_sha256` 三行照记跑的那一刻三份文件的 sha256（入口 `header_for`），读产物的执行员把它们与本条执行员报告里记的今天的值并列报。停机条款第 3 条的其余前件（第七节 ② 任何一条不符）照原文。`settle_the_allocation_record_tree` 与 `merge_into_the_rewrite_set_walking_the_release_chain` 两个函数体与第三节的描述逐项比过，结果写在本条执行员报告里，不在这里转述。
- **上一段执行员自取的五处读法，主 agent 认可**。依据：派发提示「上一段自取的五处读法（W3、W6 的 user_units 取各次之和；P1 前缀与 W5–W7 只加在同一次挂载那一段；K1000 换 U 集合两端 U 不同；回扫时 E206 产物按历史构造补 user_units；C3 的 1.107 高端 calls 取 4×32、低端取 4×25）主 agent 认可，一并写进修订 2。」逐条：① 一次操作含几次用户改动时 `user_units` 取各次之和（W3 一次建—写—删记 3、不进 `u1` 档；W6 一批建 10 个记 10）；② 探针前缀 P1 与扩展窗口 W5–W7 只加在同一次挂载那一段，重挂载那一段不加；③ 几何敏感性里 K1000 换 U 的集合取 {`files_10_units_1000_unit_area_2_pow_17`, `files_100_units_1000_unit_area_2_pow_20`}，两端 U 不同（第八节给的就是这两个点），读数照第一节读法 4 报，不另加 U 的校正；④ 回扫的 E206 产物没有声明行，`user_units` 按 E206 历史的构造补（W1、W4 与阴性对照记 1，W3 记 3，W2 第一次碰到一个文件记它建时的单元数、之后记 1）；⑤ 第七节 ③ 表 `C3` 那一行 α = 1.107 的构造输入是高端 calls = 4 × 32、低端 calls = 4 × 25（登记那一格只写了高端）。

执行员修订 3（2026-11-11，单测与产物一次没跑；只改名字，判据、门槛、取样点、臂与第六、七、九节各条的去留与报告方式一个字没动）：
- **英文名由 `b8_strict_criterion` 改成 `strict_scan_steps`**（第一节「英文名：」那一行原样留着）。依据：派发提示「文件名 e207_b8_strict_* 里的「b8」同样判红：照上一段同样的做法定新文件名」与「E206 的文件名 e206_fixed_point_cost_* 不带这类记号，跟它对齐」；共享命名纪律脚本（任务 13300）把文件名里的 `b8` 判成单字母加数字。新名说的是被量的东西：发布准入之前扫类步数（B8 那一行）在严格判据下的读数。
- 跟着改的名字（第五节「装置写在哪」、第十四节「跑法」里写的旧名照这一条读）：模型 `scripts/experiment-apparatus/e207_strict_scan_steps.rs`（bin `e207-strict-scan-steps`，与 E206 的 `e206_fixed_point_cost.rs` 同形）；历史用例 `e207_strict_scan_steps_histories.rs`（副本里 `crates/singlefs-verification/tests/e207_strict_scan_steps_histories.rs`，`cargo test --test e207_strict_scan_steps_histories`）；量窗取法 `e207_strict_scan_steps_window_choices.rs`（bin `e207-strict-scan-steps-window-choices`）；插计数脚本 `e207_strict_scan_steps_counters.py` 与它的变异表 `e207_strict_scan_steps_counters.mutations.tsv`；判定库脚本 `e207_strict_scan_steps_library.py`；入口 `e207-strict-scan-steps-run.sh`；变异表 `scripts/mutation/tables/e207_strict_scan_steps.tsv`、`e207_strict_scan_steps_histories.tsv`、`e207_strict_scan_steps_window_choices.tsv`；判定库种类 `e207-strict-scan-steps-run`；副本里读数模块 `e207_strict_scan_steps`；产物 `research/results/e207-strict-scan-steps-<点>-<臂>-<日期>.out` 与 `e207-strict-scan-steps-counters-diff-<日期>.diff`（产物名照英文名，执行员定义第 4 步）。改名之前一份产物都没有（`ls research/results | grep -i e207` 空），没有旧产物要对。
- 装置里带登记量号的声明名（`H1_*`、`Q9_*`、`C1_*`、`C2_*` 常量，`report_q7` 这类函数，`f3_`、`q9_`、`h1_` 打头的单测名）一并改成说出那个量的名字；登记量号只留在注释与报告行的字段值里（`quantity=q9`、`name=q7_h1` 这类照旧）。模型与历史用例打的报告行，键与值一个没改，读产物的写法不变；只有入口 `--break-history`（M13 证红）那一行的计数键由 `w2_same_mount_declared_at_k` 改成 `same_mount_cold_overwrite_declared_at_a_thousand`，判 `caught=` 的条件不变。

## 十三、读过的文件与跑过的命令

### 读过的文件（行区间；grep 命中行也列）

- `.claude/agent-common.md` 第 1–127 行（第 37–127 行读的是截断到每行 500 字的版本）；`.claude/singlefs-ai-sop/rules/test-discipline.md` 第 1–138 行；`.claude/rules/implementation-first.md` 第 1–19 行；`.claude/rules/mutation-sampling.md` 第 1–93 行；`.claude/singlefs-ai-sop/rules/evidence-discipline.md` 第 100–125 行与标题行（其余没读）；`.claude/rules/test-script-discipline.md` 标题行与第 77–89 行、`.claude/rules/three-way-inference.md` 第 140–154 行（其余没读，岔路单的格式照 E206 登记与本次岔路单文件）。
- `research/prompts/e203-b8-criterion-r1-forks.md` 第 1–11 行；`research/prompts/e203-b8-criterion-r1-main-verification.md` 第 1–55 行；`research/prompts/e203-b8-criterion-r1-opus-output.md` 第 1–260 行（其余的第 260–367 行没读：fs-design 那句、第三条路、腿的限度；标题行读了）；`research/prompts/e203-b8-criterion-r1-sonnet-output.md` 第 1–30、66–88、137–177 行（腿的读数，线索，列在第四节）。
- `research/prompts/e206-preregistration.md` 第 1–50、175–590 行（格式与 E206 的读数定义；第 43–175 行是被测条款的抄录，没读）；`research/prompts/e203-preregistration.md` 第 608–625 行。
- **结果产物**（派发提示点名的行之外多读的，已列在第四节第 7、8 条）：`research/results/e206-fixed-point-cost-report-2026-11-09.out` 第 695、706 行与 `grep -c '^name=q8'`；`research/results/e206-fixed-point-cost-files_1000_units_100-p2a-2026-11-09.out` 第 500–540、3484–3485 行（每行截到 500–700 字）；`ls research/results | grep e206`（只看文件名）。没读：`.claude/kb/experiments/` 下各实验页、`research/results/` 的其余。
- `.claude/kb/decisions/08-核心索引结构.md` 第 503–520 行与 516 行 `grep -o`；`.claude/rules/fs-design.md` 第 17–30 行；`crates/singlefs-core/src/mounted_session.rs` 第 165–176 行；`crates/singlefs-core/src/transaction.rs` 第 2000–2050、13795–14010 行与 `grep -n` 命中行（`settle_the_allocation_record_tree`、`merge_into_the_rewrite_set_walking_the_release_chain`、`publish_overwrite_of_an_inode`、`changed_by_the_release`、`holding_records`）；`crates/singlefs-core/src/allocation_record_tree.rs` 第 186–200 行与 `grep -n` 命中行；`crates/singlefs-core/src/mount.rs` 的 `grep -n PUBLISHES_PER_ADMISSION_AT_MOST`；`crates/singlefs-format/src/lib.rs` 第 162–172 行。
- `scripts/experiment-apparatus/e206_fixed_point_cost_histories.rs` 第 54–260、295–470、520–760 行与 `grep -n` 命中行；`scripts/experiment-apparatus/e206_fixed_point_cost.rs` 第 245–445 行与 `grep -n '^fn '` 命中行；`scripts/experiment-apparatus/e206_fixed_point_cost_counters.py` 第 1–120、700–770 行与 `grep -n` 命中行；`scripts/kb/replace-once.py`、`scripts/kb/quote-kb.py` 的文件头；`.claude/heavy-thread-default`；`scripts/kb/claim-experiment.sh` 的输出。

### 跑过的命令（原样；`$D` = `/home/fy5090/.local/share/singlefs/work/e207-b8-strict-designer`，在仓根起）

```text
bash scripts/hosts/work-directory-ownership.sh claim $D --purpose 'B8 严格判据跑前登记的草稿'
bash scripts/kb/claim-experiment.sh --next                        # E207
bash scripts/kb/claim-experiment.sh E207 B8严格判据下的步数
python3 scripts/kb/quote-kb.py $D/quoted-clauses.md '.claude/rules/fs-design.md@## 记账是事务的副产品，不是事后的遍历' '.claude/kb/decisions/08-核心索引结构.md@#### 已定项 16：第一版 unlink：只立 deleted_inodes、从尾部分批、挂载里续做到空'
awk -f $D/calc/arith.awk > $D/calc/arith.out        # 第四节第 9、12、13 条，第七节 ③
awk -f $D/calc/arith2.awk > $D/calc/arith2.out      # 同上
grep -cF '<锚点>' crates/singlefs-core/src/transaction.rs   # 第三节，九条锚点各一次
grep -rln 'by_delta\|差量视图\|DeltaView' crates/              # 零命中
sha256sum research/prompts/e203-b8-criterion-r1-forks.md research/prompts/e203-b8-criterion-r1-main-verification.md scripts/experiment-apparatus/e206_fixed_point_cost.rs scripts/experiment-apparatus/e206_fixed_point_cost_histories.rs scripts/experiment-apparatus/e206_fixed_point_cost_counters.py crates/singlefs-core/src/transaction.rs crates/singlefs-core/src/allocation_record_tree.rs crates/singlefs-core/src/allocator.rs
```

（设计员修订 1 追加，`$D2` = `/home/fy5090/.local/share/singlefs/work/e207-b8-strict-designer-2`：`bash scripts/hosts/work-directory-ownership.sh claim $D2 --purpose 'E207 登记加大点与分段的草稿'`；`awk -f $D2/calc/big.awk > $D2/calc/big.out`（已定项 1 与第四节新增推算的来源；大点 `C1`、`C2` 的 α、b 分界）；`free -g`、`nproc`（大点可达核过 ③）；`grep -n 'remount\|clone()\|image_sha256\|SparseBlockDevice' scripts/experiment-apparatus/e206_fixed_point_cost_histories.rs`；`sed -n` 读 `e206_fixed_point_cost_histories.rs` 第 88–130 行的点表与 `scripts/kb/replace-batch.py`、`replace-once.py` 的文件头。没读 `research/results/` 与 `.claude/kb/experiments/`。）

（另有几条只读的 `grep -n`、`sed -n`、`wc -l`、`ls`，命中行都列在上一小节。`--check-anchors` 我起不了（子 agent 不往统一任务服务交任务，直接跑项目脚本被钩子拒），第三节的锚点数是 `grep -cF` 对整份 `transaction.rs` 的，函数体内恰好一次由执行员现核。）

sha256（登记写时）：`e203-b8-criterion-r1-forks.md` `f84adae71af4d323286155d4aef19fe770297cedc2a8bd386110144e85212c38`；`e203-b8-criterion-r1-main-verification.md` `7ef803c188ffb604d9ed6a010f2cb9fbc2df38070fc057e59a51546b0ff26d07`；`e206_fixed_point_cost.rs` `d2da349793da1838d3f34ef6f2b090f4923e97e91ea1c598f6336f87612e5ac9`；`e206_fixed_point_cost_histories.rs` `8f71c2a11e64e1f55b89436d7f7024d63e9d5b383f3a7b17b697dcc9101474f7`；`e206_fixed_point_cost_counters.py` `90628c15a38b37615e19882fe2ee1ad7444f220844071bc580cddc7cf149c768`。

执行员修订 4（2026-11-12，第一段已出 11 项产物之后；本修订所涉的分片跑法产物一次没跑；读数口径、判据、门槛、取样点、臂与第六、七、九节各条的去留与报告方式一个字没动）：
- **跑法改成分片**。依据：派发提示「用户原话：『完成后不管撞不撞，这里都要拆。 拆了分片。』」与「一个取样点一条任务跑不进四小时」——任务 13894（`files_1000_units_100__head`）累计 14421 秒被服务按四小时硬上限停在 `publishes_done=993`，同一段别的项也有近两小时的（任务 13899 用了 6767 秒）。入口 `e207-strict-scan-steps-run.sh` 文件头加 `# task-server-shards: *`，经服务交的每一种跑法拆成计划、各片、合并三步（片数、哪台、上限由服务定）。第十四节「切分与续跑」一段说的「单元内部切不开」在这一点上被改：一个单位的历史按发布区间切开，历史前缀靠每片重放。入口参数、产物名、产物形、判定库的键与复用、每趟抽的复用哨兵都不变；直接跑（不经服务）仍整项整趟。
- **切片法与读数口径**。一片管历史里用户改动序号 `(from, to]`（序号 = `History::note` 被叫的次数 = 产物 `name=history` 行的 `user_changes`；第 i 片共 K 片：`from = i×N÷K`、`to = (i+1)×N÷K`，`N = 2 + 2×文件数 + 2×unlink 数`，单测钉在已有产物的 `user_changes` 上：42、222、2022、20022）。前面的发布怎么到位：每片从头把历史放一遍，序号在 `from` 及以前的读数照常取走、不打——读数模块的状态（固定点次数、节点年龄表、直方图的桶）与不分片时逐步相同；区间里的读数照打，与不分片时同一行；非最后一片到 `to` 打片尾标记收尾，最后一片放完历史、做量窗与重挂载。为什么读数逐行相同：每片跑同一段确定的程序，只是打印按序号过滤。合并核账（区间首尾相接不重不漏、固定点次数首尾相接、各片发布数之和 = 总数 = `name=history` 行的 `user_changes`、最后一片 `item_done`）不过判红、不出产物；过了按片的次序拼成与不分片时同名同形的一份（末行 `item_done`）。
- **它的代价（不是读数）**。放前缀不比量快多少，所以第 i 片的挂钟约等于不分片时放到序号 `to` 的那一段，最后一片仍要放完整段历史。分片换来的是前面几片短、各片被打断后只补没判的片，不是最后一片也短；历史本身超过四小时的项（`files_1000_units_100`、第四段 `files_1000_units_1000`），最后一片仍可能被四小时上限停。要让最后一片也短，得把前缀的盘镜像、驱动记账与读数模块状态落盘再续接，本修订不做（读数模块的年龄表等状态要一并存读才能保证逐行相同）。
- **被测的树**。取派发提示点名的 `/home/fy5090/.local/share/singlefs/work/e207-b8-strict-runner-4/source-before-the-precheck-patch`，含 D3 回落政策那份补丁；三份核心文件的 sha256 在已有产物头 `name=source_sha256` 三行：`transaction.rs` `8bfdc6628feb25090f53417bba9fc7652e7736ec6ade72941440085d66c69a93`、`allocation_record_tree.rs` `991623b6a9b5028eb692b8e88a3e6acaab6c1471c5302b23dffeb6f42ea958ce`、`allocator.rs` `329eb0744cfab3929e0feb2c0a2f9b364b9cfda199d4e1a210d0a5a41d99da4a`（`grep -h name=source_sha256 research/results/e207-strict-scan-steps-*-head-2026-11-12*.out | sort | uniq -c` 数出 13 份产物里 10 份是这个 `allocator.rs`，3 份是 `062f2d04…`，见报告）。主工作区今天的 `allocator.rs`（sha256 `062f2d04004a2fbfe64b2525a670ba4265f1a14a067aadd57309821ddd8c3990`，与 `transaction.rs`、`allocation_record_tree.rs` 今天的 sha256 同上）把 E203 编辑表里 `s14_cluster_segment_sum` 的锚点所在的那一行 `let pairs_inside_the_cluster_segments: u64 = cluster_segments` 抽成了函数 `pairs_inside_the_cluster_segments(...)`（`grep -c` 该行：今天的 `allocator.rs` 0 次，源树那份 1 次），而插计数脚本要求每个锚点在它的函数体里恰好命中一次、不然退 3（停机条款第 1 条）；在今天的树上跑之前这个锚点要改写成插在那个新函数里（本修订不改，所以被测的树仍取上面那份）。

执行员修订 5（2026-11-12，被测的分配器与固定点换了实现之后、新树上第一段一项也没重量之前；读数口径只收严，判据、门槛、取样点、臂与第六、七、九节各条的去留与报告方式一个字没动；三件都是主 agent 在派发提示里定的，依据照抄那几句）：
- **为什么**：被测的产品代码换了实现。三份文件现在的 sha256 前 16 位：`crates/singlefs-core/src/allocator.rs` 61ac7a8f38b4b78d、`crates/singlefs-core/src/transaction.rs` d6b2083d2d950d7f、`crates/singlefs-core/src/admission.rs` 301ffb704c7c9f57；修订 4 记的第一段产物头旧值是 `transaction.rs` 8bfdc6628feb2509、`allocation_record_tree.rs` 991623b6a9b50282、`allocator.rs` 329eb0744cfab392。新实现里用户数据落点按「还有没挡偶数槽对的聚簇段」集合升序走、段内逐对看；每块盘的空闲图多了四个随整份拷贝一起克隆的增量集合；提交内生块回落在每块盘各答一遍之后，池里再按分配记录树的叶取落点；固定点并重写集改走 `merge_into_the_rewrite_set_walking_the_release_chain`、循环变量改名 `round_index`。第一段旧产物（`research/results/` 下 `e207-strict-scan-steps-*-head-2026-11-12*.out`）量在旧分配器上，被量的代码变了，作废、不引；这不是判据打中之后改口径：新树上第一段一项也还没重量，旧产物上也没有哪一条判据被拿来改口径。
- **主 agent 定的（派发提示原文）**：
  - 「s26 的步：认上一段的读法——走到集合里一段记 1 步、答案段内每看一对记 1 步，两条都留；上界「集合段数 + 32」。」
  - 「s1 / s24 整份拷贝元素数：把 `DeviceFreeMap` 今天会随整份拷贝克隆的四个集合（`is_under_an_unreleased_record`、`slot_pairs_without_an_unreleased_record_per_segment`、`segments_with_an_unblocked_slot_pair`、`unblocked_slots_per_allocation_record_tree_leaf`）加进 `e203_whole_copy_elements`；加之前对着今天的 `DeviceFreeMap` 与 `PoolAllocator` 定义把字段清单从头对一遍（另一个会话的补丁可能已落，以现读为准），再有漏的一并加、报告列出。」
  - 「池级回落按叶取落点那一段（`commit_generated_fallback_slot_open_on_every_device`、`slots_open_on_every_device_in_the_leaf`、`lowest_slot_open_on_every_device_in_the_leaf`）：开一行新计数（名字照登记已有的编号体系取下一个没用过的），逐叶、逐槽各数什么写清；它在准入之前走不走、在 B8 模型里归哪一项，照登记对「准入之前的步」的定义现读代码判，判不出的写进报告交主 agent。」
  - 三样都照「只许收严、原判据与门槛一个字不动」办。
- **每一处相对原口径多数了什么**：这三处的计数点在 E203（准入之前扫类的步代价）的插计数脚本里，本实验的计数构建先照它的编辑表插，所以读数口径同 E203 登记修订 9，细则以那一条为准：s26 的步换成「走到集合里一段 1 步、答案段内每看一对 1 步」（步数比原口径少，不是多数）；s1、s24 整份拷贝的元素数多数六个集合（四个点名的，加字段清单从头对一遍后再有的漏数两个：`PoolAllocator` 的 `allocation_record_tree_of_the_version_without_file`、`RootRingOccupancy` 的 `instance_table_judging_the_rereads` 里的行数；写了理由、没数的两个：`frozen_publish`、`compaction`）；新开一行 s37 `pooled_fallback_by_the_leaf`，数池级回落按叶取落点的叶、候选、槽与每块盘的判。本实验自己的 E207 读数（固定点轮数、节点年龄、重写集、第 4 组）的口径一个字没动。
- **锚点**：E203 编辑表里固定点循环头 `for round_index in 0..ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT` 的锚文改成今天的写法，本脚本原来为绕它而留的 `E203_ANCHORS_REWRITTEN` 随之删掉；E206 的循环头与并重写集两处锚文同样改成今天的写法，数的还是原来那个量。
- **没定、交主 agent 的**：s37 在准入之前走（S2 每轮预演与 S3 真取落点都调到它），但 B8 式子表与三份模型的行表都没有这一行，它的步数进产物行、不进 B1 与 B8 的合计；加不加、B8 一列写什么式子，登记里读不出。
执行员修订 6（2026-11-12，执行员修订 5 之后、新树上第一段一项也没重量之前；读数口径只收严，判据、门槛、取样点、臂与第六、七、九节各条的去留与报告方式一个字没动；依据是派发提示里主 agent 定的四件中的前三件，照抄那几句）：
- **为什么**：执行员修订 5 把 s37 的归属、`compaction` 与 `frozen_publish` 两个字段交主 agent 定，主 agent 定了。三份被测文件此刻的 sha256 前 16 位：`allocator.rs` f124997462f3a4ad、`transaction.rs` d6b2083d2d950d7f、`admission.rs` 7cc59211bf6fcbee；与执行员修订 5 记的相比 `transaction.rs` 没变，`allocator.rs`（61ac7a8f38b4b78d）与 `admission.rs`（301ffb704c7c9f57）因另一个会话的补丁落地而变了，E203 计数脚本的锚点与整份拷贝字段清单已对新内容重核（细则见 E203 登记修订 10），数的量没变。
- **主 agent 定的（派发提示原文）**：
  - 「量的那一侧（B1，今天的实现）：s37 在准入之前的段（P0、S1、S2）里的步数进 B1 的扫类合计，S3 里的照登记读法归准入之后、照报不进合计（与 s25–s29 在 S3 里的数同样处置，登记第 166 行「s35 不用：准入之后真取落点那一格就是 s25–s29 在 S3 里的数」）。」
  - 「推的那一侧（B8、B4）：不给 s37 编式子（B8 对这一步的做法没人定过）。写死一条报告条款：每个取样点、每种发布报 c37（准入之前的调用次数）；c37 = 0 的点 B8、B4 合计照常；c37 > 0 的点，那一点的 B8、B4 合计标「缺 s37 一行，只对清单内的步」，不拿这一点判 B8 站不站得住，报告里列出这些点与它们各自的 s37 步数、s29 步数；岔路单第 9、10、11 行的够判条件落在这种点上的，停下交主 agent（要先由设计员给 B8 定这一步的做法）。」
  - 「s37 与 s29 同条件触发（每块盘都答回落）；登记第 194 行那一句「s29 在所有取样点上都是 0」是推的，这一段不改它，重量之后以读数为准。」
  - 「整份拷贝里没数的两个字段。`compaction` 里的 `quarantines_noted_by_the_publishes` 要数——在副本里给它加取长度的助手，产品源码不动；加不了（私有到追加助手也够不着）就写明够不着在哪一层、停下交主 agent。`frozen_publish` 不数，另加一条读数：被量的每次发布读一次它是不是空，不空的那一点整点作废并报出来。」
  - 「多并进 s37 的那个函数（`lowest_slot_in_the_leaf_with_the_most_slots_open_on_every_device`）：认，留在 s37 里。」
- **每一处落成什么**（细则以 E203 登记修订 10 为准）：本实验模型 `e207_strict_scan_steps.rs` 的 `CALLED_ROWS` 加 s37 读调用次数 pooled_fallback_calls，B8 式子表不加这一行；报告多打每次操作一行 `name=op_pooled_fallback`（pooled_fallback_calls、s37 步数、s29 步数、`by_delta_arms_total=complete|missing_pooled_fallback_row_only_for_the_listed_steps`）与判决行 `name=verdict quantity=missing_pooled_fallback`（标记的操作数、点数与点名）。pooled_fallback_calls > 0 的操作所在的点，B8、B4 合计只对清单内的步，不拿它判 B8 站不站得住；第 9、10、11 行的够判条件落在这种点上的，读报告的执行员停下交主 agent。`frozen_publish`：E207 与 E206 共用的 `state_line` 多一个字段 `frozen_publish_set`（值取 `allocator.frozen_publish().is_some()`，被量的每次操作做之前读一次），模型 `points_with_a_frozen_publish` 把不空的那一点整点作废并打 `name=void_frozen_publish` 与 `name=verdict quantity=void_frozen_publish`；没带这个字段的操作另报个数。`compaction` 在 E203 计数脚本的整份拷贝助手里数，本实验的计数构建先照它的编辑表插，所以同口径。
- **相对原口径多数了什么**：B1 扫类合计多了 s37 在准入之前三段里的步，整份拷贝元素数多了 `compaction` 里隔离落点的条数，多了冻结发布的作废条款；都只会让量更大或让点被作废，没有放宽。
- **E206 回扫点的核对**：E206 的 40 份旧产物只回扫、不重跑，臂定义按各份产物头 `source_sha256` 记的那棵树读（细则见 E206 登记「修订 4」）；本实验引它们的地方（回扫 `--retro`、`reconciliation_with_e206`、`old_criterion_pins`）读的是产物行本身、不引今天树上各臂的语义，逐处核过，没有要改的。回扫点的产物没有 `frozen_publish_set` 字段与 s37 行：回扫读到的 pooled_fallback_calls 为 0、冻结字段读不到，回扫不作废任何点。
- **改名（产物之前，只改名字不改判法）**：本实验模型新加的键名与 verdict 的 quantity 值同 E203 登记修订 10 的「改名」一条：`c37` 改 `pooled_fallback_calls`，`s37_steps` 改 `pooled_fallback_steps`，`s29_steps` 改 `per_disk_fallback_steps`，`b8_b4_total` 改 `by_delta_arms_total`，`missing_s37` 改 `missing_pooled_fallback`，`operations_with_c37_above_zero` 改 `operations_with_pooled_fallback_calls`。
- **判据**：判据、阈值、臂、作废与停机条款、取样点、第六、七、九节各条的去留与报告方式一个字不改。

执行员修订 7（2026-11-13，第一段与第二段产物已出、合并报告用新档名重出之前；只改名字，读数口径、判据、门槛、取样点、臂、作废与停机条款与第六、七、九节各条的去留与报告方式一个字没动；依据：派发提示「档名不合命名规矩：门禁 code-source-discipline 的 data-names 格红 16 处……装置里两个档名写全」）：
- **`user_units` 档的名字改写全**：读法 7 与下文各处的 `u1`（`user_units = 1`）读作 `one_user_unit`，`uK`（`user_units` 等于该点声明的 K、K ≥ 100）读作 `user_units_equal_declared`。装置 `e207_strict_scan_steps.rs` 的档名串、单测、变异表 `e207_strict_scan_steps.tsv` 与合并报告行里的 `tier=` 值一起改；档的判法、哪些集合进哪一档一个字没动。登记正文里的旧名是写登记时的名字，不改，照这一条读。
- **产物**：第一段 14 份、三份探针产物与回扫钉里没有档名（`grep -E 'tier=|u1|uK'` 无命中），不用重量；带档名的是合并报告（读数器算的），用新装置重出，新文件名另起，旧的两份不再被实验页点名。

设计员修订 2（2026-11-13，第一、二段产物已出、第三、四段在跑；只追加：第一至十一节与已有修订的原判据、带、阈值、点、臂、作废与停机条款一个字不改；依据：用户 2026-11-13 的定案「记翻面，再加点复量」，原话在 `records/2026-10-13-里程碑四预开发.md` 第 1662 行；岔路单 `research/prompts/e203-b8-criterion-r1-forks.md` 第 10 行，即本登记的行二）：
- **已记的不动**。第一段合并报告 `research/results/e207-strict-scan-steps-report-first-2026-11-13-r3.out` 第 4552 行：集合 K100、`same_mount`、`one_user_unit`、`later_peak`、`steady`、被测臂，`ends=7167/1/25346,22641/1/250190 … alpha=0.502 band=grows`，只有两点。这一格照第一节读法 4 记翻面，本修订不撤、不改，复量的读数只往行二上加注，去向在下面「去向」一条，写死在跑之前。
- **设计时已经存在的数**（不是新量的；列在这里让下面的主读选择与判读线对得上它；命令 `python3 -I $D/calc/k100_window.py research/results/e207-strict-scan-steps-report-first-2026-11-13-r3.out`，输出 `$D/calc/k100_window.out`，`$D` = `/home/fy5090/.local/share/singlefs/work/e207-revision-designer`）：这一格高端的峰 22641 落在 `files_1000_units_100` 的 W7 第 1 次（同一次挂载、扩展窗口里「覆盖写第一个文件」，`name=op_row` 里 `order=390`）；低端 `files_100_units_100` 没有扩展窗口，没有 W7。只取两端都有的 W1–W4 时，高端的峰是 W4 第 4 次的 15167（记录数 241260），低端的峰不变（W4 第 1 次，7167，记录数 25346），α 读 0.333（介于）。扩展窗口要 200 个互不相同、W1–W4 没碰过的冷文件（第五节「量窗」第 3 条），F 不到约 250 的点凑不出，所以 K100 里只有高端那一个点有 W5–W7；这是量窗的定义与点的大小定的，不是读法造成的。`remounted` 状态没有扩展窗口，K100 `one_user_unit` 档 `later_peak` 的 `steady` 格（0.470）与 `first` 的 `steady` 格（0.484）两端窗口相同。
- **点表**（列同第 211–213 行那张表；U 全取 2¹⁹，K 全取 100，U 与 K 不混进 α；F 按 10 的四分之一次幂等比取，使六点相邻五段的总量比都约 1.78，每一段的分辨相同；记录数约 250 × F，取第一段两端 25346、250190 的比）：

| 点 | U | F | K | 总单元约 | 形状 | 发布数约 | 段 | 用途 |
|---|---|---|---|---|---|---|---|---|
| `files_56_units_100` | 2¹⁹ | 56 | 10² | 5.6×10³ | `SmallFilesOfUnits(100)` | 约 134 | `k100_repeat` | 序列低端之下一点（记录数约 1.4×10⁴） |
| `files_100_units_100` | 2¹⁹ | 10² | 10² | 10⁴ | 同上 | 约 222 | `k100_repeat`（第一段同名项再跑一次） | 钉：W1–W4 逐项与第一段产物相同；序列里的这一点取这一次的读数 |
| `files_178_units_100` | 2¹⁹ | 178 | 10² | 1.78×10⁴ | 同上 | 约 378 | `k100_repeat` | 两端之间（记录数约 4.5×10⁴） |
| `files_316_units_100` | 2¹⁹ | 316 | 10² | 3.16×10⁴ | 同上 | 约 654 | `k100_repeat` | 两端之间（记录数约 8.0×10⁴） |
| `files_562_units_100` | 2¹⁹ | 562 | 10² | 5.62×10⁴ | 同上 | 约 1146 | `k100_repeat` | 两端之间（记录数约 1.4×10⁵） |
| `files_1000_units_100` | 2¹⁹ | 10³ | 10² | 10⁵ | 同上 | 约 2022 | 不重跑，读第一段产物（`sha256` `633b2634262fdd9a5349bc917068de320bbd90111d1af6927a8cc315bbfb4d87`） | 序列高端；公共窗读数里丢掉它的 W5–W7 |

发布数约用 `N = 2 + 2F + 2u`（u = 10，`e207_strict_scan_steps_window_choices.rs` 的 `notes_of_the_history`，推的），只当进度分母。序列共六点，相邻五段，总量跨度约 17.9 倍（记录数 1.4×10⁴ 到 2.5×10⁵）。**不加高端之外的点**：再往上一个等比点 F 取 1780，按下面「用时与内存」的上界缩放要 2.8 小时以上、内存按单元数比约 25 GiB，而六点已有相邻五段；主读为「不稳」时才由主 agent 另议，本修订不登记它。
- **量窗**：新点与钉的 W1–W4 同第一至三段「新点」的取法（第五节「量窗」第 2 条：冷文件池按挂载状态劈成两半，F 取 56 时每个状态的池有 28 个文件，第 4 节第 3a 步③的「至少 5 个第一次碰、仍是 K 个单元的文件」仍满足）；两种挂载状态；没有扩展窗口、阴性对照与探针（扩展窗口凑不出，阴性对照只在对账点）。
- **读数 `k100_series`（跑前写死）**：对 K100 的每一格（状态 × 档 × `first` / `later_peak` / `later_median` × `all` / `steady`，被测臂与 `C1`、`C2`、`C3`）在六点上各取该点该格的（步数、`user_units`、量到的 x）三元组，读法同第一节读法 3–8（x 取操作之前的 `allocation_records`，另报 `mapped_units` 的那一份；两端 `user_units` 相同，读原步数，`looser=no`）。每格报两份：`common_window`（只取 W1–W4 的操作，六点同一个操作集合，主读）与 `own_window`（每点取自己产物里的全部操作，高端含 W5–W7，只报）。每份报：各点的三元组、相邻五段的 α（按 x 升序）、普通最小二乘斜率 β（ln Q 对 ln x，六点同权）、β 的档（同读法 4）、相邻五段里「随 / 介于 / 不随」的个数 g、b、d、`mapped_units` 作 x 的 β，与判语 `verdict`：
  - `stable_grows`：β ≥ 0.5 且 g ≥ 3 且 d = 0；
  - `stable_does_not_grow`：β ≤ 0.25 且 d ≥ 3 且 g = 0；
  - `stable_between`：0.25 < β < 0.5 且 g = 0 且 d = 0；
  - `unstable`：以上都不是；
  - 读到的点少于三个：`not_enough_points`，不判。
  只有 `common_window` 的、状态 `same_mount`、档 `one_user_unit`、`later_peak`、`steady`、被测臂那一格标 `primary=yes`，去向只看它；其余各格照报 `verdict`，不进去向。
- **阳性对照**：同一格上 `C1`、`C3` 在 `common_window` 的 β 都须 ≥ 0.5（读法 4 的「随」），不是则主格 `verdict` 作废、记 `series_void=control_not_red`，只报数（同作废条款 1、2 的写法，只对这一系列）。`C2` 只报。
- **去向（对岔路单第 10 行，不回头改已记的翻面）**：

| 主格的 `verdict` | 对行二（第 10 行） |
|---|---|
| `stable_grows` | 复量支持：行二在 `one_user_unit` 档、K100、`same_mount`、`later_peak` 这一格上记「翻面，公共窗六点复量站住」，该格对行二够判 |
| `stable_between` 或 `stable_does_not_grow` | 复量不支持：已记的两点翻面一个字不改，加注「公共窗六点 β、相邻五段 α 不到 0.5；两点读数的峰来自只有高端有的 W7 第 1 次」；这一格最后记不记翻面由主 agent 定，登记不替它判 |
| `unstable` | 既不支持也不推翻：已记的翻面照留并加注「复量不稳」，这一格对行二仍开着；要再判得另立登记（高端再加一点、或把扩展窗口做成随 F 缩的版本），本修订不登记 |
| `not_enough_points` 或 `series_void=control_not_red` 或下面作废条款成立 | 复量未成，已记的翻面照留，岔路单第 10 行不因这一趟变动 |

`own_window` 的相邻五段 α 与 `common_window` 逐段并列报，让「翻面的两点读数里有多少是 W7 第 1 次带来的」在同一张表上看得见；它不进去向。主读选 `common_window` 是我的判断（只有它的六点操作集合相同），不是用户定的；主 agent 另判的话，在装置写之前告诉实验设计员改去向表，读数两份都照打、不受影响。
- **作废与停机（复量这一段）**：① 三份核心文件的 sha256 与第一段产物头 `name=source_sha256` 不同（此刻 `allocator.rs` `eb822f0920329af40400071c5b2f306b3fe92e1899fdca8984f5edf65a426faf`、`transaction.rs` `d6b2083d2d950d7fc7eae9a61fee3fffd67f11df1f999be1df78a6265c3a628b`、`allocation_record_tree.rs` `991623b6a9b5028eb692b8e88a3e6acaab6c1471c5302b23dffeb6f42ea958ce`，与第一段 15 份产物头相同，`grep -h name=source_sha256` 数过）：新点与高端不是同一棵树上量的，序列作废，被测的树要换时主 agent 先问。② 钉过不了：`files_100_units_100` 再跑一次，W1–W4 每个操作（两种状态）的 `rounds`、`final_roles`、`final_nodes`、`round_work`、`final_set_hash`、`previous_records` 与第一段同名产物（`sha256` `ad3e59ef57e3ec0090b4bf37a733e3a27fa914999b3b89ab4b566b86ca5501c0`）逐个相同；任何一处不同走停机条款第 3 条（两边都查，装置新加的点表与读数行改了行为，还是树变了），序列作废、不当结果。③ 缺 `name=item_done`、声明行数不等于量窗操作数、某轮缺字段：该点本次未跑（作废条款 5），序列用剩下的点；加上高端不足三点就记 `not_enough_points`。④ 「第一次碰、仍是 K 个单元」的 W2 操作少于 5 个（F8）：该点该状态记样本不足，序列里去掉这一点这一状态。⑤ 点 `name=history_refused` 或固定点前提不成立（F4）：照 F4 当场报，该点不入序列。
- **钉绝对值的断言**（独立算出，`awk -f $D/calc/series_cases.awk`，输出 `$D/calc/series_cases.out`；模型单测钉这五行，输入写进测试）：六点 x 每步乘 10 的 0.25 次幂，Q = x^a：a = 0.6 全程 ⇒ β = 0.600、相邻五段 0.600、g = 5、`stable_grows`；a = 0.4 ⇒ β = 0.400、b = 5、`stable_between`；a = 0.1 ⇒ β = 0.100、d = 5、`stable_does_not_grow`；前三段 0.6、后两段 0.1 ⇒ β = 0.414、g = 3、d = 2、`unstable`；前四段 0.1、最高一段 0.9 ⇒ β = 0.214、g = 1、d = 4、`unstable`（只在最高端翘起的不记稳）。再钉两条出自存量产物的：读数器吃第一段 `files_100_units_100` 与 `files_1000_units_100` 两份产物，`own_window` 的两端读数与第 4552 行逐字符相同（`ends=7167/1/25346,22641/1/250190`、`alpha=0.502`），`common_window` 的两端读数为 `7167/1/25346,15167/1/241260`、α 0.333（`k100_window.out`）；两条不符走停机条款第 3 条。
- **变异**（装置模型 `scripts/mutation/tables/e207_strict_scan_steps.tsv` 另加四行，每条写明在哪个取样点上改变输出；本修订的变异不占第九节的编号）：`ols_slope_replaced_by_the_end_to_end_slope`（β 换成首尾两点的 α）——构造「前四段 0.1、最高一段 0.9」：首尾 α 0.260、β 0.214，档由「不随」变「介于」；`stable_between_allows_one_grows_adjacent`（`stable_between` 的 g = 0 改成 g ≤ 1）——构造同一输入以外的「五段里一段 0.5、四段 0.4」：判语由 `unstable` 变 `stable_between`；`common_window_keeps_the_extended_operations`（公共窗不丢 W5–W7）——构造高端带一个比 W1–W4 各次都大的 W7 操作：高端峰与 α 变；`control_gate_ignored`（`C1`、`C3` 不红仍读主格）——构造 `C1` 的 β 为 0.3：主格 `verdict` 由「作废」变成它本来的判语。
- **落地清单（交执行员，装置新写，文件:行是今天的；改之前各按名字 `grep` 一遍，不重名）**：① `scripts/experiment-apparatus/e207_strict_scan_steps_histories.rs:100`–`:131` 的 `point_named`：加四行 `units(100, 19, F, about)`，F 取 56、178、316、562；`:948`–`:1035` 加四个 `item_files_<F>_units_100__head`（形同 `:953` 的 `item_files_100_units_100__head`）；`files_100_units_100` 那一项不另写，按段名再跑。② `scripts/experiment-apparatus/e207_strict_scan_steps_window_choices.rs`：新点不进 `RECONCILED_WINDOW_POINTS`（`:16`）、`EXTENDED_WINDOW_POINTS`（`:20`）、`NEGATIVE_CONTROL_POINTS`（`:18`），`cold_file`（`:74`）走劈开池；加一条单测钉这三件事。③ `scripts/experiment-apparatus/e207_strict_scan_steps.rs`：`:1286` 旁加六点序列集合 `K100_SERIES`（点名六个，高端读第一段产物），加按操作名取 W1–W4 的过滤与 `common_window` / `own_window` 两份读数、β、g / b / d、`verdict` 的函数（新函数名先 `grep`），`:2226`–`:2260` 的分段分发加 `k100_repeat` 分支，`:2280` 的用法与 `:2290` 的段名名单加 `k100_repeat`，报告多收一个 `--pin-against <第一段 files_100_units_100 产物>` 参数，打 `name=verdict quantity=k100_pin matches=…`；打 `name=series_cell …`（字段同上、`primary=yes|no`）与 `name=verdict quantity=k100_series outcome=<主格 verdict> series_void=<-|control_not_red> points=<n>`；模型单测钉上一条的五行与两条存量。④ `scripts/experiment-apparatus/e207-strict-scan-steps-run.sh`：`items_table`（`:131`）加五项（四个新点加 `files_100_units_100__head`，段名 `k100_repeat`；`files_100_units_100__head` 同属第一段与本段，第一段仍是十四项，`:1479` 那一格不变，另加一格数本段是五项）、`--segment` 认 `k100_repeat`、文件头用法与 `--list-items` 同改、分片的片数函数 `shard_count_of` 给四个新点片数；产物名照现有的（新日期，同名已有的另加 `-r2` 之类的后缀，不覆盖第一段那份）。⑤ 同步 `scripts/experiment-apparatus/replay.sh` 的登记行（复跑表，模型对已落盘产物 `exact`）。⑥ 判定库键因 histories 文件改了而变，本段五项都现判，第一段十四项的库键同样变，下次复用要重判，不是缺陷。
- **用时与内存（推的上界，没量；依据：第一段全部项在新树上的最长一片是 5678 秒，其中 `files_1000_units_100` 的最后一片要放完整段历史，所以它的整项不超过 5678 秒；每次发布的代价随总量不降，所以 F 小一点的点按线性缩放不超过这个比例；内存取第一段撞上限重来那几片的最高值 13824 MiB，新点的写到过的单元都不多于高端）**：

| 项 | 挂钟上界 | 内存上界 | 超过 2 小时吗 |
|---|---|---|---|
| `files_56_units_100__head` | 约 5 分钟（5678 秒 × 0.056） | 13.8 GiB（实际小得多） | 否 |
| `files_100_units_100__head`（钉） | 约 10 分钟 | 同上 | 否 |
| `files_178_units_100__head` | 约 17 分钟 | 同上 | 否 |
| `files_316_units_100__head` | 约 30 分钟 | 同上 | 否 |
| `files_562_units_100__head` | 约 53 分钟（线性缩放上界；若每次发布的代价按 F 的平方涨，则不到 53 分钟，更小） | 同上 | 否 |
| `--segment k100_repeat` 整段 | 项间互相独立，由服务分片分机，整段挂钟取决于最长项，约 53 分钟 | 五项并行合计上界约 69 GiB，由服务分机与放大 | 否 |

预计在用核数：每项用例单线程，五项并行约 5 个核，加编译（服务给的线程）；整段不超过 2 小时，不需要单元内部切分；分片单元是 `<点>__head`，续跑入口是同一条命令重起，只补库里没做完的项（形态同第十四节「切分与续跑」）。
- **起它的整条命令形态**（主 agent 交；不调判段入口，不带 `--features gpu`，不立 `NOT_RUN` 那一条）：

```text
SINGLEFS_RUN_SCOPE='种类=items; 跑什么=E207 K100 复量点五项; 范围=k100_repeat 段全部 5 项; 为什么=岔路单第 10 行贴线格复量，总量点加到六个' task-server/task submit -- bash scripts/experiment-apparatus/e207-strict-scan-steps-run.sh --segment k100_repeat --work <共享根下新工作目录>
```

  合并报告照第一段合并报告的起法（`e207-strict-scan-steps report --segment k100_repeat <五项新产物> <第一段 files_1000_units_100 产物> --pin-against <第一段 files_100_units_100 产物>`，经服务交，种类 `target` 或照 `replay.sh` 的行），产物名 `research/results/e207-strict-scan-steps-report-k100-repeat-<日期>.out`。

设计员修订 3（同一时点；只追加，判据、门槛、取样点、臂与第一至十一节的去留一个字不改；依据：用户 2026-11-13 的定案「加分辨读数查清」，原话在 `records/2026-10-13-里程碑四预开发.md` 第 1662 行；它补第七节 ② 第 2 条、作废条款 6 与停机条款第 3 条在 `files_1000` 上打中之后「查明是哪一种」那一步，不改那几条的字）：
- **现象**（抄第一段合并报告 `…-report-first-2026-11-13-r3.out` 第 4989 行）：`name=verdict quantity=reconciliation_with_e206 point=files_1000 settles_today=208 settles_e206=208 differing=104 matches=false first_differing=remounted/W1/1/0`；`base`、`files_10000` 两点 `matches=true`。
- **设计时已经存在的数**（不是新量的；命令 `python3 -I $D/calc/reconciliation_fields.py /home/fy5090/code/singlefs`，输出 `$D/calc/reconciliation_fields.out`，`$D` 同上）：① 104 处全在 `remounted`：W1 32、W2 32、W3 32、W4 8；`same_mount` 一处不差。② 五项里差的字段集合：只差 `final_set_hash` 的 59 处，四项加哈希 29 处，五项 12 处，`rounds`、`round_work`、哈希 4 处。③ 第一处 `remounted/W1/1/0` 只差 `final_set_hash`，`previous_records` 等输入两边相同；输入开始不同在 `remounted/W1/3/0`（`previous_records` 57 处不同）；池自己的 `name=state` 行（`allocation_records`、`cluster_segments`）E206 对今天在 `remounted` 里分别有 42、44 个操作不同，第一处 `remounted/W1/3`（`allocation_records` 5654 对 5652，`cluster_segments` 1 对 2）；`base` 与 `files_10000` 的这几个字段（`allocation_records`、`cluster_segments`、`mapped_units`、`units`）逐个相同（差 0）。④ 第一段旧树（2026-11-12 那批产物，`allocator.rs` 前 8 位 329eb074、`transaction.rs` 8bfdc662）对今天的树（eb822f09、d6b2083d）在 `files_1000` 上五项一处不差；两边对 E206 的差是同一批 104 处。所以这 104 处在第一段旧树上就有，不是之后合入的补丁带来的。⑤ E206 那一棵树取得到：`git show b14b6fe92:crates/singlefs-core/src/allocator.rs | sha256sum` 前 8 位 a07e50d8、同提交的 `transaction.rs` 前 8 位 805afb9d，与 E206 产物头记的一致；第一段旧树在提交 `8db07de9d`；这两个提交之间 `allocator.rs` 与 `transaction.rs` 共 1635 行增、178 行删（`git diff --stat b14b6fe92 8db07de9d -- crates/singlefs-core/src/allocator.rs crates/singlefs-core/src/transaction.rs`）。⑥ 池级回落（D3（空间分配） 已定项 8 的回落）在 `files_1000` 的 162 个量窗操作里调用次数全是 0（第一段合并报告的 `name=op_pooled_fallback` 行），所以「回落政策那一处改动」没走到，不是候选。⑦ 历史末镜像 `image_sha256`：E206 的 p2a 产物、E207 的 `head` 与 `plain` 三份相同（`c82a604c…`）。⑧ 两边驱动的重挂载都是 `pool.crash_and_mount_writable()`（本装置 `e207_strict_scan_steps_histories.rs:917`，E206 装置 `e206_fixed_point_cost_histories.rs:700`）。
- **问法与三种结局**：问的是这 104 处不符从哪来。(一) `tree_changed` = 被量的实现重挂载之后的行为与 E206 产物那一份不同（池自己的结构不同），今天装置打的数如实反映了它；(二) `counting_wrong` = 今天装置的计数（插了计数的构建）记错了，或计数构建改变了树的行为；(三) 两者都不是 = 下面 `neither_*` 两种。**这一条分不开的**：E206 那一份产物当时有没有被它自己的计数补丁扰动过（E206 产物里只有历史末那一个镜像哈希，窗口里没有逐操作的对盘）；(一) 只说「今天装置自洽、池结构与 E206 记录的不同」，不说差是 E206 树加 E206 的 p2a 补丁对今天原生 p2a 的哪一处引起的；要定到提交，是另一次实验（候选的两棵树：提交 `b14b6fe92`、`8db07de9d`），本修订不登记。
- **读数（新装置，跑前写死）**：
  - `image_after_the_operation`（历史用例，`base`、`files_1000` 两点，`head` 与 `plain` 两个构建都打）：每个量窗操作做完之后一行 `E207RESULT name=image_after_the_operation item=<项> phase=window state=<状态> operation=<名> operation_index=<次> image_sha256=<镜像的 sha256>`；重挂载前后各一行 `name=image_at_the_remount item=<项> when=before_the_crash|after_the_mount image_sha256=<…>`。镜像取法同 `:632` 的 `image_sha256(&pool.image())`，不读计数模块，所以 `plain` 也能打。
  - `reconciliation` 行（已有，`image_changed_unit_slots` 与 `slots_placed`）从「`base` 的 `same_mount` W1–W3 第 1 次」扩成 `base`、`files_1000` 两点两种状态的**每一个**量窗操作（只在 `head` 构建打；`plain` 的 `slots_placed` 恒 0，不打这一行）。
  - 模型（离线，吃产物）打四行 verdict：`head_against_plain`（逐操作比 `head` 与 `plain` 的镜像：`operations`、`differing`、`first_differing`、重挂载前后各是否相同）；`placement_gap`（每个操作 `slots_placed − image_changed_unit_slots`：`operations`、`negative`、`first_negative`、`zero`、`positive`；负 = 计数少记了盘上真变了的槽，正小是「放下、盘上内容没变」，E206 对账调查已见过：W2 第 1 次 32 对 30）；`rerun_against_the_first_segment`（这次 `head` 的 W1–W4 对第一段同名 `head` 产物：`settles`、`differing`、`first_differing`，比 `rounds`、`final_roles`、`final_nodes`、`round_work`、`final_set_hash`、`previous_records`）；`state_rows_against_e206`（`base`、`files_1000`、`files_10000` 三点：`operations`、`differing_operations`、`by_field` = `allocation_records` / `cluster_segments` / `mapped_units` / `units` 各几个操作不同、`same_mount_differing_operations`、`first_differing`、`inputs_equal_at_the_first_differing_publish` = 五项里第一处不同的发布上，`previous_records`、`previous_nodes`、`tree_height` 与池的 `name=state` 行在两边是否相同；`files_10000` 与 E206 的第三个 p2a 产物只做这一行，不重跑）。
  - 最后一行 `name=verdict quantity=reconciliation_split point=files_1000 outcome=<见下> reason=<哪几个谓词>`。`base` 同样打一行 `point=base`，当阴性对照。
- **谓词与结局（逐条写死，次序就是判的次序）**。令：`c_head` = `head_against_plain` 在 `files_1000` 上 `differing > 0` 或镜像重挂载前后任一不同；`c_gap` = `placement_gap` 在 `files_1000` 上 `negative > 0`；`c_rerun` = `rerun_against_the_first_segment` 在 `files_1000` 上 `differing > 0`；`t_sign` = `state_rows_against_e206` 在 `files_1000` 上 `differing_operations > 0`，且同一行里 `same_mount_differing_operations = 0`，且 `base` 那一行 `differing_operations = 0`。

| 结局 | 读到 | 记作 |
|---|---|---|
| `counting_wrong` | (`c_head` 或 `c_gap`) 且非 `t_sign` | 今天的计数记错或扰动了树：作废条款 6 前半成立，`files_1000` 读数作废；`base`、`files_10000` 只在它们的同一谓词也红时作废；交主 agent |
| `both` | (`c_head` 或 `c_gap`) 且 `t_sign` | 两样都有，不分比例；`files_1000` 读数作废（按计数那一样），池结构的差照报；交主 agent |
| `tree_changed` | 非 `c_head`、非 `c_gap`、非 `c_rerun`、`t_sign` | 作废条款 6 后半「查明是今天的树行为变了」成立，走停机条款第 3 条的原路径：不作废，差与结局写进报告交主 agent；对账点 `files_1000` 记「与 E206 在重挂载之后池结构不同」，不记「匹配」 |
| `neither_reading_differs` | 非 `c_head`、非 `c_gap`、非 `c_rerun`、非 `t_sign`，而五项仍有不同 | 两者都不是：池结构两边一样，差只在两份补丁打出来的读数的口径（哈希、轮数的定义）；对账判法要改成只比池自己的结构，另写修订，交主 agent |
| `neither_not_reproducible` | 非 `c_head`、非 `c_gap`，`c_rerun` | 两者都不是：同一棵树同一装置再跑一次与第一段不同，先查不确定性；这一趟 `files_1000` 的任何对账结论都不下 |
| `not_run` | 缺 `item_done`、缺 `plain` 对应项、产物里缺新行 | 本次未跑，不下结论 |

**阴性对照**：`base` 一行必须是 `c_head`、`c_gap`、`c_rerun`、`t_sign` 全假（第一段 `base` 对账 `matches=true`，`state` 行差 0）；不是则读数器本身有错，走停机条款第 2 条，`files_1000` 的结局不当结果。**阳性对照（每个会触发的谓词各一条）**：① `c_gap`：`E207_RUN_BREAK=slots_placed_undercounted`，`reconciliation` 行打的 `slots_placed` 减 1，`--break-history slots_placed_undercounted --item base__head` 的 `placement_gap` 里 `negative ≥ 1`（`base` 的 W1 第 1 次原本 32 对 32，gap 为 0，减后 −1），打 `caught=yes`；② `c_head`：`E207_RUN_BREAK=image_witness_flipped`，只在 `plain` 构建里把 `remounted` W2 第 5 次那一行的镜像做哈希之前翻一位，起 `base__plain` 一项，模型读正常的 `base__head` 与这份坏产物，`first_differing=remounted/W2/5`；③ `c_rerun`：模型单测把一份产物的某个 `final_set_hash` 改一位再比；④ `t_sign`：模型单测造一对 E206 与今天的 `name=state` 行，只让一个 `remounted` 操作的 `cluster_segments` 不同，`differing_operations=1`、`first_differing` 指对；真数据上 `files_1000` 的 42、44 与 `base`、`files_10000` 的 0 是现成的一正一负。
- **重跑哪几点**：`base__head`、`base__plain`、`files_1000__head`、`files_1000__plain` 四项（段名 `reconciliation_split`，项名与第一段同名；第一段仍是十四项、内容不变）。`files_10000` 不重跑（它对账匹配，只做 `state_rows_against_e206` 那一行，离线）；`files_10000` 与 `files_1000_units_100` 的逐操作对盘与 `placement_gap` 标「够判后未跑」。
- **失败条款（这一段）**：① 阴性对照 `base` 任一谓词为真：读数器或新加的打印改了行为，走停机条款第 2 条，本段结局不当结果（观测：`base` 那一行 `reason` 里有谓词名）。② 新装置改了行为：重跑的 `base__head` 或 `files_1000__head` 的 W1–W4 对第一段同名产物有差（`rerun_against_the_first_segment` 的 `differing > 0` 在 `base` 上就出现）——这时 `base` 的 `c_rerun` 为真，按上一条处理，装置的新打印要先查（加镜像哈希与逐操作 `image()` 复制不该影响固定点）。③ `t_sign` 的前提读不到：产物里 E206 的 `name=state` 行缺 `cluster_segments` 或 `allocation_records`：记 `not_run`，不拿别的字段顶替。④ 结局是 `neither_reading_differs` 或 `neither_not_reproducible`：这一条的假设「差在池结构或计数」两样都被打中，先照 `evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」问那四句，再交主 agent，不自己改谓词。
- **作废**：本段四项缺 `name=item_done`、声明行数不等于量窗操作数（作废条款 5）、三份核心文件的 sha256 与第一段产物头不同（同复量那一条的①）：该项本次未跑。`base__plain` 与 `files_1000__plain` 已有的 `image_sha256`（历史末）仍要与 `head` 相同（作废条款 3），本段重跑后同样核一遍。
- **落地清单（交执行员，装置新写；文件:行是今天的，改之前先 `grep` 名字，不重名）**：① `e207_strict_scan_steps_histories.rs:52`–`:54` 旁加常量 `PER_OPERATION_IMAGE_POINTS`（`base`、`files_1000`）；`RECONCILED_POINTS`（`:52`）的用处（`:700`–`:703`）改成：这两点两种状态每个量窗操作、`head` 构建才打 `reconciliation` 行。② `:686`–`:731` 的 `measure`：在 `change(...)`（`:706`）返回之后、拒绝分支（`:714`）之前，对这两点打 `name=image_after_the_operation`（`head`、`plain` 都打）；`:917`–`:919` 的 `crash_and_mount_writable()` 前后各打一行 `name=image_at_the_remount`；`E207_RUN_BREAK` 两个新开关（`slots_placed_undercounted`、`image_witness_flipped`）只在这两处读一次。③ `e207_strict_scan_steps.rs:436`–`:452` 的产物结构加两个字段收新行，`:563` 旁加两个读新行的分支；`Operation`（`:968`）补 `cluster_segments` 与 `units`；新函数 `report_head_against_plain`、`report_placement_gap`、`report_rerun_against_the_first_segment`、`report_state_rows_against_e206`、`reconciliation_split_outcome`（先 `grep` 名字），放在 `:2065` 的 `report_reconciliation` 旁，不改它；`:2226`–`:2260` 的分发加 `reconciliation_split` 分支，`:2280`、`:2290` 加段名，报告多收 `--previous <第一段 base、files_1000 的 head 产物>`，`--retro` 带 E206 三个 p2a 产物（`base`、`files_1000`、`files_10000`）。④ 模型单测钉：六个谓词各一正一负（用上面阳性对照的构造）、结局表六行各一条构造输入、`state_rows_against_e206` 在 `files_1000` 真产物对上的 42、44 与 `base`、`files_10000` 的 0；`scripts/mutation/tables/e207_strict_scan_steps.tsv` 加变异：`t_sign_ignores_the_same_mount_control`、`placement_gap_counts_positive_as_negative`、`rerun_compares_four_fields_only`（少比 `final_set_hash`，构造只差哈希的一对：`differing` 由 1 变 0）、`outcome_precedence_swapped`（`both` 与 `counting_wrong` 的次序对调，构造 `c_gap` 且 `t_sign`：结局由 `both` 变 `counting_wrong`）。⑤ `e207-strict-scan-steps-run.sh`：`items_table`（`:131`）加四项到段 `reconciliation_split`，`--segment` 认它；`--break-history`（文件头 `:10`–`:12`、函数 `break_history`）认两个新开关，`slots_placed_undercounted` 判 `caught=` 看 `placement_gap` 的 `negative ≥ 1`，`image_witness_flipped` 只起 `base__plain` 一项、判法在模型对拍那一步；`--selftest` 加「本段是四项、第一段仍是十四项」一格；产物名新日期（同名已有的加 `-r2` 之类的后缀，不覆盖第一段那四份）。⑥ `scripts/experiment-apparatus/replay.sh` 登记行同步。⑦ 判定库键随 histories 文件变，本段四项现判。
- **用时与内存（推的；没量）**：依据同复量那一条——第一段全部项里最长一片 5678 秒，`base` 与 `files_1000`（U = 2¹⁷、每文件 1 个单元）是第一段里最小的点，两项各不会超过这个数，预计各在 15 分钟内（推的）；新加的打印每项 164 次镜像（量窗 162 个操作、重挂载前后各一）：镜像约 50 MB（单元区约 1.1×10³ 个单元 × 2 盘 × 16 KiB 加 6 MiB 环 × 2 盘，推的），手写 sha256 按每秒 100–300 MB 估，每项加不超过 90 秒；每次再多复制一份镜像，内存加不到 0.2 GiB。内存上界按登记第十四节「U = 2¹⁷ 的点不到 4.3 GiB」。`plain` 两项只多这 164 次镜像哈希，各约 5 分钟内。四项互相独立，约 15 分钟，四个核；不需要单元内部切分。
- **起它的整条命令形态**（主 agent 交；不调判段入口，不带 `--features gpu`）：

```text
SINGLEFS_RUN_SCOPE='种类=items; 跑什么=E207 对账分辨读数四项; 范围=reconciliation_split 段全部 4 项; 为什么=分开 files_1000 对账 104 处不符是树变了还是计数记错' task-server/task submit -- bash scripts/experiment-apparatus/e207-strict-scan-steps-run.sh --segment reconciliation_split --work <共享根下新工作目录>
```

  合并报告 `e207-strict-scan-steps report --segment reconciliation_split <四项新产物> --previous <第一段 base、files_1000 的 head 产物> --retro <E206 的 base、files_1000、files_10000 的 p2a 产物>`，产物名 `research/results/e207-strict-scan-steps-report-reconciliation-split-<日期>.out`；阳性对照 ① 的 `--break-history slots_placed_undercounted --item base__head`、② 的 `image_witness_flipped` 起 `base__plain` 各经服务交一次（只此一项、不入判定库）。

设计员修订 4（同一时点；只追加，不改判据）：**plain 对照的点数以哪一处为准**。字面有两种写法：第五节「实现量与分段」第一段（本登记第 244 行）写「对账点（`base`、`files_1000`、`files_10000`、`files_1000_units_100` 的 `standard`）加 plain 对照」，字面四点；作废条款 3（第 405 行）、第七节 ② 第 5 条（第 313 行）、第五节「取样点」表（第 203 行）与装置的 `IMAGE_POINTS`（`e207_strict_scan_steps_histories.rs:54`）都只点 `base`、`files_1000` 两点。**以作废条款 3 与第七节 ② 第 5 条为准：plain 对照两点（`base`、`files_1000`）**；第 244 行那一句读作对账点名单的行文，不是 plain 的点数。判据一个字不改（判据句本来就写两点）：不欠 `files_10000` 与 `files_1000_units_100` 的 plain，不记欠账；实验页写 plain 两点。修订 3 重跑的 `base__plain`、`files_1000__plain` 与它一致。

执行员修订 8（2026-11-13，第四段大点与修订 2、3 两段的产物一次没跑之前；只补装置的做法与读数的写法，判据、门槛、取样点、臂与第六、七、九节各条的去留与报告方式一个字没动；依据：派发提示「第四段最大点装不下……做法你定……改前后同一个点的读数逐项相同……不许换判据、不许缩点」与「落设计员修订 2（复量段）与修订 3（对账分辨段）」）：
- **内存做法**：历史用例的盘改用紧凑盘（`scripts/experiment-apparatus/e207_strict_scan_steps_compact_device.rs`）：单元区里内容相同的 512 字节扇区只存一份，按槽记每个扇区用哪一份，单元区以下仍走稀疏盘；写进去什么读回什么，没写过的读 0，清零与 discard 删掉记着的扇区，不改被量的实现、读数与产物。要取镜像的点（`base`、`files_1000`）仍用整块稀疏盘。钉：第一段已有产物的小点 `files_10_units_1000_unit_area_2_pow_17__head`，改后重出与留存产物逐字节比；紧凑盘与稀疏盘读回逐字节相同由入口 `--device-test` 起的四条单测核。每片另往标准错误打 `e207 memory …` 行（常驻、峰值、写过的扇区数与不同内容份数），不进产物。
- **复量段读数的写法**：另打 `name=verdict quantity=sources_agree`（作废条款 ① 的核）；`series_void` 取值 `-`、`control_not_red`、`pin_differs`、`sources_differ`、`sources_unread`；主格行带 `route=`（去向表那一列：`supports_the_flip_on_the_common_window`、`does_not_support_the_flip`、`unstable_the_cell_stays_open`、`series_not_made`）；F8 与未做完的点用 `point_is_usable_in_the_series`，不够的点该状态不进序列。
- **对账分辨段读数的写法**：结局表六行之外，没有谓词为真、五项也不再有不同的那一格表里没写，记 `nothing_left_to_split`，交主 agent；`base` 行的 `outcome` 取 `negative_control_clean` 或 `negative_control_red`（红走停机条款第 2 条）；`--previous` 另给第一段 `files_10000` 的 head 产物，`state_rows_against_e206` 在 `files_10000` 上今天那一侧读它；总判行 `reason=` 里谓词写全名（命名不写单字母前缀，同修订 7 的理由）：`c_head` 写 `head_differs`、`c_gap` 写 `gap_negative`、`c_rerun` 写 `rerun_differs`、`t_sign` 写 `changed_tree_signature`，判法一个字没动。
- **项表**：`files_100_units_100__head`（复量的钉）与 `base`、`files_1000` 的 `head`、`plain` 四项在项表里各占本段与第一段两行，项名同；产物名取入口的同日 `-r2`，不覆盖第一段那份。

执行员修订 9（2026-11-13，修订 8 之后、复量段、对账分辨段与第四段产物一次没跑之前；只改名字与补装置的内存自报与片数做法，读数口径、判据、门槛、取样点、臂与第六、七、九节各条的去留与报告方式一个字没动；依据：派发提示「命名 13 处……这一族名字全写全……登记里设计员起的段名在第十二节加一条修订写新旧对照（判据不动）」与「片开工前自报内存」「同一个点的几片不同时在一台上涨」）：
- **名字新旧对照**（设计员修订 2、本登记第五节与第十四节里写的旧名照这一条读）：段名与项表里的段列 `k100_repeat` → `hundred_units_per_file_repeat`（入口 `--segment`、模型 `report --segment` 同改）；产物里判决行的 `quantity=k100_pin` → `quantity=hundred_units_per_file_pin`、`quantity=k100_series` → `quantity=hundred_units_per_file_series`；模型里的常量、函数与单测名里的 `k100` 一律写成 `hundred_units_per_file`（`K100_SERIES` → `HUNDRED_UNITS_PER_FILE_SERIES`、`report_k100_pin`、`report_k100_series`、`K100_REPEAT_POINTS`）；复跑登记驱动 `driver_e207_report_k100_repeat` → `driver_e207_report_hundred_units_per_file_repeat`，产物名里的 `k100-repeat` → `hundred-units-per-file-repeat`；量窗取法的变异行名 `k100_repeat_point_in_the_reconciled_points` → `hundred_units_per_file_repeat_point_in_the_reconciled_points`。比较集合名 `K100`、`K1000`（已落进留存产物的 `set=K100` 这类值）不动；设计时跑过的命令里的 `k100_window.py` 是当时的文件名，照留。读数的写法、判据与门槛一个字没动。
- **单测里元组首列的名字**：`image_before_the_crash`、`image_after_the_mount`、`same_image` 取代单字母加数字的占位串，只是测试输入的名字，不影响任何断言。
- **内存自报**：入口的计划那一步（编带计数的测试程序）与每一片开工前，读环境变量 `TASK_SERVER_MEMORY_CAP_BYTES`，低于入口里按项的内存表要的量就往标准错误打 `TASK_SERVER_MEMORY_NEEDED bytes=<字节数>`、退 250、一步不做；没设这个变量、表里没有读数的项与片照常走。表里此刻三行：计划那一步 10.71 GiB，最大点第 1/8 片末 26.82 GiB、第 2/8 片末 40.36 GiB（稀疏盘量的；紧凑盘的读数量到再以 `compact` 行加进来，有 `compact` 行的项只看它）。
- **同一个点的几片不同时涨**：紧凑盘上量到的单片内存到这一台能给的内存的四分之三以上的项，片数取 1；没有 `compact` 读数的项照原片数表（最大点仍 8 片），各片按各自的读数自报，由服务按上限排。判据与取样点不动，只是同一个点分几片、按什么次序跑。

执行员修订 10（2026-11-13，第四段中间点 `files_100_units_1000_unit_area_2_pow_21__head` 的产物已出、最大点两次都没量出之后；只补装置的诊断读数，读数口径、判据、门槛、取样点、臂、片的切法与第六、七、九节各条的去留与报告方式一个字没动；依据：派发提示「登记修订（写进第十二节，判据不动）」与「为什么平均只用半个核……要量才知道的写成整条命令交回（用中间点量，不用最大点）」）：
- **进程读数**：入口分片那一步经 `scripts/experiment-apparatus/e207_strict_scan_steps_process_sampler.py` 起测试程序，每 300 秒与退出时各往日志打一行 `e207 process …`（这一片的用户态与内核态 CPU 秒、缺页、自愿与被抢的切换、所在 cgroup 的 CPU、内存、读写压力的累计微秒），不进产物、不进判定、不进项键与片键（历史用例一个字节没改）。用来分最大点「平均只用 0.54 个核」是等 CPU、等换页与回收，还是等读写。
- **最大点放不进 7200 秒、这一次没改**：每次发布的用时随已放的发布数涨，主要在被量的实现每次发布里整份拷分配器与逐轮比两版分配记录（第三节读数的取样位置里的 s1、s24 那几处，元素数随单元数线性涨），实现一个字节不改（第五节「装置写在哪」），装置里消不掉；跨片续放要把挂着的会话与插进副本的读数状态整份存下再接上，仓里没有存读入口，经重挂载重建会多出挂载时的发布、读数与不分片时不同。所以最大点的切法与片数照旧，交主 agent 定（执行员报告 `/home/fy5090/.local/share/singlefs/work/e207-b8-strict-runner-24/result.md`）。

执行员修订 11（2026-11-13，设计员修订 3 的两份阳性对照原始输出已跑过、对拍读不进（任务 15866、15867 退 2）之后，重出之前；只改入口 `--break-history` 那一路产物的形态，判据、门槛、取样点、臂、谓词与结局表与第六、七、九节各条的去留与报告方式一个字没动；依据：派发提示「改到这两条路出的是合规产物（去掉前缀、只留产物行）」与「登记第十二节补一条执行员修订：`--break-history` 那一路的产物形态改了（去 libtest 前缀、只留产物行，原始输出另存），写明时点在两份阳性对照原始输出跑过之后、判据不动、旧两份原始输出不读不进集合」）：
- **产物形态**：入口 `scripts/experiment-apparatus/e207-strict-scan-steps-run.sh` 的 `--break-history` 那一路单线程起测试程序（`--test-threads=1`），libtest 在第一行输出前接「test <用例名> ... 」，原来又把 `cargo test` 的整份输出（含编译输出）当产物写，读数器按行首认 `E207RESULT`，读不到 `name=point`。改后 `break_history_product` 去掉前缀、只留以 `E207RESULT ` 起头的行写进产物 `.out`（先写 `.partial` 再换上），原始输出另存同目录 `.log`；产物第一行是 `E207RESULT name=point`。平常的片（`--test-threads=2`）与分片那一路本来就是这个形态，没改。
- **读数口径不动**：`caught=` 的判法、设计员修订 3 阳性对照 ①②（第 585 行）要的读数（① `placement_gap` 的 `negative ≥ 1`、打 `caught=yes`；② `first_differing=remounted/W2/5`）一个字没动。
- **旧两份原始输出**：`research/results/e207-strict-scan-steps-base-head-break-slots_placed_undercounted-2026-11-13.out` 与 `research/results/e207-strict-scan-steps-base-plain-break-image_witness_flipped-2026-11-13.out` 是改前的原始输出，一个字节没动、留着，不读、不进任何集合；对拍读照改后的入口重出的 `…-2026-11-13-r2.out` 两份。
- **自证**：自证格 `a_break_history_product_keeps_only_the_result_lines_and_starts_at_the_point` 与弄坏开关 `break-history-prefix-kept`；它们的结局记在实验页（`.claude/kb/experiments/207-严格判据下的扫类步数.md`「第四段现状」末条），不在这里转述。

设计员修订 2、3、4 读过的文件与跑过的命令（`$D` = `/home/fy5090/.local/share/singlefs/work/e207-revision-designer`，在仓根起）：
- 读过：`.claude/agent-common.md` 第 1–83 行；本登记第 1–60、170–299、296–445、505–543 行；`research/results/e207-strict-scan-steps-report-first-2026-11-13-r3.out` 第 4552、4980–4995 行与 `grep` 命中行（`name=cell`、`name=op_row`、`name=op_pooled_fallback`）；`research/results/e207-strict-scan-steps-files_1000-{head,plain}-2026-11-13.out`、`…-files_1000-head-2026-11-12.out`、`e206-fixed-point-cost-files_1000-p2a-2026-11-09.out`（`name=settle`、`name=state`、`name=image_sha256`、`name=reconciliation` 行）；`research/prompts/e203-b8-criterion-r1-forks.md` 第 7–12 行；`records/2026-10-13-里程碑四预开发.md` 第 1655–1660 行（用户定案的原话与上下文）；`scripts/experiment-apparatus/e207_strict_scan_steps_histories.rs` 第 45–58、60–180、625–770、845–1035 行；`scripts/experiment-apparatus/e207_strict_scan_steps.rs` 第 436–475、630–640、968–983、1286–1330、2025–2135、2215–2330 行；`scripts/experiment-apparatus/e207_strict_scan_steps_window_choices.rs` 的 `grep` 命中行；`scripts/experiment-apparatus/e207-strict-scan-steps-run.sh` 第 1–30 行与 `grep` 命中行；`scripts/experiment-apparatus/e207_strict_scan_steps_counters.py` 的 `grep` 命中行；`research/prompts/e207-preregistration.md` 第 473–505 行（执行员修订 4–7）。
- 命令（原样）：`python3 -I $D/calc/reconciliation_fields.py /home/fy5090/code/singlefs > $D/calc/reconciliation_fields.out`；`python3 -I $D/calc/k100_window.py research/results/e207-strict-scan-steps-report-first-2026-11-13-r3.out > $D/calc/k100_window.out`；`awk -f $D/calc/series_cases.awk > $D/calc/series_cases.out`；`sha256sum crates/singlefs-core/src/allocator.rs crates/singlefs-core/src/transaction.rs crates/singlefs-core/src/allocation_record_tree.rs`；`grep -h name=source_sha256 research/results/e207-strict-scan-steps-*-head-2026-11-13.out research/results/e207-strict-scan-steps-*-head-*-2026-11-13.out | sed 's/.*name=source_sha256//' | sort | uniq -c`（15 份产物头相同）；`git log --all --format=%h -- crates/singlefs-core/src/allocator.rs` 逐提交 `git show <提交>:<文件> | sha256sum` 与 E206 产物头的值比（命中 `b14b6fe92`）；`git diff --stat b14b6fe92 8db07de9d -- crates/singlefs-core/src/allocator.rs crates/singlefs-core/src/transaction.rs`；`grep "name=op_pooled_fallback item=files_1000__head" research/results/e207-strict-scan-steps-report-first-2026-11-13-r3.out | awk …`（统计 `pooled_fallback_calls` 为 0 的行数：162）；`sha256sum` 第一段 `files_1000_units_100`、`files_100_units_100`、`files_1000`、`base` 四份产物。


## 十四、并行与挂钟

独立单位：每个 `<点>__<臂>`（含探针）是一个单位，一条用例、一条判定、一份产物 `research/results/e207-b8-strict-<点>-<臂>-<日期>.out`，两两独立；同一单位里历史、两种挂载状态、扩展窗口共用前面那一段历史，不拆。回扫（只读 40 份产物）、模型单测、变异各是一个单位。
线程：取登记值 `.claude/heavy-thread-default` 的 31；用例按单位并行（`cargo test` 的测试线程取登记值），编译照服务给的线程；模型与变异照 `mutate.sh` 的并行度。
串行：同一单位里历史各步、W1–W7、两种挂载状态串行，因为后一步读前一步留下的那一版，次序效应（`first` 对 `later`）本身就是被量的；单位之间没有共享状态，也不量计时，都能并行；两趟编译（带计数与 plain）各一次，没有次序；第二段读第一段 `standard` 的复现钉，所以第二段在第一段之后。
预计挂钟：2 趟编译 × 20 分钟（推的，同 E206 登记第十四节）≈ 40 分钟（两趟可并行）；第一段单位：`base` 2 分钟、`files_1000` 3 分钟、`files_10000` 8 分钟（E206 登记同点估 6 分钟，加声明行）、`files_1000_units_100` 加扩展窗口约 14 分钟（E206 估 10 分钟，加 W5–W7 约 400 次发布 × 0.5 秒 ≈ 3–4 分钟）、`files_100_units_100`、`files_10_units_1000` 各约 4 分钟、`files_100_units_1000`、`files_10_units_10000` 各约 6 分钟、`files_then_big_1000` 3 分钟、`files_then_big_10000` 约 10 分钟、两个几何点约 6 分钟与 15 分钟、plain 两项各约 3 分钟；最长单位约 15 分钟；第一段整趟 ≈ 编译 40 分钟 + 15 分钟 ≈ 1 小时；第二段三个探针项各约 14 分钟，并行 ≈ 15 分钟（编译复用）；模型加回扫加变异 14 条 ≈ 20 分钟；第三段 `files_30000` 加扩展窗口约 90 分钟；第四段（用户定的大点）：`files_10_units_1000_unit_area_2_pow_21` 约 5 分钟、`files_100_units_1000_unit_area_2_pow_21` 约 8 分钟、`files_1000_units_1000` 约 0.7–7 小时（点估约 2 小时，依据见第五节「大点可达核过」④：约 4000 次发布乘每次 0.6 到 6 秒，加量窗 160 次操作约 10–20 分钟，推的，没量）；第四段挂钟由大点定。第一至三段单个单位都不到 2 小时；第四段的大点可能超 2 小时，见下面「切分与续跑」。
预计在用核数：第一段开头编译约 31 个核（服务给的线程），之后 14 个单位排开，用例本身单线程，≈ 14 个核，约 15 分钟后落到最长的 1–2 个核；第二段 3 个核；第四段三个单位并行 3 个核，两个小点十几分钟后结束，大点独占 1 个核到末尾；回扫与模型 1–2 个核，变异按 `mutate.sh` 的并行度；依据：用例单线程不另起线程（推的，没量过）。

单态耗时的依据（都是推的）：沿用 E206 登记第十四节「单态耗时的依据」，新点的发布数都比 `files_10000` 少一个数量级以上，只有 `files_then_big_10000` 与它同量级；扩展窗口的每次发布取量窗每次发布 0.3–0.5 秒的上沿。第一段跑完由执行员把每项用了几秒写进报告，第二段之前主 agent 可以照它重估。

### 单次任务的挂钟与内存（推的）

| 任务（`--item` 或 `--segment`） | 挂钟 | 峰值内存 | 单条过 2 小时吗 |
|---|---|---|---|
| 单个 U = 2¹⁷ 的点（`base`、`files_1000`、`files_10000`、FTB 两点、`..._unit_area_2_pow_17`） | 编译 + 2–10 分钟 | 不到 4.3 GiB（单元区写满的上界；只算写到过的槽 × 2 块盘 × 16 KiB，加分配器 2 份，实际小得多） | 否 |
| 单个 U = 2¹⁹ 的新点与 `files_1000_units_100` | 编译 + 4–14 分钟 | 上界约 17 GiB（实际按 10⁵ 个单元 + 元数据，约 3–5 GiB，推的） | 否 |
| `..._unit_area_2_pow_20` | 编译 + 约 15 分钟 | 上界约 34 GiB | 否 |
| `--segment first` 整趟 | 约 1 小时 | 各单位并行时上界合计约 150 GiB，由服务分机与放大 | 否 |
| `--segment second` 整趟 | 约 15 分钟（编译复用） | 三个 U = 2¹⁹ 的探针项，上界合计约 51 GiB | 否 |
| 第四段 `files_10_units_1000_unit_area_2_pow_21`、`files_100_units_1000_unit_area_2_pow_21` | 编译复用；5–8 分钟 | 写到过的槽约 0.3 GiB、3.3 GiB（上界各 64 GiB 每份，单元区只写到过的槽才占内存） | 否 |
| 第四段 `files_1000_units_1000`（U = 2²¹） | 编译复用；0.7–7 小时，点估约 2 小时（推的） | 写到过的槽约 33 GiB；复制盘（取镜像、重挂载）最坏约 66 GiB；本机内存 60 GiB，装不下记「本次未跑」 | **可能**（上界约 7 小时） |
| `--segment fourth` 整趟 | 同大点（两个小点并行，被它盖住） | 合计约 37 GiB，最坏约 70 GiB | 可能 |

**切分与续跑**：分片单元是 `<点>__<臂>`，分片交服务；续跑入口是 `e207-b8-strict-run.sh` 同一条命令重起，判定库里键相同、做完了的单位不重跑（形态照 E206 入口）。第一至三段单个单位都不到 2 小时，不需要单元内部切分。第四段的大点可能超 2 小时，单元内部切不开：历史与量窗在同一次挂载里，状态在内存里；要切就得把历史前缀的盘镜像与挂载状态落盘再续接（缺的就是这一段，我没查仓里有没有现成的存读入口，估的），登记不另写。所以大点整条跑，主 agent 按 `--long-run-confirmed` 交；分片仍是 `<点>__<臂>`，第四段三个点分三个任务，大点没跑完不挡另两个小点，也不挡第一至三段；续跑入口同一条命令重起，只补没做完的单位，大点被停则整条重跑（它是一个单位）。

### 跑法（入口与复跑）

判定入库、单跑写在第五节。整段经统一任务服务由主 agent 交，工作目录放共享根（`NFS_SHARED_ROOTS`）下，形态：

```text
SINGLEFS_RUN_SCOPE='种类=items; 跑什么=E207 B8 严格判据第一段全部项; 范围=第一段全部项; 为什么=答岔路单行一、行二，补行三的一半' task-server/task submit -- bash scripts/experiment-apparatus/e207-b8-strict-run.sh --segment first --work <共享根下新工作目录>
SINGLEFS_RUN_SCOPE='种类=items; 跑什么=E207 单跑一个取样点; 范围=1 项; 为什么=单跑这个点核读数' task-server/task submit -- bash scripts/experiment-apparatus/e207-b8-strict-run.sh --item files_1000_units_100__head --work <共享根下新工作目录>
SINGLEFS_RUN_SCOPE='种类=items; 跑什么=E207 大点三项; 范围=第四段全部项; 为什么=补行一、行二的 uK 档，总量跨度一百倍' task-server/task submit -- bash scripts/experiment-apparatus/e207-b8-strict-run.sh --segment fourth --work <共享根下新工作目录>
task-server/task submit -- bash scripts/experiment-apparatus/e207-b8-strict-run.sh --selftest
```

脚本里起的那一条是 `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=E207 历史用例; 范围=1 个测试目标; 为什么=量固定点逐轮读数与 B8 输入，只跑这一趟判定库里没有的' cargo test --release -p singlefs-verification --test e207_b8_strict_histories -- <用例名> --exact --nocapture`（写法同 E206 入口那一条）；模型复跑进 `scripts/experiment-apparatus/replay.sh`，对已落盘的产物 `exact`。探针项的项名为 `files_1000_units_100__head__cold_first` 这一类（项名里带探针，键里带探针）。
