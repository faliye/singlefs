//! 一次写请求按切分纪律切成若干事务：一事务一单元（里程碑「第二个事务」并行线一的写路径那一半）。
//!
//! 压着它的条款：
//! - D16（发布语义） 已定项 5 末段逐字「事务切分纪律是一事务一单元」「事务切分纪律把一次写请求按单元切成若干事务各自取号，
//!   一个事务最多写一个单元的用户数据，**同一次请求切出的若干事务按其单元的 key 升序取号**」；
//! - D23（journal 的角色与格式） 已定项 7：事务号按实例计数、从 1 起，一条记录一个事务号；
//! - C310（事务切分纪律与记录数口径打架） 2026-09-16 用户定案「一条记录装一个事务连同它的元数据」
//!   ⇒ N 个数据单元 = N 个事务 = N 条记录；
//! - D4（校验和位置） 已定项 5：单元恒 32768 含头，净荷不是 2 的幂，文件偏移到单元做除法（净荷容量取
//!   [`crate::unit::data_unit_payload_capacity`]，不在这里另写一个数）。
//!
//! 这个模块只算切分：每个事务写哪一段文件字节、取几号事务号，在这里定死。它不发一个写、不动分配器。
//! 每个事务的 `payload_start` 就是 extent 叶记录 key 的 offset 段与数据单元头的锚点偏移（文件字节偏移，
//! D8（核心索引结构） 已定项 3，C490（extent 叶 key 的 offset 段没定单位） 2026-09-23 定），发布路径直接拿它编 key。

use crate::address::{DataUnitIndexInFile, FileOffsetInBytes};
use crate::unit::data_unit_payload_capacity;

/// 切分出来的一个事务：它写文件里哪一个数据单元、哪一段文件字节、取几号事务号。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OneUnitTransaction {
    /// 事务号，按实例计数、从 1 起（D23（journal 的角色与格式） 已定项 7）。同一次请求切出的若干事务连号递增。
    pub transaction_number: u64,
    /// 这个事务写文件里第几个数据单元。
    pub unit_index_in_file: DataUnitIndexInFile,
    /// 这个单元载荷的第一个文件字节。
    pub payload_start: FileOffsetInBytes,
    /// 这个单元载荷之后的第一个文件字节（不含）。
    pub payload_end_exclusive: FileOffsetInBytes,
}

impl OneUnitTransaction {
    /// 这个事务写多少字节用户数据：最后一个单元之外恒等于净荷容量。
    #[must_use]
    pub const fn payload_length_in_bytes(self) -> u64 {
        self.payload_end_exclusive.0 - self.payload_start.0
    }

    /// 从整份内容里取这个事务要写的那一段。
    ///
    /// # Panics
    /// 给的内容比切分时那份短 ⇒ 切分与取载荷读的不是同一份输入，断言拦住，不许拿一段错位的字节去装单元。
    #[must_use]
    pub fn payload_of(self, content: &[u8]) -> &[u8] {
        let content_length = u64::try_from(content.len()).expect("内容长度");
        assert!(
            self.payload_end_exclusive.0 <= content_length,
            "切分算出的载荷区间越出内容：切分读的内容长度与取载荷读的不是同一份"
        );
        let start = usize::try_from(self.payload_start.0).expect("区间起点在内容长度之内");
        let end = usize::try_from(self.payload_end_exclusive.0).expect("区间终点在内容长度之内");
        &content[start..end]
    }
}

/// 一次顺序写请求要写几个数据单元：内容长度除净荷容量向上取整，长度为 0 时是 1
/// （声明长度为 0 的数据单元照样是一个单元，第一个事务那一档的下界）。
#[must_use]
pub fn data_unit_count_of_a_sequential_write(content_length_in_bytes: u64) -> u64 {
    let payload_capacity = payload_capacity_in_bytes();
    content_length_in_bytes.div_ceil(payload_capacity).max(1)
}

