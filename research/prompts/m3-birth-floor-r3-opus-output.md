# m3-birth-floor-r3 攻方腿报告（2026-10-02，终轮）

<!-- doc-lint:not-numbers TF K1 K2 K3 K4 K5 CS1 CS2 J1 J3 BI BJ R3B R3G R3S R3D V1 V2s V3s -->

批次：records/2026-09-30-里程碑三开工第一轮.md#5。攻击面：正文 `research/prompts/_m3-birth-floor-r3-body.md` 第四节分工表攻方那一行（K1、K2，兼攻 K3）。判据照正文第五节。

## 复跑与文件

复跑：`bash research/prompts/m3-birth-floor-r3-opus-model/rerun.sh /tmp/claude-1000/m3-batch5-freeze/tree <草稿目录> [段名…]`。它把冻结副本拷进草稿目录、`patch -p2` 打上 `proto.diff`（第二轮攻方原型加这一轮的改动，对冻结副本整份取差），经内存包装（2G）、线程上限 7 编 release，按段名跑原型包 `bf-proto` 的用例，每段一份日志、末尾打汇总行。段名不给时跑全部（各段在我这里单跑的 `seconds=` 加起来约 40 分钟，推的）。`proto.diff` 打在一份新拷的冻结副本上，与我的副本逐文件相同（`diff -rq --exclude=target --exclude=Cargo.lock` 无输出，退 0）；`rerun.sh` 整条没在新副本上跑过，每段的命令与环境变量是我在副本上跑出 `logs/` 的那几条。不跑 checker 档包的任何测试：`bf-proto` 只把 `singlefs-checker-tier` 当库链进来，调层 0 的枚举；测试目标名里没有 layer0。

```text
c2c960eab36748ab93d661950783c835956fbea9b6487059f9915c93bd9fd7ac  ./logs/bad-gap2.log
b4dafc3ec6de7d08815b6814d2b7ab3a6c980625a810c1fff2d1b87ab5e351b3  ./logs/bad-gap-extra2.log
479d67bca0277ae143755794571313dfbe5f18bba507b6266d6d863c6f3764db  ./logs/bad-r2-rerun.log
74671161134920c50a747c5ca2be9a1ce4571fb1130b0bc43cebbfd07c47887f  ./logs/count-A-384.log
9824adeb9040eb3868b76bf731014dbb34d2636233f429f1910c11a33f3cc8b1  ./logs/count-A.log
f7b11b757d3ada1d17b837a8b8250f4552594804e8e33502f6eddfa32d058eea  ./logs/crash-head.log
a59cb34a511b8c642cdc1adc844720eff2a3f8251806176a38923850071dcf57  ./logs/enum-A-raise1-0.log
0962f0257879ca62b9ab99f7f0182417a087972e9f73cb96207d17367d61959f  ./logs/enum-A-raise1-7000.log
6dfd1796cc035b75b24ae270401584084a0e08d6ef982a5d843749b148ef1fde  ./logs/enum-A-rb1.log
cbc9f10ca99692ca1614e1daf15d1e615dc04ffc82fd0e18a20d92fb71201705  ./logs/enum-A-remount-0.log
4d06bb927c6b8a0797c098e44589cff1e2b625066618ccb975d70eec58eb75c0  ./logs/enum-A-remount-11000.log
595f7991f71c8af92d00de73f355a7461e5af0eb038d48a6471635f12aff0ece  ./logs/enum-A-remount-18000.log
b1331b0e570166c8d96b16d6ca379d0a7b00e4b9372f5ebbf356f5b76eb69684  ./logs/enum-A-remount-4000.log
152d3382e702a0a973ff695964fd9908b93b7536e920e5aa88b49c50151d2b73  ./logs/jwrap1-a1.log
300fe2c32ddcd3cd64ce7b54926d2872cd4229ab39e9e07a7582a884ff5f331c  ./logs/jwrap1-a2.log
a70a5e211e8b290406c35805c300797961f047f92e792cc4c134381850c8f79a  ./logs/jwrap6-a2.log
36db682c72dbed3b4a3f6be7a4c506c9e1cae3b26114e8e25dca586bd7f0e840  ./logs/r2-sonnet-deep-ROLL8.out
457e4d70fc7d5db7523077537977411b6b0bc37aba246660fa4bbf0725e8314b  ./logs/r2-sonnet-deep-ROLLKX8.out
1ec909eb8bf80ade5a1fea62257a966007d12ae4971c0ced25547a5dfe74437d  ./logs/r3d-len2.log
07afc07b08b7b8ec1c387b4728e79dd5c5df7cfd2df0ecb52c5ca466b9244152  ./logs/r3d-len3-0.log
10903dc3aa5a78c3ff68898da891e2ca242696fb6be39df0ed25a45534d0d015  ./logs/r3d-len3-1.log
9afce446d3ded88445a55d86716c2b1c9babbf07817fdd230f5635788e56c2ee  ./logs/r3d-len3-2.log
2d34e96d3838b43517fed9c57a93eafe877bb6195e9c1715dcb3424eda7229e6  ./logs/r3s-384.log
5bb5eb0ea8646daee0afac13db7549fc17d2ebb62ce6f307ccf7b02fee32c652  ./logs/r3s-verbose.log
9107bed236d9b703af76dbe03dea2aeb0b3549e65c90dc0da380bd1a32eb86f6  ./logs/sweep-A-nofault.log
ef40eb8f3488b8e9745daf18bba2073ebf07362b72444a299f93af8068570302  ./logs/sweep-B-nofault.log
ad976aca694dc60c331a7df4333095312711b8e0398d5e4560a7af03236cc5be  ./logs/sweep-C-nofault.log
440713c7a83e74a651e00b499af7c16c2f4c902af5c0ee84e6a386e66a881f48  ./logs/sweep-raise-A-nofault.log
d6794951aebe8ed1e1778f8bdc1e4d725497b7989571c104e77744135caa0fa2  ./logs/torn-sw2-384.log
884413d6a9d3bf8b8752f3d3c8e2ef33467e935e783ff2067639ede72963d6c6  ./proto.diff
8deafe3f6b39bceeb2cac1f396474d6787b3641431f7ab584556fa168b5ceafe  ./rerun.sh
```

