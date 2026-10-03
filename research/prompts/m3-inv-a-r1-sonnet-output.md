# m3-inv-a-r1 正推腿报告（Sonnet，2026-09-30）

<!-- doc-lint:not-numbers A1 A2 A3 A4 A5 A6 K1 K2 K3 -->

轮名 m3-inv-a-r1，格 A1–A6，立场正推：从已定条款与冻结副本的代码推每格「应该是什么」，再对着正文第五节的跑前判据逐条比。
读的：背景正文 `research/prompts/_m3-inv-a-r1-body.md`、事实表 `research/prompts/m3-inv-r1-facts.md`、`.claude/kb/` 里的原文（引文都现查过）、冻结副本 `/tmp/claude-1000/m3-ring-r1/tree/crates/`。
没读禁读清单里的任何东西；正文里点名的 `_m3-ring-r1-body.md` H6 一段（系统配置冗余）我读了一眼，它是正文不是腿报告。

冻结副本核对（命令与原样输出）：

```text
$ (cd /tmp/claude-1000/m3-ring-r1/tree && sha256sum -c research/prompts/m3-inv-a-r1-snapshot/crates-sha256.txt | grep -vc ': OK$')
0
$ diff -rq /tmp/claude-1000/m3-ring-r1/tree/crates crates ; echo "diff exit $?"
diff exit 0
```

（第一条我用绝对路径的清单文件跑，`0` = 205 行清单里没有一行不是 OK；第二条在仓根跑，冻结副本与工作区 crates 逐文件相同，所以下面代码行号在两处都成立。）

## 先说三条跨格的事（K1–K3），后面各格都会用到

### K1　池级 checker 的违例位是 64 位字，今天用了 49 位；每加一条不变量，层 0 的 GPU 判器要跟着移一份

