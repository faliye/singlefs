# m2-final-code-r1 云端攻方腿（Opus）报告：Z1、Z4、Z6

2026-09-24 UTC（2026-09-25 JST）。背景材料 `research/prompts/_m2-final-code-r1-background.md`（sha256 5bbf908ff1e0675b86029c1187cecd53454943c663370b62f9a61f7d7c3cd089），代码一律读冻结副本 `/tmp/claude-1000/m2-final-code-r1/tree/crates/`（`mount.rs`、`transaction.rs`、`recovery.rs`、`journal.rs`、`allocator.rs` 五份的 sha256 与 `research/prompts/m2-final-code-r1-snapshot/crates-src-sha256.txt` 逐行相同，现查过）。下文代码行号都是冻结副本的行号，kb 行号是 kb 文件自己的。

## 各格判定一览

| 格 | 判定 | 一句话 | 节 |
|---|---|---|---|
| Z1 | 兑现了条款（没打中） | 读法乙、按标志认边界、三种断链、解析器两格都照字面；64 片写行那次末条再跨两条，164 个前缀崩溃 + 16 个记录写子集全绿 | 三 |
| Z1 附记 | 替没写的条款做了选择（不改字节 / 只在坏盘上） | 事务号 0 的发布跨多条时第一条提交标记写 0；读者把提交标记 2..=255 当「没有」、checker 判违例；`mount.rs:1708` 注释还是 jsn 最大的字面 | 三 |
| **Z4-1** | **替没写的条款做了选择（新，打中）** | 树表 0 条那一臂写行那次的分配记录条数准入在取号之后才判：40 片起每试一次可写挂载烧一个实例代号，池永远挂不上可写；带文件那一臂同一格在取号之前拒 | 二 |
| Z4-2 | 已知（C381、实二二） | 抬 F 那一串交回的已落盘次数，在根已落盘 / 记录已落盘两格比恢复实际落到的少一次 | 二 |
| Z4-3 | 已知（实二二） | 抬 F 失败之后同一进程接着覆盖写：按新 F 做的回收没退，I-3.1 判红 | 二 |
| Z4-4 | 已知（实二三） | 拷贝上释放核验报错交 `None`、照样取号 | 二 |
| Z4-5 | 已知（C542） | 拷贝上不读盘核，被隔离的那一格不比 | 二 |
| Z6 | 兑现（断言没打中）；合并点上打中的是 Z4-1 | 回退到根环里每一条根 × 一片 / 两片 × 带文件 / 树表 0 条 × 挂载 1..=8 次共 1225 组合全绿；新种子随机历史 三组 × 1500 种子新发现 0 | 四 |

## 复跑

全部在草稿副本上跑（冻结副本 rsync 到 `/tmp/claude-1000/m2-final-code-r1-opus/tree/`，只加 `crates/singlefs-harness/tests/opus_r1_attack.rs`，即模型目录里那份）：

```bash
cd /tmp/claude-1000/m2-final-code-r1-opus/tree
R=/home/fy5090/code/singlefs/research/scripts/capped.sh
bash $R 5 nice -n 19 cargo test --release -p singlefs-harness --test opus_r1_attack z6_a1 -- --nocapture
bash $R 5 nice -n 19 cargo test --release -p singlefs-harness --test opus_r1_attack z1_a1b -- --nocapture
bash $R 5 nice -n 19 cargo test --release -p singlefs-harness --test opus_r1_attack z4_a3 -- --nocapture
OPUS_PAGES=66 OPUS_MOUNTS=30 bash $R 5 nice -n 19 cargo test --release -p singlefs-harness --test opus_r1_attack z4_a5 -- --ignored --nocapture --exact z4_a5_without_file_row_publish_allocation_record_admission_after_acquisition
OPUS_PAGES_FROM=2 OPUS_MOUNTS=14 bash $R 5 nice -n 19 cargo test --release -p singlefs-harness --test opus_r1_attack z4_a5b -- --ignored --nocapture
OPUS_MOUNTS=1200 bash $R 5 nice -n 19 cargo test --release -p singlefs-harness --test opus_r1_attack z4_a5c -- --ignored --nocapture
OPUS_PAGES=66 OPUS_MOUNTS=10 bash $R 5 nice -n 19 cargo test --release -p singlefs-harness --test opus_r1_attack z4_a5d -- --ignored --nocapture
bash $R 5 nice -n 19 cargo test --release -p singlefs-harness --test opus_r1_attack z6_a4 -- --ignored --nocapture
OPUS_SEEDS=1500 OPUS_OPS=60 bash $R 5 nice -n 19 cargo test --release -p singlefs-harness --test opus_r1_attack z6_a2 -- --ignored --nocapture
# 改法 Fx：把 fix-fx.diff 打在另一份副本上（/tmp/claude-1000/m2-final-code-r1-opus/tree-fix/），再跑上面 z4_a5 那一行
```

模型目录 `research/prompts/m2-final-code-r1-opus-model/` 的文件（`logs/` 下是上面各条命令的原样输出，文件名对应用例）：

```text
d638896dc65feb6e914c106c315140eb7eecdff33fb82ad895649c54f0ddfb32  ./fix-fx.diff
74de6acc3740ddc096dca714c3b915dbf8b17587ac8ad1eb1f21492552604ed1  ./logs/a1b.log
eeb4c3b5d429406383eedecd1cbb5d9a91f810e0f5289445fe73b4c4711d84e8  ./logs/a1.log
44bba975e970c94c86267f642015da9cef448b9b4ebe6f9ba45b18c642d6fc66  ./logs/a2-large.log
c2ecfd75dd17e775565ec9d0440afea20e85546969d74544af12b53e0b47704f  ./logs/a3.log
a16817f71d9511f8b962e6d1b0838c83b8dc686c1c199289ab0b30f2a5e9e95d  ./logs/a4.log
477f22c7ea4288ae7d11a33afccb72d5b0c80cd647fea43903f5b69ab233ab14  ./logs/a5-66.log
d2c329d15b829eb85991c0ece69776d79ba9fbedb381a576b2e6c9c55713981c  ./logs/a5b.log
2afb93351911f55d453ff5d26eb78dce02f4d0e9789171a91b4c4c24c6ea7071  ./logs/a5c.log
70ce387ec04a931c3f90243a957dc6e3858fd286ef498d766b55b3924ec00b93  ./logs/a5d.log
2e306fd4beefdb78076a47c3cb622e53b81cb5c1caca459fff6c9d757ddf15aa  ./logs/a5-fix-fx.log
03ed3a46004bc5ca085903c9c29fe3fa86b35ba50ccc859336a7cac1ae814cf7  ./opus_r1_attack.rs
```

