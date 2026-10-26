//! 一次写请求按切分纪律切成若干事务：一事务一单元（里程碑「覆盖写、释放、回退与复用」并行线一的写路径那一半）。
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
/// （声明长度为 0 的数据单元照样是一个单元，新池新建文件那一档的下界）。
#[must_use]
pub fn data_unit_count_of_a_sequential_write(content_length_in_bytes: u64) -> u64 {
    let payload_capacity = payload_capacity_in_bytes();
    content_length_in_bytes.div_ceil(payload_capacity).max(1)
}

/// 读侧看一个 inode 在 extent 树上段叶里的那一条（D8（核心索引结构） 已定项 14；标签 3、4 与「标签 0 配 size > 0」是判决
/// `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K5 加的）。封闭集合，`match` 不写通配臂。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataUnitsOfTheInode {
    /// 上段叶里没有它的条目：建出来之后一次都没写过内容（`transaction::publish_new_inodes` 建的 inode 长度 0、数据单元与
    /// extent 树都不动）。
    NoneWritten,
    /// 条目标签 1、2：指着下段根或一个内联的数据指针，最后一个单元之后没有洞。
    SomeWritten,
    /// 条目标签 3、4、0：最后一个存在的单元之后到 size 是洞（标签 0 一个单元都没有、整个文件是洞）。
    HoleAfterTheLastDataUnit,
}

/// 读侧的单元数对不上 size：交回 size 按净荷容量算出来的单元数（报错用）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DataUnitCountDoesNotMatchTheFileSize {
    pub data_units_implied_by_the_file_size: u64,
}

/// 读侧：盘上这个 inode 有 `data_units_on_disk` 个数据单元（从 0 起连号）、size 是 `file_size_in_bytes` 时，size 之内要读几个单元
/// （之后到 size 是洞，读出 0、不读盘）。size 按净荷容量算出的单元数是 ⌈size ÷ 净荷⌉.max(1)，与写侧切分同一条除法
/// （[`data_unit_count_of_a_sequential_write`]）：
/// - 没有条目：长度 0、0 个单元；
/// - 标签 1、2：单元数不少于它（多出来的是截断中间态里 size 之外还挂着的单元，判决 K2「读者容忍」：不读、不判声明长度）；
/// - 标签 3、4、0：单元数少于它（相等就该写成 1、2）。
///
/// # Errors
/// 三样对不上 ⇒ [`DataUnitCountDoesNotMatchTheFileSize`]（没有条目而 size 不是 0 时带除法给的数）。
pub fn data_units_read_within_the_file_size(
    file_size_in_bytes: u64,
    data_units_on_disk: u64,
    data_units_of_the_inode: DataUnitsOfTheInode,
) -> Result<u64, DataUnitCountDoesNotMatchTheFileSize> {
    let data_units_implied_by_the_file_size =
        data_unit_count_of_a_sequential_write(file_size_in_bytes);
    let refused = Err(DataUnitCountDoesNotMatchTheFileSize {
        data_units_implied_by_the_file_size,
    });
    match data_units_of_the_inode {
        DataUnitsOfTheInode::NoneWritten if file_size_in_bytes == 0 && data_units_on_disk == 0 => {
            Ok(0)
        }
        DataUnitsOfTheInode::NoneWritten => refused,
        DataUnitsOfTheInode::SomeWritten
            if data_units_on_disk >= data_units_implied_by_the_file_size =>
        {
            Ok(data_units_implied_by_the_file_size)
        }
        DataUnitsOfTheInode::HoleAfterTheLastDataUnit
            if data_units_on_disk < data_units_implied_by_the_file_size =>
        {
            Ok(data_units_on_disk)
        }
        DataUnitsOfTheInode::SomeWritten | DataUnitsOfTheInode::HoleAfterTheLastDataUnit => refused,
    }
}

/// 把一次顺序写请求按切分纪律切成若干一单元事务：第 `unit_index` 个事务写文件字节
/// `[unit_index × 净荷容量, min(内容长度, (unit_index + 1) × 净荷容量))`，事务号从 `new_pool_file_creation_number`
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
    new_pool_file_creation_number: u64,
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
            transaction_number: new_pool_file_creation_number
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

