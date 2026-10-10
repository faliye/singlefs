# m3-leaf-death-r1 攻方腿（Opus）报告（2026-10-08）

攻击面：正文第三节 K1、K2、K3。只读冻结树 `~/.local/share/singlefs/work/m3-leaf-death-r1/tree/` 与 kb；代码引文写冻结树里的全路径:行号，kb 引文写 kb 文件自己的行号。

下面依次是：各格判定一览、复跑与文件、K1、K2、K3、改法表、没打中的形状、这条腿自己的限度、没做什么。

## 各格判定一览

「量过」= 在我的模型或仓副本上跑出来的（不是入库装置上的数，要主 agent 在入库装置上重做才能引）；「推的」= 按条款或代码推、没跑。

| 格 | 候选 / 形态 | 判定 | 依据（详见各节） |
|---|---|---|---|
| K1 | 候选一 退役记录（删空摘掉、合并被吸收的右半各写一条） | E177 主取样点 G-环、读法 R-S：没打中（量过：51 段 inode、`deleted_inodes` r = 0、m = 0；合并历史 64 段 × M2 / M3 两种借法 r = 0、m = 0）。附条件打中两形「误判死亡」：① 管理员回退之后，退役记录照 D18 已定项 11 登记的形态（四元组、不带写序）且退役树随现行那一版走时，杀掉被回退带回来的活容器（量过，手造盘 m = 2）；② K3 若把借法改成「借完给右片改号、出生代不变」，新身份撞上同一次发布出生的已退役身份，活容器整片判死（量过，m = 126） | K1 第 1、5 节；K3 第 2 节 |
| K1 | 候选二 删空照样落零条目版本 | 打中。只落一次（下一次发布摘掉）：G-环 上 F1 n = 900 升序 r 峰值 459，与 E177 的环窗外 459 相同，「退回更旧的版本」一形原样还在（量过）；一直留在树里：inode r = 0，但 I-9.13（类型 2 容器不空）在 1439 个版本上为假，撞 I-9 类判据（量过，计数；checker 判红是推的）；字面不罩「合并被吸收的右半」（推的，那一形今天的读法 r 峰值 182，量过） | K1 第 2 节 |
| K1 | 候选三 类型 2 记录死亡写墓碑 | 打中三形：借今天意图那次的整对象墓碑当死亡记录——「早于死亡 txg 才不收」复活 r = 3、「不晚于死亡 txg 就不收」误判死亡 m = 1（量过）；最后一批另写一条进墓碑树——同一 (号, 出生代) 两条在谱系上，I-9.20 红（副本上跑现成坏镜像用例，checker 判红，量过）；放在自己的区（不碰 I-9.20）——inode 不复活，但合并之后被吸收右半的旧版照样被收成活容器，分配记录、映射 r 峰值 5（量过） | K1 第 3 节、K2 第 2 节 |
| K1 | 候选四 只改择现行、不加盘上东西 | 打中（判别子观测不到）：两段可达历史，末态读得出的码 1 / 码 3 单元逐个相同（组、写序、内容）、最新 txg 相同，一段里 x 活着、一段里 x 已死，9 种几何 / 前缀全是这样（量过） | K1 第 4 节 |
| K1 | 「G-环 上择现行退回更旧的版本」 | 与「不落盘」同一个病：只出在已死的组（活组的现行版本不会被盖）。候选一、三修得掉（r_out 0，量过）；候选二只落一次修不掉（459，量过）；候选四修不掉（同判别子） | K1 第 6 节 |
| K2 | extent、分配记录、映射、文件变短 | 没有一棵要自己的死亡记录：K1 取候选一时，读法加「数据单元按重建出的 inode 记录的对象出生代与 size 折成的单元数判」（下称 Z），E177 51 段与合并 64 段 extent、分配记录、映射 r = 0、m = 0（量过）。打中两形：K1 取候选三时合并之后分配记录、映射 r 峰值 5（量过）；照 D18 已定项 10 给文件变短写区间墓碑，撞 I-9.20「第一版只许整对象」与不重号（副本上跑现成坏镜像用例，量过） | K2 |
| K3 | I-9.4 新判据 N1「左容器最大 key < 右容器最小 key」 | 打中。两份 I-9.4 坏镜像照红（量过）；合法的借在 I-9.12 上照红（分隔 key 照今天 = 容器号，量过）；放过「第三片容器号给小了、分隔 key 照旧」那一形（今天红 I-9.4，N1 下全绿，量过） | K3 第 1 节 |
| K3 | 改借法让旧判据成立（借完右片改号） | 出生代不变：撞号误判死亡（量过，m = 126）；出生代换成这次的 txg：不撞，但每次借多一条退役记录（98 → 253，量过） | K3 第 2 节 |
| K3 | 均分借 vs 借到刚好够 | 两种借法退役记录数相同（122 / 123，量过），候选三的死亡记录相同（45 970，量过）；借的次数 158 对 1 237、删除那次发布重写的叶 46 232 对 47 315（量过）：多出的崩溃点在「刚好够」 | K3 第 3 节 |

## 复跑与文件

模型目录 `research/prompts/m3-leaf-death-r1-opus-model/`：E177 装置 `research/e7-index-bench/src/bin/e177_per_tree_resurrection_without_tombstones.rs`（冻结树里同路径，sha256 `9061c07b…12848f5`）的拷贝改成 `src/main.rs`，加的部分在 `src/leaf_death.rs`；K3 的仓副本用例与补丁在 `k3-copy/`。复跑（仓根，7 个线程；每一阶段都经内存包装，编译 6G、跑 4G）：

```
M=research/prompts/m3-leaf-death-r1-opus-model; S=research/scripts; O=<空目录>
export SINGLEFS_REPO=/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree
bash $S/run-with-memory-cap.sh 6G bash $S/capped.sh 7 bash $M/rerun.sh $O build
bash $S/run-with-memory-cap.sh 4G bash $S/capped.sh 7 bash $M/rerun.sh $O e177 rc indistinguishable rollback
for k in 0 1 2 3 4 5 6 7; do bash $S/run-with-memory-cap.sh 4G bash $S/capped.sh 7 bash $M/rerun.sh $O merges-$k; done
for k in 0 1; do bash $S/run-with-memory-cap.sh 4G bash $S/capped.sh 7 bash $M/rerun.sh $O rename-$k; done
bash $S/run-with-memory-cap.sh 6G bash $S/capped.sh 7 bash $M/rerun.sh $O k3build
bash $S/run-with-memory-cap.sh 4G bash $S/capped.sh 7 bash $M/rerun.sh $O k3run summarize
bash $M/compare.sh $O
```

