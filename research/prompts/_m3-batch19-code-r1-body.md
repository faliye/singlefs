# 删除批第 19 批合入的代码：代码轮第一轮正文（2026-10-05）

<!-- doc-lint:not-numbers N1 N2 N3 N4 N5 N6 N7 K1 K2 K3 -->

## 一、题面

删除批第 19 批两份补丁 2026-10-05 合进主工作区（`records/2026-10-01-删除与并发事务.md` 2026-10-05 那几条）：R 关 m3-batch17-code-r1 判决第三节第 3、4 条（Q-F 判别形、N4 用户看得到的结局）、「放开挪到暖机之前」的钉、两份同形搭建换共用模块、C2（无界操作做成单事务） 的 unlink 斜率检查（部分）；S 关 C479（inode 号复用已发过的号今天造不出来） 的只供测试开关与用例一。这一轮判两件事：代码做的是不是条款、判决与用户定案说的；哪一格会错。

改动全文在 `research/prompts/_m3-batch19-code-r1-diff.md`（两份补丁与变异追加行原样）。冻结副本 `/tmp/claude-1000/m3-batch19-code-r1/tree/`（合入之后主工作区的 `crates/` 与 `Cargo.*`，另含根环线 10e、10f 与形式化线第 29 批已合的改动），被判的 8 份文件的 sha256 在 `research/prompts/m3-batch19-code-r1-snapshot/crates-sha256.txt`。实现员报告（结论句只当线索，以代码为准）：`/tmp/claude-1000/m3-batch19/r/report.md`、`/tmp/claude-1000/m3-batch19/s/report.md`。

| # | 改动 | 主要落点（冻结副本） | 压着的条款或定案 |
|---|---|---|---|
| N1 | Q-F 三条判别形（会话里先建一个更晚的 inode 再 unlink、给 2^32 + 7 秒、给上一版 + 7 秒）与三行变异（qfm 写成单处：会话层取给的时间与现行那一版各条记录最晚写入时间的较大者） | `crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs` | D8（核心索引结构） 已定项 6（偏移 64、72、80）；用户 Q-F 乙；m3-batch17-code-r1 判决第二节 K1-N6 那一行 |
| N2 | 第 16 批那条用例补「早于上一版」一形（拆成新用例，原用例改用带参数的搭建） | 同上 | 同上；m3-batch17-code-r1 判决第三节第 3 条 |
| N3 | 「放开挪到暖机之前」的钉：窄盘 176 槽卸载半途崩溃再挂，写行与暖机写下的单元不落在卸载之前那一版换下的槽上；配变异 | `crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs` | D16（发布语义） 已定项 1「正常卸载」那一行；m3-batch17-code-r1 判决第二节 K1-N4 那一行 |
| N4 | N4 用户看得到的结局：窄盘 176 槽，文件长到 36 个单元再缩成 1 个，卸载半途崩溃再挂，第一次写回 36 个单元；删掉暖机之后那一句放开时报 `NoSpaceAfterRaisingTheFloor`（`FloorAlreadyAtTheCeiling`），不删就做成；配变异 | 同上 | 同上；m3-batch17-code-r1 判决第三节第 4 条 |
| N5 | 两份私有搭建换成 `common_deletion_in_progress`（`publish_the_intent`、container_trees 的 `unlink` 照旧私有） | `crates/singlefs-harness/tests/unlink_single_file_publishes.rs`、`container_trees_grow_a_level_when_the_root_is_full.rs` | 第 18 批 P1 报告第九节第 7 条；行为不变 |
| N6 | C2 的 unlink 斜率检查（部分）：1、144、145、432 个单元四点，钉每次发布写数的绝对值，断言每次发布换下的单元 ≤ 144；配「整个文件一批」变异 | `crates/singlefs-harness/tests/unlink_writes_per_transaction_do_not_grow_with_the_file.rs` | `.claude/kb/checks-owed.md` C2 那一行（用户 2026-10-05 同意对象补上 unlink）；斜率那一项四点上分不出，留欠账 |
| N7 | C479 只供测试的开关 `InodeNumberWatermarkSource::PressedBackForTests`：构造拒 0、压的号不小于上一版水位时断言；用例一压回已发过的号再建 inode，I-9.6 只红这一条，关开关零违例；六行变异 | `crates/singlefs-core/src/transaction.rs`（第 233 行起）、`crates/singlefs-harness/tests/inode_number_watermark_pressed_back_reissues_a_number.rs` | `.claude/kb/checks-owed.md` C479 那一行；`.claude/kb/invariants.md` I-9.6 那一行 |

