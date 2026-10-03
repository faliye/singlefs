# 续做推满之后卸载怎么做成：三方第一轮正文（2026-10-03）

<!-- doc-lint:not-numbers Q1 Q2 Q3 Q4 U14 -->

## 一、题面

删除批代码轮第一轮判决 `research/prompts/m3-unlink-code-r1-main-verification.md` 第 64 行记用户 2026-10-03 定案「照 D16 做成可写（推荐）」：挂载续做推满抬 F 仍放不下时，挂载照样交回可写、队列留着，之后会话里每次用户改动先走空、走不动报空间不够，正常卸载照常。第 11 批照这一句实现之后（判决第三节 U14），实现员探针发现这一形上正常卸载做不成。用户 2026-10-03 弹窗定「卸载必须做成」。这一轮要回答：怎么做成、各条路的代价，交岔路给用户。

## 二、实现今天的样子（主 agent 的观测，2026-10-03）

- 冻结副本 `/tmp/claude-1000/m3-unmount-after-short-continuation-r1/tree/`（取自 2026-10-03 主工作区，第 11 批两份补丁与改钉值已合入；`.rs` 的 sha256 在 `research/prompts/m3-unmount-after-short-continuation-r1-snapshot/crates-sha256.txt`）。合入后 harness 档 126 个测试二进制全绿（`/tmp/claude-1000/m3-batch11/merge-verify/`）。
- `crates/singlefs-core/src/mount.rs` 的 `continue_the_deletion_queue_until_it_is_empty`（冻结副本第 4382 行起，文档第 4361–4381 行，「不判式子那一判、落得下那一判照判」在第 4366 行）：走空里的删除类发布「不判式子那一判、落得下那一判照判」；被拒是空间不够时推一串抬 F 再试；推满仍放不下交回 `DeletionQueueContinuationOutcome::StillShortAfterTheFloorRaises`（第 915–925 行），挂载照样做成可写。
- 正常卸载 `unmount`（第 2171 行起）走 `raise_the_floor_through(RaiseFloorEntry::Unmount, …)`，与准入抬 F 共用一串：整串预演，任一次取不到落点就一次都不发、报 `RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite`（第 2131 行文档、`rehearse_the_publishes_raising_the_floor` 第 2786 行）。
- 环窗口 `crates/singlefs-core/src/admission.rs` 的 `ring_window_on_one_device`（第 383 行）= (3S + 1) × W 槽，每块盘常驻扣。
- 实现员探针（副本里的临时测试，不在仓里；输出 `/tmp/claude-1000/m3-batch11/a/scratch/probe1.txt`，报告 `/tmp/claude-1000/m3-batch11/a/report.md` 第五节第 1 问）：「不补足环窗口的窄盘」上，单元区 192/224/256/288/320/384 槽 × 先覆盖写 0/2/4 次 × 文件 10/14/20/30/40 个单元 × 崩在意图之后 0/1/2 批。主 agent 现数：`outcome=Short` 163 行（`RaiseRefusedForSpace` 162、`AtCeiling` 1），这 163 行 `unmount-err[RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite …]` 163 次、`unmount-ok` 0 次，之后再挂 `remount-err` 154、`remount-ok` 9。「不补足环窗口」指什么、探针怎么造的盘，报告里没写全：探针源码已删，只剩输出。
- `crates/singlefs-harness/tests/` 里钉这一形的用例名以 `a_mount_whose_deletion_continuation_stays_short_` 起头（第 11 批实现员 A 加），它今天按「卸载在预演里拒、盘不动」钉。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### Q1　这一形出在哪些盘上
「续做推满仍放不下、之后卸载预演被拒」在哪些几何与历史上出现：只在环窗口装不下的窄盘上，还是在 D28（挂载期承诺量） 已定项 1 式子照常判、mkfs 收下的几何上也出现。先弄清探针「不补足环窗口」是什么造法（造法写进报告），再在冻结副本的拷贝上扫：单元区槽数、文件单元数、崩在意图之后第几批、续做之前的覆盖写次数，至少把探针那组取样点与它们向大盘那一侧各推一档。每格报：续做停因、卸载结局、卸载预演被拒那一次缺的槽数（每块盘）。

