# 第 34 批代码轮第一轮正文（2026-10-06）

<!-- doc-lint:not-numbers Q1 Q2 Q3 Q4 Q5 Q6 RV B4 F1 F2 G4 -->

## 一、题面

第 34 批（`records/2026-10-01-里程碑三形式化接入.md#34`）照第 32–33 批代码轮判决 `research/prompts/m3-formal-code-b33-r1-main-verification.md` 第三节第 1、2 条与用户 2026-10-05 定甲（选项原文「甲 checker 判它损坏」）改，改动全文 `research/prompts/_m3-formal-code-b34-r1-diff.md`：

- 34-1：`crates/singlefs-checker/src/walk.rs`——I-5.4 按版本那一层多收由记录施加出来的那几版与下一次挂载先施加的那一版（RV），`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl` 同改；自证过的根树表指针全零不再读成「没有树表」，照 core 跟过去判损坏，I-5.4、I-5.5、I-9.28、删除态、墓碑谱系五处改成与读不出同一处置；I-5.5 写专门的不适用理由，`crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs` 那一支只认它；坏镜像进 `crates/singlefs-harness/tests/checker_known_bad_images.rs`，GPU 对拍加一条。
- 34-2：`crates/singlefs-core/src/root_record.rs` 根记录四条指针改成与 journal 新根段同一判法，`crates/singlefs-core/src/pointer.rs` 只判 extent 偏移那一支删掉；新用例 `crates/singlefs-harness/tests/root_record_pointer_encryption_and_compression_fields_outside_the_first_version_are_refused_by_both_mounts.rs`；改了 `mount_transparency_judged_field_by_field_after_a_writable_mount.rs` 与 `pointer_extent_offset_that_is_not_zero_is_refused_by_both_mounts.rs`。
- 34-3：`crates/singlefs-harness/src/model.rs` B4 取样点加 100。
- 34-4：8 行变异按名重取原文（34-1 改写之后锚点命中 0 次）。
- 主 agent：`crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs` #9 三个钉值。

条款：`.claude/kb/invariants.md` I-2.4、I-5.4、I-5.5、I-9.5、I-9.16、I-9.28 几行（这批写回之后的文字）；`.claude/kb/checks-owed.md` C581（已还清那一行）、C735；`.claude/kb/decisions/16-发布语义.md`「#### 已定项 1」；`.claude/kb/decisions/19-块指针的结构与宽度预算.md`「#### 已定项 3」；`.claude/kb/decisions/09-加密.md` 开篇；用户定甲原话在 `records/2026-10-01-里程碑三形式化接入.md`「**用户 2026-10-05 弹窗定甲**」那一句。

## 二、实现今天的样子（主 agent 的观测，2026-10-06）

- 冻结副本 `/tmp/claude-1000/m3-formal-code-b34-r1/tree/`（第 34 批全部合入后，清单同目录 `crates-sha256.txt`，315 行）。树里另有根环线第 10h 批（rr2 按槽并两遍、Q3 两遍并起来一条都没有才拒）与删除线第 24 批（新九条专门不适用理由、I-9.17 / I-9.23 读孩子），不在这一轮被判。代码引文读冻结副本，写 `crates/…` 相对路径与那棵树里的行号。要编就拷到自己的草稿目录，每份副本用自己的编译目录，编完核日志里被测包确有 `Compiling`。
- 合入验证 `/tmp/claude-1000/merge-and-verify/2026-10-06-1222270/`：harness 170 个测试二进制全绿，fmt、clippy 过；34-4 打上后变异表静态核无锚点问题。抬 F 慢档那条在删除线第 24 批合入后 13 条全过（它的日志）。
- checker 档没跑：GPU 对拍新加的、GPU 那几行变异、checker 档整表；`crash_enumeration_sharded_across_processes.rs` 那三个钉值在删除线第 24 批合入后又对不上（打印行 sha 变成 4fc9cde7…），归删除线照注释重取，这一轮不判钉值本身。
- 实现员与写回员留下、条款没写的几问（Q1–Q4，第三节 P4 格）：Q1 由记录施加出来的那一版里树表指针全零要不要也判损坏（今天主走读仍当没有树表）；Q2 全零树表指针要不要同时红 I-2.5（今天 I-2.5 整条全零豁免）；Q3 GPU 判器要不要对全零树表指针同改（今天 CPU 红、GPU 不红）；Q4 I-5.5「认出了那一版但没有数据单元」要不要另写理由。另记：GPU 恢复内核 `crash_judge_common.wgsl` 根记录指针那一判今天只判 extent 偏移、比 core 松（进第 35 批）。

