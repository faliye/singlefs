# tooling-broad-mutation-prune-r2 云端辩方腿（Sonnet）：V1、V3（2026-10-01）

立场：替被第一轮判决收严的一方（P1–P5 与 B1–B8 收严之前的写法）辩护。读了：第一轮判决、两份第一轮腿报告、第二轮正文（含第三节第二版）、库目录（只读）、`git show HEAD:` 的源码（HEAD 是 fcc4d340）。没读禁读清单。没起驱动、没编译、没跑任何测试。
真 crates 上的数都是对 HEAD 源码与库目录产物的静态扫描和算术，脚本与原样输出在 `research/prompts/tooling-broad-mutation-prune-r2-sonnet-model/`（`bash run_all.sh` 复现，只读 git 与库目录；`out/` 是这一次的输出）。覆盖遍的插桩倍数没量。
「关联」「条数」都指这一版清单的 5 091 条；耗时单价用这一版已判 302 条的实测（`out/res.txt`、`out/v3.txt`），是本机数、被判的那五个文件之外外推的是量级。

## 先看一条：第二版字面读法下，函数粒度对「抓到」的条不起作用

第二版 P1 写被改函数的源码 sha256，P2 的抓到条写「T 所在测试文件、T 执行到的源文件、常量闭包（P3）、宏与 derive 的定义文件都没变，才复用」（`research/prompts/_tooling-broad-mutation-prune-r2-body.md:36`）。T 能抓到一条变异，除常量初始化式里的 170 条外，必然执行过被改的那一行，所以被改文件本身在「T 执行到的源文件」里：被改文件里别的函数一变，这个文件的 sha 就变，这一格判重判。
再加：241 条抓到里 163 条（67.6%）的 T 是被改文件自带的 `mod tests` 里的测试，「T 所在测试文件」也是被改文件（`research/prompts/tooling-broad-mutation-prune-r2-sonnet-model/out/t_inline.txt:2` 「share of caught whose T file == mutated file: 67.6%」）。
要让函数粒度起作用，被改文件那一份的「执行到」也得按函数记；这是对用户定的「关联文件照旧按文件」（`research/prompts/_tooling-broad-mutation-prune-r2-body.md:8`）又改一处，我不替用户定，写进 V3：两种读法都算（读法 A 只按被改函数，读法 B 按字面、被改文件整份），差 70 倍。
推翻它的现象：第二版的作者说明「T 执行到的源文件」不含被改文件本身、或同一文件里 T 的函数与被改函数都按函数记。

## 一、判定一览（判决第二节 10 行；判决文件 `research/prompts/tooling-broad-mutation-prune-r1-main-verification.md` 第 24–33 行）

