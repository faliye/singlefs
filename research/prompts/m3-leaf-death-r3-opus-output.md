# m3-leaf-death-r3 攻方腿（Opus）报告（2026-10-08）

攻击面：正文第三节 K1、K2、K3（最终轮）。只读冻结树 `~/.local/share/singlefs/work/m3-leaf-death-r1/tree/` 与 kb；草稿目录 `~/.local/share/singlefs/work/m3-leaf-death-r3-opus/`。代码引文写冻结树里的全路径:行号；kb 引文写 kb 文件自己的行号或 `文件@小节`。
不重复前两轮攻过的角度（正文第一节逐条列的那些）。这一轮攻的是三处没被看过的地方（克隆头共享叶、副本坏一份 w = 2、清扫删实例表行）、读法 Z 的另一组历史、区间墓碑最小改法放过的坏镜像、删 I-9.4 第三句之后放过的坏镜像。

状态：完成（分段落盘）。

## 各格判定一览

「量过」= 在我的模型或仓副本上跑出来的（不是入库装置上的数，主 agent 要在入库装置上重做才能引）；「推的」= 按条款或代码推、没跑。「没打中」都是一次抽样。

| 格 | 形态 | 判定 | 依据 |
|---|---|---|---|
| K1 克隆头 | 候选二字面（零条目版本一直留，不写退役记录） | **打中复活**（量过）：分叉后在克隆头 unlink 一个共享叶里的号，那片叶跨头首次 COW 换了身份，origin 身份那一版（≤ F）没有任何东西压掉，号在克隆头复活；984 554 次判定里 500 463 次 r > 0、峰值 3 | K1.1 |
| K1 克隆头 | 候选二补「跨头首次 COW 写退役记录」/ 补「号接替」读法 | 没打中（量过，两个补法都被攻过零轮）；前者把退役载体带回候选二 | K1.1 |
| K1 克隆头 | 候选一（甲乙同一读法，克隆模型没有回退）、TSZ | 没打中（量过）：前提 A（有克隆祖先表）下 r = m = 0，TSZ 只有 unlink 途中那一格 | K1.1 |
| K1 克隆头 | 没有克隆祖先表（前提 B） | 三个候选一起中（量过）：共用前提，不分辨；D8 自己写着多头重建没有输入 | K1.1 |
| K1 w = 2 | 候选一甲 / 乙、候选二、TSZ | 没打中（量过）：两块盘各自取槽、单份坏 87 485 次，r = m = 0；对照（第二轮的丙）照中 | K1.2 |
| K1 清扫删行 | 候选一甲 / 乙、候选二、TSZ | 没打中（量过）：删行 45 402 / 58 233 次，r = m = 0；清扫只删行不抹单元时三个一起中（共用前提，不分辨） | K1.3 |
| K2 读法 Z | 记录被借走之后再变短、多对象交错变短与删除、截到 0 再长回来 | 没打中（量过）：截短 3 086 次（其中对象之前被挪过 1 660、长回来 671），三个候选读法下 data_Z 全 0；对照 RG 每段复活 | K2.1 |
| K2 区间墓碑 | 最小改法甲（I-9.19 / I-9.20 只判整对象 + 区间起 < 区间止） | **打中**（仓副本量过）：放过今天拦的「活对象上压住活单元的区间墓碑」「删完的对象没有整对象墓碑」「区间止不是 2⁶⁴ − 1」三类；仓里已有的两条坏镜像用例断言不成立 | K2.2 |
| K2 区间墓碑 | 最小改法乙（同甲，只许尾段） | **打中**（仓副本量过）：仓里已有 15 条全过，但放过前两类（4 张镜像）；要再加一条「区间墓碑死亡 txg < 被它压住的活单元的诞生代号」才拦得住（零轮，推的） | K2.2 |
| K3 N1 删第三句 | 右半容器号给错 | **打中拦不住的一形**（仓副本量过）：c 落在 (左容器号, 左最大 key] 而分隔 key 照 N1 新形取最小 key 时，第一句、第二句、I-9.12 都不红；今天只第三句红。后果推的：不撞号、不误路由，丢的是 D8 那条写路径纪律的镜像判据 | K3 |

## 复跑与文件

模型目录 `research/prompts/m3-leaf-death-r3-opus-model/`（主模型从第二轮攻方模型改出；`src/clone.rs` 是新写的克隆头小模型；`copy/` 是放进仓副本的一个 harness 测试目标与给副本 checker 加开关的脚本）。复跑（仓根；每段经内存包装，编译 6G、其余 4G，6 线程；同一时刻只起一条 cargo；`rerun.sh` 不靠自己在仓里的位置找输入，仓路径从 `SINGLEFS_REPO` 取）：

```
M=research/prompts/m3-leaf-death-r3-opus-model; S=research/scripts; O=<空目录>
export SINGLEFS_REPO=/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree
bash $S/run-with-memory-cap.sh 6G bash $S/capped.sh 6 bash $M/rerun.sh $O build
bash $S/run-with-memory-cap.sh 4G bash $S/capped.sh 6 bash $M/rerun.sh $O sweep-4 clone determinism
bash $S/run-with-memory-cap.sh 4G bash $S/capped.sh 6 bash $M/rerun.sh $O sweep-0
bash $S/run-with-memory-cap.sh 4G bash $S/capped.sh 6 bash $M/rerun.sh $O sweep-1
bash $S/run-with-memory-cap.sh 4G bash $S/capped.sh 6 bash $M/rerun.sh $O sweep-2 sweep-3
bash $S/run-with-memory-cap.sh 6G bash $S/capped.sh 6 bash $M/rerun.sh $O copybuild
bash $S/run-with-memory-cap.sh 4G bash $S/capped.sh 6 bash $M/rerun.sh $O copyrun-today-k2
bash $S/run-with-memory-cap.sh 4G bash $S/capped.sh 6 bash $M/rerun.sh $O copyrun-r1-k2
bash $S/run-with-memory-cap.sh 4G bash $S/capped.sh 6 bash $M/rerun.sh $O copyrun-r2-k2
bash $S/run-with-memory-cap.sh 4G bash $S/capped.sh 6 bash $M/rerun.sh $O copyrun-today-k3 copyrun-drop-k3
bash $S/run-with-memory-cap.sh 6G bash $S/capped.sh 6 bash $M/rerun.sh $O existing-r1
bash $S/run-with-memory-cap.sh 6G bash $S/capped.sh 6 bash $M/rerun.sh $O existing-r2 summarize
bash $M/compare.sh $O
```

