# 第 35 批代码轮第一轮正文（2026-10-06）

<!-- doc-lint:not-numbers P1 P2 P3 P4 P5 P6 P7 Q1 Q3 Q4 FL B4 ALT1 -->

## 一、题面

第 35 批（`records/2026-10-01-里程碑三形式化接入.md#35`）照第 34 批代码轮判决 `research/prompts/m3-formal-code-b34-r1-main-verification.md` 第三节第 1 条与用户 2026-10-06 定的四问（Q1 选项原文「甲 两类都判损坏」、Q3「甲 同改」、Q4「甲 另写一句」；Q2 定乙不改）改，改动全文 `research/prompts/_m3-formal-code-b35-r1-diff.md`：

- 35-1：`crates/singlefs-checker/src/walk.rs`——FL：两类由记录造出来的版本（由记录施加出来的那一版、下一次挂载先施加的那一版）取带末条标志那条记录的新根段，下一次挂载那一版要求末条已落；Q1 甲：删掉 `AllZeroTreeTablePointerReading`，由记录施加出来的那一版树表指针全零与根环里的根同一读法（判损坏），下一次挂载那一版全零红 I-4.1；Q4 甲：I-5.5「认出那一版而它没有数据单元」另写一句不适用理由；I-9.28 部分没判与被抛弃的根 I-7.4 两处说明。`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl`（`R_DATA` 同键取末条；`w_walk_tree_table_and_mapping` 遇全零树表指针读那个单元判 I-2.1、记走读失败）与 `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl`（`record_index_of_the_version_the_next_mount_applies_first` 加末条条件）同改；坏镜像进 `crates/singlefs-harness/tests/checker_known_bad_images.rs`，GPU 对拍加四条；`crates/mutations.tsv` 追加 10 行、替换 955、965（ALT1 形）与 2847 行。
- 35-2：`crates/singlefs-checker-tier/src/crash_judge_common.wgsl` GPU 恢复内核择根对四条根记录指针各判五段（`head_ok && extent_offset_zero`，与 core `crates/singlefs-core/src/pointer.rs` 的 `judged` 同判）；新文件 `crates/singlefs-checker-tier/tests/gpu_recovery_root_choice_refuses_root_record_pointer_head_fields_like_the_core.rs` 四条用例；`mutations.tsv` 追加 4 行、替换 2655 行。
- 35-3：`crates/singlefs-harness/src/model.rs` B4 取样点 `[1, 2, 3, 4, 5, 30, 100, 120, 135]`；`crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs` 829 行注释；`crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs` I-5.4 挪进与 I-3.10 同一类；`mutations.tsv` 追加 3 行。

条款：`.claude/kb/invariants.md` I-2.1、I-3.10、I-4.1、I-5.4、I-5.5、I-7.4、I-9.28 几行；`.claude/kb/decisions/23-journal的角色与格式.md`「#### 已定项 15」；`.claude/kb/decisions/19-块指针的结构与宽度预算.md`「#### 已定项 3」；`.claude/kb/checks-owed.md` C735、C738；用户定案原话在 `records/2026-10-01-里程碑三形式化接入.md`「用户 2026-10-06 弹窗定 b34 四问」那一行。

## 二、实现今天的样子（主 agent 的观测，2026-10-06）

- 冻结副本 `/tmp/claude-1000/m3-formal-code-b35-r1/tree/`（第 35 批合入后，清单同目录 `crates-sha256.txt`，316 行）。树里另有删除线第 25–27 批与根环线第 10i 批，不在这一轮被判。代码引文读冻结副本，写 `crates/…` 相对路径与那棵树里的行号。要编就拷到自己的草稿目录，每份副本用自己的编译目录，编完核日志里被测包确有 `Compiling`。
- 合入验证 `/tmp/claude-1000/merge-and-verify/2026-10-06-2692583/`：harness 170 个测试二进制全绿；fmt、clippy（`--all-targets --all-features -D warnings`）主 agent 补跑退 0；变异表静态核 0 问题，2959 行。
- 实现员在副本上证红：35-1 harness 档 10 行、35-3 八个 B4 窗口全抓、红在断言上；35-2 的 GPU 用例与五行变异、35-1 的四条 GPU 对拍与三行 GPU 变异、35-3 那条 checker 档用例都没跑（checker 档、要 GPU）。35-1 报告：挂载之后的两条对拍流是把录好的流里根槽写拿掉造出来的，构造本身没核过。
- 35-1 自己选的、条款没写的一处：下一次挂载先施加的那一版树表指针全零时红 I-4.1（依据它引的 I-4.1「所选根（重放之后那一版）指向的树全部可完整遍历」）。

