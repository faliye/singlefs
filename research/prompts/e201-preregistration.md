# E201 跑前登记：三oracle合谋今树整表

写于 2026-11-03，装置写之前。
批次 `records/2026-10-31-删兼容旧版本代码.md#18`。被测条款与岔路单见第一、二节。

<!-- doc-lint:not-numbers Q1 Q2 P0 P1 P2 P3 R7 G1 G2 G3 G4 G5 G6 G7 K1 K2 K3 K4 A1 A2 A3 A4 A5 A6 A7 A8 V1 V2 V3 V4 V5 V6 V7 V8 V9 V10 V11 V12 V13 V14 F1 F2 F3 F4 F5 F6 F7 F8 Z1 Z2 Z3 Z4 Z5 Z6 Z7 Z8 S1 S2 S3 S4 S5 S6 S7 O1 O2 O3 -->

## 一、问题

英文名：three_oracle_collusion_full_table_on_the_current_tree

**为什么取新号、不作 E171（三oracle合谋测试） 的第 3 次重跑**：E171 的重跑登记（`e171-r<n>-prereg.md`）是同一套臂与判据、同一份装置在照登记取的输入上再跑一次，由准入判输入变没变；这一次被跑的是今天主工作区的另一份实现，跑法与装置编排不沿用 E171 第 1 次重跑修订 8 那一层（冻结树、宣布行、H1–H34 占位、按 sha256 互钉的编排层），比法也是新写的（与 r1 留在 git 里的产物逐格比、逐格归因），照 `.claude/singlefs-ai-sop/rules/test-discipline.md`「实验开跑之前，答案不许已经存在」一节「改了就是新实验，重跑」是新实验。用户 2026-11-02 定「第 2 次重跑收尾，整表另立新实验」（E171 第 2 次重跑登记修订 9 第 9.2 条原句「新实验另写一份不继承 r1 冻结层的跑前登记，先设计、后跑」），派发提示写明「准入已判：…是新立的实验，不按 E171 的准入登记判」。先例：E200（修订6近似树重跑） 照同一理由取了新号（`research/prompts/e200-preregistration.md` 第 12 行）。
取号：`bash scripts/kb/claim-experiment.sh --next` 给的是 E991，原因是 `research/prompts/d137-ledger-defs-r*-opus-model/worlds/*/after/research/prompts/e990-preregistration.md` 这类三方腿模型里的样本文件也被当成用过的号（第十三节命令与输出）；E201 是 E200 之后第一个没用过的号，`claim-experiment.sh E201` 核过两棵工作树里用过的 200 个号之后占住。`--next` 被样本文件抬高这件事交主 agent。

主 agent 给的问题（派发提示「关的是哪几条」原文逐字）：

> 1. 写这一个新实验的跑前登记（取新实验号还是作 E171 的第 3 次重跑，你照编号规矩判、写明依据），照问题单 Q1、Q2。硬约束：不继承 r1 修订 8 的冻结层（不拍 r1 那种冻结树、不要修订 6 树与它的任何产物、不要 H1–H34 那一套占位）；照今天主工作区的实现与今天的两张名单（278 行）跑；与 r1 的比对只读 r1 留在 git 里的产物（写清取哪几份、按什么键逐格对）；装置照今天的 `scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/` 与入库装置能复用的复用，不能复用的写明为什么。验收：登记写进 `research/prompts/`；岔路表每行写出够判要量什么；跑前登记的形式检查命令写进报告。
> 2. 跑法照现在的规矩写进登记：线程、内存、机器、卡都由统一任务服务定，任务里面不套任何包装；要两台的走服务的分片三步；同一条任务里同时起的几个单元在进程内分 `SINGLEFS_THREAD_CAP`；要存判定的写进 prune-server 那一份判定库。推挂钟与资源量（推的写推的、写依据）；重型的标重型。
> 3. 写明它与 E171 第 2 次重跑的关系：Q1 撤项那一格已判甲，这一次不重判；E171 第 2 次重跑问题单 Q1「够判条件字面的后半」由这一次接。

问题单 `research/prompts/e171-current-tree-full-table-questions.md`（sha256 `7215e53468d574966650b72985faa2edcd4819c31ba45e1a5ca69f4fd673cf29`，12 行）第 5–8 行逐字（`sed -n '5,8p'` 原样），就是本登记的岔路单；第 10、12 行（起因、不问）照读：

| 问题 | 候选 | 翻面观测 | 够判条件 | 状态 |
|---|---|---|---|---|
| Q1：今天主工作区的实现上，E171 两张坏实现名单（`bad-implementations-step-8.tsv`、`bad-implementations-step-4-only.tsv`，合计 278 行）逐行过三个 oracle，三方合谋的判定与 r1 那一次逐格比，变了哪几格 | 没有一格变；有格变，且每一格都归得到这期间改过的某一处实现或装置；有格变，有的格归不到 | 今天这一趟逐行判定与 r1 逐行判定并排，判定不同的格各自对应的改动 | 278 行在今天的树上跑完、与 r1 逐格并排完、判定不同的每一格写出归到哪一处改动或写明归不到 | 开着 |
| Q2：今天这一趟里 O1（内存模型对拍）、O2（独立解析与 checker）、O3（独立规范执行器）三列各自漏判的行，与 r1 比多了还是少了 | 三列都不多；有一列多了 | 三列各自漏判的行集合与 r1 的差 | 同 Q1 的够判条件 | 开着 |

> 起因：`records/2026-10-31-删兼容旧版本代码.md` 第 17 批——E171 第 2 次重跑的现状整表段整套沿用 r1 修订 8 的冻结层，起跑前的核对依赖造不回来的 r1 输入（修订 6 树、修订 6 第二段行输出根等，第 2 次重跑登记修订 9 第 9.3 条）；用户 2026-11-02 定「第 2 次重跑收尾，整表另立新实验」，原话里写「在今天的树上整表重跑另写一份不继承 r1 冻结层的跑前登记，先设计、后跑」。
> 不问：r1 那一次的读数对不对（不重判旧产物）；撤掉第 236 行之后判定词变不变（E171 第 2 次重跑已判甲）。

问题单就是岔路单：第六节每个量写明它对应 Q1、Q2 哪一行、取什么值让那一行翻面；对不上的不登记，顺带量的标「附带，够判后不跑」。

### 读法写死

1. **「今天的树」**：主 agent 交这一条任务、服务起分片计划那一步的那一刻，主工作区的 `Cargo.toml`、`Cargo.lock`、`.cargo/`、`crates/`（不带 `target`，含已暂存与未暂存的改动），连同 `crates/mutations.tsv`。计划那一步把它们拷进这一件的计划目录（服务给的 `TASK_SERVER_SHARD_PLAN_DIRECTORY`，在共享工作根下，两台同一个绝对路径），各片都从这一份同步出自己的线道树，合并那一步做完连同各片产物目录由服务收掉。计划那一步记：快照摘要与份数（`row_driver.snapshot_digest` 的算法）、`git rev-parse HEAD`、`git status --porcelain -- crates Cargo.toml Cargo.lock .cargo` 全文、`crates/mutations.tsv` 与两张名单的 sha256、入库装置文件的 sha256、第五节 5.3 点名的研究一侧各文件的 sha256。
   - 这不是 r1 那种冻结树：没有宣布行、没有拍树前对合后清单的核对、没有 H1–H34 占位、没有对 `ab35105f…` 的块表、不在 `scripts/experiment-apparatus/target/` 下留一棵长住的树；这份拷贝只活在这一次提交里，用处只有两个——各片编的是同一份实现、它的摘要进复用键（读法 10）。哪一刻交由主 agent 定，不另设宣布。
   - 主工作区里有别的会话没提交的改动时照样拷（「今天主工作区的实现」照字面），全文记进产物；拷进来的树编不过走停机 S6，由主 agent 另挑一刻重交。
2. **「r1 那一次」与它留在 git 里的产物**：「r1」指 E171 第 1 次重跑修订 6 树那一趟（第二段整表与第一段十六行）。只读下面几份，按 sha256 与行数核过才读（作废 Z5）；修订 6 树、它的导出根、行输出根、二进制与任何不在 git 里的东西都不要，派发提示「不要修订 6 树与它的任何产物」照这个意思读：跑的时候不拿它们当输入，比的时候只读 git 里留下的这几份文字产物。
   - **P1** 第二段比对器产物 `research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-comparator-r6.out`（sha256 `d0bc79ca3f580774cfd4bad2fd672006b98c8a5d02063c48fb9915cb246a8549`，451 行，HEAD 里在、工作区与 HEAD 相同）：流一、流二，281 行整表，不跑执行器（没有 O3 列）。
   - **P2** 第二段逐行驱动合成产物 `research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-rows-driver-r6.out`（sha256 `59c89b96c5bef3151c8b4885d98f8fe4db680a300106a9de400af4188cf74e7c`，565 行）：每行一条 `row row_number=…`（281 条），带 `status`、`device_exit`、`substituted_from`、`mutated_row_name`、`mutation_line_sha256` 等字段。
   - **P3** 第一段比对器产物 `research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-comparator-resume2-r6-r3-2026-10-07.out`（sha256 `40ee800a8681caf503c05131fbd05fcdba7a9514a38178660c55195330b1cabc`，811 行）：空变异加十五行，三条流，带 O3。
   - **P0**（只给归因用）新树产物 `git show 04ceaa904^:research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-frozen-tree.out`（sha256 `b720e5d28dcaead6e7c7262fd82185856e869385dad30674bf9689cb658f24c6`，125 行）：只取它记的 `git status` 那几行的路径（r1 那棵树在提交 `31e47115471ba5502a4531d25dffcc921be36899` 之外多出的未提交改动，E200 登记读法 1 说是 94 行），不读别的行。另读 `scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/revision_8_research_side_of_the_revision_6_run.json` 的 `files`（r1 那一趟研究一侧各文件的 sha256），只给归因 G7 用。
   - P1、P3 每一行都以 `E7RESULT name=<行种>` 起头；P1 的行种是 `baseline_stability config done driver_product driver_product_rows not_located_row q2 q2_named_row q2_named_row_call q2_observation q2_set_call q2_set_row q3_denominator q3_denominator_class q3_row_125 q3_self_crash_row recovery_panic_sum row substitution unstable_memory_objects untouched_row_red verdict`，P3 的是 `a1 a11 a1_empty_mutation a1_empty_mutation_touch a8 config done o3_third_stream_coverage q1 q1_row q2_named_row q2_named_row_call q3 q4 q4_counter_branch_state q4_trajectory q4_trajectory_by_inode_creation q4_trip q5_judged q5_trajectory q5_trajectory_cell verdict z1 z1_unstable_memory_objects`（第十三节命令，只取了行种名）。
3. **三个 oracle 与各列**：照 r1 读法 1（第二节整抄）：O1 三个子列（O1-对拍、O1-模型崩溃判官、O1-RefFS），O2，O3 严格与暂代两种跑法。O3 是 D13（验证路线） 已定项 6「实现今天的样子」点名的现行那一份 `scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates_journal_header_487/`，冻结行照 `frozen_o3.py` 第 41 行起那几个常量（`research/results/e170-r2-o3-spec-executor-2026-10-04-s1b-executor.out` 第 676–692 行），与 r1 用的是同一版，O3 列两边可比；每片开头与收尾 `frozen_o3.freeze_differences` 各核一次（作废 Z2）。O2 今天在装置里经给定状态引擎判（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs` 第 7036–7050 行），多机配置开着 GPU 时在卡上判，这一处是 r1 之后装置的改动，差出来的 O2 格照读法 8 的 G5 归因。
4. **历史与崩溃状态**：只用流一（`new_pool_file_creation`）与流二（`fixed_script_through_e_then_unmount`），批 1 定判、批 2 只进第八节，取样规则照原登记读法 2。不带可写挂载探针、不跑「已确认的丢了」那一形（r1 的 Q4、Q5 不在问题单里）。流三只在第二段（读法 11）。
5. **红、触达、分母、O1 够不到的地方**：一字不改照 r1 读法 3、4、5、6 与原登记读法 3、4、5（第二节整抄），类由今天的 `segment_two_comparator.py` 现算（读法 7 的 T1）。
6. **名单**：今天的两张名单 `bad-implementations-step-8.tsv`（115 行）与 `bad-implementations-step-4-only.tsv`（163 行），两个目录逐字节相同，合 278 行；整表编号不重编（E171 第 2 次重跑登记读法 4），行集 = {1, …, 281} 去掉 {33, 107, 236}；第 70、141、168 行第 5 列是改引之后的名字（那份登记修订 1 第 1.5 条）。按名字取行、锚点核对、补位照原登记 5.2（第二节整抄）与 `bad_implementation_table.py` 第 115 行 `locate_by_name`、第 159 行 `substitute_for` 今天的写法。空变异照原登记 5.6：表里第一行把替换文换成原文本身（行号记 0）。

7. **今天这一趟出哪几份、按什么键与 r1 逐格对**：
   - **T1** 今天的第二段比对器产物：今天的 `segment_two_comparator.py`（不改）读今天的基线、行输出根与逐行驱动合成产物，出与 P1 同形的行；**T2** 今天的逐行驱动合成产物，与 P2 同形；**T3** 七行的第一段形态：照 `segment_one_comparator.py` 第 227 行 `q1_lines` 与第 262 行 `row_column_lines` 原样调（import，不另写判法），对今天整表行输出根里第 15、23、42、242、269、270、271 行出 `name=q1`（`streams=two_streams` 那几行）与流一、流二的 `name=q1_row`；`q2_named_row`、`q3` 两种照同一份文件里产出它们的函数调，只取流一、流二。
   - **配对的键**（一种行种一组字段；对不上键的行记「只在一边」）：P1/T1 的 `row`、`q2_set_row`、`q2_named_row`、`q3_self_crash_row`、`not_located_row`、`substitution`、`untouched_row_red` 按 `row`；`q2_set_call`、`q2_named_row_call` 按（`row`, `stream`, `call`）；`q3_denominator` 按（`reading`, `batch`）；`q3_denominator_class` 按（`reading`, `batch`, `class`）；`q2`、`q2_observation`、`q3_row_125`、`verdict` 各一行。P2/T2 的 `row` 行按 `row_number`。P3/T3 的 `q1` 按（`reading`, `streams`, `batch`），`q1_row` 按（`row`, `stream`, `batch`），`q2_named_row` 按（`row`, `streams`），`q2_named_row_call` 按（`row`, `stream`, `call`），`q3` 一行，`verdict` 一行。
   - **格分三类**（执行员在今天任何产物出来之前，只读 P1–P3 的字段名，照下面的规则把每个（行种, 字段）归好类，写成一张表进第十二节修订、记 sha256，之后不改）：
     - **判定格**：①取值来自封闭词表的字段（`true` / `false`、类名 `category_*` 与 `class`、`status`、判定词 `verdict`、`q1`、`q2`、`q3` 这类）；②带键的行在不在（一边有、一边没有本身是一格）；③列行号的字段按集合比（`all_quiet_rows`、`invalid_rows`、`row_numbers`、`rows_o1_red` 这类）；④红的计数（`o1_comparison_red_calls`、`<列>_red`、`untouched_row_red` 的各列、`o1_red` 的计数）取「是否大于 0」当判定格。
     - **钉值格**：其余数值、计数本身、峰值、为正段数、期末值、单点数、sha256、行名与补位名。
     - **不比**：`config`、`done`、`baseline_stability`、`unstable_memory_objects`、`driver_product`、`driver_product_rows`、`recovery_panic_sum`，与任何字段里的路径、趟名、产物名、耗时；P3 的 `a1*`、`a8`、`a11`、`z1*`、`q4*`、`q5*`、`o3_third_stream_coverage`（阳性对照、稳定性、可写挂载、「已确认的丢了」那一形，不是三方合谋的判定）；P3 里流三的行与 `streams=three_streams` 的行（第二段才比，读法 11）。
   - 「一格变了」= 同一个键、同一个判定格，两边取值不同（集合按集合相等比，计数按「是否大于 0」比）；只在一边的键算那一行的「在不在」一格变了。钉值格的差另报，不进 Q1 的判定（与 E171 第 2 次重跑读法 2「判定词」「钉值」的分法同）。
   - 今天有而 r1 没有的格（整表 O3 两列、整表每行每列的红态数与轨迹、读法 9 的整表漏判集）记「今天独有」，照报，不进 Q1、Q2 的判定。
8. **归因**（问题单 Q1「每一格都归得到这期间改过的某一处实现或装置」）：判定格变了的每一格，逐条按下面七种查，查中的全列（一格可中几种）；一种都不中记「归不到」。参照树取提交 `31e47115471ba5502a4531d25dffcc921be36899`（r1 那棵树的底，E200 登记读法 1），P0 列出的那几份路径在 r1 树里不是这一版，查到它们的那一条另标 `reference_approximate`。
   - **G1 名单改了**：那一行在 {33, 107, 236}（撤）或 {70, 141, 168}（改引）里。
   - **G2 坏实现的行文变了**：今天那一行改坏用的变异行（文件、原文、替换文）的 sha256 与 P2 那一行的 `mutation_line_sha256` 不同。
   - **G3 补位不同**：P2 与 T2 那一行的 `substituted_from`、`mutated_row_name` 或定位结局（`not_found`、`anchor_not_unique`、`compile_failed`）有一样不同。
   - **G4 坏实现落点所在的函数变了**：用 `scripts/gate/lib_function_units.py` 切 Rust 单元，今天锚点所在的那个 `fn` 单元，在参照树里同文件、同限定名的单元指纹不同或不在。
   - **G5 装置算这一格的函数变了**：执行员在今天任何产物之前读入库装置，写一张「行种与字段 → 装置里算它的函数」的表（进第十二节修订，记 sha256）；那几个函数在 r1 的装置与今天的装置之间单元指纹不同。r1 的装置照 E200 登记读法 4 的 T4 取（`f8a9ddaab` 那一版删掉第 3478–3484 行），sha256 等于 `67e7a5498425ec3f27379ba8f0e5cd666faaeaac0817006d6938956d6419a156` 记 `exact`，不等记 `approximate_device`。
   - **G6 那一列 oracle 的入口函数变了**（只对列的格）：O1-对拍取 `crates/singlefs-harness/src/model.rs` 的 `judge_and_advance` 与这一行两条流用到的 `answer_*`、`crates/singlefs-harness/src/model_comparison.rs` 里造观测的函数；O1-模型崩溃判官取 `crash_recovery_disagreement`、`committed_versions`；O1-RefFS 取 `crates/singlefs-reference-filesystem/src/lib.rs` 的 `submit`、`allowed_set_for_a_crash_in`、`crash_and_recover` 与 `crates/singlefs-harness/src/reference_filesystem_translation.rs` 的 `events_of`、`crash_moment_of_a_segment_closed_at`；O2 取 `crates/singlefs-checker` 的 `check_pool_image`；O3 由冻结行定，不归这一种。这些单元本身（不收闭包）的指纹在参照树与今天的树之间不同或不在。
   - **G7 研究一侧算这一格的函数变了**：同 G5，表换成「行种与字段 → 比对器与 `collusion_matrix.py` 里算它的函数」，r1 那一版按 `revision_8_research_side_of_the_revision_6_run.json` 的 sha256 在 git 历史里找（找不到记 `research_reference_missing`，那一格这一种算查不了，走 F6）。
   - **宽归因（描述，不进判定）**：「归不到」的格另报那一行坏实现所在文件、装置入口 `main` 的单元闭包、那一列 oracle 入口的单元闭包里指纹变了的单元数。参照树到今天 `crates/` 下改了 792 份文件（第十三节 `git diff --stat`），按闭包查几乎每一格都会中（推的），所以闭包那一层只描述、判定照上面七种。
9. **Q2 的「漏判」与比的范围**：
   - 一行在 X 列「漏判」= 这一行在登记读法（`category_registration_batch_1`）下计入分母，且 X 在这一行流一、流二批 1 上一处都没判红（O1 = 三个子列合起来，任一子列红算红；O3 严格只数「不在」，暂代另一列）。判定列五条：O1、O2、O3 严格、O3 暂代各一条判定，O1 三个子列各自另报（描述）。
   - r1 一侧在 git 里有逐列读数的只有 P3 的 `q1_row` 那几行（`segment_one_comparator.py` 第 31 行 `Q1_ROWS` 七行，记作 R7 = {15, 23, 42, 242, 269, 270, 271}）；P1 只对未触达的行报各列（`untouched_row_red`），计入分母的行没有逐列读数，整表没有 O3。所以 Q2 的比只在 D 上做：D = R7 里同时满足四样的行——P3 有它流一、流二批 1 的 `q1_row` 行；P1 里它 `category_registration_batch_1=counted`；T1 里它同样 `counted`；T3 有它流一、流二批 1 的 `q1_row` 行。
   - 每一判定列：r1 漏判集 M_r1 与今天的 M_today 都只取 D 里的行；**多了** = M_today 里有一行不在 M_r1 里（问题单翻面观测「漏判的行集合与 r1 的差」照集合读）；**少了** = M_r1 里有一行不在 M_today 里；两样各报，个数之差另报（描述）。
   - 整表上今天每一列的漏判集照报（「今天独有」，读法 7），r1 那一边写「git 里没有逐列读数」。这一点进下面「未定项」。
10. **判定存在哪、复用怎么键**：逐行的结论存进 prune-server 那一份判定库，族 `apparatus-items`、种类 `three_oracle_collusion_full_table_on_the_current_tree`（`scripts/prune-server/prune_server_client.py` 第 226 行 `FAMILY_APPARATUS_ITEMS`、第 1032 行 `library_store`），不建本地库目录。一条存的是那一行的 `row-result.tsv`、`o3.out` 与两条流导出目录里除三个大文件（`base-sectors.bin`、`ops-payload.bin`、`torn-payload.bin`，逐行驱动不给 `--keep-dumps` 时本来就删）之外的全部文件；比对器在只放这一份的行目录上与在完整导出上算出的各格要逐字相同（第七节 A7）。键是下面各项的 sha256：读法 1 的快照摘要、`crates/mutations.tsv` 的 sha256、那一行的整表编号与名单那一行全文、实际改坏用的变异行的 sha256、装置参数（`row --streams 1,2`）与编装置用的特性（`gpu`）、`rustc -vV` 与 `cargo -V` 全文、给定状态引擎这一趟的判器路线（GPU 或 CPU）、逐行驱动与 O3 一步的研究一侧文件 sha256、O3 冻结行的摘要。基线、空变异、阳性对照与跨机对照那几行每次现跑、不入库。复用的行照 `.claude/rules/test-script-discipline.md`「判定可复用：按输入存，按前缀剪」抽哨兵现跑（第六节「复用账」）。
11. **两段**：第一段是整表 278 行（加补位）在流一、流二上，补 Q1 与 Q2 在流一、流二那一半；第二段是空变异加 r1 第一段的十五行（`STEP_ROWS` 之并，去掉今天不在名单里的行）在三条流上，只补 P3 里流三那几格（`q1` 的 `streams=three_streams`、`q1_row` 的流三、`q2_named_row` 的三流那几行）。第二段跑不跑由主 agent 照第一段交回的岔路表定（「未定项」第 2 条）。
12. **与 E171 第 2 次重跑的关系**：
   - E171 第 2 次重跑问题单 Q1 撤项那一格（撤掉第 236 行之后原登记 F4 扩表、补位与 r1 第二段三份读数上的判定词变不变）用户已定甲，这一次不重判、不读那三份撤项核算产物、不改它们。
   - E171 第 2 次重跑登记修订 9 第 9.2 条「Q1 够判条件字面的后半（整张名单在今天的树上跑完，与 r1 逐格对比并归因）这一次不做，留给整表另立的新实验」——由这一次接：本实验问题单 Q1 的够判条件就是那一句在今天的树上的落点（278 行跑完、与 r1 逐格并排、差的格逐格归因）。
   - 这一次的读数不进 E171 第 1、2 次重跑的任何判据，不改它们的判定、不触发它们的条款；r1 的读数也不进这一次的判据，只是逐格比的另一边（E171 第 1 次重跑登记修订 8 第 8.5 节第 3 条「谁也不进谁的判据」同形）。两边判得不同的格并列写、各带树名（今天的快照摘要与 r1 的 `7726cdd6…`），差别归到哪一处是读法 8 的机械归类，要当推论用另走三方。

未定项：
1. **Q2 在整表上没有 r1 那一边**（量：整表每列的 r1 漏判集；缺：P1 不报计入分母的行的逐列读数，r1 第二段不跑 O3，行输出根已不在，派发提示又只许读 git 里的产物；谁定：主 agent）。这份登记把 Q2 的判定限在 D（读法 9）上，整表只报今天那一边。主 agent 要么认 D 这个范围，要么把问题单 Q2 改问法（例：整表那一半记「r1 一侧没有读数」不判）。在主 agent 定之前执行员可以照常写装置与跑第一段（判定只在第一段比对之后才出，D 上的判法已写死）。
2. **流三那几格算不算 Q1、Q2 的够判范围**（量：P3 里流三的判定格；缺：问题单只说「278 行」，没说流；第二段推 4–6 小时，第十四节；谁定：主 agent）。这份登记默认第一段交回时 Q1、Q2 在流一、流二上够判，流三那几格写「第二段未跑」，主 agent 定要不要第二段。

## 二、被测条款与它引的定义

被测条款是 E171（三oracle合谋测试） 原登记与第 1 次重跑登记里三个 oracle 的判法与坏实现名单的判定口径、第 2 次重跑登记修订 9；它们的根是 D13（验证路线） 已定项 6。用 `scripts/kb/quote-kb.py` 整段抄六段进草稿目录 `quotes.md`（工具回读逐字节一致），追加在下面：D13 已定项 6 全节；原登记 `### 读法写死` 与 `### 5.2 坏实现表：…` 两节（原登记已归档，先 `git show 31128273b^:research/prompts/e171-preregistration.md` 落到草稿目录、核 sha256 等于 r1 登记第 4 行记的 `b5f47dba…` 再抽；抄完把两处「出处」行里的草稿目录路径换成 `git show` 的写法，换法在第十三节）；r1 登记 `### 读法写死`；r2 登记 `### 读法写死`（今天名单的撤项、改引与编号）与 `### 修订 9` 全节。r1 登记第五至十一节（臂、阳性对照、判据、作废与停机）太长不抄，这一次照用的几条在第五至十一节逐条点名行号（r1 登记 sha256 `5d8fb709e32e0b61df009eb5fd71d2f01285e4eb8bf5ba3dca07d50a141ebf21`）。

