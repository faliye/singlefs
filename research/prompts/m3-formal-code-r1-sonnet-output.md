# m3-formal-code-r1 云端正推（Sonnet）报告（2026-10-02）

立场：从已定条款与 `crates/` 今天的实现推「应该是什么」，再与被判的改动逐格比；一致就写一致，不一致就把两边整行并排抄出来。
范围：K1–K4（背景材料第三节），对象 C1–C7（第一节表）。方案按 `crates/` 今天的实现来谈。

## 0　读法与约定

- 代码按冻结副本 `/tmp/claude-1000/m3-formal-code-r1/tree/` 读；引文写 `路径:行号`，行号是那份文件自己的，每句在写进报告前用 `grep -nF` 现取。kb 引文写 kb 文件名加它自己的行号。
- 被引的 kb 条款多是表格的一整行（几千字），引文取其中一句、不是整行；整行见文件。
- 「改前」的代码：补丁底（第 7 批、第 10 批合入之前的主工作区）在冻结副本里已不在；`git show HEAD` 的 `recovery.rs` 与补丁底不是同一份（主工作区 `recovery.rs` 相对 HEAD 有别的批的暂存改动，`git diff HEAD --stat` 为 1 file changed, 333 insertions(+), 255 deletions(-)），所以「改前」只取补丁里 `-` 行（`research/prompts/_m3-formal-code-r1-diff.md` 第 3322–3336 行、第 3377–3389 行）与 HEAD 里同文的行互证，不拿 HEAD 的行号当补丁底的行号。
- 我跑的只有读文件、`grep`、`awk`、`git show`、自写的 python 子串计数（脚本在 `research/prompts/m3-formal-code-r1-sonnet-model/` 下：`anchors.py`、`callgraph.py`、`hits.py`；脚本里的冻结副本路径写死为 `/tmp/claude-1000/m3-formal-code-r1/tree`）。没编译、没跑任何测试、没跑 Verus、没跑门禁、没跑 E165 / E170 装置。凡「红 / 不红」「X verified」一律标来源：实现员自报（`/tmp/claude-1000/m3-formal-10-3/report.md`、`m3-formal-7-1/report.md`、`m3-formal-7-2/report.md`）或主 agent 的观测（`/tmp/claude-1000/m3-verus-rows-10-3/rows.out`），都不是我复核的。

## 1　K1　C3 胶水与核对不对得上

### 1.1　`choose_root`：行为与改前逐项相同吗

补丁 `-` 行（背景 diff 第 3322–3336 行）里改前是：遍历 `visit_valid_roots`，`best.is_none_or(|current| candidate_key > (current.checkpoint_txg, current.instance))` 才换；HEAD 里同文的行：

```
$ git show HEAD:crates/singlefs-core/src/recovery.rs | grep -nF 'best.is_none_or(|current| candidate_key > (current.checkpoint_txg, current.instance))'
1161:            if best.is_none_or(|current| candidate_key > (current.checkpoint_txg, current.instance))
```

