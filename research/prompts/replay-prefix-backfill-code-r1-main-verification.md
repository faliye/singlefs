# replay-prefix-backfill-code-r1：判决（2026-11-17）

正文 `research/prompts/_replay-prefix-backfill-code-r1-body.md`，开工快照 `research/prompts/replay-prefix-backfill-code-r1-snapshot/`。这是这一题的第一轮，补的是重放前缀包十四份在途改动没走过的三方。快照清单 `crates-sha256.txt` 末尾 `crates/singlefs-background-gpu/src/live_slot_kernels.rs`、`memory_limit_and_chunks.rs` 两行是事后补的（2026-11-18），哈希对应提交 `3a7bd4fc8` 里那一版；这一轮只把它们当出处引（并进产品 GPU 包那一题）、没判它们，派腿时没记。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 正推 | `research/prompts/replay-prefix-backfill-code-r1-sonnet-output.md` | G01 到 G30 的 `ensures` 与说明文件对得上，G04 是定义复述、G10 只证两份重数、G17 的注释没有对应 `ensures`；`extended_gpu_chunk_input` 没有产品调用点；覆盖写四条引理只罩落在已有 size 之内的写；改名 62 块纯改名；必红表 66 行锚点各命中一次，另列没有反例罩着的 `ensures`；说明文件四处数对不上；检查单「类型与名字」「带 --item 的命令与项」不成立 |
| 攻方 | `research/prompts/replay-prefix-backfill-code-r1-opus-output.md` | 空真没打中、四处 `as` 没打中；打中 G30 没人调、G04 一族 `ensures` 是定义展开、G21 与 G22 的「显存」是交进来的上限、u32 界不承重、覆盖写第一条引理的 `requires` 不承重、窗口常数不被证明钉住；两份测试缺两个边界值；说明文件七处对不上；检查单「走不到的理由」「类型与名字」不成立，「带 --item」推不出 |

正推腿自己披露：它一条过宽的 grep 带出了攻方报告的一行（第 177 行），没再读别处；这条腿当「不完全独立」。两条腿都打中的几格各有主 agent 的现查或复跑顶着，不靠两条腿互证。

核查：

- 写判决之前 `sha256sum -c research/prompts/replay-prefix-backfill-code-r1-snapshot/crates-sha256.txt research/prompts/replay-prefix-backfill-code-r1-snapshot/kb-sha256.txt` 全部 OK，没倒推快照树。
- `python3 scripts/agents/cite-check.py research/prompts/replay-prefix-backfill-code-r1-opus-output.md research/prompts/replay-prefix-backfill-code-r1-sonnet-output.md --root . --background research/prompts/_replay-prefix-backfill-code-r1-body.md` 末行「✓ 核了 52 处引文、231 处裸行号，对上 51 处，没判 232 处（其中裸行号没有引文可比 203 处；逐处列在上面）」；没对上的一处是正推腿只写文件名的引用（仓里有两个同名文件），没当依据。
- 主 agent 现查：`grep -rn extended_gpu_chunk_input crates --include=*.rs` 除重放前缀包自己外零命中；现读 `crates/singlefs-background-gpu/src/live_slot_kernels.rs:372-381`、`crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:430-438`；`ls crates/singlefs-replay-prefix/src/*.rs | wc -l` 得 10，`crates/singlefs-replay-prefix/src/README.md:7` 的节标题写「八份源文件各负责什么」；`crates/singlefs-replay-prefix/src/GPU_GC_SELFTEST.md:1` 标题写「20 项」、`:37` 写着任务号与第二台一张卡的 PCI 地址。
- 复跑的读数（任务 18248，攻方副本里、对临时服务，退 1 在期望内）：对照行 `must-red:gpu-score-window-too-narrow` 绿（改坏之后 Verus 报错、抓到）；三条攻击行都是「改坏之后 Verus 照样验过（verified 208、errors 0）：证明没盯住这一处」——`attack-lower-bound-without-u32-bound`、`attack-window-one-slot-narrower`、`attack-overwrite-exact-without-fit`。副本上的读数只用来坐实「这三处没被证明盯住」，不进 kb。
- 没核的：门禁 formal-verification 的 `--list-items` 不带 `--full-gate` 被服务拒，没交，说明文件点名的项列不列得出没量；「四个主机模块 56 个测试」没列项去数（两条腿各自在源码里数到 82 个 `#[test]`）；正推腿交的 8 项命令没交，留到改完之后那一轮；`G01` 到 `G30` 逐条与 `ensures` 的对照表主 agent 没逐行复核，采的是两条腿一致的那几格。

## 二、判决