**出处 `.claude/kb/decisions/13-验证路线.md:172-202`（整段抄，未转述）**

```markdown
#### 已定项 6：三个互相独立的 oracle

**定案**：三个 oracle 各判一样东西，独立性各有来处：

<!-- doc-lint:registry name-col=2 -->

| # | oracle | 判什么 | 独立性来自哪里 | 验不到什么 |
|---|---|---|---|---|
| O1 | 内存模型对拍（HashMap 理想文件系统） | POSIX 可见语义：内容、大小、目录结构；无崩溃处的内容期望归它，崩溃之后允许看到的集合归 RefFS（已定项 3） | 不碰盘、不解析磁盘格式 | 磁盘结构是否自洽；崩溃语义；**两份实现对同一条 POSIX 语义的共同误解** |
| O2 | 独立解析器 + checker | 磁盘字节是否满足 I-1（块自描述）~I-6（加密不变量） | **checker 不得调用被测 crate 的任何反序列化/遍历代码**，必须重新实现一份解析，直接读裸设备 | 只判结构自洽：一个块被覆盖成另一份**合法**内容，Merkle 链照样自洽、记账照样对 |
| O3 | 独立规约执行器 | 崩溃点截断后的每个磁盘状态是否落在规约允许的集合里 | **用另一种语言写**（不用 Rust），只照 decisions 与 invariants 的文字重新推导；第一版用 Python 单写（E170（O3规约执行器第一版）） | 规约本身可能写错——**必须配阳性对照才有意义**；被投影的六样的取值与六样之外的根记录字段、实现自己的恢复与读者代码、系统配置槽的见证值（journal_tail）、一次发布内各写的先后 |

**独立性要被验证，不许自称**：设计一个「三 oracle 合谋测试」——拿一个精心构造的坏实现
（例如 nonce 永远为 0）去过三关，若三者全部放行，说明存在共同盲区，回头补 oracle。
功能正确性不只押在这三个 oracle 上：O1（内存模型对拍（HashMap 理想文件系统）） 那一行已把崩溃之后允许看到的集合交给 RefFS（已定项 3），恢复函数的 M2–M4 落在 Verus 与 E165（恢复重放有界穷举与变异杀伤界）（已定项 1），需要第二个输入的核对归记录核对器（已定项 7）；三个 oracle 之间的共同盲区靠「三 oracle 合谋测试」找。

**射程**：三个 oracle 各自验不到什么写在表的最后一列；O2（独立解析器 + checker） 的定义域归已定项 7，需要第二个输入的核对归记录核对器。**实现今天的样子**：O2（独立解析器 + checker） 在 `crates/singlefs-checker`（只依赖格式常量 crate）；O1（内存模型对拍（HashMap 理想文件系统）） 的形态是 `crates/singlefs-harness/src/model.rs` 的理想模型（只住内存、只用 `singlefs_format` 的常量、不建崩溃），门禁 harness-model-differential-and-scenarios 拿随机历史逐步对拍；O3（独立规约执行器） 是只照 kb 文字写的 Python 执行器，现行那一份是 E171（三oracle合谋测试） 第 1 次重跑用的 `scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates_journal_header_487/`（E170（O3规约执行器第一版） 第 2 次重跑从第 1 次重跑冻结的 `scripts/experiment-apparatus/e170_second_run_o3_spec_executor/` 拷出 `scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates/` 加谓词 W、T，再跟上 487 字节 journal 记录头；第 1 次重跑冻结的那一份与第一次那一份 `scripts/experiment-apparatus/e170_o3_spec_executor/` 留档）：严格跑法判得出（第一次导出的批 1、批 2 上 0 态撞「规约欠一句」），与实现逐态一致（两边全放行，不当它判得对的证据；「择到的根不是自证根里 txg 最大的那条」「③ 收到不少于 2 槽」两条路径批里 0 态，C661（规约执行器批里两条路径没有态）），坏状态表照改正的期望重判有判别力（阳性对照与盲测都判对），还没接进崩溃点重放；判一个崩溃态的做法：照规约的恢复规则（择系统配置、择根、前缀按 D23（journal 的角色与格式） 已定项 14 的五条口径）从镜像自己恢复出一版，恢复不出（整池拒或没有自证根）判不在；恢复出的这一版的投影（实例代号、checkpoint_txg、树表单元指针、中央映射树根指针、树 ID 水位、F）要等于版本表里某一版的投影，且这一版的下标在 [已确认, 已开始] 之间。版本表是基镜像上自己恢复出的现行那一版，加录制流里每一次根槽写按流序各一版，每版的投影取实现写进根槽的字节；已确认取崩溃态里落了的最晚一条写之前已返回的最晚一版（只从落了的写推），已开始取落了的写所属的最晚一版。系统配置槽只用来择槽、取根环与 journal 环的几何、判池级字段越界整池拒。所以恢复规则那一半是照 kb 文字另写的，允许集合那一半取自实现自己写出的根；RefFS（已定项 3）在 `crates/singlefs-reference-filesystem`，崩溃注入与故障注入的调用点已接；「三 oracle 合谋测试」是 E171（三oracle合谋测试），读数取自一棵冻住的树（快照摘要见实验页，与现行 `crates/` 不是同一份）：O1（内存模型对拍（HashMap 理想文件系统）） 一列是模型对拍加模型的崩溃判官，不接 RefFS，与表里 O1（内存模型对拍（HashMap 理想文件系统）） 那一行不同（崩溃之后的集合那一半 E171（三oracle合谋测试） 没用 RefFS）；O3（独立规约执行器） 一列用 E170（O3规约执行器第一版） 第 1 次重跑冻结的那一份；手造坏镜像那一族只喂 O2（独立解析器 + checker）、O3（独立规约执行器），期望出自 E170（O3规约执行器第一版） 的设计员，Q2 照登记字面判「有」（4 张全是 P8），按登记 F3 判成 E170（O3规约执行器第一版） 推期望推错、这四张不是坏镜像，真坏镜像里 O2（独立解析器 + checker）、O3（独立规约执行器） 两条全放行的 0 张；三条的源头是否独立（表第 4 列）E171（三oracle合谋测试） 只在 Q2 量过 O2（独立解析器 + checker）、O3（独立规约执行器） 一对，那一对的补丁与期望同出 E170（O3规约执行器第一版） 的改法代码，不当独立读数，O1（内存模型对拍（HashMap 理想文件系统）） 不在 Q2 里；没有量到的不写成「互相独立」被验证过。

**三条 oracle 都不判的两样**（第一样由崩溃注入与崩溃点重放两侧判，崩溃点重放那一侧的观察者在实现批）：一是无读故障的崩溃态上，D23（journal 的角色与格式） 已定项 14 可写挂载那条见证判据为真（系统配置槽里的见证值超前于盘上读得出的记录）：崩溃注入第二截判拒绝合不合法时只在崩溃镜像上复算这条判据，为真就判合法（挂载透明的射程只管不带故障的状态，已定项 1）；要有判官问「这个崩溃态没注入读故障，判据凭什么为真」：崩溃注入第二、三截在没注入读故障的崩溃态上复算这条判据，为真即判拒得不合法（`crates/singlefs-checker-tier/src/crash_injection.rs` 的 `RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults`，C670（无读故障的崩溃态上可写挂载见证判据为真没有判官） 已还清，见欠账表已还清表）。「无读故障的崩溃态上每块盘至少一份自证过的系统配置」这一性质由崩溃点重放 mkfs 返回之后各条流上的观察者判：观察者按流登记，记录 mkfs 的流不挂，只记相对基镜像新出现的、一份自证过的系统配置都没有的盘，撕裂域在内，判槽自证读池级 checker 的逐盘数（`crates/singlefs-checker/src/image.rs` 的 `system_configuration_redundancy_report`），另有一条登记完整性用例判含系统配置槽写段的流都挂了它（实现在实现批）。随机历史里补写闸（D22（单元原子性怎么合成） 已定项 8）不过的那一形，由崩溃注入第二、三截的 G4 拒绝那一臂判；补写闸过了的那一形，随机历史里没人判，正确实现留不下这种盘面（两次抽样都没打中）。mkfs 返回之前的崩溃态不在观察者与 G4 拒绝那一臂的射程：mkfs 逐盘两槽在同一个循环里写、之后才逐盘屏障，崩溃注入的候选段与 E171（三oracle合谋测试） 的基镜像都取在 mkfs 之后；G4 拒绝那一臂与 C670（无读故障的崩溃态上可写挂载见证判据为真没有判官） 那条还有一处不同形：择不到系统配置时那条的复算交 `None`、G4 按真，这时读阶段本身已拒、到不了这一格。二是一次发布内各写的先后：O2（独立解析器 + checker） 的 I-7.12（系统配置 F 不低于同盘根上的 F） 判抬 F 那一串先写系统配置这一步；变异点名的用例只判 E171（三oracle合谋测试） 第 270 行那一种（发布返回之后每个崩溃点的见证值），第 242 行点名的用例判的是失败路径冻结；`layout/01-first-txn.md` 第八节把根槽 FUA 与系统配置槽轮换登记成两段，只是登记在册；其余写序不在三条之内。

**依据**：
- E170（O3规约执行器第一版）：O3（独立规约执行器） 照「另一种语言、只照 kb 文字重新推导、配阳性对照」写得出第一版，判得动单个崩溃状态、一态够进快档；第 1 次重跑（照 2026-10-03 的 kb 改过一次的执行器）严格跑法判得出、与实现在取样的批态上一致、阳性对照有判别力（第一次跑判不出与判别力不足那两笔留在实验页）。
- G4（无读故障的崩溃态上每块盘至少一份自证过的系统配置）：代码轮判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 j2-b 立、第 29 批实现，射程与 mkfs 那一段见 `research/prompts/m3-formal-code-b29-r1-main-verification.md` 第二节 L3 与第三节第 3 条。
- 两槽全废盘面改由崩溃点重放 mkfs 返回之后各条流上的观察者判、G4 拒绝那一臂留着：三方判决 `research/prompts/m3-close-g4-rewrite-r3-main-verification.md` 第二节 Y2 各行与第三节（前两轮 `research/prompts/m3-close-g4-rewrite-r1-main-verification.md`、`research/prompts/m3-close-g4-rewrite-r2-main-verification.md`）；观察者的接法（按流登记、相对基镜像、登记完整性用例）是用户定案 2026-10-22「按流登记+相对基镜像」，零轮，原话在变更史。数在三方腿的模型与副本原型上量，没立实验号。
- 三个 oracle 的分工与各自独立性的来处是验证体系的结构设计；独立性靠「三 oracle 合谋测试」观测（E171（三oracle合谋测试））。
- E171（三oracle合谋测试）：在一棵冻住的树上，坏实现表里有三条 oracle 全部放行的行、有只靠一条才抓到的行，O3（独立规约执行器） 在坏实现表上一行没红；手造的伪造根、伪造续接镜像里有 O2（独立解析器 + checker） 放行而 O3（独立规约执行器） 判不在的（读数与限定在实验页 Q1、Q2、Q3 三行）。
- 三方论证：D13（验证路线） 新立那一轮（本地 LLM / Sonnet / Opus 各一条腿，立场不重复），材料没有留存，经过在变更史。
- O1（内存模型对拍（HashMap 理想文件系统）） 与 RefFS 的分工句：三方判决 `research/prompts/m3-formal-r5-main-verification.md` 第二节另起与单独一个包那一格（两腿同向）。

**欠**：C445（模型对拍的取样点不随写路径增长）；C447（规约执行器没接进崩溃点重放与 RefFS 比对）；C407（三 oracle 合谋测试的判官没进门禁）：E171（三oracle合谋测试） 已量过一次，没有一格门禁读它的产物；C672（交回值或内存状态错而写流不变的一类）；C679（卸载记号没进模型对拍）；C874（补写那一支进不了崩溃测试、运行时看不出）：两槽全废盘面的观察者与 G4 拒绝那一臂改读池级 checker 都还没实现

```

**出处 `git show 31128273b^:research/prompts/e171-preregistration.md` 第 29-48 行（整段抄，未转述）**

```markdown
### 读法写死

1. **三条 oracle 与各自的入口**（「对同一批状态出结论」那一句的落点；行号在第三节）：
   - **O1** 分两样，分列报、合成一列判：**O1-对拍**——每次 API 调用之后拿模型的答案与实现的结局比，入口 `IdealModel::answer_*` 加 `IdealModel::judge_and_advance`（与 `crates/singlefs-harness/src/history.rs` 的 `judge_by_model` 同一种比法，观测由 `crates/singlefs-harness/src/model_comparison.rs` 的胶水造，不另写比法）；**O1-崩溃判官**——每个崩溃状态上，实现只读恢复（看 journal）读回的那一版交给 `crash_recovery_disagreement`，模型那一侧给每一步之后并起来的 `committed_versions`，「盘上已持久的最新根槽」由 `newest_persisted_root` 从截断的写流数出（与 `crates/singlefs-checker-tier/src/crash_injection.rs` 调它的那几行同一种交法）。O1 在一行上算红 = 两样里任一样判出一处对不上。
   - **O2**：池级 checker `check_pool_image` 判这个崩溃状态的崩溃镜像（恢复之前那一份，与层 0 的 `judge_crash_image` 判的是同一份），任一条不变量 `Violated` 算红；`NotApplicable` 不算红。
   - **O3**：E170（O3规约执行器第一版） 的 Python 执行器读本实验装置按 E170 登记 5.3 写出的中性格式，一态出两个判定：严格跑法（「在 / 不在 / 规约欠一句」）与暂代跑法（「在 / 不在」）。执行器是哪一版见第 8 条。
   - 记录核对器（`check_records`）与层 0 照手写版本表判的 oracle（`crates/singlefs-checker-tier/src/crash.rs` 的 `classified_oracle_violation_for_versions`）**不是三条之一**：D13（验证路线） 已定项 7「故意不给它编号」，层 0 那份 oracle 的版本表由调用方手写、不是 `model.rs`（`model.rs` 第 2246 行上方注释原文「与层 0 的 `crash::oracle_violation_for_versions` 分开写」）。两样照样算、另列（附带），不进 Q1–Q3 的判定；「三条全放行而它们红」的行单列出来交主 agent。
   - D13 已定项 6 表的 O1 行今天写着「无崩溃处的内容期望归它，崩溃之后允许看到的集合归 RefFS（已定项 3）」：崩溃判官判的正是崩溃之后的允许集合。本登记照派发提示把崩溃判官算进 O1；另用同一张矩阵把「O1 只取对拍」那一种重算一遍（附带，第六节 Q1-丙），两种结论都交主 agent。
2. **同一批历史与崩溃状态**：历史取 E170 那两条层 0 流——新池新建文件（流一，流名 `new_pool_file_creation`）与固定脚本到 E 再正常卸载（流二，`fixed_script_through_e_then_unmount`），驱动照 E170 导出装置的同名驱动。崩溃状态取 E170 登记 5.4 的批 1 取样规则（每段八个点、全持久态、每段至多 4 个带撕裂的态）作用在**这一行自己录下的写流**上：坏实现改了写流，状态的字节随之变，「同一批」指同一段历史、同一条取样规则，不指同一份字节。批 2（E170 5.4 的段内居中点）是几何取样点（第八节）。
   - 为什么取这两条流、不取崩溃注入的随机历史：O3 只在这两条流上跑过、导出格式只为它们写过；崩溃注入的历史里有冷启动、崩溃恢复抛弃根这类 O3 从没见过的调用，O3 在那里报「不在」分不清是执行器没写到还是坏实现被抓（推的，没量）。
3. **一行的红**：一行（一个坏实现）在 oracle X 上算红，当且仅当 X 在这一行两条流的批 1 里至少判红一处（O1-对拍按调用算，其余按状态算）。批 2 另判一遍，只进第八节。
4. **触达**：一行算触达，当且仅当下面三样里至少一样与基线（不改实现那一趟）不同：两条流任一条的 `ops.tsv`、`ops-payload.bin`、`calls.tsv`、`base-sectors.bin` 的 sha256；任一次调用的结局（成功 / 哪一种错误）；写流全同时，批 1 任一态上看 journal 那一遍恢复的报告（`effective_root`、`outcome` 的种类与读回内容的 sha256、施加的记录条数）。三样都相同的行叫**未触达**：它在这一批上与正常实现逐字节同行为，哪条 oracle 都不该红，也不进 Q1 的分母。只在不看 journal 那一遍恢复上不同的行另记「只在不看 journal 那一遍不同」，不进分母（那一遍是层 0 的诊断读法，不是实现交给用户的恢复）。
5. **不进分母的另几类**（逐行列出、照数报，不删）：替换之后编不过（`.claude/rules/mutation-sampling.md` 第八类，按 5.2 的补位规则换行）；锚点在源码里不唯一（第七类，同样补位）；表里按名字找不到这一行（同样补位）；**实现自崩**——驱动一次调用或某一态的看 journal 恢复里 panic（panic 不是悄无声息的错，不考 oracle 的独立性）；单行超过 1800 秒（`.claude/gate.d/checker-tier-crates-mutation-replay.sh` 第 46 行同一个默认限时）记超时。一次调用返回错误而模型说该成功，不算自崩，是 O1-对拍的红。
6. **O1 够不到的地方**：流二最后一次调用 `unmount`（卸载时抬 F 那两次发布）模型没有对应的答（`model.rs` 里没有 `answer_unmount` 一族），这次调用不对拍，任一写落在这次调用里的状态 O1-崩溃判官记「不适用」；一次对拍判出不同之后模型不再往前走，此后的调用与写落在此后的状态同样记「不适用」。不适用算不红，个数照报。除这两种来源之外出现不适用，走停机 S5。
7. **「规约欠一句」怎么算**：严格跑法判「规约欠一句」的态算 **O3 弃权，不红**——它没有判这个态不在允许集合里。Q1、Q3 各按两种 O3 判一遍、各占一行：**甲**取 O3 严格跑法（欠一句 = 不红），**乙**取 O3 暂代跑法（E170 登记 5.9 写的进门禁那一种跑法就是暂代跑法）。乙红而甲不红的行标「只靠暂代读法撑着」。执行器对一态抛出「error」（`executor_main.py` 第 45 行那一种）算不红、单列；执行器报底镜像恢复不出（`executor_core.py` 第 74 行 `BaseImageNotRecoverable`，它自己的说明是「这条流不判」）时，这一行这条流 O3 两种跑法都记弃权、不红，行上标「O3 底镜像恢复不出」。
8. **O3 用哪一版**：E170 照 C651 判决（用户定逐档：V1 逐档、V2 世代号最大同号取偏移小）改过执行器、严格跑法重跑时冻结的那一版——E170 那一次产物里记的冻结逐文件 sha256（文件集合照 E170 登记 5.6：执行器目录里的源文件，不含 `comparator*` 与 `mutation_driver*`）。执行员在装置写之前把那几行 sha256 连同产物路径抄进第十二节修订（只抄冻结的 sha256 行，不读那份产物的结论行），开跑前与最后一次跑 O3 之后各 `sha256sum -c` 一次。今天（C651 改之前）的逐文件 sha256 记在第三节，**不用它**：一行一个版本，不混。那一版还没有冻结行时 O3 那几列不跑（停机 S7）。
9. **关一条（Q3）**：三条 oracle 的判定互不吃对方的输出（各自只读镜像、写流或实现的恢复报告），所以「关掉 X 之后剩下的红不红」由每行三列的红 / 不红直接算：关掉 X = 把 X 那一列当不红。这条读法靠一次实测撑着：头三行被抓的坏实现，在装置上真的逐条关掉一条 oracle 重跑，其余两列要与不关时逐字节相同（第六节 Q3-核，失败条款 F5）。
10. **Q4 的门槛**：`.claude/rules/verification.md`「checker 档自己分快档与全量」与 D13 已定项 15 都没给快档一个挂钟数（`grep` 结果在第十三节）。**Q4-甲**——照 E170 的取法，取 D13 已定项 15 唯一的时长数「debug 下单条跑到 60 秒及以上的是重」，整表一次 ≤ 60 秒算能进（问题单 Q4 写「门槛照 checker 档快档的口径」；主 agent 2026-10-02 在装置写之前删掉了取每行 1800 秒的那一种读法）。
11. **坏镜像那一族只到两条**：O1 不碰盘（D13 已定项 6 表 O1 行「不碰盘、不解析磁盘格式」），Q2 只判 O2、O3；「正常镜像三条都放行」在这一族读作 O2、O3 都放行。实现的恢复在坏镜像上读回什么交给 O1-崩溃判官也算得出，附带，够判后不跑。

```

