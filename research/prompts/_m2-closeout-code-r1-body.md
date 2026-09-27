# 里程碑二收尾：上次提交 crates 之后的全部改动，代码轮第一轮正文（2026-09-27）

<!-- doc-lint:not-numbers Z1 Z2 Z3 Z4 Z5 Z6 Z7 Z8 -->

## 一、这一轮要判什么

`crates/` 上一次提交是 `346f5e6`（2026-09-25 22:29 JST）。之后打进主工作区的改动，一轮代码三方都没走过（`.claude/rules/implementation-workflow.md` 三步里的第 2 步）。这一轮攻的是**写好的代码与它的测试**：代码做的是不是条款说的。设计不重判；设计轮的判决当已定的前提用。

被判的改动出自这些批次，报告都在 `research/prompts/` 下：

| 批 | 做了什么 | 实现员报告 |
|---|---|---|
| 实一至实四丙 | 回退改成向前发布：SysPre、卸载时抬 F、挂着时向前回退、E156 / E158 装置跟上、测试钉值、故障注入新发现查因 | `m2-impl-rbf-1-implementer-report.md` … `m2-impl-rbf-4c-implementer-report.md` |
| 实五 | 准入改形态与 445（C565） | `m2-impl-rbf-5-implementer-report.md`、`m2-impl5-implementer-report.md` |
| 实六 | 层 0 续跑、记录核对器按持久集合判缺席 | `m2-impl6-implementer-report.md` |
| 实七 | 生成器与测试胶水 | `m2-impl-rbf-7-implementer-report.md`、`m2-impl7-implementer-report.md` |
| 实八 | 模型与记录核对器跟上 | `m2-impl8-implementer-report.md` |
| 实分一 | 层 0 崩溃重放双机分片（crates 那一侧） | `m2-impl-shard1-implementer-report.md` |
| 实审 A1、A1b、A2a、A2b、A2c | 核心静默出错：系统配置取盘上那份、设备身份去重、覆写系统配置前的屏障、抬 F 途中块设备错原样上交、树表重复、槽 1 搜索、环长下限、单元区起点随环长、错误成员 | `m2-rev-a1-…`、`m2-rev-a1b-…`、`m2-rev-a2a-…`、`m2-rev-a2b-…`、`m2-rev-a2c-implementer-report.md` |
| 实审 A4、A4b | ckpt_cost 按盘分路、按最坏情况计 | `m2-rev-a4-…`、`m2-rev-a4b-implementer-report.md` |
| 实审 B1、B2、B2b、I-1.11 | checker：同一根里的交叉引用、映射 key、计数器 0、I-7.4 被抛弃根那一半、窄判的几条 | `m2-rev-b1-…`、`m2-rev-b2-…`、`m2-rev-b2b-…`、`m2-rev-i111-implementer-report.md` |
| 实审 B3a、B3a-2、B3a-3、B3a-3b、B3b、B3c-1 | 验证装置：按设备记屏障、原地覆写第三态、稀疏盘、崩溃注入补可写挂载与二次崩溃、对拍双向 | `m2-rev-b3a-…`、`m2-rev-b3a2-…`、`m2-rev-b3a3-…`、`m2-rev-b3a3b-…`、`m2-rev-b3b-…`、`m2-rev-b3c1-implementer-report.md` |
| 实审 C11 | 头宽三方各算、交叉断言 | `m2-rev-c11-implementer-report.md` |

共用问句：

> **这一处代码的行为，是从哪条已定分项推出来的？推不出的那些，它是在替一条没写的条款做选择吗？如果是，这个选择今天有没有会红的东西钉着？**

三种结论，每格只能落一种：

