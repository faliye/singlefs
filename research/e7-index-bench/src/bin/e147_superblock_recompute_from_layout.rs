//! E147：超级块字段表按第一个事务字节表重算——D22（单元原子性怎么合成） 已定项 9 挡路条 ③ 那两句「四格数字不可引用」之后的可引用数。
//!
//! first-txn-layout.md 第一节的超级块预想字段表逐行抄进 `SUPERBLOCK_ROWS`，算 512 字节槽的余量与撕裂态，
//! 再给挡路条 ② 还没定的三格各算一个变体。只报数不判输赢；判据与失败条款在 `research/prompts/e147-preregistration.md`。

use e7_index_bench::Emitter;

/// D22 2026-09-10 用户定案：槽宽 = 探测到的 physical_block_size，本机 512。
const SLOT_BYTES: u64 = 512;
/// E115 给主密钥槽取的最小可用假设值。
const KEY_SLOT_INLINE_BYTES: u64 = 80;
/// 变体 `kdf_identifier_4` 把 KDF 标识从 2 抬到 4。
const KDF_IDENTIFIER_WIDE_BYTES: u64 = 4;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Segment {
    Bootstrap,
    EncryptionReserve,
    Geometry,
    Tunable,
    Runtime,
}

impl Segment {
    fn name(self) -> &'static str {
        match self {
            Segment::Bootstrap => "bootstrap",
            Segment::EncryptionReserve => "encryption_reserve",
            Segment::Geometry => "geometry",
            Segment::Tunable => "tunable",
            Segment::Runtime => "runtime",
        }
    }
}

const SEGMENTS: [Segment; 5] = [Segment::Bootstrap, Segment::EncryptionReserve, Segment::Geometry, Segment::Tunable, Segment::Runtime];

/// first-txn-layout.md「超级块预想字段表」逐行：段、字段、宽。
const SUPERBLOCK_ROWS: [(Segment, &str, u64); 39] = [
    (Segment::Bootstrap, "magic", 4),
    (Segment::Bootstrap, "format_version", 2),
    (Segment::Bootstrap, "feature_bits", 96),
    (Segment::Bootstrap, "fsid", 16),
    (Segment::Bootstrap, "this_device", 4),
    (Segment::Bootstrap, "device_count", 4),
    (Segment::Bootstrap, "slot_generation", 8),
    (Segment::Bootstrap, "slot_checksum", 32),
    (Segment::EncryptionReserve, "superblock_mac", 16),
    (Segment::EncryptionReserve, "nonce_watermark", 12),
    (Segment::EncryptionReserve, "kdf_identifier", 2),
    (Segment::EncryptionReserve, "encryption_kind", 1),
    (Segment::EncryptionReserve, "mac_length", 1),
    (Segment::Geometry, "node_bytes", 4),
    (Segment::Geometry, "unit_bytes", 4),
    (Segment::Geometry, "placement_grain", 4),
    (Segment::Geometry, "location_entry_bytes", 4),
    (Segment::Geometry, "journal_ring_start_slot", 8),
    (Segment::Geometry, "journal_ring_bytes", 8),
    (Segment::Geometry, "journal_record_bytes", 4),
    (Segment::Geometry, "journal_in_flight_limit", 4),
    (Segment::Geometry, "journal_worst_occupancy", 8),
    (Segment::Geometry, "journal_safety_factor", 4),
    (Segment::Geometry, "ring_regions", 1),
    (Segment::Geometry, "ring_slots_per_region", 1),
    (Segment::Geometry, "ring_prime_step", 4),
    (Segment::Geometry, "ring_chunk_bytes", 4),
    (Segment::Geometry, "ring_start_slot", 8),
    (Segment::Geometry, "ring_region_device_identity", 12),
    (Segment::Geometry, "stripe_width_upper_bound", 1),
    (Segment::Geometry, "group_size", 1),
    (Segment::Geometry, "directory_unit_pointer", 59),
    (Segment::Geometry, "mapping_origin", 24),
    (Segment::Tunable, "time_threshold", 4),
    (Segment::Tunable, "dirty_threshold", 8),
    (Segment::Tunable, "compaction_watermarks", 24),
    (Segment::Runtime, "journal_tail", 8),
    (Segment::Runtime, "journal_instance", 4),
    (Segment::Runtime, "journal_reserved", 0),
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Variant {
    AsLayout,
    KeySlotInline,
    KdfIdentifier4,
    DeviceCountDropped,
    DirectoryPointerDropped,
    AllThreeWidest,
}

const VARIANTS: [Variant; 6] = [
    Variant::AsLayout,
    Variant::KeySlotInline,
    Variant::KdfIdentifier4,
    Variant::DeviceCountDropped,
    Variant::DirectoryPointerDropped,
    Variant::AllThreeWidest,
];

impl Variant {
    fn name(self) -> &'static str {
        match self {
            Variant::AsLayout => "as_layout",
            Variant::KeySlotInline => "key_slot_inline",
            Variant::KdfIdentifier4 => "kdf_identifier_4",
            Variant::DeviceCountDropped => "device_count_dropped",
            Variant::DirectoryPointerDropped => "directory_pointer_dropped",
            Variant::AllThreeWidest => "all_three_widest",
        }
    }
    fn total_bytes(self) -> u64 {
        let base = layout_total_bytes();
        match self {
            Variant::AsLayout => base,
            Variant::KeySlotInline => base + KEY_SLOT_INLINE_BYTES,
            Variant::KdfIdentifier4 => base - row_width("kdf_identifier") + KDF_IDENTIFIER_WIDE_BYTES,
            Variant::DeviceCountDropped => base - row_width("device_count"),
            Variant::DirectoryPointerDropped => base - row_width("directory_unit_pointer"),
            Variant::AllThreeWidest => base + KEY_SLOT_INLINE_BYTES - row_width("kdf_identifier") + KDF_IDENTIFIER_WIDE_BYTES,
        }
    }
}

