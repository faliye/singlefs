# m3-birth-floor-r1 攻方腿报告（2026-10-02）

<!-- doc-lint:not-numbers V1 V2 V3 F1 F2 F3 BI CS1 CS2 -->

批次：records/2026-09-30-里程碑三开工第一轮.md#4。这是接手的腿：前任撞了用量上限，交接摘要里它只拷好了冻结副本并打上上一轮攻方的 `proto.diff`、编过一次 dev（`build1.log`），一个原型都没写、一个状态都没跑。我核了那份残留（`crates/` 与冻结副本逐文件 sha256 只差 `proto.diff` 改到的那几份），接着用它，原型、用例、跑数、报告都是这一条腿做的。

## 复跑与文件

复跑：`bash research/prompts/m3-birth-floor-r1-opus-model/rerun.sh /tmp/claude-1000/m3-batch4-freeze/tree <草稿目录> [crash]`。它把冻结副本拷进草稿目录、`patch -p2` 打上 `proto.diff`，经内存包装（8G）、线程上限 4 编 release 并跑原型包 `bf-proto` 的坏镜像用例；带 `crash` 再跑第一批四段历史的全量崩溃状态（458 857 个状态，本机约一小时）。不跑 checker 档包的任何测试：`bf-proto` 只把 `singlefs-checker-tier` 当库链进来、调它的 `judge_layer0_state_range`，测试目标名里没有 layer0。

我在一份新拷的冻结副本上跑过一遍不带 `crash` 的复跑（那一版补丁不带 V3），`BI` 行与 `logs/bad2.log` 逐字节相同；之后补丁加了 V3（只多一个谓词），重出的补丁在新拷的冻结副本上打完与我的副本逐文件相同（`diff -rq` 无输出，不比 `Cargo.lock`）。

```text
f73a00dfcd41a1ee4caa566106fa4d9f82b5ddd854fbd158c201c9a220784b7b  logs/bad3.log
b9dfdabb97f12eb1c13f5c2fad9bd3f331e9da0bac0a88b81c6da7b57042b225  logs/bad2.log
44b986a175aab29e03d37c9633a34a14452d95475726212bc29795b95f4b9a6f  logs/count-probe.log
941dfd7a1b5617af9b1d09808829b040267b6193f8436579434efa2f552f9cdc  logs/count-x.log
d95274c98848db7a727fad4f6c3c5b3ce4a886e688882acf8d7b7b7c0f4d72f1  logs/progress.md
17127c1004985a208e9683f6c37834f3dad022e9f0a1524bf132fea78a6c736c  logs/run1-ni300-ow-ni1-ow_rb3__raiseTo5_.log
de4e1e2eeaf748cbddfb2d95b64dd291a4945c56e59996bb9fa4ee17e9ff4f3b  logs/run1-ni300-ow-ni1-ow_rbcrash3-remount-raise5__rb1_.log
eac2619b3e77abd31f03db22da4ccee196230b8aa4f3decdb1560a0278f3c78f  logs/run1-sw3-ni240-ow-ni5_rb3__raiseTo5_.log
8781ce7a18d8267c5080ea0c778610a7bc1ef2e12a23312497e013c1d5c7e436  logs/run1-sw3-ow-sw2-ow_rb3__raiseTo5_.log
606fa5a6d0231751bb1ada93347941c01a80841853f4adbced76547af812ddee  logs/run2-ni300-ow-ni1-ow_rbcrash3-remount-raise5__rb1_.log
e8ef1ae61ea0e89f6f91a4bc6ac98c9010087c380a3083725bc1ac1310d2ebb0  logs/run2-sw3-ow-sw2-ow_rb3__raiseTo5_.log
41111e77a517321a5e99080a8eb67fb70ebf753438d79f80d3355cc57d68906e  logs/time-probe1.log
6b4e6eff2191438fe32563f819bb4ccd41b3e4b3cc4ddac5306f68adb9f985e9  logs/time-probe2.log
1cea152144a113305f6c3049ed9e93f9ee67d0869a19e1127aeb5ef25f2772e0  logs/time-probe3.log
7b8021b75e81208f1c3ec66c503502891f733185295d420d22e533930dce28e5  logs/time-probe4.log
a2c2c0a2907b1e58ba694c336717c15363ad7828742205a679f456c13b969ece  proto.diff
9508cf200008534a7441bffda69b5f62dfd5f05be83f69196bbdcefb23fde6f7  rerun.sh
```

`logs/run1-*` 是补丁加 I-3.5 ① 计数与 V3 之前跑的（三种写法与 CS1 / CS2 的数与之后同一份代码），`logs/run2-*` 是加了之后对其中两段的重跑；`logs/bad2.log` 不带 V3，`logs/bad3.log` 带。全部数都是冻结副本拷贝上的数，不算入库装置上的数。

