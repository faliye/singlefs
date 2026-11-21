# GPU GC 形式化验证的覆盖与边界

入口是 [gpu_gc_proofs.rs](gpu_gc_proofs.rs)。这些性质补充 GPU GC 的仲裁、计数、算术与准入；已有根环、旧读者、回退和崩溃协议证明不重复计入。G01–G20 各有独立契约和必红变异，常量、递归终止与见证不占这 20 项的名额。

容量与资源记账的 G21–G30 位于 [gpu_gc_capacity_proofs.rs](gpu_gc_capacity_proofs.rs)：整批准入、最大输入、收紧上限、预留、释放、等待完成后释放、失败折半、派发覆盖、映射表占用及追加输入准入。G21–G29 是产品直接调用的可执行核；G30 `extended_gpu_chunk_input` 产品此刻没有调用点（`crates/singlefs-background-gpu/src/` 里没有一处调它），产品追加输入那一步在 `memory_limit_and_chunks::chunks_that_fit_on_the_gpu` 里自己做加法与过滤，那一段在核外、没验。这些核不证明外层资源账本所有调用都符合协议。

## 两份打分算法的归纳证明

[gpu_gc_scoring_proofs.rs](gpu_gc_scoring_proofs.rs) 的 `theorem_sorted_search_equals_full_record_walk` 将二分查找、候选窗口、逐槽计数和排列组合起来：对任意长度不超过 `u32::MAX` 的合法记录序列，排序后的搜索算法与原始无序表逐记录求交的精确总和相等。重复记录保留重数。证明不枚举固定条数、固定槽号或固定循环次数。

证明链包括：

- `lower_bound_of_sorted_records` 对二分循环状态 `(below, above)` 归纳，验证前缀/后缀不变量、下标界限与严格递减量，得到完整下界性质；下界不是输入假设。函数体断言 `middle + 1` 不超过 `u32::MAX`，这一步靠 `requires` 里记录数不超过 `u32::MAX` 那条界（删掉那条界的反例是必红表 `gpu-score-binary-midpoint-needs-u32-bound`）。
- `lemma_candidate_window_keeps_every_contribution` 从两端的下界性质及最大跨度推导：窗内全是目标设备，窗外贡献为零。下界使用饱和减法，覆盖从段前伸入的记录。窗口下沿往前退 65535 个槽；证明钉住的是「至少退 65534 个」：跨度不超过 65535 的记录，起点早于 `start − 65534` 的尾端不超过 `start`、贡献为零，所以退 65534 个照样验得过，退 65533 个才漏（必红表 `gpu-score-window-too-narrow`）。65535 比证明要的宽一格，这一格不由证明钉住。
- `lemma_counting_slots_equals_interval_intersection` 对跨度归纳，从逐槽成员判定推导交集长度。
- `lemma_search_prefix_matches_the_walk` 对已访问记录数归纳，证明裁剪窗口与整表扫描的前缀对应。
- `lemma_permuting_records_preserves_the_walk` 以多重集合相等为排列条件，利用已验证的可交换 fold 引理证明排序保持总和。
- `lemma_every_walk_prefix_fits_the_gpu_counter` 证明每个扫描前缀的值介于 0 与 `已访问条数 × 65535`，因此装得进 u64。没有假设总分不超过段长。

前提是排序表按 `(device, start)` 非降、排序保持整个记录的多重集合、记录起止有效且跨度不超过 65535、记录数装得进 u32、段为有效的 u64 非空半开区间。非空见证含重复、异盘、释放及跨段记录；另有空表见证。

这是**算法模型的机器检查证明**，不是 WGSL 源码的直接验证。二分模型以递归表示每一次循环状态转移，使用数学整数，下一轮下界 `middle + 1` 装得进 u32 由函数体里的断言证明；逐槽模型使用完整整数槽号。WGSL 的实际双字操作、排序索引解引用、缓冲区布局与派发到该模型的精化仍未证明。主机的 `sort_by_key` 确实建立上述排列与有序前提，也仍须另证。不得把此定理称为整个 GPU GC 已验证。

完整反例清单为 `verus-must-red-worlds.tsv` 中 `gpu-score-` 开头的各行。实现反例覆盖二分跳项/错误上界、逐槽偏移、窗口过窄、释放误计、跳过记录；前提反例覆盖有序、合法跨度、重数、排列、计数上界与中点的 u32 界；另外检查归纳步、计数上下界与见证。编译失败、类型错误或只有求解超时都不能计为抓住反例。

## 覆盖清单

“产品核”表示产品直接调用 Verus 验证的同一份可执行函数。“模型”表示抽象状态或算术定理，到 Rust 容器、WGSL 或设备实现的精化仍未证明。“定义自检”表示 `ensures` 是同一文件里 spec 函数定义的展开，只钉住那份定义，对产品代码没有约束。