方案按 `crates/` 今天的实现来谈。

## 三、格

### P1　FL
两类版本改取末条之后，checker 认的那一版与 core 恢复真施加的那一版是不是逐字节同一份（读 `crates/singlefs-core/src/recovery.rs` 施加记录那一段对照）；末条没落时下一次挂载那一版不再认，I-3.10 与 I-5.4 在「记录落了一部分」的状态上判的东西变了没有、该不该变；由记录施加出来的那一版「有末条取末条、没有退回先到的」这条退路在什么历史上会走到、走到时取的那一份与 core 是否一致；合法历史上多报或少判（随机历史、层 0 前缀都可以量）；GPU 两处与 CPU 是不是同一组版本、同一取法（35-1 说「数组按计数器升序插入」，核 `checker_insert_record`）。

### P2　Q1 甲
由记录施加出来的那一版全零之后红哪几条、与根环里的根全零是不是同一结局；下一次挂载那一版全零红 I-4.1 站不站得住（I-4.1 原文的射程是不是这一版；有没有更贴的不变量；会不会在合法历史上误红），这一格是实现员自选、零轮；I-5.4、I-5.5、删除态、墓碑谱系四处对这两类版本「部分没判 / 射程里判不了」的处置与主走读改判损坏之后是否自洽。

### P3　Q3 甲（GPU 走读内核）
`w_walk_tree_table_and_mapping` 的全零分支在根环里的根、由记录造出来的两类版本、被抛弃的根上红位是不是与 CPU 逐位相同（GPU 跑不了的读码，写「推的」）；四条新对拍流（尤其挂载之后那两条「拿掉根槽写」造出来的）构造得对不对、判的是不是它名字说的那一形。

### P4　Q4 甲与两处说明
I-5.5 新理由的分支在什么条件下走到、与「一次都没判」「没有对象」三者分不分得开；坏镜像（extent 树那一条根指针写零）造出来的是不是「认出那一版而它没有数据单元」这一形，而不是别的形；I-9.28、I-7.4 两处新说明与判定是否一致。

### P5　35-2 GPU 恢复内核择根
新判法与 core `judged` 五段是不是逐段相同（`head_ok` 判的是哪几段，读 `parse_pointer_common`）；四条新用例造的根是不是 core 真不择、GPU 改前真择（GPU 跑不了，写「推的」）；有没有别的 GPU 恢复内核读根记录的入口还只判 extent 偏移。

### P6　35-3
B4 新取样点之后还有没有存活的窗口变体（`.claude/rules/mutation-sampling.md` 第三类）；新池新建文件那条用例里 I-5.4 与 I-3.10 逐状态同集的论证（35-1 改了认法之后还成不成立）；829 行注释。

### P7　变异行
17 行追加、4 行替换：锚点各恰好命中一次、点名的测试跑得到、合入后基线绿、红在断言上不在越界或编译上；955、965 换 ALT1 之后判的还是原来那条判（根记录持有的那棵分配记录树判互不相交）。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | P1–P7 | 从判决、用户定案与条款正推每处改动做的是不是它们说的，每格给判定、引文与推翻条件 |
| 云端攻方（Opus） | P1–P7，重点 P1、P2、P3 | 找反例：一段合法历史让新判法误红，或一个坏实现让它放过；在拷出的副本上造、量 |

## 五、跑前写死的判据

- 两腿同判、主 agent 现查坐实 ⇒ 照那一判；一次抽样的「没打中」只算这一次。
- 两腿分歧 ⇒ 主 agent 核分辨两者的那个事实；核不出就开第二轮，换攻击面。
- 腿提的新判法、新用例算零轮；条款没写、要用户定的，交用户，不由腿或主 agent 定。
- 重型测试不跑：checker 档的测试、GPU 对拍与 `crates/mutations.tsv` 整表都不跑，涉及它们的结论写「推的」；harness 档测试可以跑。

交付：腿的报告写 `research/prompts/m3-formal-code-b35-r1-<腿名>-output.md`，分段落盘（每次写进文件不超过 150 行，第一段排他新建，回复只写短句）；模型与脚本放 `research/prompts/m3-formal-code-b35-r1-<腿名>-model/`，带一份 `tools/rerun.sh`（每条 cargo 经 `scripts/hosts/run-with-memory-cap.sh`，上限按峰值表给或写 `RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON`）；草稿放 `/tmp/claude-1000/<腿名>/`。引 kb 条款写 kb 文件自己的行号（去 kb 文件里现查，不从背景材料里数）。方案按 `crates/` 今天的实现来谈。
