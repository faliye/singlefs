# E170 第 1 次重跑跑前登记：O3规约执行器第一版（严格跑法重跑）

写于 2026-10-03，装置写之前。原登记 `research/prompts/e170-preregistration.md`（下称原登记）。重跑不占号。

## 一、问题

英文名：second_run_o3_spec_executor

<!-- doc-lint:not-numbers O1 O2 O3 Q1 Q2 Q3 Q4 G01 G02 G03 G04 G19 G20 G21 G1 G2 G5 G6 P1 P2 P3 P4 P5 P8 P9 P10 N1 N2 N3 N4 N5 M2 V1 V2 V2b V3 V4 V5 V7 V8 V9 V10 S1 S2 S3 S4 S5 S6 F1 F2 F4 F5 F6 F7 F9 F10 F11 A1 A2 A3 A4 A5 A6 A7 A8 A9 A10 A11 A12 A13 X1 X2 X3 X4 X5 X6 X7 X8 X9 X10 X11 X12 X13 X14 X15 X16 X17 X18 X19 X20 XC1 XC2 XE1 XE2 XE3 XE4 C1 C2 C3 C4 C5 F8 G14 S5b V6 V11 U1 U2 U3 U4 U5 U6 U7 U8 Y3 -->

英文名的来历：原登记的英文名是 `o3_spec_executor`。这次的执行器从第一次冻结的那一份拷出来另放一个目录再改（5.2），照 E155（每次持久化的写量：三种 fsync 形态与反事实上界）第二次跑 `e155_second_run_fsync_write_volume` 的先例另起新英文名；原执行器目录 `scripts/experiment-apparatus/e170_o3_spec_executor/`、原变异表 `scripts/mutation/tables/e170_o3_spec_executor.tsv` 与 `scripts/experiment-apparatus/replay.sh` 里 E170 的五行复跑（第 240–244 行）留着不动。导出装置不另起：仍是 `crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs`，只加一个新模式，原有 `dump`、`bad-states` 两个模式的输出不许变（第十一节 S5）。

主 agent 给的问题（派发提示原文逐字）：

> 要回答的问题：照问题单三行的候选定义，一种读法。
> 重跑已有实验：E170（O3规约执行器第一版），第 1 次重跑，写 research/prompts/e170-r1-prereg.md。第一次的登记 research/prompts/e170-preregistration.md 与实验页 .claude/kb/experiments/170-O3规约执行器第一版.md 当定义读（它们记着冻结的执行器版本、批 1–3、坏状态表 P1–P9 与盲测族、导出装置与导出文件）。
> 要写死的：执行器解冻一次——只照今天的 kb 改（.claude/kb/layout/01-first-txn.md 的 magic 取值与「行序即盘上顺序」、.claude/kb/decisions/22-单元原子性怎么合成.md 已定项 16 全池没有可用槽 0 时逐档那一句与两个出口），改完再冻结、记新的 sha256；写执行器的执行员照第一次登记 5.7 的禁读清单（不读 crates/）；P8 的期望怎么改正、依据哪一句登记读法（第一次登记读法 3：F 与 Mk 上末次调用的返回点在 m(c) 之后、ack = n − 1、v_{n−1} 在 A(c) 里）写清，并写明这是登记修订、在产物之前；导出文件（scripts/experiment-apparatus/target/e170-dumps/，不进 git）还能不能直接用、要不要用今天的导出装置重导（第 11 批改了 mkfs，每盘多一步写零，导出的录制流会变）。
> 被测条款：.claude/kb/decisions/13-验证路线.md「已定项 6」；.claude/kb/decisions/22-单元原子性怎么合成.md「已定项 16」；.claude/kb/layout/01-first-txn.md「系统配置槽 0 与槽 1 的位置」那一段。

问题单 `research/prompts/e170-r1-questions.md` 第 7–11 行（`scripts/kb/quote-kb.py` 整段抄）：

**出处 `research/prompts/e170-r1-questions.md:7-11`（整段抄，未转述）**

```markdown
| # | 问题 | 候选（各自的定义） | 翻面观测 | 够判条件 | 状态 |
|---|---|---|---|---|---|
| Q1 | kb 补上 G01–G03 之后，只照 kb 文字写的执行器能不能对第一次跑的批 1 每个状态判出「在 / 不在」 | **判得出**：执行器照今天的 kb 改（解冻一次，改了哪几步、依据哪几句 kb 逐条写），严格跑法在批 1 每个状态上都给出在 / 不在，「规约欠一句」0 态。**判不出**：仍有态撞「规约欠一句」，逐条列出新的欠一句 | 一态撞「规约欠一句」 | 批 1 两条流每个状态严格跑法跑完 | 开着 |
| Q2 | 严格跑法给出的判定与实现一不一致 | **一致**：批 1 上执行器判「在」的与实现都放行的逐态相同。**不一致**：至少一态不同，逐态列出并判谁错 | 一态不同 | 批 1 逐态比完 | 开着 |
| Q3 | 坏状态表照改正的 P8 期望重判，执行器有没有判别力 | **有**：P1–P9 每一行（P8 照改正后的期望）与盲测族都判对。**没有**：至少一行判错 | 一行判错 | 坏状态表每一行在严格跑法下都跑过 | 开着 |
```

### 读法写死

1. **Q1 的「第一次跑的批 1」**：`scripts/experiment-apparatus/target/e170-dumps/<流名>/states.tsv` 里批号为 1 的态（两条流）。这份导出 20 个文件的 sha256 与实验页「导出文件的 sha256」一节的表逐个相同（2026-10-03 现核，第十三节命令，20 个 OK）；执行器只读这份中性格式，不依赖今天的 `crates/`，所以 Q1 直接用它，不等重导。
2. **Q2、Q3 用今天的导出**：今天的导出装置把两条流重导到 `scripts/experiment-apparatus/target/e170-second-run-dumps/`（不进 git，`dump` 模式拒绝写进已存在的目录，不会盖掉第一次的导出），取样规则照原登记 5.4 原样。「批 1」在 Q2 里指这份重导的批 1。重导的八个不带耗时的文件（`base-sectors.bin`、`ops.tsv`、`ops-payload.bin`、`calls.tsv`、`torn.tsv`、`torn-payload.bin`、`states.tsv`、`quick-tier.tsv`）与第一次的逐字节相同时，两份批 1 是同一批态，Q1 与 Q2 说的是同一批；不同时两份各报，Q1 照第 1 条取第一次的，重导那一份的严格跑法计数当几何点（第八节 G6）。
3. **「实现」= 重导那一刻工作区里的 `crates/`**。为什么要重导：第一次导出（目录日期 2026-10-01）早于第 11 批合入（2026-10-02，C651 的逐档进了恢复与池级 checker），`impl-verdicts.tsv` 是那时的实现给的判定；Q2 问的是与今天的实现一不一致。重导时在产物里记 `git rev-parse HEAD`、`git diff --cached -- crates/ | sha256sum`、`git diff -- crates/ | sha256sum` 三行，认得出是哪一份实现。
4. **版本、A(c)、L(c)、在 / 不在**：照原登记第一节读法 2、3、4、5 原样（读法 3 在第二节整段抄了），不改。
5. **严格跑法与暂代跑法**：照原登记 5.6。欠一句表 `spec-gaps.tsv` 沿用第一次的 G04–G20 行原样（行文、类别、暂代读法、触发条件都不改）；G01、G02、G03 三行按 5.2 改成「已由 kb 补」、不再触发；写执行器时新发现的欠一句从 G21 起编。Q1、Q2、Q3 的判定一律取严格跑法；暂代跑法照跑，只当附带。
6. **「判对」**（Q3）：一行的严格跑法判定等于这一行的期望判定（5.5：P1–P5、P9 与盲测族期望「不在」；P8 改正后期望「在」）。落点另报（Q3-d），不进「判对」。严格跑法判「规约欠一句」的行不算判对、也不算判错，单报个数，那几行 Q3 记「不够判」。Q2 同理：严格跑法判「规约欠一句」的态不进逐态比，单报个数，那几态 Q2 记「不够判」。
7. **实现都放行**（Q2）：照原登记读法 6 原样。
8. **执行器解冻一次**：只许改 5.2 表里列的那几步，每一处在代码旁写依据的 kb 句子（`文件@小节标题`，layout 那几行写文件加行号）；别的地方哪怕今天的 kb 也有了新句子，不改，记进 Q1-f。改完冻结（5.6），冻结之后再改照原登记 5.6 最后一条「看过实现判定之后改执行器」办。
9. **独立性边界**：照原登记读法 8；写执行器的执行员的禁读清单换成本登记 5.7 那一份。

## 二、被测条款与它引的定义

被测条款三处：D13（验证路线） 已定项 6（O3 那一行与射程）；D22（单元原子性怎么合成） 已定项 16（读槽 1 的偏移那一句，连同逐档与两个出口）；`layout/01-first-txn.md`「系统配置槽 0 与槽 1 的位置」那一段（不是标题，第 160 行一整段）。执行器这次要照着改的另外两样定义也抄：`layout/01-first-txn.md` 四种 magic 的行（第 109、175、338、369 行）与四处「行序即盘上顺序」（第 158、216、359、365 行）。P8 的期望与锚点 A1 的改正依据原登记读法 3，原登记那三行（读法 3、5.5 的 P8 一行、第七节 A1 一行）一并抄。下面由 `scripts/kb/quote-kb.py` 整段抽出，回读逐字节一致（第十三节命令）。

**出处 `.claude/kb/decisions/13-验证路线.md:130-155`（整段抄，未转述）**

```markdown
#### 已定项 6：三个互相独立的 oracle

**定案**：三个 oracle 各判一样东西，独立性各有来处：

<!-- doc-lint:registry name-col=2 -->

| # | oracle | 判什么 | 独立性来自哪里 | 验不到什么 |
|---|---|---|---|---|
| O1 | 内存模型对拍（HashMap 理想文件系统） | POSIX 可见语义：内容、大小、目录结构；无崩溃处的内容期望归它，崩溃之后允许看到的集合归 RefFS（已定项 3） | 不碰盘、不解析磁盘格式 | 磁盘结构是否自洽；崩溃语义；**两份实现对同一条 POSIX 语义的共同误解** |
| O2 | 独立解析器 + checker | 磁盘字节是否满足 I-1（块自描述）~I-6（加密不变量） | **checker 不得调用被测 crate 的任何反序列化/遍历代码**，必须重新实现一份解析，直接读裸设备 | 只判结构自洽：一个块被覆盖成另一份**合法**内容，Merkle 链照样自洽、记账照样对 |
| O3 | 独立规约执行器 | 崩溃点截断后的每个磁盘状态是否落在规约允许的集合里 | **用另一种语言写**（不用 Rust），只照 decisions 与 invariants 的文字重新推导；第一版用 Python 单写（E170（O3规约执行器第一版）） | 规约本身可能写错——**必须配阳性对照才有意义** |

**独立性要被验证，不许自称**：设计一个「三 oracle 合谋测试」——拿一个精心构造的坏实现
（例如 nonce 永远为 0）去过三关，若三者全部放行，说明存在共同盲区，回头补 oracle。
本工程功能正确性的全部故事就是这三个 oracle。

**射程**：三个 oracle 各自验不到什么写在表的最后一列；O2（独立解析器 + checker） 的定义域归已定项 7，需要第二个输入的核对归记录核对器。**实现今天的样子**：O2（独立解析器 + checker） 在 `crates/singlefs-checker`（只依赖格式常量 crate）；O1（内存模型对拍（HashMap 理想文件系统）） 今天的形态是 `crates/singlefs-harness/src/model.rs` 的理想模型（只住内存、只用 `singlefs_format` 的常量、不建崩溃），门禁 harness-model-differential-and-scenarios 拿随机历史逐步对拍；O3（独立规约执行器） 的第一版是只照 kb 文字写的 Python 执行器 `scripts/experiment-apparatus/e170_o3_spec_executor/`（E170（O3规约执行器第一版））：严格跑法撞「规约欠一句」判不出，暂代跑法与实现在 972 个取样态上一致（两边全放行，不当它判得对的证据），阳性对照照字面没有判别力（26 行判「在」4 行，原因在跑前登记的读法与那一行的期望之间），盲测 40 行全判「不在」，还没接进崩溃点重放；RefFS（已定项 3）的第一版在 `crates/singlefs-reference-filesystem`，调用点没接；「三 oracle 合谋测试」没有（`grep -rn 合谋 crates` 零命中）。

**依据**：
- E170（O3规约执行器第一版）：O3（独立规约执行器） 照「另一种语言、只照 kb 文字重新推导、配阳性对照」写得出第一版，判得动单个崩溃状态、一态够进快档；严格跑法因规约欠一句判不出、阳性对照照字面判别力不足，这两样它撑不住。
- 三个 oracle 的分工与各自独立性的来处是验证体系的结构设计；独立性要靠「三 oracle 合谋测试」观测，那个测试还没有。
- 三方论证：D13（验证路线） 新立那一轮（本地 LLM / Sonnet / Opus 各一条腿，立场不重复），材料没有留存，经过在变更史。
- O1（内存模型对拍（HashMap 理想文件系统）） 与 RefFS 的分工句：三方判决 `research/prompts/m3-formal-r5-main-verification.md` 第二节另起与单独一个包那一格（两腿同向）。

**欠**：C445（模型对拍的取样点不随写路径增长）；C446（未实现的不变量没有前置登记，也没有到期）；C447（独立规约执行器与 RefFS 都零代码）；「三 oracle 合谋测试」没有 C 编号（C407（三 oracle 合谋测试没有代码） 记着它零代码，那一格本身仍没有编号）。

```

