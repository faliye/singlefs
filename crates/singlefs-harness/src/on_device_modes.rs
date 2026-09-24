//! 虚机档 `first_transaction_on_device` 的五个模式，以及第一个事务之后那两次发布（发布 B、可写挂载之后的发布 C）的写路。
//!
//! 虚机里的真设备与宿主上的 `first_transaction_device_log_check` 都跑这一份：模式名、两版的内容与写入时刻、每次发布接在哪一版上，
//! 两边不各抄一份，宿主重跑的才是同参数同字节的那条写路（里程碑「第二个事务」增补 2 收口表第 30 行：宿主那一侧要接着重跑发布 B、
//! 可写挂载与发布 C，把程序录制流与设备侧日志逐项比）。设备侧日志与这里不共享一行代码。

use singlefs_core::address::{DeviceIdentity, InstanceGeneration};
use singlefs_core::allocator::{DeviceFreeMap, PoolAllocator};
use singlefs_core::block_device::BlockDevice;
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::transaction::{
    publish_overwrite, FirstFile, PoolWriter, PublishError, TransactionOutput,
};
use singlefs_core::write_accounting::WritesByStructureKind;

use crate::scenario::{first_file_content, FIXED_WRITE_TIME_SECONDS};

/// 虚机档的五个模式（命令行最后一个参数）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OnDeviceRunMode {
    Direct,
    PageCache,
    SkipFirstTransactionBarrier,
    SecondTransaction,
    SecondInstance,
}

/// 第一个事务之后，这个模式在同一对盘上还跑哪几步写。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PublishesAfterTheFirstTransaction {
    Nothing,
    /// 同一个进程里再覆盖写一次（发布 B，里程碑「第二个事务」步 1）。
    SecondVersion,
    /// 发布 B 之后丢掉写的那一套句柄、同一对盘冷重开，走可写挂载（恢复、取号、写行、暖机，步 3），再发布 C。
    SecondVersionThenSecondInstance,
}

impl OnDeviceRunMode {
    pub const ALL: [OnDeviceRunMode; 5] = [
        OnDeviceRunMode::Direct,
        OnDeviceRunMode::PageCache,
        OnDeviceRunMode::SkipFirstTransactionBarrier,
        OnDeviceRunMode::SecondTransaction,
        OnDeviceRunMode::SecondInstance,
    ];

    /// 命令行上的写法（跑批脚本与门禁 55 号按它送参数）。
    #[must_use]
    pub const fn argument(self) -> &'static str {
        match self {
            OnDeviceRunMode::Direct => "direct",
            OnDeviceRunMode::PageCache => "page-cache",
            OnDeviceRunMode::SkipFirstTransactionBarrier => "skip-first-transaction-barrier",
            OnDeviceRunMode::SecondTransaction => "second-transaction",
            OnDeviceRunMode::SecondInstance => "second-instance",
        }
    }

    /// 从命令行的写法认回模式；不认识的是 None。
    #[must_use]
    pub fn from_argument(argument: &str) -> Option<OnDeviceRunMode> {
        OnDeviceRunMode::ALL
            .into_iter()
            .find(|mode| mode.argument() == argument)
    }

    /// 用法里列的那一串：`direct | page-cache | …`，按 [`OnDeviceRunMode::ALL`] 的次序。
    #[must_use]
    pub fn every_argument_for_usage() -> String {
        OnDeviceRunMode::ALL
            .iter()
            .map(|mode| mode.argument())
            .collect::<Vec<_>>()
            .join(" | ")
    }

    #[must_use]
    pub const fn publishes_after_the_first_transaction(self) -> PublishesAfterTheFirstTransaction {
        match self {
            OnDeviceRunMode::Direct
            | OnDeviceRunMode::PageCache
            | OnDeviceRunMode::SkipFirstTransactionBarrier => {
                PublishesAfterTheFirstTransaction::Nothing
            }
            OnDeviceRunMode::SecondTransaction => PublishesAfterTheFirstTransaction::SecondVersion,
            OnDeviceRunMode::SecondInstance => {
                PublishesAfterTheFirstTransaction::SecondVersionThenSecondInstance
            }
        }
    }

    /// 这个模式最后发布的那一版文件内容：冷重开之后该读回的。
    #[must_use]
    pub fn last_published_content(self) -> Vec<u8> {
        match self.publishes_after_the_first_transaction() {
            PublishesAfterTheFirstTransaction::Nothing => first_file_content(),
            PublishesAfterTheFirstTransaction::SecondVersion => second_file_content(),
            PublishesAfterTheFirstTransaction::SecondVersionThenSecondInstance => {
                third_file_content()
            }
        }
    }
}

/// 发布 B 的写入时刻：第一个事务之后 60 秒（与 E142 的固定时刻一样不取系统时钟，宿主重跑才同字节）。
pub const SECOND_VERSION_WRITE_TIME_SECONDS: u64 = FIXED_WRITE_TIME_SECONDS + 60;
/// 发布 C 的写入时刻：第一个事务之后 120 秒。
pub const THIRD_VERSION_WRITE_TIME_SECONDS: u64 = FIXED_WRITE_TIME_SECONDS + 120;

/// 第二版的内容（发布 B）：与第一版不同长度、不同字节，读回时分得开。
#[must_use]
pub fn second_file_content() -> Vec<u8> {
    (0..4100usize)
        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
        .collect()
}

