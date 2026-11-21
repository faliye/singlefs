# 重放前缀代码轮第二轮：攻方腿报告

开工快照核过：`sha256sum -c research/prompts/replay-prefix-backfill-code-r2-snapshot/crates-sha256.txt`（仓根起）十份全部 OK，下面引的行号都是这棵树上的。这条腿没写模型、没建仓副本：攻的全是读码推与现查（grep、git、主 agent 给了号的任务日志），所以没有模型目录、没有 `rerun.sh`。草稿目录 `/home/fy5090/.local/share/singlefs/work/replay-prefix-backfill-code-r2-attack/` 里只有文本：必红表 HEAD 版与 `changes.patch` 底版的导出、两份 diff、任务 20692 与 20696 的日志拷贝与从中抽的项名。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 1 u32 界断言 | 没打中 | 删界只断这一处断言，WGSL 的中点与加一是同式 |
| 1 窗口常数推导 | 没打中 | 退 65534 足够、退 65533 漏，说明文件写对了 |
| 1 覆盖写按段 ensures | 没打中（注释说宽一点，推的） | 三条合起来要 `overwrite_fits`；前缀那一条单独只要起点在文件内 |
| 1 G17 量词 | 没打中 | 非空真，触发项合法；这一条由前三条推得出，没有消费者 |
| 2 75 行抽查 | 没打中 | 抽 30 行，改坏的都是名字说的那条、落点对；35 行改的是 ensures 本身（推的分类） |
| 2 还没反例罩着的 ensures | 列 5 处（推的） | 候选窗口两条、定理三条、`bytes_in_unit` 的 None 臂、覆盖写第一条引理两条 |
| 3 新参照与被测共用 | 部分 | G27 参照与 Verus 契约同式（不与函数体同式）；G29 与产品共用字面值 7；`(76, 56)` 与核的见证同值 |
| 3 九行变异 | 没打中（推的） | 都红在点名测试的第一处比对上；任务 20696 里这九行是复用，日志看不到断言位置 |
| 4 说明文件的数 | 打中 | README 文件区间模块的义务数是旧的；`GPU_GC_PROOFS.md` 五条 cargo 复跑命令交不上服务，真卡那条在现行规则下只会「本次未跑」照样退 0，读法没并进来 |
| 5 交集长度 | 打中两处 | 「改 7 行」核不出（量到改 3 行），快照补丁漏了这 3 行；`bytes_in_unit` None 臂没有反例 |
| 6 三处小问题 | 三处都成立 | 第一处范围更宽：第 394–401 行也挂在打分那段注释下；`verus-pins.tsv` 第 48 行还写 211 |
| 7 检查单八格 | 一格不成立 | 「带 --item 的命令与项」不成立，其余七格成立 |

## 攻击面 1：第一轮第三节第 3 条的修补

### u32 界断言（没打中）

- 界在「0 <= below <= above <= records.len() <= u32::MAX,」（`crates/singlefs-replay-prefix/src/gpu_gc_scoring_proofs.rs:35`），断言是「assert(middle + 1 <= u32::MAX);」（`crates/singlefs-replay-prefix/src/gpu_gc_scoring_proofs.rs:47`）。推导：`middle < above <= records.len() <= u32::MAX`，所以 `middle + 1 <= u32::MAX`。删掉界（必红行 `gpu-score-binary-midpoint-needs-u32-bound`，第 412 行）之后，函数体里只有这一句断言需要它：两处递归调用的前提是同一行，跟着一起去掉了；调用方 `theorem_sorted_search_equals_full_record_walk` 自带 `sorted.len() <= u32::MAX`，多出来的前提不让任何东西断。所以落点是 `lower_bound_of_sorted_records` 里这一处，与第六段相同。
- 断言模的是不是 WGSL：WGSL 的中点是「let middle = below + (above - below) / 2u;」（`crates/singlefs-background-gpu/src/live_slot_kernel_searching_sorted_records.wgsl:93`），下一轮是「below = middle + 1u;」（`crates/singlefs-background-gpu/src/live_slot_kernel_searching_sorted_records.wgsl:95`），与模型同式（不是 `(below + above) / 2` 那种会先溢出的写法），所以模型里只有 `middle + 1` 这一步可能越 u32，断言盯的就是它。断言放在分支之前，两支都要它，比 WGSL 只在一支加一更严，不放宽。
- 反推：断言要是不承重，第 412 行改坏之后应当照样验过。任务 20692 的必红世界一格判这 93 行（含第 412 行）都在第六段的落点报错（日志末段成功句，转述）；这一行在 20692 里是复用的判定（同键），不是现判。

### 窗口常数（没打中）

- 窗口下沿「if start >= 65535 { start - 65535 } else { 0 }」（`crates/singlefs-replay-prefix/src/gpu_gc_scoring_proofs.rs:97`），跨度界「records[position].end - records[position].start <= 65535」（`crates/singlefs-replay-prefix/src/gpu_gc_scoring_proofs.rs:103`）。
- 推导（自己重推一遍）：下沿退 `W` 个槽时，被排在窗外的是起点 `< start − W` 的记录。`W = 65534`：起点 `<= start − 65535`，尾端 `<= 起点 + 65535 <= start`，与段 `[start, end)` 交集为零，不漏。`W = 65533`：反例输入 `start = 100000`、`end = 100002`、一条同盘未释放记录 `[34466, 100001)`（跨度 65535）：它贡献 1 个槽，下沿是 34467，这条记录被排在窗外，漏 1。所以证明钉住的是 `W >= 65534`，WGSL 用的「const LONGEST_SPAN_OF_AN_ALLOCATION_RECORD: u32 = 65535u;」（`crates/singlefs-background-gpu/src/live_slot_kernel_searching_sorted_records.wgsl:43`）在这个范围里。
- `GPU_GC_PROOFS.md` 第 14 行与源文件第 94、95 行的注释写的就是这个界（「至少退 65534 个」「65535 比证明要的宽一格」），与推导一致。判决第三节第 3 条给的出路是「验得过就把说明文件里那一句改成证明实际钉住的界」，做到了。不加「窄一格」那一行是对的：65534 是正确实现，给它配必红行会把正确的改动判红。

