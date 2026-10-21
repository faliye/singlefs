# m4-c860-r1 攻方腿第二次抽样（Opus）报告（2026-10-20）

立场：正文第四节分工表「云端攻方（Opus）」那一行——找反例：哪一形合法历史让乙（或腿提的别的认法）误红，哪一形坏盘让甲或乙放过。独立抽样，没读第一次抽样的报告、模型与草稿。

## 复跑命令与文件 sha256

模型目录 `research/prompts/m4-c860-r1-opus-s2-model/`。复跑（整条经内存包装；冻结副本 = 开工时主工作区的 `crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo` 拷出来的那棵树）：

```
bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/m4-c860-r1-opus-s2-model/tools/rerun.sh \
  --repo <仓根> --frozen <冻结副本> --work <新的草稿目录>
```

可选环境 `C860_ROCKSDB_LIB_DIR=<放 librocksdb.a 的目录>`：设了就链现成的 RocksDB（本腿就是这样编的，见「这条腿自己的限度」），不设现编。脚本打完各段之后打 `name=rerun_probe_images same_as_recorded=yes` 与每种比重一行 `name=rerun_histories … same_as_recorded=yes`（与 `out/` 里录下的逐行比）。

```
e003ffd5de52f48a66157e63c87ba675759988ef2acd264a209a40edb5d728f3  ./out/c-trial3.log
7012814d8d6f30f0c07c4ca07f30b0990aa59657c35ca0d3a2260877ff1b7943  ./out/h-broadcompaction.log
68d07f71f499ce08876f2f49a835d28fa71a6ae643a86fdcc79bc82ada6436c6  ./out/h-broad.log
0d3b321e40bbb2cc9c042dbd484072a0ae65b214fc9e00aaa2d792c4e6c4a245  ./out/h-byinode.log
6ff39af8ba494162d59f670412e4ae2e510da7b88f85777383628e13589532e2  ./out/h-compaction.log
138aab33310455c2cae0339e36ef19d83a3b1638a5598ec8aec1707718870862  ./out/h-deleting.log
247b7219862f2658689b1aeae9b4f1344e17bd0a21de9d62eb7c1c75e7dc9a9a  ./out/h-orphans.log
f1b141a7c8e48bacc82e9b8fee442e9f3c962fb155de7ef85a997f27b2581732  ./out/h-reuse.log
785725428e7b3caca625f6467dcc9ff479e1d3fc2acca5bbd644cd8e805cefd4  ./out/h-rollback.log
c57c140f377a38233bbb43a5a187594e44d2cf517b05af9a22e2863e4c11921c  ./out/h-snapshot.log
0ea9d9198fd2e28c785a6bf700266a188adb0524590a771ec51f24a144eb8a56  ./out/h-truncation.log
a71977394d753404690cbfe703999a1984be7a13d163e13284142ed320d1ca19  ./out/probe-images.log
134b7193035beb4588b4708bd7797acc16bf7e7403a6ebe046360d76450befe2  ./out/scan-unlink.log
d5ac0e9a6954a3c3bcf5a0e11770517326ad75d5130d6799b20e1010ddd93e05  ./patches/c860-writer-crates.patch
696412c1e941193f33d069f0da2265294a94bcdecdf0303f54492d24446ba38b  ./patches/checker-arms.patch
fe9b32dbdfbf364cda388afc7c036682c953b5521988e11fd69a6b6e24c5338b  ./patches/harness-probe-tests.patch
a582cf9d3a2c2c198b955c1ef1c3635c2507ce8f3b960614719f4f9f9ddc6cdb  ./probe/Cargo.toml
bdd46666e1151ce5626048e49b30da25c86a31e5fa5e909837eb2387fc3de40d  ./probe/src/main.rs
90b9c802a3e20f60316d3d588cbeaf392f388e8a1196fefe8a981abbf4e5781b  ./tools/rerun.sh
```

模型三件：`patches/checker-arms.patch`（池级 checker 里 I-1.13 那一段改成四种认法各比一遍、探针记下，正式判定仍取甲）；`patches/harness-probe-tests.patch`（打在 C860 实现员补丁之上，给 `scan_rebuild_reads_lineage_and_retirement_records.rs` 加两条探针用例：14 张合法镜像、9 张坏镜像）；`probe/`（独立 bin crate：随机历史每一步、与一段写流的每个崩溃状态上四种认法各判什么）。

开工快照核对：`sha256sum -c research/prompts/m4-c860-r1-snapshot/tree-sha256.txt` 四份都 `OK`；`kb-sha256.txt` 一份对不上：

