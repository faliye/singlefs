# 增补 3 第 3 件（崩溃注入）代码轮第一轮 —— 云端正推腿（Sonnet）报告

判的问题：K2（判读取哪条根）、K3（模型目录会不会漏版本）。分工表点名的是这两条；K1/K4/K5/K6 不判。

引用规则：kb 行号一律在 kb 文件里用 `grep -n` 现查；`crates/` 行号一律在工作区当前文件里用 `grep -n`/`Read` 现查（本报告写的都是这么取到的，写之前逐条核对过一次，见下方各节的核对命令）。

## K2：`observed_read_back_after_a_crash` 取 `effective_root` 对不对

### 代码现状

`crates/singlefs-harness/src/model_comparison.rs:295-312`：

```
pub fn observed_read_back_after_a_crash(report: &RecoveryReport) -> ObservedReadBack {
    let Some((instance, checkpoint_txg)) = report.effective_root else {
        return ObservedReadBack::Failed { what: format!("没择到根（{:?}）", report.outcome) };
    };
    let root = model_root_key(instance, checkpoint_txg);
    match &report.outcome {
        RecoveryOutcome::NoFile { .. } => ObservedReadBack::NoFile { root },
        RecoveryOutcome::FileRead { content, .. } => ObservedReadBack::FileRead { root, content: content.clone() },
        RecoveryOutcome::Failed { failure, .. } => ObservedReadBack::Failed { what: format!("{failure:?}（实际走的根 {root:?}）") },
    }
}
```

`root`（身份）与 `content`（`report.outcome` 里的内容）都来自同一次 `recover()` 调用，而 `RecoveryOutcome` 本身是 `walk_to_file(reader, &effective_root, ...)` 的产物（`crates/singlefs-core/src/recovery.rs:1378`）——身份与内容用的是同一条 `effective_root`，两者天然自洽，不会出现「根说的是 A、内容却是 B」。

`crash_injection.rs` 的调用点（`crates/singlefs-harness/src/crash_injection.rs:499,508,510-511`）：`recover(&image, JournalPolicy::Consult)` 直接读崩溃点截断后的镜像，`report` 传给 `observed_read_back_after_a_crash`，结果与 `committed_versions`（理想模型此刻根环里每条根的目录）比对。

### 与 D13（验证路线） 已定项 7 的关系：这里没有「两份镜像」的问题

已定项 7（`.claude/kb/decisions/13-验证路线.md:130`）：「⚠️ 『崩溃后镜像』是两份，不是一份：harness 的顺序是先跑实现自己的恢复、再跑记录核对器，而恢复本身会改盘（实例切换写行、重发在飞 checkpoint，D23（journal 的角色与格式） 已定项 14）⇒ 择根与前缀判定看崩溃态镜像……比对的对象是实现恢复后的镜像。」

`crates/singlefs-core/src/recovery.rs:1336` 的 `recover(reader: &dyn PoolReader, ...)` 入参是只读 trait `PoolReader`：函数体（1336-1395 行）内 `grep -n "&mut"` 只命中一处（1337 行 `let mut mapping_fallbacks = 0;` 那个局部计数器，1378 行传给 `walk_to_file` 的 `&mut mapping_fallbacks` 是同一个局部变量），没有任何一处调用 `reader` 的写方法、也没有对 `reader` 之外的设备做写入；它不做实例切换、不重发在飞 checkpoint。`crates/singlefs-harness/src/crash.rs`（层 0 的 oracle）与 `crates/singlefs-harness/src/crash_injection.rs`（这一件）都只调用这一个只读的 `recover()`，两处都没有调用 `mount_writable`/`mount_rollback`（`grep -n "mount_writable\|mount_rollback" crates/singlefs-harness/src/crash.rs crates/singlefs-harness/src/crash_injection.rs` 零命中）。

所以已定项 7 说的「实例切换写行、重发在飞 checkpoint」那个会改盘的「实现自己的恢复」，在这两处 harness 代码里都没有发生——判读用的镜像自始至终只有一份（崩溃态镜像），不存在「记录流看崩溃态镜像、比对对象却是另一份恢复后镜像」的分裂。**这一点上 K2 的判读与已定项 7 不冲突，但理由是「这里没有第二份镜像」，不是「两份镜像被正确区分」**——已定项 7 描述的那个「先跑实现自己的恢复、再跑记录核对器」的完整流程，我在 `crash.rs`/`crash_injection.rs` 里没找到对应实现，复核不了它在别处是否存在（未搜索 `crates/` 之外的用例文件）。

