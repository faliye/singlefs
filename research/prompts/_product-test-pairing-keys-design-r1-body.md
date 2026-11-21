# 产品包测试的对拍键怎么算（产品与测试不共用代码）（第一轮）

方案按 `crates/` 今天的实现来谈。调度记录 `records/2026-11-19-产品GPU包测试与E202装置归位.md`。

## 问题

产品 GPU 包（`crates/singlefs-background-gpu`）的真卡对拍测试，把「输入、CPU 参照、对拍结论」按单元键写进判定库、照键复用。拆包第一簇的做法是产品包 `build.rs` 经 `singlefs-unit-graph` 算两侧的判法单元键、编进测试（`[build-dependencies]` 加 `singlefs-unit-graph`）。用户否掉了这个做法。要推的是：在下面三条前提下，产品包测试的键怎么算、算键的代码住哪。

## 已定的前提（用户原话，不在这一轮里重议）

1. 「这个部分如果是产品包就放入产品中 如果是测试包 就放入测试。但是测试和产品不得用一样的代码」
2. 「不允许。 需要的话自己去产品中写新实现。或者测试写新实现。必须解耦。 产品是产品 测试是测试 不得共用 除非是格式类的」（针对「算键那一段挂在测试才开的特性下、产品包可选构建依赖单元图」）
3. 测试就该在测试的地方：产品包的测试住 `crates/singlefs-background-gpu/tests/`（这一题的总定案）。
4. 跑判定的任务都要按键复用进库、能单项跑（用户 2026-11-05 定的，`.claude/rules/verification.md`「一个门禁：两种跑法」「按判定入库」）；键只由判定依赖的函数单元去注释指纹、真引到的 cfg 原子、工具链、依赖版本与判定自己的内容输入组成（D13（验证路线） 已定项 17）。

## 实现今天的样子

- 单元键在验证包里由 `crates/singlefs-verification/build.rs` 经 `singlefs-unit-graph` 按 `src/judging_units_entries.rs` 算好、编进二进制；`crates/singlefs-verification/src/crash_identity.rs:804` 写明键「跑时不读盘上的树，也不收仓根」，`:629` 用 `include_str!(concat!(env!("OUT_DIR"), "/judging_units_keys.tsv"))` 读回。
- 拆包第一簇补丁（`~/.local/share/singlefs/work/gpu-tests-placement-product-tests/patch` 与修补 `~/.local/share/singlefs/work/gpu-tests-placement-product-tests-repair/patch-with-the-tests`）里，产品包 `build.rs` 照 `tests/common_pairing_side_units_entries/mod.rs` 三张登记表经单元图算键；产品包测试经 `prune_server::reliable_client` 用自己一族键（`background-gpu-pairing`）存取。这一版在合入后验证里 fmt、clippy、整包 385 条测试都过，被用户的定案挡住、没合。
- 第一轮设计正推腿第 4.4 节（`research/prompts/gpu-tests-placement-design-r1-sonnet-output.md`）：测试运行时算键要读 `crates/**/*.rs` 的源码树，测试运行时也拿不到构建时的 `CARGO_ENCODED_RUSTFLAGS`、`TARGET`、`$RUSTC -vV`。
- 产品包的测试目标没有自己的构建脚本；一个包只有一个 `build.rs`，它是产品包的。

## 判据格

每格答「成立 / 不成立 / 推不出」，给依据（文件:行）；写做法的写成对实现的改动。

1. **候选清单**：列出至少三种做法，各写清算键的代码住哪个包、哪一段在构建时、哪一段在运行时、产品构建（不开任何特性的 `cargo build -p singlefs-background-gpu`）会不会编进或跑到任何测试代码。至少包括：甲，测试运行时从编进测试二进制的源码（`include_str!` 本包测试与产品源文件）现算键；乙，产品包测试的对拍不按单元键、改按别的输入（例：测试二进制摘要）入库——注意 2026-11-11 用户否过「键取测试可执行文件的摘要」的特例；丙，键由一个只依赖格式与测试支撑的测试支撑包的构建脚本算、产品包测试经 dev 依赖读它；丁，你能想到的别的。
2. **每个候选撞不撞前提**：逐条对前提 1–4；「格式类可以共用」指什么（`singlefs-format` 那一类），单元图算不算。
3. **键的组成在各候选下能不能做全**：工具链、cfg 原子、依赖版本、跨包闭包（参照一侧的函数会走到 `singlefs-core`），各候选怎么拿到；拿不全的算哪一项缺、后果是复用错还是只是少复用。
4. **代价**：各候选要写多少（推的写明推的）、要改哪些文件、对单项跑与复用的影响。
5. **防再发**：选定的做法怎么做成会红的检查（例：产品包任何依赖表里出现测试支撑包判红）。
6. **对拍的参照不得是产品代码**（用户 2026-11-19 原话：「我们的checker的实现 不得和产品一致。 这一点是最核心的规范对吧。」「可以 那么就改吧 如果判定和检测强度不够 那么就设计一下」；规则 `.claude/rules/fs-design.md:22`「审计与被审计用同一段代码」被禁止；D13（验证路线） 已定项 5「其余一律不共享」）。主 agent 现查：产品包测试拿来当参照的 `cpu_reference_outcome`（`crates/singlefs-background-gpu/src/checksum_kernels.rs:213`）、`cpu_reference_live_slots`（`live_slot_kernels.rs:479`）、`cpu_reference_classification`（`region_record_kernels.rs:1344`）都是产品运行时代码：两份 GPU 核不一致时由 `reconcile_unless_the_batch_failed` 拿它们裁决（`checksum_kernels.rs:256`、`live_slot_kernels.rs:527`、`region_record_kernels.rs:1421`）。答：产品包测试（与拆包第一簇搬过去的 13 个目标）里，哪些断言以产品代码为参照、逐条列（文件:行）；测试那一侧各自写独立的参照（CRC32C、每段活槽数、区域记录分类），写在哪、多大、与产品那一份「算法不同」怎么判（同一份算法换个写法不算独立）；CPU 替身对拍（`background_gpu_compute_agrees_with_the_cpu_backed_test_double`）的另一边是不是也落在产品代码上。
7. **判定与检测强度**：换成独立参照之后，这些对拍能抓到哪类产品错、抓不到哪类（例：两份 GPU 核与 CPU 裁决一起错、输入取样只覆盖一部分长度与偏移）；抓不到的给出补法（加哪类取样、哪条变异要红、要不要接池级 checker 判整理之后的盘），各写代价。

## 分工

- 正推腿：七格都答，第 1 格每个候选写到文件与函数，第 6 格逐条列以产品代码为参照的断言。
- 攻方腿：攻第 2、3、6、7 格：每个候选里藏着的共用代码（产品与测试经同一份算法、同一个包）、拿不全的键的组成会不会让判定被错复用、测试那一侧的「独立参照」是不是换个写法的同一份算法、换完之后哪类产品错仍然一条测试都不红；再攻「实现今天的样子」里每一句。
