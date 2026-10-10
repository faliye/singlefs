# m3-formal-code-b21-r1 云端正推腿报告（Sonnet，格 K1、K3）

日期 2026-10-04。代码引文读冻结树 `/tmp/claude-1000/m3-formal-code-b21-r1/tree/`（报告里写树内路径 `crates/…`）；kb 引文写 kb 文件自己的行号。
量的东西全在草稿目录里的副本上做（冻结树 `crates/` 的拷贝，仓内文件一个没碰）；模型、脚本、原样输出与副本改动全文在
`research/prompts/m3-formal-code-b21-r1-sonnet-model/`（`README.md` 有对照表与起法）。线程上限按派发提示（后改 7），测试二进制全经内存包装。
checker 档（`singlefs-checker-tier`）一个没跑，凡落在 `crash_injection.rs` 上的结论都标「推的」。

## 一、K1

### K1-a　H2 用例（`the_first_file_after_nine_warm_up_only_mounts_lands_past_…_one_txg_short_of_reclaim`）

**判定：一致（用例做的就是判决说的）；附一条脆处（常数 9 绑死每区槽数 S = 8），不构成「该绿而红」。**

**1. 实现今天的回收门槛怎么算。**
- 回收谓词：「.filter(|record| record.is_released && record.generation <= floor)」（`crates/singlefs-core/src/allocator.rs:1736`）。
- 门槛 floor 在挂载之内由 `record_root_written_by_this_process` 按「max(F_生效, 一圈窗口里最旧有效根)」算：「let reclaim_floor = crate::mount::reclaim_floor(」（`crates/singlefs-core/src/allocator.rs:1834`），
  `reclaim_floor` 本体：「oldest_valid_root.map_or(effective_floor, |oldest| effective_floor.max(oldest))」（`crates/singlefs-core/src/mount.rs:1351`）。
- 窗口：「checkpoint_txg」「.saturating_add(ROOT_RING_REGIONS * slots_per_region.count())」「> window_anchor.0」（`crates/singlefs-core/src/root_ring.rs:128-131`）。即 txg + 3S > N（N 是进程记到的最新根）。
- kb 条款（D3 已定项 7 的回收谓词）：「已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)」（`.claude/kb/decisions/03-空间分配.md:133` 同一行里；该行在「#### 已定项 7：分配记录条目的盘上编码，与分配准入合取的登记表」一节内，节从 120 行起），
  代码这一行与它同形。I-5.5 行写明回收之后落点不再是 U + 4：「回收之后（例如 mkfs 的实例表 U、U + 1 那两槽被回收）不再是 U + 4」（`.claude/kb/invariants.md:199`）。
- 推出的翻转点：每次只暖机挂载消耗 3 个 txg，第 k 次挂载上第一个文件 txg = 3k − 1，写它之前 N = 3k − 2；mkfs 的实例表那一对释放代 3，
  正确实现要 floor ≥ 3，即 N − 3S + 1 ≥ 3，k ≥ S + 2；放宽 1 个 txg（floor + 1 ≥ 3）k ≥ S + 1。S = 8 时分别是 10 与 9，与用例常数对上：
  「const WARM_UP_ONLY_MOUNTS_UNTIL_THE_MKFS_INSTANCE_TABLE_IS_RECLAIMED: usize = 10;」（`crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs:60`），
  新常数「const WARM_UP_ONLY_MOUNTS_ONE_SHORT_OF_RECLAIMING_THE_MKFS_INSTANCE_TABLE: usize =」（同文件 `:64`）= 10 − 1 = 9。

**2. 量：正确实现绿、放宽红。**
副本里把 `allocator.rs` 那一行改成读环境变量 `B21_SLACK` 的「generation.0 <= floor.0 + slack」（改动全文见模型目录 patch），H2 用例原样跑四次（`driver-h2.sh`，输出 `research/prompts/m3-formal-code-b21-r1-sonnet-model/out/h2.log`）：
- slack 0：「test the_first_file_after_nine_warm_up_only_mounts_lands_past_the_mkfs_instance_table_that_is_one_txg_short_of_reclaim ... ok」（`research/prompts/m3-formal-code-b21-r1-sonnet-model/out/h2.log:2`）。
- slack 1：「test the_first_file_after_nine_warm_up_only_mounts_lands_past_the_mkfs_instance_table_that_is_one_txg_short_of_reclaim ... FAILED」（`research/prompts/m3-formal-code-b21-r1-sonnet-model/out/h2.log:5`），
  左右值「  left: [(0, 50176), (1, 50176)]」「 right: [(0, 50180), (1, 50180)]」（`research/prompts/m3-formal-code-b21-r1-sonnet-model/out/h2.log:7-8`）：红在数据单元落点那一条（panic 位置 `pool_checker_judges_….rs:579`，冻结树里 `assert_eq!` 起头的下一行是 `:580`）。