**推翻条件**：若日后 `crash_injection.rs` 或 `crash.rs` 改成先调 `mount_writable`/`mount_rollback`（会写行、重发 checkpoint）再判读，这一段结论作废，需要重新核对「记录流」与「比对对象」是否被分开处理。

（下一段续 K2 的两个构造。）

### 构造一：记录前缀施加到一半

这个状态不用现造——增补 3 第 3 件第一次跑就撞出来过，且已经用这个判读修好了。背景材料记（`research/prompts/_m2-supp3-item3-code-r1-background.md:20`，转述自 `.claude/kb/milestone/02-second-txn.md` 增补 3 那一节）：第一次跑抓到 4 条「冷启动读回」对不上，全是截在「journal 记录已写、根槽未写」那个窗口——实现照 D23（journal 的角色与格式） 已定项 14 的前缀口径施加了记录，内容前进到在飞那次发布的内容而根身份还停在旧根；判读那一步当时取的是 `RecoveryOutcome` 里的 `root`（即 `choose_root` 施加之前所选的根，`crates/singlefs-core/src/recovery.rs:1363` 的 `root_key`），所以「内容已前进、身份未前进」这个错配被放过。接上 `observed_read_back_after_a_crash`（取 `effective_root`，身份与内容用同一条根）之后四条全消，坐实的数：载荷容量 32634 下，种子 1 的操作 0 = 16599 字节（模型答的）、操作 1 = 22950 字节（实现读回的）。

也就是说：**「记录前缀施加到一半」这个状态下，取 `effective_root` 是让判读从「会把真违例判绿」变成「正确判红/判对」的那个改动**，不是新的风险点——它是已经被现实数据验证过的修复，而不是这一轮凭空构造的假设。

`crates/mutations.tsv` 里留了会红的一条钉住这个方向：`_m2-supp3-item3-code-r1-background.md:21` 记「把判读改回 `observed_read_back` ⇒ `crash_injection_fast_tier_recovers_only_into_versions_the_model_committed` 红」，我核对了这条变异表条目确实存在：

```
$ grep -n "crash_injection_fast_tier_recovers_only_into_versions_the_model_committed" crates/mutations.tsv
```

（下一段贴这条命令的原样输出与构造二。）

原样输出（已跑）：

```
$ grep -n "crash_injection_fast_tier_recovers_only_into_versions_the_model_committed" crates/mutations.tsv
180:增补 3 第 3 件：判崩溃点时取择根而不是施加记录前缀之后实际走的那条根（记录已写、根槽未写那一格内容与根身份配不上）	crates/singlefs-harness/src/crash_injection.rs	        let read_back = observed_read_back_after_a_crash(&report);	        let read_back = crate::model_comparison::observed_read_back(\&report.outcome);	-p singlefs-harness --test second_transaction_supplement_three_crash_injection -- crash_injection_fast_tier	crash_injection_fast_tier_recovers_only_into_versions_the_model_committed
```

条目存在，第 4 列（变异后的代码）正是把 `observed_read_back_after_a_crash` 换回 `observed_read_back(&report.outcome)`——与背景材料说法一致。这条变异是否真的会跑红，属于「能用命令核的事实」，但要跑 `cargo mutants` 或手工改代码复跑该测试，本报告未执行（复核不了：改动 `crates/` 源码不在这条腿的写范围内，且这类复跑更适合放进主 agent 的重验证阶段）。

### 构造二：所选根的实例有回退行时，前缀被 W 截断（D23（journal 的角色与格式） 已定项 14 第五条）

**这个状态在今天的代码与设计下构造不出来——不是我没找到构造法，是该分支本身是死代码，且这一点已经被 kb 自己记录在案。**

`rollback_high_water_of_root`（`crates/singlefs-core/src/recovery.rs:451-457`）：

```
pub fn rollback_high_water_of_root(reader: &dyn PoolReader, root: &RootRecord) -> Option<u64> {
    instance_table_of_root(reader, root)?
        .rows
        .iter()
        .find(|row| row.instance == root.instance && row.is_rollback)
        .map(|row| row.applied_transaction_high_water)
}
```