/// 把一次顺序写请求按切分纪律切成若干一单元事务：第 `unit_index` 个事务写文件字节
/// `[unit_index × 净荷容量, min(内容长度, (unit_index + 1) × 净荷容量))`，事务号从 `first_transaction_number`
/// 起连号递增。
///
/// **为什么这个次序就是 D16（发布语义） 已定项 5 要的「按 key 升序取号」**：extent key 三段是
/// `(locality_id, inode, offset)`（D8（核心索引结构） 已定项 3），一次写请求内前两段不变，第三段就是文件字节偏移
/// （2026-09-23 定）⇒ 文件偏移升序即 key 升序。
///
/// # Panics
/// 事务号加到绕回 u64，或算出的区间起点乘溢出：两样都要求内容长度接近 u64 上界，而内容是一段内存里的切片。
#[must_use]
pub fn split_sequential_write_into_one_unit_transactions(
    content_length_in_bytes: u64,
    first_transaction_number: u64,
) -> Vec<OneUnitTransaction> {
    let payload_capacity = payload_capacity_in_bytes();
    // 迭代次数的上界就是单元数，循环体只读 unit_index，不跨轮携带状态，也没有提前出口。
    let unit_count = data_unit_count_of_a_sequential_write(content_length_in_bytes);
    let mut transactions = Vec::new();
    for unit_index in 0..unit_count {
        let payload_start = DataUnitIndexInFile(unit_index)
            .first_file_byte(payload_capacity)
            .0;
        let payload_end_exclusive = payload_start
            .checked_add(payload_capacity)
            .expect("单元数由内容长度除净荷容量得出，加回去不超过内容长度加一个单元")
            .min(content_length_in_bytes);
        transactions.push(OneUnitTransaction {
            transaction_number: first_transaction_number
                .checked_add(unit_index)
                .expect("事务号按实例计数从 1 起（D23（journal 的角色与格式） 已定项 7），加一次请求的单元数不绕回"),
            unit_index_in_file: DataUnitIndexInFile(unit_index),
            payload_start: FileOffsetInBytes(payload_start),
            payload_end_exclusive: FileOffsetInBytes(payload_end_exclusive),
        });
    }
    let last = transactions
        .last()
        .expect("单元数取过 max(1)：长度为 0 的请求也切出一个事务");
    assert_eq!(
        last.payload_end_exclusive,
        FileOffsetInBytes(content_length_in_bytes),
        "切分不许漏字节：最后一个事务收在文件末尾"
    );
    transactions
}

