# 重放前缀包在途改动代码轮第一轮：正推腿报告

腿：three-way-forward（Sonnet）。日期 2026-11-16。方案按 `crates/` 今天的实现来谈。

## 0 开头要交代的

- 开工先核十四份被判文件的 sha256：`sha256sum -c research/prompts/replay-prefix-backfill-code-r1-snapshot/crates-sha256.txt` 十四行全是 OK，没有被别的会话改过。kb-sha256.txt 两行也是 OK（`.claude/gate.d/verus-pins.tsv`、`.claude/kb/decisions/13-验证路线.md`）。
- **一次越线读到**：我为了核「说明文件里 `--manual` 的写法」跑了一条范围太宽的 `grep -rn -- '--manual' research/prompts/*.md`，输出里带出了另一条腿报告 `research/prompts/replay-prefix-backfill-code-r1-opus-output.md` 的第 177 行（一整行）。只有这一行，没有读那份文件的别处，也没有按它改我的结论。我关于 `--manual` 的结论在这之前已经从 `task-server/task_server.py:9286-9290` 独立读到（见第 8 节「带 --item 的命令与项」），主 agent 判决时请把这一行当作我这条腿「不完全独立」的一处。
- 没跑 Verus、没起 cargo、没起任何任务。所有「证明会不会过」「必红行会红在哪」的话是读代码推的（标「推的」）；要跑的命令在「要主 agent 跑的命令」一节。
- 没有写模型，所以没有 `<模型目录>`、`rerun.sh`。我自己的草稿脚本（只读被判文件做核对）放在 `~/.local/share/singlefs/work/replay-prefix-backfill-code-r1-forward/`：`tokdiff.py`（标记级改名核对）、`anchors.py`、`grepcheck.py`、`landings.py`（必红表锚点与落点核对）。
- 引号里的原文都在被引文件那一行 `grep -nF` 过；行号现取，不取自背景材料。
- 本报告里「对得上」只表示「性质名、头注释、说明文件说的，与 `ensures` 逐字说的是一回事」，不表示「Verus 已经验过」。

## 1 G01 到 G30 逐条对

读法：先读 `GPU_GC_PROOFS.md` 表（只列到 G20）与 `:5` 的一句话（点了 G21 到 G30 的名目，次序与代码里 `/// G21` 到 `/// G30` 的注释一一对上），再读每条 `ensures`。G21 到 G30 在 `GPU_GC_PROOFS.md` 里**没有逐条表格**，我拿代码里的 `/// G2x` 文档注释与 `:5` 的名目对。

### 1.1 G01 到 G20（`crates/singlefs-replay-prefix/src/gpu_gc_proofs.rs`）

| G | 函数 | 行 | 判定 | 说明 |
|---|---|---|---|---|
| G01 | `reconcile_gpu_entry` | gpu_gc_proofs.rs:28 | 对得上 | 六条 `ensures`：`confirmed == agree`；`ask_cpu == cpu_is_needed(previous, agree)`；两者不同时为真；`uncertain_rounds == next_uncertain_count`；`<= 1`；「既不确认也不交 CPU」当且仅当「分歧且 previous == 0」。表里「一致才确认、首轮暂缓、第二轮交 CPU、三种互斥、计数不溢出」六样都在。 |
| G02 | `lemma_two_disagreements_trigger_exactly_one_cpu_judgment` | gpu_gc_proofs.rs:47 | 对得上 | 无前提；四条：首次分歧不交 CPU，计数变 1，第二次分歧交 CPU，计数回 0，第三次不交。「恰好一次、之后重新计数」只证了从零起的两步窗口，不是任意长序列，但名字与表说的就是这个窗口。 |
| G03 | `lemma_agreement_restarts_the_disagreement_history` | gpu_gc_proofs.rs:57 | 对得上 | 对任意 `previous: nat`：一致后计数为 0、不交 CPU、再来一次分歧计数是 1。 |
| G04 | `lemma_failed_batch_keeps_all_counts_and_cpu_calls` | gpu_gc_proofs.rs:83 | 对得上（定义复述） | 三条：失败轮状态不变；成功轮 `.0 == update_count`；成功轮 `.1` 精确加一或不加。但 `round_state`（`:74`）本身就是按这三句定义的 spec 函数，引理是它的定义复述，与产品 `reconcile_unless_the_batch_failed`（`two_kernel_reconciliation.rs:110`）之间没有机器连接；表里标「模型」是诚实的。 |
| G05 | `lemma_distinct_entry_updates_commute` | gpu_gc_proofs.rs:93 | 对得上 | `requires first != second`；四条：先后次序换了的 `update_count` 相等，另一条目的计数不变，另一条目是否交 CPU 不变，整轮 `round_state`（含 CPU 调用总数）次序可换。 |
| G06 | `lemma_count_updates_preserve_the_sparse_counter_invariant` | gpu_gc_proofs.rs:116 | 对得上 | `requires` 表里每个值都是 1；三条：更新后每个值仍是 1，别的键的计数不变，本键新值等于 `next_uncertain_count`。 |
| G07 | `lemma_live_slot_intersection_bounds` | gpu_gc_proofs.rs:133 | 对得上 | 四条：`0 <= 交 <= 左跨度`，`<= 右跨度`，对称，「`左末 <= 右始` 或 `右末 <= 左始`」时为 0（比表里「相邻区间」更宽：不相交也为 0）。 |
| G08 | `lemma_splitting_a_segment_preserves_live_slot_contributions` | gpu_gc_proofs.rs:144 | 对得上 | `requires` 记录区间有效、`start <= seam <= end`；一条：原段的交 = 左半的交 + 右半的交。 |
| G09 | `lemma_record_slices_preserve_the_score` | gpu_gc_proofs.rs:171 | 对得上 | 对任意 `first <= seam <= past <= len`，`[first,past)` 的和 = `[first,seam)` + `[seam,past)`；`decreases past - seam` 的归纳，不是定深枚举。令 `first = 0, past = len` 就是「整表」。 |
| G10 | `lemma_duplicate_records_keep_their_multiplicity` | gpu_gc_proofs.rs:183 | 对得上（只证两份） | `seq![r, r]` 的和 = `2 * 单份贡献`。名字说「保留重数」，证的是重数 2；足以拒绝「按槽集合去重」，但不是任意重数 n。 |
| G11 | `lemma_released_and_foreign_records_do_not_score` | gpu_gc_proofs.rs:191 | 对得上 | `requires` 记录与段都是非空区间；三条：已释放或异盘 ⇒ 0；贡献 > 0 当且仅当「未释放、同盘、严格相交」；同盘活记录的贡献恰为 `overlap_slots`。 |
| G12 | `lemma_two_word_slot_order_is_u64_order` | gpu_gc_proofs.rs:207 | 对得上 | 无前提；三条：两个双字值落在 `0..=u64::MAX`，`<` 当且仅当「高字更小或高字相等低字更小」，`==` 当且仅当两字都等。 |
| G13 | `lemma_two_word_span_addition_carries_exactly_once` | gpu_gc_proofs.rs:216 | 对得上 | `requires` 全宽和不越 u64；四条：低字回绕值在字范围内，高字加进位不越 `u32::MAX`，重组等于全宽和，「回绕值小于原低字」当且仅当「发生进位」。进位只可能是 0 或 1（公式里写死），所以「至多一次」成立。 |
| G14 | `checked_live_slot_totals` | gpu_gc_proofs.rs:235 | 对得上 | `Some` 时两个累计各等于各自的和；`None` 时至少一个和超过 `u64::MAX`。 |
| G15 | `checked_gpu_input_bytes` | gpu_gc_proofs.rs:255 | 对得上 | `None` ⇒ 对齐后长度 > `u64::MAX`；`Some` ⇒ 等于 `aligned_input_size`（空输入为 4）、`>= 4`、`>= input`、四的倍数、`input > 0` 时 `< input + 4`（最小）。 |
| G16 | `checked_gpu_chunk_bytes` | gpu_gc_proofs.rs:267 | 对得上 | `Some` ⇒ 对齐输入 + 条数 × 每条 + 固定；`None` ⇒ 这个全宽和 > `u64::MAX`。函数体里每个中间值溢出都会 `None`，而 `None` 的 `ensures` 只说总和溢出：中间值溢出必推出总和溢出（各项非负），所以不矛盾。 |
| G17 | `remaining_gpu_memory` | gpu_gc_proofs.rs:287 | 对得上（表）；注释多半句推不出 | 三条：`remaining == 若 held < limit 则 limit - held 否则 0`，`<= limit`，`held <= limit` 时 `held + remaining == limit`。表里「不回绕、未超额时已占加剩余恰为总限」都在。`:285` 的文档注释另写「正的新增分配必在总限内」：`ensures` 里没有「新增分配」这个量，这半句**推不出**（要再加一步推理才得到）。 |
| G18 | `next_gpu_record_slice_end` | gpu_gc_proofs.rs:301 | 对得上 | `requires start <= total, capacity > 0`；六条：`start <= end <= total`，`end - start <= capacity`，`end` 等于「余量不超容量取 total，否则 start + capacity」，`start < total` 时有进展，`[start,total)` 恰是 `[start,end)` 与 `[end,total)` 的并，两段不交。「避免先加后取 min 的溢出」是函数体的写法，由 Verus 的算术溢出检查罩着。 |
| G19 | `gpu_record_is_in_scan_region` | gpu_gc_proofs.rs:316 | 对得上 | 一条：`read == 同盘 && 未释放 && start <= slot < end && slot + span <= end`（`slot + span` 在规格里是数学整数，溢出自然是 false）。零跨度允许，与表一致。 |
| G20 | `decoded_gpu_live_slots` | gpu_gc_proofs.rs:327 | 对得上 | `None` ⇒ `verdict != 1`；`Some(v)` ⇒ `verdict == 1 && v == words_value(low, high)`。两条合起来「当且仅当」，高低两字都保留。 |

### 1.2 G21 到 G30（`crates/singlefs-replay-prefix/src/gpu_gc_capacity_proofs.rs`）

| G | 函数 | 行 | 判定 | 说明 |
|---|---|---|---|---|
| G21 | `gpu_chunk_fits` | gpu_gc_capacity_proofs.rs:28 | 对得上 | `fits == chunk_fits(...)`，规格 `:17` 同时要求：对齐输入 <= 绑定上限，条数 × 每条缓冲 <= 绑定上限，条数 <= 派发上限，有限额时 对齐输入 + 条数 × 每条占用 + 固定 <= 限额。规格里的乘加是数学整数，所以「溢出不误放行」成立。 |
| G22 | `largest_gpu_piece_input` | gpu_gc_capacity_proofs.rs:52 | 对得上 | `None` ⇒ `!chunk_fits(0, 1, cap)`；`Some(b)` ⇒ `b >= 4`、四的倍数、`chunk_fits(b, 1)`、且任何装得下的输入都 `<= b`。「`None` 当且仅当最小输入也放不下」：`Some` 蕴涵 `chunk_fits(0,1)`（把 `b` 换成 4 只会更小），所以反向也成立（推的，由两条 `ensures` 推出，不是逐字写的）。 |
| G23 | `tightened_gpu_memory_limit` | gpu_gc_capacity_proofs.rs:80 | 对得上 | `limit == 无限额取 available，有限额取两者小者`；`limit <= available`。「已有限额时绝不放宽」即 `limit <= old`，由第一条的 `min` 直接得到。 |
| G24 | `checked_gpu_memory_hold` | gpu_gc_capacity_proofs.rs:91 | 对得上 | `Some(total)` ⇒ 无未完成提交、`total == held + bytes`、有限额时 `<= 限额`；`None` ⇒ 有未完成提交，或 `held + bytes > u64::MAX`，或超限额。两条臂互补。 |
| G25 | `checked_gpu_memory_release` | gpu_gc_capacity_proofs.rs:110 | 对得上 | `None` ⇒ `bytes > held`；`Some(left)` ⇒ `bytes <= held && left + bytes == held`。 |
| G26 | `gpu_bytes_after_submission_wait` | gpu_gc_capacity_proofs.rs:120 | 对得上 | `None` ⇒ `chunk > held`；`Some(left)` ⇒ `chunk <= held` 且完成则 `held - chunk`、没完成则 `held`。 |
| G27 | `gpu_memory_cap_after_allocation_failure` | gpu_gc_capacity_proofs.rs:132 | 对得上 | 三条：`2*next <= failed < 2*next + 2`（误差至多一），`failed > 0` 时严格下降，`failed == 0` 时为 0。 |
| G28 | `gpu_workgroups_for_entries` | gpu_gc_capacity_proofs.rs:143 | 对得上 | `requires per_group > 0`；四条：`groups <= entries`，`groups == 0` 当且仅当 `entries == 0`，`entries <= groups * per_group`，`entries > 0` 时 `(groups-1) * per_group < entries`，后两条合起来就是上取整。两处 `as` 的问题见第 8 节。 |
| G29 | `checked_gpu_mapping_table_bytes` | gpu_gc_capacity_proofs.rs:161 | 对得上 | `None` ⇒ `60*m(entries) + 16*m(locations) > u64::MAX`（`m` 是「空表占一项」，`:155`）；`Some((total, largest))` ⇒ 总量等于该和，最大绑定量等于 `max(56*m(entries), 16*m(locations))`。函数体里任一中间乘加溢出都蕴涵总和溢出，所以「任何溢出拒绝」成立。常量 56、16、4 对产品真缓冲的核对在第 2 节末。 |
| G30 | `extended_gpu_chunk_input` | gpu_gc_capacity_proofs.rs:183 | 对得上（但产品不调用，见第 2 节） | `Some(total)` ⇒ `total == previous + additional` 且 `chunk_fits(total, ...)`；`None` ⇒ 和超 `u64::MAX`，或把和转回 `u64` 后装不下。 |

### 1.3 打分文件里不挂 G 号的 `ensures`（`gpu_gc_scoring_proofs.rs`），逐条对 `GPU_GC_PROOFS.md:11-22`