它在 `root` 自己指着的那张实例表（`instance_table_of_root` 读 `root.instance_table.locations`，`crates/singlefs-core/src/recovery.rs:422-429`）里找一行「`row.instance == root.instance`」。但 `mount.rs:952-976` 的 `instance_rows_to_write` 决定一张表里能有哪些行：

```
let first_row_instance = previous_row.instance.0.max(1);
(first_row_instance..instance_to_acquire.0)
    .map(|row_instance| { ... })
```

区间是 `[max(previous_row.instance, 1), instance_to_acquire)`——**严格小于** `instance_to_acquire`，也就是严格小于「这次要写这张表的那个实例自己的号」。换句话说：**任何一张表里都不会有一行描述它自己所属的那个实例**，只会有描述更早实例的行。`mount_rollback`（`crates/singlefs-core/src/mount.rs:1287-1292`）写回退行时同理：

```
let previous_row = PreviousInstanceRow {
    instance: target.instance,       // = R_old 的实例号，不是新实例自己的号
    selected_root_txg: target.checkpoint_txg,
    applied_transaction_high_water: 0,
    is_rollback: true,
};
```

这一行的 `instance` 是 **R_old（被回退掉的旧实例）**，它被写进**新实例自己的表**（`establish_instance` 里 `instance_to_acquire` 就是新实例号）。所以：若日后 `recover()` 选中的 `root` 恰好就是新实例（`root.instance == instance_to_acquire`），它自己表里的这一行 `instance == R_old ≠ root.instance`，`find` 找不到；若选中的 `root` 就是 R_old 本身，R_old 自己发布时那张表（回退发生之前就已经写死）里根本不会有「R_old 自己是回退行」这一行——这行是回退之后才被写进*新实例*的表的。**两种情况下 `find` 都不可能命中**，`rollback_high_water_of_root` 对任何一次调用都只能返回 `None`。

（下一段贴这一点在 kb 里的既有记录、以及这对 K2 结论意味着什么。）

**这不是我这一轮新发现的 bug，是 D23（journal 的角色与格式） 已定项 14 自己承认过的已知缺口**（`.claude/kb/decisions/23-journal的角色与格式.md:376`，整行）：

> ⚠️ **第五条与回退行的落点被用户打回重议（设计问题，不是代码问题）**：回退行写在新实例的实例表里，只有新实例的根指得到那张表；一次恢复要落到 R_old 上，(txg, 实例) 比 R_old 大的每一条根——新实例的全部根也在内——都得读不出，于是带回退行的那张表从任何一条可读根都够不着，第五条在任何可达的历史上都取不到真……定案之前实现取 P2、第五条照写但不起作用（预想，`crates/singlefs-core/src/mount.rs` 的 `mount_rollback` 与 `crates/singlefs-core/src/recovery.rs` 的 `rollback_high_water_of_root`）。

我核对的代码结构与这句「第五条在任何可达的历史上都取不到真」逐字吻合：kb 说的是设计层面的落点问题（回退行只能落在够不着 R_old 的那张表上），我从 `instance_rows_to_write` 的区间与 `mount_rollback` 的赋值反推出的是同一件事的代码形态。**这条已经欠在 C340（回退之后记录链从哪条之后接没有定义），不是这一轮的新账。**

进一步：即便 `rollback_high_water_of_root` 不是死代码，已定项 14 描述的它要挡住的真正危险场景——「比 R_old 新的根全坏时落回 R_old、同实例连号的被抛弃记录整段重放」（`.claude/kb/decisions/23-journal的角色与格式.md:376`）——要求「比 R_old 新的根全坏」，这是**坏块 / 数据损坏**，不是**截断**。而崩溃注入（这一件）的故障模型是 D13（验证路线） 已定项 4（`.claude/kb/decisions/13-验证路线.md:69-81`，已在背景材料整段抄）定的「屏障把写请求流切段，一个崩溃状态是前若干段全持久、当前段任意子集持久、之后各段全不持久」——**纯截断，不改坏已经持久的字节**。在纯截断模型下，任何已经落盘的根记录都保持可读、校验和不变；要让 `choose_root` 真的落回 R_old（而不是选到更新的根），只能是「更新的根从未被写过」（不是「被写过又损坏」）——这种情况下 R_old 之后的记录本来就是「尚未发生」的未来历史，不存在需要拦截的「被抛弃时间线」。**换言之，这一件（增补 3 第 3 件，截断模型）在设计上根本触及不到已定项 14 第五条要挡的那类危险状态**；能造出「更新的根全坏」这个前提的是增补 3 第 5 件（坏盘输入，位翻转/清零扇区），而背景材料明确写着这一件还没做（`_m2-supp3-item3-code-r1-background.md:365`「还没做：第 4–7 件」）。

