from edit_lib import edit

UA = "crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs"
edit(UA, """//! 盘：两块内存稀疏盘（`SparseBlockDevice`），改盘上字节之后重封每一道校验和；读故障用这个文件自己的包装盘按「这一段第几次读」坏。""", """//! 代码三方第二轮（`research/prompts/m2-closeout-code-r2-main-verification.md`，规格 `/tmp/claude-1000/impl-r2-fixes-b/spec.md`）改了两样：
//! - 读根环照 D16（发布语义） 已定项 1「根槽这一次读坏」那一行全句（用户 2026-09-26 定）：知道住着根的槽「读不出**或自证不过**」都重读一次、
//!   仍坏就拒（第四节末尾两条；包装盘另加「读得出、字节被改」那一种坏法）。管理员回退算水位读根环那一遍仍是 `readable_roots`
//!   （不重读），判候选那两遍读得出、这一遍瞬时读坏时水位取内存里的现行那一版（`the_rollback_takes_the_inode_number_watermark_…`）。
//! - 五、core 读系统配置的 journal 环起点（偏移 325）与根环起点（偏移 371），与第一版常量不等整池拒（Y4-a，用户 2026-09-27 定）。
//!
//! 盘：两块内存稀疏盘（`SparseBlockDevice`），改盘上字节之后重封每一道校验和；读故障用这个文件自己的包装盘按「这一段第几次读」坏。""")
edit(UA, """    readable_roots_rereading_ring_slots_known_to_hold_a_root_once, readable_roots_with_ring_slots,
    recover, verified_system_configuration_slots,""", """    readable_roots_rereading_ring_slots_known_to_hold_a_root_once, readable_roots_with_ring_slots,
    recover, verified_system_configuration_slots, BadRootRingSlotReading,""")
edit(UA, """use singlefs_core::mount::{""", """use singlefs_core::mounted_read::{mount_read_only, MountReadOnlyFailure};
use singlefs_core::mount::{""")
edit(UA, """                if **still_unreadable
                    == StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot {
                        ring_slot: newest_slot,
                    }
        ),""", """                if **still_unreadable
                    == StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot {
                        ring_slot: newest_slot,
                        first_reading: BadRootRingSlotReading::Unreadable,
                        reread: BadRootRingSlotReading::Unreadable,
                    }
        ),""")
edit(UA, """        Err(RootRingSlotKnownToHoldARootStillBadAfterOneReread {
            ring_slot: newest_slot,
        })
    );""", """        Err(RootRingSlotKnownToHoldARootStillBadAfterOneReread {
            ring_slot: newest_slot,
            first_reading: BadRootRingSlotReading::Unreadable,
            reread: BadRootRingSlotReading::Unreadable,
        })
    );""")
# 包装盘：加「读得出、字节被改」那一种坏法
edit(UA, """/// 这个文件自己的读故障：几段落点，每一段按它自己被读的次数（从 1 数，一次读碰到这一段就算一次）点名哪几次读坏
/// （块设备报 `InputOutput`）。两块盘共用一份。
struct ChosenReadFaults {
    ranges: Vec<ChosenReadFaultsOfARange>,
}

struct ChosenReadFaultsOfARange {
    device: DeviceIdentity,
    offset_in_bytes: u64,
    length_in_bytes: u64,
    failing_read_numbers: BTreeSet<u64>,
    reads_so_far: u64,
}

impl ChosenReadFaults {
    /// 记下这一次读碰到的每一段被读了一次；这一次读里有一段点名这一次坏就交回 true。
    fn note_a_read_and_tell_whether_it_fails(
        &mut self,
        device: DeviceIdentity,
        offset_in_bytes: u64,
        length_in_bytes: u64,
    ) -> bool {
        let mut fails = false;
        for range in &mut self.ranges {
            let overlaps = range.device == device
                && offset_in_bytes < range.offset_in_bytes + range.length_in_bytes
                && range.offset_in_bytes < offset_in_bytes + length_in_bytes;
            if !overlaps {
                continue;
            }
            range.reads_so_far += 1;
            fails |= range.failing_read_numbers.contains(&range.reads_so_far);
        }
        fails
    }
}""", """/// 这个文件自己的读故障：几段落点，每一段按它自己被读的次数（从 1 数，一次读碰到这一段就算一次）点名哪几次读坏。
/// 两块盘共用一份。
struct ChosenReadFaults {
    ranges: Vec<ChosenReadFaultsOfARange>,
}

/// 点名的那几次读怎么坏。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ChosenReadFault {
    /// 块设备报 `InputOutput`（读不出）。
    Fails,
    /// 读得出，这一段落在这次读里的第一个字节取反（根槽里是 magic 那一字节：自证不过）。
    ReturnsTheFirstByteOfTheRangeFlipped,
}

struct ChosenReadFaultsOfARange {
    device: DeviceIdentity,
    offset_in_bytes: u64,
    length_in_bytes: u64,
    failing_read_numbers: BTreeSet<u64>,
    fault: ChosenReadFault,
    reads_so_far: u64,
}

/// 这一次读里点名要坏的那一段：它怎么坏，和它在这次读里从第几个字节起。
struct ChosenReadGoingBad {
    fault: ChosenReadFault,
    first_byte_of_the_range_in_the_read: usize,
}

impl ChosenReadFaults {
    /// 记下这一次读碰到的每一段被读了一次；这一次读里有一段点名这一次坏，就交回第一段点名的那一个。
    fn note_a_read_and_tell_how_it_goes_bad(
        &mut self,
        device: DeviceIdentity,
        offset_in_bytes: u64,
        length_in_bytes: u64,
    ) -> Option<ChosenReadGoingBad> {
        let mut going_bad = None;
        for range in &mut self.ranges {
            let overlaps = range.device == device
                && offset_in_bytes < range.offset_in_bytes + range.length_in_bytes
                && range.offset_in_bytes < offset_in_bytes + length_in_bytes;
            if !overlaps {
                continue;
            }
            range.reads_so_far += 1;
            if going_bad.is_none() && range.failing_read_numbers.contains(&range.reads_so_far) {
                going_bad = Some(ChosenReadGoingBad {
                    fault: range.fault,
                    first_byte_of_the_range_in_the_read: usize::try_from(
                        range.offset_in_bytes.saturating_sub(offset_in_bytes),
                    )
                    .expect("一次读里的偏移装得进 usize"),
                });
            }
        }
        going_bad
    }
}""")
edit(UA, """        let length_in_bytes = u64::try_from(buffer.len()).expect("一次读的长度装得进 u64");
        if self
            .faults
            .borrow_mut()
            .note_a_read_and_tell_whether_it_fails(self.identity, offset.0, length_in_bytes)
        {
            return Err(BlockDeviceError::InputOutput(std::io::Error::other(
                "用例点名的这一次读坏",
            )));
        }
        self.inner.read_at(offset, buffer)""", """        let length_in_bytes = u64::try_from(buffer.len()).expect("一次读的长度装得进 u64");
        let going_bad = self.faults.borrow_mut().note_a_read_and_tell_how_it_goes_bad(
            self.identity,
            offset.0,
            length_in_bytes,
        );
        match going_bad {
            None => self.inner.read_at(offset, buffer),
            Some(ChosenReadGoingBad {
                fault: ChosenReadFault::Fails,
                ..
            }) => Err(BlockDeviceError::InputOutput(std::io::Error::other(
                "用例点名的这一次读坏",
            ))),
            Some(ChosenReadGoingBad {
                fault: ChosenReadFault::ReturnsTheFirstByteOfTheRangeFlipped,
                first_byte_of_the_range_in_the_read,
            }) => {
                self.inner.read_at(offset, buffer)?;
                buffer[first_byte_of_the_range_in_the_read] ^= 0xff;
                Ok(())
            }
        }""")