| 说明文件说的 | 函数:行 | 判定 |
|---|---|---|
| `:13` 二分对 `(below, above)` 归纳、下界不是输入假设 | `lower_bound_of_sorted_records` gpu_gc_scoring_proofs.rs:32 | 对得上：前提只有有序与前缀、后缀不变量，`ensures is_lower_bound`，`decreases above - below` |
| `:14` 窗内全是目标设备，窗外贡献为零，饱和减法覆盖段前伸入的记录 | `lemma_candidate_window_keeps_every_contribution` gpu_gc_scoring_proofs.rs:103 | 对得上：三条 `ensures` 正是这三句；`earliest_candidate_start`（`:92`）是饱和减 |
| `:15` 对跨度归纳，逐槽判定推出交集长度 | `lemma_counting_slots_equals_interval_intersection` gpu_gc_scoring_proofs.rs:81 | 对得上 |
| `:16` 对已访问条数归纳，裁剪窗口与整表扫描前缀对应 | `lemma_search_prefix_matches_the_walk` gpu_gc_scoring_proofs.rs:150 | 对得上 |
| `:17` 以多重集合相等为排列条件 | `lemma_permuting_records_preserves_the_walk` gpu_gc_scoring_proofs.rs:188 | 对得上：`requires original.to_multiset() == sorted.to_multiset()` |
| `:18` 每个前缀值在 0 与 `已访问条数 × 65535` 之间、装得进 u64、不假设总分不超过段长 | `lemma_every_walk_prefix_fits_the_gpu_counter` gpu_gc_scoring_proofs.rs:227 | 对得上：`0 <= 和 <= visited * 65535 < u64::MAX`，前提含 `len <= u32::MAX` |
| `:9` 主定理：任意长度不超过 `u32::MAX` 的合法序列，排序后搜索的和 = 原表逐记录求交的和 | `theorem_sorted_search_equals_full_record_walk` gpu_gc_scoring_proofs.rs:202 | 对得上：`ensures` 给出窗口的两个下界性质、`window.0 <= window.1`、两个和相等；前提是多重集相等、有序、合法跨度、`len <= u32::MAX`、`0 <= start < end <= u64::MAX`，与 `:20` 列的前提一致 |

另核：`GPU_GC_PROOFS.md:22` 说二分的中点用不会溢出的形式。证明里是 `below + (above - below) / 2`（`gpu_gc_scoring_proofs.rs:44`），着色器里是同一个写法（`crates/singlefs-background-gpu/src/live_slot_kernel_searching_sorted_records.wgsl:93`）。这一格模型与 WGSL 没有出入；其余 WGSL 的翻译、主机排序与打包，说明文件自己写明是未证边界（`GPU_GC_PROOFS.md:22`）。

### 1.4 头注释与说明文件里指错或说宽的几处（第 1 条的「对不上」）

1. `gpu_gc_proofs.rs:1` 写「20 项核心性质见 src/README.md」。`crates/singlefs-replay-prefix/src/README.md` 里一个 `G\d\d` 都没有（`grep -c 'G[0-9][0-9]' README.md` 得 0），那 20 项的清单在 `GPU_GC_PROOFS.md`。**对不上：指错了文件。**
2. `GPU_GC_PROOFS.md:5` 写 G21 到 G30「这些是产品直接调用的可执行核」，`gpu_gc_capacity_proofs.rs:1` 写「产品调用同一份 exec 核」。十个里 G30 `extended_gpu_chunk_input` 在 `crates/singlefs-background-gpu/src/` 没有调用点（第 2 节）。**对不上（G30）。**
3. `GPU_GC_PROOFS.md` 的表只有 G01 到 G20；G21 到 G30 没有逐条的表与层次栏，也没有给 G21 到 G30 写复跑命令：`GPU_GC_PROOFS.md:24-36` 是打分模块的命令，`:89-115` 的命令只点 `gpu_gc_proofs` 模块、算术测试与后台包的测试，没有 `gpu_gc_capacity_proofs` 模块、`gpu_gc_capacity_matches_independent_reference` 测试、`gpu-gc-G21` 到 `gpu-gc-G30` 与 `gpu-gc-capacity-witness` 的点名项（`grep -n capacity` 在这份文件里只命中 `:5` 一处）。这是缺项，不是说错。

## 2 「由产品直接调用」：`crates/singlefs-background-gpu/src/` 里的调用点

命令：对三份证明文件里每个 `pub fn`，`grep -nw <名字> crates/singlefs-background-gpu/src/*.rs`（输出没有截断；`.wgsl` 不是调用点）。以下行号都在各文件 `#[cfg(test)]` 之前（`memory_limit_and_chunks.rs:912`、`background_gpu.rs:4308`、`two_kernel_reconciliation.rs:182`、`live_slot_kernels.rs:532`、`region_record_kernels.rs:1430`），即生产段。

| 被验函数 | 产品调用点 |
|---|---|
| `reconcile_gpu_entry`（G01） | two_kernel_reconciliation.rs:148（在 `reconcile_the_two_kernel_outcomes`，`:135`；把两份核结果的 `==` 传进去，`step.confirmed`、`step.ask_cpu`、`step.uncertain_rounds` 三个字段都被读，`:152`、`:160`、`:169`） |
| `checked_live_slot_totals`（G14） | live_slot_kernels.rs:376 |
| `checked_gpu_input_bytes`（G15） | memory_limit_and_chunks.rs:105（`input_buffer_bytes`，`:103`）、memory_limit_and_chunks.rs:312、memory_limit_and_chunks.rs:385；导入在 `:29` |
| `checked_gpu_chunk_bytes`（G16） | memory_limit_and_chunks.rs:304 |
| `remaining_gpu_memory`（G17） | memory_limit_and_chunks.rs:331（`with_the_room_left_after`，`:328`） |
| `next_gpu_record_slice_end`（G18） | live_slot_kernels.rs:303（`live_slot_plan_on_a_gpu`，`:246`）、memory_limit_and_chunks.rs:561（`pieces_cut_to_fit_on_the_gpu`，`:530`） |
| `gpu_record_is_in_scan_region`（G19） | region_record_kernels.rs:152（`record_is_one_the_region_scan_reads`，`:148`） |
| `decoded_gpu_live_slots`（G20） | live_slot_kernels.rs:463（`decoded_live_slot_outcomes`，`:447`） |
| `gpu_chunk_fits`（G21） | memory_limit_and_chunks.rs:400（`chunk_fits_with_the_staging_of_its_input`，`:375`）；导入在 `:27` |
| `largest_gpu_piece_input`（G22） | memory_limit_and_chunks.rs:516（`largest_piece_input_bytes`，`:497`） |
| `tightened_gpu_memory_limit`（G23） | memory_limit_and_chunks.rs:346（`with_the_memory_limit_at_most`，`:344`） |
| `checked_gpu_memory_hold`（G24） | memory_limit_and_chunks.rs:693（`hold`，`:687`） |
| `checked_gpu_memory_release`（G25） | memory_limit_and_chunks.rs:709（`release`，`:707`） |
| `gpu_bytes_after_submission_wait`（G26） | memory_limit_and_chunks.rs:723（`settle_the_chunk_after_its_submission`，`:718`） |
| `gpu_memory_cap_after_allocation_failure`（G27） | background_gpu.rs:3801 |
| `gpu_workgroups_for_entries`（G28） | background_gpu.rs:2521、background_gpu.rs:2663、background_gpu.rs:2790 |
| `checked_gpu_mapping_table_bytes`（G29） | region_record_kernels.rs:552（`mapping_table_bytes_of`，`:547`） |
| `extended_gpu_chunk_input`（G30） | **没有调用点**。`grep -rnw extended_gpu_chunk_input crates scripts .claude` 只命中定义（`gpu_gc_capacity_proofs.rs:183`）与 `crates/singlefs-replay-prefix/tests/gpu_gc_capacity_matches_independent_reference.rs:47` 的测试。产品的「追加一条看装不装得下」是 `chunks_that_fit_on_the_gpu` 里自己写的 `checked_add` 再 `filter` 调 G21（memory_limit_and_chunks.rs:434-443），没有经过 G30 |
| `witness_gpu_gc_arithmetic_boundaries`、`witness_gpu_capacity_and_accounting_boundaries` | 没有调用点；它们是见证，本来就不是产品核 |

结论：`gpu_gc_proofs.rs:2` 的话（仲裁、预算、分片、回读、区域准入的 exec 函数由产品直接调用）八个全成立，且 `GPU_GC_PROOFS.md:69` 的「八个产品核」数得对（G01、G14 到 G20）。`GPU_GC_PROOFS.md:5` 与 `gpu_gc_capacity_proofs.rs:1` 把这句话说给 G21 到 G30 全体，**G30 不成立**，另九个成立。

三条衔接上的实情（说明文件没点到的）：

1. 产品调 G21、G22 时交进去的显存上限是产品自己先改过的：把输入的暂存份数先从上限里扣掉（`memory_limit_and_chunks.rs:392-397`、`:504-513`），因为被验函数「只把对齐后的输入算一遍」（`memory_limit_and_chunks.rs:6`）。被验的是改过之后的那组参数；改的那几步是普通 Rust，不在证明里。`GPU_GC_PROOFS.md:5` 只说「并不证明外层资源账本所有调用都符合协议」，没点这一处。
2. `bytes_held_by_a_chunk`（`memory_limit_and_chunks.rs:286`，说明文件把 G16 挂在它名下）算的总数是「被验的 G16」加「暂存份数 × 对齐输入」两段，后一段的乘与加是普通 Rust 的 `checked_*`（`:311-313`）。G16 罩的只是前一段。
3. 产品调 G24 时 `pending` 实参是 `self.submission_running_past_its_wait.is_some()`，而紧前一行 `assert!` 已经保证它是 `None`（`memory_limit_and_chunks.rs:688-696`），所以 G24 的「有未完成提交就拒」那一臂产品走不到，只由 `assert!` 守。

G29 的常量与产品缓冲对得上：`region_record_kernels.rs:103` 是 `MAPPING_ENTRY_WORDS = 1 + KEY_WORDS + 6`（`KEY_WORDS = 7`，`:101`），14 字 × 4 = 56 字节；`:104` 落点项 4 字 = 16 字节；排序索引每条一字 = 4 字节（`:408-413` 空表时各补一项）。核里写死的 56、16、4 与之相等，而且 `crates/singlefs-background-gpu/tests/gpu_gc_capacity_contracts_through_the_host.rs:283` 起拿 `checked_gpu_mapping_table_bytes` 对产品真缓冲的 `bytes_on_the_gpu` 与 `largest_buffer_bytes`（`:284`）。常量在核里是字面值、不引产品常量：产品改字数而没人改核时，只有这个后台包的测试会红，核包的证明与测试都不会。

HEAD 上的情形（只读 git）：`git show HEAD:crates/singlefs-background-gpu/src/<文件> | grep -c singlefs_replay_prefix` 对 `memory_limit_and_chunks`、`two_kernel_reconciliation`、`live_slot_kernels`、`region_record_kernels`、`background_gpu` 五份都是 0（工作区分别是 7、2、6、2、4），HEAD 里 `crates/singlefs-background-gpu/Cargo.toml` 也没有 `replay-prefix`。也就是说这些调用点是已暂存、没提交的后台包改动，不在这一轮被判的十四份里；上面的行号是工作区今天的。

## 3 覆盖写四条引理与一个见证

### 3.1 `requires` 是什么（`crates/singlefs-replay-prefix/src/partial_write_proofs.rs`）

| 引理 | 行 | `requires` | 见证里满足它的那一次调用 |
|---|---|---|---|
| `lemma_overwrite_changes_exactly_the_requested_bytes` | partial_write_proofs.rs:119 | `overwrite_fits(old, start, data)`，即「`0 <= start && start + data.len() <= old.len()`」（`:110`） | `:196`：`old = [1,2,3,4]`、`start = 1`、`data = [8,9]`，1+2 <= 4 成立；`:197` 在末尾（start = 4）的空写 |
| `lemma_overwriting_the_same_range_keeps_only_the_last_write` | partial_write_proofs.rs:134 | `overwrite_fits(old, start, first)`、`first.len() == last.len()` | `:198`：`[5,6]` 与 `[8,9]` |
| `lemma_disjoint_overwrites_commute` | partial_write_proofs.rs:150 | 两个写各自 `overwrite_fits`；`first_start + first.len() <= second_start \|\| second_start + second.len() <= first_start`（`:156`） | `:199`：`[7]` 在 0、`[8,9]` 在 1，0+1 <= 1（相邻） |
| `lemma_whole_unit_overwrite_does_not_depend_on_old_bytes` | partial_write_proofs.rs:168 | 写区间与单元区间非空；`left.len() == right.len()`；`overwrite_fits(left, write_start as int, data)`；`data.len() == write_end - write_start`；`unit_is_covered(...)` | `:200`：`left = old`、`right = [5,6,7,8]`（长度都是 4）、`data = [8,9]`、写 [1,3)、单元 [1,3) |
| `witness_overwrite_laws_and_overlapping_write_order` | partial_write_proofs.rs:187 | 无 | 四个引理的每一条 `requires` 都被它的一次具体调用满足（上表最右列，我逐条核了算术）；另 `ensures` 里证了「重叠的两次写交换后结果不同」（`:192-193`），说明交换律确实要不相交这条前提 |

结论：四条引理的前提合起来可满足，见证罩住了每一条 `requires`（逐个核过，没有哪条前提是矛盾的）。必红表还有三行删掉其中三条前提（`overwrite-last-needs-equal-lengths`、`overwrite-commute-needs-disjoint-ranges`、`overwrite-whole-needs-coverage`）与五行 `ensures false`（第 5 节细说）。

### 3.2 产品写路径满不满足

产品的覆盖写是 `publish_partial_write`（crates/singlefs-core/src/transaction.rs:12548），切法由 `PartialWriteOfAFile`（crates/singlefs-core/src/write_request_split.rs:250 起）定：