| # | 判决条目 | 辩方判定 | 证据 | 什么现象会推翻它 |
|---|---|---|---|---|
| 1 | 粒度：按文件定键省不下多少 | 站得住；同样打中别的臂，且别的臂更重 | 我自己复算：14 个动了清单文件的提交（判决的 15 个里首个 53c9f384 是整体导入、没有旧版可比，剔除）被改文件名下条数中位 3119 条，「n=14 fl_own median 3119 (61.3%)」（`research/prompts/tooling-broad-mutation-prune-r2-sonnet-model/out/commits.txt:17`）；判决的 62.2% 是含导入的 15 个的中位。按 crate 为键，这 14 个提交都是 100%（它们就是因为动了 core 或 checker 的 src 选进来的）。按 `.d` 为键（读法：每个测试二进制一份，键是它自己与依赖 crate 的源文件集合）：core 的库测试二进制的 `.d` 列 core 的 27 个源文件，core 一变这一档（241 条抓到里 185 条）全重判；仓里找不到用 `.d` 做复用键的先例 | 换一段以 harness 测试改动为主的提交史，文件名下条数中位低于 20%；或量一份真的 `.d`，里面不含依赖 crate 的源文件也不并依赖指纹 |
| 2 | 超时：整段总时限在负载下把抓到判成超时 | 站得住；同样打中别的臂（与键无关） | 第 556 条主 agent 已坐实。另看 7 条超时的日志：454、586、588 在 SIGTERM 时各有 83、419、435 条没来得及跑（「83/776 tests were not run due to signal」，`/home/fy5090/.local/share/singlefs/broad-mutation/versions/19c45e021b5638f68eac7c72dcc317a4574bdaa2b9f33a4a8855284827265cd4/logs/454.one-test.log:1535`），说明挂住的不只一条测试；这与键怎么定无关，crate 键与 `.d` 键同样中 | 在空闲机器上把这 7 条各重判一次，有落在限时之内的 |
| 3 | 常量（S1、S2，3.1、3.2） | 站得住，射程窄；别的臂不中 | S1 前提（被改文件不引 format 的常量）只在 5 个 core 文件 151 条（`research/prompts/tooling-broad-mutation-prune-r1-opus-output.md:247` 「不引用 singlefs-format 的 5 个 core 文件」；我独立数出同一组，`out/fmtrefs.txt:3`）。S2（第二跳落在 P3 并进的文件之外）：只按常量与 static 名字算 0 个文件（`out/s2.txt:4`），把 const fn 名字也算上 1 个文件 86 条（`out/s2b.txt:4`）。常量初始化式里的变异我独立数出 170 条（`out/parse_list.txt:2`），与判决引的同数。只在常量上下文被叫的 const fn：2 条变异，另 1 条在语法上找不到调用（`out/constonly.txt:2`）。crate 键与 `.d` 键按源文件整体变没变判，不存在这几处漏判 | 改用类型解析后 S2 的文件数不是 0；或 const fn 的名字匹配漏掉了只在常量里被叫的函数 |
| 4 | 覆盖取哪一版（S4a、S4b） | 够不着今天的 crates；但便宜，该留；别的臂不中 | 前提是同一个类型的固有方法与 trait 方法同名：仓里 4 个自有 trait 下 0 对（`out/s4.txt:2`），名字匹配、不是类型解析。B4 只多存一份覆盖 | 类型解析后有一对；或以后写出一对 |
| 5 | 导出树之外的输入 | 站得住；同样打中别的臂 | 测试读导出树外的文件（`crates/singlefs-checker-tier/src/crash_identity.rs:1324` 「.claude/gate.d/stage-inputs.tsv」），基线退 101（`/home/fy5090/.local/share/singlefs/broad-mutation/versions/19c45e021b5638f68eac7c72dcc317a4574bdaa2b9f33a4a8855284827265cd4/baseline.tsv:8` 「exit-101」）；crate 键、`.d` 键、覆盖三种都看不到「运行时读了哪个文件」 | 把该文件放进导出树后基线仍红 |
| 6 | 键缺项 | 站得住（采）；同样打中别的臂 | nextest、cargo-mutants、llvm-tools 的版本、排除表、限时参数、线程数，任何键都缺；线程数入键见下面 B6 | 只改排除表而源码不变，旧结论仍对 |
| 7 | 实现层（`#[path]` 名字、profraw） | 站得住；只打中靠覆盖的 P2，不打中别的臂 | 这两样都是插桩带来的；crate 键与 `.d` 键不涉及（`.d` 里路径的写法我没量） | `.d` 里的路径同样带 `..` 与绝对前缀而键又要用它 |
| 8 | 宏、derive、文档测试 | 宏：够不着今天（零处）；derive：推的，一跳代价不大；文档测试：站得住 | derive 类型 305 个落在 32 个源文件里的 28 个（`out/derive.txt:1`）；被改文件一跳引用到的带 derive 的类型定义文件中位 4 个、按条数加权均值 8.1 个（`research/prompts/tooling-broad-mutation-prune-r2-sonnet-model/out/derive.txt:2`–`:3`）；文档测试那一步基线 1.5 秒，一律重判不贵 | 起点换成「T 执行到的全部文件」后，derive 一项的并集接近整个包 |
| 9 | 没打中的（泛型、内联、闭包、共享状态等） | 站得住 | 判决没有拿一次取样的「没打中」当否定，只进 P4 样本，处理对 | 无新证据，不挑 |
| 10 | P4 自证在小模型 crate 上造 | 站得住，有限度 | 小模型证规则的判断逻辑，证不了真 crates 的工具行为（名字归一、profraw 落点）；这两样要各有自己的样本（B7 已含） | 小模型样本在真 crates 的工具链上判法不同 |

## 二、B1–B8 有没有收过头

