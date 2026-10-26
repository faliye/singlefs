# E161 第 3 次重跑登记：崩溃放量的去重与分段耗时——取号写两轮之后的段形与钉值重定

写于 2026-10-23，装置改动之前、任何新产物之前。不占号：第 1 次登记 `research/prompts/e161-preregistration.md`（sha256 `5533a9c1c3de540139586d0439da96b533220368558237530dd77ca4aa247872`）；第 2 次重跑登记 `research/prompts/e161-r2-prereg.md`（sha256 `9baac104a204c373dbd31205081123afc91d78f0f642a6b73c429a2d65d1c2e9`，下称「r2」）；调查报告 `/home/fy5090/.local/share/singlefs/work/m3-close-b16-checker-tier-pins/result.md`（sha256 `c594587504890416d3713008adaa0c0540c9b2f4a074654651d4ee5d65faa2cd`，草稿目录里的文件，只当线索：它给了写序变化的来历与它在副本上量到的数，这些数列在第四节，不当钉值）。开工时 HEAD 是 `841b3beab`。

这一次是「输入变了、臂与判据不变」的重跑，但输入一变，段号与锚点（判据的对象）跟着移，所以不是只写三样的差分登记：r2 里没变的条目不重抄，写成「同 r2 第 X 节（第 A–B 行）」，以上面那个 sha256 的 r2 为准；变了的整条重写。

## 一、问题

英文名：crash_state_dedup_and_time_split（照抄 r2 第 11 行；装置不另起，改原装置 `crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs`）

### 1.1 主 agent 给的问题（派发原文，逐字）

> **e161 重跑登记**：e161 的钉值（`research/prompts/e161-r2-prereg.md` 的锚点：D1-small 45、A2 逐段数、A8 覆盖写固定脚本流 78 段等）在「取号写两轮」之后对不上，调查员量的新值在报告第六节第 8 条（第一条流写 43、段 12、逐段 `[8, 8, 3, 1, 8, 3, 1, 8, 16777215, 3, 1, 8]`、D1-small 53、覆盖写流 80 段……）。验收：写一份重跑登记（照你的定义：哪几个锚点改、改的依据是哪一处写序变化、新值由执行员现量钉，不照抄调查员的数），登记里写明判据与作废条款。

被测条款（派发原文）：系统配置落地那一批的「取号写两轮」（`crates/singlefs-core/src/transaction.rs` 第 996 行一带）。岔路单（派发原文）：重跑。

### 1.2 问题单

派发提示没有另给问题单。下面四行照 r2 1.2 那张问题单（第 23–28 行）的形状、按 1.1 的验收改写，**待主 agent 认**（文末「待主 agent 认的」第 1 条）；认之前不派执行员。

| # | 问题 | 候选 | 翻面观测 | 够判条件 | 状态 |
|---|---|---|---|---|---|
| 1 | 取号写两轮之后，新池新建文件流（下称第一条流）的段长、逐段计划计数（3^m·2^(n−m) − 1）与 `crates/`、公式、独立锚点四方还逐段相等吗 | 相等 / 哪几段不等 | 四方任一段不等 | 12 段逐段比完，不等段数（0 或列出） | 开着 |
| 2 | 取样域照 r2 读法甲的同一意图挪段号（覆盖写固定脚本流——下称第二条流——的取样段从 11–26 挪到 12–28）之后，域的状态数写死、跑得完、每一格判据都有对象吗 | 是 / 跑不完 / 哪几格失去对象 | 域在写死的内存上限与限时里跑不完；或某格判据失去对象 | 域的状态数写死（第七节 A4），可行性档跑完，Q2c 九样逐样核完 | 开着 |
| 3 | 在挪过的域上，r2 产物（`research/results/e161-crash-state-dedup-and-time-split-feasibility-r2-2026-10-05.out`）的判决与 G 行各格还同向吗 | 同向 / 哪几格翻面 | 任一判决字段或 G 行判定与 r2 产物不同 | 新产物跑完，判决字段与 G 行逐格比完 | 开着 |
| 4 | 装置的内联测试与 E161 变异行在新钉值上全绿全抓吗 | 全绿全抓 / 哪几条红或没红、无效 | 任一内联测试红，或变异没红 / 无效 | 内联测试与 E161 那几行变异跑完 | 开着 |

四行在这份登记里叫「行 1」到「行 4」。

### 1.3 读法写死（装置改之前定下，改了就是新的一次）

只列与 r2 1.3（第 34–51 行）不同的词；没列的照 r2 1.3 一字不改。

| 词 | 这份登记里指什么 |
|---|---|
| 第一条流 | 搭建同 r2。今天的段形是 12 段 `[2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2]`、43 次写、3 条根槽写（crates 测试 `crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs:816` 的段长断言与紧跟的写数断言；根数照 r2 由段表里 1 写段的个数认）。「全量」是 16 777 268 个状态（同文件 `:778`），一个不抽；这一次不跑全量 |
| 第二条流 | 脚本同 r2。今天的段形是 80 段、485 次写、19 条根槽写（`crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs:517–535` 那一支的段序列与写数断言、`:556–563` 的根数） |
| 写序变化 | 取号写两轮（D22 已定项 8「取号写两轮」，第二节；实现 `crates/singlefs-core/src/transaction.rs:1214` `ACQUISITION_WRITE_ROUNDS` 两项、`:1235` 外层循环）：每轮每盘一次系统配置槽写、轮后一道池屏障，第二轮那道屏障完成才交出新号（`:990–:996` 的文档）。录制流里每一次取号从「一个 2 写段」变成「两个 2 写段」；两条流上各是哪几处，见第七节 A8 |
| 第二条流的取样段 | **段 12–28**（从 0 数，共 17 段）。段 0–11 全持久、不枚举；段 29 起不取 |
| 取段 12–28 的理由（与 r2 1.3「取段 11–26 的理由」同意图） | r2 的意图是「第一条流整条之后，取 B、B 的轮换、取号、写行 txg 5、暖机 txg 6、7 各自的单元、记录与根槽，到 txg 7 的根槽为止；txg 7 的轮换不取」。今天第一条流 12 段整条是第二条流的前缀（A8、停机条款 S2 核逐字节），所以从段 12 起；B 之后重开的那次取号从一个 2 写段变成两个（段 16、17），两段都在这批写里；txg 7 的根槽今天是段 28，轮换是段 29（不取）。算法在第十三节 `anchors_e161_r3.py`：起点 = 第一条流的段数，终点 = 起点之后第 4 个 1 写段 |
| 大段 | 读法甲照 r2 1.4（写数等于 24 的取样段），今天第一条流是段 8、第二条流是段 12，取样段里各自唯一 |
| PC3 的段 | 第二条流「写行发布 18 个单元写」那一段，今天是段 18（r2 是段 16，同一批写） |
| 不等段数（行 1） | 同 r2 1.3 那一行，段数 11 换成 12 |
| 同向（行 3） | 同 r2 1.3 那一行；比的是 r2 产物与这一次的产物 |

### 1.4 取样域的读法

只登记读法甲（r2 1.4，第 53–61 行；读法乙在 r2 由主 agent 2026-10-05 删去，第 188 行），门槛 24 不变：今天两条流取样段里 24 写段各一段（第七节 A4），门槛取 19 与 24 得同一个域，取 25、26 时大段整段并进 D1-small（第八节）。

### 1.5 装置写在哪（写死）

**入库装置**，改原装置 `crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs`，再改 `crates/mutations.tsv` 里 E161 那几行。理由同 r2 1.5：行 1、行 4 问的就是 `crates/` 今天这份代码的段形、枚举计数与这只装置自己的钉值，另写一份独立模型答不了。

这一次要读 `crates/` 的常量：仍只是 r2 1.5 说的那十四个（装置 `local_constants_against_the_crates`，今天第 125 行起），照旧本地常量加回比断言，不直接引。取号轮数 `ACQUISITION_WRITE_ROUNDS`（`transaction.rs:1214`）是不带 `pub` 的私有常量，装置引不到、也不加回比：两轮这件事由段形（A8、S2）看出来，段形对不上就停。新钉的段长表、写数、锚点（第七节）一律写成装置里的本地字面量，不引 crates 测试里的同名数：crates 测试那一份当第三方对照（第四节 4.2）。

### 1.6 重型测试与谁跑

装置的内联测试与 E161 的变异行属 checker 档（`cargo test -p singlefs-checker-tier --bin e161_…`，直接执行它的测试二进制），执行员不跑，写成第五节 5.8「要主 agent 跑的命令」。`feasibility` 模式是 `cargo run` 起装置本体：`.claude/hooks/lib_heavy_tests.py` 第 77 行写「`cargo run` 起 checker 档包自己的装置二进制也照旧不判（只有 e152 那一个按名字算……）」，所以由执行员经内存包装跑（与 r2 1.6 不同：那时它也算进了重型，这一次照今天的判法）。改装置的 `.rs`：执行员在主工作区写 `crates/` 下的 `.rs` 被写闸拒，出路是 `scripts/hosts/compile-then-swap.py`（共用约束「执行前拒绝的写法」写闸一条）；主 agent 改派实现员也可以，第五节 5.6 的表照用。

## 二、被测条款与它引的定义

被测条款仍是 D13（验证路线） 已定项 4（崩溃状态集合的定义：段模型）与已定项 7（记录核对器的入参含持久集合），整抄同 r2 第二节（第 73–132 行，`quote-kb.py` 出处 `.claude/kb/decisions/13-验证路线.md:12-12`、`:15-15`、`:121-136`、`:183-201`），这一次不重抄：段模型的字面没变，变的是它作用的写流。

写序变化的出处是 D22（单元原子性怎么合成） 索引表第 8 行与已定项 8（「取号写两轮」在索引表第 8 行与已定项 8 依据倒数第 3 条）。以下由 `scripts/kb/quote-kb.py` 整段抄进草稿目录的 `e161-section2-quote.md`、回读逐字节一致之后原样追加（两段：索引表第 8 行用 `~正则` 取整行，已定项 8 用 `@标题` 取）。
**出处 `.claude/kb/decisions/22-单元原子性怎么合成.md:19-19`（整段抄，未转述）**

```markdown
| 8 | **固定结构的放置** | 系统配置每盘一份、槽轮换、取号写两轮；journal 环两盘镜像；mkfs 先把每盘 [0, 根环基址) 写 0、过屏障，再把第 0 代根种进全部根环区域并把区域整段写 0 **状态：已定。** **进度：已实现。** |
```

**出处 `.claude/kb/decisions/22-单元原子性怎么合成.md:177-202`（整段抄，未转述）**