- 构造时算终点 `offset + length`，越过 u64 就拒（write_request_split.rs:256；transaction.rs:12583-12594）。
- 写后的 size 是 `max(写前 size, 终点)`（write_request_split.rs:278；transaction.rs:12595）：写可以越过原来的末尾，同时变长。
- 越过末尾或落在洞里的字节补零：`payload.resize(..., 0)`（transaction.rs:12642），注释写「旧内容之外的字节（空隙、洞里、新单元、末尾单元补满的那一段）一律按零」（transaction.rs:12641）。
- 每个重写的单元由 `write.data_unit(unit)`（transaction.rs:12616）得到来源，里面调 `classify_partial_write_unit`（write_request_split.rs:361），单元区间取自 `checked_unit_byte_range`（write_request_split.rs:355，整单元净荷容量，不是声明长度）。
- 字节落到净荷：`payload[a..a+n].copy_from_slice(&bytes.bytes()[b..b+n])`（transaction.rs:12653）。

对前提逐条：

1. **`overwrite_fits`（引理 1、2、3、4 都有）**：产品没有这个前提，也没有保证它：写越过末尾是被明确允许的路径（上面的第二、三点）。产品的一次写满足 `overwrite_fits(文件字节, offset, data)` 当且仅当 `offset + length <= 写前 size`，代码里没有一行判它。所以四条引理罩住的只是「写落在已有 size 之内」那一类写；越过末尾、补洞、变长这一类不在模型里。引理头注释自己说了：「固定长度文件内的写入；扩展文件、补洞与 I/O 不在这个字节模型中」（`crates/singlefs-replay-prefix/src/partial_write_proofs.rs:108` 「固定长度文件内的写入；扩展文件、补洞与 I/O 不在这个字节模型中。」）。
2. **引理 2 的 `first.len() == last.len()`、引理 3 的不相交**：这是「两次写之间」的关系。`publish_partial_write` 一次只处理一个写；两次发布之间的关系由调用方决定，产品写路径不检查也不保证。它们是模型上的定律，不是产品要满足的前提。
3. **引理 4 的 `write_start < write_end`**：成立，长度是 `NonZeroU64`（write_request_split.rs:242 的字段 `length_in_bytes`），终点 = 起点 + 长度。**`unit_start < unit_end`**：成立，容量为正。**`data.len() == write_end - write_start`**：成立，终点按长度算出。**`unit_is_covered`**：产品喂给 `classify` 的就是同一对区间与同一个谓词（write_request_split.rs:361-366），返回 `OldCoveredWhole`、`NewCoveredWhole` 时即满足。**`left.len() == right.len()`** 是模型里的量词，产品没有对应物。
4. 一处保守：产品传的单元区间是满容量（`unit_start + 净荷容量`），而文件最后一个单元的声明长度可能不满。写盖到文件尾、没盖到满容量单元的末端时，`classify` 判「没盖满」去读旧单元，多读不少读，不违反引理。

模型与产品之间的缺口（`partial_write_proofs.rs:2` 与 README 都已写明）：产品是逐单元的 `Vec<u8>` 净荷上的 `copy_from_slice`，引理说的是整个文件的 `Seq<u8>`；两者的对应要靠 `checked_bytes_in_indexed_unit`（file_range_proofs.rs 里另一个已验的核）加上拷贝本身，这几步合起来没有证明。

### 3.3 `src/README.md` 新加那一节（`crates/singlefs-replay-prefix/src/README.md:150-174`）对证明

| 节里说的 | 证明里的 | 判定 |
|---|---|---|
| `:152` `overwrite_fits` 要求起点非负、整个请求位于文件内 | partial_write_proofs.rs:110 | 对得上 |
| `:154` 长度不变，写内字节来自请求，写外保留旧值，空请求不改变文件 | 引理 1 的三条 `ensures`（partial_write_proofs.rs:122-126） | 对得上 |
| `:155` 同起点同长度的连续覆盖只保留最后一次 | 引理 2（`:141-143`） | 对得上 |
| `:156` 不相交的两次覆盖可交换，两种顺序下第二次写仍满足范围前提，允许相邻与空请求 | 引理 3（`:158-161`；`:156` 的 `<=` 允许相邻） | 对得上。小注：空写落在另一个写的内部时（如起点 2 长 0、另一个写 [1,4)）两个析取都不成立，前提没罩住，引理保守 |
| `:157` 分类核的覆盖条件足以使单元结果不依赖旧内容 | 引理 4（`:178-181`） | 对得上 |
| `:158` 见证给出满足各前提的非空调用、EOF 处的空请求、重叠写交换后结果不同 | 见证（`:189-193`、`:197`、`:199-200`） | 对得上 |
| `:160` 不代表 core 的缓冲区拷贝、扩展文件、补洞、设备 I/O 或持久化顺序已精化到此模型 | 与上面 3.2 一致 | 对得上，这句是对的 |
| `:160` 必红表新增五条契约反例及三条删除关键前提的反例（同长度、不相交、整单元覆盖） | 表里 `overwrite-contract-*` 五行（`verus-must-red-worlds.tsv:328-332`）、`overwrite-last-needs-equal-lengths` 等三行（`:333-335`） | 条数对得上；「契约反例」五行的改法是往 `ensures` 开头插一条 `false`（见第 5 节），证明的是「这个函数被验了、前提不矛盾」，不是「改坏某一条 `ensures` 会红」 |
| `:168-173` 复跑命令点三行必红项与 `partial_write_proofs::tests` | 三个项名在表里（`:333-335`）；`--lib partial_write_proofs::tests` 的测试在 partial_write_proofs.rs:240 起 | 项名对得上；这些命令没带范围声明、带了 `--manual`（第 8 节） |

另一处小疑：README 第 150 行的节标题在 `:150`，而 `:146` 一段（旧）写「本模块新增 6 个被验函数……与 11 个必红变异」，与新节加了 5 项后的现状没有对齐（那一段不是这一轮加的行，只记一笔，不算这一轮的缺陷）。

## 4 改名那一批：逐块核是不是纯改名

方法：我写了 `tokdiff.py`（草稿目录），把 HEAD 版（`git show HEAD:<文件>`）与工作区版各切成记号（标识符、数字、单个符号，空白不算），用 `difflib.SequenceMatcher` 比两串记号，把每一处不同原样打出来。因此 `requires`、`ensures`、`decreases`、`==>`、括号、数字一个记号不同都会现形。再用 `grep -ow` 数每个旧名、新名在两版里的出现次数，看有没有撞名、有没有漏改。

| 文件 | 记号级不同处 | 旧名新名出现次数 | `git diff HEAD -U0` 的 hunk 数 | 判定 |
|---|---|---|---|---|
| compaction_plan_proofs.rs | 2 处，都是函数名换名：`a_segment_holding_a_pinned_slot_is_not_a_sound_choice`→去 `a_`；测试 `a_batch_takes_the_first_movable_units_up_to_the_batch_size`→去 `a_` | 新名各 1 处定义；旧名在 `crates`、`.claude`、`scripts` 里不再出现，只剩必红表 `verus-must-red-worlds.tsv:198` 的行名（第 5 节） | 2 | 纯改名 |
| file_range_proofs.rs | 9 处：`_max_units`→`_maximum_units`（2 处）；`(a, b, n)` 模式变量→`(offset_in_unit, offset_in_write, length)` 与其一处使用 `a`→`offset_in_unit`；测试闭包 `i`→`offset_in_overlap`（2 处），外加闭包体多了一对花括号（`file_range_proofs.rs:533-535` 一带，`mod tests` 里，rustfmt 的排版） | `a`、`b`、`n`、`i` 在新版 0 次；`offset_in_unit`/`offset_in_write` 旧版本来就有 1 次（`:268`，另一个 `if let` 作用域），新绑定在 `:295` 的 `assert ... by { }` 里，不在同一个作用域，没有撞名 | 3 | 纯改名（闭包花括号是测试里的排版，不是证明内容） |
| lib.rs | 13 处：1 处插入（三行 `pub mod gpu_gc_*`，`lib.rs:36-38`），12 处标识符换名：`a_later_record_of_the_same_publish_is_readable` 6 处（定义 + 5 处引用，含两处在 `ensures`/`implies` 里），6 个证明函数或测试去 `a_` | 九个新名在 HEAD 里都没有、现在各 1 处定义；旧名全仓 0 命中 | 13 | 纯改名 + 三行模块声明。三行模块声明不是改名，是新模块接进来 |
| unlink_proofs.rs | 113 处，全是标识符换名：`i`→`step_index`（71）、`j`→`other_step_index`（11）、`k`→`tail_step_index`（7）、`q`→`quotient`（18）、`r`→`remainder`（6），含 `forall\|step_index: int\|`、`exists\|…\|`、`let quotient = kept / span;`、文档注释里的「第 i 批」 | 旧名 71/11/7/18/6 次在新版全变 0，新名在 HEAD 里 0 次、新版恰是 71/11/7/18/6 次：一一对应，没有漏改，没有撞名 | 25 | 纯改名 |
| partial_write_proofs.rs 的形参 | 144 处里，改名部分全是 `ws`→`write_start`、`we`→`write_end`、`us`→`unit_start`、`ue`→`unit_end`，涵盖 `unit_is_touched`、`unit_is_covered`、`classify_partial_write_unit`、两条引理的签名、`requires`、`ensures`、函数体和测试的循环变量；另有两处不是改名：头注释第 2 行改写，测试里一个调用的参数列表多了尾逗号（排版）；其余是新加的 97 行覆盖写段（第 3 节） | `ws/we/us/ue` 新版 0 次（HEAD 34/30/39/35）；`write_start`/`unit_start`/`unit_end` HEAD 0 次；`write_end` HEAD 里本来有 6 次，都在 `units_after_partial_write`（HEAD 的 `:122-131`），那个函数不在被改名的函数里，没有撞名 | 19（含新加段） | 形参改名部分是纯改名 |

核过的块数：按 `git diff HEAD -U0` 的 hunk 计，compaction_plan_proofs.rs 2、file_range_proofs.rs 3、lib.rs 13、unlink_proofs.rs 25、partial_write_proofs.rs 19（其中改名的 hunk 与新加段在同一批 hunk 里分不开，改名部分按记号核）。按事务员的块（facts.md 的「逐块」）计：2、3、13（其中 12 块纯改名、1 块新增）、8、5（其中 1 块是新增的 115 行）。**`requires`、`ensures`、`decreases` 的结构：被改名的块里一处都没动**（记号级核过：这三个关键字与它们所带的表达式，除标识符换名外一个记号不差）。

**不是纯改名的块：**

1. lib.rs 的三行 `pub mod gpu_gc_capacity_proofs;`、`pub mod gpu_gc_proofs;`、`pub mod gpu_gc_scoring_proofs;`（新模块接入，不是改名）。
2. partial_write_proofs.rs 新加的覆盖写一段与见证（第 3 节）与头注释第 2 行的改写。
3. 排版性的两处（file_range_proofs.rs 的测试闭包花括号、partial_write_proofs.rs 的测试调用尾逗号）：不改语义，但严格说不是「换名」。

改名遗留：`crates/singlefs-replay-prefix/verus-must-red-worlds.tsv:198` 的行名仍写 `证明负例·a_segment_holding_a_pinned_slot_is_not_a_sound_choice 的函数体不能推出 false`，函数已经叫 `segment_holding_a_pinned_slot_is_not_a_sound_choice`，只有第六段落点跟着改了。行名是 `--item must-red:<行名>` 的项名，不影响判，但是一个指向已不存在的函数的名字（`facts.md` 说这一行「名去 a_」，不对，见第 9 节）。

## 5 必红表新加与改动的行

### 5.1 锚点命中次数

命令（草稿目录的 `anchors.py`，对每一行把第三段的 `\n` 还原成换行，在第二段那份源文件里数不重叠的出现次数，并给首次命中的行号）：`python3 -I anchors.py changed`。多行原文 grep 做不了，所以用 `str.count`；单行原文另用 `grep -cF -- <原文> <文件>` 逐行复核（`grepcheck.py`）。对象是相对 HEAD 新出现或改过的行：66 行（58 行新加 + 8 行改名）。

- 66 行全部 `count=1`；单行原文 46 行用 `grep -cF` 复核，全是 1 行；多行原文 20 行只由 `str.count` 判。
- `grepcheck.py` 末行原样：`rows 66 single-line 46 multi-line 20 grep-not-1 []`（「grep-not-1」是命中行数不为 1 的行，空表示没有）。
- 66 行的第五段都是 `proof_errors=<正整数>`、第六段都是 `landing=`；落点里点名的每个函数名，在该行点名的那份源文件里都恰好有 1 个 `fn` 定义（`landings.py`），只有第 362 行的 `lemma_search_prefix_matches_the_walk` 例外：它在 `gpu_gc_scoring_proofs.rs`，而这一行改的是 `gpu_gc_proofs.rs` 里的 `record_contribution`。这不是错：检查脚本按每条证明错报在哪份文件找函数（`scripts/gate/verus-proof-check.py:3527` 的 `proof_error_landing`），跨文件的落点合法。

逐行（表行号 / 行名 / 命中次数 / 源文件首次命中行）：

