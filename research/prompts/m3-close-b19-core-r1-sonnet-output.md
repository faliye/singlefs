# m3-close-b19-core-r1 正推腿（Sonnet）报告：R1、R2、R4

日期：2026-10-29。立场：从条款与 `crates/` 今天的代码正推，不读攻方腿报告。被判对象的开工快照 `research/prompts/m3-close-b19-core-r1-snapshot/crates-sha256.txt` 里 `mount.rs`、`root_record.rs`、`walk.rs` 三份的哈希与此刻主工作区一致（`sha256sum` 现算过，逐字节同）；`allocator.rs`、`fault_injection.rs` 两份没核，下面引 `allocator.rs` 的行号是此刻主工作区的。

## 复跑与哈希（模型在草稿目录，不在 `research/prompts/`）

模型目录 `/home/fy5090/.local/share/singlefs/work/m3-close-b19-core-r1-sonnet/model/`，只写了 R1 一个小模型（纯 python，不碰 `crates/`，不编译）。

复跑命令（新 shell，经内存包装，`--work` 给一个不存在的目录）：

```
bash scripts/hosts/run-with-memory-cap.sh 4G bash /home/fy5090/.local/share/singlefs/work/m3-close-b19-core-r1-sonnet/model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs/research/prompts/m3-close-b19-core-r1-snapshot --work <新目录> --model /home/fy5090/.local/share/singlefs/work/m3-close-b19-core-r1-sonnet/model
```

交回前我在新 shell 里整份跑过一次，退出码 0（退非 0 的件数 0），用时约一分半，末行 `name=r1_genesis_pin_model_rerun_matches_out ...`。

| 文件 | sha256 |
|---|---|
| `model/genesis_pin_model.py` | `0f3c307a9c4969c0106471d6c1517e6f695c21bba3c0a076e9f138af9238529b` |
| `model/out/model-output.txt` | `75e15679f696afae8310dd74977c8fa63dc12e82c0fec4e0b56596268b14ad96` |
| `model/tools/rerun.sh` | `9c81c5ab50b164613720e97f1ca1cce36b48b2750306104cd5a39dada093c32f` |

没有别的小模型：R2、R4 只读码推，不建模。能核的行号与引文都用 `grep -nF` 现查过；「转述」二字标出的是读码后的我的读法，不是原文。

## R1　C773：第 0 代根那一形的回收门槛钉

### 1.1 问题是什么（先把事实核实）

实现员的量数我复核不了（要编译 + 跑 harness 探针，我不编译）：他在 `~/.local/share/singlefs/work/m3-close-b19-p2-wave1/result.md`「第 0 代根那一形」一节报：被量的那次挂载 first_txg 是 14，读故障下写行那次是 (t, p) = (1, 1)，无读故障是 (0, 255)（转述；原文在该文件 71–79 行，仓外文件，cite-check 不管）。「复核不了」，原因：不编译 Rust、不跑 harness。我复核了他的推理链里能读码核的每一环：

