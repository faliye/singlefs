# GPU GC 形式化验证的覆盖与边界

入口是 [gpu_gc_proofs.rs](gpu_gc_proofs.rs)。这些性质补充 GPU GC 的仲裁、计数、算术与准入；已有根环、旧读者、回退和崩溃协议证明不重复计入。G01–G20 各有独立契约和必红变异，常量、递归终止与见证不占这 20 项的名额。

容量与资源记账的 G21–G30 位于 [gpu_gc_capacity_proofs.rs](gpu_gc_capacity_proofs.rs)：整批准入、最大输入、收紧上限、预留、释放、等待完成后释放、失败折半、派发覆盖、映射表占用及追加输入准入。这些是产品直接调用的可执行核；并不证明外层资源账本所有调用都符合协议。

## 两份打分算法的归纳证明

[gpu_gc_scoring_proofs.rs](gpu_gc_scoring_proofs.rs) 的 `theorem_sorted_search_equals_full_record_walk` 将二分查找、候选窗口、逐槽计数和排列组合起来：对任意长度不超过 `u32::MAX` 的合法记录序列，排序后的搜索算法与原始无序表逐记录求交的精确总和相等。重复记录保留重数。证明不枚举固定条数、固定槽号或固定循环次数。

证明链包括：

- `lower_bound_of_sorted_records` 对二分循环状态 `(below, above)` 归纳，验证前缀/后缀不变量、下标界限与严格递减量，得到完整下界性质；下界不是输入假设。
- `lemma_candidate_window_keeps_every_contribution` 从两端的下界性质及最大跨度推导：窗内全是目标设备，窗外贡献为零。下界使用饱和减法，覆盖从段前伸入的记录。
- `lemma_counting_slots_equals_interval_intersection` 对跨度归纳，从逐槽成员判定推导交集长度。
- `lemma_search_prefix_matches_the_walk` 对已访问记录数归纳，证明裁剪窗口与整表扫描的前缀对应。
- `lemma_permuting_records_preserves_the_walk` 以多重集合相等为排列条件，利用已验证的可交换 fold 引理证明排序保持总和。
- `lemma_every_walk_prefix_fits_the_gpu_counter` 证明每个扫描前缀的值介于 0 与 `已访问条数 × 65535`，因此装得进 u64。没有假设总分不超过段长。

前提是排序表按 `(device, start)` 非降、排序保持整个记录的多重集合、记录起止有效且跨度不超过 65535、记录数装得进 u32、段为有效的 u64 非空半开区间。非空见证含重复、异盘、释放及跨段记录；另有空表见证。

这是**算法模型的机器检查证明**，不是 WGSL 源码的直接验证。二分模型以递归表示每一次循环状态转移，使用数学整数并证明中间值落在 u32 范围内；逐槽模型使用完整整数槽号。WGSL 的实际双字操作、排序索引解引用、缓冲区布局与派发到该模型的精化仍未证明。主机的 `sort_by_key` 确实建立上述排列与有序前提，也仍须另证。不得把此定理称为整个 GPU GC 已验证。

复跑算法模块（`VERUS` 取下文固定工具路径，提交后核对并确认任务）：

```sh
task-server/task submit --manual -- "$VERUS" --crate-type=lib --edition=2021 \
  --crate-name singlefs_replay_prefix --no-cheating --rlimit 10 \
  --verify-only-module gpu_gc_scoring_proofs crates/singlefs-replay-prefix/src/lib.rs

task-server/task submit --manual -- python3 scripts/gate/verus-proof-check.py must-red-worlds . \
  --item must-red:gpu-score-binary-skips-entry \
  --item must-red:gpu-score-window-too-narrow \
  --item must-red:gpu-score-search-counts-released \
  --item must-red:gpu-score-permutation-needs-multiplicity
```

完整反例清单为 `verus-must-red-worlds.tsv` 中 `gpu-score-` 开头的各行。实现反例覆盖二分跳项/错误上界、逐槽偏移、窗口过窄、释放误计、跳过记录；前提反例覆盖有序、合法跨度、重数、排列和计数上界；另外检查归纳步和见证。编译失败、类型错误或只有求解超时都不能计为抓住反例。

## 覆盖清单

“产品核”表示产品直接调用 Verus 验证的同一份可执行函数。“模型”表示抽象状态或算术定理，到 Rust 容器、WGSL 或设备实现的精化仍未证明。

