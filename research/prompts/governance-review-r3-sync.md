<!-- knowledge-sync -->
# governance-review-r3 阶段同步

触发文件：.claude/agent-common.md、.claude/agents/crash-verifier.md、.claude/agents/experiment-designer.md、.claude/agents/gate-triage.md、.claude/agents/implementation-writer.md、.claude/agents/kb-scribe.md、.claude/agents/mutation-triage.md、.claude/agents/sweep.md、.claude/agents/three-way-verifier.md、.claude/gate.d/59-crates-mutation-replay.sh、.claude/gate.d/lib-governance-refs.py、.claude/hooks/runner-dispatch-guard.sh、.claude/rules/format-evolution.md、.claude/rules/mutation-sampling.md、research/scripts/ask-local-selftest.sh；另改了 .claude/main-agent.md、门禁 10 号两份样本与本分支带进来的证据副本 research/prompts/governance-review-r1-report-B.md（不在触发登记里，一并记）；改动范围以 d6a3bcb 为基准（第三轮严查开跑时的 HEAD）

这一阶段做成的事：对分支相对 73ba4a4 的全部改动做第三轮（确认轮）严查（甲组 6 条、乙组 11 条、丙组 7 条，报告原样入库，见「原始证据」），逐条现查后改了 23 条；丙组第 5 条（main-agent 那句照抄进派发提示会被派发闸拒）是派发闸文件头已登记的误拒形态，不改。改法被攻过零轮。

## 搜索

- 门禁 10 号两份样本 → 绿样本退 0、红样本退 1，want 全中；副本里四种改坏（lib 一律退 0、后顾去掉「–」、去掉「日」、仓外记号不计数）各自都让一份样本判错
- 往样本拷贝里放一份非 UTF-8 的定义 → lib 退 4、打出「lib-governance-refs.py 自己出错」，10 号报「没跑成（退出码 4）」
- `bash .claude/gate.d/12-no-prime-marks.sh` → 改后只剩 research/prompts/m2-lastflag-implementer-report.md:130（提交 f669c52，基准之前就在）
- 59 号内嵌 python 抽出来 `python3 -m py_compile` 通过；59 号的判别力样本在这个容器里跑不了（没有用户级 systemd），这一处没有样本复核
- `runner-dispatch-guard.sh --selftest` 113 种通过；`ask-local-selftest.sh` 通过

## 命中处置

| 载体 | 原句 | 处置 |
|---|---|---|
| .claude/gate.d/fixtures/10-kb-rot.sh/red/expect:5 | （第 4 段判红那一支没有样本） | 补了：丙 1 |
| .claude/gate.d/lib-governance-refs.py:177 | python 自己出错退别的码 | 改了：丙 2：退 4 |
| .claude/gate.d/lib-governance-refs.py:130 | 仓外与带占位的不计数 | 改了：丙 3 |
| .claude/gate.d/fixtures/10-kb-rot.sh/green/.claude/agents/sample.md:9 | （「–」「日」与仓外记号没有样本） | 补了：丙 4 |
| .claude/hooks/runner-dispatch-guard.sh:569 | 层 0 归 crash-verifier | 改了：丙 6 |
| research/scripts/ask-local-selftest.sh:12 | 缺了哪样红的就是「检测器找不到」 | 改了：丙 7 |
| .claude/main-agent.md:59 | 要跑时 54 号跑快档并核全绿标记 | 不改：丙 5，派发闸文件头已登记的误拒形态，照抄进提示时按闸给的出路改写 |
| research/prompts/governance-review-r1-report-B.md:61 | （角标写法：P 加 U+2032） | 改了：甲附带发现、门禁 12 号：本分支带进来的证据副本，只改这一个记号 |
| .claude/rules/mutation-sampling.md:75 | 无效要改的是那一行替换文 | 改了：甲 A1 |
| .claude/gate.d/59-crates-mutation-replay.sh:348 | 替换文写进源码之后编不过 | 改了：甲 A1 |
| .claude/gate.d/59-crates-mutation-replay.sh:413 | 改这一行替换文，不是去补用例 | 改了：甲 A1 |
| .claude/main-agent.md:54 | 派腿时刻与开工快照 | 改了：甲 A2：快照只代码轮给 |
| .claude/agent-common.md:57 | 按模式找进程这类命令只记检出 | 改了：甲 A3 |
| .claude/agent-common.md:46 | 251（scope 起不来）一行都没跑 | 改了：甲 A4 |
| .claude/rules/format-evolution.md:70 | 别处它都不扫：… | 改了：甲 A5 |
| .claude/agent-common.md:35 | tools 只有 Read / Bash 的 | 改了：甲 A6 |
| .claude/agents/kb-scribe.md:32 | 21 号在第 3 步的 --write 之前 | 改了：乙 1 |
| .claude/agents/gate-triage.md:27 | 碰了 stage-inputs.tsv 登记的路径 | 补了：乙 2：54 号脚本本身 |
| .claude/agents/implementation-writer.md:19 | （输入没有只修锚点这种活） | 补了：乙 3 |
| .claude/agents/three-way-verifier.md:28 | sha256sum -c 核那个文件；git status | 补了：乙 4、乙 5 |
| .claude/agents/three-way-verifier.md:32 | 要编译的写核不动 | 改了：乙 6：经内存包装在副本里跑 |
| .claude/agents/mutation-triage.md:27 | 「共 N 条」与各栏加起来对得上 | 改了：乙 7 |
| .claude/agents/crash-verifier.md:24 | 55 号只查 crates | 补了：乙 8：由 87 号兜 |
| .claude/agents/crash-verifier.md:29 | 指纹只罩 crates litmus | 改了：乙 9 |
| .claude/agents/experiment-designer.md:28 | 重跑登记照抄原登记的英文名 | 补了：乙 10 |
| .claude/agents/sweep.md:31 | 照 path-moves.md「怎么做」逐步改 | 改了：乙 11 |

## 原始证据

| 材料 | 路径 | 行数 |
|---|---|---|
| 甲组审查报告（规则与入口） | research/prompts/governance-review-r3-report-A.md | 50 |
| 乙组审查报告（agent 定义） | research/prompts/governance-review-r3-report-B.md | 71 |
| 丙组审查报告（门禁钩子脚本） | research/prompts/governance-review-r3-report-C.md | 41 |