| 改法 | 判定 | 要点 | 推翻它的现象 |
|---|---|---|---|
| B1 单条限时 | 方向翻了：假红换成假绿的风险；交用户定倍数与复核 | 见第三节 A。整段限时下超时记红是保守方向，单条限时下超时记「抓到」是冒进方向 | 对被记成「抓到」的超时条各单跑一次那条测试，全部真红 |
| B2 常量初始化式一律跑 | 一半过头：把「不靠覆盖判」读成了「不复用」 | 见第三节 B。170 条（3.3%）加 2 条永远重判，每次提交 0.9–3.0 小时 | 常量初始化式里抓到的条，T 记录不全 |
| B3 常量闭包 | 按常量项展开不收过头，按文件展开等于整个包 | 见第三节 C | 常量项级展开在真 crates 上漏了一跳 |
| B4 两版覆盖并集 | 没过头 | 只多存一份覆盖，存多大没量 | 并集在真 crates 上的体积不可接受 |
| B5 加第二版 P1 的全局项 | 过头候选：读数据文件的那几条测试用全局项管 | 见第三节 D：14 个提交里 5 个整表重判 | 把数据文件并进读它的测试之后，漏判一条 |
| B6 全局键补齐 | 线程数、限时参数入键是过头候选 | 线程数只改测试确定性，没有证据结局随它变，也没有反例；换机器、调线程上限（`.claude/agent-common.md:58`）就整版重判 | 同一变异同一树、线程数不同而结局不同 |
| B7 名字归一 | 没过头 | 实现层必需 | — |
| B8 宏、derive、文档测试 | 没过头 | 宏零处；derive 一跳中位 4 个文件；文档测试 1.5 秒 | 同第 8 行 |

## 三、V1 的展开（B1、B2、B3、B5 过头与否的数）

### A. B1 单条限时：翻过去的方向

- 方向：第一轮判决要修的是整段总时限把「抓到」判成超时（假红）；B1 加用户定的「超时就是那条测试没过」之后，错的方向变成负载或变异本身把一条本来会通过的测试拖过限时、被记成「抓到」（假绿）。原写法（`research/prompts/_tooling-broad-mutation-prune-r1-body.md:59` 「有任何一条没红、超时或只过了第一段就红」）在这件事上是保守方向。
- 这一版库里 249 份第一段日志里，通过的测试用时与基线比：基线 5 秒以上的 2 800 对，「share of passing pairs above 10x baseline: 20.25%」（`research/prompts/tooling-broad-mutation-prune-r2-sonnet-model/out/ratios2.txt:8`）。同一条基线 7.5 秒的测试在 53 份日志里用时 4.4 到 744 秒（`out/ratios2.txt:17`）。
- 这些慢是负载还是变异本身，日志分不开：库里序号 584、586、588 是 allocator.rs 同一行 `PlacementOnDevice::covers` 的三条变异，都记超时（`/home/fy5090/.local/share/singlefs/broad-mutation/versions/19c45e021b5638f68eac7c72dcc317a4574bdaa2b9f33a4a8855284827265cd4/results.tsv:302` 「PlacementOnDevice::covers」），日志里 419、435 条没跑（见第一节第 2 行），像是变异把热路径拖慢；而第 574 条是别的函数（`make_room_for_record_on_device`）也超时。
- 按「基线耗时乘倍数再加下限」代进去：「limit = 10x baseline + 300s: passing pairs over limit 204 (0.68%) in 8 logs」（`research/prompts/tooling-broad-mutation-prune-r2-sonnet-model/out/grid.txt:7`），那 8 份里 5 份是库里记超时的、3 份是抓到的；「limit 10x+60s: logs with a passing test over limit by outcome: {'caught': 12, 'timeout': 7}」（`research/prompts/tooling-broad-mutation-prune-r2-sonnet-model/out/grid.txt:13`）。只过第一段的 15 条在这几组限时下没有任何一份有通过的测试越限（`research/prompts/tooling-broad-mutation-prune-r2-sonnet-model/out/grid.txt:11`–`:15`）。所以这一版数据里 B1 改变结局的恰好是那 7 条超时、都改成「抓到」；其中第 556 条、第 102 条主 agent 与攻方各自坐实是真抓到，其余 5 条没人单测过。
- 22 份「看起来平静」的日志（每份中位用时不超过基线 2 倍）里，单条测试最高倍数是 0.6 到 63.6 倍（`research/prompts/tooling-broad-mutation-prune-r2-sonnet-model/out/calm.txt:2`），其中 4 份在 6 倍以上；这是 B1 假绿的可能来源，没有直接反例。
- 建议（我提的，被攻过零轮）：记成「抓到」的超时条，各把那一条测试在变异树上单跑一次、限时放宽，仍超时或红才算抓到。代价：这一版 7 条里每条多一次单测，第 556 条那样 147 秒量级。
- 推翻它的现象：5 条没坐实的超时条单跑后都红，且倍数 10 倍加 300 秒下再抽几版日志，通过的测试越限只出现在后来记抓到的条里。

