# m3-batch33s-code-r1 正推腿报告（Sonnet，格 K1 与 K3，2026-10-07）

立场：正推。从条款（kb、第 32 批判决第三节四条改法、用户 2026-10-07 定的「当读不出」）与冻结副本今天的实现推「应该是什么」，再与被判对象逐格比。
冻结副本：`/home/fy5090/.local/share/singlefs/work/m3-batch33s-code-r1/tree/`（下文 `T/`）。我的工作副本与编译目录在 `/home/fy5090/.local/share/singlefs/work/m3-batch33s-code-r1-sonnet/`，交回前删（见末节）。
没读攻方腿的报告与模型；没读本轮判决（它此刻不存在）。引代码写冻结副本绝对路径加行号，引 kb 写 kb 文件自己的行号（行号用 `grep -nF` 现取）。

## 0. 复跑总表（命令与原样输出）

| # | 命令（工作目录） | 原样输出 |
|---|---|---|
| 0.1 | `sha256sum -c research/prompts/m3-batch33s-code-r1-snapshot/crates-sha256.txt \| grep -c ': OK$'`（`T/`） | `18` |
| 0.2 | `python3 research/scripts/crates-mutation-rows.py static-check <我的副本>`（退出码 0） | `CHECKED	3147	304	9` |
| 0.3 | harness 基线：`cargo test --release --offline -p singlefs-harness --test checker_known_bad_images -- --include-ignored`（经 `run-with-memory-cap.sh 8G`、`capped.sh 7`） | `test result: ok. 203 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 17.43s` |
| 0.4 | 同上，带 S4 全部新用例的过滤词（变异行 3102 那一串） | `test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 187 filtered out; finished in 86.45s` |
| 0.5 | 三个内核展开表达式数（我写的小程序 `kernel-sizes`，调 `crash_judge_dispatch::kernel_inlined_size`，另用 naga 校验器验一遍；经 `run-with-memory-cap.sh 4G`） | `KERNEL recovery expressions=320015 margin=79985 validate=ok` / `KERNEL checker-scan expressions=382259 margin=17741 validate=ok` / `KERNEL checker-walk expressions=337872 margin=62128 validate=ok` |
| 0.6 | 判法闭包收了哪几份（我写的小程序 `closure`，调 `crash_identity::judging_code_of_the_judges`） | 收了 52 份（`grep -c ^FILE`）、停在门外 3 份；含 `crash.rs`、`crash_judge_tables.rs`、`singlefs-checker/src/walk.rs` 与五份 `.wgsl`（均 `raw`）；不含 `crash_amplification.rs`、`crash_judge_dispatch.rs` |
| 0.7 | S2 守卫的 u32 仿真 `guard_model.py`（9918 个格：槽号取 0、2³²−1、2³²、2³⁶、2⁴⁶ 等边界 × 长度 16 / 32 KiB × 空段两端取边界） | `cases 9918 disagree-with-truth {'old': 1133, 'new': 0}`；`first old disagreement: (4294967295, 16384, 0, 0, None)` |
| 0.8 | 变异表对比 `cmp_mut.py`：合前（git 暂存区 `:crates/mutations.tsv`，3121 行）对冻结副本（3152 行） | S 的 19 行追加在 3102–3120；S 替换的 25 行名字不变、只改第 3、4 段（原文、替换文）、第 5、6 段（cargo 参数、必须红的测试名）逐字不变；别的 32 行追加与 5 处改动是根环线 10x，不判 |

0.5 的小程序、0.7 的仿真与 0.8 的脚本在 `research/prompts/m3-batch33s-code-r1-sonnet-model/`。

### 0.9 变异行逐条现跑（harness 档；在我的副本上套用、`cargo test --release --offline` 跑该行的 cargo 参数、还原）

S 追加的 19 行里 9 行是 harness 档（改 `walk.rs`），3102–3110；S 按名字整行替换的 25 行里 21 行是 harness 档。跑法：`mut_row3.sh <行号…>`（模型目录里，经 `run-with-memory-cap.sh 8G`、`capped.sh 7`）。判据：该行第 6 段点名的测试在这一趟里是 `FAILED`。

| 行（冻结副本 `crates/mutations.tsv`） | 点名测试红 | 同一趟共红几条 |
|---|---|---|
| 3102 诞生代号按第 0 条落点记 | 红 | 7 |
| 3103 折叠表不改记 | 红 | 6 |
| 3104 F-ID-b 回落 | 红 | 2 |
| 3105 G4 那一读（E1） | 红 | 9 |
| 3106 F-KEY 几处一起退 | 红 | 8 |
| 3107 F-KEY 只退数据单元（E2） | 红 | 1（只红点名那一条） |
| 3108 F-KEY 只退 inode 内部孩子（E4） | 红 | 1（只红点名那一条） |
| 3109 `read_index_node` 的「走过」按第 0 条 | 红 | 1（只红点名那一条） |
| 3110 「自证过」只按 16 KiB 比 | 红 | 3 |
| 257、950、2883–2885、2914–2917、2951–2956、3032、3038、3039、3064、3065、3069（共 21 行，S 替换的 harness 档行） | 21 行都红 | 每行 1–9 不等（日志在草稿目录 `logs/row-<行号>.log`，交回时随草稿目录保留的只有 `bin/`、`logs/`、`model/`） |

S 的 14 行 GPU / checker 档行（追加 3111–3120，替换 2515、2973、3070、3071）：见 §K1-S8，只能核到「套用之后仍编得过、WGSL 仍过 naga 校验」（3111 除外，它在只在 `verdict-store` 特性下编的模块里），红不红复核不了。

## K1-S1　allrbmap：一个单元的各把键按读回那一份的落点认（CPU `walk.rs`、GPU 走读内核）