**出处 `.claude/kb/decisions/22-单元原子性怎么合成.md:330-354`（整段抄，未转述）**

```markdown
#### 已定项 16：系统配置槽轮换规则；根环区域公式的地址空间与区域内槽位

**定案**：系统配置槽：每盘恒 2 个槽，槽 i 的设备内偏移 = i × 固定结构槽距（D2（RAID 条带策略） 已定项 19），槽世代号从 1 起（mkfs 写下的两槽世代号都是 1）、每写一次 +1，下一次写的槽 = 世代号 mod 2，择槽取校验和过且世代号最大的（逐盘计：取号、发布末尾的轮换、取号全或无失败时的回卷三种系统配置写都写这块盘上自证过的槽里最大的世代号 + 1）。读槽 1 的偏移按这个次序取：本盘槽 0 可用（自证过、几何字段都在这一版读者收的范围里）时取它自述的槽距；否则取池里按设备次序第一块可用槽 0 自述的槽距；全池没有可用的系统配置槽 0 时，读者在格式允许的每个槽距上各读一槽（不小于 4096、512 的整数倍、不越过同一读者收的根环起点减 4096），只收自证过、且自述的固定结构槽距等于所在偏移的槽，几档都收得到时取世代号最大的，世代号相同取偏移小的；逐档读到自证过而池级字段越界的槽整池拒（第 5 句「越界即拒」），incompat 位不认得的槽不可择、不核偏移。根环：`r × P × chunk` 是设备内偏移，基址 = 系统配置「根环起点」字段（第一版 1 MiB），区域 r 的起点 = 基址 + r × P × chunk；区域内槽位 = `(txg div R) mod S`，无状态、只由 txg 与 (R, S) 决定，升格进已定项 2 的参数表。五句写死：

1. **系统配置「根环起点」字段是 16 KiB 槽号**，与 D3（空间分配） 已定项 7 的落点槽号、D19（块指针的结构与宽度预算） 已定项 4 的位置条目物理偏移同一个坐标系；第一版写 **64**（= 1 MiB ÷ 16384）。基址（设备内字节偏移）= 该字段 × 16384；区域 r 的起点 = 基址 + r × P × chunk。⚠️ 字段里写的是 64，不是 1048576。读者解槽时读这个字段与 journal 环起点字段（偏移 325），与第一版常量（64、1024）不等一律整池拒；判固定结构槽距的上界用同一槽自述的根环起点，checker 的 I-7.13（系统配置池级字段在读者收的范围里） 与 core 收同一张表（用户定，起因与依据在代码三方第二轮判决 `research/prompts/m2-closeout-code-r2-main-verification.md`「core 与 checker 对系统配置几何字段的读法」那一格）。已实现：core 的 `crates/singlefs-core/src/system_configuration.rs` `parse_slot` 按偏移 325、371 读进 `SystemImmutableConfiguration` 的 `journal_ring_start_slot`、`root_ring_base_slot`；`crates/singlefs-core/src/recovery.rs` 的 `system_configuration_values_this_reader_accepts` 择系统配置时判，不等报 `SystemConfigurationValueOutsideWhatThisReaderAccepts::RootRingBaseNotTheFirstVersionSlot` / `JournalRingStartNotTheFirstVersionSlot`，槽距上界取 `largest_fixed_structure_slot_spacing_under_the_root_ring_base`（根环起点 × 16384 − 4096；逐档找槽 1 那一处还没有哪一槽可读，仍用第一版常量）；checker 的 `crates/singlefs-checker/src/image.rs` `geometry_of` 判同一张表（325、371 等于第一版常量，417 = 环长现算的起点且在 64 槽段边界上）。用例：`crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs` 的 `a_system_configuration_field_the_reader_does_not_accept_is_refused_by_both_mounts_before_any_write`（攻方五格只读与可写都拒、盘上不变）；`crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `each_system_configuration_geometry_field_the_reader_refuses_reddens_only_its_own_invariant_naming_the_refused_field`；两边在同一份镜像上一起判的是 `crates/singlefs-harness/tests/system_configuration_fields_core_and_checker_refuse_the_same_table.rs`（五格各一条：core 两种挂载都整池拒、盘上不变，checker 报 I-7.13（系统配置池级字段在读者收的范围里） 并点名它拒的成员；根环起点写 0 那一格两边报的成员不同——core 先判槽距上界、报槽距越界，checker 先判两个起点、报根环起点）（`research/prompts/m2-impl-r2-fixes-b-implementer-report.md` 第三节第 2 行、`research/prompts/m2-impl-r2-fixes-a-implementer-report.md`「结论」第 1 行）。
2. **根槽序号 j 的设备内偏移 = 区域起点 + j × 固定结构槽距**（槽距 = max(4096, mkfs 时探测的 io_min)，D2（RAID 条带策略） 已定项 19）。按已定项 2 的字面取槽距 512 也合规，而字节表要新池新建文件的根落在 1 MiB + 4096，这一句把序号到偏移的换算钉死。
3. **P = 3、chunk = 1 MiB 是第一版的 mkfs 常量**；mkfs 要校验 `基址 + (R−1) × P × chunk + S × 槽距 ≤ 设备可用字节`，越界拒绝 mkfs。取 3 是第一版两盘下满足「P 素数且 P > devs」的最小素数，chunk 取 1 MiB 与基址同档 ⇒ 三个区域起点落在 1 / 4 / 7 MiB，与 [layout/01-first-txn.md](../layout/01-first-txn.md) 的字节表一致。⚠️ **「P > devs」那条约束今天已经没有理由**（它唯一的理由是 `gcd(P, devs) = 1` 的双射，已随 D2（RAID 条带策略） 已定项 7 把归属改成 mkfs 逐区域写下的设备身份而作废），账在 C217（P 大于设备数这条约束的理由没了而约束还在）；取 3 不依赖那条约束。
4. **根环三个区域的设备归属第一版写死 0 / 1 / 0**（区域 0 → 盘 0、区域 1 → 盘 1、区域 2 → 盘 0），mkfs 逐区域把设备身份写进系统配置的「根环逐区域设备身份」12 字节（D2（RAID 条带策略） 已定项 7）。⚠️ **它不是 mkfs 参数**：第一版 devs = 2、R = 3，按 D22（单元原子性怎么合成） 已定项 14 的鸽笼下界（取满 min(R, devs) = 2 个不同值、没有一块盘背超过 ⌈3/2⌉ = 2 个区域），只有「两个区域在一块盘、一个在另一块」这一种形状，区域编号一定之后 0 / 1 / 0 是唯一的规范写法。把它做成 mkfs 参数，买到的是「哪块盘背两个」这一格自由度，而那一格在两盘同构下没有可观测差别，代价是多一个要在挂载时校验、要在加盘时重算的参数。
5. **暖机空发布 2 次、新池新建文件 txg 3 是格式常量**，不是运行时谓词（`format-const` WARM_UP_EMPTY_PUBLISHES = 2、NEW_POOL_FILE_CREATION_TXG = 3，登记在 [layout/01-first-txn.md](../layout/01-first-txn.md) 八）。D16（发布语义） 已定项 8 要求「推到本实例的根覆盖两块盘为止」，而在 0 / 1 / 0 的归属下 txg 1 落区域 1（盘 1）、txg 2 落区域 2（盘 0），两次正好覆盖两块盘 ⇒ 次数由归属唯一确定，写成常量比写成运行时谓词少一条只有一个取值的分支。⚠️ 归属改了这两个常量要重算（加盘或开条带表时）。`NEW_POOL_FILE_CREATION_TXG` 只管 mkfs 那条流（mkfs → 取号 → 暖机两次 → 新池新建文件，同一个进程里那一串），不管任何池的第一个文件版本：只做过 mkfs 的池重开之后再写第一个文件版本，txg 取现行根 + 1。读者判固定结构槽距：≥ 4096，且槽 1 整槽落在同一槽自述的根环起点之前；`physical_block_size` ∈ [根记录宽 457, 槽距]。越界即拒：checker 对越界槽报 I-7.13（系统配置池级字段在读者收的范围里）、该池其余不变量报不适用，实现整池拒挂载（用户 2026-09-27 定「整池拒」）。

**射程**：把区域公式换成「mkfs 逐区域写起点」那条留到 zoned 开线前再走三方。全池没有可用槽 0 时按第一版常量 4096 读槽 1 的那一读（用户 2026-09-14 定的「甲」）由用户 2026-10-02 推翻、改成逐档，原话在变更史；「世代号相同取偏移小」那一句在 mkfs 清零 [0, 根环基址)（已定项 8 第 3 条）之后只在介质损坏造出的残留上有对象。池级 checker、恢复与 GPU 判器的两份按同一句读（[layout/01-first-txn.md](../layout/01-first-txn.md) 一「系统配置槽 0 与槽 1 的位置」那一段）。

**依据**：

- E142（新池新建文件的干跑）：按这套取法写出新池新建文件的全部固定结构字节（系统配置槽 0 / 4096、根环 1 / 4 / 7 MiB、新池新建文件的根落区域 0 槽 1），与 `crates/` 实装按 21 个区域比对全等（根记录、journal 记录与系统配置逐字节全比）。
- E87（固定结构的放置）：两盘上 R = 3 必有一盘背两个区域——0 / 1 / 0 是唯一规范写法的那一半。
- 逐盘计那一读法：C322（取号那一步的屏障怎么放没有条款） 三轮三方之后的用户定案（全池最大 + 1 会让滞后盘一次跳 2、落回它择到的那一槽），判决在 `research/prompts/c322-r3-main-verification.md`。
- 用户定案（按 [records/2026-09-13-总审核.md](../../../records/2026-09-13-总审核.md) 第九节的选项逐条定；第 4 / 5 句是之后的弹窗定案），原话在变更史；除逐盘计那一句，都没有走三方论证，可推翻。
- 用户定案 2026-09-23：`NEW_POOL_FILE_CREATION_TXG` 只管 mkfs 那条流（`research/prompts/m2-refusals-presumed-r1-main-verification.md` 第三节判它射程的那一格），原话在变更史。
- 用户定案 2026-10-01：「mkfs 两槽世代号都是 1」写进这一项，I-7.16（系统配置槽世代号与槽号的关系） 一并判上（`research/prompts/m3-inv-b-r1-main-verification.md` 第四节第 1 条），原话在变更史。
- 全池没有可用槽 0 时逐档、平局取法与两个出口：三方判决 `research/prompts/m3-c651-r1-main-verification.md` 第二节（常量与整池拒两读出局、逐档读法站住、平局取法站住）与第四节第 1、2 条，`research/prompts/m3-formal-r6-main-verification.md` 第二节条款句两个出口那一格；用例 `crates/singlefs-harness/tests/system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery.rs`（恢复与池级 checker 在同一份镜像上判逐档、自述槽距不等于偏移不收、取世代号最大、同号取偏移小、越界整池拒、incompat 不认得择不到）。
- 用户定案 2026-10-02：逐档连同前两步与平局取法写回；全池没有可用槽 0 时改成逐档，推翻 2026-09-14 收尾弹窗定的「甲」（按最小槽距 4096 读槽 1），原话在变更史。

