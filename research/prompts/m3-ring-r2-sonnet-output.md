# m3-ring-r2 辩方腿报告（Sonnet，2026-10-02）

立场：替被第一轮判出局或收窄的一方（P2、P3、K2 / K3 的采法）辩护，复核 K1 是否真是「四个挂点都补不了」，并推 G1–G4 的条款形态与代价。只读与推导，没编译、没跑测试；副本上没造盘面，所有「推的」都是读 `crates/` 与 kb 得出的，标在句首。
接续：前一条同立场腿撞限额，没落报告；草稿目录与模型目录都是空的（`ls` 现查），我从头做。

## 现场核对

```
$ cd /tmp/claude-1000/m3-batch4-freeze/tree && sha256sum -c /home/fy5090/code/singlefs/research/prompts/m3-ring-r2-snapshot/tree-sha256.txt | awk '{print $NF}' | sort | uniq -c
    436 OK
```

`crates/` 的几份我引到的文件（`mount.rs`、`recovery.rs`、`allocator.rs`、`transaction.rs`、`journal.rs`、`root_record.rs`、`singlefs-replay-prefix/src/lib.rs`）与主工作区逐字节相同（`cmp`），kb 的 D23、D22、`invariants.md`、`checks-owed.md` 四份主工作区之后又改过、行号移了；所以 D23、D22 我写 `@小节标题`，`checks-owed.md` 只引三行未移动的（C331 / C596 / C597），不引 `invariants.md` 的行号。

自己写的两处算术：`research/prompts/m3-ring-r2-sonnet-model/g1_g4_arithmetic.py`（sha256 6ecfe494…），输出贴在 G1、G4 两节。

## 一、判决逐条复核（`research/prompts/m3-ring-r1-main-verification.md` 第二节十二行）

判定三档：站得住（打得中、且只打被判的那一方）/ 够不着（引的东西字面上满足的是别的分句，或前提不成立）/ 同样打中别的臂。

