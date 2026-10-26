# m4-gpugc-design-r1：里程碑四七·2 GPU 垃圾回收设计轮，第一轮正文（2026-10-23）

<!-- doc-lint:not-numbers G1 G2 G3 G4 G5 G6 G7 Q1 Q2 Q3 Q5 Q6 Q7 Q9 Q10 Q13 Q16 Q17 F7 -->

## 一、题面

`.claude/kb/milestone/04-fourth-txn.md`「七　GPU 垃圾回收」作业点表七·2 那一行要这一轮定下面七件事。七·1 的量是 E185（GC一批搬迁的挂钟拆分），已跑完第 2 次重跑，问题单 `research/prompts/m4-gpugc-measure-forks.md` 四行都已够判。这一轮把四行的读数连同 D26（后台整理与放置回收） 已定项 13 的八条、D24（后台重活能不能卸给 GPU） 已定项 2 的三条硬约束一起推，给出每件事的候选与推荐。主 agent 写判决，交用户定的岔路另写岔路单。

用户的前提（原话在决策变更史 D26 节 2026-10-19 那一块，第 14611–14628 行，附录里有）：「gpu垃圾回收需要加入。不需要讨论。」，即 GPU 模式要做，不在这一轮论证做不做。值不值按 D26 已定项 13 第 7 条算：看省下的主机 CPU 时间，挂钟与 CPU 模式持平也做。

## 二、实现今天的样子（主 agent 的观测，2026-10-23）

- GC 路径都在 CPU 上：
  - 整理入口 `crates/singlefs-core/src/mounted_session.rs` 第 1956 行 `run_compaction`，第 2588 行 `run_compaction_on`；
  - 区域扫描 `crates/singlefs-core/src/region_scan.rs` 第 204 行 `scan_region`；
  - 选段与计划 `crates/singlefs-core/src/compaction_plan.rs` 第 88 行 `choose_region`、第 221 行 `plan_batch`；
  - 发布 `crates/singlefs-core/src/transaction.rs` 第 9675 行 `publish_relocation_batch`。
- 产品代码里没有 GPU 依赖：`wgpu` 只在 `crates/singlefs-checker-tier/Cargo.toml` 第 27 行，特性 `gpu` 在第 40 行。`grep -rln 'background_compute\|BackgroundCompute' crates/ scripts/` 零命中。
- 挂载走库调用：`crates/singlefs-core/src/mount.rs` 第 8043 行 `mount_writable`，第 8055 行 `mount_writable_with_space_admission`。没有挂载参数这一层，命令行要到里程碑十的 FUSE。整理三水位住系统运行配置：`crates/singlefs-core/src/system_configuration.rs` 第 415 行 `compaction_watermarks`。
- checker 档已有的 GPU 做法（测试装置，不是产品）：
  - 崩溃判器用 WGSL 经 `wgpu 30.0.1` 起，`crates/singlefs-checker-tier/src/` 下有 6 份 `.wgsl`；
  - 显卡的先后与额度读多机配置的 `GPU_CARDS_BY_PRIORITY`：`crates/singlefs-checker-tier/src/gpu_card_configuration.rs` 第 74 行 `parse_cards_by_priority`、第 176 行 `cards_by_priority_from_configuration_or_environment`。七·1 那一行写明产品不读这份配置。
