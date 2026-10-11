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
### 小节清单：`.claude/kb/invariants.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 不变量清单 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （不变量清单 标题之下、第一个下级标题之前的正文：第 2-8 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ## 状态 | 不抄 | 这一轮只引 I-2、I-4、I-5 三类；I-7.4、I-3.10、I-9.28 三行用 --extra 按行取（第 58、152、315 行） |
| ## I-1 块自描述 —— 类 | 不抄 | 这一轮只引 I-2、I-4、I-5 三类；I-7.4、I-3.10、I-9.28 三行用 --extra 按行取（第 58、152、315 行） |
| ## I-7 根环 —— 类 | 不抄 | 这一轮只引 I-2、I-4、I-5 三类；I-7.4、I-3.10、I-9.28 三行用 --extra 按行取（第 58、152、315 行） |
| ## I-8 journal —— 类 | 不抄 | 这一轮只引 I-2、I-4、I-5 三类；I-7.4、I-3.10、I-9.28 三行用 --extra 按行取（第 58、152、315 行） |
| ## I-2 校验和 —— 类 | 抄 | 默认全抄（脚本生成） |
| ## I-3 空间记账 —— 类 | 不抄 | 这一轮只引 I-2、I-4、I-5 三类；I-7.4、I-3.10、I-9.28 三行用 --extra 按行取（第 58、152、315 行） |
| ## I-4 事务与崩溃一致性 —— 类 | 抄 | 默认全抄（脚本生成） |
| ## I-5 分配器 —— 类 | 抄 | 默认全抄（脚本生成） |
| ## I-6 加密不变量 —— 类 | 不抄 | 这一轮只引 I-2、I-4、I-5 三类；I-7.4、I-3.10、I-9.28 三行用 --extra 按行取（第 58、152、315 行） |
| ## 待补 | 不抄 | 这一轮只引 I-2、I-4、I-5 三类；I-7.4、I-3.10、I-9.28 三行用 --extra 按行取（第 58、152、315 行） |
| ## I-9 inode 树结构 —— 类 | 不抄 | 这一轮只引 I-2、I-4、I-5 三类；I-7.4、I-3.10、I-9.28 三行用 --extra 按行取（第 58、152、315 行） |

### 小节清单：`.claude/kb/decisions/23-journal的角色与格式.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D23 journal 的角色与格式 —— 已定 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D23 标题之下、第一个下级标题之前的正文：第 2-13 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 已定项 | 不抄 | 分项索引表，用 --extra 按行区间取 |
| #### 已定项 1：journal 的角色本身 | 不抄 | 这一轮只引已定项 15 |
| #### 已定项 2：journal 是定长环还是链式 | 不抄 | 这一轮只引已定项 15 |
| #### 已定项 3：tail 住系统配置槽，改 jbd2 两处 | 不抄 | 这一轮只引已定项 15 |
| #### 已定项 4：记录头约束在单个原子单元内，头 487 字节 | 不抄 | 这一轮只引已定项 15 |
| #### 已定项 5：O2（独立解析器 + checker） 是一元谓词，逐字节比对归记录核对器 | 不抄 | 这一轮只引已定项 15 |
| #### 已定项 6：目标负载优先级移出 journal 这条线 | 不抄 | 这一轮只引已定项 15 |
| #### 已定项 7：一个事务可以跨多条记录，记录头带事务边界字段 | 不抄 | 这一轮只引已定项 15 |
| #### 已定项 8：记录头带一条 32 位反向链 | 不抄 | 这一轮只引已定项 15 |
| #### 已定项 9：jsn = 实例代号 32 位 + 计数器 48 位 | 不抄 | 这一轮只引已定项 15 |
| #### 已定项 10：`previous_hash` 覆盖前一条记录的整个记录头 | 不抄 | 这一轮只引已定项 15 |
| #### 已定项 11：反向链的 hash 取完整 32 位 CRC32C | 不抄 | 这一轮只引已定项 15 |
| #### 已定项 12：记录尺寸取 4 KiB | 不抄 | 这一轮只引已定项 15 |
| #### 已定项 13：载荷完整性，`header_csum` 罩整条记录 | 不抄 | 这一轮只引已定项 15 |
| #### 已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换 | 不抄 | 这一轮只引已定项 15 |
| #### 已定项 15：崩在记录持久之后、根槽持久之前，恢复由记录重建那次发布的根 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 16：mkfs 写实例代号 0，第一次可写挂载取 1 | 不抄 | 这一轮只引已定项 15 |
| #### 已定项 17：点名项 56 字节的字段表 | 不抄 | 这一轮只引已定项 15 |
| #### 已定项 18：在飞记录数上限的取值与语义，tail 字段的编码 | 不抄 | 这一轮只引已定项 15 |
| #### 已定项 19：空发布记录的事务号与提交标记、`previous_hash`、环长 | 不抄 | 这一轮只引已定项 15 |
| #### 已定项 20：记录承载指针层的目标态，分配先于 journal | 不抄 | 这一轮只引已定项 15 |
| #### 已定项 21：世代号：记录粒度的全局单调号，不是 D5（快照 / 空间记账机制） 的 txg | 不抄 | 这一轮只引已定项 15 |
| #### 已定项 22：前缀判定：严格连续、断号即止 | 不抄 | 这一轮只引已定项 15 |
| #### 已定项 23：三个死锁各自怎么破 | 不抄 | 这一轮只引已定项 15 |
| #### 已定项 24：死锁 2 与假性 ENOSPC 是两种故障，要分治 | 不抄 | 这一轮只引已定项 15 |
| #### 已定项 25：D16（发布语义）「不绕过 journal」取读法甲 | 不抄 | 这一轮只引已定项 15 |
| #### 已定项 26：记录头的两条解析纪律 | 不抄 | 这一轮只引已定项 15 |
| ## 历史版本 | 不抄 | 历史节，正文只引现状 |

### 小节清单：`.claude/kb/decisions/19-块指针的结构与宽度预算.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D19 块指针的结构与宽度预算 —— 已定 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D19 标题之下、第一个下级标题之前的正文：第 2-11 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 已定项 | 不抄 | 分项索引表，用 --extra 按行区间取 |
| #### 已定项 1：位置条目带设备身份 | 不抄 | 这一轮只引已定项 3 |
| #### 已定项 2：密文校验和取 32 位 | 不抄 | 这一轮只引已定项 3 |
| #### 已定项 3：定宽，不加密时留位 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 4：位置条目 14 字节 | 不抄 | 这一轮只引已定项 3 |
| #### 已定项 5：位置条目降为提示，中央映射是唯一入口 | 不抄 | 这一轮只引已定项 3 |
| #### 已定项 6：中央映射 key 取按类复用写序键（收严后） | 不抄 | 这一轮只引已定项 3 |
| #### 已定项 7：指针头部带出生 (树, txg)，各 8 字节 | 不抄 | 这一轮只引已定项 3 |
| #### 已定项 8：全部指针进映射，引用侧按类携带映射 key 的各段 | 不抄 | 这一轮只引已定项 3 |
| #### 已定项 9：出生序号怎么发，按什么次序发 | 不抄 | 这一轮只引已定项 3 |
| #### 已定项 10：映射 key 一律 27，条目一律 55 | 不抄 | 这一轮只引已定项 3 |
| #### 已定项 11：指针的偏移表、位置条目的顺序、映射树的根住哪 | 不抄 | 这一轮只引已定项 3 |
| #### 已定项 12：码 2 / 码 3 的映射条目 day-1 写出，固定点一个 checkpoint 只写一次 | 不抄 | 这一轮只引已定项 3 |
| #### 已定项 13：块指针分两段 | 不抄 | 这一轮只引已定项 3 |
| #### 已定项 14：位置条目逐个列出，不用几何公式从单一起点推算 | 不抄 | 这一轮只引已定项 3 |
| #### 已定项 15：extent 距起点偏移的值域与第一版取值 | 不抄 | 这一轮只引已定项 3 |
| ### 与其他决策的连锁 | 不抄 | 这一轮只引已定项 3 |
| ## 历史版本 | 不抄 | 历史节，正文只引现状 |

### 小节清单：`.claude/kb/checks-owed.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 欠的检查清单 | 抄 | 随表头：标题与第 1–16 行由 --extra 一并取 |
| （欠的检查清单 标题之下、第一个下级标题之前的正文：第 2-598 行） | 不抄 | 这一轮只引表头第 1–16 行与 C735（第 584 行）、C738（第 587 行），用 --extra 按行取 |
| ### 已还清 | 不抄 | 这一轮只引表头第 1–16 行与 C735（第 584 行）、C738（第 587 行），用 --extra 按行取 |
**出处 `.claude/kb/invariants.md:1-8`（整段抄，未转述）**

