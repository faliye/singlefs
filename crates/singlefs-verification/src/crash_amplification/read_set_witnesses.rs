//! 观察者的读集见证：崩溃镜像上逐段叠写的视图、只按读到的扇区算的见证摘要（`ReadSetWitnessDigester`），
//! 同一桶里按读集分组找相同的见证（`ReadSetWitnessIndex`），恢复报告的摘要。

use super::*;

/// 观察者判红的那个状态存进判定向量的那一项：`observer:<观察者名>:<那一句>`。
pub(super) fn observer_red_item(observer_name: &ObserverName, panic_message: &str) -> String {
    format!("observer:{}:{panic_message}", observer_name.as_str())
}

/// panic 带的那一句（`panic!`、`assert!` 的消息是字符串）；带的不是字符串时写一句说明。
pub(super) fn text_carried_by_the_panic_payload(payload: &(dyn std::any::Any + Send)) -> String {
    payload
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| {
            payload
                .downcast_ref::<&str>()
                .map(|message| (*message).to_string())
        })
        .unwrap_or_else(|| "观察者 panic 时带的不是一句话".to_string())
}

// ------------------------------ 恢复报告分桶、读集见证去重 ------------------------------

pub(super) fn put_item_count(bytes: &mut Vec<u8>, count: usize) {
    bytes.extend_from_slice(&u64::try_from(count).expect("个数装得进 u64").to_le_bytes());
}

/// 一个扇区的字节摘要：SHA-256 的前 16 个字节（只在同一个基础镜像上比扇区等不等）。
pub(super) type SectorDigest = [u8; 16];

fn sector_digest(sector_bytes: &[u8]) -> SectorDigest {
    sha256_digest(sector_bytes)[..16]
        .try_into()
        .expect("SHA-256 摘要 32 字节，前 16 个总在")
}

/// 枚举写表里一次写盖住的扇区：普通写逐扇区记字节摘要；整段清零记范围（清零把那一段里写过的扇区**删掉**，
/// `singlefs_verification::memory_pool::SparseDevice::zero_fill`：读出来是 0，journal 记录槽提示也不再报它）。
enum SectorsOfAWrite {
    Written(Vec<(u32, u64, SectorDigest)>),
    Zeroed {
        device: singlefs_core::address::DeviceIdentity,
        first_sector: u64,
        end_sector: u64,
    },
}

/// 一个状态上一个扇区相对基础镜像怎么变了：被写成这些字节，或被清零删掉（没写过）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SectorChange {
    WrittenTo(SectorDigest),
    Unwritten,
}

/// 一个状态的镜像相对基础镜像的改动，按（盘、扇区）排；没在里面的扇区照基础镜像。
pub(super) type ImageOverlay = BTreeMap<(u32, u64), SectorChange>;

/// 持久集合开头 `writes_in_the_prefix` 次写全持久时叠出来的改动（[`ImageOverlayOverAPrefix`] 的前缀那一截，几个相邻状态共用一份）。
pub(super) struct ImageOverlayOfAPrefix {
    writes_in_the_prefix: usize,
    pub(super) overlay: ImageOverlay,
}

/// 一个状态的镜像相对基础镜像的改动，分两截：前缀那一截（[`ImageOverlayOfAPrefix`]）与这个状态自己那一截（前缀之后这个状态持久了的写叠出来的，
/// 两截都有的扇区取自己那一截）。合起来与把这个状态持久了的写从头逐次叠出来的那一份逐项相同（[`ReadSetWitnessDigester::overlay_of_the_state`]）。
pub(super) struct ImageOverlayOverAPrefix {
    pub(super) prefix: std::rc::Rc<ImageOverlayOfAPrefix>,
    pub(super) own: ImageOverlay,
}

impl ImageOverlayOverAPrefix {
    /// 盘 `device` 上 `sectors` 那几个扇区的改动，按扇区号升序：自己那一截有的取它，没有的取前缀那一截的。
    fn changes_in(
        &self,
        device: u32,
        sectors: Range<u64>,
    ) -> impl Iterator<Item = ((u32, u64), SectorChange)> + '_ {
        let keys = (device, sectors.start)..(device, sectors.end);
        SortedMergeWithTheLaterWinning::of(
            self.prefix
                .overlay
                .range(keys.clone())
                .map(|(position, change)| (*position, *change)),
            self.own
                .range(keys)
                .map(|(position, change)| (*position, *change)),
        )
        .into_sorted_merge_iterator()
    }
}

/// 两串按键升序、各自键不重的（键，值）并成一串按键升序的：两串都有的键只出一次，取后一串（`later`）的值。
struct SortedMergeWithTheLaterWinning<Earlier: Iterator, Later: Iterator> {
    earlier: std::iter::Peekable<Earlier>,
    later: std::iter::Peekable<Later>,
}