/// 一次按偏移的写（判决 `research/prompts/m4-partial-write-r1-main-verification.md` 第二节）里一个要装的数据单元从哪来、读不读盘。
/// 封闭集合，`match` 不写通配臂。「有没有旧单元」只看写之前的单元数，不看任何一次读的结果；「盖满」是这次的字节罩住这个单元整个净荷容量
/// （`[单元号 × 净荷, (单元号 + 1) × 净荷)`），不是只罩住它的声明长度（R2「盖满不读」那一格）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataUnitOfAPartialWrite {
    /// 有旧单元、净荷没被这次的字节整单元盖满（含写到偏移 o 时一起重写、这次一个字节都没写到的 o 之后那几个，R4）：读回旧单元（读盘），
    /// 旧内容取它、这次的字节盖在上面。读不回或校验不过整个写失败（D4（校验和位置） 已定项 4）。
    OldUnitReadBack,
    /// 有旧单元、净荷被这次的字节整单元盖满：不读，内容全取这次的字节。
    OldUnitCoveredWhole,
    /// 没有旧单元、这次写到它而没盖满：读这次发布分到的新槽的残留一次，字节丢掉、旧内容按零（用户 2026-10-26 定「首次写也读回」，
    /// 残留读出错记数、不拒写）。
    NewUnitReadingTheResidueOfItsSlot,
    /// 没有旧单元、净荷被这次的字节整单元盖满：不读。
    NewUnitCoveredWhole,
    /// 起点越过末尾或落在洞里时它前面的空隙：补整单元零、不读（存在的单元从 0 起连号，R5）。
    GapFilledWithZeros,
}

impl DataUnitOfAPartialWrite {
    /// 运行时报出这个单元走的是哪一格（`.claude/rules/fs-design.md` 五条硬要求第 4 条）。
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            DataUnitOfAPartialWrite::OldUnitReadBack => "partial_write_old_unit_read_back",
            DataUnitOfAPartialWrite::OldUnitCoveredWhole => "partial_write_old_unit_covered_whole",
            DataUnitOfAPartialWrite::NewUnitReadingTheResidueOfItsSlot => {
                "partial_write_new_unit_reading_the_residue_of_its_slot"
            }
            DataUnitOfAPartialWrite::NewUnitCoveredWhole => "partial_write_new_unit_covered_whole",
            DataUnitOfAPartialWrite::GapFilledWithZeros => "partial_write_gap_filled_with_zeros",
        }
    }

    /// 这个单元在读回旧单元那一步读不读盘（读的是旧单元）。残留读不在这里：它在分配之后读这次分到的槽。
    #[must_use]
    pub const fn reads_the_old_unit_back(self) -> bool {
        match self {
            DataUnitOfAPartialWrite::OldUnitReadBack => true,
            DataUnitOfAPartialWrite::OldUnitCoveredWhole
            | DataUnitOfAPartialWrite::NewUnitReadingTheResidueOfItsSlot
            | DataUnitOfAPartialWrite::NewUnitCoveredWhole
            | DataUnitOfAPartialWrite::GapFilledWithZeros => false,
        }
    }

    /// 这个单元在分配之后、任何写之前读不读这次分到的槽的残留。
    #[must_use]
    pub const fn reads_the_residue_of_its_slot(self) -> bool {
        match self {
            DataUnitOfAPartialWrite::NewUnitReadingTheResidueOfItsSlot => true,
            DataUnitOfAPartialWrite::OldUnitReadBack
            | DataUnitOfAPartialWrite::OldUnitCoveredWhole
            | DataUnitOfAPartialWrite::NewUnitCoveredWhole
            | DataUnitOfAPartialWrite::GapFilledWithZeros => false,
        }
    }
}

/// 一次按偏移的写在一个文件上要做的事（只算，不读不写、不动分配器）：写之前这个文件挂着几个单元、size 多少，这次写 `[offset, offset + 长度)`。
/// 写之后的 size 取写之前的与写的终点里大的（跨末尾的写同时是一次变长，判决第二节 R5 那一格）；写之后的单元数取写之前的与这次写到的最后一个
/// 单元号加一里大的（单元从 0 起连号，洞只在尾部）。重写的是从 [`Self::first_rewritten_unit`] 起到写之后最后一个单元的那一段（R4：写到偏移 o
/// 时把 o 之后的单元一起重写，「出生 txg 随偏移不降」照旧成立，读法 Z 加写序不改）：
/// - 写落在已有的单元上（o 小于写之前的单元数）：从 o 起；
/// - 写落在最后一个存在的单元之后（空隙、洞里、紧接末尾）：最后一个存在的单元声明长度不满一个净荷时从它起（它要补满，按已有单元读回），
///   否则从写之前的单元数起（之前的单元一个都不重写）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PartialWriteOfAFile {
    pub data_units_before: u64,
    pub size_before_in_bytes: u64,
    pub offset: FileOffsetInBytes,
    pub length_in_bytes: std::num::NonZeroU64,
    /// `offset + length_in_bytes`（构造时核过不越过 u64）。
    end_exclusive: u64,
}