```markdown
# 不变量清单

**checker 是 invariants.md 的可执行形式。** invariants.md 每加一条，checker 就要加一个检查。
两者不同步的 commit 一律不收（见 [format-evolution.md](../rules/format-evolution.md)「硬约束」）。

每条不变量必须写成**可判定**的形式——能对着一个镜像回答"成立/不成立"。池级 checker 对一条不变量的判定有四种：成立、不成立、不适用（必须带理由，不许静默算成立）、部分没判（`InvariantVerdict::PartiallyJudged`，`crates/singlefs-checker/src/image.rs`）。部分没判是判到的那一部分都对得上、射程里另有一部分这张镜像上判不了，说明逐条列出没判到的是什么；它不是红，也不是成立：层 0 与崩溃放量把它算成不红，层 0 计数并进「不适用」那一格，GPU 判器只记违例位、它在那一侧与成立、不适用同是 0。今天报它的不止一条，逐条见各行（例：I-5.4（分配记录罩住的槽互不相交）、I-5.5（从 mkfs 水位建树那一版的 t1 落点）、I-9.28（全部可读根里同一树 ID 的诞生 txg 相同））。
写不成可判定形式的，说明还没想清楚，不许进 invariants.md。

```

**出处 `.claude/kb/invariants.md:125-135`（整段抄，未转述）**

```markdown
## I-2 校验和 —— 类
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-2.1 | 校验和与内容匹配 | 任一被引用的块，其校验和与内容匹配 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判；2026-09-17 起「被引用」按 I-3.1（已分配统计对得上） 那个候选集里的根算：F 之下的根引用的单元可以已被回收复用，不在它们上判——不抬 F 就回收复用时 A 的根还在候选集里、这一条红，是 C22（刚释放的块立即重分配） 的必红） |
| I-2.2 | 校验和算法全盘一致 | 校验和算法标识在整个文件系统内一致。**字段 2026-09-14 用户定案有了**（D22（单元原子性怎么合成） 已定项 9）：系统配置自举头段「校验和算法标识 1」，登记表 0 = 无效、1 = CRC-32C，第一版写 1；读到 0 或未登记的码一律拒绝挂载。算法的四个参数写死在 D18（块里携带什么信息） 已定项 17 | 未实现（条款已写、检查仍欠） |
| I-2.3 | 补齐字节为零且参与校验和 | 任一数据单元（码 1）或打包记录单元（码 3，D18（块里携带什么信息） 已定项 11），其声明长度（净荷字节数；补齐起点 = 头 + 声明长度）之后到单元末尾的补齐字节**恒为 0**，且校验和 / MAC 覆盖**整个单元含补齐**（[decisions.md](decisions.md) D4（校验和位置） 已定项 2：不足一个单元的 extent 补齐到 32 KiB；码 3 由头校验和（到头末尾）与载荷校验和（头之后到单元末尾）两段合起来覆盖整个单元）。补齐字节非零、或校验和只覆盖声明长度，一律判红。索引节点（码 2）从 2026-09-13 起同样在射程内（D18（块里携带什么信息） 已定项 18 用户定案）：声明长度 = 条目区字节数，之后到节点末尾的补齐字节恒为 0，载荷 CRC 覆盖 [86 + 2 × key 宽, 16384)——起点是明文头末尾，含 29 字节预留位、条目区与其后的补齐区，口径与码 1 的 105、码 3 的 107 同（预留位 2026-09-14 从 28 加到 29，含预留位的头是 115 + 2 × key 宽）。⚠️ **加密开启时判的是解密后的明文，不是盘上字节**（2026-09-07 用户定案，还清 C189（I-2.3 在加密开启时判明文还是盘上字节没定））：补齐在盘上是 `E(0)` 而不是 0，按盘上字节判必然全红；checker 本来就要解整单元，取值同一步就有。**与 D18（块里携带什么信息） 已定项 7 的载荷 CRC 方向相反是对的**——载荷 CRC 要给无密钥一侧用，所以算盘上字节；这一条只有 checker 用，所以判明文 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-2.4 | 头校验和覆盖范围 | 任一单元的头校验和覆盖从偏移 0 到该类明文头的最后一个字节，端点写死：码 1 到偏移 104、码 2 到偏移 85 + 2 × key 宽、码 3 到偏移 106（D18（块里携带什么信息） 已定项 7 / 已定项 11 / 已定项 18 的字段表）；头校验和字段自身按 0 参与。**29 字节 nonce / MAC / 算法类型预留位虽然算进头（已定项 16）、也是明文，但不在头校验和覆盖内**——MAC 在头校验和之后才算得出来，罩不住它；单元头的这 29 字节、指针头部的 MAC 16 + nonce 12 与算法类型 1、压缩算法码 1、压后长度 2（D19（块指针的结构与宽度预算） 已定项 3；第一版不加密、压缩算法码登记表只有码 0、码 0 时压后长度恒 0，D9（加密） 已定项 10 / 已定项 14）由同一条规则守：加密关着时恒 0、读者遇到非 0 一律判该结构损坏（已定项 17；指针头部那一半 2026-09-27 主 agent 定，池级判定在 A3-checker-2 里做），加密开启后由 MAC 自己罩（2026-09-14 用户定案把预留位从 28 加到 29，多的那 1 字节是算法类型）。根槽偏移 428..457 的算法类型 1 + nonce 12 + MAC 16（D22（单元原子性怎么合成） 已定项 7 的字段表）是同一条规则的第三处：加密关着时恒 0，池级 checker 对根环里每一条读得出、自证过的根判这 29 字节全 0（不按回退候选集滤），core 的根记录读者不读它们、挂载透明也不比它们（判决 `research/prompts/m3-formal-c655-c687-r1-main-verification.md` 第二节 Y1「算法、nonce、MAC」那一行：单镜像性质，归池级 checker）。指针头部的 extent 偏移 2 字节第一版恒 0（第一版指针指整个单元），读者遇到非 0 判该指针所在的结构损坏，与 MAC / nonce 同一处置、与加密开不开无关（D19（块指针的结构与宽度预算） 已定项 3；`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 11 行）。根记录里四条指针（树表、实例表、中央映射树、分配记录树）头部有一段不是第一版的取值（MAC / nonce 非 0、算法类型非 0、压缩算法码非 0、压后长度非 0、extent 偏移非 0，五样任一）按这条根自证不过处置：core 的根记录读者（`RootRecord::parse_slot`，与 journal 新根段那四条指针同一判法）把这条根算不可择、退回上一条根（靠 journal 重放可以回到同一版，挂载照样成功），checker 红这一条而 core 挂载成功，两边各判各的、不矛盾（`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 6 行）。加密开启（系统配置级、全有全无，I-6.4（无未加密块））后覆盖范围 = I-6.2（明文头字段白名单） 该类白名单里的全部明文头字段，密文段由 MAC 罩。改动覆盖区内任一字节必须失配、改动净荷字节不许失配 | 已实现（`crates/singlefs-checker` 的 `check_unit`：三类的端点各自写死，里程碑步 2 / 步 3 验收里改坏头任一字节判红，2026-09-14；池级走读在 `walk::judge_unit_header`（`crates/singlefs-checker/src/walk.rs:1174` 起）另判单元头偏移 4 的格式版本 = 1（walk.rs:1220，不认得就判该结构损坏、不按今天的字段表往下解）与 29 字节预留位全 0（walk.rs:1258）；走读跟随的每条指针头部按三个判法各判一遍（`crates/singlefs-checker/src/image.rs`：`judge_pointer_mac_and_nonce_are_zero`（:1381）判 MAC 16 + nonce 12 全 0，`judge_pointer_encryption_and_compression_fields_take_the_first_version_values`（:1396）判算法类型、压缩算法码、压后长度三段各一格全 0，`judge_pointer_extent_offset_is_zero`（:1456）判 extent 偏移 0），五个入口是码 2 节点指针（`Walk::read_index_node`，walk.rs:1301 起：树表单元、树表条目的树根、中央映射树、inode 树、extent 树、分配记录树的节点都经它，根记录里树表、中央映射树、分配记录树那三条根指针也经它）、墓碑树与谱系树内部条目的子指针（`walk_the_entries_of_a_container_tree_node`，walk.rs:2150 起）、inode 内部条目子指针（`walk_inode_internal_node`，walk.rs:2443 起）、实例表链指针（`walk_instance_table_chain`，walk.rs:2836 起，根记录里的实例表指针是链的第 0 片）、extent 数据指针（`walk_extent_data_pointer`，walk.rs:3009 起），只判不断、指针指向的单元照样走下去，2026-09-27 实审 A3-checker-2；根槽偏移 428..457 那一处也已实现（`image::judge_root_record_algorithm_nonce_and_mac_are_zero`，image.rs:1431，`walk::check_pool_image` 经 `check_pool_image_given_the_tombstone_container_capacity_knob` 在 `valid_roots` 的结果上调它，walk.rs:10413）；坏镜像 `crates/singlefs-harness/tests/checker_judges_reserved_bytes_and_widths_it_used_to_skip.rs`）。**extent 偏移那一半与算法类型、压缩算法码、压后长度那几段已实现**（checker 走读跟随的每条指针都判这三格；GPU 判器走读内核 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` 的 `w_judge_pointer_head` 与 `w_judge_pointer_extent_offset` 在同样那几个入口判，根槽那一处是同文件的 `judge_root_record_algorithm_nonce_and_mac_are_zero`；core 每个读盘上指针头部的读者都判这五段（MAC / nonce、算法类型、压缩算法码、压后长度、extent 偏移）、判该结构损坏：树表条目的树根指针、inode 内部条目的子指针、extent 叶记录的数据指针、extent 上段叶条目的下段根指针与内联数据指针、extent 树内部条目的子指针、分配记录树内部条目的子指针、实例表链指针各一处，加 journal 新根段、码 2 内部条目、根记录三处（`crates/singlefs-core/src/pointer.rs` 的 `read_judging_the_encryption_compression_and_extent_offset_fields_from`，`PointerHead`、`NodePointer`、`DataPointer` 各一个；`grep -rn 'Pointer::read_from' crates/singlefs-core/src/` 只剩 `pointer.rs` 自己的单测一处）；影子账认单元的两个读者（`allocation_record_tree::node_pointers_as_far_as_readable`、`recovery::instance_table_page_pointers_as_far_as_readable`）经 `NodePointer::read_with_head_fields_judgement_from` 的 `NotJudgedByTheShadowLedger` 不判（影子账要多隔离、不漏隔离）；走读不跟随的指针 I-2.4 不判：根环里走读没走到的旧根，它们的树表单元只被 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同） 读条目取诞生 txg、不跟随条目里的树根指针，这些指针的头部没人判（测过的镜像上 12 条：三个旧树表单元各四条树根指针，`research/prompts/m3-formal-code-b30-r1-sonnet-output.md`；core 把那样一条根当恢复目标时经 `TreeTableEntry::parse` 判，推的、没造盘面验证）；坏镜像遍历偏移 32、33，core 每类读者各一条用例在 `crates/singlefs-harness/tests/pointer_extent_offset_that_is_not_zero_is_refused_by_every_tree_reader.rs`，加密、压缩那几段 core 每类读者各一条在 `crates/singlefs-harness/tests/pointer_encryption_and_compression_fields_outside_the_first_version_are_refused_by_every_tree_reader.rs`，根记录四条指针各改一类字段、两种挂载都退到上一条根的在 `crates/singlefs-harness/tests/root_record_pointer_encryption_and_compression_fields_outside_the_first_version_are_refused_by_both_mounts.rs`；池级 checker 的坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`：extent 偏移落在墓碑树、谱系树的容器子指针上（`an_extent_offset_on_the_container_pointer_of_the_tombstone_tree_root_reddens_only_the_pointer_head_invariant` 与谱系树那一份），算法类型、压缩算法码、压后长度各一份（`an_inline_data_pointer_naming_an_encryption_algorithm_reddens_only_the_header_coverage_invariant`、`a_tree_table_root_pointer_naming_an_unregistered_compression_algorithm_reddens_only_the_header_coverage_invariant`、`an_inline_data_pointer_carrying_a_compressed_length_without_compression_reddens_only_the_header_coverage_invariant`），根槽算法类型、nonce、MAC 各一份（`a_root_slot_whose_algorithm_type_is_one_reddens_only_the_header_checksum_coverage_invariant` 等三条）；对应的账是 C581（extent 偏移非 0 没有判据）；`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 11 行、`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 5 行、`research/prompts/m3-formal-code-b29-r1-main-verification.md` 第二节 L4 的容器子指针那一行、`research/prompts/m3-formal-code-b30-r1-main-verification.md` 第二节 M2 的算法类型等三段与旧树表根指针两行、`research/prompts/m3-formal-code-b33-r1-main-verification.md` 第二节 P2 的根记录四条指针那一行） |
| I-2.5 | 位置条目按设备身份升序 | 任一块指针、中央映射条目的 value、journal 点名项里的位置条目数组按设备身份严格升序（D19（块指针的结构与宽度预算） 已定项 11、D23（journal 的角色与格式） 已定项 17，2026-09-13 用户定案）；两条位置条目设备身份相同或逆序，一律判损坏。⚠️ **射程（2026-09-14 补）：整条 86 字节全零的指针表示「注册了但还没有根」，整条豁免升序判定**——它内部那两条位置条目都是 (设备 0, 槽 0, 校验和 0)，按字面正是「设备身份相同」那一档。今天写出全零指针的有四处：树表里 livelist 树、稀疏旁表树与 deadlist 树那三条条目（D6（快照实现模型） 已定项 2、D5（快照 / 空间记账机制） 已定项 6 与 D5（快照 / 空间记账机制） 已定项 11 各自定案的「day-1 注册进树表，根指针为零」），以及 mkfs 与暖机的根记录里那个还没发布过的中央映射树根指针。非全零的指针不享这条豁免。 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |

```

