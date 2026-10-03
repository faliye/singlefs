# 广谱变异接进提交门禁的治理改动：第一轮正文（2026-10-02）

<!-- doc-lint:not-numbers G1 G2 G3 G4 G5 C6 -->

## 一、题面

第 4 批 4-7（调度 `records/2026-09-30-里程碑三开工第一轮.md` 第 4 批）把广谱变异接进了提交门禁，关的是 `research/prompts/tooling-broad-mutation-prune-r3-main-verification.md` 第四节用户定案「提交前跑完才能提交」，判红条件照那份判决的第三版 C6。工具实现员报告 `/tmp/claude-1000/m3-r1/broad-mutation-gate/report.md` 第二节逐条列了改在哪。改了定义、共用约束与规则，照 `.claude/rules/implementation-workflow.md`「改 agent 定义与共用约束，走同一条三步」要走一轮三方；用户 2026-10-02 点名开这一轮。

同一天别的会话照用户指示删掉了 `crash-verifier` 与 `gate-triage` 两个定义，提交时的六道重阶段改由主 agent 照 `.claude/skills/crash-test/commit-stages.md` 跑（`crash-verifier.md` 的第 2b 步挪进了那份文件的第 2b 步），那批改动由用户逐份豁免（`.claude/agent-def-review-exempt`）。这一轮只判广谱变异那一批落在今天文件里的样子，不判删定义那一批本身；但两批落在同几份文件里，广谱那一批的句子在删定义之后还成不成立，在这一轮判。

被判的（冻结副本里今天的样子）：

| 文件 | 广谱那一批改了什么（报告第二节） |
|---|---|
| `.claude/main-agent.md` | 派发提示那一段的驱动两个上限；派发表「广谱变异提交门禁」；「暂存之后、提交之前跑门禁」那一行六道、排在 crates-mutation-replay 之后、红了不许提交 |
| `.claude/agent-common.md` | 重型清单加广谱变异与它的提交门禁 |
| `.claude/rules/verification.md` | checker 档那一格、复用那一句、「门禁管哪一半」一行、重型清单 |
| `.claude/rules/implementation-workflow.md` | 复用那一节、「每次提交代码」那一行、重型测试那一段 |
| `.claude/skills/crash-test/commit-stages.md` | 第 2b 步（由 `crash-verifier.md` 挪来） |
| `.claude/gate.d/checker-tier-broad-mutation.sh` 与样本、`stage-inputs.tsv`、`stage-owners.tsv` 那几行、`.claude/hooks/lib_heavy_tests.py`、`heavy-test-guard.sh`、`selftest-registry.tsv` | 新门禁两格、登记、重型判定 |

## 二、实现今天的样子（主 agent 的观测，2026-10-02）

- 腿读冻结副本 `/tmp/claude-1000/m3-batch5-freeze/tree/`（含 `.claude/`、`research/scripts/`、`crates/`），sha256 在 `research/prompts/m3-broad-gate-gov-r1-snapshot/tree-sha256.txt`。
- 驱动 `research/scripts/broad-mutation.py` 与剪枝规则库 `research/scripts/mutation-prune.py`；第 4 批修过驱动的过时产物问题（`tar -x -m`、换树后清工作区包、判法版本 `prune-r3-e1-e6-fresh-build` 进全局键、基线前核产物新鲜退 5，报告 `/tmp/claude-1000/m3-r1/broad-mutation-stale-fix/report.md`），真库冒烟换树后基线绿（`/tmp/claude-1000/m3-r1/broad-smoke3/`）。
- 工具实现员自己报的欠：`research/scripts/runner-dispatch-guard.sh`（实际路径 `.claude/hooks/runner-dispatch-guard.sh`）拒绝说明仍写五道；重型闸对跑广谱的一方放行的是「广谱变异」整类（含 `--milestone-full`、整趟跑），不按参数只放 `--sync-tree` / `--changed-since`，靠定义约束。删定义之后跑的一方变成主 agent，这一条今天怎么判，现查 `heavy-test-guard.sh` 的 AGENT_KINDS 与主 agent 那一支。

## 三、格

### G1　判红条件对不对得上用户定案
门禁两格（library-version-is-staged-tree、gate-verdict）判的是不是「这一次暂存树那一版每一条变异都有这一版的结论、沿用的结论都在、没有一条没红」；与 `tooling-broad-mutation-prune-r3-main-verification.md` 第三版 C6 逐句对。找一个该红不红（例：库目录换到暂存树之后又被别的会话换走、沿用的结论来自判法版本不同的一版）或不该红而红的输入。

### G2　次序与复用
第 2b 步：等 crates-mutation-replay 跑完再起、包暂存树成提交、`--sync-tree --commit`、`--changed-since HEAD`、再跑门禁。几个会话共写一个仓时（别的会话同时 `--sync-tree`），库目录是全局的：会不会判到别人那一版。复用判定（`stage-inputs.tsv` 那一行、24 小时上限）罩没罩全输入（驱动、剪枝规则库、忽略用例排除表、工具链与 cargo-nextest / cargo-mutants 版本）。

### G3　定义里的句子成不成立
五份文件里广谱那一批写下的每一句，在删定义之后的今天还对不对：点名的步号、文件、谁跑、带什么前缀、红了怎么办；文件之间说法一致不一致（六道 / 五道、谁跑）。逐句列不成立的。

### G4　重型闸
`lib_heavy_tests.py` 的 `classify` 把驱动的哪些用法判重型、哪些不算（`--list-items`、`--summary`）；主 agent 不带前缀跑驱动会不会被拒、子 agent 跑会不会被拒；放行整类而不按参数的风险今天有多大（推的就写推的）。

### G5　样本与弄坏开关
12 份样本、8 个弄坏开关：每个开关打开之后是不是红在点名的那一份；有没有判红分支没有样本或开关盯着。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | G1、G3 | 从用户定案与判决推：门禁判的与定案说的逐句对；五份文件的句子逐句核 |
| 云端攻方（Opus） | G2、G4、G5，兼攻 G1 | 在冻结副本的拷贝上造该红不红、不该红而红的输入；钩子与门禁样本照它们自证的起法跑（自证登记为 light 的才跑；驱动的真跑、门禁整道都是重型，不跑，写成要主 agent 跑的命令） |

## 五、跑前写死的判据

- 一个输入让门禁该红不红或不该红而红 ⇒ 打中，改门禁或驱动。
- 五份文件里一句话与今天的做法不符 ⇒ 打中，改那一句。
- 一个判红分支没有样本或开关盯着 ⇒ 记一笔，补样本。
- 一条腿报「没打中」只抽了一次样；腿提的新形态算零轮。

交付：
- 腿的报告写 `research/prompts/m3-broad-gate-gov-r1-<模型>-output.md`，分段落盘，每次写进文件不超过 150 行，第一段排他新建，回复只写短句；
- 攻方的副本与用例放 `research/prompts/m3-broad-gate-gov-r1-opus-model/`；草稿放 `/tmp/claude-1000/<腿名>/`；
- 引 kb 与规则写文件自己的行号（去冻结副本里现查）；方案按 `crates/` 与脚本今天的实现来谈。
