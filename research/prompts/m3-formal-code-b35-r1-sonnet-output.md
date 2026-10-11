# 第 35 批代码轮第一轮 云端正推腿（Sonnet）报告（2026-10-06）

<!-- doc-lint:not-numbers P1 P2 P3 P4 P5 P6 P7 Q1 Q3 Q4 FL B4 ALT1 RV -->

立场：从判决 `research/prompts/m3-formal-code-b34-r1-main-verification.md`、用户 2026-10-06 定的四问与 kb 条款，加上冻结副本里 core 今天的实现，推「应该是什么」，再与第 35 批改动逐格比。代码引文写 `crates/…` 相对路径与冻结副本 `/tmp/claude-1000/m3-formal-code-b35-r1/tree/` 里的行号；kb 引文写 kb 文件自己的行号。两腿都没读对方的东西（攻方的提示与产出一概没读）。

## 〇、怎么量的

- 副本：冻结副本拷到 `/tmp/claude-1000/m3-formal-code-b35-r1-sonnet/work`，清单核对 0 行不对（`sha256sum -c … | grep -vc ': OK$'` 输出 0）；每次施加补丁或变异之后核还原，一律 0 行不对。仓里 `crates/` 与冻结副本一个字没动。
- 编译：每条 cargo 经 `bash scripts/hosts/run-with-memory-cap.sh 8G bash scripts/hosts/capped.sh 10 cargo test --offline …`；峰值表里没有这几组目标组合的键，8G 不触发「超过峰值两倍」那一条（`tools/env.sh` 仍带了 `RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON` 的理由，预防别的机器上表里有键）。开跑前 `ps` 看过：没有 qemu-system、vm-bench、e152、fio 在跑（别的会话有 cargo 与长跑的 e171 在，`nice -n 19` 照跑）。
- 编译日志（`results/build.log`）三个被测包都有 `Compiling`，原样：
  ```
     Compiling singlefs-checker v0.1.0 ($S/work/crates/singlefs-checker)
     Compiling singlefs-core v0.1.0 ($S/work/crates/singlefs-core)
     Compiling singlefs-harness v0.1.0 ($S/work/crates/singlefs-harness)
      Finished `test` profile [unoptimized + debuginfo] target(s) in 17.28s
  ```
- 探针（只在副本里拼、不进仓）：`probes/probe_b35_tail*.rs` 接在 `checker_known_bad_images.rs` 整份之后，编成 `zz_probe_b35`；逐状态那几条用 opt-level 2 的另一个编译目录（debug 版一个状态要几秒，256 个状态 20 多分钟、opt 版 2 分钟）。崩溃状态的枚举是我自己写的近似：每段写按层 0 的口径逐子集（段长不超过 6 或 8）或逐前缀，没有层 0 的原地覆写三态与撕裂，所以只算「近似同形」，不当成层 0 的同数。
- 基线：`checker_known_bad_images` 整文件 `test result: ok. 136 passed; 0 failed`（`results/t-kbi.log`）；`checker_narrow_invariants_and_abandoned_roots` 20 过、`pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement` 9 过、`root_record_pointer_encryption_and_compression_fields_outside_the_first_version_are_refused_by_both_mounts` 4 过、`writable_mount_of_a_formatted_pool` 15 过、`reuse_after_raising_the_floor` 那一条 1 过、`--lib` 86 过 1 ignored（`results/t-base2.log`、`t-base3.log`、`t-base4.log`）。
- GPU 对拍、GPU 变异行、层 0 快档、`mutations.tsv` 整表：按正文第五节不跑。涉及它们的结论一律写「推的」。

## P1　FL：两类由记录造出来的版本改取带末条标志那条记录的新根段

### P1-a　取的是不是 core 真施加的那一份：一致

core 侧：恢复在逐条记录的循环里，只有走到带末条标志的那一条才把它的新根来源记成换根结果。
- 「末条没到就接着往下走，这次发布先不施加」（`crates/singlefs-replay-prefix/src/lib.rs:3899`）
- 「root_source_after_replay = record.new_root_source;」（`crates/singlefs-replay-prefix/src/lib.rs:3932`）
- 拼根处「每条记录都带整次发布的新根段（已定项 15），施加末条那一份」（`crates/singlefs-core/src/recovery.rs:3658`）

checker 侧三处都取带标志那条：
- 下一次挂载那一版：「record.record_flags_byte & RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH != 0」（`crates/singlefs-checker/src/walk.rs:5614`）接着「&& record.commit_marker == CommitMarker::Present」（`crates/singlefs-checker/src/walk.rs:5615`）。
- 由记录施加出来的那几版：同一个标志位（`crates/singlefs-checker/src/walk.rs:7855`），带标志的一到就换上、之后不再换（`crates/singlefs-checker/src/walk.rs:7859`）。
- I-4.1 重放那一版：本来就取重放里最后施加那条（`crates/singlefs-checker/src/walk.rs:8482`「new_root_segment: last_record.new_root_segment.clone(),」）。

量：正推探针 `probe_p1_subsets_of_the_record_copies_of_a_three_record_publish`。造法：新池新建文件 A（txg 3）之后顺序写三个数据单元的发布 B（txg 4，计数器 4、5、6，每条两块盘各一份，只有 6 带末条标志），根槽没落；B 重写的那片分配记录叶里两条记录罩住同一个槽（沿用新用例的 `overlap_two_records_in_an_allocation_leaf_the_publish_rewrote`，我另补一次恒等重链，因为 propagate 改了记录头而没重链，不补时 5、6 两条的反向链接不上、B 进不了重放前缀）。四种改法 × 6 份记录拷贝落没落的 64 种子集 = 256 个状态，每个状态跑池级 checker 与 core 的 `mount_writable`。`tools/summarize_p1.py` 的输出（原样，FL 开是冻结副本，FL 关是把 `mutations.tsv` 两行 FL 变异临时施加的副本）：
```
== t-p1b.log（256 行 = 64 个子集 × 4 种改法）
  clean                  {('checker 不红', 'core 不施加 B'): 37, ('checker 不红', 'core 施加 B'): 27}
  overlap                {('checker 不红', 'core 不施加 B'): 16, ('checker 红 I-5.4', 'core 不施加 B'): 21, ('checker 红 I-5.4', 'core 施加 B'): 27}
  overlap+stale_first    {('checker 不红', 'core 不施加 B'): 16, ('checker 红 I-5.4', 'core 不施加 B'): 21, ('checker 红 I-5.4', 'core 施加 B'): 27}
  overlap+stale_last     {('checker 不红', 'core 不施加 B'): 37, ('checker 不红', 'core 施加 B'): 27}
== t-p1-fl-off.log（256 行 = 64 个子集 × 4 种改法）
  clean                  {('checker 不红', 'core 不施加 B'): 37, ('checker 不红', 'core 施加 B'): 27}
  overlap                {('checker 不红', 'core 不施加 B'): 1, ('checker 红 I-5.4', 'core 不施加 B'): 36, ('checker 红 I-5.4', 'core 施加 B'): 27}
  overlap+stale_first    {('checker 不红', 'core 不施加 B'): 22, ('checker 不红', 'core 施加 B'): 27, ('checker 红 I-5.4', 'core 不施加 B'): 15}
  overlap+stale_last     {('checker 不红', 'core 不施加 B'): 4, ('checker 红 I-5.4', 'core 不施加 B'): 33, ('checker 红 I-5.4', 'core 施加 B'): 27}
```
读法：「core 施加 B」= 三条记录在两块盘上各至少落一份的 27 种子集（core 逐计数器取第一块盘上自证过的那份，`prefix=3`，或在叠了两条重叠记录时拒成两条记录罩住同一个槽）。
- stale_first（首条写成 A 的新根段，叠重叠）：FL 开时 checker 红 I-5.4 的 48 种里，27 种全在 core 施加 B 的子集上、core 同拒；FL 关时这 27 种 checker 一条不红（攻方 s3 那一形，core 拒而 checker 放过）。
- stale_last（末条写成 A 的新根段，叠重叠）：FL 开时 checker 一条不红，core 在 27 个施加 B 的子集上收下并现行版是 txg 4（施加的是末条那份 A 的树，没有重叠）：两边同向；FL 关时 checker 在 60 个子集上红 I-5.4（其中 27 个是 core 收下 A 的树的子集，另 33 个是 core 根本不施加 B 的子集）。
- 所以取哪一份字节：core 施加 B 的 27 个子集上（两种坏法各 27），FL 开时 checker 与 core 逐子集同向，FL 关时 stale_first 27 个不红而 core 拒、stale_last 27 个红而 core 收下，共 54 个子集分开。core 不施加 B 的子集上 checker 认 B 与否是另一件事，见 P1-b。