**出处 `.claude/kb/invariants.md:171-189`（整段抄，未转述）**

```markdown
## I-4 事务与崩溃一致性 —— 类
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-4.1 | 重放后树可遍历 | 任一崩溃点重放后，所选根（重放之后那一版）指向的树全部可完整遍历。起点含介质损坏的镜像（前缀停在一条点名单元两份没都验过的记录上）报不适用。另三处报不适用：所选根那次发布读得出的记录里带末条标志的多于一条（锚点认哪一条条款没写，实现在任何写之前拒挂，重放不出一版）；所选根的 checkpoint_txg 是 u64 的最大值（接在它后面的发布取不到 txg）；所选根之上一次发布都不施加（重放之后那一版就是所选根，可不可遍历归 I-7.2（最新根可完整遍历））——没有记录压在最新根之上的镜像上这一条因此恒报不适用、不报成立。重放用的读环按 (实例代号, 计数器) 建键，两块盘同一计数器坐着不同实例的记录时两条都留，与实现恢复同键（`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 2、8 行）。 | 已实现（2026-10-02，池级 checker `walk::judge_trees_of_the_replayed_version_are_traversable`：R-独，checker 自己照 D23（journal 的角色与格式） 已定项 14 的前缀规则复刻只读重放（`replay_the_next_mount_would_do`），施加出来的那一版（所选根换上末条记录新根段的七项：树表指针、中央映射树根指针、树 ID 水位、F、实例表指针、分配记录树根、卸载记号，只有 fsid 照所选根；D23（journal 的角色与格式） 已定项 15）另起一份走读，走读断了判红；在飞上限按环长 ÷ 4096 ÷ `JOURNAL_SAFETY_FACTOR` 现算。读环键 2026-10-03 改成 (实例代号, 计数器)（`records_the_replay_reads`），反向链那一关改按 (实例代号, 计数器 − 1) 找前一条（`record_back_chain_lets_it_into_the_replay_prefix`），消掉两块盘同一计数器坐着不同实例的记录时与实现分开的形态（推为第一版两盘池上不可达）；`records_as_the_recovery_reads_them` 与 `record_can_enter_a_replay_prefix` 两处同改按 (实例代号, 计数器) 建键，已做（主 agent 2026-10-03 定）。坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`（重放六条规则各一份、两块盘同计数器不同实例两份）；层 0 里与 core 的 `effective_root` 逐状态差分今天没有能跑的入口，要先开口子，进下一批清单（`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 8、9 行）；`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 2 行与第四节用户定案） |
| I-4.2 | 无被引用未提交块 | 任一崩溃点重放后，不存在被引用但未提交的块 | 已实现（2026-09-22，池级 checker `walk::check_pool_image` 的 `judge_birth_identity_of_a_referenced_unit`：走读到的每个被引用单元，头里的诞生代号与写序（D18（块里携带什么信息） 已定项 7 单元头字段表里的字段）按 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2 的已发布谓词判成已发布——**「未提交」取的就是「按那个谓词判不成已发布」**，谓词与 I-1.8（归并后版本全序） 扫描方向共用同一份 `PublishedPredicate`。打包记录类型 4 的实例表单元按 I-1.2（块头写序已发布） 那一行的例外不走谓词，不进这一条。坏镜像是把数据单元头里的诞生代号与引用它的 extent 记录里那条数据指针的诞生代号一起抬到最新根的 txg 之上——指针仍与头一致 ⇒ I-1.2（块头写序已发布） 不跟着红，分得开两条的就是这一份（`crates/singlefs-harness/tests/checker_known_bad_images.rs`）；层 0 每个崩溃状态都跑它。⚠️ **「任一崩溃点重放后」那半句没实现**：池级 checker 判的是手上这一份镜像、不跑恢复，判的是「被引用 ⇒ 已发布」这个态上的性质；重放之后再判一遍归层 0） |
| I-4.3 | 提交原子 | 提交是原子的：不存在"部分生效"的事务 | 未实现（它要判的是 D23（journal 的角色与格式） 已定项 7 那两个事务边界字段——事务号与提交标记——施加之后的效果。**对着一份镜像判不出来，没硬写**：「部分生效」要拿「这次事务该有的全部效果」当 oracle 才分得出来，而一份镜像上任何状态都自洽——同一份字节既可能是「事务整体没生效的旧态」，也可能是「生效了一半」，池级 checker 手上没有第二个输入。判它的是层 0 崩溃点重放加 oracle：`.claude/kb/milestone/02-second-txn.md` 第 486 / 487 行那两条（崩在第二条记录之前 ⇒ 整个事务不施加、文件是旧长度；把提交标记提前到第一条记录 ⇒ 恢复施加了半个事务、I-4.3（提交原子） 红）说的就是这条路。形态同 C116（叶容器的分裂 / 合并纪律没有会失败的检查）：操作断言，不进池级 checker，恒真的检查比没有检查更糟） |
| I-4.4 | 意图只描述目标态 | 任一未完成的意图 key，其字段只描述目标状态，不含任何增量字段（"再删 N 个"一类） | 已实现（2026-10-05，在清单里的第一版恒报不适用：池级 checker 在 `walk::check_pool_image` 里对它调 `not_applicable`，理由逐字是 `crates/singlefs-checker/src/image.rs` 的 `INTENT_IS_NOT_APPLICABLE_IN_THE_FIRST_VERSION`；不要求坏镜像，`crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `INVARIANTS_WITHOUT_A_BAD_IMAGE` 点名；GPU 判器下标 73，内核不置位）。第一版不适用：意图载体（nlink = 0 的记录、`deleted_inodes` 的 key、墓碑）没有可装增量的字段，checker 报不适用并写理由；logged_ops 有字段表时实现（用户 2026-10-05 定，岔路单 `research/prompts/m3-delete-checker-r1-forks.md`） |
| I-4.5 | 已停用·需两镜像 | **此编号不再使用**——幂等要比对两个镜像，单镜像判不了；判据落在 [experiments.md](experiments.md) E3（意图日志的断电语义） 判据③ | 不适用 |
| I-4.8 | 近 K 代根校验和自洽 | 任一崩溃点重放后，从回退候选集（D23（journal 的角色与格式） 已定项 14：按实例表判仍然有效 ∧ txg ≥ F_生效）里**任一**根出发遍历，所有块的校验和均与其父指针记录的一致（候选集的深度由 D16（发布语义） 已定项 1 的回退下界 F 定：平时整个根环，盘紧时缩到最近 4 个可退到的状态，每块盘上最新的持久有效根都在内）。**判别力**：抓「本事务内释放的块被重新分配并写入」这一类——那会让旧根指向的块被写入新内容，**使旧根成为假的回退候选**，而只验最新那一代根是发现不了的 | 已实现（2026-09-17，池级 checker `walk::check_pool_image`，最新根与候选集里每条更早的根各判一格：从它出发的遍历有单元对不上或读不出就红；坏镜像是 mkfs 树表单元抹成零；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-4.6 | 中间态其余不变量成立 | 存在未完成意图的镜像上，其余全部不变量仍然成立（分批之间的中间态是合法状态）。「其余全部不变量仍然成立」由各条在每个被判的版本上各自判，中间态的版本也在内；I-4.6（中间态其余不变量成立） 自己判的是中间态特有的一格：每条 nlink = 0 的记录，数据单元数与 size 对得上——size 为 0 时 0 或 1 个单元，size > 0 时 ⌈size ÷ 数据单元净荷容量⌉（实现 `crates/singlefs-core/src/write_request_split.rs` 第 61 行 `div_ceil(...).max(1)`）。判哪几版、配对与读不全的报法照 I-9.5（删除队列里的号仍在树里） 的共用判法 | 已实现（2026-10-05，池级 checker `walk::check_pool_image`，函数 `judge_data_unit_counts_of_the_inodes_being_deleted_against_their_size`；净荷容量 checker 自己算一份（`data_unit_payload_capacity_in_bytes`）；每条 nlink = 0 的记录判一次，一条都没有时报不适用并带理由；坏镜像：第一个文件的 size 加一个净荷（`crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs` 的 `raise_the_size_of_the_first_file_by_one_data_unit_payload`）；GPU 判器下标 72，扫描内核判；用户 2026-10-05 弹窗定「只判单元数对 size」，出处 `research/prompts/m3-delete-checker-r1-main-verification.md` 第四节与岔路单 `research/prompts/m3-delete-checker-r1-forks.md`；射程只对 nlink = 0 的记录，每条记录都判的那一形不挂在这一行名下，另记欠账 C711（单元数对 size 只判了 nlink = 0 的记录）） |
| I-4.7 | 删除中 inode 有账 | 任一处于删除中的 inode，要么在删除中队列里有条目，要么其 extent 已全部释放——两者皆非的中间态不存在。「处于删除中的 inode」指两样：inode 树里 nlink = 0 的记录；extent 树里有数据单元、而 inode 树里没有 nlink ≥ 1 的记录的号。判哪几版、配对与读不全的报法照 I-9.5（删除队列里的号仍在树里） 的共用判法 | 已实现（2026-10-05，池级 checker `walk::check_pool_image`，函数 `judge_inodes_being_deleted_against_the_queue_and_their_data_units`；每个处于删除中的号判一次，一个都没有时报不适用并带理由；坏镜像：extent 树里有数据单元、inode 树里没有它的记录、号也不在队列里（`crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs` 的 `drop_the_record_of_the_first_file_and_keep_its_data_units`）；GPU 判器下标 71，扫描内核判；对象取这两样，出处 `research/prompts/m3-delete-checker-r1-main-verification.md` 第二节逐格判决表里 I-4.7（删除中 inode 有账） 对象那一格） |
| I-4.9 | 非最新候选根引用的单元不晚于它 | 回退候选集里不是最新的每一条根 R，它引用的单元（按落点认，读落点上单元头里的诞生代号，头校验和过）的诞生代号 ≤ R 的 `checkpoint_txg`。最新那条根归 I-4.2（无被引用未提交块），不在这一条里。依据 D16（发布语义） 已定项 7（单元先于根落盘）。 | 已实现（2026-10-02，池级 checker `walk::judge_unit_birth_against_an_older_candidate_root` 判走读里每个被引用的单元，共享子树里别的版本先走过的单元由 `merge_placements_below_a_unit_walked_by_an_earlier_version` 补判；候选集只剩一条根时报不适用，同 I-3.9（释放代落在停止引用它的那一格区间里）；坏镜像沿 I-4.2（无被引用未提交块） 那一份的造法，映射 key 也要跟着改，在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 1 行） |

