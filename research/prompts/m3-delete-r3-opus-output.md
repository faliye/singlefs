# m3-delete-r3 攻方腿报告（Opus，2026-10-01）

<!-- doc-lint:not-numbers G1 G2 G3 G4 G5 G6 G7 G8 S2 N1 N2 N3 N4 N5 N6 K-U4 K-W5 K-W5b W3 W4 W6 LC LO LT LI LR R3 M1 X1 -->

最终轮：这份报告里我提的收严与新形态都被攻过零轮。攻击面是正文第四节「云端攻方」那一行；前两轮攻过的角度（扣两次的「增量减」实现、size 不跟着截、回退两个方向的后果、R3 逐文件预付的过量、墓碑跟现行那一版走而不带谱系、区间作废的来回回退等）没有重攻。

## 零、复跑与文件

复跑（从仓根；冻结副本 `/tmp/claude-1000/m3-delete-r2/tree` 照 `research/prompts/m3-delete-r3-snapshot/crates-sha256.txt` 核过：210 份 OK、0 份不对）：

```
RUN_WITH_MEMORY_CAP_WAIT_SECONDS=3600 nice -n 19 bash research/scripts/run-with-memory-cap.sh 8G bash research/scripts/capped.sh 5 \
  bash research/prompts/m3-delete-r3-opus-model/rerun.sh /tmp/claude-1000/m3-delete-r2/tree <还不存在的草稿目录>
```

脚本拷冻结副本、打上 `research/prompts/m3-delete-r2-opus-model/core-prototype-r2.diff`（sha256 881d3c1a…79e7，这一轮没改 core，只加用例）、跑 harness 档 `m3_delete_r3_attack_prototype` 的 5 个用例，再跑模型的 6 条命令，末尾与本目录两份产物逐行比。这一份产物就是这个脚本在 `/tmp/claude-1000/m3-delete-r3-opus/rerun1` 跑出来的（「test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 491.46s」，挂钟 11 分 27 秒）；模型那一份与之前单独跑的一次逐字相同。

| 文件（`research/prompts/m3-delete-r3-opus-model/`） | sha256 | 是什么 |
|---|---|---|
| `m3_delete_r3_attack_prototype.rs` | c71f3cfa33ec604fa198f1d9cbf5838c1dde8a37c7c7df107eea3118a4663819 | 原型用例（环窗口几何、W 随文件大小、大文件窗口、回退准入、墓碑树根容量） |
| `s2_lineage_model.py` | 6ad4882596525189cd88a3dd029cc88882a91cb351270fcd3559890297b386ba | S2 删除 + 墓碑 + 谱系的计数模型与 checker |
| `rerun.sh` | 6f1a98bf015476ccb259df6dded897fa10ddc3a3086c04a6a7a9dc0596263b1a | 复跑脚本 |
| `run-e-lines.txt` | ff42292d431204923a743145ba4a7b5d6143a570d52056e6d57f3531f1530d93 | 原型的 25 行 `E name=`（排序过） |
| `model-output.txt` | 77ff3d3336258bde22ad05240cc9aa0117bbdd88bab0f4a3ffcd3eabaebfe121 | 模型的 72 行 |

崩溃状态：原型与模型一个都没枚举（层 0 枚举函数没调），这一条腿合计 0 个状态。挂钟：环窗口 14 行约 2 分钟；10000 单元那几行每行约 2 分钟；模型深度 7 一种组合约 1 分钟。没设批的限时。
## 一、各格判定一览

「量过」= 副本原型（`m3_delete_r3_attack_prototype.rs`，冻结副本 + m3-delete-r2 攻方的 `core-prototype-r2.diff`）或计数模型（`s2_lineage_model.py`）的原样输出；「推的」= 按代码与条款推、没实现没跑。原型里没有墓碑树、`deleted_inodes`、谱系记录，它们的单元数是推的；模型里没有空间。我提的改法都只在我的模型上量过、被攻过零轮。

