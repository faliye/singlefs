# m3-batch62-code-r1 攻方腿（Opus）报告（2026-10-11）

被判：第 62 批（冻结副本 `~/.local/share/singlefs/work/m3-batch62-code-r1/repo/`，代码行号按它）。攻击面 R1–R5，立场照正文第四节「云端攻方」。
不跑 checker 档包的任何测试：被判的三条 `verdict_store` 单测在原型 crate 里逐条照抄（`mirror_*`），另加两条攻方用例（`attack_*`），跑的是原型 crate 的测试二进制；流水线与 GPU 两条（R1、R2、R4）只读码。

## 复跑

```
cd /home/fy5090/code/singlefs
bash research/prompts/m3-batch62-code-r1-opus-model/rerun.sh <工作目录> all
# 环境（可不设）：SINGLEFS_REPO=<冻结副本>、OPUS_MEMORY_CAP=16G、OPUS_THREADS=15；末尾 compare 逐份比 expected/
```

本腿跑在 `~/.local/share/singlefs/work/m3-batch62-code-r1-opus/`（编译目录拷自 `~/.local/share/singlefs/work/m3-batch60-61-code-r1-verify/target` 当种子，第一次编之前副本里 `find crates -type f -exec touch {} +`）。装置有两版：

- 第一版（原型五条，sha256 `683982027c649fc8…` 的测试文件、`a1b113022a240de7…` 的 `rerun.sh`）连跑两趟 `all`，第二趟 compare 原样输出「一致 base.txt」「一致 mut-facts-after.txt」「一致 mut-flip.txt」「一致 mut-rev.txt」「一致 mut-take0.txt」「一致 mut-take1.txt」；六份输出存在 `expected-before-sweep/`。
- 第二版（今天模型目录里的样子）在同一份测试文件末尾加了一条放开批长扫的用例（`sweep_batch_lengths_against_one_batch`），`rerun.sh` 把它单独起（库放 tmpfs，不打批账），原五条照旧。六个阶段各跑了一趟，输出存在 `expected/`；与第一版逐行比，原五条那一段只差「test result」行的「0 filtered out」变「1 filtered out」，其后多了扫批长那一段（`diff` 原样见「放开批长扫」一节）。其中 `mut-take0`、`mut-flip`、`mut-facts-after` 三个阶段中途被别的会话一趟 crates 变异整表（nice 0，三个测试进程合计约 3200% CPU）卡住：`nice -n 19` 的 rustc 起来 52 秒时 `ps` 记的是 1.3% CPU，编 checker 档那一个包四次（200、220、225、225 秒）都没编完（每次 `rc=124`，编译被打断、不留半成品），机器空一些之后第五次 144 秒编完，三个阶段随后跑完。第二版重拷副本时设 `OPUS_NO_TOUCH=1`（`rerun.sh` 文件头那一段：拷来的文件带冻结副本的原时刻，只有每次重写的 `verdict_store.rs` 与原型 crate 比上次的产物新，只重编这两个包；副本 base、mut 与种子共用同一组编译单元，所以每个阶段都必须先重拷再编，`rerun.sh` 的每个阶段都是这样做的）。

模型目录 `research/prompts/m3-batch62-code-r1-opus-model/` 每个文件的 sha256：

```
ffb1651054487192f96eba5cb8579369ef5d4d97fb3bf538d7c07a554cfb9160  ./expected/base.txt
640564ed1413b4aea0289b9c3aa9c6683113cbe2143f5b13bca8bab2c45a9b62  ./expected/mut-facts-after.txt
8801618174bd32b87c598efe8a9f39cabb623cbee780e0bdd1df28158771c9e6  ./expected/mut-flip.txt
44c7892feb6df608f49f0114803b062b2631d354bc645edbda81146f547417d7  ./expected/mut-rev.txt
ad413b8372d41a6cc869bcab572f71ba0ff353565b8414bbb693fffecb1391f2  ./expected/mut-take0.txt
fedcc04710afbb9f8cc1eb87edc09ba2580ff5f6bcec0df4b9b49cf521481453  ./expected/mut-take1.txt
a106b23924cea8c6848026f3615987d29b24d2070f1215f0e73b352a74198d48  ./expected-before-sweep/base.txt
6d64bf58b1f971ddb21b85e72491b32979df0622e0f84917392b7595f50bd1e8  ./expected-before-sweep/mut-facts-after.txt
e446690eb6ebc2701eb6a30f7e32c28576c880275000f60bb24f936606750867  ./expected-before-sweep/mut-flip.txt
206aa86cb2aff2cc80afcf659ebf1f73cc440425a9cb9c338deb0d9868799b4c  ./expected-before-sweep/mut-rev.txt
f37482812636c70ed2f551417e2b0f14e92c4d35265d304e5d76c0dbb4b62615  ./expected-before-sweep/mut-take0.txt
fe5e1273fdebdef5e28e49ae4806593697d5b68800332f7bcd21fbe298f1e153  ./expected-before-sweep/mut-take1.txt
6bf09cfbe9a3a68b55729214c759e5db32d98a3f6032fac63a67295004168c72  ./expose.py
93b0edcaea7b55909081e7c4af8ecc47fa24873a5b733d9399430a1a4f766ac9  ./mutants.py
afaab318413fb471fd29b4e244fe06eabbafe2f23073436f540985e68c34d465  ./probe-crate/Cargo.toml
2dd4558beab2623618c12ee104dbccec2e33ecfe0d63cf677832b59ee7da85ef  ./probe-crate/src/lib.rs
e1731911f179dd1244e2f41e486d75f84cbdb9564fbc96f96cf6a10cb41ce3e9  ./probe-crate/tests/batch62_store_probe.rs
4f9d0c74621d6080ca0b7f674227df7b1f2e6e40c8913c105a0a8929bc9c8c0d  ./rerun.sh
```