> ⚠️ **编号只往上加，不回收、不重编。** 回收一个编号会让别处的旧引用悄悄指向另一条陈述，
> 而检索只端出其中一条，不会告诉你它换了意思（`.claude/singlefs-ai-sop/rules/kb-discipline.md` 第 4 条「矛盾比空白更糟」）。
> 退役的编号留一行说明去向，比留一个无解释的空档安全。

```

**出处 `.claude/kb/invariants.md:190-212`（整段抄，未转述）**

```markdown
## I-5 分配器 —— 类
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-5.1 | 物理范围不重叠 | 不存在被两个不同引用同时占用的物理范围（除非是显式的多副本） | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） 引用按版判（2026-09-27，`research/prompts/m2-rev-b1-implementer-report.md` 第六节第 1 条、`research/prompts/m2-rev-b2-implementer-report.md` 第八节）：同一版（根环里的一条根，或由记录施加出来的一版）里同一个 (设备, 槽) 被两条指针引用判红，(设备, 槽) 取自位置条目的设备 ID 与物理偏移（D19（块指针的结构与宽度预算） 已定项 4），一条指针自己的两条位置条目同落点算一个引用（那一格归 I-2.5（位置条目按设备身份升序））（`walk::Walk::note_references_of_a_pointer_followed_by_the_walk`）；中央映射条目指的物理范围算进引用集合；别的版本走过的共享子树不重走，按缓存把它下面的落点并进这一版再比、不读盘（`walk::Walk::merge_placements_below_a_unit_walked_by_an_earlier_version`）。坏镜像在 `crates/singlefs-harness/tests/checker_cross_links_malformed_nodes_and_mapping_entries.rs`。 |
| I-5.2 | 空闲统计对得上 | 空闲空间统计 == 总空间 − 已分配空间 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-5.3 | 报出的空闲都兑现得了 | `df` 报给用户的空闲 ≤ 当下可分配 + 能在有界步数内放回的扣住量（defer 窗口里、回退下界以下的释放）。运行时的等价形态：`df` 报出 s 字节空闲时写 s 字节必须成功；删掉 s 字节之后同样大小的写在有界步数内成功（[decisions.md](decisions.md) D3（空间分配） 已定项 9，用户定案）。界随 D16（发布语义） 已定项 1 的形态定。判别力：把「放回扣住量」那一步摘掉，近满盘上删一个对象再写同样大小必须由绿转红 | 未实现 ⚠️ **口径注（2026-09-13，D28（挂载期承诺量） 已定项 4）**：`df` 要扣的 checkpoint 保留池是每次发布现算的量，发布即重算，近满盘上报出的空闲会随树高变小。 |
| I-5.4 | 分配记录罩住的槽互不相交 | 判的版本是回退候选集里每条根，加由记录施加出来、根槽从没落盘的那几版（已被一次恢复施加过、实例表里有行，树表指针与「树表 0 条那一版的分配记录树根」取自记录的新根段）与下一次挂载先施加的那一版（记录与最新根同实例、txg 比最新根大 1、带提交标记，根环里没有它的根）；这些版本的分配记录树里，同一块盘上任意两条分配记录（不论已分配、已释放、释放代是否已低于回退下界）罩住的槽区间 [槽号, 槽号 + 跨度) 互不相交。复用一个已回收的落点时，新记录罩住的槽上别的已回收记录要随这次分配一起删掉；留着它，下一次可写挂载重建分配器时两条记录罩住同一个槽。同一条下另判一格：树表非空（带树）的那一版，根记录里「树表 0 条那一版的分配记录树根」那 86 字节必须全零（D22（单元原子性怎么合成） 已定项 7 字段表「分配记录树根指针」那一行），非零就是这一版有两棵分配记录树，判违例。这一格成立的那一次，只在这一版自己的分配记录树（树表种类 3 那一条指着的那一棵）走到叶时计进 I-5.4 的评估，没走到叶的版本不计。部分没判（逐版列出，不判红；判出违例照报违例）：树表指针非零而树表读不出的版本、树表指针全零的版本（core 的根记录读者照它读盘 0 槽 0、读不出），都认不出带不带树，根记录里分配记录树根全零那一格与它自己的分配记录树都没判；树表读得出、0 条而分配记录树由根记录直接持有（C512（树表 0 条的一版上被换下的实例表记在哪没有条款））、那棵树读不出的版本；带树一版自己的分配记录树没走到叶的版本。候选集里没有一棵分配记录树走得到叶（第 0 代树表是空的，或树表 / 分配记录树读不出、位置对不上）报不适用。 | 已实现（2026-09-18，`crates/singlefs-checker` 的 `walk::judge_allocation_records_disjoint`，吃一张版本表（`VersionToJudgeTheAllocationRecordsOf`：称呼、树表指针、树表 0 条那一版的分配记录树根），表里是回退候选集里每条根、由记录施加出来的那几版与下一次挂载先施加的那一版（`check_pool_image` 拼表）；GPU 判器扫描内核 `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl` 的 `judge_allocation_records_disjoint_of_a_version`，候选根、由记录施加出来的版本、下一次挂载先施加的那一版各调一次，只有违例位、部分没判不置位；这两类版本上的坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `two_allocation_records_covering_one_slot_in_a_version_applied_only_by_its_journal_record_redden_only_the_disjointness_invariant` 与 `two_allocation_records_covering_one_slot_in_the_version_the_next_mount_applies_first_redden_only_the_disjointness_invariant`，读不出的几格各有用例（`the_tree_table_of_a_version_applied_only_by_its_journal_record_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged`、`the_allocation_record_tree_held_by_the_root_record_of_a_version_without_trees_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged`、`an_all_zero_tree_table_pointer_in_an_older_candidate_root_is_followed_like_the_core_and_every_tree_table_reading_names_it_unjudged` 等）；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `an_allocation_record_whose_span_covers_the_next_record_reddens_only_the_disjointness_invariant`；层 0 每个崩溃状态都判，同日在新代码上全量：第二条流 2,104,413 个状态里评估 1,842,252 个、违例 0，其余 262,161 个报不适用，第一条流 262,165 个状态里评估 4 个、违例 0（`m2-wave2-crash-verifier-report.md`「主 agent 要的两个数」）。同日立，依据 `research/prompts/m2-wave2-code-r1-main-verification.md` 第二节第 1 行：复用时跨度变了留下重叠记录，下一次可写挂载在重建分配器时 panic，而 I-5.1（物理范围不重叠） 只管树里的引用、判不出记录之间的重叠；带树一版分配记录树根全零那一格的挂法见判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J1「I-5.4 挂法」那一行与 `research/prompts/m3-formal-code-b29-r1-main-verification.md` 第二节 L2「成立要至少一棵树走到叶」那一行，树表读不出与根记录直接持有的树读不出两形见 `research/prompts/m3-formal-code-b30-r1-main-verification.md` 第二节 M2「I-5.4 按版本：坏盘」那一行与 `research/prompts/m3-formal-code-b33-r1-main-verification.md` 第二节 P2 第一行，射程含两类由记录造出来的版本见同一节 P2「由记录施加出来的版本与下一次挂载先施加的那一版」那一行，树表指针全零的版本见同一节 P2「树表指针全零的根」那一行与第五节第 1 条） |
| I-5.5 | 从 mkfs 水位建树那一版的 t1 落点 | 走读到的任一版 V（候选集里每条根，含由记录施加出来的那几版）若是从 mkfs 水位第一次建树的那一版——① V 的树表非空，且种类 1（extent 树）条目的树 ID 等于 65536；② V 的树表里每条条目的诞生 txg（条目偏移 108）都等于 V 的 `checkpoint_txg`——则 V 引用的数据单元里，每块盘上落点槽号最小的那一个，槽号等于这一版分配记录里 D3（空间分配） 已定项 8 第 1 条那条政策算出的那一对的起点槽号（起点最小、32768 对齐、两槽都空、不在开放段里）。新池新建文件那一次是 U + 4，U 是所选系统配置偏移 417 的单元区起始槽号（默认 768 MiB 环 U = 50176，所以是 50180）；回收之后（例如 mkfs 的实例表 U、U + 1 那两槽被回收）不再是 U + 4。V 没有数据单元时不判；不满足 ①② 的版本不判。这一类含新池新建文件，也含 mkfs 之后重挂、实例 1、2、3 从 mkfs 水位写的第一个文件（用户 2026-10-04 定案，原话在决策变更史 D3（空间分配） 节）。只判 t1，t2–t12 不判（D3（空间分配） 已定项 10 射程没有「偏离即不合法」）。已知限度：这一版盘面上复算不出的几样只能近似——已释放的记录一律当占着（回收门槛复算不出：写死从 U + 4 起扫的坏实现与回收门槛放宽的坏实现两个方向都判不出，C689（建树落点判据抓不到写死从 U + 4 起扫的坏实现））；这一版自己分配的记录当空；开放段拿这一版树表单元所在的段当它（这次挂载更早开过、已经用尽的段认不出；提交内生块回落时会误排：这要求这一版发布时每块盘上都没有全空的 64 槽段，今天的写侧在「只暖机 / 只写行的挂载接发第一个文件」这一族里量到 0 次回落（单元区 240 / 256 / 384 槽，连续挂载 k ≤ 60，一次抽样、不是证明）；分配器上留了回落计数，`crates/singlefs-harness/tests/allocator_fallback_count_on_small_devices_after_repeated_mounts_and_the_first_file.rs` 断言它是 0，哪天走进回落、用例红，回来重看开放段认法）；隔离位、抬 F 扣住位与保护期内的 R 盘上都没有，没有分配记录罩着的槽一律当空。依据 D3（空间分配） 已定项 8 射程与已定项 10 ③ ④，以及判决 `research/prompts/m3-formal-code-b17-r1-main-verification.md` 第二节 K2 与用户 2026-10-04 对判据改法的定案（原话在决策变更史 D3（空间分配） 节）。 | 已实现（判据是 f2（按这一版分配记录现算），代码轮判决 `research/prompts/m3-formal-code-b19-r1-main-verification.md`；池级 checker `crates/singlefs-checker/src/walk.rs` 的 `judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark`：候选集里每条根与由记录施加出来的那几版各自重读树表与 extent 树判，U 取所选系统配置偏移 417；GPU 判器扫描内核 `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl` 的 `judge_first_data_unit_of_a_version`，`IMPLEMENTED_INVARIANTS` 下标 67；判不了的几格报部分没判（`InvariantVerdict::PartiallyJudged`，没有违例时整条报它，逐版列出实例、txg 与为什么，形态同 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同））：这一版的树表读不出（含树表指针全零：core 的根记录读者照它读盘 0 槽 0、读不出）或条目宽不到 200 字节，认不出这一版是不是这一类；认出来的那一版 extent 树读不全（数据单元数不全）、分配记录树读不全（落点算不出）；种类 1 条目不止一条、种类 3（分配记录树）条目不是恰好一条，这两格的树表本身由 I-9.16（树表条目按树 ID 严格升序且合发号次序） ③ ④ 判红、这里只记没判；有数据单元而池里某块盘上一个都没有，那块盘没有对象，跳过、不记（CPU 与 GPU 一致）；GPU 判器只有违例位，这几格不置位；走读到的版本里一版都不是从 mkfs 水位第一次建树的那一版时，整条报专门的不适用理由（`walk::NO_VERSION_BUILDS_TREES_FROM_THE_MKFS_WATERMARK`，`crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs` 里那条用例的 I-5.5 一支只认它），认出来而没有数据单元那一形仍落缺省理由；三格的坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `a_tree_table_that_does_not_read_back_leaves_the_first_data_unit_invariant_partially_judged`、`an_extent_tree_that_does_not_read_back_leaves_the_first_data_unit_invariant_partially_judged`、`an_allocation_record_tree_that_does_not_read_back_leaves_the_first_data_unit_invariant_partially_judged`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`（数据单元从 50180 搬到 50182），合法镜像在 `crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs`） |

> ⚠️ **I-3.1（已分配统计对得上） 与 I-5.2（空闲统计对得上） 的等式在 zoned 布局上有一个已知缺口（[decisions.md](decisions.md) D12（目标介质） 定了多布局之后才出现）。**
> zoned 设备上一个 zone 里只要还有一块活数据就不能 reset ⇒
> **一个活快照钉住的物理空间是 zone 粒度，而它引用的块数是块粒度**。
> 于是「已分配 == 遍历所有引用得到的和」在那套布局上不成立，
> 除非把「已分配」明确定义为「被引用」，**并另立一个「不可回收」量**。
>
> 连带一条对用户可见的要求：**报给用户的「这个快照占了多少」不许被定义成引用块数之和**——
> 那个数在 zoned 上会**系统性地低于**删掉该快照真正能拿回的空间，
> 而 [pitfalls.md](pitfalls.md) 第 12 条骂的正是这类「`df` 输出对用户没有意义」。
>
> **本缺口尚未有对应条目**，补它需要先定义「不可回收空间」这个量。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:1-13`（整段抄，未转述）**

