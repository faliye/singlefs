# m3-w3-r2 攻方腿（Opus）报告：K2、K3（2026-10-08）

轮名 m3-w3-r2；背景材料 `research/prompts/_m3-w3-r2-background.md`（正文 `research/prompts/_m3-w3-r2-body.md`）；前几轮判决 `research/prompts/m3-w3-r1-main-verification.md`。
冻结树 `~/.local/share/singlefs/work/m3-w3-r1/tree/`（只读，拷进草稿目录的副本上编）；草稿 `~/.local/share/singlefs/work/m3-w3-r2-opus/`；模型 `research/prompts/m3-w3-r2-opus-model/`。
这一份里的数全是仓副本装置（原型 crate 只加在冻结树的副本里，`crates/` 没动）或抽象模型上量的，不是入库装置上的数；我提的定义与改法只在我的模型与副本上量过、被攻过零轮。

## 复跑命令与模型文件

在仓根，`W` 取一个空目录；每段一条前台命令，编译与跑都经 `scripts/hosts/run-with-memory-cap.sh`（编译 6G、跑 4G），线程 6：

```
M=research/prompts/m3-w3-r2-opus-model; W=<空目录>
for p in build-proto build-model model-main-a model-main-b model-rest probe compare-quick; do SINGLEFS_REPO=$PWD bash $M/rerun.sh $W $p; done
# 长段（层 0 自己的分片，二十片合计约 35 分钟，每片 80–130 秒）：shard-0 … shard-19，再 merge，再 compare（多比 expected/ab-merge-of-20.out）
```

交回前在空目录 `~/.local/share/singlefs/work/m3-w3-r2-opus/rerun-empty/` 从头跑了 build-proto、build-model、model-main-a、model-main-b、model-rest、probe 与 shard-7，`compare-quick` 末行 `compared=19 status=0`；shard-7 的账本 205 行切片行与草稿里那一次逐字节相同（`diff` 退 0）。二十片全量是在草稿目录里用同一份源码跑的，`expected/ab-merge-of-20.out` 就是那一次 merge 的输出；复跑目录里没把二十片重跑完。`compare` 去掉 `elapsed_s` 字段、只比 `W3R2` 开头的行、排序后比。

模型目录每个文件的 sha256（`find . -type f | LC_ALL=C sort | xargs sha256sum`）：

```
cf1fdba286364d619a6af81723cb3ab4b020c34acfbd93f3178787114ba97a08  ./expected/ab-merge-of-20.out
b6303915ec30941dd2d0b26f4f0b58c9bef010607b68fac64b28d64be72a2513  ./expected/model-main-0.out
24b76c4998c68325541460b5a7d5689fffdfd5fba6a5240a98a757b23f73e6b7  ./expected/model-main-1.out
cd9315416f8b23054b6ebc085efdc25bee166f5c45304a3a0840c4bc3bd7a02e  ./expected/model-main-2.out
10442935de0292fa7f3ab1de212b024d4a456e532790786a5f297450a8099690  ./expected/model-main-3.out
80d985b54d344b4e9e5388d4ef4c379121790c9e1b2e834dedfc0524d3baf1ee  ./expected/model-mutation-0.out
d207aaeb4d5ac6504ac71ec42055cffca050ae6e572e0a427bb424c80d034979  ./expected/model-mutation-1.out
b689cd420fbb9574ae2a5e508a41746369bdd12b2c606689c7d42833bcb1adb5  ./expected/model-mutation-10.out
6ca5f9d9e2b814a48964afdd8c03568806dd14b42cb8233aa848302ff452255f  ./expected/model-mutation-11.out
7a58a254383ca90b6a5da6b3872c42eb16eafde2acdd2ac6de565dee0eb0169a  ./expected/model-mutation-2.out
dcc8442fe5d0fef96cfe87b7f1786574d323845b58358d707985d648b51cae01  ./expected/model-mutation-3.out
f5917f7f4179733d203783ca46d9d9ba9c2c48b2a4ba4e7378979910f1cc9a92  ./expected/model-mutation-4.out
d3988795c2b5e4db7eaa3e6ea3c185230d71b55596b417fd3228b084e1931c71  ./expected/model-mutation-5.out
7980580b6014b44944f2fd7f9f3b98a12332022b8a4a8e70618e009542a850f6  ./expected/model-mutation-6.out
5f59451d5dd3dd9c5c94864729cda0bb73561daba12b4e5552a5d91baba5431e  ./expected/model-mutation-7.out
43e79c8e353832c64fe3e9964e2c815cef1ff77c4ccf340af34d56bdbadcb8fa  ./expected/model-mutation-8.out
280f950574fa0add96b17bb75d31291883020f38870806348b750943317cf5c4  ./expected/model-mutation-9.out
7364aa934e291d8a85e2df5c79583d0c7888fbe52b7344bebc75870fcea8dfed  ./expected/model-ring6-0.out
9f665f15fda05de41b71d540c1f012cd8ce4c746bfaa917cc362490dfe34c458  ./expected/model-ring6-1.out
e3f0c9ff52d55691e3f9a99d629d9c00bf43b0ee0e79581185c9489713867229  ./expected/probe-AB.out
e53803d6ab1da215cfd5c86489898e832541944dda8bececbe04c53b3fe428dc  ./proto/Cargo.toml
68db5fa8f9b2d8fb568a6705ae1531653d46b1dbe350da3a9dfe61e8dd447810  ./proto/src/main.rs
ce3b5bb7e16867578c59d4b740af2d37d3252a1a8f173261973a9b572a97d365  ./proto/src/shadow.rs
d2bb6d5e66575e69949f3efa998207580fa61c5826d249a8fa439f2cf5a5120f  ./rerun.sh
f1861d55f68bc5dc61e51671c83606433c8dc040b488d9396269367c0233c507  ./timeline-model/Cargo.toml
1232ca55ff05c8938b8eeae5b53b3d8cbdc4739c66fe9fcbec216b47baf7edad  ./timeline-model/src/main.rs
```

