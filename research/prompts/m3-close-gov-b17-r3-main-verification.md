# m3-close-gov-b17-r3 判决（治理轮第三轮，最终轮，2026-10-24）

被判对象：前两轮站住的形态、第二轮挂起的三处（H2-a、H3-b、H4）与修补二批，正文 `research/prompts/_m3-close-gov-b17-r3-body.md`，开工快照 `research/prompts/m3-close-gov-b17-r3-snapshot/`（`governance-fix-2.patch` 与 `judged-sha256.txt` 12 份）。这一轮判过的文件：`.claude/rules/implementation-workflow.md`、`.claude/rules/verification.md`、`.claude/skills/crash-test/commit-stages.md`、`.claude/hooks/heavy-test-guard.sh`、`.claude/hooks/lib_heavy_tests.py`、`scripts/mutation/lib_mutation_units.py`、`scripts/mutation/crates-mutation-rows.py`、`scripts/mutation/broad-mutation.py`。腿：正推（Sonnet）`research/prompts/m3-close-gov-b17-r3-sonnet-output.md`（5ca4b705…），攻方（Opus）`research/prompts/m3-close-gov-b17-r3-opus-output.md`（95336710…）。

## 一、核查

1. 两腿各自跑 cite-check：攻方「核了 10 处引文、18 处裸行号，对上 9 处，没判 19 处」，正推「核了 12 处引文、46 处裸行号，对上 12 处，没判 46 处」，都 0 处对不上。K1 引到的几处主 agent 在主工作区现查：`scripts/mutation/crates-mutation-rows.py:1284`（基线键带判法单元指纹与点名测试的单元指纹）、`:1287`（覆盖号只带点名测试的单元指纹，不带判法单元指纹）、`:1378`「if executed_set_is_current(keys, coverage):」、`scripts/mutation/lib_mutation_units.py:553`、`:559`（覆盖落单元用 `lib_function_units.py` 给的行号区间）、`:774`（排除）。
2. 正推报告点名开工之后 `scripts/mutation/broad-mutation.py` 被别的会话改过（sha256 对不上快照）；这一轮没有一格的结论用它的新内容。
3. 复跑：攻方模型拷进 `~/.local/share/singlefs/work/m3-close-gov-b17-r3-verify/` 经内存包装跑 `tools/rerun.sh`，退 0，`name=rerun_compare file=k1-runs.txt result=identical`、`file=k1-runs-long.txt result=identical`、`name=rerun_summary identical=2 differs=0`，真树那一步「name=real_tree scripts=copy-lfu1 fn_units=19946 units=38189 placed_elsewhere=878」「name=real_tree_fingerprints identical=1」。正推模型整套复跑约 35 分钟，这一轮没复跑；它的数只作 K1 两腿对照用，判决不靠它独有的数。

## 二、逐格判定

