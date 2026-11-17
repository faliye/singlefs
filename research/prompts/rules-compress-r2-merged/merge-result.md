> 修稿注（压缩 2c）：这份是第一版合稿的说明，第二版的逐条变动见上一层的 `dispositions.md`。第一版里关于用户定案第 6、13 条（逐个按进程号停、改脚本与钩子）的几句已被用户定案第 20 条撤回；其余数字（字数、指针数）是第一版的，第二版的字数见 `result.md`。

# 规则压缩合稿（压缩 2a）

批次：records/2026-11-16-三方豁免取消与存量补轮.md#压缩 2a。仓里一个字没改；六个写手的草稿目录只读，要用的稿拷进本目录 `merged-before-edits/`（拷来的原样）再在 `merged/` 上改。没跑项目代码，没交命令给主 agent 跑；本目录 `tools/` 下的核对小脚本（python3，只读仓与合稿树）直接跑。

## 结论

- 合稿树 `merged/`：51 份（`CLAUDE.md`、`.claude/main-agent.md`、`.claude/agent-common.md`、`.claude/rules/` 22 份、`.claude/agents/` 14 份、`.claude/skills/` 12 份），相对路径就是仓里的相对路径，可以逐份换上；新建 4 份（`verification-reference.md` 与 dispatch、merge-verify、three-way 三份技能），要 `git rm` 8 份（`references.md`「七」）。
- 十二条定案与主 agent 对其余六条的处置都落了（「②」，逐条的原件与合稿 文件:行 在 `contradictions.md`「用户定」一节，脚本回原件与合稿树核过）。
- 指针：合稿树里 506 处「文件「节」」「共用约束「条目」」「「节」第 N 条 / 步」，换上之后指不到的 0 处（「③」）。
- 两处只能住一处：改了 7 处（「④」）。
- 落点核不到的条目：0 条（「⑤」写了怎么核、核到什么程度）。
- 矛盾总表 `contradictions.md`：写手报告与合稿核出的 85 条，加用户定 12 条与主 agent 处置 6 条。全仓引用处改法清单 `references.md`。
- 什么现象会推翻这份结论：仓里原件的 sha256 与「⑦」开工时的对不上（各份行号作废）；或主 agent 换上后跑 doc-lint、rules-lint、governance-refs、code-tooling 的 rules-manifest / rules-named-by-definitions / agent-definition-shape 格有一格因合稿红（我只照它们的判法写了近似检查，见「③」「⑤」）；或三方从任一份原件找出一条判据在合稿树与它点名的脚本文件头里都找不到。

## ① 字数（`len()` 数字符，与 `wc -m` 同口径，量的；`tools/` 外的那段 python 现算，原样在 `wordcounts.txt`）

常驻 = `CLAUDE.md` 与它 `@` 的项目文件（`.claude/main-agent.md` 与 `@` 的项目规则；上游七份不在这一件里，不计）；按需 = 共用约束、14 份定义、不 `@` 的规则与参考件、技能。

| | 份数 | 原件 | 合稿 | 合稿 / 原件（推的，由两个量的数算） |
|---|---|---|---|---|
| 常驻 | 原件 21、合稿 16 | 123271 | 69511 | 56.4% |
| 按需 | 原件 34、合稿 35 | 180250 | 187601 | 104.1% |
| 合计 | 原件 55、合稿 51 | 303521 | 257112 | 84.7% |

按需那一栏涨了，是常驻稿挪出去的流程与明细（合稿里参考件 20104 字、三份技能 21927 字，量的）进了按需读的文件。逐份：