fn row_width(field: &str) -> u64 {
    SUPERBLOCK_ROWS.iter().find(|(_, name, _)| *name == field).map(|(_, _, width)| *width).expect("字段表里没有这一行")
}

fn segment_bytes(segment: Segment) -> u64 {
    SUPERBLOCK_ROWS.iter().filter(|(row_segment, _, _)| *row_segment == segment).map(|(_, _, width)| *width).sum()
}

fn layout_total_bytes() -> u64 {
    SUPERBLOCK_ROWS.iter().map(|(_, _, width)| *width).sum()
}

/// 跨几个 512 字节扇区。
fn sectors_spanned(total_bytes: u64) -> u64 {
    total_bytes.div_ceil(SLOT_BYTES)
}

/// E100 的口径：跨 n 个扇区的结构有 2ⁿ − 2 种可读但不一致的撕裂态；一个扇区内没有。
fn torn_states(total_bytes: u64) -> u64 {
    (1u64 << sectors_spanned(total_bytes)) - 2
}

/// 512 槽里还剩多少；装不下时报 0 并由 `overflow_bytes` 说明爆多少。
fn slot_remainder(total_bytes: u64) -> u64 {
    SLOT_BYTES.saturating_sub(total_bytes)
}

fn overflow_bytes(total_bytes: u64) -> u64 {
    total_bytes.saturating_sub(SLOT_BYTES)
}

fn emit(emitter: &mut Emitter, line: &str) {
    println!("{}", emitter.emit_raw(line));
}

