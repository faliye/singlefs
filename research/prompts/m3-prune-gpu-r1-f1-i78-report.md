# 调查：数据单元第二个槽里像节点头的用户内容让 I-7.8 判红（2026-09-28）

主仓提交 e5253e8a；仓副本 `rsync -a --exclude target --exclude .git`，只在副本里加测试、加打印、改坏；主仓一个文件没写。
开跑前 `ps` 看负载：没有 qemu / vm-bench / e152 / fio，也没有别的 cargo、gate.sh 在跑。命令都经 `run-with-memory-cap.sh 8G` 与 `capped.sh 4`，`CARGO_TARGET_DIR` 用的是自己的目录。

## 结论

1. **是真的，属于池级 checker 的假红，池并没有违反 I-7.8。** 在 harness 档上造这个池（内存稀疏盘上依次 mkfs → 取号 → 暖机 → 新池新建文件，文件 17000 字节，内容第 16250 字节起放一个码 2 节点头），对全部写都持久之后的 `MemoryPool` 直接跑 `check_pool_image`，I-7.8 报 `Violated("根环水位最大 19，盘上出现过的最大树 ID 1099511627776")`，别的不变量没有一条判违反。恢复读回的正是写进去的 17000 字节。
   对照 I-7.8 原文：「出现过的」取的是「全部单元头的索引节点类身份段里那个树 ID」并上树表条目里的树 ID。那个头在 32K 数据单元里面（单元从槽 50180 开始，头在槽 50181 开头），是这个单元的载荷，不是一个单元头。不经 checker 另算一遍：每次单元写开头的码 2 头里树 ID 最大是 15，根环水位是 19，按定义 19 > 15，I-7.8 成立。
2. **机理**：扫描方向把每个候选槽的开头都当成可能的单元头去读，并不知道这个槽是不是落在另一个单元的中间。
   - 候选槽怎么来：`MemoryPool` 把写过的扇区各自换算成所在的槽（`crates/singlefs-harness/src/memory_pool.rs:644`），32K 数据单元写罩住 50180、50181 两个槽，两个都进候选。trait 的约定是「None = 从单元区起点扫到盘尾」（`crates/singlefs-checker/src/image.rs:24`），照这个约定整盘扫同样会读到 50181，所以不能说是 `MemoryPool` 把候选给多了；把候选改成 None 实测一样判红（见下）。叠加形态 `CrashImage` 只把单元写的第一个槽加进候选（`memory_pool.rs:747`），它的候选比 None 约定的整盘扫还窄，所以它读不到 50181。
   - checker 读法：`crates/singlefs-checker/src/walk.rs:2405`–`2407` 取候选槽，候选为 None 时退成整个单元区；`:2413` 只看开头的 magic `SFSU` 与码 2；`:2417` 查头校验和；`:2419` 看诞生代号 ≤ 根环最大 txg，并排除 `written_after_its_instance_was_last_applied`（`:2377`）判出来的孤儿；`:2427` 把树 ID 收进集合。`:6249` 再并上树表条目里的树 ID，`:6251` 做判定。
   - 头的四道各自过没过：
     - 头校验和：过。它是不带密钥的 CRC32C，把 10..42 清零后算、结果放在前 4 字节（`crates/singlefs-checker/src/lib.rs:82`–`91`），写内容的人自己就算得出来。
     - fsid：`:2396`–`2433` 这段计数路径根本不比 fsid（kb 的 I-7.8 行也写了「节点头 fsid 不是本池的照旧数」）。伪造的头是从本池一个真节点抄来的，fsid 本来就相同，只影响写序实例读不读得出来：读出来是 `write_order_instance=Some(1)`。
     - 诞生代号：伪造成 1，≤ 最新发布 txg 3，过。
     - 已发布谓词的第二道：打印出来的实例表行是 `rows=[]`，`excluded_by_instance_row=false`。另外只要诞生代号是 1，任何 Ti ≥ 1 都满足不了 `birth_txg > Ti`（`:2387`），这一道挡不住它。
   - 同一个 checker 自己在 `walk.rs:4624` 写着「32768 的单元占两个槽，第二个槽上没有 magic，扫描自然跳过它」：「第二个槽开头不会有 magic」这个前提，用户内容就能打破。
3. **最小复现**：一个 harness 档测试 `investigate_i78_minimal`，在今天的代码上失败（它断言 I-7.8 成立，实际判了违反）。全文与原样输出在下面。
4. **推翻条件**：见「推翻条件」一节，每条都在副本里造过一次。