```markdown
#### 已定项 8：固定结构的放置

**定案**：三条，都是针对 2 盘的第一版（D2（RAID 条带策略）已定项 9）：

1. **系统配置每盘放一份。** 怎么更新它按已定项 21 办（原地覆写的结构要有整单元校验和，还要有真的被检查的世代号），做成槽轮换，每盘至少 2 个槽；槽的规则见已定项 16，字段见已定项 9。
2. **journal 环在两块盘上互为镜像。** 代价是稳态下每条记录写两遍。**这不是新增的开销**——D2（RAID 条带策略）已定项 6 本来就要求任何写都得 w≥2，这条定案只是把「任何写」明确到 journal 记录上。
3. **mkfs 要把第 0 代根种进全部根环区域**；写任何元数据之前，先把每块盘的 [0, 根环基址) 整段写 0（第一版 [0, 1 MiB)，恰盖住系统配置槽 0 与逐档找槽 1 的全部 2 033 档，用过的盘上重做 mkfs 不留旧池的系统配置槽），并把三个根环区域整段写 0（含未种根的那 3 × (S−1) 个槽），journal 环也整段写 0：三者都由 mkfs 自己经块设备的写零动作发出，每块盘上 [0, 根环基址)、根环三个区域与 journal 环各一次调用（区域归属只定根种在哪块盘，不定清哪块盘），录制流里各记一步整段清零（D17（实现分层与第三方管道） 已定项 2 第 1 条的第一种），每块盘 5 步、两块盘共 10 步，段序列与步数以 [layout/01-first-txn.md](../layout/01-first-txn.md) 八「mkfs 种根」那一行为准；每块盘的 [0, 根环基址) 清零之后两块盘各过一道屏障（作废），之后才写别的；其余清零与单元写之间不加屏障。fsid 由 mkfs 的对外入口生成，前 8 字节取自随机源；内层入口可给 fsid，给测试、装置与崩溃点重放用。种满之后「刚 mkfs 完、一次都还没发布」这一格跟稳态一样，根最多退回区域深度那么多代。

**射程**：系统配置自己的更新协议（槽轮换的代号、校验和多宽）跟着单元头字段表（D18（块里携带什么信息）已定项 7）的口径走，落在已定项 9 与已定项 16；zoned 介质上这些固定结构能不能原地轮转，属于 D22（单元原子性怎么合成） 未定项 6，还没答。容错承诺是单故障：每盘一槽系统配置坏在承诺里，取号之后两块盘最新那一槽持续坏也在承诺里（用户 2026-10-19 定，原话在变更史）：取号写两轮，每轮逐盘一写、轮后一道池屏障，每块盘两槽都带取号时的新号，全或无判失败时每块盘只回卷一次（`crates/singlefs-core/src/transaction.rs` 的 `write_the_witnessed_acquisition`、`roll_back_acquisition`）；每盘各坏一槽系统配置、同时最新根一时读不出的双故障形不在承诺里（用户 2026-09-28 定不罩），那一形抬 F 之前池级 checker 一条都不红（可写挂载的预隔离护住了，承诺仍不罩它），两条用例当登记，落点在 [checks-owed.md](../checks-owed.md) 已还清表 C583（双故障形：每盘一槽系统配置坏加最新根一时读不出） 那一行。可写挂载读系统配置的见证按盘读（S-辨：一槽撕裂、另一槽自证过，与设备读不出分开），判不出 ⟺ 有槽设备报读错，且没有一块盘两槽都自证过（R2，用户定，被攻过零轮，原话在变更史）。判不出的盘面有一条离线出路（C596（见证判不出的系统配置盘面只剩只读挂载））：管理员断言一个版本，那一版的根或那次发布的末条记录读得出且是它、各盘单元区里本池单元的 `birth_txg` 最大值不大于它，才只重写不自证的系统配置槽，写的实例代号取各盘自证槽、根环、journal 环与单元写序里的最大值、不加一；否则一个字节不写（用户 2026-10-19 定、2026-10-20 加 `birth_txg` 闸，原话在变更史；`crates/singlefs-core/src/system_configuration_repair.rs` 的 `rewrite_system_configuration_slots_for_the_asserted_version`）。某块盘两槽全废（一份本池自证过的系统配置都没有）时，取号那一步补写它的两槽，要这几样都成立，有一样不成立就拒：这块盘带所选那一版的全部单元（清单非空、每个落点读一次、字节与验得过的那一份相同）；这块盘的根环槽与 journal 环槽没有一槽设备报读错；这些槽里没有比所选那一版新的根或记录（按择根的序）；这块盘单元区里没有写序实例大于所选那一版实例的单元；所选那一版的根槽那一半不缺（与落后支缺根那一半同一判）。补写失败照取号每块盘回卷一次（用户 2026-10-03 定这一步拒，2026-10-19 开例外，2026-10-20 加单元区扫描与环槽读错两样，原话在变更史；`crates/singlefs-core/src/mount.rs` 的 `rewrite_gate_not_met_by_a_device_without_any_self_verified_system_configuration`，实现只拿两槽都读得出、都自证不过的盘过闸，有一槽设备报读错的照旧拒）。丢写的界按类写：全池读回一个值的纯形丢写 120 格，一盘报读错、另一盘读回全 0 的混合形丢写 0 格（C331（择根倒挂压过已确认的写） 网格上的格数，副本上量的），丢写格都落在每盘各坏一槽系统配置、同时最新根一时读不出的双故障形里。两块盘是同一个池时，作废持久之前崩溃：旧池读回最后确认的一版；可写挂载的结局按现行补写条款。作废持久之后、最后一道屏障之前崩溃：恢复只有两种结局——整池拒（`NoValidSystemConfiguration`），或读回 mkfs 的第 0 代空池（根为实例 0、txg 0，没有文件）；后一种里系统配置的冗余可以缺，缺新槽的那块盘的可写挂载按现行补写条款处置。调用方拿自己断言的参数（例如旧池的 fsid）去挂、与盘上择到的系统配置不符而拒（`CallerParametersDisagreeWithTheSelectedSystemConfiguration`），是调用方的断言与盘不符，不算盘面的结局。设备守屏障时，作废持久之后任何结局不得读回旧池的任何一版，也不得新系统配置与旧池的根并存；屏障只到一块盘归 C6（块层语义假设写错），两块盘来自两个池的起点不在射程里。

**依据**：

- 双故障形不罩：用户 2026-09-28 定案，原话在变更史；那一形的两条用例在 [checks-owed.md](../checks-owed.md) 已还清表 C583（双故障形：每盘一槽系统配置坏加最新根一时读不出） 那一行。
- E87（固定结构的放置）：2 盘 8 种失效组合穷举——系统配置只放一份时掉了那块盘全池挂不上（数据、根、journal 健在也没用），每盘一份 + journal 镜像 8 格全可挂、重放窗口零丢失；不种满第 0 代根时「刚 mkfs、零次发布、掉任一盘」连一个能退回去的根都没有。
- 整段写 0：不写的话，在一块装过上一份 singlefs 的盘上重 mkfs，旧文件系统留在槽 1..S−1 里的根记录自证校验和照样通过、`checkpoint_txg` 还更大 ⇒ 第一次挂载择到的是上一个文件系统的根，而 I-7.1（根槽有效集非空） 与 I-7.3（环健康性） 全绿（推的，没量）。
- [0, 根环基址) 整段写 0：三方判决 `research/prompts/m3-c651-r1-main-verification.md` 第三节「用过的盘上重做 mkfs 留下的旧池系统配置槽怎么处置」那一行与用户定案（清零之后，用过的盘换槽距重做 mkfs 再撕两盘槽 0 不再择到旧池残留的槽 1：攻方在副本装置上量过、主 agent 复跑坐实，入库这一侧由 `remaking_the_filesystem_with_another_io_min_leaves_no_slot_one_of_the_earlier_pool_for_either_reader` 那条用例判）；`research/prompts/m3-formal-r6-main-verification.md` 第二节 mkfs 中途崩溃那一格（清零那一句站住，收掉的是 mkfs 第一道屏障之后那一格）。实现在 `crates/singlefs-core/src/make_filesystem.rs` 每盘清零循环的第一句，用例 `crates/singlefs-harness/tests/system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery.rs` 的 `remaking_the_filesystem_with_another_io_min_leaves_no_slot_one_of_the_earlier_pool_for_either_reader`。
- 用户定案，原话在变更史。
- 系统配置见证读按盘读、判不出取 R2：三方判决 `research/prompts/m3-ring-r2-main-verification.md` 第二节第 9 行（按盘读站得住）与 `research/prompts/m3-ring-r3-main-verification.md` 第二节第 7、8 行（按盘读加每盘合起来混合形丢写 240，界按类写；R2 零轮）、第四节末「R2 验证」段（R2 在 C331（择根倒挂压过已确认的写） 网格上量过：纯形 120、混合形 0，混合形多拒 116 格；八趟网格由事务员在新副本上复核，与辩方日志逐键逐行字节相同，网格不在仓里）。用户定案：改取 R2（原话在变更史）；某块盘一份自证过的系统配置都没有时取号那一步也拒，用户 2026-10-03 定、2026-10-19 开补写例外（原话在变更史）。无实验：数都在三方副本与事务员的副本上，没立实验号。
- mkfs 两趟写序与中途崩溃的条文：三方判决 `research/prompts/m3-close-c656-r1-main-verification.md` 第二节 V2（乙站住，条件是两趟写序）、`research/prompts/m3-close-c656-r2-main-verification.md` 第二节 U2、`research/prompts/m3-close-c656-r3-main-verification.md` 第二节 S2、S3（合成形态与两趟写序站住，挂载那两个分句删掉，条文取辩方报告 `research/prompts/m3-close-c656-r3-sonnet-output.md`「三之末」那一稿，零轮措辞）；用户定案 2026-10-20「认，照改写稿写回」，原话在变更史。无实验：数在三方副本上量（段序列 `2+12+1+1+1+4`、27 步、带撕裂 31 195 态），没立实验号。
- fsid 由对外入口生成、前 8 字节取自随机源：同一份第三轮判决第二节 S1 生成那一格（边界在第 8 / 9 字节，两腿量过）与第三节；用户定案 2026-10-20「对外生成，留内层入口」，原话在变更史。
- 取号写两轮、每块盘只回卷一次：三方判决 `research/prompts/m3-close-sysconf-r3-main-verification.md` 第二节 U1（取号写两轮第二、三两轮攻方没打中、撞号 0，每块盘只回卷一次攻过一轮没打中，副本量过）；用户定案 2026-10-19「罩，取号写两轮」、2026-10-20 维持，原话在变更史。
- 取号那一步的补写例外：三方判决 `research/prompts/m3-close-sysconf-r2-main-verification.md` 第四节、`research/prompts/m3-close-sysconf-r3-main-verification.md` 第二节 U4 与第三节（克隆盘只写了单元那一形打中，推的；补的单元区扫描与环槽读错两样被攻过零轮）；用户定案 2026-10-19「开例外，四件闸」、2026-10-20 采两样补法，原话在变更史。
- 判不出盘面的离线出路：同一份第三轮判决第二节 U2（不带 `birth_txg` 闸时断言前一版丢已确认的写，攻方量过）与第三节；用户定案 2026-10-19「许，另加扫单元写序」、2026-10-20「离线出路加 birth_txg 闸」（零轮），原话在变更史。

**欠**：C79（系统配置与 journal 环的放置没人定）：journal 环两盘镜像那一支做成能强制进入的分支（系统配置每盘一份、整盘掉了两支已能强制进入，用例见 C79（系统配置与 journal 环的放置没人定） 行）。

```

## 三、实现今天的样子

行号都是这一次现查的（`grep -n`、`sed -n`）。HEAD `841b3beab`；工作区里下表几份文件另有别的会话暂存的改动（`git status --short` 记 `M`），行号按工作区读。

| 东西 | 在哪 | 今天是什么样 |
|---|---|---|
| 取号写两轮 | `crates/singlefs-core/src/transaction.rs:990–996`（文档：「取号写两轮（甲'……）：每轮逐盘写一次系统配置槽……每轮之后一道屏障，第二轮那道屏障完成才把新号交出去」）、`:1214` `const ACQUISITION_WRITE_ROUNDS: [AcquisitionWriteRound; 2]`（私有）、`:1235` `for _round in ACQUISITION_WRITE_ROUNDS {` | 每次取号两轮、每轮两盘各一次系统配置槽写、轮后一道池屏障。变异表 `crates/mutations.tsv:4991`「甲' 取号写两轮 只写一轮」改的就是 `:1235` 那一行 |
| 切段 | `crates/singlefs-harness/src/memory_pool.rs` 的 `writes_and_segments`（同 r2 第三节，这一次没重读） | 两轮各成一段：两轮之间与第二轮之后各一道屏障 |
| crates 测试里钉的第一条流 | `crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs`：段长 `:816` `[2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2]`、写数紧跟其后 43；两态闭式 `:774` 16 777 243；全量 `:778` 16 777 268；甲二 `:781` 54；读出文件 `:785` 12 | 已是两轮的数；当第四节 4.2 的第三方对照，装置不引 |
| crates 测试里钉的第二条流 | `crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs`：到 E 再卸载那一支 `:517–535`（80 段的段序列、写数式 `199 + 5 * 33 + (2 + 2 * 21) + 33 + (2 + 2 * 21)` = 485）；根数 `:556–563` 19；全量 `:107` 1 678 377 220；按发布 `:115–120`；甲二 `:124` 294 | 同上 |
| crates 测试里钉的原地覆写数 | `crates/singlefs-checker-tier/tests/in_place_overwrite_torn_state_count.rs:310` `("system_configuration_slot", 8, 8)`、`:317` 16 777 240、`:323` `(16_777_260, 26, 46)` | 还是一轮的数（别的批在改，调查报告第六节第 6 条）；这一次不拿它对照 A5 |
| 装置：两条流的段形常量 | `e161_crash_state_dedup_and_time_split.rs`（下同）`:576` `FIRST_STREAM_SEGMENT_LENGTHS: [usize; 11]`；`:577` 41；`:579` 3；`:580–584` `SECOND_STREAM_SEGMENT_LENGTHS: [usize; 78]`；`:585` 481；`:586` 19；`:588–589` 取样段 11、26；`:590` `SHARED_PREFIX_SEGMENTS` 11 | 一轮的段形 |
| 装置：取样域 | `:2789` `STRIDED_SEGMENT_WRITES: usize = 24` | 不用改（1.4） |
| 装置：锚点 | `:4051` `ANCHOR_FIRST_STREAM_SEGMENT_STATES: [u64; 11]`；`:4052` 16 777 260；`:4054` 按发布；`:4056–4058` 第二条流 16 段；`:4059` 17 170 476；`:4061` 按发布；`:4062–4069` D1 各域与段头；`:4071–4072` 原地覆写 8、46；`:105` `SECOND_STREAM_FULL_STATES` 1 678 377 204；`:107` 最大一段 268 435 455 | 一轮的数 |
| 装置：PC3 | `:5180` `TIMING_CONTROL_SEGMENT: usize = 16`；`:5182` 写数 18；`:5807` 字段名 `second_segment_{TIMING_CONTROL_SEGMENT}_pc3` | 字段名随常量变 |
| 装置：config 行 | `:5578` `rerun_registration=research/prompts/e161-r2-prereg.md domain=first_small_45+quick_46_52+largest_segment_heads_4096` | 要改 |
| 装置：内联测试（`grep -c '#\[test\]'` 现数 15 条） | `:5918` A2 那条（`:5944–5946` 掩码宽 49）；`:5961` A4 那条；`:5884–5885` 域守卫 45；`:6001` 第二条流那条（78、481、19、393 261、7 260、52、4 096、18、262 143、1 678 377 204、268 435 455）；`:6090` 45；`:6135` 注释 45；`:6177–6178` `45 * TIMING_CONTROL_BUSY_WAIT`；`:6212` `mask_bytes_per_state(49)` | 一轮的数 |
| 变异表 | `crates/mutations.tsv` 第 1089–1100 行（M1–M12）、第 2346–2349 行（M13–M16），名字都以 `E161 crash_state_dedup` 起头（行号随别的会话的改动会移，跑时按名字现查） | M14 的原文 `const TIMING_CONTROL_SEGMENT: usize = 16;`、M15 的原文 `…FIRST_SAMPLED_SEGMENT: usize = 11;` 在装置改完之后不再命中，要照改后的源码重写 |
| 复跑驱动 | `scripts/experiment-apparatus/replay.sh:1645` `E161|@driver_e161_feasibility||e161-crash-state-dedup-and-time-split-feasibility-r2-2026-10-05.out|timing`；`:2052–2053` 驱动（`E161_THREADS=10`） | 新产物落盘之后第 1645 行的产物名换成新产物 |
| 工作区里与这次有关的未提交改动 | 装置暂存 3 行（`git diff --cached --stat` +2 / −1：`runs.push(path)` 挪到建成之后），不碰段形与锚点；`transaction.rs` 暂存 +944 / −192 行（`git diff --cached --numstat`，别的会话） | 调查报告第一节说两轮已在 HEAD `0b926bd3f` 里；这一次不靠它：S2 在跑的那一刻核段形 |

## 四、跑之前已经存在的数

读条款、读实现、读调查报告、判问法时已经知道或自己算出的，照实列；判据在第六节，不照这些数挪。r2 产物与 `research/results/` 下任何内容、实验页的结果与结论各节都没读；r2 产物只取了 sha256 与行数（第十一节比对对象表）。

