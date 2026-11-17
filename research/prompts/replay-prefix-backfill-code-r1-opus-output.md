# 重放前缀包在途改动代码轮第一轮：攻方腿报告

日期：2026-11-17（`TZ=Asia/Tokyo date +%F`）。被判的十四份文件开工时与交回前各跑一次 `sha256sum -c research/prompts/replay-prefix-backfill-code-r1-snapshot/crates-sha256.txt`，两次都是 14 份 OK，没有一份在腿跑着时被改。

这条腿没写模型（没有模型目录、没有 rerun.sh）。为三处「证明前提不承重」的打中在草稿目录建了一份仓副本，只往副本的必红表追加三条攻击行，交主 agent 跑（「要主 agent 跑的命令」第一条）。副本上的结局没出之前，那三处标「推的」。

## 各格判定一览

| 格 | 结论 | 依据（详见各节） |
|---|---|---|
| 件 1 三份新证明文件：空真 | 没打中 | 每条 requires 都有见证喂到可满足的实参；必红表三条见证行与 G 行在见证里插 `assert(false)`／`ensures false` |
| 件 1：ensures 比性质名弱 | 打中两处（G04 一族是定义展开；G21／G22 的「显存」只是交给它的那个上限，不是产品真占用） | 「件 1」A、B |
| 件 1：exec 里的 as 转换 | 没打中 | 四处都不丢位，ensures 管住了（推导在「件 1」C） |
| 件 1：说明文件把缺口说成证过了 | 打中：G30 写成「产品直接调用」而产品一处都不调，另写了一份没验过的同样判法 | 「件 1」D |
| 件 1：说明文件「证明中间值落在 u32 范围内」 | 打中（推的，待主 agent 跑副本攻击行）：那条 u32 界在证明里不承重 | 「件 1」E |
| 件 2 覆盖写四条引理的前提 | 部分打中：`lemma_overwrite_changes_exactly_the_requested_bytes` 的 requires 删掉照样成立（推的，待跑）；四条都不是产品写路径的前提（说明文件写了边界） | 「件 2」 |
| 件 3 两份新测试 | 没打中共用代码；打中「同一份口径」：参照与被测同把对齐输入算一遍，测不出产品的暂存口径 | 「件 3」 |
| 件 4 必红表新行 | 58 行数据行逐行核过落点与原文唯一，没找到不红或落点错的；`gpu-score-window-too-narrow` 改 65533 才红，65534 照样验得过（推的，待跑） | 「件 4」 |
| 件 5 改名 | 纯改名，requires／ensures／decreases 结构没动；必红表第 198 行变异名还留旧函数名 | 「件 5」 |
| 件 6 三份说明文件的数 | 打中七处与代码对不上的数 | 「件 6」 |
| 检查单·走不到的理由二次查实 | 不成立 | `checked_live_slot_totals` 的 None 在产品里是 `expect`，理由只对正确的 GPU 成立 |
| 检查单·类型与名字 | 不成立 | 字节数与条数共用裸 `u64`；布尔参数不以 `is_`／`has_` 起头 |
| 检查单·不兼容旧代码与旧格式 | 成立 | 没有认旧版本的代码；说明文件里有历史叙述，另列 |
| 检查单·不新建判定库 | 成立 | 临时服务格完即删，不留存放处；用途超出「自证与压测」，另列 |
| 检查单·所有任务多机可用 | 成立 | 复跑命令不写主机名、地址与本机路径；SELFTEST 写了第二台的一张卡的 PCI 地址，交主 agent 判 |
| 检查单·测试优先多线程 | 成立 | 两份测试是秒级以内的小循环（推的，没量） |
| 检查单·带 --item 的命令与项 | 推不出 | 项名都在表里、写法对得上判法文件头；没跑 `--list-items`，命令在「要主 agent 跑的命令」第二条 |
| 检查单·新崩溃流多罩了什么 | 成立 | 没有新加崩溃点重放流与崩溃枚举用例 |

## 件 1：三份新证明文件

### A. G04 一族的 ensures 是 spec 定义的展开（ensures 比性质名弱）

- 性质名说的：`GPU_GC_PROOFS.md` 表里「G04 | GPU 批失败不改状态；成功轮次精确更新计数与 CPU 调用次数。」（`crates/singlefs-replay-prefix/src/GPU_GC_PROOFS.md:49`）。
- ensures 实际证的：「round_state(counts, cpu_calls, key, agree, false) == (counts, cpu_calls),」（`crates/singlefs-replay-prefix/src/gpu_gc_proofs.rs:85`），而 `round_state` 的失败那一支就是「} else { (counts, cpu_calls) }」（`crates/singlefs-replay-prefix/src/gpu_gc_proofs.rs:79`）。ensures 第二、三条同样逐字是定义的成功那一支。
- 差在哪：`round_state` 没有 exec 函数、没有产品代码与它相连，所以这条对产品零约束；它只钉住 spec 函数自己（`gpu-gc-audit-successful-cpu-call-lost` 改坏定义会红，这就是它全部的作用）。G02、G03 是同一形状（spec 函数在常数点上取值）。
- 说明文件没把它说成证过了：「G04 对应批失败时的提前返回，不能据此宣称整个批处理循环已验证。」（`crates/singlefs-replay-prefix/src/GPU_GC_PROOFS.md:71`），表里层次写「模型」。所以这一格算「性质名比 ensures 宽」，不算越位。
- 四句：不分辨臂（代码轮没有臂，只判这条证明值不值它的名字）；被判对象当时看得到（定义就在同一文件）；满足的是攻击面「性质名说的比 ensures 证的多」那一分句；改法（把 G04 改名成「round_state 的两支」或在产品 `reconcile_unless_the_batch_failed` 上做精化）都碰得到这一格，后一种是新证明。

### B. G21／G22 的「显存」是交给它的上限，不是产品真占用