**应该是什么（推）。** 判决第三节第 1 条写的改法：「照攻方 allrbmap，一个单元在走读里的各把键（走过、诞生代号、合并、父单元、叶序、I-3.5 折叠表）一律按读回那一份的落点认，CPU、GPU 同改」（`research/prompts/m3-batch32-code-r1-main-verification.md:40`）；I-4.9 那一行：「按落点认，读落点上单元头里的诞生代号，头校验和过」（`.claude/kb/invariants.md:185`）。
由此推：一个单元只有一个身份，就是 `read_referenced_unit_with_the_location_it_was_read_from` 交回的那条位置条目的 (设备, 槽)；六把键取同一个值；GPU 取同一组。合法崩溃状态上两份副本都对得上 ⇒ 读回第 0 条 ⇒ 读回落点 = 第 0 条落点 ⇒ 六把键与改前逐个相同；第 0 条那份对不上时，读者对每一份各判一次 I-2.1：`judgements.judge("I-2.1", matches, || {`（`T/crates/singlefs-checker/src/image.rs:1530`），GPU 同：`w_judge(INVARIANT_I_2_1, matches0);`，所以键翻面的那些状态 I-2.1 已红，合法状态判定集合不变。

**被判对象（今天的样子）与逐把键的比：**

| 键 | CPU（`T/crates/singlefs-checker/src/walk.rs`） | GPU（`T/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl`） | 同不同 |
|---|---|---|---|
| 身份来源 | `fn read_a_unit_to_walk_and_the_placement_it_was_read_from(`（1290）交回 `Some((unit, placement_it_was_read_from))`（1316） | `fn w_read_unit_to_walk(p: Ptr, unit_bytes: u32) -> Slice {`（1324）记 `w_unit_device_read_back` / `w_unit_slot_read_back` | 同 |
| 走过 | `if !self.visited_units.insert(placement_it_was_read_from) {`（1547、2690、2995、3121、3295、3498 六处） | `if (!w_visit(out.unit_device, out.unit_slot)) {`（1545）、`fn w_visit_the_copy_read_back() -> bool {`（1348）、实例表链 `let device_read_from = w_unit_device_read_back;`（1667） | 同 |
| 诞生代号表 | `.insert(placement_it_was_read_from, birth_txg);`（1063，签名多一个 `placement_it_was_read_from` 参数，1050） | `let visited_index = w_visited_index_in_the_main_walk(unit_device, unit_slot);`（1445） | 同 |
| 并进前一版 | 与「走过」同六处的 `merge_placements_below_a_unit_walked_by_an_earlier_version(`（1548 等） | `w_merge_below(out.unit_device, out.unit_slot);`（1547）、`w_merge_below(device_read_from, slot_read_from);`（1670） | 同 |
| 父单元 | `let previous_parent_unit = self.start_walking_below(node_placement);`（2044、2373、2773、3666、3755、3863）、`self.walk_parent_unit = Some(placement_it_was_read_from);`（3301） | `w_parent_device = read.unit_device;`（2022、2147、2180、2273、2294、2321、2512、3269、3354）、`w_parent_device = device_read_from;`（1674） | 同 |
| inode 叶序 | `.insert(placement_it_was_read_from, leaves_in_leaf_order.clone());`（3095） | `sws(row, leaf_device);`（2858）、`w_cached_leaves_below(leaf_device, leaf_slot, leaves_before);`（2784） | 同 |
| I-3.5 折叠表 | `if placement_it_was_read_from != first_placement`（1309）…`.insert(placement_it_was_read_from, placement_it_was_read_from);`（1314）：读回的是第 1 条、且那个落点此刻折在第 0 条落点那个单元上时，改记成它自己 | `sws(row + 3u, p.device1);`（1336）：同一条件、同一改记 | 同 |
| 实例表页诞生代号 | `self.birth_txg_of_walked_instance_table_pages.insert(`（3309，键换成读回落点） | `appearance_note_the_birth_txg_of_an_instance_table_page(device_read_from, slot_read_from, rd64(s.base, 73u))` | 同 |
| 「先走过」的认法 | `enum IndexNodeReading {`（303）三种结局，调用方不再在读之前另按第 0 条查 `visited_units` | `w_read_index_node` 交回 `already_walked`，调用方 `if (read.already_walked) {`（3364） | 同 |

漏改的检查：`walk.rs` 里仍写 `locations[0]` 的主走读位置只有三处：`let first_placement = (pointer.locations[0].device, pointer.locations[0].slot);`（1308，用来与读回落点比）、`note_references_of_a_pointer_followed_by_the_walk` 里记折叠表初值的 `let unit = (pointer.locations[0].device, pointer.locations[0].slot);`（1358，规定的初值）与探测 I-9.1 头的 2168；别的 `locations[0]`（4694 `read_index_node_without_judging`、4769 `references_of_root`、4984 与 5076 两个 `…_without_judging` 收集器、5957 与 6199 两个分配记录扫描、9191 `read_packed_unit_without_judging`、9415 与 10302 两个删除态扫描）在另走一遍的读者与扫描里，不是走读那六把键；它们仍按第 0 条落点认，这一批没改，也不在判决第三节第 1 条列的键里。GPU 一侧同理：`p.device0` 只剩登记引用（1253–1260）、读（1299、1326）与读不出时的默认值（1528–1529）。

**判定：一致。** CPU 与 GPU 键集合同一组；合法状态上判定集合不变（读码推，GPU 一侧复核不了）；第 25 批 H-b、第 28 批 H-V、第 30 批 F-W 那几条已有坏镜像结局不变（0.3：203 条全绿；0.9：它们的变异行都照旧红）。

两处观察（不是冲突，不进判定一览的红格）：
- O1-1　「两份都读不出、而第 0 条落点早被别的版本走过」这一形，改之前 CPU（读之前按第 0 条查 `visited_units`）与 GPU（`w_visited_contains`）都判成「别的根先走过」，今天读不出就是 `NotWalkable`（`IndexNodeReading::NotWalkable => return,`，walk.rs:1730；GPU `read.already_walked` 为假）。CPU、GPU 同改，只发生在两份校验和都对不上（I-2.1 已红）的坏镜像上；harness 里没有一条坏镜像钉这一形。推翻（即这一形要补钉）：造一张树表或多层码 2 节点的指针，第 0 条落点已被前一版走过、校验和改坏、第 1 条也读不出，CPU 红集合与改前不同。
- O1-2　折叠表改记之后，读回第 1 条那份时同一个物理单元在 I-3.5 里是两个「单元」键（第 0 条落点没有诞生代号）；`let Some(birth_txg) = birth_txg_of(unit) else {`（walk.rs:670）取不到就 `continue`，所以第 0 条那个键被跳过、读回那个被判。这是设计的结果（0.9 的 3103 行证它有用：去掉改记，6 条红）。