装置：`proto.diff` 在副本的 checker 里给上一轮的只读模块 `walk/proto_b.rs` 加一个 `birth_floor_judgements`，把诞生代号下界写成四个谓词——原措辞、V1（上一轮攻方的措辞）、V2、V3（这一条腿提的收严，见 F2 一节），各在两种候选集上判：CS1 = 根环里的候选根（与上一轮原型同一份算法）；CS2 = CS1 再并上 `check_pool_image` 补进候选集的「由记录施加出来、根槽从没落盘的那一版」（冻结副本 `crates/singlefs-checker/src/walk.rs:5570`：「/// 回退候选集要补上的那几版：**由记录施加出来、根槽从没落盘的那一版**（增补 2 收口表第 54 行，」），那一版的引用集合从它所施加的那条根的记录换上新根段四个字段之后走。新包 `crates/bf-proto` 是驱动：内存稀疏盘上 mkfs、取号、暖机、写第一个文件（起点只建一次，每段历史从它拷），操作有覆盖写、顺序写（多个数据单元）、建 inode（300 个跨过一容器 233 条的门槛，inode 树多叶）、回退、抬 F、重新可写挂载、「回退那次发布崩在根槽 FUA 之前」（单元与记录持久、根槽与之后的写都没持久，下一次挂载由记录施加出那一版）。

## 各格判定一览

| 格 | 被攻的措辞 | 判定 | 依据（副本上量的） |
|---|---|---|---|
| F1 原措辞 | I-3.5 ② 按回退候选集判 | 照上一轮：合法镜像上红 | 四段历史的全量崩溃状态，三段抬 F 的 131 103 个状态里 131 099 个红、由记录施加回退那一段 65 548 个全红；红的落点逐个归类，全部是「R_first 满足 V1 的豁免条件、落点在继承来的子树之下」那一类 |
| F1 V1 | 回退根继承整棵用户可见树时下界不判 | **没打中**（合法镜像上 0 红） | 四段历史、458 857 个崩溃状态（层 0 全量展开）每个状态都判，CS1、CS2 都 0 红（其中两段加了 V3 之后重跑，又判 196 651 个，同样 0 红）；多叶 inode 树、多单元文件、由记录施加出来的回退版本三种形状都在里面 |
| F2 V1 漏红 ① | V1 按 R_first 整条根免判 | **打中：坏镜像上漏红** | 回退根自己新写的单元（树表、中央映射根、记账根、分配记录树叶）诞生代号改小、指针与映射 key 与分配代与 journal 点名项一起改：池级 checker 0 红，原措辞红、V1 不红 |
| F2 V1 漏红 ② | V1 的豁免条件读的是 inode 树根指针的出生 txg | **打中：坏镜像上漏红** | 一次普通发布，只把它新写的 inode 树根的诞生代号改小一格：池级 checker 0 红，V1 的豁免条件被这一处改动自己打开，V1 不红 |
| F2 V1 漏红 ③ | 同上 | 打中：坏镜像上漏红（V2、V3 同样漏） | 整条用户可见链（数据单元、extent 树、inode 叶、inode 根）一起改小：池级 checker 0 红，V1、V2、V3 都不红 |
| F2「只红它」 | 正文第五节第二条 | 没打中（攻不倒 V1 这一侧） | 普通发布新写的数据单元、inode 叶、树表、映射根等单个改小：池级 checker 0 红而 V1 红——V1 有只红它的坏镜像 |
| F2 候选集 | V1 没写判在哪个候选集上 | 措辞空缺，判定上没分出差别 | 由记录施加出来的回退版本在场时，CS1 的 R_first 是它后面那条写行根（实例 2），CS2 的 R_first 是它自己（实例 1）；两边 V1 都 0 红 |
| F3（顺带） | I-3.5 ① 连续无洞 | 合法镜像上不成立 | 回退目标还在候选集里时，被回退复活的落点引用集合有洞（例：txg [4, 8]）；抬 F 越过回退目标之后就连续了 |

「没打中」都是这一条腿一次抽样的结果；取样范围在「没打中的形状」一节。

## F1　V1 收窄之后在合法镜像上还红不红（没打中）

### 取了哪些历史

起点（只建一次，不枚举崩溃状态）都是第一个文件之后（txg 3）再做几步，然后回退到从新到旧第 4 条（txg 4）。要让回退目标离开候选集又留下它之后的根，得抬 F 越过 txg 4；F 的上限是「第 4 新的非空持久有效根」（冻结副本 `crates/singlefs-core/src/mount.rs:1847`：「/// 抬 F 的上限（D16（发布语义） 已定项 1）：min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根)；非空有效根不足 4 个时取最旧的有效根。」），所以回退目标之后要有三次改变用户可见树的发布。枚举崩溃状态的是起点之后的后缀：

