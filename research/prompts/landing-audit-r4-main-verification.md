# landing-audit-r4 判决：子 agent 落地审计第九批改动第一轮（主 agent，2026-10-23）

正文：`research/prompts/_landing-audit-r4-body.md`。两条腿：正推（Sonnet，K1）`research/prompts/landing-audit-r4-sonnet-output.md`（sha256 `129be6566579eb2cfd0b612196d6b6cf07bb547593b8f299d180ada5812d1d06`）、攻方（Opus，K2、K3）`research/prompts/landing-audit-r4-opus-output.md`（sha256 `aa5c9ad8f502ec1e6ca45ef26bd5a69c26fa5e434124132307d0cbf1ea97c680`），模型目录 `research/prompts/landing-audit-r4-opus-model/`。

## 这一轮被判过的文件

- `.claude/main-agent.md`：第 8 条结局账、第 10 条与「写与验分两段」验证段第 1 步的 checker 档快档那一步；攻方 K2-1、K2-2、K2-3 打中第 3 条（第 22 行）、调度表第 96 行与第 102 行没跟着改。
- `.claude/agent-common.md`：交回正文闸补的 ⑤ ⑥ 两样（没人打中）；第 58 行「其余任何时候重型不跑」一类说法没写合入后验证那一步的例外（攻方 K2-1）。
- `.claude/agents/implementation-writer.md`：第 4 步列举补命名检查；没人打中。
- `.claude/agents/kb-scribe.md`：写范围两处同步到 `**/*.md`；没人打中（调度表第 102 行没跟上，归 `.claude/main-agent.md` 那一条）。
- `.claude/agents/experiment-designer.md`：第 1 步点名比对对象表的固定形态；没人打中。
- `.claude/agents/three-way-forward.md`、`.claude/agents/three-way-attack.md`、`.claude/agents/three-way-defense.md`：frontmatter 第 9 行 `leg: three-way`；这一轮两条腿照常派出。
- `.claude/hooks/runner-dispatch-guard.sh`：⑳ 草稿目录认法、㉑ 三个前缀；攻方 K2-4、K2-5、K3-2 打中，正推判 ⑳ 一半（自证没读真提示）。
- `.claude/hooks/handback-guard.sh`：⑤ 命名检查、⑥ 写回员目标；攻方 K3-1 打中 ⑤。
- `.claude/hooks/bash-command-detector.sh`（⑰ 那一段）、`.claude/hooks/lib_three_way_legs.py`、`.claude/hooks/write-guard.sh`（kb-scribe 那几格）、`.claude/hooks/agent-write-scope.tsv`、`.claude/hooks/lib_heavy_tests.py`（顶层 `lib-` 库）、`.claude/hooks/selftest-registry.tsv`：没人打中。
- `.claude/gate.d/code-tooling.sh`、`.claude/gate.d/doc-experiments.sh`、`.claude/gate.d/doc-decisions.sh`、`.claude/gate.d/doc-registries.sh`、`.claude/gate.d/checker-independence-and-sync.sh`、`.claude/gate.d/experiment-number-directories.tsv`、`.claude/gate.d/break-switch-replay-files.tsv`、`.claude/gate.d/stage-owners.tsv` 与正文列的八个新样本：没人打中；正推腿报五道样本档判对。
- `.claude/gate.d/replay-comparison-units.tsv`：第 17 到 21 行「# 不登记的比对对象」；攻方 K2-6 打中它与 `.claude/kb/checks-owed.md` 的 C879（E165 根选择与杀伤界正式跑没有单元）判法相反、互不指向。
- `scripts/gate/merge-and-verify.sh`（checker 档快档那一步）：攻方 K3 没打中（本仓 8 个成员都在 checker 档包的依赖闭包里，「补丁没碰闭包」只剩两种情形）；K2-2 打中的是调度表里的命令。
- `scripts/agents/agent-watch.py`、`scripts/agents/agent-cost.py`、`scripts/agents/watch.conf`、`scripts/agents/watch.sh`、`scripts/agents/dispatch-prompt.py`、`scripts/kb/claim-experiment.sh`、`scripts/kb/lib_experiment_number_directories.py`：没人打中。
- `records/2026-09-16-subagent拆分提案.md` 第九十节去向列、`records/2026-10-22-子agent结果落地审计.md` 节「第九批」：正推腿逐行对过，⑯ ⑳ ㉓ 判一半。

## 一、核查