装置：`expose.py` 在副本的 `verdict_store.rs` 里加三个公开包装（按给定批长导入、列整库键值行、按编号读内存表）与一行设了 `OPUS_TRACE_BATCHES` 才打的批账（每批几行、几种新向量），不改已有的行；`mutants.py` 的前三种是第 62 批追加进 `crates/mutations.tsv` 的 Q3 三行，逐字核过与表里第 3989、3990、3991 行（与第 3965 行）的原文、替换文相同，后两种（`take1`、`rev`）是本腿加的邻居。机器没被占满时每阶段编约 20 秒、原五条跑 0.3 秒、扫批长那一条在 tmpfs 上几秒，没缩取样；没用层 0 枚举，崩溃状态数 0。开跑前 `ps` 看到别的会话一条 `cargo test --offline --no-fail-fast -p singlefs-format …`（pid 1071484）在跑，没有性能测量，照常 `nice -n 19` 跑，没等锁。

## 各格判定一览

| 格 | 判定 | 依据 |
|---|---|---|
| R1 核对行钉得住、h02 形状、`mv` 红在哪 | 没打中（读码） | 第二台那一块只在第二台库里核；比对走全计划的块；`mv` 下红在 (0,0,0) 那一句 |
| R1-a 事实行那半句 | **打中（小，读码）** | 导入那一趟第 ② 段开头自己写这条流的事实行，「事实行与一台判完的相同」不管导入带没带事实都过 |
| R2 重导计数的口径与四元组 | 没打中（读码） | 口径是「点名且对方库里有的块」= 写下 + 已有且相同；与报的 `imported_blocks=` 同一字段；四元组只含四个计数 |
| R3 私有批长 | 没打中（读码） | 产品只经 64 MiB 那一个入口 |
| R3 28 行 / 7 批 | 没打中（量过） | 逐批 5、7、4、7、3、1、1 行；盘上 27 个新键加 1 个改值 |
| R3 跨批编号的判别力 | **打中（量过）** | 一批里登记两种以上新向量的形状没有用例：`take1`、`rev` 两个邻居变异下，被判三条单测（镜像）全绿 |
| R3-b 换批判据那一行红在哪一句 | **打中（小，量过）** | 第 3989 行在 Q3 用例里红在「照产品的批长：一批写完」，与第 3965 行同一个变异、同一种证据 |
| R3 事实行补写 | 没打中（量过） | 形状可达（批账里事实独占最后一批）；只点核对行重导补得齐；`facts-after` 红在点名那一句 |
| R3 整行替换那一行 | 没打中（量过） | 新写法下 `flip` 照改行为：21 行那条红在账上（`synced_writes: 6`） |
| R4 「本次未跑」与门禁 | 没打中（读码） | 门禁按子串认 `NOT_RUN_NO_GPU_ADAPTER`，记本次未跑；`stopped_early` 只由借不到显存或叫停文件来，叫停变量被守卫清掉 |
| R4 2 块、8 块与三行 GPU 变异 | 没打中（读码，推的） | 2 = 第 1 块 GPU 判器 + CPU 对照；8 = 4 张 × 两种；三行各红处见 R4 节 |
| R5 产品代码与签名 | 没打中（读码 + 量过） | Q1–Q4 点名的几处产品代码原样就对（原型五条在原样下全绿）；签名变化只在 `verdict_store` 模块里 |

## R1　核对行钉得住没有（Q1）

### 主问：没打中（读码）

逐步数用例里哪一块在哪台核（四趟都设了核对，`crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs:1252`「environment.set("SINGLEFS_CRASH_AMPLIFICATION_VERIFY", OsStr::new("cpu"));」）：