```markdown
## D23 journal 的角色与格式 —— 已定

<!-- doc-lint:not-numbers CRC-64 SSE4.2 CRC32C SHA-256 CJ2 -->

D23（journal 的角色与格式） 管 journal 是什么、写什么、崩溃之后怎么用它：记录承载什么与分配的先后（已定项 20）、每次 fsync 写什么（已定项 1）、环的几何、记录尺寸与在飞上限（已定项 2 / 12 / 18 / 19）、tail（已定项 3 / 18）、记录头与点名项的字节（已定项 4 / 7 / 8 / 9 / 10 / 11 / 13 / 15 / 17 / 21）、前缀判定与重放下界（已定项 14 / 22）、管理员回退、失败处置与实例切换（已定项 14 / 16）、与 journal 相关的三个死锁和保留池（已定项 23 / 24 / 25）、记录头的解析纪律（已定项 26）。不管的：发布的持久顺序、checkpoint 的触发与暖机归 D16（发布语义）；实例表的行格式与写行规则归 D18（块里携带什么信息） 已定项 11；根记录与根环归 D22（单元原子性怎么合成）；保留池与切换预留怎么进准入不等式归 D28（挂载期承诺量）；checker 与记录核对器各是什么归 D13（验证路线） 已定项 6 / 7；目标负载归 D25（目标负载优先级）。

<!-- doc-lint:registry name-col=2 -->

| 编号 | 简称 |
|---|---|
| T6 | 维持拒绝时拒不拒取决于池的历史 |
| L8 | 并行线三跨记录层0流 |

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:450-464`（整段抄，未转述）**