| 编号 | 性质与被验入口 | 对应 GPU GC 路径 | 层次 |
| --- | --- | --- | --- |
| G01 | 一致才确认，首轮分歧暂缓，第二轮交 CPU，三种决策互斥且计数不溢出。`reconcile_gpu_entry` | `two_kernel_reconciliation::reconcile_the_two_kernel_outcomes` | 产品核 |
| G02 | 连续两次分歧恰好触发一次 CPU 判定，之后重新计数。`lemma_two_disagreements_trigger_exactly_one_cpu_judgment` | 双核仲裁 | 模型，定义自检 |
| G03 | 一次一致打断连续分歧，不能累计隔轮分歧。`lemma_agreement_restarts_the_disagreement_history` | 双核仲裁 | 模型，定义自检 |
| G04 | GPU 批失败不改状态；成功轮次精确更新计数与 CPU 调用次数。`lemma_failed_batch_keeps_all_counts_and_cpu_calls` | `reconcile_unless_the_batch_failed` | 模型，定义自检 |
| G05 | 不同条目的更新可交换，重排后计数表和 CPU 调用总数相同。`lemma_distinct_entry_updates_commute` | 按条目键保存的计数表 | 模型 |
| G06 | 稀疏表只保存一个未决轮次，未出现的条目不变。`lemma_count_updates_preserve_the_sparse_counter_invariant` | 同上 | 模型 |
| G07 | 区间贡献非负、对称、不超过任一跨度，相邻区间贡献零。`lemma_live_slot_intersection_bounds` | 逐记录求交的打分核 | 模型 |
| G08 | 段切开后的贡献可加，跨界记录不漏不重算。`lemma_splitting_a_segment_preserves_live_slot_contributions` | 两份打分核的段边界 | 模型 |
| G09 | 任意有限记录表在任意切口分片，分片求和等于整表。`lemma_record_slices_preserve_the_score` | `live_slots_summed_over_the_record_slices` | 模型，归纳而非定深枚举 |
| G10 | 重复记录保留计数重数，不能误改为槽集合去重。`lemma_duplicate_records_keep_their_multiplicity` | 两份打分核和 CPU 参照 | 模型，定义自检（`ensures` 是 `record_slice_sum` 定义展开两层，证明体只有 `reveal_with_fuel(record_slice_sum, 3)`），只证重数 2 |
| G11 | 已释放或异盘记录贡献零；同盘活记录贡献恰为交集长度，严格相交才为正。`lemma_released_and_foreign_records_do_not_score` | 打分核的记录过滤 | 模型，定义自检（三条 ensures 是 `record_contribution` 定义的展开） |
| G12 | 双 u32 的高字优先比较等价于全宽槽号比较。`lemma_two_word_slot_order_is_u64_order` | WGSL 槽号比较 | 模型 |
| G13 | 低字加跨度至多进位一次，全宽和合法时高字不溢出。`lemma_two_word_span_addition_carries_exactly_once` | WGSL `slot_past_the_span` | 模型 |
| G14 | 两核分片计数各自累加，任一溢出都拒绝。`checked_live_slot_totals` | `live_slots_summed_over_the_record_slices` | 产品核 |
| G15 | 最小非零四字节对齐输入长度，向上取整溢出拒绝。`checked_gpu_input_bytes` | `memory_limit_and_chunks::input_buffer_bytes` | 产品核 |
| G16 | 预算包含对齐输入、逐条占用和固定占用，乘加溢出拒绝。`checked_gpu_chunk_bytes` | `ChunkFootprint::bytes_held_by_a_chunk` | 产品核 |
| G17 | 扣除其他阶段已占显存后不回绕；未超额时已占加剩余恰为总限；不超过剩余量的正的新增分配加上已占不超过总限。`remaining_gpu_memory` | `ChunkBoundsOfAGpu::with_the_room_left_after` | 产品核 |
| G18 | 下一记录片有进展、不过界，与余下区间不重不漏，避免先加后取 min 的溢出。`next_gpu_record_slice_end` | `live_slot_plan_on_a_gpu` | 产品核 |
| G19 | 同盘、未释放、起点及全跨度在区内才扫描，跨度溢出拒绝。`gpu_record_is_in_scan_region` | `region_record_kernels::record_is_one_the_region_scan_reads` | 产品核 |
| G20 | 只接受 counted=1，拒绝填充字和未知字，保留数值高低两字。`decoded_gpu_live_slots` | `live_slot_kernels::decoded_live_slot_outcomes` | 产品核 |
| G21 | 显存、输入绑定、条目绑定和派发四道界同时满足才放行；乘加溢出不误放行。`gpu_chunk_fits` | `memory_limit_and_chunks::chunk_fits_with_the_staging_of_its_input` | 产品核；显存上限是交进来的，扣暂存那一层在核外，见下 |
| G22 | 最大单条分片是装得下的最大四字节倍数；`None` 当且仅当最小输入也放不下。`largest_gpu_piece_input` | `memory_limit_and_chunks::largest_piece_input_bytes` | 产品核；显存上限是交进来的，扣暂存那一层在核外，见下 |
| G23 | 收紧显存上限：原来无限额时采用新上限，已有限额时不放宽。`tightened_gpu_memory_limit` | `ChunkBoundsOfAGpu::with_the_memory_limit_at_most` | 产品核 |
| G24 | 尚有未完成提交、账本溢出或超限时拒绝新增占用。`checked_gpu_memory_hold` | `GpuMemoryAccount::hold` | 产品核 |
| G25 | 只释放已记账的字节，超额释放拒绝。`checked_gpu_memory_release` | `GpuMemoryAccount::release` | 产品核 |
| G26 | 等待未成功时保留全部占用，完成才释放本段；段大小超过占用时拒绝。`gpu_bytes_after_submission_wait` | `GpuMemoryAccount::settle_the_chunk_after_its_submission` | 产品核 |
| G27 | 显存不足后把失败段占用折半：正占用严格下降，零仍为零，取整误差至多一。`gpu_memory_cap_after_allocation_failure` | `background_gpu::outcomes_of_a_batch_over_the_cards` | 产品核 |
| G28 | 派发恰好所需的工作组数，零条目不派发，全宽条数不溢出。`gpu_workgroups_for_entries` | `background_gpu` 里三份核的派发 | 产品核 |
| G29 | 映射条目、落点、排序索引三张表含空表占位，交回总量与最大绑定量，任何溢出拒绝。`checked_gpu_mapping_table_bytes` | `region_record_kernels::mapping_table_bytes_of` | 产品核 |
| G30 | 追加输入只有和可表示且整批满足四道界才接受，拒绝不把和回绕成小批。`extended_gpu_chunk_input` | 无：产品此刻没有调用点 | 核，未接产品 |