## 没跑原现场

原现场是 E161 装置里的攻方原型，属于 checker 档的二进制。派发只允许跑 harness 档，所以这里没跑，只引主 agent 在副本上复跑、逐行相同的那次结果。这边的复现换成 harness 档上独立造的池，没有经过 E161 装置、`attack.rs` 或 checker 档；造法照 `attack.rs:1156` 起那个世界的描述，代码是重写的，不是抄的。

## 复现：全持久 MemoryPool 与几组对照（测试 `investigate_i78_data_unit_interior`，今天的代码）

命令（在副本根目录）：

```
CARGO_TARGET_DIR=/tmp/claude-1000/investigate-f1-i78/target nice -n 19 bash research/scripts/run-with-memory-cap.sh 8G bash research/scripts/capped.sh 4 cargo test -p singlefs-harness --test investigate_i78_data_unit_interior -- --nocapture --test-threads=4
```

退出码 0；run1.log 原样全文（24 行）：

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.00s
     Running tests/investigate_i78_data_unit_interior.rs (/tmp/claude-1000/investigate-f1-i78/target/debug/deps/investigate_i78_data_unit_interior-685bf09d36de19ad)

running 7 tests
name=carriers header_bytes=134 data_units_device_start_slot=[(0, 50180), (1, 50180)]
name=recover content_matches=true outcome_is_file_read=true
name=definition watermark_by_core=19 newest_txg_by_core=3 highest_tree_id_at_unit_write_starts=15 forged=1099511627776
name=memory_pool_candidates second_slot_listed=[(0, 50181, true), (1, 50181, true)]
name=birth_above_newest birth=4 carriers=[(0, 50180), (1, 50180)] i78=Holds
name=shifted_512 i78=Holds
test control_header_with_a_birth_txg_above_every_published_root_keeps_i78_holding ... ok
test control_header_shifted_512_bytes_off_the_slot_start_keeps_i78_holding ... ok
name=broken_checksum carriers=[(0, 50180), (1, 50180)] i78=Holds
test control_header_with_a_broken_checksum_keeps_i78_holding ... ok
name=final_memory_pool not_holding=[("I-7.8", Violated("根环水位最大 19，盘上出现过的最大树 ID 1099511627776")), ("I-7.9", NotApplicable("根环里没有抬 F 的根：每条根带的 F 都不高于同一实例里它前一条根带的（回退与新实例的第一条根不算抬）")), ("I-8.7", NotApplicable("环里没有哪个实例写出过两条非 0 事务号的记录：一对可比的号都凑不出来")), ("I-8.8", NotApplicable("环里没有哪一组（同一块盘上实例代号与事务号都相同的非 0 记录）落了两条以上、也没有哪一组一条提交标记都没有：③ ④ 没有判的对象，提交标记字节单独成立不算这一条成立")), ("I-9.14", NotApplicable("没有一棵树的树表条目出现在两个不同的树表单元里：跨根比不出来"))]
name=final_pool_from_device_images not_holding=[("I-7.8", Violated("根环水位最大 19，盘上出现过的最大树 ID 1099511627776")), ("I-7.9", NotApplicable("根环里没有抬 F 的根：每条根带的 F 都不高于同一实例里它前一条根带的（回退与新实例的第一条根不算抬）")), ("I-8.7", NotApplicable("环里没有哪个实例写出过两条非 0 事务号的记录：一对可比的号都凑不出来")), ("I-8.8", NotApplicable("环里没有哪一组（同一块盘上实例代号与事务号都相同的非 0 记录）落了两条以上、也没有哪一组一条提交标记都没有：③ ④ 没有判的对象，提交标记字节单独成立不算这一条成立")), ("I-9.14", NotApplicable("没有一棵树的树表条目出现在两个不同的树表单元里：跨根比不出来"))]
name=overlay second_slot_listed=[(0, 50181, false), (1, 50181, false)] i78=Holds
test overlay_of_every_write_on_the_mkfs_base_keeps_i78_holding ... ok
name=without_second_slots removed=[(0, 50181), (1, 50181)] i78=Holds
test a_node_header_in_the_second_slot_of_a_data_unit_turns_i78_red_on_the_fully_persisted_memory_pool ... ok
name=control i78=Holds not_holding=[("I-7.9", NotApplicable("根环里没有抬 F 的根：每条根带的 F 都不高于同一实例里它前一条根带的（回退与新实例的第一条根不算抬）")), ("I-8.7", NotApplicable("环里没有哪个实例写出过两条非 0 事务号的记录：一对可比的号都凑不出来")), ("I-8.8", NotApplicable("环里没有哪一组（同一块盘上实例代号与事务号都相同的非 0 记录）落了两条以上、也没有哪一组一条提交标记都没有：③ ④ 没有判的对象，提交标记字节单独成立不算这一条成立")), ("I-9.14", NotApplicable("没有一棵树的树表条目出现在两个不同的树表单元里：跨根比不出来"))]
test control_same_history_without_the_forged_header_keeps_i78_holding ... ok
name=full_scan_none_candidates i78=Violated("根环水位最大 19，盘上出现过的最大树 ID 1099511627776")
test removing_only_the_second_slots_from_the_candidates_turns_i78_back_to_holding_and_a_full_scan_keeps_it_red ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.67s

