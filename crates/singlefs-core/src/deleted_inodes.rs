//! `deleted_inodes` 树（D8（核心索引结构） 已定项 16）：删除中的 inode 号，key = inode 号 8 字节、无 value、不带对象出生代；
//! 队列 = inode 树里 nlink = 0 的号集合（派生态，第 3 条）。队列空时树表条目的根指针置零、一个节点都不写；
//! 非空时只有一个码 2 根（层级 0，条目宽 8 = key 宽）。
//!
//! 任何一版里至多一条 key（第 6 条：会话里每一次用户改动先把队列走空、unlink 一次调用走完全部批）——发布路径按这一条断言。
//! 队列多于一个根装得下的号（2031）要在根之下长层，条款没写怎么长：那一支在构造根的条目时拒成类型化错误
//! （[`DeletedInodesQueueRefusal::MoreKeysThanOneRootNodeHolds`]，不是空间不够），只在构造函数这一层测——发布路径上至多一条，走不到它。

use singlefs_format::DELETED_INODES_ENTRY_BYTES;

use crate::address::InodeNumber;
use crate::unit::index_node_entry_capacity;

/// `deleted_inodes` 树的 key 宽：inode 号 8 字节（与 inode 树的 key 同一个量）。
pub const DELETED_INODES_KEY_WIDTH_IN_BYTES: usize = 8;

/// 一个码 2 根装得下几个号：(16384 − 含预留位的头 131) ÷ 8 的整数部分 = 2031。
#[must_use]
pub fn deleted_inodes_one_root_node_holds() -> usize {
    index_node_entry_capacity(
        DELETED_INODES_KEY_WIDTH_IN_BYTES,
        usize::try_from(DELETED_INODES_ENTRY_BYTES).expect("8"),
    )
}

/// 装 `deleted_inodes` 根时走得到、条款没写的那一格。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeletedInodesQueueRefusal {
    /// 队列里的号多于一个码 2 根装得下的：要在根之下长层，D8（核心索引结构） 已定项 16 只写了「任何一版至多一条 key」，
    /// 没写这棵树怎么长层。拒成类型化错误（第 6 条：「多于一个根装得下那一支拒成类型化错误、不是空间不够」）。
    MoreKeysThanOneRootNodeHolds { keys: usize, capacity: usize },
}

/// `deleted_inodes` 根要装的条目：每个号一条 8 字节 key（小端），按号升序。`keys` 由调用方按升序给、不重复。
///
/// # Errors
/// 号多于一个根装得下的 ⇒ [`DeletedInodesQueueRefusal::MoreKeysThanOneRootNodeHolds`]。
///
/// # Panics
/// `keys` 不是严格升序：队列是发布路径自己按号维护的集合（加一个号、删一个号），乱序说明调用方拼错了。
pub fn deleted_inodes_root_entries(
    keys: &[InodeNumber],
) -> Result<Vec<Vec<u8>>, DeletedInodesQueueRefusal> {
    assert!(
        keys.windows(2).all(|pair| pair[0] < pair[1]),
        "deleted_inodes 的号按升序、不重复：发布路径按集合维护它"
    );
    let capacity = deleted_inodes_one_root_node_holds();
    if keys.len() > capacity {
        return Err(DeletedInodesQueueRefusal::MoreKeysThanOneRootNodeHolds {
            keys: keys.len(),
            capacity,
        });
    }
    Ok(keys
        .iter()
        .map(|key| key.0.to_le_bytes().to_vec())
        .collect())
}

/// 从一个 `deleted_inodes` 根的条目读回号（条目宽在这之前由读者核过等于 8）。
#[must_use]
pub fn inode_numbers_of_the_deleted_inodes_root_entries(entries: &[Vec<u8>]) -> Vec<InodeNumber> {
    entries
        .iter()
        .map(|entry| {
            InodeNumber(u64::from_le_bytes(
                entry[..DELETED_INODES_KEY_WIDTH_IN_BYTES]
                    .try_into()
                    .expect("条目宽在读者那里核过等于 8"),
            ))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 一个号一条 8 字节 key，小端；读回来是同一个号。
    #[test]
    fn one_inode_number_is_one_eight_byte_little_endian_entry_and_reads_back() {
        let entries =
            deleted_inodes_root_entries(&[InodeNumber(0x0102_0304)]).expect("一个号装得下");
        assert_eq!(entries, vec![vec![4, 3, 2, 1, 0, 0, 0, 0]]);
        assert_eq!(
            inode_numbers_of_the_deleted_inodes_root_entries(&entries),
            vec![InodeNumber(0x0102_0304)]
        );
    }

    /// 一个根装 2031 个号；多一个就拒成类型化错误（不是空间不够），这一支只在构造函数这一层测得到。
    #[test]
    fn a_queue_longer_than_one_root_holds_is_refused_as_a_typed_error() {
        let capacity = deleted_inodes_one_root_node_holds();
        assert_eq!(capacity, 2031, "(16384 − 含预留位的头 131) ÷ 8 的整数部分");
        let full: Vec<InodeNumber> = (1..=u64::try_from(capacity).expect("容量"))
            .map(InodeNumber)
            .collect();
        assert_eq!(
            deleted_inodes_root_entries(&full).map(|entries| entries.len()),
            Ok(capacity),
            "正好装满一个根"
        );
        let one_too_many: Vec<InodeNumber> = (1..=u64::try_from(capacity).expect("容量") + 1)
            .map(InodeNumber)
            .collect();
        assert_eq!(
            deleted_inodes_root_entries(&one_too_many),
            Err(DeletedInodesQueueRefusal::MoreKeysThanOneRootNodeHolds {
                keys: capacity + 1,
                capacity,
            })
        );
    }
}
