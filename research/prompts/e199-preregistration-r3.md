# E199（发布照抄上一版的代价） 跑前登记（第三份）

写于 2026-11-03，装置写之前。

**取代哪一份**：`research/prompts/e199-preregistration-r2.md`（第二份，sha256 `fc9195ecf530988edbbe6dc770dfa6738532c5f135fba6ae82e77981b46abdbd`）；第二份取代的是 `research/prompts/e199-preregistration.md`（第一份，sha256 `8b9ae1ac1547fdcfc624097a291a98defc0ed932d9a6c79fe005d3642c4e8612`）。两份都是冻结证据，一个字不改；执行员从这一份开工，前两份的判据不再单独判。

**为什么**：第二份第一段执行员开工核对时打中第十一节停机条款第 2 条与第 1 条 ③（执行员报告 `~/.local/share/singlefs/work/e199-runner-segment-1-r2/result.md`，sha256 `3231fc28a28a3e7534bd79049bb2778aa9fe77e6207a2da0bccf242f91dbf224`，「两边」「停机条款第 1 条」「停机条款第 2 条」三节）：容器小树代价第二件（B 簇第二件）打进主工作区之后，「照抄那一支」每次发布仍抄一份上一版的形状——`transaction.rs:8640` `tombstone_shape_before.clone()`（墓碑树不追加那一支）与 `transaction.rs:8965` `shape_before.clone()`（谱系树不追加那一支）——第二份把它们放在甲-B2后的「去掉」一列、按零份算。同一份报告另报停机条款第 1 条 ② 原样做不了：第二份引的两位设计员的草稿目录、`callgraph.py` 与六份旧输出都已不在。那一段没写装置、没落产物，判据一次都没判过。

**这一棵树**（查的时刻，2026-11-03）：主工作区 `git rev-parse HEAD` = `59ce5a86550de2c2c9bc6b51b34b4f881313682c`；`crates/singlefs-core/src`、`crates/singlefs-format/src`、`crates/singlefs-replay-prefix/src` 下相对这个提交改过的文件 40 份（`git diff --name-only HEAD -- <三个目录>`，暂存的新文件在内），sha256 逐份列在第十三节「这一棵树」。另一路代码轮 m4-close-code-r2 在改同一批产品代码；这一份引的行号都是这一棵树的，执行员开工照第十一节停机条款第 1 条在那时的主工作区现查。这一份落盘之前又核了一遍：三个目录下 49 份 `.rs` 的 sha256 与开工时逐份相同（第十三节）。

**改了什么**（逐节写明，下面只列总账）：

1. **甲-B2后补进两处形状照抄**（主 agent 定，派发提示第 1 条）：`transaction.rs:8640`（墓碑树不追加那一支，每次发布 s + 1 遍，只在不删除的发布上走到）进 O，`transaction.rs:8965`（谱系树不追加那一支，每次发布 s 遍，G0 走到）进 Rl。第三节两张表与「去掉 / 留着」、第五节 O、Rl 与甲-今天的式子、第七节锚点、第九节变异、第四节同一组数跟着改。
2. **全扫重做**（主 agent 定，派发提示第 2 条）：调用图脚本重写（源码整份抄在第三节「全扫」），在这一棵树上扫三个目录（第二份只扫 `crates/singlefs-core/src`），准入前、准入后两段各三组写法，202 条命中逐条归属；命令、原样输出与归属表整份写进第三节。基线里没改过、现在连进发布路径的函数照样收进（第三节列今天比基线多收的函数）。同名误收另用去边复核（第三节）。
3. **全扫新补两项进甲-B2后**（设计员按全扫归属补，交主 agent 认）：`transaction.rs:14643` 第一个文件照抄的数据指针（每次发布 88·D），`transaction.rs:14640` 第一个文件照抄的数据单元字节抄进 `data_units` 那一份（这些单元在 `16009` 经 `carried_unit` 再抄一份；第二份的式子 (L_I − 1 + D)·32816 只算了后一份；这一份每次发布 32792·D）。两项只在 D > 0 的点上不为 0：第七节 G0 与其余已钉的点上一个数都不因它们变。
4. **行号整体挪了**（主 agent 定，派发提示第 3 条）：第三节各表改成这一棵树的行号（逐行现查，第十三节），并进第十二节修订。
5. 第二节照第二份的四段取法用 `scripts/kb/quote-kb.py` 重抽，内容逐字相同；D18（块里携带什么信息） 已定项 24 的行区间从 578–610 挪到 580–612。
6. 第十节第一行核对命令里 `entries == 135` 在 `container_tree.rs:173`（第二份写 169）；`&TransactionOutput` 现跑 469 处（第二份 459）。
7. 第十一节停机条款第 1 条改成照这一份第三节抄的脚本原样重跑、与这一份第三节的输出 `diff`；第 2 条改成在执行员开工那一刻的主工作区核「去掉 / 留着」两列（B 簇第二件已打进主工作区，补丁 sha256 `5180b75835fe157f20bb07dbdc586ad6a25563f132bfaca29466a7f36ab92ad2`）。

**只收严，不放宽**（`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「臂的定义也在「跑前写死」之列——失败条款打中的时候怎么办」）：

- **记一次输**：打中的是停机条款，不是失败条款；按最弱读法记：第二份的甲-B2后少数了照抄点，在「甲的照抄字节由第三节那张表数全」这一格上不成立（补丁之后不追加那一支仍每遍抄一份上一版形状，第二份按零份算），第二份的第一段不跑、也不判。
- **只许收严**：第二份的每一条臂、每一个门槛、每一条失败与作废条款照留；臂的定义只多不少——甲-B2后多数 `8640`、`8965` 两处形状与全扫新补的 `14640`、`14643` 两项，乙2、丙、丁对这几样各写明怎么做（第五节）；判据一个字没改。被第二份判据判过的格，新形态下判得同样严或更严：甲的照抄字节只增不减（G0 准入前 +128，不删除的发布准入前 +808，D = 32768 准入后 +1077411840），乙2、丙、丁的照抄字节不变，「甲 ÷ 臂」的分子只涨。
- **写明收严在哪**：甲多要求「不追加那一支每遍抄一份上一版形状算进去」「第一个文件照抄的数据单元两份、数据指针一份都算进去」；全扫多要求「三个目录、按函数逐个收、同名误收经去边复核」；停机条款第 1 条多要求「脚本源码与原样输出都在登记里，照原样重跑」。

## 一、问题

英文名：transaction_output_copy

主 agent 这一次给的问题（派发提示「关的是哪几条」一节逐字）：

> 第一段执行员开工核对打中第二份登记的停机条款（执行员报告 `/home/fy5090/.local/share/singlefs/work/e199-runner-segment-1-r2/result.md`，sha256 3231fc28…，第二份登记冻结不改）。写第三份登记 `research/prompts/e199-preregistration-r3.md`（排他新建），逐条验收：
>
> 1. 停机条款第 2 条与第 1 条 ③：容器小树代价第二件打进主工作区之后，「照抄那一支」每次发布仍抄一份上一版的形状——`transaction.rs:8640` `tombstone_shape_before.clone()`（墓碑树不追加那一支，每次发布 s + 1 遍，只在不删除的发布上走到）与 `transaction.rs:8965` `shape_before.clone()`（谱系树不追加那一支，每次发布 s 遍，G0 走到）。第二份登记把它们放在甲-B2后的「去掉」一列、按零份算。主 agent 定：补进甲-B2后（只给甲收严），第三节两张表与「去掉 / 留着」、第五节 O 与 Rl、第七节锚点、第九节变异跟着改，第四节同一组数跟着改；改动照 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「臂的定义也在跑前写死之列」记一次输、只许收严、写明收严在哪。
> 2. 停机条款第 1 条 ②：第二份登记要的全扫与调用图原样复现不了（前两份的草稿目录、`callgraph.py` 与六份旧输出都不在了）。主 agent 定：不认执行员的 grep 替代做法，第三份登记里重写调用图脚本、重做一遍发布路径全扫（命令、原样输出、逐条归属都写进登记第三节「全扫」一节），覆盖基线里没改过、但现在连进发布路径的函数。
> 3. 行号整体挪了：照执行员报告「行号挪动」一节改第三节各表的行号（今天主工作区现查），并进第十二节修订。
> 4. 代码轮 m4-close-code-r2 在攻同一批产品代码；登记里引的行号以主工作区为准，开工时现查一遍，把查的时刻那一棵树写进登记（`git` 提交与相对它的改动文件 sha256）。

被测条款（派发提示「被测条款」一行逐字）：同第二份登记 `research/prompts/e199-preregistration-r2.md` 第二节（`.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」与它引的 D16（发布语义）、D8（核心索引结构） 分项），照它整节抄，不改口径。

这个实验要回答的问题照旧是第一份第一节「主 agent 给的问题」那一段（`research/prompts/e199-preregistration.md` 第 11–16 行）与第二份第一节两段派发判定（`research/prompts/e199-preregistration-r2.md` 第 31–46 行），这里不重抄。被测条款的读法同第二份第 52 行：`.claude/rules/fs-design.md` 第 20–27 行那张表运行时决策路径一格（「不许遍历，且代价不许随盘容量增长」），用户 2026-10-31 定案读成也管随容器总数、记录总数涨、只管准入之前；D8（核心索引结构） 已定项 16；被量的是准入之前与之后照抄上一版 `TransactionOutput` 的那几步，另量同两段里每次按全部单元现算重建的那几步。

### 读法（写死；第二份第一节十条照留，只改行号与第 2 条点名的行）

1. **「每次发布」**：一次 `publish_*` 从拿到上一版到交回这一版（`mounted_session.rs:1943` `publish_the_change_once` 一次）。分两段：**准入之前** = `judge_the_publish_before_any_read_or_write`（`transaction.rs:12489`）整段：`12510-12512` 那一次 `ordinary_allocation_part`、`settle_the_allocation_record_tree`（`13017`）里 s 轮 `plan.resolve`（`13063`，每轮里再跑一次 `ordinary_allocation_part`，`8729`）、准入（`12621`）；**准入之后** = `publish_admitted`（`14656`）装单元、装输出。两段各报各的，任何一格都不把两段加起来判。
2. **「照抄字节」**：第三节「甲臂的照抄点」那张表列的每一处在内存里再复制一份的字节数（复制的是上一版已有的内容，或这一版里装着上一版内容的那一份）。**「新装字节」**：这次发布新造的内容（追加的记录、重写单元的新字节、新分配记录、这次新加的映射条目、这次的记账行）——四条臂一样多，单列，不算照抄。**「新装第二份」**：甲把新装的单元字节在装进 `units` 时再抄一份（第三节全扫归属表里标「新装第二份」与「照抄·进模型 + 新装第二份」的那几行；`16562` 把要写的单元再抄进 `PublishWrites` 那一份也归这一类），只给甲报一行附带量，不进照抄、不进新装、不判。
3. **「内存」**：模型里每样东西的内存宽度是第五节「模型常量」那张表的数（装置自己的类型，`size_of` 回比），不是 `crates/` 里 Rust 类型的真宽（装置不引实现代码，见第十一节停机条款第 3 条）。
4. **「模型挂钟」两种量都报，各占各的行**：推算挂钟 = 第一段的字节 ÷ 第二段在本进程里量的拷贝带宽；执行挂钟 = 第二段在装置里真建四条臂的内存结构、真做一次发布，在被测进程里用 `std::time::Instant` 计（`.claude/singlefs-ai-sop/rules/test-discipline.md`「分段计时在被测进程里计，不在转发输出的循环里计」）。两者都不是 `crates/` 实现的挂钟。
5. **「10⁶ 次 unlink 之后的形状」**：墓碑树记录数 R_T = 10⁶；C_T、各层节点数、N_T、映射树与分配记录树的形状由第五节公式在模型里现算，不取调查报告的数（调查报告的 C = 1227、N = 11 列在第四节当「已存在的数」）。
6. **取值范围**：u（= R_T）∈ {0, 10, 10³, 10⁴, 10⁵, 10⁶, 10⁷, 10⁸}（第一段全取，第二段取到 10⁶ 加第八节两处等高点）；每次追加墓碑条数 a_T ∈ {0, 1, 8150, 81500, 815000}（k = 0、1、11、101、1001，跨三个数量级）；映射树叶的装满程度 φ ∈ {1, ½}；映射树与分配记录树被碰到的叶取「分散」「集中」两档（第五节）；其余旋钮见第八节。
7. **比值的分母**：甲对乙2、甲对丙、甲对丁**逐对**各算一个比值、各占一行（不取「乙丙丁里最省的那一个」）；比值 = 甲的量 ÷ 那一臂的量，同一几何、同一段。
8. **「现算重建字节」**：第三节「现算重建」那张表列的每一处，按这一版全部单元（或全部 key）现算一份与上一版里相同的内容的字节数——不是从上一版拷来的，是从这一版的指针、记录重新算出来的。单列成第三类，准入前、准入后各报，不并进照抄、不并进新装，判据另占几行（第六节）。两张表之外的拷贝不计，归属写在第三节全扫归属表里，一行一条。
9. **字节比的两种形式都留、各判各的**（主 agent 2026-10-31 判定）：形式 A = 照抄字节之比；形式 B = （照抄 + 新装）字节之比。同一格（同一对、同一段、同一几何）里两种形式各判一次；两种形式落在门槛 2 的两侧，那一格记「不稳定」，不下「倾向留甲 / 不倾向留甲」。形式 A 分母为 0 时报「分母 0」、判 ≥ 2。现算重建字节比（第六节）只有一种形式（现算重建之比），分母为 0 时同样报「分母 0」、判 ≥ 2。
10. **判定只用甲-B2后**；乙行判用乙2，乙1 只报数、不判；第三段（入库装置与实现对拍）不做，第一份第五节末尾那一段作废。

### 岔路单（逐行照抄 `research/prompts/m4-transaction-output-copy-forks.md`）

问题：每次发布，上一版 `TransactionOutput` 在内存里完整照抄一份（容器树全部容器与记录、形状、单元字节、inode 树、分配记录），代价随盘上容器总数、记录总数涨。用户 2026-10-31 定案：条款「代价不许随盘容量增长」（`.claude/rules/fs-design.md:22`）只管准入之前；容器小树五处与准入增量维护这一批改，根源「先量」。出处：`~/.local/share/singlefs/work/m4-close-container-cost-investigation-2/result.md` 第三问。

| 候选 | 翻面观测 | 够判条件 | 状态 |
|---|---|---|---|
| 甲 今天的做法：每版完整一份，发布时照抄上一版 | 甲的每次发布内存拷贝字节数、时间随容器总数 C 与记录总数 R 的增长阶，与乙丙丁在同一组 C、R 取样点上之比；比值在用户关心的规模（10⁶ 次 unlink 量级）上低于 2 倍则倾向留甲 | 四条臂在同一组 C、R 取样点（至少跨三个数量级、含 10⁶ 次 unlink 之后的形状）上都有每次发布的拷贝字节数与挂钟，绝对值断言钉住 | 开着 |
| 乙 版本间共享：没改的容器、记录、单元在相邻两版之间共享（引用计数或持久结构） | 乙每次发布的拷贝量只随这一次碰到的容器数与树高涨；另量共享带来的读路径额外间接与内存峰值 | 同上，另有读路径的每次查找代价与常驻内存峰值 | 开着 |
| 丙 上一版按值交进来：发布消费上一版、就地改出新一版，不留两份 | 丙每次发布的拷贝量；丙要求调用方不再持有上一版，量出哪几处今天持有上一版（回退、checker、崩溃判器）要改 | 同上，另列出今天读上一版的全部调用点（文件:行）与改法 | 开着 |
| 丁 只记差量：一版只存相对上一版的改动，读时合成 | 丁每次发布的拷贝量与读一次要合成的层数；层数随发布次数涨时要另算压实代价 | 同上，另有读路径合成层数与压实频率 | 开着 |

第二份没有未定项，这一份同样没有：新补的两处形状、两项数据单元照抄都按第五节式子现算出来了（第四节、第七节），第 3a 步三样都算得出。

## 二、被测条款与它引的定义

`scripts/kb/quote-kb.py` 照第二份的四段取法整段重抽（出口 `~/.local/share/singlefs/work/e199-designer-revision-3/quotes.md`，在仓根直接起，命令在第十三节；日志一行「✓ 4 段整抄进 …/quotes.md，回读逐字节一致」，退 0；出口 123 行，sha256 `4014c802a194f096d02caa1e3d359bb3d4ca8bc85047c2a8c13ea8a6c6761905`）。与第二份第二节抄录（`e199-preregistration-r2.md` 第 85–207 行）`diff` 只差一行：D18（块里携带什么信息） 已定项 24 那一段的出处行区间从 `578-610` 变成 `580-612`，抄出的正文逐字相同。取法与理由照第二份第二节第一段：被测条款一节、D8（核心索引结构） 已定项 16、模型常量的出处 D8（核心索引结构） 已定项 2 与 D18（块里携带什么信息） 已定项 24；D18（块里携带什么信息） 退役记录宽 34、959 条与码 2 头 115 + 2 × key 宽两处只引行、不整节抄；多层码 2 树的节点容量、条目宽与 key 字段表与这一份新加的数据指针宽 88 取实现今天的常量，不出自被测条款，归「模型选择」，开工时按第十节第一行核。

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

**射程**：第一版按 inode 号删任一 inode、无目录（用户 2026-09-28 定的是单文件、按 inode 号、无目录、不带 truncate，任一号由多文件删除的判决放开）；截断与变长不走这一条，走已定项 5 的截断意图（判决 `research/prompts/m4-trunc-r1-main-verification.md`），照这一条的走空与幂等纪律；删完第一个文件之后（K-U5，记欠账）按 inode 号写数据只拒对象出生代晚于 inode 1 墓碑死亡 txg 的 inode，删之前就有的别的文件照常能写；取这个死亡 txg 只认这一版父链上的墓碑：「父链」照 I-9.18（删除队列里的号有谱系上的墓碑） 的定义，从这一版沿谱系记录往上走，只排掉被这条父链上的回退抛下的那几段 (T_old, T_rb) 开区间里的死亡 txg，不在父链上的谱系记录不排，再取最大；inode 1 的记录还在时不判；这一读法依赖墓碑第一版不回收（D18（块里携带什么信息） 已定项 24），墓碑回收落地时要换读法（C618（删完之后不能再建带数据的文件））；崩溃流接在 `crates/singlefs-checker-tier/tests/crash_enumeration_multi_record_publish_stream.rs` 那条流后面 unlink 它的 3 单元文件（K-U6 F1），截断成 0 另立一条流。只改尾叶的 extent 分支、写入类发布按发布之后的标签判、重写角色集合钉绝对值被攻过零轮；第 6 条增补的会话里每次用户改动先走空、与任何一版至多一条 key，也被攻零轮。`.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」那张表里运行时决策路径「代价不许随盘容量增长」，在这一格读成只管准入之前（用户 2026-10-31 定，原话在变更史）：准入之前的步代价不随盘上容器总数、记录总数涨；准入之后装单元、拷字节不算运行时决策路径。

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

**出处 `.claude/kb/decisions/08-核心索引结构.md:43-58`（整段抄，未转述）**

```markdown
#### 已定项 2：节点大小 16 KiB，钉成常量

**定案**：**节点大小 16 KiB。mkfs 把 16 KiB 写进系统配置，实现按常量走，不按设备特性算**——格式里是一个常量，不是一条公式。
<!-- format-const: NODE_BYTES = 16384 -->

**射程**：⚠️ **登记 `format-const` 不等于冻结这个数**（2026-08-31 用户定案：「这个数需要可以调整」）。那条标记只保证 **kb 与实验源码写的是同一个数**，改它照常走 D8（核心索引结构） 的流程；它写进系统配置，仍然是 mkfs 参数。⚠️ **它现在只钉得住一部分源码**：门禁阶段「格式常量在 kb 与实验源码之间同步」（`.claude/gate.d/code-source-discipline.sh` 的 format-constants 格）的形态是 `const 名 = 整数字面量`，`e56_epsilon.rs` 的 `16 * 1024`、`e43_extension_point_budget.rs` 与 `e52_head_mechanisms.rs` 的数组形式都不认。⚠️ **16 KiB 是 CPU 缓存轴上的选择，不是设备 I/O 轴上的**：E7（离线索引 harness） 量设备 I/O 时说小节点两端都赢、方向相反，两者量的是不同资源，不矛盾。上限那一维归已定项 4；E1（节点大小 vs 可恢复性） 的恢复时间那一半一旦给出比 16 KiB 更紧的上限，本项重开。

**依据**：

- E20（指针变宽的 CPU 缓存代价）：唯一超出 L3 的工作集上 16 KiB 是最小点；同一实验里 40 字节的指针头部在 4 KiB 节点付出的相对代价远高于 16 KiB 节点，大节点吸收指针变宽；同一条曲线上 8 KiB 的反常六轮稳定且无人解释 ⇒ 在解释掉它之前不许把节点大小写成「按设备特性算」。
- E6（加密算法选型）：加密 / 校验单元的增益在 16 KiB 之后基本饱和。
- E73（节点 key 区间的扇出代价）：码 2 头带上 key 区间之后树高一格都没涨，16 KiB 不用重开。
- 用户定案 2026-08-30：16 KiB 钉成常量（原话在变更史）。

**欠**：C56（没登记的格式常量仍会漂）、C386（节点大小结论欠一次按最终指针宽度的复跑，没有编号）。

```

**出处 `.claude/kb/decisions/18-块里携带什么信息.md:580-612`（整段抄，未转述）**

```markdown
#### 已定项 24：第一版墓碑：墓碑树、40 字节记录、谱系记录

**定案**：

1. **墓碑树**：一棵专用树，day-1 零根注册进树表；根恒为码 2，条目照 inode 树内部节点的 120 字节形态（分隔 key 容器号 8 + 身份引用 26 + 子指针 86），孩子是码 3 容器（打包记录类型 1 墓碑）；容器与根进中央映射，条目里的指针是位置提示；根满时长高，照 D8（核心索引结构） 已定项 6，实现要做这一步（用户定案 K-R3-3）；容器号从 1 起，下一个号 = 树里最大号 + 1，零根时 1（与已定项 11 里克隆头的类型 1 容器号从 1 起按树内单调分配同一条规则）。墓碑树根的码 2 头层级取容器树的层数：根直接管码 3 墓碑容器时 1，根满长高一次加一（`crates/singlefs-core/src/container_tree.rs` 的 `ContainerTreeShape::root_level`）；池级 checker 按根层级 ≥ 1 读墓碑树（`crates/singlefs-checker/src/walk.rs` 的 `largest_published_tombstone_object_identifier`）。
2. **墓碑记录**：对象 ID 8 + 对象出生代 8 + 区间起 8 + 区间止 8 + 死亡 txg 8 = 40 字节；整对象写 [0, 2⁶⁴ − 1]；一个 32 KiB 容器装 ⌊(32768 − 136) ÷ 40⌋ = 815 条；死亡 txg = 意图那次发布的 checkpoint_txg。
3. **开放容器**：有新记录的那次发布才重写，不删除的发布照抄；满了才关；关闭路径加只供测试的容量旋钮，旋钮不落盘。
4. **回收**：第一版不回收墓碑，统计量 13、14（D5（快照 / 空间记账机制） 已定项 4）第一版不登记；10⁶ 次 unlink 的墓碑约 76.7 MiB（两份）。
5. **谱系记录**：一棵专用谱系树，day-1 零根注册，随现行那一版、只增不删；孩子是码 3 容器（打包记录类型 5，不复用已定项 11 登记过的类型 3）；记录 = T_rb 8 + T_old 8 + H_cur 8 = 24 字节（T_rb 是回退那次发布的 txg，T_old 是 R_old 的 txg，读作「T_rb 的父是 T_old」；H_cur 是 cur（回退之前的现行那一版）的 inode 号水位，从 cur 那一版读，不从发布路径交来的「上一版」现算；三段的逐字节次序是 T_rb 8 | T_old 8 | H_cur 8，H_cur 接在已有两段之后、旧两段的偏移不动），一个 32 KiB 容器装 ⌊(32768 − 136) ÷ 24⌋ = 1359 条；管理员回退那次发布追加一条；谱系树根满同样长高，容器号从 1 起，下一个号 = 树里最大号 + 1，零根时 1。H_cur 与记账里 inode 号水位的台阶行（D5（快照 / 空间记账机制） 已定项 2 射程）互核：每条谱系记录上，台阶行里代段 ≤ T_rb − 1 的那几行的最大水位**等于** H_cur，H_cur 为 0 也判红（W3 的 C2aL 形态；已立为 I-9.30（inode 号水位的台阶行与水位、记录、谱系对得上），已实现）。有克隆头时这一判与台阶行都按头（带树 ID）：台阶行按 (统计量, 树) 分，建克隆那次给新头留一行；按头之后谱系记录的字节形态没定（推的，第一版单头）。
6. **登记**：树表 7 → 12 条（`deleted_inodes`、墓碑树、谱系树、logged_ops、退役树零根，logged_ops 见 D8（核心索引结构） 已定项 5，退役树见已定项 11 的打包记录类型 3），树 ID 水位 65544 → 65549（推的：五棵各占一个树 ID；树 ID 从 65536 起编，D8（核心索引结构） 已定项 11）；不加新 incompat 位，并进第一条布局线。

