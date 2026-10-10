# m3-leaf-death-r3 攻方腿第二次抽样（Opus，s2）报告（2026-10-08）

攻击面：正文 `research/prompts/_m3-leaf-death-r3-body.md` 第三节 K1、K2、K3。只读冻结树 `~/.local/share/singlefs/work/m3-leaf-death-r1/tree/` 与 kb；照正文独立做，没读这一轮第一次抽样（禁读清单）。代码引文写冻结树里的全路径:行号；kb 引文写 kb 文件自己的行号或 `文件@小节`。
不重复前两轮攻过的角度（第一轮：E177 51 段与合并 64 段；第二轮：修补本身、回退之后谱系链、Z 的三形、区间墓碑与 I-9.19 / I-9.20 的两张镜像、N1 分隔 key 还原）。这一轮攻：克隆头共享叶、副本坏一份（w = 2）、清扫删实例表行；Z 换三组新历史；区间墓碑的最小改法放过哪些坏镜像；删 I-9.4 第三句之后 C116「右半容器号给错」哪一形没有判据拦。

状态：完成（分段落盘）。

## 各格判定一览

「量过」= 在我的模型或草稿目录的仓副本上跑出来的（不是入库装置上的数，主 agent 要在入库装置上重做才能引）；「推的」= 按条款或代码推、没跑。没打中的都是一次抽样。

| 格 | 形态 | 判定 | 依据 |
|---|---|---|---|
| K1 克隆头 | 候选二（零条目一直留，照第二轮辩方的读法不读退役、不读墓碑） | **打中复活**（量过）：克隆头删一条记录（叶不必删空），重生之前的旧身份在克隆头里还被收，2913 / 4681 个后缀 r > 0、峰值 4；最小反例「O 建 6 个、分叉 K、K 删 1 号」 | K1.1 |
| K1 克隆头 | 候选一（带 D8 的重生退役）、TSZ（不写退役） | 没打中（量过）：候选一在 D8 原样那一臂、TSZ 在写不写重生退役两臂上，各 4 681 个后缀，K 与 O 都 0 | K1.1 |
| K1 克隆头 | 三个候选共用：读侧不按祖先表分头 | 全中（量过），不分辨候选；条款没写（D8 射程、C347） | K1.1 |
| K1 清扫删行 | 候选一甲 / 乙、TSZc、候选二 | 没打中（量过）：288 + 72 段（清扫 9 864 次、删行 20 244）、手造 6 景 × 57 后缀，全 0；不分辨候选。剩一口子（清扫那一次头校验和读错）推的、不分辨 | K1.2 |
| K1 w = 2 | 一份坏、错向写、一盘不在 | 没量；推的不分辨（两份都坏时的爆炸半径按候选不同，记代价） | K1.3 |
| K1（TSZ 的定义） | TSZ 的墓碑容器与叶同一套谱系滤（TSZu） | **打中复活**（量过）：族 k1 12 / 36 格、峰值 3，全在 G-环；只按 P2 择（TSZc）0。反例 `trace 64` | K1.4 |
| K2 Z | 借走 / 吸收之后再变短、多对象交错变短与删除、截短到 0 再长回来 | 没打中（量过）：族 z 72 段 + 手造 4 景（2 575 个后缀）0；对照 RG 全中；阳性对照（打洞）Z 中 | K2.1 |
| K2 区间墓碑 | I-9.19、I-9.20 的最小改法 | **打中**（仓副本量过）：今天红的五张坏镜像全绿（压掉活单元的尾区间三张、同一次写两条、unlink 写成尾区间）；分开合法与坏的那个数在数据单元上，I-9.19 / I-9.20 读不到 | K2.2 |
| K3 N1 | 删 I-9.4 第三句之后右半容器号给错 | **打中**（仓副本量过）：左邻容器号 < 右半号 ≤ 左最大 key 那一形，镜像族与写路径写错的整池上判据一条不红，只剩 `inode_tree` 单测（5 / 16 红）；N1 下这一形镜像上没有可见后果（推的） | K3 |

## 复跑与文件

模型目录 `research/prompts/m3-leaf-death-r3-opus-s2-model/`（Rust 模型从第二轮攻方模型拷起改的；`clone/` 是克隆头模型；`copy/` 是放进仓副本的一个 harness 测试目标与给副本加开关的补丁脚本）。复跑（仓根；每条经内存包装，编译 6G、其余 4G，6 线程；同一时刻只起一条 cargo；路径从环境变量与参数取）：

```
S=research/scripts; M=research/prompts/m3-leaf-death-r3-opus-s2-model; O=<空目录>
export R3S2_MODEL_DIR=$PWD/$M SINGLEFS_REPO=/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree
bash $S/run-with-memory-cap.sh 6G bash $S/capped.sh 6 bash $M/rerun.sh $O build
for st in sweep-0 sweep-1 sweep-2 sweep-3 sweep-4; do bash $S/run-with-memory-cap.sh 4G bash $S/capped.sh 6 bash $M/rerun.sh $O $st; done
bash $S/run-with-memory-cap.sh 4G bash $S/capped.sh 6 bash $M/rerun.sh $O demos trace determinism clone
bash $S/run-with-memory-cap.sh 6G bash $S/capped.sh 6 bash $M/rerun.sh $O copybuild
bash $S/run-with-memory-cap.sh 4G bash $S/capped.sh 6 bash $M/rerun.sh $O copyrun-k2 copyrun-k3 copyrun-writer corelib summarize
bash $M/compare.sh $O
```

交回前在空目录 `/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r3-opus-s2/rerun2/` 里按上面从头跑通（`R3S2_MODEL_DIR` 指仓里的模型目录）：五批 sweep 8.0、4.4、41.9、29.0、23.8 秒，副本三条用例 33.85、2.73、0.29 秒，`compare.sh` 12 份全 `same`、退 0（`expected/` 是前一次 `rerun1/` 的 `summary/` 拷来的，`rerun2/` 是独立的第二次）。

模型目录每个文件的 sha256（`find . -type f | sort | xargs sha256sum`）：

