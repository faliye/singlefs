## E201 三oracle合谋今树整表 —— 装置已写、两段产物未跑；第 1 次重跑已跑（够判，2026-11-18：变异表第 57 行没红判等价，换成会改行为的一行之后 57 条全抓，六个单测模块全过，变异驱动起得来）

**中文** · [English](201-three-oracle-collusion-full-table-on-the-current-tree.en.md) · [日本語](201-3オラクル共謀現ツリー全表.ja.md)

<!-- doc-lint:not-numbers Q1 Q2 P0 P1 P2 P3 T2 T3 R7 G1 G2 G3 G4 G5 G6 G7 K2 K3 V1 V14 F3 F5 F6 F7 Z1 Z2 Z3 Z5 Z6 Z7 Z8 S1 S2 S6 S7 U1 F1 -->

跑前登记 `research/prompts/e201-preregistration.md`（第十二节修订 1 写在装置写之前，修订 2 写在 2026-11-03 任何产物之前）。单测、变异与自证跑过，此刻的数见「第 1 次重跑」一节；两段整表产物还没跑。回答问题单 `research/prompts/e171-current-tree-full-table-questions.md` 两行：Q1 问今天主工作区的实现上，E171（三oracle合谋测试） 两张坏实现名单（合计 278 行）逐行过三个 oracle，判定与 r1 那一次逐格比变了哪几格、每一格归不归得到这期间改过的某一处；Q2 问三列各自漏判的行与 r1 比多了还是少了。

取新号、不作 E171 的一次重跑：被跑的是今天主工作区的另一份实现，跑法与编排不沿用 r1 修订 8 那一层（冻结树、宣布行、占位、按 sha256 互钉的编排），比法是新写的（登记第一节）。这一次的读数不进 E171 第 1、2 次重跑的任何判据，r1 的读数也不进这一次的判据，只是逐格比的另一边（登记读法 12）。

用户 2026-11-02 在弹窗里定的三条（登记修订 1）：Q2 只判在两边都有逐列读数的那几行上（D，至多 7 行），整表上三列各自漏判的行只描述、不与 r1 比；流三那几格也算进够判范围，第一段之后接着起第二段；「今天的树」认交任务那一刻的主工作区，产物里记下那一刻的 HEAD 与工作区改动清单，归因时用，不设宣布行。

### 现状

- 研究一侧装置 `scripts/experiment-apparatus/e201_three_oracle_collusion_full_table_on_the_current_tree/`（Python，不与 `crates/` 共用代码；E171 研究一侧的比对器、`collusion_matrix.py` 等只按路径 import，不改、不拷）：
  - `full_table_run.py`：入口，文件头声明分片、要卡与续跑；经服务分片三步——计划拷此刻主工作区的树进计划目录、记快照摘要与 HEAD 与改动清单、核名单与锚点、按读法 10 的键查 prune-server、抽哨兵、把要现跑的行分两片；各片编不改坏的装置（带 gpu 特性）、跑基线，片 0 另跑第二次基线、E170 导出装置与基线逐文件比、O3（独立规约执行器） 基线与九样阳性对照两遍，两片都跑空变异，再起逐行驱动跑分到的行、判完一行就把那一行打包写进库；合并从库里取回全部行、出表 1–3 与逐格比、归因、Q2、整表矩阵、对照与条款逐条。
  - `current_tree_row_driver.py`：照抄 E171 的逐行驱动，编装置去掉 `nice`、加 `--features gpu`，O3（独立规约执行器） 阳性对照那一步去掉 `nice`，`--positive-control` 收全九样；删掉了只为复跑 E171 旧产物留的「编不过不补位」那一支。
  - `contrast_against_r1.py`（读法 7 的配键与逐格比）、`attribution.py`（读法 8 的 G1–G7 与宽归因）、`q2_misses.py`（读法 9 的 D 与漏判）、`whole_table_matrix.py`（整表每行每列的红数与轨迹）。
  - 三份表：`cell_classes.tsv`（读法 7 的分类表，sha256 `4c053fa2…`）、`device_function_map.tsv`（G5，`95b93eb4…`）、`research_function_map.tsv`（G7，`9761c84f…`），sha256 钉在代码里，对不上不比、不归因（登记修订 2）。
