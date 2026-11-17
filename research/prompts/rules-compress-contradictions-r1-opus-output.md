# rules-compress-contradictions-r1 攻方腿报告

日期：2026-11-16。轮名 rules-compress-contradictions-r1，第一轮，前几轮判决：无。
被判对象：`research/prompts/rules-compress-r1-merged/contradictions.md` 的 85 条与「用户定」一节，合稿树 `research/prompts/rules-compress-r1-merged/`。
没写模型，没有复跑命令与模型文件的 sha256。合稿树里的路径下文写全路径；原件写仓里的路径。

## 各格判定一览

| 判定 | 条数 | 是哪几条 |
|---|---|---|
| 改得对 | 82 | 其余各条 |
| 改错了 | 0 | 没有 |
| 其实改了判据 | 3 | 入口-4、常驻-8、合稿-1 |
| 核不动 | 0 | 没有判成这一格的；合稿-1 该照哪边另列在「核不动」一节交用户 |

条数由生成表的脚本数出（量的：草稿目录 `gen.py` 打出 `rows 85 bad 0 same order as table: True {'改得对': 82, '其实改了判据': 3}`，85 条的次序与总表逐条相同，每条引文在被引那一行里逐字核过）。

「用户定」一节 12 条与主 agent 处置 6 条：合稿落的那一句都是用户选的意思，没有落错的；有 3 条落法上的余项，写在「用户定一节」。

总表之外：没有找到总表没报、又与脚本今天的做法相反的改动；找到 2 处合稿自己造出的并存说法、1 处标签报轻了、4 处总表自己的位置写错，写在后面两节。

## 85 条逐条判定

证据一栏是这一次现读到的那一行与原话；写「—」的是依据落在标题行或靠一条命令数出来的，命令与读数写在备注里。备注里「只核了…」是没有往下深核的范围。