| 格 | 正推 | 攻方 | 主 agent 判 |
|---|---|---|---|
| K1 文件级排除在 crates 变异判法上成不成立 | 成立：副本里改 `lib_function_units.py` 五种（E1–E5），整张表 5513 行的 `resolve_test`、`mutated_units` 与 50 行取样的 `executed_units`、基线复用，没有「键不变而复用变」；登记一笔代价（等价改写在带它的指纹下全表现判） | 不成立，三条路：读代码；真 crates/ 上改 Rust 单元末行多算一行，38189 个单元指纹逐字不变、878 个 fn 落进别的单元；小仓整道 `run_stage`，同一把行键只换它，第二趟从「基线不绿、退 1」翻成「抓到、退 0」（假抓到），改回来也不作废；附：覆盖号不带判法单元指纹，放回指纹只修换代那一趟 | **采攻方**。两腿事实不矛盾：正推的五种改法没有一种让 fn 换单元（它的 E1 只动起始行、`set_current_flipped_rows=0`），量到的「复用照旧」正是排除之后应有的样子；攻方那一种让 878 个 fn 换单元，键全不变而判定翻面，主 agent 复跑逐字相同。三条路互不共享前提（只读代码 / 真树真 `executed_units` / 小仓假 cargo 整道跑）。所以 D13（验证路线） 已定项 17 文件级排除「管怎么存、怎么算键，不管判对错」放在 crates 变异判法上不成立：`lib_function_units.py` 在这里经 `executed_units` 管覆盖落单元，覆盖决定基线复用。第一轮 G1-g 采的改法（修补 3）翻面。怎么改是零轮形态，交用户（第三节） |
| K2 入口符号链接旁的 preflight | — | 入口按自己所在目录定根，cd 进链接那棵树之后 source 至少 6 份库；「连旁边 preflight.sh 一起核」只核其中一份，不够；「不认符号链接」要写成「整条真实路径不经符号链接」才堵得住，本机不误拒本仓轻用法（推的，静态读） | **采**：第二轮第二种改法不够、第一种要写严。改法零轮，交用户（第三节） |
| K3 GPU 导入退 1 | — | 退 1 的语义（有旧行没并，去起 ③）对，照 commit-stages 第 2b 步做不会误判成失败；但出路第一支「重跑那一片、再重导」在持久库上不收敛（只追加，旧被拒行还在）；正文说「第一轮 G3 判决那一句」在判决里找不到，那句在驱动自证 `broad-mutation.py:6659` | **采两处**：出路改成「被拒的旧行由本机 GPU 趟现判，不靠重导」；正文那一句的出处是主 agent 写错，记在这里更正。文字修补进下一批 |
| K4 修补二批 | 依赖版本、论证句、判据句、入口句对得上；`verification.md:35` 不在判决点名里也改了；入口句「`--list-items`」没写「单独」，`lib_heavy_tests.py:621`、`:623` 是「恰好一个」 | 同：三处入口句「单独」只挂在 `--selftest` 上，`classify` 对 `--list-items E165` 判重型 | **采**：三处（`implementation-workflow.md:49`、`heavy-test-guard.sh` 文件头、`agent-common.md:58`）照 `classify` 写「只带一个 `--list-items`」。`verification.md:35` 是主 agent 同一口径顺改，算进这一批 |
| K5 第二轮的数 | 465/961/496、1442、801、159、identical=6 现跑核到；八行 identical、13/14 倒推、cite-check 27/81 今天复现不了（树已变） | 同 | **采**：能核的都核到；核不到的那几样是修补二批改了它们读的文件，不当第二轮报错，也不当核过 |

## 三、交用户（第三轮之后不再开轮，零轮形态）

| 题 | 候选 | 代价与量过没有 |
|---|---|---|
| K1 crates 变异判法怎么收 `lib_function_units.py` | 甲：撤回修补 3，两份放回判法单元指纹（改前的样子）；乙：甲加覆盖号带判法单元指纹；丙：只把 `executed_units` 那一路用到的函数挑进指纹，再加覆盖号带指纹 | 甲修「换代那一趟」、不修「换代之后再改源码」（攻方 D、E 量过，只在它的模型上）；乙两格都修（F 量过，模型上）；丙推的、没实现。甲、乙每改一次 `lib_function_units.py`（排除 496 个单元）crates 变异整表现判一次；同时要改 D13 已定项 17 定案文件级那一句的射程 |
| K2 入口符号链接 | 甲：只认整条真实路径不经符号链接的本仓那一份；乙：认符号链接、连它 source 的全部库一起核 | 甲推的不误拒本仓轻用法、堵得住；乙要列全 source 的库，漏一份就漏 |

用户 2026-10-24 弹窗定：K1「乙：收回指纹+覆盖号带指纹 (Recommended)」；K2「甲：不认符号链接 (Recommended)」。

## 四、另登

| 项 | 去向 |
|---|---|
| 正推旁注：闭包带 wgsl 单元的测试，`executed_set_is_current` 永远不「当前」（50 行取样里 6 条），基线复用只剩闭包号那一支，偏保守 | 记进拆分提案（2026-10-23 份） |
| 第二轮判决、两腿报告里核不到的那几个数 | 不追：树已变 |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 17 | 支撑 | 2026-10-24 改了：文件级排除在 crates 变异判法上不成立（K1），定案射程要改，改法交用户（第三节）；用户定案之后写回 |
