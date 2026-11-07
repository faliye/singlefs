//! 验证包模块：无（只用验证包的稀疏内存盘，不碰崩溃引擎模块）
//! 根环写者侧的 txg 与不倒挂（C650（择根与重放没有合判） 的写者侧、不倒挂两样留档用例，同一个场景里断言；判据在判决
//! `research/prompts/m3-close-b19-e165-r3-main-verification.md` 第二节「C650 剩下那一句的判据（定）」那一格）：
//! 择根的两条改坏（比较不看实例代号、txg 平局取实例代号低的）要同一 txg 上两种实例代号同时进择根才与今天择得不同，承诺内除了读者落点核
//! （`root_ring_reader_admits_each_root_only_at_its_formula_slot_for_every_slots_per_region.rs`），还靠写者「txg 0 的根只有 mkfs 写、实例代号 0」；
//! 实例代号为主的那条改坏要环里倒挂（实例代号更高的根 txg 更低）才择得不同，承诺内靠「新实例第一次发布的 txg 大于环里一切可见的根」
//! （D23（journal 的角色与格式） 已定项 14 第 3 条，`mount` 的 `first_txg_of_new_instance`）。
//!
//! 场景：两块内存盘上 mkfs（S 取显式建池入口常用的 8），之后同一个池上三轮，每轮可写挂载一次（新实例写行、暖机），再发一版文件
//! （接在没有文件的那一版上发第一个文件，接在有文件的那一版上覆盖写）。mkfs 之后、每次挂载之后、每次发布之后都用产品读槽
//! （`recovery::read_root_ring_slot`）读一遍根环。不带故障：E165（恢复重放有界穷举与变异杀伤界） 实验页点名的掉盘、设备失而复得、
//! 管理员回退这几类状态不在这条用例里。

use std::collections::BTreeSet;

use singlefs_core::address::{DeviceIdentity, InstanceGeneration};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{
    make_filesystem, MakeFilesystemParameters, FIRST_VERSION_REGION_DEVICES, GENESIS_CHECKPOINT_TXG,
};
use singlefs_core::mount::{mount_writable, Mounted};
use singlefs_core::recovery::{
    choose_system_configuration, every_root_ring_slot, read_root_ring_slot, BadRootRingSlotReading,
    RootRingSlotReading,
};
use singlefs_core::root_ring::{RootRingSlot, RootRingSlotsPerRegion};
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{
    publish_first_file, publish_overwrite, FirstFile, PoolVersion, PoolWriter,
};
use singlefs_format::{JOURNAL_RING_DEFAULT_BYTES, ROOT_RING_REGIONS};
use singlefs_verification::memory_pool::SparseBlockDevice;

/// 每块内存盘的字节数：稀疏盘，只存写过的扇区。
const DEVICE_SIZE_IN_BYTES: u64 = 4 << 30;
/// 物理块宽，也是 io_min。
const PHYSICAL_BLOCK_SIZE_IN_BYTES: u32 = 512;
const FIXED_STRUCTURE_SLOT_SPACING_IN_BYTES: u32 = 4096;
const FILESYSTEM_IDENTIFIER: [u8; 16] = *b"writer-ring-txgs";
/// 同一个池上可写挂载几次（每次之后发一版文件）：三个实例，后一个要越过前一个写下的根。
const WRITABLE_MOUNTS: u32 = 3;
/// 每次发布的文件内容字节数：装得进一个数据单元。
const FILE_CONTENT_SIZE_IN_BYTES: usize = 3000;
/// 发布的写入时间从这一秒起，每次挂载之后的那一版加上已挂载的次数（时间是参数，不取系统时钟）。
const WRITE_TIME_SECONDS_OF_THE_PUBLISHES: u64 = 1_788_000_000;

type PoolDevices = Vec<(DeviceIdentity, SparseBlockDevice)>;

