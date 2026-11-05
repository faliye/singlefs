# m3-close-b19-checker-r1 正推腿（Sonnet）报告：池级 checker 一簇判据 K1–K10

日期：2026-10-29。立场：从条款与 `crates/` 今天的代码正推每格应当是什么；没读攻方腿与 `m3-close-b19-core-r1-*` 的任何产出。开工快照 `research/prompts/m3-close-b19-checker-r1-snapshot/` 里 walk.rs、mount.rs 与两份 WGSL 的 sha256 与主工作区此刻逐份相同（`sha256sum` 对过），下面引的行号既是快照的也是此刻主工作区的。

## 复跑与产物

模型在 `research/prompts/m3-close-b19-checker-r1-sonnet-model/`（纯 Python，标准库；不碰 cargo、不碰 GPU）。K5、K6 的数字都出自它。复跑命令（串行，约 7 分钟，峰值约 2.5 GiB）：

```
bash scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/m3-close-b19-checker-r1-sonnet-model/tools/rerun.sh --repo <仓根> --frozen research/prompts/m3-close-b19-checker-r1-snapshot --work <新的草稿目录>
```

每个文件的 sha256：

```
42f2d2fb9f49cdc5a80a2b9c52f35a796acd75ddb234690d70d093ebf8483285  lr5.py
15a8ae4e6fe95eaa74b01fd895bfb71d41411a9a73399351c387fea59971c2bd  lr5b.py
c8a4ee7849b63e0bab15d2e92410f20aabe641d755d95b0e0c6102176647ff7e  lr5c.py
3e4ee3725234e345c2bfa5c5f336c06aefdc1b5195b84a5e898c4938a0b6c702  rcw.py
7010bcfede0c62f6d0179fa9e336834c1a6fb4d6c3583bd5a95aaf877217c6c4  rcw_t.py
3a96f63fc1d7bbdf5ea61f1e43edcc3aa09fa7116d7a7f15a3d4029364118add  tools/rerun.sh
721df0e3edbfa71933fcee3ac8b08ed71fe72880fbe7c5c80e76e00e8ce1b343  out/lr5_baseline.out
f2ea3fef21a50a467e4a5c73a14cd41eae048cc3c51b269f5e62dc6d962b6583  out/lr5_recordonly.out
02e43740de4561d2a1c650eb0cc1f240bb7cb9bcca9788ee17583651e6844e38  out/lr5_sysunread.out
180d8061911768fb267b56c82913f4aedb5f5a63fa8715945d39f76a5470f3c5  out/lr5_broken.out
f75b9c3a8dc986318f7ac3e4dae68238faee2a5af886bf333212c43b95ccfd61  out/lr5_truth_baseline.out
3084ca73d2b7958da909a3acf7e86e154b6a78171409b7e4e449b7c2ae161938  out/lr5_truth_recordonly.out
c5c052f736792b323b96d653fa3fb1143bb0e362cef68ac92a757a8da3963af7  out/rcw_baseline.out
8107feffb5ac4ff02c35c2e831048590644f9076e2d693fe39b1fde0a7d0038c  out/rcw_recordonly.out
d79919f96cf0304c7f003e66a4ec4b8bf9e18f0608592236238e9d00efca3dca  out/rcw_t_baseline.out
f6d362efe196e82c0f08632cdf60e90eebe45f6f3054e6db72613b9c3b5122dc  out/rcw_t_recordonly.out
```

模型的可信度先交代：`lr5c.py` 不带开关时逐行移植 walk.rs 里 LR5 小世界那条测试，钉出 `(180794, 2710980, 10204, 1082, 0)`，与测试里的 `(180_794, 2_710_980, 10_204, 1_082)` 四个数逐个相同（`out/lr5_baseline.out`），所以后面加开关得出的数是在同一个模型上长出来的，不是另起炉灶。

## 总览：十格各推荐什么

| 格 | 推荐 | 一句话 |
|---|---|---|
| K1 | 甲 | key 读成路由区间，I-9.8 由 I-9.12 与 I-9.4 罩住，不另立判定 |
| K2 | 新条 I-3.19，三支判法 | T 是环里的根 ⇒ W 必为 0；T 是记录单落版本 ⇒ 从环里 txg 更小的最近一条根起重放比 W；读不全的行没判 |
| K3 | 保留实现员那一形 | 同文件 I-8.8 有先例；另在不适用理由里写明「从 1 起」判过且成立 |
| K4 | 另立 I-5.7，判「写过单元的记录单落版本，分配记录树根指针出生 txg 等于这一版 txg」 | 不并进 I-5.4，免动它钉死的评估数 |
| K5 | 不重钉四个数，LR5 等价测试在 RCW 落地时退役 | 第一样论证推完：「环里有效的根都不低于释放代」在 txg 有洞时不成立，两式因此不等价，两式也都对不上账 |
| K6 | RCW，上沿的低门槛取最新根自己带的 t | 模型：旧等式在 105,812 个合法世界误红，RCW 在 12,681,102 对判定里 0 误红 |
| K7 | 取号跳过再加一条「R 的实例表指针出生 txg 等于 R 的 txg」；C698 的 I-3.9 半边只推到结构，没推到判法 | 前提「写行之后每一版照抄同一张表」被清扫打破，量的装置与判据已给 |
| K8 | 单窗口盲带靠根环一份镜像判不出红，只能判到「部分没判」；要红得借 journal 证书 | 附证明 |
| K9 | 新条 I-8.14，读法与挂载读者同一道自证门，读不出的槽整条报部分没判 | 瞬时读故障不留在盘上，镜像判不了 |
| K10 | 新条判跨统计量的和：空闲加已分配加不可回收等于容量，标签 6 对闭式；只住内存的四项写成射程外 | 不报不适用也不报部分没判 |

## K1　C871：I-9.8 在第一版格式里判什么

**候选比较**

| 候选 | 内容 | 结论 |
|---|---|---|
| 甲 | key 读成路由给的区间（父条目分隔 key 到下一条分隔 key），I-9.8 写成由 I-9.12 与 I-9.4 罩住 | 推荐 |
| 乙 | 写明第一版格式里 key 就是偏移 0、由格式保证、没有可判对象 | 事实对，但把一条已有两条判据罩着的东西写成「无对象」，条文与门禁里 I-9.8「未实现」的状态一直对不上 |
| 丙 | 另有与偏移 0 分开存的 key 载体 | 找不到；见下 |

**依据**