```
.claude/kb/decisions/08-核心索引结构.md: FAILED
```

`git diff` 看到只是文件头加了一行语言链接（`**中文** · [English](08-核心索引结构.en.md) · [日本語](08-核心索引结构.ja.md)`，2 行新增），已定项 16 的正文没变；下文引 D8 的行号按此刻的文件取。

## 各格判定一览

四种认法（K1 定义见下）：甲 = 今天；乙 = 甲且遍历方向有同号同出生代的记录；丙 = 乙且两侧那条记录 nlink 都为 0；丁 = 并集（扫描一侧见「记录 + 墓碑」，或遍历方向那条记录被墓碑压着，任一即算）。

| 格 | 判定 | 依据（量的标「量过」，推的标「推的」） |
|---|---|---|
| K1 | 四种都写清了两侧各去掉什么；有一处四种共用、与认法无关的不对称：扫描一侧凡被墓碑压着的对象的数据单元一律不认领，不看有没有记录 | `rebuild.rs:777` 那一判；推的（读代码） |
| K2 | **没打中**：合法盘上四种认法都没有误红 | 量过：14 张固定合法镜像、10 种比重 1240 段随机历史共 50697 个步后状态（其中 193 个有删除中的对象）、一段回退步的 65548 个崩溃状态，四种认法全部成立；删除写流的崩溃状态没枚举成（每段都超过约 10⁶，见「没打中的形状」） |
| K3 | **打中两形**：①C860 那张（甲、丁放过，乙、丙红）；②一条压着活文件的多余墓碑（甲、乙、丁都把那个活文件整个摘掉，叠上一处只有 I-1.13 该看见的改坏也成立；只有丙红）。两形都有别的不变量兜着 | 量过（副本），见「K3」一节 |
| K4 | M5 原句没写认法；括注「墓碑压着的对象还有记录」照字面同时容得下甲、乙，也把「多余墓碑压着的活文件」算作删除中——丙比括注的字面收严了一步。M5 的必红世界在四种认法下都红，乙、丙不改它的判法 | 文本推的；必红世界量过 |
| K5 | 改法与估行数逐处列了；乙、丙的改动点相同，丙多一处 nlink 判 | 推的（行数），探针实现量过 |

要交用户的岔路（候选、翻面观测、够判条件）在本报告末尾「岔路」一节。

## K1　候选认法的定义

今天的读法在两处：扫描一侧收「删除中」的那一句 `crates/singlefs-checker/src/rebuild.rs:760`「.filter(|object| tombstoned_objects.contains(object))」，收完就从扫描一侧的 inode 记录里去掉（同文件第 755 行起的注释「删除中的对象（谱系上有墓碑、记录还在）：只对它两侧都不比」）；遍历方向一侧 `crates/singlefs-checker/src/walk.rs:14921`「遍历方向那条记录的对象出生代相同时，去掉它的记录」。

| 认法 | 「删除中」集合 S | 扫描一侧去掉 | 遍历方向去掉 | 不去掉 |
|---|---|---|---|---|
| 甲（今天） | 扫描一侧当前版本 inode 叶容器里的记录 (号, 出生代) ∩ 谱系上的墓碑 | S 里每条记录 | S 里遍历方向也有同号同出生代记录的那几个：记录、数据单元集合里它的每一项、那几项的映射 key | 遍历方向没有那条记录的 S 成员：扫描一侧已去掉，遍历方向无可去 |
| 乙 | 甲的 S ∩ 遍历方向有同号同出生代的记录 | 只去掉 S 里的（只在扫描一侧有的那条记录留着照比） | 同甲（遍历方向去掉的集合与甲相同） | 只有扫描一侧有的被墓碑压着的记录 |
| 丙 | 乙的 S，再要求两侧那条记录 nlink 都为 0（D8 已定项 16 意图那一步写的 nlink） | 同乙 | 同乙 | 被墓碑压着而 nlink ≠ 0 的记录（照比） |
| 丁（并集，作对照） | 甲的 S ∪ 遍历方向那条记录 (号, 出生代) 被谱系上的墓碑压着的 | S | S 里遍历方向有的 | 无 |

