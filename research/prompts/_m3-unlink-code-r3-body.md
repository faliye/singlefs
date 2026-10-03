# 删除批第 12 批合入的代码：代码轮第三轮正文（2026-10-03）

<!-- doc-lint:not-numbers N1 N2 N3 N4 N5 N6 N7 N8 V1 V2 V3 V4 V5 V6 U4 U8 U13 LR1 LR5 Q1 Q2 Q3 Q4 MU1 MU2 -->

## 一、题面

删除批代码轮第二轮（`research/prompts/m3-unlink-code-r2-main-verification.md`）判出的改法 V1–V6，加上同一批里几样已经定案的改动：用户 2026-10-03 定的丙1（`research/prompts/m3-unmount-after-short-continuation-r1-main-verification.md`、岔路单 `research/prompts/m3-unmount-after-short-continuation-r1-forks.md`）、Q-F「算改动，改实现」、挂载透明「续做算挂载的一部分」，以及第一轮判决留下的 U4、U8、U13 与遗留判决的 LR1。这些改法由第 12 批三位实现员写进 `crates/`，合入后又打了一份改钉值补丁（`records/2026-10-01-删除与并发事务.md` 2026-10-03 那几条）。这一轮判两件事：代码做的是不是判决与用户定案说的；哪一格会错。

第三轮的分工：V1–V6 是第二轮的修补，这一轮问它们撑不撑得住、第二轮报的数能不能核。丙1、Q-F、U4、U8、U13、LR1、挂载透明这几样是第一次被代码轮攻。这一轮新冒出来的改法算零轮，不再为它们开一轮，写进判决的交用户表。

改动全文在 `research/prompts/_m3-unlink-code-r3-diff.md`（四份补丁原样，加上变异表合入前后的差）。冻结副本 `/tmp/claude-1000/m3-unlink-code-r3/tree/`，被判的 15 份文件的 sha256 在 `research/prompts/m3-unlink-code-r3-snapshot/crates-sha256.txt`。四位实现员的报告在 `/tmp/claude-1000/m3-batch12/{x,y,z,pins}/report.md`，报告里的结论句只当线索，以代码为准。

| # | 改动 | 主要落点（冻结副本） | 压着的条款或定案 |
|---|---|---|---|
| N1 | 丙1：卸载那一串回收出来的槽不扣到生效，预演与真发都能用；准入那一串照旧扣到生效 | `crates/singlefs-core/src/mount.rs` 第 2236–2262 行（`RaiseFloorEntry::reuse_of_the_reclaimed_slots`）、第 2765 行（`raise_the_floor_through` 里的调用）；文档第 931、961、4560、4759 行（V5）；用例 `crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs` 第 466、1258、1404 行 | D16（发布语义） 已定项 1「抬 F 那一串」「正常卸载」「生效」三行；D8（核心索引结构） 已定项 16 第 6 条；丙1 的定义在 `research/prompts/m3-unmount-after-short-continuation-r1-opus-output.md` 第 196–205 行 |
| N2 | V1、V2、V3：续做里推的抬 F 自己报错时，包成 `FloorRaiseFailedAfterTheMountsPublishes`；续做那一格交 `Publish` 时，每一串先写系统配置那一步的写进错里；U9 补两条用例与 MU1、MU2 | `mount.rs` 第 512 行（结构）、第 4575 行起（`continue_the_deletion_queue_until_it_is_empty`）、第 4639、4675、4698、4827 行 | `PublishSequenceFailed` 与 `FloorRaiseFailedAfterTheMountsPublishes` 的契约文档；第二轮判决第三节 V1–V3 |
| N3 | Q-F：删除意图那次与每一批把 inode 记录的 `change_count` 设成这次发布的 txg | `crates/singlefs-core/src/transaction.rs` 第 5734、5795 行；用例 `crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs` 第 426 行 | D8（核心索引结构） 已定项 6 偏移 88 改动计数；D8 已定项 16 第 7 条 ①b |
| N4 | U8：墓碑预付的层级读上一版 `TombstoneRoot` 单元字节的码 2 头 | `transaction.rs` 第 4172–4189 行 | 第一轮判决第三节 U8 |
| N5 | 挂载透明认续做：用户可见树根指针按续做之后的那一版比，F 认续做推的抬 F，界仍按位置取；搬进 harness 的取界单测 | `crates/singlefs-harness/src/mount_transparency.rs` 第 233–293 行、第 368–393 行起 | 用户 2026-10-03 定「续做算挂载的一部分」；形式化接入那条线定的「F 的界按挂载交回的抬 F 那几串在写表末尾的位置取，取不出交 `NotLocatedInTheMountWrites`、整段 F 记不判」（`records/2026-10-01-删除与并发事务.md` 第 376 行） |
| N6 | 崩溃注入：删完那一版上挂载之后的那一次发布改调 `publish_new_inodes`；U4 拿掉 `Continuation => continue`；V1 的划段那一半；V6 | `crates/singlefs-checker-tier/src/crash_injection.rs` 第 1585、1695、1779–1790、2388–2412 行；`crates/singlefs-checker-tier/tests/crash_injection_writable_mount_after_the_crash.rs` | D8（核心索引结构） 已定项 16 第 5、6 条；第一轮判决 U4，第二轮判决 V1、V6 |
| N7 | U13：模型把文件按数据单元数记，删除步按产品批大小 144 从尾部分批；新加一种比重 `DELETING_FILES_OF_SEVERAL_LOWER_LEAVES` 与顺序写内容；改钉值补丁让模型认中央映射树根以下的节点，并沿来路带一个节点数上界 | `crates/singlefs-harness/src/history.rs` 第 314–382、448 行；`crates/singlefs-harness/src/model.rs` 第 551 行（`central_mapping_tree_nodes_upper_bound`）、第 1924 行（`answer_deletion_step`）；`crates/singlefs-harness/src/model_comparison.rs`；`crates/singlefs-harness/tests/random_histories.rs` | D8（核心索引结构） 已定项 16 第 2 条；第一轮判决第三节 U13；D8 已定项 11 ②③ |
| N8 | 其余：LR1（新文件 `crates/singlefs-harness/tests/a_writable_mount_with_system_configuration_slot_zero_unreadable.rs`）；重钉 `fault_injection_fast_tier.rs` 的取样点；A3b-15 换取样点；「清单为空那一形照新发现报」那一行变异；续做结局的守护变异；变异表追加与替换 | diff 文件第 3、4、5 节 | 遗留判决 `research/prompts/m3-leftover-code-r1-main-verification.md` 第三节 LR1；`.claude/rules/mutation-sampling.md` |