**欠**：C217（P 大于设备数这条约束的理由没了而约束还在）。

```

**出处 `.claude/kb/layout/01-first-txn.md:160-160`（整段抄，未转述）**

```markdown
**系统配置槽 0 与槽 1 的位置**：槽宽都是 4096；槽 0 在设备内偏移 0，槽 1 在设备内偏移 = 固定结构槽距（偏移 429 起那 4 字节，小端，第一版 4096）。槽距住在槽自己里，读槽 1 之前要先有一个读得出槽距的槽 0。读法照 D22（单元原子性怎么合成） 已定项 16：逐盘先读槽 0；「可用」的槽 = 自证过（magic、整槽校验和、格式版本、incompat 位、加密类型都过）且几何字段都在这一版读者收的范围里。槽 1 的偏移按这个次序取第一个有的：① 这块盘的槽 0 可用时，取它自述的槽距；② 否则取池里按 `reader.devices()` 次序第一块槽 0 可用的盘自述的槽距；③ 全池没有一个可用的槽 0 时逐档：在 4096 到 1 MiB − 4096 之间每 512 字节一档各读一槽，只收自证过、且自述的槽距正好等于所在偏移的槽，几档都收得到时取世代号最大的，世代号相同取偏移小的；读到自证过而池级字段越界的槽整池拒，incompat 位不认得的槽不可择、不核偏移。③ 这一格池级 checker 与实现已统一成逐档：checker 是 `crates/singlefs-checker/src/image.rs` 第 631 行 `system_configuration_slot_readings` 调第 583 行 `slot_one_read_at_every_slot_spacing_the_format_allows`，实现是 `crates/singlefs-core/src/recovery.rs` 第 1039 行 `choose_system_configuration` 调第 967 行 `slot_one_found_by_trying_every_slot_spacing_the_format_allows`；GPU 判器里恢复那一份是 `crates/singlefs-checker-tier/src/crash_judge_common.wgsl` 的 `slot_one_by_trying_every_spacing`，池级 checker 那一份是 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` 的 `checker_slot_one_by_reading_every_spacing`。两边在同一份镜像上一起判的用例是 `crates/singlefs-harness/tests/system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery.rs`。E170（O3规约执行器第一版） 的执行器还是暂代读法（收到第一个就停、不比世代号），没改到同一句。mkfs 写元数据之前把每块盘 [0, 根环基址) 整段写 0（D22（单元原子性怎么合成） 已定项 8 第 3 条），用过的盘上重做 mkfs 不留旧池的系统配置槽。
```

**出处 `.claude/kb/layout/01-first-txn.md:109-109`（整段抄，未转述）**

```markdown
| 自举头 | magic | 4 | 4 字节 ASCII `SFSB`，十六进制 53 46 53 42，从偏移 0 起按此顺序排（字节串，不涉字节序）；取值出处 `crates/singlefs-core/src/system_configuration.rs` 第 22 行 `SYSTEM_CONFIGURATION_MAGIC`，checker 自写的一份 `crates/singlefs-checker/src/lib.rs` 第 192 行 | 骨架 |
```

**出处 `.claude/kb/layout/01-first-txn.md:158-158`（整段抄，未转述）**

```markdown
**系统配置字段表的偏移**：行序即盘上顺序，偏移按行累加，从槽首字节起算，整数字段一律小端（checker 读整数用 `from_le_bytes`，`crates/singlefs-checker/src/lib.rs` 第 228–241 行的 `read_u16` / `read_u32` / `read_u64`）：magic 0 / 格式版本 4 / feature bits 6 / fsid 102 / 写入者身份 118 / 校验和算法标识 138 / 本盘设备号 139 / 设备数 devs 143 / 槽世代号 147 / 整槽校验和 155 / 系统配置自身 MAC 187 / nonce 水位 203 / KDF 标识 215 / 加密类型 219 / MAC 长度声明 220 / 主密钥槽 221 / 节点大小 301 / 单元大小 305 / 落点粒度 309 / 位置条目宽度 313 / mkfs 时的 physical_block_size 317 / 扩展点声明值 N 321 / journal 环起点 325 / journal 环长度 333 / journal 最大记录字节 341 / journal 在飞记录数上限 345 / journal 最坏占用 349 / journal 安全系数 F 357 / 根环区域数 R 361 / 每区槽数 S 362 / 素数步长 P 363 / 根环 chunk 367 / 根环起点 371 / 根环逐区域设备身份 379、383、387（每区 4 字节）/ 条带宽度上界 w_max 391 / 组大小 g 392 / 映射来源 393 / 单元区起始槽号 417 / mkfs 时的 io_min 425 / 固定结构槽距 429 / T_time 433 / T_dirty 437 / 整理低水位 445、高水位 453、停止线 461 / journal tail 469 / journal 实例代号 477 / 回退下界 F 481，合计 489。checker 自己读的那几个偏移——6、102、139、143、147、155、219、317、325、333、361、362、363、367、371、379、383、387、417、429、469、477、481——现查 `crates/singlefs-checker/src/lib.rs` 第 195–203 行、第 224 行、第 278–280 行与 `crates/singlefs-checker/src/image.rs` 第 245–304 行，逐个与这里按行累加的结果相同；没有一处不一致。
```

**出处 `.claude/kb/layout/01-first-txn.md:175-175`（整段抄，未转述）**

```markdown
| 单元头 | magic | 4 | 4 字节 ASCII `SFSU`，十六进制 53 46 53 55，从偏移 0 起按此顺序排（字节串，不涉字节序；码 1 / 2 / 3 三类单元同一个值）；取值出处 `crates/singlefs-core/src/unit.rs` 第 14 行 `UNIT_MAGIC`，checker 自写的一份 `crates/singlefs-checker/src/lib.rs` 第 194 行 | D18（块里携带什么信息） 已定项 7 | 已定 |
```

**出处 `.claude/kb/layout/01-first-txn.md:216-216`（整段抄，未转述）**

```markdown
**位置条目（14 字节）的内部**：行序即盘上顺序，偏移按行累加：设备 ID 0（4 字节）/ 16 KiB 槽号 4（6 字节）/ 密文校验和 10（4 字节），整数小端。一个指针里第一条副本从指针偏移 50 起、第二条从 64 起（头部 50 + 14）；journal 点名项里第一条在点名项偏移 0、第二条在 14。现查 checker：`crates/singlefs-checker/src/image.rs` 第 634–640 行 `location_at`（设备 ID 在 +0、槽号在 +4、校验和在 +10）与第 673、687 行的 `location_at(bytes, 50)` / `location_at(bytes, 64)`，`crates/singlefs-checker/src/lib.rs` 第 806–814 行点名项的 `location(0)` / `location(14)`；与这里相同。
```

**出处 `.claude/kb/layout/01-first-txn.md:338-338`（整段抄，未转述）**

```markdown
| magic | 4 | 4 字节 ASCII `SFSJ`，十六进制 53 46 53 4A，从偏移 0 起按此顺序排（字节串，不涉字节序）；取值出处 `crates/singlefs-core/src/journal.rs` 第 20 行 `JOURNAL_MAGIC`，checker 自写的一份 `crates/singlefs-checker/src/lib.rs` 第 695 行 | D23（journal 的角色与格式） 已定项 4 |
```

**出处 `.claude/kb/layout/01-first-txn.md:359-359`（整段抄，未转述）**

```markdown
**记录头字段表的偏移**：行序即盘上顺序，偏移按行累加，从记录首字节起算，整数字段一律小端：magic 0 / 类型 4 / 算法类型 6 / 记录标志 7 / 记录长度 8 / 点名项数 12 / jsn 16（实例代号 4 字节在 16，计数器 6 字节在 20）/ checkpoint_txg 26 / nonce 34 / 头校验和 46 / 事务号 78 / 提交标记 86 / 本次发布内序号 87 / 反向链 91 / 载荷校验和 95 / 新根段 99（树表单元指针 99、中央映射树根指针 185、树 ID 水位 271、回退下界 F 279）/ fsid 287 / MAC 295，到 311 为止；前十个字段是 magic 到头校验和，合计 78。现查 checker：`crates/singlefs-checker/src/lib.rs` 第 696–704 行的偏移常量与第 778–833 行读的偏移（类型 4、记录长度 8、点名项数 12、jsn 的 16 与 20、checkpoint_txg 26），逐个与这里按行累加的结果相同；没有一处不一致。
```

**出处 `.claude/kb/layout/01-first-txn.md:365-365`（整段抄，未转述）**

```markdown
**里程碑**：[01-first-txn.md](../milestone/01-first-txn.md) 步 5（mkfs 种下的第 0 代与树表单元第 0 版归步 1）。行序即盘上顺序（D22（单元原子性怎么合成） 已定项 7 的表序，两处的字段先后逐行相同），偏移按行累加，各字段的偏移数写在 D22（单元原子性怎么合成） 已定项 7 字段表之后那一句。
```

**出处 `.claude/kb/layout/01-first-txn.md:369-369`（整段抄，未转述）**

```markdown
| magic | 4 | 4 字节 ASCII `SFSR`，十六进制 53 46 53 52，从偏移 0 起按此顺序排（字节串，不涉字节序）；取值出处 `crates/singlefs-core/src/root_record.rs` 第 11 行 `ROOT_MAGIC`，checker 自写的一份 `crates/singlefs-checker/src/lib.rs` 第 193 行 | D22（单元原子性怎么合成） 已定项 7 | 已定 |
```

**出处 `research/prompts/e170-preregistration.md:29-29`（整段抄，未转述）**

```markdown
3. **允许集合 A(c)**：c 里持久或撕裂的写中流序下标最大的那个记 m(c)；ack(c) = 返回点 ≤ m(c) 的调用里流序最后的那一版（一版都没有就是版本 0）；begun(c) = 至少有一次写在 c 里持久或撕裂的发布里流序最后的那一版（没有就取 ack(c)）；A(c) = 版本表里流序落在 [ack(c), begun(c)] 的各版的根投影。「返回点」= 驱动方记下的那次 API 调用返回时录制流的长度（5.3 的 `calls.tsv`）；一次调用里发了几次发布（暖机两次、抬 F、卸载），这几版的返回点都是这次调用的返回点。依据：下界是 D16（发布语义） 已定项 7「fsync 等系统配置轮换持久之后才返回」与已定项 8（新实例根覆盖两块盘之前不返回）；「只落在整版上」是 D16（发布语义） 已定项 4 与 D13（验证路线） 已定项 1 M2；上界取「落到的那一版至少要有一样写在盘上」，比「已发出的写」更紧，对合法的落点两者等价。坏状态（字节补丁）的 A(c) 按它的底态算，补丁不算写。
```

**出处 `research/prompts/e170-preregistration.md:497-497`（整段抄，未转述）**

```markdown
| P8 最新根与它那条记录都读不出（两处） | F、Mk | 根 n 偏移 122（树 ID 水位第一个字节）翻一位，不重封；记录 n 两份各在点名项区（或点名项为空时新根段）翻一位，不重封 | 不在：择到 n − 1，链断在计数器 c_n（D23 已定项 14、22），L = v_{n−1}，而 v_n 已确认（ack = n），v_{n−1} 在下界之下 |
```

**出处 `research/prompts/e170-preregistration.md:587-587`（整段抄，未转述）**

```markdown
- A1：流一 F 态，执行器落到 (实例 1, txg 3)，A(F) 恰一个元素、实例 1、txg 3（D22（单元原子性怎么合成） 已定项 16 第 5 句「新池新建文件 txg 3」；D23（journal 的角色与格式） 已定项 16 第一次可写挂载取 1）。
```

## 三、实现今天的样子

