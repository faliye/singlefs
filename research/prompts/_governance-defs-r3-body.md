# 严查第二轮之后的七份定义改动：第一轮正文（2026-09-26）

<!-- doc-lint:not-numbers G1 G2 G3 G4 G5 -->

## 一、这一轮要判什么

2026-09-26 第二轮严查（三组只读审查员，报告原样入库：`research/prompts/governance-review-r2-report-A.md`、`-B.md`、`-C.md`，共 34 条）之后，主 agent 逐条现查，改了 33 条，甲组 A1 判不成立（处置逐条列在 `research/prompts/governance-review-r2-sync.md`「命中处置」表）。其中改了 7 份定义与共用约束，用户选「再走一轮三方（缺本地腿）」（门禁 72 号）。上一轮定义判决是 `research/prompts/governance-defs-r2-main-verification.md`。

要判的是这一批改动本身：**照改后的定义做，有没有一条可达的工作流让 agent 做错或做不下去**——被钩子拒、两份文件说法相反、一步要的东西没人给、改动越出了审查结论，或者删改时把原文里一条判据弄丢了。

被判的 7 份：`.claude/main-agent.md`、`.claude/agent-common.md`、`.claude/agents/crash-verifier.md`、`.claude/agents/experiment-runner.md`、`.claude/agents/kb-scribe.md`、`.claude/agents/mutation-triage.md`、`.claude/agents/three-way-verifier.md`。改动全文 diff 在附录二（`research/prompts/_governance-defs-r3-diff.md`，`git diff 1c58cfa bfc447e -- .claude/ research/scripts/`，一起改的规则、skill、钩子、门禁与脚本当背景读）。

| 格 | 改了什么 | 在哪 |
|---|---|---|
| G1 | 59 号计数行的读法：没有 💥、⚠️ 两栏；测试进程没跑完的，输出里有一行 `error:` 开头或 `could not compile` 就记「无效」，否则记「没红」；「无效」里尾巴带 `process didn't exit successfully` 的是进程被杀；另外三栏不为 0 也判红 | mutation-triage.md 第 4、5 步；mutation-sampling.md「改了一个格式常量之后」一节 |
| G2 | 崩溃验证员：每道取自己的输入核 `git diff --quiet`，未跟踪文件只查 `crates litmus .lkmm-static-only`；停下之后主 agent 协商；产出表带日志路径、每道输出都写进 `<草稿目录>/<阶段>.log` | crash-verifier.md 第 1 步与产出 |
| G3 | 核查员：设计轮行号对不上时拿腿开工时刻现查，没改过记 ✗；判别力自证挑文件在快照里或腿开工之后没被改过的那条；implementation-workflow 快照一节要求连同派腿时刻交核查员 | three-way-verifier.md 输入与第 1 步；implementation-workflow.md「代码轮派腿之前记一份开工快照」 |
| G4 | 执行员入库装置那一支：编在仓根 `target/`；`replay.sh` 写 `driver_e<号>` 驱动函数加一行 `@driver_e<号>` 登记；research 那一支的 `[[bin]]` name 全写连字符 | experiment-runner.md 第 2、4、5 步 |
| G5 | 其余：书记员标题状态 N 的写法、写入后钩子先红的 21、30 号；共用约束内存包装退出码 253、2 与 `cargo bench`；main-agent「退出码是经内存包装的那条 `--full` 命令的」 | kb-scribe.md 输入与写入后钩子那一段；agent-common.md「不做」一节；main-agent.md「暂存之后、提交之前跑门禁」一行 |

## 二、实现今天的样子（主 agent 的观测，2026-09-26 现查）

- 这一轮不动 `crates/`：`git diff 1c58cfa bfc447e -- crates | wc -l` → `0`。定义里点到的 `crates/` 路径（`crates/mutations.tsv`、`crates/singlefs-harness/src/bin/e*.rs`、入库装置编在仓根 `target/`）照今天的文件写；`research/scripts/replay.sh` 有 `driver_e156`、`driver_e158` 两个入库装置驱动函数，登记行 `E156|@driver_e156||…|exact`。
- `.claude/gate.d/59-crates-mutation-replay.sh` 的 `judge_one`：包装的几种退出码先判；点名测试 FAILED 记抓到；点名测试跑了没红记没红；输出里有 `^error(\[E\d+\])?: ` 或 `could not compile` 记无效；其余记「没跑到」（算没红）。
- `research/scripts/run-with-memory-cap.sh` 文件头退出码表：0–249 命令自己的、250 撞上限、251 起不来、252 排不上、253 超限时、254 被总上限挤掉、2 用法错。
- `.claude/gate.d/57-lkmm.sh`：逻辑在 `.claude/scripts/lkmm.sh`，仓根有 `.lkmm-static-only` 标记时只跑静态几层。
- 这个容器里本地模型网关不通（`~/code/ai-center` 不存在），本地腿缺席；用户级 systemd scope 起不来，`run-with-memory-cap.sh` 退 251。

