# 治理文档核对第 3 轮的十五份定义改动：第一轮正文（2026-09-26）

<!-- doc-lint:not-numbers G1 G2 G3 G4 G5 G6 -->

## 一、这一轮要判什么

2026-09-26 五个只读核对员逐份核了 29 份治理文档（报告 `research/prompts/governance-rot-r2-audit-A.md` … `-E.md`，共 97 条），主 agent 逐条现查后改了 15 份定义与共用约束，其中 12 项写法由用户在三个弹窗里定（逐条列在 `research/prompts/governance-rot-r3-sync.md` 开头一段「这一阶段做成的事」）。用户 2026-09-26 选「走三方（缺本地腿）」（`.claude/rules/implementation-workflow.md`「改 agent 定义与共用约束，走同一条三步」、门禁 72 号）。

要判的是这一批改动本身：**照改后的定义做，有没有一条可达的工作流让 agent 做错或做不下去**——被钩子拒、两份文件说法相反、一步要的东西没人给、改动越出了核对结论与用户定案（改了行为而不是描述），或者改的时候把原文里一条判据弄丢了。

被判的 15 份：`.claude/main-agent.md`、`.claude/agent-common.md`、`.claude/agents/` 下的 crash-verifier、experiment-designer、experiment-runner、gate-triage、implementation-writer、kb-scribe、mutation-triage、sweep、three-way-attack、three-way-local-attack、three-way-local-defense、three-way-materials、three-way-verifier。改动的全文 diff 在附录二（`research/prompts/_governance-defs-r1-diff.md`）。同一批一起提交、不归这一轮判的规则与脚本改动在提交 `802fcc1`，腿可以读它们当背景。

| 格 | 改了什么 | 在哪 |
|---|---|---|
| G1 | 层 0 全量归主 agent：主 agent 建 HEAD + 暂存区的 worktree、经内存包装后台跑 `--full`；崩溃验证员只跑 54 号快档与 55、57、59；门禁分诊员不再单跑登记给它的阶段；57 号写明没有复用 | main-agent.md「暂存之后、提交之前跑门禁」一行；crash-verifier.md 第 2 步；gate-triage.md 第 2 步、写范围 |
| G2 | 子 agent 跑 cargo test / run 一律经 `research/scripts/run-with-memory-cap.sh`（派发提示没给上限用 4G）；重型测试清单改成指向 implementation-workflow.md；15、74 号按轻阶段；攻方腿明写可以在副本里编译；实现员的 lint 写成命令、不跑 check.sh、跑登记给自己的阶段、删交回前 `git apply --check` | agent-common.md「不做」一节；implementation-writer.md 第 3、4 步；three-way-attack.md 写范围 |
| G3 | 本地腿用 `run_in_background` 跑 ask-local.sh；判红重跑用 `>|` 重用同一个号；退出码 6（检测器没跑成）停下报缺席；oov-check 带提示文件、生词表截到 300 字符时通读；写范围补 runlog；本地辩方补读攻方的「输入」一节 | three-way-local-attack.md 第 3–5 步、写范围；three-way-local-defense.md |
| G4 | 核查员的快照是 sha256 清单，按 `sha256sum -c` 与倒推副本核；主 agent 定本地腿派哪条并写理由；云端腿报「没打中」要支撑结论时主 agent 再派一条同立场腿 | three-way-verifier.md 输入与第 2 步；main-agent.md「要推论」一行 |
| G5 | 变异计数：mutate.sh 没有三数汇总行，照每条标记数；59 号照「计数：」行；超时、内存撞顶另列；入库装置不写 research 变异表、变异行照 crates/mutations.tsv 六段格式、三个数待 59 号；源文件与变异表用英文名；设计员排除实验索引与实验变更史文件 | mutation-triage.md；experiment-runner.md 第 2、3 步与报告；experiment-designer.md 输入与第 2 步 |
| G6 | 其余描述修正：钩子拒绝的写法补到七种；`&` 配逐个 `wait "$pid"`；草稿里改已存在脚本照 ⑦ 换 inode；只有 Edit 的新建照排他写；`progress.md`；kb-scribe 的写入后钩子、relabel 改 crates、预检检出指到草稿、标题行计数；gate-triage 的 ref 名；sweep 组号用 ASCII 连字符；材料员补 quote-rust-items.py；main-agent 的看门狗告警、「那张表」、sweep 那一行的用词 | 各文件，逐条见 `research/prompts/governance-rot-r3-sync.md`「命中处置」表 |