主 agent 与读 `crates/` 的执行员读；写执行器的执行员不读这一节（5.7）。行号是 2026-10-03 现查的。

- 读槽 1 的偏移（被测条款那一句）：恢复 `crates/singlefs-core/src/recovery.rs` 第 1039 行 `choose_system_configuration` 调第 967 行 `slot_one_found_by_trying_every_slot_spacing_the_format_allows`，池级字段收不收在第 741 行 `system_configuration_values_this_reader_accepts`；池级 checker `crates/singlefs-checker/src/image.rs` 第 631 行 `system_configuration_slot_readings` 调第 583 行 `slot_one_read_at_every_slot_spacing_the_format_allows`。择根第 1255 行 `choose_root`，恢复入口第 4330 行 `recover`。
- magic 常量：`crates/singlefs-core/src/system_configuration.rs` 第 22 行 `SYSTEM_CONFIGURATION_MAGIC` = `*b"SFSB"`，`root_record.rs` 第 11 行 `ROOT_MAGIC` = `*b"SFSR"`，`journal.rs` 第 20 行 `JOURNAL_MAGIC` = `*b"SFSJ"`，`unit.rs` 第 14 行 `UNIT_MAGIC` = `*b"SFSU"`。
- 层 0 判定：`crates/singlefs-checker-tier/src/crash.rs` 第 51 行 `check_records`、第 1222 行 `judge_crash_image`、第 1267 行 `judge_layer0_state_range`、第 1307 行 `layer0_state_ranges_by_segment`。
- mkfs 的写零：`crates/singlefs-core/src/make_filesystem.rs` 第 557 行 `device.write_zeroes_at(DeviceOffsetInBytes(0), root_ring_base.0)?;`。导出装置的录制流从 mkfs 之后起（`crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs` 第 543 行 `let operations = all_operations[mkfs_operation_count..].to_vec();`），基镜像是 mkfs 之后的池（第 549 行 `let base = pool.memory_pool_after_mkfs();`，`crates/singlefs-harness/tests/common/mod.rs` 第 204–205 行按 mkfs 那一截操作建池），层 0 也从 `mkfs_operation_count` 之后起枚举（`crates/singlefs-harness/src/history.rs` 第 2228 行注释）。
- 导出装置的模式：第 1852 行 `main`，第 1854 行认 `bad-states`，第 1689 行认 `dump`。P8 那一行在第 2688 行 `BadShape::NewestRootAndRecordUnreadable =>` 起，期望判定写 `"not_in"`、落点写第 2704 行 `ExpectedLanding::Version(facts.previous_version_text(newest.publish_index))`；Mk⁺ 底态第 2772 行 `let middle_plus = facts.mk_plus();`（是 `Option`），P9 在第 2805 行 `if let Some(plus) = &middle_plus {` 之下、第 2811 行 `"P9-Mk+"`；盲测每条流 20 行（第 1953 行 `BLIND_ROWS_PER_STREAM`）。
- 第一次冻结的执行器：`scripts/experiment-apparatus/e170_o3_spec_executor/` 20 个文件（不含 `__pycache__`），`git status --short` 全是 `A `（已暂存、工作区与暂存一致）；2026-10-03 的 sha256 存在草稿 `/tmp/claude-1000/e170-r1-designer/first-run-executor-sha256-2026-10-03.txt`（只当设计时的记录，执行员按第一次第二段产物的冻结行核，第十一节 V9）。
- `crates/singlefs-core`、`crates/singlefs-format`、`crates/singlefs-harness/src`、`crates/singlefs-checker/src`、`crash.rs` 与导出装置这几条路径里，自 HEAD 以来暂存了 33 个文件的改动（`git diff --cached --stat -- <这几条路径>` 末行 `33 files changed, 9782 insertions(+), 659 deletions(-)`，含新建的导出装置本身；命令见第十三节），其中有 `crates/singlefs-core/src/recovery.rs`、`transaction.rs`、`records.rs`、`mount.rs`、`make_filesystem.rs`、`crates/singlefs-format/src/lib.rs`。哪些在第一次导出之前、哪些在之后，从 git 里分不出（都没提交）。

## 四、跑之前已经存在的数

写执行器的执行员不读这一节（5.7）。设计时读到或自己算出的，照实列：

1. 实验页正文与原登记（主 agent 指定当定义读）里读到的第一次结果：Q1 判不出（严格跑法批 1 两条流 450 态全撞 G01、G02、G03；G03 撞 10 + 60 态）；暂代跑法 972 态全判「在」、实现全放行；Q2-a 0 / 450、批 2 0 / 126；Q3-a 4 / 26（四行全是 P8，落点与期望一致，都是 v_{n−1}）、Q3-b 0 / 40；A1 实测 `allowed=[(1, 2), (1, 3)]`、落点 (1, 3)；执行器变异 12 条 11 抓、X10 没红；种子 `8546b1258e427c34`；Q4 的几个用时。另从第一次的 `bad-expected.tsv` 读到 P8 四行的期望落点：流一 F (1,2)、Mk (1,1)，流二 F (2,18)、Mk (2,9)。**影响**：① 改正后的 P8 期望（5.5）与第一次实测的判定、落点逐个相同，P8 这一行在这次不是盲测，只当回归检查；Q3 的判别力靠 P1–P5、P9 与新种子的盲测族。② 改正后的 A1（第七节）与第一次实测的 A(F) 相同，同样只当回归锚点。③ Q1 的门槛（0）、Q2 的门槛（0）是问题单的定义，不从这些数来。
2. 调度记录 `records/2026-10-01-里程碑三形式化接入.md` 第 449–456、473、501、538–539 行（读到的数）：c651 正推腿说「E170 批 1 有 14 态是这一形」（全池没有可用槽 0）；「E170 Q3-f 的 28 / 66 是上界」；layout 那一段第 160 行写 O3 执行器的暂代读法是「收到第一个就停、不比世代号」。**影响**：Q2-e 的路径计数里「③ 逐档」那一项设计时预期大于 0，门槛照写「> 0」，不从 14 定；Q3-f 是附带，不判。
3. 判问法时自己推的：mkfs 新加的写零在录制流开始之前（第三节），新盘上写零不改任何不全为零的扇区，所以单凭这一处，`base-sectors.bin` 与录制流不会变（推的，没量）；与派发提示「导出的录制流会变」那一句不同。会不会变由重导的逐字节比对定（读法 2），这一句不进任何判据。暂存的 33 个文件里恢复与 checker 的改动会让 `impl-verdicts.tsv` 可以不同，这是要重导的理由（读法 3）。
4. 判问法时自己推的：在 F、Mk 底态上，读法 3 让 P8 落在 A(c) 里（推导在 5.5）；同一个补丁打在 Mk⁺ 底态上，读法 3 让它落在 A(c) 下界之下（P10，5.5）。P10 的期望是设计员照读法 3 推的，被攻过零轮。
5. 在派发之外读到的工作区事实：`scripts/experiment-apparatus/target/e170-dumps/` 两条流目录里除第一段的十个文件外还有第三段的五个 `bad-*` 文件；`scripts/experiment-apparatus/target/e170-dumps-v2-first-run/` 在。不影响判据。
6. CRC-32C 三个参考值（第七节 A8–A10）沿用原登记，2026-10-03 用本机 `google_crc32c` 复算一遍，相同（第十三节）。`printf 'SFSB SFSU SFSJ SFSR' | xxd -p` 的输出与 layout 写的十六进制逐字节相同（第七节 A12 的交叉核）。

## 五、臂、阳性对照、真实基线

### 5.1 臂

| 行 | 一边 | 另一边（写成支持它的人认的样子） |
|---|---|---|
| Q1 | **判得出**：照 5.2 改过的执行器，在第一次跑的批 1 每个状态上，严格跑法都给出在 / 不在，「规约欠一句」0 态 | **判不出**：改过之后仍有态撞欠一句（G04–G20 里某一行的触发条件成立，或改的那几步里冒出新的欠一句 G21 起）。支持它的人会要求：执行员只照 5.2 列的 kb 句子改，不为了凑 0 态顺手改别的步、不读 `crates/`；新冒出的欠一句照原登记 5.6 的三类记，「只有读 crates 才写得下去」那一类单报 |
| Q2 | **一致**：重导的批 1 上，严格跑法判「在」的态与今天的实现两判都放行的态逐态相同 | **不一致**：至少一态不同。支持它的人会要求比的是同一个镜像（摘要逐态相同，S2）、实现是今天的（读法 3，产物记三行 git 标识），执行器在看到实现判定之前已冻结 |
| Q3 | **有**：P1–P9（P8 照 5.5 改正的期望「在」）与新种子的盲测族，严格跑法每一行都判对 | **没有**：至少一行判错。支持它的人会要求 P8 的改正有登记读法可依且写在产物之前（5.5），盲测族的种子取这次冻结的执行器（写执行器的人造不出针对它的判法），并且有一行考下界的「不在」（P10，附带）补上 P8 改正之后空出来的那一格 |

**真实基线**：今天的实现在重导批 1 上的判定（`impl-verdicts.tsv`，Q2 的另一边）。

**阳性对照（每一条被测的判决器都跑）**：

- 执行器：P1–P5、P9 与盲测族（期望「不在」，Q3 本身）；P10（期望「不在」，考 A(c) 下界，附带 Q3-g）；N1–N5 与 P8（期望「在」，反向对照，测执行器会不会把条款容忍的改坏判成不在）；5.2 的单测 U1–U6、U8（新改的几步各自的会红点）。
- 实现那一侧：同一批坏状态在导出装置里也过一遍 `judge_crash_image`，只报（附带 Q3-f），不判。
- 比对器：Q2-pc 照原登记（实现判定表复制一份、翻掉一态的放行，必须恰好比出 1 处）；Q3 的比对器另加一条：期望表复制一份、翻掉一行的期望判定，必须恰好比出 1 处判错（Q3-pc）。

### 5.2 执行器解冻一次：改哪几步、依据哪几句

**起点**：把第一次冻结的那一份执行器目录 `scripts/experiment-apparatus/e170_o3_spec_executor/` 里进过冻结的文件（第一次第二段产物 `research/results/e170-o3-spec-executor-2026-10-02-s2-executor.out` 的冻结行逐个点名的那些）拷进新目录 `scripts/experiment-apparatus/e170_second_run_o3_spec_executor/`，拷完逐个算 sha256 与那几行比，全同才往下改（不同走 V9）；原目录一个字节不动。比对器与变异驱动不拷，由读 `crates/` 的执行员在第二、三段照第一次那几份重写或拷过来改（它们不进冻结）。

**只改下面五处**；每一处在代码旁写依据（`文件@小节标题`，layout 的行写文件加行号），并在交回里逐条列「改了哪个文件哪个函数、依据哪一句」（Q1-d）：

| 改 | 步（原登记 5.2） | 改成什么 | 依据（第二节已抄） |
|---|---|---|---|
| C1 magic | 2、3、5（7 见右列） | 系统配置槽、根记录、journal 记录自证时核 magic：`SFSB`（53 46 53 42）、`SFSR`（53 46 53 52）、`SFSJ`（53 46 53 4A），不等即这一份自证不过。步 7 验点名单元要不要核 `SFSU`：执行员照 kb 现查单元自证的条款，有一句把 magic 列进单元的自证就核、写依据；找不到这一句就不核，记进 Q1-f，不新开欠一句 | `layout/01-first-txn.md` 第 109、369、338、175 行 |
| C2 字段偏移 | 2、5、7 | 系统配置字段表、journal 记录头、位置条目、根记录：行序即盘上顺序、偏移按行累加，从暂代读法升成严格读法；载入时对 layout 第 158、216、359 行列出的**每一个**偏移逐个断言（第一次只断言了 kb 写明的少数几个） | `layout/01-first-txn.md` 第 158、216、359、365 行 |
| C3 读槽 1 的偏移 | 2 | 逐盘先读槽 0；槽 1 的偏移按次序取第一个有的：① 本盘槽 0 可用 → 它自述的槽距；② 否则池里按设备次序第一块槽 0 可用的盘自述的槽距；③ 全池没有可用槽 0 → 逐档：4096 到 1 MiB − 4096、每 512 字节一档各读一槽，只收自证过且自述槽距等于所在偏移的槽，几档都收得到取世代号最大，世代号相同取偏移小。「可用」= 自证过（magic、整槽校验和、格式版本、incompat 位、加密类型都过）且几何字段都在这一版读者收的范围里。「设备次序」在执行器里取什么由执行员照 kb 现查，kb 推不出唯一读法就记 G21 起的一行（「两种读法」类，暂代读法写明） | D22 已定项 16 定案那一句；`layout/01-first-txn.md` 第 160 行 |
| C4 逐档的两个出口 | 2 | 逐档读到自证过而池级字段越界的槽 → 整池拒（L = 无）；incompat 位不认得的槽不可择、不核偏移 | 同上 |
| C5 欠一句表 | — | `spec-gaps.tsv` 的 G01、G02、G03 三行类别改成「已由 kb 补（2026-10-02）」，补一列写依据的 kb 行，严格跑法不再因它们判欠一句；G04–G20 原样；新行从 G21 起 | 同上各行 |

