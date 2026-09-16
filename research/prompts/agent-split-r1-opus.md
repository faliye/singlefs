# subagent 拆分执行计划 正反对抗第一轮：攻方腿（Opus）

你是 `records/2026-09-16-subagent拆分提案.md` 正反对抗第一轮的**攻方腿**。立场：假设这份执行计划照着做会出事，去找能打穿它的具体输入与操作序列。你分到的判据格是背景材料第三节的 J3、J4、J6（第五节「腿的分工」第二行）。J1、J2、J5 与「自举 / 独立性」「被排除方案的辩护」归别的腿，不要花时间在那几格上。

## 先读

1. 背景材料：`research/prompts/_agent-split-r1-background.md`（正文 + 小节清单 + 附录；提案全文在附录第一段）。正文第三节是判据、第四节是跑前条款、第六节是交付规矩，照做。
2. 需要时自己去读：`.claude/hooks/refuse-overwrite-untracked.sh` 与 `.claude/settings.json`（今天唯一的 PreToolUse hook 怎么从 stdin 读工具参数）、`.claude/gate.d/46-write-hook.sh`、`.claude/gate.d/56-crates-adversarial-review.sh`、`.claude/scripts/gate.sh`（阶段怎么跑、`--staged`）、`research/scripts/stage-mine.py`、`research/scripts/check-staged.sh`、`research/scripts/relabel-item.py`、`research/scripts/replace-batch.py`、`.claude/gate.d/49-history-brief.sh`、`.claude/gate.d/21-decision-items-sync.sh`。

## 要攻的

- **A1（J4，写范围三层闸）**：提案第五节的三层——`tools` / `disallowedTools`、agent 上挂的 `agent-write-scope.sh`（只管 Write / Edit）、派发前后比 `git status --porcelain` 与未跟踪文件。对每一层给出：一个越界写而它不红的具体工具调用或命令（写明工具名与参数，例如别的会写文件的工具、Bash 里调一个会改全仓的脚本、符号链接、`../` 相对路径、仓外路径、被 `.gitignore` 挡住的路径）；一个没越界而它红的输入（例如别的会话同一段时间里的写、构建产物、生成文件）。最后判三层叠起来还剩哪条缝。
- **A2（J4，新门禁七条判据）**：第五节那张七行表，逐行造一份 `.claude/agents/*.md` 的内容：该红不红的一份、不该红却红的一份。「不抄规则」那一行的阈值没定，写出阈值取不同值时各自误判什么。「对抗过没有」那一行，造一次改动：判决文件点了路径、而那一轮根本没攻这个定义。
- **A3（J3，操作序列）**：照提案第六节的调度与第四节的定义，写出会导致下面任一结局的序列，每一步指到提案里许可它的那一句：① 某条腿读到了别的腿这一轮的产出或会被产出影响的文件（留意 `tw-verifier` 要读各腿报告、本地腿的提示与输出放在 `research/prompts/`、草稿目录、主 agent 拷报告进 `research/prompts/` 的时点）；② 三方证据被污染（提示文件与实际发给腿的内容对不上、作废副本被当合法样本）；③ 判决实际由 subagent 做出（例如 `tw-verifier` 或 `gate-triage` 的报告措辞让主 agent 只剩照抄）；④ `gate-triage` 把这一轮的红判成「别的会话的」或反过来；⑤ `sweep` 的「同一个数不同的量」分类漏掉一处该改的。
- **A4（J3，限额与资源）**：自定义 agent 撞周限额后用 SendMessage 续做、撞输出上限重派——各写一段会丢证据或重复写的序列；资源锁（`flock`）在三条腿并行、`gate-triage` 后台跑全量门禁、另一个会话在跑性能测量时，各会卡在哪、会不会死锁或饿死。
- **A5（J6，代价数）**：用背景材料正文第二节那个实测数（一次零工具调用的派发 137,814 token，只测了 general-purpose）作每次派发的下限，算：一轮正反对抗照提案要派几次、至少多少 token；「每落地一个 agent 走一次对抗、三轮多数判」时，第一批八个与全部十五个各要几轮、几次派发、至少多少 token。写明公式与每个乘数从哪来；哪个取舍依赖这个数、提案没给。

每一格打中之后先答四句（`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」）；每条结论写「什么现象会推翻它」；你自己提的改法写明「只在你的推导里成立、被攻过零轮」。没打中的写试过哪些形状。

## 交付（照做，不照做这一轮作废）

- 报告写进 `research/prompts/agent-split-r1-opus-output.md`：**分段写**，每一次工具调用写进文件的内容不超过 150 行；第一段排他新建（`set -o noclobber` 后 `cat > 文件 <<'EOF'`，文件已存在就停下报告），之后 `>>` 追加。要写脚本或样本验证的，放 `research/prompts/agent-split-r1-opus-model/`（同样排他新建），报告里给复跑命令与文件 sha256。草稿放 `/tmp/claude-1000/agent-split-r1-opus/`。
- 引规则、提案、脚本一律写**那份文件自己的行号**，行号去原文件里现查，不从背景材料里数；转述整行抄，不许摘句；不许用「本条」「本节」「上文」这类指代。
- 不许改仓里任何已有文件、不许 git 写操作、不许读别的腿这一轮的产出（`research/prompts/agent-split-r1-*-output*.md`）。
- 不许在任何位置建 agent 定义文件，也不许改 `.claude/settings.json` 或注册 hook（项目与用户级配置都会被本机别的会话加载）；要验 hook 行为的，把 hook 脚本拷到你的模型目录，用构造的 stdin JSON 直接喂它。
- 不许跑 `gate.sh` 全量、不许编译 Rust；跑任何脚本加 `nice -n 19`。
- 请在大约 45 分钟内交出（序列与输入优先，代价数其次）。最后的回复只写一两句话和报告路径。