全部数都是冻结副本拷贝上的数，不算入库装置上的数。冻结副本的 `crates/singlefs-{checker,core,harness,checker-tier,format}` 与主仓工作区逐文件相同（`diff -rq` 五个目录都无输出），所以「入库的池级 checker」就是副本里的 `check_pool_image`。

装置（在第二轮攻方原型上加的，都只在副本里）：
- `proto_b.rs`：`judge_tree_floor_on` 加缺口对的计数（`CS2:gap-pairs`、`gap-pairs-with-new`、`gap-first-placements`）、`CS2:rows`（最新根实例表的行），加我提的收严 **gap-row**（`tfgaprow-lenient` / `betagaprow-lenient`，零轮，见「改法」）；`BF_NO_CS1` 时不算 CS1（只为省时，CS1 不是措辞取的候选集）。
- `crash.rs`：加 `layer0_state_images_range`，与 `judge_layer0_state_range` 用同一份 `Layer0StatePlan`、同一份撕裂第三态，只把崩溃镜像交出来、不跑层 0 判定（调用方自己挂载、跑池级 checker）。
- `bf-proto/src/lib.rs`：`BF_WIDTH=384` 取 harness 单元区 384 槽的小盘档（6 MiB 环）；`with_pool`（拿一张崩溃镜像另起一份，没有会话）；`materialize`（崩溃状态落成内存池）；两个带崩溃的步：`SequentialWriteCrashInRecords`（顺序写那次发布崩在记录那一段，只留 `keep_mask` 选中的记录写）、`RemountCrashBeforeRoot`（可写挂载那一串在第 n 次根槽写之前崩）。
- 新用例：`tests/bf_r3_gap.rs`（层 0 枚举 + 挂载、带崩溃历史扫描、盘与根环区域故障）、`tests/bf_r3_bad.rs`（记录撕裂 16 个子集、缺口右边的坏单元）；`tests/bf_r2_bad.rs` 的 `report` 另报池级 checker 成立 / 不适用的条数。

## 各格判定一览

| 格 | 被攻的东西 | 判定 | 依据（副本上量的） |
|---|---|---|---|
| K1-a 缺口在合法历史里多常出现 | gap | 量出：不带崩溃的历史上 0 次；「写行那次发布的记录落了、根没落」或「两条以上记录的发布只落了一部分记录」之后再可写挂载，每次都留一个缺口 | 第二轮 2 712 段终态与我的 604 段终态 0 个缺口对；层 0 挂载之后 41 056 个状态里 3 个；人造 16 个记录子集里 6 个；1 332 段带崩溃历史里 894 段 |
| K1-b 缺口右边第一次出现的坏单元 | TF + β + gap | **打中：造出 10 张池级 checker 0 红、TF + β + gap 不红、去掉 gap 时 TF 红的坏镜像** | `logs/bad-gap2.log`：缺口右边那条根（新实例写行那次发布）新写的 5 个单元，诞生代号各改小到 R⁻.txg 与 R⁻.txg − 1 |
| K1-c 池级 checker 别的条兜不兜得住 | 池级 checker | 兜不住：10 张都 0 红，不适用的几条与干净镜像相同 | 同上，再接两次覆盖写照旧（`logs/bad-gap-extra2.log`） |
| K2 gap 之后合法镜像上红 | TF + β + gap | 没打中（一次抽样） | 层 0 41 056 个状态 × 四张（崩溃镜像、挂载之后、再接两种动作各一张）、1 332 段带崩溃历史、604 段 × 三个根环区域 / 最新根槽 / 换盘、J1-a 3 456 张与 700 个状态、2 712 段：池级 checker 0 红的镜像上 TF + β + gap 一次没红 |
| K3 第二轮的数 | 696 / 700、472、2 712、2 892 万 | 四个都在副本上重做出来，逐字段相同 | `logs/crash-head.log`、三份 jwrap、四份 sweep、两份辩方深枚举 |
| K3 J3-c、J3-d 在入库池级 checker 上 | 「严格小于」、β 再次出现的只红它 | 仍只红它：池级 checker 65 项里 0 项违例，不适用的四项与干净镜像相同 | `logs/bad-r2-rerun.log` |

「没打中」都是这一条腿一次抽样的结果；取样范围在「没打中的形状」一节。

## K1　gap 的盲区

### K1-a　缺口在合法历史里从哪来、多常出现

**机理（读代码）。** 新实例第一条根的 txg 不是「所选那一版 + 1」，而是根环与 journal 里见过的最大 txg + 1：「/// 新实例的第一次发布的 checkpoint_txg = max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1」（`crates/singlefs-core/src/mount.rs:1143`）。所以只要盘上留着一条自证过、却没被恢复施加的记录，它的 txg 就被下一个实例跳过去，候选集里留一个缺口。checker 只把「带末条标记、实例表行说施加过」的记录补成一版：「let applied_by_a_recovery = record.commit_marker == CommitMarker::Present」（`crates/singlefs-checker/src/walk.rs:5830`），「&& record.checkpoint_txg <= row.published_checkpoint_txg」（`crates/singlefs-checker/src/walk.rs:5838`），被跳过的那个 txg 补不回来。

