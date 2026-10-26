## D7 Merging into Linux Mainline —— Settled

[中文](07-是否进Linux主线.md) · **English** · [日本語](07-Linux-メインラインへのマージ是非.ja.md)

D7 (Merging into Linux Mainline) governs the relationship between this project and Linux mainline: whether to merge, and two matters directly settled by this choice—format freedom and where acceptance criteria come from. Out of scope: what acceptance criteria themselves look like (`.claude/singlefs-ai-sop/rules/show-me-test.md`), and kernel interface-level facts and tradeoffs ([prior-art.md](../prior-art.md)).

### Settled Items

| # | Item | Settlement |
|---|---|---|
| 1 | **Whether to merge into Linux mainline in the initial years** | Stay out-of-tree, develop as a single-maintainer project; format is therefore not frozen prematurely, acceptance criteria take gates **Status: Settled.** **Progress: No code object.** |

#### Settled Item 1: Out-of-tree for initial years, developed as a single-maintainer project

**Settlement**: **Do not merge into Linux mainline in the initial years, develop as a single-maintainer project.** This settles two matters: format is not frozen prematurely—change when desired with our own cadence, compatible with [format-evolution.md](../../rules/format-evolution.md) "until the first external user appears, the disk format is soft"; acceptance criteria do not adopt Linux conventions, but take gates (`.claude/singlefs-ai-sop/rules/show-me-test.md`).

**Scope**: Governs solely the single choice of "whether to take the mainline path", regardless of when it is re-judged, and regardless of kernel interface tradeoffs. Merging into mainline incurs social costs, review cycles, and maintainer relations—all three must enter the budget and are uncontrollable, while the format must also be frozen prematurely.

**Basis**:

- No experiment: Merging into mainline is a choice of product and collaboration direction, without measurable quantities—the three costs to compare (social costs, review cycles, and timing of format freeze) are not numbers measurable locally.
- bcachefs being removed from mainline was a social failure, not a technical failure; [prior-art.md](../prior-art.md) 7.8 contemporary check strengthens this to "after removal from mainline, the project continues to evolve normally, and only then dropped its experimental label"—outside mainline is a viable distribution path, not merely a fallback after failure.

**Owed**: None.

## Historical Versions

Historical entries for D7 (Merging into Linux Mainline) are centralized in [decisions-history.md](../decisions-history.md).