fn parameters_with_eight_slots_per_region() -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: FILESYSTEM_IDENTIFIER,
        region_devices: FIRST_VERSION_REGION_DEVICES,
        geometry: SystemImmutableSizes {
            physical_block_size: PHYSICAL_BLOCK_SIZE_IN_BYTES,
            minimum_input_output_bytes: PHYSICAL_BLOCK_SIZE_IN_BYTES,
            fixed_structure_slot_spacing: FIXED_STRUCTURE_SLOT_SPACING_IN_BYTES,
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
        },
    }
}

/// 两块内存盘上 mkfs 一次，交回那两块盘（之后的挂载与发布都写它们）。
fn devices_after_make_filesystem(parameters: &MakeFilesystemParameters) -> PoolDevices {
    let mut devices: PoolDevices = [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| {
            (
                identity,
                SparseBlockDevice::new(
                    DEVICE_SIZE_IN_BYTES,
                    PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_SIZE_IN_BYTES),
                ),
            )
        })
        .collect();
    make_filesystem(parameters, &mut devices).expect("两块内存盘上 mkfs");
    devices
}

/// 环里一条产品读槽收下的根：住在哪一槽、txg、实例代号。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct VisibleRoot {
    ring_slot: RootRingSlot,
    checkpoint_txg: u64,
    instance: u32,
}

/// 照盘上择到的系统配置，按读根环次序逐槽调产品读槽，交回收下的根。
fn roots_visible_in_the_ring(devices: &PoolDevices) -> Vec<VisibleRoot> {
    let system_configuration = choose_system_configuration(devices).expect("系统配置择得出");
    let immutable = &system_configuration.immutable;
    every_root_ring_slot(&immutable.sizes)
        .into_iter()
        .filter_map(|ring_slot| {
            match read_root_ring_slot(
                devices,
                &immutable.region_devices,
                &immutable.sizes,
                &immutable.filesystem_identifier,
                ring_slot,
            ) {
                RootRingSlotReading::SelfVerified(root) => Some(VisibleRoot {
                    ring_slot,
                    checkpoint_txg: root.checkpoint_txg.0,
                    instance: root.instance.0,
                }),
                RootRingSlotReading::Bad(BadRootRingSlotReading::Unreadable) => {
                    panic!("内存盘上的根槽读不出：{ring_slot:?}")
                }
                RootRingSlotReading::Bad(BadRootRingSlotReading::NotSelfVerified) => None,
            }
        })
        .collect()
}

/// 根环打成一行：`r<区域>s<槽>:(<txg>,<实例代号>)`，报红时照打。
fn ring_text(roots: &[VisibleRoot]) -> String {
    roots
        .iter()
        .map(|root| {
            format!(
                "r{}s{}:({},{})",
                root.ring_slot.region, root.ring_slot.slot, root.checkpoint_txg, root.instance
            )
        })
        .collect::<Vec<String>>()
        .join(" ")
}

fn highest_txg_of(roots: &[VisibleRoot]) -> u64 {
    roots
        .iter()
        .map(|root| root.checkpoint_txg)
        .max()
        .expect("环里至少有 mkfs 种下的根")
}

fn highest_instance_of(roots: &[VisibleRoot]) -> u32 {
    roots
        .iter()
        .map(|root| root.instance)
        .max()
        .expect("环里至少有 mkfs 种下的根")
}

