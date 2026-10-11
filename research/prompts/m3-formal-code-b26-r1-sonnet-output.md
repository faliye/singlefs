# 第 26 批代码轮第一轮：云端正推腿（Sonnet）报告（2026-10-05）

<!-- doc-lint:not-numbers J1 J2 J3 C1 C2 C3 C4 R1 R2 F3 A2 B3 RR8 M7a M7d M7f Y1 Y2 Y3 K1 H6 U6 -->

轮 m3-formal-code-b26-r1，腿：云端正推；格 J1、J2、J3；立场：从三份判决与条款正推每处改动做的是不是条款说的，再逐格比。前一轮判决：没有（第一轮）。
读：背景材料 `research/prompts/_m3-formal-code-b26-r1-background.md`、改动全文 `_m3-formal-code-b26-r1-diff.md`、三份判决（`m3-formal-c655-c687-r1-main-verification.md`、`m3-formal-code-b24-r1-main-verification.md`，26-2 那一批没有单独判决，钉值的出处是 2026-10-04 主工作区 checker 档日志）、冻结副本 `/tmp/claude-1000/m3-formal-code-b26-r1/tree/`。
引 kb 写 kb 文件自己的行号（逐条 `grep -nF` 现查）；代码引文是冻结副本的 `crates/…` 路径与那棵树里的行号；我自己的产物在 `research/prompts/m3-formal-code-b26-r1-sonnet-model/`（`tools/rerun.sh` 复跑，`out/` 是原样输出，`in/` 是从两份日志摘出来的原行）。

## 〇、实现今天的样子（我的观测）与我复跑了什么

- 冻结副本 297 个文件；我拷到草稿 `/tmp/claude-1000/m3-formal-code-b26-r1-sonnet/work*`（排除 target）里编、跑。**只跑了 harness 档**：`cargo test -p singlefs-harness`，经 `capped.sh 7` 与 `run-with-memory-cap.sh 12G`；checker 档测试、`crates/mutations.tsv` 整表、GPU 一格没跑，涉及它们的结论写「推的」。
- 未改动的冻结副本上：`checker_known_bad_images` 83 过（我的探针加进去后 84）、`mount_transparency_judged_field_by_field_after_a_writable_mount` 45 过（加 2 个探针 47）、`allocator_fallback_…` 10 过、`new_pool_file_creation_publish` 9 过、26-1r 新加的几份测试与随机历史慢档用例都过：`research/prompts/m3-formal-code-b26-r1-sonnet-model/out/harness-b26-unmutated.txt` 与 `…/out/harness-j2-unmutated.txt`。
- 补丁里 47 行变异行：38 行点名 harness 档，我逐行打进拷贝、跑那一行点名的测试，**38 行全红**（主 agent 的记录里 26-3 只证了 2 行，其余 21 行留提交时）；另 9 行点名 checker 档（26-2 的 C577 一行、26-1r 的 `crash_injection.rs` 八行）没跑：`research/prompts/m3-formal-code-b26-r1-sonnet-model/out/mutation-rows-final.tsv`。47 行的旧串在冻结副本里各恰好命中 1 次：`tools/check_anchors.py` 末行 `rows 47 bad 0`。
- 主 agent 观测里的「新加的 checker 档用例与各份变异行只编过」我同样没法补：见末尾「没做什么」。

## 一、J1：26-3（C655、C687 的改法）

### J1-1　代码做的是不是判决说的：**一致**，另有两处与判决的前提或口径对不上（C1、C2，下面逐条）

判决第三节去向第 1 条列了七处改动，逐处对：

