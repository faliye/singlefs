# 代码轮 judging-closure-and-real-card-instance-code-r1：正推腿报告（2026-11-18）

腿：正推（three-way-forward）。从 D13（验证路线）已定项 17 与 `crates/` 今天的实现推「应该是什么」，再与被判的四份文件、一张表逐格比。
读的是合入之后主工作区里的文件（行号是此刻 `grep -nF` 取的）；三份实现员报告当线索读，不当证据。本腿一条命令都没起，
要量才知道的写进末尾「要主 agent 跑的命令」。问题编号照派发提示的 1 至 5：1 是两批登记，2 是 `units_changed_between`，3 是数据文件清单三条测试，4 是真卡测试共用实例，5 是八格检查单。

## 问题 1：两批登记（七个单元）对条款，与守它们的测试

### 1.1 条款里与这件事有关的句子

- `.claude/kb/decisions/13-验证路线.md:435`：「加这份判定自己的内容输入」。键由单元指纹、cfg 原子、工具链、依赖版本与这份判定自己的内容输入组成。
- `.claude/kb/decisions/13-验证路线.md:436`：「键过窄（引用的常量、类型、宏漏收）」算违反；同一行的「行号」不进键、也不许当复用依据。
- `.claude/kb/decisions/13-验证路线.md:437`：「每把键连同组成它的单元清单落盘，作废时报得出是哪几个单元变了」（问题 2 用）。
- `.claude/kb/decisions/13-验证路线.md:438`：单元级「登记要配五条测试」，五条是：不带登记收时够到的恰好等于登记的；登记的每个名字在工作区里恰好一个单元；
  不沿它收少掉的恰好是登记的那些；留下的单元不写登记里的包装类型名；改一个登记了的单元，这一组的键不变。
  同一行写「今天只有 GPU 判器那一组有登记」，并点名两个常量。
- `.claude/kb/decisions/13-验证路线.md:444`：射程里写「GPU 判器那一组要切的几类（故障注入、读计数、核心层挂载这类包装设备）走上面的单元级登记」。

条款对单元级登记没有写「什么样的单元可以登」的判据，只写机制（不进闭包、不往下收、真够到的另交）与五条测试；
为什么不改判定结局由登记的调用方担（`crates/singlefs-unit-graph/src/closure.rs:3147` 的文档「登记什么、为什么不改判定结局由调用方担」）。

### 1.2 代码今天做的（读的行）

