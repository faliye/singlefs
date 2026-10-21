# 全仓 Markdown 文档分析报告（一）：核心规范、规则、Agent 与基础说明

> 本报告针对 `singlefs` 根目录、`.claude/rules/`、`.claude/agents/`、`.claude/skills/`、`crates/.../invariants/` 以及 `scripts/` 下的全部 **59 篇** 核心活体与说明文档进行逐篇过时与一致性分析。

---

## [CLAUDE.md](file:///home/fy5090/code/singlefs/CLAUDE.md)
- **文件规模**：114 行，14503 字节
- **判定结论**：⚠️ **存在过时或对不上的内容**
- **具体过时/不一致点**：
  - 第 59 行列出 `@.claude/rules/test-script-discipline.md`：需注意上游已在 0.0.67 撤回该规范，本地保留作为专属规则，需明确标注本地属性。
  - 第 112 行记录门禁与负载情况：需同步最新崩溃枚举用例（现已扩充为 24~29 条，文中数字需核验是否对齐）。
- **建议处置**：按最新代码基准、门禁格名（如 `pinned-red-examples-table`）及 crate 数量（8 个）进行定点 `replace-once.py` 更新。

## [README.md](file:///home/fy5090/code/singlefs/README.md)
- **文件规模**：157 行，16699 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：路径引用有效，术语、常量与当前 codebase 严格对齐。

## [README.zh.md](file:///home/fy5090/code/singlefs/README.zh.md)
- **文件规模**：157 行，14232 字节
- **判定结论**：⚠️ **存在过时或对不上的内容**
- **具体过时/不一致点**：
  - 中英日三语 README 的崩溃枚举用例条数、测试总集数量需确认是否已同步 2026-10-18 最新登记。
- **建议处置**：按最新代码基准、门禁格名（如 `pinned-red-examples-table`）及 crate 数量（8 个）进行定点 `replace-once.py` 更新。

## [README.ja.md](file:///home/fy5090/code/singlefs/README.ja.md)
- **文件规模**：157 行，20851 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：路径引用有效，术语、常量与当前 codebase 严格对齐。

## [.claude/agent-common.md](file:///home/fy5090/code/singlefs/.claude/agent-common.md)
- **文件规模**：107 行，41569 字节
- **判定结论**：⚠️ **存在过时或对不上的内容**
- **具体过时/不一致点**：
  - 第 28 行 `KS_THREADS` 需确认是否已在当前环境中全部同步为 `E168_THREADS`。
- **建议处置**：按最新代码基准、门禁格名（如 `pinned-red-examples-table`）及 crate 数量（8 个）进行定点 `replace-once.py` 更新。

## [.claude/main-agent.md](file:///home/fy5090/code/singlefs/.claude/main-agent.md)
- **文件规模**：102 行，40295 字节
- **判定结论**：⚠️ **存在过时或对不上的内容**
- **具体过时/不一致点**：
  - 第 10 行对 D13 已定项 9 的引文措辞需注意与全量弹窗及无变树驱动的最新判决保持完全一致。
- **建议处置**：按最新代码基准、门禁格名（如 `pinned-red-examples-table`）及 crate 数量（8 个）进行定点 `replace-once.py` 更新。

## [.claude/rules/changelog-format.md](file:///home/fy5090/code/singlefs/.claude/rules/changelog-format.md)
- **文件规模**：47 行，3940 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：路径引用有效，术语、常量与当前 codebase 严格对齐。

## [.claude/rules/directory-layout.md](file:///home/fy5090/code/singlefs/.claude/rules/directory-layout.md)
- **文件规模**：29 行，2425 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：路径引用有效，术语、常量与当前 codebase 严格对齐。

## [.claude/rules/format-evolution.md](file:///home/fy5090/code/singlefs/.claude/rules/format-evolution.md)
- **文件规模**：75 行，10252 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：路径引用有效，术语、常量与当前 codebase 严格对齐。

## [.claude/rules/fs-design.md](file:///home/fy5090/code/singlefs/.claude/rules/fs-design.md)
- **文件规模**：190 行，10531 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：路径引用有效，术语、常量与当前 codebase 严格对齐。

## [.claude/rules/implementation-first.md](file:///home/fy5090/code/singlefs/.claude/rules/implementation-first.md)
- **文件规模**：20 行，2170 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：路径引用有效，术语、常量与当前 codebase 严格对齐。

## [.claude/rules/implementation-workflow.md](file:///home/fy5090/code/singlefs/.claude/rules/implementation-workflow.md)
- **文件规模**：69 行，18999 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：路径引用有效，术语、常量与当前 codebase 严格对齐。