| 编号 | 判定 | 现读到的证据 | 备注 |
|---|---|---|---|
| 入口-1 | 改得对 | `.claude/skills/crash-test/commit-stages.md:9`：「只在这两种时候跑」 | 合稿树 `rules/verification.md` 第 32 行仍是不加限定的「提交之外要跑先弹窗问用户」，两种说法在合稿树里并存；带 `--full-gate` 的提交钩子照旧弹窗 |
| 入口-2 | 改得对 | `.claude/main-agent.md:57`：「结局分五支」 |  |
| 入口-3 | 改得对 | `scripts/gate/merge-and-verify.sh:9`：「fmt、clippy 与 build」 | 合稿把这句挪进技能 merge-verify 第 18 行，三样都在 |
| 入口-4 | 其实改了判据 | `research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:9`：「主 agent 可以自己写的只有两类」 | 指针补得对；「只有」是新加的，见「其实改了判据」一节 |
| 入口-5 | 改得对 | `scripts/agents/watch.conf:23`：「工作根所在文件系统（根分区）的可用量」 |  |
| 入口-6 | 改得对 | `.claude/hooks/runner-dispatch-guard.sh:1309`：「第 4 条」 | 原第 7 条括号里的四处登记位合稿没抄，第 2 条里都在 |
| 入口-7 | 改得对 | — | `ls -F crates` 这一次数出 11 个目录加 `mutations.tsv`（量的） |
| 入口-8 | 改得对 | `research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:21`：「除非要查来历，别读它们」 | 只核了依据那一节的标题在（上游 rules-discipline.md 第 32 行） |
| 入口-9 | 改得对 | `.claude/gate.d/split-proposal-ledgers.tsv:6`：「新欠账往这一份末尾追加」 |  |
| 入口-10 | 改得对 | `research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:92`：「一批实现员交回到齐、合入之前」 | 行名在合稿派发表里恰有这一行 |
| 入口-11 | 改得对 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:58`：「的清单只取验证包不带特性那一趟的测试函数」 | 总表把合稿位置写成第 28 行，这句实际在第 58 行 |
| 入口第二遍-1 | 改得对 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:12`：「回扫、分诊、三方腿各是一类」 | 与原件第 24 行逐字相同 |
| 入口第二遍-2 | 改得对 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:47`：「照各份实现员报告的算法改」 |  |
| 验证-1 | 改得对 | `.claude/gate.d/crash-point-replay.sh:361`：「弄坏开关 feature-pass-outer-takes-card 下照外层先只编、再领一张」 | 代码第 4251 行同：只有开关打开才走外层领卡 |
| 验证-2 | 改得对 | `.claude/gate.d/crash-point-replay.sh:141`：「N 照这一趟要现判的量定」 | 合稿 `verification-reference.md` 第 211 行只写「上界是多机配置点名的卡数」，文件头第 140 行的「上界取它与卡数的大者」那半句没带，靠「照文件头」兜住 |
| 验证-3 | 改得对 | `.claude/gate.d/crates-mutation-replay.sh:139`：「按内存定片数」 | `mutation-shard-run.sh` 第 86 行：固定两片是弄坏开关 shards-fixed-two |
| 验证-4 | 改得对 | `.claude/gate.d/lib/stage-cells.sh:53`：「整道退 75」 | 代码第 578 行 `exit 75` |
| 验证-5 | 改得对 | `task-server/task_server_lib.py:231`：「SINGLEFS_THREAD_CAP 不在白名单里，带了当场拒」 |  |
| 验证-6 | 改得对 | `.claude/gate.d/crash-point-replay.sh:334`：「服务起跑不分卡、不清场」 |  |
| 验证-7 | 改得对 | `scripts/gate/verus-proof-check.py:58`：「verus! 里带值的 const」 | 合稿里引的是脚本行号 `:55-60`，脚本一改就漂 |
| 验证-8 | 改得对 | `task-server/task_server_lib.py:2084`：「提交时拒」 |  |
| 验证-9 | 改得对 | `scripts/gate/admission.py:164`：「exhaustive=false 不判、成功句写缩过域、不算门禁通过」 | 只核了文件头，没读判法函数 |
| 规则第二遍-1 | 改得对 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/three-way/SKILL.md:72`：「被别的会话定点替换过的」 |  |
| 规则第二遍-2 | 改得对 | `.claude/agent-common.md:105`：「不按日期、不按轮名免」 |  |
| 规则第二遍-3 | 改得对 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/three-way/SKILL.md:39`：「对不上而原话找得到照收」 |  |
| 常驻-1 | 改得对 | — | 上游 `command-safety.md` 整份读过，没有「别用 echo 假装命令成功」一节（量的）；并进来的四条与原件逐条对过，没少 |
| 常驻-2 | 改得对 | — | `ls .claude/singlefs-ai-sop/rules` 没有 `evidence-discipline.md`（量的）；并进来的三条加判法与原件对过，没少 |
| 常驻-3 | 改得对 | — | 上游目录没有 `machine-first.md`（同一条 `ls`，量的） |
| 常驻-4 | 改得对 | — | 两处节名各在：上游 code-discipline.md 第 80 行、本地 code-discipline-by-review.md 第 15 行（都是标题行，`grep -n` 量的） |
| 常驻-5 | 改得对 | `.claude/singlefs-ai-sop/rules/code-discipline.md:29`：「一个语义概念全仓一个名字」 |  |
| 常驻-6 | 改得对 | — | `grep -n 下限 .claude/singlefs-ai-sop/rules/sop-first.md` 退 1、零命中（量的） |
| 常驻-7 | 改得对 | `.claude/singlefs-ai-sop/rules/kb-discipline.md:87`：「历史条目写成」 | 两份依据的节标题都在（上游 rules-discipline.md 第 40 行、kb-discipline.md 第 81 行） |
| 常驻-8 | 其实改了判据 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/session-wrapup.md:37`：「不为分清是谁的红另起门禁」 | 见「其实改了判据」一节 |
| 常驻-9 | 改得对 | `.claude/agent-common.md:85`：「探活这类真会结束的循环同样照拒」 |  |
| 常驻-10 | 改得对 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:8`：「信号只按进程号逐个发，不对进程组发」 | 整节已照用户定-6 改写，原来 `setsid` 对 `setsid --wait` 的分歧不再单独存在 |
| 常驻-11 | 改得对 | `.claude/main-agent.md:76`：「看门狗只报、不停子 agent 也不停进程」 | 标签该是「改了判据」：原件第 22 行许「检出的工具」在调度方不响应时终止，合稿删了这一方；删得与 main-agent 一致 |
| 常驻-12 | 改得对 | `.claude/agent-common.md:79`：「等自己起的后台任务一律结束本轮等完成通知」 |  |
| 常驻-13 | 改得对 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:39`：「没给时一律取登记的每台默认重活线程数」 | 照用户定-3 |
| 常驻-14 | 改得对 | `.claude/main-agent.md:38`：「没有会话认领、或对应的会话已经不在，自己做」 |  |
| 常驻-15 | 改得对 | `.claude/agent-common.md:127`：「交回内容只写文件路径、`sha256sum` 与判定一览」 | 两处原话不矛盾（一句管回复长短、一句管回复里写什么），合稿只加指针，没删哪一句 |
| 常驻-16 | 改得对 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/session-wrapup.md:37`：「找管它的那一个」 |  |
| 常驻-17 | 改得对 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:95`：「能就值得，不能就不值得」 |  |
| 常驻-18 | 改得对 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:4`：「只写跨决策生效的纪律」 |  |
| 常驻-19 | 改得对 | `.claude/singlefs-ai-sop/rules/kb-discipline.md:87`：「撞号由门禁阶段」 | 总表依据写 `:127-128`，那份文件只有 96 行，这句在第 87 行 |
| 共用-1 | 改得对 | `.claude/agents/kb-scribe.md:43`：「这几处只许由那个脚本改，你不手改」 |  |
| 共用-2 | 改得对 | `research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:18`：「点名的别的文件只读点名的小节」 | 两条都留着，射程分开 |
| 共用-3 | 改得对 | `.claude/agents/sweep.md:4`：「tools: Read, Bash」 | `grep -c SendMessage .claude/agents/*.md` 14 份都是 0（量的） |
| 共用-4 | 改得对 | `.claude/main-agent.md:32`：「用 SendMessage 发给那个会话」 |  |
| 共用-5 | 改得对 | `.claude/rules/implementation-workflow.md:10`：「射程是 `crates/` 下的全部代码」 | 「治理文件」的射程合稿 CLAUDE 第 21 行与 main-agent 第 27 行各写了一遍 |
| 共用-6 | 改得对 | `.claude/hooks/lib_project_code_runs.py:1092`：「主 agent 与子 agent 都直接跑」 |  |
| 共用-7 | 改得对 | `.claude/hooks/write-guard.sh:31`：「目标在仓里」 |  |
| 共用-8 | 改得对 | `.claude/hooks/lib_project_code_runs.py:1094`：「gate-overlap.py」 | 名单里登的是只许 `--list` 这一个参数 |
| 共用-9 | 改得对 | `.claude/hooks/completion-cleanup-guard.sh:104`：「只认认领目录与草稿目录里单子上列出的那几处」 |  |
| 共用-10 | 改得对 | `.claude/hooks/handback-guard.sh:46`：「这一类副本里改、交补丁的，根还可以写占位」 | 钩子认占位；它文件头第 45 行转述的那句共用约束原件里已经没有，文件头该改（归治理文件那一批） |
| 定义-1 | 改得对 | `.claude/rules/implementation-workflow.md:10`：「由主 agent 写判决时逐份点名」 | 执行员定义那句与它自己指去的这一行相反，照这一行改 |
| 定义-2 | 改得对 | `research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:35`：「建仓副本」 | 合稿定义里几处都用这个名字（`grep` 量的） |
| 定义-3 | 改得对 | — | `grep -rhoE` 在 `crates/singlefs-verification/src` 里数出 `NOT_RUN_NO_GPU_ADAPTER` 13 处（量的）；原件两处是下划线被吃掉的写法 |
| 定义-4 | 改得对 | `.claude/agents/three-way-forward.md:29`：「不从背景材料里数行号、不把自己的读法写成条款原文」 | 写全的那一句与正推腿定义这一行同义 |
| 定义-5 | 改得对 | `.claude/gate.d/stage-owners.tsv:36`：「kb 那一边书记员改」 |  |
| 定义-6 | 改得对 | `research/prompts/rules-compress-r1-merged/dot-claude/agents/mutation-triage.md:21`：「留下的日志路径」 | 只核了两处原话说的是同一样东西 |
| 定义-7 | 改得对 | `.claude/agent-common.md:68`：「门禁只有一个，跑法两种」 |  |
| 定义-8 | 改得对 | `.claude/agents/sweep.md:44`：「逐行盘点一张表」 |  |
| 定义-9 | 改得对 | `.claude/agents/kb-scribe.md:16`：「两节」 | 两节的标题在合稿 `format-evolution.md` 第 6、12 行 |
| 定义-10 | 改得对 | `research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-designer.md:39`：「写进第一节末尾的」 |  |
| 定义-11 | 改得对 | `.claude/agents/investigator.md:33`：「上下文多大都不交回，活没做完就接着做」 | 删掉的原第 6 步是这一句，共用约束有同一条，调查员不在用户定-5 的例外里 |
| 按需-M1 | 改得对 | `.claude/rules/code-discipline-by-review.md:77`：「不按行数」 |  |
| 按需-M2 | 改得对 | `.claude/agent-common.md:56`：「进 commit message」 |  |
| 按需-M3 | 改得对 | `.claude/gate.d/lib-lag-baseline.py:237`：「只算顶层」 | 规则没改 |
| 按需-M4 | 改得对 | — | 上游 `show-me-test.md` 的二级标题 6 个，没有这一节（`grep -n` 量的）；本地 `show-me-test-by-review.md` 第 4 行是这一节的标题 |
| 按需-M5 | 改得对 | — | 上游目录没有 `test-discipline.md`（`ls` 量的）；本地那一份第 80 行是这一节的标题 |
| 按需-M6 | 改得对 | `.claude/rules/code-discipline-by-review.md:37`：「处置」 |  |
| 按需-M7 | 改得对 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:95`：「文档纪律不在放宽之列」 | 总表把合稿位置写成第 13 行，这句实际在第 95 行；上游 writing-discipline.md 的标题里没有现状与历史那一条（量的） |
| 按需-M8 | 改得对 | — | 本地 `show-me-test-by-review.md` 第 86 行的节名是「不按来源分类，只看证据」（标题行，量的） |
| 按需-M9 | 改得对 | — | 上游 `pushback-discipline.md` 第 6 行是「警告写在哪」的标题（量的）；原件那句是指回自己 |
| 按需-M10 | 改得对 | — | 两份文件各在本地与上游（`ls` 量的） |
| 按需-M11 | 改得对 | `.claude/gate.d/crash-point-replay.sh:133`：「一块不现判才算过」 | 总表把合稿位置写成第 12 行，实际改的是第 16 行表格里那一格 |
| 按需-M12 | 改得对 | `.claude/agent-common.md:68`：「带 `--item` 的四条照」 |  |
| 按需-M13 | 改得对 | `.claude/hooks/run-scope-guard.sh:142`：「点名格的与直接起的放」 |  |
| 按需-M14 | 改得对 | `.claude/hooks/selftest-registry.tsv:8`：「没有轻重之分」 |  |
| 按需-M15 | 改得对 | — | 只核了两处原话，意思相同 |
| 按需-M16 | 改得对 | `.claude/hooks/timer-only-remind-guard.sh:17`：「crontab（除 -l 之外）」 |  |
| 按需-M17 | 改得对 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/path-moves.md:58`：「自己查」 | 只核了同一份文件 |
| 合稿-1 | 其实改了判据 | `.claude/rules/implementation-workflow.md:10`：「打中的写回代码，再攻一轮」 | 见「其实改了判据」一节；该照哪边另列进「核不动」交用户 |
| 合稿-2 | 改得对 | — | `ls prune-server` 有 `Cargo.toml`、`src`、`control`，`scripts/prune-server` 也在（量的） |
| 合稿-3 | 改得对 | — | 同按需-M4 |

## 其实改了判据（3 条）

### 入口-4：指针补对了，同一句里新加了「只有」

- 原件说的是许可，不是穷举：`.claude/main-agent.md:7`：「改动可以自己写」。
- 合稿改成穷举：`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:9`：「主 agent 可以自己写的只有两类」。
- 合稿树自己另一处还许第三类：`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:11`：「自己直接做，不派」。
- 原件里同一条：`.claude/agent-common.md:8`：「简单任务不派发，自己直接改」。
- 四句回答：总表把这一条报成「改指针」，补上「定案之后写回 kb」那一行是对的（原件派发表第 124 行确有主 agent 自己写规格那一支）；被改掉的是「可以」到「只有」，两处原话在这一点上不矛盾，合稿多加了一条限制，并且与它自己保留的「简单任务自己直接做」对不上。
- 该是什么：去掉「只有」，写回「主 agent 可以自己写的：探索性的 `crates/` 改动……与定案后自己写得清的小处 kb 改动……」；或者在这一句里把共用约束「派发」一节的简单任务那一条也点上。
- 什么现象会推翻这一条：原件别处有一句明写「除这两类之外主 agent 一律不自己写」。我只核了 `.claude/main-agent.md` 第 7 行与 `.claude/agent-common.md` 第 8 行，没全仓搜。

### 常驻-8：两处原话可以并存，合稿把其中一条做法删了

- 原件的做法：`.claude/rules/session-wrapup.md:54`：「分不清的时候跑」（后面是 `gate.sh --staged`）。
- 被拿来当依据的那一边并不禁止提交之外跑，只要求先问：`.claude/rules/verification.md:33`：「提交之外要跑先弹窗问用户」。
- 合稿落成了禁令：`research/prompts/rules-compress-r1-merged/dot-claude/rules/session-wrapup.md:37`：「不为分清是谁的红另起门禁」。
- 四句回答：两句管的是两件事（一句是分不清谁的红时的一个办法，一句是提交之外跑整道门禁的手续），按手续做就不矛盾；合稿选的不是「更新的那一份规则」的原意，那一份留着弹窗这条路；脚本那一侧，不指定范围的提交没带放宽标识的被拒、带 `--full-gate` 的弹窗（`.claude/hooks/run-scope-guard.sh:45`：「提交时照 commit-stages.md 带 --full-gate（会弹窗要用户确认）」），也就是脚本今天许「用户确认之后跑」，不是一律不许。
- 该是什么：「分不清而确要跑的，照 `.claude/rules/verification.md`「一个门禁：两种跑法」先弹窗问用户，同意了经服务交 `scripts/gate/gate-staged.sh`」；原件写的直接起 `bash .claude/scripts/gate.sh --staged` 那个起法确实过时（项目代码闸只认经 submit 起），起法该改，这条路不该删。
- 什么现象会推翻这一条：用户说过「分清谁的红不值得起一趟整轮门禁」这一类的话。`records/2026-11-16-三方豁免取消与存量补轮.md` 第 108 到 141 行我读过，没有这一句；别的记录没查。

### 合稿-1：代码轮「再攻一轮」被换成了通则「再攻两轮、三轮多数」

- 原件代码轮的一句：`.claude/rules/implementation-workflow.md:10`：「打中的写回代码，再攻一轮」。
- 原件通则的一句：`.claude/rules/three-way-inference.md:120`：「同一个结论再攻两轮，三轮里多数打穿才算打穿」。
- 合稿：`research/prompts/rules-compress-r1-merged/dot-claude/rules/implementation-workflow.md:10`：「打中的写回代码，开不开下一轮照」，指过去的那一段写着 `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:43`：「打中的东西先挂起、不直接写进正文」。
- 四句回答：
  1. 总表报的是「改指针」，实际改的是轮数与判法：代码轮打中之后从「改了再攻一轮」变成「再开两轮、按三轮多数判」。
  2. 两处原话是通则与它在代码轮上的特例，不是同一件事说了两遍：代码轮打中的是能复现的错，原件许当场写回（`.claude/main-agent.md:61`：「判决打中的，合成一批派」），通则管的是结论，打中先挂起。
  3. 合稿这一句自己前后对不上：前半句「打中的写回代码」，指过去的那一段说「先挂起、不直接写进正文」。
  4. 依据栏写「用户定案第 2 条」，那一条的原话只点了两句、没点代码轮这一句：`records/2026-11-16-三方豁免取消与存量补轮.md:119`：「写成一处、另一处指过去」，同一行说的是「第一轮两条腿一致、没打中就停」那两句；把 `implementation-workflow.md` 第 10 行并进来是合稿自己的取舍（总表补充一节也这样写）。
- 该是什么：在用户另说之前保留原句「打中的写回代码，再攻一轮」，另加一句「第一轮一致、没打中就停，照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」」；代码轮打中之后到底再攻一轮还是照三轮多数，交用户定（见「核不动」一节第 1 件）。
- 什么现象会推翻这一条：用户定案时明说过代码轮也照三轮多数；或者近几轮代码轮的判决实际都是照三轮多数做的（我没查历轮判决）。

## 改错了（0 条）

没有一条合稿的句子与脚本今天的做法相反。两处不算改错、但句子不全或标签不对的：

- 验证-2：合稿 `verification-reference.md` 第 211 行把片数上界只写成卡数；门禁文件头另有一支，`.claude/gate.d/crash-point-replay.sh:140`：「上界取它与卡数的大者」（点名的用例登记了 `shards-at-least=` 时）。合稿那一句后面跟着「在它文件头」，读的人照文件头做不会错；要不要把这半句补上由主 agent 定。
- 常驻-11：总表标「只改措辞」。原件许三方在调度的一方不响应时终止，`.claude/rules/command-safety-by-review.md:22`：「报告的一方（人、agent 或检出的工具）可以终止它」；合稿只留「报告的人或 agent」。这是把检出工具的终止权删了，属于改了判据；删的方向与 `.claude/main-agent.md:76`：「看门狗只报、不停子 agent 也不停进程」一致，看门狗脚本里停被看对象的代码我没找到（`grep -n` 搜 `os.kill`、`killpg`、`SIGTERM`、`.terminate()` 在 `scripts/agents/agent-watch.py` 里命中 2 处 `.terminate()`，第 2649、3259 行，各在自证函数 `selftest_root_free`、`selftest_session_watch_in` 里，停的是自证自己起的进程；另有第 462 行一条写给人照跑的停进程命令串；量的，只核了这一份脚本）。判定仍是改得对，标签该改。

## 用户定一节（12 条定案与 6 条主 agent 处置）

只判合稿落的那一句是不是用户选的意思，用户原话取自 `records/2026-11-16-三方豁免取消与存量补轮.md` 第 117 到 141 行（现读）。

| 编号 | 落得对不对 | 现读到的证据 | 余项 |
|---|---|---|---|
| 用户定-1 | 对 | `research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:47`：「只派一个也这样」 | 调查员「查到就修」那一支仍在主工作区改，见下面第 1 件 |
| 用户定-2 | 对 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:43`：「第一轮两条腿结论一致、没打中，就停，不开第二轮」 | 连带把代码轮那一句并过去是合稿自己加的，见合稿-1 |
| 用户定-3 | 对 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:39`：「不按 `nproc` 取」 | 合稿树里 `nproc` 只剩这一处（`grep -rn` 量的） |
| 用户定-4 | 对 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/show-me-test-by-review.md:7`：「两样都要」 | 无 |
| 用户定-5 | 对 | `research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:68`：「例外只有 `experiment-runner`」 | 执行员定义写「上下文吃紧或卡住时」，共用约束的例外只写了吃紧，卡住那一半没写进例外 |
| 用户定-6 | 对 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:8`：「信号只按进程号逐个发，不对进程组发」 | 见下面第 2 件 |
| 用户定-7 | 对 | `research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:73`：「逐条以它的文件头为准」 | 只核了这一句在；下沉之后合规写法留没留全归另一题 |
| 用户定-8 | 对 | — | 代码围栏行数：合稿 `fs-design.md` 0、`changelog-format.md` 2，原件各 4、2（`grep -c` 量的） |
| 用户定-9 | 对 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:153`：「门禁那一格还许 `singlefs-format`，收严记在欠账」 | 欠账号没登（总表补充已写） |
| 用户定-10 | 对 | `research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:58`：「为准」 | 见下面第 3 件 |
| 用户定-11 | 对 | — | 合稿树里搜「项目本地规则」只剩 `CLAUDE-merged.md` 第 45 行的节标题与第 62 行一句，都不是那句声明；搜「2026-10-11」零命中（`grep -rn` 量的） |
| 用户定-12 | 对 | `.claude/hooks/bash-command-detector.sh:215`：「作用在工作根本身、它的上级，或工作根下第一层带通配的路径上」 | 七类各一行都在（合稿共用约束第 83、85、89、90、91、97、98 行）；逐条对过钩子文件头第 106、215、295、307、317 行与写闸文件头第 71、125 行，只对了每类的头一句 |
| 主 agent 处置-1 到 5 | 没改，原话都在 | — | 总表点的合稿位置逐条回查，片段都命中（草稿目录 `show2.py`，0 处找不到；量的） |
| 主 agent 处置-6 | 对 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:112`：「规则里还没有对应的节，记在欠账表」 | 欠账号没写（总表补充已写） |

余项三件：

1. 用户定-1 的射程。用户原话是实现员一律交补丁。合稿里调查员接着修的那一支仍在主工作区改：`research/prompts/rules-compress-r1-merged/dot-claude/agents/investigator.md:23`：「这时在主工作区改」，它照的是实现员定义的那几步。这不算落错（原话没点调查员），但同一种活两种做法，要不要一并改由用户或主 agent 定。
2. 用户定-6 合稿另留了一句：`research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:10`：「按会话号在 `/proc` 里认」。它只许拿会话号认进程、信号仍逐个按进程号发，与「一律不许对进程组发信号」不冲突；这一句是合稿自己加的做法，被攻过零轮。
3. 用户定-10 此刻照合稿做会被拒（推的，没跑）：合稿说名单以钩子库为准，而 `grep -n 'quote-rust-items' .claude/hooks/lib_project_code_runs.py` 这一次退 1、零命中（量的）。钩子库改完之前，材料员定义第 29 行叫它直接跑的那一条与「名单为准」对不上。

## 总表之外发现的做法变化

做法：拿原件与合稿树四样机械对过，再读对出来的句子。

| 对了什么 | 怎么对的 | 读数（量的） | 结论 |
|---|---|---|---|
| 14 份定义的 frontmatter | 逐份 `diff` 两个 `---` 之间的行 | 14 份全同 | model、effort、tools、required-inputs 没被动过 |
| 门槛数 | 草稿目录 `nums.py`：抽「不超过 / 至少 / 满 / 超过 / 退 N」带数的短语，列合稿有而原件没有的 | 9 处，逐处读过 | 都是同一个数换了说法（例「不超过 4000 字」），没有新数 |
| 指针 | 草稿目录 `ptr.py`：合稿树里每个「文件」加「小节」的引用，回合稿树或上游那份文件里找 | 引的小节名找不到的 0 处；指到合稿树里已不在的规则文件的 0 处 | 并掉的 8 份规则没有留下断指针 |
| 带「一律 / 只有 / 不许 / 例外 / 不算」的分句 | 草稿目录 `novel.py`：按 5 字片段算与原件全文的重合率，列低于 0.55 的 22 句，放宽到 0.72 共 70 句（含前 22 句） | 70 句逐句读过 | 除下面三件外都能在原件里找到同义的原话 |

对出来的三件：

1. 入口-4 的「只有两类」（已在「其实改了判据」）。
2. 合稿树里「提交之外不指定范围的那一趟」并存两种说法。一处免了用户要求那一趟的弹窗：`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:26`：「提交之外，用户要求的照跑」；两处仍是不加限定的原话：`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:32`：「提交之外要跑先弹窗问用户」，`research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/SKILL.md:12`：「提交之外要跑先弹窗问用户」。入口-1 把一处矛盾改成了另一处并存。该是什么：三处写成同一句；`verification.md` 是两种跑法的落点，改它，另两处指过去。
3. 常驻-11 删了检出工具的终止权而标成只改措辞（已在「改错了」一节下写明，判定是改得对）。

没有找到的：合稿改了、总表没报、又与脚本今天的做法相反的句子。这一句的范围是上面四样对法能照到的；压缩时整句丢掉的判据不在这四样里（那是另一题）。

## 总表自己写错的位置（不影响判定，改总表时一并改）

- 入口-11：合稿位置写的第 28 行是另一句，被改的那句在 `research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:58`：「的清单只取验证包不带特性那一趟的测试函数」。
- 按需-M7：合稿位置写的第 13 行是表格里另一格，被改的那句在 `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:95`：「文档纪律不在放宽之列」。
- 按需-M11：合稿位置写的第 12 行是「验证包」那一段，被改的是第 16 行表格里「崩溃点重放」那一格。
- 常驻-19：依据写上游 `kb-discipline.md` 的第 127 到 128 行，那份文件只有 96 行（`wc -l` 量的），原话在 `.claude/singlefs-ai-sop/rules/kb-discipline.md:87`：「撞号由门禁阶段」。
- 共用-8、定义-8、定义-9、按需-M1、按需-M9 的依据只写了文件名、没带目录，照 `cite-check.py` 的认法要靠文件名唯一才找得到。

## 核不动

按判据格，85 条里没有一条判成「核不动」：每条依据点名的脚本或规则都读得出该照哪边。下面两件是判了格、但该是什么要用户一句话才定得下来的，原样交用户。

1. 代码轮打中之后再攻几轮（合稿-1）。
   - 两边各有规则撑着：代码轮那一句「打中的写回代码，再攻一轮」（`.claude/rules/implementation-workflow.md` 第 10 行），通则那一句「同一个结论再攻两轮，三轮里多数打穿才算打穿」（`.claude/rules/three-way-inference.md` 第 120 行）。
   - 用户定案第 2 条的原话没提代码轮。
   - 要用户选：代码轮打中的，是改了再攻一轮就收，还是与别的三方一样开满三轮、按多数判；选后一种的，「打中的写回代码」要不要改成先挂起。
   - 要查什么才能不问用户就判：近几轮代码轮判决（`research/prompts/*-code-r*-main-verification.md` 与 git 历史里归档的）打中之后实际开了几轮。我没查。
2. 提交之外为分清「是谁的红」要不要许跑整轮门禁（常驻-8，连同入口-1 留下的并存说法）。
   - 原件许跑（`.claude/rules/session-wrapup.md` 第 54 行），通则要先弹窗（`.claude/rules/verification.md` 第 33 行），合稿写成不许。
   - 要用户选：不许；或弹窗同意后许。用户要求的那一趟还问不问一次「要不要跑」，一并定（带 `--full-gate` 的提交钩子那一次确认两种选法下都在）。

## 要主 agent 跑的命令

没有。这一条腿没写模型、没改代码，不要编译与复跑。

## 没打中的形状

- 攻「依据栏的 文件:行 读出来不是写手说的意思」：85 条的依据逐条现读。读出来与写手说法相反的 0 条；行号写错而原话在别处的 1 条（常驻-19）；依据不够用、要另读别处才判得了的 3 条（入口-4、常驻-8、合稿-1，都落进「其实改了判据」）。
- 攻「只改措辞、改指针里藏做法变化」：标这两种的条目逐条对了原件与合稿那一句的前后文。藏了的 3 处：入口-4（改指针里多了「只有」）、合稿-1（改指针里换了轮数）、常驻-11（只改措辞里删了一方的终止权，删得与脚本一致）。别的没有。
- 攻「标改了判据的选错了边」：验证-1 到 5、验证-9、按需-M11、M13、M14 各读了被照的那份脚本文件头，验证-1、3、4 另读到代码行；合稿都照的是脚本今天的做法。只有常驻-8 照的那一边读出来不是合稿写的意思。
- 攻「总表之外」：四样机械对法，见上一节。

## 这条腿自己的限度

- 只抽了一次样。「改得对」82 条里每一条都是一次读的结论；照 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测——否定结论尤其不算」，「没有改错的」「总表之外没有与脚本相反的改动」这两句否定结论要再抽一次才能拿去支撑别的结论。
- 多数「改了判据」的条目只读到脚本文件头，没读到判法函数（表里标了「只核了…」的，加上验证-2、5、6、8、按需-M13、M14、M16）。文件头与代码对不上的话，这些判定会翻。
- 「总表之外」只照得到合稿新写出的句子、数与指针；原件有、合稿整句没了的判据照不到。重合率的门槛（0.55、0.72）是我定的，没扫过别的值。
- 合稿树内部并存的说法只找到入口-1 那一组，是顺着读到的，没有系统地找。
- 没读禁读清单里的文件，没读写手各自的 `result.md`（总表「出处」指的那几份），依据只取脚本、钩子、规则原件与用户定案的记录。

## 没做什么

- 没判正推腿那几格，没替主 agent 采纳。
- 没往统一任务服务交任务，没跑任何项目代码、自证与门禁；全部读数来自 `grep`、`sed -n`、`diff`、`ls`、`wc` 与草稿目录里自己写的只读脚本。
- 没建仓副本与编译目录，草稿目录 `~/.local/share/singlefs/work/rules-compress-contradictions-r1-opus/` 里只有几份小脚本与它们的输出，没有要删的副本。
- 没改合稿树、总表与仓里任何别的文件；该改的句子都写在上面各节的「该是什么」。