| 结论 | 什么样 | 要交出什么 |
|---|---|---|
| **兑现了条款** | 行为是某条已定分项的字面后果 | 那条分项的**整段原文**，加从原文到这段代码的那一步 |
| **替没写的条款做了选择** | 不同实现会做出不同的、都说得通的选择 | 那个选择是什么、它影响哪些字节或哪条可达历史、今天有没有会红的东西钉着 |
| **和条款说反话** | 代码做的与某条已定分项的字面相反 | 两边各自的原文，以及这次差异在哪个字节 / 哪条历史上看得出来 |

## 二、实现今天的样子（主 agent 的观测，2026-09-27 08:4x JST 现查）

- **腿读代码一律读快照**：提交 `67f447de9761f826711565032d8ce16fbf44b902`（ref `refs/sop/m2-closeout-code-r1-snapshot`，树 `ae2a472004d614fbe7e9b261cb236d0ce750520c`）。它是 08:4x 主工作区 `crates/` 的原样，含未跟踪的新测试，不在任何分支上。取法：`git archive refs/sop/m2-closeout-code-r1-snapshot crates | tar -x -C <草稿目录>`。主工作区在腿跑着的时候还会被在飞的补丁改，别读主工作区。
- 快照里 `crates/*/src/*.rs` 56 份的 sha256 在 `research/prompts/m2-closeout-code-r1-snapshot/crates-src-sha256.txt`。
- 被判的范围：`git diff 346f5e6 refs/sop/m2-closeout-code-r1-snapshot -- crates/`，123 个文件，+47000 / −11385 行。其中 src 41 份（`git diff --stat 346f5e6 -- crates/*/src` 在快照上同数）：
  - checker：`image.rs`、`lib.rs`、`position_addressed.rs`、`walk.rs`；
  - core：`admission.rs`、`allocation_record_tree.rs`、`allocator.rs`、`code_two_tree.rs`、`extent_tree.rs`、`instance_table.rs`、`journal.rs`、`lib.rs`、`make_filesystem.rs`、`mount.rs`、`mounted_read.rs`、`recovery.rs`、`rollback_witness.rs`（删）、`root_record.rs`、`system_configuration.rs`、`transaction.rs`、`unit.rs`、`write_request_split.rs`；
  - format：`lib.rs`；
  - harness：`bin/e142_first_transaction_write_dump.rs`、`bin/e142_first_transaction_write_dump_one_device.rs`、`bin/e156_allocation_basis_counts.rs`、`bin/e158_root_choice_repair.rs`、`bin/first_transaction_on_device.rs`、`bin/first_transaction_region_bytes.rs`、`crash.rs`、`crash_injection.rs`、`device_log.rs`、`fault_injection.rs`、`first_transaction_regions.rs`、`history.rs`、`lib.rs`、`model.rs`、`model_comparison.rs`、`on_device_modes.rs`、`read_tally.rs`、`segments.rs`。
  - 另有两份新文件不在上面 41 份里（材料员按范围现查出来的，全文在附录二「二、新文件全文」）：core `mounted_session.rs`（291 行，实审 A1 第 23 条与 A2c 加的会话一侧错误成员）、harness `layer0_progress.rs`（1911 行，实六的层 0 续跑）。Z3、Z5 连它们一起判。
