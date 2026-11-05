# m3-close-b19-c699-r1 攻方腿报告（2026-11-05）

立场：攻——假设主 agent 的倾向错，给甲、乙、丙各造会错的那一形。只攻派发点名的攻击面（`walk.rs` 的 `hole_in_the_candidate_set_between` 与调用处、甲乙丙的定义、E198 四组镜像、写者的清扫与写行）。模型写了、没跑：子 agent 不交任务，下面凡标「模型上量的」都还是**待主 agent 跑**的预言，主 agent 复跑之前一律按「推的」读。

## 模型与复跑

模型目录 `research/prompts/m3-close-b19-c699-r1-opus-model/`，三份文件（sha256 是写完那一刻 `sha256sum` 现算的）：

```
816f91b7052454355b59c89e4745bc42e9b73d88950239be8e0cb5f6dd61415e  attack_model_appended.rs
e56da699596b8d87696a6cb4581f5d5ce67d1ada7fdc2014a2652a032c6b65b5  model-crate-Cargo.toml
90b19d94a7f11caa2e62456c1f5fb66b08e2a6453d1ce2ee0e833594be776195  tools/rerun.sh
```

- 做法：`tools/rerun.sh`（从 `scripts/agents/three-way-rerun-template.sh` 拷，只换了 `rerun_steps`）把冻结树的 `crates/`、`prune-server/`、`Cargo.toml`、`Cargo.lock`、`.cargo/` 拷进 `--work`，把 E198（取号跳过认出生代号的前提） 装置源码 `crates/singlefs-checker-tier/src/bin/e198_instance_table_pointer_birth_skip_premise.rs` 的 `fn main() {` 改名为 `premise_apparatus_main`、末尾接上 `attack_model_appended.rs`，放进副本里新建的工作区成员 `crates/c699-attack-model`（只依赖 singlefs-core、singlefs-checker、singlefs-harness，不在 checker 档包里），`cargo build --release --offline -p c699-attack-model`，再跑编出来的二进制 `full`。装置的造历史、账本真值、判法读镜像（候选集、由记录施加出来的版本、洞对、今天的臂与甲）原样复用，模型只另加三条臂与七组历史。
- 语法：拼好的源码在草稿目录用 `rustfmt --edition 2021 --check` 解析过（只有排版差异、0 条 `error`）；**没编过、类型没核过**——子 agent 不起 cargo。编不过时主 agent 照「要主 agent 跑的命令」第 1 条的 `build.log` 新派承接腿改。
- 估时闸：二进制先每组跑一段、按组内段数与线程数估全量挂钟，超过 2400 秒就打 `mode=stopped_over_the_estimate_limit`、退 3、不跑全量（定义第 3b 步）。
- 臂（全部只改「接不接得上」这一判，不碰 I-3.5 的 (a)(b) 本身）：

| 臂（产物里的名字） | 定义 |
|---|---|
| 今天 `today` | `walk.rs` 三条件（装置的 H1-A，照抄） |
| 甲 `with_pointer_birth` | 今天 ∧ R 的实例表指针出生 txg = R.txg（装置原样） |
| 乙 `read_bad_gap_slot_guarded` | 甲 ∧ (R⁻.txg, R.txg) 里每个 txg 的落点槽都不是「读得出不是全零而自证不过，或读不出」（背景材料第三节乙的字面） |
| 乙计全零 `read_bad_or_zero_gap_slot_guarded` | 乙再把读回全零的落点槽也算进去（D16（发布语义） 已定项 1 对全零槽与读坏槽同样处置，见附录 D16 第 38 行；乙的字面没并它） |
| 丁 `accounting_generation_guarded`（攻方自提，只在我的模型上，被攻过零轮） | 今天 ∧ R 那条根的分配记录里没有一条代（未释放的分配代、已释放的释放代，隔离记录另列不算）落在 (R⁻.txg, R.txg) 开区间里；R 不是根环里的根或账读不出记 `unknown` |

- 组：`sw-read-control`（E198 sw-read 原形，锚 Q10）、`sw-crash-control`（E198 sw-crash 原形，锚 Q6）、`sw-crash-wrapped`（Q2.2 新形）、`skr-gap-fault`（Q2.1）、`rs-two-files`（Q2.3）、`nrb-recheck` 与 `rs1-recheck`（E198 第一段两组原样重造，只换判法）。取样域与每组为什么这样取写在源码各函数的文档注释里。

## 各格判定一览

| 格 | 判定 | 依据的种类 |
|---|---|---|
| Q2.1 乙的判别子在合法写者造的镜像上 | 今天的崩溃模型里写者自己造不出读坏的落点槽（根槽写一个扇区、只取两态）；读故障能造：skr 的真取号跳过加一个读坏的洞内落点槽，乙把它降成不判（E2）。最小历史给在下面 | 读码坐实（写者一侧）；E2 那一形「模型上量的，待主 agent 跑」 |
| Q2.1 附：乙改成连全零槽一起并（与 D16 同一口径） | 不加任何故障就丢 skr：真取号跳过洞里那一槽在年轻的环上读回全零 | 推的（读码），模型待跑 |
| Q2.2 sw-read 之外三个候选都认错的合法历史 | 打中（待跑坐实）：sw-crash 那一形里清扫实例在清扫之后写大的顺序写，把清扫之前那几版的记录盖掉；实例 4 挂载之后的第一张镜像上洞对 (2@16, 3@20)，三个候选都认作接得上，真值是中间有存在过的版本。洞里三个落点槽住着实例 4 自证过的根，乙一个读坏槽都看不到 | 推的（逐步走码），模型待跑 |
| Q2.2 附：这一形与 sw-read 的判别子观测得到吗 | 观测得到（推的）：R 那条根自己的分配记录里有代落在洞里的释放记录；丁按它判两形都分对、skr 不丢 | 推的，丁被攻过零轮 |
| Q2.3 丙在 C698 两组上走到哪、红在哪 | E198 的 291 张上 I-3.5 判定 0 张红（量的，E198 第一段 Q19）；I-3.9 那 150 张红与洞的分类无关，三个候选一样红（读码坐实）。丙的代价不在这 291 张上，在两个文件的回退形：今天的判法把错认的那一对照常判，I-3.5 红 | 291 张那一半是量的（E198 产物）；两文件那一形「模型上量的，待主 agent 跑」 |
| Q1 辅：三个候选各组判对、认错 | 见 Q1 一节的表（E198 量过的格照抄产物，新格待跑） | 量的 + 推的 |