impl PartialWriteOfAFile {
    /// 写的终点越过 u64 ⇒ `None`（调用方在任何读写之前拒）。
    #[must_use]
    pub fn new(
        data_units_before: u64,
        size_before_in_bytes: u64,
        offset: FileOffsetInBytes,
        length_in_bytes: std::num::NonZeroU64,
    ) -> Option<Self> {
        let end_exclusive = offset.0.checked_add(length_in_bytes.get())?;
        Some(PartialWriteOfAFile {
            data_units_before,
            size_before_in_bytes,
            offset,
            length_in_bytes,
            end_exclusive,
        })
    }

    /// 写的终点（不含）。
    #[must_use]
    pub const fn end_exclusive(self) -> FileOffsetInBytes {
        FileOffsetInBytes(self.end_exclusive)
    }

    /// 写之后那条记录的 size。
    #[must_use]
    pub fn size_after_in_bytes(self) -> u64 {
        self.size_before_in_bytes.max(self.end_exclusive)
    }

    /// 这次的字节落到的第一个单元（o）。
    #[must_use]
    pub fn first_written_unit(self) -> DataUnitIndexInFile {
        DataUnitIndexInFile(self.offset.0 / payload_capacity_in_bytes())
    }

    /// 这次的字节落到的最后一个单元。
    #[must_use]
    pub fn last_written_unit(self) -> DataUnitIndexInFile {
        DataUnitIndexInFile((self.end_exclusive - 1) / payload_capacity_in_bytes())
    }

    /// 写之后这个文件挂着几个单元（从 0 起连号）。
    #[must_use]
    pub fn data_units_after(self) -> u64 {
        self.data_units_before.max(self.last_written_unit().0 + 1)
    }

    /// 这次从第几个单元起一起重写（到 [`Self::data_units_after`] 为止，见类型的文档）。
    #[must_use]
    pub fn first_rewritten_unit(self) -> DataUnitIndexInFile {
        let first_written = self.first_written_unit().0;
        if first_written < self.data_units_before {
            return DataUnitIndexInFile(first_written);
        }
        match self.data_units_before.checked_sub(1) {
            Some(last_existing)
                if payload_length_within_a_size(last_existing, self.size_before_in_bytes)
                    < payload_capacity_in_bytes() =>
            {
                DataUnitIndexInFile(last_existing)
            }
            Some(_) | None => DataUnitIndexInFile(self.data_units_before),
        }
    }

    /// 这次重写的每个单元号，升序。
    #[must_use]
    pub fn rewritten_units(self) -> std::ops::Range<u64> {
        self.first_rewritten_unit().0..self.data_units_after()
    }

    /// 第 `unit` 个单元写之后的声明长度：min(写之后的 size − 起点, 净荷)。
    #[must_use]
    pub fn declared_length_after_the_write(self, unit: DataUnitIndexInFile) -> u64 {
        payload_length_within_a_size(unit.0, self.size_after_in_bytes())
    }

    /// 第 `unit` 个单元（在 [`Self::rewritten_units`] 里）这次怎么装。
    ///
    /// # Panics
    /// `unit` 不在这次重写的那一段里：调用方只问那一段。
    #[must_use]
    pub fn data_unit(self, unit: DataUnitIndexInFile) -> DataUnitOfAPartialWrite {
        assert!(
            self.rewritten_units().contains(&unit.0),
            "只问这次重写的那一段里的单元"
        );
        let payload_capacity = payload_capacity_in_bytes();
        let unit_start = unit.0 * payload_capacity;
        let unit_end_exclusive = unit_start + payload_capacity;
        let is_written = self.offset.0 < unit_end_exclusive && unit_start < self.end_exclusive;
        let is_covered_whole =
            self.offset.0 <= unit_start && unit_end_exclusive <= self.end_exclusive;
        let has_an_old_unit = unit.0 < self.data_units_before;
        match (has_an_old_unit, is_written, is_covered_whole) {
            (true, _, true) => DataUnitOfAPartialWrite::OldUnitCoveredWhole,
            (true, _, false) => DataUnitOfAPartialWrite::OldUnitReadBack,
            (false, false, _) => DataUnitOfAPartialWrite::GapFilledWithZeros,
            (false, true, true) => DataUnitOfAPartialWrite::NewUnitCoveredWhole,
            (false, true, false) => DataUnitOfAPartialWrite::NewUnitReadingTheResidueOfItsSlot,
        }
    }