- `bin/e142_*`、`bin/e156_*`、`bin/e158_*` 是实验装置，对不对由各自实验页的变异表与单测判，不在这一轮的八格里；判决里只按路径点名，写明为什么不判。
- **在飞、不在快照里、归第二轮的**：A4c（`admission.rs`、`allocator.rs`、`mount.rs`：每块盘改几片叶、C545 那一格、压小容量）、C11b（format 里剩下 8 个带算术的常量、`journal_in_flight_record_limit` 三方各算）、B3c-2（崩溃注入第二、三截交记录核对器的写表）、B3a-3c（4 份层 0 文件重新布置流、position_addressed 的打印溢出）、A3（坏盘 panic、可写挂载判全树头与 key 次序、读者判预留位、挂载侧单元区起点随环长），以及 C554 的修法（E158 第 4 次跑之后用户定）。
- **已知、不算打中**（腿打中这几格时写「已知」并点名去向）：
  - C554：挂载时读不出的最新根被当成抛弃、它的单元被重发（`.claude/kb/checks-owed.md` 的 C554 行；调查员报告 `research/prompts/m2-investigate-gate74-reds-report.md`）。门禁 74 号快档 28 段、收口表第 43 行那条、formatted_pool 那条都红在这一形。
  - 4 份层 0 文件先红在 `prepare` 的形状断言上，position_addressed 快用例的 `println!` 溢出（`m2-rev-b3a3b-implementer-report.md` 第八节 F1、F2）。
  - ckpt_cost「每块盘两条叶路径」可能没算全、准入放行之后落点取不到（`m2-rev-a4b-implementer-report.md` 第八节第 2 条、调查员报告第 3 条）。
  - format 里 8 个带算术的 `pub const`（`m2-rev-c11-implementer-report.md` 设计问题第 1 条）。
  - `acquire_instance` 实例代号到顶 panic、零单元发布不判记录条数上限、挂载侧单元区起点没切到按环长现算（`m2-rev-a2c-implementer-report.md`「要你定的」第 1、4、5 条）。

## 三、八格

### Z1　回退改成向前发布（实一至实三、实八的写者一侧）

`mount.rs` 的挂着时向前回退、卸载时抬 F、SysPre；`recovery.rs` 择根与按记录重建；`transaction.rs` 回退入口与发布计划；`root_record.rs`；`rollback_witness.rs` 删掉之后原先它钉的事由谁钉。压着 D16（发布语义） 已定项 1、已定项 7，D23（journal 的角色与格式） 已定项 14、15、16、18，D18（块里携带什么信息） 已定项 11；设计三轮判决 `research/prompts/m2-rollback-forward-r1-main-verification.md` … `r3-main-verification.md` 定下的形态。

### Z2　准入（实五、实审 A4、A4b）

`admission.rs` 的 ckpt_cost（分配记录树 `盘数 × 2 × (高 − 1) + 1`、中央映射树逐层 min(节点数, 删 + 插) 加切出的节点）；`mount.rs` 抬 F 的准入与「推满仍不够」那一格（C565）；`system_configuration.rs`。压着 D28（挂载期承诺量） 已定项 1、已定项 3、已定项 4，D2（RAID 条带策略） 已定项 13，D16（发布语义） 已定项 1「准入」那一行。A4b 自报的「每块盘两条路径不是全部最坏情况」归第二轮，这一格不重报。

### Z3　挂载入口的静默出错与坏盘（实审 A1、A1b、A2a、A2b、A2c）

`mount.rs`：可写挂载取盘上选中的那份系统配置、调用方参数不一致就拒；设备身份重复就拒；抬 F 途中的块设备错原样上交。`transaction.rs`：覆写系统配置之前的池屏障、取号回卷那一支、新错误成员。`recovery.rs`：`checked_add`、树表同一种树两条就报 `UnitMalformed`。`make_filesystem.rs`：环装不下 3 条记录就拒。`allocator.rs`：单元区起点由环长现算。压着 D22（单元原子性怎么合成） 已定项 16、已定项 21，D23（journal 的角色与格式） 已定项 16，D15（格式冻结政策） 里系统配置不可变段那几行，D2（RAID 条带策略） 已定项 13。

### Z4　checker 新判定（实审 B1、B2、B2b、I-1.11；实四乙、实六、实八的 checker 部分）

`walk.rs`：同一根里第二次引用同一单元判 I-5.1，别的版本走过的共享子树不判；映射条目判 I-1.11；计数器 0 判 I-8.6 违例；I-7.4 被抛弃根那一半；I-2.5 点名项位置条目升序；I-9.2 类型段 0；I-9.4 沿叶序容器号；根记录持有的分配记录树进 I-3.9、I-5.4；I-7.7 有读不出的槽时两句各怎么判；记录只由记录施加的那一版怎么并进遍历。`image.rs`、`lib.rs`。压着 `.claude/kb/invariants.md` 那几行（每条整行抄）与 D13（验证路线） 已定项 7。判据还有一条：每条新判定是不是只在它该有对象的状态上有对象（不适用与判绿分得开）。