**不改的**：G04–G20 的暂代读法与触发条件、择根、扫环、取前缀、版本表与 A(c)、建镜像、驱动、CRC。执行员读 kb 时发现 G04–G20 里某一行今天的 kb 已经有句子可依（例如逐档里那句 incompat 与 G14 的关系），不改代码，逐行记进 Q1-f（行号、kb 句子、照它改会不会改变触发条件）。

**必须有的单测**（合成池，`python3 -m unittest`；期望照 kb 句子推，旁注依据；第九节新变异的会红点就在这里）：

| 单测 | 合成池 | 期望 |
|---|---|---|
| U1 | 三支：(a) 某盘世代号最大的系统配置槽 magic 改 `SFSX`、整槽校验和重封；(b) 一条 txg 比最新根大的伪根，magic 改 `SFSX`、重封，落在区域公式给的槽；(c) 一条接得上链的伪记录（计数器 c_n + 1、txg n + 1、末条标志 1），magic 改 `SFSX`、重封 | (a) 那一槽不可用，择同盘另一槽；(b) 伪根不进候选，择根不变；(c) 链断在 c_n，落点不变 |
| U2 | 盘 0 槽 0 翻一字节（自证不过），盘 1 槽 0 可用、自述槽距 8192；盘 0 真的槽 1 在 8192；盘 0 偏移 4096 另放一个自证过的诱饵槽：自述槽距 4096、根环起点 65、世代号更大 | 走 ②：盘 0 槽 1 在 8192 读到，不读诱饵，池正常、L 与诱饵不在时相同 |
| U3 | 两盘槽 0 都自证不过；每盘 4096 处一槽（世代号 g、自述槽距 4096）、8192 处一槽（世代号 g + 2、自述槽距 8192），其余字段都在收的范围里 | 走 ③：每盘择 8192 那一槽（执行器轨迹里记所择槽的偏移与世代号） |
| U4 | 同 U3，两槽世代号相同 | 每盘择 4096 那一槽 |
| U5 | 两盘槽 0 都自证不过；4096 处一槽合规（世代号 g）；8192 处一槽自证过、自述槽距 8192、根环起点 65 | 整池拒，L = 无 |
| U6 | 两盘槽 0 都自证不过；4096 处一槽合规（世代号 g）；8192 处一槽自证过、自述槽距 8192、世代号 g + 2、incompat 位里有一位第一版不认得 | 8192 那一槽不可择，择 4096 那一槽，池不拒 |
| U8 | 一个触发 G04–G20 里某一行的合成态（例：同一块盘两槽世代号相同而内容不同，G19） | 严格跑法判「规约欠一句 G19」，`executor_main.py` 的批汇总行把它数进撞欠一句的态数 |

U7 不设（① 由批里每一态都走，锚点 A13 钉住）。合成池的槽距 8192 要与它自述的根槽距、根环起点一致，执行员照 D22 已定项 16 第 2 句与已定项 2 自己配。

### 5.3 崩溃状态从哪来：两份导出

- **第一次的导出**（Q1）：`scripts/experiment-apparatus/target/e170-dumps/` 原样，格式照原登记 5.3。执行器开跑前按实验页「导出文件的 sha256」一节的表核 20 个文件（V8）。
- **今天的重导**（Q2、Q3）：导出装置 `dump` 模式不改代码，连跑两趟：`dump --output-directory scripts/experiment-apparatus/target/e170-second-run-dumps-v2-first-run/`，再 `dump --output-directory scripts/experiment-apparatus/target/e170-second-run-dumps/ --compare-with scripts/experiment-apparatus/target/e170-second-run-dumps-v2-first-run/`；确定性照原登记 V2 与修订 1 的 V2b 判。产物另记三行 git 标识（读法 3），并逐文件报重导与第一次导出的比对：八个不带耗时的文件 `identical=true/false`，`impl-verdicts.tsv` 去掉耗时列后逐行比、报不同的态数与态号（附带 Q2-i）。导出装置今天编不过或跑不起来时，只许改到编得过、不改任何输出（改了什么记第十二节）；要改输出才跑得起来的，走停机 S6。
- **坏状态的新模式**：导出装置加一个模式 `bad-states-second-run --dump-directory <目录> --seed-hex <种子>`，造出的行与原 `bad-states` 模式在同一份导出、同一个种子上逐行相同，只差两处：P8 四行的期望判定写 `in`（期望落点不变，仍是 v_{n−1}）；每条流多一行 P10（5.5）。原 `bad-states` 与 `dump` 两个模式的代码路径不改（S5 核）。写出的文件名与原模式相同（`bad-states.tsv`、`bad-patches.bin`、`bad-expected.tsv`、`bad-impl-verdicts.tsv`、`bad-seed.tsv`），进 `scripts/experiment-apparatus/target/e170-second-run-dumps/<流名>/`。

### 5.4 取样规则

批 1、批 2 照原登记 5.4 原样（导出装置做，执行器不挑态）。批 3 只为原登记的 Q4，这次问题单没有 Q4：导出照样带着它，执行器不判（「不适用」，第十一节）。

### 5.5 坏状态表：P8 期望的登记修订与 P10

**修订（对原登记 5.5 P8 一行的期望；写于 2026-10-03，本次重跑任何产物之前——重导、执行器改动、坏状态都还没跑）**：

- 原文（第二节已抄）：「不在：择到 n − 1，链断在计数器 c_n（D23 已定项 14、22），L = v_{n−1}，而 v_n 已确认（ack = n），v_{n−1} 在下界之下」。
- 改成：**在，L = v_{n−1}**。补丁、底态（F、Mk）、两条流都不变。
- 依据原登记读法 3（第二节已抄），逐步推：
  - 记 v_n 是底态里最新的一版、发它的那次调用记作第 k 次。
  - **底态 F**：全部写持久，m(c) = 录制流最后一次写的流序下标。第 k 次调用就是最后那次发布所在的调用，它的返回点 = 返回时录制流的长度，大于 m(c)，所以第 k 次调用发的每一版都不算确认。ack(c) = 第 k 次之前那些调用发的最后一版；v_n 是第 k 次发的，v_{n−1} 要么也是第 k 次发的、要么就是第 k 次之前最后一版，所以 ack(c) 在流序上不晚于 v_{n−1}。begun(c) = v_n。于是 v_{n−1} ∈ [ack(c), begun(c)]。
  - **底态 Mk**：持久的是流序下标小于第 k 次调用返回点的写，m(c) < 返回点，第 k 次调用发的版本同样不算确认；同上，v_{n−1} ∈ A(c)。
  - P8 照条款落到 v_{n−1}（根 n 自证不过、择到根 n − 1；记录 n 两份都读不出，链断在 c_n；D23 已定项 14、22），所以 L ∈ A(c)，判「在」。原文的「v_n 已确认（ack = n）」与读法 3 在这两种底态上都不符。
- 按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「臂的定义也在「跑前写死」之列——失败条款打中的时候怎么办」三步：**记一次输**——第一次跑照原登记字面，Q3 = 没有（四行 P8 判「在」），这一笔留在实验页，不回改；**只许收严**——P8 改成期望「在」之后，P 行里少了唯一一行考 A(c) 下界的「不在」，为此补 P10；**收严在哪**——新表多要求执行器在 Mk⁺ 底态上把落到下界之下的 v_{n−1} 判「不在」，这正是原 P8 想考而没考到的那一格。
- P10 进附带量 Q3-g，不翻 Q3：问题单 Q3 的候选只点名「P1–P9 每一行（P8 照改正后的期望）与盲测族」。要不要让 P10 进 Q3 的判定，交主 agent 在导出装置的新模式写之前定（定了改问题单与第六节 Q3-g 一行）。

**P10**（新行，期望「不在」，附带 Q3-g；设计员照读法 3 推的，被攻过零轮）：

| 行 | 底态 | 补丁 | 期望与依据 |
|---|---|---|---|
| P10 最新根与它那条记录都读不出，底态 Mk⁺ | Mk⁺（原登记 5.5 的定义，含「不同实例就顺延」） | 同 P8：根 n 偏移 122 翻一位，不重封；记录 n 两份各在点名项区（或点名项为空时新根段）翻一位，不重封 | 不在，L = v_{n−1}：Mk⁺ 里持久了 Mk 之后那次发布的第一次单元写，它的流序下标不小于第 k 次调用的返回点，所以 ack(c) = v_n；begun(c) = v_{n+1}；落点 v_{n−1} 在下界之下 |

某条流没有 Mk⁺ 底态（导出装置 `mk_plus()` 给不出）时那条流没有 P10，照报个数，不停。

**其余各行**（P1–P5、P9、N1–N5、盲测族的形态与期望）照原登记 5.5 原样。盲测族的种子换成这次冻结的执行器算出的那一个（原登记 5.5 的取法：冻结文件按路径排序逐个喂进 sha256，取前 16 个十六进制字符，当 u64 喂 splitmix64）；第一次的种子 `8546b1258e427c34` 不再用。

### 5.6 严格跑法、暂代跑法、冻结

照原登记 5.6，改三处：① 冻结的时点是第一段在**第一次的导出**上判完批 1、批 2 之后；冻结的文件 = 新目录里除 `comparator*`、`mutation_driver.py`、`__pycache__` 之外的全部（含 `spec-gaps.tsv` 与单测）。② 冻结之前写执行器的执行员不打开任何一份 `impl-verdicts.tsv`、`bad-*.tsv`（第一次的与重导的都算）。③ 冻结之后执行器不再改；在重导的批 1、批 2 与坏状态上判，用的都是冻结的那一份，产物开头逐行核冻结行与第一段产物相同。

### 5.7 装置写在哪；禁读清单；资源上限