## 二、Z4　取号之前在分配器副本上预演、抬 F 失败的账

### Z4-1（新，打中）　树表 0 条那一臂：写行那次发布的分配记录准入在取号之后才判，每试一次烧一个实例代号

**结论**：替没写的条款做了选择（D18（块里携带什么信息） 已定项 11「可写挂载的顺序」列的合取里没有准入这一项；带文件的一臂按 2026-09-18 用户定案在取号之前判、树表 0 条的一臂在取号之后判，两臂选得不一样，树表 0 条那一臂今天没有会红的东西钉着）。它同时与 `checks-owed.md` 第 329 行 C378 现状栏里那一句「被准入拒不再烧号」对不上（整行抄在附录 A2）。

**被判对象里许可它的那几句**（冻结副本 `/tmp/claude-1000/m2-final-code-r1/tree/crates/singlefs-core/src/`）：

- `mount.rs:1151`：`/// 分配记录树、记账树与中央映射树那三条只在带文件的一版上判：树表 0 条 ⇒ 这一版没有这三棵树，写行与暖机都不写它们`
- `mount.rs:1174`：`    if let PreviousVersion::WithFile { output, .. } = &start.previous {`（取号之前的分配记录准入只在这一臂里）
- `transaction.rs:689`：`/// 写行那次发布，上一版树表 0 条：**重写实例表链与分配记录树**（链有几片就几个实例表单元，加一个分配记录树节点），`
- `transaction.rs:732`：`    version_without_file_row_publish_admission(`（同一道条数准入，在 `publish_instance_table_on_version_without_file` 里，取号之后才走到）

`mount.rs:1151` 那句的前提自 C512（树表 0 条的一版上被换下的实例表记在哪没有条款）（`checks-owed.md` 第 504 行，2026-09-23 定）起不成立：树表 0 条的一版上写行那次发布写一片分配记录树节点，`transaction.rs:732` 也按条数拒。

**历史**（用户动作放开扫过：片数 2..=66 每档连挂 14 次；另有纯挂载 1200 次的对照）：

1. mkfs → 可写挂载一次（实例 1，树表 0 条，零单元发布）。
2. 「取号之后崩溃」k 次（`acquire_instance` 连取 k 次号，每次是一段合法历史：取号写完、写行那次发布之前掉电；`second_transaction_supplement_two_instance_table_page_full.rs` 用的同一招）。k = (片数 − 1) × 369，下一次挂载一次写 k + 1 行。
3. 之后一路可写挂载。树表 0 条的一版每次写行重写整条链（66 片 + 1 片分配记录节点），被换下的旧链进 defer；根环 24 槽还没转过，一条都回收不了，分配记录条数每次涨 134。

**原样输出**（`logs/a5-66.log`，片数 66）：

```text
OBS a5 pages=66: mount#5 ok instance=23992 rows_written=1 allocation_records=808 released=672
OBS a5 pages=66: mount#6 err (next instance before 23993 after 23994) "Publish(PublishSequenceFailed { cause: AllocationRecordsExceedOneNode { records: 942, capacity: 812 }, writes_of_persisted_publishes: [], writes_of_failed_publishes: [] })"
OBS a5 pages=66: mount#7 err (next instance before 23994 after 23995) "Publish(PublishSequenceFailed { cause: AllocationRecordsExceedOneNode { records: 942, capacity: 812 }, writes_of_persisted_publishes: [], writes_of_failed_publishes: [] })"
```

第 6 次挂载起，每一次都先取号（下一个号 23993 → 23994）、再在写行那次发布里被 `AllocationRecordsExceedOneNode` 拒；之后连试 23 次，号每次涨 1、错一字不差（`logs/a5-66.log` 第 21–44 行）。分配器被拒之后原样退回、盘上只多了取号那几次系统配置写，所以下一次还是同一个 942：**池从此每试一次可写挂载烧一个号，永远挂不上可写**。

**阳性对照**（`logs/a5d.log`，同样 66 片、同样节奏，只把起点换成带文件的一版）：

```text
OBS a5d pages=66 mount#4 next instance before 23991 after 23992: ok instance=23991 allocation_records=784
OBS a5d pages=66 mount#5 next instance before 23992 after 23992: err RowPublishAdmissionRefusedBeforeAcquisition { instance_to_acquire: InstanceGeneration(23992), cause: AllocationRecordsExceedOneNode { records: 924, capacity: 812 } }
```

带文件的一臂在取号之前拒、号不涨；两臂只差 `mount.rs:1174` 那个分支。

**射程**（`logs/a5b.log`，片数 2..=66 × 每档 14 次挂载）：39 片及以下 14 次之内没拒（39 片那档第 13 次挂载时分配记录 724 条）；40 片起每一档都在取号之后被拒（40 片：第 9 次挂载，`records: 824`；44 片：第 8 次；50 片：第 7 次；57–66 片：第 6 次）。不靠「取号之后崩溃」、纯挂载的对照（`logs/a5c.log`）：1200 次挂载、表长到 4 片，分配记录停在 94 条、一次没拒——纯挂载要涨到 40 片得挂一万四千多次，我没跑到那里。

**四句**：

| 问 | 答 |
|---|---|
| 分不分辨臂 | 这一轮是代码轮、没有候选臂；它分辨的是「两臂准入判在取号前还是后」：带文件的一臂同一格不中（阳性对照），树表 0 条的一臂中 |
| 被判的系统当时看不看得到判别它的东西 | 看得到：取号之前 `allocator.records().len()` 与这次之后的片数都已在内存里（`transaction.rs:732` 在取号之后读的就是同一个分配器，取号不碰它）；副本上的改法 Fx 把同一个判断搬到取号之前、号就不涨（下面的表） |
| 满足的是判据字面的哪一个分句 | 正文第一节「替没写的条款做了选择」：D18（块里携带什么信息） 已定项 11「可写挂载的顺序」（`decisions/18-块里携带什么信息.md` 第 314 行，附录 A1）合取里只有「副本上预演取得到全部落点」，没有条数准入；带文件那一臂在取号之前判是 2026-09-18 的用户定案（`milestone/02-second-txn.md` 第 371 行（收口表第 57 行），附录 A3），树表 0 条那一臂在 C512 之后长出了同一道准入而没跟上 |
| 跑前条款给的改法在打中的那几格上还中不中 | 这一轮正文没有跑前改法；下表是我自己的 |

**改法**（只在我的模型上量过、被攻过零轮）：