**量出来的两种来源（副本上）：**

1. 记录撕裂：一次发布写两条以上记录（顺序写 2 单元就是两条，每条两盘各一份，记录那一段 4 次写），崩在记录那一段里、只落了一部分，之后可写挂载。记录那一段的 16 个子集逐个当一段历史（`tests/bf_r3_bad.rs` 的 `torn_records_then_remount`，384 槽小盘）：6 个子集（只落第一条的三种、只落第二条的三种）挂载之后有缺口，其余 10 个没有（全落或一盘完整时恢复照施加）。原样（`logs/torn-sw2-384.log` 第 105、107 行）：

```text
R3B sw2 keep=0011 remount pool_red=[] pool_na=["I-7.9", "I-8.8", "I-4.1"] tf=0/86 tf+gap=0/72 beta=0/86 beta+gap=0/72 gap_pairs=1 gap_first_placements=14
R3B sw2 keep=0011 remount gap=R⁻ txg 5 [实例 1] → R txg 7 [实例 2] | with_new=R⁻ txg 5 [实例 1] → R txg 7 [实例 2] 新落点 14 个、第一次出现 14 个
```

2. 写行那次发布崩在根槽之前：可写挂载取号之后的写行发布，记录落了、根没落，再挂载时这一版不被施加（实例 2 的行记成 (2, 0, 0)），实例 3 从 txg 9 起，txg 8 成了缺口；暖机那次（第二次）崩在根槽之前则照常由记录施加、没有缺口。原样（`logs/r3s-verbose.log` 第 18、20 行）：

```text
R3V final A/mcrash0,remount pool_red=[] versions=txg 0 [实例 0] | txg 1 [实例 1] | txg 2 [实例 1] | txg 3 [实例 1] | txg 4 [实例 1] | txg 5 [实例 1] | txg 6 [实例 1] | txg 7 [实例 1] | txg 9 [实例 3] | txg 10 [实例 3] rows=(1, 7, 0) (2, 0, 0) gap=R⁻ txg 7 [实例 1] → R txg 9 [实例 3] applied-without-base=0
R3V final A/mcrash1,remount pool_red=[] versions=txg 0 [实例 0] | txg 1 [实例 1] | txg 2 [实例 1] | txg 3 [实例 1] | txg 4 [实例 1] | txg 5 [实例 1] | txg 6 [实例 1] | txg 7 [实例 1] | txg 8 [实例 2] | txg 9 [实例 2·由记录施加] | txg 10 [实例 3] | txg 11 [实例 3] rows=(1, 7, 0) (2, 9, 0) gap= applied-without-base=0
```

**频率：**

| 取样 | 镜像 / 状态数 | 有缺口对（且缺口右边有新落点）的 |
|---|---|---|
| 不带崩溃的历史：第二轮 2 712 段终态（三种前缀 × 回退目标 × 长度 0–2 的用户动作，在我的副本上重跑） | 2 712 张 | 0（四份 `sweep-*-nofault.log` 里 `betagap-lenient` 的 evaluated 与 `beta-lenient` 逐份相等：151 426、156 966、193 212、90 078） |
| 不带崩溃的历史：前缀 A / R × 长度 0–3 的用户动作（`device_and_region_faults` 的 `none` 那一张） | 604 张 | 0（四份 `r3d-*.log` 都没有 `none:gap` 开头的行） |
| 层 0 枚举（前缀 A、384 槽小盘，每一段都展开、段内整写子集逐个、原地覆写带撕裂第三态），每个崩溃状态可写挂载之后 | 回退 4 108、抬 F 12 330、可写挂载 24 618，合计 41 056 个状态 | 3（都在可写挂载那条流里：写行那次的记录那一段两个子集，加根槽那一段的空子集，即上面第 2 种），`logs/enum-A-remount-11000.log` 第 59 行 |
| 顺序写 2 单元那次发布记录那一段的 16 个子集（人造，见上） | 16 | 6 |
| 带崩溃的历史扫描（前缀 A / R × 崩溃 6 种 × 挂载之后长度 0–2 的动作，字母表里也带崩溃 + 挂载） | 1 332 段终态 | 894（`logs/r3s-384.log` 第 26 行） |
| 第二轮 J1-a 短环（重跑） | 3 456 张 | CS2 上 gap 跳过的落点 64 072、85 232、7 656 个（三份 jwrap 日志） |

按层 0 状态数算，可写挂载那条流里 3 / 24 618；顺序写 2 单元那条流（前缀 A、384 槽小盘）16 777 240 个状态（`logs/count-A-384.log`）超过约 10⁶，照定义没跑，按上面 16 个子集里 6 个推，占全流不到百万分之一（算术；16 个子集是在另一个前缀「覆盖写两次」上造的，推的）。按「崩在哪一刻」算：写行那次发布从第一条记录落盘到根槽落盘之间崩、或多记录发布在记录之间崩，之后的每一次可写挂载都留一个缺口，缺口右边那条根就是新实例的写行根，它新写的单元（384 槽 14 个落点、4 GiB 18 个，两盘各一份）都在盲区里。「写行那次发布不碰 inode 树」（`crates/singlefs-core/src/mount.rs:2799`），所以这一形的盲区只罩固定点单元；罩到用户数据单元的只有 J1-a（缺口右边是覆盖写或顺序写的根），它要短环。

### K1-b　缺口右边的坏单元（打中）