impl<Earlier: Iterator, Later: Iterator> SortedMergeWithTheLaterWinning<Earlier, Later> {
    fn of(earlier: Earlier, later: Later) -> Self {
        Self {
            earlier: earlier.peekable(),
            later: later.peekable(),
        }
    }
}

impl<Key, Value, Earlier, Later> SortedMergeWithTheLaterWinning<Earlier, Later>
where
    Key: Ord,
    Earlier: Iterator<Item = (Key, Value)>,
    Later: Iterator<Item = (Key, Value)>,
{
    fn into_sorted_merge_iterator(mut self) -> impl Iterator<Item = (Key, Value)> {
        std::iter::from_fn(move || self.take_the_next_in_key_order())
    }

    fn take_the_next_in_key_order(&mut self) -> Option<(Key, Value)> {
        let order = match (self.earlier.peek(), self.later.peek()) {
            (None, None) => return None,
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (Some((earlier_key, _)), Some((later_key, _))) => earlier_key.cmp(later_key),
        };
        match order {
            std::cmp::Ordering::Less => self.earlier.next(),
            std::cmp::Ordering::Greater => self.later.next(),
            std::cmp::Ordering::Equal => {
                self.earlier.next();
                self.later.next()
            }
        }
    }
}

/// 一条流的崩溃镜像算读集见证要的几样（录入段每条流现算一次）：基础镜像、枚举写表每次写盖住的扇区与每个扇区的摘要、基础镜像每个写过的扇区的摘要
/// （按盘与扇区排，见证摘要按段取一截）。
/// 两个状态在同样几段上的见证摘要相同 ⟺ 那几段里写过的扇区集合相同、每个写过的扇区的字节相同（同一个基础镜像上；扇区摘要取 128 位）。
/// 写没写过也进摘要：journal 记录槽提示的答案由哪些扇区写过定，一个写满 0 的扇区与一个没写过的扇区读出来相同、提示却不同。
pub(super) struct ReadSetWitnessDigester<'flow> {
    pub(super) base: &'flow MemoryPool,
    sectors_of_each_write: Vec<SectorsOfAWrite>,
    base_digest_of_each_written_sector: BTreeMap<(u32, u64), SectorDigest>,
}

impl<'flow> ReadSetWitnessDigester<'flow> {
    pub(super) fn of(base: &'flow MemoryPool, enumerated_writes: &[RetainedWrite]) -> Self {
        let sector_bytes =
            usize::try_from(crate::memory_pool::SECTOR_BYTES).expect("扇区字节数装得进 usize");
        let mut base_digest_of_each_written_sector = BTreeMap::new();
        for (identity, device) in &base.devices {
            for (sector, bytes) in device.written_sectors() {
                base_digest_of_each_written_sector
                    .insert((identity.0, *sector), sector_digest(bytes));
            }
        }
        let sectors_of_each_write = enumerated_writes
            .iter()
            .map(|write| {
                let first_sector = write.offset.0 / crate::memory_pool::SECTOR_BYTES;
                match &write.contents {
                    WrittenContents::Bytes(bytes) => SectorsOfAWrite::Written(
                        bytes
                            .chunks(sector_bytes)
                            .enumerate()
                            .map(|(sector_index, chunk)| {
                                let sector = first_sector
                                    + u64::try_from(sector_index).expect("扇区下标装得进 u64");
                                (write.device.0, sector, sector_digest(chunk))
                            })
                            .collect(),
                    ),
                    WrittenContents::Zeros { length } => SectorsOfAWrite::Zeroed {
                        device: write.device,
                        first_sector,
                        end_sector: first_sector
                            + length.div_ceil(crate::memory_pool::SECTOR_BYTES),
                    },
                }
            })
            .collect();
        Self {
            base,
            sectors_of_each_write,
            base_digest_of_each_written_sector,
        }
    }