- slack 2、5 同样红在同一条（`out/h2.log` 第 11–14、17–20 行，同样的左右值）。

**3. 别的历史长度、别的环几何上会不会「该绿而红」。**
用例本身只跑一段写死的历史（`parameters()` 的默认 768 MiB 环、S = 8、9 次挂载），没有随长度或几何变的输入，所以同一个用例不会在别处红。我另做了扫描（`b21_scan.rs`，
每个 (S, 环长, slack, k) 起一个新池、只暖机挂 k 次、写第一个文件，记 txg、数据单元相对 U 的槽号、I-5.5 与池级红条），翻转点（U + 4 → U）如下（全表在 `out/` 下三份 scan 日志）：

| 每区槽数 S（环长） | 正确实现翻转 k | 放宽 1 个 txg 翻转 k | 日志行 |
|---|---|---|---|
| 8（默认 768 MiB） | 10（k = 9 仍 U + 4） | 9 | 「ROW slack=0 journal=805306368 S=8 k=9 txg=26 data_off_U=[(0, 4), (1, 4)] I55=Holds red=[]」（`research/prompts/m3-formal-code-b21-r1-sonnet-model/out/scan-default-S8.log:3`）；「ROW slack=1 journal=805306368 S=8 k=9 txg=26 data_off_U=[(0, 0), (1, 0)] I55=Holds red=[]」（同文件 `:7`） |
| 8（128 MiB） | 10 | 9 | 「ROW slack=0 journal=134217728 S=8 k=10 txg=29 data_off_U=[(0, 0), (1, 0)] I55=Holds red=[]」（`research/prompts/m3-formal-code-b21-r1-sonnet-model/out/scan-j128-S5to8.log:15`） |
| 5 | 7 | 6 | 「ROW slack=0 journal=134217728 S=5 k=7 txg=20 data_off_U=[(0, 0), (1, 0)] I55=Holds red=[]」（同文件 `:3`） |
| 16 | 18 | 17 | 「ROW slack=0 journal=134217728 S=16 k=18 txg=53 data_off_U=[(0, 0), (1, 0)] I55=Holds red=[]」（`research/prompts/m3-formal-code-b21-r1-sonnet-model/out/scan-j128-S1216.log:7`） |

S = 5、6、7、8、12、16 六个值上正确实现的翻转点都是 k = S + 2，放宽 1 个 txg 的都是 k = S + 1（每个 S 扫 k = S .. S + 3，共 64 + 16 + 8 行，日志里各有）。
推论：
- 在 S = 8 上，k = 9 恰是「正确实现 U + 4、放宽 1 个 txg 即 U」的那一格，也是唯一一格（k ≤ 8 两边都 U + 4，k ≥ 10 两边都 U）。用例选得准。
- 环长（默认 / 128 MiB）不改翻转点，只改 U 的绝对值（用例用 `genesis.unit_area_start` 现取，不写死）。
- 脆处：常数 9、10 不是从 S 推的。`S` 在格式里是占位常数：「pub const ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM: u64 = 8;」（`crates/singlefs-format/src/lib.rs:279`）。
  S 改小（≤ 7）用例在**正确实现**上红（k = 9 ≥ S + 2），改大（≥ 9）用例仍绿但不再分辨放宽（k = 9 < S + 1）。这一格的原用例（k = 10）同样绑 S，不是新引入的；写「推的」：S 真改时两条用例都会一起响，不会静默。

**4. 副发现（与 C689 对得上）。** 放宽 1 个 txg 时池级 checker 全绿（扫描日志里 slack=1 的 16 行 `red=[]`、`I55=Holds`）；放宽 2 个以上才有别的不变量红：
「ROW slack=2 journal=134217728 S=8 k=9 txg=26 data_off_U=[(0, 0), (1, 0)] I55=Holds red=["I-3.1"]」（`research/prompts/m3-formal-code-b21-r1-sonnet-model/out/scan-j128-S5to8.log:46`）。
checks-owed C689 写的是：「回收门槛放宽一个 txg 的坏实现（攻方称 H2，mkfs、可写挂载 9 次再写第一个文件，正确落 U + 4、H2 落 U）在最终镜像上 I-5.5 判成立、池级 checker 一条不红」（`.claude/kb/checks-owed.md:546`，同一行里），
扫描复现了它：放宽 1 个 txg 只有 H2 用例（钉实现落点）抓得到。