| 历史（起点 / [枚举的后缀]） | 起点里多出来的形状 | 层 0 状态数（带撕裂第三态） | 闭式 |
|---|---|---|---|
| 建 300 个 inode、覆盖写、建 1 个 inode、覆盖写、回退到 txg 4 / [抬 F 到 5] | inode 树两片叶；回退复活旧的两片叶 | 131 103 | 131 088 |
| 顺序写 3 单元、覆盖写、顺序写 2 单元、覆盖写、回退到 txg 4 / [抬 F 到 5] | 文件 3 个数据单元，回退复活三个 | 131 103 | 131 088 |
| 顺序写 3 单元、建 240 个 inode、覆盖写、建 5 个 inode、回退到 txg 4 / [抬 F 到 5] | 两样一起 | 131 103 | 131 088 |
| 建 300 个 inode、覆盖写、建 1 个 inode、覆盖写、回退那次发布崩在根槽之前、重新可写挂载、抬 F 到 5 / [再回退到从新到旧第 2 条] | 回退版本由记录施加、根槽从没落盘；之后是实例 2 的写行根 | 65 548 | 65 543 |

四段合计 458 857 个状态，每个状态跑层 0 的整套判定（冻结副本 `crates/singlefs-checker-tier/src/crash.rs:1240`：「    let pool_checker = check_pool_image(image);」，连同两种 journal 政策的恢复与两份 oracle、记录核对器），再在同一张崩溃镜像上判四种写法。四段的层 0 判定都是 0 个红的状态（`layer0-red-states evaluated=131103 violations=0` 三行与 `evaluated=65548 violations=0` 一行）。

### 结果（原样，`logs/run1-*.log` 与 `logs/run2-*.log`）

```text
BF name=sw3-ow-sw2-ow/rb3/[raiseTo5] :: crash:CS1:lower evaluated=11799742 violations=1835386 states_red=131099 first=sw3-ow-sw2-ow/rb3/[raiseTo5] state=2 :: 原措辞：盘 0 槽 50182 类 1 诞生代号 4 ≤ R_prev (实例 1, txg 7)，R_first (实例 1, txg 8)
BF name=sw3-ow-sw2-ow/rb3/[raiseTo5] :: crash:CS1:v1 evaluated=7866708 violations=0 states_red=0 first=-
BF name=sw3-ow-sw2-ow/rb3/[raiseTo5] :: crash:CS2:v1 evaluated=7866708 violations=0 states_red=0 first=-
BF name=sw3-ow-sw2-ow/rb3/[raiseTo5] :: crash:CS1:v3 evaluated=9964356 violations=0 states_red=0 first=-
BF name=ni300-ow-ni1-ow/rbcrash3-remount-raise5/[rb1] :: crash:CS2:applied-only-versions evaluated=65548 violations=0 states_red=0 first=ni300-ow-ni1-ow/rbcrash3-remount-raise5/[rb1] state=0 :: 实例 1、txg 8
BF name=ni300-ow-ni1-ow/rbcrash3-remount-raise5/[rb1] :: crash:CS1:lower evaluated=9045768 violations=524384 states_red=65548 first=ni300-ow-ni1-ow/rbcrash3-remount-raise5/[rb1] state=0 :: 原措辞：盘 0 槽 50180 类 1 诞生代号 3 ≤ R_prev (实例 1, txg 7)，R_first (实例 2, txg 9)
BF name=ni300-ow-ni1-ow/rbcrash3-remount-raise5/[rb1] :: crash:CS2:lower evaluated=10094536 violations=524384 states_red=65548 first=ni300-ow-ni1-ow/rbcrash3-remount-raise5/[rb1] state=0 :: 原措辞：盘 0 槽 50180 类 1 诞生代号 3 ≤ R_prev (实例 1, txg 7)，R_first (实例 1, txg 8，由记录施加)
BF name=ni300-ow-ni1-ow/rbcrash3-remount-raise5/[rb1] :: crash:CS1:v1 evaluated=7079328 violations=0 states_red=0 first=-
BF name=ni300-ow-ni1-ow/rbcrash3-remount-raise5/[rb1] :: crash:CS2:v1 evaluated=8521384 violations=0 states_red=0 first=-
```

四段逐段的数：

| 历史 | 原措辞 CS1 红 / 判 | V1 CS1 红 / 判 | V1 CS2 红 / 判 | 原措辞红的落点里属于「R_first 满足 V1 条件、落点在继承子树下」的 |
|---|---|---|---|---|
| 300 inode … / [抬 F] | 1 048 792 / 9 964 292 | 0 / 6 817 852 | 0 / 6 817 852 | 1 048 792 / 1 048 792 |
| 顺序写 3 … / [抬 F] | 1 835 386 / 11 799 742 | 0 / 7 866 708 | 0 / 7 866 708 | 1 835 386 / 1 835 386 |
| 顺序写 3 + 240 inode … / [抬 F] | 524 396 / 9 439 920 | 0 / 6 817 876 | 0 / 6 817 876 | 524 396 / 524 396 |
| 由记录施加的回退 … / [再回退] | 524 384 / 9 045 768 | 0 / 7 079 328 | 0 / 8 521 384 | 524 384 / 524 384（CS1、CS2 各自） |

### F1 的归类