1. **三份新证明文件**：
   - 空真、四处 `as` 转换：攻方没打中，正推对得上，记「没打中」。
   - G30 `extended_gpu_chunk_input` 没有产品调用点而头注释（`gpu_gc_capacity_proofs.rs:1`）与 `GPU_GC_PROOFS.md:5` 写成产品直接调用：打中，两条腿一致，主 agent 的 grep 坐实。
   - `GPU_GC_PROOFS.md:22`「证明中间值落在 u32 范围内」：打中，删掉 `gpu_gc_scoring_proofs.rs:35` 那条 u32 界照样验过（任务 18248）。
   - 活槽打分窗口的常数 65535 不被证明钉住（改成 65534 照样验过，任务 18248）：打中，必红表「窗口过窄」那一类只罩窄两格以上。
   - G04 一族的 `ensures` 是 spec 定义的展开、对产品没有约束；G21、G22 的「显存」是交进来的上限，产品先扣暂存份数的那一层在核外：两条腿一致。说明文件对 G04 写了它是模型，对暂存折算那一层没点到。记「性质名比 `ensures` 宽」，改说明文件；暂存折算进不进核并进产品 GPU 包那一题。
   - G17 的注释「正的新增分配必在总限内」没有对应 `ensures`（正推腿，推不出）：算打中，照注释补 `ensures` 或改注释。
2. **固定长度覆盖写**：`lemma_overwrite_changes_exactly_the_requested_bytes` 的 `requires overwrite_fits(...)` 不承重（任务 18248），它是对 `overwritten_bytes` 定义的自检而必红表没有一行改坏定义：打中。另三条引理的前提各有删前提的必红行。四条都不是产品写路径的前提，README 新节写了边界，记「没打中」。
3. **两份新测试**：参照与被测不共用代码；边界值缺 `u64::MAX − 2`、`u64::MAX − 1`（两条腿一致），G27、G29 的参照与被测是同式同字面值（正推腿）：打中，补边界值与独立参照。两份测试没有变异行（`crates/mutations.tsv` 里 0 行）：它们测的函数由必红表罩着，测试自己会不会红没有东西证，打中。
4. **必红表**：新反例是 58 行（另 8 行随改名换锚点），正文写「加 68 行反例」把注释与空行算进去了。逐行的落点两条腿读码推过、没找到落点错的；没有反例罩着的 `ensures` 两条腿各列了一份（正推第 5.3 节、攻方件 4）。第 198 行的变异名还留着旧函数名 `a_segment_…`。
5. **改名一批**：两条腿都判纯改名，`requires`、`ensures`、`decreases` 的结构没动，记「没打中」。`later_record_of_the_same_publish_is_readable` 是定义 1 处加引用 5 处，正文写「6 处引用」不对。
6. **三份说明文件**：对不上的数坐实（主 agent 现查了前两处）：README「八份」「另七份」应为十份、九个模块且表里缺两份新证明文件；`gpu_gc_proofs.rs:1` 把 20 项清单指到 README；`GPU_GC_SELFTEST.md` 写的是加 G21 到 G30 与打分证明之前那棵树的数（186 条义务，登记表今天 208）；两份都写「56 个测试」；`GPU_GC_PROOFS.md` 没提第二份新测试。`GPU_GC_SELFTEST.md` 整份是一轮自测的经过（任务号、「修复前」、第二台那张卡的 PCI 地址），不是现状说明。
7. **检查单八格**：
   - 走不到的理由二次查实：被判的十四份里没有新加的走不到断言，这一格成立；攻方判不成立，点的是产品 GPU 包的 `live_slot_kernels.rs:379`（对 GPU 回读求和的 `None` 用 `expect`），那一份不在这一轮被判的文件里，并进产品 GPU 包那一题当已抓到的问题。
   - 类型与名字：两条腿都判不成立（exec 核的参数是裸 `u32`、`u64`，字节数、条数、槽号、显存上限共用）。
   - 带 --item 的命令与项：正推判不成立（说明文件里的复跑命令是 `--manual`、不带范围声明、不能原样交），攻方推不出。算打中，说明文件里的命令照现行写法改。
   - 其余五格两条腿都判成立。`GPU_GC_SELFTEST.md` 用自己起的临时服务，合「判定库只有一份」许的那一种。

## 三、改法

