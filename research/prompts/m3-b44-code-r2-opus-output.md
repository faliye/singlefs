# m3-b44-code-r2 攻方腿（Opus）报告（2026-10-17）

攻击面：正文「三、格」J1、J2、J3、J4，J5 只顺带一条。量法全在冻结副本的拷贝 `~/.local/share/singlefs/work/m3-b44-code-r2-opus/repo/` 上（冻结副本本身没动）；
这一节以下引代码的行号是冻结副本的，四份被判文件与 `pointer.rs`、`journal.rs`、`fault_injection.rs` 和主工作区此刻逐字节相同（sha256 前 16 位逐份比过）。副本上量出的数只算副本的数。

## 复跑

```
bash scripts/hosts/run-with-memory-cap.sh 24G bash research/prompts/m3-b44-code-r2-opus-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs \
  --frozen /home/fy5090/.local/share/singlefs/work/m3-b44-code-r2/repo \
  --work <新的草稿目录>
```

它把冻结副本拷进草稿目录、把原型放进 `crates/singlefs-harness/tests/`、编一次（`capped.sh default`，debug 档，singlefs-core 与 harness 关调试信息；本机这一次编 5 分 31 秒），
再跑四样：J1 坏记录、读故障窗口加之后两次重挂、写账字节、两段窗口扫描（起点 330–350 宽 2 与 6，945–1020 宽 2）。完整的 1–1072 扫描用分片跑，命令在「这条腿自己的限度」一节。

模型目录 `research/prompts/m3-b44-code-r2-opus-model/` 每个文件的 sha256：

```
a789e13185f95ae83882fe6d1a5deb3a836c4d65e6ac59a5cea86eb7287c5516  ./attack_mount_rehearsal_probe.rs
c68f63f403a1f66adfd892e8ef2371717df943c07da7496e9548f67d9ec0b5ed  ./out/b1-1.log
78eac3ba15a8c0ffbafa7f8a28c6b1f0d3f4f746055d0a54c1565196ae03600e  ./out/b1-2.log
8ca9dadad0e10e1146d4889097e725df49f42766232b00b093bd3ee09f8225f2  ./out/b1-3.log
ed6e5eca39b99d27d1bfac8393795f73afd825522a45aa2d2f2c061716e33c88  ./out/b1-4.log
f7bf2fcb97d07b81e3f826a54d221fbee389a408144a2b4f15e5c0b2b32a1527  ./out/b2-1.log
b21cb39d88a997d120cf6c91b0c3028c0af3873c88d4e92a81d8a05d59578172  ./out/b2-2.log
db894c3ed5ddb5b5013a82e05ba970bd1b0ca9f53f6a44e5ea89466ae27bec89  ./out/b2-3.log
8223d44068ffd93ba69ebec028464360c0ba838d047c159cb70cd61f9d854ad7  ./out/b2-4.log
3577f48a560f1c39e69ff8a180c945110f827ae2f3048bfe31ea5800889f2a88  ./out/b2-5.log
d1689c9af1b5672f31264be58e6d675eb52587dc077514d219eebb60e396cdab  ./out/b2-6.log
1cf770e77b28754a7f1b35e88f88da86dfaac69e8aea80bbb903549539612a2a  ./out/b2-7.log
6a9b6fb080c1cdb3be7da0183cdeefd703418892233848d4500797cf0d7922bc  ./out/b2-8.log
bb5430f0b7034bb01e01b71f86c5a8d65cab6d036ed488ccd370bf9f24674cea  ./out/run1.log
8e78979a38129a610de9d78cc1d3cec32245b0386ddfce357fcf3fdb40f2c315  ./out/run-bytes.log
f08e3140e773dcedee07a165566b203a46eb641ead8d64ffea925a0e4ba39627  ./out/run-followup.log
27476b266d986c84f05af06d7293a8cf0c849ea5ee9ee61a84f737d4e4ea1fe0  ./out/run-j1-bt.log
96aab63127973d5b9c415608cb7a3de68eb01177aa4d41f22ec253fa42c9019b  ./out/run-j1.log
f1e13cc87c456ba40328b47777bf1dc246aaecc92df0c616d77472b6227a9d1f  ./out/window-hits.txt
2296532a18f140ba7c42eeb192eb8281c957cf665ea378aead0d9d701de072d1  ./tools/rerun.sh
```