**这一轮不判的**（各有去处，腿不攻）：
- U1：用户 2026-10-03 定记欠账。
- LR5：没写代码，留档测试归下一批。
- C511 那一行变异：在 HEAD fcc4d340 上也没红，归下一批补写死用例。
- 「满盘删不动」那 7 格：欠账。
- 开工第一轮接手那条线 7-2（根环、R2）合进主工作区的改动。它们在冻结副本里，但不属于这一批：`mount.rs` 的 `unreadable_ring_slots_inside_the_window`、系统配置读、见证读、取号、根环窗口判定那几段，`recovery.rs`、`root_ring.rs` 与格式常量。合入验证里的 3 条红归它们：`unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over` 1 条，`writable_mount_refusal_judged_against_the_d23_list` 2 条，`core_review_unit_area_start_and_publish_limits` 的无界循环。
- 变异表第 1873、1876 两行：别的会话改的。

## 二、实现今天的样子（主 agent 的观测，2026-10-03）

- 冻结副本取自 2026-10-03 主工作区：第 12 批 X、Y、Z 三份补丁加上改钉值补丁，已经合入。
- 合入验证（`merge-and-verify.sh`，日志 `/tmp/claude-1000/m3-batch12/merge-verify/`）：harness 档红 3 条加 1 个停掉的二进制，都在第一节点名的 7-2 那几处；fmt 过，clippy 过；checker 档没跑。
- 改钉值补丁打进来之后，主 agent 在主仓重跑了它动到的 6 个测试目标（`--lib`、`random_histories`、`unlink_in_histories_judged_by_the_model`、`model_comparison_judges_…`、`zero_unit_publishes_…`、`fault_injection_fast_tier`），全绿（`/tmp/claude-1000/m3-batch12/pins-main-rerun.log`）。U13 慢档用例（release）在改钉值实现员的副本上：48 段、新发现 0，删除批 19 段，走空多步 5 段（`/tmp/claude-1000/m3-batch12/pins/report.md`）。
- `mount.rs` 第 2251–2255 行的文档写的是：回收的槽被卸载这一串的单元写掉时，「引用它们的那几条根都已低于 F_生效」。实现员 X 在只剩一块盘的盘面上量过，见用例第 1404 行与 X 报告第六节：恢复择到的是那块盘上最新的根 txg 19，而系统配置里的 F 是 21；择根这一步不看 F_生效。txg 19 那条根引用的槽，与卸载第一次空发布写进的槽交不交，X 没量。池级 checker 在这份单盘镜像上红 I-1.1、I-2.1、I-4.8、I-7.2、I-7.4、I-7.6。「只写了系统配置、一个单元都没写」那一刀上也红同样六条。改之前那一版（`HeldUntilFloorTakesEffect`）在同一刀上红不红，没量。
- 实现员 X 报告第六节 Q2：`FloorRaiseFailedAfterTheMountsPublishes::writes_of_persisted_publishes` 在续做那一处与写行之后那一处口径不同，写在结构的文档里。Q3：V2 的相加是按种类重放写调用凑出来的。Q4：U8 零根那一支交 0。
- 实现员 Y 报告第六节：
  - ①：续做中间二次崩溃时，用户可见树根指针取「txg 不大于它的最近一步续做那一版」。
  - ②：期望值取自挂载交回值，不另外读盘。
  - ⑤：取界单测搬进了 harness。