    /// 一个状态（持久集合按枚举写表给）的镜像相对基础镜像的改动（[`ImageOverlayOverAPrefix`]）：持久集合开头连着全持久的那几次写是前缀；
    /// `prefix_kept` 里留着的那一份前缀不长于它就照用（一块里相邻的状态多半共用同一份，前缀那几次写不逐态叠），长于它（这个状态在前缀里有写没持久）
    /// 就照这个状态的前缀重叠一份、换掉留着的；前缀之后这个状态持久了的写叠成自己那一截。与 [`Self::overlay_of`] 从头叠的那一份合起来逐项相同：
    /// 清零那一次把前面叠过的扇区标成没写过时，前面叠过的是前缀那一截与自己那一截已有的扇区之并。
    ///
    /// # Panics
    /// 持久集合与枚举写表不一样长。
    pub(super) fn overlay_of_the_state(
        &self,
        persisted: &[bool],
        prefix_kept: &std::cell::RefCell<Option<std::rc::Rc<ImageOverlayOfAPrefix>>>,
    ) -> ImageOverlayOverAPrefix {
        assert_eq!(
            persisted.len(),
            self.sectors_of_each_write.len(),
            "持久集合与枚举写表逐条对应"
        );
        let writes_persisted_from_the_start = persisted
            .iter()
            .take_while(|is_persisted| **is_persisted)
            .count();
        let prefix = {
            let mut kept = prefix_kept.borrow_mut();
            match kept.as_ref() {
                Some(prefix) if prefix.writes_in_the_prefix <= writes_persisted_from_the_start => {
                    std::rc::Rc::clone(prefix)
                }
                Some(_) | None => {
                    let prefix = std::rc::Rc::new(ImageOverlayOfAPrefix {
                        writes_in_the_prefix: writes_persisted_from_the_start,
                        overlay: self.persisted_writes_laid_over(
                            0..writes_persisted_from_the_start,
                            persisted,
                            None,
                        ),
                    });
                    *kept = Some(std::rc::Rc::clone(&prefix));
                    prefix
                }
            }
        };
        let own = self.persisted_writes_laid_over(
            prefix.writes_in_the_prefix..persisted.len(),
            persisted,
            Some(&prefix.overlay),
        );
        ImageOverlayOverAPrefix { prefix, own }
    }

    /// 枚举写表里 `writes` 那几次写中持久了的，按写的次序叠成的改动（后写盖前写），叠在 `earlier`（前面那几次写叠出来的，没有就是空的）之上：
    /// 普通写把它盖住的扇区记成写成的字节；清零把那一段里基础镜像写过的、前面叠过的（`earlier` 里的与这一次之前已经叠进来的）扇区都记成没写过。
    /// 交回的只是 `writes` 这几次叠出来的那一截，`earlier` 不拷进来。
    fn persisted_writes_laid_over(
        &self,
        writes: Range<usize>,
        persisted: &[bool],
        earlier: Option<&ImageOverlay>,
    ) -> ImageOverlay {
        let mut overlay: ImageOverlay = BTreeMap::new();
        for (sectors, is_persisted) in self.sectors_of_each_write[writes.clone()]
            .iter()
            .zip(&persisted[writes])
        {
            if !*is_persisted {
                continue;
            }
            match sectors {
                SectorsOfAWrite::Written(sectors) => {
                    for (device, sector, digest) in sectors {
                        overlay.insert((*device, *sector), SectorChange::WrittenTo(*digest));
                    }
                }
                SectorsOfAWrite::Zeroed {
                    device,
                    first_sector,
                    end_sector,
                } => {
                    let written_in_the_base =
                        self.base
                            .devices
                            .get(device)
                            .map_or_else(Vec::new, |sparse| {
                                sparse.written_sectors_in(
                                    singlefs_core::address::DeviceOffsetInBytes(
                                        first_sector * crate::memory_pool::SECTOR_BYTES,
                                    ),
                                    (end_sector - first_sector) * crate::memory_pool::SECTOR_BYTES,
                                )
                            });
                    let zeroed_range = (device.0, *first_sector)..(device.0, *end_sector);
                    let written_by_earlier_writes: Vec<(u32, u64)> = earlier
                        .into_iter()
                        .flat_map(|earlier_overlay| earlier_overlay.range(zeroed_range.clone()))
                        .chain(overlay.range(zeroed_range.clone()))
                        .map(|(position, _)| *position)
                        .collect();
                    for sector in written_in_the_base {
                        overlay.insert((device.0, sector), SectorChange::Unwritten);
                    }
                    for position in written_by_earlier_writes {
                        overlay.insert(position, SectorChange::Unwritten);
                    }
                }
            }
        }
        overlay
    }

    /// 一个状态的镜像在这几段上的见证摘要：每一段写盘、起止扇区，接上这一段里每个写过的扇区（基础镜像的叠上这个状态的改动）的扇区号与字节摘要，
    /// 按扇区号排；没写过的扇区不写（写没写过因此也在摘要里）。基础镜像那一截与改动那一截按扇区号并着走，不另建一份这一段的扇区表。
    pub(super) fn witness_digest(
        &self,
        overlay: &ImageOverlayOverAPrefix,
        ranges: &[ReadRangeOnTheCrashImage],
    ) -> [u8; DIGEST_BYTES] {
        let mut message = b"singlefs read set witness 1\n".to_vec();
        for range in ranges {
            message.extend_from_slice(&range.device.to_le_bytes());
            message.extend_from_slice(&range.first_sector.to_le_bytes());
            message.extend_from_slice(&range.end_sector.to_le_bytes());
            let written_in_the_base = self
                .base_digest_of_each_written_sector
                .range((range.device, range.first_sector)..(range.device, range.end_sector))
                .map(|(position, digest)| (*position, SectorChange::WrittenTo(*digest)));
            let changes_of_the_state =
                overlay.changes_in(range.device, range.first_sector..range.end_sector);
            for ((_, sector), change) in
                SortedMergeWithTheLaterWinning::of(written_in_the_base, changes_of_the_state)
                    .into_sorted_merge_iterator()
            {
                match change {
                    SectorChange::WrittenTo(digest) => {
                        message.extend_from_slice(&sector.to_le_bytes());
                        message.extend_from_slice(&digest);
                    }
                    SectorChange::Unwritten => {}
                }
            }
            message.extend_from_slice(b"\nend of range\n");
        }
        sha256_digest(&message)
    }