/// 一刻的根环上判写者侧与不倒挂：txg 0 的根只有实例代号 0（mkfs 写的），实例代号 0 的根只有 txg 0，别的实例写出的根 txg 都大于 0；
/// 没有一对根实例代号更高而 txg 更低。`moment` 写这是哪一刻，报红时照打。
fn assert_the_writer_side_and_no_inversion_hold(roots: &[VisibleRoot], moment: &str) {
    let ring = ring_text(roots);
    let instances_at_txg_zero: BTreeSet<u32> = roots
        .iter()
        .filter(|root| root.checkpoint_txg == GENESIS_CHECKPOINT_TXG.0)
        .map(|root| root.instance)
        .collect();
    assert!(
        instances_at_txg_zero.iter().all(|instance| *instance == 0),
        "写者侧：txg 0 的根只有 mkfs 写、实例代号 0；{moment}环里 txg 0 上的实例代号是 {instances_at_txg_zero:?}：[{ring}]"
    );
    assert!(
        roots
            .iter()
            .filter(|root| root.instance == 0)
            .all(|root| root.checkpoint_txg == GENESIS_CHECKPOINT_TXG.0),
        "写者侧：实例代号 0 的根只有 mkfs 写的第 0 代根；{moment}：[{ring}]"
    );
    assert!(
        roots
            .iter()
            .filter(|root| root.instance != 0)
            .all(|root| root.checkpoint_txg > GENESIS_CHECKPOINT_TXG.0),
        "写者侧：不是 mkfs 写出的根 txg 都大于 0；{moment}：[{ring}]"
    );
    let inversions: Vec<(VisibleRoot, VisibleRoot)> = roots
        .iter()
        .flat_map(|higher_instance_root| {
            roots
                .iter()
                .filter(move |lower_instance_root| {
                    higher_instance_root.instance > lower_instance_root.instance
                        && higher_instance_root.checkpoint_txg < lower_instance_root.checkpoint_txg
                })
                .map(move |lower_instance_root| (*higher_instance_root, *lower_instance_root))
        })
        .collect();
    assert!(
        inversions.is_empty(),
        "不倒挂：环里不许有实例代号更高而 txg 更低的根；{moment}倒挂的 {} 对：{inversions:?}：[{ring}]",
        inversions.len()
    );
}