交回前在空目录 `~/.local/share/singlefs/work/m3-leaf-death-r3-opus/rerun2/` 里按上面从头跑通（本机当时另有会话占着 24–30 核）：五批 sweep 182.1、145.4、64.7、39.6、59.6 秒；copyrun 三个 K2 段 129、111、112 秒，两个 K3 段约 8–10 秒；existing 两段 50、47 秒。`expected/` 是 `rerun2/summary/` 拷下来的，在 `rerun2/` 上跑 `compare.sh` 是同义反复；独立的核对是更早在空目录 `rerun1/` 里的一次全量（没有 existing 两段），它的 `summary/` 七份与 `rerun2/` 逐字节相同（`cmp` 全过），`agg.txt`、`clone.txt` 又与开发中那次全量（草稿目录 `dev/run2/`）逐字节相同。模型确定：`expected/determinism.txt`「R3 name=determinism range=1152:1440 histories=288 two_runs_identical=true」。

模型目录每个文件的 sha256（`find . -type f | sort | xargs sha256sum`）：

```
9e65173150b4282ae205931b9782e253eae6946758ddaaec67f425600026e9cf  ./Cargo.lock
09765a85ab8f02855f9362fd2c00f386a37c022a47e3866c095c6893a2da8827  ./Cargo.toml
998590fce71af3c640fe481188c05d6fa30cd07731420dad4d13c3a0cdea6d91  ./compare.sh
edeca50c4bd8d3b5274764fa1fa8454d8f766eb95a99957b4e7b54f81cf3e02f  ./copy/opus_r3_leaf_death_images.rs
92162c6a5a1ab9c9f2fa397f875d47b41f6ef43f640c45baaee8ea9db6bbf364  ./copy/patch_checker.py
a85dbcebee8af7119ac11df7ecede57fd1b65952238a891d4252165e9f5ab629  ./expected/agg.txt
562186ba0cd486619a88e34d73a44c9e09914bdd0ae1f9ef36d878d7dbe4f2d4  ./expected/clone.txt
986f717136181215c6b99d45bb4f1144876ea07e806e4fbbbab0e20d25b10778  ./expected/copy-k2.txt
f8c22c02f1e6508b4e7dd1c213819c49c398791c055b99c76ef56f59441271dd  ./expected/copy-k3.txt
c32c3447b5ce3c30f773ca2aa0514667819016737de226faef24ac5243b1aaae  ./expected/coverage.txt
0f32ef06e3661680c660173618f6150f1f06490dd7fa49f81f30a74e3c0268af  ./expected/determinism.txt
52ae5077c5db9166055624856e15052f5d0cb43e77e8059751a03d07f2570872  ./expected/existing-r1.txt
2913fb523081651b51c37fe16fb04d83c62355084d52d22616c66234fc9ef735  ./expected/existing-r2.txt
02cae59e65078d4fe314d3e8d632a2497ed8fd3964f0196b6d02a4af9896f0b9  ./expected/sorted-output-sha256.txt
637d1d8d9df5a3d4d74e59209d007897803de3c0174e0d4c383d96f8f873db4e  ./rerun.sh
10fe70587a82d79514bc512875b22de8c867569dd5f378fae2b6b2a31516bbb9  ./src/clone.rs
5515a2976d216b7dce553cf470538e4fab11efd58d6863883059fa08967848d3  ./src/main.rs
f97b078217752f9bfe5156d5431e3873d2201e2ea8921b56e0e12ffee0cbff42  ./src/model.rs
239b58a9bdec20b3b786455510c800552695662207e2dcdb69201a406b0f0cb9  ./tools/agg.awk
f5701ccacf4e0706022ad9840b6baba066168809908cd4720305eb90726788c4  ./tools/coverage.awk
```

完整模型输出（`sweep-*.out` 五份合计 18 725 行）只留在草稿目录，`expected/sorted-output-sha256.txt` 钉着它们排序后的 sha256。

## K1　三个候选在克隆头、w = 2、清扫删行三处

### K1.0　装置（我自己的模型，从第二轮攻方模型改出，不是 E177 装置的拷贝）

主模型 `src/model.rs` + `src/main.rs`：inode 叶容器、数据单元、退役容器、墓碑容器四类单元落在槽里，每个单元头带诞生代号与写序实例；故障与第二轮攻方同一套（在飞发布的写逐子集全枚举、丢根三种、管理员回退、续做）。这一轮加的：

- 三条写路径：`ret_carried`（退役记录与意图那次的墓碑都写；回退那次退役树、墓碑树随现行）、`ret_r_old`（同上，回退那次退役树取 R_old）、`stubs`（候选二：删空 / 被吸收的叶留零条目版本，合并与借跳过零条目桩，不写退役）。借法：前两条路径 × {不借, N1, N2}，`stubs` × {不借, N1}。
- 读法（每次发布之后扫盘重建，与真值比）：候选一甲 `C1jia`（退役容器只按 P2 择版本、记录按死亡 txg 在不在谱系上滤；`ret_carried`）、候选一乙 `C1yi`（退役容器按 P2 + 版本谱系择、不按死亡 txg 滤；`ret_r_old`）、候选二 `C2`（`stubs`；同身份取最新版本，零条目版本就是一版）、`TSZ`（意图那次的整对象墓碑：墓碑容器只按 P2 择、记录按死亡 txg 在谱系上滤，按 (号, 对象出生代) 在所有版本里不收；同号取诞生代号最大的版本供出；数据单元读法 Z）。四个读法的数据单元都按 Z 收。
- 对照（证明装置看得见错）：`S`（不读任何死亡记录）、`C1bing_uniform`（第二轮打中的丙：退役容器按谱系择 + 死亡 txg 滤）、`TSZ_noLin`（墓碑不按谱系滤）、数据单元 `data_RG`（只看记录在不在、不看 size）、清扫写错的 `sweep_without_erasing`。
- 每格记 r（读出来与真值不同、或真值没有的）、m（真值有而没读出来的，不含删除队列里那个对象）、q（删除队列里那个对象没读出来；TSZ 的「意图那次生效」就落在这一格，是第二轮已量的 X3 代价，不另算打中）。
- 变体：`plain`、`zhist`（K2 的历史）、`w2`、`sweep`、`w2sweep`、`sweep_without_erasing`。
- 取样：2 种形状（叶容量 233、起始 700 个空 inode、1500 个动作；叶容量 6、起始 40 个、600 个动作）× 3 几何（G-宽、G-环 取最低号、G-环 随机取）× 8 种路径与借法 × 6 变体 × 8 种子 = 2304 段。估时：先各跑 12 段，叶容量 233 那 12 段 9.2 秒、叶容量 6 那 12 段 1.1 秒（6 线程），推全量约 15 分钟（本机当时另有会话占着 24–27 核），不缩。正式分五批（300 / 300 / 300 / 252 / 1152 段），批不设限时。

走到的形状（`expected/coverage.txt`，按变体相加，两种形状合起来；`plain` 一行原样）：

