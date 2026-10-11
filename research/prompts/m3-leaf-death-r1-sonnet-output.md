# 正推腿报告：叶容器与记录的死亡怎么留在盘上（m3-leaf-death-r1，Sonnet 正推）

日期 2026-10-08。立场：正推（从条款、冻结树 `crates/`、E177 与 E176 的产物推出对的世界）。批次：`records/2026-10-01-删除与并发事务.md#删除线第 47 批`。本报告分段落盘，每段不超过 150 行；判定一览与「没做什么」在最后一段。

## 0 读法与前提

- 冻结树：`~/.local/share/singlefs/work/m3-leaf-death-r1/tree/`。我引的四份文件的 sha256 与 `research/prompts/m3-leaf-death-r1-snapshot/tree-sha256.txt` 逐份对过，原样输出：

```
ok crates/singlefs-core/src/inode_tree.rs
ok crates/singlefs-checker/src/walk.rs
ok crates/singlefs-harness/tests/checker_known_bad_images.rs
ok crates/singlefs-core/src/transaction.rs
```

- 引文写法：代码是冻结树内的 `crates/…:行号`（交回前用 `cite-check.py --root <冻结树> --root <仓根>` 跑）；kb 是仓里 `.claude/kb/…:行号`，行号这一轮 `grep -nF` 现取，不是背景材料的行号；产物是 `research/results/…:行号`。
- 「量」= 产物里有这一行；「推的」= 从条款与代码推出、没有哪个模型跑过。没量的数一律标「推的」。
- F1 的 460 是**臂 N**（不写任何墓碑）上量的，而 `crates/` 今天写的是**臂 B**（unlink 意图那次写整对象墓碑）。两臂在 inode 那一格的结局差得很远，下面 K1 先把这一点钉死再谈候选。

## K1　甲：对的世界是什么，改法候选

### K1.1 条款并排：同一件事 kb 与 `crates/` 各说了什么

并排一：D8 已定项 6 自己与 D18 已定项 8 对「死亡要不要在盘上留痕」说了相反的话。

- `.claude/kb/decisions/08-核心索引结构.md:169`：「类型 2 的记录死亡不写墓碑，死亡由现行叶版本体现，择现行版本压在 C113（扫描重建时多版单元的现行版本判定无输入）。」
- `.claude/kb/decisions/18-块里携带什么信息.md:204`：「删除若在盘上不留痕迹，扫盘重建就**没有任何输入**能知道这个 key 死过」
- 判定：冲突。D8 这句只管类型 2 记录，D18 已定项 8 说的是删除一般；D18 的理由「没有任何输入」对类型 2 同样成立，D8 那句靠的输入是「现行叶版本」，它只在叶容器还有现行版本时才有。

并排二：D8 的合并与删空条款、D18 登记的退役记录、`crates/` 的删空落法，三处对「一片容器不再存在」写的东西不一样。

- `.claude/kb/decisions/08-核心索引结构.md:171`：「保留左半身份，右半退役走 COW 释放路径；合并时机是写路径策略（单测 / 模型对拍），不进格式。」
- `.claude/kb/decisions/08-核心索引结构.md:171`：「记录数为 0 的类型 2 容器不落盘**（发布前必须合并掉；唯一的叶被删空时是不落盘、根变成零条目）。」
- `.claude/kb/decisions/18-块里携带什么信息.md:300`：「容器合并时右半退役、重生时旧身份退役，记录 = 被退役的 (出生树, 类型, 容器号, 出生代) 四元组，不带写序；住自己的容器，容器号按类型 1 那套单调分配」
- `crates/singlefs-core/src/inode_tree.rs:277`：「/// - 那一片删空了 ⇒ 整片不落盘（「记录数为 0 的类型 2 容器不落盘」，D8 已定项 6：左吸收右的退化形——右边那片是空的），」
- `crates/singlefs-core/src/inode_tree.rs:308`：「containers.remove(index.position());」
- 写者数：`grep -rn '退役记录' crates --include=*.rs | wc -l` 在冻结树里输出 `0`（我在 `~/.local/share/singlefs/work/m3-leaf-death-r1/tree/` 下跑的）。
- 判定：冲突。`crates/` 把「删空」当作 D8 说的「左吸收右的退化形」，D18 对「左吸收右」要求写一条退役记录，`crates/` 的退化形一条都没写。

并排三：退役记录有定义、有实验、没有载体。

- `.claude/kb/decisions/21-权威态与派生态的分界.md:207`：「写序 + 实例表行 + 祖先表 + 死亡写序 + 容器退役记录 + 清扫门六样全上时」（E104 的结论句，D21 已定项 9 依据段引它）。
- `.claude/kb/experiments/104-扫描重建的现行版本判定.md:85`：「| merge | 1 | 0 | 0 | no_retire 复活 2 | 退役容器最后一版里已死的 inode 2 |」
- `.claude/kb/experiments/104-扫描重建的现行版本判定.md:108`：「退役容器的旧版本恰好被盖掉，`no_retire` 与闭式都是 0，测试 `0 == 0` 空过」（E104 自己踩过的空过：旧版本被分配器盖掉时，没有退役记录也看不出复活）。
- `.claude/kb/decisions/18-块里携带什么信息.md:572`：「孩子是码 3 容器（打包记录类型 1 墓碑）」（墓碑树的孩子只有类型 1）。
- `research/prompts/m3-delete-r3-main-verification.md:60`：「W1「孩子类型 3 退役」在谱系一族里没有写者，删掉」（2026-10-01 把类型 3 从墓碑树的孩子里删了，没有另给它树）。
- 判定：规则没说。类型 3 退役记录在 D18 已定项 11 登记、在 E104 里承重，但没有任何一棵树的孩子集合含它，D18 已定项 24 的 6 条也没登记它的树种类码。

### K1.2 E177 的数：F1 打中的是臂 N，`crates/` 今天是臂 B

臂 N 的 inode 复活是量的，臂 B 的 inode 复活是 0，但臂 B 在「删空容器自己那一槽」上仍复活。产物行（R-S 读法，同一份产物 `research/results/e177-per-tree-resurrection-without-tombstones-segment-one.out`，sha256 `8708b822…bbbb5`）：

- `research/results/e177-per-tree-resurrection-without-tombstones-segment-one.out:37322`：「tree=inode status=resurrected r_peak=460」（G-环、臂 N）。
- `research/results/e177-per-tree-resurrection-without-tombstones-segment-one.out:37218`：「geometry=wide arm=N reader=R-S tree=inode status=resurrected r_peak=3」（G-宽、臂 N）。
- `research/results/e177-per-tree-resurrection-without-tombstones-segment-one.out:37346`：「geometry=ring arm=B reader=R-S tree=inode status=zero_but_missed_live r_peak=0 m_peak=1」（G-环、臂 B：inode 不复活，漏活 1，只在 unlink 途中，Q5）。
- `research/results/e177-per-tree-resurrection-without-tombstones-segment-one.out:5452`：「family=F1 n=900 order=ascending geometry=ring arm=B reader=R-S tree=allocation_records_a kind_code=3 publishes=1819 r_peak=3」（臂 B 下分配记录树仍复活 3，n = 900 删空 3 片容器）。
- `research/results/e177-per-tree-resurrection-without-tombstones-segment-one.out:5455`：「family=F1 n=900 order=ascending geometry=ring arm=B reader=R-S tree=central_mapping kind_code=5 publishes=1819 r_peak=3」（中央映射同）。
- `research/results/e177-per-tree-resurrection-without-tombstones-segment-one.out:435`：「family=F1 n=300 order=ascending geometry=wide arm=B reader=R-S tree=allocation_records_a kind_code=3 publishes=607 r_peak=1」（n = 300 删空 1 片：复活数 = 删空的片数）。

读法：臂 B 的 inode 为 0 是因为整对象墓碑 (inode, 对象出生代) 把留在旧版本里的已死记录压掉了（登记 5.3 的墓碑效力，`research/prompts/e177-preregistration.md` 里是装置自定的，K-W5b 没定，见 D18 `.claude/kb/decisions/18-块里携带什么信息.md:579`：「第一版单头、无快照、无清扫。扫盘重建怎么读墓碑与谱系记录不在这里定，记欠账（K-W5b」）。臂 B 的分配记录与映射仍复活，是因为读法 R-S 把「删空容器最新的那一版单元」当作活单元收下，它占的槽照样进分配记录与映射（`scripts/experiments/e177_per_tree_resurrection_without_tombstones.rs:2928` 起，对每个被收下的容器单元记 `slot` 与 `mapping_key`）；墓碑压的是记录，压不到容器单元自己。