### K2 结论

| 子问题 | 判定 |
|---|---|
| `effective_root` 与 D13（验证路线） 已定项 7「比对对象是实现恢复后的镜像」冲不冲突 | **不冲突，但理由窄**：`crash.rs`/`crash_injection.rs` 都只调只读的 `recover()`，不做会改盘的实例切换/重发 checkpoint，所以这里根本没有已定项 7 说的「两份镜像」，不是「两份镜像被正确分开处理」 |
| 构造「记录前缀施加到一半」 | **能构造，且已经真实发生过并已修复**（背景材料 4 条冷启动读回不对 → 接 `effective_root` 后全消）；今天的判读在这个状态下是对的 |
| 构造「所选根的实例有回退行，前缀被 W 截断」 | **构造不出来**：`rollback_high_water_of_root` 对任何 `root` 都恒返回 `None`（`instance_rows_to_write` 的区间保证一张表不会有描述它自己所属实例的行），这是 D23（journal 的角色与格式） 已定项 14 自己承认的已知缺口（C340），不是这一件新引入的问题；而它要挡的真正危险场景需要「坏块」，在这一件（纯截断模型）的故障域之外 |
| 会不会把一次真违例判成绿 | 我构造不出一个「今天可达、且 `effective_root` 用错了根」的状态；两个可能的候选一个已被修复验证过，另一个在当前设计+当前故障模型下不可达。**没能打中不代表以后也打不中**：C340 一旦按 P1（或别的读法）改掉「回退行落在哪张表」，`rollback_high_water_of_root` 就可能从「恒 None」变成「有时候有值」，那时候这条判读要重新核一遍——今天的「不冲突」是建立在「这个函数是死代码」之上的，不是建立在「这个函数的逻辑本身被验证过」之上的 |

**推翻条件**：① 有人跑出一段可达历史，`rollback_high_water_of_root` 对某个 `root` 返回 `Some`（说明我对 `instance_rows_to_write` 区间的读法有误，或者代码已经改了）；② C340 定案后回退行的落点改了，此时要重新判 `effective_root` 是否与新的口径一致；③ 增补 3 第 5 件（坏盘输入）接上之后，在「更新的根全坏」的状态上跑这条崩溃注入，看 `effective_root` 会不会落回 R_old 并重放被抛弃的记录——这是今天真正没被验证到的那一格。

（下一段开始 K3。）

## K3：模型目录「一步至多写出四条根，下一步之前必然还在环里」这个论证成不成立

### 代码现状

`crates/singlefs-harness/src/model.rs:665-680`（`committed_versions` 的文档注释与实现）：

```
/// 模型此刻根环里的每一条根：身份与它下面的文件内容（树表 0 条是 None）。
/// 崩溃注入（增补 3 第 3 件）每一步之后并一次，攒成「模型提交过的每一版」的目录，再拿它判崩溃状态恢复到的那一版
/// （[`crash_recovery_disagreement`]）。并不掉：根环有 R × S = 24 个槽，一步至多写出四条根（可写挂载的写行加暖机至多三次、
/// 抬 F 两次空发布），下一步之前必然还在环里。
pub fn committed_versions(&self) -> Vec<(ModelRootKey, Option<Rc<[u8]>>)> {
    self.ring.values().map(|root| (root.key, root.file.as_ref().map(|file| Rc::clone(&file.content)))).collect()
}
```

`R × S = 24`：`crates/singlefs-format/src/lib.rs:185-186` 定 `ROOT_RING_REGIONS = 3`、`ROOT_RING_SLOTS_PER_REGION = 8`，3×8=24，与注释一致（已核，命令见上一段）。

`crates/singlefs-harness/src/crash_injection.rs:418-421` 是这句论证被使用的地方：

