# m3-close-b19-c699-r1 正推腿报告（Q1、Q3）

立场：正推。方案按 `crates/` 今天的实现谈（开工快照 `research/prompts/m3-close-b19-c699-r1-snapshot/` 里 `walk.rs`、`mount.rs`、`instance_table_sweep.rs` 的 sha256 与工作区现状逐字相同，已核）。
没往统一任务服务交任何任务；本腿没写模型。标记：「量的」= E198 两份产物里的读数；「读码坐实」= 我读了 `crates/` 的行；「推的」= 没有对应的量，也没有直接的一行代码。

## 0. 先厘清两个撞名

- E198 问题单第 1 行（`research/prompts/m3-close-b19-c699-forks.md:7`）的「乙」是「甲还有第四条件挡不住的形」，不是一个独立的判法。本轮背景材料里的「乙」是「甲加读坏槽护住」。下面写「E198 的乙」与「本轮乙」分开。
- 本轮的「甲」「丙」没有撞名。丙 = 今天的判法（E198 里叫今天的臂、H1-A）。

## 1. 读到的实现（读码坐实）

- 认洞函数只有一个 CPU 调用点：`crates/singlefs-checker/src/walk.rs:865` 定义，`crates/singlefs-checker/src/walk.rs:970` 调用；GPU 镜像是 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:4602` 的 `appearance_hole_between`，调用在 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:4699`。全仓 `grep -rn hole_in_the_candidate_set_between crates` 在 `walk.rs` 之外只命中 wgsl 注释与 E198 装置的一行注释，没有第二个消费者。所以今天「认作取号跳过」这一个分类只决定 I-3.5 的一件事：接不上时用户可见树子树里的落点不判（`crates/singlefs-checker/src/walk.rs:999` 起的 `continue`）。
- 「接不上」与「报部分没判」在 `judge_the_appearance_rule_of_adjacent_candidate_versions`（`crates/singlefs-checker/src/walk.rs:916`）里是两条不同的出口：
  - 接不上：只跳用户可见树子树里的落点（`crates/singlefs-checker/src/walk.rs:999`），树表、记账树、分配记录树、中央映射树、实例表单元仍照 (a) 判，判定类别不变，不进未判清单；
  - 报部分没判：`crates/singlefs-checker/src/walk.rs:964` 与 `crates/singlefs-checker/src/walk.rs:1006` 往 `unjudged_because_something_does_not_read_back` 里 push，前一处 `continue` 掉整一对，`crates/singlefs-checker/src/walk.rs:1029` 汇成 `partially_judged`，判定类别变成「部分没判」。