- 违例位的类型：`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:49`「pub fn pool_checker_violated_bits(words: &[u32; VERDICT_WORDS]) -> u64」；CPU 侧对拍用 `crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:807`「bits |= 1u64 << index;」。位 i 对 `IMPLEMENTED_INVARIANTS[i]`，今天清单长度 `crates/singlefs-checker/src/image.rs:68`「pub const IMPLEMENTED_INVARIANTS: [&str; 49] = [」。
- 覆盖：`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:41`「/// 池级 checker 内核覆盖到的不变量（`IMPLEMENTED_INVARIANTS` 的下标）：三批接完之后是全部 49 条，对拍逐位比。」——GPU 内核（WGSL）自己判全部 49 条，新加的每一条在层 0 放量上要有一份 WGSL 才判得到。
- 本轮 A 格会新加的号：I-4.1、I-8.1、I-7.5、I-8.5、（C466 的）设备数一致那条，共 5 个（I-8.4 与 A6 不占位，见各格）；49 + 5 = 54，离 64 还剩 10 位，要与 b 轮（代字段与指针身份）新加的数合起来看。（命令：`grep -c '^| I-.*已实现' .claude/kb/invariants.md` 输出 49，今天的；本轮新加数是我按各格推荐数的，不是产物。）
- 推翻条件：`crash_judge_gpu.rs` 里违例字改成 128 位或改成按块号分段，这条就不成立；或层 0 决定新加的不变量只在 CPU 快档判、不进 GPU 放量（那要写进状态列，因为「层 0 每个崩溃状态都判」是现有各行状态列的共同尾句）。

### K2　三处「今天的样子」与代码对不上（条款正文或注释过期，不是实现错）

| 处 | 过期的一句（整行抄） | 今天代码里的对应（整行抄） |
|---|---|---|
| D23 已定项 14 射程末条 | 「实现今天的链首不锚在所选根覆盖的最后一条」（`.claude/kb/decisions/23-journal的角色与格式.md:407`） | 「let root_own_record_counter = counter_of_the_last_record_the_root_covers(root, records)?;」（`crates/singlefs-core/src/recovery.rs:3094`）；函数定义 `crates/singlefs-core/src/recovery.rs:2962`「fn counter_of_the_last_record_the_root_covers(」 |
| `singlefs-format` 里 F 常量的注释 | 「checker 不算在飞上限（它不判重放前缀取几条），」（`crates/singlefs-format/src/lib.rs:219`） | 「let in_flight_record_limit = journal_ring_bytes / JOURNAL_RECORD_BYTES / JOURNAL_SAFETY_FACTOR;」（`crates/singlefs-checker/src/image.rs:342`，I-7.13 那一格已经在算） |
| I-2.4 行里的实现落点 | 「`judge_pointer_mac_and_nonce_are_zero`（`crates/singlefs-checker/src/image.rs:619`）」（`.claude/kb/invariants.md:124`） | 「pub fn judge_pointer_mac_and_nonce_are_zero(」（`crates/singlefs-checker/src/image.rs:699`） |

第一行的后果：C365（链首锚点）在 `recovery.rs` 已经落地，A1 写 checker 的重放时锚点规则照 `recovery.rs:2962` 的读法乙写，不照「今天不锚」写。
推翻条件：这三行任一处在 kb 或代码里再查一遍，那一句其实还成立（比如 kb 那一句指的是别的函数）。

### K3　事实表里一句推断我核过，不成立（A3 展开）

事实表第四节写「推的：io_min = 3072、pbs = 4096 时槽距 6144，槽 1 偏移不是 4096 的整数倍，mkfs 不拒」。
静态读的结果：mkfs 没有对齐的事前判定这句对；但「mkfs 接受」不对——这一组参数 mkfs 会在最后写系统配置槽 1 时被块设备层拒（`Unaligned`），不是接受。细节与另一组 (pbs 512、io_min 3072) 那边 mkfs 真的接受，见 A3。

## A1　I-4.1 与 I-8.4（重放后可遍历、重放幂等）

### A1.1　「重放」有三层，I-8.4 的三层里只有最后一层不平凡

| 层 | 是什么 | 单份镜像上判得出吗 | 依据（整行抄） |
|---|---|---|---|
| 记录级 | 施加一条记录 = 把所选根四个字段换成记录里的 | 恒真：按值覆写，写两次同写一次 | 「施加一条记录 = 把所选根的这四个字段换成记录里的，不需要树语义。」（`.claude/kb/decisions/23-journal的角色与格式.md:443`）；「目标态天然幂等（把指针写成某个值，写两次结果一样）」（`.claude/kb/decisions/23-journal的角色与格式.md:132`）；代码 `crates/singlefs-core/src/recovery.rs:3210`「rebuilt = RootRecord {」 |
| 重放函数级 | `replay_journal(读者, 所选根, 环长, 记录集)` 交回重建的根 | 同一镜像跑两次是同一个输出：函数只读，入参是 `&dyn PoolReader` | `crates/singlefs-core/src/recovery.rs:3063`「) -> Result<(JournalScanReport, RootRecord), RecoveryFailure> {」（返回值里没有写盘）；`crates/singlefs-core/src/recovery.rs:3073`「let mut rebuilt = *root;」 |
| 盘上状态级 | 恢复 + 可写挂载写行那整段 | 单份镜像判不出，要两份（恢复前后） | 「O2（独立解析器 + checker） 判不了：I-4.5（已停用·需两镜像） 退役理由逐字「幂等要比对两个镜像，单镜像判不了」」（`.claude/kb/decisions/13-验证路线.md:33`） |

记录级那一格套 `.claude/singlefs-ai-sop/rules/test-discipline.md` 的话就是同义反复：「- **门槛不能是被测臂的定义**：写门槛之前把被测臂的定义并排放一遍，门槛能从定义直接推出来的，那一格不算判据。」（`.claude/singlefs-ai-sop/rules/test-discipline.md:56`）——覆写按值，幂等从定义直接推出来，这一层出局。

### A1.2　重放函数级有一个不平凡的形态：把重建出的根当所选根再放一遍（不动点），它断在哪

我把 `replay_journal` 的循环逐个 `break` 对了一遍（`crates/singlefs-core/src/recovery.rs:3109` 起到 `3208`）。第二遍的输入是第一遍的输出根 R1：

- R1 的实例与 txg 就是最后一条被施加记录的；`crates/singlefs-core/src/recovery.rs:3079`「record.instance == root.instance && (record.instance, record.checkpoint_txg) > water」把已施加的记录挡在水位之下；锚点 `recovery.rs:3094` 由 R1 自己那次发布里带末条标志的那一条给出，就是第一遍施加的最后一条，接链首 = 它的计数器 + 1。
- 第一遍的每一个 `break`（反向链不等、序号不是 1、换 txg 缺末条、提交标记没出现、点名单元验证失败……）只看记录自己与它前一条，第二遍从同一处接上、同一条记录上判出同一个结果——一致。
- 唯一不一致的一处：`crates/singlefs-core/src/recovery.rs:3109`「for record in above.into_iter().take(in_flight_limit) {」。第一遍只看水位之上的头 N 条（N = 环槽数 ÷ F）；第二遍水位抬高，又能看 N 条，把第一遍没看的接着施加。所以「不动点」在且仅在「所选根之上连续合法的记录多于 N 条」时断（推的，逐个 `break` 读出来的，没跑）。

这一句与 I-8.3 的第三段是同一件事：「且整段落在在飞记录数上限之内」（`.claude/kb/invariants.md:86`）。⇒ I-8.4 在单份镜像上的不平凡形态 = I-8.3 的窗口那一半，不是一个独占的判据。

合法历史里这一形到不了：一次发布的记录数被发布路径拒在 N 以内（`crates/singlefs-core/src/transaction.rs:5403`「if records_of_the_publish > record_limit {」），串行提交下一次发布的根落定之后才有下一次，所选根之上留着的至多是没落根的那一次发布；要超过 N 得根环里连着好几个新根都读不出——多重故障（推的）。

### A1.3　对着正文第五节跑前判据

- 「A1：R-独 的坏镜像造不出（I-8.4 在单份镜像上恒真） ⇒ I-8.4 不在 checker 判」：**触发**。记录级恒真（A1.1）；重放函数级只剩窗口那一格，且它与 I-8.3 第三段是同一个事实——写一份「只红 I-8.4」的坏镜像做不到：一份连续合法超过 N 条的镜像在 checker 里同时是 I-8.3 窗口那一半的坏镜像（I-8.3 今天也是「未实现」，`invariants.md:86`，且 R-独 的重放函数就是 I-8.3 的实现，无法只落一个）。
- 所以 I-8.4 二选一交用户：R-换 或 R-形（下面 A1.5）。
- I-4.1 不受这条影响：它的对象是「重放之后那一版能不能走完」，判定是走读，不是不动点。

### A1.4　I-4.1 走 R-独：措辞、checker 怎么判、要写多少、独立不独立、坏镜像

**措辞**（写进 `invariants.md` 那一行「不变量」列，草稿，零轮）：

> 恢复择出的根（根环里自证过的根按 (txg, 实例代号) 取最大者），按 D23 已定项 14 的五条前缀口径与注 1–4 施加 journal 之后得到的那一版，它的树表与中央映射树全部可完整遍历：每个被引用的单元读得出、整单元校验和与引用它的指针一致、头自描述与指针对得上。一条记录都没施加时这一版就是所选根本身，报不适用并写明归 I-7.2。

「系统配置指向的树」要改：系统配置里没有树指针（`crates/singlefs-checker/src/image.rs:213`「pub slot_spacing: u64,」附近 `PoolGeometry` 各字段里没有任何树指针；根住根环）。这一改动不是新条款，是把行里写错的主语换成实现今天的对象（择根口径 `crates/singlefs-checker/src/walk.rs:5864`「let newest_index = roots」起的 `(checkpoint_txg, 实例)` 取最大，与 core `choose_root` 同一个读法）。

**checker 怎么判**：
1. 新函数（暂名）`versions_the_next_mount_applies`：输入 `records_as_the_recovery_reads_them`（`walk.rs:5028`，每个计数器取第一块盘上那份，与恢复的次序一致）、根环里的根、最新根下标；照 D23 已定项 14 五条口径与注 1–4 自己写一遍：锚点（读法乙：带末条标志的那一条，多于一条时报「认哪一条条款没写」的那个失败）、链首（锚点读不出时 txg + 1 且序号为 1）、断号即止、反向链（复用 `record_can_enter_a_replay_prefix`，`walk.rs:5046`）、提交标记、发布边界、点名单元两份都验、在飞上限 N（`image.rs:342` 已有式子）。
2. 输出与 `VersionAppliedOnlyByRecords` 同形（`walk.rs` 里 `walk_version_applied_only_by_records` 的入参，定义在 `walk.rs:890`），最后一次被施加的那一版从记录新根段起走。
3. 用**另起一份 `Walk`**、判定丢掉、只看走读失败与 I-2.1 首个违例——形态照 `crates/singlefs-checker/src/walk.rs:5652`「fn judge_blocks_referenced_by_abandoned_roots(」；它在 walk.rs 第 5682 行另起 `Walk::starting_with`、第 5689 行走一条被抛弃的根，只把结果喂给 I-7.4。这样这一条红了不会连累 I-1.x / I-2.1 / I-4.8 / I-7.4 等（它们读的是最新根那一遍）。
4. 一条记录都没施加：报不适用、理由写「没有可施加的前缀，这一版就是最新根，归 I-7.2」，不报成立（免得与 I-7.2 双报同一个谓词）。

**要写多少行**（命令与原样输出）：core 里 `replay_journal` 本体去掉整行注释与空行 128 行，锚点函数 26 行——

```text
$ sed -n '3057,3226p' crates/singlefs-core/src/recovery.rs | grep -v '^\s*//' | grep -vc '^\s*$'
128
$ sed -n '2962,2993p' crates/singlefs-core/src/recovery.rs | grep -v '^\s*//' | grep -vc '^\s*$'
26
```

checker 侧独立写一份的量级同（推的：约 150–200 行，含点名单元验证 ~20 行——checker 已有 `check_unit` 与按位 CRC-32C，缺的是 `ScannedJournalRecord` 只留了位置条目 `crates/singlefs-checker/src/walk.rs:4913`「named_entry_locations: Vec<[PointerLocation; 2]>,」，要再存每个点名项的 `unit_class`（`crates/singlefs-checker/src/lib.rs:711`「pub unit_class: u8,」，core 用它定读多长 `crates/singlefs-core/src/recovery.rs:330`「fn unit_bytes_for_class(unit_class: u8) -> Option<usize> {」））；I-4.1 粘合 ~40 行；坏镜像与用例另计。

**与 core 的 `replay_journal` 独立不独立**：代码不共享——checker 与实现只共享 `singlefs-format`：「//! checker：与实现只共享 `singlefs-format` 这一个常量模块（D13（验证路线） 已定项 5），」（`crates/singlefs-checker/src/lib.rs:1`）。但两份都是从同一段 D23 文字写出来的，独立性只到「不抄代码」为止。真正让它值钱的是**差分**：层 0 每个状态已经跑 core 的 `recover`（`crates/singlefs-checker-tier/src/crash.rs:1226`「let consulted = recover(image, JournalPolicy::Consult);」）与池级 checker（`crates/singlefs-checker-tier/src/crash.rs:1240`「let pool_checker = check_pool_image(image);」），`recover` 交回 `effective_root`（`crates/singlefs-core/src/recovery.rs:4297`「effective_root: Some((effective_root.instance, effective_root.checkpoint_txg)),」）。让 checker 的重放交出同一对 (实例, txg) 并在 `judge_crash_image` 里逐状态比，两份实现哪边写错都当场红；这一步比 I-4.1 那条不变量本身更重要（推的），并且正好是 milestone 七·4「两边对拍」要的对拍口子。代价：GPU 那一份也要移（K1）。

**坏镜像怎么造，一份只红 I-4.1**：取「记录已落盘、根槽没落」的镜像（`crates/singlefs-harness/tests/checker_known_bad_images.rs:4213` 的 `image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_record` 就是这个形），把那条记录新根段里的树表指针改成指向一个全 0 槽（不指向任何点名单元，所以点名单元验证仍过），重封记录头校验和（罩整条 4096，D23 已定项 13）。core 施加它、走读读不出；checker 的重放施加它、走读走不完 ⇒ I-4.1 红；因为走读在另起的 `Walk` 里、判定丢掉（上面第 3 点），其余不变量不受连累，最新根那一遍照常绿。变异行：`crates/mutations.tsv` 一条「判定恒真」——把 I-4.1 那句 `judge` 的条件改成 `true`，必须红的用例是这份坏镜像的用例（原文锚点要实现之后取，现在还没有）。

### A1.5　I-8.4 的两条路（R-换、R-形），我的推荐

- R-换：对象换成「恢复 + 写行」整段，在崩溃注入的二次崩溃上判。这一层才是 `Recover(Recover(c))` 有内容的地方——恢复本身不写盘，写盘的是随后的可写挂载。落点是 milestone 一·1 的多次挂载流（`.claude/kb/milestone/03-third-txn.md` 一·1；`crash_injection_record_checker_sees_every_publish_across_a_second_crash.rs` 已经在测二次崩溃后记录核对器看得到每一次发布，文件名可证有这个形态，我没读用例体）。
- R-形：交给七·4（M3 证明）。
- 推荐 R-换，理由：R-形要证 `Recover(Recover(c)) = Recover(c)`，先得有「Recover 是镜像到镜像」的定义，而今天 core 的恢复不写盘，这个对象只有 R-换 定义得出来；R-换 先做，R-形 证的是同一个对象。

**kb 内部的一处对不上要交主 agent**（两句并排整行抄）：
- 「**M3 幂等与单调**：`Recover(Recover(c)) = Recover(c)` 就是 I-8.4（重放幂等）——与二·1 里它的 checker 判定是同一条性质的两种形式，两边对拍；」（`.claude/kb/milestone/03-third-txn.md:165`，七·4 那一行）
- 「| I-4.5 | 已停用·需两镜像 | **此编号不再使用**——幂等要比对两个镜像，单镜像判不了；」（`.claude/kb/invariants.md:169`）；加上上面 A1.1 引的 D13 已定项 1 M3 那一句（`.claude/kb/decisions/13-验证路线.md:33`）。
- 前一句说 I-8.4 有「二·1 里的 checker 判定」，后两句说单镜像 checker 判不了。按 A1.2–A1.3，我站后两句；七·4 那一行的措辞要改成「checker 判 I-4.1 与二次崩溃上的差分，I-8.4 的证明版在这里」。

**推翻条件**：
- A1.2「不动点只断在窗口」：找到 `replay_journal` 里第二个依赖「这次调用之前施加过多少条」的 `break`（我读的 `3109-3208` 里没有；`report` 里的计数只往上加、不进任何判断）。
- A1.4 的「坏镜像只红 I-4.1」：走读另起 `Walk` 时仍有别的不变量红（比如 I-1.2 的已发布谓词读了新根段那一版）——要在实现里跑一遍才知道，我没跑。
- 「独立性只到不抄代码」：checker 的重放写出来后，与 core 在层 0 全部状态上零差分、且两份的 `break` 结构逐条同构——那说明它在复述而没有独立判断。

## A2　I-8.1（环几何够大，C28）

### A2.1　三个字段今天的样子

- mkfs 写：在飞上限 = `journal_in_flight_record_limit(环长)`：「journal_ring_bytes / JOURNAL_RECORD_BYTES / JOURNAL_SAFETY_FACTOR」（`crates/singlefs-core/src/system_configuration.rs:89`）；最坏占用 = 「writer.put_u64(in_flight_limit * JOURNAL_RECORD_BYTES);」（`crates/singlefs-core/src/system_configuration.rs:433`）；F = 「writer.put_u32(u32::try_from(JOURNAL_SAFETY_FACTOR).expect("安全系数"));」（`crates/singlefs-core/src/system_configuration.rs:434`），常量 `crates/singlefs-format/src/lib.rs:221`「pub const JOURNAL_SAFETY_FACTOR: u64 = 3;」。三个字段在槽里的偏移是 345 / 349 / 357：从环长（333，8 字节）往后数 record 尺寸 4、在飞上限 4、最坏占用 8、F 4，落到 R（361），而 checker 读 R 的位置是 `crates/singlefs-checker/src/image.rs:245`「let regions = u64::from(slot[361]);」——数得回去。
- 读：两边都不读这三个字段。命令与原样输出（冻结副本）：

```text
$ grep -rn '\b34[15]\b\|\b349\b\|\b357\b' singlefs-core/src singlefs-checker/src singlefs-format/src ; echo "grep exit $?"
grep exit 1
```

  恢复按环长现算在飞上限（`crates/singlefs-core/src/recovery.rs:781`「if journal_in_flight_record_limit(sizes.journal_ring_bytes) == 0」）。挂载比对参数与盘上字段时，「// 两边都整个解构：参数或几何加了字段、这里没比，编译不过。」（`crates/singlefs-core/src/mount.rs:3526`）——`MakeFilesystemParameters` 里没有这三样，所以它们从没被比过。
- 「任一事务的最坏占用」在 core 里对应的是发布路径拒绝：「if records_of_the_publish > record_limit {」（`crates/singlefs-core/src/transaction.rs:5403`），`record_limit` = `journal_record_limit_of_one_publish`（`crates/singlefs-core/src/transaction.rs:5387`）。度量单位是**记录条数**。
- D28 那个 ckpt_cost 的单位是别的量纲：「单位是 16 KiB 元数据块，代进已定项 1 那条按字节写的式子时乘 16384」（`.claude/kb/decisions/28-挂载期承诺量.md:106`）；它进的是空间准入，不进 journal 记录数。

### A2.2　正文那个改写，逐句过一遍

改写候选三句：① 字段自洽（在飞上限 = ⌊环长 ÷ 4096 ÷ F⌋、最坏占用 = 在飞上限 × 4096、F ≥ 2），mount 时复算比对；② 「环 ≥ F × 最坏占用」；③ 「任一事务的最坏占用 ≤ 在飞上限」落在发布时拒绝。

- ② **同义反复**：把 ① 的两个等式代进去，⌊环长 ÷ 4096 ÷ F⌋ × 4096 × F ≤ 环长恒成立，它不是一格判据（`.claude/singlefs-ai-sop/rules/test-discipline.md:56`，引文见 A1.1）。写成不等式的那一句可以留在措辞里当推论，不单独判。
- ① 拆成三个等式各占一格（同一份 test-discipline 的第一条「不把两个量合成一个 true / false」）：对 mkfs 的产物它按构造恒真（`system_configuration.rs:433` 由环长倒算），所以它的会红性只来自「别的实现写的、改过的、将来 F 可调之后的镜像」，与 I-7.14 那一格同处境（盘体对调也是靠坏镜像钉判别力，`invariants.md` I-7.14 行）。
- ③ **不是恒真，也不是同义反复**：它是发布路径的一道拒绝（记录条数 ≤ N），镜像上有对应物——环里同一实例、同一 checkpoint_txg 的自证记录条数不超过 N（环转过圈之后只会少数不会多数，所以观察到的条数是真实条数的下界，判红不会误红）。这一半才是「任一事务」的落点，措辞里把「事务」读成「一次发布」，因为 core 拒的是一次发布切出的记录条数（`crates/singlefs-core/src/transaction.rs:5383`「/// 一次发布至多切成几条 journal 记录（代码审阅第 26 条）：在飞记录数上限（环槽数 ÷ F，D23（journal 的角色与格式） 已定项 18，」）。

结论：改写按 ①（拆三格）+ ③ 保留，② 降成推论。它没有把 I-8.1 缩成同义反复——③ 那一半有独立会红的坏镜像（下面）。① 那一半今天只对镜像自洽有意义，且与 core 的读法有一处要连带改（A2.4）。

### A2.3　D28 已定项 4 口径注那一格（mkfs 取上界还是挂载期内复算）

口径注原文：「mkfs 时声明的最坏占用要么取上界、要么挂载期内也复算比对——哪一种没定。」（`.claude/kb/invariants.md:84`）。

今天的实现已经答了：最坏占用 = 在飞上限 × 4096，由环长与 F 倒算（A2.1 第一条），是**上界**，不随树高变。会随树高长的 ckpt_cost 是空间块数，进准入不进 journal（A2.1 第四条），两者不在同一条式子里。所以口径注里的两难在实现上不存在：取上界，「挂载期复算」退化成「把三个字段与环长、F 现算的数比一次」，环长与 F 都是不可变字段，复算是确定的。
D23 已定项 19 ③ 那句「**挂载时有效 `T_dirty` = min(`T_dirty` 字段, 环长 ÷ F)**」（`.claude/kb/decisions/23-journal的角色与格式.md:511`）在代码里没有落点：「也没有触发路径——T_time 与 T_dirty 的发布触发没实现，整理三水位恒 0 = 内置默认」（`crates/singlefs-core/src/system_configuration.rs:221`）。这是 T_dirty 触发没做的连带，不是 I-8.1 的账；写一句进 I-8.1 的状态列「T_dirty 夹取待触发路径落地」即可（推的）。
推翻条件：找到任一处 core 代码把最坏占用按事务结构（树高、数据单元数）现算——`grep -n 'journal_record_limit_of_one_publish\|in_flight_limit' crates/singlefs-core/src` 之外还有别的算最坏占用的地方（我 grep 到的只有这两个名字，都是环长的函数）。

### A2.4　措辞、checker 怎么判、坏镜像

**措辞**（草稿，零轮；条款变更，交用户）：

> ① 任一盘任一自证过的系统配置槽里：journal 在飞记录数上限 = ⌊环长 ÷ 4096 ÷ F⌋，journal 最坏占用 = 在飞上限 × 4096，安全系数 F = `JOURNAL_SAFETY_FACTOR`（格式常量，第一版 3，读者要求 F ≥ 2）；三样任一不等，判红；读者挂载时同样比，不等整池拒。② 环里同一实例、同一 checkpoint_txg 的自证记录不多于在飞记录数上限条（一次发布写出的记录不超过最坏占用 ÷ 4096）。环大小 ≥ F × 最坏占用是这两条的推论，不另判。

**checker**：
1. ①：`geometry_of`（`crates/singlefs-checker/src/image.rs` 里读 R、S 等字段的那个函数）多读偏移 345 / 349 / 357 三个字段，`PoolGeometry` 加三个字段；判定比现算的数，报成 I-7.13 那一族的成员（同一张「读字段，不等于常量就整池拒」的表，`crates/singlefs-checker/src/image.rs` 第 239 行的注释记着这条用户定案，2026-09-27）还是单列 I-8.1，是编号取舍：I-7.13 的原句「**判的字段与 core 收同一张表**」（`.claude/kb/invariants.md:64`）要求 core 也收——见下。
2. ②：`scanned_journal_records_by_device` 已经把每盘的记录按计数器收齐；按 (实例, checkpoint_txg) 分组数条数，N 用 `crates/singlefs-checker/src/image.rs:342`「let in_flight_record_limit = journal_ring_bytes / JOURNAL_RECORD_BYTES / JOURNAL_SAFETY_FACTOR;」同一个式子；约 40 行（推的）。
3. core 要连带改：`system_configuration_values_this_reader_accepts`（`crates/singlefs-core/src/recovery.rs`，I-7.13 那八项判定的所在）加三项，不然 checker 判红而 core 照样挂上，破坏 I-7.13 立下的「两边收同一张表」。这是**实现变更加条款变更**（挂载多一道拒绝），交用户；不接受就把 ① 降成「checker 判、core 不判」并在状态列写明，那样 ① 与 I-7.13 的口径不一致，我不推荐。

**坏镜像**（各只红自己）：
- ①：干净镜像上把每盘所选槽里偏移 345 的 4 字节加一、重封整槽校验和（`checker_known_bad_images.rs` 里 `mutate_system_configuration_slot` 就是这个用途，I-7.13 的坏镜像用它）。只红 I-8.1 ①：I-7.13 的八项不读这三个字段，`geometry_of` 其余判定不动（推的，没跑）。同一份在 core 上应被整池拒（若 A2.4 第 3 点接受）。
- ②：小环 mkfs（环长取 F × 4096 × k，k 小，例如 k = 2 时 N = 2）后伪造 N + 1 条同 (实例, txg) 的自证记录（序号 1..N+1，末条带标志），逐条重封记录头。harness 默认环是 768 MiB（`crates/singlefs-harness/tests/common/mod.rs:54`「journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,」），默认环下 N = 65536，伪造 65537 条约 256 MiB，不适合做已知坏镜像，所以必须小环；`format_pool` 收不收环长参数我没读签名。
**变异行**（`crates/mutations.tsv` 形态：六段制表符，见文件头；原文锚点实现后取）：① 判定条件改 `true`；② 判定条件改 `true`；再各一行「池级走读不调这一判」（照 I-7.14 那三行的形态，`crates/mutations.tsv` 里含 `I-7.14` 的行）。

**状态列怎么写**：①② 各写会红的坏镜像用例名；「T_dirty 夹取待触发路径落地」；C28 改成「已还」时把「mount 时复算比对」一句换成「core 三项判定的位置」。

**推翻条件**：
- 「② 是推论」：找到一个 F、环长、字段都自洽而 `环 < F × 最坏占用` 的镜像——那说明字段定义与式子不是我读的那个（`system_configuration.rs:433` 之外还有写最坏占用的地方）。
- 「③ 判红不会误红」：找到一份合法历史，同 (实例, txg) 的记录条数超过 N（比如重发把同一发布的记录写进不同槽、留下超过 N 份不同计数器的同发布记录）。原样重发用同一计数器（`.claude/kb/decisions/23-journal的角色与格式.md:399` 那一段「原样重发」），我读的是「同计数器」，没跑。

## A3　I-7.5（根槽按判定宽度对齐，C47）

### A3.1　今天 checker 手里有什么、缺什么

- 逐槽偏移现成：`crates/singlefs-checker/src/image.rs:593`「pub fn root_slot_positions(geometry: &PoolGeometry) -> Vec<(u64, u64, u32, u64)> {」，偏移式子在 `crates/singlefs-checker/src/image.rs:602`「let offset = geometry.base_slot * SLOT_BYTES」起：基址 × 16384 + 区域 × P × chunk + 槽 × 槽距，全从系统配置读；槽距住 `PoolGeometry`（`crates/singlefs-checker/src/image.rs:213`「pub slot_spacing: u64,」）。
- 判定宽度：声明的 `physical_block_size` 在 `PoolGeometry`；声明的 `io_min` 不在——字段表偏移 425 只在注释里点了名：「/// chunk 367、根环基址 371、逐区域设备 379、单元区起点 417、io_min 425、槽距 429）。」（`crates/singlefs-checker/src/image.rs:199`），结构里没这个字段。
- 探测：`ImageReader` 没有探测口子（`crates/singlefs-checker/src/image.rs:19`「pub trait ImageReader {」，五个方法都不返回设备粒度）；`DecisionWidth::Probed` 只在测试里构造，命令与原样输出：

```text
$ grep -rn 'DecisionWidth' --include=*.rs .
singlefs-checker/src/lib.rs:22:pub enum DecisionWidth {
singlefs-checker/src/lib.rs:27:impl DecisionWidth {
singlefs-checker/src/lib.rs:31:            DecisionWidth::Probed { bytes } | DecisionWidth::DeclaredOnly { bytes } => bytes,
singlefs-checker/src/lib.rs:846:        let declared = DecisionWidth::DeclaredOnly { bytes: 512 };
singlefs-checker/src/lib.rs:847:        let probed = DecisionWidth::Probed { bytes: 512 };
```

  这就是 C388：「D13（验证路线） 已定项 5 定「判定宽度由 checker 自己探测再与系统配置比」」（`.claude/kb/checks-owed.md:319`）。
- core 的块设备有 pbs 的探测：`crates/singlefs-core/src/block_device.rs:134`「fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes;」，没有 io_min 的探测（`grep -n 'io_min\|minimum_input_output' crates/singlefs-core/src/block_device.rs` 无输出）。所以「checker 自己探 io_min」今天没有可接的东西；pbs 那一半接得上。
- 池级判定用第一块盘的几何：`crates/singlefs-checker/src/walk.rs:5769`「let geometry = chosen[0].2;」；每盘所选槽的视图与几何在 `chosen` 里都有。

### A3.2　判定宽度取什么

条款的两处：C47 的判法「镜像侧可判：读系统配置拿到根环起始偏移与槽宽，`起始偏移 % 判定宽度 != 0` 判红；每个槽逐个判，不是只判第 0 个。」（`.claude/kb/checks-owed.md:48`）；D2 已定项 15 把粒度参数定给 io_min：「把起始偏移抬到 `io_min`」（`.claude/kb/decisions/02-RAID条带策略.md:256`，全句「……把起始偏移抬到 `io_min`（I-7.5（根槽按判定宽度对齐） 的粒度参数）即满足已定项 17，槽宽仍等于 `physical_block_size`」）；I-7.5 行射程注：「**射程注（2026-09-13，D2（RAID 条带策略） 已定项 15）**：`io_min` 大于 `physical_block_size` 时按 `io_min` 判」（`.claude/kb/invariants.md:56`）。

⇒ **判定宽度 W = max(声明的 physical_block_size, 声明的 io_min)**（io_min 为 0 表示没声明，取前者）。逐槽判 `偏移 % W == 0`，R × S 个槽全判，偏移式子就是 `root_slot_positions` 的那一个。
- 异构池：mkfs 只收一份 `SystemImmutableSizes`（`make_filesystem` 的 `parameters.geometry`），今天池内每盘同宽，C47 说的「取 max」没有对象；实现时按每盘所选槽各自声明的宽度取 max 判，这样以后异构了不用改式子（推的）。
- 探测不到时：按声明值判，判定说明里写「声明值，未探测」（`DecisionWidth::DeclaredOnly` 就是为这个留的）；不许把声明值报成探到的。pbs 的探测比对可以接：给 `ImageReader` 加一个默认返回 `None` 的方法 `probed_physical_block_size(device)`，有值时另判「声明 = 探测」，不等判红；io_min 那一半今天没探测口子，C388 只能还一半（推的，接口形状我没设计成型）。

### A3.3　算术：哪些参数组合会让 I-7.5 红（我的算术，脚本与输出在 `research/prompts/m3-inv-a-r1-sonnet-model/`）

```text
$ python3 research/prompts/m3-inv-a-r1-sonnet-model/a3_alignment.py
pbs=4096 io_min=3072 -> (槽距, 判定宽度, 不对齐的根槽数, 前两个(区域,槽,偏移,余数), 按 pbs 不对齐的系统配置槽) = (6144, 4096, 12, [(0, 1, 1054720, 2048), (0, 3, 1067008, 2048)], [(1, 6144)])
pbs=512 io_min=3072 -> (槽距, 判定宽度, 不对齐的根槽数, 前两个(区域,槽,偏移,余数), 按 pbs 不对齐的系统配置槽) = (6144, 3072, 24, [(0, 0, 1048576, 1024), (0, 1, 1054720, 1024)], [])
pbs=512 io_min=4096 -> (槽距, 判定宽度, 不对齐的根槽数, 前两个(区域,槽,偏移,余数), 按 pbs 不对齐的系统配置槽) = (4096, 4096, 0, [], [])
pbs=4096 io_min=0 -> (槽距, 判定宽度, 不对齐的根槽数, 前两个(区域,槽,偏移,余数), 按 pbs 不对齐的系统配置槽) = (4096, 4096, 0, [], [])
pbs=512 io_min=0 -> (槽距, 判定宽度, 不对齐的根槽数, 前两个(区域,槽,偏移,余数), 按 pbs 不对齐的系统配置槽) = (4096, 512, 0, [], [])
pbs=4096 io_min=65536 -> (槽距, 判定宽度, 不对齐的根槽数, 前两个(区域,槽,偏移,余数), 按 pbs 不对齐的系统配置槽) = (65536, 65536, 0, [], [])
pbs=4096 io_min=8192 -> (槽距, 判定宽度, 不对齐的根槽数, 前两个(区域,槽,偏移,余数), 按 pbs 不对齐的系统配置槽) = (8192, 8192, 0, [], [])
pbs=512 io_min=2097152 -> (槽距, 判定宽度, 不对齐的根槽数, 前两个(区域,槽,偏移,余数), 按 pbs 不对齐的系统配置槽) = (2097152, 2097152, 16, [(0, 0, 1048576, 1048576), (0, 1, 3145728, 1048576)], [])
```

（`sha256sum`：脚本 `365fc810ef097e3755fc0d387069813f1b936244f478429cb877933e88303233`，输出 `22daa7ffa1ec4b0c7e094ccf3ee678a1fa234000c9e4c073de6472f2d92fc0b4`。）
读法：槽距按 4096 向上取整到 io_min 的整数倍算（`crates/singlefs-core/src/system_configuration.rs:156`「pub fn slot_spacing_for(minimum_input_output_bytes: u32) -> u32 {」，同文件 `:859` 的单测钉了 3072 → 6144）；区域起点是 1 MiB 的整数倍。所以 io_min 是 512 到 1 MiB 的 2 的幂时，布局按构造对齐，I-7.5 恒绿；红只出现在 io_min 不是 2 的幂、或大于 1 MiB、或 pbs 不整除槽距的组合上——今天现实里的盘都在恒绿的那一边，这条不变量挡的是「参数组合没人核」与「系统配置被改」。

### A3.4　mkfs：事实表那句推断不成立，另有一处 mkfs 真的接受

- (pbs 4096, io_min 3072)：槽距 6144，系统配置槽 1 在偏移 6144。mkfs 的几何判定 `crates/singlefs-core/src/make_filesystem.rs:306`「fn check_geometry(」里没有对齐判定（`grep -n 'is_multiple_of\|% ' crates/singlefs-core/src/make_filesystem.rs crates/singlefs-core/src/recovery.rs` 无输出），它先清根环与 journal 环（`crates/singlefs-core/src/make_filesystem.rs:497`「device.write_zeroes_at(region_start(region_to_clear), root_ring_region_bytes)?;」，长度 8 × 6144 = 49152 是 4096 的整数倍，过）、写两个单元、写三条第 0 代根（都在区域起点、1 MiB 的整数倍，过），**最后**写系统配置槽：`crates/singlefs-core/src/make_filesystem.rs:593`「device.write_at(offset, &slot, WriteDurability::Plain)?;」在偏移 6144 上被块设备层拒：`crates/singlefs-core/src/block_device.rs:226`「if !offset.0.is_multiple_of(block) || !length.is_multiple_of(block) {」（harness 的稀疏盘按同一判据，`crates/singlefs-harness/src/memory_pool.rs:114` 的注释写着这一点）。结局：mkfs 返回块设备错误，盘上留着清过的环与半个池——不是「接受」，也不是「事前拒」（静态读的，没跑）。
- (pbs 512, io_min 3072)：槽距 6144，块设备按 512 判对齐，所有写都过；mkfs 成功，产物按 io_min 3072 判有 24 个根槽不对齐（上面第二行）。这是 mkfs 真的接受一个违反 D2 已定项 15 要求的组合。
- 更简单的一组：mkfs 根本不把槽距与 io_min 联系起来——`slot_spacing_for` 只被测试与 e142 装置调用（`grep -rn 'slot_spacing_for' --include=*.rs .` 命中的是 `system_configuration.rs` 自己的定义与单测、`e142_new_pool_file_creation_write_dump_one_device.rs:110`、harness 三个测试文件；e158 装置里另有一个测试函数名恰含这个子串，不是调用），`fixed_structure_slot_spacing` 是调用方给的字段，`check_geometry` 只判 `physical_block_size` 不宽过槽距（`crates/singlefs-core/src/make_filesystem.rs:378`「if geometry.physical_block_size > geometry.fixed_structure_slot_spacing {」）。所以 (pbs 512, io_min 8192, 槽距 4096) 也被 mkfs 接受。

**推荐 mkfs 同时加事前拒绝**：`check_geometry` 里、任何写之前，判 `槽距 % W == 0`、三个区域起点 `% W == 0`（`region_start(r)` 是编译期常量，只随 W 变），与 `槽距 == slot_spacing_for(io_min)`；各一个 `MakeFilesystemError` 成员。理由：`check_geometry` 的文档写明「几何判定，全在任何写之前」；(pbs 4096, io_min 3072) 那组今天是写到最后才炸，留半个池。这是实现变更不是条款变更（D2 已定项 15 与 C47 已经要求 mkfs 按判定宽度划）。变异行：把新判定的条件改 `false`，必须红的是 mkfs 拒绝的用例。加了之后「mkfs 造出坏镜像」的路就断了，坏镜像走下面的改系统配置那条。

### A3.5　措辞、坏镜像、状态

**措辞**（草稿，零轮）：

> 每个根槽的起始偏移是它所在盘的判定宽度的整数倍；判定宽度 = max(该盘所选系统配置声明的 `physical_block_size`, 声明的 `io_min`)，io_min 为 0 视为没声明。逐槽判（R × S 个槽，不只判第 0 个）。射程注不变；系统配置槽 1（偏移 = 槽距）同一条规则判（D2 已定项 17 的射程写的是「根槽、系统配置槽、墓碑一类」，`.claude/kb/decisions/02-RAID条带策略.md:254`）。判定宽度探不到时按声明值判并注明「声明值，未探测」。

系统配置槽那一句是我加的（零轮）：条款射程写了它，checker 零覆盖（C254 记着）；不加也行，I-7.5 行的字面只有「根槽」。

**坏镜像（只红 I-7.5）**：干净镜像上把每盘所选槽里偏移 425 的 io_min 改成 8192、重封整槽校验和（用 `mutate_system_configuration_slot`）：偏移 425 不进 `PoolGeometry`、I-7.13 的八项都不读它，别的不变量不动（推的）；槽距 4096 下槽 1 偏移 1 MiB + 4096，对 8192 余 4096，红。
（mkfs 加事前拒绝之前，也可以直接用 (pbs 512, io_min 3072) 由真 mkfs 造出，比改字段更像真实的坏 mkfs。）

**状态列**：「已实现（判定宽度取声明值；探测比对仍欠 C388）」；C47 标已还，C388 标「pbs 探测口子待接、io_min 无探测口子」。

**推翻条件**：
- 「恒绿」：找到一个现实里的 (pbs, io_min, 槽距) 三元组，pbs 与 io_min 都是 2 的幂且 io_min ≤ 1 MiB，而 `a3_alignment.py` 判不对齐——那说明区域起点式子不是我读的（`root_ring.rs:94` `region_start` 起）。
- 「mkfs 写到最后才炸」：块设备层对系统配置槽 1 的写不走 `check_aligned_and_in_range`（比如 mkfs 给块设备的是另一个写口）——在冻结副本上跑一次 mkfs 就分辨，我没跑。

## A4　I-8.5 改写与系统配置冗余（C367、C461、C466）

### A4.1　今天两边各怎么读

- tail 的载体：「#### 已定项 3：tail 住系统配置槽，改 jbd2 两处」（`.claude/kb/decisions/23-journal的角色与格式.md:116`）。checker 已解出 tail（`crates/singlefs-checker/src/lib.rs:175`「pub journal_tail: u64,」）但 walk 不读它：`grep -n journal_tail crates/singlefs-checker/src/walk.rs` 无输出（冻结副本，grep 退出码 1）。
- core：一块盘两槽都无效就跳过：「// 这块盘上的两份都废了，但别的盘各自还带着一份完整的系统配置：跳过它，别让整池挂不上。」（`crates/singlefs-core/src/recovery.rs:1005`）；文档注释写明这是冗余买的：「/// 一块盘两个槽都无效时**跳过这块盘**，接着看别的盘：系统配置每盘放一份买的就是这份冗余」（`crates/singlefs-core/src/recovery.rs:954`）；失败只在全池都没有：「/// 池里每块盘的两个槽都无效（[`RecoveryFailure::NoValidSystemConfiguration`]）；」（`crates/singlefs-core/src/recovery.rs:958`）。用例钉着挂得上且读回文件：`crates/singlefs-harness/tests/system_configuration_per_device_redundancy.rs:133`「fn both_system_configuration_slots_unreadable_on_device_one_still_read_the_file_back() {」。
- checker：`chosen` 为空时整片报不适用：`crates/singlefs-checker/src/walk.rs:5760`「if chosen.is_empty() {」；注释自己承认缺一个编号：「（.claude/kb/checks-owed.md 的 C461）。⚠️ 「哪块盘的系统配置全废了」今天没有编号报得出来，仍欠在 C461。」（`crates/singlefs-checker/src/walk.rs:5741`，整行是注释的后半）。
- checker 的槽读法把三种「不可用」并成一个成员：「/// 不自证（读不到、magic 或整槽校验和不对）、incompat 位不认得、R 或 S 越界：这一槽不可择（R、S 的既有处置，不动）。」（`crates/singlefs-checker/src/image.rs:384`）→ `NotChoosable`（`crates/singlefs-checker/src/image.rs:385`「NotChoosable,」）。core 那边分得开：`crates/singlefs-core/src/recovery.rs:945`「(None, None) => SystemConfigurationSlotReading::NotSelfDescribing,」与 `IncompatBitsNotRecognized`。这一点决定 V-池 能不能误红（A4.2）。

### A4.2　三个候选逐个过跑前判据

**V-盘（每块盘非空）：出局。** 两处独立的理由：
1. 判据「A4：V-盘 与 core 的挂载行为冲突 ⇒ 出局，除非同时改 core」——触发：core 在一块盘全废时照样挂上（`recovery.rs:1005`、用例 `system_configuration_per_device_redundancy.rs:133`），而 V-盘 判这样的镜像违例。两句并排：
   - C367 的改写：「I-8.5 改写之后 checker 判「每块盘两个系统配置槽里自证过的集合非空」：两槽都改坏必须红，mkfs 刚写完的镜像必须绿」（`.claude/kb/checks-owed.md:305`）
   - core：「// 这块盘上的两份都废了，但别的盘各自还带着一份完整的系统配置：跳过它，别让整池挂不上。」（`crates/singlefs-core/src/recovery.rs:1005`）
2. 第一条判据「一条措辞在任一合法镜像……上判红 ⇒ 出局」：`system_configuration_per_device_redundancy.rs` 的全废盘状态是合法（容错射程内）镜像，V-盘 判它红；`whole_device_loss_after_warm_up.rs` 的丢整块盘状态大概同理，文件名与 D23 已定项 14 的 C519 那一格对得上，用例体我没读。
改“除非同时改 core”那条路：改 core 让一块盘全废拒挂，就废掉「系统配置每盘放一份」买的冗余（`recovery.rs:954`）——条款变更，我不推荐。

**V-池（全池至少一块盘有自证槽）：推荐**，措辞：

> 池里至少有一块盘的两个系统配置槽里有一个槽自证过（magic 与整槽校验和对）；tail 住这些槽（D23 已定项 3），这个集合就是全部 tail 见证者。（C367 说站不住的后半句「除代号最大者外至少还有一条更早的」删掉。）

- checker：把 `walk.rs:5760` 那个早退支拆成两支：(a) 全池没有任何一个槽自证过（每个槽都是「不自证」）⇒ I-8.5 判红、别的不变量照旧报不适用；(b) 有槽自证过、只是 incompat 位不认得或 R、S 越界 ⇒ 不报 I-8.5 红，报不适用并写「布局不认识 / R、S 越界」（core 报的是另外的失败，`RecoveryFailure::SystemConfigurationIncompatBitsNotRecognized`，不是「全废」）。这要求把 `NotChoosable` 拆成「不自证」与「自证过但不可择」两个成员（`image.rs:385` 一处定义，`value_refused_by_this_reader` 旁的几处 match 跟着改，约 10 行；推的）。不拆就会把「新版本布局的池」误报成「系统配置全废」。
- 形态照 I-7.13：那一条判红后其余不变量一律报不适用，代码在 `crates/singlefs-checker/src/walk.rs:5751`「if any_system_configuration_slot_carries_a_refused_value {」起的支。V-池 的红同样带「其余不适用」，所以「只红它」的读法是「它红、别的全不适用」，与 I-7.13 的坏镜像用例同一个断言形态。
- 坏镜像：干净镜像上两块盘各两个槽都翻一个字节（整槽校验和不对）。层 0 不会出现这种状态：每块盘一次至多一个系统配置写在飞、同时撕的至多一个槽（推的，层 0 的撕裂状态里没见过四槽全撕，没枚举核过）。
- 现有的用例 `the_pool_checker_still_judges_every_invariant_when_one_device_has_no_valid_system_configuration`（`crates/singlefs-harness/tests/system_configuration_per_device_redundancy.rs:277` 起）只坏一块盘，V-池 不动它。
- 变异行：I-8.5 判定改 `true`、「池级走读不调这一判」各一行（照 I-7.14 三行的形态）。

**V-报（同 V-池，另加一类「降级」判定）：不推荐这个形态。** 代价：`InvariantVerdict` 只有三个成员（`crates/singlefs-checker/src/image.rs:32`「pub enum InvariantVerdict {」），加第四个，引用它的文件 54 个，且层 0 的匹配是穷举的（`crates/singlefs-checker-tier/src/crash.rs:1202`「InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,」，没有通配臂）：

```text
$ grep -rl 'InvariantVerdict' --include=*.rs . | wc -l
54
$ grep -rn 'InvariantVerdict::NotApplicable' --include=*.rs . | wc -l
76
```

而「盘 X 系统配置全废」不是违例（是容错射程内的降级），报进违例通道会污染层 0 的红。替代：单开一个只读函数（暂名）`devices_without_a_self_certified_system_configuration(reader) -> Vec<u32>`，不进 `IMPLEMENTED_INVARIANTS`（所以不占 K1 的位、不用 WGSL）。这就是 C461 欠的那一半（「哪块盘的系统配置全废了」），出口形态选「另开一个出口」，不是新判定类。

### A4.3　与二·4 那一轮 H6（C463）的条款对不对得上

H6 的两个候选：报出来就够、或可写挂载当场自愈重写（`research/prompts/_m3-ring-r1-body.md` 的 H6 一段，正文，我没读那一轮的腿报告）。V-池 与这个旁路函数对两个结局都成立：
- H6 取「报」：core 的 `RecoveryReport` 加一个「系统配置全废的盘」字段；旁路函数与它是同一个集合，层 0 每个状态可以逐状态断言两边相等（又一个差分口子，不占不变量位）。
- H6 取「自愈」：可写挂载之后那一批镜像上，旁路函数交回的集合应为空（挂载写过的盘）；层 0 的多次挂载流可以断言。V-池 本身不受影响（它只管全池皆废）。
- 两个结局下「一块盘全废」都不该是 I-8.5 违例——H6 两个候选都把它当容错射程内的事。
对不上的只有一处：H6 若取「自愈」且要求「自愈重写与 H4 的 S-修 是同一个写」，那次重写落在哪个发布序列位置，会动 D22 已定项 8 与 D23 已定项 3 的发布顺序（tail 是发布最后一步）；我没读 H4，这一格交主 agent 与那一轮判决对。

### A4.4　C466：几块盘的系统配置设备数对不上

- core：`crates/singlefs-core/src/recovery.rs:1025`「|| previous.immutable.device_count != best_on_device.immutable.device_count」，与 fsid 不同同一个出口 `SystemConfigurationsDisagree`，整池挂不上。checker：`device_count` 已解（`lib.rs:279` 附近读偏移），walk 里无命中；fsid 那一半有 I-1.4 接住：`crates/singlefs-checker/src/walk.rs:5774`「view.filesystem_identifier == geometry.filesystem_identifier,」。
- 措辞（新号，下一个空号是 I-7.15：`grep -n 'I-7\.15' .claude/kb/invariants.md` 无输出）：

> 各盘所选的系统配置槽（自证过、世代号最大者）里的设备数相同。

  形态抄 I-1.4 那个 `for (device, view, _) in &chosen` 循环（`walk.rs:5774` 前后），多一格 `device_count` 比较；约 15 行。对 core：core 是这一句的原型（`recovery.rs:1025`），两边同表。
- 「等于盘表」那半句**不要**：`crates/singlefs-core/src/mount.rs:3485`「if devices_handed_in != device_count_in_the_system_configuration {」比的是调用方交进来的盘数，只在可写挂载与挂着之后收盘表的入口做（`crates/singlefs-core/src/mount.rs:3334`「/// 「挂着之后收盘表的每个入口……核三样」的第二样）。可写挂载与挂着之后收盘表的入口经 [`device_table_disagreeing_with`] 调它，」），只读挂载不比；checker 手里的盘列表是「镜像里有几块盘」，降级镜像（掉了一块盘）里它小于设备数是合法的。硬写成「等于盘表」会把合法降级判红。
- 坏镜像：盘 1 的两个系统配置槽里偏移 143 的设备数改成 3、重封（偏移 = `SYSTEM_CONFIGURATION_FSID_OFFSET + 16 + 20 + 1 + 4`，`crates/singlefs-checker/src/lib.rs` 里读 `device_count` 的那一行）；只红 I-7.15（fsid 循环与它同形，不受影响），core 报 `SystemConfigurationsDisagree`。
- 状态：C466 标已还；变异行两条（判定恒真、不调这一判）。

**状态列（I-8.5）**：「已实现（全池皆废判红；单盘全废不判违例，旁路函数报出）」；C367 标已还，C461 的「仍欠」那半改成「旁路函数」；C463 归 H6。

**推翻条件**：
- 「V-池 不误红」：找到一份 core 报 `NoValidSystemConfiguration` 之外的失败、而 checker 拆分后仍判 I-8.5 红的镜像（比如某个槽 magic 与校验和对、fsid 是别的池的——core 的 fsid 过滤在 `choose_system_configuration` 之后，我没核它先于「自证」还是后于）。
- 「层 0 无全撕状态」：层 0 的任一撕裂展开里出现一块盘两槽同时撕（那就是四槽全撕不可能、两槽全撕可能，V-池 仍不误红，但 V-盘 的出局理由 2 也一并要重看）。
- 「加第四个 verdict 成员太贵」：54 个文件里的匹配大多带通配臂（我只 grep 了引用，没逐个看 match 形状）——真要加时编译器会列出来，成本可能比我说的低。

## A5　树 ID 水位（C147、I-7.8）

### A5.1　「两种取法」其实是三个量，不是同一个谓词的两种读法

| 量 | 在哪 | 干什么 |
|---|---|---|
| checker 的 I-7.8 | `crates/singlefs-checker/src/walk.rs:6344`「let watermark = roots」起：根环里全部自证过的根的水位取 max，要大于 `seen`（扫描到的树 ID 并上走过的树表条目：`crates/singlefs-checker/src/walk.rs:6361`「seen.extend(walk.tree_identifiers_in_tables.iter().copied());」）；判 `crates/singlefs-checker/src/walk.rs:6363`「judgements.judge("I-7.8", watermark > highest_seen,」 | 读方向，只看盘上的根 |
| core 的冷走读判 | `crates/singlefs-core/src/recovery.rs:3539`「if entry.tree.0 >= root.tree_identifier_watermark {」：树表条目的树 ID 与**所走那条根自己的**水位比；重放出来的那一版的水位是记录新根段里的：`crates/singlefs-core/src/recovery.rs:3219`「tree_identifier_watermark: record.new_tree_identifier_watermark,」 | 读方向，逐版 |
| core 的取水位 | `crates/singlefs-core/src/recovery.rs:1203`「pub fn highest_tree_identifier_watermark_in_the_ring<Reader: PoolReader + ?Sized>(」，内部先取全部记录新根段水位的 max（`crates/singlefs-core/src/recovery.rs:1212`「record.new_tree_identifier_watermark)」）再并上根环；给新实例第一次发布用（`crates/singlefs-core/src/mount.rs:1133`「fn tree_identifier_watermark_of_the_ring<Device: BlockDevice>(」） | **写方向**：新根要带多高的水位，不是判镜像 |

所以「checker 只取根、core 并上记录」不是两个读法在打架：core 那一个是写路径的义务（新实例的第一次发布不许把水位拉低），checker 判的是这个义务落成盘上根之后的结果。事实表末尾那句「方向是 checker 更严（多红），不会漏红」的方向我同意：checker 的门槛不高于 core 写入时取的，门槛低 ⇒ 更容易红。

### A5.2　在「根槽读不出而记录在」的合法镜像上 checker 会不会误红：论证不会（推的，没造镜像）

误红要求：镜像合法，`seen` 里有一个树 ID N ≥ 根环最大水位 W。逐个来源过：
1. 单元头扫描来的 N：`scanned_tree_identifiers`（`walk.rs` 里，读法甲那一版，用户 2026-09-14 收尾弹窗定甲、2026-09-24 补第二道，写在 `invariants.md` I-7.8 行的「checker 读法」一段）不数两类孤儿——诞生代号超过根环最大 checkpoint_txg 的，与实例表行说没发布过的。「记录已落、根槽没落」的那次发布（txg T+1）诞生代号 T+1 > 根环里读得出的最大 txg T，被第一类挡掉。
2. 走读树表来的 N：主走读只走候选根与「已被恢复施加过、根槽读不出」的那几版（`versions_applied_only_by_records`，`walk.rs:5577`）；后者要求最新根的实例表里有它那个实例的行。那行是写行那次发布（新实例第一次发布）写的，那次发布的根就带着 `max(根环, 记录)` 的水位（`mount.rs:1133` 那个函数），所以最新根的水位 ≥ 那条记录的水位 > N；那行读不到（那次发布的根读不出）时这一版不入走读。
3. 被抛弃的根：走读在另起的 `Walk` 里、树 ID 不进主 `seen`（`walk.rs:5682` 起，A1.4 引过）。

三条都过了没有缺口，唯一的洞是 C342 自己写的那一格：「根与它那条记录都读不出」，新实例取号时下界拉低。那一格盖不住，但被第二类孤儿规则（行说 Ti、诞生代号 > Ti 的不数）挡在 `seen` 之外，所以 checker 不红也不该红。
**这不是「造不出」**：正文第五节要求「两次抽样都没造出才算」，我这一份是论证、不是造镜像；攻方腿造出一份误红镜像即推翻这一节。

### A5.3　要不要改 I-7.8 的措辞把记录新根段并进来：不改

- 并进来的效果是门槛升高（记录水位常高于根环水位），**变宽松**：新实例第一次发布若忘了并记录（C342 那个 bug），根的水位偏低，只要记录还在环里 checker 就不红——而记录会被转圈盖掉，水位低的根与已发过的号一起留下来。根上的水位才是持久的义务，I-7.8 的题面就是「根记录」的水位；只看根更严也更对。
- 记录一侧的义务另有一个候选，零轮，本轮**不做**：「每条自证根的水位 ≥ 环里 txg 更小的每条自证记录的新根段水位」，它是 core 那个写方向义务落成盘上根的形态，能抓 C342。占 K1 一个位；且记录新根段的水位与 F 目前没有任何 checker 判定，那是 b 轮代字段（C395）的对象（事实表第八节「记录新根段的 F / 水位 / 两条指针」那一行：`new_tree_identifier_watermark` 在 walk.rs 无命中），两轮要合着定。
- core 的冷走读判（`recovery.rs:3539`）拿「重放出的那一版自己的水位」比树表条目，checker 没有对应的逐版判；R-独（A1.4）的重放版走读可以顺手带一格。这也交 b 轮与主 agent 合并处理。

### A5.4　「根轮出环」坏镜像的造法：第一版建不了「每个 checkpoint 一棵树」

- 只有 `publish_first_file` 发树 ID（`crates/singlefs-core/src/transaction.rs:2390`「pub fn issued_from_watermark(」一次发八个）；其余发布路径一个都不发，只照抄上一版的水位：`crates/singlefs-core/src/transaction.rs:4748`「// 这次一个树 ID 都不发：上一版的水位就是根环里全部根记录的 max（本会话每次发布只照抄或推高它，见 `publish_first_file`）。」（覆盖写；顺序写与新建 inode 两条同样，`transaction.rs:4809`、`transaction.rs:4873` 的 `tree_identifier_watermark: previous.root.tree_identifier_watermark,`）。所以 C147 写的「连发 R + 1 个 checkpoint 各建一棵树」在 `crates/` 今天造不出。（回退到无文件那一版再发第一个文件版本可以再发一次号，C511；那条路每轮两次发布，要的发布数更多，没用。）
- 换一个能造出来、且钉住同一件事的形态：**写方向的变异 + 真历史**。C147 想钉的是「水位是累计高水位，不是本 checkpoint 发出的最高号」。写方向的错法只有一种可行的：第一个文件版本之后的发布把水位写成 0（或别的低值）。造法：
  1. 在文件发布之后连做 R × S = 24 次覆盖写（同一次挂载内，`crates/singlefs-harness/tests/root_ring_turn_in_one_mount.rs:70` 的 `turning_the_root_ring_in_one_writable_mount_...` 已经有「一次挂载转一圈根环」的骨架）。槽的落法：`crates/singlefs-core/src/root_ring.rs:114`「region: checkpoint_txg.0 % ROOT_RING_REGIONS,」、`crates/singlefs-core/src/root_ring.rs:115`「slot: (checkpoint_txg.0 / ROOT_RING_REGIONS) % slots_per_region.count(),」，R = 3、S = 8：第一个文件版本的根（txg t0）在 t0 + 24 那次发布被盖掉。
  2. 变异：`crates/mutations.tsv` 一行，文件 `crates/singlefs-core/src/transaction.rs`，原文取覆盖写那两行（注释行 `transaction.rs:4748` 加下一行 `tree_identifier_watermark: previous.root.tree_identifier_watermark,`，这一对在文件里恰好一次；第二行单独出现三次，不能单独当锚点），替换成 `tree_identifier_watermark: 0,`；必须红的用例是上面转圈的新用例。
  3. 断言（钉绝对值，不只互比）：转圈之前（根环里还留着 t0 那条根）I-7.8 在变异版上是绿的；转圈之后红，红在「根环水位最大 0，盘上出现过的最大树 ID 15」这样的绝对数上（数字待实现时取：`checker_known_bad_images.rs:5029` 的 `assert_lowering_the_watermark_reddens_only_the_watermark_invariant` 就是这个断言形态）。没变异的真实现上同一条流转圈前后都绿。
- 阳性对照（取 max 的是整个根环，不是最新根）：干净镜像上只把最新根的水位压低、老根不动 ⇒ I-7.8 仍绿；`checker_known_bad_images.rs:5004` 的 `set_the_tree_identifier_watermark_of_every_root` 是改**每一条**根，不是只改最新那条，我没读到只改最新根的阳性对照（按函数名搜的，没读用例体），实现员先核有没有。
- 手工伪造替代路线：在转圈之后的镜像上把每条根的水位改低——与现有坏镜像（`checker_known_bad_images.rs:5029` 起）是同一个谓词，钉不住「累计」，所以不选。

**状态列**：I-7.8 行不改；C147 标已还的条件 = 上面那条变异 + 用例入库，且前后各一条绝对值断言。

**推翻条件**：
- A5.2：任一合法历史上 `seen` 里有 N ≥ W——攻方腿造出，或层 0 的全量里出现 I-7.8 红（今天层 0 全量对 I-7.8 是 0 红：`invariants.md` I-7.8 行里的「层 0 全量 262165 个状态里 261891 个判红」是**改孤儿规则之前**的字面读法的数，现在的读法没有这个数，我没查 E152 最近一次的 I-7.8 红数）。
- A5.4：`crates/` 里另有发树 ID 的路径（`grep -n 'issued_from_watermark' crates/singlefs-core/src/*.rs` 除 `publish_first_file` 之外还有调用点）。

## A6　指针 extent 偏移（C581）

### A6.1　今天的样子

- 头部字节序：core 读头部的顺序是 MAC + nonce 28 字节、算法类型 1、压缩算法码 1、压后长度 2、extent 偏移 2，再是出生树 8——`crates/singlefs-core/src/pointer.rs:126`「struct PointerHeadAsRead {」与它的 `read_from` 一路读下来，extent 偏移在字节 32..34，出生树从 34 起（checker 的 `parse_node_pointer` 读出生树也在 34）。
- core 不判它，注释自己写了：「/// 压缩算法码登记表第一版只有码 0「不压缩」，码 0 时压后长度恒 0（D9（加密） 已定项 14）。extent 偏移那 2 字节不在这里判：」（`crates/singlefs-core/src/pointer.rs:108`，下一行「读到非 0 怎么处置没有条款（第一版写 0）。」）；读的地方 `crates/singlefs-core/src/pointer.rs:140`「reader.skip(2); // extent 偏移：第一版写 0，读到非 0 怎么处置没有条款，不判」。
- 判的入口 `judged()` 只判 MAC/nonce、算法类型、压缩码、压后长度四样（`crates/singlefs-core/src/pointer.rs:154`「fn judged(self) -> Result<PointerHead, PointerHeadFieldOutsideTheFirstVersion> {」）；只有两个读者走它（多层码 2 树内部条目、journal 新根段指针，D19 已定项 3 射程写着），别的五个读者走不判的 `PointerHead::read_from`（`crates/singlefs-core/src/pointer.rs:201`「pub fn read_from(reader: &mut ByteReader<'_>) -> Self {」上面的注释列了它们）。
- 条款：「extent 偏移那 2 字节读到非 0 怎么处置仍没有条款，记进欠账 C581（extent 偏移非 0 没有判据）」（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:64`）；同一行前半是 MAC/nonce 的处置：「加密关着时 MAC 16、nonce 12 恒 0，读者遇到非 0 判该指针所在的结构损坏」。
- 零命中：`grep -rn 'ExtentOffset\|extent_offset' crates/singlefs-core/src crates/singlefs-checker/src | wc -l` 输出 `0`（冻结副本）；checker 的指针判定只有一个入口函数 `crates/singlefs-checker/src/image.rs:699`「pub fn judge_pointer_mac_and_nonce_are_zero(」，只判 MAC/nonce，四个入口都调它（`invariants.md` I-2.4 行列了四个：码 2 节点指针、inode 内部条目子指针、实例表链指针、extent 数据指针）。

### A6.2　第一版非 0 判什么：同 MAC / nonce，判该指针所在的结构损坏

理由，三条都是仓里已有的做法，不是我的偏好：
1. 第一版恒 0 的保留位，读到非 0 一律拒收，是这个格式反复用的政策：根记录 flags「其余位第一版恒 0、非 0 拒收（已定项 17）」（`crates/singlefs-checker/src/lib.rs:211`「/// 根记录 flags 位 0 = 卸载记号（D22（单元原子性怎么合成） 已定项 7）；其余位第一版恒 0、非 0 拒收（已定项 17）。」）、journal 点名项 flags（`crates/singlefs-checker/src/lib.rs:803`「if entry[55] != 0 {」→ `NonZeroFlags`）、单元头 29 字节预留位。
2. extent 偏移的用途是「extent 距起点的偏移」：第一版指针指整个单元，偏移恒 0。读者遇到非 0 而悄悄丢掉，就是从单元头读、而写者想说的是从偏移处读——静默读错内容。这正是 D19 已定项 3 依据里用的判据（格式分支要把「用错了」变成「挂不上」）；用 `skip(2)` 违反它。
3. 与 MAC/nonce 同一处置，不多立一套：`invariants.md` I-2.4 行已经把「指针头部的 MAC 16 + nonce 12」并进「单元头 29 字节」那条规则（`.claude/kb/invariants.md:124`「单元头的这 29 字节与指针头部的 MAC 16 + nonce 12（D19（块指针的结构与宽度预算） 已定项 3）由同一条规则守」）。

### A6.3　写进哪条：不新立编号，扩 I-2.4 的指针一半；条款写进 D19 已定项 3

- 编号：I-2.4 的指针一半（不占 K1 的位）。I-2.4 那句「加密关着时恒 0」对 extent 偏移不成立（它与加密无关），措辞要单加：「指针头部的 extent 偏移 2 字节第一版恒 0（第一版指针指整个单元），读者遇到非 0 判该指针所在的结构损坏」。
- 条款：D19 已定项 3 射程末句改成上面这一句，C581 标已还。这是条款变更，交用户（本轮判决表里标零轮）。
- 顺带看到（不在 A6 范围，没做）：checker 对算法类型（字节 28）、压缩码（29）、压后长度（30..32）也不判（`grep -n 'algorithm\|compress' crates/singlefs-checker/src/image.rs crates/singlefs-checker/src/walk.rs` 无输出），而 core 的 `judged()` 判了、D9 已定项 14 写着判损坏。同一个函数顺带扩这四个字节几乎不加成本，是否并入交主 agent 与 b 轮的指针身份格一起看。

### A6.4　checker、core 怎么改，坏镜像

- checker：`PointerView` 加一个 `extent_offset: u16`（`read_u16(bytes, 32)`，`parse_node_pointer` 与 `parse_data_pointer` 各一行）；`judge_pointer_mac_and_nonce_are_zero` 里再判一格，说明写「指针头部 extent 偏移 2 字节有非 0 字节，第一版恒 0」，改名为 `judge_pointer_reserved_bytes_are_zero`（四个调用点跟着改名）。约 10 行。判定不改走读（「只判不断」）。
- core：`PointerHeadAsRead` 加 `extent_offset: u16`（`reader.skip(2)` 换成 `get_u16`），`judged()` 加 `ExtentOffsetNotZero { extent_offset: u16 }` 一支；只有走 `judged()` 的两个读者受影响，与 MAC/nonce 今天的射程一致。别的五个读者仍不判——这个不对称今天已经有（checker 判四个入口、core 判两个读者），改后不变；要不要把五个读者也换成判，是另一笔账（实审 A3a 的报告列了它们，`pointer.rs:199` 注释），不在 C581。
- 坏镜像：`crates/singlefs-harness/tests/checker_judges_reserved_bytes_and_widths_it_used_to_skip.rs` 已有两个造法函数，`root_ring_with_a_pointer_head_byte_set`（:703）与 `parent_with_a_pointer_head_byte_set`（:730），MAC/nonce 那一格遍历的是 `crates/singlefs-harness/tests/checker_judges_reserved_bytes_and_widths_it_used_to_skip.rs:778`「for byte_in_pointer_head in [0usize, 15, 16, 27] {」（MAC、nonce 的首尾）。extent 偏移的坏镜像 = 同样遍历 [32, 33]，断言那条指针所在的四个入口各红 I-2.4，别的不变量照旧（这个用例族原来就断言「只红 I-2.4」，我没读断言体，实现员先核）。
- 变异行：`judge_pointer_mac_and_nonce_are_zero` 里新加的那一格条件改 `true`（判定恒真），必须红的是上面扩出的用例；core 侧 `judged()` 里新支的条件改 `false`，必须红的是 core 的读者用例（照 MAC/nonce 那两个读者的用例形态，我没找具体名字）。

**状态列**：I-2.4 行状态列追加「指针头部 extent 偏移 2 字节判该指针所在结构损坏（四个入口）」；C581 标已还。

**推翻条件**：
- A6.2 第 2 条：D19 或 D18 里有一处写着 extent 偏移非 0 在第一版是合法的（比如打包容器的子 extent 偏移）——`grep -rn 'extent 偏移' .claude/kb/decisions/` 我看到的只有 D19 已定项 3 与已定项 11 的偏移表；打包容器（码 3）里的偏移是记录内字段，不在指针头部（推的，没逐条读 D18 已定项 11）。
- A6.4「只判不断」：走读遇到非 0 偏移仍要跟着指针读单元——与 MAC/nonce 一样，checker 不因它停走（`image.rs:699` 上面注释写着「只判不断：指针指的单元照走」）。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| A1　I-4.1 | 冲突（措辞与实现） | 行里「系统配置指向的树」与实现不符（根住根环）；推 R-独：checker 自写重放（约 150–200 行）+ 另起 `Walk` 走读，不重放时报不适用归 I-7.2；最值钱的一步是层 0 里 checker 重放与 core `effective_root` 逐状态差分 |
| A1　I-8.4 | 冲突（kb 内部）+ 跑前判据触发 | 记录级恒真、重放函数级只剩「链长 > N」这一格，等于 I-8.3 的窗口那一半，写不出只红它的坏镜像 ⇒ 不在 checker 判；R-换 / R-形 二选一交用户，推 R-换；七·4 一行「checker 判定」与 D13 M3 / I-4.5 对不上 |
| A2　I-8.1 | 规则没说（口径注）+ 部分同义反复 | 不等式是推论，降级；三个字段等式与「同发布记录数 ≤ N」各占一格；口径注在实现上已答（取上界），T_dirty 夹取没落点；要 core 同判，是条款加实现变更 |
| A3　I-7.5 | 一致（可判），事实表一句推断不成立 | 判定宽度取声明值 max(pbs, io_min)，逐槽判，探测那一半仍欠 C388；mkfs 应事前拒（今天 pbs 4096 + io_min 3072 写到最后才炸、pbs 512 + io_min 3072 被接受） |
| A4　I-8.5 | V-盘 冲突出局；V-池 一致；V-报 不推荐 | V-池：全池皆废判红，单盘全废走旁路函数不进违例通道；要把 `NotChoosable` 拆成两类；C463 与 H6 两个结局都对得上 |
| A4　C466 | 一致 | 新号 I-7.15，「各盘所选槽设备数相同」，形态抄 I-1.4；「等于盘表」不要（降级镜像会误红） |
| A5　I-7.8 | 一致 | 三个量不是两个读法；论证合法镜像上不误红（没造镜像）；措辞不改；「根轮出环」用写方向变异 + 转 24 次覆盖写造，发布路径造不出「每 checkpoint 一棵树」 |
| A6　C581 | 规则没说 | 非 0 判该指针所在结构损坏，扩 I-2.4 指针一半、条款写进 D19 已定项 3；core 同改 `judged()` 两个读者；不占 K1 的位 |

## 没做什么

- 不出判决、不替主 agent 采纳：上面每处「推荐」都是我的正推结论，被攻过零轮；腿提的新形态（I-7.15、I-4.1 的新措辞、I-8.1 的两格、A6 的措辞）按正文第五节一律算零轮。
- 没编译、没跑任何测试、没跑 mkfs：A3 mkfs 的结局、A5 不误红、A4「层 0 无全撕状态」、A6 坏镜像「只红 I-2.4」都是静态读代码与算术，标了「推的」；`a3_alignment.py` 是唯一跑过的脚本（纯算术）。
- 没读：禁读清单里的一切；`crash_injection_record_checker_sees_every_publish_across_a_second_crash.rs`、`whole_device_loss_after_warm_up.rs` 与各坏镜像用例的断言体只看了名字与开头；b 轮与 ring 轮的腿报告（`_m3-ring-r1-body.md` 的 H6 一段是正文，读了）。
- 没量：R-独 的行数是从 core 的行数推的、不是写出来的；层 0 加一次 checker 重放的挂钟与 GPU 移植量没量（K1 只数了位数）；I-7.8 在 E152 最近一次全量上的红数没查。
- 没动：任何仓内文件（除本报告与 `research/prompts/m3-inv-a-r1-sonnet-model/` 下两份脚本产物）。
- 不归这条腿的：I-8.3 的实现（R-独 的重放函数顺带就是它，正文没把它分给任何一格，`invariants.md:86` 还是「未实现」——主 agent 要不要并进本轮，交你定）、b 轮的代字段与指针身份、H1–H6（ring 轮）。
- 草稿：`/tmp/claude-1000/m3-inv-a-r1-sonnet/` 下只有 `find.py` 与几张引文查行表（文本，没有编译目录、没有仓副本），不在仓里，也不需要入库。