| 格 | 攻的是什么 | 判定 | 量过 / 推的 |
|---|---|---|---|
| G1-a 删除发布按「发布之后的标签」判 | 满池（式子可用 = 0，账里只有环窗口），删除各批 | **打中**：环窗口是常量，发布之后标签 4、6 一个字节都不降，可用按发布后算每一次都为负——18 个几何行、每一次删除发布都是负的（例 4 GiB、S = 16：71 次里 71 次，最低 −10 223 616 字节 / 盘）。照这一读法每一批都被拒 | 量过 |
| G1-b 删除各批不判式子（只判落点） | 同上 | **打中「同一批字节扣两次」字面**：删除新占的槽进「已分配」，覆盖它们的环窗口仍整份留在标签 6，删除途中可用少一个顶点（4 GiB、S = 16 顶点 624 槽 ≈ 9.75 MiB / 盘）。与 checkpoint 保留池同形，见 2.2 第四句 | 量过 |
| G1-c W 上界取小文件的量 | 4 GiB、S = 16，10000 单元文件、K = 144 | **打中「环窗口 < 顶点」**：W 取 1000 单元那一批 24 槽时窗口 1176 槽，量到的顶点 3092 槽；512 MiB 盘、5000 单元文件顶点 1276 槽，同样大于 1176 | 量过 |
| G1-d W 随文件大小 | K = 144，1000 / 3000 / 6000 / 10000 单元 | 一批 W = 24 / 38 / 59 / 87 槽，四点都等于「⌈单元数 ÷ 144⌉ + 17」：今天唯一的 extent 写法整段重写下段，W 跟文件大小线性长。上界按盘上最大文件取，4 GiB 盘约 747 槽 / 批，窗口占单元区 8.8 %（S = 8）/ 17.3 %（S = 16） | 四点量过，外推推的 |
| G1-e 小几何上窗口多大 | journal 1 MiB / 盘 32 MiB 起五档 × S = 4 / 16 | 不是跑前判据的打中，是代价：最小那一档（32 MiB、S = 16）窗口 637 槽 = 单元区的 66.3 %，带一个 70 单元文件时按窗口算的可用只剩 25 槽 | 量过 |
| G1-f 每对象一份墓碑按记录（40 字节）预付 | 第一次 unlink 开第一个墓碑容器 | **打中（推的）**：容器 32 KiB 一份常驻，记录摊只付 40 字节；按发布之后的标签判时满池上第一次 unlink 被拒，不判时保留量被永久吃 32 728 字节 / 盘 | 推的（算术） |
| G1-g 每对象一份按整容器（32 KiB）预付 | 建空 inode | 代价：4 GiB 盘 31 455 个 inode（一个根装得下的数）预付 62 910 槽 = 单元区 29.7 %；512 MiB 盘（单元区 27 648 槽）13 824 个 inode 之后建 inode 被拒 | 推的（算术） |
| G1-h 标签 6 数「inode 树里的记录」而不是「nlink = 1 的」 | 删除中那一段 | **打中（推的）**：墓碑记录已进容器（已分配），它的预付还在标签 6，扣两次 | 推的 |
| G1 环长 + 1 够不够 | 罩不罩得住量到的顶点（W 取同一池探针量到的最大一次） | 没打中：W 取探针量到的最大一次的 14 个几何行里顶点都 ≤ (3S + 1) × W，顶点出现在第 12 / 24 / 47–54 次发布 | 量过 |
| G3-a 回退那次多写谱系容器 | 满池、账里环窗口 + 每对象墓碑，回退到删之前 | **打中**：多 2 槽（另立）或 3 槽（挂墓碑树下）普通分配时照「有普通分配就判」被拒；只把标签 6 从 R_old 重算抬上去的那 40 字节也被拒。今天的回退（不判）做成，之后可用 −622 632 / −655 360 字节 / 盘 | 量过（准入函数） |
| G3-b 按谱系过滤的 checker 判活判死 | 删、回退、再删、来回回退、崩溃落到旧根、回退进删除中间再续做，深度 7 全枚举 4 243 236 个状态 | 没打中：谱系过滤的「墓碑不指活对象」0 次红；按父链走出的谱系与真值逐条相同（0 次错分） | 量过（模型） |
| G3-c 2026-09-26 删回退行的理由 | 三条理由逐条对谱系记录 | 一条都打不到（推的，3.4） | 推的 |
| G4 住处 | 另立随 cur / 另立随 R_old / 挂墓碑树下与墓碑容器共用容器号 | 前两者在 G3 那几段历史上都 0；**挂墓碑树下打中 N1、N5 字面**（合法状态上 N1 红 253 046 次、N5 红 160 523 次 / 363 791），且谱系容器吃墓碑树根的格子 | 量过（模型） |
| G6-a 墓碑树根满 | 根恒为码 2、照 inode 树内部节点 120 字节 | **打中**：根装 135 个容器，815 条一容器 ⇒ 第 110 026 次 unlink 要树长高，今天 inode 树的写法在那一步拒；容量旋钮 1 / 2 条时第 136 / 271 次。墓碑不回收、回退不撤墓碑，回退掉的删除也占 | 常量量过，拒是推的 + 模型 |
| G6-b 容量旋钮下的 N5 | 旋钮不进盘 | **打中**：checker 按产品容量判时合法状态 N5 红 1206 次；按旋钮判时 checker 要从镜像外拿旋钮值 | 量过（模型） |
| G7 | 13 种改坏 × 18 条判据 | N4 字面在合法历史上红（删 → 回退 → 再删）；N2 不按谱系过滤时 3 种改坏不红；墓碑树随 R_old 那一种改坏 18 条一条都不红；谱系过滤与核心层不是同一段算法（核心层只写不读谱系） | 量过（模型） |
| G2 | 续做把墓碑的死亡 txg 改成自己的 txg | **打中**：「逻辑内容不含 txg」那一读法下五条都过；之后回退进删除中间再续做，这个对象没有一条谱系上的墓碑 | 量过（模型） |
| G5 | 墓碑树不空对第三成员 | 没打中（推的） | 推的 |

## 二、G1　环窗口的式子

装置（`ring_window_row_with`）：可写挂载之后的池（分配器按根环回收）上一个文件；探针在拷贝上照常删一遍，量意图那次与每批新占的槽，W_max 取其中最大一次；账里只给「(3S + 1) × W_max」那一份（已承诺预留，按全部副本之和记），待删占用给 0；把池填到式子可用 = 0（只动分配器拷贝）；删除各批关式子那一判、落点那一判照判（m3-delete-r2 攻方的两个开关）；量「填满那一刻的空闲 − 此刻空闲」的顶点。S2 的墓碑树与 `deleted_inodes` 每次删除多 3–4 槽，原型里没有（推的）。

### 2.1 删除发布按「发布之后的标签」判：每一批都被拒（打中，量过）

S2 第 10 条写「删除类发布按「发布之后的标签」抵扣，不按发布前读数判」。这句是从 R3（逐文件预付、意图那次转进标签 4、各批递减）那一支搬来的：那时一批做完标签 4 降一批的量，发布之后的可用不降。换成全池一份环窗口之后，标签 6 是「每对象墓碑 + 常量窗口」，一批删除做完标签 4、6 一个字节都不降，而空闲少了这一批新占的槽，所以发布之后的可用每一次都比发布前低。

```
E name=ring_window journal_mib=768 image_mib=4096 slots_per_region=16 ring_roots=48 units=70 batch=1 w_bound=probe unit_area_slots=211968 checkpoint_cost=537 reserve_slots=6989 w_intent=15 w_batch_max=17 window_slots=833 window_per_mille_of_unit_area=3 available_slots_with_the_window_before_fill=203946 free_slots_at_full=7822 publishes=71 result=Ok vertex_slots=624 at_publish=50 vertex_le_window=true publishes_with_negative_available=71 min_available_bytes=-10223616 consumed_after_last=616
```

全部 18 个几何行（第零节 `run-e-lines.txt` 的 `name=ring_window`）`publishes_with_negative_available` 都大于 0，W 取探针最大值的 14 行与 512 MiB 两行等于 `publishes`（4 GiB 10000 单元两行 71 次里 53 次）。`result=Ok` 是因为这里只判落点；照「发布之后的标签」判，第一批就被拒。

四句：① 分辨臂：分辨「常量窗口」与 R3 那种「随删除递减的预付」；不分辨环窗口的各种 W 取法。② 系统看得到：发布之后的标签是这一版的函数，看得到；要让它「抵扣」，就得知道窗口里已经被在飞的删除用掉多少，而那是「最近 3S 个 txg 里哪几次是删除发布、各占了多少」——镜像上没有这一项（推的：分配记录只带分配代，不带是谁分配的）。③ 满足 G1 跑前判据「某次删除发布……在已按环窗口预付的池上被准入拒」，前提是按 S2 第 10 条字面判。④ 改法：(a) 删除各批不判式子、只判落点（就是装置这一档，它撞 2.2）；(b) 窗口里减去「在飞」那一份——镜像上算不出，不收。

### 2.2 删除各批不判式子：同一批字节扣两次（打中字面，量过）

照 2.1 的 (a)：删除不被拒，但删除途中它新占的槽进了「已分配」（空闲少了），覆盖这些槽的窗口仍整份留在标签 6。上一行里删除途中可用最低 −10 223 616 字节 / 盘（624 槽），到环转过一圈、换下的东西回收才回来。这一段里用户的普通写按式子判，少放行 624 槽。

