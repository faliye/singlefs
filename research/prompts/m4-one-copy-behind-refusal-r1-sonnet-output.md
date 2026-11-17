# 坏副本放行第一轮：正推腿报告（2026-11-09）

批次：`records/2026-10-13-里程碑四预开发.md#收口修补二批`。立场：正推（从已定条款与 `crates/` 今天的实现推「应该是什么」，再与被判对象逐格比）。
背景材料：`research/prompts/_m4-one-copy-behind-refusal-r1-background.md`。模型目录：`research/prompts/m4-one-copy-behind-refusal-r1-sonnet-model/`。
没读禁读清单里的任何文件。方案都按 `crates/` 今天的实现谈；行号都是这一次 `grep -nF` 现查的。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 关 1：放行条件的写法 | 推得出，候选 1（点名窗口）的形态站得住，带一个前提 | 落后盘缺的单元里，只有 tail 之后那几条记录点名的才可能「从没落盘」；窗口之外缺的，本盘 tail 已见证它收过，是落盘之后坏的单份。前提是「tail 只在本盘收到单元之后才写进这块盘」，离线出路 α' 一处不满足 |
| 关 1：依赖的盘上字段 | 四样，全是今天的字段，没有新字段 | 两槽系统配置的 `journal_instance` 与 `journal_tail`、journal 环槽里记录头的 (实例代号, 计数器) 与点名项的位置条目、所选那一版的单元清单（重建出来的，不是盘上字段）、本盘在这些落点上的字节 |
| 关 1：判别子系统当时看得到吗 | 取号之前逐盘核与挂着之后的入口都看得到；验证包判官看得到但不被问；池级 checker 与 GPU/CPU 后台计算不在这条路径上 | 三处读者都是 `PoolReader`，同一函数共用，函数签名不动 |
| 关 1：改法与条款新句 | 已并排写（第 1.5 节）；没编没跑 | 改 `mount.rs` 一个函数加一个辅助函数；D18 已定项 11 改两句，D23 已定项 14 末段点名一读，D23 已定项 3 射程加一句前提 |
| 关 2：tail 取哪块盘 | 每块盘各用自己世代号最大那份槽的 tail；tail 值全池同值写到每块盘；会写到没收过单元的盘上，仅限离线出路 α' | 轮换写、取号写、抬 F 写之前各有一道核本盘单元的闸或单元屏障；α' 不核，是用户断言 |
| 关 2：点名是不是全集 | 每次发布恒点名它写出的全部单元（推的，两处由同一份角色清单装配）；空发布不写单元也不点名 | 别人在一条五次发布的流上量过点名集合与单元写集合全等，我没复跑 |
| 关 2：放行之后接着可写 | 后果与今天已放行的「只落一份记录、没有坏字节」同形；坏的那份由挂载写行与会话的搬迁批换；卸载之后判真盘绿不绿没有读数 | 待主 agent 在打了补丁的副本里跑新用例 |
| 关 2：候选 1、2 判得对不对（四节问 1） | 候选 1 在模型上普通断电加一份坏的拒错 0（今天 840）、旧盘换进来的放错 0；候选 2 字面读法拒错 252；「tail 被写高」那一族里候选 1 与紧邻一次发布比今天多放错 402、字面读法多放错 258 | 模型读数是我草稿目录里跑的，等主 agent 复跑 |
| 关 2：两块盘都落后、坏不止一份、坏的正是这次写的（四节问 4） | 模型里三种都不会让候选 1 拒；今天会拒 | 坏的正是这次发布写的那份：恢复先不施加这次发布，所选那一版退一格，不落后 |
| 关 2：改动面（四节问 5） | `mount.rs` 一函数加一辅助；`mounted_session.rs` 零改；GPU/CPU 零改；`mount_transparency.rs` 零改 | 见代价表 |
| 关 3：代价数 | 已每候选一行（第 3 节） | 正常挂载多读 0；只有「落后且本盘环槽没有末条记录」那一路才多读 |
| 模型 | 写了；pins 全绿（我草稿目录里的一次，当「推的」） | 复跑命令在「要主 agent 跑的命令」 |

模型文件 sha256（`sha256sum` 量的）：
- `behind_model.py`：`c87a4e09685ac782c163d12584ebe114e7f87587a1b3d7d6e1b5de2064423ab5`
- `tools/rerun.sh`：`79c3c1eb2fecec0dd4549fe0ca367a2ecd3eea44443d0a30b75f56ea346ecdda`
- `out/expected-output.txt`：`b8cba423451b7dee66e64d49e7b2095fb09b7a445fa7a607e0f499d7dbef5d0c`

## 1. 放行条件的精确写法

### 1.1 先把今天的判法读一遍

