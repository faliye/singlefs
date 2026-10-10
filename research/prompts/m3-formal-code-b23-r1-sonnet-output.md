# 第 23 批代码轮第一轮 云端正推（Sonnet）报告

<!-- doc-lint:not-numbers K1 K2 K3 F3 H2 A1 B1 -->

轮名 m3-formal-code-b23-r1，格 K1（K3-a、F3-c）、K3（H2、两处注释）。代码引文读冻结副本 `/tmp/claude-1000/m3-formal-code-b23-r1/tree/`（对 `crates-sha256.txt` 逐份核过，`sha256sum -c --quiet` 无输出、退出 0），行号是那棵树里的；kb 行号是仓里现行文件的。
方案按 `crates/` 今天的实现来谈。我的副本在 `/tmp/claude-1000/m3-formal-code-b23-r1-sonnet/ws/`（冻结副本的 crates 加根 `Cargo.toml`、`Cargo.lock`，只加 `tests/b23_*_probe.rs` 四个 harness 档探针），仓内文件一个没碰；探针源、脚本、输出落在 `research/prompts/m3-formal-code-b23-r1-sonnet-model/`。

checker 档（`singlefs-checker-tier`）一律没跑：K3-a 用例本身、F3-c 用例本身、两条变异都是「读码推的」；能在 harness 档复现的那一半（态、判官一层的 (1, 1) 与 (1, 0)、种子算术、H2 两条用例换 S）量了，标「量」。

## 一、K1-a：K3-a 的态是不是攻方那个态

**判定：一致（量 + 读码）。**

判决把 K3 的越屏障靶向态落在 `research/prompts/m3-formal-code-b21-r1-opus-model/out/k3-sweep.txt` 那一行 `first(last=80,withheld=[0, 1, 4]) second(last=19,withheld=[0, 1, 3])`。

读码对法（攻方探针与入库用例逐句对）：

- 第一次崩溃的持久集合：攻方取 candidates 为第一个文件两份记录加它根槽之后两槽轮换，相对第一次崩溃那一段段首的下标，「`let candidates: Vec<usize> = ff_records.iter().chain(&ff_rotation).map(|w| w - first_write).collect();`」（`research/prompts/m3-formal-code-b21-r1-opus-model/probe/opus_probe.rs:504`），写数「`let first_count = last + 1 - first_write;`」（`research/prompts/m3-formal-code-b21-r1-opus-model/probe/opus_probe.rs:507`）；用例取两份记录加第二槽轮换，「`.chain(std::iter::once(&rotation_after_the_first_file[1]))`」（`crates/singlefs-checker-tier/src/crash_injection.rs:6521`），下标「`.map(|write| write - first_write_of_the_first_crash_segment)`」（`crates/singlefs-checker-tier/src/crash_injection.rs:6522`），写数「`root_of_the_overwrite + 1 - first_write_of_the_first_crash_segment;`」（`crates/singlefs-checker-tier/src/crash_injection.rs:6525`）。
- 第二截：攻方「`let second_count = second_last + 1 - second_first;`」（`research/prompts/m3-formal-code-b21-r1-opus-model/probe/opus_probe.rs:558`）；用例取写行两份记录加根槽之后第一槽轮换，「`.chain(std::iter::once(&rotation_after_the_root[0]))`」（`crates/singlefs-checker-tier/src/crash_injection.rs:6599`）。
- 二次崩溃镜像的叠法：注入里「`let mut second_base = crash_image_materialized.clone();`」（`crates/singlefs-checker-tier/src/crash_injection.rs:2984`）从第一次崩溃镜像起、挂载写表段首之前整段落、本段按持久集合；用例里的前提镜像「`let mut second_crash_image = first_crash_image.clone();`」（`crates/singlefs-checker-tier/src/crash_injection.rs:6603`）同一叠法。

量（我自己的 harness 档探针 `research/prompts/m3-formal-code-b23-r1-sonnet-model/probes/b23_k3a_probe.rs`，把用例的摆法逐句抄成 harness 公开入口，输出 `…/out/k3a-run1.log`）：

```
[384槽 1次覆盖写 (用例原样)] writes=83 segments=20 first_ow=58 root_ow=80 root_ff=55 ff_records=[53, 54] ff_rotation=[56, 57]
[384槽 1次覆盖写 (用例原样)] 第一次崩溃那一段 index=13 段长=2 段首写=53
[384槽 1次覆盖写 (用例原样)] 挂载写表 55 写 13 段；写行记录=[16, 17] 根=18 轮换=[19, 20]
[384槽 1次覆盖写 (用例原样)] 第二截段 index=2 段首写=16 withheld=[0, 1, 3] count=4
[384槽 1次覆盖写 (用例原样)] 恢复落点：第一次崩溃 Some((InstanceGeneration(1), CheckpointTxg(4)))，二次崩溃 Some((InstanceGeneration(2), CheckpointTxg(5)))，两者不同=true
```

