#!/usr/bin/env bash
# 五条判据各犯一处，分在三处被管文件里 ⇒ 本该判红。② 另有两处：别处脚本的路径、只提到 .claude/kb/ 目录，都不算指路。
# ⑤ 的五种各一行，放在共用约束样本末尾，每行只犯 ⑤。样本由这里生成，不放成 .md：doc-lint 会扫 fixtures 下的 .md。
set -e
mkdir -p .claude/agents
cat > .claude/agents/demo.md <<'MD'
---
name: demo
tools: Read, Bash
omitClaudeMd: true
---

# 样本（demo）

开工先读 `.claude/agent-common.md`。

实测：这一段在记一次经过。

## 做什么

1. 跑完贴原样输出，因为别的会话会改仓。
2. 照规格写回。

## 历史版本

- 这一节不该在定义里。
MD
cat > .claude/agent-common.md <<'MD'
# 共用约束样本

- 在主工作区改（用户 2026-09-16 定）。
- 另跑一次 `bash .claude/singlefs-ai-sop/scripts/doc-lint.sh .` 贴末行（2026-09-17 两次试跑都漏了）。
- 快照路径（`crates/` 与 `.claude/kb/` 两样；2026-09-17 步 6 那一轮没给）。
- 54 号在本机负载下实测约 47 分钟。
- 用它不用逐条 Edit（agent-defs-r2 攻方模型：1/3 对 0/3）。
- 表头写着 shell 探针的（今天 4 张）照表头跑。
- 跑产物之前连别的 cargo 也要等，免得读数被抢。
- target 放草稿目录，计划第十八节。
MD
cat > .claude/main-agent.md <<'MD'
# 主 agent 样本

职责写在这里。
依据：这一行接在上一句后面，渲染出来是同一段的后半句。
MD