```
5f8ebc78bc7e9e53e9112205495e39b02ea0786633f8ada2c3956ecd03ddb21e  ./Cargo.lock
31799b95c863fcab353e0a1a1ecd0d362627242c83bc62271764dd6663d78d68  ./Cargo.toml
ef0cd14426b97dee52a948f1c81970a572779fe3197a609e19f6b25995ac2b8c  ./clone/clone_heads.py
c124712c1128be0550156b3e6f5b6733f2b0ac513b40a66bf83751770f35ea56  ./compare.sh
f0308e8e3347c3e8ecabe3e54da48f696a12f8c4009d1de5559b75d9efd98aba  ./copy/opus_r3s2_leaf_death_images.rs
c2c57ca418506ad5078d1c5f774bb339ed9c62398f1f5878a72e2f7b22734e57  ./copy/patch_copy.py
8a43460062db1c744b7e26faa5b934274dd990de4463bc3248ed9026d5998fa4  ./expected/agg.txt
f949ec31899b101d37ff8921d49bdeaed220771bec5f7a0cc8d17999f3dcf336  ./expected/clone-trace.txt
92c7276abceefad4fbef5f9cb966ef7b5c045884187acba83469308ea9630b30  ./expected/clone.txt
d5af98db56ff2783438752d4bcdd756615cf89337814dddad72826ad188bbf50  ./expected/copy.txt
af851b3c3b49542091bfef59ddc8e53a754f371195e0a87c5dc20892d1118105  ./expected/corelib.txt
1df5a527746bb55eba77591bc463248bf4fb98b73983e4005ab85f6eecc6cc42  ./expected/counts.txt
cfa239972a5d8b8d3f1363c9b44a1ce209ea1b582460b39296c9d54a160720ab  ./expected/demo-prefix-counts.txt
2d0a5e67128e2f9c066794bd8cbf92e87925201ad1bcbe39adecf78e69de1487  ./expected/demos.txt
e02a90c74d347be281c628b4ddceeae1a1765b69c53b5acc038278791111d246  ./expected/determinism.txt
3f053d694c0f6ef52ec9b7aa3e996167fa8b881d2d58faf11e19a98a93768a6c  ./expected/group.txt
0f30c1e5fab0cd63ef97d5cdd9f292b9814ce8d3012ab90baeafe53345b2ae02  ./expected/sorted-output-sha256.txt
36e31bd234cfe2c6440c56e82338782fa8ef03311abfabeb374b45cdf4d40da0  ./expected/trace-64-first.txt
45b225f61ec51653f66320f6c5abf27e00fae9ab3b268ba403389ebefb43195b  ./rerun.sh
c59db04b40e6a5d2a323e7952b239537bbe86d4c30842bf017b2431e6bd140e8  ./src/demos.rs
e4cbbf324f4aa9a9a16ad5c03e839982cd8634b192470c2559d69e86328235e3  ./src/main.rs
a658c376ac057f23ddaa69fa2ed0af914f3245898e1cdeb385adfccd448586d7  ./src/model.rs
f234c3bb07d3b8f4a9272aa46b6d8fc229f63fb68cfa62abf4bfc8c30c79d03d  ./tools/agg.awk
7b6783b50f32fedabd4b622caf263309801b7a608578d8d90ec4016ff57736d1  ./tools/counts.awk
07a49d206da9c00634c6b9a7a291166dfd729d703e30d039e10ababc7e6f7e80  ./tools/demos.awk
72b2a810b945c9fbaff79f2fe89c45432e616b358217fb260cf444f9d72ff044  ./tools/group.awk
```

第 K1.0 节按族相加的命令（在 `expected/` 里）与原样输出：

```
awk '{split($4,a,"="); if (a[1] ~ /^(crash_points|crash_states_enumerated|publishes|rollbacks|sweeps|sweep_rows_deleted|sweep_units_wiped|truncations|truncations_to_zero|shrinks_of_an_object_whose_record_was_moved_to_another_leaf|regrows_of_an_object_truncated_to_zero|records_with_data_moved_to_another_leaf|zero_versions|drops|absorbs|borrows|holes)$/) s[$1" "a[1]]+=a[2]} END{for(k in s) print k"="s[k]}' counts.txt | sort
holes absorbs=172
holes borrows=0
holes crash_points=236
holes crash_states_enumerated=1528
holes drops=246
holes holes=686
holes publishes=10732
holes records_with_data_moved_to_another_leaf=110
holes regrows_of_an_object_truncated_to_zero=66
holes rollbacks=108
holes shrinks_of_an_object_whose_record_was_moved_to_another_leaf=88
holes sweep_rows_deleted=270
holes sweeps=76
holes sweep_units_wiped=257
holes truncations=336
holes truncations_to_zero=194
holes zero_versions=0
k1 absorbs=9501
k1 borrows=2323
k1 crash_points=13619
k1 crash_states_enumerated=89764
k1 drops=5543
k1 holes=0
k1 publishes=692257
k1 records_with_data_moved_to_another_leaf=11245
k1 regrows_of_an_object_truncated_to_zero=1567
k1 rollbacks=6049
k1 shrinks_of_an_object_whose_record_was_moved_to_another_leaf=1605
k1 sweep_rows_deleted=17275
k1 sweeps=8830
k1 sweep_units_wiped=13694
k1 truncations=4639
k1 truncations_to_zero=2991
k1 zero_versions=4428
z absorbs=2976
z borrows=455
z crash_points=2512
z crash_states_enumerated=19198
z drops=621
z holes=0
z publishes=133496
z records_with_data_moved_to_another_leaf=3079
z regrows_of_an_object_truncated_to_zero=2973
z rollbacks=1520
z shrinks_of_an_object_whose_record_was_moved_to_another_leaf=3106
z sweep_rows_deleted=2969
z sweeps=1034
z sweep_units_wiped=2247
z truncations=6433
z truncations_to_zero=6012
z zero_versions=0
```

## K1　三个候选在克隆头、w = 2、清扫删行三处

### K1.0　装置与取样（我的模型，不是 E177 装置，也不是入库装置）