正文要的归类（`research/prompts/_m3-birth-floor-r1-body.md:28`）：我这四段里原措辞红的每一个落点都落在「R_first 的 inode 树根指针出生 txg ≤ R_prev.txg、落点 R_prev 不引用」且「落点在 R_first 继承来的用户可见子树之下」那一格（日志里 `red-class:v1-root-and-under-inherited` 的计数等于原措辞的违例数，另外三类一次都没出现）。R_first 有两种：回退根本身（实例 1、txg 8），或由记录施加出回退版本之后的写行根（CS1 上是实例 2、txg 9；CS2 上 R_first 换成被施加出来的那一版）。两种都整棵照抄 R_old 的 inode 树、extent 树（冻结副本 `crates/singlefs-core/src/mount.rs:5220`：「/// 持久顺序）：新根的 inode 树、extent 树取 R_old 的；树表单元、分配记录树、中央映射树与记账统计量按这次发布照调整之后的分配器现算；」）。

为什么我造不出 V1 红的合法镜像（推的，供下一轮去打）：一个落点在候选集里被 R_first 引用、却不被 R_prev 引用，而出生早于 R_prev，只能是 R_first 的父版本不在候选集而 R_prev 在。今天候选集按 txg 是一段连续的区间（F 之上、被实例表判抛弃的除外，冻结副本 `crates/singlefs-checker/src/walk.rs:5977`：「            let walked = *index == newest_index || (!abandoned && !below_floor);」），被抛弃的根不在任何活下来的根的祖先链上，于是这种落点只能来自回退（父版本是 R_old）——V1 的条件正好认它。今天的用户改动里没有「改 extent 树而不改 inode 树」的一种：覆盖写与顺序写都换 inode 记录，建 inode 只换 inode 树、不动 extent 树（冻结副本 `crates/singlefs-core/src/transaction.rs:4848`：「/// 这次发布之后水位加 N；数据单元与 extent 树不动（新建的 inode 长度 0，`blocks` = ⌈0 ÷ 512⌉ = 0）。」），所以只看 inode 树根指针在今天够用。

## F2　V1 的措辞与判法（坏镜像上漏红：打中两处）

被攻的措辞是上一轮攻方报告里的 V1（`research/prompts/m3-inv-b-r1-opus-output.md:105`：「V1：R_first 的树表里 inode 树根指针的出生 txg ≤ R_prev.txg，且那个根指针的落点 R_prev 不引用——这条 R_first 的用户可见树是整棵从候选集之外的一版继承来的——这时它名下「最早由它引用」的落点都不判下界；否则照原措辞判。」）。攻击面是正文分工表里攻方那一行的「在坏镜像上漏红的」（`research/prompts/_m3-birth-floor-r1-body.md:41`）。

### 坏镜像怎么造（`tests/bf_bad_images.rs`）

从合法历史的终态镜像出发，把一个单元头里的诞生代号从 n 改成 m（m ≤ R_prev.txg），并一起改：指它的每条码 2 指针与根槽里指针的诞生代号、中央映射条目 key 的出生 txg（连同节点头里与它相同的最小 / 最大 key）、分配记录树叶里两盘那两条未释放记录的分配代、journal 里点名它的那一项的诞生代号；每个被改的单元重封两道校验和，新的整单元校验和沿引用链补进父节点、根槽与 journal 点名项（改过的记录重接反向链）。目的是让盘上除了「诞生代号比写它的那次发布早」之外处处自洽，看池级 checker 与四种写法谁红。有几份装置没造干净（分配记录树中间节点、改了以后中央映射 key 顺序被打乱，池级 checker 红 I-1.1 或 I-2.1），下面只用池级 checker 0 红的那几份。

### 结果（`logs/bad3.log` 逐份整理，池级 checker 0 红的才列）

| 份 | 起点（合法历史终态） | 改了哪个单元 | 原措辞 | V1 | V2 | V3 |
|---|---|---|---|---|---|---|
| ① 回退根自己新写的单元 | 覆盖写 3 次、回退到 txg 4（回退根 txg 7，R_old 还在候选集） | 槽 50289（分配记录树叶）、50294（记账根，key 宽 22）、50295（中央映射根，key 宽 27）、50296（树表，key 宽 8），各 7 → 6，一份改一个 | 红 | **不红** | 红 | 红 |
| ①b 同上，R_old 已离开候选集 | 覆盖写 4 次、回退到 txg 4、抬 F 到 5（回退根 txg 8） | 槽 50301、50306、50307、50308，各 8 → 7 | 红 | **不红** | 红 | 红 |
| ② 普通发布新写的 inode 树根 | 覆盖写 3 次（最新根 txg 6，没有回退） | 槽 50280（key 宽 8，改它 V1 的条件就打开，所以是 inode 树根）6 → 5，只改这一个 | 红 | **不红** | **不红** | 红 |
| ②b 普通发布新写的 extent 树根 | 同上 | 槽 50277（key 宽 24）6 → 5 | 红 | 红 | **不红** | 红 |
| ③ 整条用户可见链 | 同上 | 50186（数据单元）、50277、50278（inode 叶，类 3）、50280 依次 6 → 5 | 红 | **不红** | **不红** | **不红** |
| ④ 普通发布新写的单个单元 | 同上 | 50186、50278、50281、50286、50287、50288，各 6 → 5，一份改一个 | 红 | 红 | 红 | 红 |