```
let run = execute_history_with(history, execution, &stream, &mut |observation| {
    for (key, file) in observation.model.committed_versions() {
        committed_versions.insert(key, file);
    }
    ...
```

`committed_versions` 是外层一个 `BTreeMap`，**逐步 `insert`、从不清空**——它是「历史上每一步观察到的根环快照」的**并集**，不是「最后一步的根环快照」。这一点比 model.rs 注释字面说的更强，见下面「比论证更强的理由」。

### 一步最多能写出几条根：逐条数

历史里的一步只能是 `HistoryOperation` 的一个成员（`crates/singlefs-harness/src/history.rs:268-284`：`PublishFirstFile`/`PublishOverwrite`/`PublishWithoutUnits`/`CloseAndMountWritable`/`CloseAndMountRollback`/`RaiseRollbackFloor`/`ColdStartRecover`），不会在同一步里叠加两类操作。逐类数它各写几条根：

- `PublishFirstFile`/`PublishOverwrite`/`PublishWithoutUnits`：各一次 `publish_version` 调用，各写 **1** 条根。
- `ColdStartRecover`：只读，**0** 条。
- `CloseAndMountWritable`/`CloseAndMountRollback`：都走 `establish_instance`（`crates/singlefs-core/src/mount.rs:979`），先 `row_publish`（**1** 条根，1026-1064 行），再 `for planned_txg in &warm_up_publishes_planned`（1075-1084 行）逐次 `publish_empty_after`，`warm_up_publishes_planned` 由 `warm_up_publish_txgs`（923-948 行）算出：`covered` **预置为 `row_publish_txg` 自己那个设备**（931 行 `vec![device_of_txg(row_publish_txg)]`），循环上限 `< ROOT_RING_REGIONS`（3，937 行）。两块盘、设备序列按 `ROOT_RING_REGION_DEVICES = [0, 1, 0]`（`region = txg % 3`，`root_ring.rs:35`）：region 0→盘0、1→盘1、2→盘0。`covered` 已经算上 `row_publish` 自己那个设备，只需要覆盖**另一块盘**，逐一枚举 `row_publish_txg % 3` 的三种取值，最坏情况（`row_publish_txg % 3 == 2`）要 **2** 次暖机才覆盖到另一块盘（第一次仍落在已覆盖的盘上）。所以 `CloseAndMountWritable`/`CloseAndMountRollback` 一步最多写 **1 + 2 = 3** 条根，不是「至多三次」字面暗示的「暖机本身至多 3 次」——暖机最多 2 次，加写行的 1 次，合计 3。

（下一段继续算 `RaiseRollbackFloor` 的上限、并给出与「四条根」claim 的对照表。）

`RaiseRollbackFloor` 走 `raise_rollback_floor`（`crates/singlefs-core/src/mount.rs:609`）。它自己的空发布循环（676-706 行）：

```
let mut covered: Vec<DeviceIdentity> = Vec::new();
...
while all_devices.iter().any(|identity| !covered.contains(identity))
    && u64::try_from(publishes.len()).expect("次数") < ROOT_RING_REGIONS
{
    let next = publish_version(...);
    let device = device_of_txg(next.root.checkpoint_txg);
    if !covered.contains(&device) { covered.push(device); }
    publishes.push(next.clone());
    *current = next;
}
```

**这里 `covered`是空的 `Vec::new()`（677 行），不像 `warm_up_publish_txgs` 那样预置「当前状态自己的设备」**（对照 931 行）。逐一枚举「调用这个函数时 `current.root.checkpoint_txg % 3`」的三种取值（下一次发布的 txg 从 `current.checkpoint_txg + 1` 起，region 按新 txg 算）：

- 当前 txg % 3 = 0：下一批 region 依次是 1,2,0…→设备 1,0,0…，第 2 次迭代覆盖两块盘，**2** 次。
- 当前 txg % 3 = 1：下一批 region 依次是 2,0,1…→设备 0,0,1…，第 1、2 次都落在盘 0（region 2 和 0 都映射盘 0），第 3 次（region 1）才落盘 1，**3** 次——正好撞上 `ROOT_RING_REGIONS` 那个循环上限。
- 当前 txg % 3 = 2：下一批 region 依次是 0,1,2…→设备 0,1,0…，第 2 次迭代覆盖两块盘，**2** 次。

