# m3-close-gov-b17-r2：批 17 治理第一轮判决之后的修补，攻修补本身与没看过的地方（推论轮第二轮，治理轮）正文（2026-10-24）

## 一、题面

第一轮判决 `research/prompts/m3-close-gov-b17-r1-main-verification.md`（四格全采）之后，这一批照判决写了四份修补，都已打进主工作区；开工快照目录 `research/prompts/m3-close-gov-b17-r2-snapshot/` 里放这四份补丁原样与被判文件的 sha256：

1. `governance-fix-1.patch`（治理文本）：`.claude/rules/implementation-workflow.md`、`.claude/rules/verification.md` 两处 D13（验证路线） 已定项 17 复述照判决 G1 改（文件级补「管怎么存、怎么算键，不管判对错」，边级改成「按实测排，不沿它收」，「现在只有那一条」换成指登记位 `EXCLUDED_PROGRAM_EDGES`，自证要求只写在 implementation-workflow 一处）；`.claude/rules/three-way-inference.md` 判决复跑第 3 步改成 G2 的限定写法；`.claude/skills/crash-test/commit-stages.md` 第 2b 步改 G3 四处；`.claude/agent-common.md` 一行点名 `replay.sh` 的重型与只有 `--list-items` 不算；`.claude/hooks/heavy-test-guard.sh` 文件头一处（这一处后来由另一会话经 hook-swap 换上，以主工作区现文为准）。
2. `hooks.patch`（G4-a）：`.claude/hooks/lib_heavy_tests.py` 新加 `own_repository_script` 与表 `OWN_COPY_ONLY_SCRIPTS`（`replay.sh`、`crash-point-replay.sh`、`crash-amplification.sh`），`judged_by_name` 对表里的名字只认本仓那一份，别处同名拷贝照读正文；`.claude/hooks/heavy-test-guard.sh` 自证与文件头。
3. `mutation-judging-files.patch`（G1-g）：`scripts/mutation/lib_mutation_units.py` 的 `judging_units` 排掉 `scripts/crash/lib_verdict_library.py` 的 `NOT_JUDGING_FILES` 两份的单元（名单按路径载入那份库取），`scripts/mutation/crates-mutation-rows.py` 判法拷贝带上那两份、自证加一格与开关 `judging-files-included`。
4. `broad-gpu-import.patch`（G3-a）：`scripts/mutation/broad-mutation.py` 的 `--import-results` 把第二台的 `gpu-verdicts.tsv` 也并进本机（逐行核行键在本机清单、`gpu_key` 等于本机此刻 GPU 趟的键，核不过不并、列出、整条退 1；整表不成形整表拒；两边定论不同不导），自证加四格、开关四个；`.claude/skills/crash-test/commit-stages.md` 第 2b 步跟着改导入那一句。

要回答：这四份修补有没有把第一轮打中的口子补上、有没有补出新口子；第一轮四格得出结论的方法站不站得住；第一轮没看的地方（下面「格」一节点名）有没有问题。

## 二、实现今天的样子（主 agent 的观测，2026-10-24，主工作区现文）

- `crates/` 不在这一轮被判的文件里；被判的规则复述压着的实现在 `crates/singlefs-unit-graph`（Rust 与 WGSL 的单元与键）、`scripts/gate/lib_function_units.py`（python 单元、`EXCLUDED_PROGRAM_EDGES`）、`scripts/crash/lib_verdict_library.py`（`NOT_JUDGING_FILES`、`judging_unit_key`）。
- 修补 1 打进去之后，第 2b 步与驱动又被修补 4 改过一次；`heavy-test-guard.sh` 文件头那一处由另一会话换上；`crates-mutation-rows.py` 在修补 3 之后另一会话又换过一版（带 gpu 的 cargo 逐格经 `scripts/crash/gpu-quiesce-run.sh` 起，判法单元指纹 c63379ff…，465 个单元）。判的是主工作区现文，快照 `judged-sha256.txt` 记的就是现文。
- 主 agent 在主工作区现跑：修补 3 之后 `crates-mutation-rows.py judging-units` 465 个单元、出自那两份的 0 个；修补 4 打之前与之后 `broad-mutation.py` 的 `judging_units_key`、`gpu_judging_key` 都是 e8e3f3be…、7b912e93…；`BROAD_MUTATION_SELFTEST_CELLS=selftest_import_cells` 不带开关「7 格都对」，四个开关各红在文件头点名的格；`lib_heavy_tests.py` 自证 801 种、`heavy-test-guard.sh` 自证 1442 种，开关 `judged-by-name-path-tail`、`experiment-replay-unnamed` 各让它们红。