```

每一行说明什么：

- `name=carriers`：头落在两块盘的数据单元（起始槽 50180）第二个槽开头。
- `name=recover`：恢复读回的内容就是写进去的内容，这个池是健康的。
- `name=definition`：用 core 的 `readable_roots` 读出来的水位是 19；不经 checker，只看每次单元写开头的码 2 头，树 ID 最大是 15。
- `name=memory_pool_candidates`：50181 在 `MemoryPool` 的候选里。
- `name=final_memory_pool` 与 `name=final_pool_from_device_images`：两条不同路子造出的池（前者是把录制流施加到空内存盘上，后者是直接取写完之后的稀疏盘镜像）都只有 I-7.8 判违反，其余都是成立或不适用。
- `name=control`：同一段历史、内容里不放头，I-7.8 成立。
- `name=shifted_512`：头往后挪 512 字节，不在槽开头了，I-7.8 成立。
- `name=birth_above_newest`：头照样落在第二个槽开头，只是诞生代号改成 4（大于最新发布的 3），I-7.8 成立。
- `name=broken_checksum`：头照样落在第二个槽开头，只是头校验和翻了一位，I-7.8 成立。
- `name=without_second_slots`：同一个池，只把两块盘的 50181 从候选里拿掉，I-7.8 成立。
- `name=full_scan_none_candidates`：候选改成 None（照约定整盘扫），I-7.8 仍判违反。
- `name=overlay`：mkfs 之后的基线叠上全部写，候选里没有 50181，I-7.8 成立。这就是层 0 叠加那一侧，与攻方的 `overlay_red=[]` 对得上。

阴性对照不是「代码没跑到」：birth 与 checksum 两组的 `carriers` 行说明头确实落在第二个槽开头；overlay 与 without 两组的候选里确实没有 50181；shifted 按构造就不在槽开头。

### 在副本的 walk.rs 里加打印（`scanned_tree_identifiers` 里树 ID ≥ 2^40 时打一行），再跑同一个测试

以 `--test-threads=1` 跑，退出码 0。下面是 `grep -E '^name=investigate_scan|^test control_header_with_a_birth.*investigate_scan' run2.log | sed 's/^test control_header_with_a_birth_txg_above_every_published_root_keeps_i78_holding \.\.\. //' | sort | uniq -c` 的原样输出：

```
      4 name=investigate_scan device=0 slot=50181 tree_id=1099511627776 birth_txg=1 newest_published_txg=3 write_order_instance=Some(1) rows=[] excluded_by_instance_row=false counted=true
      1 name=investigate_scan device=0 slot=50181 tree_id=1099511627776 birth_txg=4 newest_published_txg=3 write_order_instance=Some(1) rows=[] excluded_by_instance_row=false counted=false
      4 name=investigate_scan device=1 slot=50181 tree_id=1099511627776 birth_txg=1 newest_published_txg=3 write_order_instance=Some(1) rows=[] excluded_by_instance_row=false counted=true
      1 name=investigate_scan device=1 slot=50181 tree_id=1099511627776 birth_txg=4 newest_published_txg=3 write_order_instance=Some(1) rows=[] excluded_by_instance_row=false counted=false
