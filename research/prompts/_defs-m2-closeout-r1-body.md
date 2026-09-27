# 里程碑二收尾：定义改动一轮三方，第一轮正文（2026-09-26）

<!-- doc-lint:not-numbers D1 D2 D3 D4 -->

## 一、这一轮要判什么

`.claude/rules/implementation-workflow.md`「改 agent 定义与共用约束，走同一条三步」：改完的定义走一轮三方，判决按路径点名每一份改过的定义（门禁 72 号）。这一轮被判的是 HEAD（`73ba4a4`）之后工作区里这 9 份定义的改动，以及 `97f5904` 那次提交（用户要求跳过验证）里从来没有一份判决判到内容的几块。改动的来历与逐项理由：两份起草报告 `research/prompts/defs-closeout-draft-tmp-evidence/report.md`、`research/prompts/defs-closeout-draft-2-tmp-evidence/report.md`（材料员把它们要紧的节抄进附录，不指 `/tmp` 当依据）；欠账原文在 `records/2026-09-16-subagent拆分提案.md` 第四十节第 1、29、30、33、40 行。

| 格 | 被判的改动 | 问题 |
|---|---|---|
| D1 | **第一批**：`three-way-local-attack.md`、`three-way-local-defense.md` 写范围与产出一致；`three-way-attack.md` 第 3b 步取样（内存稀疏盘、每个起点只建一次池、超过 40 分钟缩历史 / 候选 / 几何的取样而崩溃点不缩、checker 判结束状态、随机跑批小批量限时）；`experiment-runner.md` 第 4c 步逐行读 `name=verdict` 判决行、false 与 not_run 点名 | 照这几步干活会不会与别的定义、共用约束或 `.claude/singlefs-ai-sop/rules/` 冲突；起草者自己加的两处（「崩溃点不缩」与共用约束「崩溃点…不为省时间缩范围」怎么对上，段内整段子集枚举算不算崩溃点；「checker 判结束状态」是每段历史判一次还是每个崩溃状态判一次）站不站得住 |
| D2 | **第二批**：`agent-common.md` 新加「跑编译出来的代码经内存包装」与「执行前拒绝的写法」两条，`experiment-runner.md`、`crash-verifier.md`、`gate-triage.md`、`implementation-writer.md` 各一个第 1b 步指过去；`three-way-attack.md` 第 3c 步「内存与进程」；`.claude/gate.d/stage-owners.tsv` 登记 84 号给 `experiment-runner` | 拒绝清单与 `.claude/hooks/` 里四份 hook 今天实际拒的逐项对得上吗（多写、漏写、写反）；「外面不再包」的嵌套理由（推的，没量过）；崩溃验证员派发没给上限时默认 8G 够不够（第四十节第 30 行写着三道要多少内存都没量过）；门禁分诊员不包 `gate.sh` 之后，`check.sh`、15 号、74 号起的 cargo 不在任何包装里；攻方第 3c 步要求 `cargo build` 也经包装、比共用约束严，要不要统一 |
| D3 | **97f5904 里没被判到的几块，与主 agent 2026-09-26 JST 09:1x 的四处小改**：`main-agent.md` 第 5 条「弹窗问用户之前…出处」与随它顺延的编号、「禁止」一节重型清单改成「以 `implementation-workflow.md` 那张清单为准」、第 3 条同、「派出去之后」拒绝清单改成指到共用约束、「暂存之后、提交之前跑门禁」那一行层 0 全量写明整条经内存包装；`agent-common.md` 停进程那一句的 `proc.py` 路径；`implementation-writer.md` 第 4 步变异表名改成 `crates/mutations.tsv`、删掉末尾打补丁交法留下的 `git apply --check`；第一批报告第五、六节列的「只被通查扫过」的各块 | 每一块照字面做，会不会让派出去的 agent 做错、做不了、或与另一处矛盾；「以那张清单为准」这类指过去的写法，被指的那一处真的列全了吗 |
| D4 | **第四十节第 40 行**：重型测试闸遇到 55、57、59 号不起虚机、不起 herd7、不跑变异的静态分支怎么判（今天靠根目录的标记文件选分支：55 号认 `.qemu-prerecorded`、57 号认 `.lkmm-static-only`，59 号没有静态分支）。两种判法：甲，参数白名单（钩子这一侧认）；乙，阶段自报（阶段自己声明，钩子读声明） | 两种判法各会误放、误拒哪些调用（举具体命令）；乙要改 54 号的话层 0 全绿标记全部作废，这个代价是不是必然的；有没有第三条路；`herd7 -version` 这类只取版本号的裸调用各怎么判 |