推翻它的现象：一份写者历史（每条记录同一份新根段，D23 已定项 15）上两种取法给出不同判定；或 stale_first 变体里 checker 在 core 施加 B 的任一子集上不红、或 stale_last 变体里 checker 红。

### P1-b　认哪些状态：条款没写、实现更窄但没到 core 的认法

核对条款（整行并排）：
- 条款 I-3.10：「与最新根同实例、checkpoint_txg = 最新根的 txg + 1、带提交标记、根环里没有它的根的那条记录」（`.claude/kb/invariants.md:152`）；代码：「record.record_flags_byte & RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH != 0」（`crates/singlefs-checker/src/walk.rs:5614`）。条款没有「带末条标志」这一条，实现多了它：冲突。b34 判决第三节第 3 条的写回批清单里没有这一条（那份清单列的是「I-5.4 那一行补由记录施加出来的那一版的三个条件」等），写回时要补。
- 条款 I-5.4：「与最新根同实例、txg 比最新根大 1、带提交标记，根环里没有它的根」（`.claude/kb/invariants.md:198`）；同一个函数认这一版（`crates/singlefs-checker/src/walk.rs:5595` 起），同上冲突。
- 实现自己写的限度：「发布边界不复刻」（`crates/singlefs-checker/src/walk.rs:5573`）。

量（上表同一批 256 个状态）：core 不施加 B 的 37 个子集里，FL 开时 checker 仍在 21 个上认 B（末条有一份落了、前面某个计数器在两块盘上都没落；`python3` 数 64 个子集里「6 号有、4 号或 5 号两盘都没」恰 21 个）；FL 关时 36 个。就是说末条条件把「超认」从 36 个收到 21 个、没收到 0，剩下的是「前缀断号、点名单元验证、发布边界」三样 `replay_the_next_mount_would_do` 才复刻的规则。
这 21 个状态在合法历史上不红（B 的树是合法的，单元先于记录落，D16 已定项 7），只多评估一次，见 P1-c；坏写者历史上多报的是 B 自己的坏树，core 此时不施加、checker 红，属于「判的是盘上有这一版的记录」而非「core 会施加它」，同 C738（`.claude/kb/checks-owed.md:587`「第 35 批 FL 让 checker 与 core 同取末条之后，写旧了的非末条从此谁都看不见」）一族。
末条没落时 I-3.10 / I-5.4 判的东西变了没有：
- 在「记录落了一部分、末条没落」的状态上，FL 之后这一版不进读集，只由别的版本（候选根）作保；合法历史上这些状态的判定向量不变（P1-c），新池新建文件那条流是单条记录的发布（带末条标志），评估集不变（P6）。
- 该不该变：向 core 靠，方向对；是否还要把前缀三规则也搬过来，条款没写，不归这一轮（零轮交主 agent）。

推翻它的现象：某个合法历史的崩溃状态上，FL 之后 I-3.10 或 I-5.4 由「成立」变「不适用 / 部分没判」而池里同时存在一个 core 真会施加的、判得出违例的版本；或 FL 开 / 关的判定向量在合法历史某状态上不同（见 P1-c 的比法）。

### P1-c　合法历史上多报或少判：没打中（我这一次抽样、两条历史）

两条合法历史，每个崩溃状态取全套判定（每条不变量一个字符 H / V / P / N 加全部说明文字的哈希），FL 开与 FL 关（`mutations.tsv` 两行 FL 变异临时施加）各跑一遍逐行比：
- H2：新池新建文件 A 之后顺序写三个数据单元的发布 B，崩溃状态 102 个（段写数 `[30, 6, 1, 2]`：三十个单元写取前缀、六份记录拷贝逐子集 64 个、根槽、尾段）。`PROBE-H2 states_with_a_violation=0`；`diff <(grep '^SIG' t-h2-on.log) <(grep '^SIG' t-h2-off.log)` 无差（`h2same`）。
- H3：三个数据单元的第一个文件之后重开可写挂载（取号 2、写行、暖机）、实例 2 覆盖写一次，从流里拿掉 B 的根槽写（B 只在它的三条记录里，挂载之后是由记录施加出来、根槽从没落盘的那一版——这就是新对拍用例 2、3 的搭法，harness 里重做），从 B 的记录段起 191 个状态。`PROBE-H3 states_with_a_violation=0`；FL 开 / 关逐行相同（`h3same`）。
- 新池新建文件那条流（一次只有一条记录的发布，带末条标志）：60 个状态，I-5.4 与 I-3.10 评估集逐状态相同，见 P6。
- harness 档 136 个坏镜像与合法镜像用例全绿。随机历史与层 0 快档我没跑（攻方的活、层 0 归提交时）。

坏写者那一侧的探针有区分力（防「两趟相同是因为探针没碰到这一版」）：H3 流上把 B 的首条记录写成 A 的新根段（`probe_h3_stale_first_record_of_b`，281 个状态）：FL 开 `union_of_red_invariants_over_all_states={}`，FL 关 `{"I-3.1", "I-3.5"}`——与新对拍用例 2 的注释写的形状一致（取先到的那条就走 A 的树、B 自己的单元没走到）。
推翻它的现象：任一合法历史上 FL 开 / 关的判定向量不同，或出现红。

### P1-d　「有末条取末条、没有退回先到的」那条退路

走到的条件：这一版被实例表的行判为施加过（行里 T ≥ 它的 txg）、记录自证过、反向链过，而带末条标志那一条在环里读不到（`crates/singlefs-checker/src/walk.rs:7810`「环里读不到带末条标志的那一条时退回先到的那一条」）。环是先进先出（旧的先被盖），一次发布的末条是这次发布里最新的一条，合法写者的历史里「末条没了、前面的还在」只有介质损坏或撕裂才会有。
取的那一份与 core 是否一致：合法写者每条记录同一份新根段，一致；写旧了的非末条不一致，与 C738 同形。量（`probe_p1_fallback_when_the_last_record_of_an_applied_only_version_is_not_readable`，H3 流最终镜像上把 B 的末条两份抹掉）：
```
PROBE-P1F legal_final red=[] partial=[]
PROBE-P1F legal_final_without_record6 red=[] partial=[]
PROBE-P1F stale_first_final red=[] partial=[]
PROBE-P1F stale_first_final_without_record6 red=["I-3.1", "I-3.5"] partial=[]
```
首条写旧而末条读不到时，退路取了首条的旧根段、走 A 的树，红 I-3.1 与 I-3.5，而 core 在那次恢复施加的是末条那份 B。判定：一致（合法历史）/ 同 C738 的盲区（坏写者）。没有新问题，也不是 b35 引入：改之前取先到的那条是这一形的唯一读法。
推翻它的现象：合法写者的历史上抹掉末条之后 checker 红（上表第二行）。

### P1-e　GPU 两处与 CPU 是不是同一组版本、同一取法（读码，推的）