四句：① 分辨「常量窗口」与「递减预付」；**同一形状今天就有**：checkpoint 保留池也是常量，空发布的固定点从空闲里出、保留池照扣（`.claude/kb/decisions/28-挂载期承诺量.md:34`：「判不判照旧：没有普通分配的发布（空发布、写行、暖机、抬 F）不判」）。所以这一中只分辨「窗口放在标签 6」还是「窗口当成式子里一个常驻扣项（与 checkpoint 保留池同类）」。② 系统看得到。③ 满足 G1 判据「「已分配」「已承诺预留」「待删占用」三项里有同一批字节被扣两次」的字面。④ 改法：把窗口从标签 6 挪出来，在式子里另立一个按几何现算的常驻扣项（不进记账行），字面就不再是「已承诺预留」里的字节；扣的量与今天一样，代价不变（零轮）。

### 2.3 W 随文件大小线性长；W 上界取小文件的量时窗口小于顶点（打中，量过）

```
E name=batch_w units=1000 batch=144 publishes=8 max_new_slots=24 at=batch_2 rewritten_slots_by_kind_at_max={"accounting_root":1,"allocation_node":8,"allocation_root":1,"extent_lower":6,"extent_root":1,"inode_leaf":2,"inode_root":1,"mapping_node":2,"mapping_root":1,"tree_table":1} new_slots_series=[23, 24, 24, 23, 23, 21, 19, 16]
E name=batch_w units=10000 batch=144 publishes=71 max_new_slots=87 at=batch_2 rewritten_slots_by_kind_at_max={"accounting_root":1,"allocation_node":8,"allocation_root":1,"extent_lower":69,"extent_root":1,"inode_leaf":2,"inode_root":1,"mapping_node":2,"mapping_root":1,"tree_table":1} new_slots_series=（71 个数，原样在 run-e-lines.txt）
```

（第二行 `new_slots_series` 太长，这里省掉，原样行在产物里。）1000 / 3000 / 6000 / 10000 单元的最大一批是 24 / 38 / 59 / 87 槽，四点都等于 ⌈单元数 ÷ 144⌉ + 17；涨的全在 `extent_lower`（6 / 18 / 41 / 69）。原因：删除批走的是今天唯一的 extent 写法，「两个起建下段（整段重写）」（`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-core/src/transaction.rs:3940`）——每一批把下段整段重写一遍。

同一个 10000 单元文件、4 GiB、S = 16，窗口的 W 取两种：

```
E name=ring_window journal_mib=768 image_mib=4096 slots_per_region=16 ring_roots=48 units=10000 batch=144 w_bound=24 unit_area_slots=211968 checkpoint_cost=671 reserve_slots=8731 w_intent=0 w_batch_max=0 window_slots=1176 window_per_mille_of_unit_area=5 available_slots_with_the_window_before_fill=181812 free_slots_at_full=9907 publishes=71 result=Ok vertex_slots=3092 at_publish=47 vertex_le_window=false publishes_with_negative_available=53 min_available_bytes=-31916032 consumed_after_last=-4627
E name=ring_window journal_mib=768 image_mib=4096 slots_per_region=16 ring_roots=48 units=10000 batch=144 w_bound=87 unit_area_slots=211968 checkpoint_cost=671 reserve_slots=8731 w_intent=0 w_batch_max=0 window_slots=4263 window_per_mille_of_unit_area=20 available_slots_with_the_window_before_fill=178725 free_slots_at_full=12994 publishes=71 result=Ok vertex_slots=3092 at_publish=47 vertex_le_window=true publishes_with_negative_available=53 min_available_bytes=-31916032 consumed_after_last=-4627
```

W 取 24（1000 单元文件量到的最大一批，也是第二轮攻方 2.1 节表里最大那一格）时窗口 1176 槽 < 顶点 3092 槽；这里删完了，是因为保留量 8731 槽兜住了差额。小一档的盘同样中（journal 64 MiB、512 MiB 盘、S = 16、5000 单元，journal 记录上限让一次顺序写最多约 5400 单元）：

```
E name=ring_window journal_mib=64 image_mib=512 slots_per_region=16 ring_roots=48 units=5000 batch=144 w_bound=24 unit_area_slots=27648 checkpoint_cost=142 reserve_slots=1854 w_intent=0 w_batch_max=0 window_slots=1176 window_per_mille_of_unit_area=42 available_slots_with_the_window_before_fill=14474 free_slots_at_full=3030 publishes=36 result=Ok vertex_slots=1276 at_publish=36 vertex_le_window=false publishes_with_negative_available=36 min_available_bytes=-7536640 consumed_after_last=1276
```

按最大文件取 W 才罩得住：4 GiB 盘单元区 211 968 槽，一个文件最多约 105 000 单元，W ≈ 747 槽 / 批，窗口 25 × 747 = 18 675 槽（S = 8，单元区 8.8 %）、49 × 747 = 36 603 槽（S = 16，17.3 %）（推的：四点线性外推）。那时顶点约 48 × 700 ≈ 33 600 槽，大于保留量，窗口要是取小了，删除在落点那一判被拒（推的，没量到被拒：内存装不下那么大的文件）。

四句：① 分辨「W 取几何函数（随最大文件）」与「W 取量到的常数」；不分辨窗口与 R3（R3 的逐文件预付也要同一个 W）。② 写者看得到这个文件的单元数；checker 从镜像也算得出最大文件——W 能写成镜像上的函数，只是它大。③ 满足「环窗口的量在某个几何上小于那个几何上量到的借保留量顶点」，几何：4 GiB、journal 768 MiB、S = 16、文件 10000 单元、K = 144、W 取 24。④ 改法：(a) W 按最大文件取（代价如上）；(b) 删除批只重写下段尾叶与它的祖先（下段按位置寻址，去掉尾部 144 个单元正好去掉一片叶），W 回到与文件大小无关的十几槽——这是 extent 写法的新分支，零轮，要过 `.claude/rules/fs-design.md` 五条硬要求。

### 2.4 小几何上窗口多大（代价，不是跑前判据的打中）

| journal / 盘 | S | 单元区槽 | 保留量 | W_max | 窗口槽 | 窗口 / 单元区 | 顶点 | 带文件时按窗口算的可用 |
|---|---|---|---|---|---|---|---|---|
| 1 MiB / 32 MiB | 4 | 960 | 112 | 11 | 143 | 14.8 % | 88 | 599 |
| 1 MiB / 32 MiB | 16 | 960 | 112 | 13 | 637 | 66.3 % | 538 | 25 |
| 16 MiB / 128 MiB | 16 | 6144 | 294 | 13 | 637 | 10.3 % | 562 | 5027 |
| 64 MiB / 512 MiB | 16 | 27648 | 970 | 13 | 637 | 2.3 % | 532 | 25855 |
| 768 MiB / 3 GiB | 16 | 146432 | 4896 | 17 | 833 | 0.5 % | 624 | 140503 |