```
26 ensures·冒烟：两条记录的整次发布改成断言不是拼接 count=1 @4125
27 ensures·冒烟：一条不带末条标志的记录改成断言是拼接 count=1 @4138
29 ensures·readable_record_of_the_same_publish_follows 交回的判定取反 count=1 @3130
198 证明负例·a_segment_holding_a_pinned_slot_is_not_a_sound_choice 的函数体不能推出 false count=1 @1445
305 partial-touching-endpoints-are-not-written count=1 @64
306 partial-equal-range-covers-whole-unit count=1 @65
312 partial-whole-coverage-needs-nonempty-unit count=1 @78
317 partial-touch-needs-nonempty-write count=1 @96
328 overwrite-contract-lemma_overwrite_changes_exactly_the_requested_bytes count=1 @119
329 overwrite-contract-lemma_overwriting_the_same_range_keeps_only_the_last_write count=1 @134
330 overwrite-contract-lemma_disjoint_overwrites_commute count=1 @150
331 overwrite-contract-lemma_whole_unit_overwrite_does_not_depend_on_old_bytes count=1 @168
332 overwrite-contract-witness_overwrite_laws_and_overlapping_write_order count=1 @187
333 overwrite-last-needs-equal-lengths count=1 @139
334 overwrite-commute-needs-disjoint-ranges count=1 @156
335 overwrite-whole-needs-coverage count=1 @176
338 gpu-gc-G01 count=1 @42
339 gpu-gc-G02 count=1 @49
340 gpu-gc-G03 count=1 @59
341 gpu-gc-G04 count=1 @85
342 gpu-gc-G05 count=1 @94
343 gpu-gc-G06 count=1 @117
344 gpu-gc-G07 count=1 @136
345 gpu-gc-G08 count=1 @145
346 gpu-gc-G09 count=1 @178
347 gpu-gc-G10 count=1 @185
348 gpu-gc-G11 count=1 @196
349 gpu-gc-G12 count=1 @210
350 gpu-gc-G13 count=1 @224
351 gpu-gc-G14 count=1 @243
352 gpu-gc-G15 count=1 @263
353 gpu-gc-G16 count=1 @279
354 gpu-gc-G17 count=1 @293
355 gpu-gc-G18 count=1 @312
356 gpu-gc-G19 count=1 @321
357 gpu-gc-G20 count=1 @333
358 gpu-gc-witness_gpu_gc_protocol_and_score_premises count=1 @337
359 gpu-gc-witness_gpu_gc_arithmetic_boundaries count=1 @355
362 gpu-gc-audit-score-capped-at-two count=1 @159
363 gpu-gc-audit-successful-cpu-call-lost count=1 @78
366 gpu-gc-G21 count=1 @40
367 gpu-gc-G22 count=1 @75
368 gpu-gc-G23 count=1 @86
369 gpu-gc-G24 count=1 @99
370 gpu-gc-G25 count=1 @116
371 gpu-gc-G26 count=1 @128
372 gpu-gc-G27 count=1 @138
373 gpu-gc-G28 count=1 @152
374 gpu-gc-G29 count=1 @174
375 gpu-gc-G30 count=1 @192
376 gpu-gc-capacity-witness count=1 @198
379 gpu-score-binary-skips-entry count=1 @55
380 gpu-score-binary-drops-upper count=1 @65
381 gpu-score-binary-needs-sorted-records count=1 @34
382 gpu-score-slot-loop-shifted count=1 @77
383 gpu-score-slot-count-induction-starts-one-slot-late count=1 @88
384 gpu-score-window-too-narrow count=1 @93
385 gpu-score-search-counts-released count=1 @144
386 gpu-score-search-skips-record count=1 @143
387 gpu-score-candidate-needs-valid-spans count=1 @105
388 gpu-score-prefix-skips-induction-step count=1 @165
389 gpu-score-fold-skips-induction-step count=1 @182
390 gpu-score-permutation-needs-multiplicity count=1 @189
391 gpu-score-theorem-needs-permutation count=1 @206
392 gpu-score-counter-needs-record-bound count=1 @228
393 gpu-score-witness-is-checked count=1 @240
```

### 5.2 改坏之后红在哪条 `ensures`（读代码推的，没起 Verus）

「红在哪」只指我从改法推出会不满足的那一条；检查脚本判的是落点（函数名），不判到 `ensures` 条。

| 表行 | 改法 | 红在哪（推的） |
|---|---|---|
| 328-332 | 在 `ensures` 后头第一条之前插 `false`（覆盖写四条引理与见证各一行） | 红的是插进去的 `false`，不对应任何原有 `ensures`。这五行的作用是「该函数确被验、`requires` 不矛盾」：`requires` 若互相矛盾，`ensures false` 也能证，这一行就红不了。五行**不**证明原来的 `ensures` 会被抓 |
| 333 | 删引理 2 的 `first.len() == last.len()` | 引理 2 两条 `ensures`：`overwrite_fits(结果, start, last)`（`last` 可更长）与「先后两次等于只写最后一次」 |
| 334 | 删引理 3 的不相交 | 引理 3 的 `fits` 两条与交换等式 |
| 335 | 删引理 4 的 `unit_is_covered` | 引理 4 的「单元字节在文件内且两版相等」 |
| 338 G01 | 暂缓那一支改成 `confirmed: true` | `confirmed == agree`；「既不确认也不交 CPU 当且仅当分歧且 previous == 0」 |
| 339 G02 | 第一条 `ensures` 改成断言 `cpu_is_needed(0, false)` | 被改的那一条 |
| 340 G03 | `next_uncertain_count(previous, true) == 1` | 被改的那一条 |
| 341 G04 | 失败轮的 `cpu_calls + 1` | 第一条（失败轮状态不变） |
| 342 G05 | 删 `first != second` | 次序可换那一条，以及随后三条（取 `first == second` 时） |
| 343 G06 | 删「每个值都是 1」 | 第一条（更新后每个值仍是 1：别的键的旧值若不是 1 就保留不是 1） |
| 344 G07 | 上界改成 `左跨度 - 1` | 第一条的上半 |
| 345 G08 | 去掉 `seam` 的上下界 | 唯一一条等式（`seam` 在段外时左右之和不等于原段） |
| 346 G09 | 递归调用的 `past - 1` 改 `past - 2` | 递归调用处的 `requires`（`seam <= past - 2` 不成立）与终止，随后唯一的等式 |
| 347 G10 | 去掉 `2 *` | 唯一一条（单份贡献非零时） |
| 348 G11 | 「贡献 > 0 当且仅当…」里去掉 `record.device == device` | 第二条（异盘有交集时左真右 0） |
| 349 G12 | `ah < bh` 改 `ah > bh` | 第二条 |
| 350 G13 | `>= 4294967296` 改 `> 4294967296` | 第四条（和恰为 2^32 时） |
| 351 G14 | 第二个 `checked_add` 的加数写成 `next_walking` | `Some` 臂的 `searching_total == searching + next_searching` |
| 352 G15 | `checked_mul(1)` | `Some` 臂的 `bytes == aligned_input_size`（及后面几条） |
| 353 G16 | 不加 `fixed` | `Some` 臂的等式 |
| 354 G17 | `limit - held` 改 `limit` | 第一条与第三条（`held + remaining == limit`） |
| 355 G18 | `start + capacity - 1` | `end == ...` 那一条；`capacity == 1` 时还有「有进展」那一条 |
| 356 G19 | 溢出臂 `None => false` 改 `true` | 唯一一条（`slot + span` 越界时应为 false） |
| 357 G20 | `verdict != 1` 改 `false` | `Some` 臂的 `verdict == 1` |
| 358、359 | 见证里注入 `false` / `assert(false)` | 注入的断言；证明见证函数在被验、没有走在矛盾上 |
| 362 | `record_contribution` 里每条贡献截顶为 2 | G11 第三条（同盘活记录贡献恰为交集长度）与打分模块 `lemma_search_prefix_matches_the_walk`（落点两个函数，`proof_errors=2`） |
| 363 | 成功轮丢掉 CPU 调用的 `+1` | G04 第三条（成功轮精确加一） |
| 366 G21 | 把 `entries > dispatch_entries` 的判断换成 `false` | `fits == chunk_fits(...)`（条数超派发上限时） |
| 367 G22 | `bounded / 4 * 4` 小四字节 | 最大性那一条（`forall input ... ==> input <= bytes`） |
| 368 G23 | 有限额也取新上限 | `limit == match ...` |
| 369 G24 | `if pending { return None }` 换 `if false` | `Some` 臂的 `!pending` |
| 370 G25 | `checked_sub(0)` | `Some` 臂的 `left + bytes == held`（`bytes > 0` 时） |
| 371 G26 | 没完成时返回 `held - chunk` | `Some` 臂的 `left == if completed ... else held` |
| 372 G27 | `/ 2` 改 `/ 1` | 第一条 `2*next <= failed` 与第二条（严格下降） |
| 373 G28 | `wide as u32` 换成 `entries / per_group`（下取整） | 第二条（非零条数得 0 个组）与第三条 `entries <= groups * per_group` |
| 374 G29 | `checked_mul(4)` 改 `checked_mul(3)` | `Some` 臂的总量等式 |
| 375 G30 | 判 `additional` 而不是 `total` | `Some` 臂的 `chunk_fits(total, ...)` |
| 376 | 见证里注入 `assert(false)` | 注入的断言 |
| 379、380 | 二分递归的下界 `middle + 1` 改 `middle + 2`、上界 `middle` 改 `middle - 1` | 递归调用处的前缀、后缀不变量（`requires`），随后 `is_lower_bound` |
| 381 | 删 `records_are_sorted` | 函数体里用有序的两个 `assert` |
| 382、383 | 逐槽判定偏一槽、归纳起点偏一槽 | `lemma_counting_slots_equals_interval_intersection` 的等式 |
| 384 | 饱和减的 65535 改 65533 | `lemma_candidate_window_keeps_every_contribution` 第三条（窗外贡献为零） |
| 385、386、388、389 | 搜索不跳已释放、`past - 2`、归纳递归写错一档 | `lemma_search_prefix_matches_the_walk` 或 `lemma_record_sum_is_a_fold` 的等式 |
| 387 | 删 `valid_scoring_records` | `lemma_candidate_window...` 第三条 |
| 390、391 | 排列条件换成集合相等、删多重集相等 | `lemma_permuting...`、主定理的等式（`proof_errors=2` 落两个函数） |
| 392 | 删 `<= u32::MAX` | `lemma_every_walk_prefix_fits_the_gpu_counter` 的 `visited * 65535 < u64::MAX` |
| 393 | 见证里注入 `assert(false)` | 注入的断言 |
| 26、27、29、198 | lib.rs 与 compaction 的四行，只是落点与锚点随改名更新 | 与改名前相同的那几条（不是这一轮新写的判据） |
| 305、306、312、317 | 部分写四行，只是形参改名 | `classify_partial_write_unit` 的 `touched`、`whole` 判据，两条引理的 `requires` |

### 5.3 没有反例罩着的

G01 到 G30、覆盖写四条引理与见证，每一个都至少有一行（没有哪个函数整个没行）。下面列的是「函数有行，但这一行没碰到的 `ensures` 条或函数体支」，读法是每行改一处，所以每个多条 `ensures` 的函数都有没碰到的条（推的）：

1. **覆盖写**：引理 1 的三条 `ensures` 没有任何改坏的行（只有插 `false` 的一行）。而且引理 1 的 `ensures` 与 `overwritten_bytes` 的定义（partial_write_proofs.rs:113-116）同形，是定义的复述：改坏 `overwritten_bytes` 的定义（例如 `start <= byte < start + data.len()` 改成 `<=`）引理 1 照样过，只会让见证里的具体值红，而表里**没有**这样一行。引理 2、3、4 只有删前提的一行与插 `false` 的一行，没有改 `overwritten_bytes` 或引理体的行。
2. **G01**：`ask_cpu == cpu_is_needed`、两者互斥、`uncertain_rounds == next_uncertain_count`、`<= 1` 四条没碰到；只有暂缓一支被改过，确认、交 CPU 两支没有。
3. **G02、G03**：各只改第一条，其余三条、两条没碰到。**G04**：失败轮碰了、成功轮 CPU 调用靠审计行（363）碰了，成功轮 `.0 == update_count` 一条没碰到。
4. **G05**：只删前提；四条 `ensures` 没有哪条被单独改坏。**G06**：第二、三条（别的键不变、本键新值）没碰到。**G07**：第二、三、四条没碰到。**G08、G09、G10**：各一条 `ensures`，都碰到了。**G11**：第一条（已释放或异盘为零）没碰到（第三条被 362 碰到，第二条被 348 碰到）。**G12**：第一、三条没碰到。**G13**：前三条没碰到。
5. **G14、G15、G16**：只碰了 `Some` 臂；`None` 臂（溢出拒绝）三个函数都没碰到。**G17**：第二条（`remaining <= limit`）没碰到。**G18**：六条里只碰到「`end` 的值」那一条（`capacity == 1` 时可能连带「有进展」），`start <= end <= total`、`end - start <= capacity`、两条区间分划没碰到。**G19** 只有一条 `ensures`，碰到了。**G20**：`Some` 臂碰到了，`None => verdict != 1` 一臂没碰到。
6. **G21**：四道界里只碰了派发那一道，绑定的两道与显存限额那一道没碰到。**G22**：最大性碰到了；`None` 臂、`b >= 4`、四的倍数、`chunk_fits(b, 1)`（`b` 偏大）没碰到。**G23**：第二条 `limit <= available` 没单独碰。**G24**：「超限额」「溢出」两条拒绝与 `Some` 的等式没碰到，碰到的只有 `pending`。**G25**：`None` 臂没碰到。**G26**：「完成则 `held - chunk`」与 `None` 臂没碰到。**G27**：第三条（零仍为零）没碰到。**G28**：`groups <= entries`、`(groups-1) * per_group < entries` 没碰到，**也没有任何一行把 `wide as u32` 改成会丢位的写法**（373 把整个转换换成了下取整，不是丢位）。**G29**：56、16 两个常量、`largest` 臂、空表占位、`None` 臂没碰到。**G30**：「和溢出就拒绝」那一支没碰到。
7. **三条常量**（`GPU_UNCERTAIN_ROUNDS_BEFORE_CPU`、`GPU_INPUT_ALIGNMENT`、`GPU_LIVE_SLOTS_COUNTED`，gpu_gc_proofs.rs:9-11）：没有一行改它们的值（推：改了会让 G01、G15、G20 红，但没登记）。
8. **入口空真筛查**不在表里：`GPU_GC_SELFTEST.md:26` 说「8 处入口 `requires` 空真筛查」跑了一次，不是表里的行；持续罩住 `requires` 可满足的，是见证（gpu_gc_proofs.rs:337 与 :355、gpu_gc_capacity_proofs.rs:198、gpu_gc_scoring_proofs.rs:240）加上门禁自己按 `requires` 逐处列的 `vacuity:` 项（`.claude/gate.d/formal-verification.sh` 文件头 `--list-items` 一段）。

