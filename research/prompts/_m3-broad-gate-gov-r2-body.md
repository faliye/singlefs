# 广谱变异接进提交门禁的治理改动：第二轮正文（2026-10-03）

<!-- doc-lint:not-numbers G1 G2 G3 G4 G5 H1 H2 H3 H4 H5 A1 A3 A4 A5 A6 A7 A8 A9 A10 C6 E4 -->

## 一、题面

第一轮判决 `research/prompts/m3-broad-gate-gov-r1-main-verification.md` 第三节定了十条改法，第四节用户定丙（编译步、文档测试步整步超时与内存撞顶照旧算没结论、门禁红，补一条走得通的出路）。工具实现员照改，报告 `/tmp/claude-1000/m3-r1/batch5-broad-gate-fix/report.md`（「逐条」一节第 1–10 条写了改在哪、出口怎么核的）。改过的定义与规则是 `.claude/agent-common.md`、`.claude/main-agent.md`、`.claude/rules/verification.md`、`.claude/rules/implementation-workflow.md`，另有 skill `.claude/skills/crash-test/commit-stages.md`。用户 2026-10-03 弹窗定：这五份走第二轮三方，不豁免。

这一轮攻的是第一轮的修补本身：十条改法落在今天文件里的样子，加上用户定案丙的出路。第一轮已判、修补没碰的形态不重攻。

被判的（冻结副本里今天的样子）：

| 文件 | 这一批改了什么（报告「逐条」） |
|---|---|
| `.claude/gate.d/checker-tier-broad-mutation.sh` 与 26 份样本 | 样本模式起 `.broad-sample/prepare.py` 造库目录那一版（第 1 条）；8 份新红样本与开关（第 2 条）；写标记前核工作区等于暂存区（第 3 条，`marker_inputs_clean`、`write_marker_if_clean`）；开格前钉住 `tree.source`、跑完再核（第 4 条）；第一格现算全局键逐行比（第 5 条）；两行各数互核（第 7 条） |
| `research/scripts/broad-mutation.py` | 全局键进 `version_key`、`global_key_lines` 一种写法、`check_global_key` 退 7（第 5 条）；`--changed-since` 重选 timeout / memory-cap（第 6 条）；文件头出路句（第 9、10 条） |
| `.claude/gate.d/stage-inputs.tsv` | 这一道那一行补 `run-with-memory-cap.sh`、`capped.sh`、llvm-tools 版本（第 5 条） |
| `.claude/hooks/lib_heavy_tests.py`、`heavy-test-guard.sh`、`selftest-registry.tsv` | `--check-ignored`、`--check gate-verdict` 放行，旧名 `AGENT_KINDS` 换掉（第 8 条） |
| `.claude/skills/crash-test/commit-stages.md` | 第 1 步免检删掉；第 22、26、27、28 行改写，③ 改走 `detached-run.sh`（第 3、9 条） |
| `.claude/agent-common.md`、`.claude/main-agent.md`、两份规则 | 重型例外清单三处对齐（第 8 条）；内存包装清单加驱动；六道上限那一句挪到「一轮怎么开、怎么收」第 12 条（第 9 条） |

## 二、实现今天的样子（主 agent 的观测，2026-10-03）

- 腿读冻结副本 `/tmp/claude-1000/m3-gov-r2-freeze/tree/`（`crates/`、`litmus/`、`.claude/`、`.cargo/`、`research/scripts/`、`Cargo.*`，3 191 个文件），sha256 在 `research/prompts/m3-broad-gate-gov-r2-snapshot/tree-sha256.txt`。
- 工具实现员自己核过的：26 份样本照「cd 进样本根、根参数写同一个目录」逐份跑全对、20 个开关各红在点名那几份；驱动 `--selftest` 55 格、`lib_heavy_tests.py --selftest`、`heavy-test-guard.sh --selftest` 都绿。`stage-selftest.sh --item checker-tier-broad-mutation.sh` 被重型闸拒（样本与暂存区不同），没跑。
- 真库今天那一版（`~/.local/share/singlefs/broad-mutation/tree.source`，指纹 e1ba257413cbad99）没记全局键：报告第 5 条说提交时第 2b 步 ② 重做 `--sync-tree` 就是新一版。
- `crates/` 这一轮不被判；冻结副本里的 `crates/` 只当驱动与门禁的输入用。

## 三、格

