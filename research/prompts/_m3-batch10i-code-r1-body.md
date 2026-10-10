# 根环实现第 10i 批：代码轮第一轮正文（2026-10-06）

<!-- doc-lint:not-numbers K1 K2 K3 K4 S1 R1 -->

## 一、题面

第 10 批代码轮第三轮判决 `research/prompts/m3-batch10-code-r3-main-verification.md` 第六节，用户 2026-10-06 定 S1「另立一题改」：两遍里有一遍读坏的自证根，在根环表里另记一类；抬 F 那一遍重读它仍坏时，照「挂载那一刻读坏」钉住、不拒。第 10i 批照这条改进 `crates/`，同批还补强了几条用例（交回与主 agent 的判定在 `records/2026-09-30-里程碑三开工第一轮.md` 第 10i 批那一条）。这一轮攻的是第 10i 批合入之后的代码与它的测试。第 10 批三轮判决第二节攻过的角度不重复。

改动全文在 `research/prompts/_m3-batch10i-code-r1-diff.md`，实现员报告在 `/tmp/claude-1000/m3-r1/batch10i/report.md`，冻结副本在 `/tmp/claude-1000/m3-batch10i-code-r1/tree/`：

| # | 改动 | 主要落点 | 压着的条款或定案 |
|---|---|---|---|
| A | 新类 `RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses`：进程内照读出来的那条根算；抬 F 那一遍读它读坏、重读仍坏时，照 `BadAtMount` 钉住（回收门槛按可能住过的最旧 txg、准入上限按能住的最大 txg 并），不拒 | `crates/singlefs-core/src/allocator.rs`、`crates/singlefs-core/src/mount.rs` | 用户 2026-10-06 定 S1；`.claude/kb/decisions/16-发布语义.md` 已定项 1 第 37 行「根槽这一次读坏」 |
| B | rollback 三条慢档每次覆盖写之后核「只许 I-3.1 红、红在记账多的一边」；R1 用例补盘宽与前置覆盖写两维（新增 12 条）；丢盘整读新起点补全零与乱字节 | `crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs`、`crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs` | 第三轮判决第三节 |
| C | 变异表第 1314 行「A3b 善后一」改用 `crate::recovery::effective_rollback_floor`；`model_comparison.rs` 注释跟上按槽并 | `crates/mutations.tsv`、`crates/singlefs-harness/src/model_comparison.rs` | 第 10h 批报告 |

## 二、实现今天的样子（主 agent 的观测，2026-10-06）

- 冻结副本取自第 10i 批合入之后的主工作区。被判文件的 sha256 在 `research/prompts/m3-batch10i-code-r1-snapshot/crates-sha256.txt`。合入后验证日志在 `/tmp/claude-1000/merge-and-verify/2026-10-06-2019192/`。
- 主 agent 认的实现员读法（零轮，腿判对不对）：
  - S1 那一类在挂着时管理员回退判候选、算 F_生效 时，照旧当「这个进程知道住着根的槽」，读坏就在任何写之前拒。D16 第 37 行两种字面都套得上，用户只定了抬 F 那一半。
  - 「读坏」含读回全零。
  - 那一槽「能住的 txg」并进读出来那条根的 txg（第 0 代根住的槽落点公式算不到）。
- 已知没做、不在这一轮被判范围的（腿报出来不算打中）：
  - `DeletedRowsStillPresent` 的用例与变异，在 checker 档 `crash_injection.rs`，归下一簇；
  - 实例表 kind 那一族，第 10j 批与下一簇在做；
  - 欠账 C717–C724；
  - S2，带覆盖写的层 0。
- 用户已认的代价（腿报出来不算打中，只核「代价是不是只有这么大」）：
  - 钉最旧 txg、全零槽钉门槛时，I-3.1 在合法盘面上红在记账多的一边，到钉解开为止；
  - Q1、Q2 照今天。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　新类撑不撑得住（A）
新类钉住时，有没有一形让回收门槛越过盘上真住着、可回退到的有效根，或让被抛弃的根掉出影子账。准入上限按它能住的最大 txg 并，那一个「能住的 txg」集合算得对不对。并进读出来那条根的 txg 之后，有没有一形让 F 越过它，或卡住上限多于必要。它什么时候解开：这个进程写掉那一槽、进程内重读读出有效根或被抛弃的根，这几条路径都摆得出吗。

### K2　回退与 F_生效 那两处的读法（A 的零轮读法）
回退判候选、算 F_生效 时照旧「读坏就拒」：有没有一形本可以安全回退却被拒，或拒的理由与条文对不上；这一类在这两处与抬 F 那一处处置不同，会不会在同一次挂载里前后不一致（例如抬 F 推过了，回退却因同一槽拒）。

### K3　补强的用例（B）
rollback 三条每一步核方向，罩没罩住「中间某一步红在记账少的一边」；R1 那 16 条与丢盘整读新起点，每一条红在它要罩的那件事上吗，改回第 10h 批之前的样子时红不红。

### K4　数能不能核
第 10i 批报告里的数（新用例的 narrow-384-20-2 起点、`(5, 3)` 对 `(4, 2)`、R1 31 条 14 秒、txg 40 那一次证红），每一个从哪一段代码推出来、钉它的测试与变异行改回旧值时红不红。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1、K3、K4 | 逐条从条款或定案原文推到冻结副本里的行，再推到钉它的测试与变异行 |
| 云端攻方（Opus） | K1、K2，重点 K1 | 在冻结副本的拷贝上造反例，只跑 harness 档；不重复第 10 批三轮判决第二节已攻过的角度；用户认下的代价只攻「是不是只有这么大」 |

## 五、跑前写死的判据

- 两腿都没打中 ⇒ 站住（一次抽样，「没打中」照 `.claude/rules/three-way-inference.md` 再抽一次才记）。
- 打中 ⇒ 主 agent 现查坐实，改法进下一批；腿提的收严记「零轮」，交用户或登记实验。
- 第二节列为已知没做、用户已认的几样，腿报出来不算打中，判决照原去向写。

交付：腿的报告写 `research/prompts/m3-batch10i-code-r1-<腿名>-output.md`，分段落盘（每次写进文件不超过 150 行，第一段排他新建，回复只写短句）；模型与脚本放 `research/prompts/m3-batch10i-code-r1-<腿名>-model/`；草稿放 `/tmp/claude-1000/<腿名>/`。引 kb 条款写 kb 文件自己的行号（去 kb 文件里现查，不从背景材料里数）；引代码写冻结副本里的行号。方案按 `crates/` 今天的实现来谈。