- 两份模型，都在 `research/prompts/m3-leaf-death-r3-opus-s2-model/`：
  - `src/`（Rust）：从第二轮攻方模型拷起，加了四样——墓碑容器（类型 1，unlink 意图那次追加整对象墓碑、随现行，照 `crates/` 今天的写法「tombstone: TombstoneRecord::of_the_whole_object(inode, record.object_birth, txg),」（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-core/src/transaction.rs:5997`））；候选二的写路径（删空与被吸收的右半写零条目版本、留在叶序里，合并与借只在有记录的片之间，照第二轮辩方的最强形态）；实例表删行的清扫；截短 / 截短到 0 / 长回来 / 打洞（对照）。读法：`C1t`（候选一 + 改法甲）、`r_old` 臂上的 `C1t`（改法乙）、`TSZc`（TSZ，墓碑容器只按 P2 择版本）、`TSZu`（TSZ，墓碑容器与叶同一套 P2 + 版本谱系滤）、`C2Hold`（候选二，零条目路径上每组取最新、同号取诞生代号最大的供出）；消融 `_noP2`、`_noLin` 与对照 `S`、`data_RG`。TSZ 另记 `_out`：删除队列里那一个对象（unlink 途中）不算漏活，那是它自报的 X3 代价。
  - `clone/clone_heads.py`：两个头（origin O、克隆 K，在 txg F 分叉、共享 O 那一刻的叶），跨头首次 COW 重生照 D8：「保留容器号、只换出生树与出生代，同时给旧身份写退役记录」（`.claude/kb/decisions/08-核心索引结构.md:171`）；读法带祖先表（克隆头读 origin 出生树的单元只取诞生代号 ≤ F 的，退役 / 墓碑只认本头的与 origin 死亡 txg ≤ F 的）与不带（`_naive`）。没有崩溃与回退（限度）。
- 取样（Rust）：族 k1（三个候选并排）2 形状（叶 6 / 233）× 3 几何（G-宽、G-环取最低、G-环随机）× 5 写路径臂（不借、N1、N2、零条目不借、零条目 N1）× 退役回退臂（随现行 / R_old，零条目臂只一臂）× 种子 8 / 4 = 288 段；族 z（K2）72 段；族 holes（阳性对照）8 段；每次挂载末尾千分之 500（族 z、holes 千分之 300）跑一次清扫。估时：先跑 6 段，叶 6 那 6 段 0.9 秒、叶 233 那 6 段 8.2 秒（6 线程），推全量约 3 分钟，不缩。正式分五批 96 / 96 / 48 / 48 / 80 段，挂钟 8.0、4.4、41.9、29.0、23.8 秒（交回前那次复跑的 `sweep-*.err`），批不设限时。复跑两次逐份相同（`compare.sh` 12 份 same，见「复跑与文件」）；`expected/determinism.txt`：「R2 name=determinism histories=46 two_runs_identical=true」（`research/prompts/m3-leaf-death-r3-opus-s2-model/expected/determinism.txt:1`）。
- 走到的形状（`expected/counts.txt` 按族相加，命令原样在「复跑与文件」末尾）：族 k1 崩溃点 13 619、枚举崩溃状态 89 764（在飞发布的单元写逐子集全枚举，k ≤ 8）、发布 692 257、回退 6 049、清扫 8 830 次（删行 17 275、抹掉未发布单元 13 694）、删空 5 543、吸收 9 501、借 2 323、零条目版本 4 428；族 z 崩溃状态 19 198、清扫 1 034 次；族 holes 崩溃状态 1 528。三族崩溃状态合计 110 490，没有一段历史超过 10⁶。`crates/` 的 `enumerate_layer0` 没调（`crates/` 没有退役、零条目、区间墓碑、克隆头的写者，没东西可录）。

### K1.1　克隆头共享叶：候选二打中复活（量过）；候选一、TSZ 没打中

`expected/clone.txt`（深度 0–4 的全部后缀，菜单 8 样：K / O 各 建、删最小号、删最大号、改最小号；每臂 4 681 个后缀，固定的只有前缀「O 建 6 个、在 txg 7 分叉 K」）：

| 写路径臂 | 读法 | 克隆头 K | origin O |
|---|---|---|---|
| 摘叶，重生写旧身份退役（D8 原样） | C1 | 0 / 4681 | 0 |
| 摘叶，重生写旧身份退役 | TSZ | 0 / 4681 | 0 |
| 摘叶，重生不写退役 | C1 | r > 0：2913 / 4681，峰值 4 | 0 |
| 摘叶，重生不写退役 | TSZ | 0 / 4681 | 0 |
| 零条目一直留（候选二） | C2（不读退役、不读墓碑） | r > 0：2913 / 4681，峰值 4 | 0 |
| 零条目一直留，重生写旧身份退役 | C2r（读那几条退役） | 0 / 4681 | 0 |
| 零条目一直留 | C2T（加 TSZ 那一套墓碑读法） | 0 / 4681 | 0 |
| 各臂 | `_naive`（不分头） | 全臂都中（例「R3C arm=drop_path_rebirth_retirement head=K reader=C1_naive suffixes=4681 suffixes_r_pos=3751」） | 同 |

原行：「R3C arm=zero_path_rebirth_retirement head=K reader=C2 suffixes=4681 suffixes_r_pos=2913 suffixes_m_pos=0 r_max=4 m_max=0 first=K.unlink_min」（`research/prompts/m3-leaf-death-r3-opus-s2-model/expected/clone.txt:33`）；「R3C arm=zero_path_rebirth_retirement head=K reader=C2r suffixes=4681 suffixes_r_pos=0 suffixes_m_pos=0 r_max=0 m_max=0 first=none」（`research/prompts/m3-leaf-death-r3-opus-s2-model/expected/clone.txt:32`）；「R3C arm=drop_path_no_rebirth_retirement head=K reader=TSZ suffixes=4681 suffixes_r_pos=0 suffixes_m_pos=0 r_max=0 m_max=0 first=none」（`research/prompts/m3-leaf-death-r3-opus-s2-model/expected/clone.txt:4`）。

可复跑的最小反例（`python3 -I clone/clone_heads.py trace zero_path_rebirth_retirement K.unlink_min`，`expected/clone-trace.txt`）：O 建 1..6（叶 (O,1,1) 装 1..3、(O,4,4) 装 4..6）→ txg 7 分叉 K → K 删 1 号：意图那次 K 第一次改 (O,1,1)，重生成 (K,1,8)；最后一批 (K,1,8) 剩 2、3。K 的扫盘重建：(K,1,8) 最新一版只有 2、3；(O,1,1) 在 F 之前那一版（诞生代号 3）装着 1、2、3，候选二没有任何东西说这个身份在 K 里已经被换掉，1 号从它那里供出来——「R3C trace head=K reader=C2 rebuilt=[(1, 1, 1, 1), (2, 2, 1, 2), (3, 3, 1, 3), (4, 4, 1, 4), (5, 5, 1, 5), (6, 6, 1, 6)]」（`research/prompts/m3-leaf-death-r3-opus-s2-model/expected/clone-trace.txt:24`），真值「R3C trace head=K truth=[(2, 2, 1, 2), (3, 3, 1, 3), (4, 4, 1, 4), (5, 5, 1, 5), (6, 6, 1, 6)]」（`research/prompts/m3-leaf-death-r3-opus-s2-model/expected/clone-trace.txt:23`）。叶没删空也中：病不在叶死亡怎么表达，在「重生之后旧身份在克隆头里还活着」。

四句：
- 分不分辨臂：分辨。候选一（D8 本来就要重生写旧身份退役）0；TSZ 不写退役也 0（墓碑在所有版本里压掉被删的号，挪动、改写由同号取诞生代号最大的供出）；候选二照第二轮辩方的读法（不读退役、不读墓碑）中。
- 判别子看不看得到：看得到。旧身份 (O,1,1) 与新身份 (K,1,8) 都在盘上，区分「K 里谁现行」的是重生那一刻写不写一条点名旧身份的东西（退役记录）或删号写不写墓碑；候选二两样都不读。
- 满足的是判据哪一句：正文第五节第二条「被攻方坐实一形复活」——候选二那一形（零条目版本一直留、改 I-9.13，不写退役）在克隆头上复活。
- 改法在打中的格上还中不中：改法 A「候选二也保留 D8 的重生退役记录（C2r）」不中，但候选二就要带候选一的载体（退役树、树表条目、I-9.16 十一种变十二种这一整套），「盘上不加退役」的那一半代价优势没了（推的）；改法 B「候选二改读意图那次的整对象墓碑（C2T）」不中，但它就带上 TSZ 的 X3 代价（unlink 途中漏活）。两个改法都只在我的模型上量过、被攻过零轮。

三个候选共用的前提（不分辨候选）：不带祖先表的读法（`_naive`）全臂都中——txg 全池一条，按 I-9.18 那条「其余 txg 的父是 txg − 1」的父链，克隆头会把 origin 分叉之后写的退役、墓碑、零条目版本都认进来。kb 自己写着这一格没有输入：「多头镜像的归属划分要克隆祖先表」（`.claude/kb/decisions/08-核心索引结构.md:207`）；墓碑不带树那一半在 C347：「墓碑记录只有对象 ID + 对象出生代 + 区间，不带树」（`.claude/kb/checks-owed.md:282`）。候选一的退役记录、TSZ 的墓碑都要「本头的 + origin 死亡 txg ≤ 分叉的」，候选二的零条目版本要「origin 诞生代号 ≤ 分叉的」；三者都靠容器的出生树分头（类型 1 那一半 kb 已写「别的头在扫描重建时靠祖先表读到它们」（`.claude/kb/decisions/18-块里携带什么信息.md:301`））。

### K1.2　清扫删实例表行：没打中（量过），不分辨候选

装置：每次可写挂载末尾按概率跑一次清扫（`src/model.rs` 第 449 行起 `sweep`）：根环每个槽都读得出才判（「if self.ring.iter().any(|e| !e.2) { self.stats.add("sweeps_not_deleting_unreadable_root_slot", 1); return; }」（`research/prompts/m3-leaf-death-r3-opus-s2-model/src/model.rs:451`）），根环里没有某实例的根就抹掉那个实例的全部未发布单元、删它的行；删行之后那个实例剩下的单元照「无行 ⇒ 已发布」判，照 D18 已定项 11 那一句「只对同一时间线成立」（`.claude/kb/decisions/18-块里携带什么信息.md:314`）。

| 取样 | C1t（候选一甲 / 乙） | TSZc（`_out`） | C2Hold（候选二） |
|---|---|---|---|
| 族 k1 288 段（清扫 8 830 次、删行 17 275、抹 13 694 个单元） | inode、data_Z 36 格全 0 | inode_out、data_Z_out 36 格全 0 | 12 格全 0 |
| 族 z 72 段（清扫 1 034 次） | 12 格全 0 | 12 格全 0 | —（这一族不走零条目） |
| 手造 6 景 × 57 个后缀（删空那次崩、零条目那次崩、意图那次崩、回退跨删空之后崩，各接根环转过去 + 清扫，之后放开扫 0–2 步） | 全 0 | 全 0 | 全 0 |

原行例：「k1 C1t inode cells=36 histories=216 cells_r_pos=0 cells_m_pos=0 r_max=0 m_max=0」（`research/prompts/m3-leaf-death-r3-opus-s2-model/expected/group.txt:41`）、「k1 TSZc inode_out cells=36 histories=216 cells_r_pos=0 cells_m_pos=0 r_max=0 m_max=0」（`research/prompts/m3-leaf-death-r3-opus-s2-model/expected/group.txt:61`）、「k1 C2Hold inode cells=12 histories=72 cells_r_pos=0 cells_m_pos=0 r_max=0 m_max=0」（`research/prompts/m3-leaf-death-r3-opus-s2-model/expected/group.txt:50`）；手造那几景在 `expected/demos.txt` 第 1–56 行（`s_` 开头），前缀里真的删了行、抹了单元：「R3 name=demo_counts scenario=s_crash_in_the_unlink_intent_then_sweep suffix=none crash_points=1 crash_states_enumerated=4 sweeps=1 sweep_rows_deleted=1 sweep_units_wiped=2」（`research/prompts/m3-leaf-death-r3-opus-s2-model/expected/demo-prefix-counts.txt:3`）。装置看得见 P2 出错：同几景的 `_noP2` 消融非 0（例 `expected/demos.txt` 里 `s_crash_in_the_unlink_intent_then_sweep TSZc_noP2 inode_out` 57 个后缀全 r > 0）。

为什么不中（推的，与量到的相容）：清扫按 D18 已定项 11「回收」那一条是「先抹掉那个实例的全部未发布单元，再删行」，删行前后已发布谓词对盘上剩下的每一个单元给同一个答案；三个候选滤掉崩溃抛弃那一段，靠的都是同一个 P2（候选一的退役容器、TSZ 的墓碑容器、候选二的零条目版本都是码 3 单元）。所以删行不改变任何一个候选的读数，也就分辨不了它们。

剩下一条不分辨候选的口子（推的，没量）：清扫认「是不是单元」只看头校验和（「只看头校验和，不比任何读盘判定更严」（`.claude/kb/decisions/18-块里携带什么信息.md:315`），`crates/` 那一处是「UNIT_CLASS_PACKED => usize::try_from(PACKED_UNIT_HEADER_BYTES).expect("107"),」（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-core/src/instance_table_sweep.rs:195`））。一份未发布单元在清扫那一次读成头校验和不过（读成功、内容错，不报读失败），它不被抹、行照删；之后读对了，它按「无行 ⇒ 已发布」生效。三个候选都中（退役记录、墓碑、零条目版本、叶都一样是码 3），不分辨；属清扫门本身，不属这一题。