- 单测六份（`test_contrast_against_r1.py`、`test_q2_misses.py`、`test_attribution.py`、`test_whole_table_matrix.py`、`test_current_tree_row_driver.py`、`test_full_table_run.py`）107 条时一趟 `python3 -B -m unittest` 点那六个模块跑过 `Ran 107 tests`、`OK`（`type=succeeded exit=0`），此刻 125 条（第 1 次重跑那一节）；自证 `full_table_run.py --selftest` 四格（复用三问、哨兵、复用账、存进库的行与完整导出逐字相同），该自证那一趟（`type=succeeded exit=0`）`FULL_TABLE_SELFTEST cells=4 passed=4 failed=0 failed_cells=-`；弄坏开关六条（各带一个弄坏开关的环境变量，跑 `full_table_run.py --selftest` 或 `test_current_tree_row_driver`，结局行都是 `type=command-failed exit=1`）各红在点名的那一项；变异表 `scripts/mutation/tables/e201_three_oracle_collusion_full_table_on_the_current_tree.tsv` 45 条时（V1–V14、24 条落地行与 7 条拷树行）的产物 `research/results/e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-04-r2-s1-mutations.out` 末行 `E7RESULT name=verdict mutations=45 caught=45 invalid=0 survived=0 timed_out=0 restored_and_baseline_green=true`，此刻 57 条（第 1 次重跑那一节）。
- 过程与前期产物：变异初版 14 条（`research/results/e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-03-s1-mutations.out`，末行 `E7RESULT name=verdict mutations=14 caught=14 invalid=0 survived=0 timed_out=0 restored_and_baseline_green=true`）、变异二版 38 条（`research/results/e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-03-r2-s1-mutations.out`，末行 `E7RESULT name=verdict mutations=38 caught=38 invalid=0 survived=0 timed_out=0 restored_and_baseline_green=true`）；分片计划阶段产物 `research/results/e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-03-s1-plan.out`、`research/results/e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-03-r2-s1-plan.out`、`research/results/e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-03-r3-s1-plan.out`、`research/results/e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-03-r4-s1-plan.out`。
- O3（独立规约执行器） 一侧（登记修订 8，2026-11-11）：执行器目录 `scripts/experiment-apparatus/e170_second_run_o3_spec_executor_journal_header_496/`（journal 记录头 496 字节），冻结参照 `research/results/e170-o3-spec-executor-journal-header-496-2026-11-09-freeze-executor-e171-r3.out`，与 E171 第 3 次重跑同版；不再是 487 目录与 r2 冻结参照，所以 O3（独立规约执行器） 列与 r1 的比对要记 O3（独立规约执行器） 版本不同。装置只在 `e171_research_side.py` 里换了本进程的三个常量，E171 两个目录没动；单测加 `FrozenO3Is496Tests` 两条、变异表加两行，这两格 2026-11-11 跑过（单测与变异表产物见「口径」第一条）。
- 两段产物都还没跑；`research/results/` 下还没有 E201 的两段产物。

### 复跑

在仓根起，整条交 `task-server/task submit`：

```
python3 -B -m unittest test_contrast_against_r1 test_q2_misses test_attribution test_whole_table_matrix test_current_tree_row_driver test_full_table_run   # --cwd 装置目录
python3 mutation_driver.py --table ../../mutation/tables/e201_three_oracle_collusion_full_table_on_the_current_tree.tsv --scratch <草稿目录> --output <新产物>   # --cwd 装置目录
python3 scripts/experiment-apparatus/e201_three_oracle_collusion_full_table_on_the_current_tree/full_table_run.py --selftest
python3 scripts/experiment-apparatus/e201_three_oracle_collusion_full_table_on_the_current_tree/full_table_run.py run                  # 第一段
python3 scripts/experiment-apparatus/e201_three_oracle_collusion_full_table_on_the_current_tree/full_table_run.py run --stream-three   # 第二段
```