**射程**：第一版单头、无快照、无清扫。扫盘重建怎么读墓碑与谱系记录不在这里定，记欠账（K-W5b，与 C113（扫描重建时多版单元的现行版本判定无输入） 同组）。谱系树的载体、打包记录类型 5 被攻过零轮；记录多带 H_cur 的 24 字节形态被攻过一轮（W3 第三轮两次抽样）。

**依据**：

- E83（墓碑的粒度）：第 2 次（墓碑记录 40 字节、记录区起点 136）复出第 2 条的 815 条一容器与第 4 条的 10⁶ 次 unlink 约 76.7 MiB（两份）。
- E102（单元类登记表与打包记录单元）：第 2 次 cap(136, 40) = 815，与第 2 条的 815 对得上。
- 墓碑树、谱系树的其余数：三方腿的模型与副本原型上量的（墓碑判决、m3-delete-r3 判决），还没在入库装置上量过。
- 墓碑树：墓碑判决 `research/prompts/m3-tomb-r1-main-verification.md` 第三节 W1 载体那一行（两轮；`research/prompts/m3-delete-r3-main-verification.md` 第三节「墓碑第一轮一轮各行」那一行把孩子里的类型 3 退役删掉）；根满长高：用户定案 K-R3-3。
- 墓碑记录：墓碑判决第三节 W2 字段表、W2 死亡 txg 两行；死亡 txg 随用户定案 K-W5，m3-delete-r3 判决第三节「墓碑第一轮一轮各行」那一行。
- 开放容器：墓碑判决第三节 W4 开放容器那一行（两轮）；旋钮不落盘：m3-delete-r3 判决第三节 G6-容量旋钮那一行（零轮）。
- 回收：墓碑判决第三节 W3 回收那一行（两轮）；76.7 MiB 是 m3-delete-r3 判决第三节 G8-其余数那一行（40 字节、815 条一容器）。
- 谱系记录：m3-delete-r3 判决第三节 G4-谱系住处那一行（零轮）。
- 谱系记录多带 H_cur、按「等于」判、从 cur 读，有克隆头时按头：W3 判决 `research/prompts/m3-w3-r2-main-verification.md` 第二节攻方 K3 那一条（用「被盖住」比放过谱系写错的两种变异，改成「等于」全红；H_cur 从发布路径交来的「上一版」现算时一处单点变异让两个最强形漏全部 26 410 次）与第三节 C2aL 那一行、`research/prompts/m3-w3-r3-main-verification.md` 第二节克隆头各行（不分头的读法两次抽样都误报，三个形态都要按头）；用户定案 2026-10-08 Y2「C2aL：记账留台阶行（推荐）」，原话在变更史。无实验：数在三方腿的模型与仓副本原型上量，没立实验号。
- 登记：墓碑判决第三节登记那一行（零轮；两棵树时水位 19 → 21，是那一份判决当时的树 ID 编号，树 ID 现从 65536 起编）。
- 登记加退役树（树表 12 条、水位 65549）：判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 1 条与第五节；用户定案「开池就建（推荐）」，原话在变更史。
- 谱系记录三段的逐字节次序 T_rb | T_old | H_cur：第 49 批 W3 实现员取的（实现员报告第二节「谱系记录三段次序」一行，`m3-batch49-w3-implementer-report.md`），主 agent 采纳，零轮。无实验：次序是字节摆放的取值，没有可量的量。
- 用户定案 K-U2「t1：意图那次写整对象墓碑」、K-W5「谱系记录」、K-W5b「写侧随谱系落地，读侧记欠账」、K-R3-3「写根长高（推荐）」，原话在变更史。
- 容器号从 1 起、墓碑树与谱系树根满长高：`research/prompts/m3-blanks-r1-main-verification.md` 第二节容器号、墓碑树根长高两行（一轮）。
- 树 ID 从 65536 起编：用户定案 2026-10-01「丙 树 ID 整体挪开」，原话在变更史。

**欠**：C615（扫盘重建对挂着时的回退没有输入）、C616（谱系父链靠 txg 连号没有断言）、C617（删后回退留下的墓碑没有预付罩着）、C614（回退时墓碑树取错与删完对象无墓碑单镜像判不了）；C84（墓碑单元的粒度没人定） 还清、C103（容器索引无落点） 的类型 1 那半还清、C109（哪些树的 key 死亡要写墓碑） 收窄。

```
## 三、实现今天的样子

读的是这一棵树（文件头「这一棵树」，sha256 在第十三节）：`crates/singlefs-core/src/transaction.rs` `1178862ceaa82964e4a69c815434684ca365380663014658d957b760b8ea0973`（17498 行）、`container_tree.rs` `392e25fc97b8a1b43d41f3f461b29aee43dbadc3eca29b5859580d075062daaa`（4385 行）、`mounted_session.rs` `2b8d27e012b41f4859b0c68e4d1bd82af94d51f2de60f2c57d8f275a99a670a5`；`inode_tree.rs` `69fee61e…`、`code_two_tree.rs` `991fc0de…`、`allocation_record_tree.rs` `f69d2798…`、`crates/singlefs-format/src/lib.rs` `f0a126bc…` 与第二份第三节相同，那几处行号不动。下文行号都是这一棵树的，逐行用 `awk 'NR==行号'` 现查过（第十三节）。

**`TransactionOutput` 一版在内存里是完整一份**：结构体 `transaction.rs:3195-3343`（比第二份多 `inode_record_counts` 与根单元下标两个字段，都不随 C、R 涨）；发布拿 `previous: &TransactionOutput`（`mounted_session.rs:1947`）造整份新版。

**准入之前 `ordinary_allocation_part`（`transaction.rs:8325`）跑 s + 1 遍**：`judge_the_publish_before_any_read_or_write` 在固定点之前先调一次（`12510-12512`，取 `rewritten_roles()` 判落不落得下），固定点每轮 `resolve`（`8718`）里再调一次（`8729`）；`lineage_tree_after_this_publish`（`8734` 调，`8928` 起）与 `plan_the_multi_level_trees`（`8976` 起）只在 `resolve` 里，跑 s 遍。G0 那种 unlink 意图走 `publish_unlink_intent`（`11169`）→ `publish_deletion_step`（`12009`）→ `publish_version_of_trees_relocating_units_after_a_check`（`12018` 调，`12294` 起）→ `prepare_the_version_publish_relocating_units`（`12315` 调，`12659` 起）→ `judge_the_publish_before_any_read_or_write`（`12676` 调），每次都过这一判；`publish_admitted` 在 `12785` 调。

### 甲臂的照抄点（模型的「甲-B2后」按这张表计；「第三份」= 这一份补进的）

| 段 | 来源 | 文件:行 | 照抄什么 | 每次发布几遍 | G0 跑不跑到 | 模型里是哪一项（第五节） |
|---|---|---|---|---|---|---|
| 准入前 | 甲1 | `transaction.rs:8601-8609`（墓碑，`.clone()` 在 `8606`）、`8646-8654`（退役，`8651`） | 上一版每片容器 `contents.clone()` | s + 1（`ordinary_allocation_part` 里） | 跑 | O 部分 |
| 准入前 | 甲1 | `transaction.rs:8934-8942`（谱系，`8939`） | 同上 | s（只在 `resolve` 里） | 跑 | Rl 部分 |
| 准入前 | 原表 | `transaction.rs:8331-8333` → `4208`（`inode_leaf_container_contents`） | 上一版 inode 叶 | s + 1 | 跑 | O 部分 |
| 准入前 | 第二份 | `inode_tree.rs:1093` `copy_of_the_leaf_containers`（`transaction.rs:8376` 调，逐片 `inode_tree.rs:1302`）；最后一批走 `inode_tree.rs:1728` → `1970` `copy_of_one_leaf_container`（`transaction.rs:8367` 调） | 同一次 `ordinary_allocation_part` 里把刚抄来的 inode 叶再整份抄一遍 | s + 1 | 跑（意图走 `1093`；最后一批走 `1970`） | O 部分（inode 叶记两份） |
| 准入前 | **第三份** | `transaction.rs:8640`（`TombstoneTreeAfterThisPublish::carried` 的第二个参数；形状 `Clone` 的实现 `container_tree.rs:1489-1503` → `1505`、逐层 `1520`、逐个节点 `1540`） | 墓碑树不追加那一支照抄上一版形状 | s + 1 | 不跑（G0 是意图，`8616-8629` 走追加那一支）；不删除的发布上跑（`8631-8641` 列的八种步，含最后一批） | O：Σ_{t∈T,X 不追加} shape(N_t, h_t)（退役树每次都追加，实际只有 [a_T = 0]·shape(N_T, h_T)） |
| 准入前 | **第三份** | `transaction.rs:8965`（`LineageTreeAfterThisPublish::carried` 的第二个参数，`8963-8966`） | 谱系树不追加那一支照抄上一版形状 | s | 跑（G0 a_L = 0，`NoLineageRecord`） | Rl：[a_L = 0]·shape(N_L, h_L) |
| 准入前 | 甲4b | `container_tree.rs:3575` → `2648` `copy_of_the_containers`（逐片 `2664`）→ `2627` `copy_of_the_records`（逐条 `2639`） | 追加那一支抄全部容器与记录 | 追加的树每遍：退役树每次（s + 1），墓碑树意图那次（s + 1），谱系树回退那次（s） | 跑 | O / Rl 部分 |
| 准入前 | 甲4c 的一半 | `container_tree.rs:3576` | 追加那一支抄一份形状 | 同上 | 跑 | O / Rl 里追加那一项的 shape |
| 准入前 | 第二份 | `transaction.rs:9050-9056`（`carried`，`.to_vec()` 在 `9055`） | 每个没重写的映射单元的映射 key 从上一版 `mapped_units` 逐个抄 | s | 跑 | Rl 部分 |
| 准入前 | 第二份 | `code_two_tree.rs:650-651` | 上一版形状里全部 key 抄进一个 `BTreeSet` | s，三棵多层码 2 树各一次 | 跑（deadlist 零根那一棵 key 数 0） | Rl 部分 |
| 准入前 | 第二份 | `code_two_tree.rs:390-412`（`397`、`405`） | 从上一版形状建规划树：全部叶 key 与全部分隔 key 再抄一份 | s，三棵各一次 | 跑（同上） | Rl 部分 |
| 准入前 | 第二份 | `transaction.rs:13107`、`13120` → `allocation_record_tree.rs:372` → `311-333`（`327` `push(*record)`） | 上一版全部分配记录按叶分组抄一份 | 每轮两次（s 轮） | 跑 | Rl 部分 |
| 准入前 | (d) | `transaction.rs:13045` `node_set()`（`allocation_record_tree.rs:480`）与 `13051` `previous_nodes.clone()` | 上一版分配记录树的节点集抄两份 | 一次（在 `13053` 起那一轮循环之外） | 跑 | Once 部分 |
| 准入后 | 丙1 | `transaction.rs:14144`（`build_container_tree_units`；调用 `14409`、`14440`、`14473`） | 新版每片容器内容再抄一次 | 一次，三棵树 | 跑 | 准入后 |
| 准入后 | (a) | `transaction.rs:13912`（没重写那一支）、`13947`（重写那一支），`15156` 调 | 新版每片 inode 叶的内容再抄一次 | 一次 | 跑 | 准入后 |
| 准入后 | 丙5 | `carried_unit` `transaction.rs:13272-13277`（`.clone()` 在 `13276`），装 `units` 那一段（`16004` 起）逐角色调它：第一个文件的数据单元 `16009`、inode 叶 `16074`、inode 根 `16090`、删除那两棵树 `16121`、容器树节点 `16136`；索引 `15061` | 没重写的每个单元整份拷单元字节 | 一次 | 跑 | 准入后 |
| 准入后 | **第三份（全扫）** | `transaction.rs:14626-14645` `carried_file_version_units`（`14880` 调）：`14640` 数据单元字节 | 第一个文件照抄的数据单元抄进 `data_units`（这些单元在 `16009` 经 `carried_unit` 再抄一份，那一份在上一行） | 一次 | 跑（G0 D = 0，0 个） | 准入后：32792·D |
| 准入后 | **第三份（全扫）** | 同上，`14643` `carried.data_pointers[..carried_data_units].to_vec()` | 第一个文件照抄的数据指针 | 一次 | 跑（G0 D = 0，0 个） | 准入后：88·D；一版大小另加 88·D |
| 准入后 | 第二份 | `transaction.rs:16097-16111`（`16107` → `carried_unit`） | 分配记录树没重写的节点整份拷单元字节 | 一次 | 跑（「集中」那一档上有照抄的节点；「分散」那一档 G0 全部重写） | 准入后 |
| 准入后 | (b) | `transaction.rs:16813`（`build_multi_level_tree` 没重写那一支），调用 `15326`（记账树）、`15348`（deadlist）、`15642`（中央映射树） | 多层码 2 树没重写的节点整份拷单元字节 | 一次，三棵各一次 | 函数三棵每次都调；有照抄节点的只有中央映射树：记账树每条 key 带这次的 txg（`transaction.rs:7994-8016`），每次全部换代；deadlist 在 G0 没有快照、零根 | 准入后（只算中央映射树） |
| 准入后 | 第二份 | `transaction.rs:16116`、`16117`、`16191`（`units.extend(… .units.iter().cloned())`），`16065`（extent 树同形） | (b) 那一批单元在装进 `units` 时再整份抄一次 | 一次 | 跑（extent 树 D = 0 时没有照抄节点，不进模型） | 准入后（照抄节点记第二份；重写节点那一份归「新装第二份」） |
| 准入后 | (c) | `transaction.rs:16880` | 多层码 2 树这一版的形状（全部叶 key 与分隔 key） | 一次，三棵各一次 | 跑（deadlist 零根时形状为空） | 准入后 |
| 准入后 | 丙6 | `transaction.rs:16635`、`16653`、`16671` | 新版容器树形状再抄一次 | 一次，三棵树 | 跑 | 准入后 |
| 准入后 | 原表 | `transaction.rs:15204-15205` | `allocator.records().to_vec()` 整份分配记录 | 一次 | 跑 | 准入后 |
| 准入后 | 第二份 | `transaction.rs:15217-15221` → `allocation_record_tree.rs:372` | 上一版全部分配记录按叶分组抄一份（`assert!` 里，产品构建照跑） | 一次 | 跑 | 准入后 |
| 准入后 | 第二份 | `transaction.rs:16203-16210`（`.cloned()` 在 `16209`） | 实例表照抄的那几片从上一版 `units` 整份拷 | 一次 | 跑（G0 实例表不重写，一片） | 准入后 |

轮数上限 `ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT` = 64（`transaction.rs:13141`），注释 `13138-13140`：「产品路径上三四轮就定」。

### 现算重建（第三类，读法第 8 条；不并进照抄）

| 段 | 文件:行 | 现算什么 | 每次发布几遍 | 模型里是哪一项（第五节） |
|---|---|---|---|---|
| 准入前 | `transaction.rs:9399-9409`（经 `code_two_tree.rs:88`） | 这一版全部映射 key 建成 `CodeTwoTreeKey` 的集合 | s | 每轮 M'·W_ck,M |
| 准入前 | `transaction.rs:9022-9026`（经 `code_two_tree.rs:88`） | deadlist 全部 key 的集合 | s | 每轮 E'_D·W_ck,D（G0 为 0） |
| 准入前 | `transaction.rs:13107`、`13120` → `allocation_record_tree.rs:373`；`13110`、`13121` → `343` | 这一版（分配器拷贝上）全部分配记录按叶分组，三次 | 每轮三次（s 轮） | 每轮 3·A'·40 |
| 准入前 | `transaction.rs:13121`（与 `13110`） | 装着记录的节点集 | 每轮一次 | 每轮 N'_AR·16 |
| 准入后 | `transaction.rs:15420-15613` | `mapped_units_with_locations`：每个映射单元按这一版的指针现算 key 与两条位置项 | 一次 | M'·95 |
| 准入后 | `transaction.rs:15614-15617`（`15616`） | `mapped_units`：上一行的 key 再抄一份 | 一次 | M'·67 |
| 准入后 | `transaction.rs:15623-15636`（经 `code_two_tree.rs:88`） | `mapping_entries_by_key`：全部 key 建成 `CodeTwoTreeKey`、全部条目字节 | 一次 | M'·194 |
| 准入后 | `transaction.rs:15638-15641`（`15640`） | `mapping_keys`：全部 key 字节再抄一份 | 一次 | M'·51 |
| 准入后 | `transaction.rs:16818-16824`（`16822`） | 重写的映射树叶的每条条目字节 | 一次 | min(M', τ_M·⌊φ·294⌋)·79 |
| 准入后 | `transaction.rs:16616`、`16623`、`16624` | deadlist 条目、`deleted_inodes` 的号、logged_ops 条目抄进新版 | 一次 | E'_D·67 + Q'·32 + 0（G0 logged_ops 为空） |
| 准入后 | `transaction.rs:15217-15224` → `allocation_record_tree.rs:373`、`343`；`13776` → `311` | 这一版全部分配记录按叶分组三次；装着记录的节点集与 `node_set()` | 一次 | 3·A'·40 + 2·N'_AR·16 |

记账树的 key（`transaction.rs:9004`、`15326-15338`）每次发布全部换代（`generation: txg`），现算的是这次的新内容，归「新装」，不进这一张。

### B 簇第二件「改完之后」：甲臂的基线（甲-B2后）

B 簇第二件已打进主工作区：重新开工的实现员 `m4-close-code-r1-fix-b2-writer-2` 交回了报告 `~/.local/share/singlefs/work/m4-close-code-r1-fix-b2-writer-2/result.md`（sha256 `bd0b3ca20f8653c7c63ffdbcfab53a477b7c260ff90587cdce1fd54ab15d342d`）与补丁 `patch/crates.patch`（sha256 `5180b75835fe157f20bb07dbdc586ad6a25563f132bfaca29466a7f36ab92ad2`）；执行员核过补丁已在主工作区（`git apply --check -R` 退 0）。那份报告自己点名了这两处，第 34 行原文整行：「     - 照抄那一支：`transaction.rs:8640`、`8965` 的 `shape_before.clone()`。新一版就是上一版那份形状，`&上一版` 之下只能抄。」对第二份那张表的影响按这一棵树写死成：

| 去掉 | 留着 |
|---|---|
| 甲2（基线 `transaction.rs:8437`、`8480`、`8764` 三处 `shape.clone()`；今天改成借引用 `8612-8615`、`8655-8658`、`8945-8947`）、甲3（基线 `container_tree.rs:2315`，`carried` 里那一份；今天 `carried` 只记每层节点数，`container_tree.rs:2696`）、甲4c 里的 `3188`（今天 `3718` 改成 `node_counts()`）、甲4d（基线 `2076` / `2161`；今天 `container_tree.rs:2369` 按值原地改，`3658` 调） | 甲1、inode 叶照抄与它的第二份（`4208`、`inode_tree.rs:1093` / `1970`）、**第三份补的 `8640`、`8965`（不追加那一支由调用方抄的那一份上一版形状）**、甲4b、甲4c 里的 `3576`、`12510-12512` 那一次 `ordinary_allocation_part`、映射 key 照抄（`9055`）、多层码 2 树规划照抄（`code_two_tree.rs:651`、`397`、`405`）、分配记录按叶分组（`allocation_record_tree.rs:372`）、(d)（`13045`、`13051`）、丙1、(a)、丙5、**第三份（全扫）补的 `14640`、`14643`**、分配记录树照抄节点（`16107`）、(b) 与它的第二份、(c)、丙6、分配记录 `to_vec`、实例表照抄；现算重建那一张表全部留着 |

基线行号是这两份基线文件的：`transaction.rs` `06c58028…`（就是 `git show HEAD:crates/singlefs-core/src/transaction.rs`）、`container_tree.rs` `e94e23ed…`（B 簇第二件第一位实现员留下的 `~/.local/share/singlefs/work/m4-close-code-r1-fix-b2-writer/base/crates/singlefs-core/src/container_tree.rs`；HEAD 里那一份是 `fcc1af29…`，不是基线）。模型同时算「甲-今天」与「甲-B2后」两个变体，**判据只用甲-B2后**；甲-今天照基线代码算，绝对值不随这一份变（G0 准入前 404074052），甲-今天 − 甲-B2后在 G0 上从第二份的 1936 变成 1808（第七节）。

### 全扫：命令、原样输出与归属

**脚本**：`~/.local/share/singlefs/work/e199-designer-revision-3/sweep/callgraph.py`（设计员只读草稿，不与装置共用代码，sha256 `8e966328c1fcbd4ac085c18995faf68b9dccc7bbba977759d64bb4cfb2ee60b6`，358 行），源码整份抄在下面三段代码块里（按次序拼起来逐字节就是那份文件）；执行员照第十一节停机条款第 1 条把它拷进自己的草稿目录原样重跑。切法写在它的文件头：先把注释与字面量内容换成空白再认函数、配括号；`#[cfg(test)]` 下的模块与函数跳过；spec fn、proof fn 收进清单标 ghost-skipped、不往下收、不 grep；函数体里 `proof { … }` 块不取调用名、不 grep；从根函数起按调用名（`名(`、`.名(`、`路径::名(`、带 turbofish 的、`::名`）一路收，同名的 exec 函数都收；一行归它所在的最内层函数，最内层函数没收到的行不报。

**范围**：`crates/singlefs-core/src`、`crates/singlefs-format/src`、`crates/singlefs-replay-prefix/src` 下全部 `*.rs`（49 份；第二份只扫第一个目录）。`skipped` 列出跳过的 `#[cfg(test)]` 61 处。

**根与收到的函数**：准入前根 `judge_the_publish_before_any_read_or_write`，收到 exec 函数 755 个、ghost 跳过 89 个（`pre-functions.txt` 844 行）；准入后根 `publish_admitted`，exec 692、ghost 88（`post-functions.txt` 780 行）。`crates/singlefs-format/src` 一个都没收到（那份文件这条路上只有常量）；`crates/singlefs-replay-prefix/src` 准入前收到 5 个 exec、1 个 ghost，准入后 5 个 exec，函数体里三组写法零命中。

**三组写法**（同第二份）：① `\.clone\(\)|\.to_vec\(\)` → `pre-clone-to_vec.txt` 39 行、`post-clone-to_vec.txt` 61 行；② `\.cloned\(\)|\.to_owned\(\)|\.copied\(\)|\bcopy_of_[a-z_]+\s*\(` → `pre-cloned-copied-copy_of.txt` 47 行、`post-cloned-copied-copy_of.txt` 37 行；③ `\.push\(\*|extend_from_slice\(|\.push\(\s*[a-z_]+\[|copy\.push\(` → `pre-push-deref-extend.txt` 11 行、`post-push-deref-extend.txt` 7 行。合 202 行（同一行命中两组的各算一次；准入前去重 94 行、准入后 104 行）。