- 七·1 的读数：E185 实验页 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md`「问题单逐行（岔路表）」一节，四行原样在背景材料的附录里。摘四行的结论，数以实验页为准：
  - 第 1 行：一批的挂钟随池大小翻面。P = 1024 的六格 Q1、Q2 都判 `both`，P = 8192 的六格都判 `cpu`；不随段内活槽占比 f 翻，也不随批大小 K 翻。
  - 第 2 行：甲（校验和核对）、乙（区域扫描里的单元头解析与映射反查）、丙（选段打分）三段都判 `not_worth`，每段份额都不到一批 CPU 的 10%。原登记 F7 打中：CPU 的大头在读这一版与发布。Q9（第 1 次重跑 72 行全 `gpu_loses`）算不算翻面量，交这一轮定。
  - 第 3 行：Q13 56 行全判 `merge_batches`，φ 最小 0.911860，两张卡同判：一批一交（甲）出局，乙（连续几批攒起来一次交）与丙（放宽一条意图的 R）之间七·1 只给了 n* 与每批字节。
  - 第 4 行：Q16 `judgement=single_run_enough`（第 1 次重跑 0.011019，这一趟 0.015977），并发那一项（Q17）没跑。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### G1　GPU 上算哪几段
候选：问题单第 2 行的甲、乙、丙，F7 提的「发布那一半」，以及别的拆法。要答：
- 在 D26 已定项 13 第 7 条（省下的主机 CPU 时间）这把尺下，每个候选能省多少。
- 三段都不到 10% 时，GPU 模式省下的 CPU 时间从哪来；只放甲、乙、丙三段的话，准备、提交、等结果、两份比对、交 CPU 判这几样加起来，会不会比省下的还多。
- 发布那一半里有没有能放上 GPU、又不碰 D24 已定项 2 第 2 条（不许进前台路径）与 `.claude/rules/fs-design.md`「一个事务层，所有结构共用」的部分。
- Q9 要不要算翻面量。

### G2　攒批的形态与显存上限
在问题单第 3 行的乙（连续几批攒起来一次交）与丙（放宽 R）之间选。要答：
- 两者各自怎么受 `background_gpu_memory_limit`（上限，不是预留）管。
- 攒批中途要不要重扫：前一批发布改了这一版，后一批的输入怎么办。
- 一批之内不换模式（`.claude/rules/fs-design.md`「分支变量不许在操作中途改变」），在两种形态下各怎么落。

### G3　多张卡怎么分活
产品不读 `multi-host.env`。要答：
- 不给 `background_gpu_devices` 时，本机全部卡按什么排、按什么切。
- 一张卡与几张卡判定相同，怎么保证。

### G4　两份 GPU 核比对的形态与 N
D26 已定项 13 第 8 条：两份独立写的核（不共用核函数），各算一次再比对。要答：
- 「独立」的判据：可以共用和不许共用的分别是什么（输入表的准备、驱动与着色器编译器）。
- 一张卡时两份都在这张卡上跑，几张卡时怎么分；不许要求多张卡。
- 比对放在 GPU 上还是 CPU 上。
- 连续几轮不确定才交 CPU 判的 N 取几、依据是什么。

### G5　控制接口（挂着切换）的形态
一组挂载参数管全部后台任务；后台任务种类与计算后端各做成封闭枚举。要答：
- 切换经已挂载会话的哪一种调用进来，从下一批生效的边界落在代码哪一处。
- 里程碑七的 GPU 打包搬迁怎么只加一个枚举成员就接上。

### G6　三个默认值
定时兜底间隔、写带宽上限、显存上限，各给默认值与依据。要答：
- 问题单第 4 行 Q16 判单跑就够之后，写带宽上限从什么量取。
- 兜底间隔与 E10（老化：结构抗老化 vs 事后整理）、C412（整理的挂钟代价没量过）的关系。
- 显存上限不给时「能用多少用多少」，有没有要另设的保护。

### G7　GPU 代码住哪、用什么写
推的落点是单开一个可选的包，`singlefs-core` 不带它也能编（D24 已定项 2 第 1 条）。要答：
- 这个包与 `singlefs-core` 的依赖方向。
- CPU 参照住哪：D24 已定项 2 第 3 条要求两份实现。
- WGSL 经 `wgpu` 之外，有没有更好的做法值得比。
- 与 checker 档已有的 GPU 代码共不共用：判器与产品不许共用同一段代码的那一类，以 D13（验证路线） 为准。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | G1–G7 | 从 D26 已定项 13、D24 已定项 2 与 E185 四行读数正着推：每件事给候选，逐条写推荐与依据，依据写到条款的 kb 文件:行号与产物整行。G1 要拿实验页里的份额算一遍「省下的 CPU 时间」的上界，写明每个数取自哪一行 |
| 云端攻方（Opus） | G1–G7 | 找每条推荐最脆弱的地方：<br>G1 找一种拆法，让 GPU 模式按第 7 条那把尺是赔的；<br>G2 找乙或丙会撞显存上限、或让「一批之内不换」落空的序列；<br>G4 找两份「独立」的核仍共用同一个错误前提的写法；<br>G6 找默认值会让前台 fsync 让路失效的负载。<br>能用算术或小模型核的，在自己的草稿目录里核，报告写命令与输出 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写 `crates/` 下的路径与行号（主工作区，开工快照在 `research/prompts/m4-gpugc-design-r1-snapshot/sha256sums.txt`）。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出；要编译的经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 16G 的上限，用自己的编译目录，跑完删。不用第二台。
- 报告写进 `research/prompts/m4-gpugc-design-r1-<模型>-output.md`，分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 交岔路的地方写成表：候选（各自的定义）、翻面观测、够判条件；不写倾向。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不起），不跑 checker 档测试，不起 GPU。