**什么现象会推翻「一致」：** ① 一个合法崩溃状态上第 0 条那份对不上而 I-2.1 不红；② 两份都对得上却读回第 1 条（读者不是取第一份对得上的）；③ GPU 对拍流 `the_gpu_judge_agrees_with_the_cpu_when_a_shared_leaf_container_born_after_the_rollback_target_reads_back_from_its_second_copy` 在卡上出现不一致（本轮没跑，复核不了）。

## K1-S2　P4：全零单元守卫与 F-α（`crash_judge_common.wgsl`、`crash_judge_checker.wgsl`）

**应该是什么（推）。** 判决第三节第 2 条：「守卫改按 `SLOT_SHIFT`、`first_slot + n` 改 64 位」（`research/prompts/m3-batch32-code-r1-main-verification.md:41`）。守卫的语义由函数头注释定：读的起点按槽对齐、读的几槽都在空段 [落点表第 9 字, 第 10 字) 里才交全零单元（`T/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:497-501`）。把它写成算术：接受 ⇔ 起点按槽对齐 ∧ 槽号 S < 2³² ∧ 第 9 字 ≤ S ∧ S + n ≤ 第 10 字（n = 读的槽数，16 KiB 为 1、32 KiB 为 2）。槽号装进 32 位 ⇔ 偏移高 32 位 < 2¹⁴（偏移 = 槽号 × 2¹⁴）。
F-α（比 32 KiB 时另按 32 KiB 读）要的是 CPU 稀疏读者的行为：读 32 KiB 就是读盘上连续 32 KiB。

**被判对象。**
- 守卫：`if (((offset.x & (NODE_BYTES - 1u)) | (offset.y >> SLOT_SHIFT)) != 0u || first_slot < location_word(before, 9u)`（common.wgsl:510）与 `|| first_slot > first_slot_past_the_gap || first_slot_past_the_gap - first_slot < slots_read) {`（common.wgsl:511）。对齐与「装进 32 位」按 `offset.y >> SLOT_SHIFT`；先判起点不越过空段终点，再拿差比，不做会回绕的加法。
- F-α：`fn w_copy_matches_its_location_checksum_at_the_length_its_header_names(device: u32, slot: vec2<u32>, checksum: u32) -> bool {`（checker.wgsl:3149）里头类标签没登记的一支：`|| slice_crc_matches(read_at(device, slot_to_bytes(slot), DATA_UNIT_BYTES), DATA_UNIT_BYTES, checksum);`（checker.wgsl:3159）；两个调用方改成传 (盘, 槽, 校验和)。CPU 对照：`copy_matches_its_location_checksum_at_the_length_its_header_names`（walk.rs:4623）读一次 16 KiB 取头类标签，没登记就 16 KiB 与 32 KiB 各读各比。

**逐格比。**
- 守卫的界：0.7 仿真（把 u32 的回绕逐位仿出来，和整数算术的真值比）：9918 格，新守卫与真值 0 处不同，改前守卫 1133 处不同（首例：槽 2³²−1、长度 16 KiB、空段 (0,0)，`first_slot + 1` 回绕成 0，改前接受）。边界取值：槽号 2³²−3 … 2³²+1、2³⁶ ± 1、2⁴⁶ ± 1、空段两端 0 与 2³²−1、长度 1 槽与 2 槽。对齐位非零两个版本都拒。
- 空段两端：`first_slot > first_slot_past_the_gap` 先拒，之后 `第 10 字 − first_slot` 不会下溢；空段为空（0, 0）时，S = 0 也被 `0 − 0 < slots_read` 拒；S = 第 10 字时差为 0，读 1 槽也拒（空段是半开区间）。
- 没写过的槽的 16 KiB / 32 KiB 两种读：版本由 `select(0u, 1u, first_slot >= location_word(before, 11u))`（common.wgsl:514）定，32 KiB 要两槽都在空段里，所以盘尾只剩一槽、或下一槽有落点的空槽，32 KiB 读交 `NONE`。CPU 的 32 KiB 读在盘尾失败（`None`），GPU `NONE` 同；下一槽有落点时 CPU 读到「16 KiB 零 + 下一槽字节」，校验和是 32 KiB 全零的 CRC 对不上，GPU 的 `no_slice()`（`c2 = 0`，`slice_crc_matches` 在 `(s.c2 & 4u) == 0u` 时交 `false`，common.wgsl:606）同样对不上。攻方第 32 批那一形两边同判。
- 「别的读法有没有跟着变」：只有这一支的第二项变。起点是 32 KiB 落点：旧新都取那一版的 `c1`；起点是 16 KiB 落点：旧 `c2 & 4` 为 0、新 `NONE`，都 `false`；整段在空段里的两个空槽：旧新都取全零 32 KiB 单元的 `c1`；变的只有「没写过而下一槽有落点 / 在盘尾」，从误判对得上变成对不上，正是要改的。头类标签登记过的那一支、读不出的那一支没动。

**判定：一致。**（守卫与真值逐格相同；F-α 在攻方那一形上 CPU、GPU 同判；别的读法没变。守卫上方注释仍写旧界，主 agent 已知，不另报。）

一处没被这一批碰、也没被钉的既有边界（推的，GPU 没跑）：GPU 的 `read_at` 只读「起点是一个落点且不超过它的长度」或「整段在空段里」的区间（common.wgsl:448-496），CPU 稀疏读者读任意连续区间。位置条目的校验和若恰等于「16 KiB 节点加下一槽」或「没写过的槽加下一个已写槽前 16 KiB」那 32 KiB 的 CRC（要人为算出来填），CPU 判对得上、GPU 读不到判对不上。现有对拍流只造「32 KiB 全零的 CRC」与「16 KiB 全零的 CRC」，没造这两种。不是 S 引入的，是 Z3 之后仍留着的一个 CPU / GPU 读区间的差；要钉就是下一批的一条对拍流。
**什么现象会推翻「一致」：** 仿真之外，任一槽号、长度、空段端点上 GPU 守卫与「S < 2³² ∧ 第 9 字 ≤ S ∧ S + n ≤ 第 10 字」不同；或对拍流 `…names_the_last_slot_in_thirty_two_bits…`、`…past_thirty_two_bits…`、`…before_a_written_slot…` 在卡上不逐位相同（没跑）。