**历史**（`tests/bf_r3_bad.rs` 的 `bad_unit_right_of_a_gap`，4 GiB 默认几何）：第一个文件（txg 3）、覆盖写两次（4、5）、顺序写 2 单元那次发布崩在记录那一段、只落了第一条记录的两份（`keep=0011`，txg 6 只有一条记录）、可写挂载（实例 2 写行 txg 7、暖机 txg 8）。这张镜像池级 checker 0 红、TF 0 红，候选集 `txg 0…5 [实例 1] | txg 7, 8 [实例 2]`，缺口对 (5, 7)，实例表行 (1, 5, 0)。然后把 txg 7 那次发布新写的单元逐个改：诞生代号 7 → 5（= R⁻.txg）与 7 → 4，指它的指针、中央映射 key、两盘的分配记录分配代、journal 点名项一起改，校验和沿引用链补到根槽与记录（第二轮 `bf_r2_bad.rs` 的改造法，journal 计数器改成跳着找、反向链只重接改之前接得上的那几对）。

一步一步许可它的那几句：
1. txg 6 的记录自证过、留在 journal 里，所以实例 2 的第一条根是 7 不是 6（上面 `mount.rs:1143` 那一句）。
2. txg 6 没带末条标记、实例 1 的行写的是 T = 5，checker 不把 6 补进候选集（上面 `walk.rs:5830`、`walk.rs:5838`）；所以候选集里 5 与 7 相邻，gap 让这一对整对报不适用。
3. txg 7 那次发布新写的单元只被 7（及之后不重写它的根）引用，第一次出现就在 (5, 7) 这一对上；之后每一对里 R⁻ 都引用它，不再判。

原样（`logs/bad-gap2.log` 第 37、162、165 行；第 162 行与第 161 行是同一张）：

```text
R3B clean pool_red=[] pool_na=["I-7.9", "I-8.8", "I-4.1"] tf=0/106 tf+gap=0/88 beta=0/106 beta+gap=0/88 gap_pairs=1 gap_first_placements=18
R3B bad slot=50312 class=2 key_width=27 7->4 counts=[1, 0, 2, 1] pool_red=[] pool_na=["I-7.9", "I-8.8", "I-4.1"] tf=2/106 tf+gap=0/88 beta=2/106 beta+gap=0/88 gap_pairs=1 gap_first_placements=18
R3B bad slot=50312 class=2 key_width=27 7->4 counts=[1, 0, 2, 1] first[CS2:tf-lenient]=TF-lenient：盘 0 槽 50312 类 2 诞生代号 4 ≤ R⁻ txg 5 [实例 1]；R txg 7 [实例 2]；第一次出现；在继承树（不论压不压得住）下 false；R 的树：种类 1 根槽 50265 出生 5；种类 2 根槽 50268 出生 5
```

| txg 7 新写的单元 | 改到 5 / 改到 4 | 池级 checker | TF（无 gap） | TF + gap | β + gap |
|---|---|---|---|---|---|
| 槽 50304，类 3 | 两张 | 0 红 | 2 | 0 | 0 |
| 槽 50306，类 2、key 宽 10（分配记录树） | 两张 | 0 红 | 2 | 0 | 0 |
| 槽 50307–50310，类 2、key 宽 10 | 各两张 | I-1.1（改造法没造干净，同第二轮，不算） | 2 | 0 | 0 |
| 槽 50311，类 2、key 宽 22 | 两张 | 0 红 | 2 | 0 | 0 |
| 槽 50312，类 2、key 宽 27（中央映射） | 两张 | 0 红 | 2 | 0 | 0 |
| 槽 50313，类 2、key 宽 8 | 两张 | 0 红 | 2 | 0 | 0 |

池级 checker 0 红的 10 张，不适用的都是 I-7.9、I-8.8、I-4.1 三项，与干净那张相同：池级 checker 别的条一条也没兜住。这 10 张之上再接两次覆盖写（txg 9、10），结果照旧（`logs/bad-gap-extra2.log`：TF 2/158、TF + gap 0/140、池级 checker 0 红），坏单元一直挂在 txg 7 这条候选根下、永远不被判。

#### K1-b 打中之后的四句

照 `.claude/singlefs-ai-sop/rules/evidence-discipline.md@判据自己也会写错：打中之后先判是哪一种`「打中之后先问四句」：

1. **分不分辨臂：分。** 同一张上 TF + β 不带 gap 红（`tf=2/106`），带 gap 不红（`tf+gap=0/88`、`beta+gap=0/88`）；我提的 gap-row（下一节）红（`tf+gaprow=2/106`，`logs/bad-gap2.log` 第 161 行）。辩方提的两种处置：丙（等谱系记录）——谱系记录只在回退那次发布写，这一形里没有回退，推的：照样不判；「洞两侧整对不适用只在根槽读不出处触发」的窄形——这里没有一个根槽读不出（txg 6 从没写过根），推的：会判这一对、会红，但它在 J1-a 上（txg 9、10 的槽是被盖掉、读得出、坐着别的 txg）怎么判取决于「读不出处」怎么认，没量。
2. **系统看得到判别它的东西吗：看得到。** 最新根的实例表里有一行 (1, 5, 0)（日志里 `rows=(1, 5, 0)`），行的 T 是「/// 这一行记的 T：该实例被施加到哪一代为止（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2 的 T_pub）。」（`crates/singlefs-checker/src/walk.rs:372`），而 R（txg 7）是实例 2 的根。「实例 2 接在实例 1 的 txg 5 那一版上」写在盘上；缺口里的 txg 6 只是被撕裂发布用掉的号。中间若隔着没发布过的实例，它们的行是 (j, 0, 0)：「/// 新实例写行那一行的来历：所选根那个实例那一行；中间实例的行由 establish_instance 补 (i, 0, 0)。」（`crates/singlefs-core/src/mount.rs:1105`）。J1-a 的缺口（实例 1 内部 8 → 10 / 11）没有哪一行能说明，看不到。
3. **满足的是判据字面的哪一句：** 「- gap 下造得出一份坏镜像：池级 checker 0 红、TF + β + gap 不红、而去掉 gap 时 TF 红 ⇒ gap 的盲区坐实，交用户定收不收（写明那一形出现的频率）。」（`research/prompts/_m3-birth-floor-r3-body.md:52`）。三个分句逐个对上：池级 checker 0 红（10 张）、TF + β + gap 不红、去掉 gap 时 TF 红。频率见 K1-a。
4. **跑前条款给的改法在打中的格上还中不中：** 条款只给「交用户定收不收」两条路；各改法逐格见下一节的表。「不收」（保留 gap）在这 10 张上照样不红；「收」若指去掉 gap，这 10 张红，但 J1-a 的 696 个状态跟着红，打中第五节第一条。