**归属**：`classify_r3.py`（sha256 `595746de44eb553b97fbdad0a87ccfbd93a8fd8f7afce4d00bd1f7658d1d9576`）按「文件名:行」查两张手写的归属表，把 202 行逐条归一类，写 `classification.tsv`（202 行，sha256 `3340e1d70a0636d62a042fb5335512edd4659be5435f90071a50019afccf0b24`；一行一条：哪份输出、文件:行、所在函数、类、理由、原样那一行）。归属表整份抄在下面两段代码块里；按第一列取第二、三、六列逐字节就是那份输出（六份都用 `awk -F'\t' -v n=<输出> '$1==n{print $2"\t"$3"\t"$6}' classification.tsv | cmp - <输出>` 核过，退 0）。各类条数（`classify_r3.py` 现跑）：不随 C、R、U 涨 83、照抄·进模型 41、G0 不走到 27、调用名误收 17、新装第二份 11、现算重建·进模型 8、照抄·不进模型 6、照抄·进模型 + 新装第二份 3、建索引或遍历（不进模型）3、照抄·进模型 + 现算重建·进模型 2、射程外 1。

**同名误收的复核**：按名字收会多收同名函数。核过四个只经同名调用收进来的函数：`mount.rs:1251` `units`（`transaction.rs:8239` 的 `rewritten.units.units()` 调的是 `truncate.rs:243` 那一个；`mount.rs:1251` 这个只有 `mount.rs:7468` 调）、`recovery.rs:906` 与 `6266` `tree`（`transaction.rs:7998` 的 `row.tree(inode_tree)` 调的是 `transaction.rs:7932`；这两个只有 recovery.rs 自己调，`930`、`936`、`6290`、`6296`）、`make_filesystem.rs:605` `make_filesystem`（`root_ring.rs:188-189` 的路径 `crate::make_filesystem::…` 里的模块名被 `::名` 认成调用）。末尾加 `--exclude=mount.rs:1251,recovery.rs:906,recovery.rs:6266,make_filesystem.rs:605` 把三组写法合成一个正则再扫一遍：准入前 94 行变 83 行，准入后 104 行变 98 行，少掉的 11 + 6 行就是归属表里「调用名误收」的 17 行（`misnamed-pre.diff`、`misnamed-post.diff`）。准入后读新槽残留那一读（`read_the_residue_of_the_new_slots`）是真调，经同名 `read` / `read_at` 收进的几种读者里哪一种真走到不细分，归「不随 C、R、U 涨」（一次读一个单元）。

**与基线对照**：同一个脚本只扫 `crates/singlefs-core/src`（`--directories=crates/singlefs-core/src`），扫 `~/.local/share/singlefs/work/m4-close-code-r1-fix-b2-writer/base/`（`transaction.rs`、`container_tree.rs`、`mounted_session.rs` 的 sha256 与第二份第三节相同）：exec 函数准入前 736、准入后 680；命中 ① 42 / 60、② 47 / 37、③ 11 / 7，合 204 行。第二份那一版报的是函数清单 667 / 611 行、命中 41 / 59、47 / 34、10 / 5，合 196 行；旧脚本与旧输出都不在，只比得了条数，比不了是不是超集（推的：这一版每一组都不少于旧版）。基线与今天两次扫按「文件、函数、原样那一行」比（不比行号）：今天少了 `carried` 里那一份形状（甲3）、`append_records_to_the_open_container` 里 `3188` 那一份、`after_a_container_is_opened_at_the_right_end` 里的 `copy_of_one_level`（甲4d）、`ordinary_allocation_part` 与 `lineage_tree_after_this_publish` 里三处 `version.…_index_nodes.shape.clone()`（甲2）；今天多了 `8640`、`8965`、`admission.rs:337`（`after_writing`）、`transaction.rs:14161`（`build_container_tree_units` 拷一个 `Range`）；`container_tree.rs` `clone` 与 `transaction.rs` `first_entry_of` 两行是同一句换了写法。

**今天比基线多收的函数**（只比 `crates/singlefs-core/src`，按名字）：准入前 `after_removing`、`after_writing`、`ancestors_of_the_rightmost_containers`、`ancestor_suffix_starts_of_the_rightmost_containers`、`first_rewritten_container`、`has_a_node_at`、`inode_record_found_by_its_leaf`、`is_live`、`is_the_root`、`node_counts`、`retirement_root_unit`、`retirement_tree_internal_levels_read_from_its_root_node_header`、`rightmost_count_of_the_rewritten_containers`、`root_level`、`root_position`、`tombstone_root_unit`、`tombstone_tree_internal_levels_read_from_its_root_node_header`、`unit_at_the_recorded_position`、`value_stored_for_identity`；准入后 `ancestors_of_the_rightmost_containers`、`ancestor_suffix_starts_of_the_rightmost_containers`、`children_of_every_node`、`first_rewritten_container`、`has_a_node_at`、`is_the_root`、`of_the_units`、`retirement_root_unit`、`retirement_tree_internal_levels_read_from_its_root_node_header`、`rightmost_count_of_the_rewritten_containers`、`root_level`、`root_position`、`tombstone_root_unit`、`tombstone_tree_internal_levels_read_from_its_root_node_header`、`unit_at_the_recorded_position`、`value_stored_for_identity`。它们的函数体都在今天这一次扫里；有命中的只有 `after_writing`（`admission.rs:337`，归「不随 C、R、U 涨」）。

**全扫没有罩住的**：经闭包、trait 对象、宏、运算符与 `?` / `for` 隐含的调用（`From::from`、`IntoIterator::next`、`Drop`、`Deref`）走的路，按名字收不一定收得到；`#[derive(Clone)]` 的结构整体被 `=` 复制而没写 `.clone()` 的；三组写法之外的拷贝（`Vec::from`、`iter().map(|x| *x).collect()`、`to_string` 这类）没扫；`#[cfg(feature = …)]` 下的代码不分特性一律算进来。执行员开工时照第十一节停机条款第 1 条再核。

`callgraph.py` 第 1–147 行：