| 改法 | 修哪一格 | 标注 | 打中那几格上的结果 |
|---|---|---|---|
| Fx：`refuse_publishes_before_acquisition_that_do_not_pass_admission` 里对 `PreviousVersion::WithoutFile` 且要写的行不为空时，先调 `version_without_file_row_publish_admission`，错映射成 `RowPublishAdmissionRefusedBeforeAcquisition`（补丁 `fix-fx.diff`，33 行） | 取号之后才拒 ⇒ 烧号 | 量过（副本 `/tmp/claude-1000/m2-final-code-r1-opus/tree-fix/`） | `mount#6 err (next instance before 23993 after 23993) "RowPublishAdmissionRefusedBeforeAcquisition { instance_to_acquire: InstanceGeneration(23993), cause: AllocationRecordsExceedOneNode { records: 942, capacity: 812 } }"`（`logs/a5-fix-fx.log` 第 21 行）：号不再涨 |
| Fx 不修的那一格 | 池进吸收态（记录不减、以后每次都拒） | 推的 | 与带文件那一臂今天的样子相同（阳性对照第 5 次起一直拒），属于里程碑「第二个事务」收口表第 39、57 行那一族（分配记录树只有一个节点），不是这一格新添的 |

**什么现象会推翻它**：在冻结副本上跑同一条历史，第 6 次起的挂载在取号之前被拒（系统配置里的号不涨）；或者证明「取号之后崩溃」连取两万多次不是合法历史。

### Z4-2（已知，去向 C381（根已落盘之后发布失败，分配器仍退回） 与实二二「发布失败原样重发」）　抬 F 那一串失败时交回的「已落盘次数」与盘上对不上

**历史**：带文件的池（第一个事务 → 可写挂载，实例 2）→ 同一进程里覆盖写 6 次（txg 11，F 0，上限 8）→ `raise_rollback_floor` 抬到 8，这一串的第 n 次写（整池数，从 0 数）注入块设备错（用例 `z4_a3_raise_floor_failure_then_continue_in_the_same_process`，包装设备 `FailTheNthWrite`）。这一串每次空发布 13 次写：四个固定点 × 两盘 8、记录 2、根 1、系统配置 2。

**原样输出**（`logs/a3.log` 第 28、30、34、36 行）：

```text
OBS a3 fail-first-sysconfig: raise failed persisted=0 failed_accounts=1 cause=BlockDevice(InputOutput(Custom { kind: Other, error: "注入的写错" }))
OBS a3 fail-first-sysconfig: cold recovery right after the raise: effective=Some((InstanceGeneration(2), CheckpointTxg(12))) applied=0
OBS a3 fail-second-root: raise failed persisted=1 failed_accounts=1 cause=BlockDevice(InputOutput(Custom { kind: Other, error: "注入的写错" }))
OBS a3 fail-second-root: cold recovery right after the raise: effective=Some((InstanceGeneration(2), CheckpointTxg(13))) applied=1
```

`persisted` 是 `writes_of_persisted_publishes.len()`。系统配置槽那一步报错时根已落盘（第一格：交回 0 次，冷启动落在 txg 12 那条根上）；根槽写报错时记录已落盘（第二格：交回 1 次，冷启动由记录施加出 txg 13）。两格里「交回的已落盘次数」都比恢复实际落到的少一次。

**归属**：第一格就是 `checks-owed.md` 第 332 行 C381 的题面（根已落盘而发布路径把分配器退回）；第二格同一族——记录已落盘、根没落，恢复照样施加它。条款侧 D23（journal 的角色与格式） 已定项 14「这一版的失败处置」（`decisions/23-journal的角色与格式.md` 第 383 行）要求失败之后下一次先逐字节原样重发，重发成功之后「交回几次」与盘上就对得上；实现随实二二（背景材料「已知」第 5 条）。C516（抬 F 那一串发布被拒时前面几次已落盘） 的已还清行（第 502 行）钉的是落点被拒、一个写都没发的那一格（`second_transaction_supplement_two_commit_generated_fallback.rs` 那条用例），块设备错这两格今天没有会红的东西钉着。

**顺带（文字，不算打中）**：C516 已还清行写的是 `MountError::RaiseFloorSequencePublishFailed { publishes_persisted, cause }`；冻结副本里这个成员是 `RaiseFloorSequencePublishFailed(PublishSequenceFailed { cause, writes_of_persisted_publishes, writes_of_failed_publishes })`（`mount.rs:143` 起的结构体），`publishes_persisted` 这个名字在 `crates/` 里只出现在 `history.rs` 的一句报告格式串里。

### Z4-3（已知，去向实二二「发布失败原样重发」）　抬 F 失败之后同一进程接着发别的发布：I-3.1 判红

**原样输出**（`logs/a3.log` 第 22、23、25 行）：

```text
OBS a3 fail-first-record: raise failed persisted=0 failed_accounts=1 cause=BlockDevice(InputOutput(Custom { kind: Other, error: "注入的写错" }))
OBS a3 fail-first-record: after raise: current txg=11 F=0 free_slots_dev0=211926 deferred_dev0=30 checker=[]
OBS a3 fail-first-record: same-process overwrite ok txg=12 F=0 checker=[("I-3.1", "\"盘 0：记账的已分配 Some(851968)，遍历全部有效根得到 1523712；机理：根环槽数 24、最新根 txg 12、环里自证过的根槽 13 个、最老的自证过的根 txg 0、遍历的候选根槽 13 个、并进遍历的由记录施加出来的版本 0 个、被实例表判抛弃的根槽 0 个、回退下界 F 0、低于 F 的根槽 0 个\"")]
```

机理：`raise_rollback_floor` 在推第一次空发布之前就按新 F 回收并扣住（`mount.rs:962` `    let reclaimed = allocator.reclaim_released_up_to(`），发布失败时 `publish_version` 只把分配器换回这一次发布之前（`transaction.rs:2957` `        *allocator = allocator_before_this_publish;`），回收那一步不退；于是抬 F 失败（盘上没有一条带新 F 的根）之后，这个进程的分配器空闲 211885 → 211926、defer 71 → 30，下一次覆盖写带着旧 F = 0 的根把少了 41 槽的「已分配」写进记账行。对照组（不注入）同一步 checker 全绿（第 19 行），重挂之后同一份盘面也全绿（第 26 行）。

**归属**：D23（journal 的角色与格式） 已定项 14「这一版的失败处置」要求失败之后下一次发布先原样重发失败那一次；重发的那次带新 F，与已经做过的回收说同一件事。今天实现允许同一进程接着发别的发布，正是实二二要堵的那条路。`raise_rollback_floor` 今天只有测试入口（`mount.rs:885` ``/// ⚠️ **今天只有测试入口，没有产品路径**：`crates/*/src/` 里一个调用点都没有（2026-09-22 现查），``；C482（只供测试的开关有三个走了哪一条看不出来） 第 ② 条）。

### Z4-4（已知，去向实二三）　副本上预演时写行那次的释放核验报错，不在取号之前拒

