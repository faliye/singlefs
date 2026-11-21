# replay-prefix-backfill-code-r2：辩方腿报告（2026-11-19）

立场：复核第一轮判决 `research/prompts/replay-prefix-backfill-code-r1-main-verification.md` 的第二节与第三节第 3 到 6 条改法合进来之后的样子，替被判出局的一方（第一轮「没打中」的几格与实现员的做法）找站得住的理由，同时指出判错、写宽与没改全的地方。没写模型；读码与计数用了两个只读小脚本（草稿目录 `~/.local/share/singlefs/work/replay-prefix-backfill-code-r2-defense/tools/`，sha256：`cover.py` 89c75dfb…afb3、`clauses.py` efddc729…1a18，量的），它们只读树、不改任何文件，输出在同目录 `cover.txt`、`clauses.txt`，只当线索。

读的树：`sha256sum -c research/prompts/replay-prefix-backfill-code-r2-snapshot/crates-sha256.txt` 十份全 OK（量的）。没读禁读清单里的文件。

## 一、结论一览

| 要回答的 | 结论 |
|---|---|
| 一、第二节每条打中够不够得着 | 十四条里：十条站得住；窗口常数那条写宽了（证明本来只需要退 65534 个，「不被钉住」不是缺陷）；G04 一族那条的口径同样打中 G11；G30 那条的清扫没做完（还有两处写着「产品直接调用」）；检查单「走不到的理由」那条打在这一轮没判的文件上，够不着这一轮 |
| 二、第三节第 3 到 6 条合进来之后 | 第 3、4、5 条做到了，没改坏判决没要求的东西；第 6 条做了大半，但说明文件的复跑命令仍不是服务收的写法（五条 `cargo test` 只点目标、没有 `--exact` 与测试全名），G30 与「十个产品核」还有两处没改，说明文件里「卡由服务分」与规则相反 |
| 三、检查单八格 | 成立 7 格（其中类型与名字带一处保留，见第六节），不成立 1 格（带 --item 的命令与项），推不出 0 格 |
| 交集长度那一路 | 定义与两条分割引理对（读码核了数学）；新 ensures 与产品消费处对得上；必红表新加 8 行里有两处新 ensures 没有行罩着（`None` 分支的长度为零）；8 行新必红与 3 行改过第五段的行都在任务 20692 的 93 项里、都判绿 |

「推的」一律是读码推的、没起 Verus；判决要引的数由主 agent 复跑（命令见「要主 agent 跑的命令」）。

## 二、第二节逐条复核（判决条目 / 辩方判定 / 证据）

判定三种：站得住、够不着、同样打中别的臂。「别的臂」在代码轮指同一形状在别的函数、别的文件上是不是同样成立。

