# Feature Bits Accounting Ledger

[中文](feature-bits.md) · **English** · [日本語](feature-bits.ja.md)

Three bitmaps (incompat / compat_ro / compat) each having 256 bits, residing in the system configuration. Whenever a bit is allocated, write a line here explaining its name, which format version and which commit introduced it, its status, and its one-sentence semantics.

**The sole authoritative registration point for bit numbers is not in this file, but in the registry table of settled item 4 of D15 (Format Freeze Policy)** (`.claude/kb/decisions/15-格式冻结政策.md`). "Which bit belongs to what" follows that table authoritatively; when the two places mismatch, modify `.claude/kb/feature-bits.md`, do not modify that table. The feature-bits slot of gate code-source-discipline (`.claude/gate.d/code-source-discipline.sh`) compares the two bit-by-bit and flags mismatches as red.

Allocation rights follow "first merged, first claimed", not an approval system (settled item 10 of D15 (Format Freeze Policy)). **There is no format freeze before milestone 17 closes; before the format is frozen there are no old disks, so bit numbers may be reclaimed and renumbered**; when a bit number changes, `crates/` and the checker follow in the same change; after the freeze a used bit must never be reclaimed, with the rationale in settled item 10 of D15 (Format Freeze Policy).

<!-- feature-bits:table -->
One row per bit, seven columns following the column order established in settled item 10 of D15 (Format Freeze Policy). "One-sentence semantics" must verbatim contain the "meaning" of the same bit in the registry table of settled item 4 of D15 (Format Freeze Policy); the feature-bits slot of gate code-source-discipline compares against this rule. "Name" contains the constant name in code; if there is no constant on the code side, write `—`.

| Bit | Category | Name | Introduced version | Introduced commit | Status | One-sentence semantics |
|---|---|---|---|---|---|---|
| 0 | incompat | — | Format version 1 | 53c9f38 | Free | Left free, may be reallocated; no layout uses it, and readers seeing it refuse it as an unrecognized bit |
| 1 | incompat | INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT | Format version 1 | 45d49aaf | Active | First pure SSD layout line (system configuration carries rollback floor F, root record carries unmount mark, without rollback witness or rollback rows); set from mkfs onward; readers not recognizing this bit cannot mount |
| 2 | incompat | INCOMPAT_INODE_RECORD_TRUNCATION_FIELDS_BIT | Format version 1 | 0b926bd3f | Active | inode record carries truncation fields (T and N_w starting at offset 120), carried unconditionally; bit 2 is part of the layout identity, set from mkfs onward, and readers accept only the form carrying both bit 1 and bit 2 (first byte 0x06) |

**Neither compat_ro nor compat bitmaps have allocated any bits; incompat has allocated bit 1 and bit 2, bit 0 is left free, and all other bits remain completely unallocated** (source: registry table in settled item 4 of D15 (Format Freeze Policy)).

**Which bits are referenced on the code side**: `INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT = 0x02` in `crates/singlefs-core/src/system_configuration.rs` is bit 1, and `INCOMPAT_INODE_RECORD_TRUNCATION_FIELDS_BIT = 0x04` is bit 2; the free bit 0 has no constant on the code side; `INCOMPAT_FIRST_BYTE_OF_THIS_LAYOUT` in the same file is synthesized from bit 1 and bit 2 (first byte 0x06) and is not a bitmask literal, its declaration spanning lines, so gate code-source-discipline feature-bits slot cannot parse its bit number, listing it in the success message under "unparsed bit numbers". `crates/singlefs-checker/src/lib.rs` checks incompat using its own named constant `INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT = 0x02` (line 292) and `INCOMPAT_INODE_RECORD_TRUNCATION_FIELD_BIT = 0x04` (line 295, one letter S shorter than the constant name in the core layer).

**Bitmap endianness**: Bitmaps are little-endian; bit n lives in byte n div 8 at low bit n mod 8 (settled item 13 of D22 (Synthesizing Unit Atomicity)).