- 根环每一槽的读法已经现成：`RootRingSlotReading { SelfVerified, AllZero, DoesNotReadBack }`（`crates/singlefs-checker/src/walk.rs:22268`），`DoesNotReadBack` 的文档是「读不到……或不全零而不自证（撕了、位腐、别的池的根）」（`crates/singlefs-checker/src/walk.rs:22273`）；`AllZero` 另算，注释写「全零的槽可以是从没写过的，不算读不出」（`crates/singlefs-checker/src/walk.rs:22266`）。读它的函数 `root_ring_slot_readings`（`crates/singlefs-checker/src/walk.rs:22278`）在 `check_pool_image` 同一个函数体里已经算好（`crates/singlefs-checker/src/walk.rs:21002`），而认洞那次调用在更后面（`crates/singlefs-checker/src/walk.rs:21243`）。txg 到槽的换算也现成：`root_ring_slot_the_txg_names`（`crates/singlefs-checker/src/walk.rs:22306`），写者侧公式 `target_for_publish` 是 `region = txg % 3`、`slot = (txg / 3) % S`（`crates/singlefs-core/src/root_ring.rs:172`），所以一个 txg 恰好对应一个槽，同一槽下一次被写是 txg + 3S。
- GPU 一侧：根环槽样子的私有数组 `root_ring_slot_readings` 已在认洞所在的同一份着色器里（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:307`）。每版一行 48 个字，前 7 字是实例、txg、走读完整位等，之后每棵用户可见树根 5 字、至多 8 棵（`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:1084`、`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:1090`；7 + 8 × 5 = 47，只余 1 个字，算术）。
- 第四条件要的量：R 的实例表指针出生 txg。CPU 走读已经把这条指针解成 `PointerView`（带 `birth_txg`，`crates/singlefs-checker/src/image.rs:1786`），根环里的根在 `crates/singlefs-checker/src/walk.rs:2539`，由记录施加出来的版本在 `crates/singlefs-checker/src/walk.rs:2572` 一带，但 `VersionReferencesForTheAppearanceRule`（`crates/singlefs-checker/src/walk.rs:840`）没有这个字段，所以今天用不上。

## 2. Q1：三个候选在各组上判对、认错、降成不判

### 2.1 先说清三个候选在判洞这一步的包含关系（读码坐实 + 算术）

今天的臂三条件全成立才认接得上（函数 `hole_in_the_candidate_set_between`，`crates/singlefs-checker/src/walk.rs:865`）。甲 = 三条件再加「R 的实例表指针出生 txg 等于 R.txg」，乙 = 甲再加「(R⁻.txg, R.txg) 之间有 txg 的落点槽读坏则不认」。每多一个条件，「接得上」的集合只会变小，所以：

- 甲、乙相对丙只会把「接得上」改成「接不上」，不会反过来；E1（真值不是直接接上、却认作接得上）只会降，E2（真值直接接上、却认成接不上）只会升；
- 乙相对甲同理。三个候选在「真值直接接上」的洞对上，只有 E2 这一种可能的差别。

### 2.2 每组每个候选（洞对的真值都来自 E198 账本，量的）

「判对」= 洞对的分类与真值一致；「认错」= E1 或 E2；「降成不判」= 真值直接接上，却被降成接不上（这是 E2，在 I-3.5 里的后果是少判用户可见树子树里的落点，不是误红）。

| 组 | 洞对真值 | 丙（今天） | 甲 | 乙 |
|---|---|---|---|---|
| nrb 6 张 | 6 对全是「中间有存在过的版本」 | 认错 6（量的） | 判对 6，认错 0（量的） | 判对 6（推的，见 2.3） |
| rs1 主点 285 张 | 285 对全是「中间有存在过的版本」 | 认错 285（量的） | 判对 285，认错 0（量的） | 判对 285（推的） |
| rs1 G4 点 114 张 | 同上 | 认错 114（量的） | 认错 0（量的） | 判对（推的） |
| sw-read 315 张 | 315 对全是「中间有存在过的版本」 | 认错 315（量的） | 认错 315（量的） | 判对 315（推的，见 2.3） |
| sw-crash 432 张、G1 13 张 | 一对洞对都没有 | 没有对象（量的） | 没有对象（量的） | 没有对象（推的：洞对集合与候选无关） |
| ss 主点 432 张 | 153 对「中间有存在过的版本」 | 判对（量的） | 判对（量的） | 判对（推的，见 2.3） |
| skr 主点 45 张 | 26 对「直接接上」 | 判对（量的） | 判对（量的） | 判对、降成不判 0（推的，见 2.3） |
| skr G3 点 45 张 | 44 对「直接接上」 | 判对（量的） | 判对（量的） | 判对（推的） |

产物行（整行抄，带文件名）：

- `research/results/e198-instance-table-pointer-birth-skip-premise-segment-one-2026-11-02.out:1415`：`E7RESULT name=verdict question=Q13 group=nrb point=main scope=all_histories arm=today error=E1 value_pairs=6 value_images=6 hole_pairs=6 judged_images=6 unreadable_images=0 judgement=branch_two`
- `research/results/e198-instance-table-pointer-birth-skip-premise-segment-one-2026-11-02.out:1416`：`E7RESULT name=verdict question=Q13 group=rs1 point=main scope=all_histories arm=today error=E1 value_pairs=285 value_images=285 hole_pairs=285 judged_images=285 unreadable_images=0 judgement=branch_two`
- `research/results/e198-instance-table-pointer-birth-skip-premise-segment-one-2026-11-02.out:1417`：`E7RESULT name=verdict question=Q14 group=nrb point=main scope=all_histories arm=with_pointer_birth error=E1 value_pairs=0 value_images=0 hole_pairs=6 judged_images=6 unreadable_images=0 judgement=classified_right`
- `research/results/e198-instance-table-pointer-birth-skip-premise-segment-one-2026-11-02.out:1418`：`E7RESULT name=verdict question=Q15 group=rs1 point=main scope=all_histories arm=with_pointer_birth error=E1 value_pairs=0 value_images=0 hole_pairs=285 judged_images=285 unreadable_images=0 judgement=classified_right`
- `research/results/e198-instance-table-pointer-birth-skip-premise-segment-two.out:2064`：`E7RESULT name=verdict question=Q10 group=sw-read point=main scope=swept_histories arm=with_pointer_birth error=E1 value_pairs=315 value_images=315 hole_pairs=315 judged_images=315 unreadable_images=0 judgement=branch_two`
- `research/results/e198-instance-table-pointer-birth-skip-premise-segment-two.out:2072`：`E7RESULT name=verdict question=Q19 group=sw-read point=main scope=all_histories_attached_not_judged arm=today error=E1 value_pairs=315 value_images=315 hole_pairs=315 judged_images=315 unreadable_images=0 judgement=branch_two`
- `research/results/e198-instance-table-pointer-birth-skip-premise-segment-two.out:2060`：`E7RESULT name=verdict question=Q6 group=sw-crash point=main scope=swept_histories arm=with_pointer_birth error=E1 value_pairs=0 value_images=0 hole_pairs=0 judged_images=432 unreadable_images=0 judgement=no_object`
- `research/results/e198-instance-table-pointer-birth-skip-premise-segment-two.out:2063`：`E7RESULT name=verdict question=Q9 group=sw-crash point=main scope=swept_histories arm=with_pointer_birth error=E2 value_pairs=0 value_images=0 hole_pairs=0 judged_images=432 unreadable_images=0 judgement=no_object`
- `research/results/e198-instance-table-pointer-birth-skip-premise-segment-two.out:2079`：`E7RESULT name=verdict question=Q18 group=sw-read point=main hole_pairs=315 directly_bridged=0 existed_versions_in_between=315 previous_not_on_the_chain=0 version_missing_from_the_ledger=0 pointer_unreadable_pairs=0 judgement=coverage_only`

ss、skr 两组的甲读数在 `research/results/e198-instance-table-pointer-birth-skip-premise-segment-two.out:2058`（Q4）、`research/results/e198-instance-table-pointer-birth-skip-premise-segment-two.out:2059`（Q5）、`research/results/e198-instance-table-pointer-birth-skip-premise-segment-two.out:2061`（Q7）、`research/results/e198-instance-table-pointer-birth-skip-premise-segment-two.out:2062`（Q8），都是 `classified_right`；今天的臂在 ss、skr 上的读数在 `research/results/e198-instance-table-pointer-birth-skip-premise-segment-two.out:2066` 到 `research/results/e198-instance-table-pointer-birth-skip-premise-segment-two.out:2069`（Q19，E1、E2 都是 0）。
rs1 G4 点的汇总行在 `research/results/e198-instance-table-pointer-birth-skip-premise-segment-one-2026-11-02.out:1397`，skr G3 的汇总行在第二段产物里（`grep -n 'group=skr point=G3'` 可找），这两行的 114、44 我没有逐字段重核，按背景材料转述记。

### 2.3 乙那一列为什么是推的，推的依据是什么

E198 的装置只有「今天的臂」与「甲」两条臂（`research/prompts/e198-preregistration.md:284` F1 一行与产物里的 `arm=today`、`arm=with_pointer_birth`），没有本轮乙的臂，所以乙那一列没有量。我能做的是读装置怎么造这几组镜像，看乙的判别子（洞里有没有 DoesNotReadBack 的落点槽）在每组上会不会触发：

- nrb：读坏的槽是挂载写行那一版起的 `bad_slots` 个（`crates/singlefs-checker-tier/src/bin/e198_instance_table_pointer_birth_skip_premise.rs:2595`），洞右边的 R 是它们之后第一条读得出的根，所以读坏的槽全在 (R⁻.txg, R.txg) 里。读坏的方式是翻一个字节（`crates/singlefs-checker-tier/src/bin/e198_instance_table_pointer_birth_skip_premise.rs:1604` 的 `CORRUPTED_BYTE_IN_THE_ROOT_SLOT`），槽不全零、自证不过，正是 `DoesNotReadBack`（`crates/singlefs-checker/src/walk.rs:22273`）。乙触发，判对。
- rs1：装置文档写「读坏实例 2 在 [r, R) 里的每一版的根槽（R 成了洞右边第一条读得出的根）」（`crates/singlefs-checker-tier/src/bin/e198_instance_table_pointer_birth_skip_premise.rs:2613`），同上，乙触发，判对。
- sw-read：读坏的是清扫那个实例在清扫那一版之前的每一版（`crates/singlefs-checker-tier/src/bin/e198_instance_table_pointer_birth_skip_premise.rs:2747` 的 `key.instance == sweep_summary.instance && key.txg < first_version_on_the_image`），洞是 (最后一版实例 2, 清扫版)，读坏的槽全在洞里。这些槽此后会不会被环转过去盖掉：槽 t 要等 t + 3S 才再被写（上面第 1 节），历史只写到 `last_before_the_crash = last_txg_before_the_sweep_instance + ring_slots - 1`（`crates/singlefs-checker-tier/src/bin/e198_instance_table_pointer_birth_skip_premise.rs:2738`），读坏的版本的 txg 都大于 `last_txg_before_the_sweep_instance`，所以加上一整圈 ring_slots 之后才会被盖，历史没写到那里。产物也印证：315 张 sw-read 镜像的 `corrupted_skipped` 全是 0（`grep '^E7RESULT name=image group=sw-read ' ...segment-two.out | grep -o 'corrupted_skipped=[0-9]*' | sort | uniq -c` 输出 `315 corrupted_skipped=0`，量的，意思是装置要读坏的每一槽在翻字节那一刻都还住着那一版）。所以 315 张里洞里的读坏槽都还在，乙触发，判对。这是读码与上面那条命令的输出推出来的，乙的臂没有量过，写「推的」。
- ss、skr：这两组的镜像不翻任何根槽（它们用 `sink.judge_now`，不是 `judge_with_corrupted_root_slots`，`crates/singlefs-checker-tier/src/bin/e198_instance_table_pointer_birth_skip_premise.rs:2569`），且 skr 的崩是「施加第一个根槽写之前的那一截」（`crates/singlefs-checker-tier/src/bin/e198_instance_table_pointer_birth_skip_premise.rs:1154` 起的 `land_up_to_the_first_root_slot_write`，`self.image.apply(&operations[..cut])`），没有写一半的槽。洞里的槽要么全零、要么坐着环转之前更早一圈的自证过的根，都不是 `DoesNotReadBack`，乙不触发，分类与甲相同（ss 判对，skr 判对、降成不判 0）。
（补一条行号：ss 的镜像也是 `sink.judge_now`，在 `crates/singlefs-checker-tier/src/bin/e198_instance_table_pointer_birth_skip_premise.rs:2542`。）

### 2.4 问题单第 1 行点名的、E198 没造出来的两形：三个候选都是「没量」，下面是读码推的

- 清扫那一形（不读故障）：sw-crash 432 张、G1 13 张一对洞对都没有（上面 `research/results/e198-instance-table-pointer-birth-skip-premise-segment-two.out:2060`、`research/results/e198-instance-table-pointer-birth-skip-premise-segment-two.out:2063`）。三个候选在这里无从分：没有洞对。这里没有「判对」可报，只能报「没有对象」，不能写成分对。
- 「记录也被盖掉」的清扫那一形：E198 没造出来（`.claude/kb/experiments/198-取号跳过认出生代号的前提.md:306` 起的「它答不了的」一节）。我从 sw-crash 的 k = 0 那一张读出了这一形的骨架，产物第 930 行整行：`E7RESULT name=image group=sw-crash point=main history=sw-crash.main.S4.ring1048576.a4.d0.threshold0.accumulatefalse image=publishes_after_the_mount=0 step=0 newest=4@31 floor=0 candidates=3@24,3@27,4@30,3@21,3@25,2@16,4@31,3@22,3@26,4@29,3@20,3@23 applied_only_by_records=3@17,3@18,3@19,3@28 hole_pairs=0 pairs=none today_e1=0 today_e2=0 with_pointer_birth_e1=0 with_pointer_birth_e2=0 pointer_unreadable_pairs=0 corrupted_skipped=0 release_generation_verdict=not_run reference_interval_verdict=not_run`（`research/results/e198-instance-table-pointer-birth-skip-premise-segment-two.out:930`）。读法：候选里 2@16 还在（崩掉的 3@28 没落槽，它的槽还住着 2@16），实例 3 在 17、18、19 的槽已被 4@29、4@30、4@31 盖掉；17、18、19 之所以没成洞，只因为它们的记录还读得出，被施加成了「由记录施加出来」的版本。若 journal 环短到这几条记录也被盖掉（装置取样域里没有这一点），这一张就会变成洞对 (2@16, 3@20)，真值「中间有存在过的版本」，而 17、18、19 的槽里住着 4@29 到 4@31，自证过的根，不是 `DoesNotReadBack`。此时：丙认错（行 (2, 16) 在，三条件成立）；甲认错（清扫版 3@20 的指针出生 txg 等于 20，与 sw-read 同形）；乙也认错（洞里没有读坏的槽）。这一条是推的，没有量；它说明三个候选在这一形上一起认错，原因是证据（那几版的槽与记录）都被合法地覆盖了。
- 长环那一形（实例 2 前几版已被环盖、记录还在）：G2（ss 的 4 MiB 环，117 张）一对洞对都没有（`.claude/kb/experiments/198-取号跳过认出生代号的前提.md:242`；产物汇总行 `research/results/e198-instance-table-pointer-birth-skip-premise-segment-two.out:2000`，`produced_images=117 ... hole_pairs=0`），三个候选在这一形上没有数。推的：它与上一条同类，三个候选分不分得开取决于覆盖掉的证据在不在，没有量。

### 2.5 每个候选的一句代价

- 甲：`walk.rs` 三处：给 `VersionReferencesForTheAppearanceRule`（`crates/singlefs-checker/src/walk.rs:840`）加一个字段（R 的实例表指针出生 txg 是否等于 R.txg，根环里的根在 `crates/singlefs-checker/src/walk.rs:2539` 取、由记录施加出来的版本在 `crates/singlefs-checker/src/walk.rs:2572` 一带取），在 `crates/singlefs-checker/src/walk.rs:2045` 的 `versions_folded_into_units_by_the_final_fold_table` 里带出，`hole_in_the_candidate_set_between`（`crates/singlefs-checker/src/walk.rs:865`）多一个参数、`bridged_from` 里多一个条件，调用点 `crates/singlefs-checker/src/walk.rs:970` 改一处；GPU 要跟：`appearance_hole_between`（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:4602`）多一个条件，版本表行里多存一位（行 48 字只余 1 字，够，常量不用动，算术），走读内核在 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:4502` 解实例表指针处填；不要新量（E198 已在 nrb、rs1、ss、skr 上量过甲，sw-read 上量过它认错 315）；变异行与坏镜像要加。
- 乙：甲的全部改动，再加：CPU 把 `root_ring_slot_readings`（`crates/singlefs-checker/src/walk.rs:21002` 已算好）与几何传给 `judge_the_appearance_rule_of_adjacent_candidate_versions`，在 `hole_in_the_candidate_set_between` 的 `bridged_from` 之外加「(previous_txg, version_txg) 里任一 txg 经 `root_ring_slot_the_txg_names`（`crates/singlefs-checker/src/walk.rs:22306`）换成的槽是 `DoesNotReadBack`」（txg 来自盘上 8 字节，要把迭代数封在 3S 以内，因为槽随 txg 模 3S 转，超过一圈的洞等于所有槽都算）；GPU 要跟，同样用已有的 `root_ring_slot_readings`（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:307`），多一个循环，不用动内存布局以外的东西；要新量：E198 没有乙的臂，至少要给装置加一条臂，并补两个对乙有判别力的组（见 2.7）。
- 丙：checker 与 GPU 判器都不改，不要新量；C698 那两组的 E1 6、285 与 sw-read 的 315 都留着，I-3.5 在这些镜像上今天照样报成立（`research/results/e198-instance-table-pointer-birth-skip-premise-segment-one-2026-11-02.out:1422`：`E7RESULT name=verdict question=Q19 group=nrb point=main reference_interval_verdicts=holds:6 judgement=attached_not_judged`；`research/results/e198-instance-table-pointer-birth-skip-premise-segment-one-2026-11-02.out:1424`：`E7RESULT name=verdict question=Q19 group=rs1 point=main reference_interval_verdicts=holds:285 judgement=attached_not_judged`，量的）。也就是说 E1 在 I-3.5 上是潜伏的：错分了，没红。I-3.9 那 150 张红（`research/results/e198-instance-table-pointer-birth-skip-premise-segment-one-2026-11-02.out:1419`：`E7RESULT name=verdict question=Q16 group=rs1 point=main value_images=150 verdicts=holds:135,violated:150 judgement=branch_two`，量的）三个候选都不碰。