### K1.3　副本坏一份（w = 2）：没量，推的不分辨候选

模型里一个单元一份（限度）。逐形推：
- 一份读不出 / 头校验和不过 / 载荷校验和不过：另一份读得出，择版本不变（I-1.8 的副本归并只在两份都可读时比载荷）；三个候选读的都是「那一组里读得出的一份」，没有一个候选依赖某个单元「不在」。
- 错向写落在一份上：被盖掉的那个单元少一份（同上）；被写错地方的那一份头里自述地址不对，不当单元。旧版本在另一盘上多活一阵，三个读法本来就要扛「旧版本都读得到」（G-宽）。
- 丢失写：D18 已定项 21 定为不在自描述的射程里，三个候选同等暴露，不分辨。
- 一块盘整盘不在：第一版两盘掉一块只读挂载，清扫不跑；扫盘只有一份，同第一条。
- 两份都坏（不是这一格，记一笔代价）：最新那一版没了就退回上一版。爆炸半径按候选不同（推的）：候选一一个退役容器装 959 条（第二轮攻方的载体定义），它的最新版两份都没了，上一版之后写进去的那几条身份复活；关掉的满容器两份都没了，959 个身份一起复活；TSZ 的墓碑容器 815 条同形；候选二一个零条目版本只管一片叶。这是「把多少死亡放进同一个单元」的账，不是复活形态。

### K1.4　TSZ 的墓碑容器怎么择版本：按版本谱系滤就复活（量过），只按 P2 择不中

第二轮判决把这一形记给 C615（墓碑树按统一谱系滤会照第二轮那一形丢掉链上的墓碑，推的）。TSZ 用墓碑在所有版本里压掉被删的号，墓碑丢了就直接复活，所以它在 TSZ 上是读数，不只是一笔账。这一轮在崩溃 + 回退 + G-环 + 清扫上量了（第二轮辩方的模型没有崩溃状态，第二轮攻方的模型没有墓碑树）：

- 「k1 TSZu inode_out cells=36 histories=216 cells_r_pos=12 cells_m_pos=0 r_max=3 m_max=0」（`research/prompts/m3-leaf-death-r3-opus-s2-model/expected/group.txt:73`）、数据「k1 TSZu data_Z_out cells=36 histories=216 cells_r_pos=12 cells_m_pos=0 r_max=8 m_max=0」（`research/prompts/m3-leaf-death-r3-opus-s2-model/expected/group.txt:71`）；族 z 4 / 12 格。中的格全在叶 6 的两种 G-环上，G-宽 0（`expected/agg.txt` 里 `TSZu inode_out` 50 行，G-宽那 17 行 `histories_r_pos=0`）。
- 同一批历史上 `TSZc`（墓碑容器只按 P2 择，就是第二轮给退役树的改法甲）inode_out、data_Z_out 全 0（上一节表）。
- 可复跑的反例：`trace 64`（族 k1、叶 6、G-环取最低、不借、种子 1），`expected/trace-64-first.txt`。242 号在 txg 970 发意图（墓碑 242/938@970 进墓碑容器 80），各批到 976；txg 992 管理员回退到 973（谱系记录 (992, 973)），队列里的 242 续做，994 最后一批删掉它、它那片叶 (230, 889) 删空摘掉。到 txg 1006：链上那条墓碑 970 在谱系上，但容器 80 留在盘上的那一版诞生代号 980，落在被回退抛弃的 (973, 992) 里——「tombstone container Ident { num: 80, birth: 970 } version birth=980 inst=39 slot=98 on_chain=false published=true referenced_by_the_current_root=true」（`research/prompts/m3-leaf-death-r3-opus-s2-model/expected/trace-64-first.txt:116`），诞生代号 970 那一版的槽已被环再分配；按版本谱系滤一版都不剩，242 从那片叶链上的旧版供出来：「leaf Ident { num: 230, birth: 889 } version birth=993 inst=39 slot=51 on_chain=true published=true live_in_truth=false records_not_in_truth=[242]」（`research/prompts/m3-leaf-death-r3-opus-s2-model/expected/trace-64-first.txt:3`）。

