# m3-batch25-code-r1 正推腿报告（K1、K3）（2026-10-06）

腿：云端正推（Sonnet）。格：K1（N1–N4）、K3（7 份各一格）。被判文件 sha256 与 `research/prompts/m3-batch25-code-r1-snapshot/crates-sha256.txt` 一致：冻结副本（`/tmp/claude-1000/m3-batch25-code-r1/tree/`）与主工作区各跑一次 `sha256sum -c`，7 行都是 OK。
引 kb 写 kb 文件自己的行号；引代码写冻结副本里的路径:行号（行号都是 `grep -nF` 现取）。方案按 `crates/` 今天的实现来谈。

## 〇、结论速览

- N1（CPU）：与条款、判决一致。「走到末片」就是主走读 `walk_instance_table_chain` 在最新根那一遍收到合法的「没有下一片」链指针记录；门直接读它。门开、门关各有用例和变异钉住，我在自己的副本上把 4 行 CPU 变异加自拟的一条「门永不开」都跑了，全红（`research/prompts/m3-batch25-code-r1-sonnet-model/expected-summary.txt`）。
- N2（GPU）：走读内核与扫描内核的读写逐处对上，交接段第 30 个字确是原来空着的、不变长。读代码得到的「走到末片」条件与 CPU 同形，**只有一处读出来的差**：跨片上限（CPU 无显式页数上限，GPU 循环 256 片，掉出循环不置位也不判不了）。这一差在本批之前就有（旧门 `r_instance_rows` 也是 256 步），本批没引入，也没改。GPU 不跑，期望由 CPU 算。
- N3：三份坏镜像我各量了一次：N5-a 零红（只钉 I-9.29 成立，用例自己写明别的不钉）、N5-c 恰红 `I-2.3`、N5-b 红 `I-3.8` 且 I-9.29 部分没判。
- N4：两处写死偏移改成已有的命名常量，值同为 8；头注释补 I-9.24 与 CPU 一侧一致；三行替换点名的测试与参数与批前逐字段相同（第 2、3 项锚点/替换文变，命令与测试名不变）。
- 弱处（不是错）：GPU 对拍补的 N5-a 流在 GPU 一侧分不出「kind 2 页上门开不开」（两种开关在这条流上判定一样）；N5-b、N5-c 两形 GPU 没有对拍流。

## 一、K1 N1（CPU：`Walk::abandoned_roots_filtering`）

### 1.1 条款与判决今天说的

`.claude/kb/invariants.md:316`：「最新根的实例表读不出、被崩溃恢复抛弃的根滤不掉时记判不了」
`research/prompts/m3-batch23-code-r1-main-verification.md:37`：「N5：I-9.29 判不了的门改成看主走读有没有沿链走到末片（改法 A），N5-a、N5-c 那一形转成立、N5-b、N5-d 仍报部分没判；配变异；GPU 同改，对拍补 N5-a。」

由此推出「应该是什么」：判不了 当且仅当 候选集滤法所用的那一遍走读没有收全最新根的整张实例表；N5-a（链指针记录前多一条 kind 2）、N5-c（载荷 CRC 坏）成立；N5-b（链指针记录不合法）、N5-d（两份都读不出）判不了。

### 1.2 实现读到的

