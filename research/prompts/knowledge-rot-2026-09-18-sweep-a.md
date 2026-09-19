# 知识腐烂回扫报告 —— 领域 A（里程碑「第二个事务」代码与 checker）

阶段：knowledge-rot-a。时区：本机 UTC，人在 JST（UTC+9）。本报告写于 2026-09-18（UTC）。

## 关键词（第 7 步取自派发提示「做成的事」）

命令：`grep -n "<关键词>" <文件列表>`，文件列表见下方「载体」。

关键词清单：`publish_overwrite`、`mount_writable`、`mount_rollback`、`reclaim_released_up_to`、
`instance_table`（instance_table.rs 相关）、`write_accounting`、`history\.rs`、
`second_transaction_supplement_three_random_history`、checker 条数（29/23/26/68/69）、
`I-3.8`/`I-7.4`/`I-4.8`/`I-3.9`/`I-9.14`/`I-5.4`、`invariant-count`、
门禁 10/54/55/57/59 号、层 0、第一个事务/第二个事务、`54 段`/`2104413`/`262165`、
`E142`、`checkpoint_txg`、`增补 1`/`增补 2`/`增补 3`、`mutations.tsv`、
`layout/02-second-txn.md`/`layout/01-first-txn.md`、`m2-wave2-crash-verifier-report`。

## 载体（第 8 步，96 个文件，见 `research/prompts/knowledge-rot-2026-09-18-sweep-a-carriers.txt`）

`CLAUDE.md`、`README.md`、`.claude/kb/INDEX.md`、`.claude/kb/invariants.md`、
`.claude/kb/verification-build.md`、`.claude/kb/checks-owed.md`、`.claude/kb/pitfalls.md`、
`.claude/kb/milestone/01-first-txn.md`、`.claude/kb/milestone/02-second-txn.md`、
`.claude/kb/layout/01-first-txn.md`、`.claude/kb/layout/02-second-txn.md`、
`.claude/skills/crash-test/SKILL.md`、`.claude/skills/gate/SKILL.md`、
`.claude/kb/decisions/*.md`（28 个）、`records/*.md`（约 50 个）。

命令与计数（每个关键词跑一次 `grep -c`，逐个见下方明细）：
```
grep -n "publish_overwrite" $F | wc -l   # 2
grep -n "mount_writable" $F | wc -l      # 3
grep -n "mount_rollback" $F | wc -l      # 6
grep -n "reclaim_released_up_to" $F | wc -l  # 1
grep -n "instance_table" $F | wc -l      # 2
grep -n "write_accounting" $F | wc -l    # 3
grep -n "history\.rs" $F | wc -l         # 2
grep -n "second_transaction_supplement_three_random_history" $F | wc -l  # 1
```
（`$F` = carriers.txt 里的 96 个文件）

## 逐处命中与判定

全部命中都落在下列三个文件之一：`.claude/kb/milestone/02-second-txn.md`（本阶段正在被
另一个会话持续改动，工作区 `M` 状态）、`.claude/kb/checks-owed.md`（C340 一条）、
`records/2026-09-16-subagent拆分提案.md`（历史记录，事件句）。逐条判定：

| 载体:行 | 判定 | 理由 |
|---|---|---|
| `.claude/kb/milestone/02-second-txn.md:90,141,166,195,384,401,415,533,553,556,559`（publish_overwrite/mount_writable/mount_rollback/reclaim_released_up_to/instance_table/write_accounting/history.rs 各处） | **不相干（现状句自洽）** | 这些正是「做成的事」的权威登记位本身，用换值判据核过：把句中数字换成今天的值，句子仍然为真（例如 141 行的 `mount_writable` 现状句、166 行 `mount_rollback` 现状句，均已是 2026-09-16/17 的最新落地实况，与派发提示给的完成清单逐项吻合）。这份文件是本阶段的 WIP 载体，**别的会话在写**，本轮不改 |
| `.claude/kb/checks-owed.md:311`（C340） | **事件句，不改** | 这段说的是 P1/P2 两种取法的实测差异（「代码三方两轮实测…」），是那一次论证发生的事，不是「现在是什么样」；欠账仍然开着（回退行落点/C340 未定案），与派发提示里「增补 2 收口…checkpoint_txg = 3」等已收口项不冲突 |
| `records/2026-09-16-subagent拆分提案.md`（mutations.tsv/增补 1/增补 2 各处） | **事件句，不改** | 这份 records 文件按日期记录当天的建设过程，写的是「那一次改了什么」，`design-doc-discipline.md` 规则本就不回改 records 正文 |

## checker 条数、`invariant-count` 交叉核实