**`RaiseRollbackFloor` 的上限是 3，不是 model.rs 注释里写的「两次」**——当调用它时现行版本的 `checkpoint_txg % 3 == 1`，`raise_rollback_floor` 自己会写 3 条根（空发布），这条路是可达的：这个方法完全由 `checkpoint_txg` 的余数决定，不依赖任何特殊的历史构造，历史生成器只要在恰好那个 txg 上安排一次 `RaiseRollbackFloor` 就会撞上。

### 对照表：注释说的 vs. 代码实际的上限

| 操作 | 注释里写的 | 我数出来的 | 差在哪 |
|---|---|---|---|
| 可写挂载 / 回退（写行 + 暖机） | 「至多三次」（合计，1 行 + 至多 2 次暖机） | **1 + 2 = 3** | 一致 |
| 抬 F（空发布） | 「两次」 | **至多 3**（当前 txg ≡ 1 mod 3 时） | **注释算少了 1**：`raise_rollback_floor` 的 `covered` 起点没有预置「当前状态自己的设备」，与暖机那条路的写法不对称 |
| 单次发布类（写文件、覆盖写、零单元发布） | 不在注释的枚举里 | 1 | 注释没提，但显然只有 1，不影响结论 |

**一步实际的上限是 3（两条路都是 3，不是注释合计出来暗示的 4）**，比 24 个槽还远得多；「一步至多写出四条根」这句话本身把「暖机 3」和「抬 F 2」并列写在括注里，字面加总是 3+2=5、若各自取 max 是 max(3,2)=3，都对不上「四」这个数字——**这句括注的具体数字站不住，但它想论证的结论（一步写出的根远小于 24）没有被推翻，因为实际上限 3 比写错的「4」更小，不是更大**。

（下一段给出比这句论证本身更强的理由，以及 K3 结论表。）

### 比这句论证本身更强的理由：即使算错也不影响正确性

`execute_history_with`（`crates/singlefs-harness/src/history.rs:2635`）的循环里，`observer(&StepObservation { ... model: &pool.model, ... })` 在每一步 `apply_operation` 成功之后**立刻同步调用**（2756 行；出问题的那一步在 2732-2755 行判定 `!violations.is_empty() || harness_judgement.is_some() || model_disagreement.is_some()` 时提前 `return`，不会走到 2756 行的 `observer` 调用——但那种情况下整段历史直接停止，不存在「跳过一步的 observer 之后又继续写更多根」的可能）。`crash_injection.rs:418-421` 在这个回调里做的是 `committed_versions.insert(key, file)`——**逐步并入一个外层 `BTreeMap`，历史上任何一步曾经在 `self.ring` 里出现过的根，只要那一步的 observer 被调用过，就已经被并入这个目录，不会因为后面的步把它从模型自己的环里挤出去而被追溯删除**。

这意味着「一步至多写出几条根」这个数字，真正要小于的不是「下一步开始之前它还在环里」这么弱的条件，而是**「它不会在写出它的这同一步里、被同一步后面的写挤出环」**——因为 observer 是在这一步的全部写完成之后才调用一次，只要同一步自己写的根数不超过 24，这一步内部就不可能发生自己挤自己的情况。上面数出来的实际上限是 3，远小于 24，这个更强的条件同样满足。**换句话说，就算 model.rs 的括注数字是错的，只要错的方向不是「某一步真的能写出 ≥24 条根」，`committed_versions()` 的并集就不会漏掉任何一条曾经存在过的根**——而我逐类数过的三条路径（发布类 1、挂载/回退类 3、抬 F 类 3）没有一条接近 24。

### K3 结论

| 子问题 | 判定 |
|---|---|
| 「一步至多写出四条根」这句话本身对不对 | **不对**：应为「至多 3 条」（可写挂载/回退最坏 1+2=3；抬 F 最坏 3，不是注释写的 2；单纯发布类 1）。这是文档注释里的一处算术误差 |
| 有没有哪条路能在一步里写出超过 24 条根 | **没有**。三类路径的实际上限分别是 1、3、3，逐条数过，离 24 差一个数量级以上 |
| 「下一步之前必然还在环里」这个结论会不会被推翻 | **不会**：即便按错的「四」算，4 ≪ 24；按我数出来的「三」算，3 ≪ 24 更安全。此外 `committed_versions()` 的调用点是逐步并入外层 `BTreeMap`（`crash_injection.rs:418-421`），是并集不是快照替换，即便某一步真的挤出了环里的旧根，只要那条旧根曾经被某一步的 observer 观察过，它已经进了并集，不会被追溯删除 |
| 挤出去之后崩溃点落在那一版上会怎么判 | 这一格没有可构造的前提（没有任何一步能写出 ≥24 条根），**这一问在今天的代码上问不出对象**：不是「判得对不对」，是「这个状态到不了」 |