### 2.6 乙的两种写法：哪一种与 I-3.5 今天的缺口条款同一个口径

结论：降成接不上。依据：

1. 条款本身。`crates/singlefs-checker/invariants/05-空间记账.md:11` 的缺口那一段：不被实例表行接上的洞，处置是「用户可见树子树里的落点不判，别的落点（树表、记账树、分配记录树、中央映射树的单元与实例表单元）仍照 (a) 判 `birth(b) > R⁻.txg`」；同一行的射程句把「根槽读不出」「根槽被环盖掉而记录也被盖掉（短 journal 环）」列为缺口的来源。也就是说读坏槽造成的洞今天就走降成接不上这条路，本轮乙只是让判洞这一步看得见这个来源（读码坐实，条款行 grep 过）。
2. 代码两条出口不等价（第 1 节）：降成接不上只丢用户可见树子树里的落点，别的落点照判；报部分没判（`crates/singlefs-checker/src/walk.rs:964`）整对丢掉，判定类别变成部分没判。乙要护的是「中间可能存在过一版、回退可能复活旧单元」，只影响用户可见树里的落点（这正是 N3 的前提），树表、记账树那些落点不受影响，丢掉它们是白丢（读码 + 条款推的）。
3. 判定类别。降成接不上时 nrb、rs1 的 I-3.5 仍报成立（今天就是成立，量的，见上面 `research/results/e198-instance-table-pointer-birth-skip-premise-segment-one-2026-11-02.out:1422`、`research/results/e198-instance-table-pointer-birth-skip-premise-segment-one-2026-11-02.out:1424`；sw-read 上 I-3.5 没跑，汇总行写 `reference_interval_verdict=none`，那一组今天是否成立没有数）；改成报部分没判，nrb、rs1 的 I-3.5 会从成立变成部分没判（推的）。