四句：分辨「墓碑容器怎么择版本」这一维，不分辨候选一对 TSZ（候选一按谱系滤退役树是同一个病，第二轮已量；候选二的零条目版本住随 R_old 的 inode 树，不受这一形，`C2Hold` 0）；判别子看得到（随现行的树在规则层面分出来，`TSZc` 0）；满足的是「坐实一形复活」——被打中的是 TSZ 没写明的那一句（墓碑容器的择版本规则）；改法「随现行的墓碑树只按 P2 择版本、墓碑按死亡 txg 在不在谱系上滤」在打中的格上不中（量过，只在我的模型上，被攻过零轮）。TSZ 交用户时的定义要把这一句写进去，它与 C615 读侧是同一句。

## K2　读法 Z 与区间墓碑

### K2.1　Z 换三组新历史：没打中（量过）；前提（文件没有洞）一破就中

Z 的定义照第二轮攻方：重建出的 inode 树里每个对象取一条记录（同号取诞生代号最大的那一版供出的），数据单元只在「对象出生代相等且锚点 < size 折成的单元数」时收；装置里是 `C1t data_Z`（inode 读对的前提下判 Z）。这一轮新加的写路径比 `crates/` 今天宽（`crates/` 只有第一个文件写数据、每次写都从偏移 0 重写全部单元）：截短只改 size、留下的单元不重写；截短到 0；顺序写只追加新锚点（长回来）。

| 历史 | 走到的次数（族 z 72 段，`expected/counts.txt` 相加） | Z（`C1t data_Z`） | 对照 `data_RG`（只看记录在不在） |
|---|---|---|---|
| 记录被借走 / 被吸收之后对象再变短 | 挪过叶的对象被截短 3 106 次（带数据的记录挪叶 3 079 次） | 12 格全 0 | 12 格全 r > 0，峰值 317 |
| 多对象交错变短与删除 | 截短 6 433、截短到 0 6 012，与 unlink 各批（各批也截短）交错 | 同上 | 同上 |
| 截短到 0 再长回来 | 刚截到 0 的对象被追加 2 973 次 | 同上 | 同上 |

原行：「z C1t data_Z cells=12 histories=72 cells_r_pos=0 cells_m_pos=0 r_max=0 m_max=0」（`research/prompts/m3-leaf-death-r3-opus-s2-model/expected/group.txt:80`）、「z C1t data_RG cells=12 histories=72 cells_r_pos=12 cells_m_pos=0 r_max=317 m_max=0」（`research/prompts/m3-leaf-death-r3-opus-s2-model/expected/group.txt:78`）。族 k1 的 288 段里也有截短（4 639 次）与截短到 0（2 991 次），`C1t data_Z` 36 格全 0（`expected/group.txt` 第 40 行）。

手造四景，固定前缀、之后放开扫 0–3 步（菜单 8–9 样，含截短到 1、截短到 0、追加、覆盖写、unlink、回退到最旧候选、下一次发布崩溃逐子集枚举）：
- 「z_borrow_then_shrink_n1 C1t data_Z suffixes=585 suffixes_r_pos=0 suffixes_m_pos=0 r_max=0 m_max=0」（`research/prompts/m3-leaf-death-r3-opus-s2-model/expected/demos.txt:71`）
- 「z_absorb_then_shrink C1t data_Z suffixes=585 suffixes_r_pos=0 suffixes_m_pos=0 r_max=0 m_max=0」（`research/prompts/m3-leaf-death-r3-opus-s2-model/expected/demos.txt:59`）
- 「z_interleaved_shrink_and_unlink C1t data_Z suffixes=820 suffixes_r_pos=0 suffixes_m_pos=0 r_max=0 m_max=0」（`research/prompts/m3-leaf-death-r3-opus-s2-model/expected/demos.txt:83`）
- 「z_truncate_to_zero_then_regrow C1t data_Z suffixes=585 suffixes_r_pos=0 suffixes_m_pos=0 r_max=0 m_max=0」（`research/prompts/m3-leaf-death-r3-opus-s2-model/expected/demos.txt:95`）
同几景的 `data_RG` 对照 379–820 个后缀 r > 0（第 58、70、82、94 行）。