| 文件 | 合稿里 | 原件字数 | 合稿字数 |
|---|---|---|---|
| `.claude/agent-common.md` | 按需 | 40232 | 27373 |
| `.claude/agents/clerk.md` | 按需 | 1557 | 1126 |
| `.claude/agents/experiment-designer.md` | 按需 | 12228 | 7533 |
| `.claude/agents/experiment-runner.md` | 按需 | 12941 | 11757 |
| `.claude/agents/implementation-writer.md` | 按需 | 16192 | 13992 |
| `.claude/agents/investigator.md` | 按需 | 2984 | 2464 |
| `.claude/agents/kb-scribe.md` | 按需 | 6370 | 5524 |
| `.claude/agents/mutation-triage.md` | 按需 | 3685 | 3224 |
| `.claude/agents/prior-art.md` | 按需 | 1017 | 960 |
| `.claude/agents/sweep.md` | 按需 | 4936 | 4772 |
| `.claude/agents/three-way-attack.md` | 按需 | 5157 | 4220 |
| `.claude/agents/three-way-defense.md` | 按需 | 2139 | 1809 |
| `.claude/agents/three-way-forward.md` | 按需 | 2406 | 2078 |
| `.claude/agents/three-way-materials.md` | 按需 | 2725 | 2599 |
| `.claude/agents/tooling-writer.md` | 按需 | 5214 | 5050 |
| `.claude/main-agent.md` | 常驻 | 32098 | 13058 |
| `.claude/rules/artifact-line-pointer.md` | 原常驻 | 463 | —（并掉） |
| `.claude/rules/changelog-format.md` | 常驻 | 1959 | 1704 |
| `.claude/rules/code-discipline-by-review.md` | 按需 | 7770 | 4495 |
| `.claude/rules/command-safety-by-review.md` | 常驻 | 3315 | 3446 |
| `.claude/rules/communication-discipline-by-review.md` | 原常驻 | 558 | —（并掉） |
| `.claude/rules/design-doc-discipline-by-review.md` | 原按需 | 282 | —（并掉） |
| `.claude/rules/directory-layout.md` | 按需 | 1210 | 1132 |
| `.claude/rules/engineering-philosophy.md` | 原按需 | 1572 | —（并掉） |
| `.claude/rules/evidence-discipline.md` | 常驻 | 5176 | 6148 |
| `.claude/rules/exhaustive-interleaving.md` | 原常驻 | 628 | —（并掉） |
| `.claude/rules/format-evolution.md` | 常驻 | 4607 | 4087 |
| `.claude/rules/fs-design.md` | 常驻 | 4915 | 4054 |
| `.claude/rules/implementation-first.md` | 常驻 | 1061 | 945 |
| `.claude/rules/implementation-workflow.md` | 常驻 | 11687 | 5698 |
| `.claude/rules/long-run-split.md` | 常驻 | 1851 | 1670 |
| `.claude/rules/machine-first.md` | 按需 | 1510 | 4083 |
| `.claude/rules/mutation-sampling.md` | 按需 | 4566 | 3937 |
| `.claude/rules/path-moves.md` | 按需 | 7009 | 5085 |
| `.claude/rules/pushback-discipline-by-review.md` | 原按需 | 1121 | —（并掉） |
| `.claude/rules/service-stop-readback.md` | 原常驻 | 550 | —（并掉） |
| `.claude/rules/session-wrapup.md` | 常驻 | 2326 | 2019 |
| `.claude/rules/show-me-test-by-review.md` | 常驻 | 2404 | 2032 |
| `.claude/rules/test-discipline.md` | 按需 | 3094 | 2782 |
| `.claude/rules/test-script-discipline.md` | 按需 | 6113 | 5892 |
| `.claude/rules/three-way-inference.md` | 常驻 | 10203 | 4688 |
| `.claude/rules/timer-only-remind.md` | 常驻 | 1521 | 1564 |
| `.claude/rules/verification-reference.md` | 按需 | —（新建） | 20104 |
| `.claude/rules/verification.md` | 常驻 | 24322 | 7273 |
| `.claude/rules/verify-before-claiming.md` | 原常驻 | 1525 | —（并掉） |
| `.claude/rules/writing-discipline-by-review.md` | 常驻 | 1635 | 2041 |
| `.claude/skills/crash-test/SKILL.md` | 按需 | 3113 | 2838 |
| `.claude/skills/crash-test/commit-stages.md` | 按需 | 11431 | 10051 |
| `.claude/skills/decide/SKILL.md` | 按需 | 562 | 549 |
| `.claude/skills/dispatch/SKILL.md` | 按需 | —（新建） | 8201 |
| `.claude/skills/gate/SKILL.md` | 按需 | 893 | 748 |
| `.claude/skills/gate/staged-gate.md` | 按需 | 3659 | 3591 |
| `.claude/skills/halt/SKILL.md` | 按需 | 186 | 151 |
| `.claude/skills/merge-verify/SKILL.md` | 按需 | —（新建） | 6909 |
| `.claude/skills/queue-health/SKILL.md` | 按需 | 1733 | 1726 |
| `.claude/skills/remind/SKILL.md` | 按需 | 1820 | 1275 |
| `.claude/skills/three-way/SKILL.md` | 按需 | —（新建） | 6817 |
| `.claude/skills/trilingual/SKILL.md` | 按需 | 2823 | 2754 |
| `CLAUDE.md` | 常驻 | 10467 | 9084 |

## ② 十二条定案逐条落在哪（合稿树 文件:行；原话与原件位置在 `contradictions.md`「用户定」一节）

