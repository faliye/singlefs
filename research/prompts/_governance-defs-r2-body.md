# 严查第一轮之后的十份定义改动：第一轮正文（2026-09-26）

<!-- doc-lint:not-numbers G1 G2 G3 G4 G5 -->

## 一、这一轮要判什么

2026-09-26 三个只读审查员对分支 `claude/exciting-bohr-4olowk` 相对 `73ba4a4` 的全部改动做了第一轮严查（报告原样入库：`research/prompts/governance-review-r1-report-A.md`、`-B.md`、`-C.md`，共 36 条），主 agent 逐条现查后改了 10 份定义与共用约束，连同几份钩子、门禁、规则与 skill（逐条列在 `research/prompts/governance-review-r1-sync.md`「命中处置」表）。用户 2026-09-26 选「走一轮三方（缺本地腿）」（门禁 72 号；`.claude/rules/implementation-workflow.md`「改 agent 定义与共用约束，走同一条三步」）。上一轮定义判决是 `research/prompts/governance-defs-r1-main-verification.md`。

要判的是这一批改动本身：**照改后的定义做，有没有一条可达的工作流让 agent 做错或做不下去**——被钩子拒、两份文件说法相反、一步要的东西没人给、改动越出了审查结论（改了行为而不是描述），或者改的时候把原文里一条判据弄丢了。

被判的 10 份：`.claude/main-agent.md`、`.claude/agent-common.md`、`.claude/agents/` 下的 crash-verifier、experiment-designer、experiment-runner、gate-triage、implementation-writer、kb-scribe、mutation-triage、three-way-verifier。改动全文 diff 在附录二（`research/prompts/_governance-defs-r2-diff.md`，基准 `71f0cbc`，同时带着一起改的钩子、门禁、规则与 skill，那几份当背景读）。

| 格 | 改了什么 | 在哪 |
|---|---|---|
| G1 | 整轮门禁改由门禁分诊员跑 `research/scripts/gate-staged.sh`；复用上一次整轮全绿判定只在那一趟里有（判据指到 `research/scripts/stage-must-run.sh`）；层 0 全量的三行命令放进同一次 Bash 调用、判绿之后才往下；54 号出路打印把说明与命令分开 | main-agent.md「暂存之后、提交之前跑门禁」一行与「探索性改 crates」一行；gate-triage.md 第 2 步与写范围；implementation-workflow.md 表；heavy-test-guard.sh 文件头与 POLICY；gate/SKILL.md；54-layer0-replay.sh 的 `print_staged_worktree_full_commands` |
| G2 | 崩溃验证员不再放行层 0（钩子与派发闸）；它跑 55、57、59 前用 `git diff --quiet` 核工作区与暂存区在这几道的输入上相同，不同就停下报；实现员两处「层 0 归崩溃验证员」改掉；crash-test skill「子 agent 不跑」补例外 | crash-verifier.md 第 1 步；implementation-writer.md 第 4 步与「没做什么」；crash-test/SKILL.md；heavy-test-guard.sh 与 runner-dispatch-guard.sh 的放行集合 |
| G3 | 变异计数：`💥` 不判整轮失败，`⚠️` `⏱` `🧱` 判；`mutate.sh` 与 59 号两行「计数：」分开说 | mutation-triage.md 第 5 步；mutation-sampling.md「改了一个格式常量之后」一节 |
| G4 | 核查员三处「分不清」统一标签、补输入「腿开工时刻（UTC）」、「文件不在快照清单里」限定为给了快照的轮、末尾计数加两栏 | three-way-verifier.md |
| G5 | 其余：共用约束「写」一节排他新建与「只许点名的脚本写」的冲突、「门禁」一节重型阶段不按轻阶段那样单跑；执行员 7b ② 的「这一步」、bin 名只对 research 一支用连字符；书记员的标题状态加「待定」、状态列交给生成器；设计员的英文名挪进第 2 步与 `## 一、问题` 第一行；75 号认「两项未定」；20、90 号两句指向改成新规则 | agent-common.md「写」「门禁」；experiment-runner.md 第 2 步与 7b；kb-scribe.md 输入；experiment-designer.md 输入、第 2 步、固定节名表 |

## 二、实现今天的样子（主 agent 的观测，2026-09-26 现查）

- 这一轮不动 `crates/`：`git diff 71f0cbc -- crates | wc -l` → `0`。定义里点到的 `crates/` 路径（`crates/mutations.tsv`、`crates/singlefs-harness/src/bin/e*.rs`）照今天的文件写；入库装置 bin 目录现有 `e156_allocation_basis_counts.rs`、`e158_root_choice_repair.rs`，`crates/singlefs-harness/Cargo.toml` 里 `[[bin]]` 0 处。
- `research/scripts/stage-must-run.sh` 第 51–52 行：没有 `SINGLEFS_STAGED_TREE` 一律要跑；这个变量全仓只有 `research/scripts/gate-staged.sh` 设；`refs/sop/staged-green` 也只有它前移。54、55、59、74 号调 `stage-must-run.sh`；54、55、74 另有一问 `research/scripts/change-touches-crates.sh`（改动没碰登记路径就退 77）。
- `research/scripts/mutate.sh`：`record_verdict` 第 3 个参数是判不判整轮失败，💥 那一处传 0（第 514 行附近），收尾「计数：」只报内存撞顶与超时（第 647 行）；门禁 59 号收尾「计数：」各栏都有（第 407 行）。
- `.claude/hooks/heavy-test-guard.sh --selftest` 562 种通过；`runner-dispatch-guard.sh --selftest` 113 种通过；10、20、62、63、68、73、75、90 号现跑绿。
- 这个容器里本地模型网关不通（`~/code/ai-center` 不存在），本地腿缺席；另外用户级 systemd scope 起不来，`run-with-memory-cap.sh` 退 251，55、59、74 号的判别力样本在这个容器里红（改动前后一样，环境原因）。