fn main() {
    let mut emitter = Emitter::new();
    emit(&mut emitter, &format!("name=config slot_bytes={SLOT_BYTES} rows={} key_slot_inline={KEY_SLOT_INLINE_BYTES} kdf_wide={KDF_IDENTIFIER_WIDE_BYTES}", SUPERBLOCK_ROWS.len()));
    for segment in SEGMENTS {
        emit(&mut emitter, &format!("name=segment segment={} bytes={}", segment.name(), segment_bytes(segment)));
    }
    let mut fits = 0u64;
    for variant in VARIANTS {
        let total = variant.total_bytes();
        if total <= SLOT_BYTES {
            fits += 1;
        }
        emit(&mut emitter, &format!(
            "name=variant arm={} total_bytes={total} slot_remainder={} overflow_bytes={} sectors={} torn_states={}",
            variant.name(), slot_remainder(total), overflow_bytes(total), sectors_spanned(total), torn_states(total)
        ));
    }
    emit(&mut emitter, &format!(
        "name=verdict layout_total={} variants={} variants_fitting_512={fits} e115_e124_cells_superseded=true",
        layout_total_bytes(), VARIANTS.len()
    ));
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 绝对值断言 1：五段与总计按 first-txn-layout.md 表后合计钉住（413，512 槽内余 99）。
    #[test]
    fn segments_and_total_match_the_layout_table() {
        assert_eq!(segment_bytes(Segment::Bootstrap), 166);
        assert_eq!(segment_bytes(Segment::EncryptionReserve), 32);
        assert_eq!(segment_bytes(Segment::Geometry), 167);
        assert_eq!(segment_bytes(Segment::Tunable), 36);
        assert_eq!(segment_bytes(Segment::Runtime), 12);
        assert_eq!(layout_total_bytes(), 413);
        assert_eq!(slot_remainder(413), 99);
        assert_eq!(SUPERBLOCK_ROWS.len(), 39, "跑前登记条款 b：行数");
    }

    /// 绝对值断言 2：六条变体按跑前登记的手算。
    #[test]
    fn variants_match_the_preregistered_hand_arithmetic() {
        assert_eq!(Variant::AsLayout.total_bytes(), 413);
        assert_eq!(Variant::KeySlotInline.total_bytes(), 493);
        assert_eq!(Variant::KdfIdentifier4.total_bytes(), 415);
        assert_eq!(Variant::DeviceCountDropped.total_bytes(), 409);
        assert_eq!(Variant::DirectoryPointerDropped.total_bytes(), 354);
        assert_eq!(Variant::AllThreeWidest.total_bytes(), 495);
        assert_eq!(slot_remainder(495), 17);
    }

    /// 撕裂态公式：一个扇区内 0，跨 2 个扇区 2，跨 3 个 6（E100 的 2ⁿ − 2）。
    #[test]
    fn torn_states_follow_the_e100_formula() {
        assert_eq!(sectors_spanned(413), 1);
        assert_eq!(torn_states(413), 0);
        assert_eq!(sectors_spanned(600), 2);
        assert_eq!(torn_states(600), 2);
        assert_eq!(torn_states(1300), 6);
        assert_eq!(overflow_bytes(600), 88);
        assert_eq!(overflow_bytes(413), 0);
    }

    /// 每个变体都在一个扇区内：六条全 ≤ 512、撕裂态全 0。
    #[test]
    fn every_variant_fits_one_sector() {
        for variant in VARIANTS {
            assert!(variant.total_bytes() <= SLOT_BYTES, "{:?}", variant);
            assert_eq!(torn_states(variant.total_bytes()), 0, "{:?}", variant);
        }
    }

    /// 被变体引用的三行确实在表里且宽度按 kb：KDF 2、devs 4、间接目录指针 59。
    #[test]
    fn referenced_rows_have_the_knowledge_base_widths() {
        assert_eq!(row_width("kdf_identifier"), 2);
        assert_eq!(row_width("device_count"), 4);
        assert_eq!(row_width("directory_unit_pointer"), 59);
        assert_eq!(row_width("feature_bits"), 96, "D15 三个 bitmap 各 256 位");
        assert_eq!(row_width("nonce_watermark"), 12, "E124");
    }
}