/// 第三版的内容（`second-instance` 模式的发布 C）：与前两版都不同长度、不同字节。
#[must_use]
pub fn third_file_content() -> Vec<u8> {
    (0..2500usize)
        .map(|index| u8::try_from((index * 7 + 11) % 253).expect("小于 256"))
        .collect()
}

/// 一次发布失败：发布交回的错，以及这次发布的写入口上落盘阶段失败的发布各自已记的写
/// （`PoolWriter::writes_of_failed_publishes`，增补 2 收口表第 58 行：失败那次落盘的写不属于任何一次成功发布的账，
/// 不交出来，设备一层数到的写就对不上）。落盘之前就失败的（准入、释放判定、分配）一个写都没发，这一份是空的。
#[derive(Debug)]
pub struct FailedPublish {
    pub cause: PublishError,
    pub writes_of_failed_publishes: Vec<WritesByStructureKind>,
}

/// 发布 B 的分配器：从第一个事务的分配记录重建（与可写挂载同一条路）。
#[must_use]
pub fn allocator_rebuilt_from_the_records_of<Device: BlockDevice>(
    devices: &[(DeviceIdentity, Device)],
    version: &TransactionOutput,
) -> PoolAllocator {
    PoolAllocator::rebuild_from_records(
        devices
            .iter()
            .map(|(identity, device)| DeviceFreeMap::new(*identity, device.size_in_bytes()))
            .collect(),
        version.allocation_records.clone(),
    )
}

/// 发布 B：接在第一个事务那一版上覆盖写第二版，同一个实例。
///
/// # Errors
/// 发布的错，连同这次写入口的失败账（[`FailedPublish`]）。
pub fn publish_the_second_version<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    allocator: &mut PoolAllocator,
    first_version: &TransactionOutput,
) -> Result<TransactionOutput, FailedPublish> {
    publish_one_overwrite(
        parameters,
        devices,
        allocator,
        first_version,
        OverwrittenVersion {
            content: &second_file_content(),
            write_time_seconds: SECOND_VERSION_WRITE_TIME_SECONDS,
            instance: first_version.root.instance,
        },
    )
}

/// 发布 C：可写挂载之后，接在挂载交回的现行那一版上覆盖写第三版，挂载取到的实例。
///
/// # Errors
/// 发布的错，连同这次写入口的失败账（[`FailedPublish`]）。
pub fn publish_the_third_version<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    allocator: &mut PoolAllocator,
    mounted_version: &TransactionOutput,
    mounted_instance: InstanceGeneration,
) -> Result<TransactionOutput, FailedPublish> {
    publish_one_overwrite(
        parameters,
        devices,
        allocator,
        mounted_version,
        OverwrittenVersion {
            content: &third_file_content(),
            write_time_seconds: THIRD_VERSION_WRITE_TIME_SECONDS,
            instance: mounted_instance,
        },
    )
}

/// 一次覆盖写要写的那一版：内容、写入时刻、发布用的实例代号。
struct OverwrittenVersion<'content> {
    content: &'content [u8],
    write_time_seconds: u64,
    instance: InstanceGeneration,
}

/// 一个新写入口上的一次覆盖写；失败时把这个写入口的失败账一起交回。
fn publish_one_overwrite<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    allocator: &mut PoolAllocator,
    previous_version: &TransactionOutput,
    version: OverwrittenVersion<'_>,
) -> Result<TransactionOutput, FailedPublish> {
    let mut pool = PoolWriter::new(parameters, devices);
    publish_overwrite(
        &mut pool,
        allocator,
        previous_version,
        FirstFile {
            content: version.content,
            write_time_seconds: version.write_time_seconds,
        },
        version.instance,
    )
    .map_err(|cause| FailedPublish {
        cause,
        writes_of_failed_publishes: pool.writes_of_failed_publishes().to_vec(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_mode_argument_reads_back_as_the_same_mode_and_unknown_text_is_refused() {
        for mode in OnDeviceRunMode::ALL {
            assert_eq!(OnDeviceRunMode::from_argument(mode.argument()), Some(mode));
        }
        assert_eq!(OnDeviceRunMode::from_argument("second_instance"), None);
        assert_eq!(
            OnDeviceRunMode::every_argument_for_usage(),
            "direct | page-cache | skip-first-transaction-barrier | second-transaction | second-instance"
        );
    }

    /// 三版内容两两不同（长度与字节）：冷重开读回哪一版分得开。
    #[test]
    fn the_three_versions_differ_in_length_and_bytes_and_each_mode_reads_back_its_last_one() {
        let first = first_file_content();
        let second = second_file_content();
        let third = third_file_content();
        assert_eq!((first.len(), second.len(), third.len()), (3000, 4100, 2500));
        assert_ne!(second, first);
        assert_ne!(third, first);
        assert_ne!(third, second);
        assert_eq!(OnDeviceRunMode::Direct.last_published_content(), first);
        assert_eq!(OnDeviceRunMode::PageCache.last_published_content(), first);
        assert_eq!(
            OnDeviceRunMode::SkipFirstTransactionBarrier.last_published_content(),
            first
        );
        assert_eq!(
            OnDeviceRunMode::SecondTransaction.last_published_content(),
            second
        );
        assert_eq!(
            OnDeviceRunMode::SecondInstance.last_published_content(),
            third
        );
    }
}