要留给主 agent 或用户的一处张力：`.claude/kb/checks-owed.md:424`（C701）记着用户 2026-10-20 定「一律报部分没判」，要求「射程里有一部分在镜像上读不出、判不了时报部分没判并逐条列出」。乙的降级由读坏的槽触发，字面上落在这条之内；而今天的缺口条款对同一类来源（根槽读不出）是静默降成接不上、不进未判清单。所以口径上有两层：判哪些落点，同缺口条款（降成接不上）；报不报、怎么列，是 C701 的事，我读不出用户的意思，不替定。若要两样都满足，可以叫「乙加注」：降成接不上，同时 `part_of_the_range_left_unjudged` 列一条说明（转述：哪一对相邻两版、洞里哪个 txg 的根槽读坏、用户可见树子树里的落点没判；入口 `part_of_the_range_left_unjudged` 现成，`crates/singlefs-checker/src/image.rs:695`），代价是判定类别变，推的，没量。

### 2.7 乙要新量什么、什么现象会让 Q1 的各条翻面

乙那一列全是推的。要把它变成量的，需要给 E198 装置（`crates/singlefs-checker-tier/src/bin/e198_instance_table_pointer_birth_skip_premise.rs`）加一条乙的臂，臂只读镜像（走 `recovery` 的读根环），再补下面三组，每组写明翻面观测：

