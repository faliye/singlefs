# m2-treesplit-r1 云端攻方（Opus）：T1、T4、T6

2026-09-24，时刻都是 UTC。副本 `/tmp/claude-1000/m2-treesplit-opus/repo`，06:00:58 从主工作区拷（`rsync -a --exclude target --exclude .git`），拷完对开工快照 `research/prompts/m2-treesplit-r1-snapshot/sha256sums.txt` 的 61 个文件逐个 `sha256sum -c`，没有一个不符（命令只打印不符的行，输出为空）。**这里所有数都是副本上量的，不是入库装置上的数。**

## 复跑

```
bash research/prompts/m2-treesplit-r1-opus-model/run.sh <副本根> <输出目录>
```

副本根要是一份仓副本，并且先把 `treesplit_opus_prototype.rs`、`treesplit_opus_costs.rs` 拷进 `<副本根>/crates/singlefs-harness/tests/`。脚本依次跑 selftest、costs、t6、t1、targeted 五段，每段写一个 `<段名>.out`。t1 只扫 PA/PB/PC/PE 四个前缀；PD（树高 3）只在 `out/t1-one-config-all-five-prefixes.out` 那一格跑过（那一行是被 `cut -c1-600` 截断以后才存的）。机器负载 load 230–340（32 核），t1 那一段挂钟 653 秒。

模型目录 `research/prompts/m2-treesplit-r1-opus-model/` 的 sha256：

```
ac3465a30a9810ce0faf62d99bb541707e84f6ec987946be36fc7e228c1b2c0b  treesplit_opus_prototype.rs
b122f65963c5df8353121eedefd92ea27050e58cace50d3314a75ae8015413eb  treesplit_opus_costs.rs
827dc125aeebd2dc5e080e3800eec7f49f7e3c841a5693aa53464e3e6fda7c6b  run.sh
74f85627727c99baa0ebfd9be11ee5109322eb7d6d68a3cb00408a10101c2e59  out/costs.out
c4841919771f4f910c0f655e1802a7ae603ca5c41733b9573d87afe6e98d5306  out/selftest.out
5b17453cd876accdcec0a4e4ca00e6818aa2b34da724bb7348f625755571d80b  out/t1-one-config-all-five-prefixes.out
c2cb3deb97053c123eb8f186d19e742cdb6860a9b0334201e980d77302c0e3b8  out/t1.out
59c3e264fd13a1721c06630233778981dcf43def1d998cbc53e760a1bd098c41  out/t6.out
cb20855650a4eef0ee502378f34d7eb78916ae6881fd9598f7f5754a2876fbb2  out/targeted.out
```

⚠️ selftest / costs / t6 / t1 四段是原型文件后面还没追加两个定向用例（`borrow_from_the_right_under_the_creation_min_separator`、`tail_split_root_growth_is_enumerated`）时跑的；那两个用例只追加在文件末尾，前面的代码一个字节没改。targeted.out 是追加之后跑的。

## 原型是什么（写在前面，判读每个数都要用到）

- 一棵码 2 树（树 ID 11、key 宽 8）。节点字节用实现的 `singlefs_core::unit::build_index_node` 写、`parse_index_node` 解，所以头里的树 ID、层级、key 宽、key 区间、诞生代号、实例代号、出生序号就是实现的偏移。条目格式（叶 16 字节、内部 32 字节 = 分隔 key 8 + 简化指针）、journal 记录、根记录是原型自己的简化格式。
- 节点容量用测试开关压到每节点 4 条，点名容量在 T6 那一段压到每条记录 3 项，这样分裂、根分裂、合并、「末条装不下」在两位数写的负载上出得来。
- 一次发布的写序照实现（`crates/singlefs-core/src/transaction.rs` 第 3824 行的注释 `// 持久顺序（D16（发布语义） 已定项 7）：这次重写的单元 → 屏障 → journal 记录 → 屏障 → 根槽 FUA → 系统配置槽轮换。`）：单元（两盘各一写）→ 屏障 → 记录（两盘各一写）→ 屏障 → 根槽 FUA。系统配置槽不建模。
- 崩溃状态照 D13（验证路线） 已定项 4（`.claude/kb/decisions/13-验证路线.md` 第 71 行）：屏障切段、FUA 自成段尾、当前段任意子集、没持久的位置放旧字节。每段历史的状态数都断言等于实现的闭式 `closed_form_state_count`（`crates/singlefs-harness/src/crash.rs`）。
- 每个状态跑三样：恢复（择最新根；从根覆盖的最后一条之后接 jsn 严格连续、txg 大于根的记录；按发布分组，本次发布内序号 1 打头；按候选的规则认末条；施加前逐项核点名单元的 CRC）；checker（从恢复出的根往下走，只用指针与子节点码 2 头里的字段判：树 ID、层级 = 父层级 − 1、诞生代号 = 指针里的、≤ 根 txg、条目 key 严格递增、容量、非根叶不空、头里的 key 区间 = 子树覆盖区间（D18 已定项 2）、分隔 key ≤ 孩子区间下界且 > 左邻孩子区间上界（I-9.12 泛化到任意层））；oracle（恢复出的内容 = 那一版发布时的内容；恢复出的版本不早于最新已持久的根；发布边界：某次发布的根没落而它被施加了，而它的末条一份都没落 ⇒ 记 `cut_publish_applied`）。另按分隔 key 走一遍读者的查找，找不到任何一个 key 记 `lookup_miss`。
- 用户动作不写死：每个前缀之后，第一步从菜单（前缀里每个 key 之间 / 之下 / 之上各插一个、删每一个 key、两插一批、两删一批）任取，第二步从单操作菜单任取，两步全组合。只固定前缀（PA：升序插 8 个；PB：升序插 20 个；PC：PB 之后删 7 个；PD：升序插 64 个、树高 3；PE：根的孩子接近满）。

## 各格判定一览

