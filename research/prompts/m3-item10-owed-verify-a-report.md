# 核实段甲报告

判定表：`/tmp/claude-1000/m3-owed-verify-a/verdicts.md`（18 行，次序照派发提示）。现查日期 2026-09-28（东京）。

## 各判定计数

命令（在草稿目录跑，按表第二列数）：

```
python3 -c "import re,collections; …按 (?<!\\)| 切第二格计数…" verdicts.md
```

原样输出：

```
已还 5 C29 C262 C311 C318 C338
部分已还 12 C66 C78 C236 C237 C248 C277 C280 C297 C307 C308 C309 C312
过期 1 C131
未还 0 
拿不准 0 
合计 18
```

## 拿不准

没有。以下几处按推的写进了判定，逐条标出，主 agent 要坐实时可以补跑：

| 编号 | 推的那一句 | 怎么坐实 |
|---|---|---|
| C311 | 两份都要时 `journal_record_one_copy` 探针的 valid_records 会从 3 变 2、断言 `:214` 红 | 在副本里把 `scan_journal` 改成只认两盘都解析得开的记录，跑 `-p singlefs-harness --test recovery_reads_the_created_file_after_reopen`（harness 档）；这一轮按派发提示不跑任何 cargo 测试 |
| C312 | 删掉 checker `lib.rs:265` 那一支，现有 checker 用例不会红 | 同上，删那一支后跑 `system_configuration_rollback_floor_and_layout_identity` |
| C29 | 变异 `crates/mutations.tsv:249` 被它点名的用例杀掉 | 变异行的锚点这一次只核了唯一命中（`grep -c` 得 1），没有跑变异 |

## 判定里要主 agent 留意的几处

- C29、C311 判「已还」，但欠账行文本自己有过时处：C29 写的「第 285 行」今天是 `crates/mutations.tsv:249`，行末出处列「模型层已还，实现层未还」与第四列「已还（2026-09-22）」说反话；C311 与已还清表里的 C328（`checks-owed.md:576`）是同一题，C328 的判别力自证是一次手测，变异表里没有这一条。
- C131 判「过期」：D1 已定项 6 撤回、「受保护 N 代」在决策与不变量里零命中；「怎么拦」那道检查从没建过，按今天的正文它扫到 0 项。
- C66 与 C78 的「还欠」是同一处：32 号门禁对写「无分项」或空着「指向」列的字段行不判红（32 号头注第 21 行明写合法）。这是 32 号当初有意的取舍还是欠账没还，要主 agent 判；副本探针的输出在 verdicts.md 那两行。
- C309 前置列「checker 侧那份独立实现……一行代码都没有」按今天的仓已不成立（`crates/singlefs-checker/src/lib.rs:38`），真正还欠的是 I-2.2 算法标识的读与判、以及换函数证红的变异。
- C307 与 C277 的代码或脚本里有与今天条款对不上的注释：`crates/singlefs-core/src/records.rs:377`–`:378`「补不补零没定」（D19 已定项 10 已定），`research/scripts/cite-check.py:15`「行号超出文件」列为不判（`:108` 实际判红）。

## 这一轮跑过的命令（只读仓）

- 门禁 32 号现跑一次（`.` 为根）、在两份草稿副本上各跑一次；门禁 79 号现跑一次；`research/scripts/quote-kb.py --selftest`、`research/scripts/cite-check.py --selftest` 与 `cite-check.py probe-cite.md` 在草稿目录跑。都是轻阶段或研究脚本，没有 cargo、没有重型测试。
- 草稿副本：`/tmp/claude-1000/m3-owed-verify-a/g32probe`、`/tmp/claude-1000/m3-owed-verify-a/g32probe2`（只含 `.claude/kb/decisions/` 与 `.claude/kb/layout/01-first-txn.md` 的拷贝，不是 git 仓、没有编译目录），探针报告 `probe-cite.md`，生成脚本 `gen_verdicts.py`。

## 没做什么

- 没跑任何 cargo 测试、变异、层 0 或别的重型测试（派发提示：本机在跑里程碑二提交前的全量）；「会红」凡是没有现跑的，都引用例与变异行，推出来的在「拿不准」那张表里标了。
- 没改仓里任何文件；没做 git 写操作。
- 初判员的 `m3-item10-owed-part*-verdicts.tsv` 只读了判定列（全是「疑似已还或过期」），依据列与 `-report.md` 没有读，每一行都是这一次自己现查。
- 没核这 18 行以外的欠账行；C68、C196、C271、C272 只在引到时读了它们自己那一行的状态。
- 没有删草稿目录里的副本：它们只是 kb 文本的拷贝（两份合计见交回），主 agent 核判定时可以直接复跑 32 号。
- 没删 `/tmp/claude-1000/m3-owed-verify-a/g32probe`（1.3M）：kb 文本拷贝（第七节插了两行无指向的字段行），C66 / C78 判定的探针现场，主 agent 可复跑 32 号。
- 没删 `/tmp/claude-1000/m3-owed-verify-a/g32probe2`（1.3M）：kb 文本拷贝（D5 索引删了第 6 行），C236 判定的探针现场。