## 三、五格各要答什么

### G1 整轮门禁与复用

要答：门禁分诊员照 gate-triage.md 第 2 步跑 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash research/scripts/gate-staged.sh` 会不会被 `heavy-test-guard.sh` 拒；它里面起的 `gate.sh --staged` 与各阶段（54 快档、55、57、59、87）在钩子眼里算谁的；复用判定在改后文字里说得对不对（哪几道调 `stage-must-run.sh`、另一问退 77 算不算「复用」）；主 agent 照 main-agent.md 把 54 号打印的三行放进同一次 Bash 调用，能不能在 `run_in_background` 里跑通、看门狗 `--processes` 盯不盯得到；「全量判绿之后才往下」与崩溃验证员、分诊员的派发次序有没有冲突；main-agent、gate-triage、implementation-workflow、heavy-test-guard 文件头、gate skill 五处说法有没有互相矛盾。

### G2 崩溃验证员

要答：`git diff --quiet -- crates Cargo.toml Cargo.lock litmus research/scripts research/results` 这组路径与 `.claude/gate.d/stage-inputs.tsv` 里 55、59 的登记、57 号实际读的东西对不对得上；多会话共写时这一步会不会让它几乎每次都停；钩子收紧之后崩溃验证员照定义做的每一条命令还放不放行；实现员、crash-test skill 改后的句子与 main-agent 是否一致。

### G3 变异计数

要答：照改后的 mutation-triage、mutation-sampling 数，在 `mutate.sh` 与 59 号真实的输出形态上数得出来吗；「💥 不判失败」对 59 号那一侧也成立吗（59 号是否把 💥 那一栏判红）；两行「计数：」的说法与两份脚本对不对得上。

### G4 核查员

要答：改后的核查员定义里，给了快照、没给快照、快照清单外的文件、报告 sha256 对不上这四种情形各走哪一支，有没有一种落不进任何一支或落进两支；「腿开工时刻」谁给、main-agent 与 implementation-workflow「代码轮派腿之前记一份开工快照」里有没有要求主 agent 记。

### G5 其余

要答：每一处改后的句子与它指向的脚本、钩子、规则、门禁今天的行为是否一致（例：书记员说「N 写阿拉伯数字或汉字数字都认」与 20、75 号的正则；执行员「入库装置 bin 名就是文件名」与 `replay.sh` 登记、写范围表；设计员「重跑登记照抄原登记的英文名」与旧登记里有没有英文名这一行）；删改的原句有没有丢判据（对着附录二逐处看删掉的半句）。

## 四、分工（本地腿缺席）

| 腿 | 立场 | 格 | 要它交什么 |
|---|---|---|---|
| **云端正推（Sonnet）** | 从审查报告的现查结论推 | G1–G5 全部 | 每处改动：它对应审查报告的哪一条（整行抄）；改后的句子是不是那一条的直接后果；推不出、或改动比那一条多做了事的，写出来。另逐处看 diff 里删掉的原文，丢了判据的列出来 |
| **云端攻方（Opus）** | 造工作流历史打改动 | G1、G2、G3 为主，G4、G5 有余力再看 | 每条历史写成可复现的命令序列：钩子的判定用合成的 PreToolUse JSON 喂给 `.claude/hooks/*.sh` 真跑（`AGENT_HOOK_DETECTIONS` 指到草稿目录），脚本的判定用它们自己的 `--selftest` 或合成输入核；要么给出一条照定义做会被拒、做不下去或结果错的历史，要么写「构造不出」并说清卡在哪一步 |
| 本地攻方 / 本地辩方 | — | — | **缺席**：网关不通（第二节最后一条）。按 `.claude/rules/three-way-inference.md`「本地腿缺席时必须显式报告」，这一轮只有两条云端腿，判决里写明 |

两条云端腿的立场不重叠：正推腿核「改的是不是该改的」，攻方腿核「照改后的做会不会出事」。重型测试两条腿都不跑（门禁 54、55、57、59、87、整轮门禁、`gate-staged.sh`），要它们的行为就读脚本、用合成输入喂钩子。

## 五、跑前条款

| 条款 | 什么观测会让它触发 |
|---|---|
| 一格判「站得住」⇒ 那一格的改动照留 | 正推推得出（每处都对得上一条审查结论），攻方在那一格构造不出照做会出事的历史 |
| 一格判「站不住」⇒ 改那一处，写回之后判决标「被攻过零轮」，交第二轮严查接着攻 | 攻方给出一条可复现的历史：照改后的定义做被钩子拒、两处文件给出相反做法、一步要的输入没人给，或正推指出改动越出了审查结论 |
| 删改丢了判据 ⇒ 把判据补回 | 正推或攻方在 diff 里指出一句被删的原文带着一条今天仍然成立的判据，而改后的文字与它指向的文件里都找不到 |

⚠️ 一条历史若让改前、改后两种写法一起出局（打中不分辨候选），不拿它判这一批改动，按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」那一节另记一笔。

## 六、各条腿交什么

- 报告按派发提示给的路径分段写（每次写进文件不超过 150 行，第一段排他新建），回复只写路径、`sha256sum` 与判定一览。
- 引规则、定义、钩子、脚本写那份文件自己的行号，行号去原文件里现查，不从背景材料里数。
- 每条结论写「什么现象会推翻它」。