edit(UA, """/// 一段读故障：哪块盘、从哪个字节起、多长，这一段第几次读坏。
fn chosen_read_faults_of(
    device: DeviceIdentity,
    offset_in_bytes: u64,
    length_in_bytes: u64,
    failing_read_numbers: &[u64],
) -> ChosenReadFaultsOfARange {
    ChosenReadFaultsOfARange {
        device,
        offset_in_bytes,
        length_in_bytes,
        failing_read_numbers: failing_read_numbers.iter().copied().collect(),
        reads_so_far: 0,
    }
}""", """/// 一段读故障：哪块盘、从哪个字节起、多长，这一段第几次读坏（读不出）。
fn chosen_read_faults_of(
    device: DeviceIdentity,
    offset_in_bytes: u64,
    length_in_bytes: u64,
    failing_read_numbers: &[u64],
) -> ChosenReadFaultsOfARange {
    ChosenReadFaultsOfARange {
        device,
        offset_in_bytes,
        length_in_bytes,
        failing_read_numbers: failing_read_numbers.iter().copied().collect(),
        fault: ChosenReadFault::Fails,
        reads_so_far: 0,
    }
}

/// 同 [`chosen_read_faults_of`]，点名的那几次读得出、这一段第一个字节取反（自证不过）。
fn chosen_corrupted_reads_of(
    device: DeviceIdentity,
    offset_in_bytes: u64,
    length_in_bytes: u64,
    corrupted_read_numbers: &[u64],
) -> ChosenReadFaultsOfARange {
    ChosenReadFaultsOfARange {
        device,
        offset_in_bytes,
        length_in_bytes,
        failing_read_numbers: corrupted_read_numbers.iter().copied().collect(),
        fault: ChosenReadFault::ReturnsTheFirstByteOfTheRangeFlipped,
        reads_so_far: 0,
    }
}""")
edit(UA, """/// 一池内存盘的一份拷贝（盘宽与镜像都照抄）：读故障包在拷贝外面，原池不动。""", open(__import__("os").path.join(__import__("os").path.dirname(__import__("os").path.abspath(__file__)), "10_unit_area_tests_new.rs.txt"), encoding="utf-8").read() + """/// 一池内存盘的一份拷贝（盘宽与镜像都照抄）：读故障包在拷贝外面，原池不动。""")
print("10_unit_area_tests done")
edit(UA, """    acquire_instance, publish_first_file, publish_overwrite, warm_up, FirstFile, PoolWriter,
    TransactionOutput,""", """    acquire_instance, publish_first_file, publish_new_inodes, publish_overwrite, warm_up, FirstFile,
    PoolWriter, TransactionOutput,""")
edit(UA, """    JOURNAL_SAFETY_FACTOR, SLOT_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES, UNIT_AREA_START_SLOT,""", """    JOURNAL_SAFETY_FACTOR, ROOT_RING_BASE_SLOT, SLOT_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES,
    UNIT_AREA_START_SLOT,""")
print("10_unit_area_tests imports done")