### K1 的改法：几个改法各修哪一格（我提的只在我的模型上量过、被攻过零轮）

**gap-row（我提的收严，零轮）**：相邻两条候选根 txg 不相邻时，若最新根的实例表里有一行 (R⁻ 的实例, T = R⁻.txg)、R 的实例比 R⁻ 的大、两者之间每个实例的行都是 (j, 0, 0)，这一对照判；否则照 gap 整对报不适用。它比 gap 多要求的是：缺口能由实例表行证明是「新实例跳过的号」时不放过。代价：实例表行今天已为判抛弃读出来（同一份 `rows`），多一次比较（推的，没量时间）。

| 格 | TF + β + gap（现形） | 去掉 gap | gap-row（我提的） | 丙：等谱系记录 | 窄形（只在根槽读不出处整对不适用） |
|---|---|---|---|---|---|
| K1-b 10 张坏镜像（要红） | 不红（量过） | 红（量过：TF 2） | 红（量过：TF + gap-row 2、β + gap-row 2） | 推的：不红（没有回退） | 推的：红 |
| J1-a 终态 3 456 张（池级 checker 红 I-3.1 的 472 张 TF 红）与抬 F 那一步的头 700 个状态（696 个层 0 全绿而 TF 红） | 不红（量过） | 红（量过） | 不红（量过：三份 jwrap 与 `crash-head.log` 上 `tfgaprow`、`betagaprow` 0） | 推的：仍红（第二轮判决） | 推的：取决于「读不出处」怎么认 |
| 带崩溃历史 1 332 段、层 0 41 056 个状态 × 四张、604 段终态、2 712 段终态（合法，要不红） | 不红（量过） | 不红（量过） | 不红（量过；1 332 段里每个缺口对 gap-row 都照判了：`betagaprow` evaluated 255 742 = `beta` evaluated） | — | — |
| 记录撕裂 16 个子集挂载之后（合法） | 不红（量过） | 不红（量过） | 不红（量过） | — | — |
| J3-c 链改到 R⁻.txg（要红） | 红（量过：8） | 红 | 红（量过：8） | 推的：红 | 推的：红 |
| J3-d 再次出现（要红） | 红（量过：β + gap 2） | 红 | 红（量过：β + gap-row 2） | 推的：红 | 推的：红 |
| 第一轮 BI 六份（要红） | 红（第二轮量过 TF-gap 列） | 红 | 推的：红（都是相邻对，gap-row 与 gap 判法相同） | — | — |

gap-row 自己的射程（推的，没量）：它要 R⁻ 是实例表行点名的那一版、R 是之后某个实例在候选集里最早的根。若那个实例的写行根丢了、R 其实是它更后面的一条根（中间隔着一次回退），gap-row 会照判这一对、可能在合法镜像上红。单根环、不带介质故障时，R⁻（txg T）的槽在 T + 24 被盖掉，早于写行根（T + 2）的槽在 T + 26 被盖掉；R⁻ 是由记录施加的那一版时，它的槽里坐着 T − 24 的旧根，T + 24 落盘时旧根没了、最旧有效根升到 T + 2 以上，施加版跟着出候选集——所以两件事碰不到一起。这是按槽位公式推的，没造反例，也没证没有。

## K2　gap 之后还有没有别的洞形（没打中）

判据是第五节第一条：TF + β + gap 在池级 checker 与层 0 其余判定一项不红的镜像上红。下表每一行都按「池级 checker 0 红的那几张上 TF + β + gap 红没红」数；TF、β 不带 gap 的红另列，它们全落在池级 checker 已红的镜像上。

| 洞形 | 取样 | 池级 checker 0 红的镜像上 TF + β + gap 红 | 别的 |
|---|---|---|---|
| 可写挂载写行留下的缺口 | 层 0：可写挂载那条流 24 618 个状态逐个挂载、再接覆盖写 / 回退各一次；人造「写行崩在根槽之前」历史 | 0 | 缺口出现在 3 个状态上，每个都 0 红；第二次挂载不施加写行那一版，行记 (2, 0, 0) |
| 多记录发布的记录撕裂 + 挂载 | 16 个子集 × 挂载之后、再接覆盖写 / 回退 / 抬 F | 0 | 80 张池级 checker 都 0 红；有缺口的 6 个子集 TF 不带 gap 也 0 红（缺口右边的写行单元诞生代号 7 > 5） |
| 层 0 其余流挂载之后 | 回退 4 108、抬 F 12 330 个状态逐个挂载、再接两种动作 | 0 | 0 个缺口；45 个状态挂载被拒（`NewerStateStillUnreadableAfterOneReread`，回退流 5、可写挂载流 20、抬 F 流 20） |
| 带崩溃的历史扫描（崩溃 6 种：两种记录撕裂、写行 / 暖机崩在根槽之前、回退崩在根槽之前两种；挂载之后长度 0–2 的动作，字母表含崩溃 + 挂载） | 1 332 段终态 | 0 | 池级 checker 0 张红；TF、β、gap、gap-row 一次没红 |
| 两块盘各自的根环区域整块读坏（区域 0、2 在盘 0，区域 1 在盘 1） | 604 段 × 三个区域 = 1 812 张 | 0 | 池级 checker 红 1 653 张（多为 I-3.1）；β 不带 gap 红 133 张（再次出现，洞两侧），都在池级 checker 红的张上；TF + gap、β + gap 0 |
| 最新根自己的槽丢了 | 604 张 | 0 | 池级 checker 0 张红、0 个缺口（由记录补回那一版） |
| 设备失而复得（一块盘换回 1–3 步之前的那一份，两块盘各试） | 3 216 张 | 0 | 池级 checker 红 2 720 张（I-2.1、I-4.8、I-7.4 等）；可写挂载 3 216 次全被拒（`NewerStateStillUnreadableAfterOneReread` 与 `WritableMountRefusedByDevicesWithoutTheSelectedVer…` 各一半）；β 不带 gap 红 57 张都在池级 checker 红的张上 |
| 默认 768 MiB 环上更长的历史（能不能把记录也盖掉） | 算术，没跑 | — | 见下 |
| J1-a 短环（第二轮那一形，重跑） | 3 456 张终态、700 个层 0 状态 | 0 | 见 K3 |