```
variant=plain histories=384 publishes=1096321 crash_points=21690 crash_states_enumerated=131796 crash_points_skipped_more_writes_than_k_max=89 rollbacks=10254 recoveries=161603 lost_root_one=2802 lost_root_one_readable_but_unusable=2788 lost_root_three=2527 drops=10781 absorbs=9237 borrows=2513 records_with_data_moved_to_another_leaf=13557 truncates=0 truncates_to_zero=0 truncates_of_a_record_moved_to_another_leaf_before=0 regrows_after_a_truncate=0 zero_entry_versions=5305 retirement_records=15784 units_tombstone=283909 w2_damage_events=0 w2_copies_repaired=0 w2_copies_lost_for_good=0 w2_damage_refused_last_copy_of_a_ring_unit=0 w2_units_readable_only_on_disk1_peak=0 sweep_rows_deleted=0 sweep_units_erased=0 sweep_rounds_skipped_for_a_bad_copy=0 sweep_rows_deleted_without_erasing=0
```

全部变体合计枚举的崩溃状态 1 088 944 个（`coverage.txt` 六行 `crash_states_enumerated` 相加：131796 × 3 + 283644 × 2 + 126268），每批都不到 10⁶；`crates/` 的层 0 枚举函数一次都没调（`crates/` 没有退役记录、零条目版本、克隆头的写者，没东西可录）。

### K1.1　克隆头共享叶：打中候选二的字面形态（复活）；三个候选的强形态都没打中

装置是另一个小模型 `src/clone.rs`（`m3-leaf-death-r3-opus-model clone`）：origin O 新池建 5 个 inode（叶容量 3），O 上先做 0–2 步（建 / 覆盖写 / unlink 到底，逐序列全取，125 个前缀），在 txg F 拍快照、建克隆头 C（C 的 inode 树共享 O 那一刻的叶；C 的墓碑树、退役树是自己的，C 读 O 的那几棵只读 ≤ F 的版本；水位 = O 那一刻的水位）；分叉之后两个头上 0–3 步全取（每个前缀约 7 900 个序列节点），每次发布之后两个头的视图都从盘上重建、与各自的真值比。盘是 G-宽（旧版本一个都不盖，对复活最不利）。克隆头的「首次 COW 换身份」照 D8 已定项 6 原文：「新身份 = (本头树 ID, 类型 2, 原容器号, 本次发布代号)——保留容器号、只换出生树与出生代，同时给旧身份写退役记录」（`.claude/kb/decisions/08-核心索引结构.md:171`）。

`expected/clone.txt`（C 的视图那几行；O 的视图全部 r = m = 0）：

```
R3 name=clone reader=C1_A view=clone sequence_nodes=984679 op_checks=984554 r_ops=0 m_ops=0 q_ops=0 r_max=0 m_max=0 first=none
R3 name=clone reader=C2lit_A view=clone sequence_nodes=984679 op_checks=984554 r_ops=500463 m_ops=0 q_ops=0 r_max=3 m_max=0 first=prefix_on_O=[]after_fork=[Unlink(2,1)]r=1m=0
R3 name=clone reader=C2num_A view=clone sequence_nodes=984679 op_checks=984554 r_ops=0 m_ops=0 q_ops=0 r_max=0 m_max=0 first=none
R3 name=clone reader=C2ret_A view=clone sequence_nodes=984679 op_checks=984554 r_ops=0 m_ops=0 q_ops=0 r_max=0 m_max=0 first=none
R3 name=clone reader=TSZ_A view=clone sequence_nodes=984679 op_checks=984554 r_ops=0 m_ops=0 q_ops=220971 r_max=0 m_max=0 first=none
R3 name=clone reader=C1_B view=clone sequence_nodes=984679 op_checks=984554 r_ops=396997 m_ops=280983 q_ops=0 r_max=3 m_max=3 first=prefix_on_O=[]after_fork=[Create(1)]r=1m=0
R3 name=clone reader=TSZ_B view=clone sequence_nodes=984679 op_checks=984554 r_ops=474536 m_ops=454167 q_ops=220721 r_max=3 m_max=3 first=prefix_on_O=[]after_fork=[Create(1)]r=1m=0
R3 name=clone reader=C2num_B view=clone sequence_nodes=984679 op_checks=984554 r_ops=394943 m_ops=281019 q_ops=0 r_max=3 m_max=3 first=prefix_on_O=[]after_fork=[Create(1)]r=1m=0
```

（`expected/clone.txt` 第 3、7、9、13、15、5、17、11 行原样；`Unlink(2,1)` 是在头 C（树 2）上 unlink 1 号，`Create(1)` 是在 O 上建一个。）`_A` 是前提 A（有克隆祖先表：C 的视图收出生树 = C 的单元 + 出生树 = O 且诞生代号 ≤ F 的单元），`_B` 是前提 B（C 收出生树 = O 的全部单元）。

- **打中（候选二字面形态，复活）**：可复跑的反例是最短的那一条——前缀空、分叉之后在 C 上 unlink 1 号。逐步：① 分叉时 C 共享 O 的叶 (O, 1, g)，装 [1, 2, 3]；② C 上 unlink 1 的意图那次重写这片叶，跨头首次 COW 换成 (C, 1, t₁)，[1(nlink 0), 2, 3]；③ 最后一批删掉 1，(C, 1, t₂) 装 [2, 3]。候选二字面上不写退役记录（零条目版本是它的唯一载体，这片叶没删空）。C 的视图里 (O, 1, g) 与 (C, 1, *) 是两个身份，(O, 1, g) 诞生代号 ≤ F 那一版是它自己身份里最新的，1 号只有它供出 → 1 号在 C 里复活，r = 1。984 554 次判定里 500 463 次 r > 0。
- 改法（都只在我的模型上量过、被攻过零轮）：`C2ret`（跨头首次 COW 照 D8 已定项 6 写退役记录）0；`C2num`（C 的视图里同一容器号只要有出生树 = C 的身份，出生树 = O 的同号身份整个不收，「号接替」，不写盘）0。前者让候选二在有克隆头时也要一棵退役树（于是第二轮那条甲 / 乙回退规则跟着回来）；后者要改 D8 已定项 6 那一句（「同时给旧身份写退役记录」换成读侧规则）。
- 候选一（`C1_A`，删空、被吸收、跨头首次 COW 都写退役记录）0；TSZ（`TSZ_A`，跨头首次 COW 不写任何东西，靠「同号取诞生代号最大」与 C 自己的墓碑）r = m = 0，只有 q（unlink 途中，X3 那一格）。TSZ 在克隆头上同样不需要 D8 那一句的退役记录，所以取 TSZ 也要改那一句（推的，读条款）。
- 前提 B 下三个候选一起中（`C1_B`、`TSZ_B`、`C2num_B`，第一例都是分叉之后在 O 上建一个，O 的新版本被 C 当成自己的）：这是共用前提，不分辨候选。D8 自己写着这一格没有输入：「落地前多头镜像的 inode 树重建没有输入」（`.claude/kb/decisions/08-核心索引结构.md:207`）。