`crates/singlefs-core/src/mount.rs:7049`：「fn units_of_the_version_missing_on_a_device_behind_it<Reader: PoolReader + ?Sized>(」。它先后做三件事：
- `crates/singlefs-core/src/mount.rs:7061`：「if newest_system_configuration_journal_tail >= version_journal_position {」，tail 不落后就交回 `None`，单元一个不读。
- `crates/singlefs-core/src/mount.rs:7064`：「if last_record_of_the_version_in_its_ring_slot_on_the_device(」，本盘环槽里有那一版末条记录就交回 `None`。
- `crates/singlefs-core/src/mount.rs:7076`：「.filter(|unit| !unit.bytes.is_empty())」起，对**所选那一版的全部单元**逐个比本盘字节，读不出或不同的记进缺的清单。

所选那一版的单元清单是整版全集：`crates/singlefs-core/src/mount.rs:1326`：「Ok(Cow::Borrowed(&output.units))」，文档在 `crates/singlefs-core/src/mount.rs:1305` 一带写「全部角色」。所以被拒的三个状态里，缺的两份（映射树叶槽 50269、实例表槽 51136）是这一版沿用下来、这次发布没写的老单元。

### 1.2 条款怎么说，与它的理由

两边整行并排（左：条款；右：我从实现与写序推出的东西）。

| D18 已定项 11 的整句 | 实现与条款推出的 |
|---|---|
| `.claude/kb/decisions/18-块里携带什么信息.md:322`：「落后而这块盘的 journal 环槽里有那一版的末条记录、缺几份单元——单元写在末条记录那道屏障之前，末条记录在本盘是「落盘之后坏了一份」与「从没落盘」唯一分得开的盘上证据」 | `crates/singlefs-core/src/transaction.rs:1514`：「for unit in &writes.units {」→ 1521 屏障 → 1522 记录 → 1528 屏障 → 系统配置槽轮换（`crates/singlefs-core/src/transaction.rs:604`：「CommitStep::RotateSystemConfigurationSlots {」）。轮换写在单元屏障之后，所以**某块盘上读到 tail = c，就说明这块盘在写这个 tail 之前，已经收齐了计数器 ≤ c 的每次发布的单元**（推的，从写序推；前提见 1.4）。「唯一」不成立：tail 之前的单元，tail 本身就分得开 |
| 同一行的前一句：「系统配置跟得上、只缺几份单元（单份坏，由 D19（块指针的结构与宽度预算） 已定项 5 硬规则 1 的读盘核隔离）」 | 这一句已经承认：盘的 tail 够新时，缺的单元按单份坏处理。候选 1 的新意只是把这个许可从「按整版的 tail 比」改成「按每个单元被哪次发布写出来、那次发布在不在 tail 之内比」 |

`research/prompts/m3-close-sysconf-r3-main-verification.md:23`：「乙' 站住：单元写在屏障前、末条记录在屏障后，所以记录在本盘是能区分「落盘后损坏」与「从没落盘」的唯一盘上证据」。这一格站住的是「记录**能**分开两种」，被攻两轮没打中的也是这一点；「唯一」是顺带写进去的，那一轮没人问过 tail 能不能分开。我不推翻乙'（本盘有末条记录照旧放行，不动）。我补的是乙' 够不着的那一半。

### 1.3 D23 已定项 14 压着的两句

- `.claude/kb/decisions/23-journal的角色与格式.md:359`：「**恢复只施加 `(实例代号, checkpoint_txg)` 严格大于所选根的记录；陈旧 tail 只是「从哪开始扫环」的优化，不再决定重放集合。**」——tail 不决定重放集合，这条不碰。候选 1 用 tail 的地方不是重放，是落后支，D18 已定项 11 本来就在用。
- `.claude/kb/decisions/23-journal的角色与格式.md:364`：「**一个点名单元的两份（两条位置条目）都读得出、校验和都对才施加**，有一份读不出或对不上，这条记录与它之后的都不施加。」——这一句对窗口有直接后果：所选那一版若是**重放施加**得来的，被施加的每条记录点名的单元，两块盘上的两份都已验过（`crates/singlefs-core/src/recovery.rs:5477`：「|index: usize| every_named_unit_of_the_record_verifies(reader, records_in_key_order[index]);」）。窗口里**被重放的那几条**因此在落后支这一判之前就已经全盘验过，窗口真正要查的只剩「所选根已经覆盖、没有重放」的那几次发布。普通断电加单份坏的盘面里，这一段是空的（推的，模型里算出来也是空，见 4）。
- `.claude/kb/decisions/23-journal的角色与格式.md:138`：「落后支（D18（块里携带什么信息） 已定项 11）读 tail 不是这条依据。」——这是 D23 已定项 3 射程里的一句，说明「每次发布轮换系统配置槽并等它持久」的依据是 N-配置、不是落后支。候选 1 让落后支对 tail 的依赖更重（从「比大小」变成「当本盘收过的见证」），这一句要补一句前提，见 1.5。

### 1.4 判据（精确写法）与依赖的盘上字段

记：d 是被核的盘；T = d 上世代号最大的自证系统配置槽的 (`journal_instance`, `journal_tail`)；P = 所选那一版那次发布末条记录的 jsn（`crates/singlefs-core/src/mount.rs:9757`：「let selected_version_journal_position =」）；V 的单元清单 U。

d 在缺单元这一半「不带所选那一版」，当且仅当：
1. T < P（今天不动，`mount.rs:7061`）；
2. d 自己的环槽里没有 V 的末条记录（乙'，今天不动，`mount.rs:7064`）；
3. 缺的集合 M 非空，其中 M = { u ∈ U：u 的字节非空，d 上 u 的落点读不出或字节与验得过的那一份不同，且 **u 的落点（设备 d、槽 u.slot）被窗口 W 里某条记录点名** }；
4. W = 计数器在 (T.计数器, P.计数器] 的那几条记录，每条取池里任一块盘环槽上、同实例（= P 的实例代号）、同计数器、自证过的那一份。**取不全就退回今天**：T 的实例代号不是 P 的实例代号、W 里有一个计数器在任何一块盘上都取不到、或 P.计数器 − T.计数器 大于环槽数，W 取作「覆盖 U 的全部单元」。

依赖的盘上字段，全是今天已有的：
- 系统配置槽：`journal_instance`、`journal_tail`（`crates/singlefs-core/src/mount.rs:6769`：「.max_by_key(|slot| slot.quantities.slot_generation);」取每块盘世代号最大的那份）；
- journal 记录头：实例代号、计数器；点名项 `NamedUnit.locations`（设备、槽、校验和，`crates/singlefs-core/src/journal.rs:161`：「pub struct NamedUnit {」）。记录按 `header_csum` 罩整条（D23 已定项 13），点名项不会半读；
- 单元清单 U 与字节：不是盘上字段，是挂载读阶段重建出来的那一版（`crates/singlefs-core/src/mount.rs:7724`：「let units_of_the_selected_version = start.previous.units(」）与本盘字节。

「同实例同计数器」就够认出窗口里的记录，不必走反向链：同一实例每个计数器只写一次（重发冻结的发布写的是逐字节相同的记录），不同实例的同计数器记录是 (实例代号, 计数器) 不同的键（`crates/singlefs-core/src/recovery.rs:5256`：「// 同一 (实例代号, 计数器) 先到先得（按上面的盘次序、环内偏移次序，第一份读到的留下）：留哪一份在」）。跨实例的窗口不取（上面第 4 点），因为跨实例边界没有链可核。

这个判据成立的前提（推的）：**tail 只在这块盘收到 ≤ tail 的各次发布的单元之后才写进这块盘**。写 tail 的入口一共五处，逐个核（行号是这次 `grep -nF` 取的）：
- 发布末尾的轮换：`crates/singlefs-core/src/transaction.rs:604`，在单元屏障之后写、每块盘各写一份（`transaction.rs:608`：「for index in 0..self.devices.len() {」）。前提成立。
- 取号写：`crates/singlefs-core/src/transaction.rs:1308`：「if let Err(cause) = pool.write_system_configuration_slot(」，tail 取见证值 = 每块盘两槽里 tail 的最大值（`transaction.rs:1010`：「fn highest_journal_tail_of(slots_of_every_device: &[Vec<SystemConfiguration>]) -> u64 {」），**写到每一块盘**。这一支前面有取号之前的逐盘核（`mount.rs:7792` 一带），每块盘都得过它才取号，前提成立（成立的程度正是这条判据自己）。
- 抬 F 写：`crates/singlefs-core/src/mount.rs:5581`：「match write_the_raised_floor_into_every_system_configuration(」，写的是现行那一版的末条计数器；它前面的入口核是同一个落后支判（`mounted_session.rs` 与 `mount.rs` 里 `devices_behind_the_current_version` 共用）。前提成立。
- 取号失败的回卷写：`transaction.rs:747` 起的 `roll_back_acquisition`，写的仍是同一个见证值；只在取号写已经过了逐盘核之后发生。前提成立。
- **离线出路 α'**：`crates/singlefs-core/src/system_configuration_repair.rs:264`：「let journal_tail_written = match (」，tail 取断言那一版末条记录的计数器，只核记录或根在不在、单元区没有更新的出生 txg，**不核本盘有没有那一版的单元**；写入口在 `system_configuration_repair.rs:383`：「.write_system_configuration_slot_carrying_the_pool_effective_floor(」。前提在这一支上由管理员的断言承担。

这一支是候选 1 的弱点，不是它独有的：今天 tail ≥ P 时单元一个都不读（`mount.rs:7061`），α' 之后的盘同样放过。候选 1 只是把这个洞的射程从「所选那一版」延到「所选那一版往后每一版里窗口之外的单元」。模型里单独造了这一族，数在第 4 节。

### 1.5 改法与条款新句，并排

**代码**（`crates/singlefs-core/src/mount.rs`，没编、没跑，clippy 与格式交主 agent；`Option::is_none_or` 我没核工具链版本，用了 `map_or`）。左是今天的 `mount.rs:7049` 起那个函数后半，右是改后；前三段（tail 取法、tail 不落后即 `None`、本盘环槽里有末条记录即 `None`）一字不动。

今天（`mount.rs:7073`–`:7084`）：

```rust
    let units_missing: Vec<(TransactionUnit, SlotNumber)> = version
        .units
        .iter()
        .filter(|unit| !unit.bytes.is_empty())
        .filter(|unit| {
            !reader
                .read(device, unit.slot.to_device_offset(), unit.bytes.len())
                .is_some_and(|bytes_on_this_device| bytes_on_this_device[..] == unit.bytes[..])
        })
        .map(|unit| (unit.identity, unit.slot))
        .collect();
    (!units_missing.is_empty()).then_some((newest_system_configuration_journal_tail, units_missing))
```

改后（并排的另一边）：

```rust
    let slots_named_after_the_tail = slots_on_the_device_named_by_the_records_after_the_tail(
        reader,
        parameters_of_the_pool,
        device,
        newest_system_configuration_journal_tail,
        version_journal_position,
    );
    let units_missing: Vec<(TransactionUnit, SlotNumber)> = version
        .units
        .iter()
        .filter(|unit| !unit.bytes.is_empty())
        .filter(|unit| {
            slots_named_after_the_tail
                .as_ref()
                .map_or(true, |slots| slots.contains(&unit.slot))
        })
        .filter(|unit| {
            !reader
                .read(device, unit.slot.to_device_offset(), unit.bytes.len())
                .is_some_and(|bytes_on_this_device| bytes_on_this_device[..] == unit.bytes[..])
        })
        .map(|unit| (unit.identity, unit.slot))
        .collect();
    (!units_missing.is_empty()).then_some((newest_system_configuration_journal_tail, units_missing))
```

新增辅助函数（放在 `last_record_of_the_version_in_its_ring_slot_on_the_device` 前面；类型、函数名都取自这个文件今天已用的，没核 `SlotNumber` 有没有 `PartialEq`——`SelectedVersionLackingOnDevice` 派生了 `PartialEq`，它的成员里装着 `SlotNumber`，所以有）：

```rust
/// tail 之后到那一版末条记录之间（含末条）每条记录点名、落在 `device` 上的槽号。本盘的 tail 是这块盘收过 tail 及之前每次发布的单元之后才写下的，
/// 缺在这些槽之外的单元是落盘之后坏了的单份，由 D19 已定项 5 硬规则 1 的读盘核隔离，不算「不带」。
/// 交回 `None`（今天的读法：缺的按整版算）：tail 的实例代号不是末条记录的实例代号（计数器跨实例不连）、窗口比环的槽数还长、
/// 窗口里有一个计数器在池里任何一块盘的那一槽上解不出同实例同计数器的记录。只读盘。
///
/// 循环：按窗口里的计数器（轮数 = 末条计数器 − tail 计数器），内层按盘找盘次序里先解得出的那一份；跨轮只往槽号清单里追加，提前出口一个：某个计数器取不全，整个交回 `None`。
fn slots_on_the_device_named_by_the_records_after_the_tail<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    parameters_of_the_pool: &MakeFilesystemParameters,
    device: DeviceIdentity,
    tail: JournalSequenceNumber,
    version_journal_position: JournalSequenceNumber,
) -> Option<Vec<SlotNumber>> {
    let ring_bytes = parameters_of_the_pool.geometry.journal_ring_bytes;
    if tail.instance_generation != version_journal_position.instance_generation
        || version_journal_position.counter - tail.counter > ring_bytes / singlefs_format::JOURNAL_RECORD_BYTES
    {
        return None;
    }
    let record_bytes =
        usize::try_from(singlefs_format::JOURNAL_RECORD_BYTES).expect("一条记录 4096 字节");
    let filesystem_identifier =
        crate::unit::unit_filesystem_identifier(&parameters_of_the_pool.filesystem_identifier);
    let mut slots = Vec::new();
    for counter in (tail.counter + 1)..=version_journal_position.counter {
        let offset = crate::journal::record_offset(counter, ring_bytes);
        let record = reader.device_identities().into_iter().find_map(|holder| {
            reader
                .read(holder, offset, record_bytes)
                .and_then(|bytes| crate::journal::JournalRecord::parse(&bytes, filesystem_identifier))
                .filter(|record| {
                    record.instance == version_journal_position.instance_generation
                        && record.counter == counter
                })
        })?;
        for named in &record.named {
            for location in &named.locations {
                if location.device == device {
                    slots.push(location.slot);
                }
            }
        }
    }
    Some(slots)
}
```

函数本体的 `///`（`mount.rs:7040`–`:7048`）里「单元写在记录之前那道屏障之前，末条记录在本盘是……唯一分得开的盘上证据」一句同改，`mount.rs:848` 一带 `units_missing` 的文档「所选那一版有单元在这块盘上它的落点读不出……」加一句「只算 tail 之后的记录点名的」。`devices_behind_the_current_version`（`mount.rs:7245`）、`what_a_device_behind_the_version_lacks`（`mount.rs:7132`）、会话那一处（`mounted_session.rs:2575`：「devices_behind_the_current_version(devices, parameters, current_version_of_trees);」）都经这个函数，**签名不动、调用方不改**。

**条款新句**（D18 已定项 11，同一行 `.claude/kb/decisions/18-块里携带什么信息.md:322`，整行在 1.7 抄）。两处并排，左是今天的整句，右是新句：

| 今天 | 新句 |
|---|---|
| 「世代号最大那份系统配置的 (实例代号, journal tail) 小于所选那一版那次发布末条记录的 jsn，**并且**所选那一版有单元在这块盘上它的落点读不出或字节不同、这块盘自己的 journal 环槽里又没有那一版那次发布的末条记录，或所选那一版就是择到的那条根、它的根槽归这块盘而那一槽读一次、重读一次都不是它」 | 「世代号最大那份系统配置的 (实例代号, journal tail) 小于所选那一版那次发布末条记录的 jsn，**并且**所选那一版有单元在这块盘上它的落点读不出或字节不同、**而这个落点被 tail 之后的记录点名**（窗口 = 计数器在 (这块盘 tail 的计数器, 所选那一版末条记录的计数器] 的那几条记录，每条取池里任一块盘环槽上同实例同计数器、自证过的那一份；tail 的实例代号不是那一版末条记录的实例代号、窗口里有一条哪块盘上都取不到、或窗口比环的槽数还长，窗口当作覆盖那一版的全部单元）、这块盘自己的 journal 环槽里又没有那一版那次发布的末条记录，或所选那一版就是择到的那条根、它的根槽归这块盘而那一槽读一次、重读一次都不是它」 |
| 「落后而这块盘的 journal 环槽里有那一版的末条记录、缺几份单元——单元写在末条记录那道屏障之前，末条记录在本盘是「落盘之后坏了一份」与「从没落盘」唯一分得开的盘上证据」 | 「落后而缺的单元都不被 tail 之后的记录点名——这块盘的 tail 是在那些发布的单元与屏障之后才轮换写下的（D16 已定项 7 的持久顺序），本盘收过 tail 及之前每次发布的单元，缺在窗口之外的是落盘之后坏了的单份，与「系统配置跟得上、只缺几份单元」同一类；落后而这块盘的 journal 环槽里有那一版的末条记录、缺几份单元——单元写在末条记录那道屏障之前，窗口之内末条记录在本盘是「落盘之后坏了一份」与「从没落盘」分得开的盘上证据」 |

另三处联动句（都只加不删）：
- D23 已定项 14 末段 `.claude/kb/decisions/23-journal的角色与格式.md:376`：「这一读不算按干净关闭分支」后加「，窗口里每条记录那一槽的读同样点名认下（C499（干净关闭标记有没有没有条款））」。
- D23 已定项 3 射程 `.claude/kb/decisions/23-journal的角色与格式.md:138`：「落后支（D18（块里携带什么信息） 已定项 11）读 tail 不是这条依据。」后加「落后支缺单元那一半把 tail 当本盘收过单元的见证，前提是 tail 只由单元屏障之后的轮换写、与逐盘核过本盘单元之后的取号写、抬 F 写写进盘；离线出路（α'）写的 tail 不核本盘单元，那一支由管理员的断言承担」。
- C593 行（`.claude/kb/checks-owed.md` 第 355 行）的「立账时的缺口」后加「乙' 没罩到的一形：记录只落盘 1、盘 0 那一份没落，早先弄坏的单元在盘 0 上；点名窗口罩它」，销账证据加三个状态。

### 1.6 每个判别子系统当时看得到吗

| 子系统 | 看得到 tail | 看得到窗口的记录 | 看得到单元字节 | 要不要动 |
|---|---|---|---|---|
| 取号之前的逐盘核：`devices_passing_the_check_before_the_acquisition`（`crates/singlefs-core/src/mount.rs:8063`）→ `devices_without_the_selected_version`（`mount.rs:6741`） | 看得到，自己读两槽（`mount.rs:6756` 一带） | 看得到：读盘不经读阶段共用的那一份（`mount.rs:8084`：「} = with_reads_straight_to_the_disk(read_route, || {」），环槽用 `reader.read` 取；读阶段扫环得到的记录表（`recovery.rs:5222` 的 `scan_journal`）不在 `InstanceStart` 里，我的写法不要它 | 看得到，今天就读 | 只动被调用的那个函数 |
| 挂着之后的入口（正常卸载、抬 F、管理员回退、会话每次发布）：`devices_behind_the_current_version`（`mount.rs:7245`） | 看得到，同函数 | 看得到：同一个 `reader`；窗口的记录在好盘的环槽里，被换进来的旧快照盘的环里没有它们，所以窗口取自「任一块盘」 | 看得到 | 零改 |
| 验证包判官 `judge_writable_mount_refusal`（`crates/singlefs-verification/src/mount_transparency.rs:3301`：「let Some(reason) = refusal_reason_on_the_d23_list(refusal) else {」） | 它读崩溃镜像，看得到 | 看得到 | 看得到 | 零改。这个拒因今天不在 D23 已定项 14 的单子上（`mount_transparency.rs:2330`：「| MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion { .. }」），「判法不认」即「理由不在单子上，照崩溃注入第二截今天的写法判，别的报红」（`mount_transparency.rs:3282`）。候选 1 之后崩溃枚举里不再有这个拒因，出现就报红是对的 |
| 池级 checker（`crates/singlefs-checker/src/lib.rs:209`、`:395` 只把 `journal_tail` 当字段读出来） | 读字段，不判落后支 | 有逐盘 journal 不变量（I-8.7、I-8.9、I-8.12、I-8.13），条款字面只判读得出的记录 | 判单元，不比两盘 | 零改 |
| 后台计算 GPU 与 CPU（`crates/singlefs-background-gpu`、核心里的 `background_compute.rs`） | 不在这条路径上 | 不读 | 校验和批与活槽、区域记录的核，服务整理与 GC | 零改；落后支这一判只用 `PoolReader::read` 加切片相等，没有一处走后台计算 |

最后一行的依据命令与原样输出（这一次跑的）：`grep -rlE 'journal_tail|落后支|BehindTheSelected' crates/singlefs-background-gpu crates/singlefs-checker crates/singlefs-mount-runtime crates/singlefs-unit-graph crates/singlefs-replay-prefix crates/singlefs-reference-filesystem --include=*.rs` 只命中 `crates/singlefs-checker/src/lib.rs` 一份（GPU 包零命中，量的）。

## 2. 第二节三处没证与第四节五问

### 2.1 没证一：tail 是跨盘取的见证值，会不会写到没收过单元的盘上

命令与原样输出（这一次跑的）：`grep -n 'write_system_configuration_slot\|write_system_configuration_slot_carrying' crates/singlefs-core/src/*.rs`，去掉注释行之后，系统配置槽的写入口：

```
crates/singlefs-core/src/system_configuration_repair.rs:383:                .write_system_configuration_slot_carrying_the_pool_effective_floor(
crates/singlefs-core/src/transaction.rs:609:                    self.write_system_configuration_slot(
crates/singlefs-core/src/transaction.rs:638:        self.write_system_configuration_slot(
crates/singlefs-core/src/transaction.rs:772:            if let Err(rollback_error) = self.write_system_configuration_slot(
crates/singlefs-core/src/transaction.rs:882:        if let Err(cause) = pool.write_system_configuration_slot(
crates/singlefs-core/src/transaction.rs:1308:            if let Err(cause) = pool.write_system_configuration_slot(
```

`transaction.rs:638` 是 `…_carrying_the_pool_effective_floor` 的函数体，它的调用方只有 `system_configuration_repair.rs:383` 一处（同一条 grep 的输出里 `carrying` 只在这两行）；mkfs 另在 `crates/singlefs-core/src/make_filesystem.rs:789` 写 `journal_tail: 0`。所以写 tail 的路径：发布末尾轮换（609）、取号写（1308）、取号回卷（772）、抬 F 写（882）、离线出路 α'（383）、mkfs。

答：
- 值是**全池同值**写到每块盘：轮换写 `for index in 0..self.devices.len()`（`transaction.rs:608`），取号写取每块盘两槽 tail 的最大值写到每块盘（`transaction.rs:1010` 与 `:1261`：「let witnessed_journal_tail =」）。判落后时每块盘各取自己世代号最大那份槽（`mount.rs:6769`），不取池里的最大 tail。
- 「会」写到没收过单元的盘上的只有 α'（1.4 逐处核过）。其余四处，前面都有一道核本盘单元的闸或写在单元屏障之后：轮换（屏障之后）、取号与回卷（取号之前逐盘核）、抬 F（入口的落后支核）。这是推的，依据是读码，没有一个用例在「取号写写高了没收过单元的盘的 tail」上红过。
- 「取号之前逐盘核」这道闸自己就是候选 1 要改的函数，所以候选 1 之后这一条前提成立的程度，等于候选 1 自己：窗口之内缺的单元仍拒，窗口之外缺的当单份坏放行，放行之后取号把 tail 写到 P，窗口之外的坏份留在盘上，不会再被当成「没收过」，因为它们本来就在 tail 之内。
- 推翻它的现象：找到任何一条写 tail 的路径把 tail 写成 ≥ c 而本盘没有计数器 ≤ c 的发布点名的某个单元（α' 除外），或造一个历史让 α' 之外的写入口造出这个状态。模型里 α' 造出的那一族见第 4 节。

### 2.2 没证二：取号、暖机、抬 F、写行这几种发布的记录点名是不是全集

「全集」指：一次发布写进盘的每个单元，都被这次发布的某条记录点名。窗口规则整个依赖它：写了而没点名的单元，窗口外缺它就被当成单份坏放行。

逐种读码（`crates/singlefs-core/src/transaction.rs`）：
- 单元写只有一处循环：`transaction.rs:1514`：「for unit in &writes.units {」（经 `CommitStep::WriteUnitToEveryDevice`，每块盘各写），`PublishWrites.units` 与 `PublishWrites.records` 在同一个函数里一起装。
- 带文件的发布：`transaction.rs:3492`：「/// 装一次发布要写的单元：这次重写的每个角色按角色取它在这一版 `units` 里的单元（`publish_admitted` 的 `written_units`）。」与 `transaction.rs:3495`：「/// 装一次发布的记录：点名的每个角色按角色取它在这一版 `units` 里的单元（`publish_admitted` 的 `named_unit_of`），」，写的和点名的取自同一份「这次重写的角色」；`transaction.rs:14008`：「pub fn roles_named_by_each_record_of_the_publish(」把它切成记录：写了 N 个数据单元（N ≥ 2）就是 N 条记录，`transaction.rs:13998`：「/// 第 k 个事务（k < N − 1）一条记录、只点名文件第 k 个数据单元；这次发布共享的提交内生块（数据单元之外的每个重写角色）」，共享的节点只在最后一个事务的记录里点名。
- 写行那次发布（每次可写挂载都有一次）：点名的角色 `transaction.rs:2248`：「let named_roles: Vec<TransactionUnit> = settled.rewritten_roles.clone();」，写的单元从同一份 `settled.rewritten_roles` 筛出来（`crates/singlefs-core/src/transaction.rs:2395`：「let units: Vec<PublishedUnit> = settled」）。
- 暖机、推抬 F 的空发布：不写单元也不点名，`transaction.rs:1810`：「        named: Vec::new(),」与 `transaction.rs:1816`：「        units: Vec::new(),」。
- 取号：不是发布，只写系统配置槽，不写单元，不写记录。
- 整理批（搬迁）：点名项 = 搬迁副本 + 这次重写的角色，`transaction.rs:13615`：「(relocated_copies + rewritten.len()).div_ceil(named_entry_capacity.named_entries_per_record())」。
- mkfs 写的单元（树表、实例表、根）没有记录，tail 0；那是窗口之外，缺了按单份坏处理，与今天一致。

量的：别人在一条五次发布的流上，10 个记录写的点名集合与前一段单元写的 (盘, 槽) 集合逐次相等，命中弄坏落点 0 个（`/home/fy5090/.local/share/singlefs/work/m4-close-partial-persist-investigator-2/result.md` 第 129 行那一段，转述，我没复跑）。

缺口（我搜到的范围：这一次对测试名与池级 checker 不变量表搜了 `named` 与 `written`、`names_every`、`点名` 几种写法，`crates/singlefs-checker/invariants/03-journal.md` 里的 I-8.11 判的是「点名项指到的单元头与点名项相同」，方向是点名 → 单元；**没找到「写出的单元都被点名」这个反方向的检查**。窗口规则一落地就把它从「没人依赖」变成「承重」，这一格要先有会红的检查（推的；可造的最小检查：每次发布装好的 `PublishWrites` 里 `units` 的 (设备, 槽) 集合等于 `records` 点名项位置条目的集合，在 `persist_publish_writes` 之前判）。

### 2.3 没证三：放行之后盘 0 没有这条记录，接着发布、卸载、判真盘

- **同形态今天已被放行**：记录只落盘 1 而盘 0 没有坏字节的状态，今天挂得上；别人量的 63 个没弄坏的状态全部挂得上（`/home/fy5090/.local/share/singlefs/work/m4-close-partial-persist-investigator-2/result.md` 第 42 行：`PROBE_TALLY never-damaged states=63 mounted=63 not_mounted=0`，转述）。所以「盘 0 的环槽缺 411」本身不是候选 1 新造的盘面，新造的只是「同时盘 0 上有早先弄坏的单元」。
- **接着发布**：下一个计数器取读阶段扫环得到的最大计数器加一（`crates/singlefs-core/src/mount.rs:9865`：「let next_counter = records」），记录表里有盘 1 上那条 411，所以新实例从 412 起；盘 0 的 411 那一槽没人补写，洞留着。取号写把 tail 写成 (新实例, 411) 到每块盘（`transaction.rs:1308`），之后盘 0 的 tail 不落后，下一次入口核在 `mount.rs:7061` 就返回，不再碰单元（推的）。
- **下一次崩溃**：只要取号写的两轮没有全持久，盘 0 的 tail 仍是旧值，下一次挂载重新走候选 1，结局不变（放行）。今天则是每次都拒。
- **坏的那一份由谁换**：D4 已定项 4 的搬迁接得上，因为接口在放行之前就建好了：读阶段建所选那一版时读到只坏一份的单元，位置条目记在 `PreviousVersion` 里（`mount.rs:9856`–`:9863`），不经落后支这一判，放行与否不改它。`.claude/kb/decisions/04-校验和位置.md:73`：「豁免映射的三类（映射树节点、树表、实例表片）认不出映射 key，不进计划：它们在挂载这一串的发布里本来就换落点」——实例表片在每次可写挂载的写行发布里整条链重写（`transaction.rs:2248` 一带），映射树叶节点在挂载之后第一次用户改动之后另发的搬迁批里换（`crates/singlefs-core/src/mounted_session.rs:2211`：「fn relocate_the_units_read_with_one_copy_failing_after_the_change<Device: BlockDevice>(」，豁免映射的单元由 `mounted_session.rs:2249`：「let repair = repair_of_the_exempt_units_at_these_locations(」换）。挂载之后一直没有用户改动，映射叶上的坏份就不换（D4 已定项 4 原话：「挂载之后一直没有用户改动就不搬」，`04-校验和位置.md:71`）。
- **卸载之后判池级 checker**：**没有读数**。池级 checker 的逐盘 journal 不变量里，I-8.9 的条款字面写了只判读得出的记录（`crates/singlefs-checker/invariants/03-journal.md:16`：「只判读得出的记录，读不出的那条不判」）；I-8.7、I-8.12、I-8.13 逐盘判环里自证过的记录，缺一条在不在它们的判据里，我没逐条核（转述，推的）。核心层的反向链读者对「前一条不在盘上」的处理是（`crates/singlefs-core/src/recovery.rs:5278`：「判不了，不拿它断前缀」），池级 checker 那一份的 I-8.6 我没核到。别人也没跑到：「判真盘还是没跑到」（`m4-close-partial-persist-investigator-2/result.md` 第 156 行）。**待主 agent 在打了补丁的副本里跑新用例**（命令在末尾）。

### 2.4 四节问 1：候选 1、2 判得对不对

读法：候选 2 有两种读法，要分开。背景材料写「只在 tail 紧挨所选发布时放宽」；调查员的原文写「`tail.counter + 1 == 所选.counter`」。我叫前者「紧邻一次发布」（窗口恰为所选那一次发布的记录），后者「紧邻一版（字面）」（只差一个计数器）。

`transaction.rs:13998` 说明一次发布可以是 N 条记录（N ≥ 2 个数据单元），所以「所选那一次发布」的末条计数器减 tail 可以大于 1：字面读法在多记录发布上不放行。这是读码推的；模型数在第 4 节。

模型读数（`tools/rerun.sh` 的输出，我在草稿目录里跑的一次，等主 agent 复跑，当推的）：

| 族（描述见第 4 节） | 状态数 | 今天拒错 | 候选 1 拒错 | 紧邻一版（字面）拒错 | 紧邻一次发布拒错 |
|---|---|---|---|---|---|
| 普通崩溃加一份坏 | 122736 | 840 | 0 | 252 | 0 |
| 普通崩溃加两份坏 | 829702 | 8308 | 0 | 2568 | 0 |
| 旧盘换进来（缺单元是真的没收过） | 28080 | 0（拒了 21528，全对） | 0 | 0 | 0 |

放错（该拒而放）：「旧盘换进来」三个候选与今天都是 0。「tail 被写高而没收单元」一族里，今天已经放错 11856 / 14040（tail 不落后，单元一个不读）；候选 1 与紧邻一次发布比今天**多**放错 402，紧邻一版（字面）多放错 258。

- 找一段历史，放行之后盘 0 真的缺这一版要的、而 tail 之后又没点名的单元：**有，条件是 tail 被写高而没收单元（α'）**。模型里的最小一段（`EXAMPLE … kind=newly_false_allow`）：发布 1 写角色 a、发布 2 写角色 b，各一条记录；盘 1 缺 (a, 发布 1)，它的 tail 却是发布 1 的末条计数器；发布 2 的记录只落盘 0，所选那一版是发布 2；今天拒（盘 1 缺 a），候选 1 放（a 不在窗口里）。在 α' 之外造不出来（2.1）。
- 或者该放行的还拒：候选 1 与紧邻一次发布在模型里 0 个；紧邻一版（字面）有，最小一段：发布 1 写 a，发布 2 写 a、b 两个数据单元切两条记录，记录只落盘 1，盘 0 上早先的 c 单元坏；所选那一版是发布 2，tail = 发布 1 末条，tail + 1 ≠ 所选末条，字面读法退回今天的整版比，拒。
- 候选 1 与紧邻一次发布在这批族里读数相同，说明紧邻一次发布没有换来额外安全，只是读码更窄；差别要靠没建的族（跨实例窗口、环转圈）才分得出，那两样候选 1 都退回今天的读法，所以它们不分。

要造到真盘面的最小做法（待主 agent 另派）：「普通崩溃加坏」那一族照别人已有的探针形状（任务 11560 那条用例）加上候选 1 的补丁；「tail 被写高」那一族需要一个调 `rewrite_system_configuration_slots_for_the_asserted_version`（`system_configuration_repair.rs:230`）的历史步，我没读它在验证装置里有没有现成的调法，不写进「已有」。

### 2.5 四节问 4：两块盘都落后、坏的不止一份、坏的一份正是这次发布写的

都是模型里的态（`ReasonNotOnTheD23List` 之外不判），状态数见上表：
- **两块盘都落后**：本盘环槽里有末条记录的盘直接放行（`mount.rs:7064`）。两块盘都没有末条记录而所选那一版又是重放得来的，不可能：重放要求那条记录读得出，读出来必在某一块盘的环槽里（推的）；反之所选那一版是根环里的根，它的记录在根 FUA 之前就屏障过、两盘都有。真两盘都缺要两个记录槽同时坏（两个故障），模型不造；那时窗口里取不到末条记录，窗口退回今天的整版比，结局与今天一样。
- **坏的不止一份**：只要每个单元还有一份好的（读阶段建得出那一版），窗口规则对每个单元独立判；两份坏落在窗口之外都放行，落在窗口之内且本盘没有末条记录才拒。模型「普通崩溃加两份坏」的 829702 个状态里候选 1 拒 0 个，今天拒 8308 个。**一个单元的两份都坏**：读阶段建不出那一版，挂载在读阶段就失败，到不了这一判（模型里的 `buildable` 筛掉这些）。
- **坏的一份正是这次发布写的**：恢复施加这条记录前要验它点名单元的两份（`.claude/kb/decisions/23-journal的角色与格式.md:364`），有一份坏就不施加这次发布，所选那一版退到上一版，tail 不落后，不进这一判（模型里同样）；这次发布的根已经落盘（所选那一版是根环里的根）时，两块盘的环槽里都有这次发布的记录（记录在根之前屏障过），走乙' 放行。窗口之内缺单元而又拒的，只剩「本盘那条末条记录的环槽自己也坏」这一种两故障态，没造。

### 2.6 四节问 5：改动面（各文件要不要改）

| 文件 | 候选 1 点名窗口 | 说明 |
|---|---|---|
| `crates/singlefs-core/src/mount.rs` | 改：`units_of_the_version_missing_on_a_device_behind_it`（`:7049`）加三行过滤与一个辅助函数；`:7040`–`:7048` 与 `:848` 一带两处文档注释跟改 | 取号之前逐盘核与挂着之后的入口共用这一个函数，一处改两处用 |
| `crates/singlefs-core/src/mounted_session.rs` | 不改 | 调用 `devices_behind_the_current_version`（`:2575`），签名不动 |
| GPU 与 CPU 后台计算 | 不改 | 落后支这一判不经后台计算（1.6 的 grep） |
| `crates/singlefs-verification/src/mount_transparency.rs` | 不改 | 拒因今天不在单子上（`:2330`），候选 1 之后崩溃枚举里不会出现这个拒因 |
| 用例、变异表、kb | 改 | 见第 3 节 |

## 3. 代价数

每候选一行。「挂载多读」分两格：正常挂载（tail 不落后，或本盘环槽里有末条记录——这两种在 `mount.rs:7061`、`mount.rs:7064` 提前返回）与「落后且本盘环槽没有末条记录」那一路。数里的 L = 所选那一版末条记录的计数器 − 这块盘 tail 的计数器，D = 盘数，U = 所选那一版的单元数。没有量的，都是从读码推的算术。

| 候选 | 改动面 | 正常挂载多读 | 落后且无末条那一路多读 | GPU / CPU 后台计算 | 判官与判法（`mount_transparency.rs`） |
|---|---|---|---|---|---|
| 候选 1 点名窗口（记录读本盘环槽以外任一盘的环槽） | `mount.rs` 一个函数加约 8 行、一个新函数约 45 行、两处文档注释；用例：改 `crash_points_of_the_repair_of_the_units_exempt_from_the_mapping.rs` 那条的断言（三个状态不再拒）、加两条（窗口之内缺单元仍拒、窗口之外坏份放行）；`crates/mutations.tsv` 现有 625 与 4831 两行要核还红不红、加一行「窗口不看点名」；kb：D18 已定项 11 两句、D23 已定项 14 与已定项 3 各一句、C593 一格，各三份语言文件 | 0 | 今天读 U 个单元（每个 16 KiB 或 32 KiB）；候选 1 读至多 L × D 个 4 KiB 记录槽，再读窗口点名交集里的单元（不多于 U）。L 超过环槽数时我写的函数直接退回整版比；默认环 196608 槽（D23 已定项 18 写「默认环下 196608 ÷ 3 = 65536」），最坏 768 MiB × D，等于读阶段每次挂载本来就扫的整环（`recovery.rs:5222` 的 `scan_journal`），不更多（推的）；普通崩溃那一族 L 就是所选那次发布的记录数 | 0 | 0 |
| 候选 1，记录改取读阶段已扫的 `records` | 同上，另在 `InstanceStart`（`mount.rs:1461` 一带）加一个记录表字段，沿 `establish_instance` 往下传；会话那一处（`devices_behind_the_current_version`）用不到读阶段的表，要另读盘，等于两套 | 0 | 0 | 0 | 0 |
| 候选 2 紧邻一版（字面，`tail.计数器 + 1 == 所选末条计数器`） | 同一函数，不要窗口循环，读一个记录槽；用例要加「落后两版仍整版比」 | 0 | 至多 D 个记录槽 | 0 | 0 |
| 候选 2 紧邻一次发布（窗口恰为所选那一次发布的记录） | 同一函数，另要从末条记录的「本次发布内序号」算出首条计数器（`journal.rs` 里记录头有这一字段） | 0 | 至多 N × D 个记录槽，N 是所选那次发布的记录数 | 0 | 0 |
| 认下拒（产品不动，单子加一行） | `mount_transparency.rs`：`RefusalReasonOnTheD23List` 加一个成员、`refusal_reason_on_the_d23_list`（`:2275`）改 `:2330` 那一臂、`conditions_not_met_by_a_refusal_on_the_d23_list` 加一臂复算（落后、本盘无末条记录、所选那一版的单元在本盘字节不同）；D23 已定项 14 的单子、D13 已定项 1 的射程句、新用例的断言、变异行；kb 同样三份语言 | 0 | 0 | 0 | 要改（上一格） |
| 附加：离线出路 α' 写 tail 之前核本盘单元（与候选 1 或 2 搭配） | `system_configuration_repair.rs:264` 之后、`:383` 之前加一道「本盘带所选那一版的全部单元」，复用 `rewrite_gate_not_met_by_a_device_without_any_self_verified_system_configuration` 第一件（`mount.rs:6859` 一带）；用例一条；这一支关掉模型里「tail 被写高」那一族 | 0 | 离线一次读 U 个单元每块被写的盘 | 0 | 0 |

「多少步」的另一种数法（改动点个数，推的）：候选 1 是 1 个函数改、1 个函数新增、2 处文档注释、3 条用例、2 行变异核对加 1 行新增、kb 4 句话乘 3 份语言；候选 2 少一个循环；认下拒产品 0 步、验证包 3 处、kb 3 句话乘 3 份语言，但与 C593 立账的话相撞（C593：「一次正常崩溃（最后一次系统配置轮换没持久）加同一块盘上一份单元坏（单故障），可写挂载被拒、盘上不变、之后每次都拒」，`.claude/kb/checks-owed.md:355`），要用户定。

## 4. 模型

位置：`research/prompts/m4-one-copy-behind-refusal-r1-sonnet-model/behind_model.py`。纯 Python，不读 `crates/`，不编译；从 D18 已定项 11 与 D23 已定项 14 的条款字面和 D16 已定项 7 的持久顺序（`.claude/kb/decisions/16-发布语义.md:215`：「一次发布的持久顺序恒为 COW 单元 / 节点 → 屏障 → journal 记录 → 屏障 → 根槽（FUA）→ 系统配置槽 → 屏障」）重述判法，**先写臂、后看数**：今天的判法照 `mount.rs:7049` 起那个函数的行为重述一份，用「别人量到的三个被拒状态的形状」当它的已知读数（模型的「今天」臂在「普通崩溃加一份坏」族里拒了 840 个，形状与那三个同形，见下面的例子）。

范围：两块盘、三个角色、两次与三次发布、单实例；每次发布切一条或两条记录；崩溃只在最后一次发布，按三段（单元写之前、记录写之间、轮换写之间）逐写独立落或没落地枚举；一份坏 = 那块盘上它的字节验不过；没收过 = 那块盘从没写进过它。不判根槽那一半，不判跨实例窗口，不造记录槽自己坏的两故障态，不造环转圈，不造三块盘以上。

族与臂：
- 普通崩溃加一份坏、加两份坏：崩溃态之上，任意一块盘上任意已落盘单元的一份（两份）弄坏。真值：没有任何一块盘缺单元（只坏不缺），池该放行。
- 旧盘换进来：盘 1 换成早先某次发布之后的快照（整点或发布中途），盘 0 是崩溃态。真值：盘 1 缺活单元时池该拒。
- tail 被写高而没收单元：盘 1 缺早先几次发布的单元与记录，tail 却写着跟上了（α' 造的形状）。真值同上。今天的判法在这一族上本来就会放（tail 不落后不读单元），所以这一族另报「候选比今天多放错的」。
- 臂：今天、点名窗口、紧邻一版（字面）、紧邻一次发布，加两条对照臂（一律放、一律拒）。

会红的检查：对照臂「一律放」在「旧盘换进来」上放错 > 0、对照臂「一律拒」在「普通崩溃加一份坏」上拒错 > 0、今天的判法在「普通崩溃加一份坏」上拒错 > 0（否则没造出那一形）、今天在前两族与两份坏上放错 = 0、今天在「tail 被写高」上放错 > 0。五条任一不成立，模型退 1 并打 `PIN_FAILED`。这是模型自证它的计数器分得出对错；不是实现的变异。

读数（我在草稿目录里直接跑过一次，输出与 `out/expected-output.txt` 逐行同；等主 agent 经服务复跑，之前都是推的）：

```
family=F1_crash_and_one_damaged_copy arm=today states=122736 refused=840 over_refuse=840 false_allow=0 newly_false_allow=0
family=F1_crash_and_one_damaged_copy arm=window states=122736 refused=0 over_refuse=0 false_allow=0 newly_false_allow=0
family=F1_crash_and_one_damaged_copy arm=adjacent_literal states=122736 refused=252 over_refuse=252 false_allow=0 newly_false_allow=0
family=F1_crash_and_one_damaged_copy arm=adjacent_publish states=122736 refused=0 over_refuse=0 false_allow=0 newly_false_allow=0
family=F2_stale_disk_swapped_in arm=today states=28080 refused=21528 over_refuse=0 false_allow=0 newly_false_allow=0
family=F2_stale_disk_swapped_in arm=window states=28080 refused=21528 over_refuse=0 false_allow=0 newly_false_allow=0
family=F2_stale_disk_swapped_in arm=adjacent_literal states=28080 refused=21528 over_refuse=0 false_allow=0 newly_false_allow=0
family=F2_stale_disk_swapped_in arm=adjacent_publish states=28080 refused=21528 over_refuse=0 false_allow=0 newly_false_allow=0
family=F3_stale_disk_tail_raised_without_units arm=today states=14040 refused=402 over_refuse=0 false_allow=11856 newly_false_allow=0
family=F3_stale_disk_tail_raised_without_units arm=window states=14040 refused=0 over_refuse=0 false_allow=12258 newly_false_allow=402
family=F3_stale_disk_tail_raised_without_units arm=adjacent_literal states=14040 refused=144 over_refuse=0 false_allow=12114 newly_false_allow=258
family=F3_stale_disk_tail_raised_without_units arm=adjacent_publish states=14040 refused=0 over_refuse=0 false_allow=12258 newly_false_allow=402
family=F5_crash_and_two_damaged_copies arm=today states=829702 refused=8308 over_refuse=8308 false_allow=0 newly_false_allow=0
family=F5_crash_and_two_damaged_copies arm=window states=829702 refused=0 over_refuse=0 false_allow=0 newly_false_allow=0
family=F5_crash_and_two_damaged_copies arm=adjacent_literal states=829702 refused=2568 over_refuse=2568 false_allow=0 newly_false_allow=0
family=F5_crash_and_two_damaged_copies arm=adjacent_publish states=829702 refused=0 over_refuse=0 false_allow=0 newly_false_allow=0
```

「普通崩溃加一份坏」今天拒错的最小一段（模型输出的 `EXAMPLE` 行，原样）：

```
EXAMPLE family=F1_crash_and_one_damaged_copy arm=today kind=over_refuse selected_version=2 tails={0: 1, 1: 1} crash=(('记录段', (False, True)), '损坏', (0, ('b', 0))) shape=[(('a',), 1), (('a',), 1)]
```

读法：两次发布各写角色 a；记录段，第 2 次的记录只落盘 1；盘 0 上 mkfs 的 b 单元坏；恢复施加第 2 次发布（所选那一版 = 2），盘 0 的 tail 停在 1，环槽里没有计数器 2：与调查员量到的三个被拒状态同形（记录只落盘 1、盘 0 的 tail 落后一条、缺的是早先弄坏、这次发布没写的单元）。

模型怎么改过（照实写）：「tail 被写高」那一族我造过两种。先造的那种里盘 1 换成旧快照、没参加最后一次发布，候选与今天读数相同，分不出臂；改成盘 1 参加最后一次发布、缺的是更早几次发布的单元，才有了「候选比今天多放错」的读数。另外「今天在 tail 被写高上放错 > 0」那条检查是看了先造那种的读数之后加的。其余族与臂没改。

模型的局限（读数时带着）：「点名全集」在模型里是假设（2.2 的缺口）；「恢复施加要点名单元两份都验过」是照条款字面写的，没读恢复代码的每一支；窗口里的记录取自「任一块盘」，模型里记录不会被坏盘上的旧时间线冒充（单实例）；真值只看「这块盘有没有收过」，不看根槽那一半。

## 5. 要主 agent 跑的命令

模型复跑（纯 Python，一条）。在仓根起；跑完看 `task-server/task wait` 末行 `TASK_SERVER_RESULT` 的 `exit=0`，`task-server/task log <号>` 里有一行 `PINS ok`，最后一行 `name=behind_model … diff_rc=0`（与 `out/expected-output.txt` 逐行比，`diff` 无输出）；`exit` 非 0 或 `diff_rc` 非 0 算不过：

```
SINGLEFS_RUN_SCOPE='种类=items; 跑什么=m4-one-copy-behind-refusal-r1 正推腿状态模型的 tools/rerun.sh; 范围=1 个纯 Python 模型脚本、两次与三次发布的全部崩溃态; 为什么=复跑模型读数并与 out/expected-output.txt 逐行比，判决要引的数由主 agent 复跑' task-server/task submit -- bash research/prompts/m4-one-copy-behind-refusal-r1-sonnet-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs/research/prompts/m4-one-copy-behind-refusal-r1-snapshot --work /home/fy5090/.local/share/singlefs/work/m4-one-copy-behind-refusal-r1-sonnet-rerun
```

模型不用冻结副本里的任何文件（`--frozen` 只为让模板的参数检查过）。

打了 1.5 的改法之后才有的命令（现在还没有那棵树，**待主 agent 先建副本并把改法打进去**，新派的实现员带「承接：本报告」）：在打了补丁的副本里 `--cwd <那棵副本>`，各为一个测试目标里点名的测试函数，不整包：
- `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 crash_points_of_the_repair_of_the_units_exempt_from_the_mapping 里点名的一个测试函数; 范围=1 个测试目标里 1 个测试函数; 为什么=核点名窗口原型让坏盘流 153 个状态都挂得上，其余状态卸载之后判真盘' task-server/task submit --cwd <副本> -- cargo test -p singlefs-verification --test crash_points_of_the_repair_of_the_units_exempt_from_the_mapping every_persisted_set_of_the_batches_on_the_disk_with_the_damage_still_on_it_mounts_writable_and_is_green_after_the_unmount`。看 `test result: ok. 1 passed`；这条用例今天在三个状态上拒（调查员量的），补丁后该过。**它同时回答 2.3 的「卸载之后判真盘」没有读数那一格。**
- 守住的三条现有用例（调查员列的，我核了它们在原文件里存在，没读全文）：`--test writable_mount_takes_a_device_holding_the_last_record_of_the_version_as_not_behind`（里面「发布单元写到一半时取下的快照盘照拒」那条，函数名 `a_device_snapshotted_halfway_through_the_unit_writes_of_the_version_is_still_refused_before_any_write`）、`--test fsync_drop_and_devices_without_the_selected_version stale_device_one_refuses_the_writable_mount_naming_every_unit_it_misses_before_any_write`、`--test entries_after_mount_refuse_swapped_or_behind_devices stale_snapshot_substitution_is_refused_by_every_entry_over_the_swept_user_steps`。三条都该仍过。
- 命名检查：我没改 `crates/` 下的 `.rs`，不需要；原型补丁打出来之后由实现员照共用约束写。

## 6. 什么现象会推翻这些结论

| 结论 | 推翻它的现象 |
|---|---|
| 窗口之外缺的单元是落盘之后坏的单份（1.2、1.4） | 找到 α' 之外的一条写 tail 的路径，把 tail 写成 ≥ c 而本盘缺计数器 ≤ c 的发布点名的单元；或一块诚实盘上轮换写持久而它之前的单元屏障没完成（违反 `transaction.rs:1514`–`:1528` 的写序） |
| 点名是全集（2.2） | 任一种发布装好的 `PublishWrites` 里，`units` 的 (设备, 槽) 集合有一项不在 `records` 点名项的位置条目里 |
| 候选 1 在普通断电加单份坏上拒 0 个（2.4） | 补丁后新用例仍有拒；或主 agent 复跑模型读数与 `out/expected-output.txt` 不同 |
| 候选 2 字面读法在多记录发布上拒错（2.4） | 多记录发布的真盘面上 `tail + 1 == 所选末条计数器` 总成立，即我对 `transaction.rs:13998` 的读法错 |
| 放行之后的后果与今天已放行的同形相同（2.3） | 补丁后「卸载之后判真盘」红，而红的位置不在今天已放行的同形（没弄坏的 63 个状态）里出现过 |
| 后台计算与判官零改（1.6） | 落后支这一判的调用链上出现后台计算的调用，或验证包判官在候选 1 之后的崩溃枚举里遇到了这个拒因 |
| α' 是唯一破前提的路径（1.4、2.1） | 造出一个不经 α' 的历史，盘上 tail ≥ c 而本盘缺 ≤ c 的发布点名的单元，并且今天的判法与候选 1 都放过它 |

## 7. 没做什么

- 没编、没跑 `crates/` 里的任何东西；1.5 的补丁和辅助函数没编，签名里用到的 `SlotNumber`、`JournalRecord::parse`、`record_offset` 的用法按这个文件里今天的调用照写，clippy 与格式没过。
- 没造真盘面：候选 1 在真池上放行之后接着发布、卸载、判池级 checker 的读数没有（2.3）；「tail 被写高」那一族没在 `crates/` 里造过，只在模型里。
- 没读 `m3-close-sysconf-r3` 的两条腿，只读了判决 `research/prompts/m3-close-sysconf-r3-main-verification.md:23` 那一行。
- 没核英日两份 kb（`18-information-carried-in-a-block.en.md` 一类）的同条，改时要核。
- 只在两块盘、三个角色、两次与三次发布、单实例上建了模型；没扫三块盘以上、跨实例窗口、环转圈、记录槽自己坏。
- 我在草稿目录里直接跑过模型四次（改模型时三次、定稿后一次；定稿后那一次与 `out/expected-output.txt` 逐行同）；按共用约束这些读数在主 agent 复跑之前都是推的。没往统一任务服务交任务，没起后台任务。
- 不判别：没替主 agent 采纳候选、没出判决；岔路单（`research/prompts/<轮>-forks.md`）归主 agent。
- 没在 kb 里改一个字；条款新句在 1.5 里只是提案。

## 8. 条款原文（整行从文件抄，供 1.5 并排用）

`.claude/kb/decisions/18-块里携带什么信息.md` 第 322 行（已定项 11，要改的两句都在这一行里）：

>   - **取号之前逐盘核带不带所选那一版**（主 agent 2026-09-25 定，被攻过零轮；实现 `crates/singlefs-core/src/mount.rs` 的 `devices_without_the_selected_version`，实二七）：可写挂载核施加前缀之后那一版；交进来的盘有一块不带就在取号之前、任何写之前拒可写（`MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion`），只读挂载不做这道核、逐条试位置条目读所选那一版。「不带」两种：两个系统配置槽一份自证过的都没有（校验和过、fsid 与本池相同），即这块盘不「可见」；世代号最大那份系统配置的 (实例代号, journal tail) 小于所选那一版那次发布末条记录的 jsn，**并且**所选那一版有单元在这块盘上它的落点读不出或字节不同、这块盘自己的 journal 环槽里又没有那一版那次发布的末条记录，或所选那一版就是择到的那条根、它的根槽归这块盘而那一槽读一次、重读一次都不是它（读不出、自证不过，或自证得过而 (实例, txg) 不同；拒成 `SelectedVersionLackingOnDevice::BehindTheSelectedVersionAndItsRootNotInItsRingSlot`）；施加过 journal 记录得来的那一版崩在根槽 FUA 之前、哪块盘的根环里都没有它，不核根槽那一半。放行三种：只落后、单元都在、根槽那一半不缺（根落盘之后、轮换之前崩；取号失败回卷写回取号那一刻的见证值（C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙-配置续）），这是正常崩溃状态；系统配置跟得上、只缺几份单元（单份坏，由 D19（块指针的结构与宽度预算） 已定项 5 硬规则 1 的读盘核隔离）；落后而这块盘的 journal 环槽里有那一版的末条记录、缺几份单元——单元写在末条记录那道屏障之前，末条记录在本盘是「落盘之后坏了一份」与「从没落盘」唯一分得开的盘上证据（C593（落后且缺一份单元的盘被永久拒可写） 那一形；`crates/singlefs-core/src/mount.rs` 的 `units_of_the_version_missing_on_a_device_behind_it` 先判 `last_record_of_the_version_in_its_ring_slot_on_the_device`）。挂着之后收盘表的每个入口（正常卸载、管理员回退、会话每次发布、抬 F）在任何写之前核三样：「可见」那一支（一块盘一份本池 fsid 的自证系统配置槽都没有就拒）；「可见」之后，每块盘自证槽里的本盘设备号等于盘表给它的身份；落后支——这块盘最新那份自证系统配置的 (实例代号, journal tail) 小于现行那一版末条记录的 jsn，**并且**（现行那一版有单元在这块盘上它的落点读不出或字节与这个进程验得过的那一份不同、这块盘自己的 journal 环槽里又没有那一版的末条记录，或现行那一版的根槽归这块盘而那一槽读一次、重读一次都不是它），才拒（改法 C，与可写挂载的落后支同一判，「所选那一版」换成调用方持着的现行那一版；缺根那一半的重读照 D16（发布语义） 已定项 1「根槽这一次读坏」读一次、重读一次，单元那一半读不出直接记缺、不重读；只落后、单元都在、根槽里是它的不拒，只剩一槽自证、那一槽停在上一次轮换的盘由此不误拒）。后两样用户定，起因与依据在代码三方第二轮判决 `research/prompts/m2-closeout-code-r2-main-verification.md`「Y2-a 同池旧快照的盘经挂着之后的入口「洗白」」与「盘体对调的盘表」两格；落后支取「落后且缺单元」、不取「落后就拒」，照攻方原型 `fix_prototype_swapped_or_behind.patch`（`m2-impl-r2-fixes-b-implementer-report.md` 第四节第 6 条）。会话「每次发布」按「每次用户改动」读（一次调用只核一次，在循环之前），循环里为空间不够推的每一串抬 F 自己再核一次三样、紧挨着重发之前，所以实际发生的每个写之前都核过——这是推的，没有用例在循环中途把盘换掉。三样都已实现：「可见」那一支见 `m2-impl-z3a-yi-implementer-report.md`；本盘设备号与落后支在 `crates/singlefs-core/src/mount.rs` 的 `caller_inputs_agreeing_with_the_disk`（正常卸载、抬 F、管理员回退；本盘设备号那一判是 `own_device_numbers_on_disk_differing_from_the_identities_handed_in`，落后支是 `devices_behind_the_current_version`，与可写挂载共用 `what_a_device_behind_the_version_lacks`：先 `units_of_the_version_missing_on_a_device_behind_it`，单元都在再 `root_of_the_version_not_in_its_ring_slot_on_the_device`）与 `crates/singlefs-core/src/mounted_session.rs` 的 `publish_user_change`（会话），用例 `crates/singlefs-verification/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs`。拒时报的成员：「可见」——正常卸载、抬 F 报 `MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration`（`disagreeing_device_table` 带 `NoSelfVerifiedSystemConfigurationOnTheDeviceHandedIn`），管理员回退报 `RollbackError::CallerInputsDisagreeWithTheDisk`，会话报 `UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration`；本盘设备号——正常卸载、抬 F 报同一个 `MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration`（`disagreeing_device_table` 带 `OwnDeviceNumberDiffersFromTheIdentityHandedIn`），管理员回退报 `RollbackError::CallerInputsDisagreeWithTheDisk`，会话报 `UserChangeRefused::OwnDeviceNumbersDifferFromTheDeviceTable`；落后支——正常卸载、抬 F 报 `MountError::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot`，管理员回退报 `RollbackError::CallerInputsDisagreeWithTheDisk`（带 `CallerInputsDisagreeingWithTheDisk::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot`），会话报 `UserChangeRefused::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot`。

`.claude/kb/decisions/23-journal的角色与格式.md` 第 376 行（已定项 14 末段）：

> **第一版没有干净关闭标记**：每次挂载一律走恢复（全环扫描、逐条验证、择根，按五条口径与注 1–4 施加），不按上一次是不是干净关闭分支；准入、暖机、回退候选集、F 的生效都不读这样一个标记。取号之前逐盘核时，落后支缺单元那一半读本盘环里那一版末条记录所在的那一槽（系统配置三方第三轮判决 `research/prompts/m3-close-sysconf-r3-main-verification.md` 第二节 U3 的乙'，实现 `crates/singlefs-core/src/mount.rs` 的 `last_record_of_the_version_in_its_ring_slot_on_the_device`），这一读不算按干净关闭分支（C499（干净关闭标记有没有没有条款） 的读法，用户 2026-10-20 定「认，点名这一读」）。

`.claude/kb/decisions/23-journal的角色与格式.md` 第 138 行（已定项 3 射程，整行）：

> **射程**：管 tail 住哪、多久写一次、恢复怎么用它。字节表 [layout/01-first-txn.md](../layout/01-first-txn.md) 零与一两节按「每次发布一次」写：两次暖机空发布与新池新建文件各推进一次 tail。⚠️ 取系统配置形态时用过的代价表（E23（journal 几何） 的「两条臂差 21 : 500」）已作废：第一版每次发布都是 checkpoint，稳态多写从 21 次变成每次发布两盘各一槽；重放那一维不存在（恢复不读 tail、每次挂载全环扫描）。「每次发布轮换系统配置槽、返回前等它持久」的依据是判据 N-配置（已定项 14「可写挂载读到的更新状态读不出」，C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙、C577（系统配置没见证到的最新根，乙罩不到））：只减轮换次数或不等它持久就返回，「已确认的发布 + 根槽一时读不出 + 一份点名单元坏」时丢已确认的发布，而且那一格与「根槽 FUA 之前崩 + 一份单元坏」盘面逐字节相同、改读者分不开。这条依据成立在那一类双故障在容错射程里（用户 2026-09-28 定在）；落后支（D18（块里携带什么信息） 已定项 11）读 tail 不是这条依据。
