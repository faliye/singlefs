<!-- knowledge-sync -->
# owed-batch1to3 阶段同步

按 `.claude/kb/checks-owed.md` 欠着那张表的顺序从表头逐条过前三批，还清五笔、新立三笔、另有三笔从「没有检查」变成「检查已立、存量仍欠」，并新立六道门禁。

触发文件：.claude/gate.d/92-layout-checker-sync.sh、.claude/gate.d/93-feature-bits.sh、.claude/gate.d/94-checker-implementation-disjoint.sh、.claude/gate.d/95-fixture-claims.sh、.claude/gate.d/96-experiment-source-discipline.sh、.claude/gate.d/97-invariant-field-anchors.sh、.claude/gate.d/98-kb-registry.sh、.claude/gate.d/layouts.tsv、.claude/gate.d/layouts-checker-lag.tsv、.claude/gate.d/experiment-seed-fold-lag.tsv、.claude/gate.d/experiment-constant-reading-lag.tsv、research/scripts/verify-citations.sh、research/scripts/test-environment-check.py、.claude/gate.d/77-test-environment.sh、research/scripts/change-touches-crates.sh

## 搜索

回扫走的是阶段同步那套工具，事实表 21 行（F1–F21）、候选表 138 行（123 个不同的行），两段逐行判、主 agent 逐行全看：

- `python3 research/scripts/stale-candidates.py --changes --base HEAD --out /tmp/claude-1000/owed-sync/changes.md` → 2 条变更记录（H1 E155 第四次跑第一段、H2 feature-bits.md 历史节）
- `python3 research/scripts/stale-candidates.py --check-facts /tmp/claude-1000/owed-sync/facts.tsv --base HEAD` → 罩全，21 行事实的检索词在基准那一版都命中
- `python3 research/scripts/stale-candidates.py --facts … --out /tmp/claude-1000/owed-sync/candidates.tsv` → 138 行（123 个不同的行）
- `python3 research/scripts/stale-candidates.py --check-report candidates.tsv <两份判定报告>` → 138 行都有逐行判定，退出码 0

另外几条现查（这一批改数用的）：

- `python3 -c` 逐文件扫裸 `seed | 1`（不是只按等号后面那一种形态） → 17 处、14 个文件（立账时记的是「四处」）
- `grep -cE '^(ck|ckn|ckdoc|ckdocn) ' research/scripts/verify-citations.sh` → 100，其中 11 条只在 `--selftest` 里跑，真断言 89 条（`prior-art.md` 原写 72）
- `grep -c "^pub const" crates/singlefs-format/src/lib.rs` → 72，其中 2 个是 `pub const fn`，格式常量 70 个
- `ls .claude/kb/*.md` 与 CLAUDE.md「项目本地事实」表比对 → kb 根目录 14 份，表里只登记了 10 份
- `grep -rnwE "fuse|FUSE|Fuse"`、`grep -rn "POSIX"`、`grep -rnw "fsync"` 于 `crates/*/src/*.rs` → 三样全零命中（C381 第 1 题的前提）

## 命中处置