**推翻条件**：
- 在默认环、S = 8 上，正确实现跑 9 次只暖机挂载后第一个文件落 U（不是 U + 4）；或放宽 1 个 txg 的实现仍落 U + 4——都会推翻「用例恰在边界上」。我量到的是 U + 4 / U（`out/h2.log`、scan 日志）。
- 另一种放宽法（改窗口 `txg + 3S > N` 的比较、而非改 `<= floor`）也会在 k = 9 提早回收，用例按落点钉，同样抓；这一条我没量（推的）。
- 取等号那一格（`<=` 改 `<`）：H2 历史里 floor 只取 2 与 5（跳 3 个 txg），碰不到 floor = 3，H2 不管它；形 B（零单元发布每次 +1 txg，翻转 21 → 22）会碰到，推的、没量，不在本格。

### K1-b　六张 I-9.27 坏镜像（第 22–27 张，种类 4、6、7、8、11、12）

**判定：一致（六张各只红 I-9.27、改动只落在点名那一种条目的头 ID 上、六种「不归 Zero」各由自己那一张抓住）。**

**1. 条款与实现。** I-9.27 的取值：「种类 3、4、6、7、8、11、12 的条目等于 0」（`.claude/kb/invariants.md:314`，同一行里）。
实现里这七种归 Zero，别的归 NotJudged：「| TREE_KIND_LOGGED_OPERATIONS => Self::Zero,」（`crates/singlefs-checker/src/walk.rs:3442`）、「_ => Self::NotJudged,」（`crates/singlefs-checker/src/walk.rs:3443`）、
NotJudged 一臂在判里是跳过：「HeadIdentifierOfATreeTableEntry::NotJudged => continue,」（`crates/singlefs-checker/src/walk.rs:3560`）。
所以「种类 X 不归 Zero 那一臂」= 那一种落进 `_`，其头 ID 改成什么都不判，正是六行变异的语义。
用例这一边：新增六种由常量数组给出，每种一张，改法照种类 3 那张（头 ID 0 → inode 树条目的树 ID）：
「images.extend(」（`crates/singlefs-harness/tests/checker_known_bad_images.rs:6510`）、
「const TREE_KINDS_WITH_A_ZERO_HEAD_IDENTIFIER_BESIDES_THE_ALLOCATION_RECORD_TREE: [u16; 6] = [」（`crates/singlefs-harness/tests/checker_known_bad_images.rs:6531`）；
改写函数先核「本来的取值」再改，且种类恰好一条：「let [entry] = entries_of_the_kind.as_slice() else {」（`crates/singlefs-harness/tests/checker_known_bad_images.rs:6561`）。

**2. 量一：28 张坏镜像逐张与干净镜像比（`b21_probe_bad_images`，`driver-k1b.sh`，`out/k1b.log`）。** 第 22–27 张判红清单都只有 `I-9.27`，
树表单元（槽 50252，两块盘各一份）上变的字节只有那一条条目的头 ID（条目内偏移 116、118：0 → 65537 = 0x10001 的两个非零字节）加单元自己的重封校验和（单元头 10–13、92–95），根槽改了 1 处（补新校验和）：
- 「PROBE img=22 inv=I-9.27 violated=["I-9.27"] i927_detail=Some("根（实例 1、txg 3）的树表：第 3 条（树 ID 65539、种类 4）的头 ID」（`research/prompts/m3-formal-code-b21-r1-sonnet-model/out/k1b.log:80`）与其下一行「PROBE   diff slot 50252 dev 0: hdr+10,hdr+11,hdr+12,hdr+13,hdr+92,hdr+93,hdr+94,hdr+95,e3+116,e3+118」（`research/prompts/m3-formal-code-b21-r1-sonnet-model/out/k1b.log:81`）；
- 第 23–27 张依次落在 e4（种类 6）、e5（7）、e6（8）、e9（11）、e10（12）：`out/k1b.log` 第 83–96 行（逐张的 `PROBE img=` 行与 `diff` 行）。
- 六张之外，别的条目的字节一个没动；树表 11 条里被 I-9.27 覆盖的种类现在是满的：既有 17（种类 1）、18（2）、19（3）、20（9）、21（10）加这六张，十一种一种一张。
没改的字段不在这一比里的（journal 记录里的新根副本）由 `propagate` 另补，我没逐字节比，写「没量」。

**3. 量二：六种「不归 Zero」各丢一次。** 副本 `walk.rs` 的 `of_tree_kind` 加环境变量钩子 `B21_DROP=<种类码>`，命中那一种就返回 `NotJudged`（语义与 `crates/mutations.tsv` 六行文本替换相同：那一种落进 `_`；
我没有按 tsv 原文替换，原文替换由主 agent 的 prove-red 证过，见背景第二节）。每丢一种，28 张里恰好那一张变成 `violated=[]`，别的 27 张判红清单一个不变；原用例红在恰好那一张上：
- drop 4：「PROBE img=22 inv=I-9.27 violated=[] i927_detail=None root_slots_changed=1」（`research/prompts/m3-formal-code-b21-r1-sonnet-model/out/k1b.log:180`）；原用例「  left: ["第 22 张坏镜像判红的该只有 I-9.27，实际 []：[(\"I-1.1\", Hold」（同文件 `:201`）。
- drop 6、7、8、11、12：`PROBE img=23/24/25/26/27 … violated=[]` 在 `out/k1b.log` 第 284、388、492、596、700 行；原用例红且点名 23/24/25/26/27 号：第 302、403、504、605、706 行。
- 干净（不丢）时：原用例「test each_pointer_journal_and_generation_field_bad_image_reddens_only_its_own_invariant ... ok」（`research/prompts/m3-formal-code-b21-r1-sonnet-model/out/k1b.log:100`）。
- 六个 mutations.tsv 行的「旧串」在冻结树里各恰好命中一次（我用脚本按 `\n` 转义还原后数的，输出 `7`、七行都是 `old count 1`；另一行 H2 变异的旧串也恰好一次，`allocator.rs:1736`），所以主 agent 的变异替换不会因串不唯一而落空。

**推翻条件**：任一变异下红的不是恰好它自己那一张，或某张坏镜像在干净实现上除 I-9.27 外还红别的。我量到的都是「恰好一张」。

### K1-c　原用例改成「判完再一起断言」有没有放宽判定

**判定：一致（没放宽）。**
- 原写法：对清单里每张，`assert_eq!(violated_invariants(&verdicts), [invariant])`，第一张不符就 panic。
- 新写法：逐张判完，把「判红清单 ≠ [自己那一条]」的各张收进 `Vec<String>`，最后断言它为空：「(violated != [invariant]).then(|| {」（`crates/singlefs-harness/tests/checker_known_bad_images.rs:6661`）、
  「images_not_reddening_only_their_own_invariant,」（`crates/singlefs-harness/tests/checker_known_bad_images.rs:6669`）。
- 通过条件两边相同：全部 28 张都 `violated == [invariant]`；失败条件也相同：存在任一张不等。比较式 `Vec<&str> != [&str; 1]` 与 `assert_eq!` 用的是同一个元素相等。
  收集式只多做的事是不在第一张失败时停下，所以失败时多列几张，不会让原本红的变绿。前面干净镜像上「每条 Holds」那一轮没动。
- 量：不丢时绿（`out/k1b.log:100`）；六种各丢一次时各红、且报出的恰好是那一张（见 K1-b 第 3 点）；若新写法把不符的吞掉，这六次不会红。

**推翻条件**：存在一张判红清单 ≠ [自己那一条] 而新用例仍绿。构造不出：断言的是「不符的集合为空」，集合里少一张当且仅当那张相符。

### K1-d　`writable_mount_refusal_on_a_crash_state_without_read_faults.rs` 改过的注释与 `mount_transparency.rs` 逐句对

**判定：逐句对得上（实现一致）；一条措辞偏宽，写成注意，不判冲突。**

改过的两处（冻结树）：文件头第 4–5 行、撞「两槽都撕裂」那条用例的文档注释第 288–289 行。逐句：

| 注释里的句子 | 代码里对应 | 对不对 |
|---|---|---|
| 「某块盘两槽都撕裂那一支（一份自证过的系统配置都没有，C664）」（`crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs:4`，同一行里） | 「/// R2 之下这一形不并进判不出（一块盘两槽都撕裂、另一块盘两槽都自证过时 R2 判得出），实现在读阶段判完之后单列一个成员报它、」（`crates/singlefs-harness/src/mount_transparency.rs:1375`）；判法：「.any(|slots| slots.self_verified.is_empty())」（`crates/singlefs-harness/src/mount_transparency.rs:1383`） | 一致 |
| 「C670 那一条…跳过撕裂的槽复算为假、不报」（`crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs:4-5`；第 288 行的「C670 那一条跳过撕裂的槽复算判据为假」（`crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs:288`）同义） | C670 判官取 c_见证 时「自证不过的槽（撕裂的、没写过的）跳过」（`crates/singlefs-harness/src/mount_transparency.rs:1489`），之后比：「return (witnessed_journal_counter > last_record_counter).then_some(」（`crates/singlefs-harness/src/mount_transparency.rs:1468`） | 对**这份盘面**一致，见下「注意」 |
| 「系统配置见证照 R2 判得出（撕裂不算判不出）」 | 「/// 撕裂的轮换（读得出、自证不过，设备没报错）不算判不出；一块盘报读错而另一块盘两槽都自证过也不算。」（`crates/singlefs-harness/src/mount_transparency.rs:1360`），判不出式：「some_slot_reported_a_read_error && !some_device_self_verified_both_slots」（`crates/singlefs-harness/src/mount_transparency.rs:1371`，要有槽设备报读错） | 一致（内存盘不报读错） |
| 「拒照 U6 判」 | 「/// 那一形的拒照 U6 判。一份自证过的系统配置都择不到时 c_见证 为 0，交回 `None`。」（`crates/singlefs-harness/src/mount_transparency.rs:1492`） | 一致 |
| 「判据按「某块盘一份自证过的都没有」那一支为真，合法」 | 「/// 系统配置见证判不出（[`system_configuration_witness_undecidable_on_the_crash_image`]，R2）⇒ 按真；某块盘一份自证过的系统配置都没有」（`crates/singlefs-harness/src/mount_transparency.rs:1387`）与「/// （[`some_device_without_any_self_verified_system_configuration_slot_on_the_crash_image`]）⇒ 按真；否则全部自证过的槽的 journal tail」（`crates/singlefs-harness/src/mount_transparency.rs:1388`）；用例断言 `Legal(NewerPublishWitnessedBySystemConfiguration)` | 一致（副本里跑该文件四条用例全绿，`out/refusal-probe.log`） |
| 第 288–289 行：「（c_见证 等于第一个文件那一版末条的计数器）」 | 副本探针（`b21_probe_torn_both_slots_witness`）读出：「PROBE2 chosen_slot_journal_tail=3 selected=Some((InstanceGeneration(1), CheckpointTxg(3))) last_record_counter_of_selected=Some(3) max_counter_in_ring=Some(3)」（`research/prompts/m3-formal-code-b21-r1-sonnet-model/out/refusal-probe.log:2`）：c_见证 3 = 所选那一版末条计数器 3，`3 > 3` 为假 → `None` | 一致（这份盘面） |

**注意（措辞，不是代码错）**：文件头第 4–5 行把「某块盘两槽都撕裂那一支」整支写成「C670 那一条……复算为假、不报」。代码对这一支写的是「那一判按真、这里照常比」（`crates/singlefs-harness/src/mount_transparency.rs:1491` 同一行里），
也就是 C670 那一条在这一支上是「照常比」，真假看另一块盘的 journal tail：另一块盘上轮换落在更晚的发布上时 c_见证 大于所选那一版末条计数器，会报。注释说的「为假」只对用例搭的那份盘面（没有更新的轮换）成立。
用例本身（断言 `Legal`）没问题，只是文件头一句读成通则会偏宽。老写法里的「U6 照甲把撕裂判成按真」与 C331 已删，与 `mount_transparency.rs` 现在的「C331 判不出改取 R2」（该文件 `:1263`）一致。

**推翻条件**：注释里任一句与代码对不上。我逐句对过，唯一的差是上面那条通则宽度。

## 二、K3　第四种接错（第三截「所选那一版」交成第一次崩溃那一版）按「支」算还清对不对

**判定：成立，带两个前提（都读码核过；前提 P1 在真函数上量过，P2 与「崩溃枚举域」两条是推的，checker 档没跑）。**
正文第一节第 4 条的推论是：判据对所选那一版单调，换成更旧那一版只会真到真。我把它拆成五步逐步引码。

**步 1　接线点。** 第三截调用判官时交的版本是二次崩溃镜像上只读恢复落到的那一版：
「let report = &fields_read_after_the_second_crash.report;」（`crates/singlefs-checker-tier/src/crash_injection.rs:3002`），
「version_landed_on_after_the_crash: report.effective_root,」（`crates/singlefs-checker-tier/src/crash_injection.rs:3041`）。
变异行把它换成第一次崩溃那份恢复报告的 `effective_root`（该函数入参 `fields_read_after_the_first_crash`，`mutations.tsv` 末行）。
第二截那一处（第一次崩溃的镜像配第一次崩溃的版本）本来就是：「version_landed_on_after_the_crash: self.version_landed_on_after_the_first_crash,」（`crates/singlefs-checker-tier/src/crash_injection.rs:2152`），与这条无关。

**步 2　所选那一版进到判官之后，只影响哪一格。** 判官 `judge_writable_mount_refusal_of_a_crash_state` 对 N-配置拒因：没注入读故障时再加 C670 那一条，注入过读故障时一律不加：
「ReadFaultsInTheCrashState::Injected,」（`crates/singlefs-harness/src/mount_transparency.rs:1653`，与它同臂的 `=> {}` 在 `:1658`）。
第三截的 `read_faults` 沿用第一次崩溃那个态的认定：「read_faults: read_faults_in_the_first_crash_state,」（`crates/singlefs-checker-tier/src/crash_injection.rs:3047`），与所选那一版无关。
第三截那一判只拿 C670 定红绿：「条款只管第二截，这一截不拿它们定红绿」（`crates/singlefs-checker-tier/src/crash_injection.rs:3228`，同一行里）。计数：
- 拒因是 N-配置、判官交回 Legal 或 Illegal，都记一次「判过」：「tally.remount_refusals_for_a_newer_publish_witnessed_by_system_configuration += 1;」（`crates/singlefs-checker-tier/src/crash_injection.rs:3319` 与 `:3326`）；
- 判官列出的没满足的条件里有 C670 那一条才记一次不合法并报红：「if !witness_criterion_true_without_read_faults {」（`crates/singlefs-checker-tier/src/crash_injection.rs:3336`）、「tally.remount_refusals_with_the_witness_criterion_true_without_read_faults += 1;」（`crates/singlefs-checker-tier/src/crash_injection.rs:3339`）。
所以：拒因、读故障认定、「判过」数都与所选那一版无关；计数（判过, 不合法）、红与不红只随「C670 那一条判官有没有交回一支」变，所选那一版只经这一条进来。
（U6 那一条「判据复算为假」另走 `newer_publish_witnessed_on_the_crash_image`，也吃所选那一版，但它的结果在第三截只改 Legal / Illegal 的标签，不改上面两个计数。）

**步 3　C670 那一条对所选那一版怎么判。** `witnessed_counter_holds_against_the_selected_version`（`crates/singlefs-harness/src/mount_transparency.rs:1448`）五步：
1. 「if witnessed_journal_counter == 0 {」（`crates/singlefs-harness/src/mount_transparency.rs:1454`）→ 假；
2. 「let selected_version = selected_version?;」（`crates/singlefs-harness/src/mount_transparency.rs:1457`）→ 没有所选那一版交假；
3. 所选那一版末条记录读得出：「return (witnessed_journal_counter > last_record_counter).then_some(」（`crates/singlefs-harness/src/mount_transparency.rs:1468`）→ c_见证 > 它的计数器才真；
4. 末条读不出而计数器等于 c_见证 的记录读得出：「Some(record) => (record > selected_version).then_some(」（`crates/singlefs-harness/src/mount_transparency.rs:1478`）→ 那条记录的 (实例代号, txg) 大于所选那一版才真；
5. 两条都读不出：「None => Some(」（`crates/singlefs-harness/src/mount_transparency.rs:1481`）→ 按真。

**步 4　单调。** 记 T(V) 为上面五步在所选那一版 V 上的结果，V1 是第一次崩溃那一版、V2 是二次崩溃那一版（V1 不新于 V2）。要证：T(V2) 真 ⇒ T(V1) 真。
- 第 3 步 / 第 3 步：c > L(V2) 且 L(V1) < L(V2)（发布先后即计数器先后）⇒ c > L(V1)。
- V2 末条读不出而 c 处记录读得出且 > V2：它也 > V1，且 c > V1 的末条计数器（那条记录属于更晚的发布，计数器更大）；V1 的末条读不出时走第 4 步，同样真。
- V2 两条都读不出（第 5 步，按真）：V1 若末条读得出就要 c > L(V1)。这一支**代码本身不保证**，要靠两条前提：c 是某次发布的末条计数器（P1a），且 V1 之前的发布的记录都读得出（P1b）；那么 c ≥ V1 前一次发布的末条计数器，
  c 若小于 L(V1) 就落在 V1 之前、那一处记录读得出（第 4 步给假，到不了第 5 步）。
前提的出处：
- P1a：系统配置里的 tail 存的是「这次发布末条记录的计数器」：「// 系统配置里的 tail 存这次发布末条记录的 jsn 计数器（D23（journal 的角色与格式） 已定项 18）。」（`crates/singlefs-core/src/transaction.rs:1977`）；取号那一写带照抄见证到的最大 tail（`transaction.rs:930` 一带），也是某次发布的末条计数器。
- P1b：崩溃态只丢当前段里没持久的写，更早的段整段持久：「// 与整条挂载流逐条对应：更早的段整段持久、当前段按二次崩溃的子集、更晚的段一个都没持久。」（`crates/singlefs-checker-tier/src/crash_injection.rs:2983`）；第一次崩溃同一个枚举域（`crash_injection.rs` 第 2 节正文）。
  注入过读故障的态走 `Injected` 那一臂，C670 不判（步 2），不进这条。
- 「V2 不旧于 V1」：二次崩溃镜像 = 第一次崩溃镜像加挂载写出去的写，只增不删；唯一抹东西的是实例表清扫，它只抹「按候选行判未发布」的单元头、先要求环里没有该实例的根：
  「//! 把那一行删掉。」（`crates/singlefs-core/src/instance_table_sweep.rs:3`）、「//! 次序是先判、后抹（F6）：判根环每个槽都自证过、环里没有该实例的根，判池中每一个落点（[`judge_the_instance_table_sweep`]），」（`crates/singlefs-core/src/instance_table_sweep.rs:5`）——V1 的根还在环里，它引用的表不会被抹。这条是读码推的，没量。
（上面第一条只写了两条读得出的情形；其余组合——V2 末条读得出而 V1 的读不出、等等——不手推，下一步用真函数全量枚举。）

**步 5　在真函数上量。** `b21_k3_witness.rs`（harness 档，副本里把 `witnessed_counter_holds_against_the_selected_version` 改成 pub，直接调它）：
手写日志世界——4 次发布 P0..P3（每次 1 或 2 条记录共 16 种形状，前两次实例 1、后两次实例 2，txg 递增，末条带末条标志），每条记录读得出 / 读不出（2^条数 种），c_见证 取 0 .. 总条数 + 1，
V 取 P0..P3，用真记录字节（`JournalRecord::to_bytes`）写进 mkfs 过的内存池，让真 `scan_journal` 去读。同时把五步抄成纯函数 `model`，逐点与真函数比：

- 「K3 calls=44928 model_vs_code_mismatches=0」（`research/prompts/m3-formal-code-b21-r1-sonnet-model/out/k3.log:2`）：抄本与真函数 44928 次调用逐点一致，所以上面五步的读法就是代码做的。
- 域 R（P1b：早于 V1 的发布的记录全读得出；P1a：c 取某次发布的末条计数器、且不早于 V1 前一次发布的）：「K3 domain R: pairs=23328 counterexamples listed=0 first=[]」（`research/prompts/m3-formal-code-b21-r1-sonnet-model/out/k3.log:7`）——23328 个 (世界, c, V1 旧于 V2) 三元组里没有「T(V2) 真而 T(V1) 假」。
- 其中「K3 domain R: pairs with T(V2) true and T(V1) true = 9072; pairs with T(V2) false and T(V1) true = 6912」（`research/prompts/m3-formal-code-b21-r1-sonnet-model/out/k3.log:6`）：真到真 9072、假到真 6912、真到假 0（其余 23328 − 9072 − 6912 = 7344 个假到假，算出来的，程序没单数）。**方向是「更旧那一版只会更真」**，与正文说的相符。
- 域外反例存在，推论对「任意日志世界」不成立：「K3 general: pairs=67392 counterexamples(T(V2) true, T(V1) false) listed=6 first=[」（`research/prompts/m3-formal-code-b21-r1-sonnet-model/out/k3.log:3`，首例 shape=0 mask=0010 c=1 V1=P1 V2=P2：P0 的记录丢了、c 指到它，V2 走「两条都读不出」按真，V1 末条读得出而 c < L(V1) 为假）。
  只放宽 P1b（早的记录读得出、c 不限）也有 408 个反例（`out/k3.log:5`，例：c 指到 V1 自己那次发布的非末条记录），要靠 P1a 排除。
  这两处域外态都需要「tail 指向一条读不出的更早记录」或「tail 指到非末条」，由 P1a、P1b 的出处排除（读码推）。

**这一步对「按支算还清」意味着什么。**
- 对正确实现：拒因是 N-配置 ⇒ 实现自己的判据在它的所选那一版（= V2，同一份盘面、同一套择根，见 `mount_transparency.rs:1387` 一带「R1 落到的那一版」那句）上为真，即 T(V2) 真（没读故障且不是两槽都撕裂那一形）。
  由单调 T(V1) 也真 ⇒ 接错不改「C670 那一条有没有交回一支」，于是不改计数 (判过, 不合法)、不改红不红、不改合法与否；它只改交回的是哪一支（步 3 的第 3 / 4 / 5 步）。这正是新用例那一格：计数仍 (1, 1)，只换支。**按「支」算还清成立。**
- 接错能改计数的只有「T(V2) 假而 T(V1) 真」：它要求实现在判官认为判据为假的态上拒了（即实现自己有缺陷）。这时接错版**多记一次不合法、多报一条 C670 红**（正确接线时这一态在第三截不报，步 2：「条款只管第二截，这一截不拿它们定红绿」）。
  方向是多报，不是漏报：不会放过实现缺陷，但会把「拒得没理由」错记成「C670（判据为真）」。这类态在正确实现上到不了；我没有构造出能走到它的合法崩溃态（推的）。
- 另一种接错（交成 `None`）会把 T 变假、少报；不在本题，也没被这批变异覆盖，只提一句，不展开。

**推翻条件（K3）**：
1. 找到一个崩溃枚举域里的态，V1 < V2、T(V2) 真而 T(V1) 假，即 tail 指向一条读不出的更早记录，或 tail 不是某次发布的末条（推翻 P1a / P1b）；我用 `transaction.rs:1977` 与 `crash_injection.rs:2983` 排除，没有在 checker 档上量。
2. 找到 V2 比 V1 旧的态（清扫抹掉 V1 的表）；我读码判不会（上面「V2 不旧于 V1」），没量。
3. 找到一个态：实现在 T(V2) 假时拒（拒因 N-配置）而 `crash_injection` 的枚举域能走到它。
以上三条只有 checker 档能证实或推翻，判决里标「推的」。

## 三、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1-a H2 用例 | 一致 | 正确实现绿（`out/h2.log:2`）、放宽 1 / 2 / 5 个 txg 红在落点断言；翻转点 k = S + 2（正确）/ S + 1（放宽），S = 5、6、7、8、12、16 上量过，默认环与 128 MiB 环都一样；用例恰在 S = 8 的唯一分辨格上。脆处：常数 9、10 绑死 S = 8（推的，S 真改时两条用例一起响）。 |
| K1-b 六张坏镜像 | 一致 | 第 22–27 张各只红 I-9.27，改动只落在 e3/e4/e5/e6/e9/e10 的头 ID 两字节加重封校验和；六种「不归 Zero」各丢一次，恰好各自那一张变不红、原用例红在恰好那一张；十一种树表条目一种一张。 |
| K1-c 判完再断言 | 一致 | 通过 / 失败条件与原写法相同，只多列几张；六次丢臂都红且点名恰好一张，没放宽。 |
| K1-d 注释对代码 | 一致（一处措辞偏宽） | 逐句对上 `mount_transparency.rs`；「C670 那一条……为假、不报」对用例那份盘面成立（c_见证 = 3 = 所选那一版末条计数器 3），对「两槽都撕裂」整支是「照常比」，不是恒假。 |
| K3 第四种接错 | 成立（推的，带 P1a、P1b、V2 不旧于 V1 三前提） | 真函数 44928 次调用与五步抄本逐点一致；域 R 23328 个三元组 0 反例、方向是「更旧更真」；域外有反例但被 tail 取法与段持久排除。按「支」还清：对正确实现接错只换支、不改计数与红绿；只在「实现无理由拒」的态上会多报一条 C670。 |

## 四、没做什么

- checker 档（`singlefs-checker-tier`）一个没跑：21-2 的 F3、改法 a、改法 b 与 C670 接线那条用例全没跑，K2 不归我；K3 里凡落在 `crash_injection.rs` 的结论（步 1、2 的计数、前提 P1b、V2 不旧于 V1）只读码，标「推的」。
- 没按 `mutations.tsv` 原文逐行替换后跑 prove-red（那是主 agent 已做的）；用的是环境变量钩子，语义相同，我只核了每行旧串在冻结树里恰好命中一次（`out/mutation-old-strings.log`）。
- 没量放宽之外的回收门槛偏差（窗口比较改法、`<=` 改 `<`），只推。
- 没量「journal 新根副本」在六张坏镜像上是否同步改得对，只量了 11 个登记单元与根槽。
- 日志世界 K3 只做了 4 次发布、两个实例、每次 1 或 2 条记录；更长的历史、环回绕没做。
- 没有算 GPU 判官侧。
- 没有给主 agent 的「要主 agent 跑的命令」：这一份没有留下需要后台跑的活。
- 草稿目录 `/tmp/claude-1000/m3-formal-code-b21-r1-sonnet/`（内含副本 `work/` 与编译目录 `target/`）：交回前删掉；要复现按模型目录 `README.md`。