- 条款：I-9.8 的原文是「打包记录类型 2 的记录偏移 0 的 inode 号 == 它在 inode 树里的 key」（`crates/singlefs-checker/invariants/09-inode树结构.md:20`）；D8 已定项 6 定「key = inode 号 8 字节」（`.claude/kb/decisions/08-核心索引结构.md:189`）。记录里偏移 0 就是 inode 号，没有第二份。
- 代码：池级 checker 走叶容器时 `let inode = read_u64(&record, 0);` 当号（`crates/singlefs-checker/src/walk.rs:4348` 起的 `walk_inode_leaf_container`），记录是 136 + 140n 切出来的，没有另存 key 的字节。
- 丙为什么找不到：候选载体只有两个。其一是容器头的容器号，其二是父条目的分隔 key。D8 与实现都写明它们不是「这一条记录的 key」：分隔 key 是创建时的最小 key，被 COW 重写后今天的最小 key 可以比它大，且不许跟着变（「创建时的最小 key 就是容器号」，`crates/singlefs-core/src/inode_tree.rs:133`）；checker 判的也是 `≤`（「self.judgements.judge("I-9.12", separator <= smallest, || {」，`crates/singlefs-checker/src/walk.rs:4628`）。一个只准 `≤` 的量不能当相等判的对象。
- 罩住了什么：落在容器里的每条记录 `inode >= 容器号`（「judge("I-9.4", inode >= container, || {」，`crates/singlefs-checker/src/walk.rs:4400`），同容器内严格递增、沿叶序容器号递增（I-9.4 的三句），分隔 key 小于等于孩子最小 key 且大于上一个孩子的最大 key（I-9.12，`crates/singlefs-checker/src/walk.rs:4628` 之后的两判）。三者合起来就是「每条记录的号落在它在树里的路由区间 [分隔 key_i, 分隔 key_{i+1}) 内」，最后一片上沿无界。GPU 两侧都判（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3741` 的 I-9.4，I-9.12 在同文件分隔 key 那一段）。

**推荐条文**（改 I-9.8 一行）：「打包记录类型 2 的记录偏移 0 的 inode 号落在它在 inode 树里的路由区间 [本条目分隔 key, 下一条目分隔 key)（最右一片上沿无界）。第一版格式里叶记录没有与偏移 0 分开存的 key（D8 已定项 6），所以这一条不另立判定函数：下沿与上沿由 I-9.12 判，容器内次序与不小于容器号由 I-9.4 判。」状态格写「已罩住，不另立判定」；C871 一行照这个关掉。

**最容易被打穿的地方**

1. 区间内换号判不出：把记录 5 的偏移 0 改成 6（邻居是 4 与 8）后次序、下沿、上沿都成立，I-9.8 的任何读法都红不了它，因为改完是一条合法的另一个号。它由跨引用判据接：数据单元头五元组按 inode 号对记录（I-9.10 的对象出生代三处一致）、删除队列（I-9.5）。欠账行「只改偏移 0、只红 I-9.8」的验收造不出，要在 C871 里改写这条验收。
2. I-9.12 只判读得出的孩子，别的版本先走过的孩子不重判这一版的分隔 key（C896 口径，`walk.rs` 里 `judge_separators_against_the_inode_children` 的注释）。所以「罩住」限于每个单元第一次走到的那一版。
3. 如果有人坚持 I-9.8 要有自己的判定函数，恒真的函数判别力为零，从这个角度乙更诚实；取甲是因为甲保住了「这条不变量有人判、判的是什么」这一句，并且不新造恒真函数。

**实现落点**：只改文档（`09-inode树结构.md` I-9.8 一行、`checks-owed.md` C871 一行）；代码不动。补三张「改偏移 0」的必红世界写进 `crates/singlefs-harness/tests/checker_known_bad_images.rs`，各自点名红的是哪一条：低于容器号 ⇒ I-9.4（`crates/singlefs-checker/src/walk.rs:4400`）；同容器里改成与邻居相同 ⇒ I-9.4 第四句；跨过下一条分隔 key ⇒ I-9.12。

**会红的变异**（已有，不用新写）：`crates/mutations.tsv:189` 「并行线三：I-9.12 的「分隔 key ≤ 孩子最小 key」那一条恒真」、`crates/mutations.tsv:2532` 「第22批 C I-9.4 第四句」。新三张世界要各挂进这两行的「必须红的测试」。

**什么现象会推翻它**：找到第一版格式里一处与偏移 0 分开存的 key（实现员的丙）；或造出一张只改偏移 0、次序与路由区间都保持成立、而 I-9.10 与 I-9.5 也都绿的镜像，且这张镜像是写者可达的。

## K2　C740：实例表行的 W 等于那次挂载重放施加的最大事务号

**先把写行的代码读清**（全部判法都建在这上面）

- 行是下一个实例的挂载写的：`instance_rows_to_write` 给 [max(上一个实例, 1), 要取的号) 里每个实例各一行，上一个实例写 (i, T, W)，中间实例写 (i, 0, 0)（`crates/singlefs-core/src/mount.rs:5841`）。
- T 与 W 的来源：`selected_root_txg: effective_root.checkpoint_txg`，`applied_transaction_high_water: journal.maximum_applied_transaction`（`crates/singlefs-core/src/mount.rs:8914` 一带）。`effective_root` 是重放之后那一版，所以 T 是重放施加的最后一次发布的 txg，没施加任何记录时就是所选根自己的 txg。
- 退回分支不重放：「退过去不重放：所选根与所选那一版都换成它」（`crates/singlefs-core/src/mount.rs:8774`），那一支把扫环报告里的 `maximum_applied_transaction: 0` 写进行（`crates/singlefs-core/src/mount.rs:8845`）。这一支里环上仍有实例 i 在 T 之后的、可重放的记录，W 却是 0。
- 实例切换（D23 注 4）写的 W 是「被重发的那个 checkpoint 里的最大事务号」（`.claude/kb/decisions/23-journal的角色与格式.md:374`），不是重放施加的最大号。`crates/` 里没有切换写者（只有准入为它留预留，`admission.rs`），所以今天写出的行都来自挂载恢复。

**候选比较**

| 候选 | 内容 | 问题 |
|---|---|---|
| 甲 | 对每一行，按 T 找起点根、重放、比 W，读不全的行没判 | 推荐，但起点根要先证明 T 是不是一条真根 |
| 乙 | 只判最新根的实例表里最后一行（上一个实例） | 老行的记录通常已被环盖掉，判得到的行少，但 T 是根那一支（W 必为 0）不用读记录，不该丢 |
| 丙 | 一律拿「从最新根起的重放」比 | 那是下一次挂载会做的事，不是那次挂载做的事；错对象 |

**推荐条文**（新条 I-3.19，落 `05-空间记账.md`：行是实例表的字段，同一族有 I-3.8、I-3.12、I-3.14；`03-journal.md` 只加一句指针）：

「最新根的实例表里每一行 (i, T, W)（i 低于最新根的实例）逐行判，最新根的实例表链没走到末片时整条不适用，理由同 I-3.8。分三支：(a) 根环里有实例 i、txg 等于 T 的自证过的根 ⇒ W 必须为 0（那次挂载没施加记录：落在这条根上，或退到这条根而不重放）；(b) 没有这条根，且按 I-7.17 的公式 T 该住的那一槽读得出、里面是 txg 小于 T 的根（含第 0 代种子）⇒ T 那一版是记录单落的：取根环里实例 i、txg 小于 T 的最大一条根 S，从 S 起照 D23 已定项 14 的重放规则重放实例 i 的记录；重放前缀最后一次发布的 txg 等于 T 时 W 必须等于施加的记录里最大的非 0 事务号（没有非 0 的取 0）；(c) 其余一律这一行没判：T 该住的槽读不出或住着 txg 大于等于 T 的别的根、没有起点根 S、重放停在点名单元验不过、锚认不定、重放读回的最后一次发布不是 T。有违例照报违例；至少判了一行而别的行有没判的，报部分没判并逐行列出；一行都没判到（含没有行）报不适用。」

**回答四问**

1. 条文落 `05-空间记账.md`（行字段），`03-journal.md` I-8.7 那一行的「W 能当精确前缀的依据」后面加一句指到 I-3.19。
2. 判最新根实例表里的每一行，不只判最近一行：T 是根那一支（a）零成本、对老行同样成立；记录那一支（b）老行多半没判，不误红。
3. 记录被环盖掉或读不出：报「这一行没判」，汇总成部分没判，不报不适用；理由是 I-3.19 的射程里别的行仍判得了（C701 的定案「读不出的一律报部分没判」，`checks-owed.md` 里 C701 的口径同此）。环转圈时先丢的是最早的记录，从 S 起的链首接不上，重放读回的最后一次发布不是 T，走 (c)，不会误红。
4. T 那条根已不在环里：靠 (b) 的槽检查分三种。槽里住着 txg 小于 T 的根 ⇒ T 从没落过盘，是记录单落版本，照 (b) 判；槽里住着 txg 不小于 T 的根 ⇒ T 那条根可能存在过又被盖掉，不能当记录单落，没判；槽读不出同样没判。不检查槽、直接对「环里没有 T 那条根」重放，会在 T 原本是一条真根、W 本来就是 0 的行上把链上更早的发布当成施加过的，误红。

**实现落点**

- `crates/singlefs-checker/src/walk.rs`：`replay_the_next_mount_would_do`（`crates/singlefs-checker/src/walk.rs:11925`）本来就吃一个 `chosen_root` 参数，筛记录用的是 `record.instance == chosen_root.instance`（`crates/singlefs-checker/src/walk.rs:11934`），所以「从给定根起、只取上一个实例的记录」不用另写一份：把 S 包成 `RootView` 传进去即可。要改的是它的结果：`Applied` 一支（`crates/singlefs-checker/src/walk.rs:11837`）加一个「施加的记录里最大的非 0 事务号」字段（在 `open_publish` 收进一次发布时累加；`ScannedJournalRecord.transaction` 在 `crates/singlefs-checker/src/walk.rs:10149`），再加 `last_record.checkpoint_txg` 已有，用来对 T。
- `judge_instance_table_rows`（`crates/singlefs-checker/src/walk.rs:4836`）判行的那一处旁边加 `judge_the_applied_transaction_high_water_of_the_rows`，行里的 W 已经解出来了（`applied_transaction_high_water_mark`，`crates/singlefs-checker/src/walk.rs:8508`）；起点根 S 与 T 的槽检查用 I-7.17 的公式（`crates/singlefs-checker/src/image.rs` 的 `judge_root_records_live_in_the_slot_their_txg_names` 同一个公式）。
- `crates/singlefs-checker/src/walk.rs:11870` 的 `named_units_of_the_record_verify_on_both_copies` 读的是现在这张镜像：记录点名的单元在那次挂载之后可以被释放、复用，今天验不过不等于那次挂载验不过。这一类停止一律走 (c)，不当违例。
- GPU：`crash_judge_checker.wgsl` 里同名的 `replay_the_next_mount_would_do(chosen: CheckerRoot, …)` 也吃 `chosen`（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:4975`），只有 I-4.1 用最新根调了一次（`:5543`）。同样给结果结构加最大事务号字段，在走读内核走实例表链到末片之后（I-3.8 那一处）对每一行调一次；行数设个上限，超出的行记射程里判不了。
- 条目：`IMPLEMENTED_INVARIANTS` 末尾加一条、覆盖表加下标（`image.rs`，`crash_judge_gpu.rs`），随条数登记（`invariants/README.md` 的 129 → 130）。

**会红的镜像**（用真写者出来的流造，不手捏字节）

1. 三次挂载的流：第 1 个实例发布 A、B，B 的记录已落而根槽没落（崩），第 2 次挂载恢复施加 B，写行 (1, T=B.txg, W=B 的事务号)；把 W 改成 0 ⇒ I-3.19 红（b35 攻方量到的那一形：合法 4、构造 0，`.claude/kb/checks-owed.md:455`）。
2. 同一张镜像把 W 改成 B 的事务号 + 1 ⇒ 红（防「≥」读法）。
3. 退回分支的镜像（所选根之后环上还有可重放的记录，挂载按 F2 退到旧根）：行是 (i, T=旧根的 txg, W=0)，走 (a)，绿。
4. 把第 1 张镜像里 T 那一槽换成 txg 更大的根，或把 S 的根槽翻坏 ⇒ 这一行没判，整条部分没判，不红。

**会红的变异**：核心侧 `crates/singlefs-core/src/mount.rs:8914` 的 `applied_transaction_high_water: journal.maximum_applied_transaction,` 换成 `applied_transaction_high_water: 0,`，镜像 1 必红（`crates/mutations.tsv:735` 已有这一行变异「实七 「写的实例表行」那一形：可写挂载给上一个实例写行时 W 恒写 0」，但它必须红的测试是 `fault_injection_fast_tier` 里模型对账的那一条，不是池级 checker；新条落地后要在这一行的必须红测试里再挂上 I-3.19 的那张镜像，让 checker 一侧也抓得到）；checker 侧把 (a) 支的 `W == 0` 换成恒成立、把 (b) 支的比较换成恒成立，各一行。

**最容易被打穿的地方**

1. (a) 支的「T 是环里的根 ⇒ W 必为 0」把退回分支的 W=0 当成规则写死了。若以后实例切换落地，切换写的行 W 非 0 而 T 是根，会误红。条文要加「本条只判挂载恢复写的行；切换写者落地时改写 (a)」。
2. (b) 支的槽检查依赖「同一槽里 txg 大的根一定后写」；mkfs 种进的第 0 代根在所有区域的槽 0，T 小于等于 S 的情形要走 (c)。
3. 重放读的是此刻的环：那次挂载之后别的实例的记录占了环槽（计数器对不上）时重放只会更短，走 (c)，不会更长。反过来，那次挂载时读不出而今天读得出（瞬时读故障）的记录会让 W' 大于 W 而误红；这一形不留在盘上，与 I-7.9 同一处已认代价（`crates/singlefs-checker/invariants/02-根环.md:16` 写着「读故障不留在盘上」）。

**什么现象会推翻它**：一张写者可达的合法镜像上 (b) 支红；或造出行 (i, T, W) 里 T 是环里一条真根、而 W 非 0 的真写者镜像。

## K3　C469 ②：I-8.7「从 1 起」那一判算不算评估

**候选**：甲（实现员取的）只在不成立时记一次评估，成立时不把整条从「凑不出一对可比的号」的不适用抬成成立（`crates/singlefs-checker/src/walk.rs:10909` 的 `if *transaction != FIRST_TRANSACTION_NUMBER_OF_AN_INSTANCE {`；注释 `crates/singlefs-checker/src/walk.rs:10833`）；乙 成立也算评估。

**推荐：甲**。依据：

- 同一份 `walk.rs` 里 I-8.8 已经是这个先例：「提交标记字节」这时照样判、判出别的值照样红，「只是它单独成立不算这一条成立」（`crates/singlefs-checker/src/walk.rs:10947`），理由是第一版可达镜像上它恒绿，这一条的判别力在坏镜像上。「从 1 起」在第一版可达镜像上同样恒绿（真写者的第一条非 0 记录就是 1），形状相同，取法应当相同。
- 乙会把「这张流上一对可比的号从没凑出来」这个覆盖缺口抹掉：现在三份测试各钉了它——`crash_enumeration_new_pool_file_creation_stream.rs` 的 `never_comparable_on_this_stream = ["I-9.14", "I-8.7", …]`（`crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs:275`）、`common_images_for_the_checker_tier/writable_mount_of_a_formatted_pool.rs` 的 `NOT_APPLICABLE_WITH_ONE_FILE_VERSION` 一族、`checker_known_bad_images.rs` 的干净镜像用例。这三份钉的就是「① ② 在这条流上没有对象」，改乙以后它们被一句「从 1 起成立」盖住，流上 ① ② 的盲区看不见了。
- 乙的好处（流上真有正面证据证明真写者从 1 起）可以不改评估计数拿到：把不适用的理由改成「环里没有哪个实例写出过两条非 0 事务号的记录；『从 1 起』那半句判过且成立」。理由是 `&'static str`，三份测试钉的是不变量名单与判定类别（我读了第一份的名单，另两份按实现员的描述，没逐份核），推的是它们不钉理由文字，所以不动它们（`crates/singlefs-checker/src/walk.rs:10805` 现在的理由是一句「一对可比的号都凑不出来」，要改成两个静态串按「从 1 起」判没判过选一个）。

**实现落点**：`judge_transaction_numbers_start_from_one_per_instance`（`crates/singlefs-checker/src/walk.rs:10836`）返回「判过几个实例」，`judge_transaction_numbers_per_instance`（`crates/singlefs-checker/src/walk.rs:10747`）末尾 `judged_pairs == 0` 那一处据此选理由；GPU 走读内核同名函数不动评估位，不置位。

**会红的变异**：已有「C469 I-8.7 从 1 起那一判恒成立」「…那半句整段不判」等（作业包 P1 波一的追加行）；新增一行：理由串选择恒取「没判过」。

**最容易被打穿的地方**：「评估次数」与「判过」两个量在这里分开了，以后别的不变量要抄这个写法时容易抄错。条文里要把这条规则写成一般句：「某一半句在可达镜像上恒成立、判别力只在坏镜像上的，成立不单独算评估」，I-8.8 与 I-8.7 各引它。

**推翻条件**：有人证明三份测试钉的「不适用」其实不是覆盖缺口的信号（例如流上本来就不该出现两条非 0 记录）；或乙那一形让某条流把一个真缺口报成了成立。

## K4　C706 没罩住的一形：记录单落版本的分配记录树根

**问题的精确形**：记录单落的版本只有 journal 记录里的新根段，没有同代根可比（I-8.10 比不到）。树表 0 条的版本，分配记录树根由根记录（或记录新根段）直接持有；攻方把记录里这一项换成另一棵合法的旧树。I-5.4 现在红的是带树的版本里根记录这一项非零；这一格是暂挂在 I-5.4 下的：「不变量清单里没有一条专管根记录这一项，暂挂在 I-5.4 下（归属交主 agent 定）」（`crates/singlefs-checker/src/walk.rs:7949`）。0 条的版本两样都是非空合法树，红不了。

**候选**

| 候选 | 内容 | 问题 |
|---|---|---|
| 甲 | 这一版写过单元时，分配记录树根指针的出生 txg 等于这一版 txg | 推荐 |
| 乙 | 点名项里的每个单元在这一版的分配记录树里有一条分配代等于这一版 txg 的未释放记录 | 更强，要逐个点名项读树，读不出的多；留作甲的加强 |
| 丙 | 条款写明这一项在这一格无害、没有判据 | 不成立：恢复按记录重建分配器，换成旧树会让下一次挂载在已用的槽上重新分配；D8 已定项 8 没有说这一项冗余（C706 一行自己写了） |

**依据**：发布路径的两条分叉。写过单元的发布每次都重建分配记录树，新的根指针出生 txg 是这次发布的 txg：`let allocation_record_tree_root = allocation_built.version.root_pointer();`（`crates/singlefs-core/src/transaction.rs:2161`），没有改动的节点只取指针（`crates/singlefs-core/src/transaction.rs:13575`）。零单元发布一个字节都不写，分配记录树根照抄上一版的指针（`crates/singlefs-core/src/transaction.rs:1720`「零单元发布一个字节都不写：分配记录树照抄上一版的那一条指针，账也因此一条都没变。」，`:1721` `allocation_record_tree_root: previous_root.allocation_record_tree_root,`），出生 txg 小于这一版 txg，且记录的点名项为空。所以「点名项非空 ⇒ 根指针出生 txg == 这一版 txg」在两种发布上都成立，且「点名项非空」来自记录自己（`ScannedJournalRecord.named_entries`，`crates/singlefs-checker/src/walk.rs:10163` 一带），不依赖被换的那个指针，不循环。换成旧树的坏镜像：指针整条拷自旧版，出生 txg 是旧版的；若攻方连出生 txg 也改成这一版的，指针与被指单元头的诞生代号对不上，红在 I-1.2 的遍历方向（`01-块自描述.md` I-1.2）。

**推荐条文**（新条 I-5.7，落 `07-分配器.md`）：「由记录施加出来、根槽从没落盘的那几版（含下一次挂载先施加的那一版），记录的点名项非空时，新根段里分配记录树根指针的出生 txg 等于这条记录的 checkpoint_txg；点名项为空（零单元发布）不判。树表非空的版本，分配记录树根在树表种类 3 的条目里，不在这一格。」

**为什么另立而不并进 I-5.4**：I-5.4 的评估数是钉死的（状态格写着「评估 1,842,252 个、违例 0」「新池新建文件流评估 4 个」），并进去会让它们变；`crates/singlefs-checker/src/walk.rs:7949` 的注释自己说这一项没有专管的条目、暂挂 I-5.4。新条也让 I-5.4 里「带树一版根记录该项全零」那一格有机会以后搬过来（本轮不搬）。

**实现落点**：`walk.rs` 里 `versions_applied_only_by_records` 与 `version_the_next_mount_applies_first`（`crates/singlefs-checker/src/walk.rs:8206`）已把这两类版本连同它们的新根段交给 `judge_allocation_records_disjoint`（`crates/singlefs-checker/src/walk.rs:7966`），其入参 `VersionToJudgeTheAllocationRecordsOf`（`crates/singlefs-checker/src/walk.rs:7931`）只有名字与三个指针。给它加两项：这一版的 txg 与「记录点名项是否非空」，新函数 `judge_the_allocation_record_tree_root_born_with_the_version` 在同一处循环里调，读指针的出生 txg 用 `parse_node_pointer`。GPU：扫描内核 `judge_allocation_records_disjoint_of_a_version` 三个调用点（候选根、记录施加的版本、下一次挂载先施加的版本）传同样两项，只有记录施加的两类版本调新判（候选根有同代根可比，I-8.10 管）。

**会红的镜像**：三次挂载的流，第 3 次挂载的写行发布（带实例表单元，点名项非空）记录已落、根槽没落；把记录里分配记录树根换成第 2 次写行那一版的根指针 ⇒ I-5.7 红，I-5.4 绿（两样都是 0 条版本的合法树）。wave 1 加的那张 C706 坏镜像是带树版本，走 I-5.4 的旧格，别拿它当这一形的镜像。

**会红的变异**：新判的 `==` 换成 `<=`（放过旧树）；「点名项非空」恒取假（整条不判）；GPU 侧调用点漏一个。

**最容易被打穿的地方**

1. 「点名项非空 ⇒ 分配记录树被重写」靠的是写者每次重建树：`allocator.records()` 全量交给 `build_allocation_record_tree`（`crates/singlefs-core/src/transaction.rs:2161` 上面）。若以后有发布会写单元但把分配记录推迟到下一次发布，这一条在合法镜像上误红。条文里写明依据，核心一侧的这个前提要有一条核心用例钉住。
2. 不覆盖树表非空的记录单落版本里「树表种类 3 条目的根指针被换」，那一形要读树表单元（它被点名，内容却没人比），留作乙的加强。
3. 零单元发布不判，换成旧树的坏镜像若恰好出现在零单元记录上（点名项空）看不见，但零单元发布不改账，按定义没有「新的」分配记录树，这个盲区是格式性的。

**推翻条件**：一张写者可达的合法镜像上点名项非空而根指针出生 txg 不等于这一版 txg。

## K5　C685：LR5 小世界补的四样

**先说结论**：四样里第一样的论证推完以后，结果是「两式不等价，且两式都对不上账」。这不是补一个数的问题，是 LR5 那一族选根式（丁-defer）本身在有 txg 洞的世界上不成立；K6 的 RCW 落地就把这个选根式换掉了，所以不该把四个数重钉，LR5 等价测试在 RCW 落地的同一批退役，换成 RCW 的小世界测试。下面的数都在模型上得出：`lr5c.py` 逐行移植 `crates/singlefs-checker/src/walk.rs:22102` 那条测试，不带开关时与 `(180_794, 2_710_980, 10_204, 1_082)` 四个数逐个相同。

**第一样：由记录施加出来、根槽从没落盘的那一版（r）**

模型里的加法（`lr5c.py recordonly`）：任一现行版本之后再发一版 V（引用任选，规则同普通发布），V 的记录落了、根槽没落，不进环；恢复施加 V，新实例的写行发布从 V 起，txg = max(环里最大, V 的 txg) + 1（D23 已定项 14 第 3 条：`.claude/kb/decisions/23-journal的角色与格式.md:373`）。

推完：原论证最后一步写的是「r 出了环，环里有效的根都 ≥ g、门槛 ≥ g，S 不算 defer」（`crates/singlefs-checker/src/walk.rs:21688`）。这一步假设环里没有洞。记录单落版本恰好造洞：V 的槽没写，这一圈里留着更老一圈的有效根，门槛（环里最旧有效根）就低于 g，S 仍在 defer。模型里的反例（`out/lr5_recordonly.out` 的第一个例子）：

```
ring 0 (2,4) rows {1:2}      ring 1 (1,1) refs [0]
ring 2 (3,6) rows {1:2}      ring 3 (1,3) refs [1]      当前 (3,6)，F 0
最新根账：槽 1 在 txg 4 被 (1,2) 释放；(1,2) 的槽已被 (3,6) 盖掉，txg 5 是洞
```

槽 1 的释放代 4 大于门槛 1（环里最旧有效根是 (1,1)），在 defer 里；引用它的只有被抛弃的 (1,3)。原式：不并（(1,3) 被抛弃），并进 ∅；LR5 式：并进 {1}。账里的已分配槽集是 {0,1}：原式 walked ∪ 并进 = {0} ≠ {0,1}，LR5 式 = {0,1} 对上。也就是说在这个世界上「被抛弃根还引用得到这一槽」，并且被抛弃的根正好是对的那个。

**四样各自加进模型的形态与两式是否同值**（读不出的位置子集照原测试枚举，除非另注）：

| 样 | 加进模型的形态 | 世界数 / 判过的对数 / 原式并进非空 / 两式不同 / 破原断言 | 同值？ |
|---|---|---|---|
| 基线 | 原测试 | 180,794 / 2,710,980 / 10,204 / 1,082 / 0 | 只在双故障形不同 |
| 一 记录单落版本 | 上面 `recordonly` | 1,120,960 / 15,115,218 / 241,551 / 19,707 / 13,706 | 不同，且原断言被破 |
| 二 走读断掉的根 | 根头读得出（txg、实例、F 都算数）、引用读不到：每个位置子集各当一次「走读断掉」，同时没有读不出的位置 | 180,794 / 2,891,774 / 15,256 / 111,342 / 111,342 | 不同 |
| 三 系统配置读不出 | F_生效只取根上的（`sysunread`） | 180,794 / 2,710,980 / 7,628 / 1,082 / 0 | 同值（不同的 1,082 个世界同基线） |
| 四 txg 跳号留下的洞 | 模型里跳号只有两个来源：恢复取号（基线已有，落在被抛弃根之上，不留洞）与记录单落版本（样一）；根槽写失败重发跳一格今天代码走不到 | 并入样一 | 并入样一 |

- 样二的 111,342 个不同，LR5 式多知道的是被抛弃根的引用，而原式依赖的有效根恰是断掉的那条。这类世界里 I-3.1 本就不该比：选根集合里任何一条根走读断掉，它的引用集合不全，应当报部分没判（C701 的定案），而不是比两式；所以样二不是「两式要同值」的对象，是「I-3.1 要先判走读完整」的对象。
- 样三同值：F_生效取小只让原式并得更少（10,204 → 7,628），不动两式之差。
- 账对照（`out/lr5_truth_*.out`）：全可读世界里 walked ∪ 并进 与最新根账的「已分配槽集」逐个比。基线 180,794 个世界两式都对得上（0 个不符）；样一 1,120,960 个世界原式不符 105,812、LR5 式不符 100,966。所以样一上两式都不是对的判法，LR5 式只是更接近。

**推荐动作**：不重钉；K6 落地同批把测试换成 RCW 的小世界测试（`rcw.py`，数见 K6）。若 K6 暂不落，则只做样三（同值）并记「样一、样二不在等价范围，I-3.1 在这两形上已知误红」。

**会红的结构断言**：测试里「两式在槽 S 上不同而换下它的那版根不是『在环里、读不出』」那条断言，样一加进去后它转红并打出整个盘面（模型里 13,706 处），这正是欠账行要的判别力自证。

**最容易被打穿的地方**：样一的模型假设（记录单落版本不占环槽、新实例 txg 跳到它之后、账由它继承）是我按 D23 已定项 14 推的，没有真写者流对过；真写者流是 `crash_enumeration_*` 一族，不起 checker 档的 cargo，所以这一格复核不了。数是模型的数，方向（两式不等价、旧等式在有洞的合法世界误红）与 C697 攻方在真盘面上量到的 153 / 180 同向。

**推翻条件**：真写者流里造出样一的世界（记录单落、txg 有洞、被抛弃根引用着在 defer 里的槽），原式与账对得上（则样一的模型假设错了）；或 RCW 的小世界测试在合法世界上红。

## K6　C697：I-3.1 改成记账对最新根账里的记录比（RCW）

**今天的判法**（要换掉的）：逐盘 `allocated == walked`，walked = 候选集遍历的槽 + 隔离豁免 + F 之下仍在 defer 的并进 + 钉带出来的（`crates/singlefs-checker/src/walk.rs:20238` 的 `allocated == Some(walked),`；GPU `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2573` 的 `judge(INVARIANT_I_3_1, have_allocated && eq64(allocated, walked));`）。「最新根带的 F 低于 F_生效时不判」那一支（`crates/singlefs-checker/src/walk.rs:20204` 的 `accounting_was_written_under_the_effective_floor`）保持。

**RCW 的精确条文**（用户 2026-10-04 定取 RCW；这里把「上沿」的低门槛写实）：逐盘 d，设
- C_d：最新根记账树里「已分配」那一行（统计量 1）；
- 记录集 R_d：最新根的分配记录树里盘 d 上全部记录；E_d(θ) = 未带已释放标志的记录罩住的槽，并上带已释放标志而释放代大于 θ 的记录罩住的槽；
- Ω_d：候选集遍历到的槽（加隔离豁免的槽、加钉带出来的槽，口径同今天的 `walked` 去掉 defer 并进那一项）；
- θ_high = `deferred_slots_reclaim_threshold(可读根, F_生效)`（`crates/singlefs-checker/src/walk.rs:9111`，今天丁-defer 与 I-7.18 共用的那个）；
- θ_low = 最新根自己带的回收门槛 t（根记录偏移 428，I-7.18：「根记录带回收门槛 t（8 字节）」，`crates/singlefs-checker/invariants/02-根环.md:25`）。

要同时成立三条：(L) C_d 不小于 |E_d(θ_high)|；(P) Ω_d 里每个槽都在 E_d(θ_high) 里；(U) C_d 不大于 min(|E_d(θ_low)|, |Ω_d ∪ U_d|)，其中 U_d 是带已释放标志、释放代大于 θ_low、不在 Ω_d 里的记录罩住的槽。(L) 与 (P) 管记账少的方向（不安全方向），(U) 管泄漏。

用 t 而不用旧 RCW 里那个按窗口洞算的 θ_low，理由是：t 是 C773 之后每条根自己带的、写者记这条根时的门槛，账（统计量 1）就是照它记的；旧 RCW 那轮（2026-10-04）还没有这个字段，才要从窗口洞里反推下沿。t 可信与否不影响不安全方向：(L) 用的是 checker 自己现算的 θ_high，写者把 t 往高写想少记，C_d 就会低于 |E_d(θ_high)|，(L) 红。

**模型对 RCW 的检验**（`rcw.py`、`rcw_t.py`；θ_low 在 `rcw.py` 里取 F_生效，在 `rcw_t.py` 里取 t；世界同 K5，写者的计数器 = 它自己账的 |E(t)|）

| 量 | 基线 180,794 世界 | 加记录单落版本 1,120,960 世界 |
|---|---|---|
| 旧等式在全可读合法世界误红 | 0 | 105,812 |
| 旧等式在（世界 × 读不出子集）误红 | 484,090 / 2,239,343 | 3,426,030 / 12,681,102 |
| RCW 误红（两种 θ_low） | 0 | 0 |
| 泄漏变异（凭空加一条未释放记录）抓到 | θ_low=F：39,513 / 54,008；θ_low=t：54,008 / 54,008 | θ_low=F：159,841 / 201,609；θ_low=t：201,609 / 201,609 |
| 计数器少 1 抓到 | 180,794 / 180,794（旧等式同） | 1,120,960 / 1,120,960（旧等式 1,021,933） |
| 删一条记录抓到 | 218,827 / 307,580（旧等式同） | 1,373,335 / 2,040,311（旧等式 1,336,297） |

读法：旧等式的误红正是 K5 样一的世界（有洞、记录单落版本），RCW 在同一批世界上 0 误红；上沿的低门槛取 F_生效 会漏掉一部分泄漏，取 t 一个不漏，所以条文写 t。删一条记录的抓到数小于变异数，是因为被删的记录释放代不高于门槛时它本来就不在 E 里，删了不改 C，是空变异，不是漏判。

**现行等式必红的流**：mkfs 收最小 journal 环（1 MiB）、崩在根槽写之前、重挂、之后大文件顺序写。要加的流在 `crates/singlefs-checker-tier/tests/` 下（作业包 P1 登记的 `crash_enumeration_<短环大文件>_stream.rs`），参数是 journal 环 1 MiB、文件顺序写到环转过去、崩点取根槽写之前；旧等式在这条流上必须有红的状态（攻方在副本上量到的 153 / 180，`.claude/kb/checks-owed.md:421`），RCW 全绿。这条流我跑不了（checker 档），所以「旧等式在这组参数的流上必红」是复核不了的一项，只有 K5 样一的模型给了同向的证据。

**改完是否在攻方 180 个有洞盘面上对记账少的坏写者全红**：旧一轮的原始输出（`git show 04ceaa904^:research/prompts/m3-candidate-hole-r1-opus-output.md` 第 167–174 行）写明 RC / RCW 在 m4a 记账少的 180 个有洞盘面全红、各集正确写者 0 误红，用的是窗口洞反推的 θ_low；换成 t 之后没有真盘面可跑，复核不了。模型里「记账少」的两种坏写者（计数器少 1、删一条记录）在 RCW 下的抓到数不低于旧等式。

**实现落点**：`crates/singlefs-checker/src/walk.rs:20235` 一带那个 `if accounting_was_written_under_the_effective_floor {` 里把 `allocated == Some(walked)` 换成 (L)(P)(U) 三判，仍记在 "I-3.1" 名下（违例说明写明是哪一条）；要读 R_d：最新根分配记录树（今天隔离与丁-defer 各读一遍，同一批节点，`allocation_records_of_the_newest_root_without_judging`）。丁-defer 那一族（`deferred_slots_*`、`slots_the_pin_kept_in_defer`、LR5 测试）在 RCW 落地后不再是 I-3.1 的输入，I-3.16 还吃 `occupied_slots`，那一份保留。GPU：`judge_accounting`（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2541`）里换掉 2573 那一判；E_d(θ) 的和在 `C_ALLOC_RECORDS` 数组上一趟累加，(P) 要逐槽，用已有的 `c_slot_covered_by_the_newest_root` 同形的循环，注意 WGSL 展开大小钉值（`crash_judge_dispatch.rs` 的两条预算测试）要重钉，归主 agent 跑。

**最容易被打穿的地方**

1. 泄漏方向靠 (U)。(U) 的 U_d 包含了释放代在 (θ_low, θ_high] 之间、已被回收却还写着已释放的记录，是故意放宽的，放宽的量就是漏判的上限；t 小于 θ_high 的差越大越松。
2. t 在「零单元发布、无文件版本上的复制根」是抄上一版的，环转满后与现算不等（I-7.18 因此只判建账的发布）。账同样是抄的，所以 t 与账配对，没问题；但 C560 那一态（最新根之后的槽读不出且 N+1 ≥ 3S）I-7.18 已知红（`crates/singlefs-checker/invariants/02-根环.md:25`），写者按旧内容算门槛、checker 读不到旧内容，RCW 的 θ_high 现算在那一态也可能与写者不同，(L) 有没有红我没推，需要核；核之前在条文里把 C560 那一态并进已知红。
3. 隔离记录（带隔离记号）在 E 里，在 Ω 里不在，(U) 要把整组豁免的隔离槽并进 Ω_d，与今天 `exempted_all` 同口径。

**推翻条件**：一张写者可达的合法镜像上 (L)(P)(U) 任一条红；或一个记账少的坏写者在 RCW 下与旧等式同绿（旧等式红才算数）。

## K7　C699 与 C698：取号跳过改认出生代号

**今天的分类**（I-3.5 的缺口条款，`crates/singlefs-checker/invariants/05-空间记账.md:11`）：相邻两版 (R⁻, R) 的 txg 差大于 1 时，只有「最新根的实例表里有一行 (R⁻ 的实例, T = R⁻.txg)、R 的实例比 R⁻ 的大、两者之间每个实例的行都是 (j, 0, 0)」才认作取号跳过（gap-row），别的缺口里用户可见树子树里的落点不判。认错的形：R 所在实例自己更早的版本存在过却读不出（根槽读坏），行证据盖不到它，被当成取号跳过（`.claude/kb/checks-owed.md:422` 的 nrb 6 / 6、rs1 285 / 285）。

**先定前提怎么量**（C699 要求「先量前提」：写行之后每一版照抄同一张实例表）

- 静态一刀：实例表指针只有两处被改写，构造 `InstanceTablePlan::Rewrite(` 的只有挂载写行（`crates/singlefs-core/src/mount.rs:5267`）与实例表清扫（`crates/singlefs-core/src/instance_table_sweep.rs:721`）；别的发布都走 `InstanceTablePlan::Carry(previous.root.instance_table)`（`crates/singlefs-core/src/transaction.rs:9593` 一带十来处）。所以前提在没有清扫的历史上成立，在清扫发生的那一版之后不成立（指针被改写，出生 txg 变成清扫那一版的）。这条静态结论不需要装置，但清扫在产品上真发生，前提必须带一个清扫例外。
- 动态一刀（装置）：用真写者出来的流（可写挂载三次、每次发布若干版，清扫开与关各一组；清扫的开关是 `InstanceTableSweepConfiguration`），对每个状态里每条自证过的根取 (实例, txg, 实例表指针出生 txg)。判据三句：(P1) 一个实例的第一条根的实例表指针出生 txg 等于它自己的 txg；(P2) 同一实例相邻两条根，后一条的指针出生 txg 等于前一条的，或等于后一条自己的 txg（清扫那一版）；(P3) 指针出生 txg 不大于根自己的 txg。关清扫的组要求三句全成立、违例数 0；开清扫的组要求违例只出现在 (P2) 的第二分支，且那一版确有清扫的写账。量的口径写进流的断言里，不靠人读输出。
- 这一步我复核不了：它要真写者的流（checker 档），这里只给装置与判据。

**候选**

| 候选 | 判法 | 结论 |
|---|---|---|
| H1-A | 只认行（今天） | 根槽读坏的镜像上认错 |
| H1-b | 行 + R 的实例表指针出生 txg：gap-row 成立，且指针出生 txg 等于 R.txg，才认取号跳过；指针出生 txg 小于 R.txg 说明 R 所在实例在 [指针出生 txg, R.txg) 里有存在过的版本，缺口里至少有一个是它的，不算取号跳过 | 推荐 |
| H1-c | 再并 journal 里已落记录 | 已被上一轮判决出局（短环盖掉记录那一形退回行证据） |

**推荐条文**（改 I-3.5 缺口条款）：在 gap-row 的三个条件后加第四个：「且 R 的根记录里实例表指针（偏移 170）的出生 txg 等于 R 的 txg」。第四个条件不成立（出生 txg 小于 R.txg）⇒ 按「不是 gap-row」处理（用户可见树子树里的落点不判，别的落点照判 birth > R⁻.txg）。指针全零或读不出出生 txg ⇒ 这一对按「不是 gap-row」处理并记部分没判。

**已知口子**：R 恰好是清扫改写实例表的那一版、而它前面同实例的版本又读不出时，指针出生 txg 等于 R.txg，仍被认成取号跳过（H1-b 在这一形上等于今天的 H1-A）。要补得靠清扫写账或根记录里另有实例的起始 txg，条文里写成已知代价。

**C698 的验收**：(1) I-3.5 那一半：nrb 6 / 6 照上面第四个条件不再认错。(2) I-3.9 那一半（回退加根槽读坏的 rs1 里红 150 / 285）我只推到结构：`judge_release_generations`（`crates/singlefs-checker/src/walk.rs:7556` 起的注释）取 (L, T]，L 是候选集里最后一条还引用该落点的根、T 是 L 之后第一条不引用的根；回退目标那一版根槽读坏时它不在候选集，复活的单元只被回退之后的根引用，L 偏晚，释放代落不进 (L, T]。可行的收法是照 `crates/singlefs-checker/src/walk.rs:7640` 的前例（后面的根走不完就跳过），对「(L, T) 之间有一个 txg 在环上没有可读的根、而 I-7.17 公式给它的槽读不出或住着更新的根」的记录一律走 `part_of_the_range_not_judged`。这一步没有 rs1 的 150 张镜像，判法是推的，复核不了；三条各读各的（Q3 的定案，`m3-candidate-hole-r1-main-verification.md` 第六节），所以不抽共用洞表，I-3.9 的洞判断写在自己的函数里。

**实现落点**：`judge_the_appearance_rule_of_adjacent_candidate_versions`（I-3.5，walk.rs，下标 65）里 gap-row 判断那一处加第四条件，入参要把每条候选根的实例表指针出生 txg 带进去（`ScannedCandidateRoot` 一带）；GPU 走读内核同名函数（`crash_judge_checker_scan.wgsl`）同改。

**会红的镜像**：nrb 一组（最新实例写行那条根读坏）与 rs1 一组，H1-A 认错的世界改完 0；变异：第四条件恒真、第四条件取 `<=`、GPU 侧漏改。

**最容易被打穿的地方**：清扫口子；指针出生 txg 靠根槽读到的值，R 自己的根槽读不出时 R 不在候选集，这一对不存在，所以第四条件只对读得出的 R 有意义。

**推翻条件**：动态装置上 (P1)(P2)(P3) 在关清扫的组里出现违例；或造出写者可达、gap-row 与第四条件都成立却其实有存在过的版本的镜像（清扫那一形之外）。

## K8　C717：回退下界上限的单窗口盲带

**结论先说**：只靠根环这一份镜像，做不出「F 越过洞里旧根 X 时判红、又不在合法洞上误红」的判法；能做到的最强是「报部分没判」。要红，必须再借一条 journal 证书。下面给证明、给能做的、给证书的形状。

**问题的精确形**（`crates/singlefs-checker/invariants/02-根环.md:16` I-7.9 与 `crates/singlefs-checker/src/walk.rs:8800` 的 `judge_rollback_floor_raises_against_their_ceilings`）：抬 F 的根 r 的上限，是「用它之前的根」算的 min(每块盘最新持久有效根, 第 4 新非空有效根; 非空不足 4 个取最旧有效根)。checker 只取锚在镜像最新根 N 上的一圈窗口（txg > N − 3S）里的根。窗口里的根在抬 F 那一刻就在环里、此后不会被盖（盖它要 txg ≥ 它 + 3S > N 的根）；窗口之外（txg ≤ N − 3S）的根在抬 F 那一刻可以在环里、今天可能已被盖掉，也可能因为同槽的更晚的根从没写成而作为 X 留在洞里。窗口只会让上限算高（更松），不会误红；代价是 F 在 (X, W] 里时看不见（W 是窗口里最旧的有效根）。

**证明：只靠根环不可能有靠谱的红**

设抬 F 那一刻的环为 A = 窗口里的根 ∪ 窗口外现存的根 ∪ 窗口外已被盖掉的根 M。窗口外的 t（X < t ≤ N − 3S）若曾有根，今天它的槽（I-7.17 公式给出）住着 t + 3S·j 的更晚的根；若从没有根，槽里住着更老一圈的 X 一类的根。对一个 t 来说，「被盖掉」与「从没写过」在今天的镜像上区分不开的唯一情形是：槽里住着 txg 大于 t 的根。而每个这样的 t，它的槽今天必住着窗口里的根（t + 3S ≤ N），所以窗口外 (X, N − 3S] 里每一个没有现存根的 t 都是「可能被盖掉」。M 里若有非空根 Y（X < Y），抬 F 那一刻的非空根数可以 ≥ 4，上限是第 4 新非空而不是 X，F ∈ (X, Y] 是合法的。因为 M 的成员及其非空与否不可见，「F > X ⇒ 违例」会在这类合法镜像上误红。只有 N − 3S = X 时 (X, N − 3S] 为空，而那时 N 自己的槽正是 X 的槽（X + 3S = N），X 已被 N 盖掉，不会作为洞里的旧根现存。所以窗口外有现存旧根 X 时，不确定集 M 一定非空，根环一份镜像给不出红的证书。

**能做的（推荐，乙）：用抬 F 之前那条根的 t 把「上限的最旧有效根」读出来，落在盲带里时报部分没判**

- 抬 F 的根 r 的前一条（同实例、txg 比 r 小的最新那条，checker 本来就找它来认「抬」，`crates/singlefs-checker/src/walk.rs:8800` 一带）如果自己是建账的发布（它树表里记账树根指针的出生 txg 等于它的 txg，I-7.18 的认法），它带的回收门槛 t（根记录偏移 428）= max(它的 F, 环里最旧有效根 txg)。抬 F 的上限「之前的根」那一圈恰是这条前一条根记下 t 时的环，所以当 t 大于它的 F 时，t 就是抬 F 那一刻环里最旧有效根的 txg，记作 O，不管 O 那条根今天还在不在、在窗口里还是洞里。（推的：t 的取法是 `crates/singlefs-core` 里写根时的 `reclaim_floor_when_the_latest_root_was_recorded`，I-7.18 的写者取 t 一行；「环里最旧有效根」的有效口径与上限里的有效口径是否逐字相同，没逐处核。）
- 判法：F ≤ O ⇒ 照今天的窗口口径判，必不比今天松（O 一定不高于整环口径的上限，因为上限 ≥ 最旧有效根）；F > O 且窗口口径判成立（F ≤ 窗口口径的上限）⇒ 这一根报部分没判，理由写「F 越过了抬 F 那一刻环里最旧有效根 O，而 (O, 窗口最旧根] 里被盖掉的根的非空情况不可见」；窗口口径本来就判违例的照红。
- 三条验收：洞里留 X、窗口非空根不足 4、F 抬到 (X, W] ⇒ 部分没判（不红；这是 C717 要的「红」的一半）；F 放回 X 以下 ⇒ 绿；洞里有旧根而 F 没越过 ⇒ 绿。

**要红的话借 journal 证书（丙，只在条件齐时）**：对 [O, r) 里每个没有现存根的 txg u，journal 环里有同一实例的一条记录 u（记录自带新根段，树表指针可比出这一版是否非空）或有证据证明 u 没发布过（实例换届的取号跳过，用 K7 的第四条件加行证据）。全部 u 都有着落时，抬 F 那一刻的环完全可知，用整环口径（`crates/singlefs-core/src/mount.rs:3894` 的 `rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount` 同一个函数的 checker 版）重算，红绿都准。任何一个 u 没着落就退回乙。注意记录单落版本也有记录却没有根（K4 的那一形），记录证明「发布过」不证明「根在环里」，所以要把它按「根不在环里」处理，非空与否照记录的新根段算。这一步比乙重得多（要读多条记录的新根段和树表），建议先落乙，丙另开一批。

**实现落点**：`rollback_floor_ceiling_before_the_raise`（`crates/singlefs-checker/src/walk.rs:8682`，返回 `lowest_possible` / `highest_possible` 的那个函数）旁边加 O 的读出和「落在盲带里」的判断；`judge_rollback_floor_raises_against_their_ceilings` 里不带卸载记号那一支在「窗口口径判成立」之后加一道；带卸载记号那一支同形（它也只取窗口）。GPU：扫描内核 `c_rollback_floor_ceiling`（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1682`）与 `in_the_one_revolution_window` 旁同改，部分没判记射程里判不了、不置位。

**会红的镜像 / 变异**：用 `pool_checker_floor_raise_ceiling_takes_only_roots_inside_the_one_revolution_window.rs` 里的洞历史造：洞里留 X、前一条根的 t 等于 X 的 txg、F 抬到 (X, W]，部分没判；F=X 绿。变异：O 恒取窗口最旧根（退回今天）；O 的取法把 t 当成上限；部分没判改成绿。

**最容易被打穿的地方**：O = t 依赖「前一条是建账的发布」，零单元发布的 t 是抄的，不能用，退回今天的窗口口径；t 里可能带钉（C773），钉值要当一条有效根并进 O，I-7.18 的现算式已有这一口径。

**推翻条件**：造出一张根环里有洞留旧根 X、F 抬到 (X, W]、而 t 读出的 O 不等于 X 的合法镜像（则 O 的解读错了）；或一张合法镜像上 F > O 却整环口径成立（则「部分没判」是多余的但不是误红）。

## K9　C764：新实例第一个 txg 被改坏

**写者怎么取**：`first_txg_of_new_instance`（`crates/singlefs-core/src/mount.rs:1391`）取 max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1，最后一句是 `checkpoint_txg_after_the_version(highest_ring_txg.max(highest_record_txg))`（`crates/singlefs-core/src/mount.rs:1408`）。条款是 D23 已定项 14 第 3 条「新实例…第一次发布的 checkpoint_txg ≥ max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1」（`.claude/kb/decisions/23-journal的角色与格式.md:373`）。

**候选**

| 候选 | 内容 | 问题 |
|---|---|---|
| 甲 | 对每个实例 k，k 的第一个 txg f_k 大于实例代号小于 k 的每条读得出的根与记录的 txg；读得出的口径与挂载读者同一道自证门；有读不出的槽整条报部分没判 | 推荐 |
| 乙 | 第二次抽样的 X-txg：新实例在环里最早那条根的 txg 不大于被抛弃的根 ⇒ 红 | 误报 1,037,260 张（`.claude/kb/checks-owed.md:473` 与作业包 C764 一行） |
| 丙 | 读不出一律不判 | 漏掉「根或记录写进了环、被盖或损坏之外的坏 f_k」，且与 C701 的「读不出一律部分没判」不一致 |

**推荐条文**（新条 I-8.14，落 `03-journal.md`：它是 D23 已定项 14 注 3 的直接判据）：「对每个实例代号 k ≥ 1，设 f_k 为环里读得出的、实例代号 k 的根与记录里最小的 txg；实例代号小于 k 的每条读得出的根与每条自证过的记录，txg 严格小于 f_k。『读得出』与挂载读者同一道门：根要过 I-7.17 的落点公式与 fsid（core 的 `read_root_ring_slot`），记录要过 `scan_journal` 的自证与 I-8.13 的环槽。判到至少一对 (k, 更小的实例) 才算评估；环里有读不出、不自证、落点对不上的根槽或记录槽时，整条报部分没判；没有任何实例有更小的实例可比时报不适用。」

**根与记录在挂载时都读不出那一形，判不判**：分两种读不出。
(1) 镜像上今天仍读不出（槽撕了、位腐）：它不在「读得出」的集合里，不参与比较，没有误红；但它可能藏着本该红的 txg，所以整条报部分没判，不报成立。
(2) 挂载那一刻读不出而今天读得出（瞬时读故障，不留在盘上）：镜像上与坏 f_k 逐字节相同，判不了。这一形与 I-7.9 同一处已认代价（`crates/singlefs-checker/invariants/02-根环.md:16` 写着「读故障不留在盘上」）；带读故障的流（`crash_enumeration_mounts_with_read_faults_stream.rs`、坏盘输入）要在流的登记里写明这条不判（流本身知道注入了哪些读故障），checker 本体不为它开口子。
X-txg 读法误报的主因，我的推断是它用了 checker 自己较松的根列表，而挂载读者的根要过落点公式与 fsid、记录要过环槽；甲把门写成与挂载读者同一道，是堵这一条。原始的 1,037,260 张没有交给我，复核不了。

**实现落点**：`walk.rs` 里在 `check_pool_image` 拿到全部根与 `records_as_the_recovery_reads_them`（`crates/singlefs-checker/src/walk.rs:10526`）之后加 `judge_the_first_txg_of_each_instance_above_the_older_instances`；输入已有：根的实例代号与 txg（`RootView`）、记录的 `instance` 与 `checkpoint_txg`（`ScannedJournalRecord`，`crates/singlefs-checker/src/walk.rs:10140`）。GPU：走读内核里 journal 那一段（`collect_the_records_the_replay_reads`，`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:4842`）之后按实例做两趟（求 f_k，再比）。条目加一条。

**会红的变异（锚在核心）**：`crates/singlefs-core/src/mount.rs:1408` 的 `highest_ring_txg.max(highest_record_txg)` 换成 `highest_ring_txg`（漏记录，记录单落版本的 txg 大于所有根时 f_k 等于它，不严格大）；换成 `.min(highest_record_txg)`；`checkpoint_txg_after_the_version(…)` 的 `+ 1` 换成 `+ 0`。三行写进 `crates/mutations.tsv`，必须红的测试是造「第 1 个实例发布 A、B，B 的记录已落根槽没落，第 2 次挂载」那张流里的 I-8.14。今天 `mutations.tsv` 点名这一句的只有一条溢出变异（`.claude/kb/checks-owed.md:473`）。

**会红的镜像**：同一张三次挂载的流，把第 2 个实例写行那条根与它的记录的 txg 都改成等于 B 的 txg（比 B 小一也行）⇒ 红。另一张：只改根不改记录，仍红（f_k 取两者最小）。

**最容易被打穿的地方**：(a) 门的口径要与挂载读者逐处同；core 的门以后加一条，checker 要跟。(b) 取 `f_k` 为最小值，若实例 k 的第一个根与记录都被盖掉，f_k 被抬高、漏判；条文里写成「最小的读得出的」，这一漏是已知代价。(c) 对「实例代号更小」的限定不能写成「别的实例」：实例代号更大的实例 txg 当然更大。

**推翻条件**：一张写者可达、无瞬时读故障的合法镜像上甲红。

## K10　C70：跨统计量的和

**先划清哪几项镜像上判得出**（对照 D28 已定项 1 的式子 `可用(d) = 容量(d) − 已分配(d) − 不可回收(d) − 挂载期承诺量(d) − 被抛弃根独占量(d) − 待删占用 ÷ 副本数 − 已承诺预留 ÷ 副本数 − checkpoint 保留池 ÷ 副本数 − 环窗口(d)`，`.claude/kb/decisions/28-挂载期承诺量.md:21`；D5 已定项 4 的「式子逐项对照」表，`.claude/kb/decisions/05-快照-空间记账机制.md:99`）：

| 项 | 谁维护 | 镜像上判得出？ |
|---|---|---|
| 容量 | 设备几何 | 判得出（盘字节与系统配置，I-5.2 已在用） |
| 已分配 | 统计量 1 | 判得出（I-3.1、I-3.11） |
| 不可回收 | 统计量 3 | 判得出（第一版非 zoned，应为 0；`walk.rs` 里没有它的常量，今天不读） |
| 待删占用 | 统计量 4，恒 0 | 判得出（I-3.4） |
| 已承诺预留 | 统计量 6 | 判得出：有闭式 `标签 6 = 副本数 × ((2 + H_墓碑 + 1 + 2 + H_退役 + 1) × 16384 + 41 × n_live + 35 × n_记录)`（`.claude/kb/decisions/03-空间分配.md:54`），H 从墓碑树、退役树根节点现读，n_live、n_记录 从 inode 树现数；今天没有人比 |
| 挂载期承诺量、被抛弃根独占量、checkpoint 保留池、环窗口 | 只住内存、挂载或发布时现算 | 判不出；D5 的表自己就把这四项标成「例外」 |

**判不出的项报什么**：这四项是式子里本来就不落盘的量，不是「这张镜像上碰巧读不出」。条文把它们写成射程外，不报不适用也不报部分没判；报部分没判的只留给「射程内的某一行读不出」。这样 README 里「部分没判」的含义（判到的那一部分对得上、射程里另有一部分这张镜像上判不了）不被一个永久的射程外冲掉。

**推荐条文**（新条 I-3.20，I-3.19 给了 K2，落 `05-空间记账.md`）：「逐盘 d：(a) 空闲(d) + 已分配(d) + 不可回收(d) == 容量(d)（I-5.2 加上统计量 3；第一版非 zoned 池统计量 3 的行必须在且为 0）；(b) 最新根记账里标签 6（已承诺预留）等于 D3 已定项 2 的闭式在这一版上现算的值，树表里没有 `deleted_inodes` 条目的版本取 0；射程外：挂载期承诺量、被抛弃根独占量、checkpoint 保留池、环窗口只住内存，由运行时准入自测（D28 已定项 1）和门禁 doc-registries 的 admission-terms 格守，镜像上没有可比的量。(a)(b) 任一行读不出报部分没判，最新根下面没有记账树报不适用。」

**实现落点**：(a) 在 `walk.rs` I-5.2 那一判旁（`judge("I-5.2", …)`，`crates/singlefs-checker/src/walk.rs:20321` 一带）读统计量 3，常量 `STATISTIC_UNRECLAIMABLE_BYTES = 3` 加在 `crates/singlefs-checker/src/walk.rs:314` 一带；(b) 仿 `judge_pending_deletion_occupancy_of_a_version`（`crates/singlefs-checker/src/walk.rs:13007`，I-3.4 的函数，同样逐版读记账树）另写，H_墓碑、H_退役 用 `ckpt_cost` 同一个读法（根节点码 2 头的层级）；GPU 扫描内核 `judge_accounting`（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2541`）里 (a) 是一行加法，(b) 要读墓碑树与退役树根头、数 inode 树记录，工作量大，可以先让 (a) 在 GPU 判、(b) 先 CPU 判并在覆盖表里登记 GPU 暂缺（`crash_judge_conclusions.rs` 的 `POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS` 现在是空表，这样要加一项，主 agent 定行不行）。

**会红的镜像 / 变异**：(a) 统计量 3 的行写成 4096 且空闲少 4096 ⇒ I-5.2 绿、新条也绿，所以要另造「空闲 + 已分配 == 容量 但统计量 3 非 0 且池非 zoned」红，以及「空闲 + 已分配 + 统计量 3 ≠ 容量」红；(b) 标签 6 +1、墓碑树多一层而标签 6 没变各一张，新条红，I-3.1、I-5.2 都绿（独立增量维护的两套 `+=` 对不上，正是 C70 的原话）。变异：(a) 恒成立；(b) 闭式里 41 换 40；(b) 的 n_live 换成 n_记录。

**最容易被打穿的地方**：(b) 的闭式里 n_live 与 n_记录 要从「这一版」的 inode 树现数，删除批进行中间的版本（墓碑已写、记录还在，nlink 为 0）会让 n_live 与记账不同步，D3 已定项 2 写了「写入类发布按发布之后的标签判；删除类发布不判」，所以新条只判写入类发布的那一版（识别方法：这一版的 `logged_ops` 里有未完成的删除意图的版本不判），这一限定要写进条文；(a) 对 zoned 池没有意义。

**推翻条件**：一张写者可达的合法镜像上 (a)(b) 红；或式子里某一项我归错了「谁维护」（对照 D5 的表）。

## 判定一览

| 格 | 判定（本腿对被判对象的读法） | 一句话 |
|---|---|---|
| K1 | 推荐甲 | I-9.8 读成路由区间，由 I-9.12 与 I-9.4 罩住；区间内换号判不出，由 I-9.10、I-9.5 接 |
| K2 | 推荐新条 I-3.19，三支 | T 是根 ⇒ W=0；T 是记录单落版本 ⇒ 从 S 起重放比 W；读不全的行没判，整条部分没判 |
| K3 | 推荐甲（保留实现员那一形） | 同文件 I-8.8 有先例；不适用理由里写「从 1 起」判过且成立 |
| K4 | 推荐甲，另立 I-5.7 | 点名项非空 ⇒ 分配记录树根指针出生 txg 等于这一版 txg；不并进 I-5.4 |
| K5 | 不重钉；LR5 等价测试在 RCW 落地时退役 | 样一、样二两式不等价，样三同值，样四并入样一；样一上两式都对不上账 |
| K6 | 推荐 RCW，θ_low 取 t | 模型：旧等式在 105,812 个合法世界误红，RCW 在 12,681,102 对里 0 误红，泄漏变异全抓 |
| K7 | 推荐 H1-b 加第四条件 | 前提被清扫打破，已给装置与判据；I-3.9 那半边只推到结构 |
| K8 | 只靠根环做不出红；推荐乙（t 读出 O，落盲带报部分没判） | 要红借 journal 证书，另开一批 |
| K9 | 推荐新条 I-8.14 | 读法与挂载读者同一道门；今天读不出的槽整条部分没判；瞬时读故障判不了 |
| K10 | 推荐新条 I-3.20 | 空闲加已分配加不可回收等于容量，标签 6 对闭式；四项只住内存的写成射程外 |

## 复核不了的

- K2、K4、K9 的世界要用真写者流造镜像（checker 档、需要 cargo 与流），这里不起；条文与落点是推的。
- K6 现行等式在「1 MiB 环加大文件顺序写」流上必红、攻方 180 个有洞盘面上改完全红：要真流、真盘面。模型给了同向证据，不是同一份证据。
- K7 的 (P1)(P2)(P3) 动态装置、I-3.9 的 150 / 285 张镜像：同上。
- K9 的 1,037,260 张：原始产物没有交给我。
- K10 (b) 的闭式与写者实际记的标签 6 是否逐字节相等：没有对过真镜像。
- 所有 GPU 一侧的写法（展开大小、对拍）：不碰 GPU，只给落点。

## 没做什么

- 不判别的腿的格；没读攻方腿与 core 两腿这一轮的任何产出。
- 不替主 agent采纳或出判决：上面「推荐」都是本腿的读法。
- 没改仓里任何一个受版本管理的文件（除了这份报告与模型目录）；没跑 cargo、没碰 GPU、没跑重型测试。
- 没写条文进 kb；K1–K10 的「推荐条文」是草稿，落库由主 agent 定。