四句（对 `C2lit_A` 那一格）：
- 分不分辨臂：分辨。同一个前缀、同一组用户动作上 `C1_A`、`TSZ_A`、`C2ret_A`、`C2num_A` 都是 0，只有候选二字面形态中；它分辨的是候选二自己的两个形态（字面 / 补了的），也分辨候选二对候选一、TSZ。
- 判别子看不看得到：看得到。C 视图里 (O, 1, g) 与 (C, 1, t) 同号、出生树不同，单元上都在；缺的是规则（谁接替谁），不是信息——补一条退役记录或补「号接替」读法都分得开（量过 0）。
- 满足的是判据哪一句：正文第五节第二条「一个候选被攻方坐实一形复活或误判死亡」；打中的是第二轮判决给候选二的定义「零条目版本一直留在树里」在有克隆头时没有交代跨头首次 COW 的旧身份怎么死。
- 改法在打中的格上还中不中：`C2ret` 不中、`C2num` 不中（量过）；「只改 I-9.13」照中（`C2lit_A` 就是只改 I-9.13 的那一形）。

什么现象会推翻它：用户定的候选二本来就保留 D8 已定项 6 那一句（跨头首次 COW 写退役记录），则这一格不是打中而是候选二的代价多一项（退役载体只在跨头 COW 时用）；或第一版不做克隆头，则这一格推到做克隆头的那一批（第一版做不做克隆头我没查）。

### K1.2　副本坏一份（w = 2）：没打中（量过）

装置：`w2` 变体每个单元两份，两块盘各用一个分配器取槽（盘 0 照几何；盘 1 在 G-环 两种几何下一律随机取、另一个随机源），所以同一个旧版本在两块盘上活得不一样久；扫盘重建收两块盘上读得出的单元（同一次写的两份算一个）。每次发布之后按 60‰ 掷一次单份坏（随机盘、随机槽），坏了 6 次发布之后修回（另一份还读得出就照它写回，否则那一份永远丢了）；根环窗口里任一版引用的单元不许被坏到两份都读不出（w = 2 的承诺），别的单元（垃圾）两份都可以坏。

| 走到的（`expected/coverage.txt` 的 `w2` 一行） | 数 |
|---|---|
| 单份坏 / 修回 / 永久丢一份 / 因为会让环里单元两份全坏而拒掉 | 87 485 / 99 147 / 4 982 / 332 |
| 只在盘 1 上还读得出的单元（峰值） | 103 |
| 崩溃状态（每个单元两刀写，逐刀子集枚举） | 283 644 |

结果（`expected/agg.txt`，`w2` 与 `w2sweep` 两个变体）：`C1jia`、`C1yi`、`C2`、`TSZ` 的 inode、data_Z 每格 r = m = 0（TSZ 只有 q），例第 38 行原样「path=ret_carried variant=w2 reader=C1jia tree=inode histories=144 r_max=0 m_max=0 q_max=0 histories_r_pos=0 histories_m_pos=0」、第 114 行「path=stubs variant=w2 reader=C2 tree=inode histories=96 r_max=0 m_max=0 q_max=0 histories_r_pos=0 histories_m_pos=0」、第 90 行「path=ret_r_old variant=w2 reader=TSZ tree=inode histories=144 r_max=0 m_max=0 q_max=1 histories_r_pos=0 histories_m_pos=0」。对照照中（装置看得见副本带来的差别）：第二轮打中的丙在 `w2` 下照样复活，「path=ret_carried variant=w2 reader=C1bing_uniform tree=inode histories=144 r_max=9 m_max=0 q_max=0 histories_r_pos=38 histories_m_pos=0」（第 35 行），`plain` 下同一格是第 2 行的 r_max=61。

为什么三个候选都不中（推的，与量相符）：三个候选的死亡证据（甲的退役容器、乙的退役容器、候选二的零条目版本、TSZ 的墓碑容器）都是现行那一版引用的单元，w = 2 保它至少一份读得出；单份坏只能让垃圾（旧版本）更少读得出，或者因为两块盘各自取槽让垃圾在另一块盘上多活一阵——后者只多出「旧版本」，三个候选对旧版本的处理与 w = 1 时同一条（死亡证据在现行版本里）。所以 w = 2 不分辨候选；它只放大第二轮那种「读侧依赖已经被放出去的旧版本」的错（对照丙那一格）。没建的：一块盘整个离线一段、回来时带着旧内容（只是「只在一块盘上读得出的垃圾」更多，同一个机理，推的）；两份都坏（超出 w = 2 的承诺）。

### K1.3　清扫删实例表行：没打中（量过）；清扫写错时三个候选一起中（共用前提）

装置：`sweep` / `w2sweep` 变体每次发布之后清扫一轮：这一轮没有坏副本（照 `crates/` 的「有一次读失败这一轮一行都不删」，`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-core/src/instance_table_sweep.rs:7`「这次清扫里有一次读失败、设备不全、根环有一槽读不出或全零或重读一次仍自证不过，」）、某实例不是现行实例且根环里没有它的根（含丢根后读得出而用不了的那条），就抹掉两块盘上按它那一行判未发布的单元，再删掉那一行；删了行之后它的单元一律当已发布。阳性对照 `sweep_without_erasing`：只删行、不抹单元。

| 变体 | 删掉的行 | 抹掉的单元 | 因坏副本跳过的轮 |
|---|---|---|---|
| `sweep` | 45 402 | 74 593 | 0 |
| `w2sweep` | 58 233 | 158 384 | 407 299 |
| `sweep_without_erasing` | 45 402（只删行） | 0 | 0 |

结果：`sweep`、`w2sweep` 下四个候选读法每格 r = m = 0（例第 16 行「path=ret_carried variant=sweep reader=C1jia tree=inode histories=144 r_max=0 m_max=0 q_max=0 histories_r_pos=0 histories_m_pos=0」）；`sweep_without_erasing` 下三个候选一起中，例第 111 行「path=stubs variant=sweep_without_erasing reader=C2 tree=inode histories=96 r_max=34 m_max=3 q_max=1 histories_r_pos=72 histories_m_pos=24」，`C1jia`、`C1yi`、`TSZ` 同形（r_max 25、m_max 2，`expected/agg.txt` 带 `sweep_without_erasing` 的行）。