```python
#!/usr/bin/env python3
"""E199（发布照抄上一版的代价）第三份跑前登记的发布路径全扫：只读源文件，不起任何程序。

用法（<仓根> 下扫 crates/singlefs-core/src、crates/singlefs-format/src、crates/singlefs-replay-prefix/src 里的 *.rs）：
  callgraph.py <仓根> list <根函数>             收到的函数清单：文件:起-止、名字、类（exec / ghost-skipped）
  callgraph.py <仓根> grep <根函数> <正则>      收到的 exec 函数体里按正则命中的行：文件:行、所在函数、原样那一行
  callgraph.py <仓根> trace <根函数> <函数名或 文件:fn 行>…   按收到的次序（广度优先）列出从根到它的一条调用链
  callgraph.py <仓根> skipped                   跳过的 #[cfg(test)] 模块与函数（文件:起-止）
  末尾加 --directories=<目录,目录> 换掉扫的目录（第二份登记只扫 crates/singlefs-core/src，对旧基线复现计数时用）
  末尾加 --exclude=<文件名:fn 行,…> 把点名的函数当不在（核过只被同名调用收进来的那几个；去掉之后收不到的命中归「调用名误收」）

切法：
- 先把注释换成空白、把字符串与字符字面量的内容换成空白（保留行列），在这份「代码面」上认函数、配括号、取调用名、grep。
- 函数：一行里 `fn 名` 之前只有可见性与修饰词（pub、const、unsafe、async、extern、open、closed、spec、proof、exec、broadcast…）。
  函数体：从 `fn` 起按圆括号与方括号深度往后找，深度 0 处先碰到 `;` 是没有函数体的声明；碰到 Verus 的
  requires / ensures / recommends / decreases / returns / opens_invariants 之后，函数体是与 `fn` 同缩进、去空白后只有 `{` 的那一行；
  否则是深度 0 处第一个 `{`。函数体尾按花括号配对。
- 跳过：`#[cfg(test)]` 与 `#[cfg(all(test` 下的模块与函数；spec fn、proof fn 收进清单标 ghost-skipped，不往下收、不 grep。
- 收法：从根函数起，函数体里每个 `名(`（含 `.名(`、`路径::名(`、带 turbofish 的）与 `::名` 当一次调用，凡同名的 exec 函数都收（同名多收的在归属里标「调用名误收」）；
  函数体里 `proof { … }` 块的内容不取调用名、不 grep。
- 一行归它所在的最内层函数；最内层函数没收到的行不报。
"""
import os
import re
import sys

SOURCE_DIRECTORIES = ("crates/singlefs-core/src", "crates/singlefs-format/src", "crates/singlefs-replay-prefix/src")
MODIFIERS = r"(?:(?:pub(?:\s*\([^)]*\))?|const|unsafe|async|extern(?:\s*\"[^\"]*\")?|default|open|closed|spec|proof|exec|broadcast|uninterp|tracked)\s+)*"
FUNCTION_LINE = re.compile(r"^(\s*)(" + MODIFIERS + r")fn\s+([A-Za-z_][A-Za-z0-9_]*)")
VERUS_CLAUSE = re.compile(r"^\s*(requires|ensures|recommends|decreases|returns|opens_invariants|no_unwind)\b")
CALL_NAME = re.compile(r"([A-Za-z_][A-Za-z0-9_]*)\s*(?:::\s*<[^()]*?>\s*)?\(")
PATH_NAME = re.compile(r"::\s*([A-Za-z_][A-Za-z0-9_]*)")
CFG_TEST = re.compile(r"#\[\s*cfg\s*\(\s*(?:test\s*\)|all\s*\(\s*test\b)")
PROOF_BLOCK = re.compile(r"\bproof\s*\{")


def code_surface(text):
    """注释换成空白、字符串与字符字面量的内容换成空白，换行与列不变。"""
    out = list(text)
    i, n = 0, len(text)

    def blank(start, end):
        for k in range(start, end):
            if out[k] != "\n":
                out[k] = " "

    while i < n:
        c = text[i]
        if c == "/" and i + 1 < n and text[i + 1] == "/":
            j = text.find("\n", i)
            j = n if j < 0 else j
            blank(i, j)
            i = j
        elif c == "/" and i + 1 < n and text[i + 1] == "*":
            depth, j = 1, i + 2
            while j < n and depth > 0:
                if text.startswith("/*", j):
                    depth += 1
                    j += 2
                elif text.startswith("*/", j):
                    depth -= 1
                    j += 2
                else:
                    j += 1
            blank(i, j)
            i = j
        elif c in "rb" and re.match(r"(?:br|r)(#*)\"", text[i:i + 300]) and (i == 0 or not (text[i - 1].isalnum() or text[i - 1] == "_")):
            m = re.match(r"(?:br|r)(#*)\"", text[i:])
            hashes = m.group(1)
            start = i + m.end()
            end = text.find("\"" + hashes, start)
            end = n if end < 0 else end
            blank(start, end)
            i = end + 1 + len(hashes)
        elif c == "\"":
            j = i + 1
            while j < n and text[j] != "\"":
                j += 2 if text[j] == "\\" else 1
            blank(i + 1, j)
            i = j + 1
        elif c == "'":
            if i + 1 < n and text[i + 1] == "\\":
                j = text.find("'", i + 2)
                if text[i + 2:i + 3] == "'":
                    j = i + 3
                blank(i + 1, j)
                i = j + 1
            elif i + 2 < n and text[i + 2] == "'":
                blank(i + 1, i + 2)
                i = i + 3
            else:
                i += 1
        else:
            i += 1
    return "".join(out)


def matching_brace(surface, open_index):
    depth = 0
    for k in range(open_index, len(surface)):
        if surface[k] == "{":
            depth += 1
        elif surface[k] == "}":
            depth -= 1
            if depth == 0:
                return k
    return len(surface) - 1


class SourceFile:
    def __init__(self, root, relative):
        self.relative = relative
        with open(os.path.join(root, relative), encoding="utf-8") as handle:
            self.text = handle.read()
        self.lines = self.text.split("\n")
        self.surface = code_surface(self.text)
        self.surface_lines = self.surface.split("\n")
        self.line_starts = [0]
        for line in self.lines:
            self.line_starts.append(self.line_starts[-1] + len(line) + 1)

    def line_of(self, index):
        lo, hi = 0, len(self.line_starts) - 1
        while lo < hi:
            mid = (lo + hi + 1) // 2
            if self.line_starts[mid] <= index:
                lo = mid
            else:
                hi = mid - 1
        return lo + 1


class Function:
    def __init__(self, source, name, mode, fn_line, body_open, body_close):
        self.source = source
        self.name = name
        self.mode = mode
        self.fn_line = fn_line
        self.body_open = body_open
        self.body_close = body_close
        self.first_line = source.line_of(body_open)
        self.last_line = source.line_of(body_close)

    def key(self):
        return (self.source.relative, self.fn_line)


```

`callgraph.py` 第 148–282 行：

```python
def find_body_open(source, fn_line):
    """从 fn 那一行起找函数体的 `{`；没有函数体交 None。"""
    indent = len(source.surface_lines[fn_line - 1]) - len(source.surface_lines[fn_line - 1].lstrip())
    start = source.line_starts[fn_line - 1] + FUNCTION_LINE.match(source.surface_lines[fn_line - 1]).start(3)
    paren = 0
    saw_clause = False
    k = start
    surface = source.surface
    while k < len(surface):
        ch = surface[k]
        if ch == "\n":
            next_line = source.line_of(k + 1)
            if next_line <= len(source.surface_lines) and paren == 0 and VERUS_CLAUSE.match(source.surface_lines[next_line - 1]):
                saw_clause = True
        if ch in "([":
            paren += 1
        elif ch in ")]":
            paren -= 1
        elif paren == 0 and ch == ";":
            return None
        elif paren == 0 and ch == "{":
            if not saw_clause:
                return k
            line_number = source.line_of(k)
            line = source.surface_lines[line_number - 1]
            if line.strip() == "{" and len(line) - len(line.lstrip()) == indent:
                return k
            k = matching_brace(surface, k)
        k += 1
    return None


def skipped_ranges(source):
    ranges = []
    for match in CFG_TEST.finditer(source.surface):
        k = match.end()
        brace = source.surface.find("{", k)
        semicolon = source.surface.find(";", k)
        if brace < 0 or (0 <= semicolon < brace):
            continue
        ranges.append((source.line_of(match.start()), source.line_of(matching_brace(source.surface, brace))))
    return ranges


def load(root, directories):
    sources, functions = [], []
    for directory in directories:
        base = os.path.join(root, directory)
        for walk_root, _, names in os.walk(base):
            for name in sorted(names):
                if name.endswith(".rs"):
                    sources.append(SourceFile(root, os.path.relpath(os.path.join(walk_root, name), root)))
    sources.sort(key=lambda s: s.relative)
    skipped = {}
    for source in sources:
        ranges = skipped_ranges(source)
        skipped[source.relative] = ranges
        for number, line in enumerate(source.surface_lines, start=1):
            match = FUNCTION_LINE.match(line)
            if not match:
                continue
            if any(lo <= number <= hi for lo, hi in ranges):
                continue
            modifiers = match.group(2)
            mode = "ghost-skipped" if re.search(r"\b(spec|proof)\b", modifiers) else "exec"
            body_open = find_body_open(source, number)
            if body_open is None:
                continue
            body_close = matching_brace(source.surface, body_open)
            functions.append(Function(source, match.group(3), mode, number, body_open, body_close))
    return sources, functions, skipped


def ghost_mask(function):
    """函数体里 proof 块占的下标区间。"""
    surface = function.source.surface
    spans = []
    k = function.body_open
    while True:
        match = PROOF_BLOCK.search(surface, k, function.body_close)
        if not match:
            break
        brace = match.end() - 1
        close = matching_brace(surface, brace)
        spans.append((match.start(), close))
        k = close + 1
    return spans


def body_without_ghost(function):
    surface = function.source.surface
    text = list(surface[function.body_open:function.body_close + 1])
    for lo, hi in ghost_mask(function):
        for k in range(lo, hi + 1):
            if text[k - function.body_open] != "\n":
                text[k - function.body_open] = " "
    return "".join(text)


def reach(functions, root_name):
    by_name = {}
    for function in functions:
        by_name.setdefault(function.name, []).append(function)
    queue = [f for f in by_name.get(root_name, []) if f.mode == "exec"]
    if not queue:
        sys.exit(f"根函数 {root_name} 没找到")
    reached = {f.key(): f for f in queue}
    parent = {f.key(): None for f in queue}
    ghost = {}
    while queue:
        function = queue.pop(0)
        body = body_without_ghost(function)
        names = set(CALL_NAME.findall(body)) | set(PATH_NAME.findall(body))
        for name in sorted(names):
            for callee in by_name.get(name, []):
                if callee.mode != "exec":
                    ghost[callee.key()] = callee
                    continue
                if callee.key() not in reached:
                    reached[callee.key()] = callee
                    parent[callee.key()] = function.key()
                    queue.append(callee)
    reach.parent = parent
    return reached, ghost


def innermost(functions):
    """每一行归它所在的最内层函数（按函数体区间）。"""
    owner = {}
    for function in sorted(functions, key=lambda f: (f.source.relative, f.first_line, -f.last_line)):
        for line in range(function.first_line, function.last_line + 1):
            owner[(function.source.relative, line)] = function
    return owner


```

`callgraph.py` 第 283–358 行：

```python
def main(argv):
    if len(argv) < 3:
        sys.exit(__doc__)
    directories = SOURCE_DIRECTORIES
    excluded = set()
    while argv[-1].startswith("--"):
        if argv[-1].startswith("--directories="):
            directories = tuple(argv[-1][len("--directories="):].split(","))
        elif argv[-1].startswith("--exclude="):
            excluded = set(argv[-1][len("--exclude="):].split(","))
        else:
            sys.exit(f"认不得的选项 {argv[-1]}")
        argv = argv[:-1]
    root, command = argv[1], argv[2]
    sources, functions, skipped = load(root, directories)
    unknown = excluded - {f"{f.source.relative.split('/')[-1]}:{f.fn_line}" for f in functions}
    if unknown:
        sys.exit(f"--exclude 点名的函数不在：{sorted(unknown)}")
    functions = [f for f in functions if f"{f.source.relative.split('/')[-1]}:{f.fn_line}" not in excluded]
    if command == "skipped":
        for relative in sorted(skipped):
            for lo, hi in skipped[relative]:
                print(f"{relative}:{lo}-{hi}")
        return
    root_name = argv[3]
    reached, ghost = reach(functions, root_name)
    if command == "trace":
        for wanted in argv[4:]:
            for key, function in sorted(reached.items()):
                if f"{function.source.relative}:{function.fn_line}" != wanted and function.name != wanted:
                    continue
                chain = []
                cursor = key
                while cursor is not None:
                    chain.append(f"{reached[cursor].name}（{cursor[0].split('/')[-1]}:{cursor[1]}）")
                    cursor = reach.parent[cursor]
                print(" ← ".join(chain))
        return
    if command == "list":
        rows = [(f.source.relative, f.fn_line, f.last_line, f.name, "exec") for f in reached.values()]
        rows += [(f.source.relative, f.fn_line, f.last_line, f.name, "ghost-skipped") for f in ghost.values()]
        for relative, first, last, name, mode in sorted(rows):
            print(f"{relative}:{first}-{last}\t{name}\t{mode}")
        return
    if command == "grep":
        pattern = re.compile(argv[4])
        owner = innermost(functions)
        hits = []
        for function in reached.values():
            source = function.source
            spans = ghost_mask(function)
            for line in range(function.first_line, function.last_line + 1):
                if owner.get((source.relative, line)) is not function:
                    continue
                surface_line = source.surface_lines[line - 1]
                line_start = source.line_starts[line - 1]
                if function.first_line == line:
                    offset = function.body_open - line_start
                    surface_line = " " * offset + surface_line[offset:]
                if function.last_line == line:
                    offset = function.body_close - line_start + 1
                    surface_line = surface_line[:offset]
                for match in pattern.finditer(surface_line):
                    position = line_start + match.start()
                    if any(lo <= position <= hi for lo, hi in spans):
                        continue
                    hits.append((source.relative, line, function.name, source.lines[line - 1].strip()))
                    break
        for relative, line, name, text in sorted(set(hits)):
            print(f"{relative}:{line}\t{name}\t{text}")
        return
    sys.exit(f"认不得的命令 {command}")


if __name__ == "__main__":
    main(sys.argv)
```

**归属表**（`classification.tsv` 整份；列以制表符分开：输出、文件:行、所在函数、类、理由、原样那一行）准入前三份输出（第 1–97 行）：

```text
pre-clone-to_vec.txt	crates/singlefs-core/src/allocation_record_tree.rs:101	of_devices	不随 C、R、U 涨	每块盘一项（两块盘）	let mut slots_of_each_device = slots_of_each_device.to_vec();
pre-clone-to_vec.txt	crates/singlefs-core/src/code_two_tree.rs:88	new	现算重建·进模型	被调方：9022 起 deadlist 全部 key、9399 起这一版全部映射 key 按全部 key 调它（现算重建表两行）；别的调用方一次一把改动的 key	bytes: key_bytes.to_vec(),
pre-clone-to_vec.txt	crates/singlefs-core/src/code_two_tree.rs:163	parse_internal_entry	调用名误收	去掉经核只经同名调用收进来的四个函数（mount.rs:1251 units、recovery.rs:906 与 6266 tree、make_filesystem.rs:605 make_filesystem）之后收不到	Ok((entry[..key_width_in_bytes].to_vec(), child))
pre-clone-to_vec.txt	crates/singlefs-core/src/code_two_tree.rs:397	planning_node_of	照抄·进模型	多层码 2 树规划照抄：叶 key（Rl）	keys: keys.clone(),
pre-clone-to_vec.txt	crates/singlefs-core/src/code_two_tree.rs:405	planning_node_of	照抄·进模型	多层码 2 树规划照抄：分隔 key（Rl）	separator_key: child.separator_key.clone(),
pre-clone-to_vec.txt	crates/singlefs-core/src/code_two_tree.rs:423	route_for_insertion	不随 C、R、U 涨	插入一把 key，随这次改动条数	children[0].separator_key = key.clone();
pre-clone-to_vec.txt	crates/singlefs-core/src/code_two_tree.rs:443	split_in_the_middle	不随 C、R、U 涨	一次分裂	separator_key: right_keys[0].clone(),
pre-clone-to_vec.txt	crates/singlefs-core/src/code_two_tree.rs:455	split_in_the_middle	不随 C、R、U 涨	一次分裂	separator_key: right_children[0].separator_key.clone(),
pre-clone-to_vec.txt	crates/singlefs-core/src/code_two_tree.rs:518	insert_at_the_root	不随 C、R、U 涨	一次插入	.clone();
pre-clone-to_vec.txt	crates/singlefs-core/src/code_two_tree.rs:660	plan_the_tree_after_this_publish	不随 C、R、U 涨	插入的 key，随这次改动条数	insert_at_the_root(&mut root, inserted.clone(), capacity)?;
pre-clone-to_vec.txt	crates/singlefs-core/src/container_tree.rs:3576	append_records_to_the_open_container	照抄·进模型	甲4c 的 3046 那一半：追加那一支抄一份形状（O / Rl）	let mut shape = shape_before_this_publish.clone();
pre-clone-to_vec.txt	crates/singlefs-core/src/extent_tree.rs:581	parse	调用名误收	去掉经核只经同名调用收进来的四个函数（mount.rs:1251 units、recovery.rs:906 与 6266 tree、make_filesystem.rs:605 make_filesystem）之后收不到	.to_vec();
pre-clone-to_vec.txt	crates/singlefs-core/src/mount.rs:8711	read	调用名误收	去掉经核只经同名调用收进来的四个函数（mount.rs:1251 units、recovery.rs:906 与 6266 tree、make_filesystem.rs:605 make_filesystem）之后收不到	return Some(bytes.clone());
pre-clone-to_vec.txt	crates/singlefs-core/src/mount.rs:8717	read	调用名误收	去掉经核只经同名调用收进来的四个函数（mount.rs:1251 units、recovery.rs:906 与 6266 tree、make_filesystem.rs:605 make_filesystem）之后收不到	.insert(placement, bytes.clone());
pre-clone-to_vec.txt	crates/singlefs-core/src/mounted_session.rs:1032	read_unrecorded_blocks_from_the_disk	调用名误收	去掉经核只经同名调用收进来的四个函数（mount.rs:1251 units、recovery.rs:906 与 6266 tree、make_filesystem.rs:605 make_filesystem）之后收不到	recorded.insert(block_index, block.to_vec());
pre-clone-to_vec.txt	crates/singlefs-core/src/mounted_session.rs:1164	block_device_error_with_the_same_content	调用名误收	去掉经核只经同名调用收进来的四个函数（mount.rs:1251 units、recovery.rs:906 与 6266 tree、make_filesystem.rs:605 make_filesystem）之后收不到	BlockDeviceError::ImageFileAlreadyExists { path: path.clone() }
pre-clone-to_vec.txt	crates/singlefs-core/src/mounted_session.rs:1167	block_device_error_with_the_same_content	调用名误收	去掉经核只经同名调用收进来的四个函数（mount.rs:1251 units、recovery.rs:906 与 6266 tree、make_filesystem.rs:605 make_filesystem）之后收不到	BlockDeviceError::ImageFileMissing { path: path.clone() }
pre-clone-to_vec.txt	crates/singlefs-core/src/mounted_session.rs:1209	keep_the_self_verified_reread	调用名误收	去掉经核只经同名调用收进来的四个函数（mount.rs:1251 units、recovery.rs:906 与 6266 tree、make_filesystem.rs:605 make_filesystem）之后收不到	recorded.insert(block_index, block.to_vec());
pre-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:4208	inode_leaf_container_contents	照抄·进模型	上一版 inode 叶容器内容（O）	.map(|container| container.contents.clone())
pre-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:6213	deleted_inodes_after_this_publish	不随 C、R、U 涨	deleted_inodes 至多一条 key（D8（核心索引结构） 已定项 16 第 6 条）	let mut keys = before.to_vec();
pre-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:6301	logged_operations_after_this_publish	不随 C、R、U 涨	logged_ops 至多一条	let mut entries = before.to_vec();
pre-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:6585	deadlist_changes_of_this_publish	不随 C、R、U 涨	这次换下的每个用户可见角色抄一把映射 key，随这次换下的角色数	.to_vec();
pre-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:6616	deadlist_changes_of_this_publish	G0 不走到	同上	mapping_key: mapping_key.clone(),
pre-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:6653	deadlist_changes_of_this_publish	G0 不走到	deadlist 条目放不掉的出错那一支	mapping_key: entry.mapping_key.clone(),
pre-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:6665	deadlist_changes_of_this_publish	G0 不走到	deadlist 全部条目：没有快照时 0 条（模型 E_D 恒 0）	entry.clone(),
pre-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:6676	deadlist_changes_of_this_publish	G0 不走到	快照留存的映射条目：没有快照时 0 条	(key.clone(), *locations),
pre-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:8104	rewritten_roles	不随 C、R、U 涨	这次重写的数据角色	let mut roles: Vec<TransactionUnit> = self.data_roles_rewritten.clone();
pre-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:8606	ordinary_allocation_part	照抄·进模型	甲1 墓碑树每片容器 contents.clone()（O，s + 1 遍）	.map(|container| container.contents.clone())
pre-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:8640	ordinary_allocation_part	照抄·进模型	第三份新补：墓碑树不追加那一支照抄上一版形状（O，s + 1 遍，只在不删除的发布上）	tombstone_shape_before.clone(),
pre-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:8651	ordinary_allocation_part	照抄·进模型	甲1 退役树（O）	.map(|container| container.contents.clone())
pre-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:8939	lineage_tree_after_this_publish	照抄·进模型	甲1 谱系树（Rl，s 遍）	.map(|container| container.contents.clone())
pre-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:8965	lineage_tree_after_this_publish	照抄·进模型	第三份新补：谱系树不追加那一支照抄上一版形状（Rl，s 遍，G0 走到）	shape_before.clone(),
pre-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:9055	plan_the_multi_level_trees	照抄·进模型	没重写的映射单元的映射 key 逐个从上一版抄（Rl）	.to_vec()
pre-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:12974	placement_of_a_deadlist_entry_to_release	G0 不走到	deadlist 条目释放：G0 没有	mapping_key: entry.mapping_key.clone(),
pre-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:12990	role_standing_for_the_unit_of_a_deadlist_entry	G0 不走到	同上	mapping_key: entry.mapping_key.clone(),
pre-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:13033	settle_the_allocation_record_tree	G0 不走到	整理的一批（搬单元）	.map(|relocation| relocation.mapping_key.clone())
pre-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:13051	settle_the_allocation_record_tree	照抄·进模型	(d) 上一版分配记录树节点集第二份（Once）	let mut nodes_after: BTreeSet<AllocationRecordTreeNode> = previous_nodes.clone();
pre-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:13098	settle_the_allocation_record_tree	射程外	每轮整份拷分配器：拷的是分配器，不是上一版 TransactionOutput（同第二份登记）	let mut rehearsal = allocator.clone();
pre-clone-to_vec.txt	crates/singlefs-core/src/unlink.rs:313	plan_the_extent_tree_after_the_final_batch	照抄·不进模型	最后一批：有 extent 条目的文件号集合，按有数据的文件数涨；模型里有数据的只有第一个文件（至多一个号）	let mut inodes_with_an_entry_after = inodes_with_an_entry_before.clone();
pre-cloned-copied-copy_of.txt	crates/singlefs-core/src/admission.rs:337	after_writing	不随 C、R、U 涨	一条 inode 记录，次数等于这次写的条数	.copied()
pre-cloned-copied-copy_of.txt	crates/singlefs-core/src/admission.rs:1511	central_mapping_tree_nodes_an_empty_publish_rewrites_at_most	不随 C、R、U 涨	每层一项	for (level, nodes_at_this_level) in nodes_at_each_level.iter().copied().enumerate() {
pre-cloned-copied-copy_of.txt	crates/singlefs-core/src/allocation_record_tree.rs:376	nodes_whose_contents_changed	建索引或遍历（不进模型）	两版有记录的叶号并集（叶号 16 字节，按叶数），是建索引，同第三节「不进模型的」口径	before.keys().chain(after.keys()).copied().collect();
pre-cloned-copied-copy_of.txt	crates/singlefs-core/src/allocation_record_tree.rs:448	plan_the_tree_after_this_publish	不随 C、R、U 涨	换下的旧节点（重写的与这一版没有的）；外层是遍历上一版节点	.copied()
pre-cloned-copied-copy_of.txt	crates/singlefs-core/src/allocator.rs:2616	reclaim_released_records_up_to	不随 C、R、U 涨	这次回收的那几条记录	.copied()
pre-cloned-copied-copy_of.txt	crates/singlefs-core/src/code_two_tree.rs:651	plan_the_tree_after_this_publish	照抄·进模型	上一版形状全部 key 抄进集合（Rl）	previous.keys_in_order().into_iter().cloned().collect();
pre-cloned-copied-copy_of.txt	crates/singlefs-core/src/code_two_tree.rs:1126	flatten_read	调用名误收	去掉经核只经同名调用收进来的四个函数（mount.rs:1251 units、recovery.rs:906 与 6266 tree、make_filesystem.rs:605 make_filesystem）之后收不到	leaf_entries_in_key_order.extend(node.header.entries.iter().cloned());
pre-cloned-copied-copy_of.txt	crates/singlefs-core/src/container_tree.rs:1498	clone	照抄·进模型	被调方：容器树形状 Clone 的实现，计在调它的那几行（准入前 3576、8640、8965；准入后丙6）	children_of_each_node_at_each_level: copy_of_the_children_counts(&self.children_of_each_node_at_each_level),
pre-cloned-copied-copy_of.txt	crates/singlefs-core/src/container_tree.rs:1520	copy_of_the_children_counts	照抄·进模型	被调方：同上，逐层照抄孩子数	copy.push(copy_of_one_level(&children_of_each_node_at_each_level[level_index]));
pre-cloned-copied-copy_of.txt	crates/singlefs-core/src/container_tree.rs:2664	copy_of_the_containers	照抄·进模型	被调方：copy_of_the_containers，计在甲4b（3575）	copy.push(RecordContainer { identity: container.identity, records: copy_of_the_records(&container.records) });
pre-cloned-copied-copy_of.txt	crates/singlefs-core/src/container_tree.rs:3575	append_records_to_the_open_container	照抄·进模型	甲4b：追加那一支抄全部容器与记录（O / Rl）	let mut containers = copy_of_the_containers(containers_before_this_publish);
pre-cloned-copied-copy_of.txt	crates/singlefs-core/src/extent_tree.rs:183	canonical_upper_segment_of_the_inodes_with_an_entry	G0 不走到	删除的一批（extent 上段规范形），取一个号	let Some(largest_inode_with_an_entry) = inodes_with_an_entry.last().copied() else {
pre-cloned-copied-copy_of.txt	crates/singlefs-core/src/extent_tree.rs:244	plan_the_extent_upper_segment	G0 不走到	删除的一批，换下的上段节点	.copied()
pre-cloned-copied-copy_of.txt	crates/singlefs-core/src/inode_tree.rs:1093	write_records_into_leaf_containers	照抄·进模型	inode 叶第二份（O）	let mut containers = copy_of_the_leaf_containers(containers_before_this_publish);
pre-cloned-copied-copy_of.txt	crates/singlefs-core/src/inode_tree.rs:1302	copy_of_the_leaf_containers	照抄·进模型	被调方：同上	copy.push(copy_of_one_leaf_container(&containers[position]));
pre-cloned-copied-copy_of.txt	crates/singlefs-core/src/inode_tree.rs:1970	from_the_previous_version	照抄·进模型	最后一批那一支的 inode 叶第二份（O）	contents: copy_of_one_leaf_container(&containers[position]),
pre-cloned-copied-copy_of.txt	crates/singlefs-core/src/mounted_read.rs:975	read_at	调用名误收	去掉经核只经同名调用收进来的四个函数（mount.rs:1251 units、recovery.rs:906 与 6266 tree、make_filesystem.rs:605 make_filesystem）之后收不到	.copied()
pre-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:3558	count	不随 C、R、U 涨	只供测试的查找计数（lookup-work-counters 特性，默认关）	let site = state.sites_entered.last().copied();
pre-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:3693	first_entry_of	不随 C、R、U 涨	取一项	.copied()
pre-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:4181	inode_record_of	不随 C、R、U 涨	一条 inode 记录	.copied()
pre-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:4251	inode_number_watermark_steps	不随 C、R、U 涨	inode 号水位的台阶行（模型取一行），不随 C、R、U 涨	.copied()
pre-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:4772	roles_replaced_via_mapping	不随 C、R、U 涨	这次经映射换下的角色	.copied()
pre-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:6464	snapshot_inode_root_entry	G0 不走到	快照：G0 没有快照	.copied()
pre-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:6499	deadlist_entries_the_destruction_of_a_snapshot_releases	G0 不走到	销毁快照	.cloned()
pre-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:7119	previous_roles_replaced_by_this_publish	不随 C、R、U 涨	这次换下的上一版角色	self.rewritten_roles.iter().copied().filter(|identity| {
pre-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:7133	previous_roles_replaced_by_this_publish	不随 C、R、U 涨	同上	.copied()
pre-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:7147	previous_roles_replaced_by_this_publish	不随 C、R、U 涨	同上	.copied(),
pre-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:7153	previous_roles_replaced_by_this_publish	不随 C、R、U 涨	同上	.copied(),
pre-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:7158	previous_roles_replaced_by_this_publish	不随 C、R、U 涨	同上	.copied()
pre-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:7166	previous_roles_replaced_by_this_publish	不随 C、R、U 涨	同上（extent 树换下的）	replaced.extend(self.extent_tree.replaced_previous_roles.iter().copied());
pre-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:7181	previous_roles_replaced_by_this_publish	不随 C、R、U 涨	同上	replaced.extend(self.rewritten_roles.iter().copied().filter(|identity| {
pre-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:7196	previous_roles_replaced_by_this_publish	不随 C、R、U 涨	同上	.copied()
pre-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:7648	replaced_previous_roles	不随 C、R、U 涨	extent 树这次换下的角色	.copied()
pre-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:8750	resolve	不随 C、R、U 涨	这次没有后继的上一版角色	.copied(),
pre-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:8789	resolve	不随 C、R、U 涨	这次重写的角色	.copied()
pre-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:8801	resolve	不随 C、R、U 涨	这次没有后继的角色	.copied()
pre-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:8804	resolve	不随 C、R、U 涨	这次换下的 extent 角色	.chain(extent_tree.replaced_previous_roles.iter().copied())
pre-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:12914	placements_released_by_this_publish	不随 C、R、U 涨	这次换下的角色	.copied()
pre-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:12987	role_standing_for_the_unit_of_a_deadlist_entry	G0 不走到	同上	let unit_class = entry.mapping_key.first().copied().unwrap_or(0);
pre-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:13037	settle_the_allocation_record_tree	不随 C、R、U 涨	这次条目变了位置、key 不变的映射 key	.cloned(),
pre-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:13055	settle_the_allocation_record_tree	不随 C、R、U 涨	重写集与这一版节点的交集	rewritten.intersection(&nodes_after).copied().collect();
pre-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:13084	settle_the_allocation_record_tree	G0 不走到	整理的一批	.cloned()
pre-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:13161	roles_named_by_each_record_of_the_publish	不随 C、R、U 涨	这次发布的记录点名的角色	.copied()
pre-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:13210	roles_other_than_the_data_roles_named_before_the_last_record	不随 C、R、U 涨	同上	.copied()
pre-cloned-copied-copy_of.txt	crates/singlefs-core/src/truncate.rs:356	plan_the_lower_segment_after_a_size_change	G0 不走到	改 size	.copied()
pre-cloned-copied-copy_of.txt	crates/singlefs-core/src/truncate.rs:361	plan_the_lower_segment_after_a_size_change	G0 不走到	改 size	.copied()
pre-cloned-copied-copy_of.txt	crates/singlefs-core/src/unlink.rs:182	previous_nodes_replaced	G0 不走到	删除的一批	.copied()
pre-push-deref-extend.txt	crates/singlefs-core/src/allocation_record_tree.rs:327	records_of_each_leaf	照抄·进模型 + 现算重建·进模型	上一版分配记录按叶分组（照抄）与这一版的按叶分组（现算重建），两张表各有一行	.push(*record);
pre-push-deref-extend.txt	crates/singlefs-core/src/container_tree.rs:1520	copy_of_the_children_counts	照抄·进模型	被调方：同上，逐层照抄孩子数	copy.push(copy_of_one_level(&children_of_each_node_at_each_level[level_index]));
pre-push-deref-extend.txt	crates/singlefs-core/src/container_tree.rs:1540	copy_of_one_level	照抄·进模型	被调方：同上，逐个节点照抄孩子数	copy.push(children_of_each_node[node]);
pre-push-deref-extend.txt	crates/singlefs-core/src/container_tree.rs:2639	copy_of_the_records	照抄·进模型	被调方：copy_of_the_records，计在甲4b（3575）	copy.push(records[position]);
pre-push-deref-extend.txt	crates/singlefs-core/src/container_tree.rs:2664	copy_of_the_containers	照抄·进模型	被调方：copy_of_the_containers，计在甲4b（3575）	copy.push(RecordContainer { identity: container.identity, records: copy_of_the_records(&container.records) });
pre-push-deref-extend.txt	crates/singlefs-core/src/inode_tree.rs:1302	copy_of_the_leaf_containers	照抄·进模型	被调方：同上	copy.push(copy_of_one_leaf_container(&containers[position]));
pre-push-deref-extend.txt	crates/singlefs-core/src/mounted_read.rs:1014	read_at	调用名误收	去掉经核只经同名调用收进来的四个函数（mount.rs:1251 units、recovery.rs:906 与 6266 tree、make_filesystem.rs:605 make_filesystem）之后收不到	bytes.extend_from_slice(
pre-push-deref-extend.txt	crates/singlefs-core/src/transaction.rs:4262	inode_number_watermark_steps	不随 C、R、U 涨	同上	steps.push(*row);
pre-push-deref-extend.txt	crates/singlefs-core/src/transaction.rs:6612	deadlist_changes_of_this_publish	G0 不走到	快照钉住的角色：G0 没有快照	user_visible_roles_kept_for_the_snapshots.push(*identity);
pre-push-deref-extend.txt	crates/singlefs-core/src/transaction.rs:8017	accounting_entries_of_this_publish	不随 C、R、U 涨	照抄上一版的台阶行（模型取一行，记账条目 E_A 里那一行）	entries.extend_from_slice(inode_number_watermark_steps_carried_from_the_previous_version);
pre-push-deref-extend.txt	crates/singlefs-core/src/transaction.rs:8314	inode_record_writes	不随 C、R、U 涨	这次新写的 inode 记录（新装）	records.extend_from_slice(self.new_inode_records);
```

归属表准入后三份输出（第 98–202 行）：

```text
post-clone-to_vec.txt	crates/singlefs-core/src/allocation_record_tree.rs:101	of_devices	不随 C、R、U 涨	每块盘一项（两块盘）	let mut slots_of_each_device = slots_of_each_device.to_vec();
post-clone-to_vec.txt	crates/singlefs-core/src/allocator.rs:1780	oldest_txg_in_the_ring_rereading_slots_bad_at_mount	不随 C、R、U 涨	根环里有效根的 txg	let mut valid_root_txgs: Vec<CheckpointTxg> = valid_root_txgs.to_vec();
post-clone-to_vec.txt	crates/singlefs-core/src/code_two_tree.rs:88	new	现算重建·进模型	被调方：15623 起全部映射 key 建成 CodeTwoTreeKey（现算重建表那一行）；别的调用方一次一把	bytes: key_bytes.to_vec(),
post-clone-to_vec.txt	crates/singlefs-core/src/code_two_tree.rs:163	parse_internal_entry	调用名误收	去掉经核只经同名调用收进来的四个函数（mount.rs:1251 units、recovery.rs:906 与 6266 tree、make_filesystem.rs:605 make_filesystem）之后收不到	Ok((entry[..key_width_in_bytes].to_vec(), child))
post-clone-to_vec.txt	crates/singlefs-core/src/code_two_tree.rs:1193	key_ranges_in_bump_order	不随 C、R、U 涨	每个重写节点的 key 区间两把	(smallest.bytes().to_vec(), largest.bytes().to_vec())
post-clone-to_vec.txt	crates/singlefs-core/src/extent_tree.rs:581	parse	调用名误收	去掉经核只经同名调用收进来的四个函数（mount.rs:1251 units、recovery.rs:906 与 6266 tree、make_filesystem.rs:605 make_filesystem）之后收不到	.to_vec();
post-clone-to_vec.txt	crates/singlefs-core/src/make_filesystem.rs:374	location_entries	不随 C、R、U 涨	pool.location_entries 一次一个单元：两块盘的身份	let mut sorted: Vec<DeviceIdentity> = devices.to_vec();
post-clone-to_vec.txt	crates/singlefs-core/src/mount.rs:1513	record_root_written_by_this_process_rereading_root_ring_slots_unreadable_at_mount	不随 C、R、U 涨	实例表的行，随实例数（模型 P_IT 取 1 片），不随 C、R、U 涨	|occupancy| occupancy.instance_table_judging_the_rereads().clone(),
post-clone-to_vec.txt	crates/singlefs-core/src/mount.rs:8711	read	不随 C、R、U 涨	读新槽残留那一读（read_the_residue_of_the_new_slots）经同名 read / read_at 收进的读者；一次读一个单元	return Some(bytes.clone());
post-clone-to_vec.txt	crates/singlefs-core/src/mount.rs:8717	read	不随 C、R、U 涨	读新槽残留那一读（read_the_residue_of_the_new_slots）经同名 read / read_at 收进的读者；一次读一个单元	.insert(placement, bytes.clone());
post-clone-to_vec.txt	crates/singlefs-core/src/mounted_session.rs:793	keep_in_memory	调用名误收	去掉经核只经同名调用收进来的四个函数（mount.rs:1251 units、recovery.rs:906 与 6266 tree、make_filesystem.rs:605 make_filesystem）之后收不到	.insert(first_block + block_offset, block.to_vec());
post-clone-to_vec.txt	crates/singlefs-core/src/mounted_session.rs:1032	read_unrecorded_blocks_from_the_disk	不随 C、R、U 涨	读新槽残留那一读（read_the_residue_of_the_new_slots）经同名 read / read_at 收进的读者；一次读一个单元	recorded.insert(block_index, block.to_vec());
post-clone-to_vec.txt	crates/singlefs-core/src/mounted_session.rs:1121	record_the_written_blocks	调用名误收	去掉经核只经同名调用收进来的四个函数（mount.rs:1251 units、recovery.rs:906 与 6266 tree、make_filesystem.rs:605 make_filesystem）之后收不到	recorded.insert(first_block + block_offset, block.to_vec());
post-clone-to_vec.txt	crates/singlefs-core/src/mounted_session.rs:1164	block_device_error_with_the_same_content	不随 C、R、U 涨	读新槽残留那一读（read_the_residue_of_the_new_slots）经同名 read / read_at 收进的读者；一次读一个单元（出错那一支）	BlockDeviceError::ImageFileAlreadyExists { path: path.clone() }
post-clone-to_vec.txt	crates/singlefs-core/src/mounted_session.rs:1167	block_device_error_with_the_same_content	不随 C、R、U 涨	读新槽残留那一读（read_the_residue_of_the_new_slots）经同名 read / read_at 收进的读者；一次读一个单元（出错那一支）	BlockDeviceError::ImageFileMissing { path: path.clone() }
post-clone-to_vec.txt	crates/singlefs-core/src/mounted_session.rs:1209	keep_the_self_verified_reread	调用名误收	去掉经核只经同名调用收进来的四个函数（mount.rs:1251 units、recovery.rs:906 与 6266 tree、make_filesystem.rs:605 make_filesystem）之后收不到	recorded.insert(block_index, block.to_vec());
post-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:13276	carried_unit	照抄·进模型	丙5：carried_unit，没重写的单元整份拷单元字节	units_of_the_previous_version.unit(identity).clone()
post-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:13476	build_extent_tree	照抄·不进模型	extent 树照抄节点：模型 D = 0 时没有、D = 32768 那一点也不建 extent 树	bytes: carried.bytes.clone(),
post-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:13912	build_inode_tree_units	照抄·进模型	(a) inode 叶没重写那一支再抄一次	contents: contents.clone(),
post-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:13947	build_inode_tree_units	照抄·进模型	(a) inode 叶重写那一支再抄一次	contents: contents.clone(),
post-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:14144	build_container_tree_units	照抄·进模型	丙1 新版每片容器内容再抄一次	contents: contents.clone(),
post-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:14161	build_container_tree_units	建索引或遍历（不进模型）	拷一个 Range；外层是逐个节点的遍历	.clone();
post-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:14324	build_deletion_tree_units	不随 C、R、U 涨	deleted_inodes 根的首尾 key	let smallest_key = entries.first().expect("队列非空才有根").clone();
post-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:14325	build_deletion_tree_units	不随 C、R、U 涨	同上	let largest_key = entries.last().expect("队列非空才有根").clone();
post-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:14364	build_deletion_tree_units	不随 C、R、U 涨	logged_ops 根的 key	let key_of = |entry: &Vec<u8>| entry[..LOGGED_OPERATIONS_KEY_WIDTH_IN_BYTES].to_vec();
post-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:14640	carried_file_version_units	照抄·进模型	丙5 第一个文件照抄的数据单元字节抄进 data_units（第三份新补成一项：这些单元在 16008 经 carried_unit 再抄一份）	.clone()
post-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:14643	carried_file_version_units	照抄·进模型	第三份新补：第一个文件照抄的数据指针	data_pointers: carried.data_pointers[..carried_data_units].to_vec(),
post-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:14750	publish_admitted	G0 不走到	整理的一批	bytes: relocation.bytes.clone(),
post-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:14760	publish_admitted	G0 不走到	整理的一批	mapping_key: relocation.mapping_key.clone(),
post-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:14773	publish_admitted	G0 不走到	整理的一批	.map(|(_, _, relocated)| (relocated.mapping_key.clone(), relocated.destination))
post-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:14787	publish_admitted	G0 不走到	快照留存的映射条目：没有快照时 0 条	key.clone(),
post-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:14795	publish_admitted	G0 不走到	整理的一批：整份克隆上一版（destination_of_key 非空才走）	let mut relocated_version = previous_version.clone();
post-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:15072	publish_admitted	G0 不走到	写第一个文件之外的文件	(file_plan.inode, written_units.data_pointers.clone())
post-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:15204	publish_admitted	照抄·进模型	allocator.records().to_vec() 整份分配记录	let mut allocation_records: Vec<AllocationRecord> = allocator.records().to_vec();
post-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:15616	publish_admitted	现算重建·进模型	mapped_units：全部 key 再抄一份	.map(|(identity, key, _)| (*identity, key.clone()))
post-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:15640	publish_admitted	现算重建·进模型	mapping_keys：全部 key 字节再抄一份	.map(|key| key.bytes().to_vec())
post-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:15880	publish_admitted	不随 C、R、U 涨	销毁中的快照号	.map(|previous_version| previous_version.snapshots_being_destroyed.clone())
post-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:16016	publish_admitted	新装第二份	重写的数据单元装进 units	None | Some(_) => rewritten_unit(identity, data_unit.clone()),
post-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:16034	publish_admitted	新装第二份	写的那个文件的数据单元装进 units	.clone(),
post-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:16051	publish_admitted	新装第二份	改 size / 部分写重写的数据单元装进 units	.clone(),
post-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:16070	publish_admitted	新装第二份	重写的 inode 叶装进 units	rewritten_unit(identity, container.bytes.clone())
post-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:16089	publish_admitted	新装第二份	重写的 inode 根装进 units	Some(root) => rewritten_unit(TransactionUnit::InodeRoot, root.bytes.clone()),
post-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:16106	publish_admitted	新装第二份	重写的分配记录树节点装进 units	Some(allocation_record_tree_node_unit) => allocation_record_tree_node_unit.clone(),
post-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:16120	publish_admitted	新装第二份	删除那两棵树重写的节点装进 units	Some(rewritten_node) => rewritten_unit(identity, rewritten_node.bytes.clone()),
post-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:16134	publish_admitted	新装第二份	容器树重写的节点装进 units	Some(rewritten_node) => rewritten_unit(node.identity, rewritten_node.bytes.clone()),
post-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:16194	publish_admitted	新装第二份	树表装进 units	tree_table_unit.clone(),
post-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:16200	publish_admitted	新装第二份	实例表重写的那几片（G0 不重写）	units.push(rewritten_unit(page.role, page.bytes.clone()));
post-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:16553	publish_admitted	不随 C、R、U 涨	这次记录点名的单元	named: named_units.clone(),
post-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:16567	publish_admitted	不随 C、R、U 涨	这次的 journal 记录字节	bytes: written_record.bytes.clone(),
post-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:16591	publish_admitted	不随 C、R、U 涨	这次重写的角色	rewritten: rewritten.to_vec(),
post-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:16616	publish_admitted	现算重建·进模型	deadlist 条目抄进新版（E'_D）	deadlist_entries: resolved.deadlist.entries_after_this_publish.clone(),
post-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:16623	publish_admitted	现算重建·进模型	deleted_inodes 的号抄进新版（Q'）	deleted_inodes: resolved.deleted_inodes.clone(),
post-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:16624	publish_admitted	现算重建·进模型	logged_ops 条目抄进新版（G0 为空）	logged_operations: resolved.logged_operations.clone(),
post-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:16635	publish_admitted	照抄·进模型	丙6 墓碑树形状再抄一次	shape: resolved.tombstone_tree.shape.clone(),
post-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:16653	publish_admitted	照抄·进模型	丙6 谱系树形状再抄一次	shape: resolved.lineage_tree.shape.clone(),
post-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:16671	publish_admitted	照抄·进模型	丙6 退役树形状再抄一次	shape: resolved.retirement_tree.shape.clone(),
post-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:16688	publish_admitted	不随 C、R、U 涨	这次释放的	released: release.to_vec(),
post-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:16689	publish_admitted	不随 C、R、U 涨	这次隔离的	quarantined_after_release_checksum_mismatch: quarantine.to_vec(),
post-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:16813	build_multi_level_tree	照抄·进模型	(b) 多层码 2 树没重写的节点整份拷单元字节	bytes: carried.bytes.clone(),
post-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:16822	build_multi_level_tree	现算重建·进模型	重写的映射树叶的每条条目字节	.map(|key| leaf_entry_bytes_by_key[key].clone())
post-clone-to_vec.txt	crates/singlefs-core/src/transaction.rs:16880	build_multi_level_tree	照抄·进模型	(c) 多层码 2 树这一版的形状	shape: plan.shape.clone(),
post-cloned-copied-copy_of.txt	crates/singlefs-core/src/allocation_record_tree.rs:376	nodes_whose_contents_changed	建索引或遍历（不进模型）	两版有记录的叶号并集（叶号 16 字节，按叶数），是建索引，同第三节「不进模型的」口径	before.keys().chain(after.keys()).copied().collect();
post-cloned-copied-copy_of.txt	crates/singlefs-core/src/allocation_record_tree.rs:465	children_of	不随 C、R、U 涨	一个节点的孩子（扇出以内）	.copied()
post-cloned-copied-copy_of.txt	crates/singlefs-core/src/allocator.rs:1371	placements_named_by_unapplied_journal_records	不随 C、R、U 涨	隔离保护的落点：还没落盘的 journal 记录点名的	.copied()
post-cloned-copied-copy_of.txt	crates/singlefs-core/src/allocator.rs:1643	placements_referenced_by_abandoned_roots	不随 C、R、U 涨	隔离保护的落点：被抛弃的根点名的	.copied()
post-cloned-copied-copy_of.txt	crates/singlefs-core/src/allocator.rs:1783	oldest_txg_in_the_ring_rereading_slots_bad_at_mount	不随 C、R、U 涨	同上	let oldest_valid_root = valid_root_txgs.iter().copied().min();
post-cloned-copied-copy_of.txt	crates/singlefs-core/src/allocator.rs:2616	reclaim_released_records_up_to	不随 C、R、U 涨	这次回收的那几条记录	.copied()
post-cloned-copied-copy_of.txt	crates/singlefs-core/src/code_two_tree.rs:1126	flatten_read	调用名误收	去掉经核只经同名调用收进来的四个函数（mount.rs:1251 units、recovery.rs:906 与 6266 tree、make_filesystem.rs:605 make_filesystem）之后收不到	leaf_entries_in_key_order.extend(node.header.entries.iter().cloned());
post-cloned-copied-copy_of.txt	crates/singlefs-core/src/container_tree.rs:1498	clone	照抄·进模型	被调方：容器树形状 Clone 的实现，计在调它的那几行（准入前 3576、8640、8965；准入后丙6）	children_of_each_node_at_each_level: copy_of_the_children_counts(&self.children_of_each_node_at_each_level),
post-cloned-copied-copy_of.txt	crates/singlefs-core/src/container_tree.rs:1520	copy_of_the_children_counts	照抄·进模型	被调方：同上，逐层照抄孩子数	copy.push(copy_of_one_level(&children_of_each_node_at_each_level[level_index]));
post-cloned-copied-copy_of.txt	crates/singlefs-core/src/mounted_read.rs:975	read_at	不随 C、R、U 涨	读新槽残留那一读（read_the_residue_of_the_new_slots）经同名 read / read_at 收进的读者；一次读一个单元	.copied()
post-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:3558	count	不随 C、R、U 涨	只供测试的查找计数（lookup-work-counters 特性，默认关）	let site = state.sites_entered.last().copied();
post-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:3693	first_entry_of	不随 C、R、U 涨	取一项	.copied()
post-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:4251	inode_number_watermark_steps	不随 C、R、U 涨	inode 号水位的台阶行（模型取一行），不随 C、R、U 涨	.copied()
post-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:6464	snapshot_inode_root_entry	G0 不走到	快照：G0 没有快照	.copied()
post-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:13161	roles_named_by_each_record_of_the_publish	不随 C、R、U 涨	这次发布的记录点名的角色	.copied()
post-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:13210	roles_other_than_the_data_roles_named_before_the_last_record	不随 C、R、U 涨	同上	.copied()
post-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:13686	build_extent_tree	照抄·不进模型	extent 树重写的叶装条目	.copied()
post-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:13712	build_extent_tree	照抄·不进模型	extent 树重写的节点装孩子	.copied()
post-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:14184	build_container_tree_units	不随 C、R、U 涨	装好的孩子节点的指针	.copied()
post-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:14686	publish_admitted	不随 C、R、U 涨	取一个号	.copied()
post-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:14702	publish_admitted	不随 C、R、U 涨	取一条记录（外层遍历）	.copied();
post-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:14788	publish_admitted	G0 不走到	同上	destination_of_key.get(key).copied().unwrap_or(*locations),
post-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:14797	publish_admitted	G0 不走到	整理的一批	destination_of_key.get(key).copied()
post-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:15001	publish_admitted	不随 C、R、U 涨	取一条记录（外层遍历）	.copied()
post-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:15085	publish_admitted	G0 不走到	改 size 或部分写第一个文件之外的文件	.copied()
post-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:15124	publish_admitted	照抄·不进模型	第一个文件之外有数据的文件照抄数据指针；模型里别的文件不带数据单元	.copied()
post-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:15170	publish_admitted	不随 C、R、U 涨	取一条记录（外层遍历）	.copied();
post-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:15413	publish_admitted	不随 C、R、U 涨	取一个指针	.copied()
post-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:16065	publish_admitted	照抄·不进模型	extent 树那一批单元装进 units（模型 D = 0 没有照抄节点；重写节点那一份是新装第二份）	units.extend(extent_built.units.iter().cloned());
post-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:16116	publish_admitted	照抄·进模型 + 新装第二份	(b) 那一批单元再抄一次：记账树（照抄节点 0）	units.extend(accounting_built.units.iter().cloned());
post-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:16117	publish_admitted	照抄·进模型 + 新装第二份	(b) 那一批单元再抄一次：deadlist（G0 零根）	units.extend(deadlist_built.units.iter().cloned());
post-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:16191	publish_admitted	照抄·进模型 + 新装第二份	(b) 那一批单元再抄一次：中央映射树	units.extend(mapping_built.units.iter().cloned());
post-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:16209	publish_admitted	照抄·进模型	实例表照抄的那几片从上一版 units 整份拷	.cloned(),
post-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:16479	publish_admitted	不随 C、R、U 涨	这次记录点名的单元	.map(|named_roles| named_roles.iter().copied().map(named_unit_of).collect())
post-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:16491	publish_admitted	不随 C、R、U 涨	同上	.chain(rewritten.iter().copied().map(named_unit_of))
post-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:16521	publish_admitted	不随 C、R、U 涨	取一个号	.copied();
post-cloned-copied-copy_of.txt	crates/singlefs-core/src/transaction.rs:16562	publish_admitted	新装第二份	要写的单元（搬走的拷贝与重写的单元）再抄进 PublishWrites	units: written_units.into_iter().cloned().collect(),
post-push-deref-extend.txt	crates/singlefs-core/src/allocation_record_tree.rs:327	records_of_each_leaf	照抄·进模型 + 现算重建·进模型	上一版分配记录按叶分组（照抄）与这一版的按叶分组（现算重建），两张表各有一行	.push(*record);
post-push-deref-extend.txt	crates/singlefs-core/src/container_tree.rs:1520	copy_of_the_children_counts	照抄·进模型	被调方：同上，逐层照抄孩子数	copy.push(copy_of_one_level(&children_of_each_node_at_each_level[level_index]));
post-push-deref-extend.txt	crates/singlefs-core/src/container_tree.rs:1540	copy_of_one_level	照抄·进模型	被调方：同上，逐个节点照抄孩子数	copy.push(children_of_each_node[node]);
post-push-deref-extend.txt	crates/singlefs-core/src/logged_operations.rs:818	logged_operations_root_entries	不随 C、R、U 涨	logged_ops 至多一条	root_entries.push(entries[written_count].to_bytes());
post-push-deref-extend.txt	crates/singlefs-core/src/mounted_read.rs:1014	read_at	不随 C、R、U 涨	读新槽残留那一读（read_the_residue_of_the_new_slots）经同名 read / read_at 收进的读者；一次读一个单元	bytes.extend_from_slice(
post-push-deref-extend.txt	crates/singlefs-core/src/transaction.rs:4262	inode_number_watermark_steps	不随 C、R、U 涨	同上	steps.push(*row);
post-push-deref-extend.txt	crates/singlefs-core/src/transaction.rs:8017	accounting_entries_of_this_publish	不随 C、R、U 涨	照抄上一版的台阶行（模型取一行，记账条目 E_A 里那一行）	entries.extend_from_slice(inode_number_watermark_steps_carried_from_the_previous_version);
```

**命令**（原样；`$S` = `~/.local/share/singlefs/work/e199-designer-revision-3/sweep/callgraph.py`，`$R` = 仓根 `/home/fy5090/code/singlefs`，`$B` = `~/.local/share/singlefs/work/m4-close-code-r1-fix-b2-writer/base`；在 `$D/sweep/today/` 里起，`$D` = `~/.local/share/singlefs/work/e199-designer-revision-3`；第一个文件也照抄在第十三节）：

```
python3 -I $S $R list judge_the_publish_before_any_read_or_write > pre-functions.txt
python3 -I $S $R list publish_admitted > post-functions.txt
python3 -I $S $R grep judge_the_publish_before_any_read_or_write '\.clone\(\)|\.to_vec\(\)' > pre-clone-to_vec.txt
python3 -I $S $R grep publish_admitted '\.clone\(\)|\.to_vec\(\)' > post-clone-to_vec.txt
python3 -I $S $R grep judge_the_publish_before_any_read_or_write '\.cloned\(\)|\.to_owned\(\)|\.copied\(\)|\bcopy_of_[a-z_]+\s*\(' > pre-cloned-copied-copy_of.txt
python3 -I $S $R grep publish_admitted '\.cloned\(\)|\.to_owned\(\)|\.copied\(\)|\bcopy_of_[a-z_]+\s*\(' > post-cloned-copied-copy_of.txt
python3 -I $S $R grep judge_the_publish_before_any_read_or_write '\.push\(\*|extend_from_slice\(|\.push\(\s*[a-z_]+\[|copy\.push\(' > pre-push-deref-extend.txt
python3 -I $S $R grep publish_admitted '\.push\(\*|extend_from_slice\(|\.push\(\s*[a-z_]+\[|copy\.push\(' > post-push-deref-extend.txt
python3 -I $S $R skipped > skipped.txt
python3 -I $S $R trace judge_the_publish_before_any_read_or_write <命中所在的函数名…> > pre-traces.txt      # 收到的次序里从根到它的一条调用链，归属时读
python3 -I $S $R trace publish_admitted <命中所在的函数名…> > post-traces.txt
python3 -I $S $R grep judge_the_publish_before_any_read_or_write '<三组合一>' > pre-all-three-patterns.txt
python3 -I $S $R grep judge_the_publish_before_any_read_or_write '<三组合一>' --exclude=mount.rs:1251,recovery.rs:906,recovery.rs:6266,make_filesystem.rs:605 > pre-all-three-patterns-without-misnamed.txt
（准入后两条同形，根换成 publish_admitted）
diff pre-all-three-patterns.txt pre-all-three-patterns-without-misnamed.txt > misnamed-pre.diff      # 11 行「<」
diff post-all-three-patterns.txt post-all-three-patterns-without-misnamed.txt > misnamed-post.diff   # 6 行「<」
python3 -E -s ../classify_r3.py . && mv classification.tsv.new classification.tsv                   # 打「lines 202 unclassified 0」与各类条数
（基线：在 $D/sweep/baseline-core/ 里把上面前八条的 $R 换成 $B、末尾加 --directories=crates/singlefs-core/src）
```

`<三组合一>` 是三组写法用 `|` 接成的一个正则。输出文件的 sha256 在第十三节。

**归属里「进模型」那几类**（照抄·进模型 41、现算重建·进模型 8，另有带着别的类的 5 行：照抄·进模型 + 新装第二份 3、照抄·进模型 + 现算重建·进模型 2）落到的每一处，都在上面「甲臂的照抄点」「现算重建」两张表里有一行（被调方那几行计在调它的那一行，理由列写明是哪一行）；两张表里另有几行不是这三组写法的命中（`12510-12512` 那一次调用、`13045` `node_set()`、`16107` / `16009` 等经 `carried_unit` 的那几处、`13776`），是读代码补的，执行员照停机条款第 1 条 ① 逐行 `grep` 核。「照抄·不进模型」6 行：extent 树（`13476`、`13686`、`13712`、`16065`：模型 D = 0 时那棵树没有照抄节点，D = 32768 那一点也只算数据单元、不建 extent 树）、最后一批有数据的文件号集合（`unlink.rs:313`，模型里至多一个号）、第一个文件之外有数据的文件照抄数据指针（`15124`，模型里别的文件不带数据单元）。「射程外」1 行：每轮整份拷分配器（`13098`，拷的是分配器，不是上一版 `TransactionOutput`）。

不进模型的另几样（照第二份第三节）：`let mut rehearsal = allocator.clone()`（`13098`）与外层 `allocator.clone()`（`12314`）——拷的是分配器；`units_by_role`（`15061`）、`mapped_units_by_role`（`9048-9049`）、`files_other_than_the_first_by_inode`（`15058-15059`）、`node_origins` / `children_of` 与列全部容器、列全部节点那几遍、两版有记录的叶号并集（`allocation_record_tree.rs:376`）——是遍历或建索引，不是照抄，也不按全部单元现算一份与上一版相同的内容（读法第 8 条）。整理的一批整份克隆上一版（`14795`，`destination_of_key` 非空才走）归「G0 不走到」：模型的取样点里没有整理的发布。它们与四条臂的关系另记在第十一节「够判后未跑」。

### 丙要改的调用点：今天借上一版的地方

`grep -rn '&TransactionOutput' crates/ --include=*.rs` 现跑 469 处命中（2026-11-03，这一棵树；第二份 459）；回退入口 `mount.rs:10284` `roll_back_by_a_forward_publish`、`10592` `roll_back_by_a_forward_publish_in_the_session`；会话持有现行那一版 `mounted_session.rs:108` `pub current: PoolVersion`，`2620` `let mut walked = current_version_of_trees.clone()`。分类与改法由执行员按第六节「丙调用点清单」那一行的命令做。

## 四、跑之前已经存在的数

读条款、读前两份登记、读两份执行员报告、做第 3a 步时已经知道或自己算出的数，照实列。现算的命令与原样输出在设计员草稿目录 `~/.local/share/singlefs/work/e199-designer-revision-3/calc/`：`model_r3.py` 按第二份第五节的式子重建甲-B2后准入前（O、Rl、Once）并加这一份补的两项，`anchors_r3.py` 打锚点；两份源码与原样输出整份抄在第七节。第二份的 `model_r2.py`、`anchors_r2.py`、`extra_r2.py`、`items_r2.py` 与它们的输出都已不在，第二份引它们的数这一份只当「已存在的数」读，不重算——`model_r3.py` 只重建了准入前甲-B2后那一项，拿它把第二份第七节准入前甲-B2后的 13 个锚点、甲-今天 G0 与丁 256 次甲准入前合计逐个复现（第七节「复现」那几行全是「相同」），复现得了才在它上面加新项。数都按第五节公式，标「推的」——是公式的算术，不是量的。

| 数 | 出处 | 对判据的影响 |
|---|---|---|
| 第二份第四节整张表（G0 甲-B2后准入前 404072116、准入后 81200231，新装 14232 / 610728，\|V\| 81793847，现算重建 1814496 / 925712，比值、G-small、等高点、阳性对照、丁 256 次各数） | 读第二份时读到（推的，按第二份的式子） | 准入后、新装、现算重建、乙2、丙、丁各数这一份的式子不改（新补的 `8640`、`8965` 都在准入前；`14640`、`14643` 只在 D > 0 时不为 0），照第二份第七节钉；准入前甲-B2后各数改成下面几行 |
| 调查报告的 C = 1227、N = 11、每次发布照抄单元字节约 38.4 MiB | 调查报告结论第 2 条（推的） | 第七节锚点自己现算出同样的 C、层数，对不上照第七节办 |
| 10⁶ 次 unlink 的墓碑约 76.7 MiB（两份） | D18（块里携带什么信息） 已定项 24 第 4 条 | 盘上两份的量，不是内存照抄；不进判据 |
| 第二份执行员核对时的读数：三份实现文件 sha256（今天 `1178862c…`、`392e25fc…`、`2b8d27e0…`）、各常量与第五节相同、`entries == 135` 在 `container_tree.rs:173`、补丁已在主工作区、照抄那一支两处新抄的形状量级按式子推 128 与 680 字节 | 执行员报告 `e199-runner-segment-1-r2/result.md`（量的：sha256、grep；推的：量级） | 只是现状；量级与下面 `model_r3.py` 现算的相同 |
| G0 甲-B2后准入前照抄 404072244（O = 80346376、Rl = 585019、Once = 288；比第二份多 128 = s·shape(1, 1) = 4 × 32，全是 `8965`）；`8640` 在 G0 为 0（意图走追加那一支） | `anchors_r3.py` G0 一行（推的） | G0 上任何一格比值落在 2 的哪一侧都不变：甲 ÷ 乙2 准入前形式 A 200.6006 → 200.6006、形式 B 199.2002 → 199.2003 |
| 不删除的发布（a_T = 0）甲-B2后准入前 203729488（比第二份多 808：`8640` 5 × 136 = 680、`8965` 128） | `anchors_r3.py`（推的） | 只在 a_T = 0 那一点上 `8640` 不为 0；第九节「漏掉 `8640`」那条变异靠这一点抓 |
| 甲-今天 − 甲-B2后（G0）1808（第二份 1936；甲-今天自己 404074052 不变） | `anchors_r3.py`（推的） | 第六节「甲-今天 − 甲-B2后」那一行的数 |
| 逐一扫的点上甲-B2后准入前都多 s·32（u 扫、φ、碰叶档、k 扫、开新片、等高点、阳性对照两点：s = 4 时 +128；s = 1 +32，s = 64 +2048）；G-small 不变（R_L = 0，谱系树零根、形状 0） | `anchors_r3.py`（推的） | 阳性对照 u = 10⁶ ÷ u = 10⁵ 的准入前比 9.7597 不变（404072244 ÷ 41402308）；G-small 上甲 ÷ 乙2 准入前 ≈ 0.75 不变 |
| 丁 256 次（G0 起连发 256 次意图）甲准入前合计 103456147732（比第二份多 32768 = 256 × 128） | `anchors_r3.py`（推的） | 丁压实摊销那几行用准入后照抄，不受影响 |
| 全扫新补的两项：D = 32768（其余 G0）时 `14640` 32768 × 32792 = 1074528256、`14643` 32768 × 88 = 2883584，合 1077411840；G0（D = 0）两项都是 0 | 命令现算（推的，第十三节） | 只在 D = 32768 那一点上改准入后与一版大小；那一点第二份没钉锚点，判定的几格（第六节）都在 G0 与第八节的点上，D 只在逐一扫里报数 |
| 全扫：202 条命中、各类条数（第三节） | `classify_r3.py` 现跑（量的，源文件） | 进模型那几类都在第三节两张表里；新补的四处（`8640`、`8965`、`14640`、`14643`）进表 |
| `&TransactionOutput` 在 `crates/` 里 469 处命中 | 第三节现跑 | 丙调用点清单的规模；不进比值 |

第 3a 步 ② 现算的各臂基线符号与量级（G0，推的）同第二份：照抄字节甲 ~4 × 10⁸（准入前）/ ~8 × 10⁷（准入后）为正；乙2 ~2 × 10⁶ / ~4 × 10⁵ 为正；丙 0 / ~5 × 10³；丁 0 / 0（非压实那次）与 ~8 × 10⁷（压实那次）。现算重建甲 ~2 × 10⁶ / ~9 × 10⁵；乙2、丙、丁非压实那次 ~6 × 10⁴ / ~1 × 10⁵。新补两处在 G0 准入前合 128 字节，相对甲的 ~4 × 10⁸ 是 3 × 10⁻⁷，计数分配器按字节计、分得出（第六节「可跑性」列）。丙、丁在形式 A 上分母为 0，由读法第 9 条定。

第 3a 步 ③：新补两处的触发条件写死——`8640` 在这次的删除步不是意图（`8631-8641` 那八种，模型里 a_T = 0）时走到，第八节逐一扫的 a_T = 0 那一点会发生；`8965` 在这次不是管理员回退（模型里 a_L = 0）时走到，G0 会发生；`14640`、`14643` 在这次不写第一个文件、上一版有第一个文件（`14880` 那一支）时走到，D > 0 时才拷到东西，D = 32768 那一点会发生。

## 五、臂、阳性对照、真实基线

**装置写在哪（写死）**：两段都走 `scripts/experiment-apparatus/`（独立手写模型，不与实现共用代码，`.claude/rules/implementation-first.md` 第 4 条）。源文件与 bin：第一段 `e199_transaction_output_copy.rs` / `e199-transaction-output-copy`（纯算术）；第二段 `e199_transaction_output_copy_executed.rs` / `e199-transaction-output-copy-executed`（真建内存结构、计数分配器、计时）。`crates/` 一行都不引；第三节的文件:行只当照抄点清单。第三段不做（读法第 10 条）。两段都不走崩溃判段，不带 `--features gpu`。

### 模型常量

第二份第五节「模型常量」两张表照留（第一份那张：`UNIT_BYTES` 32768、`NODE_BYTES` 16384、扇出 f = 135、cap_T 815、cap_L 1359、cap_X 959、inode 叶 233、内存宽度：墓碑记录 40、谱系记录 24、退役记录 40、inode 记录 64、分配记录 40、容器外壳 56、单元外壳 48、形状每节点 8 加每层 24、固定点单元 F = 16、分配记录条数 A = 2U；第二份加的：多层码 2 树叶 / 内部容量 477 / 150、376 / 125、294 / 143，映射 key 27、记账 key 22、deadlist key 43，key 字段数 5 / 4 / 7，映射条目 55，分配记录树叶罩 812 槽、内部扇出 169，E_A = 16，P_IT = 1，key 结构与几样集合项的内存宽度），另加：

| 常量 | 值 | 出处 | 类 |
|---|---|---|---|
| 数据指针 W_DP | 88 | `crates/singlefs-core/src/pointer.rs:828` 注释「指向码 1 数据单元的指针 88：头部 50 + 位置条目 14 × 2 + 写序 10（D19（块指针的结构与宽度预算） 已定项 8）」；装置写成自己的 `[u8; 88]` 类型，`size_of` 回比 | 模型选择（取实现值） |
| 照抄的数据单元字节项 | 32792 = 32768 + `Vec` 头 24（`14640` 抄进 `data_units` 的是裸 `Vec<u8>`，没有单元外壳） | 装置自己的类型，`size_of` 回比 | 模型选择 |

### 几何与派生量（第一段与第二段同一套）

第二份第五节「几何与派生量」逐条照留（几何 G = (u, a_T, R_X, a_X, R_L, a_L, I, D, s, φ, 碰叶档)；容器树各式；核心单元 U_c、r_c、n_c；分配记录树 Sl、n_ℓ、N_AR、d_AR、τ_AR、ρ_AR、κ_AR；中央映射树 M、r_M、n_M、d_M、c、c_i、N_M、τ_M、ρ_M、κ_M；记账树与 deadlist；全部单元 U、r、n_new、A、Q'；shape 与 shape_ml）。「追加那一支」照第一份：墓碑树 a_T > 0、谱系树 a_L > 0、退役树**每次都走**；「不追加那一支」是它的反面：墓碑树 a_T = 0、谱系树 a_L = 0（退役树没有这一支）。改的只有一版大小：

- **一版大小** |V| = 第二份那一式 + 88·D（这一版的 `data_pointers`，`transaction.rs:3211`）；|V'| 照这一版的量，另加 32·Q'（同第二份）。

### 四条臂（「怎么做」与「做完会怎样」）

第二份第五节那张表的每一格照留，下面逐臂只写这一份多要求的那一句：

| 臂 | 多要求的「怎么做」 | 做完会怎样（由左推出） | 现算重建那一类省不省得掉 |
|---|---|---|---|
| **甲-B2后**（真实基线） | 第三节甲臂表「留着」那一列每一处都照抄，含不追加那一支每遍抄一份上一版形状（`8640` s + 1 遍、`8965` s 遍）与第一个文件照抄的数据单元两份、数据指针一份 | 照抄量 ∝ 一版大小，随 C、R 一次涨；不追加的那几棵树每遍多一份 O(N) 的形状 | 省不掉（同第二份） |
| **甲-今天**（对照） | 照基线代码：甲-B2后加「去掉」那一列，减去不追加那一支在基线里本来就有的那一份（基线不追加那一支抄两份：甲2 一份、甲3 一份；补丁之后剩调用方那一份） | 同上，多几份形状 | 同甲-B2后 |
| **乙2 持久结构** | 不追加的树的形状同样是持久结构、整份共享，不复制；第一个文件没碰到时它的数据单元与指针表经引用计数共享 | 新补几样上乙2 多付 0 | 同第二份 |
| **乙1 每片引用计数**（附带，只报数、不判） | 「× 32792」那一项照第二份的规则换成「× 8」（只抄指针）；数据指针照甲（每个 88 照抄）；不追加那一支的形状照甲 | — | 省不掉（同甲） |
| **丙 按值交进来 + 撤回日志** | 不追加的树原样留在交进来的那一版里，形状不抄；数据单元与指针表就地留着 | 新补几样上丙多付 0 | 同第二份 |
| **丁 只记差量 + 压实** | 差量层不记没改的形状与没碰到的文件；整份新基照抄 \|V'\|（含 88·D） | 非压实那次多付 0；压实那次 \|V'\| 多 88·D | 同第二份 |

「支持这条路的人认不认」一列照前两份；这一份新补的几样里，乙2「形状整份共享」、丙「不追加的树原样留着」、丁「差量层不记没改的」都是这几条臂原定义（持久结构、就地改加撤回日志、差量层）落到新补几样上的唯一读法，没有替任何一臂挑不利的形态。

### 每次发布的照抄字节（第一段按此算；第二段的分配器计数与它比，第十一节作废条款第 1 条）

记号同第二份（P = 8b + 16，depth(n)，[条件]，PV(n, w, x, 块)）。

**甲-B2后**：准入前 = (s + 1)·O + s·Rl + Once；准入后一次。

- O = Σ_{t∈T,X} (R_t·W_t + 56·C_t) + 2·(64·I + 56·L_I) + Σ_{t∈T,X 追加} (R_t·W_t + 56·C_t + shape(N_t, h_t)) + **Σ_{t∈T,X 不追加} shape(N_t, h_t)**（第三份新补，`8640`；退役树每次追加，这一项实际是 [a_T = 0]·shape(N_T, h_T)）
- Rl = (R_L·W_L + 56·C_L) + [a_L > 0]·(R_L·W_L + 56·C_L + shape(N_L, h_L)) + **[a_L = 0]·shape(N_L, h_L)**（第三份新补，`8965`）+ (M − r_M)·67 + Σ_{t∈A,D,M} E_t·W_ck,t + Σ_{t∈A,D,M; N_t ≥ 1} (E_t + N_t − 1)·W_ck,t + 2·A·40
- Once = 2·N_AR·16
- 准入后 = 第二份那一式 + **D·(32792 + 88)**（第三份全扫新补，`14640`、`14643`）

**甲-今天**：准入前 = 甲-B2后（这一份的式子）+ (s + 1)·Σ_{t∈T,X} (2 + k_new_t − [t 不追加])·shape(N_t, h_t) + s·(2 + k_new_L − [a_L = 0])·shape(N_L, h_L)；准入后同甲-B2后（`14640`、`14643` 在基线 `transaction.rs` 里同样有）。按这一式与第二份那一式，甲-今天的绝对值逐点相同（G0 404074052）。

**乙2、丙、丁**：照抄字节的式子与第二份逐字相同（新补几样上多付 0，上一节）；丁压实那次整份新基 |V'| 用这一份的 |V'|（多 88·D）。

**乙1**（附带）：照上一节「四条臂」那一行的规则由甲-B2后的式子换出，只报数。

### 每次发布的现算重建字节、新装字节

两式与第二份逐字相同（新补几样都是照抄，不是现算重建，也不是新装）。**新装第二份**（只给甲，附带）照第二份那一式，另加一份：`16562` 把要写的单元（搬走的拷贝与重写的单元）再抄进 `PublishWrites`，按准入后新装里的单元字节那几项再算一份，与第二份那一份各报一格。

### 读路径与常驻峰值、第二段真建的模型、阳性对照、真实基线

照第二份第五节那四小节逐字照留，一版大小换成这一份的 |V|；第二段真建时，不追加那一支照第三节甲臂表每遍真抄一份上一版形状（甲-B2后），第一个文件照抄的数据单元抄两份、数据指针抄一份（D > 0 的点上）。阳性对照照第二份（甲 / 乙2 / 乙1 / 丙 / 丁退化形态、u = 10⁵ 与 u = 10⁶ 两点、比 ≥ 9；现算重建的阳性对照比 ≥ 8），这一份的式子下甲那一比是 404072244 ÷ 41402308 ≈ 9.7597（准入前）、81200231 ÷ 8452936（准入后，同第二份）。

### 实现量与分段（第 3b 步）

实现量（推的）：同第二份（第一段一个 bin 约 900 行加单测约 450 行、变异表约 40 行、`replay.sh` 一行 `exact`；第二段一个 bin 约 1300 行加单测约 500 行、变异表约 9 行、`replay.sh` 一行 `timing`，收尾行带 `loadavg1=`、`cpus=`），另加：两处不追加那一支的形状项与两项数据单元照抄各一个函数与各一条锚点用例（约 40 行），第九节多的 6 条变异。不另写检查器、崩溃枚举或坏镜像。

- **第一段**（纯算术，先跑）：补岔路单四行的「拷贝字节数」与增长阶、现算重建三组、乙行与丁行的读路径跳数、丁行的层数轨迹与压实摊销、丙行的调用点清单与绝对值锚点；先做第十一节停机条款第 1、2 条与第十节第一行。补完：四行都还差挂钟，**不够判**；丙行的调用点清单、丁行的合成层数与压实频率在第一段补齐。
- **第二段**（真建、计数分配器、计时）：补甲行的执行挂钟比、乙行的「乙随 C 涨」、读路径每次纳秒与常驻峰值，并用计数分配器互核第一段（作废条款第 1 条）。补完：四行都够判。
- 第三段不做（读法第 10 条）。

## 六、报哪些量与各自的判据

第二份第六节那张表逐行照留（每个量一行、各判各的，不合取；门槛、判定、对应岔路单哪一行与翻面取值一个字不改）。这一份只改「可跑性 / 分辨率」列与报数那几行里的数，逐行写在下面；没列的行与第二份逐字相同。G0 = 第八节主几何。

| 量（第二份第六节那一行） | 这一份改的 |
|---|---|
| 准入前照抄字节 | 甲-B2后多算 `8640`、`8965` 两项（第五节）；第一段每点仍是 O(层数) 次算术；第二段计数分配器粒度 1 字节，G0 上那 128 字节分得出 |
| 准入后照抄字节 | 甲-B2后多算 D·(32792 + 88)；G0 D = 0 不变 |
| 准入前字节比、准入后字节比（甲 ÷ 乙2、甲 ÷ 丙、甲 ÷ 丁，两种形式） | 可跑性列的数：G0 上甲 ÷ 乙2 准入前形式 A 200.6006、形式 B 199.2003（第四节），三对两种形式仍都 ≫ 2（结局已知），判别力仍来自第七节的绝对值与第八节方向相反的点 |
| 甲的增长阶 | 10⁵ → 10⁶ 准入前照抄斜率 log₁₀(404072244 ÷ 41402308) ≈ 0.98943（第二份 ≈ 0.989）；10⁶ → 10⁷ 斜率 log₁₀(4030722004 ÷ 404072244) ≈ 0.99892 |
| 甲-今天 − 甲-B2后 | G0 上 1808（第二份 1936） |
| 丙调用点清单 | 命中 469 处（这一棵树，第三节） |

另加一行（附带，够判后不跑）：

| 量 | 怎么算 | 门槛与判定 | 对应岔路单哪一行 / 翻面取值 | 可跑性 / 分辨率（第 3a 步） |
|---|---|---|---|---|
| 新补四项逐项（甲-B2后，每次发布） | `8640` = (s + 1)·[a_T = 0]·shape(N_T, h_T)、`8965` = s·[a_L = 0]·shape(N_L, h_L)、`14640` = 32792·D、`14643` = 88·D，每几何各报一格 | 报数，不判（附带，够判后不跑） | 无（说明甲-B2后比第二份多算了什么；判定照上面各行） | 算术；第七节逐项钉 |

**结果反过来接不接受（跑前写下）**：照第二份第六节那一段逐字照留（任何一对的字节比或执行挂钟比在 G0 上 < 2 照判「倾向留甲」，不改门槛、不换几何；乙2「乙随 C 涨」≥ 1% 照判、不换更强形态；丁每个 p 都 < 2 照判翻面；两种形式分到 2 的两侧照记「不稳定」，不挑一种形式下结论；现算重建比 < 2 照判「这一类上倾向留甲」，不把它并进照抄字节去改字节那几行的判定）。另加：新补四项在哪一格上让结局换了一侧，照记、照判，不为把它退回去而删项或改式子。

## 七、钉绝对值的断言

常量断言写加法、不写减法（`.claude/singlefs-ai-sop/rules/test-discipline.md`「变异测试证明的是断言会红，不是覆盖」第 1 条）。

### 出自被测条款本身的（不符走第十节失败条款「条款与实现不一致」，不作废）

第二份这一组照留：`UNIT_BYTES == 32768`；`NODE_BYTES == 16384`；`(115 + 2 * 8) + 135 * 120 + 53 == 16384`；`136 + 815 * 40 + 32 == 32768`；`136 + 1359 * 24 + 16 == 32768`；`136 + 959 * 34 + 26 == 32768`；`136 + 233 * 140 + 12 == 32768`。

### 取实现值的模型常量（不出自被测条款；不符走第十节第一行，模型照第五节常量算，报两边）

第二份这一组照留（`135 + 169 * 96 + 25 == 16384`；`ALLOCATION_RECORD_TREE_LEAF_SLOTS == 812`；多层码 2 树容量 (477, 150)、(376, 125)、(294, 143)；`3 + 2 * 6 + 1 == 16`），另加 `50 + 14 * 2 + 10 == 88`（数据指针）、`32768 + 24 == 32792`。

### 独立算出、用命令核过的（不符先对第五节公式逐项，两边各查；属文字有两种读法的交回主 agent，否则作废）

**第二份第七节「独立算出」那张表里不含甲-B2后准入前的格照留**（容器树形状、G0 两棵树、准入后照抄 81200231、新装、\|V\|、现算重建、(a)–(d) 逐项、G0 乙2 / 丙 / 丁、G-small 的乙2 / 丙 / 新装 / 现算重建、u = 10⁷ 准入后与 u = 10⁸ 两棵树形状、不删除的发布准入后 81266111、开新片 k_old / k_new 与准入后 81233498、k = 1001、φ = ½、集中档、等高点、阳性对照准入后、丁 256 次的整份新基各 p 合计与层数）；那几格出自第二份的 `model_r2.py` 等，草稿已不在，这一份没有重算（第四节第一行）。**含甲-B2后准入前的格改成下表**（`model_r3.py` 按第二份式子复现第二份的数、再按这一份式子算新数，两份源码与原样输出在下面）：

| 几何 | 断言（甲-B2后准入前照抄，这一份的式子） | 第二份那一格 |
|---|---|---|
| G0 | 404072244（O = 80346376、Rl = 585019、Once = 288）；甲-今天 404074052；甲-今天 − 甲-B2后 1808 | 404072116（Rl = 584987）；1936 |
| 取样 u | u = 0 1107132、10 1114972、10³ 1513360、10⁴ 5139628、10⁵ 41402308、10⁷ 4030722004、10⁸ 40297407128 | 各少 128 |
| s | s = 1 161278059；s = 64 5259955944 | 161278027；5259953896 |
| 不删除的发布（a_T = 0） | 203729488（O = 40277664、Rl = 585220） | 203728680 |
| 开新片（u = 1000005） | 404074512 | 404074384 |
| G-small | 14398（不变：R_L = 0，谱系树零根，形状 0） | 14398 |
| φ = ½ / 集中档 / k = 1001（a_T = 815000） | 404076644 / 404073852 / 404071172 | 第二份没钉准入前（这一份新钉） |
| 等高点（集中档）u = 5216400 / u = 13040400 | 甲-B2后准入前 2103124788 / 5255907796 | 第二份只钉乙2、丙、残余式（照留） |
| 阳性对照（其余 G0） | 甲准入前 u = 10⁶ ÷ u = 10⁵ = 404072244 ÷ 41402308 | 404072116 ÷ 41402180 |
| 丁 256 次（G0 起） | 甲准入前合计 103456147732 | 103456114964 |
| 新补两处逐项（`8640`, `8965`） | G0 (0, 128)；a_T = 0 (680, 128)；s = 64 (0, 2048)；a_L = 1 (0, 0)；G-small (0, 0)；u = 10⁸、a_T = 0 (37040, 128) | 第二份按零份算 |
| 新补两项逐项（`14640`, `14643`） | G0 (0, 0)；D = 32768（其余 G0）(1074528256, 2883584)，合 1077411840；\|V\| 在 D = 32768 那一点多 2883584 | 第二份没算 |

互比断言（「乙2 两等高点相等」「新装四臂相同」「残余式三臂相同」「甲-今天在两份式子下相同」）旁边各有上表或第二份第七节一条绝对值。10⁶ 次 unlink 累计的精确和设计员没算，由第一段算出后与第二段计数分配器（累计那一项不进第二段）互核，不当锚点。

`model_r3.py`（`~/.local/share/singlefs/work/e199-designer-revision-3/calc/model_r3.py`，sha256 `7361c60e7ffc829fc38d8aefaad857d601ce7a49599aad5c34647f4d1a4a5c96`，146 行，python 草稿，不与装置共用代码）：

```python
"""E199 第三份登记：甲-B2后准入前照抄（O、Rl、Once）按第二份登记第五节的式子重建，另算第三份补进的两项。
只做算术，不读仓、不起程序。用法：python3 -I model_r3.py（打印锚点）。"""
from math import ceil

F_FANOUT, CAP = 135, {"T": 815, "L": 1359, "X": 959}
W = {"T": 40, "L": 24, "X": 40}
INODE_LEAF, F_UNITS, P_IT = 233, 16, 1
ML = {"A": (477, 150, 102), "D": (376, 125, 147), "M": (294, 143, 115)}   # 叶容量、内部容量、W_ck
E_A = 16
AR_LEAF, AR_FAN = 812, 169


def shape(n, h):
    return 8 * n + 24 * h


def levels_of(c, fan):
    """容器树（或按扇出建的树）每层节点数，从下往上；c = 0 时没有层。"""
    if c == 0:
        return []
    out = [ceil(c / fan)]
    while out[-1] > 1:
        out.append(ceil(out[-1] / fan))
    return out


class Tree:
    def __init__(self, records, added, cap, always_append=False):
        self.R, self.a, self.cap = records, added, cap
        self.C = ceil(records / cap) if records else 0
        self.lv = levels_of(self.C, F_FANOUT)
        self.N, self.h = sum(self.lv), len(self.lv)
        self.fill = records - (self.C - 1) * cap if self.C else 0
        self.room = cap - self.fill if self.C else 0
        self.k_old = 1 if (added > 0 and self.C > 0 and self.room > 0) else 0
        self.k_new = ceil((added - self.room) / cap) if added > self.room and added > 0 else 0
        t0 = self.C - 1 if self.k_old else self.C
        self.rho = sum(max(0, n - t0 // F_FANOUT ** (l + 1)) for l, n in enumerate(self.lv)) if added > 0 else 0
        self.appends = always_append or added > 0
        self.R2 = records + added
        self.C2 = ceil(self.R2 / cap) if self.R2 else 0
        self.lv2 = levels_of(self.C2, F_FANOUT)
        self.N2, self.h2 = sum(self.lv2), len(self.lv2)


def ar_levels(units_c, trees, LI, D):
    sl = 2 * (sum(t.C for t in trees) + LI + D + P_IT) + (sum(t.N for t in trees) + F_UNITS)
    n1 = 2 * ceil(sl / AR_LEAF)
    out = [n1]
    while out[-1] > 1:
        out.append(ceil(out[-1] / AR_FAN))
    return sl, out


def ml_levels(entries, leaf, internal):
    if entries == 0:
        return []
    out = [ceil(entries / leaf)]
    while out[-1] > 1:
        out.append(ceil(out[-1] / internal))
    return out


def rewritten_nodes(lv, touched):
    x, total = touched, 0
    for n in lv:
        x = min(n, x)
        total += x
    return total


def geometry(u=10**6, aT=1, RX=10**3, aX=0, RL=16, aL=0, I=10**3, D=0, s=4, phi=1.0, focused=False):
    T, X, L = Tree(u, aT, CAP["T"]), Tree(RX, aX, CAP["X"], always_append=True), Tree(RL, aL, CAP["L"])
    trees = [T, X, L]
    LI = max(1, ceil(I / INODE_LEAF))
    Uc = sum(t.C + t.N for t in trees) + LI + D + F_UNITS
    rc = sum(t.k_old + t.rho for t in trees) + 1 + F_UNITS
    Uc2 = sum(t.C2 + t.N2 for t in trees) + LI + D + F_UNITS
    nc = Uc2 - (Uc - rc)
    sl, ar = ar_levels(Uc, trees, LI, D)

    class After:
        pass
    after = After()
    after.C, after.N = None, None
    trees2 = [type("t", (), {"C": t.C2, "N": t.N2}) for t in trees]
    sl2, ar2 = ar_levels(Uc2, trees2, LI, D)
    N_AR, N_AR2 = sum(ar), sum(ar2)
    d_AR = 2 * (rc + nc)
    tau_AR = min(ar2[0], 2) if focused else min(ar2[0], d_AR)
    rho_AR = rewritten_nodes(ar2, tau_AR)
    leaf, internal, wck_M = ML["M"]
    c, ci = int(leaf * phi), int(internal * phi)
    M = Uc - 1 + N_AR
    M2 = Uc2 - 1 + N_AR2
    r_M = rc - 1 + rho_AR
    n_M = M2 - (M - r_M)
    d_M = r_M + n_M
    mlv, mlv2 = ml_levels(M, c, ci), ml_levels(M2, c, ci)
    N_M = sum(mlv)
    tau_M = min(mlv2[0], ceil(d_M / c)) if focused else min(mlv2[0], d_M)
    rho_M = rewritten_nodes(mlv2, tau_M)
    U = Uc + N_AR + N_M + P_IT
    A = 2 * U
    g = dict(T=T, X=X, L=L, LI=LI, I=I, D=D, s=s, Uc=Uc, rc=rc, nc=nc, sl=sl, ar=ar, N_AR=N_AR, M=M, r_M=r_M,
             d_M=d_M, mlv=mlv, N_M=N_M, rho_M=rho_M, rho_AR=rho_AR, U=U, A=A, wck_M=wck_M)
    return g


def jia_pre(g, edition="r3"):
    """甲-B2后准入前：(O, Rl, Once, 合计)。edition = "r2" 是第二份登记的式子，"r3" 加第三份补进的两项。"""
    T, X, L = g["T"], g["X"], g["L"]
    O = sum(t.R * W[k] + 56 * t.C for k, t in (("T", T), ("X", X)))
    O += 2 * (64 * g["I"] + 56 * g["LI"])
    O += sum(t.R * W[k] + 56 * t.C + shape(t.N, t.h) for k, t in (("T", T), ("X", X)) if t.appends)
    if edition == "r3":
        O += sum(shape(t.N, t.h) for t in (T, X) if not t.appends)          # transaction.rs:8640（墓碑树不追加那一支）
    Rl = L.R * W["L"] + 56 * L.C
    if L.appends:
        Rl += L.R * W["L"] + 56 * L.C + shape(L.N, L.h)
    elif edition == "r3":
        Rl += shape(L.N, L.h)                                               # transaction.rs:8965（谱系树不追加那一支）
    Rl += (g["M"] - g["r_M"]) * 67
    entries = {"A": E_A, "D": 0, "M": g["M"]}
    nodes = {"A": 1, "D": 0, "M": g["N_M"]}
    wck = {"A": ML["A"][2], "D": ML["D"][2], "M": ML["M"][2]}
    Rl += sum(entries[t] * wck[t] for t in "ADM")
    Rl += sum((entries[t] + nodes[t] - 1) * wck[t] for t in "ADM" if nodes[t] >= 1)
    Rl += 2 * g["A"] * 40
    Once = 2 * g["N_AR"] * 16
    s = g["s"]
    return O, Rl, Once, (s + 1) * O + s * Rl + Once


def jia_today_pre(g):
    """甲-今天准入前：照第二份登记的式子（甲-B2后 r2 加 (s+1)·Σ_{T,X}(2 + k_new)·shape + s·(2 + k_new_L)·shape_L）。代码已改，绝对值不随第三份变。"""
    T, X, L = g["T"], g["X"], g["L"]
    s = g["s"]
    extra = (s + 1) * sum((2 + t.k_new) * shape(t.N, t.h) for t in (T, X)) + s * (2 + L.k_new) * shape(L.N, L.h)
    return jia_pre(g, "r2")[3] + extra


def items_new(g):
    """第三份补进的两项逐项（每次发布准入前）：8640、8965。"""
    T, L, s = g["T"], g["L"], g["s"]
    return ((s + 1) * shape(T.N, T.h) if not T.appends else 0, s * shape(L.N, L.h) if not L.appends else 0)
```

`anchors_r3.py`（sha256 `a50bbd624dd286d4972b0557df976617e9073c70f4077b6d3caba2d75ad073f4`，38 行；在 `calc/` 里 `python3 -E -s anchors_r3.py > anchors_r3.out` 起，`-E -s` 而不是 `-I`：它要从自己的目录导入 `model_r3`）：

```python
"""第二份登记第七节里甲-B2后准入前那几格的锚点：按 r2 式子复现，再按 r3 式子算新值。"""
from model_r3 import geometry, jia_pre, jia_today_pre, items_new

R2_ANCHORS = [
    ("G0", {}, 404072116),
    ("u=0", {"u": 0}, 1107004), ("u=10", {"u": 10}, 1114844), ("u=1e3", {"u": 10**3}, 1513232),
    ("u=1e4", {"u": 10**4}, 5139500), ("u=1e5", {"u": 10**5}, 41402180),
    ("u=1e7", {"u": 10**7}, 4030721876), ("u=1e8", {"u": 10**8}, 40297407000),
    ("s=1", {"s": 1}, 161278027), ("s=64", {"s": 64}, 5259953896),
    ("aT=0", {"aT": 0}, 203728680), ("u=1000005", {"u": 1000005}, 404074384),
    ("G-small", {"u": 10, "RX": 0, "RL": 0, "I": 1, "s": 1}, 14398),
]

for name, kw, r2 in R2_ANCHORS:
    g = geometry(**kw)
    O2, Rl2, Once, old = jia_pre(g, "r2")
    O3, Rl3, _, new = jia_pre(g, "r3")
    i8640, i8965 = items_new(g)
    print(f"{name}\tr2式={old}\t登记r2={r2}\t{'相同' if old == r2 else '不同'}\tr3式={new}\t差={new - old}\t8640={i8640}\t8965={i8965}\tO={O3}\tRl={Rl3}\tOnce={Once}")

g = geometry()
print(f"G0 甲-今天准入前={jia_today_pre(g)}（第二份登记 404074052）\t甲-今天−甲-B2后(r2)={jia_today_pre(g) - jia_pre(g, 'r2')[3]}\t甲-今天−甲-B2后(r3)={jia_today_pre(g) - jia_pre(g, 'r3')[3]}")
for kw in ({"aT": 0}, {"s": 64}, {"u": 10**8, "aT": 0}, {"RL": 2000}, {"aL": 1}):
    g = geometry(**kw)
    print(f"逐项 {kw}: 8640={items_new(g)[0]} 8965={items_new(g)[1]} 甲-B2后准入前 r3={jia_pre(g, 'r3')[3]} r2={jia_pre(g, 'r2')[3]}")

# 丁 256 次（G0 起连发 256 次意图，a_T = 1、u 逐次加一）：甲准入前合计
tot2 = tot3 = 0
for k in range(256):
    g = geometry(u=10**6 + k)
    tot2 += jia_pre(g, "r2")[3]
    tot3 += jia_pre(g, "r3")[3]
print(f"丁256 甲准入前合计 r2式={tot2}（登记 103456114964，{'相同' if tot2 == 103456114964 else '不同'}） r3式={tot3} 差={tot3 - tot2}")
g5, g6 = geometry(u=10**5), geometry()
print(f"阳性对照 u=1e6÷u=1e5 r2={jia_pre(g6,'r2')[3]}/{jia_pre(g5,'r2')[3]}={jia_pre(g6,'r2')[3]/jia_pre(g5,'r2')[3]:.4f} r3={jia_pre(g6,'r3')[3]}/{jia_pre(g5,'r3')[3]}={jia_pre(g6,'r3')[3]/jia_pre(g5,'r3')[3]:.4f}")
for kw in ({"phi": 0.5}, {"focused": True}, {"u": 5216400, "focused": True}, {"u": 13040400, "focused": True}, {"aT": 815000}):
    g = geometry(**kw)
    print(f"其余 {kw}: r2={jia_pre(g, 'r2')[3]} r3={jia_pre(g, 'r3')[3]} 差={jia_pre(g, 'r3')[3]-jia_pre(g, 'r2')[3]}")
```

`anchors_r3.out` 原样（sha256 `c80cbbf3d5739b82190a08f30904bc1924f162e7631a5c640dca599fa09c5cce`，26 行；「登记r2」列是第二份第七节那一格，「相同」是复现得了）：

```text
G0	r2式=404072116	登记r2=404072116	相同	r3式=404072244	差=128	8640=0	8965=128	O=80346376	Rl=585019	Once=288
u=0	r2式=1107004	登记r2=1107004	相同	r3式=1107132	差=128	8640=0	8965=128	O=208816	Rl=15739	Once=96
u=10	r2式=1114844	登记r2=1114844	相同	r3式=1114972	差=128	8640=0	8965=128	O=209760	Rl=16519	Once=96
u=1e3	r2式=1513232	登记r2=1513232	相同	r3式=1513360	差=128	8640=0	8965=128	O=289072	Rl=16976	Once=96
u=1e4	r2式=5139500	登记r2=5139500	相同	r3式=5139628	差=128	8640=0	8965=128	O=1010304	Rl=22003	Once=96
u=1e5	r2式=41402180	登记r2=41402180	相同	r3式=41402308	差=128	8640=0	8965=128	O=8222624	Rl=72273	Once=96
u=1e7	r2式=4030721876	登记r2=4030721876	相同	r3式=4030722004	差=128	8640=0	8965=128	O=801583840	Rl=5700197	Once=2016
u=1e8	r2式=40297407000	登记r2=40297407000	相同	r3式=40297407128	差=128	8640=0	8965=128	O=8013958624	Rl=56898598	Once=19616
s=1	r2式=161278027	登记r2=161278027	相同	r3式=161278059	差=32	8640=0	8965=32	O=80346376	Rl=585019	Once=288
s=64	r2式=5259953896	登记r2=5259953896	相同	r3式=5259955944	差=2048	8640=0	8965=2048	O=80346376	Rl=585019	Once=288
aT=0	r2式=203728680	登记r2=203728680	相同	r3式=203729488	差=808	8640=680	8965=128	O=40277664	Rl=585220	Once=288
u=1000005	r2式=404074384	登记r2=404074384	相同	r3式=404074512	差=128	8640=0	8965=128	O=80346776	Rl=585086	Once=288
G-small	r2式=14398	登记r2=14398	相同	r3式=14398	差=0	8640=0	8965=0	O=1184	Rl=11934	Once=96
G0 甲-今天准入前=404074052（第二份登记 404074052）	甲-今天−甲-B2后(r2)=1936	甲-今天−甲-B2后(r3)=1808
逐项 {'aT': 0}: 8640=680 8965=128 甲-B2后准入前 r3=203729488 r2=203728680
逐项 {'s': 64}: 8640=0 8965=2048 甲-B2后准入前 r3=5259955944 r2=5259953896
逐项 {'u': 100000000, 'aT': 0}: 8640=37040 8965=128 甲-B2后准入前 r3=20263056488 r2=20263019320
逐项 {'RL': 2000}: 8640=0 8965=128 甲-B2后准入前 r3=404264760 r2=404264632
逐项 {'aL': 1}: 8640=0 8965=0 甲-B2后准入前 r3=404073468 r2=404073468
丁256 甲准入前合计 r2式=103456114964（登记 103456114964，相同） r3式=103456147732 差=32768
阳性对照 u=1e6÷u=1e5 r2=404072116/41402180=9.7597 r3=404072244/41402308=9.7597
其余 {'phi': 0.5}: r2=404076516 r3=404076644 差=128
其余 {'focused': True}: r2=404073724 r3=404073852 差=128
其余 {'u': 5216400, 'focused': True}: r2=2103124660 r3=2103124788 差=128
其余 {'u': 13040400, 'focused': True}: r2=5255907668 r3=5255907796 差=128
其余 {'aT': 815000}: r2=404071044 r3=404071172 差=128
```

派生量对第二份第七节的复核（`focused-check.out` 原样，命令在第十三节；G0、集中档、φ = ½、两处等高点、u = 10⁸ 的 M、映射树层、分配记录树层、Sl、d_M、ρ_M、ρ_AR、r 与第二份第七节逐个相同）：

```text
{} M 1272 mlv [5, 1] ar [8, 1] sl 2501 d_M 56 rho_M 6 rho_AR 9 r 35 C_T 1227 N_AR 9
{'focused': True} M 1272 mlv [5, 1] ar [8, 1] sl 2501 d_M 44 rho_M 2 rho_AR 3 r 25 C_T 1227 N_AR 9
{'phi': 0.5} M 1272 mlv [9, 1] ar [8, 1] sl 2501 d_M 56 rho_M 10 rho_AR 9 r 39 C_T 1227 N_AR 9
{'u': 5216400, 'focused': True} M 6508 mlv [23, 1] ar [32, 1] sl 12887 d_M 44 rho_M 2 rho_AR 3 r 25 C_T 6401 N_AR 33
{'u': 13040400, 'focused': True} M 16227 mlv [56, 1] ar [80, 1] sl 32158 d_M 44 rho_M 2 rho_AR 3 r 25 C_T 16001 N_AR 81
{'u': 100000000} M 124255 mlv [423, 3, 1] ar [608, 4, 1] sl 246353 d_M 218 rho_M 222 rho_AR 89 r 332 C_T 122700 N_AR 613
```

## 八、轨迹与几何敏感性

### 取样点

第二份第八节「取样点」逐条照留：G0（u = 10⁶、a_T = 1、R_X = 10³、a_X = 0、R_L = 16、a_L = 0、I = 10³、D = 0、s = 4、φ = 1、分散档；乙2 b = 32、g = 64；丁 p = 64、整份新基）、逐一扫（u、a_T、s、R_X、a_X、R_L、a_L、I、D、乙2 的 b × g、丁的 p × 压实方式、φ、碰叶档；第二段只取 u ≤ 10⁶ 与 D = 0，另加两处等高点）、事件点、等高点（u = 5216400 与 u = 13040400，集中档）、G-small（u = 10、R_X = 0、R_L = 0、I = 1、s = 1，其余 G0）。另加事件点（第 3a 步 ③，触发是前提、不是被问的量）：

- **墓碑树不追加那一支的形状照抄（`8640`）**：触发条件 a_T = 0（删除步不是意图）。逐一扫的 a_T = 0 那一点会发生（(s + 1)·shape(11, 2) = 5 × 136 = 680）；G0 不发生。
- **谱系树不追加那一支的形状照抄（`8965`）**：触发条件 a_L = 0。G0 会发生（4 × 32 = 128）；逐一扫的 a_L > 0 那一点不发生（0）；G-small 上 R_L = 0、形状为空，发生了也是 0。
- **第一个文件照抄的数据单元与指针（`14640`、`14643`）**：触发条件 D > 0 且这次不写第一个文件。逐一扫的 D = 32768 那一点会发生（1077411840，第七节）；G0 不发生（D = 0）。

### 几何敏感性

第二份第八节那一段逐字照留（判定那几格在 G-small、s = 1、a_T = 815000、I = 1、φ = ½、集中档上各重跑一格；翻面记「不稳定」、写明哪一点翻；判别力自证把门槛挪到 G0 与 G-small 两个值之间，那一格的检查必须由一侧转到另一侧）。这一份补的几项在 G0 上合 128 字节，第四节现算它们不改 G0 与 G-small 上任何一格落在门槛的哪一侧；判定那几格另在 a_T = 0 与 D = 32768 两点上各报一次比值（报数，不判、不进「不稳定」的认定），让新补几项不为 0 的点上的比值也看得见。

### 轨迹（被谓词消费的量）

第二份第八节那一小节逐字照留（丁的层数 L 报峰值、为正次数、期末值；s 当旋钮给、不报轨迹；各臂常驻峰值在 256 次连发里报峰值与期末值；映射树与分配记录树的被碰叶数 τ 由碰叶档给定、不报轨迹）。新补几项都不是任何谓词的输入（不进停机、起跑、准入那几判），不报轨迹。

## 九、变异

变异表 `scripts/mutation/tables/e199_transaction_output_copy.tsv`（第一段）与 `scripts/mutation/tables/e199_transaction_output_copy_executed.tsv`（第二段），由执行员按源码写原文（每条原文在源码里恰好命中一次）。第二份第九节的 24 + 18 条照留（取样点照它写的）；另加下表。每条写它在哪个取样点上改变输出、在哪个点上不变（不变的点不能当它被抓的依据）。

| 变异 | 改变输出的取样点 | 不变的点 |
|---|---|---|
| O 漏掉墓碑树不追加那一支的形状（`8640` 那一项） | a_T = 0（−680） | G0（a_T = 1，那一项本来是 0） |
| `8640` 那一项乘 s 不乘 s + 1 | a_T = 0（−136） | G0 |
| Rl 漏掉谱系树不追加那一支的形状（`8965` 那一项） | G0（−128） | a_L = 1（那一项本来是 0）、G-small（R_L = 0） |
| `8965` 那一项乘 s + 1 不乘 s | G0（+32） | G-small（形状 0） |
| 不追加那一支的形状条件接反（[不追加] 写成 [追加]） | G0（谱系 −128；墓碑 +680） | G-small |
| 准入后漏掉第一个文件照抄的数据单元那一份（`14640`，32792·D） | D = 32768（−1074528256） | G0（D = 0） |
| 准入后漏掉数据指针（`14643`，88·D）或 \|V\| 漏掉 88·D | D = 32768（−2883584） | G0 |
| （第二段）不追加那一支借形状不抄（甲-B2后那一份真抄被省掉） | G0 甲准入前；作废条款第 1 条的互核转红 | G-small |

变异只证明写了断言的那几处会红，实验页写「<条数> 条变异全部被抓」，不写「零盲区」。

## 十、失败条款

第二份第十节那十条照留（条款与实现不一致；调查报告的代价阶不成立——第三段不做，这一条不会触发，照留只为记账；乙2 做不到「只随 k 与树高涨」；结果与第四节预期相反；字节分不出挂钟；读路径或峰值反过来；丁在每个 p 上都不省两倍；现算重建反过来；两种形式分到两侧；碰叶档决定结局），每条的「什么观测会让它触发」与「触发之后」一个字不改；第一条的核对命令换成下面这一行：

| 条款 | 什么观测会让它触发 | 触发之后 |
|---|---|---|
| **条款与实现不一致**（第七节「出自被测条款本身」与「取实现值的模型常量」两组） | 执行员开工时现跑 `grep -n 'pub const \(NODE_BYTES\|DATA_UNIT_BYTES\|INODE_LEAF_RECORDS\|TOMBSTONE_CONTAINER_RECORDS\|LINEAGE_CONTAINER_RECORDS\|RETIREMENT_CONTAINER_RECORDS\|ALLOCATION_RECORD_TREE_LEAF_SLOTS\|ALLOCATION_RECORDS_TREE_INDEX_NODE_HEADER_BYTES\|ACCOUNTING_KEY_BYTES\)' crates/singlefs-format/src/lib.rs`、`grep -n 'entries == 135' crates/singlefs-core/src/container_tree.rs`（这一棵树在 `173`）、`grep -n '记账树叶 477' crates/singlefs-core/src/transaction.rs`（`2679`）、`grep -n 'CENTRAL_MAPPING: Self\|DEADLIST: Self\|ACCOUNTING: Self' crates/singlefs-core/src/code_two_tree.rs`、`grep -n 'const ALL: \[Self; ' crates/singlefs-core/src/transaction.rs`（`7915`、`7955`）、`grep -n '指向码 1 数据单元的指针 88' crates/singlefs-core/src/pointer.rs`（`828`）；任一个与第五节常量不同 | 模型照第五节常量算，不改；报告并列两边，写「条款或实现有一边偏了」（取实现值那一组写「实现改了」），交主 agent |

## 十一、作废条款与停机条款

### 作废条款

第二份第十一节作废条款 1–8 逐字照留（第 1 条两段对不上差超过 max(5%, 4096 字节)；第 2 条四臂结局不同；第 3–7 条照第一份；第 8 条现算重建的阳性对照没过）。第 1 条的「照抄」那一笔在甲-B2后上含新补四项：第二段计数分配器量出的甲准入前「照抄」要包住 `8640`、`8965` 那两份形状，D = 32768 那一点不进第二段（读法第 6 条），`14640`、`14643` 只在第一段报。

### 停机条款（装置与 `crates/` 对不上：两边都查，既不作废也不当结果，`.claude/rules/implementation-first.md` 第 4 条）

1. **照抄点或现算重建点挪了**：执行员写装置之前，在开工那一刻的主工作区：
   - ① 对第三节「甲臂的照抄点」「现算重建」两张表与「去掉 / 留着」两列的每一行，用 `grep -n` 或 `awk 'NR==行号'` 现核那一处的代码还在；先记下开工那一刻的 `git rev-parse HEAD` 与三个目录下每份 `.rs` 的 sha256，与第十三节「这一棵树」逐份 `diff`，命令与输出原样写进报告。
   - ② 把第三节三段 `callgraph.py` 代码块按次序拼成一份文件（拼法：`awk '/^`callgraph.py` 第 /{m=1; next} m==1 && /^```python$/{inb=1; next} inb && /^```$/{inb=0; m=0; next} inb{print}' research/prompts/e199-preregistration-r3.md > <草稿目录>/callgraph.py`），sha256 必须是 `8e966328c1fcbd4ac085c18995faf68b9dccc7bbba977759d64bb4cfb2ee60b6`，不是就停；同样把第三节两段归属表拼成 `classification.tsv`（拼法同形，起头的标记换成 `^\*\*归属表\*\*|^归属表准入后三份输出`、围栏换成 `text`），sha256 必须是 `3340e1d70a0636d62a042fb5335512edd4659be5435f90071a50019afccf0b24`；照第三节「命令」原样重跑那八条 `list` / `grep`（这是草稿里自己写的只读脚本，不起项目代码；交不交服务照执行员定义与共用约束办），每份输出与归属表还原出的那一份（`awk -F'\t' -v n=<输出> '$1==n{print $2"\t"$3"\t"$6}' classification.tsv`）`diff`。
   - ③ `diff` 里新出现的每一条命中（「>」行），在归属表里找不到同一个「文件、所在函数、原样那一行」的，逐条归一类写进报告；归属落在「照抄·进模型」「现算重建·进模型」或带着它们的那两类、而第三节两张表里没有的，或两张表里有而代码没了的，停，报两边。只是行号挪了（同一文件、同一函数、同一行原文）的，照新行号写进第十二节修订，不停。
2. **B 簇第二件与另一路代码轮的落地状态**：B 簇第二件已在主工作区（这一棵树）。执行员开工那一刻在主工作区核：「去掉」那一列真没了——`grep -n 'version.tombstone_tree_index_nodes.shape.clone()\|version.retirement_tree_index_nodes.shape.clone()\|version.lineage_tree_index_nodes.shape.clone()' crates/singlefs-core/src/transaction.rs` 与 `grep -n 'let shape_before_this_publish = shape.clone();\|shape_before_this_publish: shape_before_this_publish.clone(),' crates/singlefs-core/src/container_tree.rs` 与 `grep -n 'copy_of_one_level(&self.children_of_each_node_at_each_level' crates/singlefs-core/src/container_tree.rs` 三条都零命中（这一棵树上设计员现跑零命中；对基线那两份文件跑，第一条命中 `8437`、`8480`、`8764`，第二条 `2315`、`3188`，第三条 `2161`，分得出）；「留着」那一列每一处都在（含 `8640`、`8965`、`12510-12512`、`14640`、`14643` 与 (a)–(d)）。命令与原样输出写进报告；对不上的（例如另一路代码轮把 `8640`、`8965` 改成借引用或按值移走），停、报两边，不自己改登记。
3. **内存宽度没与实现对**：第二份这一条照留（第五节的内存宽度是装置自己的类型，没与 `crates/` 的 Rust 布局对过；不把第五节的常量改成实现的真宽当结果）。

### 够判停机

每段交回附一张表，逐行写岔路单四行「已够判 / 还差什么 / 剩下的量能不能让它翻面」。四行都够判就停，没跑的量逐条标「够判后未跑」。今天就知道不跑的：每轮整份拷分配器（`transaction.rs:13098`）与外层那一份（`12314`）、列全部容器与节点、建 `units_by_role` / `mapped_units_by_role` / `files_other_than_the_first_by_inode` 那几遍（第三节「不进模型的」），记「射程外，不量」；extent 树节点（`13476`、`13686`、`13712`、`16065`）、最后一批的文件号集合（`unlink.rs:313`）、别的文件的数据指针（`15124`），记「模型里为 0，不量」；整理的一批整份克隆上一版（`14795`），记「G0 不走到，不量」；第三段，记「主 agent 判定不做」。

## 十二、修订

**设计员修订（这一份落盘时，装置写之前；派发提示第 3 条）**：第三节各表、第一节读法第 1 条、第十节第一行、第十一节「够判停机」的行号，从第二份那一棵树（`transaction.rs` `06c58028…`、`container_tree.rs` `e94e23ed…`、`mounted_session.rs` `2ea0d883…`）改成这一棵树（`1178862c…`、`392e25fc…`、`2b8d27e0…`），代码没变、只是行号挪了；逐行对照照第二份执行员报告「行号挪动」一节与它「停机条款第 2 条」那张表的「今天」列，每一个新行号都在这一棵树里用 `awk 'NR==行号'` 现查过（第十三节）。执行员报告只给了单行、这一份按现查补成区间或改写法的四处：甲1 三段写成 `8601-8609`、`8646-8654`、`8934-8942`（报告给 `8606`、`8651`、`8939`）；`mapping_keys` 那一段写成 `15638-15641`（报告给 `15640`）；丙5 装 `units` 那一段第二份写 `15930-16000`（报告映到 `16120`），这一份按逐个调 `carried_unit` 的角色列 `16009`、`16074`、`16090`、`16121`、`16136`，起行 `16004`；分配记录树照抄节点那一段写成 `16097-16111`（报告给 `16107`）。另：`mount.rs` 的两个回退入口 `10016` → `10284`、`10324` → `10592`，`&TransactionOutput` 459 → 469 处，`pointer.rs:828` 是新引的。

（装置写之后、产物之前由执行员接着写，只许收严或补臂。）

## 十三、读过的文件与跑过的命令

### 这一棵树

`git rev-parse HEAD` = `59ce5a86550de2c2c9bc6b51b34b4f881313682c`。`git diff --name-only HEAD -- crates/singlefs-core/src crates/singlefs-format/src crates/singlefs-replay-prefix/src` 列出的 40 份（暂存的新文件 `verified_model.rs` 在内）与它们此刻的 sha256（`sha256sum $(cat changed-files.txt)` 原样，草稿 `snapshot/changed-sha256.txt`）：

```text
900a2ab30fd411a1ca3daa742951626a8ec32c2d75dfc11d3727e0d50912d31d  crates/singlefs-core/src/address.rs
ea6bc6cdf9b2abd781911efbbb650f01f244eb789e26e773d0fb4b4c4756d4b3  crates/singlefs-core/src/admission.rs
03b3e57fad9412490e6ace5a99e6f8887f7dda3c6cdb6e4c169461966c6b0d01  crates/singlefs-core/src/allocator.rs
7015f662496aa2d8d611df1e1bd43d493374612205a3de1ede0252647979bfa1  crates/singlefs-core/src/background_compute.rs
64739500e712da5832d495a4b31cba3e7b087abcfe70c2f5f75c1230770d6267  crates/singlefs-core/src/block_device.rs
a865fa355ceff69d6bba3983516383c29064e7ad3f47ded35b07ba84b8f1c7e2  crates/singlefs-core/src/bytes.rs
6c5e0633702d088f1f3f17ae051d8aa7d54972633cf728db4f1b57e2aff98916  crates/singlefs-core/src/checksum.rs
97adbba4e382b37ffc6af5be5badbc6fc6d76090d66028281c4cd807972bce4d  crates/singlefs-core/src/compaction_plan.rs
392e25fc97b8a1b43d41f3f461b29aee43dbadc3eca29b5859580d075062daaa  crates/singlefs-core/src/container_tree.rs
94d185dccd0cf51fbf78826c7bceb23c390dcd7131770dbac8a88334328c9b52  crates/singlefs-core/src/deleted_inodes.rs
69fee61e831248e6b1b88c70bc84b5b3cc8dd16177ae72cb8fcfc3ac6f113069  crates/singlefs-core/src/inode_tree.rs
431b59ba15e95069411f913aac56b7d768245e294f3f41f9fb5a33a1d898a5a6  crates/singlefs-core/src/instance_table.rs
aecc4c1a9adc0b5938fda62e32d0cb3d66411fc27bf88730a6289686dd3a5eb0  crates/singlefs-core/src/instance_table_sweep.rs
0571b33b31ca6e90d0c726c956b36624dace293e0d9bb1df7d7f879612066b23  crates/singlefs-core/src/journal.rs
743b674560f8766355af19ffc8db9b62178a95dfaa6bec4bd8e2e5e53d744a61  crates/singlefs-core/src/lib.rs
e8039626e7b5864f69d75f7df3c08745c76551b41a1d5d89025c4acb4490bb62  crates/singlefs-core/src/lineage_tree.rs
5c8949b1ac4885b5356a1f4e5673f3f2fe673740aa13daa24a50c54f0a7b72bb  crates/singlefs-core/src/logged_operations.rs
b45515cb39b46ee57836ead7fa606e322807aa681bf229c5cfcc1bcabdfe6348  crates/singlefs-core/src/make_filesystem.rs
502121e27d325c74b257da04707c7832bd12eb9768093e2ab7f9e63405306a33  crates/singlefs-core/src/mount.rs
6de51ef734f8d2b3c0ef7350c214c568e77cee1a2793c08765f6c9243d31aa4e  crates/singlefs-core/src/mounted_read.rs
2b8d27e012b41f4859b0c68e4d1bd82af94d51f2de60f2c57d8f275a99a670a5  crates/singlefs-core/src/mounted_session.rs
c4c0de97c934f023ab074ef49eb113e46ec564e1ba2b2c8627620cd4a1737d41  crates/singlefs-core/src/pointer.rs
bc576521f0892190669743cf746cd9e2831bd2fc5e72ea275ddab5b36808132b  crates/singlefs-core/src/records.rs
e8fbdf8620ba8e856d352963c04b77c361fc96784b6cfd53b18e9b0cbd46ef21  crates/singlefs-core/src/recovery.rs
67ca9d7baaa8a937b3293e9d7ccf209c5d319ad45c825f030769470f9986852a  crates/singlefs-core/src/retirement_tree.rs
853b12051caa0978f95d22946eef711ac6adc36d088577cb7f01f0adc1b63899  crates/singlefs-core/src/root_record.rs
2c41a1cbc2abab666832477391843105849dfeb0d91176feab18b5b6fcea4336  crates/singlefs-core/src/system_configuration.rs
29403c6e884b94706a87d28ac359b3fe8e568863ac5b1c91b515a75740b382c5  crates/singlefs-core/src/tombstone_tree.rs
1178862ceaa82964e4a69c815434684ca365380663014658d957b760b8ea0973  crates/singlefs-core/src/transaction.rs
978fefaeb1c73482184b7fd8bd6da930a3a447e1dc3f14b435e14db04c9eed45  crates/singlefs-core/src/truncate.rs
029df55be8987c5c64c5b479a86ab192f0ae550180ee5995d6a8cf1e74a5906f  crates/singlefs-core/src/unit.rs
b9ac0c12b0abbdb2e0ec8eb1ef131a8d8e12da6127457ded619a635e887d31d8  crates/singlefs-core/src/unlink.rs
e812d29913b96b349530757fbb14b7a4e9db9a6408dc90a25c41e21c56c45092  crates/singlefs-core/src/verified_model.rs
d05b4cfc8f3262146bb1957de16fadf28e7581fe9f09af298f34ad8044e7ae7c  crates/singlefs-core/src/write_request_split.rs
517a63626edba012b8c48ab9af5327e5bd54b790b98050d63bec844a0e5e7ced  crates/singlefs-replay-prefix/src/README.md
81a505877bed4d185a4314de6b1a0ef96cae228de8f9edbf292db03953911184  crates/singlefs-replay-prefix/src/file_range_proofs.rs
cf38f4544a823424635e4496cc4f080a985f25461594fe7ae95506bb8e42409f  crates/singlefs-replay-prefix/src/gc_ring_interval_proofs.rs
66f08858c51a97a98b8ebbad2209140f7dcd9da0cf1f6b8ab793de242f10a2f4  crates/singlefs-replay-prefix/src/lib.rs
d5a52cf2fa68ab6a35faff3f291de6a0381e21b13d66acc3f5c36ea4dc75f348  crates/singlefs-replay-prefix/src/partial_write_proofs.rs
b3cb742fc044c1683fe0703b3b4a2d542f433e79430a3afd4db462935336c4d7  crates/singlefs-replay-prefix/src/unlink_proofs.rs
```

三个目录下全部 49 份 `.rs` 的 sha256 存在草稿 `snapshot/all-scanned-sha256.txt`（那份文件的 sha256 `af05a23c5ab4365c912a624f2bbd642267ce4d99f3ff1118d77ccbcbdf63d394`）；这一份落盘之前重跑同一条 `ls … | xargs sha256sum` 与它 `diff` 退 0（打「UNCHANGED_SINCE_SNAPSHOT」），全扫、锚点与行号现查都在这一棵树上做。

### 读过的文件（行区间；grep 命中行也列）

- `.claude/agent-common.md` 1–123；`.claude/agents/experiment-designer.md`（定义经派发进上下文，另 `cat` 前 20 行）。
- 规则整份：`.claude/singlefs-ai-sop/rules/test-discipline.md`、`.claude/rules/three-way-inference.md`、`.claude/singlefs-ai-sop/rules/evidence-discipline.md`、`.claude/rules/mutation-sampling.md`、`.claude/rules/implementation-first.md`、`.claude/rules/test-script-discipline.md`。
- `research/prompts/m4-transaction-output-copy-forks.md` 整份；`research/prompts/e199-preregistration-r2.md` 1–682 整份；`research/prompts/e199-preregistration.md` 标题行（`grep -n '^#'`）与 172–333（第三到第五节）；`.claude/heavy-thread-default` 整份。
- `~/.local/share/singlefs/work/e199-runner-segment-1-r2/result.md` 1–206 整份；`~/.local/share/singlefs/work/m4-close-code-r1-fix-b2-writer-2/result.md` 第 34、235 行（`awk`）；它的 `patch/crates.patch` 只算了 sha256。
- `~/.local/share/singlefs/work/m4-close-code-r1-fix-b2-writer/base/crates/singlefs-core/src/`：全部 `.rs` 的 sha256；`transaction.rs`、`container_tree.rs` 的 grep 命中 8437、8480、8764、2315、3188、1388、1396、2161；整目录由 `callgraph.py` 机器读（基线对照那一次）。
- 工具：`scripts/kb/quote-kb.py` 1–30、`scripts/kb/replace-once.py` 1–25；`.claude/hooks/heavy-test-guard.sh` 里 `python` 的 grep 命中（3、55、56、66、73、164、185、200–203、206、217、240、258、275、278、294、314、323、329、332 等行）；`.claude/hooks/lib_heavy_tests.py` 里「工作根」的 grep 命中（102、231、233、234、2157、2158、2175、2176、2178、4896–4912 等行）。
- `crates/singlefs-core/src/transaction.rs`（这一棵树）：读 8560–8700、8915–8980、9015–9060、9395–9412、13100–13125、14600–14660、14730–14810、14868–14890、15055–15135、15990–16062、16222–16240、16424–16434、755–770；`context.py` 读的 ±3–4 行：6582–6676、7644–7648、8746–8804、12910–12914、13033–13084、14157–14184、14320–14364、14683–14702、14998–15001、15068–15124、15167–15170、15410–15413、15612–15640、15876–15880、16012–16209、16518–16567、16587–16591、16684–16689、16818–16822、13472–13476、13682–13712；`awk 'NR==行号'` 现查第三节、第一节、第十节、第十一节、第十二节写到的每一个行号（3195、3211、3343、4208、7915、7955、7994、8016、8017、8239、8325、8331、8333、8367、8376、8511、8600–8616、8629–8658、8718、8729、8734、8928、8933–8947、8963–8966、8976、9004、9022、9026、9027、9048、9049、9055、9399、9409、11169、12009、12018、12294、12314、12315、12489、12508–12513、12621、12659、12676、12785、13017、13045、13051–13053、13063、13098、13107、13110、13120、13121、13136–13141、13272、13276、13277、13476、13776、13912、13947、14144、14409、14440、14473、14626、14640、14643、14645、14646、14656、14795、14880、15058、15059、15061、15156、15204、15205、15217、15221、15224、15225、15326、15338、15348、15416、15417、15420、15423、15424、15427、15433、15446、15447、15451、15613、15614、15616、15617、15623、15636、15637、15640–15642、16003、16004、16009、16065、16074、16090、16096、16097、16107、16111、16112、16116、16117、16120、16121、16136、16191、16203、16208、16210、16616、16623、16624、16635、16653、16671、16813、16817、16818、16822、16824、16880）；grep 命中 2967、7932、7998、14825、16445、17325（`tree`）、8239（`units()`）、3211、3738、8414、8519、8523、9079、13283、13286、14863、14882、14941–14950、15062–15144、15172、15421、15435、15436、16219、16230、16592–16602（`data_pointers`）、14626、14880（`carried_file_version_units`）、11169–12785（发布路径调用）。
- `crates/singlefs-core/src/container_tree.rs`：1478–1520、2600–2720、3540–3600；单行 173、1484、1490、1505、1528、1540、2037、2369、2627、2648、2683、2696、3537、3575、3576、3658、3718；grep 命中 447、1484、1489、1505、2037、2369、2608、2683、1520、1528。
- `crates/singlefs-core/src/`：`retirement_tree.rs` 194–215；`allocation_record_tree.rs` 305–385，单行 327、343、372、373、480，`context.py` 98–103、373–378、445–450、462–465；`pointer.rs` 826–845；`root_ring.rs` 183–230 的 grep；`mount.rs` 1249–1253、1503–1516、8708–8717，grep 命中 1503、6361、7468、8703、10284、10592、11322、12389、12845、12846；`recovery.rs` 904–908，grep 命中 133、236、313、380、448、906、930、936、6266、6290、6296；`code_two_tree.rs` 160–164、897、970、1111、1120–1128、1188–1193，单行 397、405、650、651；`inode_tree.rs` 单行 1093、1302、1692、1728、1970；`extent_tree.rs` 180–185、241–245、578–583；`mounted_read.rs` 972–977；`allocator.rs` 2613–2617、1368–1371、1640–1643；`truncate.rs` grep 243；`make_filesystem.rs` grep 309、374、605、831、881；`mounted_session.rs` 单行 108、1943、1947、2620。
- 全扫由 `callgraph.py` 机器读三个目录下 49 份 `.rs`（命中行全在第三节归属表里）。
- 没读：`.claude/kb/experiments/` 下任何实验的结果与结论小节、`research/results/`、E109（位置权威三臂的运行时代价）与 E110（条带表在连续发布下的期望写放大）的登记与产物。

### 跑过的命令（原样；`$D` = `~/.local/share/singlefs/work/e199-designer-revision-3`，`$B` = `~/.local/share/singlefs/work/m4-close-code-r1-fix-b2-writer/base`，在仓根起的不另注）

```
bash scripts/hosts/work-directory-ownership.sh claim $D --purpose 'E199 第三份跑前登记：甲-B2后补两处形状照抄、重做全扫与调用图'
git rev-parse HEAD > $D/snapshot/head.txt
git diff --name-only HEAD -- crates/singlefs-core/src crates/singlefs-format/src crates/singlefs-replay-prefix/src > $D/snapshot/changed-files.txt
(sha256sum $(cat $D/snapshot/changed-files.txt)) > $D/snapshot/changed-sha256.txt
ls crates/singlefs-core/src/*.rs crates/singlefs-format/src/*.rs crates/singlefs-replay-prefix/src/*.rs | xargs sha256sum > $D/snapshot/all-scanned-sha256.txt
ls crates/singlefs-core/src/*.rs crates/singlefs-format/src/*.rs crates/singlefs-replay-prefix/src/*.rs | xargs sha256sum | diff - $D/snapshot/all-scanned-sha256.txt && echo UNCHANGED_SINCE_SNAPSHOT   # 落盘之前，打 UNCHANGED_SINCE_SNAPSHOT
git show HEAD:crates/singlefs-core/src/transaction.rs | sha256sum      # 06c58028…（就是基线）
git show HEAD:crates/singlefs-core/src/container_tree.rs | sha256sum   # fcc1af29…（不是基线 e94e23ed…）
sha256sum $B/crates/singlefs-core/src/*.rs
sha256sum /home/fy5090/.local/share/singlefs/work/e199-runner-segment-1-r2/result.md research/prompts/e199-preregistration*.md
grep -rn '&TransactionOutput' crates/ --include=*.rs | wc -l                                  # 469
python3 -m py_compile $D/sweep/callgraph.py                                                   # 改过三次，每次写到同目录临时文件再 mv 换上
（第三节「命令」那十几条，在 $D/sweep/today/ 与 $D/sweep/baseline-core/ 里）
for f in <六份输出>; do awk -F'\t' -v n=$f '$1==n{print $2"\t"$3"\t"$6}' classification.tsv | cmp - $f && echo "$f 还原一致"; done   # 六份都打「还原一致」
awk '/^`callgraph.py` 第 /{m=1; next} m==1 && /^```python$/{inb=1; next} inb && /^```$/{inb=0; m=0; next} inb{print}' research/prompts/e199-preregistration-r3.md > $D/sweep/callgraph-from-registration.py; sha256sum $D/sweep/callgraph-from-registration.py   # 8e966328…，与草稿那一份相同
（归属表两段同形拼回，cmp 与 classification.tsv 相同，打 TABLE_RECONSTRUCTS）
python3 scripts/kb/quote-kb.py $D/quotes.md '.claude/rules/fs-design.md@## 记账是事务的副产品，不是事后的遍历' '.claude/kb/decisions/08-核心索引结构.md@#### 已定项 16：第一版 unlink：只立 deleted_inodes、从尾部分批、挂载里续做到空' '.claude/kb/decisions/08-核心索引结构.md@#### 已定项 2：节点大小 16 KiB，钉成常量' '.claude/kb/decisions/18-块里携带什么信息.md@#### 已定项 24：第一版墓碑：墓碑树、40 字节记录、谱系记录' > $D/quote-run.txt 2>&1   # rc=0，「✓ 4 段整抄进 …，回读逐字节一致」
awk 'NR>=85 && NR<=207' research/prompts/e199-preregistration-r2.md > $D/r2-section2-quotes.txt; diff $D/quotes.md $D/r2-section2-quotes.txt   # 只差 D18 那一段出处的行区间
cat $D/quotes.md >> research/prompts/e199-preregistration-r3.md; awk 'NR>=74 && NR<=196' research/prompts/e199-preregistration-r3.md | diff - $D/quotes.md   # 打 QUOTE_BLOCK_EXACT
python3 -E -s anchors_r3.py > anchors_r3.out          # 在 $D/calc/ 里
python3 -E -s -c "from model_r3 import geometry; …" > focused-check.out   # 在 $D/calc/ 里，打 G0、集中档、φ = ½、两处等高点、u = 10⁸ 的派生量
python3 -c "print('D', 32768*32792, 32768*88, 32768*(32792+88))" > d_items.out   # D 1074528256 2883584 1077411840
python3 scripts/kb/replace-once.py research/prompts/e199-preregistration-r3.md <旧串> <新串>   # 第一、三、四、九、十一、十二、十四节几处定点改（行号、说法、条数），逐次回读
bash $D/sweep/rerun-sweep.sh /home/fy5090/code/singlefs $D/sweep/rerun-selfcheck > $D/sweep/rerun-selfcheck.log 2>&1   # rc=0：从这一份登记拼出脚本与归属表（sha256 对上），八份输出与登记、两份函数清单与草稿逐份「相同」，status=0；rerun-sweep.sh sha256 45eca601bd79b389a219510f9efe140ec9ecbc729b44b5e24b71da1ba4d26dff
```

草稿文件的 sha256（`$D` 下）：

```text
8e966328c1fcbd4ac085c18995faf68b9dccc7bbba977759d64bb4cfb2ee60b6  sweep/callgraph.py
595746de44eb553b97fbdad0a87ccfbd93a8fd8f7afce4d00bd1f7658d1d9576  sweep/classify_r3.py
a0d3a35001bcc05adfe5ccb41920217eaa691bade0c0e4853781d468e271eeb5  sweep/context.py
d5d8d7fa8fd1b1020b763d26ca0ee3ebde7e788d0bc226255d375ddfce287850  sweep/today/pre-functions.txt
bdefa3016365ec31d9a7e3e6a45c251ef212b74f26d5960131302481dbea6e19  sweep/today/post-functions.txt
a55672633a5c38cd6d3d18c0798686cda0c5dd9ba6fedb00353c370bc8f72d17  sweep/today/pre-clone-to_vec.txt
36545672a6b4024d56b370d7f04211fad32ef2763e0f276095a9bce3264a1f7b  sweep/today/post-clone-to_vec.txt
1cb9498810a0a87106958a2e7cd8a56362c67152257f5568fac607b92e56ab4d  sweep/today/pre-cloned-copied-copy_of.txt
b6b54c264385865633b75f4071e4aff63a510238a4b8d9f6dbcf0028c82d249a  sweep/today/post-cloned-copied-copy_of.txt
8b68365c153eb99268651f616c701504c38a5c5257325674e6665ab52f0e0554  sweep/today/pre-push-deref-extend.txt
bc7cb5121a61fa1aeebf4b878fb24ea8751f27205d8a8a3f4087f9be5d4b18e2  sweep/today/post-push-deref-extend.txt
3340e1d70a0636d62a042fb5335512edd4659be5435f90071a50019afccf0b24  sweep/today/classification.tsv
359fa7f1b99d8f196fd8cd64607ba68455b992c1eab842a330c921541d204bf0  sweep/today/skipped.txt
98bdefa5ed622ab9091b9b8e65aaa172aeee9f6cf664629214852e04f5087a60  sweep/today/pre-traces.txt
72bce7ce89ff4b9a7065118835809701dd8c01209ad036a54a803bd1e1379606  sweep/today/post-traces.txt
7361c60e7ffc829fc38d8aefaad857d601ce7a49599aad5c34647f4d1a4a5c96  calc/model_r3.py
a50bbd624dd286d4972b0557df976617e9073c70f4077b6d3caba2d75ad073f4  calc/anchors_r3.py
c80cbbf3d5739b82190a08f30904bc1924f162e7631a5c640dca599fa09c5cce  calc/anchors_r3.out
fa7e2be8e3e4b283b81d62a141aace65d15bea80b57e39b89542f35f69a9dadb  calc/d_items.out
4014c802a194f096d02caa1e3d359bb3d4ca8bc85047c2a8c13ea8a6c6761905  quotes.md
```

## 十四、并行与挂钟

独立单位：第一段每个取样点乘每个臂变体互相独立（纯算术，不共享状态）；两段历史（丁的 256 次连发、10⁶ 次 unlink 累计）各是一串次序相关的发布，串内不切、串与串之间独立；第二段每个（几何, 臂, 碰叶档）一格是一个独立单位；停机条款第 1 条那八条全扫命令两两独立。
线程：取登记值 `.claude/heavy-thread-default` 的 31；第一段按取样点切片可开到 31 条线程（全部算完估不到 2 秒，合并按片的次序），第二段计时与计数都只开 1 条线程；全扫每条命令单线程，八条可并行。
串行：第二段的计时一格一格串行，因为几格并行会互相抢内存带宽与缓存，执行挂钟就不可比；分配器按线程分三笔计、在同一进程里一格一格读差，也串行；第一段与全扫没有必须串行的步。
预计挂钟：全扫八条每条约 3–5 秒（设计员在本机现跑的，没计时，推的），并行约 5 秒；第一段约 35 分钟（编译约 2 分钟、跑不到 2 秒、单测约 1 分钟、变异约 40 条按 `mutate.sh` 并行估 20–35 分钟，都是推的）；第二段约 60 分钟（同第二份：编译约 2 分钟、计时与逐一扫估 25–35 分钟、u = 13040400 那一处等高点建一版估 2–3 分钟、变异约 10 条估 15 分钟，都是推的）；三段都不到 2 小时，不另设分片单元与续跑入口。
预计在用核数：全扫 1–8 个核；第一段跑与单测时 1 个核，变异时按 `mutate.sh` 并行最多 31 个核；第二段计时与计数时 1 个核，变异时同上最多 31 个核（推的，依据是上面「线程」一行）。