| # | 数 | 出处 | 对判据的影响 |
|---|---|---|---|
| 4.1 | 调查员在副本上用今天的 crates 编的装置跑 `stop-clauses` 读到：第一条流写 43、段 12、根 3，逐段 `[8, 8, 3, 1, 8, 3, 1, 8, 16777215, 3, 1, 8]`，全域 16 777 268，按发布 `[20, 12, 16777227, 8]`，D1-small 53、D1-stride 7 260、D1-quick 54、段头 4 096，原地覆写 10，w 53；第二条流写 485、段 80、根 19；`anchor_segment stream=first segment=1 … writes=2 in_place_overwrites=2 counted_by_the_plan=8 counted_by_the_crates=8 counted_by_the_formula=8 three_agree=true`。只把 `transaction.rs:1235` 换成只取一轮时，旧钉值全部对上 | 调查报告第三节 3.1、第六节第 8 条（报告转述的输出，原件 `e161-{now,head,one-round}.out` 没读） | 行 1、行 2 的答案大半已知。门槛不照它定：第七节 A2–A9 由 `anchors_e161_r3.py` 从 crates 测试的段长字面量独立算，执行员量出的数要等于第七节，等于调查员的数不算数 |
| 4.2 | crates 测试钉的今天的数：第一条流全量 16 777 268、甲二 54、读出文件 12、两态闭式 16 777 243；第二条流 80 段、485 次写、19 条根、全量 1 678 377 220、甲二 294、按发布 `instance1_txg1=20 instance1_txg2=12 instance1_txg3=16777227 instance1_txg4=16777227 instance2_txg5=262171 …` | 第三节「crates 测试里钉的」两行 | 当第七节锚点的第三方对照：`anchors_e161_r3.py` 独立算出的整条流全量、甲二与按发布逐格等于它们（第十三节原样输出），说明 m 的认法（r2 那一条）在两轮的段形上与 crates 仍一致 |
| 4.3 | 我算的这一次的域与锚点：第一条流逐段 `[8, 8, 3, 1, 8, 3, 1, 8, 16777215, 3, 1, 8]`、全量 16 777 268、按发布 `[20, 12, 16777227, 8]`、D1-small 53、D1-stride 7 260、D1-quick 54、段头 4 096、原地覆写 10、W 53；第二条流取样段 12–28 逐段 `[16777215, 3, 1, 8, 8, 8, 262143, 3, 1, 8, 65535, 3, 1, 8, 65535, 3, 1]`、和 17 170 484、按发布 `[16777219, 262171, 65547, 65547]`、D1-small 393 269、D1-quick 60、原地覆写 50、W 535；整条第二条流最大一段 268 435 455、全量 1 678 377 220 | 第十三节 `anchors_e161_r3.py` | 进第七节 A2–A9 |
| 4.4 | 两轮多出来的段在哪：第一条流段 1（取号第二轮，8 个状态）；第二条流段 1（同）与段 17（B 之后重开那次取号的第二轮，8 个状态）。r2 的 D1-small 45 → 53、D1-quick 46 → 54、第二条流 D1-small 393 261 → 393 269、D1-quick 52 → 60，每处恰好多一个 8 | `anchors_e161_r3.py` 与 r2 第七节 A2–A4 并排 | 行 3 里 `first_small`、`first_quick`、`second_quick` 三格的域比 r2 多一段，第十节 F3 的已知差别表照写 |
| 4.5 | 读法甲的门槛 19 与 24 都得 53 / 393 269，25、26 得 16 777 268 / 17 170 484 | `anchors_e161_r3.py` | 第八节几何敏感性那一格 |
| 4.6 | r2 产物 `e161-crash-state-dedup-and-time-split-feasibility-r2-2026-10-05.out`：sha256 `265d97ff420579077920ba38fc0624b21e62fc9f169a659e451cf02faf83d8cb`、186 行（`sha256sum`、`wc -l`，内容没读） | 第十三节命令 | 行 3 的比对对象；V9 核它在不在、行数对不对 |
| 4.7 | 128 位键撞车概率上界：全部域合起来不同键至多 17 170 484 + 16 777 268 = 33 947 752 个，n² / 2¹²⁹ ≈ 1.7 × 10⁻²⁴ | 同 r2 4.8 的算法 | 同 r2：内容 hash 不靠这个界，逐字节核 |
| 4.8 | r2 4.6 的内存峰值（只在本机成立）：`feasibility` 1 814 036 480 字节（10 线程）；13 条单测整跑 8 688 500 736 字节 | r2 第四节 4.6（这一次没重读 `memory-peaks.tsv`） | 定 5.8 的内存上限；这一次域与 r2 同量级（多 8 + 8 + 8 + 8 个状态） |

## 五、臂、阳性对照、真实基线

### 5.1 每个状态上装置做的事

同 r2 5.1（第 174–176 行），一步不改。

### 5.2 各行的臂

同 r2 5.2（第 178–190 行），读法乙那一行不再有（r2 已删）。行 1 的「独立锚点」换成第七节 A2（`anchors_e161_r3.py`），段数 11 换成 12。

### 5.3 阳性对照（每条臂都跑）

同 r2 5.3（第 192–203 行）的 PC1–PC5 与 PCr1a–PCr4，改这几处：PC3 的段换成第二条流段 18；PCr1c 的「M1、M2 改的是装置计划在段 0（与段 3、6、10）、段 2 上的数」改成「M1 改的是装置计划在系统配置槽段 0、1、4、7、11 上的数，M2 改每一段；施加时头一个不成立的必然是『A1：段 0 装置对 crates』」（段号出自 `anchors_e161_r3.py` 末行）；PCr3 的比对对象换成 r2 产物。

### 5.4 真实基线

同 r2 5.4（第 205–207 行）。行 3 拿来比的「上一次」是 r2 产物，不另跑旧代码。

### 5.5 取样域（替换 r2 5.5 第 209–218 行）

| 域 | 包含哪些状态 | 状态数 | 这一次跑不跑 |
|---|---|---|---|
| D1-small | 第一条流段 0–7、9–11 与全部持久那一个；第二条流段 13–28 | 53 + 393 269 | 第一条流跑；第二条流只当锚点 A4 |
| 段头 | 第一条流段 8、第二条流段 12 各段内序号 0–4095 | 4 096 × 2 | 跑 |
| D1-quick | 两条流取样段的甲二快档全部状态 | 54 + 60 | 跑 |
| PC3 的段 | 第二条流段 18（写行 18 个单元写，262 143 个状态）序号最小的 1 000 个 | 1 000 | 跑（六个计时区各一遍） |
| D1-stride | 第一条流段 8、第二条流段 12 里段内序号 ≡ 0 (mod 2311) | 7 260 × 2 | 不跑，只当锚点 A4 |
| 对照样本、D2、D3 | 同 r2 5.5 | — | 不跑 |

### 5.6 装置要改哪几处（要动的文件：`crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs`、`crates/mutations.tsv`、`scripts/experiment-apparatus/replay.sh`）

行号是 2026-10-23 工作区的（第三节）。数一律写成本地字面量，不引 crates 测试里的常量。**表里每一个新值都是执行员要量出来、再与第七节对的数**：先照下表把常量改成第七节的值，再跑 C3，`*_match` 全为 true 才算钉住；对不上的照第十一节 V8 / S2 / S3 判，不把量到的数改进常量来让它对上。

| # | 改哪里 | 改成什么 | 依据（哪一处写序变化） | 验收 |
|---|---|---|---|---|
| J-1 | `:576–579` 第一条流段长、写数、根数与它们的文档注释 | `[usize; 12] = [2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2]`、43、3 | 第一条流开头的取号：一轮 2 写段 → 两轮两个 2 写段（`transaction.rs:1235`） | C3 `name=stop_clause_stream stream=first … writes=43 segments=12 roots=3 segment_lengths_match=true writes_match=true roots_match=true` |
| J-2 | `:580–590` 第二条流段长、写数、根数、取样段、共享前缀 | 段长照 `crash_enumeration_fixed_script_stream.rs:517–535` 那一支的 80 段字面量抄；485；19；取样段 12、28；`SHARED_PREFIX_SEGMENTS` 12 | 第二条流的两次取号（mkfs 那个进程里的、B 之后重开的）各多一个 2 写段 | `stop_clause_stream stream=second … writes=485 segments=80 roots=19` 三个 match 都 true；`stop_clause_shared_prefix … shared_prefix_segments=12 shared_prefix_segments_identical=true` |
| J-3 | `:4051–4072` 锚点；`:105`、`:107` | 照第七节 A2、A3、A4、A5、A7 改（`[u64; 12]`、`[u64; 17]` 等长度跟着改）；`SECOND_STREAM_FULL_STATES` 1 678 377 220，最大一段不变 | 同上 | `name=anchor_stream` 两行各个 `*_match=true`；判决行 `anchors_a1_to_a7_hold=true` |
| J-4 | `:5180` | `TIMING_CONTROL_SEGMENT` 18；`:5182` 写数 18 不变 | 第二条流在写行之前多了段 17 | 判决行 `second_segment_18_pc3=true`，`positive_control_timing` 六行 `states=1000` |
| J-5 | `:5578` config 行；文件头 `:9–10` | `rerun_registration=research/prompts/e161-r3-prereg.md`，`domain=first_small_53+quick_54_60+largest_segment_heads_4096`；文件头加一句第 3 次重跑登记的路径与「取号写两轮之后的段形」 | — | 产物首行 |
| J-6 | 内联测试：`:5884–5885`、`:5944–5946`、`:5961` 起 A4 那条、`:6001` 起第二条流那条、`:6090`、`:6135`、`:6177–6178`、`:6212` | 45 → 53；掩码宽 49 → 53，消息「41 次写 + 8 份撕裂镜像」→「43 次写 + 10 份撕裂镜像」；D1-quick 46 → 54、52 → 60；第二条流 78 → 80、481 → 485、393 261 → 393 269、1 678 377 204 → 1 678 377 220；`mask_bytes_per_state(53)` 仍是 7（⌈53 / 8⌉）；PC3 下限 `53 * …` | 同上 | C2 里这几条 ok |
| J-7 | `crates/mutations.tsv` E161 那 16 行 | 原文仍恰好命中一次的不改；M14 原文改成 `const TIMING_CONTROL_SEGMENT: usize = 18;`、替换文 `= 16;`；M15 原文改成 `…FIRST_SAMPLED_SEGMENT: usize = 12;`、替换文 `= 11;`；变异名里写的旧数（M14「改回 14」、M15「11 改成 10」）照第九节改；必须红的测试名不变 | — | 每条原文在改后的源码里 `count == 1`（用 python 数，不跑变异） |
| J-8 | `scripts/experiment-apparatus/replay.sh:1645` 与 `:2052–2053` | 产物名换成这一次 C3 的产物；`E161_THREADS` 照第十四节 | — | 第十一节比对对象表那一行 |

J-1 到 J-7 编得过：`cargo build --release -p singlefs-checker-tier --bin e161_crash_state_dedup_and_time_split`、同参数的 `cargo clippy … -- -D warnings`、`cargo fmt --check`（经 `scripts/hosts/compile-then-swap.py` 换进主工作区时它自己起的那几条算）。

### 5.7 分段（按岔路排；每段交回之后主 agent 对着 1.2 的四行判续不续，一行开着的都不剩就停）

| 段 | 谁做 | 做什么 | 补哪一行 | 补完够不够判 |
|---|---|---|---|---|
| 第一段 | 执行员 | 5.6 的 J-1 到 J-7；C1 编译；C3 可行性档（执行员自己经内存包装跑，1.6）；产物落 `research/results/`；照第六节 Q1–Q3 判、照 PCr3 先证比对脚本会红；第十一节比对对象表那一行登记 | 行 1 全部（12 段在可行性档里逐个数）、行 2 全部、行 3 全部 | 三行够判；行 4 不够 |
| 第二段 | 主 agent | 5.8 的 C2（内联测试） | 行 4 的内联那一半 | 够判 |
| 第三段 | 主 agent 提交时 | 5.8 的 C4（E161 变异行） | 行 4 的变异那一半 | 够判 |
| 附带 | 执行员 | C3b（线程数 1） | 行 3：只在 Q3 有翻面格时跑 | 够判后不跑 |

第一段停在 S2、S3 的，后面的段不跑，交回主 agent。

### 5.8 命令

工作目录一律 `/home/fy5090/code/singlefs`；`<草稿目录>` 由主 agent 定；产物名里的日期是登记日 2026-10-23（11.3）。主工作区里起会编译的 cargo 一律经 `scripts/hosts/run-build-directory.sh` 包一层（`.claude/hooks/lib_workspace_cargo_lock.py` 文件头「编译目录」一段），范围声明与内存包装写在它外面。开跑前照共用约束看负载；只跑 `--bin e161_crash_state_dedup_and_time_split` 这一个目标，不跑整个 checker 档包（r2 1.6：整包一趟撞过 32G）。

| # | 谁跑 | 命令 | 内存上限与理由 | 限时 | 线程 | 跑完看哪几行 |
|---|---|---|---|---|---|---|
| C1 | 执行员 | `scripts/hosts/compile-then-swap.py` 换装置时它起的编译（或 `nice -n 19 bash scripts/hosts/capped.sh 31 bash scripts/hosts/run-build-directory.sh cargo build --release -p singlefs-checker-tier --bin e161_crash_state_dedup_and_time_split`） | 不跑编出来的代码，不经包装 | — | 31 | 退出码 0 |
| C2 | 主 agent | `{ SINGLEFS_HEAVY_TESTS=commit SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-checker-tier 的装置 e161_crash_state_dedup_and_time_split 的内联测试; 范围=1 个测试目标; 为什么=E161 第 3 次重跑登记行 4：取号写两轮之后的新钉值全绿' RUN_WITH_MEMORY_CAP_TIME_LIMIT=1800 nice -n 19 bash scripts/hosts/capped.sh 31 bash scripts/hosts/run-with-memory-cap.sh 12G bash scripts/hosts/run-build-directory.sh cargo test --release -p singlefs-checker-tier --bin e161_crash_state_dedup_and_time_split -- --test-threads=4; echo "exit=$?"; } > <草稿目录>/e161-r3-c2-inline-tests.log 2>&1` | 12G：同 r2 C2 的理由（r2 4.6 同一批单测整跑峰值 8 688 500 736 字节的约 1.4 倍）；每条评 53 个状态（r2 45） | 1800 秒（含编译） | `--test-threads=4`（同 r2：每条测试自带线程 1、2，四条并行时 12G 内；登记值 31 只给编译） | `exit=`；`test result:`（passed 等于 `grep -c '#\[test\]'` 现数、failed 0、ignored 0）；有 `panicked at` 的连同后面三行 |
| C3 | 执行员 | `{ set -o pipefail; E161_THREADS=31 RUN_WITH_MEMORY_CAP_TIME_LIMIT=1800 nice -n 19 bash scripts/hosts/capped.sh 31 bash scripts/hosts/run-with-memory-cap.sh 8G bash scripts/hosts/run-build-directory.sh cargo run -q --release -p singlefs-checker-tier --bin e161_crash_state_dedup_and_time_split -- feasibility \| grep '^E7RESULT ' > research/results/e161-crash-state-dedup-and-time-split-feasibility-r3-2026-10-23.out; echo "exit=${PIPESTATUS[0]}"; } 2> <草稿目录>/e161-r3-c3-feasibility.err` | 8G：`replay.sh` 复跑的默认上限（`REPLAY_MEMORY_CAP`），取同一个数让复跑与这一次同条件；r2 4.6 的 10 线程峰值 1.8 GB，31 线程推的不过 3 倍（没量） | 1800 秒（含编译；r2 10 线程约 3 分钟） | `E161_THREADS=31`（登记值，第十四节） | `exit=`（err 文件里）；产物行数等于 `name=done emitted=`；`name=verdict` 一行；`name=anchor_segment stream=first` 12 行；`name=anchor_stream` 两行；跑完记 `memory-peaks.tsv` 里这条命令的峰值行 |
| C3b（附带，够判后不跑） | 执行员 | 同 C3，`E161_THREADS=1`、`RUN_WITH_MEMORY_CAP_TIME_LIMIT=7200`，产物写 `e161-crash-state-dedup-and-time-split-feasibility-r3-threads1-2026-10-23.out` | 8G | 7200 秒 | 1 | 与 C3 产物去掉计时字段（名字以 `_elapsed_ns`、`_ratio` 结尾的）与 config 行的 `threads=` 之后逐行相同 |
| C4 | 主 agent 提交时 | 照 `.claude/skills/crash-test/commit-stages.md` 跑 `checker-tier-crates-mutation-replay`（它逐条经包装） | 阶段自己的 | 阶段自己的 | 阶段自己的 | 变异名以 `E161 crash_state_dedup` 起头的每一行，抓到、无效、没红三栏分开数 |