1. 实现员一律交补丁：`.claude/main-agent.md:47`（写段第 2 条，只派一个也在副本里改）；`.claude/agents/implementation-writer.md:15`（开头一段改成一律在副本里改）、`:50`（static-check 删「在主工作区改的填仓根」）、`:65`（写范围补「`crates/`、`litmus/` 只在副本里改」，模式不变，与写范围表照旧双向一致）、`:70`（产出删「派发提示写明交补丁时」）；`.claude/agent-common.md:109`（「命名检查交主 agent 跑」）；`.claude/skills/dispatch/SKILL.md:14`（第 4 条）。
2. 三方开轮写成一处：`.claude/rules/three-way-inference.md:43`（「多轮」首段「**开不开下一轮**」）；`.claude/main-agent.md:90`（「要推论」一行改成指过去，另把「一轮派两条推论腿」那一段改指「各条腿必须互不重复」）；`.claude/rules/implementation-workflow.md:10`（「再攻一轮」改成指过去，见矛盾总表「合稿-1」）。
3. 并行度缺省：`.claude/rules/command-safety-by-review.md:39`（没给时一律取 `.claude/heavy-thread-default`，不按 `nproc` 取）。别处的稿本来就写登记值（合稿树 `grep -rn nproc` 只命中这一行自己的「不按 `nproc` 取」，量的）。
4. 证红两样都要：`.claude/rules/show-me-test-by-review.md:7`；`.claude/rules/implementation-workflow.md:9`（第 1 步「谁在判」一格）。
5. 执行员吃紧可交回：`.claude/agent-common.md:68`（「上下文多大都不是交回的理由」后补例外，指到执行员第 2b 步）；`.claude/agents/experiment-runner.md:39`（第 2b 步照原文留着）。
6. 一律不对进程组发信号：`.claude/rules/command-safety-by-review.md:4`、`:6`、`:8-10`（节名改「装置起的样本进程，逐个按进程号停」，判据「每条样本跑完，它起的进程一个不剩」与跑完在 `/proc` 里核一遍留着）；`.claude/agent-common.md:67`（条目起名「**样本进程逐个按进程号停**」）；`.claude/agents/tooling-writer.md:33`（第 5 步）；三份腿定义指这一条的写法跟着改名：`.claude/agents/three-way-forward.md:30`、`.claude/agents/three-way-defense.md:31`、`.claude/agents/three-way-attack.md:39`。
7. 钩子认法下沉、只留合规写法：共用约束稿已这样写，`.claude/agent-common.md:73` 起「执行前拒绝的写法」照收。
8. 示例：`.claude/rules/changelog-format.md:10-19` 放回原件的变更史示例（原件 `.claude/rules/changelog-format.md:17-26`，逐字）；`.claude/rules/fs-design.md` 两段 Rust 示意稿里已删，照删。
9. RefFS 只许 `std`：`.claude/rules/verification.md:11`（定义表照写「依赖只有 `std`」）；`.claude/rules/verification-reference.md:153`（「门禁管哪一半」那一行改成只有 `std`，括注「门禁那一格还许 `singlefs-format`，收严记在欠账」）。
10. `quote-rust-items.py` 进可直接跑的名单：规则与定义不用改（`.claude/agents/three-way-materials.md:29` 照叫材料员直接跑，`.claude/agent-common.md:58` 写名单以 `TEXT_TOOLS_RUN_DIRECTLY` 为准）；钩子库那一行没加，列在 `references.md`「四」。
11. 开头说明句全删：`.claude/rules/verification.md`（删原第 3 行）、`.claude/rules/implementation-workflow.md:3`（留「三方论证照 `.claude/rules/three-way-inference.md`。」）、`.claude/rules/verification-reference.md:3`（删「这是 singlefs 的项目本地规则的参考件，不常驻、不进 `CLAUDE.md` 的 `@`」）；其余 19 份规则写手已删（合稿树 `grep -rn '项目本地规则' .claude/rules` 0 命中，量的）。`CLAUDE.md`「一句话版本」那句实测第一遍稿已删。
12. 七类每类一行：`.claude/agent-common.md:83`（⑨）、`:85`（⑪）、`:89`（⑯）、`:90`（⑰）、`:91`（⑱）、`:97`（写闸第六道）、`:98`（写闸第十道）。

今天改定的三方口径：`crates/` 下全部代码含测试代码进代码轮——`.claude/rules/implementation-workflow.md:10`、`:13`，`.claude/main-agent.md:60`、`:105`，`.claude/agent-common.md:52`，`.claude/agents/experiment-runner.md:36`（入库装置改成进代码轮三方，定义稿已改）；治理文件同规矩——`.claude/rules/implementation-workflow.md:21`、`.claude/main-agent.md:27`；豁免表不再加行——权威落点 `.claude/rules/implementation-workflow.md:10`，`.claude/main-agent.md:32`、`.claude/agent-common.md:52` 改成指过去。

主 agent 对其余拿不准条款的六条处置：`session-wrapup.md`「改 singlefs-ai-sop」照原话（`.claude/rules/session-wrapup.md:22`）；「层 0」节名照原话（`.claude/agents/implementation-writer.md:51`）；`task-server/control check` 不加条款（`.claude/agent-common.md:69` 照原话）；`code-discipline-by-review.md` 两行「留下」原样（`:38`、`:64`）；测试三份与 `path-moves.md`、`directory-layout.md` 不并（五份都在合稿树里）；后两样记欠账（`.claude/rules/fs-design.md:112`，欠账号待主 agent 登）。另照调度记录「现查的三件」把判定库服务的两个目录都写明：`.claude/main-agent.md:27`、`.claude/rules/implementation-workflow.md:21`。

## ③ 指针核对