- `CLAUDE.md:97`「checker 判 29 条不变量」与 `.claude/kb/invariants.md:12`「池级 checker … 判 29 条」**一致，已是本阶段最新值**（含 I-3.9、I-9.14、I-5.4）。不改。
- `.claude/kb/invariants.md:16`「现共 69 条在用」是 `invariant-count` 标记登记位，与 checker 的 29 条是**不同的量**（69 = 全部不变量类目，29 = 池级 checker 已实现的子集），两处口径不同、都现查过，未见混用。
- `.claude/kb/milestone/02-second-txn.md:547`「清单 26 条」是 2026-09-17（步 6 落地当时，加 I-7.4 / I-4.8 之后、加 I-3.9 等三条之前）的历史数——事件句，换成今值会变假话（26 → 29 才对当下），**留着不改**。

## `crates/mutations.tsv` 条数

`wc -l crates/mutations.tsv` 现读 130 行（工作区，另一会话仍在改）。载体里出现的
「123 条」（`.claude/kb/milestone/02-second-txn.md:359,392`、`records/2026-09-16-subagent拆分提案.md:749`）
均标注着日期与那一次门禁 59 号复跑的产物（`m2-anchor-fix-crash-verifier-report.md`），是
「那一次复跑得到 123 条」的事件陈述，不是「现在有多少条」的现状句 —— 换值判据：把 123
换成 130，那句「同日整表复跑 59 号，123 条变异各自红…08:50:08 到 08:52:25 UTC」就变成假话
（那次跑确实只有 123 条），**不改**。

## CLAUDE.md「一句话版本」末条核对

`CLAUDE.md:97` 逐句核过：说「层 0 崩溃点重放（门禁 54 号）的负载是两条流：第一个事务，
以及固定脚本到 E」。增补 3 新起的随机历史生成器（`history.rs`、
`second_transaction_supplement_three_random_history.rs`）目前**没有**接入门禁 54 号
——milestone/02:384 明写「还没做：第 1 件的代码三方、门禁 54 与 59 号」——所以这句「两条流」
今天仍然为真，**不改**，也不是「要补」：增补 3 第 1 件本身也还没有交回三方复核，尚未到
「该登记而没登记」的地步。

`CLAUDE.md:97` 的 checker 29 条、CLAUDE.md:6「当前里程碑」指向 `02-second-txn.md`——都现查过，仍然为真。

## 反向核对：每件做成的事有没有被记着

逐件核对（对应派发提示的「做成的事」清单）：

| 做成的事 | 登记位 | 是否已记 |
|---|---|---|
| publish_overwrite / release，门禁 59 号 | milestone/02:90，`.claude/gate.d/59-*` | 已记（5f9e449 已提交） |
| mount_writable / mount_rollback / reclaim_released_up_to | milestone/02:141,166,195 | 已记 |
| 层 0 两条流、54 段、2104413 状态 | milestone/02:559, CLAUDE.md:97 | 已记 |
| checker 23→29 条、invariant-count | invariants.md:12,16 | 已记 |
| 增补 1（write_accounting） | milestone/02:266 | 已记 |
| 增补 2 收口 | milestone/02:553,556 | 已记 |
| 增补 3 立项（进行中） | milestone/02:384 | 已记（如实标「还没做」） |
| crates/mutations.tsv 到 123 条 | milestone/02:359,392 | 已记（事件句） |
| 层 0 全量零违例、55/57 绿 | milestone/02:222,556 | 已记 |
| layout/01,02 | CLAUDE.md:74 提及 `.claude/kb/layout/`；02-second-txn 布局文件存在 | 已记 |

未发现「做成的事有登记载体却漏记」的情况。

## 没做什么

- 只搜了派发提示给的关键词与派发提示点名的载体集合；没有全仓无差别搜索，`.claude/kb/decisions/`
  正文里大量「还没有」「尚未」类措辞（例如 `23-journal的角色与格式.md:595` 的
  「本仓还没有记录核对器」）**没有逐条核**——那是 2026-08-30 前后 E49/E51 论证过程中的历史陈述，
  记录核对器本身早在第一个事务里程碑就已存在（`crates/singlefs-harness/src/crash.rs`），
  但这条不在这一阶段「做成的事」清单里，按定义第 9 步「只搜得到派发提示里写了的事」不展开核。
- 没有跑门禁、没有编译，只做了文本搜索与人工判读。
- `crates/mutations.tsv`、`.claude/kb/milestone/02-second-txn.md` 等文件工作区仍有另一个会话在改（`git status` 的 `M` 行），
  本报告按当前快照判定，交回之后若这些文件再变，结论要重新核。
- `research/prompts/`（冻结证据）与 `briefs/`（按日期不回改）按定义不搜。
- 未发现任何一处需要改的「要改」或「要补」命中；全部命中要么是本阶段权威登记位本身（自洽），
  要么是事件句（不改）。

报告路径：`research/prompts/knowledge-rot-2026-09-18-sweep-a.md`。