## [.claude/rules/mutation-sampling.md](file:///home/fy5090/code/singlefs/.claude/rules/mutation-sampling.md)
- **文件规模**：94 行，9745 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：路径引用有效，术语、常量与当前 codebase 严格对齐。

## [.claude/rules/path-moves.md](file:///home/fy5090/code/singlefs/.claude/rules/path-moves.md)
- **文件规模**：59 行，8220 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：路径引用有效，术语、常量与当前 codebase 严格对齐。

## [.claude/rules/test-script-discipline.md](file:///home/fy5090/code/singlefs/.claude/rules/test-script-discipline.md)
- **文件规模**：107 行，14836 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：路径引用有效，术语、常量与当前 codebase 严格对齐。

## [.claude/rules/three-way-inference.md](file:///home/fy5090/code/singlefs/.claude/rules/three-way-inference.md)
- **文件规模**：167 行，18805 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：路径引用有效，术语、常量与当前 codebase 严格对齐。

## [.claude/rules/timer-only-remind.md](file:///home/fy5090/code/singlefs/.claude/rules/timer-only-remind.md)
- **文件规模**：30 行，2327 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：路径引用有效，术语、常量与当前 codebase 严格对齐。

## [.claude/rules/verification.md](file:///home/fy5090/code/singlefs/.claude/rules/verification.md)
- **文件规模**：139 行，36104 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：路径引用有效，术语、常量与当前 codebase 严格对齐。

## [.claude/agents/clerk.md](file:///home/fy5090/code/singlefs/.claude/agents/clerk.md)
- **文件规模**：43 行，3683 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：路径引用有效，术语、常量与当前 codebase 严格对齐。

## [.claude/agents/experiment-designer.md](file:///home/fy5090/code/singlefs/.claude/agents/experiment-designer.md)
- **文件规模**：69 行，14598 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：路径引用有效，术语、常量与当前 codebase 严格对齐。

## [.claude/agents/experiment-runner.md](file:///home/fy5090/code/singlefs/.claude/agents/experiment-runner.md)
- **文件规模**：59 行，20149 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：路径引用有效，术语、常量与当前 codebase 严格对齐。

## [.claude/agents/implementation-writer.md](file:///home/fy5090/code/singlefs/.claude/agents/implementation-writer.md)
- **文件规模**：64 行，16932 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：路径引用有效，术语、常量与当前 codebase 严格对齐。

## [.claude/agents/investigator.md](file:///home/fy5090/code/singlefs/.claude/agents/investigator.md)
- **文件规模**：49 行，5985 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：路径引用有效，术语、常量与当前 codebase 严格对齐。

## [.claude/agents/kb-scribe.md](file:///home/fy5090/code/singlefs/.claude/agents/kb-scribe.md)
- **文件规模**：53 行，10327 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：路径引用有效，术语、常量与当前 codebase 严格对齐。

## [.claude/agents/mutation-triage.md](file:///home/fy5090/code/singlefs/.claude/agents/mutation-triage.md)
- **文件规模**：48 行，8048 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：路径引用有效，术语、常量与当前 codebase 严格对齐。

## [.claude/agents/prior-art.md](file:///home/fy5090/code/singlefs/.claude/agents/prior-art.md)
- **文件规模**：41 行，2111 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：路径引用有效，术语、常量与当前 codebase 严格对齐。

## [.claude/agents/sweep.md](file:///home/fy5090/code/singlefs/.claude/agents/sweep.md)
- **文件规模**：62 行，10607 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：路径引用有效，术语、常量与当前 codebase 严格对齐。

## [.claude/agents/three-way-attack.md](file:///home/fy5090/code/singlefs/.claude/agents/three-way-attack.md)
- **文件规模**：62 行，9250 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：路径引用有效，术语、常量与当前 codebase 严格对齐。

## [.claude/agents/three-way-defense.md](file:///home/fy5090/code/singlefs/.claude/agents/three-way-defense.md)
- **文件规模**：45 行，4031 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：路径引用有效，术语、常量与当前 codebase 严格对齐。

## [.claude/agents/three-way-forward.md](file:///home/fy5090/code/singlefs/.claude/agents/three-way-forward.md)
- **文件规模**：46 行，4082 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：路径引用有效，术语、常量与当前 codebase 严格对齐。

## [.claude/agents/three-way-materials.md](file:///home/fy5090/code/singlefs/.claude/agents/three-way-materials.md)
- **文件规模**：46 行，5034 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：路径引用有效，术语、常量与当前 codebase 严格对齐。

## [.claude/agents/tooling-writer.md](file:///home/fy5090/code/singlefs/.claude/agents/tooling-writer.md)
- **文件规模**：52 行，7456 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：路径引用有效，术语、常量与当前 codebase 严格对齐。

