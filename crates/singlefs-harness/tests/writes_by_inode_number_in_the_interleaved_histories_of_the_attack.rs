//! 攻方腿（m3-close-w5-code-r1）的交错历史原样钉进 harness 档（代码轮判决 `research/prompts/m3-close-w5-code-r1-main-verification.md`
//! 第三节第 2 条「攻方 W4 的两段历史钉进 harness 档用例」）：驱动照抄
//! `research/prompts/m3-close-w5-code-r1-opus-model/attack_histories_of_multi_file_writes_and_unlinks.rs` 的 `interleaved_history`
//! （建 inode、按号写、按号删、回退到更早的一版、可写挂载一份拷贝，抽法与次序逐字不变，同一个种子走出同一段历史），带它自己的小模型：
//! 活着的号与内容、出生代、这条时间线上 inode 1 的死亡 txg（回退时整份换成回退目标那一刻的）。每次发布之后池级 checker 全绿、每一步之后活着的号
//! 只读挂载读回与小模型相同、按号写的结局与这条时间线上的 β 相同（拒 ⇔ 第一个文件在这条时间线上删完了而目标生于那之后）。
//! 种子 200 与 1051 是攻方量到 β 对不上的两段（`out/run-seed200-trace.log`、`out/b3-5.log`）。另数这两段里「这条时间线上的 β 拒、而墓碑树里
//! inode 1 最晚那条死亡 txg 会放过」的写，至少一次：钉的历史真走到父链与最晚那条分岔的那一形。
mod common;

use std::collections::BTreeMap;