表里命令格中的 `\|` 是表格转义，跑时写成 `|`。内存或限时撞了，照第十一节 S8 停，不加大上限、不放长限时重跑。

## 六、报哪些量与各自的判据

每个量各占一行、各报各的判定，合取留给人。门槛不能从臂定义推出的理由同 r2 第六节开头（第 267 行）。

### 行 1

| 量 | 怎么算 | 门槛 | 判定 | 对应的行与翻面取值 |
|---|---|---|---|---|
| Q1a 第一条流段形 | C3 产物 `name=stop_clause_stream stream=first` 的 `writes`、`segments`、`roots` 与 `segment_lengths_match` | 43、12、3、true | 不符走停机条款 S2，不判行 1 | 行 1 的前提 |
| Q1b 不等段数 | C3 产物 `name=anchor_segment stream=first` 12 行，每行 `counted_by_the_plan`、`counted_by_the_crates`、`counted_by_the_formula` 与第七节 A2 同一段的数，四个不全相等的段数 | 钉绝对值：= 0 | = 0 记「相等」；≥ 1 逐段列四个数；「装置 = crates ≠ 公式」的段另走 F1 | 行 1：任一段不等即翻 |
| Q1c 逐个数到的段数 | 同那 12 行里 `in_domain_counted_one_by_one=true` 的行数 | = 12 | 少于 12：缺的段记「没跑到」，行 1 对那几段不判 | 行 1 够判条件 |
| Q1d 内联那一半 | C2 里 `first_stream_plan_counts_match_the_formula_the_crates_and_the_anchor` 的结局 | ok | 同 r2 Q1d（段 8 在这条测试里不逐个数） | 行 1 的前一半 |

### 行 2

| 量 | 怎么算 | 门槛 | 判定 | 对应的行与翻面取值 |
|---|---|---|---|---|
| Q2a 域大小 | C3 产物 `name=anchor_stream` 两行的 `d1_small`、`d1_stride`、`d1_quick`、`d1_heads` 与对应的 `*_match`；C2 里 A4 那条与第二条流那条测试 | 第七节 A4：第一条流 53、7 260、54、4 096；第二条流 393 269、7 260、60、4 096 | 全等记「域写死且装置算得对」；不等走 V8 或 S3 | 行 2 够判条件 |
| Q2b 跑得完 | C2、C3 各自的包装退出码与峰值行 | 不是 250、253 | 是：记「跑不完」，走 S8 | 行 2 翻面观测一 |
| Q2c 没有对象的判据格数 | 同 r2 Q2c 九样（第 284 行），数换成：① 五格 `states` 各等于 53、54、4 096、60、4 096；⑤ `pc3_segment_holds=true` 的段是 18；其余照抄 | 钉绝对值：= 0 | = 0 记「每格有对象」；≥ 1 逐格列 | 行 2 翻面观测二 |

### 行 3

比法同 r2 行 3（第 287–299 行：按 `键=值` 拆、键的组成、不比的行、字段分类表），比对对象换成 r2 产物（第十一节比对对象表 #1），对名表换成 `second_segment_16_pc3` ↔ `second_segment_18_pc3`；`shared_prefix_segments` 的值 11 ↔ 12 属「不比方向」。

| 量 | 怎么算 | 门槛 | 判定 | 对应的行与翻面取值 |
|---|---|---|---|---|
| Q3a 判决行翻面数 | `name=verdict scope=feasibility` 那一行，按 r2 字段分类表逐字段比 | 钉绝对值：= 0 | = 0 记「同向」；≥ 1 逐字段列 | 行 3：任一判决字段不同即翻 |
| Q3b G 行各格翻面数 | 其余参与比的行，按键配对、逐字段比 | 钉绝对值：= 0 | 同上，逐格列 | 行 3 |
| Q3c 不可比的键与字段 | 只出现在一边的键与字段（对名表之外） | — | 报数并逐个列 | 行 3：哪些格比不了 |
| Q3d 翻面格的已知变化（附带，够判后不跑） | 对每个翻面格照第十节 F3 的表写它所在的域与 r2 差在哪 | — | 报 | 不改判定 |

### 行 4

| 量 | 怎么算 | 门槛 | 判定 | 对应的行与翻面取值 |
|---|---|---|---|---|
| Q4a 内联测试 | C2 的 `test result:` 行 | passed 等于 `grep -c '#\[test\]'` 现数、failed 0、ignored 0 | 记「全绿」或逐条列红的测试与 panic 那一句 | 行 4 |
| Q4b E161 变异行 | C4 日志里 E161 那几行（第九节 M1–M16），抓到、无效、没红三栏分数 | 抓到数等于行数；无效 0；没红 0 | 记「全抓」或逐条列 | 行 4 |
| Q4c 变异原文唯一 | 执行员交回前用 python 数（J-7），提交前门禁 code-source-discipline 的 mutation-tables 格再判 | 每条恰好 1 次 | 不是 1 次：那几条这一趟没跑（`.claude/rules/mutation-sampling.md` 第七类），Q4b 不判 | 行 4 的前提 |

## 七、钉绝对值的断言

### 7.1 出自被测条款本身的（不符走第十节 F1「条款可能错」）

| 编号 | 断言 | 出自 |
|---|---|---|
| A1 | 同 r2 A1（第 322 行）：每段状态数 = 3^m · 2^(n−m) − 1，整个域 = 1 + Σ（只取一部分段时不加 1）；装置计划、crates 只展开那一段、公式三个数逐段相等。这一次第一条流 12 段全在取样段里、全逐个数（Q1c） | D13 已定项 4（r2 第二节整抄） |
| A1b | 每次取号在录制流里是两个 2 写段，两段都是系统配置槽写（m = 2，各 3² − 1 = 8 个状态），中间与之后各一道屏障 | D22 已定项 8「取号写两轮」（第二节）与 `transaction.rs:990–996` 的文档；条款只写了「两轮」，「轮后各一道屏障、各成一段」出自实现的文档，不符时两边都查（S2） |

### 7.2 独立算出、用命令核过的（不符才作废，第十一节 V8）

命令是第十三节的 `anchors_e161_r3.py`：它从两份 crates 测试文件里按断言消息取出段长字面量（不手抄），m 的认法同 r2（第 3 行注释）；整条流的全量、甲二与按发布逐格等于 crates 测试钉的数（第四节 4.2），这是第三方对照，不是它的输入。

| 编号 | 断言 | 值 | 第三方（crates 测试）有没有同一个数 |
|---|---|---|---|
| A2 | 第一条流逐段状态数；全量；按发布 | `[8, 8, 3, 1, 8, 3, 1, 8, 16777215, 3, 1, 8]`；16 777 268；`[20, 12, 16777227, 8]` | 全量 `:778` 同；按发布 txg 1、2、3 三格与第二条流文件 `:115–116` 的 `instance1_txg1=20 instance1_txg2=12 instance1_txg3=16777227` 同 |
| A3 | 第二条流取样段 12–28 逐段状态数；和；按发布（只留取样段里大于 0 的组） | `[16777215, 3, 1, 8, 8, 8, 262143, 3, 1, 8, 65535, 3, 1, 8, 65535, 3, 1]`；17 170 484；`[16777219, 262171, 65547, 65547]` | txg 5、6、7 三格同（`instance2_txg5=262171`、`instance2_txg6=65547`、`instance2_txg7=65547`）；txg 4 那一组少了段 11 的 8（不在取样段里），crates 整条流那一格是 16 777 227 |
| A4 | 取样域大小 | 第一条流 D1-small 53、D1-stride 7 260、D1-quick 54、段头 4 096；第二条流 D1-small 393 269、D1-stride 7 260、D1-quick 60、段头 4 096 | 第一条流 D1-quick 54 同（`:781`）；其余没有 |
| A5 | W = 录制流写数 + 取三态的写数 + 重放数 | 第一条流 43 + 10 + 重放数；第二条流 485 + 50 + 重放数（重放数 0 时 53、535）；处置同 r2 A5 | 取三态的数 crates 那一份（`in_place_overwrite_torn_state_count.rs:310`）还是一轮的 8，不拿它对照（第三节） |
| A6 | 容量与预算 | 同 r2 A6，不变 | — |
| A7 | 第二条流整条流最大一段；整条流全量 | 268 435 455；1 678 377 220 | 全量 `:107` 同 |
| A8 | 两条流的段形 | 第一条流 12 段、43 次写、3 条根；第二条流 80 段、485 次写、19 条根；第二条流前 12 段的段长与原地覆写数逐段等于第一条流（`prefix: second[:12] == first True m equal True`）；两轮多出来的段：第一条流段 1，第二条流段 1 与段 17 | 段长与写数同 crates 测试；根数第二条流 `:556–563` 同 |
| A9 | PC3 的段 | 第二条流里写数 18 的段只有段 18，状态数 262 143 | 没有 |
| A10 | 第一条流 D1-small 里读出文件的状态数 | 12（同 r2 A10 的推法：读出文件的 12 个全在段 9–11 与全部持久那一个里） | 取自 crates `:785` `FILE_READ_STATES`；不符走 S4 |
| A11 | PC3 缩小版多出的计时下限 | 53 × 2 ms × 0.9 = 95 400 000 纳秒 | 没有 |

### 7.3 互比旁边的绝对值

同 r2 7.3（第 341–345 行）：行 1 四个数互比旁边钉 A2；行 3 判决行的 S1–S6 与 `anchors_a1_to_a7_hold` 在新产物里必须是 `true`（不靠与 r2 相同来判）；行 4 passed 数钉在 `grep -c` 现数上。

## 八、轨迹与几何敏感性

### 8.1 轨迹

同 r2 8.1（第 349–351 行）：被谓词消费的量都是有限确定枚举数完之后的个数，没有「轮」，峰值、为正轮数、期末值三样不适用；Q2b 的内存报峰值。正文与交回不写「从来」「恒」。

### 8.2 几何敏感性

| 旋钮 | 取样点 | 方向 | 重判哪几行 | 记法 |
|---|---|---|---|---|
| 取号轮数 | 一轮（r2 的段形，`transaction.rs:1235` 只取一轮）与两轮（今天） | 一个 2 写段 / 两个 2 写段 | 行 2 的「取样段按同一意图认」：一轮时 `anchors_e161_r3.py` 那条规则给回 r2 的 11–26，两轮时给 12–28（同一条规则，起点 = 第一条流段数，终点 = 其后第 4 个 1 写段）；一轮那一点由 r2 第七节 A3 的段序列核，设计时已算完 | 两点都不翻，记两点 |
| 读法甲的门槛 | 19、24、25、26 | 门槛挪到 24 与 26 之间 | 行 2 的域大小：19、24 得 53 / 393 269，25、26 得 16 777 268 / 17 170 484 | 判别力自证：门槛挪到 25 时域守卫必须由绿转红，由变异 M13 落实 |
| 流 | 第一条流 12 段与第二条流取样段 17 段 | 段数、原地覆写的分布不同 | 行 1 的四方相等：C3 产物里第二条流 17 段的 `anchor_segment` 同样逐段比（附带，不改行 1 的判定） | 翻面记「不稳定」，不翻记两点 |
| 线程数 | `E161_THREADS=31` 与 1 | 并发有无 | 行 3：C3 与 C3b 去掉计时字段与 `threads=` 之后逐行相同 | 附带：只在 Q3 有翻面格时跑；不同走 V6 |

## 九、变异

E161 的变异行在 `crates/mutations.tsv`（第三节），六段格式照文件头。16 条意思不变，取样点按今天的段形重写；M14、M15 的原文与替换文跟着常量改（J-7）。

| 编号 | 改什么 | 在哪个取样点上改变输出 | 哪条测试红 |
|---|---|---|---|
| M1 | 装置计划里原地覆写的写取两态（3 → 2） | 第一条流段 0（取号第一轮）：装置计划 8 → 3，crates 仍 8；段 1、4、7、11 同 | `first_stream_plan_counts_match_the_formula_the_crates_and_the_anchor`（A1 段 0 装置对 crates） |
| M2 | 装置计划每段不减 1 | 第一条流段 0：8 → 9 | 同上 |
| M3–M7、M9–M12 | 同 r2 第九节（第 371–380 行） | 同 r2；M4、M5 的取样点是第一条流 D1-small（读出文件的 12 个在段 9–11 与全部持久那一个里，A10），M7 是 53 个状态、看 journal 那一区忙等 2 ms 多出 95.4 ms 以上（A11） | 同 r2 |
| M8 | 掩码宽只数录制流的写 | 第一条流：53 → 43 | A2 那条测试（A5） |
| M13 | `STRIDED_SEGMENT_WRITES` 24 → 26 | 第一条流 D1-small：53 → 16 777 268 | `stride_and_quick_domains_have_the_registered_sizes`（A4），且在变异门禁的限时与内存里退出（PCr2a，靠域守卫） |
| M14 | `TIMING_CONTROL_SEGMENT` 18 → 16（变异名改成「PC3 的段改回 16」） | 第二条流段 16 是取号第一轮那 2 写、8 个状态：「写数 18、不少于 1 000 个」不成立 | `second_stream_shape_and_sampled_domains_have_the_registered_sizes` |
| M15 | 第二条流第一个取样段 12 → 11（变异名改成「12 改成 11」） | 取样段前面多出段 11（第一条流 A 的轮换，8 个状态），D1-small 393 269 → 393 277 | 同上（A3、A4） |
| M16 | 装置公式里的 3 换成 2 | 第一条流段 0：公式 8 → 3，装置计划与 crates 仍 8 | A2 那条测试（A1 crates 对公式） |

