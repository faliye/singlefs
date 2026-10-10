# 第 20 批代码轮第一轮正文（2026-10-04）

<!-- doc-lint:not-numbers K1 K2 K3 F1 F2 F3 F5 V242u U6 -->

## 一、题面

第 20 批（`records/2026-10-01-里程碑三形式化接入.md#20`）合入了 20-1 一份补丁，改动全文 `research/prompts/_m3-formal-code-b20-r1-diff.md`：

- F1：第二截那条接线用例加两条断言——判官列出的没满足条件里要有 `WitnessCriterionTrueOnACrashImageWithoutReadFaults`、不能有 `CriterionFalseWhenRecomputedOnTheCrashImage`；变异一行「所选那一版交成 None」。
- F2：新靶向二次崩溃摆法 `SecondCrashPointDraw::ExactlyOneSecondCrashState`，入口 `inject_crashes_into_history_drawing_second_crashes`（原 `inject_crashes_into_history` 改成薄壳）；一条经第三截调用点的用例，摆「写行根槽没落、两槽轮换落了」，断言第三截判过 1、不合法 1；三种接错各一行变异。
- F5：`stream_index_of_the_last_write_of_the_crash_window` 抽成纯函数、内联单测钉录制流下标；变异一行「下标取错」。
- 18-2「F 不动」「会话关」两条断言各一行变异（`crates/singlefs-harness/src/model.rs` 不改）。
- 发布臂：`mount_and_publish_once_on_the_crash_image` 在带文件的一版上只有第一个文件那条记录活着（在树里、nlink 不是 0）才覆盖写，否则 `publish_new_inodes(…, 1, …)` 建一个 inode；固定历史用例「unlink → 建 inode → 崩在建 inode 那次根落、轮换没落」；变异一行「一律覆盖写」。
- `crates/singlefs-checker/src/image.rs` I-5.5 常量的文档注释改成照分配记录现算；`crash_judge_checker_common.wgsl` 删一个没人用的常量。
- 主 agent 合入时：主仓 `crash_injection.rs` 已被根环那条线第 9 批改过（加了实例表清扫两段、`writable_mount_phase_of_each_write` 多一个参数），20-1 的导入行与第三截解构入参两段手工接上，靶向用例里那一处调用补了第三个参数。

条款：判决 `research/prompts/m3-formal-code-b18-r1-main-verification.md` 第二节 K2（F1、F2、F5 的出处）与第五节；`.claude/kb/decisions/08-核心索引结构.md`「#### 已定项 16」第 4、5、6 条（写入口拒不在树里与正在删的 inode、删完那一版建 inode、会话里每次用户改动先走空删除队列）；`.claude/kb/decisions/16-发布语义.md`「#### 已定项 1」「正常卸载」那一行；`.claude/kb/checks-owed.md` C670（无读故障的崩溃态上可写挂载见证判据为真没有判官）。

## 二、实现今天的样子（主 agent 的观测，2026-10-04）

- 合入验证（日志 `/tmp/claude-1000/merge-and-verify/2026-10-04-495580/`）：harness 档红一条 `admission_raises_the_floor_before_refusing` 的 `root_slot_already_unreadable_at_mount_counts_as_no_root_and_does_not_block_the_floor_raise`，20-1 不碰它（harness 档不依赖 checker 档），已转根环那条线；fmt 过；`--all-features` 的 clippy 红在 `crash_judge_gpu.rs` 第 74 行覆盖断言（根环那条线新立 I-9.28、GPU 内核还没跟上）与第 5999 行少一个参数（主 agent 已补，`cargo clippy -p singlefs-checker-tier --all-targets -- -D warnings` 退 0）。
- checker 档用例与 6 行 checker 档变异只编过、没跑（20-1 实现员在副本里用公开入口写了 harness 档探针逐步复做：F1 交 None 时条件只剩「复算为假」；F2 注入恒交 Injected 时判官判合法、在第一次崩溃镜像或 `second_base` 上再挂都做成；建 inode 之后那一版 `inode_record` 是 None，覆盖写被拒成 `WriteToAnInodeThatIsNotInTheTree { inode: 1 }`、建 inode 做成）。
- 20-1 交主 agent 的两条：发布臂 nlink = 0 那一支不先走空删除队列、直接建 inode，D8（核心索引结构） 已定项 16 第 6 条只写到「会话」，崩溃注入这一处没写；F3（二次崩溃种子混进第一次崩溃那个子集）主 agent 已判进下一批。

方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　F1、F2、F5 的用例证不证得到它们点名的接错
第二截用例的两条断言、第三截靶向用例、F5 单测，各自在对应的变异下红不红、在正确的接线下绿不绿；靶向摆法 `exactly_one_second_crash_point`（可以越过段尾、崩在挂载哪一段按第一个被扣下的写取）摆出来的是不是它文档说的那个态；薄壳改造有没有让原入口的行为变（种子、摆法、计数）。

### K2　发布臂改法
`first_file_is_a_live_record` 的判法与实现写入口拒的那几种是不是一一对应（不在树里、nlink = 0、别的拒法）；nlink = 0 那一支直接建 inode 而不先走空删除队列，在哪些崩溃态上会让第二截报假发现或漏掉真发现；固定历史用例摆的态是不是它名字说的那个。

### K3　18-2 两行变异、注释与常量、重定基
「F 不动」「会话关」两行变异各自只让点名的那条断言红；`image.rs` 注释与 I-5.5 现行条款一致；wgsl 删掉的常量全仓没有别的引用；主 agent 手工接的两段与 20-1 原补丁意图一致、没丢掉根环那条线加的东西。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1、K3 | 逐句核代码做的是不是条款与判决说的，每格给判定与引文 |
| 云端攻方（Opus） | K1、K2 | 攻哪一格会错：用例判不出的接错、发布臂在哪个崩溃态上报假发现或漏报；量得了的在副本上写小模型或 harness 用例量，checker 档不跑、写推的 |

## 五、跑前写死的判据

- 两腿同判且主 agent 现查坐实 ⇒ 按那一判；打中而坐实 ⇒ 改法并进下一批实现员；分歧 ⇒ 开第二轮。
- checker 档、GPU 没跑的格，腿的结论只能是「推的」，判决里照实标。

交付：腿的报告写 `research/prompts/m3-formal-code-b20-r1-<腿名>-output.md`，分段落盘（每次写进文件不超过 150 行，第一段排他新建，回复只写短句）；模型与脚本放 `research/prompts/m3-formal-code-b20-r1-<腿名>-model/`；草稿放 `/tmp/claude-1000/<腿名>/`。引 kb 条款写 kb 文件自己的行号（去 kb 文件里现查，不从背景材料里数）。方案按 `crates/` 今天的实现来谈。