1. 一台判完那一趟：三块都判、都核，进一台判完那份库。
2. 1/2 那一趟：只判第 1 块（发布那个点），只核领到的块（CPU 核那一支传的是领块条件，`crates/singlefs-checker-tier/src/crash_amplification.rs:8130`「&|block| selected.contains(&block),」），第 1 块的核对行只在第二台的库里。
3. 0/2 那一趟（不导入）：判、核第 0、2 块。
4. 0/2 导入那一趟：第 0、2 块已判已核（复用）；第 1 块的结论与核对行只能从导入来（第 ② 段同样只核领到的块）。第 ③ 段走计划里每一块（`crates/singlefs-checker-tier/src/crash_amplification.rs:3668`「for (point_index, point) in plan.points.iter().enumerate() {」），判过而没有核对行的记一块（`crates/singlefs-checker-tier/src/crash_amplification.rs:3675`「comparison.blocks_without_verifier += 1;」）。

h02 的形状在用例里真的出现：第 1 块只在第二台的库里核过。变异「两种核对代码都不点」（表里第 62 批 Q1 那一行，打在 `crates/singlefs-checker-tier/src/crash_amplification.rs:7770`「for verifier_digest in &verifier_digests {」）之下第 1 块判过、没核对行，导入那一趟 `blocks_without_verifier = 1`，红在 Q1 的第一句断言（`crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs:1289`「"导入那一趟第 ③ 段：第二台那一块的核对行随结论一起导进来，没有一块缺核对行"」），后面按块读回核对行那一句走不到。上一轮判决复跑的 h02 数与这一步一致（主 agent 判决第六节「mut-mv： … without_verifier=1」），本腿没在被判用例上量。

按块读回核对行那一句自己的判别力：第 1 块的核对行只能从导入来，读回相同要靠导入，不是摆设；变异把它挡在前面那一句，表里没有一行单独证它红。没再造只打掉内容、不打掉有无的变异（这条流全绿，核对位全 0，改值的变异在 (0,0,0) 那一句也看不出，见「没打中的形状」）。

### R1-a　事实行那半句对导入没有判别力：打中（小，读码）

用例对每块断言「核对行点名的事实行：导入之后的本机库与一台判完的读回不同」（`crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs:1403`「"块 {} 的核对行点名的事实行：导入之后的本机库与一台判完的读回不同",」）。这一句不管导入带没带事实行都过：只要这一趟核（设了 `VERIFY=cpu` 就核，`crates/singlefs-checker-tier/src/crash_amplification.rs:8083`「|| std::env::var("SINGLEFS_CRASH_AMPLIFICATION_VERIFY").is_ok_and(|value| value == "cpu")」），第 ② 段一开头先把这条流的事实行写进本机库（`crates/singlefs-checker-tier/src/crash_amplification.rs:3537`「.store_flow_facts(&facts_digest, &facts.to_bytes())」），写在逐块判领不领之前；同一条流、同一份展开，摘要与一台判完那份相同。0/2 不导入那一趟就已经写过一次。

把「按计划导入不导事实行」整段打掉（`crates/singlefs-checker-tier/src/verdict_store.rs:1909`「for facts_digest in &facts_digests_named {」那个循环一行都不写），这一句照绿——推的，没在被判用例上跑。按计划导入的事实那一半由库层两条单测钉着（21 行那条的 `flow_facts_written: 1`、Q3 事实行补写那条），表里第 3991 行在镜像上红在点名那一句（见 R3），所以不是缺口，是这一句的注释与断言说多了。

四句：
1. 分不分辨臂：不分辨。「导入带事实行」与「导入不带事实行」两种实现在这一句上读数相同。
2. 系统看不看得到：流水线这一层看不到。跑完之后可观测的只有库，而第 ② 段自己写同一行；`EnvironmentRun` 不带按计划导入的账（`flow_facts_written`）。属于「判别子观测不到」那一形，只能在库层判。
3. 满足的是判据字面的哪一句：判决 Q1 的验收只要求「按块读回的核对行与整库导入（或一台判完那份库）相同」，没要求事实行；这是用例文件头 `crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs:1213`「它点名的事实行与一台判完的逐项相同」那半句自己说多了，不是 Q1 的字面没做到。
4. 改法：把这半句与它的断言删掉，或在注释里写明「事实行由这一趟第 ② 段自己写，这一句不判导入」；判导入带不带事实，留给库层那两条。只在我的读码上推过、被攻过零轮。

## R2　重导计数（Q2）：没打中（读码）

- 口径：流水线与给定状态表两路都把「写下的」与「本库已有且相同的」加起来（`crates/singlefs-checker-tier/src/crash_amplification.rs:7787`「tally.conclusion_blocks_written + tally.conclusion_blocks_already_identical;」，同文在第 8910 行），对方库里没有的不算（`crates/singlefs-checker-tier/src/crash_amplification.rs:8373`「写下的加本库已有且相同的，对方库里没有的不算」）。所以钉的是「点名、且在对方库里找到的块」，不是「写下的块」，也不是「点名的块」。第二次导入全是已有且相同，等于第一次，正是这个口径（`crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs:1297`「(importing_run.imported_blocks, 0, 0),」）。
- 与报出去的那一行同一个量：流水线那一行由用例拿同一个字段拼（`crates/singlefs-checker-tier/tests/crash_amplification_fixed_script_through_unmount.rs:473`「run.imported_blocks,」），给定状态表那一行在引擎里拼（第 8807、8828 行）。与判决 P5-2 的改法（把「已有且相同」那一项也数进来）是同一个量。
- 邻居变异逐个推：「只数写下的」第一次报 2（流水线）、4（给定状态表），绝对值那一句照过，第二次报 0，红在「再导一次」那一句——表里两行就是这个；「只数已有且相同的」第一次报 0，红在绝对值那一句；「连对方库里没有的也数」流水线第一次报 3（第 1 块 GPU 判器那一种两边都没有），红在「导入写的结论块数等于第二台那一份」那一句。三个都推的，没跑。
- 给定状态表那一路的四元组只含四个计数（`crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs:1546`「let tally_counts = |run: &GivenStatesRun| {」，到第 1551 行「run.tally.tables_left_to_the_other_hosts,」），不含计数之外的东西。另一句「再导一遍之后引擎交回的结论与第一次逐张相同」（第 1570 行）比的是交回的结论：两趟读的是同一份本机库、同一批表，正确实现下没有会让它变的输入。正确实现下会让「再导一遍」红的只有一种改动：以后有人把「本库已有就不导」做成优化，`imported_blocks` 第二次报 0——那时红是对的，因为口径变了，要跟着改的是用例与报出去的那一行，不是放过它。
- 驱动脚本按 `imported_blocks>0` 认导入行（`scripts/crash/crash-amplification.sh:79`「导入行（行尾 import_run=true，或带 imported_blocks>0）不是分份行」）：数进「已有且相同」之后，重跑一趟全已导过的也报 >0，照样认成导入行，与这个口径一致，不是回退。读码，没跑驱动。

## R3　换批、跨批编号与事实行补写（Q3）

### 私有批长、28 行 / 7 批、事实行补写、整行替换：没打中

- 私有批长：`fn import_the_planned_rows_in_batches_of(` 不带 `pub`（`crates/singlefs-checker-tier/src/verdict_store.rs:1785`「fn import_the_planned_rows_in_batches_of(」），产品唯一的调用把常量传进去（`crates/singlefs-checker-tier/src/verdict_store.rs:1777`「self.import_the_planned_rows_in_batches_of(other, planned, PLANNED_IMPORT_BATCH_BYTES)」）；全仓 `crates/` 下按名字 grep，别的调用只在同一文件的单测里。产品路径上拿不到别的批长。本腿为了从库外调它在副本里加了包装（`expose.py`），这是量法，不是产品的口。
- 28 行 / 7 批（量过）：镜像（`research/prompts/m3-batch62-code-r1-opus-model/probe-crate/tests/batch62_store_probe.rs:230`「fn mirror_q3_one_byte_batches() {」）在原样下全绿，批账逐批打出来，与实现员的拆法逐批对上——四块结论各一批（F1 GPU 5 行、F2 GPU 加新向量 7 行、F2 观察者 4 行、F3 GPU 加新向量 7 行），两条路径加 F1 核对行 3 行一批，F2 核对行一批，事实一批：

```
OPUS_IMPORT q3 batch=one-byte
OPUS_BATCH rows=5 new_vectors=0
OPUS_BATCH rows=7 new_vectors=1
OPUS_BATCH rows=4 new_vectors=0
OPUS_BATCH rows=7 new_vectors=1
OPUS_BATCH rows=3 new_vectors=0
OPUS_BATCH rows=1 new_vectors=0
OPUS_BATCH rows=1 new_vectors=0
```

  （`research/prompts/m3-batch62-code-r1-opus-model/expected/base.txt` 第 25–32 行。）换一条路数同一件事：导入前后盘上的差是 27 个新键加 1 个改值（f.0/f.1 那条路径换了身份），合 28 行：`research/prompts/m3-batch62-code-r1-opus-model/expected/base.txt:35`「OPUS_Q3 disk new_keys=27 changed_values=1 same_as_one_batch=true」。批账是本腿在副本里加的一行打印，账里的 28 是被判代码自己数的，两条路不共用计数。
- 事实行补写（量过 + 读码）：一字节一批时事实独占最后一批，排在 F2 核对行那一批之后（上面最后两行），按产品批长时只要攒到 64 MiB 的那一刻落在某个核对行之后，事实行同样落进收尾那一批：断在两批之间就是「核对行在、事实行缺」，形状在产品里可达。只点核对行重导补得齐（镜像 `mirror_q3_facts_backfill` 原样下绿，账「flow_facts_written: 1, rows_written: 1, synced_writes: 1」）。另：流水线下一趟只要核，第 ② 段自己也会写这一行（R1-a），所以这一形在流水线里还有第二条补法；给定状态表那一路不导核对行，碰不到这一形。
- 表里第 3991 行（事实点名挪到跳过之后）在镜像上只红在点名那一条、红在账那一句（`research/prompts/m3-batch62-code-r1-opus-model/expected-before-sweep/mut-facts-after.txt:22`「thread 'mirror_q3_facts_backfill' panicked at crates/opus-probe-b62/tests/batch62_store_probe.rs:282:5:」，第 282 行是「核对行都已有且相同、一行不写；事实行补写 1 行」那一句）。
- 整行替换那一行（第 3965 行）在新写法下照改行为：21 行那条（镜像）红在账上，`synced_writes` 从 1 变 6：`research/prompts/m3-batch62-code-r1-opus-model/expected-before-sweep/mut-flip.txt:31`「thread 'mirror_21_rows' panicked at crates/opus-probe-b62/tests/batch62_store_probe.rs:317:5:」，第 33 行「left: PlannedImportTally { conclusion_blocks_written: 3, conclusion_blocks_already_identical: 1, conclusion_blocks_missing_in_the_other_library: 5, paths_written: 2, verifier_blocks_written: 2, flow_facts_written: 1, rows_written: 21, synced_writes: 6 }」。

### 跨批编号的判别力：打中（量过）

被判的 Q3 用例里，一字节一批的每一批最多登记一种新向量（上面批账 `new_vectors=` 全是 0 或 1），只有照产品批长那一趟一批登了两种，而那一份导完只比盘上字节、随即丢掉、不重开、也不经它读（`crates/singlefs-checker-tier/src/verdict_store.rs:4209`「drop((imported_in_one_batch, imported_in_many_batches));」）。写下一批之后把这一批登记的向量放进内存表的那个循环（`crates/singlefs-checker-tier/src/verdict_store.rs:2069`「for verdict_vector in pending.verdict_vectors_registered_in_the_batch {」，编号照内存表的长度取，第 2071 行「u8::try_from(self.verdict_vector_by_number.len())」）因此只有「一种都不进」（表里第 3990 行 `take(0)`）被钉住；「一批里第二种以后不进」与「一批里倒着进」两种错法没有一条用例看得见。

造的两个邻居变异（`mutants.py` 的 `take1`、`rev`，都只改那一行）与两条攻方用例：

- `attack_in_memory_table_after_a_product_batch_import`（`research/prompts/m3-batch62-code-r1-opus-model/probe-crate/tests/batch62_store_probe.rs:332`「fn attack_in_memory_table_after_a_product_batch_import() {」）：照产品批长从同一份 B（加 F3）导入，导完不重开，经同一个库按内容读回 F1–F3 每种结论与 B 比，再登记一种新向量（第 351 行「let fresh = reg(&mut local, b"registered-after-the-import");」），重开再比。
- `attack_two_new_vectors_in_one_batch_then_one_more_batch`（`research/prompts/m3-batch62-code-r1-opus-model/probe-crate/tests/batch62_store_probe.rs:376`「fn attack_two_new_vectors_in_one_batch_then_one_more_batch() {」）：对方库 F2 GPU 一块里用两种新向量（同一批登记），F3 GPU 再用第三种（一字节一批时在下一批），一字节一批与一批写完逐字节比，重开读编号 2、3、4。

结果（每格是那一条用例在那个副本上的结局；原样输出在 `expected-before-sweep/`（第一版）与 `expected/`（第二版）对应文件，这五条的那一段两版逐行相同）：

| 用例 | 原样 | take0（表 3990） | take1 | rev | flip（表 3965/3989） | facts-after（表 3991） |
|---|---|---|---|---|---|---|
| mirror_q3_one_byte_batches（被判 Q3） | 绿 | 红 | **绿** | **绿** | 红 | 绿 |
| mirror_q3_facts_backfill（被判 Q3 事实） | 绿 | 绿 | **绿** | **绿** | 绿 | 红 |
| mirror_21_rows（被判 21 行） | 绿 | 红 | **绿** | **绿** | 红 | 绿 |
| attack_in_memory_… | 绿 | 红 | 红 | 红 | 绿 | 绿 |
| attack_two_new_vectors_… | 绿 | 红 | 红 | 绿 | 绿 | 绿 |

（`expected/*.txt` 各文件的「test result」行与「panicked at」行；原样「test result: ok. 5 passed」在 `expected/base.txt` 第 36 行，`take1`「3 passed; 2 failed」在 `expected/mut-take1.txt` 第 37 行，`rev`「4 passed; 1 failed」在 `expected/mut-rev.txt` 第 42 行。）

两种错法在产品上会怎样（量的是原型上的库，产品路径那一半是推的）：

- `rev`：导完不重开时内存表把 2、3 两个编号的内容对调了（`research/prompts/m3-batch62-code-r1-opus-model/expected/mut-rev.txt:5`「OPUS_MEM in_memory_numbers_0_to_4=[Some(""), Some("red-f"), Some("red-f-block-three-on-the-other-host"), Some("red-f-judged-on-the-other-host"), None]」），经同一个库读回有 3 种结论与 B 不同，重开之后又对了（同一文件第 10 行「OPUS_MEM differs_before_reopen=3 differs_after_reopen=0 fresh_read_back=Some("registered-after-the-import")」）。流水线里导入（`crates/singlefs-checker-tier/src/crash_amplification.rs:7708`「if let Some(list) = &import_list {」）与之后读违例（`crates/singlefs-checker-tier/src/crash_amplification.rs:8275`「let violations = read_the_violations_stored_under(&store, &plan, &judged_conclusions)」）在同一个进程、同一个库上，读违例按内存表换内容：导进来的块报的违例会是另一种判定向量。第 ③ 段比对只看向量空不空，对调两种红向量它看不出（推的）。
- `take1`：导完内存表少了 3 号（`research/prompts/m3-batch62-code-r1-opus-model/expected/mut-take1.txt:5`「OPUS_MEM in_memory_numbers_0_to_4=[Some(""), Some("red-f"), Some("red-f-judged-on-the-other-host"), None, None]」），经同一个库读 F3 报「读结论: StoredBytesDoNotMatchTheLayout」（第 7 行）；多批时下一批的新向量撞上 3 号，盘上两张向量表对不上，库再也开不了（第 18 行「重开多批那一份: StoredBytesDoNotMatchTheLayout { table: VerdictVectorNumberByContent」，原因原文「按内容的判定向量表与按编号的那张对不上」）。流水线里第 ① 段判本机的块排在导入之后（`crates/singlefs-checker-tier/src/crash_amplification.rs:7874`「let judging_started = std::time::Instant::now();」），判出没见过的红向量时同样经内存表登记、同样撞号（推的）。
- 产品原样都对：原型五条在原样下全绿（`expected/base.txt` 第 36 行），盘上与内存表一致、导入之后再登记的编号接在后面（`research/prompts/m3-batch62-code-r1-opus-model/expected/base.txt:6`「OPUS_MEM fresh_number=4」）。

四句：
1. 分不分辨臂：被判三条单测不分辨「原样」与 `take1`、`rev`（三条在三个副本上都绿）；两条攻方用例分辨（原样绿，`take1` 两条红，`rev` 一条红）。
2. 系统看不看得到：看得到。被判用例在 `drop` 之前手里就拿着导完的库，能经它读回；换一份对方库让一块里有两种新向量就造得出那一批。
3. 满足的是判据字面的哪一个分句：判决 Q3 的改法「用例在小批长下走多批，断言跨批编号接续、读回与一批相同」与验收「至少三行变异：换批判据反过来、跨批编号重起、…；各点名、证红」字面都做到了，这一格不是 Q3 字面没做到；它落在正文第四节攻方立场的「改坏照绿」，依据 `.claude/singlefs-ai-sop/rules/test-discipline.md@变异测试证明的是断言会红，不是覆盖`「变异只证明「凡是写了断言的代码都被变异测过」，**不证明**「结论涉及的每一处代码都测过」。」。一批多种新向量正是产品 64 MiB 批长下的常态（一块 GPU 结论里有几种红向量就登几种），被判用例只量了一批一种那一形。
4. 改法在打中的格上还中不中：见末尾改法表；Q3 已有的三行变异对这两格不起作用（它们本来就红，`take1`、`rev` 不在其中）。

### R3-b　换批判据那一行红在产品批长那一句：打中（小，量过）

表里第 62 批 Q3 第一行（冻结副本的 `/home/fy5090/.local/share/singlefs/work/m3-batch62-code-r1/repo/crates/mutations.tsv:3989`「第62批 Q3 按计划导入 换批判据反过来（一字节一批时一批都不换，正推 B1）」）与被整行替换的第 3965 行是同一处、同一个替换。名字说的是一字节一批时一批都不换（账应当是 1 批而不是 7 批），可 Q3 用例先导的是产品批长那一份：判据反过来之后那一份每放一组就写一批，账成了 7 批，红在第一句「照产品的批长：一批写完」（`crates/singlefs-checker-tier/src/verdict_store.rs:4188`「"照产品的批长：一批写完"」），一字节一批那一句（第 4202 行）走不到。镜像上量的就是这个次序：`research/prompts/m3-batch62-code-r1-opus-model/expected-before-sweep/mut-flip.txt:49`「thread 'mirror_q3_one_byte_batches' panicked at crates/opus-probe-b62/tests/batch62_store_probe.rs:248:5:」，第 248 行是镜像里「照产品的批长：一批写完」那一句，第 51 行左值「synced_writes: 7」。

所以这一行证的与第 3965 行证的是同一件事（产品批长下判据反过来），一字节一批那一句自己会不会红，表里没有一行证过。判别力本身没丢（推的：那一句在 `flip` 下读到 1 批，走得到就红），丢的是这一行的名字与它实际红的地方对不上。

四句：1. 不涉及臂，是一行变异的证据口径。2. 看得到：换批判据只在小批长下反过来的改法（例如只在 `batch_size_in_bytes` 小于产品批长时反过来）就落在那一句上。3. 依据 `.claude/singlefs-ai-sop/rules/test-script-discipline.md@测试脚本自己也要被测试证明有效`「每一处会判红的分支，配一个弄坏开关或一条变异：弄坏之后自证要红，红在点名的那一项。」：表里点名到测试名为止，字面上「红在点名的那一项」做到了；名字里那半句（一字节一批）没有证据。4. 改法：要么把第 3989 行的名字改成与第 3965 行同一件事、承认它是重复的，要么换一个只在小批长下改行为的替换让它红在第 4202 行那一句。被攻过零轮，没量。

## R4　GPU 两条与「本次未跑」（Q4）：没打中（读码）

- 门禁认不认得那一行：两条在没卡、借不到显存时打的是 `not_run_line`，记号是 `crates/singlefs-harness/src/gpu_memory_ledger.rs:567`「pub const GPU_NOT_RUN_MARKER: &str = "NOT_RUN_NO_GPU_ADAPTER";」；feature-gated-tests 格按子串逐处认它（`.claude/gate.d/checker-tier-layer0-replay.sh:684`「while ((at = index(rest, "NOT_RUN_NO_GPU_ADAPTER ")) > 0) {」），逐条记「本次未跑」、整格写「不算全绿」，不会把「打了这一行的 1 passed」当跑过。跑到的条数另按「test result」行数，一个文件 0 条另记本次未跑。
- `stopped_early` 那一支什么时候走到：只有第 ① 段或第 ② 段借不到显存（卡池此刻借不到、卡上报 Out of Memory）或叫停文件在（`crates/singlefs-checker-tier/src/crash_amplification.rs:8075`「let stop_was_requested = gpu_memory_was_short || stop_is_requested();」，叫停读 `crates/singlefs-checker-tier/src/crash_amplification.rs:1582`「std::env::var_os("SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE")」）。CPU 负载高不进这一支。叫停变量在环境守卫要清的那一张表里（第 7370 行起，含「"SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE",」），守卫开头整张清掉（`crates/singlefs-checker-tier/src/crash_amplification.rs:7458`「EnvironmentRestoredOnDrop::saved_then_cleared(ENVIRONMENT_VARIABLES_READ_BY_A_RUN);」），调用方环境里留下的叫停文件进不来。卡在而别的进程占着显存时会走到这一支、报本次未跑——这是照设计的「没判」，不是绿；门禁那一格先清场再跑，证红由主 agent 照清场规则排卡，`prove-red.sh` 把「点名的测试 ok 又打了 NOT_RUN_NO_GPU_ADAPTER」记本次未跑。三行 GPU 变异都不碰借显存，不会把自己变成「本次未跑」。
- 2 块与 8 块：流水线那条第二台只判第 1 块，配了 GPU 时第 ① 段 GPU 判器一份、对照（CPU 判器）在旁边那份库里判同一块、判完导回主库一份，共 2 块；对照默认开，关它的变量在同一张清掉的表里（第 7376 行「CROSS_CHECK_SKIPPED_BY_THE_USER_ENVIRONMENT_VARIABLE,」），这条流没挂观察者。给定状态表那条 4 张表各 GPU 判器、CPU 对照两块，共 8。两个数都另有前提断言把「GPU 真判了」钉住：`crates/singlefs-checker-tier/tests/crash_amplification_planned_import_names_the_gpu_conclusions.rs:229`「"前提：配置开了 GPU，第二台那一份在卡上判"」、第 237 行「"前提：配置开了 GPU，第二台那一份在卡上核"」、第 435 行「"前提：权重 0,1 下每张表都归 1/2，那一台在卡上判了 4 张表、8 个状态"」；错的配置（GPU 没判）在这几句先红，到不了 2 与 8。推的，没卡没跑。
- 三行 GPU 变异各红在哪（推的）：漏 GPU 判器那一种（打在第 7736 行），第 1 块导进来的只有 CPU 那一份，GPU 判器那一种对账剩 1 块，红在 `crates/singlefs-checker-tier/tests/crash_amplification_planned_import_names_the_gpu_conclusions.rs:268`「"导入之后 GPU 判器那一种一块没判的都没有：第二台那一块的 GPU 判器结论导进来了"」；漏 GPU 核对代码那一把（打在第 7714 行），第二台只在卡上核、库里没有 CPU 核对行，两把都是 CPU 时一行都导不进来，第 ③ 段拿 GPU 摘要比缺 1 块，红在第 286 行那一句；给定状态表点名只剩 CPU（打在 `crates/singlefs-checker-tier/src/crash_amplification.rs:8884`「conclusions: [JudgedBy::GpuJudge, JudgedBy::CpuJudge]」），不归本机的表导完库里没有 GPU 判器那一种，引擎报 `CRASH_POINT_REPLAY_GIVEN_STATES_TABLE_JUDGED_BY_NO_HOST` 起头的错（第 9414 行那一支），它不以 `CRASH_POINT_REPLAY_GIVEN_STATES_RUN_CONDITION_NOT_MET` 起头（两串在第 8422、8426 行，互不是前缀），用例走 panic 那一支（`crates/singlefs-checker-tier/tests/crash_amplification_planned_import_names_the_gpu_conclusions.rs:365`「"项 {GIVEN_STATES_ITEM} 判得了表 {}：{reason}",」），不吞成本次未跑。

## R5　改动范围与产品路径：没打中（读码 + 量过）

- Q1–Q4 点名的缺口在产品代码里原样就对、只差测试：流水线排键时每块点两把核对摘要（第 7770 行，配 GPU 时第二把是 GPU 的，第 7714 行）、两种判器各一把版本（第 7736 行起）；`imported_blocks` 数进已有且相同（第 7787、8910 行）；给定状态表点两种判器（第 8884 行）；换批、跨批编号、事实行补写由 `verdict_store.rs` 第 1785–1948 行做，原型五条在原样下全绿（`expected/base.txt` 第 36 行），连同本腿加的两条（一批多种新向量、导完经同一个库读回与再登记）。这一批没有一处该改产品代码而没改。
- 签名变化：公开入口 `pub fn import_the_planned_rows_from(`（第 1772 行）签名不变；新加的那个与 `write_the_planned_import_batch_when_full` 都是私有的，调用都在 `verdict_store.rs` 里。旧 `import_from`（`crates/singlefs-checker-tier/src/verdict_store.rs:1681`「pub fn import_from(&mut self, other: &VerdictStore) -> Result<u64, VerdictStoreError> {」）一行没动；旁库导回主库走的是它（`crates/singlefs-checker-tier/src/crash_amplification.rs:9851`「let imported = main_store.import_from(&self.store)?;」），不经按计划导入。按名字在冻结副本 `crates/` 下 grep 过全部调用方，别的 crate 没有。

## 放开批长扫（R3 补量，第二版装置）

用例 `research/prompts/m3-batch62-code-r1-opus-model/probe-crate/tests/batch62_store_probe.rs:422`「fn sweep_batch_lengths_against_one_batch() {」：两份对方库（B 加 F3；一块两种新向量那一份）× 52 档批长（1–40 字节逐个，再 64 字节起每档乘 1.5 到 8 KiB），每档从同一份关好的 A 拷目录再开、照这个批长导入，与照产品批长导入那一份比盘上字节，重开之后 F1–F3 按内容读回与对方库比。这一条把「批的边界落在哪一组之后」放开扫，只固定对方库与计划。

- 原样：两份对方库 52 档都与一批写完那一份相同，见过的批数 1–7 与 1–3 种（`research/prompts/m3-batch62-code-r1-opus-model/expected/base.txt:38`「OPUS_SWEEP shape=b-second lengths=52 batch_counts_seen={1, 2, 3, 4, 5, 6, 7} differing=0」，第 39 行「OPUS_SWEEP shape=two-in-one lengths=52 batch_counts_seen={1, 2, 3} differing=0」）。块内原子（一块一种结论的行不被批拆开）与跨批编号在这两种形状、这 52 档上都对，没打中。
- `take1`：B 加 F3 那一份 52 档都没看出来（`research/prompts/m3-batch62-code-r1-opus-model/expected/mut-take1.txt:39`「test sweep_batch_lengths_against_one_batch ... OPUS_SWEEP shape=b-second lengths=52 batch_counts_seen={1, 2, 3, 4, 5, 6, 7} differing=0」）：两种新向量落进同一批时后面再没有新向量，盘上照对；换成一块两种新向量那一份，重开就开不了（同一文件第 40 行「thread 'sweep_batch_lengths_against_one_batch' panicked at crates/opus-probe-b62/tests/batch62_store_probe.rs:457:68:」，第 457 行是重开那一句）。
- 表里三行：`take0` 在 B 加 F3 那一份上重开就开不了（`research/prompts/m3-batch62-code-r1-opus-model/expected/mut-take0.txt:44`「thread 'sweep_batch_lengths_against_one_batch' panicked at crates/opus-probe-b62/tests/batch62_store_probe.rs:457:68:」）；`flip` 只改批数、不改内容，两份都 `differing=0`，B 加 F3 那一份见过的批数只剩 1 与 7（`research/prompts/m3-batch62-code-r1-opus-model/expected/mut-flip.txt:54`「test sweep_batch_lengths_against_one_batch ... OPUS_SWEEP shape=b-second lengths=52 batch_counts_seen={1, 7} differing=0」）；`facts-after` 在这两份对方库上碰不到（本库没有现成的核对行），两份都 `differing=0`。
- `rev`：两份对方库 52 档都与一批写完那一份逐字节相同、重开读回也对（`research/prompts/m3-batch62-code-r1-opus-model/expected/mut-rev.txt:45`「OPUS_SWEEP shape=two-in-one lengths=52 batch_counts_seen={1, 2, 3} differing=0」）：只比盘上、只在重开之后读，怎么扫批长都看不见这一种，只有导完不重开、经同一个库读回那一种用例看得见。

第一版与第二版 `base` 逐行比的原样 `diff`（`expected-before-sweep/base.txt` 对 `expected/base.txt`）：

```
36c36,40
< test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in Xs
---
> test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in Xs
> sweep_rc=0
> test sweep_batch_lengths_against_one_batch ... OPUS_SWEEP shape=b-second lengths=52 batch_counts_seen={1, 2, 3, 4, 5, 6, 7} differing=0
> OPUS_SWEEP shape=two-in-one lengths=52 batch_counts_seen={1, 2, 3} differing=0
> test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in Xs
```

## 改法表（都是本腿提的：只在我的模型上量过、被攻过零轮）

| 改法 | 修哪一格 | take0（表 3990） | take1 | rev | 标注 |
|---|---|---|---|---|---|
| G1 Q3 用例导完、丢掉之前经同一个库按内容读回 F1–F3 每种结论，再登记一种新向量看编号，重开再比（`attack_in_memory_…` 的做法） | 跨批编号判别力 | 红 | 红 | 红 | 量过（`expected-before-sweep/mut-take0.txt`、`expected/mut-take1.txt`、`expected/mut-rev.txt` 里这一条的 panicked 行） |
| G2 对方库换一份：一块里两种新向量、下一批再一种，一字节一批与一批比字节、重开读编号（`attack_two_…` 的做法） | 同上 | 红 | 红 | **绿**（不修 rev） | 量过 |
| G3 放开批长扫（`sweep_…`） | 同上 | 红（B 加 F3 那一份就红） | 红（只在两种新向量那一份） | **绿**（不修 rev） | 量过 |
| G4 把 `take1`、`rev` 两种替换（`mutants.py` 里原文）追加进 `crates/mutations.tsv`，点名改过的 Q3 用例 | 让 G1 / G2 的判别力进表 | — | — | — | 推的（表里怎么写、证红归主 agent） |
| G5 流水线用例删掉事实行那半句或改注释 | R1-a | — | — | — | 推的 |
| G6 第 3989 行改名承认与第 3965 行重复，或换一个只在小批长下改行为的替换 | R3-b | — | — | — | 推的 |

要同时关 `take1` 与 `rev`，G1 一样就够；只做 G2 或 G3 关不了 `rev`。原样（产品代码）在 G1–G3 上都绿（`expected/base.txt`）。

## 没打中的形状

- R1：数了四趟各在哪台判、哪台核；按 `mv` 推到第一句断言红。没造「导进来的核对行内容错、有无对」的变异：这条流全绿、核对位全 0，改内容的变异在 (0,0,0) 那一句与读回那一句上都难分；(0,0,0) 那一句不含 `judge_red_only`（判器红、核对绿），照第 ③ 段的设计那一类只计数，不在第 62 批的改动里，没往下攻。
- R2：「只数写下的」「只数已有且相同的」「连对方库里没有的也数」三个计数邻居逐个推到红在哪一句；给定状态表四元组与「交回的结论逐张相同」两句找正确实现下会红的输入，没找到。都是读码。
- R3：五种变异（表里三行加两个邻居）× 原型五条，第一版全量、两趟一致；放开批长扫 52 档 × 两份对方库，第二版六个阶段都跑。被判的三行变异都红在点名那一条（`flip` 红在产品批长那一句，见 R3-b）。28 行 / 7 批用账与盘上差两条路对上。私有批长按名字 grep 全部调用方。
- R4：门禁那一格的判法、`stopped_early` 的来源、叫停变量与对照开关是不是被守卫清掉、两串前缀互不是前缀、三行 GPU 变异逐个推到红处，都是读码；没卡，一行没跑。
- R5：按名字在冻结副本 `crates/` 下 grep 三个导入入口与批写函数的调用方；产品代码原样在原型五条与扫批长上都绿。
- 取样范围：对方库三种形状（B、B 加 F3、一块两种新向量），本库一种（A），计划一种（F 不归本机的三块），批长 1 字节、52 档扫、64 MiB；没扫计划的形状（只点核对行之外的子集、重复的键）与多份对方库（C786 已记账）。

## 这条腿自己的限度

- 被判的三条单测没跑（checker 档）。镜像是照抄：库用公开接口照原形状重建，计数向量用的是 `BlockCountVector::default()`（原文是 `count_vector_of`）、版本与摘要的字节不同，行数、批数、编号与原文一致（28 行 / 7 批、21 行 / 1 批都对上）。「被判用例在 `take1`、`rev` 下照绿」「`flip` 先红在产品批长那一句」是在镜像上量的，搬到被判用例上是推的。
- 流水线与 GPU 两条（R1、R2、R4）只读码；R1 主问的红处引的是上一轮判决复跑的 h02 数，这一轮没在被判用例上量。
- 第一版六个阶段两趟一致；第二版六个阶段各一趟，原五条那一段与第一版逐行相同（只差 filtered out 的计数），扫批长那一段没跑第二趟。
- 「产品上会怎样」那几句（违例报成另一种向量、判本机块时撞号）是按 `crash_amplification.rs` 的调用次序推的，没在流水线上造出来。
- 每一格只抽了这一次样；「没打中」的格按 `.claude/rules/three-way-inference.md` 要再抽一次才算。
- 开跑时与后半程机器满载（loadavg 50–120），编译慢，不影响判的东西；没量任何时长。

## 没做什么

- 没跑 checker 档包的任何测试、门禁阶段、证红脚本、变异整表，没起 GPU。
- 没改冻结副本与主工作区的 `crates/`；只写了这份报告、模型目录与草稿目录。
- 没判正推那一边的格；没替主 agent 采纳；副本上量的数不算入库装置上的数。
- 没读禁读清单里的文件。

## 要主 agent 跑的命令

1. 复跑核对（判决第三步照例做的那一种，不是补跑）：工作目录 `/home/fy5090/code/singlefs`，内存上限 16G、线程 15（脚本里 `OPUS_MEMORY_CAP`、`OPUS_THREADS` 的缺省就是这两个），编译目录先拷一份种子（例如 `cp -a --reflink=auto ~/.local/share/singlefs/work/m3-batch60-61-code-r1-verify/target <新工作目录>/target`），再：

```
bash research/prompts/m3-batch62-code-r1-opus-model/rerun.sh <新工作目录> all
```

   看末尾 compare 六行都是「一致」。跑的是原型 crate 的测试二进制，不是重型测试；机器满载时每个阶段的编译可能超过前台时限，照 `rerun.sh` 文件头分段起（prep、build 重起到 rc=0、probe）。
2. 若采纳 G1 / G2：改过的 Q3 用例与追加的变异行要在 checker 档上跑与证红，归主 agent（重型规则照旧）。

## 草稿目录

`~/.local/share/singlefs/work/m3-batch62-code-r1-opus/`：交回前删掉了编译目录 `target/` 与两份仓副本 `base/`、`mut/`，以及 tmpfs 上的库目录 `/dev/shm/opus-b62-run/`；留下 `out/`（第二版六份输出）、`tmp/`（编译与运行日志）与根上改报告时用的四段文字、`mut.mutant`（共约 192K）。