| 组 | 造法 | 乙的预期（推的） | 什么观测会推翻 |
|---|---|---|---|
| skr 加一个无关的读坏槽 | skr 历史，洞里某个 txg 的槽此刻住着更早一圈的根，翻一个字节 | 真值「直接接上」，乙降成接不上：E2 非 0，后果是少判用户可见树子树里的落点，不是误红 | E2 为 0（说明洞里那个槽在装置里没被翻到，或乙读槽的 txg 换算错了） |
| 清扫加记录也被盖掉 | sw-crash 的 k = 0 形加更短的 journal 环，使 17、18、19 的记录被盖 | 三个候选一起认错（2.4） | 乙判对（说明那一形里有我没看到的证据，例如槽里坐着的 4@29 到 4@31 之外的字节） |
| sw-read 的读坏改成读回全零 | 同 sw-read，把翻字节换成整槽清零 | 乙不触发（`AllZero` 按设计不算读坏，`crates/singlefs-checker/src/walk.rs:22266`），回到认错 315 | 乙仍判对 |

第三组是乙的一个已知边：D16 的「抬 F 的上限」一行把读回全零的槽也并进了「可能是一条非空有效根」（`.claude/kb/decisions/16-发布语义.md:38`，用户 2026-11-02 定「并」），而本轮乙的定义只收读坏的槽。若判洞也把全零收进去，没写过的洞里槽全是零，真取号跳过就会一律降成接不上，判别力没了（推的），所以乙只能不收全零；这样「读回全零的故障」是乙护不到的一个口，两件事口径不同的原因是 D16 那一行是在算一个上限（宁大勿小），判洞是在分两种情形（要分得开）。这是读码加条款读出来的，不是量的。

