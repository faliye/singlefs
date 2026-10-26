# landing-audit-r1 判决：子 agent 落地审计这一批改动第一轮（主 agent，2026-10-22）

<!-- doc-lint:not-numbers K1 K2 K3 K4 -->

正文 `research/prompts/_landing-audit-r1-body.md`；正推腿 `research/prompts/landing-audit-r1-sonnet-output.md`（sha256 37e65d74…）；攻方腿 `research/prompts/landing-audit-r1-opus-output.md`（sha256 f9753faf…），模型 `research/prompts/landing-audit-r1-opus-model/`。

## 这一轮被判过的文件

`.claude/agent-common.md`、`.claude/main-agent.md`、`.claude/agents/experiment-runner.md`、`.claude/agents/experiment-designer.md`、`.claude/agents/sweep.md`、`.claude/agents/kb-scribe.md`、`.claude/agents/implementation-writer.md`、`.claude/hooks/runner-dispatch-guard.sh`、`.claude/hooks/lib_heavy_tests.py`、`.claude/hooks/heavy-test-guard.sh`、`.claude/hooks/write-guard.sh`、`.claude/hooks/handback-guard.sh`、`.claude/hooks/agent-write-scope.tsv`、`.claude/hooks/selftest-registry.tsv`、`scripts/agents/agent-cost.py`、`scripts/agents/dispatch-prompt.py`、`scripts/agents/agent-watch.py`、`scripts/agents/cite-check.py`、`scripts/agents/README.md`、`scripts/gate/apply-writer-patch.py`、`scripts/kb/claim-experiment.sh`、`scripts/kb/stale-candidates.py`、`.claude/gate.d/doc-experiments.sh`、`.claude/gate.d/replay-comparison-units.tsv`、`.claude/gate.d/replay-comparison-units-lag.tsv`、`.claude/gate.d/stage-owners.tsv`

## 一、核查

- `python3 scripts/agents/cite-check.py <两份腿报告> --root . --background research/prompts/_landing-audit-r1-background.md`：「✓ 核了 79 处引文、84 处裸行号，对上 79 处，没判 84 处（其中裸行号没有引文可比 84 处；逐处列在上面）」。
- 开工快照 `research/prompts/landing-audit-r1-snapshot/judged-sha256.txt` `sha256sum -c`：七份对不上。六份是 `.claude/kb/checks-owed{,.en,.ja}.md` 与 `.claude/kb/milestone/03-third-txn{,.en,.ja}.md`，被里程碑三收尾那个会话的第七批写回改过（规格 `~/.local/share/singlefs/work/m3-close-b16-text-batch-7/spec.json`）；第七份是 `.claude/gate.d/stage-owners.tsv`，别的会话加了 16 行。腿引到这几份的四处（`stage-owners.tsv:106`、`checks-owed.md:560`、`:561`、`03-third-txn.md:185`）主 agent 现查，指的仍是这一批写的那一行（C879、C880、新格的归属行、「待优化」第 3 行），只是行号随别人删行挪了一行；腿引的行都不落在别的会话改的那几处。
- 主 agent 现查（裸行号逐处读原文）：`.claude/agents/tooling-writer.md:36`「交回前不跑门禁阶段与各类 lint」、`.claude/agents/experiment-runner.md:45`「门禁阶段交回前不跑」、`.claude/agent-common.md:3`「与定义冲突时以定义为准」；`.claude/main-agent.md:86`「文件名一律写 `result.md`」；`.claude/hooks/handback-guard.sh:119`「交回里没点名报告文件（research/prompts/<轮>-…-output.md）」；`.claude/kb-directories` 第 2 行登记 `crates/singlefs-checker/invariants`；`.claude/gate.d/stage-owners.tsv:106`「登记行由写重跑登记的设计员与执行员加」；`.claude/agents/experiment-designer.md:61` 设计员只写跑前登记与重跑登记；执行员定义「写范围」一节没有 `replay-comparison-units.tsv`（`grep -c` 为 0）；`.claude/agents/experiment-designer.md:33` 第 3a 步原文；`.claude/agents/sweep.md:52`「交实现员」一节原文；调度记录第 42 行原文。
- 主 agent 复跑：
  - 派发闸：照 `dispatch-prompt.py clerk` 骨架填好的提示 `--check` 退 0；「报告：…/report.md」退 3；「报告路径：…/report.md」退 0。攻方 K2-4 属实。
  - 单元表 19 个单元、登记表 13 个不同单元名，缺「r3 合法」「r5 合法」「r5 对抗」「r5 多盘」「奇数版本的 r5 合法」「奇数版本的 r5 对抗」。正推第 10 条属实。
  - `gate-overlap.py .`（判这一次改动）：报 8 段整段相同，全在 `.claude/hooks/lib_two_logs_guard.py` 与 `.claude/hooks/lib_run_scope.py` 之间，两份都不在这一批被判之列；这一批的文件 0 段，与攻方自己的核对一致。输出 `research/prompts/landing-audit-r1-snapshot/gate-overlap-judge.log`。
  - `gate-overlap.py --touched-by` 对第三批四个工具员的会话记录各跑一次：四份都退 77「这个会话没有新加或改动门禁与钩子」。它只认 Write、Edit 与 Bash 里几种直接写的形态，认不出 `scripts/gate/hook-swap.py` 与 `.new` 加 `mv` 这两种项目规定的换文件写法。输出 `research/prompts/landing-audit-r1-snapshot/gate-overlap-touched.txt`。