```

（诞生代号 4 那一组有一行和 `test …` 前缀粘在同一行输出里，所以用 sed 把前缀剥掉再计数。打印代码随后删掉了，walk.rs 用主仓原件拷回，`diff -rq` 核过只多出两个测试文件。）

## 最小复现（`crates/singlefs-harness/tests/investigate_i78_minimal.rs`，只在副本里）

全文（79 行，sha256 `18ef576aba20a2c3e9d0dc8ab0e1943f2179548f9a24d7dbf7336be5b4a196d4`；副本删掉之后这份留在 `/tmp/claude-1000/investigate-f1-i78/kept/`）：

```rust
//! 最小复现（不入库）：健康池（第一个文件 17000 字节，内容第 16250 字节起是一个码 2 节点头的字节）全部持久之后，
//! I-7.8 应当成立——数据单元内部不是单元头。今天的池级 checker 判它违反。

mod common;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::DeviceIdentity;
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{make_filesystem, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT};
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_core::transaction::{acquire_instance, publish_first_file, warm_up, FirstFile, PoolWriter};
use singlefs_format::{DATA_UNIT_PAYLOAD_OFFSET, SLOT_BYTES};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::{RecordingBlockDevice, RetainedOperation, SharedStream};

fn first_file_pool(content: &[u8]) -> Vec<RetainedOperation> {
    let parameters = common::parameters();
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<_> = (0..2u32)
        .map(|n| {
            let device = SparseBlockDevice::new(common::IMAGE_BYTES, PhysicalBlockSizeInBytes(512));
            (DeviceIdentity(n), RecordingBlockDevice::with_shared_stream(DeviceIdentity(n), device, stream.clone()))
        })
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), common::IMAGE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), common::IMAGE_BYTES),
    ]);
    allocator.mark_format_time_units(
        Placement { slot: INSTANCE_TABLE_SLOT, span: 2 },
        Placement { slot: TREE_TABLE_GENESIS_SLOT, span: 1 },
    );
    let mut writer = PoolWriter::new(&parameters, &mut devices);
    let instance = acquire_instance(&mut writer).expect("取号");
    let warm = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
    let file = FirstFile { content, write_time_seconds: common::FIXED_WRITE_TIME_SECONDS };
    publish_first_file(&mut writer, &mut allocator, warm.roots.last().expect("根"), file, instance, &warm.last_record_bytes)
        .expect("新池新建文件");
    stream.retained_operations()
}

fn memory_pool(operations: &[RetainedOperation]) -> MemoryPool {
    let mut pool = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], common::IMAGE_BYTES);
    pool.apply(operations);
    pool
}

#[test]
fn user_content_shaped_like_a_node_header_at_the_second_slot_of_a_data_unit_is_not_a_unit_header() {
    let mut content: Vec<u8> = (0..17000usize).map(|index| u8::try_from(index % 251).expect("字节")).collect();
    // 抄本池一个真的码 2 节点头（第一次写出来的那个），树 ID 改 2^40、诞生代号改 1、CRC32C 头校验和重算。
    let probe = first_file_pool(&content);
    let mut header = probe
        .iter()
        .filter_map(|operation| operation.contents.as_deref())
        .find(|bytes| bytes.len() == 16384 && &bytes[..4] == b"SFSU" && bytes[6] == 2)
        .expect("码 2 节点")
        .to_vec();
    let key_span = 2 * usize::from(header[51]);
    header.truncate(86 + key_span);
    header[42..50].copy_from_slice(&(1u64 << 40).to_le_bytes());
    header[52 + key_span..60 + key_span].copy_from_slice(&1u64.to_le_bytes());
    header[10..42].fill(0);
    let checksum = singlefs_checker::crc32_castagnoli_bitwise(&header);
    header[10..14].copy_from_slice(&checksum.to_le_bytes());
    let place = usize::try_from(SLOT_BYTES - DATA_UNIT_PAYLOAD_OFFSET).expect("16250");
    content[place..place + header.len()].copy_from_slice(&header);

    let pool = memory_pool(&first_file_pool(&content));
    assert!(
        matches!(recover(&pool, JournalPolicy::Consult).outcome, RecoveryOutcome::FileRead { content: ref read, .. } if *read == content),
        "健康池：恢复读回写进去的内容"
    );
    let verdict = check_pool_image(&pool).into_iter().find(|(name, _)| *name == "I-7.8").expect("I-7.8").1;
    assert_eq!(verdict, InvariantVerdict::Holds);
}
```

命令（在副本根目录）：

```
CARGO_TARGET_DIR=/tmp/claude-1000/investigate-f1-i78/target nice -n 19 bash research/scripts/run-with-memory-cap.sh 8G bash research/scripts/capped.sh 4 cargo test -p singlefs-harness --test investigate_i78_minimal
```

今天的代码（walk.rs 已从主仓拷回、`diff -rq` 只多出两个测试文件）上跑，退出码 101。原样输出（去掉 Compiling 行）：

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.62s
     Running tests/investigate_i78_minimal.rs (/tmp/claude-1000/investigate-f1-i78/target/debug/deps/investigate_i78_minimal-bd1a20a1f800c47c)

running 1 test
test user_content_shaped_like_a_node_header_at_the_second_slot_of_a_data_unit_is_not_a_unit_header ... FAILED

failures:

---- user_content_shaped_like_a_node_header_at_the_second_slot_of_a_data_unit_is_not_a_unit_header stdout ----

thread 'user_content_shaped_like_a_node_header_at_the_second_slot_of_a_data_unit_is_not_a_unit_header' (3263577) panicked at crates/singlefs-harness/tests/investigate_i78_minimal.rs:78:5:
assertion `left == right` failed
  left: Violated("根环水位最大 19，盘上出现过的最大树 ID 1099511627776")
 right: Holds
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    user_content_shaped_like_a_node_header_at_the_second_slot_of_a_data_unit_is_not_a_unit_header

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s

error: test failed, to rerun pass `-p singlefs-harness --test investigate_i78_minimal`
```

