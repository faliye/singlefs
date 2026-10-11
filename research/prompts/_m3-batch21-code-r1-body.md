# 删除批第 21 批合入的代码：代码轮第一轮正文（2026-10-05）

<!-- doc-lint:not-numbers N1 N2 N3 N4 N5 N6 N7 N8 N9 N10 K1 K2 K3 S1 G1 X1 -->

## 一、题面

删除批第 21 批四份补丁 2026-10-05 合进主工作区（`records/2026-10-01-删除与并发事务.md` 2026-10-05 那几条）：A 关 m3-batch20-code-r1 判决第三节第 1、3、4 条；G 关同一判决第三节第 2 条与第 20 批 G 交回时留下的两件；R 关 m3-batch19-code-r1 判决第三节第 1–4 条与 C479（inode 号复用已发过的号今天造不出来） 变体 c；W 关「四个固定点单元」那一族旧说法与 C479 开关在没有上一版的发布上的那一格。这一轮判两件事：代码做的是不是条款与判决说的；哪一格会错。

改动全文在 `research/prompts/_m3-batch21-code-r1-diff.md`（四份补丁与变异追加、替换、删除行原样）。冻结副本 `/tmp/claude-1000/m3-batch21-code-r1/tree/`（合入之后主工作区的 `crates/` 与 `Cargo.*`），被判文件的 sha256 在 `research/prompts/m3-batch21-code-r1-snapshot/crates-sha256.txt`。实现员报告（结论句只当线索，以代码为准）：`/tmp/claude-1000/m3-batch21/{a,g,r,w}/report.md`。

| # | 改动 | 主要落点（冻结副本） | 压着的条款或定案 |
|---|---|---|---|
| N1 | S1：extent 树那一侧也做反方向配对，每棵 extent 树要恰好一棵 inode 树的头 ID 指着它，配不上记判不了（新成员 `ExtentTreeHeadIdentifierNamesNoSingleInodeTree`）；S1 那一形做成坏镜像 | `crates/singlefs-checker/src/walk.rs`、`crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs`、`common_deletion_in_progress/mod.rs` | 设计轮判决 `research/prompts/m3-delete-checker-r1-main-verification.md` 第二节 F4-b；m3-batch20-code-r1 判决第二节 S1 那一行 |
| N2 | `walk.rs` 那句「号的唯一性交给 I-9.4、I-1.1」改准；`AccountingTreeUnreadable` 补用例；`lib.rs`「全仓这一处定义」改准 | `walk.rs`、`crates/singlefs-harness/src/lib.rs` | m3-batch20-code-r1 判决第二节 K1、S4a 两行 |
| N3 | N5：GPU 恢复内核放过上段根层级 0 的零条目 extent 树（删完那一形），与 CPU 同判 | `crates/singlefs-checker-tier/src/crash_judge_common.wgsl` | `crates/singlefs-core/src/recovery.rs` 删完那一支；D8（核心索引结构） 已定项 16 第 5 条 |
| N4 | 对拍补一条删到最后一批的流 `deletion-through-the-last-batch`；6 条对拍用例改名连带 11 行变异替换；扫描内核与共用内核两个同值 nlink 常量合一 | `crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`、`crash_judge_checker_scan.wgsl`、`crash_judge_common.wgsl` | m3-batch20-code-r1 判决第三节第 2 条；用户 2026-10-03 定 GPU 判器全覆盖 |
| N5 | 卸载崩在暖机要推两次的那一刀：新用例断言写行与两次暖机都不写进卸载前那一版换下的槽；两刀变异（第一次暖机后就放开、只在真发处放开）；第 725 行三元组断言补变异；N4 文档写明准入关着才走得到与单元数的判别力边；N3 注释出处 | `crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs` | D16（发布语义） 已定项 1「正常卸载」那一行；m3-batch19-code-r1 判决第二节 K2 N3 / N4 那一行 |
| N6 | 同一次挂载里两次 unlink、后一次时间更早的用例；qfm 删旧行、加新写法（只记建 inode 给过的时间） | `crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs` | D8（核心索引结构） 已定项 6；m3-batch19-code-r1 判决第二节 K2 N1 那一行 |
| N7 | C2：每批换下恰好 min(144, 剩下)；快档加 433 个单元一点、慢档 2304、4608 两点；常数改由 `admission::ring_window_on_one_device` 现算（上界 748），钉写调用数绝对值 | `crates/singlefs-harness/tests/unlink_writes_per_transaction_do_not_grow_with_the_file.rs` | `.claude/kb/checks-owed.md` C2 那一行；m3-batch19-code-r1 判决第二节 K1 N6 那一行 |
| N8 | C479：文件头那句改准（判不出的区域是 p + n 大于两处最大号）；变体 c 压到低于墓碑的号，先抬 F 让旧根出候选集再看墓碑那一半 | `crates/singlefs-harness/tests/inode_number_watermark_pressed_back_reissues_a_number.rs` | `.claude/kb/checks-owed.md` C479 那一行；`.claude/kb/invariants.md` I-9.6 那一行 |
| N9 | C479 开关遇没有上一版的发布当场 panic（`watermark_of_a_publish_without_a_previous_version`），core 库单测 2 条加一份新 harness 用例 | `crates/singlefs-core/src/transaction.rs`、`crates/singlefs-harness/tests/inode_number_watermark_switch_on_a_publish_without_a_previous_version.rs` | `.claude/kb/checks-owed.md` C479 那一行 |
| N10 | 「四个固定点单元」那一族旧说法只改注释与消息串（写行每盘 9 个单元、暖机与抬 F 空发布每盘 8 个） | `transaction.rs`、`crates/singlefs-harness/src/{model,history}.rs`、若干 harness 测试、`crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs` | `.claude/kb/layout/01-first-txn.md` 第八节 |