### B. B2：「不靠覆盖判」不等于「不复用」

- 原写法 P2：「一条测试都没覆盖到那一行的变异，直接记没红」（`research/prompts/_tooling-broad-mutation-prune-r1-body.md:48`）。这一句对常量初始化式里的变异与只在常量上下文被叫的 const fn 判错，判决要它们不靠覆盖判，我同意；第二版写成「不靠覆盖判，一律跑（B2）」（`research/prompts/_tooling-broad-mutation-prune-r2-body.md:38`）。
- 「一律跑」要是每次提交都跑，就是 170 条加 2 条永远不复用。数：每次提交 0.9–3.0 小时（「B2: 170+2 mutants always run: 2.98 h full / 0.94 h T-first per commit」，`research/prompts/tooling-broad-mutation-prune-r2-sonnet-model/out/v3.txt:12`；单价见第七节）。
- 但「不靠覆盖判」只是不拿零覆盖当红。这一版已判的 4 条常量初始化式变异里，抓到的 3 条都在库里记了抓到它的测试：序号 5（`/home/fy5090/.local/share/singlefs/broad-mutation/versions/19c45e021b5638f68eac7c72dcc317a4574bdaa2b9f33a4a8855284827265cd4/results.tsv:8` 「journal_sequence_numbers_order_by_instance_then_counter」）、序号 13（同文件第 16 行 「publish_reading_reserves_the_switch_share」）、序号 14（第 17 行，同一条测试）。有 T 记录，抓到条就能按 P2 抓到那一行复用（T 所在文件、T 执行到的文件、常量闭包没变），用不着覆盖在初始化式那一行有区域。
- 所以 B2 该拆成两句：零覆盖不判红（要）；抓到而有 T 记录的照常复用（不该被「一律跑」吃掉）。只有没抓到、没 T 记录的（4 条里的第 118 条是只过第一段）才真正每次都跑。
- 推翻它的现象：常量初始化式里抓到的条，T 在初始化式的值变后换成另一条（T 记录不稳）；或 T 的执行集合漏掉了常量定义文件而 P3 闭包也没补上。

### C. B3：闭包按常量项展开，真 crates 上不扩到整个包；按文件展开就扩到

- 范围：被扫的 32 个源文件（core 27、checker 4、format 1）里，常量与 static 的定义 339 个（336 个 `const`、3 个 `static`）、const fn 51 个（`out/constgraph.txt:2`）。
- 从单个被改文件出发、按常量项展开（只读常量初始化式与 const fn 的函数体里引到的名字）：闭包文件数中位 4、最大 14，共 32 个（`research/prompts/tooling-broad-mutation-prune-r2-sonnet-model/out/constgraph.txt:4` 「mode sc item-level closure size over 1 start file: median 4.0 min 1 max 14 of 32」）；按文件展开（把被引到的常量所在文件整份再读一遍）：中位 20、最大 21（`research/prompts/tooling-broad-mutation-prune-r2-sonnet-model/out/constgraph.txt:6` 「mode sc file-level closure size over 1 start file: median 20.0 min 1 max 21 of 32」）。B3 写「沿常量、`static`、const fn 的引用逐跳展开」，没说按项还是按文件；按文件实现就是整个包。
- 起点取「被改文件与测试执行到的文件」时，执行到的文件已经很多，闭包比起点多的只是常量定义文件；起点取全部 core 文件，闭包 31 或 32 个（`out/constgraph.txt:12`、`:13`），所以结论是「看展开的粒度」，不是「看起点」。
- 代价集中在 format 的 `lib.rs`（89 个常量、整个包里最大的常量文件）：25 个被改文件直接引它，占 4940 条（「directly referencing format consts 25 mutants in them 4940 of 5091」，`out/fmtrefs.txt:2`）。它在 14 个提交里变了 8 个（「format lib.rs changed in 8 of 14」，`research/prompts/tooling-broad-mutation-prune-r2-sonnet-model/out/globals.txt:16`）。按文件并进关联，这 8 个提交各让 4940 条（97%）重判；按常量项只让引到被改常量的那些文件的条重判，这 8 个提交里 0%、0%、7.7%、33.2%、34.6%、35.9%、36.1%、85.5%（`out/fmtitem.txt:4`、`:6` 等，逐行）。
- 所以 B3 本身不是过头；过头在「关联文件按文件」这个粒度上，对常量项级的键我不替用户定，交用户（第六节）。原 P3 的写法「被改文件里引用到的常量，它的定义文件并进关联文件」对这 25 个文件已经是整份并进 format，所以 B3 比原写法多出来的主要是那 5 个文件的 151 条。
- 推翻它的现象：常量项级展开在真 crates 上漏了一跳（我的展开是名字匹配，不是类型解析）；或 format 的 lib.rs 在后面的提交里几乎不变，8 of 14 这个频率回落。

