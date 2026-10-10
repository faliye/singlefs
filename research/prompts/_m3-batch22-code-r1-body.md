# 删除批第 22 批合入的代码：代码轮第一轮正文（2026-10-05）

<!-- doc-lint:not-numbers N1 N2 N3 N4 N5 N6 N7 N8 N9 N10 N11 N12 K1 K2 K3 T2 T3 T4 T5 T6 T7 T8 T10 T11 T14 -->

## 一、题面

删除批第 22 批两份补丁 2026-10-05 合进主工作区（`records/2026-10-01-删除与并发事务.md` 2026-10-05 那几条）：C 照设计轮判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 与用户三项定案，在池级 checker 上实现三·3 那九格、I-9.18、I-9.29、I-9.4 第四句与 I-9.20 死亡 txg 界；T 关 m3-batch21-code-r1 判决第三节第 1–3 条。这一轮判两件事：代码做的是不是条款、判决与用户定案说的；哪一格会错。

改动全文在 `research/prompts/_m3-batch22-code-r1-diff.md`。冻结副本 `/tmp/claude-1000/m3-batch22-code-r1/tree/`（合入之后的主工作区 `crates/` 与 `Cargo.*`，另含形式化线 32-1、32-2、33 批与根环线 10g 已合的改动），被判的 12 份文件的 sha256 在 `research/prompts/m3-batch22-code-r1-snapshot/crates-sha256.txt`。实现员报告（结论句只当线索，以代码为准）：`/tmp/claude-1000/m3-batch22/{c,t}/report.md`。条款以 `.claude/kb/invariants.md` 那十几行今天的写法为准（2026-10-05 已按判决写回；I-9.29 同日改成「各版合起来」并写进实例表读不出时记判不了）。

| # | 改动 | 主要落点（冻结副本） | 压着的条款或定案 |
|---|---|---|---|
| N1 | `IMPLEMENTED_INVARIANTS` 74 → 83（下标 74–82：I-9.17、I-9.18、I-9.19、I-9.20、I-9.21、I-9.22、I-9.23、I-9.24、I-9.29）；GPU 覆盖表登记九位、内核不置位 | `crates/singlefs-checker/src/image.rs`、`crates/singlefs-checker-tier/src/crash_judge_gpu.rs` | 判决 T13；用户 2026-10-03 定 GPU 判器全覆盖（这一批先登记，第 23 批补内核） |
| N2 | I-1.7 类型 1、类型 5 两种记录宽的坏镜像 | `crates/singlefs-harness/tests/checker_known_bad_images.rs` | `invariants.md` I-1.7 那一行 |
| N3 | I-9.10 墓碑那一个等号：按号认、每版各比自己的 | `crates/singlefs-checker/src/walk.rs` | 判决 T2-a |
| N4 | I-9.17、I-9.23：身份不符单独判定、不再记走读失败；类型段不认得先登记引用再判红；重合小句留合取 | `walk.rs` | 判决 T7-a |
| N5 | I-9.18、I-9.19、I-9.20：「在谱系上」每版从这一版自己的 txg 与树表里的谱系树起算；删除态扫描版本清单加 checkpoint_txg；不按已发布滤；I-9.20 不重号只判谱系上的、区间与「对象出生代 ≤ 死亡 txg ≤ 这一版的 txg」判全部墓碑 | `walk.rs`；`pool_checker_judges_the_deletion_state_of_every_walked_version.rs`、`common_deletion_in_progress/mod.rs` | 判决 T3-a、T4、T14；用户定「可以一起」「补」 |
| N6 | I-9.21、I-9.22：带旋钮值的新入口，旧入口当没装旋钮；坏镜像用「旋钮写出的合法镜像 + 与写时不同的旋钮信号」造 | `walk.rs`、`crates/singlefs-checker/src/lib.rs`、`container_trees_grow_a_level_when_the_root_is_full.rs` | 判决 T5-K1、T6（T6 那一格同日改准：I-9.22 只在墓碑树零根时报不适用） |
| N7 | I-9.24：每条 `deleted_inodes` 条目都判、读队列提到配对之前，干净的一版报成立 | `walk.rs` | 判决 T8-a |
| N8 | I-9.4 第四句：同一片类型 2 容器里记录号严格递增；S4a 那一形只红 I-9.4 | `walk.rs`、`checker_known_bad_images.rs` | 判决 T11 甲 |
| N9 | I-9.29：候选集各版合起来同号只许一个出生代；水位文件用例一、用例二按条款改断言 | `walk.rs`、`inode_number_watermark_pressed_back_reissues_a_number.rs` | 判决 T10 乙；D8（核心索引结构） 已定项 6「同一条时间线上 inode 号不复用」（用户定「写吧」） |
| N10 | `DELETION_QUEUE_SCAN_INVARIANTS` 注释改准；`writable_mount_of_a_formatted_pool.rs` 不适用表与条数 | `walk.rs`、`writable_mount_of_a_formatted_pool.rs` | m3-batch21-code-r1 判决第二节 K1 那一行 |
| N11 | T：N7 纯算用例 r 到 2⁴⁰；N5 比「盘 1 单独恢复择到的根之后每一版换下的槽」、文档改准；盘数断言 | `unlink_writes_per_transaction_do_not_grow_with_the_file.rs`、`the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs` | m3-batch21-code-r1 判决第三节第 1–3 条 |
| N12 | 变异表追加 23 行、替换 1 行（锚点随函数签名变） | `crates/mutations.tsv` | `.claude/singlefs-ai-sop/rules/show-me-test.md`「新增的测试必须先证明它会红」 |