### 覆盖写第一条引理的按段 ensures（没打中，注释说宽一点，推的）

- 前提「0 <= start && start + data.len() <= old.len()」（`crates/singlefs-replay-prefix/src/partial_write_proofs.rs:110`）。三条按段结论在第 127–130 行，函数体第 134–137 行逐条 `=~=`。
- 三条合起来离了 `overwrite_fits` 推不出：写入段 `subrange(start, start + data.len())` 在起点为负或请求越过文件尾时落在序列外（vstd 的 `subrange` 只在下标合法时有长度与逐元素的公理），后缀段同理。必红行 `overwrite-exact-needs-fit`（第 417 行）删整条前提，任务 20692 判红、落点是这条引理。
- 说宽的一处：前缀那一条「overwritten_bytes(old, start, data).subrange(0, start) == old.subrange(0, start),」（`crates/singlefs-replay-prefix/src/partial_write_proofs.rs:128`）只要 `0 <= start <= old.len()`，不要请求整段在文件内。输入 `old = [1, 2, 3]`、`start = 2`、`data = [9, 9]`：`overwrite_fits` 不成立（2 + 2 > 3），写后序列是 `[1, 2, 9]`，前缀 `[1, 2]` 等于原值。源文件注释「按段说的三条（写入那一段恰是请求、前后两段恰是原值）要 overwrite_fits」（`crates/singlefs-replay-prefix/src/partial_write_proofs.rs:119`）与 README「这三条要 `overwrite_fits`」（`crates/singlefs-replay-prefix/src/README.md:156`）把三条一起说成都要它；说成「三条合起来要它，前缀那一条只要起点在文件内」更准。没起 Verus，推的；落点按函数算，必红表分不出是三条里哪一条断。
- 这三条没有消费者：全仓调这条引理的只有见证（「lemma_overwrite_changes_exactly_the_requested_bytes(old, 1, seq![8u8, 9]);」，`crates/singlefs-replay-prefix/src/partial_write_proofs.rs:206`，与第 207 行）。前提承重是靠新加的结论自己承重，不是别处用到了它；判决给的两条出路（删前提或改成承重）里选的是后一条，合判决字面。
- 改坏定义的两行（第 418、419 行）落点都含这条引理，`overwritten_bytes` 的定义被钉住了。

### G17 的量词（没打中）

- 新结论「forall|request: int| 0 < request <= remaining ==> #[trigger] allocation_stays_within_the_limit(held, request, limit),」（`crates/singlefs-replay-prefix/src/gpu_gc_proofs.rs:296`），谓词体「held + request <= limit」（`crates/singlefs-replay-prefix/src/gpu_gc_proofs.rs:286`）。
- 空真：`held < limit` 那一支 `remaining = limit − held >= 1`，请求取值非空，结论有内容；`held >= limit` 那一支 `remaining = 0`，结论空真，这一支的意思由第 293 行的等式（剩余为 0）担着。必红行 `gpu-gc-G17-positive-allocation-within-the-remaining-fits`（第 454 行）把上界放宽到 `remaining + 1`，两支都断（`held >= limit` 时取请求 1，`held + 1 <= limit` 不成立），所以它不是空真。
- 触发项：含量词变量 `request`，形状合法。全仓（`crates/` 下 `.rs`、`.md`、`.tsv`）只有定义（第 285 行）与这一条结论两处提到它，没有哪条证明实例化它；产品的调用点（`crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:331`）是普通 Rust，不读 ensures。它能由「held <= limit ==> held + remaining == limit,」（`crates/singlefs-replay-prefix/src/gpu_gc_proofs.rs:295`）加第 293 行推出，函数体的任何改坏都会先断第 293 行。所以它把注释那句话写成了机器可读的结论，没有新约束到产品，这正是判决要的「注释与 ensures 对齐」。

## 攻击面 2：75 行新必红与还没有反例罩着的 ensures

行号是 `crates/singlefs-replay-prefix/verus-must-red-worlds.tsv` 的行号（`awk -F'\t' 'NR>=411 && $0 !~ /^#/ && $0 != ""'` 现列，75 行）。落点、原文恰好一次、Verus 报错，任务 20692 已逐行判过（93 行全绿，其中 92 行复用）；这里判的是机器判不了的那一半：改坏的是不是名字说的那条、为什么断。全部读码推的，没起 Verus。表里「断的是哪条」一列的行号是被改的那份源文件的行号（G01–G20 是 `gpu_gc_proofs.rs`、G21–G30 是 `gpu_gc_capacity_proofs.rs`、覆盖写是 `partial_write_proofs.rs`、打分是 `gpu_gc_scoring_proofs.rs`）。