`out/b1-*.log` 是单次读错的扫描（四片），`out/b2-*.log` 是读错窗口的扫描（八片），`out/window-hits.txt` 是从 b2 里抽出的打中的起点与宽，
`out/run-j1*.log` 是 J1 坏记录（带与不带调用栈），`out/run-followup.log` 是窗口加重挂，`out/run-bytes.log` 与 `out/run1.log` 是写账字节与小段计时。

## 各格判定一览

| 格 | 判定 | 一句话 | 证据 |
|---|---|---|---|
| J1 | 打中（清单漏一类，panic） | 所选那一版的那条 journal 记录新根段里四条指针不在 I-2.5 那一判里；重建在判之前先把这条记录重新编码，写者断言 panic。可写挂载 panic，只读挂载照常 | `out/run-j1-bt.log` |
| J2 | 打中（预演放过、真做取号之后交 C841、盘变了） | 取号之前那一趟预演里有两次读报错（同一单元两份镜像都没读出、过后又读得出），预演当作「别的错」放行；真做取号、写 19 次、走到最后一批才交 C841。之后不带故障再挂，取号之前拒 | `out/b2-*.log`、`out/run-followup.log` |
| J3 | 打中两样，另给一个量 | 同一个读错窗口在没改坏的镜像上被预演吃掉：真做那一趟一次读错都没见、盘与无故障那一次逐字节相同；只进内存的盘只增不删，内存随续做步数线性涨（每步实测约 0.41 MB，大文件的数是推的） | `out/run-followup.log`、`out/run-bytes.log` |
| J4 | 没打中 | 查了 8 处按 `MountError::Publish` 分路的调用方，空写账在每一处都走得通；有一处只在点名了取号屏障时成立，它的池走不到这一拒（推的） | 「J4」那一节 |
| J5 | 顺带一条 | `walk_the_continuation_after_acquisition_on_devices_keeping_writes_in_memory` 的文档说「续做报的错就是真发时会报的那一个」「真发时报它自己的」，J2 那一格两句都不成立 | 「J5」那一节 |

打中的三格各按四句过了一遍（J1 那一节、J2 与 J3 那一节的「四句」）。改法全是我提的，只在我的模型上量过打中的那几格是不是还在、改法本身都没实现，被攻过零轮。

## 一、J1：重建时 I-2.5 那一判的指针清单

### 打中：所选那一版的那条 journal 记录，新根段四条指针

造法（`attack_mount_rehearsal_probe.rs` 的 `swap_the_location_entries_of_the_tree_table_pointer_in_the_record_of_the_newest_root`）：
`common::build_pool` 起的池（新池新建文件之后），取最新那条根；两块盘的 journal 环里找同实例、同 txg、带「本次发布末条」标志的那条记录
（两块盘各一份，都在计数 3），把它新根段里树表指针的两条位置条目对调（设备 1 在前、设备 0 在后），重算记录头的 32 字节宽校验和。
载荷校验和只罩点名项，不用动；根槽、单元一字不改。改完 `JournalRecord::parse` 照样收下（`out/run-j1.log` 原样）：

```
device DeviceIdentity(0) counter 3: pointer at 99, reparsed=true devices_in_order=Some([DeviceIdentity(1), DeviceIdentity(0)])
device DeviceIdentity(1) counter 3: pointer at 99, reparsed=true devices_in_order=Some([DeviceIdentity(1), DeviceIdentity(0)])
journal records changed: 2
thread 'journal_record_whose_new_root_segment_pointer_does_not_ascend' (1166764) panicked at crates/singlefs-core/src/pointer.rs:325:9:
writable mount: Err(Any { .. })
read-only mount: Ok("Ok(\"mounted read-only\")")
```

调用栈（`out/run-j1-bt.log`，`RUST_BACKTRACE=1`，抄 singlefs_core 那几帧）：

```
位置条目按设备身份严格升序（I-2.5），整条全零的空指针（empty_root）豁免：盘上读来、要照抄写回的指针在 recovery::rebuild_version 判过
   2: <singlefs_core::pointer::NodePointer>::write_to
   3: <singlefs_core::journal::JournalRecord>::to_bytes
   4: singlefs_core::recovery::rebuild_version_in_the_unit_area_starting_at
   5: singlefs_core::mount::rebuild_previous_version::<singlefs_harness::memory_pool::SparseBlockDevice>
   6: singlefs_core::mount::mount_writable_with_every_test_only_switch::<singlefs_harness::memory_pool::SparseBlockDevice>
```

一步一步许可它的那几句：

