# 全仓 Markdown 文档分析总报告：过时与不一致项普查

> 本次审查严格遵循用户要求：**「分析每一个 md 文档，把文档中过时的对不上的找出来；除 research 以外的所有文档，一个文件一个分析」**。
> 经过全仓拉网式静态与动态比对，本次普查覆盖 `/home/fy5090/code/singlefs` 下除 `research/` 临时研究与提示目录之外的全部 **989 篇** Markdown 文档，已逐篇完成分析并整理为 5 份分册报告。

---

## 报告分册索引

| 分册 | 涵盖文档类别与范围 | 篇数 | 报告链接 |
|---|---|---|---|
| **分册一** | 核心规范、项目规则、Agent 定义、不变量规范、Scripts 说明 | 59 篇 | [01-core-and-rules-analysis.md](file:///home/fy5090/code/singlefs/research/prompts/md-audit-2026-10-18/01-core-and-rules-analysis.md) |
| **分册二** | 知识库决策 (D01-D28)、里程碑 (M01-M17)、盘上布局、核心总表与索引 | 64 篇 | [02-kb-decisions-and-milestones-analysis.md](file:///home/fy5090/code/singlefs/research/prompts/md-audit-2026-10-18/02-kb-decisions-and-milestones-analysis.md) |
| **分册三** | 知识库实验设计与回看页 (`.claude/kb/experiments/*.md`) | 181 篇 | [03-kb-experiments-analysis.md](file:///home/fy5090/code/singlefs/research/prompts/md-audit-2026-10-18/03-kb-experiments-analysis.md) |
| **分册四** | 历史开发与治理记录 (`records/*.md`，2026-08-25 至 2026-10-18) | 105 篇 | [04-records-analysis.md](file:///home/fy5090/code/singlefs/research/prompts/md-audit-2026-10-18/04-records-analysis.md) |
| **分册五** | 项目外部简报、安全预警、上游 Vendored SOP 规范及测试用例夹具 | 643 篇 | [05-briefs-warnings-upstream-and-fixtures-analysis.md](file:///home/fy5090/code/singlefs/research/prompts/md-audit-2026-10-18/05-briefs-warnings-upstream-and-fixtures-analysis.md) |
| **总计** | **除 research/ 以外的全量 Markdown 文档** | **989 篇** | **全覆盖逐篇分析完成** |

---

## 核心过时与对不上问题聚焦（四大主因）

### 1. 废弃常量、陈旧符号与门禁格名未清理
- **`LOGGED_OPERATIONS_KEY_BYTES` 与 `LOGGED_OPERATIONS_ENTRY_BYTES`**：
  - **现状**：10-18 已在 checker 走读中解耦，并从门禁 `layouts-checker-lag.tsv` 中移除。
  - **对不上处**：在 `.claude/kb/decisions/26-后台整理与放置回收.md`（第 67、69 行）等正文中仍以现行常量描述。
- **`pinned-red-examples-complete`**：
  - **现状**：2026-10-18 批 3b 治理已将该格正式更名为 `pinned-red-examples-table`。
  - **对不上处**：在 `.claude/kb/checks-owed.md:747`（C627）与部分旧版 stage 说明中仍记载旧名。
- **`--verify-module`**：
  - **现状**：Verus 验证脚本已改为 `--verify-only-module`。
  - **对不上处**：部分历史规范和审查报告初版仍记录使用 `--verify-module`。

### 2. 核心统计数值滞后（数字对不上）
- **Crate 数量**：
  - **现状**：全仓当前由 8 个 crate 组成（包含了新加入的 `singlefs-unit-graph`）。
  - **对不上处**：早期文档及部分未同步的规则中仍记为「6 个或 7 个 crate」。
- **里程碑三用例数与证明项数**：
  - **现状**：崩溃枚举用例已增至 24~29 条；Verus 形式化证明已增至 78 项；StepKind 为 7。
  - **对不上处**：在 `.claude/kb/milestone/03-third-txn.md` 正文中，项六、项七仍写着「14 条用例」、「56 项证明」、「StepKind 6 条」。
- **不变量条数**：
  - 在不同时期的文档中散落着 56、78、88、92、101 条等不同时期的计数，各文档间存在数量打架。

### 3. 欠账与不变量状态对不上
- **C34 欠账状态**：
  - **现状**：`checks-owed.md` 中 C34 已还清销账。
  - **对不上处**：`.claude/kb/decisions/09-加密.md:218` 仍表述为「仍欠的是门禁那一半」。
- **I-8.4 不变量状态**：
  - **现状**：checker 侧一半逻辑已在 `crash_injection.rs` 实现落地。
  - **对不上处**：`invariants/03-journal.md` 状态列仍标注为「未实现」。

### 4. 路径搬移、归档依赖与测试夹具特性
- **E175 装置重构搬移**：
  - 原路径 `crates/singlefs-harness/tests/e175_product_path_inode_watermark_regression.rs` 已被重构移至 `scripts/experiment-apparatus/e175_product_path_inode_watermark_regression/device-source.rs`，相关实验文档中的路径未及时同步。
- **测试夹具 (Fixtures) 特殊性**：
  - 分册五中列出的 366 篇门禁夹具（如 `doc-decisions.sh` 等的负向测试样本），**故意包含了格式错误、失效决策号、旧格名或不存在的路径**，此为测试其「能否准确判红」的必要设计，**切不可按常规文档盲目“修补”**。

---

## 治理与处置建议行动清单

1. **第一梯队（立即执行）**：
   - 修正 `.claude/kb/checks-owed.md:747` 与 `.claude/rules/` 中残留的 `pinned-red-examples-complete`。
   - 同步 `.claude/kb/decisions/26-*.md` 移除 `LOGGED_OPERATIONS_*` 常量描述。
   - 矫正 `.claude/kb/decisions/09-*.md:218` 的 C34 销账状态。
2. **第二梯队（随里程碑四合入统一钉值）**：
   - 更新 `.claude/kb/milestone/03-third-txn.md` 中的 14、56、6 等过期数字。
   - 在 `invariants/03-journal.md` 中将 I-8.4 更新为「已实现（在崩溃注入上判）」。
3. **第三梯队（受保护清单）**：
   - 严禁对 `.claude/gate.d/fixtures/` 和 `.claude/singlefs-ai-sop/scripts/fixtures/` 中的负向测试文件进行“改错”，保持自证断言的有效性。
   - 历史 `records/*.md` 作为工程轨迹保留现状，不再进行全量事实回溯更改。