### 2.8 Q1 小结（每条都标）

1. 在 E198 量过的四组上：丙认错 nrb 6、rs1 285、sw-read 315，甲只剩 sw-read 315（量的）；乙在这三组上是 0 错（推的，读装置构造 + `corrupted_skipped=0` 推出）；sw-crash 没有对象，三个候选无差别（量的）。
2. 合法历史 ss、skr 上三个候选都判对，降成不判 0（丙、甲量的，乙推的）。
3. 清扫「记录也被盖掉」与长环两形没量，我从 sw-crash k = 0 的候选列表读出：三个候选在「记录也被盖掉」那一形上一起认错（推的）。
4. 乙的写法选「降成接不上」（读码 + 条款行，坐实）；C701 要不要再加一条未判清单是另一件事，我不替定。
5. 代价：甲 = `walk.rs` 三处加 GPU 一处；乙 = 甲加根环读数一处、GPU 一处，要新量；丙 = 零改，I-3.5 在 C698 两组上今天成立（量的），E1 潜伏，一旦别的不变量（I-3.1 在短环上洞里的误红，`crates/singlefs-checker/invariants/05-空间记账.md:11` 写「共用同一个「候选集有洞」判法」）改用这个分类，E1 就会变成红（推的）。
6. 什么会推翻这些：乙若在 sw-read 上量出 E1 非 0，则说明 315 张里有洞里槽已被盖过，或乙的 txg 到槽换算错；丙若在 nrb、rs1 之外的合法历史上让 I-3.5 红，则「E1 潜伏」不成立。

## 3. Q3：F1 那一判，照「判据自己也会写错」四问

先把被判的东西抄全。`research/prompts/e198-preregistration.md:284`：`| F1 | 第 1 行判乙：甲在合法历史上把取号跳过与存在过的版本分不全 | Q4–Q9 任一 ≥ 1（报组、参数、k、R⁻、R、真值链）；按问题单字面 Q10 ≥ 1 同样触发（第一节待认项 2） |`。问题单第 1 行（`research/prompts/m3-close-b19-c699-forks.md:7`）问的是：甲加第四条件之后，「行 + R 的实例表指针出生代号」把取号跳过与存在过的版本分不分得全，还是「乙 还有第四条件挡不住的形」；翻面观测里写明合法历史「含根环转一圈留洞、可写挂载末尾清扫、读故障」；够判条件写「错分非 0 就判乙并给出那一形」。所以 E198 的「乙」不是一个备选判法，是「甲有一形挡不住，给出那一形」。

### 3.1 四问逐个