## 衔接与未覆盖边界

后台 GPU crate 直接依赖 `singlefs-replay-prefix`，调用 G01–G20 表里的八个产品核与 G21–G29 九个产品核。两轮阈值、四字节对齐和 counted 判定字由核定义，主机引用同一份常量。两份 WGSL 仍独立实现，CPU 打分参照也保持独立。门禁 formal-verification 的 verus-proof 格另比产品构建与 `verus --compile` 构建的 MIR 逐条目相同：产品编进的就是被验的那一份可执行核；这不代表外围容器、WGSL、驱动或硬件已形式化验证。

G02–G04 的 `ensures` 是 `next_uncertain_count`、`cpu_is_needed`、`round_state` 这几个 spec 函数在给定输入上的展开：它们只钉住 spec 定义（改坏定义会红，例 `gpu-gc-audit-successful-cpu-call-lost`），与产品的 `two_kernel_reconciliation::reconcile_unless_the_batch_failed` 之间没有机器连接。仲裁适配层将真实结果的相等比较传给 G01，只有 `ask_cpu` 分支调用 CPU，只有暂缓分支保存一个未决轮次。`Map<int, nat>` 抽象条目身份与稀疏表；模型不证明 `HashMap`、结果类型的 `PartialEq`、批内键唯一性或外层循环的精化。G04 对应批失败时的提前返回，不能据此宣称整个批处理循环已验证。G11 的三条 `ensures` 同样是 `record_contribution`（连同它用的 `overlap_slots`）定义的展开：只钉住这份定义（改坏定义会红，例 `gpu-gc-audit-score-capped-at-two`），对打分核的 WGSL 与主机一侧的记录过滤没有约束。

G07–G11 按记录分别计数，重叠记录可多次贡献同一槽，不能推导“总分不超过段长”。G09 依赖真实分片有序、不重不漏；G18 验证下一片的边界，Vec 构建、GPU 派发与返回段数仍属于适配层。G14 防止回绕，不会把错误 GPU 计数变成正确结果。

G12–G13 没有直接解析或验证 WGSL 源码；G13 显式要求尾端点装得进 u64。G19 对任意 u64 输入定义并拒绝溢出，允许零跨度；合法单元跨度、钉住和隔离是调用方的其他检查。

G15–G16 的外层接口返回 u64，核返回 `None` 时外层明确 panic。G17 不证明多任务预留的原子性，无上限时外层保持 `None`。G20 无法检测“两份核都给出相同错误结果”。

