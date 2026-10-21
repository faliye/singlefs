# Feature Bits Accounting Ledger

[中文](feature-bits.md) · **English** · [日本語](feature-bits.ja.md)

Three bitmaps (incompat / compat_ro / compat) each having 256 bits, residing in the system configuration. Whenever a bit is allocated, write a line here explaining its name, which format version and which commit introduced it, its status, and its one-sentence semantics.

**The sole authoritative registration point for bit numbers is not in this file, but in the registry table of settled item 4 of D15 (format freeze policy)** (`.claude/kb/decisions/15-格式冻结政策.md`). "Which bit belongs to what" follows that table authoritatively; when the two places mismatch, modify `.claude/kb/feature-bits.md`, do not modify that table. The feature-bits slot of gate code-source-discipline (`.claude/gate.d/code-source-discipline.sh`) compares the two bit-by-bit and flags mismatches as red.

Allocation rights follow "first merged, first claimed", not an approval system (settled item 10 of D15 (format freeze policy)). **Once a bit has been used, it must never be reclaimed**: if reclaimed and reassigned to a new feature, that bit in an old image would be interpreted by new code as "new feature enabled", causing the file system to execute genuinely incorrect actions (for example, assuming nonexistent encryption characteristics are present and searching for nonexistent keys). Retired bits have their status written as "retired", and their semantics permanently locked to the sentence used at the time of retirement, never to be rewritten.

<!-- feature-bits:table -->
One row per bit, seven columns following the column order established in settled item 10 of D15 (format freeze policy). "One-sentence semantics" must verbatim contain the "meaning" of the same bit in the registry table of settled item 4 of D15 (format freeze policy); the feature-bits slot of gate code-source-discipline compares against this rule. "Name" contains the constant name in code; if there is no constant on the code side, write `—`.

| Bit | Category | Name | Introduced version | Introduced commit | Status | One-sentence semantics |
|---|---|---|---|---|---|---|
| 0 | incompat | INCOMPAT_RETIRED_FIRST_SSD_LINE_WITH_ROLLBACK_WITNESS_BIT | Format version 1 | 53c9f38 | Retired | First pure SSD layout line; set from mkfs onward; readers not recognizing this bit cannot mount (settled item 1 of D12 (target media): "one incompat bit per layout family") |
| 1 | incompat | INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT | Format version 1 | 45d49aaf | Active | First pure SSD layout line (system configuration carries rollback lower bound F, root record carries unmount mark, without rollback witness or rollback rows); set from mkfs onward; readers not recognizing this bit cannot mount; after bit 0 is retired, readers encountering bit 0 must refuse mount |

**Neither compat_ro nor compat bitmaps have allocated any bits; incompat has allocated bit 0 (retired) and bit 1; all other bits remain completely unallocated** (source: registry table in settled item 4 of D15 (format freeze policy)).

**Which bits are referenced on the code side**: `INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT = 0x02` in `crates/singlefs-core/src/system_configuration.rs` is bit 1; `INCOMPAT_RETIRED_FIRST_SSD_LINE_WITH_ROLLBACK_WITNESS_BIT = 0x01` in the same file is retired bit 0, used only in tests and comments to preserve "used bits are never reclaimed" in code; `SUPPORTED_INCOMPAT_BITS` is synthesized from bit 1 and is not a bitmask literal, so gate code-source-discipline feature-bits slot cannot parse its bit number, listing it in the success message under "unparsed bit numbers". `crates/singlefs-checker/src/lib.rs` checks incompat using its own named constant `INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT = 0x02` (line 219).

**Bitmap endianness**: Bitmaps are little-endian; bit n lives in byte n div 8 at low bit n mod 8 (settled item 13 of D22 (unit atomicity synthesis)).