## 6 两份新测试：参照与被测是不是各写各的

先说共性：两份测试都只 `use singlefs_replay_prefix::...::*` 取被测的 exec 函数（`gpu_gc_arithmetic_matches_wide_integer_reference.rs:1`、`gpu_gc_capacity_matches_independent_reference.rs:1`）；参照式全写在测试文件里，用 `u128` 加宽、标准库的 `div_ceil`、`saturating_sub`、`checked_*`、区间 `contains`、`zip`、`(1..=binding).rfind` 穷举，没有调被测 crate 里任何函数（被验函数里的 spec 函数在普通构建里被擦掉，也调不到）。被测函数体用的是 `units_for_sequential_write`、`checked_mul` 一类另一条算法路径。所以「共用代码」这一类对拍两边同错：没有。

可能同错的是**共用同一个决定**（两边由同一个人按同一份理解写）：空输入占 4 字节（产品文档 `memory_limit_and_chunks.rs:100` 也这样写，wgpu 不许绑定长度为 0 的缓冲）、`slot + span <= end`、零跨度允许等，这是产品语义，不是缺陷。

### 6.1 `gpu_gc_arithmetic_matches_wide_integer_reference.rs`（4 条）

| 测试 | 行 | 被测 | 参照 | 各写各的？ |
|---|---|---|---|---|
| `gpu_budget_arithmetic_matches_a_u128_reference_at_full_width` | :16 | G15、G16、G17 | `max(4).div_ceil(4) * 4` 的 u128、`u128 乘加链 .checked_add .and_then try_from`、`saturating_sub` | 是 |
| `gpu_slices_scans_and_independent_totals_match_wide_arithmetic` | :46 | G18、G19、G14 | `min(total, start + capacity)` 的 u128、`(start..end).contains(&slot)` 加 u128 和、`u128 和 try_from … zip` | 是 |
| `gpu_readback_rejects_unwritten_words_and_preserves_both_halves` | :103 | G20 | 先 `value.to_le_bytes()` 拆高低两字，再 `(verdict == 1).then_some(value)` | 是（被测用 `low as u64 + 2^32 * high as u64` 拼，参照用字节切分，方向相反） |
| `gpu_reconciliation_matches_the_two_round_history_rule` | :118 | G01 | 4096 条十二轮历史，用一个布尔 `previous_disagreement` 当状态，不用计数器；计数器取自被测上一步的输出 | 是（参照的状态是布尔，被测的是整数；计数器在两轮之间由被测输出接力，所以参照没有独立的计数器，但每步都断言 `uncertain_rounds == u32::from(previous_disagreement)`） |

边界值：`EDGES` 九个（`:3-13`）：0、1、3、4、5、`u32::MAX`、2^32、`u64::MAX - 3`、`u64::MAX`。我的判断：

- 抓全宽回绕、u32 截断、空输入、对齐进位、`u64::MAX` 溢出，够了：参照是 u128，被测凡回绕、凡截断都与它不等。
- **不够的两处**（推的，没跑变异）：九个值对 4 取余只有 0、1、3，**没有余 2 的值**（2、6、…）；对齐溢出的分界是 `u64::MAX - 3`（能装）与 `u64::MAX - 2`（装不下），`EDGES` 里只有前者与 `u64::MAX`，**没有一个值正落在分界外一格**。要抓的是形如「用 `+3` 做向上取整再回绕」的写法，`u64::MAX` 已经抓得到，所以风险小。G16 的「和恰等于 `u64::MAX`」在 `EDGES` 的组合里取得到（`edgecheck.py` 数出 19 组，例如对齐输入取 `u64::MAX - 3`、条数 0、固定占用 3），所以「恰好装满」与「差一溢出」这个边界这条测试是碰到的。
- G19 的 `slot + span == end` 恰等边界在 `EDGES` 里取得到（如 `slot = 1`、`span = 3`、`end = 4`），G18 的 `total - start == capacity` 也取得到。

### 6.2 `gpu_gc_capacity_matches_independent_reference.rs`（5 条）

| 测试 | 行 | 被测 | 参照 | 各写各的？ |
|---|---|---|---|---|
| `chunk_admission_and_extension_match_full_width_arithmetic` | :23 | G21、G30 | `reference_fits`（`:5`，u128，四道界逐条写）；G30 的参照是 `u64::try_from(input + additional).ok().filter(reference_fits)` | 是 |
| `largest_piece_matches_exhaustive_search_and_is_maximal` | :69 | G22 | `(1..=binding).rfind(reference_fits)` 穷举，`binding` 取 0 到 40，五个固定占用 × 五个每条占用 × 三个条缓冲 × 两个派发上限 × 七个限额，共 43,050 组 | 是（闭式对穷举），最强的一条 |
| `holds_releases_and_waits_preserve_the_exact_ledger` | :118 | G24、G25、G26 | u128 和、`pending \|\| 超限额` 判 `None`；另外断言 `release(hold(...)) == held`（往返），`:136-141` 的 `for _ in 0..3` 重复调用纯函数，没有增加覆盖 | 是 |
| `memory_caps_never_expand_and_allocation_failure_makes_progress` | :165 | G23、G27 | G23：`previous.unwrap_or(u64::MAX).min(available)`；G27：`u128::from(cap) / 2` | G23 是；**G27 的参照与被测是同一个式子 `cap / 2`**（只是加宽了），独立多断言的是「严格下降」与「64 轮内到 0」 |
| `mapping_buffers_and_dispatch_cover_their_exact_full_width_sizes` | :186 | G29、G28 | G29：`56 * max(e,1)`、`16 * max(l,1)`、`4 * max(e,1)`，最大取三者的最大；G28：`u64::from(entries).div_ceil(u64::from(per_group))` | G28 是；**G29 的 56、16、4 是核里同一组字面值**，参照把排序索引也放进最大值里（被测与规格只比两张表），这是一处额外的独立核对；产品真缓冲对这些常量的核对在 `crates/singlefs-background-gpu/tests/gpu_gc_capacity_contracts_through_the_host.rs:283` |

边界值：`EDGES` 十个（`:3`）：0、1、3、4、5、16、63、2^32、`u64::MAX - 3`、`u64::MAX`。

- **缺口一**（`edgecheck.py` 逐组枚举核过：限额取 100、`u64::MAX`、0 时，`对齐输入 + 40 × 条数 + 16 == 限额` 的 `(输入, 条数)` 组合数是 0）：`chunk_admission_...` 里 `fixed_bytes` 固定 16、`bytes_per_entry` 固定 40，限额取 `None`、0、100、`u64::MAX`。对齐输入、条数取 `EDGES` 时，「对齐输入 + 40 × 条数 + 16」没有一组恰好等于 100（需要对齐输入 = 84 − 40 × 条数，`EDGES` 里没有这样的值），所以 `gpu_chunk_fits` 里显存限额那一个 `<=` 若被写成 `<`，这一条测试抓不到；绑定上限（输入 4、条 × 每条 4 对绑定 4）与派发上限（条数 3 对 3）的 `<=` 恰等边界取得到。限额恰等的边界由 Verus 的规格（G21 的 `ensures fits == chunk_fits`）与见证（`gpu_gc_capacity_proofs.rs:202-203`，44 字节恰装满、45 字节装不下）管，不由这份测试管。`largest_piece_...` 那条用限额 20 配每条占用 4 等，限额恰等取得到，但它测的是 G22（另一条路径），不是 G21。
- **缺口二**：`wide as u32`（`gpu_gc_capacity_proofs.rs:152`）的丢位这一格，`mapping_buffers_...` 里的 `entries = u32::MAX`、`per_group = 1` 直接穿过转换（得 `u32::MAX`），测试对它是实打实的；但必红表没有改这个转换的行（第 5 节）。
- 条目数 `[0, 1, 63, 64, 65, u32::MAX - 1, u32::MAX]` 里 63、64、65 夹住了产品的每组 64 个调用（`checksum_kernels.rs:37` 等三处常量都是 64）：够。
- **缺口三：两份测试没有变异行罩着。** `grep -c 'gpu_gc_arithmetic_matches\|gpu_gc_capacity_matches\|gpu_gc_proofs.rs\|gpu_gc_capacity_proofs.rs\|gpu_gc_scoring_proofs.rs' crates/mutations.tsv` 得 0。必红表（`verus-must-red-worlds.tsv`）是 Verus 的，不跑 `cargo test`；所以「往被测函数里塞已知错，这两份测试会红」没有一处记录在案，测试会不会红只是读代码推的。这与正文「三份新证明文件的反例都在必红表里」不矛盾，但必红表罩的是证明，不是这两份测试。

## 7 三份说明文件里的数，逐个现数

现数命令与读数（都是量的，被数的是工作区今天的文件）：

| 要数的 | 命令 | 读数 |
|---|---|---|
| `src/` 下 `.rs` 份数 | `ls crates/singlefs-replay-prefix/src/*.rs \| wc -l` | 10 |
| `lib.rs` 里 `pub mod` 行数 | `grep -c '^pub mod' crates/singlefs-replay-prefix/src/lib.rs` | 9（`lib.rs:32-40`） |
| README 表的行数 | `grep -c '^\| \[' crates/singlefs-replay-prefix/src/README.md` | 8 |
| `gpu_gc_proofs.rs` 里 `requires` 行数 | `grep -c 'requires' crates/singlefs-replay-prefix/src/gpu_gc_proofs.rs` | 8 |
| `gpu_gc_proofs.rs` 里 `pub fn` 数（exec） | `grep -c '^pub fn' crates/singlefs-replay-prefix/src/gpu_gc_proofs.rs` | 9（八个产品核加一个见证） |
| 两份新测试的 `#[test]` | `grep -c '^#\[test\]' crates/singlefs-replay-prefix/tests/*.rs` | 4、5 |
| 必红表行名前缀 | `grep -c "^<前缀>" crates/singlefs-replay-prefix/verus-must-red-worlds.tsv` | `gpu-gc-` 35（其中 `gpu-gc-G` 30、`gpu-gc-witness` 2、`gpu-gc-audit` 2、`gpu-gc-capacity` 1）、`gpu-score-` 15、`overwrite-` 8（其中 `overwrite-contract` 5）、`partial-` 12 |
| 登记的义务数 | `sed -n 48p .claude/gate.d/verus-pins.tsv` | `crate	crates/singlefs-replay-prefix	208` |

逐项：

| 文件:行 | 写的数 | 实数 | 判定 |
|---|---|---|---|
| crates/singlefs-replay-prefix/src/README.md:7 | 「八份源文件」 | 10 份；表 8 行，缺 `gpu_gc_capacity_proofs.rs`、`gpu_gc_scoring_proofs.rs` | **对不上** |
| crates/singlefs-replay-prefix/src/README.md:20 | 「另七份文件通过模块引入」 | `lib.rs` 引了 9 份 | **对不上**（应为另九份） |
| crates/singlefs-replay-prefix/src/README.md:15 | 「八个纯函数入口」「20 项核心性质」 | 8 个产品核；G01 到 G20 共 20 项 | 对得上（G21 到 G30 属另一份文件） |
| crates/singlefs-replay-prefix/src/README.md:160 | 五条契约反例、三条删前提反例 | 5、3 | 对得上 |
| GPU_GC_PROOFS.md:3 | G01 到 G20 共 20 | 20 | 对得上 |
| GPU_GC_PROOFS.md:5 | G21 到 G30 共 10 个名目 | 10（与代码 `/// G21` 到 `/// G30` 次序一致） | 对得上 |
| GPU_GC_PROOFS.md:69 | 「调用表内八个产品核」 | 8，且都有调用点（第 2 节） | 对得上 |
| GPU_GC_PROOFS.md:85 | 「共 24 条」（20 + 两见证 + 两审计） | `gpu-gc-G01`…`G20`（20）+ 2 + 2 = 24；表里 `gpu-gc-` 前缀共 35，另 11 行（`G21` 到 `G30` 与 `gpu-gc-capacity-witness`）这份文件没提 | 对得上，但只说了 35 行里的 24 |
| GPU_GC_PROOFS.md:85 | 「八处 `requires` 另做入口空真筛查」 | `gpu_gc_proofs.rs` 里 `requires` 8 行；`gpu_gc_capacity_proofs.rs:144` 另有 1 处、`gpu_gc_scoring_proofs.rs` 另有若干处，不在这「八」里 | 对得上（限 `gpu_gc_proofs.rs`） |
| GPU_GC_PROOFS.md:106 | 「模块共 26 条义务：20 项核心性质、3 条常量、1 条递归终止、2 个见证」 | 20 项（G01 到 G20）、常量 3（`:9-11`）、`record_slice_sum` 的 `decreases`（`:163`）1 条、见证 2 | 数得出来（义务数由 Verus 数，要主 agent 跑；这里是项数） |
| GPU_GC_PROOFS.md:106 | 「新增四个独立核对测试」「4096 条十二轮历史」 | 4；`0u32..(1 << 12)`、12 轮（`gpu_gc_arithmetic_matches_wide_integer_reference.rs:119-122`） | 对得上 |
| GPU_GC_PROOFS.md:106 | 「后台包命令点名四个模块的 56 个测试」 | 这四个模块里 `#[test]` 静态数是 7 + 36 + 23 + 16 = 82（`two_kernel_reconciliation.rs`、`memory_limit_and_chunks.rs`、`live_slot_kernels.rs`、`region_record_kernels.rs`），四份文件里 `#[cfg(...)]` 只有 `#[cfg(test)]` | **推不出**（82 对 56，差在哪不知道；要主 agent 列测试，命令见末节） |
| GPU_GC_PROOFS.md:117 | 「117,649 条六步历史」 | `7usize.pow(6)`（`crates/singlefs-background-gpu/tests/gpu_gc_formal_contracts_through_the_host.rs:40`）= 117649 | 对得上 |
| GPU_GC_SELFTEST.md:1 | 标题「20 项」 | 这份文件只谈 `gpu_gc_proofs` 一个模块（`:25` 的「24 个必红反例」就是 20 + 2 + 2）；而 `GPU_GC_PROOFS.md:5` 与表里已是 30 项 | 口径与别处不同：对 G01 到 G20 是对的，对「GPU GC 新增形式化验证」整体是旧的 |
| GPU_GC_SELFTEST.md:23 | 「义务合计 186」 | 登记表现是 208（`verus-pins.tsv:48`）；186 是加 G21 到 G30 与打分模块之前的读数 | **对不上现状**（过去那一次的读数，不是错的，但放在「实测」表里说现状会误导） |
| GPU_GC_SELFTEST.md:25 | 「24 个必红反例」 | 24 | 对得上 |
| GPU_GC_SELFTEST.md:26 | 「8 处入口 `requires` 空真筛查」 | 8（`gpu_gc_proofs.rs`） | 对得上（限该模块） |
| GPU_GC_SELFTEST.md:28、:29 | 「4 通过」「2 通过」 | 4、2（`#[test]` 个数） | 对得上 |
| GPU_GC_SELFTEST.md:31 | 「56 通过」 | 同上 82 对 56 | **推不出** |
| GPU_GC_SELFTEST.md:37 | 「85 项通过（4 + 2 + 11 + 56 + 6 + 6）」 | 加起来是 85 | 算术对得上；11、56、6、6 我没核 |
| GPU_GC_SELFTEST.md:61 | 「4 种初始状态 × 4 种相符组合 × 2 种顺序，共 32 个批次」 | 4 × 4 × 2 = 32 | 算术对得上 |