### Z5　验证装置：按设备记屏障、原地覆写第三态、续跑与分片（实六、实分一、实审 B3a、B3a-2）

`segments.rs` 的 `SegmentClosingRule`：按设备记屏障，FUA 只放行它那块盘。`crash.rs`：`TearableInPlaceOverwrites` 与撕裂镜像，条件是长于 512 字节的写，撕裂点取新旧不同那一截的中点；状态数闭式；分片与续跑。`device_log.rs`。压着 D13（验证路线） 已定项 4、已定项 7。要连同答的一句：第三态只罩长于 512 字节的原地覆写、撕裂点只取一处，这一取法是 D13 已定项 4 原文的字面后果，还是替它做的选择；如果是选择，≤ 512 字节的原地覆写与别的撕裂点在哪条可达历史上会给出不同的恢复结局。

### Z6　崩溃注入与对拍（实七、实八、实审 B3b、B3c-1）

`crash_injection.rs`：每个崩溃状态跑可写挂载、一次发布、checker，再摆取号 / 写行 / 暖机三个二次崩溃；第一截交记录核对器整条流。`fault_injection.rs`、`history.rs`、`model.rs`（`ObservedRoot` 三个字段、双向比）、`model_comparison.rs`。压着 D13（验证路线） 已定项 7。第三截的前缀交法与第二截没接记录核对器已知、归 B3c-2，不重报。

### Z7　头宽三方各算（实审 C11）

format 只留标量；core `unit.rs`、checker `lib.rs`、模型 `model.rs` 各一份 `index_node_header_bytes`，互不调用；交叉断言扫 key 宽 0..=255，另钉 6 个 `format-const` 头宽。压着 D13（验证路线） 已定项 5。问：三份是不是真独立（有没有经别的共享常量绕回同一个式子），扫的取值域是不是全的。

### Z8　合并点

`mount.rs` 从 346f5e6 起被实一、实二、实三、实五、实八、A1、A1b、A2a、A2c、A4、A4b 各改过（`git log` 看不到，改动都在工作区里，靠各份报告的「写过的文件」那一节）；`transaction.rs` 同样。实现员多在各自副本里改、再由主 agent 打补丁合进来。问：合并之后，同一条发布路径上各批加的判定与屏障，次序与各自报告里写的一样吗；有没有哪一批加的拒绝在另一批加的写之后才判（写了再拒）。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端攻方（Opus） | Z1、Z3、Z5、Z8 | 找反例：造一条合法历史或一个合法盘面，让代码的行为与条款字面不同，在快照副本上写用例跑出来 |
| 云端正推（Sonnet） | Z2、Z4、Z6、Z7 | 逐格核「代码做的是不是条款说的」，三种结论每格落一种 |
| 本地攻方 | 算术（与云端攻方不重叠：它不造历史，只按事实表逐格算数） | 按事实表逐格填数、与仓里钉的数比：第一条流全量 150994980 与快档 54、第二条流 14960689284 与快档 390 的闭式怎么来（按段序列与每段写数，第三态记一格）；A4b 量表里 `two-4GiB`、`two-1GiB`、`two-4GiB-mapping-4-8` 三格的 ckpt_cost 各项；头宽 `86 + 2 × k + 29` 在 k = 8、24 上；一条记录装几个点名项 |

## 五、交付

每条腿的报告按格写：结论（三种之一）、原文、代码位置（快照里的 `文件:行号`）、打中的给复现（命令、种子、原样输出）。攻方写的用例与模型放各自的模型目录，在快照副本上跑，不改主工作区。

## 六、回看决策

判决里写。