`mount.rs:1534` `        PlacementsOnTheCopy::ReleaseCheckFailedBeforeTheFirstPlacement => None,`：拷贝上经映射核换下的落点报错时交 `None`，照样取号，取号之后由发布路径报同一个错。D18（块里携带什么信息） 已定项 11 那一句「预演里写行那次的释放核验报错，同样判这次不能可写」（附录 A1）与它说反话；背景材料「已知」第 6 条（用户 2026-09-24 定，实现随实二三）。没造镜像跑：它要一张映射里查不到 key 的坏镜像。

### Z4-5（已知，C542（取号前预演不读盘核，坏盘加回退到最老根会烧号））

拷贝上不做释放前读盘核；这一串里有一份核出对不上时 `establish_instance` 不比（`some_copy_was_quarantined`）。没另造镜像：C542 的题面就是这一格。

## 三、Z1　末条标志、末条再跨记录、锚点读法乙

### Z1-1　结论：兑现了条款（没打中）

条款整段原文见附录 A4–A7（D23（journal 的角色与格式） 已定项 4 的序号那一条、已定项 14 第六条与注 1、已定项 17 的末条再跨记录那一条）。原文到代码的那一步：

| 条款的那一句 | 代码（冻结副本，整行） |
|---|---|
| 注 1「「那条」按末条标志认……不取读得出的同 txg 记录里 jsn 最大的那条」 | `recovery.rs:1744` `    let root_own_record_counter = counter_of_the_last_record_the_root_covers(root, records)?;`；带标志多于一条在 `recovery.rs:1682` `    if counters_carrying_the_last_record_flag.len() > 1 {` 拒，至多一条取 `recovery.rs:1692` `    Ok(counters_carrying_the_last_record_flag.first().copied())` |
| 注 1「那条读不出时，链首接水位之上第一条可读记录，前提是它的「本次发布内序号」为 1、`checkpoint_txg` = 所选根的 txg + 1」 | `recovery.rs:1755` `        } else if record.checkpoint_txg != chain_start_txg_without_anchor` 与 `recovery.rs:1756` `            \|\| record.ordinal_within_publish != JournalRecordOrdinalWithinPublish::FIRST` |
| 第六条「一次发布的末条 = 记录标志位 0 为 1 的那一条」；已定项 7「提交标记……」 | `recovery.rs:1793` `        if !record.is_commit && record_ends_its_publish(record) {` |
| 已定项 17「只有真正的最后一条带「本次发布末条」标志」 | `transaction.rs:4144` `        let place_in_publish = if transaction_of_the_next_record.is_none() {`；提交标记 `transaction.rs:4154` `            is_commit: is_the_last_record_of_its_transaction,` |
| 已定项 4「序号 0……当那条记录损坏」 | `journal.rs:279` `        if ordinal_within_publish.0 == 0 {` |

**跑过的历史**（草稿副本上，用例在 `opus_r1_attack.rs`）：

- `z6_a1_…`：带文件的一版上写行，实例表 64 片（取号之后崩溃 23248 次之后挂载，写 23249 行）⇒ 点名 64 + 4 = 68 项，末条再跨成两条。原样输出（`logs/a1.log` 第 6、7、9 行）：

```text
OBS a1 instance=23250 rows=23249 records=2 named=[67, 1] ordinals=[1, 2] last_flags=[false, true] commit=[false, true] transactions=[0, 0] counters=[4, 5]
OBS a1 checker after first mount: []
OBS a1 checker after second mount: []
```

  写行那次发布事务号 0、跨两条，提交标记与末条标志只在第二条。这一格正文没点名：事务号 0 的记录跨多条时第一条提交标记写 0（`publish_without_units` 那条空记录写 1）。恢复只在「换了事务号」时看提交标记（`recovery.rs:1784` 那一判），同一个 0 不断；checker 的 I-8.8 把事务号 0 整个排除（`walk.rs` 的 `judge_commit_markers_per_transaction` 按 `TRANSACTION_NUMBER_OF_AN_EMPTY_PUBLISH` 滤掉）。两边一致，今天不出错；D23（journal 的角色与格式） 已定项 7 与已定项 19 ① 没写「不承载事务的发布跨多条时提交标记怎么写」，记一笔，不算打中。
- `z1_a1b_…`：同一个 64 片的池再挂一次（写行那次发布同样跨两条），这次挂载录制流 163 步，每个前缀崩一次（164 个状态），外加写行那次发布四个记录写（两条 × 两盘）的全部 16 个子集（单元写全落、记录写任意落）。每个状态上冷启动恢复一次、可写挂载一次、挂载成了跑池级 checker。原样输出（`logs/a1b.log` 第 7–13 行，五类前缀结局与一类子集结局）全是 `mount ok … violated=[]`，没有 panic；子集那 16 格恢复都落在上一个实例的根上（`applied=0`）——写行那次发布是新实例的第一次发布，前缀规则不跨实例边界（注 1 第一句），记录落不落都不施加，与条款相同。

### Z1-2　没打中、记一笔的两处（都不改字节）

1. `mount.rs:1708` `    // 所选根覆盖的最后一条记录读得出就拿它当上一版的记录：同实例、同 checkpoint_txg 的记录里 jsn 最大的那条`，下面是 `mount.rs:1718` `        .max_by_key(|record| record.counter)`（回退那一路 `mount.rs:1850` 同一句）。注释引的是注 1，写法却是读法乙之前的字面（jsn 最大）；这条记录只拿来当重建出的上一版的 `record` 与 `highest_transaction_number_in_this_instance`，写行那次发布的计数器取环里最大 jsn + 1、反向链 0、事务号 0，都不读它——我没找到一条历史让两种读法写出不同的字节。去向：注释与注 1 字面对齐即可，不算打中。
2. `journal.rs:276` `        let is_commit = reader.get_u8() == 1;`：提交标记字节是 2..=255 时读者当「没有提交标记」、记录照样在；checker（`walk.rs:3065` `                CommitMarker::Unrecognized(byte) => {`）判「只许 0 或 1」违例。两边判的不是一件事，但走到它要一条校验和恰好对上的坏记录，合法历史到不了；D23（journal 的角色与格式） 已定项 7 没写其余值。替没写的条款做了选择，影响只在坏盘上，今天没有会红的东西钉读者这一侧。

### Z1-3　已知、没重打

背景材料「已知」第 4 条（锚点读得出时首条序号不是 1、末条标志之后还有同 txg 的记录，C539、C540）与第 5 条（原样重发）都没再造镜像。

## 四、Z6　三份补丁在 `mount.rs` 的合并点

### Z6-1　结论：`establish_instance` 那条断言在扫过的形态上没有 panic、也没有判错（没打中）；合并点上打中的是 Z4-1