判：清扫「先抹后删」是 C113 P2 的承诺，三个候选共用；它写错时打中不分辨候选（`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」那张表的第一形），不拿它判候选。没有候选专属的漏网形，理由（读码，推的）：`crates/` 的清扫只在根环里没有那个实例的根时才动它，「一个实例的根全部轮出根环之后，抹掉池里按它那一行判未发布的单元头」（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-core/src/instance_table_sweep.rs:2`），跳过的只是「再跳过根环里任一可读根的账里仍分配的槽」（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-core/src/instance_table_sweep.rs:17`），而被崩溃抛弃的单元当初写在当时空着的槽里，之后被任一根分配到就已经被那一根的单元盖掉；认单元只看码与头校验和、不看打包记录类型（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-core/src/instance_table_sweep.rs:195`「UNIT_CLASS_PACKED => usize::try_from(PACKED_UNIT_HEADER_BYTES).expect("107"),」），类型 3 的退役容器、类型 1 的墓碑容器、零条目叶都照同一条抹。模型里清扫不读账（模型的未发布单元从不落在环里根引用的槽里），这一点是限度。

### K1.4　三个候选在三处并排

| 候选（读法） | 克隆头共享叶 | 副本坏一份（w = 2） | 清扫删行 |
|---|---|---|---|
| 候选一甲（`C1jia`，随现行 + P2 择 + 死亡 txg 滤） | 0（`C1_A`，前提 A） | 0 | 0；清扫写错时中（共用） |
| 候选一乙（`C1yi`，回退取 R_old + 谱系择） | 克隆模型里没有回退，甲乙同一读法：0 | 0 | 0；清扫写错时中（共用） |
| 候选二字面（零条目版本，不写退役） | **复活**：r 峰值 3，500 463 / 984 554 次判定 | 0（`C2`） | 0；清扫写错时中（共用） |
| 候选二 + 跨头 COW 退役（`C2ret`） / + 号接替（`C2num`） | 0 / 0（零轮） | 没建（主模型没有克隆头） | 同上 |
| TSZ（墓碑 + 同号取诞生代号最大 + Z） | 0（`TSZ_A`），q 在 unlink 途中 | 0，q 峰值 1 | 0，q；清扫写错时中（共用） |
| 三个一起 | 前提 B（没有克隆祖先表）一起中 | — | 清扫只删行不抹单元一起中 |

每处的「m」都不含删除队列里那个对象；TSZ 的 q 就是第二轮已量的「unlink 途中漏活」那一格，不新算。

## K2　读法 Z 换一组历史；区间墓碑的最小改法

### K2.1　Z 在三形历史上：没打中（量过）

同一个主模型的 `zhist` 变体（384 段，2 种形状 × 3 几何 × 8 种路径与借法 × 8 种子；崩溃、丢根、回退照常掷）。写类动作里 40% 是**截短**（`truncate`：留下的单元不重写、尾部单元离开这一版、size 落进剩下的单元数里，其中 30% 截到 0）；截短与覆盖写先挑最近被合并 / 借挪过的记录（40%），再挑「盯着的」那片叶里的对象（40%，unlink 也是 70% 落在这片叶上，即多对象交错变短与删除）；顺序写 40% 挑最近截短过的对象（截到 0 再长回来）。

| 走到的（`expected/coverage.txt` 的 `zhist` 一行） | 数 |
|---|---|
| 截短 / 截到 0 | 1 372 / 1 714 |
| 截短的对象是之前被挪到别的叶的（记录被借走 / 吸收之后再变短） | 1 660 |
| 截短之后长回来（size 0 起顺序写） | 671 |
| 记录带数据被挪到别的叶 | 11 022 |
| 崩溃状态 / 回退 / 恢复 | 126 268 / 10 119 / 156 415 |

结果（`expected/agg.txt`）：三个候选的读法下 data_Z 全 0，例第 59 行「path=ret_carried variant=zhist reader=C1jia tree=data_Z histories=144 r_max=0 m_max=0 q_max=0 histories_r_pos=0 histories_m_pos=0」、第 97 行「path=ret_r_old variant=zhist reader=C1yi tree=data_Z histories=144 r_max=0 m_max=0 q_max=0 histories_r_pos=0 histories_m_pos=0」、第 119 行「path=stubs variant=zhist reader=C2 tree=data_Z histories=96 r_max=0 m_max=0 q_max=0 histories_r_pos=0 histories_m_pos=0」；TSZ 只有删除队列里那个对象的单元（q，第 65 行 q_max=42）。对照：同一批历史上只看记录在不在、不看 size 的 RG 每段都复活，第 58 行「path=ret_carried variant=zhist reader=C1jia tree=data_RG histories=144 r_max=41 m_max=0 q_max=0 histories_r_pos=144 histories_m_pos=0」——截短之后尾部那几个旧单元在盘上、过 P2、在谱系上，只有 size 把它们挡掉。

Z 在这三形上不出错的机理（推的，与量相符）：截短与长回来都改 inode 记录的 size，记录与它供出的那片叶同一次发布写；Z 的前提只有「inode 记录读对了」与「文件没有洞」。记录挪到别的叶之后，现行版本里只有一片叶装着它（被挪走的那一片同一次发布重写），同号取诞生代号最大的那一版就是挪之后的那一版。截到 0 再长回来，同一锚点有旧单元与新单元两份，新的诞生代号大；回退到长回来之前，新单元不在谱系上。

「没打中」是一次抽样；第二轮攻方的 Z 抽样（挪动 24 481 次、号重发 76 次）与这一轮是两份不同的历史生成器，但都出自攻方腿，不是「同一份提示再派一条同立场腿」。

### K2.2　区间墓碑：两种最小改法都放过今天拦的坏镜像（量过，仓副本）

最小改法（我写的两种，只改 CPU checker 的谓词，`copy/patch_checker.py` 加环境变量开关，`crates/` 别处不动）：

- 改法甲（`r1`）：I-9.19 只判整对象墓碑；I-9.20 不重号只判整对象墓碑；I-9.20「第一版只许整对象」换成「区间起 < 区间止」。
- 改法乙（`r2`）：同甲，「第一版只许整对象」换成「区间起 < 区间止 且 区间止 = 2⁶⁴ − 1」（只许尾段）。