    /// 这次的字节落在第 `unit` 个单元净荷里的那一段（[`BytesOfTheWriteInOneUnit`]）；这次没写到它交回 `None`。
    #[must_use]
    pub fn bytes_written_into(self, unit: DataUnitIndexInFile) -> Option<BytesOfTheWriteInOneUnit> {
        let payload_capacity = payload_capacity_in_bytes();
        let unit_start = unit.0 * payload_capacity;
        let unit_end_exclusive = unit_start + payload_capacity;
        let from = self.offset.0.max(unit_start);
        let until = self.end_exclusive.min(unit_end_exclusive);
        (from < until).then(|| BytesOfTheWriteInOneUnit {
            offset_in_the_payload: from - unit_start,
            offset_in_the_bytes_written: from - self.offset.0,
            length_in_bytes: until - from,
        })
    }
}

/// 一次按偏移的写落在一个单元里的那一段（[`PartialWriteOfAFile::bytes_written_into`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BytesOfTheWriteInOneUnit {
    pub offset_in_the_payload: u64,
    pub offset_in_the_bytes_written: u64,
    pub length_in_bytes: u64,
}

/// 第 `unit` 个单元在 size 为 `size_in_bytes` 的文件里装几字节：min(size − 起点, 净荷)，起点在 size 之外是 0
/// （与 `truncate::declared_length_within_a_size` 同一条式子；`truncate` 依赖这个模块，这里不反过来引它）。
fn payload_length_within_a_size(unit: u64, size_in_bytes: u64) -> u64 {
    let payload_capacity = payload_capacity_in_bytes();
    size_in_bytes
        .saturating_sub(unit.saturating_mul(payload_capacity))
        .min(payload_capacity)
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
    fn write_no_longer_than_one_payload_splits_into_one_transaction_covering_the_whole_content() {
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

    /// 净荷容量再加一个字节就要两个事务：覆盖写、释放、回退与复用只写那一个字节，事务号接着第一个数
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

    /// 跨过 64（journal 记录点名项上限 ⌊(4096 − 487) ÷ 56⌋ 那一档）与 144（一片 extent 下段叶罩的单元数
    /// ⌊(16384 − 163) ÷ 112⌋）两个门槛：单元数、区间的连续性、事务号的连号与升序都不因门槛变。整份内容被恰好一次地盖住——
    /// 相邻两个事务首尾相接、不重不漏。
    #[test]
    fn the_split_covers_every_file_byte_exactly_once_across_the_sixty_four_and_one_hundred_forty_four_unit_thresholds(
    ) {
        // 切分本身不读这两个常量，取样点写成字面量；先钉住两个门槛今天的值，门槛一变这里先红，
        // 取样点才会跟着挪，不会贴着旧门槛照样绿下去。
        assert_eq!(
            singlefs_format::JOURNAL_NAMED_ENTRIES_PER_RECORD,
            64,
            "取样点 63 / 64 / 65 压的是一条 journal 记录的点名项上限；上限变了要跟着挪取样点"
        );
        assert_eq!(
            singlefs_format::EXTENT_TREE_LOWER_LEAF_DATA_UNITS,
            144,
            "取样点 143 / 144 / 145 压的是一片 extent 下段叶罩的单元数；它变了要跟着挪取样点"
        );
        let payload_capacity = payload_capacity_in_bytes();
        for expected_unit_count in [63u64, 64, 65, 143, 144, 145] {
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
                    "新池新建文件从文件开头写起"
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

    /// 读侧的单元数：没有条目的长度 0 ⇒ 0 个；标签 1、2 长度 0 ⇒ 1 个（写侧的下界），多出来的（截断中间态）只读 size 之内的；
    /// 标签 3、4、0 少于除法给的数、读存在的那几个，不少于就拒；没有条目而长度不是 0 拒。
    #[test]
    fn the_reader_reads_the_data_units_within_the_size_and_refuses_counts_the_tag_does_not_allow() {
        let refused = |implied: u64| {
            Err(DataUnitCountDoesNotMatchTheFileSize {
                data_units_implied_by_the_file_size: implied,
            })
        };
        assert_eq!(
            data_units_read_within_the_file_size(0, 0, DataUnitsOfTheInode::NoneWritten),
            Ok(0)
        );
        assert_eq!(
            data_units_read_within_the_file_size(32_635, 0, DataUnitsOfTheInode::NoneWritten),
            refused(2)
        );
        let without_a_hole = DataUnitsOfTheInode::SomeWritten;
        assert_eq!(
            data_units_read_within_the_file_size(0, 1, without_a_hole),
            Ok(1)
        );
        assert_eq!(
            data_units_read_within_the_file_size(32_635, 2, without_a_hole),
            Ok(2),
            "净荷容量加一个字节是两个单元"
        );
        assert_eq!(
            data_units_read_within_the_file_size(32_635, 5, without_a_hole),
            Ok(2),
            "截断中间态：size 之外还挂着三个，只读 size 之内的两个"
        );
        assert_eq!(
            data_units_read_within_the_file_size(32_635, 1, without_a_hole),
            refused(2)
        );
        let with_a_hole = DataUnitsOfTheInode::HoleAfterTheLastDataUnit;
        assert_eq!(
            data_units_read_within_the_file_size(32_635, 1, with_a_hole),
            Ok(1)
        );
        assert_eq!(
            data_units_read_within_the_file_size(32_635, 0, with_a_hole),
            Ok(0),
            "标签 0 配 size > 0：整个文件是洞"
        );
        assert_eq!(
            data_units_read_within_the_file_size(32_635, 2, with_a_hole),
            refused(2),
            "单元数与除法相等却声明有洞"
        );
    }

    fn payload() -> u64 {
        payload_capacity_in_bytes()
    }

    fn partial_write(
        data_units_before: u64,
        size_before_in_bytes: u64,
        offset: u64,
        length_in_bytes: u64,
    ) -> PartialWriteOfAFile {
        PartialWriteOfAFile::new(
            data_units_before,
            size_before_in_bytes,
            FileOffsetInBytes(offset),
            std::num::NonZeroU64::new(length_in_bytes).expect("用例给的长度不是 0"),
        )
        .expect("用例的终点装得进 u64")
    }

    fn kinds_of_the_rewritten_units(write: PartialWriteOfAFile) -> Vec<DataUnitOfAPartialWrite> {
        write
            .rewritten_units()
            .map(|unit| write.data_unit(DataUnitIndexInFile(unit)))
            .collect()
    }

    /// 判决 m4-partial-write-r1 第二节 R2 逐格表与 R4：三个单元的文件改单元 1 中间 4 KiB，单元 1、2 一起重写、都读回旧单元；
    /// 整单元盖满单元 1 时单元 1 不读、单元 2 照读；单元数与 size 不变。
    #[test]
    fn a_write_inside_existing_units_rewrites_every_unit_from_the_offset_and_reads_back_what_it_does_not_cover(
    ) {
        let size = 3 * payload() - 7;
        let inside = partial_write(3, size, payload() + 100, 4096);
        assert_eq!(inside.first_rewritten_unit(), DataUnitIndexInFile(1));
        assert_eq!(inside.data_units_after(), 3);
        assert_eq!(inside.size_after_in_bytes(), size);
        assert_eq!(
            kinds_of_the_rewritten_units(inside),
            vec![
                DataUnitOfAPartialWrite::OldUnitReadBack,
                DataUnitOfAPartialWrite::OldUnitReadBack
            ]
        );
        let covering_unit_one = partial_write(3, size, payload(), payload());
        assert_eq!(
            kinds_of_the_rewritten_units(covering_unit_one),
            vec![
                DataUnitOfAPartialWrite::OldUnitCoveredWhole,
                DataUnitOfAPartialWrite::OldUnitReadBack
            ],
            "盖满单元 1 的净荷容量：不读它；单元 2 一起重写、照读"
        );
    }

    /// 0 个单元的 inode（新建、size 0）写 4 KiB：单元 0 读残留；起点在第三个单元里：前两个补零、不读，第三个读残留。
    #[test]
    fn a_write_into_an_inode_without_units_reads_the_residue_of_the_units_it_writes_and_zeroes_the_gap(
    ) {
        let first_write = partial_write(0, 0, 0, 4096);
        assert_eq!(first_write.first_rewritten_unit(), DataUnitIndexInFile(0));
        assert_eq!(first_write.data_units_after(), 1);
        assert_eq!(first_write.size_after_in_bytes(), 4096);
        assert_eq!(
            kinds_of_the_rewritten_units(first_write),
            vec![DataUnitOfAPartialWrite::NewUnitReadingTheResidueOfItsSlot]
        );
        let past_a_gap = partial_write(0, 0, 2 * payload() + 5, 10);
        assert_eq!(past_a_gap.data_units_after(), 3);
        assert_eq!(
            kinds_of_the_rewritten_units(past_a_gap),
            vec![
                DataUnitOfAPartialWrite::GapFilledWithZeros,
                DataUnitOfAPartialWrite::GapFilledWithZeros,
                DataUnitOfAPartialWrite::NewUnitReadingTheResidueOfItsSlot
            ]
        );
        assert_eq!(
            past_a_gap.declared_length_after_the_write(DataUnitIndexInFile(0)),
            payload(),
            "空隙补整单元零"
        );
        assert_eq!(
            past_a_gap.declared_length_after_the_write(DataUnitIndexInFile(2)),
            15
        );
    }

    /// 写落在最后一个存在的单元之后：它满一个净荷就不重写（从写之前的单元数起），不满就从它起、按已有单元读回；洞里写换不了 size。
    #[test]
    fn a_write_after_the_last_existing_unit_rewrites_that_unit_only_when_it_is_not_full() {
        let after_a_full_unit = partial_write(1, payload(), payload(), 10);
        assert_eq!(
            after_a_full_unit.first_rewritten_unit(),
            DataUnitIndexInFile(1)
        );
        assert_eq!(
            kinds_of_the_rewritten_units(after_a_full_unit),
            vec![DataUnitOfAPartialWrite::NewUnitReadingTheResidueOfItsSlot]
        );
        let after_a_short_unit = partial_write(1, 3000, payload() + 7, 10);
        assert_eq!(
            after_a_short_unit.first_rewritten_unit(),
            DataUnitIndexInFile(0)
        );
        assert_eq!(after_a_short_unit.size_after_in_bytes(), payload() + 17);
        assert_eq!(
            kinds_of_the_rewritten_units(after_a_short_unit),
            vec![
                DataUnitOfAPartialWrite::OldUnitReadBack,
                DataUnitOfAPartialWrite::NewUnitReadingTheResidueOfItsSlot
            ],
            "末尾单元要补满：按已有单元读回"
        );
        let into_the_hole = partial_write(1, 3 * payload(), 2 * payload() + 1, 10);
        assert_eq!(into_the_hole.first_rewritten_unit(), DataUnitIndexInFile(1));
        assert_eq!(into_the_hole.data_units_after(), 3);
        assert_eq!(into_the_hole.size_after_in_bytes(), 3 * payload());
        assert_eq!(
            kinds_of_the_rewritten_units(into_the_hole),
            vec![
                DataUnitOfAPartialWrite::GapFilledWithZeros,
                DataUnitOfAPartialWrite::NewUnitReadingTheResidueOfItsSlot
            ]
        );
    }

    /// 终点越过 u64 不成计划；跨单元的写在两个单元里各落一段。
    #[test]
    fn the_end_of_a_write_past_u64_is_refused_and_a_write_across_a_boundary_lands_in_two_units() {
        assert_eq!(
            PartialWriteOfAFile::new(1, 0, FileOffsetInBytes(u64::MAX), std::num::NonZeroU64::MIN),
            None
        );
        let across = partial_write(3, 3 * payload(), payload() - 4, 10);
        assert_eq!(
            across.bytes_written_into(DataUnitIndexInFile(0)),
            Some(BytesOfTheWriteInOneUnit {
                offset_in_the_payload: payload() - 4,
                offset_in_the_bytes_written: 0,
                length_in_bytes: 4
            })
        );
        assert_eq!(
            across.bytes_written_into(DataUnitIndexInFile(1)),
            Some(BytesOfTheWriteInOneUnit {
                offset_in_the_payload: 0,
                offset_in_the_bytes_written: 4,
                length_in_bytes: 6
            })
        );
        assert_eq!(across.bytes_written_into(DataUnitIndexInFile(2)), None);
    }
}