1. 它分不分辨臂？不分辨（量的）。Q10 上甲认错 315（`research/results/e198-instance-table-pointer-birth-skip-premise-segment-two.out:2064`），今天的臂也认错 315（`research/results/e198-instance-table-pointer-birth-skip-premise-segment-two.out:2072`），两条臂在这一格一样。分辨得出臂的格在 nrb、rs1：今天 6、285，甲 0（`research/results/e198-instance-table-pointer-birth-skip-premise-segment-one-2026-11-02.out:1415` 到 `research/results/e198-instance-table-pointer-birth-skip-premise-segment-one-2026-11-02.out:1418`）；ss、skr 上两条臂都是 0（`research/results/e198-instance-table-pointer-birth-skip-premise-segment-two.out:2058`、`research/results/e198-instance-table-pointer-birth-skip-premise-segment-two.out:2059`、`research/results/e198-instance-table-pointer-birth-skip-premise-segment-two.out:2061`、`research/results/e198-instance-table-pointer-birth-skip-premise-segment-two.out:2062` 与 `research/results/e198-instance-table-pointer-birth-skip-premise-segment-two.out:2066` 到 `research/results/e198-instance-table-pointer-birth-skip-premise-segment-two.out:2069`）。所以这一格属「打中不分辨臂」，与主 agent 2026-11-02 的判一致。
2. 被判的系统当时看得到吗？分两种形：
   - sw-read 这一形：看得到（推的，没有单独量，读码与产物拼出来的）。判洞的输入只有行、指针出生 txg、实例，E198 实验页写「镜像上没有别的字节把它与真正的取号跳过分开」是就这几样说的；但 checker 做决定那一刻手里还有根环每一槽的读数（`crates/singlefs-checker/src/walk.rs:22278` 的 `root_ring_slot_readings`，`crates/singlefs-checker/src/walk.rs:21002` 先于认洞算好），sw-read 的 315 张里洞里的槽都被翻过字节（2.3），真取号跳过（skr）的洞里没有 `DoesNotReadBack` 的槽。两种情形在这一样东西上不相同。所以对 sw-read 这一形，不能算「判别子观测不到」。
   - 「记录也被盖掉」的清扫那一形：看不到（推的，2.4）。洞里的槽已被后来的根合法覆盖，记录也没了，判洞手里的几样都与真取号跳过逐条相同。这一形才属第二种，但 E198 没造出来，也没有量。
3. 它满足的是判据字面的哪一个分句？F1 里「按问题单字面 Q10 ≥ 1 同样触发」那一句。归属没有错：Q4–Q9 全是 0 或没有对象（`research/results/e198-instance-table-pointer-birth-skip-premise-segment-two.out:2058` 到 `research/results/e198-instance-table-pointer-birth-skip-premise-segment-two.out:2063`，2060、2063 是没有对象），触发只靠 Q10。sw-read 在问题单范围内：第 1 行翻面观测明写合法历史含「读故障」（`research/prompts/m3-close-b19-c699-forks.md:7`），E198 登记修订 1 又照主 agent 定案没把它从第 1 行剔掉（`.claude/kb/experiments/198-取号跳过认出生代号的前提.md:160`）。预注册自己也预见了这一形：`research/prompts/e198-preregistration.md:293` 写「F1 打中时先核那一形是不是 sw-read 那一类（R 正是清扫那一版、它前面同实例的版本读坏或被盖）」。所以 F1 字面成立、没有归错判据。
4. 跑前条款给的改法，在打中的格上还中不中？跑前条款给的改法只有甲一个（`.claude/kb/experiments/198-取号跳过认出生代号的前提.md:181` 写「跑前条款给的改法只有甲一个，没有第二个改法可核」），在 sw-read 的 315 格上甲照样中（量的，`research/results/e198-instance-table-pointer-birth-skip-premise-segment-two.out:2064`）。交出去时要写明：甲对这 315 格不起作用。本轮乙是一个事后提出的第二个改法，它在这 315 格上不中是读码推的，没量过。

### 3.2 这一判该不该改成「不拿它判臂」

该，但只改「F1 拿来做什么」，不改「F1 报的事实」。分开说：

- 事实保留：甲在 sw-read 这一形上分不全（量的，315 / 315）。E198 页写「判乙并给出那一形」没有错，这一格不能擦。
- 臂的取舍不能拿它做：`evidence-discipline.md` 那一行（`.claude/singlefs-ai-sop/rules/evidence-discipline.md:124`）写「不拿它判臂：把那个共用前提另立一笔账先修，判臂只看分辨得出臂的那几格」。判甲对丙，看的格是 Q13 到 Q15（nrb 6 → 0，rs1 285 → 0）与 ss、skr（两边都是 0）；Q10 两边相同，不进比较。照这几格，甲在每一格都不比丙差，在两组上严格更好（量的）。所以 F1 触发不能当作「甲不值得做」或「回到丙」的理由。
- 预注册 F1 那一行没写「全部出局时怎么记」：`research/prompts/e198-preregistration.md:284` 只写 F1 触发就判乙，没有给所有臂一起出局的那一格（读原文，坐实）。这是该补的：「Q10 这类两条臂一起错的格，记为共用前提的账，不进臂的比较」。
- 共用前提是什么：「R 的实例表指针出生 txg 等于 R.txg，就说明 R 是新实例的第一版」。清扫在可写挂载末尾发一次清扫发布（`crates/singlefs-core/src/instance_table_sweep.rs:12` 起的文档注释写「可写挂载末尾……判一次」），删行时新建实例表，指针出生 txg 落在清扫那一版上（背景材料第二节转述；E198 前提 Q1 的 `sweep_pointer_changes` 量的就是这个，产物 `research/results/e198-instance-table-pointer-birth-skip-premise-segment-two.out:2085`：`E7RESULT name=control control=PC-P2 swept_histories=55 sweep_pointer_changes=55 hit=yes`），所以这一句在清扫版上也成立，而清扫版前面同实例的版本可以存在过。这条前提不属于 P1 到 P3 能覆盖的东西，是第四条件自己用来代表「首版」的那个代理。

