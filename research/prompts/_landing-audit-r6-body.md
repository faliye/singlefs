# landing-audit-r6：子 agent 落地审计第九到十一批改动第三轮、最后一轮正文（2026-10-23）

<!-- doc-lint:not-numbers K1 K2 K3 -->

## 一、题面

前两轮判决：`research/prompts/landing-audit-r4-main-verification.md`、`research/prompts/landing-audit-r5-main-verification.md`。第十一批三件（任务 35 到 37，逐件结果在调度记录 `records/2026-10-22-子agent结果落地审计.md` 节「第十一批」）照第二轮判决节「三、判决」第 3 条改了交回闸、派发闸、看门狗与几处定义，交回报告归档在 `research/prompts/subagent-landing-audit-2026-10-22/b11-*.md`。这是第三轮、最后一轮（`.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」第三轮分工：「已定」项撑不撑得住、未定项清单的完整性、那一轮报出的数字能不能核）：第三轮之后不再开轮，这一轮新冒出来的零轮形态写进判决的交用户表。

不重复前两轮已攻过的角度：第一轮（每处改法做没做到九十节那一行、新句子与已有的矛盾、哪一条最脆）、第二轮（第一轮判决够不够得着、第十批的修补本身、方法与两轮都没看过的地方）。

1. 「已定」项撑不撑得住：前两轮判了采、改完了的形态（第九到十一批的改法，连同两份判决里加了限定的结论），今天还站不站得住；第十一批的修补带没带出新的误拦、误放。
2. 未定项清单的完整性：拆分提案 `records/2026-09-16-subagent拆分提案.md` 第九十节 ㉕ 到 ㉝ 与 ⑩，是不是收全了前两轮判决、两轮腿报告、第九到十一批交回报告里点到的「没做」「没查」「留给下一批」「交主 agent」；每一行的形态写没写到能直接派。
3. 报出的数字能不能核：两份判决与调度记录节「第九批收尾」「第十批」「第十一批」引的数（真会话记录上的新拒新放、自证种数、样本用例数、结局账），照写明的命令能不能复出来。

被判的文件：

| 类 | 文件 |
|---|---|
| 第十一批改的 | `.claude/hooks/handback-guard.sh`（⑤ 认命名检查自己的退出码、② 只认仓里的报告）、`.claude/hooks/lib_three_way_legs.py`（`leg_report_mentions`、`registered_trees`、`leg_report_mentions_in_trees`：认本仓 `git worktree list` 列出的每一棵树）、`.claude/hooks/runner-dispatch-guard.sh`（⑳ 读钉住的样本）、新样本目录 `.claude/hooks/fixtures/runner-dispatch-guard/real-prompts/`、`scripts/agents/agent-watch.py`（`progress_file_of`）、`.claude/hooks/lib_prompt_scratch_line.py`（文件头调用方表）、`.claude/hooks/heavy-test-guard.sh`（`MERGE_VERIFY_QUICK_STEP`）、`.claude/main-agent.md`、`.claude/agent-common.md`、`.claude/rules/implementation-workflow.md`（例外那几句） |
| 前两轮站住的形态（照两份判决「这一轮被判过的文件」） | 第九、十批改的定义、钩子、门禁与脚本今天的样子：`.claude/agents/kb-scribe.md`、`.claude/agents/experiment-designer.md`、`.claude/agents/implementation-writer.md`、三份三方定义的 frontmatter、`.claude/hooks/agent-write-scope.tsv`、`.claude/hooks/write-guard.sh`、`.claude/hooks/completion-cleanup-guard.sh`（认草稿目录那一行）、`.claude/gate.d/code-tooling.sh`（kb-directories-write-scope 与 break-switch-replay 两格）、`scripts/gate/merge-and-verify.sh`（checker 档快档那一步）、`scripts/agents/agent-cost.py`、`scripts/agents/subagent_transcript.py` |
| 记录与清单 | `records/2026-09-16-subagent拆分提案.md` 第九十节 ⑩、㉕ 到 ㉝；调度记录节「第九批」「第九批收尾」「第十批」「第十一批」；归档目录 `research/prompts/subagent-landing-audit-2026-10-22/` 下 `b9-*.md`、`b10-*.md`、`b11-*.md` |

## 二、实现今天的样子（主 agent 的观测，2026-10-23）

方案按 `crates/` 与 `scripts/` 今天的实现来谈（主工作区现状）。这一批没有改 `crates/` 下的任何文件；`scripts/gate/merge-and-verify.sh` 的 checker 档快档那一步判的是 `crates/singlefs-checker-tier` 的快档测试，第十一批只改了说它的几句。

- 主 agent 在主工作区重跑过的读数（调度记录节「第十一批」）：交回闸 74 种、派发闸 512 种、`heavy-test-guard.sh` 1402 种（其中该拒 266 种）、`lib_three_way_legs.py`、`lib_prompt_scratch_line.py` 18 格、`dispatch-prompt.py` 92 项、`agent-cost.py` 23 项、`agent-watch.py`，都退 0。
- 第十一批工具员在真会话记录上量的（工具员的数，主 agent 没复算）：交回闸 ⑤ 交回层面新拒 0、新放 3；交回闸 ② 新拒 46 次（仓外副本里写的三方腿报告）、新放 0；派发闸 ⑰ 在 693 份真三方腿派发提示上不变；看门狗取到别人的 progress.md 2026-10-22 起 231 份里 22 → 0、全部 3358 份里 41 → 0。
- 第十一批工具员没收的：命名检查后面接 `| tail`、输出看得出跑完了的（`aa68d151c2fa72bb7` 那一份照旧拒）。
- 任务 35 收严交回闸 ② 时误拒了里程碑五会话在本仓登记的 worktree `~/.local/share/singlefs/work/m5-dev` 里写的三方腿报告（37 次），任务 37 改成认 `git worktree list` 列出的每一棵树；工具员量的：那 37 次改回放行、仓外副本 9 次照拒、新拒 0。主 agent 在主工作区重跑交回闸 78 种、派发闸 512 种、`heavy-test-guard.sh` 1402 种都退 0，门禁 code-tooling 的 break-switch-headers、break-switch-replay 两格绿。
- 别的会话已把 `scripts/experiments/` 全仓改名为 `scripts/experiment-apparatus/`（搬迁登记在 `.claude/gate.d/path-moves.tsv`），被判文件与前两轮证据里的旧路径都已换；只差路径的不算改动（`.claude/rules/path-moves.md`）。
- 交回闸 ② 收严后会拒仓外副本里写报告的三方腿，主 agent 已通知在跑的各会话。
- 别的会话这几天在同几份文件上也有改动：`bash-command-detector.sh` 的 ⑫ 收严、`completion-cleanup-guard.sh` 的运行目录归属判法、`heavy-test-guard.sh` 拒在主工作区直接起会编的 cargo、`scripts/hosts/` 下的编译目录写法、全仓路径改名与它顺带改的 `lib_heavy_tests.py` 目录清单。腿判到它们时写明是别人的改动。
- 暂存区里有一截不是本会话暂存的改动，快照对 HEAD 取 diff。

开工快照：`research/prompts/landing-audit-r6-snapshot/judged-sha256.txt`、`research/prompts/landing-audit-r6-snapshot/judged-vs-head.diff`。

## 三、格

### K1　未定项清单的完整性（辩方）

- 两份判决、四份腿报告（`research/prompts/landing-audit-r4-sonnet-output.md`、`landing-audit-r4-opus-output.md`、`landing-audit-r5-sonnet-output.md`、`landing-audit-r5-opus-output.md`）与第九到十一批交回报告里每一处「没做」「没查」「留给下一批」「交主 agent」：逐条找去向（九十节第几行、调度记录哪一行、判决哪一条），找不到的列出来。
- 九十节 ⑩、㉕ 到 ㉝ 每一行：形态写没写到能直接派（判据、样本、出路），有没有两行说的是同一件事、该并，有没有已经做了却还写「下一批」的；`.claude/kb/checks-owed.md` 的 C879（E165 根选择与杀伤界正式跑没有单元）与 `.claude/gate.d/replay-comparison-units.tsv` 文件头那五行第十批起互指，两处说法今天还对不对得上。

### K2　「已定」项撑不撑得住（攻方）

- 第十一批四处（交回闸 ⑤ 的退出码认法、交回闸 ② 与派发闸 ⑰ 共用的仓内路径认法与任务 37 加的 worktree 认法、看门狗按行首「草稿目录：」取 progress.md）：做没做到第二轮判决那一条，在真会话记录上换一批没被量过的输入还对不对；`| tail` 那一类不收是不是该收。
- 前两轮站住的形态今天还站不站得住：第十一批改了交回闸、派发闸、`lib_three_way_legs.py` 之后，第九、十批立的判法（写回员目标、三方腿类型表、三个报告名前缀、草稿目录一行收严、坏字节记红）有没有被改坏。
- 几处定义里的例外说法，第十一批改成同一句之后，与 `scripts/gate/merge-and-verify.sh` 今天打出的重跑命令逐字对不对得上。

### K3　报出的数能不能核（攻方）

- 两份判决与调度记录节「第九批收尾」「第十批」「第十一批」引的数：照写明的命令在副本上复出来，或指出哪一个复不出来、为什么。重点：第二轮判决的 402 份与 4 份，第十一批工具员的 3、46、693、22、41 与任务 37 的 37、9 几个数（它们由交回报告给出，主 agent 没复算）。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端辩方（Sonnet） | K1 | 复核两份判决与清单：每一条延后有没有去向、形态写没写到能直接派 |
| 云端攻方（Opus） | K2、K3 | 攻已定的形态今天站不站得住，复核报出的数 |