原样（`logs/bad3.log`）：

```text
BI BI-1 slot=50296 class=2 key_width=8 counts=[1, 0, 2, 1] pool_checker_violated=[]
BI BI-1 slot=50296 class=2 key_width=8 counts=[1, 0, 2, 1] CS1:lower evaluated=112 violations=2 first=原措辞：盘 0 槽 50296 类 2 诞生代号 6 ≤ R_prev (实例 1, txg 6)，R_first (实例 1, txg 7)
BI BI-1 slot=50296 class=2 key_width=8 counts=[1, 0, 2, 1] CS1:v1 evaluated=96 violations=0 first=-
BI BI-1 slot=50296 class=2 key_width=8 counts=[1, 0, 2, 1] CS1:v2 evaluated=112 violations=2 first=V2：盘 0 槽 50296 类 2 诞生代号 6 ≤ R_prev (实例 1, txg 6)，R_first (实例 1, txg 7)
BI BI-3 slot=50280 class=2 key_width=8 counts=[1, 1, 2, 1] pool_checker_violated=[]
BI BI-3 slot=50280 class=2 key_width=8 counts=[1, 1, 2, 1] CS1:lower evaluated=96 violations=2 first=原措辞：盘 0 槽 50280 类 2 诞生代号 5 ≤ R_prev (实例 1, txg 5)，R_first (实例 1, txg 6)
BI BI-3 slot=50280 class=2 key_width=8 counts=[1, 1, 2, 1] CS1:v1 evaluated=72 violations=0 first=-
BI BI-3 slot=50280 class=2 key_width=8 counts=[1, 1, 2, 1] CS1:v3 evaluated=96 violations=2 first=V3：盘 0 槽 50280 类 2 诞生代号 5 ≤ R_prev (实例 1, txg 5)，R_first (实例 1, txg 6)
BI BI-2 slots=[50186, 50277, 50278, 50280] pool_checker_violated=[]
BI BI-2 slots=[50186, 50277, 50278, 50280] CS1:lower evaluated=96 violations=8 first=原措辞：盘 0 槽 50186 类 1 诞生代号 5 ≤ R_prev (实例 1, txg 5)，R_first (实例 1, txg 6)
BI BI-2 slots=[50186, 50277, 50278, 50280] CS1:v1 evaluated=72 violations=0 first=-
BI BI-3 slot=50186 class=1 key_width=1 counts=[1, 1, 2, 1] pool_checker_violated=[]
BI BI-3 slot=50186 class=1 key_width=1 counts=[1, 1, 2, 1] CS1:v1 evaluated=96 violations=2 first=V1：盘 0 槽 50186 类 1 诞生代号 5 ≤ R_prev (实例 1, txg 5)，R_first (实例 1, txg 6)
```

（`counts` 四格是改了几处：指针、映射 key、分配记录、journal。日志里份名 BI-1、BI-1b、BI-3、BI-2 对应表里的 ①、①b、②/②b/④、③。）

合法那一侧同一份用例里的三张终态镜像（回退目标在、回退目标被抬 F 越过、没有回退）上 V1、V2、V3 都 0 红，池级 checker 0 红；崩溃状态那一侧见 F1 一节，V2、V3 在两段重跑里 0 红（`crash:CS1:v3 evaluated=9964356 violations=0` 与 `crash:CS2:v3 evaluated=9570152 violations=0`）。

### 打中之后的四句

**漏红 ①（回退根自己新写的单元）**

1. 分不分辨臂：分。原措辞、V2、V3 红，V1 不红。病根是 V1 的免判单位：它按 R_first 整条根免，回退根这一次发布新写的固定点单元（树表、分配记录树、中央映射树、记账树——它们「按这次发布照调整之后的分配器现算」，见上面 `mount.rs:5220` 那一行）跟着继承来的用户可见树一起被免掉。合法镜像上每张回退之后的图都有这一批：`v1-exempted-where-lower-holds`（V1 免了而原措辞本来判得绿的落点）在 `logs/run1-sw3-ow-sw2-ow_rb3__raiseTo5_.log` 里是 2 097 648 次，每个状态 16 次。
2. 被判的系统看不看得到判别它的东西：看得到。坏单元不在 R_first 树表里任何一棵「继承来的」用户可见树之下，这是盘上走得出来的结构；V2 用的就是它。
3. 满足的是判据字面的哪一句：正文第五节没有一条写「坏镜像上漏红 ⇒ …」。字面上最近的一条是「V1 写不出一份只红它的坏镜像 ⇒ 下界不立，状态列写明」（`research/prompts/_m3-birth-floor-r1-body.md:48`），这一格**不满足**它：表里第 ④ 行就是 V1 只红它的坏镜像。所以这一格交出去的是「V1 在哪类坏镜像上丢了判别力」，不是一条跑前判据打中。
4. 跑前条款给的改法：正文没给改法。我提的 V2、V3 在这一格上都红（量过）。

**漏红 ②（只改 inode 树根的诞生代号）**