| 格 | 候选 | 判定 | 依据（产物） |
|---|---|---|---|
| T1 | 分隔 key 取「孩子创建时的最小 key、之后不变」（D8 已定项 6 inode 树那一句搬到别的树） | **打中**：两种形状都会让读者按分隔 key 找不到一条在树里的 key，checker 在恢复后判红 | t1.out 全部 24 格 CreationMin 的 `check_red` > 0（34952–562983），同样的历史换成 Maintained 全是 0；targeted.out `BORROW` 两行 |
| T1 | D8 已定项 6「合并：只许左吸收右 / 左从右借」里的「左从右借」 | **打中**（对 inode 树本身也成立）：借过来的那条 key ≥ 右孩子的分隔 key，I-9.4「左容器的最大 key < 右容器号」当场假 | targeted.out `BORROW sep=CreationMin … lookup_misses=1`（推到 inode 树那一步是推的） |
| T1 | 写序：整条路径一次发布 COW（OnePublish）/ 分裂单开一段（SplitOwnBarrier）/ 分裂单开一次发布（SplitOwnPublish） | 崩溃历史上**没打中任何一条**：三条都是 recover_fail = root_lost = content_mismatch = 0 | t1.out 24 格 Maintained；targeted.out `TAILROOT` 三行 |
| T1 | SplitOwnPublish | **打中**（不是崩溃，是回退窗口）：只改结构的发布被实现的「非空」判定算成改了用户可见状态，挤掉「最近 4 个可退到的不同状态」 | costs.out `WINDOW` 三行：3454 / 33859 段历史窗口里不同内容不足 4 个，OnePublish 同样的历史是 120（装置造成的底数，见 T1 第 3 节） |
| T1 | 收缩：只摘空节点 / 低于 2 条合并（借或不借） | 崩溃历史上没打中（Maintained 下）；只摘空节点那几格树高从来不降（`collapse_root:false` 时 `ev_height_down=0`） | t1.out |
| T1 | 分裂点：中间切 / 追加时末尾切 | 崩溃历史上没打中；末尾切在两步菜单里一次根分裂都没出现（`ev_height_up=0`），用 targeted.out 补了一段 | t1.out；targeted.out `TAILROOT` |
| T4 | 层 0 状态数 | 整条路径一次发布 COW：每多一个单元，那一段的状态数乘 4；第一个事务那条流上一次发布同时让三棵根兼叶的树分裂，闭式 262168 → 1073741848 | costs.out `VARIANT` / `SEGMENT` |
| T4 | 今天的实现 | 今天已有的那一种分裂（inode 容器末尾分裂）五条层 0 流里没有一条罩到 | 本报告 T4 第 2 节的 grep |
| T6 | 末条再跨记录、按今天的字面认末条（SpreadLiteral） | **打中**：记录段里只落了共享点名的前一条，恢复把它认成末条、施加半次发布；再加一个「只在后一条点名的单元两份都读不出」就读到坏树 | t6.out `cut_publish_applied=8826`、`overlay_red=53520` |
| T6 | 末条再跨记录、末条带标志位（SpreadFlag） | 没打中 | t6.out 同一批历史 0 / 0 |
| T6 | 维持拒绝（Refuse） | **打中**（不是崩溃，是拒不拒取决于池的年龄）：同一个「覆盖写 40 个数据单元」，新池只碰 7 片分配记录叶，老化的池至少碰 70 片 > 67 | costs.out `AGED` 五行 |
| T6 | 发布拆成几次（CutPublish） | 修不到 Refuse 被打中的那几段历史：拒掉的历史数相同（2875 / 8684），因为装不下的是单个操作 | t6.out |

## T1 分裂与收缩规则

### 1. 候选，与各自靠块头哪几样身份字段

四棵树共用一套（D8（核心索引结构） 已定项 14，`.claude/kb/decisions/08-核心索引结构.md` 第 362 行「**一套 btree 实现 + 多个独立 keyspace + 两个前端。不做异构结构。**」）。原型里每个候选都是同一套代码换一个参数：

| 轴 | 候选 | 靠块头的什么 |
|---|---|---|
| 分裂点 | Middle：从中间切，左右两半都新写 | key 区间：左右两半各自的头自证覆盖段，恢复与 checker 不用父节点就知道这片叶该装哪些 key；层级：新根比旧根高一层，孩子的层级 = 父 − 1 在每一步可判 |
| 分裂点 | TailWhenAppend：新 key 比叶里每一条都大时末尾切，左半一个字节不重写（inode 容器今天的写法外推） | 同上，外加诞生代号：左半没重写，诞生代号比父小，「子的诞生代号 ≤ 根的 txg」照样成立 |
| 分隔 key | CreationMin：孩子创建时的最小 key、之后不许变（D8 已定项 6 原句） | 不靠头；靠「key 单调」这个 inode 树才有的前提 |
| 分隔 key | Maintained：插到最左分隔 key 之下时把它压低；借一条之后把右孩子的分隔 key 改成右孩子新的最小 key | 靠头里的 key 区间当判据：分隔 key ≤ 孩子头的 min_key、> 左邻孩子头的 max_key，逐层可判 |
| 收缩 | DropEmpty：不合并，只摘空节点；`collapse_root` 决定根只剩一个孩子时降不降高 | 层级：降高时孩子直接当根，不重写，它的层级字段就是新树高 |
| 收缩 | Merge：低于 2 条（容量 4 的一半）就左吸收右，合不下时借一条（`borrow`） | key 区间：合并后左节点头的区间变宽、被吸收的右节点整片退役 |
| 写序 | OnePublish：整条路径在一次发布里 COW 重写，分裂出来的节点与别的单元同一段 | 写序、诞生代号、出生序号：同一次发布里切出来的左右两半 (树, 层级, 诞生代号, 写序) 四样相同，分开它们的是出生序号与 key 区间 |
| 写序 | SplitOwnBarrier：同一次发布里，分裂新建的单元先写、屏障、再写其余单元 | 同上 |
| 写序 | SplitOwnPublish：先发一次只改结构的发布，把路径上满的节点预先切开，再发内容 | 诞生代号：结构发布与内容发布各一个 txg |