### K1.3 把「死亡」拆成三种，各自需要什么载体（推的，每种在 E177 上有对应的量）

| 种 | 谁造成 | 记录还活着吗 | 今天 `crates/` | E177 对应的量 |
|---|---|---|---|---|
| D1 删空摘片 | unlink 最后一批删掉容器里最后一条 | 全死 | 有（`inode_tree.rs:308`） | F1 全族、F3 |
| D2 合并吸收右半（M3 要做） | 左吸收右 | 全活，搬进左半 | 没有（`inode_tree.rs:16` 起的模块头写明没写合并） | 无：E177 的历史没有合并（装置照 `crates/` 今天写） |
| D3 择现行退回更旧版本 | 一组的版本全已释放，槽按落点而不是按年龄被盖 | 旧版本里有些已死 | 同 D1 的后果 | G-环 r_out 峰值 459（`…out:37322` 的 `r_out_peak=459`） |

D3 不是独立的第三种病：只有「这一组已经没有现行版本」（D1、D2 之后）才会退回旧版本；活着的组现行版本被根持有、不会被盖。所以 D3 与 D1、D2 同根，下面 K1.6 单列判定。

### K1.4 副本装置量到的结局（先交代量法，下面每个候选引它）

我把入库的 E177 装置拷成副本（`research/prompts/m3-leaf-death-r1-sonnet-model/`，原文件 sha256 `9061c07b72a3c74693ddcacb885b0b4a9ebc4e4dc2c7082cfee320b3112848f5`，与冻结树里那份相同），加候选 `retire`、`zero`、`all_suppressed`（K2 里另加 `shrink_range`、`retire_shrink`）与两种合并政策，在同一份 51 段历史、三个几何上重跑。基线（不设环境变量）的产物与入库的第一段产物逐行相同（`rerun.sh run` 末尾自检，输出 `基线复现入库产物：逐行相同（除 name=candidate_run 与 name=done）`）。16 次运行里 15 份产物的 sha256 在两次独立重跑（两个空目录）之间逐份相同。**这些数是副本上的，不进 kb；要引，先在入库装置上重做。**

- 候选 `retire`：容器身份不再出现在这一版（删空摘片、被吸收的右半）时，模型里记一条 (容器号, 容器出生代) 退役；读侧 R-S 丢掉该身份的整组。
- 候选 `zero`：删空或被吸收的容器照样写一个零条目版本（写序最新，同一发布里释放，不被树引用，按 `inode_tree.rs` 删空的落法其余不变）。
- 候选 `all_suppressed`：只改读侧——一片容器里每条记录都被墓碑压掉就不当活单元收（只对臂 B 有意义）。
- 合并政策：`absorb` 是 E176 的 M2（相邻两片合起来不超过 233 条就左吸收右），`absorb_borrow` 是 E176 的 M3（再加删完不足 117 条且有右邻就均分借）。E177 原装置没有合并，这是副本加的。
- 格 = 该族各段 R-S 读法下 r 峰值的最大值，m 非 0 时加 `/m`。完整摘要 `research/prompts/m3-leaf-death-r1-sonnet-model/expected/summary.txt`（207 行）。

族 F1（建删空 inode，14 段，D1 与 D2 都在这里发生），臂 N（不写墓碑），inode 树 / `deleted_inodes` / 分配记录树 / 中央映射：

```
几何 ring（主取样点）   基线    absorb  absorb+借  retire  zero   all_suppressed
inode                   460     300     297        0       459    460
deleted_inodes          3       2       2          0       2      3
allocation_records_a    3       3       3          0       3      3
central_mapping         3       3       3          0       2      3
几何 wide               基线    absorb  absorb+借  retire  zero   all_suppressed
inode                   3       667     351        0       0      3
allocation_records_a    3       3       3          0       3      3
```

臂 B（`crates/` 今天），同一族，ring：inode 与 `deleted_inodes` 的峰值是 `0/m1`（漏活 1，只在 unlink 途中）；分配记录树与中央映射基线 3、`absorb` 3、`absorb+借` 3；加 `retire` 后 0；加 `all_suppressed` 后无合并时 `0/m1`、有合并时仍 3。
产物行佐证（基线 G-环 臂 N，同一格的原行）：`research/results/e177-per-tree-resurrection-without-tombstones-segment-one.out:5289`：「geometry=ring arm=N reader=R-S tree=inode kind_code=2 publishes=1819 r_peak=460」。

### K1.5 四个候选逐个

**候选一　退役记录（kb 已定的 C113 P5/P6，`crates/` 没有写者）**

- 定义：一个 inode 叶容器身份 (出生树, 类型 2, 容器号, 容器出生代) 不再出现在发布之后的树里（删空摘片、左吸收右的右半、唯一的叶被删空）时，同一次发布往一棵被根持有的容器树里追加一条退役记录；扫描重建对该身份的整组版本一律不收。记录字段取 `.claude/kb/decisions/18-块里携带什么信息.md:300`：「记录 = 被退役的 (出生树, 类型, 容器号, 出生代) 四元组，不带写序；住自己的容器，容器号按类型 1 那套单调分配」（出生树 8 + 类型 2 + 容器号 8 + 出生代 8 = 26 字节，一容器 ⌊(32768 − 136) ÷ 26⌋ = 1255 条，推的，字段表 kb 说与 C84 同一张表定、没定）。
- 载体（kb 没定，两个选法）：一A 另立一棵树，种类码 13，孩子是类型 3 容器。先例：`crates/singlefs-core/src/write_accounting.rs:25` 写着谱系树「另立一种、不并进墓碑树那一种（用户 2026-10-02 定「另立一种」）」。落点：`crates/singlefs-core/src/records.rs:385` 之后加常量、`crates/singlefs-format/src/lib.rs:188` 的 `NEW_POOL_FILE_CREATION_TREE_TABLE_ENTRIES` 由 11 改 12（新池新建文件的字节变，E142 的干跑数跟着变）、`crates/singlefs-core/src/recovery.rs` 里谱系树出现的 11 处、`crates/singlefs-checker/src/walk.rs` 的 9 处（I-9.17 / I-9.23 那一类条目身份判）要照样来一份。冻结树里 `grep -rln 'lineage' --include=*.rs . | wc -l`（在 `crates/` 下）输出 `32`，是上一次加一棵同类树碰到的文件数（含测试）。一B 并进墓碑树，孩子类型集合从 {0, 1} 改成 {0, 1, 3}：不加树种类码，但 `crates/singlefs-core/src/container_tree.rs:423` 的 `append_record_to_the_open_container` 按「最右那片」追加，两种类型混在一棵树里要按类型找各自的开放容器，且 `.claude/kb/decisions/18-块里携带什么信息.md:572` 的 I-9.23 合法集合要改。我倾向一A（先例），但这是取舍，交主 agent。
- 写者落点：`crates/singlefs-core/src/inode_tree.rs:286` 的 `remove_record_from_leaf_containers` 今天只回 `containers_dropped: usize`（`:125`），要回被摘的身份；`crates/singlefs-core/src/transaction.rs:5194` 起那一段 `match` 里 `Intent` 追加墓碑，最后一批与别的发布照抄（`:5209`），退役树要在 `FinalBatch` 那一支追加；`:4298` 起的重写根判定要加退役树；写账 `write_accounting.rs` 要给退役树的容器与根记账（unlink 的预付，D3 已定项 17「释放空间本身不需要申请空间」）。读侧：`crates/singlefs-checker/src/walk.rs:7385` 的 `scanned_content_units` 只判组内载荷相同与全序键不等，没有「择现行」也没有「这组没了」，要新写。
- 新增类：不新增单元类；类型 3 已在 D18 已定项 11 登记；新增一棵树（一A）或一种孩子类型（一B）。
- 崩溃点：退役容器与它的根进最后一批那次发布的单元段（持久顺序：单元、屏障、journal 记录、屏障、根槽，`.claude/kb/experiments/104-扫描重建的现行版本判定.md` 引的 D16 已定项 6 / 7），随根槽提交，不加屏障；该次发布的单元段多 2 个单元（容器 + 根，各 × 副本数）。崩溃点重放今天已经有 unlink 的流（`crates/singlefs-checker-tier/src/crash.rs:5188`「多记录流之后 unlink 的版本表」），这一格会被现有的流碰到；要加的判是「根在则退役记录在、根不在则退役记录也不可见（I-1.2 已发布谓词）」，今天的 checker 没有这一条（C116 的崩溃点那一格，`.claude/kb/checks-owed.md:106`，写的「层 0 固定脚本流没有 unlink」已经过期）。
- 要不要改条文：D8 已定项 6 的两句要补——`.claude/kb/decisions/08-核心索引结构.md:169`「类型 2 的记录死亡不写墓碑，死亡由现行叶版本体现」补「记录所在的容器身份消失时写退役记录」；`:171`「右半退役走 COW 释放路径」补「并写退役记录（D18 已定项 11 类型 3）」；删空的退化形（`inode_tree.rs:277`）按同一条办。C113 定案不改字，只多一个实例（删空摘片，原只列合并与重生）。
- E177 主取样点结局：量（见 K1.4）。臂 N ring：inode 460 → 0，`deleted_inodes` 3 → 0，分配记录 3 → 0，映射 3 → 0；wide 的 `absorb` 下 inode 667 → 0；`absorb+借` 351 → 0；tight（摘要里 `## 族 F1 几何 tight` 一节）inode 基线 3、`absorb` 6、`absorb+借` 7，加 `retire` 后全是 0。m 全程 0（没有判死实活）。剩下的非零格全是 extent 派生的（F3 的 195、F8 的 300），退役记录不管它们（K2）。E104 的 merge 世界在计数模型上给了同一方向的数：`.claude/kb/experiments/104-扫描重建的现行版本判定.md:85`「| merge | 1 | 0 | 0 | no_retire 复活 2 | 退役容器最后一版里已死的 inode 2 |」。
- 代价：每个消失的身份一条 26 字节记录；有记录的那次发布多写 2 个单元（容器 + 根）。次数：E177 的 51 段里 F1 28 次、F3 3 次、F8 32 次，占发布数 0.16% / 0.16% / 2.7%（`summary.txt` 事件数一节：28 / 16982、3 / 1837、32 / 1184）。E176 的 22 条序列里 M3 的 `merges` 与 `operations_accepted` 之比最大 0.214%（`wa_keep1of233_a2330`：`research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1032`「merges=28634 borrows=179568」）。第一版不回收（同 D18 已定项 24 第 4 条「第一版不回收墓碑」，`.claude/kb/decisions/18-块里携带什么信息.md:575`），按 26 字节算，1e6 次吸收约 25 MiB（推的）。
- 风险（推的，没人攻过）：管理员回退取 R_old 的树时，退役树若不随 R_old，被回退掉的那段里写的退役记录会把 R_old 里仍活着的容器身份丢掉——要像墓碑树那样列进取 R_old 的一类（`research/prompts/m3-tomb-r1-main-verification.md:65`：「D23 已定项 14 把墓碑树（与它的根）列进取 R_old 的一类」；检查欠账在 C614、C615，`.claude/kb/checks-owed.md:474-475`）。克隆头的跨头首次 COW 另有一条 P6 退役，不在这一轮。