- 重建先判根记录的四条指针：「root_record_pointers_ascend_by_device(root)?;」（`crates/singlefs-core/src/recovery.rs:2535`）。根没改，过。
- 接着取那条记录：「record_standing_for_root.ok_or(RebuildVersionFailure::NoRecordStandingForFileVersion)?;」（`crates/singlefs-core/src/recovery.rs:2548`）。
  记录怎么取：同实例、同 txg、带末条标志，取不到就「.or_else(|| records.values().max_by_key(|record| record.counter))」（`crates/singlefs-core/src/mount.rs:7432`），两条路都不比记录的新根段与根是不是同一组指针。
- 然后立刻重新编码：「let record_bytes = record.to_bytes();」（`crates/singlefs-core/src/recovery.rs:2549`），编码新根段时头一条写的就是
  「self.new_tree_table.write_to(&mut writer);」（`crates/singlefs-core/src/journal.rs:245`），写者断言「self.location_entries_ascend_by_device().is_ok(),」（`crates/singlefs-core/src/pointer.rs:326`）。
- W8 并进清单的那一判在七百多行之后：「pointers_of_the_rebuilt_version_ascend_by_device(&rebuilt)?;」（`crates/singlefs-core/src/recovery.rs:3259`），清单里本来也没有 `output.record` 的新根段。
- 那一判自己的文档说的是「重建出来的这一版里每一条盘上读来的指针按 I-2.5 判」（`crates/singlefs-core/src/recovery.rs:5132`）；I-2.5 的射程是「任一块指针、中央映射条目的 value、journal 点名项里的位置条目数组按设备身份严格升序」（`crates/singlefs-checker/invariants/04-校验和.md:10`）。

这一格不是 W8 引进来的：`record.to_bytes()` 在 W8 之前就排在判之前。J1 问的是清单全不全，这一类就是漏的那一类。

四句：

1. 分不分辨臂：J1 只有一条臂（W8 的清单）。打中的是清单少一类，不是各臂共用的前提；改法（下面）能把它和现状分开。
2. 系统当时看不看得到：看得到。记录在 `crates/singlefs-core/src/recovery.rs:2548` 已经解出来，新根段四条指针就在 `record` 里，判一次升序就够。
3. 满足的是哪一个分句：正文 J1 那一行「并进清单的指针全没全（这一版里还有哪类节点指针没进这一判）」（`research/prompts/_m3-b44-code-r2-body.md:30`）。
   与派发验收「之后某次写在 `pointer.rs` 写者断言上 panic」的字面差一点：断言是同一句，触发不在之后的写，而在重建里重新编码那一步，比写更早。
   同一行另一分句「坏镜像在任何写之前交类型化错误」也不成立：交的是 panic。
4. 改法在这一格上还中不中（都是推的，没实现、没跑）：
   - 只把 `record` 的四条指针并进 `pointers_of_the_rebuilt_version_ascend_by_device`：**不修**。那一判在 `crates/singlefs-core/src/recovery.rs:3259`，panic 在 `crates/singlefs-core/src/recovery.rs:2549`，碰不到。
   - 在 `crates/singlefs-core/src/recovery.rs:2549` 之前判 `record` 新根段四条指针升序（全零豁免照 `NodePointer` 那一份），不过交 `InvariantViolated { invariant: "I-2.5", .. }`：修。可写挂载在任何写之前拒，只读挂载不受影响（它今天就照常挂上）。
   - 不重新编码，`record_bytes` 取环里读来的原字节：修 panic，但坏指针留在内存里的 `record` 上。后面还有没有别处再编码它我没查完，这条比上一条弱。

### 查过、没打中的几类指针

逐个找了 core 里 `NodePointer::write_to` / `DataPointer::write_to` 的调用处，看每一处写回的指针从哪来：