- `crates/singlefs-verification/src/judging_units_entries.rs:1178`：「pub const UNITS_THAT_ONLY_PRINT_THE_GPU_LIFECYCLE_LINE_THE_GPU_JUDGE_GROUP_DOES_NOT_FOLLOW:」，三个单元 `gpu_lifecycle_line`、`gpu_lifecycle_trace`、`gpu_work_trace`。
- `crates/singlefs-verification/src/judging_units_entries.rs:1196`：「pub const UNITS_REACHED_ONLY_THROUGH_THE_UNITS_THAT_PRINT_THE_GPU_LIFECYCLE_LINE_THE_GPU_JUDGE_GROUP_DOES_NOT_FOLLOW:」，四个单元，其中 `judging_units_entries.rs:1200`：「"singlefs_verification::enum_listing_every_member!@78",」。
- `crates/singlefs-verification/src/judging_units_entries.rs:1207`：「pub fn units_not_followed_of_the_group(group_name: &str) -> Vec<&'static str> {」，`gpu_judge` 一组把四批 `.chain` 起来，别的组交空表（`judging_units_entries.rs:1217`：「Vec::new()」）。
- 收法：`crates/singlefs-unit-graph/src/closure.rs:1389`：「if self.units_not_followed.contains(&unit.qualified_name) {」，命中的单元记进真够到又没沿它收的那一份（`closure.rs:1391`：「.insert(unit.qualified_name.clone());」）后直接返回，不进闭包、不往下收。
  键只由收出来的闭包算（`closure.rs:3264` 起的 `reuse_key`，单元那几行在 3282 起：每个单元只取（种类，指纹）），登记本身与名字、行号都不进键。
- 登记在组入口登记表旁，按限定名，登记为空时与不带登记逐字相同：与条款 438 一致。

### 1.3 七个单元对五条测试（读测试体得出）

测试都在 `crates/singlefs-verification/src/crash_identity.rs`，挂 `gpu` 特性、读活树。取整份登记的是 `units_not_followed_of_the_group_as_a_set("gpu_judge")`（`crash_identity.rs:1327`：「units_not_followed_of_the_group_as_a_set("gpu_judge")」），所以按名字比的几条自动含这七个。

| 条款五条里的 | 守它的测试（行） | 对七个单元的作用 |
|---|---|---|
| 不带登记收时够到的恰好等于登记的 | `the_gpu_judge_group_reaches_no_unit_of_the_memory_ledger_beyond_the_listed_ones`（1268） | 只比显存记账那个文件里的单元：`crash_identity.rs:1285`：「.filter(|line| line.contains(MEMORY_LEDGER_FILE))」。对七个没有作用。 |
| 登记的每个名字恰好一个单元 | `every_listed_unit_of_the_memory_ledger_names_exactly_one_unit_of_the_workspace`（1316） | 名字虽带 memory_ledger，取的是整份登记，七个都罩。`<impl RecordedOperationKind>` 在 `lib.rs:97` 只有一块（`grep -rnE 'impl[^{;]*RecordedOperationKind'` 全 crates：除 `judging_units_entries.rs` 里登记与文档的字符串外，只命中 lib.rs:97 一处）。 |
| 少掉的恰好是登记的 | `not_following_the_listed_units_drops_only_the_listed_ones`（1351） | 七个都罩，两个方向：少掉而没登记的（新进来、只经登记够到的）与登记了而没少掉的（陈旧）都红。承担了上一行那条对七个的作用。 |
| 留下的单元不写包装类型名 | `no_unit_the_gpu_judge_group_keeps_names_a_wrapper_type_it_does_not_follow`（1381） | 包装类型名只从包装设备那一批取（`crash_identity.rs:1389`：「UNITS_REACHED_ONLY_THROUGH_WRAPPER_DEVICES_THE_GPU_JUDGE_GROUP_DOES_NOT_FOLLOW」），对七个没有作用。换成两条对应的：见下两行。 |
| （对应）留下的单元读不到打日志单元交的值 | `no_unit_the_gpu_judge_group_keeps_reads_a_value_returned_by_a_listed_unit_that_only_prints_the_gpu_lifecycle_line`（1463） | 罩三个打日志的单元。 |
| （对应）只经登记够到的四个没被留下的单元真够到 | `the_units_listed_as_reached_only_through_the_units_that_print_the_gpu_lifecycle_line_are_reached_through_them_and_by_no_unit_the_group_keeps`（1587） | 罩四个，两半：沿打日志那批收时至少一个被真够到（不陈旧）；带全部登记收时一个都没被真够到。 |
| 改一个登记了的单元键不变 | `an_edit_of_the_listed_gpu_lifecycle_trace_…`（1548）、`…_gpu_work_trace_…`（1561）、`…_gpu_lifecycle_line_…`（1573） | 三个打日志的单元各一条；四个只经它们够到的单元没有改它们的测试。 |

### 1.4 条款有而代码没做、代码做了而条款没写

条款有、代码没做或只做一半：
1. 「不带登记收时够到的恰好等于登记的」那一条对新两批没有同形的测试（上表第一行只比显存记账文件）。由第三行的两个方向兜住「陈旧」与「新进来的只经登记够到的单元」；
   兜不住的一种：不带登记收时，被留下的单元够到、而又不在登记里的单元——这种单元本来就该留在键里，不算漏。所以这一条对新两批是「被更强的两条替了」，不是缺口。
2. 「改一个登记了的单元，键不变」：四个只经打日志批够到的单元没有改它们的测试。它们在带全部登记收时不在闭包里，改它们不会动键，这条对它们是同义反复；条款字面是「一个」，按组算有三条，够。
3. 条款 438 点名了两个常量，今天有四个（`units_not_followed_of_the_group` 链了四批）。条款现状句过时。三份实现员报告里「交主 agent 的事」已写到，归写回员（不归本腿）。

代码做了、条款没写：
1. 交值检验与「只经登记够到」检验两条测试（1463、1587）；条款的五条里没有这两种前提，是实现员为新两批补的。
2. 带行号的限定名登记（`enum_listing_every_member!@78`）。条款写「按单元（限定名）」，没提行号；包装设备那一批早有同形登记（`@134`、`@1101`、`@8740`），是先例。
3. 打日志批的理由是「只打提示、不改判定结局」，这是条款 438 里边级排除的理由（`一条只打提示、不改判定结局的子进程边`），这里却用在单元级。边级要配三格自证（异常不改退出码、读输出前去掉这条路的行、切边前后少掉的只经这条边可达），条款没说单元级要不要同样的三格。
   代码里对应的是：第三格 = `not_following_…`（1351）；第二格 = 门禁与变异判法里滤掉这一行（见 1.6）；第一格（这条路上的异常不改结果）没有测试，靠读码（见 1.5）。

### 1.5 三个打日志的单元：交不交值、有没有副作用（读码）

- `crates/singlefs-verification/src/gpu_unit_checks.rs:1007`：「pub(crate) fn gpu_lifecycle_trace(card: &GpuCard, phase: &str, detail: std::fmt::Arguments<'_>) {」，没有返回类型，只转给 `gpu_work_trace`。
- `crates/singlefs-verification/src/gpu_unit_checks.rs:1018`：「pub(crate) fn gpu_work_trace(phase: &str, detail: std::fmt::Arguments<'_>) {」，没有返回类型。函数体里：时钟起点是函数内的 `static`（`gpu_unit_checks.rs:1020`：「static ORIGIN: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();」），只此函数读写；写标准错误的结果被吞（`gpu_unit_checks.rs:1029`：「let _ = writeln!(」）；
  取墙钟用 `unwrap_or_default`，没有会 panic 的 `expect`、`unwrap`。
- `crates/singlefs-verification/src/gpu_unit_checks.rs:1044`：「fn gpu_lifecycle_line(」，返回 `String`，是三个里唯一交值的；测试之外只被 `gpu_work_trace` 调（`gpu_unit_checks.rs:1032`：「gpu_lifecycle_line(」），测试里一处（`gpu_unit_checks.rs:3244`）。拼字里的 `thread.name()`（`gpu_unit_checks.rs:1055`：「thread.name(),」）是线程名，不碰卡。
- 调用点：`grep -nE '(=|let|return|\?|,|\()\s*(crate::gpu_unit_checks::)?gpu_(lifecycle|work)_trace\b' crates/singlefs-verification/src/*.rs`，输出只有一行 `crash_judge_dispatch.rs:30` 的 `use` 引进；
  别的调用点（`crash_judge_dispatch.rs`、`gpu_unit_checks.rs` 里共 20 来处）都是语句位置、不取值。唯一一处把它当值传的是 `crates/singlefs-verification/src/gpu_unit_checks.rs:745`：「&gpu_work_trace,」，交给参数类型是 `&dyn Fn(&str, Arguments)`、不交值的 `trace`（`gpu_unit_checks.rs:516`）。
- 副作用只有两样：往标准错误写一行、动函数内那个只给自己用的时钟起点。没有写全局状态给别的单元读、没有改卡的状态。
  标准错误锁是可重入的，写的时候没有别的锁；管道没人读时写会卡住，卡住是挂钟问题，不是判定字（条款 436 把限时排在键外）。
- panic 路径：写失败被吞。`format!` 里 `detail` 的 `Display` 实现若返回错会 panic，我只核了 `crash_judge_dispatch.rs:2444` 那类调用实参是 `{test_name:?}`、卡序号与总线号，没逐个读全部二十来处实参：这一点**推不出到每一处**，写进推翻条件。
- 结论：三个单元不交值给留下的单元、没有会改判定字的副作用——**成立（读码）**。要换成「量的」，看末尾命令 1 与命令 2。

### 1.6 读输出的一侧（这一行谁读、谁滤）

- 门禁：`.claude/gate.d/crash-point-replay.sh:1530`：「BULK_PREFIXES = (b"GPU_LIFECYCLE ", b"CRASH_JUDGE_ACCOUNTING_SLICE ")」，逐提交的行不进日志。
- 变异判法：`scripts/mutation/crates-mutation-rows.py:5267`：「return "GPU_LIFECYCLE unix_us=" if break_is(environment, "lifecycle-prefix-renamed") else "GPU_LIFECYCLE unix_microseconds="」，前缀函数；两处都有自证项（门禁 749 行起的说明、变异脚本 9075 与 11921 起的项）。
- 全仓 `grep -rln 'GPU_LIFECYCLE' --include=*.rs --include=*.sh --include=*.py --include=*.tsv`（去掉 `research/`、`target/`）只命中这两个读日志的脚本、变异判法的自证夹具 `scripts/mutation/crates_mutation_rows_selftest_fixtures.py`、`crates/mutations.tsv` 与验证包自己的源文件。
- 我**没核**：别的读标准错误的地方（任务服务取日志、各测试里读子进程标准错误）有没有把这一行当内容比。命中数为零只说明它们不点名前缀，不说明它们不读；推不出。

### 1.7 四个「只经它们够到」的单元：两条边（读 `closure.rs` 与解析层）

条款不要求读边，但实现员的话是读码推的、没跑，这里把每一环对到行：

1. 裸名收进 `plain_names` 时不看前面有没有 `::`，只去关键字（`crates/singlefs-unit-graph/src/rust_items.rs:277`：「plain_names.insert(name);」），所以 `use std::io::Write;`（`gpu_unit_checks.rs:1019`：「use std::io::Write;」）里的 `Write` 是 `gpu_work_trace` 的裸名。
2. 条目级宏调用的「声明名」是它括号里所有非关键字标识符（`rust_items.rs:2580`：「new_unit.declared_names = Some(self.item_macro_declared_names(open, close));」），`lib.rs:78` 那次调用里有成员 `Write`；按名字登进索引（`crates/singlefs-unit-graph/src/workspace.rs:1283`：「by_name.entry(name.clone()).or_default().push(unit_index);」）。
3. 单元里另有函数体里的 `use`，认不出落在哪一份：`closure.rs:1889`：「if word == "use" && is_identifier_at(position) && position != first_token {」，之后 `closure.rs:1951`：「return candidates.to_vec();」全连。所以 `gpu_work_trace` 连到那次宏调用。
4. 单元名带行号：`rust_items.rs:2574`：「format!("{}!@{}", invocation.macro_name, self.line_of(header)),」，`lib.rs:78` 的头在 78 行（`crates/singlefs-verification/src/lib.rs:78`：「singlefs_core::enum_listing_every_member! {」），名字 `…::enum_listing_every_member!@78` 对得上登记（`judging_units_entries.rs:1200`）。
5. 宏调用进闭包时，括号里 `enum RecordedOperationKind` 算宏生成的类型（`closure.rs:1441`：「self.name_type(type_name);」，类型表来自 `workspace.rs` 里 `TYPE_KEYWORDS_IN_AN_ITEM_MACRO` 那一段），类型被点名之后，之前「见过」的方法名 `name` 连到它 impl 里的成员；
   `name` 是「见过」的因为 `gpu_lifecycle_line` 里写了 `.name(`（`rust_items.rs:244`：「method_names.insert(name);」，`closure.rs:1970`：「if !self.methods_seen.insert(method_name.clone()) {」）。成员连头走 R-结构。`lib.rs` 里 `RecordedOperationKind` 的 inherent impl 只有一块（`lib.rs:97`，成员 `name` 在 `lib.rs:101`）。
6. 宏调用按宏名连到 `macro_rules!`：`history.rs:114` 的转发宏（`#[macro_export]` 在 `history.rs:113`）。

各环对得上，**两条进来的边都是同名撞上**（标准库 `Write` 撞成员 `RecordedOperationKind::Write`、`Thread::name` 撞 `RecordedOperationKind::name`）——**成立（读码）**。
「只经那三个」这半句靠的是测试 1587 的后半段：带全部登记收时，留下的单元一个都没有边连到这四个（边是整张图的，不止头一条）。
`unit-graph why` 只打头一条边（`crates/singlefs-unit-graph/src/output.rs` 里 `why_lines` 的文档「这里只有头一条」），证不了「只经」，只能核头一条是不是我读的那条；命令 2 给了它。
另做了一次读码旁证：验证包里写到 `RecordedOperationKind` 这个词的源文件是 `fault_injection.rs`、`segments.rs`、`memory_pool.rs`、`device_log.rs`、`lib.rs`、`new_pool_file_creation_regions.rs`、`history.rs`、`reference_filesystem_translation.rs`、`scenario.rs`（`grep -lw`）；
GPU 判器入口所在的 `crash_judge_*.rs`、`crash_facts.rs`、`crash.rs`、`crash_given_states.rs` 里这个词的计数都是 0（`grep -c`）。这些是文件级旁证，不是闭包级。

### 1.8 「把它们登进不沿它收」会不会切掉留下的单元真要的东西（键过窄，条款 436）

推的链，每一环有测试守：
1. 闭包带登记 = 闭包不带登记 − 登记的：`not_following_the_listed_units_drops_only_the_listed_ones`（1351）断言「少掉的恰好是登记的」，所以没有别的单元被带着切掉。
2. 三个打日志的单元不交值、没被留下的单元当值读：1.5 的读码加 1463 的断言（「留下的单元里写到交值的登记单元的名字」为空）。
3. 四个单元没有留下的单元的边连到它们：1587 后半段。
4. 图本身漏边：条款 437 的做法是认不出的形态拒算，不静默少收，所以图漏边会表现成拒算而不是键变窄；这一轮没碰这一点。
结论：键不变窄——**成立（读码，并有 1351、1463、1587 守着）**。它们改了字，这一组的键不变，是条款 436 要的「键只由函数单元组成」的结果，不是漏收。

### 1.9 带行号的登记名（`enum_listing_every_member!@78`）

- 收闭包按限定名逐字比（`closure.rs:1389`），名字里的 `@78` 是切单元时拼的所在行（`rust_items.rs:2574`）。`lib.rs` 的 1 至 77 行是 `pub mod …;` 声明，加一条模块声明就让这次调用挪一行，登记成陈旧。
- 陈旧时键：键只取闭包里每个单元的（种类，指纹），不看名字与行号（`closure.rs:3264` 起的 `reuse_key`）。陈旧的名字不匹配任何单元，所以挪行本身**不让键变宽或变窄**：
  这次调用只经 `gpu_work_trace` 够到，而 `gpu_work_trace` 仍在登记里，不被收，它进不了闭包，键不动。
- 变窄的唯一走法：另有一次同名宏调用刚好落到 78 行（比如在 `lib.rs` 上面再加一次 `singlefs_core::enum_listing_every_member!`，原来那次挪去 80 行）。这时 `@78` 匹配到新的那次：
  若它被留下的单元真够到，它被切掉（窄）——1587 后半段红（「真够到又没沿它收」里有它）；若没被够到，没有影响；而原来那次（现在 `@80`）仍只经 `gpu_work_trace`，进不了闭包，在「少掉而没登记的」里——1351 红。两条在同一趟测试里一起红，窄不会先于红。
- 实现员那句「届时 `not_following_the_listed_units_drops_only_the_listed_ones` 当场红」——**成立（读码）**。红的是挂 `gpu` 特性的库测试，不是默认 `cargo test` 那一趟；提交时的 `feature-gated-tests` 那一格会带它们（合入后验证里该格带 `--features gpu` 跑 `--lib`）。
- 代价：每次改 `lib.rs` 的 78 行以上都要同步改一次登记的行号（包装设备那批在 `history.rs` 里的 `@134`、`@1101`、`@8740` 同理，行号更大、更易挪）。这是噪声，不是错。

### 1.10 问题 1 的推翻条件

- 命令 1 里的 `not_following_…`、1463、1587 任一条红，或命令 2 的 `why` 头一条边不是 1.7 写的那几条：1.7、1.8 的读码就不对。
- 标准错误的某个读者把 `GPU_LIFECYCLE` 行的内容（不是前缀）比进判定：1.6 的「读的一侧」不成立。
- 哪一处调用 `gpu_work_trace` 的 `detail` 在 `Display` 里会 panic：1.5 的 panic 路径一句不成立。

## 问题 2：`units_changed_between` 改成两步配对之后，红句还报不报得出变了的单元

### 2.1 读到的

- 比的是组成清单行：`crates/singlefs-verification/src/crash_identity.rs:787`：「let base = format!("{} {} @ {file}", unit.kind, unit.qualified_name);」，即 `<种类> <限定名> @ <文件>`，不带行号；但限定名本身带 `@<行>` 的那几种单元（`use`、条目级宏调用、WGSL 的 `const_assert`）名字里仍有行号。函数单元名字里没有行号。
- 两步：`crash_identity.rs:483` 的 `units_changed_between` 先按原名配一轮、再按去掉 `@` 后数字的名字配一轮，配的条件都是（名字，指纹）整个相等（`crash_identity.rs:521`：「.entry((compared_name_of(unit.unit.as_str()), unit.fingerprint))」）；
  配上的从两边各拿掉，剩下的并成一个有序去重的名字表。去行号只去 `@` 后紧跟的数字（`crash_identity.rs:553`：「if follows_an_at_sign && character.is_ascii_digit() {」）。
- 用处只有红句的文字：`crates/singlefs-verification/src/crash_amplification.rs:27196`：「changed_units_sentence(&units_changed_between(&stored_composition, now.composition()))」。不进键、不进判定，所以这里漏报只会让人少看到一个名字，不会让判定错。

### 2.2 四种输入各交什么（推，不跑）

记 `(去行号名, 指纹)` 为桶。一个桶里两边各有 s、n 个单元，配完剩下 |s−n| 个，哪个一次也配不上的就是没配上的那一边的名字。

| 输入 | 第一步 | 第二步 | 交什么 |
|---|---|---|---|
| 只挪行：库里 `use…use@108` 指纹 1，现在 `use…use@109` 指纹 1 | 原名不同，不配 | 去行号同名、指纹同，配上 | 空表。`crash_identity.rs:3143` 的断言：「units_changed_between(&stored, &only_moved),」期望空表 |
| 挪行又改内容：`use@108` 指纹 1 → `use@109` 指纹 9 | 不配 | 指纹不同，不配 | 旧名与新名各一个（两个名字）。测试 `crash_identity.rs:3167` 期望 `use@108`、`use@109` 与一个指纹变了的 `fn` 单元共三个名字 |
| 删一个单元：库里有 U，现在没有 | U 没有对手 | 没有对手 | `[U]`。不带行号的函数单元就是 `the_composition_reads_back_and_the_red_sentence_names_the_changed_units`（3005）里的 `walk` |
| 加一个单元：现在有 U，库里没有 | 同上 | 同上 | `[U]`（该测试里的 `crc32`） |

函数单元的名字里没有行号，两步里第一步就把名字与指纹都相同的配完，第二步对它们不起作用：函数单元的报法与改之前逐字相同。只有三种按行号起名的单元走第二步。

### 2.3 问你的三种配错

- 「挪了行又改了内容」：指纹变了，第二步也配不上，旧名新名都报，不漏。
- 「两条同名单元互换了内容」：名字不同的两个函数（A 指纹 1、B 指纹 2 换成 A 指纹 2、B 指纹 1），名字与指纹两样都得相同才配，两步都配不上，A、B 都报。
  名字去行号后相同的两条（比如同一模块里两条不同的 `use`，互换了内容）会配上、都不报：它们（种类，指纹）的多重集没变，键的单元部分（`closure.rs:3282` 起：只取种类与指纹的多重集）也没变，本来就不该变键、不该有红句。
- 「同一名字去行号后撞名」：撞名的几个单元指纹不同时按指纹分桶，各配各的（测试 3112 一带的「两条 `use` 一起往下挪两行」就是这一种）；指纹也相同的几个单元彼此无法区分，只配上少的那一边的个数。
  这一种的残余：几个内容逐字相同、去行号名也相同的单元（同文件同模块两行一样的 `use`）里，删掉或新加的是哪一个，报出来的名字可能是另一个——个数对、名字归属不定。只出现在按行号起名的三种单元里，红句多点名一个行号不同的同样内容的单元，不会少点名。
- 为什么不会漏：任何一个桶里两边个数不等，配完就有剩，剩下的一定被报。名字表为空，推出每个桶两边个数相等，即（去行号名，指纹）多重集相等，也就是键里的单元部分相等。
  所以「键的单元部分变了而这个函数交空表」的情形推不出来。
- 键变而这个函数交空表的情形是有的：键里还有工具链行、依赖版本、cfg 原子与数据单元，组成清单不含它们。那时红句写的是 `crash_identity.rs:566`：「"变了的单元：（组成清单比不出：库里那一把或这一趟的组成清单是空表）".to_string()」，把原因说成清单是空表，不确。
  这一句本轮没改，先前就有；改成两步配对之后更常见（原来挪行产生的假名字不再占位），实现员报告「没做什么」已写到，归主 agent 判。

### 2.4 覆盖

- 测试：`crash_identity.rs:3097`（只挪行、挪了行又改、四种按行号起名的单元）、`crash_identity.rs:3005`（删、加、改、读回）。没有「按行号起名的单元被删、或新加」的输入。
- 变异行：`crates/mutations.tsv` 8197 行「判法单元红句：按行号起名的单元去行号那一步不去数字」一行。第二步的配对条件里指纹也相同这一条（桶键在 `crash_identity.rs:521` 与 `crash_identity.rs:528` 两处）没有对应的变异行：
  把指纹从桶里拿掉，3167 的断言（`use@108` 指纹 1 与 `use@109` 指纹 9 会被配上、不报）应当红，但没有人证过。要证，得补一行变异（写法见末尾「建议、不采纳」一节），本腿不写 `crates/mutations.tsv`。
- 结论：条款 437 那一句（作废时报得出是哪几个单元变了）对这个函数**成立**：不漏报；按行号起名的三种单元，名字归属在内容逐字相同的几个之间可能点错。

### 2.5 问题 2 的推翻条件

- 手造一组（库里、现在）两份组成清单，满足「有一个桶两边个数不等」而 `units_changed_between` 交了空表：推翻 2.3 的「不漏」。
- 去行号函数把不是行号的数字也去掉（限定名里 `@` 后紧跟数字而又不是行号，例如以后某种单元名带 `@2`）：第二步会把两个不同的单元配成一对。今天的三种按行号起名的名字（`use@`、`!@`、`const_assert@`）后面都是行号。

## 问题 3：数据文件清单那三条测试

### 3.1 各钉住了什么

- `crates/singlefs-verification/src/crash_identity.rs:1954`：「fn watched_files_outside_the_crates_and_the_root_manifests(」是纯函数，过滤式在 `crash_identity.rs:1961`：「!path.starts_with("crates/") && *path != "Cargo.toml" && *path != "Cargo.lock"」。
- 1980 `only_the_watched_files_outside_the_crates_and_the_root_manifests_are_copied_beside_the_crates`：现造清单（`.claude/heavy-thread-default`、`Cargo.lock`、`Cargo.toml`、`crates/…/heavy_threads.rs`、`prune-server/src/lib.rs`），
  钉：`crates/` 底下的与根上两份清单被挑掉；剩下的按给的次序。不钉：`crates/` 前缀的边界（`crates-x/…` 这类写法）、`prune-server/Cargo.toml` 这类非根清单该不该挑（今天是挑）。变异行 3787（换过的那一行）去掉 `!`，红。
- 2002 `the_build_without_the_gpu_feature_reads_no_file_outside_the_crates`（`cfg(not(gpu))`）：编进二进制的清单挑出来为空。变异行 8207 不过滤，红。
- 2016 `the_build_with_the_gpu_feature_reads_the_prune_server_library_files_outside_the_crates`（`cfg(gpu)`）：挑出来的含 `prune-server/src/lib.rs` 且每份都在 `prune-server/src/` 底下。变异行 8208 把交给纯函数的清单换成 `[..0]`，红。
- 两条记清单的读的是编进去的那份表（`compiled_judging_units_keys().watched_files`），不读活树，不拷树，所以不是拒算的测试。
- 今天清单什么样是量过的：实现员在快照上跑被换掉的那一条，它的 panic 输出就是清单：不带特性是 `[]`（任务 19593，`~/.local/share/singlefs/work/ledger-topdown-watched-data-files-repair/log19593.txt` 第 62 至 63 行），带 `gpu` 是 `prune-server/src/` 底下 34 份（任务 19587，同目录 `log19587.txt` 第 110 至 111 行）。
  这是被换掉的测试在快照上的输出，不是换成的三条自己的绿；三条自己的绿待命令 4。

### 3.2 被替掉的那条断言原来守的是哪一句

被替掉的 `the_data_files_outside_the_crates_include_the_heavy_thread_registration`（旧版 `git show HEAD:crates/singlefs-verification/src/crash_identity.rs` 第 1705 行起）断言清单含 `.claude/heavy-thread-default`。
条款里没有一句直接讲拷树重算；最近的两句是：键的组成含「这份判定自己的内容输入」（`.claude/kb/decisions/13-验证路线.md:435`，原话见 1.1）与「收不全」的那一份拒算键、照跑（`.claude/kb/decisions/13-验证路线.md:437`）：
构建时闭包读到的 `crates/` 之外的数据文件进键，拷树重算漏拷它，那一组在拷贝上判 `data-file-unreadable` 拒算，键对不上编进去的那一把。所以旧断言守的是一个测试装置的前提：
拷树重算与构建那一趟用同一套输入。它实际压的是「清单里有东西，拷贝循环才有东西可拷」，而那个东西今天没了（观察者结论拼块那一组不再经核对线程数读登记文件：切段的函数 `images_after_the_observers_cut_into_pieces_for_the_cpu` 在 `crates/singlefs-verification/src/crash_amplification.rs:31746` 取 `checking_threads()`，它由调用方调，不是这一组的入口，入口 `judge_the_images_after_the_observers_on_the_cpu` 只收切好的 `pieces`）。

### 3.3 前提「线程数只决定盘切成几段、不改判定结论」（读码）

- 切段：`crash_amplification.rs:31758`：「let piece_length = image_count.div_ceil(threads_taken.max(1)).max(1);」，`(0..image_count).step_by(piece_length)` 每段 `start..min(start+piece_length, image_count)`，段与段连着、不重、罩全，条数至少按 1 算；盘数是 0 时段表空。
- 每段一条线程：`crash_amplification.rs:31775`：「let chunk = &images[piece.clone()];」，一张盘的结论只由那一张盘算（`pool_after_the_observer(image)` 之后 `check_pool_image`），不读段号、不读别的盘；结果按段的次序 `flat_map` 收回（`crash_amplification.rs:31812`：「.flat_map(|handle| {」），次序与 `images` 相同。
- 池级 checker 里有每线程各一份的校验和记录（`crates/singlefs-checker/src/lib.rs:92`：「static REMEMBERED_CHECKSUMS_OF_THIS_THREAD: std::cell::RefCell<RememberedChecksumsOfThisThread> =」），取用要求整段逐字节相同（`crates/singlefs-checker/src/lib.rs:117`：「候选与输入整段逐字节相同才取记下的值」），
  所以记录怎么分到各线程只改命中数，不改值。
- 结论：条数只改段数与并行度，不改每张盘的结论与收回的次序——**成立（读码）**。切段有单测守着：`crash_amplification.rs:14625` 的 `images_after_the_observers_judged_on_the_cpu_are_cut_into_no_more_pieces_than_threads_were_taken`（31 张盘、17 条线程）。
- 一处边：切段函数不在这一组的闭包里（上面 3.2），它切错（漏盘、多出段）会改这一组交回的结论而键不变，只由那条单测守着。同一份登记表的文档对 GPU 那一路上卡判的函数写过同一种安排：`judging_units_entries.rs:56`：「由调用方经参数交进来，不在这一组的闭包里，改了它们拼过的块照旧复用」。
  条款 436 的「键过窄」说的是闭包里「引用的」常量、类型、宏漏收；调用方供给的参数不在「引用」里，这里按字面不违反。要不要并进键是一条设计判断，不归本腿；实现员报告已交主 agent。

### 3.4 改完之后「拷树重算漏拷 `crates/` 之外的数据文件就拒算」还有没有东西守着

- 拷贝循环在 `crash_identity.rs:2221`：「for data_file in data_files_outside_the_crates_the_build_read() {」。今天不带特性时清单为空、循环不拷任何东西；带 `gpu` 时清单是 `prune-server/src/` 的 34 份，而 `prune-server` 整个目录已经被 `crate_directories_outside_the_crates_directory` 拷过
  （测试 `crash_identity.rs:2165`：「fn the_crates_outside_the_crates_directory_include_the_prune_server() {」断言这个集合恰好是 `{prune-server}`）。所以删掉这个循环，两种编法下今天没有测试红——实现员那句**成立（读码）**。
- 还在的守：摘要表比对 `assert_the_watched_files_are_the_ones_the_build_read`（`crash_identity.rs:2241` 起，比较在 2051 至 2053：读不到的文件算「变了」），只在两条测试里调（`crash_identity.rs:2297`、`crash_identity.rs:2732`）。
  它是有条件的守：只要有一份被监视的文件拷贝里没有，那两条就红（报「树在编译后变了」，不是报拷树漏了）。所以将来清单里真出现一份 `crates/` 之外、又不在已拷目录里的数据文件、而循环被删，这两条会红；今天没有这样的文件，所以没有正例。
- 补的三条守：过滤规则（1980）、不带特性清单为空（2002）、带 `gpu` 时清单没在交给循环之前被弄空（2016，变异行 8208）。没有一条守「循环真的拷了」。
- 判：「漏拷就拒算」这件事今天由摘要表比对条件性守着（有数据文件时才有牙），正例没有。实现员写了、本腿同意，也同意他提的补法（把拷数据文件那一段拆成对来源根、目标根、清单的函数，拿临时目录钉它），归主 agent 定。

### 3.5 (b) (c) 两条钉死今天的清单：是警报还是噪声

- (b) 2002 红的条件：不带 `gpu` 编时，某一组的闭包第一次读 `crates/` 之外的数据文件，或经 path 依赖收进 `crates/` 之外的 crate 的库文件。这是一次真的输入类别变化。
  红了之后拷树重算那一套不需要改（循环按清单拷、目录按 path 依赖拷，都是自动的），要改的只有这条测试的期望值：警报的价值是「看一眼新输入类别该不该进键」，不是「有东西坏了」。低频。
- (c) 2016 红的条件：`prune-server` 的库根不再是 `src/lib.rs`，或带 `gpu` 的闭包够到了 `prune-server/src/` 之外的文件、第二个 `crates/` 之外的 crate。前一种是 prune-server 自己改布局（别的会话常动它），对验证包是纯噪声；后一种和 (b) 同类。
  实现员已把「每份都在 `prune-server/src/` 底下」写成前缀判，而不是写死 34 个文件名，prune-server 加一份源文件不会红。
- 没核的：只带 `apparatus-binaries`、不带 `gpu` 编、以及 `--all-features` 编时清单是什么（实现员报告也写了没核）。`--all-features` 下 (b) 不编、(c) 编，若多出别的特性带进来的 `crates/` 之外文件，(c) 会红。
- 多机：两条断言里只有仓内相对路径，没有主机名与地址。清单随编法变，两台编法相同（同一个特性集）时结局相同。

### 3.6 问题 3 的推翻条件

- 命令 4 里 2002 或 2016 红：3.1 的量（来自被换掉那条测试的输出）与换成的断言对不上，3.4 的「今天没有正例」要重核。
- 删掉 `crash_identity.rs:2221` 那个循环、再跑 2297 与 2732 仍绿，而清单里确有 `crates/` 之外又不在已拷目录里的文件：3.4「有条件的守」不成立。

## 问题 4：真卡测试共用实例

### 4.1 要求、代码、先例

- 要求：`crates/singlefs-background-gpu/src/background_gpu.rs:1014`：「为什么一个进程只建一个：同一个进程里新建的实例多了，Vulkan 驱动不一定还认得出独显」，同一段写了第二台上量到的现象：同一进程新建到第 12 个实例起，枚举只剩软件 Vulkan。
- 代码：`crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs:676`：「static VULKAN_INSTANCE_OF_THIS_TEST_PROCESS: std::sync::OnceLock<VulkanInstanceOfThisProcess> =」；判法抽成泛型函数（`mod.rs:702` 起，`instance_kept: &std::sync::OnceLock<Instance>` 由参数给），
  取实例一句：`mod.rs:721`：「let vulkan_instance = instance_kept.get_or_init(instance_created).clone();」；真卡那一份是薄壳 `mod.rs:747` 的 `handed_over_gpus_or_report_not_run`，它在 `mod.rs:759` 把静态项交进去：「&VULKAN_INSTANCE_OF_THIS_TEST_PROCESS,」。
- 先例：同文件 `mod.rs:664` 的 `CARDS_LEASED_ON_DEMAND_FOR_THIS_PROCESS` 也是进程级 `OnceLock`。
- 规则：`.claude/rules/code-discipline-by-review.md:54`：「不用全局可变状态；不用单例」。

### 4.2 这一处该不该有

- 一个测试二进制里的几十条用例各自独立调 `handed_over_gpus_or_report_not_run`，libtest 没有「进程级的共用参数」可传；要「一个进程只建一个」，实例只能放在进程里某个位置。
  `OnceLock` 是写一次、之后只读的全局，不是可变状态；它仍是单例，字面上碰了规则。理由是外部事实（驱动的 12 个实例上限；`background_gpu.rs:1014` 起的文档写成第二台上量到的，本腿没复量），文档里写了。
- 单例的范围压到最小：判法（`mod.rs:702`）不碰它，由参数收一个 `&OnceLock<Instance>`，不要卡的单测各自传自己的一格。静态项只出现在薄壳那一处。我判：该有，且已是最小形。
- 规则本身没有留例外口。这一处的文档（`mod.rs:672` 起）写了为什么，没写「这是规则的例外」。建议文档里加一句指到规则与理由；不是缺陷。
- 一条措辞问题：文档与新测试文件头都写「一个进程只建一个」，但产品开卡器 `GpuOpenerOfThisProcess::of_this_process()` 自己另建一个实例（`crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs:162`：「Arc::new(VulkanInstanceOfThisProcess::created_for_this_process),」），
  `crates/singlefs-background-gpu-tests/tests/mount_runtime_thread_on_a_real_gpu_compacts_while_the_foreground_fsyncs.rs:96`：「let opener = GpuOpenerOfThisProcess::of_this_process();」那条用例先取了共用的那份、丢掉手里的克隆（静态项里还留着），又让开卡器建一个：该进程里有两个实例。
  远没到 12，不碍事；说「只建一个」比事实宽。

### 4.3 三条不要卡的测试各证明了哪一半

「一个进程只建一次」拆成：A 建过一次之后再取不再建，且交回、枚举用的是同一份；B 没交卡（或没交这一台的卡）时一个都不建、不碰 nvidia-smi 与枚举。

| 测试（行） | 证明 | 怎么证明的 |
|---|---|---|
| `taking_the_gpus_twice_in_one_process_creates_the_instance_once_and_hands_over_the_same_one`（`…per_process_and_none_without_handed_over_gpus.rs:89`） | A 的全部 | 两次取卡后建了几次是 1；两次交回的替身都是编号 1 那一份；两次枚举都在那一份上；两次对上的卡都按交来的次序。`instance_kept` 用例里自备，所以证的是泛型函数，不是真静态项。 |
| `no_gpu_handed_over_returns_before_the_instance_is_created`（同文件 147） | B 的「没交卡」一支 | 变量没设：读多机配置、读 nvidia-smi、枚举都 `panic!`，没被调；建了 0 个；`instance_kept` 仍空。 |
| `gpus_handed_only_to_another_host_return_before_the_instance_is_created`（同文件 170） | B 的「只交给别的机」一支 | 交的卡全在别的机：读 nvidia-smi 与枚举都 `panic!`，没被调；建了 0 个；`instance_kept` 仍空。 |

变异行 8204 至 8206 与三条一一对应（取实例换成每次新建；没交卡也先建；交别的机也先读 nvidia-smi），原文各恰好命中一次（本腿的静态核：草稿目录里的 `check_rows.py` 对这一轮的 17 行，即 3787 与 8193 至 8208，逐行数原文在目标文件里的命中数，都是 1）。

### 4.4 没罩到的

1. 薄壳（`mod.rs:747`）里交进去的是不是那个进程级静态项：若改成每次 `OnceLock::new()` 一个局部格，三条不红。实现员自己写了这一处；我同意。
2. 薄壳里的两行打印不在任何测试里：`mod.rs:766`：「println!("NOT_RUN_NO_GPU_ADAPTER test={test_name} reason={reason}");」与 `mod.rs:771`：「"GPUS_HANDED_OVER_BY_THE_TASK_SERVER test={test_name} gpus={}",」。
   前一行被门禁与任务服务读来把「本次未跑」认出来（`.claude/gate.d/crash-point-replay.sh`、`task-server/task_server.py` 都点名 `NOT_RUN_NO_GPU_ADAPTER`）；删掉它，没交卡的用例函数交回 `None`、调用方 `return`，门禁读不到这一行，会不会把它当绿是推的，没核门禁的读法细节。
   这两行本来也没有测试（改前在不要卡测试够不到的函数里），这一轮拆函数是把它们测起来的机会，没有测。`crates/mutations.tsv` 里 `NOT_RUN_NO_GPU_ADAPTER` 只有 7939、7940 两行，是别的函数的，没有这一行的变异。
3. 并发：两条线程同时第一次取卡，`OnceLock::get_or_init` 保证只跑一次建实例的闭包（标准库保证）；没有测试，也不需要自证标准库。
4. 取实例在读 nvidia-smi 之后：没有测试钉这个先后（A 的测试里没断言 `pci_buses_reported` 先于建实例）。

### 4.5 问题 4 的推翻条件

- 命令 5 里三条任一条红：4.3 的读码不对。
- 删掉 `mod.rs:766` 的打印后没有任何测试红、而门禁认不出这个用例是「本次未跑」：4.4 第 2 条成真漏洞。

## 问题 5：八格检查单逐格回

格式：格 / 成立·不成立·推不出 / 证据。「成立」都是读码或只读命令得出的，没起任何测试；要量的在「要主 agent 跑的命令」。

### 格 1 走不到的理由二次查实：成立

- 三份实现员报告的「走不到的理由」都写无。本腿对四份文件的 `git diff -U0` 里加的行逐行 grep `todo!|unreachable!|unimplemented!|.expect(|panic!|assert!|assert_eq!|.unwrap()`：
  `judging_units_entries.rs` 一处也没有；`crash_identity.rs` 里全部命中的行号都大于 1468，在 `crates/singlefs-verification/src/crash_identity.rs:929` 的 `#[cfg(test)]` 模块之内（`crash_identity.rs:930`：「mod tests {」）；`mod.rs` 的 711、718 两处 `unwrap_or_else(|reason| panic!(…))` 是测试支持模块里「判红」的出口，与改前同形。
- `units_changed_between`（`crash_identity.rs:483` 至 506）与两个私有函数（510 至 560）里没有 `expect`、`unwrap`、`unreachable!`。唯一的下标是 `crash_identity.rs:532`：「is_now_unit_paired[position] = true;」，`position` 来自 `crash_identity.rs:519` 起对同一个 `now_unpaired` 的 `enumerate`，向量按 `crash_identity.rs:525`：「let mut is_now_unit_paired = vec![false; now_unpaired.len()];」开，下标不越界。
- 新测试里的 `expect`：`crash_identity.rs:1468` 等五处 `.expect("本 crate 在仓根的 crates/ 下")` 依赖 `CARGO_MANIFEST_DIR` 是 `<仓根>/crates/<包>`，仓副本与主工作区同形；`crash_identity.rs:1489` 的 `.expect("rsplit 至少交一段")` 依赖 `str::rsplit` 至少交一段（标准库保证）；
  `function_text_returns_a_value`（1236）里的 `parenthesis_depth -= 1` 在 1250：第一个 `(` 先加到 1，回到 0 就返回，不会减到负。`source_text_of_the_composition_line`（1212）里 `last_line + 1 - first_line.min(last_line + 1)` 不会下溢。

### 格 2 类型与名字：成立（一处小意见）

- 新名字都在报的行上：两个常量（`judging_units_entries.rs:1178`、`:1196`）、`crash_identity.rs:510`、`crash_identity.rs:549`、`crash_identity.rs:1954`、`mod.rs:676`、`mod.rs:702`；新测试文件里的 `InstanceStandInNumberedByItsCreation` 与三条测试名（89、147、170）。名字说内容，没有序数、位置代称与缩写。
- 单元名仍是字符串键（`JudgingUnitFingerprint.unit: String`，`crash_identity.rs:408`），本轮没改这一点，也没有新立的字符串键。
- 小意见：`data_files_outside_the_crates_the_build_read`（`crash_identity.rs:1970`）这个旧名留着，但清单里今天带 `gpu` 时全是 `prune-server/src/*.rs` 的源文件，不是数据文件。文档（1966 起）写清了，函数名说窄了。不碍事。
- 命名检查（门禁 `code-source-discipline.sh` 四格与 naming-lint）在快照上过了是主 agent 的观测，本腿没复跑。

### 格 3 不兼容旧代码与旧格式：成立

- 对三份被改的源文件的 `git diff -U0` 加的行与新测试文件 grep `legacy|兼容|迁移|旧版|旧库|旧镜像|compat|migrat|deprecated`，无命中。
- GPU 判器那一组的键变一次，不迁移：键只由收出来的闭包算（1.2），旧库里的块键对不上就现判。没有认旧键的分支。
- 唯一碰「旧」的是 `crates/mutations.tsv` 整行替换那一行（3787），换的是同名行的第五、六段（测试名），不是为旧数据留的放宽。

### 格 4 不新建判定库：成立

- 新测试里拷活树的三条（`an_edit_of_the_listed_gpu_*` 与 2016 一族）经 `RepositoryCopy::new`（`crash_identity.rs:2188`：「fn new(label: &str) -> Self {」）建 `ExclusiveTemporaryDirectory`，`Drop` 时 `remove_dir_all`（`crates/singlefs-verification/src/crash_library.rs:253`：「impl Drop for ExclusiveTemporaryDirectory {」）。
  对 `crash_identity.rs` 的加行 grep `CrashLibrary|verdict_store|put(|store_`，只命中一个测试名里的 `prune_server` 字样；没有写判定的调用。新测试文件不碰库。

### 格 5 所有任务多机可用：成立（一处没核）

- 新测试里没有主机名、地址与本机路径：真卡新测试文件用文档地址段 `192.0.2.1`、`192.0.2.2`（文件里 `THIS_HOST`、`OTHER_HOST` 两个常量），总线号现造。`crash_identity.rs:2002`、`2016` 两条断言只有仓内相对路径（`prune-server/src/`、`prune-server/src/lib.rs`）。
- 清单随特性集变，两台编法相同时结局相同；这三条不读本机独有状态。拷树用 `std::env::temp_dir()`，与同文件已有的拷树测试同法。
- 没核：两台是不是用同一个特性集编这个测试二进制（`--all-features` 或只带 `apparatus-binaries` 时 2016 的「都在 `prune-server/src/` 底下」会不会红，见 3.5）。推不出：缺一次各编法下的清单。

### 格 6 测试优先多线程：成立，但并行时的内存推不出

- 加行里没有写死线程数（grep `thread|THREAD`，只命中合成清单里的路径 `.claude/heavy-thread-default` 与 `heavy_threads.rs`）。五条挂 `gpu` 的读活树测试都用 `workspace_loading_threads_from_the_test_process_share()` 取线程（`crash_identity.rs:1163`：「let (threads_taken, threads) = workspace_loading_threads_from_the_test_process_share();」），三条拷树的各拷各的、不共用状态。
- 推不出：这些测试各自装一遍整个工作区（`gpu_judge_composition_under` 每次调用装一次：1471 与 1497 两次、1608 与 1618 两次、1358 与 1360 两次），libtest 默认并行起来时的内存峰值没量；任务服务按内存上限排，超了由服务放大重排。缺：装一次工作区的内存与时间，命令 1 的日志有耗时但没有内存。

### 格 7 带 --item 的命令与项：成立

- 这一轮的 17 行变异（3787、8193 至 8208）六段齐（`check_rows.py` 输出六个字段），名字在表里各恰好一行（`cut -f1 | grep -cxF` 对 17 个名字都是 1），所以 `--item <名字>` 点得到、不会撞名。
- 点名的测试都是 lib 测试或新测试文件，不走崩溃枚举：`crash_identity.rs` 里没有 `SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO`、`SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO`（`grep -n` 无输出），新测试文件里没有 `enumerate_crash_states`（`grep -c` 为 0）。所以验证段点名这些行不需要缩域或全域理由；这是读码，闸怎么判以闸为准（命令 6 交的时候闸会说）。
- 新测试文件在 `crates/singlefs-background-gpu-tests/Cargo.toml` 里没有 `required-features`（该文件里的 `[[test]]` 项都是真卡那几个目标），不挂特性：变异行 8204 至 8206 的参数里也没有 `--features`，对得上。

### 格 8 新崩溃流多罩了什么：成立（本轮没有这一类）

- 没有新加崩溃点重放流或崩溃枚举用例：新测试文件与五条挂 `gpu` 的测试都不调 `enumerate_crash_states`，`crates/singlefs-verification/src/crash_identity.rs` 里新测试只装工作区、算键、拷树。

## 要主 agent 跑的命令

每条在仓根 `/home/fy5090/code/singlefs` 起（`--cwd` 写在命令里）。本腿没写模型，没有 `rerun.sh`；下面是只读判据的复核，不是模型复跑。

1. 两批登记的十条 gpu 特性库测试（五条新的 + 按整份登记判的五条，都是读活树的，不走崩溃枚举）
   - 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 带 gpu 特性的 lib 里 10 条测试; 范围=10 个测试; 为什么=核两批登记的五条新测试与按整份登记判的五条旧测试在主工作区此刻的树上都绿' task-server/task submit --cwd /home/fy5090/code/singlefs -- cargo test -p singlefs-verification --release --features gpu --lib -- crash_identity::tests::no_unit_the_gpu_judge_group_keeps_reads_a_value_returned_by_a_listed_unit_that_only_prints_the_gpu_lifecycle_line crash_identity::tests::an_edit_of_the_listed_gpu_lifecycle_trace_keeps_the_gpu_judge_key crash_identity::tests::an_edit_of_the_listed_gpu_work_trace_keeps_the_gpu_judge_key crash_identity::tests::an_edit_of_the_listed_gpu_lifecycle_line_keeps_the_gpu_judge_key crash_identity::tests::the_units_listed_as_reached_only_through_the_units_that_print_the_gpu_lifecycle_line_are_reached_through_them_and_by_no_unit_the_group_keeps crash_identity::tests::not_following_the_listed_units_drops_only_the_listed_ones crash_identity::tests::every_listed_unit_of_the_memory_ledger_names_exactly_one_unit_of_the_workspace crash_identity::tests::the_gpu_judge_group_reaches_no_unit_of_the_memory_ledger_beyond_the_listed_ones crash_identity::tests::no_unit_the_gpu_judge_group_keeps_names_a_wrapper_type_it_does_not_follow crash_identity::tests::an_edit_of_a_listed_unit_of_the_memory_ledger_keeps_the_gpu_judge_key --exact`
   - 在哪起：仓根。看：末行 `TASK_SERVER_RESULT` 的 `exit=0`；日志里 `test result: ok. 10 passed`。红了点名的是 1.8 至 1.9 的哪一环（`not_following_…` 的「少掉而没登记的」「登记了而没少掉的」两段抄下来）。
2. `unit-graph why` 核登记文档里两条进来的边（头一条边，只当旁证；「只经」靠命令 1 里的 1587）。两条命令，`--unit` 各给一个子串
   - 命令 2a：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=单元图命令行 why 查 GPU 判器那一组闭包里宏生成成员表那几个单元的头一条进来的边; 范围=1 条命令; 为什么=核登记文档里两条进来的边是同名撞上的读码' task-server/task submit --cwd /home/fy5090/code/singlefs -- cargo run -q --release -p singlefs-unit-graph --bin unit-graph -- why /home/fy5090/code/singlefs singlefs_verification::crash_judge_gpu::JudgeKernelSources::of_the_judge singlefs_verification::crash_judge_gpu::JudgeKernelSources::of_the_judge_given_the_container_capacity_knobs singlefs_verification::crash_judge_dispatch::GpuCrashJudge::on_card_with_the_given_dispatch_width singlefs_verification::crash_judge_dispatch::GpuCrashJudge::judge_one_dispatch singlefs_verification::crash_judge_gpu::gpu_verdict_items singlefs_verification::crash_judge_gpu::recovery_words_of_the_gpu_verdict singlefs_verification::crash_judge_gpu::cross_check_words_of_gpu_verdict singlefs_verification::crash_judge_gpu::scan_rebuild_outcome_of_the_gpu_verdict singlefs_verification::crash_judge_gpu::self_verified_system_configuration_slots_of_each_device_of_the_gpu_verdict singlefs_verification::crash_judge_conclusions::StateCountsOfABlock::count_the_gpu_bits singlefs_verification::crash_judge_conclusions::StateCountsOfABlock::into_count_vector singlefs_verification::crash_judge_conclusions::encoded_verdict_of_the_red_items singlefs_verification::crash_judge_tables::build_judge_tables_against_the_version_table_with_slices singlefs_verification::crash_judge_tables::build_judge_tables_for_the_given_persisted_sets_broken_at_the_seams_with_their_claimed_versions singlefs_verification::crash_judge_tables::JudgeTables::words singlefs_verification::crash_judge_dispatch::PackedJudgeTables::of --target lib --cfg 'feature="gpu"' --toolchain-of-the-build release --lock-file /home/fy5090/code/singlefs/Cargo.lock --judgment-kind product-behavior --unit 'enum_listing_every_member'`
   - 命令 2b：同 2a，只把最后的 `--unit 'enum_listing_every_member'` 换成 `--unit 'RecordedOperationKind'`（范围声明的「跑什么」同样改成「录制流操作种类的 impl 头与 name」）。
   - 在哪起：仓根。看：`exit=0`，标准输出里 `why` 行的 `via` 链。2a 应有 `lib.rs` 的 `…enum_listing_every_member!@78` 一行，来处是 `gpu_work_trace`，规则 `R-裸名`、名字 `Write`；
     2b 应有 `<impl RecordedOperationKind>::name`，规则 `R-隐式（条目级宏调用生成的类型被点名）` 或 `R-方法`。头一条边与读码不同，不推翻「只经」（命令 1 管那一句），只改 1.7 的某一环。特性只给了 `gpu`，与构建脚本逐字相同的取法在 `crates/singlefs-verification/build.rs`，不同的以它为准。
3. `units_changed_between` 的两条不挂特性的测试
   - 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的 lib 里 2 条测试; 范围=2 个测试; 为什么=核两步配对在主工作区此刻的树上的只挪行、挪了行又改、删、加四种输入' task-server/task submit --cwd /home/fy5090/code/singlefs -- cargo test -p singlefs-verification --lib -- crash_identity::tests::units_named_by_their_line_that_only_moved_are_not_named_as_changed_and_edited_ones_are crash_identity::tests::the_composition_reads_back_and_the_red_sentence_names_the_changed_units --exact`
   - 在哪起：仓根。看：`exit=0`；`test result: ok. 2 passed`。
4. 数据文件清单三条（不带特性两条、带 gpu 两条，纯函数那一条各编法都编得到）
   - 命令 4a：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的 lib 里 2 条测试; 范围=2 个测试; 为什么=核不带 gpu 特性时拷树重算挑 crates 之外文件的纯函数测试与清单为空那条在主工作区此刻的树上绿' task-server/task submit --cwd /home/fy5090/code/singlefs -- cargo test -p singlefs-verification --lib -- crash_identity::tests::only_the_watched_files_outside_the_crates_and_the_root_manifests_are_copied_beside_the_crates crash_identity::tests::the_build_without_the_gpu_feature_reads_no_file_outside_the_crates --exact`
   - 命令 4b：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 带 gpu 特性的 lib 里 2 条测试; 范围=2 个测试; 为什么=核带 gpu 特性时清单里有 prune-server 库文件那条在主工作区此刻的树上绿' task-server/task submit --cwd /home/fy5090/code/singlefs -- cargo test -p singlefs-verification --release --features gpu --lib -- crash_identity::tests::only_the_watched_files_outside_the_crates_and_the_root_manifests_are_copied_beside_the_crates crash_identity::tests::the_build_with_the_gpu_feature_reads_the_prune_server_library_files_outside_the_crates --exact`
   - 在哪起：仓根。看：两条各 `exit=0`、`test result: ok. 2 passed`。红了抄断言里打出的清单（3.1 里量过的两个形态：`[]` 与 `prune-server/src/` 底下 34 份）。
5. 真卡共用实例的三条测试（不要卡、不挂特性）
   - 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-background-gpu-tests 的测试目标 real_gpu_tests_create_one_vulkan_instance_per_process_and_none_without_handed_over_gpus 里的 3 条测试; 范围=3 个测试; 为什么=核进程里只建一次实例与没交卡、交的卡里没有这一台时一个实例都不建' task-server/task submit --cwd /home/fy5090/code/singlefs -- cargo test -p singlefs-background-gpu-tests --test real_gpu_tests_create_one_vulkan_instance_per_process_and_none_without_handed_over_gpus -- taking_the_gpus_twice_in_one_process_creates_the_instance_once_and_hands_over_the_same_one no_gpu_handed_over_returns_before_the_instance_is_created gpus_handed_only_to_another_host_return_before_the_instance_is_created --exact`
   - 在哪起：仓根。看：`exit=0`；`test result: ok. 3 passed`。

## 建议、不采纳（不归本腿写，列出来交主 agent 判）

- 第二步配对的指纹条件补一行变异：对 `crash_identity.rs:521` 与 `:528` 两处同时把桶键里的指纹拿掉（只改一处会让所有配对落空，红的是别的断言），点 `units_named_by_their_line_that_only_moved_are_not_named_as_changed_and_edited_ones_are`，期望红在 3167 那一句。
- 薄壳 `handed_over_gpus_or_report_not_run` 里两行打印（`mod.rs:766`、`mod.rs:771`）没有测试也没有变异行；要测得先把它们拆成不碰卡、可传输出的函数，再补一条测试与一行变异。
- 清单、红句：`changed_units_sentence` 空表那一句说「是空表」，键变而清单没变时说错原因（2.3）。
- 登记文档的现状句：条款 438 点名两个常量，今天四个（1.4）。写回员的活。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 问题 1：两批登记对条款 | 一致（两处条款比代码窄） | 机制与五条测试都有；条款点名两个常量今天四个；五条里「不带登记收时够到的恰好等于登记的」对新两批由更强的两条替代，「改一个登记了的单元」对四个只经它们够到的单元同义反复 |
| 三个打日志的单元不改判定字 | 成立（读码） | 只有 `gpu_lifecycle_line` 交值，只被 `gpu_work_trace` 调；副作用只有标准错误一行与函数内时钟起点；读日志的门禁与变异判法按前缀滤掉；panic 路径只核了一处实参 |
| 四个单元只经那三个够到 | 成立（读码） | 两条边是同名撞上（`Write`、`name`），每一环对到行；「只经」靠 1587 后半段，`why` 只能旁证头一条边 |
| 登记会不会让键过窄 | 不窄 | 1351 守「少掉的恰好是登记的」，1463 守不读值，1587 守无边连到四个 |
| 带行号的登记名 | 成立 | 挪行不先让键变宽变窄；同名宏调用落到 78 行才可能窄，1351 与 1587 同趟红；代价是改 `lib.rs` 前 77 行要同步改登记 |
| 问题 2：两步配对 | 成立 | 不漏报；只有按行号起名的三种单元可能点错名字；键变而清单没变时红句说错原因（先前就有） |
| 问题 3：三条测试 | 钉住了过滤规则与今天两种编法的清单 | 前提「线程数不改结论」成立（读码）；「漏拷就拒算」今天无正例，只有有条件的摘要比对守着 |
| 问题 3：(b)(c) 是警报还是噪声 | 低频警报，(c) 的 `lib.rs` 一半是噪声 | 拷树机制自动处理新输入，红的作用是让人看一眼；prune-server 改布局会让 (c) 红 |
| 问题 4：该不该有进程级 `OnceLock` | 该有，已是最小形 | 规则字面禁单例，理由是 Vulkan 驱动的实例上限；判法不碰它，静态项只在薄壳一处 |
| 问题 4：三条测试 | 证了「建一次、同一份」与「没交卡不建」两支 | 没罩：薄壳交进去的是不是静态项、薄壳里两行打印、取实例与读 nvidia-smi 的先后；一条用例里仍有两个实例的措辞问题 |
| 问题 5 格 1、2、3、4、7、8 | 成立 | 读码与只读 grep，证据在各格 |
| 问题 5 格 5 | 成立（一处没核） | 缺各编法下的清单 |
| 问题 5 格 6 | 成立；并行内存推不出 | 缺装一次工作区的内存 |

## 没做什么

- 不判别的腿的格：不替主 agent 采纳或出判决。没往统一任务服务交任务，判决要引的数没复跑（命令在上一节；命令 1 至 5 都待主 agent 跑）。
- 没起 `cargo`、没建仓副本与编译目录、没写模型；草稿目录 `~/.local/share/singlefs/work/code-r1-judging-closure-forward` 里只有 `check_rows.py`（读 `crates/mutations.tsv` 数原文命中次数的只读脚本）、`log19725.txt`（合入后验证任务日志的只读拷贝）与 `cite.txt`（引文核对的输出），共不到 100K，没有副本、编译目录与库拷贝，不用删。
- 没读三份实现员报告以外别的腿的文件；没读禁读清单里的文件。
- 没扫全仓读标准错误的所有读者（1.6）；没逐个读 `gpu_work_trace` 全部二十来处调用实参（1.5）；没量 `--all-features` 与只带 `apparatus-binaries` 编时的清单（3.5）；没量并行内存（格 6）。
- 没核 `.claude/gate.d/crash-point-replay.sh` 读 `NOT_RUN_NO_GPU_ADAPTER` 那一行的细节（4.4 第 2 条是推的）。

## 交回前的引文核对

`python3 scripts/agents/cite-check.py research/prompts/judging-closure-and-real-card-instance-code-r1-sonnet-output.md`，退 0，末行：核了 64 处引文、61 处裸行号，对上 53 处，没判 72 处（其中裸行号没有引文可比 38 处；没判的主要是只写 `mod.rs`、`gpu_unit_checks.rs` 这类仓里同名文件多、认不出是哪个的路径，这些行本腿另用 `grep -nF` 逐处核过）。