| 判决要的 | 代码 | 对没对 |
|---|---|---|
| I-9.16 加 ③ 每种至多一条、④ 树表非空时种类 1–4 各恰好一条 | 「first_kind_carried_more_than_once.is_none(),」(`crates/singlefs-checker/src/walk.rs:3561`)；「judgements.judge("I-9.16", first_rooted_kind_missing.is_none(), || {」(`crates/singlefs-checker/src/walk.rs:3571`)；先于 ② 判：「judge_tree_table_entry_kind_counts(judgements, tree_table_entries, version_name);」(`crates/singlefs-checker/src/walk.rs:3505`) | 对。③ 的谓词与 core 的 `tree_table_entries_each_kind_at_most_once`（「if !kinds_seen.insert(entry.kind) {」，`crates/singlefs-core/src/recovery.rs:2335`，登记与没登记的种类都算）同一个。④ 窄于 core 可写路径，见 C1 |
| I-5.5 判不了的几格改「部分没判」，某盘没有数据单元跳过 | 「if let Some(&smallest_slot) = smallest_data_unit_slot_on_each_device.get(device) {」(`crates/singlefs-checker/src/walk.rs:4372`)；「if !versions_not_judged.is_empty() {」(`crates/singlefs-checker/src/walk.rs:4392`) 后接 `partially_judged` | 对（树表读不出、条目窄、extent 树读不全、分配记录树读不全、种类 1 不止一条、种类 3 不是恰好一条六处各 `versions_not_judged.push`，第 4274、4288、4303 行起）。种类 1 一条都没有仍「[] => continue,」(`crates/singlefs-checker/src/walk.rs:4300`)：判决没列这一格，且「树表非空缺种类 1」由 ④ 判红 |
| I-2.4 加根槽 428..457 全 0，每条可读根判 | 「const ROOT_RECORD_ALGORITHM_NONCE_AND_MAC: std::ops::Range<usize> = 428..457;」(`crates/singlefs-checker/src/image.rs:1296`)；调用「judge_root_record_algorithm_nonce_and_mac_are_zero(&mut root_ring_judgements, &roots);」(`crates/singlefs-checker/src/walk.rs:7987`) | 对（`roots` 是 `valid_roots`，不按候选集滤） |
| 带树一版分配记录树根全零 | 「allocation_roots[0].all_zero,」(`crates/singlefs-checker/src/walk.rs:4861`) 挂在 I-5.4 | 对判决；挂法的副作用见 C2 |
| 挂载透明：三根指针并进根指针那一行 | 「pub const TREE_KINDS_WHOSE_ROOTS_THE_MOUNT_KEEPS: [u16; 4] = [」(`crates/singlefs-harness/src/mount_transparency.rs:573`) 旁边是只由续做改根的五种（种类 1、2、9、10、11） | 对 |
| `mapping_root` 与分配记录树根的空性关系 | 「is_all_zero(after) == is_all_zero(before),」(`crates/singlefs-harness/src/mount_transparency.rs:1886`)；「let expected_non_zero = new_instance.0 >= 2;」(`crates/singlefs-harness/src/mount_transparency.rs:2051`) | 对 |
| journal_tail 与系统配置 journal 实例代号两行 | 「&& tail_after <= highest_record_counter」(`crates/singlefs-harness/src/mount_transparency.rs:2112`)；「&& (!mount_completed || tail_after == highest_record_counter)」(`crates/singlefs-harness/src/mount_transparency.rs:2113`)；实例代号「observed.is_some() && (observed == before || observed == new_instance),」(`crates/singlefs-harness/src/mount_transparency.rs:2151`) | 对判决第二节 Y1 两行 |
| 运行配置「R2 = 内置默认」 | 「observed == Some(built_in.as_slice()),」(`crates/singlefs-harness/src/mount_transparency.rs:1919`) | 对 |
| 回落看门（分配器只读计数 + harness 轻用例） | 计数在回落那一支里加一；用例 10 过（`research/prompts/m3-formal-code-b26-r1-sonnet-model/out/harness-b26-unmutated.txt` 第 2 行）；变异行「分配器回落计数不加」红 | 对 |

核：全部 38 条 harness 档变异行红（含 26-3 的 23 行：`research/prompts/m3-formal-code-b26-r1-sonnet-model/out/mutation-rows-final.tsv` 第 3 至 25 行，行首是改动全文里的行号）。

### J1-2　判决第五节六条零轮形态，逐条

**(1) I-9.16 ③ ④**：合法历史上没见误红。依据：未改动副本上 83 个坏镜像用例与 153 个合入验证里的测试都过（后者是主 agent 的观测，我没复跑）；③ 与 core 同谓词；④ 要的种类 1–4 是 core 读路径要的（见 C1 里的并排）。放过坏镜像的一面见 **C1**。

**(2) 根槽归 I-2.4**：一致。core 不判那 29 字节：「实现今天不判这 29 字节」(`crates/singlefs-checker/src/image.rs:1302`)，checker 另判一份；三条坏镜像（m4–m6）红在 I-2.4（变异行 3674–3676 红）。GPU 判器没改，补丁交 26-5（记录第十八节 26-3 交回那一条：「GPU 没改」）；坏镜像上 CPU 红而 GPU 不红的分歧在 26-5 之前存在，合法历史上两边一致（推的，GPU 没跑）。

**(3) journal_tail 不等式与系统配置 journal 实例代号**：没找到误红，但这是推的加我的探针，不是证明。
- 推：每次发布先发记录、过池屏障、再发根槽 FUA、再每盘轮换系统配置写 tail（`CommitStep::RotateSystemConfigurationSlots` 在 `crates/singlefs-core/src/transaction.rs`），所以一块盘上轮换写落了，记录必已持久，`tail ≤ 环里最大计数器` 成立；取号写带的是见证值（全池自证过的槽的 tail 最大值），不超过环里记录。二次崩溃里下界 R1 ≤ R2：新写总落在两槽里较旧的那一槽，撕裂只毁较旧那一槽，R1 择到的那一槽留着。
- 探针（`probe/mount_transparency_probe.rs.append`）：7 种底盘面（mkfs 之后、重挂一次无文件、第一个文件、卸载后、由记录重建、挂载推抬 F、崩在 unlink 意图之后）上，只挂载与挂载加一次发布各判一遍：13 个场景一样都没红（`research/prompts/m3-formal-code-b26-r1-sonnet-model/out/probe-mount-transparency-whole.txt:13`「PROBE_WHOLE base=crashed_right_after_the_unlink_intent action=mount_alone violated=[]」）；每个底盘面上挂载写表的每个写前缀各摆一个二次崩溃共 341 个，红 0：`research/prompts/m3-formal-code-b26-r1-sonnet-model/out/probe-mount-transparency-second-crash-prefix.txt:8`「PROBE_SECOND_TOTAL states=341 red=0」。只摆了写前缀，崩溃注入还会摆段内子集与撕裂的原地覆写，那一面没量（攻方的活，也是 checker 档）。
- 判别力缺口（不是逻辑错）：上界一半有变异行与红用例；**R1 ≤ R2 与「挂载做完取等」两半没有一条红用例、没有变异行**（用例 `a_journal_tail_past_every_record_in_the_ring_of_a_second_crash_image_is_red` 只改成越过上界，`crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs:1554`；变异行只有「journal_tail 去掉上界」一行）。把 `tail_before <= tail_after` 或取等那一半改成恒真，今天没有一条测试红（读码，没造变异）。

**(4) `mapping_root` 与分配记录树根的空性关系**：一致。判决 Y1 那两行与代码逐句对上；我的整挂载探针里「第一次挂载（实例 1）R2 全零」「重挂（实例 ≥ 2）R2 非零」两种底盘面都判绿（`probe-mount-transparency-whole.txt` 第 1 至 4 行）；两条变异行红（3683、3684）。

**(5) 运行配置「R2 = 内置默认」**：一致，一行变异红（3685）；今天没有挂载选项通道，写槽总写默认（判决 Y2 的前提，我没另验）。

**(6) 回落看门**：一致。`the_fallback_count_is_above_zero_once_no_empty_cluster_segment_is_left`（`crates/singlefs-harness/tests/allocator_fallback_count_on_small_devices_after_repeated_mounts_and_the_first_file.rs:211`）是阳性对照，变异行 3678 红；阴性那一半（挂载 1、12、60 次后接第一个文件，计数 0）10 个用例过。

### J1-3　判决里两处与今天实现对不上（并排抄）

**C1　I-9.16 ④ 的「种类 1–4」窄于 core 可写路径要的种类：冲突**

| 判决（第一节） | 今天的实现 |
|---|---|
| 「core 在树表非空时要的有根的树是种类 1、2、3、4」(`research/prompts/m3-formal-c655-c687-r1-main-verification.md:17`) | 可写挂载接着这一版发布走 `rebuild_version_in_the_unit_area_starting_at`：先 4 个 `pointer_of`（「let extent_pointer = pointer_of(TREE_KIND_EXTENT)?;」(`crates/singlefs-core/src/recovery.rs:2533`)），再逐种取树 ID，**十一种各要一条**：「deadlist: tree_of(TREE_KIND_DEADLIST)?,」(`crates/singlefs-core/src/recovery.rs:2556`)、「logged_operations: tree_of(TREE_KIND_LOGGED_OPERATIONS)?,」(`crates/singlefs-core/src/recovery.rs:2560`)，缺了报「what: "树表里没有那棵树",」(`crates/singlefs-core/src/recovery.rs:2545`) |

判决那句只对读路径（`walk_to_file`、`:4927-4941` 报的「树表里缺一棵有根的树」，今天在 `crates/singlefs-core/src/recovery.rs:4991`）成立。④ 照 4 种写的：「const TREE_KINDS_EACH_EXACTLY_ONCE_IN_A_TREE_TABLE_WITH_ENTRIES: [u16; 4] = [」(`crates/singlefs-checker/src/walk.rs:3531`)。
我在拷贝里造了一份（`probe/known_bad_probe.rs.append`，用 26-3 自己的 `remove_tree_table_entry` 去掉 deadlist 那一条、重封校验和）：
- 池级 checker：`research/prompts/m3-formal-code-b26-r1-sonnet-model/out/probe-i916-missing-kind.txt:1`「PROBE_I916 deadlist_entry_6 violated=[]」与 `…:2`「PROBE_I916 deadlist_entry_6 I-9.16=Holds」，一条红都没有；
- core 可写挂载同一份盘：`research/prompts/m3-formal-code-b26-r1-sonnet-model/out/probe-core-writable-mount-missing-kind.txt:1`「PROBE_CORE writable_mount_outcome=Err(Recovery(UnitMalformed { what: "树表里没有那棵树" }))」。
去掉 logged_ops 那一条只红 I-1.1（`research/prompts/m3-formal-code-b26-r1-sonnet-model/out/probe-i916-missing-kind.txt:3`），I-9.16 也是 Holds。
含义：缺种类 6、7、8、9、10、11、12 任一种的树表，实现拒、checker 不红；④ 放过这一类。kb 的 I-9.16（`.claude/kb/invariants.md:303`）今天只有 ① ②，没有 ③ ④（我 grep 过，写回批还没做），所以条款没说该不该红：**规则没说**，判决里「就是种类 1–4」的前提是我这一格的冲突点。
推翻它的现象：如果 `rebuild_version_in_the_unit_area_starting_at` 对缺种类 8 的树表其实不拒（我的探针里它拒了，见上），或判决本意就是 ④ 只管读者；那样这一格改记「一致、判决范围窄」。

**C2　I-5.4 的挂法把「分配记录树读不出」从「不适用」变成「成立」：冲突（判决没碰，实现员留了「归属交主 agent 定」）**

| 条款 / 旧行为 | 今天 |
|---|---|
| I-5.4 判的是分配记录树里的记录互不相交：「任何一条有效根的分配记录树里，同一块盘上任意两条分配记录」(`.claude/kb/invariants.md:198`)；旧代码走不到记录的根不判，整条 `not_applicable`：「候选集里没有一条根的分配记录树走得到叶」(`crates/singlefs-checker/src/walk.rs:4908`) | 新判「allocation_roots[0].all_zero,」(`crates/singlefs-checker/src/walk.rs:4861`) 只要树表非空就评估一次；而判定优先级是「} else if self.evaluated.get(invariant).copied().unwrap_or(0) > 0 {」(`crates/singlefs-checker/src/image.rs:287`)，评估过就报 Holds，压过 `not_applicable`（即使「if judged_allocation_nodes == 0 {」(`crates/singlefs-checker/src/walk.rs:4905`) 成立） |

探针：把新池新建文件那一版的分配记录树根（槽 50249）或一片叶（槽 50245）两份全零。旧代码（我在拷贝里只去掉 26-3 加的那个 `judge`）：`research/prompts/m3-formal-code-b26-r1-sonnet-model/out/probe-i54-old.txt:2`「PROBE_I54 root_50249 zeroed=NotApplicable」；新代码：`research/prompts/m3-formal-code-b26-r1-sonnet-model/out/probe-i54-new.txt:2`「PROBE_I54 root_50249 zeroed=Holds」；叶那一份同（两文件第 5 行）。别的不变量照红：`zeroed_violated=["I-2.1", "I-4.8", "I-7.2", "I-7.4"]`（两文件第 3、6 行）。
含义：I-5.4 报「成立」不再保证互不相交真被判过。层 0 按判定种类钉的值不受影响（J1-5）；会受影响的是任何把 I-5.4 的成立数当「判过互不相交」的读法。改法候选（不替主 agent定）：新判只在根记录那一项非零时才算评估，或树表非空而没有一棵分配记录树被走到时补 `part_of_the_range_not_judged`。
推翻它的现象：用判决原意读「带树一版分配记录树根全零」本来就是一条独立的小判定、与互不相交无关，那 Holds 不算串味；条款归属定下来之前我只报现象。

### J1-4　字节区间字段表的归类与区间：**一致**，三处判决没列的补充只记不判

- 区间无洞无重叠：三张表各自首尾相接盖满总宽（根记录 457、系统配置 489 且四档 389、4、36、60、树表条目 200）。26-3 自己有一条轻用例钉着（`crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs:1628`，断言四档「[389, 4, 36, 60],」在第 1664 行、「系统配置字段表 46 行，一行一段」在第 1670 行），我复跑过（47 过）；我另按 kb 的偏移表读了一遍：「journal tail 469 / journal 实例代号 477 / 回退下界 F 481，合计 489」(`.claude/kb/layout/01-first-txn.md:158`)，根记录「15 个字段合计 457 字节」(`.claude/kb/decisions/22-单元原子性怎么合成.md:16`)，表里偏移逐个对得上（根记录 13 段里算法、nonce、MAC 合一段，15 个字段）。
- 归类对判决 Y1、Y2 逐行：magic、flags 其余位、自证校验和、整槽校验和不比（「D13_STRUCTURAL_FIELDS」`crates/singlefs-harness/src/mount_transparency.rs:986`）；实例表与树表单元指针不比字节；「must_be_the_same("节点大小", 301..305, D13_MUTABLE_CONFIGURATION),」(`crates/singlefs-harness/src/mount_transparency.rs:1222`)；运行配置 36 字节 R2 = 内置默认；不可变档除本盘设备号按原字节必须相同；算法、nonce、MAC 归 I-2.4、挂载透明不比（`crates/singlefs-harness/src/mount_transparency.rs:1147` 那一段写明）；槽世代号交 I-7.16。一致。
- 判决没列、实现加的：树表条目的条目长度、flags、`previous_snapshot_txg`（第 1318 行起）按必须相同，预留 124..200 必须为 0（第 1327 行）。与 core 解条目时「预留 76 非零」拒收同向，且 47 个用例过、7 种底盘面上整挂载零红，我记「规则没说，不冲突」。
- 判官按表比的是 R1 与 R2 读出来的原字节；`flipping_one_byte_of_each_compared_range_…` 对每个比的段翻一字节重封校验和、判官红在那一行：这一条变异行（3686、3685）我打了、红。

### J1-5　I-5.4 挂法与「部分没判」的报法对层 0 按判定种类钉的值、崩溃注入的计数：**钉值不变（推的）；一处计数前提要留意**

- 层 0 里按判定种类逐条钉评估数的只有新池新建文件那一条流（`crash_enumeration_new_pool_file_creation_stream.rs`，26-2 #8 钉的 69 条）；别的四份流只钉「评估的 + 不适用的 = 状态数」与「至少一个状态评估过」（`crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs` 等，我读过四份）。
- 「部分没判」与旧的「部分射程判不了」落同一个桶：「InvariantVerdict::NotApplicable(_) | InvariantVerdict::PartiallyJudged(_) => {」(`crates/singlefs-checker-tier/src/crash.rs:1493`)，旁注「计进不适用这一格」(`crates/singlefs-checker-tier/src/crash.rs:1491`)。所以 I-5.5 把几格由 `part_of_the_range_not_judged` 改 `partially_judged`，层 0 的评估数不变。「某盘没有数据单元」由不适用变成跳过后，那一盘不再拖垮整条，新池流上两块盘都有数据单元（单元与记录先于根持久），评估数仍是根槽 txg 3 落了的 9 个状态：日志里「I-5.4=9/0」(`research/prompts/m3-formal-code-b26-r1-sonnet-model/in/checker-line-410.txt:1`) 与「I-5.5=9/0」同一行。I-2.4 的新判逐条根、每个状态都有种子根；I-9.16 的 ③ ④ 只在树表有条目时加判：二者的评估数都还是 46。
- 崩溃注入：`invariant_partially_judged` 是个只增不钉的计数（`crates/singlefs-checker-tier/src/crash_injection.rs:3818` 一带）。要留意的是 26-1r 让注入过读故障的崩溃态不判挂载透明，而旧断言「每个挂载做成的崩溃状态都判了只挂载那一个」(`crates/singlefs-checker-tier/src/crash_injection.rs:5753`) 假定没有这样的态：它用的历史是默认比重种子 3 的 10 步、一段 unlink、两段固定历史，崩溃恢复抛弃根那一步在今天的历史上从不发写（C696 自己的数：「那一步出现 122 次、发了写的 0 次」(`.claude/kb/checks-owed.md:551`)），所以这条断言仍成立；换成含新步的比重就会红。checker 档没跑，推的。
## 二、J2：26-1r（C664、C696 丁、F3-e）

### J2-1　拒因分开之后两条拒因各自的合法判法

条款：D13 已定项 1「挂载透明的射程」写的是「拒绝合法 ⇔ 拒因在 D23（journal 的角色与格式） 已定项 14 那张单子上，且照那一条的判据在崩溃镜像上自己复算成立，且盘上逐字节不变，且只读读出在允许集合里」，又写「带故障的状态不在射程」(`.claude/kb/decisions/13-验证路线.md:57`)；C664 的还清条件是「带读故障的档里造一个取号那一刻瞬时读错的态，合成一个拒因时判不合法、分开后判合法」(`.claude/kb/checks-owed.md:522`)。

| 拒因 | 崩溃镜像上复算判据 | 判定 |
|---|---|---|
| 判据 N-配置（`NewerPublishWitnessedBySystemConfiguration`） | 判不出（R2）⇒ 真；否则全部自证过的槽 tail 取最大再比：「if system_configuration_witness_undecidable_on_the_crash_image(&slots_of_every_device) {」(`crates/singlefs-harness/src/mount_transparency.rs:2385`)，去掉了原来并在一起的「某块盘一份自证过的都没有」那一支。实现读阶段的判据同一个形：「WitnessedCounterComparison::Undecidable」(`crates/singlefs-core/src/mount.rs:447`) 加「读错且没有一块盘两槽都自证过」两支为真，删掉的成员 `SomeDeviceWithoutAnySelfVerifiedSystemConfigurationSlot` 本来就只由读阶段之后那一处报 | 一致。是收严：某块盘两槽都撕裂而数值比较为假的镜像，拿 N-配置 当理由拒从合法变不合法（`mount_transparency_judges_the_system_configuration_witness_by_r2.rs` 第二条断言；变异行 6090、6108 红） |
| 新拒因（`SystemConfigurationUnreadableAfterTheReadStage`） | 判不出（R2）或某块盘一份自证过的都没有 ⇒ 真，择不到系统配置也按真：「fn system_configuration_unreadable_after_the_read_stage_on_the_crash_image(」(`crates/singlefs-harness/src/mount_transparency.rs:2333`)，「let Ok(system_configuration) = choose_system_configuration(crash_image) else {」(`crates/singlefs-harness/src/mount_transparency.rs:2336`) | 没注入读故障的态：一致。第二截的可写挂载在没有读故障的普通盘上起（「let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> =」`crates/singlefs-checker-tier/src/crash_injection.rs:2052` 一带都是普通 `SparseBlockDevice`），设备不报读错，这拒因今天只会因「一块盘两槽都撕裂」出现，判据复算为真；变异行 6087、6089 红 |

**C3　注入过读故障的态上，新拒因的判据复算被整条放掉：条款射程外，但放过「无故拒」——冲突（并排）**

| 条款 | 实现 |
|---|---|
| 「带故障的状态不在射程」(`.claude/kb/decisions/13-验证路线.md:57`)；合法的四个条件里第二个是「照那一条的判据在崩溃镜像上自己复算成立」(同一行) | 注入态 + 新拒因：「*condition != RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage」(`crates/singlefs-harness/src/mount_transparency.rs:2626`) 把「复算为假」那一条件整条滤掉，只剩盘上不变与只读读出两条 |

实现的理由是「两遍读之间的瞬时读错在崩溃镜像上复算不出」。但被判的那次挂载是第二截在崩溃镜像上起的，没有读故障（上表第二行的 `crash_injection.rs:2052`；`DeviceReadingZerosOverHiddenRanges` 只在历史自己的那一步里用）。所以第二截这拒因只有一种合法来源（某块盘一份自证过的都没有），那时判据复算为真，**滤掉复算只会放过一种拒：盘面健康、却以这个理由拒的**。26-1r 自己的用例第一条断言就是它：健康盘面（两槽都自证过）+ 注入过读故障 ⇒ 「注入过读故障：读阶段之后系统配置又读不出那一拒合法（瞬时读错在崩溃镜像上复算不出）」(`crates/singlefs-harness/tests/writable_mount_refusal_after_the_read_stage_on_a_read_faulted_crash_state_is_judged_apart_from_the_witness_criterion.rs:143`)；同一盘面没注入则判不合法（第三条断言）。
判定：对条款是「一致」（带故障不在射程，判官还多判了盘上不变与只读读出两条）；对「该判不合法的有没有被放过」是**有一格放过**——注入态镜像上无故以这个理由拒可写。历史那一步自己的挂载若被无故拒，模型对拍会兜住（新成员映射成 `Unexplained`，变异行 6085 红）；第二截在崩溃镜像上起的挂载没有模型兜底，放过的就是这一处。
推翻它的现象：第二截或第三截的挂载其实也带着历史那一步的读故障起（我读 `crash_injection.rs:2052-2074` 没有）；或有一份注入态镜像上以这个理由拒、判据复算为假、而实现拒得对。checker 档没跑，这条是读码加用例自己的断言。

### J2-2　N-配置 去掉「有盘一份都没有就按真」那一支：**一致**

方向是收严不是放宽：旧判法对「某块盘两槽都撕裂」按真、所以拿 N-配置 当理由拒是合法，与实现读阶段不一致（实现那一遍只在判不出时按真）；新判法两边同形。该判不合法的没放过：同一份盘面上 N-配置 的拒判不合法，那一条用例所在的文件复跑 2 过（`research/prompts/m3-formal-code-b26-r1-sonnet-model/out/harness-j2-unmutated.txt:8`「test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s」），变异 6090（复算仍按真）红。

### J2-3　新步与新比重会不会改已有比重的取样（指纹钉住了七组）：**一致，六组独立复现，orphans 复核不了**

- 新步只进它自己那一张比重表：「HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros」出现在 `GenerationWeights::NEWEST_SYSTEM_CONFIGURATION_SLOT_OF_EACH_DEVICE_READING_ZEROS` 里（`crates/singlefs-harness/src/history.rs:975` 起），已有七组的表里没有；`HistoryOperationKind::ALL` 由 11 变 12 只用在计数与报告次序（`history.rs:2628`、`fault_injection.rs:1193`、`crash_injection.rs:842`、`random_histories.rs:185`），不进抽样。
- 钉值我拿 git HEAD（31e47115）的 `crates/` 编（那一版没有新步），把指纹用例去掉 orphans 一组后跑：`research/prompts/m3-formal-code-b26-r1-sonnet-model/out/fingerprint-at-head.txt:4`「test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s」，broad、reuse、rollback、wall、unit-area-wall、several-leaves 六个钉值与加新步之前逐位相同。orphans 一组（钉 10448260464455344558，`crates/singlefs-harness/tests/random_history_generation_keeps_the_existing_weights_step_for_step.rs:62`）在 HEAD 上没有那一组比重，26-1r 之前的那棵树没留：**复核不了**，只有读码（表里没有新步）与「变异 6091（把新步按崩溃恢复抛弃根的份数放进快档那一组）红」两样。
- 新步的前提「每块盘两槽系统配置都自证过」不满足就记「前提不满足没调」，不当作单故障之外的双故障去挂：`SomeDeviceWithoutTwoSelfVerifiedSystemConfigurationSlots`；慢档用例 48 段 24 步全绿：「单故障在承诺里，这一步不许被拒：{step:?}\n{rendered}」(`crates/singlefs-harness/tests/random_histories.rs:491`)，我复跑 1 过（`research/prompts/m3-formal-code-b26-r1-sonnet-model/out/harness-j2-unmutated.txt:11`；第 9 行那条第一次被我的 238 秒限时截断、没有结果行，第 10、11 行是单跑那一条）；变异 6092（藏成两槽都读回全 0）红。

### J2-4　二次崩溃镜像在注入态下照旧判那一处会不会误红：**没找到误红（推的）**

范围：26-1r 只对第一截的挂载透明按「注入过读故障」整个不判（`crates/singlefs-checker-tier/src/crash_injection.rs:1586` 的 `NoneInjected` 臂 / `:1598` 的 `Injected` 臂），第三截的二次崩溃镜像照旧判，并把第一次崩溃那个态的注入标志原样带过去（`:1564`「read_faults: read_faults_in_the_crash_state,」，第三截用第 1633 行同一个标志）。
读码：注入态的第一次崩溃镜像上，历史那一步的取号写把系统配置写进被藏的那一槽，带的是见证值（另一槽的 tail），所以 tail 不高于被盖掉的槽；之后第二截的挂载（无故障）的见证值取全池自证过的槽最大 tail，R1（落到的那一槽）≤ R2 仍成立，环里记录不少于 tail，实例代号 ∈ {R1 的, 新实例}：两行关系的三个量都只从镜像读，不受「第一次崩溃那一步带没带故障」影响。我的探针（J1-2(3)）量的是不带故障的七种底盘面，不含注入态；注入态上的第三截要 checker 档，我没跑。
推翻它的现象：注入态镜像上二次崩溃的 R2 择到的系统配置槽 tail 低于 R1 择到的（意味着见证值回退不止一次）；或 R2 的 journal 实例代号既不是 R1 的也不是新实例。

### J2-5　F3-e 三条取样在更长历史、更多步数上罩不罩得住：**三条各罩住它点名的那一行变异；联合与段长两面没罩到（C4）**

做的对判决：判决写的 A2 是「段号 0..=63 再加 2^k、2^k + 1（k = 6..=20）共 384930 个种子两两不同」，B3 是「长 32 的段扣下 1 或 2 个写的 528 个子集」，C2 是「历史种子 0、1、2^63」(`research/prompts/m3-formal-code-b24-r1-main-verification.md:31`)。代码：「let segment_indexes: Vec<usize> = (0..=63)」(`crates/singlefs-checker-tier/src/crash_injection.rs:4835`) 后接 `1 << exponent` 与 `+ 1`；「const WRITES_OF_THE_SEGMENT: usize = 32;」(`crates/singlefs-checker-tier/src/crash_injection.rs:4868`)；「const HISTORY_SEEDS: [HistorySeed; 3] =」(`crates/singlefs-checker-tier/src/crash_injection.rs:4911`)。我把 `fingerprint_of_the_persisted_subset` 与 `seed_of_the_second_crash_points`（`:3062`、`:3077`）逐位复写成 Python（`tools/f3e_model.py`），三条取样的规模对上判决的数：384930、528、12285（`research/prompts/m3-formal-code-b26-r1-sonnet-model/out/f3e-model.txt:1`），每条取样自己的种子集合在没改坏时 0 撞（第 3 行）。
模型里对三行变异：M7a 只红 B3（449 撞，第 4 行）、M7d 只红 A2（122850 撞，第 9 行）、M7f 只红 C2（4095 撞，第 12 行）：三条各罩着它点名的那一行，与变异行（6098–6100）一一对上。checker 档的那三条用例我没跑，这是 Python 模型上的数。

**C4　四条取样（旧 F3-d 加 F3-e 三条）漏的形：**
- 段长 > 32：「只折段末 32 个写」「只折前 32 个写」「只折前 36 个写」四条取样里 0 撞、段长 40 的加宽取样里 37058、37058、22086 撞（`research/prompts/m3-formal-code-b26-r1-sonnet-model/out/f3e-model.txt:6`、`:7`、`:8`）。B3 的段长正好 32，对折窗口 ≥ 32 的改坏全绿。判决第三节自己写的是「它自己的取样（段号 2^k、段长 32、历史种子 2^63）对更长的历史、更多步数、别的种子基罩不罩得住没人攻过」(`research/prompts/m3-formal-code-b24-r1-main-verification.md:39`)；b24 量到的真历史「4 GiB 档段长到 30」(同文件第 29 行)，所以在已量过的档里窗口 ≥ 32 是等价变异，换更宽的档（段更长）才不是。
- 历史种子与段号用异或合并：「历史种子、盐、第一次崩溃的段号与它段内子集的指纹（[`fingerprint_of_the_persisted_subset`]）四样异或。」(`crates/singlefs-checker-tier/src/crash_injection.rs:3076`)，所以同一子集在 (历史种子 h, 段号 s) 与 (h', s') 上只要 h ^ s = h' ^ s' 就拿到同一个种子：(0, 17) 与 (1, 16) 的 4095 个第一次崩溃态一一撞（`research/prompts/m3-formal-code-b26-r1-sonnet-model/out/f3e-model.txt:15`「(0,17) 与 (1,16) 的种子相同 = True」；三条取样与旧 F3-d 各自一个集合，每条只变一维，这一类对一个都不进）。这个指纹「只用来岔开二次崩溃的随机源」(`crates/singlefs-checker-tier/src/crash_injection.rs:3060`)，撞车的后果是不同历史里同形的子集摆同一串二次崩溃——少了点覆盖多样性，不是误判。
- 平凡的漏网：段号只取低 21 位、历史种子去掉第 62 位（`research/prompts/m3-formal-code-b26-r1-sonnet-model/out/f3e-model.txt:11`、`:14`）三条取样都不红，真历史里段号到 235、历史种子基起连号，不会碰到。
推翻 C4 的现象：checker 档里这三条用例换成含「h ^ s」联合与段长 40 的取样仍全绿且指纹窗口改坏仍红——那是实现比模型多了一层混合，我读不到（`SeededRandomSource::from_seed` 没读）。

### J2-6　替换的 5 行与删掉的 RR8 还各罩着原来那个行为：**一致**

- 替换 5 行：旧串在冻结副本里各恰好命中 1 次（`tools/check_anchors.py` 对 `mutations-replacements` 五行，md 行 6106–6110，均 `hits=1`），点名的测试今天对改后的代码都是红：C554 乙（6106）、C579（6107）、拒绝合法 11-4（6108）、挂载透明两棵树根指针（6109）、I-5.5 落点恒成立（6110）全红，`research/prompts/m3-formal-code-b26-r1-sonnet-model/out/mutation-rows-final.tsv` 第 35 至 39 行。6109 的新写法 `true || …` 比旧行（「只看 R1 读不读得出」）多关掉「恒全零四种相同」那一半，仍红在 `an_inode_tree_root_pointer_that_moved_with_the_mount_is_red`。
- 删 RR8（「harness 判法复算判据 N-配置 不认某块盘一份自证过的系统配置都没有」）：它判的行为——「某块盘一份都没有」算拒可写的合法条件——挪进新拒因的判据，由 6089（那一拒的判据不认）红在同一个用例的第一条断言守着；反方向（N-配置 复算仍把它按真）由 6090 守着。RR8 原来的那一支今天在代码里已不存在，没有悬空。

## 三、J3：26-2 的 8 条钉值与 C686

出处是 2026-10-04 主工作区 checker 档日志（会话草稿里的 `checker-tier-integration-early.log`、`head-tree-tests.log`）；我把用到的原行摘进 `research/prompts/m3-formal-code-b26-r1-sonnet-model/in/`，下面引那份摘录（行号是摘录文件的行号；`early-log-assertions.txt` 每行前的数字是原日志行号）。日志是合入 10b 之前的树，这一点放在最后一条里说。

| 钉 | 出处站不站得住（我核了什么） | 判定 |
|---|---|---|
| #1 `crash_identity.rs` 补登记两行 | HEAD 日志点名恰是这两份未登记的库模块：`research/prompts/m3-formal-code-b26-r1-sonnet-model/in/head-log-lines.txt:2`「crates/singlefs-harness/src/fault_injection.rs」「crates/singlefs-harness/src/reference_filesystem_translation.rs」。补的两行：「library_module: "crates/singlefs-harness/src/reference_filesystem_translation.rs",」(`crates/singlefs-checker-tier/src/crash_identity.rs:903`)、「library_module: "crates/singlefs-harness/src/fault_injection.rs",」(`crates/singlefs-checker-tier/src/crash_identity.rs:908`)。我在冻结副本里 grep：引 `history` 的 harness 与 checker-tier 源文件恰是这四份已登记者（`reference_filesystem_translation.rs`、`fault_injection.rs`、`model_comparison.rs`、`crash_injection.rs`）；10b 新模块 `writable_mount_phases.rs` 只在注释里提到 `singlefs_harness::history`，没有引入，不用登记 | 一致 |
| #2–4 轮换用例按盘放行 mkfs 清零 | 三条都挂在同一个判法函数上（`a_publish_returns_only_after_the_system_configuration_rotation_is_durable.rs` 第 345、384、534 行同一句 `witnesses_behind_returned_publishes(…)`），HEAD 日志三条红在同一句话（`research/prompts/m3-formal-code-b26-r1-sonnet-model/in/early-log-assertions.txt:11`、`:17`、`:22`「整段清零落在系统配置两槽上」）。口径的前提是 mkfs 每块盘先清零再写系统配置槽：「写任何元数据之前，先把每块盘的 [0, 根环基址) 整段写 0」(`.claude/kb/decisions/22-单元原子性怎么合成.md:176`)；代码「device.write_zeroes_at(DeviceOffsetInBytes(0), root_ring_base.0)?;」(`crates/singlefs-core/src/make_filesystem.rs:557`) 在前、「device.write_at(offset, &slot, WriteDurability::Plain)?;」(`crates/singlefs-core/src/make_filesystem.rs:655`) 在后。放行条件「|| !devices_with_a_system_configuration_slot_write」(`crates/singlefs-checker-tier/tests/a_publish_returns_only_after_the_system_configuration_rotation_is_durable.rs:155`) 只对「先清零后写」放行，「先写槽后清零」仍停下。口径只在流从空白镜像起时成立：判法的起点是「MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);」(同文件第 141 行)，三条用例的流都从 mkfs 起。变异行 C577（mkfs 写槽后又清零再重写）点名 checker 档，我没跑：读码它会停在放行条件上 | 一致（变异没跑） |
| #6 门槛 10 → 8 | 「15 个签名，最多的 8 个状态」(`research/prompts/m3-formal-code-b26-r1-sonnet-model/in/head-log-lines.txt:4`)；新门槛「rejudged.red_states_by_signature.len() >= 4 && largest_signature_states >= 8,」(`crates/singlefs-checker-tier/tests/crash_enumeration_findings_log_dedupes_by_signature_and_survives_resume.rs:587`)。新加的「跨台阶行恰好每个跨过 10 的签名一行」读的前缀是写日志真写的那一个：「.filter(|line| line.starts_with("layer0_finding_threshold\t"))」(同文件第 603 行)，写的一侧 `crates/singlefs-checker-tier/src/layer0_progress.rs:1963` 同名，不是空断言；「跨过就报一次」那一面留给库单测 `absorbing_a_slice_reports_new_signatures_by_their_first_state_and_every_threshold_crossed`（`crash.rs:4560`，我读了，checker 档没跑）。8 是观测最大值，这条断言从此是「别掉到 8 以下」的钉 | 一致 |
| #7 段序列 `[…,24,5]`、`<= 5`、计数 1 → 9 | 段序列：「left: [2, 2, 1, 2, 2, 1, 2, 24, 5]」(`research/prompts/m3-formal-code-b26-r1-sonnet-model/in/early-log-assertions.txt:45`)，旧值是第 46 行那条 right；选择器「&|_segment_index, segment| segment.len() <= 5,」(`crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs:847`)：不放宽到 5，新出的 5 写段（记录 2、根槽 1、轮换 2）只以整段持久进来，「根在案而记录不在」一个摆不出，所以是必须的而不是放松；别的短段本来就 ≤ 3，展开不变。计数 9：判据是「&& in_place(publish.root)」(`crates/singlefs-checker-tier/src/crash.rs:143`) 加「&& !publish.records.iter().any(|record| in_place(*record))」(`crates/singlefs-checker-tier/src/crash.rs:144`)，命中即「check.root_without_record = true;」(`crates/singlefs-checker-tier/src/crash.rs:146`)，5 写段里根落、两份记录没落的态 = 两次轮换写（原地覆写，各取没落 / 撕开 / 落了三态）3 × 3 = 9，对上钉值「tally.record_root_without_record, 9,」(`crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs:853`)。9 是读码算的，checker 档没跑 | 一致（9 是推的） |
| #8 69 条评估数期望值 | 日志 CHECKER 行 69 条（`research/prompts/m3-formal-code-b26-r1-sonnet-model/in/checker-line-410.txt:1`）。我写了 `tools/check_pin8.py`：按用例里九张分类表与六个常量算每条的期望，对日志逐条比，`mismatches 0`，分类 6、22、1、2、1、1、1、1、1、33 条合 69。常量的算术另核：21 = 8 + 3 + 1 + 8 + 1，34 = 46 − 9 − 3，25 = 46 − 21，12 = 9 + 3，45 = 46 − 1；`GOLDEN_TALLY` 里「verification_ran=6」(`research/prompts/m3-formal-code-b26-r1-sonnet-model/in/early-log-assertions.txt:85`) 与「STATES_WHERE_THE_REPLAY_APPLIES_A_RECORD: u64 = 6;」(`crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs:511`) 对上 | 一致（对的是 10b 之前那棵树的日志，见 #9 末句） |
| #9 打印行与计划哈希，进度文件哈希待重录 | 打印的 58 行：「left: (58, "7e64a883680d9d609e033fd228790db29e8c32b3beb3798b920712f749d70ad4")」(`research/prompts/m3-formal-code-b26-r1-sonnet-model/in/early-log-assertions.txt:87`)；我把原日志那 58 行单独摘成 `research/prompts/m3-formal-code-b26-r1-sonnet-model/in/printed58-golden-lines.txt`，`sha256sum` 得 7e64a883…（与钉值 `crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs:845` 逐字相同）。计划哈希 0370ad26… 取自日志 `LAYER0_RESUME` 行：「fingerprint0-0370ad26cad051266c57d7129a63840319034b488aefec9adebf007a9809b91e.txt」(`research/prompts/m3-formal-code-b26-r1-sonnet-model/in/resume-line-480.txt:1`)，与钉值 `GOLDEN_PROGRESS_FILE_NAME` 同。**进度文件内容 SHA-256 没有出处**：「const GOLDEN_PROGRESS_FILE_SHA256: &str = "待重录：进度文件内容的 SHA-256，取法见上";」(`crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs:851`)，这一核今天必红，等根环线重跑填 | 打印行与计划哈希一致；进度文件哈希是明说的待办（规则没说，不冲突） |
| C686 用例 | 断言落点、层级、出生序号逐行等于 D3 已定项 10 ⑤ 那五行：「| t5 | 分配记录树叶，设备 0（码 2） | 65538 | 50245 | 0 |」(`.claude/kb/decisions/03-空间分配.md:216`) 到「| t9 | 分配记录树根，层级 2（码 2） | 65538 | 50249 | 4 |」(`.claude/kb/decisions/03-空间分配.md:220`)，用例表「("t9 分配记录树根，层级 2", 50249, 2, 4, None),」(`crates/singlefs-harness/tests/new_pool_file_creation_publish.rs:1270`) 同；出生序号的发号次序由条款「这个次序同时就是出生序号的发号次序」写明 (`.claude/kb/decisions/03-空间分配.md:207`)。条款没直接写、用例加的两样：两块盘上的两份逐字节相同（「{row}：两块盘上的两份逐字节相同」`crates/singlefs-harness/tests/new_pool_file_creation_publish.rs:1277`）、叶里的记录全属表里写的那块盘（第 1302 行）。前者有产物坐实：E142 干跑里每个节点两块盘各一行、`sha256` 相同、出生序号一致，例如「step=16 writer=transaction unit=allocation_root … slot=50249 … birth_sequence=4」(`research/results/e142-new-pool-file-creation-dry-run-2026-09-25-r16-combined.out:353`)；后者是 ⑤ 表「设备 0 / 设备 1」两个字的读法。变异行 470（同层按盘号倒序 bump）我打了：红在第二样断言上（「left: [DeviceIdentity(1)]」`research/prompts/m3-formal-code-b26-r1-sonnet-model/out/c686-mutation-red.txt:3`），不是红在出生序号那一样——对调两个出生序号的 C686 判别力条件，用例第一样断言按读码会红，我没单独造这一行 | 一致 |

**8 个钉值里最该留意的一条（复核不了）**：#8、#9 的值取自合入 10b、26-3、26-1r 之前那棵树上的日志。计划哈希的消息里有基线扇区、写表内容、段与展开、版本表（`crates/singlefs-checker-tier/src/crash.rs:2570` 起的 `layer0_plan_hash`），所以 10b 或 26-x 只要改了这条流里 mkfs、挂载、发布写出的任何字节，两个钉值都会变；这要 checker 档重跑才能判，根环线点名重跑的那 8 条正是这件事，结果我没拿到。
## 四、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| J1-1 代码做的是不是判决说的 | 一致 | 七处改动都对得上；点名 harness 档的 38 条变异行我逐条打、全红（26-3 的 23 行里主 agent 只证过 2 行） |
| J1-2(1) I-9.16 ③ ④ 在合法历史上误红 | 一致（没见误红） | ③ 与 core 的 `tree_table_entries_each_kind_at_most_once` 同谓词；83 个坏镜像用例过 |
| J1-2(1) I-9.16 ④ 放过坏镜像 | **冲突 C1** | 判决「core 要的是种类 1–4」只对读路径成立；可写路径十一种各要一条，缺种类 8 的镜像 checker 零红、core 拒 |
| J1-2(2) 根槽 428..457 归 I-2.4 | 一致 | 三条坏镜像红在 I-2.4；GPU 没改（26-5） |
| J1-2(3) journal_tail 不等式与系统配置 journal 实例代号 | 一致（推的） | 7 种底盘面整挂载 13 场景、341 个二次崩溃前缀态零红；R1 ≤ R2 与取等两半无红用例、无变异行 |
| J1-2(4) `mapping_root` 与分配记录树根的空性关系 | 一致 | 两种实例号底盘面都判绿，两行变异红 |
| J1-2(5) 运行配置 R2 = 内置默认 | 一致 | 一行变异红 |
| J1-2(6) 回落看门 | 一致 | 10 个用例过，阳性对照红 |
| J1-3 C2 I-5.4 挂法 | **冲突 C2** | 分配记录树读不出：旧 NotApplicable、新 Holds（探针原样输出）；条款归属待主 agent 定 |
| J1-4 字节区间字段表归类与区间 | 一致 | 三张表无洞无重叠，归类对 Y1、Y2；多出的三段（条目长度、flags 等、预留为 0）规则没说、不冲突 |
| J1-5 I-5.4 挂法与部分没判对层 0 钉值、崩溃注入计数 | 一致（推的） | 部分没判与旧「部分射程判不了」同桶；只有新池流按种类钉，评估数不变；崩溃注入那条「判了的数 = 做成的数」只对不含新步的历史成立 |
| J2-1 N-配置 复算 | 一致 | 收严，去掉的那一支本来与实现读阶段不一致 |
| J2-1 新拒因（没注入读故障） | 一致 | 判据只会因「一块盘两槽都撕裂」为真，复算同 |
| J2-1 新拒因（注入读故障） | **冲突 C3**（条款射程外，放过一格） | 判据复算整条滤掉，盘面健康也判合法；被判的挂载其实无故障 |
| J2-2 N-配置 去掉「有盘一份都没有按真」 | 一致 | 方向是收严；变异 6090 红 |
| J2-3 新步与新比重改已有比重取样 | 一致（六组独立复现；orphans 复核不了） | 六个钉值在 git HEAD 源码上原样复现；orphans 一组在 HEAD 上没有 |
| J2-4 二次崩溃镜像在注入态下照旧判 | 没找到误红（推的） | 三个量只从镜像读；注入态上的第三截要 checker 档，没跑 |
| J2-5 F3-e 三条取样 | 一致；**C4** 漏形 | 三条各罩住自己点名的变异（模型上量）；段长 > 32 的窗口改坏与 (历史种子 ^ 段号) 联合撞车没罩到 |
| J2-6 替换 5 行与 RR8 | 一致 | 旧串各命中 1 次，5 行都红；RR8 的行为由 6089、6090 接着 |
| J3 #1 crash_identity 补登记 | 一致 | HEAD 日志点名的正是这两份 |
| J3 #2–4 轮换用例按盘放行 | 一致（变异没跑） | mkfs 先清零后写槽；口径只对从空白镜像起的流成立 |
| J3 #6 门槛 10 → 8 | 一致 | 日志「最多的 8 个状态」；新断言读的前缀是真写的那个 |
| J3 #7 段序列、`<= 5`、计数 9 | 一致（9 是推的） | 段序列取自日志；选择器放宽是必须的，9 = 3 × 3 |
| J3 #8 69 条期望值 | 一致 | 对日志逐条比 0 不符；取自合入 10b 之前的树 |
| J3 #9 打印行与计划哈希 | 一致；进度文件哈希待重录 | 58 行 SHA-256 我独立重算对上；进度文件那一核今天必红（明说待办） |
| J3 C686 用例 | 一致 | 落点、层级、出生序号逐行等于 ⑤ 表；两份逐字节相同有 E142 产物坐实；变异 470 红 |

## 五、没做什么

- 不判别的腿的格，不替主 agent 采纳、不出判决；C1–C4 与「放过一格」都只报现象、并排抄两边、写推翻条件，改不改、归哪一条由主 agent 定。
- **checker 档与整表一概没跑**：`singlefs-checker-tier` 的测试（包括 26-1r 新加的靶向用例与 `crash_injection_read_faulted_states_never_hold_the_witness_criterion.rs`、F3-e 三条、`crash_injection::tests` 下的各条）、`crates/mutations.tsv` 整表、GPU 判器。点名 checker 档的 9 行变异（改动全文行 469、6093 至 6100）没打。涉及它们的结论都标了「推的」或「复核不了」。
- 没跑：注入态镜像上的二次崩溃、段内子集与撕裂的原地覆写那一面的 journal_tail 关系（我只摆了写前缀）；orphans 一组指纹的加新步之前的值；#8、#9 在合入 10b 之后那棵树上的值（根环线点名重跑的 8 条）。
- 没验：运行配置「今天没有挂载选项通道」那一前提（判决 Y2 的前提）；GPU 判器上 ③ ④ 与 I-2.4 根槽的缺口（26-5）。
- 读了会话草稿里另一个会话的两份日志（`checker-tier-integration-early.log`、`head-tree-tests.log`，原路径写在 `in/` 摘录的第一行）：它们就是 26-2 那些钉值自己写的出处；禁读清单里的文件与别轮的腿报告一份没读。
- 没改仓里任何文件：写的只有这份报告、`research/prompts/m3-formal-code-b26-r1-sonnet-model/`（`tools/rerun.sh`、`tools/*.py`、`tools/fingerprint_at_head.sh`、`probe/`、`in/`、`out/`）与草稿目录。

## 六、要主 agent 跑的命令（我没跑的）

- 判 J3 #7、#8、#9 是否仍对（根环线点名重跑那 8 条的同一件事）：`SINGLEFS_HEAVY_TESTS=commit bash scripts/hosts/run-with-memory-cap.sh <上限> cargo test --release -p singlefs-checker-tier --test crash_enumeration_new_pool_file_creation_stream --test crash_enumeration_sharded_across_processes --test a_publish_returns_only_after_the_system_configuration_rotation_is_durable`，工作目录仓根；看 `CHECKER` 行是否仍是日志里那 69 条、`unsharded_enumeration_prints_and_writes_byte_for_byte_what_it_did_before_sharding` 左值是否仍是 `7e64a883…`。
- 判 J2-5 的取样与 C3 的第三截：`… cargo test --release -p singlefs-checker-tier --lib -- crash_injection::tests::first_crash_states`，另加 `--test crash_injection_read_faulted_states_never_hold_the_witness_criterion -- --include-ignored`。
- 复跑我的东西（harness 档，几分钟）：`FROZEN=/tmp/claude-1000/m3-formal-code-b26-r1/tree REPO=$PWD WORK=<草稿> bash research/prompts/m3-formal-code-b26-r1-sonnet-model/tools/rerun.sh all`；只看静态那几格：`… rerun.sh static`（几秒）。

## 七、cite-check 末行

`python3 scripts/agents/cite-check.py research/prompts/m3-formal-code-b26-r1-sonnet-output.md --root /tmp/claude-1000/m3-formal-code-b26-r1/tree --root . --background research/prompts/_m3-formal-code-b26-r1-background.md` 末行：

  ✓ 核了 86 处引文，对上 86 处，没判 0 处（逐处列在上面）