### 2. 打中一：分隔 key 照 inode 树取「创建时的最小 key、不许跟着变」

D8 已定项 6（`.claude/kb/decisions/08-核心索引结构.md` 第 162 行）原句：「**分隔 key** = 该孩子创建时的最小 key，条目按分隔 key 单调递增，查找取最后一个分隔 key ≤ 目标 key 的条目。」inode 树成立，是因为同一段第 169 行「inode 号在每一条时间线上单调……⇒ 插入永远落在最右的叶」。分配记录（key = (盘, 槽)）、映射（key 首段是类标签与出生树）、extent（key 首段是 locality）三棵树的插入不单调，这个前提没有。

两种形状，都在恢复后的镜像上判红，三种写序一样红：

- **插到最左分隔 key 之下**：PA 之后插 1（或 5）。最左孩子的分隔 key 10 不许改，1 进了最左叶，按「最后一个分隔 key ≤ 1」一条都找不到。t1.out 的例子原样：`EXAMPLE T1/Middle/CreationMin/DropEmpty{collapse_root:false}/OnePublish CHECK_RED: 槽 116 分隔 key 10 > 孩子区间下界 5 (recovered txg 9)`。
- **左从右借**（只删不插，targeted.out）：TailWhenAppend、升序插 10..80（两片叶 [10,20,30,40] 与 [50,60,70,80]），第一步删 10、20，第二步删 30。左叶剩 [40]，低于 2 条、合不下（1 + 4 > 4），从右边借 50。CreationMin 下右孩子的分隔 key 仍是 50，50 住在左叶里：

```
BORROW sep=CreationMin tree=L1<10:[40, 50] 50:[60, 70, 80]> checker=red: 槽 117 分隔 key 50 ≤ 左邻孩子区间上界 50 lookup_misses=1 keys=[40, 50, 60, 70, 80]
BORROW sep=Maintained tree=L1<10:[40, 50] 60:[60, 70, 80]> checker=green lookup_misses=0 keys=[40, 50, 60, 70, 80]
```

同一段历史枚举全部 99 个崩溃状态：CreationMin `check_red=4`（根已落的那几个状态），Maintained 0（targeted.out `BORROW_ENUM` 两行）。

**推到 inode 树本身（推的，没在实现上跑）**：inode 树的右半「容器号」就是它的分隔 key（`crates/singlefs-core/src/inode_tree.rs` 的 `separator_key`），合并纪律写的是「合并：只许**左吸收右 / 左从右借**（右吸收左或右从左借会破「记录号 ≥ 容器号」），保留左半身份」（08 第 169 行）。左从右借一条 r：r ≥ 右容器号（I-9.4 前半），借走以后左容器的最大 key = r ≥ 右容器号，而 I-9.4（`.claude/kb/invariants.md` 第 271 行）写「沿叶序容器号严格递增，且左容器的最大 key < 右容器号」——**任何一次「左从右借」都让 I-9.4 当场为假**，除非右容器换号，而换号就是换身份。今天这一版没有删除（`inode_tree.rs` 模块注释「合并不在这个模块里」），所以还没有历史走到这里。

四句：
1. 分不分辨臂：分辨分隔 key 那一轴（CreationMin 24 格全红、Maintained 24 格全 0），不分辨写序那一轴（三种写序同样红）。
2. 系统当时看不看得到判别它的东西：看得到。借的那一刻写者手里两个孩子都在；读者一侧，孩子头里的 key 区间（D18 已定项 2，第 50 行「**区间取子树覆盖区间**」）让 checker 不读叶内容就判出「分隔 key ≤ 左邻孩子的 max_key」。
3. 满足的是哪一句：08 第 162 行「分隔 key = 该孩子创建时的最小 key」与第 169 行「只许左吸收右 / 左从右借」两句合用；I-9.4 的「左容器的最大 key < 右容器号」分句。
4. 改法在打中的格上还中不中：Maintained 在全部 24 格 0 红（量过）；只删掉「左从右借」、分隔 key 仍不许变，能挡借那一形，挡不住「插到最左分隔 key 之下」那一形（推的：那一形不经过借）；另一个改法「最左那一条的分隔 key 读成 −∞」挡插入那一形、挡不住借那一形（推的）。

### 3. 打中二：分裂单开一次发布，挤掉回退窗口里可退到的不同状态

D16（发布语义） 已定项 1 第 29 行定「回退候选集保留最近 4 个可退到的不同状态——只数改过用户可见状态的根，推空与抬 F 产生的空发布根不算」，第 43 行定「非空」从盘上怎么认：比树表里 inode 树、extent 树的根指针。实现就是 `crates/singlefs-core/src/mount.rs` 第 728 行的 `user_visible_trees_changed`，第 732 行 `    root_pointers.inode_tree != previous_valid_root_pointers.inode_tree`。一次只改结构的发布（extent 树叶分裂、inode 树根分裂）换了根指针、内容一个字节没变，被这个函数判成非空。

costs.out 用实现的这个函数判每条根（把原型的根指针字节喂给 `extent_tree` 那一格），前缀 PA、PE，之后三步用户动作全组合，只数整条历史里内容不同的版本 ≥ 5 的那些：

```
WINDOW step=OnePublish histories=33859 window_short_of_4_distinct=120 min_distinct=3 example=prefix=PA suffix=[[Del(80)], [Ins(1)], [Del(80)]] window_roots=5 restructure_in_window=0 distinct=3
WINDOW step=SplitOwnBarrier histories=33859 window_short_of_4_distinct=120 min_distinct=3 example=prefix=PA suffix=[[Del(80)], [Ins(1)], [Del(80)]] window_roots=5 restructure_in_window=0 distinct=3
WINDOW step=SplitOwnPublish histories=33859 window_short_of_4_distinct=3454 min_distinct=2 example=prefix=PA suffix=[[Ins(1)], [Ins(1)], [Ins(55)]] window_roots=4 restructure_in_window=1 distinct=3
```

