# W3：盘上留发号历史，让单镜像判得出同一条时间线上重发 inode 号——设计轮第一轮正文（2026-10-08）

<!-- doc-lint:not-numbers K1 K2 K3 K4 W1 W2 W3 V1 V2 P1 -->

## 一、题面

岔路单 `research/prompts/m3-3-5-forks.md` 第 11 行 W（C479（inode 号复用已发过的号今天造不出来） 剩下的「活记录被原地换」那一形）用户 2026-10-08 定「W3：盘上留发号历史」。依据是 E175（产品路径上 inode 号水位退回与重发号）两段（执行员报告 `~/.local/share/singlefs/work/e175-runner-3/report.md`，sha256 bec47403…；结论句当线索）：
- 产品臂登记的 17 105 段历史里每族 `sum_q1=0`、`segments_q2_one_birth_violated=0`（`research/results/e175-product-path-inode-watermark-regression-2026-10-08/summary-segment-one.out`、`summary-segment-two.out`）；
- 阳性对照臂 V1 在 G-long 上有两次重发判成「看不见」：`g-long-V1.out` 第 389–390 行原样 `E7RESULT name=event family=G-long arm=V1 index=24 kind=q1_same_timeline position=op92 inode=7 new_birth=77 older_births=[51] q9=Gone i9_29_at_step=holds` 与 `… position=op95 inode=8 new_birth=78 older_births=[52] q9=Gone i9_29_at_step=holds`：持有旧出生代的根已经转出回退候选集时重发，I-9.29（同号只许一个出生代） 判成立。

这一轮判：盘上留什么发号历史、留在哪，单镜像上才判得出这一形；它的代价；新不变量怎么写。用户已选 W3，这一轮不再比 W1、W2，只在 W3 之内比形态。

## 二、实现今天的样子（主 agent 的观测，2026-10-08）

- inode 号水位是记账里的一条统计量（D8（核心索引结构） 已定项 6「下一个 inode 号是记账里的一条统计量」，D5（快照 / 空间记账机制） 已定项 4 第 12 项），每条根的记账树只装这条根那一代的行（D5 已定项 2）；崩溃恢复落到较旧的根时水位随所选根那棵账载入，被抛弃时间线里发过的号可以重发（D8 已定项 6 射程；C143（inode 号水位在崩溃恢复落到旧根后退回去重发） 定案）。管理员回退那次发布水位取 max（D23（journal 的角色与格式） 已定项 14）。
- 实例表行（`crates/singlefs-core/src/instance_table.rs` 第 17–25 行）：`kind 1 | 实例代号 4 | 所选根的 checkpoint_txg 8 | 属于该实例的最大已施加事务号 W 8 | flags 1 | 预留 66`，只在恢复与实例切换时写（D18（块里携带什么信息） 已定项 11「行怎么写」）。预留 66 字节今天恒 0。
- I-9.6（水位大于两处最大号） 与 I-9.29 在池级 checker `crates/singlefs-checker/src/walk.rs` 里判；两条都只看回退候选集里还在的那几版。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　留什么、留在哪
至少比这几个候选，各写定义（字段与宽、谁写、什么时候写、什么时候可以删）、落 `crates/` 哪几处、D15（格式冻结政策） 哪一层、每次发布多写多少字节与几个单元、碰哪些崩溃点：
- 候选一：实例表行的预留字节里记「这个实例发过的最大 inode 号」（或这个实例结束时的水位）；
- 候选二：记账里水位行不只留这一代，保留历代（或保留每个实例最后一代）的水位行；
- 候选三：另立一条只追加的发号记录（单元或树），记 (号区间, 出生代) 或 (实例, 最大号)；
- 候选四：腿自己提的别的形态。
判每个候选能不能让 E175 V1 那两次（`g-long-V1.out` 第 389–390 行）在单镜像上判红。

### K2　和「崩溃恢复抛弃的时间线可以重发」怎么共存
- D8 已定项 6 的射程允许崩溃恢复落到较旧的根之后重发被抛弃时间线里的号。证据要分得开「同一条时间线上重发」（要红）与「被崩溃恢复抛弃的那一段里发过、恢复之后重发」（合法）。每个候选靠什么分开（实例表判抛弃、出生代比较……），分不开的写明。
- 管理员回退（挂着时的一次向前发布，水位取 max）、实例切换、实例表行回收（D18 已定项 11「回收」）三处，各候选的历史会不会丢、丢了是误报还是漏报。

### K3　新不变量与 checker
- 写出新不变量（或改 I-9.6、I-9.29 的判据）的文字，checker 在 `walk.rs` 哪里判，读不出那份历史时报什么。
- 一条变异：把那份历史写错或不写，它必须红；E175 V1 的 G-long 那两次必须红。

### K4　代价要不要先量
- 照 `.claude/rules/three-way-inference.md`「岔路交用户定之前，每条路要有代价数」：候选之间若要用户再挑一次，要量什么（每次发布多写的字节、实例表片数随实例数怎么涨、记账树变高与否），写成可交实验设计员的问题单行。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1、K2、K3、K4 | 从条款与代码推出每个候选的定义、落点与代价 |
| 云端攻方（Opus） | K1、K2、K3 | 给每个候选造反例：同一条时间线上重发而它看不见、合法的抛弃时间线重发被它判红、历史在回收或切换时丢掉；用户决定的那几步（什么时候崩、回退到哪、建几个对象）放开扫 |

## 五、跑前写死的判据

- 一个候选被攻方坐实一形漏报（同一条时间线上重发而判不红）或误报（合法重发判红），主 agent 现查坐实，出局。
- 两条腿给出不同的形态：写进判决的交用户表，带各自的代价数；没有代价数的先量再交。
- 「没打中」一次抽样不算数，照 `.claude/rules/three-way-inference.md` 再抽。
- 动格式的定案写回之前交用户。

交付：
- 腿的报告写进 `research/prompts/m3-w3-r1-<腿名>-output.md`，分段落盘：每次写进文件不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 模型与脚本放 `research/prompts/m3-w3-r1-<腿名>-model/`，`rerun.sh` 交回前在空目录里从头跑通一次；仓路径从环境变量 `SINGLEFS_REPO` 取；编译与跑都经 `research/scripts/run-with-memory-cap.sh`，上限显式给（编译 6G、跑 4G），不用 `auto`；同一时刻只起一条 cargo；`compare` 比之前排序、去掉耗时字段、自己不依赖 `SINGLEFS_REPO`；用自己的编译目录，不沿用别的树编过的编译目录。只跑 harness 档。草稿放 `~/.local/share/singlefs/work/<腿名>/`。E175 装置源码原样在 `research/results/e175-product-path-inode-watermark-regression-2026-10-08/device-source.rs`（它是 harness 档测试文件，放进副本的 `crates/singlefs-harness/tests/e175_product_path_inode_watermark_regression.rs` 就能编），可以拷进自己的目录改臂。
- 引 kb 条款写 kb 文件自己的行号；引代码写冻结树里的全路径:行号。
- 方案按 `crates/` 今天的实现来谈。