**这一轮不判的**（各有去处，腿不攻）：
- C479 用例二（删掉的号压回重发 I-9.6 拦不住）：设计判断，归三·3 那九格的设计轮作单独一问。
- C2 斜率那一项在四点上分不出、常数是推的：已留欠账，进下一批。
- 同一棵冻结树里根环线、形式化线的改动：由那两条线的代码轮判。

## 二、实现今天的样子（主 agent 的观测，2026-10-05）

- 冻结副本取自合入之后的主工作区。合入验证 `merge-and-verify.sh`「✓ 测试全绿（163 个测试二进制）」（`/tmp/claude-1000/merge-and-verify/2026-10-05-1382690/`）；fmt 与 `cargo clippy --offline --all-targets --all-features -- -D warnings` 经内存包装单跑都退 0；checker 档没跑。
- 13 行新变异的原文在冻结副本里各恰好命中一次（主 agent 现数）。R 的第 3、4 条用例与证红是在 10e 之前那一版 `mount.rs`（1af35923…）上量的，冻结副本里是 10f 之后那一版（6f95b6fb…）。
- 实现员 S 自报：开关两处读点（`publish_new_inodes` 的起号、`publish_admitted` 写进记账的水位）在任何落盘之前、读同一个开关与同一份上一版。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　条款对实现（N1–N7）
逐条从条款、判决格与用户定案推到冻结副本里的行，再推到钉它的测试与变异行：
- N1、N2：每条用例断言的是不是盘上三个时间秒等于给的值；qfm 的单处写法与攻方原来那种「会话里时间只进不退」是不是同一类错；三行变异改坏的是不是名字说的那一处。
- N3、N4：断言判的是不是用户看得到的结局；变异下红在哪一条断言（还是在 core 里 panic）。
- N5：换成共用模块前后，两个测试二进制判的东西是不是一样。
- N6：断言「每次发布换下的单元 ≤ 144」与 C2 的「斜率为 0」是不是同一件事。
- N7：开关的类型是不是让产品路径走不到它；两处读点是不是读同一份；用例一两边各断言了什么。

### K2　哪一格会错（重点 N3、N4、N7）
在冻结副本的拷贝上攻，只跑 harness 档：
- N3、N4：在 10f 之后的 `mount.rs` 上，这两条用例与它们的变异还红不红在原来那条断言；把用户决定的几步（单元区槽数、覆盖写次数、文件长到多少单元再缩、卸载崩在哪个写之后）放开扫，用例会不会在合法实现上误红、在错实现上漏红。
- N7：开关会不会被产品路径（挂载、续做、管理员回退）读到；压回的号与水位、墓碑、回退的组合里，有没有一形让用例一的「关开关零违例」不成立。
- N1、N6：有没有一种错实现让这几条用例照样过。

### K3　被判的文件名单
`research/prompts/m3-batch19-code-r1-snapshot/crates-sha256.txt` 里那 8 份；冻结副本里每一份都要有一格判到。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1、K3 | 逐条从条款、判决格与用户定案推到代码行、测试与变异行 |
| 云端攻方（Opus） | K2，重点 N3、N4、N7 | 在冻结副本的拷贝上造反例，只跑 harness 档；用户决定的那几步放开扫，不写死 |

## 五、跑前写死的判据

- 两条腿都没打中：判站住。这只是一次抽样，「没打中」照 `.claude/rules/three-way-inference.md` 再抽一次才记。
- 打中：主 agent 现查坐实，改法进下一批；改法算零轮，进下一批的代码轮。
- 误红：一个合法实现上用例红。漏红：一种违反条款或定案的实现上用例照样绿。判不了的不算漏红。
- 腿提的新改法算零轮。

交付：
- 腿的报告写进 `research/prompts/m3-batch19-code-r1-<腿名>-output.md`，分段落盘：每次写进文件不超过 150 行，第一段排他新建，回复只写短句。
- 模型与脚本放 `research/prompts/m3-batch19-code-r1-<腿名>-model/`，`rerun.sh` 交回前在空目录里从头跑通一次。草稿放 `/tmp/claude-1000/<腿名>/`。
- 引 kb 条款写 kb 文件自己的行号，去 kb 文件里现查，不从背景材料里数；引代码写冻结副本里的文件:行号（写全路径）。
- 方案按 `crates/` 今天的实现来谈。