## 十、失败条款

同 r2 第十节 F1–F5（第 391–397 行），段数 11 换成 12、比对对象换成 r2 产物；都是正当结果，照实记，不作废。各条「什么观测会让它触发」：

| 编号 | 条款 | 什么观测会让它触发 |
|---|---|---|
| F1 | 条款可能错（行 1）：某段装置计划 = crates ≠ 公式 | C3 某一行 `name=anchor_segment stream=first` 上 `counted_by_the_plan = counted_by_the_crates ≠ counted_by_the_formula`；或 C2 里 A2 那条红在「A1：段 N crates 对公式」 |
| F2 | 行 1 翻到「这几段不等」，按 r2 F2 的四种样子分开记 | Q1b 那 12 行里任一行四个数不全相等 |
| F3 | 行 3 翻：逐格列翻面，每格写它所在格与 r2 的已知差别（下表）与「r2 产物之后 crates 改过」（执行员跑时记 `git log --oneline 31e47115..HEAD -- crates/singlefs-core crates/singlefs-checker crates/singlefs-checker-tier/src/crash.rs` 的条数与 `git diff HEAD --stat -- crates` 的原样输出）；不归因，交主 agent | 任一判决字段或 G 行一格在新旧产物里落在不同的类 |
| F4 | 行 4 翻，按 r2 F4 分 | C2 `test result:` 里 failed ≥ 1；C4 里 E161 那几行有没红或无效 |
| F5 | 行 2 翻：跑不完，或某格失去对象 | C2 或 C3 的包装退 250、253；Q2c 九样里任一样不成立 |

F3 用的已知差别表（设计时按 5.5 与第四节 4.4 写，跑之前定下）：

| 格 | 与 r2 的域差在哪 |
|---|---|
| `first_small` | r2 45 个；这一次 53 个 = r2 那 45 个（段号 0、2–7、9–11 与全部持久，崩溃镜像里取号多写了一轮）+ 段 1 的 8 个（取号第二轮撕裂或没持久） |
| `first_quick` | 46 → 54：多段 1 的 8 个 |
| `first_segment_head` | 同一批写（A 的前 12 次单元写的全部子集），段号 7 → 8；基镜像之上已持久的写多两次系统配置槽写 |
| `second_quick` | 52 → 60：多段 17（B 之后重开那次取号的第二轮）的 8 个 |
| `second_segment_head` | 同 `first_segment_head`，B 的单元写，段号 11 → 12 |
| PC3 | 同一批写、同 1 000 个序号；段号 16 → 18 |

## 十一、作废条款与停机条款

### 11.1 作废

| 编号 | 条件 | 作废什么 |
|---|---|---|
| V1–V7 | 同 r2 11.1 的 V1–V7（第 416 行，即第 1 次 11.1 的 V1–V7），只看可行性档跑到的那几格 | 同 r2 |
| V8 | 第七节 A2–A11（A5 的重放数那一句除外，A10 见那一格）有一条与装置算出的数对不上，而装置与 crates 一致（C3 的 S3、S4 过） | 那一次 C3 的行 1、行 2 判定；重算锚点（改 `anchors_e161_r3.py` 是新的一次登记修订，写在第十二节，时点在产物之前；产物之后发现的，这一次作废、重跑） |
| V9 | r2 产物（比对对象表 #1）不在、sha256 不等于表里那一格、或行数不等于它自己 `name=done emitted=` 的数 | 行 3 不判（照共用约束「找不到历史实验的数据」去 git 里取，取到再判） |
| V10 | C3 产物行数不等于 `name=done emitted=` 的数 | 那一次 C3 |
| V11 | 包装退 251、252、254 | 那一次的输出不算结果、也不算「跑不完」 |
| V12 | PCr3 两项有一项没看到 | 行 3 的比对 |

### 11.2 停机（装置与 `crates/` 对不上：两边都查，既不作废也不当结果，写清哪一处对不上，交回主 agent）

| 编号 | 核什么 | 对不上时 |
|---|---|---|
| S1 | 同 r2 S1：十四个本地常量与 crates 回比 | 停 |
| S2 | 两条流的段形与写数、根数照第七节 A8；两条流基镜像逐字节相同；第二条流前 12 段的写与第一条流 12 段逐字节相同（`shared_prefix_segments=12 shared_prefix_segments_identical=true`）；第一条流段 0、1 与第二条流段 16、17 都是两块盘各一次系统配置槽写（A1b；执行员从产物的段表或装置的写表读写的种类核） | 停。段形不是 A8 的样子，说明今天的写序不是这份登记认的「取号写两轮」（例如工作区里的 `transaction.rs` 又改了），取样段与锚点要重定，交主 agent |
| S3 | 同 r2 S3（第 429 行）：第一条流 D1-small、两条流 D1-quick 上装置的持久集合与写表与 crates 枚举逐个相同；A1 里装置计划与 crates 逐段相等 | 停 |
| S4 | 同 r2 S4（第 430 行） | 停 |
| S5 | 同 r2 S5 | 停 |
| S6 | 同 r2 S6 | 只停 K4-walk 这条臂 |
| S8 | 域跑不完：C2 或 C3 的包装退 250 或 253 | 停。输出不算结果；不加大上限、不放长限时重跑；交回里写命令原样、退出码、`memory-peaks.tsv` 那一行、最后在跑的测试名或产物最后一行；行 2 记「跑不完」（F5） |
| S9 | 域守卫：C3 退 3 而判决行 `anchors_a1_to_a7_hold=false`，或 C2 里共用函数的「D1-small 等于 53」先断 | 停，不评状态；照 V8 与 S3 的分法判是锚点错还是装置算错 |

### 11.3 比对对象

行 3 的「上一次」。sha256 与行数是 2026-10-23 现算的（`sha256sum`、`wc -l`），内容没读。重跑产物名写全名，日期取登记这一天（2026-10-23），C3 哪一天跑都照这个名字落，不改日期；同名文件已在就停下交回（共用约束「执行前拒绝的写法」⑤）。

| # | 比对对象 | sha256 | 行数 | `replay.sh` 行 / 跑法 | 重跑产物名 | 备注 |
|---|---|---|---|---|---|---|
| 1 | `e161-crash-state-dedup-and-time-split-feasibility-r2-2026-10-05.out` | `265d97ff420579077920ba38fc0624b21e62fc9f169a659e451cf02faf83d8cb` | 186 | `1645` `driver_e161_feasibility`（比法 `timing`） | `e161-crash-state-dedup-and-time-split-feasibility-r3-2026-10-23.out` | 第 2 次重跑的可行性档；E161 没有单元表，`.claude/gate.d/replay-comparison-units.tsv` 里要由执行员加一行「# 不登记的比对对象：research/prompts/e161-r3-prereg.md #1：E161 装置没有正式跑单元表」，加之前门禁 doc-experiments 的 replay-comparison-units 格判红 |

### 11.4 够判停机

每段交回附 1.2 那张表，行 1–4 逐行写「已够判 / 还差什么 / 剩下的量能不能让它翻面」。四行都够判就停，没跑的量与段（C3b、Q3d、第八节附带格）逐条标「够判后未跑」，这是合法结局。

## 十二、修订

（留空：装置写之后、产物之前由执行员写，只许收严或补臂，写明改了什么、依据哪个读数、时点在产物之前。）

### 12.1 C3 起法：判段上 GPU（2026-10-26，设计员照主 agent 派发写；时点在第一次试跑之后、任何有效产物之前）

这一条只改 C3（与跟着它的 C3b）的起法，补作废情形与要记的出处；第六节各量的门槛、第十节、第十一节已有各条与 5.8 C3 的内存上限、限时、线程原样不动，原文都留着。依据的读数：

- 第一次试跑（主 agent 2026-10-25 照 5.8 C3 原样起，原样留在 `/home/fy5090/.local/share/singlefs/work/m3-close-b16-e161-r3-run/c3-try1-not-run-without-gpu-feature.out`，sha256 `5a912d04bbde9d3320e31d3745fd6ac938860c709c2bc2d43ba755036e92f064`，183 行）：err 文件同目录 `e161-r3-c3-feasibility.err` 末行 `exit=77`；输出第 100、101、144 行三格 `name=stop_clause_crates … judge_stage=not_run`，第 182 行 `E7RESULT name=verdict scope=feasibility clause_S3_holds=not_run clause_S4_holds=not_run judge_stage=not_run`，第 183 行 `E7RESULT name=done emitted=183 loadavg1=14.24 cpus=32`。
- 原因：装置 `crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs:3015–3021`（`CRATES_COMPARISON_NOT_RUN_EXIT_CODE` 的文档：多机配置开了 GPU 而二进制没编进 `gpu` 特性时 crates 那一遍枚举的判段不跑、只录，要结论编进 `--features verdict-store,gpu` 起）；选路在 `crates/singlefs-checker-tier/src/crash_state_enumeration.rs:176`（配置开 GPU 且编进特性才上卡）、`:1247`（没编进特性：判段报本次未跑，不退回 CPU）；用户 2026-10-13 定「配 GPU 必用 GPU」（`crates/singlefs-checker-tier/src/crash_amplification.rs:1396–1399`）。装置收尾 `:5838–5843` 打上面那行判决、退 77。
- 复跑表起这只装置已经是这个样子：`scripts/experiment-apparatus/replay.sh` 文件头 GPU 一段（第 72–84 行）与驱动 `driver_e161_feasibility`（第 2068–2069 行）在配置开了 GPU 时带 `--features verdict-store,gpu`、经 `scripts/crash/gpu-quiesce-run.sh` 起，上限 `REPLAY_MEMORY_CAP`（第 92 行，默认 8G）。

#### 12.1.1 C3 改成这样起（主 agent 跑；C3b 若要跑，同样加特性与清场，其余照 5.8 C3b）

| 项 | 改成什么 | 依据 |
|---|---|---|
| 特性 | `cargo run … -p singlefs-checker-tier --features verdict-store,gpu --bin e161_crash_state_dedup_and_time_split -- feasibility` | 上面三条依据 |
| 清场 | 整条经 `bash scripts/crash/gpu-quiesce-run.sh -- …` 起，跑完不复原 | `.claude/rules/verification.md`「凡用 GPU 先清场，跑完不复原」：带 `gpu` 特性的 cargo 算在内 |
| 清场包在哪一层 | 包在 `run-with-memory-cap.sh` 外面：8G 与 1800 秒罩的仍是 5.8 C3 罩的那一条（`run-build-directory.sh cargo run …`），清场等锁、停服务的时间不算进限时 | 与 `replay.sh:2069` 包在里面不同；差的只是清场脚本自己的 bash 与清场时间，不碰被测的域 |
| 显卡 | 不给 `GPU_CARDS_BY_PRIORITY`，命令里先 `unset`：清场之下装置照多机配置的 `GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED` 取卡 | `crates/singlefs-checker-tier/src/gpu_card_configuration.rs:145–160`（看 `GPU_QUIESCE_HOLDER` 指的登记文件在不在）、`:193–215`（环境里设了就照环境，漏进来的会顶掉清场之后那一份） |
| 内存上限 | 不变，8G | 见 12.4 |
| 限时 | 不变，`RUN_WITH_MEMORY_CAP_TIME_LIMIT=1800`（含编译） | 行 2 的翻面观测是「在写死的内存上限与限时里跑不完」，放大上限或限时就是放宽 Q2b、F5、S8，修订里不许 |
| 线程 | 不变，`capped.sh 31`、`E161_THREADS=31` | 第十四节登记值；判段的 CPU 一侧线程数取 `SINGLEFS_THREAD_CAP`（`capped.sh` 设成 31），不另设 |
| 落盘 | 先写进草稿目录，跑完核过下面「挪进产物名的条件」再 `mv -n` 成 11.3 的产物名 | 第一次试跑直接写在产物名上、留下一份本次未跑的输出，要主 agent 挪走；先落草稿目录，11.3「同名已在就停」只会碰到有效产物 |
| 原始标准输出 | `tee` 一份进草稿目录 | A1b 要读 `CRASH_FLOW_COVERAGE` 行（执行员报告「要主 agent 跑的命令」第 3 条）；判段走哪一路也只在这里（12.1.3） |

命令（工作目录 `/home/fy5090/code/singlefs`；`<草稿目录>` 由主 agent 定，可以沿用 `/home/fy5090/.local/share/singlefs/work/m3-close-b16-e161-r3-run`）：

```bash
ls -l research/results/e161-crash-state-dedup-and-time-split-feasibility-r3-2026-10-23.out   # 应当报不存在；在就停（11.3）
{ set -o pipefail; unset GPU_CARDS_BY_PRIORITY; E161_THREADS=31 RUN_WITH_MEMORY_CAP_TIME_LIMIT=1800 nice -n 19 bash scripts/hosts/capped.sh 31 bash scripts/crash/gpu-quiesce-run.sh -- bash scripts/hosts/run-with-memory-cap.sh 8G bash scripts/hosts/run-build-directory.sh cargo run -q --release -p singlefs-checker-tier --features verdict-store,gpu --bin e161_crash_state_dedup_and_time_split -- feasibility | tee <草稿目录>/e161-r3-c3-gpu-stdout.raw | grep '^E7RESULT ' > <草稿目录>/e161-r3-c3-gpu-feasibility.out; echo "exit=${PIPESTATUS[0]}" >&2; } 2> <草稿目录>/e161-r3-c3-gpu-feasibility.err
```

跑完先读这几样（读完再决定挪不挪）：

```bash
grep -E '^GPU_QUIESCE |^exit=' <草稿目录>/e161-r3-c3-gpu-feasibility.err
wc -l < <草稿目录>/e161-r3-c3-gpu-feasibility.out; tail -n 1 <草稿目录>/e161-r3-c3-gpu-feasibility.out
grep -c '^NOT_RUN_' <草稿目录>/e161-r3-c3-gpu-stdout.raw
grep -E '^CRASH_POINT_REPLAY_JUDGE |^CRASH_CASE_BLOCKS ' <草稿目录>/e161-r3-c3-gpu-stdout.raw
```

