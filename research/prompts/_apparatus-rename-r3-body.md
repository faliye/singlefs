# apparatus-rename-r3：收窄生效窗口之后的搬迁换算撑不撑得住、没管到的清单全不全、报出的数能不能核（推论轮第三轮，最终轮，治理轮）正文（2026-10-23）

## 一、题面

第二轮判决 `research/prompts/apparatus-rename-r2-main-verification.md` 判 H1、H2、H3 都打中，根子是「登记行永久生效、能串」；用户在弹窗里定「回到甲路」：收窄登记的生效窗口，再开第三轮（最终轮）攻它（判决「### 用户定案」一节）。甲路由批 6 工具员落地（报告 `~/.local/share/singlefs/work/path-move-window/result.md`）；批 6b 给两份 `frozen_o3.py` 的命令行入口加了弄坏开关守卫、改了三份表头句子（报告 `~/.local/share/singlefs/work/path-move-6b/result.md`）；批 6c 让门禁 doc-experiments 的 decision-links 格比两版正文之前先照此刻生效的登记换算基准（报告 `~/.local/share/singlefs/work/path-move-6c/result.md`）。调度记录 `records/2026-10-23-实验装置目录改名.md`。

这一轮是最终轮：攻前两轮站住的形态与甲路这一形态；这一轮里腿提的新改法算被攻过零轮，不再开第四轮，进判决的交用户表。

被判的改动：开工快照 `research/prompts/apparatus-rename-r3-snapshot/judged-sha256.txt` 列的 15 份，对 HEAD 的 diff 在同目录 `judged-vs-head.diff`（搬了家的按 HEAD 里的旧路径比；diff 里另有别的会话同期的改动，这一批的是第一轮判决第三节「去向」表的甲到辛、第二轮判决交用户表甲路那一行，以及上一段列的批 6b、6c）。

前两轮已经攻过的角度这一轮不重复：登记表加一行就能洗、指历史的写法被换、按次数抵行、倒换认半搬与串起来钉在中间、搬进按路径管的地方、规则与定义句子漏写（第一轮）；在老行前面插一行串到早就在的实现、来回登记、落点被忽略、老行永久生效让没改的表判涨、无序键换次序加中间目录、整个目录换实现、按扩展名挑文件的格、同一块里挪节的行、自证只覆盖打中过的形状、同一函数新旧两版互比当对照（第二轮）。

## 二、实现今天的样子（主 agent 的观测，2026-10-23，行号按主工作区现查）

- `crates/` 下与这套换算无关：`grep -rn 'path-moves\|path_moves\|rewrite_moved\|move_evidence\|effective_moves' crates/` 零命中。
- 生效窗口与实据在 `.claude/gate.d/lib-lag-baseline.py`：`effective_among`（第 327 行）、`effective_moves`（第 346 行）、`exact_rename`（第 409 行）、`baseline_text`（第 437 行）、`baseline_file_bytes`（第 458 行）、`moved_only_paths`（第 626 行）、`drop_moved_lines`（第 732 行）、`untracked_unignored`（第 890 行）、`chained_rows`（第 900 行）、`move_evidence`（第 912 行）、`path_move_report`（第 987 行）；读法写在文件头「搬迁登记」一段。钉值那一侧（`reuse_library.py` 的 `pinned_sha256_now`、两份 `frozen_o3.py` 的 `RegisteredMoves`）照旧按全部登记行倒换。
- `.claude/hooks/lib_long_run.py` 第 476 行 `exempt_main` 取 `lag_baseline.effective_moves(base, root)`；`.claude/gate.d/doc-experiments.sh` 第 1352 行 `baseline_page_text` 调 `baseline_text`，弄坏开关 `EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=moves-not-rewritten`；两份 `frozen_o3.py` 的守卫在 `scripts/experiment-apparatus/e171_three_oracle_collusion/frozen_o3.py` 第 346 行、`scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/frozen_o3.py` 第 322 行。
- 现跑（主工作区，2026-10-23）：`python3 .claude/gate.d/lib-lag-baseline.py --selftest` 32 项都过，弄坏开关 `window-ignored`、`occupied-unchecked`、`chain-allowed`、`untracked-landing-accepted`、`name-change-dropped`、`hunk-any-position`、`governed-lines-dropped`、`move-evidence-unchecked` 各红在文件头点名的那几格；`bash scripts/gate/changed-paths.sh --selftest` 19 项通过；`python3 .claude/hooks/lib_long_run.py exempt-check . --base HEAD` 6 行绿；`python3 .claude/gate.d/lib-lag-baseline.py path-moves-check .`「此刻生效 1 行（第 6 行），不串、有搬迁实据（第 6 行 395 份里 360 份换算之后相同；新前缀下没有基准里本来就在的文件）」；E171 `reuse_library.py --selftest` 48 项红 0；门禁 doc-experiments 的 decision-links 格「没回看 21」（批 6c 之前 140），剩下的 21 页是别的会话的改动。
- 批 6 工具员报告「剩下的、没做的」一节自己列了几条没管到的（同一类里只换目录、文件名不变的单文件搬家照剔；换实现时只差前缀的调用方照剔；从来不生效的登记行登记表不判而钉值那一侧照用）：这是线索，不是结论，判的是代码。
- 前两轮腿的模型目录：`research/prompts/apparatus-rename-r1-opus-model/`、`apparatus-rename-r1-sonnet-model/`、`apparatus-rename-r2-opus-model/`、`apparatus-rename-r2-sonnet-model/`（样本可以拷进草稿目录对着今天的脚本重跑）。

