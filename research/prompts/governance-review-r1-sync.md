<!-- knowledge-sync -->
# governance-review-r1 阶段同步

触发文件：.claude/agent-common.md、.claude/agents/crash-verifier.md、.claude/agents/experiment-designer.md、.claude/agents/experiment-runner.md、.claude/agents/gate-triage.md、.claude/agents/implementation-writer.md、.claude/agents/kb-scribe.md、.claude/agents/mutation-triage.md、.claude/agents/three-way-verifier.md、.claude/gate.d/20-kb-shape.sh、.claude/gate.d/54-layer0-replay.sh、.claude/gate.d/75-decision-experiment-links.sh、.claude/gate.d/90-term-renames.sh、.claude/gate.d/lib-governance-refs.py、.claude/hooks/heavy-test-guard.sh、.claude/hooks/runner-dispatch-guard.sh、.claude/rules/implementation-workflow.md、.claude/rules/mutation-sampling.md、research/scripts/archive-past-rounds.py、research/scripts/ask-local-selftest.sh、research/scripts/ask-local.sh；另改了 .claude/main-agent.md、.claude/skills/gate/SKILL.md、.claude/skills/crash-test/SKILL.md 与门禁 10 号的两份样本（不在触发登记里，一并记）

这一阶段做成的事：对分支 claude/exciting-bohr-4olowk 相对 73ba4a4 的全部改动做第一轮严查（甲组规则与入口 11 条、乙组 agent 定义 11 条、丙组门禁钩子脚本 14 条，报告原样入库，见「原始证据」），每条现查之后改掉；改法都只被这一轮之后的第二轮严查攻过。

## 搜索

- `grep -rnF '输入没变就复用' .claude records research/scripts CLAUDE.md`（改后）→ 0
- `grep -rnF '复用上一次全绿判定' …`（同一范围）→ 0；`grep -rnF '历史类文件保留旧名' …` → 0；`grep -rnF '层 0、QEMU、herd7 与 crates 变异表归' …` → 0；`grep -rnF '分不清：文件在腿交回之后被改过' …` → 0；`grep -rnF '一条都没有时跳过这一步' …` → 0
- `grep -rnF '不为 0 的整轮已判失败' …` → 3（experiment-runner.md:43、mutation-triage.md:28、mutation-sampling.md:82，改后三处都只说 ⚠️ ⏱ 🧱 或内存撞顶与超时，💥 另写不判失败）
- `grep -rn 'gate.sh --staged' .claude/kb records`（非历史类）→ 4 处在 checks-owed 欠账题面里（C452、C528 等说的是 gate.sh 本身的行为，不涉及谁来起它），不改
- `env -u SINGLEFS_STAGED_TREE bash research/scripts/stage-must-run.sh "$PWD" 59-crates-mutation-replay.sh` → 「这一趟不是 gate-staged.sh 起的（没有 SINGLEFS_STAGED_TREE）…不许复用」退 0（要跑）：复用只在 gate-staged.sh 那一趟里有
- 门禁 10 号样本：绿样本退 0、红样本退 1 且 want 全中；真仓「扫 29 份治理文档：门禁号 106 处、路径 314 处、小节 94 处；没判 10 处」退 0
- `heavy-test-guard.sh --selftest` 562 种通过；`runner-dispatch-guard.sh --selftest` 113 种通过；10、20、62、63、73、75、90 号现跑绿

## 命中处置