- 改钉值实现员报告：中央映射树的节点数按条目数现算不出来（叶从中间切，收缩不合并，见 `crates/singlefs-core/src/code_two_tree.rs` 第 447、543、646 行），模型沿来路带一个只涨不落的上界（读法甲）。另外，长出根之下的节点在三片下段叶那一档（289–292 个单元），不是两片。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　条款对实现（N1–N8）
逐条从判决行、用户定案或条款原文推到冻结副本里的行，再推到钉它的测试与变异行。要看的有这几处：
- N1：丙1 是不是只改了卸载入口；第 2251–2255 行那段文档的推理，对不对得上恢复的择根代码（`crates/singlefs-core/src/recovery.rs`，择根那一段）。
- N2：两处 `writes_of_persisted_publishes` 的口径与契约文档对不对得上；「落下的写 − 取号两次 = 写账合计」在三种起点上是不是都钉住了。
- N3：意图那次与每一批的 `change_count` 是不是这次发布的 txg；①b（同一份崩溃镜像各自续做，盘逐字节相同）还成不成立。
- N4：层级是不是读的码 2 头。
- N5：界是不是仍按位置取，有没有退回按内容认。
- N6：`publish_new_inodes` 那一臂与 V1 划段的下标，是不是和 N2 的口径一致。
- N7：模型的分批与实现是不是同一个产品批大小、同一个从尾部起的次序。
- N8：每条新变异改坏的是不是它名字说的那一处。

### K2　哪一格会错（N1–N8，重点 N1、N2、N7）
在冻结副本的拷贝上攻：
- **N1 丙1 的崩溃安全**：
  - 算 X 第 1404 行那份盘面上 txg 19 引用的槽与卸载第一次空发布写的槽交不交，并把用户决定的那几步放开扫（单元区槽数、覆盖写次数、文件单元数、崩在哪一批、卸载里崩在哪个写之后、丢哪块盘）。
  - 对照：同一刀在改之前那一版（`HeldUntilFloorTakesEffect`）上 checker 红哪几条。拿它分开「单盘镜像本来就红」与「丙1 写掉了旧根引用的槽」。
  - 双盘都在时，有没有一刀能读到被写掉的单元。
  - 卸载—再挂循环里回收的槽，会不会被一条仍可能被择中的根引用。
- **N2 写账**：注错点放开扫，找「落下的写 − 取号两次 ≠ 写账合计」的形；Y 的划段在准入与续做都推过时，取界对不对。
- **N7**：映射树上界会不会低于实现的真节点数。模型里只涨不落的上界，会不会让该拒的被模型判成「允许拒」而漏掉；有没有一段合法历史，模型的分批与实现不同。
- **N3–N6、N8**：每条新用例改坏点名的那一行红不红；有没有错实现照样过；Y 选的二次崩溃规则（报告第六节 ①）有没有一形判错。

### K3　被判的文件名单
`research/prompts/m3-unlink-code-r3-snapshot/crates-sha256.txt` 里那 15 份；冻结副本里每一份都要有一格判到。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1、K3 | 逐条从判决行、用户定案与条款原文推到代码行、测试与变异行 |
| 云端攻方（Opus） | K2，重点 N1、N2、N7 | 在冻结副本的拷贝上造反例，只跑 harness 档；用户决定的那几步放开扫，不写死 |

## 五、跑前写死的判据

- 两条腿都没打中：判站住（一次抽样；「没打中」照 `.claude/rules/three-way-inference.md` 再抽一次才记）。
- 打中：主 agent 现查坐实，改法进下一批。这是第三轮，改法算零轮，不再开第四轮，写进判决的交用户表。
- N1 专门的一条：K2 算出「txg 19 引用的槽」与「卸载写的槽」有交集，而且改之前那一版在同一刀上不红那几条，就判丙1 在单盘恢复上打穿，交用户。交集为空，或者改之前那一版同样红，就判那六条红是单盘镜像本身的事，记进欠账，不归丙1。
- 腿提的新改法算零轮。

交付：
- 腿的报告写进 `research/prompts/m3-unlink-code-r3-<腿名>-output.md`，分段落盘：每次写进文件不超过 150 行，第一段排他新建，回复只写短句。
- 模型与脚本放 `research/prompts/m3-unlink-code-r3-<腿名>-model/`；草稿放 `/tmp/claude-1000/<腿名>/`。
- 引 kb 条款写 kb 文件自己的行号，去 kb 文件里现查，不从背景材料里数；引代码写冻结副本里的文件:行号。
- 方案按 `crates/` 今天的实现来谈。