核法：`tools/pointers.py`（只读）扫合稿树每份 `.md` 的三种指针——`` `文件` ``「节」（含「节」一节「子节」、文件头「段」）、共用约束「条目」、同一份文件里的「节」第 N 条 / 那一行 / 那一节；目标取合稿树里那一份，树里没有的取仓里原件（上游规则、脚本文件头），被并掉的 8 份单报；节名去空白、反引号、星号与「」后按标题、加粗条目、表格首列、全文四档认，「第 N 条 / 步」另在那一节下面找编号。`.claude/agents/*.md` 的「第 N 步」另用一段 python 核（7 处，都在）。

| | 处数（量的） |
|---|---|
| 指针总数 | 506 |
| 命中标题、加粗条目或表格首列 | 464（含「第 N 条 / 步」核到编号的 26 处；另 1 处编号在表格行里、脚本没认出，手核在） |
| 只写文件名、脚本没解析出目标的 | 5（同一行前面写过全路径，手核都在） |
| 只在全文里找得到（脚本文件头里的段名、报告节名这类） | 37：20 处指脚本文件头的段名（逐个 `grep -n` 现查过都在，例 `scripts/gate/lib_reuse_checks.py:8`「一、」、`scripts/agents/agent-watch.py:90`「--ack 的写法」），17 处是同一份定义里点名的报告节名（「要主 agent 跑的命令」「走不到的理由」这类） |
| 指不到 | 0 |

合稿前（六份稿直接拼起来，`pointers-before.txt`）指不到的 10 处，都改了：指向被并掉文件的 9 处（`.claude/agent-common.md:111`、`.claude/rules/test-discipline.md:17` 与 7 份定义，改指 `evidence-discipline.md`、`machine-first.md` 的同名节）、`.claude/rules/implementation-workflow.md:23` 指上游 `show-me-test.md`「新增的测试必须先证明它会红」（上游没有这一节，改指 `.claude/rules/show-me-test-by-review.md`，`:9` 同改）；另有三份腿定义指共用约束里一条没有名字的条目（只在全文里认得出），那一条按用户定案第 6 条改写并起名。合稿树里按旧文件名写、没被上面三种形态认出的几处（`.claude/agent-common.md:20`、`.claude/rules/fs-design.md:87`、`writing-discipline-by-review.md:61`、`machine-first.md:118`、`:128`、`.claude/agents/prior-art.md:16`、`CLAUDE.md` 的 `@` 与 read-once），`grep` 出来一并改了；合稿树里此刻没有一处写被并掉的 8 个文件名（`grep -rn` 0 命中，量的）。

脚本认不出、靠手核的两处：`.claude/skills/crash-test/commit-stages.md:3` 指 `implementation-workflow.md`「三步，缺一步就不算做完」第 3 步，那一步是表格的一行（「3 checker 测试」），在；`.claude/rules/verification-reference.md` 里只写文件名的 5 处（`lib_reuse_checks.py`「二」、`prune_server_client.py`「键空间」、`formal-verification.sh`「复用」、`lib_function_units.py` 两处）同一行前面写过全路径，节名都现查过在。

近似 lint（`tools/` 外的 python，只读）：照 `.claude/singlefs-ai-sop/scripts/doc-lint.sh` D-1 的位置指代词表与 D-2 的自称写法扫合稿树，0 处；写手稿里的四处自称（`fs-design.md:4`「这份规则」、`code-discipline-by-review.md:4`「这份文件」、两份技能开头「这份技能」）改成文件名。`⇒` 只剩 `changelog-format.md:16` 那一处，是用户定案第 8 条要放回的原件示例，在代码围栏里。

## ④ 两处只能住一处（逐处）