**候选二　删空的容器照样落一个零条目版本**

- 定义：删空（或被吸收）那次发布，给这个身份写一个记录数为 0 的版本，写序最新；扫描重建按「同身份取最新」就取到零条目版本。
- 落点：`crates/singlefs-core/src/inode_tree.rs:308` 的 `containers.remove(index.position());` 之前补写一个空容器进写集；`transaction.rs` 的角色表要加「不被树引用的容器」（今天写集只来自 `inode_tree.containers`，`transaction.rs:5526` 起按 `0..inode_tree.containers.len()` 列）；`.claude/kb/decisions/08-核心索引结构.md:171`「记录数为 0 的类型 2 容器不落盘」要改；`.claude/kb/invariants.md:301` 的 I-9.13「类型 2 容器不空」若空容器被树引用就红。
- 新增类：不新增。崩溃点：单元段多 1 个单元（× 副本数）。
- 要不要改条文：改 D8 已定项 6 的「不落盘」那一句；C113 不动。
- E177 主取样点结局：量。ring 臂 N F1：inode 460 → 459，`deleted_inodes` 3 → 2，分配记录 3 → 3，映射 3 → 2——几乎没动；wide：inode 3 → 0，但分配记录仍 3；tight：inode 3 → 3。原因（推的，副本里零版本的释放代与别的版本同一发布）：零条目版本不被任何树引用，同一发布里就进释放清单，环里等一个环长后与旧版本一样被回收、被按落点盖掉（基线 ring 的 `reuses=18506`，`research/results/e177-per-tree-resurrection-without-tombstones-segment-one.out:5289`），盖掉的先后与年龄无关，零版本被盖在前、旧版本留在后时又退回到旧版本。wide 不复用槽所以零版本一直在，inode 回到 0。分配记录与映射的复活还有第二个来源：零版本自己占的槽是「被判活的单元」，真值里没有它，所以分配记录 3 不降；装置自己的断言 A3、A4、A13（死亡记录恒 0、wide 读全）因此变红，`void=true`（摘要最后一节）。
- 代价：每个消失的身份多写 1 个 32 KiB 单元，且这个单元活不过一个环长，所以不是死亡的载体。要让它活着只能让树继续引用空叶——那是「不合并」，I-9.13 红、叶数只增不减（推的）。

**候选三　类型 2 的记录死亡也写墓碑**

- 逐条（每条记录一条）：不用新写——`crates/singlefs-core/src/transaction.rs:5997`「tombstone: TombstoneRecord::of_the_whole_object(inode, record.object_birth, txg),」已经在 unlink 意图那次发布按 (inode, 对象出生代) 写了整对象墓碑，`transaction.rs:5195` 追加进墓碑树（`records.rs:122` 起的 `TombstoneRecord` 五个字段：对象 inode、对象出生代、区间起止、死亡 txg）。一条 inode 记录对应一个对象，所以「逐条」与「整对象」在 inode 树上是同一条。
- 逐容器：墓碑记录的字段是对象 + 区间，指认不了容器身份；逐容器墓碑要新记录形，等于候选一换名字。
- E177 结局：量，就是臂 B。G-环 inode：`research/results/e177-per-tree-resurrection-without-tombstones-segment-one.out:37346`「geometry=ring arm=B reader=R-S tree=inode status=zero_but_missed_live r_peak=0 m_peak=1」。三个几何、F1 的 N 与 B 对照：臂 N inode 460 / 3 / 3，臂 B 峰值 0。**F1 在 `crates/` 今天的写法（臂 B）下，inode 树和 `deleted_inodes` 不复活**——前提是读侧认「整对象墓碑压掉类型 2 记录」，而 kb 没写读侧（`.claude/kb/decisions/18-块里携带什么信息.md:579` K-W5b 欠）；E177 登记 5.3 是装置自定。
- 没盖住的：臂 B 下分配记录与映射的复活（F1：3，每个删空的容器一个，`…out:5452`「tree=allocation_records_a kind_code=3 publishes=1819 r_peak=3」），原因是墓碑压的是记录，容器单元自己仍被 R-S 收下；左吸收右的右半（记录还活着，墓碑指认不了）——`absorb` 下臂 B inode 仍 0，分配记录 / 映射 3。
- 漏活：墓碑在意图那次生效，意图到最后一批之间还活着的 nlink = 0 记录被压掉，m = 1（`…out:37453`「tree=inode m_peak=1 m_positive_scans=9905 m_positive_scans_in_unlink=9905 m_positive_scans_outside_unlink=0」）。nlink = 0 的对象对只读视图本就是孤儿，我推它无害（推的，没人攻过）；要消掉它得让墓碑生效等最后一批（K-W5b 的另一边），那时意图到最后一批之间崩溃重建出的视图会多一个 nlink = 0 的对象。
- 代价：inode 树上 0 条新写；条文要补一句读侧（墓碑压类型 2 记录、生效时刻）。

**候选四　只改读侧的择现行，不加盘上东西**