（「604 段」= 前缀 A / R × 长度 0–2 的 86 段加长度 0–3 的 518 段，字母表 {覆盖写、建 1 个 inode、回退到第 1 条、回退到第 3 条、抬 F 1 步、重新挂载}，`logs/r3d-len2.log`、`logs/r3d-len3-{0,1,2}.log`；各键合计用草稿目录里的 `bin/agg.py` 相加。池级 checker 0 红的张上 TF / β 红会以 `…-red&pool-green` 键出现，四份日志里一行都没有。）

**默认 768 MiB 环（算术，推的）。** J1-a 要两件事同时成立：洞里那几版的根槽被根环盖掉（24 次发布之内），它们的记录也被 journal 环盖掉。环里能放的记录数是「pub const JOURNAL_RING_DEFAULT_BYTES: u64 = 805_306_368;」（`crates/singlefs-format/src/lib.rs:220`）除以「pub const JOURNAL_RECORD_BYTES: u64 = 4096;」（`crates/singlefs-format/src/lib.rs:198`），即 196 608 条；顺序写一个数据单元一条记录（第二轮 sw16 每次 16 条、我的 sw2 每盘 2 条）。所以 22 次发布里要写约 196 608 个数据单元，而旧根要留在候选集里，F 就不能抬过它，这些单元全得留着；一个数据单元「pub const DATA_UNIT_BYTES: u64 = 32768;」（`crates/singlefs-format/src/lib.rs:20`），即两个「pub const SLOT_BYTES: u64 = 16384;」（`crates/singlefs-format/src/lib.rs:14`）的槽，合计约 393 216 槽，超过 harness 4 GiB 盘的「单元区 211968 槽」（`crates/singlefs-harness/src/history.rs:109`）。所以在 harness 的盘宽上走不到；单元区够大（每盘约 6.5 GiB 以上）的池上推得出走得到，那时它与 J1-a 同形，gap 整对报不适用、gap-row 也不判（缺口在一个实例内部，没有行能说明）。

**四句不用答**：没有一格打中。「没打中」是一次抽样；取样的样子在「没打中的形状」一节。

## K3　终轮复核：第二轮的结论与数

### 数能不能核（都在我的副本上重做，副本 = 冻结副本 + `proto.diff`）

| 第二轮的数 | 重做 | 结果 |
|---|---|---|
| 攻方 696 / 700（抬 F 到 8 那一步的层 0 头 700 个状态，TF 红而层 0 整套判定全绿） | `crash-head`，`logs/crash-head.log` | 相同：第 34 行（下）；第 30 行这 696 个状态 I-3.1 报不适用。另：这 700 个状态上 TF + gap、β + gap、gap-row 都 0 红 |
| 攻方 472 张（J1-a 终态 TF 红） | `jwrap1-a1`、`jwrap1-a2`、`jwrap6-a2` | 172 + 256 + 44 = 472，相同；三份日志里 `CS2:tf-lenient`、`CS2:beta-lenient`、`CS2:tfgap-lenient`、`CS2:betagap-lenient`、`pool-checker`、`CS2:tf-strict`、`CS2:beta-re-lenient` 七行的 evaluated / violations / images 与第二轮 `logs/` 里同名日志逐字段相同（21 行逐行比，21 行相同） |
| 攻方 2 712 段（终态 TF、β 0 红） | 四份 `sweep-*-nofault`（不叠根槽读坏，只判终态） | R2_JOBS 728 + 728 + 728 + 528 = 2 712；`none:` 三行合计与第二轮八份日志合计相同：TF 561 292 次 0 红、β 591 682 次 0 红、池级 checker 2 712 张 0 红 |
| 辩方 2 892 万张 | 第二轮辩方模型拷进草稿目录，`python3 run_r2_deep.py ROLL 8 7`、`… ROLLKX 8 7` | 12 618 652 + 16 307 544 = 28 926 196 张，各写法红的张数逐行相同；只有「首例」不同（进程池分片的先后不同） |