use common::{
    crash_state_devices, file_content, parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{DeviceIdentity, FileOffsetInBytes, InodeNumber, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{
    make_filesystem, root_ring_occupancy_after_make_filesystem, INSTANCE_TABLE_SLOT,
    TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_deletion_switches, roll_back_by_a_forward_publish,
    RollbackTarget, ShadowLedger,
};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_new_inodes, publish_sequential_write_of_an_inode,
    publish_unlink_intent, publish_unlink_step, warm_up, FirstFile, PoolWriter, PublishError,
    TransactionOutput, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::{
    DeletionSwitches, UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize,
};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

type Device = RecordingBlockDevice<SparseBlockDevice>;
const INSTANCE: InstanceGeneration = InstanceGeneration(1);
/// 建 inode、按号写、unlink 意图那几次的写入时间：比第一个文件晚。
const WRITE_TIME_SECONDS_AFTER_THE_FIRST_FILE: u64 = FIXED_WRITE_TIME_SECONDS + 600;
/// 一次建 inode 至多发几个（一次发布里的新记录）：多于它分几次发布。
const INODES_CREATED_IN_ONE_PUBLISH_AT_MOST: u64 = 200;
/// 一个号 unlink 到底至多几步（批大小 1、文件至多四个数据单元，远到不了）。
const DELETION_STEPS_OF_ONE_UNLINK_AT_MOST: usize = 400;
/// 攻方量到 β 对不上的两段：外层种子号（驱动按 `种子号 × 0x9e37_79b9` 起随机数）与每段的步数。
const SEED_NUMBER_OF_THE_ATTACK_SEED_TWO_HUNDRED: u64 = 200;
const SEED_NUMBER_OF_THE_ATTACK_SEED_ONE_THOUSAND_AND_FIFTY_ONE: u64 = 1051;
const STEPS_OF_EACH_INTERLEAVED_HISTORY: usize = 40;

/// 两块 4 GiB 的稀疏盘，录制流开内容保留；mkfs、取号、暖机、第一个文件都在这个进程里。
struct PoolOfTheInterleavedHistory {
    devices: Vec<(DeviceIdentity, Device)>,
    output: TransactionOutput,
    allocator: PoolAllocator,
}

impl PoolOfTheInterleavedHistory {
    fn build() -> PoolOfTheInterleavedHistory {
        let parameters = parameters();
        let stream = SharedStream::retaining_contents();
        let mut devices: Vec<(DeviceIdentity, Device)> = (0..2u32)
            .map(|number| {
                (
                    DeviceIdentity(number),
                    RecordingBlockDevice::with_shared_stream(
                        DeviceIdentity(number),
                        SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                        stream.clone(),
                    ),
                )
            })
            .collect();
        let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
        let mut allocator = PoolAllocator::new(vec![
            DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES),
            DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES),
        ]);
        allocator.mark_format_time_units(
            Placement {
                slot: INSTANCE_TABLE_SLOT,
                span: 2,
            },
            Placement {
                slot: TREE_TABLE_GENESIS_SLOT,
                span: 1,
            },
        );
        allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(
            &parameters,
            &genesis,
        ));
        let content = file_content();
        let output = {
            let mut pool = PoolWriter::new(&parameters, &mut devices);
            let instance = acquire_instance(&mut pool).expect("取号");
            let warm = warm_up(&mut pool, &genesis.root, instance).expect("暖机");
            publish_first_file(
                &mut pool,
                &mut allocator,
                warm.roots.last().expect("暖机两代根"),
                FirstFile {
                    content: &content,
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS,
                },
                instance,
                &warm.last_record_bytes,
            )
            .expect("新池新建文件")
        };
        PoolOfTheInterleavedHistory {
            devices,
            output,
            allocator,
        }
    }

    fn image(&self) -> MemoryPool {
        MemoryPool {
            devices: self
                .devices
                .iter()
                .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
                .collect(),
            device_size_in_bytes: IMAGE_BYTES,
        }
    }

    fn create_inodes(&mut self, count: u64) {
        let mut left = count;
        // 迭代上界 ⌈count ÷ 200⌉；跨轮只减 `left`。
        while left > 0 {
            let created_now = left.min(INODES_CREATED_IN_ONE_PUBLISH_AT_MOST);
            let previous = self.output.clone();
            let parameters = parameters();
            let mut writer = PoolWriter::new(&parameters, &mut self.devices);
            self.output = publish_new_inodes(
                &mut writer,
                &mut self.allocator,
                &previous,
                created_now,
                WRITE_TIME_SECONDS_AFTER_THE_FIRST_FILE,
                INSTANCE,
            )
            .expect("建 inode");
            left -= created_now;
        }
    }

    fn try_write(&mut self, inode: u64, content: &[u8]) -> Result<(), PublishError> {
        let previous = self.output.clone();
        let parameters = parameters();
        let mut writer = PoolWriter::new(&parameters, &mut self.devices);
        self.output = publish_sequential_write_of_an_inode(
            &mut writer,
            &mut self.allocator,
            &previous,
            InodeNumber(inode),
            content,
            WRITE_TIME_SECONDS_AFTER_THE_FIRST_FILE,
            INSTANCE,
        )?;
        Ok(())
    }

    /// 意图那次加全部批（批大小 1），每一版交池级 checker；交回意图那次的 txg。
    fn unlink(&mut self, inode: u64, what: &str) -> u64 {
        let parameters = parameters();
        {
            let previous = self.output.clone();
            let mut writer = PoolWriter::new(&parameters, &mut self.devices);
            self.output = publish_unlink_intent(
                &mut writer,
                &mut self.allocator,
                &previous,
                InodeNumber(inode),
                WRITE_TIME_SECONDS_AFTER_THE_FIRST_FILE,
                INSTANCE,
            )
            .unwrap_or_else(|refusal| panic!("{what}：unlink {inode} 的意图：{refusal:?}"));
        }
        let intent_txg = self.output.root.checkpoint_txg.0;
        assert_checker_green(&self.image(), &format!("{what}：unlink {inode} 意图之后"));
        let mut steps = 0;
        // 迭代上界 DELETION_STEPS_OF_ONE_UNLINK_AT_MOST；每一轮队首那个号少一个单元或删掉。
        while self.output.deleted_inodes.contains(&InodeNumber(inode)) {
            steps += 1;
            assert!(
                steps < DELETION_STEPS_OF_ONE_UNLINK_AT_MOST,
                "{what}：unlink {inode} 走不到最后一批"
            );
            let previous = self.output.clone();
            let mut writer = PoolWriter::new(&parameters, &mut self.devices);
            self.output = publish_unlink_step(
                &mut writer,
                &mut self.allocator,
                &previous,
                InodeNumber(inode),
                batches_of_one_data_unit(),
                INSTANCE,
            )
            .unwrap_or_else(|refusal| panic!("{what}：unlink {inode} 第 {steps} 步：{refusal:?}"));
            assert_checker_green(
                &self.image(),
                &format!("{what}：unlink {inode} 第 {steps} 步之后"),
            );
        }
        intent_txg
    }

    fn roll_back_to(&mut self, target: &TransactionOutput) -> bool {
        let parameters = parameters();
        roll_back_by_a_forward_publish(
            &parameters,
            &mut self.devices,
            &mut self.allocator,
            &mut self.output,
            RollbackTarget {
                instance: target.root.instance,
                checkpoint_txg: target.root.checkpoint_txg,
            },
        )
        .is_ok()
    }

    /// 墓碑树里 inode 1 那几条墓碑最晚的死亡 txg（不看谱系：β 改法之前的读法）。
    fn latest_death_txg_of_the_first_file_in_the_tombstone_tree(&self) -> Option<u64> {
        self.output
            .tombstone_containers
            .iter()
            .flat_map(|container| container.contents.records.iter())
            .filter(|tombstone| tombstone.object.0 == FIRST_INODE_NUMBER)
            .map(|tombstone| tombstone.death_txg.0)
            .max()
    }
}