| 载体 | 原句 | 处置 |
|---|---|---|
| .claude/main-agent.md:59 | gate-triage 跑 gate.sh --staged，55、59 照 stage-inputs.tsv 复用 | 改了：甲 1 / 乙 1：改跑 research/scripts/gate-staged.sh，复用只在那一趟、判据指到 stage-must-run.sh；甲 2：三行命令放进同一次 Bash 调用；甲 3：层 0 全量判绿之后才往下 |
| .claude/main-agent.md:56 | 提交时 `crash-verifier` | 改了：甲 8：照「暂存之后、提交之前跑门禁」那一行 |
| .claude/rules/implementation-workflow.md:52 | 整轮门禁由 gate-triage 带前缀跑 gate.sh --staged | 改了：甲 1 |
| .claude/rules/implementation-workflow.md:55 | 55、59 靠「输入没变就复用上一次全绿判定」不重跑 | 改了：甲 1 |
| .claude/rules/implementation-workflow.md:39 | stage-must-run.sh 自动比对 | 补了：只在 gate-staged.sh 起的那一趟里比 |
| .claude/agents/gate-triage.md:26 | 暂存过的跑 gate.sh --staged；55、59 复用 | 改了：乙 1 |
| .claude/agents/gate-triage.md:32 | 门禁自己的 git 写 | 补了：gate-staged.sh 前移 refs/sop/staged-green |
| .claude/hooks/heavy-test-guard.sh:40 | gate-triage：gate.sh 里 54 / 55 / 57 / 59 靠「输入没变就复用」 | 改了：甲 6 |
| .claude/hooks/heavy-test-guard.sh:520 | gate-triage 只跑 gate.sh 整轮与 87 号（…复用） | 改了：同甲 6 |
| .claude/hooks/heavy-test-guard.sh:129 | crash-verifier 放行集合含层 0 | 改了：判决 governance-defs-r1 交用户的第 1 行、丙组 1：不再放行，自证 4 例改期望 2 |
| .claude/hooks/runner-dispatch-guard.sh:139 | crash-verifier 那一份含层 0 | 改了：同上，另加自证「崩溃验证员跑层 0 不归它」 |
| .claude/skills/gate/SKILL.md:12 | 由 gate-triage 跑 gate.sh --staged | 改了：甲 1 |
| .claude/agents/crash-verifier.md:24 | （55、57、59 在主工作区跑，没核工作区与暂存区） | 补了：乙 2：开跑前 git diff --quiet 核这几道的输入，不同就停下报 |
| .claude/agents/implementation-writer.md:28 | （`crash-verifier` 与整轮门禁） | 改了：乙 6 |
| .claude/agents/implementation-writer.md:46 | 层 0、QEMU、herd7 与 crates 变异表归 `crash-verifier` | 改了：甲 7 / 乙 6 |
| .claude/skills/crash-test/SKILL.md:12 | 子 agent 不跑 | 改了：甲 9 |
| .claude/rules/mutation-sampling.md:82 | 其余几栏另列，不为 0 的整轮已判失败 | 改了：甲 4 / 乙 3：💥 不判失败；两行「计数：」分开说 |
| .claude/agents/mutation-triage.md:28 | 其余几栏另列、不并进三个数，不为 0 的整轮已判失败 | 改了：乙 3 |
| .claude/gate.d/20-kb-shape.sh:46 | path-moves.md「历史类文件保留旧名」 | 改了：甲 5 |
| .claude/gate.d/90-term-renames.sh:22 | 确实该留旧名的（…历史类文件…） | 改了：甲 5：历史类文件里换了就成假话的那一句，逐文件登记 |
| .claude/gate.d/75-decision-experiment-links.sh:226 | 未定条数不认「两」 | 改了：甲 10：与 20 号同认「两」 |
| .claude/agent-common.md:34 | Bash 里只许定义点名的脚本自己写 | 改了：甲 11：排他新建除外 |
| .claude/agent-common.md:63 | 逐个 nice -n 19 bash 跑登记给你的阶段 | 补了：丙 3：重型的那几道照「不做」一节带前缀，gate-triage 的不单跑 |
| .claude/agents/three-way-verifier.md:22 | 三处「分不清」标签各写各的；输入没有腿开工时刻 | 改了：乙 4 / 乙 5：统一标签，补输入 |
| .claude/agents/three-way-verifier.md:28 | 文件不在快照清单里的 | 改了：乙 5：限定为给了快照的轮 |
| .claude/agents/three-way-verifier.md:32 | 「核不动」与「分不清：文件在腿交回之后被改过」单列 | 改了：乙 4 |
| .claude/agents/three-way-verifier.md:42 | 末尾计数 | 补了：核不动、分不清两栏 |
| .claude/agents/experiment-runner.md:35 | 一条都没有时跳过这一步 | 改了：乙 7 |
| .claude/agents/experiment-runner.md:27 | `name` 写连字符 | 改了：乙 10：只对 research/e7-index-bench，入库装置 bin 名就是文件名 |
| .claude/agents/kb-scribe.md:20 | 标题行状态与 decisions.md 状态列改成什么 | 改了：乙 8 |
| .claude/agents/experiment-designer.md:28 | （英文名写在输入一节） | 改了：乙 9：挪进第 2 步 |
| .claude/agents/experiment-designer.md:38 | `## 一、问题` 写什么 | 补了：第一行「英文名：…」 |
| .claude/gate.d/54-layer0-replay.sh:230 | 在项目根、暂存之后：layer0_full_base=… | 改了：甲 2：说明另起一行，三行是纯命令 |
| .claude/gate.d/lib-governance-refs.py:34 | 「第」「月」后隔空白、`origin/master` 类误报；没判的不列 | 改了：丙 5–9：见文件头 |
| .claude/gate.d/lib-governance-refs.py:73 | 第一段不在仓里就不判 | 改了：丙 10 / 11：带扩展名的照判，样本红得出来 |
| .claude/gate.d/fixtures/10-kb-rot.sh/red/.claude/agents/sample.md:7 | （样本） | 补了：cd 之后的悬空路径、research/ 下悬空脚本 |
| .claude/gate.d/fixtures/10-kb-rot.sh/green/.claude/agents/sample.md:8 | （样本） | 补了：cd 解析与不许误报的四种写法 |
| research/scripts/ask-local.sh:11 | （退出码没有总表） | 补了：丙组：文件头退出码表；运行前清零 UNCHECKED |
| research/scripts/ask-local-selftest.sh:47 | （调用方环境里的 UNCHECKED 没测） | 补了：丙组：带 UNCHECKED=1 跑，证「清零」那一行会红 |
| research/scripts/ask-local-selftest.sh:11 | 副本位置 | 补了：丙组：写明副本要同构 |
| research/scripts/archive-past-rounds.py:262 | 下一步指旧门禁号 | 改了：丙组：指共享 link-targets.py |

## 原始证据

| 材料 | 路径 | 行数 |
|---|---|---|
| 甲组审查报告（规则与入口） | research/prompts/governance-review-r1-report-A.md | 48 |
| 乙组审查报告（agent 定义） | research/prompts/governance-review-r1-report-B.md | 64 |
| 丙组审查报告（门禁钩子脚本） | research/prompts/governance-review-r1-report-C.md | 57 |