- 拼 `R_DATA` 那一段（由记录施加出来的那几版）：同键（实例、txg）去重后，已占着的那条没带标志而新来的带了，就把下标换成新来的（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3833`「if ((existing.flags & 1u) == 0u && (r.flags & 1u) != 0u) {」）；已占着的是带标志的就不换。与 CPU 的 `takes_this_record` 三支（没有 → 取；已是末条 → 不换；先到的 → 遇末条换）逐支同。`flags` 是记录头偏移 7 那个字节（`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:284`「r.flags = rd8(r.base, 7u);」），CPU 的 `JOURNAL_RECORD_FLAGS_OFFSET` 也是 7（`crates/singlefs-checker/src/lib.rs:705`）。
- 下一次挂载先施加那一版：「if ((r.flags & 1u) != 0u && r.commit_byte == 1u && r.instance == newest_root.instance」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1086`）与 CPU 的 5614–5615 同条件。
- 「数组按计数器升序插入」：核了。插入时只在计数器上去重、按计数器排（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:327`「if (eq64(existing, counter)) {」、`:330`「if (lt64(counter, existing)) {」）。两条筛选都先卡实例（下一次挂载那一版 `r.instance == newest_root.instance`，拼 `R_DATA` 的去重键本身含实例），所以同一实例内数组次序就是计数器升序，取到的与 CPU 的 `BTreeMap<(实例, 计数器)>` 的 `find` 同一条。差一处是改动之前就有的：GPU 去重键只有计数器、CPU 键是 (实例, 计数器)（`crates/singlefs-checker/src/walk.rs:7183`「let mut first_copies: BTreeMap<(u32, u64), &ScannedJournalRecord> = BTreeMap::new();」），两块盘同一计数器坐不同实例的记录时两边留的不同；kb I-4.1 那一行写了这一形在第一版两盘池上不可达，FL 没碰它。
推翻它的现象：对拍用例 1、2 在显卡上跑出 GPU 与 CPU 不一致（我跑不了）。

## P2　Q1 甲：全零树表指针

### P2-a　红哪几条、是不是同一结局：一致（版本自己有单元时）；最新根多两格

同一个池级 checker，六种形态（`probe_p2_matrix_of_the_all_zero_tree_table_pointer_forms`、`probe_p2_older_candidate_root_with_units_of_its_own_all_zero`，原样摘自 `results/t-p2.log`、`results/t-p2c.log`，省去 DETAIL 行）：
```
F1.root_ring_root_txg5 (最新根)       red=["I-2.1","I-4.8","I-7.2","I-7.4"] partial=["I-5.4","I-5.5","I-9.28"]   I-3.1 不适用（最新的根下面还没有记账树）
F1.root_ring_root_txg4 (暖机候选根)   red=["I-2.1","I-4.8","I-7.4"]         partial=["I-5.4","I-5.5","I-9.28"]
F1c.older_candidate_root_txg3 (带自己的单元) red=["I-2.1","I-3.1","I-4.8","I-7.4"] partial=["I-5.4","I-5.5","I-9.28"]
F2.applied_only_by_records_all_zero   red=["I-2.1","I-3.1","I-4.8","I-7.4"] partial=["I-5.4","I-5.5"]
F3.next_mount_version_all_zero        red=["I-4.1"]                          partial=["I-5.4"]   core=refused(Recovery(UnitUnreadable { slot: SlotNumber(0) }))
F4.abandoned_root_all_zero            red=["I-7.4"]                          partial=["I-9.28"]
```
- 由记录施加出来的那一版（F2）与「自己带单元的」回退候选根（F1c）红同一组 {I-2.1, I-3.1, I-4.8, I-7.4}，部分没判里 I-9.28 只在根环里的根上有（F2 不在根环里）。树表指针全零时 I-3.1 红是因为这一版的单元没并进遍历、记账多于遍历（`I-3.1` 说明 `记账的已分配 Some(1032192)，遍历全部有效根得到 901120`）。
- 暖机候选根（F1 txg 4、3）没有自己的单元，不红 I-3.1；最新根（F1 txg 5）I-3.1 因「最新的根下面还没有记账树」报不适用、多红 I-7.2。这是两种形态在 I-3.1 与 I-7.2 上本来就不同的机理，不是 Q1 引入的分歧：同一个判定函数（`crates/singlefs-checker/src/walk.rs:1484`「self.follow_an_all_zero_tree_table_pointer_like_the_root_record_reader_of_the_core(」）三类共用，持有者只改说明（`TreeTablePointerHolder`，`walk.rs:1464`）。
- 下一次挂载先施加的那一版（F3）：主走读不走它，I-4.1 另起的走读判红（`crates/singlefs-checker/src/walk.rs:8495`「walk_failures.is_empty(),」）；core 拒挂成 `UnitUnreadable { slot: SlotNumber(0) }`。
- 部分没判与「射程里判不了」四处：F1 / F2 的 I-5.4、I-5.5 报部分没判点名这一版，I-9.5 与 I-9.18 报「走读到的某一版树表指针全零（core 照它读盘 0 槽 0、读不出）」；F3 的 I-5.4 点名「下一次挂载先施加的那一版」。三处与主走读判损坏同向、不互相打架：判不了的不报成立，判得了的（读不出）报红。
- 对 CPU / GPU 一致性：GPU 走读内核对这几个阶段共用同一个调用点（见 P3）。

推翻它的现象：某形态上红的集合与上表不同（探针原样可复跑：`tools/rerun.sh probes`）；或某形态上一条本该「判不了」的不变量报了成立。

### P2-b　下一次挂载那一版全零红 I-4.1（实现员自选、零轮）：站得住

- 射程：I-4.1 条款写「所选根（重放之后那一版）指向的树全部可完整遍历」（`.claude/kb/invariants.md:176`），就是这一版。core 在这一版上拒挂（F3：`UnitUnreadable { slot: SlotNumber(0) }`），树确实不可遍历。
- 有没有更贴的：没有。I-7.2 只管最新根；I-4.8、I-7.4 按回退候选集判，下一次挂载那一版不在候选集里；I-2.1、I-4.2 走主走读，主走读不走这一版（F3 里除 I-4.1 外全不红，I-5.4 报部分没判）。I-4.1 是条款里唯一射程写着「重放之后那一版」的一条。
- 合法历史上误红：没打中。core 里写进记录新根段的树表指针不会是全零：全零的 `NodePointer::empty_root()` 在非测试代码里只用在 mkfs 的映射根与分配记录树根（`crates/singlefs-core/src/make_filesystem.rs:612`「mapping_root: NodePointer::empty_root(),」），journal.rs 里那一处在测试模块里（`crates/singlefs-core/src/journal.rs:388`「#[cfg(test)]」，`crates/singlefs-core/src/journal.rs:404`「new_tree_table: NodePointer::empty_root(),」）。H2 / H3 / H3S 的 102 + 191 + 281 个状态与新池流 60 个状态、harness 136 条里没有一个 I-4.1 红。
- 它与 core 的结局：「core 拒、checker 红」同向；这一格与 b34 判决第五节第 1 条（用户定「甲」：下一次挂载先施加的那一版全零另判损坏）相符。条款没写这一格的取法，是实现员的读法，仍按零轮记。

推翻它的现象：一个 core 写者历史里出现新根段树表指针全零的记录（那就是 core 写出了会被自己拒的根）；或 I-4.1 在一条合法历史的前缀状态上红。

### P2-c　I-5.4、I-5.5、删除态、墓碑谱系对两类版本的处置自洽：一致；kb 三处与实现冲突

自洽见 P2-a 第四条。冲突（两边整行并排）：
1. 条款 I-9.28：「由记录施加出来那一版里的树表指针全零，主走读仍当这一版没有树表、不记走读失败」（`.claude/kb/invariants.md:315`）；实现：由记录施加出来的那一版也走「`self.follow_an_all_zero_tree_table_pointer_like_the_root_record_reader_of_the_core(`」（`crates/singlefs-checker/src/walk.rs:1484`），那里「self.walk_failures.push(match unit {」（`crates/singlefs-checker/src/walk.rs:1572`）记一次走读失败。F2 的 I-4.8、I-7.4 红就是它。
2. 条款 I-9.28：「GPU 判器的走读对根记录里全零的树表指针没有跟过去，这种镜像上 CPU 红这几条而 GPU 不红」（`.claude/kb/invariants.md:315`）；实现：GPU 走读内核「if (tree_table_pointer.all_zero) {」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3138`）先读两条位置条目、记一次失败、不再往下走。
3. 条款 I-5.5：「认出来而没有数据单元那一形仍落缺省理由」（`.claude/kb/invariants.md:199`）；实现：那一形落到专门理由「VERSIONS_BUILDING_TREES_FROM_THE_MKFS_WATERMARK_HAVE_NO_DATA_UNIT,」（`crates/singlefs-checker/src/walk.rs:4903`）。
这三处 b34 判决第三节第 3 条的写回批清单里都有（「写明各处『自证过的根』的集合与 Q1 甲」「随第 35 批 Q3 甲合入改」「随 Q4 甲改」）。第四处见 P1-b：I-3.10、I-5.4 两行没写「带末条标志」，那份清单里没有。别的我没读出冲突：I-5.4 那一行「树表指针全零的版本（core 的根记录读者照它读盘 0 槽 0、读不出）」（`.claude/kb/invariants.md:198`）与 F1 / F2 / F3 的部分没判一致。

## P3　Q3 甲：GPU 走读内核

### P3-a　全零分支红位与 CPU 逐位相同（读码，推的）

两边同一件事，逐点对：
- 读两条位置条目、各判一次 I-2.1：CPU 的 `read_referenced_unit` 在 `self.walk_failures.push(match unit {` 之前调（`crates/singlefs-checker/src/walk.rs:1572`）；GPU 的 `w_read_referenced_unit` 两份各判（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1286`「fn w_read_referenced_unit(p: Ptr, unit_bytes: u32) -> Slice {」）。GPU 在这里调它：「_ = w_read_referenced_unit(tree_table_pointer, NODE_BYTES);」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3139`）。
- 记一次走读失败、映射根不走、不判 I-2.5、不记引用：CPU `return`（`walk.rs:1483` 起那个 `if tree_table_pointer_view.all_zero {` 里直接返回），GPU 「w_failures = w_failures + 1u;」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3140`）后返回。
- 先走实例表链与分配记录树、最后才走树表这个次序两边相同（CPU `walk_root` / `walk_version_applied_only_by_records`，GPU `w_walk_one`，调用点「w_walk_tree_table_and_mapping(parse_node_pointer(record_base, tree_table_pointer_offset)」`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3206`）。
- 阶段：根环最新根、候选根、由记录施加出来的版本、被抛弃的根、低于下界的根、重放那一版都经这一个调用点。各阶段怎么由失败翻成红位：候选根与由记录施加的版本 `walked_into` 判 I-7.4 与 I-4.8；最新根 `newest_failures` 判 I-7.2；被抛弃的根 `w_start(1u, false)` 不判、`w_failures > 0u || w_mismatches > 0u` 红 I-7.4；重放那一版同样不判、`w_failures > 0u` 红 I-4.1。与 CPU 的 F1 / F2 / F3 / F4 红集合一一对得上：F3 里主走读不红 I-2.1（重放那一版用的是一份丢弃的判定累加器），GPU 在这一阶段 `w_judging` 是 false、也不红 I-2.1。
推翻它的现象：对拍用例 3、4 在显卡上 GPU 与 CPU 不一致，或在显卡上跑 GPU 变异（`mutations.tsv` 2952 行）不红。我跑不了。

### P3-b　四条新对拍流造得对不对：CPU 一侧在 harness 里重做了，GPU 一侧推的

流的搭法就是主 agent 提到的、实现员没核过的那两条「拿掉根槽写」。在 harness 里照同一搭法重做（`probe_h3_*`：三个数据单元的第一个文件、重开可写挂载取号 2、实例 2 覆盖写、从写表里拿掉 txg 4 的根槽写；枚举每个写段的子集），CPU 池级 checker 的判定：
| 流（对拍用例） | 我重做的 CPU 一侧 | 与用例注释 / 断言对得上吗 |
|---|---|---|
| 1 下一次挂载那一版首条写旧 + 叠重叠 | P1 矩阵的 `overlap+stale_first`：core 施加 B 的 27 个子集上红 I-5.4，FL 关时这 27 个不红 | 对（用例断言 CPU 红 I-5.4；行 2950 去掉末条条件时 CPU 红 GPU 不红的状态就是记录落了、根没落的那几个） |
| 2 挂载之后由记录施加那一版首条写旧 | `probe_h3_stale_first_record_of_b`：281 个状态 `union_of_red_invariants_over_all_states={}`；FL 关 `{"I-3.1", "I-3.5"}` | 对：注释写「取先到的那一条就走 txg 3 的树……I-3.1、I-3.5 在 GPU 一侧红出来」 |
| 3 挂载之后由记录施加那一版的三条记录树表指针全零 | `probe_h3_all_zero_tree_table_pointer_in_the_records_of_b`：281 个状态红的并集 `{"I-2.1", "I-3.1", "I-4.1", "I-4.8", "I-7.4"}`；挂载之前的状态只红 I-4.1，挂载之后（行写下去之后）红 I-2.1、I-3.1、I-4.8、I-7.4 | 对：用例断言 CPU 红 I-2.1、I-4.8、I-7.4（`for invariant in ["I-2.1", "I-4.8", "I-7.4"]`），名字说的「由记录施加出来的那一版」那一形确实在挂载之后的状态上出现；下一次挂载那一形在同一条流里挂载之前出现（只红 I-4.1） |
| 4 新池那次发布的根与它的记录里树表指针全零 | `probe_flow4_…`：60 个状态，根落了的 6 个状态红 {I-2.1, I-4.8, I-7.2, I-7.4}，只记录落了的 4 个状态红 {I-4.1}，其余 50 个不红 | 对：用例断言 CPU 红 I-2.1、I-7.2、I-4.1 |
我重做的与对拍用例的流不是同一份代码（对拍用例在 checker 档、要显卡的 test 二进制里），差在：我没经 `RecordedFlow` 的 `judged_root_index` 与 `flow.versions`（层 0 oracle 用的），没有原地覆写三态。所以「构造得对」只做到「同一搭法在 CPU 池级 checker 上给出用例注释写的红集合」，不做到「那份二进制编出来就是这样」（那要在 checker 档编、跑，`cargo test -p singlefs-checker-tier` 归提交时）。
推翻它的现象：对拍用例自带的 CPU 一侧断言（`cpu_checker_red_invariants.contains(…)`）在 checker 档里不成立。

## P4　Q4 甲与两处说明

### P4-a　新理由的分支在什么条件下走到，与「一次都没判」「没有对象」分不分得开：分得开

走到的条件（读码加探针）：
- 先数认出来的版本：「versions_built_from_the_mkfs_watermark += 1;」（`crates/singlefs-checker/src/walk.rs:4811`），之后这一版有三条路不判而安静走开：extent 树根全零（`crates/singlefs-checker/src/walk.rs:4813`「if extent_root.all_zero {」）、extent 树里一条数据指针都没有（`crates/singlefs-checker/src/walk.rs:4836`「if smallest_data_unit_slot_on_each_device.is_empty() {」）、两样都有但这一版在每块盘上都没有数据单元（不可达，见下）。读不全 / 分配记录树条目不止一条 / 分配记录树读不全走的是 `versions_not_judged`，报部分没判，不进这一支。
- 末尾：认出来的一版都没有 → 「judgements.not_applicable(invariant, NO_VERSION_BUILDS_TREES_FROM_THE_MKFS_WATERMARK);」（`crates/singlefs-checker/src/walk.rs:4899`）；认出来了 → 「VERSIONS_BUILDING_TREES_FROM_THE_MKFS_WATERMARK_HAVE_NO_DATA_UNIT,」（`crates/singlefs-checker/src/walk.rs:4903`）。
- 它不压过别的结局：`Judgements::into_report` 的次序是违例、部分没判、射程内判不了、评估过（成立）、最后才是不适用的理由（「if let Some(detail) = self.first_violation.get(invariant) {」`crates/singlefs-checker/src/image.rs:345`，「} else if let Some(unjudged) = self.partially_judged.get(invariant) {」`:347`，「} else if self.evaluated.get(invariant).copied().unwrap_or(0) > 0 {」`:351`，缺省理由「.unwrap_or("这个镜像上没有它判的对象"),」`:358`）。所以认出来的版本里只要有一版判到了，这一句就不出现（`not_applicable` 不改已评估过的）。
三种结局分得开（`probe_p4_first_data_unit_verdicts_by_form`，原样）：
```
PROBE-P4 first_file_clean I-5.5=H
PROBE-P4 mkfs_only I-5.5=N 回退候选集里每条根与由记录施加出来的那几版，没有一版是从 mkfs 水位第一次建树的那一版（树表非空、种类 1 条目的树 ID 是 65536、每条条目的诞生 txg 等于这一版的 txg）：一版都不判
PROBE-P4 extent_root_zeroed I-5.5=N 认出了从 mkfs 水位第一次建树的那一版，但它们都没有数据单元（extent 树根全零或一条数据指针都没有）：一版都不判
```
「没有对象」（认出来的版本有数据单元、而某块盘上一个都没有）在代码里是该盘跳过、不记不报（`crates/singlefs-checker/src/walk.rs:4863` 的 `for device in &devices {` 里 `walk.rs:4865` 的 `if let Some(&smallest_slot) = …`），不单独成一句理由，也不落到新句子上：只要有一块盘上有数据单元，就有一次评估。落到新句子上的「说谎」形：认出来的版本的数据指针全落在 `reader.devices()` 之外的设备身份上，这时一次评估也没有、新句子写「没有数据单元」而实际有；这种镜像在别处先红（位置条目指向池里没有的盘），没造，推的。
GPU 判器只有违例位，这一句不在它那一侧（条款 I-5.5 那一行写了「GPU 判器只有违例位，这几格不置位」）。

推翻它的现象：一张认出了从 mkfs 水位建树那一版、且至少一块盘上有数据单元而被评估过的镜像，I-5.5 却报这一句；或三种结局里有两种落同一句。

### P4-b　坏镜像造出来的是不是「认出来而没有数据单元」这一形：是；顺带一个不归本轮的观察

用例把树表里 extent 树那一条的根指针 86 字节写零、重封到根（`a_version_building_trees_from_the_mkfs_watermark_without_a_data_unit_leaves_…`）。探针原样：`PROBE-P4 extent_root_zeroed I-5.5=N 认出了从 mkfs 水位第一次建树的那一版，但它们都没有数据单元…`，同一份镜像上 I-5.5 之前是成立（`first_file_clean` 那行）。认出来的条件 ①②（树表非空、extent 条目树 ID 65536、诞生 txg 等于这一版 txg）没动，所以是「认出来、没有数据单元」，不是「没认出来」，也不是读不出（没有部分没判）。
逐条比干净镜像与这张镜像（`probe_p4_what_else_changes_when_the_extent_root_pointer_is_zeroed`）：
```
PROBE-P4B I-9.10 clean=H zeroed=N | 这个镜像上没有它判的对象
PROBE-P4B I-5.5 clean=H zeroed=N | 认出了从 mkfs 水位第一次建树的那一版，但它们都没有数据单元（extent 树根全零或一条数据指针都没有）：一版都不判
```
就是说一个文件的 extent 树根被写成零，整套池级 checker 只有 I-5.5 与 I-9.10 的判定变（成立 → 不适用），一条不红：数据单元仍被中央映射条目引用，I-3.1 照样对得上。这是 b35 之前就有的盲区（不是本轮引入），也不是 Q4 问的，只记一笔；要不要补一条判它的不变量是新立条款的事，交主 agent，我不提方案。

### P4-c　I-9.28 与被抛弃的根 I-7.4 两处新说明与判定是否一致：一致

- I-9.28：前缀改成「这几条读得出、自证过的根树表条目没读全（树表指针全零、树表读不出或有条目比字段表短），没进比较：」（「这几条读得出、自证过的根树表条目没读全（树表指针全零、树表读不出或有条目比字段表短），没进比较：」`crates/singlefs-checker/src/walk.rs:5270`），F1 / F4 里 I-9.28 的部分没判点名的根就是树表指针全零的那几条（`PROBE-P2 F4.abandoned_root_all_zero … partial=["I-9.28"]`），用例里「树表指针全零（core 的根记录读者照它读盘 0 槽 0、读不出）」在同一行出现。判定（部分没判）没动，只改了说明的措辞。
- I-7.4（被抛弃的根）：新增一支说明，条件是这条根的树表指针全零（`crates/singlefs-checker/src/walk.rs:7953`「let tree_table_pointer_is_all_zero =」、`:7960`「if tree_table_pointer_is_all_zero {」），红不红没动（`judgements.judge("I-7.4", !walked_into_a_reallocated_or_erased_unit, …)` 的条件没碰）。F4 的原样输出：`F4.abandoned_root_all_zero red=["I-7.4"] partial=["I-9.28"]`，I-7.4 的说明以「被抛弃的根（实例 2、txg 8）走不完：根记录里树表指针全零，core 的根记录读者照它读盘 0 槽 0、读不出（这条根本身损坏，不只是它引用的单元被重新分配或抹头）」起头。一致。
- 与 GPU：被抛弃的根在 GPU 上由 `WALK_PHASE_ABANDONED_ROOTS` 判，只有违例位、没有说明，说明改动不影响对拍。

推翻它的现象：F4 里 I-7.4 红的集合变（红 / 不红）；或另一条非全零的被抛弃根被写成「根记录里树表指针全零」那句。

## P5　35-2：GPU 恢复内核择根

### P5-a　新判法与 core 的 `judged` 五段逐段相同：一致

- core 的五段（`crates/singlefs-core/src/pointer.rs:209` 起的 `judged`）：MAC 与 nonce 全 0（「if !self.is_every_mac_and_nonce_byte_zero {」`pointer.rs:210`）、算法类型 0、压缩算法码 0、压后长度 0、extent 偏移 0（「if self.extent_offset != 0 {」`pointer.rs:234`）。根记录读者对四条指针各调一次这个判法（`crates/singlefs-core/src/root_record.rs:150`、`:158`、`:163`、`:168` 的 `read_judging_the_encryption_compression_and_extent_offset_fields_from`），任一条不过就是这条根不可择。
- GPU：`head_ok` 在 `parse_pointer_common` 里是「p.head_ok = pointer_encryption_and_compression_fields_are_the_first_version(base, o);」（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:541`），函数体 `return all_zero(base, o, 28u) && rd8(base, o + 28u) == 0u && rd8(base, o + 29u) == 0u && rd16(base, o + 30u) == 0u;`（`:518`）= MAC 与 nonce 28 字节、算法类型、压缩算法码、压后长度 2 字节：五段里的前四段；第五段 extent 偏移是 `extent_offset_zero`（「return rd16(base, o + POINTER_EXTENT_OFFSET_FIELD_OFFSET) == 0u;」`:511`）。新谓词「return pointer.head_ok && pointer.extent_offset_zero;」（`:928`）把两样合起来，四条指针各判（`root_record_pointer_heads_are_all_the_first_version`，`:934`），择根处调它（「if (!candidate.valid || !root_record_pointer_heads_are_all_the_first_version(candidate)) {」`:965`）。
- 逐字节量（`probe_p5_core_root_choice_against_the_gpu_rule_byte_by_byte`）：新池新建文件发布的根里，四条指针各翻 0..39 号字节之一（共 160 种，每种重封整槽校验和），问 core 的 `choose_root` 还择不择这条根，与「按 WGSL 谓词仿写：四条指针各自头部 34 字节全 0」比。原样末行：
  ```
  PROBE-P5 total=160 agree=160 disagree=0 core_rejects=136 gpu_rule_rejects=136 disagreements=[]
  ```
  core 拒的恰是每条指针的 0..33 号字节（4 × 34 = 136），34 号及以后（出生树、出生 txg、位置条目）翻了照择（24 种）。注意这是我照 WGSL 文本在 Rust 里写的仿写，不是 GPU 跑出来的：GPU 一侧「推的」。
- 全零指针（树表 0 条一版的分配记录树根、mkfs 的映射根）：`head_ok` 与 `extent_offset_zero` 在全零上都成立，core 同（全零读出来五段都过）；翻其中一个头部字节就不再是全零、同样被拒（探针里 `allocation_root` 原本全零，字节 0 / 28 / 29 / 30 / 32 都 `core_takes_publish_root=false`）。

推翻它的现象：对拍用例（`gpu_recovery_root_choice_refuses_root_record_pointer_head_fields_like_the_core.rs`）在显卡上 GPU 与 CPU 在某状态上红位或恢复落点不同；或探针里出现 `disagree` 非 0。

### P5-b　四条新用例造的根：core 真不择；GPU 改前真择（推的）

四条用例翻的是：树表指针 MAC 第 0 字节（根记录偏移 36 + 0）、中央映射树根指针算法类型（256 + 28）、分配记录树根指针压缩算法码（342 + 29）、实例表指针压后长度（170 + 30，压缩算法码仍 0）。这四个位置都在上面探针的 160 种里：`core_takes_publish_root=false`（原样行：`PROBE-P5 tree_table byte 0 … core_takes_publish_root=false`、`mapping_root byte 28 …`、`allocation_root byte 29 …`、`instance_table byte 30 …`）。用例本身在取卡之前也断言「改之前每个写都落了时 core 择到发布的根、改之后退到更早的那条」。
GPU 改前（谓词只有 extent 偏移）：那四个字节都不在 extent 偏移（32、33 号）里，旧谓词「return pointer.extent_offset_zero;」（`crates/mutations.tsv:2953`）会择，所以 GPU 与 CPU 在发布的根落了的状态上落点不同，用例红；这一句是读码推的。
推翻它的现象：GPU 在旧谓词下对这四条流照样与 CPU 一致（那就是用例没有区分力）；变异行 2953–2956 去掉 `head_ok &&` 之后对应用例不红。

### P5-c　别的 GPU 恢复内核读根记录的入口还只判 extent 偏移吗：没有

`parse_root` 只有两个调用点：恢复内核择根 `choose_root`（本轮改的）与池级 checker 的 `checker_collect_roots`（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:277`），后一处按设计不判头部（它读的是池级 checker 的根环读法，红 I-2.4 的是走读跟随那几条指针的地方，`crash_judge_common.wgsl` 里那段注释写明）。记录一侧 `parse_record` 对新根段四条指针已各判五段（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:1017`「return parse_node_pointer(base, o).head_ok && rd16(base, o + 32u) == 0u;」，调用在 `:1045`、`:1048`、`:1049`，偏移 99、185 与 99 + 188、99 + 274）。旧名字 `root_record_pointers_carry_no_extent_offset` 在整个 `crates/` 里 0 处残留（`grep -rn … | wc -l` 输出 0）。

## P6　35-3

### P6-a　B4 新取样点之后还有没有存活的窗口变体：b34 点名的三个全红；取样点之间的空档窗口仍存活（有限取样的性质）

取样点（`crates/singlefs-harness/src/model.rs:4335`「const CONTAINER_COUNTS: [u64; 9] = [1, 2, 3, 4, 5, 30, 100, 120, 135];」）折成「最左之外的片数」是 0、1、2、3、4、29、99、119、134。`tools/b4_windows2.py` 把变异的窗口换成 [lo, hi]（该窗口内回退只取回前 3 片），原样输出（`results/b4-windows2.log`，第一条 panic 的位置与消息头截到 200 字）：
```
窗口 6..=58: exit=101 红（抓到） | … 现行 1 片、目标 30 片：回退写出的那一版最左那片之外的叶容器要恰好是目标那一版的 …
窗口 5..=98: exit=101 红（抓到） | … 现行 1 片、目标 30 片：…
窗口 100..=133: exit=101 红（抓到） | … 现行 1 片、目标 120 片：…
窗口 5..=28: exit=0 绿（存活）
窗口 30..=98: exit=0 绿（存活）
窗口 100..=118: exit=0 绿（存活）
窗口 120..=133: exit=0 绿（存活）
```
b34 判决点名的三个存活窗口（6–58、5–98、100–133）都红在断言上（`assertion `left == right` failed`），不是越界。下面四个空档窗口仍存活：九个点之间任何一段都放得下一个只在这一段触发的变异，有限取样只能追阈值型的错（「片数 ≥ T 才错」对任何 T ≤ 134 都被 134 那一点抓，「片数 ≤ T」被 0 那一点抓），追不了两头封口的窗口型错。`.claude/rules/mutation-sampling.md`「第三类不许当等价变异留档」的处置是补一个敏感的取样点，补完又会有新的空档，所以这一格我判：b34 点名的三个窗口已关；空档窗口属取样的固有限度，条款没写要关到哪一步，不是本轮改动的缺陷。三行 B4 变异（`crates/mutations.tsv:2957`–`:2959`）在副本上逐行施加：都红在断言上，见 P7。
推翻它的现象：一个 b34 点名的窗口之外、而又是真实缺陷形态（回退取回的片数按位置而不是片数错）的变异在这组取样点上存活；或三个点名窗口里有一个绿。

### P6-b　新池新建文件那条用例里 I-5.4 与 I-3.10 逐状态同集（35-1 改了认法之后）：站得住

- 两条认这一版用的是同一个函数：I-5.4 直接调「version_the_next_mount_applies_first(&journal_records_by_device, &roots, newest_index);」（`crates/singlefs-checker/src/walk.rs:10974`），I-3.10 经「tree_table_pointer_of_the_version_the_next_mount_applies_first(」（`crates/singlefs-checker/src/walk.rs:11023`）调到同一个函数，FL 的末条条件在那里一处（`walk.rs:5614`），对两条同时生效。
- 这条流上每次发布都是一条记录、带末条标志（用例自己的注释「w1、w4、t9 各是一次只有一条记录的发布（序号 1、带末条标志）」`crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs:223`），FL 的末条条件在这条流上恒真，认法与改之前相同。
- 近似枚举（`probe_p6_new_pool_stream_i54_and_i310_evaluated_sets`，段写数 `[2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2]`，段长不超过 8 的逐子集、24 个单元写逐前缀）原样：
  ```
  PROBE-P6 states=60 root_write_index=38 disagreements=0 tally={"i54=H/i310=H": 10, "i54=N/i310=N": 50}
  ```
  60 个状态里 I-5.4 与 I-3.10 评估与否逐状态相同（10 个都成立，50 个都不适用）；I-5.4 在这条流上一个状态都不报部分没判，印证用例注释论证的 ③（第 0 版树表读得出、0 条、根记录里分配记录树根全零）。24 个单元写自成一段、排在记录段与根槽段之前，也与注释 ② 的段次序相同。
- 用例里的列表：`only_under_the_new_root` 现有 25 项（我用 `python3` 数出来 25）、与注释「那 25 条只在这些状态上评估」（`crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs:98`）对得上；I-5.4 挪进与 I-3.10 同一类（「let under_the_new_root_or_through_the_record_the_next_mount_applies_first = ["I-3.10", "I-5.4"];」`:181`）。
- 近似的地方：我的枚举没有层 0 的原地覆写三态与撕裂，「同集」在层 0 真数上的验证在 checker 档，`states_judging_allocation_generations` 与 `GOLDEN_TALLY` 里 I-5.4 的数我对不上、也核不动（要跑 checker 档）。
推翻它的现象：层 0 快档里 I-5.4 评估过的状态数不等于 I-3.10 的；或某个状态上一条评估一条不评估。

### P6-c　829 行注释：一致

改的是「打的行与进度文件内容此后又重取过几次，经过与今天这组钉值取自哪一版见下。」（`crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs:829`）。下面确有「钉值的来历：头一版取自加分片之前的代码…」（`:831`）起的逐次重取记录与「2026-10-06 删除线第 24 批合入之后…重取」那段，所以「见下」指得到。钉值本身（`GOLDEN_PRINTED_LINES_SHA256`）这一轮没动；它与第 34 批 RV 之后 I-5.4 评估数的关系我核不动（要 checker 档）。

## P7　变异行：17 行追加、4 行替换

### P7-a　锚点各恰好命中一次、整行与改动全文逐字相同

`tools/select_rows35.py`（读冻结副本的 `crates/mutations.tsv` 与改动全文 `research/prompts/_m3-formal-code-b35-r1-diff.md`）对 21 行逐行核：整行名字在表里恰好出现一次、与改动全文那一行逐字相同（`same`）、锚点在源文件里恰好命中 1 次（`hits=1`）。原样输出的 21 行里，每行都是 `same … hits=1`（`results/rows.json` 里同样），行号：追加 17 行在 `crates/mutations.tsv` 的 2943–2952、2953–2956、2957–2959；替换 4 行在 955、965、2847、2655。档：harness 13 行、checker-tier 8 行（35-1 的 3 行 GPU、35-2 的 4 行追加加 1 行替换）。

### P7-b　点名的测试跑得到、合入后基线绿

13 行 harness 档点名的测试（11 个不同的 `cargo test` 参数串，B4 三行共用一条）逐条在未变异的副本上跑一遍，每条都 `test result: ok. 1 passed`（`results/baseline-rows.out`）：
```
rows [0]: … a_stale_first_record_of_the_publish_the_next_mount_applies_first_… -> ['test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 135 filtered out; finished in 27.21s', 'exit=0']
rows [18, 19, 20]: … --lib -- model::tests::a_rollback_takes_exactly_the_inode_leaf_containers -> ['test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 86 filtered out; finished in 0.01s', 'exit=0']
```
（全文 11 行见 `results/baseline-rows.out`。）
8 条 checker-tier 档行：锚点命中 1 次已核；点名的测试函数在对应测试文件里各恰好定义一次（行 2950–2952 的三条在 `crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`，2953–2956 在新文件 `gpu_recovery_root_choice_refuses_root_record_pointer_head_fields_like_the_core.rs`，2655 在原对拍文件；用 `grep` 数函数定义，各 1），参数串里的 `--features verdict-store,gpu` 与别的 GPU 行同形，参数最后一段与第六列测试名一致（2953–2956、2655 没有 `--exact`，按子串过滤，名字各自唯一）。跑它们要显卡与 checker 档：红不红「推的」。

### P7-c　红在断言上，不在越界或编译上：13 行 harness 档全是

`tools/mutate_rows2.py` 逐行施加、跑、还原（还原后核 sha256，0 行不对；`restore=ok`），原样汇总（k 是 `rows.json` 里的序号，下标列是 `mutations.tsv` 行号）：
```
k  行号  exit kind                     named_test_failed Compiling  restore | 第一条 panic 头
0  2943  101  断言（或自带消息 panic）  True   True  ok | assertion `left == right` failed: 第一条记录写旧了也不许把 B 的两条记录罩住同一个槽藏起来：只红 I-5.4…
1  2944  101  同上                      True   True  ok | assertion `left == right` failed: B 只由记录施加出来、第一条记录写旧了：…
2  2945  101  同上                      True   True  ok | assertion `left == right` failed: (1, 5) 那条记录里树表指针全零：…
3  2946  101  同上                      True   True  ok | assertion `left == right` failed: B 那条记录里树表指针全零：重放施加出来的那一版照 core 跟过去走不完：…
4  2947  101  同上                      True   True  ok | assertion `left == right` failed: 认出了从 mkfs 水位第一次建树的那一版而它没有数据单元：I-5.5 报这一句专门理由 …
5  2948  101  同上                      True   True  ok | I-7.4 说明写明是这条根自己的树表指针全零：被抛弃的根（实例 2、txg 8）引用的单元已被重新分配或抹头…
6  2949  101  同上                      True   True  ok | I-9.28 部分没判的前缀把树表指针全零的根也说得通：…树表指针非零而条目没读全…
10  955  101  同上                      True   True  ok | assertion `left == right` failed: 两条记录罩住同一个槽，只该红 I-5.4：[] left: [] right: ["I-5.4"]
11  965  101  同上                      True   True  ok | assertion `left == right` failed: 写行那次发布之后：与登记的判定不符 …
12 2847  101  同上                      True   True  ok | assertion `left == right` failed: 最新那条根的树表指针全零：照 core 跟过去读不出，不再一条都不红：…
18 2957  101  同上                      True   True  ok | assertion `left == right` failed: 现行 1 片、目标 30 片：…
19 2958  101  同上                      True   True  ok | assertion `left == right` failed: 现行 1 片、目标 30 片：…
20 2959  101  同上                      True   True  ok | assertion `left == right` failed: 现行 1 片、目标 120 片：…
```
13 行全红在断言（行 5、6 的 panic 是用例自带消息的 `assert!`，消息头就是新说明的断言），点名的测试都是失败的那一条（`named_test_failed=True`），编译通过（日志里 `Compiling`），没有一行是 `index out of bounds` 一类的越界。完整原样输出在 `results/mut-results.jsonl`。
B4 三行（2957–2959）的窗口分别是 6–58、5–98、100–133 片，由行 18 / 19 / 20 的 panic 头看，红的是目标 30 片（前两行）与目标 120 片（后一行）的那一对：与取样点 30、120 各罩一个窗口的设计相符。

### P7-d　955、965 换 ALT1 之后判的还是原来那条判：是

两行的替换文都是「let mut allocation_roots = vec![parse_node_pointer(&[0u8; 86])];」，把根记录持有的那棵分配记录树根换成全零指针（该根跳过、不判互不相交），而长度为 1 的向量仍在、不再越界（b34 里原替换让 `allocation_roots[0]` 越界 panic）。它们点名的测试判的还是那条判：
- 955：`overlapping_records_in_the_allocation_record_tree_the_root_record_holds_redden_only_the_disjointness_invariant`，用例断言「两条记录罩住同一个槽，只该红 I-5.4」，变异后 `left: [] right: ["I-5.4"]`（红 I-5.4 的那一红没了）；
- 965：`a_formatted_pool_mounted_twice_writes_the_row_then_the_first_file_and_reads_it_back_cold`，用例登记 I-5.4 真被评估过且成立，变异后实际是「候选集里没有一条根的分配记录树走得到叶…」的不适用（红在「与登记的判定不符」上）。
- 2847：`an_all_zero_tree_table_pointer_in_the_newest_root_is_followed_like_the_core_…`，变异让主走读把根记录里全零的树表指针读成「没有树表」，最新根那一红没了（这行不是 ALT1，是 34-1 那行的替换）。
推翻它的现象：把 ALT1 换回 b34 的原替换文，同一条测试红在越界 panic 上（那一条是 b34 判决里测出的）；或 ALT1 之下那两个用例里有一个绿。

## 补：上面没单列推翻条件的几格

- P2-c（kb 三处与实现冲突）：推翻它的现象是 kb 那三行被改过之后与实现一致（写回批做完），或实现回退到「由记录施加的那一版不记走读失败」「GPU 不跟全零」「认出来而没有数据单元落缺省理由」；这一格是两边并排读出的差，不依赖我的判断。
- P4-b（坏镜像是这一形；0 红的观察）：推翻它的现象是同一张镜像上 I-5.5 的结局不是新句子，或逐条比出来除 I-5.5、I-9.10 之外还有变的（我这一次比的是干净镜像与这一张镜像，一次抽样）。
- P6-c（829 行注释）：推翻它的现象是 829 行之后找不到逐次重取的记录。
- P7-a（锚点）：推翻它的现象是 `mutations.tsv` 在此之后被改过（冻结副本之后）而锚点命中次数变了；我核的是冻结副本。
- P7-b、P7-c（基线、红在断言）：推翻它的现象是同一条命令在别的机器或别的编译目录里基线红，或变异行在合入后的树上红在越界；我跑的是冻结副本，每行施加前后核了 sha256。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| P1-a 取的新根段与 core 施加的同一份 | 一致 | core 在带末条标志那一条上换根，checker 三处都取带标志那条；64 子集 × 两种坏法：core 施加 B 的 27 个子集上 checker 与 core 同向，FL 关时 27 + 60 个子集分开 |
| P1-b 认哪些状态、末条没落时判什么 | 条款没写 / 冲突 | 条款 I-3.10、I-5.4 没写「带末条标志」，实现写了（并排两行）；core 不施加 B 的 37 个子集里 checker 仍认 21 个（FL 关时 36 个），发布边界不复刻是实现已知限度 |
| P1-c 合法历史多报少判 | 没打中 | H2 102、H3 191、新池流 60 个状态 FL 开 / 关逐行相同、0 红；136 条 harness 用例绿；随机历史、层 0 没跑 |
| P1-d 退路 | 一致（合法）/ 同 C738（坏写者） | 末条读不到才走，合法写者抹掉末条不红，首条写旧时退路走旧树红 I-3.1、I-3.5 |
| P1-e GPU 两处与 CPU 同组同取法 | 一致（读码，推的） | 条件、去重、次序对得上；去重键只有计数器、CPU 键含实例是改动之前就有的 |
| P2-a 红哪几条、同一结局 | 一致（版本自己有单元时） | 由记录施加那一版与带自己单元的候选根都红 {I-2.1, I-3.1, I-4.8, I-7.4}；最新根多红 I-7.2、I-3.1 不适用；下一次挂载那一版只红 I-4.1 |
| P2-b 下一次挂载那一版红 I-4.1 | 一致 | I-4.1 是唯一射程写着「重放之后那一版」的一条；core 写者写不出全零树表指针，合法历史 0 红 |
| P2-c 四处部分没判 / 判不了的处置 | 一致；kb 三处冲突 | I-9.28 两句（由记录施加那一版「不记走读失败」、GPU「CPU 红 GPU 不红」）与 I-5.5「仍落缺省理由」都与实现冲突 |
| P3-a GPU 全零分支红位 | 一致（读码，推的） | 两份 I-2.1、一次失败、不判 I-2.5、映射根不走；六个阶段共用一个调用点，replay 与被抛弃阶段不判、只由失败翻红位 |
| P3-b 四条新对拍流的构造 | CPU 一侧一致，GPU 一侧推的 | 我在 harness 里同搭法重做，CPU 红集合与用例注释 / 断言对得上；用例二进制本身没编没跑 |
| P4-a 新理由分支条件、与另两句分不分得开 | 一致 | 认出来且一版都没判到才走，不压过违例 / 部分没判 / 成立；mkfs_only、extent_root_zeroed、干净镜像三种结局三个说法 |
| P4-b 坏镜像是不是这一形 | 一致 | 认法 ①② 没动、只有 extent 根写零；顺带记：整套 checker 在这张镜像上只有 I-5.5 与 I-9.10 变、0 红 |
| P4-c I-9.28、I-7.4 两处说明 | 一致 | 判定没动，只改说明；F4 红 {I-7.4}、部分没判 {I-9.28}，说明点名树表指针全零 |
| P5-a GPU 择根与 core `judged` 五段 | 一致 | 160 种单字节翻转：core 与 WGSL 谓词仿写 160 / 160 相同（136 拒、24 收） |
| P5-b 四条用例的根 | 一致（GPU 改前择，推的） | 四个位置都在 core 拒的 136 里；旧谓词只判 32、33 号字节 |
| P5-c 别的入口只判 extent 偏移 | 没有 | `parse_root` 两个调用点、`parse_record` 已各判五段；旧名字 0 处残留 |
| P6-a B4 存活窗口 | 点名三个已关；空档窗口仍存活 | 6–58、5–98、100–133 红在断言；5–28、30–98、100–118、120–133 仍绿，有限取样的固有限度 |
| P6-b I-5.4 与 I-3.10 逐状态同集 | 一致（近似枚举） | 同一个函数认版本；60 个状态同集、0 分歧；层 0 真数与 GOLDEN_TALLY 核不动 |
| P6-c 829 行注释 | 一致 | 「见下」指得到 |
| P7-a 锚点 / 整行 | 一致 | 21 行 `same`、`hits=1` |
| P7-b 点名的测试 / 基线 | 一致（harness 档）/ 推的（GPU 档） | 13 条 harness 测试基线各 1 过；8 条 GPU 行测试函数各定义一次 |
| P7-c 红在断言上 | 一致（harness 档 13 行） | 13 行 `exit=101`、红在断言、点名测试失败、编译过；GPU 8 行没跑 |
| P7-d ALT1 判的还是原来那条 | 一致 | 955、965 都是把根记录持有的树跳过，点名用例红在 I-5.4 评估与否上，不越界 |

## 没做什么

- 没跑任何重型测试：checker 档（`singlefs-checker-tier` 的测试与 `crash_enumeration_*`）、GPU 对拍 8 条用例（35-1 的 4 条、35-2 的 4 条）、8 行 GPU 变异、`mutations.tsv` 整表、层 0 快档、`GOLDEN_TALLY` 钉值、WGSL 在显卡上的编译与校验（没有 naga 一类的离线校验工具在手）。涉及它们的结论都写了「推的」或「核不动」。要主 agent 跑的：提交时 checker 档整套，其中 35-1 的对拍用例 4 条（`crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 里名字带 `stale_first` / `all_zero_tree_table_pointer` 的）、35-2 新文件 4 条与它们对应的 `mutations.tsv` 2950–2956、2655 行，以及 `crash_enumeration_new_pool_file_creation_stream.rs`、`crash_enumeration_sharded_across_processes.rs`（钉值）。
- 没有写探针以外的任何仓内文件；没提交、没改 kb、没改 `mutations.tsv`。kb 与实现的冲突（P1-b、P2-c）只并排列出，写回哪一句、怎么改交主 agent。
- 随机历史没做（攻方的活）。崩溃状态枚举是我自己的近似（见〇），没有层 0 的原地覆写三态与撕裂；对拍流的 GPU 一侧只读码。
- 「有没有更好的不变量判全零树表指针」「extent 树根写零 0 红」是新判法 / 新条款，条款没写、按正文第五节交主 agent 与用户，不由我定。
- `tools/rerun.sh` 的阶段是我把已经手跑过的命令串起来写的，没有整条跑过一遍（每个阶段里的命令各跑过；`probes` 阶段里新加的 `flow4`、`P4B` 两条探针见 README）。第一次跑 `flow4` 时内存包装排不上（退出码 252：别的会话占着 slice，那一次输出不算结果）；之后改为 2G 上限、`RUN_WITH_MEMORY_CAP_WAIT_SECONDS=200` 重跑，正常。
- 临时目录清理：交回前删了自己建的仓副本 `/tmp/claude-1000/m3-formal-code-b35-r1-sonnet/work`（20M）与两个编译目录 `/tmp/claude-1000/m3-formal-code-b35-r1-sonnet/target`（2.8G，debug，基线与变异行用）、`/tmp/claude-1000/m3-formal-code-b35-r1-sonnet/target-opt`（1.5G，opt-level 2，探针用）；没有不删的。草稿目录 `/tmp/claude-1000/m3-formal-code-b35-r1-sonnet/` 里只剩日志（已全部拷进 `research/prompts/m3-formal-code-b35-r1-sonnet-model/results/`）。