**这一轮不判的**（各有去处，腿不攻）：
- G 交回时提的三件：只在坏盘上才有的两处 GPU / CPU 分叉（零条目层级 0 的 extent 上段根与 inode 树根，GPU 按条目宽拒、CPU 不核）、扫描内核与共用那一半 size 偏移两处定义、层级条件没有变异：进下一批。
- R 交回时停下的四件：C2 更紧的常数（要推「一批碰分配记录树几片叶」的上界）、「斜率为 0」怎么读、checker 跨根取最大 key 且只留第一处违例、锚点相交：前两件进 C2 欠账，第三件在设计轮 m3-delete-checker-r2 的 T12。
- 三·3 九格与两问：设计轮 m3-delete-checker-r2 判。
- 别的线的改动：由那两条线的代码轮判。

## 二、实现今天的样子（主 agent 的观测，2026-10-05）

- 冻结副本取自合入之后的主工作区。合入验证 `merge-and-verify.sh`「✓ 测试全绿（166 个测试二进制）」（`/tmp/claude-1000/merge-and-verify/2026-10-05-3262092/`）；fmt 只红在形式化线改到一半的 `e156_allocation_basis_counts.rs`，`cargo clippy --offline --all-targets --all-features -- -D warnings` 经内存包装单跑退 0。
- 合入前在副本上试打四份都打得上；合并后变异表 2759 行，每行原文在源码里恰好命中一次（主 agent 现数）。
- checker 档没跑：GPU 对拍、G 与 A 的变异行、新流的耗时都归提交时。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　条款对实现（N1–N10）
逐条从条款、判决格与用户定案推到冻结副本里的行，再推到钉它的测试与变异行：
- N1、N2：extent 那一侧的配对与 inode 那一侧（第 20 批）是不是同一套两方向；配不上时 I-9.5、I-4.7、I-4.6 各报什么，与 `invariants.md` 文件头「判定有四种」对得上没有；改准的两句是不是今天的事实。
- N3、N4：GPU 那一判与 `recovery.rs` 删完那一支逐支对；新流是不是真走到最后一批之后的版；11 行变异替换后点名的是不是同一条用例。
- N5–N8：每条新用例断言的是不是名字说的那一件；每行新变异改坏的是不是名字说的那一处、红在哪条断言；N7 的常数是不是从代码算出来的。
- N9、N10：panic 的条件与开关的两处读点对不对得上；改过的注释与 layout/01 第八节逐处对。

### K2　哪一格会错（重点 N1、N3、N5、N7）
在冻结副本的拷贝上攻，只跑 harness 档（GPU 不跑，GPU 那一侧读代码、用 CPU 算期望）：
- N1：两方向都配上、违例照旧漏判的树表形态还有没有（inode 树与 extent 树交叉指、一棵树被两条条目指着）；合法删除中间态上会不会误报判不了；用户决定的几步（文件单元数、批大小、unlink 对象与次序、进程里 / 会话 / 挂载续做、管理员回退）放开扫。
- N3：零条目根之外，删完之后还有没有一形 CPU 答「没有文件」而 GPU 不答。
- N5、N6：错实现能不能让新用例照样过；卸载崩在哪个写之后、覆盖写几次放开扫。
- N7：错实现（每批换下数随规模变、写调用数随规模变）能不能让三点照样过。
- 上一轮「没打中（一次抽样）」的第二次抽样：m3-batch20-code-r1 的 N1 合法态误报、N3、N5 其余几形、N9；m3-batch19-code-r1 的 N3、N4 合法实现误红、N7 产品路径与关开关零违例。

### K3　被判的文件名单
`research/prompts/m3-batch21-code-r1-snapshot/crates-sha256.txt` 里那几份；冻结副本里每一份都要有一格判到。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1、K3 | 逐条从条款、判决格与用户定案推到代码行、测试与变异行 |
| 云端攻方（Opus） | K2，重点 N1、N3、N5、N7 | 在冻结副本的拷贝上造反例，只跑 harness 档；用户决定的那几步放开扫，不写死 |

## 五、跑前写死的判据

- 两条腿都没打中：判站住。这只是一次抽样，「没打中」照 `.claude/rules/three-way-inference.md` 再抽一次才记。
- 打中：主 agent 现查坐实，改法进下一批；改法算零轮，进下一批的代码轮。
- 误红：一个从入口走得到的已发布状态上报不成立，或合法实现上用例红。漏判：镜像违反 `invariants.md` 那一行的字面而 checker 报成立，或报不适用而镜像上有它的对象；配不上的版报部分没判不算漏判。漏红：一种违反条款或判决的实现上用例照样绿。
- GPU 那一侧：读代码找到一支与 CPU 不同、并用 CPU 算出期望写清 GPU 会给什么，算打中；GPU 不跑。
- 腿提的新改法算零轮。

交付：
- 腿的报告写进 `research/prompts/m3-batch21-code-r1-<腿名>-output.md`，分段落盘：每次写进文件不超过 150 行，第一段排他新建，回复只写短句。
- 模型与脚本放 `research/prompts/m3-batch21-code-r1-<腿名>-model/`，`rerun.sh` 交回前在空目录里从头跑通一次；仓路径从环境变量 `SINGLEFS_REPO` 取，不按目录层数推。编译与跑都经 `scripts/hosts/run-with-memory-cap.sh`，上限显式给（编译 6G、跑 4G），不用 `auto`。草稿放 `/tmp/claude-1000/<腿名>/`。
- 引 kb 条款写 kb 文件自己的行号，去 kb 文件里现查，不从背景材料里数；引代码写冻结副本里的全路径:行号。
- 方案按 `crates/` 今天的实现来谈。
