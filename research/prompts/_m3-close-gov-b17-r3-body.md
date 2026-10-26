# m3-close-gov-b17-r3：批 17 治理第三轮，攻前两轮站住的形态与第二轮挂起的三处（推论轮第三轮，治理轮，最终轮）正文（2026-10-24）

## 一、题面

前两轮判决：`research/prompts/m3-close-gov-b17-r1-main-verification.md`、`research/prompts/m3-close-gov-b17-r2-main-verification.md`。第二轮之后打进主工作区的修补二批 `governance-fix-2.patch`（开工快照目录 `research/prompts/m3-close-gov-b17-r3-snapshot/` 放原样与被判文件 sha256）：`.claude/rules/implementation-workflow.md` 第 49 行与 `.claude/hooks/heavy-test-guard.sh` 文件头补实验复跑入口 `scripts/experiment-apparatus/replay.sh`；`implementation-workflow.md` 第 31 行、`.claude/rules/verification.md` 第 35、50 行的依赖版本照 D13（验证路线） 已定项 17 定案写「解析出的闭包真引到的依赖版本」、删「理由不同、分开记」；`scripts/mutation/crates-mutation-rows.py` 自证那一格删掉 `lib_verdict_library.py` 的空测一半；`scripts/mutation/lib_mutation_units.py` 第 36 行引文同步。D13 已定项 17 定案现状句已改成指登记位 `EXCLUDED_PROGRAM_EDGES`（中英日）。

这是第三轮，也是最后一轮：只攻前两轮站住的形态；这一轮新冒出来的零轮形态不再开轮，写进判决的交用户表。

## 二、实现今天的样子（主 agent 的观测，2026-10-24，主工作区现文）

- `crates/` 不在被判文件里；被判的规则复述压着的实现在 `crates/singlefs-unit-graph`、`scripts/gate/lib_function_units.py`（`EXCLUDED_PROGRAM_EDGES` 现 3 条边）、`scripts/crash/lib_verdict_library.py`（`NOT_JUDGING_FILES`）。
- `scripts/mutation/crates-mutation-rows.py judging-units .` 首行「c63379ff285eba00ca251e038844715f9057189fb3fca205c2985172c9d0dccc 465」；`lib_function_units.py`、`lib_verdict_library.py` 的单元 0 个；开 `GATE_MUTATION_BREAK=judging-files-included` 是 961 个、其中 `lib_function_units.py` 496 个（第二轮攻方 out/closure-base.txt）。
- `crates-mutation-rows.py` 的判法入口 `JUDGING_ENTRIES`（现文第 510 行）含 `UnitKeys.mutated_units`、`resolve_test`、`executed_units`，后两者调 `lib_function_units.py` 切单元、算闭包；`executed_units` 的结果经 `executed_set_is_current`（第 1378 行）决定基线复不复用。
- `.claude/hooks/lib_heavy_tests.py` 的 `own_repository_script`（第 3375 行）用 `os.path.realpath` 比本仓那一份；三份入口都按 `dirname "${BASH_SOURCE[0]}"` source `.claude/scripts/preflight.sh`（`scripts/experiment-apparatus/replay.sh:99`）。
- `scripts/mutation/broad-mutation.py --import-results` 对 GPU 判定行核行键在本机清单（第 4322 行）、`gpu_key` 等于本机 GPU 趟的键（第 4324 行），有被拒的就整条退 1（第 4454 行）；CPU 那一侧不核行键在不在本机清单。
- 主工作区现跑：`heavy-test-guard.sh --selftest` 1442 种、`lib_heavy_tests.py --selftest` 801 种过；`crates-mutation-rows.py --selftest`「159 格都对」。

## 三、格

| 格 | 问什么 |
|---|---|
| K1 文件级排除在 crates 变异判法上成不成立（第二轮 H3-b） | D13 已定项 17 定案文件级「管怎么存、怎么算键，不管判对错」放在 crates 变异判法上：`lib_function_units.py` 经 `resolve_test`、`executed_units` 决定点名测试是哪个单元、覆盖落进哪几个单元、基线复不复用——改它会不会让某一行的判定（抓到 / 没红 / 复用与否）变而键不变。要三条互不共享前提的路径：读代码定位、在副本里改 `lib_function_units.py` 一处切单元的逻辑实测复用判定变不变、在副本里造一条变异行实测结局变不变；结论写成「成立 / 不成立」并给翻面的观测 |
| K2 入口符号链接旁的 preflight（第二轮 H2-a） | 第二轮判决给的两种零轮改法——只认命令词路径本身落在本仓（不认符号链接）、认符号链接时连旁边 `preflight.sh` 一起核——各自会放过什么、误拒什么；入口除 `preflight.sh` 之外还 source 什么（`scripts/hosts/`、`scripts/crash/` 下的库），第二种改法要核到哪一层才够 |
| K3 GPU 导入退码（第二轮 H4） | 持久的第二台 GPU 库里旧行被拒时整条退 1 是对的吗：commit-stages 第 2b 步与驱动文件头写的退 1 含义、操作者照出路做会不会误判成失败；CPU 那一侧旧行不退 1 的依据（第一轮 G3 判决那一句）放在 GPU 一侧成不成立 |
| K4 修补二批 | 修补二批每一处对第二轮判决第二节、第三节逐项比，有没有改出判决没要的东西；补进去的实验复跑入口那一句与 `lib_heavy_tests.py` 的 `classify` 现判法逐项对不对得上（`--list-items` 后面还带别的词算重型那一例） |
| K5 第二轮报出的数能不能核 | 第二轮判决与两腿报告里的数（13/14 倒推、cite-check 53/27/81、961/496/465、1442、801、159、identical=6 与八行）逐个现跑能不能核到 |

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1、K4、K5 | 从代码与定案正推：K1 走三条互不共享前提的路径给结论；K4、K5 逐项核 |
| 云端攻方（Opus） | K1–K5 | 找反例；K1 专找「改 `lib_function_units.py` 而某一行判定变、键不变」的具体输入；不许重复前两轮已攻过的角度（两轮报告 `research/prompts/m3-close-gov-b17-r1-*-output.md`、`research/prompts/m3-close-gov-b17-r2-*-output.md`） |

方案按 `crates/` 与脚本今天的实现来谈。引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