挪进产物名的条件（四样都成立）：`exit=` 是 0 或 3（装置 `:5856–5866`：0 是 S1–S4 等都过，3 是有停机条款没过或域守卫，两种都是有判决行的产物）；err 里有 `GPU_QUIESCE step=command result=exit-0` 或 `result=exit-3`；行数等于末行 `name=done emitted=`（V10）；原始标准输出里 `NOT_RUN_` 开头的行数是 0。成立就挪：

```bash
mv -n <草稿目录>/e161-r3-c3-gpu-feasibility.out research/results/e161-crash-state-dedup-and-time-split-feasibility-r3-2026-10-23.out
ls -l <草稿目录>/e161-r3-c3-gpu-feasibility.out research/results/e161-crash-state-dedup-and-time-split-feasibility-r3-2026-10-23.out   # 前者应当不在了
```

不成立的留在草稿目录，改名带上结局（照第一次试跑的写法，例 `c3-try2-<结局>.out`），照 12.1.2 记，不进 `research/results/`。

#### 12.1.2 退出码怎么记（补 V11，只收严）

| 结局 | 怎么认 | 记法 |
|---|---|---|
| 判段本次未跑 | `exit=77`（装置 `:5843`），或原始标准输出里有 `NOT_RUN_` 行 | 补进 V11：那一次的输出不算结果、也不算「跑不完」；核命令里带了 `--features verdict-store,gpu`，改好再起 |
| 清场没过、命令没起 | `exit=78` 且 err 里 `GPU_QUIESCE step=command result=not-started` | 补进 V11；等卡上不归清场白名单的计算进程跑完再起，不停它（`.claude/rules/verification.md` 同一条） |
| 装置拒卡配置 | `exit=78` 且 `GPU_QUIESCE step=command result=exit-78` | 补进 V11；照报错改多机配置，交主 agent |
| 显存不够 | `exit=245`（`scripts/hosts/run-with-memory-cap.sh:28`） | 补进 V11：那一次的输出不算结果 |
| 撞内存上限、超时 | `exit=250`、`253` | 照 S8，不改；另报 err 里被杀之前有没有 `progress cell=` 行（没有就是停在编译里），连同 `memory-peaks.tsv` 里这条命令的峰值行 |
| 装置 panic、编不过 | `exit=101` 或别的码 | 不算结果，交回主 agent |

V11 原有的 251、252、254 不变。

#### 12.1.3 判段上 GPU 之后行 3 的可比性（比法照第六节行 3 与 r2 的字段分类表，一格不放宽）

r2 那一趟的判段在 CPU 上判，依据：r2 产物 `research/results/e161-crash-state-dedup-and-time-split-feasibility-r2-2026-10-05.out` 由 `18b30adbf`（2026-10-10）入库，那时的装置（`f8a9ddaab` 版第 3023 行）S3、S4 调的是 `singlefs_checker_tier::crash::enumerate_layer0_in_state_slices`（层 0 直接枚举，不经判定库），`f8a9ddaab` 与 `bc57af7a9` 两版的 `crash.rs` 里 `gpu` 零处（`git show <提交>:crates/singlefs-checker-tier/src/crash.rs | grep -ci gpu` 都是 0），装置里 `crash_amplification` 与 `gpu` 零处；带 GPU 选路的 `crash_state_enumeration.rs` 由 `d6dd6da42`（2026-10-14）加进来，晚于 r2 跑的日子；r2 登记里 `gpu`、`cpu` 零处；r2 产物里 `gpu`、`route=`、`judge_stage` 这类字段零处（只数了字段名，没读值），首行 `threads=10`。

这一次：crates 那一遍分两段，录入段（含 S3 的观察者，逐状态比写表与持久集合）在 CPU 上跑（`crash_state_enumeration.rs:110`），判段在 GPU 上出 crates 那一侧的计数；装置自己的照跑、记录跑、G 行与 PC3 都在装置自己的 CPU 路径上（`gpu-exempt` 标记在装置 `:2041`、`:2231`、`:5532`），而且按次序在那条流的 crates 比对做完之后才跑（`:5694–5718` 在 `:5753` 之前，PC3 在 `:5827`，全部之后），两者不并发。逐格：

| 行 3 里的格 | 判段换路碰不碰它 | 照登记怎么记 |
|---|---|---|
| 判决行 `clause_S3_holds`；`stop_clause_crates` 三格的 `writes_table_differs`、`persisted_mismatches`、`clause_S3_holds` | 不碰：观察在录入段、CPU 上 | 照字段分类表比（布尔、钉零）。另有一处与 GPU 无关的已知变化：crates 那一侧从层 0 直接枚举改成经判定库两段（`d6dd6da42` 起），翻面时写进 F3 的说明 |
| 判决行 `clause_S4_holds`；`stop_clause_crates` 三格的 `clause_S4_holds` | 碰：装置照跑的计数（CPU）比的 crates 计数改由 GPU 判器出（装置 `:4343–4352` 比两份计数文本是否相同） | 照布尔比，判法不变。7.3 要求新产物里 S4 为 true；是 false 先走 11.2 S4 停机（两边都查：把同一格 `stop_clause_tally side=device` 与 `side=crates` 两行逐字段并排，分清是 GPU 判器少报或多报的字段，还是同一字段值不同），停机没查清之前这一格不当行 3 的翻面 |
| `stop_clause_crates` 三格的 `domain_states`、`crates_observed_states`、`compared_states` | — | 「不比方向」，照旧 |
| `stop_clause_tally` | 碰（crates 那一侧） | r2 行 3 本来就不比这一种行，照旧；只在上一格停机时并排读 |
| `stop_clause_crates_per_state`（段头上的 S4） | 不碰：装置调 crates 的单状态入口，在 CPU 上（`:5530–5533`） | 照旧比 |
| G 行 `g1`、`g5_contents` 各字段 | 不碰：内容决定的计数与布尔 | 照旧比 |
| G 行 `g3` 的计时字段（`*_elapsed_ns`、`*_ratio`） | 不碰判法；计时环境见下一行 | 「不比方向」，照旧 |
| 判决行 `g1`–`g5` | 不碰：两边都是字面 `not_judged_feasibility_only`（`:5846`） | 字面相等，照旧 |
| PC3 六行（`positive_control_timing` 的 `busy_zone_added_at_least_1800ms`、`other_zones_changed_under_1000ms`、`pc3_zone_holds`、`pc3_segment_holds`） | 不碰判法：计时在装置 CPU 路径上、在全部 GPU 工作之后。推的、没量：进程里留着 GPU 判器建的对象，清场停掉了白名单里的服务，后台负载与 r2 那一趟不同 | 照布尔比，门槛不动；收尾行的 `loadavg1`、`cpus` 与 err 里的 `GPU_QUIESCE` 行抄进实验页；翻面时写进 F3 的说明 |

F3 的已知差别表补两行（只写给翻面格的说明用，不改判定）：

| 格 | 与 r2 的差在哪 |
|---|---|
| 判决行与 `stop_clause_crates` 的 S3、S4 | r2 的 crates 那一侧在 CPU 上经层 0 直接枚举；这一次经判定库两段，S3 的观察仍在 CPU，S4 比的 crates 计数由 GPU 判器出 |
| PC3、G 行计时 | 这一次在 GPU 清场之下跑，进程里先跑过 GPU 判段 |

要记的出处（补，不进判定）：原始标准输出里 `CRASH_POINT_REPLAY_JUDGE` 行（有现判时打，`crash_state_enumeration.rs:1879` 是 `route=gpu` 那种，`:1584` 是 `route=cpu` 那种）与 `CRASH_CASE_BLOCKS` 行（`:2819`，现判与复用的块数）原样抄进实验页；`route=cpu` 出现说明那一刻多机配置没开 GPU，照记，不作废。

比对对象的取法：2026-10-26 现查，r2 产物在工作区里是暂存删除（`git status --short` 记 `D`），`git show HEAD:research/results/e161-crash-state-dedup-and-time-split-feasibility-r2-2026-10-05.out` 取到的 sha256 是 `265d97ff420579077920ba38fc0624b21e62fc9f169a659e451cf02faf83d8cb`、186 行，与 11.3 表相同；行 3 照 V9 那一句从 git 取这一份比，不算 V9 打中。

#### 12.1.4 第一次试跑怎么算

- 不是产物：退 77 是本次未跑（12.1.2 第一行），判决行没有 S1–S6，不判行 1–3，也不当 Q2b「跑得完」的证据。
- 它不占 11.3 的产物名：2026-10-26 现查，那个名字在 `research/results/` 下不在；修订后的跑法先落草稿目录，有效的才挪成这个名字，「同名已在就停」照旧指那一刻已在的文件。
- 留一笔：实验页记「第一次试跑（2026-10-25）判段没跑」，写路径、sha256、行数、`exit=77` 与原因（没编进 `gpu` 特性），只当经过，不当出处；输出留在执行员草稿目录，不挪进 `research/results/`。
- 第二次起照 12.1.1；还有没成的，照同一格式往这一条后面接着记（第几次、结局、路径、sha256）。

#### 12.1.5 4.8 的内存峰值与 5.8 的上限

- 4.8 写的 r2 峰值取错了行：1 814 036 480 是 `~/.local/share/singlefs/memory-peaks.tsv` 里 2026-09-27 那一行（命令是 `-p singlefs-harness` 起的第 1 次）；r2 那一趟（2026-10-05，`-p singlefs-checker-tier … feasibility`，产物首行 `threads=10`）的峰值行是 3 420 454 912。4.8 原文不改，以这一条为准。
- 第一次试跑（2026-10-25，31 线程，`run-build-directory.sh` 现编，不带 GPU 判段）的峰值行是 7 253 000 192，8G（8 589 934 592）的 84.4%，余 1 336 934 400。这个峰值含页缓存（`memory-peaks.tsv` 第 2 行的口径），退 250 要 scope 里真有 oom-kill（`run-with-memory-cap.sh:30`），页缓存回收得掉，峰值近上限本身不等于会撞。
- 带特性之后多出来的（推的，没量）：同一个 scope 里多编 `wgpu` 一族；判段在主机一侧的缓冲；三格 crates 比对各多一遍装置照跑（53、54、60 个状态，`:4333–4341` 的 `BaselineOnly`）。
- 上限不重定，仍 8G；限时仍 1800 秒。理由：放大就是放宽行 2 的翻面观测（12.1.1 限时那一格）；8G 也是复跑表起同一只装置、带同一组特性时的上限（`replay.sh:92`），与复跑同条件。撞了照 S8（12.1.2），要换上限是新的一次登记，由主 agent 照 `evidence-discipline.md`「臂的定义也在「跑前写死」之列——失败条款打中的时候怎么办」办。
- 第十四节的预计挂钟随之多出编 `wgpu` 一族与清场的时间（推的，几分钟量级），预计在用核数照旧约 31 个核，另占清场之后那一份配置里的卡。

#### 12.1.6 写这一条时读到的数（照第四节的规矩列，不删）

读了第一次试跑输出的行名计数（`awk '{print $2}' | sort | uniq -c`）、第 1、100、101、144、182、183 行，`grep -i 'gpu\|device\|cuda\|not_run'` 命中的第 103、128、146、160 行（各前 300 字：两格段头 `stop_clause_tally side=device` 的计数与两行 `positive_control_device_memory`）；err 文件全份，其中七个 `pc3-*` 格的 `progress … elapsed_seconds=` 是 17–19 秒。G 行、`reuse_arm`、`positive_control_timing` 的值没读。这些数一条都没进判据；判据在 12.1 之前已写死，这一条没动任何门槛。

### 12.2 C2 起法：带 `gpu` 特性、清场，认出「passed 而判段没跑」那一条（2026-10-26，设计员照主 agent 派发写；时点在 C2 任何有效结果之前）

这一条只改 C2 的起法，给 Q4a 补「本次未跑」的认法（只收严）；第六节 Q4a 的门槛、第十节 F4、第十一节已有各条、5.8 C2 的内存上限与限时原样不动，原文都留着。依据的读数（行号都是这一次现查的，装置 sha256 `cc3954157aad6edf6a030369f0856c27589c4ea13c73dcd59e4a190af377fdab`，6 379 行）：

- 内联测试 `device_tally_equals_the_crates_tally_on_the_first_stream_small_domain`（装置 `:6238`）调 `report_crates_comparison`（`:6253`）。判段没跑时那个函数打两行、交 `CratesComparisonClauses::JudgeStageNotRun`（`:4318–4331`），测试在 `:6268–6275` 的 `let … else { return; }` 直接返回，libtest 照样记 passed。15 条内联测试里调它的只有这一条（`:5871` 起的 `mod tests` 里 `report_crates_comparison` 只命中 `:6253`）。
- 判段没跑的成因有三种，都让 crates 的 `enumerate_crash_states_with_expansion` 交只录、`.judged()` 交 `None`（`crash_library.rs:705–709`：`RecordedOnly`、`JudgedMyShareOnly` 都是 `None`），走的是同一个 return：多机配置开了 GPU 而没编进 `gpu` 特性（`crash_state_enumeration.rs:171–182` 选路、`:1247–1249` 交只录）；环境设了 `SINGLEFS_CRASH_CASE_BLOCKS_QUERY=1`（`:951–953`、`:1187–1189`）；两台分片里只判了自己那一份。5.8 C2 不带特性，在多机配置开着 GPU 的这台上起，就是第一种（推的：同一成因在 C3 第一次试跑上实测打中，12.1 依据第一条）。

#### 12.2.1 C2 改成这样起（主 agent 跑）