| # | 判决条目 | 辩方判定 | 证据与推翻条件 |
|---|---|---|---|
| 1 | H1：定 P1，理由是「唯一 K2、K3 都不中的挂点」 | **结论站得住，理由够不着挂点 5**（P2 根前、分配器先记） | 判决原文：「唯一 K2、K3 都不中的挂点，两条腿同向」（`research/prompts/m3-ring-r1-main-verification.md:26`）。攻方自己的对照表里挂点 5 也是 K2 不中、K3 前缀截点 0 红：「前缀截点 0 红（量过）；门槛先抬的窗口（推的）」（`research/prompts/m3-ring-r1-opus-output.md:278`）。能把挂点 5 与 P1 分开的只剩两样：门槛先抬的窗口（攻方「推的」、242 个截点没红、「没分清」，`research/prompts/m3-ring-r1-opus-output.md:164`）、记录核对器 58983 态红（攻方也说原因是「推的」，装置按位置认发布）。所以「P1 唯一」不是 K2、K3 分出来的；维持 P1 的理由要改写成「P1 是不必回答挂点 5 那个窗口的挂点」。推翻：带文件后洞（4 325 419 态，没全量跑）里挂点 5 出现 I-7.4 红，则挂点 5 出局、原理由补全；该段全量里挂点 5 零红，则判决只剩「P1 更省事」，不再是「唯一」 |
| 2 | H1-b：写行、首个文件、卸载三种发布施加出来是错根；要么加三个字段（点名项 67 → 64）要么不补 | **够不着：三种里写行那一种走不到；字段数偏大** | 见 G1 一节：写行记录只在所选根是同一个实例时才会被恢复施加，而新实例的第一次发布就是写行，所以它不可能被施加（`crates/singlefs-replay-prefix/src/lib.rs:1265`「record.instance.0 == root_instance.0 && record.checkpoint_txg.0 > root_checkpoint_txg.0」；攻方在副本上量到 `research/prompts/m3-ring-r1-opus-output.md:92` 的 `chosen=27 effective=27`）。只有首个文件与卸载两种真会补出错根，各差一位信息，借记录标志字节里的两位，头不涨、点名项仍 67。推翻：造出一份写行记录已持久、根槽没持久、而 effective_root 的 txg 大于 chosen_root 的盘面 |
| 3 | K1：「打中，不分辨臂」；交用户的两条路是记欠账或一圈窗口 | **「补不了」站得住，「只剩这两条路」够不着** | 补（写一条有效根）确实四个挂点都做不到：恢复不施加别的实例的记录（`research/prompts/m3-ring-r1-opus-output.md:84`）。但同一个挂点位置（取号之后、写行之前）可以做「抹」：把被跳过的 txg 的槽（K1 里是 slot(h)）里留着的残留根写成自证不过。这是第三条路，零轮，见第二节。推翻：第二节「抹」的实现里有一个合法崩溃状态让候选集里的根引用已复用单元，或抹掉的根在合法历史里是回退要用的 |
| 4 | K6 前洞：补了不放槽（61 对 61） | **站得住** | 攻方量到 `withheld=61 over_withheld=8` 补前后扣住的总数不变（`research/prompts/m3-ring-r1-opus-output.md:126`、`:127`）；K1 的前洞多扣为 0（`research/prompts/m3-ring-r1-opus-output.md:102`）。对 P1、挂点 5、抹都一样，不分臂 |
| 5 | H2：「没有挂点按字面出局」；正推小模型只写了 P1 红 0 | **够不着（漏了一行）** | 正推腿 H2-b 表里 P2-eager（补洞记根排在写行落点之前，即攻方挂点 5）后洞红 5、标「出局（这一种写法）」（`research/prompts/m3-ring-r1-sonnet-output.md:183`；原始输出 `research/prompts/m3-ring-r1-sonnet-output.md:153`）。判决这一行写的是「正推小模型 P1 红 0」（`research/prompts/m3-ring-r1-main-verification.md:30`），没写 P2-eager 的 5。两条腿冲突：小模型红 5，真代码前缀截点红 0 且「没分清」。小模型的前提是补洞写与写行根同段、可以没落；真录制流里根槽写是 FUA、两个根槽写落在相邻的段里（`research/prompts/m3-ring-r1-opus-output.md:135`）——但「单元已写、补洞根没落」的状态在真流里同样存在，前提不同只改红的个数。结论：挂点 5 在字面判据上「正推模型出局、真代码没造出、没全量」，不是「没挂点出局」 |
| 6 | K4 必红世界：补洞写成全零或所选根字节，I-3.1 只红在假根成为最旧那一步，checker 没有槽位与 txg 对得上的不变量 | **站得住，同样打中所有「补」的臂** | 补洞写错字节与挂点无关，P1、P2、P3 都吃；不分臂（`research/prompts/m3-ring-r1-opus-output.md:218` 攻方自己写了「checker 里也没有槽位公式」）。对「抹」不适用（抹写的是自证不过的槽，不冒充根），但它让槽位不变量的缺口换了个形状：抹掉的槽与读不出的槽在盘上分不出，这一点判决已记 |
| 7 | K7：不补时带后洞的池上 I-3.1 在今天的代码上合法状态红（50 个跟进状态里 32 与 35 个） | **站得住（副本数）；与第二节的「抹」同向** | 判决已把它放进「进下一批派调查员在入库装置上复现」，我不动。补充一点推的：红的根源是残留根 s 拖低门槛而账又按别的口径记（分配器按 `oldest_valid_root` 取门槛，`crates/singlefs-core/src/allocator.rs:998`「fn oldest_valid_root(&self) -> Option<CheckpointTxg> {」），把 s 抹成自证不过之后两边看见的环相同。推翻：入库装置上复现不出这 32 / 35 |
| 8 | H3：C380 不关，对象改写成被抛弃或低于 F 的根、读不出的根槽、P1 不补的洞（K1 那一类） | **站得住，但「K1 那一类」那一项要等第二节的结果** | 若走「抹」，K1 那一类残留根被抹掉，I-3.9 单值判法在 K1 后洞上的 10 红（`research/prompts/m3-ring-r1-opus-output.md:229`）随之消失（推的：红的是同一个槽「释放代 5 不在 (4, 8] 里」，L 是残留根 txg 4）。C380 改写的措辞不要先写死 K1 |
| 9 | H4：S-辨 与 C331 甲的丢写 0 冲突，120 | **站得住，但这 120 落在用户已定「不罩」的双故障形里** | 见 G3 一节 |
| 10 | H5：C597 要整轮清扫；不做的代价 10⁴ 次 28 片、10⁶ 次 2 711 片；主 agent 倾向这个里程碑不做 | **数站得住（我重算同数）；「(j, 0, 0) 行正是删不掉的那一类」够不着** | 见 G4 一节；片数我重算：`M=10000: 片数 28`、`M=1000000: 片数 2711`（`/tmp/claude-1000/m3-ring-r2-sonnet/arith.out`，脚本在模型目录） |
| 11 | H6：C462 单列错误成员、C463 加字段、自愈随 H4 | **站得住**，我没有反例 | 与 S-辨 同一个闸：一块盘少于两槽自证过时可写挂载在取号之前拒（`crates/singlefs-core/src/mount.rs:406`「| WitnessedCounterComparison::SomeSystemConfigurationSlotUnreadOrUnverified => true,」）；用户选了 S-辨，自愈那一半随 G3 的结果走 |
| 12 | 旁及 C333：前提「记账是那次发布的」与施加记录时分配记录树根照抄所选根冲突 | **站得住，且第 2 行的发现让它更窄** | 代码照抄：「allocation_record_tree_root: root.allocation_record_tree_root,」（`crates/singlefs-core/src/recovery.rs:3173`）；C333 描述的「施加一条写行记录」那一形按第 2 行走不到。进下一批回扫时先核这一点 |

## 二、K1：抹，第三条路（零轮；推的，没造盘面）

### 它是什么

取号之后、写行那次发布之前（P1 的那个位置，单独一段 [根槽 FUA][屏障]），对每个被跳过的 txg t（`first_txg` 与生效那一版之间没有根的 txg，至多 3S 个）：若 slot(t) 里住着自证过、txg 严格小于生效那一版 txg 的根，把那个槽写成自证不过的字节；再对环里 txg ≤ 生效那一版 txg − 3S 的残留根（更早留下的洞）同样处理。从不抹生效那一版自己的槽。屏障之后分配器根环表摘掉那些槽的占用者（被抛弃根被摘时要走与轮转覆写同一支清隔离位，`crates/singlefs-core/src/allocator.rs:1658`「match displaced {」），再往下做。能补的（`effective_root` 大于 `chosen_root`）照 P1 补，补不了的（K1、洞②、旧盘旧洞）抹。语义上就是：被跳过的那次发布若成了，它的根会盖掉 slot(t)，抹只是把这件事提前做完。

为什么这一步写得出：
- 门槛本来就是「环里最旧有效根」：「oldest_valid_root.map_or(effective_floor, |oldest| effective_floor.max(oldest))」（`crates/singlefs-core/src/mount.rs:1209`）；占用者表里读不出、自证不过的槽根本不在：「读不出、自证不过的槽不在 `RootRingOccupancy` 里」（`crates/singlefs-core/src/allocator.rs:941`）；挂载读环时两种坏槽各有一支：「RootRingSlotReading::Bad(BadRootRingSlotReading::NotSelfVerified) => {}」（`crates/singlefs-core/src/recovery.rs:1097`）。所以抹掉残留根之后，最旧有效根自然前移到窗口里最旧的那一条，等于「一圈窗口」的门槛，不用改门槛式子、分配器、checker 的候选集。
- 写盘的现成东西：`CommitStep::WriteRootRecordForceUnitAccess`（`crates/singlefs-core/src/transaction.rs:1218`「writer.perform_commit_step(CommitStep::WriteRootRecordForceUnitAccess {」）按 txg 算槽；抹只是把写进去的字节换成非根字节。
- 不需要往根环表里「插一个 txg 小于已记最新的根」：补洞写在表里要另开入口（正推腿 H1-c 的 P1 第 2 步），抹只是 `occupants.remove`，不碰逐个加一的断言：「let expected_txg = CheckpointTxg(occupancy.latest_recorded_txg.0 + 1);」（`crates/singlefs-core/src/allocator.rs:1641`）。

### 对 K1 的数（推的）

K1 后洞的 350 / 158 槽（`research/prompts/m3-ring-r1-opus-output.md:93`）来自 slot(h) 里的残留根拖低门槛；抹掉它之后门槛等于攻方 `window_floor` 那一列：攻方量到 `peak_released_by_window_floor=366`（S = 8）与 174（S = 4）（`research/prompts/m3-ring-r1-opus-output.md:92` 同一块输出）。抹与窗口门槛在后洞上等价这一点是代数推的（门槛只看有效根的最小 txg，抹与「候选集不收一圈以外的根」让最小 txg 取同一个值），没造。

### 它的代价与风险（别当成免费）

1. 少一条回退候选：残留根 h − 3S 本是环里最旧的合法候选，抹了就没了。P1 补后洞同样少这一条（正推腿 H1-d）；K1 那一类今天不少。要交用户的是「比最新根旧一整圈的残留根算不算回退候选」。
2. 每个被跳过的 txg 一次根槽 FUA，至多 3S 次（D23 注 3 写了跳号最多到被抛弃时间线的长度，`@已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换` 里「跳号比今天大（最多到被抛弃时间线的长度）」）；取号之后单独一段，崩在中间只有「抹了」「没抹」两态，没抹时与今天相同。
3. 抹错的风险，要一条硬规则挡住：抹的范围以生效那一版（`effective_root`，没有补洞时就是 `chosen_root`）的 txg 为锚，不以 `first_txg − 1` 为锚。`first_txg` 取环里根与环里记录的最大 txg 加一（`crates/singlefs-core/src/mount.rs:1165`「checkpoint_txg_after_the_version(highest_ring_txg.max(highest_record_txg))」），被抛弃时间线上记录的 txg 可以大得多（D23 注 2 写了「被抛弃实例开放 checkpoint 里已提交的记录 txg 可以更大」，`@已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换`）；锚在 `first_txg − 1` 上，跳号一长就会把生效那一版自己和它之前的整个环都抹掉。锚在生效那一版上，最坏是跳号 ≥ 3S 时环里只剩生效那一版、回退深度没了——一圈窗口在同一种跳号下结果相同（窗口同样要锚在最新有效根上，见 G2），不是抹独有的代价。读不出的槽不抹（那一槽里可能正住着一条要用的根，与正推腿 H1-a 同一条理由）。
4. 「抹」与「撕裂」在盘上分不出（攻方 K4 的第二点），所以它不让槽位不变量更容易写；这一点与「补」同。
5. 与读不出的根槽共用一个处置：抹之后环里这个槽「读成功、自证不过」，不是「读不出」。C597 的回收条件要写清这两种槽不同对待（G4 一节）。

### 对第二、三轮判决要追加的一条前提（G2 一节展开）

若判决仍选「一圈窗口」，窗口的「最新根」在挂载时要取环里读得出的最大 txg，不是 `first_txg − 1`：分配器根环表装表时 `latest_recorded_txg` 写的是 `first_txg − 1`（`crates/singlefs-core/src/mount.rs:1648`「新实例第一次发布的 txg = 环里最大 txg + 1 ≥ 1」那句 `expect` 的上一行），K1 之后它比环里最大有效根大 1（每个被跳过的 txg 多 1），窗口在挂载时就已把残留根踢出，第一次发布的落点就可能复用它引用的单元，而崩溃状态里残留根仍是候选。见 G2。

## 三、G1：新根段加三个字段

### 先找先例：写行那一种已经销过账（C533）

找到同类先例：C533（记录新根段缺实例表与分配记录树两个根指针）问的是同一件事，用户 2026-09-24 已销账：「前提不成立——写行那条记录因实例边界不施加」（`.claude/kb/checks-owed.md:585`，冻结副本里同一行是 583）。钉它的用例文件头写着：「写行那次发布恒是新实例的第一次发布，所以…今天走不到。」（`crates/singlefs-harness/tests/row_publish_record_without_its_root.rs:12`）。

代码侧的链条，逐环现查：
1. 恢复只施加所选根自己那个实例、txg 更大的记录：「record.instance.0 == root_instance.0 && record.checkpoint_txg.0 > root_checkpoint_txg.0」（`crates/singlefs-replay-prefix/src/lib.rs:1265`）；条款是 D23 注 1：「下一条的实例代号与所选根不同即停」（`@已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换`）。
2. 写行是新实例的第一次发布：「写行那次发布是新实例的第一次发布」（`.claude/kb/decisions/18-块里携带什么信息.md:308`）。它的记录是新实例的第一条，所选根一定是旧实例的，所以不被施加。
3. 根里实例表指针与上一版不同的只有写行那一次：整个 `crates/singlefs-core/src` 里 `InstanceTablePlan::Rewrite(` 的构造只有一处，在写行的计划里（`crates/singlefs-core/src/mount.rs:2801`「instance_table: InstanceTablePlan::Rewrite(instance_table),」）；别的发布都是 `Carry`（「InstanceTablePlan::Carry(pointer) => *pointer,」，`crates/singlefs-core/src/transaction.rs:6255`）。
4. 所以「施加出来的根的实例表指针照所选根」对每一条会被施加的记录都恰好对：「instance_table: root.instance_table,」（`crates/singlefs-core/src/recovery.rs:3169`）。

不一致只剩一处要交代：施加代码自己的注释写了「施加一条写行记录而它的根槽没落盘时，重建出来的这一版仍指着上一版的分配记录树」（`crates/singlefs-core/src/recovery.rs:3172`），像是认为那一形走得到。按上面四环走不到（推的）；C533 的用例钉的是同一个结局。要验：在 harness 里再造一份「写行记录已持久、根槽没持久」的盘面，断言 `effective_root` 等于 `chosen_root`——C533 那条用例已经在做，我没重跑它（重型测试不跑）。

结论：第一轮正推腿 H1-b 的三种里，写行那一种是死分支；判决第 2 行「要么加三个字段（点名项 67 → 64）」里的实例表指针那 86 字节没有对象。

### 真要补出错根的只有两种，各差一位信息

施加出来的根逐字段（`crates/singlefs-core/src/root_record.rs` 的 `RootRecord`）与那次发布写的根比：

| 字段 | 施加时取自 | 与当初写的根 |
|---|---|---|
| filesystem_identifier、instance、checkpoint_txg、tree_table、tree_identifier_watermark、rollback_floor、mapping_root | 记录或所选根 | 相同（记录与根在同一处用同一批局部量造：记录 `crates/singlefs-core/src/transaction.rs:6843`「new_rollback_floor: plan.rollback_floor,」，根在同一函数里 `crates/singlefs-core/src/transaction.rs:6850` 起） |
| instance_table | 所选根 | 相同（上面第 3、4 环） |
| unmount_marker | 常量 0：「unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,」（`crates/singlefs-core/src/recovery.rs:3163`） | **不同**：卸载那一串的每条根写 1：「RaiseFloorEntry::Unmount => UnmountMarker::WrittenByTheUnmountSequence,」（`crates/singlefs-core/src/mount.rs:2048`） |
| allocation_record_tree_root | 所选根：「allocation_record_tree_root: root.allocation_record_tree_root,」（`crates/singlefs-core/src/recovery.rs:3173`） | **不同**：带文件的一版恒全零：「allocation_record_tree_root: NodePointer::empty_root(),」（`crates/singlefs-core/src/transaction.rs:6862`）；零单元发布照抄上一版：「allocation_record_tree_root: previous_root.allocation_record_tree_root,」（`crates/singlefs-core/src/transaction.rs:1322`）。所选根是树表 0 条的一版、被施加的是首个文件那次发布时才不同 |

所以要的是两位：记号一位、「这次发布的根里分配记录树根是全零」一位。上面这张表是推的，没造盘面；造法：层 0 里首个文件发布与卸载那一串各造一次「记录持久、根槽没持久」，把 `effective_root.to_slot(...)` 与那次发布实际写的根槽字节逐字节比。

### 放哪里：借记录标志字节的两位，头不涨、点名项仍 67

记录标志是一个字节，今天只用位 0，其余位写 0、读到非 0 当损坏（`@已定项 4：记录头约束在单个原子单元内，头 311 字节`「其余位写 0，读到非 0 当损坏」）。位 1 = 本次发布的根写卸载记号，位 2 = 本次发布的根的分配记录树根全零；施加时据此填（原来 `recovery.rs:3163` 与 `:3173` 两行改成按位取）。我的算术：

```
$ python3 research/prompts/m3-ring-r2-sonnet-model/g1_g4_arithmetic.py
现行: 头 311 字节，装 67 个点名项
正推腿三字段：实例表指针 86 + 分配记录树根指针 86 + 记号 4: 头 487 字节，装 64 个点名项
只加 1 个标志字节（记号一位、分配记录树根一位）: 头 312 字节，装 67 个点名项
借记录标志字节里的两位（头不涨）: 头 311 字节，装 67 个点名项
多占多少字节点名项仍是现行那个数：至多 33
```

也就是说就算不借位、另加字段，只要不超过 33 字节，点名项还是 67；三字段那一版要 176 字节才降到 64。

### G1 的代价（按 `crates/` 今天的实现）

借位版：
- 一次发布最多点名项不变（67），一次发布跨记录的起点不变：67 片 × 369 行 = 24 723 行（正推腿 H5-d 的数，我重算同数）。
- 点名项 67 → 64 的代价只在用户坚持三字段时才付：一条记录装的点名项少 4.5%，大发布的记录条数最多多 4.7%（67 ÷ 64 = 1.0469）；跨记录起点 24 723 → 23 616 行。
- 要同步的东西（我列到的，推的）：D23 已定项 4 的标志位说明与已定项 15 的施加句（`@已定项 15：崩在记录持久之后、根槽持久之前，恢复由记录重建那次发布的根`「施加一条记录 = 把所选根的这四个字段换成记录里的，不需要树语义」这一句要加两位）；`.claude/kb/layout/01-first-txn.md` 里记录标志那一行（首个文件与卸载那几条流的值变）；`crates/singlefs-core/src/journal.rs` 的标志位编解码与「其余位非 0 当损坏」那一判（`place_in_publish` 字段，`crates/singlefs-core/src/journal.rs:156`）；三处造记录的地方（`crates/singlefs-core/src/transaction.rs:1305`、`:1827`、`:6843`）；施加那一处（`recovery.rs:3163`、`:3173`）；层 0 固定脚本流里含首个文件发布与卸载那一串的几条（记录字节变，要重跑登记）；记录核对器要加一条「施加出来的根逐字节等于那次发布写的根」（单镜像 checker 做不了，要 journal 作第二输入，D13 已定项 7）。`singlefs-replay-prefix`（Verus 证过的前缀核）里记录的类型是按根来源参数化的，标志位两位不进它的证明（推的，没读证明）。
- 格式是软的：「第一个外部用户出现之前格式是软的」（`.claude/kb/decisions/15-格式冻结政策.md:173`），不走兼容通道。

### G1 的前提

- 只有补洞挂点是 P1（或第二节的抹）才用得着这两位：补洞写的字节要等于当初的根字节，P1 补的就是 `effective_root`。
- 若用户仍选三字段（比如想让别的实例的记录将来也能施加），那 86 字节指针在今天的恢复规则下没有消费者；要有消费者，得先改 D23 注 1「前缀规则不跨实例边界」，那是另一题、另一轮。
- 推翻第 2、3 行（写行那种走不到）：见上文「要验」那一句。

## 四、G2：一圈窗口门槛

### 式子（写全；推的，没实现）

记 n = 环里读得出、按实例表判仍然有效的根里最大的 txg，R = 3S（`crates/singlefs-core/src/root_ring.rs:114`「region: checkpoint_txg.0 % ROOT_RING_REGIONS,」决定槽位，环长 3S）。

- 窗口：txg > n − R 的有效根。
- 门槛：max(F_生效, 窗口里最旧的有效根的 txg)。今天是「max(F_生效, 环里最旧有效根)」（`.claude/kb/decisions/16-发布语义.md:33`）。
- 回退候选集：今天「按 cur 的实例表判仍然有效 ∧ txg ≥ F_生效」，加一个 ∧ txg > n − R。
- 与 F_生效：F 只管下界，窗口只管上一圈之外；谁高取谁，F 的抬法不动。
- 与 I-3.9：窗口之外的残留根不再进候选集，K1 那一类里「L 是残留根」那一项消失；窗口之内的洞（被跳过的那个 txg 本身）仍在，区间写法与 C380 不动。

### 要改的地方（今天的实现）

| 处 | 现在 | 要改 |
|---|---|---|
| 分配器根环表 | `oldest_valid_root` 取 `ValidRoot` 占用者的最小 txg：「fn oldest_valid_root(&self) -> Option<CheckpointTxg> {」（`crates/singlefs-core/src/allocator.rs:998`） | 先滤 txg > n − R；n 取表里 `ValidRoot` 的最大 txg（表里没有这个读法，新加）；进程内每记一条根（`crates/singlefs-core/src/allocator.rs:1655`「crate::mount::reclaim_floor(occupancy.effective_floor, occupancy.oldest_valid_root());」）n 随之前移 |
| 挂载重建 | `roots` 里按实例表判有效的最小 txg（`crates/singlefs-core/src/mount.rs:1590`） | 同一个窗口；n 取这一遍读到的有效根最大 txg |
| 抬 F | 同样一段（`crates/singlefs-core/src/mount.rs:2509`） | 同上 |
| checker 候选集 | 滤被抛弃与低于 F（`crates/singlefs-checker/src/walk.rs:5975`「let abandoned = abandoned_by_the_newest_roots_table(root);」、`:5976`「let below_floor = root.checkpoint_txg < effective_rollback_floor;」） | 再滤窗口外；I-3.1、I-3.9、I-7.4 都吃这个候选集（`@I-3.9` 等条目引用「回退候选集」） |
| 条款 | D16 已定项 1 的谓词与「形态取 E139（按盘回退下界的收严形态） 的臂 G2S，与装置里跑的一字不差」（`.claude/kb/decisions/16-发布语义.md:29`）；D23 已定项 14 的候选集一行；I-7.4 | 谓词换了就不再是 E139 那一字不差的臂，要重跑计数模型（E156 装置里门槛 = 环里最小 txg，`crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs` 的 `missing_txg_count` 一带） |

### 放出多少槽（攻方盘面度量，不是入库装置上的数）

K1 后洞：不补时扣住的多出 350（S = 8）与 158（S = 4）（`research/prompts/m3-ring-r1-opus-output.md:93`）；窗口门槛多放出 366 与 174，四个补洞挂点在这一格放出 0：「不可补的后洞上 366（S = 8）、174（S = 4），而四个挂点在这一格放出 0」（`research/prompts/m3-ring-r1-opus-output.md:112`）。可补的后洞不补时窗口放出 382 / 190、补了之后 0，所以窗口与补洞在可补的后洞上放出的相同。前洞 0（环还没转满）。这些数我没重跑，是攻方副本上的盘面度量。

### 最要紧的一问：有没有合法状态让候选集里的根引用已复用的单元

先立一个能推的不变量：今天分配器的回收发生在「这次发布的落点全部取完」之后：「这次发布的落点全部取完、之后不再分配」（`crates/singlefs-core/src/allocator.rs:1630`），所以一次发布抬高的门槛只对下一次发布的落点起作用，那时这一次的根已经持久。只要候选集 ⊆ {txg ≥ 门槛的有效根}，被复用的单元就不会被候选集里的根引用。窗口的候选集与窗口的门槛由同一个窗口定，这条不变量照样成立；健康的环（没有残留根）里窗口就是整个环，与今天相同。

所以只有「分配器认的 n 比盘上读得到的 n 大」的地方能造出反例。找到一处（推的，读代码得出，没造盘面）：

- 挂载装表时 `latest_recorded_txg` 是 `first_txg − 1`：「环里只有记录、没有根的那几个 txg 这时已经跳过，它们的槽上照旧是读出来的内容」（`crates/singlefs-core/src/allocator.rs:973`）。K1、洞② 之后它比盘上有效根的最大 txg 大（每个被跳过的 txg 大 1）。
- 若窗口的 n 直接用这个值：挂载那一刻残留根 s（txg h − 3S）已被踢出门槛，第一次发布（txg h + 1）的落点可以复用 s 引用而别的根不引用的单元。
- 崩在那次发布的单元已写、根槽没写：盘上有效根最大 txg 仍是 h − 1，窗口 txg > h − 1 − 3S 含 s，s 是回退候选，而它引用的单元已被覆盖。即「候选集里的根引用已复用的单元」，合法崩溃状态。

按第五节的跑前判据，这一形打中的话一圈窗口出局。修法是前提而不是出局：窗口的 n 在挂载时取盘上读到的有效根最大 txg，不取 `first_txg − 1`；进程内再往前移。这样 s 要等本进程的第一条根（txg h + 1）落盘记下之后才被踢出，那时才轮到下一次发布的落点用它，反例消失（推的）。第二处同类：n 必须只数有效根、不数被抛弃根（被抛弃时间线上的根 txg 可以大得多，数进去会把活的时间线的根踢出窗口）。

「抹」不需要这个前提：残留根在任何单元写之前已被抹掉。

判定：一圈窗口站得住的条件是上面两条 n 的定义；没造盘面，要在 K1 历史上造「挂载 → 第一次发布单元写完、根槽没写」逐前缀截点扫一遍，看 I-7.4 红不红。推翻：n 取 `first_txg − 1` 的版本在那条前缀扫里 I-7.4 绿（那就是我读错了分配器取落点的次序）；n 取盘上最大有效根的版本在同一扫里 I-7.4 红。

## 五、G3：S-辨（C596 取 S-辨，C331 甲 改成 甲-读错）

### S-辨 就是 甲-读错，代价表已有

岔路单的定义：「**甲-读错**：只有设备报读错的槽当判不出，自证不过（撕裂、全零、内容坏）照今天。」（`research/prompts/unreadable-at-mount-r2-forks.md:7`）。上一轮判决的表（`research/prompts/unreadable-at-mount-r2-main-verification.md:46`）：「| 甲-读错 | 120（全是 `Zeros` 形） | 284（全是 `DeviceError` 形） | 可写 | 永远只读 |」，列头是丢写 / 多拒 / 撕裂轮换 / 一槽系统配置持续设备读错。三个对照：今天 丢写 240；甲 丢写 0、多拒 719；甲-读错-每盘（攻方提，零轮）丢写 120、多拒 98，撕裂与一槽读错都可写（`research/prompts/unreadable-at-mount-r2-main-verification.md:47`）。

### 丢写 0 → 120，这 120 回来多少、落在哪

- 相对 甲：丢写 0 变 120；相对 今天：240 变 120（回来的是 `DeviceError` 那一半）；多拒 719 变 284（`DeviceError` 361、`Zeros` 358，S-辨 不再拒那 358 个 `Zeros` 形，`research/prompts/unreadable-at-mount-r2-opus-output.md:173`「多拒 284 格：全是 `DeviceError`」）。数都是攻方副本上量的，不是入库装置上的数。
- 这 120 个格最少要 4 段读故障：「lost: fewest_fault_ranges=4; cells with that many (up to 12):」（`research/prompts/unreadable-at-mount-r2-opus-model/c331-summary-with-dev.txt:13`）；攻方的说法：「最短丢写反例 4 段读故障：两块盘最新那一槽读回全 0」（`research/prompts/unreadable-at-mount-r2-opus-output.md:175`，后面接「加 C 的根与它盘 0 那一份数据单元读不出」）。即两盘各一槽读回全零、最新根读不出、再加一份数据副本读不出。
- 条款已写明不罩的形：「每盘各坏一槽系统配置、同时最新根一时读不出的双故障形不在承诺里（用户 2026-09-28 定不罩）」（`@已定项 8：固定结构的放置`，D22）。120 个格是这个形再加一份数据副本读不出，是它的超集：不罩的形再加一处故障当然也不罩（推的，逻辑上的）。C331 甲与这一句是同一天（2026-09-28）的两个用户定案，甲 为不罩的形多付了 719 个多拒。
- 所以判决第 9 行的「丢写 0 → 120」，120 不在容错承诺里；S-辨 是把代码对齐到 D22 自己写的射程。判决说它「推翻用户 2026-09-28 的定案」，更准的说法是「收回 甲 买得比射程更多的那一部分」。
- 能不能让这 120 也回来而不付 719：不能（推的）。最新根与一份数据副本都读不出时，journal 记录那条路走不通：点名单元「两份都验过才施加」（`@已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换`「一个点名单元的两份（两条位置条目）都读得出、校验和都对才施加」），一份读不出就不施加，退到更早那一版。放宽成「任一份验过」是 C519 认下的另一读法，会不会放过 E77 的嫁接没量。「根优先」（岔路单第 7 行写的零轮）看的是根环里读得出的根，而这 120 里最新根正是读不出的，它也救不回（推的，没验）。
- V4 那一支：「甲 的取号扩展丢 0；甲-读错 的取号扩展丢 10（全是 `Zeros`）」（`research/prompts/unreadable-at-mount-r2-main-verification.md:49`），取号写的见证值也随 S-辨 变。

### 「一槽持续设备读错」S-辨 覆盖不了的那一格

甲-读错 对 `DeviceError` 仍判不出，所以一槽系统配置持续设备读错之后可写挂载仍永远拒（表第 46 行最后一列「永远只读」）。三条出路，都零轮：

| 出路 | 做法 | 这一格 | 代价（攻方量的数，副本上） |
|---|---|---|---|
| 甲-读错-每盘 | 每块盘都有一槽设备报读错才判不出 | 一盘一槽读错可写；两盘各一槽读错仍拒 | 丢写 120、多拒 98（全是两盘各坏一槽）；定义最小，改的是同一道判（`crates/singlefs-core/src/mount.rs:406`「| WitnessedCounterComparison::SomeSystemConfigurationSlotUnreadOrUnverified => true,」）的聚合方式 |
| 显式放行参数 | 正推腿 H4-c：默认逐字节不变，调用方放行后取号写重写缺的那一槽 | 放行后可写 | 要改两道拒（读阶段、取号读盘 `crates/singlefs-core/src/transaction.rs:645`「fn self_verified_system_configurations_of_every_device<Device: BlockDevice>(」），再加入口与用例；约 60–70 行加用例 4 条（正推腿 H4-c 各项推估的和，我没核） |
| 根优先 | 根槽 FUA 先于系统配置轮换，所选根可以绕开见证 | 没验 | 没数 |

我的判定：这一格要同时答「条款射程」：D22 已定项 8 写「每盘一槽系统配置坏在承诺里」，没写承诺含不含可写（正推腿 H4-d 的「规则没说」，我核同）。甲-读错-每盘 只拒「两盘各坏一槽」，字面上仍拒了一个「每盘一槽」的形；要不要认「读得回」就够，要用户定。我推荐的组合：S-辨 加 甲-读错-每盘（同一道判，改聚合），放行参数留着当第二道（它还管「两盘各一槽」的人为放行）。组合没被攻过，是零轮。

### 撕裂轮换之后怎么走出来（C596 的出口）

S-辨 之后「一槽撕裂、另一槽自证过」不再判不出，可写挂载往下走，取号写本来就重写那一槽：每块盘新世代号 = 自证过的最大世代号 + 1（`crates/singlefs-core/src/transaction.rs:370`「.unwrap_or(0)」那一段），槽位是世代号模 2（`crates/singlefs-core/src/transaction.rs:414`「(slot_generation % SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE) * spacing,」），落回撕裂的那一槽。所以 C596 的出口不用新写代码，只放开拒的判据（改两处，同上表）。C596 的验收（`.claude/kb/checks-owed.md:465`「一次可写挂载都做不成时怎么走出来，全仓没有条款」）：造撕裂轮换的盘面，断言之后可写挂载做成且确认过的写不丢；判别力自证：把读阶段改回「自证过的少于 2 就拒」必须仍然永远只读。

### 前提与要同步的

- 读数要分「设备读不出」与「读得出、自证不过」：今天的读数把两种都丢掉只交自证过的（正推腿 H4-a 的观测，我核到函数是 `verified_system_configuration_slots`，`crates/singlefs-core/src/mount.rs:51` 引入）；新读数约 15 行（正推腿的推估）。
- 记进变更史：D23 已定项 14 射程那一条「C331 取甲的代价」要改写，C596 行与判决表一起；用户 2026-09-28 的甲 被收回的那一部分要写成「收回」。
- 推翻：入库装置上重量 120 里有一格故障范围少于 4（那就在承诺里了，要重新交用户）；或造出一份「一槽撕裂、另一槽自证过」的盘面，S-辨 放开后取号写之后那一槽仍读不出自证过（观测 3 的世代号推错）。

## 六、G4：清扫器与删行

### 条款今天写了什么、缺什么

回收条件已经有条款：「整轮清扫对池中每一个落点都得出了判定（读成功并按行判、已抹头、已被覆盖）」（`.claude/kb/decisions/18-块里携带什么信息.md:313`），再加「那次清扫里没有任何一次读失败」「全部设备在线」「根环里没有该实例发布的根」。C597 缺的是三件：谁删、落在哪次发布、崩在中间怎么办（`.claude/kb/checks-owed.md:466`「每次可写挂载写一行实例表，今天没有删行代码」那一行末列「删行条件的条款（回收条件、谁删、崩在删行那次发布中间怎么办）」）。下面把这三件加上读什么、何时跑、多久，写成最小形态。

### 最小形态（推的，按 `crates/` 今天的实现；没造）

1. 读什么：池里每块盘、单元区每个槽的头；根环全部 3S 个槽；实例表链。槽 16 384 字节（`crates/singlefs-format/src/lib.rs:14`「pub const SLOT_BYTES: u64 = 16384;」），每槽读头 4 096 字节。逐槽按已发布谓词判：「i < i_now 且无行 ⇒ 已发布」，有行 (i, T_pub, W) 时单元的诞生代号 b ≤ T_pub 或写序 n ≤ W 才算已发布（`.claude/kb/decisions/18-块里携带什么信息.md:312`）。没发布的单元就是要抹头的对象：写一个非单元头的字节进去（FUA，一个扇区内、两态）。
2. 多久：我的算术（不是量的，只是槽数与读数）：
   ```
   $ python3 research/prompts/m3-ring-r2-sonnet-model/g1_g4_arithmetic.py | grep TiB
   1 TiB 一块盘：67108864 个槽；每槽读头 4096 字节共 256 GiB；两块盘各读一遍共 134217728 次读
   4 TiB 一块盘：268435456 个槽；每槽读头 4096 字节共 1024 GiB；两块盘各读一遍共 536870912 次读
   ```
   每 TiB 1.34 亿次 4 KiB 读，按每秒 50 万次随机读算约 4.5 分钟（假设值，要实测），大盘按线性涨。它不能放在挂载路径上：挂载握着独占打开与取号，加几分钟到几十分钟不可接受（推的）。
3. 何时跑：独立入口，管理员或后台触发，在一次可写挂载的会话里、不在取号与写行之间。清扫结果只活在这个会话里（不落「已清扫」标记）：同一会话是独占的、旧实例的单元不会再多出来，所以清扫之后到删行之前的时间窗里证明不会失效；会话里自己的写只产生当前实例的单元，不影响旧实例行的判定。
4. 读失败、设备不全：把条款原样当判据，一票否决整轮：
   | 条款里的一项 | 清扫器的做法 |
   |---|---|
   | 池中每一个落点都得出判定 | 逐槽读成功；任一槽读失败（设备报错）整轮作废、一行不删 |
   | 没有任何一次读失败 | 同上，重读一次仍失败才作废（照读阶段「重读一次」的口径，`@已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换`），并计数上报 |
   | 全部设备在线 | 降级挂载（少于池里全部设备）不清扫 |
   | 根环里没有该实例发布的根，每个槽都读成功 | 设备读不出的槽 ⇒ 作废；读得出而自证不过的槽：全零按「没有」，非全零按「可能有」（规则没说，我提的读法，理由：全零 = 从没写过或被抹，非全零自证不过 = 可能是一条被误读的根；第二节的抹写全零，与这一条口径衔接）。读环的两支今天已分开：「RootRingSlotReading::Bad(BadRootRingSlotReading::Unreadable) => continue,」（`crates/singlefs-core/src/recovery.rs:1096`）、「RootRingSlotReading::Bad(BadRootRingSlotReading::NotSelfVerified) => {}」（`crates/singlefs-core/src/recovery.rs:1097`） |
   代价：一块盘上有一个持续读不出的扇区，整池永远不删行（C335 已写明：「代价是一个根槽持续读不出时行永远删不掉」，`.claude/kb/decisions/18-块里携带什么信息.md:313`；量词取「池中每一个」是条款有意选的）。我不建议在这一轮放松成逐行判。
5. 删行那一写落在哪：落在一次专用的清扫发布上，不是写行那次。写行那次发布发生在挂载开头，清扫结果这时还没有（清扫在会话里、之后才跑）；所以正推腿 H5-c 的「并进写行那次发布」要改：清扫完成之后发一次实例表重写发布，把证明可回收的行滤掉。今天 `InstanceTablePlan::Rewrite(` 的构造只有写行那一处（`crates/singlefs-core/src/mount.rs:2801`），要新加一个构造点；行的拼法在 `crates/singlefs-core/src/mount.rs:1065`「fn instance_table_rewrite_of_the_row_publish(」，删行是在它拼出的 `rows` 上多一道过滤。准入读的是同一个函数的输出，行数减了切换预留自动减。这次发布本身要空间（整条链 COW 重写），清扫发布前先过准入，不够就不发、行留着。
6. 崩在中间：
   | 崩在 | 盘上 | 下一次挂载 |
   |---|---|---|
   | 抹头写进行中 | 每个槽两态：抹了 / 没抹（头在一个扇区内） | 没抹的下一轮再抹；抹了的算「已抹头」，单元本来就没发布、分配器认它是空闲槽 |
   | 抹完、清扫发布前 | 行都在、单元已抹 | 行还在，下一轮清扫什么都不用抹，直接发 |
   | 清扫发布：单元与记录落了、根槽没落 | 新链的单元是当前实例的未发布单元，旧链与行原样 | 记录属于当前实例的前一实例（取号后变成上一个实例），所选根是同一实例的更早一根 ⇒ 记录会被施加，施加出来的实例表指针照所选根（旧链，`crates/singlefs-core/src/recovery.rs:3169`），等于「删行没发生」，一致 |
   | 清扫发布落盘之后 | 行已删 | 被删的实例在根环里没有根、单元已无，被抛弃根重新可选的路径不存在（H5-a 两个消费者都挡住） |
   一个必须钉的次序：抹头全部 FUA 返回、屏障之后才发清扫发布；否则「行已删、单元仍在」会让那些单元按「无行 ⇒ 已发布」被当成已发布。

### 第一轮攻方「(j, 0, 0) 行正是删不掉的那一类」：不成立

原话：「连崩挂载留下的 (j, 0, 0) 行正是删不掉的那一类」（`research/prompts/m3-ring-r1-opus-output.md:53`）；理由是 j 必有写行那次发布写下、没发布的单元，条件前一半管的正是它。

- 行怎么来：写行时给 [所选根的实例, 新实例) 中间没发布过的实例各写一行：「严格介于所选根的实例与新实例之间的实例写 (i, 0, 0)」（`.claude/kb/decisions/18-块里携带什么信息.md:309`）。
- j 的未发布单元是哪些：C533 的用例核过：写行那次发布的两个单元（实例表与分配记录树，后者五个节点）一共六个，崩溃镜像上两块盘都有（`crates/singlefs-harness/tests/row_publish_record_without_its_root.rs:6`「下文说的「那两个单元」指实例表与分配记录树」起的一段）；它们「不在任何一条分配记录里、是空闲槽」（`crates/singlefs-harness/tests/row_publish_record_without_its_root.rs:14`），下一次挂载写行取的落点「与崩掉的那次逐槽相同，把它们盖掉」（`crates/singlefs-harness/tests/row_publish_record_without_its_root.rs:15`）。条款里「已被覆盖」本来就算判过的落点（D18:313）。
- 没被盖掉的由清扫抹头，抹头也算判过。(j, 0, 0) 在根环里没有 j 的根（j 的第一次发布根槽没落盘），「根环里没有该实例发布的根」天然成立。
- 所以这一类是最容易删的：没有根、至多一次发布的几个单元、多半已被下一次写行覆盖。真让行删不掉的是任何一个读失败或根槽读不出，对所有行一样，不分 (j, 0, 0)。攻方这一句的链条漏了「抹头算判过」这一步。推翻：造一份连崩 n 次的盘，清扫后某个 (j, 0, 0) 的单元仍残留且抹不掉（比如落点在读不出的槽上）——那是读失败，整轮作废，属于上面表里已写的一项，不是 (j, 0, 0) 独有。

### 做不做的代价（用户已定做；数照旧）

不做：10⁴ 次挂载 28 片、10⁶ 次 2 711 片（我重算同数，`/tmp/claude-1000/m3-ring-r2-sonnet/arith.out`），每次写行重写整条链。做：新增一个读池的清扫入口、一个专用发布、一个核心里的已发布谓词（今天谓词只在 checker 的扫描重建里，core 里没有：正推腿 H5-b 的 `grep` 在 `crates/singlefs-core/src` 零命中，我没重跑）；这是第二个实现：谓词在清扫器里与 checker 里各写一份时，要有一条互相核对的用例（推的）。8 条登记的崩溃流都从 mkfs 起、行数个位数，删行分支走不到（`research/prompts/m3-ring-r1-facts.md` 第 (g) 节），要加一条多于一圈发布的多实例流才触发。

## 七、替 P2、P3 与 K2 / K3 的采法辩护

先说判决这一步的规则位置：判决第 1 行把 K2、K3 当作分辨臂的依据采用，而它们「不在跑前判据字面里」（`research/prompts/m3-ring-r1-main-verification.md:26`）；攻方自己也写「K2、K3 分辨得出臂，但不在跑前判据的字面里」（`research/prompts/m3-ring-r1-opus-output.md:283`）。规则的最后一步是「四句都过，才按跑前条款判」（`.claude/singlefs-ai-sop/rules/evidence-discipline.md:126`）：第 3 句答不出「哪个分句」，没有可套的跑前条款。所以采 K2、K3 是主 agent 的取舍，规则没推出它；交用户时应标「依据在跑前判据之外」。这不推翻 P1，只改它的名分。

### P2（补洞并进写行那次发布）能站住的理由

- 挂点 2（根前、分配器后记）：K3 是真的，且是可达状态：21 个前缀截点红：「point=P2BeforeRowRoot cuts=73 red_at_crash=21」（`research/prompts/m3-ring-r1-opus-output.md:191`）。但攻方自己写了红的性质：「账多记是漏放的方向，不是复用」（`research/prompts/m3-ring-r1-opus-output.md:199`），它是 I-3.1 的口径问题（账按补洞之前的门槛算），不是安全问题；攻方给的推翻条件之一就是「I-3.1 的口径改成认写这条根时的门槛」（`research/prompts/m3-ring-r1-opus-output.md:308`）。辩方判定：站得住的是「这一红能靠口径或把分配器的记法挪到写行之前消掉」，挂点 5 就是挪法，前缀截点 0 红（`research/prompts/m3-ring-r1-opus-output.md:278`）。挂点 5 的代价是两条推的：门槛先抬的窗口、记录核对器按位置认发布判红 58983 态（攻方也标「推的」）。第一条要在带文件后洞全量里答，第二条是核对器按位置认发布的写法问题，P2 根前要连它一起改。
- 判决不能说「P2 打中 K3 所以出局」：在跑前判据里 K3 没有格；说「P2 的两个代价比 P1 的一个多」才对。
- 攻方的「枚举」里挂点 2 与 1 的差很小：补洞之后同一处只剩带着不可补洞 19 的那几个状态红，挂点 1 是 3 个、2、3、5 是 4 个、4 是 14 个（`research/prompts/m3-ring-r1-opus-output.md:203`）。这是另一段历史上的数，与 21 个截点不是同一个域（「前缀截点是枚举域的子集，打中的是可达状态，没打中不算证据」，`research/prompts/m3-ring-r1-opus-output.md:158`），不能相减，只能说明 P2 与 P1 在全量枚举上的红只差一个状态、P3 才拉开。

### P3（暖机之后）能站住的理由

- K2 对 P3 是一整段窗口：写行那次根已落而补洞写没落的状态，下一次挂载择到的根越过了洞，洞永远留下：「P3 是一整段」（`research/prompts/m3-ring-r1-opus-output.md:179`）。P3 站得住的唯一理由是「永远留下」的损失可以由第二节的抹在下一次挂载清掉：那时洞留下的是一个残留根 s，抹按「被跳过的 txg 的槽」处置，与 K1 同一个处置，不需要补。也就是说，K2 的代价只有在「补是唯一的清洁手段」的前提下才大；有了抹，K2 对 P2 根后、P3 的咬合变小。这条是推的，没造。
- K3 对 P3 是静止盘面红（挂载做完那个盘面自己红）：这一条抹救不了，因为是补洞写落盘之后最新根的账没有新门槛的事。P3 在这一条上站不住，我不替它辩。

### 第三条路：只抹不补（零轮，没量）

抹是第二节的机制。把它用在所有洞上（包括可补的）就是「只抹不补」：不用往根槽写出有效根，也就不用 G1 那两位。差别：
- 洞那一版（`effective_root`）没有根：I-3.9 的区间不收窄、C380 的对象不缩（补了才收）；回退候选少一条（洞那一版自己）。
- 挂载刚做完时放出的槽与补洞相同：「挂载刚做完那一步，窗口门槛放出的与补洞放出的相同」（`research/prompts/m3-ring-r1-opus-output.md:112`）。
- 不改格式、不改 checker、不改字节表。
它不是我推荐的结论，是给主 agent 的对照：用户在第四节第 1 条选了「补」，前提是补出错根要加字段；第三节已指出前提只剩两位，补的代价小得多，补仍然占优（I-3.9 收窄、候选不少）。若主 agent 认为连两位也不想动格式，「只抹不补」是退路。

## 八、给主 agent 的前提一览（G1–G4 与第四节的辩方那一行）

| 格 | 代价 | 前提（不成立就改写） |
|---|---|---|
| G1 | 借两位，头 311、点名项 67 不变；要同步 D23 已定项 4、15、layout 记录标志行、`journal.rs` 标志位、三处造记录、施加两行、层 0 首个文件与卸载流、记录核对器新加一条 | 写行记录走不到（C533、`lib.rs:1265`）；要验写行那一种用 C533 用例的路径再看一次 effective = chosen |
| G2 | 三处门槛、checker 候选集、D16 / D23 / I-7.4 条款、重跑计数模型；放出 366 / 174 槽（攻方度量） | 窗口的 n 取盘上读到的有效根最大 txg，不取 `first_txg − 1`；只数有效根；否则 K1 历史上有合法状态让候选集引用已复用的单元（推的） |
| G3 | 读数分两种、判据改两处；丢写 0 → 120 | 这 120 都 ≥ 4 段读故障、落在 D22 已定项 8 的不罩形里；一槽读错那一格用 甲-读错-每盘（零轮）或放行参数 |
| G4 | 读池一遍每 TiB 1.34 亿次读（算术）、一个专用清扫发布、核心里的已发布谓词 | 清扫不在挂载路径上；抹头 FUA 先于清扫发布；根环全零槽按「没有」、非全零不自证的按「可能有」（规则没说，要用户定） |
| 第 1 条补三种洞（用户定） | 同 G1；若坚持三字段：头 487、点名项 64、跨记录起点 23 616 行 | 三字段里实例表指针没有消费者（需另改 D23 已定项 14 注 1） |
| 第 2 条一圈窗口（用户定） | 同 G2；另有抹这条零轮的对照 | 同 G2；交用户时标「抹」是第三条路、没被攻过 |
| 第 3 条 S-辨（用户定） | 同 G3 | 同 G3 |
| 第 4 条现在做（用户定） | 同 G4 | 同 G4；还要核对 C333 是否过期（第一轮判决第 12 行） |

## 九、什么现象会推翻这些结论（汇总；各节已写的不重复）

- 「P1 唯一」够不着挂点 5：带文件后洞（4 325 419 态）全量里挂点 5 出现 I-7.4 红则 P1 理由补全；零红则 P1 只剩「更省事」。
- 写行那一种是死分支：造出 effective_root 的 txg 大于 chosen_root 且那条记录是新实例第一条的盘面。造出来，G1 的实例表指针要回到字段表里。
- 「抹」：K1 历史上抹之后仍有合法崩溃状态让候选集引用已复用的单元，或抹掉的根在合法历史里是回退要用的；或抹造成的「读成功、自证不过」槽被 C597 的回收条件误判。
- G2 窗口前提：K1 历史上「挂载 → 第一次发布单元写完、根槽没写」逐前缀扫，n 取 `first_txg − 1` 的版本 I-7.4 绿，或 n 取盘上最大有效根的版本 I-7.4 红。
- G3：入库装置上重量那 120 里有一格故障范围少于 4。
- G4：造出连崩 n 次的盘，清扫抹头后某个 (j, 0, 0) 的单元仍残留且不是读失败造成的。

## 没做什么

- 没编译、没跑任何测试与门禁，没造盘面：第二节的抹、第四节的反例形态、第三节的字节相等都是读代码推的，已逐处标「推的」。
- 没重量攻方副本上的数（350 / 158 / 366 / 174、21 个截点、58983 态、120 / 284 / 98）；它们不是入库装置上的数。
- 没读第二轮攻方腿的任何产物（`research/prompts/m3-ring-r2-opus-output.md`、其模型目录、`/tmp/claude-1000/m3-ring-r2-opus/`、`/tmp/claude-1000/m3-r1/handover/ring-r2-attack.md`）。
- 没核 `singlefs-replay-prefix` 的 Verus 证明是否碰标志位两位；没核层 0 固定脚本流里哪几条含首个文件发布与卸载那一串；没核 `layout/01-first-txn.md` 记录标志行的全部取值。
- 没替主 agent 撤销或维持任何判决，只交复核结果。
- 没验第 G4 节「每秒 50 万次随机读」这个假设值。
- 落盘产物：本报告；`research/prompts/m3-ring-r2-sonnet-model/g1_g4_arithmetic.py` 与同目录 `g1_g4_arithmetic.out`（纯算术，sha256 见交回）。草稿目录 `/tmp/claude-1000/m3-ring-r2-sonnet/` 只有 `arith.out` 一份，没建编译目录或仓副本。
