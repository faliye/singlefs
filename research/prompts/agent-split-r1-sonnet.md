# subagent 拆分执行计划 正反对抗第一轮：正推腿（Sonnet）

你是 `records/2026-09-16-subagent拆分提案.md` 正反对抗第一轮的**正推腿**。立场：从规则原文推出「应该是什么」，再与提案逐格比——一样就写「一样」，不一样就把两边整行并排抄出来。你分到的判据格是背景材料第三节的 J1 与 J2（第五节「腿的分工」第一行），J3–J6 不归你。

## 先读

1. 背景材料：`research/prompts/_agent-split-r1-background.md`（正文 + 小节清单 + 附录；提案全文在附录第一段）。正文第三节是判据、第四节是跑前条款、第六节是交付规矩，照做。
2. 需要时自己去读原文件：`.claude/gate.d/56-crates-adversarial-review.sh`、`.claude/gate.d/46-write-hook.sh`、`.claude/hooks/refuse-overwrite-untracked.sh`、`.claude/settings.json`、`.claude/gate.d/50-rules-manifest.sh`、`research/scripts/ask-local.sh`、`research/prompts/` 下已有的腿提示与判决（例如 `c143-r3-*`、`m2-code-r1-*`）。

## 要做的

- **F1（J1 事实）**：逐条复核提案第一节的表与三条观测、背景材料正文第二节的每一行——能用命令复核的就复跑（`nice -n 19`），报原样输出与提案是否一致；只能由主 agent 实测、你复核不了的（例如那次派发的 token 用量），写明「复核不了」及原因。再把提案其余各节里每一句可以用一条命令或一次读文件核实的事实挑出来核（例如对上游 `agents/INDEX.md` 的转述、对 56 号门禁形态的转述、对 `ask-local.sh` 行为的转述、对 `stage-mine.py` 的转述）。
- **F2（J2 规则冲突，第二、三节）**：提案第二节六条判据与第三节保留清单的每一行，逐条找它压着的规则原文（附录里有；写规则文件自己的行号），判「一致 / 冲突 / 规则没说」。冲突的把两句并排抄。
- **F3（J2 规则冲突，第一批八个 agent）**：第一批是 `tw-forward`、`tw-attack`、`tw-defense`、`tw-local-attack`、`tw-local-defense`、`tw-verifier`、`gate-triage`、`sweep`。对每一个，把提案给它写的「做什么 / 输入 / 写范围 / 默认模型 / 类型」逐列对到它压着的规则原文：规则要求而定义里没有的、定义里有而规则没要求的、写范围装不下它要做的活的（例如它要跑的脚本会写到范围外）。每个 agent 一张小表。
- **F4（J2，提案第九节还开着的那一问）**：按 `.claude/rules/three-way-inference.md`「三条腿，必须互不重复」原文推：本地攻方与本地辩方「每轮都派」与「每轮按缺的立场挑一个」两种做法，各要改那份规则的哪几句（整行抄）、各自与「不能重复指的是不同的切入角」相不相容。不替用户选，只把两边的规则后果摆清。
- **F5（J2，第七节与第五节「对抗过没有」）**：读 `.claude/gate.d/56-crates-adversarial-review.sh` 全文，写出「每个新增或改过的 `.claude/agents/*.md` 要在同一次改动带来的判决文件里被按路径点名」照它的形态做成会红的阶段时，改动范围、点名、退 77 各怎么定；这道检查挡得住什么、挡不住什么（例如判决里点了名但那一轮根本没攻这个定义）。

每一格各报各的判定；每条结论写「什么现象会推翻它」。

## 交付（照做，不照做这一轮作废）

- 报告写进 `research/prompts/agent-split-r1-sonnet-output.md`：**分段写**，每一次工具调用写进文件的内容不超过 150 行；第一段排他新建（`set -o noclobber` 后 `cat > 文件 <<'EOF'`，文件已存在就停下报告），之后 `>>` 追加。草稿放 `/tmp/claude-1000/agent-split-r1-sonnet/`。
- 引规则、提案、脚本一律写**那份文件自己的行号**，行号去原文件里现查，不从背景材料里数；转述整行抄，不许摘句；不许用「本条」「本节」「上文」这类指代。
- 不许改仓里任何已有文件、不许 git 写操作、不许读别的腿这一轮的产出（`research/prompts/agent-split-r1-*-output*.md`）。
- 不许在任何位置建 agent 定义文件（项目 `.claude/agents/` 与 `~/.claude/agents/` 都会被本机别的会话加载）；要实测的写成「第 0 步该怎么测、两种结果各读到什么」。
- 不许跑 `gate.sh` 全量、不许编译 Rust；跑任何脚本加 `nice -n 19`。
- 请在大约 40 分钟内交出。最后的回复只写一两句话和报告路径。