OnePublish 那 120 段是装置造的底数：筛选条件数的是整条历史（含前缀）的不同内容，窗口里却可能落进同一个 key 删两次那种什么都没改的发布。三条臂用的是同一批 33859 段历史、同一个筛选条件，差出来的是 SplitOwnPublish 多的那些：最坏窗口里只剩 2 个不同内容。

四句：
1. 分不分辨臂：分辨写序那一轴，只打中 SplitOwnPublish；OnePublish 与 SplitOwnBarrier 逐格相同。
2. 系统看不看得到：看不到。「非空」只比根指针字节，结构发布与内容发布在这一比上完全一样；要分开就得读两棵树比内容，或者让结构发布在盘上带一个标记。按字面这是「判别子观测不到」那一形（evidence-discipline「判据自己也会写错」表第二行）：不改 D16 已定项 1 的「非空」定义，SplitOwnPublish 做不到只数用户可见的改动。
3. 满足的是哪一句：第 29 行「只数改过用户可见状态的根」与第 43 行「比的是树表里这两棵树的条目」两句，在结构发布上一个说不算、一个说算。
4. 改法：根记录或树表条目加一位「只改结构」、判非空时跳过它（推的，要改格式，改完是新一轮）；或者不单开发布（OnePublish / SplitOwnBarrier，量过：与 OnePublish 同 120）。

### 4. 写序与收缩在崩溃历史上：没打中，代价各是多少

t1.out，分隔 key 取 Maintained、分裂点从中间切的 12 格（8684 段历史，每段两步用户动作全组合；recover_fail、root_lost、content_mismatch、cut_publish_applied、lookup_miss 在这 12 格里全是 0，check_red 也是 0）：

| 收缩 | 写序 | 层 0 状态数 | 发布次数（其中只改结构的） | 写出的单元数 | 分裂出的单元数 |
|---|---|---|---|---|---|
| DropEmpty（不降高） | OnePublish | 2746620 | 17368（0） | 52539 | 4747 |
| DropEmpty（不降高） | SplitOwnBarrier | 1219536 | 17368（0） | 52539 | 4747 |
| DropEmpty（不降高） | SplitOwnPublish | 10718612 | 19164（1796） | 73187 | 3830 |
| Merge（不借） | OnePublish | 2737332 | 17368（0） | 52662 | 4863 |
| Merge（不借） | SplitOwnBarrier | 1192356 | 17368（0） | 52662 | 4863 |
| Merge（不借） | SplitOwnPublish | 7549057 | 19391（2023） | 66480 | 4602 |
| Merge（借） | OnePublish | 2799900 | 17368（0） | 53076 | 4893 |
| Merge（借） | SplitOwnBarrier | 1245474 | 17368（0） | 53076 | 4893 |
| Merge（借） | SplitOwnPublish | 7673869 | 19379（2011） | 66859 | 4560 |

（DropEmpty 降高那三格与不降高逐项同数，只差 SplitOwnPublish 那一格：少 6 个状态、少 2 个单元。）

- 三种写序在同一批历史上崩溃结果相同，理由在写序本身：根槽 FUA 前面有两道屏障，根落了 ⇒ 单元与记录全落了；根没落 ⇒ 要么施加记录（点名单元逐项核过，全在），要么回到上一版。分裂出来的单元与别的单元走同一条路，没有新的崩溃结局。
- SplitOwnBarrier 买不到任何安全性（与 OnePublish 零对零），却让状态数少一半多（一段 2^(a+b) 拆成 2^a + 2^b）；代价是分裂那几次发布每次多一道屏障（每块盘一次 FLUSH）。
- SplitOwnPublish 在同一批用户动作上多发 11.6% 的发布（19379 / 17368）、多写 26.0% 的单元（66859 / 53076），层 0 状态数 2.74 倍；外加第 3 节那一格。
- 分裂点：TailWhenAppend 在不是追加的历史上切出一条一条的叶，Maintained / DropEmpty 下分裂次数 5427、Middle 2360，单元数 54861 对 52539；它省的「左半不重写」只在追加负载上兑现。它的根分裂两步菜单里一次都没出现，补了一段（targeted.out，升序插 52 个 key（10..520）之后再插一个让树高 2 → 3，再删一个）：

```
TAILROOT step=OnePublish prefix_len=52 CFG tail-root histories=1 states=4359 closed_form=4359 recover_fail=0 check_red=0 content_mismatch=0 root_lost=0 cut_publish_applied=0 unverified_applied=0 lookup_miss=0 refused_publishes=0 refused_histories=0 publishes=2 restructure_publishes=0 units=10 split_units=4 ev_split=1 ev_height_up=1 ev_height_down=0 ev_node_removed=0 max_segment_writes=12 overlay_states=0 overlay_red=
```

  （行尾是被我抽行时截的，原样在 targeted.out。）SplitOwnBarrier 同一段 534 个状态、SplitOwnPublish 16714 个（它的前缀长 16，presplit 从中间切，树高在更早就涨了），三格同样零违例。
- 收缩只摘空节点、不降高（`collapse_root:false`）那三格 `ev_height_down=0`：树高只涨不降。它不在崩溃历史上出错，出在 T5 的 ckpt_cost（「每棵记录树当前的高」）只增不减——这一格归正推腿与本地攻方，我只报这个观测。

## T4 写序与层 0 崩溃点

### 1. 每一种新写序落在哪一段、状态数涨多少（真实现的闭式）

costs.out 拿入库用例断言的真段序列算（`crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs` 第 390 行 `        vec![2, 2, 1, 2, 2, 1, 18, 3, 2],`；`crates/singlefs-harness/tests/second_transaction_parallel_line_one_layer0.rs` 第 154 行 `        vec![2, 2, 1, 2, 2, 1, 18, 2, 1, 20, 4, 1, 22, 6, 1, 2],`），在第一个事务那次发布上叠一次分裂：

