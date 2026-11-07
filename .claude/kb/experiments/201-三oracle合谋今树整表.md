## E201 三oracle合谋今树整表 —— 装置已写、产物未跑

**中文** · [English](201-three-oracle-collusion-full-table-on-the-current-tree.en.md) · [日本語](201-3オラクル共謀現ツリー全表.ja.md)

<!-- doc-lint:not-numbers Q1 Q2 P0 P1 P2 P3 T1 T2 T3 R7 G1 G2 G3 G4 G5 G6 G7 K2 K3 V1 V14 F3 F5 F6 F7 Z1 Z2 Z3 Z5 Z6 Z7 Z8 S1 S2 S6 S7 O1 O2 O3 -->

跑前登记 `research/prompts/e201-preregistration.md`（第十二节修订 1 写在装置写之前，修订 2 写在今天任何产物之前；单测、变异、自证与产物都还没跑）。回答问题单 `research/prompts/e171-current-tree-full-table-questions.md` 两行：Q1 问今天主工作区的实现上，E171（三oracle合谋测试） 两张坏实现名单（合计 278 行）逐行过三个 oracle，判定与 r1 那一次逐格比变了哪几格、每一格归不归得到这期间改过的某一处；Q2 问三列各自漏判的行与 r1 比多了还是少了。

取新号、不作 E171 的一次重跑：被跑的是今天主工作区的另一份实现，跑法与编排不沿用 r1 修订 8 那一层（冻结树、宣布行、占位、按 sha256 互钉的编排），比法是新写的（登记第一节）。这一次的读数不进 E171 第 1、2 次重跑的任何判据，r1 的读数也不进这一次的判据，只是逐格比的另一边（登记读法 12）。

用户 2026-11-02 在弹窗里定的三条（登记修订 1）：Q2 只判在两边都有逐列读数的那几行上（D，至多 7 行），整表上三列各自漏判的行只描述、不与 r1 比；流三那几格也算进够判范围，第一段之后接着起第二段；「今天的树」认交任务那一刻的主工作区，产物里记下那一刻的 HEAD 与工作区改动清单，归因时用，不设宣布行。

### 现状

- 研究一侧装置 `scripts/experiment-apparatus/e201_three_oracle_collusion_full_table_on_the_current_tree/`（Python，不与 `crates/` 共用代码；E171 研究一侧的比对器、`collusion_matrix.py` 等只按路径 import，不改、不拷）：
  - `full_table_run.py`：入口，文件头声明分片、要卡与续跑；经服务分片三步——计划拷此刻主工作区的树进计划目录、记快照摘要与 HEAD 与改动清单、核名单与锚点、按读法 10 的键查 prune-server、抽哨兵、把要现跑的行分两片；各片编不改坏的装置（带 gpu 特性）、跑基线，片 0 另跑第二次基线、E170 导出装置与基线逐文件比、O3 基线与九样阳性对照两遍，两片都跑空变异，再起逐行驱动跑分到的行、判完一行就把那一行打包写进库；合并从库里取回全部行、出 T1–T3 与逐格比、归因、Q2、整表矩阵、对照与条款逐条。
  - `current_tree_row_driver.py`：照抄 E171 的逐行驱动，编装置去掉 `nice`、加 `--features gpu`，O3 阳性对照那一步去掉 `nice`，`--positive-control` 收全九样；删掉了只为复跑 E171 旧产物留的「编不过不补位」那一支。
  - `contrast_against_r1.py`（读法 7 的配键与逐格比）、`attribution.py`（读法 8 的 G1–G7 与宽归因）、`q2_misses.py`（读法 9 的 D 与漏判）、`whole_table_matrix.py`（整表每行每列的红数与轨迹）。
  - 三份表：`cell_classes.tsv`（读法 7 的分类表，sha256 `4c053fa2…`）、`device_function_map.tsv`（G5，`95b93eb4…`）、`research_function_map.tsv`（G7，`9761c84f…`），sha256 钉在代码里，对不上不比、不归因（登记修订 2）。
- 单测六份（`test_contrast_against_r1.py`、`test_q2_misses.py`、`test_attribution.py`、`test_whole_table_matrix.py`、`test_current_tree_row_driver.py`、`test_full_table_run.py`）合计 107 条，任务 6325 跑过 `Ran 107 tests`、`OK`；自证 `full_table_run.py --selftest` 四格（复用三问、哨兵、复用账、存进库的行与完整导出逐字相同），任务 6326 `FULL_TABLE_SELFTEST cells=4 passed=4 failed=0 failed_cells=-`；弄坏开关六条（任务 5540–5545）各红在点名的那一项；变异表 `scripts/mutation/tables/e201_three_oracle_collusion_full_table_on_the_current_tree.tsv` 45 条（V1–V14、24 条落地行与 7 条拷树行），产物 `research/results/e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-04-r2-s1-mutations.out` 末行 `E7RESULT name=verdict mutations=45 caught=45 invalid=0 survived=0 timed_out=0 restored_and_baseline_green=true`。
- 过程与前期产物：变异初版 14 条（`research/results/e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-03-s1-mutations.out`，末行 `E7RESULT name=verdict mutations=14 caught=14 invalid=0 survived=0 timed_out=0 restored_and_baseline_green=true`）、变异二版 38 条（`research/results/e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-03-r2-s1-mutations.out`，末行 `E7RESULT name=verdict mutations=38 caught=38 invalid=0 survived=0 timed_out=0 restored_and_baseline_green=true`）；分片计划阶段产物 `research/results/e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-03-s1-plan.out`、`research/results/e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-03-r2-s1-plan.out`、`research/results/e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-03-r3-s1-plan.out`、`research/results/e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-03-r4-s1-plan.out`。
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