G21、G22 的显存界是交进来的上限，不是卡上的真占用：映射着建的输入在卡上另带暂存（`memory_limit_and_chunks::COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION` 份），产品的 `chunk_fits_with_the_staging_of_its_input` 交给 G21 之前先从上限里扣掉输入的暂存，`largest_piece_input_bytes` 交给 G22 之前把扣掉固定占用之后剩下的地方按份数分开。这一层折算在核外、没进 Verus，「卡上真占用不超过上限」靠的是这一层没验的代码。G30 没有产品调用点（见开头）。

不覆盖 GPU 驱动、编译器、执行内存模型、设备屏障、校验和算法及整个 GC 发布路径到模型的精化，不构成整个 GPU GC 的完整正确性证明。

## 见证与反例

`witness_gpu_gc_protocol_and_score_premises` 构造非空计数表、跨段活记录、切口与进位边界；`witness_gpu_gc_arithmetic_boundaries` 调用产品核，覆盖空输入、全宽极值、预算/累计溢出、越界记录与未写判定字；`witness_gpu_capacity_and_accounting_boundaries` 覆盖恰好装满与超出一字节、超时保留、完成释放、空表占位及全宽派发。见证不占 G01–G30 的编号。`gpu_gc_proofs` 模块共 26 条义务：20 项核心性质、3 条常量、1 条递归终止（`record_slice_sum`）、2 个见证。

`verus-must-red-worlds.tsv` 里 `gpu-gc-G01` 到 `gpu-gc-G30` 各一行改坏对应的决策、前提、边界或计算；名字形如 `gpu-gc-G<编号>-<说明>` 的行逐条罩住同一编号里那一行没碰到的 `ensures` 条、拒绝分支与三条常量（多数是改函数体的一处，推不出别的反例的派生条改 `ensures` 本身）；名字带 witness 的 `gpu-gc-` 行有五条，分两类：`gpu-gc-witness_gpu_gc_protocol_and_score_premises`、`gpu-gc-witness_gpu_gc_arithmetic_boundaries`、`gpu-gc-capacity-witness` 三条向三个见证注入不可能断言（头一条在 `ensures` 里加 `false`，后两条在函数体头插 `assert(false)`），`gpu-gc-witness-record-slice-sum-value`、`gpu-gc-witness-first-disagreement-count-value` 两条把 `witness_gpu_gc_protocol_and_score_premises` 里钉的值改错（单条记录对段的 `record_slice_sum` 由 2 改成 3、头一次分歧之后的计数由 1 改成 2）；两条 `gpu-gc-audit-*` 改坏模型：`gpu-gc-audit-score-capped-at-two` 把每条记录的贡献截顶为 2（同盘未释放记录 `[0,3)` 对段 `[0,3)` 应贡献 3，G11「同盘活记录贡献恰为交集长度」那一条抓它），`gpu-gc-audit-successful-cpu-call-lost` 让成功轮不累计 CPU 调用（G04 成功那一支抓它）。每行要求 Verus 报证明错误且落点等于第六段，编不过或只耗尽求解资源都不算。每处 `requires` 的入口空真由门禁 formal-verification 的 verus-vacuity 格逐处判（项 `vacuity:<所属函数>#<第几处>`）；那不是一般可满足性证明，具体可满足输入由见证补充。

## 独立核对测试

- `crates/singlefs-replay-prefix/tests/gpu_gc_arithmetic_matches_wide_integer_reference.rs`（4 个测试）：用 u128 参照核对 G14–G20 的预算、溢出、片界、扫描区准入、双核累计与回读，4096 条十二轮历史核对 G01 的仲裁。边界值表含 `u64::MAX − 3` 到 `u64::MAX` 四个相邻值，四字节对齐向上取整溢出的分界落在 `u64::MAX − 3` 与 `u64::MAX − 2` 之间。
- `crates/singlefs-replay-prefix/tests/gpu_gc_capacity_matches_independent_reference.rs`（5 个测试）：G21–G30。G22 拿闭式结果对 43,050 组容量的穷举；G27 不做除法，按折半的定义（两倍不超过失败占用、加一的两倍超过）判；G29 每项字节数按产品交给核的缓冲布局（`region_record_kernels.rs` 文件头「缓冲布局」）逐个字段数出来，不抄核里的乘数。
- 两份测试的每个 `#[test]` 在 `crates/mutations.tsv` 挂一行「改坏被测核的哪一处、这条测试红」。
- 后台包 `two_kernel_reconciliation`、`memory_limit_and_chunks`、`live_slot_kernels`、`region_record_kernels` 四个模块源码里共 82 个 `#[test]`（7、36、23、16）。
- `crates/singlefs-background-gpu/tests/gpu_gc_formal_contracts_through_the_host.rs`（2 个测试）走真实主机适配层：穷举 `7^6 = 117649` 条六步事件历史（两条目的相符/分歧、失败批、空批、重新创建挂载状态），逐步核对按键计数、决策、CPU 回调次数及批统计；另对 4 种初始状态 × 4 种相符组合 × 2 种顺序共 32 个批次核对回调位置和按键结果。参照每条目一个布尔未决状态，不调被验的仲裁函数算期望值。有限枚举不替代任意长度历史的精化证明。
- `crates/singlefs-background-gpu/tests/gpu_gc_capacity_contracts_through_the_host.rs`（5 个测试）在主机一侧核容量与记账，其中 G29 的尺寸对真打包出来的映射表。
- 真 GPU 两组测试（`singlefs-background-gpu-tests` 的 `live_slot_kernels_on_a_real_gpu_agree_with_the_cpu_reference`、`region_record_kernels_on_a_real_gpu_agree_with_the_cpu_reference`，各 6 个测试，挂 `tests-on-a-real-gpu` 特性）做双核与 CPU 对拍、跨 32 位槽界、小显存分段/分片与预算过低时排除设备。