- **执行器走 `research/` 那一支**：`scripts/experiment-apparatus/e170_second_run_o3_spec_executor/`（Python，目录里不放 `main.rs`）；执行员先跑 `cargo metadata --format-version 1 --no-deps --manifest-path scripts/experiment-apparatus/Cargo.toml`，确认 bin 目标里没有多出 e170 那一项。执行器、单测、`spec-gaps.tsv` 放这里；比对器与变异驱动也放这里，不进冻结。
- **导出装置是入库装置**（这条岔路的 Q2 问的就是「`crates/` 今天这份代码在同一批状态上判得与执行器一不一致」，另写一份独立模型答不了）：仍是 `crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs`，只读、只驱动、只观测，不改 core、checker、harness、format。**要读 `crates/` 常量的量**：照原登记 5.7 那张清单（盘数、盘字节数、物理块与 io_min、固定 fsid 与写时刻、两份文件内容的生成式、流二的段长表与写数与根数），装置里已经是本地常量加 `main` 开头回比（S1）；新模式不新读任何 `crates/` 常量，补丁与重封照第一次 `bad-states` 模式调 core 的写者。
- **写执行器的执行员的禁读清单**（换掉原登记 5.7 那一份）：`crates/` 全部（含 `git log -p` / `git show` / `git diff` 里落在 `crates/` 的部分）；`scripts/experiment-apparatus/` 下除新目录与原目录里进过冻结的那 15 个文件以外的全部（原目录的 `comparator*`、`mutation_driver.py` 也不读）；`scripts/` 下的源文件（照用法跑 `run-with-memory-cap.sh`、`capped.sh` 不算读）；`research/results/` 全部，只有 `grep '^name=freeze' research/results/e170-o3-spec-executor-2026-10-02-s2-executor.out` 那几行例外；`research/prompts/` 下除本登记（第一、二、五至十一节）、原登记（第一、二、五至十一节）与问题单 `research/prompts/e170-r1-questions.md` 以外的全部；`records/` 全部；`.claude/kb/` 下除 `decisions/`、`invariants.md`、`layout/` 以外的全部（`.claude/kb/experiments/` 在内）；本登记与原登记的第三、四、十二、十三节；`scripts/experiment-apparatus/target/` 下任何一份 `impl-verdicts.tsv` 与 `bad-*` 文件（冻结之前）。kb 正文里点名的 crates 函数名、文件名、行号只当文字读，不顺着去读代码。执行员交回时附自己读过的文件清单（从自己的工具调用里列）；主 agent 可用 `scripts/agents/agent-handover.py` 抽它的会话记录核对。写执行器的执行员不是这一批读过 `crates/` 的任何一位。
- **线程与内存上限从派发提示取**（「线程上限：N」与内存上限）：导出装置与执行器的工作线程 / 进程数都不超过 N；编译、跑装置、跑执行器、跑变异一律经 `bash scripts/hosts/capped.sh N` 与 `bash scripts/hosts/run-with-memory-cap.sh <上限>`。导出装置在 checker 档包里，跑它照 `.claude/agent-common.md` 的重型测试那一条由主 agent 判归谁跑（第一次是执行员经 `cargo run` 跑的）。

### 5.8 实现量与分段

实现量（推的，没量）：执行器改动 150–250 行 Python（C1 约 20、C2 断言约 40、C3 与 C4 约 100、C5 与欠一句表的读写约 30），单测 U1–U6、U8 与合成池扩展（槽距 8192、诱饵槽、incompat 位）约 300 行；导出装置新模式约 80–120 行 Rust（复用 `bad-states` 的建行函数，改 P8 的期望、加 P10、S5 的自检）；比对器在第一次两份上改约 80 行（读严格跑法那一列、Q3 按期望判对、Q3-pc、P10、git 标识）；变异表 X1–X20、XC1–XC2 约 22 行、XE4 一行。历史族、几何格、崩溃支线不另枚举：态由导出装置给（两份导出 × 批 1、批 2 × 两条流），坏状态 = 原 76 行形态 + P10 两行。

分段（每段交回之后主 agent 对着问题单逐行判续不续，一行开着的都不剩就停，后面的段不跑）：

| 段 | 谁做 | 做什么 | 要改的装置与产物 | 补问题单哪一行 | 补完够不够判 |
|---|---|---|---|---|---|
| 第一段 | 只读 kb 的执行员（禁读清单见 5.7） | 拷执行器（V9）；照 5.2 改 C1–C5；单测 U1–U6、U8；在**第一次的导出**上严格与暂代两种跑法判批 1、批 2；冻结；锚点 A1–A13、S2、S3 | `scripts/experiment-apparatus/e170_second_run_o3_spec_executor/`；`research/results/e170-r1-o3-spec-executor-<日期>-s1-executor.out` | Q1 | Q1 够判 |
| 第二段 | 读 `crates/` 的执行员 | 重导两趟（V2、V2b、S1、S4、与第一次逐文件比）；冻结的执行器在重导的批 1、批 2 上严格与暂代两种跑法判；Q2 比对器与 Q2-pc | 导出目录 `scripts/experiment-apparatus/target/e170-second-run-dumps*/`；`research/results/e170-r1-o3-spec-executor-<日期>-s2-export.out`、`-s2-q2-comparator.out` | Q2 | Q2 够判（严格跑法撞欠一句的态单报，那几态不够判） |
| 第三段 | 读 `crates/` 的执行员（可与第二段同一位） | 导出装置加 `bad-states-second-run` 模式（S5、S5b）；种子取第一段冻结行；冻结的执行器判坏状态；Q3 比对器与 Q3-pc；执行器变异 X1–X20、XC1–XC2 在草稿目录的副本上跑；XE4 | 导出装置同一个文件；`scripts/experiment-apparatus/target/e170-second-run-dumps/*/bad-*`；`research/results/e170-r1-o3-spec-executor-<日期>-s3-bad-states.out`、`-s3-mutations.out`；变异表 `scripts/mutation/tables/e170_second_run_o3_spec_executor.tsv`；`crates/mutations.tsv` 末尾一行 XE4 | Q3 | Q3 够判（严格跑法撞欠一句的行单报，那几行不够判） |

第二段的重导不依赖第一段，可以与第一段同时开工；它判批与比对要等第一段的冻结行落盘。第三段的新模式代码可以与第一、二段同时写，跑要等种子（第一段）与重导目录（第二段）。

## 六、报哪些量与各自的判据

每个量各占一行、各判各的；合取留给人做。「附带」的量不翻任何一行，同一趟顺带算出的照报，要另跑的够判后不跑。Q1、Q2、Q3 的判定一律取严格跑法（读法 5）。

| 量 | 怎么算 | 门槛 | 判定 | 问题单哪一行、取什么值翻面 |
|---|---|---|---|---|
| Q1-a | 严格跑法下，第一次导出的批 1 两条流里撞上欠一句的去重编号数与受影响的态数（两条流分报、合计各一） | 0 | 0 → 判得出；≥ 1 → 判不出（逐个编号列，带类别与条款） | Q1：从 0 变成 ≥ 1 即翻到判不出 |
| Q1-b | `spec-gaps.tsv` 里类别为「只有读 crates 才写得下去」的行数 | 0 | ≥ 1 → 单报 | Q1：≥ 1 翻 |
| Q1-c | 严格跑法下第一次导出的批 1 里给出在 / 不在的态数 ÷ 批 1 态数 | 1 | < 1 时报差多少 | Q1（覆盖口径，与 Q1-a 各报各的） |
| Q1-d | 5.2 的 C1–C5 每一处：改了哪个文件哪个函数、依据的 kb 句子（`文件@小节标题` 或文件加行号） | 五处都有 | 缺一处 → 那一处改动按没有依据记，交主 agent（V10） | Q1 候选「判得出」的定义要求「改了哪几步、依据哪几句 kb 逐条写」 |
| Q1-e | 同 Q1-a，第一次导出的批 2 | 0 | 与 Q1-a 判定不同向记「不稳定」 | Q1，几何敏感性（G2） |
| Q1-f | G04–G20 里今天的 kb 已有句子可依的行（行号、句子、照它改会不会改变触发条件）；步 7 单元 magic 有没有依据 | — | 不判 | 附带：交主 agent 定下一次解冻 |
| Q1-g | 同 Q1-a，重导的批 1；只在读法 2 判两份批 1 不是同一批时算 | 0 | 与 Q1-a 判定不同向记「不稳定」 | Q1，几何敏感性（G6） |
| Q1-h | 暂代跑法下第一次导出的批 1 给出在 / 不在的比例 | — | 不判 | 附带 |
| Q2-a | 冻结的执行器在重导批 1 上的严格跑法判定与今天的实现判定逐态比：执行器「在」而实现没有两判都放行、或执行器「不在」而实现两判都放行，这样的态数（严格跑法撞欠一句的态不进比） | 0 | 0 → 一致；≥ 1 → 不一致 | Q2：≥ 1 翻 |
| Q2-b | Q2-a 里每一态：执行器的轨迹（择到哪个系统配置槽、槽 1 走 ①②③ 哪一步、择到哪条根、前缀停在哪个计数器）、实现的 `red_items`、两边的落点；执行员给暂定类别（规约欠一句 / 执行器写错 / 实现错 / 本登记的读法错）并引条款 | — | 交主 agent 判 | Q2「判谁错」 |
| Q2-c | 重导批 1 里严格跑法撞欠一句、因此没进 Q2-a 的态数 | 0 | > 0 → 那几态 Q2 不够判，写明 | Q2 的够判条件「批 1 逐态比完」 |
| Q2-d | 同 Q2-a，重导批 2 | 0 | 与批 1 判定不同向记「不稳定」 | Q2，几何敏感性（G2） |
| Q2-e | 执行器路径计数（重导批 1，严格跑法给出判定的态）：重放过 ≥ 1 条记录的态数；择到的根不是根环里自证过的根中 txg 最大那条的态数；槽 1 偏移走 ①、②、③ 的「盘 × 态」数各一；③ 里收到 ≥ 2 槽的次数；判「不在」的态数；镜像里有撕裂段的态数 | 重放 > 0；③ > 0 | 某项为 0 写明那条路径批里没走到，Q2 的「一致」不覆盖那条路径（F6） | Q2：阴性结果与「代码没跑到」分开 |
| Q2-pc | 比对器阳性对照：实现判定表复制一份、翻掉一态的放行再比 | 恰 1 | ≠ 1 → Q2 作废（V5） | Q2 |
| Q2-f | 两边落到的 (实例代号, txg) 不同的态数 | — | 不判 | 附带，同一趟算出 |
| Q2-g | 暂代跑法判定与实现判定不同的态数 | — | 不判 | 附带，同一趟算出 |
| Q2-i | 重导与第一次导出逐文件比：八个不带耗时的文件 `identical=`；`impl-verdicts.tsv` 去掉耗时列后判定不同的态数与态号 | — | 不判 | 输入核对：决定读法 2 用哪一批；附带 |
| Q3-a | P 行 26 个实例（P1–P5、P8 × 底态 F、Mk × 2 条流，加 P9 × 2 条流）里严格跑法判定与期望不同的个数（P8 期望「在」，其余期望「不在」） | 0 | 0 → 有；≥ 1 → 没有 | Q3：≥ 1 翻 |
| Q3-b | 盲测族 40 行（新种子）里严格跑法判「在」的个数 | 0 | 0 → 有；≥ 1 → 没有 | Q3：≥ 1 翻 |
| Q3-c | P 行与盲测族里严格跑法判「规约欠一句」的个数 | 0 | > 0 → 那几行 Q3 不够判，写明 | Q3 的够判条件「每一行在严格跑法下都跑过」 |
| Q3-d | P 行落到的版本与期望不同的个数（P1 期望 L = 无，P8 期望 L = v_{n−1}，其余期望 L 不在版本表里） | — | 不判 Q3，逐行列 | 附带 |
| Q3-e | N 行 10 个实例里判「不在」的个数、落点与期望不同的个数 | — | 不判 | 附带（F4） |
| Q3-f | 实现在 P 行、P10 与盲测族上两判都放行的个数 | — | 不判 | 附带（Y3（三 oracle 合谋测试） 的数） |
| Q3-g | P10（每条流至多一行）里严格跑法判「在」或落点不是 v_{n−1} 的个数 | 0 | 不翻 Q3；≥ 1 → F10 | 附带：A(c) 下界的阳性对照；主 agent 定 P10 进 Q3 时这一行改成判 Q3 |
| Q3-h | 暂代跑法下的 Q3-a、Q3-b | — | 不判 | 附带，只在 Q3-c > 0 时报 |
| Q3-pc | 比对器阳性对照：期望表复制一份、翻掉一行 P 行的期望判定再比 | 恰 1 | ≠ 1 → Q3 作废（V11） | Q3 |

门槛与臂的定义并排看：Q1-a、Q2-a、Q3-a、Q3-b 的门槛 0 是问题单给候选下的定义本身，判的是逐态 / 逐行观测出来的个数，臂的定义推不出这个个数是多少；Q2-e 的「③ > 0」不从任何臂推出，它管的是「一致」覆不覆盖这次改的那一步。问题单没有 Q4，这次不报用时（判定行里的 `time_ns` 照写，不汇总、不判）。

## 七、钉绝对值的断言

**出自被测条款、它引的字节表与登记读法**（不符走第十节 F2「条款可能错」，两边都查，不作废）：