    /// 参照（对拍 [`Self::overlay_of_the_state`] 用）：一个状态（持久集合按枚举写表给）的镜像相对基础镜像的改动，从头逐次叠这个状态持久了的写，
    /// 后写盖前写；清零把那一段里基础镜像写过的与前面写过的扇区都标成没写过。
    ///
    /// # Panics
    /// 持久集合与枚举写表不一样长。
    #[cfg(test)]
    fn overlay_of(&self, persisted: &[bool]) -> ImageOverlay {
        assert_eq!(
            persisted.len(),
            self.sectors_of_each_write.len(),
            "持久集合与枚举写表逐条对应"
        );
        let mut overlay: ImageOverlay = BTreeMap::new();
        for (sectors, is_persisted) in self.sectors_of_each_write.iter().zip(persisted) {
            if !*is_persisted {
                continue;
            }
            match sectors {
                SectorsOfAWrite::Written(sectors) => {
                    for (device, sector, digest) in sectors {
                        overlay.insert((*device, *sector), SectorChange::WrittenTo(*digest));
                    }
                }
                SectorsOfAWrite::Zeroed {
                    device,
                    first_sector,
                    end_sector,
                } => {
                    let written_in_the_base =
                        self.base
                            .devices
                            .get(device)
                            .map_or_else(Vec::new, |sparse| {
                                sparse.written_sectors_in(
                                    singlefs_core::address::DeviceOffsetInBytes(
                                        first_sector * crate::memory_pool::SECTOR_BYTES,
                                    ),
                                    (end_sector - first_sector) * crate::memory_pool::SECTOR_BYTES,
                                )
                            });
                    let written_by_earlier_writes: Vec<(u32, u64)> = overlay
                        .range((device.0, *first_sector)..(device.0, *end_sector))
                        .map(|(position, _)| *position)
                        .collect();
                    for sector in written_in_the_base {
                        overlay.insert((device.0, sector), SectorChange::Unwritten);
                    }
                    for position in written_by_earlier_writes {
                        overlay.insert(position, SectorChange::Unwritten);
                    }
                }
            }
        }
        overlay
    }

    /// 参照（对拍 [`Self::witness_digest`] 用）：同一个见证摘要，照整份改动（[`Self::overlay_of`]）先收这一段里基础镜像写过的扇区、再叠改动算。
    #[cfg(test)]
    fn witness_digest_of_the_whole_overlay(
        &self,
        overlay: &ImageOverlay,
        ranges: &[ReadRangeOnTheCrashImage],
    ) -> [u8; DIGEST_BYTES] {
        let mut message = b"singlefs read set witness 1\n".to_vec();
        for range in ranges {
            message.extend_from_slice(&range.device.to_le_bytes());
            message.extend_from_slice(&range.first_sector.to_le_bytes());
            message.extend_from_slice(&range.end_sector.to_le_bytes());
            let mut written: BTreeMap<u64, SectorDigest> = self
                .base
                .devices
                .get(&singlefs_core::address::DeviceIdentity(range.device))
                .map(|sparse| {
                    sparse
                        .written_sectors()
                        .range(range.first_sector..range.end_sector)
                        .map(|(sector, _)| {
                            (
                                *sector,
                                *self
                                    .base_digest_of_each_written_sector
                                    .get(&(range.device, *sector))
                                    .expect("基础镜像每个写过的扇区的摘要建摘要器时都算过"),
                            )
                        })
                        .collect()
                })
                .unwrap_or_default();
            for ((_, sector), change) in
                overlay.range((range.device, range.first_sector)..(range.device, range.end_sector))
            {
                match change {
                    SectorChange::WrittenTo(digest) => {
                        written.insert(*sector, *digest);
                    }
                    SectorChange::Unwritten => {
                        written.remove(sector);
                    }
                }
            }
            for (sector, digest) in &written {
                message.extend_from_slice(&sector.to_le_bytes());
                message.extend_from_slice(digest);
            }
            message.extend_from_slice(b"\nend of range\n");
        }
        sha256_digest(&message)
    }
}

/// 观察者真跑一次读过的段（归一之后，[`normalized_read_ranges`]）与这份段的摘要（它的字节 [`observed_read_set_bytes`] 的 SHA-256）。
/// 库里同一组段只存一行读集行（键就是这个摘要，`verdict_store` 的读集行），见证行按摘要引它；内存里一个桶的同一组段只留一份
/// （[`ReadSetWitnessIndex`] 每组一份），几条见证拿的是同一份段的共享引用。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ObservedReadSet {
    pub(super) ranges: std::sync::Arc<[ReadRangeOnTheCrashImage]>,
    pub(super) digest: [u8; DIGEST_BYTES],
}