- mkfs 把第 0 代根种进三个区域各自的槽 0：「(0..ROOT_RING_REGIONS).map(|region| RootRingSlot { region, slot: 0 })」（`crates/singlefs-core/src/make_filesystem.rs:826`），第 0 代根的 txg 是 0：「pub const GENESIS_CHECKPOINT_TXG: CheckpointTxg = CheckpointTxg(0);」（`crates/singlefs-core/src/make_filesystem.rs:56`）。
- 落点公式在 0 到 3S 之内是 (txg mod 3, txg div 3)：「region: checkpoint_txg.0 % ROOT_RING_REGIONS,」（`crates/singlefs-core/src/root_ring.rs:172`）；读根环核落点时第 0 代根另认：「if checkpoint_txg == crate::make_filesystem::GENESIS_CHECKPOINT_TXG {」（`crates/singlefs-core/src/root_ring.rs:188`）。所以区域 1、2 的槽 0 照公式要等 txg 1、2 才被盖；这两个 txg 的根槽写若被跳过（记录落了、根槽没写，D16 已定项 7 的崩溃形），第 0 代根就留在那里。
- 挂载时读坏的槽的钉值只照公式找：「for txg in 0..first_txg.0.min(ring_length_in_txgs) {」（`crates/singlefs-core/src/mount.rs:2947`），区域 1 槽 0 得 1、区域 2 槽 0 得 2，不是第 0 代根的 0。
- 条款要的是「可能住过的最旧 txg」：`.claude/kb/decisions/16-发布语义.md` 第 39 行「钉也落盘：根记录带这次发布用的回收门槛 t 与钉槽 p，写者取 t = max(这条根自己带的 F, 分配器记这条根时的回收门槛)」；钉在「最旧 txg」的原句在同一行：「钉在它可能住过的最旧 txg」。
- 欠账行自己记着这处疑点：「写者一侧另有一处没量的疑点（攻方推的）」（`.claude/kb/checks-owed.md:481`）。

写者的 p 来自钉值本身，所以钉值一改成 0，p 就丢了槽：「crate::root_record::PinnedRingSlot::of_the_lowest_pin(self.lowest_pinned_txg)」（`crates/singlefs-core/src/allocator.rs:1812`）；规格写死了 p 就是钉值：「Some(pinned_txg) => pin.0 == pinned_txg.0,」（`crates/singlefs-core/src/root_record.rs:157`）。checker 一侧把 p 当钉值反推槽：「region: pinned_txg % geometry.regions,」（`crates/singlefs-checker/src/walk.rs:9185`），GPU 走读内核同样：「let pinned_region = pinned % g.regions;」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:5141`）。钉值 0 只还原得出区域 0 槽 0。这一环与实现员说的一致。

### 1.2 一个关键观察：钉值是槽的函数

写者在 `record_root_written_by_this_process` 里取的是一个「槽 → 钉值」的表再取最小：「.ring_slots_pinned_since_the_mount_at_the_oldest_txg_they_can_have_held()」（`crates/singlefs-core/src/allocator.rs:2763`）接着「.into_values()」（`crates/singlefs-core/src/allocator.rs:2764`）把槽丢了、只留值。表里每个槽的钉值 = 公式里落到这一槽的最小 txg = `3·槽 + 区域`（转述：`crates/singlefs-core/src/mount.rs:2924` 起那个函数的算法，没有读故障外的输入），**只由槽决定**。所以：

- 把「钉值」改成「槽 0 的钉值认成 0」之后，钉值仍然是槽的函数；p 完全可以继续存 `3·槽 + 区域`（对 0..3S−1 是槽的双射编码），不必改成别的编码。
- 候选甲（p 改存槽号、与钉值脱钩）与候选乙在信息量上等价，差别只在字节编码；甲多出格式、Verus 规格、GPU 内核、条文的改动，没有换来任何乙做不到的判别力。

### 1.3 候选比较

| | 定义 | 对的盘面 | 错的盘面 |
|---|---|---|---|
| 甲 | p 改存槽号（编码另定），钉值由槽另算 | 同乙 | 同乙；多动 `crates/singlefs-core/src/root_record.rs:151` 的 Verus 规格与 `verus-must-red-worlds.tsv` 里押在 `of_the_lowest_pin` 上的几行（形式化验证档要重跑）、折叠编码要写进 `.claude/kb/layout` 与 D16 |
| 乙 | 写者钉值认槽 0 为 0，p 仍存 `3·槽 + 区域`；checker 与 GPU 内核把槽号为 0 的钉值认成 0 | 全部（见 1.4 的模型） | 钉在槽 0 而实际那一槽早被盖过：钉得过低，回收门槛比必要的低，推迟回收到这个进程写掉那一槽为止（至多 3S 次发布），不丢数据 |
| 丙 | 认成已知结局，写进条文 | 无读故障的一切盘面 | 第 0 代根独留在区域 1 或 2 的槽 0、那一槽挂载时读坏或读回全零、first_txg > 3S：池级 checker 对一段合法历史判 I-7.18 红（实现员推的，没跑 checker）；另外这一形下门槛越过了第 0 代根，见 1.4 |

### 1.4 小模型：谁在哪些盘面上错

模型（`model/genesis_pin_model.py`）：槽 (r, i) 里可能住的内容 = 公式落到它的、小于 first_txg 的 txg，外加槽 i = 0 时的第 0 代根（txg 0；跳过的根槽写会留下更旧的内容）；好槽的真实内容已知，坏槽的内容取可能值里任一个；写者的门槛底 = min(好槽内容, 坏槽的钉值)；违例 = 门槛底高于全部槽内容的最小值（回收越过了某槽里住着的根）。S 取 1 到 4，first_txg 取 1 到 6S+2，坏槽 1 到 3 个，所有内容组合穷举。原样输出：

```
current total 15958404 violations 829794 violations_with_first_txg_le_3S 0 slot_indexes_holding_genesis_in_violations [0]
   e.g. S,first,bad,good_contents,bad_contents,pins = (1, 4, ((1, 0),), (3, 2), (0,), [1])
   e.g. S,first,bad,good_contents,bad_contents,pins = (1, 4, ((2, 0),), (3, 1), (0,), [2])
   e.g. S,first,bad,good_contents,bad_contents,pins = (1, 4, ((1, 0), (2, 0)), (3,), (0, 0), [1, 2])
genesis_zero total 15958404 violations 0 violations_with_first_txg_le_3S 0 slot_indexes_holding_genesis_in_violations []
```

读法：`current` 是今天的取法，`genesis_zero` 是乙。今天的取法只在 first_txg > 3S 时违例（≤ 3S 的 0 个，因为第 0 代根还在区域 0 槽 0，那一槽读得出就是门槛 0、读坏就钉在 0），违例的坏槽全是槽号为 0 的槽；乙在这 1595 万个盘面上一个违例都没有。丙等于今天的取法，所以丙的错盘面就是 `current` 那一行。模型的局限：它给「内容」的取法比真实历史宽（任意组合都放行），所以 82.9 万个违例里绝大多数在真实历史里造不出；它证的是乙的充分性（没有任何盘面乙会错），不是今天的取法会在多少真实盘面上错。

### 1.5 推荐：乙（写者与 checker 各改，p 仍存 `3·槽 + 区域`）

**推荐乙**。依据：1.1 的链、1.2「钉值是槽的函数」、1.4 的模型（乙 0 违例、甲与乙同样 0 违例但多动格式）。丙不推荐：它让池级 checker 对一段合法历史判红，而且 1.4 的 `current` 行里违例就是回收门槛越过了第 0 代根；这一形里门槛 1 或 2 在实现员的盘面上无害（他报：释放代 ≤ 1 的槽只能是 txg 1 释放的，而那是零单元暖机，转述，没跑），但这是他那个盘面的偶然，不是条款保证的——换一个 txg 1 或 2 带释放的历史（实例 1 取号后一发布都没有就崩，实例 2 的写行排在 txg 1，要不要有「释放代 1 的单元」我没造出来，推的），门槛越过第 0 代根引用的 mkfs 单元，管理员回退到第 0 代根就读到被复用的内容。这一条是推的，没造盘面；不推翻乙，只是丙不能靠它说「无害」。

乙的细则（容易写错的几处）：

1. 写者只对**已被交回的槽**改钉值：`txgs_ring_slots_bad_at_mount_can_hold`（「fn txgs_ring_slots_bad_at_mount_can_hold(」，`crates/singlefs-core/src/mount.rs:2924`）里 `oldest_txg_of_each_bad_slot` 那张表里属于 `genesis_root_ring_slots()` 的槽，可能住过的最旧 txg 取 `GENESIS_CHECKPOINT_TXG`。不要为了「完整」把公式范围里没有 txg 的槽 0（first_txg ≤ 2 的年轻环）也交回：那里的 `.expect("这个槽在 [0, first_txg) 里住得下一个 txg…`（`crates/singlefs-core/src/mount.rs:2959`）会 panic。1.4 的模型证了不交回是安全的（first_txg ≤ 3S 时第 0 代根还在区域 0 槽 0，那一槽读得出门槛 0、读坏钉 0）。
2. 钉值与槽要一起往下传：`ReclaimFloorWhenTheRootWasRecorded` 的字段「pub lowest_pinned_txg: Option<CheckpointTxg>,」（`crates/singlefs-core/src/allocator.rs:1794`）改成带槽的 `Option<(RootRingSlot, CheckpointTxg)>`；`record_root_written_by_this_process` 取最小那一步（`crates/singlefs-core/src/allocator.rs:2763` 起的 `.into_values().min()`）改成按 (钉值, 槽) 全序取最小——**三个区域的槽 0 钉值全是 0，必须有确定的破平局**，否则 p 在重复挂载之间抖。
3. p 由槽编码，不由钉值编码：`PinnedRingSlot::of_the_lowest_pin`（「pub fn of_the_lowest_pin(lowest_pinned_txg: Option<CheckpointTxg>) -> (pin: Self)」，`crates/singlefs-core/src/root_record.rs:151`）改成收槽，输出 `3·槽 + 区域`。这是实现员量到的「候选修法那一行变异写出 p = 0」的根因：钉值 0 编成 p = 0，checker 当成钉着区域 0 槽 0。
4. checker 与 GPU 内核对称：`pinned_ring_slot_of_the_root`（「fn pinned_ring_slot_of_the_root(」，`crates/singlefs-checker/src/walk.rs:9170`）还原出的槽号为 0 时，交给 I-7.18 现算用的钉值取 0（`crates/singlefs-checker/src/walk.rs:9249` 的「PinnedRingSlotOfTheRoot::Pinned { pinned_txg, .. } => Some(pinned_txg),」那一处用它）；内核里「oldest = u64_of(pinned);」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:5125`）同样按 `pinned / g.regions == 0` 取 0。checker 不 import core：「第 0 代根种在每个区域的槽 0」在 checker 里写成自己的常量，并把理由写进 `crates/singlefs-checker/invariants/02-根环.md` I-7.18 一行（`:25`「取建账时还钉着的槽里钉值最小的那个」那一句要补「槽号为 0 的槽钉值取 0」）。
5. 条文：D16 已定项 1「根槽这一次读坏」一行（`.claude/kb/decisions/16-发布语义.md:39`）补一句：槽号为 0 的槽的钉值是第 0 代根的 txg 0（mkfs 把第 0 代根种在每个区域的槽 0，`crates/singlefs-core/src/make_filesystem.rs` 的 `genesis_root_ring_slots`）；p 是 `3·槽 + 区域`，不再等于钉值。三语文档（`.en.md`、`.ja.md`）与 `crates/singlefs-checker/invariants/05-空间记账.md`、layout 01-first-txn 里写「p = 钉值」的地方同批改。

### 1.6 最容易被打穿的地方

- **钉得过低**：槽号为 0 的槽被钉在 0，哪怕那一槽事实上早被盖过。代价是回收被拖到这个进程写掉那一槽（至多 3S 次发布）；今天的 1 或 2 同样要等这一步，所以拖延的长度不变，只是门槛从 1、2 掉到 0。不丢数据。要打穿它得说明「拖延」会让某个 ENOSPC 门槛红——释放代 ≤ 2 的槽极少，我没造。
- **平局只记一个槽**：I-7.19 只看 p 指着的那一槽。三个槽 0 同时读坏时 p 只记一个，另两个槽的「钉着的槽不住本实例写的根」不判。这与今天「p 只记最低钉值那个槽」同一种缺口（C863 同族），不是乙引入的；但今天每个钉值唯一，缺口只在多槽钉着时存在，乙让平局成了常态，所以破平局规则要写进条文。
- **p = 0 的歧义**：p = 0 现在只能是区域 0 槽 0。**最容易写错的是把「钉值 0」直接当 p 写出去**（上面细则 3）：这时 I-7.18 仍绿（现算 0 = t 0），只有 I-7.19 在「区域 0 槽 0 恰好住着同一实例的根」时红。实现员量的盘面里区域 0 槽 0 住的是更早实例的 txg 12，I-7.19 不红，变异会活。所以必须有下面 T3 那个 first_txg = 3S 的变体。
- **GPU 内核的展开大小**：内核里有预算钉（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3287` 的注释点名了那条钉着展开大小的测试）。加一个分支会动展开大小；这是 checker 档重型，我没跑。

### 1.7 会红的用例与变异

- T1（改实现员第一波补丁里的那条，补丁在 `~/.local/share/singlefs/work/m3-close-b19-p2-wave1/patch/`，主工作区还没有——我 `grep -rn genesis_root_left_in_region_one crates` 零命中）：`a_genesis_root_left_in_region_one_slot_zero_reading_bad_at_the_mount_is_pinned_at_txg_one_above_the_genesis_root`。今天它钉 (txg, t, p) = (14, 1, 1) 并期望 `.and_violated("I-7.18")`；乙之后改成设备错与全零两种都是 (14, 0, 1)，池级 checker 全绿，无读故障那一格仍 (14, 0, 255)。
- T2（新增）：同一场景，第 0 代根留在区域 2 槽 0（实例 1 崩在 txg 2 的根槽写之前）。期望 (t, p) = (0, 2)。
- T3（新增，抓 p 编码错）：first_txg = 3S 的变体（S = 4 时 first_txg = 12，写行那次本身盖掉区域 0 槽 0，区域 1 槽 0 仍钉着）。期望 (t, p) = (0, 1)；p 若写成钉值 0，I-7.19 因区域 0 槽 0 住着同实例的 txg 12 的根而红。
- walk.rs 库内单测（`crates/singlefs-checker/src/walk.rs:20527` 起那条）加两格：`pinned_ring_slot_of_the_root(1, …)` 还原 (区域 1, 槽 0)、现算钉值 0；`(2, …)` 同。
- 变异（加进 `crates/mutations.tsv`，每条单独一行）：
  - m1：`txgs_ring_slots_bad_at_mount_can_hold` 里槽 0 的最旧 txg 退回公式值 → T1、T2 红。
  - m2：walk.rs 里槽号为 0 的钉值退回公式值 → T1 红在 I-7.18（写者 0、checker 现算 1）。
  - m3：GPU 内核同一处退回 → `crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state` 一族红（checker 档，我没跑）。
  - m4：`of_the_lowest_pin` 退回按钉值编码（p = 0）→ T3 红在 I-7.19，T1 不红。
  - m5：破平局规则去掉（`min()` 只比钉值）→ 需要三个槽 0 里不止一个读坏的用例才红：T4（新增）区域 1、2 的槽 0 同时读坏，期望 p 恒取 (钉值, 槽) 最小的（区域 1），两次挂载 p 相同。

### 1.8 什么现象会推翻 R1 的推荐

- 找到一个盘面，其中槽号为 0 的槽被钉在 0 之后，写者 t 与 checker 现算的 t* 仍不等（模型没覆盖：模型不含 F、不含实例表滤掉的被抛弃根）。
- 找到「第 0 代根不种在每个区域槽 0」的池（mkfs 之外的写者，或将来改 `genesis_root_ring_slots`）：checker 那个常量就错了。
- 证明年轻环（first_txg ≤ 2）里有一种盘面，区域 0 槽 0 的第 0 代根也不读作有效根而又不钉（例如被判成「被抛弃」）：则细则 1 的「不交回」不安全。

## R2　C724：准入抬 F 上限里同名的 `NoValidRoot`

### 2.1 三处各是什么（事实）

- 背景说的「两处」都在 `rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount` 里：「.ok_or(RecoveryFailure::NoValidRoot)?;」（`crates/singlefs-core/src/mount.rs:3944`，`newest_per_device` 取不出最小值，也就是 `valid` 为空）与「.ok_or(RecoveryFailure::NoValidRoot)?;」（`crates/singlefs-core/src/mount.rs:3987`，`ceiling_from_newest_and_non_empty_roots` 交回 `None`）。
- **后一处在结构上到不了**（转述，读码推的）：`ceiling_from_newest_and_non_empty_roots` 只在「第 4 新的不同状态不存在、`oldest_valid_root` 也是 `None`」时交回 `None`（「let fourth_newest = distinct_states_newest_first」，`crates/singlefs-core/src/mount.rs:4041` 起那一句的 `.or(oldest_valid_root)?`），而 `oldest_valid_root` 是 `valid.iter().map(..).min()`，`None` 即 `valid` 为空；`valid` 为空时前一处（`crates/singlefs-core/src/mount.rs:3940` 起的「let newest_on_every_device = newest_per_device」）早已返回。所以两处里真正可能触发的只有前一处。
- `crates/singlefs-core/src/mount.rs:8346`「choose_root(cache, system_configuration).ok_or(RecoveryFailure::NoValidRoot)?;」是读阶段的 `replay_journal`，不属 C724：它是读阶段择不到根，只读挂载同样挂不上，harness 的单子明写「恢复报的错（读阶段择不到根那一拒在内，只读挂载同样挂不上）：不在单子上。」（`crates/singlefs-harness/src/mount_transparency.rs:2301`）。
- 公开函数文档把这个错当作它的契约之一：「一条有效根一条都没有 ⇒ `RecoveryFailure::NoValidRoot`」（`crates/singlefs-core/src/mount.rs:3850`）。同一份函数体有两种调用方：这个进程的准入（`rollback_floor_ceiling_of_this_process`，`crates/singlefs-core/src/mount.rs:4349`）和不带根环表的外部调用方（`rollback_floor_ceiling`，`crates/singlefs-core/src/mount.rs:3857`）。

### 2.2 「单子」是什么，这两处该不该上去

单子是 harness 里的 `RefusalReasonOnTheD23List`（`crates/singlefs-harness/src/mount_transparency.rs:2238`）。判一次拒可写合法要同时过四条：理由在单子上、判据在崩溃镜像上复算为真、盘上逐字节不变、只读读出在允许集合里（「理由在单子上（U6 第一条）之后，U6 另三条里没满足的：判据在崩溃镜像上复算为真、盘上逐字节不变、只读读出在允许集合里。」，`crates/singlefs-harness/src/mount_transparency.rs:3195`）。

这两处不能上单子，理由是**类别错**，不是「还没来得及加」：

1. 它们在写行与暖机**之后**才跑：「**可写挂载的准入不够时，写行那次发布照走切换预留，写完行之后推抬 F 的空发布、再判**」（`.claude/kb/decisions/16-发布语义.md:44`），挂载里为准入抬 F 报的错包进 `FloorRaiseFailedAfterTheMountsPublishes`（`crates/singlefs-core/src/mount.rs:340`），这个成员在单子映射里本来就是 `None`（「| MountError::FloorRaiseFailedAfterTheMountsPublishes(_) => None,」，`crates/singlefs-harness/src/mount_transparency.rs:2328`）。单子的第三条「盘上逐字节不变」对写行之后的拒必然不成立。
2. 单子上每一项都带一个能在崩溃镜像上独立复算的判据（`crates/singlefs-harness/src/mount_transparency.rs:3195` 起的函数按理由分派）。这两处的判据是「写行之后的环里没有 txg ≥ F 的有效根」，崩溃镜像看不到写行之后的环。
3. 不在单子上不等于放过：映射为 `None` 的拒，裁决是 `ReasonNotOnTheD23List`，「照崩溃注入第二截今天的写法判（单元区墙计数，别的报红）」（`crates/singlefs-harness/src/mount_transparency.rs:3023`）——也就是前提一旦破了，扫描会报红，不是静默。

所以候选甲（加进单子）会让单子的第三、四条判据对这两处永远不成立，等于造一个永远判不合法的成员；不推荐。

### 2.3 走得到吗

**产品路径（这个进程的准入）走不到**，论证（读码推的，三个前提，各有出处）：

- 前提 a：调用时 `valid` 里有这个进程自己写的最新根 `current.root`。它的槽在这个进程的根环表里是「知道住着根的槽」：`record_root_written_by_this_process`（`crates/singlefs-core/src/allocator.rs:2731`）把它记成 `ValidRoot`，D16 把「这个进程写过且 FUA 返回过」列进这一类（`.claude/kb/decisions/16-发布语义.md:39`）。这一类的槽这一遍读坏会重读一次，仍坏就走别的错：「if !ring_slots_known_to_hold_a_root.contains(&ring_slot) {」（`crates/singlefs-core/src/mount.rs:3805`）那一支之外的路径返回 `RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`，根本到不了 `valid` 为空。
- 前提 b：`current.root.checkpoint_txg ≥ current.root.rollback_floor`。过滤用的是「.filter(|root| root.checkpoint_txg >= current_floor)」（`crates/singlefs-core/src/mount.rs:3918`），传进来的 `current_floor` 就是 `current.root.rollback_floor`（`crates/singlefs-core/src/mount.rs:4446`、`:4824`）。一条根带的 F 来自盘上已有的根与系统配置，新实例第一次发布的 txg 取环里全部根记录与全部自证记录的 txg 的最大值加一：「新实例（普通挂载、切换）的第一次发布的 checkpoint_txg ≥ max(根环里全部根记录的 txg」（`.claude/kb/decisions/23-journal的角色与格式.md:373`）。所以 F（≤ 某条已存在根的 txg）不会越过新根的 txg。**这一条是推的**，我没有找到一句条款直接写「一条根带的 F 不高于它自己的 txg」。
- 前提 c：`current.root` 不被它自己的实例表判成被抛弃。过滤是「.filter(|root| !abandoned_by_table(root, table))」（`crates/singlefs-core/src/mount.rs:3919`），`table` 是 `current.root` 指着的那张表（`crates/singlefs-core/src/mount.rs:4440` 起 `instance_table_chain_of_root`）；表里的行是写给**上一个实例**的（「给上一个实例写行」，`crates/singlefs-core/src/mount.rs:2`），当前实例没有行，`root_is_abandoned_by_the_instance_table` 要求「表里有它那个实例的行」才判被抛弃。

三条都成立 ⇒ `valid` 非空 ⇒ 前一处过得去 ⇒ 后一处也过得去。

**走得到的盘面只有非产品调用方**：公开函数 `rollback_floor_ceiling`（`crates/singlefs-core/src/mount.rs:3857`）给外部的实验装置与用例，分配器没装根环表时「读坏的槽一律当没有根」（`crates/singlefs-core/src/mount.rs:3845`），环里全读坏就 `valid` 为空：这是它文档里写的契约，调用方有 `crates/singlefs-checker-tier/src/on_device_modes.rs:332`（「let ceiling = rollback_floor_ceiling(」）、`crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs:1988` 与 `crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs:1203`。我没有造出一份**合法盘面 + 产品调用方**走到它的盘面（造不出，不是没试：三个前提已经把它封死）。

### 2.4 推荐：乙（写明走不到），配三样

1. **条文**：D16 已定项 1「抬 F 的上限」一行（`.claude/kb/decisions/16-发布语义.md:38` 那一行）或 D23 已定项 14「在任何写之前拒」那一行旁边补一句：准入抬 F 的上限在这个进程里走不到「没有有效根」，前提是上面 a b c 三条；若前提破了，错出来是 `Recovery(NoValidRoot)`，它属于写行之后的错，不在单子上、被扫描判红。三条前提逐字写。
2. **删死支**：后一处（`crates/singlefs-core/src/mount.rs:3987`）改成不可能为 `None` 的写法——`ceiling_from_newest_and_non_empty_roots` 收 `oldest_valid_root: CheckpointTxg`（非 `Option`），由前一处在 `valid` 上取最小值时一并取出。这样 C724 说的「两处」变成一处，不引入 panic。
3. **保留前一处并保留名字**：它是公开函数的契约，改名要连带外部调用方；不新加 `MountError` 成员（`refusal_reason_on_the_d23_list` 与 `history.rs` 里按该枚举穷举匹配，加成员要连着改，代价高、收益只是错误名字好看）。

### 2.5 变异第 5240 行在推荐下怎么红

背景写的「第 5240 行」我现查是 **5238 行**（`crates/mutations.tsv:5238`，名字以「C724：准入抬 F 算上限读根环时，这个进程知道住着根的槽读坏也当没有根、不重读」起头；5240 行现在是 C755 的另一条）。它把 `crates/singlefs-core/src/mount.rs:3805` 那一句换成「if true {」，点名测试 `the_admission_ceiling_refuses_with_the_reread_refusal_when_every_root_ring_slot_reads_bad_only_on_its_own_read`（`crates/singlefs-harness/tests/admission_floor_ceiling_with_every_root_ring_slot_unreadable_on_its_own_read.rs:30`）。在推荐下它**照红、一字不改**，但含义变了：它钉的是前提 a（知道住着根的槽读坏必重读、不当没有根），变异一开，环全读坏 ⇒ `valid` 为空 ⇒ `Recovery(NoValidRoot)` 而不是重读那一拒。变异名里「报单子外的 Recovery(NoValidRoot)」要改成「前提 a 破了」。

另外两条前提没有变异押着，要补（每条一行 `mutations.tsv`，各点一条测试）：

- m6（前提 b）：`crates/singlefs-core/src/mount.rs:3918` 的 `>=` 改成 `>`。测试 T-R2b（新增，harness）：mkfs 同一个进程里、F = 0、环里只有三条 txg 0 的第 0 代根，调公开函数 `rollback_floor_ceiling`，期望 `Ok(0)`；变异后 `valid` 为空，得 `Recovery(NoValidRoot)`，红。这是 txg 与 F 相等的边界，也是产品里 mkfs 同一个会话第一次发布的准入的形状。
- m7（前提 c）：`crates/singlefs-core/src/mount.rs:3919` 的过滤去掉 `!`（只留被抛弃的）。红的测试：任一条现有的准入抬 F 用例，如 `admission_raises_the_floor_before_refusing.rs` 里走到 `:1203` 的那条。
- m8（后一处删掉之后的结构）：删死支之后不需要变异；但要一条库内单测钉住 `ceiling_from_newest_and_non_empty_roots` 对非空输入恒交回值（T-R2c），防有人再把 `Option` 加回来。

### 2.6 最容易被打穿的地方

- 前提 b 是推的：找到一条根带的 F 高于它自己的 txg 的合法盘面，推荐就倒了。可疑处：F_生效 取「被抛弃时间线上的根所带 F」（D16 的「生效」一行）与系统配置里的 F；它们都 ≤ 当时已存在根的 txg，而新实例第一次发布的 txg 高于环里一切，但**根环转过一圈之后被盖掉的根**带的 F 若在系统配置里留着，也只会 ≤ 当时的 txg ≤ 现在的 txg，不越界；我没找到反例。
- 前提 a 靠 `record_root_written_by_this_process` 的调用时机：零单元发布是根落盘之后才记的（`crates/singlefs-core/src/allocator.rs:2731` 的文档）；失败时整份分配器换回，表退回，落盘途中失败后会话冻结，不会再走到准入。C745（R3）正是要攻这一环，本腿不判。
- 外部调用方把产品路径的契约借走：`crates/singlefs-checker-tier/src/on_device_modes.rs:332` 用公开函数而不是 `of_this_process` 版，它的 `allocator` 是真的，所以 `ring_slots_whose_reread_still_bad_refuses_the_ceiling_by(allocator)` 有值；换一个手拼空分配器的调用方，才会真的走到前一处。

### 2.7 什么现象会推翻 R2 的推荐

- 一份合法盘面 + 产品调用方，写行之后的准入返回 `Recovery(NoValidRoot)`（例如 `crash_enumeration_mounts_with_read_faults_stream` 一类扫描里出现 `ReasonNotOnTheD23List` 且原因是它）。
- 找到一条根带的 F 高于它自己的 txg 的合法盘面（前提 b）。
- 找到写行之后当前实例有行的表（前提 c），例如实例切换把当前实例的行提前写了。

## R4　C874：补写那一支的可观测与崩溃流

### 4.1 今天的事实

- 补写闸函数交回的是 `Option<RewriteGateNotMet>`，调用处把原因扔了：「Some(_gate_not_met) => return Err(device),」（`crates/singlefs-core/src/mount.rs:6023`）。闸前还有一条同样不带原因的拒：「if slots_reporting_a_read_error > 0 {」（`crates/singlefs-core/src/mount.rs:6009`，有一槽设备报读错的盘不过闸、不补写）。
- 原因的枚举是私有的：「enum RewriteGateNotMet {」（`crates/singlefs-core/src/mount.rs:5957`，五个成员：单元不全在本盘、环槽读错、有更新的根或记录、单元区有更晚实例的单元、所选根不在本盘的根环槽里），文档明写不对外：「只在核里分辨：调用方照旧拒成 [`SystemConfigurationUnreadableWhen::CheckingEachDeviceBeforeTheAcquisition`]，不另起错误成员」（`crates/singlefs-core/src/mount.rs:5954`）。外面看到的只有「CheckingEachDeviceBeforeTheAcquisition { device: DeviceIdentity },」（`crates/singlefs-core/src/mount.rs:374`）。
- **成功的一支同样看不出**：`MountOutput` 里有「pub shadow_ledger_branch: &'static str,」（`crates/singlefs-core/src/mount.rs:1037`）这种走了哪一支的字段，却没有哪块盘被补写了；`devices_rewritten_by_the_acquisition` 只在 `PublishesPlannedAfterAcquisition` 里流转（「devices_rewritten_by_the_acquisition: Vec<DeviceIdentity>,」，`crates/singlefs-core/src/mount.rs:7594`），挂载做成之后丢了。设计纪律是「4. **分支必须可观测**：运行时能看出走了哪一条。」（`.claude/rules/fs-design.md:127`）。所以缺口是两个：拒的原因、成的事实。
- 条款：补写五件闸与「补写失败照取号每块盘回卷一次」在 `.claude/kb/decisions/22-单元原子性怎么合成.md:185`（「某块盘两槽全废（一份本池自证过的系统配置都没有）时，取号那一步补写它的两槽」）。欠账行写明判官不复算五件闸、由 harness 档逐件判：「补写闸五件判官不复算，明写成不判」（`.claude/kb/checks-owed.md:540`）。

### 4.2 可观测的形态：候选与推荐

| 候选 | 定义 | 评价 |
|---|---|---|
| 甲 | 拒的原因进错误成员的载荷：`CheckingEachDeviceBeforeTheAcquisition { device, why_not_rewritten }`，`why_not_rewritten` 是新的公开封闭枚举（闸前读错一个成员 + 五件闸五个成员），`RewriteGateNotMet` 改成 `pub` 或被它包住 | 外层 `MountError::SystemConfigurationUnreadableAfterTheReadStage` 不变，单子映射不动（`crates/singlefs-harness/src/mount_transparency.rs:2295` 按外层成员分派）；`crates/singlefs-harness/src/history.rs:4710` 起的匹配已经写成 `{ .. }`，不用改；要改的是按 `==` 构造变体值的用例（十来处，列于下），而这正是要的：每条用例从此必须说是哪一件没过 |
| 乙 | 另起 `MountError` 成员 | 单子映射、`history.rs`、`model_comparison.rs` 按该枚举穷举匹配，都要改；还要决定它进不进单子；`crates/singlefs-core/src/mount.rs:5954` 的取舍就是为了躲这个。收益只是错误名字更直，载荷不如甲细。不推荐 |
| 丙 | 观测记录一行（日志、计数） | core 里没有日志设施（`crates/singlefs-core/src/*.rs` 里只有 `block_device.rs` 一处清理失败的 `eprintln!`），拒的路径只返回 `Err`、没有输出；要加一个汇入参数穿过 `mount_writable` 与全部调用方。不推荐用在拒的一支 |
| 丁（成功一支） | `MountOutput` 加 `devices_rewritten_by_the_acquisition: Vec<DeviceIdentity>`（与 `shadow_ledger_branch` 同类：这次挂载走了哪一支的事实） | 成功的一支只能走输出；没有别的形态 |

**推荐：甲加丁。** 改动落点：

1. `crates/singlefs-core/src/mount.rs`：`RewriteGateNotMet` 公开（`:5957`）；新枚举（闸前读错一个成员 + 五件）；`devices_without_the_selected_version` 的返回类型 `Result<…, DeviceIdentity>` 改成带原因的 `Err((DeviceIdentity, WhyNotRewritten))`（`:6009`、`:6023` 两处 `return Err(device)` 改带原因）；`:7284` 把原因装进 `CheckingEachDeviceBeforeTheAcquisition` 的载荷；`MountOutput` 加字段并在构造处（`:7110` 附近）从 `PublishesPlannedAfterAcquisition` 带出。
2. `crates/singlefs-harness/tests/common_images_for_the_checker_tier/acquisition_rewrites_a_device_that_lost_both_system_configuration_slots_only_through_the_gates.rs`：`assert_refused_before_the_acquisition_naming_device_one`（`:123`）加一个「期望的原因」参数；七条拒用例（空白盘、别的池的盘、克隆盘带更新的根与记录、环里有更新的记录、单元区有更晚实例的单元、环槽读错、所选根不在槽里）逐条传自己的那一件。过的那条用例（`:146`）断言 `MountOutput` 里恰好是那一块盘。其余按 `==` 构造变体值的用例同批改：`writable_mount_refusal_on_a_crash_state_without_read_faults.rs`、`a_writable_mount_with_system_configuration_slot_zero_unreadable.rs`、`mount_transparency_judges_the_system_configuration_witness_by_r2.rs`、`fsync_drop_and_devices_without_the_selected_version.rs`、`writable_mount_refusal_after_the_read_stage_counts_self_verified_slots_by_the_pool_checker.rs`，与 `crates/singlefs-harness/src/model_comparison.rs:2067`。
3. 逐件变异（欠账行要求「逐件变异判」）：每件闸一行，把该件的 `return Some(...)` 换成不拒，点名上面对应那条拒用例；再加一行把原因换成恒报同一件（`Some(_gate_not_met)` 改成固定成员），七条拒用例里至少六条红。这是甲的判别力来源：今天这些用例判不出是哪一件没过，所以逐件变异无处可红。

### 4.3 补写中途崩溃的枚举流：要罩住哪几个崩溃点

事实（读 `transaction.rs`，推的、没跑）：基镜像取第一个文件之后的池里把一块盘（记为 D）两个系统配置槽清零的那一份（`.claude/kb/checks-owed.md:540` 的话：「补写流的造法是第一个文件之后的池把一块盘两个系统配置槽清零当基镜像」），另一块盘 L 完好。重开可写挂载，取号写按「ACQUISITION_WRITE_ROUNDS」两轮发（「const ACQUISITION_WRITE_ROUNDS: [AcquisitionWriteRound; 2] = [」，`crates/singlefs-core/src/transaction.rs:1251`），每轮「for index in 0..pool.devices.len()」逐盘写一次，轮后一道池屏障（`crates/singlefs-core/src/transaction.rs:1262` 起 `write_the_witnessed_acquisition`）。每次写的世代号 = 这块盘两槽里自证过的最大世代号 + 1（两槽都读不出时从 1 起），槽 = 世代号 mod 2（`crates/singlefs-core/src/transaction.rs:609` 那一行的文档）。所以 D 第一轮：无自证槽 ⇒ 世代号 1 ⇒ 槽 1；第二轮：槽 1 若落了，世代号 2 ⇒ 槽 0；L 第一轮写较旧的那一槽，第二轮写取号之前最新的那一槽。

| 崩溃点 | 状态数 | 为什么必须罩 |
|---|---|---|
| 取号写之前那道屏障 | 1（无写） | 闸只读盘，崩在这里与基镜像逐字节同；证「读闸不留痕」 |
| 第一轮一段（D 槽 1、L 较旧槽，两次原地覆写，每次落了 / 没落 / 新旧都读不出） | 3² = 9，真子集 8 | D 的写撕了或没落 ⇒ D 仍两槽全废，重开要再过一次闸；D 落了 ⇒ D 有一槽自证过，重开不再过闸走落后支。L 的较旧槽撕了不能让 L 两槽全废（它最新那槽在） |
| 第一轮后屏障 | 0 | 屏障把两轮隔开；删掉它两轮并成一段，枚举会摆出「L 第一轮没落而第二轮撕了」，L 两槽全废 |
| 第二轮一段（D 槽 0、L 取号之前最新的那一槽） | 9，真子集 8 | D 此时槽 1 已持久（带新号），第二轮撕了槽 0 不让 D 变死盘；L 最新槽撕了，L 较旧槽已是新号 |
| 第二轮后屏障 | 0 | 同上，与写行那次发布的单元写隔开 |
| 写行、暖机、轮换 | 与已有的重开流同 | 这一段之后 D 两槽都是新号，枚举写表与已有流同形，不另罩；共用前缀 |

两段各 8 个真子集态，共 16 个（对照 `crates/singlefs-checker-tier/tests/crash_enumeration_crash_in_the_middle_of_the_acquisition_write_stream.rs:8` 的「每段的真子集 3² − 1 = 8 个」）。每个状态要判的结局：

- 观察者（已有）：相对基镜像，只记新出现的死盘（`crates/singlefs-checker-tier/src/crash_point_replay_provenance.rs:958` 的 `observe_the_devices_without_a_self_verified_system_configuration_slot`）；这条流里正确实现不该出现新死盘，D 本来就死不记。
- 重开可写挂载（已有的取号写流那一套）：挂得上，取到的号越过盘上读得出的每个实例代号；挂完之后池级 checker 的 `devices_with_fewer_than_two_self_verified_slots` 为空（D 被补成两槽）。
- 新计数（新增）：D 在这个状态上无自证槽的状态数；等于 D 的槽 1 写取「没落」或「撕」的状态数，用闭式预测，防止撕裂态被并进落了态。

落地：新测试文件 `crash_enumeration_acquisition_rewriting_a_dead_device_stream.rs`（名字待定，不用序数）、`PROVENANCE_OF_THE_FLOWS` 加一行 `enumeration_of_the_item(…, FlowProvenance::RecordedAfterTheMakeFilesystemReturned)`（`crates/singlefs-checker-tier/src/crash_point_replay_provenance.rs:144` 一带的写法）、`.claude/gate.d/crash-point-replay-items.tsv` 与 `stage-inputs.tsv` 各登一行；登记完整性由 `assert_every_state_was_seen_by_the_observers_attached_to_the_flow`（`crates/singlefs-checker-tier/src/crash_point_replay_provenance.rs:897`）判。

**不是崩溃点、要另用写故障用例罩的**：补写失败的回卷（第六件，`roll_back_acquisition`，`crates/singlefs-core/src/transaction.rs:713`）。崩溃点重放录的是不带故障的真实运行，盘报写错的分支不在录制流里。用 `InjectedFault::WriteFails` 打在 D 的第一轮与第二轮写上，各断言：挂载报 `MountError::Acquisition(…)`、回卷写按「每块盘只回卷一次」发、D 第一轮落了而第二轮失败时盘上槽 0 是旧实例号、槽 1 仍是新号（下一次取号跳过它）。这条我没读到现成用例（`grep` 过 `RolledBack` 与补写文件名的交集没有命中，但没逐个测试文件核），写成待查。

### 4.4 观察者「只判整写不判撕裂」那一行变异放在哪

观察者函数本身放不下它：`observe_the_devices_without_a_self_verified_system_configuration_slot`（`crates/singlefs-checker-tier/src/crash_point_replay_provenance.rs:958`）收到的只是逐盘自证槽数，没有「这个状态含不含撕裂镜像」；判段按槽数相同合并成段（`crates/singlefs-checker-tier/src/crash_amplification.rs:21711` 起的 `for run in &handed_back.slots_of_the_states.runs`），撕裂的信息在那之前就丢了。已有的几行（`crates/mutations.tsv:5165` 不判、`:5166` 不取基镜像、`:5167` 判槽读法恒为「有」，以及 `:5241`–`:5243`、`:5256`–`:5257` 判段一族）都押在读法与基镜像上，没有一行押在「撕裂态是否进枚举」。

**这一行要放在枚举侧**：`crates/singlefs-checker-tier/src/crash.rs:2325` 的 `torn[first_old_byte..].copy_from_slice(&old[first_old_byte..]);`（`torn_image_of_in_place_overwrite`，`:2309`）换成不拷旧字节——撕裂镜像退化成完整新写，等于撕裂态与落了态逐字节相同、「只判整写」。点名的测试必须是 4.3 那条新流的测试，不是 `crash.rs` 里 `:3058` 一带的库内单测（那几条会红，但那是低层，不算押住观察者）。新流测试里断言「D 在第一轮一段无自证槽的状态数 = 闭式预测」，变异后撕裂态变成落了态，这个数少一截，红。

### 4.5 最容易被打穿的地方

- **槽号与世代号的读法**（`crates/singlefs-core/src/transaction.rs:609`）是我读文档推的，没有跑：D 第一轮落在槽 1、第二轮落在槽 0。若实现里 D 的第一轮落在槽 0，上表「第二轮 D 槽 0」那行要换槽，状态数不变，计数的预测要跟着换。
- 「正确实现不留新死盘」假设取号写每轮之后都有屏障：这是 D23 已定项 16 的条款（`.claude/kb/decisions/23-journal的角色与格式.md`「取号那一步的屏障」），取号之前那一道在 `transaction.rs` 里读到（`CommitStep::Barrier`）。观察者的用处正是这个条款被删时报红。
- 观察者的 `slots_of_each_device_on_the_base_image` 对这条流是 D 为 0：若有人把基镜像换成「清零之前」的池，D 被当成新出现的死盘，整条流每个状态都红。基镜像要取清零之后的那一份，写进流的设点文件。

### 4.6 什么现象会推翻 R4 的推荐

- 甲的载荷改动让 harness 档以外的消费者（`crates/singlefs-checker-tier` 的 `on_device_modes.rs` 等）编不过且改法不止机械替换。
- 找到一种 D 的补写在第一轮之后崩溃而下一次可写挂载**不过闸**的合法盘面（则 4.3 的「再过一次闸」那一格要拆成单独的崩溃点）。
- 找到观察者与枚举之间还有一层丢撕裂信息的合并（4.4 的放置假设）：则变异要往前挪。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| R1 C773 第 0 代根那一形 | 推荐乙 | 钉值只由槽决定，槽号为 0 的槽钉值取 0，p 仍存 `3·槽 + 区域`；写者传槽与钉值、checker 与 GPU 内核各改一处；1595 万盘面的小模型上乙 0 违例，今天的取法 82.9 万违例且全在 first_txg > 3S、坏槽号为 0 |
| R1 甲 | 不推荐 | 信息量与乙相同，多动格式、Verus 规格、GPU 内核与条文，没有多出判别力 |
| R1 丙 | 不推荐 | 合法历史上 I-7.18 判红；这一形下门槛越过第 0 代根，无害只是实现员那个盘面的偶然 |
| R2 C724 两处 `NoValidRoot` | 推荐乙（写明走不到） | 后一处结构上到不了，前一处在产品路径上被三个前提封死；加进单子是类别错（写行之后才跑、盘已变、判据在崩溃镜像上复算不了）；删死支、补三条前提进条文、补两条变异 |
| R2 变异行 | 现查在 5238 行 | 照红不改，押前提 a；前提 b、c 另补 m6、m7 |
| R4 C874 可观测 | 推荐甲加丁 | 拒的原因进 `CheckingEachDeviceBeforeTheAcquisition` 载荷（公开封闭枚举，闸前读错 + 五件闸）；成功一支加 `MountOutput` 字段；日志式观测记录不可行（core 无设施） |
| R4 崩溃流 | 罩取号写两轮 | 取号写前屏障、两轮各一段（各 8 个真子集态）、两道轮后屏障，加基镜像上一块盘两槽清零；回卷（第六件）用写故障用例罩，不在崩溃流里 |
| R4 「只判整写不判撕裂」变异 | 放枚举侧 | `crates/singlefs-checker-tier/src/crash.rs:2325` 撕裂镜像退化成新写，点名新流测试；观察者函数与判段合并层放不下它 |

## 没做什么

- 没有编译 Rust、没有跑 harness、没有跑 checker 档与形式化验证；R1 里 P2 实现员的量数（t、p 的值）我复核不了，原因是要编译并跑探针；所有「会红」都是推的，没有一条跑出过红。
- 没有读攻方腿报告，没有判 R3（C745）；R2 的前提 a 靠 `record_root_written_by_this_process` 的调用时机，那一环 R3 在攻，这里不判。
- 没有核 `allocator.rs`、`fault_injection.rs` 的开工快照哈希（`crates-sha256.txt` 里有，只核了 `mount.rs`、`root_record.rs`、`walk.rs`，一致）。
- 没有造出 R2 的走得到的合法盘面（造不出，论证见 2.3）；R1 里「回收门槛越过第 0 代根会不会真让管理员回退读到被复用的单元」没有造盘面，推的。
- 没有核 R4 里「槽 = 世代号 mod 2」对补写盘的第一轮落在槽 1 这一读法的运行结果，只读了文档与代码；没有逐个核补写失败回卷现有用例。
- 没有改任何 `crates/`、kb、门禁表；`.claude/kb/checks-owed.md` C874 一行说观察者「还没有」，主工作区里观察者函数与七条变异行已在（`crates/singlefs-checker-tier/src/crash_point_replay_provenance.rs:958`、`crates/mutations.tsv:5165` 起），这一行的文字该由主 agent 或书记员更新，我没动。