**这一轮不判的**（各有去处，腿不攻）：
- GPU 内核那一侧（九位不置位、I-9.4 第四句与 I-9.10 墓碑等号没判、I-9.17 / I-9.23 走读内核仍记失败）：第 23 批。
- I-9.29 在最新根实例表读不出时可能误红：条款已写「记判不了」，代码第 23 批补；这一轮不当打中。
- checker 档按不变量名单分组计数的用例（`crash_enumeration_new_pool_file_creation_stream.rs` 的 `layer0_quick_tier_matches_the_full_tally_shape`）随新下标要改：第 23 批，钉值提交时取。
- 别的线的改动：由那两条线的代码轮判。

## 二、实现今天的样子（主 agent 的观测，2026-10-05）

- 合入验证 `merge-and-verify.sh`「✓ 测试全绿（169 个测试二进制）」（`/tmp/claude-1000/merge-and-verify/2026-10-06-4185440/`）；fmt 与 `cargo clippy --offline --all-targets --all-features -- -D warnings` 经内存包装单跑都退 0。checker 档没跑。
- 实现员 C 自报：20 行新变异各经 `prove-red.sh` 证红；`cargo build -p singlefs-checker-tier --all-targets` 与带 gpu 特性的 check 都过；新加两段合法历史（「删 → 回退到意图之前 → 再回退到删完」「删 → 回退 → 再删」）全程零违例。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　条款对实现（N1–N12）
逐条从 `invariants.md` 那几行今天的写法、判决格与用户定案推到冻结副本里的行，再推到钉它的测试与变异行：
- 每条新判定判的是不是那一行字面（尤其 I-9.18–I-9.20 的谱系链怎么走、I-9.29 比的是哪几样出生代、I-9.24 读队列的位置）；判不了与不适用各在哪几支，与 `invariants.md` 文件头「判定有四种」对得上没有。
- 每份坏镜像是不是只红目标（断言写的是不是恰好）；每行新变异改坏的是不是名字说的那一处、红在哪条断言。
- N11：判决第三节的三条验收是不是逐条达到。

### K2　哪一格会错（重点 N5、N9、N4、N7）
在冻结副本的拷贝上攻，只跑 harness 档（GPU 不跑）：
- N5、N9：合法删除中间态、管理员回退（回退到意图之前、再回退回删完那一版、删 → 回退 → 再删）、崩溃恢复落到旧根之后重发、崩在屏障段界上，有没有一形误红；违反字面的镜像有没有漏判。用户决定的那几步（文件单元数、unlink 次序、回退到第几新的根、崩溃恢复落到哪条根、水位开关压到哪）放开扫，不写死。
- N4：身份不符的坏镜像是不是只红 I-9.17 / I-9.23，不再连带 I-7.2、I-4.8、I-7.4、I-3.1。
- N7：队列读不出、头 ID 配不上时 I-9.24 报什么。
- N6：旋钮信号与写时不一致之外，有没有合法镜像让 I-9.21 产品读法红。
- 上一轮「没打中（一次抽样）」的第二次抽样：m3-batch21-code-r1 的 N3（删完之后 GPU 分叉，读代码）、N7 写调用随规模变。

### K3　被判的文件名单
`research/prompts/m3-batch22-code-r1-snapshot/crates-sha256.txt` 里那 12 份；冻结副本里每一份都要有一格判到。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1、K3 | 逐条从条款、判决格与用户定案推到代码行、测试与变异行 |
| 云端攻方（Opus） | K2，重点 N5、N9、N4、N7 | 在冻结副本的拷贝上造反例，只跑 harness 档；用户决定的那几步放开扫，不写死 |

## 五、跑前写死的判据

- 两条腿都没打中：判站住。这只是一次抽样，「没打中」照 `.claude/rules/three-way-inference.md` 再抽一次才记。
- 打中：主 agent 现查坐实，改法进下一批；改法算零轮，进下一批的代码轮。
- 误红：一个从入口走得到的已发布状态上报不成立，或合法实现上用例红。漏判：镜像违反 `invariants.md` 那一行的字面而 checker 报成立，或报不适用而镜像上有它的对象；配不上的版报部分没判不算漏判。漏红：一种违反条款或判决的实现上用例照样绿。
- 腿提的新改法算零轮。

交付：
- 腿的报告写进 `research/prompts/m3-batch22-code-r1-<腿名>-output.md`，分段落盘：每次写进文件不超过 150 行，第一段排他新建，回复只写短句。
- 模型与脚本放 `research/prompts/m3-batch22-code-r1-<腿名>-model/`，`rerun.sh` 交回前在空目录里从头跑通一次；仓路径从环境变量 `SINGLEFS_REPO` 取；编译与跑都经 `research/scripts/run-with-memory-cap.sh`，上限显式给（编译 6G、跑 4G），不用 `auto`；`compare` 比摘要之前先把并行测试的 `test … ok` 行排序。草稿放 `/tmp/claude-1000/<腿名>/`。
- 引 kb 条款写 kb 文件自己的行号，去 kb 文件里现查，不从背景材料里数；引代码写冻结副本里的全路径:行号。
- 方案按 `crates/` 今天的实现来谈。