| 判据 | 原来各留全文的两处（合稿前） | 权威落点（合稿树） | 改成指针的那一处（合稿树） |
|---|---|---|---|
| 「日常开发的测试不跑整个包」的两小时、四小时那一半 | 原件 `.claude/main-agent.md:19` 与 `.claude/agent-common.md:69` | `.claude/agent-common.md:65`（共用约束稿留全文） | `.claude/main-agent.md:20`（入口稿已改成指针，合稿照收；主 agent 特有的「三遍确认另写进调度记录」「子 agent 报告没写三遍的不交」留着） |
| 提交时六道门禁怎么交、各判什么 | `.claude/main-agent.md:96`（派发表「暂存之后、提交之前跑门禁」）与 `.claude/skills/crash-test/commit-stages.md:20`、`:23` | `commit-stages.md`（调度记录口径 2 定的落点） | `.claude/main-agent.md:96`：删掉「都经 `task-server/task submit --full-gate -- <命令>` 交（带 `--full-gate` 的提交弹窗要用户确认，对话不自己决定）」，改成「`--full-gate` 的写法与弹窗确认、次序与各道判什么照那一份」；「对话不自己决定」那半句挪进 `commit-stages.md:20`。六道的名字与带 `cfg`、`gpu` 那一句照留（门禁 code-tooling 的 agent-definition-shape 格 ⑥ 要派发表点名 `cfg`、`gpu`，`commit-stages.md` 里没有这一句） |
| 「崩溃点测试准确率和正确率优先，其次再衡量时间成本」 | `.claude/main-agent.md:13` 与 `.claude/agent-common.md:54` | `.claude/agent-common.md:54` | `.claude/main-agent.md:13`（加粗名留着，判据句改成「照共用约束「不做」一节同名那一条」；主 agent 特有的「管枚举域，不管跑的时机」「门禁红了必须修好…红着不提交」留着） |
| 几个会话共写一份文件时只核这一批自己的改动 | `.claude/main-agent.md:32`（第 9 条）与 `.claude/agent-common.md:52` | `.claude/agent-common.md:52`（条目起名「**本机常有别的会话同仓干活**」） | `.claude/main-agent.md:32` |
| 按内容哈希登记的豁免表不再加行、补完一轮三方删一行 | `.claude/main-agent.md:32`、`.claude/agent-common.md:52`、`.claude/rules/implementation-workflow.md:10` 三处 | `.claude/rules/implementation-workflow.md:10`（代码轮的落点） | `.claude/main-agent.md:32`、`.claude/agent-common.md:52`（共用约束留「`crates/` 下全部代码（含测试代码）与治理文件的改动一律走三方」那一句口径） |
| 旧数据只是参考、要推翻照三方重走 | `.claude/agent-common.md:24` 与 `.claude/rules/evidence-discipline.md:61` | `.claude/rules/evidence-discipline.md`「原样保存的证据不许事后改」「旧证据是参考，不是依据」那一条 | `.claude/agent-common.md:24`（git log 怎么查照留） |
| `mutate.sh` 结果行怎么数、「计数：」九栏 | `.claude/agents/mutation-triage.md:32` 与 `.claude/rules/mutation-sampling.md:58` | `mutation-sampling.md:58`（分诊员整份读它；它另有「`⚠️`、`⚓` 不为 0 整轮已判失败」半句，分诊员那一份原本没有） | `.claude/agents/mutation-triage.md:32` |
| 三方开第几轮 | `.claude/main-agent.md:90`、`.claude/rules/implementation-workflow.md:10`、`.claude/rules/three-way-inference.md:43` | `three-way-inference.md:43`（用户定案第 2 条） | `main-agent.md:90`、`implementation-workflow.md:10` |

两份稿都改成了指针、全文哪也不在的：没找到（逐句比的 `tools/duplicates.py` 与「⑤」的片段覆盖核都没报出这种情形）。

看过、没改的重复（`tools/duplicates.py` 报的跨文件 24 字以上逐字相同的分句 41 组，原样在 `duplicates-final.txt`，多数是同一个指针串）：
- 三份腿定义之间的复跑包装脚本、`cite-check.py`、冻结副本与 `rerun.sh` 模板，设计员与执行员之间的判段入口认法：各腿、两员只读自己的定义与共用约束，并进共用约束是定义稿「⑥」的建议，没定，照两份留。
- `commit-stages.md:9` 与 `staged-gate.md:8`「这一次是提交时跑还是用户要求时跑」：两份流程各自「跑之前定下来」清单里的一条，管的是两趟，照留。
- `.claude/skills/crash-test/SKILL.md:20-21` 与 `.claude/rules/verification-reference.md:157-158`：前者是崩溃测试几种手段的表，后者是判据归哪道门禁，照留。
- `.claude/main-agent.md:14` 与 `.claude/agent-common.md:51`「禁止自行扩大任务范围」：两句措辞不同，主 agent 那一条多「一件活可以关多条问题…出口是全部达成」，各管各的读者，照留。

## ⑤ 落点核不到的条目

清单：空（0 条）。

怎么核的（左栏是仓里全部原件 55 份：`CLAUDE.md`、`main-agent.md`、`agent-common.md`、`.claude/rules/` 29 份、`.claude/agents/` 14 份、`.claude/skills/` 9 份）：
1. 六份写手报告的逐条对照表（入口第二遍 424 行、规则第二遍 238 行、常驻 337 行、共用约束 269 行、定义 202 行、按需 498 行；量的，`grep -c` 或报告自报）都报 0 行「丢」；合稿照收它们的稿，只在「②」「③」「④」列的那几处动了字。
2. 片段覆盖核 `tools/coverage.py`：从 55 份原件取每个反引号片段、加粗片段、带单位的数，共 4899 个，去空白后在合稿树全文里找，找不到 427 个（`coverage-missing.txt`）：加粗 226 个（去掉了强调，节名与条目名由写手「③」逐个核过还在）；反引号 201 个，其中 87 个是原件 `.claude/agent-common.md:82-109` 下沉到钩子文件头的认法细节（用户定案第 7 条），24 个是被并掉的 8 个文件名，其余 90 个逐个对过写手对照表的去向：例子删掉（`code-discipline-by-review.md` 的代码样例、`verification.md:17` 的测试名样例）、改成指脚本文件头（`verification.md:39`、`:126`，`implementation-workflow.md:88`，矛盾总表「验证-1」「验证-2」「验证-3」「验证-7」）、定义里的命令模板改成指共用约束「子 agent 不交任务」「建仓副本」、`three-way-inference.md` 材料工具的自证与开关已在各脚本文件头（规则第二遍对照表 :241–:250）。
3. 跨稿的「已在 <原件:行>」：对照表里写「已在」某份原件某一行、而那一行有片段在合稿树里找不到的，3 行（`already-in-check.txt`），逐行看过都有落点：`agent-common.md:69` 的 `--check <格>` 写法在合稿 `.claude/agent-common.md:65` 换了说法；`three-way-inference.md:21` 那条 `records/` 链接是指历史、照删；`verification.md:39` 的三个环境变量在 `.claude/rules/verification-reference.md`「门禁 crash-point-replay 怎么跑」指到门禁文件头「显卡与清场」。
4. 合稿这一件自己删掉的字（「④」七处）逐处核过全文在权威落点；唯一挪过去的半句「对话不自己决定」补进了 `commit-stages.md:20`。