**出处 `git show 31128273b^:research/prompts/e171-preregistration.md` 第 221-232 行（整段抄，未转述）**

```markdown
### 5.2 坏实现表：复用 `crates/mutations.tsv`，挑法写死

**二选一取复用**，依据：① 表里每一行都是「把实现改坏一处」、形态就是问题单 Q1 写的「形态照 `crates/mutations.tsv`」；② 锚点由门禁 code-source-discipline 的 mutation-tables 格逐条核唯一命中，不会悄悄腐化；③ 另立一张要由我挑坏法，而我读过三条 oracle 的代码，挑出来的坏法会带着我对三条盲区的猜测——机械取样不带。代价：D13 已定项 6 举的「精心构造」那一种（例：nonce 永远为 0）不在表里；那一例今天造不出（岔路单第二节 Y3 代价表「罩不住」列：加密关着），本实验不补。

**挑法**（脚本 `/tmp/claude-1000/collusion-designer/select_bad_implementations.py`，sha256 `4032a0e06b614a66c67df5fe5eae82e15f3399177655ee4230e50aee4b7785b4`，执行员照抄进 `research/e7-index-bench/src/bin/e171_three_oracle_collusion/select_bad_implementations.py`）：取 `crates/mutations.tsv` 里文件列以 `crates/singlefs-core/src/` 起头、或等于 `crates/singlefs-replay-prefix/src/lib.rs` 的行（注释行与不是六段的行不算）；`singlefs-format` 不取——它是 checker 与实现共用的常量（D13 已定项 5），改它同时改 O2，不是「坏实现」；harness、checker、checker-tier 的行改的是验证一侧，不取。按文件名字典序分组，组内保持表里的先后；一组 n 行取 m = ⌈n/8⌉ 行，组内下标 ⌊i·n/m⌋（i = 0..m−1，0 起）。在 sha256 `40c75af9…` 那一版上得 117 行（逐行在 5.2 末尾的表里，**以这张表为准**，执行员开工时表若已变，照名字取行，不重挑）。

- **第一刀**：每个文件 i = 0 那一行，24 行（表里「刀」列标 1）。
- **补位**：一行编不过、锚点不唯一、或按名字找不到，换同一文件组内下标 j + 1（到组尾回到 0）、不在本表里的那一行；同一行至多补三次，补不上照实报。补位换上的行在产物里带「补位自 <原名>」。
- **扩表**（失败条款 F4、F6 触发时）：步长 4 的挑法多出的 164 行（与步长 8 重叠 56 行不重跑），按同一张脚本的输出顺序跑。扩表与补位都不重挑已跑过的行。
- **行文变了**：开工时某一行的原文或替换文与 sha256 `40c75af9…` 那一版不同（别的会话修了锚点），照今天的行跑，产物里记两份行文的 sha256。
- **触发顺序**：先第一刀 24 行，再其余 93 行。

```

**出处 `research/prompts/e171-r1-prereg.md:37-81`（整段抄，未转述）**

```markdown
### 读法写死

1. **三条 oracle 与 O1 的三个子列**。一行在 O1 上算红 = 下面三个子列任一个红；三个子列各自照报。
   - **O1-对拍**：同原登记读法 1（`IdealModel::answer_*` 加 `judge_and_advance`，观测由 `crates/singlefs-harness/src/model_comparison.rs` 的胶水造），改两处：① 卸载调第 16 批 16-2 加的模型卸载答，参数与观测怎么造以 16-2 交回为准；② 成功调用的交回值整份记下（读法 4），交给模型的仍只是胶水从交回值造出的那份观测——装置不另写比法（另写一份就成了第四条 oracle）。
   - **O1-模型崩溃判官**：同原登记读法 1 与停机 S8（`newest_persisted_root` → `model_root_key`、`observed_read_back_after_a_crash`、`crash_recovery_disagreement` 三处）。
   - **O1-RefFS**：`crates/singlefs-reference-filesystem` 的 `ReferenceFileSystem`，每个取样态上判实现看 journal 那一遍恢复读回的内容在不在允许集合里。与模型崩溃判官**并列**、不替换（问题单 Q5 的翻面观测要两者各自的判定）。事件映射、根到版本号的 m 表、崩溃时刻归属照 `crates/singlefs-harness/src/reference_filesystem_translation.rs` 第 1–18 行文件头那一段与它的 `events_of`（第 304 行起）、`crash_moment_of_a_segment_closed_at`（第 700 行）；判绿 / 判红 / 弃权与弃权的种类照它的 `ReferenceFileSystemVote`、`ReferenceFileSystemAbstention`。弃权不算红，也不算绿，单列。
   - **O2**：`check_pool_image`，第 16 批 16-1 合入后的那一份，其余同原登记。
   - **O3**：哪一版见读法 9；严格 / 暂代两种跑法、欠一句算弃权、error 与底镜像恢复不出照原登记读法 7。
   - 记录核对器、层 0 手写版本表的 oracle、崩溃注入第二截的 `judge_writable_mount_refusal`：**都不是三条之一**。第二截只进 Q4，不进 Q1 的三列：第 242、269 行的补 oracle 改法 ①（C673）落在第二截，Q1 里这两行的三列不因 ① 而变，① 改没改好由 Q4 判。
2. **历史与崩溃状态**。流一（`new_pool_file_creation`）、流二（`fixed_script_through_e_then_unmount`）的驱动与批 1、批 2 取样规则同原登记读法 2。新加**流三**（`first_file_unlinked_then_inodes_across_one_leaf`，C673 ⑦「历史加跨一片叶」）：
   - 驱动：mkfs、取号、暖机、发布第一个文件（与流一三步相同）；删第一个文件——`publish_unlink_intent` 一次，再 `publish_unlink_step` 一次一调，直到删除队列空；之后逐次 `publish_new_inodes`，每次建 1 个 inode，直到 inode 树的叶容器第一次变成两片的那一次发布（记这一次是第 K 次），再多建一次，共 K + 1 次；不卸载。每一次调用都经装置的调用记录与模型对拍（`answer_unlink_intent`、`answer_deletion_step`、`answer_create_inode`）。
   - 为什么一次建一个：理想模型只答一次建一个（`crates/singlefs-harness/src/model.rs` 第 1964 行注释「count 1」）；一次建多个，模型那一步交不出答，O1 在流三上从那一步起全不适用。为什么先删文件：模型的建 inode 只接在第一个文件已删完的一版上（同文件第 1969 行 `answer_create_inode` 的前提）。
   - K 照 D8（核心索引结构） 已定项 6「一容器 233 条」（`crates/singlefs-format/src/lib.rs` 第 110 行 `INODE_LEAF_RECORDS`）推：删完之后 inode 树没有记录时 K = 234（推的）。装置不靠这个推算停：每次发布之后读交回值里 inode 树的叶容器数，第一次到 2 就记 K；基线那一趟把 K 与段长表、写数、根槽写数一起写成本地常量回比（S1）。
   - 取样：批 1、批 2 规则作用在每条流自己的写流上，与原登记同。
   - **哪个问题用哪几条流**：Q1、Q4 用三条流；Q5 用流二上的造法（读法 7）；Q2、Q3 的「未触达」与分母只在流一、流二上算（与 s4 同口径，第二段整表只跑这两条），流三上的读数另报、不进 Q2、Q3 的判定。
3. **一行在一条 oracle 上的红**：同原登记读法 3——这一行在 X 上至少判红一处（O1-对拍按调用算，其余按态算），批 1 定判，批 2 只进第八节。三条流合起来算；每条流各自另报。
4. **触达与未触达**：原登记读法 4 一个字不改（两条流的写流四个文件的 sha256、调用结局、看 journal 那一遍的恢复报告；C672 改法原文「不改登记的触达定义」），只在流一、流二上算。新增两样只记、不进触达：
   - **交回值**：每次成功调用的交回值 `{:?}` 文本的 sha256，连同顶层字段逐个的 sha256（字段名照 `{:?}` 文本切出）；
   - **内存对象**：每次调用之后装置手里由实现交出的对象（分配器、现行那一版、暖机输出、可写挂载的输出与会话里装置持有的那几样）各自 `{:?}` 文本的 sha256。
   - 两样都与基线同一条流、同一次调用的比。基线连跑两次摘要不同的对象（迭代次序不定一类）从比较里拿掉、单列名字，不作废整趟。
   - **Q2 的行集合**：整表里按本读法未触达、而至少一次调用的交回值或内存对象摘要与基线不同的行。C672 点名的七行（第 49、110、155、161、162、174、252 行）不论在不在这个集合里，逐行照报。
5. **panic 与报错同样截断照判；不进分母的几类**（C673 ⑨）：
   - **调用 panic**：与交回错误同样截断——这次调用的结局记 `panic`，之前的调用照判，这条流停在这次调用，写流（含 panic 那次调用已发出的写）照导出、照取样、照判；`calls.tsv` 那一行写 `err`（E170 登记 5.3 的格式只有 `ok` / `err`）。模型停在这次调用；这次调用的 O1-对拍记「实现自崩、不比」，不红不绿。RefFS 把这一步当在途（`events_of` 对 `NeverReturned` 的映射：内容改动照交、发布调用记 `FailedAfterWriting`）。
   - **态上的恢复 panic**：不截断。那一态记「恢复 panic」，O1 的两个崩溃子列在那一态记不适用，O2、O3 照判那一态，其余态照判（各态是互不依赖的镜像，截断会让判了几态取决于判的次序）。
   - **不进分母**：原登记读法 5 的编不过、锚点不唯一、找不到、超时照旧（补位规则照原登记 5.2 与实验页「逐行驱动」一条写的第八段起的补位）；「实现自崩」一类收窄成：流一、流二上第一处调用 panic 之前，O1-对拍判过（`agrees` 或 `disagrees`）的调用两流合计为 0 的行（问题单 Q3 翻面观测原文「自崩前可判的调用数为 0」）。不在这一类的带 panic 的行，照读法 4 判进不进分母。
6. **O1 够不到的地方**：卸载由模型答（读法 1），不再是不适用的来源。O1-对拍与 O1-模型崩溃判官的不适用只许来自三处：对拍判出不同之后、调用交回错误之后（含那一次）、调用 panic 之后（含那一次）；别的来源走停机 S5。RefFS 的弃权不受这一条管，照它自己的弃权种类报。16-2 交回说某种卸载形态模型不答的，不在这里补来源，走停机 S12。
7. **Q5「已确认的丢了」那一形**（C673 ⑧「带内容的已确认丢了」）：
   - 造法：取本次冻结树上基线那一趟流二的导出。流二第一次带文件内容的发布（第 2 次调用 `publish`，前两次是取号、暖机）返回成功，它是**已确认**的；在这次调用里，把第一条 journal 记录写之后、返回点之前的每一道屏障删掉，这次调用写根槽的那次 FUA 写改成普通写；写流截到下一次调用（第 3 次调用，覆盖写）发出的第一道屏障之前，第 3 次调用在那道屏障之前发出的写留着，第 3 次调用没有返回。写的字节一个不改，只改屏障与写种类。
   - 崩溃状态：末段（最后一道留下的屏障之后到截断处的全部写）的整写子集全枚举，可撕裂的写另加撕裂态，一个不落；前面各段与基线相同，不重判。
   - **「已确认的丢了」的态**：看 journal 那一遍恢复落到的版本按 (checkpoint_txg, 实例代号) 早于第 2 次调用写的那一版的态。装置算一遍，比对器从导出的根槽写与恢复报告再算一遍（第七节 A10）。
   - 每一态上 O1 三个子列都算：O1-对拍不涉及（没有新调用返回）；模型崩溃判官的版本目录与「盘上已持久的最新根」照读法 1；RefFS 的事件：第 0、1 次调用不进序列（与 `events_of` 对取号暖机同），第 2 次调用是「建第一个文件 + 发布调用（成）」，第 3 次调用在途（「覆盖 + 发布调用（写了之后报错）」），崩溃时刻照 `crash_moment_of_a_segment_closed_at`（末段没关上，算在全部步都返回之后）。O2、O3 在那一形上附带照判。
   - 方向相反的取样点：截断点挪到第 2 次调用的返回点（第 3 次调用的写一条都不带），别的同（第八节）。
   - 附带（够判后不跑）：不带内容的那一形——流一暖机第二次空发布同样删屏障、根槽 FUA 改普通写，截到下一次调用的第一道屏障之前。
8. **Q4 的口径**（C670、D23 已定项 14「可写挂载读到的更新状态读不出」那一段、D13 已定项 1 挂载透明射程「可写挂载被拒的状态不判透明，判拒绝本身合不合法」）：
   - 态的集合：基线、第 242 行、第 269 行三趟，三条流批 1、批 2 的每一个取样态。这些都是层 0 截断出的态，**没有注入任何读故障**。
   - 每态做什么：照崩溃注入第二截那一处（`crates/singlefs-checker-tier/src/crash_injection.rs` 今天第 2044 行调 `judge_writable_mount_refusal`）：崩溃镜像落成内存池，在副本上起一次可写挂载（入口与空间准入开关同第二截）；挂上了记「挂上」；拒了记拒因，拒因在 D23 已定项 14 那张单子上的，用第二截同一个调用判合法 / 不合法——签名以 16-3 合入后的为准，「这一态没注入读故障」照实交；另几样入参（拒之后盘上逐字节同不同、只读读出在不在允许集合里）照第二截那一处现算（停机 S10）。
   - **见证判据为真分两支记**（照 D23 已定项 14 那一段的字面，在崩溃镜像上由装置自己算）：**（i）判不出那一支**——某块盘自证过的系统配置槽少于两槽（`crates/singlefs-core/src/recovery.rs` 第 3232 行 `verified_system_configuration_slots` 现数），即撕裂或没写全的系统配置；**（ii）计数那一支**——每块盘两槽都自证过、c_见证 ≠ 0、按那一段「所选那一版那次发布带末条标志的记录读得出……两条都读不出按真」三句判为真。
   - Q4 的「这种态」只取（ii）：C670 改法原文「撕裂系统配置那一支（C664、C331）先分开，否则基线上同样有的撕裂拒一起红」。（i）照数另报，不进 Q4 的候选。
   - 判：三趟合计（ii）态数为 0 → 甲；≥ 1 且每一态第二截判不合法 → 乙；≥ 1 且至少一态判合法 → 丙（逐态列出）。基线（今天的实现）与第 242、269 行分开写结论。
9. **O3 用哪一版（接口）**：O3 列用 Q6 那份登记（第 16 批 16-6）写死给 E171 用的那一版——执行器目录、冻结的逐文件 sha256 行、种子。执行员在装置写之后、任何 O3 列产物之前，把那几行原样抄进第十二节修订并写明出处（那份登记的路径与 sha256）；核对做法同原登记修订 2（`frozen_o3.py` 的 `freeze_differences`，每趟开头与收尾各一次）。那份登记没写、或写了而没有冻结行：O3 列不跑（停机 S7），O1、O2 照跑，第一段照交回；第一段各行的导出目录留着不删，等 O3 列补跑。
10. **树：何时取、怎么取**。第 16 批 16-1 到 16-4 合入、主 agent 的合入验证（16-7 的 `merge-and-verify.sh`）绿之后，执行员写完新装置与研究一侧、编过，再冻一棵树：照原登记修订 5 的逐行驱动 `--snapshot-from`，把仓的 `Cargo.toml`、`Cargo.lock`、`.cargo`、`crates/`（不带 target）拷进 `scripts/experiment-apparatus/target/e171-r1-frozen-tree/`。第一段第一份产物开头写 `name=frozen_tree` 几行：快照摘要（驱动的 `snapshot_digest`）、`git rev-parse HEAD`、`git status --porcelain -- crates Cargo.toml Cargo.lock` 原样、16-1 到 16-4 各自改过的文件在冻结树里的 sha256、新装置文件的 sha256。之后每一趟（基线、对照、逐行、Q4、Q5）的二进制都从这棵冻结树编，每趟开头与收尾核快照摘要（作废 Z5）。
11. **s4 读数只当对照**：s4 冻结树（快照摘要 `73c9487c41a862ad495c16155773e4f8b58e920545762edf2d44c6fd80627dc5`，原登记修订 5）上的读数不进任何判据，比对器不读 s4 的产物；判定写完之后另出一张逐行对照表（附带）。
12. **第 16 批改法与 Q1 七行的对应**（C673 原文，是改法落点，不是预期）：第 15 行 ← ⑥（O2 新池新建文件落点，16-1）；第 23 行 ← ⑦（流三）；第 42 行 ← ④（O1 卸载的答，16-2）；第 242、269 行 ← ②（O3 谓词 W，Q6）与 ①（第二截，16-3，只进 Q4）；第 270 行 ← ③（O3 见证值不低于 tail，Q6）；第 271 行 ← ⑤（O2 树表条目头 ID，16-1）。
13. 原问题单（`research/prompts/three-oracle-collusion-questions.md`）的 Q1–Q4 不是这一次的判据；比对器照旧算出的那几格（原 Q1-甲、Q1-分母等）只作描述。

```

**出处 `research/prompts/e171-r2-prereg.md:28-49`（整段抄，未转述）**