## 三、五格各要答什么

### G1 59 号计数

要答：改后的读法在 `59-crates-mutation-replay.sh` 的真实分支上逐条对不对（每种结局落哪一栏、哪几栏判红）；「无效」里用尾巴区分进程被杀与编不过，在 cargo 的真实输出上分不分得开；分诊员照第 4 步核「共 N 条」与各栏相加，八栏都算进去了没有。

### G2 崩溃验证员

要答：每道取的输入与 `stage-inputs.tsv`、57 号实际读的东西对不对得上；照定义写的命令（取输入、`git diff --quiet`、`git ls-files`、带前缀跑阶段并把输出写进日志）喂钩子会不会被拒；停下之后 main-agent「一轮怎么开、怎么收」第 9 条接得住吗；日志路径与变异分诊员输入一节「59 号输出路径」对得上吗。

### G3 核查员

要答：给了快照、没给快照、快照清单外的文件、报告 sha256 对不上、没给开工时刻，这五种情形各走哪一支，有没有落空或重叠；判别力自证在设计轮做得出 ✗ 吗；「派腿时刻」由谁、在哪一步记，main-agent 有没有要求。

### G4 执行员入库装置

要答：照改后的第 2、4、5 步做，写范围闸（`.claude/hooks/agent-write-scope.tsv` 里 experiment-runner 那几行）放不放行 `research/scripts/replay.sh` 的驱动函数；`driver_e<号>` 的写法与 E156、E158 的先例对得上吗；「编在仓根 `target/`」与「生成产物与第 5 步都不设 `CARGO_TARGET_DIR`」是否相容。

### G5 其余

要答：每一处改后的句子与它指向的脚本、钩子、门禁今天的行为是否一致；删改的原句有没有丢判据（对着附录二逐处看删掉的半句）。

## 四、分工（本地腿缺席）

| 腿 | 立场 | 格 | 要它交什么 |
|---|---|---|---|
| **云端正推（Sonnet）** | 从审查报告的现查结论推 | G1–G5 全部 | 每处改动：它对应审查报告的哪一条（整行抄）；改后的句子是不是那一条的直接后果；推不出、或改动比那一条多做了事的，写出来。另逐处看 diff 里删掉的原文，丢了判据的列出来 |
| **云端攻方（Opus）** | 造工作流历史打改动 | G1、G2、G3 为主，G4、G5 有余力再看 | 每条历史写成可复现的命令序列：钩子的判定用合成的 PreToolUse JSON 喂给 `.claude/hooks/*.sh` 真跑（`AGENT_HOOK_DETECTIONS` 指到草稿目录），脚本的判定读源码或用合成输入核；要么给出一条照定义做会被拒、做不下去或结果错的历史，要么写「构造不出」并说清卡在哪一步 |
| 本地攻方 / 本地辩方 | — | — | **缺席**：网关不通（第二节最后一条）。按 `.claude/rules/three-way-inference.md`「本地腿缺席时必须显式报告」，这一轮只有两条云端腿，判决里写明 |

两条云端腿的立场不重叠：正推腿核「改的是不是该改的」，攻方腿核「照改后的做会不会出事」。重型测试两条腿都不跑（门禁 54、55、57、59、87、整轮门禁、`gate-staged.sh`）。

## 五、跑前条款

| 条款 | 什么观测会让它触发 |
|---|---|
| 一格判「站得住」⇒ 那一格的改动照留 | 正推推得出（每处都对得上一条审查结论），攻方在那一格构造不出照做会出事的历史 |
| 一格判「站不住」⇒ 改那一处，写回之后判决标「被攻过零轮」，交第三轮确认严查接着攻 | 攻方给出一条可复现的历史：照改后的定义做被钩子拒、两处文件给出相反做法、一步要的输入没人给，或正推指出改动越出了审查结论 |
| 删改丢了判据 ⇒ 把判据补回 | 正推或攻方在 diff 里指出一句被删的原文带着一条今天仍然成立的判据，而改后的文字与它指向的文件里都找不到 |

⚠️ 一条历史若让改前、改后两种写法一起出局（打中不分辨候选），不拿它判这一批改动，按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」那一节另记一笔。

## 六、各条腿交什么

- 报告按派发提示给的路径分段写（每次写进文件不超过 150 行，第一段排他新建），回复只写路径、`sha256sum` 与判定一览。
- 引规则、定义、钩子、脚本写那份文件自己的行号，行号去原文件里现查，不从背景材料里数。
- 每条结论写「什么现象会推翻它」。