产物名：`research/results/e201-three-oracle-collusion-full-table-on-the-current-tree-<计划那一天>-<段>-<件>.out`，段 `s1`（流一、流二整表）或 `s2`（流三），件是 plan、t1、t2、t3、contrast、attribution、q2、matrix、controls、verdict。`replay.sh` 的复跑登记等产物出来再写（登记 5.6「提交时」）。

### 口径

- 「今天的树」：计划那一步拷进计划目录的主工作区 `Cargo.toml`、`Cargo.lock`、`.cargo`、`crates/`、`prune-server/`，与 crates 编、跑时读的六份仓根文件 `.claude/heavy-thread-default`、`scripts/crash/crash-point-replay-shard-configuration-check.sh`、`scripts/hosts/peer-host-lib.sh`、`.claude/scripts/preflight.sh`、`.claude/singlefs-ai-sop/scripts/preflight.sh`、`.claude/singlefs-ai-sop/scripts/preflight.py`（不带 `target`，含别的会话没提交的改动；`prune-server/` 是 crates 经 path 依赖到的仓根目录，登记修订 5 加，缘由是第一段整趟在任务 5503 停在 S6；六份文件是编译期 `include_str!` 读的那一份与装置运行期从编它的那棵树起的多机配置判法连同它 source、起的四份，登记修订 6 加，缘由是第一段整趟在任务 5606 又停在 S6）；各片从这一份同步线道树，开头与收尾核快照摘要与各文件 sha256（作废 Z3）。这几样不写死，由 `scripts/hosts/lib_repository_copy.py` 的 `copy_closure` 现算（登记修订 7）。`multi-host.env` 不拷：副本里起的命令显式带主工作区那一份的路径。
- r1 一边只读 git 里的三份产物：P1 第二段比对器、P2 第二段逐行驱动合成产物、P3 第一段比对器（sha256 与行数核过才读，作废 Z5）；P0 只取它记的 git status 那几行的路径，给归因用。
- 一格变了 = 同一个键、同一个判定格两边取值不同；红的计数取「是否大于 0」，行号集合按集合比；只在一边的键算那一行「在不在」一格变了。钉值格的差另报，不进 Q1；P2 的 `o3_status` 这类说整表 O3 跑没跑的字段记今天独有（登记修订 2 第 2.1 条）。
- 归因只认七种窄的（名单、坏实现行文、补位、落点函数、装置函数、oracle 入口、研究一侧函数）；一格有一种中就记归得到，都不中而有查不了的记查不了（F6）。r1 那一版两份比对器与 `collusion_matrix.py` 的 sha256 不在 E171 研究一侧的记录里，落到它们的那一种一律查不了（登记修订 2 第 2.3 条）。
- Q2 的 D：R7（第 15、23、42、242、269、270、271 行）里 P3 有流一流二批 1 的逐列读数、P1 与今天的 T1 都计入分母、今天的 T3 也有逐列读数的行；判定取登记读法、批 1；批 2 与原登记读法 5 并列报。

### 岔路表

| 问题单那一行 | 已够判 / 还差什么 | 剩下的量能不能让它翻面 |
|---|---|---|
| Q1 | 还差：单测、变异与自证，第一段整趟（流一、流二整表的逐格比与归因），第二段（流三十六行） | 能：三个候选都还开着 |
| Q2 | 还差：同 Q1；D 的大小跑之前不知道（为空走 F4） | 能：两个候选都还开着 |

### 它答不了的

- r1 那一次的读数对不对（问题单「不问」）；撤掉第 236 行之后判定词变不变（E171 第 2 次重跑已判）。
- 整表 O3 与计入分母的行的逐列读数在 r1 那一边：git 里没有，只报今天这一边。
- 「归得到」只说那一格落在某一种改动上，不说那一处改动造成了它；要当因果用另走三方。

### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 6 | 备料 | 2026-11-05 不受影响：只改用法与复跑说明（一个门禁、两种跑法；验证包改名），结论、数与产物不变；此前 2026-11-03 不受影响：装置已写、产物未跑，没有读数；E201（三oracle合谋今树整表） 答的是 E171（三oracle合谋测试） 的判定在今天的实现上变没变，依据段没有引它，只写备料 |

## 历史版本

### 2026-11-03

- 建档，登记 E201（三oracle合谋今树整表） 研究一侧装置、单测、变异表与决策回看；登记修订 1（用户定的三条）、修订 2（分类表与两张对应表）；产物未跑。