### D. 第二版 P1 把「测试会读的仓内非 `.rs` 文件」放进全局项

- 原写法留了两条路：「后五样主 agent 倾向一变就整版重判，或并进关联文件」（`research/prompts/_tooling-broad-mutation-prune-r1-body.md:57`）；第二版选了前一条：「全局项变了就整版重判」（`research/prompts/_tooling-broad-mutation-prune-r2-body.md:32`）。
- 仓里读这类文件的测试很少：`.claude/gate.d/stage-inputs.tsv` 一处（`crates/singlefs-checker-tier/src/crash_identity.rs:1324`），litmus 一处（`crates/singlefs-harness/tests/publish_order_matches_litmus.rs`，Sonnet 第一轮已引），各由一条测试读。
- 14 个提交里这两样各变了 3 次（`research/prompts/tooling-broad-mutation-prune-r2-sonnet-model/out/globals.txt:16`）。两种读法的逐提交重判条数（`out/regimes.txt`，读法定义写在 `regimes.py` 文件头）：L1 字面第二版（litmus 与 stage-inputs 进全局、常量闭包按文件）中位「L1 median 4940 (97.0%)」（`research/prompts/tooling-broad-mutation-prune-r2-sonnet-model/out/regimes.txt:16`）；L2 松一档（这两样并进读它的测试、常量闭包按常量项，format 项取下界）中位「L2 median 3543 (69.6%)」（`research/prompts/tooling-broad-mutation-prune-r2-sonnet-model/out/regimes.txt:17`）；整表重判的提交：「commits at 100% of the list: L1 5 of 14, L2 4 of 14」（`research/prompts/tooling-broad-mutation-prune-r2-sonnet-model/out/regimes.txt:18`）。
- 翻面的主要是第 495bede6 号提交：L1 整表、L2 只有被改函数那 198 条（「495bede6 eff 198 L1 5091 L2 198.0」，`out/regimes.txt:14`）。Cargo.lock、Cargo.toml、`.cargo/` 的变更在两种读法里都整表重判，我不辩。
- 推翻它的现象：读这两个数据文件的测试比我数的多（我只扫了 `std::fs::read*`、`include_str!`、`CARGO_MANIFEST_DIR`、`.claude/` 与 `research/` 路径字面），并进读它的测试时漏了一条。

## 四、同样的打法打不打得中替代方案（按 crate、按编译依赖 `.d`）

读法：「按 crate」是键 = 被变异的 crate、它依赖的 crate、测试所在 crate 的全部源文件 sha；「按 `.d`」是每个测试二进制一份，键 = 它自己与依赖单元的 `.d` 列出的源文件集合（`.d` 只列本 crate 用到的源文件与 `include_*!` 的文件，依赖 crate 的源文件要靠依赖单元的指纹并进来，这一点是我的读法，没量过真 `.d`）。

