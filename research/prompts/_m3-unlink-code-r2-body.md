# 删除批第 11 批合入的代码：代码轮第二轮正文（2026-10-03）

<!-- doc-lint:not-numbers N1 N2 N3 N4 N5 N6 U1 U2 U3 U4 U5 U6 U8 U9 U10 U11 U12 U13 U14 LR1 LR2 LR3 LR4 LR5 LR6 LR7 -->

## 一、题面

删除批代码轮第一轮（`research/prompts/m3-unlink-code-r1-main-verification.md`）与遗留代码轮第一轮（`research/prompts/m3-leftover-code-r1-main-verification.md`）打中的改法，由第 11 批两位实现员写进 `crates/`，合入主仓（`records/2026-10-01-删除与并发事务.md` 2026-10-03 那几条）。第一轮判决第 45 行写明「U9–U14 被攻过零轮，改完进代码轮第二轮」。这一轮判：代码做的是不是判决与用户定案说的，哪一格会错。

改动全文在 `research/prompts/_m3-unlink-code-r2-diff.md`（A、B 两份补丁原样，加变异表的差），冻结副本 `/tmp/claude-1000/m3-unlink-code-r2/tree/`，被判 21 份文件的 sha256 在 `research/prompts/m3-unlink-code-r2-snapshot/crates-sha256.txt`。两位实现员的报告：`/tmp/claude-1000/m3-batch11/a/report.md`、`/tmp/claude-1000/m3-batch11/b/report.md`，报告里的结论句是线索，以代码为准。

| # | 改动 | 主要落点（冻结副本） | 压着的条款或定案 |
|---|---|---|---|
| N1 | U14：续做推满抬 F 仍放不下时挂载照样交回可写、队列留着；新类型 `DeletionQueueContinuationOutcome`，`MountOutput` 加字段 | `crates/singlefs-core/src/mount.rs` 第 908–925 行、`continue_the_deletion_queue_until_it_is_empty`（第 4382 行起） | D8（核心索引结构） 已定项 16 第 6 条；D16（发布语义） 已定项 1「准入」那一行；用户 2026-10-03 定案「照 D16 做成可写」（判决第 64 行） |
| N2 | U9：续做里不是空间不够的错带上这次挂载已落盘各次发布的写账 | `mount.rs` 第 4273 行起 `writes_of_the_publishes_before_the_continuation`、第 4437 行起 | `PublishSequenceFailed` 的契约（`mount.rs` 第 720–732 行）；判决第三节 U9 |
| N3 | U10、U11：`NoFileVersionToChange` 带走空发成的几版与推的抬 F；随机历史执行器不再跳过删除中的版本（覆盖写、unlink、建 inode），模型加拒绝理由 `OnlyNewInodesOnAVersionWithTreesAndNoFile` | `crates/singlefs-core/src/mounted_session.rs`；`crates/singlefs-harness/src/history.rs`、`model.rs`、`model_comparison.rs`；`crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs` 一行穷举臂 | D8（核心索引结构） 已定项 16 第 5、6 条；判决第三节 U10、U11 |
| N4 | 用例与变异：U2（多片下段叶删除，产品批大小）、U3、U5（取号前落点拒、0 写）、U6、LR2（两片实例表 C393 现算）、LR3（影子账被拒走到 `isolate_slots_referenced_only_by_abandoned_roots` 交 `Err`）、LR4、LR6（吞取号写造孤记录盘面）；删空 `KNOWN_RED_FORMS`（收口表第 43 行） | `crates/singlefs-harness/tests/` 下十份（diff 文件第一节点名）；`history.rs` | 两份判决第三节各行 |
| N5 | 实现员 B：`crash_injection.rs` 的抬 F 界认续做里推的抬 F（按位置取，取不出交 `NotLocatedInTheMountWrites`）；U12 续做里交回错时按写账划续做段；LR7 写零折叠两条单测；剩余七项两批在今天底座上重做（`CrashPointDraw::ExactlyOneCrashState` 与三条靶向用例、回退 D 段序列钉 24+2+1+2、`transaction.rs` 一处注释、零单元发布那条用例、C592 floorreuse 用例） | `crates/singlefs-checker-tier/src/crash_injection.rs` 第 1585–1666、1792 行起；`device_log.rs`；`tests/crash_injection_campaign.rs`；`crates/singlefs-core/src/transaction.rs`；`crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs`、`named_unit_verification_stands_in_for_the_barrier_before_the_record.rs` | D8（核心索引结构） 已定项 16 第 6 条；形式化接入那条线定的「F 的界按挂载交回的抬 F 那几串在写表末尾的位置取，取不出交 `NotLocatedInTheMountWrites`、整段 F 记不判，不许改回按内容认」（`records/2026-10-01-删除与并发事务.md` 第 376 行）；判决第三节 U12、遗留判决第三节 LR7 |
| N6 | 变异表：追加 31 行、替换 7 行、删 4 行（120、156、746、2128） | `crates/mutations.tsv`（diff 文件第三节） | 改钉值报告 `/tmp/claude-1000/m3-batch11/pins/report.md` |