## 二、实现今天的样子（主 agent 的观测，2026-09-26 现查）

- 这一轮不动 `crates/`：`git diff HEAD -- crates | wc -l` → `0`。定义里点到的 `crates/` 路径（`crates/mutations.tsv` 第 1 行的六段表头、`crates/singlefs-harness/src/bin/e*.rs` 的入库装置）照今天的文件写。
- 钩子：`.claude/hooks/heavy-test-guard.sh` 文件头「重型测试，按类」一段与第 74–75 行（子 agent 的 cargo test / run 不经 `run-with-memory-cap.sh` 就拒）；`.claude/hooks/bash-command-detector.sh` 文件头「拒绝七种」① 至 ⑦；写范围表 `.claude/hooks/agent-write-scope.tsv`。
- `research/scripts/ask-local.sh`：检测器退 2 或找不到时退 6、不打正文、留作废副本（提交 `802fcc1`，自证 `research/scripts/ask-local-selftest.sh` 覆盖这两支）。
- `research/scripts/mutate.sh` 收尾只打「计数：内存撞顶 … 超时 …」一行，每条变异打 `✅ [抓到]` / `⏭  [无效]` / `❌ [没红]` 标记；门禁 59 号收尾打「计数：抓到 … 没红 … 无效 … 内存撞顶 … 超时 … 被总上限挤掉 … 排不上没跑 … scope 起不来没跑 …（共 N 条）」。
- `.claude/gate.d/stage-inputs.tsv` 登记了 54、55、59、74、87，没有 57。
- 这个容器里本地模型网关不通（`~/code/ai-center` 不存在，`curl localhost:8200` 退出码 7），本地腿缺席；用户知情后选择在两条云端腿上走这一轮。

## 三、六格各要答什么

### G1 层 0 全量归主 agent

要答：主 agent 照 main-agent.md 那一行写出的命令（带 `SINGLEFS_HEAVY_TESTS=commit`、经 `run-with-memory-cap.sh 16G`、`bash <根>/.claude/gate.d/54-layer0-replay.sh --full <根>`）会不会被 `heavy-test-guard.sh` 拒或放错类；崩溃验证员、门禁分诊员照改后的定义做，54 号快档核标记时标记从哪来、这批输入没有标记时谁在哪一步补；三份文件（main-agent、crash-verifier、gate-triage）与 implementation-workflow.md、54 号文件头对同一件事的说法还有没有不一致。

### G2 内存包装与重型清单

要答：子 agent 照 agent-common 的写法（`bash research/scripts/run-with-memory-cap.sh 4G cargo test -p … --test …`）在钩子里是不是放行；攻方腿、实现员在自己的仓副本里编译跑时，这条要求会不会让它们做不下去（副本里有没有 `research/scripts/run-with-memory-cap.sh`、systemd 用户级 scope 起不来时退 251 怎么办）；「15、74 号按轻阶段」与钩子对 15 号正文里 `cargo test --release` 的判法是否矛盾。

### G3 本地腿

要答：`run_in_background` 起 ask-local.sh 时 `>` / `>|` 重定向、退出码取法与 `bash-command-detector.sh` ④（后台里单独的 `&`）、`agent-common.md`「长活可以等」是否相容；`noclobber` 下 `>|` 重用号会不会覆盖一份不是空的样本；退出码 6 与「非 0 非 5 停下报缺席」对得上吗。