| 判决条目 | 辩方判定 | 证据 |
|---|---|---|
| 一·1 空真、四处 `as` 没打中 | 站得住（本轮不重复攻） | 派发提示明令不重复 |
| 一·2 G30 没有产品调用点 | 站得住；但同一说法的清扫没做完（同样打中别处） | `grep -rn extended_gpu_chunk_input crates --include=*.rs` 只命中定义与一条测试：`crates/singlefs-replay-prefix/src/gpu_gc_capacity_proofs.rs:183`「pub fn extended_gpu_chunk_input(」、`crates/singlefs-replay-prefix/tests/gpu_gc_capacity_matches_independent_reference.rs:85`「extended_gpu_chunk_input(input, additional, entries, &capacity),」。仍写着产品调用的两处：`.claude/gate.d/verus-pins.tsv:47`「# GPU GC G21–G30：十个产品核与一个边界见证」、`crates/singlefs-replay-prefix/verus-must-red-worlds.tsv:365`「# G21–G30：产品直接调用的容量、记账、派发与映射表核。」。九个有调用点的核是 `gpu_chunk_fits`、`largest_gpu_piece_input`、`tightened_gpu_memory_limit`、`checked_gpu_memory_hold`、`checked_gpu_memory_release`、`gpu_bytes_after_submission_wait`、`gpu_memory_cap_after_allocation_failure`、`gpu_workgroups_for_entries`、`checked_gpu_mapping_table_bytes`（`crates/singlefs-background-gpu/src/` 下逐个 grep 有调用行：`memory_limit_and_chunks.rs:346`、`:400`、`:516`、`:693`、`:709`、`:723`，`region_record_kernels.rs:552`，`background_gpu.rs:2521`、`:3801`），第十个是 G30，所以「十个产品核」不对 |
| 一·3 `GPU_GC_PROOFS.md` 说「证明中间值落在 u32 范围内」，删界照样验过 | 站得住 | 原函数体没有一条义务引用那条界；今天 `crates/singlefs-replay-prefix/src/gpu_gc_scoring_proofs.rs:47`「assert(middle + 1 <= u32::MAX);」引它。删 requires 的攻击行当时「照样验过」的缘由：同一个谓词同时是递归调用的前提，删了两头一起松（推的），所以只有函数体里的断言能让删界变红 |
| 一·4 窗口常数 65535 不被证明钉住 | 写宽了（事实对，缺陷不成立） | 见下面「窗口常数」一段 |
| 一·5 G04 一族是定义展开、对产品没有约束 | 站得住（G02、G03、G04）；同样打中 G11 | `crates/singlefs-replay-prefix/src/gpu_gc_proofs.rs:158`「if record.released || record.device != device { 0 }」是 `record_contribution` 的定义，`:197`「!record.released && record.device == device ==>」那一条 ensures 是它另一支的展开；审计行 `gpu-gc-audit-score-capped-at-two` 改的就是这份定义、G11 那条抓它（`crates/singlefs-replay-prefix/src/GPU_GC_PROOFS.md:28`「“定义自检”表示 `ensures` 是同一文件里 spec 函数定义的展开，只钉住那份定义，对产品代码没有约束。」）。同一口径下 G11 也该标「定义自检」，表里 `GPU_GC_PROOFS.md:42` 的 G11 行只写「模型」 |
| 一·6 G21、G22 的显存是交进来的上限，暂存折算在核外 | 站得住 | `crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:375`「fn chunk_fits_with_the_staging_of_its_input(」先从上限里扣暂存再交给 `gpu_chunk_fits`（`:400`）；`:497`「fn largest_piece_input_bytes(」按份数分开再交 G22（`:516`）。G23 到 G29 的参数都是产品原样交的量，不受这条牵连 |
| 一·7 G17 注释「正的新增分配必在总限内」没有对应 ensures | 站得住 | 补丁前 ensures 只有三条（等式、`remaining <= limit`、`held <= limit ==> held + remaining == limit`），没有量词一条 |
| 二 覆盖写第一条引理的 `requires` 不承重 | 站得住，且要补一句限定 | 原三条 ensures（长度、逐字节、空请求）不涉及 `subrange`，离了 `overwrite_fits` 仍成立；承重只对按段说的三条（`crates/singlefs-replay-prefix/src/partial_write_proofs.rs:119` 的注释写了）。判决说「不承重」是对补丁前的三条而言，准确 |
| 三·1 两份测试缺 `u64::MAX − 2`、`u64::MAX − 1` | 站得住，并有独立证据 | 变异行「GPU 输入对齐只在 u64 最大值上拒绝」只在这两个值上与原实现不同（原文 `words.checked_mul(GPU_INPUT_ALIGNMENT)`，替换文只对 `input == u64::MAX` 返回 `None`、其余回绕乘），所以没有这两个边界值这一行不会红（推的）；两份 `EDGES` 现有 `u64::MAX - 2`（`tests/gpu_gc_arithmetic_matches_wide_integer_reference.rs:12`、`tests/gpu_gc_capacity_matches_independent_reference.rs:13`） |
| 三·2 G27、G29 参照与被测同式同字面值 | 站得住 | 补丁前 G27 参照是 `u128::from(cap) / 2`、G29 是 `56 * …`、`16 * …`、`4 * …`（补丁里的删除行），现已换；见第三节 |
| 三·3 两份测试没有变异行 | 站得住 | 补丁前 `crates/mutations.tsv` 对这两份测试 0 行；今天末尾 9 行，与快照 `mutations-append.tsv` 逐字相同（`diff` 无输出，量的） |
| 四 必红表新反例 58 行、正文写 68 行 | 站得住 | 数的是注释与空行；本轮正文又写「非空行 480」同样把 35 行注释算进去：数据行 445（量的：去掉注释行与空行后数），比提交 HEAD 多 84 行、删 1 行（改名）、改 3 行第五段 |
| 六 说明文件的数对不上 | 站得住，今天对上了 | 十份源文件（数 src 下的 .rs 得 10）、九个 pub mod（数 lib.rs 里的 pub mod 行得 9）、测试数 7+36+23+16=82、4、5、2、5、6、6 逐个现数，都与 `GPU_GC_PROOFS.md` 现文对得上 |
| 七 检查单「走不到的理由」打在 `live_slot_kernels.rs:379` | 够不着这一轮 | 那一份不在这一轮被判的文件里，判决自己写了并进产品 GPU 包那一题 |

### 窗口常数：为什么一·4 写宽了，实现员没加「窄一格」那一行是对的

- 模型里的跨度界：`crates/singlefs-replay-prefix/src/gpu_gc_scoring_proofs.rs:103`「&& records[position].end - records[position].start <= 65535」；窗口下沿 `:97`「if start >= 65535 { start - 65535 } else { 0 }」。产品着色器里的同一个常数是 `crates/singlefs-background-gpu/src/live_slot_kernel_searching_sorted_records.wgsl:43`「const LONGEST_SPAN_OF_AN_ALLOCATION_RECORD: u32 = 65535u;」，两处相同。
- 推导（读码推的）：窗口下沿退 W 个槽，被排除的同盘记录满足 `record.start <= start - W - 1`，跨度至多 65535，所以 `record.end <= start + 65534 - W`；它贡献为零当且仅当 `record.end <= start`，对所有这样的记录成立就是 `W >= 65534`。W 取 65534 照样验过，W 取 65533 时 `record.start = start - 65534`、`record.end = start + 1` 的记录贡献 1 被漏，证明断。所以「证明钉住的界是 W ≥ 65534」，65535 是比证明要的宽一格的余量，宽一格只多收窗外贡献为零的记录，永远是对的。
- 必红行 `verus-must-red-worlds.tsv:384` 的 `gpu-score-window-too-narrow` 把 `start - 65535` 改成 `start - 65533`，落在 `lemma_candidate_window_keeps_every_contribution`；W 取 65534 本来就验得过，不可能有「窄一格」的必红行，实现员没加是对的。判决 `research/prompts/replay-prefix-backfill-code-r1-main-verification.md:46`的原话带着条件：「验得过就把说明文件里那一句改成证明实际钉住的界」，补丁做的正是这一支：`crates/singlefs-replay-prefix/src/GPU_GC_PROOFS.md:14`「窗口下沿往前退 65535 个槽；证明钉住的是「至少退 65534 个」」与 `gpu_gc_scoring_proofs.rs:94`「/// 窗口下沿往前退 65535 个槽，比证明要的多一格」。
- 为什么一·4 写宽了：`git show HEAD:crates/singlefs-replay-prefix/src/GPU_GC_PROOFS.md` 里没有一句声明「证明钉住 65535」（该文件里与 65535 有关的是「已访问条数 × 65535」与前提「跨度不超过 65535」，量的，grep 命中）。「不被证明钉住」对任何合法证明都成立，不是缺陷，打中不了说明文件的哪句话。判决没有因此多要求什么（第三节第 3 条是有条件的），所以没有改坏东西，只是一·4 的措辞应收成「说明文件没有交代证明实际钉住的界」。
- 同样打中别的臂：没有，WGSL 的常数与模型一致。没有任何证明把 WGSL 的 65535 与 `valid_scoring_records` 的 65535 连起来（靠说明文件），那是模型到着色器的精化，说明文件第一轮前就写了「仍未证明」。
- 什么会推翻：主 agent 在 W 取 65534 的副本上跑 `lemma_candidate_window_keeps_every_contribution` 报错（任务 18248 的读数说它验得过，我没有复跑）。

## 三、第三节第 3 到 6 条合进来之后的样子

### 第 3 条（u32 界、窗口、覆盖写、G17）

| 判决要求 | 补丁做到了没有 | 证据与限定 |
|---|---|---|
| 补「中点加一不超过 u32」的断言并配一行删界的必红行 | 做到 | `gpu_gc_scoring_proofs.rs:47`「assert(middle + 1 <= u32::MAX);」，行 `verus-must-red-worlds.tsv:412` 的 `gpu-score-binary-midpoint-needs-u32-bound` 删 `:35` 的「0 <= below <= above <= records.len() <= u32::MAX,」。删之后递归调用的前提同时松掉（同一个谓词），所以只剩函数体里这一条断言推不出，落点只在 `lower_bound_of_sorted_records`，与第六段相同（推的）。注释 `:46` 写「requires 里记录数不超过 u32::MAX 这条界让它不回绕」是现状句。`theorem_sorted_search_equals_full_record_walk` 自己的 requires 保着 `sorted.len() <= u32::MAX`，落点不会溢到它 |
| 窗口常数配「窄一格」行，验得过就改说明文件 | 做到（走验得过那一支） | 见上一节 |
| `overwritten_bytes` 的定义配改坏定义的必红行；第一条引理的 `requires` 删掉或改成承重 | 做到（走改成承重那一支） | 行 `:418` 的 `overwrite-definition-start-shifted-by-one`、其后的 `overwrite-definition-length-short-by-one`，加 `:417` 的 `overwrite-exact-needs-fit`（删 `crates/singlefs-replay-prefix/src/partial_write_proofs.rs:121`「requires overwrite_fits(old, start, data),」）。离了 `overwrite_fits` 三条按段结论推不出：`Seq::subrange` 的长度与下标公理只在 `0 <= j <= k <= len` 时给，请求越出文件时 `:128`「overwritten_bytes(old, start, data).subrange(0, start) == old.subrange(0, start),」这类等式没有依据（推的；任务 20692 对 `overwrite-exact-needs-fit` 的读数见「任务 20692 回读」一节）。限定：新三条 ensures 没有消费者（本文件里别的引理都不引它们），承重是靠新加的结论得来的；这合判决写的「改成承重」，不算改坏，但原来那条 requires 自己仍不承重。说明文件没有掩饰：`crates/singlefs-replay-prefix/src/README.md:156`「逐字节那一条是 `overwritten_bytes` 定义的展开，它钉住的是定义本身。」 |
| G17 的注释与 ensures 对齐 | 做到 | `crates/singlefs-replay-prefix/src/gpu_gc_proofs.rs:285`「pub open spec fn allocation_stays_within_the_limit(held: u64, request: int, limit: u64) -> bool {」做触发项，`:296`「forall|request: int| 0 < request <= remaining ==> #[trigger] allocation_stays_within_the_limit(held, request, limit),」。量词有没有触发上、是不是空真：`held < limit` 时 `remaining = limit - held`，范围非空，`held + request <= limit` 真有内容；`held >= limit` 时 `remaining = 0`，范围空、该支空真，这是饱和情形（已占不少于总限，没有正的新增分配可放），语义上就该空；必红行 `:454` 的 `gpu-gc-G17-positive-allocation-within-the-remaining-fits` 把上界改成 `remaining + 1`，两支都变假（`held < limit` 时 `held + remaining + 1 > limit`，`held >= limit` 时 `request = 1` 就超），所以量词不是整个空真。触发项是对开放 spec fn 的应用，Verus 能展开，没有「触发不上」的形态（推的）。`:289` 的注释与 ensures 一致 |

### 第 4 条（75 行新必红）

- 条数（量的）：`git diff` 相对 HEAD，数据行加 84（含交集长度 8、改名 1），其中这一题 75，与实现员 `commands.txt` 点名 92 项（75 + 17 条已有行）对得上。
- 落点：75 行我逐行看了原文、替换文和第六段是不是改到的那个函数，没找到落点写错函数的；Verus 会不会多报别的函数我没有逐行推。改函数体的行（G01 到 G30 的多数）落点是那个函数，改 ensures 或前提的行落点是引理自己。有一行的落点比别的长：`overwrite-definition-start-shifted-by-one` 落四个函数（`lemma_a_contained_overwrite_folds_into_the_payload`、`lemma_overwrite_changes_exactly_the_requested_bytes`、`lemma_whole_unit_overwrite_does_not_depend_on_old_bytes`、`witness_split_and_contained_overwrites`），是改定义的后果（推的）；「短一格」那一行的落点是另四个（含 `lemma_splitting_an_overwrite_preserves_the_result`、不含 `lemma_a_contained_overwrite_folds_into_the_payload`）。哪个引理对哪种改坏敏感我没有逐个推，落点对不对靠任务 20692 的实测。
- 改坏的是不是名字说的那条 ensures：多数是（名字写 `…-needs-…` 的是删前提，其余是改 ensures 或改函数体）。读码没找到名不副实的。
- 第一轮两条腿列的没有反例罩着的 ensures 还剩哪些：见第五节（两条腿列的都已有行；剩下的是这一路新增与两条腿没列的几条）。

### 第 5 条（两份测试与九行变异）

| 判决要求 | 做到没有 | 证据与限定 |
|---|---|---|
| 两份测试补 `u64::MAX − 2`、`u64::MAX − 1` | 做到 | 两份 `EDGES` 都有（`tests/gpu_gc_arithmetic_matches_wide_integer_reference.rs:12`、`tests/gpu_gc_capacity_matches_independent_reference.rs:13`）。变异行「GPU 输入对齐只在 u64 最大值上拒绝」靠这两个值才红，证明它们真承重 |
| G27 的参照改成与被测不同式 | 做到，带一处共用 | 参照现在按折半的定义不等式判：`tests/gpu_gc_capacity_matches_independent_reference.rs:216`「doubled <= u128::from(cap) && u128::from(cap) < doubled + 2,」，不做除法，与被测函数体 `failed_bytes / 2` 无共用。共用的是 Verus ensures `2 * next <= failed_bytes < 2 * next + 2`（`crates/singlefs-replay-prefix/src/gpu_gc_capacity_proofs.rs:134`）：测试的期望式就是证明的后置条件，这是「被证明的规格对被测的实现」，不是「被测对被测」。这两个不等式唯一确定 `next = floor(cap / 2)`，参照不比原来弱 |
| G29 的参照按产品布局逐字段数 | 做到，带两处限定 | 常数 `tests/gpu_gc_capacity_matches_independent_reference.rs:39`「const KEY_WORDS: u128 = 7;」是手抄产品 `crates/singlefs-background-gpu/src/region_record_kernels.rs:101`「pub const KEY_WORDS: usize = 7;」（重放前缀包不能依赖后台 GPU 包，只能手抄；抄的口径就是文件头布局那几行）。`:43`「const WORDS_PER_SORTED_INDEX_ITEM: u128 = 1;」在布局文字里没有逐字写「一个字」，是从「按 key 排的条目序号」推的，测试头注释写的「逐个字段数出来」对这一项说宽了一点。限定之外：产品 `MAPPING_ENTRY_WORDS = 1 + KEY_WORDS + 6`（`region_record_kernels.rs:103`）与测试的 `1 + 7 + 2 × 3` 是两种写法给同一个 14，后台包的 `gpu_gc_capacity_contracts_through_the_host.rs` 另有一条拿真打包出来的映射表对尺寸，是第三条独立路 |
| 每条测试挂一行变异、红在点名的测试上 | 做到 | 九行的原文各命中一次（我的脚本逐行数，量的），九行点名的测试正好是两份文件里的九条 `#[test]`，一一对应、没有重复。逐行读码推红在哪条断言（推的，任务 20696 已报「抓到 71 条」，读数不重推）：对齐回绕行红在 `gpu_budget_arithmetic_matches_a_u128_reference_at_full_width` 里第一个 `assert_eq!(checked_gpu_input_bytes(input), …)`（`u64::MAX − 2` 的期望是 `None`、变异给 `Some(0)`）；扫描区行红在 `slot == end`、跨度 0 的一组（变异放过后接的 `checked_add` 判，`past <= end` 为真）；高字行红在 `value` 的高字非零的一组；仲裁行红在第二次分歧的 `expected_cpu`；追加输入行红在 `additional = u64::MAX` 而前面 `input + additional` 溢出的一组；最大片行红在 `binding = 5` 这类非 4 的倍数；预留行红在 `limit = Some(u64::MAX)`、`held + bytes == u64::MAX` 与 `Some(0)`、`0 + 0` 两组；折半行红在奇数 `cap` 的 `doubled <= cap`；条目表行红在 `mapping_entry_item_bytes()` 的 56 对 52 |

测试优先多线程：九条都是几毫秒的小用例（任务 20675 的 `finished in 0.00s`、`0.01s` 由主 agent 读到），`cargo test` 默认按测试并行；没有该切片而没切的。

### 第 6 条（说明文件）

| 判决要求 | 做到没有 | 证据与限定 |
|---|---|---|
| README 的份数与表 | 做到 | `crates/singlefs-replay-prefix/src/README.md:7`「## 十份源文件各负责什么」；表 `:11` 到 `:20` 十行（`grep -c '^| \['` 得 10，量的）；`:22`「核的入口文件为 `lib.rs`，另九份文件由 `lib.rs` 的九个 `pub mod` 引入。」 |
| `gpu_gc_proofs.rs:1` 的指向、测试数与第二份测试、G04 与暂存折算、必红表第 198 行 | 做到 | `gpu_gc_proofs.rs:1`、`GPU_GC_PROOFS.md` 逐个对过：82（7、36、23、16）、4、5、2、5、6、6 逐个现数相等；G04 与暂存折算见第二节；改名那行见 `git diff` |
| 复跑命令的写法照现行写法改 | 没做完 | 见下面「复跑命令」 |
| `GPU_GC_SELFTEST.md` 删掉，其中还成立的现状句并进 `GPU_GC_PROOFS.md` | 做到 | 现状句并进了：两条审计行与它们抓的是 G11、G04 成功支（`GPU_GC_PROOFS.md` 必红行一段）、产品构建与 `verus --compile` 的 MIR 逐条目相同、主机侧两组测试的枚举规模（`7^6 = 117649`、32 个批次）、真 GPU 两组测试、「有限枚举不替代任意长度历史的精化证明」。没并进的是经过类：任务号、「修复前」两次 186 条义务、首次尝试被内存上限打断、临时判定库的自证命令、第二台那张卡的地址；这些是一轮的经过，判决明说不是现状说明。`grep -rn GPU_GC_SELFTEST` 在 `crates`、`.claude`、`scripts` 下零命中，没有悬空引用（量的，`records/` 与 `research/` 下的命中是当时记录） |

#### 复跑命令：说明文件里的五条 `cargo test` 不是服务收的写法

- 现状：`crates/singlefs-replay-prefix/src/GPU_GC_PROOFS.md` 的复跑一段，五条 `cargo test` 只点测试目标或模块前缀：`:104`「cargo test -p singlefs-replay-prefix --test gpu_gc_arithmetic_matches_wide_integer_reference」、`:106`「cargo test -p singlefs-replay-prefix --test gpu_gc_capacity_matches_independent_reference」、`:108`「cargo test -p singlefs-background-gpu --lib -- two_kernel_reconciliation::tests memory_limit_and_chunks::tests live_slot_kernels::tests region_record_kernels::tests」、`:110`「cargo test -p singlefs-background-gpu --test gpu_gc_formal_contracts_through_the_host」、`:112`「cargo test -p singlefs-background-gpu-tests --features tests-on-a-real-gpu --test live_slot_kernels_on_a_real_gpu_agree_with_the_cpu_reference」。没有一条带 `-- <测试全名…> --exact`。
- 规则：`.claude/agent-common.md:65`「只点 `--test` 目标、只给筛选词的跑整个目标或一批测试，提交时被服务拒」。判据在 `task-server/task_server_lib.py:5313`「def unnamed_cargo_test_problem(after_test, shown, environment=None):」，放行的只有 `:5328`「if "--exact" in after and names and not broken("unnamed-cargo-exact-uncounted", environment):」那一支（`--` 之后带 `--exact` 与至少一个测试全名）。五条都落在没点名那一支，按字面读提交时被拒（推的，我没提交）。
- 对着判决：第三节第 6 条写的是「复跑命令的写法照改」（`research/prompts/replay-prefix-backfill-code-r1-main-verification.md:49`）。补丁把 `--manual` 改成了带范围声明的 `task-server/task submit --`，没有改到点名测试函数，所以这一条只做了一半。同一份补丁里 `README.md:172` 的 `partial_write_proofs` 那一条带了 `--exact` 与四个全名，是对的；两份说明文件的写法不一致。
- 别的臂：`formal-verification.sh --item must-red:…` 那些命令的项名我逐个核了，各在必红表里恰好一行（量的），没有这个问题。补丁前的写法（`--manual`、不带范围声明）更不合规，所以这不是补丁改坏，是没改完。
- 说明文件还写着「卡、内存与线程由服务分」（`GPU_GC_PROOFS.md:97`「卡、内存与线程由服务分」）；现行规则相反：`.claude/rules/verification.md:50`「统一任务服务只调配 CPU」（同一段说任务不领卡、不分卡、不清场，用卡的测试交 GPU 服务代跑，接口待定）。所以 `:112` 那条真 GPU 命令不能按说明文件原样交，「卡由服务分」这半句也与规则相反。这是补丁新写的句子（补丁前的复跑命令里没有这半句）。
- 推翻条件：主 agent 把 `:104` 那条原样提交，被接收且跑到测试；或查到服务对 `种类=target` 声明另有放行（我在 `unnamed_cargo_test_problem` 里没读到）。

### 补丁里没改全的两处（G30、G11）

- G30：`.claude/gate.d/verus-pins.tsv:47` 与 `verus-must-red-worlds.tsv:365` 仍把 G30 算作产品核，见第二节一·2。这两处是第一轮「写成产品直接调用」那一条的同族，没被回扫到。
- G11：见第二节一·5。是不是并进「定义自检」归主 agent 判，判决没点它，补丁不算漏做。

## 四、交集长度那一路

- 定义：`crates/singlefs-replay-prefix/src/file_range_proofs.rs:157`「pub open spec fn range_intersection_length(」：最大起点到最小终点之间的正差，否则为零。
- 两条分割引理的前提（读码推数学，没起 Verus）：`:345`「pub proof fn lemma_adjacent_unit_intersection_lengths_add_up(」前提 `unit_start <= boundary <= unit_end`：写区间为空或倒置时（`write_start >= write_end`）最大起点不小于 `write_start`、最小终点不大于 `write_end`，两边都是零；否则是线段加法。`:357`「pub proof fn lemma_split_write_intersection_lengths_add_up(」前提 `write_start <= split <= write_end`：单元为空或倒置时两边都是零。两个前提各自恰好是该拆法成立所需，没有多余、也没有缺的。
- 新 ensures 与产品：`:176`「&&& length == range_intersection_length(write_start as int, write_end as int, unit_start as int, unit_end as int)」是 `Some` 支；`None` 支 `:171`「&&& range_intersection_length(write_start as int, write_end as int, unit_start as int, unit_end as int) == 0」。产品只有一处消费 `checked_bytes_in_indexed_unit`：`crates/singlefs-core/src/write_request_split.rs:385`「singlefs_replay_prefix::file_range_proofs::checked_bytes_in_indexed_unit(」、`:393`「|(offset_in_the_payload, offset_in_the_bytes_written, length_in_bytes)| {」、`:397`「length_in_bytes,」，三元组的第三个就是被证明等于交集长度的 `length`，字段含义（落在这个单元净荷里的那一段的长度）与新 ensures 对得上；新条款只加强后置条件、不改函数体，调用方不受影响。`bytes_in_unit` 没有仓外调用（`grep -rn 'bytes_in_unit\b' crates --include=*.rs` 除本文件外零命中，量的）。
- 必红表新加 8 行（`verus-must-red-worlds.tsv:402` 起）：原文各命中一次（脚本数，量的）。三条 `intersection-length-contract-*` 往 ensures 里插 `false`；两条 `…-valid-boundary` 删前提；`write-intersection-exact-length`、`indexed-intersection-exact-length` 把 `Some` 支的长度加一；`intersection-length-witness-sum` 改见证值。没有行罩着的：两条 `None` 支的长度为零（`:171`、`:221`）。这两条是这一路新加的 ensures，现有行里没有一行的原文碰到它们（脚本逐行比，量的）；原有的 `indexed-intersection-does-not-drop-written-bytes` 把函数体改成恒 `None`，会让 `Some` 支的预期落空，但它不能说明 `None` 支的这一条存在。建议各加一行把 `== 0` 改成 `== 1`（推的，应红在 `bytes_in_unit` 与 `checked_bytes_in_indexed_unit`）。
- 改过的行：相对 HEAD 第五段 `proof_errors=1` 变 `proof_errors=2` 的三行（`indexed-intersection-does-not-drop-written-bytes`、`overwrite-split-needs-valid-boundary`、`overwrite-contained-needs-payload-bounds`）加一行改名，共 4 行（量的）；正文写「改 7 行」我对不上，可能算的基准不同。第五段只报不判，改它不影响红绿。
- 这 8 行与上面 3 行改过的行，实现员交的 92 项命令里没有（`commands.txt` 点名的 92 个必红项逐个比，量的），但任务 20692 的日志里都有：8 行新必红与 3 行改过第五段的行逐项 `ITEM_DONE … exit=0`（见「任务 20692 回读」一节，量的）。`.claude/gate.d/verus-pins.tsv:49` 写「整 crate 实测 214 verified」，20692 日志的 verus-proof 一行报「各项 verified 之和 214、0 errors（登记 214）」，对得上。

## 五、还没有行罩着的 ensures（读码加脚本，不是 Verus 实测）

方法：脚本逐行比必红表每行的「原文」落在哪个函数的 ensures 条上（`clauses.txt`），再人工看没碰到的条会不会被别的行的改法让它单独变假。判法限定：exec 函数的 ensures 靠「改函数体的行」罩着，一行改体会同时违反几条，所以不证明每条各有一行单独让它变红；表头（`verus-must-red-worlds.tsv` 注释）也只承诺「每行改坏一处」。

第一轮两条腿列的：覆盖写四条引理与见证、G01 到 G30 的 ensures 条与分支、三条常量、`lemma_every_walk_prefix_fits_the_gpu_counter` 的两半，今天都能找到至少一行让它变假（见第三节第 4 条与上表）；没找到漏的（读码，没有逐条证明）。

还剩的（我读码找到的，行号是现在文件里的）：

| 位置 | 没罩着的是什么 | 为什么说没有行 |
|---|---|---|
| `crates/singlefs-replay-prefix/src/gpu_gc_scoring_proofs.rs:117` | `lemma_candidate_window_keeps_every_contribution` 的 `first <= past` | 没有行的原文落在这一条上；改 `earliest_candidate_start`（窗口行）与删有效跨度前提（`gpu-score-candidate-needs-valid-spans`）都不让它变假：`past < first` 的矛盾只靠 `earliest_candidate_start(start) <= end` |
| `gpu_gc_scoring_proofs.rs:214`、`:215`、`:216` | `theorem_sorted_search_equals_full_record_walk` 前三条（两端的 `is_lower_bound`、`window.0 <= window.1`） | 第四条靠 `gpu-score-theorem-needs-permutation` 罩着；前三条是定理把两次二分的结论原样交出来，改二分的行断在 `lower_bound_of_sorted_records` 自己，不断定理 |
| `crates/singlefs-replay-prefix/src/gpu_gc_proofs.rs:345`、`:346` | `witness_gpu_gc_protocol_and_score_premises` 的两个具体值（`== 2`、`== 1`） | 见证只有「注入 `false`」的行（`gpu-gc-witness_*`），没有把具体值改错的行 |
| `crates/singlefs-replay-prefix/src/file_range_proofs.rs:171`、`:221` | 交集长度那一路新加的 `None` 支「长度为零」 | 见第四节 |

这四处都是小条：前三处是「证明把已有结论转手交出」或「见证的具体值」，交集长度两条是新写的。是否值得为它们各加一行归主 agent 判；加了的话每行改法我给了建议（交集那两条）。

## 六、检查单八格（逐格回成立、不成立、推不出）

| 格 | 结论 | 证据 |
|---|---|---|
| 走不到的理由二次查实 | 成立 | 在 `changes.patch` 与 `changes-intersection-length.patch` 的加行里搜 `todo!`、`unreachable!`、`panic!`、`unimplemented!`、`.expect(`、`.unwrap(`、`assert_eq!`、`assert!`、`_ =>`：只有测试文件里的 `assert!`（`crates/singlefs-replay-prefix/tests/gpu_gc_capacity_matches_independent_reference.rs:215` 等，是测试断言，不是产品里的走不到）；`_ =>` 0 命中。产品代码段的新加全是 Verus `assert(...)`：`crates/singlefs-replay-prefix/src/gpu_gc_scoring_proofs.rs:47`、`crates/singlefs-replay-prefix/src/partial_write_proofs.rs:134`、`:135`、`:136`、`:139`，都在 `proof fn` 里（`lower_bound_of_sorted_records`、`lemma_overwrite_changes_exactly_the_requested_bytes`），普通构建擦掉；交集那份补丁 0 处。与实现员报告列的五处逐处相同 |
| 类型与名字 | 成立（带一处严读会打中的说明） | 新名字都说得出是什么：`allocation_stays_within_the_limit`（已占加请求不超总限）、`range_intersection_length`、`lemma_adjacent_unit_intersection_lengths_add_up`、`lemma_split_write_intersection_lengths_add_up`、`witness_intersection_length_partitions`；测试里的常量与辅助函数 `WORD_BYTES`、`KEY_LENGTH_WORDS`、`KEY_WORDS`、`LOCATIONS_PER_MAPPING_ENTRY`、`WORDS_PER_LOCATION_OF_AN_ENTRY`、`WORDS_PER_LOCATION_ITEM`、`WORDS_PER_SORTED_INDEX_ITEM`、`mapping_entry_item_bytes`、`location_item_bytes`、`sorted_index_item_bytes` 都不缩写、名字带单位。没有新加 `expect`。严读会打中的一处：测试常量里字数与字节数共用 `u128`（靠名字带单位区分），spec fn `allocation_stays_within_the_limit(held: u64, request: int, limit: u64)` 三个量都是字节、同一语义，不算。exec 核参数的裸整数按正文并进产品 GPU 包那一题，这一轮不判 |
| 不兼容旧代码与旧格式 | 成立 | 两份补丁与变异行加行里搜 legacy、compat、deprecated、migrat、backward、旧版、兼容、迁移、老格式、version：0 命中（量的）；没有版本号常量、旧布局号、迁移路径 |
| 不新建判定库 | 成立 | 加行里搜 sqlite、rocksdb、`create_dir`、`File::create`、`fs::write`、`OpenOptions`：0 命中（量的）。必红表与 `crates/mutations.tsv` 是门禁读的登记表，不是判定存放处 |
| 所有任务多机可用 | 成立 | 两份测试只用编进去的常量与 u128 现造输入；加行里搜 `env::var`、`hostname`、`/tmp`、`/home`、HOME：0 命中（量的）。必红行读的是仓里相对路径。说明文件里的一句「卡由服务分」与规则相反，是文档句、不是测试读本机状态，记在第三节「复跑命令」 |
| 测试优先多线程 | 成立 | 九条测试各几毫秒、没有崩溃枚举，该并行的是测试之间，`cargo test` 默认并行；没有长循环该切片而没切（最大一条 43,050 组 × 至多 40 次内层比较，毫秒级，主 agent 读任务 20675） |
| 带 --item 的命令与项 | 不成立 | 一、`GPU_GC_PROOFS.md:104`、`:106`、`:108`、`:110`、`:112` 五条 `cargo test` 不带 `--exact` 与测试全名，按服务判据提交时被拒（第三节「复跑命令」）；二、说明文件写的复跑命令与规则口径不一致。必红项与变异项本身列得出：每行名字唯一（445 个数据行名字无重复，量的），任务 20696 用 `--item` 点名这九行跑出「抓到 71 条」；交集那 8 行读结构应列得出（`formal-verification.sh` 头注释：必红世界每行一项 `must-red:<变异名>`），没实际列过，推的 |
| 新崩溃流多罩了什么 | 成立 | 两份补丁与追加变异里搜 `crash-case:`、`enumerate_crash_states`、`crash_states`：0 命中（量的），没有崩溃点重放流、崩溃枚举用例 |

## 任务 20692 回读（主 agent 给了号，我只读了 `task-server/task log 20692 --all`，落盘在草稿目录 `log20692.txt`）

量的（日志原样）：

- 末尾汇总：「✓ formal-verification.sh：3 格里 3 格绿（verus-proof verus-vacuity verus-must-red-worlds），0 格本次未跑（无）」；`ITEM_DONE item=must-red:` 共 93 行，93 行都 `exit=0`；我拿相对 HEAD 加的 84 行与改的 3 行（共 87 行）逐个比，除改了名的那一行（日志里同名，名字里有空格与中点，我的正则切开了，另在日志 2594 行找到）外 86 行都在这 93 项里；所以交集长度的 8 行与改过第五段的 3 行都点了、都绿。
- 但 93 项里只有 1 项这一趟现判（复用哨兵抽中强制现判），其余 92 项是「绿，复用（2026-11-18 判绿：落点 …；证明错 N 条）」，即读的是判定库里键相同的旧判绿。键含函数单元指纹、原文、替换文、第六段落点与 Verus 版本，复用本身合那一道门禁的设计；我看不到那 92 条当初是谁在哪棵树上判的。主 agent 要把「这 92 行每行红在第六段」当这一趟的新实测引，先核复用的键就是这棵树的键。
- 整 crate：日志 verus-proof 一行报「成项 198 个……各项 verified 之和 214、0 errors（登记 214）」。所以 `gpu_gc_scoring_proofs.rs:47` 的断言、G17 的量词与触发项、覆盖写按段三条 ensures、交集长度两条引理与见证都被 Verus 验过（`lower_bound_of_sorted_records`、`remaining_gpu_memory`、`lemma_overwrite_changes_exactly_the_requested_bytes` 三项各 `绿，现判`）；上面第三节我写的「触发项能展开」「按段三条要 `overwrite_fits`」里的「推的」，其中前一半（验得过）现在是量的，后一半（离了前提推不出）靠 `overwrite-exact-needs-fit` 的读数：「绿，复用（2026-11-18 判绿：落点 lemma_overwrite_changes_exactly_the_requested_bytes；证明错 2 条（条数与登记的 proof_errors=1 不同，条数只报））」。
- 第五段条数：日志里 8 行报「条数与登记的 proof_errors=1 不同」；今天 `verus-must-red-worlds.tsv` 里其中 7 行第五段仍写 `proof_errors=1`：`overwrite-exact-needs-fit`、`gpu-gc-G01-confirmation-never-asks-cpu`、`gpu-gc-G01-cpu-judgment-restarts-the-count`、`gpu-gc-G01-second-disagreement-asks-cpu`、`gpu-gc-G01-threshold-constant`、`gpu-gc-G18-short-remainder-is-not-overrun`（实测 2 条）与 `gpu-gc-G28-group-count-is-not-truncated`（实测 3 条）；第八行 `indexed-intersection-does-not-drop-written-bytes` 今天写的已是 `proof_errors=2`。表头写第五段「只报不判」，所以不影响红绿，但表头同时写它是「最近一次观测的证明错条数」，这七行是实现员读码写的 1，没改成实测。
- 这几行的落点读数与第六段一致（日志里每行「落点 …」就是实测集合，门禁已按第六段判）：`gpu-score-binary-midpoint-needs-u32-bound` 落点 `lower_bound_of_sorted_records`、证明错 1 条；`gpu-gc-G17-positive-allocation-within-the-remaining-fits` 落点 `remaining_gpu_memory`；`overwrite-definition-start-shifted-by-one` 与 `overwrite-definition-length-short-by-one` 的落点集合各是四个函数（与我第三节读码推的「差在 contained 与 splitting 两个引理」一致，推翻不了）。

## 七、实现员与回扫报的三处小问题（只读核，没改）

| 问题 | 核得 | 证据 |
|---|---|---|
| 必红表第 402–409 行没有自己的注释行、挂在上一段注释底下 | 属实 | 这八行（`verus-must-red-worlds.tsv:402` 起，到 `:409`）紧接在 `:401` 覆盖写行之后；它们上面最近的注释是 `:327`「# 固定长度部分写字节模型：五项契约与三个关键前提。」，那一段讲的是 `partial_write_proofs`，而这八行落在 `file_range_proofs.rs` |
| README 文件表 `file_range_proofs.rs` 一行没写精确交集长度 | 属实 | `crates/singlefs-replay-prefix/src/README.md:18` 写「部分写交集的精确覆盖、两侧偏移与容量上界」；精确长度只在 `:181` 的正文段里写了 |
| `.claude/gate.d/verus-pins.tsv` 第 49 行注释带日期 | 属实；门禁的射程看不到 | `.claude/gate.d/verus-pins.tsv:49`「# 2026-11-18 交集长度组合」；同文件 `:46` 的注释带「2026-11-10」，是存量。`scripts/gate/comment-history-check.py:25` 写射程取 .rs 与 .wgsl、.py、.sh、没有后缀的 shebang 脚本、文件名是 README.md 的，.tsv 不在里面，所以门禁 comment-history 红不了它；`.claude/agent-common.md` 那条「代码与脚本的注释、说明只写现状，不写历史」的字面射程含 `.claude/gate.d/` 下的 `#` 注释。同一份文件 `:47` 另有一句过时的话：「十个产品核」（G30 没有产品调用点，见第二节） |

## 八、什么现象会推翻上面的结论

- 一·4 写宽：W 取 65534 的树上 Verus 报错（推翻我对推导的复核）；或 `GPU_GC_PROOFS.md` 的哪一版写过「证明钉住 65535」（我只查了 HEAD 那一份）。
- 一·5 的 G11：主 agent 认为 G11 的三条是 `record_contribution` 之外的新事实（比如拿产品的记录过滤去对），则「定义自检」标注不适用于它。
- 复跑命令：`:104` 原样提交被接收并跑起测试。
- G30 两处：`.claude/gate.d/verus-pins.tsv:47` 的「十个产品核」若指的是「十个核」不是「十个产品调用的核」，则不算过时（我只读到字面）。
- 必红表两条 `None` 支：有一行的原文落在 `:171` 或 `:221`（我的脚本比过全部 445 行，没有）。
- 「92 行是复用」：键与这棵树不同；那时 20692 的结论要重新取。

## 九、要主 agent 跑的命令

1. 验证复跑命令的写法被拒（探针，原样提交 `GPU_GC_PROOFS.md:104` 那一条；预期提交当场被拒、没有任务号）。在仓根起；看 `task submit` 的退出码与它打的原因，原因含「没点名到单条测试函数」算我的读法对；被接收并跑出「4 passed」算我读错。

SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-replay-prefix 的测试目标 gpu_gc_arithmetic_matches_wide_integer_reference; 范围=1 个测试目标; 为什么=用 u128 参照复核 G01 与 G14 到 G20 的产品核' task-server/task submit -- cargo test -p singlefs-replay-prefix --test gpu_gc_arithmetic_matches_wide_integer_reference

2. 同一目标按服务收的写法点名四条（建议承接的实现员改成这样）。在仓根起；看末行 `TASK_SERVER_RESULT` 的 `type=` 与 `exit=0`，`task-server/task log <号>` 里 `test result: ok. 4 passed`。

SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-replay-prefix 的测试目标 gpu_gc_arithmetic_matches_wide_integer_reference 里的 4 条测试; 范围=4 个测试; 为什么=核点名写法被服务接收，并复核 u128 参照对 G01 与 G14 到 G20 的产品核' task-server/task submit -- cargo test -p singlefs-replay-prefix --test gpu_gc_arithmetic_matches_wide_integer_reference -- gpu_budget_arithmetic_matches_a_u128_reference_at_full_width gpu_slices_scans_and_independent_totals_match_wide_arithmetic gpu_readback_rejects_unwritten_words_and_preserves_both_halves gpu_reconciliation_matches_the_two_round_history_rule --exact

没有别的要跑：判决要引的 Verus 读数已有 20692 的日志（核键后可引）；我提的两条 `None` 支新行要先由实现员写进必红表才能跑，现在没有它们的项名。没写模型，没有 `rerun.sh`、没有「在干净运行目录里复跑」的包装脚本，探针的内存与时间：没起样本进程，只读文件、`grep`、`python3 -I` 读文件的小脚本（秒级）。

## 十、没做什么

- 没往统一任务服务交任务，没起 Verus、cargo、门禁；上面标「推的」的都是读码推的，第九节两条命令由主 agent 跑。
- 没读禁读清单里的文件（另一条腿的报告与模型目录）。
- 没复跑任务 18248 的「W 取 65534 照样验过」；没在副本里改树。
- 没逐行推 75 行新必红每一行 Verus 会报哪几个函数（落点靠 20692 的读数，且那 92 项是复用的）。
- 没判产品 GPU 包那一题的内容（`extended_gpu_chunk_input` 改调、newtype、`live_slot_kernels.rs:379`）。
- 没替主 agent撤销或维持判决，只交复核结果。
- 没数全仓还有没有别的「十个产品核」类的过时句（只在 `crates` 与 `.claude/gate.d` 下 grep 过）。