const OBSERVED_READ_SET_MAGIC: &[u8; 8] = b"SFOBRDS1";

/// 一组段的字节：标记、段数，再逐段盘（4 字节）、起扇区、止扇区（各 8 字节），整数小端。读集行的值就是它。
pub(super) fn observed_read_set_bytes(ranges: &[ReadRangeOnTheCrashImage]) -> Vec<u8> {
    let mut bytes = OBSERVED_READ_SET_MAGIC.to_vec();
    put_item_count(&mut bytes, ranges.len());
    for range in ranges {
        bytes.extend_from_slice(&range.device.to_le_bytes());
        bytes.extend_from_slice(&range.first_sector.to_le_bytes());
        bytes.extend_from_slice(&range.end_sector.to_le_bytes());
    }
    bytes
}

impl ObservedReadSet {
    pub(super) fn of_the_ranges(ranges: Vec<ReadRangeOnTheCrashImage>) -> Self {
        let digest = sha256_digest(&observed_read_set_bytes(&ranges));
        Self {
            ranges: std::sync::Arc::from(ranges),
            digest,
        }
    }

    /// 从库里读回来的读集行：字节的摘要与键相同、解得出、解完没有剩，才认；否则 `None`。
    pub(super) fn of_the_stored_bytes(digest: &[u8; DIGEST_BYTES], bytes: &[u8]) -> Option<Self> {
        if sha256_digest(bytes) != *digest {
            return None;
        }
        let mut reader = RecordedBytesReader { bytes, at: 0 };
        if reader.take(OBSERVED_READ_SET_MAGIC.len())? != OBSERVED_READ_SET_MAGIC {
            return None;
        }
        let mut ranges = Vec::new();
        for _ in 0..reader.count()? {
            ranges.push(ReadRangeOnTheCrashImage {
                device: reader.u32()?,
                first_sector: reader.u64()?,
                end_sector: reader.u64()?,
            });
        }
        (reader.at == bytes.len()).then(|| Self {
            ranges: std::sync::Arc::from(ranges),
            digest: *digest,
        })
    }
}

/// 一个桶里的一个读集见证：观察者在这个桶里真跑的那一次读过的段（几条见证读同一组段时共用一份）、那几段在那张镜像上的见证摘要、
/// 它看出的那张镜像在库里的键。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ReadSetWitness {
    pub(super) read_set: ObservedReadSet,
    pub(super) digest: [u8; DIGEST_BYTES],
    pub(super) observed_image_key: [u8; DIGEST_BYTES],
}

impl ReadSetWitness {
    /// 库里这条见证那一行（`verdict_store` 的读集见证行）：段不在行里，只引读集摘要。
    pub(super) fn row(&self) -> ObserverReadSetWitnessRow {
        ObserverReadSetWitnessRow {
            witness_digest: self.digest,
            read_set_digest: self.read_set.digest,
            observed_image_key: self.observed_image_key,
        }
    }
}

/// 一个桶里读集组的位置（[`ReadSetWitnessIndex`] 里按读集第一次出现的次序编；组只添不删，同一趟里位置不变）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ReadSetGroupPosition(pub(super) usize);

/// 一个桶里读过同一组段的见证：那几段（这组见证共用的一份），与这组里每条见证的摘要到它看出的镜像键。
#[derive(Clone, Debug, PartialEq, Eq)]
struct WitnessesOfOneReadSet {
    ranges: std::sync::Arc<[ReadRangeOnTheCrashImage]>,
    observed_image_key_of_each_digest:
        std::collections::HashMap<[u8; DIGEST_BYTES], [u8; DIGEST_BYTES]>,
}

/// 一个桶的读集见证按读集分组的索引：匹配一个状态时每组读集只算一次见证摘要、拿摘要查表，摘要次数是桶里不同读集的组数、不是见证条数。
/// 段只在组里存一份，见证只占组表里的一项（见证摘要到镜像键）：常驻是「组数 × 一组的段 + 见证条数 × 两份摘要」，添一条见证不碰别的见证。
/// 库里一条见证一行（`verdict_store` 的读集见证行），读回时按见证摘要的次序重建（[`Self::of_the_stored_witnesses`]），组的次序因此可能与录的时候不同，
/// 只影响先查哪一组（[`Self::find_the_matching_witness`]）。
/// 不变量：`digests_in_the_bucket` 是全部见证摘要的集合，每个摘要恰在一组的表里；`group_of_each_read_set` 是每组读集摘要到它的位置。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ReadSetWitnessIndex {
    groups: Vec<WitnessesOfOneReadSet>,
    group_of_each_read_set: std::collections::HashMap<[u8; DIGEST_BYTES], ReadSetGroupPosition>,
    digests_in_the_bucket: std::collections::HashSet<[u8; DIGEST_BYTES]>,
}