fn batches_of_one_data_unit() -> UnlinkBatchSize {
    UnlinkBatchSize::CappedForTests(UnlinkBatchDataUnitsCappedForTests::new(1).expect("1..=144"))
}

fn content_of(length: usize, seed: u64) -> Vec<u8> {
    let mut state = seed | 1;
    (0..length)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            u8::try_from(state & 0xff).expect("低 8 位")
        })
        .collect()
}

fn content_of_data_units(data_units: usize, seed: u64) -> Vec<u8> {
    content_of(data_units * data_unit_payload_capacity() - 5, seed)
}

fn violations(image: &MemoryPool) -> Vec<(&'static str, String)> {
    check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some((invariant, detail)),
            InvariantVerdict::Holds
            | InvariantVerdict::NotApplicable(_)
            | InvariantVerdict::PartiallyJudged(_) => None,
        })
        .collect()
}

fn assert_checker_green(image: &MemoryPool, what: &str) {
    let found = violations(image);
    assert!(found.is_empty(), "{what}：池级 checker 红 {found:?}");
}

fn read_back(image: &MemoryPool, inode: u64) -> Result<Vec<u8>, String> {
    let mounted = mount_read_only(image).map_err(|failure| format!("只读挂载 {failure:?}"))?;
    let file = mounted
        .mounted
        .open_file(image, InodeNumber(inode))
        .map_err(|failure| format!("打开 {inode}：{failure:?}"))?;
    let size = file.inode_record().size;
    file.read_at(image, FileOffsetInBytes(0), size)
        .map(|read| read.bytes)
        .map_err(|failure| format!("读 {inode}：{failure:?}"))
}

/// 这一份镜像可写挂载一次（续做按批大小 1 走）。
fn mount_a_copy(image: &MemoryPool) -> Result<(), String> {
    let stream = SharedStream::new();
    let mut devices = crash_state_devices(image, &[], &[], &stream);
    let switches = DeletionSwitches {
        batch_size: batches_of_one_data_unit(),
        ..DeletionSwitches::PRODUCT
    };
    mount_writable_with_test_only_switches_and_deletion_switches(
        &parameters(),
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        switches,
    )
    .map(|_| ())
    .map_err(|error| format!("可写挂载 {error:?}"))
}

/// 攻方驱动的小模型：活着的号（内容、出生代）与这条时间线上 inode 1 的死亡 txg。
#[derive(Clone, Debug)]
struct LineModel {
    alive: BTreeMap<u64, (Vec<u8>, u64)>,
    death_of_the_first_file_on_this_line: Option<u64>,
}