| 编号 | 性质与被验入口 | 对应 GPU GC 路径 | 层次 |
| --- | --- | --- | --- |
| G01 | 一致才确认，首轮分歧暂缓，第二轮交 CPU，三种决策互斥且计数不溢出。`reconcile_gpu_entry` | `two_kernel_reconciliation::reconcile_the_two_kernel_outcomes` | 产品核 |
| G02 | 连续两次分歧恰好触发一次 CPU 判定，之后重新计数。`lemma_two_disagreements_trigger_exactly_one_cpu_judgment` | 双核仲裁 | 模型 |
| G03 | 一次一致打断连续分歧，不能累计隔轮分歧。`lemma_agreement_restarts_the_disagreement_history` | 双核仲裁 | 模型 |
| G04 | GPU 批失败不改状态；成功轮次精确更新计数与 CPU 调用次数。`lemma_failed_batch_keeps_all_counts_and_cpu_calls` | `reconcile_unless_the_batch_failed` | 模型 |
| G05 | 不同条目的更新可交换，重排后计数表和 CPU 调用总数相同。`lemma_distinct_entry_updates_commute` | 按条目键保存的计数表 | 模型 |
| G06 | 稀疏表只保存一个未决轮次，未出现的条目不变。`lemma_count_updates_preserve_the_sparse_counter_invariant` | 同上 | 模型 |
| G07 | 区间贡献非负、对称、不超过任一跨度，相邻区间贡献零。`lemma_live_slot_intersection_bounds` | 逐记录求交的打分核 | 模型 |
| G08 | 段切开后的贡献可加，跨界记录不漏不重算。`lemma_splitting_a_segment_preserves_live_slot_contributions` | 两份打分核的段边界 | 模型 |
| G09 | 任意有限记录表在任意切口分片，分片求和等于整表。`lemma_record_slices_preserve_the_score` | `live_slots_summed_over_the_record_slices` | 模型，归纳而非定深枚举 |
| G10 | 重复记录保留计数重数，不能误改为槽集合去重。`lemma_duplicate_records_keep_their_multiplicity` | 两份打分核和 CPU 参照 | 模型 |
| G11 | 已释放或异盘记录贡献零；同盘活记录贡献恰为交集长度，严格相交才为正。`lemma_released_and_foreign_records_do_not_score` | 打分核的记录过滤 | 模型 |
| G12 | 双 u32 的高字优先比较等价于全宽槽号比较。`lemma_two_word_slot_order_is_u64_order` | WGSL 槽号比较 | 模型 |
| G13 | 低字加跨度至多进位一次，全宽和合法时高字不溢出。`lemma_two_word_span_addition_carries_exactly_once` | WGSL `slot_past_the_span` | 模型 |
| G14 | 两核分片计数各自累加，任一溢出都拒绝。`checked_live_slot_totals` | `live_slots_summed_over_the_record_slices` | 产品核 |
| G15 | 最小非零四字节对齐输入长度，向上取整溢出拒绝。`checked_gpu_input_bytes` | `memory_limit_and_chunks::input_buffer_bytes` | 产品核 |
| G16 | 预算包含对齐输入、逐条占用和固定占用，乘加溢出拒绝。`checked_gpu_chunk_bytes` | `ChunkFootprint::bytes_held_by_a_chunk` | 产品核 |
| G17 | 扣除其他阶段已占显存后不回绕；未超额时已占加剩余恰为总限。`remaining_gpu_memory` | `ChunkBoundsOfAGpu::with_the_room_left_after` | 产品核 |
| G18 | 下一记录片有进展、不过界，与余下区间不重不漏，避免先加后取 min 的溢出。`next_gpu_record_slice_end` | `live_slot_plan_on_a_gpu` | 产品核 |
| G19 | 同盘、未释放、起点及全跨度在区内才扫描，跨度溢出拒绝。`gpu_record_is_in_scan_region` | `region_record_kernels::record_is_one_the_region_scan_reads` | 产品核 |
| G20 | 只接受 counted=1，拒绝填充字和未知字，保留数值高低两字。`decoded_gpu_live_slots` | `live_slot_kernels::decoded_live_slot_outcomes` | 产品核 |

## 衔接与未覆盖边界

后台 GPU crate 直接依赖 `singlefs-replay-prefix`，调用表内八个产品核。两轮阈值、四字节对齐和 counted 判定字由核定义，主机引用同一份常量。两份 WGSL 仍独立实现，CPU 打分参照也保持独立。