## K1-S3　P2：「自证过」合成一份、两份都没自证过当读不出（`walk.rs`、GPU 走读内核）

**应该是什么（推）。** 判决第三节第 3 条：「三份 CPU 抄写合成一份」（`research/prompts/m3-batch32-code-r1-main-verification.md:42`）；用户 2026-10-07 定、写进 kb：「两份都没自证过时当读不出」（`.claude/kb/invariants.md:311`，I-9.17 那行同句在 `.claude/kb/invariants.md:305`）。推：合成之后三处调用要么与合成前逐处同义，要么就是用户定的那一处；两份都没自证过时 CPU 当读不出（「孩子是码 2」那一小句不判）、GPU 同。

**被判对象与逐处比。**
- 合成函数：`fn class_of_a_copy_verifying_at_the_length_of_its_class(`（walk.rs:4570）：头读 16 KiB 取类标签 → 登记过的码才有长度 → 按那一类的长度读、整单元 CRC 对上才交那一类，别的交 `None`。
- 调用 1　`child_unit_class_other_than_the_entry_type_names`（4553，调用在 4559）：旧写法逐份读、第一份自证过且类 ≠ 条目类的记下；新写法 `find_map` + `filter(类 ≠ 条目类)`——取的都是「第一份自证过且类不同的」，同义。
- 调用 2　`some_copy_of_the_child_verifies_as_the_class_the_entry_type_names`（4605，调用在 4615）：旧写法按条目类的长度读、头类标签等于条目类且 CRC 对；新写法头类标签等于条目类 ⇒ 长度就是条目类的长度，读失败与 CRC 不对都得 `false`，同义；旧的 `expect` 换成函数头一句 `assert!`，没长度的类都同样 panic。
- 调用 3　`class_of_the_first_copy_verifying_at_the_length_of_its_class`（4588，调用在 2461）：这一处是用户定的改动——旧写法 `.or_else(read_unit_without_judging…)` 的回落去掉，两份都没自证过交 `None`，判在 `child_class.is_none_or(|class| class == MappingKeyUnitClass::IndexNode)`（2468）里当成立。
- GPU：`let class_of_the_child_of_a_type_zero_entry = w_class_of_the_first_copy_verifying_at_the_length_of_its_class(child);`（checker.wgsl:3243，`var` 改 `let`，回落那 6 行删掉）与 `let child_is_a_code_two_node_or_unreadable = class_of_the_child_of_a_type_zero_entry == NONE || class_of_the_child_of_a_type_zero_entry == 2u;`（3244）。函数本体（3118）两份各看一次，类标签落在 1–3 才有长度，按那一类的长度比 `c0` / `c1`（common.wgsl:606），与 CPU 的合成函数逐步同。
- 两份都没自证过：CPU `None` ⇒ `is_none_or` 真；GPU `NONE` ⇒ 「or_unreadable」真。两边同判；harness 的 16 种副本对（`C16`、`BAD`、`UNREAD`、`ZERO` 两两）墓碑、谱系各一条用例，身份那一格都不红；0.9 的 3104、3110 两行证它们会红。

**判定：一致。**（三处调用，两处与合成前同义，一处是用户定的改动；CPU、GPU 在两份都没自证过时同判。）
**什么现象会推翻：** 一张镜像上 CPU 的合成函数与 GPU 的函数给出不同的「自证过的类」（例如 GPU 对 16 KiB 落点按 32 KiB 比而 CPU 读到相邻槽字节，同上节末那一类人为校验和）；或合成前后某一处调用在合法镜像上结局不同（203 条 + 21 条老变异行照旧红，没有）。

## K1-S4　P3：补的坏镜像（`crates/singlefs-harness/tests/checker_known_bad_images.rs`）

**应该是什么（推）。** 判决第三节第 4 条：「G4 那一读、F-KEY 的 extent 与 inode 内部孩子、F-ID-b 回落、I-4.9 那一形各补一形坏镜像与变异」（`research/prompts/m3-batch32-code-r1-main-verification.md:43`）。推：每一点名要有一条坏镜像，它在对应变异下红、且红的就是它点名的那一句；E7 实现员停下（不判）。
**逐格比（用例行号是冻结副本 harness 文件里的 `fn` 行；变异行是冻结副本 `crates/mutations.tsv` 的行；红的数是 0.9 现跑的）：**

| 判决点名 | 新用例（行） | 对应变异行 → 红情况 |
|---|---|---|
| G4 那一读（E1） | `a_tombstone_container_copy_verifying_as_a_node_whose_mapping_entry_matches_it_reddens_the_checksum_invariant_on_either_device`（14647）：让中央映射那一读对得上，只剩往下走时那一读红 I-2.1，断言违例文字含「墓碑树内部条目的子指针」，两种盘序 | 3105 → 点名那条红；同趟共 9 条红（依赖 I-2.1 的 S 新用例与老用例） |
| F-KEY 数据单元（E2） | `a_data_unit_with_another_object_whose_location_names_the_walked_extent_root_reddens_its_five_tuple_on_either_device`（14733）：断言 I-1.1 红在「数据单元五元组」，两种盘序红集相同 | 3107 → 只红点名那一条 |
| F-KEY inode 内部孩子（E4） | `a_type_zero_inode_entry_whose_location_names_the_walked_extent_root_walks_the_copy_verifying_as_a_node_on_either_device`（14811） | 3108 → 只红点名那一条 |
| `read_index_node` 那一处（第 32 批实现员报告停下第 2 条） | `an_inode_leaf_with_a_non_zero_reserved_byte_below_a_root_pointer_naming_the_walked_tree_table_reddens_the_reserved_bytes_invariant_on_either_device`（14887） | 3109 → 只红点名那一条 |
| F-ID-b 回落（E6） | `assert_a_type_zero_entry_over_two_copies_that_do_not_verify_leaves_the_identity_unjudged_on_either_device`（14501）经墓碑 `…tombstone_entry_invariant_holding_on_either_device`（14555）、谱系 `…lineage_entry_invariant_holding_on_either_device`（14568）：四种自证不过的副本（`C16`、`BAD`、`UNREAD`、`ZERO`）两两 16 格，身份那一条都不红，互换两条位置条目红同一批（I-2.5 之外） | 3104 → 两条都红；3110（「自证过」只按 16 KiB 比）→ 3 条红 |
| I-4.9 那一形（allrbmap） | `assert_both_device_orders_judge_alike_with_the_unit_birth_and_appearance_rules_holding`（14097）经 6 条用例（墓碑撞 extent、谱系撞 inode / extent 各 bad / good）＋老用例 `a_lineage_container_with_a_wrong_record_width_reddens_the_packed_container_invariant_whichever_location_names_a_node_walked_before`（13560，第三个实参从「多红 I-4.9」改成 `&[]`）；分叉形两条 `a_shared_leaf_container_born_after_the_rollback_target_with_a_garbled_copy_reddens_the_birth_and_appearance_rules_on_either_device`（14360）、`a_leaf_container_directly_shared_and_born_after_the_rollback_target_with_a_garbled_copy_reddens_the_birth_and_appearance_rules_on_either_device`（14382） | 3102（诞生代号按第 0 条记）→ 点名那条红，共 7 条；3103（折叠表不改记）→ 点名那条红，共 6 条 |