与攻方产物对：「k3-sweep combos=200 same=162 branch_only=32 counts_differ=6 first_crash_mount_not_published=0 (ff_records=[53, 54] ff_rotation=[56, 57] ow_root=80 writes=83)」（`research/prompts/m3-formal-code-b21-r1-opus-model/out/k3-sweep.txt:7`）：写下标 53、54、56、57、根 80、写表 83 一个不差；第一次崩溃写数 80 + 1 − 53 = 28 即攻方 `last=80`；第二截段首 16、轮换 19、写数 4 即攻方 `last=19`、withheld `[0, 1, 3]`（第一次崩溃那边 57 − 53 = 4 即 `[0, 1, 4]`）。判决点名的那一行就是「COUNTS-DIFFER first(last=80,withheld=[0, 1, 4]) second(last=19,withheld=[0, 1, 3]): correct=(1, 1, [」（`research/prompts/m3-formal-code-b21-r1-opus-model/out/k3-sweep.txt:9`）。

两次崩溃落点不同（前提断言 `assert_ne!`，`crates/singlefs-checker-tier/src/crash_injection.rs:6614`）：(1, 4) 与 (2, 5)，实测成立。

**推翻条件**：用例里的写表下标算出来的态与上面逐个写对不上（比如我这边 `withheld` 不是 `[0, 1, 4]` / `[0, 1, 3]`），或 harness 档探针里两次崩溃落点相同。没出现。

## 二、K1-b：断言在接对时 (1, 1)、接错时 (1, 0)；写下标现算会不会在别的写表长度上摆错态

**判定：断言对；现算在别的写表长度上不会静默摆错态，只会在前提断言上响（量判官一层 + 读码计数）。**

读码：计数从哪两处加。第三截的判法「`tally.remount_refusals_for_a_newer_publish_witnessed_by_system_configuration += 1;`」（`crates/singlefs-checker-tier/src/crash_injection.rs:3339`）在拒因是判据 N-配置、判官判成不合法时加一次（判成合法的在 3332 行加一次），「`tally.remount_refusals_with_the_witness_criterion_true_without_read_faults += 1;`」（`crates/singlefs-checker-tier/src/crash_injection.rs:3352`）只在判官列出的没满足的条件里有 C670 那一条时再加一次。所选那一版的接线在调用点「`version_landed_on_after_the_crash: report.effective_root,`」（`crates/singlefs-checker-tier/src/crash_injection.rs:3054`；1404 行是第二截那一处，不是变异的靶）。判官里 C670 那一条的复算「`RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults(`」（`crates/singlefs-harness/src/mount_transparency.rs:1645`）拿 `selected_version` 去比。

量（同一份探针，判官 `judge_writable_mount_refusal_of_a_crash_state` 在二次崩溃镜像上，`selected_version` 分别取二次崩溃镜像的落点（接对）与第一次崩溃镜像的落点（接错），读故障取 `NoneInjected`）：

```
[384槽 1次覆盖写 (用例原样)] 再挂拒：NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion { first_read: SelectedVersionAgainstTheWitness { selected_version: RollbackTarget { instance: InstanceGeneration(2), checkpoint_txg: CheckpointTxg(5) }, witness: NewerPublishWitness { witnessed_journal_counter: 3, comparison: Undecidable } }, reread: …
[384槽 1次覆盖写 (用例原样)] 接对(二次崩溃镜像落点) selected=Some((InstanceGeneration(2), CheckpointTxg(5))) => Illegal { reason: NewerPublishWitnessedBySystemConfiguration, conditions_not_met: [WitnessCriterionTrueOnACrashImageWithoutReadFaults(NeitherTheLastRecordOfTheSelectedVersionNorTheRecordAtTheWitnessedCounterReadable)] }
[384槽 1次覆盖写 (用例原样)] 接错(第一次崩溃落点) selected=Some((InstanceGeneration(1), CheckpointTxg(4))) => Illegal { reason: NewerPublishWitnessedBySystemConfiguration, conditions_not_met: [CriterionFalseWhenRecomputedOnTheCrashImage] }
```

接对：拒因对、判官列出 C670 那一条 ⇒ 两个计数各加一次 = (1, 1)；接错：拒因同样是判据 N-配置（第一个计数照加）、判官只列「判据复算为假」、没有 C670 那一条 ⇒ (1, 0)。与判决里攻方量的 (1, 1) / (1, 0) 一致。为什么接错会假：c_见证 取 3（第一个文件那一版末条的计数器，它的两份记录在第一次崩溃里被扣下），接错时「所选那一版」是第一次崩溃落到的覆盖写那一版 (1, 4)，它的末条记录读得出、计数器不小于 3，判据假；接对时所选 (2, 5) 的末条与计数器 3 那条记录都读不出，「两条都读不出按真」。

**写下标现算在别的写表长度上**：我把用例的摆法换到别的写表（盘宽 384、256、240 槽与 4 GiB，覆盖写内容空、满一个单元载荷、一次与两次覆盖写，共 7 个配置，同日志 `…/out/k3a-run1.log` 全 71 行）：
- 5 个配置（384 槽一次覆盖写、覆盖空内容、覆盖满一个单元载荷、256 槽、240 槽）写表都是 83 写、下标与用例原样同；384 槽两次覆盖写是 108 写、4 GiB 是 91 写（下标整体平移 4：`ff_records=[57, 58]`、根 88）；7 个配置的 `ff_records`、`ff_rotation`、写行记录与轮换的个数每处都是 (2, 2)；
- 第一次崩溃那一段每个配置都只有 2 个写（段长=2），持久集合靠「越过段尾」摆，摆法与段首在哪无关：持久集合 = 写表里从段首起到覆盖写根槽为止、除去扣下的（段首之前整段持久），所以段首变了态不变；
- 7 个配置上两次崩溃落点都不同，判官一层都是接对带 C670 条件、接错不带。
- 前提断言 `(2, 2)`（`crates/singlefs-checker-tier/src/crash_injection.rs:6513`、6591 行）在写表形状变了（记录或轮换个数不是每盘一份）时会在摆态之前响，不会摆出别的态后绿过去。

**推翻条件**：换一份写表后 `(2, 2)` 仍成立而扣下的写落在别的种类上（比如 `ff_rotation[1]` 落在非轮换的写上）；或接对时判官没有 C670 条件。7 个配置里没出现。

## 三、K1-c：F3-c 抽函数前后种子逐位相同

**判定：逐位相同（读码 + 量抄出来的两份公式）。**

读码：合入前内联的是历史种子、盐、段号、指纹四样异或喂给 `SeededRandomSource::from_seed`（diff 里的 `-` 行）；合入后抽成「`fn seed_of_the_second_crash_points(`」（`crates/singlefs-checker-tier/src/crash_injection.rs:2821`），函数体里同样四项、同一个运算符 `^`、`u64::try_from(段号)` 同一个 `expect`，指纹还是「`fn fingerprint_of_the_persisted_subset(persisted_within_the_segment: &[bool]) -> u64 {`」（`crates/singlefs-checker-tier/src/crash_injection.rs:2806`）；调用点只换成「`SeededRandomSource::from_seed(seed_of_the_second_crash_points(seed, crash_point).0);`」（`crates/singlefs-checker-tier/src/crash_injection.rs:2845`），`.0` 取新类型 `SecondCrashPointSeed(u64)` 里那个 u64。`^` 满足交换结合，四项在函数里的次序与合入前相同，并且指纹函数一个字没动。仓里没有第二处拿这四样异或起随机源（`grep -nF 'SECOND_CRASH_POINT_SEED_SALT' crash_injection.rs` 命中 109 行定义与 2827 行函数体两处）。

量：`research/prompts/m3-formal-code-b23-r1-sonnet-model/f3c_identity.rs` 把合入前内联那段与合入后那个函数逐字抄成两个函数（常量与指纹抄自 109、112、115、2806 行起），对 2000000 个随机输入（种子、段号 0..999、长 0..79 的随机子集）与穷举（长 0..14 的全部子集 × 段号 0..39 × 种子 {0, 7}）各比一次 u64：

```
f3c-identity random=2000000 exhaustive=2621360 mismatches=0
```

（输出 `…/out/f3c_identity.out`；这是 `rustc --edition 2021 -O` 编出的独立小程序，不经内存包装之外的任何入口，没链 checker 档。）

**推翻条件**：合入后那个函数里有一项与合入前的内联不是同一个表达式（如段号换成别的字段），或仓里另有一处照老写法算种子；我读 diff 与 `grep` 都没见到。我抄出来的两份公式若与原文有出入，则这条量不算——抄的是 diff 里 `-`/`+` 行原文。

## 四、K1-d：4095 个子集种子两两不同的断言能不能抓住别的弱指纹

**判定：能抓「折得太少」一类，抓不住「折到第 12 个写之后才缺」「只留低位」「掩码当整数」一类（量算术 + 量真历史的段长）。**

读码：用例「`fn all_first_crash_states_of_a_segment_of_twelve_writes_seed_their_second_crashes_differently`」（`crates/singlefs-checker-tier/src/crash_injection.rs:4405`）固定历史种子 0、段号 16、长 12，对掩码 0..4095 各算一个 `seed_of_the_second_crash_points`，收进 `BTreeSet` 断言个数 4095。种子 = 0 ^ 盐 ^ 16 ^ 指纹，前三项是常数，所以「4095 个种子两两不同」恰等价于「指纹在这 4095 个长 12 的子集上单射」。它只测指纹的单射性，不测段号那一项，也不测长度超过 12 的子集。

量：`research/prompts/m3-formal-code-b23-r1-sonnet-model/f3c_weak_fingerprints.py` 把指纹换成各种弱版本，用与用例同样的输入算一遍（输出 `…/out/f3c_weak_fingerprints.out`）。列：len12 = 用例那一格（要 4095 才绿）；cross = 段号 0..39 × 全部 4095 个长 12 子集共 163800 对里种子两两不同的个数；len16 = 长 16 的 65535 个子集里种子两两不同的个数。

```
指纹变体                                 len12(用例)     cross(段0..40×12写)    len16(65535子集)  用例结论
入库 FNV-1a 全折                              4095                163800             65535  绿(拦不住)
只折第一个写(.take(1))                             2                    80                 2  红
少折最后一个写(.take(len-1))                     2048                 81920             32768  红
跳过第一个写(.skip(1))                          2048                 81920             32768  红
只折前 11 个写(.take(11))                      2048                 81920              2048  红
只折前 12 个写(.take(12))                      4095                163800              4096  绿(拦不住)
只折前 13 个写(.take(13))                      4095                163800              8192  绿(拦不住)
只数持久的个数(popcount)                           12                    48                16  红
只取第一个被扣下的写的下标                               12                    48                16  红
掩码当整数直接异或(低位 mask)                        4095                  4096             65535  绿(拦不住)
FNV 截成低 32 位                              4095                163800             65535  绿(拦不住)
FNV 只留低 12 位                              2799                  4096              4096  红
FNV 只留低 16 位                              4015                 61176             42919  红
常数 0                                         1                    40                 1  红
只折最后 12 个写                                4095                163800              4096  绿(拦不住)
```

读法：
- 抓得住：只折第一个写（2 个，与攻方 `fp=first-only` 的 2 个一致，`research/prompts/m3-formal-code-b21-r1-opus-model/out/f3.txt:18` 那一行 `distinct_seeds=2`）、少折一端、只折前 11 个、只数个数、只看第一个被扣下的写（12 个，与攻方 `fp=first-withheld` 的 12 个一致）、常数、低 12 / 16 位。
- 抓不住：只折前 12 或 13 个写、只折最后 12 个写（窗口式）、截成低 32 位、直接用掩码当整数。前两类在长 12 的段上与全折没有区别；掩码当整数的 cross 列只剩 4096 个（段号 s 与掩码 m 的异或在 (16, 0) 与 (0, 16) 上撞），用例固定段号 16 看不出来。
- 这些「抓不住」在真历史里碰得到：我在 harness 档按 `generate_history` 取 10 个种子 × 2 档盘宽（384 槽、4 GiB）各跑 16 步，写表切段后的段长分布是 `{1: 350, 2: 634, 4: 20, 12: 27, 14: 30, 16: 42, 18: 9, 20: 73, 22: 7, 24: 55, 26: 2, 28: 21}`，共 1270 段、239 段长于 12，最长 28（`research/prompts/m3-formal-code-b23-r1-sonnet-model/probes/b23_segment_lengths_probe.rs`，输出 `…/out/segment-lengths.log`；这是全段含 mkfs 那几段，候选段子集没单数）。代码自己也预期长段：「`const WRITES_A_SUBSET_MASK_HOLDS: usize = 63;`」（`crates/singlefs-checker-tier/src/crash_injection.rs:121`）说段内写数超过 63 就换抽法。长 12 的用例对这些段上的弱指纹没有判别力。

可行的补法（没做、没量过）：加一格长度超过 12 的段（如 16，65535 个子集，4095 个以内抽样也行），再加一格段号变、子集变的对（cross 那一类）。

**推翻条件**：上表任一格的数在别的实现上不同（算术是我自己写的 FNV 抄原文常量；`f3c_identity.out` 已证抄的公式与合入后一致）；或真历史里长于 12 的段不会被摆进二次崩溃（第一次崩溃态从段内取子集，见 `crash_injection.rs` 的枚举域注释，我没逐段数候选段的子集，只读到「抽的那一路同一段同一个子集只摆一次」）。

## 五、K3-a：H2 两条用例挂载次数从根环槽数算之后，前提断言在别的根环槽数下成不成立

**判定：在今天 mkfs 能写的每一个 S（5 ≤ S ≤ 15）上成立（量：S = 5、6、7、8、12、15）；前提断言有判别力（量：把挂载次数改错一次，红在前提断言上）。一处注释偏差与一处域边界见下。**

读码（H2 文件 `crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs`）：

- 根环一圈的槽数是「`const ROOT_RING_SLOT_COUNT: u64 = ROOT_RING_REGIONS * ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM;`」（`crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs:52`），「回收了」那条用例的挂载次数是「`ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM + 2;`」（`crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs:65`）。
- 前提断言：写它之前环里最旧有效根 txg = 第一个文件版本 txg 减一圈，「`first_file.root.checkpoint_txg.0 - ROOT_RING_SLOT_COUNT;`」（`crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs:532`）；「回收了」那条断言它不早于释放代「`oldest_valid_root_txg_before_the_first_file >= RELEASE_TXG_OF_THE_MKFS_INSTANCE_TABLE,`」（`crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs:534`），「差一个 txg」那条断言早恰好一个「`oldest_valid_root_txg_before_the_first_file + 1,`」（`crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs:578`）。
- 与实现的谓词的关系：实现的回收谓词是「`.filter(|record| record.is_released && record.generation <= floor)`」（`crates/singlefs-core/src/allocator.rs:1736`），条款是「已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)」（`.claude/kb/decisions/03-空间分配.md:133`）。用例的前提断言是算术（txg − 3S），不是读环得到的最旧有效根；两者在「环已转满一圈」时一致，k = S + 1 时第一个文件之前已发 3S + 1 个根、环已转满。

算术：量下表（`research/prompts/m3-formal-code-b23-r1-sonnet-model/probes/b23_h2_txg_probe.rs`，S = 8，输出 `…/out/h2-txg.log`）：第 k 次只暖机挂载上第一个文件版本的 txg 是 k = 2 → 5、3 → 8、5 → 14、8 → 23、9 → 26、10 → 29、11 → 32、12 → 35（= 3k − 1），k = 1 → 3（不等于 3k − 1 = 2）。于是 k = S + 2 时 txg − 3S = 5 ≥ 3，k = S + 1 时是 2，加一等于 3，都与 S 无关；k ≥ S + 1 ≥ 6 落在 3k − 1 成立的范围里。

量：`research/prompts/m3-formal-code-b23-r1-sonnet-model/h2_sweep.sh` 在我的副本上把 `singlefs-format` 的 `ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM` 改成 S，跑这两条用例（输出 `…/out/h2-sweep.log`）；slack1 是 `mutations.tsv` 里 21-1 那行的变异（回收门槛放松一个 txg，改 `allocator.rs:1736`）：

| S | 原样两条 | 放宽一个 txg |
|---|---|---|
| 5 | 2 passed | 「差一个 txg」红在落点断言（左 `[(0, 50176), (1, 50176)]`、右 `[(0, 50180), (1, 50180)]`），「回收了」绿 |
| 6 | 2 passed | 同上 |
| 7 | 2 passed | 同上 |
| 8 | 2 passed | 同上 |
| 12 | 2 passed | 同上 |
| 15 | 2 passed | 同上 |
| 4、16 | 编不过 | 编不过 |

S = 4 与 16 在今天的 mkfs 路径上编不过：编译期断言「`slots_per_region > ROOT_RING_SLOTS_PER_REGION_MINIMUM`」（`crates/singlefs-core/src/root_ring.rs:50`）与「`&& slots_per_region < ROOT_RING_SLOTS_PER_REGION_MAXIMUM,`」（`crates/singlefs-core/src/root_ring.rs:51`）要求严格在 4 与 16 之间，而 H2 文件的 `parameters()` 用的就是 `AT_MAKE_FILESYSTEM`（`crates/singlefs-harness/tests/common/mod.rs:65`）。所以 H2 文件注释里写的「正推在 S = 5、6、7、8、12、16」（`crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs:62`）里的 16，今天走不到这条路：b21 正推那次的 16 走的是运行时入口「`if !(ROOT_RING_SLOTS_PER_REGION_MINIMUM..=ROOT_RING_SLOTS_PER_REGION_MAXIMUM)`」（`crates/singlefs-core/src/root_ring.rs:65`，闭区间）加它自己的探针（`research/prompts/m3-formal-code-b21-r1-sonnet-model/b21_scan.rs:31`），不是这个用例文件的路。不影响 H2 的判定，是注释里一个范围说宽了。

前提断言的判别力（量，`h2_premise_mutants.sh`，输出 `…/out/h2-premise-mutants.log`）：
- S = 8、「回收了」那条挂载次数改成 S + 1：红在前提断言，消息「前提：第 9 次挂载上的第一个文件版本（CheckpointTxg(26)）写之前环里最旧有效根 txg 2 不早于 mkfs 实例表那一对的释放代 3」；
- S = 12 同一变异：红在前提断言，「第 13 次挂载……txg 2 不早于……释放代 3」；
- S = 8、「差一个 txg」那条挂载次数改成 S + 2：红在前提断言，消息「前提：第 10 次挂载上的第一个文件版本（CheckpointTxg(29)）写之前环里最旧有效根比 mkfs 实例表那一对的释放代早恰好一个 txg」，left: 6（最旧有效根 txg 加一）、right: 3（释放代）。
- 「回收了」那条改成 S + 1 时，「差一个 txg」那条（它的次数是前一个减一 = S）红在 576 行 `txg − 3S` 那个无符号减法上（行号列号推的下溢：挂载 S 次时 txg 3S − 1 < 3S；消息行没进我的过滤）。

两处小偏差（不改判定）：
- 常数注释「每次只暖机的挂载发 3 版」（`crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs:59`）对前两次挂载不对（k = 1 时第一个文件 txg 3、k = 2 时 5：头两次挂载各发 2 版，从第 3 次起各发 3 版）；3k − 1 的式子对 k ≥ 2 成立；H2 取的 k 是 S + 1、S + 2，S ≥ 5 时 k ≥ 6，不受影响。
- 两条用例的函数名还写 nine / ten（正文第二节「23-1 交主 agent 的」已列）：只有 S = 8 时名字与挂载次数对得上；换 S 时名字说错次数，判决不受影响。

**推翻条件**：换一个 S ∈ [5, 15] 后两条原样用例有一条红、或前提断言先于落点断言红（我扫的 6 个值上没出现）；或某个 S 上放宽一个 txg 的变异两条都绿（没出现，六个值上「差一个 txg」都红）。

## 六、K3-b：`writable_mount_refusal_on_a_crash_state_without_read_faults.rs` 那句注释与 harness 实现逐句对

**判定：四句都对得上代码（量一处 + 读码），一处偏差在同一文件的模块文档（第 4–5 行没跟着改）。**

注释在 `crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs` 第 287–290 行。拆成四句，各对一处：

1. 「撕裂系统配置那一支：盘 0 两槽都撕裂、没注入读故障，实现在取号之前以」（`crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs:287`）「这块盘一份自证过的系统配置都没有」拒、报成判据 N-配置 的拒因。
   - 取号之前：核盘落后的那一步「`let devices_without = devices_without_the_selected_version(`」（`crates/singlefs-core/src/mount.rs:4584`）在「`let instance = acquire_expected_instance(&mut pool, instance_to_acquire).map_err(`」（`crates/singlefs-core/src/mount.rs:4609`）之前；它在某块盘两槽里一份自证过的都读不出时「`return Err(device);`」（`crates/singlefs-core/src/mount.rs:3823`），调用点 `map_err` 成「`WitnessedCounterComparison::SomeDeviceWithoutAnySelfVerifiedSystemConfigurationSlot,`」（`crates/singlefs-core/src/mount.rs:4594`）的拒因，包进 `NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion …)`（`mount.rs:3869–3880` 那个构造函数），即判据 N-配置 的拒因。用例前提断言取到的就是这一支：「`Some(WitnessedCounterComparison::SomeDeviceWithoutAnySelfVerifiedSystemConfigurationSlot),`」（`crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs:299`）。
   - 对：取号那一步里也有同一个拒因的第二处出口（`mount.rs:4631–4636`，「一个字节都没写」），两处都在第一个写之前；这个用例走的是 4584 那一处。
2. 「（跳过撕裂的槽取 c_见证），不因盘 0 一份自证过的都没有就按真；这份盘面上从盘 1 自证过的槽取、」（`crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs:288`）：C670 那一条在「某块盘两槽都撕裂」这一形上照常比。
   - 代码：C670 的复算是「`fn witnessed_counter_holds_on_a_crash_image_without_read_faults(`」（`crates/singlefs-harness/src/mount_transparency.rs:1493`），c_见证 取所有盘全部自证过的槽 journal tail 的最大值（`mount_transparency.rs:1517`、1519 行，一份都没有取 0），文档写明「两边只在一形上分开：某块盘两槽都撕裂、」（`crates/singlefs-harness/src/mount_transparency.rs:1490`）判据本身（`newer_publish_witnessed_on_the_crash_image`）那一判按真、C670 这里照常比。对。
3. 「c_见证 等于第一个文件那一版末条的计数器，复算为假，不报（换一份盘面照常比可以为真）」（`crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs:289`）。
   - 量（`probes/b23_k3_comment_probe.rs`，输出 `…/out/k3-comment.log`）：盘 0 两槽撕裂后自证过的槽 journal tail 按盘是 `(盘 0, [])`、`(盘 1, [2, 3])`，c_见证 = 3；所选那一版 (实例 1, txg 3) 的末条记录计数器 3；c_见证 > 末条 = false ⇒ 复算为假。健康盘面上同样是 3 与 3。同文件 3 条用例我在副本上跑了（`…/out/writable-mount-refusal-tests.log`）：撕裂那条与注入读故障那条合法、「只落了轮换」那条「`fn a_witness_refusal_on_a_crash_image_without_read_faults_where_only_a_rotation_landed_is_illegal()`」（`crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs:228`）不合法——括号里「换一份盘面照常比可以为真」有这一条作证。对。
4. 「不算判不出（没有槽设备报读错），盘 0 一份自证过的系统配置都没有 ⇒ 按真：合法。」（`crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs:290`）：判不出的定义「`some_slot_reported_a_read_error && !some_device_self_verified_both_slots`」（`crates/singlefs-harness/src/mount_transparency.rs:1371`）在没有读错时为假；判据复算在判不出或某块盘一份自证过的都没有时按真，「`|| some_device_without_any_self_verified_system_configuration_slot_on_the_crash_image(`」（`crates/singlefs-harness/src/mount_transparency.rs:1414`）后面紧跟「`return true;`」（`crates/singlefs-harness/src/mount_transparency.rs:1418`）。用例断言它合法（`…tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs:312–314`），副本上跑绿。对。

**偏差（读文件）**：同一文件的模块文档第 4–5 行仍是合入前的措辞「跳过撕裂的槽复算为假、不报，拒照 U6 判」（`crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs:5`，起头在 4 行「某块盘两槽都撕裂那一支……：C670 那一条」），没带上「只对这份盘面成立、换一份可以为真」这层限定，与 b21 判决 K1-d 指出的偏宽同一句话。`grep -rn '跳过撕裂的槽复算为假'` 在冻结副本的 `crates/` 下只命中这一处。这是 23-1 只改了函数注释、没改模块文档留下的一处。

**推翻条件**：三条之一——盘 0 两槽都撕裂的盘面上，C670 复算在这份盘面上为真（我的探针里 c_见证 = 3 = 末条计数器，为假）；取号之前的拒不是 4584 那一处也不是 4631 那一处；「只落了轮换」那条用例红。都没出现。

## 七、K3-c：`root_record.rs` 第 26 行注释与 `recovery.rs` 逐句对

**判定：对得上（读码）；有一处读法上要提醒的（不是错）。**

注释：「flags 位 0 = 0：别的每一条根——mkfs 的第 0 代、暖机、写行、发布、准入抬 F、回退；由记录施加出来的那一版取那条记录新根段里的记号（D23（journal 的角色与格式） 已定项 15，`recovery` 施加处）。」（`crates/singlefs-core/src/root_record.rs:26`，它是 `UnmountMarker::NotWrittenByTheUnmountSequence` 的文档）。

| 句子的一块 | 代码 | 对 |
|---|---|---|
| mkfs 的第 0 代 | 「`unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,`」（`crates/singlefs-core/src/make_filesystem.rs:581`） | 对 |
| 暖机、写行、发布 | `publish_version` 给 `NotWrittenByTheUnmountSequence`（`crates/singlefs-core/src/transaction.rs:6161` 起那一次调用）；第一个文件与会话发布同样（`transaction.rs:1368`、1886 行的根） | 对 |
| 准入抬 F | 「`RaiseFloorEntry::Admission => UnmountMarker::NotWrittenByTheUnmountSequence,`」（`crates/singlefs-core/src/mount.rs:2435`） | 对 |
| 回退 | 管理员回退那次发布「`UnmountMarker::NotWrittenByTheUnmountSequence,`」（`crates/singlefs-core/src/transaction.rs:6222`） | 对 |
| 由记录施加出来的那一版取那条记录新根段里的记号 | 「`unmount_marker: record.new_unmount_marker,`」（`crates/singlefs-core/src/recovery.rs:3601`，`replay_journal` 里施加末条那一段，3596 行注释「每条记录都带整次发布的新根段（已定项 15），施加末条那一份」） | 对 |
| 条款 | 「实例表指针、卸载记号、分配记录树根从新根段来，不再照抄所选根」（`.claude/kb/decisions/23-journal的角色与格式.md:451`） | 对 |

要提醒的一处：这句话挂在 `NotWrittenByTheUnmountSequence`（flags 位 0 = 0）这个变体的文档下，而分号后面的「施加出来的那一版取记录里的记号」不总是 0——卸载那一串的根带 `WrittenByTheUnmountSequence`（「`RaiseFloorEntry::Unmount => UnmountMarker::WrittenByTheUnmountSequence,`」，`crates/singlefs-core/src/mount.rs:2436`），它那些发布写的记录新根段照抄根上的记号（「`new_unmount_marker: root.unmount_marker,`」，`crates/singlefs-core/src/transaction.rs:1399`），崩在记录持久、根槽没持久时恢复施加出来的那一版带的是 1。这一句话的字面是对的（「取记录里的记号」），但放在「= 0」的列表后面容易被读成「施加出来的也是 0」，那正是合入前旧措辞的错。我没有在副本上造「卸载发布崩在记录与根槽之间」的态去量施加出来的根带 1（读码 + 条款 `23-journal的角色与格式.md:460` 里三方判决「首个文件、卸载两种发布补进去之后施加出来的根与当初会写的逐字节相同，攻方量的」，那一次是攻方的量，我没复跑）。

**推翻条件**：`recovery.rs:3601` 施加处在某条路径上不取记录里的记号而取所选根的（我只读到这一处构造 `RootRecord`）；或卸载那一串的记录新根段带的不是 1（`transaction.rs:1399` 是照根抄，根由 `entry.unmount_marker()` 给，`mount.rs:3110`、3152 两处调用）。

## 八、K1-e：23-1 为 K1 加的两行变异的锚点（只核锚点，没跑）

`research/prompts/m3-formal-code-b23-r1-sonnet-model/mutation_anchors.py` 读冻结副本的 `crates/mutations.tsv`（共 2422 行），逐行核第 2407–2413 行的原文在目标文件里命中几次（输出 `…/out/mutation_anchors.out`）：第 2407 行（K3-a 接线变异）、第 2408 行（F3-c 弱指纹变异）、A1 四行与 B1 一行都是命中 1 次、替换与原文不同。第 2407 行的原文与 21-2 那行一字不差，命中的是 `crates/singlefs-checker-tier/src/crash_injection.rs` 里第三截那一处（3054 行，不是第二截那一处 1404 行：原文要连着上一行「`crash_image_materialized: &second_crash_image_materialized,`」才命中，1404 行那处写的是 `&crash_image_materialized`）。两行变异点名的测试各是各自的新用例（2407 行是 K3-a 用例，2408 行是 F3-c 用例）。

变异本身红不红（用例 + 变异在 checker 档要编、要跑）我没跑：K3-a 在判官一层量到 (1, 1) → (1, 0)（第一节与第二节），调用点把 `report.effective_root` 换成 `fields_read_after_the_first_crash.report.effective_root` 在判官一层就是我探针里的「接错」；F3-c 的 `.take(1)` 在算术上给 2 个种子（第四节表第 2 行，与攻方 `f3.txt` 同）。这两条都停在「推的」。

## 九、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1-a K3-a 的态 | 一致（量 + 读码） | 写下标 53、54、56、57、根 80、第二截 16–19、withheld `[0, 1, 4]` / `[0, 1, 3]` 与攻方 `k3-sweep.txt` 逐个对上；两次崩溃落点 (1, 4) 与 (2, 5) 不同 |
| K1-b 断言 (1, 1) / (1, 0) 与写下标现算 | 对（判官一层量，调用点与计数读码推） | 接对判官列 C670 条件、接错只列「判据复算为假」；7 个配置（盘宽、覆盖写内容、覆盖写次数）上都对，形状变了前提断言先响 |
| K1-c F3-c 逐位相同 | 一致（读码 + 量抄出的两份公式） | 200 万随机加 262 万穷举 0 个不同 |
| K1-d 4095 断言抓别的弱指纹 | 部分 | 抓得住折得太少（`.take(1)`、`.take(11)`、popcount、低 12/16 位）；抓不住 `.take(12/13)`、最后 12 个窗口、低 32 位、掩码当整数；真历史段长到 28 |
| K1-e 两行变异的锚点 | 各命中 1 次 | 变异红不红在 checker 档，没跑 |
| K3-a H2 前提断言换 S | 成立（量 S = 5、6、7、8、12、15） | 前提断言有判别力（挂载次数改错红在前提）；放宽一个 txg 的变异在各 S 都红在「差一个 txg」那条；S = 4、16 在今天的 mkfs 路径上编不过 |
| K3-b 第 287–290 行注释 | 对得上（量一处 + 读码） | 取号之前、照常比、c_见证 = 3 = 末条计数器、按真合法四句都对；模块文档第 4–5 行仍是旧措辞 |
| K3-c `root_record.rs:26` 注释 | 对得上（读码） | 施加处 `recovery.rs:3601` 取记录里的记号；放在「= 0」列表后面可能被读成 0，卸载发布的记录带 1（读码，没造态量） |

## 十、附带发现（没人问，读到的）

1. `crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs` 模块文档第 4–5 行没跟着改（第六节）。
2. `crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs` 常数注释里的 S = 16 在这个文件走的 mkfs 路径上编不过（5 ≤ S ≤ 15）；「每次只暖机的挂载发 3 版」头两次挂载不是（第五节）。
3. F3-c 的 12 个写只测到长度 12 的段，真历史有长到 28 的段（第四节）。

## 十一、没做什么

- checker 档（`singlefs-checker-tier`）一个测试、一条变异都没跑也没编：K3-a 用例与 F3-c 用例本身红绿、变异 2407、2408 红不红、(1, 1) 与 (1, 0) 在注入函数里的真值，都是「推的」；我在 harness 档量到的是判官一层与算术，调用点接线、`tally` 的记数、`inject_crashes_into_history_drawing_second_crashes` 走的路径读码推。提交时主 agent 跑 checker 档才能定。
- K2（A1、B1）不是我的格，没判；变异 2409–2413 只核了锚点命中数。
- 不替主 agent 采纳或出判决；第四节「可行的补法」只是读出的缺口、没做没量。
- 我的数都在副本上量：`/tmp/claude-1000/m3-formal-code-b23-r1-sonnet/ws/` 与 `/tmp/claude-1000/m3-formal-code-b23-r1-sonnet/target/`（编译目录，交回前已删，量过的探针源与输出都在 `research/prompts/m3-formal-code-b23-r1-sonnet-model/`）。副本上的数不进 kb；要引需在入库装置上重做。
- 没复跑攻方 b21 的探针（只读了它的产物与源）；没读禁读清单里的任何文件。
- 没造「卸载发布崩在记录与根槽之间」的态去量施加出来的根带 1（第七节）。