仲裁适配层将真实结果的相等比较传给 G01，只有 `ask_cpu` 分支调用 CPU，只有暂缓分支保存一个未决轮次。`Map<int, nat>` 抽象条目身份与稀疏表；模型不证明 `HashMap`、结果类型的 `PartialEq`、批内键唯一性或外层循环的精化。G04 对应批失败时的提前返回，不能据此宣称整个批处理循环已验证。

G07–G11 按记录分别计数，重叠记录可多次贡献同一槽，不能推导“总分不超过段长”。G09 依赖真实分片有序、不重不漏；G18 验证下一片的边界，Vec 构建、GPU 派发与返回段数仍属于适配层。G14 防止回绕，不会把错误 GPU 计数变成正确结果。

G12–G13 没有直接解析或验证 WGSL 源码；G13 显式要求尾端点装得进 u64。G19 对任意 u64 输入定义并拒绝溢出，继续允许原实现允许的零跨度；合法单元跨度、钉住和隔离是调用方的其他检查。

G15–G16 的外层接口保留返回 u64 的形状，核返回 `None` 时明确 panic，正常输入行为不变。G17 不证明多任务预留的原子性，无上限时外层仍保持 `None`。G20 无法检测“两份核都给出相同错误结果”。

本次不覆盖 GPU 驱动、编译器、执行内存模型、设备屏障、校验和算法及整个 GC 发布路径到模型的精化，不构成整个 GPU GC 的完整正确性证明。

## 见证、反例与复跑

两条见证不占 20 项名额：`witness_gpu_gc_protocol_and_score_premises` 构造非空计数表、跨段活记录、切口与进位边界；`witness_gpu_gc_arithmetic_boundaries` 调用产品核，覆盖空输入、全宽极值、预算/累计溢出、越界记录与未写判定字。

`verus-must-red-worlds.tsv` 的 `gpu-gc-G01` 到 `gpu-gc-G20` 分别改坏上述决策、前提、边界或计算；另两条 `gpu-gc-witness_*` 向见证注入不可能断言，两条 `gpu-gc-audit-*` 改坏模型中的贡献值和 CPU 调用累计，共 24 条。要求 Verus 报证明错误且落点匹配，编不过或只耗尽求解资源都不算。八处 `requires` 另做入口空真筛查；筛查不是一般可满足性证明，具体可满足输入由见证补充。

仓根执行，`VERUS` 指向 `.claude/gate.d/verus-pins.tsv` 固定版本的可执行文件。任务服务按返回的确认单确认后运行。

```sh
task-server/task submit --manual -- "$VERUS" --crate-type=lib --edition=2021 \
  --crate-name singlefs_replay_prefix --no-cheating --rlimit 10 \
  --verify-only-module gpu_gc_proofs crates/singlefs-replay-prefix/src/lib.rs

task-server/task submit --manual -- python3 scripts/gate/verus-proof-check.py must-red-worlds . \
  --item must-red:gpu-gc-G01 --item must-red:gpu-gc-G02 \
  --item must-red:gpu-gc-G03 --item must-red:gpu-gc-G04

task-server/task submit --manual -- cargo test -p singlefs-replay-prefix --offline \
  --test gpu_gc_arithmetic_matches_wide_integer_reference

task-server/task submit --manual -- cargo test -p singlefs-background-gpu --offline \
  --lib two_kernel_reconciliation::tests -- memory_limit_and_chunks::tests \
  live_slot_kernels::tests region_record_kernels::tests
```

反例按同样写法继续点名 G05–G20、两条见证和两条审计反例，每批至多四项。模块共 26 条义务：20 项核心性质、3 条常量、1 条递归终止、2 个见证。新增四个独立核对测试用 u128 覆盖全宽边界、4096 条十二轮历史核对仲裁，并检查判定字和两份累计值的独立性。后台包命令点名四个模块的 56 个测试，不开启真 GPU 特性。

严格自测还直接检查真实主机适配层：

```sh
task-server/task submit --manual -- cargo test -p singlefs-background-gpu --offline \
  --test gpu_gc_formal_contracts_through_the_host
task-server/task submit --manual -- cargo test -p singlefs-background-gpu-tests --offline \
  --test background_gpu_compute_agrees_with_the_cpu_backed_test_double
```

新增两个主机测试穷举 117,649 条六步历史（两条目的相符/分歧、失败批、空批、重挂），逐步核对状态、决策、回调次数和批统计；另对 32 个不同初态、输入与排列的批次检查按键结果及回调位置。有限枚举补充适配层证据，不替代任意长度的形式化证明。严格自测发现、修正及实测结果见 [GPU_GC_SELFTEST.md](GPU_GC_SELFTEST.md)。