`mount.rs:1659` 起那条 `assert_eq!(&placements_taken, planned, …)` 比的是拷贝上与真发的 (角色, 槽) 逐次序列。逐条对过的两侧：

| 这一步 | 拷贝上（`placements_of_the_publishes_after_acquisition_on_a_copy`） | 真发 |
|---|---|---|
| 带文件的一版，写行的角色 | `PublishShape::row_publish_rewriting_instance_table_pages(片数).rewritten_roles()` | `PublishPlan::resolve`：没有文件内容、不碰 inode 树时同一张表（实例表各片尾片先，再四个固定点） |
| 带文件的一版，写行换下的 | 旧链（`instance_table_chain_to_release`）在前、经映射查到的在后 | `publish_version` 同序 |
| 暖机换下的 | 上一次在拷贝上取到的四个固定点 | 经映射查上一版（分配记录树、记账树）与根记录（映射树、树表） |
| 树表 0 条的一版，写行 | 实例表各片尾片先 + 分配记录树节点；不释放旧链 | 先释放旧链与旧节点再取落点——释放代是这一次的 txg，同一次发布里回收不到；之后只有零单元发布，所以取到的相同 |
| 每次取完 | `record_root_written_by_this_process(txg)` | 同一个调用、同一个 txg |

**用户动作放开扫过的**（草稿副本上）：

| 用例 | 取样 | 结局 |
|---|---|---|
| `z6_a4_rollback_to_every_ring_root_across_shapes` | 起点 {带文件, 树表 0 条} × 取号之后崩溃次数 {0, 366, 367, 368, 369}（实例表一片 / 两片）× 之后可写挂载 {1..=8} 次 × 回退目标取根环里每一条可读根（含最旧那条与 mkfs 的第 0 代根）；回退之后再普通挂载一次 | 1225 个组合、20 类结局，全是 `rollback ok … violated=[] \| remount ok violated=[]`，没有 panic（`logs/a4.log`） |
| `z6_a1_…` / `z1_a1b_…` | 带文件的一版 64 片（写行那次末条再跨记录），挂载两次；第二次挂载 164 个前缀崩溃状态 + 16 个记录写子集 | 断言没 panic，checker 全绿（见 Z1-1） |
| `z6_a2_random_histories_on_fresh_seeds` | 门禁窗口之外的新种子：三组比重（BROAD、ROLLBACK_AFTER_RAISING_THE_FLOOR、REUSE_AFTER_RAISING_THE_FLOOR）各 1500 个种子 × 每段 60 步，每步之后池级 checker 与理想模型，panic 被执行器接住记成新发现 | 三组各 1500 种子 × 60 步：新发现 0（已知红 107 / 14 / 45 段，全是清单里那一条「回退之后抬 F」）（`logs/a2-large.log` 第 7、85、163 行） |

**「不该相同时判相同」那一半**：断言只在 `placements_planned` 是 `Some` 且这一串里没有一份被隔离时比。两种不比的格都已登记：`None` 是 Z4-4（已知，实二三），被隔离是 Z4-5（C542）。另一种「拷贝上取得到、真发在取号之后因为别的原因失败」的格就是 Z4-1：树表 0 条那一臂写行那次的条数准入不在拷贝那一遍里、也不在取号之前那道准入里，断言根本走不到。它落在合并点上：实十六接续的多片写路径让树表 0 条的一版能写到 40 片以上，C512 让那一臂的写行发布长出分配记录树与它的条数准入，实十八的取号之前那一串只对带文件的一臂判条数。

### Z6-2　已知、没重打

背景材料「已知」第 3 条：树表 0 条那一版写行、片数 ≥ 67 时 `ByteWriter` 越界（我的 Z4-1 扫到 66 片为止，没碰这一格）。

## 五、没打中的形状

| 格 | 试过的形状 | 取样范围 |
|---|---|---|
| Z1 | 64 片写行那次发布末条再跨两条（事务号 0）：写者写的序号、标志、提交标记；这次挂载的每个前缀崩溃状态；四个记录写的全部子集；每个状态上恢复 + 可写挂载 + checker | 164 个前缀 + 16 个子集（`logs/a1b.log`） |
| Z1 | 锚点读法乙在「所选根那次发布的末条读不出、前几条读得出」「下一次发布首条读不出」「txg 跳一格」三种形状上按代码逐步推 | 只推没跑：`second_transaction_parallel_line_one_last_record_flag.rs` 已有这几格的用例，我没重复 |
| Z1 | 写者 `roles_named_by_each_record_of_the_publish` / `transaction_offset_of_each_record_of_the_publish` 与条款逐句对（多数据单元 + 末条再跨同时出现） | 按代码推：带文件的发布最多 extent 叶容量那么多个数据单元，最后一个事务点名 8 项左右，今天走不到两者同时出现 |
| Z4 | 取号之前那道准入与发布路径那一遍的条数算法（基数只加不减、复用已回收记录不加条数） | 按代码推：真发的条数 ≤ 预估，带文件的一臂不会在取号之后被条数拒 |
| Z4 | 抬 F 那一串：第 1 次空发布的记录写、系统配置写，第 2 次的根写、系统配置写各注入一次块设备错；之后同一进程覆盖写、另起进程重挂覆盖写 | 4 个注入点 + 对照（`logs/a3.log`）；打中的都归已知（Z4-2、Z4-3） |
| Z6 | 见第四节那张表 | 1225 个回退组合；新种子随机历史三组 × 1500 × 60 步 |

没打中的那几格只抽了一次样（一条腿、一次派发），照 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」不能拿去撑「没问题」。

## 六、这条腿自己的限度

- 全部数字是草稿副本（`/tmp/claude-1000/m2-final-code-r1-opus/tree/`，由冻结副本 rsync 而来、只加了一个测试文件）与改法副本（`…/tree-fix/`）上量的，不是入库装置上的数；要引，得在入库装置上重做。
- Z4-1 用「取号之后崩溃」两万多次把表推到 40 片以上；纯挂载的对照只跑到 1200 次（4 片），「纯挂载一万四千多次之后同样烧号」是按片数与条数的关系推的，没跑。
- 包装设备 `FailTheNthWrite` 是我自己写的，不是 harness 的 `FaultInjectingBlockDevice`；它只在第 n 次写上报错、一个字节不落，不模拟写撕裂与吞写。
- 随机历史的生成器只写一个数据单元以内的内容、不建 inode、不崩溃，Z1 的末条再跨记录在那里碰不到；Z1 只靠 64 片那一条历史覆盖。
- 改法 Fx 只在我的模型上量过一格（66 片），被攻过零轮；它不修吸收态。

## 七、没做什么