```markdown
### 读法写死

1. **「E171 的结论」逐格指哪几份**（行级读数都按整表编号；一行的判定只取决于这一行自己的坏实现与树，行与行之间只经补位相关，读法 6）：
   - **原登记整表**：`e171-three-oracle-collusion-2026-10-03-s4-f4-comparator.out`（F4 扩表，第 1–281 行）与 `e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator.out`（加编不过 6 行的补位），比对器 `scripts/experiment-apparatus/e171_three_oracle_collusion/collusion_report.py` 的 `q1_alpha`、`q1_beta`、`q1_gamma`（每种读法的 `verdict=` 与 `row_numbers=`）、`q1_o1_and_o2_both_quiet`、`q1_denominator`、`q1_column_red_rows`、`q3_alpha`、`q3_beta`（`only_<列>=` 各行）、`untouched_but_red`、`q1_denominator_by_table`。117 行那张的 s4 整表没有第 236 行，不算。
   - **r1 第二段（整表，修订 6 树）**：`e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-comparator-r6.out`，比对器 `segment_two_comparator.py` 的 `name=verdict` 那一行（`q2=`、`q2_set_rows=`、`q2_o1_red_rows=`、`d_<读法>_batch_<批>=` 四个、`rows_not_located=`）、`q2_set_row`、`q3_denominator_class`（`row_numbers=`）、`q3_row_125`、`not_located_row`、`substitution`。
   - **r1 第一段（Q1–Q5 逐行，修订 6 树）**：`e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-comparator-resume2-r6-r3-2026-10-07.out`。它只跑空变异与 15 个点名行（`rows_orchestrator.py` 第 155 行 `STEP_ROWS`、`segment_one_comparator.py` 第 31–34 行），撤项效应只用第七节 A23 的结构核对判，不读它的行；现状整表段才与它逐格比。
2. **「结论变」与「钉值跟着变」怎么分**（问题单 Q1 甲的字面「结论不变，只是名单少一项、钉值跟着变」）：
   - **判定词**：上面各份里取值是词的格——`q1_alpha` / `q1_beta` / `q1_gamma` 的 `verdict=`（`yes_a_row_is_quiet_on_every_oracle`、`no_such_row`、`not_enough_rows`、`not_reported_f7_strict_o3_reaches_almost_no_state`），r1 第二段的 `q2=`（`jia`、`yi`、`bing`、`not_enough_f4`）与 `row_125_counted_registration_batch_1=`，r1 第一段 Q1–Q5 的甲乙丙。
   - **钉值**：其余取值是数或行号列表的格（`counted_D`、`d_*`、`all_quiet_rows`、`row_numbers`、`only_<列>`、`q2_set_rows`、名单行数、sha256）。
   - Q1 判**乙**：撤项之后任一份的任一判定词与撤项之前不同。判**甲**：判定词全同；变了的钉值逐格列出，一格不漏。判定词全同而某个钉值是实验页、索引或 D13（验证路线） 已定项 6 正文里写着的数，照列成「要回改的现状句」，不因此判乙。
3. **撤掉什么、改引什么**（问题单 Q2 的处置；「取不到」= 读法 5 的取法命中 0 次）。按变异名取不到的项逐项分三类，每类的判法都能用命令判：
   - **甲·旧格式旧位**：名字在 `git show HEAD:crates/mutations.tsv` 里有、在开工那一刻的主工作区表里没有，且 `records/2026-10-31-删兼容旧版本代码.md` 第 4 批记它随兼容代码删了。**撤掉，不换**：两份名单删掉那一行，整表编号不重编（读法 4）。这一类今天只有第 236 行（第四节第 2 条）。
   - **乙·同一个坏实现、名字变了**：今天的表里有一行的（文件，原文，替换文）三样与它在 r1 用的那张表里那一行逐字相同，且这样的行恰一行，或其中恰一行的名字等于把它的旧名照 `.claude/kb/term-renames.md` 登记的改名换过之后的名字。**改引**：只改名单第 5 列成今天那一行的名字，第 1–4、6 列不动（第 6 列的行文摘要照旧，逐行驱动照 `bad_implementation_table.locate_by_name` 记 `line_matches_the_registration=false`、照今天的行跑）。
   - **丙·坏实现本身变了或没了，又不是旧格式**：三样在今天的表里找不到逐字相同的一行。**撤掉，不换**：同一个坏实现在今天的表里没有对象；改引同一处锚点上替换文不同的那一行，是换了一个坏实现，等于重挑（原登记 5.2「照名字取行、不重挑」，上一次登记修订 8 第 8.3 节「重新取锚是换了一行坏实现，是新实验」）；留着取不到的名字就是这一次要修的毛病。
   - 三类都对不上的（例：三样逐字相同的行不止一行、按改名也挑不出一行）记「未分类」，不改那一行，交主 agent（F13）。
   - 「r1 用的那张表」= r1 第二段那一趟冻结树里的 `crates/mutations.tsv`；那棵树不在工作区时，取 r1 产物里那一行的 `not_located_row` / `substitution` 记载与 `git log -S'<名字>' -- crates/mutations.tsv` 定它当时取不取得到，取法写进第十二节修订。
4. **整表编号不重编**：撤掉的行留空号，两张表第一列的并集就是行集；行集不再写死成 1–281。好处是逐行与 r1、原登记按编号一一对上。
5. **Q2 的取法**：与逐行驱动同一个读法——`bad_implementation_table.read_mutation_rows`（跳过 `#` 起头的行、只收恰 6 列的行）读开工那一刻的主工作区 `crates/mutations.tsv`（HEAD 加暂存区加未暂存），按名单第 5 列逐字比第 1 列，数命中次数；命中 0 次是「取不到」，命中 2 次以上是「名字不唯一」（另列，不在 Q2 的「取不到」里）。两份名单各核一遍。表的 sha256 写进产物；核的那一刻与改名单那一刻表的 sha256 不同走 S21。
6. **补位与撤项的关系**：逐行驱动的补位只在同一文件组里换下一行，跳过两张名单里所有的名字（`substitute_for`，`names_in_the_table_of_rows`）。撤掉一行之后，同文件组里别的行在旧树上补位时可能换到被撤那一行的变异。撤项核算把被撤的行从读数里减掉，只在「与被撤行同一文件、在那一趟里有过补位或定位失败」的行都没有时成立；有的，那几行走 F12。
7. **钉值怎么重核**（问题单 Q1 够判条件「钉值重核」）：分三样，各在第六节占一行——名单的项数与 sha256（第七节 A21）；研究一侧代码里写死的行集与项数（`segment_two_comparator.py` 第 35 行 `TABLE_ROWS = 281`、`rows_orchestrator_segment_two.py` 第 108–109 行与第 218–223 行、`count_reusable_units.py` 第 11 行与第 29 行、`rows_orchestrator_segment_two_revision_11.py` 第 345 行，执行员开工时 `grep -n '\b281\b\|\b282\b\|\b164\b\|\b117\b\|\b330\b'` 现列）；装置本地常量（`stream-constants --streams 1,2,3` 在今天的主工作区上，与上一次登记修订 12 的产物比）。各 oracle 的合谋计数不是代码里的常量，是比对器读产物现算的钉值，由读法 2 那一份逐格比管，不另钉。
8. **两条比法分开**：撤项效应只在 r1 与原登记那几棵旧树的读数上算（今天的树上被撤的行没有对象，量不到「撤之前」，第八节）；现状整表段在今天的树上重跑整张名单，与 r1 逐格比，差出来的格逐行归因（被撤的行、改引的行、树变了的行），不并进 Q1 的判定词。

```

**出处 `research/prompts/e171-r2-prereg.md:519-587`（整段抄，未转述）**

````markdown
### 修订 9（2026-11-02，承接的执行员；收尾：现状整表段不跑；这一条修订写完时现状整表段一份产物都没跑，这一条没有单测、变异与自证要跑）

依据：派发提示（批 17 续，收尾）「关的是哪几条」第 1 条原句——「用户 2026-11-02 在弹窗里选「第 2 次重跑收尾，整表另立新实验」（问句：「E171 第 2 次重跑的现状整表段，起跑前的「拍树前五项核对」照现在的代码与登记会退 1（推的…）。这一段整套沿用 r1 修订 8 的冻结层，卡口六处：B1、B2 判法还认 r1 的「合后清单」与「修订 9」；B3 要宣布那一刻的草稿二进制；B4 要把仓里 ab35105f 那版装置逐块对到今天的装置（执行员量的：差 191 段、833 行）；B5 要修订 6 第二段的行输出根，已不在；B6 要修订 6 树，造不回来（E200 量的）。Q1 撤项那一格已判甲…，Q2 已判「有」；整表段只补 Q1 的逐格对比与归因。整表段怎么办？」；选项原文「E171 第 2 次重跑照已有读数收尾（Q1 撤项判甲、Q2 有，整表段记「冻结层依赖的 r1 输入造不回来，不跑」）；在今天的树上整表重跑另写一份不继承 r1 冻结层的跑前登记，先设计、后跑。」）。写登记修订 9（收尾，连同 B1–B6 原文与上一段报告出处）；问题单两行状态改用户已定（Q1 撤项那一格判甲、整表逐格对比与归因留给新实验；Q2 有）」。读数依据：上一段执行员的报告 `~/.local/share/singlefs/work/compat-removal-e171-r2-run-j/result.md`「卡口」一节第 135–153 行（逐字抄在第 9.3 条）；那份报告里「拍树前五项核对退 1」是读代码推的（那时没有宣布行、没有树，没有跑过 `before-snapshot`），B4 的 191 段、833 行与 B5 的「不在」是执行员量的，B6 是 E200（修订6近似树重跑） 页量的。第一至十一节、修订 1–8 原文一字不改；第六、七、九节每一条的去留与报告方式只动第 9.1 条点名的「现状·逐格」一行，依据是上面这句里「整表段记「冻结层依赖的 r1 输入造不回来，不跑」」与「整表逐格对比与归因留给新实验」；判据一个没动，没有收严也没有补臂：这一条只记不跑、不新增要跑的量。

9.1 **现状整表段不跑，逐条记（跑法与报告方式；判据不动）**：

| 登记里的条款 | 这一次的状态 |
|---|---|
| 第五节 5.6 分段表「现状整表段（第二段）」 | 不跑：冻结层依赖的 r1 输入造不回来（第 9.3 条） |
| 第六节「现状·可取行」 | 已跑，在撤项核算段：名单核对产物 `…-2026-11-01-list-check-2026-11-01.out` 第 37 行 `E7RESULT name=current_rows rows=278 rows_not_hit_once_by_name_or_anchor=0` |
| 第六节「现状·逐格」 | 主 agent 定不跑（第十一节「够判停机」末句两种写法里的这一种：问题单 Q1 够判条件字面的后半「与 r1 逐格对比」没满足，不是够判之后没必要）；用户定的，逐格的差与归因（被撤、改引、树变了）这一次没有读数，留给整表另立的新实验 |
| 第六节「附带（够判后不跑）」 | 照原条款，没跑 |
| 第八节「树」一行（旧树读数与今天的树读数逐格并列） | 只有旧树那一边的读数；今天的树那一边没有 |
| 第十一节 Z13、S22，与沿用的上一次登记停机 S1–S3、S5–S13、S16、S17 | 现状整表段不起，不触发也不判；S16 的宣布行不写（`records/2026-10-31-删兼容旧版本代码.md` 里没有行首「- E171 第 2 次重跑拍树宣布」的行） |
| 第十一节比对对象表 #5、#6、#7（现状整表段逐格比、逐行归因） | 不比。机读登记 `.claude/gate.d/replay-comparison-units.tsv` 第 48–50 行的「不登记的比对对象」三行与登记表逐行对，登记表原文不改，所以那三行留着 |
| 修订 5 第 5.9 条、修订 6 第 6.6 条、修订 7 第 7.8 条、修订 8 第 8.8 条「现状整表段起跑之前还差的」 | 不再执行；原文照留 |
| 第十四节现状整表段的挂钟与核数 | 不用 |

Q1 撤项那一格与 Q2 不动：Q1 撤项那一格仍是 Z12 重跑之后三份撤项核算的读数（实验页第 9 条，三份判决行都是 `q1_judgement=jia`），写结论仍照第六节「写结论时」那一句，不写成「E171 在今天的实现上结论不变」；Q2 仍是名单核对产物第 20 行 `E7RESULT name=q2 directories_agree=true not_found_rows=33,70,107,141,168,236 not_found_besides_row_236=5 verdict=yes`。

9.2 **问题单两行的最终状态**：Q1 撤项那一格判甲（用户定）；Q1 够判条件字面的后半（整张名单在今天的树上跑完，与 r1 逐格对比并归因）这一次不做，留给整表另立的新实验，新实验另写一份不继承 r1 冻结层的跑前登记，先设计、后跑；Q2 有（用户定）。两行都不再开着。

9.3 **六处卡口原文**（上一段报告「卡口」一节第 137–153 行逐字；原文里的「第 2 条 b」「第 2 条 d、e」指那份报告第 2 条（占位）的子项，「清单第 N 步」指那份报告里一份 16 步的起跑清单，那份清单只在报告里，没进仓，也不进登记；原文里各处的「量的」标注照留）：

```text
- **B1 宣布行与第 2 条 b**：`frozen_tree_lines_revision_8.py` 第 405–418 行的第 2 条 b 要三样：`H1.after_sha256_files_in_merge_order` 不空、每份 after 清单都在宣布行里点名、仓里文件等于清单里最后一份的 sha256。
  - r2 登记第十一节末段（第 287 行）写的是「三条线各自最后合入的批号那一项不适用，改写 HEAD 与 git status 全文」，判法没跟着改。
  - 照 run-g 第 5 组第 0 步那种宣布行，b 判 false，`before-snapshot` 退 1。
  - 出路二选一：(a) 写一条登记修订，判法照修订改（第 2 条 b 在 r2 下记不适用，或改核 HEAD 与 status），这连着重钉一串、H17 重填；(b) 不改判法：宣布那一刻 `sha256sum` 几份要钉的文件进一份清单，放在认领的目录里，宣布行里用 `~/.local/share/singlefs/work/…` 那种写法点名它（判法认 `~` 的写法，第 409 行）。(b) 也改了宣布行写什么，要写进登记修订。
- **B2 第 2 条 e 与 H6 要 r1 登记里有「### 修订 9」**：`frozen_tree_lines_revision_8.py` 第 89–90 行 `PREREGISTRATION = "research/prompts/e171-r1-prereg.md"`、`REVISION_9_HEADING = "### 修订 9"`。
  - r1 登记里这一行 0 处（`grep -c '^### 修订 9'` 得 0，量的）：修订 8 那一趟从没拍过，修订 9 没写过，标题从修订 8 直接到修订 10。
  - r2 下与它对应的是 r2 修订 2（S20 重钉）。不改判法，第 2 条 e 与 H6 都过不了。
  - 出路：写修订，判法改读 r2 修订 2；或在 r1 登记补一条修订 9，照抄 r2 修订 2 的常量。要不要动 r1 登记由主 agent 定。
- **B3 草稿副本**：第 2 条 d 要草稿副本里 `target/release/` 下两个二进制都在、可执行、且比副本里装置源码新。e 要那份副本上的 `stream-constants` 产物甲类全过（r1 修订 8 第 8.2 节「怎么取」：宣布之后、拍树之前，在宣布那一刻的副本上跑；r2 修订 2 第 2.4 条照这句）。这一步 run-g、run-h、run-i 的清单里都没有；我在下面清单里补成第 4 步。
- **B4 H5 块表**：S16 ③ 要 `ab35105f…` 依次打上块表各块之后等于树里装置，每块来源只认三种（r1 登记第 1480 行）。
  - 今天的装置 `cbecfceb…` 与 `ab35105f…` 差 191 段、833 行（`diff | grep -c '^[0-9]'`、`grep -c '^[<>]'`，量的）。
  - 改动来自 `f8a9ddaab` 之后 4 个提交（`ff7e45d01`、`0b926bd3f`、`df5866f67`、`d6dd6da42`），外加这一批暂存的一处（72 行加、84 行删）。
  - 这些块大多只能归「主 agent 认定点名的别的装置改动」，要主 agent 逐块认、写批号与改了什么。没有这张表，第 0 步与 start 阶段的 H5 都不过。块的 fixture 照第 1481 行用 `diff -u --label` 出、头两行不带时刻。
- **B5 H28（第二段对照）**：`check_h28` 要修订 6 那一趟第二段的行输出根是个目录。
  - `scripts/experiment-apparatus/target/e171-r1-dumps` 与 `<工作根>/e171-r1-segment2-multi-host` 都不在（`ls` 报不在，量的）。
  - 第二段对照照现在的判法出不来，要像 r2 修订 6 第 6.1 条处理第一段那样，另写一条修订。
- **B6 H33、H34**：H33 是「修订 6 树对修订 8 树」的复用计数，r1 登记第 2293 行要修订 6 树。r2 修订 6 第 6.1 条已记修订 6 树造不回来，H33 照字面出不来。H34 由主 agent 照 H33 定；库此刻是空的，r1 登记第 2468 行写「不导也不影响作数」。
```

量法（该执行员的报告；这里照量的、推的分开记）：B1–B3 是读代码与登记推的（没有宣布行，没有冻结树，`before-snapshot` 没有真跑过）；B4 的 191 段、833 行与 B5 的「不在」是执行员现查量的；B6 的「修订 6 树造不回来」出自修订 6 第 6.1 条（主 agent 转述的现查：全部提交与 reflog、stash、悬空对象、第二台整盘与本机整盘，都没有修订 6 用的文件）与 E200（修订6近似树重跑） 页（照 git 拼的两棵近似树都停在停机 S40）。六处里 B5、B6 是 r1 的输入已不在，输入补不回来，只能像修订 6 第 6.1 条处理第一段对照那样改记「造不回来」；B1–B4 改得动，但要连着改冻结层的判法、重钉一串钉值再重填占位，用户的选项把这一整套沿用 r1 冻结层的做法一并收了。

9.4 **收尾之后不再用的东西，逐项去向**（这一段做完的；判法：有活的读者就留作已冻结的装置，没有读者的过渡状态撤掉）：

| 东西 | 去向 | 依据 |
|---|---|---|
| 上一段填进 `revision_8_placeholders.json` 的 H17（15 份文件的 sha256） | 撤回，`files` 改回 `null`；文件与暂存区那一版逐字节相同（sha256 `62953b70…`，`git show :<路径> \| sha256sum` 量的） | 读它的只有 `revision_8_placeholders.py` 的 `check_h17`（第 346 行），起跑前 `--phase start` 才调，这一段不跑，所以没有读者；自证格 `h17-files`（第 515 行）用假配置，不读它；模板的约定是「值先全是 null；拍树那一刻由执行员与主 agent 照表填」（同文件第 3 行）。那 15 份文件任何一份再改，这 15 个值就过期，没有读者的过期钉值是过渡状态。15 个值留在上一段的报告里，不进仓 |
| 其余占位 JSON：`revision_8_placeholders.json` 的其余各节、`revision_10_placeholders.json`、`revision_11_placeholders.json` | 留着，作已冻结装置的模板 | 活的读者：`revision_8_placeholders.py` 第 470 行 `template()`（自证格 `template-shape`、`unfilled-named` 载入）、`rows_orchestrator_revision_8.py` 第 269、300 行、`revision_10_placeholders.py` 第 539 行 `template()`，门禁 `.claude/gate.d/code-tooling.sh` 里有这两份 `--selftest` 的命令行（`grep -n 'revision_8_placeholders.py --selftest\|revision_10_placeholders.py --selftest' .claude/gate.d/code-tooling.sh` 各命中一行）；`revision_11_placeholders.json` 是 `run_segment_one_revision_11_multi_host.sh` 第 17 行与 `run_segment_two_revision_11_multi_host.sh` 第 19 行不给配置时的默认 |
| `revision_8_research_side_of_the_revision_6_run.json` | 留着 | E200（修订6近似树重跑） 的 `build_approximate_tree.py` 第 56 行读它 |
| 那份 16 步起跑清单 | 不进仓、不进登记 | 它只在上一段的报告里；清单是为现状整表段写的执行步骤，整表另立新实验之后由新登记写自己的步骤，抄进仓就是给不跑的东西留一份可执行的样子。修订 5 第 5.9 条、修订 6 第 6.6 条、修订 7 第 7.8 条、修订 8 第 8.8 条的「还差的」清单原文照留，状态见第 9.1 条 |
| 已冻结的装置整体：三份占位检查器、冻结层判法 `frozen_tree_lines_revision_8.py`、逐行编排与单元编排、复用库、对照脚本、跑法脚本、单测模块、研究一侧变异表、自证 | 留着，不删 | 活的读者：门禁 `.claude/gate.d/code-tooling.sh` 里那两份 `--selftest` 的命令行与 `.claude/hooks/selftest-registry.tsv` 里 E171 研究目录的 14 行（`grep -c 'e171_three_oracle_collusion_after_the_oracle_fixes' .claude/hooks/selftest-registry.tsv` 得 14，都登记为 light）的 `--selftest`；`scripts/experiment-apparatus/replay.sh` 里 E171 研究一侧变异的复跑行，它们的产物是实验页第 10–12 条整行引的读数；E200 读其中一份 JSON（上一行）。删它们要连着改门禁表、自证登记、复跑登记、实验页引用与冻结层钉值，是另一件事，不在这一段，写范围也不含那些表。新实验的跑前登记沿用其中哪几件、不沿用哪几件，由它写 |
| 上一段留在工作根草稿目录的东西：`originals/`、`tools/`、`device-ab35105f.rs`（`git show f8a9ddaab:` 取出的装置，供 H5 块表用）、`rev12-stream-constants.out`、`h17-files.txt`、`h25-files.txt` 等 | 不在仓里；交主 agent 删 | H5 块表、H17、H25 都随现状整表段不跑而没有用处 |
| `scripts/experiment-apparatus/replay.sh` 里 E171 第 2 次重跑的复跑登记行（第 1923–1932 行，十行） | 不动，核对得上 | 十行的驱动函数都在 `replay.sh` 里定义，期望产物都在 `research/results/` 里；其中第 1923 行 `driver_e171_r1_revision_12_stream_constants` 指 S20 重钉之后的 `…-2026-11-01-stream-constants-r2.out`，第 1920 行指修订 8 那张表的 `-r3`；命令与原样输出在实验页第 13 条 |
| r1 登记 `research/prompts/e171-r1-prereg.md` | 没动 | B2 的出路之一是在 r1 登记补一条修订 9，这一段没做；整表另立新实验，不再需要 |

9.5 **岔路表**（第十一节「够判停机」要的那张；问题单 `research/prompts/e171-r2-questions.md` 两行）：

| 行 | 已够判 / 还差什么 | 剩下的量能不能让它翻面 | 算它的命令 |
|---|---|---|---|
| Q1 撤项那一格 | 已够判：甲（用户定） | 不能（读法 8：现状整表段的读数不并进撤项的判定词） | `grep -n 'name=verdict' research/results/e171-r2-*-removal-effect-on-the-*-2026-11-01.out` |
| Q1 够判条件字面（整张名单在今天的树上跑完，与 r1 逐格对比并归因） | 关了：用户定这一次不做，留给整表另立的新实验；不是够判 | 这份登记里没有剩下的量；这一格没有读数 | 没有：新实验另写跑前登记 |
| Q2 | 已够判：有（用户定） | 不能 | `grep -n 'name=q2 \|name=disposition' research/results/e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-01-list-check-2026-11-01.out` |

没有开着的行。

````

## 三、实现今天的样子

2026-11-03 现查。`git rev-parse HEAD` = `59ce5a86550de2c2c9bc6b51b34b4f881313682c`；`git status --porcelain -- crates Cargo.toml Cargo.lock .cargo` 334 行（别的会话的已暂存与未暂存改动）。执行员开工时逐项再查，对不上的照实写进第十二节修订，不改判据。

- **入库装置** `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs`（12 960 行，sha256 `cbecfcebadf9f9c6135230071625dd5477da7b92942e2795ec693936318739bc`，暂存区里有改动）：第 14–24 行用法（`baseline`、`row`、`confirmed-loss-form`、`stream-constants`，`--streams`，第 18 行 `--positive-control <O1-对拍|O1-对拍-卸载|O1-崩溃判官|RefFS|O2|交回值|内存对象|panic|见证>`）；第 7636–7646 行认模式；第 7746 行 `baseline` / `row` 的判决行（`s1_failed_checks`、`s3_o2_judgement_mismatches`、`s9_failed_checks`、各列红数、`recovery_panic_states`）；第 7456、7484 行 S5；第 7036–7050 行给定状态引擎（O2 名单经它判）。
- **判段**：`crates/singlefs-checker-tier/src/crash_amplification.rs` 第 18589 行 `GivenStatesEngine::start_from_environment`；第 18606 行起那一份写明「配了 GPU 而这份二进制没编进 gpu 特性……判段这一趟不跑、报本次未跑」「配 GPU 必用 GPU、做不到报错不降级」。装置第 80–82 行另引 `crash_state_enumeration::sample_crash_states_on_the_cpu`。多机配置有 `ENABLE_GPU` 与 `GPU_CARDS_BY_PRIORITY` 两键（只看了键名）。
- **E170 导出装置** `crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs`（4 033 行，sha256 前 16 位 `7bf68ca1584c7bc2`）：只引 `sample_crash_states_on_the_cpu`，不走判段入口。
- **研究一侧**（`scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/`，sha256 前 16 位）：
  - `row_driver.py`（763 行，`ada76e750630ba60`）：第 241 行 `snapshot_digest`；第 260 行 `run_compile`，第 264 行编装置的命令是 `nice -n 19 cargo build --release --offline … -p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes`，不带 `--features gpu`；第 318 行 O3 阳性对照那一步同样套 `nice -n 19`；第 656 行 `--snapshot-from`；第 14 行 `--positive-control` 只认 `O1-对拍|O1-崩溃判官|O2|O3`。
  - `segment_two_comparator.py`（510 行，`d026f465cccbd57a`）：第 37 行 `TABLE_ROWS = len(list_check.whole_table_row_numbers())`；第 313 行 `row_line`；第 233 行 `q2_verdict`；第 379 行 `q3_lines`；第 456 行 `main`。
  - `segment_one_comparator.py`（651 行，`7244bbdcda93e7f3`）：第 31 行 `Q1_ROWS`；第 227 行 `q1_lines`（`three_streams` 与 `two_streams` 两种都出）；第 262 行 `row_column_lines`（只在三流严格那一圈调）。
  - `collusion_matrix.py`（397 行，`47eaa6b87503b079`）：第 57 行 `touch_report`、第 113 行 `column_is_red`、第 126 行 `all_quiet`、第 283 行 `trajectory`。
  - `frozen_o3.py`（336 行，`1a562c8770a6e8a6`）：第 41 行起冻结目录与冻结行的常量、第 229 行 `freeze_differences`、第 306 行 `thread_environment`。
  - `list_check.py`（`8a5adf7c7c53c096`）第 245 行 `whole_table_row_numbers`；`bad_implementation_table.py`（`d7003a5a4652609c`）第 75 行 `read_mutation_rows`、第 115 行 `locate_by_name`、第 126 行 `anchor_hit_count`、第 159 行 `substitute_for`；`run_directory.py`（`e4aa151f8da6dbc6`）、`result_lines.py`（`7f273b61a1b8f2bb`）、`device_timing.py`（`2d092a373c3f0f2a`）、`o3_bad_image_judge.py`（`9bbc45ca572daf70`）。
  - 修订 8 那一层：`frozen_tree_lines_revision_8.py`、`revision_8_placeholders.py` 与 `revision_8/10/11_placeholders.json`、`rows_orchestrator*.py` 六份、`segment_one_units_revision_*.py` 与 `segment_one_unit_revision_*.sh`、`run_segment_*_multi_host.sh` 六份、`reuse_library.py` 与 `reuse_library_root.py`、`cross_host_o3_check.py`、`shard_relay.py`、`count_reusable_units.py`、`contrast_revision_6_against_revision_8.py`、`contrast_segment_two_revision_10.py`。编排层按 sha256 互相载入（E171 第 2 次重跑登记第三节 C936 那一条与修订 2 第 2.8 条），对照脚本读 H15、H28 占位。
- **名单**：两个目录里 `bad-implementations-step-8.tsv`（115 行，sha256 `18cdb0c6707157de2fb969ab0bf3a0848783dbf7fd8b300871d800316834ffb1`）与 `bad-implementations-step-4-only.tsv`（163 行，`57c981f0231a28c879ee977cb84c8daadc95a0d4e70657adeb0711d3b7e696a1`）逐字节相同；278 行都是 6 列，编号不重，缺 33、107、236。`crates/mutations.tsv` sha256 `07fcc8caa410365c9051f07d27d1a6d3b0cd44091bb3ef8111c6808e05c8111e`：278 个名字照 `read_mutation_rows` 的读法（跳过 `#` 行、只收 6 列）各命中 1 次（第十三节 awk）。锚点唯一没核（要读各源文件，执行员在计划那一步现核，不编不跑）。
- **O3 执行器** `scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates_journal_header_487/` 在；冻结行出处 `research/results/e170-r2-o3-spec-executor-2026-10-04-s1b-executor.out` 在工作区与 git 里。
- **统一任务服务**：`task-server/task_server_lib.py` 第 279–302 行「分片」（计划、各片、合并三步；`plan.json` 只认 `label`；片退 0 要建好产物目录；整件成了之后删各片产物目录）；第 973 行 `MAXIMUM_SHARDS = 8`；第 235–237 行同一件的兄弟片同一时刻一台只跑一片；第 240–247 行「GPU」：脚本文件头前 60 行一行 `# task-server-gpu-arguments: <词>…`，第一个非选项参数等于其中一个词就要卡。`.claude/heavy-thread-default` 登记值 31。
- **prune-server 客户端** `scripts/prune-server/prune_server_client.py`：第 226–227 行五个族（含 `apparatus-items`），第 1032 行 `library_store(family, kind, library_directory)`；连不上退 75。
- **oracle 一侧**（G6 要比的几份，sha256 前 16 位）：`crates/singlefs-harness/src/model.rs`（13 522 行，`54859ad3f91d4275`）、`model_comparison.rs`（2 322 行，`92e291af610db54d`）、`crates/singlefs-reference-filesystem/src/lib.rs`（2 143 行，`44087123c6ac1fa1`）、`crates/singlefs-harness/src/reference_filesystem_translation.rs`（1 936 行，`2fac6e0f27058333`）、`crates/singlefs-checker/src/lib.rs`（1 366 行，`aa9cb353991973ec`）。这几份加 `crates/singlefs-checker/` 与 RefFS 包，从参照提交 `31e47115…` 到工作区改了 19 份文件（`git diff --stat` 末行「19 files changed, 41542 insertions(+), 6684 deletions(-)」）。