```text
R2C name=jring=1MiB sw=16 d=0 [raiseTo8] :: tf-red-state:layer0-green evaluated=700 violations=696 states=696 first=jring=1MiB sw=16 d=0 [raiseTo8] state=500 versions=txg 8 [实例 1] | txg 11 [实例 1] | txg 12 [实例 1] | txg 13 [实例 1] | txg 14 [实例 1] | txg 15 [实例 1] | txg 16 [实例 1] | txg 17 [实例 1] | txg 18 [实例 1] | txg 19 [实例 1] | txg 20 [实例 1] | txg 21 [实例 1] | txg 22 [实例 1] | txg 23 [实例 1] | txg 24 [实例 1] | txg 25 [实例 1] | txg 26 [实例 1] | txg 27 [实例 1] | txg 28 [实例 1] | txg 29 [实例 1] | txg 30 [实例 1] | txg 31 [实例 1] | txg 32 [实例 1·由记录施加] | txg 33 [实例 2] | txg 34 [实例 2]
```

辩方那一行是抽象模型上的数（Python），不是 checker 上的数；我只核「数得出同样的数」，没核模型与实现对不对得上。

### 只红它的坏镜像在入库池级 checker 上（`logs/bad-r2-rerun.log`）

| 份 | 池级 checker（65 项） | TF | β | 备注 |
|---|---|---|---|---|
| J3-c 整条链改到 R⁻.txg（「严格小于」的只红它） | 61 成立、0 违例、4 不适用（I-7.9、I-8.8、I-3.12、I-4.1） | 8 | 8 | 第 68、69 行；TF + gap、gap-row 也是 8 |
| J3-d 第三步（β 再次出现的只红它） | 61 成立、0 违例、4 不适用（同上） | 0 | 2（再次出现） | 第 275、276 行；β + gap、β + gap-row 也是 2 |
| J3-d 的干净底图（J3-c 那段历史没单独报底图） | 61 成立、0 违例、4 不适用（同上） | 0 | 0 | 不适用的四项与坏镜像相同：坏镜像没让任何一项从成立变成不适用（J3-c 那一侧是推的） |
| J3-b 链改到 R⁻.txg 之下 + 压改动计数 | 0 违例 | 0 | 0 | 仍全漏（判别子观测不到，同前两轮） |

第一轮 V1 出局、取 TF 不取 V2s / V3s 那两条这一轮我没有再攻（分给辩方）；J3-c 在入库 checker 上仍只红 TF，「严格小于」那一处收严仍有只红它的坏镜像。

## 没打中的形状

| 形状 | 取样 | 打的是哪几格 |
|---|---|---|
| 层 0 崩溃状态逐个可写挂载、再接一步 | 前缀 A、384 槽小盘：回退 4 108、可写挂载 24 618、抬 F 12 330 个状态，全量；挂载之后再接覆盖写与回退（回退流是回退到第 3 条） | K1-a 频率、K2 |
| 带崩溃的历史，挂载之后的用户动作放开 | 1 332 段：前缀 2 种 × 崩溃 6 种 × 长度 0–2 的序列（字母表 10 个：覆盖写、建 1 个 inode、顺序写 2 单元、回退第 1 / 3 条、抬 F 1 步、重新挂载、记录撕裂 + 挂载、写行崩 + 挂载、回退崩 + 挂载） | K2 |
| 根环区域整块读坏、最新根槽丢、一块盘换回旧份 | 604 段（长度 0–3）各叠 3 + 1 + 至多 6 张 | K2 |
| 默认 768 MiB 环把记录也盖掉 | 算术 | K2 |
| 缺口右边出现用户数据单元的坏镜像（池级 checker 0 红） | 没造：带崩溃的缺口右边只有写行根（不碰 inode 树）；J1-a 那一形缺口右边是用户发布，但那几张终态池级 checker 先红 I-3.1，层 0 全绿的只有抬 F 那一步的崩溃状态，我没在它上面造坏单元 | K1-b |
| 记录撕裂的整条层 0 流 | 顺序写 2 单元那条流 16 777 240 个状态（4 GiB 下 268 435 480），超过约 10⁶，照定义没跑；只把记录那一段 16 个子集各当一段历史 | K1-a |
| 多次挂载、多次撕裂叠在一起，长度 3 以上 | 没扫 | K2 |

## 每批的段数、估时与层 0 状态账

- 层 0 枚举（`gap_frequency_over_crash_states_and_remount`）：先跑 1 000 个状态（回退流，不接后续动作）24.4 秒，估回退流 4 108 个状态接两种动作约 100–150 秒，实际 145.1 秒；可写挂载流先跑 4 000 个 118.0 秒，之后按 7 000 个一批：169.9、181.6、159.2 秒；抬 F 流 7 000 个 151.9 秒、5 330 个 135.3 秒。每次跑之前测试先打 `closed_form_state_count` 与带撕裂第三态的状态数（`R3G_HISTORY` 行）。
- 带崩溃历史扫描：先跑 34 段 3.2 秒，估 1 332 段约 125 秒，实际 38.1 秒，一批。盘与区域故障：长度 0–2 那 86 段 46.6 秒一批；长度 0–3 那 518 段分三批 173 / 173 / 172 段，117.1、105.8、115.4 秒。
- 重跑第二轮：三份 jwrap 89.8、111.3、180.0 秒；四份 sweep 53.6、51.9、52.7、32.3 秒；层 0 头 700 个 32.7 秒。
- 层 0 状态账（这一条腿）：跑完全量的 41 056 个（回退 4 108、可写挂载 24 618、抬 F 12 330，各流全量，没有一条超过约 10⁶）；第二轮那一段头 700 个；估时探针与被中途停掉的几次合计至多约 1.4 万个（4 GiB 回退流尾 20 个两次、6 MiB 环尾 20 个、头 700 个、384 槽 350、70、70、1 000 个；被停掉的：可写挂载流两批各至多 5 000、第二轮头 700 与 350、14）。合计不超过约 5.6 万，在约 10⁷ 以内。没跑的：顺序写 2 单元（16 777 240 / 268 435 480）、覆盖写（1 048 588 / 16 777 228）两条流超过约 10⁶；第二轮 `crash-full-raiseTo8`（196 665）按 700 个 32.7 秒估约 2.5 小时，超过 40 分钟，交主 agent。
- 越出定义的一处：我每条前台命令外面套了 `timeout`（115–235 秒，为了不撞前台 240 秒的上限），定义写的是批不设限时。被它停掉的 12 次（层 0 头 700 与 350 各一次、jwrap 分片三次、扫描探针一次、可写挂载流两批、盘与区域故障整批与两个半批、4 GiB 记录撕裂一次）输出都没用，表里的数全来自跑完退 0 的那几次。那几次慢主要是内存包装排队（4G 上限时 slice 已占 36.5 GiB，排了 155 秒）与机器负载（`ps` 看到别的会话一个 `fault_injection_fast_tier --ignored` 占 2 500% 以上 CPU、负载均值 50–80）；改用 2G 上限之后不再排队。