## [.claude/skills/crash-test/SKILL.md](file:///home/fy5090/code/singlefs/.claude/skills/crash-test/SKILL.md)
- **文件规模**：42 行，5552 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：路径引用有效，术语、常量与当前 codebase 严格对齐。

## [.claude/skills/crash-test/commit-stages.md](file:///home/fy5090/code/singlefs/.claude/skills/crash-test/commit-stages.md)
- **文件规模**：46 行，24544 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：路径引用有效，术语、常量与当前 codebase 严格对齐。

## [.claude/skills/decide/SKILL.md](file:///home/fy5090/code/singlefs/.claude/skills/decide/SKILL.md)
- **文件规模**：13 行，1012 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：路径引用有效，术语、常量与当前 codebase 严格对齐。

## [.claude/skills/gate/SKILL.md](file:///home/fy5090/code/singlefs/.claude/skills/gate/SKILL.md)
- **文件规模**：13 行，1458 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：路径引用有效，术语、常量与当前 codebase 严格对齐。

## [.claude/skills/gate/staged-gate.md](file:///home/fy5090/code/singlefs/.claude/skills/gate/staged-gate.md)
- **文件规模**：31 行，7282 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：路径引用有效，术语、常量与当前 codebase 严格对齐。

## [.claude/skills/remind/SKILL.md](file:///home/fy5090/code/singlefs/.claude/skills/remind/SKILL.md)
- **文件规模**：43 行，3052 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：路径引用有效，术语、常量与当前 codebase 严格对齐。

## [crates/singlefs-checker/invariants/01-块自描述.md](file:///home/fy5090/code/singlefs/crates/singlefs-checker/invariants/01-块自描述.md)
- **文件规模**：30 行，32434 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：路径引用有效，术语、常量与当前 codebase 严格对齐。

## [crates/singlefs-checker/invariants/02-根环.md](file:///home/fy5090/code/singlefs/crates/singlefs-checker/invariants/02-根环.md)
- **文件规模**：36 行，37268 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：路径引用有效，术语、常量与当前 codebase 严格对齐。

## [crates/singlefs-checker/invariants/03-journal.md](file:///home/fy5090/code/singlefs/crates/singlefs-checker/invariants/03-journal.md)
- **文件规模**：43 行，21558 字节
- **判定结论**：⚠️ **存在过时或对不上的内容**
- **具体过时/不一致点**：
  - I-8.4 状态列与实现说明：文中状态列若仍标为未实现则过时（2026-10-18 已核实 checker 一半已在 crash_injection.rs 中实现落地）。
- **建议处置**：按最新代码基准、门禁格名（如 `pinned-red-examples-table`）及 crate 数量（8 个）进行定点 `replace-once.py` 更新。

## [crates/singlefs-checker/invariants/04-校验和.md](file:///home/fy5090/code/singlefs/crates/singlefs-checker/invariants/04-校验和.md)
- **文件规模**：12 行，14874 字节
- **判定结论**：⚠️ **存在过时或对不上的内容**
- **具体过时/不一致点**：
  - 第 9 行：引用的文件路径已不存在：`research/prompts/m3-formal-code-b30-r1-sonnet-output.md`
  - 部分不变量条目引用的历史探针或验证脚本路径已在目录重构后归档。
- **建议处置**：按最新代码基准、门禁格名（如 `pinned-red-examples-table`）及 crate 数量（8 个）进行定点 `replace-once.py` 更新。

## [crates/singlefs-checker/invariants/05-空间记账.md](file:///home/fy5090/code/singlefs/crates/singlefs-checker/invariants/05-空间记账.md)
- **文件规模**：39 行，44315 字节
- **判定结论**：⚠️ **存在过时或对不上的内容**
- **具体过时/不一致点**：
  - 第 21 行：引用的文件路径已不存在：`research/prompts/m3-batch10o-code-r1-forks.md`
  - 部分不变量条目引用的历史探针或验证脚本路径已在目录重构后归档。
- **建议处置**：按最新代码基准、门禁格名（如 `pinned-red-examples-table`）及 crate 数量（8 个）进行定点 `replace-once.py` 更新。

## [crates/singlefs-checker/invariants/06-事务与崩溃一致性.md](file:///home/fy5090/code/singlefs/crates/singlefs-checker/invariants/06-事务与崩溃一致性.md)
- **文件规模**：21 行，16877 字节
- **判定结论**：⚠️ **存在过时或对不上的内容**
- **具体过时/不一致点**：
  - 第 9 行：引用的文件路径已不存在：`research/prompts/m3-delete-checker-r1-forks.md`
  - 第 12 行：引用的文件路径已不存在：`research/prompts/m3-delete-checker-r1-forks.md`
