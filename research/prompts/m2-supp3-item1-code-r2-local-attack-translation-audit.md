# 翻译核对表：m2-supp3-item1-code-r2 本地攻方提示

逐句核对 `research/prompts/m2-supp3-item1-code-r2-local-attack.md`（下称「提示」）里每一句转述英文与原文（`crates/singlefs-harness/src/crash.rs`、`.claude/gate.d/54-layer0-replay.sh`）是否一致；行号都是现查所得。「首稿缺的」是本轮写提示时第一遍翻译就漏掉、经核对补上的；写「无」表示核过之后首稿已经忠实，未改。

| 英文项（提示文件行号） | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| 提示:25「This function decides how many states go into each slice, then cuts [0, state_count) into consecutive slices of that size.」 | `crates/singlefs-harness/src/crash.rs:1055`「把 [0, `state_count`) 按序号切成首尾相接的区间。」 | 无——"consecutive" 加上提示:42 显式补的「no gap and no overlap」覆盖了「首尾相接」 | 同首稿，未改 |
| 提示:31-32（scaled 模式两行公式） | `crates/singlefs-harness/src/crash.rs:1061-1066`（代码本身）；对照简化文档注释 `crash.rs:923`「片数取 max(64, 16 × 工作线程数)，每片至少 16 个状态。」 | 无——译的是 1061-1066 行代码本身，不是只按 923 行的简化注释；注释省掉的 ceil_div(state_count, slice_count) 项在提示里保留了 | 同首稿，未改 |
| 提示:38「end_ordinal = min(state_count, first_ordinal + states_per_slice) (saturating add, then clamp with min)」 | `crash.rs:1072-1074`（`.saturating_add(states_per_slice).min(state_count)`） | 无 | 同首稿，未改 |
| 提示:47（合并规则，计数字段逐项相加） | `crash.rs:589-597`；注释 `crash.rs:566`「计数逐项相加」 | 无 | 同首稿，未改 |
| 提示:49（合并规则，first_violation 字段） | `crash.rs:598-600`（`if self.first_violation.is_none() { self.first_violation = first_violation; }`）；注释 `crash.rs:566`「『第一处』只在前面各片都没有时取这一片的」 | 无 | 同首稿，未改 |
| 提示:51（合并规则，checker_first_violation 按 invariant 分别判定） | `crash.rs:613-616`（`for (invariant, detail) in checker_first_violation { self.checker_first_violation.entry(invariant).or_insert(detail); }`） | 无——`or_insert` 按每个 invariant 独立判定，首稿已写「decided independently for each invariant name」 | 同首稿，未改 |
| 提示:45「Slices are always merged strictly in increasing slice_index order」 | `crash.rs:566`「各片按状态序号从小到大并」 | 原文字面是「按状态序号」（state ordinal），首稿直接译成「slice_index」而非「state ordinal」 | 保留 slice_index 说法，但在提示:42（state_slices 一节）显式建立「slice_index 递增 ⇔ 状态序号递增」的等价关系，把两种说法接起来，不是凭空替换 |
| 提示 worker_threads_of_full_run 步骤 a（正则里两处数量词） | `.claude/gate.d/54-layer0-replay.sh:42`（`sed -n 's/^[A-Z0-9]* states=\([0-9]*\) .*/\1/p'`，两个 `*` 都是零次或更多） | 首稿把两处都误写成「one or more」（一次或更多），实际是「zero or more」 | 改成「zero or more uppercase letters or digits」「zero or more digits」，见提示:101（替换稿） |
| 提示 worker_threads_of_full_run 步骤 d（第二个正则的数量词） | `.claude/gate.d/54-layer0-replay.sh:43`（`sed -n 's/.* worker_threads=\([0-9]*\) .*/\1/p'`，同样是 `*`） | 首稿没提「zero or more」这个限定，写法含糊 | 补上「zero or more digits are captured, same as in step a」，见提示:104（替换稿） |
| 提示步骤 5–9（`count_line` 判空、`exhaustive` 判断、`worker_threads_are_acceptable` 调用的先后次序） | `.claude/gate.d/54-layer0-replay.sh:76`（判空）、`:81`（判 exhaustive）、`:86`（调 `worker_threads_are_acceptable`）——脚本里三处次序是 76 → 81 → 86 | 首稿把「调用 worker_threads_are_acceptable」写在判空、判 exhaustive 两步之前，与脚本实际执行次序颠倒（`worker_threads` 虽在第 74 行就算出来，但真正用它判红是第 86 行，排在 76、81 两行之后） | 改写：先算 `actual_worker_threads`（步骤 5，对应第 74 行）与删日志（步骤 6，对应第 75 行）；再判 `count_line` 是否为空（步骤 7，对应第 76 行）；再判是否 `exhaustive=true`（步骤 8，对应第 81 行）；最后才调用 `worker_threads_are_acceptable`（步骤 9，对应第 86 行）。核过表 3 六行：每行都只设计成恰好一个检查失败，原次序颠倒不会改变这 6 行任何一行的判定，但次序本身错了，照样改（见替换稿提示:100-112） |
| 提示:99（`checker_line` 抽取，「每一处命中」而非「只取第一处命中」） | `.claude/gate.d/54-layer0-replay.sh:73`（`grep -A1 '^LAYER0 ' "$log" \| grep '^CHECKER ' \| head -1`） | 无——这处首稿写提示时就自觉写全（`grep -A1` 对每一处命中都取一行，不是只对第一处命中），不是先写窄后来才补 | 同首稿，未改 |
| 提示:91（`THREADS_VAR` 判「非空」而非只判「有没有 set」） | `.claude/gate.d/54-layer0-replay.sh:22`（`if [[ -n "${SINGLEFS_LAYER0_THREADS:-}" ]]; then`） | 无——`-n` 测的是非空，首稿已写「非空」，没有把「设成空字符串」误算进 explicit 分支 | 同首稿，未改 |
| 提示:110（`worker_threads_are_acceptable` 分支 b 的完整布尔条件） | `.claude/gate.d/54-layer0-replay.sh:54`（`if [[ "$2" == 1 && "$machine_cores" -gt 1 && ! ( "$threads_origin" == "显式设的" && "$SINGLEFS_LAYER0_THREADS" == 1 ) ]]; then`） | 无 | 同首稿，未改 |
| 三个常量的数值 | `crash.rs:890`（`LAYER0_MINIMUM_SLICE_COUNT: u64 = 64`）、`:892`（`LAYER0_SLICES_PER_WORKER_THREAD: u64 = 16`）、`:894`（`LAYER0_MINIMUM_STATES_PER_SLICE: u64 = 16`） | 无 | 同首稿，未改 |
| （未译入提示）编译期穷尽性理由 | `crash.rs:567`「按字段拆开写全：新加一个字段而这里没并，编译不过。」 | 不适用——这句是实现动机（防止漏合并新字段），不是可计算的规则，本轮判题（Q1–Q18）用不到它 | 有意不译入提示；本表登记「不抄，因为它是实现动机而非可计算规则，不影响 Q1–Q18 任何一题的答案」 |
| （未译入提示）`LAYER0_PROGRESS` 边跑边转发到本阶段输出 | `.claude/gate.d/54-layer0-replay.sh:30, 32-36`（`run_layer0_test_binary` 函数体） | 不适用——这是给人看日志用的旁路，不影响任何判红判绿分支 | 有意不译入提示；本表登记「不抄，因为它只影响运维时看到什么，不影响 Q13–Q18 的判定逻辑」 |

## 多出来的限定词（英文比原文多，且不是从别处接回来的）

未发现。提示:45 的「slice_index」说法虽然字面上换了原文「状态序号」的词，但不算「凭空多出的限定词」——它在提示:42 已经与「状态序号递增」显式建立等价关系（见上表），不是无来源地加宽或加窄。