- 开工快照：`sha256sum -c research/prompts/landing-audit-r4-snapshot/judged-sha256.txt` 80 份全部 OK。`kb-sha256.txt` 里 `.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.en.md`、`.ja.md` 两份对不上：腿跑着的时候被别的会话改的，这一批没碰；两份腿报告引它们 0 处（`grep -c`），不影响判决。
- 引文：`python3 scripts/agents/cite-check.py <两份腿报告> --root . --background research/prompts/_landing-audit-r4-background.md` 末行「✓ 核了 52 处引文、93 处裸行号，对上 52 处，没判 93 处（其中裸行号没有引文可比 93 处；逐处列在上面）」。判决要用的裸行号主 agent 逐处现查：`.claude/main-agent.md` 第 22 行（第 3 条「提交之外任务确实要跑，先弹窗问用户」）、第 96 行（调度表 `bash scripts/gate/merge-and-verify.sh [--memory 上限] [--threads N] <补丁目录…>`，不带前缀）、第 102 行（`crates/singlefs-checker/invariants/*.md`（不变量清单，只限这些）；`.claude/gate.d/replay-comparison-units.tsv` 第 17 到 21 行五条「# 不登记的比对对象：research/prompts/e165-r5-prereg.md #15」到「#19」；`.claude/kb/checks-owed.md` 第 560 行 C879 仍写比对对象表第 15 至 19 行的正式跑在单元表里没有单元。
- 攻方模型复跑：模型目录拷到草稿目录，`bash scripts/hosts/run-with-memory-cap.sh 4G bash <副本>/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs --work <新草稿目录> --model <副本>` 退 0，四份输出 `naming_probe.out`、`threshold_probe.out`、`scratch_prompts.out`、`merge_select_probe.out` 与腿交回的 `out/` 逐字节相同（`cmp`）。
- 正推腿报的样本档读数，主 agent 重跑两道：`SINGLEFS_HEAVY_TESTS=user-request bash scripts/hosts/run-with-memory-cap.sh 4G bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh /home/fy5090/code/singlefs/.claude/gate.d --item doc-registries.sh` 退 0「点名的 1 道阶段判得都对（51 个用例）」；`--item checker-independence-and-sync.sh` 退 0「（16 个用例）」。另三道（doc-decisions 53、doc-experiments 60、code-tooling 94）没重跑，判决不引它们的数。

## 二、逐格判

### K1（正推：每处改法做没做到它那一行）

| 九十节 | 腿判 | 主 agent 判 | 依据 |
|---|---|---|---|
| ⑥ ⑦ ⑧ ⑨ ⑫ ⑬ ⑭ ⑮ ⑰ ⑱ ⑲ ㉑ ㉒ ㉔ | 做到 | 采（⑥ 另见 K3-1） | 腿逐行引了文件:行；主 agent 重跑的自证读数见调度记录节「第九批」 |
| ⑯ | 一半：加键后真派一次没核成；线程上限与预计挂钟两张类型表里仍手抄三个名字 | 前一半不采：这一轮两条三方腿是在三份定义改完之后新派的，照常派出（`.claude/main-agent.md` 开头 `CLAUDE.md` 引的那句「改了定义要新派才生效」），派得出就是这一条要的观测；后一半不采：那两张是派发闸 ⑭ ⑮ 判的全部定义类型的名单，不是三方腿类型表，三方腿在里面和别的类型并列 | 两条腿的派发结果；`.claude/main-agent.md`「派发提示怎么写」两段 |
| ⑳ | 一半：真提示放行拒绝都对，自证里是仿写 | 采：九十节 ⑳ 那一行写「自证拿第三轮攻方 `research/prompts/landing-audit-r3-opus-model/` 里那几份真提示当样本」，`grep -c 'landing-audit-r3-opus-model' .claude/hooks/runner-dispatch-guard.sh` 为 0 | 现查 |
| ㉓ | 一半：第 3 条没写例外，另两处口子 | 采第一处（与 K2-1 同一件）；另两处见 K3 | `.claude/main-agent.md` 第 22 行 |
| ⑩ ⑪ | 不在这一批 | 采 | ⑩ 归上游，⑪ 第七批已做 |
| 推的、没量过的三样 | 都没写成量 | 采 | `scripts/agents/watch.conf` 第 25 行写「512 本身是推的，没量过」 |

### K2（攻方：新句子与已有的有没有矛盾）