```markdown
#### 已定项 15：崩在记录持久之后、根槽持久之前，恢复由记录重建那次发布的根

**定案**：**由记录重建那次发布的根：记录头加「新根段」= 树表单元指针 86 + 中央映射树根指针 86 + 树 ID 水位 8 + 回退下界 F 8 + 实例表单元指针 86 + 分配记录树根指针 86 + 卸载记号 4 = 364 字节 <!-- format-const: JOURNAL_NEW_ROOT_SEGMENT_BYTES = 364 stale=新根段 188 字节|JOURNAL_NEW_ROOT_SEGMENT_BYTES = 188 -->，4096 记录装 64 个点名项。施加一条记录 = 把所选根的这七个字段换成记录里的（实例表指针、卸载记号、分配记录树根从新根段来，不再照抄所选根），不需要树语义。一次发布切成几条记录时，每条记录都带同一份新根段。已定项 14 条 2「严格大于所选根就施加」原样成立。**

**射程**：管「记录在、根槽不在」那一格恢复施加什么；指针的宽度随 D19（块指针的结构与宽度预算） 已定项 7 / 11 走。施加一条记录时指针层上其余的事（分配器登记、overlay）不在这一格，归 C284（施加一条记录在指针层上做什么没有定义） 与 D28（挂载期承诺量） 已定项 2。

**依据**：

- E142（新池新建文件的干跑）：新根段加进记录头之后，「施加一条记录」在指针层上有了定义，忽略 journal 与查 journal 的恢复结果在一批状态上不同——journal 承重。
- 用户定案，原话在变更史。
- 用户定案 2026-10-01：「多条记录的发布每条都带同一份新根段」补进已定项 15（`research/prompts/m3-inv-b-r1-main-verification.md` 第四节第 3 条），原话在变更史。
- 三方判决 `research/prompts/m3-ring-r3-main-verification.md` 第二节第 1、2 行：施加处的实例表指针、卸载记号、分配记录树根三项改从新根段来（首个文件、卸载两种发布补进去之后施加出来的根与当初会写的逐字节相同，攻方量的；专用清扫发布崩在记录已持久、根槽没持久时，实例表指针照抄所选根会让下一次可写挂载在取号之前被拒，攻方在副本上量的）；用户定案 2026-10-02「回到甲」，原话在变更史。无实验：数都在三方副本上，没立实验号。

**欠**：C284（施加一条记录在指针层上做什么没有定义）。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:1-11`（整段抄，未转述）**

```markdown
## D19 块指针的结构与宽度预算 —— 已定

D19（块指针的结构与宽度预算） 管块指针这一格：指针分几段、每段有哪些字段、各多宽、按什么次序排；位置条目与中央映射谁是解引用与释放判定的权威；中央映射条目的 key 与 value 怎么编码；映射 key 的哪几段跟着指针走，出生序号怎么发。不管的：单元头的字段表与单元类型码在 D18（块里携带什么信息）；节点大小与树内条目布局在 D8（核心索引结构）；落点坐标系与分配记录的 key 在 D3（空间分配） 已定项 3 / 7，提交内生块的 bump 次序在 D3（空间分配） 已定项 10；校验和覆盖多大、逻辑 extent 与单元的关系在 D4（校验和位置）；加密算法、nonce 怎么产出、AAD 期望值从哪来在 D9（加密）；根记录与系统配置的字段表在 D22（单元原子性怎么合成） 已定项 7 / 9；条带宽度的粒度在 D2（RAID 条带策略） 已定项 1。

<!-- doc-lint:registry name-col=2 -->

| 编号 | 简称 |
|---|---|
| N1 | 读不出先重读一次 |
| N3 | 对不上的留在已分配 |

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:60-73`（整段抄，未转述）**

