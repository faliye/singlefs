# 事务员出的事实表（线索，主 agent 只现查了正文点名的那几处）

### replay-prefix/src/GPU_GC_PROOFS.md（+117 / -0，1 块）

**1 这一次改了什么**
新增整份：GPU GC 形式化验证的覆盖与边界说明（首行「# GPU GC 形式化验证的覆盖与边界」，117 行）：列 G01–G30 各条性质、边界（不证 WGSL 编译器、GPU 执行、设备 I/O）、必红表 gpu-gc-/gpu-score- 反例与复跑命令。

逐块：
- GPU_GC_PROOFS.md:1 +117/-0 〔〕 加：# GPU GC 形式化验证的覆盖与边界

**2 改动处注释里引到的编号**（行号：原话；regex 取 I-x.y、D/E/C 数字、「已定项」、「欠账」）
- 加行的注释里没有 D、E、C、I 起头的编号或「已定项」「欠账」。
- 另有该包自己的性质编号 G01…G30（共 22 个不同的 G 编号），不是决策分项编号。

**3 检查单八格的原始事实**（只查加行；「生产段」指文件里 `mod tests` 之前，写在 `mod tests` 之外的 `#[cfg(test)]` 辅助也算在里面，要看行上方的 cfg）
- 非 Rust 源：八格里「走不到、类型与名字、多线程」不适用；旧兼容、判定库、本机状态：加行里没有认旧版本的分支或新判定存放处（文本文件）。
- 带 --item 的命令与项 / 新崩溃流多罩了什么：加行（非注释）里提到 crash/崩溃的 1 处；本文件没有新加崩溃点重放流或崩溃用例。 重放前缀的新证明与必红表行走 `scripts/gate/verus-proof-check.py must-red-worlds` 的 `--item must-red:<行名>`（README.md 原文），登记在 verus-pins.tsv。

**4 测它的测试与变异**
- 本文件没有改到非测试函数，或不是 Rust 源（coverage.txt 无条目）。
- crates/mutations.tsv：指到本文件的行，新增 0 行（键新出现）、消失 0 行、行变 0 行（多为必红测试名随改名变）。

**5 线索，没核**
- GPU_GC_PROOFS.md:38、:85 点名 verus-must-red-worlds.tsv 的行名前缀，行名以 verus-must-red-worlds.tsv 为准，没逐条核对

### replay-prefix/src/GPU_GC_SELFTEST.md（+65 / -0，1 块）

**1 这一次改了什么**
新增整份：「GPU GC 20 项新增形式化验证的严格自测」（65 行），记两条审计变异 gpu-gc-audit-* 与自测用临时判定库的跑法。

逐块：
- GPU_GC_SELFTEST.md:1 +65/-0 〔〕 加：# GPU GC 20 项新增形式化验证的严格自测

**2 改动处注释里引到的编号**（行号：原话；regex 取 I-x.y、D/E/C 数字、「已定项」、「欠账」）
- 加行的注释里没有 D、E、C、I 起头的编号或「已定项」「欠账」。
- 另有该包自己的性质编号 G01…G20（共 8 个不同的 G 编号），不是决策分项编号。

**3 检查单八格的原始事实**（只查加行；「生产段」指文件里 `mod tests` 之前，写在 `mod tests` 之外的 `#[cfg(test)]` 辅助也算在里面，要看行上方的 cfg）
- 非 Rust 源：八格里「走不到、类型与名字、多线程」不适用；旧兼容、判定库、本机状态：加行里没有认旧版本的分支或新判定存放处（文本文件）。
- 带 --item 的命令与项 / 新崩溃流多罩了什么：加行（非注释）里提到 crash/崩溃的 0 处；本文件没有新加崩溃点重放流或崩溃用例。 重放前缀的新证明与必红表行走 `scripts/gate/verus-proof-check.py must-red-worlds` 的 `--item must-red:<行名>`（README.md 原文），登记在 verus-pins.tsv。

**4 测它的测试与变异**
- 本文件没有改到非测试函数，或不是 Rust 源（coverage.txt 无条目）。
- crates/mutations.tsv：指到本文件的行，新增 0 行（键新出现）、消失 0 行、行变 0 行（多为必红测试名随改名变）。

**5 线索，没核**
- GPU_GC_SELFTEST.md:1 标题写「20 项」而 verus-pins.tsv 注释与 GPU_GC_PROOFS.md 已写到 G30、「十个产品核」，两处数目口径不同，没核哪边过时
- GPU_GC_SELFTEST.md:14、:39、:44 是实验经过类叙述（「已加入」「原 G04 变异的锚点也已收窄」「临时服务初始没有判定记录，日志逐条记下现判、复用 0」），按共用约束「文档正文只写现状、历史放文末」应复核；它在 crates/ 下的 .md，不在代码注释射程内，没判

### replay-prefix/src/README.md（+30 / -3，3 块）

**1 这一次改了什么**
三处：L7 标题「七份」改「八份」源文件；L15 表里加 gpu_gc_proofs.rs 一行；L20 「另六份」改「另七份」；L150 起新增 26 行「固定长度覆盖写的字节规律」节（五项证明义务、边界、复跑命令）。

