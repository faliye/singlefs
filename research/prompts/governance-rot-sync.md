<!-- knowledge-sync -->
# governance-rot 阶段同步

触发文件：.claude/agent-common.md、.claude/agents/experiment-runner.md、.claude/agents/sweep.md、.claude/agents/three-way-attack.md、.claude/agents/three-way-defense.md、.claude/agents/three-way-materials.md、.claude/rules/path-moves.md、.claude/rules/three-way-inference.md、.claude/gate.d/10-kb-rot.sh、.claude/gate.d/lib-governance-refs.py、.claude/gate.d/69-evidence-in-repo.sh、research/scripts/archive-past-rounds.py

这一阶段做成的事：治理文档（CLAUDE.md、main-agent、agent-common、agents、rules、skills）里指向已删门禁、已删脚本、已归档样例、改过名的小节的地方改成现存目标；门禁 10 号加第 4 段，机械判这三类指向（用户定：增强现有门禁，不新立）。

## 搜索

- 改之前在 HEAD 上跑 `python3 .claude/gate.d/lib-governance-refs.py` → 10 处红（原样输出在 research/prompts/governance-rot-head-scan.out）
- 改之后跑同一条 → 0 处红，「扫 29 份治理文档：门禁号 87 处、路径 256 处、小节 76 处」
- `grep -rn "门禁 65 号\|65 号（\|rewrite-moved-paths\|path-moves\.tsv\|23 号（文档指向）\|scripts/proc\.py stop"`，排除 research/prompts、research/results、规范副本与两份变更史 → 改完剩 records 6 行、research/perf-by-milestone.md 2 行、stale-candidates 基准事实表 1 行、kb 3 行（checks-owed.md 两行、vm-harness.md 一行），逐行处置见下表；另有 hook、agent-watch.py 与 73 号样本里写的是正确路径 `python3 .claude/singlefs-ai-sop/scripts/proc.py`，不是腐烂
- `grep -n "23 号" research/scripts/archive-past-rounds.py` → 2 处，都改了

## 命中处置

| 载体 | 原句 | 处置 |
|---|---|---|
| .claude/agents/experiment-runner.md:32 | 门禁 65 号查读子进程输出的循环里一边取时间一边输出。 | 改了：共享 `gate.sh` 的「转发计时」阶段查…（65 号在 8186d5b 随 SOP 0.0.56 收归上游并删掉） |
| .claude/rules/path-moves.md:31 | 23 号（文档指向） | 改了：共享 `gate.sh` 的「链接指向」阶段（23 号同上收归上游） |
| .claude/agents/sweep.md:31 | 照 `.claude/rules/path-moves.md` 登记搬迁、用 `research/scripts/rewrite-moved-paths.py` 全仓改 | 改了：照 `.claude/rules/path-moves.md`「怎么做」逐步改（脚本与登记表 3cff909 已拆，用户定） |
| .claude/agent-common.md:48 | 逐个 `scripts/proc.py stop <pid>` | 改了：`python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`（与 research/scripts/agent-watch.py:143 同一写法） |
| .claude/agents/three-way-attack.md:15 | 「多轮：一次打穿不算数，三轮里多数打穿才算（2026-09-06 用户明令）」 | 改了：去掉括注，与 .claude/rules/three-way-inference.md:129 的标题一致 |
| .claude/agents/three-way-defense.md:15 | 同上 | 改了：同上 |
| .claude/agents/three-way-materials.md:20 | 形态照 `research/prompts/_m2-code-r1-diff.md` | 改了：形态照已归档的 `_m2-code-r1-diff.md`，指到共用约束里的取法（3cff909 归档） |
| .claude/agents/three-way-materials.md:28 | 同上 | 改了：同上 |
| .claude/rules/three-way-inference.md:125 | （`research/prompts/c143-r3-main-checks/`） | 改了：写裸名 `c143-r3-main-checks/`、标已归档并指到取法（3cff909 归档） |
| .claude/rules/three-way-inference.md:112 | `.claude/singlefs-ai-sop/rules/show-me-test.md`「没实现的要明说」 | 改了：「门禁不许假装通过」（原文没有那一句小节名，意思在那一节第 99 行） |
| .claude/gate.d/69-evidence-in-repo.sh:243 | （path-moves.tsv 只收从仓库根写的路径，/tmp 登记不进去） | 改了：删掉这句括注，登记表 3cff909 已拆 |
| research/scripts/archive-past-rounds.py:20 | 门禁 23 号判的就是这个。 | 改了：共享 `gate.sh` 的「链接指向」阶段判的就是这个。 |
| research/scripts/archive-past-rounds.py:262 | 跑门禁 23 号（文档指向）确认没有指空的链接 | 改了：跑共享 gate.sh（「链接指向」阶段）确认没有指空的链接 |
| .claude/gate.d/10-kb-rot.sh:2 | （新增） | 补了：第 4 段「治理文档里的指向」，逻辑在 .claude/gate.d/lib-governance-refs.py，样本各加一份 |
| .claude/kb/vm-harness.md:161 | 谁在拦：门禁 65 号（`research/scripts/relay-timing-lint.py`）… | 不改：kb 写回归 kb-scribe、要带「## 历史版本」条目，这一轮只管治理文档；留给下一轮，登记在收尾报告 |
| .claude/kb/checks-owed.md:375 | C428 路径回写的自检与被检共用同一条跳过规则 | 不改：被测脚本已拆，这笔欠账该不该挪进已还清或作废是欠账状态的判断，交用户，不在这一轮 |
| .claude/kb/checks-owed.md:376 | C429 路径回写不保形，也不核新路径指不指得到 | 不改：同上 |
| research/perf-by-milestone.md:431 | …门禁 65 号拦装置里的这种循环。 | 不改：research 正文，不在治理文档射程里；留给下一轮，登记在收尾报告 |
| research/perf-by-milestone.md:491 | …并指到 vm-harness.md 那一节与门禁 65 号。 | 不改：记的是那一次改了什么，是事件句 |
| research/scripts/fixtures/stale-candidates-benchmark-facts.tsv:56 | 新立：…门禁 65 号 | 不改：阳性对照用的历史事实表，内容是那一阶段的事实 |
| records/2026-09-16-subagent拆分提案.md:970 | …门禁 65 号（`relay-timing-lint.py`）归 `experiment-runner`。 | 不改：records 记建设过程，那一天的事实 |
| records/2026-09-16-subagent拆分提案.md:696 | 自证 `rewrite-moved-paths.py --selftest` … | 不改：同上 |

## 原始证据

| 材料 | 路径 | 行数 |
|---|---|---|
| 改之前在 HEAD 上跑新检查的原样输出（10 处红） | research/prompts/governance-rot-head-scan.out | 11 |