## 四、跑之前已经存在的数

设计时读到或自己算出的，照实列，不删。

1. **E171 索引行**（`.claude/kb/experiments.md` 第 195 行，全仓搜「新实验号」时 grep 命中整行；那一次没加 `--exclude=experiments.md`，是设计员的疏漏）：r1 第一段与第二段的判定（Q1 甲、Q2 乙、Q3 甲且 D = 39、Q4、Q5 甲），s4 整表的计数（三条全放行甲乙各 12 行、丙 13 行，D = 37，真盲区候选 7 行，「O3 在 37 行计入分母的行上一行都没红」，「两行没触达却被 O1-对拍判红」，「7 行成功调用交回值不同」），r1 第二段「Q2 的集合正是那 7 行，O1-对拍红 2 行（第 155、161 行）」「D = 39（原登记读法 5 算 38，批 2 定触达 35）」，第 2 次重跑的撤项与 278 行。影响：这些是 r1 与 s4 那一边的判定与钉值；这一次 r1 那一边一律从 P1–P3 的行现读，不从这里抄；Q1、Q2 的门槛出自问题单候选的字面与读法 7、9，不从这些数来。「O3 在 s4 整表上几乎不红」让 Q2 的 O3 两列在 D 上大概率两边都漏（推的），这只影响「少了 / 多了」会不会出现，不改判法。
2. **E171 第 2 次重跑登记**：第四节第 1 条（实验页标题里的同一批判定词）；修订 2 第 2.1–2.2 条（2026-11-01 的 `stream-constants` 乙类 19 项变了：流一段长 11 → 12 段、写数 41 → 43，流二 78 → 80 段、481 → 485，流三 959 → 960 段、5980 → 5982，Q5 那一形的下标后推 4）；修订 3 第 3.1 条（三份撤项核算 `q1_judgement=jia`）；修订 9（`name=current_rows rows=278 rows_not_hit_once_by_name_or_anchor=0`、Q2 名单核对那一行、六处卡口）。影响：流的段长与写数变了，批 1 的取样态跟着变，计数类的格两边大多会不同（推的）——这是读法 7 把计数归「钉值格」、只拿「是否大于 0」当判定格的缘由之一；278 行今天按名字各命中 1 次是我在第三节另算的，与修订 9 那一行同向。
3. **E171 第 1 次重跑登记**第四节（s4 三趟挂钟 5 569.6、7 607.9、1 277.9 秒，七行名单，D「40 → 45」，C670 的「15、63 个」「61 个见证拒」，Q5 副本上「RefFS 红 36 态」）、修订 8 第 8.4、8.9、8.11 节（第 7 步约 4.1 小时、带流三的阳性对照各约 2.3–2.7 小时、第 11–13 步本机约 9.5 小时与第二台约 7.0 小时、本机 5 槽与第二台 3 槽，都是推的）。影响：只进第十四节挂钟；Q4、Q5 不在这一次。
4. **D13（验证路线） 已定项 6 现行正文**（第二节整抄）：E171 的结论句（「坏实现表里有三条 oracle 全部放行的行、有只靠一条才抓到的行，O3 在坏实现表上一行没红」）、「Q2 照登记字面判「有」（4 张全是 P8）……真坏镜像里 O2、O3 两条全放行的 0 张」。影响：同第 1 条。
5. **r1 的计时产物** `research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s2-device-baseline-streams-1-2-r6-r3-timing.out`（25 行，整份读过）：基线流一、流二整趟进程挂钟 75.274 秒，流二批 1 的 O2 一段 14.3 秒、批 3 一段 13.5 秒，O2 每态中位约 35 毫秒。P2 第 1 行 `name=row_time_limit seconds=1800 baseline_seconds=75.274 …`，第 284 行 `table lanes=281 threads_per_lane=6,10 rows=281 … sum_of_row_ns=27322997295845 …`：281 行各行耗时之和 27 323 秒，平均每行 97.2 秒（拷树、改坏、编、跑装置，不跑执行器）。影响：只进第十四节推挂钟；读的是耗时，不是判定。
6. **P1–P3 的结构**（只数了行种与字段名）：P1 有 281 条 `name=row`、P2 有 281 条 `row row_number=`；两份的行种表在读法 2；P2 的字段名在读法 2；P3 的 `q1_row` 只在「三流、严格」那一圈对有效的行出（第三节 `segment_one_comparator.py` 第 262 行的调法）。影响：Q2 的范围只能是 D（读法 9、未定项第 1 条）。
7. **E200 登记**读法 1–4（r1 那棵树 = 提交 `31e47115…` 加 94 行未提交改动；r1 装置 `67e7a549…` 不在 git 里，`f8a9ddaab` 那一版删七行可以近似）与第 9 条。影响：只进读法 8 的参照树与 G5 的 r1 装置取法。
8. **自己算的**：今天两张名单 115 + 163 = 278 行、缺 33、107、236、278 个名字各命中 1 次（第十三节）；参照提交到今天 `crates/` 等改了 792 份文件、oracle 那几份 19 份、装置文件有 5 个提交改过（`git log --oneline 31e47115..HEAD -- <装置>` 数的）。影响：「按闭包宽归因几乎一格不落」是从 792 份推的，所以判定用读法 8 的七种窄归因；不进门槛。
9. **判别力（第 3a 步 ②）**：判定格逐字比、集合按集合比，分辨率是一格、一行；分不出的情形只有「计数在 0 以上变了」，那一种按读法 7 归钉值格、另报。Q1 在甲那一臂上的基线是 0 格变、在乙丙那两臂上 ≥ 1 格；Q2 两臂的基线是「某一判定列在 D 上多 0 行 / ≥ 1 行」。两者都是计数、不是比值，在哪一臂的基线上都不退化。D 的大小（≤ 7 行）跑之前不知道：r1 那一边要 P3 里的 `q1_row` 行在、今天那一边要触达。D 为空走 F4。
10. **要触发的事件（第 3a 步 ③）**：跨机对照要两片落在两台：计划写 2 片、服务「各片第二台在前」（第三节），第二台有余量时两台各一片（推的）；落在同一台的那一次照「本次未跑」报。哨兵要库里已有同键的行：头一次交时库里没有（新种类），复用 0、哨兵 0，照报；同一份树重交时才有。第一段别的判定不以事件为前提。

## 五、臂、阳性对照、真实基线

### 5.1 臂（问题单的候选；每一边写成支持它的人认的样子）

| 行 | 候选 | 支持它的人会要求 |
|---|---|---|
| Q1 | **没有一格变** | 判定格与钉值格分开（读法 7），流的段长变了带来的计数差不当「判定变了」；比的是 r1 那几份产物里有的格，r1 没有的格（整表 O3）不算进来；比对器两边是同一份函数 |
| Q1 | **有格变，且每一格都归得到这期间改过的某一处实现或装置** | 七种归因（读法 8）逐格查、全列，名单、补位、坏实现行文、落点函数、装置、oracle 入口、研究一侧都算「某一处」；参照树取得到 r1 那棵树的底（`31e47115…`），不拿今天的树对今天的树 |
| Q1 | **有格变，有的格归不到** | 归因只认窄的七种，闭包一层不算「归得到」（宽归因只描述）；只在一边的键（行没了、行多了）也是格；归因查不了的格（F6）单列、不悄悄算进「归得到」；批 2、另一种读法上的格照报（第八节） |
| Q2 | **三列都不多** | 只在 r1 有逐列读数的行上比（D），不拿今天整表的漏判去比 r1 没有的东西；O3 严格与暂代分开；「多了」按集合读，不按个数 |
| Q2 | **有一列多了** | O1 按三个子列合起来判、子列另报；D 里的行两边都要计入分母、都要有逐列读数，缺一边的行不当「不漏」；整表上今天的漏判集照报，不因 r1 没有就不写 |

### 5.2 阳性对照（每一条臂、每一条子列、每一条管子都跑）

**装置这一条管子**（每一样在基线那一趟与空变异那一趟各跑一遍，都只跑流一、流二；证明逐行驱动、比对器、逐格比这条管子把红一路带到产物）。精确数照 E171 第 1 次重跑登记第七节 A8 第 469 行「空变异那一趟只跑流一、流二，崩溃判官与 RefFS 两项各 2 态、其余同」：

| 对照 | 怎么造（装置的 `--positive-control`） | 必须看到（K3） |
|---|---|---|
| O1-对拍 | `O1-对拍` | 恰 1 次调用红 |
| O1-对拍·卸载 | `O1-对拍-卸载` | 恰 1 次调用红（流二卸载那一次） |
| O1-模型崩溃判官 | `O1-崩溃判官` | 恰 2 态红（每条流一态） |
| O1-RefFS | `RefFS` | 恰 2 态红 |
| O2 | `O2` | 恰 1 态红 |
| O3 | 流一全持久态叠 P1-F 补丁交冻结执行器（`o3_bad_image_judge.py control --row-name P1-F`，补丁出自 E170 导出装置在今天这份基线导出上现导的 `bad-states-second-run`，种子 `85441ae491419411`，E171 第 1 次重跑登记修订 8 第 2b 步） | 判「不在」，`base_digest_matches=true` |
| 交回值 | `交回值` | 比对器报恰 1 次调用交回值不同，那一次 O1-对拍 `agrees` |
| 内存对象 | `内存对象` | 比对器报恰 1 次调用内存对象不同 |
| panic 截断 | `panic` | 流二导出、截在第 3 次调用；之前 O1-对拍判过的调用恰 1 次；取样态数大于 0 |

（`见证` 那一项属可写挂载探针，这一次不跑。）

**逐格比与归因这一条管子**（比对、归因、Q2 三份脚本的单测里手造输入，每一臂各一份；不碰真产物）：

| 对照 | 怎么造 | 必须看到 |
|---|---|---|
| Q1·没有一格变 | 手造 r1 形态的 P1、P2、P3 片段与逐字相同的今天那一份，只改钉值格（计数从 3 变 5、峰值变） | 判定格变了 0 格；钉值格差逐格列出 |
| Q1·有格变且归得到 | 同上，改一格判定格（某行 `category_registration_batch_1` 从 `counted` 变 `untouched`），手造参照树让那一行落点函数的指纹不同 | 报 1 格、G4 中 |
| Q1·有格变有的归不到 | 同上，改一格判定格，七种都不中 | 报 1 格「归不到」 |
| Q1·只在一边 | r1 那一份有第 236 行、今天那一份没有 | 那一行的「在不在」格变了、G1 中 |
| Q1·计数过 0 | 某行 `O2_red` 从 0 变 1 | 判定格变了 1 格（「是否大于 0」那一格） |
| Q2·不多 | D 上两边漏判集逐行相同 | 每列多 0 行 |
| Q2·多了 | D 上今天 O2 多漏一行 | O2 列多 1 行、报「有一列多了」 |
| Q2·一出一进 | D 上 O1 一行出、另一行进（个数相同） | O1 列「多了」1 行、「少了」1 行、个数之差 0 |
| Q2·缺一边 | D 候选里一行 r1 有 `q1_row`、今天未触达 | 那一行不进 D，单列「不够判」 |

**阴性对照**：基线与空变异两条流、批 1 批 2 上 O1 三个子列、O2、O3 严格与暂代判红都是 0 处；空变异判未触达（K2）。

### 5.3 真实基线

r1 留在 git 里的那几份读数（P1–P3）就是逐格比的另一边，原样读、不重判（问题单「不问：r1 那一次的读数对不对」）；今天的不改实现那一趟（基线）是触达的参照与阴性对照。

### 5.4 装置写在哪（写死）

**写死成入库装置**：两个问题问的就是「今天 `crates/` 这份实现上，三条 oracle 对这张名单逐行判成什么」，另写一份独立模型答不了。用的是现成的入库装置 `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs`，**不改它**（它的输出钉在 E171 的复跑行上，改它就改了 E171 那几份产物的复跑）。它要读的 `crates/` 常量是 E171 第 1 次重跑登记 5.3 那一批本地常量（盘数、盘字节数、流一流二的段长表、写数、根槽写数、全量与快档态数……），装置里早已写成本地常量、`baseline` 一开头与 crates 回比（S1，装置第 7746 行判决行的 `s1_failed_checks`）；这一次不加新的。今天的树上 S1 回比不过时走停机 S1，出路在那一条里写死（不改 E171 的装置，另起本实验自己的入库装置）。
**新写的一律走研究一侧** `scripts/experiment-apparatus/e201_three_oracle_collusion_full_table_on_the_current_tree/`（Python，不与 `crates/` 共用代码），见 5.5。
装置编出来一律 `cargo build --release --features gpu -p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes`（装置走给定状态引擎这一判段入口，第三节），驱动起它的那一层文件头写 `# task-server-gpu-arguments: run`，卡由服务分；判据里有一条：装置的标准输出与它写出的每份产物里 `grep -c NOT_RUN` 合计得 0（`.gz` 用 `zgrep -c NOT_RUN`），`NOT_RUN_<原因>` 行与 `CRASH_AMPLIFICATION_NOT_RUN` 行都算，不满足的那一份按本次未跑报（作废 Z6）。

### 5.5 复用什么、不复用什么

| 东西 | 怎么用 | 为什么 |
|---|---|---|
| 入库装置（E171 那一份） | 原样编、原样跑 `baseline`、`row`、`--positive-control`、`stream-constants` | 5.4 |
| E170 导出装置 `e170_o3_spec_executor_export` | 原样：`dump` 给停机 S2 比流一流二的八个文件；`bad-states-second-run` 给 O3 阳性对照现导补丁 | 与 r1 同一种做法（r1 登记 5.2、修订 8 第 2b 步） |
| `segment_two_comparator.py`、`segment_one_comparator.py`（只调 `q1_lines`、`row_column_lines` 与产出 `q2_named_row`、`q3` 的函数）、`collusion_matrix.py`、`run_directory.py`、`result_lines.py`、`list_check.py`、`bad_implementation_table.py`、`frozen_o3.py`、`o3_bad_image_judge.py`、`device_timing.py` | 按路径 import 或原样起，不改、不拷 | 两边同一份判法，r1 那一版与今天这一版的差由 G7 归因 |
| `row_driver.py` | 不原样用：照抄成新目录里的 `current_tree_row_driver.py`（文件头记来源 sha256），改四处：编装置去掉 `nice -n 19`、加 `--features gpu`；O3 阳性对照那一步去掉 `nice -n 19`；`--positive-control` 收全 5.2 那九样；快照从计划目录同步（读法 1） | 原样那一份在任务里套 `nice`（任务里面不许套包装）、编出来的装置没有 `gpu` 特性（配了 GPU 时判段报本次未跑）；改原文件会改 E171 第 2 次重跑与它的复跑行用的那一份；文件名不同是因为比对器按名字 `import row_driver`，同名会串 |
| `mutation_driver.py` | 只认本目录的话照抄一份进新目录（记来源 sha256），表换成本实验的 | 同上 |
| 修订 8 那一层（第三节列的二十几份：冻结树判法、占位与占位检查器、编排层、单元编排、多机入口、`reuse_library.py`、`cross_host_o3_check.py`、`shard_relay.py`、`count_reusable_units.py`、两份对照脚本） | 不用、不 import、不改 | 用户硬约束「不继承 r1 修订 8 的冻结层」；编排层按 sha256 互钉、要 H 占位与宣布行；`reuse_library.py` 是本地复用库，不在 prune-server（`.claude/rules/verification.md`「判定库只有一份」）；两份对照脚本的入口读 H15、H28 占位；分片三步已在服务里，不要 `shard_relay.py` |