6 对 today 臂换盘判得不同的形：墓碑撞 extent（bad / good）、谱系撞 inode（good，bad 是老用例）、谱系撞 extent（bad / good）——与判决第二节 P1 那一行的「6 对」同数。0.3、0.4 两趟全绿，0.9 的 9 + 21 条变异行各把点名测试改红；3107–3109 三行只红点名那一条，说明那三个落点各自被钉住。

**判定：一致。**（E7 不在判之列；E1、E2、E4、E6、I-4.9 那一形各一条，变异下红的包含点名那一条。）
**什么现象会推翻：** 把点名落点改回按第 0 条认，点名用例仍绿（0.9 三行已证会红）；或这些用例在没改的副本上红（0.3 全绿）。

## K1-S5　删除流进崩溃放量与 GPU 一侧

**应该是什么（推）。** D8 已定项 16 第 4 条：「从意图提交起只读挂载、冷走读、模型读回答「没有文件」」（`.claude/kb/decisions/08-核心索引结构.md:468`）。推：版本表带两样（带文件、文件从这一版起没有了），崩溃放量 CPU 判器、GPU 输入表、GPU 恢复内核对同一个状态要给同一个 oracle 结论；落到的根在表里：带文件的读回得是它的内容、没有文件的只许报没有文件；不在表里：报没有文件时看「比它旧的最新那一项」，有文件才违例，读出内容一律违例；一张不带「没有文件」的表上与改前一字不差。

**三处判法逐支比**（CPU：`T/crates/singlefs-checker-tier/src/crash.rs`；GPU：`T/crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl`）：

| 落到的根与结局 | CPU `classified_oracle_violation_for_the_version_table`（crash.rs:1096） | GPU `classify_oracle`（recovery.wgsl:84） | 同 |
|---|---|---|---|
| 没择到根 / 根槽已持久而落到旧根 / 走读失败 | 前三支，不看版本表 | 同序，`has_effective`、`newest` 之后、`OUTCOME_FAILED` | 同（这几支这一批没动） |
| 不在表里 + 没有文件 | `newest_older_version_if_it_has_a_file`（crash.rs:1046）：带文件的与没有文件的各取比它旧的最大 (txg, 实例)，带文件的更大才违例 | 一趟扫全表取比它旧的最大 (txg, 实例)，那一项第 5 字为「有文件」才报 `ORACLE_NO_FILE_WHERE_AN_OLDER_ROOT_HAS_ONE`（`if (newest_older != NONE && tab(T_EXPECTED, newest_older * EXPECTED_WORDS + 5u) == EXPECTED_VERSION_WITH_A_FILE) {`，120） | 同（两张表合起来取最大 = 分别取最大再比） |
| 在表里是「没有文件」+ 没有文件 | `(RecoveryOutcome::NoFile { .. }, Layer0VersionAtTheRoot::WithoutAFile) => None,`（1145） | `if (!version_has_a_file) {`（125）返回 `ORACLE_NONE` | 同 |
| 在表里有文件 + 没有文件 | `NoFileWhereThisRootHasOne` | 落到 125 之后的 `ORACLE_NO_FILE_WHERE_THIS_ROOT_HAS_ONE` | 同 |
| 不在表里或「没有文件」+ 读出内容 | `FileReadWhereThisRootHasNone` | `if (version == NONE \|\| !version_has_a_file) {`（130） | 同 |
| 有文件 + 读出内容 | 比内容 | `!recovery.content_matches` | 同 |

`version_has_a_file` 的来源：`version_has_a_file = tab(T_EXPECTED, version * EXPECTED_WORDS + 5u) == EXPECTED_VERSION_WITH_A_FILE;`（96）；表一侧 `FileOfTheExpectedVersion::WithoutAFile => EXPECTED_VERSION_WITHOUT_A_FILE,`（`crash_judge_tables.rs:1404`）、`file: FileOfTheExpectedVersion::WithoutAFile,`（364），`pub const EXPECTED_WORDS: usize = 6;`（1481）不变，第 5 字原来恒写 0，今天 0 = 有文件，所以不带「没有文件」的流表字节与改前逐字相同（只有摘要里的形态版本变）。

**复用键与判法闭包。**
- 对已有流一字不变：新增的一段 `for version in flow.versions_without_a_file.iter().filter(|version| {`（`T/crates/singlefs-checker-tier/src/crash_amplification.rs:670`）只对表里的项写 `"version_without_a_file instance={} txg={}\n"`（676），空表一个字不写；过滤的上界与带文件那一段同一个 `highest_reachable_root`。落到的根 ≤ 上界，它「比它旧的最新那一项」也 ≤ 上界，所以按同一上界进键够用。
- 旧判定不被复用：`pub const JUDGE_TABLES_VERSION: u32 = 5;`（tables.rs:42）进表摘要；判法闭包（0.6：收了 52 份）里有 `crash.rs`、`crash_judge_tables.rs`、五份 `.wgsl`（原文进摘要）与 `walk.rs`，这一批改了它们，闭包摘要必变，所有旧块的复用键都变。`crash_amplification.rs` 与 `crash_judge_dispatch.rs` 不在闭包里（流水线与派活，按设计）；改的是流程输入的那几行已进 `JudgeInputsOfTheFlow`。
- 两个判器入口：CPU `judge_one_block` 用 `flow.version_table()`（crash_amplification.rs:1214；`fn version_table` 在 186 行用 `Layer0VersionTable::new`，同一个根两边都有时 panic）；GPU 两处 `build_judge_tables_against_the_version_table(`（2568、5514）；旧的 `build_judge_tables` 留作空表入口，旧调用方不动。