**共用问句**：每一格都要回答「按改后的字面干活，哪一步会做错或做不了」，举出具体的派发情形或命令，不许只说「可能有歧义」。

## 二、实现今天的样子（主 agent 的观测，2026-09-26 JST 09:1x）

- 被判的 9 份定义与 1 份阶段归属表，工作区相对 HEAD 的改动：`git diff HEAD -- .claude/agents .claude/agent-common.md .claude/main-agent.md .claude/gate.d/stage-owners.tsv`（10 个文件，41 行加、11 行删；材料员整份放进附录二）。开工快照 `research/prompts/defs-m2-closeout-r1-snapshot/sha256sums.txt`。
- `97f5904` 里 7 份定义的改动：`git show 97f5904 -- .claude/agent-common.md .claude/agents/crash-verifier.md .claude/agents/experiment-runner.md .claude/agents/gate-triage.md .claude/agents/implementation-writer.md .claude/agents/mutation-triage.md .claude/main-agent.md`；哪几块被 `research/prompts/defs-gate54-tiering-r1/r2-main-verification.md`、`m2-final-code-r2/r3/r4-main-verification.md` 判到过，见第一批起草报告第五节的覆盖表。
- 四份拒人的 hook：`.claude/hooks/bash-command-detector.sh`（`main()` 起第 2339 行，七种拒绝 ①–⑦）、`.claude/hooks/write-guard.sh`（`decide()` 第 159–170 行）、`.claude/hooks/heavy-test-guard.sh`、`.claude/singlefs-ai-sop/scripts/claude-hooks/pattern-process-guard.sh`；第二批起草者喂 JSON 只看退出码的探针与原样输出在 `/tmp/claude-1000/defs-closeout-draft-2/probe-hooks.log`（材料员抄进附录）。
- 这一轮不碰 `crates/`：被判的都是定义与门禁表；`crates/` 里跟这一轮有关的只有 `crates/singlefs-harness/src/crash.rs` 的 `SparseBlockDevice`（攻方第 3b 步点名的内存稀疏盘），材料员照工作区现查行号。

## 三、条款（材料员整段抄进附录）

- `.claude/rules/implementation-workflow.md`「三步，缺一步就不算做完」「改 agent 定义与共用约束，走同一条三步」「重型测试只在提交时跑」；
- `.claude/singlefs-ai-sop/rules/command-safety.md`「起了后台任务和子 agent，就要定时复检，不强制结束」「`pkill -f` / `killall` 一律禁用」；
- `.claude/singlefs-ai-sop/rules/rules-discipline.md` 第 1、2 条（定义同规矩：只写怎么做）；
- `.claude/rules/three-way-inference.md`「各条腿必须互不重复」「一条腿只抽一次样不算一次观测——否定结论尤其不算」；
- `records/2026-09-16-subagent拆分提案.md` 第四十节第 1、29、30、33、40 行（欠账原文，整行抄）。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | D1、D2、D3 | 逐条核「改后的字面做的是不是那一行欠账要的」：每条改动对上第四十节哪一行的哪一句，欠账要的有没有漏做、有没有做过头 |
| 云端攻方（Opus） | D1、D2、D3、D4 | 造派发情形与命令攻：照改后的字面干活会做错、做不了、或两处定义互相打架的；D4 两种判法各举误放、误拒的具体命令 |
| 本地攻方 | D2 | 按事实表逐格填：`agent-common.md`「执行前拒绝的写法」列的每一种写法，在探针日志里是拒（退 2）还是放（退 0）；每行写来源文件与行号，每张表不超过 6 行 |

两条攻方腿不重叠：Opus 造情形与命令攻语义，本地只对着探针日志逐格核清单的字面。

## 五、交付

- 腿的报告写 `research/prompts/defs-m2-closeout-r1-<腿名>-output.md`，分段落盘（每次工具调用写进文件不超过 150 行，第一段排他新建）。
- 要喂 hook 做探针的，放 `research/prompts/defs-m2-closeout-r1-<腿名>-model/`，带 `SHA256SUMS` 与 `rerun.sh`；探针只喂 JSON、只看退出码，被判的命令一条都不执行。
- 引 kb 与规则写那份文件自己的行号，现查，不从背景材料里数。
- 不跑重型测试；只停自己起的进程，一次一个 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`；交回之前后台不许留着跑的东西。