- 要有输入才认得出「这一组没了」。盘上今天能用的只有三样：墓碑（臂 B）、记录自带的改动计数（`.claude/kb/decisions/08-核心索引结构.md:189` 记录字段表，偏移 88）、父指针（派生态，丢了才重建；`.claude/kb/decisions/18-块里携带什么信息.md:137`：「候选 ①（靠父指针）在「索引丢了才要重建」这个前提下是循环的」）。**不加墓碑的臂 N 上读侧没有任何输入**：副本里 `all_suppressed` 加在臂 N 上与基线逐格相同（摘要里 `allsup` 列的 N 行 = `base` 列）。D18 已定项 8 的话原样适用：`.claude/kb/decisions/18-块里携带什么信息.md:204`「删除若在盘上不留痕迹，扫盘重建就**没有任何输入**能知道这个 key 死过」。
- 有墓碑的臂 B 上能做的读侧规则：一片容器里每条记录都被墓碑压掉就不当活单元收（副本 `all_suppressed`）。量：F1 无合并时分配记录 / 映射 3 → `0/m1`（删空的容器不再算活单元，漏活 1 是 unlink 途中那片还装着 nlink = 0 记录的容器）；有合并（`absorb`）时仍 3，因为被吸收的右半记录还活着、墓碑压不掉它；F3 的 m 从 1 变 2（多漏一个）。落点：`crates/singlefs-checker/src/walk.rs:7385` 起的读侧，新写。
- 结论：读侧单独救不了；依附于候选三，且管不到 D2。

### K1.6 「G-环 上择现行还会退回更旧的版本」是不是同一个病

是同一个病，三个依据（第 1 条是机制，推的；第 2、3 条是量）：

1. 机制：一组只有在没有现行版本时才会退回旧版本。活着的组，现行版本被根持有、不会进释放清单，轮不到被盖；只有删空、被吸收这两种让整组失去现行版本的事之后，这一组的所有版本都在释放清单里，按落点被盖，剩下哪一版取决于落点，不取决于年龄。
2. 量：臂 N ring F1 inode 460，其中环窗外 459（`research/results/e177-per-tree-resurrection-without-tombstones-segment-one.out:37322` 的 `r_in_peak=3 r_out_peak=459`）；同一格 wide 只有 3。环窗外那 459 是旧版本被择中的部分（E177 页 Q14 一节的说法），它们装着已死的记录。
3. 谁治得了：按身份挂的东西（候选一，按记录挂的候选三）把整组或每条记录都标死，不管剩下哪一版——候选一 ring 460 → 0，候选三（臂 B）0；按版本挂的东西（候选二，零版本）本身也是一个会被盖的版本，ring 460 → 459。

### K1.7 我推出的对的世界

- 对的世界不是「死亡由现行叶版本体现」一句话，而是两层：记录死亡由叶的现行版本体现（组还活着时成立，量到：加了 `retire` 后臂 N 的 m 峰值 0、没有一格因它而复活），**容器身份消失由一条被根持有的退役记录体现**。这正是 D8 已定项 6 自己引的 C113 P5/P6、D18 已定项 11 登记的类型 3、D21 已定项 9 依据段引的 E104 六样之一（`.claude/kb/decisions/21-权威态与派生态的分界.md:207`）。E177 的 F1 量到的是缺了这一件时的后果（臂 N 的 R-S 没有退役记录，登记 F2 条款字面上要求查它，见 K4）。
- 合并与借做成后：吸收的右半记录还活着，墓碑指认不了，必须有退役记录（副本 `absorb` 下臂 N 的 inode wide 3 → 667，臂 B 的 inode 仍 0 但分配记录 / 映射 3 不降，`retire` 全 0）；借不让任何身份消失，不需要退役记录（见 K3）。
- 候选一和候选三不互斥：墓碑负责 extent / 数据单元那一类（K2），退役记录负责容器身份。若只留一个：只留候选一，inode 记录的死亡仍要有读侧的「组还活着时看现行版本」，量到全 0；只留候选三，分配记录 / 映射的容器槽与 D2 管不到。

## K2　乙：C109 的候选怎么改写

### K2.1 逐类：谁的 key 死亡需要载体，谁不需要

判法：先问这棵树的态别（kb），再看 E177 臂 B（`crates/` 今天）在 R-S 下的状态，最后问复活是不是随它的源单元来的。

- `.claude/kb/decisions/21-权威态与派生态的分界.md:189`：「| **派生态** | **索引（树）** |」；`:202`：「分配记录落派生态（已定项 5）」。E177 对 extent、分配记录、中央映射、`deleted_inodes` 都按派生态从被判活的单元算（`scripts/experiments/e177_per_tree_resurrection_without_tombstones.rs:2839` 起的 `select_current_reading`，`:2928` 起对每个被收下的数据单元与容器记槽号与映射 key）。
- 所以这几棵树的 key 死亡不需要自己的死亡记录：源单元（数据单元、inode 叶容器）判死了，它们跟着没有。副本里证这一点：把源单元的死亡补齐（候选 `retire_shrink`）后，臂 B 的 extent / 分配记录 / 映射在 F1、F3、F8 的 ring 与 wide 上 r 全 0（摘要 `ret+shr` 列）。

| 树 | 态别 | 今天的死亡载体 | E177 臂 B ring（原行） | 对的世界 | 缺口 |
|---|---|---|---|---|---|
| inode（码 2） | 权威态单元（容器） | 整对象墓碑 + 现行叶版本，没有退役记录 | `…out:37346` inode `zero_but_missed_live r_peak=0 m_peak=1` | 记录死亡看墓碑 / 现行版本，容器身份消失看退役记录（K1） | 退役记录无写者；读侧 K-W5b 欠 |
| `deleted_inodes`（码 9） | 派生（nlink = 0 的号） | 无 | `…out:37353` 同上 `r_peak=0 m_peak=1` | 随 inode | 无 |
| extent（码 1） | 派生（从数据单元重算） | 整对象墓碑；变短没有 | `…out:37345` `status=resurrected r_peak=299 m_peak=300` | 数据单元的死亡有载体：整对象墓碑 + 变短时的区间墓碑 | 区间墓碑无写者 |
| 分配记录（码 3） | 派生（D21 已定项 5） | 无（释放写成已释放 + 释放代，`allocator.rs:1796`） | `…out:37347` `status=resurrected r_peak=299 m_peak=300` | 随源单元：数据单元（墓碑）与容器（退役） | 同上两条 |
| 中央映射（码 5） | 派生（R-S 从被判活的单元算 key） | 无 | `…out:37395` `status=resurrected r_peak=299 m_peak=300` | 同分配记录 | 同上两条 |
| 记账（码 4） | 统计量，读现行值 | 每次发布整批重写、取代号最大 | `…out:37349` `status=measured_zero r_peak=0 m_peak=0` | 不需要载体 | 无（F3：R-U 下会复活，R-S 量到 0 只说明按读现行值读） |
| 墓碑（码 10）、谱系（码 11） | 权威态（只增） | 无死亡 | `…out:37354` `status=no_death` | 第一版不回收（D18 已定项 24 第 4 条） | 回收那天要有门 |
| livelist、稀疏旁表、deadlist、logged_ops（6 / 7 / 8 / 12） | — | `crates/` 恒零根 | 判定行全是 `status=no_object`（这四棵 56 行，加谱系树 70 行，用 `grep 'name=verdict' <产物> | grep -v 'question=Q13' | grep -E 'tree=(lineage|livelist|sparse_side_table|deadlist|logged_operations) ' | grep -c 'status=no_object'` 数出 `70`） | 登记时定 | 无对象 |
| dirent、反向索引、空闲空间、LRU | — | `crates/` 没有写者，kb 没有载体 | E177 Q7：「不可量」 | 见 K2.4 | 载体没定义 |

状态列的原行（G-环、臂 B、R-S）：`research/results/e177-per-tree-resurrection-without-tombstones-segment-one.out:37345`「geometry=ring arm=B reader=R-S tree=extent status=resurrected r_peak=299 m_peak=300」；`:37395`「geometry=ring arm=B reader=R-S tree=central_mapping status=resurrected r_peak=299 m_peak=300」。

### K2.2 文件变短那种死亡（F8 e = OU）：对的世界是区间墓碑