/// 场景：mkfs 之后同一个池上可写挂载三次，每次挂载之后发一版文件；mkfs 之后、每次挂载之后、每次发布之后读根环。
/// 预期：每一刻 txg 0 的根只有 mkfs 写的实例代号 0、别的实例写出的根 txg 都大于 0、没有倒挂；每次挂载的新实例代号高于挂载之前环里一切可见的根，
/// 它写出的每条根 txg 都大于挂载之前环里一切可见的根；每次发布的根 txg 大于发布之前环里一切可见的根。
/// 判别力：`crates/mutations.tsv`「C650 写者侧留档：挂载一侧接在一版后面的 txg 加 0」让这一条红（挂载一侧的下一个 txg 停在上一版上）；
/// 「C650 不倒挂留档：新实例第一次发布不看环里的根与记录」让这一条红（后面的实例从 txg 1 起，写出比前一个实例的根 txg 更低的根）。
/// 不并行：三轮挂载与发布落在同一个池上，后一轮读的就是前一轮写下的根环，次序本身是被测对象；整条只有三次挂载、三次发布。
// harness-test-granularity:one-scenario 同一个池上按次序挂载、发布三轮：后一轮的新实例要越过前一轮写下的根，拆开就没有这一层
#[test]
fn writable_mounts_and_publishes_on_a_fresh_pool_leave_txg_zero_to_make_filesystem_and_cross_every_visible_root_without_inversion(
) {
    let parameters = parameters_with_eight_slots_per_region();
    let mut devices = devices_after_make_filesystem(&parameters);
    let roots_after_make_filesystem = roots_visible_in_the_ring(&devices);
    assert_eq!(
        roots_after_make_filesystem
            .iter()
            .map(|root| (root.ring_slot, root.checkpoint_txg, root.instance))
            .collect::<Vec<(RootRingSlot, u64, u32)>>(),
        (0..ROOT_RING_REGIONS)
            .map(|region| (
                RootRingSlot { region, slot: 0 },
                GENESIS_CHECKPOINT_TXG.0,
                0
            ))
            .collect::<Vec<(RootRingSlot, u64, u32)>>(),
        "前提：mkfs 之后环里只有三个区域各自槽 0 的第 0 代根"
    );
    assert_the_writer_side_and_no_inversion_hold(&roots_after_make_filesystem, "mkfs 之后");
    let mut roots_before_the_mount = roots_after_make_filesystem;
    // 迭代上界：WRITABLE_MOUNTS 次；跨轮携带的是同一个池的两块盘与上一刻读到的根环，没有提前出口。
    for mounts_done in 1..=WRITABLE_MOUNTS {
        let Mounted {
            output,
            mut allocator,
            current,
        } = mount_writable(&parameters, &mut devices).unwrap_or_else(|error| {
            panic!("挂载 {mounts_done} 次：没故障的池可写挂载要做成：{error:?}")
        });
        let new_instance: InstanceGeneration = output.instance;
        let roots_after_the_mount = roots_visible_in_the_ring(&devices);
        let ring_after_the_mount = ring_text(&roots_after_the_mount);
        assert_the_writer_side_and_no_inversion_hold(
            &roots_after_the_mount,
            &format!("挂载 {mounts_done} 次之后"),
        );
        let highest_txg_before_the_mount = highest_txg_of(&roots_before_the_mount);
        assert!(
            new_instance.0 > highest_instance_of(&roots_before_the_mount),
            "新实例的实例代号 {} 高于挂载之前环里一切可见的根：[{}]",
            new_instance.0,
            ring_text(&roots_before_the_mount)
        );
        let roots_of_the_new_instance: Vec<VisibleRoot> = roots_after_the_mount
            .iter()
            .copied()
            .filter(|root| root.instance == new_instance.0)
            .collect();
        assert!(
            !roots_of_the_new_instance.is_empty(),
            "可写挂载写行、暖机都发布：新实例 {} 在环里至少一条根：[{ring_after_the_mount}]",
            new_instance.0
        );
        assert!(
            roots_of_the_new_instance
                .iter()
                .all(|root| root.checkpoint_txg > highest_txg_before_the_mount),
            "新实例第一次发布的 txg 大于环里一切可见的根：挂载之前最高的 txg 是 {highest_txg_before_the_mount}，挂载之后：[{ring_after_the_mount}]"
        );
        let content =
            [u8::try_from(mounts_done).expect("挂载次数装得进一字节"); FILE_CONTENT_SIZE_IN_BYTES];
        let file = FirstFile {
            content: &content,
            write_time_seconds: WRITE_TIME_SECONDS_OF_THE_PUBLISHES + u64::from(mounts_done),
        };
        let published = {
            let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
            let publish_outcome = match &current {
                PoolVersion::WithoutFile(version) => publish_first_file(
                    &mut writer,
                    &mut allocator,
                    &version.root,
                    file,
                    new_instance,
                    &version.record_bytes,
                ),
                PoolVersion::WithFile(version) => {
                    publish_overwrite(&mut writer, &mut allocator, version, file, new_instance)
                }
                PoolVersion::WithTreesAndNoFile(version) => panic!(
                    "这段历史只写第一个文件，挂载接上的一版不会是有树而文件集合为空的那一种：txg {}",
                    version.root.checkpoint_txg.0
                ),
            };
            publish_outcome
                .unwrap_or_else(|error| panic!("挂载 {mounts_done} 次之后发一版文件：{error:?}"))
        };
        let published_txg = published.root.checkpoint_txg.0;
        let roots_after_the_publish = roots_visible_in_the_ring(&devices);
        assert_the_writer_side_and_no_inversion_hold(
            &roots_after_the_publish,
            &format!("挂载 {mounts_done} 次、发一版文件之后"),
        );
        assert!(
            published_txg > highest_txg_of(&roots_after_the_mount),
            "发布的根 txg {published_txg} 大于发布之前环里一切可见的根：[{ring_after_the_mount}]"
        );
        assert!(
            roots_after_the_publish
                .iter()
                .any(|root| root.checkpoint_txg == published_txg && root.instance == new_instance.0),
            "发布的根（txg {published_txg}、实例代号 {}）在环里读得出：[{}]",
            new_instance.0,
            ring_text(&roots_after_the_publish)
        );
        println!(
            "ROOT_RING_WRITER_TXGS mounts_done={mounts_done} instance={} after_the_mount=[{ring_after_the_mount}] published_txg={published_txg} after_the_publish=[{}]",
            new_instance.0,
            ring_text(&roots_after_the_publish)
        );
        roots_before_the_mount = roots_after_the_publish;
    }
}