交回前在空目录 `~/.local/share/singlefs/work/m3-leaf-death-r1-opus/rerun1/` 里按上面从头跑通一次，各阶段挂钟：build 5 秒，e177 + rc + indistinguishable + rollback 32 秒，merges 八批各 88–135 秒（合计约 15 分钟），rename 两批 77、84 秒，k3build 15 秒，k3run + summarize 43 秒。`compare.sh` 比 `summary/` 与 `expected/`，11 份全是 `same`、退 0。另在空目录 `rerun2/` 里重跑 e177、rc、indistinguishable、rollback 四段，排序后 sha256 与 `expected/sorted-output-sha256.txt` 里四行逐个相同（模型确定，线程数不影响输出）。

模型目录每个文件的 sha256（`find . -type f | sort | xargs sha256sum`）：

```
619892e3c1889ded57eaa63318e38a8ac75562fdd255d2014eaa11eed22fab5c  ./Cargo.lock
6a0ccb8237ff75a45e27af6166eeb0161ef7e95d8f53a4a89e4589514bfddedc  ./Cargo.toml
d67b9c61e1c284563758c83770ede06f76a4ccbea517ddf3cf1b3c0b80c3dffd  ./compare.sh
1f4c6d61a94891ba8a4675beda9cd659ef8f51f8f50d7a74c02767a63b8aef75  ./expected/agg-e177.txt
5411f36bd3da059da28e515190a5bdc2de06b822a0ddd4457802c967601319f4  ./expected/agg-merges.txt
b633c4bd66a8e63c8c1dd3ebe448aeb7edb9e6f24832b82871938150077fb8c5  ./expected/agg-rename.txt
c862515e763a6bc5701927b360e6bff33b8bb51ba47f3d9a445fd0d1f949a913  ./expected/counts-e177.txt
8573f03a1218a1460574772ad8d6e13e9414d910d0df676f3566908135ce81d8  ./expected/counts-merges.txt
72d436af9a53bf642b617827323b3e8e14ae26561f71479e1f74857507a282c4  ./expected/counts-rename.txt
590609e536a9dc08b44dd2976118ffe405fb7113fa2ca386ba81e0d0741f44e2  ./expected/indistinguishable.txt
aa6655ca049d4cc21a0125e5f71b764f4f5a1c0fd8ce3af900e4235044db1ddb  ./expected/k3.txt
d92a09236d85a816ea4ec2fcd6c48cb9e5252b7c680d1e13468e3cd1d605c3c0  ./expected/rc.txt
1576ebc3961b29783ef14900dc63910aa5f311d48802e1177fb3091a89f96fb7  ./expected/rollback.txt
419137bbe28d0b6c39021e567f85fa1768d3c49742491aa4c454f037d30d095a  ./expected/sorted-output-sha256.txt
75b8c2009cd5a633b342bc9675f749a5fccc368fd0053b2a927d86d88ef95229  ./k3-copy/opus_k3_borrowed_leaf_images.rs
09fea7845b0dd6e46a20a56d2b36b54a713ee274232e4c8b57eff54757c274d5  ./k3-copy/patch_checker.py
7c304dde73d1a7fc9574f1a2aabc2970aad252a332f7b88cd815d813b6989f12  ./rerun.sh
e7c03620fa0ada1b238a9ff7f63c7c62c77b3fc9e20be3867d6ae2addbd9b752  ./src/leaf_death.rs
ec1e322d7c355504a5d13fd1429deb86ed4921dbdfd45fd8bec9d172a2b7e4e9  ./src/main.rs
ea9ca4c27ca7bb2445b23ff7188790b5b28b7e45c48df72f1f58d4bec159fd0f  ./tools/agg.awk
4f3efbe850942e06591f2d77b7f70275683e8e088fb665026e5df48e05fc8a69  ./tools/counts.awk
```

完整模型输出（每行一个 (历史, 几何, 写法, 读法, 树) 的轨迹）只留在草稿目录 `rerun1/*.out`（e177.out 约 5 900 行、merges 八份各 2 483 行）：太大不入库，`expected/` 里入库的是按格聚合的那几份与每份输出排序后的 sha256，`rerun.sh` 能逐字节复出。

## K1　叶容器与记录的死亡：四个候选

### 0　装置与取样