## 三、格

| 格 | 问什么 |
|---|---|
| J1 已定项撑不撑得住 | 甲路的每一条（生效窗口：基准里旧前缀下有文件、新前缀下没有才生效；生效的行不许串；实据：落点逐份在、被跟踪、新前缀原先没有、旧前缀不在磁盘上；改了路径的只在恰好换前缀时剔、单文件登记文件名不变；新增行同一块第 k 条删配第 k 条加；跨管辖类整份搬家不剔、新增行一条不剔），连同前两轮站住的乙（没搬家的脚本重钉照红）、丙（倒换前要求正着换不动）：有没有新的放过或新的误判。尤其「gate 基准」随场合不同（`scripts/gate/changed-paths.sh` 基准回退的几档、`gate.sh --staged` 的临时树、提交之后下一次改动）时窗口开关的时机；登记行与搬迁分在两次提交里；钉值那一侧仍按全部登记行倒换而门禁一侧只认生效行，两边对同一份内容判得一不一致 |
| J2 没管到的清单全不全 | 这套机制自己承认管不到的（规则 `.claude/rules/path-moves.md`「门禁管哪一半」、各文件头、批 6 报告「剩下的」）是不是全了，还有哪一种放过或误判没被任何一处写下；规则、定义（`.claude/agent-common.md`、`.claude/main-agent.md` 长活豁免表那一句）与三份表头的句子跟此刻的代码逐句对不对得上 |
| J3 报出的数能不能核 | 第二节与批 6、6b、6c 报告里的数（32 项、19 项、48 项、6 行、395 份里 360 份、140 → 21、只改切换行 12 → 132 份、119 页）各能不能用一条命令重算出来，算法本身分不分得出对错（自证项与弄坏开关是不是只覆盖了自己造的形状，真登记那一行照过算不算对照） |

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端辩方（Sonnet） | J1–J3 | 复核第二轮判决与它定的甲路：甲路是不是从根上关了第二轮打中的几种（老行前插串、老行让没改的表判涨、无序键），有没有关过头（把真搬迁误判成改动、把该生效的行判成不生效）；替被收紧的那几处辩护（例：只核新加的行、过半门槛、钉值那一侧照用全部登记行） |
| 云端攻方（Opus） | J1–J3 | 找反例：照改完的脚本造一个被放过的真改动或被误判的真搬迁；不重复第一节列的已攻角度；能在草稿目录里拷脚本造样本实测的就实测 |

方案按 `crates/` 与脚本今天的实现来谈。引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