| 条 | 腿报 | 主 agent 判 |
|---|---|---|
| K2-1 | 合入后验证带 `=user-request` 跑 checker 档快档、不弹窗；`.claude/main-agent.md:22`、`.claude/rules/implementation-workflow.md:45`、`:47`、`.claude/hooks/heavy-test-guard.sh:857`、`.claude/agent-common.md:58` 仍写「提交之外先弹窗」或「其余任何时候重型不跑」 | 采。用户定案「合入后验证跑受影响的快档」（第三轮判决节「用户定案」）就是这一步的授权，几处说法要写出这个例外，指到定案；`.claude/rules/implementation-workflow.md` 是规则文件，改它与改定义同一批 |
| K2-2 | `.claude/main-agent.md:96` 调度表里的 merge-and-verify 命令没带前缀，照抄起新加的那一步拒、整条退非 0 | 采（现查第 96 行） |
| K2-3 | `.claude/main-agent.md:102` 写回员例外只有 `invariants/*.md` | 采（现查第 102 行） |
| K2-4 | 「本提示的草稿目录」四处认法：派发闸 ⑰ 冒号可有可无、不限行首；派发闸 ⑯、完工清理闸、交回闸 ⑥ 要行首带冒号 | 采，与 K3-2 一起改：四处共用一份认法，取骨架那一种（行首「草稿目录：」）。⑰ 放宽是为第三轮攻方那份不照骨架写的工具员提示；照骨架写的提示都有这一行，没有这一行的提示照旧只判裸文件名（九十节 ㉗ 第三处已记），所以收严不丢罩住的对象 |
| K2-5（轻） | 三方腿报告路径三种认法：派发闸报告行那一支放行没填的 `<轮>`，新加那一支不认；交回闸 ② 连子目录都认 | 采，并进九十节 ㉗：三方腿报告路径的认法收进 `.claude/hooks/lib_three_way_legs.py` 一处 |
| K2-6 | `replay-comparison-units.tsv` 第 17 到 21 行把 C879 要登记的五行写成合法的不登记；C879 仍写「把这几条登记进」，两处互不指向 | 采。不是判法错（五行确实还没单元，不登记是如实），是两处说法没连上：登记表那五行指到 C879，C879 写明这五行现记在登记表「不登记」一栏、补了单元就删那几行；主 agent 自己写（kb 小处改动） |
| 没打中 | 结局账 `.txt` 放 `records/`、共用约束与交回闸文件头、交回闸 ⑤ 与 rsync 一支、`experiment-number-directories.tsv` 的位置 | 记没打中；单条腿一次没打中不支撑「没有问题」（`.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」），这几处留给下一轮换角度攻 |

### K3（攻方：哪一条最脆）

| 条 | 腿报 | 主 agent 判 |
|---|---|---|
| K3-1 | 交回闸 ⑤ 把 `false &&`、`true ||`、`if false`、只定义不调用的函数、`timeout 0.1` 被杀退 124 这五种没跑完的命名检查认成真跑过 | 采。复跑 `naming_probe.out` 逐行相同：五行都是「退 0」，对照「改了 .rs 没跑：退 2」。改法：只认工具结果退出码 0、且命令里那两条处在顺序执行位置（不在 `&&` / `||` 右侧之外的条件里、不在 `if` / 函数体里、外层不套会截断的 `timeout`），各加红格与开关 |
| K3-2 | ⑰ 认草稿目录太宽也太死：转述里「，草稿目录 <别处>」「- 草稿目录 <别处>」误拦，「草稿目录在 <路径>」误放 | 采。复跑 `scratch_prompts.out`：`relay-comma`、`relay-list` 退 2，`own-zai` 退 0。改法与 K2-4 同 |
| ⑬ | 真大的命令照报；峰值为 0 时阈值不单调 | 记没打中，阈值不单调记进九十节 ㉙ 一并给下一轮 |
| ⑭ | 只读了代码，推的 | 不算一次观测，下一轮换腿量 |
| merge-and-verify 闭包 | 8 个成员都在闭包里，判法落空，「本次未跑」只剩改 `scripts/experiment-apparatus` 与空补丁两种 | 采为观测：这一步实际上每次合入都跑，不是缺陷；正文 K3 担心的「闭包外被测试读到的文件让它误报本次未跑」在本仓今天不发生 |

## 三、判决

1. 第九批的改法，K1 十四行做到、⑯ 做到（这一轮派发坐实），这几处站住第一轮。
2. 采的八条（K2-1、K2-2、K2-3、K2-4 并 K3-2、K2-5、K2-6、K3-1、⑳ 自证读真提示）是这一批的修补，记成九十节 ㉘，并进第十批，第十批改完开 landing-audit-r5 攻修补本身。改法都是主 agent 与腿提的，被攻过零轮。
3. 派发前主 agent 自己撞上的两处一并登记：派发闸不判「禁读清单点到这条腿自己的报告路径」（这一轮攻方提示头一版就是这样，派发闸放行，主 agent 自己改的）；写闸的会话名一判只管 Write / Edit，经 Bash 的 `insert-row.py`、`replace-once.py` 写进 `records/` 的会话名不判（调度记录 11 处、拆分提案 1 处，主 agent 已逐处换掉）。记进九十节 ㉙。
4. K2-6 的两处互指由主 agent 写；其余交工具员。

## 四、交用户

无设计岔路。第十批要不要现在开，属于「一批任务停下来就停，等用户指令」（`.claude/main-agent.md`「一轮怎么开、怎么收」第 12 条），收尾时问。

## 五、没核动的

- 正推腿报的 doc-decisions 53、doc-experiments 60、code-tooling 94 三道样本档读数，主 agent 没重跑，不引。
- 攻方没扫真会话记录与真提示语料，K3-1、K3-2 那几种形状实际出现过几次不知道。
- ⑭ 只读了代码，没量。

## 回看决策

不涉及决策：被判的都是 agent 定义、共用约束、钩子、门禁与协作脚本的工作流写法，加上两份调度记录；`.claude/kb/checks-owed.md` 的 C879 只补一句指向，不改它管的 E165（恢复重放有界穷举与变异杀伤界） 结论与状态。