产物名：`research/results/e201-three-oracle-collusion-full-table-on-the-current-tree-<计划那一天>-<段>-<件>.out`，段 `s1`（流一、流二整表）或 `s2`（流三），件是 plan、t1、t2、t3、contrast、attribution、q2、matrix、controls、verdict。两段的复跑登记等两段产物出来再写（登记 5.6「提交时」）；第 1 次重跑的变异那一条已登记：`scripts/experiment-apparatus/replay.sh` 复跑表 `E201|@driver_e201_mutations||e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-18-r1-mutations-2026-11-18.out|exact`（第 57 行换上之后那一版表的产物；驱动把六个单测模块逐个用 `--unit-test-module` 给，与不给时同名同序），点名跑 `bash scripts/experiment-apparatus/replay.sh --item E201`（树动过的那一趟与留存产物不同是树的事，六个单测模块读真仓）。

### 口径

- O3（独立规约执行器） 用哪一版：496 目录与 E171 第 3 次重跑的冻结参照（登记修订 8），每片开头与收尾 `frozen_o3.freeze_differences` 各核一次（作废 Z2）；与 r1 的 O3（独立规约执行器） 列不同版，O3（独立规约执行器） 一侧差出来的格不算 E201 实现的差。换版之后单测 `Ran 115 tests`、`OK`，变异表产物 `research/results/e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-11-mutations.out` 末行 `E7RESULT name=verdict mutations=50 caught=50 invalid=0 survived=0 timed_out=0 restored_and_baseline_green=true`。

- 「今天的树」：计划那一步拷进计划目录的主工作区 `Cargo.toml`、`Cargo.lock`、`.cargo`、`crates/`、`prune-server/`，与 crates 编、跑时读的六份仓根文件 `.claude/heavy-thread-default`、`scripts/crash/crash-point-replay-shard-configuration-check.sh`、`scripts/hosts/peer-host-lib.sh`、`.claude/scripts/preflight.sh`、`.claude/singlefs-ai-sop/scripts/preflight.sh`、`.claude/singlefs-ai-sop/scripts/preflight.py`（不带 `target`，含别的会话没提交的改动；`prune-server/` 是 crates 经 path 依赖到的仓根目录，登记修订 5 加，缘由是第一段整趟在任务 5503 停在 S6；六份文件是编译期 `include_str!` 读的那一份与装置运行期从编它的那棵树起的多机配置判法连同它 source、起的四份，登记修订 6 加，缘由是第一段整趟在任务 5606 又停在 S6）；各片从这一份同步线道树，开头与收尾核快照摘要与各文件 sha256（作废 Z3）。这几样不写死，由 `scripts/hosts/lib_repository_copy.py` 的 `copy_closure` 现算（登记修订 7）。`multi-host.env` 不拷：副本里起的命令显式带主工作区那一份的路径。
- r1 一边只读 git 里的三份产物：P1 第二段比对器、P2 第二段逐行驱动合成产物、P3 第一段比对器（sha256 与行数核过才读，作废 Z5）；P0 只取它记的 git status 那几行的路径，给归因用。
- 一格变了 = 同一个键、同一个判定格两边取值不同；红的计数取「是否大于 0」，行号集合按集合比；只在一边的键算那一行「在不在」一格变了。钉值格的差另报，不进 Q1；P2 的 `o3_status` 这类说整表 O3（独立规约执行器） 跑没跑的字段记今天独有（登记修订 2 第 2.1 条）。
- 归因只认七种窄的（名单、坏实现行文、补位、落点函数、装置函数、oracle 入口、研究一侧函数）；一格有一种中就记归得到，都不中而有查不了的记查不了（F6）。r1 那一版两份比对器与 `collusion_matrix.py` 的 sha256 不在 E171 研究一侧的记录里，落到它们的那一种一律查不了（登记修订 2 第 2.3 条）。
- Q2 的 D：R7（第 15、23、42、242、269、270、271 行）里 P3 有流一流二 `batch=1` 的逐列读数、P1 与今天的表 1 都计入分母、今天的 T3 也有逐列读数的行；判定取登记读法、`batch=1`；`batch=2` 与原登记读法 5 并列报。

### 岔路表