没有任何一处说明文件提到 `gpu_gc_capacity_matches_independent_reference` 的 5 条测试（`grep -n capacity_matches` 在三份说明文件里零命中）。

## 8 检查单八格

### 8.1 走不到的理由二次查实

被判十四份里新加的生产段没有 `unreachable!`、`panic!`、`.expect(`、`.unwrap(`、`assert!`（`grep -nE 'unreachable!|panic!|\.expect\(|\.unwrap\(|todo!|unimplemented!|assert!\(|assert_eq!\(' ` 对三份新证明文件零命中；Verus 里的 `assert(...)` 是证明命题，不算）；新文件里也没有 `_ =>`。两份新测试里有 3 处 `.unwrap()`（`gpu_gc_arithmetic_matches_wide_integer_reference.rs:106`、`:107`，`gpu_gc_capacity_matches_independent_reference.rs:195`），都是不可能失败的转换（8 字节切 4 字节、已装进 u64 的数再取大），按 `.claude/rules/code-discipline-by-review.md` 的「不写 `unwrap()`，写 `expect`，消息写清依赖哪条不变量」应写 `expect`；测试是否在这条之外，规则没写明，**推不出**，记一笔。

返回 `Option` 的几个核，`None` 在什么输入上出现、调用方怎么接，我追到调用点：

| 核 | `None` 出现在 | 调用方怎么接 | 判定 |
|---|---|---|---|
| `checked_live_slot_totals`（G14） | 两个累计里任一个的加法超 `u64::MAX` | crates/singlefs-background-gpu/src/live_slot_kernels.rs:376-379 `.expect("活槽数之和不超过记录数乘 65535，装得进 u64")` → panic；`# Panics` 写了（`:340`） | 走不到的理由「活槽数不超过记录数乘 65535」对**正确的核**成立，那就是 `lemma_every_walk_prefix_fits_the_gpu_counter`（gpu_gc_scoring_proofs.rs:227）证的。但相加的两个数来自设备回读：`decoded_gpu_live_slots`（live_slot_kernels.rs:463）只核判定字、不核数值大小，一份坏掉的核交回接近 `u64::MAX` 的数就会走到这条 `expect`，host 进程 panic，而不是交回错误。设计上「一份核算错只坏它自己那一份」（`region_record_kernels.rs:163`）。**走得到**（推的，没有构造出具体坏输入）；不在这一轮被判的十四份里，交主 agent |
| `checked_gpu_input_bytes`（G15） | `input > u64::MAX - 3` | memory_limit_and_chunks.rs:105-106 `.expect("四字节对齐后的输入长度装得进 u64")`（`input_buffer_bytes`，`:103`，文档没有 `# Panics`）；:312 `?` 上抛到 `bytes_held_by_a_chunk` 的 `.expect`（`:291-292`，`# Panics` 在 `:282-283`）；:385 经 `and_then` 到 `else { return false }`（装不下，不 panic） | 真缓冲长度到不了这个数（主机内存里的切片长度 <= `isize::MAX`），走不到成立；但 `InputLengthOnTheHostInBytes(pub u64)`（`:44`）是公开字段，`input_buffer_bytes` 是 `pub fn`，文档缺 `# Panics` |
| `checked_gpu_chunk_bytes`（G16） | 乘加任一越界 | memory_limit_and_chunks.rs:304-309 的 `?` → `.expect`（`:291-292`，`# Panics` 写了「调用方交的是已经切过、装得下这张卡的一段」） | 靠前面切批已按同一上限切过；成立（依赖切批正确，不由证明保证） |
| `decoded_gpu_live_slots`（G20） | 判定字不是 1 | live_slot_kernels.rs:463-471 `.ok_or(UnrecognizedVerdictWord { entry_position, verdict_word })` 交回 `Err`；调用方 background_gpu.rs:2803-2806 起 `.map_err(GpuFailureCause::KernelLeftAnUnrecognizedVerdictWord)?` | 走得到（核没写到、填充字、写坏），接成错误，**成立** |
| `largest_gpu_piece_input`（G22） | 一条的最小一片都放不下（绑定、派发、显存任一不够） | memory_limit_and_chunks.rs:516-521 `.map(InputLengthOnTheHostInBytes)`，`:535` 起的调用方 `let Some(largest_piece_bytes) = largest_piece else { return Err(SmallestPieceLargerThanTheRoom {...}) }`（`:547-553`） | 走得到，接成 `Err`，**成立**。另 `:554` 有 `assert!(largest_piece_bytes < input_bytes)`：成立的前提是 G21 经产品改过限额的调用（`:375-409`）与 G22 经产品另一种改法的调用（`:497-522`）两处结论一致，没有定理罩这一点（交主 agent） |
| `checked_gpu_memory_hold`（G24） | 有未完成提交、溢出、超限额 | memory_limit_and_chunks.rs:693-699 `.expect("本包在这张卡上占的缓冲超过了上限或 u64：切批与记账必须一致")`，`# Panics` 在 `:682-686`；`pending` 实参此刻必为 `false`（`:688-691` 的 `assert!` 在前） | 内部一致性断言，文档化；超限额走得到的前提是切批与记账不一致，成立 |
| `checked_gpu_memory_release`（G25） | `bytes > held` | :709 `.expect("放掉的缓冲比记下的多")`，`# Panics` `:705-706` | 内部一致性断言，成立 |
| `gpu_bytes_after_submission_wait`（G26） | `chunk > held` | :723-731 `.expect("结算的那一段比记下的多")`，`# Panics` `:716-717` | 同上，成立 |
| `checked_gpu_mapping_table_bytes`（G29） | `60 * 条目 + 16 * 落点 > u64::MAX`，即条目数约大于 3 × 10^17 | region_record_kernels.rs:552-556 `.expect("映射表的总字节数与最大绑定量装得进 u64")` | 内存里装不下这么多条（每条 56 字节），走不到，**成立** |
| `extended_gpu_chunk_input`（G30） | 和溢出，或装不下 | 没有调用方 | 无从追；第 2 节 |

### 8.2 类型与名字

- `as` 转换（任务点名的四处），逐个：
  1. `gpu_gc_capacity_proofs.rs:151` `entries as u64`、`per_group as u64`：`u32` 到 `u64`，无损。
  2. `gpu_gc_capacity_proofs.rs:152` `wide as u32`：`u64` 到 `u32`，**窄化**。`ensures` 管住了：被调的 `units_covering_size` 保证 `count <= size`（crates/singlefs-replay-prefix/src/file_range_proofs.rs:16），所以 `wide <= entries <= u32::MAX`；本函数的 `groups <= entries` 与 `entries <= groups * per_group`（`:146`、`:148`）都建立在这个转换无损上，丢位的话这两条里至少有一条证不过。`#[allow(clippy::cast_possible_truncation, reason = ...)]` 写了理由（`:142`）。**必红表没有把这个转换改成会丢位的行**（373 行换掉了整个表达式，第 5 节）；测试里 `entries = u32::MAX`、`per_group = 1` 穿过它得 `u32::MAX`（`gpu_gc_capacity_matches_independent_reference.rs:205-211`）。
  3. `gpu_gc_capacity_proofs.rs:187` `(previous + additional) as u64`：在 `ensures`（规格）里，不是可执行代码。它在 `||` 的右支，左支 `previous + additional > u64::MAX` 先管住了溢出，右支只有在不溢出时才有意义。管住了。
  4. `gpu_gc_proofs.rs:333` `low as u64 + 4294967296u64 * high as u64`：`as` 比 `*`、`+` 先结合；两个 `u32` 到 `u64` 无损，最大值 `(2^32 - 1) + 2^32 * (2^32 - 1) = 2^64 - 1` 正好装满；`ensures` 写的是 `value == words_value(low, high)`（`:330`），Verus 的算术溢出检查罩着。
- 同一函数里不同语义的量共用裸整数（`.claude/singlefs-ai-sop/rules/code-discipline.md@类型：让非法状态写不出来` 写了「同一种底层类型承载不同含义（逻辑地址、物理地址、代号；字节数、块数），各给一个 newtype。」与「**布尔参数换成枚举**」）：
  - `checked_gpu_chunk_bytes(input, entries, per_entry, fixed)`（gpu_gc_proofs.rs:267）：字节、条数、字节、字节四个裸 `u64`；调用点 memory_limit_and_chunks.rs:304-309 逐个 `.0` 剥壳位置传入，`entries` 与 `per_entry` 对调编得过。
  - `gpu_record_is_in_scan_region(same_device, released, slot, span, start, end)`（gpu_gc_proofs.rs:316）：两个紧挨着的 `bool`，三个槽号（`slot`、`start`、`end`）与一个槽数（`span`）同为 `u64`。
  - `decoded_gpu_live_slots(low, high, verdict)`（gpu_gc_proofs.rs:327）：三个 `u32`，低字、高字对调编得过。
  - `gpu_workgroups_for_entries(entries, per_group)`（gpu_gc_capacity_proofs.rs:143）：两个 `u32`；`checked_gpu_mapping_table_bytes(entries, locations)`（`:161`）两个 `u64`；`checked_gpu_memory_hold(held, bytes, pending, limit)`、`remaining_gpu_memory(limit, held)`、`gpu_bytes_after_submission_wait(held, chunk, completed)` 里字节量互换编得过，另有 `pending`、`completed` 两个 `bool`。
  - `next_gpu_record_slice_end(start, total, capacity)`（gpu_gc_proofs.rs:301）同一个函数承载两种语义：记录位置与条数（live_slot_kernels.rs:303），输入字节偏移（memory_limit_and_chunks.rs:561）。
  - 这几个核在只依赖 `vstd` 的独立 crate 里，`GpuMemoryInBytes` 等 newtype 在 `singlefs-core`，核引不到；产品一侧有 newtype，到核的边界处剥成 `.0`。核的 `GpuChunkCapacity` 的字段名带着语义（`fixed_bytes`、`bytes_per_entry`、`dispatch_entries` 等）。
  - 判定：**有**共用裸整数与布尔参数而按规则该分开的函数（上列）；这个 crate 能不能定义自己的 newtype 而不拖累证明，我没有核，**交主 agent 判**。
- 名字：新文件里的函数名、类型名读起来都说得出内容；`value`（`gpu_gc_proofs.rs:330` 的模式变量）、`data`（partial_write_proofs.rs:109 起的参数）在 `.claude/naming-vague-words` 的词表里，门禁判的是整名由空泛词拼成才红，单个参数名是否算，我没跑；参数名 `first`、`second`、`first_start`、`second_start`（gpu_gc_proofs.rs:93、partial_write_proofs.rs:150）不在 `.claude/naming-ordinal-tokens` 的 19 个词里。**命名检查要主 agent 跑**（命令在末节）。

### 8.3 不兼容旧代码与旧格式

对十四份文件的加行 `grep -nE '旧版|旧布局|旧格式|legacy|migrat|迁移|兼容旧|backward|compat|deprecated|旧库|旧镜像|legacy_ok'`：零命中（`verus-must-red-worlds.tsv:211` 命中的是 HEAD 里原有的一行，不是加行）。README 加行里的「旧」是「旧值」「旧内容」，指被覆盖的数据，不是旧版本。**成立。**

另记（不属这一格）：`GPU_GC_SELFTEST.md` 的实验经过叙述（`:3`「本轮自测发现并修复了」、`:7`「任务 `12857` 在修复前」、`:9` 起一张「原来漏在哪里」「修复」表、`:14`「原 G04 变异的锚点也已收窄」、`:35`「任务的首次尝试因内存上限中断」、`:37`、`:39`）按 `.claude/agent-common.md@写` 里「代码与脚本的注释、说明只写现状，不写历史」属历史叙述；那一条的射程是注释、脚本文件头与几处 `README.md`，`GPU_GC_SELFTEST.md` 在 `crates/` 下但不叫 `README.md`，**射程够不够得着我判不了**，交主 agent。`GPU_GC_SELFTEST.md:23` 的「义务合计 186」这个数也是那一次的读数（现登记 208），见第 7 节。

### 8.4 不新建判定库