- 没读、没判 Z2、Z3、Z5；没读别的腿的产出。
- 没跑名字含 layer0 的测试二进制，没跑 `cargo test` 全量、`gate.sh`、崩溃注入与故障注入的整轮；只跑了自己写的 `opus_r1_attack` 这一个测试目标。
- Z4-4（释放核验报错不在取号之前拒）没造坏镜像跑，只引了代码行。
- 背景材料「已知」六条都没再造历史去打。
- 冻结副本、主工作区的 `crates/` 一个字节没动。

## 附录：引到的 kb 条款（整行抄，行号是 kb 文件自己的，`awk 'NR==行号'` 取出）

### A1　`.claude/kb/decisions/18-块里携带什么信息.md` 第 314 行

````markdown
  - **可写挂载的顺序**：先判这次能不能可写（可写设备数够 w 的下限 ∧ **独占打开池中过半的设备**——任意两个过半集合相交，每设备的独占打开才成为池级互斥；这条与它的代价（4 盘池只剩 2 块可写时只能只读）是 D2（RAID 条带策略） 已定项 13 定的，不是已定项 11 ∧ 实例表可读 ∧ 实例切换的预留拿得到 ∧ 这次挂载要发的写行与暖机在分配器的副本上预演取得到全部落点——预演里写行那次的释放核验报错，同样判这次不能可写；预演不做释放前读盘核，差别见 C542（取号前预演不读盘核，坏盘加回退到最老根会烧号）），再取新代号 = max(**这次挂载独占打开成功的那个集合**里各系统配置的代号（每块盘两槽里全部自证过的槽——校验和过且 fsid 与本池相同）, 根环里全部根记录的实例代号) + 1 并写进**那个集合**里的每一份系统配置、**全或无**（写系统配置与数据单元写同一个取向：一次 I/O 错要重试到 T_retry 用尽才算失败；用尽后全或无判失败时**先把已经写出的那几份回卷成旧代号**（旧代号 = 取号之前这个集合里全部自证过的槽中最大的实例代号；回卷写发生在任何单元之前），回卷不成才只读挂载——回卷期间盘上出现的「同一集合内代号不等」由 I-7.7（系统配置实例代号不低于根环） 的第 ② 句判成立（较大的号没有任何根、记录、单元带着），不另开例外，下一次挂载按 max + 1 修复；取号之后那道屏障（D23（journal 的角色与格式） 已定项 16）在任一块盘上报错同样判全或无失败，不许只重发屏障就继续；取号的写入集合与 max 的取值集合取同一个——写「全部可见」而只独占一部分时，集合外那些盘写不写得进去不由这个挂载说了算，两个挂载能互相逼成只读。**过半是准入门槛，不是写入范围**：写入集合取独占打开成功的全部设备，只写过半会把集合外的健康盘落下、它们的代号从此是旧的，与真正错过取号的盘分不开。**「可见」= 独占打开成功且系统配置读得通**；I-7.7（系统配置实例代号不低于根环） 的第 ② 句限定到同一个集合），之后才动任何单元；只读挂载不取号；从 1 起、0 无效。
````

### A2　`.claude/kb/checks-owed.md` 第 329 行

````markdown
| C378 | 取号之后写行发布被拒，已烧掉的实例代号不回卷 | 2026-09-18 起写行与暖机那几次发布的准入都在取号之前算（`crates/singlefs-core/src/mount.rs` 的 `refuse_publishes_before_acquisition_that_do_not_pass_admission`，错误成员 `RowPublishAdmissionRefusedBeforeAcquisition`、`WarmUpAdmissionRefusedBeforeAcquisition`；2026-09-18 用户定案暖机那几次的准入也挪到取号之前），被准入拒不再烧号：用例 `second_transaction_supplement_two_row_publish_admission.rs` 钉「两块盘逐字节不变、四个系统配置槽的实例代号不变」，`crates/mutations.tsv` 第 107、111、112 行钉着。还开着的是取号之后因块设备报错失败的那一路：写行或暖机那几次写报错时代号已经烧了，回卷只管取号自己那几次写（`mount.rs` 第 849 行注释）。 | 造「取号之后写行或暖机那几次写报块设备错」的历史，按用户定的那一边断言（回卷：失败之后系统配置槽的实例代号退回；认了：记进失败账并在报告里点名）；判别力自证：把定下来的那一边改回去必须红 | 用户 2026-09-23 定「认了，写成已知行为」（D23（journal 的角色与格式） 已定项 16 的射程）；按「认了」断言的用例已落（2026-09-23）：写行那次写报错、暖机那次写报错两格，断言重开之后系统配置仍是 2、失败账记一次（`FaultInjectionTally::acquired_instances_left_after_a_failed_mount`，单开的一本账，不进已知红清单）、报告里点名；待代码三方与崩溃验证（里程碑「第二个事务」增补 2 收口表第 57 行） | 代码三方 `m2-wave1-code-r1` 第一轮云端攻方「拒绝时盘上不是逐字节不变」那一格，主 agent 现查 `mount.rs` 坐实；2026-09-19 重核（`m2-closeout-recheck-d.md` 第 20a 行与「表外」第 10 条）改写成剩下的题面 |
````

### A3　`.claude/kb/milestone/02-second-txn.md` 第 371 行