## 复跑

下面的命令由主 agent 在仓根经统一任务服务交（`.claude/agent-common.md`「子 agent 不交任务」）；统一任务服务只调配 CPU，内存、线程与机器由它定，任务不拿卡（`.claude/rules/verification.md`「卡只归 GPU 服务，任务不拿卡」）。证明逐项的项名是 `proof:<全路径>`，照门禁 formal-verification 的 `--list-items` 列出的写；`cargo test` 一律点名测试全名、带 `--exact`。

```sh
SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 formal-verification.sh 带 --item; 范围=点名的 4 项; 为什么=复判打分算法的二分跳项、窗口过窄、释放误计与重数四行反例' task-server/task submit -- bash .claude/gate.d/formal-verification.sh --item must-red:gpu-score-binary-skips-entry --item must-red:gpu-score-window-too-narrow --item must-red:gpu-score-search-counts-released --item must-red:gpu-score-permutation-needs-multiplicity .

SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 formal-verification.sh 带 --item; 范围=点名的 4 项; 为什么=复判 G01 到 G04 四行反例，看仲裁与计数模型的契约还抓得住' task-server/task submit -- bash .claude/gate.d/formal-verification.sh --item must-red:gpu-gc-G01 --item must-red:gpu-gc-G02 --item must-red:gpu-gc-G03 --item must-red:gpu-gc-G04 .

SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-replay-prefix 的测试目标 gpu_gc_arithmetic_matches_wide_integer_reference 里的 4 条测试; 范围=4 个测试; 为什么=用 u128 参照复核 G01 与 G14 到 G20 的产品核' task-server/task submit -- cargo test -p singlefs-replay-prefix --test gpu_gc_arithmetic_matches_wide_integer_reference -- gpu_budget_arithmetic_matches_a_u128_reference_at_full_width gpu_slices_scans_and_independent_totals_match_wide_arithmetic gpu_readback_rejects_unwritten_words_and_preserves_both_halves gpu_reconciliation_matches_the_two_round_history_rule --exact

SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-replay-prefix 的测试目标 gpu_gc_capacity_matches_independent_reference 里的 5 条测试; 范围=5 个测试; 为什么=用独立参照复核 G21 到 G30 的容量与记账核' task-server/task submit -- cargo test -p singlefs-replay-prefix --test gpu_gc_capacity_matches_independent_reference -- chunk_admission_and_extension_match_full_width_arithmetic largest_piece_matches_exhaustive_search_and_is_maximal holds_releases_and_waits_preserve_the_exact_ledger memory_caps_never_expand_and_allocation_failure_makes_progress mapping_buffers_and_dispatch_cover_their_exact_full_width_sizes --exact

SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-background-gpu 的 --lib 里四个主机模块的 82 条测试; 范围=82 个测试; 为什么=复核接这些核的四个主机模块' task-server/task submit -- cargo test -p singlefs-background-gpu --lib -- \
  two_kernel_reconciliation::tests::identical_outcomes_are_confirmed \
  two_kernel_reconciliation::tests::different_outcomes_are_uncertain_for_one_round_without_asking_the_cpu \
  two_kernel_reconciliation::tests::two_consecutive_uncertain_rounds_hand_the_entry_to_the_cpu_reference \
  two_kernel_reconciliation::tests::agreeing_again_after_a_disagreement_restarts_the_consecutive_count \
  two_kernel_reconciliation::tests::an_entry_absent_from_a_batch_keeps_its_count \
  two_kernel_reconciliation::tests::dropping_the_counts_forgets_every_uncertain_round \
  two_kernel_reconciliation::tests::the_same_entry_key_twice_in_one_batch_is_an_error_leaving_the_counts_and_never_asking_the_cpu \
  memory_limit_and_chunks::tests::the_chunk_whose_wait_timed_out_stays_held_until_its_submission_completes_on_the_gpu \
  memory_limit_and_chunks::tests::the_gpu_whose_timed_out_submission_is_still_running_gets_no_chunk_of_the_next_batch \
  memory_limit_and_chunks::tests::holding_a_chunk_while_a_timed_out_submission_is_still_running_panics \
  memory_limit_and_chunks::tests::the_gpu_given_no_entries_of_the_next_batch_does_not_wait_for_its_timed_out_submission \
  memory_limit_and_chunks::tests::copies_on_the_gpu_times_the_bytes_of_one_copy_give_the_bytes_of_the_copies \
  memory_limit_and_chunks::tests::copies_on_the_gpu_checked_bytes_of_the_copies_refuse_past_u64 \
  memory_limit_and_chunks::tests::copies_on_the_gpu_bytes_of_the_copies_past_u64_panics \
  memory_limit_and_chunks::tests::entry_count_of_a_chunk_times_the_bytes_of_one_entry_gives_the_bytes_of_the_entries \
  memory_limit_and_chunks::tests::entry_count_of_a_chunk_bytes_of_the_entries_past_u64_panics \
  memory_limit_and_chunks::tests::entry_count_of_a_chunk_times_the_input_bytes_of_one_entry_gives_the_input_bytes_of_the_entries \
  memory_limit_and_chunks::tests::entry_count_of_a_chunk_input_bytes_of_the_entries_past_u64_panics \
  memory_limit_and_chunks::tests::input_length_on_the_host_of_this_many_bytes_is_the_slice_length \
  memory_limit_and_chunks::tests::input_length_on_the_host_checked_add_gives_the_sum_and_refuses_past_u64 \
  memory_limit_and_chunks::tests::input_length_on_the_host_plus_gives_the_sum \
  memory_limit_and_chunks::tests::input_length_on_the_host_plus_past_u64_panics \
  memory_limit_and_chunks::tests::input_length_on_the_host_minus_gives_what_is_left \
  memory_limit_and_chunks::tests::input_length_on_the_host_minus_more_than_there_is_panics \
  memory_limit_and_chunks::tests::input_lengths_on_the_host_summed_over_the_pieces_are_their_total \
  memory_limit_and_chunks::tests::input_buffers_round_up_to_whole_words_and_never_have_zero_bytes \
  memory_limit_and_chunks::tests::without_a_memory_limit_a_batch_is_one_chunk \
  memory_limit_and_chunks::tests::the_batch_that_does_not_fit_the_memory_limit_is_cut_into_chunks_that_each_fit \
  memory_limit_and_chunks::tests::the_chunk_exactly_at_the_memory_limit_is_kept_whole \
  memory_limit_and_chunks::tests::the_dispatch_and_binding_limits_also_cut_chunks \
  memory_limit_and_chunks::tests::two_segments_held_on_one_gpu_together_stay_within_its_memory_limit \
  memory_limit_and_chunks::tests::the_room_left_after_other_segments_without_a_memory_limit_is_still_unlimited \
  memory_limit_and_chunks::tests::an_entry_that_alone_exceeds_the_memory_limit_refuses_the_batch \
  memory_limit_and_chunks::tests::an_empty_batch_has_no_chunk \
  memory_limit_and_chunks::tests::holding_up_to_the_memory_limit_is_allowed_and_releasing_gives_it_back \
  memory_limit_and_chunks::tests::holding_past_the_memory_limit_panics \
  memory_limit_and_chunks::tests::without_a_memory_limit_holding_is_never_refused \
  memory_limit_and_chunks::tests::the_batch_whose_entries_each_fit_is_cut_like_the_chunks_with_one_whole_piece_per_entry \
  memory_limit_and_chunks::tests::an_entry_larger_than_the_memory_limit_is_cut_into_pieces_that_each_fit \
  memory_limit_and_chunks::tests::the_memory_limit_below_the_smallest_piece_reports_how_much_the_smallest_piece_needs \
  memory_limit_and_chunks::tests::checksum_chunk_fitting_without_the_staging_but_not_with_it_is_cut_into_pieces_that_fit_with_the_staging \
  memory_limit_and_chunks::tests::the_chunk_whose_buffers_were_not_created_is_released_once_the_flush_of_the_pending_writes_completes \
  memory_limit_and_chunks::tests::the_chunk_whose_buffers_were_not_created_stays_held_while_the_flush_of_the_pending_writes_is_running \
  live_slot_kernels::tests::allocation_records_are_listed_in_order_with_their_released_word_and_sorted_by_device_and_slot \
  live_slot_kernels::tests::records_at_the_same_device_and_slot_are_sorted_by_their_position \
  live_slot_kernels::tests::an_empty_record_table_still_has_one_item_in_both_buffers \
  live_slot_kernels::tests::segment_bounds_are_the_first_slot_and_the_slot_past_each_cluster_segment \
  live_slot_kernels::tests::parameters_carry_the_segment_count_the_record_count_and_the_device_of_the_unit_area \
  live_slot_kernels::tests::readback_records_decode_into_sixty_four_bit_live_slot_counts \
  live_slot_kernels::tests::live_slot_outcome_the_kernels_never_wrote_is_an_unrecognized_verdict_word \
  live_slot_kernels::tests::the_segment_counted_differently_by_the_two_kernels_is_uncertain_once_and_judged_by_the_cpu_reference_the_next_round \
  live_slot_kernels::tests::segments_other_than_every_full_segment_of_the_unit_area_panic \
  live_slot_kernels::tests::both_live_slot_kernels_pass_the_shader_validator \
  live_slot_kernels::tests::the_walking_and_searching_live_slot_kernels_are_separate_sources_with_their_own_entry_points \
  live_slot_kernels::tests::record_table_bytes_counted_from_the_record_count_match_the_tables_built \
  live_slot_kernels::tests::allocation_record_count_of_this_many_records_is_the_slice_length \
  live_slot_kernels::tests::cluster_segment_count_of_this_many_segments_is_the_slice_length \
  live_slot_kernels::tests::the_record_table_cut_into_slices_under_a_small_memory_limit_sums_to_the_cpu_reference \
  live_slot_kernels::tests::the_memory_limit_below_the_smallest_live_slot_piece_reports_how_much_it_needs \
  live_slot_kernels::tests::the_middle_segment_twice_in_place_of_the_last_is_returned_as_a_repeated_entry_key \
  live_slot_kernels::tests::the_middle_segment_twice_with_the_kernels_failed_is_returned_as_a_repeated_entry_key \
  live_slot_kernels::tests::the_last_segment_appended_again_is_returned_as_a_repeated_entry_key \
  live_slot_kernels::tests::the_last_segment_appended_again_with_the_kernels_failed_is_returned_as_a_repeated_entry_key \
  live_slot_kernels::tests::the_first_segment_appended_again_is_returned_as_a_repeated_entry_key \
  live_slot_kernels::tests::the_first_segment_appended_again_with_the_kernels_failed_is_returned_as_a_repeated_entry_key \
  live_slot_kernels::tests::the_full_list_of_segments_without_a_repeat_is_reconciled \
  region_record_kernels::tests::pieces_are_packed_end_to_end_with_their_location_span_read_flag_kind_and_registers \
  region_record_kernels::tests::mapping_tables_list_each_entry_with_its_key_length_and_both_locations \
  region_record_kernels::tests::the_location_index_is_sorted_by_device_and_slot_and_lists_an_entry_once_per_distinct_location \
  region_record_kernels::tests::the_key_index_is_sorted_by_key_length_and_words_with_ties_in_table_order \
  region_record_kernels::tests::one_mapping_key_longer_than_the_kernels_hold_refuses_the_tables \
  region_record_kernels::tests::empty_mapping_tables_still_have_one_item_in_every_buffer \
  region_record_kernels::tests::every_outcome_kind_decodes_into_its_region_scan_class \
  region_record_kernels::tests::region_record_outcome_the_kernels_never_wrote_is_an_unrecognized_verdict_word \
  region_record_kernels::tests::the_cpu_reference_classifies_each_record_as_the_whole_region_scan_does \
  region_record_kernels::tests::the_cpu_reference_refuses_a_record_outside_the_region \
  region_record_kernels::tests::both_region_record_kernels_pass_the_shader_validator \
  region_record_kernels::tests::the_walking_and_searching_region_record_kernels_are_separate_sources_with_their_own_entry_points \
  region_record_kernels::tests::the_mapping_cut_into_slices_under_a_small_memory_limit_classifies_every_record_as_the_whole_mapping \
  region_record_kernels::tests::units_cut_into_pieces_under_a_memory_limit_smaller_than_a_unit_classify_every_record_as_the_whole_unit \
  region_record_kernels::tests::registers_after_the_pieces_decode_only_from_the_kind_the_kernels_write_for_them \
  region_record_kernels::tests::mapping_table_bytes_counted_from_the_entries_match_the_tables_built \
  --exact

SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-background-gpu 的测试目标 gpu_gc_formal_contracts_through_the_host 里的 2 条测试; 范围=2 个测试; 为什么=复核真实主机仲裁适配层的有限历史枚举' task-server/task submit -- cargo test -p singlefs-background-gpu --test gpu_gc_formal_contracts_through_the_host -- interleaved_entries_failures_empty_batches_and_remounts_match_the_history_model permuting_distinct_entries_preserves_keyed_results_and_cpu_calls --exact
```