- **建议处置**：按最新代码基准、门禁格名（如 `pinned-red-examples-table`）及 crate 数量（8 个）进行定点 `replace-once.py` 更新。

## [crates/singlefs-checker/invariants/07-分配器.md](file:///home/fy5090/code/singlefs/crates/singlefs-checker/invariants/07-分配器.md)
- **文件规模**：25 行，17479 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：路径引用有效，术语、常量与当前 codebase 严格对齐。

## [crates/singlefs-checker/invariants/08-加密不变量.md](file:///home/fy5090/code/singlefs/crates/singlefs-checker/invariants/08-加密不变量.md)
- **文件规模**：51 行，6394 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：路径引用有效，术语、常量与当前 codebase 严格对齐。

## [crates/singlefs-checker/invariants/09-inode树结构.md](file:///home/fy5090/code/singlefs/crates/singlefs-checker/invariants/09-inode树结构.md)
- **文件规模**：55 行，60213 字节
- **判定结论**：⚠️ **存在过时或对不上的内容**
- **具体过时/不一致点**：
  - 第 28 行：引用的文件路径已不存在：`tests/tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker.rs`
  - 第 42 行：引用的文件路径已不存在：`research/prompts/m3-w3-r1-sonnet-output.md`
- **建议处置**：按最新代码基准、门禁格名（如 `pinned-red-examples-table`）及 crate 数量（8 个）进行定点 `replace-once.py` 更新。

## [crates/singlefs-checker/invariants/README.md](file:///home/fy5090/code/singlefs/crates/singlefs-checker/invariants/README.md)
- **文件规模**：47 行，13818 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：路径引用有效，术语、常量与当前 codebase 严格对齐。

## [scripts/README.md](file:///home/fy5090/code/singlefs/scripts/README.md)
- **文件规模**：19 行，1276 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：路径引用有效，术语、常量与当前 codebase 严格对齐。

## [scripts/agents/README.md](file:///home/fy5090/code/singlefs/scripts/agents/README.md)
- **文件规模**：14 行，1231 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：路径引用有效，术语、常量与当前 codebase 严格对齐。

## [scripts/crash/README.md](file:///home/fy5090/code/singlefs/scripts/crash/README.md)
- **文件规模**：13 行，1665 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：路径引用有效，术语、常量与当前 codebase 严格对齐。

## [scripts/devices/README.md](file:///home/fy5090/code/singlefs/scripts/devices/README.md)
- **文件规模**：12 行，909 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：路径引用有效，术语、常量与当前 codebase 严格对齐。

## [scripts/experiments/README.md](file:///home/fy5090/code/singlefs/scripts/experiments/README.md)
- **文件规模**：10 行，1826 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：路径引用有效，术语、常量与当前 codebase 严格对齐。

## [scripts/experiments/perf-by-milestone.md](file:///home/fy5090/code/singlefs/scripts/experiments/perf-by-milestone.md)
- **文件规模**：768 行，67166 字节
- **判定结论**：⚠️ **存在过时或对不上的内容**
- **具体过时/不一致点**：
  - 第 8 行：引用的文件路径已不存在：`research/prompts/e152-preregistration.md`
  - 第 28 行：引用的文件路径已不存在：`research/prompts/e152-preregistration.md`
  - 里程碑性能对照数据停留在历史版本，里程碑三与四的新增基准（如 compaction 与 truncation）尚未回填。
- **建议处置**：按最新代码基准、门禁格名（如 `pinned-red-examples-table`）及 crate 数量（8 个）进行定点 `replace-once.py` 更新。

## [scripts/gate/README.md](file:///home/fy5090/code/singlefs/scripts/gate/README.md)
- **文件规模**：22 行，4471 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：路径引用有效，术语、常量与当前 codebase 严格对齐。

## [scripts/hosts/README.md](file:///home/fy5090/code/singlefs/scripts/hosts/README.md)
- **文件规模**：16 行，2035 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：路径引用有效，术语、常量与当前 codebase 严格对齐。

## [scripts/kb/README.md](file:///home/fy5090/code/singlefs/scripts/kb/README.md)
- **文件规模**：13 行，1654 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：路径引用有效，术语、常量与当前 codebase 严格对齐。

## [scripts/mutation/README.md](file:///home/fy5090/code/singlefs/scripts/mutation/README.md)
- **文件规模**：12 行，2567 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：路径引用有效，术语、常量与当前 codebase 严格对齐。

## [scripts/mutation/tables/README.md](file:///home/fy5090/code/singlefs/scripts/mutation/tables/README.md)
- **文件规模**：60 行，3877 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：路径引用有效，术语、常量与当前 codebase 严格对齐。