**新写的**（每份带单测；全进本实验的变异表）：
- `full_table_run.py`：入口。文件头 `# task-server-shards: run` 与 `# task-server-gpu-arguments: run`。三步照服务的分片：**计划**（拷读法 1 的树进计划目录、记 `name=snapshot` 几行；核名单与锚点（按名字取几次、锚点命中几次，不编不跑）；按读法 10 算每行的键、查 prune-server；写 `plan.json` 两片，各行按推的用时由长到短交替分给两片，库里有同键的行不派、按 5.6 抽哨兵）；**各片**（先跑这一片的基线 `baseline --streams 1,2` 取 T，逐行限时 max(1800, ⌈3T⌉) 秒，照 E171 第 1 次重跑登记修订 2 第 2.5 节；片 0 另跑第二次基线（作废 Z1）、停机 S2 那一比、O3 基线与九样阳性对照两遍（基线一遍、空变异一遍）；两片都跑空变异（两片的基线与空变异互比是跨机对照，停机 S7）；再跑分到的行；线道数 = ⌊`SINGLEFS_THREAD_CAP` ÷ 6⌋，每线道 6 线程，每线道一棵树一个编译目录（放在服务给的 `CARGO_TARGET_DIR` 下各自的子目录）；每行判完把读法 10 那一份写进库）；**合并**（从库取回全部行、在合并自己的目录里摆成行输出根，起 `segment_two_comparator.py` 出 T1、合成 T2、调第一段那几个函数出 T3，再起下面几份）。同一条命令再交就是续跑：键相同的行从库里取。
- `whole_table_matrix.py`：整表每行、每条流、每批、每列（O1 三个子列、O2、O3 严格与暂代）的红态数（O1-对拍是调用数）、适用数、峰值、为正段数、期末值、单点数（`collusion_matrix.trajectory`），与每列的整表漏判集（读法 9 第一条的定义，整表上）。
- `contrast_against_r1.py`：读法 7 的配键、分类与逐格比；输入 P1–P3（核 sha256 与行数）与 T1–T3；每格一行，收尾每份产物各一行计数。
- `attribution.py`：读法 8 的 G1–G7 与宽归因；读参照树（`git archive 31e47115…` 落到草稿目录）、P0 的路径、P2、T2、r1 装置（照 G5 那一条取）、G5 与 G7 的两张对应表。
- `q2_misses.py`：读法 9 的 D、M_r1、M_today、多了、少了。
- 新的变异表 `scripts/mutation/tables/e201_three_oracle_collusion_full_table_on_the_current_tree.tsv`（第九节）。
- 机读登记：`.claude/gate.d/verdict-library-scripts.tsv` 加 `full_table_run.py` 一行（第三列 `prune-server（apparatus-items 族，键空间 apparatus-items/three_oracle_collusion_full_table_on_the_current_tree/…）`，自证 `reuse-three-questions`）；`.claude/hooks/selftest-registry.tsv` 登记各份 `--selftest` 的档。谁改这两张表由主 agent 定（不一定在执行员写范围里）。

### 5.6 谁跑什么、实现量、分段

| 跑什么 | 谁 | 依据 |
|---|---|---|
| 新目录的脚本、单测、变异表、两张机读登记的行 | 执行员写（`crates/` 一行不改） | 5.5 |
| 新目录的单测、`--selftest`、变异表（经照抄的 `mutation_driver.py`） | 执行员写进「要主 agent 跑的命令」，主 agent 经统一任务服务交（自证照 `.claude/hooks/selftest-registry.tsv` 登记的档） | 子 agent 不交任务 |
| 第一段整趟：`task-server/task submit -- python3 scripts/experiment-apparatus/e201_three_oracle_collusion_full_table_on_the_current_tree/full_table_run.py run`（服务认分片与要卡，起计划、两片、合并） | 主 agent，**重型**（编 checker 档装置、跑三百来趟装置、两台、要卡、推 1.5–3 小时） | 第十四节 |
| 第二段：同一个入口 `run --stream-three`（计划只排空变异与 r1 第一段那十五行、三条流） | 主 agent 定跑不跑，**重型** | 读法 11、未定项第 2 条 |
| `replay.sh` 里本实验的复跑登记行 | 提交时 | 全部实验复跑是重型 |

**哨兵**（`.claude/rules/test-script-discipline.md`「判定可复用」）：计划那一步取到 M ≥ 1 行库里同键的结论时，挑 k = max(2, ⌈0.02 × M⌉) 行现跑（按 sha256(快照摘要 ‖ 键) 由小到大取，挑法记进 `name=reuse_sample`），现跑的结论与库里那一条判定格逐格相同才算复用有效（作废 Z7）。M = 0 时 k = 0，照报。

**实现量**（推的，没量）：`full_table_run.py` 约 700–900 行（分片三步、快照、键与库、两片的单元次序、线道、哨兵、跨机对照、复用账、自证与弄坏开关），`current_tree_row_driver.py` 照抄 763 行、改约 40 行，`whole_table_matrix.py` 约 150 行，`contrast_against_r1.py` 约 400 行，`attribution.py` 约 400 行（参照树、单元指纹比、两张对应表、P0 路径），`q2_misses.py` 约 150 行，单测合约 900 行，变异表 14 行。不要另写检查器、触发式重算或崩溃枚举；装置不改。

**分段**（每段交回之后主 agent 对着问题单两行判续不续；一行开着的都不剩就停）：

| 段 | 做什么（按能最先让某一行够判的次序） | 补哪一行 | 补完够不够判 |
|---|---|---|---|
| 第一段 | 先写并单测 `contrast_against_r1.py`、`q2_misses.py`、`attribution.py`（只读 P0–P3 与手造输入，不等整表）与 5.2 第二张表的阳性对照；再写 `full_table_run.py` 与驱动；主 agent 交整趟；合并出 T1–T3、整表矩阵、逐格比、归因、Q2 | Q1 与 Q2 在流一、流二上的全部格 | Q1 够判（流一、流二）；Q2 在 D 上够判（未定项第 1 条认了的话）；流三那几格写「第二段未跑」 |
| 第二段 | 空变异加 r1 第一段十五行，三条流；只补 P3 里流三那几格 | Q1、Q2 流三那一半 | 够判（只在那十六行上） |

## 六、报哪些量与各自的判据

每个量各占一行、各报各的判定，合取留给主 agent 写问题单状态时做。问题单两行的候选本身是计数（「有没有一格 / 一行」），照 `.claude/singlefs-ai-sop/rules/test-discipline.md`「门槛不能是臂定义的同义反复」，每一行另配有效性门槛（跑完、对得上键、阳性阴性对照、D 非空），由它定那一行够不够判。

| 量 | 怎么算 | 门槛与判定 | 对应问题单哪一行、取什么值翻面 |
|---|---|---|---|
| 有效·前置 | 片 0 两次基线（Z1）、两片基线的 S1/S3/S5/S9（装置判决行）、S2、O3 冻结核对（每片开头收尾）、NOT_RUN 合计、快照与名单与研究一侧 sha256 每片开头收尾等于计划记的 | 全过才往下判 Q1、Q2；任一不过照第十、十一节那一条办 | Q1、Q2 前提 |
| 阳性 | 5.2 第一张表九样 × 基线、空变异两遍的读数 | 每一格等于 K3 那个精确数；不等走 F1 | Q1、Q2 前提（那一列在这条管子上接没接对） |
| 阴性 | 基线、空变异两条流批 1 批 2 上各列判红处数；空变异触达否 | 都是 0、空变异未触达（K2）；不是走 F2 | Q1、Q2 前提 |
| 有效·整表 | T1 的 `name=row` 行数；行集；每行的 `status` 与 `ran` | 行数 = 278 行，行集 = 读法 6 那一个（A1）；不等走作废 Z3 或 F3 | Q1 够判条件「278 行在今天的树上跑完」 |
| Q1-配键 | P1–P3（只取第一段那几种）每一种行种在 T1–T3 里有没有、键对上几把、只在一边的键几把；读法 7 的分类表里每个（行种, 字段）在今天那一份里在不在 | 判定格所在的行种两边都在才比；r1 有而今天没有的判定行种或字段走 F5 | Q1 前提 |
| Q1-判定格差 | 读法 7：逐格比，判定格变了的格逐格列（产物、行种、键、字段、r1 值、今天值），按产物与行种各计一数、合一个总数 c | c = 0 → 「没有一格变」；c ≥ 1 → 交下一行定「归得到 / 有的归不到」 | Q1：任一判定格变了即离开「没有一格变」 |
| Q1-归因 | 读法 8：c 格每格的 G1–G7 命中表；一种都不中的格数 u；查不了的格数 q（F6） | c ≥ 1 且 u = 0 且 q = 0 → 「每一格都归得到」；u ≥ 1 → 「有的格归不到」（逐格列）；u = 0 而 q ≥ 1 → 两者之间不够判（F6） | Q1：有一格七种都不中即翻向「有的格归不到」 |
| Q1-宽归因 | 读法 8 末条：u 格每格的闭包一层指纹变了的单元数 | 描述量 | Q1（写结论时用） |
| Q1-钉值格差 | 钉值格变了的格数与逐格列表（读法 7） | 描述量，不进判定 | Q1 |
| Q1-今天独有 | 整表 O3 两列、整表矩阵（`whole_table_matrix.py`）的格数 | 描述量 | Q1（「逐行过三个 oracle」那一句的今天一侧） |
| Q2-域 | 读法 9：D 与 R7 里每一行不进 D 的原因 | |D| ≥ 1 Q2 才判；= 0 走 F4 | Q2 前提 |
| Q2-判 | 每一判定列（O1、O2、O3 严格、O3 暂代）在 D 上的 M_r1、M_today、多了的行、少了的行、个数之差；O1 三个子列另报 | 有一判定列「多了」≥ 1 行 → 「有一列多了」（逐列逐行列出）；每列都是 0 → 「三列都不多」 | Q2：一列在 D 上多出一行漏判即翻向「有一列多了」 |
| Q2-整表 | 今天整表每列的漏判集（读法 9 末条） | 描述量（r1 一侧写「git 里没有逐列读数」） | Q2（未定项第 1 条） |
| 跨机 | 两片的基线导出（去掉耗时）与两片空变异的行目录（去掉耗时）逐文件比；两片各落在哪台（`TASK_SERVER_SHARD_HOST`） | 两片在两台而有一份不同走停机 S7；两片在同一台记「本次未跑」 | Q1、Q2 前提（两台判得一样） |
| 复用账 | 每行现判还是复用、哨兵 k 与结果；现判数 N + 复用数 M = 计划登记的行数 | N + M 不等于登记数退 1；哨兵有一行判定格不同走作废 Z7 | Q1、Q2 前提 |
| 附带（够判后不跑） | 第二段（空变异加十五行共 16 趟、三条流，只补流三那几格）；原登记 s4 两份整表产物的逐格比；P3 的 Q4、Q5 各格 | 照数报 | 第二段对应 Q1、Q2 流三那一半；后两样不对应任何一行 |

**写结论时**：Q1 判「没有一格变」只能写成「在 P1–P3 有的那些判定格上（流一、流二），今天这份实现与 r1 判得一样」，不写「E171 的结论在今天的实现上不变」——r1 没有的格（整表 O3、计入分母的行的逐列读数）不在比的范围里。「归得到」只说「那一格落在读法 8 某一种改动上」，不说「那一处改动造成了它」：因果要另走三方。Q2 判定只说 D 上那几行。

## 七、钉绝对值的断言

每一条互比（今天对 r1、两片对两片、库里对现跑）旁边配一条绝对值：比对器、归因、两边产物一起错了，下面这几条照样会红。

**出自被测条款或问题单本身**（不符时走「条款或实现可能错」——两边都查，交主 agent，不作废）：

| # | 断言 | 出处 |
|---|---|---|
| K1 | O1 恰三个子列，O2、O3 各一列（O3 两种跑法）；第二截、记录核对器、手写版本表那一份不出现在三列里 | D13（验证路线） 已定项 6 表；E171 第 1 次重跑登记读法 1 末项与第七节 A2 |
| K2 | 基线与空变异两条流批 1 批 2 上 O1 三个子列、O2、O3 严格与暂代判红都是 0 处；空变异判未触达 | 原登记 A3「正常实现、正常镜像必须三条都放行」；E171 第 1 次重跑登记 A1 |
| K3 | 5.2 第一张表九样的精确数，基线与空变异两遍都是：O1-对拍 1 次调用；卸载 1 次；O1-模型崩溃判官 2 态；O1-RefFS 2 态；O2 1 态；O3 判「不在」；交回值 1 次调用且那一次 `agrees`；内存对象 1 次调用；panic 截断前判过的调用 1 次 | E171 第 1 次重跑登记第七节 A8（第 469 行，流一、流二那一种） |
| K4 | R7 = {15, 23, 42, 242, 269, 270, 271}；r1 第一段的行集 = {0, 15, 23, 42, 49, 110, 125, 155, 161, 162, 174, 242, 252, 269, 270, 271} | E171 第 1 次重跑问题单 Q1 七行；`segment_one_comparator.py` 第 31 行；E171 第 2 次重跑登记 A23 |

**独立算出、用命令核过的**（不符就作废那一格；命令与原样输出在第十三节）：

| # | 断言 | 怎么算的 |
|---|---|---|
| A1 | 今天两张名单 115 + 163 = 278 行，sha256 `18cdb0c6…`、`57c981f0…`，两个目录逐字节相同；第一列之并 = {1, …, 281} 去掉 {33, 107, 236}，没有重号；每行 6 列 | `wc -l`、`sha256sum`、`cut -f1 \| sort -n`、`seq 1 281 \| comm`、`uniq -d`、`awk '{print NF}'` |
| A2 | 在 `crates/mutations.tsv`（`07fcc8ca…`）上 278 个名字各命中 1 次；计划那一刻的表 sha256 不同时在那张表上重算、照实写（表变了是事实，不作废） | 第十三节 awk |
| A3 | P1 451 行、P2 565 行、P3 811 行、P0 125 行，sha256 照读法 2；P1 恰 281 条 `name=row`，P2 恰 281 条 `row row_number=` | `sha256sum`、`wc -l`、`grep -c` |
| A4 | 行集对齐：只在 r1 的行 = {33, 107, 236}，只在今天的行 = 空 | A1 与 A3 两个行集求差 |
| A5 | 逐格比的自检：每份产物每种行种，比对脚本数出的键数与合并那一步另用 `grep -c '^E7RESULT name=<行种> '` 数的相等；每份产物「相同 + 变了 + 只在一边」= 那一份的格总数 | 两份各算 |
| A6 | Q2：每列「多了」「少了」各用集合差与逐行循环两种写法算一遍，相等；M_r1、M_today 都 ⊆ D | 两份各算 |
| A7 | 存进库的那一份够用：比对器在只放读法 10 那一份的行目录上与在完整导出上出的 T1 那几行逐字相同（自证一格；真跑时对空变异与每一行哨兵各核一次） | 两份各算 |
| A8 | 参照与 r1 装置：提交 `31e47115…` 里 `crates/` 与 `Cargo.*` 的 `.rs`、`Cargo.` 文件 270 份；`git show f8a9ddaab:<装置>` 的 sha256 是 `ab35105f861d509457296741d2c60515a3a94b9fefe77944f03d1b358afd45df`，删第 3478–3484 行（其中一行含 `SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults`）之后是 `67e7a5498425ec3f27379ba8f0e5cd666faaeaac0817006d6938956d6419a156`，等于 r1 那一趟的装置（G5 记 `exact`） | `git ls-tree`、`git show \| sed '3478,3484d' \| sha256sum`（第十三节） |

## 八、轨迹与几何敏感性

**轨迹**：被谓词消费的量是「这一行在这一列（子列）上红不红」（Q1 的判定格④与 Q2 的漏判都吃它）。整表矩阵（`whole_table_matrix.py`）对每行、每条流、每批、每列报按段号排开的红态数的**峰值**（红态最多的那一段与它的数）、**为正段数**、**期末值**（最后一段的红态数）与单点红的段数（只在一段里红且那一段只红一态）；P3 的 `q1_row` 带同样几样，七行两边并排。正文写某行「被 X 抓」时引峰值与为正段数，不写「恒红」「从来不放」，除非为正段数等于这条流有写的段数。基线与空变异的同几样必须全是 0（K2）。

**几何敏感性**（`.claude/rules/mutation-sampling.md` 第六类）：

| 判决 | 主取样点 | 方向相反的取样点 | 怎么报 |
|---|---|---|---|
| Q1 判定格与 Q2 漏判 | 批 1：每段两端点、六个等分点、全持久态、带撕裂的态（偏向段界与撕裂） | 批 2：每段七个居中点，不取段界、不取全持久态、不专挑撕裂 | Q1-判定格差、Q1-归因、Q2-判在批 2 上各重算一遍、并列；两批判得不同记「不稳定」，相同记两点 |
| 「计入分母」的读法 | 登记读法（`category_registration_*`） | 原登记读法 5（`category_original_reading_5_*`，自崩一律不进） | Q2 的 D 与漏判在两种读法上各算；Q1 的类那几格本来就两种都比 |
| 流 | 流一、流二 | 流三（第二段，十六行） | 第二段跑了才报；只在流三上变的格单标 |
| 判别力自证 | — | — | `contrast_against_r1.py` 单测：把「计数取是否大于 0」挪成「计数逐值比」，5.2「Q1·没有一格变」那一份必须由「0 格」变成「≥ 1 格」；`q2_misses.py` 单测：把「多了按集合」挪成「按个数」，5.2「Q2·一出一进」那一份必须由「多了」变成「不多」 |

## 九、变异

全进 `scripts/mutation/tables/e201_three_oracle_collusion_full_table_on_the_current_tree.tsv`，经新目录里的变异驱动跑（单测模块是新目录的 `test_*.py`）。入库装置不改，不加装置一侧的变异行。每条写明在哪个取样点上改变输出：

| # | 改哪 | 改成什么 | 在哪个取样点上改变输出（会红的那一格） |
|---|---|---|---|
| V1 | `contrast_against_r1.py` 计数格取「是否大于 0」 | 逐值比 | 5.2「Q1·没有一格变」：原式判定格差 0，变异 ≥ 1 |
| V2 | 同上 | 取「是否大于 1」 | 5.2「Q1·计数过 0」（`O2_red` 0 → 1）：原式 1 格，变异 0 格 |
| V3 | `q1_row` 的键 | 只按（`row`, `batch`），不带 `stream` | 手造同一行流一红、流二不红而 r1 反过来：原式 2 格变了，变异配错键、报 0 或报重键 |
| V4 | 只在一边的键 | 不当一格 | 5.2「Q1·只在一边」：原式第 236 行那一格在、G1 中，变异没有这一格 |
| V5 | 读法 7 分类 | 判定词字段（`verdict`）归钉值格 | 手造 r1 `q2=yi`、今天 `q2=jia`：原式判定格 1 格，变异 0 格 |
| V6 | `attribution.py` G2 | 比名字，不比变异行 sha256 | 手造一行名字同、替换文不同：原式 G2 中，变异不中（那一格成「归不到」） |
| V7 | G4 | 比整份文件 sha256，不比所在函数的单元指纹 | 手造同文件另一个函数变了、落点函数没变：原式 G4 不中，变异中 |
| V8 | G4 | 不标 `reference_approximate` | 手造落点文件在 P0 那几份路径里：原式那一条带标记，变异不带 |
| V9 | G6 | O1-RefFS 一列不查 RefFS 那几个入口 | 手造只有 `allowed_set_for_a_crash_in` 指纹变了、RefFS 格变了：原式 G6 中，变异「归不到」 |
| V10 | `q2_misses.py` 漏判 | 取批 2 | 手造一行批 1 不红、批 2 红：原式漏、变异不漏 |
| V11 | 「多了」 | 按个数 | 5.2「Q2·一出一进」：原式多了 1 行，变异不多 |
| V12 | D | 不要求 r1 那一边计入分母 | 手造 R7 里一行 r1 `untouched`：原式不进 D，变异进 D 而 r1 一侧当「全漏」 |
| V13 | `full_table_run.py` 键 | 不含实际改坏用的变异行 sha256 | 自证 `reuse-three-questions` 的「改闭包里一个单元」那一问：原式现判 1 条，变异现判 0 条（复用了不该复用的） |
| V14 | `full_table_run.py` 哨兵 | 不挑 | 自证哨兵那一格：库里同键的结论被改坏一处，原式现跑的哨兵判出不同、作废，变异照复用 |

`current_tree_row_driver.py` 的四处改动另配两条弄坏开关（写在它文件头，自证红在点名的那一格）：编装置不带 `--features gpu`（假 cargo 记下的参数里没有 `gpu`）；编装置的命令前面有 `nice`（假 cargo 记下的 argv[0] 不是 cargo）。变异全抓才算这套比法有判别力；没红的照 `.claude/rules/mutation-sampling.md`「没红的三类」分，不直接记盲区或等价；无效的照第八类另列、换一条补上。

## 十、失败条款

| # | 条款 | 什么观测会让它触发 |
|---|---|---|
| F1 | 某一样阳性对照不是 K3 那个精确数：那一列在这条管子上接错了，Q1、Q2 里那一列的格这一趟全部作废，不记成「这条 oracle 盲」；修好重跑 | 「阳性」那一行有一格与 K3 不等（基线或空变异任一遍） |
| F2 | 阴性对照上任一列判红：不是结果，是 oracle 误报或装置与实现对不上——两边都查（`.claude/rules/implementation-first.md` 第 4 条），既不作废也不当结果，交主 agent | 「阴性」那一行的数大于 0，或空变异判触达 |
| F3 | 整表里有行没跑成（驱动报错、导出缺流）：那几行的判定格记「今天没跑成」，Q1 里只在一边的格照记；行数对得上而有行没跑成，Q1 照判、写明行号 | 「有效·整表」那一行有行 `ran=false` 而类不是读法 5 那几类（编不过、锚点不唯一、找不到、自崩、超时） |
| F4 | D 为空：Q2 记「不够判（r1 有逐列读数的行在今天的树上都不进分母）」，不报多了少了；整表那一边照报 | 「Q2-域」|D| = 0 |
| F5 | r1 有、今天没有的判定行种或字段（今天的比对器改过了）：那一种的格记「未比」，Q1 写「c 只数了能比的格」并列出未比的种类；判定行种整个未比的，Q1 不够判 | 「Q1-配键」列出 r1 一边独有的（行种, 字段）在读法 7 的分类表里是判定格 |
| F6 | 归因查不了（参照里找不到 r1 那一版研究一侧文件、单元切不开、表里点名的函数在两边都找不到）：那一格记「查不了」，不算进「归得到」也不算进「归不到」 | 「Q1-归因」q ≥ 1 |
| F7 | 第二段没跑：流三那几格记「第二段未跑」，不当相同 | 主 agent 定不跑第二段 |
| F8 | 名单与今天的变异表对不上（计划那一刻有行按名字取不到或锚点不唯一）：那几行照原登记 5.2 补位、照读法 5 记类；不是这一次的失败，照实写 | 计划那一步核名单的读数有命中不是 1 的行 |

## 十一、作废条款与停机条款

**作废**（那一次的输出不算结果；修好之后重交同一条命令，不改判据）：