- A1（改正）：流一 F 态，执行器落到 (实例 1, txg 3)；A(F) 恰两个元素 (1, 2)、(1, 3)。依据：读法 3（末次调用即新池新建文件那次发布，返回点大于 m(F)，ack = 暖机那次调用的最后一版）与 D22（单元原子性怎么合成） 已定项 16 第 5 句（暖机空发布 2 次、新池新建文件 txg 3）。原登记 A1 写「恰一个元素」，与读法 3 不符（第一次已按 F2 记），这次改正；这个值第一次已经量到过（第四节第 1 条），只当回归锚点。
- A2–A7、A11：照原登记第七节原样（自证过的根四条与位置、journal 记录三条与偏移、择到的系统配置槽世代号 5 / 槽 1 / tail 3 / 实例代号 1、版本表四版、相邻版 txg 差、记录 (1, 3) 的头字段、流二根 flags 位 0）。
- A12：执行器核的 magic 恰是 `SFSB` = 53 46 53 42（系统配置）、`SFSR` = 53 46 53 52（根记录）、`SFSJ` = 53 46 53 4A（journal 记录）；流一 F 态里每块盘择到的系统配置槽、A2 的四条根、A3 两盘共六份记录，前 4 字节逐个等于对应的值（`layout/01-first-txn.md` 第 109、369、338 行）。
- A13：流一 F 态，两块盘的槽 1 都走 ①、偏移 4096（`layout/01-first-txn.md` 第 160 行「第一版 4096」；D22 已定项 16「本盘槽 0 可用……时取它自述的槽距」）。

**独立算出、用命令核过的**（不符 → 第十一节 V3 作废那一趟）：

- A8：CRC-32C(ASCII "123456789") = 0xE3069283。
- A9：CRC-32C(4096 个零字节) = 0x98F94189。
- A10：CRC-32C(512 个零字节) = 0x30FCEDC0。

A8–A10 原登记用两条路子算过，2026-10-03 用本机 `google_crc32c` 又复算一遍（第十三节）。A12 的十六进制另用 `xxd` 核过（第十三节）。互比断言（执行器对实现、批 1 对批 2、第一次导出对重导）旁边各有钉绝对值的：Q2 对应 A1–A7、A11–A13；Q3 的期望表每一行是一个绝对的期望判定与落点。

## 八、轨迹与几何敏感性

**轨迹**：本实验没有「轮」，被判据吃的量（Q1-a、Q2-a、Q3-a、Q3-b）是一批状态或一张表上的计数，不随轮变化。代替轨迹报沿流序的分布：每条流按段序报 Q1-a 的逐段撞欠一句态数与 Q2-a 的逐段不一致态数，各报峰值、为正的段数、最后一段的值；正文只报这几样时不写「从来」「恒」。

**几何敏感性**（至少一个方向相反的取样点；翻面记「不稳定」，不翻记两点）：

| 点 | 判据那一格 | 两点 |
|---|---|---|
| G1 | Q1-a、Q2-a | 流一（取号、暖机两次、新建一个文件）对流二（重开、回退、复用、抬 F、卸载） |
| G2 | Q1-a 对 Q1-e、Q2-a 对 Q2-d | 批 1 对批 2（取样位置错开半格） |
| G5 | Q3-a | 底态 F 对底态 Mk |
| G6 | Q1-a 对 Q1-g | 第一次的导出对今天的重导；读法 2 判两份批 1 是同一批时只有一点，照写「同一批，一点」 |

**判别力自证**：Q2 由 Q2-pc 证；Q3 由 Q3-pc 与第九节 X8（P10：不在 → 在）、X9（P9：不在 → 在）证；Q1 的撞欠一句计数由 XC2（U8：1 → 0）证。

## 九、变异

执行器的变异（X 行）与比对器的变异（XC 行）登记在 `scripts/mutation/tables/e170_second_run_o3_spec_executor.tsv`，由变异驱动在草稿目录的副本上逐条替换、跑单测与第三段的判定、还原，结局照 `.claude/rules/mutation-sampling.md` 三栏记（抓到 / 无效 / 没红），先逐条核原文在新目录的源文件里恰好命中一次。导出装置的变异（XE 行）追加进 `crates/mutations.tsv` 末尾；`crates/mutations.tsv` 整表复跑是重型，归主 agent 在提交时跑。

| 变异 | 改什么 | 在哪个取样点上改变输出 |
|---|---|---|
| X1–X7、X9、X11 | 照原登记第九节原样（择根取最小、前缀判定改「大于上一条」、不看末条标志、施加前不验点名单元、择槽取世代号最小、根槽不验自证、版本只比 (实例代号, txg)、读不出的记录槽跳过） | 照原登记第九节写的取样点（流一 F 态的 A1、A2、A4，N2、N3、N4、N5，P9） |
| X8 | A(c) 不算下界（ack 一律当版本 0） | P10（两条流）：不在 → 在。原登记的敏感点 P8 改正之后不再敏感 |
| X2、X10 | 照原登记（X10 用第一次第三段改过的锚点：链首候选那一句跨实例比序） | 本实验仍没有敏感取样点，按 `mutation-sampling.md` 第三类记「缺敏感取样点、未覆盖」，不当等价变异留档 |
| X12 | 系统配置槽不核 magic | U1(a)：择到 magic 是 `SFSX` 的那一槽；两条流的批态里没有敏感点（没有 magic 错而校验和对的槽） |
| X13 | 根记录不核 magic | U1(b)：择到伪根 |
| X14 | journal 记录不核 magic | U1(c)：伪记录接上链，落点变 |
| X15 | ③ 收到第一个就停（第一次的暂代读法） | U3：择 4096 那一槽而不是 8192；U5：停在 4096、不读到越界槽，不再整池拒 |
| X16 | ③ 世代号相同取偏移大 | U4：择 8192 那一槽 |
| X17 | 去掉出口一（逐档读到越界槽当不可择） | U5：L 由「无」变成正常落点 |
| X18 | incompat 位不认得的槽改成整池拒 | U6：池由不拒变成整池拒 |
| X19 | 跳过 ②（本盘槽 0 不可用直接走 ③） | U2：读到诱饵槽，整池拒 |
| X20 | 系统配置字段表里「槽世代号」的偏移 147 改 148 | C2 的偏移断言在载入时红；批 1 每一态 |
| XC1 | 比对器把实现的放行读成恒真 | Q2-pc：比出 0 处 ≠ 1 |
| XC2 | `executor_main.py` 数撞欠一句的态时读暂代跑法那一列 | U8：撞欠一句的态数 1 → 0 |
| XE4 | 导出装置新模式的 P10 底态取 Mk 而不是 Mk⁺ | S5b 触发（P10 的底态摘要与 P9-Mk+ 那一行的不同） |

XE1–XE3 照原登记，不另跑（导出装置的 `dump` 模式代码不改）。X2、X10 两条没有敏感点，实验页照实写「N 条变异里 2 条缺取样点、没覆盖」，不写「全抓」。X12–X19 的敏感点只在合成池单测里，两条流的批态上它们不改变输出，实验页照实写这一句。

## 十、失败条款

| 条款 | 结论怎么记 | 什么观测会让它触发 |
|---|---|---|
| F1 阳性对照打中 | 执行器没有判别力（Q3 = 没有）；Q2 的「一致」不再能当执行器判得对的证据 | Q3-a ≥ 1 或 Q3-b ≥ 1 |
| F2 条款锚点不符 | 执行器读错条款，或条款与实现写出的字节不符（条款可能错）；两边都查：执行器那一步的依据与导出字节逐字段对，结论交主 agent，不作废、不改锚点 | A1–A7、A11–A13 任一项与执行器在流一 / 流二 F 态的输出不同 |
| F4 反向对照判不在 | 执行器比设计员的读法严，或设计员读错条款（N 行零轮；P8 的改正依据读法 3）；两边都查，Q3 不因 N 行翻；P8 判「不在」照 Q3-a 算判错 | Q3-e 判「不在」的个数 ≥ 1，或 P8 任一行判「不在」 |
| F5 看过实现判定才对上 | Q2 正式结论取冻结那一份 | 第十二节有「看过实现判定之后的改动」，且改前改后 Q2-a 不同 |
| F6 路径没跑到 | Q2 的「一致」不覆盖那条路径，射程缩小写明 | Q2-e 的「重放 ≥ 1」或「③」为 0 |
| F7 Q2 不稳定 | Q2 记「不稳定」，不判一致 / 不一致 | Q2-a 与 Q2-d 一个为 0、一个 ≥ 1 |
| F8 Q1 不稳定 | Q1 记「不稳定」，逐编号列哪一批撞了 | Q1-a 与 Q1-e（或 Q1-g）一个为 0、一个 ≥ 1 |
| F9 只有读 crates 才写得下去 | Q1 判不出，且那一步按问题单的翻面观测原文命中；Q2、Q3 在严格跑法下照量，撞上的态与行单报 | Q1-b ≥ 1 |
| F10 下界没有判别力 | 执行器在 A(c) 下界这一格上没有判别力；Q3 不因它翻（P10 是附带），实验页照实写，X8 的敏感点同时落空 | Q3-g ≥ 1 |
| F11 严格跑法留下不够判的态或行 | Q2 或 Q3 在那几态 / 那几行上不够判；要不要再解冻一次（新的重跑）由主 agent 定，这次不再改执行器 | Q2-c > 0 或 Q3-c > 0 |

## 十一、作废条款与停机条款

**作废**：

- V1：装置、执行器或变异驱动经 `run-with-memory-cap.sh` 退 250–254：那一次输出不算，命令与退出码写进报告，不绕开包装重跑。
- V2 / V2b：重导连跑两趟，照原登记 V2 原文与修订 1 的 V2b 各判一遍、各报；V2b 触发即重导作废。V2 原文触发而 V2b 不触发时，照第一次主 agent 的判法记（认 V2b、V2 原文触发照实记）。
- V3：A8–A10 任一不符：执行器的 CRC-32C 写错，那一趟作废。
- V4：某态或某行的判定行缺失、读出空行或解析不了：读不到不是读到 0，那一批作废重跑。
- V5：Q2-pc ≠ 1：比对器分不出差别，Q2 作废。
- V7：写执行器的执行员读了 5.7 禁读清单里的文件：执行器作废，换一位没读过的执行员从头写（从第一次冻结的那一份重新拷，不在作废的那一份上改）。
- V8：任一份导出（第一次的或重导的）文件 sha256 与它的 `manifest.tsv`、实验页的表或第二段产物里记的不符：作废，第一次的那份不能用时 Q1 改用重导的批 1 并在交回里写明。
- V9：拷进新目录的文件与第一次第二段产物冻结行里的 sha256 有一个不同：起点不是冻结的那一份，停下交回，不往下改。
- V10：Q1-d 缺一处（C1–C5 里某一处改了而没写依据，或改了 5.2 之外的地方）：那一处改动按没有依据记，Q1 的判定交主 agent，执行器不在原地补改。
- V11：Q3-pc ≠ 1：比对器分不出差别，Q3 作废。

**停机**（`.claude/rules/implementation-first.md` 第 4 条：装置与 `crates/` 对不上，两边都查，既不作废也不当结果，产物照打、交主 agent）：

- S1–S4：照原登记第十一节原样（本地常量回比、执行器建的镜像摘要与 `states.tsv`、执行器数的 op / 写 / 调用 / 取样态数与 `manifest.tsv`、导出装置折回重建的摘要与 `CrashImage`），两份导出各判一遍。
- S5：新模式与原 `bad-states` 模式在同一份重导目录的两份草稿拷贝上、用同一个种子（第一次的 `8546b1258e427c34`）各跑一遍，`bad-states.tsv`、`bad-patches.bin`、`bad-impl-verdicts.tsv`、`bad-seed.tsv` 除 P10 行之外逐行相同，`bad-expected.tsv` 除 P8 四行的期望判定那一列与 P10 行之外逐行相同；有别的不同即停。
- S5b：新模式里每条流 P10 那一行的底态落地向量与摘要，与同一条流 P9-Mk+ 那一行的底态相同；不同即停。
- S6：导出装置今天要改输出才跑得起来（编不过之外的改动，或改了之后 S1 不过）：停下交回，写明改哪里、为什么。

**不适用**：问题单没有 Q4，原登记的 Q4 各量、计时包含自检（原 V6）与批 3 这次都不判、不跑；登记里写「不适用」，实验页照写，不算「够判后未跑」。