````markdown
| 57 | 取号（两次系统配置槽写 + 屏障）在写行发布之前，发布被拒时不回卷：实例代号已经 1 → 2、录制流多 3 步，池此后每试一次可写挂载再烧一个代号；今天可达（分配记录树满了的池，第 50 次覆盖写被准入挡住之后每次可写挂载都被挡） | 实现缺口（代码三方第一轮打中，主 agent 现查 `mount.rs` 的 `establish_instance` 次序坐实）；2026-09-17 已改（主 agent 现查代码）：与内容无关的两条准入抽成只读的 `transaction::publish_admission`，可写挂载按 `PublishShape::ROW_PUBLISH` 的角色表在取号之前算，算不过返回 `MountError::RowPublishAdmissionRefusedBeforeAcquisition`，一个写都不发；用例钉「812 条正好装满、写行要 822 ⇒ 被拒，两块盘逐字节不变、四个槽的实例代号都还是 1」。⚠️ 只算写行那一次：写行之后的 1–3 次暖机空发布没进这一遍，「写行装得下、暖机装不下」那一格仍是取号写完才报错；映射与树表也没有条数上限（树表那格是第 28 行） | 不等 | 2026-09-18 用户定案：暖机那几次的准入也挪到取号之前，回卷那一格先不定。同日实现（`PublishShape::EMPTY_PUBLISH`、`publish_sequence_admission`、`MountError::WarmUpAdmissionRefusedBeforeAcquisition`、`mount::warm_up_publish_txgs` 让准入与推发布共用一份计划），两条新用例（暖机第 1 次、第 2 次装不下各一条）、三条变异。⚠️ 实现员量出的新问题：这套准入算的是上界（每次按「角色数 × 盘数」加，连乘几次），而真发起来重开之后释放掉的落点是改写不是追加——它量的那个池按上界在暖机第一次就被拒，真跑起来条数没到（796 → 798 → 798，容量 812）⇒ 新准入会拒掉发得起来的池，而「写行装得下、暖机第 N 次装不下」在真实分配下今天造不出来。要不要把可复用的已释放记录抵扣掉（抵扣多少取决于分配器真会挑哪些槽）是设计判断，交用户；改完走第二轮。抵扣那一问 2026-09-18 用户在第 39 行定了（保留上界）；回卷还是认了：**用户 2026-09-23 定「认了，写成已知行为」**，写进 D23（journal 的角色与格式） 已定项 16 的射程；C378（取号之后写行发布被拒，已烧掉的实例代号不回卷） 按「认了」断言的用例已落（2026-09-23）：写行那次写报错、暖机那次写报错两格，断言重开之后系统配置仍是 2、失败账记一次（`FaultInjectionTally::acquired_instances_left_after_a_failed_mount`，单开的一本账，不进已知红清单）、报告里点名；待代码三方与崩溃验证 | C378（取号之后写行发布被拒，已烧掉的实例代号不回卷）；`research/prompts/m2-wave1-code-r1-main-verification.md` 第二节第 1 行 |
````

### A4　`.claude/kb/decisions/23-journal的角色与格式.md` 第 145 行

````markdown
- **本次发布内序号 4 字节**（无符号 32 位，从 1 起：一次发布切成 N 条记录时依次是 1..N，只有一条时是 1；空发布记录也写 1）：已定项 14 注 1 在所选根那条记录读不出时靠它接链首。4 字节的宽度与紧跟事务号、提交标记的落点是主 agent 按「不为省空间牺牲自包含」取的，可推翻。字段在头里的偏移：事务号 78、提交标记 86、本次发布内序号 87、反向链 91、载荷校验和 95、新根段 99、fsid 287、MAC 295、头末 311。一次发布的 N 条记录序号依次 1..N、与 jsn 同步；读者遇到这四格，都当那条记录损坏、断链即止，那次发布整体不施加：序号 0；一次发布之内跳号；锚点读得出时，下一次发布的首条序号不是 1（断在这一条）；同一 (实例代号, checkpoint_txg) 里带末条标志的那条之后还有记录（断在带标志的那条）。与 I-8.9（一次发布的记录序号连续且只有末条带标志） 判的是同一件事。
````

### A5　`.claude/kb/decisions/23-journal的角色与格式.md` 第 354 行

````markdown
⚠️ **第六条：施加的单位是一次发布——合法前缀停在一次发布的两个事务之间时，那次发布整体不施加，前五条判出来的前缀再按发布边界截短一次；引用这五条时连这一条一起引。**（D16（发布语义） 已定项 4）**发布边界怎么认**：一次发布的末条 = 记录标志位 0 为 1 的那一条（已定项 4、17）；前缀里一次发布的末条没到，那次发布整体不施加。
````

### A6　`.claude/kb/decisions/23-journal的角色与格式.md` 第 362 行

````markdown
1. **前缀规则不跨实例边界**：链从所选根覆盖的最后一条记录之后接，下一条的实例代号与所选根不同即停（六条口径第一条原样）。**所选根覆盖的最后一条** = 与所选根同实例、同 `checkpoint_txg` 的记录里 jsn 最大的那条——一次发布切成多条记录时它们共享一个 `checkpoint_txg`（D16（发布语义） 已定项 6），那次发布的最后一条才是根覆盖到的末端。**那条读不出时**，链首接水位之上第一条可读记录，前提是它的「本次发布内序号」为 1（已定项 4）、`checkpoint_txg` = 所选根的 txg + 1；序号不是 1 说明下一次发布的开头缺了，断号即止。**「那条」按末条标志认**——所选根那次发布读得出的几条里没有一条带末条标志，就算「那条读不出」，走「序号为 1 的第一条可读记录当链首」那一支；不取读得出的同 txg 记录里 jsn 最大的那条（用户 2026-09-24 定，读法乙；三方 `research/prompts/m2-presumed-clauses-r1-main-verification.md` K4 那一支四条臂同中、没分出候选，这一读法被攻过零轮）。所选根是 mkfs 的第 0 代根时它一条记录都不覆盖，之后的记录全属于更新的实例 ⇒ 一条都不施加。代价写在 D16（发布语义） 已定项 7 的注，要不要让新实例先暖机是 D16（发布语义） 已定项 8。
````

### A7　`.claude/kb/decisions/23-journal的角色与格式.md` 第 453 行

````markdown
- **共享的提交内生块多于最后一条记录装得下（4096 记录 67 项）时，末条再跨记录**：从最后一个事务那条记录起依次多写几条，装满一条再开下一条，只有真正的最后一条带「本次发布末条」标志；恢复按这个标志认一次发布的边界，一次发布的记录里没读到带标志的那条，整次不施加（已定项 14 第六条）。标志放在记录头十个字段里「填充 1」那个字节（改名「记录标志 1」），位 0 = 本次发布末条；每一次只有一条记录的发布（含空发布记录），那一条的位 0 也写 1——改第一个事务的字节（w1、w4、t9 三条记录的这一字节与头部校验和）。用户 2026-09-24 定。
````

### A8　`.claude/kb/decisions/23-journal的角色与格式.md` 第 383 行

````markdown
**这一版的失败处置**：发布不接受失败。一次发布失败之后把它冻结，下一次发布之前先逐字节原样重发它（checkpoint_txg、计数器、本次发布内序号、记录标志、单元的位置与字节都不变），重发成功才建下一次发布；于是合法历史里同一 (实例代号, checkpoint_txg) 不会有两条带末条标志的记录，恢复遇到它就是盘坏了，在任何写之前停下（`crates/singlefs-core/src/recovery.rs` 的 `RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided`）。原样重发管这一版的串行提交；探针写、两支分流、实例切换与根槽写失败推进 txg 仍在本项，随失败处置的代码在后面的里程碑做。原样重发在串行提交下会把一次失败放大成整条发布流阻塞，欠 C541（原样重发把一次失败放大成整条发布流阻塞）。
````

### A9　`.claude/kb/checks-owed.md` 第 332 行