- 攻方 K3-4 的最小副本实测（四格在不带 `.git`、不带词表的副本上判不到）主 agent 没复跑；采它，依据是 `.claude/gate.d/code-source-discipline.sh` 第 294 行读 `<根>/.claude/naming-vague-words`、第 316 行用 `git ls-files` 列文件，两样在 `.claude/agent-common.md` 建副本那一条只拷的目录里都没有。

## 二、逐格判

### K1（正推）

八条缺口里做到的：K4 文字、K3 文字、K9、延后项文字、回扫 `crates/` 行文字、载体搬家的扫描与写范围两侧、报告文件名、`apply-writer-patch.py`、两日志一句。正推判「做了一半」的四条与主 agent 的判定：

| 正推那一条 | 判定 | 去向 |
|---|---|---|
| 3 命名检查：挡不住「说自扫过、没真跑」 | 采：没有检查核报告里有没有两条命令的输出，九十节没登 | 九十节加一条 |
| 7 写回员「找不到的目标」只有文字 | 采 | 九十节加一条 |
| 10 比对登记只读手填的 14 行，6 个单元没有行，没有定义要求登记 | 采；另见 K3-6 | 这一批改 |
| 13 返工词表单字「修」把「修订」算进去 | 采（攻方 K4-2 同） | 这一批改 |
| 8 ⑰ 判名字里任意位置带 report，比实测被拒的宽 | 不采：宽是向安全的一边，名字里带 report 的别的文件 16 次里拒 6 次 | 不改 |

### K2（攻方：合并与重合）

| 攻方那一条 | 判定 | 去向 |
|---|---|---|
| K2-1 「滞后表与基准比」这段判法全仓 7 处，新格那段与 multipath-registry 那段只差变量名 | 采。这一批只并同一道门禁 doc-experiments.sh 里的两处（新格与 multipath-registry），抽成 `.claude/gate.d/lib-lag-baseline.py`；doc-process-records、code-source-discipline、code-tooling、`lib_long_run.py` 那五处不在这一批的文件里，交用户定要不要现在一起并 | 两处这一批改；五处交用户 |
| K2-2 新格的滞后表就是登记表 none: 行的复本 | 采：删 `replay-comparison-units-lag.tsv`，「只缩不涨」改成拿登记表里 none: 的行与基准比；相应的四样判法与开关 `comparison-none-lag-unchecked` 删，`comparison-lag-growth-unchecked` 改成不比登记表的基准 | 这一批改 |
| K2-3 `replay.sh` 复跑表在 doc-experiments.sh 里被解析第四遍，取法不同 | 采：一个 `replay_rows(path)` 放进同一份库，同一道门禁四处都调它（`scripts/gate/admission.py` 那一份不在这一批） | 这一批改 |
| K2-4 派发闸 ④ 与 ⑰ 各认一半报告标签 | 采：一个报告行解析，认「报告」「报告路径」「报告写进」「报告写到」，路径同时交 ④（写范围）与 ⑰（文件名）；⑰ 那一格样本改成「报告路径：…/report.md」拒 | 这一批改 |
| K2-5 候选扫描写死不变量清单路径，旁边已有登记位 `.claude/kb-directories` | 采：`CARRIER_PATTERNS` 里 `.claude/kb/` 与不变量清单两行改成读登记位；写范围一侧「登记位里每个目录在写回员写范围里有一行」的门禁格是新立的检查 | 读登记位这一批改；新格进九十节 |
| K2-6 结局账「交回」只认调用、续做闸与看门狗要送达 | 采：`scripts/agents/` 下抽一份子 agent 会话记录的库（`handback_delivered`、按首条记录取起点），`agent-cost.py`、`continuation-guard.sh`、`agent-watch.py` 都从它取 | 这一批改 |
| K2-7、K2-8 | 不采：攻方自己判值不大；K2-8 里「实验号住在哪」两份目录表不一致那一处进九十节 | 九十节加一条 |