### Q2　候选：卸载怎么做成
下面三条只是起点，腿可以加，每条写清它改 `crates/` 哪个文件的哪一步：

- 甲 删除类发布给卸载留量：走空里的删除类发布在落点那一判之外，另判「发完之后每块盘仍放得下卸载那一串最坏要的槽数」，不够按空间不够拒，接今天推抬 F 再试那条路。
- 乙 卸载不抬 F 也算做成：卸载那一串预演被拒时一个字节不写，交回一个新成员「卸载做成、F 没抬」，下次挂载照旧走恢复、续做接着走。
- 丙 腿提的别的形态（例如把卸载那一串的最坏量并进环窗口或保留池的式子，或别的）。

每条要答：卸载在 Q1 找到的每一格上做不做得成；对续做的影响（队列留多长、续做停得多早）；对会话里用户改动的影响（报空间不够提前多少）；乙这一类是不是满足 D16（发布语义） 已定项 1「正常卸载」那一行，不满足的话 F 不抬对下一次挂载的回退候选集与续做意味着什么，会不会一直停在这一形（挂载、走不动、卸载、再挂载循环）。

### Q3　代价数
每条候选在 Q1 那几格上的代价：每块盘多扣或少用多少槽、`df` 少报多少（D16（发布语义） 已定项 1「`df`」那一行的式子）、卸载多几次写。只报自己算过的格，说清是量的还是推的。

### Q4　条款要补的句子
每条候选落地要改哪几条已定项的哪一句（D8（核心索引结构） 已定项 16 第 6 条、D16（发布语义） 已定项 1「准入」「正常卸载」「`df`」几行、D28（挂载期承诺量） 已定项 1），只写要改哪一句、改成什么意思，不写正文。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | Q1、Q2、Q4 | 从条款原文与冻结副本的代码推：每条候选在条款上站不站得住、改哪几处；Q1 先从式子推哪些几何会出这一形，再挑几格在拷贝上量 |
| 云端攻方（Opus） | Q1、Q2、Q3，重点 Q1 与 Q2 | 在冻结副本的拷贝上造反例：每条候选在哪段合法历史上卸载仍做不成、或把别的形压成永远报空间不够；Q1 扫几何，只跑 harness 档 |

## 五、跑前写死的判据

- Q1 两腿都只在环窗口装不下的窄盘上造得出这一形 ⇒ 主 agent 把「mkfs 拒这类几何」当一条候选并进岔路（它也要有代价数）；在 mkfs 收下的几何上造得出 ⇒ 候选只在甲、乙、丙里比。
- 一条候选在 Q1 任一格上卸载仍做不成 ⇒ 那条出局，判决写明是哪一格。
- 两腿都没打中某条候选 ⇒ 站住（一次抽样，「没打中」照 `.claude/rules/three-way-inference.md` 再抽一次才记）。
- 判决不替用户选：站住的候选连同代价数写进 `research/prompts/m3-unmount-after-short-continuation-r1-forks.md` 交用户。
- 腿提的新候选算零轮。

交付：腿的报告写 `research/prompts/m3-unmount-after-short-continuation-r1-<腿名>-output.md`，分段落盘（每次写进文件不超过 150 行，第一段排他新建，回复只写短句）；模型与脚本放 `research/prompts/m3-unmount-after-short-continuation-r1-<腿名>-model/`；草稿放 `/tmp/claude-1000/<腿名>/`。引 kb 条款写 kb 文件自己的行号（去 kb 文件里现查，不从背景材料里数）。方案按 `crates/` 今天的实现来谈。