四种共用、不随认法变的一处：扫描一侧凡 (号, 出生代) 被谱系上的墓碑压着的数据单元都不认领、不进扫描一侧的中央映射（`crates/singlefs-checker/src/rebuild.rs:777`「if tombstoned_objects.contains(&(inode, object_birth)) {」），不看那个对象有没有记录、记录 nlink 多少；inode 叶容器的映射 key 在去掉记录之前就进了扫描一侧的中央映射（同文件第 737 行）。丙在合法盘上与乙等价、在「墓碑压着 nlink ≠ 0 的记录」上与乙分开，靠的正是这一处：丙不摘那个对象，扫描一侧照墓碑判它的数据单元死，遍历方向照读出它们，两张表对不上。

D8 已定项 16 第 1 条 `.claude/kb/decisions/08-核心索引结构.md:484`「意图那次发布把 inode N 的记录写成 nlink = 0」：墓碑与 nlink = 0 同一次发布写下，崩溃截不开它们；丙多要求的 nlink = 0 在合法历史上与「墓碑压着、记录还在」同真（推的，K2 量到的 50697 + 65548 个状态里没有反例：探针记的 `walk_tombstoned_nonzero_nlink` 在合法状态上恒为 0）。

## K2　合法盘上会不会误红（没打中）

### 试过的形状与取样

固定合法镜像 14 张（`out/probe-images.log` 里 `what=legal_` 起头的那 14 个名字，每张四种认法各一行）：挂着回退过同长覆盖写；右片叶删空退役；左片吸收右片；第一个文件删一半、inode 2 带两个单元活着；三单元文件意图之后与逐批（批大小 1）每一批之后；删完之后挂着回退到意图那一版、再逐批续删的每一批之后；再回退到意图之前（墓碑落到谱系外）；234 条记录时 unlink 234 意图那一版（右片叶只装 234 一条 nlink 0 的记录）。原样（每张四行都是 holds，这里只抄两张）：

```
C860_PROBE what=legal_intent_of_the_only_inode_of_the_right_leaf meta=being_deleted=1 tombstoned=1 scan_only=0 walk_tombstoned_nonzero_nlink=0
C860_PROBE what=legal_rolled_back_to_the_intent_after_the_deletion meta=being_deleted=1 tombstoned=1 scan_only=0 walk_tombstoned_nonzero_nlink=0
```

数（命令 `grep -c 'C860_PROBE what=legal' out/probe-images.log` 得 98 行 = 14 × 7；其中带 `arm=… violated` 的 `grep 'C860_PROBE what=legal' out/probe-images.log | grep -c 'arm=.* violated'` 得 0）。

随机历史：10 种比重（`deleting broad reuse rollback orphans snapshot compaction broadcompaction byinode truncation`，即 `GenerationWeights` 里删多片下段叶、各类、抬 F 复用、抬 F 回退、孤儿实例、快照、挂着整理带回退、各类带整理、按 inode 号写、截断那几组），每种种子 5000–5123 共 124 段、每段 40 步，384 槽小盘，每步之后四种认法各判一次。用户动作（删哪个、回退到哪一版、何时挂载、建几个 inode）由生成器按种子放开扫，不写死。合计（`grep -h '^TALLY.* states ' out/h-*.log | awk '{s+=$NF} END{print "states_total="s}'`）：

```
states_total=50697
states_some_deleting=193
```

不是「成立」的认法判定行数（`grep -h '^TALLY.* arm ' out/h-*.log | grep -vc ' holds = '`）得 0；四种认法判法不一或任一违例时记的 `FINDING` 行合计 0。逐比重的原样在 `out/h-<比重>.log` 的 `TALLY` 行，例：

```
TALLY weights=deleting first_seed=5000 seeds=124 ops=40 arm yi some_deleting holds = 115
```

崩溃状态：`deleting` 种子 2000 第 4 步（回退，前面第 1 步是 Unlink）那段写流 21 条写、4 段，`crash_state_count_with_torn_in_place_overwrites` 算 65548 个状态，调 `enumerate_crash_states_versions`（每段都展开）逐个判（`out/c-trial3.log`）：

```
CRASH_STEP seed=2000 step=4 kind=RollBackWhileMounted writes=21 segments=4 states=65548 tally_states=65548 official_jia_evaluated=Some(65548) official_jia_violated=Some(0) official_jia_not_applicable=Some(0) seconds=96.5
CRASH_ARM_CALLS seed=2000 step=4 yi some_deleting holds = 9
CRASH_ARM_CALLS seed=2000 step=4 bing some_deleting holds = 9
```

（四种认法各 65539 + 9 次成立、0 次违例；`CRASH_POINT_REPLAY_FINDING` 那几行是我给的版本表只填了一版、oracle 判的，与 I-1.13 无关。）

### 为什么造不出乙的误红（推的）

乙比甲多红的只有一形：扫描一侧有一条被谱系上墓碑压着的记录、遍历方向没有。扫描一侧选 inode 叶容器的当前版本要过四道（已发布、诞生在谱系上、按退役树不认领、全序键最大），每片当前容器的映射 key 都进扫描一侧的中央映射。合法盘上扫描一侧选中的那片容器若不在遍历方向的 inode 树里，中央映射那张表先差一把码 3 key（`erasing_the_retirement_record_reddens_the_scan_rebuild_after_the_right_leaf_retired` 那一形）；在遍历方向的树里而版本不同，落点对不上也先红。所以「只差一条被墓碑压着的记录」要求中央映射与 inode 树在同一片叶上说法不一，合法盘上这正是 I-9.35（退役完备）与 I-3.9、I-3.11 判违例的样子。推翻这一条的观测：一张合法镜像上 `meta` 行 `scan_only=` 不为 0。

（上面那块 `states_some_deleting=193` 一行出自另一条命令 `grep -h '^TALLY.* states some_deleting' out/h-*.log | awk '{s+=$NF} END{print "states_some_deleting="s}'`。）

## K3　坏盘上会不会放过（打中两形）

9 张坏镜像，都在 `out/probe-images.log` 里，每张一行 `official=`（今天的正式判定，即甲）、四行 `arm=`、一行 `violated_invariants=`（池级 checker 全部判违例的不变量）。

| 坏法 | 甲 | 乙 | 丙 | 丁 | 同一张镜像上别的违例 |
|---|---|---|---|---|---|
| 只抹退役记录（右片删空退役之后） | 红 | 红 | 红 | 红 | I-9.35 |
| C860：抹退役记录 + 补回退役叶的映射条目 | **成立** | 红 | 红 | **成立** | I-3.9、I-3.11、I-9.35 |
| 抹谱系记录（回退过同长覆盖写） | 红 | 红 | 红 | 红 | 无（只有 I-1.13） |
| M5 必红世界：第一个文件删一半时挪 inode 2 一条映射的落点 | 红 | 红 | 红 | 红 | I-2.1、I-3.1、I-4.8、I-7.4、I-3.15 |
| 同一张删一半的镜像上，墓碑容器里多一条压着活的 inode 2（带两个单元）的整对象墓碑 | **成立** | **成立** | 红 | **成立** | I-9.19、I-9.20 |
| 上一行再叠 M5 必红世界那一招（挪 inode 2 一条映射的落点） | **成立** | **成立** | 红 | **成立** | I-2.1、I-3.1、I-4.8、I-7.4、I-9.19、I-9.20、I-3.15 |
| 删一半时墓碑记录的出生代改成 999999 | 成立 | 成立 | 成立 | 成立 | I-9.10、I-9.18、I-9.20、I-9.29、I-9.30 |
| 删一半时墓碑容器的记录全抹 | 成立 | 成立 | 成立 | 成立 | I-9.18 |
| 删完 inode 1 之后多一条压着活的空 inode 2 的墓碑 | 成立 | 成立 | 成立 | 成立 | I-9.19、I-9.20 |

原样（C860 那张与多余墓碑叠挪映射那张）：

```
C860_PROBE what=bad_c860_retirement_erased_and_mapping_put_back official=Holds
C860_PROBE what=bad_c860_retirement_erased_and_mapping_put_back arm=jia holds
C860_PROBE what=bad_c860_retirement_erased_and_mapping_put_back arm=yi violated 扫描重建的三张表与遍历方向从最新根读出的不同：「inode 记录集合」差 1 处，第一处 key inode 234：扫描重建 记录（对象出生代 4、nlink 0、size 0）、遍历方向 没有这一条
C860_PROBE what=bad_c860_retirement_erased_and_mapping_put_back arm=ding holds
C860_PROBE what=bad_c860_retirement_erased_and_mapping_put_back violated_invariants=["I-3.9", "I-3.11", "I-9.35"]
C860_PROBE what=bad_spurious_tombstone_over_live_inode_two_and_its_mapping_moved official=Holds
C860_PROBE what=bad_spurious_tombstone_over_live_inode_two_and_its_mapping_moved arm=yi holds
C860_PROBE what=bad_spurious_tombstone_over_live_inode_two_and_its_mapping_moved meta=being_deleted=2 tombstoned=2 scan_only=0 walk_tombstoned_nonzero_nlink=1
C860_PROBE what=bad_spurious_tombstone_over_live_inode_two_and_its_mapping_moved violated_invariants=["I-2.1", "I-3.1", "I-4.8", "I-7.4", "I-9.19", "I-9.20", "I-3.15"]
```

丙在后一张上的那一行（`out/probe-images.log` 原样，只抄到数据单元表第一处）：`C860_PROBE what=bad_spurious_tombstone_over_live_inode_two_and_its_mapping_moved arm=bing violated 扫描重建的三张表与遍历方向从最新根读出的不同：「中央映射」差 2 处，第一处 key （码 1、出生树 65536、出生 txg 6、实例 1、事务号 6）：扫描重建没有这一条、遍历方向 落点 {盘 0 槽 54284、盘 1 槽 50188}；「每个 inode 的数据单元集合」差 2 处，第一处 key （inode 2、偏移 0）：扫描重建 没有这一条、遍历方向 （码 1、出生树 65536、出生 txg 6、实例 1、事务号 6）`。

### 打中一：C860 那张（甲、丁放过）

历史：`pool_with_new_inodes(…, 233)` 之后 `unlink_to_the_end(234)`（右片叶删空退役）；坏法两处：抹掉那条退役记录、把右片叶最后一版的映射条目补回最新根的中央映射叶（实现员补丁里的构造，原样复现）。每一步许可它的句子：退役记录按退役树认领（I-1.13 定义里 C768 那一支）——退役记录没了，扫描一侧认领右片叶最后一版；它只装一条被谱系上墓碑压着的 (234, nlink 0)，甲按 `rebuild.rs:760` 把它算作删除中、从扫描一侧去掉；遍历方向没有 234，`walk.rs:14921` 那一句什么都不去掉；补回的映射条目让中央映射两侧对上。

四句：①分辨臂：分辨——甲、丁成立，乙、丙红。②判别子看不看得到：看得到——遍历方向读出的 inode 记录集合里有没有 234，正在 I-1.13 自己手里（`walked.inode_records`）。③满足的是哪一个分句：I-1.13 定义里「删除中（墓碑压着的对象还有记录）……只对那一个 (号, 对象出生代) 两侧都不比」那一句（`crates/singlefs-checker/invariants/01-块自描述.md:18`「删除中（墓碑压着的对象还有记录）不再整张报不适用：只对那一个 (号, 对象出生代) 两侧都不比」）；甲把「还有记录」读成「扫描一侧还有」。④改法在这一格上还中不中：乙、丙都修它（量过）；丁不修（量过）；GPU 内核今天与甲同一读法（`crates/singlefs-checker-tier/src/crash_judge_checker_scan_rebuild.wgsl:1347`「if (scan_rebuild_record_is_of_an_object_being_deleted(sw(row + 2u), sw(row + 3u))) {」只看扫描一侧，推的：内核在这张镜像上同样跳过 234）。

兜住它的：I-9.35、I-3.9、I-3.11（量过）。I-1.13 自己的判别力在这一形上不是唯一的一道。

### 打中二：多余的墓碑压着一个活文件（甲、乙、丁都放过，只有丙红）

历史：`pool_with_a_three_unit_file` → 建 inode 2 → 按 inode 号顺序写 inode 2 两个单元 → 删第一个文件：意图与第一批（批大小 1）；这一版 inode 1 删除中、inode 2 活着（nlink 1）。坏法：往最新那一片墓碑容器里追加一条 (2, inode 2 的对象出生代, [0, 2⁶⁴ − 1], 这一版的 txg) 的整对象墓碑（记录数、声明长度、载荷 CRC、头校验和重封，整单元校验和沿引用链补到根槽）；再叠 M5 必红世界那一招：最新根中央映射叶里 inode 2 第一个数据单元那条条目的落点挪 4096 槽。每一步许可它的句子：墓碑判 key 死亡（`crates/singlefs-checker/invariants/01-块自描述.md:18`「墓碑判 key 死亡」）——扫描一侧照 `rebuild.rs:777` 不认领 inode 2 的两个单元；inode 2 的记录被墓碑压着、还有记录，甲、乙都把它算删除中（乙要的「遍历方向也有同号同出生代的记录」这里满足），两侧摘掉 inode 2 的记录、单元与映射 key，挪过的那条映射跟着被摘掉，I-1.13 成立。

四句：①分辨臂：分辨——甲、乙、丁成立，丙红；它分得开乙与丙，分不开甲与乙。②判别子看不看得到：看得到——两侧那条记录的 nlink（1）就在 I-1.13 手里的两份记录里。③满足的分句：同上那一句括注「墓碑压着的对象还有记录」；照字面它把 nlink 1 的活文件也算删除中。I-9.19 的定义（`crates/singlefs-checker/invariants/09-inode树结构.md:31`「inode 树里有号与对象出生代都相同的记录时，那条记录的 nlink = 0」）说这种盘本身坏，I-9.19 红。④改法：乙不修（量过）；丙修（量过）；丁不修（量过）。

兜住它的：多余墓碑由 I-9.19、I-9.20 红；叠上的那处改坏由 I-2.1、I-3.1、I-4.8、I-7.4、I-3.15 红（量过）。所以整张盘没有被池级 checker 放过；被放过的是 I-1.13 这一道。M5 的必红世界（别的文件删除中途改坏一条映射）一旦那个「别的文件」上多压一条墓碑，I-1.13 在甲、乙下都看不见它——这是 M5 括注字面的射程，不是 C860 那一支。

### 没分开四种认法的几形

墓碑本身坏（出生代改坏、整容器抹掉）：扫描一侧不再认 inode 1 删除中，两侧照比、对得上，四种都成立，由 I-9.18（`crates/singlefs-checker/invariants/09-inode树结构.md:30`「墓碑树（D18（块里携带什么信息） 已定项 24）里都有一条在谱系上的墓碑记录」）等兜住。墓碑压着一个没有数据单元的活空 inode：四种都成立（丙照比那条记录，两侧逐字节相同），由 I-9.19 兜住。谱系记录与退役记录单独丢失：四种都红在中央映射或数据单元表。

## K4　判决 M5 的文意与 I-1.13 的定义

- M5 原句（`research/prompts/m3-multi-file-unlink-r1-main-verification.md:41`「两侧都去掉它的映射、单元与记录，别的对象照比」）只写了去掉什么，没写由哪一侧认。I-1.13 那一行的同一句（`crates/singlefs-checker/invariants/01-块自描述.md:18`「删除中（墓碑压着的对象还有记录）不再整张报不适用：只对那一个 (号, 对象出生代) 两侧都不比」）的括注「还有记录」没说哪一侧的记录。两种读法照字面都过：甲读「扫描一侧还有」，乙读「两侧都还有」。文本不蕴含其中一种（推的）。
- 括注照字面把「墓碑压着、nlink ≠ 0 的活记录」也算删除中；D8 已定项 16 的「删除中」是意图那一步写下 nlink = 0 的那一态。丙把括注收窄到 D8 那一态：多要求两侧 nlink = 0，原判据下被判成立的镜像，丙判成立或红，不会把原来红的判成立（收严，推的；9 张坏镜像上丙红的集合包含甲、乙红的集合，量过）。丙的收严是本腿提的，只在我的模型上量过、被攻过零轮。
- 乙、丙会不会改 M5 必红世界（`research/prompts/m3-multi-file-unlink-r1-main-verification.md:41`「必红世界：别的文件删除中途改坏一条映射，I-1.13 红」）的判法：不改，四种认法都红在中央映射、差 1 处（量过，`what=bad_m5_must_red_world_inode_two_mapping_moved`）。会改的是那个「别的文件」被多压一条墓碑时：甲、乙下 I-1.13 看不见那一处改坏，丙看得见（K3 打中二）。
- 今天 I-1.13 定义里「墓碑判 key 死亡」只作用在扫描一侧；遍历方向不读墓碑。所以「由遍历方向自己读墓碑认」那一种（正文 K1 举的例）在 C860 上与甲同结局：扫描一侧照样认 234 删除中并去掉，遍历方向没有 234、无可去，两侧相同（推的，与丁同理：丁就是「两侧各自认、取并集」，量过成立）。

## K5　改法与代价（行数是推的，探针实现量过而它不是产品改法）

| 处 | 乙 | 丙（在乙之上） |
|---|---|---|
| `crates/singlefs-checker/src/rebuild.rs` | 删掉收完 `objects_being_deleted` 之后从 `inode_records` 去掉记录的那三行循环，文档注释改「两侧都有才摘」；约 −3 / +2 行 | 无（nlink 判放在 walk.rs 一处，两侧记录都在手里） |
| `crates/singlefs-checker/src/walk.rs` 比表之前那一段 | `let rebuilt` 改可变；认出的集合 = 扫描一侧集合 ∩ 遍历方向同号同出生代；对这些号同时从 `rebuilt.inode_records` 与遍历方向三张表去掉；约 +4 行 | 过滤再加「两侧那条记录偏移 36 的 nlink 都为 0」；约 +4 行 |
| GPU 内核 `crash_judge_checker_scan_rebuild.wgsl` | 比记录那一处（第 1347 行那一判）只在遍历方向找得到同号同出生代那一行时才跳过；第 1279 行起那一判已经要遍历方向有行，不动；约 +6 行 | 两处都加 nlink = 0 两侧都判；约 +6 行 |
| 不变量清单 I-1.13 那一行 | 括注改写「两侧都有同号同出生代的记录」；1 句 | 再加「且两侧那条记录 nlink = 0（D8 已定项 16 意图那一步）」；1 句 |
| 用例 | 去掉 C860 用例的 `#[ignore]`；`erasing_the_retirement_record_reddens_the_scan_rebuild_after_the_right_leaf_retired` 里那句「不出现 inode 记录集合」（主工作区此刻第 244 行，C860 补丁打上之后第 336 行）改成「inode 记录集合差 1 处、inode 234」；约 ±6 行 | 加一条必红世界：删一半镜像上多一条压着活 inode 2 的墓碑，丙下 I-1.13 红在数据单元表（今天成立）；约 +40 行，可直接用本腿 `harness-probe-tests.patch` 里的追加墓碑那个函数 |
| `crates/mutations.tsv` | 第 4958 行那一条（「认删除中的对象那一判恒不命中」）的判别力随乙变了：乙下 234 的记录无论那一判命中与否都在扫描一侧，这一行要改写或换目标；新加一行「认删除中改回只凭扫描一侧」，C860 用例证红；GPU 那四行（第 4885–4888 行一组）看要不要加一行同形的 | 再加一行「去掉 nlink 判」，新必红世界证红 |

代价：乙改 4 处产品代码 + 2 处用例 + 1–2 行变异，约 20 行；丙在乙之上约 +50 行（多半是新用例）。都不加盘上字段，判一次的工作量不变（推的）。

## 岔路

| 候选 | 翻面观测 | 够判条件 |
|---|---|---|
| 甲（今天：扫描一侧见「记录 + 谱系上的墓碑」即算删除中） | 一张合法镜像（含崩溃状态）上乙误红：探针 `meta` 行 `scan_only=` 不为 0 而全盘别的不变量都成立 | 删除写流的崩溃状态（意图、每一批、最后一批前后、删空退役、左片吸收右片）在乙下逐个判过 |
| 乙（甲且遍历方向有同号同出生代的记录） | 同上的反面：那批崩溃状态上乙零误红，则乙在 C860 一形上严格强于甲；若要 I-1.13 自己看得见「多余墓碑压着活文件」，乙不够 | 同上；加上主 agent 定 I-1.13 在多余墓碑一形上要不要有独立判别力 |
| 丙（乙且两侧 nlink 都为 0） | 一张合法镜像上「谱系上的墓碑压着 nlink ≠ 0 的同号同出生代记录」（探针 `walk_tombstoned_nonzero_nlink=` 不为 0）而盘是好的 | 同一批崩溃状态在丙下零误红；丙被攻过零轮，要再攻一轮 |

## 没打中的形状

- 合法盘上让乙、丙误红：14 张固定镜像、10 种比重 × 124 段 × 40 步（50697 个步后状态，193 个有删除中的对象）、一段回退步 65548 个崩溃状态，四种认法零误红、零不适用。试过的合法形：删一半（意图之后、每一批之后、最后一批之后）、删完之后回退到意图那一版再续删、回退到意图之前、删空退役、左片吸收右片、多文件（第一个文件删一半而 inode 2 带单元活着）、随机历史里的快照、整理、截断、按号写、抬 F 复用与回退。
- 删除写流本身的崩溃状态（意图那次、每一批、最后一批前后）**没有枚举成**：每一段删除写流里第一次发布那一段就有 20–28 条写，整写子集 2²⁰ 起跳，一段历史就超过约 10⁶，按定义第 3b 步那一段不跑。数（`fixed` 模式只算状态数、不枚举）：

```
FIXED scenario=right_leaf_retired writes=58 segments=8 segment_sizes=[26, 2, 1, 2, 22, 2, 1, 2] states=71303191
FIXED scenario=left_absorbs_right writes=60 segments=8 segment_sizes=[26, 2, 1, 2, 24, 2, 1, 2] states=83886103
FIXED scenario=right_leaf_intent_only writes=31 segments=4 segment_sizes=[26, 2, 1, 2] states=67108876
FIXED scenario=three_unit_batches_of_one writes=116 segments=16 segment_sizes=[26, 2, 1, 2, 24, 2, 1, 2, 22, 2, 1, 2, 24, 2, 1, 2] states=104857645
FIXED scenario=rolled_back_to_the_intent_then_continued writes=122 segments=16 segment_sizes=[20, 2, 1, 2, 28, 2, 1, 2, 26, 2, 1, 2, 28, 2, 1, 2] states=605028397
```

  随机历史里 54 个 Unlink 步也一样（`grep -c '^SCAN' out/scan-unlink.log` 得 54；5 种比重各 12 段、每段 30 步）（`out/scan-unlink.log`，最小一段 4259863 个状态）。所以 K2「删一半崩溃」那几形我只覆盖了发布边界上的镜像（固定镜像与步后状态），没覆盖发布中途的崩溃镜像；这是 K2「没打中」最大的缺口。
- 挂载续做的崩溃镜像（交回镜像的观察者）没跑。

## 这条腿自己的限度

- 副本上的数，不是入库装置上的数。副本 = 主工作区 `crates/` + C860 实现员补丁 + 本腿两份补丁 + 探针 crate。
- 编译：checker 档包默认带 RocksDB，副本里现编 librocksdb-sys 在前台 225 秒内编不完（每次被截断都从头编），改为把主工作区 `target/release/build/librocksdb-sys-f7ce5abb7da4f505/out/librocksdb.a` 拷进草稿目录、经 `ROCKSDB_LIB_DIR` + `ROCKSDB_STATIC=1` 链它。同一版本、同一构建脚本哈希目录名；没核两份编译参数逐项相同。
- 固定镜像用 `tests/common` 的搭法（`FileBackedBlockDevice` 建在 `std::env::temp_dir()` 下），不是定义第 3b 步要的内存稀疏盘；我把 `TMPDIR` 指到 `/dev/shm` 下自己的目录（内存里的稀疏文件），跑完那个目录里 0 个文件（`ls … | wc -l` 得 0：用例结束时自己删）。随机历史与崩溃枚举用的是 `MemoryPool` / `SparseDevice`。
- 崩溃枚举调的是 `enumerate_crash_states_versions`（每段都展开）；我只给了一版版本表，所以 oracle 那几行发现与 I-1.13 无关；四种认法的数来自池级 checker 里的全进程计数（每个状态调一次池级 checker，调用次数 = 状态数 65548，量过）。运行时设了 `CARGO`（库要记工具链）、`SINGLEFS_CRASH_AMPLIFICATION_HOME`（库放草稿目录）、`SINGLEFS_CRASH_LIBRARY_WRITE_ONLY=1`（不复用旧结论）。
- 状态量：这条腿崩溃原型合计跑了 65548 个状态（远低于约 10⁷）。
- 随机历史批：10 批（每种比重一批 124 段），估时按先跑的 4 段 × 30 步用了约 4 秒；10 批实跑合计 2 分 53 秒。没缩历史条数与长度。
- 丙是我提的收严，只在我的模型上量过、被攻过零轮。
- 「几个改法各修哪一格」：C860 一格——乙修（量过）、丙修（量过）、丁不修（量过）；多余墓碑一格——乙不修（量过）、丙修（量过）、丁不修（量过）；GPU 内核在两格上的判法：推的（没在卡上跑）。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；副本上的数不算入库装置上的数。
- 重型测试一样都没跑；没跑崩溃、变异两条轻型；没跑 GPU 对拍。
- 没在乙、丙下跑仓里已有的那几条 I-1.13 用例全集（只跑了 `scan_rebuild_reads_lineage_and_retirement_records` 里两条探针用例；它们复现了已有三条合法用例与两条必红世界的镜像）。
- 没造「遍历方向从 inode 树内部节点摘掉一片只装删除中对象的叶」那一形（比 C860 少一处退役、多一处改码 2 节点）；推的：它与 C860 同类，甲放过、乙丙红，I-9.5 与 I-9.35 兜。

## 要主 agent 跑的命令

- 复跑：本报告开头那一条。预计挂钟：现编 RocksDB 时编译占大头；跑三段约 5 分钟（量过的是 1.1 秒 + 2 分 53 秒 + 1 分 37 秒）。
- 补 K2 缺口（重活，要后台跑）：删除写流的崩溃状态在乙、丙下逐个判，用 `probe/src/main.rs` 的 `fixed` 模式（参数：场景名、状态上限、可选分片 `<i>/<n>`），例：`CARGO=$(command -v cargo) SINGLEFS_CRASH_AMPLIFICATION_HOME=<草稿>/crash-home SINGLEFS_CRASH_LIBRARY_WRITE_ONLY=1 TMPDIR=<内存目录> bash scripts/hosts/capped.sh 31 bash scripts/hosts/run-with-memory-cap.sh 8G <树>/target/release/c860-probe fixed right_leaf_intent_only 70000000`；按量过的约 36 毫秒每状态每线程，6.7 × 10⁷ 个状态 31 线程约 22 小时，要么分片多机，要么先定要不要为这一格放量（超过约 10⁷ 的上限，要主 agent 定）。看 `FIXED_ARM_CALLS … yi … violated`、`… bing … violated` 两种行有没有。