| 类 | 写回处 | 在不在清单里 | 结论 |
|---|---|---|---|
| 树表条目根、extent 树上下段、数据指针、inode 叶容器、分配记录树节点、记账树、中央映射树 | 原清单 | 在 | 不攻 |
| 墓碑、谱系、退役三棵容器树的容器指针与根之下节点指针 | W8 并进的 | 在 | 回退的目标那一版另走 `rebuild_version_in_the_unit_area_starting_at`，同一判；回退照抄的容器树取现行那一版，判过 |
| 根记录四条 | `crates/singlefs-core/src/recovery.rs:2535` | 在 | 不攻 |
| journal 记录新根段四条 | `crates/singlefs-core/src/recovery.rs:2549` | **不在** | 打中，见上 |
| 实例表链指针（第 1 片起） | 不照抄：重写整条链，下一片的指针是新取的（「Some(next_page) => InstanceTableChainRecord::NextPage(next_page),」，`crates/singlefs-core/src/transaction.rs:1783`），旧链只拿去释放 | 不在 | 不 panic；I-2.5 在 core 里没人判这一类（池级 checker 判） |
| 中央映射条目的位置条目 | 「location.write_to(&mut writer);」（`crates/singlefs-core/src/records.rs:394`），`LocationEntry::write_to` 只断言宽度 | 不在 | 不 panic；逆序不判，同设备那一种在发布时报 `MappingEntryLocationsOnTheSameDevice` |
| journal 点名项 | 「location.write_to(writer);」（`crates/singlefs-core/src/journal.rs:106`），同上 | 不在 | 不 panic；I-2.5 写了它，core 不判 |

误拒那一面：全零空指针的豁免读者与写者是同一份；树里没有容器的一版那几棵容器树给 `ContainerTreeIndexNodesVersion::zero_root`，根之下指针为空；池恒两块盘（`refuse_locations_that_do_not_name_two_pool_devices`）。
没造出合法写路径写出、而这一判拒的镜像；harness 里现有的挂载用例在合入验证时全绿（正文「一、这一轮要判什么」），我没另跑。

补一句：上表「池恒两块盘」是按函数名 `refuse_locations_that_do_not_name_two_pool_devices` 推的，没读它的正文。回退目标那一版重建时同样先 `record.to_bytes()`
（`roll_back_by_a_forward_publish` 经 `rebuild_version_in_the_unit_area_starting_at`），目标那一版的记录新根段逆序时挂着回退也会走到同一句断言：推的，没造盘面。

## 二、J2 与 J3：取号之前预演整个续做、抬 F 也预演

### 装置

镜像照现有用例 `opening_a_container_past_the_largest_number_returns_an_error.rs` 窄盘那一条的造法（不补环窗口的窄盘单元区 304 槽、第一个文件 16 个数据单元、
删除队列 {1}、一批一个单元、退役容器一片 1 条），造法拷进我的原型，没改。两份：

- 改坏的：退役树唯一那片的号改成 `u64::MAX`。无故障挂一次：取号之前拒、盘不变，整次挂载读 1070 次（`out/run1.log`：`corrupted fault-free outcome=C841-before-acquisition total_reads=1070 …`）。
- 没改坏的：无故障挂一次做成、续做 16 步走到空、中间推过抬 F；取号之前读 1159 次，整次读 3104 次（`out/run1.log`：`unchanged fault-free outcome=Ok total_reads=3104 reads_before_the_first_write=1159 fault_free_changed_disk=true`）。

盘是 harness 的 `SparseBlockDevice`（内存稀疏盘），外面包一层读故障。两种故障：

1. 单次读错：harness 的 `FaultSchedule::the_nth_call_across_the_pool(InjectedFault::ReadFails, n)`，n 取 1 到 1070，每个 n 一份新拷贝的盘挂一次。
2. 读错窗口：原型里的 `WindowedReadFailure`，挂载开始之后整池第 s 到第 s + w − 1 次读报块设备错（两块盘一起数），之后照常。s 取 1 到 1072，w 取 2 与 6。

### 单次读错：没打中

1070 个 n 全部「取号之前拒、盘不变」，读错每次都有一次真注入（`out/b1-*.log` 汇总，四片各 268/268/268/266 个 n）：

```
corrupted summary 268 × C841-before-acquisition disk-unchanged before-first-write fired=1
corrupted summary 266 × C841-before-acquisition disk-unchanged before-first-write fired=1
```

单元两盘各一份，一份读错读另一份，预演照样走到最后一批。

### 读错窗口：打中

2144 次挂载（1072 个起点 × 2 种宽），汇总（`out/b2-*.log` 各片 summary 行按键相加；这一张表是我用 awk 加的，每片原样在文件里）：

| 次数 | 宽 | 结局 | 盘 |
|---|---|---|---|
| 1004 | 2 | 取号之前拒 C841 | 不变 |
| 958 | 6 | 取号之前拒 C841 | 不变 |
| **62** | 2 | **取号之后交 C841，已落盘 19 次发布** | **变了** |
| 58 + 4 + 3 | 6 | `SystemConfigurationUnreadableAfterTheReadStage` | 不变 |
| 36 + 3 | 6 / 2 | `Recovery` | 不变 |
| **8** | 6 | **取号之后交 C841，已落盘 19 次发布** | **变了** |
| 其余 8 | — | 窗口落空（4）、`InstanceTableMalformed`（2）、取号之前拒而注入数不等于宽（2） | 不变 |