这一节的边界：片段覆盖核只认得反引号、加粗与带数的片段，纯文字的判据靠写手的逐条对照表；推翻它的现象是三方从原件找出一条判据，在合稿树与它点名的脚本文件头里都找不到。

## ⑥ 拿不准的

1. 用户定案第 6 条的射程：合稿把规则、共用约束与工具员定义改成逐个按进程号停，另留一句「样本可以起在自己的会话里、按会话号在 `/proc` 里认出脱离父子链的子孙」（`.claude/rules/command-safety-by-review.md:10`）。不留这一句，跑完在 `/proc` 里核「一个不剩」就认不出孤儿进程；留了它像是还在用进程组。仓里按组停的代码 110 行、27 份，钩子 ⑥ 还放行对自己新会话子进程的组发信号（`references.md`「四」）；换上之后规则与代码、钩子不一致，直到治理文件那一批改完。要不要先改代码再换这一节，交主 agent。
2. 用户定案第 1 条：实现员定义的「写范围」照旧列 `crates/**`、`litmus/**`（模式不变，门禁 code-tooling 的 agent-write-scope 格要定义与写范围表两向相等），只加一句「只在草稿目录的副本里改，主工作区的不写」；写范围表 `.claude/hooks/agent-write-scope.tsv` 还许实现员写主工作区的 `crates/`。要不要收严那张表，交主 agent。
3. `.claude/rules/verification-reference.md` 登进 read-once（合稿 `CLAUDE.md:66`、`:67`）：rules-manifest 格只认 `@` 与 read-once 两种登记，参考件不 `@` 就只能登这里；doc-lint 的说法是「开会话读一次」，参考件实际是到那一步才读。另起一种登记要改门禁。
4. 「两处只能住一处」里改成指针的四处（「④」崩溃点那一条、共写文件那一条、豁免表、旧数据）：主 agent 不 `@` 共用约束，指过去之后主 agent 要多读一份；我照派发提示「留权威落点、另一处改指针」做了，权威落点取的是两类读者都会读到的那一份。觉得主 agent 那一侧该留全文的，照「④」表反着改回。
5. 写手报告里没被十二条覆盖、合稿照稿里现状留着的拿不准：入口 ⑤2（`CLAUDE.md` 指现行账本不带劝阻句）；入口第二遍 ⑤6–⑤10（第 11 条细则、坏卡与服务健康、派发表括注放进 dispatch 技能；`main-agent.md` 没压进 1.2 万，合稿是 13058 字，量的；指针句两种写法）；规则第二遍 ⑤2–⑤5、⑤9–⑤12；常驻 ⑤6–⑤8；共用约束 ⑤2、⑤3、⑤5、⑤6、⑤9；定义 ⑤6 与「⑥」九条并进共用约束的建议；按需 ⑤1–⑤4、⑤6。
6. `.claude/rules/changelog-format.md:10-19` 放回的示例里有 `⇒` 与日期 `2026-08-28`，逐字照原件、在代码围栏里；原件就是这样，lint 的结局与原件相同（推的，没跑 lint）。
7. 「合稿-2」判定库服务两个目录：照调度记录「现查的三件」两处都写明了（`main-agent.md:27`、`implementation-workflow.md:21`）；`CLAUDE.md` 合稿里没有写这个目录的地方，没加。
8. 主 agent 处置第 6 条与用户定案第 9 条都要登欠账，合稿里写的是「记在欠账（表）」没写号；登了之后要回来补号。

## ⑦ 仓里各原件开工与收尾时的 sha256

开工清单 `orig-sha256-start.txt`、收尾清单 `orig-sha256-end.txt`（各 55 行，`sha256sum` 原样，量的），`diff` 两份：相同（收尾时现跑，输出 `SAME`）。每份原件的哈希也都在六份写手报告或它们的 `orig-sha256.txt` 里找得到（开工时用一段 python 逐个比过，0 份对不上），所以写手对照表的行号对得上仓里此刻的原件。几份原件此刻在工作区里有没暂存的改动（`git status` 两列都是 M 的有 `CLAUDE.md`、`.claude/main-agent.md`、`.claude/agent-common.md`、`.claude/rules/implementation-workflow.md`），写手稿取的就是带这些改动的那一版。