1. 分不分辨臂：分。原措辞、V3 红，V1、V2 不红。V1 的判别子就是被改的那个字段：把一次普通发布新写的 inode 树根的诞生代号改成 ≤ R_prev.txg，V1 的豁免条件就成立，免掉的包括这个坏单元自己与这次发布写的全部单元。
2. 看得到吗：看得到。被改的树根比它下面的单元出生得早（inode 叶 50278 的诞生代号还是 6、树根变成 5）；合法的回退根继承的是整棵旧树，旧树里没有比它的根更晚出生的单元。V3 加的就是这一条。
3. 满足的字面：同 ①，不满足任何一条跑前判据。
4. 改法：V3 在这一格上红，V2 不红（量过）。

**漏红 ③（整条用户可见链一起改小）**

1. 分不分辨臂：分原措辞与 V1、V2、V3，后三者分不开（都不红）。
2. 看得到吗：**看不到**（今天）。盘上剩下的东西与一次合法的回退逐项同形：用户可见树整棵「出生早于 R_prev、R_prev 不引用」，树内诞生代号自上而下不增。唯一能分开的是「这条根是一次回退」本身。D23 已定项 14 写了回退那次发布要写谱系记录（冻结副本那份文件第 376 行，今天主工作区里挪到了第 377 行；`.claude/kb/decisions/23-journal的角色与格式.md@已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换`：「墓碑树、谱系树随现行那一版，不取 R_old，回退那次发布写一条谱系记录 (T_rb, T_old)」），但今天的实现不写：冻结副本 `crates/singlefs-core/src/mount.rs` 里「谱系」命中 0 次（`grep -c '谱系'` 输出 `0`）。这是「判别子观测不到」那一形，照 evidence-discipline 的表是改判据、新一轮的事，不是 V1 的措辞能修的。
3. 满足的字面：同 ①。
4. 改法：V1、V2、V3 都不红（量过）；要红只能等谱系记录落地后改用它判（推的，没实现）。

### 我提的改法 V2、V3（只在我的模型上量过、被攻过零轮）

- **V2**：不按 R_first 整条根免，只免落点在 R_first「继承来的用户可见树」之下的：R_first 树表里 extent 树、inode 树、`deleted_inodes` 那几条的根指针，出生 txg ≤ R_prev.txg 且那个落点 R_prev 不引用，就算继承来的；落点在其中某一棵之下（含树根自己）才免，其余照原措辞判。
- **V3**：V2 再收严一条——那棵继承来的树要「压得住」：它下面每个单元头里的诞生代号都 ≤ 树根指针的出生 txg，否则不算继承来的。

**注意，原型里实现的 V2、V3 不是 V1 的严格收严**：它们不要求 V1 的条件（inode 树根是继承来的），extent 树单独「继承来」也免——坏镜像 ②b 上 V1 红而 V2 不红，就是这一处放宽。按「只许收严」，交出去的形态应当是 **V2s = V1 的条件成立 ∧ 落点在继承子树之下才免**、**V3s = V2s ∧ 那棵子树压得住**，多要求的分别是「免判只到继承子树为止」与「继承子树自上而下不增」。V2s、V3s 我没单独实现，下表那两列是从量到的数推出来的：合法那一侧，原措辞红的每个落点都同时满足「V1 条件」与「在继承子树下」（F1 一节的归类计数），V3 在重跑的两段 0 红，所以 V2s 在四段、V3s 在两段上也是 0 红；坏镜像那一侧，①、①b、②、③ 上 V1 条件成立，V2s / V3s 与 V2 / V3 判得相同，②b 上 V1 条件不成立，V2s / V3s 照原措辞判、红。

| 格 | V1 | V2 | V3 | 量过 / 推的 |
|---|---|---|---|---|
| F1 四段合法历史的崩溃状态 | 0 红 | 0 红（四段） | 0 红（重跑的两段） | 量过：`logs/run1-*`、`logs/run2-*`；V3 只在重跑的两段（196 651 个状态）上量过 |
| 坏镜像 ①、①b（回退根自己新写的单元） | 漏 | 红 | 红 | 量过：`logs/bad3.log` |
| 坏镜像 ②（只改 inode 树根） | 漏 | 漏 | 红 | 量过 |
| 坏镜像 ②b（只改 extent 树根） | 红 | 漏 | 红 | 量过 |
| 坏镜像 ③（整条链） | 漏 | 漏 | 漏 | 量过 |
| 坏镜像 ④（普通发布单个单元） | 红 | 红 | 红 | 量过 |
| 多文件、删除落地之后（`deleted_inodes` 有根、改 extent 而不改 inode 树的操作出现） | 只看 inode 树根，可能误免或误判 | 按三棵树判 | 同 V2 | 推的：今天没有这种操作，没造 |
| 谱系记录落地之后的「整条链偏小」 | 漏 | 漏 | 漏 | 推的：要改成按谱系记录认回退，三条都不是 |

V2s、V3s（上一段）在表里各行与 V2、V3 相同，只有 ②b 一行是「红」（推的，没单独实现）。

V3 的代价：每个继承来的树根要把整棵子树的头读一遍（我的原型对每个待判落点各走一遍，没做缓存，耗时没单独量）。

