## E117 Header Width and Geometry After Factoring in Reserved Bits —— Run (2026-09-07, pure arithmetic, 12 unit tests / 12 mutations caught)

[中文](117-预留位算进去之后的头宽与几何.md) · **English** · [日本語](117-予約ビット算入後のヘッダ幅と幾何.ja.md)

**What is asked**: D18 (Information Carried in a Block) settled item 7 field table calculated data unit header as 105 bytes,
**whereas the two fields in that section's phrase "v1 reserves field slots for nonce code and MAC" were outside the table**.
Factoring them in, how wide are the headers for the three classes of units, and how much do downstream geometries change?

**What it answers**: C193 (size of unit header when encryption is enabled never calculated across repo), and also
the figure requested by the user before settling D18 (Information Carried in a Block) settled item 14
(user: "take Arm A, but calculate header width clearly first before deciding").

### Three Arms

| Arm | Reserved Bits | What It Is |
|---|---|---|
| `bare` | 0 | Measurement basis used by four downstream experiments on 2026-09-07 |
| `resv12` | 12 + 16 = 28 | Settled item 14 **Arm A**: nonce code = complete 96-bit nonce, MAC full 128-bit |
| `resv4` | 4 + 16 = 20 | **Discriminative control arm, not a candidate** —— if code had only 4 bytes |

### Clauses Pasted Verbatim into Source Code

| Clause | Sentence Used |
|---|---|
| D18 (Information Carried in a Block) settled item 7 | "v1 reserves field slots for nonce code and MAC", rationale "without reservation, enabling encryption becomes a layout change" |
| D9 (Encryption) settled item 2 | MAC full 128-bit without truncation ⇒ reserved MAC field 16 bytes |
| D9 (Encryption) settled item 4 | "Pointer header stores a 96-bit nonce for each logical extent" ⇒ complete nonce 12 bytes |
| D18 (Information Carried in a Block) settled item 7 supplementary note | E73 (Fanout Cost of Node Key Interval) three lower bounds 58 / 67 / 76 each plus write sequence 10 and birth sequence 4 become 72 / 81 / 90 |
| D19 (Block Pointer Structure & Width Budget) settled item 4 | Pointer 59 ⇒ internal entry width 22 + 59 = 81 (E73 (Fanout Cost of Node Key Interval) itself used 22 + 32 = 54) |

### Criteria and Failure Clauses Hardcoded Before Run

1. Header width closed form: each class of header = fields in table + reserved bits, pinning absolute values class by class.
2. Code 2 fanout = `(16384 − header) / entry_width`, taking both entry widths 54 and 81, pinning absolute values cell by cell.
3. Code 3 records per container = `(32768 − header) / 140`, pinning absolute values cell by cell.
4. Code 1 net payload = `32768 − header`, pinning absolute values cell by cell.
5. **Discriminative power**: `resv12` and `resv4` must differ on at least one cell. Identical on all cells ⇒ judge as "sampling point insensitive" (`.claude/rules/mutation-sampling.md` category three), look for sampling points crossing integer boundaries, **forbidding recording as "reserved bits do not affect geometry"**.
6. **Positive control run arm by arm**: when header forced to 0, fanout = unit / entry_width, record count = 32768 / 140.
7. **Negative control**: entry width > usable bytes ⇒ return 0.

**Failure clauses**: if criterion 5 fails, changing criterion is forbidden; supplement sampling points per category three.

### Actual Run

| Arm | Code 1 Header | Code 3 Header | Code 1 Payload | Code 2 Three-Tier Header | Fanout (Entry 54) | Fanout (Entry 81) | Code 3 Records per Container |
|---|---|---|---|---|---|---|---|
| `bare` | 105 | 107 | 32663 | 72 / 81 / 90 | 302 / 301 / 301 | 201 / 201 / 201 | 233 |
| **`resv12` (Arm A)** | **133** | **135** | 32635 | 100 / 109 / 118 | 301 / 301 / 301 | 201 / 200 / 200 | **233** |
| `resv4` | 125 | 127 | 32643 | 92 / 101 / 110 | 301 / 301 / 301 | 201 / 201 / 200 | 233 |

