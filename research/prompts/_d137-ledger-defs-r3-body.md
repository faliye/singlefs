# d137-ledger-defs-r3：账本逐节现查那一批的定义与规则改动——第三轮（最终轮，2026-11-02）

## 一、题面

前两轮判决：`research/prompts/d137-ledger-defs-r1-main-verification.md`、`research/prompts/d137-ledger-defs-r2-main-verification.md`。第二轮判「采」的各条已改完：

- 定义与规则：报告 `~/.local/share/singlefs/work/d137-ledger-defs-r2-fix-defs/result.md`；主 agent 另改 `.claude/rules/verification.md` 第 35 行核卡三处、`.claude/skills/crash-test/commit-stages.md` 第 19 行（要卡三处、重跑三格）与第 23 行（带 `--skip-gpu` 的那一片服务照样清场分卡）。
- 三份脚本：报告 `~/.local/share/singlefs/work/d137-ledger-defs-r2-fix-scripts/result.md` 与承接的 `~/.local/share/singlefs/work/d137-ledger-defs-r2-fix-mav/result.md`；主 agent 经服务复跑：`compile-then-swap.py` 任务 5090–5095，`merge-and-verify.sh` 任务 5119–5123，基线退 0、开关红格与点名逐格相同。`doc-experiments.sh` 的新样本要先暂存才喂得了样本档，还没跑。
- 同一批另改了几份定义的「开工先读」：三份规则（`session-wrapup.md`「5. 子 agent 交回之前，删掉自己建的编译目录与仓副本」、`timer-only-remind.md`「不用」、`path-moves.md`「怎么做」与「改一个全仓术语：正文之外还有五处会红」）由 implementation-writer、investigator、three-way-attack、tooling-writer、kb-scribe 按小节点名，`.claude/agent-common.md` 删两处重复括注（报告 `~/.local/share/singlefs/work/d137-b19-three-rules-named/result.md`）。
- 入库装置二进制要卡：统一任务服务已换上按同名源 `src/bin/<名字>.rs` 前 60 行的 `// task-server-gpu-arguments:` 认要卡；E156、E158、E161、E168 四份源文件头写了声明（`~/.local/share/singlefs/work/d137-ledger-defs-r2-bin-gpu-headers/result.md`），E171 两份归做 E171 的会话。

这一轮按 `.claude/rules/three-way-inference.md`「多轮」第三轮分工：前两轮站住的形态撑不撑得住；前两轮判「不采」、记欠账、交别的会话的那些去向完整不完整；判决里报的数能不能核。第三轮之后不再开，这一轮新冒出的零轮形态写进交用户表、标「零轮」。做法本身不重议。

被判的文件（快照 `research/prompts/d137-ledger-defs-r3-snapshot/files-sha256.txt`，19 份）：`.claude/agent-common.md`，`.claude/agents/` 下 clerk、experiment-designer、experiment-runner、implementation-writer、investigator、kb-scribe、mutation-triage、three-way-attack、tooling-writer，`.claude/main-agent.md`，`.claude/rules/verification.md`、`implementation-workflow.md`、`mutation-sampling.md`，`.claude/skills/crash-test/commit-stages.md`、`SKILL.md`，`scripts/hosts/compile-then-swap.py`、`scripts/gate/merge-and-verify.sh`、`.claude/gate.d/doc-experiments.sh`。

## 二、实现今天的样子（主 agent 的观测，2026-11-02）

- 这一批不改 `crates/` 的源码，四份入库装置 `crates/singlefs-checker-tier/src/bin/` 只加了 `//` 注释；判段入口表 `JUDGE_ENTRIES` 由 `doc-experiments.sh` 新格 judge-entries-in-sync 核与 `crash_amplification.rs`、`crash_state_enumeration.rs` 同步。
- 门禁单格：code-tooling 的 rules-named-by-definitions、agent-definition-shape、fixture-claims、break-switch-headers 绿（任务 5106、5126）；上游 rules-lint 只红 `CLAUDE.md:111`（HEAD 里就在）。
- 第一轮留的两样欠账在 `records/2026-10-29-subagent拆分提案.md` 一百八十节；第二轮「入口表与源码同步」已做成新格。

## 三、格

### R　前两轮站住的形态撑不撑得住（辩方腿）
逐条看前两轮判「采」并已落地的改法：落地的句子与代码是不是真做到了判决写的那一句（例：九栏计数、三种出路、三处要卡、按判段入口认、续行与注释的认法、特性写法取并集、产品 GPU 包读不出与不是成员都照跑）；前两轮判「不采」与交别的会话、记欠账的去向是不是都有落点（判决里的文件:行、账本节号现查得到）；判决里报的任务号与数（红格数、项数）能不能从 `task-server/task log <号> --all` 核到（命令写进报告交主 agent 跑）。

### A　找第三类漏洞（攻方腿）
不重复前两轮攻过的角度（第一轮 A1–A13、第二轮 B1–B11 不再原样提）。攻第二轮修补之后才有的东西：新加的格与开关、新的认法表（`JUDGE_ENTRIES`、文件头 `task-server-gpu-arguments`）、三份规则按小节点名之后定义之间互相说的话、改动之后定义与规则和服务的说法还有没有对不上的地方。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 辩方腿 | R | 复核前两轮判决与落地，核数 |
| 攻方腿 | A | 只攻第二轮修补之后才有的东西 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引定义、规则、脚本、门禁写主工作区的路径与行号，同时写那份文件此刻的 sha256 前 8 位；快照里的 sha256 与此刻对不上的，写明是哪一份。
- 子 agent 不往统一任务服务交任务：要跑的命令（只读的 `task-server/task log` 除外）拼成 `task-server/task submit -- <命令>` 的整条，写进报告「要主 agent 跑的命令」一节，由主 agent 交。
- 不造新的绕过钩子的写法。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。草稿与临时文件放 `~/.local/share/singlefs/work/<腿名>/`。