### 3.3 问题单第 1 行该判什么（建议写法，判决归主 agent）

拆成两句，各标出处：

1. 按问题单字面验收甲：分不全，有一形——清扫版前面同实例的版本根槽读坏、记录也没读到可施加（sw-read，315 / 315，量的）；今天的臂同样错 315（量的），所以这一形不分辨臂。
2. 判臂：只看分辨得出臂的格，判甲（nrb 6 → 0、rs1 285 → 0，ss、skr 两边 0，量的）。sw-read 一形另立账，写进 C699 的欠账一行，账里写：判别子对这一形看得到（根环槽读数），对「证据被合法覆盖」那一形看不到；后者本轮不论。

什么会推翻这一建议：本轮乙在 sw-read 上量出 E1 非 0（说明槽读数在 checker 手里其实分不开这一形，第二问就要改成「看不到」，第 1 行应改判判据）；或者发现 nrb、rs1 之外有一类合法历史里丙比甲好（E2 非 0），则「甲在每一格都不比丙差」不成立。

## 4. 要主 agent 跑的命令

- 没有我能写成 `task-server/task submit` 整条的命令：乙那一列要量，得先给入库装置 `crates/singlefs-checker-tier/src/bin/e198_instance_table_pointer_birth_skip_premise.rs` 加一条乙的臂和 2.7 的三组，这是改入库装置，不是跑现成命令。乙那一列的读数因此全是「推的」，其状态是「待主 agent 跑」（要先有装置）。我没写模型，没有复跑脚本与 sha256。
- 只读核对（文本命令，直接跑，不经统一任务服务），用来复核我的推：
  - `grep '^E7RESULT name=image group=sw-read ' research/results/e198-instance-table-pointer-birth-skip-premise-segment-two.out | grep -o 'corrupted_skipped=[0-9]*' | sort | uniq -c`，我跑的输出是 `315 corrupted_skipped=0`。
  - `grep -rn 'hole_in_the_candidate_set_between' crates`，我跑的输出五行：`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:4600` 注释、`crates/singlefs-checker-tier/src/bin/e198_instance_table_pointer_birth_skip_premise.rs:1691` 注释、`crates/singlefs-checker/src/walk.rs:865` 定义、`crates/singlefs-checker/src/walk.rs:914` 注释、`crates/singlefs-checker/src/walk.rs:970` 唯一调用。
  - `sha256sum crates/singlefs-checker/src/walk.rs crates/singlefs-core/src/mount.rs crates/singlefs-core/src/instance_table_sweep.rs` 与 `research/prompts/m3-close-b19-c699-r1-snapshot/crates-sha256.txt` 三行逐字相同（我核过）。

## 5. 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| Q1 nrb 6 | 丙认错 6、甲判对、乙判对 | 丙与甲量的，乙推的 |
| Q1 rs1 285 | 丙认错 285、甲判对、乙判对 | 同上 |
| Q1 sw-read 315 | 丙认错、甲认错、乙判对 | 丙甲量的，乙推的（读装置构造 + `corrupted_skipped=0`） |
| Q1 sw-crash 432 / G1 13 | 没有对象 | 一对洞对都没有，三者无从分，量的 |
| Q1 ss 153 对 | 三者判对 | 丙甲量的，乙推的 |
| Q1 skr 26 对 | 三者判对，降成不判 0 | 丙甲量的，乙推的 |
| Q1 清扫「记录也被盖掉」与长环 | 没量；读码推的是三者一起认错 | 证据被合法覆盖，槽与记录都不在 |
| Q1 乙两种写法 | 降成接不上同口径 | 缺口条款行与代码出口都支持；C701 要不要再列未判清单不替定 |
| Q1 代价 | 甲：walk.rs 三处加 GPU 一处；乙：再加根环读数一处与 GPU 一个循环，要新量；丙：不改 | 读码坐实；乙的新量是推的 |
| Q3 一问 | 不分辨臂 | Q10 两条臂各 315，量的 |
| Q3 二问 | sw-read 一形看得到；盖掉那一形看不到 | 前者推的（槽读数现成），后者推的 |
| Q3 三问 | F1 字面成立、没有归错 | 预注册 293 行已预见这一形 |
| Q3 四问 | 甲对这 315 格不起作用 | 跑前改法只有甲一个；本轮乙不中是推的 |
| Q3 结论 | F1 事实保留，不拿它判臂；问题单第 1 行拆两句 | 判臂只看 Q13 到 Q15 与 ss、skr，甲不比丙差 |

## 6. 没做什么

- 不判别的腿的格（Q2 的三问）没碰；没替主 agent 采纳或出判决。
- 没往统一任务服务交任务；判决要引的数没复跑，命令在上面「要主 agent 跑的命令」一节：乙那一列需要先改装置，复核不了现成命令。
- 没读禁读清单里的攻方报告与它的草稿目录。
- 没逐行重核 rs1 G4 的 114、skr G3 的 44，按背景材料转述记。
- 没量任何乙的读数，没造任何镜像。