```
REAL first_transaction sizes=[2, 2, 1, 2, 2, 1, 18, 3, 2] closed_form=262168
REAL parallel_line_one sizes=[2, 2, 1, 2, 2, 1, 18, 2, 1, 20, 4, 1, 22, 6, 1, 2] closed_form=5505123
VARIANT A_root_leaf_split_one_tree sizes=[2, 2, 1, 2, 2, 1, 22, 3, 2] closed_form=4194328 ratio_to_first=16.00
VARIANT A_root_leaf_split_two_trees sizes=[2, 2, 1, 2, 2, 1, 26, 3, 2] closed_form=67108888 ratio_to_first=255.98
VARIANT A_root_leaf_split_three_trees sizes=[2, 2, 1, 2, 2, 1, 30, 3, 2] closed_form=1073741848 ratio_to_first=4095.63
VARIANT B2_root_leaf_split_one_tree sizes=[2, 2, 1, 2, 2, 1, 4, 18, 3, 2] closed_form=262183 ratio_to_first=1.00
VARIANT B2_root_leaf_split_three_trees sizes=[2, 2, 1, 2, 2, 1, 12, 18, 3, 2] closed_form=266263 ratio_to_first=1.02
VARIANT B1_root_leaf_split_one_tree sizes=[2, 2, 1, 2, 2, 1, 16, 2, 1, 18, 2, 1, 2] closed_form=327704 ratio_to_first=1.25
```

段序列怎么改是**推的**（A：根兼叶换成左叶 + 右叶 + 新根，多 2 个单元 = 同一段多 4 写；B2：分裂新建的两个单元自成一段；B1：结构发布 7 个单元 14 写 + 上一次系统配置 2 写一段、记录 2、根 1，之后原来那次）；闭式是实现的函数量的。

| 候选 | 一次叶分裂多写 | 一次根分裂（树高 +1）多写 | 多几段 | 层 0 状态数 |
|---|---|---|---|---|
| OnePublish | 1 个单元（右半；左半本来就要 COW）+ 父节点在路径上本来就重写 | 2 个单元（右半 + 新根） | 0 | 那一段 ×4^u（u = 多的单元数）；第一个事务上三棵树同时根分裂：×4096，10.7 亿 |
| SplitOwnBarrier | 同上 | 同上 | 分裂的那次发布 +1 | 加法：+2^(2u) |
| SplitOwnPublish | 结构发布：左 + 右 + 路径上的祖先 + 四个固定点单元（分配记录、记账、映射、树表）+ 一条记录 + 一个根 | 同，多一个新根 | +3（单元、记录、根各一段）/ 每次结构发布 | 原型 ×2.74；第一个事务上 ×1.25 |
| 收缩（合并 / 摘空 / 降高） | 合并：左半重写、右半退役（少写 1）；借：两个都重写 | 降高：0 个新单元，旧根退役 | 0 | 不涨或降 |

「分裂出来的那几写」在 OnePublish 下没有独立的段，所以 T4 问的三种流（叶分裂、根分裂、收缩）在层 0 眼里只是「单元段更长」；要罩住它们，要的不是新的崩溃点类型，是**让那次分裂的发布落在被枚举的流里**。代价按段长指数涨：D13 已定项 9（第 170 行）`| 层 0 冒烟 | 固定种子的最小复现负载（写请求数两位数） | **不抽样，全量** | 任何触碰写路径的提交 |`，而今天一次发布的单元段已经是 18–22 写。

### 2. 今天已有的那一种分裂没有层 0 流

实现里已经在分裂的只有 inode 树的叶容器（末尾分裂，`crates/singlefs-core/src/inode_tree.rs` 的 `write_records_into_leaf_containers`）。副本上现查：

```
$ grep -ln 'publish_new_inodes' crates/singlefs-harness/tests/*.rs crates/singlefs-harness/src/*.rs
crates/singlefs-harness/tests/second_transaction_parallel_line_three_many_inodes.rs
crates/singlefs-harness/src/model_comparison.rs
$ grep -ln 'enumerate_layer0' crates/singlefs-harness/tests/*.rs | wc -l
5
```

调 `enumerate_layer0` 的 5 个用例文件没有一个调 `publish_new_inodes`；那份 many_inodes 用例走的是「恢复加 oracle」，不枚举崩溃状态（它自己的模块注释写「这一份是 C116 的「恢复加 oracle」那一路」）。按用户 2026-09-24「分裂的每一种新写序都要进层 0」，这一种今天就欠着。

### 3. 层 0 要加的流（候选，与跑前条款第四条对表）

写法：分裂之前的那一大截（装满一片叶要 812 / 477 / 294 / 144 条）放进 `memory_pool_after_mkfs` 那样的起点镜像、不枚举，只把触发分裂的那一次（与它之后的一次）录进被枚举的流——原型里的 `prefix` / `suffix` 就是这个切法，段长因此只随「这一次发布写几个单元」涨，不随树里有多少条涨。另一条路是测试开关把节点容量压小（原型的 `leaf_cap = 4`）；它不改盘上格式，但走的是「只供测试强制进入」那一档，要照 fs-design 五条硬要求让它在运行时看得出来。

| 流 | 罩什么 | OnePublish | SplitOwnBarrier | SplitOwnPublish | 原型里出现过（t1.out / targeted.out，Maintained） |
|---|---|---|---|---|---|
| L1 叶分裂 | 左右两半 + 父 | 单元段 | 分裂段 + 其余段 | 结构发布的三段 | `ev_split` 2360–5480 次 |
| L2 根分裂、树高 +1 | 新根，旧根变成左半 | 同上 | 同上 | 同上 | Middle `ev_height_up` 922–932；TailWhenAppend 只在 targeted.out 那 1 段 |
| L3 两层连着分裂 | 叶、内部、根三层都切 | 同上 | 同上 | 同上 | PB/PE 前缀之后的插入（没单独计数，下限见限度一节） |
| L4 合并（左吸收右） | 右半退役、左半重写 | 单元段 | 不适用（不新建单元） | 不适用 | Merge 各格 `ev_node_removed` 1401–3607 |
| L5 借一条 | 两个孩子都重写、右孩子分隔 key 改 | 单元段 | 同左 | 同左 | Merge borrow 各格 |
| L6 降高 | 孩子当根，不重写 | 只有根与固定点 | 同左 | 同左 | `ev_height_down` 58–1902（DropEmpty 不降高那几格 0） |
| L7 inode 容器末尾分裂 | 今天实现里就有 | — | — | — | 原型没罩（格式不同），本节第 2 小节 |
| L8 一次发布多条记录、共享点名跨记录 | 只在 T6 取「末条再跨记录」时有 | 记录段 | 同左 | 同左 | t6.out SpreadLiteral / SpreadFlag |