- 写路径就是 E177 装置那一份（照 `crates/` 今天的规则手写，删空摘掉在 `plan_inode_record_removal`：「if plan.inode_containers[position].records.is_empty() {」（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/research/e7-index-bench/src/bin/e177_per_tree_resurrection_without_tombstones.rs:1957`），对应 `crates/` 的「containers.remove(index.position());」（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-core/src/inode_tree.rs:308`））。我只加：合并策略 M2、M3 均分、M3 刚好够（规则照 E176 装置 `merge_until_no_pair_fits` / `borrow_from_right_if_below_floor` 重写一份，重写清单按 `crates/` 的位置记法）；候选二的两种写法；候选一、三的记录放模型专用区、不占槽（与 E177 臂 P 同一种理想化：只量读侧有了它够不够，盘上载体另算代价）。
- 读法 S 就是 E177 的 R-S。锚：S 在 F1 n = 900 升序、G-环 上的 inode 那一行与 E177 产物逐字段对得上：模型输出「reader=S tree=inode r_peak=460 r_final=460 r_positive_scans=869 r_out_peak=459」（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1-opus/rerun1/e177.out:816`），E177 产物「r_peak=460 r_peak_at=1821 r_positive_scans=869 r_final=460」（`research/results/e177-per-tree-resurrection-without-tombstones-segment-one.out:5289`）。另每次扫描都把 S 与拷来的 E177 `select_current_reading`（R-S、臂 N）逐个名字空间比：1048 次模拟每一次 `leaf_s_differs_from_e177_r_s=0`、`assertion_violations=0`（除了候选二「留在树里」那一写法按定义会违反的 K4_emptied_container_not_written 不计）、`stopped_at=none`（`grep -c` 四个数都是 1048）。
- 取样：E177 第一段的 51 段历史 × G-环、G-宽（G-紧 没跑，见「限度」）。合并历史 64 段（建 700 / 1400 个空 inode，再走 1000 步：建 10% / 30%，覆盖写 5%，顺序写 3%，其余 unlink；unlink 的号 0% / 70% 落在每 50 步换一片的「盯着的」容器里，其余在全部活号里随机挑；种子 1–8）——用户决定的那几步（建多少、删哪些号、什么时候写文件、往哪一片集中删）都在随机里放开，只固定前缀（mkfs、新池新建文件、可写挂载）。每段 × 两个几何 × 五种写法（M1、M2、M3 均分、M3 刚好够、M1 + 候选二只落一次）。挂钟：先跑 20 件估出每件约 1 秒，全量 640 件分八批、每批 80 件（88–135 秒）；没缩历史、没缩几何。这个模型没有崩溃状态（与 E177 一样只有干净发布之后的扫描），见「限度」。

### 1　候选一：退役记录

定义（我量的那一形）：一片容器整片消失（删空摘掉、合并时被吸收的右半）的那次发布，给它的 (容器号, 容器出生代) 写一条退役记录；读法 C1 = S 之外，被退役点名的组整组不收。D8 已定项 6 已有这一形的一半：「同时给旧身份写退役记录」（`.claude/kb/decisions/08-核心索引结构.md:171`），登记表里是「3 退役记录」（`.claude/kb/decisions/18-块里携带什么信息.md:300`），合并时右半退役也已登记在那一行（C113 定案 P5），删空那一形今天走「记录数为 0 的类型 2 容器不落盘」（`.claude/kb/decisions/08-核心索引结构.md:171`），没有记录。

主取样点：没打中。

```
ring M1_dropped C1 inode histories=51 r_peak_max=0 r_out_peak_max=0 histories_r_pos=0 m_peak_max=0 histories_m_pos=0
ring M2_dropped C1 inode histories=64 r_peak_max=0 r_out_peak_max=0 histories_r_pos=0 m_peak_max=0 histories_m_pos=0
ring M3just_dropped C1 inode histories=64 r_peak_max=0 r_out_peak_max=0 histories_r_pos=0 m_peak_max=0 histories_m_pos=0
```

（`expected/agg-e177.txt` 第 5 行；`expected/agg-merges.txt` 第 50、120 行。同一张表里 S 的那几行：E177 历史 G-环 r 峰值 460，合并历史 M2 下 182——合并之后被吸收右半的最后一版还装着后来死掉的记录，今天的读法照样复活。）

附条件打中一（误判死亡，回退）：D23 已定项 14 写「墓碑树、谱系树随现行那一版，不取 R_old」（`.claude/kb/decisions/23-journal的角色与格式.md:378`）。退役记录若照墓碑那样随现行那一版走，而记录照登记表「不带写序；住自己的容器」（`.claude/kb/decisions/18-块里携带什么信息.md:300`），读侧就没有字段判它在不在谱系上。手造盘（`rollback-demo`，G-宽、不经分配器）：R_old = txg 10 时三片 A [1..233]、B (234, 8) [234, 235]、C (467, 10) [467]；被回退掉的那一段把 B 删空、txg 14 写退役 (234, 8)；txg 15 回退到 10，B 回来；回退之后 txg 17 把 C 删空、写退役 (467, 10)，退役容器在新线上重写一版（随现行那一版，就把 txg 14 那条一起带进新版）。

```
name=rollback_demo retirement_tree_at_rollback=carried_from_current select_by_lineage=true retirement_record_carries_death_txg=false resurrected=0 missed=2 missed_inodes=[234, 235]
name=rollback_demo retirement_tree_at_rollback=carried_from_current select_by_lineage=true retirement_record_carries_death_txg=true resurrected=0 missed=0 missed_inodes=[]
name=rollback_demo retirement_tree_at_rollback=taken_from_r_old select_by_lineage=true retirement_record_carries_death_txg=false resurrected=0 missed=0 missed_inodes=[]
```

（`expected/rollback.txt`。读侧已经按谱系择现行——也就是 C615 的读侧补上之后——还是杀掉 234、235；记录带死亡 txg、或退役树在回退那次取 R_old 那一版，就不杀。不按谱系择现行的四行全是 missed=2，那是 C615 本身，所有候选共用。）

附条件打中二（误判死亡，改号借法）：见 K3 第 2 节。

四句：
- 分不分辨臂：回退那一形分辨——候选三的墓碑记录有死亡 txg（D18 已定项 24 第 2 条），按 I-9.18 的谱系读法滤得掉；候选二不涉及。改号那一形只打中候选一（读侧按身份认）。
- 判别子看不看得到：看不到——退役记录四元组里没有任何 txg，记它的容器在随现行那一版重写之后诞生代号是新线的，读侧分不出哪一条是被回退掉的那一段写的。
- 满足的是判据哪一句：正文第五节第一条「误判死亡（活记录被判死）」。
- 改法在打中的格上还中不中：「记录加死亡 txg + 谱系滤」不中（量过，第二行）；「回退那次退役树取 R_old」不中（量过，第三行），但与 D23 已定项 14 给墓碑树定的方向相反，要另立一条；只「读侧按谱系择现行」照中（量过，第一行）。三个改法都只在我的手造盘上量过、被攻过零轮。

崩溃点与落点（推的）：每一次有退役的发布多写退役容器一版与它那棵树的根（照墓碑树 `plan_tombstone_append` 那一形：开放容器有新记录那次才重写）。频度：E177 51 段 20 003 次发布里 63 次（全是删空），合并历史 M2 / M3 110 131–110 283 次发布里 122–123 次（`expected/counts-e177.txt` 第 1 行、`expected/counts-merges.txt`）。落点：`inode_tree.rs` 已交回的「containers_dropped: 1,」（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-core/src/inode_tree.rs:315`）是写退役的触发点；写者、树表登记、checker 的新判据都没有。

### 2　候选二：删空照样落零条目版本

两种写法都量了：「只落一次」= 删空那次发布照样写一版零条目容器（树里留它一版），下一次发布摘掉、不写；「一直留在树里」= 零条目容器不摘。读法 C2 = 一组最新读得出的那一版记录数为 0 就整组死。

打中（仍复活）：只落一次那一写法在 G-环 上把 E177 的复活原样留着。

```
E7RESULT name=leaf_cell family=F1 n=900 order=ascending geometry=ring policy=M1_zero_oneshot reader=C2 tree=inode r_peak=459 r_final=459 r_positive_scans=374 r_out_peak=459 m_peak=0 m_final=0 m_positive_scans=0 m_positive_scans_in_unlink=0 d_peak=899
ring M1_zero_oneshot C2 inode histories=51 r_peak_max=459 r_out_peak_max=459 histories_r_pos=3 m_peak_max=0 histories_m_pos=0
ring M1_zero_oneshot C2 inode histories=64 r_peak_max=21 r_out_peak_max=21 histories_r_pos=10 m_peak_max=0 histories_m_pos=0
```

（第一行 `rerun1/e177.out` 第 868 行；后两行 `expected/agg-e177.txt` 第 40 行、`expected/agg-merges.txt` 第 40 行。G-宽 上同一写法 r = 0：只有会再分配的几何才打中。）机制：零条目那一版在下一次发布被换下、等一环之后槽被再分配，它比这一组更旧的几版不一定被盖（G-环 不按释放先后再分配），择现行又退回装着已死记录的旧版——复活全在环窗外（r_out = r）。要它不复活，零条目那一版必须比同组每一个旧版都活得久，写侧没有条款能保证，只能一直留在树里。

打中（撞 I-9 类判据）：一直留在树里那一写法 inode 不复活（同表第 50 行 `ring M1_zero_persist C2 inode … r_peak_max=0`），但留下的零条目容器就是 I-9.13 判的东西：「任一打包记录类型 2 的容器记录数 ≥ 1」（`.claude/kb/invariants.md:301`），checker 每走到一片就判一次：「self.judgements.judge("I-9.13", !records.is_empty(), || {」（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-checker/src/walk.rs:3056`）。E177 51 段 20 003 次发布里有零条目容器的版本 1 439 个、同时最多 3 片（`expected/counts-e177.txt` 第 3 行 `versions_with_empty_container=1439 empty_peak=3`）；这些版本上 I-9.13 为假是按判据字面推的，我没在 `crates/` 上造这种镜像。它们还占一个根的 135 条条目，删空的位置不回收（推的）。

字面没罩的一形：候选二说的是「删空」，合并时被吸收的右半不是删空。照字面，它不写零条目版，最后一版照旧可读——就是今天 M2 / M3 下 S 那一格：`ring M2_dropped S inode histories=64 r_peak_max=182`（`expected/agg-merges.txt` 第 80 行）。给被吸收的右半也写零条目版，又回到上面两条：只落一次在 G-环 上复活，一直留着撞 I-9.13。

四句：
- 分不分辨臂：分辨。同一批 G-环 历史上候选一、三 r = 0（第 1、3 节），只有候选二（只落一次）还复活；一直留在树里那一写法撞的 I-9.13 别的候选都不碰。
- 判别子看不看得到：看得到——零条目那一版读得出时读侧确实认得出；打中的是它读不出之后（被再分配盖掉），读侧拿不到任何东西。
- 满足的是判据哪一句：只落一次——正文第五节第一条「仍复活」；一直留着——任务里「撞上 I-9 类判据」那一句。
- 改法：「零条目版本一直留着」修得了复活（量过 r = 0），在 I-9.13 上照中；「改 I-9.13 允许零条目」是改不变量原文，交用户（推的，没量）。

### 3　候选三：类型 2 记录死亡写墓碑

三种落法：

（a）不另写，借今天意图那次写的整对象墓碑当死亡记录。墓碑的死亡 txg 是意图那次的 txg；读侧拿它比供出这条记录的容器版本的诞生代号，只有两种比法：

```
ring M1_dropped C3_intent_lt inode histories=51 r_peak_max=3 r_out_peak_max=2 histories_r_pos=48 m_peak_max=0 histories_m_pos=0
ring M1_dropped C3_intent_le inode histories=51 r_peak_max=1 r_out_peak_max=0 histories_r_pos=10 m_peak_max=1 histories_m_pos=51
```

（`expected/agg-e177.txt` 第 25、20 行。）「诞生代号 < 死亡 txg 才不收」：删空那片最后一版就是意图那次写的（nlink = 0 那一版，诞生代号 = 死亡 txg），收下，复活 3（F1 n = 900 的三片）。「≤ 就不收」：意图到最后一批之间那条 nlink = 0 的活记录被压掉，51 段每段都误判死亡 1（E177 臂 B 的 m = 1 是同一件事）；F8 那 10 段还复活 1——1 号的记录在各批里被重写（诞生代号 > 意图 txg），删空之后那一版收下。两种比法一个复活、一个误判死亡，意图那次的墓碑不够。

（b）最后一批另写一条墓碑进墓碑树。I-9.18 要求队列里的号在意图那次起就有谱系上的墓碑（`.claude/kb/invariants.md:306`），意图那条不能挪到最后一批，只能再加一条；同一 (号, 出生代) 两条都在谱系上，正是 I-9.20 判的「按 (号, 对象出生代) 不重复」（`.claude/kb/invariants.md:308`）。仓副本上跑现成用例 `two_tombstones_of_one_object_on_the_lineage_redden_only_the_tombstone_record_invariant`：`I920 test … ... ok`（`expected/k3.txt`），即 checker 在这种镜像上只红 I-9.20。候选三这一落法的每一张删完的镜像都是这张坏镜像的形状（推的：用例追加的是原样一条，候选三那条死亡 txg 不同、(号, 出生代) 相同，判据只看后者）。

（c）放在自己的区（新记录类型 / 新树，不碰 I-9.20；我量的就是这一形，死亡 txg = 记录离开树的那次发布）。inode 与 `deleted_inodes` 在 E177 与合并历史上都是 r = 0、m = 0（`expected/agg-e177.txt` 第 15 行、`expected/agg-merges.txt` 第 95 行）。但它只说「这条记录死了」，不说「这片容器没了」：合并被吸收的右半，最后一版里的记录没死（挪到左片了），读侧照样把那片收成一片活容器，它的槽与映射 key 进重建出的分配记录树与映射树——见 K2 第 2 节（r 峰值 5）。

四句（对 (a)）：分辨（候选一、(c) 在同一批历史上 r = m = 0）；判别子看不到——删空那片的最后一版与「还在删除途中」那一版是同一个单元、同一个诞生代号，墓碑的死亡 txg 也相同，读侧在两种情形下拿到的输入逐条相同；满足「仍复活」与「误判死亡」两句；改法（b）在 I-9.20 上照中、（c）在 inode 上不中、在合并后的分配记录 / 映射上照中。

### 4　候选四：只改扫描重建的择现行，不加盘上东西

打中（判别子观测不到）。造两段可达历史，前缀相同（mkfs、新池新建文件、可写挂载、建 n 个空 inode，x = n 恰好一个人在最右那片）：

- A：unlink x 只发到意图那次，再发一次零单元的空发布（可写挂载的暖机、会话里为空间推的抬 F 都是这种发布；一次 unlink 的意图与各批之间可以夹着它们：「unlink 之后走空的删除步；与推的抬 F 的空发布按 txg 交错」（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-core/src/mounted_session.rs:82`））——x 还活着（nlink = 0、在队列里）；
- B：unlink x 发完意图与最后一批——那片删空、不落盘，x 已死。

两段末态 txg 相同；读得出的码 1 / 码 3 单元按（属主、组、写序、内容）排序之后逐个相同：

```
E7RESULT name=indistinguishable geometry=ring prefix_overwrites=0 inode_count=234 x=234 latest_txg_a=12 latest_txg_b=12 code1_code3_readable_units_a=11 code1_code3_readable_units_b=11 units_identical=true x_alive_a=true x_alive_b=false reader_s_reads_x_a=true reader_s_reads_x_b=true
```

（`expected/indistinguishable.txt` 第 1 行；九行——G-宽、G-环、G-环 前缀先覆盖写 40 次 × n = 234、467、700——全是 `units_identical=true x_alive_a=true x_alive_b=false`。）B 那次发布写出的只有码 2（inode 树根、分配记录树、记账、映射）、树表与根记录；`deleted_inodes` 删空了也不写。所以只读码 1 / 码 3 与根环 txg 的读法，对 A、B 只能给同一个答案：答 x 活，B 上复活；答 x 死，A 上把一条活记录判死。

四句：
- 分不分辨臂：分辨（候选一、二、三都在 B 上多写一样东西）。
- 判别子看不看得到：看不到，就是这一格。要分开只能读码 2：B 的 inode 树根是新的一版、不再指 x 那片。码 2 不进择新规则（D18 已定项 12 那张表「择新键」一行码 2「不进规则」），给码 2 补择版本规则的已定项 15 已撤回；读码 2 等于让派生态当死亡的权威。
- 满足的是判据哪一句：两句都满足，读侧选哪一种答法决定落在「仍复活」还是「误判死亡」。
- 改法：「只改读侧」本身就是被打中的那一形；能让它不中的改法都要往盘上加东西（回到候选一、二、三）。推的：抬 F 的空发布在 B 那一侧前面先写系统配置（F 变了），A、B 的系统配置槽会不同；那一份只记 F，不点名 x，不改上面的结论；挂载暖机那一形没有这处差异，但 A 那一侧要多一次暖机，暖机次数「至多 R 次」由什么定我没查到。

### 5　候选一的另外两处（没打中，写清扫了什么）

- 崩溃重发的 txg 与 inode 号：同身份 (容器号, 出生代) 在被抛弃的那次发布里退役、新线上活着，靠已发布谓词 P2 把被抛弃的退役容器滤掉（推的：退役记录住码 3 容器，带写序）。这个模型没有崩溃，没量。
- 跨头首次 COW 的重生：单头取样，没有对象。

### 6　「G-环 上择现行还会退回更旧的版本」

同一个病。活着的组，现行那一版不会被释放、不会被盖，择到的就是它；退回更旧的版本只出在已死的组（删空摘掉、被吸收的右半）：最后一版被再分配盖掉，更旧的几版还在。E177 的 inode 460 里 459 在环窗外（产物第 37322 行 `r_in_peak=3 r_out_peak=459`），我模型的 S 同（`rerun1/e177.out` 第 816 行）。各候选：

| 候选 | G-环 r_out 峰值（E177 51 段 / 合并 64 段） | 出处 |
|---|---|---|
| S（今天） | 459 / 182 | `expected/agg-e177.txt` 第 35 行；`expected/agg-merges.txt` 第 80 行（M2） |
| 候选一 C1 | 0 / 0 | 第 5 行；第 50、120 行 |
| 候选二 只落一次 | 459 / 21 | 第 40 行；第 40 行 |
| 候选三 (c) | 0 / 0 | 第 15 行；第 95 行 |
| 候选四 | 同 S（读侧没有输入） | 第 4 节 |

候选一按身份整组不收，旧版装什么都不读；候选三逐条看死亡记录，旧版里的记录要么死了（有记录）、要么挪到别片还活着（不算复活）。候选二只落一次，零条目那一版自己也会被盖，盖掉之后与今天一样。

## K2　臂 B 罩不住的 extent、分配记录、中央映射与文件变短

### 1　我认为够的那一形：不给这几棵树写死亡记录，用 inode 记录判

形态（读法 Z）：K1 取候选一之后，重建出的 inode 树里每个对象取一条记录（同号几条时取诞生代号最大的那一版供出的）；数据单元只在「对象出生代与记录偏移 8 相等，且锚点 < size 折成的单元数」时收下；分配记录树、映射树从收下的数据单元与容器重算（E177 的 R-S 本来就这样算这两棵）。权威态是 inode 记录（类型 2，权威态单元）与数据单元头的五元组；size 在每一批都改：「inode 记录的 size 改成 `size_after_the_batch_in_bytes`」（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-core/src/unlink.rs:104`），覆盖写、顺序写写的也是新长度。扫描重建拿得到它，条件是 K1 先让死掉的 inode 记录不复活（否则复活的旧记录带着旧 size 把数据一起带回来）。

量过，没打中：

```
ring M1_dropped C1Z extent histories=51 r_peak_max=0 r_out_peak_max=0 histories_r_pos=0 m_peak_max=0 histories_m_pos=0
ring M1_dropped C1Z allocation_records histories=51 r_peak_max=0 r_out_peak_max=0 histories_r_pos=0 m_peak_max=0 histories_m_pos=0
ring M1_dropped C1Z central_mapping histories=51 r_peak_max=0 r_out_peak_max=0 histories_r_pos=0 m_peak_max=0 histories_m_pos=0
ring M3half_dropped C1Z extent histories=64 r_peak_max=0 r_out_peak_max=0 histories_r_pos=0 m_peak_max=0 histories_m_pos=0
ring M3half_dropped C1Z allocation_records histories=64 r_peak_max=0 r_out_peak_max=0 histories_r_pos=0 m_peak_max=0 histories_m_pos=0
```

（`expected/agg-e177.txt` 第 9、6、7 行；`expected/agg-merges.txt` 第 89、86 行。G-宽、M1 / M2 / M3 刚好够的同几行同为 0。）文件变短那一形（E177 Q11_OU 的 F8 e = OU）单看一段：

```
E7RESULT name=leaf_cell family=F8 n_u=300 k=0 e=OU batch=1 geometry=ring policy=M1_dropped reader=C1 tree=extent r_peak=300 r_final=300 r_positive_scans=3 r_out_peak=0 m_peak=0 m_final=0 m_positive_scans=0 m_positive_scans_in_unlink=0 d_peak=300
E7RESULT name=leaf_cell family=F8 n_u=300 k=0 e=OU batch=1 geometry=ring policy=M1_dropped reader=C1Z tree=extent r_peak=0 r_final=0 r_positive_scans=0 r_out_peak=0 m_peak=0 m_final=0 m_positive_scans=0 m_positive_scans_in_unlink=0 d_peak=300
```

（`rerun1/e177.out` 第 4648、4668 行。）只有候选一不够（r 300），加上按记录判数据单元才到 0；也不出 E177 臂 B 那种「意图那次就生效、压掉还活着的数据单元」（m = 0）——Z 不读墓碑。

### 2　打中的两形

（a）K1 取候选三 (c) 时，合并之后分配记录与映射复活。

```
ring M3half_dropped C3Z allocation_records histories=64 r_peak_max=3 r_out_peak_max=3 histories_r_pos=41 m_peak_max=0 histories_m_pos=0
wide M2_dropped C3Z allocation_records histories=64 r_peak_max=5 r_out_peak_max=5 histories_r_pos=41 m_peak_max=0 histories_m_pos=0
```

（`expected/agg-merges.txt` 第 106、221 行；映射同数。M1 下 0：不合并就没有「记录活着、容器已死」的片。）被吸收右半的最后一版里那些记录挪到左片了、没死，记录级的死亡记录一条都不点它，读侧把那片当活容器，它的槽进分配记录、它的映射 key 进映射。四句：分辨（同一批历史 C1Z 为 0）；判别子看不到——记录级的死亡记录说不出容器死了；满足「仍复活」；改法「候选三再加容器级退役」就是候选一，不中。

（b）照 D18 已定项 10 给文件变短写区间墓碑（「truncate / punch 这类部分删除按每段连续 extent 区间一条」那一句；E177 Q11_OU 那一形最直接的补法）。撞 I-9.20：「第一版只许整对象 [0, 2⁶⁴ − 1]」（`.claude/kb/invariants.md:308`），同一对象意图那次已有整对象墓碑，再写区间墓碑又是同 (号, 出生代) 两条。仓副本上跑现成用例 `a_tombstone_that_does_not_cover_the_whole_object_reddens_only_the_tombstone_record_invariant` 与上面那条不重号用例，都是 `... ok`（`expected/k3.txt`）：checker 在这两种镜像上都只红 I-9.20。要走区间墓碑，I-9.20 两句都要先改，交用户（推的）。

### 3　T1、T2 怎么改写（推的，代价只给了次数）

| 候选 | 定义 | 多写什么 | 频度（量过的次数） | 碰的崩溃点（推的） |
|---|---|---|---|---|
| T1-改 | inode 叶容器整片消失（删空、被吸收）写容器级退役记录（候选一，记录要带死亡 txg，K1 第 1 节）；类型 2 记录、extent、分配记录、映射、`deleted_inodes` 不写死亡记录，读侧按 Z 判；记账读现行值（E177 量到 0）；墓碑树、谱系树只增 | 每次有退役的发布：退役容器一版 + 它那棵树的根 | E177 51 段：20 003 次发布里 63 次；合并历史 M2 / M3：约 11 万次里 122–123 次（`expected/counts-*.txt`） | 删除最后一批那一次、合并那一次多两刀写（照墓碑树的形状） |
| T2-改 | T1-改 + 今天没有对象的 dirent、反向索引、空闲空间、LRU 各自定死亡形态 | 不可量（`crates/` 没有写者，E177 Q7） | — | — |

两条都要先定：退役记录的格式（加死亡 txg 是 incompat，类型 3 冻结前零代价，D18 登记表那一行原话）、退役树在管理员回退时随谁（K1 第 1 节）、扫描重建怎么读谱系（C615）。

### 4　Z 没打中的形状（扫了什么）

E177 51 段（含文件变短 16 段、各批截短、覆盖写 k = 5 次再删）与合并 64 段（覆盖写、顺序写 1–200 个单元、删 1 号）× G-环、G-宽；只有 1 号有数据，1 号住最左那片、合并与借都挪不动它。没试：一个对象的记录被借走或被吸收之后它的数据单元怎么判（模型里没有住在会被挪动那几片里的带数据文件）；有洞的文件（`crates/` 今天没有 punch，「锚点 < 单元数 ⇒ 活」在有洞时不成立，推的）；快照与克隆头共享数据单元而两头 size 不同（单头取样，D8 已定项 6 的重建射程本来就是单头）。

## K3　C536：I-9.4 的新判据与借多少

### 1　新判据 N1：把第三句「左容器的最大 key < 右容器号」换成「左容器的最大 key < 右容器的最小 key」

今天的第三句：「且左容器的最大 key < 右容器号」（`.claude/kb/invariants.md:292`），checker 那一行「.judge("I-9.4", left.largest_inode < right.container, || {」（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-checker/src/walk.rs:2782`）。在仓副本里给它加一个开关（设了 `OPUS_K3_I94_N1` 就比右容器的最小 key，`k3-copy/patch_checker.py`），同一个测试二进制开关前后各跑一遍。镜像在 `crates/` 写出的一版上改（`k3-copy/opus_k3_borrowed_leaf_images.rs`）：1 号 + 466 个空 inode，三片 [1..233]、[234..466]、[467]；左片删到 [1..100]（当作 101..233 已删完）；再照 M3 均分借（右片最小的 66 条接到左片）与刚好够（17 条）。

```
OPUSK3 image=borrow_half_separator_is_container_number n1_rule=false leaves=[no=1 1..299 n=166][no=234 300..466 n=167][no=467 467..467 n=1] separators=[1, 234, 467] violated=["I-9.4", "I-9.12"]
OPUSK3 image=borrow_half_separator_is_container_number n1_rule=true leaves=[no=1 1..299 n=166][no=234 300..466 n=167][no=467 467..467 n=1] separators=[1, 234, 467] violated=["I-9.12"]
OPUSK3 image=borrow_half_separator_is_new_smallest n1_rule=true leaves=[no=1 1..299 n=166][no=234 300..466 n=167][no=467 467..467 n=1] separators=[1, 300, 467] violated=[]
OPUSK3 image=third_leaf_numbered_300_separator_467 n1_rule=false leaves=[no=1 1..233 n=233][no=234 234..466 n=233][no=300 467..467 n=1] separators=[1, 234, 467] violated=["I-9.4"]
OPUSK3 image=third_leaf_numbered_300_separator_467 n1_rule=true leaves=[no=1 1..233 n=233][no=234 234..466 n=233][no=300 467..467 n=1] separators=[1, 234, 467] violated=[]
```

（`expected/k3.txt`；刚好够那两行同形；没借的两张镜像开关前后都是 `violated=[]`。）

- 不放过今天那两份 I-9.4 坏镜像：`checker_known_bad_images.rs` 里 I-9.4 只有两份——容器号改成 5 而记录是 1（表驱动用例 `the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target` 里那一项），与同一片叶里 1 号两条（`a_leaf_holding_the_same_inode_number_twice_reddens_only_the_container_number_invariant`）。开关前后两条用例都 `... ok`（`expected/k3.txt` 的 KNOWNBAD 行）：它们打的是第一句与第四句，N1 没碰。
- 打中一（把合法的借判红）：C536 只说 I-9.4，借之后 I-9.12 也红。I-9.12 的判据「第 i 条的分隔 key ≤ 第 i 个孩子的 key 区间下界，且 > 第 i−1 个孩子的 key 区间上界」（`.claude/kb/invariants.md:300`），而今天的分隔 key 就是容器号：「self.identity.container」（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-core/src/inode_tree.rs:83`），D8 的定义是「该孩子创建时的最小 key」（`.claude/kb/decisions/08-核心索引结构.md:164`）。右片的分隔 key 234 不大于左片借完的最大 key 299，I-9.12 红（第一、二行）。分隔 key 改成右片借完的最小 key 才两条都绿（第三行）——这一改动碰 D8 已定项 6 的分隔 key 定义，也碰同一行「查找取最后一个分隔 key ≤ 目标 key 的条目」：分隔 key 不改时按这条规则找 234..299 会走到右片、找不到（模型数：M3 均分下按容器号路由走错片的记录峰值 198、刚好够 137，`expected/counts-merges.txt` 的 `misrouted_peak`；`crates/` 的可写读今天逐片全读、不按分隔 key 路由——「for entry_bytes in &inode_root.entries {」（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-core/src/mounted_read.rs:480`）——所以今天读不丢，按 D8 的查找规则实现就丢，推的）。
- 打中二（放过坏镜像）：第三片容器号写成 300（落在中间那片 key 区间里、大于中间那片的号），分隔 key 留 467。今天红 I-9.4，N1 下一条不红（第四、五行）。这是 C116 那一族「分裂出来的右半容器号不是触发分裂那条新记录的 inode 号」（`crates/mutations.tsv` 第 194 行那条变异把号写成 0；写成 300 这一种只有第三句抓得到）。借合法之后「容器号 < 片内最小 key、左片最大 key ≥ 右容器号」本来就是合法形状，单镜像上分不开「借过」与「分裂时号给错」：这一格只能回到 C116 说的单测、模型对拍与崩溃点重放。分隔 key 跟着写成 300 的那一形 N1 下还有 I-9.12 与 I-1.1 红（`expected/k3.txt` 第 26 行），分隔 key 与容器号脱钩之后就没有了（推的）。

四句：分辨（今天的判据与 N1 在打中二上给出不同答案）；判别子看得到（容器号、key 区间都在镜像上），被打中的是判据的写法；满足的是任务 K3「放过坏镜像 / 把合法的借判红」两句；改法「分隔 key 随借更新」修打中一、在打中二上不起作用。

### 2　改借法让旧判据成立：借完给右片改号

改法：借完右片的容器号改成它借完之后的最小 key，旧身份写退役记录（不写，旧身份那几版里的记录将来死了会复活）。两种出生代：

```
E7RESULT name=leaf_cell family=RC geometry=wide policy=M3half_rename_keep_birth_dropped reader=C1 tree=inode r_peak=0 r_final=0 r_positive_scans=0 r_out_peak=0 m_peak=126 m_final=126 m_positive_scans=1 m_positive_scans_in_unlink=0 d_peak=340
E7RESULT name=leaf_cell family=RC geometry=wide policy=M3half_rename_new_birth_dropped reader=C1 tree=inode r_peak=0 r_final=0 r_positive_scans=0 r_out_peak=0 m_peak=0 m_final=0 m_positive_scans=0 m_positive_scans_in_unlink=0 d_peak=340
```

（`rerun1/rc.out` 第 81、153 行；G-环 两行同数。）历史（逐步写死在 `LeafHistory::RenameCollision`）：一次发布建 2..700，后三片 (234, T)、(467, T)、(700, T) 出生代相同；删 699，Y = (467, T) 与 Z = (700, T) 之和 233，Y 吸收 Z、Z 退役；删 477..698；建 125 个（701..825，进最右的 Y）；把 X = (234, T) 从 233 删到 116，X + Y = 252 不并，X 从 Y 均分借 (136 − 116) ÷ 2 = 10 条（467..476），Y 借完最小 key 是 700，改号成 (700, T)——撞上已退役的 (700, T)，读法 C1 把这片活容器整片不收，126 条活记录判死。这与 E104 打出的「重生取最小 inode 号当容器号会撞车」是同一族。出生代换成这次的 txg 就不撞（第二行），代价是每次借多一条退役：随机 32 段 155 次借，退役 98 → 253（`expected/counts-rename.txt`）。

四句：分辨（不改号的 M3 均分同一段 m = 0，`rerun1/rc.out` 第 9 行）；判别子看不到——退役记录只点名 (号, 出生代)，撞号之后新旧两片在读侧是同一个身份；满足「误判死亡」；改法「改号换出生代」在打中的格上不中（量过），但给每次借添一条退役记录与 K1 的崩溃点。

### 3　均分借与借到刚好够下限

| 量 | M3 均分 | M3 刚好够 | 出处 |
|---|---|---|---|
| 合并历史 64 段、G-环：退役记录（候选一） | 123 | 122 | `expected/counts-merges.txt` |
| 记录死亡（候选三） | 45 970 | 45 970 | 同上 |
| 借的次数 | 158 | 1 237 | 同上 |
| 删除那次发布重写的叶（位置记法，合计） | 46 232 | 47 315 | 同上（M1 45 948、M2 46 093） |
| 每千次删除借几次：随机删 / 70% 集中删一片 | 1.7 / 5.2 | 11.0 / 42.8 | 下面那条 awk 的输出 |
| 一段历史里最多借几次：随机删 / 集中删 | 8 / 14 | 68 / 194 | 同上 |
| 第三句为假的版本数 | 17 000 | 15 091 | `expected/counts-merges.txt` |

```
M3half_dropped focus=0 runs=32 borrows=39 removals=22972 merges=47 rewritten_in_removals=23059 borrows_per_1000_removals=1.7 max_borrows_in_one_history=8
M3half_dropped focus=70 runs=32 borrows=119 removals=22998 merges=56 rewritten_in_removals=23173 borrows_per_1000_removals=5.2 max_borrows_in_one_history=14
M3just_dropped focus=0 runs=32 borrows=252 removals=22972 merges=47 rewritten_in_removals=23269 borrows_per_1000_removals=11.0 max_borrows_in_one_history=68
M3just_dropped focus=70 runs=32 borrows=985 removals=22998 merges=57 rewritten_in_removals=24046 borrows_per_1000_removals=42.8 max_borrows_in_one_history=194
```

（`cat rerun1/merges-*.out | grep name=leaf_counts | grep geometry=ring | grep policy=M3 |` 按 policy 与 focus 累加的 awk，原样输出。）

- K1 的死亡记录：借不退役，两种借法的退役数与记录死亡数一样；借多少不让候选一、三多写。改号借法（第 2 节）才让退役数跟着借的次数走。
- 崩溃点：一次借让那次删除的发布多重写一片（借方本来就重写，多的是出借方），每多一片多一刀写（推的：照 `crates/` 每个单元一次写请求）。刚好够借得多 7.8 倍，集中删一片时每千次删除 42.8 次借——借到 117 的那片下一次删就又低于下限，又借。
- 哪一种代价界好证：两种的单次上界一样（内容记法 ≤ 2 片加合并，E176 的丙类断言）。频度上，刚好够借完借方恰在 117，下一次删同一片就再借，最坏是每次删除都借；均分借完借方到 x + ⌊(r − x) ÷ 2⌋，两片之和恰为 234 时也只到 117（例：借方 116、出借方 118，借 1 条），最坏情形与刚好够相同。所以最坏的频度界两种一样，差在平均：量过的每千次删除 1.7 对 11.0（随机删）、5.2 对 42.8（集中删）。要给均分证一个比刚好够好的频度界，得先给「两片之和」一个大于 234 的下界，今天的合并规则只保证 ≥ 234（推的）。
- E176 的「不稳定」翻在可忽略 / 不翻两档之间，与这里同向：刚好够那一侧的多重写来自借的频度，不来自单次。

## 改法表（我自己提的改法：只在我的模型或仓副本上量过、被攻过零轮）

| 改法 | 修哪一格 | 在打中的那几格上 | 量过 / 推的 |
|---|---|---|---|
| 候选一 + 退役记录带死亡 txg + 读侧按谱系滤（与 I-9.18 对墓碑同一读法） | K1 候选一回退那一形 | 不中（`expected/rollback.txt` 那一行 missed=0） | 量过（手造盘）；`crates/` 上没量 |
| 候选一 + 回退那次退役树取 R_old | 同上 | 不中（missed=0）；与 D23 已定项 14 给墓碑树的方向相反 | 量过（手造盘） |
| 候选一 + 读侧按谱系择现行（C615 的读侧），记录不带 txg | 同上 | 照中（missed=2） | 量过（手造盘） |
| 候选二一直留零条目容器 | K1 候选二复活 | 复活不中（r = 0）；I-9.13 照中（1 439 个版本） | 量过（计数）；I-9.13 判红是推的 |
| 候选三写进自己的新记录类型 / 新树 | K1 候选三 (a)(b) | inode 不中（r = m = 0）；合并后分配记录、映射照中（r 峰值 5） | 量过 |
| K2 读法 Z（K1 取候选一） | E177 Q11_OU、extent / 分配 / 映射 | 不中（全 0） | 量过 |
| K2 读法 Z（K1 取候选三） | 同上 | 合并后分配、映射照中 | 量过 |
| I-9.4 第三句换成 N1 | 合法的借在 I-9.4 上判红 | I-9.4 不中；I-9.12 照中（分隔 key = 容器号时） | 量过（仓副本） |
| N1 + 分隔 key 随借改成右片最小 key | 合法的借在 I-9.4、I-9.12 上判红 | 两条都不中；「号给小了」那一形放过 | 量过（仓副本，分隔 key 是手改的，`crates/` 没有这条写路径） |
| 借完右片改号、出生代不变 | 让旧第三句成立 | 撞号误判死亡 m = 126 | 量过 |
| 借完右片改号、出生代换成这次 txg | 同上 | 不中（m = 0）；每次借多一条退役 | 量过 |

## 没打中的形状

- 候选一在 E177 51 段 × G-环 / G-宽、合并 64 段 × 两个几何 × M1 / M2 / M3 均分 / M3 刚好够、改号两种 32 段（G-宽）上：inode、`deleted_inodes` r = m = 0。没试：崩溃支线、跨头重生、副本坏一份（`crates/` 的 w = 2）。
- 候选三 (c) 在同一批上 inode r = m = 0。
- Z（取候选一）在同一批上 extent、分配记录、映射 r = m = 0；形状与取样范围见 K2 第 4 节。
- K3 两份 I-9.4 坏镜像在 N1 下照红（不放过）。
- 随机 32 段 × 改号保留出生代没撞上号（155 次借、0 次误判死亡），撞号那一形是按条件逐步造出来的：随机取样够不着不说明撞不上。

## 这条腿自己的限度

- 模型是 E177 装置的拷贝，读侧是手写的择现行，不是 `crates/` 的扫描重建（`crates/` 没有）；只有干净发布之后的扫描，没有崩溃状态、没有坏镜像、只有单头。G-紧 没跑：它的盘按 G-宽 的峰值定，加合并之后要重算那条规则，我没做。层 0 的崩溃状态一个都没枚举（任务里的崩溃点都是「推的」）。
- 候选一、三的记录放在模型专用区、不占槽：量的是「读侧有了它够不够」，载体本身（容器、树根、会不会被盖、要不要回收）没进模型；它们自己的复活与回收问题（D18 已定项 10、E104 的 P5）没量。
- 只有 1 号有数据，住最左那片，合并与借从不挪它；Z 的取样没有「带数据的记录被挪走」那一形。
- 回退只在手造盘上（8 个单元），不经分配器；`crates/` 的回退写路径没跑。
- K3 的镜像是在 `crates/` 写出的一版上手改字节（删 133 条、挪记录、改分隔 key），不是 `crates/` 的借写路径写出来的（它没有）；「删完的 101..233 没有墓碑」这一处与真的删除不同，单镜像 checker 不判它（C614 那一格）。
- 合并历史只有一种生成器（建 / 删 / 覆盖写 / 顺序写的比例写死、集中删每 50 步换片），种子 1–8；「没打中」都是一次抽样。

## 没做什么

- 没跑重型测试；harness 档只在仓副本上跑了 4 个测试目标里的 6 条用例（`k3run` 阶段），没跑别的 harness 用例。
- K4 不归我；正推那几格没判。
- 副本上的数没有在入库装置上重做：要引，主 agent 拿入库的 E177 装置加臂重跑。
- 候选一的写者落点、checker 新判据、退役树的树表登记，只写了「推的」，没写代码。

## 草稿与副本

- 草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1-opus/`：`rerun1/`（交回前从头跑通的那一次：`*.out` 全量模型输出、`k3.out`、`summary/`、`k3build.log`、`k3run.log`，报告里引的 `rerun1/…` 行号都在这里）、`rerun2/`（确定性复跑的四份输出）、`out-*.txt` 与 `agg-*.txt`（开发中的旧版输出，旧版模型少两种读法，不作依据）、`progress.md`。编译目录与仓副本（`target/`、`repo/`、`repo-target/`、`rerun1/target/`、`rerun1/repo/`、`rerun1/repo-target/`、`rerun2/target/`、开发用的 `model/`）交回前已删。
- 没入库的：完整模型输出（太大，`rerun.sh` 逐字节复出，`expected/sorted-output-sha256.txt` 钉着）。