| 行 | 名字 | 改了什么 | 断的是哪条（推的） | 类 |
|---|---|---|---|---|
| 412 | gpu-score-binary-midpoint-needs-u32-bound | 删记录数的 u32 界 | 第 47 行断言 | 前提 |
| 413 | gpu-score-counter-bound-per-record-span | 上界 65535 改 65534 | 一条跨度 65535 的记录就超 | 改 ensures |
| 417 | overwrite-exact-needs-fit | 删 `overwrite_fits` | 第 127、129 行两段（见攻击面 1） | 前提 |
| 418 | overwrite-definition-start-shifted-by-one | 定义里写入段右移一格 | 第一条引理逐字节那一条，另三个函数 | 定义 |
| 419 | overwrite-definition-length-short-by-one | 定义里写入段少一格 | 同上，另三个函数 | 定义 |
| 420 | overwrite-last-write-wins | 结论右边换成只写第一次 | 第一次与最后一次内容不同时不等 | 改 ensures |
| 424 | gpu-gc-G01-confirmation-never-asks-cpu | 一致那一支 `ask_cpu: true` | 第 31、32 行 | 函数体 |
| 425 | gpu-gc-G01-cpu-judgment-restarts-the-count | 交 CPU 那一支计数 2 | 第 33、34 行 | 函数体 |
| 426 | gpu-gc-G01-second-disagreement-asks-cpu | 门槛减一去掉 | 上一轮分歧 1 次时走暂缓：第 31、33、35 行 | 函数体 |
| 427 | gpu-gc-G01-threshold-constant | 常量 2 改 3 | 同上一行 | 常量 |
| 433 | gpu-gc-G04-successful-round-updates-the-counts | 成功轮的表写成不变 | 空表上分歧一次就插入键，不等 | 改 ensures |
| 438 | gpu-gc-G06-other-entries-unchanged | 量词去掉「别的键」 | 本键在空表上分歧时从 0 变 1 | 改 ensures |
| 442 | gpu-gc-G07-disjoint-ranges-score-zero | 不相交条件放宽一格 | `[0,1)` 与 `[0,1)` 交 1 | 改 ensures |
| 443 | gpu-gc-G11-released-or-foreign-scores-zero | 前件加「起点在段尾之前」 | 同盘活记录相交时贡献为正 | 改 ensures |
| 447 | gpu-gc-G13-high-word-does-not-overflow | `<=` 改 `<` | 高字取 u32 最大、跨度 0 | 改 ensures |
| 450 | gpu-gc-G15-representable-input-is-not-refused | 输入 1 交 None | None 臂（对齐值 4 不越界） | 函数体 |
| 451 | gpu-gc-G15-alignment-constant | 常量 4 改 8 | 输入 1 交 8，规格写死 4 | 常量 |
| 454 | gpu-gc-G17-positive-allocation-within-the-remaining-fits | 请求上界加一 | 两支都断（攻击面 1） | 改 ensures |
| 459 | gpu-gc-G18-short-remainder-is-not-overrun | 余量够时也取起点加容量 | `end <= total` 与等式，或加法溢出 | 函数体 |
| 460 | gpu-gc-G20-counted-verdict-is-not-refused | 低字为 7 也拒 | None 臂（判定字是 1） | 函数体 |
| 466 | gpu-gc-G21-memory-limit-exact-fit-is-accepted | `<=` 改 `<` | 恰好装满时 `fits` 与规格不等 | 函数体 |
| 468 | gpu-gc-G21-footprint-overflow-is-refused | 占用溢出时放行 | 条目乘每项字节溢出时 | 函数体 |
| 470 | gpu-gc-G22-empty-room-is-refused | 去掉不足 4 的拒绝 | `bytes >= 4` | 函数体 |
| 472 | gpu-gc-G22-piece-leaves-the-fixed-bytes | 不扣固定占用 | 交回的分片 `chunk_fits` 不成立 | 函数体 |
| 478 | gpu-gc-G24-hold-adds-the-bytes | 无上限那一支交回原占用 | `total == held + bytes` | 函数体 |
| 482 | gpu-gc-G27-zero-stays-zero | 结论改成交 1 | 第 134 行已推出 0 | 改 ensures |
| 485 | gpu-gc-G28-group-count-is-not-truncated | 先截成 u16 | 条目多于 65535 组时覆盖不足 | 函数体 |
| 488 | gpu-gc-G29-largest-compares-both-tables | 最大绑定只取条目表 | 落点表更大时（条目 0、落点 5） | 函数体 |
| 491 | gpu-gc-G29-representable-total-is-not-refused | 总量 76 时拒 | None 臂（空表总量 76） | 函数体 |
| 492 | gpu-gc-G30-sum-overflow-is-refused | 和溢出时交回 `previous` | `total == previous + additional` | 函数体 |

抽的 30 行里没有一行改坏的不是名字说的那条，也没有落点说错的。名字形如 `-constant` 的三行把常量钉住了：规格 `aligned_input_size` 写死 4、G20 的 ensures 写死 1、G01 的 spec 写死「上一轮至少 1 次」，所以改常量会让 exec 体与写死的规格对不上。

75 行按改了什么分（推的，逐行读替换文分的）：改 ensures 本身 35 行（413、414、420、421、428–448 共 21 行、453–458 共 6 行、474、482、483、484），改函数体或常量 36 行，删前提 2 行（412、417），改定义 2 行（418、419）。其中 `gpu-gc-G<编号>-<说明>` 的 67 行是函数体 36、ensures 31，`GPU_GC_PROOFS.md` 第 83 行「多数是改函数体的一处」成立。

那一句的后半「推不出别的反例的派生条改 `ensures` 本身」（`crates/singlefs-replay-prefix/src/GPU_GC_PROOFS.md:83`）对 G02–G13 那 21 行不准：那几条是模型引理，函数体是空的，ensures 是 spec 定义的展开，别的反例推得出来，是改 spec 定义。例：「if record.released || record.device != device { 0 }」（`crates/singlefs-replay-prefix/src/gpu_gc_proofs.rs:158`）去掉 `record.released ||`，G11 第一条与 `lemma_search_prefix_matches_the_walk` 都会断。选改 ensures 本身是因为改定义一行会连带断几处、落点不止一个，不是因为推不出。这是说明文件措辞的宽窄，不影响哪一行红不红（推的）。

改 ensures 本身的行，钉住的是「这一条结论还在、没被放宽、界是紧的」：结论被删或被改宽，原文对不上、那一行先红。它们不证明「实现改坏了有这一条接住」，那一半要靠改函数体或改定义的行。

### 还没有反例罩着的 ensures（推的）

对第一轮两条腿的清单（正推第 5.3 节第 1–7 条、攻方件 4）逐项核过，那两份里点到的每一条现在都有一行；下面是那两份没点到、现在也没有一行把它弄断的：