跑前条款第四条：「T4 给出的层 0 流少罩了 T1 候选的任何一种新写序 ⇒ 那个候选不算有崩溃点覆盖」。按上表：OnePublish 与 SplitOwnBarrier 的新写序都在 L1–L6 里出现过；SplitOwnPublish 多出来的「结构发布」一种在原型里出现 1796–2040 次（`restructure_publishes`）；TailWhenAppend 的根分裂只有一段定向历史（4359 个状态），它是覆盖最薄的一格。**这张表只证原型的流罩到了原型的写序；实现里的流还一条都没有。**

## T6 末条装不下

### 1. 候选与原型里的样子

今天的约定：D23（journal 的角色与格式） 已定项 17（`.claude/kb/decisions/23-journal的角色与格式.md` 第 442 行）「这次发布共享的提交内生块（extent 树节点、inode 叶容器与根、分配记录树、记账树、中央映射树、树表单元）只在最后一条记录里点名」；已定项 14 第六条（第 352 行）「**发布边界怎么认**：一次发布的末条 = 点名了这次发布共享的提交内生块的那一条」。实现在装不下时拒（`crates/singlefs-core/src/transaction.rs` 第 2607 行 `        return Err(PublishError::MoreNamedUnitsThanOneJournalRecordHolds {`，容量常量是 67，`crates/singlefs-format/src/lib.rs` 第 324 行 `        assert_eq!(JOURNAL_NAMED_ENTRIES_PER_RECORD, 67, "一条记录装几个点名项");`；正文第二节写 66，副本上的常量是 67，头 307 改 311 之后按 ⌊(4096 − 311) ÷ 56⌋ 算仍是 67，66 从哪来我没查到）。

| 候选 | 原型里 | 靠块头 / 记录头的什么 |
|---|---|---|
| Refuse 维持拒绝 | 共享单元 > 3 就整次拒 | 不靠 |
| SpreadLiteral 末条再跨记录、按今天的字面认末条 | 共享单元按 3 个一组摊到最后几条记录；恢复认「点名了共享单元的那一条」 | 本次发布内序号（1 打头才接） |
| SpreadFlag 末条再跨记录、末条带标志位 | 同上，最后一条记录带标志；恢复认标志 | 本次发布内序号 + 一个新标志位 |
| CutPublish 发布拆成几次 | 按操作切，每次共享单元 ≤ 3；单个操作就装不下的拒 | 诞生代号：每次一个 txg |

### 2. 打中三：末条再跨记录、按今天的字面认末条

t6.out，同一批 8684 段历史（PA/PB/PC/PE 四个前缀、两步动作全组合）：

```
CFG T6/SpreadLiteral/named_cap=3 histories=8684 states=2814312 closed_form=2814312 recover_fail=0 check_red=0 content_mismatch=0 root_lost=0 cut_publish_applied=8826 unverified_applied=8826 lookup_miss=0 refused_publishes=0 refused_histories=0 publishes=17368 restructure_publishes=0 units=53076 split_units=4893 ev_split=2486 ev_height_up=932 ev_height_down=124 ev_node_removed=2468 max_segment_writes=10 overlay_states=17747219 overlay_red=53520
CFG T6/SpreadFlag/named_cap=3 histories=8684 states=2814312 closed_form=2814312 recover_fail=0 check_red=0 content_mismatch=0 root_lost=0 cut_publish_applied=0 unverified_applied=0 lookup_miss=0 refused_publishes=0 refused_histories=0 publishes=17368 restructure_publishes=0 units=53076 split_units=4893 ev_split=2486 ev_height_up=932 ev_height_down=124 ev_node_removed=2468 max_segment_writes=10 overlay_states=17747219 overlay_red=0
```

一段具体的历史（t6.out 第一条 EXAMPLE，原样）：前缀 PB（升序插 10..200），第一步插 1，第二步插 25。第二次发布（txg 22）写 4 个单元（槽 154–157），点名摊成两条记录：jsn 25 点名 3 个、jsn 26 点名 1 个。崩溃状态：

```
persisted=[Unit(151)@d0,Unit(151)@d1,Unit(152)@d0,Unit(152)@d1,Unit(153)@d0,Unit(153)@d1,Rec(24)@d0,Rec(24)@d1,Root(5)@d1,Unit(154)@d0,Unit(154)@d1,Unit(155)@d0,Unit(155)@d1,Unit(156)@d0,Unit(156)@d1,Unit(157)@d0,Unit(157)@d1,Rec(25)@d0]
```

jsn 25 落了一份、jsn 26 两份都没落（同一记录段里任意子集，D13 已定项 4 允许）。恢复：根是 txg 21，链接 jsn 25，它点名了共享单元 ⇒ 按字面它就是末条 ⇒ 施加 txg 22。纯崩溃下读回的内容是对的（单元段在屏障之前全落了），这与入库用例 `second_transaction_parallel_line_one_layer0.rs` 第 197 行那句同一件事：`/// 这一条 oracle 判不出来：层 0 按屏障切段，记录段里的状态上这次发布的单元全都落了，半次发布施加上去读回的也是完整的新内容。`。坏处在两处：
- 第六条的字面被违反：「前缀里一次发布的末条没到，那次发布整体不施加」，而 jsn 26 就是没到的末条（8826 个状态）。
- 只在 jsn 26 点名的那个单元没被核过就施加了（`unverified_applied=8826`）。叠一个介质故障——这次发布的某一个单元两盘都读不出、根没落——SpreadLiteral 有 53520 个状态恢复施加了那次发布、checker 读到坏指针判红；SpreadFlag 同样的 17747219 个叠加状态一个不红（它不认 jsn 25 是末条，回到 txg 21）。