## 推翻条件，以及在副本里各造了一次

| # | 什么现象会推翻定位 | 造法 | 结果 |
|---|---|---|---|
| 1 | checker 不再把数据单元的第二个槽当单元头读之后，最小复现仍然红：说明红另有来源 | 副本 walk.rs 的 `scanned_tree_identifiers`：如果前一个槽开头是头校验和过的码 1（32K 数据单元），就跳过这一槽（diff 见下） | 最小复现变绿（退出码 0）；interior 测试里全持久池与整盘扫两组都变成成立 |
| 2 | 这个池按定义本来就违反 I-7.8：某个真的单元头或树表条目里有树 ID ≥ 19 | 不经 checker，取每次单元写开头的码 2 头；另做一个不放头的对照池 | 单元写开头最大是 15，水位 19；对照池 I-7.8 成立 |
| 3 | 红只是 `MemoryPool` 的候选给多了造成的，checker 照自己的约定整盘扫是绿的 | 候选改成 None | 仍判违反：`name=full_scan_none_candidates i78=Violated(...)` |
| 4 | 这个池本身不健康 | 恢复读回内容再逐字比 | `content_matches=true` |
| 5 | 头校验和或诞生代号那两道本来就能挡住它，只是这次没生效 | 只翻一位校验和；诞生代号改成 4 | 都变成成立，打印行 `counted=false`（诞生代号那一组） |

第 1 条的改法（在副本里，相对主仓原件的 diff）：

```
2415a2416,2423
>             // 调查用改动：前一个槽开头是头校验和过的码 1（32K 数据单元）单元，这一槽是它的后半，不当单元头读。
>             if slot > 0
>                 && reader
>                     .read(device, (slot - 1) * SLOT_BYTES, 4096)
>                     .is_some_and(|previous| &previous[..4] == b"SFSU" && previous[6] == 1 && checksum_field_holds(&previous, 105, 10))
>             {
>                 continue;
>             }
```

在这个改法下跑最小复现，原样输出（mutant-minimal.log，退出码 0）：

```
test user_content_shaped_like_a_node_header_at_the_second_slot_of_a_data_unit_is_not_a_unit_header ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
```

同一个改法下跑 interior 测试（mutant2.log，退出码 101，是 interior 里那条断言「会红」的测试失败了）：`grep -E "name=final_memory_pool|name=full_scan|test result" mutant2.log | cut -c1-160` 的原样输出：

```
name=final_memory_pool not_holding=[("I-7.9", NotApplicable("根环里没有抬 F 的根：每条根带的 F 都不高于同一实例里它前一条根带的（
name=full_scan_none_candidates i78=Holds
test result: FAILED. 6 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.22s
```

`grep 'name=final_memory_pool' mutant2.log | grep -c '"I-7.8"'` 输出 `0`：改法之下，全持久池的不成立清单里已经没有 I-7.8。