**够判停机**：问题单 Q1–Q3 每一行够判就停（够判条件照问题单原文：Q1 要第一次的批 1 两条流严格跑法跑完；Q2 要批 1 逐态比完；Q3 要坏状态表每一行在严格跑法下跑过）。每段交回附一张表逐行写「已够判 / 还差什么 / 剩下的量能不能让它翻面」；某一行开着而剩下的段补不到它时写明。没跑的量逐条标「够判后未跑」（例如第一段之后 Q1 判了而主 agent 判停，Q1-g、Q2-i 这类没跑的就照这样标）。

## 十二、修订

**修订 1（写于 2026-10-03，第 1 次重跑第一段的产物之前；执行员写；依据：单测写完后单测层变异第一遍的读数）**

读数：执行器目录里的单测 50 个全绿之后，变异表 X1–X20、XC2（共 21 条，逐条在执行器目录的副本上替换、编译、跑单测）第一遍 19 条抓到、0 条无效、2 条没红：X9（版本只比 (实例代号, txg)）与 X10（记录水位 txg 为主）；另外 X17 只被 G22 那条单测抓到，U5 自己没红。原判据一个字不动，只补下面三处与一处报告字段（都是收严或补取样点，没有放宽）：

1. 补两个取样点。X9 的敏感点第九节只写了第三段的 P9；单测小池里原来的 P9 同形测试用测试自己的在 / 不在判定、没走 `judge_state` 里那一行比对，所以补一条走 `judge_state` 的（把伪根放进基镜像，让落点与版本 3 同 (实例, txg)、水位不同；`VersionComparisonInTheJudgement`）。X10 在合成池上有敏感点（P5 型根 + 记录 3 改坏：水位之上剩实例 1 的记录 1、2，链首停在实例边界；改成 txg 为主比之后水位之上一条都没有），补一条单测。第九节原文「X2、X10 两条没有敏感点」改成：X2 没有敏感点；X10 的敏感点只在合成池单测里，两条流的批态上仍没有。实验页照这两句写，「N 条变异里几条缺取样点」按批态口径数。
2. U5 的世代号。5.2 的 U5 没写世代号；第一版单测把越界槽的世代号写得比合规槽大，去掉出口一（X17）之后越界槽被择中、后面另一处检查仍然拒池，U5 不红。改成越界槽世代号小于合规槽（4096 处合规、世代号 5；8192 处越界、世代号 3）：不整池拒时择到 4096 处那一槽、落点正常，X17 下 U5 红。
3. X20 的替换文。登记写「「槽世代号」的偏移 147 改 148」。第一版替换文（设备数宽度加一）把其后所有偏移一起挪了，被旧的 325、371 等断言抓到，看不出 C2 新加的逐偏移断言会不会红；改成设备数宽度加一、槽世代号宽度减一，只有槽世代号的偏移变成 148、其后偏移不变，只有 C2 新加的逐偏移断言会红。
4. 补的报告字段（不是判据，不改任何门槛）：`name=path_counts` 加 `chosen_root_below_largest_self_verified_txg`（第六节 Q2-e 第二项的新写法）、槽 1 走 ①②③ 的盘 × 态数、③ 里收到 ≥ 2 槽的盘 × 态数；新增 `name=gap_trajectory`（第八节 Q1-a 的逐段撞欠一句态数，峰值、为正的段数、最后一段的值）。
5. 时点说明：写本条之前，执行员已在草稿目录对第一次的导出（批 1、批 2）试跑过一次执行器，只为查它跑得通、输出没进 `research/results/`；本条 1–3 三处都出自单测层变异的读数，不依赖那次试跑；第 4 条里 `name=gap_trajectory` 是试跑之后补的报告字段，不改任何判定。

## 十三、读过的文件与跑过的命令

**读过的文件**（行号区间；grep 命中行也列）：

- `.claude/agent-common.md` 全文；`.claude/singlefs-ai-sop/rules/test-discipline.md` 1–122；`.claude/rules/implementation-first.md` 1–20；`.claude/rules/three-way-inference.md` 1–174；`.claude/singlefs-ai-sop/rules/evidence-discipline.md` 1–176；`.claude/rules/mutation-sampling.md` 1–96。
- `research/prompts/e170-r1-questions.md` 1–11；`research/prompts/e170-preregistration.md` 1–42、378–745 与标题行。
- `.claude/kb/experiments/170-O3规约执行器第一版.md` 1–32、67–131、238–280 与标题行，`grep -n '冻结\|sha256'` 命中 7、11、21、23、69、84、86、88、115、125、134、159、161、203、229、243、252、289、308 行（读到的结果列在第四节）。
- `.claude/kb/experiments.md`：只用 `grep -o '^| E155（[^）]*）'` 与 `grep -o '^| E170（[^）]*）'` 取了两个简称。
- `records/2026-10-01-里程碑三形式化接入.md` 标题行；grep 命中 440、445、449、450、451、456、469、473、491、498、499、501、529、531、536、538、539 行；第 485–537 行的 grep 命中 492、495、499、501、502、505、510、515、517、519、528、529、531、533、534、536 行。
- `.claude/kb/decisions/13-验证路线.md` 标题行、130–155；`.claude/kb/decisions/22-单元原子性怎么合成.md` 标题行、330–354；`.claude/kb/layout/01-first-txn.md` 标题行，grep 命中 109、158、160、175、198、216、338、359、365、369 行。
- `scripts/experiment-apparatus/e170_o3_spec_executor/`：文件清单与行数；`spec-gaps.tsv` 全部 21 行（每行前 400 字符，G01–G03 的缺什么、类别、暂代读法、触发条件四列，G04–G20 的触发条件列）；`comparator_q3_bad_states.py` grep 命中 1、7、32、136、144、158、167、177、179、182、184、202、204、207、212、213、214、216、221、225、226、227、231、232、271 行；`executor_main.py` grep 命中 45、48、54、102、103、105、106、107、109、110、111、112、119 行。
- `scripts/experiment-apparatus/target/e170-dumps/`：目录清单、20 个文件的 sha256；两条流 `bad-expected.tsv` 的前 3 行与 P8 四行（读到的期望列在第四节）。
- `crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs` 2636–2760、2785–2845，grep 命中 9、10、78–140 的常量行、158、166、175、200、251、252、266、286、291、296、320、393、394、406、423、429、448、455、476、486、504、531、539、540、543、549、619、620、622、627、628、629、632、854–865、881、928、946、997、1411、1689、1852、1854、1942、1953、2002–2027、2462、2553、2626、2638–2759 内的 `"in"` / `"not_in"` 行、2688、2704、2754、2772、2792、2798、2805、2811、2812、2827、2839、2879、2974、3157、3191、3194、3199、3200、3209、3252、3272、3273、3278、3280、3284、3285、3290、3292、3311、3693 行。
- `crates/singlefs-core/src/make_filesystem.rs` grep 命中 557、559、561 行；`crates/singlefs-core/src/recovery.rs` grep 命中 741、967、1039、1064、1255、4330 行；`crates/singlefs-checker/src/image.rs` grep 命中 583、631 行；`crates/singlefs-checker-tier/src/crash.rs` grep 命中 51、108、1222、1267、1307 行；`crates/singlefs-core/src/system_configuration.rs` 第 22 行、`root_record.rs` 第 11 行、`journal.rs` 第 20 行、`unit.rs` 第 14 行；`crates/singlefs-harness/src/scenario.rs` grep 命中 58、145、189 行；`crates/singlefs-harness/src/history.rs` 第 2228 行；`crates/singlefs-harness/tests/common/mod.rs` grep 命中 93、190、191、204、205、227 行。
- `scripts/kb/quote-kb.py` 1–60；`scripts/kb/claim-experiment.sh` 1–70；`scripts/kb/replace-once.py` 1–25；`scripts/experiment-apparatus/replay.sh` grep 命中 240–244、617、619、620、621、623 行；`scripts/mutation/tables/` 文件清单里 e170 那一项。
- 没读：`research/results/` 下任何文件；`.claude/kb/experiments/` 下别的实验页；别的实验的跑前登记与产物。

**跑过的命令**（原样；只读命令的输出没进登记的不贴，进了登记的数在对应小节）：

```bash
TZ=Asia/Tokyo date +%F                                         # 2026-10-03
D=/tmp/claude-1000/e170-r1-designer
grep -E '^\| (new_pool_file_creation|fixed_script_through_e_then_unmount) \|' ".claude/kb/experiments/170-O3规约执行器第一版.md" | awk -F'|' '{gsub(/[ `]/,"",$2);gsub(/[ `]/,"",$3);gsub(/[ `]/,"",$4); print $4"  "$2"/"$3}' > $D/first-run-dump-sha256.txt
(cd scripts/experiment-apparatus/target/e170-dumps && sha256sum -c $D/first-run-dump-sha256.txt) | awk '{print $NF}' | sort | uniq -c
#        20 OK
nice -n 19 python3 -c "import google_crc32c as g; print('crc32c_123456789=0x%08X zero4096=0x%08X zero512=0x%08X' % (g.value(b'123456789'), g.value(bytes(4096)), g.value(bytes(512))))"
#   crc32c_123456789=0xE3069283 zero4096=0x98F94189 zero512=0x30FCEDC0
printf 'SFSB SFSU SFSJ SFSR' | xxd -p
#   534653422053465355205346534a2053465352
cd scripts/experiment-apparatus/e170_o3_spec_executor && git status --short . | cut -c1-2 | sort | uniq -c     # 20 个 「A 」
sha256sum $(ls | grep -v __pycache__) > /tmp/claude-1000/e170-r1-designer/first-run-executor-sha256-2026-10-03.txt
git diff --cached --stat -- crates/singlefs-core crates/singlefs-format crates/singlefs-harness/src crates/singlefs-checker-tier/src/crash.rs crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs crates/singlefs-checker/src | tail -20
#   末行 33 files changed, 9782 insertions(+), 659 deletions(-)
git log --since='2026-09-30' --format='%h %ad %s' --date=iso -- crates/ | head     # 无输出
ls -ld --time-style=+%F scripts/experiment-apparatus/target/e170-dumps                                 # 日期 2026-10-01
nice -n 19 python3 scripts/kb/quote-kb.py $D/questions-quote.md 'research/prompts/e170-r1-questions.md:7-11'
#   ✓ 1 段整抄进 /tmp/claude-1000/e170-r1-designer/questions-quote.md，回读逐字节一致
L=.claude/kb/layout/01-first-txn.md; nice -n 19 python3 scripts/kb/quote-kb.py $D/quotes.md \
 '.claude/kb/decisions/13-验证路线.md@#### 已定项 6：三个互相独立的 oracle' \
 '.claude/kb/decisions/22-单元原子性怎么合成.md@#### 已定项 16：系统配置槽轮换规则；根环区域公式的地址空间与区域内槽位' \
 "$L~^\*\*系统配置槽 0 与槽 1 的位置\*\*" "$L~^\| 自举头 \| magic" "$L~^\*\*系统配置字段表的偏移\*\*" "$L~^\| 单元头 \| magic" \
 "$L~^\*\*位置条目（14 字节）的内部\*\*" "$L~^\| magic \| 4 \| 4 字节 ASCII \`SFSJ\`" "$L~^\*\*记录头字段表的偏移\*\*" \
 "$L~行序即盘上顺序（D22" "$L~^\| magic \| 4 \| 4 字节 ASCII \`SFSR\`" \
 'research/prompts/e170-preregistration.md:29-29' 'research/prompts/e170-preregistration.md:497-497' 'research/prompts/e170-preregistration.md:587-587'
#   ✓ 14 段整抄进 /tmp/claude-1000/e170-r1-designer/quotes.md，回读逐字节一致   exit=0
set -o noclobber; cat $D/part1.md $D/part2.md > research/prompts/e170-r1-prereg.md      # created
# 第二节追加之后：tail -n +<追加前行数+1> 登记 | cmp - quotes.md  → appended-quotes-identical
nice -n 19 python3 scripts/kb/replace-once.py research/prompts/e170-r1-prereg.md <旧串> <新串>   # 四次，各命中 1 次
```