1. 「first <= position < past ==> records[position].device == device,」（`crates/singlefs-replay-prefix/src/gpu_gc_scoring_proofs.rs:118`）与第 117 行的 `first <= past`：罩着候选窗口这条引理的只有第 384、387 行，两行都只断第三条（窗外贡献为零）。改坏 `record_key_before` 的盘号比较会断第 118 行，表里没有。
2. 「window.0 <= window.1,」（`crates/singlefs-replay-prefix/src/gpu_gc_scoring_proofs.rs:216`）与第 214、215 行两条下界：落在定理上的第 390、391 行只断总和那一条与前提。
3. `bytes_in_unit` 的 None 臂，含交集长度那一路新加的「&&& range_intersection_length(write_start as int, write_end as int, unit_start as int, unit_end as int) == 0」（`crates/singlefs-replay-prefix/src/file_range_proofs.rs:171`）与第 172 行：落在这个函数的第 275–278、407 行都只断 Some 臂。可补的一行：「if from < until {」（`crates/singlefs-replay-prefix/src/file_range_proofs.rs:187`）改成 `if from + 1 < until {`，输入写 `[0, 1)`、单元 `[0, 10)` 时交 None 而交集是 1 字节，两条一起断（推的，待主 agent 定要不要跑）。
4. 「overwritten_bytes(old, start, data).len() == old.len(),」（`crates/singlefs-replay-prefix/src/partial_write_proofs.rs:123`）与「data.len() == 0 ==> overwritten_bytes(old, start, data) == old,」（`crates/singlefs-replay-prefix/src/partial_write_proofs.rs:131`）：改定义的两行都不改长度，空请求时写入段本来就空，两条都不断。
5. `witness_file_range_boundaries`：必红表里没有一行落在它上（`grep -c` 得 0）；它是带 `assert` 的 exec 见证，不是 ensures，存量，列在这里备查。

## 攻击面 3：两份测试的新参照与九行变异

### 参照与被测还共用什么

- G27：参照「doubled <= u128::from(cap) && u128::from(cap) < doubled + 2,」（`crates/singlefs-replay-prefix/tests/gpu_gc_capacity_matches_independent_reference.rs:216`）与核的契约「2 * next <= failed_bytes < 2 * next + 2,」（`crates/singlefs-replay-prefix/src/gpu_gc_capacity_proofs.rs:134`）是同一条不等式；与函数体（第 138 行的整除）不同式。判决要的「与被测不同式」按被测函数体读是做到了；按「独立的校验路子」读，这条断言能抓到的 Verus 都先抓到，它的增量只是不跑 Verus 的 `cargo test` 也会红（第 8316 行变异证的就是这一点）。不算打中，记一条：参照与契约同式，换掉契约里那条不等式时这条测试不会替它把关。
- G29：每项字节数按产品的缓冲布局逐字段数，被测核里的 56、16、4 没有出现在参照里，这一点做到了。仍共用的两样：
  - 键长字数「const KEY_WORDS: u128 = 7;」（`crates/singlefs-replay-prefix/tests/gpu_gc_capacity_matches_independent_reference.rs:39`）是把产品「pub const KEY_WORDS: usize = 7;」（`crates/singlefs-background-gpu/src/region_record_kernels.rs:101`）抄成字面值（这份测试在重放前缀包里，读不到后台包的常量）。产品把键长改成 8 字时，核的 56 与这份参照一起错、一起绿。接住这一种的是后台包的 `gpu_gc_capacity_contracts_through_the_host.rs` 里 `verified_mapping_sizes_match_the_actual_packed_tables`（拿真打包出来的表比），不是这份测试。
  - 「assert_eq!(checked_gpu_mapping_table_bytes(0, 0), Some((76, 56)));」（`crates/singlefs-replay-prefix/tests/gpu_gc_capacity_matches_independent_reference.rs:247`）与核的见证「assert(tables == Some((76, 56)));」（`crates/singlefs-replay-prefix/src/gpu_gc_capacity_proofs.rs:214`）是同一对字面值。这一行是存量，不是这一轮加的。
  - 「u64::try_from(entry.max(location).max(index)).unwrap(),」（`crates/singlefs-replay-prefix/tests/gpu_gc_capacity_matches_independent_reference.rs:238`）把排序索引表也拿去比最大，核只比条目表与落点表；索引表每项 4 字节、条目表每项 56 字节、项数相同，索引表永远不会最大，两者等价，无害。
- 对齐输入的口径（空输入算 4 字节）两份测试与规格相同，第一轮已记，不重复。
- 别的参照（G23 的 `unwrap_or(u64::MAX).min(available)`、G28 的 `div_ceil`、G22 的穷举、u128 和）都与被测函数体不同式，没找到共用。

### 九行变异各红在哪条断言（推的）

行号是 `crates/mutations.tsv` 第 8309–8317 行（`grep -nF` 现取）。每一行只推第一处会失败的比对：

| 变异（mutations.tsv 行） | 测试 | 第一处失败的断言 | 输入 |
|---|---|---|---|
| 输入对齐只在最大值上拒（8309） | `gpu_budget_arithmetic_matches_a_u128_reference_at_full_width` | 算术测试第 21 行 | `u64::MAX − 2`：变异交 `Some(0)`，参照 None |
| 扫描区收下起点等于区尾（8310） | `gpu_slices_scans_and_independent_totals_match_wide_arithmetic` | 算术测试第 70 行 | 槽 0、跨度 0、区 `[0, 0)`、同盘未释放：变异交 true |
| 回读丢高字（8311） | `gpu_readback_rejects_unwritten_words_and_preserves_both_halves` | 算术测试第 111 行 | 值 `1 << 32`、判定字 1：变异交 0 |
| 第二次分歧不交 CPU（8312） | `gpu_reconciliation_matches_the_two_round_history_rule` | 算术测试第 129 行 | 历史 0（全分歧）第二轮：参照要交 CPU |
| 追加和溢出回绕（8313） | `chunk_admission_and_extension_match_full_width_arithmetic` | 容量测试第 84 行 | 绑定 4、条目字节 0、无上限、输入 1、追加 `u64::MAX`、条目 0：变异交 `Some(0)` |
| 分片不取四的倍数（8314） | `largest_piece_matches_exhaustive_search_and_is_maximal` | 容量测试第 132 行 | 绑定 5、无上限、派发 1：变异交 5，穷举交 4 |
| 预留恰到上限也拒（8315） | `holds_releases_and_waits_preserve_the_exact_ledger` | 容量测试第 168 行 | 占用 0、预留 0、上限 `Some(0)`：变异交 None |
| 折半向上取整（8316） | `memory_caps_never_expand_and_allocation_failure_makes_progress` | 容量测试第 215 行（新参照） | 占用 1：变异交 1，两倍 2 超过 1 |
| 映射条目每项少 4 字节（8317） | `mapping_buffers_and_dispatch_cover_their_exact_full_width_sizes` | 容量测试第 241 行 | 条目 0、落点 0：变异总量 72，参照 76 |

九行都红在点名的那条测试上，而且第 8309、8316 两行红在这一轮新加的东西上（新边界值 `u64::MAX − 2`、新参照），说明新加的部分本身承重。

读数这一侧：任务 20696 的日志里这九行都是「ITEM_DONE … judged=0 reused=1」、成功句写「红了（复用）」，是按键复用了更早的一次判定，那一趟没有现跑，日志里也就没有 panic 的位置；上表的断言位置只能是推的。要量的话在副本上对这九行起一次证红，看 panic 行（命令没写进「要主 agent 跑的命令」，主 agent 定要不要量）。

## 攻击面 4：说明文件的数与树

### 对得上的（量的，命令是只读的 `ls`、`grep -c`）

| 说明文件里的数 | 现查命令 | 现查得 |
|---|---|---|
| README 第 7 行「十份源文件」 | `ls crates/singlefs-replay-prefix/src/*.rs \| wc -l` | 10 |
| README 第 22 行「九个 `pub mod`」 | `grep -c '^pub mod' crates/singlefs-replay-prefix/src/lib.rs` | 9 |
| `GPU_GC_PROOFS.md` 第 87 行「4 个测试」、第 88 行「5 个测试」 | 两份测试各 `grep -c '#\[test\]'` | 4、5 |
| 第 90 行「82 个 `#[test]`（7、36、23、16）」 | 四个模块各 `grep -c '#\[test\]'` | 7、36、23、16 |
| 第 91 行「2 个测试」、第 92 行「5 个测试」 | 后台包两份测试各 `grep -c '#\[test\]'` | 2、5 |
| 第 93 行「各 6 个测试」 | 两份真卡测试各 `grep -c '#\[test\]'` | 6、6 |
| 第 83 行 G01–G30 各一行、三条见证、两条审计 | 必红表 `grep -c` 三种名字 | 30、3、2 |
| 第 88 行「43,050 组容量」 | 推的：绑定 41 × 固定 5 × 每条 5 × 条目缓冲 3 × 派发 2 × 上限 7 | 43050 |
| 第 81 行「26 条义务」 | 推的：exec 9、proof 13、常量 3、带 `decreases` 的 spec 1；新加的 `allocation_stays_within_the_limit` 不带 `decreases`，不添义务 | 26 |
| `.claude/gate.d/verus-pins.tsv` 第 50 行 214 | `git show HEAD:.claude/gate.d/verus-pins.tsv` 是 211，交集长度那一路加三个 proof fn | 211 + 3 |

`GPU_GC_PROOFS.md` 第 5 行、第 75 行点名的产品函数 `chunks_that_fit_on_the_gpu`、`chunk_fits_with_the_staging_of_its_input`、`largest_piece_input_bytes`、常量 `COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION` 都在 `crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs` 里（第 423、375、497、193 行）；`extended_gpu_chunk_input` 在 `crates/` 下除重放前缀包之外零命中。

### 打中 4-1：README 文件区间模块的义务数是旧的

「文件区间模块的 5 项 Verus 义务（两个产品入口、一个无前提边界见证、两个分区引理）」（`crates/singlefs-replay-prefix/src/README.md:109`）与「文件区间模块合计 9 项义务」（`crates/singlefs-replay-prefix/src/README.md:111`）读起来是现状句。树上：`grep -cE '^pub (proof )?fn' crates/singlefs-replay-prefix/src/file_range_proofs.rs` 得 19（exec 10、proof 9），另有两处 `by (nonlinear_arith)`；分区一类的引理现在是四条（`lemma_tail_batches_partition_the_released_suffix`、`lemma_adjacent_units_partition_the_write` 与交集长度那一路的两条）。按模块的 Verus 义务数这条腿没量，但 19 个被验函数对不上 9。第 119、126、148 行（「同日继续补齐…共 5 条义务」「本轮新增 6 个必红变异」「本模块新增 6 个被验函数」）是按批次累加的写法，同一个病。这几行不是这一轮加的，但 README 是这一轮被判的文件、正文第 4 条点名要核义务数；交集长度那一路往这个模块加了三条义务，没回头改这几处。

### 打中 4-2：`GPU_GC_PROOFS.md` 的 cargo 复跑命令交不上服务，真卡那条在现行规则下只会照样退 0

- 第 104、106、110 行只点 `--test` 目标，第 112 行同样只点目标，第 108 行只给筛选词、没有 `--exact`。服务的判法：「算点名的：cargo test / cargo t 的 `--` 之后带 --exact 与至少一个测试全名」（`task-server/task_server_lib.py:5342`），不点名的提交时拒，拒绝说明是「没点名到单条测试函数」。所以这五条照原样交，五条都在提交时被拒。第一轮判决第二节第 7 条「说明文件里的命令照现行写法改」，这一轮换掉了 `--manual`、加了范围声明，cargo 那五条还差点名这一步。这份 `task_server_lib.py` 此刻在别的会话手里改着（`git status` 是 `MM`），行号会挪，引的那一句以函数 `unnamed_run_problem` 的文档串为准。
- 第 112 行是真卡测试，经 `task-server/task submit` 交。现行规则：统一任务服务交的任务「不领卡、不分卡、不清场」（`.claude/rules/verification.md:50`），同一行写「崩溃判段之外用卡的测试交 GPU 服务代跑（接口待定）」（`.claude/rules/verification.md:50`）。这两份真卡测试没拿到卡时打 `NOT_RUN_NO_GPU_ADAPTER` 报本次未跑、「返回成功；交了卡却对不上本机的独显、或开不了判红」（`crates/singlefs-background-gpu-tests/tests/live_slot_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs:4`）。也就是：照第 112 行交（补上点名之后），「6 passed」是没上卡的六条。测试文件头另写着「带 `--test-threads 1`」（`crates/singlefs-background-gpu-tests/tests/live_slot_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs:5`），第 112 行也没带。
- 删掉的 `GPU_GC_SELFTEST.md` 里有一句读法是现状：真卡用例看日志里有没有 `NOT_RUN_NO_GPU_ADAPTER`、`GPU_DEVICE_LOST`，没有才不是「无设备返回成功」；另一句「预算过低的用例……不声称该用例派发了着色器」。`GPU_GC_PROOFS.md` 里 `grep -c 'NOT_RUN_NO_GPU_ADAPTER\|GPUS_HANDED_OVER'` 得 0，第 93 行只写「预算过低时排除设备」。判决第三节第 6 条要「其中还成立的现状句并进 `GPU_GC_PROOFS.md`」，这两句没并进来。
- 并进来了的（核过）：MIR 逐条目相同不代表外围已验证（第 65 行）、主机测试的参照不调被验函数（第 91 行）、有限枚举不替代精化（第 91 行）、两条审计行改坏了什么（第 83 行）。
- 四句：不分辨臂（这是说明文件，不是两条臂）；系统看得到（服务按命令字面判，测试按交没交卡判）；满足的是正文第 4 条「说明文件的数与树对不对得上」与检查单「带 --item 的命令与项」里「命令有没有漏项」那一分句；改法「补 `--exact` 与测试全名」修得了五条的提交，修不了第 112 行的卡（要换成交 GPU 服务的写法，或在接口定之前写明这条现在跑不了）。

## 攻击面 5：交集长度那一路

### 定义与两条分割引理的前提（没打中）

- 定义的结尾「if intersection_start < intersection_end { intersection_end - intersection_start } else { 0 }」（`crates/singlefs-replay-prefix/src/file_range_proofs.rs:162`）：最大起点到最小终点的正差，否则零。空、倒置、不相交都落到零，与第 156 行文档注释一致。
- 「requires unit_start <= boundary <= unit_end,」（`crates/singlefs-replay-prefix/src/file_range_proofs.rs:348`）两半都承重。分界越过单元尾：写 `[0, 10)`、单元 `[0, 5)`、分界 10，左边 `[0, 5)` 是 5，右边两段是 10 + 0，不等。分界早于单元头：写 `[0, 10)`、单元 `[5, 10)`、分界 0，左边 5，右边 0 + 10，不等。「requires write_start <= split <= write_end,」（`crates/singlefs-replay-prefix/src/file_range_proofs.rs:360`）对称：写 `[0, 5)`、分界 10、单元 `[0, 10)`，左 5、右 10 + 0；写 `[5, 10)`、分界 0、单元 `[0, 10)`，左 5、右 0 + 10。必红表第 405、406 行删的是整条前提；拆半删同样会断（推的），那两行罩得住。
- 写区间空或倒置（单元引理）、单元区间空或倒置（请求引理）时三项都是零，注释说的「可以为空或倒置」成立。见证第 371–382 行的十一条算过一遍都对，`4 = 1 + 3`（第 371–373 行）、`4 = 2 + 2`（第 374–376 行）与 README 第 183 行说的一致。

### 产品消费返回长度那几处与新 ensures（对得上）

- 「singlefs_replay_prefix::file_range_proofs::checked_bytes_in_indexed_unit(」（`crates/singlefs-core/src/write_request_split.rs:385`）的四个实参依次是 `self.offset.0`、`self.end_exclusive`、`unit.0`、`payload_capacity`，对上核的 `write_start`、`write_end`、`unit_index`、`capacity`。
- 「|(offset_in_the_payload, offset_in_the_bytes_written, length_in_bytes)| {」（`crates/singlefs-core/src/write_request_split.rs:393`）按核的返回次序（单元内偏移、请求内偏移、长度）解开，第 397 行把长度原样放进 `length_in_bytes`。所以 `length_in_bytes` 等于「&&& length == range_intersection_length(write_start as int, write_end as int, unit_index * capacity, unit_index * capacity + capacity)」（`crates/singlefs-replay-prefix/src/file_range_proofs.rs:226`）右边那个数；`expect` 那一支对应外层 None（地址溢出），不当成零长度，与 README 第 181 行一致。
- 新 ensures 由旧的两条推得出：Some 臂里「长度为正」加「逐字节恰在 `[起点+偏移, 起点+偏移+长度)` 里」已经把长度定死成交集的字节数，函数体的任何改坏都会先断旧的那几条。所以第 407、408 两行只能改 ensures 本身（加一），它们钉住「这条等式还在」，不证「实现错了它接得住」。这不是错，是新 ensures 的性质：它给消费者一个闭式，没有新约束到实现。
- 没有消费者：`range_intersection_length` 与两条分割引理在 `crates/` 下除这份文件外零命中（`grep -rn` 退 1）。两条引理只被同文件的见证调。README「规格与组合引理只涉及字节区间和返回长度」（`crates/singlefs-replay-prefix/src/README.md:183`）说得稍宽：两条引理的参数是数学整数端点，没有一条证明把它们接到 `bytes_written_into` 逐单元交回的长度上（例如「重写段里各单元的长度之和等于请求长度」）。要那样用，得另写一条把产品的单元区间代进去的引理（推的，没写）。

### 打中 5-1：「改 7 行」核不出，快照补丁漏了 3 行改动

- 正文 `research/prompts/_replay-prefix-backfill-code-r2-body.md` 的「实现今天的样子」一节与「这一轮攻什么」第 5 条两处，写交集长度那一路加 8 行、改 7 行（转述）。
- 量的：`changes.patch` 的必红表底版是 git 对象 `f46815c567e2fffe49dc4b1a989536036ca4797e`（`git cat-file -t` 得 blob）。拿它与 `git show HEAD:crates/singlefs-replay-prefix/verus-must-red-worlds.tsv` 比（`diff`，输出存在草稿目录），只有：第 324 行 `indexed-intersection-does-not-drop-written-bytes`、第 399 行 `overwrite-split-needs-valid-boundary`、第 400 行 `overwrite-contained-needs-payload-bounds` 三行改了（都只改第五段 `proof_errors` 由 1 到 2），另加第 402–409 行八行。改 3 行，不是 7 行。
- 快照里的 `changes-intersection-length.patch` 的必红表那一段只有八行加行，这三行改动不在里面；它的 `file_range_proofs.rs` 那一段与 `git diff HEAD` 逐行相同（排序后 `diff` 退 0），只有必红表那一段不全。
- 三行里第 324 行的条数变化说得通：那一行把交集换成 None，None 臂多了一条「长度为零」，多断一条。第 399、400 行是覆盖写的两行，交集长度那一路没碰 `partial_write_proofs.rs`，条数为什么变这条腿推不出。条数只报不判（第五段是最近一次观测的条数，「与实测不同只报不判」（`scripts/gate/verus-proof-check.py:100`）），所以不影响红不红；影响的是「被判过的改动」清单：这三行没在任何一份快照补丁里出现，第一轮与这一轮的判决都没有点到它们。
- 任务 20692 的必红世界一格点了 93 行，是新加的 83 行、上面三行、第 198 行，加上第 394–398、401 行六行存量；三行改动的结局都在里面（绿，复用）。

## 攻击面 6：实现员与回扫报的三处小问题

1. **必红表第 402–409 行没有自己的注释行：成立，范围比报的宽。** 第 394–401 行（覆盖写组合那八行，存量）同样没有自己的注释行，跟第 402–409 行一起挂在「# GPU 打分算法：实现偏移、裁剪漏算、重数、承重前提与归纳步的反例。」（`crates/singlefs-replay-prefix/verus-must-red-worlds.tsv:378`）底下（`grep -n '^#\|^$'` 列出第 378 行之后下一个注释在第 411 行）。门禁按行判、不读注释，不影响红不红；读表的人会把覆盖写与交集长度的十六行当成打分的反例。该改：第 394 行、第 402 行之前各加一行注释（覆盖写组合、交集长度），改的是注释行，锚点与判定不动。
2. **README 文件表 `file_range_proofs.rs` 那一行没写精确交集长度：成立。** 那一行的验证内容是「部分写交集的精确覆盖、两侧偏移与容量上界」（`crates/singlefs-replay-prefix/src/README.md:18`），说的是逐字节覆盖；精确长度规格与两条长度可加的引理只在第 181–183 行。该改，轻：表格那一格补「精确交集长度、单元分界与请求分界两侧长度可加」。
3. **`.claude/gate.d/verus-pins.tsv` 第 49 行注释带日期：成立，另有一处更该改的。**
   - 规则：共用约束「代码与脚本的注释、说明只写现状，不写历史」那一条的射程含 `.claude/gate.d/` 下的 `#` 注释（`.claude/agent-common.md@写`，那一条在第 42 行）。第 49 行「# 2026-11-18 交集长度组合」（`.claude/gate.d/verus-pins.tsv:49`）带日期，「整 crate 实测 214」是一次量的读数。
   - 门禁不拦它：comment-history 格的判法只取「取下面五类：.rs 与 .wgsl；.py；.sh；」（`scripts/gate/comment-history-check.py:25`）那几类加 README，不读 `.tsv`。规则的射程比判法宽，这一行与第 44–46 行（存量，同样带日期）都在缝里。这是判法的缺口，写进报告交主 agent，不归这一轮改。
   - 更该改的是第 48 行：「整 crate 实测 211 verified、0 errors」（`.claude/gate.d/verus-pins.tsv:48`）紧挨着 crate 行，而 crate 行此刻是 214。单读第 48 行会以为 211 是现状。第 44–49 行是一串按批次累加的账（155、160、186、211、214），按规则该收成一句说 crate 行那个数是什么、怎么数，账进 `records/`。第 49 行是这一轮加的，至少它与第 48 行这一轮该改；第 44–47 行存量交主 agent 定。

## 攻击面 7：检查单八格

| 格 | 判 | 证据 |
|---|---|---|
| 走不到的理由二次查实 | 成立 | 两份补丁的加行里搜 `todo!`、`unreachable!`、`unimplemented!`、`panic!`、`assert!`、`assert_eq!`、`.expect(`、`.unwrap(`、`assert(`：非测试段只有 Verus 的 `assert(`，在 `changes.patch` 第 376、412–414、418 行（对应 `gpu_gc_scoring_proofs.rs:47` 与 `partial_write_proofs.rs` 第 134–137、139 行），都是证明义务，普通构建擦掉；第 507 行的 `assert!` 在测试段；第 536 行是必红表的替换文。实现员列的五处与此相同 |
| 类型与名字 | 成立 | 新名字 `allocation_stays_within_the_limit`、`range_intersection_length`、两条 `lemma_*_intersection_lengths_add_up`、`witness_intersection_length_partitions` 都说得出是什么；测试新加的七个常量与三个函数（`WORD_BYTES`、`KEY_LENGTH_WORDS`、…、`mapping_entry_item_bytes`）没有缩写、序数与位置代称。`range_intersection_length` 的形参叫 `write_*`、`unit_*`，见证拿它算两个任意区间，名字偏向一种用法，不算错。exec 核的裸整数照判决不在这一轮 |
| 不兼容旧代码与旧格式 | 成立 | 两份补丁加行里搜 `legacy`、旧版、旧布局、`version`、`VERSION`：零命中 |
| 不新建判定库 | 成立 | 搜 `sqlite`、`rocksdb`、`RocksDB`、`.db`：零命中；必红表是登记表，判定进 prune-server（任务 20692 日志） |
| 所有任务多机可用 | 成立 | 两份测试只读编进来的常量；必红行只读 crate 里的文件；补丁加行里搜 `/tmp`、`HOME`、`home/` 零命中 |
| 测试优先多线程 | 成立 | 最重的 `chunk_admission_and_extension_match_full_width_arithmetic` 是 12 × 12 × 4 × 12 × 12 × 4 约 33 万次比较（推的）；任务 20675 的用时这条腿没读日志，正文转述是 0.00s、0.01s；不值得切片 |
| 带 --item 的命令与项 | 不成立 | 打中 4-2：`GPU_GC_PROOFS.md` 第 104、106、108、110、112 行五条 cargo 命令不点名、提交时被拒；第 112 行在现行规则下拿不到卡。项这一侧没漏：75 + 8 行必红名字在表里各一次（`cut -f1 \| sort \| uniq -d` 为空），九行变异名在 `crates/mutations.tsv` 里各一次（`grep -c -F` 各得 1），任务 20692 点的 93 项包住了新加与改动的每一行。`--list-items` 这条腿没跑（要 `--full-gate`，第一轮记过） |
| 新崩溃流多罩了什么 | 成立 | 两份补丁里 `crash-case:`、`enumerate_crash_states` 都是 0 次 |

## 没打中的形状

- u32 界：试了三种删法——删整条界（第 412 行）、把界放到 `u32::MAX + 1`、只删定理那一层的 `sorted.len() <= u32::MAX`。前两种断在第 47 行断言，第三种断在定理调二分处的前提（推的）。WGSL 的中点与加一各对了一遍，没找到模型与 WGSL 在 u32 上不一致的步。
- 窗口：W 取 65533、65534、65535 三值，起点取小于 65535 与大于等于 65535 两档（饱和那一支），推得 65534 是最小的不漏值，说明文件写对了。
- 覆盖写：起点为负、起点在文件内而请求越过尾、起点越过尾三种越界各推了一遍按段三条哪条断；只找到前缀那一条注释说宽，不是证明错。
- G17：两支各取请求下界、上界、上界加一；没有空真，触发项合法。
- 75 行：抽 30 行逐行推断在哪、名字对不对，没有一行改坏的不是名字说的那条；另 45 行只核了类别（改函数体、改 ensures、删前提、改定义）。
- 九行变异：每行推出第一处失败的比对与输入，都在点名测试里。
- 交集长度：两条前提各拆两半给反例；产品三处实参与解构次序逐个对；没找到长度对错位或空交集被当成地址溢出的路径。
- 检查单：两份补丁加行里按八格各搜一遍（关键词列在攻击面 7）。

## 这条腿自己的限度

- 没起 Verus、没起 cargo：攻击面 1–3、5 的「断在哪一条」全是读码推的；机器读数只用了主 agent 给了号的任务 20692、20696 的日志，而这两趟里这一题的行几乎都是按键复用的判定（20692 现判 1、复用 92；20696 这九行全复用），断言位置看不到。
- 「改 7 行」可能是正文用了别的数法（比如把文件里的改行也算进来）；这条腿只核得出必红表 HEAD 与 `changes.patch` 底版之间改了 3 行。
- `task-server/task_server_lib.py` 在别的会话手里改着，攻击面 4 引的那一句行号可能挪；结论按函数 `unnamed_cargo_test_problem` 的判法，不按行号。
- 没读同一轮另一条腿的产出（禁读清单）。第一轮两条腿的报告读了「没有反例罩着的」那两节，用来对清单。

## 要主 agent 跑的命令

1. 坐实打中 4-2（`GPU_GC_PROOFS.md` 第 104 行照原样交）。

```
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-replay-prefix 的测试目标 gpu_gc_arithmetic_matches_wide_integer_reference; 范围=1 个测试目标; 为什么=用 u128 参照复核 G01 与 G14 到 G20 的产品核' task-server/task submit -- cargo test -p singlefs-replay-prefix --test gpu_gc_arithmetic_matches_wide_integer_reference
```

   在哪起：仓根 `/home/fy5090/code/singlefs`。看什么：提交当场被拒、拒绝说明里有「没点名到单条测试函数」，打中 4-2 坐实；被收下并跑完（末行 `TASK_SERVER_RESULT` 的 `exit=0`），打中 4-2 的第一半推翻。

不再列别的：攻击面 2 第 3 条那一行可补的必红行与九行变异的证红位置都是「推的，待主 agent 定要不要量」，要量的话，前者在副本的必红表里加那一行后照门禁 formal-verification 带 `--item must-red:<名>` 交，后者照 `scripts/mutation/prove-red.sh --copy <副本> singlefs-replay-prefix <变异名…>` 交。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳。没往统一任务服务交任务，判决要引的数没复跑（命令在上一节）。
- 没建仓副本、没写模型、没建编译目录：草稿目录里只有文本（必红表两个版本的导出、两份 diff、两份任务日志拷贝与抽出的项名），没有要删的副本或编译目录。
- 没改任何被判文件；第 6 条三处与 `verus-pins.tsv` 第 44–47 行、comment-history 判法不读 `.tsv` 的缺口，都写在上面交主 agent。
- 没跑 `--list-items`、没核「这一轮之外」的说明文件段落（README 第 103–150 行只核了攻击面 4 点名的几行的数）。

什么现象会推翻这份报告的打中：第 1 条命令被服务收下（推翻 4-2 前半）；`GPU_GC_PROOFS.md` 别处有 `NOT_RUN_NO_GPU_ADAPTER` 的读法（推翻 4-2 后半，这条腿 `grep -c` 得 0）；有一份补丁或记录列出必红表那 3 行改动的出处、且正文的「改 7 行」能按某种数法数出（推翻 5-1）；按模块的 Verus 义务数恰好是 9（推翻 4-1，这条腿认为不可能是 9，因为被验函数有 19 个，推的）。