别的反例照第一条的写法点名（`must-red:<变异名>`，变异名是必红表第一段）。

真 GPU 两组对拍用卡，不写成上面那种点名测试函数的 `cargo test`：任务不拿卡（`.claude/rules/verification.md`「卡只归 GPU 服务，任务不拿卡」），这两组由门禁 crash-point-replay 的 product-gpu-real-card-tests 格（`.claude/gate.d/crash-point-replay.sh:12`）起，卡由 GPU 服务交（那份文件头「显卡」一段）。那一格对 `singlefs-background-gpu-tests` 里挂 `tests-on-a-real-gpu` 特性的每个测试目标先只录、再只判，每条测试函数按单元键复用；点这一格算整包跑法，归提交时的门禁（`.claude/agent-common.md`「日常开发的测试不跑整个包」）。这两个测试目标各 6 个测试：

- `live_slot_kernels_on_a_real_gpu_agree_with_the_cpu_reference`：`the_kernel_walking_every_record_matches_the_cpu_reference_on_both_unit_areas`、`the_kernel_searching_the_sorted_records_matches_the_cpu_reference_on_both_unit_areas`、`the_two_live_slot_kernels_confirm_every_segment_on_a_healthy_gpu`、`the_memory_limit_smaller_than_the_batch_cuts_the_segments_into_chunks_with_the_same_live_slots`、`the_memory_limit_smaller_than_the_record_table_cuts_it_into_slices_with_the_same_live_slots`、`the_memory_limit_below_the_smallest_piece_leaves_every_gpu_out_without_refusing_the_opening`。
- `region_record_kernels_on_a_real_gpu_agree_with_the_cpu_reference`：`the_walking_kernel_matches_the_cpu_reference_on_every_class_and_parser_refusal`、`the_searching_kernel_matches_the_cpu_reference_on_every_class_and_parser_refusal`、`the_two_region_record_kernels_confirm_every_record_on_a_healthy_gpu`、`the_memory_limit_smaller_than_the_region_batch_cuts_it_into_chunks_with_the_same_classifications`、`the_memory_limit_smaller_than_the_mapping_cuts_it_into_slices_with_the_same_classifications`、`the_memory_limit_smaller_than_a_unit_cuts_each_unit_into_pieces_with_the_same_classifications`。