### 「只红它的坏镜像」这一条攻不倒 V1

表里第 ④ 行：一次普通发布（txg 6）新写的数据单元、inode 叶、分配记录树叶、记账根、中央映射根、树表，任挑一个把诞生代号改成 5、其余照上面一起改，池级 checker 0 红、V1 红。按正文第五节第二条，这一格对 V1 不成立；我只在冻结副本今天的 checker 上量过，下一轮 I-8.11、I-1.12 等新立的几条落地之后，这些镜像会不会被别的不变量一起红，没量。

### V1 的措辞还缺两样（不是打中，是空缺）

- **判在哪个候选集上**：由记录施加出来、根槽从没落盘的回退版本在场时，根环候选集（CS1）上的 R_first 是它后面那条写行根（实例 2、txg 9），`check_pool_image` 的候选集（CS2）上是被施加出来的那一版（实例 1、txg 8，见 F1 一节两行原样）。两边 V1 都 0 红、免掉的落点数不同（`v1-exempted-where-lower-holds` CS1 1 442 056、CS2 1 048 768，`logs/run1-ni300-ow-ni1-ow_rbcrash3-remount-raise5__rb1_.log`）。措辞要写明候选集，CS2 上 R_first 的树表指针要从记录的新根段取。
- **R_prev 数不全时**：原型照上一轮的写法，R_prev 的引用集合数不全就整条不判；V1 的条件也读 R_prev 的引用集合，这一半措辞没写。我的四段里没出现数不全（`skipped-incomplete` 一次都没有）。

## F3（顺带，不是主攻面）

I-3.5 ①（冻结副本那份文件第 137 行，今天主工作区里挪到了第 146 行；`.claude/kb/invariants.md@I-3 空间记账 —— 类`：「① `Reach(b)` 在 txg 序上**连续无洞**」）我在同一个原型里按候选集数了一下：

- 回退目标还在候选集里时，被回退复活的落点引用集合有洞（`logs/bad3.log`：`BI rollback-kept/clean CS1:i35-1-contiguous evaluated=116 violations=8 first=盘 0 槽 50182 类 1 诞生代号 4：引用它的版本 txg [4, 7]`），池级 checker 同一张 0 红。
- 抬 F 越过回退目标之后同一批落点的引用集合又连续了（同一份日志 `rollback-raised/clean CS1:i35-1-contiguous ... violations=0`），这时轮到 ② 的下界红。
- 由记录施加回退、之后再回退的那一段里，65 548 个状态每个都有洞（`logs/run2-ni300-ow-ni1-ow_rbcrash3-remount-raise5__rb1_.log`：`crash:CS1:i35-1-contiguous evaluated=10881112 violations=131096 states_red=65548`，第一处 `引用它的版本 txg [5, 9, 10, 11, 12, 13]`）。

所以 ① 与「回退是一次向前发布」在合法镜像上确实对不上（真冲突），而且 ① 与 ② 的下界在同一段历史的不同时刻各红一次。按正文第五节「I-3.5 ① 的改法会让任一合法镜像红 ⇒ 不改」，任何改法都要在上面这三类镜像上过一遍；我没提改法。

## 没打中的形状

| 形状 | 取样 | 打的是哪几格 |
|---|---|---|
| 回退之后抬 F 越过回退目标，起点带多叶 inode 树 / 多单元文件 / 两样一起 | 3 段，后缀 [抬 F 到 5] 的全量崩溃状态各 131 103 个 | V1、V2（V3 只第二段） |
| 回退那次发布崩在根槽之前、恢复由记录施加出回退版本、写行、抬 F，之后再回退 | 1 段，后缀 [回退到第 2 新] 的全量崩溃状态 65 548 个 | V1、V2、V3，CS1 与 CS2 |
| 用户动作放开 | 没放开扫：后缀只取了抬 F 与再回退两种；起点里的步骤（几次覆盖写、建几个 inode、回退目标）只取了上表四种 | — |

每段的段数与估时：先在第一段上跑 2000 个状态（`logs/time-probe4.log`，四线程 10.9 秒），按它估四段 458 857 个状态约 40 分钟；实际 3 452 秒（四段 952、903、926、674 秒，`logs/progress.md`），比估的长四成。第二批两段（196 651 个状态）实际 1 570 秒。没给批设限时。

没取到的（交主 agent，下一轮可以去打）：