四句：
1. 分不分辨臂：分辨 SpreadLiteral 与 SpreadFlag（同一批状态 8826 对 0、53520 对 0）；Refuse 与 CutPublish 不写多条共享记录，这一格上没有对象。
2. 系统看不看得到：今天的记录头看不到。jsn 25 手里只有「本次发布内序号 = 1」与点名项，它不知道后面还有没有一条；记录头没有「这次发布共几条」也没有「我是末条」。所以「末条再跨记录」在今天的格式上**写不成**，要么加一个字段，要么落回字面、吃这一格。
3. 满足的是哪一句：23 第 352 行「一次发布的末条 = 点名了这次发布共享的提交内生块的那一条」与同一句的「前缀里一次发布的末条没到，那次发布整体不施加」——同一句的两个分句在跨记录时互相打架。
4. 改法在打中的格上：末条标志位（原型 SpreadFlag，量过：0 / 0）。落点两个都是推的：点名项里本来有一个 flags 字节，实现写 0、读时跳过（`crates/singlefs-core/src/journal.rs` 第 44 行 `        writer.put_u8(0);`、第 59 行 `        reader.skip(1);`），在末条的点名项上置一位不加字节，但今天的读者跳过它、老读者照样按字面认，是不是 incompat 要按 D15 判；另一个落点是记录头加「本次发布记录总数」，要改记录头宽。提交标记那一字节不能借：读者按 `let is_commit = reader.get_u8() == 1;`（journal.rs 第 194 行）读，写 2 会被旧读者当成没提交。

### 3. 打中四：维持拒绝，拒不拒取决于池的年龄

原型的点名容量压到 3，拒绝在 8684 段里出现在 2875 段（t6.out `refused_histories=2875`），那是装置的数。真容量上用实现的分配器量（costs.out，`PoolAllocator` 两盘、复用窗口置 0 只为让洞当场可复用，只数分配记录树叶、按真容量 812 满填充——最省的那种，祖先、记账、映射、树表一个都没算，所以是下界）：

```
AGED filler=4000 stride=50 d=40 records_before=8000 records_after=8000 alloc_leaves_total=10 touched_alloc_leaves_lower_bound=7 named_capacity=67 refused_under_keep_refusing=false
AGED filler=20000 stride=400 d=40 records_before=40000 records_after=40000 alloc_leaves_total=50 touched_alloc_leaves_lower_bound=41 named_capacity=67 refused_under_keep_refusing=false
AGED filler=20000 stride=400 d=20 records_before=40000 records_after=40000 alloc_leaves_total=50 touched_alloc_leaves_lower_bound=21 named_capacity=67 refused_under_keep_refusing=false
AGED filler=40000 stride=700 d=40 records_before=80000 records_after=80000 alloc_leaves_total=99 touched_alloc_leaves_lower_bound=70 named_capacity=67 refused_under_keep_refusing=true
AGED filler=40000 stride=700 d=34 records_before=80000 records_after=80000 alloc_leaves_total=99 touched_alloc_leaves_lower_bound=59 named_capacity=67 refused_under_keep_refusing=false
```

同一个用户动作「覆盖写一个 40 个数据单元的文件」：池里 4000 个对象时碰 7 片分配记录叶，40000 个对象、每 700 个删一个留洞时至少碰 70 片 > 67，Refuse 下这次写被拒；用户什么都没换，换的只是池的历史。按 40000 × 2 槽 × 16 KiB 算这个池约 1.2 GiB。

**推的、没跑**：删掉同一个文件只有释放那一半（40 个落点 × 两盘，落在各自的叶上），碰到的叶数与上面同一量级；它若也被拒，腾空间的那一次写就做不成。fs-design 第 23 行那一格写的是「准入控制要「进门前先算最坏情况」，而「释放空间这个操作本身不需要申请空间」也压在同一个数上」——它说的是空间，不是点名项；拿它判这一格是类比，没有条款直接管「释放要不要点名项」。

四句：
1. 分不分辨臂：分辨 Refuse 与 SpreadFlag（后者不拒）。CutPublish 在原型里拒掉的历史与 Refuse 是同一批 2875 段（`refused_publishes` 4279 对 3479，拆开以后被拒的次数反而多），因为装不下的是单个操作——**CutPublish 碰不到这一格**，要写明「对单个操作就超出的那几格不起作用」。
2. 系统看不看得到：看得到，写者在动盘之前就数得出（实现今天就在动分配器之前判，transaction.rs 第 2607 行）。
3. 满足的是哪一句：实现的 `MoreNamedUnitsThanOneJournalRecordHolds`；kb 里没有一条条款说「拒」，正文第一节把它列为候选之一。
4. 改法：SpreadFlag（原型量过：0 拒绝、0 违例，代价是多一条或几条记录、多一个标志位）；CutPublish 只修「多个操作挤在一次」那一种，单个操作超出的它照拒（量过）。

### 4. 发布拆成几次：没打中崩溃，代价

t6.out：CutPublish 与 Refuse 同样 0 违例；它比 Refuse 多发 639 次发布（14528 对 13889）、多写 1278 个单元（25482 对 24204）。崩在拆开的两次之间，读回的是前一半——D23 已定项 14 第 369 行写了「同一次定案把事务按单元切分 ⇒ 崩溃原子性的单位是一个单元、不是一次写请求」，所以不违反已定项；但每一次拆出来的发布若换了 extent / inode 树的根指针，就与 T1 第 3 节同一形，在回退窗口里多占一格（推的，没跑）。

## 自己提的改法（只在我的模型上量过、被攻过零轮）