打中的 70 个（起点/宽）在 `out/window-hits.txt`：起点 335–345 一簇、951–1016 一簇。改坏的那一份上无故障挂载的 1070 次读全在取号之前，两簇都落在这 1070 次里，
而打中的那几次盘都变了，说明窗口吃在取号之前那一趟、真做那一趟没撞上它。

拿起点 340、960（宽 2）各接两次不带故障的重挂（`out/run-followup.log` 原样，`cause=` 之后截掉）：

```
corrupted start=340 width=2: first mount C841-after-writes(persisted=19) cause=NextContainerNumberPastTheTopOfItsRange(… rightmost_container: 18446744073709551615 }) failed_reads=2 disk_changed=true
corrupted start=340: second mount without fault C841-before-acquisition … disk_changed_by_it=false
corrupted start=340: third mount without fault C841-before-acquisition … disk_changed_by_it=false
unchanged start=340 width=2: Ok instance=InstanceGeneration(2) continuation_steps=16 outcome=WalkedToEmpty failed_reads=2 disk_same_as_fault_free=true
corrupted start=960 width=2: first mount C841-after-writes(persisted=19) … failed_reads=2 disk_changed=true
corrupted start=960: second mount without fault C841-before-acquisition … disk_changed_by_it=false
unchanged start=960 width=2: Ok instance=InstanceGeneration(2) continuation_steps=16 outcome=WalkedToEmpty failed_reads=2 disk_same_as_fault_free=true
```

（文件里每行 `cause=` 后面是完整的错；上面用「…」省掉了重复的那一截。）

一步一步许可它的那几句：