| 问题单那一行 | 已够判 / 还差什么 | 剩下的量能不能让它翻面 |
|---|---|---|
| Q1 | 还差：单测、变异与自证，第一段整趟（流一、流二整表的逐格比与归因），第二段（流三十六行） | 能：三个候选都还开着 |
| Q2 | 还差：同 Q1；D 的大小跑之前不知道（为空走 F4） | 能：两个候选都还开着 |
| 第 1 次重跑 MT（变异表每一行） | 已够判：r1 那一版表有行不是抓到，第 57 行没红、判等价；等价留档、第 57 行换成会改行为的一行之后，整表 57 条全抓（第 1 次重跑那一节） | 不能：两版表的 57 行结局与三个数都落定，没有剩下的量 |
| 第 1 次重跑 UT（六个单测模块） | 已够判：全过，读真仓的那一条红分到 U1、装置一侧已修；换上留档用例之后 125 条全过 | 不能：剩下的只有附带量 |
| 第 1 次重跑 DR（变异驱动起得来） | 已够判：起得来，退 2 的原因是基线单测红 | 不能 |

### 第 1 次重跑（2026-11-18：变异表、装置单测与变异驱动在今天的树上；已跑，够判）

重跑登记 `research/prompts/e201-r1-prereg.md`（修订 1 在产物之前：单测红的分类与装置一侧的修；修订 2 在产物之后：只记读数与第 57 行的处置；修订 3：复跑登记改指换上之后的产物、复跑驱动的写法），问题单 `research/prompts/e201-r1-questions.md` 三行：MT 变异表此刻每一行各是抓到、无效还是没红；UT 装置六个单测模块在今天的树上是否全过；DR 变异驱动此刻起不起得来。只跑变异驱动 `mutation_driver.py` 与六个单测模块，两段整表、`full_table_run.py --selftest` 与弄坏开关不在里面；答的是装置此刻健不健康，不进 Q1、Q2 的判据。

今天的树：控制之前、控制之后、正式之后三次树摘要都是 `f44b458d8cb589976a607fe388f0a8948ef4d394e318905380d57390a4ed6722`，装置摘要前后都是 `196a8d7ecad5612fb54f0a44043fc855b65fe291e0b3b1f36b4c90675cc774f8`，作废 Z2、Z3 都不触发；改后单测那一趟（`python3 -B -m unittest -v` 点那六个模块，`type=succeeded exit=0`）前后没取树摘要，它的「全过」另由控制与正式产物里四次同六个模块的基线绿印证。第 57 行换上之后的单测与整表（`python3 -B -m unittest -v` 那一趟与 `mutation_driver.py` 那一趟，两趟都是 `type=succeeded exit=0`）前后三次树摘要都是 `c6105f91ff0aa72be325b1d67c33ee923252a90402de8954e9831e874c061ed8`，装置摘要前后都是 `59f018b854bb13606bb438e833ccc29ea18020a207072f2d61fb91cbf48f222e`（重跑登记修订 2 末条，主 agent 取）。

两份整表产物各是一版表的读数：`e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-18-r1-mutations.out`（`mutation_driver.py` 那一趟，`type=command-failed exit=1`）是第 57 行还是「判法 E171 目录只在 e171_research_side 里插」那一版表（sha256 `7ff739ee…`），留着、读数不改；`e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-18-r1-mutations-2026-11-18.out`（`mutation_driver.py` 那一趟，`type=succeeded exit=0`）是第 57 行换成「判法 E171 目录不进 import 路径」那一版表（sha256 `2ab53e47…`），复跑表那一行指它。两份按变异名逐行比，只有第 57 行不同（`diff` 两份的 `name=mutation` 行去掉 `number=`）。

