# 第 23 批代码轮第一轮正文（2026-10-04）

<!-- doc-lint:not-numbers K1 K2 K3 F3 H2 A1 B1 -->

## 一、题面

第 23 批（`records/2026-10-01-里程碑三形式化接入.md#23`）合入了 23-1 一份补丁，改动全文 `research/prompts/_m3-formal-code-b23-r1-diff.md`。它照判决 `research/prompts/m3-formal-code-b21-r1-main-verification.md` 第三节第 2 条改：

- K3-a：`crates/singlefs-checker-tier/src/crash_injection.rs` 内联用例 `a_second_crash_losing_the_records_of_the_row_publish_after_a_first_crash_losing_the_records_of_the_first_file_counts_one_illegal_witness_refusal`：第一次崩溃扣下第一个文件那次的两份记录、二次崩溃扣下写行的记录（越屏障的靶向摆法），写下标从写表现算，断言 (判过, 不合法) = (1, 1)；变异「第三截所选那一版交成第一次崩溃那一版」。21-2 那条按支的用例与变异留着。
- F3-c：种子那段异或抽成 `seed_of_the_second_crash_points`，返回 `SecondCrashPointSeed`，摆法逐位不变；内联用例：长 12 的段里扣下集合非空的 4095 个子集算出的种子两两不同；变异「指纹只折第一个写」（`.take(1)`）。
- A1：固定历史用例改成断言新建那条 inode 记录整条等于预期（出生代与改动计数 = 发布后那一版根的 txg、nlink 1、size 0、写入时间 `FIXED_WRITE_TIME_SECONDS`），崩溃那一版的记录原样；4 行变异：nlink 0、出生代早一版、size 4096 三行改 `crates/singlefs-core/src/transaction.rs` 的 `publish_new_inodes` 原文（会话里建 inode 也走它），写入时间 0 改调用点。
- B1：`crates/singlefs-checker-tier/tests/crash_injection_publish_after_a_mount_whose_deletion_continuation_stays_short.rs` 盘面 128 → 136 槽；变异「那一支建 0 个」新开一行（与 A1 那一行原文相同，点名的测试不同）。
- H2：`crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs` 两条用例的挂载次数改成 `ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM + 2` 与 `+ 1`，写死的 txg 换成前提断言。
- `crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs` 一句注释措辞；`crates/singlefs-core/src/root_record.rs` 第 26 行文档注释照今天的施加改。

条款：`.claude/kb/decisions/08-核心索引结构.md`「#### 已定项 16」第 4、5 条；`.claude/kb/decisions/03-空间分配.md`「#### 已定项 7」；`.claude/kb/checks-owed.md` C670（无读故障的崩溃态上可写挂载见证判据为真没有判官）、C696（崩溃注入的读故障一支在随机历史里从没触发）。

## 二、实现今天的样子（主 agent 的观测，2026-10-04）

- 合入验证：harness 档 150 个测试二进制全绿；fmt、clippy 过。
- checker 档的四条用例与 checker 档变异只编过、没跑；23-1 实现员在 harness 档用公开入口写探针量过：K3 接对 (1, 1)、接错 (1, 0)；F3 入库指纹 4095 个种子、只折第一个写只剩 2 个；A1 正确实现下整条对上，nlink 0 那处 core 改法在探针上抓到；B1 136 槽建 1 个是 `UnitAreaWall`、建 0 个做成。出生代、size 两行变异没上探针。21-1 那行「放宽一个 txg」变异经 prove-red 仍抓到。
- 23-1 交主 agent 的：I-9.5 池级 checker 没实现、条款只一个方向（已转删除与并发那条线，记 C708）；H2 两条用例名还带 nine / ten（没改）。

方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　K3-a 与 F3-c
K3-a 的态是不是攻方那个态、断言在接对时 (1, 1)、接错时 (1, 0)；写下标现算会不会在别的写表长度上摆错态；F3-c 的种子抽函数前后是不是逐位相同、4095 个子集两两不同的断言能不能抓住别的弱指纹（不只是 `.take(1)`）。

### K2　A1 与 B1
A1 的整条比对有没有漏字段（blocks、权限、别的时间字段）；三行改 `transaction.rs` 的变异是不是只让这条用例红、会话里建 inode 的别的用例怎样；B1 136 槽盘面是不是稳地走到 nlink = 0 那一支、单元区槽数对别的参数敏不敏感。

### K3　H2 与注释
挂载次数从根环槽数算之后，前提断言在别的根环槽数下还成不成立；两处注释与代码对不对得上。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1、K3 | 逐句核代码做的是不是条款与判决说的，每格给判定与引文 |
| 云端攻方（Opus） | K1、K2 | 攻哪一格会错：用例判不出的坏改法、摆错的态、漏比的字段；量得了的在副本上写小模型或 harness 用例量，checker 档不跑、写推的 |

## 五、跑前写死的判据

- 两腿同判且主 agent 现查坐实 ⇒ 按那一判；打中而坐实 ⇒ 改法并进下一批实现员；分歧 ⇒ 开第二轮。
- checker 档、GPU 没跑的格，腿的结论只能是「推的」，判决里照实标。

交付：腿的报告写 `research/prompts/m3-formal-code-b23-r1-<腿名>-output.md`，分段落盘（每次写进文件不超过 150 行，第一段排他新建，回复只写短句）；模型与脚本放 `research/prompts/m3-formal-code-b23-r1-<腿名>-model/`；草稿放 `/tmp/claude-1000/<腿名>/`。引 kb 条款写 kb 文件自己的行号（去 kb 文件里现查，不从背景材料里数）。方案按 `crates/` 今天的实现来谈。