/// 往索引里添一条见证的结果：添进了这个桶里已有的那组读集（known）、添进了这个桶里头一回出现的一组读集（new），
/// 或桶里已有同一摘要的见证、没添（与另一个线程刚添的那一条看出的是同一张镜像）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ReadSetWitnessAddition {
    AddedToAKnownReadSet,
    AddedWithANewReadSet,
    AlreadyInTheBucket,
}

/// 一个桶的见证索引连同护着它的锁：这一趟里每个桶只有这一份，读者（匹配一个状态）拿读锁、添见证的拿写锁原地添，
/// 读者看到的是添之前或添之后的整份索引，谁都不另拷一份。
pub(super) type ReadSetWitnessIndexOfABucket = std::sync::RwLock<ReadSetWitnessIndex>;

/// 一个状态对上的见证：在哪一组读集里对上的（下一个状态先查这一组）、那条见证看出的镜像键。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct MatchedReadSetWitness {
    pub(super) group: ReadSetGroupPosition,
    pub(super) observed_image_key: [u8; DIGEST_BYTES],
}

impl ReadSetWitnessIndex {
    pub(super) fn empty() -> Self {
        Self {
            groups: Vec::new(),
            group_of_each_read_set: std::collections::HashMap::new(),
            digests_in_the_bucket: std::collections::HashSet::new(),
        }
    }

    /// 从库里读回来的见证建索引，按给的次序逐条添；同一摘要出现几次的只认第一条。
    pub(super) fn of_the_stored_witnesses(stored_witnesses: Vec<ReadSetWitness>) -> Self {
        let mut index = Self::empty();
        for witness in stored_witnesses {
            index.add(witness);
        }
        index
    }

    /// 添一条见证：桶里已有同一摘要的见证就不添（与 [`ObserverRecordingContext::store_the_image_with_its_witness`] 的判同一条）；
    /// 读集已在桶里的记进那一组的表、段不再存，头一回出现的读集新开一组（排在最后，存下这一份段）。只动这一条见证那一项与它那一组，不碰别的见证。
    pub(super) fn add(&mut self, witness: ReadSetWitness) -> ReadSetWitnessAddition {
        if self.digests_in_the_bucket.contains(&witness.digest) {
            return ReadSetWitnessAddition::AlreadyInTheBucket;
        }
        self.digests_in_the_bucket.insert(witness.digest);
        let (position, addition) = match self.group_of_each_read_set.get(&witness.read_set.digest) {
            Some(position) => (*position, ReadSetWitnessAddition::AddedToAKnownReadSet),
            None => {
                let position = ReadSetGroupPosition(self.groups.len());
                self.groups.push(WitnessesOfOneReadSet {
                    ranges: witness.read_set.ranges,
                    observed_image_key_of_each_digest: std::collections::HashMap::new(),
                });
                self.group_of_each_read_set
                    .insert(witness.read_set.digest, position);
                (position, ReadSetWitnessAddition::AddedWithANewReadSet)
            }
        };
        self.groups[position.0]
            .observed_image_key_of_each_digest
            .insert(witness.digest, witness.observed_image_key);
        addition
    }

    pub(super) fn holds_the_witness_digest(&self, witness_digest: &[u8; DIGEST_BYTES]) -> bool {
        self.digests_in_the_bucket.contains(witness_digest)
    }

    pub(super) fn holds_the_read_set(&self, read_set_digest: &[u8; DIGEST_BYTES]) -> bool {
        self.group_of_each_read_set.contains_key(read_set_digest)
    }

    pub(super) fn witness_count(&self) -> usize {
        self.digests_in_the_bucket.len()
    }

    pub(super) fn read_set_group_count(&self) -> usize {
        self.groups.len()
    }

    /// 各组存的段数之和：段只在组里存一份，这个数是组数乘一组的段数，与见证条数无关。
    #[cfg(test)]
    fn ranges_held(&self) -> usize {
        self.groups.iter().map(|group| group.ranges.len()).sum()
    }