十四份文件的加行里没有 `rocksdb`、`sqlite`、`prune` 一类新的存放处。`GPU_GC_SELFTEST.md:39` 与 `:44-48` 用的是 `scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- ...`：`.claude/rules/verification.md@判定库只有一份：全仓统一写进 prune-server 的那份 RocksDB` 写「自证与压测一律对自己起的临时服务，格完即删，不写真服务」；该脚本的 `--with-temporary-service` 起服务、跑命令、「收尾停服务、删临时目录」（`scripts/prune-server/prune_server_client_selftest.py:2009`）。**成立。** 一处顺带：这条自证命令跑的是真 Verus 的必红行，不是假驱动；`.claude/agent-common.md@门禁` 写的「自证一律用假驱动」管的是脚本与钩子的 `--selftest`，这里是借临时服务重判两条新反例，我认为不在它的射程里（推的）。

### 8.5 所有任务多机可用

`grep` 三份说明文件、三份新证明文件、两份新测试、`partial_write_proofs.rs` 里的 `/home`、`/tmp`、`~/`、`$HOME`、`localhost`、IP、`ssh`、`hostname`、`第二台`、`本机`：只命中 `GPU_GC_SELFTEST.md:18` 的「本机」（泛指）与 `:37` 的「最终在第二台的卡 1（`0000:05:00.0`）完成」。说明文件里的命令用相对仓根的路径与 `$VERUS`，没有写死主机名、地址与本机路径。两份新测试不读环境变量、不读时钟、不读本机文件。`:37` 那一处写的是第二台上一张卡的 PCI 地址：`.claude/agent-common.md@写` 列的私有值是「主机名与别名、IP、第二台上的路径、服务单元名、清场脚本内容」，PCI 地址不在名单里；写出来算不算「第二台的」私有信息，**我判不了**。判定：**成立**，附这一处边缘交主 agent。

### 8.6 测试优先多线程

两份新测试都是单线程的纯计算对拍（`grep -nE 'thread|spawn|available_parallelism|--test-threads|std::env'` 零命中），没有写死或覆盖线程数；最重的一条是 `largest_piece_matches_exhaustive_search_and_is_maximal`，43,050 组、每组至多 40 次参照判定，没有重活。**成立。**

### 8.7 带 --item 的命令与项

- 说明文件里的项名：`GPU_GC_PROOFS.md`、`GPU_GC_SELFTEST.md`、`README.md` 里写的 13 个 `must-red:<行名>`（`gpu-gc-G01` 到 `G04`、四个 `gpu-score-*`、两个 `gpu-gc-audit-*`、三个 `overwrite-*`）在 `verus-must-red-worlds.tsv` 第一列里各恰好 1 行（逐个 `cut -f1 | grep -cxF` 得 1）。`--list-items` 实际列不列得出要跑，命令在末节（**推不出**）。
- 命令写法：这些命令都是 `task-server/task submit --manual -- ...`。`--manual` 是「人手交」的标记：从 Claude Code 会话里带它交，服务当场拒，`task-server/task_server.py:9290` 写「带了 --manual，而这条命令是会话」（后接会话号与「起的：这一条不收、不跑」）；主 agent 与子 agent 都不能照抄。它们也都没带 `SINGLEFS_RUN_SCOPE='…'` 范围声明（`.claude/rules/verification.md@一个门禁：两种跑法` 写「起任何验证的命令开头带范围声明」）。两处都是写给人手的命令，我按这个用途判不算写错，但**不能原样交任务**。
- `--item must-red:<名>` 的形态与 `scripts/gate/verus-proof-check.py must-red-worlds <根> --item` 一致（该脚本 `:93`、`:135`）。
- 点名的测试目标：`cargo test -p singlefs-replay-prefix --offline --test gpu_gc_arithmetic_matches_wide_integer_reference` 目标名对得上文件名；`gpu_gc_capacity_matches_independent_reference` 没有出现在任何一份说明文件的命令里。
- 判定：**不成立**（命令不能原样交、缺范围声明、缺 G21 到 G30 与 capacity 测试的点名项）；项名本身对。

### 8.8 新崩溃流多罩了什么

十四份文件里没有新加崩溃点重放流或崩溃枚举用例：两份新测试与三份新证明文件里 `grep -nE 'crash|崩溃|CRASH'` 零命中；`GPU_GC_PROOFS.md:3` 的「崩溃协议证明不重复计入」是在说「已有的」。`crates/mutations.tsv` 里含 `gpu_gc` 字样的行是两行（`grep -n 'gpu_gc\|gpu-gc' crates/mutations.tsv` 命中 `:5494`、`:5602`，HEAD 里 0 行），它们的第二段是 `crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs`、`live_slot_kernels.rs`，改的是后台包调用点，不是这十四份；指到这十四份文件的新行我没有另查（正文说 0 行，我没复核）。**成立。**

## 9 facts.md 写错或说得不准的地方

下面的 facts.md 行号指 `research/prompts/replay-prefix-backfill-code-r1-snapshot/facts.md`（cite-check 认不出只写文件名的这份，列进「没判」，行号我对着文件现取），对的是被判文件今天的样子。

1. facts.md:6 写 `GPU_GC_PROOFS.md`「列 G01–G30 各条性质」。表只到 G20；G21 到 G30 只在 `GPU_GC_PROOFS.md:5` 一句话里点了名目，没有逐条的表。
2. facts.md:140 写 `gpu_gc_capacity_proofs.rs`「共 11 个 exec 函数与一个边界见证」。产品 exec 函数 10 个，加上见证本身（也是 exec 函数）才是 11；说成 11 加一个见证多算了一个。facts.md:156 自己列的 12 个新名（一个结构体、十个函数、一个见证）是对的。
3. facts.md:396、:400 写必红表「L198 一行证明负例名去 a_」。行名**没有**去 `a_`（`verus-must-red-worlds.tsv:198` 行名仍是 `证明负例·a_segment_holding_a_pinned_slot_is_not_a_sound_choice …`），改的是第六段落点（`landing=segment_holding_a_pinned_slot_is_not_a_sound_choice`）。
4. facts.md:48 写 `GPU_GC_SELFTEST.md:14、:39、:44` 是实验经过叙述。`:44` 是代码块里的一条命令，不是叙述；叙述在 `:3`、`:7`、`:9-12`、`:14`、`:35`、`:37`、`:39`（第 8.3 节）。
5. facts.md:152 把 `gpu_gc_capacity_proofs.rs:187` 列为「生产段 `as` 转换」。那一行在 `ensures` 里（规格，普通构建里擦掉），不是可执行代码；另外 facts 的 `as` 清单没有列规格里的 `as int`、`as nat`（`gpu_gc_capacity_proofs.rs:123`、`:156`，gpu_gc_proofs.rs 里十来处，gpu_gc_scoring_proofs.rs 里十来处），那些是无损的升宽，列不列不影响结论。
6. facts.md:218「`gpu_gc_scoring_proofs.rs` 类型与名字：生产段新加的条目名 0 个」。这份文件没有结构体与可执行函数，但有 9 个证明函数与 7 个规格函数（`grep -cE 'spec fn'` 得 8 是因为 `gpu_gc_scoring_proofs.rs:5` 的属性说明里也写了「spec fn」；facts.md:228 自己列的是 16 个，对），它们是新名字；「0 个」只是说没有 exec 层的名字，写进检查单格里会让读的人以为没有新名字。
7. facts.md:267 写 partial_write_proofs.rs「L78 起新增 115 行」。覆盖写那一段在 `partial_write_proofs.rs:108-204`，共 97 行；115 是那个 hunk 加删合起来的行数，含改名的行。

facts.md 里我核过且对的：必红表行名前缀计数（`gpu-gc` 35、`gpu-score` 15、`overwrite-contract` 5，第 7 节的命令读数一致）；README 的「八份」「另七份」与 src 下 10 份对不上；`GPU_GC_SELFTEST.md` 标题「20 项」与别处 G30 口径不同；新加测试里 `assert/expect/panic/unwrap` 的个数（14、22）；两份新测试的 `#[test]` 个数（4、5）；改名块的「不动 requires/ensures/decreases」。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 1 G01 到 G20 | 对得上 | 二十条 `ensures` 与表一一对上；G04 是定义复述，G10 只证两份重数，G17 的源码注释「正的新增分配必在总限内」没有对应 `ensures`（推不出） |
| 1 G21 到 G30 | 对得上 | 十条与代码里 `/// G2x` 注释、`GPU_GC_PROOFS.md:5` 的名目逐条对上；说明文件没有这十条的表，也没有复跑命令 |
| 1 打分模块七处 | 对得上 | `GPU_GC_PROOFS.md:9-22` 的每句都有对应 `ensures`；二分中点形式与 WGSL 一致 |
| 1 头注释与说明文件指错 | 对不上两处 | `gpu_gc_proofs.rs:1` 指向 README（那里没有 G 清单）；`GPU_GC_PROOFS.md:5` 说 G21 到 G30 都被产品直接调用，G30 没有 |
| 2 产品直接调用 | 八个核与 G21 到 G29 成立，G30 不成立 | 调用点逐个列在第 2 节；`extended_gpu_chunk_input` 在 `crates/singlefs-background-gpu/src/` 没有调用点；调用点在已暂存未提交的后台包改动里，HEAD 为 0 |
| 3 覆盖写 | 前提可满足、产品满足的只是子集 | 见证罩住每条 `requires`；产品写路径允许越过末尾与补洞，`overwrite_fits` 只在 `offset + length <= 写前 size` 时成立；README 新节与证明对得上 |
| 4 改名 | 纯改名（五个文件共 62 个 hunk），非改名三处 | 记号级核过，`requires`/`ensures`/`decreases` 结构没动；非改名：三行模块声明、覆盖写新段、两处排版；必红表 `:198` 行名遗留旧名 |
| 5 必红表 | 66 行锚点各命中 1 次 | 改坏后红在哪条 `ensures` 是推的；五行 `ensures false` 只证明函数被验、前提不矛盾；按条没罩到的清单在 5.3，覆盖写的 `overwritten_bytes` 定义、三条常量、`wide as u32` 丢位、G14 到 G16 的溢出臂没有行 |
| 6 两份测试 | 各写各的 | 参照用 u128 与标准库、不共用代码；G27 的参照与被测同式，G29 的 56/16/4 同字面值；memory-limit 恰等边界取不到；没有变异行罩着测试 |
| 7 说明文件的数 | 对不上四处，推不出两处 | README「八份」「另七份」（实 10、9）；SELFTEST 标题「20 项」与「义务合计 186」（登记 208）；「56 个测试」与静态 82 对不上、推不出 |
| 8.1 走不到的理由 | 成立（被判范围） | 新文件没有新加走不到的断言；调用点 `live_slot_kernels.rs:376-379` 的 `expect` 对设备交回的数走得到，交主 agent |
| 8.2 类型与名字 | 不成立（裸整数与布尔参数）；`as` 四处成立 | `as`：四处都管住；`checked_gpu_chunk_bytes` 等函数共用裸整数、`bool` 参数相邻，能否在 Verus 核里改 newtype 交主 agent；命名检查要跑 |
| 8.3 不兼容旧代码 | 成立 | 加行零命中；SELFTEST 的经过叙述射程交主 agent |
| 8.4 不新建判定库 | 成立 | 用的是自己起的临时服务，收尾删临时目录 |
| 8.5 多机可用 | 成立 | 无主机名、无本机路径；GPU_GC_SELFTEST.md:37 的第二台卡 PCI 地址属边缘，交主 agent |
| 8.6 多线程 | 成立 | 单线程纯计算，无线程数设置 |
| 8.7 带 --item 的命令与项 | 不成立 | 13 个项名都在表里；命令是 `--manual` 人手交、不带范围声明、不能原样交，没有 G21 到 G30 与 capacity 测试的点名项 |
| 8.8 新崩溃流 | 成立 | 零命中 |
| 9 facts.md | 错七处 | 见第 9 节 |

## 要主 agent 跑的命令

我没有起任何命令。每条的「在哪个目录起」都是仓根 `/home/fy5090/code/singlefs`。下面凡是「读数」，我这里写的都是推的，没有跑出来。

1. 先列项，核 `must-red:` 项名、`vacuity:` 项、`proof` 项都在，并数 G21 到 G30、打分模块的 `vacuity:` 项条数：
   `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=formal-verification 的 --list-items; 范围=只列项不判; 为什么=核说明文件点名的项都列得出，并数新模块的空真项' task-server/task submit -- bash .claude/gate.d/formal-verification.sh --list-items`
   在仓根；看 `TASK_SERVER_RESULT` 的 `exit=0`，`task-server/task log <号>` 里 `must-red:gpu-gc-G21` 到 `G30`、`must-red:gpu-gc-capacity-witness` 在不在、`must-red:` 项共几条（表里应是 `grep -vc '^#\|^$'` 的行数）、`vacuity:` 里 `gpu_gc_capacity_proofs`、`gpu_gc_scoring_proofs` 各几处。
2. 整个 crate 的证明一格（核 `186 → 208` 与三份新模块、覆盖写四条引理真验过）：
   `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=formal-verification 的 --item proof（整格）; 范围=1 项; 为什么=核 replay-prefix 全包证明含三份新证明文件与覆盖写段' task-server/task submit -- bash .claude/gate.d/formal-verification.sh --item proof`
   在仓根；看 `exit=0`、log 里 `verified` 与登记 208 是否相等、`errors 0`。