| # | 条款 | 触发观测 |
|---|---|---|
| Z1 | 基线不确定 | 片 0 两次基线的导出逐文件比（`impl-verdicts.tsv` 去掉耗时那一列，`oracle-verdicts.tsv`、`call-outcomes.tsv`、`call-return-values.tsv` 去掉 E171 第 1 次重跑读法 4 写明拿掉的对象），有一份不逐字节相同 |
| Z2 | O3 不是冻结的那一版 | 某一片开头或收尾 `frozen_o3.freeze_differences` 有一处不同：那一片的 O3 列全部作废 |
| Z3 | 编的不是计划记的那一份树 | 某一片开头或收尾，线道树的快照摘要、`crates/mutations.tsv`、两张名单或第三节研究一侧各文件的 sha256，与计划那一步记的不同；或 T1 的 `name=row` 行集不等于 A1 那一个 |
| Z4 | 逐行驱动带着上一行的变异编 | 驱动退 5（编前编后核对不过），或 T2 有一行 `packages_checked_rebuilt` 不是真：那一行作废、重交时现跑 |
| Z5 | 比的不是登记的那几份 r1 产物 | P0–P3 有一份的 sha256 或行数与读法 2 不同 |
| Z6 | 判段没跑 | 某一个装置单元的标准输出与它写出的产物里 `grep -c NOT_RUN`（`.gz` 用 `zgrep -c`）合计不是 0：那一个单元按本次未跑报，那一行不进 Q1、Q2 |
| Z7 | 复用无效 | 有一行哨兵现跑出的判定格与库里那一条不同：这一趟复用来的行全部作废，关掉复用（`run --without-reuse`，写进库照写）重交 |
| Z8 | 统一任务服务的结局不是跑成 | `task wait` 末行 `TASK_SERVER_RESULT` 的 `exit=` 不是 0、或带 `shard=`：那一次的产物不进判定；已入库的行照留，重交同一条命令时复用 |

**停机**（装置与 `crates/` 对不上，或两台判得不一样：两边都查，既不作废也不当结果，交主 agent）：

| # | 条款 | 触发观测 |
|---|---|---|
| S1 | 装置的本地常量在今天的树上不成立 | 任一片基线判决行 `s1_failed_checks` 不是 0（装置退 3）。照 E171 第 2 次重跑登记 S20 引的 E171 第 1 次重跑修订 8 第 8.2 节甲乙丙三类归：甲类有一项不过，停、两边都查；只有乙类不过，**不改 E171 的装置**（它的输出钉在 E171 的复跑行上），由主 agent 定另起本实验自己的入库装置 `crates/singlefs-checker-tier/src/bin/e201_three_oracle_collusion_full_table_on_the_current_tree.rs`（照抄 E171 那一份，只改乙类常量的值与出处注释，换装置名），那时写一条登记修订、重交；改装置名之后 G5 的「今天的装置」读作新的那一份 |
| S2 | 装置的流一、流二与 E170 导出装置的不是同一份 | 原登记 S2 原样：同一份树上 E170 导出装置 `dump` 写出的八个文件与基线导出的 sha256、`impl-verdicts.tsv` 去掉耗时列之后逐字节比，有一份不同 |
| S3 | O2 两处取法不一致 | 基线判决行 `s3_o2_judgement_mismatches` 不是 0 |
| S4 | 推模型那一层与实现对不上 | O1 不适用出现 E171 第 1 次重跑读法 6 那三处之外的来源（装置的 S5）；或基线判决行退 3 而点名 S5 |
| S5 | RefFS 的接法与翻译层对不上 | 基线判决行 `s9_failed_checks` 不是 0 |
| S6 | 计划拷进来的树编不过 | 任一片基线那一步 `cargo build` 退非 0（编空变异与基线用的是没改坏的树）：主 agent 另挑一刻重交，或等别的会话那几处改动落定 |
| S7 | 两台判得不一样 | 两片落在两台（`TASK_SERVER_SHARD_HOST` 不同），而两片的基线导出或空变异行目录去掉耗时之后有一份不逐字节相同；同一台上 Z1 过。触发时这一趟第二台跑的行全部不作数，交主 agent（`.claude/agent-common.md`「所有任务都要多机可用」：派到某一台就红是缺陷，要修） |

**够判停机**：每段交回附问题单两行的表（已够判 / 还差什么 / 剩下的量能不能让它翻面）。第一段交回之后：Q1 在流一、流二的判定格上应够判（或照 F3、F5、F6 写明哪几格不够）；Q2 在 D 上应够判（或照 F4）；流三那几格写「第二段未跑」。主 agent 定不跑第二段的，第二段的量逐条标「主 agent 定不跑」；第一段已经让问题单两行都够判、而主 agent 认了未定项第 2 条的默认读法的，第二段标「够判后未跑」。附带量一律够判后不跑。

**比对对象**：不适用固定形态的比对对象表——这不是 E171 的重跑登记，没有「这一次要与之比的上一次产物」那种逐份重出；与 r1 比的四份产物在读法 2 点名，核法在 A3、Z5。

## 十二、修订

（留空；装置写之后、产物之前由执行员写，只许收严或补臂。读法 7 的分类表、读法 8 的 G5 与 G7 两张对应表（各记 sha256，都在今天任何产物之前）、计划那一刻的 `crates/mutations.tsv` sha256 与名单核对读数、落产物那一天的产物名，抄在这里。）

### 修订 1（2026-11-03，执行员；写在装置写之前，单测、变异、自证与任何产物都还没跑；三条都照派发提示，依据是用户 2026-11-02 在弹窗里定的三条，问句选项记在 `records/2026-10-31-删兼容旧版本代码.md` 第 18 批）

1.1 **未定项第 1 条关掉：认 D 这个范围。** 派发提示原句：「Q2「认这个范围（最多 7 行）」：Q2 只判在两边都有逐列读数的那几行上；整表上三列各自漏判的行照今天这一趟出，只描述、不与 r1 比。」所以 Q2 的判定照读法 9 只在 D 上做（D ⊆ R7，至多 7 行）；整表上今天每一列的漏判集照第六节「Q2-整表」那一行出、只描述，不与 r1 比、不进 Q2 的判定。读法 9 与第六节的判据一字不改；问题单 Q2 的问法不改。

1.2 **未定项第 2 条关掉：流三那几格算进够判范围，两段都跑。** 派发提示原句：「第二段「两段都跑」：流三那几格也算进够判范围，第一段之后接着起第二段（重型）。」所以：
- 第一段交回时 Q1、Q2 在流一、流二上照第六节判；流三那几格写「第二段未跑、等第二段」，问题单两行**都不写够判**，直到第二段跑完、并进比对与归因（收严：够判要两段都齐）。
- 第二段（读法 11，`run --stream-three`）由主 agent 在第一段交回之后接着交，**重型**；第十一节「够判停机」里「主 agent 定不跑第二段」与「第二段标够判后未跑」两种出路这一次不用；第十节 F7 只在第二段交了而没跑成时触发（那时流三那几格记「第二段未跑」，问题单两行照 F7 写不够判）。
- 第二段的比对照读法 7 的配键与分类：P3 里 `streams=three_streams` 的 `q1` 行、流三的 `q1_row` 行与三流的 `q2_named_row` 那几行由「不比」改成与流一、流二同一套判定格比（只对第二段的十六行），归因照读法 8；Q2 补一行判据「Q2-流三」：D 与漏判在第二段上另按「流一、流二、流三批 1 任一处判红」重算一遍（r1 一侧取 P3 三流严格那一圈的 `q1_row`），门槛与「Q2-判」同形（有一判定列在 D 上多了 ≥ 1 行记「有一列多了」），与第一段的「Q2-判」各占一行、各报各的判定，合取留给主 agent。

1.3 **「今天的树」认交任务那一刻的主工作区，不设宣布行。** 派发提示原句：「「今天的树」「认：交任务那一刻的主工作区」：产物里记下那一刻的 HEAD 与工作区改动清单，归因时用；不设宣布行。」与读法 1 同义，补一条用法：计划那一步记的 `git rev-parse HEAD` 与 `git status --porcelain -- crates Cargo.toml Cargo.lock .cargo` 全文（`name=snapshot` 那几行）照抄进归因产物的头几行；读法 8 的「今天的树」取计划目录里拷进来的那一份（不取合并那一刻的主工作区），G4、G6 的今天一侧单元指纹都在这一份上算；某一格归到的文件在那份改动清单里的，那一条另标 `uncommitted_at_submission`（描述，不改七种的判定）。第二段同样记它自己那一刻的 HEAD 与清单，两段的快照摘要不同时照实写、各按各的树归因。

其余读法、判据、作废与停机条款不动。

### 修订 2（2026-11-03，执行员；写在今天任何产物之前，单测、变异与自证也都还没跑；依据读法 7「执行员在今天任何产物出来之前……写成一张表进第十二节修订、记 sha256，之后不改」与读法 8 G5、G7 同一句）

2.1 **读法 7 的分类表**在 `scripts/experiment-apparatus/e201_three_oracle_collusion_full_table_on_the_current_tree/cell_classes.tsv`（215 行，sha256 `4c053fa2a7325ec16a7983121231ade70ca5269547baff922654162aca93d39a`；`contrast_against_r1.py` 的 `CLASSES_SHA256` 钉同一个值，对不上退 2、不比）。只读了 P1–P3 的字段名（行种与字段名，没读值），P1–P3 每个（行种, 字段）都归了类（第十三节那种 awk 数的，没有归不了的）。各比的行种归法（键 / 判定格① / 判定格③ 集合 / 判定格④ 红的计数 / 钉值格；没列的行种整种不比）：

| 产物 行种 | 键 | 判定格① | 判定格③ | 判定格④ | 钉值格 | 不比 |
|---|---|---|---|---|---|---|
| P1 row | row | status、ran、write_stream_differs、call_outcomes_differ、recovery_report_differs_batch_1/2、ignored_pass_differs_batch_1/2、call_panic、panicked_before_the_first_call、category_*（四种读法 × 批） | — | — | device_exit、substituted_from、failed_attempts、mutated_row_name、judged_calls_before_the_first_panic、recovery_panic_states | — |
| P1 substitution / not_located_row | row | status | — | — | failed_attempts、substituted_by / substituted_from、name_hits_in_the_frozen_table | — |
| P1 untouched_row_red | row | — | — | O1-对拍、O1-模型崩溃判官、O1-RefFS、O2 | — | — |
| P1 q2_set_row | row | o1_red | — | o1_comparison_red_calls | calls_* 三个计数 | — |
| P1 q2_set_call、q2_named_row_call | row, stream, call | observation_differs、o1_comparison | — | — | label、fields、objects | — |
| P1 q2_named_row | row | ran、category、touched、in_the_set | — | o1_comparison_red_calls | calls_* 三个计数 | — |
| P1 q2_observation | （一条） | — | — | — | 全部计数 | — |
| P1 q2 | （一条） | verdict | rows_o1_red、rows_o1_not_red | o1_red_rows | set_rows | — |
| P1 q3_denominator | reading, batch | — | — | — | 各类计数 | — |
| P1 q3_denominator_class | reading, batch, class | — | row_numbers | — | rows | — |
| P1 q3_self_crash_row | row | registration、original_reading_5 | — | — | where | — |
| P1 q3_row_125 | （一条） | 四种读法 × 批的类与 counted_*、ran、category | — | — | row | — |
| P1 verdict | （一条） | q2、row_125_counted_registration_batch_1 | rows_not_located | q2_o1_red_rows、untouched_rows_with_a_red_column | 其余计数 | unstable_memory_objects、stability_baseline_files_differing |
| P2 row | row_number | status、line_matches_the_registration | — | — | row_name、device_exit、substituted_from、mutated_row_name、mutation_line_sha256、failed_attempts、failed_attempt_*、packages_checked_rebuilt | lane、耗时四样、compile_kind、bulky_dump_files_removed；o3_status、o3_exit、o3_streams 记今天独有 |
| P3 q1 | reading, streams, batch | verdict | invalid_rows、all_quiet_rows | — | rows | — |
| P3 q1_row | row, stream, batch | — | — | o1_comparison_red_calls 与五列的 `_red` | 峰值、为正段数、期末值、单点数 | — |
| P3 q2_named_row | row, streams | ran、status、touched | — | o1_comparison_red_calls | calls_* 三个计数 | — |
| P3 q2_named_row_call | row, stream, call | observation_differs | — | — | fields、objects | — |
| P3 q3 | （一条） | ran、status、call_panic、touched、verdict | — | — | row、judged_calls_before_the_first_panic、states_judged、recovery_panic_states | — |
| P3 verdict | （一条） | q1（第二段比）、q3（第一段比） | — | — | — | 其余字段 |

   - 整种不比：读法 7「不比」点名的 P1 七种与 P3 的 `a1*`、`a8`、`a11`、`z1*`、`q4*`、`q5*`、`o3_third_stream_coverage`、`config`、`done`。
   - 归类里要主 agent 知道的一处：P2 的 `o3_status`、`o3_exit`、`o3_streams` 照封闭词表本该是判定格①，归成「今天独有」——r1 第二段整表不跑执行器，这三个字段两边必不同，说的是整表 O3 那一列跑没跑，读法 7 末条把整表 O3 两列记今天独有；归成判定格会让 278 行每行一格「变了」、七种都归不到。
   - 只在一边的字段：一把键两边都在而一个判定字段只在一边的，那一格按「缺」对「有值」记变了；r1 这一种有、今天这一种一条都没有的判定字段与判定行种照 F5 记「未比」。

2.2 **读法 8 G5 的对应表**在同目录 `device_function_map.tsv`（65 行，sha256 `95b93eb4bf7be327215bbfcdf80bdaf890d6a36e3db1ea9be2edcc955cbce0b9`，`attribution.py` 的 `DEVICE_MAP_SHA256` 钉同一个值）：八组装置函数（导出、触达、panic、O1-对拍、崩溃判官、RefFS、O2、交回值）按（产物, 行种, 字段）对应，点名的 34 个不同的函数名在今天的装置里各恰好一个 `fn`（`grep -cE "fn <名>(<|\()"` 逐个数的）；定位、补位那几格写「-」（装置不碰，G5 不适用）。

2.3 **读法 8 G7 的对应表**在同目录 `research_function_map.tsv`（36 行，sha256 `9761c84f03d4295f28cf157279bf9e09010e0f0aee84049141d8e8f017d56a6a`，`RESEARCH_MAP_SHA256` 钉同一个值）：九组研究一侧函数（读导出、触达、分类、O1-对拍、按态红、交回值、第二段行、第一段行、驱动）。点名的 51 个不同的函数在今天各份文件里各恰好一个 `def`（`grep -cE "^def <名>\("` 逐个数的）。r1 那一版：`revision_8_research_side_of_the_revision_6_run.json` 的 `files` 只有 `row_driver.py`、`run_directory.py`、`bad_implementation_table.py`、`frozen_o3.py`、`device_timing.py` 等驱动一侧，两份比对器与 `collusion_matrix.py` 不在里面，所以落到它们的格 G7 一律记 `research_reference_missing`（查不了，读法 8 G7 原句）。

2.4 **归因的落法**（读法 8 原文没写死、这一次照下面算，都不放宽七种）：
- 一格合起来的结局：七种里有一种中记「归得到」；都不中而有一种查不了记「查不了」（F6）；都不中、都查得了记「归不到」。一种之内几行、几个入口、几份文件合起来：有一处中就中。
- 集合字段（行号集合）那一格的行取两边集合的对称差；单条的行种与计数那几格没有行，G1–G4 记不适用。
- G4：今天锚点（变异行原文在今天那份文件里第一次出现处的行）所在的最里层 `fn` 单元，按同文件、同嵌套限定名在参照树里找；变异行在今天的表里找不到的行（定位失败、补不上）G4 不适用；今天切不出单元记查不了。
- G6：O1-对拍的 `answer_*` 取这一行今天流一、流二导出里出现过的调用名接上 `answer_`（`model.rs` 里没有对应函数的调用名不添）；`model_comparison.rs` 里「造观测的函数」取名字里带 `observation` 的 `fn`；O2 的 `check_pool_image` 今天在 `crates/singlefs-checker/src/walk.rs`，两棵树里都按 `crates/singlefs-checker/src/` 下全部 `.rs` 找。
- 宽归因（描述）：坏实现所在文件按同名单元数指纹变了的 `fn`；装置入口 `main` 与那一列 oracle 入口用 `scripts/gate/lib_function_units.py` 的 `rust_closure` 在参照树与今天的树上各算一份闭包，数指纹变了或新出的单元，算不出写 `-`。

2.5 第十二节原注说要抄进这里的「计划那一刻的 `crates/mutations.tsv` sha256 与名单核对读数、落产物那一天的产物名」：计划那一步写进产物的 `name=snapshot`、`name=list_check` 那几行就是它们；产物跑过之后执行员不改登记，这几样由实验页整行抄，要不要补进登记交主 agent。

### 修订 3（2026-11-03，承接的执行员；写在今天任何产物之前，单测、变异与自证这一段之前也都没跑；依据是派发提示（批次 `records/2026-10-31-删兼容旧版本代码.md#19`）「关的是哪几条」与「出口」两节的原句，逐条抄在下面）

3.1 **作废 Z8 由主 agent 判，不在装置里、不造用例。** 派发提示原句：「Z8 服务结局不是跑成：不在装置里，判法是主 agent 读 `task-server/task wait` 末行 `TASK_SERVER_RESULT`；这一行在登记里写明由谁、读哪一行判，不造假用例」。落法：交第一段、第二段整趟的主 agent 读那一条 `task-server/task wait <号>` 的末行 `TASK_SERVER_RESULT`，`exit=` 不是 0、或这一行带 `shard=`（有一片没成），那一次的产物不进判定，已入库的行照留、重交同一条命令时复用（第十一节 Z8 原文的处置一字不改）；`full_table_run.py` 的 `-verdict.out` 里没有 Z8 那一行，`name=verdict` 的 `failed_checks` 不含它。登记落地清单里这一行写「主 agent 判读」，不写用例。

3.2 **第九节的变异表补 24 条，V1–V14 一条不动。** 派发提示原句：「每行的验收标准：除第 8 行外，每行有一条单测，把那一处判法改坏（弄坏开关写进文件头，或变异表加一行，锚点恰好一次）它就红，红在点名的那条用例」。补的 24 条在 `scripts/mutation/tables/e201_three_oracle_collusion_full_table_on_the_current_tree.tsv` 末尾，名字以「落地」起头（不另编号），每条改坏读法 1、读法 7 的 T3、读法 8 宽归因、Z1–Z4、Z6、S1–S7、K2、K3、F3、A7 里一处判法；经同一个 `mutation_driver.py` 跑，判法与 V1–V14 同（单测红记抓到）。只加条、不删条、不改判据，是收严。

3.3 **照登记字面补齐、上一段装置没写全的几处**（不改判据；都在产物之前，记在这里给复核）：
- Z1：两次基线另比 `call-return-values.tsv` 去掉两次就不稳的内存对象之后的部分（Z1 原文「`call-return-values.tsv` 去掉 E171 第 1 次重跑读法 4 写明拿掉的对象」，上一段只比了另十一份）；第二次基线退的不是 0 或 3（没有可比的导出）记不同。
- Z4：合并那一步加一条 `Z4`：片里逐行驱动退 5、或 T2 有 `status=ok` 而没带 `packages_checked_rebuilt` 的行（Z4 原文两样，上一段合并那一步没判）。
- Z6：数的单元加上片 0 的第二次基线（Z6 原文「某一个装置单元」；上一段漏了这一个）。装置现在是合计不为 0 就整趟记 `Z6` 不过，没写「那一个单元按本次未跑报、那一行不进 Q1、Q2」的逐单元剔除：整趟不过比逐单元剔除严，要不要改成逐单元交主 agent。
- F3：照 F3 的触发观测改成 T1 里 `ran=false` 而登记读法批 1 的类不在读法 5 那几类（`compile_failed`、`anchor_not_unique`、`ambiguous_name`、`not_found`、`self_crash`、`timeout`）里的行，加库里取不回的行；`ambiguous_name` 算进读法 5 那几类依据 F8 原文「按名字取不到或锚点不唯一……照读法 5 记类」；装置停机（`device_stop`）不在那几类里，照 F3 记。
- A7：照第七节 A7 原文实现：各片对空变异与这一片的每一行哨兵，把存进库那一份摆成行目录，比对器（`segment_two_comparator.row_line` 那一行加这一行与基线逐次调用的交回值差）在它上面与在完整导出上出的逐字相同，各写一行 `name=a7`；合并那一步加一条 `A7`；自证加一格 `stored-row-matches-the-full-export`（比对器换成假的）。为此空变异与各行的逐行驱动都带 `--keep-dumps`（完整导出要留着），入库之后哨兵以外的行删掉三个大文件；跨机对照 S7 比的空变异导出因此带上三个大文件（上一段不带 `--keep-dumps` 时两边都缺、按缺记不同）。
- K3 panic 那一格加核「截在第 3 次调用」：流二导出了、第一处 panic 在从 0 数第 3 次调用、流二最后一次调用就是它（5.2 第一张表原文「流二导出、截在第 3 次调用」；上一段只核了判过的调用与态数）。
- Z2、Z3：一片的开头或收尾那一次核对没写出来（片中途退了）也记不过。
其余读法、判据、作废与停机条款不动。

### 修订 4（2026-11-03，主 agent；写在今天任何产物之前）

4.1 **Z6 收严成整趟不过，不改回逐单元剔除。** 修订 3 第 3.3 条交来的问题，主 agent 定：第十节 Z6 原文「那一个单元按本次未跑报，那一行不进 Q1、Q2」改成「任一装置单元（含片 0 的第二次基线）的 `NOT_RUN` 合计不是 0，整趟记 Z6 不过、这一趟不出 Q1、Q2 的判定」，与装置现在的写法相同。收严在哪：原文只剔出那一行，其余照判；改后一个单元没跑，整趟都不判。被原来那套判据判成「Z6 过、剔一行」的每一趟，改后判成「Z6 不过」或相同，没有一趟变宽。原判据原样留在第十节，两条都照记：Z6 不过时，主 agent 读各片日志里 `name=shard_row row=<行> … not_run=<数>` 那几行（`full_table_run.py` 写它的那一句在 `shard_step` 里），把 `not_run` 不是 0 的行号写进实验页，即按原文会剔掉的那几行。

### 修订 5（2026-11-03，承接的执行员；写在今天任何行的产物之前：第一段两次都没有一行判过，任务 5444 停在 card-fault、任务 5503 停在 S6；依据是派发提示（批次 `records/2026-10-31-删兼容旧版本代码.md#19`）「关的是哪几条」第 1、2 条的原句，逐条抄在下面）