/// 一个数据单元装得下多少字节用户数据，换成 `u64`：唯一的来源是
/// [`crate::unit::data_unit_payload_capacity`]，这里不另写一个数。
fn payload_capacity_in_bytes() -> u64 {
    u64::try_from(data_unit_payload_capacity()).expect("32634")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 净荷容量是 32634（32768 − 头 105 − 预留 29，字节表二）：切分的每个数都压在它上面，先把它钉住。
    #[test]
    fn the_split_measures_file_bytes_in_units_of_the_data_unit_payload_capacity() {
        assert_eq!(payload_capacity_in_bytes(), 32_634);
    }

    #[test]
    fn a_write_no_longer_than_one_payload_splits_into_one_transaction_covering_the_whole_content() {
        for content_length_in_bytes in [0, 1, 3000, 4100, 32_633, 32_634] {
            let transactions =
                split_sequential_write_into_one_unit_transactions(content_length_in_bytes, 7);
            assert_eq!(
                transactions,
                vec![OneUnitTransaction {
                    transaction_number: 7,
                    unit_index_in_file: DataUnitIndexInFile(0),
                    payload_start: FileOffsetInBytes(0),
                    payload_end_exclusive: FileOffsetInBytes(content_length_in_bytes),
                }],
                "{content_length_in_bytes} 字节只要一个单元"
            );
            assert_eq!(
                data_unit_count_of_a_sequential_write(content_length_in_bytes),
                1
            );
        }
    }

    /// 净荷容量再加一个字节就要两个事务：第二个事务只写那一个字节，事务号接着第一个数
    /// （D23（journal 的角色与格式） 已定项 7：同一实例内事务号连号）。
    #[test]
    fn one_byte_past_the_payload_capacity_splits_into_two_transactions_with_consecutive_numbers() {
        let transactions = split_sequential_write_into_one_unit_transactions(32_635, 2);
        assert_eq!(
            transactions,
            vec![
                OneUnitTransaction {
                    transaction_number: 2,
                    unit_index_in_file: DataUnitIndexInFile(0),
                    payload_start: FileOffsetInBytes(0),
                    payload_end_exclusive: FileOffsetInBytes(32_634),
                },
                OneUnitTransaction {
                    transaction_number: 3,
                    unit_index_in_file: DataUnitIndexInFile(1),
                    payload_start: FileOffsetInBytes(32_634),
                    payload_end_exclusive: FileOffsetInBytes(32_635),
                },
            ]
        );
        assert_eq!(transactions[0].payload_length_in_bytes(), 32_634);
        assert_eq!(transactions[1].payload_length_in_bytes(), 1);
    }

    /// 跨过 67（journal 记录点名项上限那一档）与 144（一片 extent 叶装得下的记录数）两个门槛：
    /// 单元数、区间的连续性、事务号的连号与升序都不因门槛变。整份内容被恰好一次地盖住——
    /// 相邻两个事务首尾相接、不重不漏。
    #[test]
    fn the_split_covers_every_file_byte_exactly_once_across_the_sixty_seven_and_one_hundred_forty_four_unit_thresholds(
    ) {
        let payload_capacity = payload_capacity_in_bytes();
        for expected_unit_count in [66u64, 67, 68, 143, 144, 145] {
            for last_unit_payload in [1u64, payload_capacity] {
                let content_length_in_bytes =
                    (expected_unit_count - 1) * payload_capacity + last_unit_payload;
                let transactions =
                    split_sequential_write_into_one_unit_transactions(content_length_in_bytes, 1);
                assert_eq!(
                    u64::try_from(transactions.len()).expect("单元数"),
                    expected_unit_count,
                    "{content_length_in_bytes} 字节要 {expected_unit_count} 个单元"
                );
                assert_eq!(
                    transactions[0].payload_start,
                    FileOffsetInBytes(0),
                    "第一个事务从文件开头写起"
                );
                for (previous, next) in transactions.iter().zip(transactions.iter().skip(1)) {
                    assert_eq!(
                        previous.payload_end_exclusive, next.payload_start,
                        "相邻两个事务首尾相接：不重不漏"
                    );
                    assert_eq!(
                        previous.payload_length_in_bytes(),
                        payload_capacity,
                        "最后一个之外每个事务写满一个净荷"
                    );
                    assert_eq!(
                        next.transaction_number,
                        previous.transaction_number + 1,
                        "事务号连号"
                    );
                    assert_eq!(
                        next.unit_index_in_file.0,
                        previous.unit_index_in_file.0 + 1,
                        "单元序号升序：文件偏移升序即 extent key 升序（D16（发布语义） 已定项 5）"
                    );
                }
                let last = transactions.last().expect("至少一个事务");
                assert_eq!(
                    last.payload_end_exclusive,
                    FileOffsetInBytes(content_length_in_bytes),
                    "最后一个事务收在文件末尾"
                );
                assert_eq!(last.payload_length_in_bytes(), last_unit_payload);
            }
        }
    }

    /// 取载荷读的是切分算出的那一段：逐个事务拼回去与整份内容逐字节相同。
    #[test]
    fn the_payload_slices_of_all_transactions_concatenate_back_into_the_whole_content() {
        let content: Vec<u8> = (0..100_000u32)
            .map(|index| u8::try_from(index % 251).expect("小于 256"))
            .collect();
        let transactions = split_sequential_write_into_one_unit_transactions(
            u64::try_from(content.len()).expect("内容长度"),
            1,
        );
        assert_eq!(transactions.len(), 4, "100000 ÷ 32634 向上取整是 4");
        let rejoined: Vec<u8> = transactions
            .iter()
            .flat_map(|transaction| transaction.payload_of(&content).to_vec())
            .collect();
        assert_eq!(rejoined, content);
    }
}