方案按 `crates/` 今天的实现来谈。

## 三、格

### P1　RV
多收的两类版本在合法历史上会不会多报（写行、暖机、回退、抬 F、多实例、崩溃恢复之后的那几版）；两类版本的新根段取自哪里，取错一版（取成别的记录或同一记录的旧段）会不会让判法与实现一起错；GPU 扫描内核那一处与 CPU 是不是同一组版本、同一判法（GPU 跑不了的读码，写「推的」）。

### P2　全零树表指针（用户定甲）
五处改成与读不出同一处置之后，有没有一处把本该判的东西整版放过，或在合法历史上误红；「自证过的根」这个前提在每一处是不是同一个集合；改法与 core 读路径（根记录读者判树表指针时）是不是同一个结局。

### P3　34-2 根记录四条指针
新判法在 core 每个读根槽的入口是不是都走到（只读恢复、可写挂载、抬 F 预演、回退判候选、checker 档装置），有没有一处还在只判 extent 偏移；改了的两份测试文件改得对不对（翻全零指针段改出生 txg 那一字节，判的还是不是原来要判的那一件）。

### P4　条款没写的四问
Q1–Q4 各自给出候选、翻面观测与代价（能在副本上量的量），不替用户定；每一问判一句「挡不挡这一批合入」。

### P5　I-5.5 专门理由、B4、#9、变异行
I-5.5 专门理由与它的看门用例分不分得出「没有对象」与「一次都没判」；B4 新取样点；#9 三个钉值的取法与注释；34-1、34-2、34-3 新加与 34-4 重取的变异行：锚点各恰好命中一次、点名的测试跑得到、合入后基线绿、有没有一行红在越界 panic 而不是断言上。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | P1–P5 | 从判决、用户定案与条款正推每处改动做的是不是它们说的，每格给判定、引文与推翻条件 |
| 云端攻方（Opus） | P1–P5，重点 P1、P2、P4 | 找反例：一段合法历史让新判法误红，或一个坏实现让它放过；在拷出的副本上造、量；P4 每一问给量过的代价 |

## 五、跑前写死的判据

- 两腿同判、主 agent 现查坐实 ⇒ 照那一判；一次抽样的「没打中」只算这一次。
- 两腿分歧 ⇒ 主 agent 核分辨两者的那个事实；核不出就开第二轮，换攻击面。
- 腿提的新判法、新用例算零轮；P4 那四问是条款选择，交用户，不由腿或主 agent 定。
- 重型测试不跑：checker 档的测试、GPU 对拍与 `crates/mutations.tsv` 整表都不跑，涉及它们的结论写「推的」；harness 档测试可以跑。

交付：腿的报告写 `research/prompts/m3-formal-code-b34-r1-<腿名>-output.md`，分段落盘（每次写进文件不超过 150 行，第一段排他新建，回复只写短句）；模型与脚本放 `research/prompts/m3-formal-code-b34-r1-<腿名>-model/`，带一份 `tools/rerun.sh`；草稿放 `/tmp/claude-1000/<腿名>/`。引 kb 条款写 kb 文件自己的行号（去 kb 文件里现查，不从背景材料里数）。方案按 `crates/` 今天的实现来谈。