### G4 核查员与三方调度

要答：核查员按 `sha256sum -c` 核、对不上改用倒推副本，这两样输入在 main-agent.md 与 implementation-workflow.md「代码轮派腿之前记一份开工快照」里有没有人给；「本地腿派哪条由主 agent 定」「云端再抽由主 agent 再派」写进 main-agent 的那一句与 three-way-inference.md 对应段落是否逐字同义。

### G5 变异计数

要答：照改后的 mutation-triage、experiment-runner 数「三个数」，在 mutate.sh 与 59 号的真实输出上数得出来吗；入库装置那一支的新写法与实验执行员的写范围（`.claude/hooks/agent-write-scope.tsv` 里 experiment-runner 那几行）、门禁 33 号对 `crates/mutations.tsv` 的判法是否相容；英文名的规定与 `research/scripts/claim-experiment.sh`、`replay.sh` 登记行是否相容。

### G6 其余描述修正

要答：每一处改后的句子与它指向的脚本、钩子、规则今天的行为是否一致；有没有哪一处在删改原句时丢了一条判据（对着附录二的 diff 逐处看删掉的半句）。

## 四、分工（本地腿缺席）

| 腿 | 立场 | 格 | 要它交什么 |
|---|---|---|---|
| **云端正推（Sonnet）** | 从核对报告的现查结论与用户定案推 | G1–G6 全部 | 每处改动：它对应核对报告的哪一条、用户定案的哪一项（整行抄）；改后的句子是不是那一条的直接后果；推不出、或改动比那一条多做了事的，写出来。另逐处看 diff 里删掉的原文，丢了判据的列出来 |
| **云端攻方（Opus）** | 造工作流历史打改动 | G1、G2、G3、G5 为主，G4、G6 有余力再看 | 每条历史写成可复现的命令序列：钩子的判定用合成的 PreToolUse JSON 喂给 `.claude/hooks/*.sh` 真跑（照 kb-scribe.md 第 1 步写范围预检那种写法，`AGENT_HOOK_DETECTIONS` 指到草稿目录），脚本的输出用它们自己的 `--selftest` 或合成输入核；要么给出一条照定义做会被拒、做不下去或结果错的历史，要么写「构造不出」并说清卡在哪一步 |
| 本地攻方 / 本地辩方 | — | — | **缺席**：网关不通（见第二节最后一条）。按 `.claude/rules/three-way-inference.md`「本地腿缺席时必须显式报告」，这一轮只有两条云端腿，判决里写明 |

两条云端腿的立场不重叠：正推腿核「改的是不是该改的」，攻方腿核「照改后的做会不会出事」。

## 五、跑前条款

| 条款 | 什么观测会让它触发 |
|---|---|
| 一格判「站得住」⇒ 那一格的改动照留 | 正推推得出（每处都对得上一条核对结论或用户定案），攻方在那一格构造不出照做会出事的历史 |
| 一格判「站不住」⇒ 改那一处定义，写回之后这一轮的判决标「被攻过零轮」 | 攻方给出一条可复现的历史：照改后的定义做被钩子拒、两处文件给出相反做法、一步要的输入没人给，或正推指出改动越出了核对结论与用户定案 |
| 删改丢了判据 ⇒ 把判据补回 | 正推或攻方在 diff 里指出一句被删的原文带着一条今天仍然成立的判据，而改后的文字与它指向的文件里都找不到 |

⚠️ 一条历史若让改前、改后两种写法一起出局（打中不分辨候选），不拿它判这一批改动，按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」那一节另记一笔。

## 六、各条腿交什么

- 报告按派发提示给的路径分段写（每次写进文件不超过 150 行，第一段排他新建），回复只写路径、`sha256sum` 与判定一览。
- 引规则、定义、钩子、脚本写那份文件自己的行号，行号去原文件里现查，不从背景材料里数。
- 每条结论写「什么现象会推翻它」。