### H1　修补本身：十条改法对不对
逐条核报告第 1–10 条的改法做没做到第一轮判决第二节那一行要的：找一个改完之后仍该红不红、或新引入的不该红而红的输入。重点：
- 第 3 条：`marker_inputs_clean` 取的路径集合与 `stage_fingerprint` 同一份吗；别的五道写标记前核的是什么，这一道与它们同规矩吗。
- 第 4 条：钉住的是 `tree.source` 的内容，库目录里结论表与 `global-key.txt` 在两格之间被换走时判不判得出。
- 第 5 条：全局键进指纹后，`carry_decision` 一条都不沿用的范围对不对（只该在全局键变了时不沿用）；门禁现算的全局键与驱动 `--sync-tree` 算的是不是逐字节同一份（限时参数取那一版记的那一行）。
- 第 7 条：互核的各等式在驱动所有结局组合下都成立吗（`--skip-stage-two`、驱动错、没跑）。

### H2　产生结论的方法：样本与自证够不够得着
第 1 条让样本经 `prepare.py` 载入今天的驱动造库目录那一版：样本与被判的驱动共用 `fingerprint_of_manifest`、`version_key`、`global_key_lines`。这些函数写错时样本还红不红；有没有一个判红分支只有共用这几个函数的样本盯着。20 个开关逐个打开时，是不是只有点名那一份翻面、别的份不翻（开关之间有没有互相兜住）。

### H3　用户定案丙的出路走不走得通
第 6 条：`--changed-since` 重选 timeout / memory-cap、加大 `--timeout-floor` 或 `--memory-cap` 再 `--item <序号> --rerun`、仍没结论交用户。在真库的规模下（一版几千条）这条出路能不能在一次提交里走完；「交用户定」之后门禁照旧红，用户定了算什么以后，有没有一个机制让门禁认（没有就写明没有）。

### H4　定义里的句子：从没被看过的地方
五份文件里这一批写下或挪动的每一句，今天还对不对：`commit-stages.md` ③ 改走 `detached-run.sh` 之后，前缀 `SINGLEFS_HEAVY_TESTS=commit` 进不进脱离单元（`detached-run.sh` 只带 PATH、HOME）；第 28 行退出码表与驱动文件头逐个对；`main-agent.md` 第 12 条挪过去之后两处指针都指得到；三处重型例外清单与 `lib_heavy_tests.py` 的 `BROAD_MUTATION_QUERY_ARGUMENTS` 逐项相同。

### H5　几个会话共写一个仓
库目录是全局的。别的会话在第 2b 步 ② 与 ④ 之间 `--sync-tree` 或真跑：第 4 条的钉住与第 5 条的退 7 各拦得住哪一种，拦不住的给出输入（推的写推的）。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端辩方（Sonnet） | 复核第一轮判决；H3、H4 | 复核第一轮判决每一行够不够得着、改法选得对不对（第 5 行选「全局键进版本指纹」而不是「门禁另核」的理由站不站得住；第 6 行丙的出路）；替这一批的做法辩护，也逐句核五份文件 |
| 云端攻方（Opus） | H1、H2、H5，兼攻 H3 | 在冻结副本的拷贝上造该红不红、不该红而红的输入；样本照「cd 进样本根、根参数写同一个目录」起，钩子与驱动只跑登记为 light 的自证；驱动真跑与门禁整道是重型，不跑，写成要主 agent 跑的命令 |

## 五、跑前写死的判据

- 一个输入让门禁或驱动该红不红、不该红而红 ⇒ 打中，改门禁或驱动。
- 五份文件里一句话与今天的做法不符 ⇒ 打中，改那一句。
- 一个判红分支只有与被判代码共用同一段函数的样本盯着、那段函数写错时样本不翻面 ⇒ 打中，补一份不共用的样本或开关。
- 丙的出路在某种结局组合下走不出去 ⇒ 打中，交用户。
- 一条腿报「没打中」只抽了一次样；这一轮新提的形态算零轮。

交付：
- 腿的报告写 `research/prompts/m3-broad-gate-gov-r2-<模型>-output.md`，分段落盘，每次写进文件不超过 150 行，第一段排他新建，回复只写短句；
- 攻方的副本与用例放 `research/prompts/m3-broad-gate-gov-r2-opus-model/`；草稿放 `/tmp/claude-1000/<腿名>/`；
- 引规则与定义写文件自己的行号（去冻结副本里现查）；方案按 `crates/` 与脚本今天的实现来谈。