| 行 | 答案 | 产物整行 |
|---|---|---|
| MT（r1 那一版表） | 有行不是抓到：56 条抓到、1 条没红、无效与超时 0 | `e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-18-r1-mutations.out` 第 117 行 `E7RESULT name=verdict mutations=57 caught=56 invalid=0 survived=1 timed_out=0 restored_and_baseline_green=true`；第 116 行 `E7RESULT name=mutation number=57 outcome=❌ mutation=判法_E171_目录只在_e171_research_side_里插 unit_test_exit=0` |
| MT（第 57 行换上之后那一版表） | 57 条全抓，无效、没红与超时 0 | `e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-18-r1-mutations-2026-11-18.out` 第 117 行 `E7RESULT name=verdict mutations=57 caught=57 invalid=0 survived=0 timed_out=0 restored_and_baseline_green=true`；第 116 行 `E7RESULT name=mutation number=57 outcome=✅ mutation=判法_E171_目录不进_import_路径 unit_test_exit=1` |
| UT | 全过：修 U1 之后 124 条，换上等价留档用例之后 125 条；两次 `skipped`、`FAIL:`、`ERROR:` 都零命中，钉值三条都 `ok` | `e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-18-r1-unit-tests-after.out` 第 217 行 `Ran 124 tests in 20.026s`、第 219 行 `OK`；`e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-18-r1-unit-tests-after-2026-11-18.out`（`python3 -B -m unittest -v` 那一趟，`type=succeeded exit=0`）第 182 行 `Ran 125 tests in 13.567s`、第 184 行 `OK` |
| DR | 起得来：第 57 行换上之前那一趟 `mutation_driver.py` 退 1（`type=command-failed exit=1`）、换上之后那一趟退 0（`type=succeeded exit=0`），两份产物都完整；更早那一趟退 2（`type=command-failed exit=2`）的原因是基线单测红，与 UT 红同一件事 | 两份正式产物第 59 行都是 `E7RESULT name=baseline unit_tests=green`、第 118 行都是 `E7RESULT name=done emitted=118`；退 2 那一趟的残留 `e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-18-mutations.out` 第 59 行 `E7RESULT name=baseline unit_tests=red`、第 60 行 `E7RESULT name=done emitted=60` |

- UT 那一条红（`python3 -B -m unittest` 点那六个模块那一趟，`type=command-failed exit=1`、末行 `FAILED (failures=1)`，改前日志 `e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-18-r1-unit-tests-before.out`）：读真仓那条单测扫到 `crates/singlefs-implementation-experiments/src/bin/e185_gc_batch_wall_clock_split/main.rs` 里解到仓根自身的 `concat!(env!("CARGO_MANIFEST_DIR"), "/../..")`，报 `['.'] != []`。分到 U1（装置的扫描要跟）：`test_full_table_run.py` 把仓根自身单列、要求它的出处行带 `CARGO_MANIFEST_DIR`，别的路径照旧要被两份 `SYNCED_ENTRIES` 罩住；补三条手造用例，三条手造变异各红（三条各跑一趟 `python3 -B -m unittest -v test_full_table_run`，两条各红 1 条（末行 `FAILED (failures=1)`），另一条红 3 条（末行 `FAILED (failures=3)`）：多红的那一条的假文件里也有仓根自身那一句）。
- 第 57 行没红，判等价：运行期 `e171_research_side.py` 无条件把 E171 修后版目录插进 `sys.path`，逐行驱动那一句的判断恒为假；登记预期的取样点（算键的库静态解逐行驱动的 import）上也同值，因为 `scripts/gate/lib_function_units.py` 的 `PythonModule.search_directories` 把模块级 import 进来的 `e171_research_side` 加的目录算在逐行驱动上。只在算键的库开弄坏开关 `python-imported-sys-path-ignored` 时两式不同，那是换了判法。
- 第 57 行的处置（照 `.claude/rules/test-discipline.md`「变异测试证明的是断言会红，不是覆盖」第 2 条：等价的写成测试留档，再换一条真会改行为的）：
  - 留档用例：`test_current_tree_row_driver.py` 的 `JudgeImportTests.test_without_the_drivers_own_insertion_the_key_library_still_resolves_the_e171_modules_through_the_research_side`（拷进小树的逐行驱动去掉自己那一句，算键的库照样把四份 import 解到 E171 修后版目录）。默认开关下 `ok`（`e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-18-r1-unit-tests-after-2026-11-18.out` 第 74–75 行）；算键的库开着弄坏开关、只跑 `JudgeImportTests` 时退 1、「FAILED (failures=1)」，红的恰是这一条（在装置拷贝里跑，`type=command-failed exit=1`）。这一红一绿是等价的校验：去掉逐行驱动那一句之后四份还解得出，靠的是 `e171_research_side` 加的目录。
  - `current_tree_row_driver.py` 那一句上方的注释改成写两条路（这份模块自己的插入、`e171_research_side` 加的目录）都让四份解得出、这一句与研究一侧那一句同值，代码不动。
  - 表第 57 行换成「判法 E171 目录不进 import 路径」：`e171_research_side.py` 的 `    sys.path.insert(1, E171_DIRECTORY)` 换成 `    pass`，锚点只在 `e171_research_side.py` 命中一次（新产物第 58 行 `E7RESULT name=anchor mutation=判法_E171_目录不进_import_路径 hits=1 files=e171_research_side.py`）；表里这一行上方一行注释写明逐行驱动那一句等价、不进表。表仍 57 行。
  - 换上之前在装置拷贝里预跑这一行一次（那一趟 `type=succeeded exit=0`，只这一行的表抓到）。