## Q2.1 乙的判别子：读坏的落点槽能不能出现在真取号跳过的洞里

**结论**：能，但只能由读故障造，写者自己在今天的崩溃模型里造不出。最小历史是 E198 的 skr 原形加一个读坏的洞内落点槽：乙把一对真取号跳过降成不判（E2），今天的臂与甲都分对。另有两处让乙丢判别力的放大：读坏槽认不出它当时住的是哪个 txg；乙若照 D16（发布语义） 的口径把读回全零的槽也并进来，不加任何故障就丢 skr。

**写者一侧为什么造不出（读码坐实）**

1. 取号跳过怎么来：新实例第一次发布的 txg 是环里根与环里自证过的记录两者最大的 txg 加一，`crates/singlefs-core/src/mount.rs:1397`：「新实例的第一次发布的 checkpoint_txg = max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1」，`crates/singlefs-core/src/mount.rs:1419`：「checkpoint_txg_after_the_version(highest_ring_txg.max(highest_record_txg))」。skr 里实例 2 的写行那次发布（txg last + 1）记录落了、根槽没落、之后没被施加，实例 3 就从 last + 2 起号，洞对 (last, last + 2) 的真值是直接接上（E198 第二段产物第 2057 行 C2 锚 `holds=true`，`(6,8):birth8:directly_bridged`）。
2. 洞里那个 txg 的落点槽此刻装的是什么：根槽写落了就是一条自证过的根（那一版就存在过，不再是跳过）；没落就还是旧内容——年轻的环（S = 4 时 3S = 12，last ≤ 10）上是全零，转过一圈的环上是上一圈那条自证过的根。
3. 崩溃不会把这一槽写成半截：`crates/singlefs-checker-tier/src/crash.rs:2087`：「一个扇区的写撕不开（根槽写是一个物理块，照旧两态）」。所以在崩溃点重放的模型里，真取号跳过的洞里那一槽只有「全零」「上一圈的自证根」两种，没有乙要的「读得出不是全零而自证不过，或读不出」。

**读故障一侧：最小反例（模型上量的，待主 agent 跑；格是 `skr-gap-fault`）**