阳性对照（族 holes，8 段）：加「size 变长而不写单元」（打洞，`crates/` 今天拒：「这个文件的 extent 记录条数与 inode size 按净荷容量除出来的单元数不符」（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-core/src/recovery.rs:4901`））之后 Z 中：「holes C1t data_Z cells=2 histories=8 cells_r_pos=2 cells_m_pos=0 r_max=4 m_max=0」（`research/prompts/m3-leaf-death-r3-opus-s2-model/expected/group.txt:7`）——截短到 e 再把 size 撑回去而不写，锚点 ≥ e 的旧单元在谱系上、已发布、锚点 < 新 size，Z 把旧内容当成活的。装置看得见 Z 出错，三组新历史上的 0 不是装置瞎。

为什么三组新历史上不中（推的，与量到的相容）：没有洞时，锚点 a < 现行 size 的每一个，都在「size 最后一次落到 ≤ a」之后被写过；那一次写在现行版本的谱系上、已发布、诞生代号比之前每一版都大，每个锚点取 (诞生代号, 实例) 最大的那一版就是它。记录被借走只换装记录的叶，供出规则取诞生代号最大的那一版，后挪的那一版总是更新。

四句（对阳性对照那一形）：分辨（只打 Z，不写区间墓碑的读法才靠「没有洞」）；判别子在镜像上看得到（洞没有单元、旧单元的诞生代号早于变短那一次），被打中的是 Z 的前提，不是 Z 的读法；满足的是「Z 在截短到 0 再长回来上」那一格的一个变体（长回来不写单元），三组按 `crates/` 写法（长回来就写单元）都不中；改法「打洞时写零单元」或「打洞那一次写区间墓碑」都能补（推的，没量），`crates/` 今天拒打洞，所以这一形现在不可达。

### K2.2　区间墓碑的最小改法：放过今天红的五张坏镜像（仓副本上量过）

最小改法（我写的、只在仓副本上加了开关 `OPUS_R3S2_RANGE_TOMBSTONES_MINIMAL`，`copy/patch_copy.py`）：I-9.19 只判在谱系上的**整对象**墓碑；I-9.20 不重号那一句只判整对象墓碑；I-9.20 形状那一句从「第一版只许整对象 [0, 2⁶⁴ − 1]」（`.claude/kb/invariants.md:308`）放宽成「整对象或 [起, 2⁶⁴ − 1]、起 < 止」；死亡 txg 区间一句、每片 ≤ 815 一句不动。动的是 CPU 那三处：I-9.19 的谱系滤「.filter(|tombstone| lineage_chain.holds_txg(tombstone.death_txg))」（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-checker/src/walk.rs:10780`）、形状「judgements.judge(invariant, tombstone.covers_the_whole_object(), || {」（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-checker/src/walk.rs:10822`）、不重号（同文件第 10852 行那个谱系滤之后）。这是第二轮攻方改法表里「I-9.19 收成整对象」「I-9.20 不重号只判整对象」两行合起来，加上形状那一句不得不放宽。

镜像：1 号顺序写三个单元（txg 4）→ 建 2、3 号，3 号删完 → 覆盖写成一个单元（txg 8，「变短」）→ 顺序写回三个单元（9）→ 再变短（10）→ 再长回来（11）→ 根环转过一圈（到 txg 41）；墓碑树第一片容器整份换成表里的几条，checker 原样判（`expected/copy.txt` 第 1 行：「OPUSR3S2 k2 facts birth_of_inode_one=3 txg_three_units=4 txg_shrink=8 txg_first_regrow=9 txg_second_shrink=10 txg_regrow=11 death_after_the_regrow=12 newest_txg=41 live_units_of_inode_one=3」（`research/prompts/m3-leaf-death-r3-opus-s2-model/expected/copy.txt:1`））。表里 T3 = 3 号那条整对象墓碑 (3, 5, [0, 2⁶⁴ − 1], 6)，MAX = 2⁶⁴ − 1。

| 镜像（`expected/copy.txt` 行） | 墓碑 | 今天 | 最小改法 |
|---|---|---|---|
| 合法：变短那次的尾区间（14） | T3、(1, 3, [1, MAX], 8) | I-9.19、I-9.20 | 无 |
| 合法：变短 → 长回来 → 再变短（15） | T3、(1,3,[1,MAX],8)、(1,3,[1,MAX],10) | I-9.19、I-9.20 | 无 |
| **坏：死亡 txg 晚于长回来，压掉活的单元 1、2（3）** | T3、(1, 3, [1, MAX], 12) | I-9.19、I-9.20 | **无** |
| **坏：死亡 txg 等于长回来那次（4）** | T3、(1, 3, [1, MAX], 11) | I-9.19、I-9.20 | **无** |
| **坏：第二条在长回来之后（9）** | T3、(…, 8)、(…, 12) | I-9.19、I-9.20 | **无** |
| **坏：同一次变短写了两条一样的（8）** | T3、(…, 8) 两条 | I-9.19、I-9.20 | **无** |
| **坏：unlink 写成了尾区间、删完之后的版本（11）** | (3, 5, [1, MAX], 6) | I-9.20 | **无** |
| 坏：不收尾的区间（2） | T3、(1, 3, [0, 5], 8) | I-9.19、I-9.20 | I-9.20 |
| 坏：死亡 txg 早于对象出生代（5） | T3、(1, 3, [1, MAX], 2) | I-9.19、I-9.20 | I-9.20 |
| 坏：点名从没有过的对象（6） | T3、(999, 8, [1, MAX], 8) | I-9.6、I-9.20 | I-9.6 |
| 坏：出生代给错（7） | T3、(1, 4, [1, MAX], 8) | I-9.10、I-9.20、I-9.29 | I-9.10、I-9.29 |
| 坏：活对象上的整对象墓碑（12） | T3、(1, 3, [0, MAX], 8) | I-9.19 | I-9.19 |
| 坏：同一对象两条整对象墓碑（10） | T3 两条 | I-9.20 | I-9.20 |

原行（第 3、14 行）：「OPUSR3S2 k2 image=bad_tail_tombstone_after_the_regrow_kills_live_units…violated_today=["I-9.19", "I-9.20"] violated_minimal=[]」（`research/prompts/m3-leaf-death-r3-opus-s2-model/expected/copy.txt:3`）；「OPUSR3S2 k2 image=legit_tail_tombstone_of_the_shrink…violated_today=["I-9.19", "I-9.20"] violated_minimal=[]」（`research/prompts/m3-leaf-death-r3-opus-s2-model/expected/copy.txt:14`）。

- 打中一（判别子在 I-9.19 / I-9.20 读的那两棵树里看不到）：第 3 行与第 14 行的 inode 记录逐字相同、墓碑只差死亡 txg（12 对 8），分开它们的是「死亡 txg 比那几个活单元的诞生代号（11）晚还是早」，那个数住数据单元头与 extent 指针，I-9.19、I-9.20 只读 inode 树与墓碑树。只用这两棵树写的任何改法，要么第 14 行误红（今天），要么第 3 行放过（最小改法）。要两头都对，得加一句跨树的：「谱系上的尾区间墓碑 (号, 出生代, [起, MAX], 死亡 txg)，它点名的对象活着时，这一版 extent 树里这个对象锚点 ≥ 起的每个数据单元，诞生代号 > 死亡 txg」（推的，没给 checker 打补丁）。这已不是最小改法：I-9.19 要读 extent 树，GPU 判器那一处也要跟。
- 打中二（不重号那一句的判别力）：同一次变短写两条（第 8 行）今天红在 I-9.20 不重号上，最小改法不判区间那一半就放过；要留住它得按 (号, 出生代, 区间起, 死亡 txg) 判不重号——第 15 行那种两条只差死亡 txg，按这个键不重，按 (号, 出生代, 区间) 照第二轮所说会红。
- 打中三（unlink 写错成尾区间）：I-9.18 只在号还在队列里的那几版判它（队列里的号要有谱系上的整对象墓碑），删完之后的版本只剩「第一版只许整对象」这一句拦；最小改法放宽形状之后，第 11 行那种删完之后的镜像整张绿。崩溃点重放在意图到最后一批之间那几版上照样由 I-9.18 拦（推的）。

四句：分辨（只打区间墓碑那一臂，Z 不写墓碑不碰）；判别子：打中一在 I-9.19 / I-9.20 读的东西里看不到、在镜像上看得到（数据单元的诞生代号），属「改判据，改完是新一轮」那一类；满足的是 K2 第二问「最小改法会不会放过今天 I-9.19、I-9.20 拦的坏镜像」——会，五张；改法：加上面那句跨树判据修打中一（推的），不重号的键加死亡 txg 修打中二（推的），打中三要「unlink 那条墓碑恒整对象」另立一句或让 I-9.18 判到删完之后（推的）。三个改法都只在我这里写出来、被攻过零轮，没在 checker 上实现。

## K3　删 I-9.4 第三句之后，C116「右半容器号给错」哪一形没有判据拦

I-9.4 四个小句：记录号 ≥ 容器号（CPU「self.judgements.judge("I-9.4", inode >= container, || {」（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-checker/src/walk.rs:3079`））、沿叶序容器号严格递增（「.judge("I-9.4", left.container < right.container, || {」（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-checker/src/walk.rs:2775`））、左最大 key < 右容器号（要删的那一句，`walk.rs:2782`）、片内记录号严格递增。仓副本加开关 `OPUS_R3S2_DROP_I94_THIRD` 让第三句恒真（N1 那一形），`OPUS_R3S2_N1_SEPARATOR` 让写路径的分隔 key 取这一版的最小 key（今天是「self.identity.container」（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-core/src/inode_tree.rs:83`）），`OPUS_R3S2_RIGHT_HALF_NUMBER_OFFSET=k` 让分裂出来的右半容器号写成新记录号 − k（今天「container: inode.0,」（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-core/src/inode_tree.rs:244`））。

### K3.1　一族镜像：第二、三片的容器号扫一遍（`expected/copy.txt` 第 16–66 行）

叶 [1..150][234..466][467..699][700]（左片先删到 150 条，留出 (150, 234) 那一段）；改第二片（左邻容器号 1、左最大 key 150、本片最小 234）或第三片（左邻 234、左最大 466、本片最小 467）的容器号，叶头与根条目的身份引用一起改（I-9.2 照成立）；分隔 key 两种：跟着容器号走（今天的代码，号写错时分隔 key 一起错）与这一版最小 key（N1）。

| 右半容器号 c 落在哪 | 分隔 key 跟着号走：今天 / 删第三句 | 分隔 key = 最小 key：今天 / 删第三句 |
|---|---|---|
| c ≤ 左邻容器号（0、1；第三片 0、233、234） | I-9.4（第二句）、I-9.12，有的再加 I-1.1 / I-1.8；删了照红 | I-9.4（第二句），234 那一格加 I-1.8；删了照红 |
| **左邻容器号 < c ≤ 左最大 key（2、100、150；235、300、466）** | I-9.4、I-9.12 / **只剩 I-9.12** | I-9.4 / **全绿** |
| 左最大 key < c < 本片最小 key（151、200、233） | 全绿 / 全绿（今天就拦不住） | 全绿 / 全绿 |
| c = 本片最小 key（正确） | 全绿 | 全绿 |
| c > 本片最小 key（235、466、467、468；468、699、700、701） | I-9.4（第一句）、I-9.12 等；删了照红 | I-9.4（第一句）等；删了照红 |

原行：「OPUSR3S2 k3 image=leaf1_number150_separator_smallest_key leaves=[no=1 sep=1 1..150][no=150 sep=234 234..466][no=467 sep=467 467..699][no=700 sep=700 700..700] violated_today=["I-9.4"] violated_without_the_third_clause=[]」（`research/prompts/m3-leaf-death-r3-opus-s2-model/expected/copy.txt:22`）；「OPUSR3S2 k3 image=leaf2_number235_separator_smallest_key leaves=[no=1 sep=1 1..150][no=234 sep=234 234..466][no=235 sep=467 467..699][no=700 sep=700 700..700] violated_today=["I-9.4"] violated_without_the_third_clause=[]」（`research/prompts/m3-leaf-death-r3-opus-s2-model/expected/copy.txt:50`）；分隔 key 跟着号走时删了第三句照红 I-9.12：「OPUSR3S2 k3 image=leaf1_number150_separator_follows_the_number leaves=[no=1 sep=1 1..150][no=150 sep=150 234..466][no=467 sep=467 467..699][no=700 sep=700 700..700] violated_today=["I-9.4", "I-9.12"] violated_without_the_third_clause=["I-9.12"]」（`research/prompts/m3-leaf-death-r3-opus-s2-model/expected/copy.txt:21`）。

### K3.2　写路径上写错：整池建 699 个 inode，判据一条不红；只有 inode_tree 的单测红

- 「OPUSR3S2 k3 image=writer_right_half_number_minus_1_n1_separator_true leaves=[no=0 sep=1 1..233][no=233 sep=234 234..466][no=466 sep=467 467..699][no=699 sep=700 700..700] violated_today=["I-9.4"] violated_without_the_third_clause=[]」（`research/prompts/m3-leaf-death-r3-opus-s2-model/expected/copy.txt:70`）
- 「OPUSR3S2 k3 image=writer_right_half_number_minus_40_n1_separator_true leaves=[no=0 sep=1 1..233][no=194 sep=234 234..466][no=427 sep=467 467..699][no=660 sep=700 700..700] violated_today=["I-9.4"] violated_without_the_third_clause=[]」（`research/prompts/m3-leaf-death-r3-opus-s2-model/expected/copy.txt:71`）
- 分隔 key 照今天跟着号走：「violated_today=["I-9.4", "I-9.12"] violated_without_the_third_clause=["I-9.12"]」（`research/prompts/m3-leaf-death-r3-opus-s2-model/expected/copy.txt:69`）
- 同一个开关下 `cargo test -p singlefs-core --lib -- inode_tree`：16 条里 5 条红——「offset=1 n1_separator=1 rc=101 test result: FAILED. 11 passed; 5 failed; 0 ignored; 0 measured; 187 filtered out」（`research/prompts/m3-leaf-death-r3-opus-s2-model/expected/corelib.txt:14`），红的里有「the_two_hundred_thirty_fourth_record_splits_at_the_end_and_the_left_half_keeps_everything」（同文件第 19 行），开关关着全绿（第 1 行）。

（写错的那一版第一片容器号也成了 0，`inode.0 − 1` 落到了建第一片的同一条路径上；第一片的号没有左邻，今天的 I-9.4 也不比它，那一格不在这一题里，没另造镜像。）

结论：删第三句之后（N1：分隔 key = 这一版最小 key），**右半容器号给小了、但仍大于左邻容器号**（左邻容器号 < c ≤ 左最大 key）这一形，CPU checker 一条不红（量过）；GPU 判器那两句是「w_judge(INVARIANT_I_9_4, lt64(left_container, right_container));」（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2917`）与要删的第 2918 行，I-9.12 那几句比的是分隔 key，同样不红（读码推的，GPU 没跑）；层 0 的崩溃点重放每个状态只跑 checker，同样拦不住（推的）。剩下拦它的只有写路径纪律那一层：`inode_tree` 的单测（量过，5 / 16 红），即 C116 的「单测 / 模型对拍」。第一句拦给大了的，第二句拦给到左邻及以下的；I-9.12 只在分隔 key 跟着容器号走时顺带拦（N1 下分隔 key 不再跟号，就不拦了）。

这一形有没有后果（推的）：N1 下查找按分隔 key（最小 key）走，号给小了不改路由；容器身份 (出生树, 号, 出生代) 的出生代 = 这次发布，同一次发布里两片同号由第二句与 I-1.8 拦（第三片号给成 234 那一格 I-1.8 红，`expected/copy.txt` 第 48 行），所以身份不撞，候选一按身份点名的退役记录也不受影响。它违反的是 D8「右半容器号 = 那条新记录的 inode 号」那一句写路径纪律，镜像上没有可见后果。左最大 key < c < 本片最小 key 那一段今天就拦不住（`expected/copy.txt` 第 24、30 行今天也全绿），不是删第三句带来的。

四句：分辨（只打 N1 那一臂；N2 不删第三句，同一张镜像今天的判据照红）；判别子：在 N1 下这一形与「左片借走右片最小那几条之后」的合法形状在镜像上不可区分（第一轮判决已推、这一轮整族量过），属「判别子观测不到」——不是判据写错，是 N1 让这个量合法了；满足的是 K3「拦不住的那一形写出来」；改法：留着第三句就与 N1 合法的借冲突（第二轮量过），要拦这一形只能在写路径上（单测、模型对拍、C116 第二格的崩溃点重放带 oracle 比容器身份集合），镜像判据里没有可加的一句（推的）。

## 改法表（我提的改法：只在我的模型或仓副本上量过、被攻过零轮）

| 改法 | 修哪一格 | 在打中的格上 | 量过 / 推的 |
|---|---|---|---|
| 候选二保留 D8 的重生退役记录（C2r） | K1.1 克隆头复活 | 不中：4681 个后缀 0（`expected/clone.txt` 第 32 行）；代价是候选二也要退役树载体 | 量过（克隆模型） |
| 候选二改读意图那次的整对象墓碑（C2T） | K1.1 | 不中：0（`expected/clone.txt` 第 34 行）；代价是带上 X3 的 unlink 途中漏活 | 量过（克隆模型） |
| 三个候选的读侧都按祖先表分头（本头的 + origin 截到分叉的） | K1.1 的 `_naive` 那一格（共用前提） | 不分头全中，分头之后各候选回到上表 | 量过（克隆模型）；条款没写（C347、D8 射程那一句） |
| TSZ 的墓碑容器只按 P2 择版本（改法甲搬到墓碑树） | K1.4 | 不中：`TSZc` inode_out / data_Z_out 0；`TSZu` 12 / 36 格中 | 量过（Rust 模型） |
| I-9.19 加一句跨树：谱系上的尾区间墓碑点名的活对象，锚点 ≥ 起的数据单元诞生代号 > 死亡 txg | K2.2 打中一 | 推的不中：第 3、14 行只差这一个数 | 推的，没给 checker 打补丁 |
| I-9.20 不重号的键改成 (号, 出生代, 区间起, 死亡 txg) | K2.2 打中二 | 推的不中（第 8 行两条键相同，第 15 行键不同） | 推的 |
| unlink 那条墓碑恒整对象另立一句（或 I-9.18 判到删完之后） | K2.2 打中三 | 推的不中（第 11 行） | 推的 |
| 最小改法（I-9.19、I-9.20 不重号只判整对象，形状放宽到尾区间） | 区间墓碑落地 | 照中：五张坏镜像全绿（`expected/copy.txt` 第 3、4、8、9、11 行） | 量过（仓副本，checker 打了开关） |
| N1 下拦「右半号给小了」：只能在写路径上（单测、模型对拍、崩溃点重放带 oracle） | K3 | 单测 5 / 16 红（`expected/corelib.txt`）；镜像判据里没有可加的一句 | 单测那一半量过；镜像判据推的 |

## 没打中的形状（一次抽样，照规则不算「没打中」）

- 候选一（改法甲 / 乙）、TSZc、候选二（C2Hold）在清扫删行上：族 k1 288 段、族 z 72 段（崩溃状态 108 962 个，清扫 9 864 次、删行 20 244 行），手造 6 景 × 57 个后缀，全 0。
- 候选一、TSZ 在克隆头上（带祖先表、候选一带 D8 的重生退役）：候选一 1 个臂、TSZ 2 个臂，各 4 681 个后缀（深度 0–4、菜单 8 样），全 0；没有崩溃与回退。
- Z 在三组新历史上：族 z 72 段（挪过叶的对象截短 3 106 次、截短到 0 后追加 2 973 次）、族 k1 288 段、手造 4 景（585 / 585 / 820 / 585 个后缀），全 0。
- w = 2：没建模，逐形推（K1.3）。
- K3：右半号给大了、给到左邻及以下，删第三句之后照红（第一句、第二句、I-1.8），没打中「拦不住」。

## 这条腿自己的限度

- Rust 模型不是 `crates/`：数据单元直接按 (对象, 出生代, 锚点) 落槽，没有 extent 树、分配记录、记账、映射；单元一个槽一份，没有 w = 2 副本；读侧是手写的 P2 / 版本谱系滤 / 择新 / 供出规则，不是 `crates/` 的扫盘重建（`crates/` 没有）。清扫照 D18 已定项 11 那一条写成「根环全读得出才判、先抹该实例全部未发布单元再删行」，它对候选不中在一定程度上是写法本身带来的（K1.2 的推论）；清扫「跳过可读根账里仍分配的槽」那一句没建（模型的根槽读不出就当这一轮不删）。
- 崩溃只在「在飞发布的单元写」那一段逐子集枚举，k > 8 的发布不当崩溃点（族 k1 跳过 145 次、族 z 11 次，`expected/counts.txt` 的 `crash_points_skipped_more_writes_than_k_max`），这是取样的缩法，不是崩溃状态的缩法；journal、根槽两段没建。
- 截短、截短到 0、只追加新锚点的顺序写比 `crates/` 今天宽（`crates/` 只有第一个文件写数据、每次从偏移 0 重写全部单元）；K2.1 的三组历史是按将来的写路径造的，今天的写路径是它的子集。
- 克隆模型：两个头、一次分叉、没有崩溃 / 回退 / 环、没有数据单元、没有克隆的克隆；txg 全池一条、一次发布只改一个头。D6 的克隆头在 `crates/` 里不存在，读侧的「分头」是我按 D8 / D18 的条文补的最强形态。
- 仓副本：区间墓碑、N1 的分隔 key 在 `crates/` 都没有写者，K2.2、K3.1 的镜像是在写路径写出的一版上手改字节（墓碑容器、叶头容器号、根条目）；K3.2 的整池是写路径写的，只是加了开关。checker 打的开关只在 CPU 那一份，GPU 判器没改、没跑（checker 档）。副本测试照 harness 现成的搭建（`build_pool` 经录制的镜像文件建池、再取内存镜像），没有照 3b 从 `SparseBlockDevice` 起；`crates/` 的层 0 枚举一次没调（这一题没有可录的写者）。
- 叶 233 那一档结构变化少（族 k1 叶 233 的 96 段里 TSZu 一格不中，`expected/agg.txt`），K1.4 的复活全在叶 6 的 G-环上；它在 `crates/` 真分配器上多久发生没量。

## 没做什么

- 没跑重型测试：checker 档（`singlefs-checker-tier`）、层 0、GPU 判器、全量 `cargo test`、变异整表都没跑；harness 档只在草稿目录的仓副本上跑了自己的一个测试目标（3 条用例）与 `singlefs-core --lib -- inode_tree`（16 条，四种开关）。
- 不判正推、辩方的格；不替主 agent 采纳；副本与模型上的数都不是入库装置上的数，要引照 `.claude/rules/three-way-inference.md`「一条腿在副本装置上量出来的数，要在入库的装置上重做一次才能引」。
- K2.2 的三个改法、K1.3 的爆炸半径、K3 的「没有后果」都是推的，没实现、没量；N2 那一臂（借时换身份）这一轮没另量（第二轮已量撞号），K3 只攻 N1。
- 跑之前 `ps` 看到另一会话的 `cargo test --offline --no-fail-fast -p singlefs-format -p singlefs-core -p singlefs-harness -p singlefs-checker`（pid 654336）与一条别的会话起的 `opus_r3_leaf_death_images` 测试（pid 509067）在跑；没有性能测量。我的 cargo 一次一条、各用自己的编译目录，没等锁。

## 草稿与副本

- 草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r3-opus-s2/`：`run1/`（开发中的第一次全量与 `trace-64.out`）、`rerun1/`（生成 `expected/` 的那一次）、`rerun2/`（从模型目录在空目录里从头跑通的那一次，`compare.sh` 12 份 same）、`copyrun1.log`、`copyrun2-k2.log`（debug 档跑的两次副本测试，K3 那一族与 release 档逐行相同）、`trial/`（估时试跑）、`clone/`、`copy/`、`model/`（开发副本）。完整模型输出（`sweep-*.out` 合计 10 645 行、`demos.out` 3.7 万行、`trace-64.out`）只留在草稿目录，`expected/sorted-output-sha256.txt` 钉着它们排序后的 sha256。
- 交回前删掉的：编译目录 `target/`、`repo-target/`、`rerun1/target`、`rerun1/repo-target`、`rerun2/target`、`rerun2/repo-target`，仓副本 `repo/`、`rerun1/repo`、`rerun2/repo`、`patchtest/`（删后 `ls` 核过）。