```
87510a525cc97c082a47c119c95c5f7c76e2a59faa24cbc7a6a8bc42ddb048a7  CLAUDE.md
c310eb2f472bca9172c9e5bb5d0be21f0da5c3a752191ffb90014e41b5293bc9  .claude/main-agent.md
b0a6dc619e4e16311bb1e6a0bf521eac55837b3b1d46886c342fc0e395e9a03a  .claude/agent-common.md
3268acb51c2a65afbc950d0ce9a777f9edcbadf0409ca35a515aefda15f50f4f  .claude/rules/artifact-line-pointer.md
0c2f2a886905167de07de722522d9a07f7b6804460fb06b389c16e5c839a3962  .claude/rules/changelog-format.md
2319fa561849f11a798b3a0cc39231e0624ce1791c948476c2d6007a327444aa  .claude/rules/code-discipline-by-review.md
180596a2112699e5a1d567404ea1ffacde887fb4bb758007ef4dd288252cef14  .claude/rules/command-safety-by-review.md
0a32fbb79fd38406763f953ba0e3c3572c4b5ff8d2a012985070be09791abf87  .claude/rules/communication-discipline-by-review.md
e5895a20421277e71965434c6cdccc487d6c612be66b7bbb1b0c2efc001dfca9  .claude/rules/design-doc-discipline-by-review.md
964f7d9ce9c4053d3e7e95014c171c8ac8420485e0d4e5bd0809bb20a8a59b69  .claude/rules/directory-layout.md
b84e062e109bf01c2f0d3fda151767c013af03fc004eaf098f45442af0090c29  .claude/rules/engineering-philosophy.md
6e523312ce01ab355397101c3ab1d5c9227eaa5b50095818fed003b58ca36fe1  .claude/rules/evidence-discipline.md
587d43d8ecb63444591dc6c6b5d40fd0ecd7585008042e779cb8201db4292819  .claude/rules/exhaustive-interleaving.md
4f987c790d68ae0deb1750ee2cf1f0c72b761ebf7b3ef1d54214afde744b5156  .claude/rules/format-evolution.md
7c60cc868776eb1f4cc70d2a0bd70f7a78b57a8cb24e3748b5505de8159b9e18  .claude/rules/fs-design.md
e16b9d4e06d20e2a32bda0ab17988a81669c74dd12f854bcb9312b5ff66ecce1  .claude/rules/implementation-first.md
27a1ab71a7a2a9cab97e7bbabca0dfa1a3eb17c9d965a0f9c3660e36c893d60d  .claude/rules/implementation-workflow.md
1355f6d71f6eb9c93ae7f1fc18aa72cb6d6f84a2f162db01a2747a84c7bb4c3a  .claude/rules/long-run-split.md
6859a301d59453f052049c23a9fe30688fe69260b76c9835bf1fa8511c6a5e1b  .claude/rules/machine-first.md
3a8d063a24995f9f2ea4be1981414b46b2556ea3be83ab57dc6330223bc60183  .claude/rules/mutation-sampling.md
e3f1f8029e6670f194f296c2f258cf8c1ae9634972cf9ffd990a7da796b627f0  .claude/rules/path-moves.md
33895c5e4973f89799e6691a78413d3f0620a2b7ab58eb8825a717894c488b58  .claude/rules/pushback-discipline-by-review.md
c034e70ec8344571f1efab044bfe596c93d9621065fd867e6e5b98d04f59eb49  .claude/rules/service-stop-readback.md
0dd4be29f6acdf17c01d3848202812cca2002d24117d002afcb6b8cb7bcbc96f  .claude/rules/session-wrapup.md
976106244d419497057b7e6a821774ed9e6653fcd90f575e7f473543f4a98040  .claude/rules/show-me-test-by-review.md
be88281a6af1af34d0a55c0cdb19a8b5d645f8450632945ff743ef9152126151  .claude/rules/test-discipline.md
9bc54146103fd4eaf11c09437c34b31276a170d7d13fafd638a7dcffe4b04a48  .claude/rules/test-script-discipline.md
bcc837d3b2f6918b8c9a323c12ca1ea3f8841aae34d8663d209cdee77e211233  .claude/rules/three-way-inference.md
14534cc40631ad52ae61f7b1b4d2e5956e070125db8e82e488bd4d25dfc4a0da  .claude/rules/timer-only-remind.md
58c9d45cd688b06359105114868cdfacc446b38ce2dafdddb2682b0fe7b1d3bc  .claude/rules/verification.md
49324365450a875cb2d20685a659de66bb1913e151371d41160d5d6c2f0cf4bd  .claude/rules/verify-before-claiming.md
972e78df20c50422991e12f97c2061282c27e90284f71b0452d29a9da2d96f70  .claude/rules/writing-discipline-by-review.md
17c851d78bb84846fc791584a653d235cb7f337d445f8bb0f7dc295c5b328fda  .claude/agents/clerk.md
4cdfa7bed757d442e4d9984d7ce55a28ceca16f79bca1726615408b701d4d3d3  .claude/agents/experiment-designer.md
9cb7a0ab1ee9fca38eb2861582191d1380f5e41016faefd1b307274d8baad983  .claude/agents/experiment-runner.md
3008aa32a01e1011b6b6549996015167c1dc456f86533fe301e322051e3ccfb4  .claude/agents/implementation-writer.md
e14576be3b52114009983379d3ce799ff8c6b33ade7f493f953bdcf4d91646ed  .claude/agents/investigator.md
fbab9980d2b0380dc8225e2907fef5855a378ad01cf697f8daac7681a9457c80  .claude/agents/kb-scribe.md
b7a5d0aa04b16c7581bbe024276d376fadf01ca53513c2db08eeacd9974bded7  .claude/agents/mutation-triage.md
71044967b9320948f5f18e09ae089f88889fdbe9140afb2cb4471293a6da16a9  .claude/agents/prior-art.md
cd5f368832bf7a9040ad0d029b86f6c003d214feb32aecfaae12908f40c1877f  .claude/agents/sweep.md
677c585efd427b4b5ad3462dfb36a32420bb30547e3882ace13672a87ee384c4  .claude/agents/three-way-attack.md
7562aa5051ec3628cce11f1996d1ed79f406eae00988d1bdc2c4dfe8d3e7efce  .claude/agents/three-way-defense.md
48fbaba41885da2fbf352aa3efa0128e7a237454fa3c820b1522e3138055263d  .claude/agents/three-way-forward.md
f992bbfa47739c2cfca9513da7c9b292ca956a787daeb532291874cfaeb1c1cc  .claude/agents/three-way-materials.md
97692f8da7ad49e8a09b4e0227f2a587332c8549d3f3c2b6e713727ff0437170  .claude/agents/tooling-writer.md
130019a6e49572826863ab3ce1950ad9f6fdc183da6ae3e39fab54f54d793b59  .claude/skills/crash-test/commit-stages.md
58f9983c97b5acdd866807e2a1c45c78cbde54bc7fb27f8880cfd121ddae1306  .claude/skills/crash-test/SKILL.md
b3f0e310887f9f9d30f757b73caa128dbcfb5dd50eb4c4c0307c97061806dcdd  .claude/skills/decide/SKILL.md
90fa9cc67d7c624660e5e33c4d31b8dbf910b1b80c3501228ecc0d6284a5e19f  .claude/skills/gate/SKILL.md
96f8e8e0b878e9b21aa9b3dbb8c2bda586df32b3f6c73d75059312e8a8d1f880  .claude/skills/gate/staged-gate.md
bbbfeb981b1a2344798e818d425a0f4ce6df087d17f723c7a0d043208a0fcfdd  .claude/skills/halt/SKILL.md
83a76367771e61407c8a16aaafbbf959476e0b2ab1fca8e758caec690be038ea  .claude/skills/queue-health/SKILL.md
ccda063da2c547ebac2aa0a72ff8261134dcfbfe1668a0ee9ac6d1eef4c9a8fe  .claude/skills/remind/SKILL.md
c33fccea7872e049e3a1b07aa2eb574cbf2435d6abcf9212f7f82ea541957e6f  .claude/skills/trilingual/SKILL.md
```

