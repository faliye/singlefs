# m3-batch29-code-r1 正推腿报告（Sonnet，格 K1 与 K3）

日期 2026-10-06。立场：正推。被判对象是冻结副本 `/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/` 里的 6 份文件；引代码写冻结副本的绝对路径加行号，引 kb 写仓里 `.claude/kb/invariants.md` 的行号。模型与脚本在 `research/prompts/m3-batch29-code-r1-sonnet-model/`（`rerun.sh`、`mutrun.py`、`rowcheck.py`、`sites.py`、`extra-mutations.tsv`）。

## 0 前提复核

- 冻结副本 6 份与快照 sha256 逐份相同。命令 `cd <冻结副本> && sha256sum -c research/prompts/m3-batch29-code-r1-snapshot/crates-sha256.txt`，原样输出：
  - `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl: OK`
  - `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl: OK`
  - `crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs: OK`
  - `crates/singlefs-checker/src/walk.rs: OK`
  - `crates/singlefs-harness/tests/checker_known_bad_images.rs: OK`
  - `crates/mutations.tsv: OK`
- 只跑了 harness 档（`-p singlefs-harness --test checker_known_bad_images` 与 9 个合法流二进制），checker 档包（含 GPU）一概没跑；编译经 `run-with-memory-cap.sh 6G`、跑经 4G，线程 15，一份树副本一个编译目录。
- 基线：冻结副本上 `checker_known_bad_images` 整份二进制 `test result: ok. 154 passed; 0 failed`；其中本批 5 条新坏镜像与第 27 批那条（改了注释的）全 ok（`rerun.sh` 的 `new` 单元，输出落 summary 的 `NEW` 行）。

## K1-L1　三棵树「不往下走」两支每份副本判 I-2.1

条款与判决依据：
- kb 条款：`.claude/kb/invariants.md:130`：「| I-2.1 | 校验和与内容匹配 | 任一被引用的块，其校验和与内容匹配 | …」（`.claude/kb/invariants.md:130`）。
- 判决改法（m3-batch27-code-r1 第二节 H-J3a 行）：「改法照攻方 F-α（零轮）推到三棵树：类型段不认得那一支不往下走，但每条位置条目按头里那一类的长度（没登记的 16 KiB、32 KiB 都试）判 I-2.1，两份都对不上记走读失败；GPU 同改」（`research/prompts/m3-batch27-code-r1-main-verification.md:26`）。
- 既有读路径的口径：「/// 按位置条目读一个被引用的单元：每条位置条目都读、都判 I-2.1（校验和与内容匹配），返回第一份对得上的。」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker/src/image.rs:1490`）——逐份判、读不出算对不上，新函数与它同口径。

### L1-a　三棵树两支各调到共用函数没有：一致