## 三、格

| 格 | 问什么 |
|---|---|
| H1 文本修补 | 修补 1 的每一处与判决第二节那一格的「主 agent 判」逐项比：判决要的改了没有、改出判决没要的东西没有；G1 两处复述与 `.claude/kb/decisions/13-验证路线.md` 已定项 17 定案现在逐项对得上吗；G2 限定写法与 `.claude/main-agent.md`「一轮怎么开、怎么收」第 11 条、`.claude/hooks/completion-cleanup-guard.sh` 的收工闸现在还矛盾吗；G3 第 2b 步四处与 `scripts/mutation/broad-mutation.py` 现文对得上吗（修补 4 之后再核一遍导入那一句）；`.claude/agent-common.md` 那一行是定义级改动，读来与 `.claude/rules/implementation-workflow.md`「checker 档只在提交时跑，harness 随时跑」那一节、`lib_heavy_tests.py` 的现判法矛不矛盾 |
| H2 只认本仓那一份 | `own_repository_script` 的认法会不会放过真重型或误拒本仓的轻用法：相对路径、`cd` 之后的相对路径、经 `bash -c` / `env` / `nice` / `run-with-memory-cap.sh` 包着、符号链接（指进与指出本仓）、`source`、被判的根是 `gate.sh --staged` 的临时树或别的 worktree 时本仓那一份指哪一份；表里只有三个名字，`KNOWN_SCRIPT_LOCATIONS` 里别的靠参数放行的名字有没有同一个口子 |
| H3 变异判法排除算键库 | `crates-mutation-rows.py` 的判法单元排掉 `NOT_JUDGING_FILES` 两份之后，有没有哪条路径上这两份的改动会改变 crates 变异的判定结局而键不变（少作废）：`lib_function_units.py` 在 crates 变异判法里除了算键还干什么；`not_judging_files()` 按路径载入另一份库，那份库载入失败或名单为空时怎么判；自证新格与开关能不能分辨「排掉了」与「本来就没进闭包」 |
| H4 GPU 判定导回 | `--import-results` 并 GPU 判定行时逐行核的两把键够不够：第二台与本机源码不同、`--source` 不同、本机 GPU 趟的键在导入之后才变、`--skip-gpu` 的行、部分写入的尾行、同一份结果导两次；被拒的行之后由本机 GPU 趟照现判，这一句在驱动里成立吗；自证四格的前提格是不是真的造出了两台分片的现场 |
| H5 第一轮的方法 | 第一轮判决第一节「核查」与第五节「复跑」得出四格结论的方法：两腿在新旧两版钩子上喂命令结果相同、正推一次复跑计数多 1 记「未稳」、G1-g 用 961 个单元数推「多作废不少作废」，这几步能不能撑判决里写的结论 |

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端辩方（Sonnet） | H1、H5 | 复核第一轮判决与修补 1：判决每一格够不够得着、修补是不是照判决写的；替被判「打中」的原文辩：第一轮采的改法是不是同样打中别的读法 |
| 云端攻方（Opus） | H1–H5 | 找反例：照修补后的规则字面做会做错的情形、钩子放过或误拒的命令、导入与排除下判定结局变而键不变的输入；不许重复第一轮已攻过的角度（第一轮报告 `research/prompts/m3-close-gov-b17-r1-opus-output.md`、`-sonnet-output.md`） |

方案按 `crates/` 与脚本今天的实现来谈。引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
