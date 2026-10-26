# m3-close-rootring-r1 云端正推腿报告（Sonnet）

日期 2026-10-20。立场：正文第四节分工表「云端正推」——从 `crates/` 今天的实现出发，把 C743、C751、C755、C773 四笔用户选定那一支的条文写出来，逐处列改动点与结局，量代价；不再论要不要选。
读的东西：主工作区 `crates/` 现状（行号都是这一次现查的主工作区行号）；系统配置落地补丁 `~/.local/share/singlefs/work/m3-close-w8-sysconf-landing/patch/crates.patch`（5037 行，只读）；kb 条款（写 kb 文件名加那份文件自己的行号，或 `文件@小节标题`）。

## 约定

- 判定词：一致（条文写得出、与已定条款和实现对得上）、冲突（两句并排抄在该格里）、规则没说（条款没写、我只能给候选读法）。
- 补丁前后：补丁改 `mount.rs`、`recovery.rs`、`transaction.rs`。补丁在 `recovery.rs` 的改动全在 1506 行之前（择系统配置一段），`read_root_ring_slot`、`choose_root`、F2 判法函数不在补丁里，只是行号整体后移；`mount.rs` 补丁改动从 5695 行起，所以 5695 行之前的行号（重建分配器、抬 F 读根环、`rebuild_previous_version`）补丁前后不变，之后的（F2 退回调用点、`root_of_the_version_not_in_its_ring_slot_on_the_device`）补丁后后移，按函数名认。补丁新增的读根环处与 F1 重扫在各格里点名，写明落在补丁前还是补丁后。
- 没跑任何 cargo、门禁、重型测试（派发写了「重型测试：不跑」）；下面凡是「会红」「会绿」「走得到」都是读码推的，标「推的」；别的会话量过的数标「别人量的」并写出处。
- 没有建模型目录：这一轮的论证是读码加算术，没有需要独立复跑的模型。

## 各格判定一览（开头版；末尾另有一张）

| 格 | 判定 | 依据 |
|---|---|---|
| X1 C743 错位读收进失败模型 | 一致，带 3 处冲突（测试手摆根）与 2 处规则没说 | 读根环槽的入口收在 `read_root_ring_slot`（`crates/singlefs-core/src/recovery.rs:1555`）与 `read_root_ring_slot_telling_never_written_apart`（`crates/singlefs-core/src/mount.rs:2720`）两个读函数加 2 处直接解析（`instance_table_sweep.rs:481`、`crates/singlefs-core/src/recovery.rs:4172`），在读函数里加一道落点公式核就收齐；合法写者（`crates/singlefs-core/src/transaction.rs:508` 与 mkfs 第 0 代）写出来的根都过这一核；已核实会红的测试 3 个 |
| X2 C751 只读挂载在那一形照报失败 | 冲突（入库用例 `:656`）；规则没说（改一处还是改共用判法） | 条件写得出：所选根的实例表读不出，且（见证到的那一版，所选根）之间有读得出的根；`:656` 的盘面就是这一形，它的只读挂载断言（`:674`）要翻；单改只读挂载会让恢复落 P、只读报失败，新造一处分叉 |
| X3 C755 三处连触发一起管 | 一致，带 3 处规则没说 | 「建得出」= 走读、打开、建上一版三样读的单元并集都读得出；只在系统配置没见证所选那一版时才多读，常态路径不多读一个单元；残留的差别从「落哪一版」收窄到「落到之后成不成」 |
| X4 C773 回收门槛落盘 | 一致，带 4 处同步改的冲突（数字与「同取六处」那一句） | 根记录在分配记录树根指针之后、算法类型之前加 8 字节（457 到 465），journal 新根段末尾加 8 字节（364 到 372，记录头 487 到 495），写盘次数与崩溃状态数不变；I-3.1 改式后 C773 登记行那一形转绿（推的，复现盘面已不在，复核不了）；与 C684 不冲突，但「已认的代价」在 I-3.1 上落空 |
| X5 四笔之间 | 一致，带 2 处要写进条文的衔接 | X1 把错位读收成「读坏」后，X4 的「环里最旧有效根」在 core 与 checker 两边会不同取，要在 checker 的重算里也按落点公式滤；X2 的条件住在 X3 统一出来的判法函数里，次序要写死；别的没有互相打架的地方 |

## X1　C743 错位读收进失败模型

### X1.1 条文（要写进 kb 的字）

D17 已定项 1 的定案句（`.claude/kb/decisions/17-实现分层与第三方管道.md@已定项 1：单一失败模型，核心层合成`）现在是「核心层规定一个失败模型——扇区 512 原子、写可撕裂到扇区、屏障之间可重排」。在「屏障之间可重排（…口径…）」之后加这一句：

> 读根环槽读回的可以是本池另一个根环槽里一条自证得过的根（读请求被送到别的槽、或别的盘的同一偏移）；核心层用落点公式合成：读根环槽读出的根，它的 checkpoint_txg 照 `root_ring::target_for_publish` 不该住这一槽的（第 0 代根只认槽 0、不核区域），按自证不过处置，core 里读根环槽的每一处同一处置。这一项只收根环槽的读；读回别池的字节（fsid 已挡）、字节自证不过（校验和已挡）、读回同一槽较旧一圈写下的内容（位置对、时间错，归「写失败的槽按旧内容算」）、写错位（I-7.17 判）不在这一项里。

D16 已定项 1「根槽这一次读坏」那一行（`.claude/kb/decisions/16-发布语义.md:39`）开头的「挂载那一刻就读不出或自证不过」改成「挂载那一刻就读不出、自证不过、或读出的根按落点公式不住这一槽（D17 已定项 1 的读可错位）」；同一行后面两处「读不出或自证不过」同改。池级 checker 的 I-7.17 那一行（`crates/singlefs-checker/invariants/02-根环.md:24`）末尾加「读错位只在 core 处置（镜像里没有读故障），不进 checker」。C743 登记行（`.claude/kb/checks-owed.md:495`）随实现销账。

### X1.2 读回的是什么，进不进模型

| 读回的是 | 进不进 | 理由 |
|---|---|---|
| (a) 本池另一个根环槽里一条自证得过的根 | 进 | 校验和与 fsid 都过，现行读法分不出（`RootRecord::parse_slot` 只验这两样，`crates/singlefs-core/src/root_record.rs:128`）；落点公式一步判得出；这就是 C743 的形 |
| (b) 别的盘同偏移的槽 | 进，不需要新处置 | 第一版区域归属写死 0、1、0「pub const FIRST_VERSION_REGION_DEVICES: [DeviceIdentity; 3] =」（`crates/singlefs-core/src/make_filesystem.rs:63`），mkfs 在每块盘上清三个区域的整段「device.write_zeroes_at(region_start(region_to_clear), root_ring_region_bytes)?;」（`crates/singlefs-core/src/make_filesystem.rs:561`），根只写在属主盘；别的盘同偏移读回全零（现行「读回全零」类）；同池同偏移若真有根，就是 (a) |
| (c) 别的偏移的任意字节 | 进，不需要新处置 | 要么 magic 或校验和不过（现行自证不过），要么恰好是本池某个根环槽的内容，就是 (a) |
| (d) 同一槽较旧一圈写下的内容 | 不进 | 位置对、时间错，公式看不出（txg 与 txg + 3S 落同一槽）；归「写失败的槽按旧内容算」 |
| (e) 写错位 | 不进 | 写者缺陷，不是读故障；I-7.17 判（`crates/singlefs-checker/src/image.rs:1265`） |
| (f) 别的自证结构（系统配置槽、journal 记录、单元）读错位 | 这一项不管 | 它们各有自己的位置绑定（槽世代号与槽号、jsn、位置条目），这一轮没查；条文限定「根环槽」，别写成全结构 |

### X1.3 core 里读根环槽的入口，逐个列

