//! 实审 A3a：盘上内容可控时读者走得到的 panic（代码审阅第 29、32、33、35、38 条，C476 落在 core 读者这一侧的那几处）。
//! 规矩（`/tmp` 规格与 `code-discipline.md`「错误：能恢复的写进类型，破坏不变量的就断言」）：盘上读到的坏内容是可恢复的失败，
//! 每一处改成带名字的错误成员往上交；这里每条用例造一份会让代码 panic、静默收下或报别的成员的坏镜像（各条文档写明是哪一种），钉住该有的结局。
//!
//! 盘都是内存稀疏盘（`singlefs_verification::memory_pool::SparseBlockDevice`），池在进程里 mkfs、取号、暖机、写第一个文件（txg 3）造出来；
//! 改盘上字节之后，指着它的每一道校验和（父指针里的位置条目、映射条目、树表、根记录的自证校验和）都重算，读者走到被改的那一格才判得到。
//! 「在任何写之前拒」用两块盘此刻的整份镜像比：拒之前、拒之后逐字节相同。

mod accounting_and_allocation_record_rows_after_the_root;
mod location_entry_order_and_tree_root_birth_sequences;
mod pool_builders_and_image_rewrites;
mod reserved_bytes_and_pointer_heads_outside_the_first_version;
mod system_configuration_values_outside_the_reader;
mod writable_mount_refusing_tree_and_root_contents;

use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
    TreeIdentifier,
};
use singlefs_core::allocator::{AllocationRecord, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::bytes::{ByteReader, ByteWriter};
use singlefs_core::checksum::{crc32_castagnoli, wide_checksum_with_field_zeroed};
use singlefs_core::code_two_tree::{
    build_internal_entry, parse_internal_entry, InternalEntryRefusal,
};
use singlefs_core::journal::{
    JournalRecord, JournalRecordOrdinalWithinPublish, JournalRecordPlaceInPublish,
    JOURNAL_HEADER_CHECKSUM_OFFSET,
};
use singlefs_core::make_filesystem::{
    allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters,
};
use singlefs_core::mount::{mount_writable, MountError, StillUnreadableAfterOneReread};
use singlefs_core::pointer::{
    BirthSequence, DataPointer, LocationEntry, NodePointer, PointerHead,
    PointerHeadFieldOutsideTheFirstVersion,
};
use singlefs_core::records::{
    mapping_key_for_node, parse_mapping_entry, AccountingEntry, TreeTableEntry,
    STATISTIC_INODE_WATERMARK, TREE_KIND_ACCOUNTING, TREE_KIND_ALLOCATION, TREE_KIND_INODE,
    TREE_KIND_RETIREMENT,
};
use singlefs_core::recovery::{
    choose_system_configuration, every_root_ring_slot, read_root_ring_slot, recover, JournalPolicy,
    PoolReader, RecoveryFailure, RecoveryOutcome, RootRingSlotReading,
    SystemConfigurationValueOutsideWhatThisReaderAccepts,
};
use singlefs_core::root_record::{RootRecord, UnmountMarker};
use singlefs_core::root_ring::{
    slot_offset, target_for_publish, RootRingSlot, RootRingSlotsPerRegion,
};
use singlefs_core::system_configuration::{
    SystemConfiguration, SystemImmutableSizes, SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_new_inodes, warm_up, FirstFile, PoolWriter,
    TransactionOutput,
};
use singlefs_core::unit::{
    build_data_unit, build_index_node, build_packed_unit, parse_data_unit, parse_index_node,
    parse_packed_unit, seal_header_checksum, DataUnitIdentity, PackedIdentity, UnitError,
    UnitStructureDefect, WriteOrder, PACKED_TYPE_INODE, UNIT_CLASS_INDEX_NODE,
};
use singlefs_format::{
    ACCOUNTING_ENTRY_BYTES, ACCOUNTING_KEY_BYTES, ALLOCATION_RECORD_KEY_BYTES, DATA_UNIT_BYTES,
    JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES, NODE_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES,
    UNIT_AREA_START_SLOT,
};
use singlefs_verification::memory_pool::{SparseBlockDevice, SparseDevice};

use pool_builders_and_image_rewrites::*;