- 性质名：「G21：显存、输入绑定、条目绑定和派发四道界必须同时满足；乘加溢出不能误放行。」（`crates/singlefs-replay-prefix/src/gpu_gc_capacity_proofs.rs:27`）。
- 产品侧：「//! 被验的 gpu_chunk_fits 只把对齐后的输入算一遍，映射着建的输入连同它的暂存在卡上占两遍：交给它之前把显存上限先扣掉一份对齐后的输入」（`crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:6`）。扣上限那一步在 `chunk_fits_with_the_staging_of_its_input`（「fn chunk_fits_with_the_staging_of_its_input(」，`crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:375`），没进 Verus。
- 差在哪：ensures 证的是「对齐输入 + 条数×每条 + 每段 ≤ 交进来的上限」；「真占用 ≤ 卡的上限」靠的是没验的那一层折算。`GPU_GC_PROOFS.md` 只写「并不证明外层资源账本所有调用都符合协议」，没点出这一层折算（暂存份数）在核外。
- 四句：不分辨臂；被判对象看不到暂存份数（它的参数里没有）；满足「模型与产品调用点之间的缺口」那一分句；改法「把份数作为参数进核」碰得到，「只改说明文件」只修说法不修缺口。

### C. 四处 as 转换（没打中）

- `gpu_gc_capacity_proofs.rs:151`：`entries as u64`、`per_group as u64` 是 u32 加宽，不丢位。
- `gpu_gc_capacity_proofs.rs:152`：`wide as u32`。被调的 `units_covering_size` 的 ensures 有 `count <= size`，`size` 就是 `entries as u64` ≤ `u32::MAX`；本函数 ensures 也有 `groups <= entries`。改成截断值的话 `entries <= groups * per_group` 这条会断，ensures 管住了。
- `gpu_gc_capacity_proofs.rs:187`：在 ensures 里（spec），`(previous + additional) as u64` 只在前一个析取项 `previous + additional > u64::MAX` 为假时才起作用，那时不截断。
- 「if verdict != GPU_LIVE_SLOTS_COUNTED { None } else { Some(low as u64 + 4294967296u64 * high as u64) }」（`crates/singlefs-replay-prefix/src/gpu_gc_proofs.rs:333`）：`as` 比 `*`、`+` 结合得紧，算的是 `low + 2^32 × high`，最大 `2^64 − 1`，不溢出；Verus 对 exec 算术查溢出，ensures 写了 `value == words_value(low, high)`。
- 试过的形状：把四处换成截断、换成 `high << 32 | low`、把 `per_group` 设 0（requires 挡住，产品三处都传常数工作组大小）。都没找到丢位而 ensures 还成立的输入。

### D. G30 写成「产品直接调用」，产品不调它（打中）