| 判决条目 | 覆盖加关联文件（P2） | 按 crate | 按 `.d` |
|---|---|---|---|
| 1 粒度（省多少） | 文件名下条数中位 61.3%；函数粒度加第二版 F2 后有效条数中位 42.4%（`research/prompts/tooling-broad-mutation-prune-r2-sonnet-model/out/commits.txt:19` 「n=14 eff median 2157 (42.4%)」） | 更重：14 个提交都 100% | 更重（推的）：core 一变，这一档全重判 |
| 2 超时、负载 | 中 | 中 | 中 |
| 3 常量（S1、S2） | 中（有限，见第一节第 3 行） | 不中 | 并入依赖单元指纹则不中；只用本 crate 的 `.d` 则中（format 是另一个 crate） |
| 4 同名方法遮蔽（S4） | 中（今天 0 对） | 不中 | 不中 |
| 5 导出树外的输入 | 中 | 中 | 中 |
| 6 键缺项 | 中 | 中 | 中 |
| 7 `#[path]` 名字、profraw | 中 | 不中 | 不中（路径写法没量） |
| 8 宏、derive | 中（宏零处） | 不中 | 不中 |

读表：S1、S2、S4、实现层这几打只打中「覆盖得出关联文件」这条路；换成按 crate 或按 `.d` 它们不再成立，代价是第 1 行那一格（14 个提交都整表重判，整表 88.3 小时整套重判或 27.9 小时先跑 T，见 `research/prompts/tooling-broad-mutation-prune-r2-sonnet-model/out/v3.txt:11`）。第 2、5、6 行与键怎么定无关，三条路一起中。
先例：仓里有一处同形的实测，「按 crate 闭包取会把整个 harness 收进来，E164 W5 量到崩溃枚举节点 20 个提交全部重跑；按模块引用取就不会」（`crates/singlefs-checker-tier/src/crash_identity.rs:1317`）；那是 checker 档崩溃枚举的键，不是广谱变异的键。`.d` 做复用键的先例，我在 `crates/`、`research/scripts/`、`.claude/gate.d/` 里 grep `dep-info`、`.d 文件`、`depinfo` 找不到。
推翻第 1 行的「更重」：把按 crate 的键改成「只含被改的包及它的直接测试」之类更窄的写法，并量到低于覆盖那条路。

## 五、原写法里还站得住的几处（替被收严的一方留着的）

1. P2 抓到条的存在性：抓到只要「T 在这一版变异树上仍红」，记录 T 就够；常量初始化式里抓到的条同样有 T（第三节 B）。判决第 3 行要「不靠覆盖判」，我只反对把它写成「不复用」。推翻：常量值变后 T 记录不稳。
2. 原 P3 留的第二条路：非 `.rs` 的数据文件「并进关联文件」，只让读它的那几条测试的键带上它；第二版选了全局项，这一版数据里差一个提交（第三节 D），交用户。推翻：读数据文件的测试比我数的多。
3. 原 P5 的「超时就红」是保守方向；用户定了「算抓到」之后，要补的是倍数与单测复核（第三节 A），不是回到整段限时。推翻：复核用的单测本身也被拖过限时。
4. 原 P2 的「零覆盖直接记没红」在非常量初始化式、非只在常量里被叫的 const fn 上仍然是对的：这两类是 172 条（3.4%），其余 4 919 条不受影响。判决要求的次序（先编、再看覆盖）我不辩。推翻：零覆盖而被抓到的变异出现在这 4 919 条里。
5. 原 P1 的「按文件」在今天是简单、可核的：第二版的函数粒度在字面读法下对抓到的条不起作用（开头那一条）；若用户要函数粒度，要同时改执行集合的粒度。推翻：字面读法下被改文件不算 T 执行到的文件（开头那一条）。

## 六、交用户定的几条（按判据「B 改法收过头 ⇒ 那一条交用户定」）

| 条 | 候选 | 我要用户定的是什么 |
|---|---|---|
| 一 | 被改文件的「执行到」按文件，或按函数 | 开头那一条：字面读法下函数粒度是空的 |
| 二 | B1 超时记「抓到」后要不要单测复核 | 第三节 A |
| 三 | B2 拆成「零覆盖不判红」与「抓到的照常复用」 | 第三节 B：每次提交省 0.9–3.0 小时 |
| 四 | 关联文件里的常量按文件还是按常量项 | 第三节 C：按常量项偏离「关联文件照旧按文件」 |
| 五 | litmus 与 stage-inputs.tsv 进全局，还是并进读它的测试 | 第三节 D |
| 六 | 线程数、限时参数进不进键 | B6：没有证据结局随线程数变 |

## 七、V3 代价：一次提交改 1、5、20 个函数