- 覆盖写、顺序写、建 1 个以上 inode 放在枚举的后缀里：单次覆盖写一次发布就是 16 777 228 个层 0 状态、建 1 个 inode 是 1 048 588 个（`logs/count-probe.log`），都超过约 10⁶ 的线，照定义那段不跑；它们只出现在不枚举的起点里。回退之后再写、再建 inode、再抬 F 的组合（`logs/count-x.log` 里 `[ni2,raiseTo5]` 那三段各 1 179 690 个、回退之后挂载再抬 F 那几段 2 162 730 个、抬 F 之后再挂载 4 325 418 个）同样超线没跑。
- 被崩溃恢复抛弃的根与回退同在一段历史里；设备失而复得；根槽读坏的中间根；journal 环绕回把由记录施加出来的那一版的记录盖掉（那时 CS2 里那一版消失、候选集出现一个洞）。上面 F1「为什么造不出」那一段的推理正好落在这几处，都没量。
- 上一轮两批随机历史（`m3-inv-b-r1-opus-model/` 的 `rollback48`、`broad48`）没在这一轮重跑：它们每步只看做完之后的镜像，按这一轮的定义每段历史要把崩溃状态枚举全，30 步的历史一段就远超 10⁶，照定义不跑；所以正文 F1 要的「上一轮两份红的镜像归类」我没对着那两份产物做，归类用的是这一轮四段历史里的红。

## 这条腿自己的限度

- 状态账：四段第一批 458 857 个、两段重跑 196 651 个、估时与看终态的探针 8 019 个（`time-probe1..4` 各 2 000 个，只判终态的 19 个），合计 663 527 个，在约 10⁷ 的线以内；前任一个都没跑。
- `time-probe2` 那一次在每个状态上再做一次可写挂载、在挂载之后的镜像上再判（`BF_MOUNT=1`），2 000 个状态 370 秒，比不挂载慢二十多倍，正式跑没开；所以崩溃之后「恢复、写行、暖机」那一张镜像我只在起点里构造的那一段（由记录施加回退、写行）上判过，没在每个崩溃状态之后都判。
- 估时偏了：按前 2 000 个状态估四段 40 分钟，实际 57.5 分钟，超过定义说的 40 分钟线，我没为此再缩段数。
- 坏镜像装置有一半没造干净（分配记录树中间节点、改小之后中央映射 key 乱序，池级 checker 红 I-1.1 / I-2.1），那几份没用；用了的每一份都贴了池级 checker 0 红。坏镜像都只在三张起点上造，起点之外的形状没造。
- 冻结副本里 I-8.11、I-1.12、I-4.9 等上一轮新立的几条还没实现，池级 checker「0 红」是今天这份 checker 的 0 红。
- V2、V3（以及推出来的 V2s、V3s）只在我的模型上量过、被攻过零轮；表里标「推的」那几格没实现没跑。
- 原型对每个待判落点各走一遍继承子树（V3 还读整棵子树的头），没缓存，耗时只在总挂钟里体现。
- 「没打中」都只抽了一次样。

## 什么现象会推翻这些结论

- F1「V1 在合法镜像上 0 红」：任一合法历史（含崩溃状态）上出现一个落点，R_first 不满足 V1 的条件、而诞生代号 ≤ R_prev.txg。最该找的是候选集出现洞的地方：journal 环绕回盖掉由记录施加的那一版、被抛弃的根、设备失而复得。
- F2 漏红 ①、②：在入库装置上把这几份坏镜像重造一遍，若池级 checker 有别的不变量跟着红（比如 I-8.11、I-1.12 落地之后），「池级 checker 0 红而 V1 不红」就不成立，漏红变成「别处已经红」。
- F2 漏红 ③「判别子观测不到」：回退的实现写了谱系记录、或盘上另有字段记下「这条根是回退」，这一格就可分。
- F2「V1 有只红它的坏镜像」：第 ④ 行那几份在入库装置上有别的不变量跟着红。
- F3「真冲突」：回退的定案改成重写一份用户可见树（不再重新引用旧单元）。

## 没做什么

- 不判正推那几格，不替主 agent 采纳；副本上的数不算入库装置上的数。
- 没跑 checker 档包的任何测试、门禁、QEMU、变异；没改仓里 `crates/` 一个字，改动都在草稿目录的副本上，以 `proto.diff` 入库。
- 没读禁读清单里的文件；没读正推腿的报告。
- 没提 I-3.5 ① 的改法；没造正文 F2 要的「只红 V1 的坏镜像」的入库版本（第 ④ 行是原型上的）。
- 草稿目录 `/tmp/claude-1000/m3-birth-floor-r1-opus/` 里留着日志与进度；仓副本与编译目录交回前删掉，删了哪些见下一节。

## 删了哪些

- `/tmp/claude-1000/m3-birth-floor-r1-opus/tree`（前任拷的仓副本、我接着用的那一份，连同里面的编译目录 `target/`）：14G，已删。
- `/tmp/claude-1000/m3-birth-floor-r1-opus/rerun-check/tree`（核复跑脚本时新拷的仓副本与它的编译目录）：787M，已删；`rerun-check/` 里剩下的一份日志（52K）随草稿目录一起删了。
- 草稿目录里留着日志、进度与几个小脚本（几百 K），不是仓副本也不是编译目录。

## 引文核对

`python3 research/scripts/cite-check.py research/prompts/m3-birth-floor-r1-opus-output.md` 末行（另带 `--root /tmp/claude-1000/m3-batch4-freeze/tree --root /home/fy5090/code/singlefs` 跑一遍，末行相同）：

```text
  ✓ 核了 12 处引文，对上 12 处，没判 0 处（逐处列在上面）
```