    /// 找一条对得上这个状态的见证：每组读集只调一次 `witness_digest_of_the_state`（这个状态在那几段上的见证摘要），拿摘要查那一组的表；
    /// `group_to_try_first` 那一组先查（上一个状态对上的那一组），其余按组的次序，对上就停。一条都对不上交回 `None`，这时每组恰好调一次。
    /// 同一组里一个摘要至多一条见证（[`Self::add`]），组内交回的就是按添的次序逐条扫会找到的那一条。一个状态对得上两组里的见证，要观察者在
    /// 两张镜像上读的段不同、而这个状态在两组段上又各与其中一张逐字节相同：观察者读哪几段只由它读到的字节定时不会有（去重本身就靠这一条）；
    /// 真有的话，先查的那一组赢，交回的可能与按添的次序逐条扫找到的那一条不同。
    pub(super) fn find_the_matching_witness(
        &self,
        group_to_try_first: Option<ReadSetGroupPosition>,
        mut witness_digest_of_the_state: impl FnMut(&[ReadRangeOnTheCrashImage]) -> [u8; DIGEST_BYTES],
    ) -> Option<MatchedReadSetWitness> {
        let preferred_group = group_to_try_first.filter(|position| position.0 < self.groups.len());
        let order = preferred_group.into_iter().chain(
            (0..self.groups.len())
                .map(ReadSetGroupPosition)
                .filter(|position| Some(*position) != preferred_group),
        );
        for position in order {
            let group = &self.groups[position.0];
            let digest = witness_digest_of_the_state(&group.ranges);
            if let Some(observed_image_key) = group.observed_image_key_of_each_digest.get(&digest) {
                return Some(MatchedReadSetWitness {
                    group: position,
                    observed_image_key: *observed_image_key,
                });
            }
        }
        None
    }
}

/// 见过的内容各自的 SHA-256：再来一份逐字节相同的内容，交回记下的那一个摘要、不再算；值与 [`sha256_digest`] 现算的逐字节相同。
/// 认「相同」只凭逐字节比（先比长度），不凭别的指纹。记下的份数与字节合计各有上限，满了之后没见过的内容照算、不再记。
struct DigestsOfContentsSeenBefore {
    seen: std::sync::RwLock<Vec<ContentSeenBeforeWithItsDigest>>,
}

/// 记下的一份内容与它的 SHA-256。
type ContentSeenBeforeWithItsDigest = (Box<[u8]>, [u8; DIGEST_BYTES]);

impl DigestsOfContentsSeenBefore {
    /// 最多记几份内容。
    const MAXIMUM_CONTENTS_KEPT: usize = 64;
    /// 记下的内容合计最多几个字节。
    const MAXIMUM_BYTES_KEPT: usize = 256 * 1024 * 1024;

    const fn empty() -> Self {
        Self {
            seen: std::sync::RwLock::new(Vec::new()),
        }
    }

    /// `content` 的 SHA-256：记下的里面有逐字节相同的一份就取它的，没有就现算、放得下就记下。
    fn digest_of(&self, content: &[u8]) -> [u8; DIGEST_BYTES] {
        let digest_kept_for = |seen: &[ContentSeenBeforeWithItsDigest]| {
            seen.iter()
                .find(|(kept, _)| kept.len() == content.len() && **kept == *content)
                .map(|(_, digest)| *digest)
        };
        if let Some(digest) = digest_kept_for(
            &self
                .seen
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        ) {
            return digest;
        }
        let digest = sha256_digest(content);
        let mut seen = self
            .seen
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let bytes_kept: usize = seen.iter().map(|(kept, _)| kept.len()).sum();
        if digest_kept_for(&seen).is_none()
            && seen.len() < Self::MAXIMUM_CONTENTS_KEPT
            && bytes_kept + content.len() <= Self::MAXIMUM_BYTES_KEPT
        {
            seen.push((content.into(), digest));
        }
        digest
    }

    #[cfg(test)]
    fn contents_kept(&self) -> usize {
        self.seen
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len()
    }
}

/// 恢复报告的摘要（桶键的一截）：每个字段都进，一个不省——结局（落到哪一版、读回的文件内容的摘要、走不下去的原因）、实际走的那条根、
/// journal 扫描报告的六个数、转查映射的次数、系统配置两槽都不可择而被跳过的盘。走不下去的原因按它的 `Debug` 文本进（成员多、改了就是新桶，
/// 只会少合、不会多合）。
pub(super) fn recovery_report_digest(report: &RecoveryReport) -> [u8; DIGEST_BYTES] {
    let put_root = |message: &mut Vec<u8>, root: &(InstanceGeneration, CheckpointTxg)| {
        message.extend_from_slice(&root.0 .0.to_le_bytes());
        message.extend_from_slice(&root.1 .0.to_le_bytes());
    };
    let put_optional_root =
        |message: &mut Vec<u8>, root: Option<&(InstanceGeneration, CheckpointTxg)>| match root {
            None => message.push(0),
            Some(root) => {
                message.push(1);
                put_root(message, root);
            }
        };
    let mut message = b"singlefs recovery report 1\n".to_vec();
    match &report.outcome {
        RecoveryOutcome::NoFile { root } => {
            message.push(0);
            put_root(&mut message, root);
        }
        RecoveryOutcome::FileRead { root, content } => {
            message.push(1);
            put_root(&mut message, root);
            // 一条流的状态读回的文件内容只有逐版那几份：逐字节相同的那一份只算一次摘要，值与每次现算的相同
            static DIGESTS_OF_THE_FILE_CONTENTS_READ_BACK: DigestsOfContentsSeenBefore =
                DigestsOfContentsSeenBefore::empty();
            message.extend_from_slice(&DIGESTS_OF_THE_FILE_CONTENTS_READ_BACK.digest_of(content));
        }
        RecoveryOutcome::Failed { root, failure } => {
            message.push(2);
            put_optional_root(&mut message, root.as_ref());
            put_text(&mut message, &format!("{failure:?}"));
        }
    }
    put_optional_root(&mut message, report.effective_root.as_ref());
    for number in [
        report.journal.valid_records,
        report.journal.above_water,
        report.journal.prefix_applied,
        report.journal.verification_passed,
        report.journal.verification_failed,
        report.mapping_fallbacks,
    ] {
        message.extend_from_slice(&u64::try_from(number).expect("计数装得进 u64").to_le_bytes());
    }
    message.extend_from_slice(&report.journal.maximum_applied_transaction.to_le_bytes());
    put_item_count(
        &mut message,
        report
            .devices_without_a_mountable_system_configuration_slot
            .len(),
    );
    for device in &report.devices_without_a_mountable_system_configuration_slot {
        message.extend_from_slice(&device.0.to_le_bytes());
    }
    sha256_digest(&message)
}