- 历史：E198 的 skr 主点 `skr.main.S4.last6`，k = 0：mkfs → 实例 1 覆盖写到 txg 6 → 实例 2 产品挂载崩在写行那次（txg 7）的根槽写之前 → 实例 3 产品挂载（写行 3@8，C2 那一张）。
- 故障：txg 7 的落点槽（本地落点公式 区域 = 7 mod 3 = 1，槽 = ⌊7 / 3⌋ mod 4 = 2）第 40 字节取反——读坏一个根槽，用户 2026-10-04 定算池级 checker 的合法镜像（附录 C698 那一行，`.claude/kb/checks-owed.md:421`）。那一槽原来全零，取反之后读得出、不是全零、自证不过。
- 走到的判定：今天的三条件成立（行 (1, 6)，实例 3 > 1，中间实例 2 一行 (2, 0, 0)；`crates/singlefs-checker/src/walk.rs:865`：「fn hole_in_the_candidate_set_between(」），甲也成立（C2 锚 `birth8`）；乙看到 (6, 8) 里 txg 7 的落点槽读坏 ⇒ 不认作接得上 ⇒ 用户可见树子树里的落点不判，`crates/singlefs-checker/src/walk.rs:999`：「if hole == HoleInTheCandidateSet::HoleNotBridgedByTheInstanceTableRows」。真值直接接上 ⇒ 乙记一个 E2。
- 预言的数（推的，跑前写下）：`skr-gap-fault` 一组里原样镜像上的洞对数等于 E198 skr 主点 26 对加 G3（S = 6）44 对 = 70 对、真值全是 `directly_bridged`；读坏那一张上同样 70 对，乙的 E2 = 70，今天的臂与甲的 E1、E2 都是 0。理由：洞对 (last, last + 2) 还在的每一张上，last + 1 那一槽一定还没被下一圈写过（下一圈先写到 last + 12，那时 R⁻ 那一槽已经被盖、这一对就没了），取反的就是这一对洞里那一槽。
- 什么会推翻它：`skr-gap-fault` 组汇总行里 `read_bad_gap_slot_guarded_e2` 为 0，或读坏那一张的 `name=pair` 行 `gap_slots=7:bad` 而 `read_bad_gap_slot_guarded=1`。

**两处放大（推的）**

1. 读坏的槽不告诉你它住过哪个 txg：一槽对应 txg ≡ 槽号 (mod 3S) 的一整串。乙按「洞里的 txg 映到槽」判，于是洞对 (R⁻, R) 之外、别的圈住在同一槽里的根读坏了也算到这一对头上。对一对洞长 g 的真取号跳过，「随机一个根槽读坏」打中乙的比例是 g / 3S（skr S = 4 是 1/12）。
2. 读回全零：乙的字面只并「读得出不是全零而自证不过、或读不出」，背景材料说乙「与 D16 已定项 1 把读坏槽当『可能有根』同一个思路」；而 D16 那一行同时把读回全零的槽并进上限（附录 D16 第 38 行）。照这个思路把全零也并进来（模型里的 `read_bad_or_zero_gap_slot_guarded`），skr 原样镜像上洞里那一槽就是全零（上面第 2 条），不加故障就丢掉全部 70 对（预言：原样那一半 E2 = 70，读坏那一半 E2 = 70，组合计 140）。所以「同一个思路」只成立一半：乙必须刻意不并全零，理由与 D16 那一行相反。

**乙的两种写法在这一形上的区别（推的）**：「降成接不上」时这一对的用户可见树落点静默不判，`walk.rs` 的 I-3.5 没有一个字报出来；「报部分没判」时这张镜像的 I-3.5 报 `PartiallyJudged`，看得见丢了判，但判别力一样丢了。

**写者另一侧不在模型里的一形（推的，没量）**：物理块大于一个扇区的池上，根槽写本身能撕开（`crates/singlefs-checker-tier/src/crash.rs:2087` 那一条只对崩溃点重放按 512 字节物理块建的池成立）。那时实例 2 崩在写行那次根槽写的**中途**，洞里那一槽就是撕开的半截（不是全零、自证不过），乙在没有任何读故障的合法崩溃状态上就丢 skr；而且造成跳过的与造成读坏的是同一次崩溃。今天的崩溃枚举不出这一态，模型也没造它。

**打中之后的四问**

1. 分不分辨臂：分辨。只有乙（与乙计全零）在这一形上错，今天的臂与甲分对。
2. 被判的系统当时看不看得到判别它的东西：读坏那一槽看得到；看不到的是那一槽当时住的是哪个 txg、是不是洞里那一版——读坏把它抹掉了。丁看的是 R 自己的分配记录，不靠那一槽（3@8 是写行发布，它的账接在 1@6 上，2@7 没被施加，账里应当没有代 7 的记录：推的，模型的 `gap_generations` 一列会打出来）。
3. 满足判据字面的哪一个分句：E198 口径的 E2「真值直接接上而不认作取号跳过」（E198 实验页「口径」第三条），不是 E1。
4. 跑前条款给的改法在打中的格上还中不中：本轮的改法就是三个候选。甲、丙在这一格不中；乙中；乙计全零在原样镜像上就中。

## Q2.2 sw-read 以外、三个候选都认错的合法历史

**结论（推的，逐步走码；模型格 `sw-crash-wrapped` 待主 agent 跑坐实）**：有。就是 E198 没造出的「记录也被盖掉」的清扫那一形——不用读故障、不用撕裂写，只要清扫实例在清扫之后写得够大，把清扫之前那几版的 journal 记录盖掉。实例 4 挂载之后第一张镜像上，洞对 (2@16, 3@20) 被今天的臂、甲、乙一致认作接得上，真值是中间有存在过的版本（3@17、3@18、3@19 的根槽都落过）。乙之所以挡不住：洞里三个落点槽此刻住着实例 4 自证过的根，没有一槽读坏。

**最小历史（S = 4、a = 4、d = 0，与 E198 `sw-crash.main.S4.ring1048576.a4.d0` 只差清扫之后那几次写的大小）**

| 步 | 发生什么 | 依据 |
|---|---|---|
| 1 | 实例 1 写到 txg 4；实例 2（带清扫配置的挂载）覆盖写到 b = 4 + 12 + 0 = 16 | E198 `build_the_sweep` 同一段 |
| 2 | 实例 3 挂载：写行 3@17、暖机 3@18、3@19、清扫发布 3@20（删实例 1 的行，实例 2 在环里还有根，行 (2, 16) 留着） | E198 第二段产物 sw-crash 历史行 `sweep_outcome=rows_deleted1 sweep_txg=20`（背景材料第一段引的那一行）；清扫只删环里一条根都没有的实例，`crates/singlefs-core/src/instance_table_sweep.rs:325`：「实例表里实例在环里一条根都没有的行」 |
| 3 | 清扫发布重写整条实例表 ⇒ 3@20 的实例表指针出生 txg = 20 | `crates/singlefs-core/src/instance_table_sweep.rs:725`：「instance_table: InstanceTablePlan::Rewrite(self.instance_table.clone()),」；sw-read 同一版 `birth20`（E198 第二段产物第 1524 行） |
| 4 | **用户改动**：实例 3 接着写 3@21 … 3@27，每次是 c × 524 288 字节的顺序写（c ∈ {3, 4}）；1 MiB 的 journal 环被转过去，3@17、3@18、3@19 的记录被盖掉 | 顺序写把 1 MiB 环转过去在 E198 的 ss 一组上发生过（ss c3 第 6 步起出现洞对，那一对就是被施加过的 1@5 记录被盖掉）；这一组上会不会转过去是推的，模型每张镜像打 `applied_only_by_records` |
| 5 | 3@28 那次覆盖写崩在根槽写之前：记录落了，根槽没落，28 的落点槽（= 16 的那一槽）还住着 2@16 | E198 `publish_crashing` 同一刀 |
| 6 | 实例 4 挂载：从 29 起号（环里记录最大 28，`crates/singlefs-core/src/mount.rs:1419` 那一行），写行 4@29（行 (3, 28)），暖机 4@30、4@31，盖掉 17、18、19 的落点槽 | E198 第二段产物第 930 行（k = 0 那一张）`newest=4@31`，候选里有 4@29、4@30、4@31 |
| 7 | 这一张上候选集：2@16、3@20 … 3@27、3@28（由记录施加出来）、4@29 … 4@31；3@17 … 3@19 根槽已被盖、记录也读不出 ⇒ 不进候选集 ⇒ 洞对 (2@16, 3@20) | `walk.rs` 由记录施加出来那一判要有读得出的记录，`crates/singlefs-checker/src/walk.rs:12920`：「let its_units_are_not_reclaimable_yet = oldest_valid_root_txg < record.checkpoint_txg;」与它上面 ①；E198 原形里这三版正是靠记录补上的（第 930 行 `applied_only_by_records=3@17,3@18,3@19,3@28`） |

E198 第二段产物第 930 行整行（原形，记录还在、没有洞对）：

```
E7RESULT name=image group=sw-crash point=main history=sw-crash.main.S4.ring1048576.a4.d0.threshold0.accumulatefalse image=publishes_after_the_mount=0 step=0 newest=4@31 floor=0 candidates=3@24,3@27,4@30,3@21,3@25,2@16,4@31,3@22,3@26,4@29,3@20,3@23 applied_only_by_records=3@17,3@18,3@19,3@28 hole_pairs=0 pairs=none today_e1=0 today_e2=0 with_pointer_birth_e1=0 with_pointer_birth_e2=0 pointer_unreadable_pairs=0 corrupted_skipped=0 release_generation_verdict=not_run reference_interval_verdict=not_run
```

**三个候选各走到哪（推的）**

- 今天（丙）：最新根 4@31 的实例表行 (2, 16)、R 的实例 3 > 2、中间没有实例 ⇒ 接得上（`crates/singlefs-checker/src/walk.rs:865` 那个函数的三条件）。
- 甲：再加 3@20 的指针出生 txg = 20 = R.txg（上表第 3 步）⇒ 接得上。
- 乙：洞里 17、18、19 的落点槽按本地落点公式是 (2, 1)、(0, 2)、(1, 2)，此刻住着 4@29、4@30、4@31（29 → (2, 1)，30 → (0, 2)，31 → (1, 2)），都是自证过的根，没有一槽「读得出不是全零而自证不过、或读不出」⇒ 接得上。乙计全零同样接得上。
- 真值：3@20 的父版本链 3@19 → 3@18 → 3@17 → 2@16，三版根槽都落过 ⇒ 中间有存在过的版本 ⇒ 三个候选各记一个 E1。
- k ≥ 1 之后：4@32 写进 (2, 2) 盖掉 3@20，R 换成 3@21（指针出生 20 ≠ 21），甲、乙回到分对，今天的臂继续认错，直到 2@16 那一槽在 txg 40 被盖、这一对消失。预言（推的）：每段记录被盖掉了的历史，k = 0 那一张三个候选同错一对；今天的臂另在之后几张上各错一对。模型汇总行的 `all_three_candidates_e1` 数这一格，跑前预言它 ≥ 1。
- 什么会推翻它：`sw-crash-wrapped` 组汇总行 `hole_pairs=0`（记录在取样的 c 与写的次数下没被盖掉；那时这一形在这个取样域里没造出，不算三个候选分对），或者有洞对但 `all_three_candidates_e1=0`。

**这一中会不会让 I-3.5 红（推的）**：不会。3@20 相对 2@16 新出现的落点都生在 17 … 20（> 16），(a) 照样成立；和 E198 的 nrb 一样是「认错而不红」。要红得在洞里夹一次回退，而指针出生等于自己 txg 的 R 只有写行发布与清扫发布（`crates/singlefs-core/src/mount.rs:5618`：「instance_table: InstanceTablePlan::Rewrite(instance_table),」与清扫那一处），两者都在挂载里、回退之前，这一形拼不出「R 在回退之后」。所以这一形的代价是「这一对多判了一截」，不是误红。

**判别子观测得到吗（攻方自提的丁，推的，被攻过零轮）**：三个候选读的三样（行、R 的指针出生、洞里落点槽读没读坏）在这一张上与一对真取号跳过同形，按这三样确实分不开。但镜像上还有别的字节：R 那条根自己的分配记录。每一次发布（空发布也算，D16（发布语义） 已定项 9）都重写固定点单元，上一版的那一份当次释放、释放代就是这一次的 txg（`crates/singlefs-core/src/allocator.rs:72`：「/// 仍分配时是分配代；已释放时是释放代。」）。3@17、3@18、3@19 各释放一次，释放代 17、18、19 都落在 (16, 20) 里；3@20 发布时最旧有效根约是 txg 9（环里 9 … 20），这几条还没到回收的时候，留在 3@20 的账里。真取号跳过 (R⁻, R) 中间没有 R 的祖先，R 的账接在 R⁻ 上，R 自己释放的那几条释放代等于 R.txg（不在开区间里）。所以「R 的账里有代落在 (R⁻.txg, R.txg) 的记录 ⇒ 不是直接接上」在 sw-read、这一形、nrb、rs1 上都判对，在 skr 上不丢（3@8 的账里不该有代 7：2@7 没被施加）。模型的 `accounting_generation_guarded` 臂与 `gap_generations` 一列逐对打出这几个代，跑前预言：sw-read-control、sw-crash-wrapped、nrb-recheck、rs1-recheck 四组 `accounting_generation_guarded_e1` 为 0，skr-gap-fault 的 `accounting_generation_guarded_e2` 为 0。会推翻它的：任一组这两个数不为 0，或 skr 那几对 `gap_generations` 里有不带隔离记号的代 7（例如挂载的预隔离把 2@7 点名的落点写进了账）。

**打中之后的四问**

1. 分不分辨臂：不分辨三个候选——三个一起错，属「打中不分辨臂」，病根是三者共用的输入（行、指针出生、读坏槽都不记录 R 的父版本）；分辨丁（推的）。
2. 被判的系统当时看不看得到判别它的东西：按三个候选读的三样看不到；按 R 自己的分配记录看得到（推的，待模型那一列坐实）。所以「判别子观测不到」只对三个候选的输入成立，不对镜像成立。
3. 满足判据字面的哪一个分句：E1「认作取号跳过而真值不是直接接上」。
4. 改法在打中的格上还中不中：甲中、乙中、丙中，三个改法都碰不到这一格；对这一格起作用的只有不在候选里的丁。

## Q2.3 丙的代价：C698 那两组误红在今天的判法上走到哪、红在哪

**结论**：在 E198 造的 291 张上，今天的错认（nrb 6 对、rs1 285 对认作接得上）走进 I-3.5 的逐落点判，但一张都没红（量的）；那两组真正红的是 I-3.9 的 150 张，与洞怎么分类无关，甲、乙、丙一样红（读码坐实）。所以丙在这 291 张上的代价是 0 张额外的红。丙的代价在别处：同一类合法镜像（回退加读坏实例 2 的根槽）只要用户写的是第二个文件，今天的错认就让 I-3.5 误红（模型上量的，待主 agent 跑）；按用户 2026-10-04 的定案「checker 不许因读不出的根槽误红」（附录 C698 那一行），丙接受不了，除非把这一形登成已知误红。

**291 张上走到哪（量的 + 读码坐实）**

- 洞的分类只有一个调用处，在 I-3.5 的逐对判里，`crates/singlefs-checker/src/walk.rs:970`：「let hole = hole_in_the_candidate_set_between(」；分类只决定用户可见树子树里的落点判不判（`crates/singlefs-checker/src/walk.rs:999` 那一行，接不上就 `continue`），判的式子是 (a) 或 (b)，`crates/singlefs-checker/src/walk.rs:1013`：「let born_after_the_previous_version = birth_txg > previous_txg;」。GPU 判器在 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:4600`：「// walk.rs hole_in_the_candidate_set_between」 抄了同一份。I-3.1 今天不接这份分类，`crates/singlefs-checker/src/walk.rs:850`：「/// 回退候选集里相邻两版之间有没有洞（I-3.5 的缺口条款；I-3.1 在短环上洞里的误红归下一批调查，那一处要接同一份判法，」——C698 那一行写的「只有 I-3.1 按分类判时才红」指的是这一处接上之后。
- 认错的那几对照常判，判出来全成立。E198 第一段产物第 1422、1424 行整行：

```
E7RESULT name=verdict question=Q19 group=nrb point=main reference_interval_verdicts=holds:6 judgement=attached_not_judged
E7RESULT name=verdict question=Q19 group=rs1 point=main reference_interval_verdicts=holds:285 judgement=attached_not_judged
```

  为什么不红（推的）：真值是中间有存在过的版本，R⁻ 在 R 的祖先链上，R 新引用的落点多数生在 R⁻ 之后，(a) 成立；rs1 的回退把目标那一版的老单元带回来，但 rs1 的动作要么重写唯一那个文件（老数据单元被换掉）、要么不碰 extent 树（整棵继承下来，(b) 成立），两条路都不红。
- I-3.9 的 150 张：E198 第一段产物第 1419 行整行：

```
E7RESULT name=verdict question=Q16 group=rs1 point=main value_images=150 verdicts=holds:135,violated:150 judgement=branch_two
```

  I-3.9 的判法不调 `hole_in_the_candidate_set_between`（全文件只有第 970 行一个调用处），甲、乙、丙在这 150 张上一样红；C698 的 I-3.9 一半不随这一轮的哪个候选关。

**丙真正红的那一形（模型格 `rs-two-files`；预言是推的，跑前写下）**

- 历史：mkfs → 实例 1 挂载 → 第一个文件（inode 1，数据单元 A）→ 建 inode 2 → 写 inode 2 → 覆盖写 inode 1 到 txg 10（每次换一个新的 A）→ 实例 2 产品挂载（写行 r = 11）→ 管理员回退到 (实例 1, t)，t ∈ 4 … 9（回退那一版 v）→ 覆盖写 inode 2（`publish_overwrite_of_an_inode`，第一个文件的数据单元照抄：`crates/singlefs-core/src/transaction.rs:9994-9995`：「第一个文件与别的文件的数据单元、…下段照抄」）→ 读坏实例 2 在 [r, R) 里每一版的根槽（C698 那一类，与 E198 rs1 同一刀）。
- R = v + 1 那一张走到的判定：候选集 …、1@10、（洞）、2@R；最新根的行 (1, 10) ⇒ 今天认作接得上；R 引用 t 那一版的数据单元 A_t（诞生代号 ≤ t < 10），1@10 不引用它 ⇒ (a) 不成立；写 inode 2 让 extent 树的根换成新的（出生 R.txg > 10）⇒ (b) 不成立 ⇒ I-3.5 报违例，说明由 `walk.rs` 第 1021 行那个格式串拼出，形如：相邻两版 (实例 {1}、txg 10) → (实例 2、txg R)（HoleBridgedByTheInstanceTableRows）：后一版引用、前一版不引用的落点 …（这是按格式串推的样子，不是引文）。甲：R 的实例表指针出生 txg 是写行那次的 11 ≠ R.txg ⇒ 接不上 ⇒ A_t 在 extent 树里、不判 ⇒ 不红。乙同甲。
- 预言的数：动作串 `second`、`empty+second`、`second+second`、`second+first` 四种在「写过 inode 2 之后、没再覆盖写第一个文件」的那几张上报 `reference_interval_verdict=violated`，每个 t 5 张，t = 4 … 9 合计 30 张（前提是实例 1 的 txg 正如上面推的：第一个文件 1、建 inode 2 在 2、写 inode 2 在 3；模型的 `name=first_instance` 行打出实际 txg，t 落在写 inode 2 之前的那几段停下、不算）；`first`、`empty` 两条对照与 R = v 那一张都 `holds`；每段另出一张一槽都不读坏的，候选集连续、没有洞对、I-3.5 `holds`。
- 什么会推翻它：`rs-two-files` 组汇总行 `pool_checker_reference_interval` 里没有 `violated`；或有 `violated` 而说明里写的是 `HoleNotBridgedByTheInstanceTableRows`、或不读坏那一张也红（那就是别的原因，不是错认的代价）。

**打中之后的四问**

1. 分不分辨臂：分辨。只有今天的臂（丙）红，甲、乙在这一形上不红。
2. 被判的系统当时看不看得到判别它的东西：看得到——甲读的 R 的指针出生 txg（11 ≠ R.txg）就在镜像上。
3. 满足判据字面的哪一个分句：C698 的「误红」（合法镜像上 I-3.5 报违例）；对应 E198 口径是今天的臂的 E1 带出了一张红。
4. 改法在打中的格上还中不中：甲不中、乙不中、丙中。

## Q1（辅）三个候选在各组上的分类

E198 量过的格引 E198 两份产物（文件名缩写：一段 = `research/results/e198-instance-table-pointer-birth-skip-premise-segment-one-2026-11-02.out`，二段 = `research/results/e198-instance-table-pointer-birth-skip-premise-segment-two.out`）；新格是推的、模型待跑。

| 组（洞对数） | 今天（丙） | 甲 | 乙 | 丁（自提） |
|---|---|---|---|---|
| nrb（6） | 认错 6（量的，一段第 1415 行） | 分对（量的，一段第 1417 行） | 分对（推的：洞里 txg 的落点槽正是被读坏的那几槽；模型 `nrb-recheck`） | 分对（推的） |
| rs1（285） | 认错 285（量的，一段第 1416 行） | 分对（量的，一段第 1418 行） | 分对（推的，同上；`rs1-recheck`） | 分对（推的） |
| sw-read（315） | 认错 315（量的，二段第 2072 行） | 认错 315（量的，二段第 2064 行） | 分对（推的：17、18、19 三槽被读坏；`sw-read-control`） | 分对（推的） |
| sw-crash（0） | 没有对象（量的，二段第 2060 行） | 同左 | 同左 | 同左 |
| ss（153） | 分对（量的，二段第 2066 行） | 分对（量的，二段第 2058 行） | 分对（乙比甲只少认、不多认，真值里没有直接接上的对：按定义推的） | 分对（同理） |
| skr（26） | 分对（量的，二段第 2068、2069 行） | 分对（量的，二段第 2059 行） | 原样分对；洞里落点槽读坏时 E2（Q2.1） | 分对（推的） |
| 盖记录的清扫（新） | 认错（Q2.2） | 认错 | 认错 | 分对（推的） |
| 两个文件的回退（新） | 认错且 I-3.5 误红（Q2.3） | 分对 | 分对 | 分对（推的） |

各候选的代价一句（推的）：甲改 `crates/singlefs-checker/src/walk.rs:865` 那个函数加一个参数（R 的实例表指针出生 txg），GPU 判器 `crash_judge_checker.wgsl` 第 4600 行那一段跟着改；乙在甲之上还要把根环每槽的读数（读坏 / 全零 / 自证根）带进 I-3.5，GPU 一侧要多一张槽状态表；丙不改代码，C698 的 I-3.5 一半要另立账、写明两个文件那一形会误红；丁要把 R 的分配记录按代筛一遍，`walk.rs` 走读已经读每条候选根的账（I-3.21、I-3.18 用），GPU 一侧要按版本取账的代——三者都没量。

## 改法各修哪一格

「推的，待主 agent 跑」的格由模型那一组的汇总行判；E198 那几格是 E198 量过的（入库装置，不是攻方副本）。

| 改法 | sw-read | 盖记录的清扫 | skr 原样 | skr 洞内槽读坏 | nrb / rs1 | 两文件回退的 I-3.5 误红 |
|---|---|---|---|---|---|---|
| 丙（今天） | 认错（E198 量的） | 认错（推的，待主 agent 跑） | 对（E198 量的） | 对（推的，待主 agent 跑） | 认错、不红（E198 量的） | 红（推的，待主 agent 跑） |
| 甲 | 认错（E198 量的） | 认错（推的，待主 agent 跑） | 对（E198 量的） | 对（推的，待主 agent 跑） | 对（E198 量的） | 不红（推的，待主 agent 跑） |
| 乙 | 对（推的，待主 agent 跑） | 认错（推的，待主 agent 跑） | 对（推的，待主 agent 跑） | E2（推的，待主 agent 跑） | 对（推的，待主 agent 跑） | 不红（推的） |
| 乙计全零 | 对（推的，待主 agent 跑） | 认错（推的，待主 agent 跑） | E2（推的，待主 agent 跑） | E2（推的，待主 agent 跑） | 对（推的，待主 agent 跑） | 不红（推的） |
| 丁（攻方自提，只在我的模型上，被攻过零轮） | 对（推的，待主 agent 跑） | 对（推的，待主 agent 跑） | 对（推的，待主 agent 跑） | 对（推的，待主 agent 跑） | 对（推的，待主 agent 跑） | 不红（推的：R 的账里有写行那次的代 11） |

## 没打中的形状

- 写者自己造出真取号跳过洞里的读坏槽：试的是崩溃点重放的两态（落 / 没落）在写行、暖机、清扫各一刀；读码看到根槽写一个扇区、不撕（`crates/singlefs-checker-tier/src/crash.rs:2087`），造不出。物理块大于扇区的撕裂一形在模型之外（Q2.1 末段）。
- 长环那一形（记录还在、实例 2 前几版的根槽已被环盖）：只按读码推，没造。记录读得出就按 `walk.rs` 的 ①–④ 补回候选集；④ 不成立时那一版之前的根都已轮出环，R⁻ 跟着没了、洞对不成立。推的，没进模型，一次抽样，不算「没打中」的证据。
- 洞里落点槽全零而中间版本存在过、三个候选都认错：要求中间那几版只靠记录存在、记录后来没了、而 R 又是写行或清扫发布；写行与清扫都在挂载里、前面那几版根槽都落了，拼不出。推的。
- 丁的反例：没攻自己（预隔离会不会把没被施加的记录点名的落点写进账、被抛弃时间线的账会不会混进 R 的账，模型只打出 `quarantined` 一列，没有专门造这两形）。

## 这条腿自己的限度

- 模型没编、没跑；上面所有「模型上量的」「预言」都是推的。编不过、或跑出来与预言不同，按「什么会推翻它」那几句改结论。
- 崩溃只用了装置的单一崩溃点（那一步写流截在第一个根槽写之前），没有调 `enumerate_crash_states` / `enumerate_crash_states_versions` 逐个展开崩溃状态，与定义第 3b 步不合：这两个入口经判定库录流、按登记的项名跑，我的分类要在每个状态上另做一次可写挂载，没接上。三处打中都只需要一个合法状态存在；没有一条「没打中」靠这个模型。交主 agent 判要不要补。
- 没有冻结副本：`--frozen` 写的是仓根，`rerun.sh` 第一屏打出被拷的 `walk.rs`、`mount.rs`、`instance_table_sweep.rs` 等的 sha256。开工时我核过三份被判文件与快照 `research/prompts/m3-close-b19-c699-r1-snapshot/crates-sha256.txt` 一致（`walk.rs`、`instance_table_sweep.rs`、`mount.rs`）；E198 装置源码不在快照清单里。
- 用户决定的动作只放开了一部分：盖记录那一组只试「清扫之后每次都写 c ∈ {3, 4} 个 512 KiB 块」，两文件那一组 6 条动作串；没扫「只有几次写得大」「写大文件之外的别的方式转环」。
- 落点槽用装置的本地落点公式（E198 停机条款 S4 拿它对过 `target_for_publish`），模型没再核。
- 乙、丁都只改「接不接得上」，没有实现进 `walk.rs`；丁的账用 core 的 `allocation_records_under_root` 读，不是 checker 自己那份读法。

## 要主 agent 跑的命令

1. 模型整份复跑（编 + 跑；不碰 checker 档包，不是重型；新 shell 里起）：
   - 命令：`task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m3-close-b19-c699-r1-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs --work /home/fy5090/.local/share/singlefs/work/m3-close-b19-c699-r1-opus/rerun-1`
   - 在哪起：仓根 `/home/fy5090/code/singlefs`（`--work` 那个目录此刻不在，`rerun.sh` 自己建；它在本腿认领目录的 `.deliverables` 里登成「交接」）。有冻结副本的，`--frozen` 换成它。
   - 跑完看：`task wait` 末行 `TASK_SERVER_RESULT` 的 `exit=0`；日志里 `name=build exit=0`（不为 0 时日志里列着 `build.log` 前 40 行 `error`，整份在 `--work` 下的 `build.log`）；`OPUSMODEL name=estimate … within_the_limit=true`（`false` 时二进制退 3、打 `mode=stopped_over_the_estimate_limit`，不跑全量，按定义第 3b 步先缩取样）；七行 `OPUSMODEL name=group group=…` 与末行 `OPUSMODEL name=done mode=full histories=261`。逐对的行在 `--work` 下的 `model.out`（`name=pair`），各段的挂钟在 `model.err`。退非 0 的件数为 0 算过。
   - 跑完新派同立场的承接腿（带「承接：research/prompts/m3-close-b19-c699-r1-opus-output.md」与任务号），读 `task-server/task log <号> --all` 与 `model.out`，逐格对本报告的预言：Q2.1 看 `skr-gap-fault` 组的 `read_bad_gap_slot_guarded_e2`（预言 70）与 `read_bad_or_zero_gap_slot_guarded_e2`（预言 140）；Q2.2 看 `sw-crash-wrapped` 组的 `hole_pairs` 与 `all_three_candidates_e1`（预言 ≥ 1）；Q2.3 看 `rs-two-files` 组的 `pool_checker_reference_interval`（预言 `violated` 30 张）与对应 `name=image` 行里的说明；锚：`sw-read-control` 的 `today_e1` 与 `with_pointer_birth_e1` 各 315、`sw-crash-control` 的 `hole_pairs=0`、`nrb-recheck` 与 `rs1-recheck` 的 `today_e1` 6 与 285、`with_pointer_birth_e1` 0（与 E198 两份产物同数才算模型接上了装置）；把 `model.out` 拷进模型目录 `out/model.out`。

## 没做什么

- 没往统一任务服务交任何任务；模型没编、没跑，判决要引的数一个都没复跑（命令在上一节）。
- 不判正推、辩方那几格（Q1 只作辅、Q3 没碰），不替主 agent 采纳；丁是我自己提的，只在我的模型上，被攻过零轮。
- 崩溃状态没有逐个展开（限度一节第 2 条）。
- 草稿目录里我建过一份语法核用的拼接源码目录 `syntax/`（920K，只有三份拼好的 `.rs` 与 rustfmt 输出，不是仓副本、没编过），交回前已删。草稿目录里留着 `cite1.txt`（cite-check 中途的输出）与 `.deliverables`。

## 附录：引到的 kb 条款整行

`.claude/kb/checks-owed.md` 第 421 行（C698）整行：

```
| C698 | 读坏一个根槽的镜像上的误红 | 读坏一个根槽的镜像算池级 checker 的合法镜像（用户 2026-10-04 定：checker 不许因读不出的根槽误红）。现行判法在这类镜像上：I-3.5（引用区间的精确性）「取号跳过」那一判只认实例表行，最新实例写行那条根读坏时把它存在过的那一版认成取号跳过（攻方副本上 nrb 6 / 6、rs1 285 / 285；只有 I-3.1（已分配统计对得上）按分类判时才红，I-3.5（引用区间的精确性）在攻方 21072 行判定里红 0 行，一次抽样）；I-3.9（释放代落在停止引用它的那一格区间里）在回退加根槽读坏的 rs1 285 张镜像里红 150 张（回退复活的单元，回退那一版读不出）。这些都按误红记；E198（取号跳过认出生代号的前提） 第一段已量（`research/results/e198-instance-table-pointer-birth-skip-premise-segment-one-2026-11-02.out`，问题单 `research/prompts/m3-close-b19-c699-forks.md` 第 2 行）：I-3.5 一半，nrb 6 张、rs1 285 张按 C699（取号跳过改认出生代号） 的第四条件分类全对（第 1417、1418 行 `judgement=classified_right`），今天的臂在这两组上各认错 6、285（第 1415、1416 行），判甲；I-3.9 一半，改后 rs1 上仍有误红（第 1419 行 `value_images=150`），判乙 | 下一轮修：这类镜像上取号跳过不把存在过的版本认成跳过（改法见 C699（取号跳过改认出生代号）），I-3.9（释放代落在停止引用它的那一格区间里）不红；判别力自证：攻方 nrb、rs1 两组镜像现行判法认错 / 红、改完 0 | 用户 2026-10-04 定算合法镜像，下一轮实现；验收要等 C699（取号跳过改认出生代号） 的实现；去向：里程碑三二·5（I-3.5（引用区间的精确性）、I-3.9（释放代落在停止引用它的那一格区间里） 读坏一个根槽的镜像误红，随 C699（取号跳过改认出生代号））（五·11，2026-10-12 判）。E198（取号跳过认出生代号的前提） 判完之后：I-3.5 一半随 C699（取号跳过改认出生代号） 往后怎么走（下一轮推论轮定）；I-3.9 一半仍开着（问题单第 2 行判乙）。 | `research/prompts/m3-candidate-hole-r1-main-verification.md` 第二节「「取号跳过」认什么证据」表第 1 行（只认实例表行）、第五节第 2 行与第六节「读坏一个根槽的镜像算不算合法镜像」行；攻方报告 `m3-candidate-hole-r1-opus-output.md`「四、旁及」那一节（rs1 上 150 / 285）；`research/prompts/m3-birth-floor-r2-main-verification.md` 第二节第 3 行「不算合法镜像上的红」那一判被这一定案替换（冻结证据不回改）；`research/prompts/m3-close-b19-c699-forks.md` 第 2 行；调度记录 `records/2026-10-18-里程碑三收尾与近期欠账.md` 第 2340 行 |
```

`.claude/kb/decisions/16-发布语义.md` 第 38 行（「抬 F 的上限」那一行）整行：

```
| 抬 F 的上限 | **准入抬 F**：min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根)；非空有效根不足 4 个时取最旧的有效根；「有效根」「非空」在环里全部的根上取；挂载那一刻读不出或自证不过、之后这个进程没写过的槽，当作可能是一条非空有效根，F 不越过它（用户 2026-10-04 定，零轮：先定只并读不出的槽，代码轮第三轮之后重定为读不出或自证不过两类都并；与「环里最旧有效根」那一行把读不出的槽并进回收门槛同一个思路）。读坏槽与读回全零的槽都并进上限：挂载那一刻读回全零、之后这个进程没写过的槽，同读坏的槽一样当作可能是一条非空有效根，F 不越过它，能住的最大 txg 的取法同读坏的槽（「环里最旧有效根」那一行「挂载时读不出的槽」那一句；用户 2026-11-02 定「并」，原话在变更史）；上限卡在这两类槽上（并进它们之后上限不高于现行 F，不并它们时上限高于现行 F）时，不在这次挂载里报 ENOSPC，推一次不抬 F 的空发布（带现行 F、txg 加一，算进这次准入的已推次数，受「准入」那一行的 B 限）再判；预算放不下这一次，或这一次取不到落点，就拒这次请求，下一次请求从零预算接着推，推到这个进程写掉那一槽，上限不再被它卡住（用户 2026-11-02 定「推空」，原话在变更史；F 不越过读坏槽或读回全零的槽里的非空根）。一次处置的目标 = min(这次释放的释放代, 第 4 新的非空根)。**卸载抬 F**：抬到现行那一版的 txg，不另判上限（等价于上限取最新的持久有效根，不按盘取小） |
```

## cite-check

`python3 scripts/agents/cite-check.py research/prompts/m3-close-b19-c699-r1-opus-output.md` 退 0，末行：

```
  ✓ 核了 15 处引文、7 处裸行号，对上 15 处，没判 7 处（其中裸行号没有引文可比 7 处；逐处列在上面）
```