读根环槽的字节只经 4 个地方解析：`read_root_ring_slot`「pub fn read_root_ring_slot<Reader: PoolReader + ?Sized>(」（`crates/singlefs-core/src/recovery.rs:1555`）、`read_root_ring_slot_telling_never_written_apart`「fn read_root_ring_slot_telling_never_written_apart<Reader: PoolReader + ?Sized>(」（`crates/singlefs-core/src/mount.rs:2720`）、实例表清扫自己的读「root_read = RootRecord::parse_slot(&bytes, filesystem_identifier);」（`crates/singlefs-core/src/instance_table_sweep.rs:481`）、挂载透明取字节用的「(RootRecord::parse_slot(&bytes, &root.filesystem_identifier).as_ref() == Some(root))」（`crates/singlefs-core/src/recovery.rs:4172`）。其余入口都经前两个读函数，所以公式核加在这 4 处。

| 入口（调用点） | 经哪个读函数 | 改后错位读的结局 | 结局变没变 |
|---|---|---|---|
| 恢复择根：`recover`（`crates/singlefs-core/src/recovery.rs:7054`）→ `choose_root`（`crates/singlefs-core/src/recovery.rs:1582`） | 前者 | 错位槽摊成 `None` 进择根核（`choose_root_slot`，Verus 验的那份不动） | 合法盘面不变；错位根不再可能被选中 |
| 可写挂载读阶段、只读挂载择根（`mounted_read.rs:626`）、`instance_table_of_the_newest_root_read_at_most_twice`（`crates/singlefs-core/src/mount.rs:1496`、`:1502`） | 前者（都经 `choose_root`） | 同上 | 同上 |
| 重建分配器两遍：`read_every_root_ring_slot_once`（`crates/singlefs-core/src/mount.rs:2984`） | 后者 | 错位槽归「读坏不是全零」：回收门槛钉最旧 txg、并进抬 F 上限；改前被记成这一槽的占用者（`crates/singlefs-core/src/mount.rs:3327` 那一类的来源） | C743 的形消失，其余不变 |
| 抬 F 那一串：`readable_roots_for_raising_the_floor`（`crates/singlefs-core/src/mount.rs:4404`）、`roots_of_the_ring_for_the_ceiling`（`crates/singlefs-core/src/mount.rs:3528`） | 前者（经 `crates/singlefs-core/src/recovery.rs:1710` 那一族） | 错位 = 读坏：知道住着根的槽重读，仍坏拒这次抬 F | 改前错位根被收下、拿错的 txg 算上限；改后拒或重读，更保守 |
| 抬 F 那一串里给根归槽：`oldest_valid_root_in_the_ring_for_raising_the_floor`（`crates/singlefs-core/src/mount.rs:4491`） | 不读盘 | 它按公式把根归到槽「target_for_publish(root.checkpoint_txg, slots_per_region))」（`crates/singlefs-core/src/mount.rs:4506`），本来就假设根住公式槽 | 与新读法一致，是改前不自洽的一处 |
| 管理员回退判候选：`crates/singlefs-core/src/mount.rs:8412` → `readable_roots_rereading_ring_slots_known_to_hold_a_root_once`（`crates/singlefs-core/src/recovery.rs:1770`） | 前者 | 同抬 F | 同上 |
| 写系统配置槽前算 F_生效：`crates/singlefs-core/src/transaction.rs:588` → `crates/singlefs-core/src/recovery.rs:2046` → `crates/singlefs-core/src/recovery.rs:1798` | 前者 | 错位 = 自证不过，不重读，当没有根 | 与现行读坏自证不过同 |
| 取号见证读的实例代号最大值：`crates/singlefs-core/src/transaction.rs:935` → `highest_root_instance`（`crates/singlefs-core/src/recovery.rs:4198`） | 前者 | 错位根的实例代号不再进 max | 与「根读不出」同类的既有洞，不新增类 |
| 新实例第一次发布的 txg 与树 ID 水位：`crates/singlefs-core/src/mount.rs:1347`、`crates/singlefs-core/src/mount.rs:1379` | 前者 | 错位根不再抬 max | 同上；记录那一半的 max 仍在 |
| F2 退回候选：`crates/singlefs-core/src/recovery.rs:6977` `readable_roots` | 前者 | 错位根不再算「读得出的根」，落进「根读不出」那一类 | 退回条件只会变严（更不退） |
| 落后支缺根：`root_of_the_version_not_in_its_ring_slot_on_the_device`（`crates/singlefs-core/src/mount.rs:5873`，补丁后行号后移） | 前者 | 原先错位根走 `AnotherRoot`，改后走 `Bad`；两者都是「不是它」 | 结局不变，只有成员名变 |
| 实例表清扫：`every_root_of_the_ring_self_verified`（`instance_table_sweep.rs:457`） | 自己解析 | 加同一核；错位 → `ARootRingSlotIsNotSelfVerifiedAfterOneReread` → 这一轮不删行 | 更保守 |
| 挂载透明取字节：`root_record_bytes_of_the_version`（`crates/singlefs-core/src/recovery.rs:4158`） | 自己解析 | 只比字节，不必改；要同口径就加同一核 | 不变 |
| 补丁新增：`rings_of_the_device_against_the_selected_version`（`crates.patch` 第 303 行 起，在 `mount.rs`，补丁后） | 前者 | 闸三「盘上有比所选那一版新的根就不补写」看不见错位读出的新根；读错照旧计 | 闸三少拒一格；在补丁后落地时要和 X1 一起核 |
| 补丁新增：α' 离线出路 `root_of_the_asserted_version_is_readable`（`crates.patch` 第 1280 行 起，`system_configuration_repair.rs`，补丁后） | 前者 | 已按 txg 与实例比对，错位根不会冒充 | 不变 |

### X1.4 按自证不过处置之后，有没有一处变坏

改动的本质：把「读出一条自证得过、但不住这一槽的根」从「收下」改成「读坏」。所以每一处的新结局都落在这一处对「读坏」本来就有的处置里，没有新增分支。逐类看会不会丢已确认的写或挂不上本来挂得上的池：

- 丢已确认的写：要丢，得是错位读成了某条已确认根的唯一证据。已确认的根 R 是 FUA 写在它自己的公式槽里的（`crates/singlefs-core/src/transaction.rs:508` 取目标，`crates/singlefs-core/src/transaction.rs:1533` 写），读它自己的槽就读得到；错位读与「它自己的槽读不出」要同时发生才会让错位读成唯一证据，那是两个故障，在单故障加一次断电之外。反过来，错位读把某一槽自己的内容盖住时，改前改后这一遍都看不到那一槽的真内容；差别只在改后那一槽被标成读坏，钉住回收门槛、重读、必要时拒抬 F，改前这一槽被记成别的根，门槛会漏钉。所以改后只会更保守，不会更少保护。
- 挂不上本来挂得上的池：新增的拒绝只有两条路径，`NoValidRoot`（`crates/singlefs-core/src/recovery.rs:7054` 那一支）与 `NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread`，条件是环里全部自证过的根都不住自己的槽。合法写者写出来的根都住自己的槽（见 X1.5），所以这两条在合法盘面上走不到；走得到的只有手摆根的测试盘面和写者缺陷造出的盘面。
- 抬 F 与回退的「知道住着根的槽重读仍坏就拒」：错位读持续不退时，改前收下一条错 txg 的根去算上限（上限可以算错，抬过头），改后拒；这是该拒的地方。
- 实例表清扫：保守一格（这一轮不删行）。

### X1.5 落点公式会不会把合法根判成错位

核的函数（写进 `crates/singlefs-core/src/root_ring.rs`，挨着 `target_for_publish`「pub fn target_for_publish(」（`crates/singlefs-core/src/root_ring.rs:109`））：

```
fn root_lives_in_the_slot_its_txg_names(txg, ring_slot, slots_per_region) -> bool
    txg == 0 ⇒ ring_slot.slot == 0           // 第 0 代，不核区域
    否则     ⇒ target_for_publish(txg, slots_per_region) == ring_slot
```

- 写根的路径只有两处：发布路径（「let target = target_for_publish(」，`crates/singlefs-core/src/transaction.rs:508`，唯一的 `WriteRootRecordForceUnitAccess` 落盘点）与 mkfs 的第 0 代（「for ring_slot in genesis_root_ring_slots() {」，`crates/singlefs-core/src/make_filesystem.rs:621`，槽集合 = 三个区域的槽 0，`make_filesystem.rs:676`）。补丁不新增写根的路径（补丁只写系统配置槽）。
- 第 0 代：三份同字节，落在 (区域 0、1、2 的槽 0)；核放行槽 0 任一区域，与 I-7.17 同口径「if txg == GENESIS_ROOT_CHECKPOINT_TXG {」（`crates/singlefs-checker/src/image.rs:1272`）。一旦 txg 1、2 写下去，区域 1、2 的槽 0 被盖，区域 0 的槽 0 要到 txg = 3S 才被盖；这几份都放行，不会被判错位。
- 根环洞：`target_for_publish` 是纯函数，对任何 txg 都给槽，洞里留着的上一圈旧根（崩在记录之后、根槽之前那一形，D16 已定项 1 那一行的「环里最旧有效根」所指）是当年同一公式写下的，它的 txg 与它所在的槽对得上；判错位看的是位置，不是新旧。
- 跨区域：区域号 = txg mod 3，读槽取盘用同一张区域归属表「let device = region_devices[usize::try_from(ring_slot.region).expect("区域号")];」（`crates/singlefs-core/src/recovery.rs:1563`），写读同表。
- 回绕：txg 与 txg + 3S 落同一槽，公式看不出，也不该看出（X1.2 的 (d)）。
- S 与 R：S 住系统不可变配置的 `sizes`：「let slots_per_region = system_configuration」（`crates/singlefs-core/src/mount.rs:2973`），往下三行是 `.immutable.sizes.root_ring_slots_per_region`；不可变那一档 mkfs 后不可改（`.claude/kb/decisions/22-单元原子性怎么合成.md@已定项 26：系统配置的三档可改性`）。R = 3 是第一版常量：「pub const ROOT_RING_REGIONS: u64 = 3;」（`crates/singlefs-format/src/lib.rs:286`）。公式用的 S 与写者用的是同一个字段。
- 回退、抬 F、卸载、暖机、写行都走同一个发布路径写根，txg 逐个加一或取 max + 1，没有第二套落点。

结论：在第一版的格式约束下，合法写者写出来的根没有一条会被这一核判成错位。**这一条结论里最脆弱的一处**是下面这对冲突：

- `.claude/kb/decisions/22-单元原子性怎么合成.md:140`：「择新在 checkpoint_txg 平局时按它高者赢（设备失而复得会造出两条同 txg 的合法根」
- `.claude/kb/decisions/22-单元原子性怎么合成.md:335`：「区域内槽位 = `(txg div R) mod S`，无状态、只由 txg 与 (R, S) 决定」

同一个 txg 只有一个公式槽（第 0 代除外），区域又固定住在一块盘上，两条同 txg 的根只能争同一个槽，不可能并存在两个槽里；池级 checker 的 I-7.17 本来就把「同 txg 的根住到另一个槽」判红。C88 登记行（`.claude/kb/checks-owed.md:80`）写的「设备失而复得会让根环里并存两条同 checkpoint_txg 的自证合法根」我没找到走得到的盘面。入库的两个择根平局用例就是手摆这一形：`crates/singlefs-harness/tests/root_choice_on_a_tied_checkpoint_txg_takes_the_higher_instance_in_a_later_slot.rs`（把同 txg 的两条根放进「读根环次序里 mkfs 根之后第一个与最后一个空槽」）与 `.../common_images_for_the_checker_tier/root_choice_through_the_glue_puts_the_checkpoint_txg_before_the_instance.rs`。加了公式核之后，这两个用例里至少一条根被判错位，用例要改成把两条根各放在合法槽里（同 txg 做不到，只能改成不同 txg 的择新用例），或者条款明说「平局只在第 0 代的三份副本之间有意义」。这是规则没说的一格：D22 已定项 7 那一句与公式核谁让步，要主 agent 或用户定。

### X1.6 池级 checker：读错位要不要也进

不进。理由：checker 读的是一张静态镜像，镜像里没有「这一次读回了别的槽」这回事——I-7.9 那一行已经写了同一个前提「推的：读故障不留在盘上」（`crates/singlefs-checker/invariants/02-根环.md:16`）。镜像里留得下的是错位写，I-7.17 已判（`crates/singlefs-checker/src/image.rs:1265`，GPU 判器里是 `crash_judge_checker.wgsl:5061`）。要对齐的一处在 X5（checker 重算「环里最旧有效根」时按同一核滤根）。恢复一路的 GPU 判官（`crash_judge_recovery.wgsl`）里的 `choose_root` 要不要也加这一核：合法写者造出来的崩溃状态里没有错位根，不加也与 CPU 一致；但 CPU/GPU 对拍用例若造了手摆根的盘面，两边要同改。

### X1.7 代价

- core 改动：`root_ring.rs` 加 1 个函数；`crates/singlefs-core/src/recovery.rs:1568` 一处、`crates/singlefs-core/src/mount.rs:2737` 一处、`instance_table_sweep.rs:481` 一处各加一核（共 3 处读解析点，加上 `crates/singlefs-core/src/recovery.rs:4172` 那一处可选）。调用方不用改签名：两个读函数都拿得到 `immutable_sizes.root_ring_slots_per_region` 与 `ring_slot`。
- 要不要给 `BadRootRingSlotReading` 加第三个成员（读得出、自证过但不住这一槽）：加了便于诊断，代价是每个穷举 match 都要补一臂（仓里禁通配臂），我没数全；按「按自证不过处置」的字面，先复用 `NotSelfVerified`，不加成员。
- 测试：harness 要一个新的故障注入「读这一槽回另一槽的字节」（现有的 `RangesReadingBackAllZero` 只造全零）；一条用例照 C743 登记行造「一遍错位读、一遍读坏的重建」，断言新类不记那条根；变异行每个解析点一行。手摆根的旧用例：已核实会红的 3 个（`core_review_tree_table_duplicates_and_slot_one_search.rs` 把 txg 取 u64::MAX 的根摆在「let ring_slot = RootRingSlot { region: 0, slot: 7 };」（`crates/singlefs-harness/tests/core_review_tree_table_duplicates_and_slot_one_search.rs:479`），u64::MAX 的公式槽是区域 0 槽 5；上面那对择根平局用例）；同时序列化根又算根槽偏移的 harness 文件共 22 个（含这 3 个；命令在「要主 agent 跑的命令」一节），其余 19 个我没逐个核，是候选。
- 对拍：E165 的读槽对拍「let reading = read_root_ring_slot(」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:11086`）拿 `read_root_ring_slot` 的读数比生成时的真相；加核之后这一对拍要重跑一次确认没有「读数与真相不同」（停机条款 TH4），重型，没跑。
- 估：核心加核一小时量级，测试与变异两三小时（推的，没量）。

### X1.8 什么现象会推翻 X1 的结论

- 找到一条合法写者写出来、却不住 `target_for_publish` 给的槽（第 0 代除外）的根：公式核会把它判错位，「合法根被误判」成立。
- 找到一个盘面，两条同 txg 的根并存在不同的槽里而且都是合法写者写的（C88 的前提）：同上。
- E165 读槽对拍加核之后出现「读数与真相不同」的槽。

## X2　C751 只读挂载在那一形照报失败

### X2.1 现状

F2 判法在所选根的实例表读不出时的取法（`crates/singlefs-core/src/recovery.rs:6974` 读表，`:6996` 取法）：「None => order == witnessed_publish,」（`crates/singlefs-core/src/recovery.rs:6996`）——表读不出就滤不了，只认与见证到的那一版恰好相等的那条，（见证到的那一版, 所选根）之间别的读得出的根一概丢掉。C751 的形：R 的实例表读不出，见证到 Pm，P 读得出而被丢，只读挂载打开 Pm 时不读实例表，所以「只读挂载打开 Pm（打开那一步不读实例表），读得出的 P 被跳过」（`.claude/kb/checks-owed.md:503`）；恢复与可写挂载在 Pm 上走读与建上一版都要读实例表，照报失败。

### X2.2 条文：「退到的那一版之上还有读得出、没法按实例表判抛弃与否的根」怎么写成当下判得出的条件

这句话里每个量，F2 判法函数走到 `:6977` 时都已经在内存里，不用多读一个字节：

- 「所选根的实例表读不出」= `instance_table_of_the_chosen_root` 是 `None`（`crates/singlefs-core/src/recovery.rs:6974`）；
- 「读得出的根」= `readable_roots_in_the_ring`（`crates/singlefs-core/src/recovery.rs:6977`）里的根；
- 「在退到的那一版之上」= 择根序 `(checkpoint_txg, 实例代号)` 严格大于 `witnessed_publish`，且小于所选根的序。恰好等于 `witnessed_publish` 的那条不算（它是退回的目标）。

条件：`instance_table_of_the_chosen_root` 为 `None`，并且 `readable_roots_in_the_ring` 里有一条根使 `witnessed_publish < order < chosen`。成立就整个判法交回 `None`（不退，照报所选那一版的失败）。

D23 已定项 14 的 F2 段（`.claude/kb/decisions/23-journal的角色与格式.md:398`）在「所选根的实例表读不出时只认与见证到的那一版恰好相等、读得出的那条根」之后加：

> 所选根的实例表读不出、而（见证到的那一版, 所选根）之间还有读得出的根（没法按实例表判它被不被抛弃）时不退，照报所选那一版的失败：退过去会越过一条可能已确认的发布。

「退回见证到的那一版的代价」那一条（`.claude/kb/decisions/23-journal的角色与格式.md:423`）不用改；C751 登记行（`.claude/kb/checks-owed.md:503`）随实现销账。

### X2.3 与入库用例 `:656` 怎么共存

用例 `a_row_publish_whose_units_and_instance_table_read_back_zero_falls_back_only_to_the_exactly_witnessed_version`（`crates/singlefs-harness/tests/recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed.rs:656`）的盘面算这一形：它的辅助函数注释写得明白，「[P, Row) 里有 P 与 M1 按实例表抛弃了的 R，R 更新」（同文件 `:585`），Row 的实例表读回全零，所以 R 读得出而判不了抛弃。条件成立。

| 断言（同文件） | 今天 | 条文落地后 |
|---|---|---|
| 恢复（`:665`–`:673`） | 退到 P，读回 P 的内容 | 取决于改一处还是改共用判法，见下 |
| 只读挂载「let read_only = mount_read_only(&devices).expect("只读挂载退回 P、打得开");」（`:674`） | 打开 P | `Err`：断言翻成 `expect_err`，并断言不是 `Ok`（这是 C751 要的） |
| 可写挂载（`:683`–`:701`） | 在取号之前拒（最新那条根的实例表读不出），盘上不变 | 不变 |

两种改法：

- 甲（只改只读挂载的调用点：`mounted_read.rs:657` 前后自己算上面那个条件）：恢复仍退到 P，只读挂载报失败。**新造出一处恢复与只读挂载的分叉**，而且和 C755 的「三处看同一种失败」方向相反。
- 乙（条件写进共用判法 `crates/singlefs-core/src/recovery.rs:6893` 那个函数）：`:656` 的恢复断言也翻成「照报 Row 读不出」，三处一致。代价：恢复在这一形上从「读回正确的 P」变成「报失败」。这一形要 Row 的单元与 Row 的实例表两处读回全零、再加崩在见证之前才走到，已经在单故障承诺之外（C751 登记行对它自己那一形写的是「三样故障同时，在单故障承诺之外」，`.claude/kb/checks-owed.md:503`，同一个量级），而「没法判 R 抛不抛弃」正是条件的字面。

我的读法：乙。理由是用户同一批选的 C755「连触发一起管」要三处同一套逻辑，甲违反它；用户选 C751 时问句只点了只读挂载，因为 C751 登记行的盘面（R 没重写实例表，R、P、Pm 共用一片读回全零的实例表）上恢复与可写挂载本来就报失败，甲乙在那一形上结局相同，差别只出在 `:656` 这类 R 重写过实例表的形上。**这一格规则没说**：甲还是乙，要主 agent 或用户定；下面按乙写改动量，甲更小（约 6 行）。

### X2.4 改了之后，F2 许可的其余格上结局变没变

F2 判法函数的出口逐个对（`crates/singlefs-core/src/recovery.rs:6893` 起）：

| 格 | 今天 | 改后 |
|---|---|---|
| 所选版本是施加记录重建的（不是所选根） | 不退 | 不变 |
| 系统配置见证判不出（R2） | 不退 | 不变 |
| c_见证 = 0 | 不退 | 不变 |
| 所选那一版见证过 | 不退 | 不变 |
| 计数器 = c_见证 的记录读不出 | 不退 | 不变 |
| 实例表读得出 | 按表滤，退到区间里最新的 | 不变 |
| 实例表读不出，区间里没有别的读得出的根 | 只认恰好等于见证到的那条 | 不变 |
| **实例表读不出，区间里有别的读得出的根** | 退到恰好等于见证到的那条（若有） | **不退** |
| 末条读得出、根读不出、不被抛弃的发布挡着（`:7003` 起） | 不退 | 不变 |

只有加粗那一格变。入库里落在这一格的用例我只核实了 `:656` 一个；C751 登记行里点名的攻方盘面（`OPUS F2 kind=ow k=5 ...` 那四组）在攻方模型上，没有入库的用例。

### X2.5 代价

- core：F2 判法函数里加约 8 行（`:6988` 之前）；甲约 6 行。
- GPU 恢复判官有同一份判法的 WGSL 拷贝「fn newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to(」（`crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl:138`），乙要同改，约 10 行；它与 CPU 判法的对拍用例 `crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 在 checker 档，没跑。
- 变异表：锚在 `None => order == witnessed_publish,` 上的行（`crates/mutations.tsv:2931` 那一行）要核锚点是否仍命中一次；新加一行变异「实例表读不出时不看区间里读得出的根」，必须由改后的 `:656` 用例抓住。
- 用例：改 `:656` 一个断言（乙再改恢复那一个）；新加一条 C751 盘面（R 没重写实例表）的用例钉「三处都报失败」。

### X2.6 什么现象会推翻

- 找到一个 F2 许可的盘面，条件成立而退回是对的且不可替代（读回的是唯一能读的已确认版本）：那说明条件把该退的也挡了。`:656` 的盘面就是这样一个（P 是对的），它靠三样同时坏才走到，我把它记成已知代价。
- 别的入库用例落在加粗那一格而我没核到：改后会红。

## X3　C755 三处连触发一起管

### X3.1 「建得出」的并集：逐类列，三处今天各读到哪里

三处的失败触发今天各是一个：恢复看走读失败（「// 走读失败才再扫一遍环拿记录给退回的判法」，`crates/singlefs-core/src/recovery.rs:7115`，调用 `walk_to_file_judging_the_truncation_fields_as`，`crates/singlefs-core/src/recovery.rs:6255`）；只读挂载看打开失败（「Err(failure_of_the_selected_version) => {」，`crates/singlefs-core/src/mounted_read.rs:655`，调用 `open_pool_for_read_judging_the_truncation_fields_as`，`mounted_read.rs:375`）；可写挂载看建上一版失败（「match rebuild_previous_version(」，`crates/singlefs-core/src/mount.rs:8000`，函数在 `crates/singlefs-core/src/mount.rs:1264`，里面调 `rebuild_version_judging_the_truncation_fields_as`，`crates/singlefs-core/src/recovery.rs:3025`，再加 `instance_table_chain_of_root`，`crates/singlefs-core/src/mount.rs:1281`）。逐类（读码，行号是各函数体里读该类的地方；打勾 = 会读且读不出算失败）：

| 单元类 | 走读（恢复） | 打开（只读） | 建上一版（可写） |
|---|---|---|---|
| 实例表第 0 片 | 勾（`crates/singlefs-core/src/recovery.rs:6255` 函数体开头） | 不读 | 勾（`crates/singlefs-core/src/recovery.rs:3053` 起） |
| 实例表后续片（沿链） | 勾「instance_table_chain_of_root(reader, root)?;」（`crates/singlefs-core/src/recovery.rs:6290`） | 不读 | 勾（`crates/singlefs-core/src/mount.rs:1281`） |
| 树表单元 | 勾 | 勾（`mounted_read.rs:434`） | 勾 |
| 中央映射树 | 首次用到才读（`CentralMappingTreeReadOnFirstUse`，`crates/singlefs-core/src/recovery.rs:5554`） | 打开时整棵读「let mapping_tree = read_code_two_tree(」（`mounted_read.rs:407`） | 整棵（`crates/singlefs-core/src/recovery.rs:3621`） |
| inode 树根与每片叶容器 | 勾 | 勾（`mounted_read.rs:479` 起） | 勾（`crates/singlefs-core/src/recovery.rs:3206` 起） |
| extent 树 | 勾 | 打开时不读，按需（`mounted_read.rs:474` 的注释） | 勾 |
| 分配记录树 | 勾 | 不读 | 勾 |
| 记账树 | 勾 | 不读 | 勾（`crates/singlefs-core/src/recovery.rs:3540`） |
| deadlist 树 | 勾 | 不读 | 勾（`crates/singlefs-core/src/recovery.rs:3592`） |
| 墓碑、谱系、retirement 三棵码 2 树 | 只读索引节点，不读容器 | 不读 | 读索引节点与容器（`crates/singlefs-core/src/recovery.rs:3437` 起） |
| 其余树根（deleted_inodes、logged_operations、livelist、稀疏旁表） | 通用分支读根（`crates/singlefs-core/src/recovery.rs:6455` 起的 `read_mapped_tree_root`） | 不读 | deleted_inodes、logged_operations 读根（`crates/singlefs-core/src/recovery.rs:3360`、`:3402`）；livelist、稀疏旁表我没在 `crates/singlefs-core/src/recovery.rs:3042`–`:3700` 里找到读根的调用，未核 |
| 第一个文件的数据单元 | 勾，读不出就失败 | 打开时不读，`open_file` 才读 | 读，提示与映射都读不出**不算失败**（D19 已定项 5 的 N2，`crates/singlefs-core/src/recovery.rs:3173`） |
| 其他文件的数据单元 | 不读 | 不读 | 读，同样读不出不算失败（`crates/singlefs-core/src/recovery.rs:3189`） |

并集 = 这张表里任一格读不出就算「所选那一版建不出来」。其中数据单元一行两边的口径不同（走读严、建上一版宽）；并集取严的那一半，即数据单元读不出算建不出来。**这一格规则没说**：N2 本来是为了不因一个坏数据单元拒挂载，并集取严会让 N2 在「系统配置没见证」这一支里让位（见 X3.3 的例一）。

### X3.2 统一之后怎么走（改动点）

1. 新函数（我建议放在新模块里，因为 `mounted_read.rs` 引 `recovery.rs`，把依赖两边的函数放进 `recovery.rs` 会反过来引 `mounted_read.rs`）：对一条根，依次跑走读、打开（`TreeTableHasNoEntries` 不算失败，与 `mounted_read.rs:648` 同）、建上一版加沿链读实例表，返回第一个失败或通过。约 80 行。
2. F2 判法函数（`crates/singlefs-core/src/recovery.rs:6893`）拆成两半：前半（所选那一版见证过没有）（`crates/singlefs-core/src/recovery.rs:6902`–`:6968`，只读系统配置槽与已扫的记录，不读单元）；后半（退到哪一版）（`:6970` 起，含 X2 的条件）。前半在三处都先算。
3. 三处的流程统一成：先做自己本来要做的读（走读 / 打开 / 建上一版）；然后若所选那一版是择到的根本身（没施加记录）且前半说没见证，跑新函数；新函数报失败才进后半；退到的那一版也要过新函数，不过照报所选那一版的失败。见证过的，什么都不多读。
   - 恢复：`crates/singlefs-core/src/recovery.rs:7100` 起的 `match` 改成上面的流程，原来 `Err(failure)` 那一支（`:7116`）并进去；
   - 只读挂载：`mounted_read.rs:640`–`:682`，打开成功那一支也要过前半；
   - 可写挂载：`crates/singlefs-core/src/mount.rs:7999`–`:8072`（补丁后行号后移，按函数 `mount_writable_with_every_test_only_switch` 认），建上一版成功那一支也要过前半。
4. 三处给后半的记录都在触发之后重扫环，见 X3.5。
5. 常态路径成本：见证过的（常态）只多算前半，前半读每块盘两个系统配置槽各一次（4 KiB），池里两块盘合计 4 次读，不读单元；没见证的（崩在根槽之后、轮换之前那一窗）才跑新函数，那时只读挂载要多读实例表、extent、分配记录、记账、deadlist 各树和所有数据单元，量级等于今天一次可写挂载的建上一版（推的，没量）。

### X3.3 统一之后，同一份盘面三处落同一版

落点由三样决定：所选根、见证与否、两个建得出判定；统一之后三处对同一份盘面这三样的输入相同（同一读者、重扫出的同一份记录），落点相同。登记行里的两个例子：

- 例一（k = 5，R 没见证，R 的记录读得出，R 只有一个数据单元两份都读回全零）：恢复 今天 P，改后 P；只读挂载 今天 R，改后新函数走读那一项失败，退到 P；可写挂载 今天 R（N2 容忍），改后同样退到 P。
- 例二（R 的分配记录树根读不出）：恢复、可写挂载 今天 P；只读挂载 今天 R（打开不读分配记录树），改后 P。

三处统一的是**落到哪一版**，不是**落下之后成不成**：见证过的 R 数据单元读坏时，恢复仍报走读失败，只读挂载仍打得开 R，可写挂载仍建在 R 上（N2）；可写挂载退到 T 之后还要过 F1 重判（`F1_AFTER_F2`，补丁后落地），可能在 T 上拒。要不要把这一层也统一，**规则没说**；统一的代价是只读挂载要拒今天打得开的池（见证过、R 的实例表读不出而其余可读），与「不挂不上本来挂得上的池」冲突，我的读法是不统一，把 D23 已定项 14 F2 段开头的「三处用同一套逻辑」（`.claude/kb/decisions/23-journal的角色与格式.md:398`）改写成：

> 恢复、可写挂载读阶段与只读挂载择根三处用同一套触发与同一套判法：所选那一版建不出来（走读、打开、建上一版三样读的单元的并集有一个读不出、解不开、判不过）而系统配置没见证它时，退到 [见证到的那一版, 所选根) 里……（判法原文不变）；退到的那一版也要建得出，否则照报所选那一版的失败。三处给判法的记录都在触发之后重扫 journal 环。不退时各处照自己读到的报。

### X3.4 恢复多读的几类单元，在不注读故障的崩溃态上会不会让恢复从「落所选根」变成「退回」

恢复今天的走读比并集少读的只有三类：建上一版读的墓碑、谱系、retirement 容器，其他文件的数据单元，以及打开读的整棵中央映射树（走读只在位置提示读不出时才读它）。结论：不会。

- 持久顺序：一次发布先写单元、屏障、再写 journal 记录、屏障、再写根槽「这次重写的单元 → 屏障 → journal 记录 → 屏障 →」（`crates/singlefs-core/src/transaction.rs:1340`），所以根槽上有的根，它的单元都已持久，无读故障时这几类单元读得出。
- 别人量的（别的会话 2026-10-20 的调查员，我没复跑）：`~/.local/share/singlefs/work/m3-close-w8-c758-c656-measure/report.md`（sha256 c2f4ac242d70ca1c6ed960c3d070a76fd3a113b3120f866f568c14b96ed89a1f）第 5 行：七条写流、两档展开共 137 902 个不注读故障的崩溃态，恢复、只读挂载、可写挂载三处各自的读都没失败（三处各 137 902 次 F2 没被调到）。三处各自的读成功，并集就成功。这一条证明力有限：态里不含「读故障」，也不含手摆的坏数据单元。
- 复核不了：我没法在不跑重型的前提下复现那份量数。

### X3.5 与 C754「重扫」一致

今天：恢复在走读失败那一刻重扫环「&scan_journal(reader, &system_configuration),」（`crates/singlefs-core/src/recovery.rs:7124`）；只读挂载把挂载开头扫的那一份传给判法（`mounted_read.rs:628` 扫，`:662` 传）；可写挂载传读阶段的 `records`（`crates/singlefs-core/src/mount.rs:8023`），补丁后又在退回之后、F1 重判之前重扫一次（补丁 `crates.patch` 第 652 行 起：`records_rescanned_before_rejudging`）。统一：三处都在新函数报失败、进后半之前重扫一次环，把重扫的一份给后半；可写挂载补丁里那一次 F1 重判前的重扫保留（用户 2026-10-20 定「重扫」的原话范围是那一次，records/2026-10-18-里程碑三收尾与近期欠账.md 第 323 行）。多出来的成本：退回路径上多扫一次 journal 环（环长 = 系统配置里 `journal_ring_bytes`），只在没见证且建不出来时发生。

### X3.6 崩溃点重放 oracle 与 GPU 判官要不要改

- oracle：崩溃点重放调的是 core 的 `recover`（例：`crates/singlefs-checker-tier/src/crash_injection.rs:2705`），统一之后自动跟着走，不用改。
- GPU 判官：`crash_judge_recovery.wgsl` 的 `recover` 把走读跑两遍、中间夹 F2 判法「fn recover(consult: bool, cfg_choice: SysChoice) -> Recovery {」（`crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl:11`），触发只看 `walk_to_file` 失败。统一后触发变并集。无读故障的崩溃态上并集 = 走读（X3.4），GPU 不改也与 CPU 一致；要让两边在所有态（含将来注读故障的态）上都一致，GPU 要把打开与建上一版多读的几类（容器树、其他文件数据单元）也移植，量级 200 行以上 WGSL（推的）。我的读法：先不移植，把「GPU 不判并集」写进对拍用例的射程句，由 X3.4 的前提（无读故障）撑着；将来给崩溃枚举注读故障那天再移植。这一格规则没说，要主 agent 定。
- 挂载透明判官只判不带读故障的态（C755 登记行写的），不受影响。

### X3.7 代价与推翻

- 代价：core 新函数约 80 行加三处流程改写约 120 行；harness 新用例一条（登记行写的「一条 harness 用例在那份盘面上各跑三处，断言落同一版」，两个例子各一，k = 3 与 5 各一）；变异行每个触发一行；已有用例里「R 数据单元读坏而可写/只读仍落 R」的断言（如果有）要改，我没逐个核。
- 推翻：不注读故障的崩溃态里出现并集失败而走读成功的态（X3.4 的前提不成立）；或者「见证过的 R 在并集上失败」的盘面被证明常见到值得统一成败。

## X4　C773 回收门槛落盘

### X4.1 字段的宽度与落点

- 宽 8 字节，小端无符号，值 = 这条根指着的那一版记账（已分配统计与分配记录树）所按的回收门槛（D16 已定项 1 的 max(F_生效, 环里最旧有效根)，读坏与读回全零的槽钉在可能住过的最旧 txg 的那几个钉也算在里面）。类型沿用 `CheckpointTxg`。
- 根记录：放在分配记录树根指针之后、算法类型之前（偏移 428 起 8 字节）。新字节表：算法类型 436、nonce 437、MAC 449、合计 465。理由：偏移 428 之前的每个字段（magic 0、fsid 4、flags 20、实例代号 24、checkpoint_txg 28、树表指针 36、树 ID 水位 122、F 130、自证校验和 138、实例表指针 170、映射根指针 256、分配记录树根指针 342）一个都不动，所以 `ROOT_CHECKSUM_OFFSET`「pub const ROOT_CHECKSUM_OFFSET: usize = 4 + 16 + 4 + 4 + 8 + 86 + 8 + 8;」（`crates/singlefs-core/src/root_record.rs:13`）、checker 里 `record[342..428]` 那类切片（`crates/singlefs-checker/src/walk.rs:2020`）、GPU 的 `rd64(base, 130u)`（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:1195`）都不变；动的只有末尾 29 字节那一段（`crates/singlefs-checker/src/image.rs:1671` 的 `428..457`、`crash_judge_checker.wgsl:5078` 的 `ROOT_RECORD_ALGORITHM_NONCE_AND_MAC_OFFSET`；I-2.4 判这一段全零，范围不跟着挪就会把新字段判成「留位非零」）。对照：放在 F 之后（242 到 250 那次的做法）要挪校验和和后面三条指针与末尾 29 字节，checker 与 GPU 判器里读这些偏移的地方都得改（处数我没数）。
- 槽宽：465 ≤ 512，合法 `physical_block_size` 最小 512，余量 55 到 47；mkfs 的拒绝「if u64::from(geometry.physical_block_size) < ROOT_RECORD_BYTES {」（`crates/singlefs-core/src/make_filesystem.rs:361`）、checker 的 `image.rs:726`、GPU 的 `crash_judge_common.wgsl:936` 都读同一个常量，跟着走；D22 已定项 16 里写的「`physical_block_size` ∈ [根记录宽 457, 槽距]」（`.claude/kb/decisions/22-单元原子性怎么合成.md:341`）改 465。
- journal 新根段：末尾（卸载记号之后）加 8 字节，364 到 372；记录头 487 到 495（新根段 99、fsid 471、MAC 479、头末 495）；512 扇区内余 25 到 17；点名项 ⌊(4096 − 495) ÷ 56⌋ = 64 不变。反向链的 `previous_hash` 罩整个记录头，自动罩住新字节。为什么记录也要带：(一) 由记录施加出来的那一版的根在内存里由记录拼（`crates/singlefs-core/src/recovery.rs:4624`、`:4648` 两处，「施加一条记录 = 把所选根的这七个字段换成记录里的」，`.claude/kb/decisions/23-journal的角色与格式.md:465`，七改八）；(二) I-8.10 逐字节比记录新根段与同代的根（`crates/singlefs-checker/src/walk.rs:9877`，常量表 `walk.rs:9587`，七段改八段）；(三) 记录施加出来的最新一版，checker 判 I-3.1 要它的门槛，根槽从没落盘就只能从记录取。
- 写盘次数与字节数：根槽写的是整个 `physical_block_size`，journal 记录写的是整个 4096，两处写调用的长度都不变，**每次发布多写 0 次、0 字节**；崩溃状态数（按写调用与段数枚举）不变。

### X4.2 什么时候取值（条文要写死的采样点）

- 带单元的发布：取完落点之后、装记账行之前，分配器记这条根的那一步「pub fn record_root_written_by_this_process(」（`crates/singlefs-core/src/allocator.rs:2591`）里算的 `reclaim_floor`「let reclaim_floor = crate::mount::reclaim_floor(」（`allocator.rs:2610`）就是这次发布回收用的门槛；让这个函数（以及包它的 `crates/singlefs-core/src/mount.rs:1438` 那一层）把它交回来，装根时写进去。调用点：`crates/singlefs-core/src/transaction.rs:1997`、`:12442`，取号之前的预演拷贝（`crates/singlefs-core/src/mount.rs:5229`、`:5285`）走同一段，所以预演与真发取值相同。
- 零单元发布（树表 0 条的一版上的暖机与写行，`crates/singlefs-core/src/mount.rs:5125` 起）：根是「根记录照上一版的根、只换 checkpoint_txg、实例代号、回退下界」（`crates/singlefs-core/src/transaction.rs:1545`；同一行还说与树 ID 水位也换），记账没变，所以字段照抄上一条根的，不取此刻分配器的值；此刻分配器在根落盘之后才回收（`crates/singlefs-core/src/mount.rs:5135`），那一截回收要到下一次带单元的发布才进记账。**这是条文必须写明的一格**：字段说的是「这条根指着的那一版记账所按的门槛」，不是「这条根落盘那一刻分配器的门槛」。
- mkfs 的第 0 代：0。
- 由记录施加出来的版本：取那条记录新根段里的。

根记录的构造点（非测试）：`make_filesystem.rs:581`、`crates/singlefs-core/src/transaction.rs:1604`、`:2131`、`:13843`，施加记录的两处 `crates/singlefs-core/src/recovery.rs:4624`、`:4648`，共 6 处；字段是必填的，漏填编译不过。

### X4.3 条文

D16 已定项 1：
- 「环里最旧有效根」那一行（`.claude/kb/decisions/16-发布语义.md:36`）「同取这个值的六处」改写：分配器的回收门槛、checker 的候选过滤、影子账的候选过滤、回退候选集、抬 F 的上限，五处同取 max(F_生效, 环里最旧有效根)；checker 的 I-3.1 丁-defer 门槛退出这一句，改按盘上门槛（下面）。
- 「根槽这一次读坏」那一行（`.claude/kb/decisions/16-发布语义.md:39`）末尾「读坏的槽与读回全零的槽」那一句后加：「钉着的门槛落进根记录：每条根带『这次发布用的回收门槛』8 字节，值是这条根指着的那一版记账所按的门槛，钉着时就是钉着的那个值；零单元发布照抄上一条根的；由记录施加出来的版本取记录新根段里的。」并把 I-3.1 那一行里「钉最旧 txg 的代价（用户认的）」整段删去（`crates/singlefs-checker/invariants/05-空间记账.md:8`，现文「⚠️ **钉最旧 txg 的代价（用户认的）**」）。

D22 已定项 7 字段表（`.claude/kb/decisions/22-单元原子性怎么合成.md:148` 与 `:149` 之间）加一行：

> | **回收门槛** | **8** | D16 已定项 1：这条根指着的那一版记账所按的回收门槛 max(F_生效, 环里最旧有效根)，钉着的槽钉在可能住过的最旧 txg；零单元发布照抄上一条根；mkfs 第 0 代为 0 |

字节表那一句（`:153`）改成「…分配记录树根指针 342 / 回收门槛 428 / 算法类型 436 / nonce 437 / MAC 449」，合计句（`:156`）改「合计 **465 字节**，512 槽内余量 47」，`format-const: ROOT_RECORD_BYTES = 465` 带上旧值 457 进 stale 列表；英、日两份同改。

D23 已定项 4（头 487 到 495、新根段 364 到 372、fsid 463 到 471、MAC 471 到 479，`.claude/kb/decisions/23-journal的角色与格式.md:152` 起）与已定项 15（新根段加第八个字段；「七个字段」改「八个字段」，`:465`）。

I-3.1（`crates/singlefs-checker/invariants/05-空间记账.md:8`）把现在的「F_生效 之下仍在 defer 的槽」一段（丁-defer 选根式）换成：

> 已分配统计（最新根的记账）逐盘等于「遍历得到的和」并上「释放代大于盘上门槛的已释放槽」。遍历得到的和照现行（最新根与回退候选集里的有效根引用的槽，含隔离豁免）。盘上门槛 = 最新根自带的「这次发布用的回收门槛」，最新一版由记录施加出来的取那条记录新根段里的；已释放槽 = 最新根分配记录树里已释放、释放代大于盘上门槛的记录罩住的槽，不论有没有一条读得出的根引用它。checker 不再自己算门槛来并槽；门槛对不对由 I-7.18 判。

新不变量（编号 I-7.18 是 `grep -rn 'I-7\.18'` 在 `.claude/kb` 与 `crates` 里零命中的下一个号，待主 agent 认）：

> | I-7.18 | 盘上回收门槛不高于 max(F_生效, 环里最旧有效根) | 最新根（最新一版由记录施加出来的取那条记录新根段）带的「这次发布用的回收门槛」不高于 max(F_生效, 环里最旧有效根)；环里一条有效根都没有时取 F_生效。门槛低于它不判：读坏与读回全零的槽把门槛钉在可能住过的最旧 txg，盘上读得出的根看不到那些槽（D16 已定项 1「根槽这一次读坏」那一行）。这一条只判门槛有没有高过，不判它为什么低。 |

重算 max 的函数现成：`deferred_slots_reclaim_threshold`「fn deferred_slots_reclaim_threshold(」（`crates/singlefs-checker/src/walk.rs:8338`）原样复用给 I-7.18，I-3.1 一侧删掉的是它的调用方（`walk.rs:18149`）和选根式（`walk.rs:8323`）。

### X4.4 I-3.1 改式与 C684 是否冲突

C684 的读法甲（用户 2026-10-20 定「不并，记成已认的代价」，`records/2026-10-18-里程碑三收尾与近期欠账.md` 第 323 行）：

- `.claude/kb/checks-owed.md:447`：「读法甲：照现行原式，这一形归读不出的根那一类已知形态去判，测试钉成例外」
- 上面 X4.3 拟的 I-3.1 新式：「已释放槽 = 最新根分配记录树里已释放、释放代大于盘上门槛的记录罩住的槽，不论有没有一条读得出的根引用它」

不冲突，但「已认的代价」在 I-3.1 上落空。C684 的形是：换下某一槽的那一版 r 的根读不出，那一槽在最新根的账里还在 defer（`.claude/kb/checks-owed.md:447` 写的「那时这一槽在最新根的账里还在 defer」），原式只并读得出的、没被抛弃的根引用着的槽，所以不并，I-3.1 红在记账多的一边。新式按释放代与盘上门槛分，不看有没有读得出的根引用，「还在 defer」= 释放代大于盘上门槛，所以这一形并进来，I-3.1 绿（推的：C684 登记行写小世界穷举里两式不同的 1 082 个形全是这一形，没造真盘面验）。「不并被抛弃根引用的槽」这个选根口径本身没变，只是选根式整个退出了 I-3.1。要改的有：C684 登记行的结局（由「已认的代价」改成「新式下绿」）；`walk.rs:8323` 的选根式与它的小世界等价测试（`walk.rs:19098`）；`crates/mutations.tsv` 里点名 LR5 的行。这是用户已定一笔的结局被后一笔改掉，交用户时要明说。

### X4.5 字段写入后，C773 那一形在 checker 上转不转绿；有没有盘上门槛高于 max 的形

- 转绿（推的）：C773 登记行的复现是种子 215 第 6 步记账 302、遍历 258，差的 44 槽释放代全在 (21, 25]。挂载钉在可能住过的最旧 txg（21），之后这个进程写的每条根按 21 回收、记账里留着释放代在 (21, 25] 的槽；checker 取读得出的根算出的门槛是 25。新式下那条根带着 21，「释放代大于 21 的已释放槽」把这 44 槽并回遍历的和，258 + 44 = 302 = 记账；I-7.18 判 21 ≤ max(F_生效, 25) 成立。复核不了：复现的盘面在 `~/.local/share/singlefs/work/i31-abandon-investigation/` 里，这个目录现在不存在，我没法重造。
- 盘上门槛高于 max(F_生效, 环里最旧有效根) 的形，我试了五种，都没造出（推的）：(一) 零单元发布照抄上一条根的值，而 F_生效 与环里最旧有效根随时间只增不减，抄来的值不会高于现在的界；(二) 抬 F：系统配置先写新 F、带新 F 的根后落，回收在第一条带新 F 的根之前做，根里的 F 字段与系统配置里的 F 都进 checker 的 F_生效（`crates/singlefs-checker/src/walk.rs:17554` 起把被抛弃根带的 F 也算进来，与 D16 已定项 1「生效」一行同口径）；(三) 被抛弃的根：分配器与 checker 都按最新根的实例表排除它们；(四) 根槽撕裂：撕裂的新根不在盘上，最新根仍是上一条，它的门槛是上一轮的；(五) 缺一块盘的崩溃镜像：checker 看到的根更少，最旧有效根只会更高。反过来会让 I-7.18 误红的是 checker 看到分配器没看到的更旧的根——错位写下的根（X1/X5）。
- 一条该说清的事实：核心层读到盘上的这个字段不做任何决定（挂载时分配器自己重算门槛，不读盘上的值；只有施加记录拼根时原样带过去），所以这个字段是给 checker 的证据，不是给 core 用的状态。写错了只有 I-3.1 与 I-7.18 抓得到，变异行必须有「字段恒写 0」「字段写成当前最新门槛而不是这次回收用的」两条，由 harness 档的 checker 用例抓。

### X4.6 代价

- 写盘：每次发布 0 次、0 字节额外（X4.1）；崩溃状态数不变。
- 格式常量三处：`crates/singlefs-format/src/lib.rs` 的 `ROOT_RECORD_BYTES`「pub const ROOT_RECORD_BYTES: u64 = 457;」（`crates/singlefs-format/src/lib.rs:234`）、`JOURNAL_NEW_ROOT_SEGMENT_BYTES`（`crates/singlefs-format/src/lib.rs:244`）、`JOURNAL_HEADER_BYTES`（`crates/singlefs-format/src/lib.rs:247`），与它们在同一文件里的断言（`crates/singlefs-format/src/lib.rs:450`–`:452`、`:415`–`:440`、`:580`–`:591`）。
- 核心：`root_record.rs`（字段、写、读、位置断言 `assert_eq!(reader.position(), ROOT_CHECKSUM_OFFSET + 32 + 3 * …)` 之后加一读）、`journal.rs`（`new_rollback_floor` 同类的新成员，`crates/singlefs-core/src/journal.rs:163`；记录头写读与偏移）、`crates/singlefs-core/src/recovery.rs:4624`/`:4648`、`transaction.rs` 三处根构造、`make_filesystem.rs:581`、`allocator.rs`（交回门槛）、`mount.rs`（零单元计划抄上一条根的值）。
- checker：`RootView` 加字段，I-3.1 改式（`walk.rs:18138`–`:18166` 一段换掉，删约 60 行选根代码），I-7.18 新增，I-8.10 七段改八段，I-2.4 判全零的范围挪到 436..465。
- GPU 判器：`crash_judge_common.wgsl` 三个常量（`:87`、`:89`、`:94`）与根字段解析，`crash_judge_checker.wgsl` 的 I-7.18 与 I-3.1 改式（现有的是 `crash_judge_checker_scan.wgsl:2037` 的 `c_deferred_below_the_floor` 一族），不变量编号表（`crash_judge_checker_common.wgsl`）与对拍用例 `crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`。
- 第三 oracle：`scripts/experiment-apparatus/e170_*` 里的规格执行器自带一张字节表（例如 `on_disk_layout.py` 里的 457、487、364）；上一次改记录头走的是另起一个变体目录（`e170_*_journal_header_487`），这次要照样另起 `…_journal_header_495` 与根记录 465 的变体，旧目录冻结不动。这就是 C773 登记行原先写「E171 冻结之后」的来由；用户去掉了前置，依赖还在：E171 之前冻结的产物对旧格式成立，之后的三 oracle 对拍要用新变体。
- 字节钉值：全盘 CRC 钉值（`unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs` 里补丁已改过一次的那个）、E142 产物、`layout/01-first-txn.md` 字节表（中英日三份）、`checker_judges_reserved_bytes_and_widths_it_used_to_skip.rs` 里的 `ROOT_RECORD_BYTES: u32 = 457`、门禁 format-constants 格登记的 stale 列表。命令数出来的涉及面（含不相干的同数字）：`grep -rlw 457` 在 `crates` 下的 `.rs`、`.wgsl` 命中 17 个文件、在 `.claude/kb` 命中 34 个；`487` 是 23 与 66；`364` 是 8 与 51（命令与原样输出见「我跑过的命令与原样输出」一节的 C）。
- 变异行约 8 条；新用例：C773 复现（读坏槽与读回全零槽，挂载后 I-3.1 绿）、I-7.18 的坏镜像（门槛高于上界只红它）、零单元发布抄上一条根的值、记录施加版本带记录里的值。
- 主 agent 在 `records/2026-10-18-里程碑三收尾与近期欠账.md` 第 338 行估「加 10–16 小时（推的）」；我按上面清单没有理由改这个数量级，没另量。

### X4.7 什么现象会推翻 X4 的结论

- 找到一个带单元的发布，分配器在装记账行之前回收用的值与根里写的值不同（采样点选错）：I-3.1 在合法盘面上红。
- 找到盘上门槛高于 max(F_生效, 环里最旧有效根) 的合法盘面：I-7.18 误红，上面五种之外还有第六种。
- 在 C773 复现盘面上，新式 I-3.1 仍红：说明差的槽不全是「释放代大于盘上门槛」那一类。
- 零单元发布抄上一条根的值之后，在某个盘面上记账与那个值对不上。

## X5　四笔之间的交互

逐对看。

- X1 到 X3（触发并集）：X1 只改「哪些根算读得出」，不改并集的定义。F2 判法里用 `readable_roots`（`crates/singlefs-core/src/recovery.rs:6977`）的两处受影响：末条读得出而根读不出的那一类（`crates/singlefs-core/src/recovery.rs:7003` 起）把错位的根也算「根读不出」，退回更难发生；X2 的新条件里「读得出的根」把错位的根排除，这条件更难成立。两处方向相反，但都是在同一个读函数收口之后用同一个读数，不会互相矛盾。
- X1 到 X4（「环里最旧有效根」）：分配器在 X1 之后把错位槽当读坏、钉最旧 txg，门槛只会更低或相等；checker 读镜像，错位写下的根（I-7.17 红的镜像）仍算有效根，最旧有效根可以比分配器的低，于是「盘上门槛不高于 checker 重算的上界」这个方向可能不成立，I-7.18 在这种镜像上多红一行，I-3.1 的候选集也会多走那条根。这些镜像本来就是 I-7.17 红的，不是合法写者造得出的；条文处理有两种：(甲) 不管，把「I-7.17 红的镜像上其余不变量不作保」写进 I-7.17 那一行；(乙) checker 重算上界与候选集时按同一道落点公式核滤根，约 10 行加 GPU 同改。我的读法：甲，因为合法写者不会造出这种镜像，乙的成本是白付的。**这一格规则没说**，要主 agent 定。
- X2 到 X3（同一个判法函数里的次序）：写死成 (一) 见证过没有（常态路径到此为止）；(二) 建不出来没有（并集）；(三) X2 的条件（实例表读不出且区间里有读得出的根则不退）；(四) 取目标、目标也要建得出。(三) 排在 (四) 前面，条件成立时不白读目标。X3 把实例表链读不出算成建不出来，所以「R 的单元都好、只有实例表读不出、R 没见证」的盘面，只读挂载今天打开 R，统一之后进 (二)(三)：区间里没有别的读得出的根就退到恰好等于见证到的那一版，有就不退照原样打开 R。前一种是 X3 带来的只读挂载变化（落到更旧的一版），这一版没见证，不算已确认，在 D23 已定项 14 的口径之内（`.claude/kb/decisions/23-journal的角色与格式.md:423`：「已确认的版本在较新的系统配置槽与它的单元同时读回全 0 时」才是已确认版本被读回上一版的代价，这里 R 是未见证的）。
- X3 到 X4：重建上一版会读到根上的新字段，但 core 不用它做任何决定（X4.5），所以并集不受影响；施加记录拼根要带过去（`crates/singlefs-core/src/recovery.rs:4624`、`:4648` 两处各加一行）。
- X1 到 X4 的另一处：X4 的采样点用的是分配器根环表里「最旧有效根」；X1 之后表里的槽都是公式槽，`oldest_valid_root_in_the_ring_for_raising_the_floor` 里按公式给根归槽（`crates/singlefs-core/src/mount.rs:4506`）与读槽归属一致，这是 X1 顺手消掉的内部不一致，X4 不用再处理。
- 条文互相打架的地方（四笔一起实现时要一起改的）：
  - D22 已定项 7 的平局句（`.claude/kb/decisions/22-单元原子性怎么合成.md:140`）与落点公式句（`:335`），见 X1.5；
  - D16 已定项 1「同取这个值的六处」（`.claude/kb/decisions/16-发布语义.md:36`）与 X4 的盘上门槛，见 X4.3；
  - D23 已定项 14 F2 段开头的「三处用同一套逻辑」（`.claude/kb/decisions/23-journal的角色与格式.md:398`）与 X3 残留的成败差别，见 X3.3 的改写；
  - D19 已定项 5 的 N2（`crates/singlefs-core/src/recovery.rs:2957`：「提示与映射都读不出的**数据单元**不算失败」）与 X3 并集取严，见 X3.1；
  - C684 的「已认的代价」与 X4 的 I-3.1 新式，见 X4.4。
  除这五处，我没找到别的。

### X5 什么现象会推翻

- 四笔一起实现时，F2 判法函数里 (二)(三)(四) 的次序让某个入库用例的结局与上面写的不同。
- X1 的公式核让 X3 的并集在某个合法盘面上失败（那说明合法写者写出了不住公式槽的根，X1 的前提不成立）。

## 要主 agent 跑的命令（我没跑的；都不是重型，除注明）

- 实现之后核 X1 的手摆根旧用例（22 个候选里我核实了 3 个，其余 19 个没核）：先列清单，再逐个核。清单命令（轻型，我已跑，输出见下一节「我跑过的命令」B）：`cd /home/fy5090/code/singlefs && grep -rl 'slot_offset(' crates/singlefs-harness/tests crates/singlefs-harness/src | xargs grep -l 'to_slot('`。
- E165 读槽对拍（重型，`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:11086` 一带的停机条款 TH4）：X1 实现之后由主 agent 照 `.claude/skills/crash-test/commit-stages.md` 的口径跑，看「读数与真相不同」的槽数是否为 0。
- C773 复现盘面（种子 215 那一形）要重造：`i31-abandon-investigation` 目录现在没有，调查员报告也不在；重造要重跑删除线攻方探针（`m3-batch50-code-r1-opus-s2-output.md` 登记行点名的那一份）。
- X3.4 的前提（无读故障的崩溃态上并集 = 走读）我只有别人量的 137 902 态；X3 实现之后让并集在同样的态上各调一次，看失败数是否为 0（重型）。

## 我跑过的命令与原样输出

A. 读根环槽字节的解析点（非测试代码，各文件在第一个 `#[cfg(test)]` 之前；`recovery.rs`、`mount.rs`、`instance_table_sweep.rs`、`transaction.rs`、`allocator.rs`、`mounted_read.rs`、`mounted_session.rs` 七个文件）：

```
crates/singlefs-core/src/recovery.rs:1568:    match RootRecord::parse_slot(&bytes, filesystem_identifier) {
crates/singlefs-core/src/recovery.rs:4172:            (RootRecord::parse_slot(&bytes, &root.filesystem_identifier).as_ref() == Some(root))
crates/singlefs-core/src/mount.rs:2737:    match RootRecord::parse_slot(&bytes, filesystem_identifier) {
instance_table_sweep.rs:481:            root_read = RootRecord::parse_slot(&bytes, filesystem_identifier);
```

A2. 两个读函数的非测试直接调用点（同样的文件范围）：crates/singlefs-core/src/recovery.rs:1507、1591、1720、1731、1807；crates/singlefs-core/src/mount.rs:1403、2984、3538、3550、4448、5895，共 11 处（输出在草稿目录的 `read-fn-callers.txt`，行内容与 X1.3 的表逐项对过）。

B. 同时含 `slot_offset(` 与 `to_slot(` 的 harness 文件数：`22`。

C. 数字 457、487、364 的涉及面（`grep -rlw`，`crates` 下 `.rs`、`.wgsl`；`.claude/kb` 下全部）：

```
457 crates(.rs/.wgsl): 17  kb: 34
487 crates(.rs/.wgsl): 23  kb: 66
364 crates(.rs/.wgsl): 8  kb: 51
```

D. `grep -rn 'I-7\.18' .claude/kb crates | wc -l`：`0`。

E. 写根槽的落盘点：`grep -rn 'WriteRootRecordForceUnitAccess' crates/singlefs-core/src`：

```
crates/singlefs-core/src/transaction.rs:199:    WriteRootRecordForceUnitAccess {
crates/singlefs-core/src/transaction.rs:504:            CommitStep::WriteRootRecordForceUnitAccess {
crates/singlefs-core/src/transaction.rs:1533:    writer.perform_commit_step(CommitStep::WriteRootRecordForceUn
```

（第三行是我 `cut -c1-110` 截断的显示，不是文件内容；原行见 `crates/singlefs-core/src/transaction.rs:1533`。）

## 判定一览（末尾版）

| 格 | 判定 | 一句话 |
|---|---|---|
| X1 C743 | 一致；冲突 3 处（`core_review_tree_table_duplicates_and_slot_one_search.rs:479` 与两个择根平局用例的手摆根）；规则没说 2 处（D22 平局句与公式句谁让步；别的自证结构的读错位） | 公式核加在两个读函数和两处直接解析上就收齐 core 的读根环处；合法写者只有 `crates/singlefs-core/src/transaction.rs:508` 与 mkfs 第 0 代，写出的根都过这一核；读错位不进 checker |
| X2 C751 | 冲突（用例 `:656` 的只读断言要翻）；规则没说（改一处还是改共用判法，我读乙） | 条件 = 实例表读不出且（见证到的那一版, 所选根）之间有读得出的根，所需量都已在内存；F2 其余格结局不变 |
| X3 C755 | 一致；规则没说 3 处（并集取严让 N2 在没见证时让位；成败不统一；GPU 判官不移植的射程句） | 并集 = 走读、打开、建上一版读的单元全部；只在没见证时才多读；无读故障崩溃态上并集 = 走读（别人量的 137 902 态，我没复跑） |
| X4 C773 | 一致；冲突 4 处要同步改（D16 同取六处、D22 合计与字节表、D23 头与新根段的数字、C684 的已认代价） | 根记录 428 处加 8 字节（465），journal 新根段末尾加 8 字节（372，头 495），写盘 0 次增加；I-3.1 改按盘上门槛并槽，I-7.18 判门槛不高于；C773 复现转绿是推的，复现盘面已不在 |
| X5 四笔之间 | 一致；规则没说 1 处（checker 重算是否也按公式滤根，我读不滤） | 五处条文要一起改；X2 条件住在 X3 统一的判法里，次序写死 |

## 没做什么

- 没跑 cargo、门禁、重型测试，没编译；所有「会红」「会绿」「走得到」「转绿」都是读码推的，已逐处标明；别人量的数（137 902 态）没复跑。
- 没改 kb、没改 `crates/`；上面的条文是拟稿，落地归写回批与实现员。
- 没造盘面：C751/C755 的盘面、C773 的复现都没造（C773 复现目录不存在）；X2、X3 的结局表是读码对出来的。
- X1 手摆根的旧用例只核实了 3 个，其余 19 个是 `grep` 出来的候选，没逐个读。
- X1 (f) 别的自证结构（系统配置槽、journal 记录、单元）的读错位处置没查。
- GPU 判器与 E170 规格执行器里要同步的偏移，只列了 grep 到的，没逐文件穷尽。
- 时间与工作量没另量，沿用主 agent 在 `records/2026-10-18-里程碑三收尾与近期欠账.md` 第 338 行的估计。
- 没写模型目录（这一轮是读码加算术，没有需要独立复跑的模型）；没判攻方腿的格，没替主 agent 采纳；没读禁读清单里的文件。

## cite-check 末行

`python3 scripts/agents/cite-check.py research/prompts/m3-close-rootring-r1-sonnet-output.md --background research/prompts/_m3-close-rootring-r1-background.md` 的末行原样：

```
  ✓ 核了 39 处引文、148 处裸行号，对上 39 处，没判 148 处（其中裸行号没有引文可比 148 处；逐处列在上面）
```