3. 必红行复跑（五组，每组一条命令，项名由 `tsv` 现读生成，写在草稿目录 `cmds.txt`，下面原样抄）。每条看 `exit=0`（每行改坏后证明错的落点与第六段相同），失败的行在 log 里点名：
   `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=formal-verification 的 must-red 项（覆盖写与部分写改名）共 12 项; 范围=12 项; 为什么=复跑这一轮判的必红行' task-server/task submit -- bash .claude/gate.d/formal-verification.sh --item 'must-red:overwrite-contract-lemma_overwrite_changes_exactly_the_requested_bytes' --item 'must-red:overwrite-contract-lemma_overwriting_the_same_range_keeps_only_the_last_write' --item 'must-red:overwrite-contract-lemma_disjoint_overwrites_commute' --item 'must-red:overwrite-contract-lemma_whole_unit_overwrite_does_not_depend_on_old_bytes' --item 'must-red:overwrite-contract-witness_overwrite_laws_and_overlapping_write_order' --item 'must-red:overwrite-last-needs-equal-lengths' --item 'must-red:overwrite-commute-needs-disjoint-ranges' --item 'must-red:overwrite-whole-needs-coverage' --item 'must-red:partial-touching-endpoints-are-not-written' --item 'must-red:partial-equal-range-covers-whole-unit' --item 'must-red:partial-whole-coverage-needs-nonempty-unit' --item 'must-red:partial-touch-needs-nonempty-write'`
   在仓根。
   `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=formal-verification 的 must-red 项（G01 到 G20、两个见证、两个审计）共 24 项; 范围=24 项; 为什么=复跑这一轮判的必红行' task-server/task submit -- bash .claude/gate.d/formal-verification.sh --item 'must-red:gpu-gc-G01' --item 'must-red:gpu-gc-G02' --item 'must-red:gpu-gc-G03' --item 'must-red:gpu-gc-G04' --item 'must-red:gpu-gc-G05' --item 'must-red:gpu-gc-G06' --item 'must-red:gpu-gc-G07' --item 'must-red:gpu-gc-G08' --item 'must-red:gpu-gc-G09' --item 'must-red:gpu-gc-G10' --item 'must-red:gpu-gc-G11' --item 'must-red:gpu-gc-G12' --item 'must-red:gpu-gc-G13' --item 'must-red:gpu-gc-G14' --item 'must-red:gpu-gc-G15' --item 'must-red:gpu-gc-G16' --item 'must-red:gpu-gc-G17' --item 'must-red:gpu-gc-G18' --item 'must-red:gpu-gc-G19' --item 'must-red:gpu-gc-G20' --item 'must-red:gpu-gc-witness_gpu_gc_protocol_and_score_premises' --item 'must-red:gpu-gc-witness_gpu_gc_arithmetic_boundaries' --item 'must-red:gpu-gc-audit-score-capped-at-two' --item 'must-red:gpu-gc-audit-successful-cpu-call-lost'`
   在仓根。
   `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=formal-verification 的 must-red 项（G21 到 G30 与容量见证）共 11 项; 范围=11 项; 为什么=复跑这一轮判的必红行' task-server/task submit -- bash .claude/gate.d/formal-verification.sh --item 'must-red:gpu-gc-G21' --item 'must-red:gpu-gc-G22' --item 'must-red:gpu-gc-G23' --item 'must-red:gpu-gc-G24' --item 'must-red:gpu-gc-G25' --item 'must-red:gpu-gc-G26' --item 'must-red:gpu-gc-G27' --item 'must-red:gpu-gc-G28' --item 'must-red:gpu-gc-G29' --item 'must-red:gpu-gc-G30' --item 'must-red:gpu-gc-capacity-witness'`
   在仓根。
   `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=formal-verification 的 must-red 项（打分模块）共 15 项; 范围=15 项; 为什么=复跑这一轮判的必红行' task-server/task submit -- bash .claude/gate.d/formal-verification.sh --item 'must-red:gpu-score-binary-skips-entry' --item 'must-red:gpu-score-binary-drops-upper' --item 'must-red:gpu-score-binary-needs-sorted-records' --item 'must-red:gpu-score-slot-loop-shifted' --item 'must-red:gpu-score-slot-count-induction-starts-one-slot-late' --item 'must-red:gpu-score-window-too-narrow' --item 'must-red:gpu-score-search-counts-released' --item 'must-red:gpu-score-search-skips-record' --item 'must-red:gpu-score-candidate-needs-valid-spans' --item 'must-red:gpu-score-prefix-skips-induction-step' --item 'must-red:gpu-score-fold-skips-induction-step' --item 'must-red:gpu-score-permutation-needs-multiplicity' --item 'must-red:gpu-score-theorem-needs-permutation' --item 'must-red:gpu-score-counter-needs-record-bound' --item 'must-red:gpu-score-witness-is-checked'`
   在仓根。
   `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=formal-verification 的 must-red 项（lib 与整理计划的四行改名）共 4 项; 范围=4 项; 为什么=复跑这一轮判的必红行' task-server/task submit -- bash .claude/gate.d/formal-verification.sh --item 'must-red:ensures·冒烟：两条记录的整次发布改成断言不是拼接' --item 'must-red:ensures·冒烟：一条不带末条标志的记录改成断言是拼接' --item 'must-red:ensures·readable_record_of_the_same_publish_follows 交回的判定取反' --item 'must-red:证明负例·a_segment_holding_a_pinned_slot_is_not_a_sound_choice 的函数体不能推出 false'`
   在仓根。
4. 两份新对拍测试：
   `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-replay-prefix 的测试目标 gpu_gc_arithmetic_matches_wide_integer_reference; 范围=1 个测试目标; 为什么=复跑新加的 4 条全宽对拍' task-server/task submit -- cargo test -p singlefs-replay-prefix --offline --test gpu_gc_arithmetic_matches_wide_integer_reference`
   在仓根；看 `exit=0`、log 里 `test result: ok. 4 passed`。
   `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-replay-prefix 的测试目标 gpu_gc_capacity_matches_independent_reference; 范围=1 个测试目标; 为什么=复跑新加的 5 条独立参照对拍' task-server/task submit -- cargo test -p singlefs-replay-prefix --offline --test gpu_gc_capacity_matches_independent_reference`
   在仓根；看 `exit=0`、log 里 `test result: ok. 5 passed`。
5. 改名那一批的库内测试（改名的四条 `lib.rs` 测试、`compaction_plan_proofs`、`file_range_proofs`、`unlink_proofs`、`partial_write_proofs` 各一条命令，点名到模块或函数）：
   `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-replay-prefix 的 --lib partial_write_proofs::tests; 范围=该模块的测试; 为什么=核形参改名后的测试仍绿' task-server/task submit -- cargo test -p singlefs-replay-prefix --offline --lib partial_write_proofs::tests`
   在仓根；看 `exit=0`、`test result: ok`。
   `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-replay-prefix 的 --lib file_range_proofs::tests; 范围=该模块的测试; 为什么=核闭包改名后的测试仍绿' task-server/task submit -- cargo test -p singlefs-replay-prefix --offline --lib file_range_proofs::tests`
   在仓根；看 `exit=0`、`test result: ok`。
   `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-replay-prefix 的 --lib unlink_proofs::tests; 范围=该模块的测试; 为什么=核循环变量改名后的测试仍绿' task-server/task submit -- cargo test -p singlefs-replay-prefix --offline --lib unlink_proofs::tests`
   在仓根；看 `exit=0`、`test result: ok`。
   `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-replay-prefix 的 --lib compaction_plan_proofs::tests; 范围=该模块的测试; 为什么=核改名后的测试仍绿' task-server/task submit -- cargo test -p singlefs-replay-prefix --offline --lib compaction_plan_proofs::tests`
   在仓根；看 `exit=0`、`test result: ok`。
   `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-replay-prefix 的 --lib 里 lib.rs 改名的四条测试; 范围=4 条测试; 为什么=核去掉 a_ 前缀后四条测试仍在且绿' task-server/task submit -- cargo test -p singlefs-replay-prefix --offline --lib -- two_record_publish_after_the_anchor_is_applied_whole_and_the_new_root_segment_comes_from_its_last_record publish_whose_last_record_is_missing_is_not_applied_and_the_root_segment_stays later_checkpoint_txg_is_chosen_over_a_higher_instance_with_an_earlier_checkpoint_txg newer_root_after_the_first_self_verified_slot_is_chosen_over_it`
   在仓根；看 `exit=0`、log 里 `4 passed`。
6. 数「四个模块的 56 个测试」（我的静态数是 82）：
   `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-background-gpu 的 --lib 四个模块的测试，只列不跑; 范围=四个模块; 为什么=核 GPU_GC_PROOFS.md:106 与 GPU_GC_SELFTEST.md:31 的 56 这个数' task-server/task submit -- cargo test -p singlefs-background-gpu --offline --lib two_kernel_reconciliation::tests -- memory_limit_and_chunks::tests live_slot_kernels::tests region_record_kernels::tests --list`
   在仓根；看 `exit=0`，log 里以 `: test` 结尾的行数是不是 56（我读到的静态 `#[test]` 是 7 + 36 + 23 + 16 = 82）。
7. 命名检查（新 `.rs` 文件的命名；改动的树就是仓根）：
   `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-source-discipline.sh 的 vague-names、test-file-names、ordinal-names、data-names 四格; 范围=4 格; 为什么=核这一批新 .rs 的命名' task-server/task submit -- bash .claude/gate.d/code-source-discipline.sh --check vague-names,test-file-names,ordinal-names,data-names .`
   在仓根；看末行 `TASK_SERVER_RESULT` 的 `exit=0` 是过、`exit=1` 是判红，红了看命中的文件是不是这十四份。
   `task-server/task submit -- bash .claude/singlefs-ai-sop/scripts/naming-lint.sh .`
   在仓根；看末行 `TASK_SERVER_RESULT` 的 `exit=0`。
8. 证红（我没做、也没法在只读下做）：把第 4 条两份测试里被测的 `checked_gpu_input_bytes` 的 `words.checked_mul(GPU_INPUT_ALIGNMENT)` 临时改成 `words.checked_mul(1)`、`gpu_workgroups_for_entries` 的 `wide as u32` 临时改成 `entries / per_group`，各看对应测试红不红。这一步按 `.claude/agents/implementation-writer.md` 第 3a 步的证红写法由主 agent 起（我没读那一节的细则，不给命令）。

## 什么现象会推翻我的结论

| 结论 | 推翻它的现象 |
|---|---|
| G01 到 G30「对得上」 | 某个 G 的名字在说明文件或代码里有一个读法，是 `ensures` 蕴涵不出来的（我对 G17 注释的后半句就是这么判的）；或第 1、2 条命令（`--item proof`）报某条引理证不过——那会是证明错，不是语义对不上，但会让「对得上」只停在文字层 |
| 「G30 没有产品调用点」 | `grep -rnw extended_gpu_chunk_input crates/singlefs-background-gpu/src/` 出现一处命中；或有人把 `chunks_that_fit_on_the_gpu`（memory_limit_and_chunks.rs:434-443）改成调它 |
| 「产品写路径不要求 `overwrite_fits`」 | `publish_partial_write`（transaction.rs:12548）里写入之前有一处判 `offset + length <= size`（我读到的只有 `target_size_rounded_up_to_a_whole_data_unit_fits` 一处判写后 size 的取整，transaction.rs:12596，不是这个条件） |
| 「改名是纯改名」 | 把 `tokdiff.py` 的输出里 `requires`、`ensures`、`decreases`、`==>`、数字、括号任何一个记号出现在不同处；或新旧名字有一处撞名（我的次数核对：旧名 71/11/7/18/6 → 新名同数，HEAD 里新名 0 次） |
| 「必红表 66 行锚点各命中 1 次」 | `anchors.py changed` 重跑得出任一行不是 1；或工作区的源文件在我读之后被别的会话改过（开工与交回时我各核过 14 份 sha256，下面写交回时的结果） |
| 「红在哪条 `ensures`」（推的） | 主 agent 跑第 3 条，某行落点与第六段不同，或不红；或红在我没写的那一条 |
| 「测试与被测各写各的」 | 某份测试里的参照式调用了被测 crate 的函数，或抄了被测的函数体（我读到零处）；或把被测植入已知错（第 8 条）测试仍绿，则「参照有效」推翻 |
| 「56 推不出」 | 第 6 条数出恰好 56 |
| 「`live_slot_kernels.rs:376-379` 的走不到对设备输出走得到」 | 能指出 `decoded_gpu_live_slots` 之外还有一道把设备交回的数卡在 `记录数 × 65535` 以内的检查（我读 `:441-474` 与 `:341-390` 没有找到） |

## 没做什么

- 没起任何命令、没跑 Verus、没起 cargo、没跑门禁、没交任务；所有「证明会过」「必红行红在哪」「测试会绿」都是读代码推的。
- 没有写模型，没有 `rerun.sh`，没有复跑用的外包脚本，也没有测探针的内存与时间；草稿目录里只有读文件的小核对脚本（`tokdiff.py` 017131f8…、`anchors.py` 6ca2917d…、`grepcheck.py` 5061af68…、`landings.py` b689d32b…、`edgecheck.py` 5c4f45c3…、`gencmds.py`），共约 64K，没有仓副本、编译目录。
- 没判别的腿的格：没采纳或否决别的腿的结论，没出判决。
- 没读 `.claude/agents/implementation-writer.md` 第 3a 步与 `scripts/mutation/` 一带，所以证红命令没写。
- 没核 `crates/mutations.tsv` 里指到这十四份文件的新行是不是 0（正文说 0；我只核了含 `gpu_gc` 字样的两行不是指到这十四份、以及指到两份新测试与三份新证明文件的行是 0）。
- 没核 `GPU_GC_SELFTEST.md` 里的任务号、通过数（11、56、6、6）与「产品 109 个条目」这类要看任务日志的读数。
- 没核后台包里被调函数之外的逻辑（切批、记账、WGSL 与主机的翻译）是否正确；只核了调用点与 `None` 怎么接。
- 那次越线读到的另一条腿报告的一行（报告开头已写）：我没有再读那份文件。

## 交回前复核

- `sha256sum -c research/prompts/replay-prefix-backfill-code-r1-snapshot/crates-sha256.txt`：十四份仍全是 OK（被判文件在我读的这段时间里没有被别的会话改过）；`kb-sha256.txt` 两行仍是 OK。
- 本报告的引文核对：`python3 scripts/agents/cite-check.py research/prompts/replay-prefix-backfill-code-r1-sonnet-output.md` 末行见交回正文（退出码 0，没有判红的引文；只写文件名而重名的路径列进「没判」，行号我都对着文件现取）。