### K3（攻方：矛盾）

| 攻方那一条 | 判定 | 去向 |
|---|---|---|
| K3-1 三方正推腿骨架默认报告 `<草稿目录>/result.md`，派发闸放行、交回闸拒 | 采，这一批带出来的。每个类型的报告路径形态写进定义 frontmatter，骨架照它填默认，三方腿填 `research/prompts/<填 轮>-<模型>-output.md` 占位；攻方、辩方骨架补「报告：」一行 | 这一批改 |
| K3-2 `.claude/main-agent.md:86`「一律 result.md」没给三方腿留例外 | 采：改成「名字里不带 report；没有别的形态要求的写 `<草稿目录>/result.md`，三方腿照 `.claude/rules/three-way-inference.md` 写 `research/prompts/<轮>-*-output.md`」 | 这一批改 |
| K3-3 命名检查与执行员、工具员定义「交回前不跑门禁阶段」相反，定义优先 | 采：两份定义那一步各加「命名检查除外（共用约束「门禁」一节）」，执行员固定的「没做什么」一行同改 | 这一批改 |
| K3-4 最小副本上四格判不到 | 采：共用约束那一条改成在主工作区或 git worktree 副本上跑；在 rsync 副本里改的，两条命令写进「要主 agent 跑的命令」，由主 agent 在补丁打进主工作区之后跑 | 这一批改 |
| K3-5 「延后的当时就登记」只列三处，漏了拆分提案 | 采：第 2、7 条的登记位加第四处「子 agent 与协作工具的事记进 `records/2026-09-16-subagent拆分提案.md` 的编号小节」 | 这一批改 |
| K3-6 比对登记表的写者对不上 | 采：登记行归执行员（它实现单元）。执行员第 5 步登记复跑那一步加一句、写范围一段补这份表；`stage-owners.tsv:106` 的归属改成执行员；补上缺的 6 个单元，或逐个写明为什么不是比对对象 | 这一批改 |
| K3-7 设计员 3a 要先写「预期的差」与「会发生的结论」，与「答案不许已经存在」冲突 | 采：② 改成每个判据要分辨的差取岔路单那一行的翻面观测，写测量手段分不分得出它；③ 只对触发是前提、不是被问的量的事件做，算出的结论列进「跑之前已经存在的数」那一节 | 这一批改 |
| K3-8（弱）执行员 3c 的证红与入库装置那一支不许跑 checker 档 | 采：3c 加一句，入库装置那一支的证红写进「要主 agent 跑的命令」，清单那一格写「待主 agent 证红」 | 这一批改 |
| K3-9（弱）「交实现员」并进「下一批」与截止批次、条款已定对不齐 | 采：回扫员定义与主 agent 派发表那一行写「进下一轮的清单」，钉值类的行先定条款再派 | 这一批改 |

### K4（攻方：数）