⇒ **Geometric overhead of Arm A is a drop of 0–1 cell in fanouts, with zero cells dropped in Code 3 records per container.**

### Judgment of Three Controls

| Control | Criterion | Actual Run |
|---|---|---|
| Positive (arm by arm) | When header is 0, fanout = closed-form value | All three arms give 303 / 202 / 234 |
| Discriminative power | `resv12` and `resv4` differ in at least one cell | **Satisfied**: at entry width 81 second tier, `resv12` gives 200, `resv4` gives 201 (artifact `cells_differing=1`) |
| Negative | Entry width > usable bytes | 0 |

⚠️ **That discriminative cell is not on E73 (Fanout Cost of Node Key Interval) own entry width**: at entry width 54 both arms are identical cell by cell,
floor division swallowed that 8-byte difference. Per `.claude/rules/mutation-sampling.md` this is **sampling point insensitive**,
not "reserved bits do not affect geometry" —— retained in unit test `t10`, with discriminative power borne by that cell in `t09`.

### One Value Registered Before Run Judged False, Retained in Archive Unaltered

Criterion 2 registered before run that "`bare` all three tiers are 302", **actual run 302 / 301 / 301** ——
second tier `(16384 − 77) / 54 = 301.05` already dropped one cell.
The registered value was extrapolated by main agent doing manual calculation on tier one alone; **when revising judgment the second time, only tier three was updated, still omitting tier two**.
Both errors stemmed from the same flaw: **treating one sampling point as three**.
E73 (Fanout Cost of Node Key Interval) own phrasing was "three tier conclusions consistent (drop 0 or 1 cell)", which does not conflict with actual run;
what conflicts is E117 (Header Width and Geometry After Factoring in Reserved Bits) stronger pre-run registration. Both values remain in source `t03`, forbidding altering pre-run value to the correct one.

### What It Cannot Answer

Pure arithmetic, no I/O, no crash-point replay. Does not answer whether "E73 (Fanout Cost of Node Key Interval) child pointer width 32
should change to 59" —— that is an account of C89 (Leftover Recomputation from Closing the Unit Header), here both widths are merely exhibited.
Does not recalculate internal models of E97 (Entry Encoding of Accounting and Allocation Records) / E98 (Geometry of Inode Records and Inode Trees) /
E103 (Update Cost of Inode Packing Path 1), only reporting what their load-bearing geometric numbers become under new header widths.
**Running N rounds indicates absence of hidden state, not statistical stability** —— it is a deterministic model.

### Measurement Basis and Artifact

Raw output `e117-reserved-header-2026-09-12.out` (21 lines, ending line `name=done emitted=21`; the revision with Code 3 header 107 and Code 2 three tiers 72 / 81 / 90, fanout and record counts identical cell by cell to previous version `e117-reserved-header-2026-09-07.out` in results/; mutation replay `e117-mutate-2026-09-12.log` caught all 12).
Apparatus `scripts/experiment-apparatus/e117_reserved_header.rs`,
mutation table `scripts/mutation/tables/e117_reserved_header.tsv` (all 12 caught).

```bash
bash scripts/experiment-apparatus/replay.sh E117     # Byte-for-byte comparison with e117-reserved-header-2026-09-12.out
```

### Influenced Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D18 (Information Carried in a Block) settled item 14 | Support | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D18 (Information Carried in a Block) settled item 16 | Support | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D18 (Information Carried in a Block) settled item 7 | No effect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D9 (Encryption) | No effect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D19 (Block Pointer Structure & Width Budget) settled item 4 | No effect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |

## Historical Versions

Historical entries for E117 (Header Width and Geometry After Factoring in Reserved Bits) are consolidated in [experiments-history.md](../experiments-history.md).