- 事实：`crates/` 里变短不写任何墓碑。覆盖写与顺序写都走 `publish_version` 的「既不是删除也不是回退」那一档（`crates/singlefs-core/src/transaction.rs:6242`、`:6312`：「&DeletionAndLineageOfThePublish::NEITHER_A_DELETION_NOR_A_ROLLBACK,」）；`TombstoneRecord` 有区间起止两个字段（`crates/singlefs-core/src/records.rs:125`「pub range_start_in_bytes: u64,」、`:126`），但构造只有整对象一种（`:132`「pub const WHOLE_OBJECT_RANGE_START_IN_BYTES: u64 = 0;」）。
- kb 的并排：`.claude/kb/decisions/18-块里携带什么信息.md:235`：「truncate / punch 这类部分删除按**每段连续 extent 区间一条**」（D18 已定项 10 的粒度）与 `:572` 起已定项 24 第 2 条「整对象写 [0, 2⁶⁴ − 1]」——后者是第一版的范围，前者是粒度；两者不冲突，变短是粒度已定、写者没有。
- 量：`research/results/e177-per-tree-resurrection-without-tombstones-segment-one.out:37369`「question=Q11_OU geometry=ring arm=B reader=R-S tree=extent status=resurrected r_peak=299 m_peak=1」；分配记录同形（`:37371` `r_peak=299`）。
- 副本里加一条区间墓碑 (对象, 出生代, 起始单元序号, 死亡 txg)，R-S 对臂 B 丢掉「起始单元序号以后、诞生 txg 不晚于死亡 txg」的数据单元组（候选 `shrink_range`，摘要 `shrink` 列）：F8 ring 臂 B：extent 299 → 0，分配记录 299 → 1，映射 299 → 1（剩下的 1 是唯一的 inode 叶被删空的那一槽，K1 的退役记录管）；F3 ring：extent 195 → 0，分配记录 / 映射 195 → 2。`retire_shrink` 之后 F8 / F3 的 extent、分配记录、映射全是 0，m 不变（F8 300、F3 1，是墓碑生效时刻，不是这条候选引入的）。
- 代价（推的，写法照 `crates/` 的 `plan_tombstone_append`）：每次变短的发布往墓碑树的开放容器追加 1 条 40 字节记录 ⇒ 容器重写 1 个 + 根重写 1 个，共多 2 个单元（× 副本数）。次数：E177 的 F3 50 次 / 1837 次发布（2.7%），F8 12 次 / 1184 次（1.0%）（摘要事件数一节 `shrink_range_none`）。崩溃点：这 2 个单元进变短那次发布的单元段，不加屏障。准入：墓碑的预付式是 `crates/singlefs-harness/tests/admission_ring_window_and_tombstone_prepay_on_the_publish_path.rs:2`「标签 6 = 副本数 × ((2 + H + 1) × 16384 + 41 × n_live)」，按活对象数预付；一个文件变短几次不在这个式子里，区间墓碑要改预付式（推的，没查式子在覆盖写路径上怎么用）。

### K2.3 T1、T2 按 K1 的结论改写成能交用户的候选

C109 现行文字：`.claude/kb/checks-owed.md:100`「2026-10-01 收窄：第一版只 inode 对象整对象写墓碑（D18（块里携带什么信息） 已定项 24），别的树的 key 死亡待定」。

- **甲　今天的整对象墓碑（原 T1 的字面）**。量：臂 B。inode、`deleted_inodes` 不复活（前提：读侧认墓碑，K-W5b）；extent / 分配记录 / 映射在 F3、F8 上复活 195 / 299，F1 上分配记录 / 映射 3（删空容器的槽）；在 `absorb` 下 inode 仍 0、分配记录 / 映射 3。多写：0。条文：只补读侧（墓碑压类型 2 记录、生效时刻）。
- **乙　甲 + 变短写区间墓碑**。多写：每次变短 +2 个单元（K2.2）。量：F3 / F8 的 extent 复活 → 0，分配记录 / 映射剩容器槽（F8 1、F3 2）。条文：D18 已定项 24 第 2 条加「变短写 [新末尾, 2⁶⁴ − 1]」，已定项 10 的粒度不动。
- **丙　乙 + 容器退役记录（K1 候选一）**。多写：再加每个消失的身份 +2 个单元（K1.5）。量：`retire_shrink` 全部格 r = 0（臂 B 的 inode 与 `deleted_inodes` 仍有 m = 1 的 unlink 途中漏活，F8 的 extent / 分配记录 / 映射 m = 300，是同一个生效时刻问题）。条文：D8 已定项 6 两句、D18 已定项 24 加一条树或一种孩子类型。
- **T2　全部 keyspace**：不再逐树列到「…」，改成登记表的一条规则——每棵树登记时同时登记它的 key 死亡由谁承载：权威态单元类 ⇒ 墓碑（对象或区间）或退役记录；派生树 ⇒ 不写，写明从哪几类单元重算；统计量 ⇒ 读现行值。今天登记表（`.claude/kb/decisions/08-核心索引结构.md` 已定项 9 的 12 行）加一列，现有 12 行按 K2.1 的表填。dirent、反向索引、空闲空间、LRU 没有 `crates/` 写者也没有 kb 载体，E177 判「不可量」，这一列在它们有载体那天填，不在今天猜。代价 0 字节；约束写者：先登记载体再写。

K-W5b 的生效时刻（墓碑什么时候压东西）要用户定：意图那次生效 ⇒ m 峰值 inode 1、extent 300（量）；最后一批才生效 ⇒ 无漏活，但意图到最后一批之间崩溃，重建视图多一个 nlink = 0 的对象和它没截完的数据（推的）。区间墓碑没有这个问题（变短那次发布就是死亡）。

## K3　丙：M3 的 I-9.4 新判据与借多少

### K3.1 今天的 I-9.4 与拦它的坏镜像