## 要主 agent 跑的命令

1. 全量复跑：工作目录仓根，`bash research/prompts/m3-birth-floor-r3-opus-model/rerun.sh /tmp/claude-1000/m3-batch5-freeze/tree <新的草稿目录>`；脚本内部经 `research/scripts/run-with-memory-cap.sh` 2G、`research/scripts/capped.sh` 7 线程；串行约 40 分钟（推的，各段 seconds 相加）。看每段末尾打出的汇总行，与本报告各表、`logs/` 逐行比；K1-b 看 `bad-gap2` 段每张的 `pool_red=`、`tf=`、`tf+gap=`、`tf+gaprow=`。
2. J1-a 那一段的层 0 全量（第二轮也交过）：同上加段名 `crash-full-raiseTo8`（196 665 个状态，约 2.5 小时，推的）。现在多看四行：`crash:CS2:tfgap-lenient`、`crash:CS2:betagap-lenient`、`crash:CS2:tfgaprow-lenient`、`crash:CS2:betagaprow-lenient` 的 violations（头 700 个上都是 0），以及 `tf-red-state:layer0-green`。

## 这条腿自己的限度

- K1-b 只在一段历史（覆盖写两次、记录撕裂、挂载）上造，坏单元都是写行那次发布的固定点单元；没造用户数据单元落在缺口右边、池级 checker 又 0 红的坏镜像（推的：带崩溃的缺口右边只有写行根）。
- 记录撕裂与写行崩溃这两种缺口来源，我是用人造的崩溃步（只留选中的写）造出来的，它们是层 0 枚举域里的状态，但那两条流的全量超过约 10⁶，没有逐个枚举；可写挂载流里的 3 个是全量枚举里数出来的。
- 「写行那一版不被下一次恢复施加」是我观测到的实现行为（`rows=(1, 7, 0) (2, 0, 0)`），没去查它是不是条款要的；它决定了第 2 种缺口的出现方式。
- gap-row 只在我的模型上量过、被攻过零轮；它在「写行根丢了而 R⁻ 还在」那一形上可能误红，我按槽位公式推这一形在单根环、无介质故障时碰不到，没造反例。
- 384 槽小盘与 4 GiB 两种几何各用了一部分：层 0 枚举、扫描在小盘上，坏镜像与盘 / 区域故障在 4 GiB 上（改造法与区域偏移按 4 GiB 写死）。
- 「没打中」都只抽了一次样。

## 什么现象会推翻这些结论

- K1-b：在入库装置上重造这段历史，候选集里 txg 5 与 7 之间没有缺口（checker 把 6 补进来，或实例 2 从 6 起），或那 10 张里池级 checker 有任何一项违例。
- K1-a「不带崩溃的历史上没有缺口」：找到一段不带崩溃、不带介质故障、环长取默认的合法历史，CS2 里出现缺口对。
- K2 没打中：找到一张池级 checker 与层 0 其余判定一项不红、而 TF + β + gap 红的镜像。
- gap-row：一张池级 checker 0 红的合法镜像上 `tfgaprow` 或 `betagaprow` 红。
- K3：同一份 `proto.diff` 在冻结副本上重跑，出来的数与上表任何一格不同。

## 没做什么

- 不判辩方那几格（K4 措辞、K5 与 I-3.1 的关系），不替主 agent 采纳；副本上的数不算入库装置上的数。
- 没跑 checker 档包的任何测试、门禁、QEMU、变异；没改仓里 `crates/` 一个字，改动都在草稿目录的副本上，以 `proto.diff` 入库。
- 没读禁读清单里的文件（这一轮辩方的报告与模型、同批 m3-ring-r3 的产出）。
- 没重攻「V1 出局、取 TF 不取 V2s / V3s」（分给辩方）；没造 J1-a 上的坏单元；`crash-full-raiseTo8` 全量没跑。

## 删了哪些

- `/tmp/claude-1000/m3-birth-floor-r3-opus/tree`（冻结副本的拷贝，连同编译目录 `target/`，1.4G）：已删。
- `/tmp/claude-1000/m3-birth-floor-r3-opus/r2-sonnet-copy`（第二轮辩方模型的拷贝，64K）：已删。
- 核 `proto.diff` 用的第二份拷贝 `patchcheck/`：核完当场删了。
- 草稿目录里留着全部日志（含没入库的探针与被停掉的那几次）、几个小脚本（`bin/t.sh`、`bin/agg.py`）、`model/`（入库那份的草稿）、`model-proto.diff`、`r3bad_head.rs`，共约 6.4M，不是仓副本也不是编译目录。

## 引文核对

`python3 research/scripts/cite-check.py research/prompts/m3-birth-floor-r3-opus-output.md` 末行：

```text
  ✓ 核了 13 处引文，对上 13 处，没判 0 处（逐处列在上面）
```