今天这两句的原文：I-9.19「指认的 (号, 对象出生代)，inode 树里有号与对象出生代都相同的记录时，那条记录的 nlink = 0」（`.claude/kb/invariants.md:307`），I-9.20「第一版只许整对象 [0, 2⁶⁴ − 1]」（`.claude/kb/invariants.md:308`）；代码 `/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-checker/src/walk.rs:10780`「.filter(|tombstone| lineage_chain.holds_txg(tombstone.death_txg))」（I-9.19 只按谱系滤）、`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-checker/src/walk.rs:10822`「judgements.judge(invariant, tombstone.covers_the_whole_object(), || {」（整对象那一句）。

镜像：池上 1 号三个数据单元（新池新建文件之后顺序写）、2 号空、3 号删完、根环转过一圈；墓碑树第一片容器整份换成表里那几条（重封、补校验和）。`live_units` 是按区间墓碑的读法被压掉的活单元数（1 号 size 81 585 折成 3 个单元）。今天 / 甲 / 乙三列是 checker 原样输出的违例清单（`expected/copy-k2.txt`）：

| 镜像 | live_units | 今天 | 甲 | 乙 | 这张镜像是什么 |
|---|---|---|---|---|---|
| deleted_three_start_one_no_whole_tombstone_left | 0 | I-9.20 | 无 | 无 | 删完的 3 号那条整对象墓碑区间起写成 1：3 号再没有整对象墓碑 |
| live_one_start_one_covers_two_live_units | 2 | I-9.19、I-9.20 | 无 | 无 | 活的 1 号一条 [1, 2⁶⁴ − 1]，压掉 2 个活单元 |
| live_one_start_two_covers_one_live_unit | 1 | I-9.19、I-9.20 | 无 | 无 | 同上，压掉 1 个 |
| whole_tombstone_retargeted_to_live_one_start_one | 2 | I-9.19、I-9.20 | 无 | 无 | 3 号那条整对象墓碑整条改指活的 1 号、区间起 1 |
| live_one_range_end_not_max | 2 | I-9.19、I-9.20 | 无 | I-9.20 | 活的 1 号一条 [1, 2] |
| today_range_end_five | 0 | I-9.20 | 无 | I-9.20 | 仓里已有坏镜像的同形（3 号墓碑 [0, 5]） |
| deleted_three_whole_plus_a_start_one_copy | 0 | I-9.20 | 无 | 无 | 3 号整对象 + 一条同死亡 txg 的 [1, 2⁶⁴ − 1] |
| live_one_start_three_covers_nothing、live_two_start_one_empty_file、live_one_two_equal_ranges_two_deaths | 0 | I-9.19、I-9.20 | 无 | 无 | 区间墓碑的合法形状（变短、变短 → 长回来 → 再变短）：今天误红，改法本来就要放过 |
| today_whole_tombstone_retargeted_to_live_two、today_two_whole_copies、today_death_zero、today_death_after_every_version | 0 | 各自原样 | 同今天 | 同今天 | 仓里已有坏镜像的同形 |
| queued_start_one（删除途中意图那条墓碑区间起写成 1） | 0 | I-9.18、I-9.20 | I-9.18 | I-9.18 | 队列里有号时 I-9.18 照拦 |

（表里每行都由 `expected/copy-k2.txt` 那三行合成；原行例「OPUSR3 k2 range=r2 drop_i94_third=false image=live_one_start_one_covers_two_live_units tombstones=[(3, 6, 0, 18446744073709551615, 7), (1, 3, 1, 18446744073709551615, 7)] live_units_under_a_range_tombstone=2 violated=[]」。）

仓里已有的坏镜像用例（`pool_checker_judges_the_deletion_state_of_every_walked_version` 里名字带 tombstone 的 15 条，各自断言今天的判定）在两种改法下跑（`expected/existing-r1.txt`、`expected/existing-r2.txt`）：甲下 2 条断言不成立（「test a_tombstone_that_does_not_cover_the_whole_object_reddens_only_the_tombstone_record_invariant ... FAILED」，那张 [0, 5] 镜像甲下全绿；「test a_queued_inode_whose_tombstone_does_not_cover_the_whole_object_reddens_the_queued_tombstone_invariant ... FAILED」，只剩 I-9.18），乙下 15 条全过。

判：
- **打中（两种改法都放过真的坏镜像）**：「活对象上区间起落在 size 之内的区间墓碑」（压掉活单元，误判死亡那一形）与「删完的对象只剩一条尾段墓碑、没有整对象墓碑」（TSZ 读法下那个对象在旧叶版本里复活那一形，推的：TSZ 只认整对象墓碑）今天都红、两种改法下都绿。前者在合法历史里有同形（变短到 k、再长回来之后那条 [k, 2⁶⁴ − 1] 还在，活单元的锚点 ≥ k 而诞生代号晚于那条墓碑），所以单看墓碑与 inode 记录分不开对错；要分开得比数据单元头里的诞生代号——码 1 数据单元头带它，「码 1 数据单元 32768：类身份段 105（偏移 42 标签副本 / 43 树 ID / 51 对象 ID / 59 对象出生代 / 67 锚点 / 75 诞生代号」（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-core/src/unit.rs:239`）。补的那一条（只在我这里写出、没实现、被攻过零轮）：「inode 树里活着的对象 (号, 出生代)，它在谱系上的每条区间墓碑 [k, 2⁶⁴ − 1]（死亡 txg T）：extent 树里这个对象锚点 ≥ k 的每个单元，单元头的诞生代号 > T」。后者（删完的对象没有整对象墓碑）是今天第一版只许整对象那一句顺带拦的；删完、根环转过之后镜像上没有别的东西说那个对象存在过，补不回来（推的）。
- 甲还多放过一形：区间止不是 2⁶⁴ − 1 的墓碑（仓里已有的 [0, 5] 那条坏镜像），乙拦得住。所以最小改法至少是乙，再加上一条单元诞生代号的判据；I-9.19、I-9.20 都改，GPU 判器那两处跟着改（没改、没跑，checker 档）。
- 这些只在区间墓碑那一支；Z 不写墓碑，不碰 I-9.19、I-9.20。

四句：分辨（只打区间墓碑，Z 不写东西）；判别子看得到（区间、size、单元诞生代号都在镜像上，被打中的是判据写法）；满足正文第三节 K2 第二条「判它会不会放过今天 I-9.19、I-9.20 拦的坏镜像」；改法：甲在 [0, 5] 那格照中、乙不中，两者在「压掉活单元」那两格都照中，只有加单元诞生代号那一条才不中（推的，没给 checker 打这条补丁）。

## K3　删 I-9.4 第三句之后，C116「分裂出来的右半容器号给错」哪一形没有判据拦

被删的那一句：CPU `/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-checker/src/walk.rs:2782`「.judge("I-9.4", left.largest_inode < right.container, || {」、GPU `/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2918`「w_judge(INVARIANT_I_9_4, lt64(left_largest, right_container));」。留下的：第一句 `/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-checker/src/walk.rs:3079`「self.judgements.judge("I-9.4", inode >= container, || {」、第二句 `/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-checker/src/walk.rs:2775`「.judge("I-9.4", left.container < right.container, || {」、I-9.12 `/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-checker/src/walk.rs:3263`「.judge("I-9.12", *separator > previous_largest, || {」。C116 管的那一族：「三条任一条写错，容器身份就会重复或空叶在重建时没有 key 范围」（`.claude/kb/checks-owed.md:106`）。

装置（仓副本，`copy/opus_r3_leaf_death_images.rs` 的 `opus_r3_right_half_container_number_family`）：`crates/` 写出的三片叶 [1..233]、[234..466]、[467]（右半由第 234 条触发分裂，正确的号是 234），把第二片的容器号（叶头与根条目身份引用两处一起）改成 c，分隔 key 取 N1 新形（孩子这一版的最小 key = 234）或今天的写法（= 容器号 c）；另一族先把左片删到 [1..199]（左最大 key 199，左右之间空出 200..233）。checker 原样判，`OPUS_R3_DROP_I94_THIRD=1` 时第三句恒真（`expected/copy-k3.txt`，每张镜像今天 / 删句各一行）：

| c 落在哪 | 例（c） | N1 分隔 key：今天 | N1 分隔 key：删第三句 | 分隔 key = c：删第三句 |
|---|---|---|---|---|
| ≤ 左容器号 1 | 1 | I-9.4 | I-9.4（第二句） | I-1.1、I-9.4、I-9.12 |
| (左容器号, 左最大 key] | 2、100、233；空号那族 150、199 | I-9.4 | **无** | I-9.12 |
| (左最大 key, 右最小 key) | 空号那族 200、210、233 | 无 | 无 | 无 |
| = 右最小 key（对的） | 234 | 无 | 无 | 无 |
| (右最小 key, 右最大 key] | 235、466 | I-9.4 | I-9.4（第一句） | I-9.4、I-9.12 |
| = 下一片的号 | 467 | I-1.8、I-9.4 | I-1.8、I-9.4 | I-1.1、I-1.8、I-9.4、I-9.12 |
| > 下一片的号 | 500 | I-9.4 | I-9.4 | I-1.1、I-9.4、I-9.12 |

原行例：「OPUSR3 k3 range=today drop_i94_third=true image=dense_right_half_number_100_separator_n1_min_key leaves=[no=1 sep=1 1..233][no=100 sep=234 234..466][no=467 sep=467 467..467] violated=[]」与同一张镜像今天那一行「OPUSR3 k3 range=today drop_i94_third=false image=dense_right_half_number_100_separator_n1_min_key leaves=[no=1 sep=1 1..233][no=100 sep=234 234..466][no=467 sep=467 467..467] violated=["I-9.4"]」。

判：
- **拦不住的那一形**：右半容器号 c 落在 (左容器号, 右半最小 key) 里、分隔 key 照 N1 新形取孩子这一版的最小 key。其中 (左容器号, 左最大 key] 那一段今天只有第三句拦，删句之后第一句（记录号 ≥ c 成立）、第二句（c > 左容器号成立）、I-9.12（分隔 key 234 不碰 c）都不拦；(左最大 key, 右最小 key) 那一段今天也没人拦（左右之间有空号时才有这一段，是今天就有的洞，不是删句造成的）。镜像上它与「N1 下左片从右片借过之后」的合法形状是同一形（第一轮判决已认，这一轮把它拆成上表的几段量全）。
- 删句之后只有在 N1 新形的分隔 key 下才漏：分隔 key 照今天 = 容器号时，c ≤ 左最大 key 那一段由 I-9.12 拦（最后一列）。所以漏的是「删第三句」与「分隔 key 与容器号脱钩」两件事合起来的结果。
- 漏的这一形有什么后果（推的，没量）：N1 新形下路由只看分隔 key（最小 key），从叶重建也按最小 key，c 只剩身份一个用处；c 仍满足第二句（沿叶序严格递增），出生代是这次发布的 txg，同一次发布只造一个右半，不撞号。所以 C116 说的「容器身份就会重复」在这一形上不发生；丢的是「右半容器号 = 那条新记录的 inode 号」这条 D8 写路径纪律的镜像判据——它只剩 `crates/` 的单测与那条变异（`crates/mutations.tsv` 第 194 行「并行线三 C116：分裂出来的右半容器号不是触发分裂那条新记录的 inode 号」，它把号改成 0，第二句就拦，不是这一形；这一形在变异表里没有变异）。
- 四句：分辨（只打 N1 新形；N2 保留第三句，不碰）；判别子看不看得到：看不到——镜像上「分裂时号给小了」与「借过之后号小于最小 key」逐字段同形，只有写路径知道这片叶借没借过（「判别子观测不到」那一形，改判据救不回来）；满足正文第三节 K3「拦不住的那一形写出来」；改法：N2（换身份、保留第三句）在这一格不中（第三句照拦，推的），「N1 + 给借过的叶记一个标志」能分开但要改盘上格式（推的，零轮）。

补记（K2.2 表里 `deleted_three_whole_plus_a_start_one_copy` 那一行）：同一个对象两条墓碑同一个死亡 txg（一条整对象、一条尾段），合法历史里没有（变短与 unlink 不在同一次发布），但它不压任何活单元，我没把它算进「放过的坏镜像」：甲放过的坏镜像按 6 张算（表里前六行），乙按 4 张算（前四行）。

## 改法表（我自己提的改法：只在我的模型或仓副本上量过、被攻过零轮）

| 改法 | 修哪一格 | 在打中的那几格上 | 量过 / 推的 |
|---|---|---|---|
| 候选二在跨头首次 COW 照 D8 已定项 6 写退役记录（`C2ret`） | K1.1 候选二字面复活 | 不中：`expected/clone.txt` 第 13 行 r_ops=0 | 量过（克隆模型） |
| 候选二加「号接替」读法：克隆头视图里同号有本头身份就不收 origin 的同号身份（`C2num`） | 同上 | 不中：第 9 行 r_ops=0；前提 B 下照中（第 11 行，共用前提） | 量过（克隆模型） |
| 候选二只改 I-9.13（第二轮的定义原样） | — | 照中：第 7 行 r_ops=500463 | 量过（克隆模型） |
| 区间墓碑最小改法甲 | 区间墓碑落地 | 放过 6 张今天红的坏镜像，仓里已有用例 2 条断言不成立 | 量过（仓副本，CPU checker） |
| 区间墓碑最小改法乙 | 同上 | 放过 4 张（压活单元 3 张、删完没有整对象墓碑 1 张），仓里已有 15 条全过 | 量过（仓副本，CPU checker） |
| 乙 + 「活对象谱系上的区间墓碑 [k, 2⁶⁴ − 1]（死亡 txg T）：extent 树里这个对象锚点 ≥ k 的单元诞生代号 > T」 | K2.2 压活单元那三张 | 推的不中；「删完没有整对象墓碑」那一张照中（镜像上没有别的证据） | 推的（没给 checker 打这条补丁） |
| N1 新形下保留第三句的判别力：改走 N2（借时换身份） | K3 | 推的不中（第三句留着） | 推的 |
| N1 + 叶头记「借过」标志、第三句只对没借过的相邻对判 | K3 | 推的不中；要改盘上格式 | 推的 |

## 没打中的形状

- K1 克隆头（`src/clone.rs`）：叶容量 3、O 上 5 个 inode；分叉前 O 上 0–2 步、分叉后两个头上 0–3 步（建 / 覆盖写 / unlink 到底），125 个前缀 × 每个约 7 900 个序列节点 = 984 679 个节点、984 554 次判定（两个视图各判，unlink 两次发布各判）；G-宽。没打中的：候选一（删空、被吸收、跨头首次 COW 都写退役）、`C2ret`、`C2num`、TSZ，都在前提 A 下。
- K1 w = 2：`w2` / `w2sweep` 各 384 段（两种形状 × 3 几何 × 8 种路径与借法 × 8 种子）；单份坏 87 485 + 86 212 次，修回 99 147 + 98 075，永久丢一份 4 982 + 4 933，只在盘 1 上读得出的单元峰值 103；崩溃状态各 283 644 个。
- K1 清扫：`sweep` / `w2sweep` 各 384 段，删行 45 402 / 58 233、抹单元 74 593 / 158 384。
- K2 Z：`zhist` 384 段，截短 1 372、截到 0 1 714、被挪过的对象再截短 1 660、截短后长回来 671、记录带数据被挪 11 022、崩溃状态 126 268、回退 10 119。
- 主模型六个变体合计 2 304 段、枚举崩溃状态 1 088 944 个；三个候选的读法（`C1jia`、`C1yi`、`C2`、`TSZ`）在 inode 与 data_Z 两棵树上、五个非对照变体里共 50 格、6 720 个「段 × 格」，r、m 全 0（`expected/agg.txt`）。
- 这些「没打中」都是一次抽样（一份提示、一条腿），照规则不算数，要同一份提示再派一条同立场腿。

## 这条腿自己的限度

- 主模型不是 `crates/`：没有 extent 树、分配记录树、记账、映射；读侧是手写的 P2 / 版本谱系 / 择新，不是 `crates/` 的扫描重建（`crates/` 没有）。候选二的零条目版本、候选一的退役记录、克隆头在 `crates/` 都没有写者，`crates/` 的层 0 枚举一次都没调；模型自己的崩溃状态是「在飞发布的单元写逐子集」，k > 8 刀的发布不当崩溃点（w2 下每个单元两刀，跳过的更多：`crash_points_skipped_more_writes_than_k_max` 每个 w2 变体 791 个）。
- 克隆头模型很小：5 个 inode、叶容量 3、分叉后至多 3 步、G-宽、没有崩溃与回退、没有数据单元（TSZ 的数据单元门与 Z 在克隆头上没量）；前提 A（克隆祖先表、C 读 O ≤ F 的版本）是我定的，`crates/` 与 kb 都还没有这一格。候选一甲 / 乙在克隆头上没分开（模型里没有回退）。
- w = 2：没建「一块盘整个离线、回来时带着旧内容」与降级写；根环窗口里的单元不许两份全坏（承诺内），所以没量超出 w = 2 承诺的情形。
- 清扫：模型不读账，「跳过根环可读根的账里仍分配的槽」那一条没建；模型的未发布单元从不落在环里根引用的槽里，这一点是构造出来的，`crates/` 那边是读码推的（K1.3）。
- K2 / K3 的 checker 改法只改了 CPU checker 的谓词（环境变量开关），GPU 判器（`crash_judge_checker.wgsl`）没改、没跑（checker 档）；镜像是在 `crates/` 写出的一版上手改字节，不是写路径写出来的（区间墓碑、N1 分隔 key 在 `crates/` 都没有写者）。
- 第一次跑克隆模型时我用的是还没改线程池的那一版，起了 8 个线程（超过线程上限 6）跑了约 22 秒；那次的输出（`dev/run1/clone.out`）与之后 6 线程的输出（`dev/run2/clone.out`、`expected/clone.txt`）逐字节相同，之后各次都是 6 线程。另有两次命令超过前台 240 秒被转成后台跑完（四种 checker 开关的第一次副本测试、叶容量 233 的第一批 400 段），没有中途停。
- 这条腿全部模型跑合计的崩溃状态约 4.6 × 10⁶（推的：每次主模型全量 1.09 × 10⁶，开发中两次 + 空目录两次，外加两次确定性复跑各 2 × 288 段）；没有一段历史自己超过 10⁶。

## 没做什么

- 没跑重型测试；harness 档只在仓副本上跑了自己的一个测试目标与仓里已有的 `pool_checker_judges_the_deletion_state_of_every_walked_version` 里名字带 tombstone 的 15 条（`checker_narrow_invariants_and_abandoned_roots` 编了没跑）；GPU 判器没跑。
- 没判正推那几格（K1 代价并排、N2 代价、改动量）；没判前两轮的数能不能并排（那是正推的格）；没替主 agent 采纳。
- 副本与模型上的数都没在入库装置上重做：要引，照 `.claude/rules/three-way-inference.md`「一条腿在副本装置上量出来的数，要在入库的装置上重做一次才能引」。
- 区间墓碑的单元诞生代号判据、N1「借过」标志、`C2num` 的读侧都没写代码进 checker 或 `crates/`。
- 没读禁读清单里的文件（这一轮正推腿的报告、模型与提示，m3-w3 两轮的腿报告与模型）。

## 草稿与副本

- 草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r3-opus/`（现 19M）留着：`rerun2/`（交回前从头跑通的那一次：`sweep-*.out`、`clone.out`、`copy-*.log`、`existing-*.log`、`summary/`）、`rerun1/`（更早一次空目录全量，没有 existing 两段）、`dev/`（开发：`model/` 是模型源码的开发副本，`run1/` 是加对照之前的一次全量、不作依据，`run2/` 是加对照之后的全量，`copy-*.log` 与 `existing-tombstone-*.log` 是开发中的副本测试，`trial-*`、`trialz/` 是估时的试跑）、`draft/`（报告分段的草稿）、`run-copy-modes.sh`、`run-existing-tombstone-tests.sh`（开发时用的两个小脚本）。
- 交回前删掉的（`du -sh` 现查的大小）：编译目录 `dev/repo-target/` 1.4G、`dev/model-target/` 1.1M、`rerun1/target/` 1.1M、`rerun1/repo-target/` 1000M、`rerun2/target/` 1.1M、`rerun2/repo-target/` 1.2G；仓副本 `dev/repo/`、`rerun1/repo/`、`rerun2/repo/` 各 21M。删完 `ls` 核过一个不剩。
- 没入库的：完整模型输出（`rerun.sh` 逐字节复出，`expected/sorted-output-sha256.txt` 钉着）。

## cite-check 末行

`python3 research/scripts/cite-check.py research/prompts/m3-leaf-death-r3-opus-output.md`：

```
  ✓ 核了 17 处引文，对上 17 处，没判 0 处（逐处列在上面）
```