**判定：一致。**
- 弱点 W-5a（测试尖度，不是实现错）：GPU 对拍用例 `the_gpu_judge_agrees_with_the_cpu_on_the_deleted_flow_against_a_version_table_without_a_file_from_the_intent_on`（`T/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:10936`）把意图那一版写死成 (实例 1, txg 5)（`let versions_without_a_file = [PublishedVersionWithoutAFile {`，10947），只断言 `comparison.mismatches.is_empty()`，不断言 CPU 一个状态都不红（`Comparison` 里有 `cpu_red`，同一文件别的用例用 `comparison.cpu_red > 0` 钉红）。写错身份时 CPU 与 GPU 一起把 txg 6–8 报没有文件判成违例，仍一致、用例仍绿；变异行 3112、3119、3120 在那种情形下都红不了。该流的文档自己写了 txg 3、4 带文件、意图是 txg 5，数是对的；缺的是把它断言出来。上卡时才分得出，复核不了。
- 事实：新增放量用例 `crash_amplification_of_the_unlink_after_the_multi_record_publish_stream_is_clean`（`T/crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:943`）的版本表（带文件：第一、第二、第三个文件版本；没有文件：意图那一步，996）与同文件 `prepare` 的 `versions` / `versions_without_a_file`（410）逐项同形，flow 经 `common_crash_points::amplify_prepared_flow_against_the_version_table(`（1000）。变异行 3111 去掉它会红，是 `verdict-store` 档用例，复核不了。
**什么现象会推翻：** 任一 (落到的根, 结局) 上 CPU 与 GPU oracle 给不同的类；旧流的复用键与改前不同；或上卡对拍 `…deleted_flow_against_a_version_table…` 不逐位相同（没跑）。

## K1-S6　C744 ①（对拍流）

**应该是什么。** C744 的验收：「对拍流补进这三类状态，每类至少一个状态上逐位比 GPU 与 CPU 的违例位」（`.claude/kb/checks-owed.md:593`）；①：「由记录施加出来的版」与「下一次挂载先施加的那一版」「同时在（新池新建文件那次发布的状态）」（同一行）。
**被判对象。** `the_gpu_judge_agrees_with_the_cpu_on_states_holding_a_version_applied_by_its_records_beside_the_version_the_next_mount_applies_first`（对拍文件 10845）用 `record_three_unit_file_applied_only_by_its_records_after_a_mount`（9227）加一个什么都不改的闭包 `leave_the_records_before_the_mount_as_they_are`（10837），红项实参是 `&[]`。
**判定：部分一致。** 对拍有、读的是老 helper（挂载、取号 2、写行、暖机、实例 2 覆盖写），比逐状态违例位。两处没有被钉住：① 流不是 kb 字面写的「新池新建文件那次发布」，是三单元文件那次发布 + 挂载 + 覆盖写（同一类状态换了一条流，用例文档自己写「推的，读录制的次序」）；② 用例不断言真有一个状态同时带「由记录施加出来的版」与「下一次挂载先施加的那一版」（实参 `&[]`、`Comparison` 里没有读这类状态的计数），枚举里没有这样的状态时用例仍绿、C744 ① 的「每类至少一个状态」没被证。存在与否要在卡上或 checker 档里数，我这边复核不了（要跑 checker 档）。
**什么现象会推翻「部分一致」成「一致」：** 在这条流的枚举里数出至少一个同时带两种版本的状态（例如由 `Comparison` 记下每个状态的候选版本种类）；数不出就是 C744 ① 没还。

## K1-S7　三个内核钉值（`crash_judge_dispatch.rs`）

**应该是什么。** 钉值是内核展开后表达式数与离预算 `pub const LARGEST_INLINED_EXPRESSIONS_OF_A_KERNEL: u64 = 400_000;`（`T/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:246`）的余量；改了内核要重量。
**复算。** 0.5：在冻结副本源码上用同一个 `kernel_inlined_size` 量三个内核，`recovery` 320015（余 79985）、`checker-scan` 382259（余 17741）、`checker-walk` 337872（余 62128），与钉值 `("recovery", recovery_kernel_source(), 320_015, 79_985),`（1347）、`382_259,`（1351）、`337_872,`（1357）逐个相同；三份内核另用 naga 校验器验过，都过。
**判定：一致。**（观察：扫描内核余量 17741，约预算的 4.4%，上一批是 18455；这一批它的增量（+714）只能来自共用的 `common.wgsl` 里 `read_at` 的守卫（扫描内核自己的源码与 `crash_judge_checker_common.wgsl` 不在这批改动的 18 份里）。下一次动 `common.wgsl` 或扫描内核时先量。）
**什么现象会推翻：** 在同一份源码上用同版 naga 量出不同的数。

## K1-S8　变异表

