# apparatus-rename-r2：第一轮判决的改法本身、得出结论的方法与没看过的地方（推论轮第二轮，治理轮）正文（2026-10-23）

## 一、题面

第一轮判决 `research/prompts/apparatus-rename-r1-main-verification.md` 判 G1、G3、G4 打中，G2 部分打中，定了改法甲到辛。改法甲到庚由批 4 工具员落地（报告 `~/.local/share/singlefs/work/path-move-verdict-fixes/result.md`），主 agent 收的时候改了甲：工具员落地的是「换算之后相同的过半」，主 agent 不收这个门槛，实据改成只认「旧前缀在基准里有文件、基准里旧前缀下每份都在登记换到的落点上、新前缀在基准里没有文件」，相同的份数只报不判。辛与两行假话主 agent 在判决当场改了。调度记录 `records/2026-10-23-实验装置目录改名.md`。

被判的改动：开工快照 `research/prompts/apparatus-rename-r2-snapshot/judged-sha256.txt` 列的 15 份，对 HEAD 的 diff 在同目录 `judged-vs-head.diff`（diff 里另有别的会话同期的改动；这一批的是第一轮判决第三节「去向」表的甲到庚、辛，以及第一轮正文第一节列的那一批）。

第一轮已经攻过的角度这一轮不重复：登记表加一行就能洗（G1-1、G1-2）、指历史的写法被换（G1-3）、按次数抵行（G1-4）、倒换认半搬与串起来钉在中间（G2-1、G2-2）、搬进按路径管的地方（G3-1）、规则与定义句子漏写（G4）。

## 二、实现今天的样子（主 agent 的观测，2026-10-23，行号按主工作区现查）

- `crates/` 下与这套换算无关：`grep -rn 'path-moves\|path_moves\|rewrite_moved\|move_evidence' crates/` 零命中。
- 实据的判法在 `.claude/gate.d/lib-lag-baseline.py` 的 `move_evidence`，经 `path_move_report` 由门禁 code-tooling 的 change-range-single-source 格判；只核「这一次新加的登记行」（基准那一版登记表里没有这一对的）。真登记那一行现跑：`python3 .claude/gate.d/lib-lag-baseline.py path-moves-check .` 报「新加的 1 行（基准 e1b1ceb95 那一版的登记表里没有）有搬迁实据（第 6 行 395 份里 360 份换算之后相同……）」。
- 现跑（主工作区，2026-10-23）：`lib-lag-baseline.py --selftest` 26 项都过，`LAG_BASELINE_BREAK=move-evidence-unchecked` 下红在「搬迁登记：登记行要有搬迁实据」；`changed-paths.sh --selftest` 19 项通过；`lib_long_run.py exempt-check . --base HEAD` 6 行绿；E171 `reuse_library.py --selftest` 48 项红 0。
- 两条第一轮的腿的模型目录：`research/prompts/apparatus-rename-r1-opus-model/`、`research/prompts/apparatus-rename-r1-sonnet-model/`（样本可以拷进草稿目录对着今天的脚本重跑）。

## 三、格

| 格 | 问什么 |
|---|---|
| H1 修补本身 | 甲（实据只认「在、新前缀原先没有、旧前缀原先有」）、乙（没搬家的脚本重钉照红）、丙（倒换前要求正着换不动，frozen_o3 逐次倒）、丁（按 diff 位置配对剔行）、戊（跨管辖类整份搬家不剔）各自有没有新的放过或新的误判；甲去掉「过半」之后，有没有一种不是搬迁、却满足三条的登记；「只核新加的行」在登记行被改写、删了再加、换了次序时还守不守得住 |
| H2 得出结论的方法 | 第一轮判决与批 4 用来证「改了之后不再放过」的样本，是不是只覆盖了被打中的那几种形状；「真登记那一行照过」「726 份照剔」这类对照是怎么量的，量法本身能不能分辨对错（同一个函数既当判法又当对照，算不算独立） |
| H3 没看过的地方 | 第一轮没碰的读者：`lib-lag-baseline.py` 的九处调用方逐个对着换算之后的基准还判得对不对（取不到基准、基准里还没有这张表、表自己搬了家的几种结局）；`gate_added_lines` 剔行之后依赖它的格；`.claude/rules/path-moves.md` 开头「不留一处旧路径」与第 5 步；跨管辖类搬家时 `drop_moved_lines` 照旧剔行（批 4 报告点名，判决没点） |

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端辩方（Sonnet） | H1–H3 | 复核第一轮判决：它判打中的够不够得着（打中的那几格是不是同样打中所有替代方案），它定的改法是不是收严、没有放宽；替第一轮被判出局的写法辩护（例：「过半」门槛、只核新加的行） |
| 云端攻方（Opus） | H1–H3 | 找反例：照改完的脚本造一个被放过的真改动或被误判的真搬迁；不重复第一节列的已攻角度；能在草稿目录里拷脚本造样本实测的就实测 |

方案按 `crates/` 与脚本今天的实现来谈。引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