逐块：
- README.md:4 +1/-1 〔〕 加：## 八份源文件各负责什么 ｜ 删：## 七份源文件各负责什么
- README.md:12 +3/-2 〔〕 加：| [gpu_gc_proofs.rs](gpu_gc_proofs.rs) | 双核仲裁、预算、分片、区域准入和回读的八个纯函数入口 | [GPU GC 新增 ｜ 删：| [partial_write_proofs.rs](partial_write_proofs.rs) | `clas
- README.md:147 +26/-0 〔core 的 `CheckpointTxg` 是单字段 u64 newtype，派生 Ord。将 `〕 加：## 固定长度覆盖写的字节规律

**2 改动处注释里引到的编号**（行号：原话；regex 取 I-x.y、D/E/C 数字、「已定项」、「欠账」）
- 加行的注释里没有 D、E、C、I 起头的编号或「已定项」「欠账」。

**3 检查单八格的原始事实**（只查加行；「生产段」指文件里 `mod tests` 之前，写在 `mod tests` 之外的 `#[cfg(test)]` 辅助也算在里面，要看行上方的 cfg）
- 非 Rust 源：八格里「走不到、类型与名字、多线程」不适用；旧兼容、判定库、本机状态：加行里没有认旧版本的分支或新判定存放处（文本文件）。
- 带 --item 的命令与项 / 新崩溃流多罩了什么：加行（非注释）里提到 crash/崩溃的 0 处；本文件没有新加崩溃点重放流或崩溃用例。 重放前缀的新证明与必红表行走 `scripts/gate/verus-proof-check.py must-red-worlds` 的 `--item must-red:<行名>`（README.md 原文），登记在 verus-pins.tsv。

**4 测它的测试与变异**
- 本文件没有改到非测试函数，或不是 Rust 源（coverage.txt 无条目）。
- crates/mutations.tsv：指到本文件的行，新增 0 行（键新出现）、消失 0 行、行变 0 行（多为必红测试名随改名变）。

**5 线索，没核**
- README.md:7「八份源文件」、:20「另七份」与 src/ 下 .rs 实际 10 份（命令 ls crates/singlefs-replay-prefix/src/*.rs | wc -l 得 10：compaction_plan、file_range、gc_crash、gc_ring_interval、三份 gpu_gc、lib、partial_write、unlink）对不上；gpu_gc_capacity_proofs.rs 与 gpu_gc_scoring_proofs.rs 也没进 :15 附近的表，没核表内其余行

### replay-prefix/src/compaction_plan_proofs.rs（+2 / -2，2 块）

**1 这一次改了什么**
两处纯改名：L1440 证明函数 a_segment_holding_a_pinned_slot_is_not_a_sound_choice 改 segment_holding_a_pinned_slot_is_not_a_sound_choice；L1551 测试 a_batch_takes_… 改 batch_takes_…。不动 requires/ensures/decreases。

逐块：
- compaction_plan_proofs.rs:1437 +1/-1 〔proof fn one_movable_allocation_is_planned_and_not〕 加：proof fn segment_holding_a_pinned_slot_is_not_a_sound_choice(only: CompactionKer ｜ 删：proof fn a_segment_holding_a_pinned_slot_is_not_a_sound_choi
- compaction_plan_proofs.rs:1548 +1/-1 〔mod tests {〕 加：fn batch_takes_the_first_movable_units_up_to_the_batch_size() { ｜ 删：fn a_batch_takes_the_first_movable_units_up_to_the_batch_siz

**2 改动处注释里引到的编号**（行号：原话；regex 取 I-x.y、D/E/C 数字、「已定项」、「欠账」）
- 加行的注释里没有 D、E、C、I 起头的编号或「已定项」「欠账」。

**3 检查单八格的原始事实**（只查加行；「生产段」指文件里 `mod tests` 之前，写在 `mod tests` 之外的 `#[cfg(test)]` 辅助也算在里面，要看行上方的 cfg）
- 走不到的理由：生产段没有新加 unreachable!/panic!/expect/unwrap/assert!；测试段 assert/expect/panic/unwrap 共 0 处。
- `_ =>`：生产段没有新加，测试段 0 处。
- `as` 转换：生产段没有新加，测试段 0 处。
- 类型与名字：生产段新加的条目名 0 个。（错误成员/枚举成员的新增与改名见第 1 样逐块与概述）
- 不兼容旧代码与旧格式：加行没有命中（关键词 旧版|旧布局|旧格式|legacy|migrat|迁移|兼容旧|backward…）。
- 不新建判定库：加行没有命中（关键词 rocksdb|RocksDB|sqlite|prune.server|prun…）。
- 多机：本机独有状态：加行没有命中（关键词 hostname|/proc/|/sys/|env::var|std::env|…）。
- 多线程：加行没有命中（关键词 available_parallelism|--test-threads|ray…）。
- 带 --item 的命令与项 / 新崩溃流多罩了什么：加行（非注释）里提到 crash/崩溃的 0 处；本文件没有新加崩溃点重放流或崩溃用例。 重放前缀的新证明与必红表行走 `scripts/gate/verus-proof-check.py must-red-worlds` 的 `--item must-red:<行名>`（README.md 原文），登记在 verus-pins.tsv。

**4 测它的测试与变异**
- 改到/新加的函数名 2 个（非测试函数）：同文件测试模块引到的 0 个，tests/ 目录文件引到的 0 个，两处都没引到（grep 名字）的 2 个：one_movable_allocation_is_planned_and_not_reported_emptied、segment_holding_a_pinned_slot_is_not_a_sound_choice。逐名明细在草稿目录 coverage.txt。
- crates/mutations.tsv：指到本文件的行，新增 0 行（键新出现）、消失 0 行、行变 1 行（多为必红测试名随改名变）。
- Verus：加减行里 requires/ensures/decreases 的行 0 条（没有动）；新加 proof/spec fn：无。

**5 线索，没核**
- 没有。

### replay-prefix/src/file_range_proofs.rs（+7 / -5，3 块）

**1 这一次改了什么**
三处纯改名（见表）：L275 局部变量 _max_units→_maximum_units；L295 模式变量 a,b,n→offset_in_unit、offset_in_write、length；L533 闭包参数 i→offset_in_overlap。不动 requires/ensures/decreases。

逐块：
- file_range_proofs.rs:272 +2/-2 〔pub fn witness_file_range_boundaries() {〕 加：let _maximum_units = units_covering_size(u64::MAX, 1); ｜ 删：let _max_units = units_covering_size(u64::MAX, 1);
- file_range_proofs.rs:292 +2/-2 〔pub fn witness_file_range_boundaries() {〕 加：if let Some((offset_in_unit, offset_in_write, length)) = _adjacent { ｜ 删：if let Some((a, b, n)) = _adjacent {
- file_range_proofs.rs:530 +3/-1 〔mod tests {〕 加：.map(|offset_in_overlap| { ｜ 删：.map(|i| unit_start + in_unit + i)

**2 改动处注释里引到的编号**（行号：原话；regex 取 I-x.y、D/E/C 数字、「已定项」、「欠账」）
- 加行的注释里没有 D、E、C、I 起头的编号或「已定项」「欠账」。

**3 检查单八格的原始事实**（只查加行；「生产段」指文件里 `mod tests` 之前，写在 `mod tests` 之外的 `#[cfg(test)]` 辅助也算在里面，要看行上方的 cfg）
- 走不到的理由：生产段没有新加 unreachable!/panic!/expect/unwrap/assert!；测试段 assert/expect/panic/unwrap 共 0 处。
- `_ =>`：生产段没有新加，测试段 0 处。
- `as` 转换：生产段没有新加，测试段 0 处。
- 类型与名字：生产段新加的条目名 0 个。（错误成员/枚举成员的新增与改名见第 1 样逐块与概述）
- 不兼容旧代码与旧格式：加行没有命中（关键词 旧版|旧布局|旧格式|legacy|migrat|迁移|兼容旧|backward…）。
- 不新建判定库：加行没有命中（关键词 rocksdb|RocksDB|sqlite|prune.server|prun…）。
- 多机：本机独有状态：加行没有命中（关键词 hostname|/proc/|/sys/|env::var|std::env|…）。
- 多线程：加行没有命中（关键词 available_parallelism|--test-threads|ray…）。
- 带 --item 的命令与项 / 新崩溃流多罩了什么：加行（非注释）里提到 crash/崩溃的 0 处；本文件没有新加崩溃点重放流或崩溃用例。 重放前缀的新证明与必红表行走 `scripts/gate/verus-proof-check.py must-red-worlds` 的 `--item must-red:<行名>`（README.md 原文），登记在 verus-pins.tsv。

**4 测它的测试与变异**
- 改到/新加的函数名 1 个（非测试函数）：同文件测试模块引到的 1 个，tests/ 目录文件引到的 0 个，两处都没引到（grep 名字）的 0 个。逐名明细在草稿目录 coverage.txt。
- crates/mutations.tsv：指到本文件的行，新增 0 行（键新出现）、消失 0 行、行变 3 行（多为必红测试名随改名变）。
- Verus：加减行里 requires/ensures/decreases 的行 0 条（没有动）；新加 proof/spec fn：无。

**5 线索，没核**
- 没有。

### replay-prefix/src/gpu_gc_capacity_proofs.rs（+218 / -0，1 块）

**1 这一次改了什么**
新增整份（218 行）。头注释原话：「GPU GC 的切批准入、最大分片、显存账本和派发尺寸；产品调用同一份 exec 核。G21–G30 续接 gpu_gc_proofs；设备完成信号的真实性和外围容器不在本模块证明。」概括：Verus 里的 GpuChunkCapacity、gpu_chunk_fits、largest_gpu_piece_input、显存账本的占用/释放/等待后字节数、分配失败后上限折半、派发组数、映射表字节数、切批延长输入，共 11 个 exec 函数与一个边界见证。

逐块：
- gpu_gc_capacity_proofs.rs:1 +218/-0 〔〕 加：//! GPU GC 的切批准入、最大分片、显存账本和派发尺寸；产品调用同一份 exec 核。

**2 改动处注释里引到的编号**（行号：原话；regex 取 I-x.y、D/E/C 数字、「已定项」、「欠账」）
- 加行的注释里没有 D、E、C、I 起头的编号或「已定项」「欠账」。
- 另有该包自己的性质编号 G21…G30（共 10 个不同的 G 编号），不是决策分项编号。

**3 检查单八格的原始事实**（只查加行；「生产段」指文件里 `mod tests` 之前，写在 `mod tests` 之外的 `#[cfg(test)]` 辅助也算在里面，要看行上方的 cfg）
- 走不到的理由：生产段没有新加 unreachable!/panic!/expect/unwrap/assert!；测试段 assert/expect/panic/unwrap 共 0 处。
- `_ =>`：生产段没有新加，测试段 0 处。
- `as` 转换：生产段 3 处，测试段 0 处：
  - gpu_gc_capacity_proofs.rs:151：let wide = crate::file_range_proofs::units_covering_size(entries as u64, per_group as u64);
  - gpu_gc_capacity_proofs.rs:152：wide as u32
  - gpu_gc_capacity_proofs.rs:187：|| !chunk_fits((previous + additional) as u64, entries, *capacity),
- 类型与名字：生产段新加的条目名 12 个：struct GpuChunkCapacity、fn gpu_chunk_fits、fn largest_gpu_piece_input、fn tightened_gpu_memory_limit、fn checked_gpu_memory_hold、fn checked_gpu_memory_release、fn gpu_bytes_after_submission_wait、fn gpu_memory_cap_after_allocation_failure、fn gpu_workgroups_for_entries、fn checked_gpu_mapping_table_bytes、fn extended_gpu_chunk_input、fn witness_gpu_capacity_and_accounting_boundaries（错误成员/枚举成员的新增与改名见第 1 样逐块与概述）
- 不兼容旧代码与旧格式：加行没有命中（关键词 旧版|旧布局|旧格式|legacy|migrat|迁移|兼容旧|backward…）。
- 不新建判定库：加行没有命中（关键词 rocksdb|RocksDB|sqlite|prune.server|prun…）。
- 多机：本机独有状态：加行没有命中（关键词 hostname|/proc/|/sys/|env::var|std::env|…）。
- 多线程：加行没有命中（关键词 available_parallelism|--test-threads|ray…）。
- 带 --item 的命令与项 / 新崩溃流多罩了什么：加行（非注释）里提到 crash/崩溃的 0 处；本文件没有新加崩溃点重放流或崩溃用例。 重放前缀的新证明与必红表行走 `scripts/gate/verus-proof-check.py must-red-worlds` 的 `--item must-red:<行名>`（README.md 原文），登记在 verus-pins.tsv。

**4 测它的测试与变异**
- 改到/新加的函数名 13 个（非测试函数）：同文件测试模块引到的 0 个，tests/ 目录文件引到的 10 个，两处都没引到（grep 名字）的 3 个：chunk_fits、mapping_entries_at_least_one、witness_gpu_capacity_and_accounting_boundaries。逐名明细在草稿目录 coverage.txt。
- crates/mutations.tsv：指到本文件的行，新增 0 行（键新出现）、消失 0 行、行变 0 行（多为必红测试名随改名变）。
- Verus：新增整份，requires/ensures/decreases 全是新写的（计数见第二节）；新加 proof fn / spec fn：open spec fn chunk_fits、open spec fn mapping_entries_at_least_one

**5 线索，没核**
- gpu_gc_capacity_proofs.rs:151-152、:187 在 Verus 里用 as 做 u32/u64 互转（gpu_workgroups_for_entries 与 extended_gpu_chunk_input），没核 requires/ensures 是否逐字覆盖「as 不丢位」

### replay-prefix/src/gpu_gc_proofs.rs（+385 / -0，1 块）

**1 这一次改了什么**
新增整份（385 行）。头注释原话：「GPU GC 的主机端纯核与着色器算术模型。20 项核心性质见 src/README.md。仲裁、预算、分片、回读与区域准入的 exec 函数由产品直接调用；Map/Seq 与双字算术证明不等于验证 WGSL 编译器、GPU 执行或设备 I/O。」概括：两核连续不确定仲裁（reconcile_gpu_entry）、稀疏计数的更新交换律、活槽区间相交、记录切片求和、双字槽序与进位、活槽总数/输入字节/切批字节的 checked 运算、剩余显存、记录切片终点、区域准入、回读解码，以及两个见证。

逐块：
- gpu_gc_proofs.rs:1 +385/-0 〔〕 加：//! GPU GC 的主机端纯核与着色器算术模型。20 项核心性质见 src/README.md。

**2 改动处注释里引到的编号**（行号：原话；regex 取 I-x.y、D/E/C 数字、「已定项」、「欠账」）
- 加行的注释里没有 D、E、C、I 起头的编号或「已定项」「欠账」。
- 另有该包自己的性质编号 G01…G20（共 20 个不同的 G 编号），不是决策分项编号。

**3 检查单八格的原始事实**（只查加行；「生产段」指文件里 `mod tests` 之前，写在 `mod tests` 之外的 `#[cfg(test)]` 辅助也算在里面，要看行上方的 cfg）
- 走不到的理由：生产段没有新加 unreachable!/panic!/expect/unwrap/assert!；测试段 assert/expect/panic/unwrap 共 0 处。
- `_ =>`：生产段没有新加，测试段 0 处。
- `as` 转换：生产段 1 处，测试段 0 处：
  - gpu_gc_proofs.rs:333：if verdict != GPU_LIVE_SLOTS_COUNTED { None } else { Some(low as u64 + 4294967296u64 * high as u64) }
- 类型与名字：生产段新加的条目名 14 个：const GPU_UNCERTAIN_ROUNDS_BEFORE_CPU、const GPU_INPUT_ALIGNMENT、const GPU_LIVE_SLOTS_COUNTED、struct ReconciliationStep、fn reconcile_gpu_entry、struct ScoringRecord、fn checked_live_slot_totals、fn checked_gpu_input_bytes、fn checked_gpu_chunk_bytes、fn remaining_gpu_memory、fn next_gpu_record_slice_end、fn gpu_record_is_in_scan_region、fn decoded_gpu_live_slots、fn witness_gpu_gc_arithmetic_boundaries（错误成员/枚举成员的新增与改名见第 1 样逐块与概述）
- 不兼容旧代码与旧格式：加行没有命中（关键词 旧版|旧布局|旧格式|legacy|migrat|迁移|兼容旧|backward…）。
- 不新建判定库：加行没有命中（关键词 rocksdb|RocksDB|sqlite|prune.server|prun…）。
- 多机：本机独有状态：加行没有命中（关键词 hostname|/proc/|/sys/|env::var|std::env|…）。
- 多线程：加行没有命中（关键词 available_parallelism|--test-threads|ray…）。
- 带 --item 的命令与项 / 新崩溃流多罩了什么：加行（非注释）里提到 crash/崩溃的 0 处；本文件没有新加崩溃点重放流或崩溃用例。 重放前缀的新证明与必红表行走 `scripts/gate/verus-proof-check.py must-red-worlds` 的 `--item must-red:<行名>`（README.md 原文），登记在 verus-pins.tsv。

**4 测它的测试与变异**
- 改到/新加的函数名 33 个（非测试函数）：同文件测试模块引到的 0 个，tests/ 目录文件引到的 9 个，两处都没引到（grep 名字）的 24 个：aligned_input_size、cpu_is_needed、in_slice、lemma_agreement_restarts_the_disagreement_history、lemma_count_updates_preserve_the_sparse_counter_invariant、lemma_distinct_entry_updates_commute、lemma_duplicate_records_keep_their_multiplicity、lemma_failed_batch_keeps_all_counts_and_cpu_calls、lemma_live_slot_intersection_bounds、lemma_record_slices_preserve_the_score、lemma_released_and_foreign_records_do_not_score、lemma_splitting_a_segment_preserves_live_slot_contributions、lemma_two_disagreements_trigger_exactly_one_cpu_judgment、lemma_two_word_slot_order_is_u64_order、lemma_two_word_span_addition_carries_exactly_once、next_uncertain_count、overlap_slots、record_contribution、record_slice_sum、round_state…另 4 个。逐名明细在草稿目录 coverage.txt。
- crates/mutations.tsv：指到本文件的行，新增 0 行（键新出现）、消失 0 行、行变 0 行（多为必红测试名随改名变）。
- Verus：新增整份，requires/ensures/decreases 全是新写的（计数见第二节）；新加 proof fn / spec fn：open spec fn next_uncertain_count、open spec fn cpu_is_needed、proof fn lemma_two_disagreements_trigger_exactly_one_cpu_judgment、proof fn lemma_agreement_restarts_the_disagreement_history、open spec fn count_of、open spec fn update_count、open spec fn round_state、proof fn lemma_failed_batch_keeps_all_counts_and_cpu_calls、proof fn lemma_distinct_entry_updates_commute、proof fn lemma_count_updates_preserve_the_sparse_counter_invariant、open spec fn overlap_slots、proof fn lemma_live_slot_intersection_bounds、proof fn lemma_splitting_a_segment_preserves_live_slot_contributions、open spec fn record_contribution、open spec fn record_slice_sum、proof fn lemma_record_slices_preserve_the_score、proof fn lemma_duplicate_records_keep_their_multiplicity、proof fn lemma_released_and_foreign_records_do_not_score、open spec fn words_value、proof fn lemma_two_word_slot_order_is_u64_order、proof fn lemma_two_word_span_addition_carries_exactly_once、open spec fn aligned_input_size、open spec fn in_slice、proof fn witness_gpu_gc_protocol_and_score_premises

**5 线索，没核**
- 没有。

### replay-prefix/src/gpu_gc_scoring_proofs.rs（+271 / -0，1 块）

**1 这一次改了什么**
新增整份（271 行）。头注释原话：「两份 GPU 活槽打分算法的精确等价证明：二分边界、候选裁剪、逐槽计数与排列。这里验证算法模型；WGSL 到此模型的翻译、主机排序/打包与 GPU 执行仍是未证边界。」概括：有序记录下界二分、候选窗口不丢贡献、查找前缀等于全记录走读、排列保持走读结果、计数不超 GPU 计数器，含 7 处 decreases。

逐块：
- gpu_gc_scoring_proofs.rs:1 +271/-0 〔〕 加：//! 两份 GPU 活槽打分算法的精确等价证明：二分边界、候选裁剪、逐槽计数与排列。

**2 改动处注释里引到的编号**（行号：原话；regex 取 I-x.y、D/E/C 数字、「已定项」、「欠账」）
- 加行的注释里没有 D、E、C、I 起头的编号或「已定项」「欠账」。

**3 检查单八格的原始事实**（只查加行；「生产段」指文件里 `mod tests` 之前，写在 `mod tests` 之外的 `#[cfg(test)]` 辅助也算在里面，要看行上方的 cfg）
- 走不到的理由：生产段没有新加 unreachable!/panic!/expect/unwrap/assert!；测试段 assert/expect/panic/unwrap 共 0 处。
- `_ =>`：生产段没有新加，测试段 0 处。
- `as` 转换：生产段没有新加，测试段 0 处。
- 类型与名字：生产段新加的条目名 0 个。（错误成员/枚举成员的新增与改名见第 1 样逐块与概述）
- 不兼容旧代码与旧格式：加行没有命中（关键词 旧版|旧布局|旧格式|legacy|migrat|迁移|兼容旧|backward…）。
- 不新建判定库：加行没有命中（关键词 rocksdb|RocksDB|sqlite|prune.server|prun…）。
- 多机：本机独有状态：加行没有命中（关键词 hostname|/proc/|/sys/|env::var|std::env|…）。
- 多线程：加行没有命中（关键词 available_parallelism|--test-threads|ray…）。
- 带 --item 的命令与项 / 新崩溃流多罩了什么：加行（非注释）里提到 crash/崩溃的 0 处；本文件没有新加崩溃点重放流或崩溃用例。 重放前缀的新证明与必红表行走 `scripts/gate/verus-proof-check.py must-red-worlds` 的 `--item must-red:<行名>`（README.md 原文），登记在 verus-pins.tsv。

**4 测它的测试与变异**
- 改到/新加的函数名 17 个（非测试函数）：同文件测试模块引到的 0 个，tests/ 目录文件引到的 0 个，两处都没引到（grep 名字）的 17 个：earliest_candidate_start、is_lower_bound、lemma_candidate_window_keeps_every_contribution、lemma_counting_slots_equals_interval_intersection、lemma_every_walk_prefix_fits_the_gpu_counter、lemma_permuting_records_preserves_the_walk、lemma_record_sum_is_a_fold、lemma_search_prefix_matches_the_walk、lower_bound_of_sorted_records、record_key_before、records_are_sorted、searched_slot_sum、slots_counted_one_by_one、theorem_sorted_search_equals_full_record_walk、valid_scoring_records、witness_scoring_search_with_duplicates_and_crossing_records、也不在。逐名明细在草稿目录 coverage.txt。
- crates/mutations.tsv：指到本文件的行，新增 0 行（键新出现）、消失 0 行、行变 0 行（多为必红测试名随改名变）。
- Verus：新增整份，requires/ensures/decreases 全是新写的（计数见第二节）；新加 proof fn / spec fn：open spec fn record_key_before、open spec fn records_are_sorted、open spec fn is_lower_bound、proof fn lower_bound_of_sorted_records、open spec fn slots_counted_one_by_one、proof fn lemma_counting_slots_equals_interval_intersection、open spec fn earliest_candidate_start、open spec fn valid_scoring_records、proof fn lemma_candidate_window_keeps_every_contribution、open spec fn searched_slot_sum、proof fn lemma_search_prefix_matches_the_walk、proof fn lemma_record_sum_is_a_fold、proof fn lemma_permuting_records_preserves_the_walk、proof fn theorem_sorted_search_equals_full_record_walk、proof fn lemma_every_walk_prefix_fits_the_gpu_counter、proof fn witness_scoring_search_with_duplicates_and_crossing_records

**5 线索，没核**
- 没有。

### replay-prefix/src/lib.rs（+15 / -12，13 块）

**1 这一次改了什么**
三类：L36 加 pub mod gpu_gc_capacity_proofs 等三行模块声明；一个 spec 函数 a_later_record_of_the_same_publish_is_readable 改名 later_record_of_the_same_publish_is_readable（定义 L278 与 L356、L379、L1733、L2161、L3130 共 6 处引用）；L4118、L4135 两个证明函数与 L4193、L4244、L4301、L4323 四个测试去掉 a_ 前缀。不动 requires/ensures/decreases 的文字，但 ensures 里的被调名随改名变了（L3130、L2161）。

逐块：
- lib.rs:33 +3/-0 〔pub mod compaction_plan_proofs;〕 加：pub mod gpu_gc_capacity_proofs;
- L275–L4320：12 块纯换名（测试名去 a_/an_ 前缀或标识符换名，各 ±1~3 行），例：L275 pub open spec fn a_later_record_of_the_s→pub open spec fn later_record_of_the_sam；L353 && a_later_record_of_the_same_publish_is→&& later_record_of_the_same_publish_is_r

**2 改动处注释里引到的编号**（行号：原话；regex 取 I-x.y、D/E/C 数字、「已定项」、「欠账」）
- 加行的注释里没有 D、E、C、I 起头的编号或「已定项」「欠账」。

**3 检查单八格的原始事实**（只查加行；「生产段」指文件里 `mod tests` 之前，写在 `mod tests` 之外的 `#[cfg(test)]` 辅助也算在里面，要看行上方的 cfg）
- 走不到的理由：生产段没有新加 unreachable!/panic!/expect/unwrap/assert!；测试段 assert/expect/panic/unwrap 共 0 处。
- `_ =>`：生产段没有新加，测试段 0 处。
- `as` 转换：生产段没有新加，测试段 0 处。
- 类型与名字：生产段新加的条目名 0 个。（错误成员/枚举成员的新增与改名见第 1 样逐块与概述）
- 不兼容旧代码与旧格式：加行没有命中（关键词 旧版|旧布局|旧格式|legacy|migrat|迁移|兼容旧|backward…）。
- 不新建判定库：加行没有命中（关键词 rocksdb|RocksDB|sqlite|prune.server|prun…）。
- 多机：本机独有状态：加行没有命中（关键词 hostname|/proc/|/sys/|env::var|std::env|…）。
- 多线程：加行没有命中（关键词 available_parallelism|--test-threads|ray…）。
- 带 --item 的命令与项 / 新崩溃流多罩了什么：加行（非注释）里提到 crash/崩溃的 0 处；本文件没有新加崩溃点重放流或崩溃用例。 重放前缀的新证明与必红表行走 `scripts/gate/verus-proof-check.py must-red-worlds` 的 `--item must-red:<行名>`（README.md 原文），登记在 verus-pins.tsv。

**4 测它的测试与变异**
- 改到/新加的函数名 11 个（非测试函数）：同文件测试模块引到的 0 个，tests/ 目录文件引到的 0 个，两处都没引到（grep 名字）的 11 个：a_publish_of_two_records_is_whole、fails_a_check_before_the_named_units、is_a_later_record_of_the_same_instance、later_record_of_the_same_publish_is_readable、lemma_a_publish_end_that_passed_the_fourth_reader_rule_closes_its_txg、lemma_the_landed_record_closes_its_txg、passes_every_check_before_the_named_units、publish_of_two_records_is_whole、readable_record_of_the_same_publish_follows、record_without_the_last_flag_alone_is_not_a_concatenation、witness_nonempty_replay_and_its_fixed_point。逐名明细在草稿目录 coverage.txt。
- crates/mutations.tsv：指到本文件的行，新增 0 行（键新出现）、消失 0 行、行变 19 行（多为必红测试名随改名变）。
- Verus：加减行里 requires/ensures/decreases 的行 0 条（没有动）；新加 proof/spec fn：open spec fn later_record_of_the_same_publish_is_readable<RootSource>。

**5 线索，没核**
- 没有。

### replay-prefix/src/partial_write_proofs.rs（+149 / -41，5 块）

**1 这一次改了什么**
五块：L2 头注释改写（原「只证明决策」改成「证明计划决策与固定长度覆盖写的抽象字节规律」）；L45-L75 形参名 ws/we/us/ue 改 write_start/write_end/unit_start/unit_end（含 requires 里同名）；L78 起新增 115 行：spec overwrite_fits、overwritten_bytes，证明 lemma_overwrite_changes_exactly_the_requested_bytes、lemma_overwriting_the_same_range_keeps_only_the_last_write、lemma_disjoint_overwrites_commute、lemma_whole_unit_overwrite_does_not_depend_on_old_bytes 与见证 witness_overwrite_laws_and_overlapping_write_order；L271、L304 测试局部变量改名。

逐块：
- partial_write_proofs.rs:1 +1/-1 〔〕 加：//! 证明计划决策与固定长度覆盖写的抽象字节规律；实际读写到字节模型的精化仍由调用方负责。 ｜ 删：//! 这里只证明决策；单元端点换算、旧尾单元是否不满的判定及实际读写由调用方负责。
- partial_write_proofs.rs:42 +13/-13 〔pub open spec fn byte_in_range(byte: int, start: u〕 加：pub open spec fn unit_is_touched(write_start: u64, write_end: u64, unit_start: u ｜ 删：pub open spec fn unit_is_touched(ws: u64, we: u64, us: u64, 
- partial_write_proofs.rs:75 +115/-17 〔pub fn classify_partial_write_unit(has_old: bool, 〕 加：pub proof fn lemma_whole_coverage_means_every_unit_byte_is_written(write_start:  ｜ 删：pub proof fn lemma_whole_coverage_means_every_unit_byte_is_w
- partial_write_proofs.rs:268 +14/-7 〔mod tests {〕 加：for write_start in 0..7 { ｜ 删：for ws in 0..7 {
- partial_write_proofs.rs:301 +6/-3 〔mod tests {〕 加：assert_eq!(reads_old, old && written < unit_end - unit_start); ｜ 删：assert_eq!(reads_old, old && written < ue - us);

**2 改动处注释里引到的编号**（行号：原话；regex 取 I-x.y、D/E/C 数字、「已定项」、「欠账」）
- 加行的注释里没有 D、E、C、I 起头的编号或「已定项」「欠账」。

**3 检查单八格的原始事实**（只查加行；「生产段」指文件里 `mod tests` 之前，写在 `mod tests` 之外的 `#[cfg(test)]` 辅助也算在里面，要看行上方的 cfg）
- 走不到的理由：生产段没有新加 unreachable!/panic!/expect/unwrap/assert!；测试段 assert/expect/panic/unwrap 共 3 处。
- `_ =>`：生产段没有新加，测试段 0 处。
- `as` 转换：生产段没有新加，测试段 1 处。
- 类型与名字：生产段新加的条目名 1 个：fn classify_partial_write_unit（错误成员/枚举成员的新增与改名见第 1 样逐块与概述）
- 不兼容旧代码与旧格式：加行没有命中（关键词 旧版|旧布局|旧格式|legacy|migrat|迁移|兼容旧|backward…）。
- 不新建判定库：加行没有命中（关键词 rocksdb|RocksDB|sqlite|prune.server|prun…）。
- 多机：本机独有状态：加行没有命中（关键词 hostname|/proc/|/sys/|env::var|std::env|…）。
- 多线程：加行没有命中（关键词 available_parallelism|--test-threads|ray…）。
- 带 --item 的命令与项 / 新崩溃流多罩了什么：加行（非注释）里提到 crash/崩溃的 0 处；本文件没有新加崩溃点重放流或崩溃用例。 重放前缀的新证明与必红表行走 `scripts/gate/verus-proof-check.py must-red-worlds` 的 `--item must-red:<行名>`（README.md 原文），登记在 verus-pins.tsv。

**4 测它的测试与变异**
- 改到/新加的函数名 13 个（非测试函数）：同文件测试模块引到的 1 个，tests/ 目录文件引到的 0 个，两处都没引到（grep 名字）的 12 个：byte_in_range、lemma_disjoint_overwrites_commute、lemma_overwrite_changes_exactly_the_requested_bytes、lemma_overwriting_the_same_range_keeps_only_the_last_write、lemma_touched_unit_has_exactly_a_nonempty_intersection、lemma_whole_coverage_means_every_unit_byte_is_written、lemma_whole_unit_overwrite_does_not_depend_on_old_bytes、overwrite_fits、overwritten_bytes、unit_is_covered、unit_is_touched、witness_overwrite_laws_and_overlapping_write_order。逐名明细在草稿目录 coverage.txt。
- crates/mutations.tsv：指到本文件的行，新增 0 行（键新出现）、消失 0 行、行变 4 行（多为必红测试名随改名变）。
- Verus：加减行里 requires/ensures/decreases 的行 15 条：-    requires ws < we, us < ue,；+    requires write_start < write_end, unit_start < unit_end,；-    requires ws < we, us < ue,；+    requires write_start < write_end, unit_start < unit_end,；-    requires ws < we, us < ue,；+    requires write_start < write_end, unit_start < unit_end,；+    requires overwrite_fits(old, start, data),；+    ensures；新加 proof/spec fn：open spec fn unit_is_touched、open spec fn unit_is_covered、proof fn lemma_whole_coverage_means_every_unit_byte_is_written、proof fn lemma_touched_unit_has_exactly_a_nonempty_intersection、open spec fn overwrite_fits、open spec fn overwritten_bytes、proof fn lemma_overwrite_changes_exactly_the_requested_bytes、proof fn lemma_overwriting_the_same_range_keeps_only_the_last_write、proof fn lemma_disjoint_overwrites_commute、proof fn lemma_whole_unit_overwrite_does_not_depend_on_old_bytes、proof fn witness_overwrite_laws_and_overlapping_write_order。

**5 线索，没核**
- 没有。

### replay-prefix/src/unlink_proofs.rs（+66 / -66，8 块）

**1 这一次改了什么**
纯改名：证明与测试里循环/量词变量 i→step_index、q→quotient 等，共 8 块 66 行换 66 行；不动 requires/ensures/decreases 的结构。

逐块：
- unlink_proofs.rs:92 +7/-7 〔pub proof fn lemma_unlink_plan_shape(remaining: na〕 加：forall|step_index: int| 0 <= step_index < unlink_plan(remaining, batch).len() -  ｜ 删：forall|i: int| 0 <= i < unlink_plan(remaining, batch).len() 
- unlink_proofs.rs:110 +11/-11 〔pub proof fn lemma_unlink_plan_shape(remaining: na〕 加：assert forall|step_index: int| 0 <= step_index < plan.len() - 1 implies { ｜ 删：assert forall|i: int| 0 <= i < plan.len() - 1 implies {
- unlink_proofs.rs:159 +5/-5 〔pub proof fn lemma_unlink_plan_length(remaining: n〕 加：/// 第 7 条 ①「计划相等」：做完第 step_index 批（不是最后一批）之后从它留下的单元数续做，算出的整串与不崩溃一路做下来的后半截逐步相同。 ｜ 删：/// 第 7 条 ①「计划相等」：做完第 i 批（不是最后一批）之后从它留下的单元数续做，算出的整串与不崩溃一路做下来
- unlink_proofs.rs:175 +4/-4 〔pub proof fn lemma_continued_plan_equals_the_rest(〕 加：if step_index == 0 { ｜ 删：if i == 0 {
- unlink_proofs.rs:189 +6/-6 〔pub proof fn lemma_each_data_unit_released_exactly〕 加：(unit < remaining) <==> exists|step_index: int| 0 <= step_index < unlink_plan(re ｜ 删：(unit < remaining) <==> exists|i: int| 0 <= i < unlink_plan(
- unlink_proofs.rs:209 +17/-17 〔pub proof fn lemma_each_data_unit_released_exactly〕 加：assert forall|step_index: int| 1 <= step_index < plan.len() implies #[trigger] p ｜ 删：assert forall|i: int| 1 <= i < plan.len() implies #[trigger]
- unlink_proofs.rs:276 +14/-14 〔pub proof fn lemma_deletion_batch_rewrites_only_th〕 加：let quotient = kept / span; ｜ 删：let q = kept / span;
- unlink_proofs.rs:361 +2/-2 〔mod tests {〕 加：for (step_index, &(_, kept)) in steps.iter().enumerate().take(steps.len() - 1) { ｜ 删：for (i, &(_, kept)) in steps.iter().enumerate().take(steps.l

**2 改动处注释里引到的编号**（行号：原话；regex 取 I-x.y、D/E/C 数字、「已定项」、「欠账」）
- 加行的注释里没有 D、E、C、I 起头的编号或「已定项」「欠账」。

**3 检查单八格的原始事实**（只查加行；「生产段」指文件里 `mod tests` 之前，写在 `mod tests` 之外的 `#[cfg(test)]` 辅助也算在里面，要看行上方的 cfg）
- 走不到的理由：生产段没有新加 unreachable!/panic!/expect/unwrap/assert!；测试段 assert/expect/panic/unwrap 共 1 处。
- `_ =>`：生产段没有新加，测试段 0 处。
- `as` 转换：生产段没有新加，测试段 0 处。
- 类型与名字：生产段新加的条目名 0 个。（错误成员/枚举成员的新增与改名见第 1 样逐块与概述）
- 不兼容旧代码与旧格式：加行没有命中（关键词 旧版|旧布局|旧格式|legacy|migrat|迁移|兼容旧|backward…）。
- 不新建判定库：加行没有命中（关键词 rocksdb|RocksDB|sqlite|prune.server|prun…）。
- 多机：本机独有状态：加行没有命中（关键词 hostname|/proc/|/sys/|env::var|std::env|…）。
- 多线程：加行没有命中（关键词 available_parallelism|--test-threads|ray…）。
- 带 --item 的命令与项 / 新崩溃流多罩了什么：加行（非注释）里提到 crash/崩溃的 0 处；本文件没有新加崩溃点重放流或崩溃用例。 重放前缀的新证明与必红表行走 `scripts/gate/verus-proof-check.py must-red-worlds` 的 `--item must-red:<行名>`（README.md 原文），登记在 verus-pins.tsv。

**4 测它的测试与变异**
- 改到/新加的函数名 5 个（非测试函数）：同文件测试模块引到的 0 个，tests/ 目录文件引到的 0 个，两处都没引到（grep 名字）的 5 个：lemma_continued_plan_equals_the_rest、lemma_deletion_batch_rewrites_only_the_cut_path、lemma_each_data_unit_released_exactly_once、lemma_unlink_plan_length、lemma_unlink_plan_shape。逐名明细在草稿目录 coverage.txt。
- crates/mutations.tsv：指到本文件的行，新增 0 行（键新出现）、消失 0 行、行变 1 行（多为必红测试名随改名变）。
- Verus：加减行里 requires/ensures/decreases 的行 0 条（没有动）；新加 proof/spec fn：proof fn lemma_continued_plan_equals_the_rest。

**5 线索，没核**
- 没有。

### replay-prefix/tests/gpu_gc_arithmetic_matches_wide_integer_reference.rs（+136 / -0，1 块）

**1 这一次改了什么**
新增整份（136 行，4 个 #[test]）。首行 use singlefs_replay_prefix::gpu_gc_proofs::*;，无文件头注释。概括：拿 u128 参照在 EDGES 九个边界值上对拍 checked_gpu_input_bytes / chunk 字节 / 切片终点 / 区域准入 / 活槽总数 / 回读解码 / 两核仲裁。

逐块：
- gpu_gc_arithmetic_matches_wide_integer_reference.rs:1 +136/-0 〔〕 加：use singlefs_replay_prefix::gpu_gc_proofs::*;

**2 改动处注释里引到的编号**（行号：原话；regex 取 I-x.y、D/E/C 数字、「已定项」、「欠账」）
- 加行的注释里没有 D、E、C、I 起头的编号或「已定项」「欠账」。

**3 检查单八格的原始事实**（只查加行；「生产段」指文件里 `mod tests` 之前，写在 `mod tests` 之外的 `#[cfg(test)]` 辅助也算在里面，要看行上方的 cfg）
- 走不到的理由：生产段没有新加 unreachable!/panic!/expect/unwrap/assert!；测试段 assert/expect/panic/unwrap 共 14 处。
- `_ =>`：生产段没有新加，测试段 0 处。
- `as` 转换：生产段没有新加，测试段 1 处。
- 类型与名字：生产段新加的条目名 5 个：const EDGES、fn gpu_budget_arithmetic_matches_a_u128_reference_at_full_width、fn gpu_slices_scans_and_independent_totals_match_wide_arithmetic、fn gpu_readback_rejects_unwritten_words_and_preserves_both_halves、fn gpu_reconciliation_matches_the_two_round_history_rule（错误成员/枚举成员的新增与改名见第 1 样逐块与概述）
- 不兼容旧代码与旧格式：加行没有命中（关键词 旧版|旧布局|旧格式|legacy|migrat|迁移|兼容旧|backward…）。
- 不新建判定库：加行没有命中（关键词 rocksdb|RocksDB|sqlite|prune.server|prun…）。
- 多机：本机独有状态：加行没有命中（关键词 hostname|/proc/|/sys/|env::var|std::env|…）。
- 多线程：加行没有命中（关键词 available_parallelism|--test-threads|ray…）。
- 带 --item 的命令与项 / 新崩溃流多罩了什么：加行（非注释）里提到 crash/崩溃的 0 处；本文件没有新加崩溃点重放流或崩溃用例。 重放前缀的新证明与必红表行走 `scripts/gate/verus-proof-check.py must-red-worlds` 的 `--item must-red:<行名>`（README.md 原文），登记在 verus-pins.tsv。

**4 测它的测试与变异**
- 改到/新加的函数名 4 个（非测试函数）：同文件测试模块引到的 0 个，tests/ 目录文件引到的 4 个，两处都没引到（grep 名字）的 0 个。逐名明细在草稿目录 coverage.txt。
- crates/mutations.tsv：指到本文件的行，新增 0 行（键新出现）、消失 0 行、行变 0 行（多为必红测试名随改名变）。

**5 线索，没核**
- 没有。

### replay-prefix/tests/gpu_gc_capacity_matches_independent_reference.rs（+214 / -0，1 块）

**1 这一次改了什么**
新增整份（214 行，5 个 #[test]）。首行 use singlefs_replay_prefix::gpu_gc_capacity_proofs::*;，无文件头注释。概括：对 gpu_chunk_fits、largest_gpu_piece_input、显存账本、折半上限、映射表字节数等用独立全宽算术与穷举对拍。

逐块：
- gpu_gc_capacity_matches_independent_reference.rs:1 +214/-0 〔〕 加：use singlefs_replay_prefix::gpu_gc_capacity_proofs::*;

**2 改动处注释里引到的编号**（行号：原话；regex 取 I-x.y、D/E/C 数字、「已定项」、「欠账」）
- 加行的注释里没有 D、E、C、I 起头的编号或「已定项」「欠账」。

**3 检查单八格的原始事实**（只查加行；「生产段」指文件里 `mod tests` 之前，写在 `mod tests` 之外的 `#[cfg(test)]` 辅助也算在里面，要看行上方的 cfg）
- 走不到的理由：生产段没有新加 unreachable!/panic!/expect/unwrap/assert!；测试段 assert/expect/panic/unwrap 共 22 处。
- `_ =>`：生产段没有新加，测试段 0 处。
- `as` 转换：生产段没有新加，测试段 0 处。
- 类型与名字：生产段新加的条目名 7 个：const EDGES、fn reference_fits、fn chunk_admission_and_extension_match_full_width_arithmetic、fn largest_piece_matches_exhaustive_search_and_is_maximal、fn holds_releases_and_waits_preserve_the_exact_ledger、fn memory_caps_never_expand_and_allocation_failure_makes_progress、fn mapping_buffers_and_dispatch_cover_their_exact_full_width_sizes（错误成员/枚举成员的新增与改名见第 1 样逐块与概述）
- 不兼容旧代码与旧格式：加行没有命中（关键词 旧版|旧布局|旧格式|legacy|migrat|迁移|兼容旧|backward…）。
- 不新建判定库：加行没有命中（关键词 rocksdb|RocksDB|sqlite|prune.server|prun…）。
- 多机：本机独有状态：加行没有命中（关键词 hostname|/proc/|/sys/|env::var|std::env|…）。
- 多线程：加行没有命中（关键词 available_parallelism|--test-threads|ray…）。
- 带 --item 的命令与项 / 新崩溃流多罩了什么：加行（非注释）里提到 crash/崩溃的 0 处；本文件没有新加崩溃点重放流或崩溃用例。 重放前缀的新证明与必红表行走 `scripts/gate/verus-proof-check.py must-red-worlds` 的 `--item must-red:<行名>`（README.md 原文），登记在 verus-pins.tsv。

**4 测它的测试与变异**
- 改到/新加的函数名 6 个（非测试函数）：同文件测试模块引到的 0 个，tests/ 目录文件引到的 6 个，两处都没引到（grep 名字）的 0 个。逐名明细在草稿目录 coverage.txt。
- crates/mutations.tsv：指到本文件的行，新增 0 行（键新出现）、消失 0 行、行变 0 行（多为必红测试名随改名变）。

**5 线索，没核**
- 没有。

### replay-prefix/verus-must-red-worlds.tsv（+76 / -8，4 块）

**1 这一次改了什么**
四块（净增 76 行删 8 行）：L26 三行 ensures 冒烟锚点随 lib.rs 改名换行；L198 一行证明负例名去 a_；L305 起部分写四行锚点随形参改名；L327 起新增 68 行：固定长度部分写五项契约与三个关键前提的反例（overwrite-*、partial-*），GPU GC 的 gpu-gc-G01…、gpu-gc-audit-*、gpu-gc-witness_* 与 gpu-score-* 反例（按行名前缀计：gpu-gc 35、gpu-score 15、overwrite-contract 5）。

逐块：
- verus-must-red-worlds.tsv:23 +3/-3 〔ensures·counter_of_the_last_record_the_root_covers〕 加：ensures·冒烟：两条记录的整次发布改成断言不是拼接	crates/singlefs-replay-prefix/src/lib.rs	    ensure ｜ 删：ensures·冒烟：两条记录的整次发布改成断言不是拼接	crates/singlefs-replay-prefix/s
- verus-must-red-worlds.tsv:195 +1/-1 〔ensures·P3 「搬」改成至少两项	crates/singlefs-replay-prefix〕 加：证明负例·a_segment_holding_a_pinned_slot_is_not_a_sound_choice 的函数体不能推出 false	crates ｜ 删：证明负例·a_segment_holding_a_pinned_slot_is_not_a_sound_choice 的
- verus-must-red-worlds.tsv:302 +4/-4 〔sequential-write-empty-keeps-one-unit	crates/singl〕 加：partial-touching-endpoints-are-not-written	crates/singlefs-replay-prefix/src/par ｜ 删：partial-touching-endpoints-are-not-written	crates/singlefs-r
- verus-must-red-worlds.tsv:323 +68/-0 〔indexed-intersection-preserves-write-order	crates/〕 加：# 固定长度部分写字节模型：五项契约与三个关键前提。

**2 改动处注释里引到的编号**（行号：原话；regex 取 I-x.y、D/E/C 数字、「已定项」、「欠账」）
- 加行的注释里没有 D、E、C、I 起头的编号或「已定项」「欠账」。
- 另有该包自己的性质编号 G01…G30（共 30 个不同的 G 编号），不是决策分项编号。

**3 检查单八格的原始事实**（只查加行；「生产段」指文件里 `mod tests` 之前，写在 `mod tests` 之外的 `#[cfg(test)]` 辅助也算在里面，要看行上方的 cfg）
- 非 Rust 源：八格里「走不到、类型与名字、多线程」不适用；旧兼容、判定库、本机状态：加行里没有认旧版本的分支或新判定存放处（文本文件）。
- 带 --item 的命令与项 / 新崩溃流多罩了什么：加行（非注释）里提到 crash/崩溃的 0 处；本文件没有新加崩溃点重放流或崩溃用例。 重放前缀的新证明与必红表行走 `scripts/gate/verus-proof-check.py must-red-worlds` 的 `--item must-red:<行名>`（README.md 原文），登记在 verus-pins.tsv。

**4 测它的测试与变异**
- 本文件没有改到非测试函数，或不是 Rust 源（coverage.txt 无条目）。
- crates/mutations.tsv：指到本文件的行，新增 0 行（键新出现）、消失 0 行、行变 0 行（多为必红测试名随改名变）。

**5 线索，没核**
- 没有。