| 项 | 改成什么 | 依据 |
|---|---|---|
| 特性 | `cargo test --release -p singlefs-checker-tier --features verdict-store,gpu --bin e161_crash_state_dedup_and_time_split` | 同 12.1.1 特性那一格 |
| 清场 | 经 `bash scripts/crash/gpu-quiesce-run.sh --` 起，包在 `run-with-memory-cap.sh` 外面、`capped.sh` 里面，跑完不复原 | `.claude/rules/verification.md`「凡用 GPU 先清场，跑完不复原」（带 `gpu` 特性的 cargo、手跑的 GPU 测试都算）；包在哪一层同 12.1.1：12G 与 1800 秒罩的仍是 5.8 C2 罩的那一条 |
| 显卡 | 起之前 `unset GPU_CARDS_BY_PRIORITY` | 同 12.1.1 显卡那一格 |
| 只查库不判的开关 | 起之前 `unset SINGLEFS_CRASH_CASE_BLOCKS_QUERY` | 设成 1 时判段交只录，这条测试同样空过（上面第二条依据） |
| `--test-threads` | 4 改成 1 | `.claude/rules/verification.md`「两档：轻型与重型」节末「单个 GPU 测试二进制只用本机点名的卡，带 `--test-threads 1` 跑」：带 `gpu` 特性编出来的这份测试二进制是一份 GPU 测试二进制。并行降下来，内存只会更低、挂钟只会更长，是收严 |
| 通过的测试的输出 | 加 `--show-output` | libtest 默认吞掉通过的测试的标准输出：不加它，12.2.2 要数的那几行都不进日志，数出来的 0 是空的。不用 `--nocapture`：单线程时 libtest 先打「test <名> ... 」不换行，测试打的头一行接在它后面 |
| 日志名 | `.log` 改成 `.out` | `.claude/hooks/two-logs-guard.sh` 文件头：`cargo test` 的输出用 `>` 写到 `.log` 的执行前拒、`.out` 不拦；5.8 C2 原样起不来 |
| 内存上限 | 不变，12G | 12.2.4 |
| 限时 | 不变，`RUN_WITH_MEMORY_CAP_TIME_LIMIT=1800`（含编译） | 理由同 12.1.1 限时那一格：放大就是放宽 Q2b、F5、S8；推的挂钟见 12.2.4 |
| 线程 | 编译不变，`capped.sh 31` | `--test-threads` 只管测试之间的并行；判段 CPU 一侧取 `SINGLEFS_THREAD_CAP`（`capped.sh` 设成 31；`.claude/rules/verification.md` 同一节「GPU 对拍与内存对拍不压并发」那一条），不另设 |
| `SINGLEFS_HEAVY_TESTS=commit`、范围声明 | 不变（为什么那一段多写半句带 `gpu` 特性） | — |

命令（工作目录 `/home/fy5090/code/singlefs`；`<草稿目录>` 由主 agent 定）：

```bash
{ unset GPU_CARDS_BY_PRIORITY SINGLEFS_CRASH_CASE_BLOCKS_QUERY; SINGLEFS_HEAVY_TESTS=commit SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-checker-tier 的装置 e161_crash_state_dedup_and_time_split 的内联测试; 范围=1 个测试目标; 为什么=E161 第 3 次重跑登记行 4：取号写两轮之后的新钉值全绿，带 gpu 特性让判段真跑' RUN_WITH_MEMORY_CAP_TIME_LIMIT=1800 nice -n 19 bash scripts/hosts/capped.sh 31 bash scripts/crash/gpu-quiesce-run.sh -- bash scripts/hosts/run-with-memory-cap.sh 12G bash scripts/hosts/run-build-directory.sh cargo test --release -p singlefs-checker-tier --features verdict-store,gpu --bin e161_crash_state_dedup_and_time_split -- --test-threads=1 --show-output; echo "exit=$?"; } > <草稿目录>/e161-r3-c2-gpu-inline-tests.out 2>&1
```

#### 12.2.2 Q4a 怎么认出「passed 而判段没跑」（补 Q4a 的判法，只收严）

判段没跑时这条测试打的两行（现查原文）：

- 装置 `:4321–4324`：`E7RESULT name=stop_clause_crates cell=test-first-small domain_states=<域大小> judge_stage=not_run`（前缀 `E7RESULT ` 由 `ResultLines::line` 加，`:707–710`）。
- 装置 `:4325–4330` 打 `crash_point_replay_judge_not_run_line("e161_crates_comparison")`（`crash_state_enumeration.rs:188–194`，格式在 `crates/singlefs-harness/src/gpu_memory_ledger.rs:694–700`）：`NOT_RUN_NO_GPU_ADAPTER test=crash_state_enumeration::e161_crates_comparison reason=配置写了 ENABLE_GPU=1，…`。第一种成因下 crates 自己在 `crash_state_enumeration.rs:1248` 再打同样一行。

判段跑了时打的那一行（`:4353–4362`）：`E7RESULT name=stop_clause_crates cell=test-first-small domain_states=… crates_observed_states=… compared_states=… writes_table_differs=… persisted_mismatches=… clause_S3_holds=… clause_S4_holds=…`，后面跟两行 `stop_clause_tally`；测试随后断言 S3、S4 都是 true（`:6276–6277`）。

记号的实样：C3 第一次试跑的原始标准输出（`/home/fy5090/.local/share/singlefs/work/m3-close-b16-e161-r3-run/e161-r3-c3-stdout.raw`，sha256 `9525fdca2d372607991ab8fafe4dcce36350a32aff57c50ec6f7bc1853429ebf`）里 `NOT_RUN_` 6 行，全是 `NOT_RUN_NO_GPU_ADAPTER test=crash_state_enumeration::e161_crates_comparison`（三格各两行：crates 一行、装置一行）；`judge_stage=not_run` 4 行（三格各一行，加判决行）。

跑完照做（`<日志>` 是 `<草稿目录>/e161-r3-c2-gpu-inline-tests.out`）：

```bash
grep -E '^exit=|GPU_QUIESCE ' <日志>
grep -E '^running [0-9]+ tests?$|^test result:' <日志>
grep -c '#\[test\]' crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs
grep -cE 'NOT_RUN_[A-Z_]+ test=' <日志>
grep -c 'judge_stage=not_run' <日志>
grep -cE 'name=stop_clause_crates cell=test-first-small domain_states=[0-9]+ crates_observed_states=[0-9]+ .*clause_S3_holds=true clause_S4_holds=true' <日志>
grep -E 'CRASH_POINT_REPLAY_JUDGE |CRASH_CASE_BLOCKS ' <日志>
```

Q4a 记「全绿」要五样都成立：① `exit=0`，且有 `GPU_QUIESCE step=command result=exit-0`；② `test result: ok.`，passed 等于 `grep -c '#\[test\]'` 现数（12.2.3：15），failed 0、ignored 0；③ `NOT_RUN_… test=` 行数 0；④ `judge_stage=not_run` 行数 0；⑤ 判了的那一行恰好 1 行。⑤ 是正面那一样：只看 ③、④ 的 0，输出没被捕获进日志时也是 0。

| 观测 | 记法 |
|---|---|
| ①–⑤ 都成立 | Q4a「全绿」 |
| failed ≥ 1 | F4 照原样（行 4 翻），红的测试与 panic 那一句照 5.8 C2 列；③–⑤ 照报 |
| failed 0，而 ③ 或 ④ 不是 0、或 ⑤ 不是 1 | Q4a 记「`device_tally_equals_the_crates_tally_on_the_first_stream_small_domain` 本次未跑」，不记全绿也不记翻面；补进 V11：那一次 C2 的 Q4a 不算结果、也不算「跑不完」。按 `NOT_RUN_` 行的 `reason=`、命令里的特性与上面两个变量核成因，改好再起。⑤ 是 0 而 ③、④ 也是 0 的，先核命令里带没带 `--show-output` |
| `exit=` 是 78、245、250、253、251、252、254 | 照 12.1.2 那几行与 S8、V11 |
| `exit=101` 而没有 `test result:` 行 | 编不过，不算结果，交回主 agent |

`CRASH_POINT_REPLAY_JUDGE route=cpu` 出现，说明那一刻多机配置没开 GPU、判段在 CPU 上判了（12.1.3 末段同一读法）：照记，不作废，⑤ 照判。`CRASH_POINT_REPLAY_JUDGE`、`CRASH_CASE_BLOCKS` 两种行（判段全部复用库里的判定时可以没有前一种）原样抄进实验页，只当出处。

#### 12.2.3 带特性那一趟的条数：仍是 15

- 装置里 `grep -c '#\[test\]'` 是 15；`cfg` 只命中 `:5871` 的 `#[cfg(test)]`，`feature` 只命中 `:3017–3020` 的文档注释，没有 `cfg(feature = "gpu")` 一类挂在测试上的；没有 `#[ignore]`、`should_panic`。`#[path]` 引进来的 `crates/singlefs-harness/tests/common/mod.rs`（装置 `:25–26`）里 `#[test]` 0 处、`cfg` 0 处。
- 所以带不带 `gpu` 编都是 15 条，Q4a 的「passed 等于现数」不变；带特性只改 `device_tally_equals_the_crates_tally_on_the_first_stream_small_domain` 的判段走哪一路，不增不减测试。
- 推翻它的观测：日志里 `running N tests` 的 N 不是 15，或 `test result:` 里 passed、failed、ignored 之和不是 15。

#### 12.2.4 内存与限时的依据

- 4.8 写的「13 条单测整跑 8 688 500 736 字节」取的是 `~/.local/share/singlefs/memory-peaks.tsv` 第 19033 行（2026-09-27，`cargo test -p singlefs-harness --bin e161_crash_state_dedup_and_time_split`，那时装置还在 harness 档、debug 档）；r2 C2 那一趟（2026-10-05，`cargo test --release -p singlefs-checker-tier --bin e161_crash_state_dedup_and_time_split -- --test-threads=4`，上限 12G）是第 28468 行，2 230 321 152。4.8 与 5.8 C2 理由那一格原文不改，以这一条为准。
- 同一份装置现编加跑的读数：C3 第一次试跑（2026-10-25，`run-build-directory.sh` 现编，31 线程）第 20562 行 7 253 000 192，含编译的页缓存（12.1.5）。
- 带特性多编 `wgpu` 一族；测试并行 4 改 1，测试那一段的峰值只会更低（推的，没量）。12G 不改：放大是放宽 Q2b；收小没有读数撑着，只会多出与被测无关的 250。
- 限时 1800 秒不改。推的挂钟（没量）：编译那一段，C3 第一次试跑同样现编、整趟在 1800 秒里跑到收尾（退 77 是跑完之后的收尾，12.1 依据第一条），多编 `wgpu` 推几分钟；测试那一段一次一条，至多是 r2 四条并行时测试段的 4 倍，r2 测试段多长没有读数。撞 253 照 S8 停、不放长重跑，交主 agent 照 `evidence-discipline.md`「臂的定义也在「跑前写死」之列——失败条款打中的时候怎么办」办；单线程时 libtest 每跑完一条打一行 `test … ok`，日志里最后一行这样的说明停在哪一条之后。

#### 12.2.5 第十四节随之变的

第十四节原文不改，C2 那几处以这一条为准：「独立单位：」里「`--test-threads=4` 并行四条」改读成一次一条；「线程：」里 C2 那半句改读成 `--test-threads=1` 出自 `.claude/rules/verification.md` 那一条（不再是内存压出来的并行度）；「预计在用核数：」第二段跑测试时改读成一条测试自带 1–2 线程，加判段 CPU 一侧至多 31（`SINGLEFS_THREAD_CAP`），另占清场之后那一份配置里的卡；「预计挂钟：」第二段仍不超过 30 分钟（限时），多出清场等锁的时间（不算进限时）。

#### 12.2.6 写这一条时读到的数与跑过的命令（照第四节的规矩列，不删）

- 读了：装置 `:3010–3060`、`:4295–4375`、`:3130–3200`、`:6225–6300`，`#[test]`、`cfg`、`feature`、`NOT_RUN`、`Judged` 的 grep 命中行；`crash_state_enumeration.rs:150–194`、`:940–960`、`:1175–1192`、`:1230–1265`、`:1875–1900`；`crash_library.rs:705–717`；`crash_amplification.rs:1394–1407`；`gpu_memory_ledger.rs:688–723`；`crates/singlefs-checker-tier/Cargo.toml` 的特性行；`.claude/rules/verification.md` 第 40–138 行；`scripts/crash/gpu-quiesce-run.sh:1–60`；`.claude/hooks/two-logs-guard.sh:1–30`；`scripts/hosts/capped.sh` 里 `RUST_TEST_THREADS`、`SINGLEFS_THREAD_CAP` 的 grep；`scripts/experiment-apparatus/replay.sh:72–92` 与 GPU、bindgen 的 grep；`memory-peaks.tsv` 里命令含 `e161_crash_state_dedup` 的行（日期、上限、峰值、命令）；上一位设计员报告 `/home/fy5090/.local/share/singlefs/work/m3-close-b17-e161-r3-c3-revision/result.md` 第 62–67 行。
- C3 第一次试跑原始标准输出：只数了 `NOT_RUN_` 行与 `judge_stage=not_run` 行，读了三格 `stop_clause_crates … judge_stage=not_run` 的格名与 `domain_states`（上面「记号的实样」）。
- 没读：C2 被停那一趟留下的 `/home/fy5090/.local/share/singlefs/work/m3-close-b16-e161-r3-run/e161-r3-c2-inline-tests.out`（只看了大小，4 794 字节）、`research/results/` 下任何内容。
- 命令：`grep -n`、`grep -c`、`awk 'NR>=…'` 读上面那几处；`sha256sum` 装置与 C3 第一次试跑原始标准输出；`TZ=Asia/Tokyo date +%F`（2026-10-26）。
- 这些数都没进判据：Q4a 的门槛没动，加的是认出本次未跑的五样条件；内存上限、限时没动。

### 12.3 勘误（2026-10-26，主 agent）

12.1.1 表里「内存上限」那一格写的「见 12.4」，登记里没有 12.4，指的是 12.1.5（内存上限与限时）。只改指路，不改任何判据与数。

## 十三、读过的文件与跑过的命令

### 13.1 读过的文件（行号区间；grep 命中行也列）