- 分配器拷贝上那一趟空间不够，转去只进内存的盘：「Err(refusal) if refusal_is_short_of_space(&refusal) => {」（`crates/singlefs-core/src/mount.rs:5276`）。
- 只进内存的盘读没写过的块穿到底下那块盘：「self.disk.read_at(」（`crates/singlefs-core/src/mounted_session.rs:410`）。窗口里的读错原样交回。
- 起点 335–345 一簇：内存里那一趟取号读不出，`acquire_then_publish_until_the_continuation` 交错，被「.ok()?;」（`crates/singlefs-core/src/mount.rs:6980`）吞成 `None`。
  宽 6 时同一处在真做那一趟也撞上（`SystemConfigurationUnreadableAfterTheReadStage`，盘不变），宽 2 时只撞预演。起点落在哪一步是按这两簇在读序号上的位置推的，我没逐次打调用栈。
- 起点 951–1016 一簇：续做里一步读不出，交 `PublishError::BlockDevice`，「| PublishError::BlockDevice(_) => false,」（`crates/singlefs-core/src/mount.rs:5334`），不在取号之前拒。
- 两种都落到文档说的「或取号之后那一串在续做之前就报错，交回 `None`：那些照旧留给真发，真发时报它自己的。」（`crates/singlefs-core/src/mount.rs:6945`）。
  真发那一趟读得出（窗口过了），一路取号、写行、暖机、推抬 F、续做，已落盘 19 次发布之后在最后一批交 C841（19 次里各占几次没拆，写账在错里）。

### J3：同一个窗口在没改坏的镜像上被预演吃掉

上面 `unchanged start=340`、`unchanged start=960` 两行：两次读错都真注入了（`failed_reads=2`），挂载做成，续做 16 步走到空，
盘与无故障那一次逐字节相同（`disk_same_as_fault_free=true`）。真做那一趟一次读错都没见到：故障整个落在取号之前那一趟里。
这正是正文 J3 问的「预演多出的读落在底下那块盘上，对带读故障注入的设备会不会改变真做那一趟看到的读结局」（`research/prompts/_m3-b44-code-r2-body.md:32`）：会。
预演那一趟在这份镜像上约多读七百多次（改坏的那一份上取号之前一共 1070 次读，两簇打中点分在 335 与 1016 两头；按位置推的，没逐次标），
harness 按整池读序号摆的读故障（`FaultCounting::AcrossThePool`）在要走只进内存那一趟的挂载上会整体错位。随机故障注入里有多少段历史走得到这一格，我没量。

### J3：只进内存的盘的内存

`DeviceKeepingWritesInMemory` 的写只进不出：字段「written_blocks: BTreeMap<u64, Vec<u8>>,」（`crates/singlefs-core/src/mounted_session.rs:365`），
写一块就「.insert(first_block + block_offset, block.to_vec());」（`crates/singlefs-core/src/mounted_session.rs:423`），文件里没有删的那一句。
它装的是取号到续做走完这一串写过的每一个不同的物理块，同一块重写只留一份。

量的（`out/run-bytes.log` 原样，没改坏的窄盘、无故障、一批一个单元）：

```
continuation steps=16 bytes_per_step=[442880, 442880, 442880, 442880, 442880, 377344, 377344, 377344, 377344, 377344, 377344, 442880, 442880, 442880, 410112, 377344] sum=6594560 floor_raise_publishes=[213504, 213504] device_written_bytes={DeviceIdentity(0): 3766784, DeviceIdentity(1): 3763712} total_reads=3104
```

续做每步 377,344–442,880 字节，16 步 6,594,560 字节；整次挂载两盘共写 7,530,496 字节。真做与预演走同一串，所以这一份镜像上那块内存盘至多约 7.5 MB（上界是推的：
不同块数没量，只量了写的总字节）。

推的（没量）：步数随要删的文件线性涨。产品批大小是一片 extent 下段叶，一片叶约 (16384 − 163) ÷ 112 ≈ 144 条记录，一条指一个 32 KiB 数据单元，
一步约放掉 4.5 MiB；删 1 TiB 的文件约 23 万步。每步按上面实测的约 0.41 MB 算（产品批一步放 144 个单元，分配记录树改得更多，这个数偏小），
内存盘约 95 GB。上界是单元区大小（同一槽重写只留一份），可单元区按 TB 计时这个上界没意义。走到这一格的前提是分配器拷贝上那一趟空间不够——
正是「盘满了、删一个大文件腾地方、删到一半崩了」那一格。没有上界的那一句在条款里也没有。

### 别的形：没打中

| 形 | 取样 | 结局 |
|---|---|---|
| 单次读错（任一次读） | 改坏的那一份上 1070 个读序号全扫 | 全部取号之前拒、盘不变（两份镜像兜住） |
| 预演写盘 | 上面 3032 次取号之前拒的挂载（单次 1070 + 窗口 1962） | 盘都不变（量的）；真分配器动没动在拒之后看不见，没量 |
| 写行那次发布、写行之后推的抬 F 改不改续做看到的容器树 | 读码 | 不改（推的）：抬 F 是「推空发布直到每块盘上都有带新 F 的根（生效），然后把释放代 ≤ 新 F 的已释放落点回收」（`crates/singlefs-core/src/mount.rs:3887`），空发布不碰三棵容器树；退役树文档也说「第一版不回收 ⇒ 容器只在最右端出现、一片都不删」（`crates/singlefs-core/src/retirement_tree.rs:25`） |
| 两边容器容量取的是不是同一份 | 读码 | 是（推的）：预演「deletion_switches_the_continuation_reads.container_tree_capacities(),」（`crates/singlefs-core/src/mount.rs:5272`），真做「pool.set_container_tree_capacities(switches.container_tree_capacities());」（`crates/singlefs-core/src/mount.rs:6544`），准备那一段读「deletion_and_lineage.container_tree_capacities.node_entries,」（`crates/singlefs-core/src/transaction.rs:5302`） |
| 读盘核对不上被隔离、两边分配器不同 | 读码 | 落点会不同，容器树的形状只看记录，C841 那一判不变（推的） |
| 队列长度、每批条数、哪棵树开新容器 | 现有三条用例（1 步、3 步、16 步带抬 F，都是退役树）在合入验证里绿；我没另造墓碑树、谱系树开新容器的续做 | 没打中，也没扫 |

C841 那一判只看树的形状，形状只跟着删除的记录走，这一点我在上面几格都没找到分叉；打中的只有「预演自己没走完」那一格。

### 四句（J2、J3 打中的那几格）

1. 分不分辨臂：分。打中的是「预演被读错打断就放行」这一臂；「被打断就重来一次」「被打断就拒」两臂在同一段历史上不中（推的，见「四句」后面那张表）。不是各臂共用的前提。
2. 系统当时看不看得到：看得到「预演没走完、断在一次读错上」（`BlockDevice` 错，或内存里那一趟取号交了错）；看不到「断点后面有没有 C841」。所以改法只能按「没走完」处置，不能按「有没有 C841」处置。
3. 满足的是哪一个分句：派发验收 J2 的「预演放过而真做在取号之后交 C841（盘变了）」与 J3 的「预演多出的读让真做那一趟看到不同的读结局（故障被预演吃掉）」。
   条款那一边，正文 J2 那一行写的是「只拒 C841（`PublishError::NextContainerNumberPastTheTopOfItsRange`），别的错照旧留给续做」（`research/prompts/_m3-b44-code-r2-body.md:31`）：代码照字面做了，
   读错是「别的错」、留给了续做；条款没写「留给续做的那个错续做里不再出现」这一形。按正文「一格的反例落在条款没写的地方 ⇒ 交用户定条款」（`research/prompts/_m3-b44-code-r2-body.md:49`）办。
4. 改法在打中的格上还中不中（都没实现，全是推的）：

| 改法 | 起点 335–345（内存里取号读错） | 起点 951–1016（续做一步读错） | J3 吃掉故障 | 代价 |
|---|---|---|---|---|
| 甲：预演被读错打断（`BlockDevice`、内存里那一趟取号或抬 F 交错）就在取号之前拒，写账空 | 修 | 修 | 不修（故障照样落在预演里，只是这回挂载拒了） | 拒的不只 C841，超出岔路单那一行，要用户定；瞬时读错也拒 |
| 乙：被读错打断就从头再预演一次（照 C554 乙那种只重读一次），仍断再按甲或放行 | 宽 2 修（窗口已过） | 宽 2 修 | 不修 | 多一趟预演的读与内存；仍断时落回甲或现状 |
| 丙：现状 | 不修 | 不修 | 不修 | — |

几个改法都碰不到「故障被预演吃掉」：只要预演读底下那块盘，读序号就会挪。要不挪，只能让预演不读盘，那就回到 W11 之前「抬 F 不预演」。

## 三、J4：取号之前那一拒复用 `MountError::Publish`、写账空

没打中。查了按 `MountError::Publish` 分路的每一处（`grep -rn 'MountError::Publish' crates --include=*.rs`，`tests/` 之外），看它们在写账全空时怎么走：

| 调用方 | 认什么 | 写账空时 |
|---|---|---|
| `crates/singlefs-harness/src/model_comparison.rs:815` | 按 `cause` 映射理由 | C841 映射成「对不上」：「PublishError::NextContainerNumberPastTheTopOfItsRange(_)」（`crates/singlefs-harness/src/model_comparison.rs:725`）；随机历史在健康内存盘上造不出 C841，与取号前后无关 |
| `crates/singlefs-harness/src/model_comparison.rs:930`、`crates/singlefs-harness/src/model_comparison.rs:1045` | 点名的根槽、上限 | 交 `None`，不看写账 |
| `crates/singlefs-harness/src/mount_transparency.rs:2320` | D23 那张单子上的理由 | 不在单子上，交 `None` |
| `crates/singlefs-harness/src/history.rs:3006` | 只格式化 | 不看写账 |
| `crates/singlefs-core/src/mount.rs:4209` | 抬 F 入口重新分类 | 不看写账 |
| `crates/singlefs-checker-tier/src/crash_injection.rs:3204` | 按写账划续做从第几次起 | 两份都空时落到最后一臂交 `None`（「None if !writes_of_persisted_publishes.is_empty()」那一臂之后，`crates/singlefs-checker-tier/src/crash_injection.rs:3264`），文档本来就写着「续做的第一步落盘之前就被拒、一个写都没发」交 `None`；挂载那一段录制流是空的，对得上 |
| `crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs:798` | 认定取号屏障已到：「counts_when_the_acquisition_barrier_arrived(&devices, &plan)?;」（`crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs:800`） | 取号之前的拒在这里不成立；但这个装置的池是新池新建文件、删除队列空，续做与它的预演都不走，走不到这一拒（推的，没跑） |

harness 故障注入判「取号之后挂载报错、新号没回卷」按的是盘上系统配置的实例代号（`crates/singlefs-harness/src/fault_injection.rs:1824` 那个函数起），不按错的成员，空写账不影响。
没找到把「`Publish` 且写账空」当成「写了一半」的调用方。

什么会推翻这一格：哪个调用方按 `MountError::Publish` 断定「已取号」（例如拿它去推下一次挂载的实例代号）而不看盘。我列的是此刻的全部 grep 命中，之后新加的调用方不在内。

## 四、J5：顺带一条注释

`walk_the_continuation_after_acquisition_on_devices_keeping_writes_in_memory` 的文档有两句被 J2 与 J3 那一节的读错窗口打中了：

- 「再试，推满仍放不下就停）照真发走一遍：同一份计划、同一个推抬 F 的判法与同一段抬 F，续做报的错就是真发时会报的那一个。」（`crates/singlefs-core/src/mount.rs:6941`）
- 「或取号之后那一串在续做之前就报错，交回 `None`：那些照旧留给真发，真发时报它自己的。」（`crates/singlefs-core/src/mount.rs:6945`）

读错窗口落在预演里时，真发报的不是预演报的那个读错，是更后面的 C841；写成「同一份输入」时，底下那块盘这一刻的读结局不在「同一份」里。
改法跟着 J2 与 J3 那一节的表走：选了甲或乙，这两句照选的那一臂改；选丙，这两句要加「读错可能只落在预演那一趟」的限定。别的 J5 注释我没逐条看。

## 五、没打中的形状（汇总）

- J1：实例表链指针、中央映射条目、journal 点名项三类不在清单里，但都不会走到 `NodePointer::write_to` 的断言（J1 那一节的表）；误拒那一面没造出反例，只读了豁免与空树两处。
- J2：单次读错 1070 个读序号全扫，0 次打中；树的形状在写行、抬 F、隔离三处都不分叉（读码，推的）；墓碑树、谱系树开新容器的续做没造。
- J3：只进内存的盘与真盘的语义：写零、读没写过的块、越界与对齐的拒（同一份 `check_aligned_and_in_range`）我读了码、没找到分得开的地方；
  journal 提示它答 `None`、真盘答它写过的槽，只影响扫环快慢、不影响结果（文档自己的话，没量）。打中的只有「读故障」那一格。
- J4：上表 8 处。

## 六、这条腿自己的限度

- 只在副本上量。镜像只有一种几何（不补环窗口的窄盘，单元区 304 槽，16 个数据单元，一批一个单元，退役树），窗口宽只取 2 与 6。
- 原型是 debug 档，每次挂载 0.5 到 3.5 秒；整份窗口扫描 2144 次挂载，八片并跑，每片 102 到 130 秒（`out/b2-*.log` 的 `scanned` 行）。单次读错扫描四片，每片 124 到 128 秒。
  估时：先跑 3 个读序号 2.1 秒（改坏的）、10.2 秒（没改坏的），按此定片数；没改坏的那一份全扫要约 3 小时，没扫，只挑了两个打中过的起点（J2 与 J3 那一节）。
- 崩溃状态：这一轮的反例都不经崩溃，我没调 `enumerate_crash_states`，原型跑的崩溃状态数是 0。
- 打中点落在哪一步（内存里取号、还是续做哪一步）是按读序号的两簇推的，没逐次打调用栈。
- 内存那一格只量了写的总字节，没量只进内存的盘的峰值；大文件的数全是推的。
- 完整扫描的命令（在副本里、编完之后）：`PROBE_FROM=<起> PROBE_TO=<止> PROBE_WIDTHS=2,6 bash scripts/hosts/run-with-memory-cap.sh 6G bash scripts/hosts/capped.sh 31 <测试二进制> --nocapture --exact scan_read_fault_windows_on_the_image_with_the_retirement_container_at_the_top_of_its_range`，起止按 134 一片切 1–1072；
  单次读错同法，测试名换 `scan_read_faults_on_the_image_with_the_retirement_container_at_the_top_of_its_range`。

## 七、没做什么

- 不判正推那一边的格，不替主 agent 采纳；副本上的数不算入库装置上的数。
- 改法一个都没实现、没在副本上跑过，表里每一格都是推的；被攻过零轮。
- 没跑任何门禁阶段、checker 档测试、GPU；没读禁读清单里的文件。
- 只读了 `crash_injection.rs`、`new_pool_file_creation_on_device.rs` 两份 checker 档源码，没编、没跑它们。
- `tools/rerun.sh` 只过了 `bash -n`，整条没跑过（编一次约 5 分半，加四样约 3 分钟）；模板自带的 `--selftest` 在换过 `rerun_steps` 之后会去假仓里起 cargo，没跑。
  打出的汇总行与 `out/` 里哪几份比，写在脚本末尾那一句。
- 草稿目录 `~/.local/share/singlefs/work/m3-b44-code-r2-opus/` 里的仓副本与编译目录交回前已删，只留日志与 `rerun.sh.new`（与 `tools/rerun.sh` 同内容）。