| 改法 | 修哪一格 | 量过 / 推的 |
|---|---|---|
| 分隔 key 跟着维护（插到最左之下压低；借之后右孩子的分隔 key 改成它新的最小 key） | T1 打中一的两形 | 量过：t1.out 24 格 Maintained check_red = 0；targeted.out `BORROW sep=Maintained … checker=green lookup_misses=0` |
| inode 树删掉「左从右借」这一半，只留左吸收右 | T1 打中一的借那一形（inode 树） | 推的：没在实现上跑，今天没有删除 |
| 最左分隔 key 读成 −∞ | T1 打中一的插入那一形 | 推的；挡不住借那一形 |
| 根记录 / 树表条目加「只改结构」一位，判非空时跳过 | T1 打中二 | 推的，改格式 |
| 不单开结构发布（OnePublish 或 SplitOwnBarrier） | T1 打中二 | 量过：costs.out 两行都是 120，与底数同 |
| 末条标志位 | T6 打中三；也修打中四（不再拒） | 量过：t6.out SpreadFlag 0 / 0 / 0 拒绝；落在点名项 flags 字节还是记录头，推的 |
| 发布拆成几次 | 打中四里「多个操作挤一次」那一种 | 量过：单个操作超出的那几格照拒（2875 段不变） |
| 层 0 流按「起点镜像不枚举 + 只录分裂那一次」切 | T4 状态数 | 推的：原型这么切（段长上限 14 写），实现上没做 |

## 没打中的形状

每一格的「没打中」都只抽了这一次样，照 three-way-inference「一条腿只抽一次样不算一次观测」，不够支撑「这条臂没问题」。

| 形状 | 取样范围 | 结果 |
|---|---|---|
| 三种写序 × 四种收缩 × 两种分裂点（Maintained）的纯崩溃历史 | 24 格 × 8684 段历史（PA/PB/PC/PE × 第一步全菜单 × 第二步单操作全菜单），每格 1.19M–10.7M 个状态全量 | recover_fail、root_lost、content_mismatch、cut_publish_applied、lookup_miss、check_red 全 0 |
| 同上，加树高 3 的前缀 PD | 只跑了 Middle / Maintained / Merge 借 / OnePublish 一格：42875 段、35893515 个状态 | 全 0（out/t1-one-config-all-five-prefixes.out，行被截过） |
| TailWhenAppend 的根分裂 | 1 段定向历史 × 三种写序（4359 / 534 / 16714 个状态） | 全 0 |
| SpreadFlag、Refuse、CutPublish 的纯崩溃与叠一个介质故障 | 8684 段；叠加状态 SpreadFlag 17747219、Refuse 926570、CutPublish 965064 | overlay_red 全 0 |
| 分裂出来的左右两半 (树, 层级, 诞生代号, 写序) 相同会不会让恢复或 checker 认错 | 全部扫描（每次分裂都产生这一对） | 没出错：指针带 CRC 与槽号，出生序号不同；这一形在扫描重建（C113）路径上才可能出事，原型没做扫描重建 |

试过而没写成打中的：把「子的诞生代号 ≤ 父」当判据（TailWhenAppend 左半不重写时子比父旧，判据成立，不是违例）；SplitOwnBarrier 在分裂段落了、其余段没落的状态上（恢复回上一版，孤儿单元不在任何树里，checker 不走到它们）。

## 这条腿自己的限度

- 原型只有一棵树。实现里一次发布要同时改分配记录树（它要装自己那几条记录，分裂会让它自己的记录变多，是个固定点）、记账树、映射树（它给除自己、树表、实例表之外的每个节点一条条目）、树表。四棵树互相喂条目的那一圈（分配记录树分裂 ⇒ 多几个单元 ⇒ 多几条分配记录与映射条目 ⇒ 可能再分裂）**原型没建**，T5 的「一次发布最多触发几次分裂」与 T6 的共享单元数都被它低估。
- 条目、记录、根记录是简化格式；只有码 2 节点头是实现的字节。没有系统配置槽、实例表、实例切换、回退、根环回绕、槽复用（断言了枚举段里同一位置只写一次）。
- 节点容量 4、点名容量 3 是测试开关的值；T6 的 2875 段拒绝、8826 个切开状态是这个开关下的数，不是真容量下的频率。真容量那一格只有 AGED 五行，而且只数了分配记录树叶、按满填充算。
- 介质故障只叠了一种：这次发布的一个单元两盘都读不出、根没落。
- 两步用户动作的菜单是我定的；前缀也是我定的（PA–PE 五个）。L3「两层连着分裂」没有单独计数。
- 回退窗口那一格用的是实现的 `user_visible_trees_changed`，但「非空根」的序列是原型发布序列里的每一版，没有根环、没有 F、没有实例表，也没有真的回退。
- 「inode 树的左从右借必破 I-9.4」是从条款推的，没在实现上跑；今天实现没有删除。

## 没做什么

- 没在入库装置上跑；这里所有数都是副本上的，要引得由主 agent 在入库装置上重做。
- 没碰 T2、T3、T5（归正推腿与本地攻方）；T5 只报了「只摘空节点不降高 ⇒ 树高只涨不降」一个观测。
- 没读禁读的 `m2-treesplit-r1-sonnet-output.md`、`m2-treesplit-r1-local-attack*`。
- 没引别家文件系统的分裂算法，候选全部从本工程条款与实现出发（inode 容器的末尾分裂、D8 已定项 6 的分隔 key 与合并纪律、D18 已定项 2 的 key 区间）。
- 门禁：`stage-owners.tsv` 里没有登记给 three-way-attack 的阶段（awk 输出为空，退出码 0），没跑门禁。
- 06:18 起的第一次全格扫描在负载 270–340 下 4 分钟没出一格，我按写死的 pid（1064899、1065036、1065055）停了它、改成缓存前缀池与只扫四个前缀后重跑；那一次没有产物。
- 副本 `/tmp/claude-1000/m2-treesplit-opus/`（repo、target、out、logs）留着没删；入库的只有模型目录里那九个文件。