**应该是什么。** 追加 19 行、按名字整行替换 25 行；替换的行只换原文与替换文，名字、cargo 参数、必须红的测试照旧；追加的行点名的测试在改坏时红；每一行的原文在源码里恰好命中一次、替换文写进去仍编得过。
**逐格比。**
- 名字与参数：0.8 逐行比，替换的 25 行只第 3、4 段不同，第 1、5、6 段逐字同；追加的 19 行在 3102–3120。
- 原文恰好命中一次：0.2 `static-check` 退 0。
- 改坏之后红的是点名那一句：harness 档 9 + 21 行现跑全红（0.9）；GPU / checker 档 14 行（追加 3111–3120，替换 2515、2973、3070、3071）我逐行套用进副本，编小程序（含 `crash_judge_gpu.rs` 的三个内核），其中 12 行是 WGSL（3113–3120、2515、2973、3070、3071），套用后三个内核都编得过、过 naga 校验；3112 是 Rust（`crash_judge_tables.rs`）也编得过；3111 在 `crash_amplification.rs`，那个模块只在 `verdict-store` 特性下编（`T/crates/singlefs-checker-tier/src/lib.rs:11` 的 `#[cfg(feature = "verdict-store")]` 管 `pub mod crash_amplification;`，12 行），我没开 RocksDB，这一行没编，读码推它编得过（替换文用的 `Layer0VersionTable` 已在文件里 use）。红不红要上卡，复核不了。
- **冲突（一行没被换，改坏后编不过）：** 变异表第 2983 行（`第 10k 批 攻方 H-b：实例表链「走过」按第 0 条位置条目认…`）的替换文是 `if !self.visited_units.insert(page_placement) {`，而 S 把实例表链里的 `let page_placement = …` 删掉了（diff 里 `-            let page_placement = (pointer.locations[0].device, pointer.locations[0].slot);` 同位置那一行），那一行原文 `if !self.visited_units.insert(placement_it_was_read_from) {` 仍恰好命中一次，所以 `static-check` 不报；套用之后现跑：`error[E0425]: cannot find value `page_placement` in this scope`，`--> crates/singlefs-checker/src/walk.rs:3295:43`，`error: could not compile `singlefs-checker` (lib) due to 1 previous error`。`research/scripts/crates-mutation-rows.py:729-732` 把这种归「无效」，门禁里「有变异无效」判红（同文件 1288 行）。方法：我拿 S 删掉的、改后文件里不再出现的每个标识符（`already_walked_from_another_root`、`node_locations`、`page_placement`、`root_locations`、`tree_table_walked_by_an_earlier_version`、`class_tag_of_the_first_copy_verifying_at_the_length_of_its_class` 等）去扫变异表每一行的替换文，只有 2983 一行中；GPU 那一行同名的 2996（`if (!w_visit(pointer.device0, pointer.slot0)) {`）替换文仍有效。改法写成对实现的改动：把 2983 行的替换文换成 `if !self.visited_units.insert((pointer.locations[0].device, pointer.locations[0].slot)) {`（与 3506、3507 两行同一写法），点名测试照旧。
**判定：冲突（一格，2983）。**其余一致。
**什么现象会推翻：** 在冻结副本上套用 2983 行后 `cargo build -p singlefs-checker` 编得过——我现跑是编不过；或门禁把它归到抓到。

## K3　被判的 18 份文件，每份一格（冻结副本 sha256 与 `crates-sha256.txt` 逐份对上，见 0.1）

| # | 文件 | 这一批改了什么 | 判定 | 依据（行号都是冻结副本里的） |
|---|---|---|---|---|
| 1 | `crates/singlefs-checker/src/walk.rs` | 六把键按读回落点认；`IndexNodeReading`；折叠表改记；「自证过」合成一份；两份都没自证过当读不出 | 一致 | S1 表、S3；`IndexNodeReading`（303）、`read_a_unit_to_walk_and_the_placement_it_was_read_from`（1290）、`class_of_a_copy_verifying_at_the_length_of_its_class`（4570）；harness 203 条绿（0.3）、9 + 21 条变异行红（0.9）；O1-1 一形没钉 |
| 2 | `crates/singlefs-checker-tier/src/crash_amplification.rs` | `CrashFlow` 加 `versions_without_a_file` 与 `version_table()`；CPU 判器、GPU 表入口改按版本表；复用键追加「没有文件那几版」 | 一致 | `pub versions_without_a_file`（177）、`version_table`（186）、`flow.version_table(),`（1214）、键 670–680、GPU 入口 2568 与 5514；空表一个字不加 |
| 3 | `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` | 走读内核同改六把键；全零守卫之外的 F-α 另按 32 KiB 读；类型段 0 的回落去掉 | 一致 | S1 表（与 CPU 键逐把相同）、S2、S3；三份内核过 naga 校验（0.5）；上卡对拍复核不了 |
| 4 | `crates/singlefs-checker-tier/src/crash_judge_common.wgsl` | `zero_unit_version_over_a_slot_nothing_wrote` 守卫按 `SLOT_SHIFT`、先判起点再拿差比 | 一致 | 510–511；仿真 9918 格新守卫 0 处与真值不同、旧守卫 1133 处不同（0.7）；注释仍写旧界（503，主 agent 已知） |
| 5 | `crates/singlefs-checker-tier/src/crash_judge_dispatch.rs` | 三个内核钉值 | 一致 | 1347、1351、1357 与 0.5 现量逐个相同；扫描内核余量 17741（约 4.4%） |
| 6 | `crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl` | `classify_oracle` 认「没有文件」的那几版 | 一致 | 与 CPU `classified_oracle_violation_for_the_version_table` 逐支同（K1-S5 表） |
| 7 | `crates/singlefs-checker-tier/src/crash_judge_tables.rs` | `JUDGE_TABLES_VERSION` 4→5；期望版本表第 5 字；`build_judge_tables_against_the_version_table` | 一致 | 42、254–259、364、1404；`EXPECTED_WORDS` 仍 6（1481），空表字节与改前同 |
| 8 | `crates/singlefs-checker-tier/src/crash.rs` | `judge_layer0_state_range_against_the_version_table`；旧入口经 `with_a_file_only` 委托 | 一致 | 旧入口 1412、新入口 1444、判一个状态 1476；`Layer0VersionTable` 974 起，旧调用方（`e170`、`e171` 几个 bin）不动 |
| 9 | `crates/singlefs-checker-tier/tests/common_crash_points/mod.rs` | 新 `amplify_prepared_flow_against_the_version_table`；旧入口委托并传 `&[]` | 一致 | 199、230、250、288；`CrashFlow` 字面量带字段 |
| 10 | `…/tests/crash_amplification_fixed_script_through_rollback.rs` | `CrashFlow` 字面量加一行 `&[]` | 一致 | 223（该文件只一处 `CrashFlow {`） |
| 11 | `…/tests/crash_amplification_fixed_script_through_unmount.rs` | 同上 | 一致 | 336 |
| 12 | `…/tests/crash_amplification_pipeline_records_checks_and_reuses.rs` | 同上，两处 | 一致 | 79、851（两处 `CrashFlow {` 各一行） |
| 13 | `…/tests/crash_amplification_two_flows_share_their_prefix.rs` | 同上，三处 | 一致 | 82、993、1116（三处各一行） |
| 14 | `…/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs` | 新放量用例（`verdict-store` 档）带「没有文件」的版本表 | 一致（没编没跑） | 943；版本表与 `prepare`（410）逐项同形；变异行 3111；`crash_amplification` 只在 `verdict-store` 下编，复核不了 |
| 15 | `…/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` | 7 条新对拍流（allrbmap 分叉形、P4 两条回绕、F-α 一条、F-ID-b 两份都没自证过、删完流带版本表、C744 ①），`compare_on_every_state_against_the_version_table` | 部分一致 | 6 条与 S1–S3 对得上；删完流那条尖度弱（W-5a，10947）；C744 ① 那条没钉状态存在（10845，K1-S6）；全部要上卡，复核不了 |
| 16 | `…/tests/gpu_unit_checks_agree_with_the_cpu_and_catch_a_flipped_byte.rs` | `CrashFlow` 字面量加一行 | 一致 | 228 |
| 17 | `crates/singlefs-harness/tests/checker_known_bad_images.rs` | S4 全部新坏镜像、旧用例一处实参改 `&[]` | 一致 | K1-S4 表；203 条绿、16 条新用例绿、变异下红 |
| 18 | `crates/mutations.tsv` | 追加 19 行、替换 25 行 | 冲突（一行） | 第 2983 行替换文引用已删的 `page_placement`，套用后编不过（K1-S8）；S 的 44 行（追加 19、替换 25）名字、参数、必须红的测试照旧，其中 30 行 harness 档现跑红；2983 是 S 没换的老行 |

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1-S1 allrbmap | 一致 | CPU、GPU 六把键同取读回落点；合法状态判定不变（读码）；`O1-1` 一形没钉 |
| K1-S2 P4 | 一致 | 守卫与真值 9918 格逐个相同；F-α 在攻方那一形上 CPU、GPU 同判；读区间差一处既有边界未钉 |
| K1-S3 P2 | 一致 | 三处调用两处同义、一处是用户定的；两份都没自证过 CPU、GPU 同判 |
| K1-S4 P3 | 一致 | E1、E2、E4、E6、I-4.9 那一形各有坏镜像，变异下点名那条红，三行只红点名那条 |
| K1-S5 删除流 | 一致 | 三处 oracle 判法逐支同、复用键对旧流不变；对拍用例尖度弱（W-5a） |
| K1-S6 C744 ① | 部分一致 | 对拍流有；两种版本同在的状态存在与否没被钉，流也不是 kb 字面那一条 |
| K1-S7 钉值 | 一致 | 三个数现量逐个相同，内核过 naga 校验 |
| K1-S8 变异表 | 冲突 | 第 2983 行（S 没换的老行）替换文编不过，门禁判「无效」红；S 自己的 44 行照旧 |
| K3-1 … K3-17 | 一致 / 部分一致 | 见上表：第 15 份部分一致，第 14 份没编没跑，别的一致 |
| K3-18 `mutations.tsv` | 冲突 | 同 K1-S8 |

