<!-- knowledge-sync -->
# governance-rot-r2 阶段同步

触发文件：CLAUDE.md、.claude/hooks/bash-command-detector.sh、.claude/hooks/runner-dispatch-guard.sh、.claude/agent-common.md（前三个随提交 3e99fcd 已进仓、当时漏写同步记录，这里补上）；另改了 .claude/main-agent.md（不在触发登记里，一并记）

这一阶段做成的事：治理文档之间同一件事说法不一致的几处改成一致；能指向唯一出处的不再手抄第二份（重型测试清单、钩子拒绝的写法）。

## 搜索

- `grep -n "重型测试" CLAUDE.md .claude/main-agent.md .claude/agent-common.md .claude/agents/*.md .claude/rules/*.md .claude/skills/*/SKILL.md` → 带清单的 4 处：main-agent.md 两处少 87 号与 E152，implementation-workflow.md:48（定义处）与 agent-common.md:46 一致；拦截钩子 .claude/hooks/lib_heavy_tests.py:52-53 含「全部实验复跑」「E152 装置」
- `grep -n "拒绝[五六七八九]种\|[五六七八]种写法"` 治理文档与钩子 → bash-command-detector.sh 两处「拒绝六种」、agent-common.md 一处「五种写法」；钩子文件头拒绝一节 ①–⑦ 共七条（`grep -n "^# [①-⑦]" .claude/hooks/bash-command-detector.sh`）
- `diff <(ls .claude/singlefs-ai-sop/rules/*.md | xargs -n1 basename | sort) <(grep -o "^@.claude/singlefs-ai-sop/rules/[^ ]*" CLAUDE.md | xargs -n1 basename | sort -u)` → 无差异；`grep -c "^@.claude/singlefs-ai-sop/rules/rules-discipline.md" CLAUDE.md` → 2（重复一行）
- 派发闸误拒：同一句喂给 runner-dispatch-guard.sh，「不跑重型测试（cargo test --workspace）。」退出 2，「不跑 gate.sh、cargo test --workspace。」退出 0
- 五个只读核对员逐份核 29 份治理文档，已交回的 E 组报告原样在 research/prompts/governance-rot-r2-audit-E.md（19 条，其中 3、6、7、11、12、14、15 已现查属实，处置在下一批改动里）

## 命中处置

| 载体 | 原句 | 处置 |
|---|---|---|
| .claude/main-agent.md:14 | 禁止在subagent中跑重型测试（层 0 全量、QEMU、herd7、`crates` 变异整表、全量测试、整轮门禁）等 | 改了：清单改成指向 implementation-workflow.md「重型测试只在提交时跑」开头那一句 |
| .claude/main-agent.md:20 | **重型测试**（层 0 全量、QEMU、herd7、`crates` 变异整表、全量测试、整轮门禁）提交之外 | 改了：同上 |
| .claude/main-agent.md:30 | 拒绝一种危险写法的（起看门狗的错误写法、……、覆盖未跟踪文件）在执行前拒绝 | 改了：改成「各拒什么写在各钩子文件头」，清单用 gate-overlap.py --list 列 |
| .claude/agent-common.md:57 | 对五种写法在执行前拒绝 | 改了：七种，补 ⑥ 终止进程不是一次点名一个、⑦ 同一 inode 改脚本，并写各自怎么做 |
| .claude/hooks/bash-command-detector.sh:13 | 见「拒绝六种」② | 改了：拒绝七种 |
| .claude/hooks/bash-command-detector.sh:15 | 见「拒绝六种」③ / ④ | 改了：拒绝七种 |
| .claude/hooks/runner-dispatch-guard.sh:41 | （新增） | 补了：「会误拒」一节补否定词在括号外、名字在全角括号里这一形态 |
| .claude/hooks/runner-dispatch-guard.sh:441 | 不是要它跑的，写成否定句（「不跑层 0」）， | 改了：补「否定词与名字放在同一个分句里」 |
| CLAUDE.md:34 | 重复的 @.claude/singlefs-ai-sop/rules/rules-discipline.md | 改了：删掉重复的一行 |
| .claude/agent-common.md:46 | 重型测试（层 0、QEMU、……、87 号全部实验复跑、E152 装置） | 不改：与定义处 implementation-workflow.md:48 逐项一致；子 agent 不读 CLAUDE.md，这份清单留在共用约束里 |

## 原始证据

| 材料 | 路径 | 行数 |
|---|---|---|
| E 组核对员报告（三方规则、变异规则、三个 skill） | research/prompts/governance-rot-r2-audit-E.md | 67 |