```markdown
#### 已定项 3：定宽，不加密时留位

**定案**：指针按最坏情形定宽——头部里的 MAC（128 位）与 nonce（96 位）恒定占位，不加密的卷把它们留成空位，不用类型标记省掉；这些位在所有卷上都占着，也不挪去装别的字段。

**射程**：可选的是 AAD 那道运算（加密关着就不算），不是格式（`.claude/rules/fs-design.md`「加密是可选的运算，不是可选的格式」）。按介质开放布局分支那个机制管不到这一格：incompat 位管的是读者认不认识这套布局，这一格管的是在同一套布局里一个指针内部怎么编码；先定宽、以后再加一套变长布局不是扩张，是多一套要永远维护的解析器加 checker 加崩溃点重放。不加密时白占 MAC 16 字节 + nonce 12 字节 = 28 字节 / 逻辑 extent，按 D4（校验和位置） 已定项 7 的口径（逻辑 extent 占盘恒 32768 字节）是 `28 / 32768 = 0.0854%`。变长那一侧省的是那 28 字节、不是省一个字段（算法类型字段 D9（加密） 已定项 12 第 4 项本来就要有），要付的是解析必须从头走：定宽指针可以按偏移直接定位第 k 个，变长不行；「恢复期扫一个部分损坏的节点时能不能跳过坏掉的那一条继续读」是推论，未实测。 加密关着时 MAC 16、nonce 12 恒 0，读者遇到非 0 判该指针所在的结构损坏，同 I-2.4（头校验和覆盖范围） 给单元头 29 字节的读法（实审 A3a 落地在两处读者——多层码 2 树内部条目、journal 新根段指针：`crates/singlefs-core/src/pointer.rs` 的 `PointerHeadFieldOutsideTheFirstVersion::MacOrNonceNotZero`）；extent 偏移那 2 字节第一版恒 0（第一版指针指整个单元），读者遇到非 0 判该指针所在的结构损坏，与加密开不开无关；checker 在走读跟随的每条指针上判，core 每个读盘上指针头部的读者都判，落点在 [checks-owed.md](../checks-owed.md) 已还清表 C581（extent 偏移非 0 没有判据） 那一行。

**依据**：

- 无实验：定宽换变长省下来的是空间，省掉的是「一个指针单独捡起来能不能直接按偏移解出来」，两边不是同一个量纲，没有可量的量。撑它的是 `.claude/rules/fs-design.md` 两条——「不为省空间牺牲自包含」，以及格式分支的判据（看它能不能把「用错了」变成「挂不上」，且判据里不许出现「这样更快 / 更省」）。
- 用户定案，原话与 2026-09-12 的重申在变更史。
- 判决 2026-10-01 采：extent 偏移第一版非 0 判该指针所在的结构损坏（`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 11 行）。

**欠**：C210（三个「第二套」属于哪一类扩展没人判过）。

```

**出处 `.claude/kb/invariants.md:58-58`（整段抄，未转述）**

```markdown
| I-7.4 | 近 K 代块未被复用 | 回退候选集里每一个根（按实例表判仍然有效 ∧ txg ≥ F_生效，D23（journal 的角色与格式） 已定项 14）所引用的块，其物理范围均未被重新分配给其他对象、也未被清扫抹头。**「K 代」2026-09-13 起就指这个候选集**：平时回退下界 F 不动时是整个根环，盘紧时抬 F 缩到最近 4 个可退到的状态、且每块盘上最新的持久有效根都在内（D16（发布语义） 已定项 1）；原「K 是运行时策略、下限 ≥ 2」作废。[decisions.md](decisions.md) D22（单元原子性怎么合成）：**根环深度是块重用延迟的上界**（D22（单元原子性怎么合成）同时明令块复用的界不能是格式常量），不满足则环里的旧根是假的回退候选。「有效」按 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P3 的回退候选集判：(i, T) 有效 ⟺ 实例表无 i 的行，或有行 (i, Ti, Wi) 且 T ≤ Ti——被抛弃时间线的根不在候选集里，这只管回退目标的选择；被抛弃时间线今天只由崩溃恢复造出（管理员回退是挂着时的一次向前发布，不抛弃任何根，D23（journal 的角色与格式） 已定项 14）。它们引用的块在离开根环之前同样不许重新分配、不许抹头（2026-09-13 用户定案，C314（崩溃恢复落到旧根后可以复用被抛弃的根引用的单元） 取影子账，D23（journal 的角色与格式） 已定项 14；留着的理由是第一轮判决 H6：崩溃恢复落到旧根之后，被抛弃的根引用的单元被复用、之后恢复落在它上面读不出）。⚠️ F_生效 按 D16（发布语义） 已定项 1「生效」取根上与系统配置里读得出的最大值；checker 今天只读根上带的 F，带 F 的根全坏时会在合法状态上误红这一条与别的按回退候选集判的几条（第三轮判决 Y7，推的，没实现），欠 C556（checker 与层 0 不读系统配置里的 F）。崩溃后 defer 队列的内存态丢失，接手的是清扫准入「释放代 ≤ max(F_生效, 环里最旧有效根)」那条闸（谓词由 D16（发布语义） 已定项 1 给，2026-09-13；有效性按实例表判），保护宽度随之变 | 已实现（2026-09-17，池级 checker `walk::check_pool_image`，按每条回退候选集里更早的根各判一格：走读它引用的单元时校验和对不上或头用不了，就是它指着的块被复用或抹头；坏镜像是 mkfs 树表单元（只有第 0 代根还引用）改内容重封；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） 被抛弃、还在根环里的根也判（2026-09-27，用户定「checker 现在就补被抛弃根那一半」，代码审阅第 7 条）：最新根的实例表有行时，根环里每条被判抛弃的根（有行 (i, Ti, ·) 且 T > Ti）各用一份自己的走读走一遍，只看读回来对不对——走出 I-2.1（校验和与内容匹配） 违例或走读失败就按这条根判红，判定丢掉、引用不进 I-3.1（已分配统计对得上） 的并集（`walk::judge_blocks_referenced_by_abandoned_roots`，用例在 `crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs`）；根环有读不出的槽且最新根的实例表有行时，没有违例就整条报不适用、不报成立；C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 那一形在这一半上照实红，等 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 修好转绿。 |
```

**出处 `.claude/kb/invariants.md:152-152`（整段抄，未转述）**

```markdown
| I-3.10 | 已分配记录的分配代等于它罩住的单元的诞生代号 | 任一**未带已释放标志**的分配记录，它的分配代（D3（空间分配） 已定项 3：`value = 分配代`；盘上编码见 D3（空间分配） 已定项 7）等于它罩住的那个单元头里的**诞生代号**（D18（块里携带什么信息） 已定项 7 的类身份段）——记录与单元是同一次发布写出来的，两个数因此同源。⚠️ **射程**：① 只管未释放的记录，已释放的那一半归 I-3.9（释放代落在停止引用它的那一格区间里）；② 记录罩住多个槽时（数据单元跨两槽）取起点槽那个单元头；③ **那个槽上读不出可用的单元头时这一条没有对象**（报不适用，不报违例）——读不出本身由 I-1.1（块头自述逻辑地址） 与 I-2.1（校验和与内容匹配） 管；④ **只读回退候选集里那几版的分配记录**（候选根的分配记录树、树表 0 条那一版根记录直接持有的那一片、由记录施加出来的那几版），**再加下一次挂载会先施加的那一版**：与最新根同实例、checkpoint_txg = 最新根的 txg + 1、带提交标记、根环里没有它的根的那条记录，新根段里树表指着的分配记录树（主 agent 2026-09-24 按代码三方 `research/prompts/m2-wave3-code-r1-main-verification.md` 第四节第 2 条定，被攻过零轮；只接这一步——恢复连着施加几条记录时第二条及以后那几版读不到，欠在欠账表）；不读被实例表判抛弃的根与低于 F 的根：低于 F 的根那棵账里未释放的记录罩住的槽，在 F 抬过之后可以被合法回收、换成诞生代号不同的单元，照「任一」去判会在合法镜像上红（主 agent 2026-09-23 定，被攻过零轮）。⚠️ 「再加下一次挂载会先施加的那一版」那一段的同源前提，靠 D16（发布语义） 已定项 7 持久顺序里单元与记录之间那道屏障撑着：拿掉那道屏障、又复用回收槽时，记录已持久而复用槽上仍是旧单元，恢复按点名验证拒施加、结局正确，这一条却在那个槽上红（2026-09-28 主 agent 复跑 floorreuse4：删屏障 271 个状态红 48、带屏障 73 个状态 0，`research/prompts/k3-9-barrier-r1-main-verification.md` 第一节）；用户 2026-09-28 定屏障维持现状、checker 不改，欠在 C592（单独拿掉单元写与记录间那道屏障今天没判据会红）。**它拦的是什么**：复用一个已回收的落点时改写了那条分配记录、却把上一次的代留着（2026-09-22 增补 3 第 1 件的代码三方第一轮攻方腿造出这条变异，快档与大档都判不出，因为全仓没有一条不变量管已分配记录的分配代） | 已实现（2026-09-23，待代码三方与崩溃验证；池级 checker `walk::judge_allocation_generations_against_unit_births`，读法照射程 ④，同一条记录（盘、槽、代）只判一次，射程 ③ 按记录逐条跳过、一条都没判到才整条报不适用；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） 下一次挂载会先施加的那一版由 `walk::tree_table_pointer_of_the_version_the_next_mount_applies_first` 并进读集；它的坏镜像（发布 B 的记录已落、根槽没落，B 那棵分配记录树里一条未释放记录的分配代改成诞生代号 + 1）是 `checker_known_bad_images.rs` 的 `an_allocation_generation_past_its_unit_birth_in_the_version_only_its_journal_record_carries_reddens_the_allocation_generation_invariant_before_the_mount` |
```

**出处 `.claude/kb/invariants.md:315-315`（整段抄，未转述）**

```markdown
| I-9.28 | 全部可读根里同一树 ID 的诞生 txg 相同 | 根环里全部读得出且自证合法的根（不按实例表判有效与否，含被崩溃恢复抛弃的旧线上的根），任两条根各自的树表里同一树 ID（条目偏移 0）的条目，诞生 txg 必须相同；只有这棵树第一次出现在树表里时才设这个值（D8（核心索引结构） 已定项 8（树表条目的字段））。判定四种：有一处诞生 txg 对不上 → 不成立；各处都对得上，但有一条读得出、自证过的根树表读不出（含树表指针全零）、或条目比字段表短 → 部分没判，列出这些根（盘号、实例、txg、为什么没判）；没有一棵树出现在两个不同的树表单元里（几条根共用一个树表单元时拿同一份字节比自己，是恒真判定）→ 不适用；其余 → 成立。树表指针全零的根不是「没有树表」：core 的根记录读者不把全零当特例，照它那两条位置条目读盘 0 槽 0、读不出（可写挂载拒成 `Recovery(UnitUnreadable { slot: SlotNumber(0) })`），池级 checker 对根环里自证过的这样的根照 core 跟过去、判这条根损坏（红 I-2.1（校验和与内容匹配），最新根再红 I-4.8（近 K 代根校验和自洽）、I-7.2（最新根可完整遍历）、I-7.4（近 K 代块未被复用），别的候选根再红 I-4.8（近 K 代根校验和自洽）、I-7.4（近 K 代块未被复用）），这一条把它与树表读不出同一处置、报部分没判并点名「树表指针全零」（用户 2026-10-05 弹窗定，选项原文「甲 checker 判它损坏」，`records/2026-10-01-里程碑三形式化接入.md` 第 33 批「第 32–33 批代码轮第一轮判决」那一条，判决 `research/prompts/m3-formal-code-b33-r1-main-verification.md` 第五节第 1 条）。只判根环里自证过的根：由记录施加出来那一版里的树表指针全零，主走读仍当这一版没有树表、不记走读失败（用户定的只是自证过的根，要不要同判没定）；不判 I-2.5（位置条目按设备身份升序）：那一行把整条全零的指针整条豁免，豁免要不要收窄没定。GPU 判器的走读对根记录里全零的树表指针没有跟过去，这种镜像上 CPU 红这几条而 GPU 不红；GPU 对拍的流里没有造这种根的。与 I-9.14（树表条目的诞生 txg 跨根不变） 的分工：I-9.14（树表条目的诞生 txg 跨根不变） 只比同一条时间线上的根，这一条不分时间线、比全部可读根。依据：树 ID 被重发（C342（树 ID 水位在根读不出时退回去重发））时，新线上的树与老线上同一个 ID 的树诞生 txg 不同，I-9.14（树表条目的诞生 txg 跨根不变） 的同线收窄看不见这一形。推翻条件：任一合法历史（池级 checker 与层 0 其余判定都不红）上红。 | 已实现（2026-10-04，欠账来源 C342（树 ID 水位在根读不出时退回去重发）；池级 checker `walk::check_pool_image` 的 `judge_tree_identifier_birth_txg_across_every_readable_root`，四种结局；GPU 判器的扫描内核挨着 I-9.14（树表条目的诞生 txg 跨根不变） 判、只置违例位；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`，合法历史、部分没判、不适用、对不上照报违例的用例在 `crates/singlefs-harness/tests/pool_checker_tree_identifier_birth_txg_agrees_across_every_readable_root.rs`，树表指针全零的根报部分没判并点名的用例在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `an_all_zero_tree_table_pointer_in_an_older_candidate_root_is_followed_like_the_core_and_every_tree_table_reading_names_it_unjudged`；`crates/mutations.tsv` 里以「第9批」「第9批改钉」打头、点名 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同）或第四种判定的各行，与以「第 9 批 9-9 GPU 判器」打头、点名 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同）的 3 行）。用户 2026-10-04 定新立，同时删 I-9.14（树表条目的诞生 txg 跨根不变） 判别力第 ③ 条与它的正向用例，判决 `research/prompts/m3-sweep-and-tree-id-r1-main-verification.md` 第六节 S2-1 行。被攻过零轮：攻方三段合法历史上 0 红（一次抽样），被切掉旧线的镜像上红。部分没判：用户 2026-10-04 定「树表读不出算部分没判」、加第四种判定；四种结局的细则主 agent 定，零轮，`records/2026-09-30-里程碑三开工第一轮.md` 第 9 批「用户 2026-10-04 弹窗定（9-6 交回的两问）」那一条 |
```