| 项 | 改前 | 改后（冻结副本） | 判 |
|---|---|---|---|
| 读哪些槽、什么次序 | `visit_valid_roots_with_ring_slots` 里 `for ring_slot in every_root_ring_slot(immutable_sizes)`（HEAD 第 1080 行，补丁没动那一段） | `crates/singlefs-core/src/recovery.rs:1195`：「pub fn every_root_ring_slot」同一函数；胶水 `every_root_ring_slot(&immutable.sizes).into_iter().map(...)` | 一致 |
| 读几次 | 每槽 `read_root_ring_slot` 一次 | 每槽 `read_root_ring_slot` 一次（`.map` 里调，`.collect()` 收齐） | 一致（读盘次数与次序不变；这一条我只读到代码，没测读盘计数） |
| 读不出 / 自证不过怎么跳 | HEAD 第 1089 行 `Unreadable => continue`，`NotSelfVerified => {}`：都不交给 `visit` | 两种都映射成 `None`，核对 `None` 不择 | 一致 |
| 比较键 | `(checkpoint_txg, instance)` 元组的 `>`；`CheckpointTxg`、`InstanceGeneration` 都 derive 了 `PartialOrd, Ord`（`crates/singlefs-core/src/address.rs:25`、`:29`）⇒ 字典序 | `crates/singlefs-replay-prefix/src/lib.rs:4002`：「Some((_, chosen_key)) => candidate_key.checkpoint_txg.0 > chosen_key.checkpoint_txg.0」接 `crates/singlefs-replay-prefix/src/lib.rs:4004`：「&& candidate_key.instance.0 > chosen_key.instance.0),」 | 一致（u64、u32 的字典序，等价） |
| 严格大于才换 | 是 | 是（核里同一写法） | 一致 |
| 错误怎么交 | 返回 `Option<RootRecord>`，没有别的出口 | 签名同（`crates/singlefs-core/src/recovery.rs:1250`：「pub fn choose_root(」），多一个 `.expect`：`crates/singlefs-core/src/recovery.rs:1285`：「择根核只交回自证过的槽」。核的 ensures 说所选槽在入参里是 `Some`（`crates/singlefs-replay-prefix/src/lib.rs:975` 起 spec `root_chosen_among_the_first_slots`），所以 `.expect` 在核的证明成立时走不到 | 一致；新增一处只在核错时才响的 panic 口 |
| 三处调用方 | `mounted_read.rs`（第 562 行）、`mount.rs`（第 1237、1243、4549 行）、`recovery.rs`（`recover`，第 4316 行） | 签名没变，不用改 | 一致 |

- 摊键：`crates/singlefs-core/src/recovery.rs:1278`：「PrefixKernelCheckpointTxg(root.checkpoint_txg.0),」与 `crates/singlefs-core/src/recovery.rs:1279`：「PrefixKernelInstanceGeneration(root.instance.0),」把两项原样搬过去，没有丢位、没有换序。
- 判 K1-1：**对上**（行为同义）。推翻它的现象：在同一个池镜像上，旧版与新版 `choose_root` 交回不同的根，或读盘日志里读槽次数 / 次序不同；我没造这样的输入。

### 1.2　`scan_journal` 先收齐再建表：行为与内存形状

补丁 `-` 行（背景 diff 第 3386–3388 行）改前是 `records.entry((record.instance, record.counter)).or_insert(record)`；HEAD 同文：

```
$ git show HEAD:crates/singlefs-core/src/recovery.rs | grep -nF '.or_insert(record);'
2923:                    .or_insert(record);
```

- 读盘次数与次序：扫环循环（`for device in reader.device_identities()`、`for offset in offsets`）一字未动，只把 `.entry().or_insert()` 换成往 Vec 里 `push`（`crates/singlefs-core/src/recovery.rs:3052`：「scanned_records_in_read_order.push(record);」）。一致。
- 留哪一份：`crates/singlefs-core/src/recovery.rs:3062`：「let first_copies = first_copy_of_each_record_key(&scanned_record_keys);」，再 `crates/singlefs-core/src/recovery.rs:3065`：「if first_copies.get(&(record.instance.0, record.counter)) == Some(&position) {」才插进最终 `BTreeMap<(InstanceGeneration, u64), JournalRecord>`。核是先查后插（`crates/singlefs-replay-prefix/src/lib.rs:4045`：「if !first_copies.contains_key(&record_key) {」），键已在就不动 ⇒ 与 `or_insert` 同义。最终表的键型与插入次序序无关（`BTreeMap` 按键排序），遍历次序同改前。一致。
- 错误：`scan_journal` 不报错，签名同。

内存形状（背景写「journal 环第一版 1024 槽」）：**这个前提对不上。**

```
$ grep -nF 'pub const JOURNAL_RING_START_SLOT: u64 = 1024;' crates/singlefs-format/src/lib.rs   （冻结副本）
216:pub const JOURNAL_RING_START_SLOT: u64 = 1024;
$ grep -nF 'pub const JOURNAL_RING_DEFAULT_BYTES: u64 = 805_306_368;' crates/singlefs-format/src/lib.rs
220:pub const JOURNAL_RING_DEFAULT_BYTES: u64 = 805_306_368;
$ grep -nF 'pub const JOURNAL_RECORD_BYTES: u64 = 4096;' crates/singlefs-format/src/lib.rs
198:pub const JOURNAL_RECORD_BYTES: u64 = 4096;
$ python3 -c "print(805306368//4096, 2*805306368//4096)"
196608 393216
```

1024 是环的起点槽号（16 KiB 槽），不是槽数；默认环 768 MiB，记录槽 4096 字节，每盘 196 608 个记录槽。
- 旧版：表里只留每个键的第一份；解出来又被 `or_insert` 丢掉的重复份当场释放。峰值 ≈ 不同键的条数 × 一条记录（含它的 `named` 向量）。
- 新版：Vec 先收齐每块盘上每一份自证过的记录，另加 `Vec<(u32, u64)>` 键表与核里的 `BTreeMap<(u32, u64), usize>`，最终表建完 Vec 才随 `into_iter` 释放。峰值 ≈ 读到的总份数 × 一条记录。双盘镜像（`.claude/kb/decisions/22-单元原子性怎么合成.md:175`：「journal 环在两块盘上互为镜像」）整环写满时总份数是不同键数的 2 倍。上界按槽数算：2 盘 × 196 608 = 393 216 份（推的，没量；一条记录在内存里多大没编译查，`named` 向量最多挂几十个点名项，所以不给字节数）。
- 判 K1-2：行为 **对上**；峰值内存是旧版的 D 倍（D = 读到该环的盘数）**不是条款问题而是取舍**，实现员已在 10-3 报告第一节「2. 扫环核」自报「双盘镜像约为原来的两倍」（`/tmp/claude-1000/m3-formal-10-3/report.md`），我的推导与它同向。读阶段缓存不收全零块（`crates/singlefs-core/src/mount.rs:4434`：「if bytes.iter().any(|byte| *byte != 0) {」）与此无关，那是块层读缓存，不是这张表。
- 什么现象推翻：一个会在改前过、改后 OOM 的池（环写满、点名项多），或 Vec 收齐前被别处释放的证据。

### 1.3　两个键完全相同的槽取前面那个：条款没写，写死在哪、改了谁会红

条款侧：D22 已定项 7 只写择新键，D22 已定项 24 只写「先验全部、再择新」：

- `.claude/kb/decisions/22-单元原子性怎么合成.md:138`：「择新在 checkpoint_txg 平局时按它高者赢」
- `.claude/kb/decisions/22-单元原子性怎么合成.md:459`：「必须**先逐个验证全部候选、再在有效者中按代号择新**」

两处都没有「键完全相同取哪一个」。实现侧写死在**核**里，三处：

- 可执行判定：`crates/singlefs-replay-prefix/src/lib.rs:4002`：「Some((_, chosen_key)) => candidate_key.checkpoint_txg.0 > chosen_key.checkpoint_txg.0」与 `crates/singlefs-replay-prefix/src/lib.rs:4004`：「&& candidate_key.instance.0 > chosen_key.instance.0),」，严格大于才换 ⇒ 前面的留。
- spec 第三句：`crates/singlefs-replay-prefix/src/lib.rs:994`：「0 <= slot_index < chosen_slot && (#[trigger] root_slots[slot_index]) is Some」，该句之前的注释 `crates/singlefs-replay-prefix/src/lib.rs:975`：「条款没写键相同取哪一个」说得老实。
- 单测：`crates/singlefs-replay-prefix/src/lib.rs:4248`：「fn of_two_slots_with_the_same_choice_key_the_earlier_slot_is_chosen」。

胶水里没有这条规则：胶水只把槽按 `every_root_ring_slot` 的次序摊成 Vec、再按核交回的下标取（`crates/singlefs-core/src/recovery.rs:1283`：「choose_root_slot(&choice_keys_in_ring_order).map(|chosen_slot| {」）。所以「前面」是读根环次序里的前面，由两处合起来定：核的严格大于与胶水的摊入次序。

改了谁会红（行号是冻结副本里的）：

| 改法 | 红的是谁 | 来源 |
|---|---|---|
| 核里 `>` 改成 `>=`（原型 m1_tie_last_wins） | Verus 行 149 `proof_errors=1`；`crates/mutations.tsv` 第 1822 行点名的核单测 | 实现员自报（`/tmp/claude-1000/m3-formal-10-3/report.md` 第三、四节）；主 agent 的 `rows.out` 里 149 行 `match=True` |
| spec 第三句改成取槽序最后 | Verus 行 157 `proof_errors=1` | 同上 |
| 胶水把摊入次序倒过来，或下标取错一格 | **没有**：`crates/mutations.tsv` 里 `every_root_ring_slot(&immutable.sizes)`、`self_verified_roots_in_ring_order` 都 0 命中（`grep -c` 输出 0）；`crates/singlefs-harness/tests/root_choice_on_a_tied_checkpoint_txg_takes_the_higher_instance_in_a_later_slot.rs` 的两个槽键不同（txg 同、实例代号不同），次序倒过来结果不变，抓不到 | 我 `grep` 的；倒次序会不会被别的用例碰巧抓到，没跑 |

判 K1-3：核一侧 **对上、有两层会红**；胶水一侧的次序与下标是 **没钉的口子**。口子的现实意义小（两个槽键相同而内容不同本身要先有一个坏池 / 重发造出的同 txg 异内容根，条款没写它会出现），但「前面那个」这句话只在核里被钉住、在真池的根环上没有用例钉。推翻它的现象：找到一个现有 harness 用例在胶水倒次序时红。

### 1.4　「胶水拿所选槽的那条根去调重放」那一步不在 Verus 里，谁罩

- 这一步的代码：`crates/singlefs-core/src/recovery.rs:4316`：「let Some(root) = choose_root(reader, &system_configuration) else {」，`replay_journal` 里 `crates/singlefs-core/src/recovery.rs:3243`：「PrefixKernelInstanceGeneration(root.instance.0),」、`crates/singlefs-core/src/recovery.rs:3244`：「PrefixKernelCheckpointTxg(root.checkpoint_txg.0),」把所选根的两项交给 `replay_core`；另两处调用方 `crates/singlefs-core/src/mounted_read.rs:562`、`crates/singlefs-core/src/mount.rs:4549`。
- 推论自己说它不罩：`crates/singlefs-replay-prefix/src/lib.rs:2979`：「胶水真的拿所选槽的根去调重放」那一句注释（行内接着写 recover、读阶段不在 Verus 里）。
- 谁罩：(a) `crates/mutations.tsv` 里没有改「交给 `replay_journal` 的根」的行（`&chosen_root,` 0 命中）；R3-9 三行（第 1646–1648 行）改的是 `verify_named_units`，不是根。(b) E165 装置不调 `choose_root`：冻结副本 `e165_replay_bounded_enumeration.rs` 里 `grep -c choose_root` 输出 0；它把所选根直接交给 `replay_journal`。(c) 层 0 与崩溃放量的 CPU 判器在每个崩溃状态上调 `recover`，D13 已定项 1 M1 一行这样写：`.claude/kb/decisions/13-验证路线.md:31`：「只有层 0 与崩溃放量的 CPU 判器在每个崩溃状态上调 `recover` 罩它们」；它们归 checker 档，我没跑。(d) harness 用例经 `mount_read_only` / 可写挂载走这一串，具体哪一条在根传错时红，我没造变异验。
- 判：这一步现在由 (c)（checker 档，没复核）与 (d)（没验）罩；没有一条专为它写的变异行。C650 的 E165 一半（记录里 8-2，`records/2026-10-01-里程碑三形式化接入.md:426`）不在这一批 diff 里，不判。

## 2　K2　C2、C3 的证明在证什么

### 2.1　M3 不动点的前件与条款字面是不是同一句

条款字面（D13 已定项 1 的 M3 一行）：

- `.claude/kb/decisions/13-验证路线.md:33`：「把重建出的根当所选根、同一张记录表、同一个读者、同一个开关再重放一次，返回的根不变」
- `.claude/kb/decisions/13-验证路线.md:33`：「前提是所选根之上的记录不超在飞上限」
- `.claude/kb/decisions/13-验证路线.md:33`：「第二次因所选根 `checkpoint_txg` 顶格报 `CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR` 且一条没施加的，算相等」

核里的 spec（`crates/singlefs-replay-prefix/src/lib.rs:844`：「pub open spec fn replaying_from_the_landed_root_is_a_fixed_point」，前件在第 859–867 行）：

```
859:    replayed matches Ok(outcome) ==> (journal_txg_does_not_decrease_with_the_counter_within_an_instance(
861:    ) && the_reader_answers_each_position_one_way(back_chain_broken)
862:        && the_reader_answers_each_position_one_way(named_units_verified)
863:        && records_above_the_chosen_root_fit_the_in_flight_limit(
```

逐项对：

| 条款字面 | spec 里 | 判 |
|---|---|---|
| 同一张记录表 | 第二次取前缀传同一个 `records`（结构上同一个参数，不是前件） | 一致 |
| 同一个开关 | 同一个 `named_unit_verification`（同上） | 一致 |
| 同一个读者 | 前件两条 `the_reader_answers_each_position_one_way`（`crates/singlefs-replay-prefix/src/lib.rs:695`）：回调在同一下标上不会既交真又交假。条款的「同一个读者」落成「回调是确定的」；产品里回调是读盘，两次之间盘变了就出了前件 | 一致，读法是实现员的（7-2 报告第 202 行自己写了） |
| 前提是所选根之上的记录不超在飞上限 | `records_above_the_chosen_root_fit_the_in_flight_limit`（第 863 行起） | 一致 |
| 顶格报错且一条没施加算相等 | 结局 `replaying_again_from_the_landed_root_stays_put`（`crates/singlefs-replay-prefix/src/lib.rs:830`：「Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor) => landed_txg.0 == u64::MAX,」）；Err 没有 outcome，所以「一条没施加」是结构上成立的 | 一致 |
| （条款没有）I-8.12 | 前件第一条 `journal_txg_does_not_decrease_with_the_counter_within_an_instance`（第 859 行） | **spec 多一条前件，条款字面没有** |

- 多出来的一条：核里的注释说它出自判决：`crates/singlefs-replay-prefix/src/lib.rs:841`：「// 前提照第二轮判决第 6 行辩方那一句：I-8.12；同一个读者；所选根之上的记录条数不超在飞上限。」；D13 已定项 1 的 M3 一行里没有 I-8.12，`grep -c 'I-8.12' .claude/kb/decisions/13-验证路线.md` 输出 0。核的 spec 因此比条款字面的前提更强 ⇒ 证出来的是「I-8.12 成立时不动点成立」，条款字面是「不超在飞上限时成立」。实现员在 7-2 报告里承认：不带 I-8.12 时不动点成不成立没证（`/tmp/claude-1000/m3-formal-7-2/report.md` 第 201 行）。
- I-8.12 的条款今天在 `.claude/kb/invariants.md:101`：「同一块盘 journal 环里自证过的记录，同一实例代号的记录按 jsn 计数器从小到大，`checkpoint_txg` 不减」。背景材料写它「还没登进 invariants.md」，**对不上**：这一行在，状态写「未实现」。注意两点不同：它写「同一块盘」，核里的前件 `crates/singlefs-replay-prefix/src/lib.rs:684`：「核不判它：记录级的第二半与不动点把它写成前提」说的是整张（两盘合并后的）记录表；它是「未实现」，没有任何判器保证记录表满足它。
- 背景里写的「P0–P5」：背景材料与我读到的补丁、报告里都没有这几个编号的定义，我没法逐项对；上表是按 spec 的实际前件对的。
- 判 K2-1：**对不上一处**——spec 多一条 I-8.12 前件，条款字面没有（其余五项对上）。需要的是 kb 写回（把 I-8.12 写进 M3 一行的前提，或在 M3 里写明「在 I-8.12 之下」），不是改 spec。推翻它的现象：证明不用 I-8.12 也过（实现员的必红行 119「M3·前提·不动点去掉 I-8.12」期望 `proof_errors=1`，即去掉就红；那一行在主 agent 的 `rows.out` 里 `match=True`）。

### 2.2　C2「可执行代码不改」与必红表加 39 行

```
$ sed -n 1388,3182p research/prompts/_m3-formal-code-r1-diff.md | grep -c '^-[^-]'
0
$ sed -n 3183,3285p research/prompts/_m3-formal-code-r1-diff.md | grep '^+[^+]' | grep -vc '^+#'
48
```

7-2 补丁里 `lib.rs` 一侧一行删除都没有；对 `replay_core` 函数体只加了 `broadcast use {…}`（`crates/singlefs-replay-prefix/src/lib.rs:3624`：「broadcast use {」）与四条 ensures。必红表一侧：加 48 行数据行，其中 9 行是「原文、替换文不动、只改第五段期望」的旧行（我按名字把 `-` 行与 `+` 行配对，9 对第 1–4 段全同、第五段不同），净增 48 − 9 = **39 行**，与背景「加 39 行」一致。对上。

### 2.3　合判推论的前提「只有两个核的 ensures」：有没有空真的地方

推论 `crates/singlefs-replay-prefix/src/lib.rs:2982`：「pub proof fn corollary_recovery_replays_from_the_newest_self_verified_root」；它的 `requires` 第一条是择根核 `Some(chosen_slot)` 那一支的 ensures，其余六条是 `replay_core` 的五条 ensures（M2 句法层加 M3 四条）取所选槽的键。读法：

- 前提能不能同时成立：每一条都是一个被 Verus 证过的函数在某个返回值上必然成立的后置条件；`choose_root_slot` 在有一个 `Some` 槽的入参上返回 `Some(下标)`，`replay_core` 在合法入参上返回满足五条 ensures 的结果——两个核各自有一个见证，所以合取不空。实现员探过空真：在推论函数体头插 `assert(false)`，输出 `55 verified, 1 errors`、位置在插入的那一行（`/tmp/claude-1000/m3-formal-10-3/logs/vacuity-corollary.log`，实现员自报；门禁 verus-vacuity 格提交时会再判，我没跑）。
- 弱在哪：`recovery_replays_from_the_newest_self_verified_root`（`crates/singlefs-replay-prefix/src/lib.rs:1069` 起）是七项合取，前六项就是前提原样搬过来，只有最后一项 `replay_lands_no_earlier_than_any_self_verified_root`（`crates/singlefs-replay-prefix/src/lib.rs:1050`）是推论新证出来的。新证的内容只是择新序的传递：每条自证过的根 ≤ 所选根（择根核）≤ 落点（J1）。这不是空真，但内容很薄：它说的是「两个核各自的 ensures 拼起来推得出的东西」，推论自己的注释也这么写（`crates/singlefs-replay-prefix/src/lib.rs:2978`：「前提就是两个核交回的东西」）。
- `Err` 两支：最后一项写成 `replayed matches Ok(outcome) ==> …`，`replay_core` 报「顶格」或「末条标志有歧义」时这一项自动为真，没有内容；条款对这两支各有说法（顶格算相等，歧义报错），不在这一项里。
- 逐前提的必红：`verus-mutations.tsv` 第 160 行去掉择根那一条（`proof_errors=2`）、第 161 行去掉 M2 那一条（`proof_errors=1`）、第 162 行把结论取反（`proof_errors=1`）。**去掉 J1（`replay_lands_no_earlier_than_the_chosen_root`）那一条前提没有对应行**：最后一项真正依赖的正是它。去掉它结论里同名那一项也会证不出，所以几乎一定红，但表里没有一行钉着。同理 M3 的另三条前提（记录级、不动点、规则只容得下一个结果）也没有逐条去掉的行；它们各是结论里的一项，去掉就红是结构上的，不是表钉的。
- 判 K2-3：没发现空真；**一处缺行**（去 J1 前提）。推翻「没有空真」的现象：有一个让七项前提同时满足的见证不存在——可以由 Verus 里手写一个具体见证（`choose_root_slot` 与 `replay_core` 的真调用）直接 assert 前提成立来查；我没写。

### 2.4　必红表第 99、125 行期望 2 → 3 的理由站不站得住

行号是冻结副本 `crates/singlefs-replay-prefix/verus-mutations.tsv` 的；两行第五段今天都是 `proof_errors=3`（`awk` 取出：99 行「改判定·在飞上限不看…」`proof_errors=3`，125 行「M3·spec·S5 版本序：实例代号破平局改成严格小于」`proof_errors=3`）。

- 第 125 行：**站得住**。择根核的 spec 经 `crates/singlefs-replay-prefix/src/lib.rs:962`：「pub open spec fn root_choice_key_is_not_newer_than」接 `crates/singlefs-replay-prefix/src/lib.rs:741`：「pub open spec fn version_is_not_later_than」，没有自己的序；把 S5 的 `<=` 改成 `<` 就让「键相同的两个槽」在循环不变式里不成立，择根核多一处红。这是读代码就推得出的因果，不靠观测。主 agent 的 `rows.out` 里这一行是 `53 verified, 3 errors`、`match=True`。
- 第 99 行：**理由没有被证明，只是推测**。实现员原文写的是「推测是新加的 spec 进了 SMT 上下文」；三条报错都落在 `replay_core` 函数体里（`/tmp/claude-1000/m3-formal-10-3/rows/final2/row-25.log`：报错位置 3686、3720、3847、3899 行，`replay_core` 从 `crates/singlefs-replay-prefix/src/lib.rs:3540` 起、`choose_root_slot` 在第 3977 行）。这一行的期望在连着两批里反向改过：7-2 的补丁里 `-` 行 `proof_errors=3` 换成 `+` 行 `proof_errors=2`（我按名字配对的输出），10-3 又从 2 改回 3。门禁按条数相等判（`.claude/gate.d/verus-pins.tsv` 里 `rlimit` 之前的说明段：「格里比条数等于 M」），所以只要别处的证明改动让求解器多报或少报一条，这一行就会再翻。它不是条款问题：变异在两个数下都红；是维护成本。
- 判：125 **对上**；99 **理由不成立为因果、数是观测**——不影响「这一行必红」，影响「这个 M 长期稳不稳」。推翻 99 的「不稳」：之后再加几处无关证明，M 保持 3 不动。

### 2.5　C650 对不对得上，与 kb 里落后的两句

- C650 欠条的要求：`.claude/kb/checks-owed.md:519`：「放哪一边等用户定，先出两边的代价数」（用户 2026-10-02 定 Y6 两边都做；代码一侧写作 `crates/singlefs-replay-prefix/src/lib.rs:1047`：「// ===== 择根与取前缀合判（C650 的 Verus 一半；用户 2026-10-02 定 Y6 两边都做）=====」）。Verus 一半在这一批：所选根是择新序最大的有效根，落点不早于任一自证过的根，改坏择根序（平局取低、实例代号为主）的行 152、153、158、159 红（实现员自报，`rows.out` 里都 `match=True`）。**对上**。
- E165 一半（在一族状态上调真 `choose_root` 再调重放）不在这一批：`e165_replay_bounded_enumeration.rs` 里 `grep -c choose_root` 输出 0；调度记录把它排成 8-2（`records/2026-10-01-里程碑三形式化接入.md:426`）。所以 Y6「两边都做」今天只做了一边，「已确认的完整发布不丢」（M4 的射程句）这一读法没有在真 `choose_root` 上判过；推论证的是「落点不早于任一自证过的根」，比 M4「恰是 j*」弱。
- **kb 与代码不一致（冲突，两边整行）**。条款：`.claude/kb/decisions/13-验证路线.md:31`：「核之外：择根与扫环（`choose_system_configuration`、`choose_root`、`scan_journal`、`recover`）不在 Verus 里」；代码：`crates/singlefs-replay-prefix/src/lib.rs:3977`：「pub fn choose_root_slot」与 `crates/singlefs-replay-prefix/src/lib.rs:4022`：「pub fn first_copy_of_each_record_key」已在 Verus 里（取最大那一截与建表那一截；读槽、自证校验和、`recover` 仍在外）。同一行还写着 E165 装置「不调 `scan_journal` 与 `choose_root`」，第四段之后装置能调 `scan_journal`（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:1198`：「fn scanned_record_table_of(」）。这是 kb 落后于代码，写回归 kb 批；`grep -n 'choose_root_slot\|first_copy_of_each_record_key' .claude/kb/` 输出为空，没有任何 kb 页提到这两个核。

## 3　K3　C1、C6 的变异行与用例

我能核的是静态那几样：每行原文在源文件里恰好命中一次、点名的用例函数在点名的文件里、点名的过滤串对得上用例名；「点名的用例真能红」只有实现员的 `prove-red.sh` 自报（`/tmp/claude-1000/m3-formal-7-1/report.md` 变异对照表 12 / 12「抓到」；`/tmp/claude-1000/m3-formal-10-3/report.md` 第四节 9 / 9「抓到」），我没跑。静态结果（冻结副本，自写脚本 `research/prompts/m3-formal-code-r1-sonnet-model/anchors.py` 与我另写的点名检查（行号、点名用例函数在点名文件里，没留成文件））：

```
$ python3 anchors.py crates/mutations.tsv .     （原文 \n 换成换行后在文件里数）
NOFILE 1369 litmus/publish-returns-after-the-rotation-barrier.litmus
1830 ['anchor=0'] E168 P1 的环按记录数取档时向下取整（N = 88 仍取 1 MiB 的环，登记修订第 15 条）
1831 ['anchor=0'] E168 P1 找不到盘时原因字段恒打 none（登记修订第 15 条）
rows 1826 bad 2
$ python3 anchors.py crates/singlefs-replay-prefix/verus-mutations.tsv .
rows 153 bad 0
```

C1、C3、C6 点名的行（第 1645–1656、1822–1829、1634–1644、1657–1659、1668–1673、1813–1818 行）全部 `anchor=1`，点名的用例函数都在点名的文件里。`bad` 的两行（1830、1831）是 E168 的，不归这一轮；1369 行指的 litmus 文件不在冻结副本里（不是 `crates/`），也不归这一轮。

### 3.1　逐条：改的是不是条款说的那一处

| 行 | 变异改什么 | 压着的条款 | 对上吗 |
|---|---|---|---|
| 1645（R2-4） | 接链只比实例代号、不比计数器 | `.claude/kb/invariants.md:92`：「被重放的记录集合，其 `jsn` 严格连续」 | 对上；锚在 `lib.rs` 的接链那一判，用例是 `writable_mount_takes_a_second_instance` 里已有的断号用例（7-1b 报告：不补用例） |
| 1646–1648（R3-9） | 三处 `verify_named_units` 的 `true` 改 `false`（`mount.rs` 读阶段、`mounted_read.rs`、`recover` 的 Consult 一臂） | `.claude/kb/decisions/13-验证路线.md:27`：「以 `verify_named_units = true` 调 `replay_journal` 的三处」 | 对上；三处与条款点名的三处一一对应；第三处锚的是 `policy == JournalPolicy::Consult,` 而不是字面 `true`，与条款「Consult 策略验、另两个臂不验」相符 |
| 1649（R3-10，10-3 改锚到核） | 先查后插改成直接插（后到者赢） | `.claude/kb/decisions/13-验证路线.md:32`：「同一 (实例代号, 计数器) 在全部时间线上只对应一份内容」（M2 语义层前提） | 对上；条款只说「唯一」，没说「先到」；先到先得是第三轮判决 S3 定的（`lib.rs` 注释引 `research/prompts/m3-formal-r3-main-verification.md`），行 1828（胶水）、1829（核只收第一份）与它互补 |
| 1650（R3-1） | 重建根改抄所选根的卸载记号 | `.claude/kb/decisions/13-验证路线.md:32`：「其余字段等于所选根，卸载记号除外」 | 对上，**但只罩了四个「投影之外的字段」里的一个**，见 3.2 |
| 1651 | 读阶段缓存不存读到的字节 | `.claude/kb/checks-owed.md:517`（C648）：「让可写挂载的读阶段多读一次或换次序，这条用例必须红」 | 对上（缓存改动在块层读次数上看得出，用例点名可写挂载那份新文件） |
| 1652–1656 | 取前缀核读盘形状：S07 一类、H08、H09、S03、S07 | 同 C648；只读那一半见 D13 的 M2 行：`.claude/kb/decisions/13-验证路线.md:32`：「只读挂载几种形状的读盘次数用例罩」 | 对上；1653、1654 是攻方 H08、H09 原样，点名新加的只读用例 |
| 1822–1829（C3） | 择根核：平局取后者、不看实例代号、取第一个有效者、平局取低、实例代号为主；胶水摊键丢实例代号；扫环胶水读到的每一份都插；扫环核只收第一份 | D22 已定项 7 / 24（引文见 1.3） | 对上；胶水一侧只有两行（1827、1828），摊入次序与下标没有行（见 1.3） |
| 1634–1644、1668–1673、1813–1818（E165 r2） | 装置的判据 / 生成器 / 阳性对照表 / 参数解析；用例是装置二进制自己的单测 | D13 M2、M3 语义层；E165 页 | 改的是装置，不是产品；本格只核静态（锚点、用例在不在），红不红是 checker 档，没跑 |
| 1657–1659（E170 XE1–XE3） | 导出装置折回落地向量、流二段长表、摘要跳全零扇区 | D13 已定项 6 O3 | 同上，只核静态 |

### 3.2　R3-1：「拼根里投影之外的字段」只有一个有变异行

条款：`.claude/kb/decisions/13-验证路线.md:32`：「拼根里投影之外的字段（卸载记号、实例表指针、分配记录树根）没有变异行」是改前的现状句；这一批补上的只有卸载记号那一行（第 1650 行）。胶水里另外三个字段的拷贝：

- `crates/singlefs-core/src/recovery.rs:3282`：「filesystem_identifier: root.filesystem_identifier,」
- `crates/singlefs-core/src/recovery.rs:3292`：「instance_table: root.instance_table,」
- `crates/singlefs-core/src/recovery.rs:3296`：「allocation_record_tree_root: root.allocation_record_tree_root,」

`crates/mutations.tsv` 里这三句 `grep -c` 都是 0（前面的 `grep -n 'instance_table: root.instance_table\|…'` 在表里 0 行）。新用例 `rebuilt_root_keeps_the_chosen_roots_fields_except_the_unmount_marker.rs` 逐字段断言了这三项等于所选根（背景 diff 第 705–716 行），所以改坏它们多半会红，但**没有变异行钉着**；E165 的 J7a 判这三项（`e165_replay_bounded_enumeration.rs` 里 `judge_the_fields_beyond_the_projection`），归 checker 档。判：条款那句话「没有变异行」只补了四分之一。推翻它的现象：补三行变异、在 `prove-red.sh` 下有一行不红。

### 3.3　S03、S07：补敏感取样点，还是换了一个场景

规则：`.claude/rules/mutation-sampling.md:17`：「| 在**我选的取样点上**恰好同值，换一个取样点就不同 | **取样点不敏感** | **补一个敏感的取样点**，不是留档 |」。

- 事实：反向链回调多调一次（S03）或提交标记判与反向链判换次序（S07）在健康镜像上只多读 / 少读一次缓存里已有的记录，块层看不出；读缓存不收全零块：`crates/singlefs-core/src/mount.rs:4434`：「if bytes.iter().any(|byte| *byte != 0) {」。把上一条记录在 0 号盘的那一份写成全零，反向链重读每次落到块层 ⇒ 这两个变异在那个镜像上不再同值。这就是第三类的定义（在取样点上恰好同值、换一个取样点就不同），不是等价变异。
- 新用例的场景（0 号盘那一份全零、1 号盘那一份在）仍是同一条性质（每个候选在块层读几次，C648），也是一个合法的崩溃形状（记录槽在一块盘上没落）。判：**算补敏感取样点**，不算换场景。
- 一个代价要知道：这两条用例钉的读次数依赖「缓存不收全零」这个实现细节；哪天缓存改成收全零，S03、S07 又会回到不敏感，而用例会因为数变了红——红的是用例，不是变异行，提示是对的。

### 3.4　背景写「`crates/mutations.tsv` 第 173、174、177 行改前就 0 命中」

这一句在冻结副本与主工作区今天的文件上都**对不上**：

```
$ python3 research/prompts/m3-formal-code-r1-sonnet-model/hits.py /tmp/claude-1000/m3-formal-code-r1/tree
line 173 file crates/singlefs-core/src/recovery.rs hits_after_newline_conversion= 1 hits_raw_backslash_n= 0
line 174 file crates/singlefs-core/src/recovery.rs hits_after_newline_conversion= 1 hits_raw_backslash_n= 0
line 177 file crates/singlefs-core/src/recovery.rs hits_after_newline_conversion= 1 hits_raw_backslash_n= 0
$ python3 research/prompts/m3-formal-code-r1-sonnet-model/hits.py /home/fy5090/code/singlefs
line 173 file crates/singlefs-core/src/recovery.rs hits_after_newline_conversion= 1 hits_raw_backslash_n= 0
line 174 file crates/singlefs-core/src/recovery.rs hits_after_newline_conversion= 1 hits_raw_backslash_n= 0
line 177 file crates/singlefs-core/src/recovery.rs hits_after_newline_conversion= 1 hits_raw_backslash_n= 0
```

按表头约定（`crates/mutations.tsv` 第 2 行：「原文 / 替换文里的 \n 表示换行；原文在文件里必须恰好命中一次」）换行后各命中 1 次。实现员的数（`/tmp/claude-1000/m3-formal-10-3/report.md` 第五节「锚点命中数核对表」）是「before=0 after=0」，它的脚本 `tools/anchor-hits.py` 第 17 行同样做了换行换算，所以不是换算的差；它数的是它自己那一刻的 `base/recovery.rs`，那一刻可能带着别的会话的未完成改动。这一点我没法回到那一刻查。判：这三行**今天命中**，背景那句「改前就 0 命中」不能当作对冻结副本的描述。（它们不归这一轮，只点名。）

## 4　K4　C4、C5 装置

### 4.1　E165 读实现结局的启动自检挪进单测之后，变体下还有没有别的启动路径会读实现

- 这一批挪的：`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:7895`：「启动自检只放不读实现结局的」那一段注释之下，`R2_SELF_CHECKS`（同文件 `:7904`）只剩两项（生成器字段、参数解析）；`check_the_j6qn_control_readings`（`:7786`）、`check_the_j7_control_readings`（`:7808`）、`check_the_odd_marker_parity_turns_the_unmount_marker_controls`（`:7841`）各带 `#[cfg(test)]`，tests 模块里有同名单测（`:8781`、`:8786`、`:8791`）。
- 没挪的：r1 档的 `SELF_CHECKS`（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:7546`：「const SELF_CHECKS: &[(&str, fn())] = &[」）这一批 diff 没动，`main` 里 `for (check_name, check) in SELF_CHECKS` 在每一档都先跑它。我用自写的调用图（`research/prompts/m3-formal-code-r1-sonnet-model/callgraph.py`，正则切函数体、闭包求「能走到 `replay_journal` / `scan_journal` / `choose_root` / `recover` 的函数」）数：

```
$ python3 callgraph.py
tests idx 272247 funcs 192 direct ['call_replay_journal', 'scanned_record_table_of']
check fns that reach impl:
  check_root_plus_disk_on_a_bad_unit_copy -> ['state_with_one_bad_unit_copy_judged_under']
  check_the_fixed_point_control -> ['control_tally']
  check_the_in_flight_limit_control_on_both_rings -> ['control_readings_of']
  check_the_state_counts_of_the_smallest_sizes -> ['tally_of_a_domain_size']
  check_the_structure_control -> ['control_tally']
  check_the_two_monotone_readings_on_one_later_record -> ['tally_of_a_domain_size']
total check fns 16
```

  六项都在 `SELF_CHECKS` 里（行号 `:7265`、`:7385`、`:7400`、`:7447`、`:7461`、`:7480` 的 `fn check_…`）。另有多盘一族的启动自检 `check_the_multi_disk_state_counts_of_the_smallest_sizes` 用 `Implementation::TodayReplayJournal`（`:7923` 附近），和 `main` 里每个 (域, N) 之后的 `assert_the_pinned_absolute_values`。（调用图是正则做的，不含 `impl Trait` 方法与闭包间接调用；它给的是下限。）
- 这些读实现的路径会不会让变体在启动时 panic：读代码看，它们断言的是**与实现结局无关的量**——状态数、j\* 见证数、宽对数、「理想恢复」的违例数（`assert_the_pinned_absolute_values` 取的 `Counter::States`、`FarthestAtLeastOne` 等；`check_the_two_monotone_readings_on_one_later_record` 比的是 `IdealAppliedSetViolations`、`IdealLandedVersionViolations`，名字就是理想恢复）；实现被改坏时 `call_replay_journal` 里的 panic 被 `catch_unwind` 接住。**一处例外**：`check_the_fixed_point_control`（`:7447`）用的假结局 `SecondCallMovesOn`，第一次调用取真 `replay_journal`（`:3038`：「Implementation::TodayReplayJournal | Implementation::Control(_) => {」），断言 `J6PremiseHoldsViolations > 0`；变体若让真 `replay_journal` 在 L 域 N=3 的态上整片不交回根，J6 不被判、这个计数为 0，会在启动时 panic——与第二段 5 个变体 panic 同一类病，只是这一项没挪。
- 经验证据（不是我的）：E165 页记着挪完之后在冻结根的副本上把 5 个变体各编一份、跑 r2 小界，5 个都过了 15 项启动自检；第二段补跑得到 `kill_bound` 29 行（`.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:292`：「`kill_bound` 29 行」）。也就是说表里那 29 行的变体启动都没倒；表外的变体没有证据。
- 判 K4-1：**不能由读代码判死**。有一处读实现结局的启动项没挪（`check_the_fixed_point_control`），读不出它在哪类变体下会红；已有的经验证据覆盖变异杀伤界那张表，不覆盖 `crates/mutations.tsv` 里任意一条改 `replay_journal` 的行。推翻「还有路径」的现象：在 `replay_journal` 上做「恒报错」「恒 panic」两类变体各编一份、跑 r1 小界，两份都过 SELF_CHECKS；这要编 E165，是 checker 档，我没跑，写进 7 节「要主 agent 跑的」。

### 4.2　多盘一族的「在」与「直接建表按盘号升序」：装置的哪一栏代表实现

- 「在」：装置侧写成「多盘域任一份在即在」（`e165_replay_bounded_enumeration.rs:1366`：「多盘域任一份在即在」）；条款 D23 已定项 14 `.claude/kb/decisions/23-journal的角色与格式.md:364`：「任一份自证校验和过即在」；实现 `crates/singlefs-core/src/recovery.rs:3017`：「全环扫描：每盘每个记录槽都读」那一段注释里同一句（「两份镜像任一份自证过即算在」，`scan_journal` 的 fsid 不符另算）。三处一致，对上。
- 建表次序：装置默认是 `RecordTableSource::Direct`（`:265`、`:273`、`:282` 三个构造都写 `table_source: RecordTableSource::Direct,`）：装置自己按盘号升序逐槽收、先到先得，**不调 `scan_journal`**；`Scanned` / `Both` 才调（`:3308`、`:3314`、`:3329` 的三臂）。`Both` 时汇总行取直接建表那一遍，扫盘那一遍进 `name=table_comparison` 行。
- 实现的次序是读者给的：`scan_journal` 里 `for device in reader.device_identities()`（`crates/singlefs-core/src/recovery.rs:3032` 起）；文件后端的池是 `crates/singlefs-core/src/recovery.rs:110`：「impl<Device: BlockDevice> PoolReader for [(DeviceIdentity, Device)] {」，交回的是切片原次序；挂载保存的是按交进来的次序（`crates/singlefs-core/src/mount.rs:830`：「device_identities_in_table_order: devices」）；内存池是 `BTreeMap` 的键序（`crates/singlefs-harness/src/memory_pool.rs:386`：「self.devices.keys().copied().collect()」）。`crates/` 里我没找到把盘表排成盘号升序的地方（`grep -n 'sort' crates/singlefs-core/src/mount.rs` 里与设备相关的没有命中）。
- 所以：真池里 `scan_journal` 的先到先得是**调用方交盘表的次序**。装置的两栏：「盘号升序」那一栏只在调用方恰好按盘号交盘时与实现一致；**代表实现的是「照 `device_identities()` 次序」那一栏**（`name=table_comparison` 里的 `*_in_reported_order` 字段，E165 页 Q4 一行已这样分栏）：`.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:295`：「**随盘次序翻面**（SF11、SQ16）」，反盘序时 R 域 `tables_differ` 3 482、「盘号升序」栏不同、「照 `device_identities()` 次序」栏 0 / 0。
- R 域为什么是 3 482：R 域在 1 号盘上放一条与 0 号盘同键、内容不同的残留（R 域定义在 r1 登记 `research/prompts/e165-preregistration.md:434`）。这正是 D13 M2 语义层前提排除的那一类：`.claude/kb/decisions/13-验证路线.md:32`：「同一 (实例代号, 计数器) 在全部时间线上只对应一份内容」。所以「盘号升序栏与实现不同」的那 3 482 态在条款之外，不是实现违约；它说明的是：同键两份内容不同时，实现的表取决于盘次序——这一点条款没有规定（核里 `first_copy_of_each_record_key` 只规定「先到先得」，「先」是读者次序）。
- 判 K4-2：「在」**对上**；「直接建表」默认栏**不代表实现**，代表实现的是 `scan` / `reported_order` 栏，要看 `--table both`。推翻它的现象：`crates/` 里有一处把盘表规范成盘号升序再交给 `scan_journal`（那样默认栏才等于实现）。

### 4.3　E170 导出装置：「补丁不算写」与记录核对器把补丁当后写看，会不会让 Q3-f 的数误导

代码事实：

- 执行器那一侧：`crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs:1939`：「补丁不算写（A(c) 按底态算，登记第一节读法 3）」；实现那一侧：`crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs:1940`：「接在写表末尾当持久的写，种类记成系统配置槽写：`publishes_of_one_recording` 不把它算进任何一次发布」，代码在 `observe_bad_row`（`:2935`：「writes.extend(row.patches.iter().map(|patch| RetainedWrite {」）。
- 记录核对器认「发布」只认 `RootRecordFua` / `JournalRecord` / `UnitWrite`：`crates/singlefs-harness/src/memory_pool.rs:1016`：「StepKind::ZeroFill | StepKind::SystemConfigurationSlot | StepKind::Barrier => {}」。第一条判据遍历的就是这些发布：`crates/singlefs-checker-tier/src/crash.rs:141`：「for publish in publishes_in(writes, persisted, continuity) {」，`root_without_record` 只在那里置位（`crates/singlefs-checker-tier/src/crash.rs:146`：「check.root_without_record = true;」）。**所以补丁（无论是伪造的根还是伪造的记录）永远不进任何一次发布，第一条判据对补丁结构上看不见。**
- 第二条判据（单元副本缺席）的「更晚的写解释」认持久集合里的所有后写，补丁在其中（kb 自己写了这一条，`.claude/kb/experiments/170-O3规约执行器第一版.md:292`：「坏状态的补丁在实现那一侧当一次持久的系统配置槽写」）；但落在单元扇区上的补丁只有 P9 一类，这一类在 Q3-f 的「两判都放行」28 行里一行没有（按 `BadShape::NamedUnitCopyCorrupted` 是唯一把补丁打在单元副本上的臂读出来的，再数了 28 行的行名：只有 P2、P3、P4、P5、P8 与盲测里的 P2、P3、P5）。

产物上的数（`research/results/e170-o3-spec-executor-2026-10-02-s3-bad-states-export.out`，我现数）：

```
bad_row lines: 76
both_pass true: 36
P+B both_pass true: 28
P+B rows: 66
root_without_record true: 0
claimed_state_missing_unit true: 20
```

- `.claude/kb/experiments/170-O3规约执行器第一版.md:195`：「- Q3-f（附带，三 oracle 合谋测试的数）」写的 28 行 / 66 行与我数的 P + B 的 28 / 66 对上（N 族 10 行另计，其中 8 行两判放行）。
- 第一条判据在 76 行里**一行都没有置位**（`root_without_record` 真 0 行）。P2（伪造根）本来要靠它判；P2-F 两条流的行是 `impl_consulted_root=1,6`（实现的恢复**采纳**了伪造的根）、`impl_root_without_record=false`、`impl_both_pass=true`。这几行「两判都放行」一部分是实现真的放行，一部分是记录核对器因为补丁的种类标签而结构上判不了——两者在这个数里分不开。
- 「缺单元」那 20 行红不是单靠补丁：同一个底态上 P2-Mk 红、P5-Mk 不红（我按 `base_digest` 分组数：17 组里 6 组组内 `claimed_state_missing_unit` 不一致），说明补丁经「恢复落到哪一版」影响了第二条判据的输入；这是判据在起作用，不是后写解释把它盖掉。
- 判 K4-3：**会误导，方向是高估实现一侧的放行**——28 / 66 是「放行行数的上界」，其中靠第一条判据本该判红、却因补丁不在任何发布里而放行的行没法剔出来；它支撑 D13 已定项 6 的合谋测试那一句（`.claude/kb/decisions/13-验证路线.md:121`：「**独立性要被验证，不许自称**」）里的「不是全部」（F3 没触发）没问题，支撑「实现这一侧判别力有多弱」不行。E170 页自己说了不判这一侧（`.claude/kb/experiments/170-O3规约执行器第一版.md:292`：「实现那一侧的判别力只报了数（Q3-f）」）。推翻我「种类标签让第一条判据看不见补丁」这一读法的现象：把补丁的种类改标成 `RootRecordFua` / `JournalRecord` 再导一次，P2 行的 `root_without_record` 仍是 0 行为真（那样原因就不在标签上）；变真则读法坐实。我没改装置、没重导（要编译，checker 档）。

### 4.4　C5 另几处（顺带，没被问到）

- 导出装置只导流、造补丁、记实现判定；执行器（Python）读的文件是 `bad-states.tsv`、`bad-patches.bin`，期望与实现判定分开放、「只给比对器读」（`crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs:1942`：「`bad-expected.tsv`（期望，只给比对器读）」）。D13 已定项 6 的 O3 独立性（`.claude/kb/decisions/13-验证路线.md:119`：「| O3 | 独立规约执行器 |」一行）落在执行器那一侧；这份导出装置在 `crates/` 里、补丁用 core 的写者编码（`crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs:1939` 同一行注释：「编码与重封都调 core 的写者」），所以补丁的字节出自实现的编码器，执行器按自己的读法判它——读的独立，字节不独立。坏状态本来就要「合法编码的坏内容」，这不算缺陷，但意味着 core 写者自己的缺陷这一路看不到。

## 5　C7　`verus-mutations.tsv` 整表与 `verus-pins.tsv` 的 56

- 条款：`.claude/rules/verification.md:98`：「| 形式化验证：恢复重放取前缀的核经 Verus 验过，E165 全量的红例都钉进快速档、在被判那棵树的实现上逐行重放与原实现相同 | 门禁 checker-tier-formal-verification：verus-toolchain-pinned、verus-proof、verus-vacuity、verus-must-red-worlds 四格用 Verus 验 `crates/singlefs-replay-prefix`（登记表 `.claude/gate.d/verus-pins.tsv`），pinned-red-examples-complete、pinned-red-examples-replay 两格判红例钉表 `.claude/gate.d/e165-pinned-red-examples.tsv` |」
- 整表 153 行：`grep -vc '^#' crates/singlefs-replay-prefix/verus-mutations.tsv` 在冻结副本上输出 153；我数的锚点命中：153 行全 `anchor=1`（第 3 节那段输出 `rows 153 bad 0`）。主 agent 的 `rows.out`：`BASE rc=0 56 verified, 0 errors`，153 行 `match=True`（`grep -c 'match=True'` 输出 153，不匹配的只有 `BASE` 行），且 `cmp` 它用的表与冻结副本的表字节相同。这是主 agent 的观测，我没跑 Verus。
- 登记数：`.claude/gate.d/verus-pins.tsv` 的 `crate` 行第三段是 56（`grep -nP '^crate\t'` 输出 `27:crate	crates/singlefs-replay-prefix	56`）；`rows.out` 的 `BASE` 行 56 verified。数对得上。
- 一处条款落后：上面那句条款还写「恢复重放取前缀的核」，被验的 crate 今天还装着择根核、扫环核与合判推论（2.5 节）；条款没有错，是范围写窄了，随 kb 写回一起补。
- 不归这一格但同表要知道：第 99 行的 M 在两批里反向改过（2.4 节）。
- 判 C7：**对上**（数与表在我能读到的两处互证；Verus 本身没复核）。推翻它的现象：门禁 verus-proof 格在提交时报的 verified 数不是 56，或 verus-must-red-worlds 有一行 M 对不上。

## 6　补：K1 里 `choose_root` 的 Vec 内存

`Vec<Option<RootRecord>>` 的长度是三个区域的槽数：`crates/singlefs-format/src/lib.rs:245`：「pub const ROOT_RING_REGIONS: u64 = 3;」，每区 4..16 个槽（`crates/singlefs-format/src/lib.rs:259`：「pub const ROOT_RING_SLOTS_PER_REGION_MAXIMUM: u64 = 16;」）⇒ 最多 48 项；`RootRecord` 是 `Copy`（`crates/singlefs-core/src/root_record.rs:51`：「#[derive(Clone, Copy, Debug, PartialEq, Eq)]」）。量级是几十项乘一条根（几百字节），**不构成问题**，与 1.2 节的扫环 Vec 不是一个量级。

## 7　不一致一览（两边整行见各节）

| 序 | 一边 | 另一边 | 节 |
|---|---|---|---|
| 1 | 背景「journal 环第一版 1024 槽」 | `crates/singlefs-format/src/lib.rs:216`：1024 是起点槽号；`:220` 默认环 805 306 368 字节 ÷ `:198` 的 4096 = 196 608 槽 | 1.2 |
| 2 | 背景「I-8.12 还没登进 invariants.md」 | `.claude/kb/invariants.md:101`：I-8.12 一行在，状态「未实现」 | 2.1 |
| 3 | `.claude/kb/decisions/13-验证路线.md:33` M3 一行：前提只有「所选根之上的记录不超在飞上限」 | `crates/singlefs-replay-prefix/src/lib.rs:859`：不动点 spec 另带 I-8.12 一条前件 | 2.1 |
| 4 | `.claude/kb/decisions/13-验证路线.md:31`：择根与扫环不在 Verus 里 | `crates/singlefs-replay-prefix/src/lib.rs:3977`、`:4022`：两个核在 Verus 里 | 2.5 |
| 5 | 背景「`crates/mutations.tsv` 第 173、174、177 行改前就 0 命中」 | 冻结副本与主工作区今天 `hits.py` 输出都是 1 | 3.4 |
| 6 | E165 默认的「直接建表」栏（盘号升序） | 实现的次序是读者给的（`crates/singlefs-core/src/recovery.rs:3032`）；`crates/` 里没有规范化盘次序 | 4.2 |

## 8　判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1-1 `choose_root` 行为 | 对上 | 读槽次序、次数、跳法、比较键、严格大于都与改前同义；只多一个核对错才响的 `.expect` |
| K1-2 `scan_journal` 行为 | 对上 | 扫环循环没动，先查后插与 `or_insert` 同义 |
| K1-2m 扫环 Vec 内存 | 取舍，非条款问题 | 峰值约为改前的 D 倍（双盘 2 倍），每盘 196 608 槽，不是背景写的 1024；推的，没量 |
| K1-3 键相同取前面 | 核对上；胶水口子 | 写死在核（严格大于、spec 第三句、单测、两条变异）；胶水摊入次序与下标没有行钉 |
| K1-4 胶水拿所选根调重放 | 无专用变异行 | 只有层 0 / 崩溃放量的 `recover`（checker 档，没复核）与 harness 挂载用例（没验）罩 |
| K2-1 M3 不动点前件 | 对不上一处 | spec 多一条 I-8.12，条款字面没有；其余五项对上 |
| K2-2 C2 可执行代码不改、39 行 | 对上 | `lib.rs` 一侧 0 行删除；48 加 − 9 改 = 39 |
| K2-3 合判推论空真 | 没发现空真；缺一行 | 前提取自两个有见证的核；去 J1 前提没有对应必红行；内容只是序的传递 |
| K2-4 必红表 99、125 | 125 站得住；99 理由是推测 | 125 因同序共用，读代码可推；99 的数两批里反向改过，门禁按条数相等判 |
| K2-5 C650 | Verus 一半对上；kb 落后 | E165 一半不在这一批；M1 一行写着择根扫环不在 Verus 里 |
| K3 变异行锚点与用例 | 静态全过 | C1、C3、C6 点名的行锚点命中 1、用例都在；红不红只有实现员自报 |
| K3 R3-1 | 只罩四之一 | 卸载记号有行；instance_table、fsid、allocation_record_tree_root 三处拷贝无行 |
| K3 S03 / S07 | 算补敏感取样点 | 第三类；依赖缓存不收全零这一实现细节 |
| K3 第 173、174、177 行 | 背景对不上 | 今天各命中 1 次 |
| K4-1 启动自检读实现 | 不能由读代码判死 | r1 的六项与多盘一项仍走到 `replay_journal`，`check_the_fixed_point_control` 取真结局；表内 29 行变体无倒、表外无证据 |
| K4-2 多盘「在」与建表次序 | 「在」对上；默认栏不代表实现 | 代表实现的是照 `device_identities()` 次序那一栏；R 域在条款前提之外 |
| K4-3 E170 补丁与 Q3-f | 会误导，高估放行 | 补丁标成系统配置槽写，第一条判据 76 行 0 次置位；28 / 66 是上界；F3「不是全部」不受影响 |
| C5 另 | 读的独立、字节不独立 | 补丁由 core 的写者编码；不是缺陷，是一个看不到的盲面 |
| C7 | 对上 | 153 行、56 在我能读到的两处互证；Verus 没复核 |

## 9　没做什么

- 不判别：没有采纳、没有出判决；表里的「对上 / 对不上」是这条腿的正推结论，主 agent 照三方规则逐条现查。
- 没跑：编译、任何测试（harness 档与 checker 档）、Verus、`prove-red.sh`、`crates-mutation-rows.py static-check`（它是门禁格）、门禁、E165 / E170 装置、E170 导出重导。凡标「自报」或「主 agent 观测」的数都不是我复核的。
- 复核不了：`prove-red` 逐行的「抓到」（要编译变异副本）；Verus 的 56 与 153 行 M（要 Verus）；E165 / E170 装置变异行（第 1634–1659、1668–1673、1813–1818 行）红不红（checker 档）；第 4.1 节「恒报错变体下 SELF_CHECKS 倒不倒」（要编 E165）；4.3 节改标签重导（要编 E170 导出）。
- 没读：这一轮攻方腿的提示与产出（`research/prompts/m3-formal-code-r1-opus-output.md`、`research/prompts/m3-formal-code-r1-opus-model/`），也没读 m3-formal-r6 一轮的任何腿产出。读过的 m3-formal-r2、r3 判决与登记属于被引用的历史。
- 没逐行读：E165 diff 5 600 行与 E170 新文件 3 742 行。读的是 K4 点名的那几处与它们的调用链；其余只做了静态检查（锚点、用例存在）。
- 引了几处 `/tmp` 路径：冻结副本与三份实现员报告是派发给的材料，不是新产物；我的脚本已放进 `research/prompts/m3-formal-code-r1-sonnet-model/`。门禁 doc-experiments 的 evidence-in-repo 格若因这几处 `/tmp` 引用判红，由主 agent 定怎么处理。
- 「P0–P5」没有定义可对，见 2.1 节。

## 10　要主 agent 跑的命令（不是我没做完的活，是我能提出但按规则不跑的复核）

- 变体启动探针（4.1 节）：在冻结副本的拷贝上，把 `replay_journal` 改成「恒报错」与「恒 panic」各一份，`cargo build -p singlefs-checker-tier --bin e165_replay_bounded_enumeration` 后跑 r1 小界（`--profile r1` 取最小一档），看 `name=self_check` 行是否出齐。需要编译，归 checker 档，由主 agent 在提交时或用户要求时起。
- 胶水口子的两行变异（1.3、1.4 节）：`crates/mutations.tsv` 补「`every_root_ring_slot` 摊入次序倒过来」「交给 `replay_journal` 的根换成别的」各一行，`prove-red.sh` 看现有 harness 用例红不红。