/// 交回镜像的观察者去重的桶键：观察者名、观察者代码那一把版本（另读的输入那一格全零）、另读的输入的摘要、恢复报告的摘要。
/// 另读的输入不同的两个观察者看同一张镜像不合并；观察者代码改了，键跟着换、旧的结果留作历史。
pub(super) fn observer_bucket_key(
    observer: &ObserverOnTheNodes<'_>,
    versions: &ObserverVersions,
    recovery_report_digest: &[u8; DIGEST_BYTES],
) -> [u8; DIGEST_BYTES] {
    let mut message = b"singlefs observer bucket key 1\n".to_vec();
    message.extend_from_slice(observer.observer_name.as_str().as_bytes());
    message.push(b'\n');
    message.extend_from_slice(&versions.observer_code.0);
    message.extend_from_slice(&observer.extra_inputs_digest);
    message.extend_from_slice(recovery_report_digest);
    sha256_digest(&message)
}

/// 一个见证看出的那张镜像在库里的键：桶键接见证摘要。
pub(super) fn observed_image_key_of_the_witness(
    bucket_key: &[u8; DIGEST_BYTES],
    witness_digest: &[u8; DIGEST_BYTES],
) -> [u8; DIGEST_BYTES] {
    let mut message = b"singlefs observed image key 2\n".to_vec();
    message.extend_from_slice(bucket_key);
    message.extend_from_slice(witness_digest);
    sha256_digest(&message)
}

/// 不去重时每个状态各一个键（剪枝自证关掉去重，与观察者 panic 没交回读过的段那一次）：桶键接块键与块内序号。
pub(super) fn observed_image_key_without_deduplication(
    bucket_key: &[u8; DIGEST_BYTES],
    block_key: &VerdictBlockKey,
    offset_in_block: u64,
) -> [u8; DIGEST_BYTES] {
    let mut message = b"singlefs observed image key without deduplication 2\n".to_vec();
    message.extend_from_slice(bucket_key);
    message.extend_from_slice(&block_key.node_input_digest.0);
    message.extend_from_slice(&block_key.enumeration_plan_hash.0);
    message.extend_from_slice(&block_key.block_start.0.to_le_bytes());
    message.extend_from_slice(&offset_in_block.to_le_bytes());
    sha256_digest(&message)
}

/// 两份池的每个扇区字节相同（写过的全零扇区与没写过的算相同）、盘的身份与大小相同。逐扇区借着比，不拷字节。
pub(super) fn pools_hold_the_same_bytes(left: &MemoryPool, right: &MemoryPool) -> bool {
    fn nonzero_sectors(
        device: &crate::memory_pool::SparseDevice,
    ) -> impl Iterator<Item = (&u64, &Vec<u8>)> {
        device
            .written_sectors()
            .iter()
            .filter(|(_, bytes)| bytes.iter().any(|byte| *byte != 0))
    }
    left.device_size_in_bytes == right.device_size_in_bytes
        && left.devices.len() == right.devices.len()
        && left.devices.iter().zip(&right.devices).all(
            |((left_identity, left_device), (right_identity, right_device))| {
                left_identity == right_identity
                    && nonzero_sectors(left_device).eq(nonzero_sectors(right_device))
            },
        )
}

/// 崩溃镜像落成一个池：基础镜像叠上持久了的写（与 `crate::writable_mount_phases::materialized_crash_image` 同一条规则）。
pub(super) fn pool_of_the_crash_image(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    persisted: &[bool],
) -> MemoryPool {
    let mut pool = base.clone();
    for (write, is_persisted) in writes.iter().zip(persisted) {
        if *is_persisted {
            pool.apply_writes(std::slice::from_ref(write));
        }
    }
    pool
}

#[cfg(test)]
mod tests;