**出处 `.claude/kb/checks-owed.md:1-16`（整段抄，未转述）**

```markdown
# 欠的检查清单

**checks-owed.md 放「已经知道要拦什么、但现在还拦不了」的检查。**

与 [invariants.md](invariants.md) 的分界：不变量是**对着一个镜像**可判定的性质，
checker 是它的可执行形式；checks-owed.md 放的是**代码路径的要求**——
它们对着镜像判不出来，只能由门禁在跑的时候拦。

**写进来的门槛**：说得出「拦什么」「怎么拦会红」「现在缺什么前置」。
三样缺一样就是还没想清楚，先别写
（`.claude/singlefs-ai-sop/rules/sop-first.md`：写不出下一步就别加那条检查）。

<!-- doc-lint:registry name-col=2 -->

| # | 简称 | 要拦什么 | 怎么拦（会红的形态） | 前置 | 出处 |
|---|---|---|---|---|---|
```

**出处 `.claude/kb/checks-owed.md:584-584`（整段抄，未转述）**

```markdown
| C735 | GPU 判器里由记录施加的版本没有对拍流与变异行 | GPU 判器 `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl` 里 I-5.4（分配记录罩住的槽互不相交） 按版本判的那一圈，第 34 批 34-1 拆出 `judge_allocation_records_disjoint_of_a_version`，候选根、`R_DATA` 里由记录施加出来的那几版、下一次挂载先施加的那一版各调一次。`R_DATA` 里由记录施加出来的那几版那一次调用，没有一条对拍流造得出「由记录施加出来的版本里两条记录重叠」，也没写变异行，去掉它 GPU 判器照绿（变异下会存活）。下一次挂载先施加的那一版有一条对拍流（`the_gpu_judge_agrees_with_the_cpu_when_two_allocation_records_cover_one_slot_in_the_version_the_next_mount_applies_first`）与一行变异（`crates/mutations.tsv` 里名字带「第34批 34-1 GPU」那一行），两样都在 checker 档、没跑、没证红 | 造一条录制流：一个版本只由记录施加出来（记录已落、根槽没落，或挂载之后只由记录施加），它的分配记录叶里两条记录罩住同一个槽，CPU 判器红，GPU 判器也要红；对拍加这条流，再加一行变异去掉 `R_DATA` 那一次调用，对拍必须由绿转红 | 这一形的录制流今天造不出（34-1 实现员没找到造法）；GPU 对拍与变异在 checker 档（要一张 NVIDIA 独显），归提交时 | 2026-10-06 `records/2026-10-01-里程碑三形式化接入.md` 第 34 批 34-1 交回那一条（「GPU 里由记录施加那一圈没有对拍流与变异行（记欠账）」）；34-1 实现员报告「变异」一节末句与「没做什么」一节最后一条 |
```

**出处 `.claude/kb/checks-owed.md:587-587`（整段抄，未转述）**

```markdown
| C738 | 同一次发布几条记录的新根段一致没有不变量判 | D23（journal 的角色与格式） 已定项 15 末句要求一次发布切成几条记录时每条记录都带同一份新根段，今天没有一条不变量判它：I-8.10 只比带末条标志的那条与同代根，I-8.9 判序号与标志、不比新根段。写者把第一条记录的新根段写成上一版时，checker 与 core 都不报（b34 攻方 s1：新 checker 0 红、两种挂载照常挂上，`research/prompts/m3-formal-code-b34-r1-opus-model/out/crafted.txt`）；第 35 批 FL 让 checker 与 core 同取末条之后，写旧了的非末条从此谁都看不见 | 一条不变量（或 I-8.10 往非末条扩）：同一块盘上同（实例代号, checkpoint_txg）的每条记录新根段逐字节相同，CPU 与 GPU 判器同判；坏镜像用攻方 s1 那一形。判别力自证：摘掉这条判，s1 那张坏镜像必须由红转绿 | 合法写者三处造记录都从同一个根取新根段（`crates/singlefs-core/src/transaction.rs` 1502、2039、8768 行一带，攻方读码，推的）；还没派 | 2026-10-06 `research/prompts/m3-formal-code-b34-r1-main-verification.md` 第二节 P1-b 与第三节第 2 条 |
```