改法第一版读前一个槽时只读了 105 字节。`MemoryPool` 的读不按扇区对齐就交回 None（`crates/singlefs-harness/src/memory_pool.rs:394` 的文档注释），所以跳过的条件恒为假，最小复现照样红（mutant.log）。加打印核过：前一个槽（50180）读 4096 字节时 `magic=SFSU code=1 check105=true`。改成读 4096 字节之后才得到上表的结果。这一步只是改法本身写错了，和定位无关。

改完之后 walk.rs 已从主仓原件拷回，重跑最小复现又回到红（就是上一节贴的输出）。

## 三次推导

- 正推：`memory_pool.rs:644` 把 50181 放进候选 → `walk.rs:2413` 看到 magic 与码 2 → `:2417` CRC32C 过 → `:2419` 诞生代号 1 ≤ 3，实例表没有行 → `:2427` 收下 2^40 → `:6251` 判 19 > 2^40 为假。每一步都有加打印那一行对应：`slot=50181 … counted=true`。
- 反推：如果红是池真的违反，或者来自别的结构，那么只拿掉 50181 这一个候选，或者让扫描跳过数据单元的后半槽，都该仍然红。实测两种都变成成立（`without_second_slots`、第 1 条改法）。
- 校验：换一条不共用 checker 扫描的路子，只从 core 的单元写开头取树 ID，最大是 15；水位用 core 的 `readable_roots` 读出来是 19。池也用两种方式各造一次（录制流施加到空盘、直接取稀疏盘镜像），判定相同。这条校验路子会不会红：对照池与伪造池走的是同一段取数代码，伪造的头不在任何一次单元写的开头，所以这条路子本来就不会看见它，这正是要比的那件事。至于它在别的场景下能不能抓到一个真的越界树 ID，没有另造坏池去验，列进「没做什么」。

## 排除掉的解释

- 「全持久之后池被写坏了」：恢复读回内容逐字相同；别的不变量没有一条判违反（`final_memory_pool` 行）。
- 「是 `MemoryPool` 候选给多了的 harness 问题」：候选改成 None、照 checker 自己的约定整盘扫也红，所以病根在 checker 的扫描读法，不在候选。
- 「头校验和、诞生代号那两道本来就能挡住，只是这次失效了」：翻一位校验和、诞生代号抬到 4，两组都变成成立，说明这两道确实在起作用，这次伪造的头是真的过了这两道。
- 「是树表条目带进来的」：不放头的对照池里树表一样，I-7.8 成立；只拿掉 50181 这个候选也变成成立。

## 顺带看到、没往下查的

- 同一种「每个候选槽的开头都当单元头读」的扫描还有两处：`walk.rs:2506`–`2508`（实例代号载体）和 `walk.rs:4633`–`4635`（码 1 / 码 3 内容单元，I-1.8 用；`:4624` 的注释写着第二个槽上没有 magic）。在数据单元后半槽里伪造码 1 / 码 3 头，或者伪造一个实例代号更高的头，会不会让 I-1.8、I-7.7 在健康池上判红：没有造，没有验。
- 叠加形态 `CrashImage` 的候选比 trait 约定的 None 整盘扫还窄（`memory_pool.rs:747` 只加单元写的第一个槽）。按这次的观测，同一份字节在层 0 这一侧和物化、整盘扫那一侧判定不同；层 0 看不见这个假红，这一点和攻方的说法一致。要不要改、改哪一侧，不归我判。

## 没做什么

- 没修，也没判该怎么改。上面第 1 条的改法只是为了造推翻现象，不是修法的建议：它只看前一个槽，别的单元宽度与伪造码 1 头的情形都没考虑。
- 没跑原现场的 E161 装置与攻方原型（checker 档），只引主 agent 的复跑。
- 没跑重型测试、门禁、层 0、checker 档；只跑了自己新加的两个 harness 档测试目标，都经过内存包装与 4 线程上限。
- 没验第二条校验路子（从单元写开头取树 ID）在真越界的坏池上会不会红。
- 没拿不同的内容长度或放置位置扫一遍，只验了攻方给的那一点（17000 字节、第 16250 字节），外加 +512 字节那个对照。
- 副本里的改动没有带回主工作区。两个测试文件留在 `/tmp/claude-1000/investigate-f1-i78/kept/`，日志留在 `/tmp/claude-1000/investigate-f1-i78/*.log`，都没入库；要不要把测试挪进仓由主 agent 定。
- 副本 `/tmp/claude-1000/investigate-f1-i78/repo/` 与编译目录 `/tmp/claude-1000/investigate-f1-i78/target/` 在交回前删除。