**不在这一轮判的**（各有去处，腿不攻）：U1（用户 2026-10-03 定记欠账）；U4 与挂载透明、挂载之后那一次发布（实现员 B 报告第五节 ①，用户已定「续做算挂载的一部分」，归下一批）；U8（要改 `transaction.rs` 见证读一带，归下一批）；U13（模型罩多单元文件，归下一批）；续做推满之后正常卸载做不成（另开设计轮 `m3-unmount-after-short-continuation-r1`）；改名带进 crates 的注释与 `#[ignore]` 理由（18 份，主 agent 逐条核过只换门禁名）。

## 二、实现今天的样子（主 agent 的观测，2026-10-03）

- 冻结副本取自 2026-10-03 主工作区：第 11 批 A、B 两份补丁与改钉值已合入。合入验证 `merge-and-verify.sh`（日志 `/tmp/claude-1000/m3-batch11/merge-verify/`）：harness 档 126 个测试二进制全绿、fmt 过、clippy 过；checker 档没跑。
- 主 agent 现查 U12 的下标假设成立：`mount.rs` 第 4437 行起的写账以 `writes_of_the_publishes_before_the_continuation` 打头，它第 0 项是 `row_publish`（第 4273 行起）。
- 实现员 A 报告第五节第 6 问自报：续做里推过抬 F 之后再出错时，每一串先写系统配置那一步的写不在错里；抬 F 自己报错那一支原样交回、不带这次挂载已落盘的写。实现员 B 报告第五节 ③ 自报：续做开头先被空间拒、先推的那一串抬 F 在划段时归暖机；接线那一处没有用例钉。
- 实现员 A 报告第五节第 5 问：LR5 那一刀推为等价变异，没跑、没写留档测试，推翻条件写在那里。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　条款对实现（N1–N3、N5）
逐条从判决行或条款原文推到冻结副本里的行，再推到钉它的测试与变异行：N1 的结局与文档是不是用户定案那一句；N2 的写账条数与契约「按先后（可写挂载写行那次在最前）」对不对得上，A 自报的两处缺口算不算违反契约；N3 执行器不跳过之后，模型与实现在删除中的版本上逐种用户改动是不是同一个结局；N5 的界是不是按位置取、没有退回按内容认，U12 划段的「只有续做才写的结构」五种取法对不对。

### K2　哪一格会错（N1–N6）
在冻结副本的拷贝上攻：N1 挂载交回可写之后的会话（每次用户改动先走空）有没有一段合法历史给出与模型不同的结局；N2 的写账与设备一层数到的写对不上的形；N3 执行器改动之后随机历史里走得到、而模型答错的形；N4 每条新用例改坏点名的那一行红不红、有没有错实现照样过；删空 `KNOWN_RED_FORMS` 之后有没有一形从「已知红」变成没人报；LR5 的等价论证造不造得出反例；N5 的界在续做与准入都推过时取得对不对；N6 删掉的四行守的行为今天是不是真没了、有没有别的行守着。

### K3　被判的文件名单
`research/prompts/m3-unlink-code-r2-snapshot/crates-sha256.txt` 那 21 份；冻结副本里每一份都要有一格判到。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1、K3 | 逐条从判决行与条款原文推到代码行、测试与变异行 |
| 云端攻方（Opus） | K2，重点 N1、N3、N5 | 在冻结副本的拷贝上造反例，只跑 harness 档；用户决定的那几步（文件单元数、unlink 时机、覆盖写次数、崩在哪一批）放开扫，不写死 |

## 五、跑前写死的判据

- 两腿都没打中 ⇒ 站住（一次抽样，「没打中」照 `.claude/rules/three-way-inference.md` 再抽一次才记）。
- 打中 ⇒ 主 agent 现查坐实，改法进下一批；改法算零轮，下一批合入后进第三轮。
- 腿提的新改法算零轮。

交付：腿的报告写 `research/prompts/m3-unlink-code-r2-<腿名>-output.md`，分段落盘（每次写进文件不超过 150 行，第一段排他新建，回复只写短句）；模型与脚本放 `research/prompts/m3-unlink-code-r2-<腿名>-model/`；草稿放 `/tmp/claude-1000/<腿名>/`。引 kb 条款写 kb 文件自己的行号（去 kb 文件里现查，不从背景材料里数）；引代码写冻结副本里的文件:行号。方案按 `crates/` 今天的实现来谈。