| 载体 | 原句 | 处置 |
|---|---|---|
| .claude/kb/decisions/12-目标介质.md:133 | **欠**：C8…、C10…、C13…、C14（格式变了 checker 没跟）、C19…、C20… | 改了：删掉已还清的 C14 |
| .claude/kb/decisions/12-目标介质.md:23 | 同一条理由 C12（增量语义共用） 已写过一半：checker 不许链接任何管道的运行时代码 | 改了：写明门禁 94 号还清的是本仓两个 crate 那一格，第三方管道那一侧要真跑 `cargo tree` 才罩得到 |
| .claude/kb/decisions/13-验证路线.md:84 | 遍历与记账代码交集为空（C12（增量语义共用） 已定的符号级判据 | 改了：形态是 crate 粒度的依赖闭包交集，不是符号级 |
| .claude/kb/decisions/13-验证路线.md:97 | **欠**：C94…；C12（增量语义共用）；C17…；C387…；C388… | 改了：删掉已还清的 C12 |
| .claude/kb/decisions/13-验证路线.md:67 | **欠**：C25（三个 oracle 零代码）；C191… | 改了：这一项是「RefFS 做到极致」对应 O3，C25 改指 C447 |
| .claude/kb/decisions/13-验证路线.md:121 | **欠**：C25（三个 oracle 零代码）；「三 oracle 合谋测试」没有 C 编号。 | 改了：三条 oracle 的总览，按 C445 / C446 / C447 三个继任编号列出 |
| .claude/kb/decisions/05-快照-空间记账机制.md:84 | **欠**：C12（增量语义共用）；C344… | 改了：删掉已还清的 C12 |
| .claude/kb/decisions/06-快照实现模型.md:42 | **欠**：C12（增量语义共用）：② 的引用计数旁表与运行时共用增量语义时…… | 改了：**不删**——这一格说的是 core crate 内部两段代码同源，门禁 94 号只判 checker 与 core 的边界，罩不到；改成写明这一格今天没有编号盯着 |
| .claude/kb/decisions/06-快照实现模型.md:147 | **欠**：C12（增量语义共用）：条件 2 要的「与运行时不共享判定代码的独立重算路径」今天没有会红的检查。 | 改了：这一条由门禁 94 号给出，欠改成「无」 |
| .claude/kb/decisions/08-核心索引结构.md:38 | **欠**：C12（增量语义共用）、C15… | 改了：删掉已还清的 C12 |
| .claude/kb/decisions/21-权威态与派生态的分界.md:249 | **欠**：C12（增量语义共用）。 | 改了：94 号给出了「从权威态重算」的独立路径，欠改成「无」，并写明重算结果对不对归 C13 |
| .claude/kb/decisions/17-实现分层与第三方管道.md:78 | **欠**：C25（三个 oracle 零代码）。 | 改了：这一项「入口 = RefFS 的操作面」对应 O3，改指 C447 |
| .claude/kb/decisions/17-实现分层与第三方管道.md:124 | **欠**：C25（三个 oracle 零代码）。 | 改了：这一项「数据接口」正文写明归 O2，改指 C446 |
| .claude/kb/decisions/17-实现分层与第三方管道.md:145 | **欠**：C25（三个 oracle 零代码）；C8（门禁范围判不出来）。 | 改了：三条继任编号都列出，并写明第 1 笔要的「oracle 成为对外发布的一致性套件」比它们都宽、那一格今天没有编号 |
| .claude/kb/experiments/69-反向索引取权威态的增量维护代价.md:7 | 等 C12（增量语义共用） 做跨装置的增量语义检查时当反例素材 | 改了：94 号是单机静态的 crate 依赖检查，不是这里预期的跨装置检查，那一种今天仍未出现 |
| .claude/kb/checks-owed.md:362 | 照 C12（增量语义共用） 的形态做符号级检查…取得到 `crates/singlefs-checker` 的调用图（与 C12 同一件工装） | 改了：C410 不能直接借 94 号的工装——那是 crate 粒度的依赖闭包，调用图仍要另建 |
| .claude/kb/checks-owed.md:359 | O3（独立规约执行器） 零代码（C25（三个 oracle 零代码） 记着） | 改了：C407 那一行改指 C447 |
| .claude/kb/prior-art.md:12 | **72 条**承重引用做成了可重跑的逐字断言 | 改了：89 条，并写明这个数该由脚本末行报出来、38 条核的不是固定点树 |
| research/scripts/test-environment-check.py:66 | 引文核对的 PDF 抽文本缓存…（research/scripts/verify-citations.sh 第 134 行） | 改了：那一段已漂到第 150 行，改成引变量名 `PDFTXT_CACHE`、不引行号 |
| .claude/gate.d/47-research-script-selftests.sh:5 | `change-touches-crates.sh --selftest` 十六份都通过。 | 改了：去掉写死的份数（那句自己列了 17 个脚本、运行时报 19 份），改成「上面列出的每一份」 |
| CLAUDE.md:22 | 跑相关门禁（26、47、62、63、89 与 doc-lint） | 改了：26 与 89 号本地阶段已收归上游，改成「47、62、63 与 doc-lint；本地阶段判别力与编号简称由共享 `gate.sh` 跑」 |
| CLAUDE.md:77 | （新增） | 补了：「项目本地事实」表补四行——feature-bits.md、term-renames.md、tooling.md、INDEX.md，此前一份都没登记 |
| .claude/main-agent.md:43 | 再从全部判定里随机抽 20 行（五种判定各至少 2 行，不够的全抽）自己复判 | 不改：这一批不带这条改动。判定是要改——用户定案，阶段同步的判定主 agent **逐行全看**，不抽样；抽样这一次漏掉了 06:42 那一处判错。改动已经在工作区里，但不进这次提交：门禁 72 号要求改过的 agent 定义在同一次改动带来的三方判决里被点名，这条规则改动单独走一轮三方之后再提交 |
| .claude/kb/checks-owed.md:471 | （新增） | 补了：「## 历史版本」补一条 2026-09-21，记这一批还清五笔、新立三笔、三笔改成「检查已立、存量仍欠」与两个数当天错了两次 |
| .claude/kb/decisions/15-格式冻结政策.md:113 | [checks-owed.md](../checks-owed.md) 的 C4（项目规则清单不一致） 与 C8（门禁范围判不出来） 已实现 | 不改：说的是冻结前置清单第 6 项的**要求**，不是现状声明；同文件 127 行明写 C8 仍欠 |
| .claude/kb/checks-owed.md:446 | 承重引用 55 → 72 条 | 不改：C38 已还清行里记的是 2026-08-29 那次做成的事，带日期落款 |
| records/2026-09-07-冻结预留第一轮.md:32 | `.claude/kb/feature-bits.md` 现查**不存在** | 不改：那天记录的检查结果，历史记录不随后来建了这个文件而改写 |
| records/2026-08-29-复跑复核轮.md:225 | 目前只有 4 个阶段有样本，另外 10 个显式列成「未自检」 | 不改：2026-08-29 那天的状态记录；今天的数（8 个未自检）写在 C46 那一行 |
| .claude/kb/experiments/155-每次持久化的写量三种fsync形态与反事实上界.md:157 | 2026-09-21 不受影响：现查…已定项 7「**欠**」段原文「E155…第四次跑第一段在跑」 | 不改：那次回看的事件记录。⚠️ 它引的原句今天已被改成「已经跑完」，引文与被引对不上——那是别的会话的改动范围 |
| .claude/gate.d/77-test-environment.sh:1 | （整份属于别的会话这一次的改动） | 不改：这一次的改动属于别的会话（与 33、59 号及它们的 fixtures 一起动的），不在这一阶段的射程里，在这里点名只为让触发文件不漏 |
| .claude/kb/checks-owed.md:22 | C8（门禁范围判不出来）：「它按**固定的 diff 基准**（`GATE_BASE`，没设时是上次门禁全绿那一点）判「碰没碰前缀」」 | 改了：这句成了假话——基准这一次改成恒取 `HEAD`。同一行末尾补了这一刀修掉哪一半（上一个提交已验过的改动不再算进这一轮）、还欠哪一半（同一轮跑第二三次仍重跑，要落盘记指纹） |
| research/scripts/change-touches-crates.sh:25 | `base_of()` 取 `GATE_BASE`，够不着才退回 HEAD | 改了：恒取 `HEAD`。`GATE_BASE` 答的是「上次过闸以来有没有人改过」，这个脚本问的是「这份代码变没变」，两者不是一回事；函数注释写明了为什么、以及改回去会踩哪一格自证 |
| research/scripts/change-touches-crates.sh:107 | `--selftest` 自证 5 格 | 补了：加第 6 格「上一个提交改了 crates、这一轮只改文档 ⇒ 判没碰」，它是这次改动唯一分得出的那一格；把基准改回 `GATE_BASE` 实测只红这一格 |

## 2026-09-21（其二）为什么这一轮还留着这份记录

门禁 91 号（上一轮及更早的实验记录归档进版本库）判它该删——它已随上一次提交进仓、相对 HEAD 没再改过；
门禁 68 号（改了规则、agent、hook、门禁、脚本或实现之后有没有写阶段同步记录）判它要留——它点名的触发文件里
还有 14 个落在当前改动范围内。**两道的时间基准差一格**：68 号的范围是 `diff(HEAD~1, 工作区)`（`refs/sop/gate-ok` 不存在时
`diff_base` 退回 `HEAD~1`），91 号看的是「相对 HEAD 改没改过」。于是每次提交之后，上一轮的同步记录必然同时满足
「91 号说该删」与「68 号说要留」。

按 C450（归档删掉的 sync 记录还在役） 的判据「91 号删一份 `*-sync.md` 之前先问它点名的触发文件还在不在当前基准的
改动范围里：在就不删、并说明为什么留」，这一轮留着它，这一段就是那句说明。等 `refs/sop/gate-ok` 前移、或者两道的
基准对齐之后，它点名的触发文件落到范围之外，才轮得到按 91 号归档。