/// 一段交错历史跑完的结局：β 对不上的几处，与「这条时间线上的 β 拒、墓碑树里最晚那条会放过」的写有几次。
struct InterleavedHistoryOutcome {
    beta_disagreements: Vec<String>,
    refusals_where_the_latest_tombstone_would_let_the_write_through: u64,
}

/// 照抄攻方的 `interleaved_history`（抽法与次序逐字不变）：每一步按抽到的数建 inode、按号写、删（批大小 1）、回退到存下的某一版、或可写挂载一份拷贝。
fn interleaved_history(seed: u64, steps: usize) -> InterleavedHistoryOutcome {
    let mut pool = PoolOfTheInterleavedHistory::build();
    let mut model = LineModel {
        alive: BTreeMap::from([(FIRST_INODE_NUMBER, (file_content(), 0))]),
        death_of_the_first_file_on_this_line: None,
    };
    let mut saved: Vec<(TransactionOutput, LineModel)> = vec![(pool.output.clone(), model.clone())];
    let mut state = seed | 1;
    let mut draw = |bound: u64| {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state % bound
    };
    let mut outcome = InterleavedHistoryOutcome {
        beta_disagreements: Vec::new(),
        refusals_where_the_latest_tombstone_would_let_the_write_through: 0,
    };
    // 迭代上界是 `steps`；跨轮携带的是池、小模型与存下的版本。
    for step in 0..steps {
        let choice = if model.alive.is_empty() {
            draw(15)
        } else {
            draw(100)
        };
        let what = format!("种子 {seed} 第 {step} 步");
        if choice < 15 {
            let count = 1 + draw(3) * draw(80);
            pool.create_inodes(count);
            let txg = pool.output.root.checkpoint_txg.0;
            for record in pool
                .output
                .inode_leaf_containers
                .iter()
                .flat_map(|container| container.contents.records.iter())
            {
                if record.inode != FIRST_INODE_NUMBER
                    && !model.alive.contains_key(&record.inode)
                    && record.object_birth.0 + 2 >= txg
                {
                    model
                        .alive
                        .insert(record.inode, (Vec::new(), record.object_birth.0));
                }
            }
        } else if choice < 50 {
            let candidates: Vec<u64> = model.alive.keys().copied().collect();
            let inode =
                candidates[usize::try_from(draw(u64::try_from(candidates.len()).expect("个数")))
                    .expect("下标")];
            let content = content_of_data_units(
                1 + usize::try_from(draw(4)).expect("小"),
                seed ^ u64::try_from(step).expect("步"),
            );
            let born = model.alive[&inode].1;
            let beta_refuses = inode != FIRST_INODE_NUMBER
                && !model.alive.contains_key(&FIRST_INODE_NUMBER)
                && model
                    .death_of_the_first_file_on_this_line
                    .is_some_and(|death| born > death);
            let latest_tombstone_refuses = inode != FIRST_INODE_NUMBER
                && !model.alive.contains_key(&FIRST_INODE_NUMBER)
                && pool
                    .latest_death_txg_of_the_first_file_in_the_tombstone_tree()
                    .is_some_and(|death| born > death);
            if beta_refuses && !latest_tombstone_refuses {
                outcome.refusals_where_the_latest_tombstone_would_let_the_write_through += 1;
            }
            match pool.try_write(inode, &content) {
                Ok(()) => {
                    if beta_refuses {
                        outcome.beta_disagreements.push(format!(
                            "{what}：写 {inode}（生于 {born}，这条时间线上 1 死于 {:?}）做成了",
                            model.death_of_the_first_file_on_this_line
                        ));
                    }
                    model.alive.get_mut(&inode).expect("活着").0 = content;
                    assert_checker_green(&pool.image(), &format!("{what} 写 {inode} 之后"));
                }
                Err(refusal) => {
                    if !beta_refuses {
                        outcome.beta_disagreements.push(format!(
                            "{what}：写 {inode} 被拒 {refusal:?}，这条时间线上 β 不拒"
                        ));
                    }
                }
            }
        } else if choice < 70 {
            let candidates: Vec<u64> = model.alive.keys().copied().collect();
            if candidates.len() <= 1 && draw(2) == 0 {
                continue;
            }
            let inode =
                candidates[usize::try_from(draw(u64::try_from(candidates.len()).expect("个数")))
                    .expect("下标")];
            let intent_txg = pool.unlink(inode, &what);
            if inode == FIRST_INODE_NUMBER {
                model.death_of_the_first_file_on_this_line = Some(intent_txg);
            }
            model.alive.remove(&inode);
        } else if choice < 85 {
            let index =
                usize::try_from(draw(u64::try_from(saved.len()).expect("个数"))).expect("下标");
            let (target, target_model) = saved[index].clone();
            if pool.roll_back_to(&target) {
                model = target_model;
                assert_checker_green(&pool.image(), &format!("{what} 回退之后"));
            }
        } else {
            mount_a_copy(&pool.image())
                .unwrap_or_else(|failure| panic!("{what}：可写挂载一份拷贝：{failure}"));
        }
        let image = pool.image();
        for (inode, (content, _)) in &model.alive {
            assert_eq!(
                &read_back(&image, *inode)
                    .unwrap_or_else(|failure| panic!("{what}：读 {inode}：{failure}")),
                content,
                "{what}：{inode} 读回"
            );
        }
        saved.push((pool.output.clone(), model.clone()));
    }
    outcome
}