5.1 **读法 1「今天的树」拷的范围加 `prune-server/`，原定义那几样照留。** 依据：第一段整趟（任务 5503）第 0/2 片停在 S6，`/home/fy5090/.local/share/singlefs/work/task-server-shards/b0f6b921-5503/output-0-attempt-1/shard.out` 原文「E7RESULT name=clean_compile exit=101」「E7RESULT name=stop clause=S6」；同目录 `clean-compile.log` 原文「failed to read `…/clean/tree/prune-server/Cargo.toml` … No such file or directory (os error 2)」。派发提示的推因：`full_table_run.py` 的 `SYNCED_ENTRIES = ("Cargo.toml", "Cargo.lock", ".cargo", "crates")` 不含 `prune-server`，而仓根 `Cargo.toml` 第 17 行 `exclude = ["scripts/experiment-apparatus", "prune-server"]`、`crates/singlefs-checker-tier/Cargo.toml` 第 30 行 `prune-server = { path = "../../prune-server" }`（执行员现查：`grep -n prune-server Cargo.toml crates/*/Cargo.toml` 只有这一条 path 依赖指到仓根下的目录）；计划拷出的 `…/b0f6b921-5503/plan/tree/` 里只有 `Cargo.lock`、`Cargo.toml`、`crates`。落法：`full_table_run.py` 与 `current_tree_row_driver.py` 两份 `SYNCED_ENTRIES` 都补 `prune-server`（拷时照旧不带 `target`）；`take_tree_snapshot` 里 `git status --porcelain --` 的路径跟着补 `prune-server`，所以 `name=snapshot_status` 行与 `status_lines` 数把 `prune-server/` 下的改动也算进去。读法 1 的摘要算法没动（`.rs` 与 `Cargo.*`），摘要因此多收 `prune-server/` 下的 `.rs` 与 `Cargo.*`，仍是拷出那一份上现算。

5.2 **没动任何判据，也没收严、没补臂。** 只多拷一个 crates 编译必需的目录；读法、作废与停机条款、第六、七、九节每一条的去留与报告方式都不动。第一段两次都没有一行判过，这一条写在今天任何行的产物之前。

5.3 **单测与变异表。** 单测加 4 条（`test_full_table_run.py` 的 `test_the_copied_tree_has_the_prune_server_manifest_and_sources_without_its_target`、`test_every_repository_root_directory_a_crate_reaches_by_path_is_copied`；`test_current_tree_row_driver.py` 的 `test_the_lane_tree_has_the_prune_server_manifest_and_not_its_target`、`test_the_synced_entries_name_the_prune_server_directory`），既有一条对 `git status` 路径的断言跟着补 `prune-server`；变异表加 3 条改回不含 `prune-server` 的写法（计划拷树、线道树同步、改动清单），既有「落地 改动清单只记第一行」的锚点跟着改成 `"prune-server").splitlines())`。

### 修订 6（2026-11-04，承接的执行员；写在今天任何行的产物之前：第一段三次都没有一行判过，任务 5444 停在 card-fault、任务 5503 与任务 5606 都停在 S6；依据是派发提示（批次 `records/2026-10-31-删兼容旧版本代码.md#19`）「关的是哪几条」第 1、2 条的原句「两份 `SYNCED_ENTRIES`（`full_table_run.py`、`current_tree_row_driver.py`）照表补，粒度取最小能罩住闭包的那几项（单个文件或目录），拷时不带 `target`」与「读法 1 拷的范围照第 1 条的表补，依据写任务 5606 的那两行原文；判据不动」）

6.1 **读法 1「今天的树」拷的范围再补六份仓根文件，原定义与修订 5 那几样照留。** 依据：第一段整趟（任务 5606）第 0/2 片又停在 S6，`/home/fy5090/.local/share/singlefs/work/task-server-shards/b0f6b921-5606/output-0-attempt-1/shard.out` 原文「E7RESULT name=clean_compile exit=101」「E7RESULT name=stop clause=S6」；同目录 `clean-compile.log` 第 92、93 行原文「error: couldn't read `〔第二台的编译根〕/builds/task-b0f6b921-5608-1/target/e201-shard-0/clean/tree/crates/singlefs-unit-graph/../../.claude/heavy-thread-default`: No such file or directory (os error 2)」「  --> crates/singlefs-unit-graph/src/heavy_threads.rs:12:42」（第二台的编译根换成了〔第二台的编译根〕，私有路径）。修订 5 只按 `Cargo.toml` 的 path 依赖补了 `prune-server/`，编译期 `include_str!` 读的仓根文件与装置运行期从编它的那棵树起的脚本没算进去。执行员在 2026-11-04 照装置在拷出的树里编与跑会读到的仓根内容现查一遍闭包（扫 `crates/` 非测试代码里 `../` 起头的字面路径、`crates/singlefs-checker-tier/build.rs` 读的、装置二进制运行期真走到的，连同被 source、被起的）：

| 仓根条目 | 谁读、什么时候 | 这一次 |
|---|---|---|
| `Cargo.toml`、`Cargo.lock`、`.cargo/`、`crates/` | cargo；`crates/singlefs-checker-tier/build.rs` 读仓根 `Cargo.toml` 与 `Cargo.lock`（编译期） | 原定义，照留 |
| `prune-server/` | `crates/singlefs-checker-tier/Cargo.toml` 的 path 依赖（编译期） | 修订 5，照留 |
| `.claude/heavy-thread-default` | `crates/singlefs-unit-graph/src/heavy_threads.rs` 与 `crates/singlefs-harness/src/heavy_threads.rs` 的 `include_str!`（编译期；两个包都在装置的依赖图里）；`e184`、`e185` 两个 bin 也读，装置不编它们 | 补 |
| `scripts/crash/crash-point-replay-shard-configuration-check.sh` | `crates/singlefs-checker-tier/src/gpu_card_configuration.rs` 的 `emitted_by_the_configuration_check` 从 `env!("CARGO_MANIFEST_DIR")` 拼出它起 `--emit-addresses --configuration <多机配置>`（运行期；配置开着 GPU 时给定状态引擎起跑认这一台是 HOSTS 里哪一项走到它） | 补 |
| `.claude/scripts/preflight.sh`、`scripts/hosts/peer-host-lib.sh` | 上一行那份判法第 158、162 行 source（运行期） | 补 |
| `.claude/singlefs-ai-sop/scripts/preflight.sh`、`.claude/singlefs-ai-sop/scripts/preflight.py` | `.claude/scripts/preflight.sh` 第 11 行拼出、第 20 行 source 规范副本那一份（树里没有时第 12–17 行退到 git common-dir 所在主仓，拷出的树不是 git 工作树就定义一个判红退 1 的 preflight）；规范副本第 18 行拼出、第 44 行起同目录的 `preflight.py`（运行期；那份判法声明的是 `admission: always` 与 `run-condition: command git bash ip`，`preflight.py` 只 `shutil.which`，不载入 `proc.py`） | 补（规范副本被 `.gitignore` 挡着，照文件系统拷，不经 git） |
| `multi-host.env` | 不拷：装置照这个进程的当前目录找（`gpu_card_configuration.rs` 的 `multi_host_addresses_of_the_caller`，与 `crash_amplification.rs` 的 `multi_host_configuration_of_this_process`），逐行驱动与各片起装置的当前目录是主工作区（git 工作树）；交给判法时带 `--configuration`，判法第 496–512 行找主仓配置那一支走不到。prune-server 客户端缺省读 `prune-server/../multi-host.env`（拷出的树里没有），只在 `SINGLEFS_MULTI_HOST_CONFIG` 没设、本机又没有服务的 socket 时才读：派到第二台的尝试由 `scripts/hosts/multi-host-run.sh` 带过去这个变量，本机有 socket | 不拷，照记 |

落法：两份 `SYNCED_ENTRIES` 都补这六份单个文件（不补它们所在的目录：目录里别的文件不在闭包里），拷与同步单个文件之前先建它在树里的上级目录（rsync 不建缺的上级目录，退 3）；`take_tree_snapshot` 的 `git status --porcelain --` 路径改成 `SYNCED_ENTRIES` 本身，`name=snapshot_status` 行与 `status_lines` 数把这六份的改动也算进去（规范副本那两份被 git 忽略，不出行）。读法 1 的摘要算法没动（`.rs` 与 `Cargo.*`），这六份不进快照摘要，也就不进读法 10 的键；要不要收进键交主 agent。

6.2 **没动任何判据，也没收严、没补臂。** 只多拷几份 crates 编与跑必需的仓根文件；读法、作废与停机条款、第六、七、九节每一条的去留与报告方式都不动。第一段三次都没有一行判过，这一条写在今天任何行的产物之前。

6.3 **单测与变异表。** 单测加 7 条：`test_full_table_run.py` 的 `RepositoryRootReferenceTests` 六条（读真仓扫 `crates/` 非测试代码里 `../` 起头的字面路径、凡解到的仓根条目不在两份 `SYNCED_ENTRIES` 罩住的范围里就红；扫到的 shell 脚本传递地 source、起的仓内文件同样核；`SYNCED_ENTRIES` 每项在仓里、两份相同、恰 11 项；扫法与跟 source 的写法各在假树上钉一条；单个文件条目拷进上级目录、同目录别的文件不拷），`test_current_tree_row_driver.py` 一条（线道树同步单个文件条目）；既有一条对 `git status` 路径的断言改成 `*run.SYNCED_ENTRIES`。变异表加 6 条（两份各去掉一样编译期或运行期读的文件、各去掉单个文件条目先建上级目录那一句），修订 5 那 3 条与「落地 改动清单只记第一行」的锚点跟着新写法改，改回的仍是原来那一样（漏 `prune-server`、只记第一行）。

### 修订 7（2026-11-04，主 agent；写在今天任何行的产物之前：第一段还是一行都没判过；依据是用户批准的「拷仓副本的范围收成一份共用清单、核对改成真编」那一批，调度记录 `records/2026-10-31-删兼容旧版本代码.md` 第 20 批）

7.1 **读法 1 拷的范围改成共用库现算。** 两份 `SYNCED_ENTRIES`（`full_table_run.py`、`current_tree_row_driver.py`）不再写死，取 `scripts/hosts/lib_repository_copy.py` 的 `copy_closure(REPOSITORY_ROOT).entries`：共用清单 `Cargo.toml`、`Cargo.lock`、`.cargo`、`crates` 加现算的闭包（path 依赖、编译期读的仓根文件、运行期指到仓根的路径连同被 source、被起的传递依赖）；`multi-host.env` 不拷，副本里起的命令照共用库的 `environment_for_commands_in_the_copy` 显式带主工作区那一份多机配置。修订 6 那 11 项仍在闭包里。

7.2 **没动任何判据，也没收严、没补臂。** 读法、作废与停机条款、第六、七、九节每一条的去留与报告方式都不变。

7.3 **单测与变异表。** 单测六份合计 107 条，任务 6325 `Ran 107 tests`、`OK`；变异表 45 条，产物 `research/results/e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-04-r2-s1-mutations.out` 末行 `E7RESULT name=verdict mutations=45 caught=45 invalid=0 survived=0 timed_out=0 restored_and_baseline_green=true`；自证任务 6326 退 0。

## 十三、读过的文件与跑过的命令

**读过的文件**（行号区间；grep 命中行也列）：
- `.claude/agent-common.md` 第 1–123 行；规则整份：`.claude/singlefs-ai-sop/rules/test-discipline.md` 第 1–138 行、`.claude/rules/three-way-inference.md` 第 1–168 行、`.claude/singlefs-ai-sop/rules/evidence-discipline.md` 第 1–182 行（读完之后它在盘上改过，又读了改后的第 26–55 行与 `grep -n '^#'`）、`.claude/rules/mutation-sampling.md` 第 1–93 行、`.claude/rules/implementation-first.md` 第 1–19 行、`.claude/rules/test-script-discipline.md` 第 1–106 行。
- `research/prompts/e171-current-tree-full-table-questions.md` 第 1–12 行；`research/prompts/e171-r1-prereg.md` 第 1–90、289–572、1433–1667 行与 `grep -n '^#'`；`research/prompts/e171-r2-prereg.md` 第 1–360 行、第 360 行起的修订 3 第 3.1 条、第 519–588 行与 `grep -n '^#'`；`git show 31128273b^:research/prompts/e171-preregistration.md`（落到草稿目录）第 29–48、221–232 行与 `grep -n '^#'`；`research/prompts/e200-preregistration.md` 第 12、16、27 行（grep 命中）与第 30–58 行。
- `.claude/kb/decisions/13-验证路线.md` 第 16、129、172–202 行；`.claude/kb/experiments.md` 第 195 行（整行，第四节第 1 条）与第 214 行（前 60 个字）。
- 研究一侧 `scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/`：`row_driver.py` 第 1–60、255–290、316–320 行与 grep 命中行；`segment_two_comparator.py` 第 1–80、310–330、430–445 行与 grep 命中行；`segment_one_comparator.py` 第 1–60、227–280 行；`collusion_matrix.py`、`list_check.py`、`bad_implementation_table.py` 的 grep 命中行；`frozen_o3.py` 第 1–50 行；`o3_bad_image_judge.py` 第 1–30 行；`mutation_driver.py` 第 1–12 行；`rows_orchestrator_segment_two_revision_11.py` 第 1–50 行；`run_segment_two_revision_11_multi_host.sh` 第 1–45 行；`contrast_revision_6_against_revision_8.py` 第 1–40 行；`contrast_segment_two_revision_10.py` 第 1–25 行；两张名单的前 3 行（前 400 字）。
- 入库装置 `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs` 第 1–30、75–90、7010–7050 行与 grep 命中行；`crates/singlefs-checker-tier/src/crash_amplification.rs` 第 18596–18645、1284–1298 行与 grep 命中行；`crates/singlefs-harness/src/gpu_memory_ledger.rs` 第 1–15 行；`crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs` 的 grep 命中行。
- `scripts/kb/claim-experiment.sh` 第 1–60 行；`scripts/kb/quote-kb.py` 第 1–40 行；`scripts/experiment-apparatus/device-code-key.py` 第 1–70 行；`scripts/prune-server/prune_server_client.py` 第 1–80 行与 grep 命中行；`scripts/gate/lib_function_units.py` 第 1–25 行与 `grep -n '^def '`；`task-server/README.md` 的标题与 grep 命中行（第 13、15、21、23、25、31、56、97、99、101 行）；`task-server/task_server_lib.py` 第 234–247、279–330 行与 grep 命中行；`.claude/gate.d/verdict-library-scripts.tsv` 第 1–25 行；`.claude/gate.d/doc-experiments.sh` 的 `gate-cell` 行与 `JUDGE_ENTRIES` 那一段；`.claude/gate.d/doc-text.sh` 的 `gate-cell` 行与第 1250–1285 行；`.claude/heavy-thread-default`；`scripts/hosts/capped.sh` 的 grep 命中行；`.claude/hooks/run-scope-guard.sh` 第 1–40 行的 grep 命中行；`.claude/term-rename-exempt` 第 15 行；`multi-host.env` 只取键名（`grep -o '^[A-Z_]*='`）。
- 产物：`research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s2-device-baseline-streams-1-2-r6-r3-timing.out` 整份（25 行，第四节第 5 条）；P1、P3 只取了行种名（`awk` 第 2 列）与 `name=row`、`q1_row`、`untouched_row_red`、`q2_set_row` 各一行的字段名；P2 只取了行首词、`row` 行的字段名、第 1 行与第 284 行整行；P0 只取了前三列去掉值之后的计数。实验页 `.claude/kb/experiments/171-三oracle合谋测试.md` 没打开。

**跑过的命令**（只读与文本工具；输出摘要照原样）：

```text
bash scripts/hosts/work-directory-ownership.sh claim <草稿目录> --purpose '…'          → ✓ 认领了已有的目录
bash scripts/kb/claim-experiment.sh --next                                             → E991
find research scripts crates/singlefs-checker-tier/src/bin -iname '*e9[0-9][0-9]*' -not -path '*/target/*'
  → 十份 research/prompts/d137-ledger-defs-r*-opus-model/worlds/*/after/research/prompts/e990-preregistration.md
bash scripts/kb/claim-experiment.sh E201 三oracle合谋今树整表                            → ✓ 占住 E201（对过 2 棵工作树里用过的 200 个号）
sha256sum <两个目录的两张名单>  → 两个目录同为 18cdb0c6…（step-8）、57c981f0…（step-4-only）
cat <两张名单> | cut -f1 | sort -n | …   → rows 278；seq 1 281 去掉之后缺 33 107 236；uniq -d 0 行；awk NF 全是 6
awk -F'\t' 'FNR==NR{ if ($0 !~ /^#/ && NF==6) c[$1]++; next } { n=c[$5]+0; h[n]++; if (n!=1) print "row", $1, "hits", n } END{ for (k in h) print "hits", k, "rows", h[k] }' crates/mutations.tsv <两张名单>
  → hits 1 rows 278
sha256sum crates/mutations.tsv → 07fcc8caa410365c9051f07d27d1a6d3b0cd44091bb3ef8111c6808e05c8111e
sha256sum、wc -l、git ls-tree HEAD、git diff --quiet HEAD -- <P1、P2、P3> → 读法 2 那几个值，三份工作区与 HEAD 相同
grep -c '^E7RESULT name=row ' <P1> → 281；grep -c '^row row_number=' <P2> → 281
git show 04ceaa904^:research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-frozen-tree.out | sha256sum → b720e5d2…；| wc -l → 125
git show f8a9ddaab:<入库装置> | sha256sum → ab35105f861d509457296741d2c60515a3a94b9fefe77944f03d1b358afd45df
git show f8a9ddaab:<入库装置> | sed '3478,3484d' | sha256sum → 67e7a5498425ec3f27379ba8f0e5cd666faaeaac0817006d6938956d6419a156
git show f8a9ddaab:<入库装置> | sed -n '3478,3484p' | grep -c SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults → 1
git ls-tree -r 31e47115471ba5502a4531d25dffcc921be36899 --name-only -- crates Cargo.toml Cargo.lock | grep -cE '\.rs$|/Cargo\.|^Cargo\.' → 270
git diff --stat 31e47115471ba5502a4531d25dffcc921be36899 -- crates Cargo.toml Cargo.lock .cargo | tail -1 → 792 files changed, 587501 insertions(+), 64615 deletions(-)
git log --oneline 31e47115…..HEAD -- <入库装置> | wc -l → 5
git status --porcelain -- crates Cargo.toml Cargo.lock .cargo | wc -l → 334
python3 scripts/kb/quote-kb.py <草稿目录>/quotes.md '.claude/kb/decisions/13-验证路线.md@#### 已定项 6：三个互相独立的 oracle' '<草稿目录>/e171-preregistration-from-git.md@### 读法写死' '<草稿目录>/e171-preregistration-from-git.md@### 5.2 坏实现表：复用 `crates/mutations.tsv`，挑法写死' 'research/prompts/e171-r1-prereg.md@### 读法写死' 'research/prompts/e171-r2-prereg.md@### 读法写死' 'research/prompts/e171-r2-prereg.md@### 修订 9（…）'
  → ✓ 6 段整抄进 …/quotes.md，回读逐字节一致
sed -i 's#`<草稿目录>/e171-preregistration-from-git.md:\([0-9-]*\)`#`git show 31128273b^:research/prompts/e171-preregistration.md` 第 \1 行#' <草稿目录>/quotes.md   （只改两处「出处」行）
第二节追加之后：tail -n +<追加前行数+1> 登记 | cmp - <草稿目录>/quotes.md → appended-quotes-identical
```

## 十四、并行与挂钟

独立单位：整表每一行（278 行加补位，各行一棵线道树里改坏一处、编、跑装置、跑 O3，行与行之间只经补位相关）互相独立；片 0 的第二次基线、停机 S2 那一比、O3 基线与十八个阳性对照单元互相独立；第二段的十六行互相独立。
线程：每台取登记值 `.claude/heavy-thread-default` 的 31（服务经 `SINGLEFS_THREAD_CAP` 交给每一片，命令里不写）；一片在进程内开 ⌊`SINGLEFS_THREAD_CAP` ÷ 6⌋ = 5 条线道、每线道 6 线程（cargo、装置、O3 执行器的线程数照线道线程数设进各自的环境，`frozen_o3.thread_environment`），剩 1 线程给驱动自己；同一条任务里同时起的单元只分这一个数。
串行：计划在各片之前（各片要它拷的树与键）；每片先跑自己的基线（逐行限时要它的 T）；一条线道里的行一行接一行（线道树与编译目录跨行增量编，编前编后核对靠这个次序，E171 第 1 次重跑登记修订 5）；合并在两片都成之后（比对器要整表与基线）；两片同一时刻一台只跑一片是服务定的。别的都并行。
预计挂钟：第一段合计约 1.5–3 小时（推的，下面四项相加）。整表的行：278 行 × 105 秒 ÷ 10 ≈ 49 分钟（105 秒 = r1 第二段量的每行均值 97.2 秒加 O3 一步推的约 8 秒，均值出自 P2 第 284 行各行耗时之和除以行数；10 是两台各 5 条线道，只落一台时除数是 5、约 1.6 小时）。每线道头一次冷编：约 5–15 分钟（没量，推的）。片 0 的前置：约 15–30 分钟（基线进程挂钟 75.3 秒是量的，r1 计时产物；十八个对照单元各一趟装置，推的）。计划与合并：约 15–25 分钟（计划拷 45 MB 的树，`du -sh` 量的大小；合并起比对器与归因，推的）。给定状态引擎走卡上判 O2 之后每态耗时没量过，可能比 r1 的 CPU 读数快也可能慢。第二段：16 趟 × 2.5 小时 ÷ 10 ≈ 4 小时，加冷编约 4–6 小时（推的：2.5 小时取 E171 第 1 次重跑修订 8 第 8.4 节「带流三的两项各约 2.3–2.7 小时」的中间）。
预计在用核数：第一段与第二段都是每台约 30 个核（5 线道 × 6 线程，推的），两台都接到片时合计约 60；计划与合并那两步各约 1–4 个核（推的）。
分片单元：一行一个单元（逐行驱动的线道从这一片的行队列里先到先领）；两片是两台各一份的装法，行按推的用时由长到短交替分。
续跑入口：同一条命令 `task-server/task submit -- python3 scripts/experiment-apparatus/e201_three_oracle_collusion_full_table_on_the_current_tree/full_table_run.py run` 再交一次：计划照读法 10 的键查 prune-server，判完入库的行不派、只补没判的行与哨兵；基线、空变异、对照每次现跑（约 15–30 分钟）。前提是主工作区那几份没变（快照摘要相同，键才相同）。