CPU 四个调用点覆盖 3 棵树 × 2 支 = 6 格（墓碑与谱系共用 `walk_the_entries_of_a_container_tree_node`，所以两个调用点管两棵树）：
- 墓碑 / 谱系，类型段不认得：「self.judge_every_copy_of_a_child_the_walk_does_not_go_into(&child, &what);」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker/src/walk.rs:2272`）。
- 墓碑 / 谱系，孩子是另一类：「self.judge_every_copy_of_a_child_the_walk_does_not_go_into(child, &what);」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker/src/walk.rs:2299`）。
- inode，孩子是另一类：「self.judge_every_copy_of_a_child_the_walk_does_not_go_into(」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker/src/walk.rs:2575`）。
- inode，类型段不认得：「self.judge_every_copy_of_a_child_the_walk_does_not_go_into(」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker/src/walk.rs:2606`）。

GPU 五个调用点（容器树那一侧 GPU 把「另一类」按类型段 0 与类型段 = 记录类型拆成两处）：
- inode 另一类：「w_judge_every_copy_of_a_child_the_walk_does_not_go_into(child);」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2640`）。
- inode 类型段不认得：「w_judge_every_copy_of_a_child_the_walk_does_not_go_into(child);」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2653`）。
- 容器树类型段 0 的孩子是另一类：「w_judge_every_copy_of_a_child_the_walk_does_not_go_into(child);」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3096`）。
- 容器树类型段不在 {0, 记录类型}：「w_judge_every_copy_of_a_child_the_walk_does_not_go_into(child);」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3136`）。
- 容器树类型段 = 记录类型的孩子是另一类：「w_judge_every_copy_of_a_child_the_walk_does_not_go_into(child);」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3144`）。

层级 1 / 2：共用函数不看层级；层级只进调用前的登记引用跨度（`node.level >= 2` 取 1 槽、否则 2 槽，调用前已登记），与本批无关。

### L1-b　比长度的规则与 I-2.1 字面、判决：一致

- CPU：「Some(unit_bytes) => vec![unit_bytes],」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker/src/walk.rs:4249`）、「None => vec![node_bytes(), data_unit_bytes()],」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker/src/walk.rs:4250`）：头里的类标签登记过（码 1、2、3）只按那一类的长度比；没登记的码或头（前 16 KiB）读不出，16 KiB 与 32 KiB 任一对上即对得上。与判决那一句逐项相同。
- 两份都对不上记走读失败：「if !some_copy_matches {」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker/src/walk.rs:2331`），上一行 2329 是 `some_copy_matches |= matches;`。
- 与 I-2.1 字面：条款只说「被引用的块」校验和与内容匹配，没说没登记的类用哪个长度；登记过的类的长度就是 `read_referenced_unit` 对那一类读的长度，没登记的两种都试是判决定的放宽（只会少红、不会多红）。判一致，没有与字面冲突的一格。
- GPU：「if (class_in_the_header >= 1u && class_in_the_header <= 3u) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3019`）、「return slice_crc_matches(copy, NODE_BYTES, checksum) || slice_crc_matches(copy, DATA_UNIT_BYTES, checksum);」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3022`）、「if (!first_matches && !second_matches) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3040`）：登记码 1..3 只按那一类，其余两种长度都比，两份都对不上记 `w_failures`，每份对不上记 `w_mismatches`（I-4.8、I-7.4 用）。头读不出：CPU 落到两种长度都读（也读不出，false），GPU 第 3014 行起 `copy.version == NONE` 直接 false，结果同。
- 差别只在既有的一类：GPU `read_at` 要求落点起点对上、CPU 读镜像任意偏移（`w_child_unit_class_other_than_the_entry_type_names` 一带第 26–28 批就是这样，不是本批引入）。

**推翻 L1-a / L1-b 的现象**：某个状态上 CPU 与 GPU 对「不往下走的孩子」判出不同的 I-2.1 / I-7.2 / I-4.8 / I-7.4 位；或合法状态（可达的）进到这四个调用点之一（这四支都以类型段不认得或孩子自证成另一类为前提，合法镜像上不应进）。本腿没找到，也没有跑 GPU。

## K1-L3 与测试　5 条坏镜像、对拍流、13 行变异

### T-a　5 条坏镜像的期望写全没有：写全，缺的几形列在后面

harness 测试 `/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-harness/tests/checker_known_bad_images.rs` 的共用断言：「fn assert_the_child_the_walk_does_not_go_into_reddens_exactly(」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-harness/tests/checker_known_bad_images.rs:11034`）要求红的不变量集合恰好等于期望、I-2.1 的违例说明点名「走读不往下读的孩子」、期望里有 I-7.2 的它的说明点名两份都对不上。5 条：

| 测试（第几行） | 形 | 期望红集 | 本腿复跑 |
|---|---|---|---|
| 11082 `an_inode_tree_entry_with_an_unregistered_type_over_a_child_whose_two_…` | inode 类型段 7，两条位置条目校验和都坏 | I-9.2、I-2.1、I-4.8、I-7.2、I-7.4 | ok |
| 11107 `a_tombstone_tree_entry_with_an_unregistered_type_over_a_child_whose_two_…` | 墓碑类型段 7，两条都坏 | 墓碑身份条款、I-2.1、I-4.8、I-7.2、I-7.4 | ok |
| 11138 `a_lineage_tree_entry_with_an_unregistered_type_over_a_child_whose_two_…` | 谱系类型段 7，两条都坏 | 谱系身份条款、I-2.1、I-4.8、I-7.2、I-7.4 | ok |
| 11172 `an_inode_tree_entry_of_type_zero_over_a_leaf_container_whose_first_…` | inode 类型段 0 指着叶容器，第一条坏 | I-9.2、I-2.1、I-4.8、I-7.4（I-7.2 不红：第二份对得上） | ok |
| 11201 `a_tombstone_tree_entry_of_type_zero_over_a_container_whose_first_…` | 墓碑类型段 0 指着容器，第一条坏 | 墓碑身份条款、I-2.1、I-4.8、I-7.4 | ok |

第 27 批那条（`an_inode_tree_entry_with_an_unregistered_type_over_a_child_that_does_not_read_back_…`，第 10960 行）期望未动、只加注：「/// 「不连带 I-2.1」只对这一张镜像成立（判决 `research/prompts/m3-batch27-code-r1-main-verification.md` 第二节 H-J3a 那一行）：第 29 批起这一支」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-harness/tests/checker_known_bad_images.rs:10955`），复跑 ok，即判决要求的「两种镜像分开写」做到了。

没有测试钉的形（用变异实测，不是读码推的；表在 `extra-mutations.tsv`，施加在 walk.rs、跑整份 `checker_known_bad_images`，154 条）：

| 变异 | 结局 | 含义 |
|---|---|---|
| X1 只判第一条位置条目 | RED（6 条红，含 4 条第 26 批的 `…stale_copy_of_the_child…on_either_device`） | 「第二份坏」那一形有老测试钉着 |
| X2 `some_copy_matches` 只看最后一份 | RED（4 条） | 「第一份好、第二份坏」有老测试钉着 |
| X6 走读失败条件取反 | RED（19 条） | 正常 |
| X3 登记过的类也按两种长度都比 | **GREEN（0 条红）** | 「登记过的类只按那一类的长度比」这一半没有测试钉：没有哪张镜像的头里是码 1/2/3、而位置条目校验和只在另一种长度上对得上 |
| X4 没登记的类只按 32 KiB 比 | **GREEN** | 「没登记的两种长度都比」里 16 KiB 那一半没有测试钉（第 27 批那条是 32 KiB 补齐的，只钉得住 32 KiB 那半；变异表第 2978 行把它改成只比 16 KiB 才红） |
| X5 读不出的副本算对得上（`is_some_and`→`is_none_or`） | **GREEN** | 「一份头读不出 / 整单元读不出」在这四个调用点上没有测试 |

三个 GREEN 是测试尖度缺口，不是实现与条款不符：X3、X4、X5 改的行为在条款与判决字面下都只出现在非法镜像的怪形上。严重性由主 agent 判；没有一个改变 5 条新测试已钉的结论。另：谱系树「孩子是另一类」那一形没有单独测试（与墓碑共用 `judge_a_container_tree_entry_whose_child_is_of_another_class`，没有谱系独有的代码，不另列缺口）；层级 2（`node.level >= 2` 取 1 槽）所有新测试都只造层级 1。

### T-b　对拍流（GPU）的期望

`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 新加两条 inode 流：第 7831 行 `…unregistered_type_names_a_child_whose_two_location_checksums_are_wrong`（期望 I-9.2、I-2.1、I-4.8、I-7.2、I-7.4）、第 7867 行 `…type_zero_names_a_leaf_container_whose_first_location_checksum_is_wrong`（期望 I-9.2、I-2.1、I-4.8、I-7.4），两条的期望集与 harness 对应两条逐项相同。**没有墓碑 / 谱系的 GPU 流**：GPU 容器树那三处调用（`crash_judge_checker.wgsl` 第 3096、3136、3144 行）没有任何对拍流或变异行点名（变异行 2985–2987 的锚点只在 inode 两处与走读失败那一处，见 T-c）。GPU 没跑，复核不了对拍本身。

### T-c　13 行变异（`crates/mutations.tsv` 第 2977–2989 行）点名的测试与参数筛不筛得到

`rowcheck.py` 对 13 行逐条核（锚点命中数、测试函数在测试文件里有、参数筛得到）：13 行全是 `anchor_hits=1`、测试函数各有一个、`selected` 或 `all`（第 2978、2979 行命令不带筛选，整份二进制跑）；`crates-mutation-rows.py static-check` 对第 2977–2989 行 0 个问题行，只报第 2935 行（锚点 0 次命中，说明里「不读）」在第 2612 行已改成「不往下走）」，正文说 30 批修，本腿不判）与第 1352 行（litmus 文件不在冻结副本里，副本不带 `litmus/`，不是本批的）。

7 行 CPU 变异逐条施加、跑、还原（`mutrun.py`，每行先在原树上跑一遍基线全绿）：第 2977–2983 行结局全是 `RED named_red`（点名的测试红），红的条数 5、1、3、1、1、1、1。要 checker 档的 6 行（第 2984–2989 行：I-3.12 `<`→`<=`、GPU 四行、K1 J2 一行）记 SKIP，**复核不了**：要 `-p singlefs-checker-tier --features verdict-store,gpu`，属重型档，由主 agent 提交时跑；本腿只核了它们的锚点各命中一次、点名的测试名都在文件里（`rowcheck.py` 的 `test_fn_in_tier=1`）。第 2989 行（「第 1 片读不出」那条关门流）：锚点是 `w_walk_instance_table_chain` 里 `s.version == NONE` 那一支，点名的是 `…second_page_of_the_newest_instance_table_does_not_read_back`，即判决第三节第 3 条要的那一条；该测试在没有 GPU 卡时只做 CPU 前置、不比 GPU（第 8138 行起的 `assert_…` 在 `one_usable_card_or_report_not_run` 之前只核 CPU），所以这一行只有带卡的机器上才能红。

**推翻 T-a / T-c 的现象**：把 X3 / X4 / X5 之一写成测试后与现有判决冲突；或 `mutrun.py` 在原树上基线红（本腿基线全绿）。

## K1-L2　扫描内核两处走满记判不了

### L2-a　命名常量、站点号、快测试：一致

- 命名常量：扫描内核「const INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED: u32 = 256u;」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:13`），走读内核「const INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED: u32 = 256u;」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1580`）。两个循环都改用它：「for (var step = 0u; step < INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED; step = step + 1u) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:493`，第 1112 行同）。
- 两句：「undecidable = UNDECIDABLE_INSTANCE_TABLE_PAGES_AFTER_THE_FIRST_LONGER_THAN_THE_PAGES_SCANNED;」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:523`）、「undecidable = UNDECIDABLE_INSTANCE_TABLE_ROWS_LONGER_THAN_THE_PAGES_SCANNED;」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1157`）。
- 站点号不撞：`sites.py` 把 `crates/singlefs-checker-tier/src/*.wgsl` 里所有 `(N << 8u)` 字面站点号与 `d_push_container_tree_entries_in_reverse` 四处实参（50、51、55、56）并起来数，原样输出：`SITES	distinct=60	max=61	duplicates=[]`、`SITES	site 37	occurrences=1	['crash_judge_checker_scan.wgsl']`、`SITES	site 38	occurrences=1	['crash_judge_checker_scan.wgsl']`、`SITES	site 61	occurrences=1	['crash_judge_checker.wgsl']`。Rust 一侧只有对拍文件里的 `3 | (37 << 8)`、`3 | (38 << 8)`、`3 | (61 << 8)` 三个常量（`grep -rn '<< 8)'`）。
- 快测试钉的是源码文本：`consts` 单元数出两个内核里 `const INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED: u32 = 256u;` 各命中 1 处（输出 `CONSTS	crash_judge_checker.wgsl	… 命中 1 处`、`CONSTS	crash_judge_checker_scan.wgsl	… 命中 1 处`）。快测试本身在 checker 档的测试二进制里，没跑；它只核常量声明那一句的文本，核不到循环是不是真用了它。

### L2-b　链长 255 / 256 / 257、根在候选集里 / 外、中途断开时扫描内核判什么（读码）

CPU 读链不设上限（`walk.rs` 第 2953、4457、5922 行三个 `loop {`）。扫描内核按下标数：
- `r_instance_rows`：第 k 片（k 从 0）在 `step == k` 读；末片链记录说没有下一片时 `chain == 0u` 返回 true（`c_r_row_count = rows;` 在 `/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1148`），所以 255、256 片的链都读完、与 CPU 同判；257 片的链在 `step == 255` 读到第 256 片时链记录还说有下一片，循环结束落到第 1157 行记判不了、返回 false。
- `r_instance_pages_after_first`：同样的边界，256 片读完、257 片记判不了并置 `r_complete = false`。
- 中途断开（读不出、身份不对、链记录不合法）：两处都在循环里提前 return（`r_instance_rows` 返回 false 不记判不了，`r_instance_pages_after_first` 置 `r_complete = false`），与 CPU `instance_table_rows_of_root_without_judging` 的 `None` / `is_complete = false` 同。
- 根在候选集里或外：这两处读的是「一条根自己指着的实例表」，不看根在不在候选集；I-3.12 对根环里每条根循环（「if (!r_instance_rows(root.base)) {」，`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1926`）。

### L2-c　走满记判不了那两句能不能由一条流单独走到：与主 agent 的读法冲突

两句并排：
- 主 agent 观测（转述，正文第二节第 2 条）：走读内核唯一调用点只走最新根、候选根与记录施加的版本，候选集外的根（被崩溃恢复抛弃的那条）只有扫描内核 I-3.12 走到。
- 代码：走读内核那唯一的调用点「w_walk_one(walk_kind, walk_base, walk_is_newest, fsid_low);」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:4155`）在 `walk_and_hand_off` 的阶段循环里，阶段不止那三类：被抛弃的根「while (w_instance_row_count > 0u && cursor < root_count && !found) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3918`）；收割阶段对不是候选根的每条根都走，写法是「if (is_candidate_root(i)) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:4074`）后接 `continue`、其余落到 `found = true`；F 之下的根再走一遍（第 4086 行起）。每一次 `w_walk_one` 都先走这条根自己的实例表链（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3247`），走满先记 61：「if (page == INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1686`）；记了判不了扫描内核不跑：「if (reached == WALK_REACHED_THE_HAND_OFF && undecidable == 0u) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:4553`）。

判定：实现员那句「走读内核先记 61、扫描内核那两句没有一条流走得到」与代码一致；主 agent 的读法与第 3918 行、第 4074 行（收割阶段）两处冲突。再往下推「扫描内核逐页的接受条件不比走读内核宽」（读码推的，没跑）：
- 走读内核每页：`w_judge_unit_header`（头校验和、版本、码 3、载荷 CRC 过时的预留位为 0）、`w_judge_packed_container` 的「let legal = record_type == expected_type && registered_width == width && count * width <= declared && declared <= DATA_UNIT_BYTES - 136u;」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1550`）、身份「let identity_holds = is_zero64(p.birth_tree) && p.record_type == 4u && eq64(p.container, page_index) && is_zero64(p.container_birth);」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1628`）、链记录。
- 扫描内核 `r_instance_rows` 每页：魔数、码 3、版本、`(s.c2 & 11u) != 11u`（头校验和过、载荷 CRC 过、预留位全 0）、「|| width != 88u || count * width > declared || declared > DATA_UNIT_BYTES - 136u) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1124`）、容器号等于片号。`r_instance_pages_after_first` 走 `parse_packed_unit`：「if ((s.c2 & 11u) != 11u || rd8(base, 42u) != 3u) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:1503`），再核身份：「if (!is_zero64(packed.birth_tree) || packed.record_type != 4u || !eq64(packed.container, page) || !is_zero64(packed.container_birth)) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:500`），这一处不核 width == 88，但链记录的读法「if (width != 88u || rd8(base, o) != 1u) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:604`）对 width ≠ 88 的页交回 2、扫描内核照样停。
- 所以扫描内核读得过的页走读内核读得过（走读内核还多容忍载荷 CRC 不过的页）；同一条根的链在走读内核先走满，先记 61。

L2 小结（读码推，没有流可跑）：今天的走读内核下，站点 37、38 这两句没有一条流单独走得到；它们是纵深。**「删掉那两句」该不该有变异**：流抓不到（走读内核先判不了、扫描内核不跑）；能抓到的只有读源码文本的快测试那一类（与钉常量声明那一条同形）。变异表第 2988 行改的是扫描内核的常量值（255），由钉常量文本的快测试抓，与这两句无关。按同一读法，第 27 批判决「O-1 坐实」说的扫描内核 256 步落到 `return false`，在今天的走读内核下也没有一条流到得了（那一条在判决里是读码推的）。

**推翻 L2-c 的现象**：找到一条根，它在走读内核里没走自己的实例表链（不在 `root_count` 的根环里、或走读内核在它之前已记别的判不了而本来就不跑扫描内核都不算），而扫描内核对它走到第 256 步；或一页走读内核读不过而扫描内核读得过（上面逐条比，没找到）；或 `w_visit` 在链中途先停了走读内核的循环（同一上下文里它已走过的页，上一次走读已经计过步数）。

## K1-合法流抽样（给 K2 的线索，不是 K2）

9 个合法流的 harness 测试二进制整份跑（新池建文件、删除、回退、容器树长高，`rerun.sh` 的 `legit` 单元）：9 个二进制全绿，`LEGIT` 行共 9 条（`new_pool_file_creation_publish` 9 条、`unlink_single_file_publishes` 9 条、`pool_checker_judges_the_deletion_state_of_every_walked_version` 45 条、`rollback_lineage_record_and_its_checkpoint_cost_term` 4 条、`container_trees_grow_a_level_when_the_root_is_full` 6 条、`deletion_scenario_writes` 5 条、`unlink_of_a_file_spanning_two_extent_lower_leaves_in_product_batches` 1 条、`unlink_mount_continuation_and_the_emptied_version` 23 条、`test_only_switches_force_every_branch` 6 条，各 0 条失败）。只说明这几条合法流上新写法没多红 I-2.1；这是一次抽样，不替 K2 的攻。

## K3　被判的 6 份文件，每份一格

| 文件（冻结副本 `crates/` 下） | 判定 | 依据 |
|---|---|---|
| `singlefs-checker/src/walk.rs` | 一致 | L1-a / L1-b：4 个调用点覆盖 3 棵树 × 2 支，比长度规则与判决 F-α 逐项相同；5 条新坏镜像与第 27 批那条在它上全 ok（154 条整份绿）；7 行 CPU 变异全红。说明里「不读）」改成「不往下走）」（第 2612 行）让变异表第 2935 行锚点 0 次命中，已知、归第 30 批。 |
| `singlefs-checker-tier/src/crash_judge_checker.wgsl` | 一致（GPU 没跑） | 5 个调用点与 CPU 4 个一一对应（容器树那侧 GPU 拆成三处）；`w_copy_matches_…` 与 CPU 的规则逐句同；`w_mismatches` / `w_failures` 与 CPU 的 I-2.1 计数 / `walk_failures` 同口径。容器树三处（第 3096、3136、3144 行）没有对拍流或变异行点名。 |
| `singlefs-checker-tier/src/crash_judge_checker_scan.wgsl` | 一致于条款与判决；L2-c 处与主 agent 读法冲突 | 命名常量、两句、站点号 37 / 38 都对；两句在今天的走读内核下没有一条流单独走得到（读码推）。 |
| `singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` | 部分复核不了 | 两条 inode 对拍流的期望集与 harness 对应两条逐项相同；重构（`instance_table_pages_from`、`writes_of_instance_table_pages`、`insert_a_segment_of_writes_before_the_segment_of`）按 diff 读是行为不变的拆分（顺序、断言、下标挪动都与原函数一一对应，读码、没跑）；257 片那条流造得对不对（CPU 一侧在整条落盘的镜像上 I-3.12 红）与快测试本身要 checker 档，复核不了。那条流接受三种原因字（61 / 37 / 38），所以它分不出走的是哪一处。 |
| `singlefs-harness/tests/checker_known_bad_images.rs` | 一致（有尖度缺口） | 5 条新测试全 ok、期望集写全（T-a）；X3 / X4 / X5 三个变异整份二进制绿，没有测试钉。 |
| `crates/mutations.tsv` | 一致（13 行里 7 行实跑红、6 行没跑） | 13 行锚点各命中 1 次、测试名在、筛选筛得到；7 行 CPU 变异红且点名的测试红；6 行 checker 档的没跑；第 2935 行锚点已断（已知）。 |

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1-L1 三棵树两支调用 | 一致 | CPU 4 处、GPU 5 处，3 棵树 × 2 支都盖到 |
| K1-L1 比长度规则 / I-2.1 字面 / 走读失败 | 一致 | 登记类按那一类、没登记两种都比、两份都对不上记走读失败，与判决 F-α 逐项同；X3 / X4 / X5 三形没有测试钉 |
| K1-L1 GPU 同判 | 一致（读码） | `w_copy_matches_…` 与 CPU 逐句同；GPU 没跑 |
| K1-L2 命名常量、站点号 37 / 38、快测试 | 一致 | `sites.py`：60 个站点号无重复；快测试只钉常量声明文本 |
| K1-L2 链长 255 / 256 / 257、断开 | 一致（读码） | 256 片读完同判，257 片记判不了，断开同 CPU |
| K1-L2 两句能否由一条流单独走到 | 冲突（与主 agent 观测） | 走读内核在被抛弃、收割、F 之下三个阶段都走根自己的链并先记 61；扫描内核逐页接受条件不比走读内核宽；两句今天是纵深，流抓不到 |
| K1-L2 删掉两句该不该有变异 | 规则没说 | 流抓不到；只有读源码文本的快测试能抓；是否收交主 agent |
| K1-L3 5 条坏镜像期望 | 一致 | 期望集写全，整份二进制 154 条绿；缺谱系「另一类」独测、层级 2、X3 / X4 / X5 |
| K1-L3 对拍流期望 | 部分复核不了 | 两条 inode 流期望同 harness；墓碑 / 谱系没有 GPU 流；257 片那条要 checker 档 |
| K1-L3 13 行变异点名与筛选 | 一致 | 13 行锚点 1 次命中、名字在、筛得到；7 行 CPU 红；6 行没跑 |
| K3 6 份 | 见上表 | 每份一格 |

## 没做什么

- 不判 K2、不采纳、不出判决；攻方腿的报告与模型一个字没读。
- 没跑任何 checker 档（`-p singlefs-checker-tier`）、没跑 GPU：第 2984–2989 行 6 行变异、257 片对拍流、快测试本身、两条 GPU inode 流都复核不了；要主 agent 提交时跑。
- 没跑变异整表、没造第二批攻击镜像（X3 / X4 / X5 只是测试尖度探针，不是攻方的反例）。
- 没读实现员报告以外的补丁目录；第 2935 行锚点不判（正文说归第 30 批）。
- 没把 `.claude/` 下任何文件改一个字；写的只有本报告、模型目录与草稿目录。

## 复跑

- 命令：`SINGLEFS_REPO=/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree [RERUN_BUDGET_SECONDS=<秒>] bash research/prompts/m3-batch29-code-r1-sonnet-model/rerun.sh <空目录> all`（本机负载高，单条命令有 240 秒上限，所以分多次续跑；预算用完在单元 / 变异行边界退 75，再跑同一条命令续）。
- 在两个空目录各从头跑通一次（各自一份树副本、一个编译目录），汇总 `summary.txt` 各 75 行、sha256 相同：`3803bbb0bc73c9a9c521e85912b0517e9593aa03f61dc9362e3e8bdd14a0c597`；`bash rerun.sh compare <a> <b>` 原样输出：`compare: 两份 summary 排序后逐行相同（75 行）`。
- 汇总里的行：`STATIC`（变异表静态核）、`SITES`、`ROWCHECK`、`CONSTS`、`NEW`（本批新测试与整份二进制 154 条）、`ROW`（变异行结局：第 2977–2983 行 `RED named_red`，第 2984–2989 行 `SKIP`；`extra-mutations.tsv` 的 X1 `RED`、X2 `RED`、X3 `GREEN`、X4 `GREEN`、X5 `GREEN`、X6 `RED`）、`LEGIT`（9 个合法流二进制）、`BASE`（每条不同命令的原树基线全绿）。
- 模型文件：`rerun.sh`（cf9ff593…）、`mutrun.py`（a8767c72…，第 28 批 `mutrun.py` 的改键版）、`rowcheck.py`、`sites.py`、`extra-mutations.tsv`，都在 `research/prompts/m3-batch29-code-r1-sonnet-model/`。