/// 一个外层种子号那一段（`interleaved_history(种子号 × 0x9e37_79b9, 40)`）：β 一处都不对不上，且走到过「父链拒、最晚那条会放过」的写。
fn assert_the_interleaved_history_of_the_attack_agrees_with_beta_on_its_line(seed_number: u64) {
    let outcome = interleaved_history(seed_number * 0x9e37_79b9, STEPS_OF_EACH_INTERLEAVED_HISTORY);
    assert!(
        outcome.beta_disagreements.is_empty(),
        "种子 {seed_number}：β 对不上 {} 次：{:?}",
        outcome.beta_disagreements.len(),
        outcome.beta_disagreements
    );
    assert!(
        outcome.refusals_where_the_latest_tombstone_would_let_the_write_through >= 1,
        "种子 {seed_number}：这段历史该走到「这条时间线上的 β 拒、墓碑树里最晚那条会放过」的写（攻方量到的那一形），一次都没有"
    );
}

/// 攻方种子 200（`research/prompts/m3-close-w5-code-r1-opus-model/out/run-seed200-trace.log`：第 38 步写 269，生于 14，这条时间线上 1 死于 6，
/// 墓碑树里最晚那条死于 20）。判别力：实现的 β 改回取最大（`crates/mutations.tsv` 以「K-U5 β 父链：」起头那一行）时那一写做成，这一条红。
#[test]
#[ignore = "harness 慢档用例：攻方种子 200 那一段 40 步交错历史，每次发布与每一步都跑池级 checker、只读挂载读回，debug 下单条量到 87 秒"]
fn the_interleaved_history_of_the_attack_seed_two_hundred_agrees_with_beta_on_its_parent_chain() {
    assert_the_interleaved_history_of_the_attack_agrees_with_beta_on_its_line(
        SEED_NUMBER_OF_THE_ATTACK_SEED_TWO_HUNDRED,
    );
}

/// 攻方种子 1051（`research/prompts/m3-close-w5-code-r1-opus-model/out/b3-5.log`：第 34、36、39 步写生于 13 的号，这条时间线上 1 死于 10）。
/// 判别力同上一条。
#[test]
fn the_interleaved_history_of_the_attack_seed_one_thousand_and_fifty_one_agrees_with_beta_on_its_parent_chain(
) {
    assert_the_interleaved_history_of_the_attack_agrees_with_beta_on_its_line(
        SEED_NUMBER_OF_THE_ATTACK_SEED_ONE_THOUSAND_AND_FIFTY_ONE,
    );
}