两套装置：

- **原型（仓副本装置）**：`proto/` 是加进冻结树副本工作区的一个 crate（`crates/w3-r2-attack`），流在原型里自己造：mkfs → 取号 → 暖机 → 第一个文件 (1, 3) → P：实例 1 一次发布建 2 个 inode（2@4、3@4，水位 2 → 4）。盘用 `HistoryDeviceWidth::UnitAreaOf256Slots` 那一档两块内存稀疏盘（理由见第九节「取样」）。崩溃状态照层 0 的枚举域取：`enumerate_layer0_against_the_version_table_or_one_shard`，每一段都展开（`full_expansion`），P 那条流段序列 `[16, 2, 1, 2]`、闭式 65543、连原地覆写的撕裂态 65548 个状态。每个状态在工作线程上跑两段历史：
  - **A（只有一次崩溃）**：可写挂载 → `c`（建 1 个）→ `1`（回退到环里除现行之外最新的那条根）→ `1`（再回退一次，即 A → B → A）。
  - **B（崩溃 + 一次读故障）**：可写挂载那一次 P 写出的根槽与 P 写的单元整次读回全 0（与 `crates/singlefs-harness/tests/common/mod.rs` 那个造被抛弃根的辅助函数同一类故障，见第二节），挂载之后原样读得出 → `c` → `1`。
  每一步之后：池级 checker（`check_pool_image`）原样判；各候选按影子历史判。影子历史只按「这一版接在哪一版后面」带，「接在哪一版后面」取实现真走的那一版（挂载交回的 `effective_root`、会话的 `current`、回退交回的新版本），水位、树、出生代都读实现真写出来的版本。
- **抽象时间线模型**（`timeline-model/`，不依赖 `crates/`）：从第一轮攻方腿的模型改写，条文走法（发号、删、回退 1–3、抬 F、空发布、挂载、崩溃恢复抛弃最新那条根、建到一半崩溃、切换重做、切换报错、清扫阈值 0 / 从不）照抄；回退水位取 max(cur, 环里全部读得出的根)，与产品代码同一个取法（第二节第 1 条引的两行）；起点固定在新池新建文件之后，用户步全部穷举（长 7，环 4 槽；另有环 6、长 6）。候选换成本轮的（第一节）；另有 V1 臂（回退取目标那一版的水位）与六种变异（第五节）。

## 各格判定一览

| 格 | 形态 | 判定 |
|---|---|---|
| K2 崩溃恢复抛弃建 inode 的发布再建 | C2aL 最强形（第一节） | **没打中**：原型 A 全层 0 域 65548 个状态 × `c11`，262192 张镜像 0 误报；其中 131072 张带跨被抛弃发布的合法重发（第一次造出来）。B 196629 张 0 误报 |
| 同上 | C3rL 最强形 | **没打中**：同上两组 0 误报 |
| 同上 | 字面 C2a（只在建 inode 的发布留台阶）配 I-9.30 草案 ③ | **打中误报**：B 的 4 个崩溃状态（根槽持久、系统配置没轮换）上，回退把水位从环里被抛弃的根抬上去，③ 红；模型 163700 张（环 4、长 7） |
| 同上 | C3r 配 I-9.30 ③（「记录盖住水位」） | **打中误报**：同上 4 个状态、模型 163700 张 |
| 同上 | 谱系那一对取回退之后的水位（T_cur, H_new） | **打中误报**：原型 B 4 个、模型 231204 张 |
| K2 清扫删实例表行 | 两个最强形 | **没打中**（模型阈值 0，2584263 次清扫，0 误报 0 漏）；历史住在根下，清扫碰不到 |
| K2 实例切换 | 两个最强形 | **没打中**（模型，切换重做 / 报错 1946516 次，0 误报 0 漏）；`crates/` 今天没有切换写行的路径，只能按条文判 |
| K2 管理员回退两次 A → B → A | 两个最强形 | **没打中**（原型 A 每个状态都走了 A → B → A；模型 581782 次回到先前的 cur，0 误报 0 漏） |
| 同上 | 谱系那一对只对「只在建 inode 时留的台阶」比（C2aL-create） | **打中误报**：模型 56624 张，最短 `[create,crash-hide,rollback1,rollback1]` |
| K2 每次发布多写 | 两个最强形 | 第四节表（算术，推的） |
| K3 I-9.30 草案 + 谱系核对 | 谱系那一对用「被历史盖住」比（C2ay 的读法） | **打中**：变异「谱系那一对取 R_old 的」「谱系那一对写 0」在 252448 / 593287 张被改过的镜像上一张都不红；改成「等于」才全红 |
| 同上 | 两个最强形 | **打中（结构，推的 + 模型量过）**：谱系那一对若在发布路径里从「上一版」现算（`crates` 今天的回退把 `previous` 交给发布路径），那一行「回退取哪几种树」的单点变异让历史与谱系一起取 R_old，两个最强形在 V1 臂上漏全部 26410 次重发 |
| 同上 | 两个最强形 | **打中（弱）**：变异「新实例的历史从被抛弃的那条根带」在 61394 / 71808 张被改过的镜像上不红；不漏真重发，只在之后的合法跨抛弃重发上误报 |
| 同上 | I-9.30 草案配 C2a-all | 没打中「台阶记成发布之前的水位」：第一次建就红 |