- `.claude/kb/invariants.md:292`：「沿叶序容器号严格递增，且左容器的最大 key < 右容器号（这条逼出合并只许左吸收右 / 左从右借）」；C536 的题面 `.claude/kb/checks-owed.md:415`：「借走右容器最小的那条之后，左容器的最大 key 可以 ≥ 右容器号，I-9.4（容器号不超最小 key） 的「沿……」那一半当场为假」。
- 四句各在哪（冻结树）：第一句（记录 inode ≥ 容器号）`crates/singlefs-checker/src/walk.rs:3079`「self.judgements.judge("I-9.4", inode >= container, || {」；第二句（容器号严格递增）`crates/singlefs-checker/src/walk.rs:2775`「.judge("I-9.4", left.container < right.container, || {」；第三句（左最大 key < 右容器号）`crates/singlefs-checker/src/walk.rs:2782`「.judge("I-9.4", left.largest_inode < right.container, || {」；第四句（同一片里记录号严格递增）`walk.rs:3067`。GPU 判器有第二、三句的副本：`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2918`「w_judge(INVARIANT_I_9_4, lt64(left_largest, right_container));」。
- 坏镜像：背景材料说它们在 `checker_known_bad_images.rs`，那个文件里 I-9.4 只有两条：`crates/singlefs-harness/tests/checker_known_bad_images.rs:1064`「// 容器号改成 5，而容器里那条记录的 inode 是 1。」（第一句）与 `:10599`「fn a_leaf_holding_the_same_inode_number_twice_reddens_only_the_container_number_invariant() {」（第四句）。第二、三句的坏镜像在 `crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs`：`:936` 阳性对照（第二片容器号 11，三句都成立）、`:947`「fn a_right_container_number_not_above_the_largest_key_on_its_left_reddens_only_the_container_number_invariant(」（第三句，第二片容器号 8、左边最大 key 10）、`:962`（第二句，第二片容器号 0）。变异表里对应的行 `crates/mutations.tsv:951`「实审 B2 第 9 条（I-9.4 第二句）」、`:952`「实审 B2 第 9 条（I-9.4 第三句）：左容器」。

### K3.2 借之后第三句为假，而且不是记账上的假

- 必然：右片每条记录 ≥ 右容器号（第一句）；借走的是右片最小的几条，所以它们 ≥ 右容器号，进了左片，左片最大 key ≥ 右容器号。借一次、第三句为假，直到那几条又被删光。
- 量：E176 第二段，M3 在 `ws_a1000_d500_smallest_first` 上 `research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:787`「operations_accepted=92245」后面的「i94_third_false_positive_publishes=44919」——该臂接受 92245 次操作，其中 44919 次发布之后第三句为假（同一行 `i94_third_false_peak=1`）。E176 登记自己写明「第三句在 M1、M2 上断言、在 M3 上计数」（`.claude/kb/experiments/176-inode树叶合并三候选的容量与写代价.md:26`）。
- 真缺陷是路由：`.claude/kb/decisions/08-核心索引结构.md:164`「分隔 key** = 该孩子创建时的最小 key，条目按分隔 key 单调递增，查找取最后一个分隔 key ≤ 目标 key 的条目」。借走的 key ≥ 右容器号 = 右片的分隔 key，查这些 key 会路由到右片，右片里已经没有。所以光改判据放行不够，借的那次发布要同时把右片在父节点里的分隔 key 抬到不小于被借走的最大 key 之后。`crates/singlefs-core/src/inode_tree.rs:80`「今天的最小 key 可以比诞生时大，而分隔 key 不许跟着变（跟着变就等于改父节点里的路由）。」今天的代码明令不许动，借恰恰要动。

### K3.3 新判据（改判据）与改借法，各拦得住什么

候选 N1　改判据，分隔 key 随借抬：
- 条文：分隔 key = 创建时取该孩子的最小 key，借之后右片的分隔 key 改成它现在的最小 key；容器号（身份）不变。I-9.4 第三句改成「沿叶序，左容器的最大 key < 右容器的分隔 key（父条目里那 8 字节）」，沿全部叶序判、跨内部节点也判；第一、二、四句不变；I-9.12（`.claude/kb/invariants.md:300`）原样保留，它只在同一个内部节点的条目之间判（`crates/singlefs-checker/src/walk.rs:3263`「.judge("I-9.12", *separator > previous_largest, || {」）。
- 落点：`crates/singlefs-core/src/inode_tree.rs:82` `separator_key()` 现返回 `identity.container`，改成容器自己存一个分隔 key；`transaction.rs:7365`、`:7369`、`:7386`、`:7541` 四处调用；`walk.rs:173` 的 `InodeLeafContainerInLeafOrder` 加一个分隔 key 字段，`:2782` 改比较；`crash_judge_checker.wgsl:2918` 跟着改；D8 `.claude/kb/decisions/08-核心索引结构.md:164` 与 `inode_tree.rs:77-80` 的注释改；`invariants.md:292` 改。
- 逐条坏镜像：

```
镜像                                           今天   N1
:1064 容器号 5、记录 inode 1（第一句）          红     红（第一句没动）
:10599 同一片里 1 号两条（第四句）              红     红
:962 第二片容器号 0（第二句）                   红     红
:936 阳性对照 第二片容器号 11                   绿     绿
:947 第二片容器号 8、左最大 key 10、右记录 20 30、分隔 key 15   红     绿
新：左片 [1, 17]、右片 [20, 30]、分隔 key 15（借后没抬分隔 key）  —      红（17 不小于 15，I-9.12 与新第三句都红）
```

  `:947` 那条在 N1 下变绿，不是漏拦：左片 [1, 10]、右片 [20, 30]、右容器号 8、分隔 key 15 正是「右片原有 8 … 30，左片借走 8 … 10 后又删了 8、9」的合法形状；它要改成阳性对照，另补最后一行那种新坏镜像。`mutations.tsv:952`、`:953` 两行要重指（第三句的变异换成新比较）。
- 代价：不加任何新写——父节点借的那次发布本来就整条路径重写（`inode_tree.rs:132` 的 `is_the_root_rewritten`）。

候选 N2　改借法，让旧判据成立：
- 借的同时给右片换身份（容器号 = 借后它的最小 key，出生代 = 这次发布），旧身份写退役记录（K1 候选一）。第三句原样成立（左最大 key < 新容器号）。
- 逐条坏镜像：六条照旧，`:947` 仍红。
- 代价：每次借 +1 条退役记录。E176 的借次数：`ws_a1000_d950_smallest_first` 均分借 17277 次、刚好够下限借 286396 次（`research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:836`「merges=2470 borrows=17277」、`:837`「merges=2471 borrows=286396」）；按 26 字节 / 条：约 0.45 MB 对 7.4 MB（推的算术），多 2 个单元（容器 + 根）的发布次数同比。「分裂后左半身份不变」与「COW 重写身份不变」两条断言之外又多一条「借之后右片换身份」。

没有第三种：要让旧第三句不改而不换身份，借走的 key 得小于右容器号，而右片里没有这样的 key（第一句）。

我的取舍：N1。理由是它不加盘上东西、不改身份、旧坏镜像除 `:947` 一条（本来是借的合法形状）外都还拦得住；N2 把借变成换身份，代价按借次数涨。交主 agent 的是：N1 要改 D8 已定项 6 的分隔 key 一句（`:164`）与 I-9.4 第三句，属于改已定项原文，写回之前交用户。

### K3.4 借多少：均分借 vs 借到刚好够下限

量（E176 第二段，原行；同一条序列上两种借法的 `merges` 相同或差 1，`borrows` 差一个数量级）：

```
序列 ws_a1000_d950_smallest_first     均分借(M3)    刚好够下限(M3_just_enough)
merges                                 2470          2471
borrows                                17277         286396
borrows ÷ merges                       6.995         115.9
内容记法每次 unlink 重写片数均值        1.0300        1.4975        （M1 0.9957）
内容记法单次最大                        2             2
序列 wa_keep1of233_a2330
merges / borrows                       28634 / 179568   28634 / 2951927    （比值 6.27 / 103.1）
```

原行：`research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:843`「arm=M3 unlinks=575700 content_sum=592977 position_sum=757638 content_mean=1.0300」；`research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:858`「arm=M3_just_enough unlinks=575700 content_sum=862096 position_sum=1026775 content_mean=1.4975」；`research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:788`「merges=130 borrows=15174」。18 条有合并的序列上 `borrows ÷ merges` 的最大值，命令与这一次跑的原样输出（仓根）：

```
grep 'name=arm_run' research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out | awk '{for(i=1;i<=NF;i++){split($i,kv,"="); v[kv[1]]=kv[2]} if ((v["arm"]=="M3"||v["arm"]=="M3_just_enough"||v["arm"]=="M3_left_first") && v["merges"]>0) { r=v["borrows"]/v["merges"]; if (r>m[v["arm"]]) {m[v["arm"]]=r; s[v["arm"]]=v["sequence"]} n[v["arm"]]++ } } END{for (a in m) printf "%s: 最大 borrows/merges = %.3f（%s），有合并的序列 %d 条\n", a, m[a], s[a], n[a]}' | sort
M3_just_enough: 最大 borrows/merges = 116.723（ws_a1000_d500_smallest_first），有合并的序列 18 条
M3_left_first: 最大 borrows/merges = 7.023（ws_a1000_d500_smallest_first），有合并的序列 18 条
M3: 最大 borrows/merges = 7.023（ws_a1000_d500_smallest_first），有合并的序列 18 条
```

我手推的界（推的，E176 没证）：设某一对 (X, R) 删完 X 里一条之后 |X| < 117、两片之和 S ≥ 234，且这次没并。
- 刚好够下限：借 117 − |X| = 1 条，借后 |X| = 117、S 不变；X 再掉一条又借一条，直到 S 掉到 233 以下并掉。一对在并之前至多借 S₀ − 233 ≤ 116 次（S₀ ≤ 349，因为 |X| ≤ 116、|R| ≤ 233）。
- 均分借：借 ⌊(|R| − |X|) ÷ 2⌋ 条，借后 |X| = ⌊S ÷ 2⌋、|R| = ⌈S ÷ 2⌉；X 要再掉 ⌊S ÷ 2⌋ − 116 条才触发下一次借，同时 S 同步少那么多。记 D = S − 232，则 D → ⌈D ÷ 2⌉，到 D ≤ 1（S ≤ 233）并掉。一对在并之前至多借 ⌈log₂ D₀⌉ ≤ 7 次。
- 对上量：最大比值 7.023、116.723 贴着 7 与 116，多出的 0.023 与 0.723 次 / 并是运行结束时还没并掉的那些对借的次数（推的，没逐对核）。

所以：
- **代价界更好证的是均分借**：每次 unlink 至多多重写 1 片（两种借法都是 2 片，`content_largest=2`），但均分借把「多重写的那次」的频率压在每次并之前 ≤ 7 次，刚好够下限是 ≤ 116 次；E176 的 Δ3c 在均分借上最大 0.0343（≤ 0.10 判可忽略），刚好够下限上 0.5018（0.10 到 1.00 之间判不翻）。E176 的不稳定正是这条旋钮：15 对里主窗口「可忽略」、刚好够下限「不翻」。均分借的界不靠取样点就能写成闭式，刚好够下限的上确界贴着显著的门槛 1.00（每次 unlink 都借一条）。
- 对 K1 选出的死亡记录形态：N1 下借不让任何身份消失，退役记录 0 条，两种借法一样；两种借法的 `merges` 一样（130 对 130、2470 对 2471），所以并带来的退役记录条数一样。N2 下每次借 +1 条退役记录，界同上（≤ 7 / 并 对 ≤ 116 / 并）。
- 对崩溃点：借的那次发布单元段多 1 个单元（右片重写，× 副本数），两种借法同；多出来的发布次数按上面的比值，均分借的约为刚好够下限的 6%（17277 ÷ 286396，推的：层 0 的有界枚举按发布数线性加状态，没量）。

### K3.5 合并与借做成后，K1 候选在两形上各写什么

```
形                         候选一 退役记录      候选二 零版本            候选三 墓碑            候选四 读侧
被吸收的右半（身份消失，     1 条退役记录         1 个零单元，但活不过      墓碑指认不了           只能靠同 key 重复时
 记录还活着，搬进左半）      +2 单元              一个环长；量：ring        （记录活着）；量：     比改动计数（推的，没写）
                                                  inode 345 对基线 300      臂 B absorb inode 0，
                                                                           分配记录 / 映射 3
借走记录的右片（身份在，     0                   0                         0                      0
 现行版本记录变少）
借的左片 X（身份在）         0                   0                         0                      0
```

量处：`absorb`、`absorb+借` 两列在 K1.4 的表与摘要里（`retire` 列全 0；`zero` 列 ring 345 / 350；`allsup` 列分配记录 / 映射 3）。借走记录的右片不需要任何死亡记录，因为它的现行版本是最新的一版：副本里 `absorb+借` 比 `absorb` 多 50 次借（F1），retire 列都是 0，baseline 列 inode 反而少（wide 667 → 351；我推是借让一部分本要被吸收的右半留了下来，没逐段核）。

## K4　E177 的读法与决策表关系

### K4.1 执行员替登记取的六个读法，逐个判

登记原文在 `research/prompts/e177-preregistration.md`，核对表 5 的各行是 `:821`（C1）到 `:835`（C14）、`:828`（C17）。

- **C1　分配记录树按槽号计一条 key：站得住。** 登记第一节自己写（`research/prompts/e177-preregistration.md:20`）「两盘镜像的两份按一条 key 计」，按 (设备, 槽号) 计时 PC1 是 +2，与登记 5.5 写的 +1 冲突。不用改。
- **C2　d 按「之前某版里有、扫描时刻这一版里没有」数：站得住，而且它不是补读法，是登记第一节那句。** `research/prompts/e177-preregistration.md:18`：「同一个 key 后来又出生（分配记录树的槽号会重用）的，按扫描时刻那一版判。」登记另一处把 d 写成「死过的不同 key 数」，两处字面打架，C2 取了与 A3（r = d）相容的那一边。d 只是分母、不进任何判定，换成「死过就计」只会让 A3 在 F3 上由定义不成立。改法：实验页「做了什么」里把 C2 从「登记没说到」挪到「登记第一节已写」。
- **C3　R-S、R-G 下分配记录树与中央映射树的真值取这一版被判活的数据单元与容器：站得住。** 登记 5.4 对这两棵树 R-S 一列写的就是「从被判活的单元算」，真值与读法同口径是同一句的另一半。后果要写明：口径 (a)、(b) 在 R-S、R-G 下逐格相同，所以这两行没有两个口径的信息；另一个后果是容器自己的槽进了这两棵树，K1 里「臂 B 下删空容器的槽仍复活」就是这个口径量出来的。
- **C13　R-G 的阳性对照只植对象在 R-S 重建出的 inode 树里有记录的已死 key：站得住，但削弱了 R-G 的阳性对照。** 不这么植时 R-G 下臂 N 那一格由定义是 +0，登记按作废处理。代价：R-G 的对象门控「拒不拒得掉」没有阳性对照，「R-G 与 R-S 逐格同状态」不能用 PC1 佐证门起了作用；E177 页自己写 V5（门控关掉）由手造盘那条单测抓到（`.claude/kb/experiments/177-逐树不写墓碑的复活数.md` 单测与变异一节）。改法：再加一种 PC1，植一个对象**不在**重建 inode 树里的已死数据单元，期望 +0，一格不对就说明门没起作用。
- **C14　PC2 的期望由另写的手算函数给，扣掉同组退回去的旧版本还供出的 key：站得住。** 登记 5.5 写「执行员先手算再跑」，字面「单元里的活 key 数」在 G-宽 下会多算（同组旧版本照样供出同一批 key），按字面是作废；执行员的读法就是「先手算」本身。局限：手算函数与扫描函数共用「同组旧版本算不算供出者」这个前提，PC2 管不了读法规则本身错。
- **C17　G-紧 的盘按「活槽 + 这次换下的槽」峰值的 1.25 倍定：这个读法本身站得住，但执行员替它写的理由站不住，实验页还留着。** 理由句「推的：51 段开头几次发布就记装不下，Q8 一格都判不了」被他自己跑的 102 行 `tight_literal_reading` 推翻：入库产物里 `grep -c 'name=tight_literal_reading'` 输出 `102`，其中 `grep 'name=tight_literal_reading' <产物> | grep -c 'does_not_fit_at=none'` 输出 `50`；按段数（两条写路径都放得下才算放得下）：F1 14 / 14、F3 3 / 5、F8 8 / 32，合计 25 段（我用 awk 按段分组数的）。

### K4.2 C17：要不要按字面读法重判 Q8

要，而且我在副本上做了（`E177_TIGHT=literal`，G-紧 的盘按 G-宽 下只数活槽的峰值 × 1.25）：

```
Q8 R-S 判定行                      基线（活槽+换下）    字面（只数活槽）
judged / excluded 段数              51 / 0              25 / 26
inode（臂 N）                       resurrected r3      resurrected r3
inode（臂 B）                       zero_but_missed_live r0/m1   zero_but_missed_live r0/m1
extent（臂 B）                      resurrected r299/m300         resurrected r299/m300
accounting（两臂）                  measured_zero r0    measured_zero r0
```

12 行 R-S 判定逐行状态、r、m 都相同（完整 12 行在 `research/prompts/m3-leaf-death-r1-sonnet-model/expected/summary.txt` 的「Q8」一节）。字面读法只留下 25 段（F1 全 14 段、F3 3 段、F8 8 段，上面 K4.1 数过），另 26 段两条写路径都在字面盘上装不下，按登记的排除处理。所以：Q8 的结论（没有一棵树随几何翻面）在字面读法下仍成立，**不需要为它重跑 E177**；要改的是 E177 页与报告里 C17 的理由句和「Q8 一格都判不了」，并把「字面读法下 26 段装不下」写进去。副本上的数不进 kb；主 agent 若要把这条写进实验页，要在入库装置上加同样一个开关重跑（三行改动）。

### K4.3 实验页「影响的决策」四行

原行（`.claude/kb/experiments/177-逐树不写墓碑的复活数.md:307-310`）：D18 已定项 8、10、24 与 D8 已定项 6 都写「备料」，理由是依据段没引 E177。我建议的关系（主 agent 判）：

- **D18 已定项 8　支撑。** 臂 N（不留痕迹）下 extent 300、inode 460、分配记录 301、`deleted_inodes` 3、中央映射 301 复活（`research/results/e177-per-tree-resurrection-without-tombstones-segment-one.out:37321`「tree=extent status=resurrected r_peak=300」起），臂 P（每个 key 死亡都有记录）按装置断言 A4 恒 0（`research/results/e177-per-tree-resurrection-without-tombstones-segment-one.out:37496`「assertion=A4_death_records_zero judged=1560234 violated=0」）。这正是 `.claude/kb/decisions/18-块里携带什么信息.md:204` 那句「没有任何输入」的量。
- **D18 已定项 10　支撑区间粒度那一半。** 文件变短（Q11_OU）臂 B 下 extent 299 复活；副本里按已定项 10 的粒度加一条区间墓碑后 extent 299 → 0（K2.2）。
- **D18 已定项 24　不推翻，加一笔：「第一版只 inode 对象写」对 C109 不够。** 已定项 24 本身只定第一版的范围，没说够不够；不够是 C109 T1 的事（K2.3 的甲）。
- **D8 已定项 6　不整条推翻；推翻的是「类型 2 的记录死亡不写墓碑，死亡由现行叶版本体现」这句话字面上的完整性，设计（现行叶版本 + 退役记录）经量支撑。** 依据：F1 打中的是臂 N 的 R-S，而登记 F2 条款就预见了这一点——`research/prompts/e177-preregistration.md:484`：「R-S 漏了 kb 里已定、第一版就该生效的择现行要素（退役记录、清扫门、祖先表）：R-S 下的「复活」可能是读法缺件造成的，不是「不写墓碑」造成的。那几格写「在缺 X 的 R-S 下复活」，不写成那棵树的性质」。执行员按 C10（`grep -rn '退役记录\|清扫门\|祖先表' .claude/kb/decisions/ .claude/kb/invariants.md | grep '第一版'`）字面判 F2「没打中」，因为退役记录的条文（`.claude/kb/decisions/18-块里携带什么信息.md:300`）里没有「第一版」三个字；按实质，删空摘片在第一版单头、无崩溃的干净历史里就发生（`inode_tree.rs:277` 自称「左吸收右的退化形」），退役记录在那里生效，R-S 不读它，F2 该判打中（登记 A9 自己写着「R-S 不读退役记录」，`research/prompts/e177-preregistration.md:449`）。F1 与 F2 都中时，登记要求的写法是「在缺退役记录的 R-S 下复活」，不是 D8 已定项 6 在 R-S 下不成立。改 D8 已定项 6 原文的，写回之前交用户（正文第五节）。

## 补遗与判定一览

### K1 补遗（K1 要求里漏写的几项）

- 候选三、四要不要改条文：候选三改 D8 已定项 6 `.claude/kb/decisions/08-核心索引结构.md:169` 那句、补读侧；候选四要改 C113 定案 P3（择新键取最大之后加一步「组内记录全被墓碑压掉的容器不当活单元」），`.claude/kb/checks-owed.md:104` C113 的定案段。候选二、候选一见上。
- 唯一的叶被删空：`.claude/kb/decisions/08-核心索引结构.md:171` 写「唯一的叶被删空时是不落盘、根变成零条目」，没提退役记录；它也是一个身份消失，F8 里 inode 1 那片就是（`ret+shr` 列 F8 的 0 是因为退役记录也退了它；`shrink` 列 F8 剩的 1 就是它）。候选一要把它算进去。
- E177 量的是 key 在不在，不量值新旧。被吸收的右半在左半里有一份现行的，右半那份是合并前的旧值；两份都读得到时取哪份是 E104 的「错版本」，`.claude/kb/experiments/104-扫描重建的现行版本判定.md:85` 的 merge 世界给的是 `no_retire` 复活 2；我没有在副本里量值（推的）。
- 扫描重建拿不拿得到墓碑与退役记录：它们都是码 3 单元，靠类身份段认领，与 inode 叶走同一条扫描路径（`crates/singlefs-checker/src/walk.rs:7385` 的 `scanned_content_units` 收码 1 与码 3 单元，先过 `:7356` 的已发布谓词），拿得到（推的，读侧没写）。
- 退役记录的回收：第一版不回收；以后要 E104 的门（`.claude/kb/experiments/104-扫描重建的现行版本判定.md:59`「| P5 退役记录的回收 |」：没有根引用该容器任一版并且过门），`crates/singlefs-core/src/instance_table_sweep.rs` 做的是实例表的清扫，不是这道门。

### 判定一览

```
格                                   判定         一句话
K1 并排一 D8 与 D18 对死亡留痕        冲突         D8:169 说类型 2 不写墓碑；D18:204 说不留痕迹就没有输入
K1 并排二 删空 / 合并 / 退役          冲突         D18:300 登记退役记录，inode_tree.rs:308 删空不写，grep 退役记录 0 条
K1 并排三 退役记录的载体              规则没说     类型 3 已登记、E104 承重，没有树收它（m3-delete-r3 把孩子里的类型 3 删了）
K1 候选一 退役记录                    推出：可行   副本量：臂 N 全部容器相关格 r = 0、m = 0，三个几何；多写 2 单元 / 次，占发布 0.16%（F1）到 2.7%（F8）
K1 候选二 零条目版本                  推出：不行   副本量：ring inode 460 → 459；零版本自己占的槽还多出一个复活
K1 候选三 墓碑                        推出：半个   crates 今天已是臂 B：inode 0 / m1；分配记录 / 映射 3；合并下盖不住
K1 候选四 只改读侧                    推出：依附   无墓碑时无输入；有墓碑时管 D1 不管 D2
K1 「退回更旧版本」                   同一个病     按身份 / 记录挂的治得了，按版本挂的治不了
K2 extent / 分配记录 / 映射           推出：派生   死亡随源单元；变短要区间墓碑，容器槽要退役记录；retire_shrink 后全 0
K2 T1 / T2 改写                       推出          甲 乙 丙三档 + T2 改成登记表规则；多写 0 / +2 单元（变短）/ +2 单元（身份消失）
K3 新判据                             推出：N1     第三句改成「左最大 key < 右分隔 key」，:947 变合法形状，其余坏镜像仍红
K3 改借法                             推出：N2 次选 右片换身份，旧判据保住，每次借 +1 退役记录
K3 借多少                             推出：均分   每对并之前借 ≤ 7 对 ≤ 116；量 7.023 / 116.723；Δ3c 0.0343 对 0.5018
K4 C1、C2、C3、C14                    一致         站得住；C2 其实是登记第一节原句
K4 C13                                一致，削弱   站得住，R-G 的门没有「拒得掉」的阳性对照
K4 C17                                冲突         读法站得住，理由句被自己的 102 行推翻；字面读法下 Q8 判定逐行相同（25 / 26 段）
K4 影响的决策                         推出          D18-8 支撑；D18-10 支撑；D18-24 不推翻；D8-6 推翻字面完整性、设计支撑
K4 F2 条款                            冲突         按实质该判打中；执行员按 grep 字面判没中
```

### 要主 agent 做、我复核不了的

- 入库装置上重做：副本上的数不进 kb。要写一份跑前登记，在 E177 装置上加四样：退役记录臂、区间墓碑臂、E176 的 M2 / M3 合并历史、G-紧字面读法，并写明这一次核实的是本腿哪几句（K1.5 候选一与二的结局、K2.2、K4.2）。
- 层 0 / E165 的崩溃状态数：「多 2 个单元、状态数 × 副本数相关因子」我没量（重型，归主 agent）。
- 复跑我的副本：`SINGLEFS_REPO=/home/fy5090/code/singlefs bash research/prompts/m3-leaf-death-r1-sonnet-model/rerun.sh <空目录> all`，编译 6G、每次运行 4G，16 次约 20 分钟（`E177_THREADS=16`）；一次前台只够跑 3 次，我按 `run` 反复调用累计跑通，没有一条命令从头到尾一次跑完。
- 我交回前跑的一致性核：两个空目录各自独立跑出的 15 份产物 sha256 逐份相同（第二次多一份 `tight_literal_none.out`）；`rerun.sh <目录> compare` 输出 `摘要与期望逐行相同（207 行）`；产物 sha256 清单在 `research/prompts/m3-leaf-death-r1-sonnet-model/expected/out-sha256.txt`。

### 没做什么

- 没读攻方腿的提示与产出，没判别的腿的格；没替主 agent 采纳或出判决。
- 没改 `crates/`、kb、门禁；没跑重型测试（checker 档、层 0、变异整表、GPU 判器）；没编译、没跑 `crates/` 里的任何代码。副本装置是 `research/` 下的独立模型，编译与运行经内存包装（编译 6G、运行 4G）。
- 没量：退役记录 / 区间墓碑的字节数与崩溃状态数；管理员回退下退役树随不随 R_old；克隆头的跨头首次 COW；清扫门；GPU 判器改第三句后的一致性；值的新旧（错版本）；N1 的 `transaction.rs` 四处调用改后写出的字节。
- 副本里区间墓碑与退役记录不占槽（像 E177 的臂 P）；它们的占槽代价是我按 `plan_tombstone_append` 的写法推的 2 个单元。零版本候选的释放时刻（同一发布内）是我定的最宽的一种，不是唯一的。
- E176 的数只用到 `arm_run` 与 `window_rewrites` 两类行，没复核它们与 `crates/` 的对拍（对拍在 E176 页「对拍」一节）。

### 草稿产物与清理

- 草稿目录 `~/.local/share/singlefs/work/m3-leaf-death-r1-sonnet/`。保留：`tmp.B6klfNHLmg/runs/`（最终一次独立重跑的 16 份产物与错误输出，共 144 MB，每份约 10 MB，没入库：太大、由 `rerun.sh` 可复现，sha256 清单在模型目录 `expected/out-sha256.txt`）、`summary-final.txt`、`sha*.txt` 与几份草稿脚本。
- 已删（自己建的编译目录、仓副本与第一遍产物）：`~/.local/share/singlefs/work/m3-leaf-death-r1-sonnet/model-work/`（E177 所在 `e7-index-bench` 的拷贝加 cargo 目标目录）、`…/runs/`（第一遍 13 份产物，已被最终一遍取代）、`…/tmp.pLfro02LNu/`、`…/tmp.8x2qarPwrF/`（前两次独立重跑的目录）、`…/tmp.B6klfNHLmg/build/`（最终一遍的编译目录与仓拷贝）、`e177_original.rs` 与 `e177_patched1.rs`、`e177_patched2.rs`、`build-1.log`。
