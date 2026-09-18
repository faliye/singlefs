#!/usr/bin/env bash
# 与红样本同样三处文件，全部合规，另放每条判据的「不认」形态 ⇒ 本该判绿：
# 规则小节名里带日期、放在「」里原样引（含一层嵌套引号）；日期在 records/ 文件名里；顿号后的「依据」「实测」；
# 「为什么这么定」「背景材料路径」这种标签词直接接别的字；表格行里的「因为」；围栏里的日期与「实测」；
# 读取指令点名的小节名里带「（…实测 3/3…）」（④ ⑤ 都在剥掉「」之后判）；表格行里的「今天 4 张」「，所以」（表格行不判 ⑤）。
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
开工先读：`.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算（2026-09-06 用户明令）」。
开工先读：`.claude/kb/tooling.md`「提示里的 `**粗体**:` 会诱发损坏（2026-08-29 实测 3/3 对 3/3）」。

## 输入（主 agent 必须给）

- 背景材料路径（用来识别误写成背景材料行号的引用）。

## 做什么

1. 跑完贴原样输出；写结论、依据与没做什么。
2. 照 `.claude/kb/vm-harness.md`「计时不在转发输出的循环里打时间戳」一节办。
3. 照 `.claude/rules/demo.md`「外层「内层」小节（2026-09-06 用户明令）」办。

| 项 | 说明 |
|---|---|
| 表格行不判解释 | 数据，因为表格只放数 |
| 表格行不判词法说明 | 今天 4 张，所以只放数 |

```text
2026-09-17 实测：围栏里的行三条都不判
```

## 没做什么（固定会有的）

- 不替主 agent 定案。
MD
cat > .claude/agent-common.md <<'MD'
# 共用约束样本

这份与各份定义只写怎么做；为什么这么定、实测的数在 `records/2026-09-16-subagent拆分提案.md`。
MD
cat > .claude/main-agent.md <<'MD'
# 主 agent 样本

职责写在这里。
判决与写回只引产物与代码。
MD