- 说的：「//! GPU GC 的切批准入、最大分片、显存账本和派发尺寸；产品调用同一份 exec 核。」（`crates/singlefs-replay-prefix/src/gpu_gc_capacity_proofs.rs:1`）；`GPU_GC_PROOFS.md` 列了十项之后说「这些是产品直接调用的可执行核；并不证明外层资源账本所有调用都符合协议。」（`crates/singlefs-replay-prefix/src/GPU_GC_PROOFS.md:5`），十项里包括「追加输入准入」即 G30。
- 实际：`git grep -n extended_gpu_chunk_input` 在 `crates/` 下只命中定义「pub fn extended_gpu_chunk_input(previous: u64, additional: u64, entries: u64, capacity: &GpuChunkCapacity) -> (answer: Option<u64>)」（`crates/singlefs-replay-prefix/src/gpu_gc_capacity_proofs.rs:183`）与测试里的一处调用（`crates/singlefs-replay-prefix/tests/gpu_gc_capacity_matches_independent_reference.rs:47`）；`crates/singlefs-background-gpu/src/` 里零命中（命令与输出：`grep -rn 'extended_gpu_chunk_input' crates/ scripts/ --include=*.rs --include=*.py --include=*.md --include=*.tsv`，除必红表外只这两行）。
- 产品自己写了同样的判法：`chunks_that_fit_on_the_gpu` 里「.checked_add(input_bytes)」（`crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:435`）接 `.filter`，再调「chunk_fits_with_the_staging_of_its_input(」（`crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:437`）。这段加法与过滤在核外、没验过；G30 证的那份核是死代码。
- 另外九项（G21–G29）各有产品调用点：`gpu_chunk_fits`、`largest_gpu_piece_input`、`tightened_gpu_memory_limit`、`checked_gpu_memory_hold`、`checked_gpu_memory_release`、`gpu_bytes_after_submission_wait` 在 `memory_limit_and_chunks.rs`，`gpu_memory_cap_after_allocation_failure`、`gpu_workgroups_for_entries`（三处）在 `background_gpu.rs`，`checked_gpu_mapping_table_bytes` 在 `region_record_kernels.rs`（同一条 grep 的输出）。G01–G20 那八个产品核也各有调用点。
- 四句：不分辨臂；被判对象看不到（证明管不到谁调它）；满足「缺口被说明文件说成证过了」那一分句；改法「产品改调 G30」碰得到但不能照搬（产品每次按输入长度折上限，G30 收的是一份固定的 capacity），「说明文件改成 G30 没有产品调用点、或删 G30 与它的必红行」碰得到说法那一格。两种改法只在我这里推过、被攻过零轮。

### E. 「证明中间值落在 u32 范围内」那条界不承重（打中，推的，待跑）

- 说的：「二分模型以递归表示每一次循环状态转移，使用数学整数并证明中间值落在 u32 范围内；」（`crates/singlefs-replay-prefix/src/GPU_GC_PROOFS.md:22`）。
- 证明里唯一的 u32 是 requires「0 <= below <= above <= records.len() <= u32::MAX,」（`crates/singlefs-replay-prefix/src/gpu_gc_scoring_proofs.rs:35`）。函数体只 `assert(below <= middle < above)`，ensures `is_lower_bound` 里没有 u32；没有一条义务说 `middle + 1 <= u32::MAX`。
- 反例形状：把那一行的 `<= u32::MAX` 删掉（副本攻击行 `attack-lower-bound-without-u32-bound`）。按代码推，Verus 照样 0 错误：说明「证明了 u32 范围」那半句没有任何会断的义务撑着。WGSL 里 `middle + 1u` 确实不溢出（`middle < above <= record_count`），这是推得出的真话，不是被证明的那一句。必红表 `gpu-score-counter-needs-record-bound` 罩的是另一个函数 `lemma_every_walk_prefix_fits_the_gpu_counter` 的 u32 界，罩不到这一条。
- 推翻条件：主 agent 跑出这一行判成红（Verus 报证明错、落点在 `lower_bound_of_sorted_records`），这一条撤回。
- 四句：不分辨臂；被判对象看得到（界就在它的 requires 里）；满足「性质说的比 ensures 证的多」；改法「在函数体加 `assert(middle + 1 <= u32::MAX)` 并配一条删界的必红行」碰得到（推的）。

### F. 空真（没打中）

- `gpu_gc_proofs.rs` 八处 requires（G05、G06、G07、G08、G09、G11、G13、G18）：七处由 `witness_gpu_gc_protocol_and_score_premises` 用具体实参调到（Map {7↦1}、区间 9..12 与 10..20、切口 11、`u32::MAX + 1` 进位），G18 由 `witness_gpu_gc_arithmetic_boundaries` 用 `(u64::MAX − 1, u64::MAX, u64::MAX)` 调到。我逐个代入核过实参满足 requires。
- `gpu_gc_capacity_proofs.rs` 唯一的 requires（G28 `per_group > 0`）：见证用 64 调到。
- `gpu_gc_scoring_proofs.rs`：定理与它调的五个引理由见证里三组输入（四条带重复与异盘的表、空表、最长跨度的单条）调到；`records_are_sorted`、`valid_scoring_records`、多重集合相等在见证里逐条 assert 过。
- 试过的形状：找两两矛盾的 requires（`is_lower_bound` 两次对同一表、`first <= past` 与 `past < first` 分支）；`lemma_candidate_window_keeps_every_contribution` 里 `past < first` 那一支是从 requires 推出矛盾的分支，不是 requires 本身矛盾。没找到空真。

## 件 2：固定长度覆盖写四条引理的前提

- `lemma_overwrite_changes_exactly_the_requested_bytes` 的 requires「requires overwrite_fits(old, start, data),」（`crates/singlefs-replay-prefix/src/partial_write_proofs.rs:120`）不承重（推的，待跑副本攻击行 `attack-overwrite-exact-without-fit`）：三条 ensures 都是 `overwritten_bytes` 定义（`Seq::new(old.len(), …)`）的直接展开，起点为负、请求越过文件尾时照样成立。这条引理是 spec 自检（改坏 `overwritten_bytes` 的定义它会断），不是对写路径的约束。必红表给它的只有「ensures 加 false」一行（空真检查），没有一行改坏定义来证它钉得住定义。
- 另三条的前提都承重：同长度、不相交、整单元覆盖各有一条删前提的必红行（`overwrite-last-needs-equal-lengths`、`overwrite-commute-needs-disjoint-ranges`、`overwrite-whole-needs-coverage`），按代码推删掉都会断。
- 是不是产品写路径真满足的前提：四条引理都是 proof fn，产品不调；字节模型限定在「/// 固定长度文件内的写入；扩展文件、补洞与 I/O 不在这个字节模型中。」（`crates/singlefs-replay-prefix/src/partial_write_proofs.rs:108`）。`lemma_whole_unit_overwrite_does_not_depend_on_old_bytes` 要「overwrite_fits(left, write_start as int, data),」（`crates/singlefs-replay-prefix/src/partial_write_proofs.rs:174`），越过旧文件尾的写（追加、扩展）不满足它，所以分类核在追加写上给出 `OldCoveredWhole`／`NewCoveredWhole` 的那几格不在这条引理罩着的范围里。README 新节写了这条边界（「不代表 core 的缓冲区拷贝、扩展文件……已经精化到此模型」），没有说成证过了。
- 四句（对第一条）：不分辨臂；看得到；满足「前提」那一分句里「requires 不是承重前提」的读法；改法「删掉这条 requires」或「加一行改坏 `overwritten_bytes` 定义的必红行」都碰得到（推的）。

## 件 3：两份新测试

- 共用算式：两份参照都另写在测试里、用 `u128`，不调被测函数，没有共用代码。但参照逐条重述的就是 ensures 的口径，两边共用同一组假设：
  - 对齐输入只算一遍：「let input_bytes = u128::from(input).max(1).div_ceil(4) * 4;」（`crates/singlefs-replay-prefix/tests/gpu_gc_capacity_matches_independent_reference.rs:6`）再与 `gpu_chunk_fits` 比。产品真占用把输入的暂存另算（件 1 B），这一点两边同错也测不出；它要由 `crates/singlefs-background-gpu` 自己的测试管。
  - `aligned_input_size(0) = 4`、`decoded` 只认判定字 1：两边写法相同，是规格本身，测试不当它的校验路子。
- 仲裁测试把被测函数的输出喂回下一轮输入（「counter = step.uncertain_rounds;」，`crates/singlefs-replay-prefix/tests/gpu_gc_arithmetic_matches_wide_integer_reference.rs:130`），但同一轮先断言了 `uncertain_rounds` 等于参照，喂回的值已被核过，不算共用。
- 九个边界值（「const EDGES: [u64; 9] = [」，`crates/singlefs-replay-prefix/tests/gpu_gc_arithmetic_matches_wide_integer_reference.rs:3`）：缺 `u64::MAX − 2`、`u64::MAX − 1`。`checked_gpu_input_bytes` 在这两点向上取整溢出、应交 None；表里只有 `u64::MAX − 3`（交 Some）与 `u64::MAX`（交 None），一个「只在 input == u64::MAX 时交 None」的错版本过得了这份测试。Verus 的 ensures 罩着这两点，所以只是测试自己的缺口。容量测试的 `EDGES` 同样缺这两点。
- 容量测试第一条把 `fixed_bytes` 固定 16、`bytes_per_entry` 固定 40、`dispatch_entries` 固定 3，乘加溢出只从 `entries`、`input` 那一侧打到；`largest_piece_matches_exhaustive_search_and_is_maximal` 另扫了 `fixed`、`per_entry` 到 `u64::MAX`，补上了那一侧。
- 打中之后的四句：不分辨臂；被测函数看不到暂存份数；满足「参照与被测是不是共用了算式」那一分句的「共用口径」读法，不是「共用代码」；改法「EDGES 加两点」碰得到边界那一格，碰不到口径那一格。

## 件 4：必红表新加的反例

- 数行（量的，`git diff HEAD -- crates/singlefs-replay-prefix/verus-must-red-worlds.tsv` 的加行按六段计）：加行里六段的数据行 66 行，其中 8 行是改名换锚点，新反例 58 行（gpu-gc 35、gpu-score 15、overwrite-contract 5、overwrite-*-needs 3），另加 5 行 `#` 注释与 5 行空行，合计 68 行加行。正文写「必红表加 68 行反例」把注释与空行算进了反例。
- 逐行核了原文在被改文件里是否恰好一次、改坏之后按代码推断不断、断在哪个函数（推的，没起 Verus）：G01–G30、两条审计、三条见证、15 条 gpu-score、8 条 overwrite 没有找到不红或落点与第六段不同的行。几处要留意的：
  - `gpu-gc-audit-score-capped-at-two` 的落点是两个函数（G11 与 `lemma_search_prefix_matches_the_walk`），GPU_GC_SELFTEST.md 还写只落在 G11（件 6）。
  - `gpu-score-search-skips-record` 落点含 spec fn `searched_slot_sum`：改成 `past - 2` 后 `decreases past - first` 在 `past = first + 1` 时降到负数，终止性义务断在 spec fn 里，第六段写对了。
  - `gpu-score-window-too-narrow` 把 65535 改 65533 才红。推得出：跨度 ≤ 65535 时窗口下沿取 `start − c` 只要 `c ≥ 65534` 就不漏（起点 ≤ `start − 65535` 的记录尾端 ≤ `start`，贡献零），所以改成 65534 照样验得过（副本攻击行 `attack-window-one-slot-narrower`，推的，待跑）。这不是错，说明 65535 这个常数不是被证明钉住的那个界，「窗口过窄」这一类反例只罩住窄两格以上。
- 哪些 ensures 没有反例罩着：新行每个函数一行，不是表头前 80 行那样每条 ensures 一行。exec 函数的 ensures 都把返回值写成等式，函数体的变异会被等式那一条抓到，派生的几条（例 G18 的两条分区 forall、G01 的互斥与「三种决策」那一条）推得出、单独改坏只能改 ensures 本身，没有行罩；proof fn 里只罩一条的：`lemma_two_word_slot_order_is_u64_order` 第一、三条，`lemma_two_word_span_addition_carries_exactly_once` 前三条，`lemma_live_slot_intersection_bounds` 后三条，`lemma_released_and_foreign_records_do_not_score` 第一条（第三条由 `gpu-gc-audit-score-capped-at-two` 罩着），`lemma_every_walk_prefix_fits_the_gpu_counter` 的 `0 <= … <= visited * 65535` 那一半（删 u32 界那一行只断 `< u64::MAX` 那一半），以及件 2 那条覆盖写引理。这是事实清单，表头这一节写的是「G01–G20 各一条必红变异」，没有违反它自己写的口径。

## 件 5：改名那一批

- 逐块读了 `changes.patch` 里 `lib.rs`、`compaction_plan_proofs.rs`、`file_range_proofs.rs`、`unlink_proofs.rs`、`partial_write_proofs.rs` 改名的减行与加行：只换标识符（`i→step_index`、`j→other_step_index`、`k→tail_step_index`、`q→quotient`、`r→remainder`、`ws/we/us/ue→write_start/write_end/unit_start/unit_end`、去 `a_` 前缀、`(a, b, n)→(offset_in_unit, offset_in_write, length)`），requires、ensures、decreases、循环不变式的结构与比较符一个没动。核了新名字在同一作用域里没有撞名或捕获（`unlink_proofs.rs` 里 `quotient`、`remainder` 只在 `lemma_deletion_batch_rewrites_only_the_cut_path` 里出现，`step_index` 只作量词变量与 `lemma_continued_plan_equals_the_rest` 的形参）。
- 旧名全仓（`research/` 外）残留：`git grep` 九个旧名，只有一处——必红表第 198 行的变异名「证明负例·a_segment_holding_a_pinned_slot_is_not_a_sound_choice 的函数体不能推出 false」（`crates/singlefs-replay-prefix/verus-must-red-worlds.tsv:198`）。第六段落点已改成新名，判法只比落点，所以不会红；变异名指着一个已不存在的函数名，是名字对不上（另一处在 `records/` 里，是记录，不改）。
- `later_record_of_the_same_publish_is_readable` 改名后在 `lib.rs` 里是定义 1 处加引用 5 处（`grep -n` 量的：278、356、379、1733、2161、3130），其中 1733 在 requires 里、3130 在 ensures 里、2161 在证明体的 `assert forall … implies` 里。

## 件 6：三份说明文件里与代码对不上的数

每一条都带核它的命令；「量的」指对源文件现跑的 `ls`／`grep -c`，不是跑出来的测试数。

1. README 源文件数：第 7 行的节标题是「## 八份源文件各负责什么」，节里写「核的入口文件为 lib.rs，另七份文件通过模块引入。」（`crates/singlefs-replay-prefix/src/README.md:20`）。量的：`ls crates/singlefs-replay-prefix/src/*.rs | wc -l` 得 10，`grep -c '^pub mod' crates/singlefs-replay-prefix/src/lib.rs` 得 9。表里没有 `gpu_gc_capacity_proofs.rs` 与 `gpu_gc_scoring_proofs.rs` 两行（`grep -n '^| \[' README.md` 列出八行）。应是「十份」「另九份」并补两行。
2. 「//! GPU GC 的主机端纯核与着色器算术模型。20 项核心性质见 src/README.md。」（`crates/singlefs-replay-prefix/src/gpu_gc_proofs.rs:1`）：README 只有一行链接，20 项的清单在 `GPU_GC_PROOFS.md`。指错了文件。
3. GPU_GC_SELFTEST.md 描的是加 G21–G30 与打分证明之前的那棵树：它写「170 个验证项重新执行，义务合计 186，0 错误」（`crates/singlefs-replay-prefix/src/GPU_GC_SELFTEST.md:23`），登记表今天是「crate crates/singlefs-replay-prefix 208」（`.claude/gate.d/verus-pins.tsv:48`）。按函数数推（推的）：186 + 容量模块 11（十个产品核与一个见证）+ 打分模块 11（九个 proof fn 与两个带 decreases 的 spec fn）= 208，对得上，说明 SELFTEST 没有记容量与打分两个模块的任何自测结果，而标题与正文没有写这一点。同一份末尾「八个直接调用的产品核与十二个抽象模型的区分保持不变。」（`crates/singlefs-replay-prefix/src/GPU_GC_SELFTEST.md:65`）只数了 G01–G20。
4. 主机模块测试数：GPU_GC_PROOFS.md 写「后台包命令点名四个模块的 56 个测试，不开启真 GPU 特性。」（`crates/singlefs-replay-prefix/src/GPU_GC_PROOFS.md:106`），SELFTEST 写「56 通过，0 失败、忽略；其他模块的 112 项未纳入本次范围」（`crates/singlefs-replay-prefix/src/GPU_GC_SELFTEST.md:31`）。量的：四个模块各只有一个 `mod tests`，`grep -c '#\[test\]'` 得 `two_kernel_reconciliation` 7、`memory_limit_and_chunks` 36、`live_slot_kernels` 23、`region_record_kernels` 16，合计 82。实际过滤出几条要主 agent 列项（「要主 agent 跑的命令」第三条）；`crates/singlefs-background-gpu` 在别的会话手里还在改，56 可能是当时的数。
5. 「打分截顶的证明错误落在 G11，漏记 CPU 调用落在 G04」（`crates/singlefs-replay-prefix/src/GPU_GC_SELFTEST.md:35`）：必红表今天这一行第六段是 `landing=lemma_released_and_foreign_records_do_not_score,lemma_search_prefix_matches_the_walk`，打分模块加进来之后落点多了一个函数。
6. GPU_GC_PROOFS.md 没提第二份新测试 `gpu_gc_capacity_matches_independent_reference.rs`（`grep -rn gpu_gc_capacity_matches_independent_reference` 在仓里 `research/` 外零命中），它写的「新增四个独立核对测试」只是第一份的 4 条。
7. G30 的「产品直接调用」（件 1 D）。

对得上的（核过）：`GPU_GC_PROOFS.md` 的「调用表内八个产品核」（表里层次写「产品核」的恰好 G01、G14–G20 八行，八个函数各有产品调用点）；「共 24 条」（G01–G20 二十行、两条见证、两条审计）；「八处 requires」；「模块共 26 条义务」（推的：9 个 exec 函数、13 个 proof fn、1 个带 decreases 的 spec fn、3 个常量）；README 新节「必红表新增五条契约反例及三条删除关键前提的反例」；`--rlimit 10` 与登记表 `rlimit 10` 一致。

说明文件里的历史叙述（不是数，单列交主 agent）：GPU_GC_SELFTEST.md 整份是一轮的经过（「本轮自测发现并修复了两处」在第 3 行，任务号、「修复前」、「首次尝试因内存上限中断」），按共用约束「一轮的经过、量到的数与用户原话进 records/」（`.claude/agent-common.md@写`）该进 `records/`；它不叫 README，不在 comment-history 格的字面射程里。`GPU_GC_PROOFS.md` 第 75 行「继续允许原实现允许的零跨度」、第 77 行「正常输入行为不变」，README 第 152 行「新增五项证明义务」，是改动经过的说法（推的判断，射程以门禁 doc-text 的 comment-history 格为准）。

## 检查单八格

### 走不到的理由二次查实：不成立

被判文件里 exec 函数返回 `Option` 的每一处，追到 `crates/singlefs-background-gpu/src/` 的调用点：

| 核 | None 在什么输入上 | 调用方怎么接 | 判 |
|---|---|---|---|
| `checked_live_slot_totals` | 两份累计任一加过 `u64::MAX` | 「.expect("活槽数之和不超过记录数乘 65535，装得进 u64");」（`crates/singlefs-background-gpu/src/live_slot_kernels.rs:379`） | 理由不成立，见下 |
| `decoded_gpu_live_slots` | 判定字不是 1 | 转成 `UnrecognizedVerdictWord`，再「.map_err(GpuFailureCause::KernelLeftAnUnrecognizedVerdictWord)?;」（`crates/singlefs-background-gpu/src/background_gpu.rs:2535`） | 成立，走出错 |
| `checked_gpu_input_bytes` | 输入长度 > `u64::MAX − 3` | 「.expect("四字节对齐后的输入长度装得进 u64"),」（`crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:106`） | 成立，主机输入长度到不了 |
| `checked_gpu_chunk_bytes` | 乘加溢出 | `checked_bytes_held_by_a_chunk` 交 None，外层「.expect("GPU 缓冲占用的乘加装得进 u64")」（`crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:292`） | 成立（推的：段先经 `gpu_chunk_fits` 判过才算占用） |
| `largest_gpu_piece_input` | 连最小一片都装不下 | 「let Some(largest_piece_bytes) = largest_piece else {」（`crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:547`）交 `SmallestPieceLargerThanTheRoom` | 成立，走出错 |
| `checked_gpu_memory_hold` | 有未完提交、加溢出或超上限 | 「.expect("本包在这张卡上占的缓冲超过了上限或 u64：切批与记账必须一致"),」（`crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:699`），之前已断言没有未完提交 | 成立（主机自己的账，超了是切批的缺陷） |
| `checked_gpu_memory_release`、`gpu_bytes_after_submission_wait` | 放的比记的多 | `expect`（第 709、731 行） | 成立（主机自己的账） |
| `checked_gpu_mapping_table_bytes` | 60 × 条数 + 16 × 落点溢出 | 「.expect("映射表的总字节数与最大绑定量装得进 u64");」（`crates/singlefs-background-gpu/src/region_record_kernels.rs:556`） | 成立，条数到不了 3 × 10¹⁷ |
| `extended_gpu_chunk_input` | — | 没有产品调用点 | 件 1 D |

打中的那一处：`checked_live_slot_totals` 两边的参数里，`walking`、`searching` 是 GPU 回读解出的数（「let walking = decoded_live_slot_outcomes(&walking_bytes)」，`crates/singlefs-background-gpu/src/background_gpu.rs:2803`），`decoded_gpu_live_slots` 只查判定字、不查数的大小。`expect` 写的理由「活槽数不超过记录数乘 65535」只对算对了的核成立；两份核加 CPU 参照这套仲裁本来就是因为不信 GPU 的结果。

- 历史（推的，没跑）：显存上限小到 `live_slot_plan_on_a_gpu` 把记录表切成至少两片；第一片里某段两份核交回正常的数（≥ 1）；第二片里同一段有一份核回读成判定字 1、高低两字全 `0xFFFFFFFF`（卡出错、显存位翻转或着色器缺陷）。`live_slots_summed_over_the_record_slices` 在「*summed = singlefs_replay_prefix::gpu_gc_proofs::checked_live_slot_totals(」（`crates/singlefs-background-gpu/src/live_slot_kernels.rs:376`）拿到 None，第 379 行 panic，而不是把这一段当分歧交 CPU 参照或当这一批出错。只有一片时 `checked_live_slot_totals(0, 0, w, s)` 不会溢出，错的数照常进仲裁。
- `grep -n catch_unwind crates/singlefs-background-gpu/src/*.rs` 命中 6 处：`background_gpu.rs` 第 1511、2116 行在生产段（开卡、轮询），第 9410、9914、9925 行在第 4309 行起的 `mod tests` 里，`gpus_of_this_host.rs` 第 1173 行在它第 1029 行起的 `mod tests` 里；生产段两处都不在选段打分的提交路径上（推的，按调用链读）。
- 四句：不分辨臂；被判系统看得到判别它的东西（回读的数在手里，界 `记录数 × 65535` 也算得出）；满足检查单「None 那一支各自在什么输入上出现、调用方怎么接」那一句；改法「None 交成一种 `GpuFailureCause`（与判定字不认得同一条路）」或「回读后按记录数 × 65535 判界，超界当分歧」都碰得到（推的，只在我这里推过，被攻过零轮）。

### 类型与名字：不成立

- 规则：「- 同一种底层类型承载不同含义（逻辑地址、物理地址、代号；字节数、块数），各给一个 newtype。」（`.claude/singlefs-ai-sop/rules/code-discipline.md:128`）。
- 新 exec 函数把字节数与条数放在同一个裸 `u64` 参数表里：「pub fn gpu_chunk_fits(input: u64, entries: u64, capacity: &GpuChunkCapacity) -> (fits: bool)」（`crates/singlefs-replay-prefix/src/gpu_gc_capacity_proofs.rs:28`），`checked_gpu_chunk_bytes(input, entries, per_entry, fixed)`、`extended_gpu_chunk_input(previous, additional, entries, …)` 同样；`GpuChunkCapacity` 五个 `u64` 字段里 `dispatch_entries` 是条数、其余是字节。「pub fn gpu_record_is_in_scan_region(same_device: bool, released: bool, slot: u64, span: u64, start: u64, end: u64) -> (read: bool)」（`crates/singlefs-replay-prefix/src/gpu_gc_proofs.rs:316`）里 `span` 是槽数、另三个是槽号，两个布尔相邻。`checked_live_slot_totals(walking, searching, next_walking, next_searching)` 四个同型参数，核内交换由 G14 那条必红行抓，调用方交换 Verus 管不到（产品那一处次序我核过是对的）。
- 同一个函数跨两种含义：`next_gpu_record_slice_end` 名字说的是记录片，产品既拿它切记录位置（`crates/singlefs-background-gpu/src/live_slot_kernels.rs:303`），也拿它切一条输入的字节（`crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:561`）。
- 布尔名：`same_device`、`released`、`pending`、`completed`、`agree`、`fits`、`read`，结构字段 `confirmed`、`ask_cpu`，不以 `is_`／`has_` 起头（转述 `.claude/rules/code-discipline-by-review.md` 第 37 行「留下布尔名以 is_、has_ 开头」那一条）。
- 外层产品用了 newtype（`GpuMemoryInBytes`、`InputLengthOnTheHostInBytes`、`EntryCountOfAChunk`），在调用点 `.0` 拆开再交给核，混淆的机会正好落在拆开那一步。Verus 收 newtype 参数（推的，没在这份代码上试）。

### 不兼容旧代码与旧格式：成立

加行里没有认旧版本的分支、旧布局号、迁移代码与只为旧数据放宽的读法（逐份读了三份新证明文件与两份测试）。说明文件里的「原实现」「修复前」是历史叙述，归件 6 末段，不是兼容代码。

### 不新建判定库：成立（带一条说明）

`.rs` 与必红表的加行没有新的判定存放处。GPU_GC_SELFTEST.md 写「证明、反例和入口空真检查通过项目提供的 prune_server_client_selftest.py --with-temporary-service 使用临时判定库」（`crates/singlefs-replay-prefix/src/GPU_GC_SELFTEST.md:39`）：临时服务在格自己的目录里、跑完删（`scripts/prune-server/prune_server_client_selftest.py` 文件头），不留存放处，不违反「判定库只有一份」。说明：规则许临时服务的用途是「自证与压测一律对自己起的临时服务，格完即删，不写真服务，也不拿真任务当测试。」（`.claude/rules/verification.md:56`），SELFTEST 拿它跑的是门禁判法本身（证明、必红、空真），图的是「不复用既有判定」；这几趟的判定没进那一份库，门禁照样要在真服务上现判一次。这一层算不算「自证」，交主 agent 判。

### 所有任务多机可用：成立

三份说明文件里的复跑命令不写主机名、地址与本机路径，Verus 路径经 `VERUS` 变量从登记表取，路径都从仓根相对写。另：SELFTEST 写「最终在第二台的卡 1（0000:05:00.0）完成」（`crates/singlefs-replay-prefix/src/GPU_GC_SELFTEST.md:37`），这是第二台上一张卡的 PCI 地址，不在共用约束私有值清单列的几类里，但是单台的设备信息，交主 agent 判要不要换成「第二台的一张卡」。

### 测试优先多线程：成立

两份测试是纯算术循环：算术测试最大一层 9⁴ × 9 ≈ 6 × 10⁴ 次调用，容量测试最大一层 41 × 5 × 5 × 3 × 2 × 7 ≈ 4.3 × 10⁴ 组、每组穷举至多 40 个输入（推的，按循环界算，没量挂钟）。没有写死或覆盖线程数。

### 带 --item 的命令与项：推不出

- 说明文件点名的必红项名（`gpu-score-binary-skips-entry`、`gpu-score-window-too-narrow`、`gpu-score-search-counts-released`、`gpu-score-permutation-needs-multiplicity`、`gpu-gc-G01`–`G04`、两条 `gpu-gc-audit-*`、三条 `overwrite-*-needs-*`）在必红表里逐个 grep 得到；写法 `must-red:<变异名>` 与判法文件头「--item <项>（可给多次，取并集）只判点名的项」（`scripts/gate/verus-proof-check.py:136`）对得上；`--rlimit 10` 与登记表的 rlimit 行一致。
- 对不上的：命令都写成「task-server/task submit --manual -- python3 scripts/gate/verus-proof-check.py must-red-worlds . \」（`crates/singlefs-replay-prefix/src/GPU_GC_PROOFS.md:31`）这种人手交的形状，不带范围声明，也不经门禁 `.claude/gate.d/formal-verification.sh --item`。从会话里交会被服务当场拒：「manual-from-session｜提交时拒｜带了 --manual，而进程树上读得到提交它的会话」（`task-server/task_server_lib.py:2093`）。人手交是这几条命令写明的用途，所以不算写错，但主 agent 与子 agent 都不能照抄。
- 项名在 `--list-items` 里列不列得出，没跑（「要主 agent 跑的命令」第二条）。

### 新崩溃流多罩了什么：成立

改动里没有新加崩溃点重放流、崩溃枚举用例或改到崩溃枚举的代码（十四份文件都在 `crates/singlefs-replay-prefix/` 下，这个包只依赖 `vstd`）。

## 事实表 facts.md 写错的地方

按 facts.md 的小节名指，不写它的行号（它是背景材料）。

1. 「replay-prefix/src/GPU_GC_PROOFS.md」第 1 样说它「列 G01–G30 各条性质」：覆盖清单表只有 G01–G20 二十行；G21–G30 只在第 5 行一句话里列了名目。`grep -oE 'G[0-9]{2}' GPU_GC_PROOFS.md | sort -u | wc -l` 得 22（G01–G20 加 G21、G30 两个端点），与 facts 自己第 2 样的「共 22 个」一致、与第 1 样矛盾。
2. 「replay-prefix/src/gpu_gc_capacity_proofs.rs」第 1 样「共 11 个 exec 函数与一个边界见证」：产品核 10 个（G21–G30），加见证一共 11 个 `pub fn`；写成「11 个加一个见证」多算一个。
3. 「replay-prefix/src/lib.rs」第 1 样「共 6 处引用」：是定义 1 处加引用 5 处；「ensures 里的被调名随改名变了（L3130、L2161）」：2161 在证明体的 `assert forall … implies` 里，不是 ensures；1733 在 requires 里没提；同一节「不动 requires/ensures/decreases 的文字」与第 4 样「requires/ensures/decreases 的行 0 条」不对：1733（requires）与 3130（ensures）这两行的文字随改名变了，只是结构没变。第 4 样把改名的 `later_record_of_the_same_publish_is_readable` 列成「新加 spec fn」，它是改名。正文第 5 件「定义与 6 处引用」跟着错了。
4. 「replay-prefix/src/partial_write_proofs.rs」第 3 样「生产段新加的条目名 1 个：fn classify_partial_write_unit」：它是只改了形参名的旧函数；真正新加的是 `overwrite_fits`、`overwritten_bytes` 两个 spec fn 与四个引理一个见证（都在 `verus!` 里）。
5. 「replay-prefix/src/unlink_proofs.rs」第 4 样「新加 proof/spec fn：proof fn lemma_continued_plan_equals_the_rest」：旧函数，只改了形参名 `i → step_index`。
6. 「replay-prefix/src/compaction_plan_proofs.rs」第 4 样把 `one_movable_allocation_is_planned_and_not_reported_emptied` 算进改到的函数：它只出现在 diff 块的上下文行里，没改。
7. 「replay-prefix/verus-must-red-worlds.tsv」第 1 样「L198 一行证明负例名去 a_」：变异名没去 `a_`（第一段还是 `a_segment_…`），改的只是第六段落点；同一样「新增 68 行」「（overwrite-*、partial-*）」：68 行加行里数据行 58、注释 5、空行 5，新反例里没有 `partial-*` 行（`partial-*` 那四行是随改名换锚点的旧行）。正文第 4 件「必红表加 68 行反例」跟着错了。

对得上的（核过）：「GPU_GC_SELFTEST.md」的「共 8 个不同的 G 编号」（`grep` 得 8）、「README.md」第 5 样关于「八份／另七份」与 10 份的线索、`gpu_gc_scoring_proofs.rs`「含 7 处 decreases」、两份测试的 4 条与 5 条、`gpu_gc_proofs.rs` 新名字 14 个、各份 `as` 转换的位置。

## 要主 agent 跑的命令

1. 副本里三条攻击行加一条对照行的必红世界判法（对临时服务跑，攻击行的结局不写进真服务那一份库）：
   - `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=副本必红世界表里三条攻击行与一条对照行; 范围=4 项; 为什么=核攻方腿报的三处证明前提不承重，对照行确认判法在副本上照常抓得到' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/replay-prefix-backfill-code-r1-attack/copy -- python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/gate/verus-proof-check.py must-red-worlds . --item must-red:attack-lower-bound-without-u32-bound --item must-red:attack-window-one-slot-narrower --item must-red:attack-overwrite-exact-without-fit --item must-red:gpu-score-window-too-narrow`
   - 在哪起：`/home/fy5090/.local/share/singlefs/work/replay-prefix-backfill-code-r1-attack/copy`（副本，已在 `.deliverables` 写成「交接」）。
   - 看哪几行：`TASK_SERVER_RESULT` 预期 `exit=1`（攻击行不红就是判法判红）；`task-server/task log <号> --all` 里三条 `attack-*` 各是「没报证明错、0 errors」那一类判红句即件 1 E、件 2、件 4 三处坐实；对照行 `gpu-score-window-too-narrow` 要是「抓到」。对照行没抓到则整趟作废（副本或工具链有问题）。三条攻击行里哪条报了证明错且落点与第六段相同，那一条撤回。
2. 门禁 formal-verification 列项，核说明文件点名的项都列得出：
   - `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 formal-verification 只列项; 范围=列项、不起 Verus; 为什么=核 GPU_GC_PROOFS.md 与 README 复跑命令里点名的 must-red 项在列项里' task-server/task submit -- bash .claude/gate.d/formal-verification.sh --list-items`
   - 在哪起：仓根。
   - 看哪几行：`TASK_SERVER_RESULT` 的 `exit=0`；`task-server/task log <号> --all` 里 `must-red:gpu-gc-G01` 到 `must-red:gpu-gc-G30`、`must-red:gpu-score-*` 15 项、`must-red:overwrite-*` 8 项各一行。范围声明闸若认为列项不该带声明，照拒绝说明去掉再交。
3. 列四个主机模块实际过滤出的测试数，核「56 个测试」：
   - `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-background-gpu 的 lib 测试目标里四个模块的测试，只列项不跑; 范围=1 个测试目标; 为什么=核 GPU_GC_PROOFS.md 与 GPU_GC_SELFTEST.md 写的 56 与源码里 82 个 test 属性对不上' task-server/task submit -- cargo test -p singlefs-background-gpu --lib two_kernel_reconciliation::tests -- memory_limit_and_chunks::tests live_slot_kernels::tests region_record_kernels::tests --list`
   - 在哪起：仓根。
   - 看哪几行：`TASK_SERVER_RESULT` 的 `exit=0`；日志末尾 libtest 打的「N tests, 0 benchmarks」那一行的 N（我按源码推 82）。

主 agent 自己可直接复核、不用交任务的只读命令：`grep -rn 'extended_gpu_chunk_input' crates/ --include=*.rs`（件 1 D）；`ls crates/singlefs-replay-prefix/src/*.rs | wc -l` 与 `grep -c '^pub mod' crates/singlefs-replay-prefix/src/lib.rs`（件 6 第 1 条）。

## 没打中的形状

- WGSL 与模型之间：读了两份活槽着色器。打分模型的比较键（先盘号、再槽号）、二分的 `below + (above − below) / 2`、窗口下沿的饱和减、逐槽「不小于段首且小于段尾」、释放记录跳过、不按盘号再筛，与 `live_slot_kernel_searching_sorted_records.wgsl` 逐处对得上；排序键 `(record.device.0, record.slot.0, *position)` 满足 `records_are_sorted`，下标表是 `0..n` 的排列，满足多重集合前提；`DeviceIdentity` 是 `u32`、`span_slots` 是 `u16`（「pub span_slots: u16,」，`crates/singlefs-core/src/allocator.rs:76`），`valid_scoring_records` 的跨度界由类型保证。没有在模型里的 WGSL 运算：`wide_minus_or_zero` 的借位（`live_slot_kernel_searching_sorted_records.wgsl` 第 68 行起）、逐条走那份核用低 32 位相减算交集长度（「let slots_in_both = slot_past_both.x - first_slot_of_both.x;」，`crates/singlefs-background-gpu/src/live_slot_kernel_walking_every_record.wgsl:95`）。这两处说明文件写了「WGSL 的实际双字操作……仍未证明」，没有说成证过了，所以不算打中；手算两处在跨度 ≤ 65535、槽号 < 2⁶⁴ 时都对。
- `checked_gpu_mapping_table_bytes` 的 56／16／4：与 `region_record_kernel_searching_sorted_indexes.wgsl` 里 `SearchMappingEntry` 十四个 u32、`SearchLocationItem` 四个 u32、下标 u32 对得上；常数与着色器结构体之间没有机器绑定，改了结构体证明不会红（缺口，说明文件没声称绑定）。
- 判定字：两份着色器各自写 `1u`（「const VERDICT_WORD_COUNTED: u32 = 1u;」，`crates/singlefs-background-gpu/src/live_slot_kernel_walking_every_record.wgsl:44`），说明文件说「主机引用同一份常量」只讲主机一侧，属实。
- 仲裁：产品调用点「let step = singlefs_replay_prefix::gpu_gc_proofs::reconcile_gpu_entry(」（`crates/singlefs-background-gpu/src/two_kernel_reconciliation.rs:148`）的三支与 `update_count`（确认、交 CPU 都删键，暂缓存 1）一致；整批有重复键先拒，对上 G05 的 `first != second`。
- 产品调用点的 requires：Verus 不查产品调用方。`next_gpu_record_slice_end` 两处调用的 `capacity` 分别在 0 时提前交错（`records_per_slice == 0`）与取自 `largest_gpu_piece_input`（≥ 4），`start <= total` 由循环条件保证；`gpu_workgroups_for_entries` 三处传的都是常数工作组大小。没找到违反 requires 的调用。
- 改名捕获：见件 5，没找到。
- 只扫了本轮这十四份与它们在 `crates/singlefs-background-gpu/src/` 里的直接调用点；`singlefs-background-gpu-tests` 与真卡测试没读。

## 这条腿自己的限度

- 没起 Verus。件 1 E、件 2、件 4 里「删掉照样验得过」三处是按代码推的，要主 agent 跑第一条命令坐实；件 4 里「每条新必红行都会红、落点对」也是逐行推的，不是跑出来的（SELFTEST 记的那几趟是别人跑的，旧证据，不当依据）。
- 「走不到」那一格的 panic 历史没造出来跑：要一张会算错的卡或在副本里改回读字，造价高，只写了推得出的触发条件。
- 件 6 第 4 条的 82 是源码里 `#[test]` 的个数，不是 libtest 过滤出的个数。
- 自己提的改法（G30 改说明或删、`checked_live_slot_totals` 的 None 交成出错、`lower_bound` 加 u32 断言与必红行、测试补 `u64::MAX − 2`、`u64::MAX − 1`）只在我这里推过、被攻过零轮。

| 改法 | 修哪一格 | 量过／推的 |
|---|---|---|
| G30 说明文件改成「没有产品调用点」，或删 G30 与它的必红行 | 件 1 D 说法那一格 | 推的 |
| 产品改调 G30 | 件 1 D 缺口那一格（要先让 G30 收按输入折过的上限） | 推的 |
| `checked_live_slot_totals` 的 None 交成 `GpuFailureCause` | 走不到那一格 | 推的 |
| `lower_bound_of_sorted_records` 体里加 `assert(middle + 1 <= u32::MAX)` 并配删界的必红行 | 件 1 E | 推的，待主 agent 跑 |
| 两份测试的 EDGES 加 `u64::MAX − 2`、`u64::MAX − 1` | 件 3 边界那一格；碰不到口径那一格 | 推的 |
| README 改「十份」「另九份」、补两行；SELFTEST 挪进 `records/` 或写明只覆盖 G01–G20 | 件 6 | 推的 |

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳。
- 没往统一任务服务交任务；判决要引的数没复跑（命令在「要主 agent 跑的命令」）。副本上跑出来的结局（如果主 agent 跑了）是副本的数，不算入库装置上的数。
- 没读禁读清单里的另一条腿报告，没拿别的轮的判决当依据。
- 草稿目录里留了：仓副本 `copy`（191M，`du -sh` 量的，`.deliverables` 写成「交接」，主 agent 跑完第一条命令后删）、`attack-rows.tsv` 与 `verus-must-red-worlds.before-attack-rows.tsv`（备份）、`tsv-diff.txt`、`partial.diff`、`unlink-changed.txt`、`minimal-copy.txt`（读材料时的中转文件）。没建编译目录。

引文核对：`python3 scripts/agents/cite-check.py research/prompts/replay-prefix-backfill-code-r1-opus-output.md` 末行「✓ 核了 45 处引文、13 处裸行号，对上 45 处，没判 13 处（其中裸行号没有引文可比 13 处；逐处列在上面）」（这一行写进报告之前跑的）。