1. `extended_gpu_chunk_input`（用户定案第 41 条，`records/2026-11-16-三方豁免取消与存量补轮.md`）：产品 `memory_limit_and_chunks.rs:430-438` 那一段改成调它。攻方提醒不能照搬：产品每次按输入长度折上限，G30 收的是一份固定的 `capacity`。**零轮**；写进产品代码之前先在探针里对拍两种写法在边界值上同值（任务号改的那一批补记），并进产品 GPU 包那一题。
2. exec 核的参数换成 newtype（用户定案第 42 条）；产品那一侧的调用点跟着改。**零轮**，改完再攻。
3. `lower_bound_of_sorted_records` 里补「中点加一不超过 u32」的断言并配一行删界的必红行；窗口常数配一行「窄一格」的必红行，验得过就把说明文件里那一句改成证明实际钉住的界；`overwritten_bytes` 的定义配一行改坏定义的必红行，`lemma_overwrite_changes_exactly_the_requested_bytes` 不承重的 `requires` 删掉或改成承重；G17 的注释与 `ensures` 对齐。攻击行在副本里**量过**不红（任务 18248），改完同三行要红。
4. 两条腿列的没有反例罩着的 `ensures`，逐条补必红行。
5. 两份测试补 `u64::MAX − 2`、`u64::MAX − 1` 两个边界值，G27、G29 的参照改成与被测不同式；每条测试挂一行变异。
6. 说明文件：README 的份数与表、`gpu_gc_proofs.rs:1` 的指向、`GPU_GC_PROOFS.md` 的测试数与第二份测试、G04 与暂存折算的边界、复跑命令的写法照改；`GPU_GC_SELFTEST.md` 从 `crates/` 删掉（它是一轮的经过，git 历史里留着），其中还成立的现状句并进 `GPU_GC_PROOFS.md`。必红表第 198 行的变异名改成新函数名。
7. 并进产品 GPU 包那一题的两条：`live_slot_kernels.rs:379` 对 GPU 回读的数 `expect`；G21、G22 之外扣暂存份数的那一层没验。

## 交用户的

| 事 | 状态 |
|---|---|
| G30 没有调用点怎么改 | 用户已定（第 41 条） |
| exec 核的裸整数参数 | 用户已定（第 42 条） |

## 这一轮被判过的文件

- crates/singlefs-replay-prefix/src/GPU_GC_PROOFS.md：判了它列的 G01 到 G30 与证明、产品调用点、测试数对不对得上，打中 G30 的调用说法、u32 那一句、测试数与复跑命令几处，照第三节第 6 条改。
- crates/singlefs-replay-prefix/src/GPU_GC_SELFTEST.md：判了它的数与代码对不对得上、它是不是现状说明，判为一轮自测的经过，照第三节第 6 条从 crates 删掉。
- crates/singlefs-replay-prefix/src/README.md：判了源文件份数、模块表与新加的覆盖写一节，份数与表对不上，照第三节第 6 条改。
- crates/singlefs-replay-prefix/src/compaction_plan_proofs.rs：判了改名的每一块，纯改名，requires、ensures、decreases 的结构没动。
- crates/singlefs-replay-prefix/src/file_range_proofs.rs：判了改名的每一块，纯改名，requires、ensures、decreases 的结构没动。
- crates/singlefs-replay-prefix/src/gpu_gc_capacity_proofs.rs：判了 G21 到 G30 的 ensures、三处 as 转换与 exec 核的参数类型，打中 G30 没有产品调用点、参数共用裸整数，照第三节第 1、2 条改。
- crates/singlefs-replay-prefix/src/gpu_gc_proofs.rs：判了 G01 到 G20 的 ensures、一处 as 转换、空真与头注释的指向，打中 G04 一族性质名比 ensures 宽、G17 注释没有对应 ensures、参数共用裸整数、头注释指错文件。
- crates/singlefs-replay-prefix/src/gpu_gc_scoring_proofs.rs：判了二分与候选窗口的证明，打中 u32 界与窗口常数不被证明钉住（副本里攻击行照样验过），照第三节第 3 条改。
- crates/singlefs-replay-prefix/src/lib.rs：判了三行模块声明与 spec 函数、证明函数、测试的改名，纯改名。
- crates/singlefs-replay-prefix/src/partial_write_proofs.rs：判了固定长度覆盖写的四条引理、一个见证与形参改名，打中第一条引理的 requires 不承重、定义没有必红行罩着。
- crates/singlefs-replay-prefix/src/unlink_proofs.rs：判了改名的每一块，纯改名，requires、ensures、decreases 的结构没动。
- crates/singlefs-replay-prefix/tests/gpu_gc_arithmetic_matches_wide_integer_reference.rs：判了参照与被测共不共用算式、边界值够不够，打中缺两个边界值、没有变异行，照第三节第 5 条改。
- crates/singlefs-replay-prefix/tests/gpu_gc_capacity_matches_independent_reference.rs：判了参照与被测共不共用算式、边界值够不够，打中缺两个边界值、没有变异行，照第三节第 5 条改。
- crates/singlefs-replay-prefix/verus-must-red-worlds.tsv：判了新加的 58 行反例与 8 行随改名换的锚点，锚点各命中一次；打中若干 ensures 没有反例罩着、第 198 行变异名留着旧函数名。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 17 | 不影响 | 2026-11-17 不受影响：这一轮没判键的组成，只引它当背景 |