这两组的结果怎么读：每条用例经 `crates/singlefs-background-gpu-tests/tests/common_pairing_input_through_the_library/mod.rs` 的 `pairing_concluded_on_the_cards_through_the_pairing_library` 过判定库，日志里打的行分四种：

- 只录那一段（`SINGLEFS_CRASH_AMPLIFICATION_STAGE=record`）：打 `RECORDED_ONLY test=<用例名> input_bytes=<字节数>`，输入与参照写进判定库就返回，不领卡、不开卡。
- 判段里判定库在这几把键下已有两边一致的结论：打 `PAIRING_CONCLUSION_REUSED test=<用例名>`，不领卡、不开卡、照通过；复用的是库里存着的、在卡上判出的结论，这一趟没上卡。
- 要现判的：领卡、开卡，拿到这一台的卡打 `GPUS_HANDED_OVER_BY_THE_TASK_SERVER test=<用例名> gpus=…`；判出两边一致打 `PAIRING_CONCLUSION_JUDGED test=<用例名> agreement=both_sides_agree`、结论入库；判出不一致的结论记红、用例失败。
- 现判时没拿到这一台的卡、或这张卡撑不起这一条：打 `NOT_RUN_NO_GPU_ADAPTER test=<用例名> reason=<一句>`、返回成功，记「本次未跑」，结论不入库。

后两种的记号行由 `crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs` 的 `not_run_line_of_the_test`、`handed_over_gpus_line_of_the_test` 拼。所以 cargo 报的通过数不说明这一趟上过卡：日志里有交了卡的那一行与 `PAIRING_CONCLUSION_JUDGED` 行、没有 `NOT_RUN_NO_GPU_ADAPTER` 行的，才是这一趟在卡上判过。预算低于最小一片的那一条核的是排除设备那一路，不说明它在卡上派发过着色器。