一句话：C2a、C3r 各加谱系核对，在崩溃恢复抛弃、清扫、切换、A → B → A 上都站得住，但要三处收严才站得住：① 台阶要在**每次水位变了的发布**都留（不只建 inode 的那次），因为管理员回退会把水位抬过环里被抛弃根发过的号；C3r 不能用「记录盖住水位」那一句；② 谱系带的值要是**回退之前 cur 的**（H_cur 或 |log(cur)|），不是回退那次发布的；③ 核对要判**等于**，不是「被盖住」。三处都只在我的模型与副本上量过、被攻过零轮。

## 一、候选的定义（我给的最强形与被打中的几种读法）

前提（今天的实现）：

- 回退那次 txg = cur 的 txg + 1：「let rollback_txg = current」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:7624`）后面接「.checked_add(1)」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:7628`）。所以谱系记录不必再存 T_cur：T_cur = T_rb − 1。
- 谱系树随现行那一版：「lineage_containers: current.lineage_containers.clone(),」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:7563`）；条文「一棵专用谱系树，day-1 零根注册，随现行那一版、只增不删」（`.claude/kb/decisions/18-块里携带什么信息.md:576`）。
- 回退取目标的只有三种树：「|| entry.kind == TREE_KIND_DELETED_INODES」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:7512`，前两种是 extent、inode），记账行取现行的、只换水位那一格：「value: if entry.statistic == STATISTIC_INODE_WATERMARK {」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:7530`）。

| 名字 | 历史写什么、谁写 | 谱系那一份 | 单镜像上怎么判红（最新那条根） |
|---|---|---|---|
| **C2aL 最强形**（我给的） | 记账水位那一统计量留台阶行 (g, H)：**每次水位变了的发布**留一行（建 inode 的发布、回退跳升那次），随根 COW、回退从 cur 带 | 谱系记录 (T_rb, T_old) 多带 H_cur = cur 的水位（8 字节） | I-9.30 ①②③ 照第一轮正推的草案逐字（行取上面的台阶），加：每条谱系记录 max{H : g ≤ T_rb − 1} **等于** H_cur，H_cur = 0 也红 |
| **C3rL 最强形**（我给的） | 根下只追加的 (号, 出生代)，随根 COW、回退从 cur 带 | 谱系记录多带 count = cur 那一版发号记录的条数 | 同号两个出生代；记录里有号 ≥ 水位；最新那版树里每条都在记录里（C3rs）；每条谱系记录 #{b ≤ T_rb − 1} **等于** count，count = 0 也红 |
| C2a 字面 | 只在建 inode 的发布留台阶（正文 K2「记账按每次建 inode 的发布留完整台阶」） | — | I-9.30 ①②③ |
| C3r:I930-3 | C3r 配草案 ③ 的 C3 读法：最大号 + 1 ≥ 水位 | — | 水位大于初值而 max(号) + 1 < 水位 |
| 谱系对 (T_cur, H_new) | — | 带回退那次发布之后的水位（发布路径里的「上一版」水位就是它） | 历史在 T_cur 那一刻盖不住 H_new |
| 谱系对 cover | — | 同最强形 | 只判「被盖住」：max{H : g ≤ T_cur} ≥ H_cur（第一轮 C2ay 的读法）；C3rL 同理 #{…} ≥ count |

真相（模型与原型都同一个）：这条时间线上发过的 (号, 出生代) 集合，按「这一版接在哪一版后面」带，回退那次并上 R_old 的树；同号两个出生代就是同一条时间线上的重发，镜像不合法。合法镜像上红叫误报；重发那一步或「被重发的对象还活着」的镜像上不红叫漏。

## 二、K2：崩溃恢复抛弃一次建 inode 的发布再建（造出来了）

### 1. 机理：管理员回退的水位把环里被抛弃的根也算进去

回退那次的水位读环里全部读得出的根，不按实例表滤：「let ring_roots = readable_roots(」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:7738`），「let inode_number_watermark = highest_inode_number_watermark_in_the_ring(」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:7748`）；`readable_roots` 的注释是「根环里全部自证过的根，每个可读根槽一条（回退候选集与影子账从这里取）。」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/recovery.rs:1374`）。条文也这么写：「| 水位 | inode 号水位与树 ID 水位 = max(cur 那一版内存里的水位, 环里读得出的根的水位)，不跟着 R_old 退回去 |」（`.claude/kb/decisions/23-journal的角色与格式.md:385`）。

所以环里一条被崩溃恢复抛弃、水位更高的根还读得出时，回退会把水位抬过那条被抛弃时间线发过、而这条时间线上从没发过的号。抬上去的那一段号在这条时间线上**没有任何发号记录**。正推腿草案的 ③「历史盖住最新根」与 C3r 的「最大号 + 1 ≥ 水位」都假设水位只由这条时间线上的发号推高，这一步上两者都红，而镜像是合法的（没有一个号被重发）。

模型里这是唯一的跳升来源：环 4、长 7 两组 P 臂，`rollback_jumps_from_a_root_on_the_timeline=0`，`rollback_jumps_from_abandoned_roots_only` 55040 / 46545（`expected/model-main-0.out`、`-1.out` 第 1 行）。原型 B 里跳升 4 次，全在 P 被抛弃、P 的根事后读得出的那 4 个状态上（下面第 3 条）。

### 2. 原型 A：只有一次崩溃（层 0 全域）

P 那次发布的 65548 个崩溃状态，每个都挂得上（`w3a_mount_ok=65548`）；65536 个状态里 P 被抛弃（P 的记录没写全，恢复落到 (1, 3)，实例表行 (1, 3)）、其余 12 个里 P 的记录写全被施加（落到 (1, 4)）。被抛弃的那 65536 个状态里，新实例再建就重发 2 号（2@6 对 P 写过的 2@4）：`w3a_cross_abandoned_reissue_images=131072`（`c` 那一步与 A → B → A 回到它那一步各一张）。E175 的 26 份汇总里这一形 0 次（`sum_q3b` 全 0），这里是第一次在实现上造出来。

`expected/ab-merge-of-20.out` 第 6–13 行（去掉每行开头相同的配置串）：

```
W3R2LAYER0 states=65548 observed_states=65548 violations=0 root_persisted_states=9 failed_states=0 checker_violated={} first_violation=None
W3R2COUNT w3a_cross_abandoned_reissue_images=131072
W3R2COUNT w3a_images=262192
W3R2COUNT w3a_legal_images=262192
W3R2COUNT w3a_mount_ok=65548
W3R2COUNT w3a_p_abandoned_states=65536
W3R2COUNT w3a_rollbacks=131096
W3R2COUNT w3a_states=65548
```

262192 张镜像全合法，没有一行 `w3a_fp_*`（每个候选误报 0），也没有一行 `w3a_real_red_*`（池级 checker 一条违例都没有，`grep -c real_red` 输出 0）。没有被抛弃的**根**：P 的根槽写在记录之后、中间隔屏障（段序列 `[16, 2, 1, 2]` 里根自成一段），根持久时记录已持久、恢复就施加它；所以纯崩溃抛弃的发布在环里只留单元孤儿与写了一半的记录，不留根，回退的跳升不会出现。两个最强形在这里没打中的理由与正推第一轮说的一样：历史住在根下，新实例接在所选那一版后面（原型里读的是挂载交回的 `effective_root`）。

### 3. 原型 B：崩溃 + 下一次挂载读回全 0 一次（打中）

故障：P 的根槽 FUA 持久、系统配置轮换没持久（层 0 域里 root 持久、两块盘的轮换没写或写撕裂的那几个状态），下一次可写挂载整次读 P 的根槽与 P 写的单元都读回全 0，挂载之后照常读得出。这是仓里已有的造被抛弃根的那一类故障：「它那次发布之后轮换写的系统配置槽（每块盘一槽）坏掉、系统配置没见证到它」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-harness/tests/common/mod.rs:743`）；执行器那一步造不出，是因为「读阶段重读一次仍读不出、取号之前拒可写（`MountError::NewerStateStillUnreadableAfterOneReread`），这一步因此造不出被抛弃的根。」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-harness/src/history.rs:4533`）——那是系统配置见证过的一形；没见证过时可写挂载照常做成。原型里系统配置见证到 P 的 5 个状态照样被拒（`w3b_mount_refused_newerstatestillunreadableafteronereread=5`），没见证的 4 个状态挂上了、P 的根事后读得出且按实例表被抛弃（`w3b_p_abandoned_with_readable_root_states=4`）。

那 4 个状态上 `c1`：建 2@8（水位 3），回退到环里除现行之外最新的那条根，实现算出的水位是 4（环里被抛弃的 (1, 4) 带 4）。`expected/probe-AB.out` 第 128、129 行（状态 524287：P 的 16 个单元写、两份记录、根都持久，两块盘的轮换都没持久）：

```
W3R2TRACE state=524287 effective=(1, 3) script=c1 key=(2, 9) wm=4 tree=[(1, 3)] legal=true cross=false rows_create=[(3, 2), (8, 3)] rows_all=[(3, 2), (8, 3), (9, 4)] log=[(1, 3), (2, 8)] lineage=[(9, 7, 8, 3, 2, 4)] red=[C2a-create:I930-3,C3r:I930-3,C2a-create:I930-123,C2aL-all:pair(Tcur,Hnew)] real=[]
W3R2TRACE state=524287 effective=(1, 3) script=1 key=(2, 8) wm=4 tree=[(1, 3)] legal=true cross=false rows_create=[(3, 2)] rows_all=[(3, 2), (8, 4)] log=[(1, 3)] lineage=[(8, 6, 7, 2, 1, 4)] red=[C2a-create:I930-3,C3r:I930-3,C2a-create:I930-123,C2aL-all:pair(Tcur,Hnew)] real=[]
```

`lineage` 一项是 (T_rb, T_old, T_cur, H_cur, count, H_new)。合法（号 2 只在这条时间线上发过一次，3 从没发过），池级 checker 不红（`real=[]`），红的是 ③ 那两种读法与 (T_cur, H_new) 那一对。全域合计（`expected/ab-merge-of-20.out` 第 15–20 行）`w3b_fp_c2a_create_i930_123=4`、`w3b_fp_c2a_create_i930_3=4`、`w3b_fp_c2al_all_pair_tcur_hnew_=4`、`w3b_fp_c3r_i930_3=4`，`w3b_images=196629`、`w3b_legal_images=196629`；两个最强形 0。

模型最短反例 `[create,crash-hide,rollback1]`（`expected/model-main-0.out` 第 16 行），第 20 行逐步打印的最后一步：

```
W3R2TRACE arm=Product mutation=None ring=4 sweep_threshold=never longest=7 candidate=I930/C2a-create kind=false_positive step=2 op=rollback1 newest=2:6 wm=3 F=0 tree=[1@3] table=[1:3] rows_create=[3:2] rows_all=[3:2,6:3] log=[1@3] lineage=[6<-3|cur5:H2:n1:Hnew3] ring=[1:4:wm3,2:5:wm2,2:6:wm3,1:3:wm2] issued_reissue=0 jump_from_abandoned=1 red=[I930/C2a-create,C2aL-all:pair(Tcur,Hnew),C3r:I930-3]
```

改法（推的，最强形就是这么定的）：C2a 的台阶在**每次水位变了的发布**留（回退那次跳升也留一行），`rows_all` 那一列 0 误报；C3r 不用「记录盖住水位」那一句，改用「树里每条都在记录里」（C3rs）；谱系带 cur 的值，不带回退那次的。

## 三、K2：清扫删实例表行、实例切换、管理员回退两次（A → B → A）

模型每一组的规模与这三种步出现的次数（`expected/model-main-0.out`、`-1.out` 第 1 行，原样）：

```
W3R2MODEL arm=Product mutation=None ring=4 sweep_threshold=never longest=7 full_length_histories=9863012 images=10836269 legal_images=10836269 events=0 residual_images=0 reissues_across_abandoned_roots=48356 rollback_jumps_from_abandoned_roots_only=55040 rollback_jumps_from_a_root_on_the_timeline=0 sweeps=0 switches=1946516 rollbacks_back_to_a_former_cur=581782
W3R2MODEL arm=Product mutation=None ring=4 sweep_threshold=0 longest=7 full_length_histories=9985685 images=10964563 legal_images=10964563 events=0 residual_images=0 reissues_across_abandoned_roots=35042 rollback_jumps_from_abandoned_roots_only=46545 rollback_jumps_from_a_root_on_the_timeline=0 sweeps=2584263 switches=1957758 rollbacks_back_to_a_former_cur=520301
```

两组 P 臂各候选的误报（同两份文件 `W3R2COUNT` 行，`false_positive` 一列；后两列是其中「这一段历史里有过从被抛弃根的跳升」与「有过清扫、切换或回到先前 cur 的回退」的）：

| 候选 | 阈值「从不」 | 阈值 0 | 有跳升的 | 有清扫 / 切换 / A → B → A 的（阈值 0） |
|---|---|---|---|---|
| C2aL 最强形 | 0 | 0 | 0 | 0 |
| C3rL 最强形 | 0 | 0 | 0 | 0 |
| I930/C2a-create（字面） | 163700 | 152330 | 163700 / 152330 | 144943 |
| C3r:I930-3 | 163700 | 152330 | 同左 | 144943 |
| C2aL-create:pair-cover | 56624 | 57302 | 56624 / 57302 | 54796 |
| C2aL-all:pair(Tcur,Hnew) | 231204 | 218538 | 231204 / 218538 | 207915 |
| I-9.29（今天的） | 0 | 0 | 0 | 0 |

**每一个误报都在跳升之后**（「有跳升的」那一列等于总数）。清扫、切换、A → B → A 那一列不是独立的打中：它们与跳升同在一段历史里，单独出现时一次都不红（把跳升那一列减掉为 0）。所以这三种形态本身没打中，打中的仍是第二节那一处。

- **清扫**：条文与实现都只在「环里没有该实例的根」时删行：「判根环每个槽都自证过、环里没有该实例的根」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/instance_table_sweep.rs:5`），C2a、C3r 的历史都住在根下，删行碰不到它；模型阈值 0 的 2584263 次清扫上两个最强形 0 误报 0 漏。按实例表滤抛弃的只有 I-9.29 与第一轮的 C2ax / C3rx 那种交叉核，最强形里没有这一判。
- **实例切换**：`crates/` 今天没有切换写行的路径（第一轮正推第 2.3 节已查），只能按条文在模型里走：切换重新读盘择根（就是 cur）、写行、重做或报错。两个最强形在 1946516 / 1957758 次切换上 0 误报，V1 臂 0 漏。
- **A → B → A**：原型 A 每个崩溃状态都走了一遍（`c` 之后连回退两次，第二次回到第一次回退之前的那条根，`w3a_rollbacks=131096` = 65548 × 2），0 误报；模型 581782 / 520301 次「回退目标是先前某次回退时的 cur」，最强形 0 误报。打中的只有 C2aL-create:pair-cover（谱系那一对去比只在建 inode 时留的台阶）：最短 `[create,crash-hide,rollback1,rollback1]`（`expected/model-main-0.out` 的 `W3R2EXAMPLE … candidate=C2aL-create:pair-cover kind=false_positive` 那一行），第一次回退跳升之后 cur 的水位不在只记建 inode 的台阶里，第二次回退把它当 H_cur 记下，比不上就红。它与第二节同一个根子。
- V1 臂（回退取目标那一版的水位）上两个最强形漏 0：环 4、长 7 两组 340776 / 329574 次重发、443952 / 446101 张「被重发的对象还活着」的镜像，`event_missed=0 residual_missed=0`（`expected/model-main-2.out`、`-3.out` 的 C2aL-strongest、C3rL-strongest 两行）；环 6、长 6 同（`expected/model-ring6-1.out`）。

## 四、K2：每个形态每次发布多写的字节、单元与写（算术，推的）

数据来源：记账条目「pub const ACCOUNTING_ENTRY_BYTES: u64 = 34;」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-format/src/lib.rs:145`），记账树节点头「pub const ACCOUNTING_TREE_INDEX_NODE_HEADER_BYTES: u64 = 159;」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-format/src/lib.rs:71`），今天一代「两盘 15 行——带设备维的六项每盘一行、池级三行」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/transaction.rs:8019`），谱系记录「pub const LINEAGE_RECORD_BYTES: u64 = 16;」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-format/src/lib.rs:119`）。闭式 `1 + Σ(2^|段| − 1)`：原型 P 那条流 `[16, 2, 1, 2]` 是 65543（原型实测输出），多 2 个写是 262151，多 4 个写是 1048583（`python3` 算的）。

| 形态 | 哪种发布多写 | 多写的字节 | 多写的单元（两份副本各一写） | 崩溃状态（单元段） |
|---|---|---|---|---|
| C2aL：台阶行 | 水位变了的发布（建 inode、回退跳升） | 34 / 次 | 台阶行 ≤ 462 条时 0（一片记账叶装 ⌊(16384 − 159) ÷ 34⌋ = 477 条，今天一代 15 条，余 462）；超过之后记账树长一层，每次发布多重写 1 个根、可能多 1 片叶：+1–2 个 16 KiB 单元，+2–4 写 | ≤ 462 条时不变；之后单元段 ×4 到 ×16 |
| C2aL：谱系多带 H_cur | 只有回退那次 | 8 / 次（记录 16 → 24 字节，一个容器 2039 → ⌊(32768 − 136) ÷ 24⌋ = 1359 条） | 0（回退那次本来就重写谱系容器） | 不变 |
| C3rL：发号记录 | 只有建 inode 的发布 | 每个新 inode 16（号 8 + 出生代 8），或每次发布一条区间记录 24 | +1 个码 3 容器（32 KiB）+ 1 个码 2 根（根恒为码 2，与谱系树、墓碑树同形）= +2 个单元、+4 写；树表条目 11 → 12（树表单元本来每次发布都写，0 单元）；journal 点名 +2 项 | 单元段 ×16（P 那条流 65543 → 1048583，越过「约 10⁶」那一线） |
| C3rL：谱系多带 count | 只有回退那次 | 8 / 次 | 0 | 不变 |

C2aL 另有一笔不是字节的代价：I-3.13「记账行代段等于这一版的 txg」（`.claude/kb/invariants.md:155`）与 D5 已定项 2「每条根的记账树只装这条根那一代的行」（`.claude/kb/decisions/05-快照-空间记账机制.md:47`）都要为台阶行开例外（第一轮正推已提，这里只确认两处都还在）。C3rL 的「+4 写」若改成把发号记录挂在记账树里（同 C2aL 的载体），就退回 C2aL 那一行。

## 五、K3：I-9.30 草案配上 K2 的形态，放过哪条变异

被判的文字是第一轮正推给的草案，开头一句「I-9.30　水位不低于时间线上发过的号。」（`research/prompts/m3-w3-r1-sonnet-output.md:163`），①②③ 照那一行逐字实现（`timeline-model/src/main.rs` 的 `floor_red`、`cut_red`、`cover_red`）；谱系核对按第一节的两种比法各实现一份。第一轮攻过的 A（回退那次历史取 R_old）、B（只照抄这一版）本身不再攻；下面六种是新的，都在模型里写进影子历史（阈值 0、长 6、环 4，每组 978878 张镜像），V1 臂那一组看漏掉的重发（共 26410 次）。数出自 `expected/model-mutation-0.out` … `-11.out` 的 `W3R2COUNT` 行，「不红 / 被改过」是历史被改过的镜像里不红的张数 / 全部张数：

| 变异 | 判词 | P 臂：不红 / 被改过 | V1 臂：漏掉的重发 |
|---|---|---|---|
| H 谱系那一对取 R_old 的（H_cur = R_old 的水位，count = R_old 的条数） | C2aL pair-cover / C3rL count-cover | 252448 / 252448；249848 / 249848 | 26410；26410 |
| 同上 | pair-eq / count-eq（最强形用的） | 0 / 252448；0 / 249848 | 0；0 |
| Z 谱系那一对写 0 | pair-cover / count-cover | 593287 / 593287；593287 / 593287 | 26410；26410 |
| 同上 | pair-eq / count-eq | 0；0 | 0；0 |
| R 回退跳升那一行不写（C2a-all） | C2aL 最强形 | 3332 / 16238 | 0 |
| D 台阶行记成发布之前的水位 | I930/C2a-all；C2aL 最强形 | 4151 / 766558；0 / 766558 | 0；0 |
| Mh 新实例的历史从被抛弃的那条根带 | C2aL 最强形；C3rL 最强形 | 61394 / 134326；71808 / 134326 | 0；0 |
| A+H 历史与谱系那一对都取 R_old（双变异） | 两个最强形 | 336805 / 336805；336805 / 336805 | 26410；26410 |

读法：

1. **打中：用「被盖住」比谱系那一对，放过 H 与 Z。** 第一轮 C2ay 与正推草案都没写谱系那一份自己写错了怎么办；「被盖住」（max{H : g ≤ T_cur} ≥ H_cur）在 H_cur 偏小、写 0 时恒成立，所以改坏谱系那一对一张都不红；再叠一次 V1（回退水位取目标那一版的），谱系那一判漏掉全部 26410 次重发（这时 ① 照样抓到，所以 C2aL 整体不漏，漏的是谱系那一判自己）。改成「等于」两种都全红。按正文 K3 的字面「把那份历史写错或不写，它必须红」，谱系那一对就是这次补丁新加的历史，「被盖住」那一读法不满足它。
2. **打中（结构，推的 + 模型量过）：谱系那一对只有从 cur 读才是独立旁证。** 今天回退把「上一版」交给发布路径：「version_to_publish_the_rollback_after(current, &target_version, inode_number_watermark);」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:7758`），谱系记录在同一处只填两样「rollback_target_txg: target_root.checkpoint_txg,」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:7802`）。实现的人若在发布路径里从这个「上一版」现算 H_cur 或 count（它就在手边），那么「回退取哪几种树」那一行（mount.rs:7510–7512）加一种的单点变异，会让 C3r 的记录与 count 一起取 R_old；同理把记账行从 `target_version` 取会让台阶与「上一版」的 max H 一起取 R_old——两者都是 A+H 那一行：两个最强形在 V1 臂上漏全部 26410 次。从「上一版」读 H_cur 的另一种结果是读到回退那次的水位 H_new（`inode_number_watermark` 那个变量），就是第二节的 (T_cur, H_new) 误报。所以定义里要写死「H_cur、count 从 cur（回退之前现行那一版）读，不从回退要接着发布的那一版读」。没写进 `crates/`、没在入库装置上跑。
3. **打中（弱）：Mh 改坏历史而不红。** 新实例的历史若从被抛弃的根带（例如历史放在根外、恢复时对环取 max，像树 ID 水位那样），两个最强形在 61394 / 71808 张被改过的镜像上不红；不漏真重发（V1 臂 0），代价是之后那一形合法的跨抛弃重发被判红（误报）。I-9.30 草案对这一形没有一句判得到它，靠的是「历史只住在根下、随所选根」这一条载体纪律。
4. 没打中：D 第一次建 inode 就红（③）；R 在 P 臂有 3332 张不红，但那几张里少的那一行被之后的建 inode 行盖住，V1 臂 0 漏，算不上放过。

## 六、每个打中先答四句

| 打中 | 分不分辨臂（候选） | 被判的系统当时看不看得到判别它的东西 | 满足的是判据字面的哪一个分句 | 跑前条款给的改法在这几格上 |
|---|---|---|---|---|
| ③ 读法在跳升之后误报（字面 C2a、C3r:I930-3） | 分辨：同一张镜像两个最强形不红（原型 B 第 128、129 行，模型同组 0） | 看得到：回退那次发布自己知道水位是从环里的哪条根取的，跳升那一行能当场写下；镜像上也有那条被抛弃的根（4 个状态里都读得出） | 正文第五节「攻方坐实一形漏报或误报」里的「误报」：合法镜像（同一条时间线上没有重发）判红 | 跑前条款只给「收严或出局」：收严成「每次水位变了都留行」在这几格上不再中（原型 B、模型 0）；C3r 只能拿掉 ③ 那一句 |
| (T_cur, H_new) 那一对误报 | 分辨：同一张镜像 (T_cur, H_cur) 不红 | 看得到：H_cur 就是 cur 的水位，回退时在手边 | 同上「误报」 | 收严成「带 cur 的值」不再中 |
| C2aL-create:pair-cover 在两次回退之后误报 | 分辨：C2aL 最强形不红 | 同第一行 | 同上 | 同第一行的收严一并修掉 |
| 「被盖住」放过 H、Z | 分辨：同一组镜像「等于」全红 | 看得到：两边的值都在最新那条根下 | 正文 K3「把那份历史写错或不写，它必须红」的「写错」（H）与「不写」（Z） | 改成「等于」不再中 |
| 从「上一版」读谱系那一对（A+H） | 不分辨：两个最强形一起出局——病在两样东西从同一个来源读，不在哪个候选 | 看不到：历史与旁证都取 R_old 之后，镜像上与「本来就这么回退」逐字段相同 | 是双变异；单点变异的说法依赖「谱系那一对在发布路径里从上一版现算」这一个实现假设（推的） | 改法不是判词，是写法纪律「从 cur 读」；判词怎么改都碰不到这一格 |
| Mh 不红 | 不分辨：两个最强形一起不红 | 看不到：多带的那几行在镜像上与真发过的行同形 | K3「写错」 | 改法是载体纪律（历史只住在根下、随所选根），判词碰不到 |

## 七、改法各修哪一格（都只在我的模型与副本上量过、被攻过零轮）

| 改法 | 修哪一格 | 标 | 依据 |
|---|---|---|---|
| C2a 台阶在每次水位变了的发布都留（含回退跳升那次） | ③ 跳升误报（C2a） | 量过 | 原型 B 4 个状态、模型两组 `I930/C2a-all` 误报 0（`expected/model-main-0.out`、`-1.out`） |
| C3r 拿掉「记录盖住水位」，用「树里每条都在记录里」 | ③ 跳升误报（C3r） | 量过 | 同上 `C3rs` 误报 0；V1 臂 `C3rL-strongest` 漏 0 |
| 谱系带 cur 的值（H_cur / count），不带回退那次的 | (T_cur, H_new) 误报 | 量过 | 原型 B、模型 `C2aL-all:pair-eq` 误报 0 |
| 谱系核对判「等于」，0 也红 | K3 H、Z | 量过 | `expected/model-mutation-0.out` … `-3.out` 的 pair-eq / count-eq 两行 |
| 写法纪律：H_cur、count 在回退入口从 cur 现算，不从交给发布路径的「上一版」算 | K3 A+H 那一格 | 推的 | 按 mount.rs 第 7758、7802 行的结构推；模型只量了「两样都取 R_old」时两个最强形都漏（`-10`、`-11`），没量「从 cur 算」的实现 |
| 载体纪律：历史只住在根下、随所选根（不对环取 max） | K3 Mh | 推的 | 模型 Mh 两组只证明判词碰不到它，没量别的载体 |
| 不写跳升那一行时，用 ③ 的地板改成「g ≤ txg 的行里 H 的最大值 ≥ 水位」并允许回退那次例外 | ③ 跳升误报 | 推的、不推荐 | 例外要知道哪次是回退，等于把跳升那一行换成另一种标记 |

## 八、没打中的形状

- **两个最强形在崩溃恢复抛弃上误报或漏报**：原型 A（层 0 全域 65548 个状态，每个都 `c11`，262192 张镜像，其中 131072 张有合法跨抛弃重发）、原型 B（65548 个状态 `c1`，196629 张）0 误报；模型 P 臂环 4、长 7 两组 21800832 张镜像、环 6 长 6 一组 1009096 张 0 误报，V1 臂三组 0 漏。扫的用户步：原型只有 `c11`、`c1` 两段（第九节「取样」）；模型是第一轮那 12 个字母全部穷举。
- **清扫、切换、A → B → A 本身造出误报或漏报**：没有（第三节，单独出现时 0）。
- **池级 checker 今天在这几形上误报**：原型 A、B 全部 458821 张镜像，`real_red` 0 行。
- **抬水位从环里没被抛弃的根来**：模型两组 `rollback_jumps_from_a_root_on_the_timeline=0`，跳升全部来自被抛弃的根；这是「回退时环里的 max 只对被抛弃根起作用」的观测，不是证明。
- **新实例第一个 txg 与被抛弃的发布撞号**：没造（第九节）。

## 九、这条腿自己的限度

- **取样**（照定义第 3b 条：崩溃状态不缩，缩历史与几何）：4 GiB 的盘上 P 那条流段序列 `[20, 2, 1, 2]`、1048588 个状态（原型 `segments` 输出），每段历史要挂载、发布、跑 checker，估全量要几天；换成 `UnitAreaOf256Slots` 那一档（单元区 256 槽、journal 环 6 MiB），段序列 `[16, 2, 1, 2]`、65548 个状态（384、240 两档同数）。用户步从「七个字母深 2 全部」（每状态 57 段）缩到 A 一段 `c11`、B 一段 `c1`。估时：层 0 枚举自己的判定 6 线程 104.4 秒（`W3_STATE_LIMIT=0` 那一次）；1/64、1/16 两次试跑 99.5、153.8 秒，按差值估观察者每状态约 18 毫秒（墙钟、6 线程），A+B 两段全量约 30 分钟，按层 0 自己的分片切成 20 批，每批估 85 秒；实跑 20 批 81.9–132.6 秒，合计 1955.1 秒（机器上同时有别的会话的 checker 档全量在跑，负载 20–40）。批不设限时；有两次（1/8 那一次试跑 484.5 秒、第 15+16 两批连跑）超过前台 240 秒被工具挪到后台，等它自己跑完，没停。
- **层 0 状态量**：这条腿起的全量枚举 9 次 × 65548（7 次试跑与探针、1 次 20 批合计、复跑目录的 probe 1 次）加复跑目录 shard-7 的 3280 个，合计 593212 个状态，在约 10⁷ 之内；单次 65548，在约 10⁶ 之内。
- 原型 B 的故障是我按仓里已有的造被抛弃根的辅助函数那一类写的（整次挂载读回全 0 之后原样读得出），落在 P 的根槽与 P 写的单元上；它是两故障（一次崩溃在根槽持久与轮换之间、一次挂载期读故障），不是单故障。
- 两个最强形都没写进 `crates/`：原型里是按实现真走的版本链算的影子，模型里是条文状态机；它们自己的写在崩溃点上怎么落（台阶行与谱系那一对和别的单元同段）没进层 0 枚举。
- 模型只有一个头、没有克隆；墓碑只在 I-9.29 那一判里按树算，②「谱系上的墓碑」没进影子（原型与模型都没有 unlink 之外的墓碑判）。
- 每格只抽了一次样（没有 s2）；「没打中」的格照 `.claude/rules/three-way-inference.md` 不算「没打中」。
- 副本上的数不是入库装置上的数；E175 的装置没有用。

## 十、没做什么

- 没跑任何重型测试（checker 档包的测试二进制、门禁、变异整表都没跑）；原型 crate 只依赖 checker 档的库，在自己的二进制里调层 0 枚举，测试目标名不带 layer0。
- 第一轮攻过的 A、B 两种变异本身、E175 V1 那两次、op92 / op95 没再攻；V2 的射程归辩方腿，没判。
- 新实例第一个 txg 取环里根与记录的最大值加一（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:1317`：「checkpoint_txg_after_the_version(highest_ring_txg.max(highest_record_txg))」）若被改坏、与被抛弃发布的出生代撞上，I-9.30 与两个最强形都看不见（(号, 出生代) 逐字相同）——这一形没造、没量，记在这里交主 agent。
- 复跑目录里只重跑了 20 批里的 shard-7（与草稿那一次账本逐字节相同），没把二十批全重跑。
- 开工时 `ps` 看到别的会话在跑 `cargo test --release --offline --no-fail-fast -p singlefs-checker-tier --lib --tests`（31 线程，pid 164800），没有性能测量在跑；我的编译与跑一律 `nice -n 19`、6 线程，没等锁。
- 草稿目录 `~/.local/share/singlefs/work/m3-w3-r2-opus/` 里的仓副本、编译目录与复跑目录（`repo/`、`target/`、`target-model/`、`rerun-empty/`）交回前删了；留着层 0 分片账本（`l0progress-ab/`，20 份）、全部原始输出（`out/`、`model-out/`）与分片脚本 `run-shard.sh`（共约 8 MB）：主 agent 要直接读 20 批的原始输出与账本时不必重跑约 35 分钟；不要了整个目录可删。这些输出没进 `research/results/`：本轮的产物按交付写进模型目录 `expected/`，草稿里的是它们的来源与中间试跑。