- 控制：`e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-18-r1-control-mutations.out` 第 9 行 `E7RESULT name=verdict mutations=3 caught=1 invalid=1 survived=1 timed_out=0 restored_and_baseline_green=true`（三条依次 ✅ ❌ ⏭）；`e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-18-r1-control-red-baseline.out` 第 5 行 `E7RESULT name=baseline unit_tests=red`、第 6 行 `E7RESULT name=done emitted=6`。管子分得出抓到、没红、无效与基线红（F1 不触发）。
- r1 那一版表的产物与 `e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-11-mutations.out`（50 行全抓）按变异名并排：同名 49 行两边都 ✅；`V13_键不含改坏用的变异行` 在重跑后的两版表里都叫 `V13_请求不含改坏用的变异行`（两份产物都 ✅）；6 条「复用」与第 57 行原产物没有。
- 附带：第七个测试模块 `test_row_selection_and_reuse` 单跑一趟（`type=succeeded exit=0`）`Ran 7 tests`、`OK`（不进 UT）。够判后未跑：登记第八节「测试集合更大」那一格（第 57 行换七个模块再跑一次）；「超时放宽」没有 ⏱、不触发。

### 它答不了的

- r1 那一次的读数对不对（问题单「不问」）；撤掉第 236 行之后判定词变不变（E171 第 2 次重跑已判）。
- 整表 O3（独立规约执行器） 与计入分母的行的逐列读数在 r1 那一边：git 里没有，只报今天这一边。
- 「归得到」只说那一格落在某一种改动上，不说那一处改动造成了它；要当因果用另走三方。
- 第 1 次重跑只说装置的单测与变异驱动在树 `f44b458d…`（r1 那一版表）与 `c6105f91…`（第 57 行换上之后那一版表）上健不健康，不说 E171（三oracle合谋测试） 的判定在今天的实现上变没变（那是 Q1、Q2）；变异只证明写了断言的那几处会红，不说装置每一处都测过。

### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 6 | 备料 | 2026-11-19 不受影响：只调整措辞与引用写法，结论、数值与产物不变；此前 2026-11-18 不受影响：第 1 次重跑只答装置的单测与变异驱动此刻健不健康（r1 那一版表 57 行 56 条抓到、第 57 行判等价；等价留档、第 57 行换成会改行为的一行之后 57 条全抓；单测 125 条全过，驱动起得来），换产物、改复跑驱动只动装置一侧，没有 Q1、Q2 的读数，依据段仍没有引 E201（三oracle合谋今树整表），关系仍是备料；此前 2026-11-11 不受影响：O3 一侧换到 496 目录与 E171（三oracle合谋测试） 第 3 次重跑的冻结参照、核对应表时照搬迁登记倒换，只动装置与登记修订 8，产物仍没跑、没有读数，依据段仍没有引 E201（三oracle合谋今树整表），关系仍是备料；此前 2026-11-05 不受影响：只改用法与复跑说明（一个门禁、两种跑法；验证包改名），结论、数与产物不变；此前 2026-11-03 不受影响：装置已写、产物未跑，没有读数；E201（三oracle合谋今树整表） 答的是 E171（三oracle合谋测试） 的判定在今天的实现上变没变，依据段没有引它，只写备料 |

## 历史版本

### 2026-11-03

- 建档，登记 E201（三oracle合谋今树整表） 研究一侧装置、单测、变异表与决策回看；登记修订 1（用户定的三条）、修订 2（分类表与两张对应表）；产物未跑。