数取自同名的 `name=ring_window` 行（K = 1）。32 MiB 是 journal 1 MiB 时 mkfs 收的盘（journal 环不超过盘的四分之一）；默认环 768 MiB 时最小盘是 3 GiB。最小档上 S = 16 的窗口吃掉单元区的三分之二；窗口从哪一次发布起计（G1 题面），在这一档上决定第一个文件版本那次发布过不过准入。顶点都在第 3S 次附近（12、49–54），都 ≤ 窗口：「(环里根数 + 1)」这个系数在 W 取对时没打中。

### 2.5 每对象一份墓碑：按记录还是按整容器（推的，算术）

- **按记录（40 字节 × 2 份）**：墓碑容器是 32 KiB 一份常驻单元；第一次 unlink 开第一个容器，记录只付了 40 字节，差 32 728 字节 / 盘；之后每开一个新容器差回到同一个数，最低 32 768 − 815 × 40 = 168 字节。窗口罩的是开放容器 COW 那一份在飞的量，罩不到常驻的那一份。按 2.1 的读法，满池上第一次 unlink 被拒；按 2.2 的读法，保留量被永久吃掉至多 32 728 字节 / 盘，外加墓碑树长高时多的码 2 节点。
- **按整容器（32 KiB × 2 份）**：4 GiB 盘 31 455 个 inode（今天一个 inode 根装得下的记录数）预付 62 910 槽，单元区的 29.7 %；512 MiB 盘单元区 27 648 槽，13 824 个 inode 之后建 inode 就被准入拒。
- **改法（零轮）**：每对象按记录（40 字节）+ 全池常驻一份「一个开放墓碑容器 + 墓碑树一条根到叶的路径」，与窗口并列；W6「按记录摊不够写第一个容器」那一中由常驻那一份接住。

### 2.6 标签 6 数哪些对象（推的）

「每个对象一份墓碑」若数 inode 树里的全部记录（含 nlink = 0 的删除中对象），意图那次之后到最后一批之间，这个对象的墓碑记录已经写进容器（已分配），它的预付还在标签 6：同一 40 字节（或 32 KiB）扣两次，满足 G1「扣两次」字面。数「nlink = 1 的 inode」就不重；S2 第 10 条要写明。回退到删之前，对象回来、预付按 R_old 重算回来，而上一次删除写的墓碑留在容器里（墓碑随现行那一版、不回收）——这 40 字节从此没有预付罩着，每一次「删 → 回退」漏一条（3.1 的 −40 字节就是它）。

## 三、G3　回退与谱系记录

### 3.1 回退那次因谱系容器与标签 6 重算被空间拒（打中，量过）

装置（`rollback_with_a_lineage_container_on_a_full_pool`）：4 GiB、S = 8，300 单元文件 A，删完（K = 144），账里给「环窗口 + 每对象墓碑」（删完之后 nlink = 1 的对象 0 个），填到式子可用 = 0，再回退到 A。准入函数（`admit_on_every_device`）直接判几种需求，再照今天的代码真回退一次：

```
E name=rollback_full_pool tomb_reading=per_record w_max=19 window_bytes_summed=15564800 ledger_at_cur=15564800 ledger_at_a=15564880 available_at_cur=[0, 0] judged=[0slots+0raise:Ok 2slots+0raise:Refused[(0, 32768), (0, 32768)] 3slots+0raise:Refused[(0, 49152), (0, 49152)] 0slots+80raise:Refused[(0, 40), (0, 40)] 2slots+80raise:Refused[(0, 32808), (0, 32808)] 3slots+80raise:Refused[(0, 49192), (0, 49192)]] today_rollback=Ok rollback_new_slots=12 units_after=300 available_after_with_tag6_from_r_old=[-622632, -622632]
E name=rollback_full_pool tomb_reading=per_container w_max=19 window_bytes_summed=15564800 ledger_at_cur=15564800 ledger_at_a=15630336 available_at_cur=[0, 0] judged=[0slots+0raise:Ok 2slots+0raise:Refused[(0, 32768), (0, 32768)] 3slots+0raise:Refused[(0, 49152), (0, 49152)] 0slots+65536raise:Refused[(0, 32768), (0, 32768)] 2slots+65536raise:Refused[(0, 65536), (0, 65536)] 3slots+65536raise:Refused[(0, 81920), (0, 81920)]] today_rollback=Ok rollback_new_slots=12 units_after=300 available_after_with_tag6_from_r_old=[-655360, -655360]
```