### 单价（这一版已判 302 条的实测，本机数；采样只来自 5 个文件）

- 一条抓到的整套重判，编译加测试均值 72.4 秒；无效（编不过）0.9 秒；稳态（只含抓到与无效）每条均值 62.4 秒（`research/prompts/tooling-broad-mutation-prune-r2-sonnet-model/out/v3.txt:1` 「full 62.4  T-first 19.7」）。
- 先只重跑上次抓到它的那条测试（Sonnet 第一轮提的，不在第二版 P2 里，没被攻过）：每条 = 增量编译均值 20.4 秒加那条测试在基线里的耗时，均值 22.8 秒、中位 10.4 秒（`out/res.txt:8`），稳态 19.7 秒。它假设这一版上那条测试仍红；不红就退回整套，要多付一遍。
- 抓到条里 T 的分布：核心库测试 185 条、harness 55 条、checker 库 1 条（`out/t_inline.txt:1`）。

### 每个版本都要付的固定量（与改了几个函数无关）

- 基线：驱动每一版每一步先在没改过的树上跑一遍（`research/scripts/broad-mutation.py:73` 「基线：每一版每一步先在没改过的树上跑一遍」），七步相加 1 426.0 秒 = 0.40 小时（`research/prompts/tooling-broad-mutation-prune-r2-sonnet-model/out/v3.txt:2`）；第七步 checker 档在这一版是红的、47.3 秒是它红掉的时刻（`/home/fy5090/.local/share/singlefs/broad-mutation/versions/19c45e021b5638f68eac7c72dcc317a4574bdaa2b9f33a4a8855284827265cd4/baseline.tsv:8`），修绿之后会更长。
- 覆盖遍：每条测试一个进程，第一段 776 条（`/home/fy5090/.local/share/singlefs/broad-mutation/versions/19c45e021b5638f68eac7c72dcc317a4574bdaa2b9f33a4a8855284827265cd4/logs/baseline.one-test.log:4` 「Starting 776 tests across 93 binaries」）加耗时用例 61 条（`/home/fy5090/.local/share/singlefs/broad-mutation/versions/19c45e021b5638f68eac7c72dcc317a4574bdaa2b9f33a4a8855284827265cd4/logs/baseline.two-test-slow-harness.log:6` 「Starting 61 tests across 93 binaries」）加 checker 档。**插桩倍数没量。** 下限就是未插桩的 1 426 秒；若倍数是 1.5 / 2 / 3（假设，没量过），覆盖遍 0.59 / 0.79 / 1.19 小时（`research/prompts/tooling-broad-mutation-prune-r2-sonnet-model/out/v3.txt:7`–`:9`），另加插桩的冷编译（三个编译步，没量）与每条测试的 profraw 合并与导出（没量）。
- 所以改一个函数也有每版至少 0.8 小时（基线加覆盖遍，倍数取 1）的固定量，倍数取 2 约 1.2 小时。
- 量法（我没做，按定义不编译）：在草稿目录的仓副本里装 llvm-tools（攻方第一轮 10 秒装好），`RUSTFLAGS="-C instrument-coverage"`、独立的 `CARGO_TARGET_DIR`、`LLVM_PROFILE_FILE` 带 `%p`，把 `baseline.tsv` 的七步各跑五次取中位数、对照同一步的基线秒数；checker 档那一步属重型，只能主 agent 在提交时或用户要求时跑。

### 1、5、20 个函数（两种读法；单价取上面的稳态；不含固定量）

函数从 1 031 个有变异的函数里等概率抽，各抽 20 000 次（`out/sim.txt`）。读法 A：只重判被改函数名下的条。读法 B：被改的整份文件名下的条（第二版字面：被改文件在 T 执行到的文件里，函数外面的部分按 F2 并进来）。溢出上界：再加上别的文件里 T 是 harness 测试、且它执行到被改文件的条，按 55/241 = 22.8% 乘以被改文件之外的条数，假设每条 harness T 都执行到被改文件，是上界；harness 占比只来自 5 个文件的采样。