````markdown
| C381 | 根已落盘之后发布失败，分配器仍退回 | 发布的持久顺序是根记录 FUA 之后再转系统配置槽；系统配置槽那一步失败时根已落盘，恢复会选中它，而发布路径把分配器整个退回到这次发布之前（`crates/singlefs-core/src/transaction.rs` 第 1221、1258 行）；同一个写入口接着发下一次发布会把这次占过的槽再分出去，写到一半断电就把已落盘的根引用的单元盖掉（代码三方第二轮攻方：checker 五条红、重开读不出单元） | 造「系统配置槽写失败 → 同一个写入口发一次空发布、单元写完记录与根没写时断电」这段历史，恢复之后 checker 全绿、重开成功——今天 checker 五条红、重开报 `Recovery(UnitUnreadable)`；判别力自证：原样重试同内容的那条对照今天就绿 | D16（发布语义） 已定项 7 的失败语义要先定：根落盘之后的失败算不算这次发布成立，失败之后这个写入口还许不许接着发布；2026-09-18 用户定走三方立题。第一轮 2026-09-19 已判（`research/prompts/c381-r1-main-verification.md`）：今天的「一律退回」在攻方原历史与「根槽 FUA 报错而根已落盘」两处坐实是缺陷；D23（journal 的角色与格式） 已定项 14 的失败表（先探针写，瞬时走实例切换、持续转只读到下次挂载）已经答了失败之后写入口怎么走，`crates/` 里探针写、实例切换、转只读三样都没有；第二、三轮都判完了（2026-09-21）：第二轮判出三条候选栽在同一个共用前提上——失败表要的探针写盘上没落点、代码里没实现、判别力上分不出落点级的故障；用户定了两题（探针落点进地址空间表、失败表射程逐个失败点列表）、打回两题；第三轮攻那两题，候选甲（不立条款、明写 fsync 报错之后数据可以出现也可以不出现）站住并要加两样，候选乙（探针写改成写失败的那个落点）被推翻。按「第三轮之后停」这条线到此为止。四题用户都定了：第二轮定第 1 题（探针落点进地址空间表）与第 3 题（失败表射程逐个失败点列表），第三轮定第 2 题（按候选甲写回并加两样，`D16（发布语义）` 新立一条已定项）与第 4 题（改法 B：失败表判别子改成固定落点探针写 + 失败落点只读复核），「实例切换取的是内存里的根」那一题 2026-09-21 定另立一题，2026-09-23 用户改定「改条款：切换要重新读盘」（C458（实例切换取内存里的根，不重新读盘）），条款写进 D23（journal 的角色与格式） 已定项 14 与 D18（块里携带什么信息） 已定项 11。2026-09-21 写回的是第 2、4 题；第二轮那两题各自还缺落地前提（落点登记在哪张表与哪个偏移、射程那八格怎么判），各记一笔欠账。**代码那一半整个仍欠**：`crates/` 里探针写、只读复核、实例切换、转只读四样都没有。**事务号也在重用之列**——根槽那一步失败之后事务号已经进了持久记录，重试拿同一个号、jsn 也重、实例没换（判决判定三，攻方探针 `opus_probe_txn_chain.rs` 复跑坐实；⚠️「改回旧写法同样中」那句是推理坐实、没有 A/B 实测）。这一族要的是 `D23（journal 的角色与格式）` 已定项 7 后半句「失败即实例切换」落地，而 `crates/` 里探针写、只读复核、实例切换、转只读四样都没有。用户 2026-09-23 划一刀：代码那一半跟里程碑「第二个事务」收口表第 16 行一起挪到后面的里程碑。 | `research/prompts/m2-wave2-code-r1-main-verification.md`、`research/prompts/c381-r2-main-verification.md`、`research/prompts/c381-r3-main-verification.md` 第二节第 7 行；攻方报告 `research/prompts/m2-wave2-code-r1-main-verification.md` 第三节 |
````

### A10　`.claude/kb/checks-owed.md` 第 502 行

````markdown
| C516 | 抬 F 那一串发布被拒时前面几次已落盘 | 用户 2026-09-24 定维持逐次发布；实三加了 `MountError::RaiseFloorSequencePublishFailed { publishes_persisted, cause }` 交回已落盘几次。 | 2026-09-24 |
````

### A11　`.claude/kb/checks-owed.md` 第 504 行

````markdown
| C512 | 树表 0 条的一版上被换下的实例表记在哪没有条款 | 条款：树表 0 条的一版上写行那次发布写实例表与一片分配记录节点两个单元，分配记录树根指针住根记录（D16（发布语义） 已定项 9；D22（单元原子性怎么合成） 已定项 7，根记录 371 → 457 字节），重开时从那片节点重建这一版的账，用户 2026-09-23 定案。实现：`crates/singlefs-core/src/root_record.rs` 的 `allocation_record_tree_root`（[342, 428)）；`crates/singlefs-core/src/transaction.rs` 写行那次发布建这片节点、换下上一版实例表与上一片节点；`crates/singlefs-core/src/mount.rs` 的 `allocator_of_version_without_file` 从这片节点重建账；`crates/singlefs-checker/src/walk.rs` 的 `walk_root` 走读它。用例 `the_third_writable_mount_keeps_the_instance_table_that_the_row_publish_swapped_out_out_of_the_free_pool`（建池 → 挂载 → 退出 → 挂载写行 → 退出 → 第三次挂载，断言被换下的那片实例表没被当空闲槽发出去）；`crates/mutations.tsv` 里这一笔的两条变异（不把根写进根记录、重建时丢掉带已释放标志的记录）都红在它上面。R8 那道拒绝（`MountError::VersionWithoutFileNotWrittenByMakeFilesystem`）留着，依据换成「根记录那一项全零而实例表或树表不是 mkfs 那一版」，只对坏镜像与外来镜像说话，`a_third_writable_mount_on_a_version_whose_rows_were_written_is_refused_before_touching_the_allocator` 钉住 | 2026-09-23 |
````

### A12　`.claude/kb/checks-owed.md` 第 478 行

````markdown
| C542 | 取号前预演不读盘核，坏盘加回退到最老根会烧号 | 取号之前在分配器副本上预演写行与暖机（D18（块里携带什么信息） 已定项 11「可写挂载的顺序」）时，不做释放前读盘核校验和、对不上就隔离那一步；这一串里有一份换下的单元核出对不上而被隔离，并且这一串自己的根把环里比它旧的有效根全盖掉（回退到环里最旧的那条根）时，预演判取得到，真发在取号之后被落点拒绝，烧掉一个实例代号，不丢数据 | 造这一格：坏一份单元加回退到最老根，断言挂载在取号之前拒；判别力：今天的预演必须放行 | 用户 2026-09-24 定记欠账、不改：预演多读一遍盘会挪动已有故障注入用例的注入点 | 2026-09-24 实十八报告 `research/prompts/m2-followups2-implementer-report.md` 第三节 2 |
````