| 文件 | 读了哪些 |
|---|---|
| `.claude/agent-common.md` | 全份（分两次） |
| `.claude/singlefs-ai-sop/rules/test-discipline.md`、`.claude/rules/implementation-first.md`、`.claude/rules/mutation-sampling.md`、`.claude/rules/three-way-inference.md`、`.claude/singlefs-ai-sop/rules/evidence-discipline.md`、`.claude/rules/test-script-discipline.md` | 全份 |
| 调查报告 `/home/fy5090/.local/share/singlefs/work/m3-close-b16-checker-tier-pins/result.md` | 1–142、206–353（第一至三节 3.1、3.5 至第九节）；没读 3.2–3.4 |
| `research/prompts/e161-r2-prereg.md` | 1–659 全份 |
| `crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs` | 1–20、560–600、5917–5960；常量、锚点、测试名 grep 命中 105、107、125、576–590、2789、4051–4076、4469、5116、5180、5182、5578、5613、5807、5854–6339 的 `#[test]` 与钉值行；6000–6086 里的断言行 |
| `crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs` | 740–860；grep 命中 774、778、781、785、816 |
| `crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs` | 90–130、470–570 |
| `crates/singlefs-checker-tier/tests/in_place_overwrite_torn_state_count.rs` | grep 命中 310、317、323 |
| `crates/singlefs-core/src/transaction.rs` | 985–1000、1205–1240；`ACQUISITION_WRITE_ROUNDS` grep（1214、1235） |
| `crates/mutations.tsv` | `^E161 crash_state_dedup` grep（1089–1100、2346–2349，每行截前 260 字）；`ACQUISITION_WRITE_ROUNDS` grep（4991） |
| `scripts/experiment-apparatus/replay.sh` | `E161\|E165` grep；1645、2049–2053 |
| `.claude/hooks/lib_heavy_tests.py` | 70–82 |
| `.claude/kb/decisions/22-单元原子性怎么合成.md` | 标题 grep；quote-kb 抄的 19、177–202 |
| `.claude/heavy-thread-default`、`.claude/hooks/lib_long_run.py` | 登记值一行；92–94 |
| `research/results/` | 只对 r2 产物跑 `sha256sum`、`wc -l`，没读内容 |

没读：r2 产物与 `research/results/` 下任何内容、`.claude/kb/experiments/` 下 E161 的实验页、`.claude/kb/experiments.md`、`.claude/kb/experiments-history.md`、`records/`、`.claude/kb/decisions-history.md`、`scripts/experiment-apparatus/perf-by-milestone.md`、调查报告点名的 `e161-*.out` 原件。

### 13.2 跑过的命令（原样）与算术的原样输出

```bash
bash scripts/hosts/work-directory-ownership.sh claim /home/fy5090/.local/share/singlefs/work/m3-close-b16-e161-e165-rereg --purpose 'e161 钉值与 e165 KA 域的重跑登记'
TZ=Asia/Tokyo date +%F; git rev-parse --short HEAD          # 2026-10-23；841b3beab
sha256sum research/prompts/e161-r2-prereg.md research/prompts/e161-preregistration.md <装置> …
git status --short -- <几份 crates 文件>; git diff --cached --numstat -- <同>
(cd research/results && sha256sum e161-crash-state-dedup-and-time-split-feasibility-r2-2026-10-05.out; wc -l < 同)   # 265d97ff…；186
nice -n 19 python3 scripts/kb/quote-kb.py <草稿目录>/e161-section2-quote.md "$D~^\| 8 \| \*\*固定结构的放置" "$D@#### 已定项 8：固定结构的放置"   # D 是 D22 那份文件；✓ 2 段整抄，回读逐字节一致
cat <草稿目录>/e161-section2-quote.md >> research/prompts/e161-r3-prereg.md
nice -n 19 python3 -I anchors_e161_r3.py /home/fy5090/code/singlefs    # 在草稿目录的 anchors/ 里跑
nice -n 19 python3 scripts/kb/replace-once.py research/prompts/e161-r3-prereg.md <旧串> <新串>   # 写登记时的定点修正，都在装置改动与产物之前
```

`anchors_e161_r3.py` 全文（草稿目录 `anchors/`）：

```python
# E161 第 3 次重跑登记的算术锚点（独立于装置与 crates 的代码；段长表从两份 crates 测试文件里按断言消息逐字取出字面量，不手抄）。
# 段模型（D13 已定项 4）：每段 3^m * 2^(n-m) - 1；整个域 = 1 + Σ（只取一部分段时不加 1）。
# m 的认法同第 2 次登记 anchors_r2.py 第 3 行：2 写段后面紧跟 1 写段（根槽）的是两份 journal 记录（m=0），其余 2 写段是系统配置槽写（m=2）。
import math, re, sys
root = sys.argv[1]
def literal_before(path, message):
    text = open(path, encoding="utf-8").read()
    at = text.index(message)
    start = text.rindex("vec![", 0, at)
    body = text[start + 5:text.index("]", start)]
    return [int(x) for x in re.findall(r"\d+", body)]
tests = root + "/crates/singlefs-checker-tier/tests/"
first = literal_before(tests + "crash_enumeration_new_pool_file_creation_stream.rs", "取号写两轮、轮后各一道池屏障，各成一段")
second = literal_before(tests + "crash_enumeration_fixed_script_stream.rs", "固定脚本到 E 再正常卸载的段序列")
def ms(segs):
    return [2 if (n == 2 and (i + 1 >= len(segs) or segs[i + 1] != 1)) else 0 for i, n in enumerate(segs)]
def full(n, m): return 3**m * 2**(n - m) - 1
def quick(n, m):
    units = n - m if n > 2 else 0
    return 1 if units > 2 else full(n, m)
for name, segs in [("first", first), ("second", second)]:
    m = ms(segs)
    f = [full(n, k) for n, k in zip(segs, m)]
    q = [quick(n, k) for n, k in zip(segs, m)]
    print(name, "segments", len(segs), "writes", sum(segs), "roots", segs.count(1), "in_place", sum(m), "W(no replay)", sum(segs) + sum(m))
    print(name, "lengths", segs)
    print(name, "per_segment_full", f)
    print(name, "full_domain", 1 + sum(f), "quick_domain", 1 + sum(q), "largest_segment", max(f), "at", f.index(max(f)))
fm, sm = ms(first), ms(second)
ff = [full(n, k) for n, k in zip(first, fm)]
sf = [full(n, k) for n, k in zip(second, sm)]
sq = [quick(n, k) for n, k in zip(second, sm)]
fq = [quick(n, k) for n, k in zip(first, fm)]
P = len(first)
print("prefix: second[:%d] == first" % P, second[:P] == first, "m equal", sm[:P] == fm)
# 取样段：第 2 次登记 1.3 的意图（第一条流整条之后、到 txg 7 的根槽为止；txg 7 的轮换不取），在今天的段序列上按「第一条流之后第一个 1 写段起数第 4 个 1 写段（txg 7 的根）」定终点
ones = [i for i in range(P, len(second)) if second[i] == 1]
S = list(range(P, ones[3] + 1))
print("second sampled", S[0], "-", S[-1], "count", len(S))
print("second sampled lengths", [second[i] for i in S])
print("second sampled full", [sf[i] for i in S], "sum", sum(sf[i] for i in S))
print("second sampled quick", [sq[i] for i in S], "sum", sum(sq[i] for i in S))
for threshold in (19, 24, 25, 26):
    fa = [i for i in range(P) if first[i] < threshold]; sa = [i for i in S if second[i] < threshold]
    print("readingA threshold", threshold, "first small", 1 + sum(ff[i] for i in fa), "second small", sum(sf[i] for i in sa))
big = 2**24 - 1
print("big segments first", [i for i in range(P) if first[i] == 24], "second", [i for i in S if second[i] == 24])
print("stride per 24-write segment", len(range(0, big, 2311)), "heads", min(big, 4096))
def by_publish(segs, counts, sampled):
    groups, run = [], 0
    for i, n in enumerate(segs):
        run += counts[i] if i in sampled else 0
        if n == 1:
            groups.append(run); run = 0
    groups.append(run)
    return groups
print("first by_publish", by_publish(first, ff, set(range(P))))
print("second by_publish (sampled, positive)", [g for g in by_publish(second, sf, set(S)) if g > 0])
print("second whole stream by_publish", by_publish(second, sf, set(range(len(second)))))
print("second whole largest", max(sf))
print("18-write segments in second", [i for i, n in enumerate(second) if n == 18])
print("mask bytes", -(-(sum(first) + sum(fm)) // 8), -(-(sum(second) + sum(sm)) // 8))
print("pc3 busy-wait floor ns", (1 + sum(ff[i] for i in range(P) if first[i] < 24)) * 2_000_000 * 9 // 10)
# 变异取样点：M15（第一个取样段往前挪一段）与 M14（PC3 的段往前挪两段）
print("M15 second small with first sampled", S[0] - 1, sum(sf[i] for i in range(S[0] - 1, S[-1] + 1) if second[i] < 24))
pc = [i for i, n in enumerate(second) if n == 18][0]
print("M14 segment", pc - 2, "writes", second[pc - 2], "states", sf[pc - 2])
print("M1 first stream segments with m=2", [i for i in range(P) if fm[i] == 2], "M1 seg0 3->2 gives", 2**first[0] - 1)
```

`anchors_e161_r3.py` 的 sha256：`0e78ee047614576eb81648efdfddb19f079c8006c928fdc78610ec8303f69d00`；原样输出（sha256 `e56188356a8cfe9030c4d1f26cf2c54afe0e52d895faf308adedc2a901071da7`）：

```text
first segments 12 writes 43 roots 3 in_place 10 W(no replay) 53
first lengths [2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2]
first per_segment_full [8, 8, 3, 1, 8, 3, 1, 8, 16777215, 3, 1, 8]
first full_domain 16777268 quick_domain 54 largest_segment 16777215 at 8
second segments 80 writes 485 roots 19 in_place 50 W(no replay) 535
second lengths [2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 2, 2, 18, 2, 1, 2, 16, 2, 1, 2, 16, 2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2, 2, 16, 2, 1, 2, 16, 2, 1, 2, 28, 2, 1, 2, 2, 16, 2, 1, 2, 16, 2, 1, 2]
second per_segment_full [8, 8, 3, 1, 8, 3, 1, 8, 16777215, 3, 1, 8, 16777215, 3, 1, 8, 8, 8, 262143, 3, 1, 8, 65535, 3, 1, 8, 65535, 3, 1, 8, 16777215, 3, 1, 8, 16777215, 3, 1, 8, 268435455, 3, 1, 8, 268435455, 3, 1, 8, 268435455, 3, 1, 8, 268435455, 3, 1, 8, 268435455, 3, 1, 8, 8, 65535, 3, 1, 8, 65535, 3, 1, 8, 268435455, 3, 1, 8, 8, 65535, 3, 1, 8, 65535, 3, 1, 8]
second full_domain 1678377220 quick_domain 294 largest_segment 268435455 at 38
prefix: second[:12] == first True m equal True
second sampled 12 - 28 count 17
second sampled lengths [24, 2, 1, 2, 2, 2, 18, 2, 1, 2, 16, 2, 1, 2, 16, 2, 1]
second sampled full [16777215, 3, 1, 8, 8, 8, 262143, 3, 1, 8, 65535, 3, 1, 8, 65535, 3, 1] sum 17170484
second sampled quick [1, 3, 1, 8, 8, 8, 1, 3, 1, 8, 1, 3, 1, 8, 1, 3, 1] sum 60
readingA threshold 19 first small 53 second small 393269
readingA threshold 24 first small 53 second small 393269
readingA threshold 25 first small 16777268 second small 17170484
readingA threshold 26 first small 16777268 second small 17170484
big segments first [8] second [12]
stride per 24-write segment 7260 heads 4096
first by_publish [20, 12, 16777227, 8]
second by_publish (sampled, positive) [16777219, 262171, 65547, 65547]
second whole stream by_publish [20, 12, 16777227, 16777227, 262171, 65547, 65547, 16777227, 16777227, 268435467, 268435467, 268435467, 268435467, 268435467, 65555, 65547, 268435467, 65555, 65547, 8]
second whole largest 268435455
18-write segments in second [18]
mask bytes 7 67
pc3 busy-wait floor ns 95400000
M15 second small with first sampled 11 393277
M14 segment 16 writes 2 states 8
M1 first stream segments with m=2 [0, 1, 4, 7, 11] M1 seg0 3->2 gives 3
```

## 十四、并行与挂钟

独立单位：C3 一次里两条流各域的状态彼此独立，装置按 `E161_THREADS` 切给线程（同 r2）；C2 的 15 条内联测试彼此独立（`--test-threads=4` 并行四条）；第一段（执行员）与第二段（主 agent 的 C2）互相独立，可以同时起，C4 等提交。
线程：编译与 C3 取登记值 31（`.claude/heavy-thread-default`，`capped.sh 31`、`E161_THREADS=31`）；C2 的 `--test-threads=4` 是内存上限压出来的并行度（r2 C2 的理由：同一批单测整跑峰值 8.7 GB，四条并行在 12G 内），编译仍是 31。
串行：C3 产物落盘之后才做行 3 的比对与比对对象表登记（比的就是这份产物）；C3 内部 PC3 的六个计时区按区串行（计时是被测量，同 r2 5.3 PC3），其余无。
预计挂钟：第一段约 40 分钟（推的：装置编译约 10 分钟、C3 在 r2 10 线程约 3 分钟、这一次域多 32 个状态与两段 8 态，31 线程不会更慢；比对与登记约 20 分钟）；第二段 C2 不超过 30 分钟（限时 1800 秒，含编译）；第三段随提交的变异整表，不单算。
预计在用核数：第一段编译与 C3 约 31 个核（登记值）；第二段编译 31、跑测试时约 4–8 个核（四条并行、每条自带 1–2 线程）；比对脚本 1 个核。

## 待主 agent 认的

1. **问题单**：派发提示写「岔路单：重跑」，没有另给问题单。1.2 那四行是照 r2 问题单的形状、按派发提示第 1 条的验收改写的，认之前不派执行员。
2. **取样段 12–28**：这是 r2「取段 11–26 的理由」那一句在两轮段形上的同一读法（1.3、8.2）；要换成别的读法就是新的一次登记。
3. **C3 由执行员跑**：照今天 `.claude/hooks/lib_heavy_tests.py` 第 77 行，`cargo run` 起装置本体不算重型（r2 那时把它算进了重型）；钩子拒了就交回，改由主 agent 跑。
4. **线程 31**：C3 与复跑驱动 `driver_e161_feasibility`（`replay.sh:2053`，今天写死 10）改成登记值 31（J-8）。产物的 config 行记 `threads=`，复跑比法 `timing` 不去这个字段，所以驱动不改就复不出；C3 的峰值超过 8G（复跑默认上限）时要给驱动单写上限，那时交回主 agent 定。

## 没做什么（固定会有的）

没写代码、没跑装置；登记在产物之前写死，之后要改由主 agent 按 `evidence-discipline.md`「臂的定义也在「跑前写死」之列——失败条款打中的时候怎么办」那三步处理。准入（`python3 scripts/gate/admission.py experiment <仓根> E161`）没跑，归主 agent。