**推翻条件**：① 有人指出某个 `HistoryOperation` 成员我没数到（我核对的是 `history.rs:268-284` 枚举的全部 7 个成员，逐个数了它们各自调用的发布次数）；② `ROOT_RING_REGIONS`/`ROOT_RING_SLOTS_PER_REGION` 的值改了、或设备数从 2 块改成更多块，需要重算 `raise_rollback_floor`/`warm_up_publish_txgs` 的覆盖轮数上限；③ `execute_history_with` 改成不再每步都调 `observer`（例如批量攒几步才回调一次），那时「并集不会漏」这条更强的理由失效，需要退回去核「注释里那句话对不对」这个较弱的条件。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K2：`effective_root` 与 D13（验证路线） 已定项 7 的关系 | 一致（理由窄） | `crash.rs`/`crash_injection.rs` 都只调只读 `recover()`，没有已定项 7 说的「两份镜像」，不是两份被正确分开 |
| K2：构造「前缀施加到一半」 | 已验证、判读正确 | 背景材料记的 4 条冷启动读回不对，正是这个状态，接 `effective_root` 后已修复并留了钉住的变异（`crates/mutations.tsv:180`） |
| K2：构造「回退行 W 截断」 | 构造不出，死代码 | `rollback_high_water_of_root` 对任何 `root` 恒返回 `None`（`instance_rows_to_write` 的区间不含实例自己），与 kb 已承认的 C340 是同一件事 |
| K2：会不会把真违例判绿 | 今天构造不出反例 | 两个候选一个已修复验证、一个不可达；不可达的原因（死代码 + 需要坏块而这一件是纯截断模型）都查到了出处 |
| K3：「一步至多四条根」这句话对不对 | 不对，应为 3 | 可写挂载/回退最坏 3（1 行 + 2 暖机），抬 F 最坏 3（不是注释写的 2，`covered` 起点没预置当前设备） |
| K3：会不会挤出更早的版本 | 不会 | 实际上限 3 远小于 24；即便注释的错误数字「4」也远小于 24 |
| K3：论证本身站不站得住 | 站得住，但引用的具体数字有误 | 更强的理由（observer 逐步调用、外层并集不追溯删除）在实际上限 1/3/3 下同样成立，结论不受这处算术误差影响 |

## 没做什么

- K1（抽样罩不罩得住）、K4（截断粒度与层 0 的关系）：归云端攻方，没判。
- K5（并行与确定性）、K6（已知红那一格的判别力）：归本地攻方，没判。
- `crates/mutations.tsv:180` 那条变异是否真的会让 `crash_injection_fast_tier_recovers_only_into_versions_the_model_committed` 判红：只核对了条目存在、第 4 列改法与背景材料说法一致，没有实际复跑 `cargo mutants`（改 `crates/` 源码不在这条腿的写范围内）。
- D13（验证路线） 已定项 7 描述的「先跑实现自己的恢复、再跑记录核对器，恢复本身会改盘」这个完整流程，我只在 `crash.rs`/`crash_injection.rs` 两个文件里搜过（零命中 `mount_writable`/`mount_rollback`），没有搜索 `crates/` 之外的用例文件（例如 QEMU 真设备二进制），复核不了这个流程是否在别处存在。
- 增补 3 第 5 件（坏盘输入）还没有代码，K2 构造二里提到「需要坏块」的那个场景没有装置可以真的跑一遍，只能推理到「这一件的故障模型够不着它」为止。
- 没有跑门禁、没有编译，全部结论基于读代码与已跑过的 `grep`/`awk` 命令；涉及「这条变异会不会红」「这段历史在真实随机种子下会不会撞上 `RaiseRollbackFloor` 恰好在 txg≡1(mod 3) 时被调用」这类需要跑程序才能坐实的问题，标注为需要主 agent 或实现员复跑。