| 攻方那一条 | 判定 | 去向 |
|---|---|---|
| K4-1 结局账的数原样复不出：命令只有下界，产物没落盘 | 采：`outcome` 加 `--until`；这一次的 `--summary` 输出存进 `research/prompts/subagent-landing-audit-2026-10-22/`，调度记录点名它 | 这一批改 |
| K4-2 调度记录第 42 行 337 / 13.8% 还写着五个词；严格词表单字「修」 | 采：第 42 行加注指到更正那一条；词表改成不认「修订」 | 这一批改 |
| K4-3 「另多 3 次」里两次是同一行换了缺口 | 采：调度记录那一句改成「4/4 复现；盲判在 E188 另归出 3 次，其中两次 rework.md 归给另一条缺口」 | 这一批改 |
| K4-4 8/54、16/129 输入目录已删，复不出 | 采为限度：输入随草稿目录删了，记进调度记录；结论只引这两个数的算术，不再当可复跑的数 | 记录 |

## 三、判决：要改的（交下一批 tooling-writer）

一、文件不相交，分三份同时派：

| 份 | 关的条 | 文件 |
|---|---|---|
| 门禁与扫描 | K2-1（doc-experiments.sh 内两处）、K2-2、K2-3、K2-5（读登记位）、K3-6 的登记表与归属行、正推 10 | `.claude/gate.d/doc-experiments.sh` 与 fixtures、`.claude/gate.d/lib-lag-baseline.py`（新）、`.claude/gate.d/replay-comparison-units.tsv`、删 `.claude/gate.d/replay-comparison-units-lag.tsv`、`.claude/gate.d/stage-owners.tsv`、`scripts/kb/stale-candidates.py` |
| 定义文字 | K3-2、K3-3、K3-4、K3-5、K3-6 的定义一侧、K3-7、K3-8、K3-9 | `.claude/agent-common.md`、`.claude/main-agent.md`、`.claude/agents/experiment-runner.md`、`.claude/agents/experiment-designer.md`、`.claude/agents/sweep.md`、`.claude/agents/tooling-writer.md`、`.claude/agents/three-way-forward.md`、`.claude/agents/three-way-attack.md`、`.claude/agents/three-way-defense.md` |
| 钩子与脚本 | K2-4、K2-6、K3-1、K3-6 的写范围表、K4-1、K4-2 | `.claude/hooks/runner-dispatch-guard.sh`、`.claude/hooks/continuation-guard.sh`、`.claude/hooks/agent-write-scope.tsv`、`scripts/agents/dispatch-prompt.py`、`scripts/agents/agent-cost.py`、`scripts/agents/agent-watch.py`、`scripts/agents/` 下新的会话记录库 |

二、主 agent 自己改：调度记录第 42 行的注、「另多 3 次」那一句、K4-4 的限度一行、`--summary` 输出存档。

三、九十节加的：正推 3（命名检查「说跑了没真跑」）、正推 7（写回员找不到的目标）、K2-5 写范围那一侧的格、K2-8 实验号目录表不一致、`gate-overlap.py --touched-by` 认不出 `hook-swap.py` 与 `.new` 加 `mv`（上游脚本）。

改完开第二轮，攻第一轮的修补本身与产生结论的方法（`.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」）。

## 四、交用户

- K2-1 那五处（doc-process-records、code-source-discipline 两处、code-tooling 两处、`lib_long_run.py`）不在这一批的文件里，现在一起并进 `lib-lag-baseline.py`，还是这一批只并 doc-experiments.sh 内的两处、五处记进九十节。用户 2026-10-22 定「七处一起并」：五处并进「三、判决」第一张表「门禁与扫描」那一份，文件加 `.claude/gate.d/doc-process-records.sh`、`.claude/gate.d/code-source-discipline.sh`、`.claude/gate.d/code-tooling.sh`、`.claude/hooks/lib_long_run.py`。

## 五、没核动的

- 攻方 K3-4 的最小副本输出主 agent 没复跑，依据见「一、核查」最后一条。
- 攻方 K4-1 的复算表主 agent 没复跑；采的是「命令只有下界、产物没落盘」，主 agent 现查过记录第 138 行没有点名产物文件。
- 正推第 8 条「被拒的名字全部以 report 起头」只来自它扫通的那一次会话记录，主 agent 的数据是「名字里带 report 的别的文件 16 次里拒 6 次」，没逐个看那 6 个名字。

## 回看决策

不涉及决策：被判的都是 agent 定义、共用约束、钩子、门禁与协作脚本的工作流写法，加上欠账表两行与里程碑三「待优化」表一句现状；不碰 `.claude/kb/decisions/` 的任何分项。