## 草稿目录里有什么

- `merged/`：合稿树（51 份）；`merged-before-edits/`：六份稿拷来的原样（合稿改动前），`edits-by-line.txt` 是两棵树逐份的改动行号。
- `contradictions.md`、`references.md`、`result.md`（这份）。
- 自查输出：`selfcheck.txt`（指针、矛盾表回核、片段覆盖、旧文件名与说明句残留、原件哈希前后比、合稿份数，命令与原样输出）。
- 核对输出：`pointers-before.txt`、`pointers-mid.txt`、`pointers-mid2.txt`、`pointers-final.txt`、`duplicates.txt`、`duplicates-final.txt`、`coverage-missing.txt`、`already-in-check.txt`、`moved-pointer-check.txt`、`wordcounts.txt`、`wc-table.md`、`refs-*.txt` 与 `refs-*-rows.md`、`contradictions-table.md`、`user-part.md`、`orig-sha256-start.txt`、`orig-sha256-end.txt`、`merged-files.txt`。
- `tools/`：`pointers.py`、`coverage.py`、`duplicates.py`、`refs.py`、`render_contradictions.py` 与它读的四份 `contradictions_data*.py`（都只读仓与合稿树）。
- 没有仓副本、编译目录与库拷贝。

## 没做什么

- 仓里一个字没改；六个写手的草稿目录只读，没改。
- 没跑任何项目代码、门禁与 lint（doc-lint、rules-lint、governance-refs、code-tooling 各格都没跑），没往统一任务服务交任务，也没写要主 agent 跑的命令（派发提示这样定）。「③」「⑤」与 `references.md`「六」里的现算都是我照那几格的判法写的只读 python，不等于门禁真跑过。
- 没改合稿树之外的任何引用处（`references.md` 只列）；没 `git rm`；没登欠账（「⑥」第 8 条）。
- 没走三方（压缩 2b）。
- 写手报告里没被十二条与主 agent 六条处置覆盖的拿不准条款，照稿里现状留着、没改（「⑥」第 5 条）。