| 改几个函数 | 读法 A 条数：均值（中位、第 90 百分位） | A 小时：整套 / 先跑 T | 读法 B 条数：均值（中位、第 90 百分位） | B 小时：整套 / 先跑 T | 溢出上界条数 | B 加溢出小时：整套 / 先跑 T | 编译次数 A / B / B 加溢出 |
|---|---|---|---|---|---|---|---|
| 1 | 5（2、11）（`out/sim.txt:3`） | 0.09 / 0.03 | 368（314、1157）（`out/sim.txt:4`） | 6.4 / 2.0 | 1077 | 25.1 / 7.9 | 4 / 317 / 1244 |
| 5 | 24（20、42）（`out/sim.txt:7`） | 0.42 / 0.13 | 1516（1490、2225）（`out/sim.txt:8`） | 26.3 / 8.3 | 815 | 40.4 / 12.8 | 21 / 1305 / 2007 |
| 20 | 96（91、134）（`out/sim.txt:11`） | 1.67 / 0.53 | 3364（3420、3815）（`out/sim.txt:12`） | 58.3 / 18.5 | 394 | 65.2 / 20.6 | 83 / 2895 / 3234 |

小时数与编译次数的算术在 `research/prompts/tooling-broad-mutation-prune-r2-sonnet-model/out/v3.txt:3`–`:5`（编译次数 = 条数乘稳态里非无效的占比 0.861，只算每条一次增量编译，不含固定量里的几趟冷编译）。

- 跑前判据：「V3 估出一次提交改 5 个函数也要几十小时 ⇒ 记下，交用户看值不值」：读法 B 下 5 个函数是 26.3 小时（整套）或 8.3 小时（先跑 T），加溢出上界 40.4 或 12.8 小时；读法 A 下 0.42 小时。⇒ 记下。取决于开头那一条（读法 A 还是 B）与溢出的真实大小。
- 随机抽函数低估了真实提交：真实提交的函数成簇落在少数文件里（B 偏小），又常落在被很多测试执行到的中心文件里（溢出偏大）。
- 真实提交的校准（14 个提交，判决里的 15 个剔除整体导入）：改的函数数（含测试函数）从 1 到 291，中位 38；函数粒度加 F2 的有效条数中位 2 157 条，「37.4 h full / 11.8 h T-first」（`research/prompts/tooling-broad-mutation-prune-r2-sonnet-model/out/v3.txt:10`）；L1、L2 两种读法中位 85.7 与 61.5 小时、27.1 与 19.4 小时（`out/regimes.txt:16`、`:17`）。14 个提交里只有第 8186d5b5 号（改 1 个函数）在 L2 下低于 1 小时（`out/regimes.txt:1`）。1、5、20 三档是真实提交里偏小的那一端。
- 推翻它的现象：量到插桩倍数在 1.5 以内且覆盖遍的合并与导出在一分钟内，固定量就只有 0.8 小时左右；或有了覆盖产物，数出「T 是 harness 测试且执行到被改文件」的真实条数，溢出上界落到 0 附近。

## 八、没做什么

- 没量插桩倍数、覆盖遍的冷编译、profraw 合并与导出、覆盖产物的体积（要编译与覆盖工具，按定义不做）；V3 的覆盖遍只给出下限与假设倍数。
- 没有覆盖，所以「溢出」（别的文件里的条因为 T 执行到被改文件而重判）只有上界，没有估计值；5 个文件的采样外推到整张清单只是量级，`walk.rs` 的 1 157 条一条没判过。
- 函数边界、常量与 derive 的名字匹配、「读数据文件的测试」都是我写的 token 级静态扫描，不是类型解析：函数名与变异名对照 0 处对不上（`out/parse_list.txt:4`）验过边界，常量与遮蔽没有第二个独立方法核。「够不着」类的零（S2 的 0 个文件、S4 的 0 对）只是一次扫描，不支撑「没有」。
- 14 个提交是里程碑建设期的大提交（9 到 22 个文件一个提交），不代表以后的提交习惯；提交里的同名函数对到 HEAD 清单时按名字与比例近似。
- 没复核第一轮腿报告里攻方的模型（S1、S2、S4a、S4b）的真跑结果，只复核了它们的前提在真 crates 上有多少条。
- 没替主 agent 撤销或维持判决，只交复核结果；第六节的候选是我推出的条件，没有一条被攻过。

交回前跑 `python3 research/scripts/cite-check.py research/prompts/tooling-broad-mutation-prune-r2-sonnet-output.md` 的末行：✓ 核了 29 处引文，对上 29 处，没判 0 处（逐处列在上面）
