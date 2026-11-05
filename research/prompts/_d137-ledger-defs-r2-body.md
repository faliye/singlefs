# d137-ledger-defs-r2：账本逐节现查那一批的定义与规则改动——第二轮（2026-11-02）

## 一、题面

第一轮 `d137-ledger-defs-r1` 的判决在 `research/prompts/d137-ledger-defs-r1-main-verification.md`：两条腿共打中 13 处，判「采」的分两件改完。

- 定义、规则与 skill：报告 `~/.local/share/singlefs/work/d137-ledger-defs-r1-fix-defs/result.md`，本批 diff 在同目录 `this-batch.diff`（sha256 前 8 位 c5376a70）；主 agent 另照它给的旧串新串改了 `.claude/skills/crash-test/commit-stages.md` 第 10、19 行与 `.claude/skills/crash-test/SKILL.md` 第 21 行，并把 `commit-stages.md` 第 23 行广谱变异驱动那一段改成「清场由服务在最外层做」（驱动已不自己清场，文件头 `# task-server-gpu-arguments: *`）。
- 三份脚本：报告 `~/.local/share/singlefs/work/d137-ledger-defs-r1-fix-scripts/result.md`，`scripts/hosts/compile-then-swap.py` 加 `--features`，`.claude/gate.d/doc-experiments.sh` 的 preregistration-judge-features 格先并续行、按判段入口表 `JUDGE_ENTRIES` 认，`scripts/gate/merge-and-verify.sh` 把产品 GPU 包算进快档射程。

这一轮按 `.claude/rules/three-way-inference.md`「多轮」第二轮分工：攻第一轮的修补本身、产生结论的方法、第一轮没看过的地方；辩方腿复核第一轮的判决够不够得着、判「采」的那几条改法是不是同样打中别的写法。做法本身（子 agent 不交任务、经统一任务服务交）不重议。

被判的文件（快照 `research/prompts/d137-ledger-defs-r2-snapshot/files-sha256.txt`，16 份）：`.claude/agent-common.md`，`.claude/agents/` 下 clerk、experiment-designer、experiment-runner、implementation-writer、mutation-triage、three-way-attack、tooling-writer，`.claude/main-agent.md`，`.claude/rules/verification.md`、`.claude/rules/implementation-workflow.md`，`.claude/skills/crash-test/commit-stages.md`、`.claude/skills/crash-test/SKILL.md`，`scripts/hosts/compile-then-swap.py`、`scripts/gate/merge-and-verify.sh`、`.claude/gate.d/doc-experiments.sh`。

## 二、实现今天的样子（主 agent 的观测，2026-11-02）

- 这一批不碰 `crates/` 的源码；判段入口表 `JUDGE_ENTRIES` 是工具员从 `crates/singlefs-checker-tier/src/crash_amplification.rs` 与 `crash_state_enumeration.rs` 现查出的名字，腿现查核对。
- 已知还没对齐、留给这一轮核的：`.claude/rules/mutation-sampling.md` 第 79 行还写「六栏」（分诊定义已改成九栏）；第一轮判决里留作欠账的两样（门禁另一种本次未跑、只在共用约束括注里提到的三份规则）。
- 脚本的自证与开关趟主 agent 已经交服务、结局待出（任务 4905–4912）；`doc-experiments.sh` 的新样本要先暂存才跑得了样本档，归提交前。定义修补的门禁单格任务 4898–4900。

## 三、格

### D　复核第一轮判决（辩方腿）
逐条看第一轮判决「逐条判」表：判「采」的那几条，现查证据够不够得着结论；判「不采」的两条理由站不站得住；改法是不是只修了被打中的那一种写法、同样打中别的写法（例：入口表漏掉的判段入口、续行之外的多行写法、产品 GPU 包之外的同类包）。

### A　攻第一轮的修补（攻方腿）
攻两件修补写出来的新句子与新代码：照新定义做会不会撞拒或交出门禁判不对的东西；新定义与新脚本、新门禁三处的说法对不对得上（例：定义里 `compile-then-swap.py --features` 的写法、门禁的判段入口表、`merge-and-verify.sh` 读产品 GPU 包名的办法）；新开关与新自证格能不能真的分出改前改后；这一批该跟改而没改的地方。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 辩方腿 | D | 替第一轮判决与被判出局的写法辩护，复核判决够不够得着 |
| 攻方腿 | A | 找第一轮修补的漏洞；不重复辩方的复核，不攻做法本身，不重复第一轮已攻过的角度 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引定义、规则、脚本、门禁写主工作区的路径与行号，同时写那份文件此刻的 sha256 前 8 位；快照里的 sha256 与此刻对不上的，写明是哪一份。
- 子 agent 不往统一任务服务交任务：要跑的命令（喂钩子、自证、门禁单格、模型）拼成 `task-server/task submit -- <命令>` 的整条，写进报告「要主 agent 跑的命令」一节，由主 agent 交。
- 不造新的绕过钩子的写法。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。草稿与临时文件放 `~/.local/share/singlefs/work/<腿名>/`。