- 今天回退那次只写固定点、不判（`.claude/kb/decisions/28-挂载期承诺量.md:34`：「判不判照旧：没有普通分配的发布（空发布、写行、暖机、抬 F）不判」），做成。
- S2 第 7 条让它多写一个码 3 谱系容器。码 3 的 inode 叶容器今天归普通分配（`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-core/src/admission.rs:925`：「| TransactionUnit::InodeLeafContainer(_)」），谱系容器若照它，回退那次就有了普通分配，照有普通分配就判的那一处（`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-core/src/admission.rs:943`：「pub fn publish_has_an_ordinary_allocation(rewritten_roles: &[TransactionUnit]) -> bool {」）在满池上被拒：需求 2 槽（另立）或 3 槽（挂墓碑树下、连根）。
- 标签 6 从 R_old 重算：R_old 里对象还活着，每对象墓碑那一份抬回来，按记录 40 字节 / 盘、按整容器 32 768 字节 / 盘；单这一项照「发布之后的标签」判也被拒。回退跨过 k 次 unlink 时抬 k 份（推的：按整容器、k = 1000 是 32 MiB / 盘）。
- 不判时：之后可用 −622 632 字节 / 盘（其中 −622 592 = 38 槽今天就有，第二轮攻方 3.1 节同一机理；−40 是标签 6 抬回来的那一份）。

四句：① 分辨「谱系容器归普通分配」与「归 checkpoint 保留池（固定点）」；标签 6 那一半分辨「每对象一份」与「不随对象数变」。② 回退那次在内存里同时有 cur 与 R_old 的账，看得到。③ 满足 G1 判据「某次……回退发布在已按环窗口预付的池上被准入拒」（按判的那一读法）；不判的读法不满足任何一条字面（I-3.4 只要求扣了），落在「可用为负」。④ 改法：(a) 谱系容器归固定点，不判，ckpt_cost 的 Σ 名单加「一次谱系容器 COW」（推的：保留量随 ckpt_cost 乘 13，约多 26–39 槽 / 盘）；(b) 谱系容器那 2–3 槽并进环窗口那一份（窗口罩的就是「不判式子、从保留里出」的发布）；(c) 标签 6 的墓碑那一份回退时照抬、不判，靠 2.5 的常驻那一份兜——每次「删 → 回退」漏一条记录（2.6），无界。三个都零轮。

### 3.2 按谱系过滤的 checker（量过，模型，没打中）

模型 `s2_lineage_model.py`：S2 第 1、2、6、7、11、12 条照字面（意图写 nlink = 0、加队列、墓碑带死亡 txg；K = 1 分批；回退取 R_old 的 inode / extent / 队列、墓碑树取 cur、写一条 (T_rb, T_old)；崩溃恢复新实例第一个根取环里最大 + 1，可写挂载把队列走到空），用户与管理员的动作全部放开（建、空发布、删、续做、回退到候选集里任一根、崩溃丢最新 0–3 条根）。checker 只看挂载根这一版，谱系从挂载根走：有谱系记录的 txg 跳 T_old，其余取 txg − 1（与 `/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-core/src/transaction.rs:3420`：「pub fn checkpoint_txg_of_the_publish_after(」的「加一」、`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-core/src/mount.rs:280`：「新实例第一次发布的 txg（环里最大的 txg + 1）」同一条规则）。

```
E name=matrix depth=7 cap=2 check_cap=knob rootcap=0 ring=6 placement=LC mutation=none states=4243236 N1=0 N2any=0 N2lin=0 N3U=504762 N3L=0 N4lit=140968 N4d=0 N4lin=0 N4cap=0 N5order=0 N5lit=0 N5type=0 I95=0 I47=0 I96=0 I910U=0 I910L=0 lineage_misclassified=0 refused=- first_N3U=…
```

（行尾 `first_N3U=`、`first_N4lit=` 两格是第一段红的历史，原样在 `model-output.txt`。）`N3L`（按谱系过滤的「墓碑指认的对象不在 inode 树里或 nlink = 0」）在 4 243 236 个状态上 0 次红；`lineage_misclassified`（按父链走出的谱系与真实祖先对某条墓碑判法不同）0 次。不按谱系过滤的那一版（`N3U`）红 504 762 次——删 → 回退到删之前就红。I-9.5（`N` 在队列 ⇒ 在树里且 nlink = 0，改成照 `.claude/kb/invariants.md:279` 的全句）与 I-4.7 都 0。

具名历史（`name=named`）：删 → 回退 → 再删同一对象、删 → 回退到删之前 → 再回退到删之后一版（来回回退，D23 已定项 14 射程第 ③ 条的那种）、回退进删除中间再续做、崩溃落到删除中间再回退到删之前，随 cur（LC）与随 R_old（LO）两种住处上 N3L 都不红。

四句（没打中这一格，只写依据）：它站得住靠两个前提：(1) 非回退的发布 txg 恰好加一、崩溃恢复后新实例的第一个根取「环里最大 + 1」而所选根就是环里最大的有效根；若有一种恢复落在「环里还有更大 txg 的可读有效根」之下，txg − 1 那一步会走过被抛弃的 txg（模型里崩溃丢的根都读不出，没造出这一种）；(2) 墓碑与谱系记录都只从挂载根这一版的树里读，被崩溃恢复抛弃的版本不在里面。

### 3.3 三种回退之后 I-4.7、I-9.5、I-3.4

回退到删之前、回退进删除中间：模型里 I-9.5、I-4.7 全部 0（上一行）。I-3.4 只要求扣了，3.1 的负可用不违反字面。回退跨过第一个文件版本：今天候选集就排除不带文件的版本（`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-core/src/mount.rs:4755`：「RollbackCandidateExclusion::VersionWithoutFile,」），没有可达历史。

### 3.4 2026-09-26 删回退行的理由打不打到谱系记录（推的）

依据原文（`.claude/kb/decisions-history.md:8036`）：「回退实例永远不给自己写回退行、截断走不到是结构事实；回退行的解析没量，按换 incompat 位之后旧镜像挂不上，新代码不必认」。

1. 「回退实例永远不给自己写回退行」：说的是回退开新实例时代的行；今天回退不新增实例（`.claude/kb/decisions/23-journal的角色与格式.md:376`：「不新增实例、不取号、不写实例表行、不施加也不删除任何记录」），谱系记录由回退那次发布自己写，这条结构事实不适用。
2. 「截断走不到」：说的是实例表的截断；谱系容器只增不删，不进实例表，没有截断。
3. 「解析没量、换 incompat 位旧镜像挂不上」：谱系记录并进第一条布局线（S2 第 3 条不加新位），没有旧镜像要认。

三条都打不到。它们本来回答的是重放与恢复要不要回退行，没回答扫描重建怎么认「挂载根是一次回退」——这正是墓碑第一轮判决第二节判 D23:417 被打穿的那一半；谱系记录补的就是它。反过来，谱系记录若放回实例表（G4 候选 LI），第 2 条的截断与实例表读不出那一形（`.claude/kb/decisions/18-块里携带什么信息.md:311`）就打到它：实例表读不出时谱系全丢，checker 按 txg − 1 走，回退掉的删除重新落进谱系，N3L 当场把回退回来的对象判死（推的，模型里没建实例表读不出这一形）。

## 四、G4　谱系记录的住处与格式

| 候选 | 定义 | G3 那几段历史上 checker 判活判死 | 另外撞了什么 | 量过 / 推的 |
|---|---|---|---|---|
| LC 另立、随 cur | 一个只增不删的码 3 容器链，回退那次在 cur 的那一份后面追加 | 深度 7 全枚举 0 | 3.1 的准入（2–3 槽） | 量过（模型） |
| LO 另立、随 R_old | 取 R_old 那一份再追加这一条 | 深度 7 全枚举 0，与 LC 逐格相同 | 同 LC；只留祖先上的回退，丢掉被回退掉的那一支上的记录（单镜像 checker 用不到它们；扫盘重建要不要，归 K-W5b 的欠账） | 量过（模型） |
| LT 挂墓碑树下 | 打包记录类型另一个号的容器，作墓碑树根的另一种孩子，与墓碑容器共用容器号 | N3L 0 | **N1 字面红**（类型段 ∈ {1, 3}）253 046 次、**N5 字面红**（非最右的类型 1 容器记录数 == 容量：开放墓碑容器右边插进了一个谱系容器）160 523 次，合法状态共 363 791；谱系容器吃墓碑树根的格子：根装 3 个孩子、容量旋钮 1 时，深度 6 里 unlink 因根满被拒 32 309 次（LC 只 4 次） | 量过（模型） |
| LI 实例表行 | 回退那次在实例表加一行 | 没建 | 就是 2026-09-26 删掉的回退行；实例表读不出时谱系全丢（3.4） | 推的 |
| LR 根记录 | 根记录带「这一次的 T_old」 | — | 只有最近一次回退，来回回退要的是整条祖先链上每一次回退；根记录 457 字节、512 字节扇区上只剩 55 字节（墓碑第一轮攻方 3.2 节第 1 条） | 推的 |

```
E name=matrix depth=6 cap=2 check_cap=knob rootcap=0 ring=6 placement=LT mutation=none states=363791 N1=253046 N2any=0 N2lin=0 N3U=41395 N3L=0 N4lit=9492 N4d=0 N4lin=0 N4cap=0 N5order=0 N5lit=160523 N5type=0 I95=0 I47=0 I96=0 I910U=0 I910L=0 lineage_misclassified=0 refused=- first_N1=… first_N3U=… first_N4lit=… first_N5lit=…
E name=matrix depth=6 cap=1 check_cap=knob rootcap=3 ring=6 placement=LT mutation=none states=298901 N1=189683 N2any=0 N2lin=0 N3U=29841 N3L=0 N4lit=0 N4d=0 N4lin=0 N4cap=0 N5order=0 N5lit=0 N5type=0 I95=0 I47=0 I96=0 I910U=0 I910L=0 lineage_misclassified=0 refused=tomb_root_full=32309 first_N1=… first_N3U=…
```

（`first_` 各格原样在 `model-output.txt`。）

格式（推的）：
- 字段 (T_rb, T_old) 各 8 字节够：单镜像 checker 只在挂载根这一版的祖先链上查，祖先链上 txg 严格递减、每个 txg 只对一个根；跨实例回退的目标按 cur 的实例表判有效（候选集），模型里跨崩溃恢复的回退 0 次错分。32 KiB 容器装 ⌊(32768 − 136) ÷ 16⌋ = 2039 条（`E name=tomb_root_capacity … lineage_records_per_container=2039`）。
- 打包记录类型：代码里只登记了 2 与 4；墓碑取 1；3（退役）在谱系那一族里没有写者（old_rw 才要它）。谱系取一个新号（5），不复用 3：「码一旦用过不许回收」那条纪律对条款里登记过的 3 同样适用（推的，3 在代码里没登记）。
- 开放容器纪律：与墓碑容器同一条（有新记录才重写、满了才关）；只有回退那次有新记录，每次回退一次 32 KiB × 2 份的 COW（在飞，环窗口那一类）+ 每次回退常驻 16 字节 × 2 份，与被回退掉的数据量无关（G3 跑前判据「随被回退掉的数据量增长」那一格不中）。
- 不回收：增长率 = 回退次数 × 32 字节（两份），每 2039 次回退一个新容器。
- 冻结层与态别：没做（第十节）。

## 五、G6　墓碑树分裂与容量旋钮下的关闭路径

### 5.1 墓碑树根满：分裂那一步没有条款，照今天 inode 树的写法拒（打中）

```
E name=tomb_root_capacity containers_one_root_holds=135 tombs_per_container=815 unlinks_until_the_root_is_full=110025 with_knob_1=135 with_knob_2=270 lineage_records_per_container=2039
```

S2 第 4 条：根恒为码 2，条目照 inode 树内部节点 120 字节，根装 135 条（`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-core/src/inode_tree.rs:106`：「一个码 2 根装得下几条 inode 内部条目：扇出 135」）。墓碑第一版不回收（W3），回退也不撤墓碑（墓碑树随现行那一版），所以这是池一生的上限：第 110 026 次 unlink（容量旋钮 1 / 2 条时第 136 / 271 次）要第 136 个容器，树要长高。D8 已定项 6 对 inode 树只写「内部节点同样在右端溢出、末尾分裂，右半是码 2、没有容器身份」（`.claude/kb/decisions/08-核心索引结构.md:169`），没写根怎么长高；今天的代码在这一步拒（`/tmp/claude-1000/m3-delete-r2/tree/crates/singlefs-core/src/inode_tree.rs:142`：「MoreLeafContainersThanOneRootNodeHolds { containers: usize, capacity: usize },」，注释写「没写树长高那一步」）。墓碑树若照搬这段写法，那一次 unlink 在意图那次被拒——删除因结构而不是空间失败，D3 已定项 17 的字面（`.claude/kb/decisions/03-空间分配.md:27`：「**释放空间这个操作本身不需要申请空间**」）在那之后不兑现。

模型（根装 3 个孩子、旋钮 1 条，深度 6）：LC 被拒 4 次，LT 32 309 次（`refused=tomb_root_full=` 两行）。删 → 回退的循环同样吃格子：回退掉的删除，墓碑留在树里。

四句：① 不分辨谱系的住处（LC / LO 也撞，LT 撞得早）；分辨「墓碑树会长高」与「照 inode 树今天的写法」。② 写者看得到孩子数。③ 跑前判据 G6 写的是「容量旋钮压小之后某个崩溃中间态 checker 红或恢复拒」——这一中是写者拒，不是崩溃态，字面归不进 G6；归到 S2 第 4 条「孩子到多少个时分裂、分裂那次多几个单元」没回答（题面自己点名的未定项）。④ 改法：写明根长高（新根 1 个码 2 + 分出的右半 1 个，那一次意图多 2 个 16 KiB 节点 × 2 份；长高要的格式与 inode 树共用，零轮）；或第一版给 unlink 次数设上限并在到达前拒——后者与 D3 已定项 17 字面冲突。

### 5.2 容量旋钮不进盘：N5 判不了「关闭 = 满」（打中，量过）

墓碑正推报告 7.2 表的 N5 写「非最右的类型 1 容器记录数 == 容器容量」，旋钮压小时要 checker 知道旋钮值。旋钮照 `CodeTwoTreeNodeCapacities::CappedForTests` 的形态是写者那一侧的参数，不落盘（推的）。

```
E name=matrix depth=6 cap=2 check_cap=product rootcap=0 ring=6 placement=LC mutation=none states=363791 … N5lit=1206 N5type=1206 …
E name=matrix depth=6 cap=2 check_cap=product rootcap=0 ring=6 placement=LC mutation=close_early states=363791 … N5lit=32773 N5type=32773 …
```

（两行中间省掉的格原样在 `model-output.txt`。）checker 按产品容量判：旋钮 2 条写出来的合法镜像 N5 红 1206 次；按旋钮判：要从镜像外拿旋钮值。把 N5 放宽成「≤ 容量」就红不了「提前关」（`close_early` 那一种改坏）。崩溃中间态：一次发布是 COW、根槽一写定版，模型里发布与发布之间的每个版本 checker 只有上面这几格红；块级的崩溃枚举没跑（第十节）。

四句：① 不分辨谱系住处；分辨「旋钮进盘」与「不进盘」。② checker 看不到旋钮。③ 满足 G6「容量旋钮压小之后……checker 红」字面（合法状态上红）。④ 改法：旋钮值进系统配置一个只供测试的字段（新池新建文件的字节要动）；或 N5 拆成「== 产品容量」（产品镜像）与「≤ 旋钮」（测试镜像）两条、坏镜像各一份（零轮）。

### 5.3 开放容器被重写几次

LC / LO：只有意图那次重写墓碑容器，续做各批、回退都不碰（回退只写谱系容器）；LT：回退那次连墓碑树根一起重写。模型里没数到一次发布重写同一个容器两次。

## 六、G7　新不变量的判别力

判据按字面与收严两种读法各一列（墓碑正推报告 7.2 表）：N2any「队列里每个号都有墓碑」、N2lin 同上但墓碑要在谱系上；N3U「墓碑不指活对象」不过滤、N3L 按谱系过滤（S2 第 14 条要的那一条）；N4lit「同一容器内 (号, 出生代) 不重复」、N4d 换成 (号, 出生代, 死亡 txg)、N4lin「谱系上的墓碑 (号, 出生代) 不重复」；N5lit「非最右的类型 1 容器满」、N5type 按类型各算最右；I95 照 `.claude/kb/invariants.md:279` 全句；I47 照 `.claude/kb/invariants.md:172`。13 种改坏核心的写法各跑深度 6 全枚举（LC，363 791 个状态，`model-output.txt` 的 `name=matrix … placement=LC` 13 行）：

| 改坏 | 红的判据（状态数） | 读法 |
|---|---|---|
| 不改 | N3U 41 395、**N4lit 9 492** | 合法历史上红：N3U 不过滤必红（S2 已改成过滤）；**N4 字面红**（删 → 回退 → 再删同一对象，两条墓碑进同一个开放容器） |
| 意图不写墓碑 | N2any、N2lin 各 70 532 | 只红 N2 |
| 墓碑写在最后一批 | N2lin 70 532（N2any 0） | **N2 不按谱系过滤时不红**：回退进删除中间那一版，树里有原时间线最后一批写的墓碑，不在谱系上 |
| 死亡 txg 写成 txg − 1 | N3L 29 648、N4lin 6 777 | 红到 N3L：回退目标正好是意图之前那一版时，txg − 1 落进谱系 |
| 死亡 txg 写 0 | N2lin 70 532、N4d 9 492（N2any 0） | N2 不过滤时不红 |
| 意图不置 nlink = 0 | N3L 74 438、I95 74 438、N4lin 11 911 | 两条一起红 |
| 回退不写谱系记录 | N3L 41 395、N4lin 9 492 | 红在 N3L（该红的） |
| 谱系记录写 T_old − 1 | N2lin 26 806、N3L 4 585、N4lin 239 | 只在死亡 txg 恰好等于 T_old 时红；N2 不过滤时那 26 806 不红 |
| 墓碑树回退时取 R_old | **一条都不红** | 单镜像判不了（墓碑第一轮判决里「已删完的对象都有墓碑」那笔欠账的同一类） |
| 队列回退时取 cur | I47 79 166 | 只红 I-4.7 |
| 容器提前关 | N5lit、N5type 各 32 773 | 只在 checker 知道旋钮值时（5.2） |
| 续做改死亡 txg | N2lin 30 769、N4d 1 887、N4lin 1 522 | N2 不过滤时不红（G2） |
| 续做多写一条墓碑 | N4lit 131 933、N4lin 75 655（N4d 0） | **N4 换成带死亡 txg 的 (N4d) 就红不了**；N4lin 红、合法历史上不红 |

读法：
1. **N4 字面不收**（判据 G7「改坏之后……红到别的」之外的另一面：合法历史上就红）。合法 + 判别力都要的是 N4lin「谱系上的墓碑 (号, 出生代) 不重复」：不改时 0、多写一条时 75 655。它要谱系，与 N3L 同一份。
2. **N2 要按谱系过滤**：不过滤时「墓碑写在最后一批」「死亡 txg 写 0」「续做改死亡 txg」「T_old − 1」四种改坏它都放过；过滤之后（N2lin）每一种都红，不改时 0。
3. **改坏不红的那一种**：回退时墓碑树取 R_old（墓碑第一轮出局的臂 old）在单镜像 checker 上 18 列全 0。S2 让墓碑随 cur，这一改坏只有扫盘重建（不在出口里）或跨根比较才看得见。
4. 「红到别处」的：死亡 txg − 1 红在 N3L 而不是 N2；意图不置 nlink = 0 红在 N3L 与 I95 两处。按 G7 跑前判据「红到别的不变量 ⇒ 字面不收」，这两种改坏没有专属的一条；我没把这判成哪条字面不收——它们红的是「该红的那一类」（墓碑指活对象），不是无关的一条。

谱系过滤与核心层是不是同一段算法：S2 里核心层只写谱系（回退那次写一条），不读；续做、准入、读者都不按谱系判。checker 的父链走法是唯一的读者，不撞「审计与被审计用同一段代码」（`.claude/rules/fs-design.md:35`）。代价在另一边：父链走法里「其余 txg 的父是 txg − 1」这一步，核心层没有一处显式写它，它是 txg 分配规则（3.2 引的两处）的推论；这一推论被改坏时（例如以后崩溃恢复的新实例第一个根不再取「环里最大 + 1」），checker 会把被抛弃的 txg 当成祖先，没有第二段代码对照（推的）。

## 七、G2、G5 找反例

### 7.1 G2：续做改了死亡 txg，五条判据过（打中，量过）

改坏：可写挂载里的续做（M1）在第一批把这个对象的墓碑记录重写一遍，死亡 txg 换成续做那一次的 txg。

```
E name=g2 placement=LC mutation=cont_redeath lost_roots=1 criterion_3_logical_without_death_txg=True criterion_3_logical_with_death_txg=False criterion_1b_same_image_twice=True criterion_2_done=True checker_red_after_continuation=- rollback_to_intent_then_red=['N2lin'] effective_tombs_of_1_after_rollback_and_continue=0 tombs_of_1_in_tree=1
```

- 第二轮判决的「逻辑内容」写「不含实例号、txg、……」：死亡 txg 若按「txg」排除，③ 过（`criterion_3_logical_without_death_txg=True`）；①、①b、② 都过；④ 不涉及（标签不随墓碑字段变）。崩溃态 checker 全绿（`checker_red_after_continuation=-`）。
- 后果：回退进删除中间那一版（意图那次的根）再续做到底，这个对象的墓碑不在谱系上（`effective_tombs_of_1_after_rollback_and_continue=0`），而它已经删完：谱系上这一次死亡没有墓碑。回退那一刻 N2lin 红，删完之后单镜像 checker 什么都看不见。
- 收严（零轮）：「逻辑内容」显式含墓碑记录的全部字段（含死亡 txg）与谱系记录；「不含 txg」只指根与单元头里的 txg。

续做多写一条墓碑（`cont_dup_tomb`）：③ 在两种读法下都不过（多重集多一条），这一种五条判据够。续做里把开放容器原样重写一次：五条都过，它不是幂等问题，是 W4「有新记录才重写」的写纪律，五条判据不管它；我没造这一种。

### 7.2 G5（推的，没打中）

墓碑树在最后一个对象删完之后不空：「有树、文件集合为空」看的是 inode 树（零条目根）与 extent 树，墓碑树不进这一判；I-9.13 只管类型 2 容器，墓碑容器是类型 1。没找到反例；三棵树的空形态在恢复与只读挂载上认不认，副本里没有实现，没量。

## 八、我提的改法各修哪一格（只在我的模型上量过、被攻过零轮）

| 改法 | G1-a 发布后判被拒 | G1-b 扣两次 | G1-c 窗口 < 顶点 | G1-f 第一个容器 | G3-a 回退被拒 | G6-a 根满 | G6-b 旋钮 | G7 N4 / N2 | G2 死亡 txg |
|---|---|---|---|---|---|---|---|---|---|
| 删除各批不判式子、只判落点 | 修（量过：装置这一档全部做成） | 不修（量过：就是它） | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 |
| 窗口挪出标签 6，作式子里按几何现算的常驻扣项 | 修（推的：删除发布后的标签不再含窗口） | 修字面（推的） | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 |
| W 按盘上最大文件取 | 不碰 | 不碰 | 修（量过：W = 87 时 4263 ≥ 3092） | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 |
| 删除批只重写下段尾叶与祖先 | 不碰 | 不碰 | 修（推的：W 回到不随文件长） | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 |
| 每对象按记录 + 常驻一份开放墓碑容器与根路径 | 不碰 | 不碰 | 不碰 | 修（推的） | 部分（推的：回退抬的只剩 40 字节 / 对象） | 不碰 | 不碰 | 不碰 | 不碰 |
| 谱系容器归固定点、进 ckpt_cost | 不碰 | 不碰 | 不碰 | 不碰 | 修 2–3 槽那一半（推的） | 不碰 | 不碰 | 不碰 | 不碰 |
| 墓碑树根长高写进条款 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 修（推的） | 不碰 | 不碰 | 不碰 |
| 旋钮进系统配置 / N5 拆两条 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 修（量过：按旋钮判时不改 0、提前关 32 773） | 不碰 | 不碰 |
| N4 换成 N4lin、N2 换成 N2lin | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 修（量过：不改 0，四种改坏都红） | 部分（量过：回退进中间那一刻红） |
| 「逻辑内容」显式含死亡 txg 与谱系记录 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 修（量过：③ 变 False） |
| 谱系另立（LC 或 LO），不挂墓碑树下 | 不碰 | 不碰 | 不碰 | 不碰 | 少 1 槽（推的） | 少吃根格（量过：32 309 → 4） | 不碰 | 修 N1 / N5 那一中（量过） | 不碰 |

## 九、没打中的形状

- G1：「(环里根数 + 1)」这个系数：14 个几何行（journal 1 MiB / 32 MiB 到 768 MiB / 4 GiB，S = 4 / 8 / 16，K = 1 与 144）里 W 取同池探针量到的最大一次时顶点全 ≤ 窗口，顶点出现在第 3S 次附近。小盘长环上删除被落点那一判拒：没造出来（保留量 13 × ckpt_cost + 8 在每个几何上都兜住了）。
- G3：按谱系过滤的 checker：LC、LO 深度 7 全枚举 4 243 236 个状态 N3L、N2lin、N4lin、I95、I47、I96、I-9.10 都 0；含崩溃丢 0–3 条根、回退到另一实例的根、来回回退、删 → 回退 → 再删、回退进删除中间再续做。
- G4：谱系记录 16 字节在 txg 唯一性上没找到反例。
- G5：见 7.2。
- G6：一次发布内的崩溃中间态没有枚举（块级），模型层面的版本之间没找到关闭路径的非法态。
- 取样范围：模型 CAP = 1 / 2、根格 3 / 不限、环 6，一个两单元文件 + 至多几个空 inode，深度 6（363 791 状态）与 7（4 243 236）；原型 18 个几何行（14 行 W 取探针、4 行 W 给定）、文件 30–10000 单元。

## 十、这条腿自己的限度与没做什么

- 原型没有墓碑树、`deleted_inodes`、谱系记录，删除批是 m3-delete-r1 攻方原型的从尾部截（inode 1 留着、nlink = 0），W 少算每次 3–4 槽（推的）；准入「判」那一半只调了准入函数，没把谱系容器接进发布路径。
- W 随文件长的那一中是今天 extent 写法的后果；S2 若另写删除专用的 extent 写法，这一中就只剩「W 取哪个数」。最大文件上的窗口与被拒是外推（10000 单元以上内存装不下），没量到一次真被拒。
- 模型是按条款字面写的，不是 crates 的实现；checker 是模型里的 checker，不是 `singlefs-checker`；空间不进模型。
- 崩溃状态一个都没枚举：原型与模型都不调层 0 的枚举函数（`closed_form_state_count` 也没调），0 个状态。
- 没做：G4 的冻结层与态别登记；G5 三棵树空形态的实测；G8 的数（不归这条腿）；I-9.6、I-9.10 的墓碑那一半只在模型里判了「水位 > 墓碑号」与「出生代一致」两句。
- 线程上限中途由主 agent 从 16 改为 10、8、6、5（四次消息）：之后的命令照改；`run-full1` 与 `bkpa3f2tz`、`bpxl7npn7` 是改之前起的（单线程用例）。内存包装两次因同一会话另一条在跑退 252，命令没跑，设 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON` 之后重起；512 MiB 盘一个文件 10000 单元那一行被 journal 记录上限拒（`JournalRecordsOfThePublishExceedTheLimit { records_of_the_publish: 10002, record_limit: 5461 }`），改成 5000 单元。
- 重型测试一个没跑；只跑了 harness 档 `m3_delete_r3_attack_prototype` 这一个测试目标（副本里）与纯 python 模型。副本上的数不是入库装置上的数。

## 十一、什么现象会推翻这份报告的打中

- G1-a：S2 第 10 条「按发布之后的标签抵扣」若本意就是「删除各批不判式子」，那一中归到 G1-b。
- G1-b：若把常驻的环窗口认作与 checkpoint 保留池同类（不是「已承诺预留」里的字节），扣两次那一句的字面不成立。
- G1-c / d：若删除批不走今天的 `plan_the_extent_tree_after_this_publish`（只重写尾叶），W 不随文件长，G1-c 只在 W 取错时成立。
- G3-a：若谱系容器归固定点（不判），被拒那一半不成立，剩「可用为负」。
- G6-a：若条款另写了码 2 根长高（我只查了 D8 已定项 6 与 inode_tree.rs），被拒不成立，只剩「多几个单元」。
- G6-b：若容量旋钮的值落盘，N5 那一中不成立。
- G7 N4：若 N4 的字面本来就带死亡 txg，那一中只剩「多写一条红不了」。
- G2：若「逻辑内容」里「txg」一词本来不含墓碑字段，那一中只是措辞。

## 十二、引文核对与草稿目录

`python3 research/scripts/cite-check.py research/prompts/m3-delete-r3-opus-output.md` 末行：「✓ 核了 14 处引文，对上 14 处，没判 0 处（逐处列在上面）」。

草稿目录 `/tmp/claude-1000/m3-delete-r3-opus/`（仓副本 `tree/`、编译目录 `target/`、复跑目录 `rerun1/`、草稿，合计 1.4G）交回前已删；复跑材料全在模型目录里。