- 字段与默认：`crates/singlefs-checker/src/walk.rs:408`：「abandoned_roots_filtering: AbandonedRootsFilteringOfTheCandidateSet,」；初值 `crates/singlefs-checker/src/walk.rs:970`：「AbandonedRootsFilteringOfTheCandidateSet::UnfilteredAsTheMainWalkStoppedBeforeTheLastPageOfTheNewestRootsInstanceTable,」，默认是不滤（安全一侧）。
- 置位处：`crates/singlefs-checker/src/walk.rs:2929`：「ChainRecordView::LastPage => {」，下一行 `crates/singlefs-checker/src/walk.rs:2930`：「if mount_root_instance_when_newest.is_some() {」。只有 `LastPage` 一支置位，别的出口（读不出、解不开、身份不对、visited、Malformed）都是 break 不置位。
- 只有最新根那一遍带实例代号：`crates/singlefs-checker/src/walk.rs:1401`：「is_newest.then_some(mount_root_instance),」；由记录施加出来的版本传 None：`crates/singlefs-checker/src/walk.rs:1433`：「self.walk_instance_table_chain(&first_page_pointer, None);」；最新根先走：`crates/singlefs-checker/src/walk.rs:10537`：「walk.walk_root(&roots[newest_index].2.record_bytes, true);」。别处没有把字段重置回去（`grep -n 'abandoned_roots_filtering' walk.rs` 只有读、初值与这一处写）。
- 门：`crates/singlefs-checker/src/walk.rs:10579`：「let abandoned_roots_filtering = walk.abandoned_roots_filtering;」，同一遍走读，不另读。
- 滤的行与置位同源：行在循环之后由 `judge_instance_table_rows` 收，`crates/singlefs-checker/src/walk.rs:2947`：「if !pages.is_empty() {」、`crates/singlefs-checker/src/walk.rs:2971`：「self.instance_table_rows.push(parse_instance_table_row(row));」；走到 `LastPage` 时 `pages` 必非空，行与「滤全」同一轮。

### 1.3 「走到末片」各支怎么算（CPU）

| 情形 | CPU 怎么走 | 记滤全吗 | 依据 |
|---|---|---|---|
| 链指针记录合法、无下一片 | `LastPage` | 记 | `crates/singlefs-checker/src/walk.rs:733`：「0 if pointer.all_zero => ChainRecordView::LastPage,」 |
| 有下一片且指针非零 | `NextPage`，接着走 | 走到末片才记 | `crates/singlefs-checker/src/walk.rs:734`：「1 if !pointer.all_zero => ChainRecordView::NextPage(pointer),」 |
| 链指针记录不合法（宽不是 88、kind 不是 1、有无下一片不是 0/1、与指针不配） | `Malformed`，break | 不记 | `crates/singlefs-checker/src/walk.rs:728`：「if record.len() != INSTANCE_TABLE_RECORD_BYTES || record[0] != 1 {」；`crates/singlefs-checker/src/walk.rs:735`：「_flag_or_pointer_that_does_not_match => ChainRecordView::Malformed,」；`crates/singlefs-checker/src/walk.rs:2942`：「ChainRecordView::Malformed => break,」 |
| 页上没有记录 | `records.last()` 为 None 走 `Malformed` | 不记 | `crates/singlefs-checker/src/walk.rs:2920`：「.map_or(ChainRecordView::Malformed, |last| chain_record_view(last));」 |
| 载荷 CRC 坏、头校验和对 | 头判过（I-2.3 红），继续 | 照记 | `crates/singlefs-checker/src/walk.rs:1257`：「if !payload_crc_holds {」后 `return true` |
| 链指针记录前有非行记录（kind 2） | 只收 kind 0 的行，`chain_record_view` 只看最后一条 | 照记 | `crates/singlefs-checker/src/walk.rs:2971` 只 push kind 0；N5-a 用例 |
| 链成环、走到已走过的单元 | `visited_units` 已有 → break | 不记 | `crates/singlefs-checker/src/walk.rs:2884`：「if !self.visited_units.insert(page_placement) {」 |
| 读不出（两份整单元 CRC 都对不上） | `read_referenced_unit` 返回 None → break | 不记 | N5-d 用例（第 23 批已有） |
| 跨片上限 | 无显式页数上限，每一轮是盘上不同的单元（visited 兜底） | 走到才记 | `crates/singlefs-checker/src/walk.rs:2884`（同上） |

与旧门（只读不判的 `instance_table_rows_of_root_without_judging`）的差，旧读法多拒的三类：`crates/singlefs-checker/src/walk.rs:5741`：「if !matches!(check_unit(&unit), Ok(3)) {」（载荷 CRC、魔数等一并判）、`crates/singlefs-checker/src/walk.rs:5765`：「if row[0] != 0 {」（链指针记录前不是行就 None）、`Malformed` 交 None。新门不再受这三类拒收影响，这是判决要的。旧读法函数还有别的调用者（`crates/singlefs-checker/src/walk.rs:5900`、`crates/singlefs-checker/src/walk.rs:8008` 一带，抬 F 上限与 I-3.12），本批没动，也不在本批改法范围。

### 1.4 门开、门关各由谁钉住（CPU）

| 方向 | 用例 | 变异行（`crates/mutations.tsv`） | 我跑的结果 |
|---|---|---|---|
| 开（滤全、照比、成立） | N5-a：`crates/singlefs-harness/tests/checker_known_bad_images.rs:4050`；N5-c：`crates/singlefs-harness/tests/checker_known_bad_images.rs:4098` | `crates/mutations.tsv:2904`、`crates/mutations.tsv:2905`（把门改回旧读法） | 两行都红（点名的测试 FAILED，两行各带红另一条同形用例） |
| 开（任何形都不置位） | 同上三条与 N5-d 那条 | 无变异行；我自拟「LastPage 一支的门改 `if false`」 | 红 3 条（N5-a、N5-c、N5-d 用例） |
| 关（读不出、不比） | N5-d（第 23 批已有）：`crates/singlefs-harness/tests/checker_known_bad_images.rs:3953` | `crates/mutations.tsv:2841`（初值改成「滤全」） | 红（N5-d 用例） |
| 关（链指针记录不合法） | N5-b：`crates/singlefs-harness/tests/checker_known_bad_images.rs:4126` | `crates/mutations.tsv:2906`（`Malformed` 也当走到末片） | 红（N5-b 用例） |

复跑：`research/prompts/m3-batch25-code-r1-sonnet-model/rerun.sh`（SINGLEFS_REPO 取仓路径，空目录里从头跑通，退 0，与 `expected-summary.txt` 一致）。变异 5 条各一行 MUTATION，红的测试列在 RED 行；探针 5 行在 `PROBE` 行。

没有逐形钉住的关向（读代码得出，我没去旧用例里逐个找）：身份四元组不符、I-1.7 记录不合法、visited 成环、加密预留位非 0 这四支都落在「不置位」的默认上；本批新加的三条用例和旧的 N5-d 都不针对它们。它们与「默认不滤」是同一个落点，要打中得有人把默认改成「滤」，那时 N5-d 和 2841 行会红。

## 二、K1 N2（GPU：交接段第 30 个字）

### 2.1 读写链

- 变量：`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1577`：「var<private> w_newest_instance_table_chain_reached_its_last_page: bool;」
- 清零：`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3690`：「w_newest_instance_table_chain_reached_its_last_page = false;」，在择根之后、`w_start(0u, true)` 之前。
- 置位：`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1666`：「if (chain == 0u) {」下一行 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1667`：「w_newest_instance_table_chain_reached_its_last_page = true;」；在 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1630`：「if (judge_rows) {」那一支里，`judge_rows` 即 `is_newest`：`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3163`：「w_walk_instance_table_chain(parse_node_pointer(record_base, instance_table_pointer_offset), is_newest, fsid_low);」；`is_newest` 只在最新根阶段为真：`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3709`：「walk_is_newest = true;」。
- 写交接段：`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:4027`：「sws(H_BASE + H_NEWEST_INSTANCE_TABLE_CHAIN_REACHED_ITS_LAST_PAGE, select(0u, 1u, w_newest_instance_table_chain_reached_its_last_page));」，与别的交接字同一块。
- 字的位置：`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:728`：「const H_NEWEST_INSTANCE_TABLE_CHAIN_REACHED_ITS_LAST_PAGE: u32 = 30u;」；`H_BASE` 在 `crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:695`：「const H_BASE: u32 = 235472u;」，`H_BELOW_FLOOR_SLOTS` 在 `crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:730`：「const H_BELOW_FLOOR_SLOTS: u32 = 235504u;   // 盘、槽 lo/hi」，235472 + 32 = 235504，第 30 字确在原来空着的两字里，不压别的；`H_END` 与 `SCRATCH_END` 本批没动（diff 里没有这两行）。
- 读：`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3098`：「d_object_births_are_compared = sw(H_BASE + H_NEWEST_INSTANCE_TABLE_CHAIN_REACHED_ITS_LAST_PAGE) == 1u;」；不再调 `r_instance_rows`（它还留着给恢复那两处用：`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1093`：「fn r_instance_rows(record_base: u32) -> bool {」）。

### 2.2 「走到末片」CPU 与 GPU 逐支对表

| 情形 | CPU | GPU | 一致吗 |
|---|---|---|---|
| 链指针记录读法 | `crates/singlefs-checker/src/walk.rs:728`：「if record.len() != INSTANCE_TABLE_RECORD_BYTES || record[0] != 1 {」，之后 0+全零指针=末片、1+非零=下一片、其余 Malformed | `crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:604`：「if (width != 88u || rd8(base, o) != 1u) {」；0 且指针 86 字节全零=0、1 且非零=1、其余 2 | 一致 |
| 页上无记录 | Malformed | `chain` 初值 2，`p.record_count > 0u` 才读（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1626`） | 一致 |
| 载荷 CRC 坏（头对） | `judge_unit_header` 返回 true 继续 | `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1364`：「if (!payload_holds) {」后 `return true;` | 一致 |
| 读不出（两份整单元 CRC 不符） | break 不置位 | 同（`w_read_referenced_unit` 返回 NONE → break，`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1601` 一带） | 一致 |
| visited 成环 | break 不置位 | `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1601`：「if (!w_visit(pointer.device0, pointer.slot0)) {」→ break 不置位 | 一致 |
| 容器不合法 / 身份不符 | break 不置位 | `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1609`：「if (!packed.ok) {」→ break；身份不符 break | 一致 |
| 只收最新根那一遍 | `is_newest.then_some(..)` | `judge_rows`（= `is_newest`） | 一致 |
| 行数上限 | 无上限 | `crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:364`：「const C_MAX_INSTANCE_ROWS: u32 = 1024u;」，超了置 `UNDECIDABLE_TOO_MANY_ENTRIES`（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1642`：「if (w_instance_row_count >= C_MAX_INSTANCE_ROWS) {」），整个状态判不了 | 不同，但 GPU 一侧是把状态整个标判不了，不会静默漏判；本批前就这样 |
| **跨片上限** | 无显式上限，visited 兜底 | `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1591`：「for (var page = 0u; page < 256u; page = page + 1u) {」；掉出循环不置位，也不置判不了 | **不同**（规则没说） |

### 2.3 跨片上限一格（读代码得出，GPU 不跑，期望用 CPU 算）

- CPU 算的期望：最新根的实例表若是 257 片的合法链（每片身份四元组 (0, 4, 片序号, 0) 对、末片链指针记录「没有下一片」），主走读走到末片，`abandoned_roots_filtering` 记滤全，I-9.29 照比；候选集也按 257 片的全部行滤。
- GPU 读到的：`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1591` 的循环只走 256 片，第 257 片不进循环，`w_newest_instance_table_chain_reached_its_last_page` 保持 false，扫描内核 `d_object_births_are_compared` 为 false，I-9.29 一个号都不比，也不报判不了；候选集按前 256 片的行滤。
- 所以在这条链上 GPU 不比、CPU 比，CPU 若红 I-9.29 则 GPU 少这一红。
- 来历：旧门 `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1097`：「for (var step = 0u; step < 256u; step = step + 1u) {」同样 256 步，掉出循环返回 false；本批把门换成走读那一遍后，256 的上限跟着走读循环走，行为一样。所以这不是本批引入的差。
- 够不够得着：造 257 片需要镜像里真有 257 个不同的实例表单元，每片至少一条链指针记录；行数上限 1024 对每片只有链指针记录的链不起作用，所以崩出这一形不靠行数。mkfs 与正常写路径不会产生，只能是人造镜像。我没有造。
- 条款对它没说话（`.claude/kb/invariants.md:316` 只说读不出、滤不掉时判不了；257 片都读得出）。判定：规则没说。

### 2.4 门开、门关各由谁钉住（GPU；我不跑 GPU，只读用例与变异行）

| 方向 | 用例 | 变异行 |
|---|---|---|
| 开（滤全、照比） | N5-a 流：`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:6744`（`fn the_gpu_judge_agrees_with_the_cpu_that_an_abandoned_root_is_filtered_by_a_newest_instance_table_carrying_a_record_of_an_unregistered_kind(`） | `crates/mutations.tsv:2908`：走读内核不置位；CPU 一侧的前提 `crates/mutations.tsv:2907`：门改回旧读法，用例里 CPU 先断言成立，取卡在断言之后（`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:6758`：「let Some(card) = one_usable_card_or_report_not_run(」），没卡也红（读代码得出，未跑） |
| 关（读不出、不比） | 第 24 批流（两份翻坏）：`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:6690` | `crates/mutations.tsv:2903`：扫描内核 `d_object_births_are_compared = true`（本批替换了锚点，命令与测试名不变） |
| 关（链指针记录不合法、载荷 CRC 坏等） | 没有 GPU 流 | 没有变异行 |

弱处一：N5-a 流上，实例 3 的根成最新的那些状态里 C 已被按行滤掉，门开或门关 I-9.29 都不红，GPU 与 CPU 判得一样，分不出；只有 C 仍是最新根的那些状态（C 自己的表没动）里，门必须开、CPU 才红 I-9.29，由此钉住「门会开」。用例自己的注释写明：`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:6741`：「这条流上门开不开在违例位上看不出（C 滤掉了，比也不红）」。也就是说 GPU 一侧「kind 2 页上门开不开」没有被一条用例单独钉住；但读代码看 GPU 的 `chain` 只看最后一条记录（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1626`），与 kind 无关，所以现在的代码没有这一支的分歧。
弱处二：GPU 一侧关向只有「读不出」一形有对拍流；N5-b（链指针记录不合法）与 N5-c（载荷 CRC 坏）在 GPU 上没有流。若有人把 GPU 的 `chain == 0u` 改成 `chain != 1u`（把不合法当末片），现有 GPU 用例与变异行都抓不到；CPU 这一侧由 `crates/mutations.tsv:2906` 抓。这是缺的钉子，不是现有代码的错。

## 三、K1 N3（坏镜像）

我在自己的副本上追加一个探针用例（`research/prompts/m3-batch25-code-r1-sonnet-model/probe.rs`），把三份镜像各过一次 `check_pool_image`，量红的不变量与 I-9.29：

| 镜像 | 量到 | 用例自己钉的 | 判 |
|---|---|---|---|
| N5-a（kind 2 插在链指针记录前，重封） | `research/prompts/m3-batch25-code-r1-sonnet-model/expected-summary.txt:6`：「PROBE N5-a violated=[] i929=Holds」 | `crates/singlefs-harness/tests/checker_known_bad_images.rs:4046`：「这张镜像别的几条红不红不在这里钉」；只钉 I-9.29 成立 | 零红，只红目标（目标是「成立」）；但「零红」没有用例钉 |
| N5-c（载荷 CRC 翻一位，只重封头） | `research/prompts/m3-batch25-code-r1-sonnet-model/expected-summary.txt:9`：「PROBE N5-c violated=["I-2.3"] i929=Holds」 | `crates/singlefs-harness/tests/checker_known_bad_images.rs:4112`：「["I-2.3"],」 | 恰红 I-2.3，成立；用例钉了「只红」 |
| N5-b（有无下一片写 2，重封） | `research/prompts/m3-batch25-code-r1-sonnet-model/expected-summary.txt:8`：「PROBE N5-b flag=2 violated=["I-3.8"] i929=PartiallyJudged(」 | `crates/singlefs-harness/tests/checker_known_bad_images.rs:4144`：「violated_invariants(&verdicts).contains(&"I-3.8"),」；I-9.29 部分没判 | 红的只有 I-3.8（探针量到，用例只钉「含」） |
| N5-b 变体（有无下一片写 1、指针全零） | `research/prompts/m3-batch25-code-r1-sonnet-model/expected-summary.txt:7`：「PROBE N5-b flag=1 violated=["I-3.8"] i929=PartiallyJudged(」 | 无 | 同上，部分没判 |
| 对照：C 被抛弃的干净镜像 | `research/prompts/m3-batch25-code-r1-sonnet-model/expected-summary.txt:10`：「PROBE clean violated=[] i929=Holds」 | `crates/singlefs-harness/tests/checker_known_bad_images.rs:3953` 一带第一段断言 | 零红 |

与判决对：判决（`research/prompts/m3-batch23-code-r1-main-verification.md:37`）要「N5-a、N5-c 那一形转成立、N5-b、N5-d 仍报部分没判」：N5-a、N5-c 成立、N5-b 部分没判、N5-d 由旧用例钉，一致。GPU 对拍补 N5-a 一条流：`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:6744` 一致（判决点名「对拍补 N5-a」，没点名 N5-b、N5-c，所以这两形没有 GPU 流不算少做）。

## 四、K1 N4（命名常量、头注释、变异表）

- 常量：`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:139`：「const INODE_RECORD_OBJECT_BIRTH_OFFSET: u32 = 8u;」（共用那一份，拼在三个内核前面，本批没动它）；CPU 一侧 `crates/singlefs-checker/src/walk.rs:213`：「const INODE_RECORD_OBJECT_BIRTH_OFFSET: usize = 8;」，同值。
- 两处替换：`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2675`：「w_judge(INVARIANT_I_9_26, le64(rd64(p.base, ro + INODE_RECORD_OBJECT_BIRTH_OFFSET), change_count) && le64(change_count, p.birth_txg));」与 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2678`：「let birth = rd64(p.base, ro + INODE_RECORD_OBJECT_BIRTH_OFFSET);」。`grep -nF 'ro + 8u'` 在三份 wgsl 里零命中（我 grep 过：`crash_judge_checker.wgsl` 与 `crash_judge_checker_scan.wgsl` 里没有输出）。
- 头注释：`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2092`：「// I-9.5、I-3.4、I-4.7、I-4.6、I-9.24 在回退候选集里每条根与由记录施加出来的那几版上各重读一遍树表与几棵树（只读）。」；里面真有 I-9.24：`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2393`：「violate(INVARIANT_I_9_24);」；条目那一处同补：`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2505`：「// I-9.24、I-9.5、I-4.7、I-4.6（walk.rs judge_deletion_queue_of_a_version）：每一条 deleted_inodes 条目」。CPU 一侧 I-9.24 住在 `judge_deletion_queue_of_a_version`（`crates/singlefs-checker/src/walk.rs:8879`：「/// I-9.24（删除队列至多一条 key）：每一条 `deleted_inodes` 条目都判，读队列在配对之前，配不配得上 inode 树都判；空队列报成立」），注释说得对。kb 一侧：`.claude/kb/invariants.md:311`：「每条 `deleted_inodes` 条目都判，不管树表条目的头 ID 配不配得上」。
- 变异表三行替换，我拿批前拷贝（`/tmp/claude-1000/m3-batch25/before/crates/mutations.tsv`）与冻结副本逐字段比，命令与输出原样：

```
old rows 2898 new rows 2903 only-new names 5 only-old names 0
第 5 批 GPU 判器：I-9.26 改动计数不比 | 文件同 | 原文 异 | 替换文 同 | 命令 同 | 测试名 同
第23批 X I-9.29：最新根的实例表读不出时当它滤 | 文件同 | 原文 异 | 替换文 异 | 命令 同 | 测试名 同
第 24 批 GPU 判器 I-9.29：最新根的实例表 | 文件同 | 原文 异 | 替换文 同 | 命令 同 | 测试名 同
```

  三行点名的测试与 cargo 参数（第 5、6 段）与批前逐字段相同，只有锚点（第 3 段）随改动变，第 23 批 X 那行的替换文也跟着换（它的变异语义「初值改成滤全」不变，变异名未改）。新追加 5 行：`crates/mutations.tsv:2904`、`crates/mutations.tsv:2905`、`crates/mutations.tsv:2906`、`crates/mutations.tsv:2907`、`crates/mutations.tsv:2908`。8 行（含三行替换）的原文我用脚本在冻结副本里数过，每行恰好命中 1 次，点名的测试 fn 都存在，命令里的过滤串能罩住点名的测试名。
- CPU 的 3 条 + 替换的 2841 行我真跑了（`expected-summary.txt` 里 MUTATION 行）；GPU 的 2907、2908、2903、1926 四行要卡，我没跑。

## 五、K3（7 份各一格）

| 文件 | 判到什么 | 判 |
|---|---|---|
| `crates/singlefs-checker/src/walk.rs` | 字段、初值、LastPage 置位、门读字段、枚举改名后两个消费处（`crates/singlefs-checker/src/walk.rs:9686` 一带 match）与报文；旧读法没有再用在 I-9.29 的门。第一节与 1.3 表 | 与条款、判决一致 |
| `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` | 变量、清零、置位、写交接段、两处命名常量。第二节 2.1、第四节 | 一致；与 CPU 差一格跨片上限（2.3） |
| `crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl` | 新常量 30，落在原空字里，`H_BASE + 32 == H_BELOW_FLOOR_SLOTS`，交接段与 `H_END` 不变。2.1 | 一致 |
| `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl` | 门改读第 30 字；旧 `r_instance_rows` 仍给恢复两处用；头注释与条目注释补 I-9.24；三处注释的新措辞与 CPU 的 `Walk::abandoned_roots_filtering` 对得上。2.1、第四节 | 一致 |
| `crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` | 重构成 `ChangeOfTheNewestInstanceTable` 两形；旧流名 `flow_name()` 不变（旧流名在 `crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:6187` 一带，由枚举给出）；新增 N5-a 流与用例。2.4 | 一致；GPU 一侧门开向在这条流上分辨力弱（2.4 弱处一） |
| `crates/singlefs-harness/tests/checker_known_bad_images.rs` | 三份新镜像与帮助函数；N5-a、N5-c、N5-b 用例按判决转成立 / 只红 I-2.3 / 部分没判。第三节 | 一致；N5-a 用例不钉「零红」（用例自己说明） |
| `crates/mutations.tsv` | 3 行替换 + 5 行追加。第四节 | 一致 |

## 六、什么现象会推翻各条结论

| 结论 | 推翻它的现象 |
|---|---|
| N1 一致 | 造出一形主走读走到了 `LastPage` 而被抛弃的根留在候选集（I-9.29 误红），或没到 `LastPage` 而候选集其实滤全了；或者找到 `abandoned_roots_filtering` 被重置或在别的入口没被写；或把 `if false` 之外的某个别的合理变异（例如把 `Malformed` 当 `LastPage`）放进去而四条用例都不红 |
| N2 与 CPU 同条件 | GPU 与 CPU 在一个我没列的情形上给出不同的「走到末片」（对拍出一个状态两侧不一致）；或者交接段第 30 个字被走读之后的别处写过 / 被扫描内核之前的别处读错 |
| 跨片上限一格 | 造一条 257 片的链，GPU 上 `d_object_births_are_compared` 仍为 1（我读到的循环 256 会被推翻），或 CPU 对这条链也报判不了 |
| N3 三镜像 | 换一台机器、同一份代码，探针的 PROBE 行与 `expected-summary.txt` 不同 |
| N4 | `grep -nF 'ro + 8u'` 在三份 wgsl 里出现命中；或三行替换的命令、测试名与批前不同 |
| 弱处一、二 | 一条能区分 GPU「kind 2 页门开不开」的 GPU 用例；或 GPU 上 N5-b 流的对拍 |

## 七、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1 N1 | 一致 | 「走到末片」= 主走读最新根那一遍收到合法的「没有下一片」；门开关四个方向 CPU 都有用例或变异，4 行 + 1 自拟变异我跑过全红 |
| K1 N2 | 一致（一处规则没说） | 读写链、位置、清零与 CPU 同形；唯一读出的差是跨片上限 CPU 无限、GPU 256，本批前就有；GPU 不跑，读代码得出 |
| K1 N3 | 一致 | N5-a 零红且成立、N5-c 只红 I-2.3、N5-b 红 I-3.8 且部分没判，探针量过；N5-a 用例不钉「零红」，GPU 无 N5-b / N5-c 流 |
| K1 N4 | 一致 | 两处 `ro + 8u` 换成同值命名常量，I-9.24 头注释属实，三行替换的命令与测试名与批前相同 |
| K3 `crates/singlefs-checker/src/walk.rs` | 一致 | 见五 |
| K3 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` | 一致 | 差一格跨片上限，见 2.3 |
| K3 `crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl` | 一致 | 第 30 字落在原空字里，交接段不变长 |
| K3 `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl` | 一致 | 门读第 30 字，注释属实 |
| K3 `crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` | 一致 | N5-a 流补上；GPU 门开向在这条流上分辨力弱 |
| K3 `crates/singlefs-harness/tests/checker_known_bad_images.rs` | 一致 | 三份镜像与判决对得上 |
| K3 `crates/mutations.tsv` | 一致 | 3 替换 + 5 追加，锚点各命中 1 次 |

## 八、没做什么

- 不判 K2；没有去攻（造反例）；257 片的链只读代码推，没造镜像。
- GPU 一律没跑（正文规定）；GPU 对拍、GPU 那两行与 2903、1926 行变异归提交时主 agent 跑。复核不了：GPU 侧的数全是读 wgsl 加 CPU 算期望，没有卡上的数。
- 重型测试没跑；我只跑了 `singlefs-harness` 的 `checker_known_bad_images` 一个测试目标（harness 档），经 `scripts/hosts/run-with-memory-cap.sh`（编译 6G、跑 4G）与 `scripts/hosts/capped.sh 10`。起跑前 `ps` 看到别的 agent 在跑 `cargo test`（`m3-r1/two-leads` 的副本），没有性能测量（qemu、vm-bench、fio）在跑，照常加 `nice -n 19` 跑。排队时遇过一次「同会话只许一条」被拒，改用 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON` 声明理由放行；峰值表对上限的 2 倍规则用 `RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON` 声明，上限仍是 6G 与 4G，见 `rerun.sh` 文件头。
- 旧用例没有逐个翻：1.4 末段列的「没有逐形钉住的关向」只看了本批新加的三条与 N5-d。
- 不采纳、不出判决。
- 实现员报告 `/tmp/claude-1000/m3-batch25/n/report.md` 没读（只当线索）；批前拷贝 `/tmp/claude-1000/m3-batch25/before/` 只用来比 mutations.tsv 的三行替换。
- 临时目录：编译与副本目录 `/tmp/claude-1000/m3-batch25-code-r1-sonnet/repo`、`/tmp/claude-1000/m3-batch25-code-r1-sonnet/target`、`/tmp/claude-1000/m3-batch25-code-r1-sonnet-rerun` 已在交回前删掉（`ls /tmp/claude-1000/m3-batch25-code-r1-sonnet` 只剩日志与脚本：build1.log、run2.log、mut*.py、run-probe*.sh、rerun*.out、old-mut.tsv 等，没有编译目录与仓副本）；草稿目录 `/tmp/claude-1000/m3-batch25-code-r1-sonnet/` 留着，里面是日志。

## 九、模型

`research/prompts/m3-batch25-code-r1-sonnet-model/`：`rerun.sh`（SINGLEFS_REPO 必填，SONNET_WORK 默认 `/tmp/claude-1000/m3-batch25-code-r1-sonnet-rerun`，不存在才跑）、`probe.rs`（追加进副本的探针用例，只在副本里）、`mut.py`（对副本 walk.rs 逐条加变异跑点名测试）、`expected-summary.txt`（排序后的 23 行）。空目录里从头跑通一次，退 0，末行「与 expected-summary.txt 一致」。