## 给主 agent 的线索（我的推，都是零轮）

1. 第 2983 行换替换文：`if !self.visited_units.insert((pointer.locations[0].device, pointer.locations[0].slot)) {`，点名测试 `a_second_instance_table_page_whose_pointer_names_the_first_page_on_device_zero_is_walked_from_the_copy_read_back` 照旧；提交时的 `checker-tier-crates-mutation-replay` 不先改会判红。
2. `…deleted_flow_against_a_version_table…` 用例补 `assert_eq!(comparison.cpu_red, 0)` 与「意图那一步的根等于 (1, 5)」；C744 ① 用例补一个计数，证真有同时带两种版本的状态。
3. 下一批若补读区间差：造位置条目校验和 = 「16 KiB 节点加下一槽」或「空槽加下一已写槽前 16 KiB」那 32 KiB 的 CRC，看 CPU、GPU 判得同不同（K1-S2 末那一段，推的）。
4. O1-1 一形（两份都读不出而第 0 条落点早被走过）补一条坏镜像。

## 要主 agent 跑的命令（我复核不了的，按条列）

| 项 | 命令（工作目录 = 仓根，经内存包装、`SINGLEFS_HEAVY_TESTS=commit`） | 看哪几行判结果 |
|---|---|---|
| 7 条新对拍流上卡 | `cargo test --release -p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_a_shared_leaf_container_born_after_the_rollback_target_reads_back_from_its_second_copy …`（另六条名字见 K3-15） | 每条 `test result: ok` 或 `NOT_RUN_NO_GPU_ADAPTER`（后者不算过） |
| 放量用例 | `cargo test --release -p singlefs-checker-tier --features verdict-store --test crash_enumeration_unlink_after_the_multi_record_publish_stream -- crash_amplification_of_the_unlink_after_the_multi_record_publish_stream_is_clean` | `test result: ok`；`CRASH_AMPLIFICATION flow=unlink-after-multi-record-publish` 那一行 `red=0` |
| GPU / checker 档 14 行变异 | 门禁 `checker-tier-crates-mutation-replay`；先改 2983 | 「有变异无效」计数 0 |

## 没做什么

- 没跑 checker 档测试与任何 GPU 用例（派发提示明令）；GPU 一侧的结论是读 wgsl 推的，加上 naga 校验与表达式数现量。
- 没读攻方腿的报告与模型，没读实现员报告；K2（反例）不是我的格。
- 没在没开 `verdict-store` 的副本上编 `crash_amplification.rs`（含变异行 3111）；没跑 harness 全表以外的别的 harness 测试二进制（主 agent 的合入验证报 170 个二进制绿，我没复跑）。
- 没把 3000 多行变异表整表套用一遍：只对 S 改动的两份源码做了「删掉的标识符 × 替换文」的扫描、对 30 行 harness 档与 14 行 GPU / Rust 行逐行现跑或现编；别的行是否还红归提交时的整表。
- 不采纳、不出判决。

模型目录 `research/prompts/m3-batch33s-code-r1-sonnet-model/`：`rerun.sh`（不编译的四项：0.1、0.2、0.7、0.8）在空目录从头跑通两次，`compare.sh` 比两次输出相同、与 `expected.out` 相同；要编译的几项（0.3–0.6、0.9）按 `README.md` 分条跑，不在 `rerun.sh` 里。
我自己建的编译目录与仓副本交回前已删（逐个全路径写在交回正文里）。
