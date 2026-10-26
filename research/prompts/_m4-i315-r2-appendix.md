**出处 `.claude/kb/experiments/112-legacy-writer-encountering-unknown-trees.en.md:1-9`（整段抄，未转述）**

```markdown
## E112 Legacy Writer Encountering Unknown Trees — Run (2026-09-06, pure arithmetic, 11 unit tests / 9 mutants caught): **Among the four mountable cells, `Skip + Carry` has zero silent outcomes, with cost of 2 extra rows written per generation; ⚠️ `Refuse` two cells also have zero silent outcomes, eliminated from balance by `mount_ok` filter — its cost did not enter the silence criterion**

[中文](112-旧写者遇到不认识的树.md) · **English** · [日本語](112-旧ライターが未知のツリーに遭遇した場合.ja.md)

**This answers two items in the closure inventory of D8 (core index structure) settled item 8**:
"legacy writer toward unknown accounting dimensions" and "which side remaining 15 bits of flags choose".
The two are **not independent**: when flags choose "skip this entry", v1 **writers** also skip that tree,
leaving its accounting rows unmaintained — falling right into the hole of the first item. Hence one experiment crosses both policy sets.

```

**出处 `.claude/kb/experiments/112-legacy-writer-encountering-unknown-trees.en.md:10-21`（整段抄，未转述）**

```markdown
### Cited Items (Copied Verbatim into Source Comments)

| Item | The Exact Sentence Used |
|---|---|
| D8 (core index structure) settled item 1 | Accounting entries take **idempotent full value**, not incremental Δ |
| D5 (snapshot / space accounting mechanism) settled item 2 | Generation takes checkpoint number, **retains only most recent K generations** |
| D5 (snapshot / space accounting mechanism) settled item 4 | Twelve metrics currently in use, of which **one carries tree dimension** (inode number watermark, maintained only for writable heads) |
| D15 (format freeze policy) settled item 2 | Tree table day-1 made as open list (from 2026-09-13 title removes "system configuration", tree table lives in indirection layer of root record); added keyspace for legacy readers **merely means not seeing new tree** ⇒ compat |
| D15 (format freeze policy) decision table | Three tiers: incompat disallows mount if unknown / compat_ro read-only mount if unknown / compat mounts arbitrarily if unknown |
| I-3.1 (allocated statistics match) | Allocated space statistics == sum obtained by actually traversing all references |
| `.claude/rules/fs-design.md` | Whether a format branch is worthwhile depends on whether it can turn "misused" into "unmountable" |

```

**出处 `.claude/kb/experiments/112-legacy-writer-encountering-unknown-trees.en.md:22-39`（整段抄，未转述）**

```markdown
### Model (Locked Before Run)

A pool has `T_known` trees known to v1 and `T_new` trees unknown to v1 (new types, or known types with unknown flags bits set for v1).
A **v1 writer** mounts and publishes `G` generations. Each tree has `R` accounting rows per generation, keys carry generation number, **retains only most recent K generations**.

**Two crossed policy sets**:

| Flags Policy `F` | Meaning |
|---|---|
| `Ignore` | Ignore unknown bits, continue interpreting entry per known semantics |
| `Skip` | Skip entry as "unknown tree" (same tier as unknown type) |
| `Refuse` | Refuse mount |

| Accounting Policy `C` | Meaning |
|---|---|
| `Carry` | Writer carries unknown accounting rows **as-is to new generation** (values unchanged, generation updated) |
| `NoCarry` | Do not carry |

```

**出处 `.claude/kb/experiments/112-legacy-writer-encountering-unknown-trees.en.md:40-52`（整段抄，未转述）**

```markdown
### Criteria (Locked Before Run, Unchanged After Run)

1. **Each cell reports four absolute values**, forbidding only cross-arm comparisons:
   `misread_records` (records interpreted under wrong encoding), `rows_lost` (accounting rows of unknown trees missing at generation G),
   `rows_written_per_gen` (rows written per generation), `i31_red` (whether I-3.1 (allocated statistics match) turns red, 0/1).
2. **`rows_lost` must have an independently calculated expected value**: under `NoCarry` and `G > K` it should be exactly `T_new × R`,
   given by another closed-form arithmetic rather than simulation counts; the two must match.
3. **Silence must be countable**: an outcome is "silent" if and only if `misread_records > 0 ∧ i31_red = 0`
   or `rows_lost > 0 ∧ i31_red = 0`. **Report silent outcome count per cell**.
4. **Write cost must report absolute values**: how many extra rows `Carry` writes per generation relative to `NoCarry`, reported as `T_new × R`, not percentage.
5. **`Refuse` column must report its cost**: generations where v1 writer cannot mount = `G` (all),
   and whether this is compatible with D15 (format freeze policy) settled item 2 "added keyspace ⇒ compat" (0/1).

```

**出处 `.claude/kb/experiments/112-legacy-writer-encountering-unknown-trees.en.md:53-70`（整段抄，未转述）**

```markdown
### Failure Clauses (Locked Before Run)

| Clause | Judgment on Trigger |
|---|---|
| **Positive control 1**: Under `Skip + NoCarry` and `G > K`, `rows_lost` must be > 0 | Not true ⇒ intergenerational discard dimension absent from model, **whole round invalidated** |
| **Positive control 2** (dedicated to `Ignore` arm): Under `Ignore`, `rows_lost` must == 0 **and** `misread_records` > 0 | Not true ⇒ "thinking it knows" dimension absent from model, **whole round invalidated** |
| **Positive control 3** (dedicated to `Refuse` arm): Under `Refuse`, `mount_ok` must == 0, and `rows_lost` and `misread_records` must both == 0 | Not true ⇒ refuse mount dimension absent from model, **whole round invalidated** |
| **Negative control**: Under `Carry` and `G > K`, `rows_lost` must == 0 | Not true ⇒ carry dimension absent from model, **whole round invalidated** |
| **Positive control 4**: Under `Ignore`, `misread_records` must be > 0 | Not true ⇒ misread dimension absent from model, **whole round invalidated** |
| **Reverse acceptance clause**: If silent outcome count of `Skip + Carry` is **not** minimum across whole table, record truthfully and adjust conclusion per minimum cell | — |
| **Reverse acceptance clause 2**: If extra rows written per generation by `Carry` exceeds `T_known × R` (carrying is more expensive than writer's own trees), conclusion changed to "carrying uneconomical, find another route" | Record truthfully |

**Acceptance clause written before run**: Main agent before 2026-09-06 run inclined toward `Skip + Carry`.
**If silent outcome count of `Skip + Carry` is not minimum, or its write cost exceeds reverse acceptance clause 2 line, change conclusion** —
written before run, unchanged after run.

⚠️ **These three positive controls were modified once on 2026-09-06 filing day before run**; previously only one "under `NoCarry` and `G > K` `rows_lost` must be > 0, failing under any `F` invalidates whole round". Reason for change: under `Ignore` arm writer **thinks it knows** that tree and maintains its rows as usual ⇒ `rows_lost` legitimately equals 0, original clause would falsely invalidate correct result. **This is the other side of discipline "positive controls must run for each arm under test"** — controls for each arm are **not the same one**, blind copying harms valid cases. Change happened **before any numbers emerged**, locked thereafter.

```

**出处 `.claude/kb/experiments/112-legacy-writer-encountering-unknown-trees.en.md:71-76`（整段抄，未转述）**

```markdown
### Critical Model Premise (Checked on 2026-09-06)

**Today only one metric carries tree dimension**: D5 (snapshot / space accounting mechanism) settled item 4 item 12 "inode number watermark", verbatim "**carries** (maintained only for writable heads)" — items 8 / 9 withdrawn same day.
⇒ **An unknown tree has accounting rows to lose only when it is a writable head**; if not writable head, zero rows exist, `NoCarry` harmless.
⇒ After loss, **who catches it**: I-9.11 (writable clone head has watermark row) verbatim "tree ID of each writable clone head has 'inode number watermark' row in accounting", which is **unimplemented** ⇒ must report two numbers: "caught by specification" vs "caught today".

```

**出处 `.claude/kb/experiments/112-legacy-writer-encountering-unknown-trees.en.md:77-102`（整段抄，未转述）**

```markdown
### Measurements (2026-09-06, pure arithmetic, 11 unit tests / 9 mutants caught)

**Measurement basis and rerun**: Code `scripts/experiment-apparatus/e112_old_writer_unknown_tree.rs`
(`cargo run --release --bin e112-old-writer-unknown-tree`),
artifact `e112-old-writer-unknown-tree-2026-09-06.out` (11 lines, ending `emitted=11`),
mutation table `scripts/mutation/tables/e112_old_writer_unknown_tree.tsv`, log `e112-mutate-2026-09-06.log`,
hooked into `scripts/experiment-apparatus/replay.sh` (rerun command `bash scripts/experiment-apparatus/replay.sh` with E112 (legacy writer encountering unknown trees)).
Parameters: `t_known=12 h_known=6 t_new=4 h_new=2 r_tree_dim=1 r_pool=11 k_keep=4 g_gens=10 records_per_new_tree=1000`.

| Cell | Mountable | Misread Records | Lost Rows | Rows Written / Gen | Silent Today | Silent by Spec |
|---|---|---|---|---|---|---|
| `Ignore + Carry` | Yes | **4000** | 0 | 19 | **1** | **1** |
| `Ignore + NoCarry` | Yes | **4000** | 0 | 19 | **1** | **1** |
| **`Skip + Carry`** | Yes | 0 | 0 | 19 | **0** | **0** |
| `Skip + NoCarry` | Yes | 0 | **2** | 17 | **1** | 0 |
| `Refuse + Carry` | **No** | 0 | 0 | 0 | 0 | 0 |
| `Refuse + NoCarry` | **No** | 0 | 0 | 0 | 0 | 0 |

⚠️ **Six cells produce only four distinct outcomes**: `Ignore` two cells and `Refuse` two cells are field-for-field identical — carrying policy has **no point of action** under these two flags policies (under `Ignore` writer thinks it knows, those rows already within its maintenance scope; under `Refuse` it never mounted). ⇒ **Carrying policy is a genuine choice only under `Skip`**, which is quantitative shape of "two policies not independent". Reporting six cells exposes this degeneracy itself, not six independent observations.
- **Criterion 2**: `rows_lost` matches independent closed form across all 6 × 4 sample points (cells × generations).
- **Criterion 4**: Carrying writes **2 extra rows** per generation (difference between `Skip` arms); writer's own rows **17 rows**;
  threshold of reverse acceptance clause 2 `T_known × 12 = 144` ⇒ **not triggered**.
- **Criterion 5**: Under `Refuse` v1 writer **fails to mount across all 10 generations**, and is **incompatible** with D15 (format freeze policy) settled item 2
  "added keyspace for legacy readers merely means not seeing new tree ⇒ compat" (in artifact `compat_with_d15_item2=0`).
- **Generation boundary**: Exactly K generations zero loss, K+1 generation loses 2 rows — off-by-one nailed by unit test.

```

**出处 `.claude/kb/experiments/112-legacy-writer-encountering-unknown-trees.en.md:103-120`（整段抄，未转述）**

```markdown
### Conclusion (Cross-Checked Against Acceptance Clause Locked Before Run)

Written before run: "Main agent inclines toward `Skip + Carry`; if its silent outcome count is not minimum, or write cost exceeds threshold, change conclusion."
**Neither triggered**: Silent outcome count of `Skip + Carry` is **0**, write cost 2 rows far below 144. ⚠️ **However "unique across whole table" is false, artifact itself shows**: `Refuse_Carry` / `Refuse_NoCarry` both show `silent_today=0 silent_by_spec=0`. Three of six cells have zero silence; `Skip + Carry` is merely unique **among mountable ones** — what removed `Refuse` from scale was `if o.mount_ok` filter in source code, not its performance. ⇒ **Selecting by "minimum silence" does not price "unmountable"**, `unmountable_gens=10` in Criterion 5 was reported separately, uncounted in comparison.
⇒ **Conclusion maintained**: Unknown bits in flags take "skip this entry", writer carries unknown accounting rows as-is to new generation.

⚠️ **Three findings obtained incidentally from measurements, outside original question**:

1. **`Ignore` is silent even by specification** (`silent_by_spec=1`) — invariant inventory has **no** check catching "records interpreted under wrong encoding".
   This is not `Ignore` losing elsewhere; the cell where it loses is **unwatched by anyone**. Placement recorded in C156 (record encoding misread has no invariant).
2. **`Skip + NoCarry` is silent only under 2026-09-06 implementation** (`silent_today=1` while `silent_by_spec=0`):
   What is lost is writable head's "inode number watermark" row, which I-9.11 (writable clone head has watermark row) directly targets, **yet is unimplemented**.
   ⇒ Implementing I-9.11 (writable clone head has watermark row) downgrades `NoCarry` from "silent error" to "catchable error",
   but **does not** make it correct — `Carry` still loses 0 rows vs 2 rows.
3. **Exposure surface is narrow**: Checked on 2026-09-06 only one metric carries tree dimension, maintained only for writable heads
   ⇒ An unknown tree, **if not a writable head, has zero rows**, `NoCarry` harmless to it.
   ⚠️ **This cell widens as metric inventory expands**: adding another metric carrying tree dimension causes `NoCarry` loss to scale with tree count.

```

**出处 `.claude/kb/experiments/112-legacy-writer-encountering-unknown-trees.en.md:121-126`（整段抄，未转述）**

```markdown
### Measurement Basis

Pure arithmetic / discrete simulation, zero I/O. **Running same binary N times is necessarily identical**,
so round count does not constitute evidence strength; evidence strength comes from mutation testing and assertions nailing absolute values
(deterministic exception to "single observation does not count" in `.claude/singlefs-ai-sop/rules/test-discipline.md`).

```

**出处 `.claude/kb/experiments/112-legacy-writer-encountering-unknown-trees.en.md:127-134`（整段抄，未转述）**

```markdown
### Decisions Affected

| Decision Item | Relationship | Review |
|---|---|---|
| D8 (core index structure) settled item 8 | Supports | 2026-10-11 unaffected: wording only (temporal words, location references, and self-references), conclusions, numbers, and artifact unchanged |
| D5 (snapshot / space accounting mechanism) | Does not affect | 2026-10-11 unaffected: wording only (temporal words, location references, and self-references), conclusions, numbers, and artifact unchanged |
| D15 (format freeze policy) settled item 2 | Does not affect | 2026-10-11 unaffected: wording only (temporal words, location references, and self-references), conclusions, numbers, and artifact unchanged |

```

**出处 `.claude/kb/experiments/112-legacy-writer-encountering-unknown-trees.en.md:135-146`（整段抄，未转述）**

```markdown
## Historical Versions

### 2026-09-06

#### (Part 1)
- Filed, criteria and failure clauses locked, unrun. Basis: remaining two items in D8 (core index structure) settled item 8 closure inventory,
  after user decided "adopt" on 2026-09-06, requested verify-then-adversarial-then-compare.

#### (Part 2)
- Completed run and wrote back conclusion; all three positive controls and one negative control held, all 9 mutations caught.


```

**出处 `.claude/kb/experiments/112-legacy-writer-encountering-unknown-trees.en.md:137-146`（整段抄，未转述）**

```markdown
### 2026-09-06

#### (Part 1)
- Filed, criteria and failure clauses locked, unrun. Basis: remaining two items in D8 (core index structure) settled item 8 closure inventory,
  after user decided "adopt" on 2026-09-06, requested verify-then-adversarial-then-compare.

#### (Part 2)
- Completed run and wrote back conclusion; all three positive controls and one negative control held, all 9 mutations caught.


```

**出处 `.claude/kb/experiments/112-旧ライターが未知のツリーに遭遇した場合.ja.md:1-9`（整段抄，未转述）**

```markdown
## E112 旧ライターが未知のツリーに遭遇した場合 —— 実行済み (2026-09-06、純粋算術、11単体テスト / 9変異全検出)：**マウント可能な 4 つのセルにおいて `Skip + Carry` はサイレントな結末がゼロ、代償は世代ごとに 2 行の追加書き込み；⚠️ `Refuse` の 2 セルも同様にサイレントゼロであるが、`mount_ok` フィルタによって天秤から除外されたものである —— その代償はサイレントという判定基準には入っていない**

[中文](112-旧写者遇到不认识的树.md) · [English](112-legacy-writer-encountering-unknown-trees.en.md) · **日本語**

**これは D8 (コアインデックス構造) 確定項目 8 の収束リストにある 2 項目に答えるものである**：
「旧ライターの未知の記帳次元に対する扱い」と「flags の残り 15 ビットでどちらを選択するか」。
この 2 項目は**独立ではない**：flags が「該当エントリをスキップ」を選択すると、v1 **ライター** もそのツリーをスキップし、
その結果その記帳行を誰も保守しなくなる —— まさに第 1 項目の穴に落ち込む。したがって 1 つの実験で 2 組のポリシーを交差させる。

```

**出处 `.claude/kb/experiments/112-旧ライターが未知のツリーに遭遇した場合.ja.md:10-21`（整段抄，未转述）**

```markdown
### 引用された条項 (ソースコードのコメントに逐語転記)

| 条項 | 使用された一文 |
|---|---|
| D8 (コアインデックス構造) 確定項目 1 | 記帳エントリは**冪等な完全値**を採用し、増分 Δ とはしない |
| D5 (スナップショット / 領域記帳メカニズム) 確定項目 2 | 世代はチェックポイント番号を取得、**直近 K 世代のみを保持** |
| D5 (スナップショット / 領域記帳メカニズム) 確定項目 4 | 統計量は 12 項目を使用中、うち**1 項目がツリー次元を伴う** (inode 番号水位、書き込み可能ヘッドのみに対して保守) |
| D15 (フォーマット凍結ポリシー) 確定項目 2 | ツリーテーブルは day-1 でオープンリストとして作成 (2026-09-13 よりタイトルから「システム構成」を削除、ツリーテーブルはルートレコードの間接層に常駐)；新 keyspace の追加は旧リーダーにとって**単に新ツリーが見えないだけ** ⇒ compat |
| D15 (フォーマット凍結ポリシー) 判定表 | 3 档：incompat は未知ならマウント禁止 / compat_ro は未知なら読み取り専用マウント / compat は未知でも任意にマウント |
| I-3.1 (割り当て済み統計の一致) | 割り当て済み領域統計 == すべての参照を実際に走査して得られた和 |
| `.claude/rules/fs-design.md` | フォーマット分岐に価値があるかは、「誤用」を「マウント不可」に変換できるか否かによる |

```

**出处 `.claude/kb/experiments/112-旧ライターが未知のツリーに遭遇した場合.ja.md:22-39`（整段抄，未转述）**

```markdown
### モデル (実行前に確定)

あるプールに v1 が認識可能な `T_known` 個のツリーと、v1 が認識不能な `T_new` 個のツリーが存在する（新種、または既知の種別であるが v1 未知の flags ビットが立っている）。
1 つの **v1 ライター** がマウントして `G` 世代パブリッシュする。ツリーごとに各世代 `R` 行の記帳行があり、key に世代番号を保持し、**直近 K 世代のみを保持**。

**2 組のポリシーの交差**：

| flags ポリシー `F` | 意味 |
|---|---|
| `Ignore` | 未知ビットを無視し、既知のセマンティクスに従い該当エントリの解釈を継続 |
| `Skip` | 「該当ツリーを認識不能」としてエントリをスキップ (未知の種別と同档) |
| `Refuse` | マウントを拒否 |

| 記帳ポリシー `C` | 意味 |
|---|---|
| `Carry` | ライターは認識不能な記帳行を**そのまま新世代へと引き継ぐ** (値は不変、世代番号のみ更新) |
| `NoCarry` | 引き継がない |

```

**出处 `.claude/kb/experiments/112-旧ライターが未知のツリーに遭遇した場合.ja.md:40-52`（整段抄，未转述）**

```markdown
### 判定基準 (実行前に確定、実行後の変更不可)

1. **各セルは 4 つの絶対値を報告**し、腕間比較のみの報告は禁止：
   `misread_records`（誤ったエンコーディングで解釈されたレコード数）、`rows_lost`（第 G 世代時点で既に存在しない v1 認識不能ツリーの記帳行数）、
   `rows_written_per_gen`（世代ごとの書き込み行数）、`i31_red`（I-3.1 (割り当て済み統計の一致) が赤になるか、0/1）。
2. **`rows_lost` は独立して計算された期待値を持つこと**：`NoCarry` かつ `G > K` の時、正確に `T_new × R` であるべきであり、
   この数値はシミュレーションカウントではなく別の閉形式算術によって与えられ、両者は一致しなければならない。
3. **サイレント性をカウント可能とすること**：ある結末が「サイレント」であるのは `misread_records > 0 ∧ i31_red = 0`
   または `rows_lost > 0 ∧ i31_red = 0` の時、かつその時に限る。**セルごとにサイレントな結末数を報告**。
4. **書き込みコストは絶対値を報告**：`Carry` が `NoCarry` に対していかに多くの行を世代ごとに追加書き込みするかを `T_new × R` で報告、パーセンテージでの報告は不可。
5. **`Refuse` の列はその代償を報告**：v1 ライターがマウントできない世代数 = `G`（全世代）、
   およびこれが D15 (フォーマット凍結ポリシー) 確定項目 2「新 keyspace 追加 ⇒ compat」と整合するか (0/1)。

```

**出处 `.claude/kb/experiments/112-旧ライターが未知のツリーに遭遇した場合.ja.md:53-70`（整段抄，未转述）**

```markdown
### 失敗条項 (実行前に確定)

| 条項 | トリガー時の判定 |
|---|---|
| **陽性対照甲**：`Skip + NoCarry` かつ `G > K` の時、`rows_lost` は必ず > 0 | 不成立 ⇒ 世代間破棄の次元がモデルに入っていない、**ラウンド全体を無効化** |
| **陽性対照乙** (`Ignore` 腕専用)：`Ignore` 下で `rows_lost` は必ず == 0 **かつ** `misread_records` > 0 | 不成立 ⇒「自身で認識していると思い込む」次元がモデルに入っていない、**ラウンド全体を無効化** |
| **陽性対照丙** (`Refuse` 腕専用)：`Refuse` 下で `mount_ok` は必ず == 0、かつ `rows_lost` と `misread_records` はともに == 0 | 不成立 ⇒ マウント拒否の次元がモデルに入っていない、**ラウンド全体を無効化** |
| **陰性対照**：`Carry` かつ `G > K` の時、`rows_lost` は必ず == 0 | 不成立 ⇒ 引き継ぎの次元がモデルに入っていない、**ラウンド全体を無効化** |
| **陽性対照二**：`Ignore` 下で `misread_records` は必ず > 0 | 不成立 ⇒ 誤読の次元がモデルに入っていない、**ラウンド全体を無効化** |
| **逆向き受容条項**：もし `Skip + Carry` のサイレント結末数が表全体で最小**でない**場合、実態通りに記述し、最小のセルに従い結論を変更 | —— |
| **逆向き受容条項二**：もし `Carry` の世代あたり追加書き込み行数が `T_known × R` を超過する場合（引き継ぎがライター自身のツリー記帳より高価）、結論を「引き継ぎは割に合わず、別の活路を模索すべき」に変更 | 実態通りに記述 |

**実行前に記載した受容条項**：メイン agent は 2026-09-06 実行前に `Skip + Carry` を支持。
**もし `Skip + Carry` のサイレント結末数が最小でない場合、またはその書き込みコストが逆向き受容条項二の基準を超える場合、結論を変更する** ——
この一文は実行前に記述し、実行後の変更は不可。

⚠️ **これら 3 つの陽性対照は 2026-09-06 ファイル作成当日、実行前に 1 回変更された**。変更前は「`NoCarry` かつ `G > K` の時 `rows_lost` は必ず > 0、いずれかの `F` で不成立ならラウンド全体を無効化」の 1 条のみであった。変更の理由：`Ignore` 腕下ではライターがそのツリーを**認識していると思い込み**、同様に行を保守する ⇒ `rows_lost` は正当に 0 に等しくなり、元の条項では正しい結果をラウンド全体の無効化と誤判定してしまう。**これは「陽性対照は被測定の各腕に対して実行されなければならない」という規律の別側面である** —— 各腕が実行すべき対照は**同一のものではない**ため、盲目的な単一のコピーは正当なケースを巻き添えにする。変更はいかなる数値が現れるより**前**に行われ、変更後の修正は禁止。

```

**出处 `.claude/kb/experiments/112-旧ライターが未知のツリーに遭遇した場合.ja.md:71-76`（整段抄，未转述）**

```markdown
### 重要なモデル前提 (2026-09-06 現状確認)

**本日ツリー次元を伴う統計量は 1 つのみ**：D5 (スナップショット / 領域記帳メカニズム) 確定項目 4 第 12 項「inode 番号水位」、逐語「**伴う** (書き込み可能ヘッドのみに対して保守)」—— 第 8 / 9 項は同日撤回済み。
⇒ **v1 未知のツリーは、それが書き込み可能ヘッドである場合のみ失う記帳行が存在する**；書き込み可能ヘッドでなければ 1 行も存在せず、`NoCarry` は無害。
⇒ 喪失後に**誰が検出するか**：I-9.11 (書き込み可能クローンヘッドは水位行を保持) 逐語「各書き込み可能クローンヘッドのツリー ID は記帳内に『inode 番号水位』行を持つ」、そしてこれは**未実装** ⇒「仕様上検出されるか」と「本日検出されるか」の 2 つの数値を分けて報告する必要がある。

```

**出处 `.claude/kb/experiments/112-旧ライターが未知のツリーに遭遇した場合.ja.md:77-102`（整段抄，未转述）**

```markdown
### 実測 (2026-09-06、純粋算術、11単体テスト / 9変異全検出)

**計測条件と再実行**：コード `scripts/experiment-apparatus/e112_old_writer_unknown_tree.rs`
(`cargo run --release --bin e112-old-writer-unknown-tree`)、
成果物 `e112-old-writer-unknown-tree-2026-09-06.out` (11 行、末尾 `emitted=11`)、
ミューテーション表 `scripts/mutation/tables/e112_old_writer_unknown_tree.tsv`、ログ `e112-mutate-2026-09-06.log`、
`scripts/experiment-apparatus/replay.sh` に接続済み (再実行コマンド `bash scripts/experiment-apparatus/replay.sh` に E112 (旧ライターが未知のツリーに遭遇した場合) の番号を付与)。
パラメータ：`t_known=12 h_known=6 t_new=4 h_new=2 r_tree_dim=1 r_pool=11 k_keep=4 g_gens=10 records_per_new_tree=1000`。

| セル | マウント可 | 誤読レコード | 喪失行 | 世代あたり書込行 | 本日サイレント | 仕様上サイレント |
|---|---|---|---|---|---|---|
| `Ignore + Carry` | 是 | **4000** | 0 | 19 | **1** | **1** |
| `Ignore + NoCarry` | 是 | **4000** | 0 | 19 | **1** | **1** |
| **`Skip + Carry`** | 是 | 0 | 0 | 19 | **0** | **0** |
| `Skip + NoCarry` | 是 | 0 | **2** | 17 | **1** | 0 |
| `Refuse + Carry` | **否** | 0 | 0 | 0 | 0 | 0 |
| `Refuse + NoCarry` | **否** | 0 | 0 | 0 | 0 | 0 |

⚠️ **6 つのセルには 4 つの異なる結末しか存在しない**：`Ignore` の 2 セルと `Refuse` の 2 セルはそれぞれフィールドごとに完全一致 —— 引き継ぎポリシーはこれら 2 つの flags ポリシー下で**作用点を持たない** (`Ignore` 下ではライターが自身で認識していると思い込み、それらの行は元々その保守範囲内にある；`Refuse` 下ではそもそもマウントされていない)。⇒ **引き継ぎポリシーは `Skip` 下においてのみ真の選択肢となる**。そしてこれこそが「2 つのポリシーは独立ではない」という文の定量的形態である。6 セルを報告するのはこの縮退自体を可視化するためであり、6 つの独立観測ではない。
- **判定基準 2**：`rows_lost` は全 6 × 4 個 (セル × 世代数) のサンプリング点において独立した閉形式とセルごとに一致。
- **判定基準 4**：引き継ぎは世代あたり **2 行** 余分に書き込み (`Skip` 2 腕の差)、ライター自身の記帳は **17 行**、
  逆向き受容条項二の閾値 `T_known × 12 = 144` ⇒ **トリガーされず**。
- **判定基準 5**：`Refuse` 下で v1 ライターは **10 世代すべてマウント不可**、かつ D15 (フォーマット凍結ポリシー) 確定項目 2
  「新 keyspace 追加は旧リーダーにとって単に新ツリーが見えないだけ ⇒ compat」と**非互換** (成果物内 `compat_with_d15_item2=0`)。
- **世代境界**：ちょうど K 世代は喪失せず、K+1 世代で 2 行喪失 —— off-by-one は単体テストで固定。

```

**出处 `.claude/kb/experiments/112-旧ライターが未知のツリーに遭遇した場合.ja.md:103-120`（整段抄，未转述）**

```markdown
### 結論 (実行前に確定した受容条項との照合)

実行前の記載：「メイン agent は `Skip + Carry` を支持；もしそのサイレント結末数が最小でない場合、または書き込みコストが閾値を超える場合は結論を変更する」。
**いずれもトリガーされず**：`Skip + Carry` のサイレント結末数は **0** であり、書き込みコスト 2 行は 144 を大幅に下回る。⚠️ **しかし「全表で唯一」という文は誤りであり、成果物自身が示している**：`Refuse_Carry` / `Refuse_NoCarry` の 2 行も同様に `silent_today=0 silent_by_spec=0` である。6 セル中 3 セルがサイレントゼロであり、`Skip + Carry` は単に**マウント可能なものの中で**唯一に過ぎない —— `Refuse` を天秤から除外したのはソースコード内の `if o.mount_ok` フィルタであり、その性能ではない。⇒ **「サイレント最小で選択」という選択方法自体は「マウント不可」を価格付けしない**。判定基準 5 の `unmountable_gens=10` は単独で報告されたものであり、その比較には算入されていない。
⇒ **結論維持**：flags 未知ビットは「該当エントリをスキップ」を選択、ライターは認識不能な記帳行をそのまま新世代へと引き継ぐ。

⚠️ **実測により付随的に得られた、元の問いには含まれていなかった 3 つの結果**：

1. **`Ignore` は仕様上もサイレントである** (`silent_by_spec=1`) —— 不変条件リスト内に「レコードが誤ったエンコーディングで解釈された」を検出する条項が**存在しない**。
   これは `Ignore` が他所で負けたのではなく、それが負けたセルを**誰も監視していない**ためである。配置先 C156 (レコードエンコーディング誤読に不変条件がない)。
2. **`Skip + NoCarry` は 2026-09-06 の実装下でのみサイレントである** (`silent_today=1` かつ `silent_by_spec=0`)：
   失われるのは書き込み可能ヘッドの「inode 番号水位」行であり、I-9.11 (書き込み可能クローンヘッドは水位行を保持) が真正面からこれに対峙しているが、**未実装**。
   ⇒ I-9.11 (書き込み可能クローンヘッドは水位行を保持) の実装は `NoCarry` を「サイレントエラー」から「検出可能なエラー」へと格下げするが、
   それを正しくするわけでは**ない** —— `Carry` は依然として 0 行喪失対 2 行喪失である。
3. **暴露面は極めて狭い**：2026-09-06 現状確認時点でツリー次元を伴う統計量は 1 つのみであり、書き込み可能ヘッドに対してのみ保守
   ⇒ v1 未知のツリーは、**書き込み可能ヘッドでなければ 1 行も存在せず**、`NoCarry` はそれに対して無害。
   ⚠️ **このセルは統計量リストの拡張に伴い拡大する**：ツリー次元を伴う統計量がさらに 1 つ追加されれば、`NoCarry` の喪失量はツリー数に伴い増加する。

```

**出处 `.claude/kb/experiments/112-旧ライターが未知のツリーに遭遇した場合.ja.md:121-126`（整段抄，未转述）**

```markdown
### 計測条件

純粋算術 / 離散シミュレーション、I/O なし。**同一バイナリを N 回実行すれば必然的に完全に同一**となるため、
ラウンド数は根拠の強度を構成せず、根拠の強度は変異テストと絶対値を固定するアサーションに由来する
(`.claude/singlefs-ai-sop/rules/test-discipline.md`「単一観測はカウントされない」の決定論的例外)。

```

**出处 `.claude/kb/experiments/112-旧ライターが未知のツリーに遭遇した場合.ja.md:127-134`（整段抄，未转述）**

```markdown
### 影響を受ける意思決定

| 意思決定項目 | 関係 | 振り返り |
|---|---|---|
| D8 (コアインデックス構造) 確定項目 8 | 準拠 | 2026-10-11 影響なし：表現のみ修正 (時間表現、位置の指示、一人称)、結論、数値、成果物は不変 |
| D5 (スナップショット / 領域記帳メカニズム) | 影響なし | 2026-10-11 影響なし：表現のみ修正 (時間表現、位置の指示、一人称)、結論、数値、成果物は不変 |
| D15 (フォーマット凍結ポリシー) 確定項目 2 | 影響なし | 2026-10-11 影響なし：表現のみ修正 (時間表現、位置の指示、一人称)、結論、数値、成果物は不変 |

```

**出处 `.claude/kb/experiments/112-旧ライターが未知のツリーに遭遇した場合.ja.md:135-146`（整段抄，未转述）**

```markdown
## 履歴バージョン

### 2026-09-06

#### (その一)
- ファイル作成、判定基準および失敗条項を確定、未実行。根拠：D8 (コアインデックス構造) 確定項目 8 収束リストの残り 2 項目、
  2026-09-06 ユーザーが「採用」決定後、検証・対抗・比較を順次実施することを要求。

#### (その二)
- 実行完了し結論を書き戻し；3 つの陽性対照と 1 つの陰性対照がすべて成立、9 件の変異がすべて検出。


```

**出处 `.claude/kb/experiments/112-旧ライターが未知のツリーに遭遇した場合.ja.md:137-146`（整段抄，未转述）**

```markdown
### 2026-09-06

#### (その一)
- ファイル作成、判定基準および失敗条項を確定、未実行。根拠：D8 (コアインデックス構造) 確定項目 8 収束リストの残り 2 項目、
  2026-09-06 ユーザーが「採用」決定後、検証・対抗・比較を順次実施することを要求。

#### (その二)
- 実行完了し結論を書き戻し；3 つの陽性対照と 1 つの陰性対照がすべて成立、9 件の変異がすべて検出。


```

**出处 `crates/singlefs-checker/invariants/05-空间记账.md:1-39`（整段抄，未转述）**

```markdown
## I-3 空间记账 —— 类

**依赖 D5（快照 / 空间记账机制）。**
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-3.1 | 已分配统计对得上 | 已分配空间统计 == 实际遍历所有引用得到的和 | 已实现（池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；崩溃点重放每个崩溃状态都判） ⚠️ **引用按映射位置记**（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3 与第七节用户定案「重构」，D19（块指针的结构与宽度预算） 已定项 5 / 8）：豁免三类（映射树自己的节点、树表单元、实例表单元）之外的码 1 / 2 / 3，走读按那一版的中央映射认位置，指针里的位置条目只当提示——引用集合记映射位置，提示过期（单元被搬走、源槽已释放）不进引用；提示与映射条目在同一序号上带的整单元校验和不同的那条指针不当过期的提示，照它自己的位置条目记（`walk::CentralMappingLocationsOfAVersion::locate`，实现员 2026-10-15 推的读法，交主 agent 认）；合法搬迁过的镜像在 `crates/singlefs-harness/tests/checker_relocated_images_judged_by_the_mapping_position.rs`。⚠️ 「源记录还在册、目的槽没有记录」那一形（映射换位、分配记录一条不动）按槽数加总对得上，这一条看不见，由 G3 的新判 N2 判红：I-3.15（未释放记录罩住的单元由映射指在这一槽），2026-10-15 实现。 ⚠️ **checker 读法（2026-09-14 用户收尾弹窗定甲）**：「实际遍历所有引用」按根环里全部有效根的引用取并集——新池新建文件之后第 0 版树表单元只被更早的根引用、仍占着空间（已释放、在 defer 队列里），只按最新根算会少 1 槽。**「有效根」= 回退候选集里的根**：按最新根指着的实例表有效（无那个实例的行，或有行 (i, Ti, Wi) 且 T ≤ Ti）且 txg ≥ F_生效（D16（发布语义） 已定项 1「生效」）。遍历的是最新根与不被抛弃、不低于 F 的有效根（RGall：候选集不再滤一圈窗口，环上有洞时洞那一槽里留着的上一圈的根只要不被抛弃、不低于 F 也走；代码 `crates/singlefs-checker/src/walk.rs` 的 `check_pool_image_given_the_container_capacity_knobs` 里 `candidate_indexes` 那一段）。F_生效 由 checker 自己算：取各块盘上最新的有效根所带 F 的最大值，再与池里自证过、fsid 与本池相同的系统配置槽带的 F 取大（`walk::check_pool_image` 里的 `effective_rollback_floor`，C556（checker 与崩溃点重放不读系统配置里的 F） 已还清）——被抛弃时间线的根引用的单元由影子账隔离、F 之下的根引用的已释放单元已可再分配，都不在当前账里（里程碑「覆盖写、释放、回退与复用」步 4 / 步 5，`crates/singlefs-checker/src/walk.rs`）。⚠️ **最新根带的 F 低于 F_生效（两者不等）时这一格不判、报「不适用」（实二的读法，主 agent 2026-09-26 暂认，最终代码三方专攻这一格）**：先把新 F 写进系统配置之后、带新 F 的根落盘之前，或带新 F 的根全读不出，最新根的「已分配」是按它自己那个较低的 F 记的，还算着 F_生效 之下那些根引用的槽，与按 F_生效 取的候选集并集不可比，照判会在合法状态上红；另一种读法（并集按最新根自己的 F 取、别的几条按 F_生效 取）条款没选。坏镜像 `checker_known_bad_images.rs` 的 `every_root_carrying_the_raised_floor_unreadable_keeps_the_candidate_invariants_green_because_the_checker_reads_the_system_configuration_floor` 钉住这一格报不适用，`crates/mutations.tsv` 里把这道例外拿掉的那一行（实二）让它红。F_生效 之下仍在 defer 的槽：F_生效 之下、没被抛弃的根引用着，候选集那一遍没走到，而最新根的账里罩住它的记录是已释放、释放代高于回收门槛的（还在 defer 里，分配器还不许回收；回收门槛 = max(F_生效, 环里最旧有效根)，D16（发布语义） 已定项 1「可再分配」那一行），并进遍历得到的和（收口表第 43 行取丁-defer，用户 2026-09-28 定，被攻过一轮，判决 `research/prompts/abandoned-floor-r1-main-verification.md` M1；`walk::deferred_slots_referenced_only_below_the_floor_per_device`，用例 `raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_the_allocated_statistic.rs`）；⚠️ 它会不会盖住「该回收而没回收」那一类错，没拿变异试过。隔离的记录：已分配而没有任何根引用的记录，按单元豁免——它罩住的那个单元只要有任何一份副本 checker 自己读出来读不出或校验和对不上，这个单元在每块盘上的那几条记录都豁免；每一份都读得出且对得上，照旧判违例（用户 2026-09-25 定「核出坏的才豁免」，按单元的读法是主 agent 同日定的，D19（块指针的结构与宽度预算） 已定项 5 的隔离）。 中央映射条目指的物理范围算进遍历得到的引用，与树里指向同一单元的引用同 (设备, 起点槽, 跨度) 时算一个（2026-09-27，`m2-rev-b1-implementer-report.md` 第六节第 2 条）。由记录施加出来的那一版照 D23（journal 的角色与格式） 已定项 15，走那条记录新根段里的实例表指针与分配记录树根（`walk::walk_version_applied_only_by_records`；用户 2026-10-02 定新根段回到三字段，实例表指针与分配记录树根随新根段换成那次发布的）。⚠️ **没量的前提**：「隔一条也是同一张实例表」是按「一个实例写行之后每一版照抄同一张实例表」推的，没量（`m2-rev-b2-implementer-report.md` 第七节第 2 条）；量它是 C699（取号跳过改认出生代号） 的第一步。 ⚠️ **钉最旧 txg 的代价（用户认的）**：读坏槽与读回全零的槽把回收门槛钉在它可能住过的最旧 txg（D16（发布语义） 已定项 1「根槽这一次读坏」），这样的合法盘面上 I-3.1（已分配统计对得上） 红在记账多的一边，到钉解开为止（这个进程写掉那一槽，或进程内记根时重读读出有效根、读出被抛弃的根，`crates/singlefs-core/src/allocator.rs` 的 `RootRingOccupant` 文档注释）；`crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs` 三条慢档用例在第 10h 批改成只许红在记账多的一边（用户 2026-10-04 定钉最旧 txg、2026-10-05 定全零槽钉门槛而不并进上限，`records/2026-09-30-里程碑三开工第一轮.md` 第 10b、10c 批）。 ⚠️ **钉落盘之后的并法（C773（读坏与全零根槽的回收门槛钉只在内存里） 定案，用户 2026-10-20 认；checker 未实现，随 C773 实现批，GPU 判器那一侧没有现成件）**：根记录带回收门槛 t 与钉槽 p 之后（I-7.18（根记录的回收门槛等于现算的门槛）、I-7.19（钉着的槽不住本实例写的根）），这一格只把「钉带出来的那一截」不看引用并进遍历得到的和，被抛弃根引用着的槽不并（C684（双故障形下已分配统计该不该并被抛弃根的槽） 保持「不并」，那一形照旧红在记账多的一边）；最新根下面没有记账树时照旧报不适用（`walk.rs` 今天已是）。「钉带出来的那一截」两腿的算例都取释放代落在 (t, 不带钉现算的门槛] 的已释放槽（推的）；checker 要走被抛弃根才分得出「被抛弃根引用着」，底子是 `walk::judge_blocks_referenced_by_abandoned_roots`。出处：根环三方第二轮判决 `research/prompts/m3-close-rootring-r2-main-verification.md` 第二节 Y4 那一行，第三轮判决 `research/prompts/m3-close-rootring-r3-main-verification.md` 第二节 C773 那一行之后的格式件补漏段。 |
| I-3.2 | 已停用·方向反了 | **此编号不再使用**——它写成「≤ 引用它的**最新**快照的代号」，方向反了：紧的约束是最**旧**的那个。旧写法被 I-3.5（引用区间的精确性） 的子判据 ② 严格蕴含，且检不出任何一类真实错误 | 不适用 |
| I-3.3 | 已停用·归属未定义 | **此编号不再使用**——「所属快照」未定义，而两种归属约定给出不同的比较符。由 I-3.6（deadlist 紧界） 取代 | 不适用 |
| I-3.5 | 引用区间的精确性 | **引用区间的精确性**。**内容**：对任一块 b 与任一根 R（快照根或活头根），**「b 从 R 可达」这个可达性谓词，必须等于记账所用的那个谓词**——两者是同一个定义，不是两份。**当前形态下该谓词的表达式**是 `birth(b) ≤ R.txg < death(b)`（[decisions.md](../../../.claude/kb/decisions.md) D5（快照 / 空间记账机制） 形态）；**换快照模型时表达式要重写，内容不变**。可判定形式（与 I-3.1（已分配统计对得上） 共用同一趟全盘遍历）：按 txg 升序枚举全部快照根与活头得 `Reach(b)`，则 ①② 合成一条**出现规则**：取回退候选集（I-7.4（近 K 代块未被复用） 用的那一份，加由记录施加出来、根槽从没落盘的那几版）按 txg 升序，同 txg 的几条并成一条（R⁻ 侧引用集合取并，R 侧逐条各判一次），相邻两条 (R⁻, R) 为一对，逐对判。R 引用而 R⁻ 不引用的每个落点 b（第一次出现与再次出现都判，`Reach(b)` 在 txg 序上是否连续不再单判）满足 (a) `birth(b) > R⁻.txg`；或 (b) b 落在 R 树表里某棵用户可见树（今天 extent 树、inode 树，`deleted_inodes` 树根非全零之后加它）的子树里，那棵树的根指针非全零、出生 txg 严格小于 R⁻.txg，且那棵子树里每个读得出的单元头的诞生代号都不大于那个出生 txg（读不出头的单元不计入）。`birth(b)` 取 b 的单元头里的诞生代号；b 自己的头读不出时这一个落点不判，读不出归 I-1.1（块头自述逻辑地址） 与 I-2.1（校验和与内容匹配）。**缺口**：R.txg ≠ R⁻.txg + 1 时这一对之间有洞，分两种处置——最新根的实例表里有一行 (R⁻ 的实例, T = R⁻.txg)、R 的实例比 R⁻ 的大、两者之间每个实例的行都是 (j, 0, 0)，即实例表行证明 R 所在的实例直接接在 R⁻ 那一版上的，这一对照常判（gap-row）；不是的，用户可见树子树里的落点不判，别的落点（树表、记账树、分配记录树、中央映射树的单元与实例表单元）仍照 (a) 判 `birth(b) > R⁻.txg`。R 或 R⁻ 的引用集合数不全（走读读不出，或走到没有条款的形状）时整对报不适用；没有一对判到任何落点时整条报不适用并带理由（同 I-4.9（非最新候选根引用的单元不晚于它） 候选集只剩一条根的写法）；③ 若 b 不被活头引用，则 b **恰好**出现在一个 deadlist 中，且该 deadlist 的归属区间 = `(Reach(b) 中最晚者.txg, 其下一个根.txg]`；④ 若 b 被活头引用，则 b 不出现在任何 deadlist 中；③ 的逆向（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S5 ①）：deadlist 中每一条 b 都不被活头引用，且 `Reach(b)` 非空（某个活快照引用它）或 b 在未完成销毁意图的待放集里（意图 (头, lo = S.txg, hi, P = prev(S).txg) 下纪元落在 [lo, hi) 里、birth > P 的条目，D5（快照 / 空间记账机制） 已定项 12）。③ ④ 与逆向里的「快照」只算活快照：带销毁中记号（码 14 条目 flags 位 0，同一判决 S4）的快照从写意图那一刻起不再钉住块（D5（快照 / 空间记账机制） 已定项 20 的例外句），不进 `Reach`；「归属区间」按纪元认：快照 X 那一份 = 纪元落在 [prev(X).txg, X.txg)、活头那一份 = [prev(头).txg, ∞)，b 该在的那一份是引用它的最晚那个活快照之后的那一侧（纪元落在 [那个快照的 S.txg, 链上下一个活快照的 S.txg 或 ∞)）。b 按单元认、写成中央映射 key（活快照与活头两棵用户可见树下面每个单元的 key 从指针现算，deadlist 条目里就是 key）。⚠️ **射程**：缺口的来源是新实例第一条根的 txg 取根环与 journal 里见过的最大 txg 加一（`crates/singlefs-core/src/mount.rs` 取号那一步），所以写行那次发布的记录已落而根槽没落、多记录发布只落一部分之后再可写挂载，必出现缺口；另有恢复丢掉尾巴记录、根槽写失败重发跳一格（今天的代码走不到，随失败处置在后面的里程碑做）、根槽读不出、根槽被环盖掉而记录也被盖掉（短 journal 环）。**盲区**：缺口右边那一对里，落在用户可见树子树里的新写单元（新叶）的诞生代号被改小到 R⁻.txg 以下，判不出；整条用户可见链（树根指针出生 txg 与树里单元的诞生代号）一起改小到 R⁻.txg 以下的坏镜像同样判不出：盘上与一次合法回退逐项同形，判别子是回退那次发布写的谱系记录（D18（块里携带什么信息） 已定项 24），`crates/` 今天没有。**谱系记录落地之后**，(b) 收成「R 的谱系树里有一条记录 (T_rb, T_old)，R⁻.txg < T_rb ≤ R.txg，且那棵用户可见树的根指针出生 txg ≤ T_old；谱系树读不全时该对不适用」，这一形与缺口条款并存（`research/prompts/m3-birth-floor-r2-main-verification.md` 第二节第 9 行）。**判别力**：诞生代号改小到恰好等于 R⁻.txg 的新写单元必须红；最新根的 inode 树指回已丢的旧叶、分配记录与映射条目一起改平必须红；回退根槽被环盖掉、记录也盖掉之后的合法镜像不许红。**推翻条件**：任一合法镜像（池级 checker 与崩溃点重放其余判定都不红）上红。 | 部分实现：出现规则（TF、β、gap、gap-row、n3）已实现（落点折成单元先按中央映射 key 折：搬迁前后两个落点是同一个单元，不算后一版新出现的，判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3；`walk::Walk::note_references_of_a_pointer_followed_by_the_walk`），池级 checker `crates/singlefs-checker/src/walk.rs` 的 `judge_the_appearance_rule_of_adjacent_candidate_versions`（`IMPLEMENTED_INVARIANTS` 下标 65；坏镜像在 `crates/singlefs-harness/tests/checker_judges_reference_intervals_and_every_new_root_segment_field.rs`；变异在 `crates/mutations.tsv` 里「第8批 I-3.5（引用区间的精确性）」那几行）；③ ④（deadlist 那两条）连同逆向已实现（2026-10-15，里程碑四只读快照 checker 侧，判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S5 ①）：并在下标 65 这一条下，池级 checker 的 `walk::judge_snapshots_and_deadlists_of_the_versions` 在回退候选集每一版（加由记录施加出来的那几版，树表单元相同的只判一次）上判：④ 每一条钉的单元活头不引用，逆向每一条钉的单元有活快照引用或在待放集里，③ 活快照引用而活头不引用的每个单元恰好一条、头是活头、纪元落在它该在的那一侧；一版里一个快照都没有、deadlist 一条都没有时这三格没有对象、不判；树表、活头两棵树、deadlist 树或活快照的两棵树读不全时报射程里判不了。纯判法的单测与必红世界在 `crates/singlefs-checker/src/walk.rs` 的测试模块（④、逆向、③ 缺一条与挂错一侧）与 `crates/singlefs-harness/tests/checker_snapshot_space_accounting_must_red_worlds.rs`（写路径建两个快照之后往 deadlist 里放一个活头还引用的块、放一个谁都不钉的块）；变异在 `crates/mutations.tsv` 里「只读快照 checker 侧」那几行。GPU 判器内核那一位判出现规则，也判 ③ ④ 与逆向：扫描内核的引用与快照那一截（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl` 的 `judge_snapshots_and_deadlists_of_the_versions`）照 `singlefs_checker::walk` 的同名函数判、置位，对拍逐位比，下标 65 不在对拍不比的表里（`crates/singlefs-checker-tier/src/crash_judge_conclusions.rs` 的 `POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS` 是空表）。出现规则取按树继承下界加 β 出现规则（用户 2026-10-02 定 ① 取 β）加缺口处置；缺口处置用户 2026-10-03 定 gap 加 n3（判决里的局部标号，只放过用户可见树里的单元）加 gap-row 两样都加。**被攻过几轮**：按树继承下界加 β 加整对不适用的缺口，在合法镜像上两轮没红（各一次抽样）；gap 加 n3 加 gap-row 的合成形、同 txg 几条并成一条、`deleted_inodes` 非全零才算继承、读不出头的单元不计入这三处措辞、谱系记录落地之后的区间形，都是零轮，终轮不再开轮，实现时再攻。实现挂在池级 checker 的主走读上（每版落点集合快照、用户可见树标签、压得住的判、新判定函数、位置键折叠、`IMPLEMENTED_INVARIANTS` 加一条、用例与变异），I-3.1（已分配统计对得上） 在短环上洞里的误红归下一批调查，与这一条共用同一个「候选集有洞」判法（`research/prompts/m3-birth-floor-r3-main-verification.md` 第二节第 4、5 行与第四节）。② 的上界半（`birth(b) ≤ R_first.txg`）另立 I-4.9（非最新候选根引用的单元不晚于它）（`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 1 行）。⚠️ **射程注（2026-09-13）**：跨头共享的块（稀疏旁表计数 > 1，D5（快照 / 空间记账机制） 已定项 6）不由 deadlist 单独管，判据 ③ / ④ 对它们不适用；它们的生死由旁表计数与各头的可达性合起来判，那条不变量还没立。 |
| I-3.6 | deadlist 紧界 | 任一 deadlist(S) 中的块满足 `birth ≤ prev(S).txg`（约定 B 下的紧界）。判别力：抓「kill 时该走直接释放却进了 deadlist」这一类（泄漏方向）。S 是活快照或活头，一条条目归哪一侧按纪元认（快照 X 那一份 = [prev(X).txg, X.txg)、活头那一份 = [prev(头).txg, ∞)，D5（快照 / 空间记账机制） 已定项 12）；`birth` 取条目里映射 key 的出生 txg 段。**豁免**（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S5 ②，攻方 H4-2）：落在未完成销毁意图 [lo, hi) 里、birth > P 的条目（写意图之后、它被放掉之前，前驱已传给下一侧）不判；「续做点之前不许再有这样的条目」那一半没有输入（续做点不存、条目被放掉就是进度，判决第七节岔路 2 的用户定案），不判。 | 已实现（2026-10-15，`crates/singlefs-checker` 的 `walk::judge_snapshots_and_deadlists_of_the_versions` 里每一版判；`IMPLEMENTED_INVARIANTS` 下标 93；写路径写出的 deadlist 上的必红世界「生在快照之后、该直接释放的单元挂进 deadlist」与 I-3.5 ③「只被快照引用的单元在 deadlist 里少一条」在 `crates/singlefs-harness/tests/checker_snapshot_space_accounting_must_red_worlds.rs`）：一版里 deadlist 一条条目都没有时判一次、成立（同 I-9.24 空队列报成立）；条目的头不是这一版的活头、纪元落不进恰好一侧（前驱链断了，归 I-9.25）的那几条报射程里判不了；树表、deadlist 树读不全报射程里判不了。必红世界与豁免去掉要红的合法态在 `crates/singlefs-checker/src/walk.rs` 的测试模块（birth 晚于活头前驱的块进了 deadlist；写销毁意图与最后一批之间的待放条目）；变异在 `crates/mutations.tsv`「只读快照 checker 侧」那几行。盘上的必红世界要等 deadlist 写者落地（里程碑四丙 2）之后补。GPU 判器内核不置位、对拍不比。 |
| I-3.4 | 可用空间扣待删占用 | 待删中 inode 没释放的单元计在记账「已分配」里（释放只进 defer），记账标签 4（待删除但意图未完成占用）恒 0，可用空间统计因此已扣除待删占用；准入式子另扣环窗口常驻项（D28（挂载期承诺量） 已定项 1）。来源 `research/prompts/m3-delete-r3-main-verification.md` 第三节判标签 4 那一格，被攻过零轮 | 已实现（池级 checker `walk::check_pool_image`，函数 `judge_pending_deletion_occupancy_of_a_version`；坏镜像两份：标签 4 那一行的值非 0、整行缺失，登记在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `.chain`，改法在 `crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs`；GPU 判器下标 70，扫描内核判）。池级 checker 只判「标签 4 恒 0」：判的每一版里，走全了的记账树都要有标签 4 那一行且值为 0，缺行判红；另两句不在这一行判：计在「已分配」里归 I-3.1（已分配统计对得上）、I-3.11（已分配减 defer 等于最新根走读），可用空间已扣待删占用归 I-5.2（空闲统计对得上），即空闲加已分配等于容量；判哪几版、配对与读不全的报法照 I-9.5（删除队列里的号仍在树里） 的共用判法；出处 `research/prompts/m3-delete-checker-r1-main-verification.md` 第二节） |
| I-3.7 | 活快照可枚举 | 盘上存在一个可枚举的持久结构，其元素恰为当前全部活快照（既不多也不少）。这个结构是树表里不带销毁中记号的码 14 条目集合（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S1；带销毁中记号的照同一判决 S4 算销毁中、不算活快照，最后一批同一次发布删掉它的码 14、15）。可判定形式：① 每一版里每一个活快照都是这一版之前的一次捕获：诞生 txg（S.txg）不大于这一版的 txg，两条根指针的出生 txg 不大于 S.txg；② 回退候选集里 txg 相接的两版之间，活快照只经建快照那次发布多出来（诞生 txg 就是后一版的 txg）、只经写销毁意图（带上记号）与最后一批（带着记号的那一条没了）少下去，同一个快照的诞生 txg 不变；txg 不相接的两版之间不判。第二来源前驱链（码 14 的前驱 = 排在它前面那条的诞生 txg，活头 inode 条目的前驱 = 最后一条的）由 I-9.25 判，这一条不重判。 | 已实现（2026-10-15，`crates/singlefs-checker` 的 `walk::judge_snapshots_and_deadlists_of_the_versions` 每一版判 ①、走完判 ②；`IMPLEMENTED_INVARIANTS` 下标 94；② 依赖写意图与最后一批各是一次独立发布：core 的 `transaction::publish_snapshot_destruction_intent` 只写记号、`publish_snapshot_destruction_step` 能放的条目不多于一批时（零条也是）才是最后一批、同一次删那一对，两者从不并成一次——2026-10-15 核过）：一个快照都没有的一版 ① 判一次、成立；树表读不全报射程里判不了。必红世界在 `crates/singlefs-harness/tests/checker_snapshot_space_accounting_must_red_worlds.rs`（写路径建三个快照之后最新那一版的树表去掉中间那一对：少一条；加一对诞生 txg 是旧 txg 的：多一条）与 `crates/singlefs-checker/src/walk.rs` 的测试模块；变异在 `crates/mutations.tsv`「只读快照 checker 侧」那几行。GPU 判器内核不置位、对拍不比。 |
| I-3.8 | 实例表行唯一且低于挂载根 | 实例表 kind 0 行按实例代号唯一；行只在回收条件成立后删（整轮清扫对池中每一个落点都得出判定、其中没有该实例的未发布单元、那次清扫里没有读失败 ∧ 全部设备在线 ∧ 根环里没有该实例发布的根、且根环每个槽都读成功）；行的实例代号 < 挂载根实例（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2） | 已实现（`crates/singlefs-checker/src/walk.rs` 的 `judge_instance_table_rows`：行按实例代号唯一、每行实例代号低于最新根的实例（记录形态归 I-3.14（实例表每片记录形态合法））；最新根的实例表第 0 片解不开时报不适用，理由点名这一片解不开、行没收全；回收那一半没有输入：删行由整轮清扫做（`crates/singlefs-core/src/instance_table_sweep.rs`，可写挂载末尾一支与后台一支），checker 不判删行的条件） |
| I-3.9 | 释放代落在停止引用它的那一格区间里 | 任一带已释放标志的分配记录，它的释放代必须落在区间 `(最后一条仍引用这个落点的有效根的 checkpoint_txg, 最早不再引用它的有效根的 checkpoint_txg]` 内；根环没有洞时这个区间只有一个值，等同于「等于最早不再引用它的那条根的 txg」。若每一条有效根都还引用它，它不许带已释放标志（D3（空间分配） 已定项 7（释放时条目不删、改写 value：跨度段最高位置 1、后 8 字节从分配代换成释放代）；C374（释放代与树表诞生 txg 只有验收断言盯着））。环上有洞时（被施加的 journal 记录做的释放写着一个根槽从没写过的 txg，实测 2026-09-18 残留记录那条崩溃点重放流的 12 个状态）按区间判，按「等于」判会在合法镜像上判红；已经回收、还没被复用的已释放记录跳过不判（见证它释放的根已不在候选集里，定位不了那个区间），全是这一类时整条报不适用并带理由。判别力：把发布 B 那次的释放代从 4 写成 3 必须红（最早不再引用那十个落点的有效根是 B、txg 4）。 | 已实现（2026-09-18，`crates/singlefs-checker` 的 `walk::judge_release_generation_and_tree_table_birth`；一条根引用的落点按它自己的中央映射认（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3：搬走之后源槽在搬迁那一版起不再被引用，释放代落在搬迁那一格；`walk::RootReferences::note_located_by_the_central_mapping`）；从 deadlist 放掉的块的必红世界（释放代改成写意图那次的 txg）在 `crates/singlefs-harness/tests/checker_snapshot_space_accounting_must_red_worlds.rs`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `each_release_generation_and_tree_table_birth_bad_image_reddens_only_its_own_invariant_after_the_overwrite`；崩溃点重放每个崩溃状态都判）。**判法收窄在实现途中**：今天任一候选根走不全就整条报不适用；收窄成只在比最后引用它的那条根晚的候选根都走得全时才判这一条已释放记录，攻方造出的对照格镜像（抬 F 的空发布之后再覆盖写，把最后一次空发布那条根的树表翻坏）钉进用例（`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 10 行）。**从 deadlist 放掉的块**（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S5 ③，攻方 H5-3）：释放代落在（写意图 txg, 最后一批 txg]。落法（2026-10-15）：「一条根引用的落点」算进它活快照的两棵树（码 14 照 inode 树、码 15 照 extent 树往下数）与它中央映射树叶里每条映射条目指的单元（被钉单元的映射条目留在映射里，放掉它的那一批同一次发布删条目，同一判决 S3、S4），带销毁中记号的快照的两棵树不算（D5（快照 / 空间记账机制） 已定项 20 的例外句）；于是一个从 deadlist 放掉的块，最后还引用它的是放它那一批之前的那条根、最早不再引用它的是那一批的根，取到的区间落在（写意图 txg, 最后一批 txg] 里，比判决那一句更紧。照数销毁中快照的树，那几个块在最后一批之前的每条根上都还算引用，合法态红；不数映射条目，写意图那条根之后就不再算引用、释放代晚于区间，合法态红。变异在 `crates/mutations.tsv`「只读快照 checker 侧」那几行（单测钉「销毁中快照的两棵树不记引用」）；盘上的必红世界（从 deadlist 放掉的块释放代写成写意图那一次的 txg）要等 deadlist 写者与删快照落地（里程碑四丙 2）之后补。 |
| I-3.10 | 已分配记录的分配代等于它罩住的单元的诞生代号 | 任一**未带已释放标志**的分配记录，它的分配代（D3（空间分配） 已定项 3：`value = 分配代`；盘上编码见 D3（空间分配） 已定项 7）等于它罩住的那个单元头里的**诞生代号**（D18（块里携带什么信息） 已定项 7 的类身份段）——记录与单元是同一次发布写出来的，两个数因此同源。⚠️ **射程**：① 只管未释放的记录，已释放的那一半归 I-3.9（释放代落在停止引用它的那一格区间里）；② 记录罩住多个槽时（数据单元跨两槽）取起点槽那个单元头；③ **那个槽上读不出可用的单元头时这一条没有对象**（报不适用，不报违例）——读不出本身由 I-1.1（块头自述逻辑地址） 与 I-2.1（校验和与内容匹配） 管；④ **只读回退候选集里那几版的分配记录**（候选根的分配记录树、树表 0 条那一版根记录直接持有的那一片、由记录施加出来的那几版），**再加下一次挂载会先施加的那一版**：下一次挂载的恢复照 D23（journal 的角色与格式） 已定项 14 的重放规则施加的**第一次**发布（与 I-4.1（重放后树可遍历） 同一份重放，所选根取最新根），取它末条那一份新根段里树表指着的分配记录树；一次都不施加、锚点认不定、所选根的 txg 没有后继时没有这一版，前缀停在后面某次发布的点名单元上时仍取第一次（`walk::version_the_next_mount_applies_first` 与 `walk::replay_the_next_mount_would_do`；认法依据 `research/prompts/m3-formal-code-b36-r1-main-verification.md` 第二节 P1 与第三节第 1 条，取末条那一份依据 `research/prompts/m3-formal-code-b35-r1-main-verification.md` 第二节 P1；主 agent 2026-09-24 按代码三方 `research/prompts/m2-wave3-code-r1-main-verification.md` 第四节第 2 条定接这一步，被攻过零轮；只接这一步——恢复连着施加几次发布时第二次及以后那几版读不到，欠在欠账表）；不读被实例表判抛弃的根与低于 F 的根：低于 F 的根那棵账里未释放的记录罩住的槽，在 F 抬过之后可以被合法回收、换成诞生代号不同的单元，照「任一」去判会在合法镜像上红（主 agent 2026-09-23 定，被攻过零轮）。⚠️ 「再加下一次挂载会先施加的那一版」那一段的同源前提，靠 D16（发布语义） 已定项 7 持久顺序里单元与记录之间那道屏障撑着：拿掉那道屏障、又复用回收槽时，记录已持久而复用槽上仍是旧单元，恢复按点名验证拒施加、结局正确，这一条却在那个槽上红（2026-09-28 主 agent 复跑 floorreuse4：删屏障 271 个状态红 48、带屏障 73 个状态 0，`research/prompts/k3-9-barrier-r1-main-verification.md` 第一节）；用户 2026-09-28 定屏障维持现状、checker 不改，欠在 C592（单独拿掉单元写与记录间那道屏障今天没判据会红）。**它拦的是什么**：复用一个已回收的落点时改写了那条分配记录、却把上一次的代留着（2026-09-22 增补 3 第 1 件的代码三方第一轮攻方腿造出这条变异，快档与大档都判不出，因为全仓没有一条不变量管已分配记录的分配代） | 已实现（2026-09-23，待代码三方与崩溃验证；池级 checker `walk::judge_allocation_generations_against_unit_births`，读法照射程 ④，同一条记录（盘、槽、代）只判一次，射程 ③ 按记录逐条跳过、一条都没判到才整条报不适用；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；崩溃点重放每个崩溃状态都判） 下一次挂载会先施加的那一版由 `walk::tree_table_pointer_of_the_version_the_next_mount_applies_first` 并进读集；它的坏镜像（发布 B 的记录已落、根槽没落，B 那棵分配记录树里一条未释放记录的分配代改成诞生代号 + 1）是 `checker_known_bad_images.rs` 的 `an_allocation_generation_past_its_unit_birth_in_the_version_only_its_journal_record_carries_reddens_the_allocation_generation_invariant_before_the_mount` |
| I-3.11 | 已分配减 defer 等于最新根走读 | 逐盘：被判镜像上记账树里「已分配」那一行（记账第 1 项）减「defer 待释放」那一行（第 5 项），等于从最新有效根走读到的、这块盘上被引用的槽数。统计量的标签与行形态是 D5（快照 / 空间记账机制） 已定项 4 与已定项 10 的那几行，读的是镜像里的记账行、不读内存里的分配器；走读与 I-3.1（已分配统计对得上） 同一个走法，只取最新有效根。⚠️ 它补的是 I-3.1（已分配统计对得上） 与 I-5.2（空闲统计对得上） 都够不着的那一种：写者把活单元记成已释放、放进 defer，已分配与空闲两个和照样对得上（alloc-basis 第三轮判决 `research/prompts/alloc-basis-r3-main-verification.md` 的 Z10）。⚠️ **判别力自证在造出来的基底上做**：E156（alloc-basis 四条岔路的代价数） 第 4 次重跑在造出来的基底 β_syn（第 5 项为 0）上再记 1 槽 defer，I-3.11（已分配减 defer 等于最新根走读） 判红、去掉「减 defer」判绿（`e156-alloc-basis-counts-2026-09-26-r2.out` 的 `E7RESULT name=q7c1_not_subtracting_defer basis=beta_syn k=1 real_check_is_red=true without_subtracting_defer_is_red=false flips_red_to_green=true`）。可达状态里第 5 项能不能为 0 没量到：挂载时回退（回退写行那次发布）没有了，同一份产物里挂着回退那一版第 5 项是 356 槽（`E7RESULT name=u13a_rollback_to_oldest_candidate overwrites=72 target_txg=52 txg_before=75 txg_after=76 instance_before=1 instance_after=1 instance_table_rows_before=0 instance_table_rows_after=0 item1_slots=374 item5_slots=356 i311=holds structure_holds=true`）。隔离的记录：已分配而没有任何根引用的记录，按单元豁免——它罩住的那个单元只要有任何一份副本 checker 自己读出来读不出或校验和对不上，这个单元在每块盘上的那几条记录都豁免；每一份都读得出且对得上，照旧判违例（用户 2026-09-25 定「核出坏的才豁免」，按单元的读法是主 agent 同日定的，D19（块指针的结构与宽度预算） 已定项 5 的隔离）。 | 已实现（池级 checker `walk::check_pool_image`：逐盘判「defer 待释放 + 最新根走读 == 已分配」，最新根走读取 `walk_root(最新根)` 刚走完那一刻、与 I-3.1（已分配统计对得上） 同一个走法；最新根下面没有记账树时报不适用。坏镜像 `live_unit_recorded_as_deferred_reddens_only_the_allocated_minus_deferred_invariant`（盘 0 defer 行 +1 / −1 槽，只红它，I-3.1（已分配统计对得上） 与 I-5.2（空闲统计对得上） 照旧真被评估过且成立）；判别力自证在造出来的基底上 `on_a_synthetic_base_with_an_empty_defer_queue_one_deferred_slot_reddens_the_allocated_minus_deferred_invariant`（两盘 defer 挪回 0 之后再记一槽）；`crates/mutations.tsv` 5 条变异（判定恒真、不减第 5 项、`==` 放宽成 `≥` / `≤`、拿全部候选版本那一份走读比）；用户 2026-09-23 定采纳：alloc-basis 岔路单第 12 行（岔路 7）） |
| I-3.12 | 实例表行 T 低于引用它的根的 txg | 根环里任一自证过的根 R，它指着的实例表的每一行 (i, T, W) 满足 T < R 的 `checkpoint_txg`；链上某一片读不出的根不判。依据 D18（块里携带什么信息） 已定项 11（恢复写的行，所选根那个实例的 T 是重放之后那个根的 txg，中间实例写 0）与 D23（journal 的角色与格式） 已定项 14（新实例第一次发布的 `checkpoint_txg` 大于环里全部根与记录的号）。射程是每条根各自指着的那张表，不是只有挂载根那一张（那一张归 I-3.8（实例表行唯一且低于挂载根））；同一实例的行在两张表里是否逐字节相同不判，条款没有这一句。 | 已实现（池级 checker `walk::judge_instance_table_rows_against_the_roots_referencing_them`：每条自证过的根各沿自己的实例表链判，链上有一片读不出的根跳过，一行都没判到时报不适用；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 6 行） |
| I-3.13 | 记账行代段等于这一版的 txg | 走读到的任一版（候选根，以及由记录施加出来的那几版）的记账树里，每一行的代段（记账 key 偏移 14，8 字节）等于这一版的 `checkpoint_txg`，只判走全了的树；inode 号水位那一统计量的台阶行不判（D5（快照 / 空间记账机制） 已定项 2 射程）。依据 D5（快照 / 空间记账机制） 与 D16（发布语义） 已定项 6 的「记账代 = checkpoint 号」（每条根的记账树只装这条根那一代的行）。 | 已实现（池级 checker 走读里的 `walk_tree_table_entry` 对每一版的记账树判，版本含由记录施加出来的，只判走全了的树；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 5 行） |
| I-3.14 | 实例表每片记录形态合法 | 走读到的实例表每一片里：每条记录的 `kind` 只有 0（行）或 1（链指针记录）；链指针记录恰好是这一片的最后一条、片中间不出现，且自身合法（有无下一片 0 或 1、无下一片时位置指针全零、有下一片时不全零，且位置指针头部 MAC / nonce、算法类型、压缩算法码与压后长度、extent 偏移都是第一版的取值：有一段不是第一版的取值，这一片也解不开）；`kind` 0 行的 flags 字节为 0。依据 D18（块里携带什么信息） 已定项 11（`kind` 不是 0 或 1 ⇒ 这一片解不开；flags「第一版恒 0、非 0 拒收」；链指针记录「恒为一片的最后一条」，最后一条不是合法的链指针记录同样解不开，位置指针头部的第一版取值见 D19（块指针的结构与宽度预算） 已定项 3），用户 2026-10-06 定另立；头部那一形用户 2026-10-06 定算第五种形态（岔路单 `research/prompts/m3-batch10o-code-r1-forks.md` K2-1 那一行） | 已实现（`crates/singlefs-checker/src/walk.rs` 的 `judge_instance_table_page_record_forms` 判前三件、`walk_instance_table_chain` 里链指针记录那一格判最后一件，走到的每一片都判；不成立的一片解不开：行不收、链不往下走、候选集滤不全（I-9.29（同号只许一个出生代） 的门关）；只读不判的读法与引用集合那一遍同一个读法；GPU 走读内核同判，扫描内核的引用集合那一段与只读读法、恢复内核读实例表同判 flags；有下一片而位置指针头部有一段不是第一版取值的那一形：CPU 三处读法（主走读、只读不判的读法、引用集合那一遍）经 `crates/singlefs-checker/src/walk.rs` 的 `chain_record_view` 判，GPU 走读与扫描经 `crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl` 的 `w_chain_record_kind` 判，恢复内核读实例表同判；坏镜像两张（位置指针 MAC 不全零、extent 偏移非 0），在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`） |
| I-3.15 | 未释放记录罩住的单元由映射指在这一槽 | 回退候选集里每一版（连同由记录施加出来的那几版）的分配记录里每条未释放的记录：罩住的那一槽是根记录或父指针直接指着的（豁免三类——映射树各节点、树表、实例表各片——与根记录持有的那几样，含树表 0 条那一版由根记录持有、树 ID 0、不进映射的那棵分配记录树的各节点），成立；否则读那一槽上的单元头，头里的出生身份（类标签、诞生代号、写序实例、尾段；码 1 / 码 3 再加偏移 43 的出生树，码 2 的出生树随 C289（码 2 的出生树取哪个字段没写） 不比，同 I-1.11（映射 key 与单元头相符））与同一版中央映射里某把 key 相符、且那把 key 的位置条目指着这一槽（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3 的 N2 反查、G9 的「已死」「不一致」两类判红）。头用不了的那一格没有对象（读不出本身归 I-1.1、I-2.1）；隔离的单元（有一份副本读盘核出与位置项对不上，D19（块指针的结构与宽度预算） 已定项 5 硬规则 1，G9 单列的「隔离」类）不判，隔离判据读哪几份按被判这一版的事实分两支（判决 `research/prompts/m4-i315-r1-main-verification.md` 第三节 C1「乙二」，用户 2026-10-21 定案）：这一版中央映射里没有一条位置条目指着这一槽（这一槽有记录的每块盘上都没有）时，读这一版这一单元（起点槽、跨度）每块盘上未释放记录的那一份，头用不了的那一份也在里面、照同一个判据核（它对不上位置项）；有位置条目指着这一槽时只读上面头用得了、对不上映射的那几份。只看被判这一版的映射，不推别的版本里有没有人指着这一槽；一版的树表、分配记录树、中央映射树或实例表链读不全时那一版不判（射程里有一部分判不了） | 已实现（2026-10-15，里程碑四己 3：池级 checker `walk::judge_unreleased_allocation_records_against_the_central_mapping`；三形必红世界——只改映射不写副本、只写副本不改映射（只供测试的发布次序开关）、有记录没人引用（G9「已死」）——与合法搬迁之后成立的世界在 `crates/singlefs-harness/tests/checker_compaction_intent_and_relocation_must_red_worlds.rs`；「映射条目改指一份没人分配的副本、分配记录不动」那一形（按槽数加总 I-3.1 / I-3.11 看不见）在 `crates/singlefs-harness/tests/checker_cross_links_malformed_nodes_and_mapping_entries.rs`，主 agent 2026-10-15 定加 N2（`records/2026-10-13-里程碑四预开发.md`「己 2 交回」那一条）。映射 key 与单元头不符的几份坏镜像上与 I-1.11 一起红。隔离那一格照 I-3.1 那一份隔离判据（`walk::some_copy_of_the_unit_fails_its_location_entries`），实现员 2026-10-15 推的读法，交主 agent 认；读哪几份的两支 2026-10-21 照判决 m4-i315-r1 C1 改，必绿世界「一份头用不了、映射里没有 key、两盘记录留在已分配」在 `checker_compaction_intent_and_relocation_must_red_worlds.rs`，有 key 指着的那一形照红由 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `a_shared_leaf_container_born_after_the_rollback_target_with_a_garbled_copy_reddens_the_birth_and_appearance_rules_on_either_device` 钉着。GPU 判器下标 95：扫描内核 `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl` 的 `g_judge_the_unreleased_records_of_the_version_against_its_central_mapping` 同判、置位，对拍逐位比（`crash_judge_conclusions::POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS` 是空表）） |
| I-3.16 | 碎片度与全空段统计对得上 | 最新根记账里每块盘的 runs（D5（快照 / 空间记账机制） 已定项 4 第 10 项：单元区空闲槽的连续段数）与全空聚簇段数（第 11 项；D3（空间分配） 已定项 10 ①：段内 64 槽都没有未释放分配记录的整段数，C372（全空聚簇段数两份定义相反） 按记录口径收口）等于全扫重算的值，全扫不与分配器的增量维护共享代码（C413（碎片度与全空段统计没有不变量））。占着的槽取 I-3.1（已分配统计对得上） 那一份并集（全部走过的版本的引用、隔离豁免、F 之下仍在 defer 的）；那块盘上它的槽数乘 16 KiB 不等于记账的已分配（I-3.1 在这块盘上不成立）、最新根带的 F 低于 F 生效值、或最新根下面没有记账树时不判 | 已实现（2026-10-15，里程碑四己 3：池级 checker `walk::judge_fragmentation_and_empty_segment_statistics`，runs 由 `walk::free_runs_between_the_occupied_slots` 全扫重算；判别力自证「增量少加一次」（runs、全空段数各少一，只红这一条）与合法搬迁之后成立的世界在 `crates/singlefs-harness/tests/checker_compaction_intent_and_relocation_must_red_worlds.rs`）。全空段那一半在两种口径分得开的盘上照判（2026-10-15 起，里程碑四写段收尾）：分配器 `allocator::DeviceFreeMap::empty_segments` 改成按记录口径增量维护（defer 槽不算占用、只数整段），C372 代码那一半收口；崩溃恢复抛弃一条根之后那份干净镜像上两种口径分得开，`checker_known_bad_images.rs` 钉住它在那里判成立。GPU 判器下标 97：内核不置位、对拍不比，主 agent 合入时补登 |

> ⚠️ **I-3.5（引用区间的精确性）四条子判据双侧夹死 与 I-3.1（已分配统计对得上） 正交，两条都必须有。** I-3.1（已分配统计对得上） 是标量总账（已分配 == 遍历和），
> 而**归属错误对它不可见**——块被挂到错的 deadlist 上时，它仍然被恰好一处引用，总账依然平。
> 要检出归属错，必须有**按快照分箱的向量等式**。
> 这与 [decisions.md](../../../.claude/kb/decisions.md) D14（双轨（大小文件 / 持久临时）） 已定项 6 推导 D-3（多轨兼容可判据） 里「标量等式 → 按轨道分箱的向量等式，
> 判别力净增」是同一个形状。
>
> ⚠️ **数据丢失方向只在空间被重新分配之前可检出。** 该进 deadlist 却被释放的块，
> 一旦其空间被复用，症状就退化成 I-2.1（校验和与内容匹配） 校验和不匹配，即「损坏，原因不明」。
> **所以这类检查必须在崩溃点重放 harness 里紧跟着删除动作跑，不能只在全量里跑。**
>
> ⚠️ I-3.1（已分配统计对得上） 是**唯一允许全盘遍历的记账检查**——它是 checker，不是运行时路径。
> 运行时记账必须增量维护（D5（快照 / 空间记账机制）），checker 才用遍历来验证增量维护有没有出错。
> 这条对照关系是本工程验证记账正确性的核心手段。


```

**出处 `.claude/kb/decisions/19-block-pointer-structure-width-budget.en.md:1-13`（整段抄，未转述）**

```markdown
## D19 (Block Pointer Structure & Width Budget) — Settled

[中文](19-块指针的结构与宽度预算.md) · **English** · [日本語](19-ブロックポインタ構造と幅予算.ja.md)

D19 (Block Pointer Structure & Width Budget) governs block pointers: how many sections a pointer is divided into, what fields are contained in each section, their widths, and their ordering; who holds the authority between position entries and the central mapping for dereferencing and release judgments; how the central mapping entry key and value are encoded; which segments of the mapping key travel with the pointer, and how birth sequence numbers are allocated. Out of scope: unit header field tables and unit type codes in D18 (information carried in a block); node size and intra-tree entry layout in D8 (core index structure); placement coordinate system and allocation record keys in D3 (space allocation) settled items 3 / 7, commit endogenous block bump sequence in D3 (space allocation) settled item 10; checksum coverage and the relationship between logical extents and units in D4 (Checksum Location); encryption algorithms, nonce derivation, and origin of expected AAD values in D9 (encryption); root record and system configuration field tables in D22 (Synthesizing Unit Atomicity) settled items 7 / 9; stripe width granularity in D2 (RAID Stripe Policy) settled item 1.

<!-- doc-lint:registry name-col=2 -->

| Identifier | Short Name |
|---|---|
| N1 | Reread once upon read failure |
| N3 | Mismatched entries remain in allocated |

```

**出处 `.claude/kb/decisions/19-block-pointer-structure-width-budget.en.md:14-310`（整段抄，未转述）**

```markdown
### Settled Items

| # | Item | Settlement |
|---|---|---|
| 1 | **Whether position entries carry device identity** | Yes: each physical position carries a dev id, without introducing a layer of chunk mapping table **Status: Settled.** **Progress: Implemented.** |
| 2 | **Bit-width of ciphertext checksum** | 32 bits; takes complete CRC32C, truncation from a wider CRC is forbidden **Status: Settled.** **Progress: Implemented.** |
| 3 | **Fixed-width or variable-length** | Fixed-width, reserving slots when unencrypted: MAC and nonce constantly occupy slots, kept empty and not repurposed **Status: Settled.** **Progress: Implemented.** |
| 4 | **Width of each section in position entry** | Device ID 4 bytes, physical offset 6 bytes (16 KiB slot number), ciphertext checksum 4 bytes => position entry 14 bytes **Status: Settled.** **Progress: Implemented.** |
| 5 | **Where location authority resides** | Position entry demoted to hint, central mapping is unique entrance for dereference and release verdicts **Status: Settled.** **Progress: Unimplemented.** |
| 6 | **Field table of central mapping entries** | Key reuses write sequence key by class (post-tightening): 27 bytes for Code 1, 25 bytes for Code 2 / Code 3; value follows settled items 4 / 5 **Status: Settled.** **Progress: Implemented.** |
| 7 | **Which segments pointer header carries** | Header 50: MAC 16, nonce 12, algorithm type 1, compression algorithm code 1, compressed length 2, extent offset 2, birth tree 8, birth txg 8 **Status: Settled.** **Progress: Implemented.** |
| 8 | **How much birth identity reference side carries, what mapping contains** | All pointers enter mapping, pointers to Code 1 are 88, pointers to Code 2 / Code 3 are 86; mapping tree nodes, tree table units, instance table units bootstrap exempt **Status: Settled.** **Progress: Implemented.** |
| 9 | **How birth sequence numbers are allocated and in what order** | Starts from 0 within tree, Code 2 and Code 3 share one counter, clears upon checkpoint switch, changes number upon rewrite; order matches bump order of commit endogenous blocks **Status: Settled.** **Progress: Implemented.** |
| 10 | **Whether mapping key is fixed-width or variable by class** | Strictly 27 bytes across the board, Code 2 / Code 3 padded with zeroes at tail; central mapping entries strictly 55 bytes across the board **Status: Settled.** **Progress: Implemented.** |
| 11 | **Pointer offset table, position entry order, where mapping tree root resides** | Segments arranged per offset table, position entries in ascending order of device identity; mapping tree root lives in root record, root record totals 457 **Status: Settled.** **Progress: Implemented.** |
| 12 | **Whether Code 2 / Code 3 mapping entries are written day-1, how many times fixed point writes** | Written day-1; an instance writes out each Code 2 node and Code 3 container only once within a checkpoint **Status: Settled.** **Progress: Implemented.** |
| 13 | **How many sections a block pointer is split into** | Split into two sections: logical properties in header, one per logical extent; physical properties in position entries, one per physical placement **Status: Settled.** **Progress: Implemented.** |
| 14 | **Whether position entries can be deduced geometrically from single origin** | Cannot, enumerated individually **Status: Settled.** **Progress: Implemented.** |
| 15 | **Value range of extent offset from origin and version 1 value** | Range [0, 32768), 15 bits sufficient; version 1 constantly 0, bits reserved as specified **Status: Settled.** **Progress: Implemented.** |

#### Settled Item 1: Position Entries Carry Device Identity

**Settlement**: Position entries carry device identity (dev id), one copy per physical position; no indirection layer of "logical address + chunk mapping table" is introduced.

**Scope**: Governs "whether to carry", not width (settled item 4) or ordering (settled item 11). The "per-occurrence variable" property of D2 (RAID Stripe Policy) settled item 1 is the premise for the forward-proving side of this cell, and carrying dev holds under both granularities (it is the more capable side) => granularity settlement only affects whether position entries can be compressed in expression, not whether they are carried. The viability of wide pointers is coupled with node size in D8 (core index structure): for the same 40-byte pointer header, a 4 KiB node pays +98% whereas a 16 KiB node pays only +14%; D8 (core index structure) settled item 2 selects 16 KiB and pins it as a constant. E20 (CPU cache cost of pointer widening) measured a 40-byte header, while the settlement is 50 (settled items 7 / 11); the extra bytes were measured in E128 (point lookup cost after pointer carries birth (tree, txg)) showing accounting tree point lookups slow down by 1.8%–4.5%, leaving the premise intact. The largest cost of carrying dev (removing a device requires a full run scan and rewrite of every tree with pointers) is reduced to O(references of that device) by the reverse index settled in D1 (Data Mobility / Reverse Index), a path this project was already committed to taking.

**Basis**:

- E20 (CPU cache cost of pointer widening): the cost of pointer widening appears only when the working set exceeds L3 cache, which 16 KiB nodes absorb—the direct premise for wide pointers being viable in this project, and the basis side of D8 (core index structure) settled item 2.
- Production implementations (inspected live 2026-08-28, not verified in this project): bcachefs with dev requires a full run scan and rewrite of every tree carrying pointers to remove a device (`bch2_dev_usrdata_drop()`), relying on `BTREE_ID_backpointers` to reduce it to O(references of that device); ZFS welds dev into blkptr, so device removal can only add an immortal layer of `vdev_indirect_mapping` (can only condense, cannot delete); btrfs without dev must carve out a fixed, non-expandable `sys_chunk_array[BTRFS_SYSTEM_CHUNK_ARRAY_SIZE]` (2048 bytes, definition of `BTRFS_SYSTEM_CHUNK_ARRAY_SIZE` in `include/uapi/linux/btrfs_tree.h` and this field in system configuration structure) in system configuration for bootstrap, and balance still rewrites all references.
- Three-party debate (2026-08-28): forward-proving leg and counter-proving leg reached opposite conclusions, and the fact distinguishing them was the variable granularity of D2 (RAID Stripe Policy); local leg invalidated for the round, leg circumstances and process in changelog.

**Owed**: None.

#### Settled Item 2: Ciphertext Checksum Bit-Width Takes 32 Bits

**Settlement**: Ciphertext checksum in position entries takes 32 bits, exactly the width of full CRC32C, involving no truncation; implementations are forbidden from taking the path of "truncating a wider CRC down to 32 bits".

**Scope**: Conclusion is conditional, and conditions must be cited together when referenced—it holds only under this set of inputs: 10 TB pool, 4 KiB blocks, weekly scrub, 10 years, corruption rate 1e-9 per block read, expected undetected upper bound 1e-6. Reopening condition is decidable: if field corruption rates are obtained exceeding 3.37e-9, this cell reopens on the spot. None of the candidate algorithms were eliminated (deviations were all within one order of magnitude). Media corruption is not an attacker in the threat model; this cell does not judge attack resistance.

**Basis**:

- E40 (ciphertext checksum bit-width): minimum bit-width under this input set is 31 (matching rounding of original derivation "n > 30.3"), margin for 32 bits is 3.37x—each of the five inputs (pool capacity, corruption rate, service life, scrub interval, block size) must be off by this factor for 33 bits to be needed; a truncated CRC is not a shorter CRC—full-width CRC32C having 0 undetected single-bit flips is a structural property, whereas truncation turns it into probabilistic detection.
- Collateral evidence (inspected live, not verified in this project): default checksum of btrfs is 32-bit CRC32C (`BTRFS_CSUM_TYPE_CRC32 = 0`), its 256-bit slot is reserved for optional SHA256 / BLAKE2.
- User settlement, verbatim words and that day's derivation in changelog.

**Owed**: None.

#### Settled Item 3: Fixed-Width, Slots Reserved When Unencrypted

**Settlement**: Pointers take fixed width per worst-case scenario—MAC (128 bits) and nonce (96 bits) in the header constantly occupy slots; unencrypted volumes leave them as empty slots, avoiding omission via type flags; these bits remain occupied across all volumes and are not repurposed for other fields.

**Scope**: What is optional is the AAD computation step (not computed when encryption is off), not the format (`.claude/rules/fs-design.md` "encryption is an optional computation, not an optional format"). The mechanism of opening layout branches per medium cannot govern this cell: incompat bits govern whether readers recognize this layout, while this cell governs how a pointer is encoded within the same layout; adopting fixed width first and adding a variable-length layout later is not expansion, but an extra parser plus checker plus crash-point replay that must be permanently maintained. Leaving MAC 16 bytes + nonce 12 bytes = 28 bytes / logical extent unoccupied when unencrypted represents `28 / 32768 = 0.0854%` under D4 (Checksum Location) settled item 7 measurement basis (logical extent occupies exactly 32768 bytes on disk). Variable-length saves those 28 bytes rather than saving a field (algorithm type field D9 (encryption) settled item 12 item 4 must exist regardless), while paying the cost that parsing must proceed from the beginning: fixed-width pointers can directly locate the k-th item by offset, whereas variable-length cannot; "whether scanning a partially corrupted node during recovery can skip a bad item and continue reading" is a deduction, not empirically measured. When encryption is off, MAC 16 and nonce 12 are constantly 0, readers encountering non-zero judge the structure containing the pointer corrupted, following I-2.4 (header checksum coverage) reading for unit header 29 bytes (audit A3a landed in two reader sites—multi-layer Code 2 tree internal entries, journal new root segment pointer: `PointerHeadFieldOutsideTheFirstVersion::MacOrNonceNotZero` in `crates/singlefs-core/src/pointer.rs`); extent offset 2 bytes is constantly 0 in version 1 (version 1 pointers point to whole units), readers encountering non-zero judge the structure containing the pointer corrupted, regardless of encryption status; checker checks on every traversed pointer, core checks in every reader of on-disk pointer headers, landing in [checks-owed.md](../checks-owed.md) repaid table C581 (extent offset non-zero has no criterion) row.

**Basis**:

- No experiments: switching from fixed to variable length saves space at the expense of "whether an isolated pointer can be resolved directly by offset", which are different dimensions with no quantifiable metric. It is underpinned by two rules in `.claude/rules/fs-design.md`—"never sacrifice self-containment to save space", and the criteria for format branches (testing whether it turns "misuse" into "unmountable", without permitting "this is faster / leaner" in criteria).
- User settlement, verbatim words and reaffirmation on 2026-09-12 in changelog.
- Verdict 2026-10-01 adopted: extent offset non-zero in version 1 judges the structure containing the pointer corrupted (`research/prompts/m3-inv-a-r1-main-verification.md` section 2 line 11).

**Owed**: C210 (which category three "second sets" belong to has not been judged).

#### Settled Item 4: Location Entry Width Takes 14 Bytes

<!-- format-const: LOC_ENTRY = 14 stale=11 * 2|tree_entry=61 -->

**Settlement**: Device ID 4 bytes, physical offset 6 bytes (16 KiB slot number), ciphertext checksum 4 bytes => position entry 14 bytes.

| Segment | Width | Why this width |
|---|---|---|
| Device ID | **4** | Upper bound points to no settled clause (D12 (Target Media) does not cap media tiers, repository has no dev id recycling clause => narrow width caps total device identities across pool lifetime), takes wide tier per `.claude/rules/fs-design.md` "never sacrifice self-containment to save space"; precedent is settled item 3 |
| Physical offset | **6**, encoded as **16 KiB slot number** | Index nodes aligned to 16384, size is exactly 16 KiB (D18 (information carried in a block) settled item 9) => encoding as 32 KiB unit number cannot express placements of half the index nodes, while byte offset makes non-16 KiB-aligned origins representable. 6-byte slot number addresses 4 EiB / device, whereas same-width byte offset addresses only 256 TiB |
| Ciphertext checksum | **4** | Settled item 2 |

**Scope**: Physical offset shares identical coordinate system with D3 (space allocation) settled item 7 (D3 (space allocation) settled item 3 basis 2 verbatim requires identical coordinate system) => settled together, independent divergent definitions forbidden. Tree table unit pointer and root record total downstream counts adjust accordingly, and root record total is not registered here; the sole authoritative location is the field table of D22 (Synthesizing Unit Atomicity) settled item 7. Relative to legacy 11 bytes, each position entry adds 3 bytes, each pointer adds 6 bytes.

**Basis**:

- E97 (entry encoding of accounting and allocation records): criterion 7 judges placement encoding by representability—when encoded as 16 KiB slot number, all legitimate origins are representable, whereas half cannot be written under coarser unit numbers.
- User settlement, verbatim words and review of "this figure was consumed without passing through decision process" in changelog.

**Owed**: C238 (derived number "53" after position entry changed from 11 to 14 un-back-scanned).

#### Settled Item 5: Location Authority Is Hybrid, Central Mapping Is Unique Entrance

**Settlement**: Takes hybrid—

| Component | Settlement |
|---|---|
| Position entry (14 bytes in settled item 4) | **Demoted to hint**. Width and segment division unchanged; what changes is its status: reading it allows directly trying that placement, but it may be stale |
| Central mapping | **Unique entrance for dereference and release verdicts**. Relocation modifies only one entry here; "who references me" need not be known |
| State classification | Neither is item 4 on the authoritative state list: mapping can be reconstructed via unit self-describing scan, load-bearing surface remains units per D20 (Load-Bearing Surface: Unit Atomicity & Self-Containment) |

Three hard rules taking effect with settlement:

1. **Releases strictly proceed via mapping, never via hints**; after verifying through mapping to that entry, a read-disk verification against the unit checksum in the position entry is required before release. **When mismatch is verified, isolate that slot, publish succeeds normally**: logically released as usual (mapping entry removed), physical slot not returned to free pool, logged into isolation and reported via counter.
2. **Unit header or hint must carry block birth**—already satisfied: data unit class identity segment in D18 (information carried in a block) settled item 7 already contains birth txg 8 bytes.
3. **Runtime observation point required for hint staleness multi-hop rate** (taking wrong branch is a performance cliff, and cliffs do not error, `.claude/rules/fs-design.md` hard requirement 4).

**How mount state reads mapping**: Version 1 mount state reads the central mapping entirely into memory when opening the pool (`central_mapping_entries` in `crates/singlefs-core/src/mounted_read.rs`); subsequent dereferences query this in-memory copy without issuing mapping reads; when central mapping grows to multiple levels, opening the pool reads the entire mapping tree into mount state, dereferences still only query memory without issuing device reads for mapping; write side checks mapping tree capacity acceptance on each publish based on current entry count plus maximum new entries from this publish, without relying on coincidental capacity numbers from other trees. Multi-level mapping tree has no entry count ceiling; structural ceiling is only the 1-byte level in header; calculates shape in planning step before any disk writes, rejecting if level cannot hold (`HeightBeyondTheLevelField`), without establishing separate entry ceiling (implemented in sprint 19, main agent settled). The reading "one dereference with stale hint issues 3 device reads" rides on this rule, measured when root doubled as leaf, remeasured by test cases when multi-level landed (milestone "overwrite, release, rollback and reuse" closeout table line 28). Allocation record tree is likewise read entirely into mount state upon mount; extent tree is not read entirely, traversing by position when opening files (D8 (core index structure) settled item 14, user settled 2026-09-24).

**What to do when read-disk check in Hard Rule 1 fails to read or mismatches**: Before release, verify checksum by reading disk per position entry; if read fails or mismatches, reread once; only if both reads fail or both mismatch is it treated as mismatch (reread mismatch half settled by main agent 2026-09-25: transient bad read must not permanently isolate a unit; implemented in sprint 25). Different classes across the two reads (first fails to read, reread mismatches, or vice versa) treated as both mismatched, reporting the reread outcome (sprint 25 adopted, zero attack rounds). If any replica mismatches, this unit's allocation records across every disk remain in "allocated", not flipped to released (even if the replica on the other disk matched, maintaining symmetry across disks)—persists across mounts, counted as allocated in acceptance without extra formulas. Position entries pointing to a disk outside the pool, or two entries pointing to the same disk, treated as corrupted mapping entry: rejected before any write, disk unmodified. Rebuilding previous version from disk (`rebuild_version` in `crates/singlefs-core/src/recovery.rs`) copies position entries of unreadable data units without reading contents; mount proceeds normally, erroring only when reading that file, except when unwitnessed (when the selected version is not witnessed by system configuration and its data units are unreadable, writable mount falls back to the witnessed version per D23 (Role and Format of Journal) settled item 14 "when the selected root fails its walk and system configuration has not witnessed it, fall back to the witnessed version"; the fallback is judged by the union of C755 (recovery and two mount triggers fail differently on rollback), zero-round wording, implemented in the implementation batch).

**Scope**: Simultaneously resolves the list half of "where multiple referrers live under sharing" in D21 (Boundary Between Authoritative and Derived States) and the three-way position authority choice in D26 (background compaction and placement reclamation) settled item 5, all other sites linking to settled item 5; originally also closed C34 (three-way contradiction in plaintext mapping layer state), which became void on 2026-09-07 when D9 (encryption) settled item 5 switched to "scrub only", leaving the other two sites unaffected. Mapping entry field table lives in settled item 6. Three known corners lack numbers: mapping tree key layout must satisfy rider in D8 (core index structure) settled item 7 (deduplication window bounded by `T_dirty`); hint refresh policy only modeled as "refresh upon rewrite"; cache hit dimension has zero measurements, E109 (runtime cost of three position authority arms) explicitly disclaims answering. Three bootstrap exemptions in settled item 8. Read-disk verification handling details in settlement paragraph (three questions of C394 (release judgment does not verify unit checksum in mapping entry position item)); "reread once" rereads only once, without backoff or multiple retries.

**Basis**:

- E109 (runtime cost of three position authority arms): three arms measured across dimensions—unit migration cost is linear in referencing trees under pointer authority, independent under central mapping and hybrid; publish pays a fixed overhead independent of leaf count; read pays only 10%–40% extra. Absolute figures must be cited with sampling points: on primary sampling point referencing tree and mapping tree happen to share height, a degenerate point.
- E96 (consistency of hybrid architecture): resolving placement via hint causes misplacement, forcing Hard Rule 1; its `hdr_key` arm references D18 (information carried in a block) settled item 3 rather than settled item 7 field table, so "requires extra fields" format cost is non-existent.
- User settlement 2026-09-23: read disk to verify unit checksum in position entry before release; isolate slot upon mismatch, publish succeeds normally (`research/prompts/m2-placement-falsepositive-r1-main-verification.md` section 3 judging C394 (release judgment does not verify unit checksum in mapping entry position item): none of the three hard rules answered whether to verify), verbatim words in changelog.
- User settlement 2026-09-24: reading mapping entirely into mount state codified into clause per that day's implementation; multi-level mapping changed to whole-tree read by another user settlement the same day (splitting four trees in milestone 2, including mount reading multi-level mapping), landed in sprint 19 (C483 (reading mapping in mount state and tree node rollback lack clauses) ①, settled during parallel line 2 without clause), verbatim words in changelog.
- User settlement 2026-09-24: C394 (release judgment does not verify unit checksum in mapping entry position item) three questions—read disk failure rereads once before treating as mismatch, tolerate unreadable data units during rebuild, allocation records for isolated replica remain allocated (verdict `research/prompts/m2-newq-r1-main-verification.md` section 3 N1 (Reread once upon read failure)–N3 (Mismatched entries remain in allocated), verbatim words in changelog); no experiments: candidate readings came from counter-proving leg prototype on replica (`research/prompts/m2-newq-r1-opus-model/`), not registered as experiment.
- User settlement 2026-09-24: multi-level mapping read entirely into mount state upon mount, write side adds mapping tree capacity admission (verdict `research/prompts/m2-treesplit-r1-main-verification.md` T3).
- User settlement 2026-09-25: when only one replica mismatches both disks retain allocated, position entries pointing outside pool codified as reject per that day's code (sprint 16 follow-up report `m2-writepath-implementer-report.md` section 6 Q2, Q3), verbatim words in changelog; no experiments: neither alters disk format, only settles policy.
- Main agent settlement 2026-09-25: `records/2026-09-24-里程碑二收尾调度.md` section 3 row on "transient bad read causes permanent unit isolation".
- Except when unwitnessed: root ring three-party round three verdict `research/prompts/m3-close-rootring-r3-main-verification.md` section 4, the row in the table "where the ones that entered no cell went" that revisits D19 (Block Pointer Structure & Width Budget) settled item 5 (defense Z5.2: data units on an unwitnessed R are unreadable, and writable mount changes from mounting R as usual to falling back to the previous version); user settlement 2026-10-20 (C755 (recovery and two mount triggers fail differently on rollback) "add all three sentences"), verbatim wording in history. No experiment: numbers measured on the three-party legs' replicas, no experiment number assigned.

**Owed**: C248 (bootstrap exemption of tree table units unwritten), C253 (contradictory kb claims on 66 048 fixed overhead), C133 (migration forwarding information lacks carrier), C279 (scan rebuild sees both old unit and slot after packing); C417 (hint refresh policy only modeled in one variant), C418 (cache hit dimension of three position authority arms zero measurements)—cells that respective basis experiments acknowledged inability to answer; C543 (rebuilding previous version carries only sheet 0 of instance table).

#### Settled Item 6: Central Mapping Key Reuses Write Sequence Key by Class (Post-Tightening)

**Settlement**: Key is fixed-width, arranged segment by segment, definition and width matching `class_reuse_tagged` arm of E134 (mapping key format cost: slot baseline and two new arms) verbatim—

| Class | Key (fixed-width, segment by segment) | Width |
|---|---|---|
| Code 1 data unit | Class tag 1 + birth tree 8 + birth txg 8 + write sequence 10 (instance number 4 + transaction number 6) | 27 |
| Code 2 / Code 3 | Class tag 1 + birth tree 8 + birth txg 8 + instance number 4 + birth sequence number 4 | 25 |

Value follows settled items 4 / 5: position entry × w replicas (28 bytes when w = 2). Padding and total entry width belong to settled item 10.

**Four tightenings carried with it** (added during three rounds of debate per "record one loss / tighten only / declare where tightened", must still be respected after settlement):

- Class tag takes code in D18 (information carried in a block) settled item 11 registry table for the named object; objects in D27 (small-data packing container) slots are constantly Code 1; upon packing and whole-container rewriting, segments of slot key take verbatim values from pre-packing unit header.
- Birth tree takes birth tree in class identity segment (Code 3 offset 43), birth txg takes birth txg (Code 3 offset 73; Code 1 and Code 2 likewise take birth txg), instance number takes first 4 bytes of write sequence.
- D16 (publish semantics) settled item 5 closing sentence "one transaction writes at most one unit of user data" is read as covering all Code 1 units (preallocations, truncated tail units, punch hole ends, reflink copies on both sides count), preferably incorporated into that statement.
- Birth sequence number serves only as one segment of key, not for multi-version selection at fixed points—using it for selection falls into "sequence number for distinguishing stale replicas" rejected in D18 (information carried in a block) settled item 1; D18 (information carried in a block) settled item 1 owes a scope note following write sequence precedent.

**Scope**: Governs which segments comprise key and value and their widths, not padding or total entry width (settled item 10), classes contained (settled item 8), or birth sequence allocation (settled item 9). Lands in D15 (format freeze policy) Layer 2. Uniqueness of Code 1 half relies on deduction that "Code 1 write sequence is already unique version identifier", built on transaction subdivision discipline in D16 (publish semantics) settled item 5 tail (paired with D23 (Role and Format of Journal) settled item 7 one transaction number per record); that section closed review on 2026-09-13 with C123 (user settlement modified without review), subdivision discipline unchanged, deduction holds; remaining contradictions logged in C310 (transaction subdivision discipline contradicts record count basis). Recorded provisionally: chosen based on existing cost and performance numbers, reopenable upon new counter-evidence, not a proven unique solution.

**Basis**:

- E137 (performance gap of mapping key forms): gap across three arms under fsync, background load, and random reads—key reusing write sequence by class leads in most fsync cells and all random read cells, losing in global random overwrites, file deletion, and single-file snapshot deletion; user selected it after reviewing table. ⚠️ In model, physical placement equals write order, the most favorable scenario for migration and snapshot deletion; shrinkage after aging unmeasured.
- E133 (format cost of mapping key candidates): bytes paid in format by each key variant across two rounds of debate, giving pointer width, entry width, and fanout across seven candidates.
- E134 (mapping key format cost: slot baseline and two new arms): arm chosen here is its `class_reuse_tagged`, reporting 1-byte class tag and two baselines of D27 (small-data packing container) slots line by line.
- E136 (cost rows of mapping key forks): four missing cost numbers before user settlement—adding 4 bytes to Code 2 header drops zero fanout under three real header widths of D18 (information carried in a block) settled item 14, mapping tree height differs by one layer only in a narrow pool scale range.
- Verdicts of three debate rounds and process in changelog; user settlement, verbatim words in changelog.

**Owed**: C288 (consequential edits of mapping key settlement not propagated to other decisions), C289 (unspecified which field Code 2 birth tree takes), C163 (identical write sequence across fixed point rounds defeats tie-breaking), C286 (whether remount after read-only takes new instance number), C287 (switching adopts open checkpoint transactions then crashes), C310 (transaction subdivision discipline contradicts record count basis).

#### Settled Item 7: Pointer Header Carries Birth (Tree, TXG), 8 Bytes Each

**Settlement**: In addition to MAC 16 + nonce 12 + algorithm type 1 + compression algorithm code 1 + compressed length 2 + extent offset 2, pointer header carries birth tree 8 and birth txg 8, header totaling **50**; pointer remains fixed-width (settled item 3). Compression 3 bytes includes compression algorithm code 1 byte (registry table in D9 (encryption), 0 = no compression, version 1 writes 0) plus compressed length 2 bytes (version 1 writes 0, sufficient for [0, 32768) intra-unit length).

**Scope**: Settles which 16 bytes reside in pointer header, not total pointer width (settled by combining settled item 8 and settled item 11 to 88 / 86). Compression 3 bytes provides the format carrier for hard requirement in D9 (encryption) settled item 13 item 2—requiring compression/encryption order to be explicitly defined in format, where previously no field carried "this ciphertext is compressed, original length X". ⚠️ Main agent recommended omitting (compression is optional computation, and "reserve bits across all volumes" principle governs only encryption); user chose retention via popup settlement, without three-party debate, overturnable. Four consumers read these two segments from pointer: D5 (Snapshot / Space Accounting Mechanism) O(1) judgment reads birth txg; deadlist entries recorded by version identity, birth tree taken from pointer; central mapping key; AAD format requires pointer to reserve slots for these fields, values independently verified (D9 (encryption) settled item 16 explicitly forbids reading back from pointer). Alters bytes of new-pool file creation: yes—adds 3 bytes to every pointer.

**Basis**:

- E128 (point lookup cost after pointer carries birth (tree, txg)): accounting tree internal entries widened slower across four rounds, inode tree pair unstable across four rounds and unusable; measures only these 16 bytes, a lower bound on this cell's cost.
- One round of three-party debate, verdict in `research/prompts/d19-item7-r1-main-verification.md`: counter-proving leg noted closing sentence of settled item 8 left ambiguity on "whether living in pointer or referencing entry", both interpretations defensible.
- User settlement ("16 bytes live in pointer"), verbatim words and disqualification of candidates B and C in changelog.

**Owed**: C295 (whether key segments in pointer count as AAD lookup path), C296 (mapping key incomplete during journal replay), C297 (unspecified handling of two birth trees in inode internal entry), C237 (no slots for birth (tree, txg) in Code 1 / Code 2 pointers).

#### Settled Item 8: All Pointers Enter Mapping, Reference Side Carries Mapping Key Segments by Class

**Settlement**: Central mapping holds Code 1, Code 2, and Code 3 units; pointers to a unit carry mapping key segments for that class excluding class tag, class tag supplied by lookup path.

| Target Class | Pointer Carries (key segments from settled item 6) | Pointer Width |
|---|---|---|
| Code 1 | Birth tree 8 + birth txg 8 + write sequence 10 | Data pointer **88** |
| Code 2 / Code 3 | Birth tree 8 + birth txg 8 + instance number 4 + birth sequence number 4 | Node pointer **86** |

**Three classes excluded from mapping (bootstrap exemption)**: Mapping tree's own nodes, tree table units, instance table units. Position entries in their parent pointers (mapping tree root, tree table pointer and instance table pointer in root record) are authoritative, not hints; settled item 5 "unique entrance" and Hard Rule 1 do not apply to these three; all three are pool-level singletons with exactly one referrer. Parent pointers of these three share 86 width with node pointers to Code 2 / Code 3: position entries 14 × 2 are authoritative, instance number 4 + birth sequence number 4 populated from target unit header without querying mapping; parser determines pointer width by target unit class, not by whether target is an exempt class. For non-exempt units (Code 1 data units and Code 2 tree nodes—extent tree root, inode tree root, inode leaf container), unreadable position hints trigger fallback through mapping; exempt three classes (mapping tree root, tree table, root record chain) do not fall back.

**Scope**: Decides which classes mapping holds and which key segments pointers carry, invariant to pointer width (where birth (tree, txg) 16 bytes living in pointer header was settled by settled item 7). Holding Code 1 only loses on two correctness points: Code 2 / Code 3 blocks likewise enter deadlist / livelist, and D5 (Snapshot / Space Accounting Mechanism) settles deadlist entries recorded by version identity and released via mapping; without mapping they must be recorded by placement, causing release at stale placements after compaction or drive-loss reconstruction; migrating a Code 2 / Code 3 referenced by read-only snapshot requires rewriting snapshot tree ancestors, taking new birth violates reference interval while keeping old birth violates `birth` definition. All pointers entering mapping avoids both losses: deadlist / livelist recorded by mapping key, migration alters only one mapping entry, migrated unit remains byte-for-byte unchanged with birth unchanged. Costs: mapping blocks per unit during fsync and background exceed Code 1-only by 10%–20%+, random read misses also rise; column undercounts two items (insert/delete of derived tree Code 2 in mapping, extent nodes with stale hints reading via mapping). Parent node rewrites populate position hints with mapping placements (C417 (hint refresh policy only modeled in one variant) version 1 takes "refresh upon rewrite"). Checker and GPU judge treat position entries of non-exempt Code 1 / 2 / 3 as hints: traversal attempts hint first, reads mapping placement if both mismatch, records mapping placement in reference set.

**Basis**:

- E137 (performance gap of mapping key forms): measured cost gaps between two configurations (Code 1 only / all pointers enter mapping) cell by cell, cost row of this cell taken directly from it.
- E134 (mapping key format cost: slot baseline and two new arms): pointer width, entry width, and fanout across configurations provided by it; 88 / 86 derived from its two rows plus 3 compression bytes from settled item 7.
- E128 (point lookup cost after pointer carries birth (tree, txg)): point lookup cost omitted on settlement day and supplemented next day, recorded in cost row of this cell.
- Two rounds of three-party debate, verdicts in `research/prompts/d19-item8-r1-main-verification.md` and `research/prompts/d19-item8-r2-main-verification.md` (first round cloud legs reached opposite conclusions, user requested rerun, stances reversed in second round).
- User settlement, verbatim words and reasons why both remedy paths required modifying settled clauses in changelog.
- Verdict in `m2-lines234-code-r1-main-verification.md` 2026-09-23 on whether tree nodes fall back via mapping: counter-proving leg tested same corruption where data unit read back correct bytes via extra hop, whereas extent tree root made `open_pool_for_read` unmountable for entire pool.
- Verdict `research/prompts/m4-gc-r1-main-verification.md` section 3 G3: write path, reconstruction, and checker refactored per "mapping is unique entrance" (Option B), parent rewrites write mapping placement, checker and GPU judge treat position entries as hints only. Section 7 user settlement "if inappropriate, illogical, or unnatural, refactor", verbatim words in changelog.

**Owed**: C293 (two items undercounted by E137 after all pointers enter mapping), C292 (rationale for zero consumers of Code 2 identity segment no longer holds), C248 (bootstrap exemption of tree table units unwritten) (merged from C280 (which classes central mapping holds lacks clauses)).

#### Settled Item 9: How Birth Sequence Numbers Are Allocated and in What Order

**Settlement**: Birth sequence numbers start from 0; increments by 1 for each Code 2 or Code 3 unit written within same tree (both classes share one counter); clears when scope shifts to next checkpoint; same node rewritten second time by fixed point within same checkpoint takes new number. Allocation sequence matches bump sequence of commit endogenous blocks from open cluster segment—ascending by tree ID, within tree leaves before root, key-ascending within same level, central mapping tree penultimate, tree table unit last.

**Scope**: Sole registration site for sequence itself is D3 (space allocation) settled item 10, no second copy stored here. Birth sequence number is a segment of central mapping key (settled item 6); key must be determined before unit is written, binding allocation to placement allocation sequence: single cursor in implementation, single sequence checked by checker. Serves only as segment of key, not for multi-version selection at fixed points (fourth tightening of settled item 6). User popup settlement, without three-party debate, overturnable; main agent recommended this tier. Alters bytes of new-pool file creation: yes—placements and birth sequence numbers of eleven commit endogenous blocks (default geometry; 13 blocks when ring is 1024 MiB) in new-pool file creation determined by it, per-block values in D3 (space allocation) settled item 10 ⑤ table.

**Basis**:

- No experiments: sequence and initial values have no quantifiable metric, cost of sharing rule between numbering and placement lives on D3 (space allocation) settled item 10 side; cell supported by deduction "key must be determined before unit is written" plus tightening arguments (same container receiving birth sequence 0 in two checkpoints buys collision-free runtime mapping) holding only under rewrite number change.
- User settlement, verbatim words in changelog.

**Owed**: C291 (allocation rules for birth sequence numbers exist only in experiment source).

#### Settled Item 10: Mapping Key Strictly 27 Bytes, Entries Strictly 55 Bytes

**Settlement**: Central mapping key is **strictly 27 bytes across the board**—Code 1 (class tag 1, birth tree 8, birth txg 8, write sequence 10) is naturally 27; Code 2 / Code 3 (class tag 1, birth tree 8, birth txg 8, instance number 4, birth sequence number 4) is 25 bytes, **zero-padded to 27 at tail**. Central mapping entries are thus **strictly 55 bytes across the board** (key 27 + value 28, value = position entry 14 × 2). Key range in mapping tree node headers is likewise stored at 27.

**Scope**: Two related sites return to fixed-width with it—sparse side table entries in D5 (Snapshot / Space Accounting Mechanism) settled item 6 fixed to 31 (mapping key 27 + count 4), livelist entries in D6 (snapshot implementation model) settled item 2 fixed to 36 (head tree ID 8 + mapping key 27 + event type 1). Varying length by class would violate two universal rules in D8 (core index structure) settled item 11: "fixed-width entries arranged contiguously" and "declared length = entry count × entry width"—entry width in Code 2 header has only one value, so two widths require a sentinel, eliminating intra-node fixed-width binary search and forcing sequential scan from start to resolve a mapping node during scan. Cost is 2 extra bytes per Code 2 / Code 3 entry, 10 extra bytes in new-pool file creation. User popup settlement, without three-party debate, overturnable; main agent recommended this tier. Alters bytes of new-pool file creation: yes.

**Basis**:

- E145 (cost of Code 2 self-describing header and mapping tree key width): measured single-width arm; "two widths 55 / 53, self-delimited by first key byte" was an unmeasured fourth form in that round, taking it as settlement would cite a number with no artifact.
- User settlement, verbatim words in changelog.

**Owed**: None (C321 (Code 2 header entry width is single value while mapping tree entries vary by class) repaid 2026-09-14 with user settlement returning mapping key to fixed-width 27, see owed check repaid table).

#### Settled Item 11: Pointer Offset Table, Position Entry Ordering, Where Mapping Tree Root Resides

**Settlement**: Pointer segments arranged from offset 0—

| Segment | Offset | Width |
|---|---|---|
| MAC | 0 | 16 |
| nonce | 16 | 12 |
| Algorithm type | 28 | 1 |
| Compression algorithm code | 29 | 1 |
| Compressed length | 30 | 2 |
| Extent offset from origin | 32 | 2 |
| Birth tree | 34 | 8 |
| Birth txg | 42 | 8 |
| Position entries × 2, ascending by device identity | 50 | 14 × 2 |
| Key tail: Code 1 write sequence 10; Code 2 / Code 3 instance number 4 + birth sequence number 4 | 78 | 10 / 8 |

=> Pointers to Code 1 are **88**, pointers to Code 2 / Code 3 are **86**. Position entries sorted ascending by device identity, established as I-2.5 (position entries ascending by device identity). **Central mapping tree root lives in root record**: root record carries an 86-byte central mapping tree root pointer, not entering tree table; root record totals **457** bytes. Tree table version 1 has **12** entries.

**Scope**: Settles only ordering—set of segments and widths settled in settled items 3 / 4 / 7 / 8. Compression 3 bytes immediately follows algorithm type, belonging to the group "how this ciphertext was produced"; placed before extent offset to keep four encryption and compression fields contiguous. Sole authoritative location for root record total is field table of D22 (Synthesizing Unit Atomicity) settled item 7; tree table version 1 count follows D5 (Snapshot / Space Accounting Mechanism) settled item 11 adding deadlist. User popup settlement, without three-party debate, overturnable. Alters bytes of new-pool file creation: yes—segment offsets and widths of every pointer, and root record total.

**Basis**:

- No experiments: offset table has no second choice, segments and widths already settled, cell only orders them with no quantifiable metric. Ascending device identity establishes deterministic order for two position entries, enabling invariant.
- User settlement, verbatim words in changelog.

**Owed**: None (C304 (instance table chain pointer record cannot fit 83-wide pointer) repaid 2026-10-11, see owed check repaid table).

#### Settled Item 12: Code 2 / Code 3 Mapping Entries Written Day-1, Fixed-Point Writes Once per Checkpoint

**Settlement**: Maintain day-1 write-out (6 entries in new-pool file creation). Fixed point codified: each Code 2 node and Code 3 container written only once per checkpoint by an instance—one-shot COW at end of publish; version 1 forbids concurrent insertion with COW (implementation permitted by D3 (space allocation) settled item 5 not adopted in version 1).

**Scope**: ⚠️ When concurrent insertion with COW is permitted later, D8 (core index structure) settled item 13 must still be respected: same key lands in leaf only once per publish based on final state. Failing checks counted per unit, not per fixed-point round. Cell settles whether to write and write frequency, not key form (settled item 6) or entry width (settled item 10). When settled, repository lacked release and reuse experiments, leaving other path as "revisit once release experiments exist", an uncodified trigger condition.

**Basis**:

- E81 (commit fixed point): origin of reading "dirty nodes COW only once per publish", and basis for fixed point convergence under this reading; applying it to Code 2 and Code 3 eliminates multi-version same-key loss within same checkpoint.
- User settlement, verbatim words in changelog.

**Owed**: C352 (settlement of writing mapping entries day-1 lacks awakeners for reopening conditions), C163 (identical write sequence across fixed point rounds defeats tie-breaking).

#### Settled Item 13: Block Pointer Split into Two Sections

**Settlement**: **Block pointer is split into two sections: one "header" following the logical extent, and N "position entries" following physical placements.**
Logical properties (MAC, nonce, algorithm type, extent offset from origin) in header, one copy per logical extent;
Physical properties (placement, and ciphertext checksum for unkeyed corruption detection) in position entries, one copy per physical position.

**Scope**: Splitting demotes "total pointer width" from a hard constraint to a budget, rewriting D4 (Checksum Location) into a two-section form. Settles only which class of properties lives in which section, not specific fields (settled items 4 / 7) or order (settled item 11); birth tree and birth txg later added to header (settled item 7) remain logical properties. "Moving out of pointer" does not equal an extra I/O: D4 (Checksum Location) requires parent node to store child checksum, constraining which node it lives in, not which struct.

**Basis**:

- D2 (RAID Stripe Policy) settled "multi-replica writes identical ciphertext N times" => all replicas must share same nonce; storing per pointer duplicates same value N times, and writing one replica wrong yields two incompatible decryption results. Testable form is I-6.7 (replicas share nonce).
- Production implementations exhibit same shape (inspected live 2026-08-28, not verified in this project): bcachefs `struct bch_extent_ptr` is exactly 64 bits (`offset:44, dev:8, gen:8` plus four flags), checksum / nonce / compression info moved to independent variable-length entries (`bch_extent_crc32/64/128`) within same bkey, one crc entry applying contiguously to all following ptr entries; nonce itself unpersisted, `extent_nonce()` in `fs/bcachefs/checksum.h` computed live from `bversion` (96 bits, in bkey header) plus small per-crc offset (`crc64.nonce:10` / `crc128.nonce:13`).
- No experiments: two-section split is a format architectural trade-off, underpinned by settled rule (replicas share nonce) and production shapes, with no quantifiable metric.

**Owed**: None.

#### Settled Item 14: Position Entries Listed Individually, Cannot Be Computed from Single Origin via Geometric Formula

**Settlement**: N position entries listed individually, each carrying device identity; not deduced from single origin plus size for remaining columns.

**Scope**: Governs expression of position entries, not constituent segments (settled item 4). Prerequisite is D2 (RAID Stripe Policy) settled item 1 defining "variable width" as **per-occurrence variable**—listing physical placements individually with device identities and avoiding chunk mapping tables is the exact premise required by this deduction. Isomorphic to D2 (RAID Stripe Policy) settled item 1: buys "no pointer to wrong disk", not "failure domain independence".

**Basis**:

- ZFS can deduce all RAID-Z column positions from "one DVA + asize" (`BP_GET_ASIZE` summing asize of 3 DVAs), predicated on top-level vdev child count being **fixed at pool creation** (inspected live, not verified in this project); D2 (RAID Stripe Policy) settled item 1 negated this premise.
- No experiments: deducibility is directly determined by stripe width granularity, a deduction with no quantifiable metric.

**Owed**: None.

#### Settled Item 15: Value Range of Extent Offset from Origin and Version 1 Value

**Settlement**: Value range of extent offset from origin is capped by **maximum extent size**, not file size; range in this project is [0, 32768), **15 bits sufficient**. Version 1 field is constantly 0—splitting is a reserved feature, disabled in version 1; bits reserved per settled item 3 "fixed-width reserved slots", justified by policy rather than usage.

**Scope**: Addresses value range; width of 2 bytes lives in settled item 11 offset table. D9 (encryption) settled item 12 item 2 estimated 32–64 bits based on file size, an overestimate. Basis for narrower range in this project is D4 (Checksum Location) settled item 7: extent on disk strictly = unit = 32768.

**Basis**:

- Production inspection (inspected live 2026-08-28 in `fs/bcachefs/extents_format.h`, not verified in this project): `crc32.offset:7` / `crc64.offset:9` / `crc128.offset:13`, matching `CRC128_SIZE_MAX (1U << 13)` => value range capped by maximum extent size.
- No experiments: value range is arithmetic dividing two settled clauses, with no quantifiable metric; version 1 disablement settled by D4 (Checksum Location) settled item 7.

**Owed**: None.

```

**出处 `.claude/kb/decisions/19-block-pointer-structure-width-budget.en.md:311-321`（整段抄，未转述）**

```markdown
### Chain Reactions with Other Decisions

| Decision | Chain point |
|---|---|
| D4 (Checksum Location) | Settled item 13 demotes "total pointer width" from constraint to budget, rewriting D4 (Checksum Location) into two-section form; logical extent strictly occupying 32768 given by D4 (Checksum Location) settled item 7, relied upon by settled items 3 and 15 |
| D2 (RAID Stripe Policy) | Prerequisite for settled items 1 and 14 is variable stripe width, settled as per-occurrence variable by D2 (RAID Stripe Policy) settled item 1 |
| D9 (encryption) | Nonce carrier settled half by settled item 13 (in header, one per logical extent); registry tables for algorithm type and compression live in D9 (encryption), expected AAD cannot be read back from pointer (D9 (encryption) settled item 16) |
| D12 (Target Media) | Mount comparison table in D12 (Target Media) settled item 4 rejects mount on device identity mismatch, based on position entries carrying dev in settled item 1. ⚠️ Criterion requires "whether **address arithmetic** consuming it carries device identity", btrfs address arithmetic includes device identity in chunk tree; criterion holds, leap "carried => must be in pointer" falls |
| D15 (format freeze policy) | Pointer field layout is Layer 2 of D15 (format freeze policy) settled item 7; central mapping entry field table in settled item 6 also lands in Layer 2 (explicitly cited in basis of D15 (format freeze policy) settled item 1) |
| D8 (core index structure) | Pointer width enters internal entries and tree table entries, several widths in D8 (core index structure) settled items 6 / 8 adjust with settled items 7 / 8; viability of wide pointers couples with 16 KiB nodes in D8 (core index structure) settled item 2 |

```

**出处 `.claude/kb/decisions/19-block-pointer-structure-width-budget.en.md:322-325`（整段抄，未转述）**

```markdown
## Historical Versions

Historical entries for D19 (Block Pointer Structure & Width Budget) are centralized in [decisions-history.md](../decisions-history.md).

```

**出处 `.claude/kb/decisions/19-ブロックポインタ構造と幅予算.ja.md:1-13`（整段抄，未转述）**

```markdown
## D19 (ブロックポインタ構造と幅予算) — 確定

[中文](19-块指针的结构与宽度预算.md) · [English](19-block-pointer-structure-width-budget.en.md) · **日本語**

D19 (ブロックポインタ構造と幅予算) はブロックポインタのセルを管轄する：ポインタがいくつのセグメントに分かれるか、各セグメントにどのようなフィールドが含まれ、それぞれ幅はいくらか、どのような順序で並ぶか；位置エントリと中央マッピングのどちらがデリファレンスおよび解放判定の権威であるか；中央マッピングエントリの key と value をどうエンコードするか；マッピング key のどのセグメントがポインタに追従し、誕生シーケンス番号をどう発行するか。対象外とする事項：ユニットヘッダのフィールド表とユニット種別コードは D18 (ブロックに含める情報)；ノードサイズとツリー内エントリレイアウトは D8 (コアインデックス構造)；配置先の座標系と割り当てレコードの key は D3 (領域割り当て) 確定項目 3 / 7、コミット内生ブロックのバンプ順序は D3 (領域割り当て) 確定項目 10；チェックサムのカバー範囲、論理 extent とユニットの関係は D4 (チェックサム位置)；暗号化アルゴリズム、nonce の生成方法、AAD 期待値の出所は D9 (暗号化)；ルートレコードとシステム構成のフィールド表は D22 (ユニット原子性の合成手法) 確定項目 7 / 9；ストライプ幅の粒度は D2 (RAID ストライプポリシー) 確定項目 1 に属する。

<!-- doc-lint:registry name-col=2 -->

| 番号 | 略称 |
|---|---|
| N1 | 読み取れない場合はまず1回再読み取り |
| N3 | 一致しないものは割り当て済みに維持 |

```

**出处 `.claude/kb/decisions/19-ブロックポインタ構造と幅予算.ja.md:14-310`（整段抄，未转述）**

```markdown
### 確定項目

| # | 項目 | 確定内容 |
|---|---|---|
| 1 | **位置エントリにデバイス識別子を含めるか** | 含める：各物理位置にそれぞれ dev id を保持、chunk マッピングテーブルの階層を挟まない **状態：確定。** **進捗：実装済み。** |
| 2 | **暗号文チェックサムのビット幅** | 32 ビット；完全な CRC32C を採用、より広い CRC からの切り捨ては禁止 **状態：確定。** **進捗：実装済み。** |
| 3 | **固定幅か可変長か** | 固定幅、非暗号化時もスロットを確保：MAC と nonce は恒常的にスロットを占有し、空きスロットとして維持し他用途への転用を禁止 **状態：確定。** **進捗：実装済み。** |
| 4 | **位置エントリ各セグメントの幅** | デバイス ID 4 バイト、物理オフセット 6 バイト（16 KiB スロット番号）、暗号文チェックサム 4 バイト => 位置エントリ 14 バイト **状態：確定。** **進捗：実装済み。** |
| 5 | **位置の権威はどこにあるか** | 位置エントリはヒントに降格、中央マッピングがデリファレンスと解放判定の唯一の入口 **状態：確定。** **進捗：未実装。** |
| 6 | **中央マッピングエントリのフィールド表** | key は種別ごと書き込みシーケンスキー再利用（厳格化後）：Code 1 は 27 バイト、Code 2 / Code 3 は 25 バイト；value は確定項目 4 / 5 に準拠 **状態：確定。** **進捗：実装済み。** |
| 7 | **ポインタヘッダにどのセグメントを含めるか** | ヘッダ 50：MAC 16、nonce 12、アルゴリズム種別 1、圧縮アルゴリズムコード 1、圧縮後長 2、extent オフセット 2、誕生ツリー 8、誕生 txg 8 **状態：確定。** **進捗：実装済み。** |
| 8 | **参照側が担持する誕生識別子の量、マッピングが収容する種別** | 全ポインタがマッピング経由、Code 1 へのポインタは 88、Code 2 / Code 3 へのポインタは 86；マッピングツリーノード、ツリーテーブルユニット、インスタンス表ユニットはブートストラップ免除 **状態：確定。** **進捗：実装済み。** |
| 9 | **誕生シーケンス番号の発行方法と順序** | ツリー内で 0 から開始、Code 2 と Code 3 は単一カウンタを共有、チェックポイント切り替え時にゼロクリア、再書き込み時に番号変更；順序はコミット内生ブロックのバンプ順序と同一 **状態：確定。** **進捗：実装済み。** |
| 10 | **マッピング key は固定幅か種別ごと可変長か** | 一律 27 バイト、Code 2 / Code 3 は末尾をゼロパディング；中央マッピングエントリは一律 55 バイト **状態：確定。** **進捗：実装済み。** |
| 11 | **ポインタのオフセット表、位置エントリの順序、マッピングツリー根の配置場所** | セグメントはオフセット表順に配置、位置エントリはデバイス識別子昇順；マッピングツリー根はルートレコードに配置、ルートレコード合計 457 **状態：確定。** **進捗：実装済み。** |
| 12 | **Code 2 / Code 3 のマッピングエントリは day-1 で書き込むか、固定点は何回書き込むか** | day-1 で書き出し；1つのインスタンスは1つのチェックポイント内で各 Code 2 ノードおよび Code 3 コンテナを1回のみ書き出し **状態：確定。** **進捗：実装済み。** |
| 13 | **ブロックポインタは何セグメントに分割されるか** | 2セグメントに分割：論理属性はヘッダに配置し論理 extent ごとに1つ、物理属性は位置エントリに配置し物理配置先ごとに1つ **状態：確定。** **進捗：実装済み。** |
| 14 | **位置エントリは単一起点から幾何学的計算で導出可能か** | 導出不能、個別に列挙 **状態：確定。** **進捗：実装済み。** |
| 15 | **起点からの extent オフセットの値域と第1版の値** | 値域は [0, 32768)、15 ビットで十分；第1版は恒常的に 0、ビットは規定通り確保 **状態：確定。** **進捗：実装済み。** |

#### 確定項目 1：位置エントリにデバイス識別子を含める

**確定内容**：位置エントリにデバイス識別子（dev id）を含め、各物理位置にそれぞれ1つ保持；「論理アドレス + chunk マッピングテーブル」という間接層は導入しない。

**射程**：管轄するのは「含めるか否か」であり、幅（確定項目 4）と順序（確定項目 11）は対象外。D2 (RAID ストライプポリシー) 確定項目 1 の「都度変更可能」が本セルの順方向推論側の前提であり、dev を含める設計は両方の粒度で成立する（能力がより高い側である）=> 粒度の決定は位置エントリを圧縮表現できるか否かにのみ影響し、含めるか否かには影響しない。幅広ポインタが成立するか否かは D8 (コアインデックス構造) のノードサイズと結合している：同一の 40 バイトポインタヘッダに対し、4 KiB ノードは +98% のコストを払うが、16 KiB ノードは +14% のみであり、D8 (コアインデックス構造) 確定項目 2 は 16 KiB を選択し定数として固定している。E20 (ポインタ肥大化の CPU キャッシュコスト) は 40 バイトのヘッダを測定したが、確定内容は 50（確定項目 7 / 11）である；余分なバイト数について E128 (ポインタへの誕生 (ツリー, txg) 付加後のポイントルックアップコスト) では会計ツリーのポイントルックアップが 1.8%–4.5% 低速化することを測定したが、前提は覆っていない。dev を含める最大のコスト（デバイス削除時にポインタを持つ全ツリーをフル実行走査 (フルスキャン)・再書き込みする必要がある）は D1 (データ移動性 / 逆引きインデックス) で確定済みの逆引きインデックスにより O(当該デバイスの参照数) に削減され、その経路は本プロジェクトがそもそも進むべき道である。

**根拠**：

- E20 (ポインタ肥大化の CPU キャッシュコスト)：ポインタ肥大化のコストはワーキングセットが L3 キャッシュを超過した場合にのみ現れ、16 KiB ノードはこれを吸収する——幅広ポインタが本プロジェクトで成立する直接の前提であり、D8 (コアインデックス構造) 確定項目 2 の根拠側でもある。
- 現役実装（2026-08-28 実地調査、本プロジェクトでの検証は未実施）：dev を持つ bcachefs はデバイス削除時にポインタを持つ全ツリーをフル実行走査 (フルスキャン)・再書き込みする必要があり（`bch2_dev_usrdata_drop()`）、`BTREE_ID_backpointers` により初めて O(当該デバイスの参照数) に低減する。ZFS は dev を blkptr に固定化しており、デバイス削除は消えることのない `vdev_indirect_mapping` 層を追加するしかない（condense のみ可能で削除不能）。dev を持たない btrfs はマッピングテーブルのブートストラップのためにシステム構成内に固定長・拡張不能な `sys_chunk_array[BTRFS_SYSTEM_CHUNK_ARRAY_SIZE]`（2048 バイト、`include/uapi/linux/btrfs_tree.h` 内の `BTRFS_SYSTEM_CHUNK_ARRAY_SIZE` 定義とシステム構成構造体内の同フィールド）を確保せねばならず、balance は同様に全参照を再書き込みする。
- 三者論証（2026-08-28）：順推論側と反駁側で正反対の結論に至ったが、両者を識別する事実は D2 (RAID ストライプポリシー) の可変粒度であった；ローカル側担当はそのラウンド全体が無効、担当状況と経緯は変更履歴に記録。

**未済の検査**：なし。

#### 確定項目 2：暗号文チェックサムは 32 ビットを採用

**確定内容**：位置エントリ内の暗号文チェックサムは 32 ビットを採用し、完全な CRC32C の幅そのものであり、切り捨ては伴わない；実装において「より広い CRC を 32 ビットに切り詰める」手法を採ることは禁止される。

**射程**：結論は条件付きであり、参照時は条件も同時に引用しなければならない——本件は以下の入力セットにおいてのみ成立する：10 TB プール、4 KiB ブロック、週次 scrub、10 年、読み取り 1 ブロックあたりの破損率 1e-9、検出漏れの期待値上限 1e-6。再開条件は判定可能：現場の破損率を取得しそれが 3.37e-9 を超えた場合、本セルは即座に再開される。候補アルゴリズムは1つも脱落していない（偏りはすべて1桁以内）。脅威モデルにおいてメディア破損は攻撃者ではなく、本セルは耐攻撃性を判定しない。

**根拠**：

- E40 (暗号文チェックサムビット幅)：この入力セット下での最小ビット幅は 31（元の演繹文「n > 30.3」の丸めと一致）、32 ビットのマージンは 3.37 倍——5つの入力（プール容量、破損率、稼働年数、scrub 周期、ブロックサイズ）のそれぞれがこれだけの倍率狂って初めて 33 ビットが必要となる；切り捨てられた CRC は短い CRC ではない——フル幅の CRC32C が単一ビット反転に対して検出漏れ 0 であることは構造的性質であり、切り捨て後は確率的検出に変質する。
- 傍証（実地調査、本プロジェクトでの検証は未実施）：btrfs のデフォルトチェックサムはまさに 32 ビット CRC32C（`BTRFS_CSUM_TYPE_CRC32 = 0`）であり、その 256 ビットスロットは任意の SHA256 / BLAKE2 のために残されている。
- ユーザー確定、原文および当日の導出は変更履歴に記録。

**未済の検査**：なし。

#### 確定項目 3：固定幅、非暗号化時もスロットを確保

**確定内容**：ポインタは最悪ケースに基づき固定幅とする——ヘッダ内の MAC（128 ビット）と nonce（96 ビット）は恒常的にスロットを占有し、暗号化を行わないボリュームではこれらを空きスロットとして維持し、種別フラグによる省略は行わない；これらのビットはすべてのボリュームで占有され続け、他のフィールドに転用されることもない。

**射程**：選択可能（オプショナル）なのは AAD の演算処理（暗号化オフなら計算しない）であり、フォーマットではない（`.claude/rules/fs-design.md`「暗号化は選択可能な演算であり、選択可能なフォーマットではない」）。メディアごとのレイアウト分岐開放メカニズムはこのセルを管轄できない：incompat ビットが管轄するのはリーダーがそのレイアウトを認識できるか否かであり、本セルが管轄するのは同一レイアウト内で単一のポインタ内部をどうエンコードするかである；先行して固定幅とし、後から可変長レイアウトを追加することは拡張ではなく、永久保守すべきパーサー、checker、クラッシュポイント再生をもう1組増やすことに他ならない。非暗号化時に MAC 16 バイト + nonce 12 バイト = 28 バイト / 論理 extent を空費することは、D4 (チェックサム位置) 確定項目 7 の計測条件（論理 extent のディスク占有は恒常的に 32768 バイト）に基づき `28 / 32768 = 0.0854%` である。可変長側が削減するのはその 28 バイトであり、1フィールドを削減することではない（アルゴリズム種別フィールド D9 (暗号化) 確定項目 12 第4項はもともと必要）；払うべき代償はパースが先頭から順次実行されねばならないことである：固定幅ポインタはオフセットにより直接 k 番目を特定できるが、可変長では不可能；「リカバリ走査時に部分破損したノードで破損した1件をスキップして読み取りを続行できるか否か」は推論であり、実測は未実施。暗号化オフ時は MAC 16、nonce 12 は恒常的に 0、リーダーが非ゼロに遭遇した場合は当該ポインタが存在する構造を破損と判定、I-2.4 (ヘッダチェックサムカバー範囲) がユニットヘッダ 29 バイトに与えた解釈と同一（実地審査 A3a は2箇所のリーダーに着地——多層 Code 2 ツリー内部エントリ、journal 新ルートセグメントポインタ：`crates/singlefs-core/src/pointer.rs` の `PointerHeadFieldOutsideTheFirstVersion::MacOrNonceNotZero`）；extent オフセットの 2 バイトは第1版で恒常的に 0（第1版ポインタはユニット全体を指す）、リーダーが非ゼロに遭遇した場合は当該ポインタが存在する構造を破損と判定、暗号化のオンオフとは無関係；checker は辿る全ポインタ上で判定し、core はディスク上のポインタヘッダを読む全リーダーで判定、着地点は [checks-owed.md](../checks-owed.md) 返済済み表の C581 (extent オフセット非ゼロに判定基準なし) 行にある。

**根拠**：

- 実験なし：固定幅を可変長に変更して節約できるのは空間であり、失われるのは「単一のポインタを単独で取り出してオフセットから直接解決できるか否か」であり、両者は異なる次元であり定量可能な数値はない。これを支えるのは `.claude/rules/fs-design.md` の2つの原則——「省スペースのために自己完結性を犠牲にしない」、およびフォーマット分岐の判定基準（「誤用」を「マウント不可」に変換できるかを問い、判定基準に「高速・省スペース」を含めることを禁じる）である。
- ユーザー確定、原文および 2026-09-12 の再確認は変更履歴に記録。
- 判定 2026-10-01 採択：第1版で extent オフセットが非ゼロの場合、当該ポインタを含む構造を破損と判定（`research/prompts/m3-inv-a-r1-main-verification.md` 第2節第11行）。

**未済の検査**：C210 (3つの「第2セット」がどの拡張分類に属するか未判定)。

#### 確定項目 4：位置エントリは 14 バイト

<!-- format-const: LOC_ENTRY = 14 stale=11 * 2|tree_entry=61 -->

**確定内容**：デバイス ID 4 バイト、物理オフセット 6 バイト（16 KiB スロット番号）、暗号文チェックサム 4 バイト => 位置エントリ 14 バイト。

| セグメント | 幅 | なぜこの幅か |
|---|---|---|
| デバイス ID | **4** | 上限は確定済み条項のいずれにも依存しない（D12 (対象メディア) はメディア層に上限を設けず、全リポジトリに dev id 回収条項なし => 狭い幅はプール生涯に出現したデバイス識別子の総数を制限する）、`.claude/rules/fs-design.md` の「省スペースのために自己完結性を犠牲にしない」に基づき広い方を選択；先例は確定項目 3 |
| 物理オフセット | **6**、**16 KiB スロット番号**としてエンコード | インデックスノードは 16384 アライメント、サイズは 16 KiB（D18 (ブロックに含める情報) 確定項目 9）=> 32 KiB ユニット番号エンコードでは半数のインデックスノードの配置先を表現不能、バイトオフセットでは 16 KiB 非アライメントの起点が表現可能になってしまう。6 バイトスロット番号は 4 EiB / デバイスをアドレス指定可能、同幅のバイトオフセットは 256 TiB のみ |
| 暗号文チェックサム | **4** | 確定項目 2 |

**射程**：物理オフセットと D3 (領域割り当て) 確定項目 7 は同一の座標系である（D3 (領域割り当て) 確定項目 3 根拠第2条が同一座標系を逐字要求）=> 両者は同時に確定し、個別の決定を禁じる。ツリーテーブルユニットポインタとルートレコード合計の2つの下流数値はこれに伴い変更されるが、ルートレコード合計はここには登録されず、唯一の権威ある箇所は D22 (ユニット原子性の合成手法) 確定項目 7 のフィールド表である。旧来の 11 バイトに対し、各位置エントリは 3 バイト増加、各ポインタは 6 バイト増加。

**根拠**：

- E97 (会計および割り当てレコードのエントリエンコード)：判定基準 7 は表現可能性により配置先エンコードを判定——16 KiB スロット番号エンコード時は全正当起点が表現可能、粗いユニット番号では半分が書き込み不能。
- ユーザー確定、原文および「この数値はこれまで決定プロセスを経ずに消費されていた」再確認は変更履歴に記録。

**未済の検査**：C238 (位置エントリが 11 から 14 へ変更後の派生値「53」の未バック走査)。

#### 確定項目 5：位置の権威はハイブリッド、中央マッピングが唯一の入口

**確定内容**：ハイブリッドを採用——

| コンポーネント | 確定内容 |
|---|---|
| 位置エントリ（確定項目 4 の 14 バイト） | **ヒントに降格**。幅とセグメント区分は不変、変わるのはその地位：読み取った場合は直接その配置先を試行可能だが、陳腐化している可能性がある |
| 中央マッピング | **デリファレンスと解放判定の唯一の入口**。移動時はここの1エントリのみを変更、「誰が私を参照しているか」を知る必要は一切ない |
| 状態分類 | いずれも権威態リストの第4項目ではない：マッピングはユニット自己記述走査から再構築可能であり、荷重受け面は依然として D20 (荷重受け面: ユニットの原子性と自己完結性) のユニットである |

確定に伴い有効化された3つの厳格な規則：

1. **解放は一律マッピングを経由し、ヒントを経由しない**；マッピング経由で当該マッピングエントリを検証した後、解放前に位置エントリ内のユニットチェックサムに基づきディスク読み取り検証を行う。**不一致を検出した場合はそのスロットを隔離し、パブリッシュは正常に成立**：論理的には通常通り解放（マッピングエントリを削除）、物理スロットは空きプールに返却せず隔離として記録しカウンタで報告。
2. **ユニットヘッダまたはヒントはブロック birth を保持しなければならない**——既に充足：D18 (ブロックに含める情報) 確定項目 7 のデータユニット種別識別セグメントに誕生代号 8 バイトがもともと含まれる。
3. **ヒント陳腐化によるマルチホップ率の実行時観測点が必要**（誤った分岐への進入は性能の崖であり、崖はエラーを出さない、`.claude/rules/fs-design.md` 厳格要件 4）。

**マウント状態におけるマッピングの読み取り方法**：第1版のマウント状態ではプールオープン時に中央マッピング全体をマウント状態へ読み込み（`crates/singlefs-core/src/mounted_read.rs` の `central_mapping_entries`）、以降のデリファレンスはこのメモリ内エントリを参照しマッピング読み取りは発行しない；中央マッピングが多層に成長した場合、プールオープン時にマッピングツリー全体をマウント状態へ読み込み、デリファレンスは従前通りメモリのみを参照しマッピングのためのデバイス読み取りは発行しない；書き込み側はパブリッシュごとに現在のマッピングツリーエントリ数と今回のパブリッシュで最大追加されるエントリ数に基づきマッピングツリー容量受入を行い、他ツリーの容量数値の偶然には依存しない。多層化後マッピングツリーにエントリ数上限はなく、構造的上限はヘッダの階層 1 バイトのみである；計画段階・ディスク書き込み前に今回実際に書き込むエントリに基づき形状を算出し、階層に収まらない場合は拒否（`HeightBeyondTheLevelField`）、別途エントリ数上限は設けない（スプリント 19 採用、メイン agent 確定）。「ヒント陳腐化による1回のデリファレンスが3回のデバイス読み取りを発行する」という数値はこの規則に準拠しており、根が葉を兼ねていた際に測定されたものであり、多層化着地時にテストケースで再測定された（マイルストーン「上書き・解放・ロールバック・再利用」クローズアウト表第28行）。割り当てレコードツリーもマウント時に同様にツリー全体をマウント状態へ読み込む；extent ツリーは全体読み込みを行わず、ファイルオープン時に位置に従い辿る（D8 (コアインデックス構造) 確定項目 14、ユーザー 2026-09-24 確定）。

**厳格規則 1 のディスク読み取り検証が読み取れない・不一致の場合の処置**：解放前に位置エントリに基づきディスク読み取りチェックサム検証を行い、読み取り自体の失敗または不一致の場合、まず1回再読み取りを行い、2回とも読み取れないまたは2回とも不一致の場合にのみ不一致として処置（再読み取り不一致の側はメイン agent 2026-09-25 確定：一時的な読み取り不良でユニットを永久隔離してはならない；スプリント 25 で実装済み）。2回の読み取り結果の種別が異なる場合（先が読み取り不可で再読み取りが不一致、またはその逆）は2回とも不一致として処置し、再読み取り時の様態を報告（スプリント 25 採用、攻撃ラウンドゼロ）。いずれかのレプリカで不一致が検出された場合、各ディスク上の当該ユニットの割り当てレコードは「割り当て済み」のまま維持され解放済みに変更されない（もう一方のディスク上のレプリカが一致していても維持され、各ディスクの台帳の対称性を保つ）——ディスク書き込みは再マウントを跨ぎ、受入において割り当て済みとして計上され別式には入らない。位置エントリがプール外のディスクを指している、または2つの位置エントリが同一ディスクを指している場合はマッピングエントリ破損として処置：いかなる書き込み前にも拒否し、ディスク上は不変。ディスクから直前版を再構築する際（`crates/singlefs-core/src/recovery.rs` の `rebuild_version`）、読み取れないデータユニットはその位置エントリをそのままコピーし内容は読まない、マウントは通常通り進行し、当該ファイルを読んだ時点でエラーとなる。ただし目撃がない場合は除く（選択されたバージョンがシステム構成に目撃されておらず、そのデータユニットが読み取れない場合、書き込み可能マウントは D23 (ジャーナルの役割と形式) 確定項目 14「選択されたルートの走査読み出しが失敗し、システム構成がそれを目撃していない場合は目撃されたバージョンへ戻る」に従い目撃されたバージョンへ戻る；C755 (リカバリと二種のマウントでロールバックを引き起こす失敗が異なる) の和集合でロールバックを判定、攻撃ゼロ回の文言、実装は実装バッチで行う）。

**射程**：D21 (権威態と派生態の境界) の「共有下で複数参照者がどこに存在するか」のリスト半分と、D26 (バックグラウンド整理と配置回収) 確定項目 5 の位置権威の3択を同時に収束させ、他箇所はすべて確定項目 5 にリンクする；元々は C34 (平文マッピング層の状態区分の3箇所での衝突) も収束させていたが、C34 (平文マッピング層の状態区分の3箇所での衝突) は 2026-09-07 に D9 (暗号化) 確定項目 5 が「scrub のみ」に方針変更したことで対象外となり、その収束は失われたが他の2箇所には影響しない。マッピングエントリのフィールド表は本セルにはなく確定項目 6 に属する。既知の3つのコーナーケースに番号はない：マッピングツリーの key レイアウトは D8 (コアインデックス構造) 確定項目 7 の付帯条件（重複排除ウィンドウが `T_dirty` で境界づけられる）を満たさねばならない；ヒントのリフレッシュ方針は「再書き込み時リフレッシュ」のみモデル化済み；キャッシュヒットの次元は測定値ゼロであり、E109 (位置権威三系統の実行時コスト) は回答不能と明記している。ブートストラップ免除の3種は確定項目 8 参照。厳格規則 1 のディスク読み取り検証が読み取れない・不一致時の処置は確定内容落着段落参照（C394 (解放判定がマッピングエントリ位置項内のユニットチェックサムを検証しない) の3問）；「まず1回再読み取り」は1回のみ再読み取りを行い、バックオフや多重リトライは行わない。

**根拠**：

- E109 (位置権威三系統の実行時コスト)：3系統を次元ごとに実測——ユニット移動コストはポインタ権威下では参照ツリー数に線形、中央マッピングおよびハイブリッド下では無関係；パブリッシュは葉の数に無関係な一定額を支払う；読み取りは1割から4割の増加にとどまる。絶対値はサンプリング点とともに引用が必要：主サンプリング点では参照ツリーとマッピングツリーが同一高であり、縮退点である。
- E96 (ハイブリッドアーキテクチャの整合性)：ヒントに基づき配置先を解決すると誤解放を引き起こすため、厳格規則 1 が必然的に導出された；その `hdr_key` 側は D18 (ブロックに含める情報) 確定項目 7 ではなく確定項目 3 のフィールド表を参照しており、「フィールド追加を要する」フォーマットコストは存在しない。
- ユーザー確定 2026-09-23：解放前にディスクを読み取り位置項内のユニットチェックサムを検証、不一致時はスロットを隔離しパブリッシュは成立（`research/prompts/m2-placement-falsepositive-r1-main-verification.md` 第3節 C394 (解放判定がマッピングエントリ位置項内のユニットチェックサムを検証しない) 判定セル：3つの厳格規則はいずれも検証要否に答えていなかった）、原文は変更履歴に記録。
- ユーザー確定 2026-09-24：マウント状態でのマッピング全体読み取りは当日の実装に準拠して条項化、多層マッピングはユーザーの同日別確定（マイルストーン2で多層マッピング読み取りを含む4ツリーの分割を実施）により全体読み取りに変更、スプリント 19 着地（C483 (マウント状態のマッピング読み取りとツリーノードロールバックに条項がない) ①、並行ライン2着地時に確定・条項なし）、原文は変更履歴に記録。
- ユーザー確定 2026-09-24：C394 (解放判定がマッピングエントリ位置項内のユニットチェックサムを検証しない) 3問——ディスク読み取り失敗時はまず1回再読み取り後に不一致処置、再構築時は読み取れないデータユニットを許容、隔離された側の割り当てレコードは割り当て済みに維持（判定 `research/prompts/m2-newq-r1-main-verification.md` 第3節 N1 (読み取れない場合はまず1回再読み取り)–N3 (一致しないものは割り当て済みに維持)、原文は変更履歴に記録）；実験なし：候補の読み取り数値は三者攻撃側担当のレプリカ上プロトタイプ（`research/prompts/m2-newq-r1-opus-model/`）由来であり、実験未登録。
- ユーザー確定 2026-09-24：多層マッピングはマウント時に全体をマウント状態へ読み込み、書き込み側にマッピングツリー容量受入を追加（三者 `research/prompts/m2-treesplit-r1-main-verification.md` T3）。
- ユーザー確定 2026-09-25：1レプリカのみ不一致時は両ディスクとも維持、位置項がプール外ディスクを指す場合は当日の拒否挙動を条項化（スプリント 16 継続レポート `m2-writepath-implementer-report.md` 第6節 Q2、Q3）、原文は変更履歴に記録；実験なし：いずれもディスク上フォーマットを変更せず処置のみを規定。
- メイン agent 確定 2026-09-25：`records/2026-09-24-里程碑二收尾调度.md` 第3節「一時的読み取り不良によるユニット永久隔離」行。
- 目撃がない場合は除く：ルートリング三方第 3 ラウンド判定 `research/prompts/m3-close-rootring-r3-main-verification.md` 第 4 節「どのセルにも入らなかったものの行き先」表で D19 (ブロックポインタ構造と幅予算) 確定項目 5 を見直したその行（防御側 Z5.2：目撃のない R 上のデータユニットが読み取れず、書き込み可能マウントは R を通常通りマウントする方式から前の版へ戻る方式に変わる）；ユーザー確定 2026-10-20（C755 (リカバリと二種のマウントでロールバックを引き起こす失敗が異なる)「三文とも補う」）、原文は変更履歴にある。実験なし：数値は三方レグのレプリカで計測、実験番号なし。

**未済の検査**：C248 (ツリーテーブルユニットのブートストラップ免除未記載)、C253 (66 048 の固定オーバーヘッドに関する2つの kb の矛盾言及)、C133 (マイグレーション転送情報にキャリアなし)、C279 (パッキング後にスキャン再構築が旧ユニットとスロットを同時に視認)；C417 (位置ヒントのリフレッシュ方針は1種のみモデル化)、C418 (位置権威三系統のキャッシュヒット次元測定値ゼロ)——根拠の2実験が回答不能と自認したセル；C543 (直前版再構築がインスタンス表第0シートのみ保持)。

#### 確定項目 6：中央マッピング key は種別ごと書き込みシーケンスキー再利用（厳格化後）を採用

**確定内容**：key は固定幅、セグメントごとに配置、定義と幅は E134 (マッピング key フォーマットコスト・スロットベースラインと2つの新系統) の `class_reuse_tagged` 系統と逐字同一——

| 種別 | key（固定幅、セグメントごと） | 幅 |
|---|---|---|
| Code 1 データユニット | 種別タグ 1 + 誕生ツリー 8 + 誕生 txg 8 + 書き込みシーケンス 10（インスタンス番号 4 + トランザクション番号 6） | 27 |
| Code 2 / Code 3 | 種別タグ 1 + 誕生ツリー 8 + 誕生 txg 8 + インスタンス番号 4 + 誕生シーケンス番号 4 | 25 |

value は確定項目 4 / 確定項目 5 に準拠：位置エントリ × w レプリカ（w = 2 時は 28 バイト）。パディングとエントリ総幅は確定項目 10 に属する。

**伴う4つの厳格化**（3ラウンドの論争で「1敗を記録 / 厳格化のみ許可 / 厳格化箇所を明記」に基づき追加、確定後も同様に遵守）：

- 種別タグは命名対象オブジェクトの D18 (ブロックに含める情報) 確定項目 11 登録表におけるコードを採用、D27 (小データパッキングコンテナ) スロット内のオブジェクトは恒常的に Code 1；パッキングおよびコンテナ全体再書き込み時、スロット内 key の各セグメントはパッキング前の版のユニットヘッダの値をそのまま採用。
- 誕生ツリーは種別識別セグメントの誕生ツリーを採用（Code 3 オフセット 43）、誕生 txg は誕生代号を採用（Code 3 オフセット 73；Code 1、Code 2 も同様に誕生代号を採用）、インスタンス番号は書き込みシーケンスの先頭 4 バイトを採用。
- D16 (パブリッシュセマンティクス) 確定項目 5 末尾の「1トランザクションは最大1ユニットのユーザーデータを書き込む」は全 Code 1 ユニットをカバーするものとして解釈（事前割り当て、切り捨てられた末尾ユニット、ホールパンチ両端、reflink 両端のコピーも含む）、該当文への明記が望ましい。
- 誕生シーケンス番号は key の1セグメントとしてのみ機能し、固定点多版の新旧選択には用いない——新旧選択に用いると D18 (ブロックに含める情報) 確定項目 1 で否決された「陳腐化レプリカ識別のためのシーケンス番号」に該当してしまう；D18 (ブロックに含める情報) 確定項目 1 は書き込みシーケンスの先例に倣い射程注記を追記する必要がある。

**射程**：管轄するのは key と value の構成セグメントと各幅であり、パディングとエントリ総幅（確定項目 10）、収容種別（確定項目 8）、誕生シーケンス番号の発行方法（確定項目 9）は対象外。D15 (フォーマット凍結ポリシー) 第2層に着地。Code 1 側の唯一性は「Code 1 の書き込みシーケンスは既に唯一のバージョン識別子である」という推論に依存しており、これは D16 (パブリッシュセマンティクス) 確定項目 5 末尾のトランザクション分割規律に基づく（D23 (ジャーナルの役割と形式) 確定項目 7 の1レコード1トランザクション番号と協調）；該当箇所は 2026-09-13 に C123 (ユーザー確定変更後の未レビュー) の再確認により収束、分割規律は不変、推論は成立、残余の基準衝突は C310 (トランザクション分割規律とレコード数基準の衝突) に記録。「暫定」記録：本セルは既存のコストと性能数値に基づき選定されたものであり、新たな反証により覆された場合は再開され、論証された唯一解ではない。

**根拠**：

- E137 (マッピング key 形態の性能ギャップ)：fsync ごと、バックグラウンド負荷、ランダム読み取りの3種負荷における3系統の格差——種別ごと書き込みシーケンスキー再利用は fsync ごとの多数のセルとランダム読み取りの全セルでリード、全域ランダム上書き、ファイル削除、単一ファイルスナップショット削除で劣後；ユーザーは本表を精査しこれを採択。⚠️ モデル内の物理配置先は書き込み順序そのものであり、マイグレーションやスナップショット削除に最も有利な状況、老朽化後の縮退は未測定。
- E133 (マッピング key 各候補のフォーマットコスト)：2ラウンドの三者論証に出現した各 key がフォーマット上で支払うバイト数、7候補のポインタ幅、エントリ幅、ファンアウトを提示。
- E134 (マッピング key フォーマットコスト・スロットベースラインと2つの新系統)：本セルが採択した系統はその `class_reuse_tagged` であり、厳格化で追加された 1 バイト種別タグと D27 (小データパッキングコンテナ) スロットの2種のベースラインを行単位で報告。
- E136 (マッピング key 分岐のコスト行)：ユーザー確定前に欠落していた4つのコスト数値——Code 2 ヘッダへの 4 バイト追加は D18 (ブロックに含める情報) 確定項目 14 の3段階の現実のヘッダ幅下でファンアウトを1段階も低下させず、マッピングツリー層数は狭いプール規模区間でのみ1層異なるにとどまる。
- 3ラウンドの論争判定と3ラウンド論証の経緯は変更履歴に記録；ユーザー確定、原文は変更履歴に記録。

**未済の検査**：C288 (マッピング key 確定の連随変更が他決定に未反映)、C289 (Code 2 の誕生ツリーがどのフィールドを採用するか未記載)、C163 (固定点各巡で書き込みシーケンス同一によるタイブレーク失敗)、C286 (読み取り専用後 remount で新インスタンス番号を取得するか否か)、C287 (切り替えでオープンなチェックポイントのトランザクション養子縁組後にクラッシュ)、C310 (トランザクション分割規律とレコード数基準の衝突)。

#### 確定項目 7：ポインタヘッダは誕生 (ツリー, txg) を保持、各 8 バイト

**確定内容**：ポインタヘッダは MAC 16 + nonce 12 + アルゴリズム種別 1 + 圧縮アルゴリズムコード 1 + 圧縮後長 2 + extent オフセット 2 の後に誕生ツリー 8 と誕生 txg 8 を保持、ヘッダ合計 **50**；ポインタは従前通り固定幅（確定項目 3）。圧縮の 3 バイトは圧縮アルゴリズムコード 1 バイト（登録表は D9 (暗号化) に配置、0 = 非圧縮、第1版は 0 を記述）と圧縮後長 2 バイト（第1版は 0 を記述、[0, 32768) のユニット内長さを表現可能）である。

**射程**：定めるのはどの 16 バイトがポインタヘッダに存在するかであり、ポインタ総幅（確定項目 8 と確定項目 11 の合成による 88 / 86）は規定しない。圧縮の 3 バイトは D9 (暗号化) 確定項目 13 第2条の厳格要件のフォーマットキャリアを補填する——圧縮と暗号化の順序がフォーマット上明確に定義されることを要求するが、それまでリポジトリには「この暗号文は圧縮済みか、圧縮前長はいくらか」を担持するフィールドが皆無であった。⚠️ メイン agent は非保持を推奨したが（圧縮はオプショナルな演算であり、「暗号化のために確保されたスロットは全ボリュームで保持」の原則は暗号化のみを対象とする）、ユーザーはポップアップ確定で保持を選択、三者論証は経ておらず、覆すことは可能。ポインタからこの2セグメントを取得するコンシューマは4つ：D5 (スナップショット / 領域会計メカニズム) の O(1) 判定が誕生 txg を読み取る；deadlist エントリはバージョン識別子で記録され誕生ツリーをポインタから取得；中央マッピング key；AAD 形態はポインタがこれら2フィールドのスロットを持つことを要求し値は独立検証を経る（D9 (暗号化) 確定項目 16 はポインタからの読み戻しを逐字禁止）。新規プールでのファイル作成のバイト列を変更：はい——全ポインタに 3 バイト追加。

**根拠**：

- E128 (ポインタへの誕生 (ツリー, txg) 付加後のポイントルックアップコスト)：会計ツリー内部エントリ拡幅後は4ラウンドとも低速化、inode ツリーのペアは4ラウンド不安定で引用不可；測定対象はこの 16 バイトのみであり、本セルのコストの下限である。
- 1ラウンドの三者論証、判定は `research/prompts/d19-item7-r1-main-verification.md`：反駁側担当が確定項目 8 当時の末文が「ポインタ内に保持するか参照エントリ内に保持するか」に余地を残しており双方の解釈が可能と指摘。
- ユーザー確定（「16 バイトはポインタ内に保持」）、原文および候補乙・丙の脱落経緯は変更履歴に記録。

**未済の検査**：C295 (ポインタ内の key セグメントが AAD のルックアップパスに含まれるか否か)、C296 (journal リプレイ時にマッピング key が揃わない)、C297 (inode 内部エントリ内の誕生ツリー重複の処置未定)、C237 (Code 1 / Code 2 のポインタ内に誕生 (ツリー, txg) のスロットがない)。

#### 確定項目 8：全ポインタがマッピング経由、参照側は種別ごとにマッピング key セグメントを担持

**確定内容**：中央マッピングは Code 1、Code 2、Code 3 の3種別ユニットを収容；ユニットを指すポインタは当該種別マッピング key の種別タグを除く各セグメントを担持、種別タグはルックアップパスから与えられる。

| 指される種別 | ポインタが担持（確定項目 6 の key 各セグメント） | ポインタ幅 |
|---|---|---|
| Code 1 | 誕生ツリー 8 + 誕生 txg 8 + 書き込みシーケンス 10 | データポインタ **88** |
| Code 2 / Code 3 | 誕生ツリー 8 + 誕生 txg 8 + インスタンス番号 4 + 誕生シーケンス番号 4 | ノードポインタ **86** |

**マッピング対象外の3種別（ブートストラップ免除）**：マッピングツリー自身のノード、ツリーテーブルユニット、インスタンス表ユニット。それらの親ポインタ（マッピングツリー根、ルートレコード内のツリーテーブルポインタおよびインスタンス表ポインタ）内の位置エントリは権威でありヒントではない、確定項目 5 の「唯一の入口」および厳格規則 1 はこれら3種別には適用されない；3種別ともプール単位の単一インスタンスであり、参照者は1つのみ。これら3種別の親ポインタは Code 2 / Code 3 を指すノードポインタと同幅の 86：位置エントリ 14 × 2 は権威、インスタンス番号 4 + 誕生シーケンス番号 4 は被参照ユニットヘッダの値をそのまま設定しマッピング検索には用いない；パーサーは被参照ユニットの種別によりポインタ幅を決定し、相手が免除種別か否かでは区別しない。免除3種別以外のユニット（Code 1 データユニットおよび Code 2 ツリーノード——extent ツリー根、inode ツリー根、inode 葉コンテナ）は位置ヒントが読めない場合は一律マッピング経由でフォールバック；免除3種別（マッピングツリー根、ツリーテーブル、ルートレコードの連鎖）はフォールバックしない。

**射程**：判定するのはマッピングがどの種別を収容しポインタが key のどのセグメントを担持するかであり、ポインタ幅には依存しない（うち誕生 (ツリー, txg) 16 バイトがポインタヘッダに存在することは確定項目 7 で確定）。Code 1 のみ収容する系統は2つの正確性において劣後：Code 2 / Code 3 のブロックも同様に deadlist / livelist に入り、D5 (スナップショット / 領域会計メカニズム) は deadlist エントリがバージョン識別子で記録されマッピング経由で解放されると規定、マッピングに入らなければ配置先で記録するしかなく、整理やドライブ脱落再構築で移動された後に旧配置先で解放されてしまう；読み取り専用スナップショットから参照される Code 2 / Code 3 を移動する場合、スナップショットツリーの祖先を書き換える必要があり、新 birth を取れば参照区間に違反、旧 birth を維持すれば `birth` の定義文に違反する。全ポインタがマッピング経由であればこれら双方で劣後しない：deadlist / livelist はマッピング key で記録、移動時はマッピングの1エントリのみ変更、移動されたユニットは1バイトも変化せず birth も不変。代償：fsync ごとおよびバックグラウンドのユニットあたりマッピングブロック数は Code 1 のみ収容時より1〜2割以上増加、ランダム読み取りミスも増加；本列はさらに2項目を過小評価（派生ツリーの Code 2 のマッピング内削除・挿入、extent ノード自身のヒント陳腐化によるマッピング読み取り）。親ノード再書き込み時は位置ヒントにマッピング位置を書き込む（C417 (位置ヒントのリフレッシュ方針は1種のみモデル化) 第1版は「再書き込み時リフレッシュ」を採用）。checker と GPU 判定器は免除3種別以外の Code 1 / 2 / 3 位置エントリをヒントとしてのみ扱う：走査時はまずヒントに従い読み取り、両方不一致の場合はマッピング位置に従い読み取り、参照集合にはマッピング位置を記録。

**根拠**：

- E137 (マッピング key 形態の性能ギャップ)：2つの構成（Code 1 のみ / 全ポインタがマッピング経由）のコスト差をセルごとに実測、本セルのコスト行全体はここから引用。
- E134 (マッピング key フォーマットコスト・スロットベースラインと2つの新系統)：両構成下のポインタ幅、エントリ幅、ファンアウトを提示、88 / 86 はその2行に確定項目 7 の圧縮 3 バイトを加えたもの。
- E128 (ポインタへの誕生 (ツリー, txg) 付加後のポイントルックアップコスト)：確定当日に記録漏れし翌日補填されたポイントルックアップコスト、本セルのコスト行に記録。
- 2ラウンドの三者論証、判定は `research/prompts/d19-item8-r1-main-verification.md` および `research/prompts/d19-item8-r2-main-verification.md`（第1ラウンドは2つのクラウド側担当が正反対の結論、ユーザーが再推論を要求、第2ラウンドで立場逆転）。
- ユーザー確定、原文および両救済策がなぜ確定済み条項群の変更を要したかは変更履歴に記録。
- 2026-09-23 判定 `m2-lines234-code-r1-main-verification.md` ツリーノードがマッピング経由でフォールバックすべきか否かの判定セル——攻撃側担当が同一の損傷下でデータユニットが1ホップ多く費やして正しいバイト列を読み戻せたのに対し、extent ツリー根が `open_pool_for_read` でプール全体をマウント不能にしたことを実測。
- 判定 `research/prompts/m4-gc-r1-main-verification.md` 第3節 G3：書き込みパス、再構築、checker を「マッピングが唯一の入口」に従いリファクタリング（乙を採用）、親ノード再書き込み時はマッピング位置を記述、checker と GPU 判定器は位置エントリをヒントとしてのみ扱う。第7節ユーザー確定「不適切、論理が通らない、または不自然であればリファクタリングせよ」、原文は変更履歴に記録。

**未済の検査**：C293 (全ポインタがマッピング経由化後に E137 が過小評価した2項目)、C292 (Code 2 識別セグメントのコンシューマゼロの根拠喪失)、C248 (ツリーテーブルユニットのブートストラップ免除未記載)（C280 (中央マッピングがどの種別ユニットを収容するか条項なし) より統合）。

#### 確定項目 9：誕生シーケンス番号の発行方法と順序

**確定内容**：誕生シーケンス番号は 0 から開始；同一ツリー内で Code 2 または Code 3 ユニットを1つ書き出すごとに 1 加算（両種別で単一カウンタを共有）；スコープが次のチェックポイントに移った際にゼロクリア；同一チェックポイント内で同一ノードが固定点により2回目に再書き込みされた場合は新番号に変更。発番順序はコミット内生ブロックがオープンなクラスタセグメントからバンプされる順序と同一——ツリー ID 昇順、ツリー内は葉が先で根が後、同一層内は key 昇順、中央マッピングツリーが最後から2番目、ツリーテーブルユニットが最後。

**射程**：順序自体の唯一の登録箇所は D3 (領域割り当て) 確定項目 10 であり、ここには重複して保持しない。誕生シーケンス番号は中央マッピング key の1セグメントであり（確定項目 6）、key はユニットが書き出される前に確定していなければならないため、発番は配置先の割り当て順序に束縛される：実装におけるカーソルは1つのみ、checker が検証する順序も1組のみ。key の1セグメントとしてのみ機能し、固定点多版の新旧選択には用いない（確定項目 6 第4の厳格化）。ユーザーポップアップ確定、三者論証は経ておらず、覆すことは可能；メイン agent の推奨も本層である。新規プールでのファイル作成のバイト列を変更：はい——新規プールでのファイル作成で書き出される11個のコミット内生ブロック（デフォルト幾何；幾何により個数は変動、リング 1024 MiB 時は 13 個）の配置先と誕生シーケンス番号はこれに準拠して決定、各ブロックの値は D3 (領域割り当て) 確定項目 10 ⑤ の表を参照。

**根拠**：

- 実験なし：順序と初期値自体に定量可能な数値はなく、発番と配置先が単一ルールを共有することのコストは D3 (領域割り当て) 確定項目 10 側にある；本セルを支えるのは「key はユニットが書き出される前に確定していなければならない」推論に加え、確定項目 6 厳格化論証における文言（同一コンテナが2つのチェックポイントでそれぞれ誕生シーケンス番号 0 を取得し、余分に得られるのは実行時マッピングの非衝突のみ）が再書き込み時改番下でのみ成立することである。
- ユーザー確定、原文は変更履歴に記録。

**未済の検査**：C291 (誕生シーケンス番号の割り当て規則が実験ソースにのみ存在)。

#### 確定項目 10：マッピング key は一律 27、エントリは一律 55

**確定内容**：中央マッピングの key は**一律 27 バイト**——Code 1 の (種別タグ 1, 誕生ツリー 8, 誕生 txg 8, 書き込みシーケンス 10) は自然に 27；Code 2 / Code 3 の (種別タグ 1, 誕生ツリー 8, 誕生 txg 8, インスタンス番号 4, 誕生シーケンス番号 4) は 25 バイトであり、**末尾を 27 にゼロパディング**。したがって中央マッピングエントリは**一律 55 バイト**（key 27 + value 28、value = 位置エントリ 14 × 2）。マッピングツリーノードヘッダの key 区間も同様に 27 で保持。

**射程**：連動する2箇所がこれに伴い固定幅に復帰——D5 (スナップショット / 領域会計メカニズム) 確定項目 6 の疎なサイドテーブルエントリは固定幅 31（マッピング key 27 + カウント 4）、D6 (スナップショット実現モデル) 確定項目 2 の livelist エントリは固定幅 36（ヘッドツリー ID 8 + マッピング key 27 + イベント種別 1）。種別ごとに可変長にすると D8 (コアインデックス構造) 確定項目 11 の2つの汎用ルールが同時に崩壊する：「固定幅エントリの連続配置」と「宣言長 = エントリ数 × エントリ幅」——Code 2 のエントリ幅フィールドは単一の値しか持てず、2幅にするには番兵が必要となり、番兵を設けるとノード内の固定幅二分探索が失われ、スキャン走査時にマッピングノードを先頭から順次解決せねばならなくなる。代償は Code 2 / Code 3 エントリごとに 2 バイト増加、新規プールでのファイル作成で 10 バイト増加。ユーザーポップアップ確定、三者論証は経ておらず、覆すことは可能；メイン agent の推奨も本層である。新規プールでのファイル作成のバイト列を変更：はい。

**根拠**：

- E145 (Code 2 自己記述ヘッダとマッピングツリー key 幅のコスト)：単一幅の系統を測定済み；「2幅 55 / 53、key 先頭バイトによる自己境界画定」はそのラウンドで未測定の第4の形態であり、それを確定内容とすることは成果物のない数値を引用することに等しい。
- ユーザー確定、原文は変更履歴に記録。

**未済の検査**：なし（C321 (Code 2 ヘッダのエントリ幅が単一値であるのに対しマッピングツリーエントリが種別ごと2幅) は 2026-09-14 にユーザー確定でマッピング key が固定幅 27 に復帰したことで返済済み、返済済み表参照）。

#### 確定項目 11：ポインタのオフセット表、位置エントリの順序、マッピングツリー根の配置場所

**確定内容**：ポインタはオフセット 0 からセグメント順に配置——

| セグメント | オフセット | 幅 |
|---|---|---|
| MAC | 0 | 16 |
| nonce | 16 | 12 |
| アルゴリズム種別 | 28 | 1 |
| 圧縮アルゴリズムコード | 29 | 1 |
| 圧縮後長 | 30 | 2 |
| 起点からの extent オフセット | 32 | 2 |
| 誕生ツリー | 34 | 8 |
| 誕生 txg | 42 | 8 |
| 位置エントリ × 2、デバイス識別子昇順 | 50 | 14 × 2 |
| key 末尾セグメント：Code 1 書き込みシーケンス 10；Code 2 / Code 3 インスタンス番号 4 + 誕生シーケンス番号 4 | 78 | 10 / 8 |

=> Code 1 へのポインタは **88**、Code 2 / Code 3 へのポインタは **86**。位置エントリはデバイス識別子昇順でソート、I-2.5 (位置エントリのデバイス識別子昇順) として確立。**中央マッピングツリーの根はルートレコードに配置**：ルートレコードは 86 バイトの中央マッピングツリー根ポインタを保持し、ツリーテーブルには入らない；ルートレコード合計 **457** バイト。ツリーテーブル第1版は **12** エントリ。

**射程**：定めるのは順序のみ——セグメントの集合と幅は確定項目 3 / 4 / 7 / 8 で確定済み。圧縮の 3 バイトはアルゴリズム種別の直後に続き、「この暗号文がどう生成されたか」のグループに属する；extent オフセットの前に配置するのは暗号化と圧縮の4フィールドを一連のセグメントとするためである。ルートレコード合計の唯一の権威ある箇所は D22 (ユニット原子性の合成手法) 確定項目 7 のフィールド表、ツリーテーブル第1版の該当エントリは D5 (スナップショット / 領域会計メカニズム) 確定項目 11 の deadlist 追加に伴う。ユーザーポップアップ確定、三者論証は経ておらず、覆すことは可能。新規プールでのファイル作成のバイト列を変更：はい——全ポインタのセグメントオフセットと幅、ならびにルートレコードの合計。

**根拠**：

- 実験なし：オフセット表自体に第2の選択肢はなく、セグメントの集合と幅は確定済みであり、本セルは順序を整えるのみで定量可能な数値はない。位置エントリのデバイス識別子昇順は2つの位置エントリに確定的順序を与え不変条件として確立するためである。
- ユーザー確定、原文は変更履歴に記録。

**未済の検査**：なし（C304 (インスタンス表チェーンポインタレコードが幅 83 のポインタを収容できない) は 2026-10-11 に返済済み、返済済み表参照）。

#### 確定項目 12：Code 2 / Code 3 のマッピングエントリは day-1 で書き出し、固定点はチェックポイントあたり1回書き出し

**確定内容**：day-1 での書き出しを維持（新規プールでのファイル作成で 6 エントリ）。固定点を固定化：各 Code 2 ノードおよび Code 3 コンテナは、1つのインスタンスが1つのチェックポイント内で1回のみ書き出し——パブリッシュ末尾に一括 COW；第1版は挿入中並行 COW を禁止（D3 (領域割り当て) 確定項目 5 が許容した実装を第1版では不採用）。

**射程**：⚠️ 将来挿入中並行 COW が開放された際も、D8 (コアインデックス構造) 確定項目 13 は従前通り遵守されねばならない：同一パブリッシュ内で同一 key は最終状態に基づき葉に1回のみ着地。失敗検査はユニット単位で計上し、固定点ラウンド数では計上しない。本セルが決定するのは書き込み要否と書き込み頻度であり、key 形態（確定項目 6）やエントリ幅（確定項目 10）は規定しない。確定時にリポジトリには解放と再利用の実験が存在せず、もう一方の経路は「解放の実験が整ってから再考」として残されたが、そのトリガー条件は本セルに記述されていない。

**根拠**：

- E81 (コミットの固定点)：「ダーティノードは今回のパブリッシュで1回のみ COW」という解釈の出所であり、この解釈下で固定点が収束する根拠；これに従い Code 2 と Code 3 を保護することで、同一チェックポイント内での同 key 2版による劣後は再発しなくなる。
- ユーザー確定、原文は変更履歴に記録。

**未済の検査**：C352 (day-1 マッピングエントリ書き込み確定の再開条件にリマインダーなし)、C163 (固定点各巡で書き込みシーケンス同一によるタイブレーク失敗)。

#### 確定項目 13：ブロックポインタは2セグメントに分割

**確定内容**：**ブロックポインタは2セグメントに分割：1つの「ヘッダ」が論理 extent に追従し、N 個の「位置エントリ」が物理配置先に追従する。**
論理属性（MAC、nonce、アルゴリズム種別、起点からの extent オフセット）はヘッダにあり、論理 extent ごとに1つ；
物理属性（配置先、および鍵を持たない側が破損検出するための暗号文チェックサム）は位置エントリにあり、物理位置ごとに1つ。

**射程**：分割により「ポインタ総幅」が制約から予算へと降格され、D4 (チェックサム位置) のセルは2セグメント形態へと書き改められた。定めるのはどの属性がどのセグメントに属するかのみであり、各セグメントに含まれるフィールド（確定項目 4 / 7）や順序（確定項目 11）は規定しない；ヘッダに後から追加された誕生ツリーと誕生 txg（確定項目 7）も論理属性である。「ポインタ外への移動」は余分な I/O を意味しない：D4 (チェックサム位置) が要求するのは親ノードが子ノードチェックサムを保持することであり、どの構造体に住むかではなく、どのノードに住むかを制約している。

**根拠**：

- D2 (RAID ストライプポリシー) が「複数レプリカは同一暗号文を N 回書き込む」と確定 => 全レプリカは同一 nonce を共有せねばならない；ポインタごとに各自保持することは同一数値を N 回複製することに等しく、あるレプリカが誤記された場合に同一暗号文から相互に相容れない2つの復号結果が生じる。判定可能形式は I-6.7 (レプリカ間での nonce 共有)。
- 現役実装が同一の形状を示す（2026-08-28 実地調査により逐字確認、本プロジェクトでの検証は未実施）：bcachefs の `struct bch_extent_ptr` はちょうど 64 ビット（`offset:44, dev:8, gen:8` プラス4つのフラグ）、チェックサム / nonce / 圧縮情報は同一 bkey 内の独立した可変長エントリ（`bch_extent_crc32/64/128`）に移動され、1つの crc エントリがそれ以降の全 ptr エントリに連続して適用；nonce 自体は永続化されず、`fs/bcachefs/checksum.h` の `extent_nonce()` は `bversion`（96 ビット、もともと bkey ヘッダに存在）に crc ごとの小さなオフセット（`crc64.nonce:10` / `crc128.nonce:13`）を加えて即時計算。
- 実験なし：2セグメントへの分割はフォーマット構造のトレードオフであり、確定済みルール（レプリカ間での nonce 共有）と現役実装の形状に支えられており、定量可能な数値はない。

**未済の検査**：なし。

#### 確定項目 14：位置エントリは個別に列挙、単一起点から幾何学的計算で導出しない

**確定内容**：N 個の位置エントリを個別に列挙し、それぞれデバイス識別子を保持；単一起点にサイズを加算して残りの各列位置を導出することはしない。

**射程**：管轄するのは位置エントリの表現方法であり、含まれるセグメント（確定項目 4）は対象外。前提となる基準は D2 (RAID ストライプポリシー) 確定項目 1 が「可変幅」の粒度を**都度変更可能**と定めたことである——フォーマット側で物理位置を個別に列挙し、それぞれデバイス識別子を保持し、chunk マッピングテーブルを挟まないことは、まさにこの演繹が必要とする前提である。D2 (RAID ストライプポリシー) 確定項目 1 と同型：得られるのは「誤ったディスクを指さないこと」であり、「障害ドメインの独立性」ではない。

**根拠**：

- ZFS は「1つの DVA + asize」から RAID-Z の全列位置を導出可能（`BP_GET_ASIZE` が3つの DVA の asize を合算）であるが、当該 top-level vdev の子デバイス数が**プール作成時に固定**されていることが前提である（実地調査、本プロジェクトでの検証は未実施）；D2 (RAID ストライプポリシー) 確定項目 1 はこの前提を否定した。
- 実験なし：導出可能か否かはストライプ幅の粒度により直接決定される推論であり、定量可能な数値はない。

**未済の検査**：なし。

#### 確定項目 15：起点からの extent オフセットの値域と第1版の値

**確定内容**：起点からの extent オフセットの値域は**最大 extent サイズ**により上限が画定され、ファイルサイズによってではない；本プロジェクトの値域は [0, 32768) であり、**15 ビットで十分**である。第1版では本フィールドは恒常的に 0——分割は予約機能であり第1版では無効化；ビットは確定項目 3 の「固定幅スロット確保」に従い維持、理由はポリシーであり使用量ではない。

**射程**：言及しているのは値域であり、幅 2 バイトは確定項目 11 のオフセット表にある。D9 (暗号化) 確定項目 12 第2項はファイルサイズに基づき 32–64 ビットと見積もったが、過大評価であった。本プロジェクトで値域がより狭い根拠は D4 (チェックサム位置) 確定項目 7 である：ディスク上の extent 占有は恒常的に = ユニット = 32768。

**根拠**：

- 現役実測（2026-08-28 `fs/bcachefs/extents_format.h` を実地調査、本プロジェクトでの検証は未実施）：`crc32.offset:7` / `crc64.offset:9` / `crc128.offset:13`、付随する `CRC128_SIZE_MAX (1U << 13)` => 本フィールドの値域は最大 extent サイズにより上限が画定される。
- 実験なし：値域は2つの確定済み条項を除算して得られる算術であり、定量可能な数値はない；分割を第1版で無効化するか否かは D4 (チェックサム位置) 確定項目 7 が決定。

**未済の検査**：なし。

```

**出处 `.claude/kb/decisions/19-ブロックポインタ構造と幅予算.ja.md:311-321`（整段抄，未转述）**

```markdown
### 他の決定との連鎖

| 決定 | 連鎖点 |
|---|---|
| D4 (チェックサム位置) | 確定項目 13 は「ポインタ総幅」を制約から予算へと降格させ、D4 (チェックサム位置) の該当セルを2セグメント形態へ書き改めた；論理 extent のディスク占有恒常 32768 は D4 (チェックサム位置) 確定項目 7 から与えられ、確定項目 3 と確定項目 15 はその上に成立している |
| D2 (RAID ストライプポリシー) | 確定項目 1 と確定項目 14 の前提はストライプ幅可変の基準であり、D2 (RAID ストライプポリシー) 確定項目 1 により都度変更可能と定められている |
| D9 (暗号化) | nonce キャリアは確定項目 13 により半分確定（ヘッダに属し、論理 extent ごとに1つ）；アルゴリズム種別と圧縮の各セグメントの登録表は D9 (暗号化) にあり、AAD 期待値をポインタから読み戻すことは禁止（D9 (暗号化) 確定項目 16） |
| D12 (対象メディア) | D12 (対象メディア) 確定項目 4 のマウント照合表において、デバイス識別子の不一致はマウントを拒絶せねばならず、根拠は確定項目 1 の位置エントリが dev を担持することである。⚠️ 当該判定基準が述べているのは「それを消費する**アドレス算術**がデバイス識別子を担持しているか否か」であり、btrfs のアドレス算術にはデバイス識別子が含まれており chunk ツリーにあるだけである；判定基準は成立するが、「担持 => ポインタ内になければならない」という飛躍は崩れる |
| D15 (フォーマット凍結ポリシー) | ポインタフィールドレイアウトは D15 (フォーマット凍结ポリシー) 確定項目 7 の第2層である；確定項目 6 の中央マッピングエントリフィールド表も第2層に位置する（D15 (フォーマット凍結ポリシー) 確定項目 1 の根拠が逐字指定） |
| D8 (コアインデックス構造) | ポインタ幅は内部エントリおよびツリーテーブルエントリに入り、D8 (コアインデックス構造) 確定項目 6 / 8 の複数の幅は確定項目 7 / 8 に伴い変更される；幅広ポインタが成立するか否かは D8 (コアインデックス構造) 確定項目 2 の 16 KiB ノードと結合している |

```

**出处 `.claude/kb/decisions/19-ブロックポインタ構造と幅予算.ja.md:322-325`（整段抄，未转述）**

```markdown
## 過去の版

D19 (ブロックポインタ構造と幅予算) の履歴項目は [decisions-history.md](../decisions-history.md) に集約されている。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:1-13`（整段抄，未转述）**

```markdown
## D19 块指针的结构与宽度预算 —— 已定

**中文** · [English](19-block-pointer-structure-width-budget.en.md) · [日本語](19-ブロックポインタ構造と幅予算.ja.md)

D19（块指针的结构与宽度预算） 管块指针这一格：指针分几段、每段有哪些字段、各多宽、按什么次序排；位置条目与中央映射谁是解引用与释放判定的权威；中央映射条目的 key 与 value 怎么编码；映射 key 的哪几段跟着指针走，出生序号怎么发。不管的：单元头的字段表与单元类型码在 D18（块里携带什么信息）；节点大小与树内条目布局在 D8（核心索引结构）；落点坐标系与分配记录的 key 在 D3（空间分配） 已定项 3 / 7，提交内生块的 bump 次序在 D3（空间分配） 已定项 10；校验和覆盖多大、逻辑 extent 与单元的关系在 D4（校验和位置）；加密算法、nonce 怎么产出、AAD 期望值从哪来在 D9（加密）；根记录与系统配置的字段表在 D22（单元原子性怎么合成） 已定项 7 / 9；条带宽度的粒度在 D2（RAID 条带策略） 已定项 1。

<!-- doc-lint:registry name-col=2 -->

| 编号 | 简称 |
|---|---|
| N1 | 读不出先重读一次 |
| N3 | 对不上的留在已分配 |

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:34-47`（整段抄，未转述）**

```markdown
#### 已定项 1：位置条目带设备身份

**定案**：位置条目里带设备身份（dev id），每个物理位置各带一份；不引入「逻辑地址 + chunk 映射表」这层间接。

**射程**：管「带不带」，不管宽度（已定项 4）与次序（已定项 11）。D2（RAID 条带策略） 已定项 1 的「逐次可变」是这一格正推那一侧的前提，而带 dev 在两种粒度下都成立（它是能力更强的那一侧）⇒ 粒度定案只影响位置条目能不能被压缩表达，不影响带不带。宽指针成立与否与 D8（核心索引结构） 的节点大小耦合：同样 40 字节的指针头部，4 KiB 节点付 +98%、16 KiB 节点只付 +14%，D8（核心索引结构） 已定项 2 取 16 KiB 并钉成常量。E20（指针变宽的 CPU 缓存代价） 量的是 40 字节的头部，定案是 50（已定项 7 / 11），多出来的那几字节 E128（指针带出生 (树, txg) 之后的点查代价） 量到记账树点查慢 1.8%–4.5%，前提没翻。带 dev 最大的那笔代价（移除设备要全量扫描重写每一棵带指针的树）由 D1（数据可移动性 / 反向索引） 已定的反向索引降到 O(该设备的引用)，那条路本工程本来就要走。

**依据**：

- E20（指针变宽的 CPU 缓存代价）：指针变宽的代价只在工作集超出 L3 时出现，16 KiB 节点把它吸收掉——宽指针在本工程成立的直接前提，也是 D8（核心索引结构） 已定项 2 的依据侧。
- 现役实现（2026-08-28 现查，未在本项目验证）：带 dev 的 bcachefs 移除设备要全量扫描重写每一棵带指针的树（`bch2_dev_usrdata_drop()`），靠 `BTREE_ID_backpointers` 才降到 O(该设备的引用)，ZFS 把 dev 焊死在 blkptr 里、移除设备只能加一层永不消失的 `vdev_indirect_mapping`（只能 condense，不能删）；不带 dev 的 btrfs 要为映射表的自举在系统配置里挖一块定长不可扩的 `sys_chunk_array[BTRFS_SYSTEM_CHUNK_ARRAY_SIZE]`（2048 字节，`include/uapi/linux/btrfs_tree.h` 里 `BTRFS_SYSTEM_CHUNK_ARRAY_SIZE` 的定义与系统配置结构里这个字段），且 balance 照样重写全部引用。
- 三方论证（2026-08-28）：正推腿与反推腿结论相反，分辨它们的事实是 D2（RAID 条带策略） 的可变粒度；本地腿整轮作废，腿况与经过在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:48-61`（整段抄，未转述）**

```markdown
#### 已定项 2：密文校验和取 32 位

**定案**：位置条目里的密文校验和取 32 位，正好是完整 CRC32C 的宽度，不涉及截断；实现不许走「更宽的 CRC 截断到 32 位」这条路。

**射程**：结论是条件式的，引用时条件要一起引——它只在这一组输入下成立：10 TB 池、4 KiB 块、每周 scrub、10 年、损坏率 1e-9 每块读、漏检期望上限 1e-6。重开条件可判定：拿到现场损坏率且它超过 3.37e-9，这一格当场重开。候选算法一个都没被淘汰（偏离都在一个数量级以内）。威胁模型里介质损坏不是攻击者，这一格不判抗攻击性。

**依据**：

- E40（密文校验和位宽）：这组输入下最小位宽 31（与原推导那句「n > 30.3」取整一致），32 位的余量 3.37 倍——五个输入（池容量、损坏率、服役年限、scrub 周期、块大小）每一个各自错这么多倍才要 33 位；截断的 CRC 不是短一点的 CRC——CRC32C 全宽对单比特翻转 0 漏检是结构性质，截断之后变成概率检出。
- 旁证（现查，未在本项目验证）：btrfs 的默认校验和就是 32 位 CRC32C（`BTRFS_CSUM_TYPE_CRC32 = 0`），它那 256 位槽位是为可选的 SHA256 / BLAKE2 留的。
- 用户定案，原话与那天的推导在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:62-75`（整段抄，未转述）**

```markdown
#### 已定项 3：定宽，不加密时留位

**定案**：指针按最坏情形定宽——头部里的 MAC（128 位）与 nonce（96 位）恒定占位，不加密的卷把它们留成空位，不用类型标记省掉；这些位在所有卷上都占着，也不挪去装别的字段。

**射程**：可选的是 AAD 那道运算（加密关着就不算），不是格式（`.claude/rules/fs-design.md`「加密是可选的运算，不是可选的格式」）。按介质开放布局分支那个机制管不到这一格：incompat 位管的是读者认不认识这套布局，这一格管的是在同一套布局里一个指针内部怎么编码；先定宽、以后再加一套变长布局不是扩张，是多一套要永远维护的解析器加 checker 加崩溃点重放。不加密时白占 MAC 16 字节 + nonce 12 字节 = 28 字节 / 逻辑 extent，按 D4（校验和位置） 已定项 7 的口径（逻辑 extent 占盘恒 32768 字节）是 `28 / 32768 = 0.0854%`。变长那一侧省的是那 28 字节、不是省一个字段（算法类型字段 D9（加密） 已定项 12 第 4 项本来就要有），要付的是解析必须从头走：定宽指针可以按偏移直接定位第 k 个，变长不行；「恢复期扫一个部分损坏的节点时能不能跳过坏掉的那一条继续读」是推论，未实测。 加密关着时 MAC 16、nonce 12 恒 0，读者遇到非 0 判该指针所在的结构损坏，同 I-2.4（头校验和覆盖范围） 给单元头 29 字节的读法（实审 A3a 落地在两处读者——多层码 2 树内部条目、journal 新根段指针：`crates/singlefs-core/src/pointer.rs` 的 `PointerHeadFieldOutsideTheFirstVersion::MacOrNonceNotZero`）；extent 偏移那 2 字节第一版恒 0（第一版指针指整个单元），读者遇到非 0 判该指针所在的结构损坏，与加密开不开无关；checker 在走读跟随的每条指针上判，core 每个读盘上指针头部的读者都判，落点在 [checks-owed.md](../checks-owed.md) 已还清表 C581（extent 偏移非 0 没有判据） 那一行。

**依据**：

- 无实验：定宽换变长省下来的是空间，省掉的是「一个指针单独捡起来能不能直接按偏移解出来」，两边不是同一个量纲，没有可量的量。撑它的是 `.claude/rules/fs-design.md` 两条——「不为省空间牺牲自包含」，以及格式分支的判据（看它能不能把「用错了」变成「挂不上」，且判据里不许出现「这样更快 / 更省」）。
- 用户定案，原话与 2026-09-12 的重申在变更史。
- 判决 2026-10-01 采：extent 偏移第一版非 0 判该指针所在的结构损坏（`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 11 行）。

**欠**：C210（三个「第二套」属于哪一类扩展没人判过）。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:76-96`（整段抄，未转述）**

```markdown
#### 已定项 4：位置条目 14 字节

<!-- format-const: LOC_ENTRY = 14 stale=11 * 2|tree_entry=61 -->

**定案**：设备 ID 4 字节、物理偏移 6 字节（16 KiB 槽号）、密文校验和 4 字节 ⇒ 位置条目 14 字节。

| 段 | 宽 | 为什么是这个宽 |
|---|---|---|
| 设备 ID | **4** | 上界指不到任何已定条款（D12（目标介质） 不封顶介质套数、全仓没有 dev id 回收条款 ⇒ 窄宽度定的是池一生里出现过的设备身份总数），按 `.claude/rules/fs-design.md`「不为省空间牺牲自包含」取宽的一档；先例是已定项 3 |
| 物理偏移 | **6**，编码为 **16 KiB 槽号** | 索引节点按 16384 对齐、大小就是 16 KiB（D18（块里携带什么信息） 已定项 9）⇒ 按 32 KiB 单元号编码表达不了半数索引节点的落点，按字节偏移则让非 16 KiB 对齐的起点变得可表示。6 字节槽号寻址 4 EiB / 设备，同宽的字节偏移只有 256 TiB |
| 密文校验和 | **4** | 已定项 2 |

**射程**：物理偏移与 D3（空间分配） 已定项 7 是同一个坐标系（D3（空间分配） 已定项 3 依据第 2 条逐字要求同一个坐标系）⇒ 两处一起定，不许各定各的。树表单元指针与根记录合计这两个下游数随它改，而根记录的合计不在这里登记，唯一权威处是 D22（单元原子性怎么合成） 已定项 7 的字段表。相对旧的 11 字节，每个位置条目多 3 字节、每个指针多 6 字节。

**依据**：

- E97（记账与分配记录的条目编码）：判据 7 按可表示性判落点编码——按 16 KiB 槽号编码时全部合法起点都表达得了，按更粗的单元号有一半写不出来。
- 用户定案，原话与「这个数此前没走过决策流程却已经被消费掉」那次复核在变更史。

**欠**：C238（位置条目从 11 变 14 之后「53」这个派生数没人回扫）。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:97-132`（整段抄，未转述）**

```markdown
#### 已定项 5：位置条目降为提示，中央映射是唯一入口

**定案**：取混合——

| 件 | 定案 |
|---|---|
| 位置条目（已定项 4 的 14 字节） | **降为提示**。宽度与段划分不变，变的是它的地位：读到它可以直接去那个落点试，但它可能过期 |
| 中央映射 | **解引用与释放判定的唯一入口**。搬迁只改这里一条条目，「谁引用我」根本不用知道 |
| 态别 | 两者都不是权威态清单的第四项：映射可从单元自描述扫描重建，承重面仍是 D20（承重面：单元的原子性与自包含） 的单元 |

随定案一起生效的三条硬规则：

1. **释放一律经映射，不经提示**；经映射核到那条映射条目之后，释放之前还要按它位置项里带的单元校验和读盘核一次。**核出对不上时隔离那个槽，发布照成**：逻辑上照样释放（映射条目去掉），物理槽不还回空闲池，记进隔离并计数报出。
2. **单元头或提示必须带块 birth**——已满足：D18（块里携带什么信息） 已定项 7 的数据单元类身份段本来就含诞生代号 8 字节。
3. **提示过期的多跳率要有运行时观测点**（走错分支是性能悬崖，而悬崖不报错，`.claude/rules/fs-design.md` 硬要求 4）。

**挂载态怎么读映射**：第一版挂载态打开池时把中央映射整片读进挂载态（`crates/singlefs-core/src/mounted_read.rs` 的 `central_mapping_entries`），之后的解引用查这份内存里的条目、不再发映射读；中央映射长成多层时，挂载态打开池时把整棵映射树读进挂载态，解引用照旧只查内存、不为映射发设备读；写侧每次发布按映射树现在的条目数与这次发布最多新增的条目数做映射树容量准入，不靠别的树容量上的数字巧合。多层之后映射树没有条目数上限，结构上限只有头里层级那 1 字节；在规划那一步、任何落盘之前按这次真要写的条目算出形状，层级写不下就拒（`HeightBeyondTheLevelField`），不另立条目数上界（实十九取，主 agent 定）。「提示过期的一次解引用发 3 次设备读」这个读数就骑在这一条上，这个读数是根兼叶时量的，多层落地时由用例重量（里程碑「覆盖写、释放、回退与复用」收口表第 28 行）。分配记录树挂载时同样整棵读进挂载态；extent 树不整棵读，打开文件时按位置走下去（D8（核心索引结构） 已定项 14，用户 2026-09-24 定）。

**硬规则 1 的读盘核读不出、核出对不上时怎么办**：释放之前按位置项读盘核校验和，读盘本身失败或核出对不上，都先重读一次，两次都读不出或都对不上才按对不上处置（重读对不上那一半主 agent 2026-09-25 定：一次瞬时坏读不许让单元永久隔离；实二五已实现）。两次读结果不同类（先读不出、重读对不上，或反过来）按两次都没对上处置，报重读那一次的样子（实二五取，被攻过零轮）。任一份核出对不上，这个单元在每块盘上的分配记录都留在「已分配」、不改成已释放（另一块盘上那一份对得上也一起留，各盘的账保持对称）——落盘即跨重挂，准入里照已分配算、不另进式子。位置项指向一块不在池里的盘，或两条位置项指同一块盘，当映射条目损坏：在任何写之前拒绝，盘上不变。从盘上重建上一版（`crates/singlefs-core/src/recovery.rs` 的 `rebuild_version`）时读不出的数据单元照抄它的位置项、不读内容，挂载照常，读到那个文件时才报错，没见证时除外（所选那一版系统配置没见证、它的数据单元读不出时，可写挂载照 D23（journal 的角色与格式） 已定项 14「所选根走读失败、系统配置没见证它时退回见证到的那一版」退回见证到的那一版；C755（恢复与两种挂载触发退回的失败不同） 的并集判退，零轮措辞，实现在实现批）。

**射程**：同时收口 D21（权威态与派生态的分界） 那条「共享之下多引用者住哪」的清单那一半与 D26（后台整理与放置回收） 已定项 5 的位置权威三选一，别处一律链到已定项 5；它原本还收口 C34（明文映射层的态别三处打架），而 C34（明文映射层的态别三处打架） 2026-09-07 随 D9（加密） 已定项 5 改取「只做 scrub」而无对象，那一份收口落空、另两处不受影响。映射条目的字段表不在这一格里，归已定项 6。三处已知边角没有编号：映射树的 key 布局要兑现 D8（核心索引结构） 已定项 7 那条骑手（去重窗口要被 `T_dirty` 界住）；提示的刷新政策只有「随重写刷新」一种被建模过；缓存命中这一维零测量，E109（位置权威三臂的运行时代价） 明说不答。自举豁免的三类见已定项 8。硬规则 1 的读盘核读不出、核出对不上时怎么办见定案那一段（C394（释放判定不核映射条目位置项里的单元校验和） 的三问）；「先重读一次」只重读一次，不退避、不多次重试。

**依据**：

- E109（位置权威三臂的运行时代价）：三条臂逐维量出来——搬一个单元的代价在指针权威下随引用树数线性、在中央映射与混合下与它无关；发布多付一笔与叶数无关的定额；读只多一成到四成。绝对值要连取样点一起引：它的主取样点上引用树与映射树恰好同高，那是一个退化点。
- E96（混合架构的一致性）：按提示解落点会误放，硬规则 1 由它逼出来；它的 `hdr_key` 臂引的是 D18（块里携带什么信息） 已定项 3 而不是已定项 7 的字段表，那条「要加字段」的格式代价不存在。
- 用户定案 2026-09-23：释放之前读盘核位置项里的单元校验和，核出对不上时隔离那个槽、发布照成（`research/prompts/m2-placement-falsepositive-r1-main-verification.md` 第三节判 C394（释放判定不核映射条目位置项里的单元校验和） 那一格：三条硬规则都不回答要不要核），原话在变更史。
- 用户定案 2026-09-24：挂载态整片读映射照当天的实现写成条款，多层映射由用户同日另一次定案（四棵树的分裂都在里程碑二做，含挂载态读多层映射）改成整棵读，实十九落地（C483（挂载态读映射与树节点回退都没有条款） ①，并行线二落地时定下、没有条款），原话在变更史。
- 用户定案 2026-09-24：C394（释放判定不核映射条目位置项里的单元校验和） 三问——读盘失败先重读一次再按对不上处置、重建时容下读不出的数据单元、隔离那一份的分配记录留在已分配（判决 `research/prompts/m2-newq-r1-main-verification.md` 第三节 N1（读不出先重读一次）–N3（对不上的留在已分配），原话在变更史）；无实验：候选的读数来自三方攻方腿在副本上的原型（`research/prompts/m2-newq-r1-opus-model/`），没有登记成实验。
- 用户定案 2026-09-24：多层映射挂载时整棵读进挂载态、写侧补映射树容量准入（原话在变更史；三方 `research/prompts/m2-treesplit-r1-main-verification.md` T3）。
- 用户定案 2026-09-25：只一份核出对不上时两块盘一起留、位置项指池外的盘照当天拒绝写成条款（实十六接续报告 `m2-writepath-implementer-report.md` 第六节 Q2、Q3），原话在变更史；无实验：两条都不改盘上格式，只定处置。
- 主 agent 定案 2026-09-25：`records/2026-09-24-里程碑二收尾调度.md` 第三节「瞬时坏读让单元永久隔离」那一行。
- 没见证时除外：根环三方第三轮判决 `research/prompts/m3-close-rootring-r3-main-verification.md` 第四节「没进任何一格的，去向」表里 D19（块指针的结构与宽度预算） 已定项 5 回看那一行（辩方 Z5.2：没见证的 R 上数据单元读不出，可写挂载由照常挂 R 改成退回上一版）；用户定案 2026-10-20（C755（恢复与两种挂载触发退回的失败不同）「三句都补」），原话在变更史。无实验：数在三方腿的副本上量，没立实验号。

**欠**：C248（树表单元的自举豁免没写过）、C253（66 048 那笔定额两份 kb 说法相反）、C133（搬迁的转发信息没有载体）、C279（打包之后扫描重建同时看到旧单元与槽）；C417（位置提示的刷新政策只建模过一种）、C418（位置权威三臂的缓存命中维零测量）——依据那两条实验各自自陈答不了的格；C543（重建上一版只带实例表第 0 片）。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:133-162`（整段抄，未转述）**

```markdown
#### 已定项 6：中央映射 key 取按类复用写序键（收严后）

**定案**：key 定宽、逐段排布，定义与宽度逐字同 E134（映射 key 格式代价·槽基线与两条新臂） 的 `class_reuse_tagged` 臂——

| 类 | key（定宽，逐段） | 宽 |
|---|---|---|
| 码 1 数据单元 | 类标签 1 + 出生树 8 + 出生 txg 8 + 写序 10（实例代号 4 + 事务号 6） | 27 |
| 码 2 / 码 3 | 类标签 1 + 出生树 8 + 出生 txg 8 + 实例代号 4 + 出生序号 4 | 25 |

value 照已定项 4 / 已定项 5：位置条目 × w 副本（w = 2 时 28 字节）。补齐与条目总宽归已定项 10。

**它带着的四条收严**（三轮对抗里按「记一次输 / 只许收严 / 写明收严在哪」加上的，定案之后照样要守）：

- 类标签取被命名对象在 D18（块里携带什么信息） 已定项 11 登记表里的码，D27（小数据打包容器） 槽里的对象恒为码 1；打包与整容器重写时，槽里 key 的各段逐字节取打包前那一版单元头里的值。
- 出生树取类身份段里的出生树（码 3 偏移 43），出生 txg 取诞生代号（码 3 偏移 73；码 1、码 2 同取诞生代号），实例代号取写序的前 4 字节。
- D16（发布语义） 已定项 5 末段那句「一个事务最多写一个单元的用户数据」要按罩全部码 1 单元读（预分配、截断的尾单元、打洞两端、reflink 两端的拷贝都算），最好写进那一句。
- 出生序号只做 key 的一段，不拿来给固定点多版择新——拿来择新就落进 D18（块里携带什么信息） 已定项 1 否掉的「为分辨陈旧副本而设的序号」；D18（块里携带什么信息） 已定项 1 照写序那条先例欠一条射程注。

**射程**：管 key 与 value 各是哪几段、各多宽，不管补齐与条目总宽（已定项 10）、不管装哪几类（已定项 8）、不管出生序号怎么发（已定项 9）。它落 D15（格式冻结政策） 第 2 层。码 1 那一半的唯一性靠「码 1 的写序已经是唯一的版本判别子」这条推论，而它建在 D16（发布语义） 已定项 5 末段那条事务切分纪律上（配 D23（journal 的角色与格式） 已定项 7 一条记录一个事务号）；那一段 2026-09-13 随 C123（用户定案被改后无人复核） 的复核收口、切分纪律没改，推论成立，剩下的口径打架记在 C310（事务切分纪律与记录数口径打架）。「先」照录：这一格是按已有的代价与性能数取的，新证据推翻它时重开，不是论证出来的唯一解。

**依据**：

- E137（映射 key 形态的性能差距）：三条臂在每次 fsync、后台负载、随机读三类负载上的差距——按类复用写序键在每次 fsync 的多数格与随机读全格领先，在全域随机覆写、删文件、单文件快照删除上输；用户看了这张表取它。⚠️ 模型里物理落点就是写入顺序，这是它在搬迁与快照删除上最有利的情形，老化之后缩多少没量。
- E133（映射 key 各候选的格式代价）：两轮三方论证里出现过的每一种 key 落到格式上各付多少字节，七条候选的指针宽、条目宽与扇出由它给。
- E134（映射 key 格式代价·槽基线与两条新臂）：这一格取的那条臂就是它的 `class_reuse_tagged`，收严加的 1 字节类标签与 D27（小数据打包容器） 槽的两种基线由它按行报出。
- E136（映射 key 岔路的代价行）：交用户定之前四样缺的代价数——码 2 头加 4 字节在 D18（块里携带什么信息） 已定项 14 的三档真实头宽下一格扇出都不掉，映射树层数只在一段池规模区间里差一层。
- 三轮对抗的判决与三轮论证的经过在变更史；用户定案，原话在变更史。

**欠**：C288（映射 key 定案的连带改动没落到别的决策）、C289（码 2 的出生树取哪个字段没写）、C163（固定点各轮写序相同使平局破除落空）、C286（只读后 remount 取不取新实例代号）、C287（切换收养开放 checkpoint 的事务后再崩）、C310（事务切分纪律与记录数口径打架）。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:163-176`（整段抄，未转述）**

```markdown
#### 已定项 7：指针头部带出生 (树, txg)，各 8 字节

**定案**：指针头部在 MAC 16 + nonce 12 + 算法类型 1 + 压缩算法码 1 + 压后长度 2 + extent 偏移 2 之后再带出生树 8 与出生 txg 8，头部合计 **50**；指针照旧定宽（已定项 3）。压缩那 3 字节是压缩算法码 1 字节（登记表住 D9（加密），0 = 不压缩，第一版写 0）加压后长度 2 字节（第一版写 0，够表达 [0, 32768) 的单元内长度）。

**射程**：定的是哪 16 字节住指针头部，不定指针总宽（那是已定项 8 与已定项 11 合出来的 88 / 86）。压缩那 3 字节补的是 D9（加密） 已定项 13 第 2 条那条硬要求的格式载体——它要求压缩与加密的次序在格式上说得清，而在此之前仓里没有任何字段承载「这份密文是压过的、压前多长」。⚠️ 主 agent 推荐的是不留（压缩是可选运算，而「为加密留的位所有卷都留」那条原则只管加密），用户弹窗定案选了留，没有走三方论证，可推翻。从指针拿这两段的消费者有四个：D5（快照 / 空间记账机制） 的 O(1) 判定读出生 txg；deadlist 条目按版本身份记，出生树要从指针取；中央映射 key；AAD 形态要指针有这两个字段的位置，而值要经独立验收（D9（加密） 已定项 16 逐字不许从指针里读回来）。改新池新建文件的字节：是——每一条指针加 3 字节。

**依据**：

- E128（指针带出生 (树, txg) 之后的点查代价）：记账树内部条目加宽之后四轮都更慢，inode 树那一对四轮不稳定、不能引；它量的只是这 16 字节，是这一格代价的下界。
- 一轮三方论证，判决在 `research/prompts/d19-item7-r1-main-verification.md`：反推腿指出已定项 8 当时的末句把「住在指针里还是住在引用条目里」留了口子，两种读法都说得通。
- 用户定案（「16 字节住在指针里」），原话与候选乙、丙的出局经过在变更史。

**欠**：C295（指针里的 key 段算不算 AAD 的查找路径）、C296（journal 重放时凑不齐映射 key）、C297（inode 内部条目里出生树两份没有定处理）、C237（码 1 / 码 2 的指针里没有出生 (树, txg) 的位置）。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:177-201`（整段抄，未转述）**

```markdown
#### 已定项 8：全部指针进映射，引用侧按类携带映射 key 的各段

**定案**：中央映射装码 1、码 2、码 3 三类单元；指向一个单元的指针携带该类映射 key 除类标签外的各段，类标签由查找路径给。

| 被指的类 | 指针携带（已定项 6 的 key 各段） | 指针宽 |
|---|---|---|
| 码 1 | 出生树 8 + 出生 txg 8 + 写序 10 | 数据指针 **88** |
| 码 2 / 码 3 | 出生树 8 + 出生 txg 8 + 实例代号 4 + 出生序号 4 | 节点指针 **86** |

**不进映射的三类（自举豁免）**：映射树自己的节点、树表单元、实例表单元。它们的父指针（映射树的根、根记录里的树表指针与实例表指针）里的位置条目是权威，不是提示，已定项 5 的「唯一入口」与硬规则 1 对这三类不适用；三类都是池级单份、引用者只有一个。这三类的父指针与指向码 2 / 码 3 的节点指针同宽 86：位置条目 14 × 2 是权威，实例代号 4 + 出生序号 4 照填被指单元头里的值、不拿去查映射；解析器按被指单元的类定指针宽，不按对面是不是豁免类区分。豁免三类之外的单元（码 1 数据单元与码 2 树节点——extent 树根、inode 树根、inode 叶容器）位置提示读不出时一律经映射回退；豁免三类（映射树根、树表、根记录那条链）不回退。

**射程**：判的是映射装哪几类、指针带 key 的哪几段，不随指针宽度变（其中出生 (树, txg) 那 16 字节住指针头部由已定项 7 定）。只装码 1 那一支在两处正确性上输：码 2 / 码 3 的块同样进 deadlist / livelist，而 D5（快照 / 空间记账机制） 定 deadlist 条目按版本身份记、经映射释放，不进映射就只能按落点记，整理或掉盘重建把它挪走之后会按旧落点释放；搬一个被只读快照引用的码 2 / 码 3 要改写快照树的祖先，取新 birth 违反引用区间、保留旧 birth 违反 `birth` 的定义句。全部指针进映射在这两处都不输：deadlist / livelist 按映射 key 记，搬迁只改映射一条条目，被搬单元逐字节不变、birth 不变。代价：每次 fsync 与后台每单元的映射块都比只装码 1 多一到两成以上，随机读未命中也涨；这一列还少算两块（派生树的码 2 在映射里的删插、extent 节点自己提示过期走映射的读）。父节点重写时位置提示写映射位置（C417（位置提示的刷新政策只建模过一种） 第一版取「随重写刷新」）。checker 与 GPU 判器把豁免三类之外的码 1 / 2 / 3 位置条目只当提示：走读先按提示读，两份都对不上再按映射位置读，引用集合记映射位置。

**依据**：

- E137（映射 key 形态的性能差距）：两种配置（只装码 1 / 全部指针进映射）逐格量出代价差，这一格的代价行整段取自它。
- E134（映射 key 格式代价·槽基线与两条新臂）：两种配置下的指针宽、条目宽与扇出由它给，88 / 86 是它那两行加上已定项 7 的压缩 3 字节。
- E128（指针带出生 (树, txg) 之后的点查代价）：定案那天漏录、次日补上的那一笔点查代价，记在这一格的代价行里。
- 两轮三方论证，判决在 `research/prompts/d19-item8-r1-main-verification.md` 与 `research/prompts/d19-item8-r2-main-verification.md`（第一轮两条云端腿结论相反，用户要求重推，第二轮立场对调）。
- 用户定案，原话与两条补法为什么都要改一批已定条款在变更史。
- 2026-09-23 判决 `m2-lines234-code-r1-main-verification.md` 里判树节点要不要经映射回退那一格——攻方腿实测同一种损伤下数据单元多跳一次读回正确字节，而 extent 树根让 `open_pool_for_read` 整池挂不上。
- 判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3：写路径、重建与 checker 照「映射是唯一入口」重构（取乙），父节点重写时写映射位置，checker 与 GPU 判器把位置条目只当提示。第七节用户定案「如果不合适，逻辑不通顺或者不自然，就重构」，原话在变更史。

**欠**：C293（全部指针进映射之后 E137 少算的两块）、C292（码 2 身份段零消费者的理由已不成立）、C248（树表单元的自举豁免没写过）（并入自 C280（中央映射装哪几类单元没有条款））。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:202-214`（整段抄，未转述）**

```markdown
#### 已定项 9：出生序号怎么发，按什么次序发

**定案**：出生序号从 0 起；同一棵树内每写出一个码 2 或码 3 单元加 1（两类共用一个计数）；作用域换到下一个 checkpoint 时清零；同一个 checkpoint 里同一个节点被固定点重写第二次换新号。发号次序与提交内生块从开放聚簇段 bump 的次序是同一个次序——按树 ID 升序，树内先叶后根、同层按 key 升序，中央映射树倒数第二，树表单元最末。

**射程**：次序本身的唯一登记位是 D3（空间分配） 已定项 10，这里不存第二份。出生序号是中央映射 key 的一段（已定项 6），key 必须在单元写出去之前就定得下来，所以发号绑在落点的分配次序上：实现里只有一个游标、checker 只核一份次序。它只做 key 的一段，不拿来给固定点多版择新（已定项 6 的第四条收严）。用户弹窗定案，没有走三方论证，可推翻；主 agent 推荐的也是这一档。改新池新建文件的字节：是——新池新建文件写出的十一个提交内生块（默认几何；个数随几何变，环 1024 MiB 时 13 个）的落点与出生序号都随它定，逐块的值见 D3（空间分配） 已定项 10 ⑤ 的表。

**依据**：

- 无实验：次序与初值本身没有可量的量，发号与落点共用一条规则这件事的代价在 D3（空间分配） 已定项 10 那一侧；撑这一格的是「key 要在单元写出去之前定得下来」这条推论加已定项 6 收严论证里那两句（同一容器在两个 checkpoint 各拿到出生序号 0、多买到的只是运行期映射不撞）只在重写换号下成立。
- 用户定案，原话在变更史。

**欠**：C291（出生序号的分配规则只在实验源码里）。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:215-227`（整段抄，未转述）**

```markdown
#### 已定项 10：映射 key 一律 27，条目一律 55

**定案**：中央映射的 key **一律 27 字节**——码 1 的 (类标签 1, 出生树 8, 出生 txg 8, 写序 10) 天然就是 27；码 2 / 码 3 的 (类标签 1, 出生树 8, 出生 txg 8, 实例代号 4, 出生序号 4) 是 25 字节，**末尾补零到 27**。中央映射条目因此**一律 55 字节**（key 27 + value 28，value = 位置条目 14 × 2）。映射树节点头的 key 区间同样按 27 存。

**射程**：连带两处随它回到定宽——D5（快照 / 空间记账机制） 已定项 6 的稀疏旁表条目定宽 31（映射 key 27 + 计数 4），D6（快照实现模型） 已定项 2 的 livelist 条目定宽 36（头树 ID 8 + 映射 key 27 + 事件类型 1）。按类变长会把 D8（核心索引结构） 已定项 11 的两条通用规则一起破掉：「定宽条目连续排列」与「声明长度 = 条目数 × 条目宽」——码 2 的条目宽字段只有一个值，两宽就得给它一个哨兵，而哨兵一立，节点内的定宽二分就没了，扫描期解一个映射节点要从头顺序走。代价是每条码 2 / 码 3 条目多 2 字节，新池新建文件多 10 字节。用户弹窗定案，没有走三方论证，可推翻；主 agent 推荐的也是这一档。改新池新建文件的字节：是。

**依据**：

- E145（码 2 自描述头与映射树 key 宽的代价）：它量过的是一宽那条臂；「两宽 55 / 53、按 key 首字节自定界」是那一轮没有量过的第四种形态，拿它当定案等于引一个没有产物的数。
- 用户定案，原话在变更史。

**欠**：无（C321（码 2 头的条目宽是单值而映射树条目按类两宽） 2026-09-14 随用户定案映射 key 回到定宽 27 还清，见欠账表已还清表）。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:228-255`（整段抄，未转述）**

```markdown
#### 已定项 11：指针的偏移表、位置条目的顺序、映射树的根住哪

**定案**：指针从偏移 0 起逐段排布——

| 段 | 偏移 | 宽 |
|---|---|---|
| MAC | 0 | 16 |
| nonce | 16 | 12 |
| 算法类型 | 28 | 1 |
| 压缩算法码 | 29 | 1 |
| 压后长度 | 30 | 2 |
| extent 距起点偏移 | 32 | 2 |
| 出生树 | 34 | 8 |
| 出生 txg | 42 | 8 |
| 位置条目 × 2，按设备身份升序 | 50 | 14 × 2 |
| key 尾段：码 1 写序 10；码 2 / 码 3 实例代号 4 + 出生序号 4 | 78 | 10 / 8 |

⇒ 指向码 1 的指针 **88**，指向码 2 / 码 3 的指针 **86**。位置条目按设备身份升序排列，立成 I-2.5（位置条目按设备身份升序）。**中央映射树的根住根记录**：根记录带一条 86 字节的中央映射树根指针，不进树表；根记录合计 **457** 字节。树表第 1 版 **12** 条。

**射程**：定的只是次序——段的集合与宽度是已定项 3 / 4 / 7 / 8 定过的。压缩那 3 字节紧跟算法类型，与它同属「这份密文是怎么产出来的」那一组，排在 extent 偏移之前是为了让加密与压缩四个字段连成一段。根记录合计的唯一权威处是 D22（单元原子性怎么合成） 已定项 7 的字段表，树表第 1 版那一条随 D5（快照 / 空间记账机制） 已定项 11 加 deadlist 而来。用户弹窗定案，没有走三方论证，可推翻。改新池新建文件的字节：是——每一条指针的段偏移与宽度，以及根记录的合计。

**依据**：

- 无实验：偏移表本身没有第二种选法，段的集合与宽度都已经定过，这一格只排次序，没有可量的量。位置条目按设备身份升序是为了让两条位置条目有一个确定的次序，好立成不变量。
- 用户定案，原话在变更史。

**欠**：无（C304（实例表链指针记录装不下 83 宽的指针） 2026-10-11 已还清，见欠账表已还清表）。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:256-268`（整段抄，未转述）**

```markdown
#### 已定项 12：码 2 / 码 3 的映射条目 day-1 写出，固定点一个 checkpoint 只写一次

**定案**：维持 day-1 写出（新池新建文件 6 条）。固定点写死：每个码 2 节点与码 3 容器，一个实例在一个 checkpoint 里只写出一次——发布末尾一次性 COW；第一版禁边插边 COW（D3（空间分配） 已定项 5 允许的那条实现第一版不取）。

**射程**：⚠️ 以后放开边插边 COW，D8（核心索引结构） 已定项 13 照旧要守：同一次发布里同一个 key 只按最终态落叶一次。会红的检查按单元计，不按固定点轮数计。这一格定的是写不写与写几次，不定 key 形态（已定项 6）与条目宽（已定项 10）。定案时仓里还没有任何释放与重用实验，另一条路留成「等有释放的实验之后再看」，而那句触发条件没有写进这一格。

**依据**：

- E81（提交的固定点）：「已脏节点本次发布只 COW 一次」这条读法的出处，也是固定点在这条读法下收敛的依据；照它罩码 2 与码 3，同一 checkpoint 里两版同 key 那一输才不再出现。
- 用户定案，原话在变更史。

**欠**：C352（day-1 写映射条目的定案，重开条件没人叫醒）、C163（固定点各轮写序相同使平局破除落空）。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:269-284`（整段抄，未转述）**

```markdown
#### 已定项 13：块指针分两段

**定案**：**块指针分两段：一份「头部」跟着逻辑 extent，N 个「位置条目」跟着物理落点。**
逻辑属性（MAC、nonce、算法类型、extent 距起点偏移）在头部，每个逻辑 extent 一份；
物理属性（落点、以及供无密钥侧检损坏的密文校验和）在位置条目，每个物理位置一份。

**射程**：分段把「指针总宽度」从一个约束降级为一个预算，D4（校验和位置） 那一格因此改写成分两段的形态。它只定哪一类属性住哪一段，不定各段有哪些字段（已定项 4 / 7）与次序（已定项 11）；头部后来又加了出生树与出生 txg（已定项 7），加的仍是逻辑属性。「挪出指针」不等于多一次 I/O：D4（校验和位置） 要求的是父节点存子节点校验和，它约束的是住在哪个节点，不是住在哪个结构体。

**依据**：

- D2（RAID 条带策略） 已定「多副本是同一份密文写 N 遍」⇒ 所有副本必须共用同一个 nonce；逐指针各存一份等于把同一个数抄 N 遍，且某副本那份被写错时同一份密文会得到两个互不相容的解密结果。可判定形式是 I-6.7（副本共用 nonce）。
- 现役实现给了同一个形状（2026-08-28 现查逐字核实，未在本项目验证）：bcachefs 的 `struct bch_extent_ptr` 恰好 64 位（`offset:44, dev:8, gen:8` 加四个标志位），校验和 / nonce / 压缩信息全部挪到同一个 bkey 里的独立变长条目（`bch_extent_crc32/64/128`），一条 crc 条目对其后所有 ptr 条目连续生效；nonce 本身不存，`fs/bcachefs/checksum.h` 的 `extent_nonce()` 由 `bversion`（96 位，本来就在 bkey 头部）加 per-crc 的一个小偏移（`crc64.nonce:10` / `crc128.nonce:13`）现算。
- 无实验：分两段是格式结构的取舍，撑它的是一条已定规则（副本共用 nonce）与现役实现的形状，没有可量的量。

**欠**：无。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:285-297`（整段抄，未转述）**

```markdown
#### 已定项 14：位置条目逐个列出，不用几何公式从单一起点推算

**定案**：N 个位置条目逐个列出，各带设备身份；不从单一起点加一个尺寸推算出其余各列的位置。

**射程**：管位置条目怎么表达，不管它带哪几段（已定项 4）。前置口径是 D2（RAID 条带策略） 已定项 1 把「宽度可变」的粒度定成**逐次可变**——格式侧物理位置逐个列出、各带设备身份、不引 chunk 映射表，正是这条推导要的那个前提。它与 D2（RAID 条带策略） 已定项 1 同型：买到的是「不指错盘」，不是「失败域独立」。

**依据**：

- ZFS 能用「一个 DVA + asize」推出 RAID-Z 全部列位置（`BP_GET_ASIZE` 把 3 个 DVA 的 asize 相加），前提是该 top-level vdev 的子设备数**在池创建时固定**（现查，未在本项目验证）；D2（RAID 条带策略） 已定项 1 否定了这个前提。
- 无实验：能不能推算由条带宽度的粒度直接决定，是一条推论，没有可量的量。

**欠**：无。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:298-310`（整段抄，未转述）**

```markdown
#### 已定项 15：extent 距起点偏移的值域与第一版取值

**定案**：extent 距起点偏移的值域由**最大 extent 尺寸**封顶，不由文件尺寸封顶；本工程的值域是 [0, 32768)，**15 位够**。第一版该字段恒 0——切分是预留特性、第一版关闭；位仍按已定项 3 的「定宽留位」留着，理由是政策不是用量。

**射程**：说的是值域，宽度取 2 字节在已定项 11 的偏移表里。D9（加密） 已定项 12 第 2 项按 32–64 位估这个字段，那是按文件尺寸估的，估高了。本工程值域更窄的依据是 D4（校验和位置） 已定项 7：extent 占盘恒 = 单元 = 32768。

**依据**：

- 现役实测（2026-08-28 现查 `fs/bcachefs/extents_format.h`，未在本项目验证）：`crc32.offset:7` / `crc64.offset:9` / `crc128.offset:13`，配套 `CRC128_SIZE_MAX (1U << 13)` ⇒ 这个字段的值域由最大 extent 尺寸封顶。
- 无实验：值域是两条已定条款相除得出的算术，没有可量的量；切分第一版关不关由 D4（校验和位置） 已定项 7 定。

**欠**：无。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:311-321`（整段抄，未转述）**

```markdown
### 与其他决策的连锁

| 决策 | 连锁点 |
|---|---|
| D4（校验和位置） | 已定项 13 把「指针总宽度」从一个约束降级为一个预算，D4（校验和位置） 那一格因此改写成分两段的形态；逻辑 extent 占盘恒 32768 由 D4（校验和位置） 已定项 7 给，已定项 3 与已定项 15 都踩在它上面 |
| D2（RAID 条带策略） | 已定项 1 与已定项 14 的前置是条带宽度可变的口径，由 D2（RAID 条带策略） 已定项 1 定为逐次可变 |
| D9（加密） | nonce 载体由已定项 13 定了一半（属头部、每逻辑 extent 一份）；算法类型与压缩那几段的登记表住 D9（加密），AAD 的期望值不许从指针里读回来（D9（加密） 已定项 16） |
| D12（目标介质） | D12（目标介质） 已定项 4 的挂载比对表里，设备身份错配要拒绝挂载，依据是已定项 1 的位置条目带 dev。⚠️ 那条判据说的是「用到它的**地址算术**是否携带设备身份」，btrfs 的地址算术里有设备身份、只是住在 chunk 树；判据成立，倒下的是「携带 ⇒ 必须在指针里」这个跳步 |
| D15（格式冻结政策） | 指针字段布局是 D15（格式冻结政策） 已定项 7 的第 2 层；已定项 6 的中央映射条目字段表也落在第 2 层（D15（格式冻结政策） 已定项 1 的依据逐字点名） |
| D8（核心索引结构） | 指针宽进内部条目与树表条目，D8（核心索引结构） 已定项 6 / 8 的几个宽度随已定项 7 / 8 改；宽指针成立与否与 D8（核心索引结构） 已定项 2 的 16 KiB 节点耦合 |

```

**出处 `.claude/kb/decisions/23-journal-role-format.en.md:1-15`（整段抄，未转述）**

```markdown
## D23 (Journal Role & Format) — Settled

[中文](23-journal的角色与格式.md) · **English** · [日本語](23-ジャーナルの役割と形式.ja.md)

<!-- doc-lint:not-numbers CRC-64 SSE4.2 CRC32C SHA-256 CJ2 -->

D23 (Journal Role & Format) governs what the journal is, what it writes, and how it is used after crashes: what records carry and the precedence between allocation and logging (Settled Item 20), what is written on each fsync (Settled Item 1), ring geometry, record size, and in-flight record count upper bounds (Settled Items 2 / 12 / 18 / 19), tail (Settled Items 3 / 18), record header and named entry bytes (Settled Items 4 / 7 / 8 / 9 / 10 / 11 / 13 / 15 / 17 / 21), prefix judgment and replay lower bounds (Settled Items 14 / 22), administrator rollback, failure handling, and instance switchover (Settled Items 14 / 16), three journal-related deadlocks and the reserve pool (Settled Items 23 / 24 / 25), and parsing disciplines for record headers (Settled Item 26). Out of scope: publish persistence ordering, checkpoint triggering, and warm-up belong to D16 (Publish Semantics); instance table row format and row-writing rules belong to D18 (information carried in a block) Settled Item 11; root records and the root ring belong to D22 (Synthesizing Unit Atomicity); how reserve pools and switchover reservations enter the admission inequality belongs to D28 (Mount-Time Commitments); definitions of checker and record checker belong to D13 (verification route) Settled Items 6 / 7; target workloads belong to D25 (Target Workload Priority).

<!-- doc-lint:registry name-col=2 -->

| Identifier | Short Name |
|---|---|
| T6 | Whether refusal is maintained depends on pool history |
| L8 | Parallel Line 3 Cross-Record Crash-Point Replay Stream |

```

**出处 `.claude/kb/decisions/23-journal-role-format.en.md:16-691`（整段抄，未转述）**

```markdown
### Settled Items

| # | Item | Ruling |
|---|---|---|
| 1 | **Role of the journal itself** | Adopt Option A: each fsync writes dirty leaves + all ancestors + root slot + one record, ancestors not deferred; axis 1 and axis 2 are not orthogonal **Status: Settled.** **Progress: Implemented.** |
| 2 | **Whether journal is fixed-length ring or chained** | Adopt fixed-length ring; decisive rationale is that chaining converts deadlock 3 criterion from static to dynamic **Status: Settled.** **Progress: Implemented.** |
| 3 | **Where tail resides, who is authoritative, and when it is persisted** | Resides in system configuration slots, rotating across ≥2 slots, written once with each publish; recovery must not trust tail first, replay must be idempotent, recovery cannot salvage geometry **Status: Settled.** **Progress: Implemented.** |
| 4 | **Whether record header can be bounded within a single atomic unit** | Yes; header 487 bytes = ten fields 78 + four increments 21 (transaction boundary 9, sequence number within this publish 4, reverse chain 4, payload checksum 4) + new root segment 364 + fsid 8 + MAC 16 **Status: Settled.** **Progress: Implemented.** |
| 5 | **What O2 (Independent Parser + Checker) actually is** | Unary predicate; second rationale rejecting logical intent logging rewritten as closure to first rationale; subject of reverse criterion is record checker **Status: Settled.** **Progress: No code object.** |
| 6 | **Target workload priority** | Moved out of journal track, see D25 (Target Workload Priority); does not block any item in journal **Status: Settled.** **Progress: No code object.** |
| 7 | **Whether a transaction produces exactly one journal record** | Can span multiple records, record header carries transaction number and commit marker; replay discards tails where commit marker did not appear **Status: Settled.** **Progress: Implemented.** |
| 8 | **Field in record distinguishing timelines** | Record header carries a reverse chain (hash of previous record), width 32 bits **Status: Settled.** **Progress: Implemented.** |
| 9 | **Bit width partition between sequence number and instance number** | jsn = instance number 32 bits + counter 48 bits, two fields totaling 10 bytes; instance number forbidden from rolling back **Status: Settled.** **Progress: Implemented.** |
| 10 | **Which fields `previous_hash` covers** | Covers the entire record header of the previous record, which contains jsn and thus instance number **Status: Settled.** **Progress: Unimplemented.** |
| 11 | **Hash algorithm for reverse chain** | Takes full 32-bit CRC32C, identical function to ciphertext checksum **Status: Settled.** **Progress: Implemented.** |
| 12 | **Record size** | 4 KiB fixed-length, written into system configuration **Status: Settled.** **Progress: Implemented.** |
| 13 | **What ensures payload integrity** | `header_csum` covers entire record [0, 4096); `payload_csum` retained, covering named entries array; failing either renders record invalid **Status: Settled.** **Progress: Implemented.** |
| 14 | **Replay lower bound, administrator rollback, failure handling, and instance switchover** | Apply only records where (instance number, checkpoint_txg) strictly exceeds selected root; five prefix rules; switchover performed as recovery, administrator rollback is a forward publish while mounted **Status: Settled.** **Progress: Unimplemented.** |
| 15 | **Crash after record persisted but before root slot persisted: what recovery applies** | Reconstruct root of that publish from record: record header carries 364-byte new root segment, applying one record = replacing seven fields of selected root; every record of a publish carries identical copy **Status: Settled.** **Progress: Implemented.** |
| 16 | **Instance numbers for mkfs and first writable mount** | mkfs writes 0 (not a valid instance), first writable mount takes 1, writes into every system configuration and passes a completed barrier before touching units **Status: Settled.** **Progress: Implemented.** |
| 17 | **Field table for 56-byte named entry** | One entry per unit: position entry 14 × 2 + type 1 + birth tree 8 + birth txg 8 + key tail segment 10 + flags 1 **Status: Settled.** **Progress: Implemented.** |
| 18 | **In-flight record count upper bound and tail encoding** | Upper bound = ring slot count ÷ F, meaning prefix of one recovery replay contains at most N records; bad checksums uniformly break chain and stop; tail stores counter **Status: Settled.** **Progress: Implemented.** |
| 19 | **Empty publish record, `previous_hash`, ring length** | Transaction number 0 reserved for empty publish; chain calculated from logical previous record within this instance; ring length is mkfs parameter, default 768 MiB, ≤ capacity ÷ 4 **Status: Settled.** **Progress: Implemented.** |
| 20 | **What records carry, precedence between allocation and logging** | Records carry target state of pointer layer, not logical intent or full physical redo; allocation precedes journal **Status: Settled.** **Progress: Implemented.** |
| 21 | **Two counter fields in record header** | jsn globally monotonic per record, prefix judgment relies on it; checkpoint_txg one per checkpoint; both cannot be merged **Status: Settled.** **Progress: Implemented.** |
| 22 | **Prefix judgment** | `jsn == expected`, stops on mismatch; accompanied by an explicit in-flight record count upper bound **Status: Settled.** **Progress: Implemented.** |
| 23 | **How each of the three deadlocks is broken** | Deadlock 1 broken by Reading Option A, deadlock 2 by reserve pool, deadlock 3 by geometric invariant: worst-case occupancy ≤ ring size / F, F ≥ 2 **Status: Settled.** **Progress: Implemented.** |
| 24 | **Deadlock 2 and spurious ENOSPC are distinct failure modes** | Lockup addressed by reserve pool, reserve ≥ ckpt_cost suffices; spurious ENOSPC cured solely by admission, reserve pool exacerbates it **Status: Settled.** **Progress: Implemented.** |
| 25 | **Which interpretation is taken for D16 (Publish Semantics) "not bypassing journal"** | Take Reading Option A: semantic effects must pass through journal; issuing root is not a journal append, but overwriting a slot in root ring **Status: Settled.** **Progress: Implemented.** |
| 26 | **Two parsing disciplines for record header** | Checksum coverage range is explicit on-disk constant, not deduced from sizeof; self-describing length undergoes boundary check before checksum computation **Status: Settled.** **Progress: Implemented.** |

#### Settled Item 1: Role of Journal Itself

**Ruling**: **Adopt Option A: each fsync writes dirty leaves + all ancestors + root slot + one record. Ancestors are not deferred.** Dirty leaves comprise all dirty leaves across the entire system, not merely those of the fsynced file: fsync = one full run publish.

- **Two axes are not orthogonal**: Once axis 1 (whether each fsync issues a root) adopts "issue root", axis 2 (whether ancestor persistence is deferred) retains only one valid value. **Deferring ancestors ⇒ new root cannot be self-consistent ⇒ impossible to issue a root reflecting this fsync.** Under copy-on-write, for leaves of this fsync to be reachable from root, the entire spine must already reside on disk; "self-consistency" is a requirement of [invariants/02-根环.md](../../../crates/singlefs-checker/invariants/02-根环.md) I-7 (Root Ring) class (recursive verification from root with highest generation traverses authoritative state). The 2×2 space has only three realizable cells:

  | | Do Not Defer Ancestors | Defer Ancestors |
  |---|---|---|
  | **Issue root on every fsync** | **Option A** | **Non-existent** |
  | Issue root only at checkpoint | `wal_full` | Option B / Option C |

  ⇒ **Axis 1 settled decision "issue root on every fsync" independently eliminates both Option B and Option C**, and the four rationales of axis 1 (ring cannot be truncated / adds mount-time replay path / journal enters verification chain / zero barriers saved) **hold verbatim against Option B**, requiring no re-argument. Option C (the form in E22 (Recording Named Ancestor Placement)) likewise omits ancestor contents, becoming meaningful only if axis 1 is reopened.
- **Under Option A redo is empty**, not "recovery has no action under Option A": the recovery period retains the standard path of "resuming uncompleted intents" ([checks-owed.md](../checks-owed.md) C1).
- **The record header type field is mandatory regardless of whether axis 2 is ever reopened.** Registry for type field (registry location D22 (Synthesizing Unit Atomicity) Settled Item 17): 0 invalid (judged corruption), 1 normal record; new types only append codes without reuse, unknown types refuse mount per clause A. Framing it as an "option reserved for axis 2" creates the false impression that paying for it purchased axis 2 reversibility — **it bought zero reversibility for that.**
- **Truly ensuring axis 2 reversibility requires four items, none of which can be omitted**:

  | # | What is Reserved | Why |
  |---|---|---|
  | A | Record header **type field** + **unknown type ⇒ refuse mount**, policy carried by **system configuration incompat bit** | Having only a field while policy is "skip unknown records" results in failure mode of **silently omitting a publish instruction**, which is worse than having no field |
  | B | Record **self-describing length**, with **boundary check before checksum computation** | Type field only states "what this is", not "how long it is". Option C record length scales with tree height, exceeding budgets pinned to Option A |
  | C | **Journal geometry enters system configuration** (ring size, maximum record bytes, in-flight record count upper bound), **forbidden from being code constant** | Switching to axis 2 ⇒ worst-case record size changes ⇒ deadlock 3 geometric invariant must change accordingly. Hardcoding into code forces file system recreation |
  | D | Entry size **determined by record type** | D9 (Encryption) settled MAC full 128 bits without truncation; Option C entries are wider |

  ⚠️ **Items A and B were already owed checks** (B is second rule of Settled Item 26); recording them as "costs paid for axis 2" constitutes double accounting.
- **Reopening criteria for axis 2.** ⚠️ **This is design-time criterion, not runtime switch.** `.claude/rules/fs-design.md` Hard Requirement 5 states "branch variables forbidden from changing mid-operation" — making "defer or not" a runtime branch switching with workload alters crash semantics mid-operation, instantly violating rules. **Axis 2 can only be switched wholesale via incompat bit.**

<!-- doc-lint:registry name-col=2 -->

| Gate | Short Name | What is Judged | Measurable? |
|---|---|---|---|
| G23.0 | 轴一被重开 | Axis 1 is reopened (otherwise subsequent items need not be evaluated) | — |
| G23.1 | 祖先链占比达标 | In single-stream synchronous write (QD=1, single thread) fsync service time, proportion of "writing ancestor chain + root slot" ≥ X%, **X declared upon experiment registration, forbidden from being decided post-hoc** | **No**, E16 (Role of Journal: WAL vs Intent Log) measurement basis explicitly specifies counting block count only, not wall time. Prerequisite: transaction layer + real-machine harness |
| G23.2 | 甲丙差不收敛 | As batch size grows, difference between Option A and Option C does **not** converge | **"Difference" takes ratio of blocks per batch between Option A / Option C** (user decision 2026-09-28; substituting Option B numbers for Option C): E16 (Role of Journal: WAL vs Intent Log) fifth run shows convergence varies with tree height under widths as of 2026-10-04, rather than a bound of "batch ≤ stream count" — on realistic baseline 1 TiB (4 levels, 11 children under root), stream count ≤ 16 first rises then converges (peaking at batch ≈ stream count), stream count ≥ 24 converges starting from batch = 1 (in `research/results/e16-fifth-run-peak-under-today-widths-2026-10-04-h487.out` stream count 64 row: `name=cell` lines 1882, 1904, ratio from batch 1 to batch 64 drops from 3.500000 to 3.169231, its `row_summary` at line 2405); tree height ≤ 3 converges from batch = 1 for all stream counts, tree height ≥ 5 all first rise then converge for all scanned stream counts, threshold at tree height 4 is approximately 2 × children under root (measured only on third-party attacker replica, re-run by main agent yielded byte-identical results, warehouse harness not redone, verdict `research/prompts/d23-e16r5-r1-main-verification.md` section 1 summary table). The "3.40× → 3.71×" dataset was under old width (fanout constant 128, tree height constant 4). New wording underwent zero attack rounds. ⚠️ Option C not yet measured |
| G23.3 | 四项盘上齐备 | Items A–D all prepared on disk | **Yes, should be settled now** |

- **Reverse criterion (must be inherent to D23 (Journal Role & Format))**: If, after record checker (D13 (verification route) Settled Item 7: checks requiring second input belong here) is deployed, cumulative events across N full runs where "record checker catches bug while checker + O1 (In-Memory Differential Model (HashMap Ideal File System)) + crash-point replay all miss" remains **0** (N declared at deployment), then incremental value of "Option B destroys byte-by-byte comparison of record checker" is measured as zero, and axis 2 re-evaluation should proceed. Without this rule, that rationale remains unfalsifiable.

**Scope**: Governs what each fsync writes, and whether axis 2 can be reopened; target workloads belong to D25 (Target Workload Priority). Settlement rests solely on structural rationale "two axes are not orthogonal", independent of gain magnitude; three rationales given at settlement time were downgraded and non-load-bearing: "Option B destroys byte comparison" temporarily non-bearing, "Option B cannot buy 'misuse turns into mount failure'" retracted due to wrong scope, and "Option B payoff profile" leaves only bonus ceiling. Option B gains exist solely in single-stream synchronous writes, which happen to be database and VM image workloads (D10 (True Answer to Random Writes / Fragmentation) records as common soft underbelly of copy-on-write file systems) ⇒ can only state "Option B bonus is bounded", not "Option B gains are negligible". Write amplification premise of axis 1 was challenged in third-party round 1 of milestone "overwrite, release, rollback and reuse" addendum 1; that round 1 challenge was overturned in round 2 (364-byte new root segment in record header already existed in round 2, applying is swapping seven fields, so "wal_full cannot modify itself" held only for form "each fsync writes separate tree table unit"), concluding with three forks collapsing into Option A with prose unchanged (`research/prompts/m2-s1-r1-main-verification.md`, `m2-s1-r2-main-verification.md`).

**Evidence**:

- E16 (Role of Journal: WAL vs Intent Log): Deferring ancestors issues root only at checkpoints, never at fsync; batching failing to erode gains of deferring ancestors varies with tree height (run 5: tree height ≤ 3 converges from batch = 1 for all streams, tree height 4 threshold ≈ 2 × children under root, tree height ≥ 5 all first rise then converge) — this alters strength of downgraded rationale "Option B payoff profile", not the conclusion.
- Third-party: 2026-08-28 third-party argument (`_axis2-background.md`), forward leg gives structural rationale "axis 1 ⇒ axis 2", downgrade of three original rationales and leg status in history.

**Owed**: N for reverse criterion not declared; record checker wired to crash-point replay (`check_records` in `crates/singlefs-checker-tier/src/crash.rs`), event counts unmetered.

#### Settled Item 2: Whether Journal Is Fixed-Length Ring or Chained

**Ruling**: **Adopt fixed-length ring.** ⚠️ Candidate rationale "chaining turns journal write path into allocation path, thereby entering deadlock 2" is overly strong and rejected: chaining increments are bounded and calculable, decoupled by raising reserve pool — it is not a true cycle. What actually eliminates chaining are three points, none passing through deadlock 2:

1. **Reserve pool must increase by an order of magnitude, and raising reserve pool exacerbates spurious ENOSPC.** Under chaining, journal blocks themselves require allocation ⇒ reserve pool must rise from `ckpt_cost` to `ckpt_cost + journal blocks within one checkpoint window`; reserve pool is pure tax on normal allocation, and spurious ENOSPC is cured solely by admission (Settled Item 24) ⇒ **chaining transforms costs not into deadlocks, but into spurious ENOSPC.**
2. **Prefix judgment and space ownership become entangled.** Under fixed-length ring, "where valid prefix reaches" determines solely which records to replay; under chaining, it also decides **which blocks allocator may dispense** — prefix rules and D21 (Boundary Between Authoritative and Derived States) accounting authority are independent judgments that must match block-by-block; mismatch causes silent leaks or double allocations; chain blocks beyond discontinuity truncation point either leak or are re-issued while replay still references them.
3. **Deadlock 3 is not eliminated, but converted from static criterion to dynamic criterion — this is decisive.**

   | | Criterion | When Calculable |
   |---|---|---|
   | Fixed-length ring | Worst-case operation ≤ ring size / F, F ≥ 2 | **Calculated at mkfs, verified at mount** (same as XFS `XFS_MIN_LOG_FACTOR`) |
   | Chained | Worst-case operation ≤ **current free space** | Incalculable — **current free space changes mid-operation** |

   `.claude/rules/fs-design.md` Hard Requirement 5 verbatim: "**Branch only on inputs invariant during operation**... 'current fill ratio' crosses thresholds mid-operation, **constituting dangerous branch variable**". ⇒ **Chaining replaces decidable geometric invariant with runtime variable strictly forbidden in this project.** This is not a trade-off, but collision with settled hard requirement.

**Scope**: Governs journal spatial layout. Coupled with Settled Item 3: under fixed-length ring, wrong tail answer **merely does redundant work** (degradable); under chaining, it causes **leaks or double allocation**, requiring tail to be authoritative state with tail-before-reuse hard ordering and FUA on critical path; **inlined tail + chaining is worst cell** (inlined can only advance hitchhiking on records, chaining during idle periods writes no records ⇒ journal blocks never reclaimed, spinning into space leak). ⚠️ Ring geometric invariant codified as [invariants/03-journal.md](../../../crates/singlefs-checker/invariants/03-journal.md) I-8.1 (Ring Geometry Sufficiently Large), checker implemented (pool-level checker `judge_journal_ring_geometry_fields_agree`, core recovery simultaneously checks three equalities, `crates/singlefs-core/src/recovery.rs`).

**Evidence**:

- E25 (Reserve Pool Size for Ring vs Chain): Checkpoint lockups drop to zero exactly at predicted tier — ring at reserve = ckpt_cost, chain at ckpt_cost plus one window of journal blocks.
- E23 (Journal Geometry): Ring and chain differ by only one item — chain requests blocks from allocator for every journal block, ring 0 times; chain lacks deadlock 3, ring requires geometric invariant.
- Third-party: 2026-08-29 three legs uniformly judged "chaining enters deadlock 2" overly strong (`_d23-background.md`).

**Owed**: None.

#### Settled Item 3: Tail Resides in System Configuration Slot, Modifying JBD2 in Two Places

**Ruling**: **Adopt system configuration form** (authoritative tail replica resides in fixed location, not inlined in record header), **with two modifications**:

| # | Change | Why |
|---|---|---|
| 1 | **Single-replica in-place overwrite forbidden**, changed to ≥2 slot rotation + generation + verify all candidates individually before selecting newest | D22 (Synthesizing Unit Atomicity) established identical discipline for root: a single torn write wipes out the sole witness |
| 2 | **Written once with each publish, being the final step in publish sequence** | Note on D16 (Publish Semantics) settled item 7: system configuration slots updated after root slot persisted, entering crash-point replay enumeration; D16 (Publish Semantics) settled item 6 verbatim "each publish increments `checkpoint_txg` by one — **publish triggered by fsync is also a publish**" ⇒ "once per checkpoint" and "once per publish" share identical frequency, publish triggered by fsync likewise must write system configuration slots |

- In-place overwrite does not conflict with copy-on-write: tail, root slots, and journal record headers belong to self-authenticating units (issuing root itself overwrites a slot in root ring), governed by D22 (Synthesizing Unit Atomicity) Settled Item 21; copy-on-write buys nothing for parentless units. **Semantically tail is derived state**: authoritative determination remains `jsn` continuous prefix + `checkpoint_txg`.
- **Three hard requirements emerge**:
  1. **Recovery must not trust tail before proceeding: must first scan entire ring, verify each record, and select longest legal prefix.** When tail is stale, section `[tail_stale, tail_actual)` was overwritten by newer wrap-around records ⇒ proceeding from stale tail causes **first record to fail `jsn != expected`** ⇒ discontinuity stops immediately ⇒ records after `tail_actual` still required are not replayed. **That leads to data loss, not 'redundant work'.** ⇒ "Stale tail merely causes redundant work" **holds only under full-ring-scan recovery**. Identical discipline to D22 (Synthesizing Unit Atomicity) root slot rule "verify all candidates individually before selecting newest".
  2. **Replay must be idempotent**:

     > **Replay of every journal record must be idempotent** — replaying same record twice yields identical result to replaying once.

     "Stale tail is merely redundant work" rests on this premise: without idempotence, stale tail converts "redundant work" into "repeatedly applying already committed records". Compatible with Settled Item 20 "records carry **target state** of pointer layer": target states are inherently idempotent (writing pointer to value twice yields identical result), whereas logical intents are not ("increment counter" executed twice yields different result).
  3. **Recovery algorithm cannot salvage geometry**: When window exceeds ring, both recovery styles lose data; ring size ≥ window is responsibility of [checks-owed.md](../checks-owed.md) C28, **not the recovery algorithm's**, and neither may substitute for the other.
- When block reuse overlays, "stale tail merely causes redundant work" no longer holds; recovery requires replay lower bound, taking root watermark (Settled Item 14); this narrows rather than overturns Settled Item 3: "scan full ring first, verify record by record, never trust tail first" remains verbatim, changing only which segment of verified legal prefix is applied.

**Scope**: Governs where tail resides, write frequency, and recovery usage. Byte table [layout/01-first-txn.md](../layout/01-first-txn.md) sections 0 and 1 specify "once per publish": two warm-up empty publishes and new-pool file creation each advance tail once. ⚠️ Cost table used when adopting system configuration form (E23 (Journal Geometry) "two arms difference 21 : 500") is superseded: first version makes every publish a checkpoint, steady-state extra writes changing from 21 writes to one slot on each disk per publish; replay dimension is non-existent (recovery skips tail, performing full ring scan on every mount). Rationale for "rotating system configuration slots on every publish, awaiting persistence before returning" rests on Criterion N-Config (Settled Item 14 "unreadable updated state read by writable mount", C554 Option B, C577): reducing rotation count or returning before persistence loses confirmed publishes under "confirmed publish + temporary unreadable root slot + one corrupt named unit", which is byte-for-byte identical to "crash before root slot FUA + one corrupt unit", indistinguishable to readers. This rationale holds because that class of dual failure falls within tolerance scope (user decision 2026-09-28); lagging branches (D18 (information carried in a block) Settled Item 11) reading tail is not this rationale.

**Evidence**:

- E24 (Recovery Algorithm: Does Trusting Tail First Lose Records): Number of records lost by trusting tail first exactly equals records written within window; full ring scan loses none, but replays all records since stale tail (forcing idempotence); window exceeding ring loses data under both arms (recovery cannot salvage geometry).
- E23 (Journal Geometry): JBD2 form and XFS inlined form behave oppositely regarding steady-state extra writes vs replay volume after idle crash — based on obsolete premise of tail advancing once per checkpoint, see Scope.
- Third-party: 2026-08-29 (`_d23-background.md`).
- Third-party verdict `research/prompts/c245-r3-main-verification.md` (C245 round 3): Rationale for per-publish rotation replaced by Criterion N-Config; attacker probe measured Option B and C losing confirmed publishes under "confirmed publish + unreadable root slot + one corrupt named unit", whereas Option A requires four concurrent failures to lose data.
- User settlement 2026-09-28: That class of dual failure falls within fault-tolerance scope (verbatim in history).

**Owed**: C77; C594.

#### Settled Item 4: Record Header Bounded Within Single Atomic Unit, Header 487 Bytes

**Ruling**: **Record header falls completely within single atomic unit** ([invariants/03-journal.md](../../../crates/singlefs-checker/invariants/03-journal.md) I-8.2 (Record Header Does Not Straddle Atomic Units)). Header **487 bytes**:

- **Ten fields 78 bytes**: magic 4 + type 2 + algorithm type 1 + record flags 1 (bit 0 = last record of publish, Settled Item 17; other bits write 0, non-zero read treated as corrupt) + self-describing length 4 + named entries count 4 + `jsn` **10** + `checkpoint_txg` 8 + nonce 12 + header checksum 32. The 10 bytes of `jsn` = instance number 32 bits + counter 48 bits (Settled Item 9). `tail_lsn` not in header: Settled Item 3 verbatim specified "authoritative tail replica resides in fixed location, **not inlined in record header**", and "inlined `tail_lsn` in each record header" is precisely definition of rejected XFS form.
- **Three settled increments 17 bytes, outside the ten fields 78**: Settled Item 7 transaction number + commit marker **9 bytes**, Settled Item 8 reverse chain **4 bytes**, Settled Item 13 payload checksum **4 bytes**.
- **Sequence number within this publish 4 bytes** (unsigned 32-bit, starting from 1: when a publish splits into N records, numbered 1..N sequentially, 1 when single record; empty publish record also writes 1): Settled Item 14 Note 1 relies on it to anchor chain head when selected root record is unreadable. Width of 4 bytes and placement immediately following transaction number and commit marker chosen by main agent per "not sacrificing self-containment to save space", overturnable. Field offsets in header: transaction number 78, commit marker 86, sequence number within publish 87, reverse chain 91, payload checksum 95, new root segment 99, fsid 463, MAC 471, header end 487. N records of publish numbered sequentially 1..N synchronized with jsn; readers encountering following four cases treat record as corrupt, breaking chain immediately and rejecting publish entirely: sequence 0; skipped sequence within publish; anchor readable but first record of next publish sequence not 1 (breaks at this record); subsequent records existing after record carrying last flag within same (instance number, checkpoint_txg) (breaks at flagged record; "subsequent" read by counter without requiring adjacency: readable records with counter greater than flagged record with same (instance number, checkpoint_txg) counted, missing numbers do not stop; premise is journal records within same instance have `checkpoint_txg` non-decreasing with counter, giving identical application result to full table scan). Identical check to I-8.9.
- **Settled Item 15 new root segment 364 bytes** (tree table pointer 86 + mapping root pointer 86 + tree ID watermark 8 + F 8 + instance table pointer 86 + allocation record tree root pointer 86 + unmount mark 4).
- **fsid 8 and MAC 16, placed after new root segment**:
  - **fsid 8** (low 8 bytes of system configuration fsid, identical measurement basis to unit header 8 bytes, D18 (information carried in a block) Settled Item 7). Without it, I-1.4 requires an exemption for journal records, and full ring scan cannot block "records left on disk by previous file system" — mkfs writing full ring zeroes holds only when mkfs completes cleanly, failing mid-mkfs. With 8 bytes, no exemption needed.
  - **MAC 16** (placed after fsid; nonce 12 and algorithm type 1 already present in ten fields). Identical rationale to root record 29 bytes (D22 (Synthesizing Unit Atomicity) Settled Item 7): journal record header is self-authenticating structure without parent pointer carrying MAC ⇒ lacking day-1 allocation forces either plaintext exemption when encryption is enabled (leaking `checkpoint_txg`, two pointers in new root segment, named entry locations) or layout change; D9 (Encryption) Settled Item 3 principle verbatim "slots reserved for encryption across all volumes, never repurposed". I-6 does not list it.
- Registered value is 487 (`JOURNAL_HEADER_BYTES`); ten fields 78 separately registered as `JOURNAL_HEADER_TEN_FIELD_BYTES`. **Stating 78 requires explicitly specifying ten fields and citing this sentence.** 487 occupies 95% of 512 sector, leaving **25 bytes** (cannot hold a 56-byte named entry); on 4096 leaves **3609 bytes** (64 named entries, ⌊(4096 − 487) ÷ 56⌋ = ⌊3609 / 56⌋).
<!-- format-const: JOURNAL_HEADER_BYTES = 487 stale=头 277 字节|的 277 字节头|合计 **277 字节**|JOURNAL_HEADER_BYTES: u64 = 277|JOURNAL_HEADER_BYTES: u64 = 84|JOURNAL_HEADER_BYTES: u64 = 86|JOURNAL_HEADER_BYTES: u64 = 78|512), 428|512), 426|512), 434|4096), 4012|4096), 4010|4096), 4018|512 上 428|512 上 426|头 86 字节|头 **307 字节**|登记值就是 307|⌊3789 / 56⌋|头 **311 字节**|余 **201 字节**|JOURNAL_HEADER_BYTES = 311|512 上余 201 -->
<!-- format-const: JOURNAL_HEADER_TEN_FIELD_BYTES = 78 -->

**Scope**: ⚠️ **Atomic width is not determined by this project**: D20 (Load-Bearing Surface: Unit Atomicity & Self-Containment) settled that self-authenticating units depend on **runtime-probed** `physical_block_size`, forbidding hardcoding 512 into format. ⚠️ **This value is a format constant; modifying it requires three coordinated edits**: kb markup and prose, test source `const` and pinning unit tests, and re-running tests to update artifacts; judged red by gate stage "format constants synced between kb and test source" (`format-constants` check in `.claude/gate.d/code-source-discipline.sh`). Fixing record size at 4 KiB ensures alignment waste is constant 0 (Settled Item 12); variable-length alignment cost tables apply only to variable records.

**Evidence**:

- E23 (Journal Geometry): Ten-field header fits in 512 unit; unaligned records sharing atomic unit cannot judge tearing except first record — header must fall completely in single atomic unit.
- E75 (Record Size and Ring Geometry): All header readings fit in single unit across 512 and 4096 `physical_block_size`; fixed power-of-two size prevents header from straddling units.
- fsid 8 and MAC 16: User modal settlement without third-party argument, overturnable, verbatim in history.
- Sequence number within publish 4 bytes: User decision 2026-09-23 (`research/prompts/m2-presumed-clauses-r1-main-verification.md` K4 Z6), deduced, zero attack rounds, verbatim in history.
- User settlement 2026-09-24: Header 311 landed, last flag placed in former "padding 1" byte, reader sequence rules (verbatim in history).
- User settlement 2026-09-24: Anchor readable but first record sequence not 1, records with same txg after last flag, both treated as corrupt breaking chain (Implementation 20 report `m2-lastflag-implementer-report.md` section 6 items 2, 3), verbatim in history; no experiment: both require accidental CRC-32C collision or modified images.
- Fourth case "subsequent" read by counter, missing numbers do not stop, premise that journal record `checkpoint_txg` in same instance non-decreasing with counter: third-party verdict `research/prompts/m3-formal-r2-main-verification.md` section 2 line 8 and `research/prompts/m3-formal-r3-main-verification.md` section 2 fourth case supplementary sentence, both legs aligned; premise establishing separate invariant verdict `research/prompts/m3-inv-b-r1-main-verification.md` section 2 line 9; user settlement, verbatim in history; no experiment: clarifies reader scope, no new readings.
- New root segment adding instance table pointer, allocation record tree root pointer, and unmount mark (header 311 → 487, named entries 67 → 64): third-party verdict `research/prompts/m3-ring-r1-main-verification.md` section 2 line 2 and section 4 item 1, `research/prompts/m3-ring-r2-main-verification.md` section 2 line 2 (first file, unmount publishes patched produce byte-identical root, checker green), `research/prompts/m3-ring-r3-main-verification.md` section 2 lines 1, 2 (two bits placed in flags broken by dedicated sweep publish; application items not from new root segment only unmount mark, instance table pointer, allocation tree root). User settlement "revert to Option A" (verbatim in history). Unmount mark occupying 4 bytes is algorithm of round 1 forward leg H1-b (86 + 86 + 4). No experiment: patches application bytes, verified on third-party replicas, no experiment number.

**Owed**: None (C94 settled 2026-09-24, see owed checks cleared table).

#### Settled Item 5: O2 (Independent Parser + Checker) Is Unary Predicate, Byte-by-Byte Comparison Belongs to Record Checker

**Ruling**: **O2 (Independent Parser + Checker) is a unary predicate judging solely single images**, not "byte-by-byte comparison" — which is binary relation requiring second input argument; checks requiring second input belong to **record checker**, taking arguments `(pre-crash image, record stream, post-crash image)`, authoritative record in D13 (verification route) Settled Item 7.

- ⇒ D23 (Journal Role & Format) second rationale rejecting logical intent logs ("indeterminate placement degrades verification") is literally empty, **rewritten as "verification transitions from syntactic to semantic", serving not as independent second rationale, but closure to first rationale (replay requires second implementation)**: logical intent records name no disk objects ⇒ verifier must independently implement tree semantics + accounting + allocation legitimacy ⇒ second implementation cost transferred to verification side.
- ⇒ **Conclusion "reject logical intent logging" remains unchanged**, first rationale holds independently (Settled Item 20).
- ⇒ Subject of D23 (Journal Role & Format) reverse criterion is **record checker** (Settled Item 1; writing subject as O2 (Independent Parser + Checker) is literally constant 0 — O2 is checker — unfalsifiable).

**Scope**: Governs solely how D23 (Journal Role & Format) refers to these two verification roles and what they reject; their definitions, input arguments, and codebase reside authoritatively in D13 (verification route) Settled Items 6 / 7.

**Evidence**:

- No experiment: unary predicate cannot become binary relation, this is dispute over type signature without measurable quantities; 2026-08-29 third-party argument (`_o2-background.md`), detailed rationales in D13 (verification route) Settled Item 7.

**Owed**: None.

#### Settled Item 6: Target Workload Priority Moved Out of Journal Track

**Ruling**: Target workload priority moved out of D23 (Journal Role & Format), see D25 (Target Workload Priority). It spans D8 (core index structure) / D10 (True Answer to Random Writes / Fragmentation) / D23 (Journal Role & Format), not a journal-specific issue; retaining it here causes it to be misconstrued as journal detail. **It does not block any item in D23 (Journal Role & Format)**: axis 2 was structurally eliminated by axis 1; target workload affects solely strength of downgraded rationale "Option B payoff profile" in Settled Item 1, not the conclusion.

**Scope**: Pure pointer; settled content resides in D25 (Target Workload Priority).

**Evidence**:

- No experiment: this is relocation because it spans three decisions, not journal problem, with no measurable quantity.

**Owed**: None.

#### Settled Item 7: Transaction Can Span Multiple Records, Record Header Carries Transaction Boundary Fields

**Ruling**: **Can span multiple records, record header carries transaction boundary fields**: transaction number 8 + commit marker 1, totaling 9 bytes (one of the three settled increments of Settled Item 4; **different fields** from reverse chain of Settled Item 8, combining their valuation is forbidden).

- Transaction numbers count per instance, starting from 1, shared by all records of same transaction, separate counter from jsn; unit header write order stores low 48 bits, high 16 bits constant 0, without establishing new field (C113 settlement reuses this field). Transaction number 0 reserved for empty publish records carrying no transaction (Settled Item 19 ①).
- Records appended in transaction number order; if transaction fails after issuing unit writes, instance switchover occurs, appending zero records with higher numbers — this forms basis for instance table row W acting as exact prefix (holds under first version serial commits, D13 (Verification Strategy) Settled Item 2; this discipline ensures it holds once parallel commit is enabled).
- **Recovery algorithm**: After "jsn strictly continuous + checksum + reverse chain", **add an additional step** — **discard all records of transaction whose commit marker has not yet appeared**. ⚠️ **Reverse chain cannot substitute for this**: discarded tail is **continuous** on chain (originally written sequentially), passing all chain checks. **Two gates govern two separate matters**: chain governs "whether on same timeline", commit marker governs "whether this transaction completed writing".
- ⚠️ **Discarding tails is standard operation, not edge case**: vast majority of crash points fall mid-transaction ⇒ recovery step will be traversed by vast majority of crash points, must be covered by crash-point replay alongside main path, forbidden from being treated as exceptional branch.
- Criterion codified as I-8.8 (single image half) and crash-point replay (integral transactions half); single image half evaluated by pool-level checker (`judge_commit_markers_per_transaction` in `walk::check_pool_image`), crash-point replay half evaluated by replay stream L8 (Parallel Line 3 Cross-Record Crash-Point Replay Stream) in `crates/singlefs-checker-tier/tests/crash_enumeration_record_spill_over_stream.rs` (transaction spanning two records, all states evaluate I-8.8), C37 cleared, see owed checks cleared table.

**Scope**: Governs correspondence between transactions and records; record-to-timeline correspondence is Settled Item 8, separate concerns — taking either answer for Settled Item 7 still leaks timeline cell. ⚠️ Not a corollary of D13 (Verification Strategy) Settled Item 2 (serial / parallel) — splits occur under serial execution as well. I-8.2 (Record Header Does Not Straddle Atomic Units) restricts solely **record header**, without prohibiting large records; user data units per transaction governed by D16 (Publish Semantics) Settled Item 5 end split discipline, conflict with record count measurement basis belongs to C310. "One record per transaction" incurs two unmodeled costs: ring wrap-around requires continuous space, and full record must buffer in memory before write starts.

**Evidence**:

- E42 (Number of Records per Transaction): Both one record per transaction and "spans multiple + transaction number + commit marker" satisfy I-4.3 (Commit Atomicity); spanning multiple without boundary fields yields illegal prefixes exactly matching split prefix count, accounting for vast majority of crash points — experiment distinguishes cost distribution rather than right vs wrong.
- E45 (Ring Space Occupancy for Multi-Record Transactions): Measured "spanning multiple inflates ring 12×" created by alignment rule stricter than I-8.2 (Record Header Does Not Straddle Atomic Units); literal multiplier is 1.00×, invalidating retraction of Settled Item 7 evidence based on it (experiment conclusion voided).
- User settlement "spans multiple + boundary fields", rationale "strictly more general", maintained after attack rounds; three duties (format, recovery algorithm, invariant) taken under user measurement basis "algorithm requires enhancement", verbatim in history.

**Owed**: None (C37 cleared 2026-10-11, see owed checks cleared table).

#### Settled Item 8: Record Header Carries 32-Bit Reverse Chain

**Ruling**: **Record header adds a reverse chain (hash of previous record), width 32 bits.** Covered fields governed by Settled Item 10, hash algorithm governed by Settled Item 11 (full 32-bit CRC32C), write path evaluation measurement basis governed by Settled Item 19 ②.

- **Deficiency it remedies**: Three settled items of D23 (Journal Role & Format) (fixed-length ring / break and stop on discontinuity / full ring scan) **hold individually but leak when combined**: fixed-length ring slot determined by `jsn` ⇒ remnant records beyond discontinuity do not disappear, awaiting overwrite; recovery continues writing from "prefix end + 1", **connecting directly ahead of remnant sequence** (Settled Item 22).
- **Dual responsibility for single field, both recorded**: distinguishing timelines; enabling record checker to re-evaluate identical prefix boundary using **traversal completely independent from forward scan** (aligning with `.claude/singlefs-ai-sop/rules/evidence-discipline.md` "two verification paths forbidden from sharing code"). If recorded solely for verification independence, it would be first cut to "save bytes".
- **Once hash covers instance number (Settled Item 10), reverse chain becomes redundant for distinguishing timelines** — instance number route is exact, not probabilistic; chain's remaining role is traversal independence.
- ⚠️ **Zero second line of defense upon single false acceptance**: I-8.3, in-flight record count upper bound, and `checkpoint_txg` gates remain untriggered (two timelines in same checkpoint window share identical field) ⇒ consequence is **installing target state of pointer layer from discarded timeline into active state**, which within I-7.4 protection window is completely legal, silently becoming stale data.

**Scope**: ⚠️ **Denominator of false acceptance is occurrences of incident sequence "hole → recovery → resume writing → crash again", not time**; scenario point (10 crashes/day, 10 years) is unmeasured assumption without crash frequency telemetry. Chain width does not need widening as files grow: chain is tested at single seam per incident; scaling pressure hits sequence counter (Settled Item 9) and ring geometry (I-8.1 (Ring Geometry Sufficiently Large)). ⚠️ Seam count measurement basis rests on unmodeled assumption "one seam per incident"; if single recovery leaves multiple breakpoints, denominator shifts ⇒ **32-bit width and Settled Item 11 CRC32C share single point of failure**, reopening conditions apply simultaneously to both (Settled Item 11 reopening condition 1). ⚠️ Cost side (whether 2 / 4 / 8-byte chain consumes extra units on disk, E49 (Reverse Chain Width: 32 vs 64) scanned six header widths) depends strictly on total header bytes mod 8, **accidental rather than structural**, requiring recomputation whenever header width changes; active header without reverse chain is 483 (487 − 4), outside scanned tiers ⇒ **citing "16 saves nothing", "64-bit is free", "no branching on either side" forbidden before recomputation.** ⚠️ XFS `h_prev_block` written on write side but unverified on recovery side; citation limited to "field syntax", not "production defense".

**Evidence**:

- E32 (Remnants of the Previous Timeline): Fixed-length ring + discontinuity stop + resume writing from prefix end causes remnants to be replayed by subsequent recovery; two natural implementations violate one criterion each in opposite directions — deficiency in specification, not implementation.
- E36 (Slot Mapping Dimension): Decoupling slots from sequence numbers cannot salvage; equal-length new and old records behave identically to sequence-based branch, blocking only when record lengths happen to differ.
- E39 (Does Reverse Chain Block Remnant Records): All cells with active chain show 0 remnants without killing single valid record; false acceptance closely matches 2⁻ⁿ across tiers.
- E51 (Number of Reverse Chain Collision Opportunities): Correct denominator for collision opportunities is seam count (chain values of co-originating records constructively equal), under which 32 bits suffices across all tiers; counting every adjacent comparison overcounts opportunities.
- User settlement: Add reverse chain "byte cost small while purchasing two benefits"; width "if 32 bits suffices then 32", verbatim in history; third-party 2026-08-30 three legs uniformly pointed to 32 bits.

**Owed**: C42; C200.

#### Settled Item 9: jsn = Instance Number 32 Bits + Counter 48 Bits

**Ruling**: **Instance number 32 bits + counter 48 bits, two fields totaling 10 bytes, not packed into single 64-bit word.** Separation purchases additional benefit: **comparisons require no bitfield unpacking**, eliminating a shift error source in recovery path.

- Bottleneck lies in instance number: instance numbers are independent of fsync rates; 10 crashes/day over 10 years = 36,500 occurrences, 16-bit capacity of 65,535 offers only 1.8× headroom ⇒ widened to 32 bits (capacity 4.29×10⁹, inexhaustible); counter 48 bits unwidened.
- Counter continues pool-wide without resetting per instance (Settled Item 14 item 3), 48-bit lifespan calculated across whole volume.
- ⚠️ **Consequent obligation: instance number forbidden from rolling back, guaranteed by enforcement.** Once rolled back, this route degrades to state without instance numbers (all remnants replayed). **Identical form to D9 (Encryption) Settled Item 8 nonce watermark requirement** — requiring watermark "forbidden from residing in bare plaintext system configuration... must land in field covered by MAC".
- **Where instance number resides and who protects it**: Residing in journal records creates circular dependency (recovery must read records to learn instance number, but reading records requires instance number to judge); residing in system configuration falls into rollback prohibition ⇒ **same carrier problem as D9 (Encryption) Settled Item 8 nonce watermark**, both must be settled together. Procedure: take max(system configuration, root ring) + 1, write into every system configuration (Settled Item 16).

**Scope**: ⚠️ **Settled numbers contain two unmeasured assumptions**: ① Counter tier tested solely on local machine **without power loss protection (PLP)**; enterprise disks with PLP achieve fsync rates one to two orders higher, unmeasured locally and remaining an assumption; local test ran ext4 fsync path, acting as **upper bound proxy** for this project. ② Instance number tier "10 crashes/day" is **scenario point, not measurement**. ⇒ **Reopening condition decidable**: if field fsync rate exceeds 10⁵/s, counter half reopens. Prefix rules unchanged post-Settled Item 14: replay resumes after last record covered by selected root; when selected root is from newest instance, new records share same instance and strict continuity; only when selected root regresses to earlier instance does it hit instance boundary, halting per strict prefix, cost noted in D16 (Publish Semantics) Settled Item 7, compensation evaluated in D16 (Publish Semantics) Settled Item 8.

**Evidence**:

- E44 (Host Testing and Cost of Sequence Number Bit Width): Under local fsync rate, 48-bit counter far exceeds needs; premise of "48 bits insufficient" fails locally; widening `jsn` to 10 / 12 bytes preserves named entry capacity without consuming extra units across 512 and 4096 units.
- E37 (Log Instance Identifier): Instance number arm satisfies both criteria, provided number never rolls back — rolling back single cell degrades to full remnant replay.
- User settlement, verbatim in history.

**Owed**: C199; carrier and anti-rollback protection for instance number lack identifier, coupled with D9 (Encryption) Settled Item 8 nonce watermark carrier issue.

#### Settled Item 10: previous_hash Covers Entire Record Header of Previous Record

**Ruling**: **Covers entire record header of previous record, including `jsn`.** `jsn` comprises "instance number 32 bits + counter 48 bits" (Settled Item 9) ⇒ **instance number included in hash input** ⇒ two timelines produce constructively distinct chain values, restoring false acceptance rate to 2⁻ⁿ curve, validating 32-bit sufficiency.

- ⚠️ **Rationale for "entire record header" over "jsn only"**: Extra covered fields **consume zero disk bytes** (hash input length not stored on disk), while omitting fields creates vulnerabilities where modified fields still match chain.
- ⚠️ **Implementation discipline**: `header_csum` 32 bytes **cannot enter hash input** (being part of header depending on header) — fixed to participate as zero (Settled Item 19 ②).
- **Chain value requires input guaranteed distinct across timelines**, otherwise false acceptance rate is **100%, independent of bit width**: If `previous_hash` took previous `header_csum`, competing records' magic / type / `jsn` (both equal expected) / `checkpoint_txg` (identical within window) become **constructively identical** when named count matches and encryption is disabled. Distinct question from Settled Item 8 width: width answers collision probability, Settled Item 10 answers whether anything distinct exists to collide.

**Scope**: Governs hash input; function is Settled Item 11, width is Settled Item 8, first record of instance constant 0 without reading disk is Settled Item 19 ②.

**Evidence**:

- E37 (Log Instance Identifier): Recovery with instance numbers yields 0 on remnant and new-write criteria, exact rather than probabilistic — including instance number in chain input gives chain field guaranteed distinct across timelines.
- User settlement, verbatim in history; 2026-08-30 third-party backward leg reading experiment code identified lack of timeline-distinguishing field in chain input, history in changes.

**Owed**: C42.

#### Settled Item 11: Reverse Chain Hash Takes Full 32-Bit CRC32C

**Ruling**: **Takes full 32-bit CRC32C**, identical function to ciphertext checksum settled in D19 (Block Pointer Structure & Width Budget) Settled Item 2.

- **Primary rationale is "choice is non-load-bearing", not "CRC32C is superior"**: 32 bits judged sufficient with **three orders of magnitude margin** ⇒ along 2⁻³² curve, both candidates (full CRC32C, 32-bit truncated cryptographic hash) **suffice**, choice does not alter risk tier; losing full 1 bit (doubling collisions) leaves sufficient headroom.
- **Beyond sufficiency, CRC32C chosen on two secondary rationales**:

  | Rationale | What It Buys |
  |---|---|
  | Identical function to D19 (Block Pointer Structure & Width Budget) Settled Item 2 | One fewer implementation, one fewer checker, one fewer class of crash-point replay |
  | Hardware SSE4.2 `crc32` instruction available | One fewer cryptographic computation on commit path. ⚠️ **Unmeasured for a single cycle in this project**, serves solely as secondary rationale and forbidden from serving as evidence |

- **Take "full" rather than "truncated"**: Truncated CRC is not shorter CRC; taking truncated CRC invalidates 2⁻ⁿ curve.
- ⚠️ **Retracted rationale**: "CRC linearity ⇒ collision provably 0 when differences fall in ≤32 bit window, whereas truncated cryptographic hash yields unprovable 2⁻³²". **Fails**: "restricted window" premise was hardcoded by model — competing records' `named_count` / `record_length` / `nonce` were modeled constructively identical, whereas they belong to **distinct transactions**, and I-6.1 requires nonce uniqueness volume-wide.
- ⚠️ **Resulting counter-risk**: **When window condition fails, linear arm tail behaves worse than cryptographic hash.** CRC collision difference set is linear subspace — identical difference patterns **collide every time**; truncated cryptographic hash gives independent 2⁻³² per pair. Under existing headroom this leaves conclusion intact, **but constitutes sole evidence pointing opposite direction, forbidden from being omitted when citing settlement.**
- **Reopening conditions (two, either triggers reopening)**:
  1. Seam count denominator measurement basis overturned, reducing margin below 10×;
  2. Settled Item 10 coverage scope changes, invalidating "guaranteed distinct across timelines" premise — false acceptance rate becomes 100%, independent of function or width.

**Scope**: Governs solely reverse chain function; `header_csum` algorithm (CRC32C 4 bytes + 28 bytes zero) belongs to D18 (Information Carried in Blocks) Settled Item 17, `payload_csum` algorithm in Settled Item 13.

**Evidence**:

- E51 (Number of Reverse Chain Collision Opportunities): 32 bits against seam count denominator suffices across all tiers with three orders margin — function choice non-load-bearing.
- E61 (Uniformity of Reverse Chain Hash Algorithm): Metric 3 eliminates "effective bit width loss ≥ 1 bit", both candidates within bounds; Metric 2 voided, retracting "linearity provably 0" rationale.
- E40 (Ciphertext Checksum Bit Width): Truncated CRC discards "single-bit flip guaranteed detected" structural property — take full, avoid truncation.
- User settlement (corrected Metric 2 unrun), verbatim in history.

**Owed**: None.

#### Settled Item 12: Record Size Fixed at 4 KiB

**Ruling**: **Record size fixed at 4 KiB, fixed-length, written into system configuration** (XFS writes same parameter into system configuration `h_size`).

- Compatible with Settled Item 4: record header 487 bytes must fall completely within single `physical_block_size`, and 4096 is divisible by both 512 and 4096 probed values.
- Power-of-two fixed-length records ensure alignment waste for I-8.2 (Record Header Does Not Straddle Atomic Units) is **constant 0**: records placed back-to-back always start at unit-multiple offsets.
- 4 KiB record holds 64 named entries <!-- format-const: JOURNAL_NAMED_ENTRIES_PER_RECORD = 64 stale== 67 项 --> (Settled Item 4); larger sizes merely raise "worst-case transaction occupancy" in I-8.1 (Ring Geometry Sufficiently Large), raising ring size lower bound without other gains.
- **User measurement basis**: Land with this figure first, adjust during coding.

**Scope**: ⚠️ Headroom cell assumes "one transaction per fsync" interpretation (target workload 12-item transaction fits in 1 record), conflicting with D16 (Publish Semantics) Settled Item 5 end split discipline (one transaction writes at most one user data unit): under split discipline plus Settled Item 7 one transaction number per record, sequential batch 1 produces 8 records with 1 to 5 named entries each, eliminating "12 items fits in 1 record" cell; reconciling headroom measurement basis with ring geometry belongs to C310, awaiting user settlement. If target workload (D25 (Target Workload Priority)) grows an order of magnitude, recalculation required via separate experiment. Ring wrap-around and memory buffering costs unmodeled.

**Evidence**:

- E75 (Record Size and Ring Geometry): At 4 KiB target workload occupies exactly one record with 0 split records, identical capacity across header readings; ring lower bound matches published binary cell; power-of-two fixed size yields constant 0 alignment waste.
- User settlement, verbatim in history.

**Owed**: C310.

#### Settled Item 13: Payload Integrity, header_csum Covers Entire Record

**Ruling**: `header_csum` **covers entire record `[0, 4096)`**, including named entries array and subsequent padding, with `header_csum` 32 bytes participating as zeroes (32 bytes holding CRC32C 4 bytes + 28 bytes zeroes, D18 (Information Carried in Blocks) Settled Item 17); `payload_csum` 4-byte field **retained**, covering named entries array using CRC32C (identical to Settled Item 11 reverse chain, avoiding second algorithm). **First version no longer distinguishes "header landed, payload half-landed"** — failure of either checksum invalidates record, recovery stops at discontinuity (Settled Item 18 / I-8.3).

- Aligns with D18 (Information Carried in Blocks) Settled Item 17 verbatim "checksums of self-authenticating structures (root records, system configuration slots, journal record headers) cover entire slot including padding": journal record is self-authenticating structure, slot is 4096 bytes; covering header alone forces exception for journal records, absent in root records and system configuration slots.
- **Three other candidates eliminated by settled clauses, representing exclusions rather than trade-offs**:

  | Candidate | What Disqualifies It |
  |---|---|
  | Option A (single checksum covering entire record) | Loses discriminative power between "header landed, payload only half landed". The first version adopts precisely this "entire record" coverage scope while retaining the `payload_csum` field — thereby accepting the cost written in this row |
  | Option C (each named entry carries checksum of target it points to) | D4 (Checksum Location) checksum proves "the pointed-to unit is intact", **does not prove "this named entry itself landed intact"**, whereas payload integrity asks whether this named entry itself landed intact on disk |
  | Option D (Option C + `named_count`) | Does not fix Option C's hole, merely adding a count check |

- `payload_csum` resides in header, **must fall within `header_csum` coverage**, otherwise modifying it is silent (corrupting payload and checksum together passes both); `header_csum` covering entire `[0, 4096)` naturally satisfies this.

**Scope**: ⚠️ **Cost is explicit**: distinguishing "header landed, payload half-landed" from "single byte payload corruption" originally relied on checksum **differences**; covering whole record conflates these with "entire record unwritten" into identical outcome (record invalid); purchased benefit is "identical rule with other two self-authenticating structures". If reopening "header only", four-cell criterion table remains applicable.

**Evidence**:

- E76 (Discriminative Power of Payload Checksum): Additional payload checksum actively checks rather than echoing header (half-landed payload and single-byte corruption flagged exclusively by it); single-checksum form cannot type-theoretically answer "header landed tail missing"; verdict holds identically across digest functions, being product of coverage scope.
- User modal settlement taking entire record (main agent recommended header only `[0, 307)`), without third-party argument, overturnable, verbatim in history.

**Owed**: None.

#### Settled Item 14: Replay Lower Bound Given by Selected Root; Administrator Rollback, Failure Handling, and Instance Switchover

**Ruling**: **Recovery applies solely records where `(instance number, checkpoint_txg)` strictly exceeds the selected root; stale tail acts merely as an optimization for "where to start scanning the ring", no longer determining the replay set.**

**Complete prefix judgment rules comprise five requirements, none of which may be omitted** (when citing I-8.3 (Strictly Continuous Replay Prefix), cite this sentence together):
jsn strictly continuous (break and stop on discontinuity), **`(instance number, checkpoint_txg)` strictly greater than root watermark**,
apply only transactions with complete commit markers (D23 (Journal Role & Format) Settled Item 7),
verify named units individually before applying (D16 (Publish Semantics) Settled Item 7, E77 (Persistence Ordering of Publications) proves load-bearing) — **both copies (two position entries) of a named unit must be readable with valid checksums before applying**, and if either copy is unreadable or fails verification, that record and all subsequent records are not applied.
⚠️ **Fifth requirement: the unit of application is a single publish — if the legal prefix halts between two transactions of a publish, that publish is entirely unapplied, truncating the prefix determined by the first four rules at the publish boundary; when citing these rules, cite this requirement together.** (D16 (Publish Semantics) Settled Item 4) **How publish boundaries are identified**: the final record of a publish = record with flag bit 0 set to 1 (Settled Items 4, 17); if the final record of a publish is not reached in prefix, that entire publish is not applied.

When two mirrored copies count as "record present": **present if self-authenticating checksum passes on either copy** (verified comprehensively in crash-point replay — both states where jsn 3 has only one copy persisted successfully read back). Named units differ: **applied only after verifying both copies** (prefix requirement 4). Trade-off: On a two-disk pool, if an entire disk is dropped, records reside on the surviving disk but one copy of named units was on the lost disk, that record is not applied — recovery regresses to earlier version, losing the last confirmed (fsync returned) version; D16 (Publish Semantics) Settled Item 8 warm-up cannot cover this cell. User knowingly accepted (C519 cell: whether "verified on either copy" permits torn grafting of one new and one old copy targeted by E77 (Persistence Ordering of Publications) was unmeasured).

**Cross-instance boundaries, chain head, comparison ordering, counters, and W at switchover** (Notes 1–4):

1. **Prefix rules do not cross instance boundaries**: Chain connects after last record covered by selected root, stopping if next record's instance number differs from selected root (prefix rule 1 holds verbatim). **Last record covered by selected root** = record with largest jsn among records sharing instance number and `checkpoint_txg` with selected root — when publish splits into multiple records they share single `checkpoint_txg` (D16 (Publish Semantics) Settled Item 6), and final record of that publish marks boundary covered by root. **When that record is unreadable**, chain head connects to first readable record above watermark, provided its "sequence number within this publish" is 1 (Settled Item 4) and `checkpoint_txg` = selected root txg + 1; sequence not 1 indicates start of next publish is missing, breaking immediately. **"That record" identified by last record flag** — if none of readable records for selected root publish carry last flag, treated as "that record unreadable", taking branch where "first readable record with sequence 1 serves as chain head"; does not take readable record with largest jsn sharing txg (user decision 2026-09-24, Reading Option B; third-party `research/prompts/m2-presumed-clauses-r1-main-verification.md` K4 branch four arms matched without resolving candidate, zero attack rounds). When selected root is generation 0 root from mkfs, it covers zero records, and subsequent records belong to newer instances ⇒ zero records applied. Cost noted in D16 (Publish Semantics) Settled Item 7, whether new instance requires warm-up belongs to D16 (Publish Semantics) Settled Item 8.
2. **Record watermark `(instance number, checkpoint_txg)` ordered primarily by instance number**, matching jsn order; root selection orders primarily by txg, breaking ties with instance number (D22 (Synthesizing Unit Atomicity) Settled Item 7), the two ordering distinct objects and observing separate orderings. If txg were primary, first root of new instance takes "root ring max + 1", while committed records in open checkpoint of abandoned instance could have larger txg, mistakenly applied onto new instance root.
3. **Counter continues pool-wide**: New instance resumes writing from prefix end + 1 without resetting to zero. Counter at prefix end takes largest readable counter in ring: when both copies of selected root's own record are unreadable, first record of new instance is numbered at largest readable counter + 1, falling into slot of unreadable record (rewriting unreadable record); counter read from record header, not deduced from txg. Fixed-length ring slots determined by jsn (decoupling slot from sequence eliminated by E36 (Slot Mapping Dimension)), resetting causes new instance records to collide with slots still required by old prefix; Settled Item 9 48 bits calculated across whole volume lifespan. ⚠️ **Same holds for checkpoint_txg**: First publish checkpoint_txg of new instance (normal mount, switchover) ≥ max(txg of all root records in root ring, checkpoint_txg of all self-verified records in ring) + 1; normal mount performs full ring scan, switchover avoids re-scanning journal ring to compute lower bound (in-memory txg already ≥ everything seen during mount; selected root for switchover re-read from disk, see switchover clause). Identical quantity to C331 repair candidate "record scan watermark"; C331 cleared (changed to R2, see owed checks cleared table). Cost: sequence gap larger than previous implementation (up to length of abandoned timeline); accounting unaffected: D5 (Snapshot / Space Accounting Mechanism) Settled Item 2 accounting tree for each root holds only rows for that generation, omitting point deletions of old generations, sequence jumps leaving no orphan rows (C341 disappearance follows identical rationale).
4. **W at switchover takes maximum transaction number in re-issued checkpoint**: Completed transactions in open checkpoint all > W, redone under new write order. Per item 2, their records do not follow selected root; if W took literal "last applied transaction number", their units would be judged already published by predicate and unapplied by anyone (C287). When re-issued checkpoint contains no records with non-zero transaction number (re-issuing empty publish), W = 0 (Settled Item 19 ①: transaction 0 excluded from W max); W recorded in row of each write-order instance whose transactions are retained, switchover selected root chosen per switchover clause.

**First version has no clean shutdown flag**: Every mount uniformly follows recovery (full ring scan, verify each record, choose root, apply per five rules and Notes 1–4), without branching on whether previous unmount was clean; admission, warm-up, rollback candidates, and F enforcement never read such a flag. During the per-disk check before acquisition, the missing-units half of the lagging branch reads the slot in this disk's ring that holds the last record of that version (system configuration three-party round three verdict `research/prompts/m3-close-sysconf-r3-main-verification.md` section 2 U3 option B', implemented by `last_record_of_the_version_in_its_ring_slot_on_the_device` in `crates/singlefs-core/src/mount.rs`); this read does not count as branching on clean shutdown (the reading of C499 (Whether clean shutdown mark exists lacks clause), user settled 2026-10-20 "accept, name this read").

**Where first file version connects**: Before writing first file version, previous record following version to connect to must be decodable, with `checkpoint_txg` equal to that version's txg — new record takes its counter + 1 for counter and its record header for reverse chain (Settled Items 8, 9); failing this refuses before any writes (`publish_first_file` in `crates/singlefs-core/src/transaction.rs` returns `FirstFileVersionDoesNotFollowTheVersionItBuildsOn`), writing zero bytes.

**Administrator rollback is forward publish while mounted**: Administrator selects old root R_old from rollback candidate set during writable mount, file system issues standard publish (D16 (Publish Semantics) Settled Item 7 persistence ordering), user-visible trees (inode tree, extent tree) and `deleted_inodes` root pointer in new root take R_old version; tombstone tree and pedigree tree follow active version, omitting R_old, rollback publish writes pedigree record (T_rb, T_old) (D18 (Information Carried in Blocks) Settled Item 24); retirement tree follows tombstone tree and pedigree tree, following active version rather than R_old; tree table units, allocation record tree, central mapping tree, and accounting statistics (D5 (Snapshot / Space Accounting Mechanism) Settled Item 4) recomputed dynamically per adjusted allocator for this publish, not loaded from R_old accounting (inode number and tree ID watermarks see table "Watermark"); instance table follows cur; checkpoint_txg increments as usual, **without adding instance, acquiring number, writing instance table row, or applying/deleting any records**. Notation: cur = active version, new = new root issued by rollback, user-visible units = data units and units of extent tree and inode tree.

| Item | Settlement |
|---|---|
| Candidate Set | Roots in root ring judged still valid per cur instance table ∧ txg ≥ F_effective (D16 (Publish Semantics) Settled Item 1) ∧ carrying files (that version wrote files, not version before any file was written; version after deleting all files that has trees but empty file set counts as carrying files): (i, T) selectable ⟺ cur instance table has no row for i, or has row (i, Ti, Wi) with T ≤ Ti — roots on timelines abandoned by crash recovery can never be selected |
| Release | Placements still allocated in cur's accounting but not referenced by new have release generation = new's txg; quarantined units (D19 (Block Pointer Structure & Width Budget) Settled Item 5 Hard Rule 1: either copy verification mismatch keeps both copies allocated) excepted, remaining allocated (user decision 2026-09-27) |
| Resurrection | Among user-visible units referenced by R_old recorded as released in cur's accounting, revert to allocated, writing back allocation generation from R_old's accounting |
| Watermark | Inode number watermark = max(in-memory watermark of cur, watermark of roots readable in ring and not abandoned by crash recovery per instance table); cur in-memory term cannot be omitted: taking only ring causes rollback publish to reissue numbers (the hole in C143); when cur instance table is unreadable, code first refuses entire rollback (`crates/singlefs-core/src/mount.rs` `RollbackError::CurrentInstanceTableMalformed`), unreadable roots inherently ignored. Tree ID watermark as before = max(cur in-memory watermark, watermark of all readable roots in ring). Neither watermark regresses with R_old |
| Records | Record transaction number for rollback publish takes 0, without advancing intra-instance counter (same as empty publish: carrying no user transaction, D23 (Journal Role & Format) Settled Item 19 ①); F carried by new root takes F on current root, not counted as raising F; jsn connects to current record (main agent decision 2026-09-26, deduced, zero attack rounds) |
| Admission & Accounting | Rollback as before does not evaluate admission inequality; pedigree containers treated as fixed points, entering ckpt_cost Σ list; accounting labels 4 and 6 recomputed for new root version (inode tree and `deleted_inodes` taken from R_old, so tombstone share of resurrected objects raised accordingly; when label 6 recomputed for new root, n_live (D3 (Space Allocation) Settled Item 2) = sum of record counts in leaf container headers of R_old inode tree minus `deleted_inodes` key count, first version inode tree has single root and at most 135 containers), unevaluated (`research/prompts/m3-delete-r2-main-verification.md` section 3 V4; `research/prompts/m3-delete-r3-main-verification.md` section 3 G3-rollback; user decisions K-W5 "pedigree record", K-W5b) |
| Refuse Before Any Write | User-visible units referenced by R_old have no record in cur accounting, or remain allocated with allocation generation differing from R_old accounting; any node in R_old or cur accounting unreadable; every unit in resurrection set (recorded as released in cur accounting and reverted to allocated this time) must be readable with valid checksum on every disk, refused if either copy fails — their slots were released on cur timeline and might have had one disk copy overwritten by an uncommitted publish write; units referenced by R_old that remain allocated with same allocation generation in cur accounting but were quarantined (D19 (Block Pointer Structure & Width Budget) Settled Item 5 Hard Rule 1: reading disk finds one mismatched copy, keeping both copies allocated) likewise verified per-disk, refused if either copy unreadable or mismatched (user decision, verbatim in history); other units remaining allocated in cur accounting will not be issued by allocator and excluded from per-disk verification (same direction as prefix Rule 4 "apply only after verifying both copies"; main agent decision 2026-09-26, deduced, zero attack rounds) |

Crashing at any point during rollback publish recovers to outcome identical to any ordinary uncommitted publish; rollback root confirmed to administrator only after persistence (D16 (Publish Semantics) Settled Item 7). Rollback depth bounded by candidate set: entire root ring normally, narrowing to 4 most recent rollback states when disks are tight, with newest persistent valid root on each disk included. **Before roots of timelines abandoned by crash recovery leave root ring, units referenced by them may neither be reallocated nor have headers wiped — allocator queries accounting of every readable root in ring, quarantining solely slots referenced exclusively by abandoned roots** — querying takes narrow interpretation: reading subject as "roots of abandoned timelines", excluding slots referenced by valid roots — "valid roots" denotes roots in rollback candidate set (valid per instance table ∧ txg ≥ F_effective), recomputed on each mount and F raise; unreadable accounting (tree table or allocation record tree) of abandoned root does not skip: dynamically computes referenced placements from central mapping, quarantining in memory only without writing disk, reported in `abandoned_roots_unreadable`, and if mapping dynamic computation fails, rereads once, refusing writable mount before acquiring number if still failing while read-only proceeds (C393 takes (b)-recompute from mapping, user decision 2026-09-28, zero attack rounds, verdict `research/prompts/unreadable-at-mount-r2-main-verification.md` L3); when accounting of abandoned root is unreadable, normal unmount, session writes, and subsequent writable mounts should succeed, retaining refusal only when credentials absent: credential is that in-memory active accounting holds record with generation > root txg covering tree table slot (and mapping root slot), inspecting accounting without reading slot contents, releasing refusal if present and refusing if absent (C694 credential release, user decision 2026-10-07 "implement both C and B", zero attack rounds); raising F chain (admission and unmount) bypasses this: abandoned roots calculate shadow accounting from placements recorded in root ring table during mount, without re-reading disk accounting or refusing, refusal retained only for subsequent writable mount (D16 (Publish Semantics) Settled Item 1 "root slot read error this time" row, user decision 2026-10-07, zero attack rounds); wide interpretation (quarantining slots referenced by any readable root in ring) prevents raising F from buying anything across 24 slots in first version (shadow accounting; how quarantined volume enters admission inequality see C318). Same settlement (C113 settlement P3) splits transactions by unit ⇒ unit of crash atomicity is single unit, not write request (no decision in repository ever promised write-request atomicity, representing shrinkage of external semantics).

**Pre-quarantine during writable mount** (C694 C, user decision 2026-10-07 "implement both C and B", zero attack rounds): During writable mount read phase after allocator reconstruction, placements named by unapplied journal records above watermark are quarantined regardless of whether root is rollback candidate root, excluding slots still allocated in active accounting (already allocated; allocator supplements quarantine upon release/reclamation during this mount, D28 (Mount-Time Commitments) Settled Item 1 item 9); uses new root segment of any readable record from that publish to read accounting, quarantining placements in that accounting (`research/prompts/m3-batch33t-code-r1-main-verification.md` section 2 "t1 C candidate exemption" and "t1 C blind spot", zero attack rounds); unions placements from readable roots in root ring newer than selected version (per root selection order) and valid per newest root instance table, grouped by root ring slot corresponding to root txg (`C_NEWER_ROOTS`). When these accountings are unreadable: if readable records of publish complete (sequences 1..N continuous with last flag), quarantine not added, deferring to "refuse writable mount when records have gaps"; if incomplete (missing sequence or last flag), follows unreadable abandoned root procedure, computing from central mapping and quarantining in memory, rereading once if computation fails, and refusing writable mount before number acquisition if still failing without active accounting credentials, reported separately from C393 refusal (`C_MAPPING` narrowed; user decision 2026-10-08 "Option B: core adds error variant"); when newer readable root publish records completely unreadable and accounting unreadable: quarantine if mapping computes; if mapping fails, inspect tree table and central mapping root pointers of root itself (C834, user settlement 2026-10-15 "both copies of identical pointer updated", verdict `research/prompts/c834-r3-main-verification.md` section 4, zero attack rounds; replacing 2026-10-08 Y2 tightening): if either pointer has both copies overwritten by newer instance units, do not refuse or reread (in-repo test case `urJ`); otherwise follow incomplete record branch (reread once, inspect credentials, refuse writable mount before number acquisition if still failing). `C_NEWER_ROOTS` and `C_MAPPING` follow verdict `research/prompts/m3-batch35-36-code-r1-main-verification.md` section 2 lines H1, H3 and section 3 item 1, measured only on attacker model, zero attack rounds. All C placements released only after this process overwrites root ring slot corresponding to record txg and completes placement acquisition, not released during in-process reread of zero-read slots per D16 (Publish Semantics) Settled Item 1. Protects newly allocated units of root whose root slot was unreadable at mount and unwitnessed by system configuration: shadow accounting queries only readable roots in ring, blind to this root; publishes witnessed by system configuration governed by Option B-Config-Cont ("unreadable updated state read by writable mount"). Quarantined slots count toward admission inequality item 9 (D28 (Mount-Time Commitments) Settled Item 1). **Refuse writable mount when records have gaps** (user decision 2026-10-07 F1 "Option B: refuse writable mount on missing records", zero attack rounds): For unapplied publish above watermark, if readable records have discontinuous sequence or miss last flag, and any named unit fails verification on both copies (`research/prompts/m3-batch33t-code-r1-main-verification.md` section 2 line t4, zero attack rounds), reread once, refusing writable mount if persisting. If writable mount regressed per "selected root read failure and unwitnessed by system configuration regresses to witnessed version", re-evaluate using regressed version as watermark, rescanning the ring before re-evaluating and using the rescanned records, not the ones scanned in the read stage (C754 (re-judging missing record after rollback does not rescan ring), user settled 2026-10-20 "rescan"): if unapplied publish above that version behaves similarly, reread once, refusing if persisting with identical refusal cause (`F1_AFTER_F2`, verdict `research/prompts/m3-batch35-36-code-r1-main-verification.md` section 2 line H2 and section 3 item 1, measured only on attacker model, zero attack rounds).

**Unreadable updated states read by writable mount (C554 Option B-Config-Cont, user decision 2026-09-27; Criterion N-Config-Cont per `e158-r3-prereg.md` "5.1 Which form Criterion N uses" and `e158-r4-prereg.md` "5.1 Which form Criterion N uses")**: After completing read phase (root selection, journal scan, replay), read system configuration of both slots on every pool disk: undecidable ⇒ treat as true (undecidable takes R2, authoritative text in D22 (Synthesizing Unit Atomicity) Settled Item 8: device reports read error on slot and no disk has both slots self-verified; user decision "switch to R2"; implementation takes R2: `system_configuration_witness_is_undecidable` in `crates/singlefs-core/src/recovery.rs`, shared between read phase and number acquisition; injection in multi-mount crash-point replay evaluated by E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bounds) r3 root selection and replay suite under multi-mount histories with read failures as KJ1T, C331 cleared, see owed checks cleared table); when decidable, take maximum journal tail across all self-verified slots as c_witness, 0 ⇒ unevaluated; if record with last flag of selected version publish is readable (either copy) ⇒ true if c_witness > its counter; if unreadable and record with counter equal to c_witness is readable ⇒ true if its (instance number, checkpoint_txg) > selected version; if both unreadable, treat as true. If true, redo read phase on read cache valid for this mount (storing only successful non-zero reads) (R = 1, from D16 (Publish Semantics) Settled Item 1 "reread once", with zero wait between reads); proceed if redone pass evaluates false, and refuse writable mount before number acquisition if still true, leaving disk bytes unchanged while read-only mount proceeds. If newest root's instance table used for allocator reconstruction abandonment check is unreadable, reread once, refusing writable mount if persisting (code review item 22). If union of two root ring read passes yields zero non-abandoned self-verified roots, refuse writable mount before number acquisition, reporting `MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread` (D16 (Publish Semantics) Settled Item 1), listed on refusal roster; mount transparency (`crates/singlefs-harness/src/mount_transparency.rs`) recomputes "no non-abandoned valid roots in ring per newest root instance table" on image, judging illegal if true without read failure injection, unevaluated when injected, and bypassing filter when instance table unreadable. **"Continuation"**: Number acquisition write (`write_acquired_instance` in `crates/singlefs-core/src/transaction.rs`) no longer writes tail = 0, writing witness value c_witness read at acquisition moment (user decision 2026-09-27 "Option B-Config-Cont (Recommended)"; implementation merged 2026-09-27, landing in `highest_journal_tail_of` in `write_acquired_instance`: takes maximum journal_tail across all self-verified system configuration slots with matching fsid on both slots of each disk, 0 if none, read once after first barrier and before first acquisition write). Rollback write upon acquisition failure (D18 (Information Carried in Blocks) Settled Item 11) carries same witness value, not rolling back to 0. mkfs writes tail 0, first acquisition post-mkfs reads 0 and writes 0: preserving bytes of new-pool file creation. Acquisition moment evaluates identical undecidability (R2, see D22 (Synthesizing Unit Atomicity) Settled Item 8): refuse before first acquisition write if undecidable (`self_verified_system_configurations_of_every_device` in `crates/singlefs-core/src/transaction.rs`, takes R2; also refuses when disk lacks any self-verified system configuration, user decision 2026-10-03; multi-mount crash-point replay injection evaluated by E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bounds) r3 root selection and replay suite, C331 cleared, see owed checks cleared table). Newest roots unwitnessed by system configuration visible to subsequent mount in its c_witness post-"Continuation"; owed item see C554 Q1.

**Selected root read failure and unwitnessed by system configuration regresses to witnessed version** (user decision 2026-10-07 F2 "Option B: implement in all three locations", zero attack rounds): Recovery, writable mount read phase, and read-only mount root selection share identical logic, regressing to newest readable root in `[witnessed version, selected root)` not abandoned by selected root instance table (ordered by root selection: checkpoint_txg primary, instance number tie-breaker); the three places judge whether to fall back by the same union: if any unit or record read by the three readers of the selected version—walk, open, and building the previous version—(including the last record that cannot be read) is unreadable, undecodable or fails its check, and system configuration has not witnessed it, fall back; the union only decides whether to fall back, not success, and after landing on a version, recovery looks at the walk, read-only mount at open, writable mount at building the previous version, each succeeding or failing by its own reader; if the fallback cannot be made, stay on the selected version. The records handed to the judgment in the three places come from the same source: after the trigger, rescan the journal ring once and use the rescanned copy; in writable mount, F2, the record needed to build the fallen-back version, and the F1 re-judgment (`F1_AFTER_F2`) all use that same post-trigger scan (C755 (recovery and two mount triggers fail differently on rollback), user settled 2026-10-20 "add all three sentences", the three added sentences are zero-round wording; implemented in the implementation batch); witnessed version = version in root ring matching (txg, instance) of record with counter = c_witness. When selected root instance table is unreadable, recognizes solely readable root strictly equal to witnessed version, and when the witnessed version and the selected root share the same instance table pointer, additionally accepts readable roots in the interval with the same instance as the selected root, treating them as not abandoned (both pointers are in root records, the table is not read; C751 (read-only mount bypasses confirmed version when instance table unreadable), user settled 2026-10-20 "Option A: accept only with a shared instance table pointer", zero rounds; implemented in the implementation batch); if that record is unreadable, c_witness = 0, or interval lacks readable non-abandoned root, do not regress; when c_witness lands on record of selected publish itself, interval is empty, selected version counted as witnessed without regressing (`F2_NEWEST`, verdict `research/prompts/m3-batch35-36-code-r1-main-verification.md` section 2 line H4 and section 3 item 1, measured only on attacker model, zero attack rounds; unreadable record and c_witness = 0 non-regression per `research/prompts/m3-batch33t-code-r1-main-verification.md` section 2 line T5). Selected version counted as "witnessed": record with last flag for that publish readable with counter ≤ c_witness; if last record unreadable, compares (txg, instance) of record with counter = c_witness, counted witnessed if not older than selected version; if both unreadable, treated as witnessed without regressing. F2 cannot reach the form "a publish in the interval whose last record is readable but whose root is not"; the argument has two halves: without injected read faults F2 is not invoked; every publish rotates the system configuration, so c_witness lags by at most one publish (scope: no injected read faults, system configuration rotation writes land; C758 (publish where last entry readable but root unreadable without read fault unmeasured), user settled 2026-10-20 "add one more test case to pin it"). Outside that scope, the form where system configuration writes are silently lost (system configuration writes on both disks report success but do not land, followed by 30 more overwrites, selected root tree table corrupt) is pinned as test cases: recovery, read-only mount and writable mount all fall back to the newest readable root in the interval, below which lie the publishes whose root slots were overwritten (the three test cases in `crates/singlefs-harness/tests/system_configuration_write_lost_silently_falls_back_without_a_read_fault.rs`).

**Failure handling splits into two branches based on failure nature** (C113 settlement P1 failure table). **Discriminator is quantity decidable at occurrence time, not count**: When failure occurs, first perform **probe write to target device fixed placement** — write failure ⇒ persistent failure, **transition to read-only until next mount**; write succeeds ⇒ perform **read-only verification on failed placement**: readable ⇒ transient failure, perform **instance switchover**; unreadable ⇒ placement-level persistent failure, **transition to read-only until next mount**. ⚠️ Verification step is **read-only without writing**: writing probe into failed placement wipes authoritative data (4 of 8 failure points land on root or system configuration slots), and fixed placement step cannot be modified — D2 (RAID Stripe Policy) Settled Item 13 verbatim "shares identical action with D23 (Journal Role & Format) Settled Item 14 two-branch discriminator" depends on it (main agent check: statement in `.claude/kb/decisions/02-RAID条带策略.md`, under Settled Item 13 "read-only during degraded mode", not Item 15). **When failure point is barrier, no placement exists to verify**, where failure handling consists solely of fixed-placement probe write. If writable device count drops below w lower bound, or switchover reservation cannot be acquired, directly enter read-only branch; consecutive switchovers exceeding **N_switch = 3** (named tunable parameter) transitions to read-only, serving as safety net while true discriminator remains probe write plus placement read verification. **Convergence relies on first step of switchover being a write that will fail**: switchover begins by acquiring number, writing all successfully opened system configurations all-or-nothing ⇒ "write-out" failures advance switchover at most one step (probe write blocks first, and all-or-nothing fails immediately even if bypassed).

- Blocks required for switchover reserved at mount admission: **it is an in-memory space quantity**, recomputed on each mount, lost on crash, **never persisted to disk** — allocation record entries express only "allocated" and "released + release generation" (D3 (Space Allocation) Settled Item 7), incapable of expressing third state on disk, while persisting requires writing units, returning to "must write to write". Reservation sized at **(N_switch + 1) × worst-case volume of single switchover** (single mount permits N_switch switchovers, and instance table chain grows by one row per switchover requiring copy-on-write rewrite of entire chain, making k-th switchover costlier than 1st; extra copy serves metadata for write-row publish of this mount); **measurement basis and formula authoritative in D28 (Mount-Time Commitments) Settled Item 3**: worst-case volume of single switchover = instance table chain rewrite (replica count 2 × 32768 × slices, per device) + warm-up empty publishes (at most R times × dynamically computed c_max); **fixed-point redoing and data unit rewriting with sequence > W require no extra space** — switchover is in-mount recovery where failed allocation does not exist in reconstructed state, redoing draws from checkpoint reserve pool; cite D28 (Mount-Time Commitments) Settled Item 3 when referencing this paragraph.
- **Reservation blocks allocation failure path, not write failure path** — write failures blocked by probe write and number acquisition all-or-nothing.
- **"Allocation failure" splits into three branches based on occurrence location** (failing to split causes entire volume to enter read-only when full, while releasing space requires writing disk and remount is blocked by reservation conjunction, violating `.claude/rules/fs-design.md` "releasing space itself requires no space allocation") — foreground user data allocation failure ⇒ **return ENOSPC to caller**, checkpoint publishes normally without altering mount state; background, continuable, non-decision paths (rebalance, tombstone reclamation, scrub, defragmentation) allocation failure ⇒ **pause task and record into intent**, without altering mount state (`.claude/rules/fs-design.md` third cell "no commandments", D2 (RAID Stripe Policy) Settled Item 4 c "foreground uninterrupted"); **switchover fails to acquire reservation** ⇒ transitions to read-only.
- Root slot write failure re-issues after advancing checkpoint_txg by one (root ring region = `txg mod R`, D22 (Synthesizing Unit Atomicity) Settled Item 2; failing to advance repeatedly rewrites same slot, collapsing R failure domains into one); ⚠️ "consecutive failure of same root slot" ceases to be criterion — slot rotates with txg, remaining unreachable when N_switch ≤ R, leaving criterion empty.
- **Instance switchover** = in-mount recovery: acquire new instance number, write row, re-issue in-flight checkpoint — selected root **re-read from disk**: re-selected against open device handles per recovery root selection rules (`choose_root` in `crates/singlefs-core/src/recovery.rs`), not taking in-memory root — when root persisted but system configuration slot failed, in-memory root is staler than on-disk active root (C458); rows fall in `[this root instance, new instance)`, each retained-transaction write-order instance k writes (k, k last published txg (0 if never published root), maximum transaction number belonging to k among retained transactions), remainder per D18 (Information Carried in Blocks) Settled Item 11 (when old instance published root and retained transactions belong to it, row is (old instance, last published txg, max transaction number in re-issued checkpoint)); transactions with number ≤ W retained (open checkpoint completed transactions all > W, C287 closed by this W, Note 4); transactions with number > W redone under new write order or return error to unreturned callers; fixed-point units completely rewritten per new instance; no transition to read-only, no waiting for remount, cost = one row + full instance table chain rewrite + data unit rewrite for number > W + single fixed-point redo, frequency unmeasured.

**Failure handling in this version**: Publishes do not accept failure. Failed publish is frozen, re-issued byte-for-byte before next publish (checkpoint_txg, counter, sequence within publish, flags, unit positions and bytes unchanged); subsequent publish created only after successful re-issuance; thus valid history never contains two records with last flag sharing (instance number, checkpoint_txg), and recovery encountering this halts before any writes as disk corruption (`RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided` in `crates/singlefs-core/src/recovery.rs`). Byte-identical re-issuance governs serial commit in this version; probe writes, two-branch diversion, instance switchover, and root slot txg advance belong to this item, scheduled for subsequent milestones alongside failure handling code. Byte-identical re-issuance under serial commit amplifies single failure into blocking entire publish stream, owed C541.

**Scope**: Governs which segment recovery applies, how rollback and switchover operate, and failure diversion; instance table row format and writing rules belong to D18 (Information Carried in Blocks) Settled Item 11, switchover reservation sizing belongs to D28 (Mount-Time Commitments) Settled Item 3, defer window and F belong to D16 (Publish Semantics) Settled Item 1. Known edge cases:

- ⚠️ Narrows rather than overturns Settled Item 3: "full ring scan first, verify record by record, never trust tail first" holds verbatim — changing solely **which segment of verified legal prefix is applied**.
- ⚠️ **Shadow accounting and instance table abandonment check (`abandoned_by_table`) retained for crash recovery, not administrator rollback**: In history devoid of administrator rollbacks, crash recovery regressing to old root due to transient read error causes instance table to abandon newer root; disabling shadow accounting permits units referenced by abandoned root to be reused, causing subsequent recovery landing on it to fail reads (round 1 H6). "Quarantining solely during mount where crash recovery landed on old root" fails: reused writes occur in subsequent clean mount (round 2 defense G6, code inspection without metrics). Codified as [checks-owed.md](../checks-owed.md) C314. Following C554 Option B-Config-Cont (user decision 2026-09-27), transient unreadability of newer roots witnessed by system configuration no longer triggers abandonment: writable mount rereads once, refusing writable mount if persisting; shadow accounting rationale narrowed to unwitnessed form (publish system configuration rotation unwritten to disk); exception: when root slot, records, and newer system configuration slots on each disk all read back zeroes, witnessed version is similarly abandoned (Settled Item 14 edge case ②).
- ⚠️ **C331 cost after adopting R2 for undecidable witness (authoritative text for R2 in D22 (Synthesizing Unit Atomicity) Settled Item 8; metrics under Option A in `research/prompts/unreadable-at-mount-r2-main-verification.md` L1)**: Torn system configuration rotation (producible via plain power cut, unconfirmed publish) no longer refuses writable mount due to undecidability; remaining undecidable case occurs when slot device reports read error and no disk has both slots self-verified (one slot persistent read error), where writable mount refuses repeatedly and read-only mount writes zero bytes, leaving pool in read-only mount; "read-only mount writes zero bytes" verified against 2026-09-28 code (2016 traces), missing from clauses, see C32, C33, C75. Escaping when zero writable mounts succeed lacks clause (C596). Form in C393 failing recomputation from mapping similarly trapped in read-only mount.
- ⚠️ **Three known edge cases of in-mount rollback**: ① Verification "R_old still-allocated placement shares slot and allocation generation in cur accounting" absent from settlement and unpatchable — released records store release generation (`AllocationRecord` in `crates/singlefs-core/src/allocator.rs`), unable to detect "slot reused then released again"; permitted erroneous rollbacks reachable only when F regresses (round 3 verdict section 2 K1: 12 of 15 cells when F takes min of max, 0 when taking SysPre), prevented by non-regressing F. ② Crash recovery abandoned roots all transiently unreadable at reopen, shadow accounting reads 0, subsequent instance reuses slots, removing failure and corrupting new instance roots causes recovery landing on abandoned roots to fail — multi-failure independent of rollback form, see C554 — Option B-Config-Cont prevents case where witness slot readable while roots newer than selected version unreadable (reread once, refuse writable if persisting); when root slot, records, and newer system configuration slots all read zeroes, confirmed witnessed version similarly abandoned, sharing class with "cost of F2" (third-party verdict `research/prompts/m3-c554-c572-r1-main-verification.md` section 2 line "abandoned instance roots all unreadable at reopen"). ③ Rolling back back-and-forth across root ring leaves fewer than 4 distinct reachable states (D16 (Publish Semantics) Settled Item 1 root ring capacity boundary).
- Implementation chain head anchored at last record covered by selected root (`counter_of_the_last_record_the_root_covers` in `crates/singlefs-replay-prefix/src/lib.rs`, `replay_journal` via `replay_core` in `crates/singlefs-core/src/recovery.rs`), checked by `one_missing_record_right_after_the_chosen_root_stops_the_prefix_even_when_later_records_are_intact` in `crates/singlefs-harness/tests/writable_mount_takes_a_second_instance.rs` and mutation rows naming anchor offset (C365 cleared, see owed checks cleared table).
- Rollback label 6 n_live dynamic computation reading ("Admission & Accounting" cell) had zero attack rounds.
- ⚠️ **Read-only verification buys protection against persistent placement failures, with cost of zero attack rounds.** Probe write alone causes persistent placement failures to trigger repeated futile switchovers (attacker measured 46 switchovers across 24 persistent failure scenarios on replica harness, unrepeated in repository harness); read-only verification discriminates this. Proposed by attacker leg, **underwent zero attack rounds**, accepted knowingly by user 2026-09-21.
- **Three corollaries of switchover re-reading disk to choose root**: When disk-selected root is newer than in-memory root, do not re-issue in-flight checkpoint, proceeding from disk-selected root; switchover instance table row and W written per disk-selected root; in rollback publish, administrator-selected R_old must fall within rollback candidate set calculated using disk-selected root as newest root, refusing with disk unchanged if absent. In-mount instance switchover implementation scheduled with milestone "overwrite, release, rollback and reuse" closeout table rows 16 and 40 (C458).
- ⚠️ **Zero-unit publish branch does not freeze**: When barrier following post-publish system configuration rotation errors, unit-bearing / record-bearing publishes share frozen byte-identical re-issuance; zero-unit publish (`publish_without_units` in `crates/singlefs-core/src/transaction.rs`, passing `persist_the_root_then_rotate_the_system_configuration`) does not freeze, returning mount error (`PublishError::BlockDevice`), next publish falling to next mount under new instance; choice made for unwritten branch (verdict `research/prompts/m2-closeout-code-r2-main-verification.md` section 1 "newest root unreadable, witness, and barrier"; `m2-impl-c577-barrier-implementer-report.md` design issue 2). Pinned by `a_zero_unit_publish_whose_post_rotation_barrier_fails_is_not_frozen_and_the_whole_mount_returns_the_error` in `crates/singlefs-harness/tests/a_zero_unit_publish_whose_post_rotation_barrier_fails_returns_the_mount_error.rs`: mkfs pool first writable mount, row-writing zero-unit publish errors at barrier after disk 1 rotation ⇒ mount returns `MountError::Publish`, cause `PublishError::BlockDevice`, 0 publishes persisted, 1 failed; recorded stream stops at disk 0 barrier, disk 1 omitted; remount on non-error devices acquires instance 2, choosing (instance 1, txg 1). Pins image outcome: harness file device lacks write cache, disk 1 rotation write entered image before barrier error, omitting crash state where rotation unpersisted at barrier error.
- ⚠️ **C366 settlement has no reachable disk state at writable mount layer**: Note 3 "counter at prefix end takes largest readable counter in ring" separates from txg only when selected root's own record is unreadable on both copies; that form judged "treat as true if both unreadable" by C554 Option B during writable mount read phase, refusing writable mount before acquisition if persisting on reread (C579 user maintained Option B literal), unreachable by number acquisition, row writing, or warm-up. Pinned by: function-level `warm_up_after_a_journal_counter_other_than_zero_counts_records_on_from_it_while_txg_stays_one_and_two` in `crates/singlefs-harness/tests/warm_up_journal_counter.rs` (ring last jsn 40 writes warm-up jsn 41, 42 with txg remaining 1, 2); same file `c366_when_the_chosen_root_own_record_is_unreadable_the_writable_mount_is_refused_by_the_undecidable_witness_and_the_read_only_mount_opens_the_chosen_root`: writable mount refused before write, disks unchanged, read-only mount chooses (instance 1, txg 4), applying zero readable jsn 1–3 records (`m2-investigate-three-reds-report.md` section 1 item 2, section 7).
- Instance table row deletion sweep (D18 (Information Carried in Blocks) Settled Item 11 "reclamation") protects units referenced by abandoned roots: successful zero-read root slots treated identically to unreadable (counted as "potentially containing root of that instance"), unremoved this pass; other slots require self-verification, otherwise reread once, unremoved if still failing (zero rounds, deduced, no disk image created).
- ⚠️ **Cost of regressing to witnessed version** (knowingly accepted by user 2026-10-07 F2): Confirmed version silently regresses to previous version when newer system configuration slot and units read back zeroes simultaneously; after writable mount acquires number on previous version, that version is abandoned (deduced, unmeasured) (`research/prompts/m3-c694-r2-main-verification.md` section 2 line "K3 recovery fix", section 3 item 7, section 4 F2).
- First version active snapshot refuses administrator rollback: refused if current or target tree table contains code 14 / 15 entries (user settlement). Form when releasing rollback (tree table takes target version wholesale, release/resurrection sets computed from live heads ∪ live snapshots ∪ deadlists, destroying snapshots excluded from references) achieved 0 violations on attacker model across zero rounds, recorded in verdict `research/prompts/m4-snapshot-r1-main-verification.md` section 3 S8.

**Evidence**:

- E78 (Starting Point for Replay): Stale tail coupled with block reuse breaks triad "full ring scan + discontinuity stop + idempotence" — recovery halting on mismatch self-aborts on healthy images, while skipping mismatch drops tear discrimination to zero; watermark and tail-deletion arms pass all criteria, watermark arm chosen because watermark matches existing root record `(instance number, checkpoint_txg)` (D22 (Synthesizing Unit Atomicity) Settled Item 7), introducing zero new format fields, whereas tail-deletion adds retry ring in recovery.
- E77 (Persistence Ordering of Publications): Removing pre-application named unit verification produces silent grafting when units share segments with records — requirement 4 is load-bearing.
- E104 (Current Version Determination in Scan Rebuild): New root taking T_old + 1 fails to supersede abandoned roots — Notes 2 and 3 rely on it to ensure new instance initial publish starts from ring max + 1; other half "abandoned units resurrect on full-rule arm without rollback row" remains load-bearing — design avoids rollback rows (tombstone round 1 verdict `research/prompts/m3-tomb-r1-main-verification.md` section 2, `research/prompts/m3-delete-r3-main-verification.md` section 3); scan rebuild recognizing "mounted root was rollback" relies on pedigree records, reader usage owed (K-W5b, grouped with C113).
- E150 (Rollback Reuse of Units Referenced by Abandoned Roots): Shadow accounting achieves zero violations across two model cells, quarantining extra units until newest abandoned root overwritten. Model abandoned timeline created by administrator rollback; shadow accounting retained for crash recovery abandoned timelines (round 1 H6), sharing mechanism without re-running that form.
- E142 (Dry Run of New File in New Pool): Harness took "present if either mirror copy valid", reading all valid records despite corrupting one copy — standardizing "record present" measurement basis.
- Third-party: `research/prompts/c199-r1-main-verification.md`, `c199-r2-main-verification.md` (Notes 1–4); `c322-r3-main-verification.md` (acquisition barrier and skipped numbers); `c329-c330-r3-main-verification.md` (switchover selected root and W row); `c143-r3-main-verification.md` (first new root txg takes CJ2); `m2-r1-main-verification.md` section 3 and `m2-step45-code-r3-main-verification.md` section 6 item 5 (shadow accounting targets, valid roots); C113 settlement (`_c113-r9-final.md`).
- User settlements (replay lower bound, rollback, failure table, switchover reservation measurement basis, mirror copy measurement basis), verbatim in history.
- `research/prompts/c381-r3-main-verification.md` section 3: Candidate Option B (probe write changed to failed placement) hit independently by three points — barrier lacks placement, 4 of 8 failure placements are authoritative structures, collides with user round 2 item 1 (placements in address space table buying "safe to corrupt"), making D2 (RAID Stripe Policy) Settled Item 13 statement false.
- User settlement 2026-09-21 (Fix B), verbatim in history.
- User settlement 2026-09-23: Switchover selected root re-read from disk (`research/prompts/c381-r3-main-verification.md` section 2 zero-round form, section 4 item 3), verbatim in history; zero attack rounds.
- User settlement 2026-09-23: Note 3 prefix end counter takes largest readable counter in ring (C366 implementation writer identified clause omitted case where selected root record unreadable), verbatim in history; unreached at writable mount layer, C579 user 2026-09-27 maintained Option B literal refusing writable mount, verbatim in history.
- User settlement 2026-09-23: Requirement 5 publish boundary recognizes last record per Settled Item 17 convention (C245 round 2 `research/prompts/c245-r2-main-verification.md` section 4 discriminator, zero attack rounds), verbatim in history. Last record recognized by flag bit 0 (Settled Item 17); publish omitted if flagged record unread.
- User settlement 2026-09-23: Note 1 anchor takes largest jsn in same txg, unreadable anchor connects chain head via publish sequence number, first version lacks clean shutdown flag (third-party round 1 `research/prompts/m2-presumed-clauses-r1-main-verification.md` K3, K4; K4 two fixes zero attack rounds, sequence number deduced, implemented: sequence number within publish, Settled Item 4), verbatim in history.
- User settlement 2026-09-24: First file version verifies previous record before write, codified per 2026-09-24 implementation (C495, third-party `research/prompts/m2-refusals-presumed-r1-main-verification.md` section 3 R5: zero bytes written before refusal, unreachable under zero failures), verbatim in history.
- User settlement 2026-09-24: Switchover re-reading disk corollaries adopted per main agent recommendation (verdict `research/prompts/m2-newq-r1-main-verification.md` section 3 N5–N7, verbatim in history); no experiment: preserves on-disk format, semantic difference.
- User settlement 2026-09-24: Last record spans multiple records, final record carries flag (verbatim in history; third-party `research/prompts/m2-treesplit-r1-main-verification.md` T6 (Whether refusal is maintained depends on pool history), recognizing last record per pre-settlement text applies half publish, flagged cell 0 / 0).
- User settlement 2026-09-24: Last record recognized by flag, Note 1 anchor Reading Option B (verbatim in history; Implementation 13 report `m2-p6-header-implementer-report.md` section 4 ① two readings).
- User settlement 2026-09-24: Publish failures re-issued byte-identical in this version (Implementation 20 report `m2-lastflag-implementer-report.md` section 6 item 1: subsequent publish in same instance leaves two records with last flag sharing (instance number, checkpoint_txg)), verbatim in history; no experiment: defines failure emission without altering format.
- `research/prompts/m2-final-code-r2-main-verification.md` section 3 Z8 (publish failures re-issued byte-identical, attacker unhit); Z9 and section 4 fixes 1–4 belong to rollback witness, non-load-bearing following witness removal.
- Administrator rollback converted to forward publish while mounted (candidates, release, resurrection, watermark, refusal), removing prefix rule 5 and rollback witness, shadow accounting rationale redirected to H6: no experiment, third-party prototype (`research/prompts/m2-rollback-forward-r1-opus-model/`, `m2-rollback-forward-r2-opus-model/`, `m2-rollback-forward-r3-opus-model/`) measured on frozen replica without experiment number; verdicts `research/prompts/m2-rollback-forward-r1-main-verification.md`, `m2-rollback-forward-r2-main-verification.md`, `m2-rollback-forward-r3-main-verification.md`. Watermark taking max(memory, ring) tightening (verdict round 3 section 2 K1) zero attack rounds; user settlement 2026-09-25 (forward publish, mounted only) and 2026-09-26 (round 3 four questions), verbatim in history.
- Main agent decision 2026-09-26: Rollback publish verifies resurrection set units per disk before write ("Refuse Before Any Write" final sentence), per user 2026-09-25 verbatim "1 is dropping one disk after fsync failure. Cannot accept, violates data safety commitment" (`records/2026-09-24-里程碑二收尾调度.md` section 3), sharing direction with C519 "apply only after verifying both copies"; no experiment, deduced, zero attack rounds; read overhead scales with released user-visible units post-R_old, unmeasured.
- User settlement 2026-09-25: Named units applied only after verifying both copies, accepting loss of confirmed version upon whole-disk loss (C519 reproduction: `crates/singlefs-harness/tests/whole_device_loss_after_warm_up.rs`), verbatim in history; no experiment: codified per 2026-09-25 implementation, whether alternative "verified on either copy" permits E77 (Persistence Ordering of Publications) grafting unmeasured.
- User settlement 2026-09-27 (C554 Option B-Config-Cont, selecting Option B then "Config-Cont (Recommended)"): Writable mount encountering unreadable updated state witnessed by system configuration rereads once, refusing writable mount if persisting; number acquisition write no longer writes tail = 0, writing witness value read at acquisition moment (merged, `highest_journal_tail_of` in `crates/singlefs-core/src/transaction.rs`); Criterion N-Config-Cont defined in `e158-r3-prereg.md` "5.1" and `e158-r4-prereg.md` "5.1", verbatim in history. E158 (Four Forks for Root Selection and Repair) run 4: segment 3 H1g separates Option B-Config and Option B-Config-Cont (16/32 vs 0/32), segment 2 identical across four suites.
- Rollback label 6 n_live dynamic computation reading: `research/prompts/m3-blanks-r1-main-verification.md` section 4 line "how n_live dynamically computed at rollback" (zero rounds; main agent adopted forward leg reading).
- Third-party verdict `research/prompts/m3-ring-r3-main-verification.md` section 2 line 9: Sweeper treating successful zero-read root slots as absence wipes units referenced by abandoned roots still in ring, violating explicit protection; zero-read root slots not treated as absent, requiring self-verification or zero-read otherwise rereading once before deletion, two tightenings adopted (zero rounds, deduced, unmodeled). No experiment: protects abandoned timeline remnants rather than active data.
- Rollback to emptied version (trees exist, empty file set) counted as candidate: main agent decision 2026-10-02 "adopt Q-K", interpreting candidate set text "that version wrote files" (`records/2026-10-01-删除与并发事务.md` batch 8); main agent adopted ninth implementation writer identical choice 2026-10-03 ("rollback to emptied version adopted, consistent with Q-K"). No experiment.
- Allocator rebuild reading root ring two passes unioned lacking valid root refuses with dedicated error variant added to roster, and candidate set removes single-cycle window conjunction: user settlement 2026-10-05 (Q3 "dedicated error variant, add row to roster" and union two passes by slot), verbatim in history; verdicts `research/prompts/m3-batch10-code-r1-main-verification.md` section 2 and section 6, `research/prompts/m3-batch10-code-r2-main-verification.md` section 6; recomputed criterion from `crates/singlefs-harness/src/mount_transparency.rs` 2026-10-05. Candidate set conjunction was unupdated residue after D16 (Publish Semantics) Settled Item 1 regression to oldest valid root in ring (user decision 2026-10-04). No experiment: measured on third-party replica without experiment number.
- Unreadable abandoned root accounting requires credentials to release, pre-quarantine during writable mount, missing records refuse writable mount, selected root read failure regresses to witnessed version: verdicts `research/prompts/m3-c694-r1-main-verification.md` section 2 lines K1, K2 and section 4, `research/prompts/m3-c694-r2-main-verification.md` section 2 lines K1–K3 and section 3 items 1, 2, 6, 7, fork roster `m3-c694-r2-forks.md`; user settlements 2026-10-07 (C694 "implement both C and B", F1 "Option B: refuse writable on missing records", F2 "Option B: implement in all three locations"), verbatim in history. No experiment: measured on third-party replica and models, no experiment number.
- Writable mount pre-quarantine unions newer readable root accounting (`C_NEWER_ROOTS`), unreadable accounting with incomplete records dynamically computes from mapping (`C_MAPPING` narrowed), re-evaluating F1 after F2 regression (`F1_AFTER_F2`), F2 regressing to newest root in interval (`F2_NEWEST`): verdict `research/prompts/m3-batch35-36-code-r1-main-verification.md` section 2 four lines K2 (H1–H4), section 3 items 1, 3 and section 4; user settlement 2026-10-08 "fix all four forms", verbatim in history. C_MAPPING refusal listed on roster, reported separately from C393 refusal: user settlement 2026-10-08 "Option B: core adds error variant", verbatim in history. Newer readable root publish records completely unreadable with accounting unreadable: protects if mapping computes, does not refuse if mapping fails: main agent decision 2026-10-08 (`records/2026-10-01-删除与并发事务.md` batch 39). Four fixes measured on attacker model with zero attack rounds. No experiment: measured on attacker model, re-run by main agent, no experiment number.
- F2 judges fallback by the union of the three readers, stays on the selected version if it cannot fall back, and hands the judgment the records rescanned after the trigger (C755 (recovery and two mount triggers fail differently on rollback)); the ring is rescanned before F1 re-judges (C754 (re-judging missing record after rollback does not rescan ring)); with no injected read faults and system configuration rotation writes landing, F2 cannot reach a publish whose last record is readable but whose root is not (C758 (publish where last entry readable but root unreadable without read fault unmeasured)); reading the last record in this disk's ring during the per-disk check before acquisition does not count as branching on clean shutdown (C499 (Whether clean shutdown mark exists lacks clause)): verdicts `research/prompts/m3-close-rootring-r2-main-verification.md` section 2 Y3 and the `research/prompts/m3-close-rootring-r3-main-verification.md` section 2 cell C755 (recovery and two mount triggers fail differently on rollback); user settlement 2026-10-20 (the three union sentences "add all three sentences", rescan before re-judging "rescan", the F2 argument "add one more test case to pin it", the clean shutdown read "accept, name this read"), original wording in history. No experiment: numbers measured on the three-party legs' replicas, no experiment number assigned.
- When the selected root's instance table is unreadable, roots of the same instance are accepted only if the witnessed version and the selected root share the instance table pointer (C751 (read-only mount bypasses confirmed version when instance table unreadable)): verdict `research/prompts/m3-close-rootring-r3-main-verification.md` section 2 cell for the narrowed form of C751 (read-only mount bypasses confirmed version when instance table unreadable) (both legs independently hit the narrowed form where a same-instance sweep publish swaps the table, reruns matched, not written back) and the Option A row of the section 3 table handed to the user; user settlement 2026-10-20 "Option A: accept only with a shared instance table pointer", original wording in history. No experiment: numbers measured on the three-party legs' models and matched by the main agent's rerun; Option A is zero-round.
- Rollback inode watermark takes solely roots not abandoned by crash recovery (Opt2), retaining cur in-memory term in max, tree ID watermark takes all roots in ring ("Watermark" cell): W3 three verdict rounds `research/prompts/m3-w3-r1-main-verification.md`, `m3-w3-r2-main-verification.md` (section 2 attacker: taking all roots causes abandoned readable high-watermark roots to inflate rollback, W3 historical check false-reporting on valid images), `m3-w3-r3-main-verification.md` (section 2 Opt2 lines: two samplings and forward leg produced zero reissuances or false reports; removing cur in-memory term reissues; cost is 11.6% more valid cross-abandonment reissuances); user settlement 2026-10-08 Y1 "Opt2: take roots not abandoned (Recommended)", verbatim in history. No experiment: measured on third-party model and repo replica prototype, no experiment number.
- Retirement tree follows tombstone tree and pedigree tree, following active version rather than R_old: verdict `research/prompts/m3-retire-r1-main-verification.md` section 2 line K3 (two attacker samplings unhit); user settlement X1 Option A (verbatim in history).
- Verdict `research/prompts/m4-snapshot-r1-main-verification.md` section 3 S8 (first version active snapshot refuses rollback; form when releasing rollback has zero rounds) and section 7 user settlement (Fork 3 "refuse if first version has snapshots", verbatim in history).

**Owed**: C77; C314; C554; C334; C287; C126; C493; C381 — third-party rounds 2 and 3 challenged two failure table discriminators (probe write lacks placement on disk, barrier failures lack defined "failed placement"), user settled "placements enter address space table, scope lists table per failure point", questions 2 and 4 settled (D16 (Publish Semantics) establishes new settled item, failure table alters two-step discriminator), "instance switchover takes in-memory root" settled as switchover re-reading disk to choose root with three corollaries pending (`research/prompts/c381-r2-main-verification.md`, `c381-r3-main-verification.md`); code half (probe write, read verification, instance switchover, read-only transition) moved alongside milestone "overwrite, release, rollback and reuse" closeout table row 16 to subsequent milestone; C458: check turning red for switchover re-reading disk to choose root; C541; C558; C596; C866 (F2 rollback does not tell lost bytes from rejected content).

#### Settled Item 15: Crash After Record Persisted but Before Root Slot Persisted: Recovery Reconstructs Root of That Publish from Record

**Ruling**: **Reconstruct the root of that publish from record: record header adds "new root segment" = tree table unit pointer 86 + central mapping tree root pointer 86 + tree ID watermark 8 + rollback floor F 8 + instance table unit pointer 86 + allocation record tree root pointer 86 + unmount mark 4 = 364 bytes <!-- format-const: JOURNAL_NEW_ROOT_SEGMENT_BYTES = 364 stale=新根段 188 字节|JOURNAL_NEW_ROOT_SEGMENT_BYTES = 188 -->, with 4096 record holding 64 named entries. Applying a record = replacing these seven fields in the selected root with those from the record (instance table pointer, unmount mark, allocation record tree root derived from new root segment rather than copying selected root), requiring no tree semantics. When a publish splits into multiple records, every record carries an identical copy of the new root segment. Settled Item 14 Item 2 "apply if strictly greater than selected root" holds verbatim.**

**Scope**: Governs what recovery applies in cell "record present, root slot absent"; pointer widths follow D19 (Block Pointer Structure & Width Budget) Settled Items 7 / 11. Remaining pointer layer actions upon applying a record (allocator registration, overlay) fall outside this cell, belonging to C284 and D28 (Mount-Time Commitments) Settled Item 2.

**Evidence**:

- E142 (Dry Run of New File in New Pool): After adding new root segment to record header, "applying a record" gained concrete definition on pointer layer, producing differing recovery results across batch of states between ignoring journal and consulting journal — proving journal is load-bearing.
- User settlement, verbatim in history.
- User settlement 2026-10-01: "Each record of multi-record publish carries identical new root segment" incorporated into Settled Item 15 (`research/prompts/m3-inv-b-r1-main-verification.md` section 4 item 3), verbatim in history.
- Third-party verdict `research/prompts/m3-ring-r3-main-verification.md` section 2 lines 1, 2: Application-site instance table pointer, unmount mark, and allocation record tree root changed to derive from new root segment (patching first file and unmount produces roots byte-identical to original writes, attacker verified; dedicated sweep publish crashing with record persisted and root unpersisted causes next writable mount to be refused before number acquisition if instance table pointer merely copied selected root, attacker verified on replica); user settlement 2026-10-02 "revert to Option A", verbatim in history. No experiment: numbers verified on third-party replica, no experiment number.

**Owed**: C284.

#### Settled Item 16: mkfs Writes Instance Number 0, First Writable Mount Takes 1

**Ruling**: **mkfs writes 0, where 0 = "mkfs, no instance yet", not a valid instance; first writable mount per C113 settlement P2 takes max(system configuration, root ring) + 1 = 1, writing into every system configuration (single system configuration slot write, generation +1) before touching any unit. mkfs writes unit write order (0, 0), generation 0 root instance number 0; D18 (Information Carried in Blocks) Settled Item 11 clause "starts from 1, 0 invalid" read as "instances start from 1, 0 reserved for mkfs".**

- **Barrier at number acquisition step**: At least one completed barrier between the two system configuration writes of number acquisition and first non-system-configuration write of this instance; opening warm-up barrier on first-mount path suffices without issuing additional barrier; all writers of this instance forbidden from issuing units, records, or roots before barrier completes; error on that barrier on any disk ⇒ number acquisition fails all-or-nothing, forbidden from proceeding by merely reissuing barrier. Acquisition max reading and rollback write number written in D18 (Information Carried in Blocks) Settled Item 11, generation increments per disk by +1 per D22 (Synthesizing Unit Atomicity) Settled Item 16.

**Scope**: Governs starting instance number and ordering of acquisition step; instance table row writing belongs to D18 (Information Carried in Blocks) Settled Item 11. ⚠️ If row writing or warm-up publishes fail due to block device errors post-acquisition, newly acquired generation written into system configuration **does not roll back** — rollback covers solely acquisition's own system configuration writes (D18 (Information Carried in Blocks) Settled Item 11); burnt generation abandoned, next writable mount taking max + 1, burnt generation gaining row per D18 (Information Carried in Blocks) Settled Item 11 row-writing rules. Known behavior, not treated as defect.

**Evidence**:

- No experiment: starting point of instance number is encoding convention; discriminative power of acquisition barrier resides in crash-point replay rather than count experiments.
- Third-party: `research/prompts/c322-r3-main-verification.md` section 5 (barrier at acquisition step, settled by user after three third-party rounds); instance number starting point settled by user. Verbatim in history.
- User settlement 2026-09-23: Failure during row-writing or warm-up post-acquisition does not roll back, accepted as known behavior (milestone "overwrite, release, rollback and reuse" closeout table row 57), verbatim in history.

**Owed**: None.

#### Settled Item 17: Field Table for 56-Byte Named Entry

**Ruling**: **One entry per unit (12 entries for new-pool file creation): position entry 14 × 2 (offset 0) + unit type tag 1 (28) + birth tree 8 (29) + birth txg 8 (37) + key tail segment 10 (45: Code 1 write sequence; Code 2 / Code 3 instance number 4 + birth sequence 4 + zero padding 2) + flags 1 (55) = 56 <!-- format-const: JOURNAL_NAMED_ENTRY_BYTES = 56 -->; no additional payload CRC carried (position entries each carry 4-byte checksum). Replay constructs mapping key directly from named entry.**

- **Every item named by record carries its own checksum**: D4 (Checksum Location) settled checksums inlined into parent pointers, whereas target named by record has not yet had parent issued when record persists ⇒ record temporarily acts as parent, otherwise units lack integrity credentials during replay; recovery replay must verify named units individually before applying (Settled Item 14 Rule 4).
- **When publish splits into N transactions and N records, commit-endogenous blocks shared by publish (extent tree nodes, inode leaf containers and root, allocation record tree, accounting tree, central mapping tree, tree table units) are named solely in final record**, omitted from earlier records to avoid duplicate naming.
- **When shared commit-endogenous blocks exceed capacity of final record (4096 record holds 64 entries), last entry spans additional records**: multiple records written sequentially starting from last transaction record, filling each record before opening next, with only true final record carrying "last record of this publish" flag; recovery recognizes publish boundary via this flag, omitting entire publish if flagged record unread (Settled Item 14 Rule 5). Flag placed in "padding 1" byte of record header ten fields (renamed "record flags 1"), bit 0 = last record of publish; single-record publishes (including empty publish records) also write 1 in bit 0 — altering bytes of new-pool file creation (w1, w4, t13 records alter this byte and header checksum [46, 50); w4, t13 reverse chain [91, 95) updates accordingly, verified byte-for-byte in E142 (Dry Run of New File in New Pool) G26). Settled by user 2026-09-24.

**Scope**: Governs named entry bytes; position entry composition belongs to D19 (Block Pointer Structure & Width Budget) Settled Items 4 / 11, mapping key format belongs to D19 (Block Pointer Structure & Width Budget) Settled Items 6 / 10. ⚠️ When shared commit-endogenous blocks named only in final record, crashing before final record persists leaves units unnamed by any record — recovery reclamation path must recognize this cell, covered by crash-point replay small workloads (C491).

**Evidence**:

- E16 (Role of Journal: WAL vs Intent Log): Parent of object named by record unwritten to disk at that moment, forcing record to act as parent — necessitating "every named item carries checksum".
- E77 (Persistence Ordering of Publications): Named entries carrying checksums and verified individually before application is quid pro quo permitting removal of barrier between units and records.
- User settlement, verbatim in history.
- User settlement 2026-09-23: Shared commit-endogenous blocks named solely in final record, verbatim in history.
- User settlement 2026-09-24: Last record spans multiple records, final record carries flag (verbatim in history; third-party `research/prompts/m2-treesplit-r1-main-verification.md` T6 (Whether refusal is maintained depends on pool history), pre-settlement text applies half publish, flagged cell 0 / 0).

**Owed**: C376.

#### Settled Item 18: In-Flight Record Count Upper Bound Value and Semantics, Tail Field Encoding

**Ruling**: In-flight record count upper bound = ring slot count ÷ F (`F` is safety factor from I-8.1 (Ring Geometry Sufficiently Large)), default ring 196608 ÷ 3 = **65536**, written into system configuration; tail 8 bytes stores 48-bit counter of jsn, utilizing two-slot rotation of system configuration for redundancy without dedicated tail slot; record n lands at ring offset `(counter − 1) mod slot_count × 4096`.

- **Semantics**: Value denotes that **prefix of single recovery replay contains at most N records**. Upon reading record failing checksum, recovery **stops immediately at discontinuity** — halting before corrupt record without inquiring whether "torn" or "corrupt", and without refusing read-only mount; writable mount evaluated per C554 Option B-Config-Cont (Settled Item 14): refuse writable if system configuration witnessed publish newer than break point and reread evaluates true. Differs from XFS rule "bad checksum within last N records treated as tear, beyond treated as corruption refusing mount".

**Scope**: ⚠️ Cost: Genuine "mid-disk corruption" and "tail tear" evaluated identically in first version, mounting normally while omitting records following corruption. First version serial commit (D13 (Verification Strategy) Settled Item 2) has N constantly 1 in-flight ⇒ XFS requirement "N = concurrent written records" and "ring slot count ÷ F" represent different dimensional quantities, incapable of fitting within single field.

**Evidence**:

- No experiment: value derived arithmetically from I-8.1 (Ring Geometry Sufficiently Large) (ring slot count ÷ F), semantics represent measurement basis ruling — stopping at discontinuity already criterion of I-8.3 (Strictly Continuous Replay Prefix), this ruling merely merging "failed checksum" into same exit without new branches.
- User modal settlement without third-party argument, overturnable, verbatim in history.

**Owed**: None.

#### Settled Item 19: Transaction Number and Commit Marker for Empty Publish Records, previous_hash, Ring Length

**Ruling**:

- **① Empty publish record**: Transaction number 0 reserved for records carrying no transaction, commit marker writes 1; excluded from instance table W max, excluded from "discard uncommitted tail" judgment.
- **② `previous_hash`** = CRC32C(**logical previous record header within this instance**, with `header_csum` 32 bytes participating as zeroes); **first record written in instance constant 0**; write path **reads zero disk blocks**. Reverse chain is **per-instance** — I-8.3 (Strictly Continuous Replay Prefix) verbatim "halt across instance boundaries", replay never connecting across instances with boundary blocked by instance number; taking "record header in preceding ring slot" forces write path to read preceding slot before every write, buying merely "connecting across instances" unused by any judgment. Byte table w1 row writing 0 justified by "first in instance", not "no predecessor in ring".
- **③ Ring length**: Ring length is **mkfs parameter**, **default 768 MiB**; constrained by **ring ≤ device capacity ÷ 4**, exceeding bound **refuses mkfs**. System configuration retains solely single field "ring length (bytes)", storing no capacity or ratio. **Mount effective `T_dirty` = min(`T_dirty` field, ring length ÷ F)** — system configuration still stores 2 GiB for `T_dirty`, effective value under default 768 MiB ring = min(2 GiB, 256 MiB) = 256 MiB. Unit area start slot follows ring length, defaulting to 784 MiB (slot 50176). Clamping `T_dirty` is direct corollary of I-8.1 (Ring Geometry Sufficiently Large) — if ring cannot hold `F ×` worst-case occupancy, dirty bytes must drop to prevent publish failures.

**Scope**: ③ "parameter + upper bound" ensures 768 MiB occupies small percentage of large disks while small images scale per parameter; `ring ≤ capacity ÷ 4` prevents ring from consuming entire disk; ⚠️ corollary: `.claude/kb/layout/01-first-txn.md` envisioned image is **4 GiB** (1 GiB image cannot accommodate 768 MiB ring under constraint). Implementation: `crates/singlefs-core/src/make_filesystem.rs` accepts ring length parameter, refusing values exceeding smallest disk capacity ÷ 4, defaulting to `JOURNAL_RING_DEFAULT_BYTES` in `crates/singlefs-format/src/lib.rs`. Mount reader side evaluates ring length: in-flight upper bound (ring slots ÷ F, Settled Item 18) ≥ 1, with ring end not exceeding compile-time unit area start (784 MiB, `UNIT_AREA_START_SLOT` and `JOURNAL_RING_START_SLOT` in `crates/singlefs-format/src/lib.rs`); violations refuse mount whole pool (audit A3a: `SystemConfigurationValueOutsideWhatThisReaderAccepts::JournalRingBytesOutsideTheSupportedRange` in `crates/singlefs-core/src/recovery.rs`).

**Evidence**:

- E143 (Journal Accounting Under One Transaction One Unit): Per split discipline, full `T_dirty` window requires ring 4× larger than envisioned 64 MiB ring, necessitating ring ≥ 768 MiB (F = 3) or reduced `T_dirty` — origin of default ring length and `T_dirty` clamping.
- ①, ②, ③ settled by user modal without third-party argument, overturnable; recommended by main agent, verbatim in history.

**Owed**: None.

#### Settled Item 20: Records Carry Pointer Layer Target State, Allocation Precedes Journal

**Ruling**: **Journal records carry "target state of pointer layer", not logical intent, and not full physical redo.** Units are written via copy-on-write to their respective final placements, carrying their own checksums (the class where "copy-on-write + D4 (Checksum Location) suffices" in D22 (Synthesizing Unit Atomicity) Settled Item 18); the journal records solely "which subtree roots moved to which blocks + new root".

- **Produces hard ordering on write path: allocation precedes journal.** Journal records are not "the entirety of the transaction", but "the **publish instruction for entities already persisted to disk**".
- Criteria:

  | Criterion | Logical Intent | Full Physical Redo | **Pointer Layer Target State** |
  |---|---|---|---|
  | Idempotence (I-4.4 (Intent Only Describes Target State) "describes only target state") | Idempotent only when written as target state | Naturally idempotent | **Naturally idempotent** |
  | Does replay require allocator | **Yes** | No | **No** |
  | Record size (D16 (Publish Semantics) "small, sequential writes") | Small | **Large** (must carry all bytes) | **Small** |
  | What record checker (D13 (Verification Strategy) Settled Item 7) needs to verify a record | Self-implement tree semantics + accounting + allocation legitimacy (verification changes from syntax to semantics) | Byte-by-byte comparison | **Format parsing + checksum + root ring selection of newest** |

- **First rationale rejecting logical intent (holds independently)**: O2 (Independent Parser + Checker) is explicitly forbidden from calling any crate code under test (D13 (Verification Strategy)), whereas replaying logical intent requires re-implementing allocator + tree operations + checksums + accounting, **effectively amounting to a second file system**. Second rationale (indeterminate placement degrades verification) is a closure of the first rationale, not an independent rationale (Settled Item 5).
- Record format and verification mechanisms of D13 (Verification Strategy) hold each other hostage: feasibility of verification mechanisms dictates record format, and record format dictates output grade of verification mechanisms; both must be settled together.

**Scope**: Governs record contents, and precedence between allocation and logging; which fields are swapped on pointer layer when applying a record is Settled Item 15, remainder belongs to C284; hard requirement of replay idempotence is in Settled Item 3.

**Evidence**:

- No experiment: direction deduced from 2026-08-28 third-party argument across four criteria: idempotence, allocator requirement in replay, record size, and verifier prerequisites (main agent ruling); all four represent structural properties without measurable controls; ZFS ZIL avoids independent implementation cost by re-invoking main write path, serving as clue rather than evidence, in history.

**Owed**: C284.

#### Settled Item 21: Generation Number: Record-Granularity Global Monotonic Number, Not D5 (Snapshot / Space Accounting Mechanism) txg

**Ruling**: **Record header requires two counter fields with distinct duties, which cannot be merged**:

| Field | Granularity | Purpose |
|---|---|---|
| `jsn` | **Per record**, globally monotonic, 10 bytes (instance number 32 bits + counter 48 bits, Settled Item 9) | Generation number required by ② of D22 (Synthesizing Unit Atomicity); prefix judgment relies on it |
| `checkpoint_txg` | Per checkpoint | Indicates which checkpoint it belongs to, used for birth/death accounting in D5 (Snapshot / Space Accounting Mechanism) |

- Rationale: D5 (Snapshot / Space Accounting Mechanism) is "one txg per checkpoint" ⇒ multiple journal records within single checkpoint window **share identical txg**, indistinguishable in sequence, rendering "generation number actually checked" required by D22 (Synthesizing Unit Atomicity) meaningless.
- ⚠️ Selecting non-wrapping width for jsn **buys "eliminating wrap-around comparisons", not new capabilities**: jbd2 tid is 32-bit and wraps, relying on signed difference comparison `tid_gt`; wrap-around comparison is precisely where independent implementations diverge most easily.
- ⚠️ **"Using txg eliminates cycle" holds only for axis ①**, see orthogonal axes table in D22 (Synthesizing Unit Atomicity).

**Scope**: Governs duties of two counters; primary ordering between them in watermark and root selection belongs to Settled Item 14 Item 2; "each publish increments checkpoint_txg by 1" belongs to D16 (Publish Semantics) Settled Item 6.

**Evidence**:

- No experiment: division of responsibilities deduced directly from D5 (Snapshot / Space Accounting Mechanism) granularity, following definitional derivation without measurable quantities.

**Owed**: None (C85 cleared 2026-10-11, merged into C88, see owed checks cleared table).

#### Settled Item 22: Prefix Judgment: Strictly Continuous, Discontinuity Halts Immediately

**Ruling**: **Prefix criterion is `jsn == expected`, stopping immediately upon mismatch.** Filtering solely with "`jsn` > last applied" is **insufficient**: if holes exist in sequence (5849 succeeded, 5850 unwritten, 5851 succeeded), ">" erroneously connects 5851 after 5849, **violating causal ordering**. Also requires an **explicit in-flight record count upper bound** (value and semantics in Settled Item 18). Complete prefix rules (five rules) reside in Settled Item 14.

**Scope**: ⚠️ Discontinuity stop is correct for **that specific** recovery (prefix halts at 5849); it alone cannot block **remnants of previous timeline**: when resuming writes post-recovery, slot 5851 still holds record from previous timeline, **with good checksum and `jsn` exactly matching expected** ⇒ recovery after second crash connects and replays it. Under slot mapping tested in D23 (Journal Role & Format) (`slot = jsn % slot_count`), this occurs constructively; decoupling placement from sequence degrades this to coincidence, which cannot rescue. Existing three gates (I-8.3 (Strictly Continuous Replay Prefix), in-flight upper bound, identical `checkpoint_txg` in same window) do not trigger; tail advancement is no solution (tail advances only after checkpoint persists, remnants never applied ⇒ `tail ≤ min(remnant jsn)` holds constant); replay idempotence cannot rescue (being reflexive: "replaying **same record** twice yields identical result to once", whereas remnant is **different record**, whose named units might have been reallocated by new timeline post-abandonment). Blocked by Settled Items 8 / 9 / 10: reverse chain, sequence high bits carrying instance number, chain value covering instance number; physically invalidating remnant slots during recovery requires in-place overwriting, which fails on zoned devices.

**Evidence**:

- No experiment: ">" incorrectly connects records across holes, serving as definitional counterexample without measurement; remnant experiment in Settled Item 8 evidence.

**Owed**: C42.

#### Settled Item 23: How Each of the Three Deadlocks Is Broken

**Ruling**: Of three journal-related deadlocks, two land in format and cannot be retrofitted post-hoc:

| # | Cycle | For This Project |
|---|---|---|
| 1 | journal full → checkpoint → issue root → **whether issuing root writes journal** | D16 (Publish Semantics) clause "not bypassing journal" takes Reading Option A (Settled Item 25): issuing root itself is not a journal append |
| 2 | **Unique to COW**: checkpoint → copy-on-write write new node → needs allocation → disk full → cannot issue root | **Reserve pool cures this**, `reserve ≥ ckpt_cost` suffices (Settled Item 24). ⚠️ **K of D22 (Synthesizing Unit Atomicity) exacerbates it**: when disk is nearly full, space most critical for rescue is locked by K ⇒ **K must be runtime policy, not format constant** |
| 3 | Single operation larger than ring | Requires equivalent geometric invariant: **worst-case journal occupancy of any operation ≤ ring size / F, F ≥ 2** ([invariants/03-journal.md](../../../crates/singlefs-checker/invariants/03-journal.md) I-8.1 (Ring Geometry Sufficiently Large)), calculated at mkfs, verified at mount |

- **Deadlock 3 yields dependency**: **D8 (Core Index Structure) intent batching mechanism is not "convenience for deleting large files", but necessary condition for journal-full deadlock avoidance** — operations exceeding budget must split into multiple individually legal transactions.

**Scope**: Governs breaking these three cycles; reserve pool and admission separation in Settled Item 24, Reading Option A in Settled Item 25, ring geometry values in Settled Items 18 / 19. I-8.1 (Ring Geometry Sufficiently Large) "occupancy" is spatial dimension, allowing transaction to span multiple records (Settled Item 7). I-8.1 checker implemented (pool-level checker `judge_journal_ring_geometry_fields_agree`, core recovery simultaneously checks three equalities, `crates/singlefs-core/src/recovery.rs`). Deadlock 2 unique to copy-on-write: in-place jbd2 / XFS writing home requires zero new space, lacking this cycle.

**Evidence**:

- E23 (Journal Geometry): Fixed-length ring has deadlock 3, requiring geometric invariant; ring too small causes forced checkpoints to rise cliff-like; chaining lacks deadlock 3, but requests blocks from allocator per journal block, entering deadlock 2 cycle.
- E19 (Spurious ENOSPC Under Defer Window): Reserve pool exactly equal to single checkpoint cost drops checkpoint lockups to 0 — deadlock 2 broken by reserve pool.

**Owed**: None.

#### Settled Item 24: Deadlock 2 and Spurious ENOSPC Are Distinct Faults Requiring Separate Solutions

**Ruling**:

| Failure Mode | What Cures It |
|---|---|
| **Checkpoint lockup** (D23 (Journal Role & Format) Deadlock 2) | **Reserve pool**, and `reserve ≥ ckpt_cost` suffices; ckpt_cost in this project is recomputed per D28 (Mount-Time Commitments) Settled Item 4 (sum of record tree heights + accounting nodes), without establishing constant |
| **Spurious ENOSPC** | **Only admission**. Reserve pool actually exacerbates it |

- **Mechanism**: Reserve pool establishes floor for checkpoints, acting as pure tax on **ordinary allocation** — provided reserve pool is already large enough to prevent lockups; outside this condition sign reverses, as lockups prevent space reclamation, causing small reserve pools to reject more. Spurious ENOSPC stems from "defer window and root ring K locking up more than remaining free space", **falling outside reserve pool scope**.
- **Spurious ENOSPC resolution is calculable admission rule**:

  > **Remaining space must be > churn per checkpoint × (defer generations + 1) + single checkpoint overhead**

  ⇒ Disk cannot be filled beyond `capacity − churn × (defer + 1) − checkpoint overhead`. Isomorphic to D3 (Space Allocation) Settled Item 15 "space reservation takes admission control form" — **"worst case" must include portion locked by defer window and K.**

**Scope**: How reserve pool enters admission inequality and whether it counts as mount commitment belongs to D28 (Mount-Time Commitments) Settled Item 4; authoritative form of admission inequality in D28 (Mount-Time Commitments) Settled Item 1. ⚠️ **Churn is variable in real workloads**, modeled as constant; estimation is responsibility of real-machine stress testing.

**Evidence**:

- E19 (Spurious ENOSPC Under Defer Window): Reservation exactly equal to single checkpoint cost reduces lockup to 0; further increasing reservation merely raises spurious ENOSPC by constant step; admission rule predictions match measurements cell-by-cell.
- E25 (Reserve Pool Size for Ring vs Chain): "Reserve pool is pure tax on normal allocation" holds only when reserve pool is large enough to prevent lockup; within lockup region small reserve pools reject more.

**Owed**: C83.

#### Settled Item 25: D16 (Publish Semantics) "Not Bypassing Journal" Takes Reading Option A

**Ruling**: The sentence in D16 (Publish Semantics):

> "No operation may bypass the journal to issue a root directly."

**takes Reading Option A**:

| Interpretation | Meaning | Result |
|---|---|---|
| **Option A (Adopted)** | Any **semantic effect** ultimately reflected in root must first pass through journal; **the action of issuing root itself is not a journal append** | Breaks Deadlock 1 |
| Option B (Literal) | Issuing root must also pass through journal ⇒ issuing root requires appending record ⇒ cannot issue root when journal is full | **Cycle closes, deadlock** |

⇒ Compatible with D22 (Synthesizing Unit Atomicity): **Root ring consists of fixed K slots outside journal; issuing root = overwriting a slot in root ring, not an append.** Checkpoint is not an "operation", but part of journal mechanism itself; ordinary user operations must still pass through journal.

**Scope**: Governs solely how that sentence in D16 (Publish Semantics) is interpreted; remainder of publish semantics belongs to D16 (Publish Semantics).

**Evidence**:

- No experiment: ruling on phrasing interpretation; one interpretation directly closes deadlock cycle, leaving no measurable quantity; jbd2 / XFS solution (space release path consumes zero log space) compatible with Reading Option A, serving as clue, in history.

**Owed**: None.

#### Settled Item 26: Two Parsing Disciplines for Record Header

**Ruling**:

1. **Byte range covered by checksum must be an explicit on-disk constant, never deduced from language's `sizeof(struct)`.** Independent parsers will not accidentally replicate identical `sizeof`, representing precisely the error class checkers catch that single implementations miss.
2. **Record self-describing length must undergo boundary check before being used in checksum computation.**

Reverse chain rule (enabling record checker to re-evaluate identical prefix boundary using completely independent traversal from forward scan) is Settled Item 8.

**Scope**: Governs parsing sequence and constant origin for journal record header; coverage scope itself (entire `[0, 4096)`) in Settled Item 13; Item B of Settled Item 1 (self-describing length boundary-checked before checksum) is Rule 2.

**Evidence**:

- No experiment: both represent disciplines derived from deficiencies in existing production implementations (XFS used `sizeof`, causing i386 and other architectures to compute differing CRCs, saddled with `XLOG_REC_SIZE` / `XLOG_REC_SIZE_OTHER` constants and compatibility scars of two-pass checks; ZFS ZILOG2 reads `zc_nused` from block to determine checksum scope, performing `if (size < nused) return ECKSUM` before reading), constraining parsing order and constant origin without measurable quantities.

**Owed**: None.

```

**出处 `.claude/kb/decisions/23-journal-role-format.en.md:692-695`（整段抄，未转述）**

```markdown
## History

D23 (Journal Role & Format) historical entries are collected in [decisions-history.md](../decisions-history.md).

```

**出处 `.claude/kb/decisions/23-ジャーナルの役割と形式.ja.md:1-15`（整段抄，未转述）**

```markdown
## D23 (ジャーナルの役割と形式) — 確定

[中文](23-journal的角色与格式.md) · [English](23-journal-role-format.en.md) · **日本語**

<!-- doc-lint:not-numbers CRC-64 SSE4.2 CRC32C SHA-256 CJ2 -->

D23 (ジャーナルの役割と形式) はジャーナルが何であるか、何を書き込むか、クラッシュ後にそれをどう利用するかを管轄する：レコードが担持するものと割り当ての先行順序（確定項目 20）、毎回の fsync で何を書き込むか（確定項目 1）、リングの幾何・レコードサイズおよびインフライト上限（確定項目 2 / 12 / 18 / 19）、tail（確定項目 3 / 18）、レコードヘッダおよび指名項目のバイト列（確定項目 4 / 7 / 8 / 9 / 10 / 11 / 13 / 15 / 17 / 21）、プレフィックス判定および再生下限（確定項目 14 / 22）、管理者ロールバック・障害処理およびインスタンス切り替え（確定項目 14 / 16）、ジャーナルに関連する3つのデッドロックとリザーブプール（確定項目 23 / 24 / 25）、レコードヘッダのパース規律（確定項目 26）。対象外とする事項：パブリッシュの永続順序・チェックポイントのトリガーおよびウォームアップは D16 (パブリッシュセマンティクス) に帰属；インスタンス表の行形式と行書き込み規則は D18 (ブロックに含める情報) 確定項目 11；ルートレコードとルートリングは D22 (ユニット原子性の合成手法)；リザーブプールと切り替え予約が受入不等式にどう算入されるかは D28 (マウント時コミットメント量)；checker とレコード照合器の定義は D13 (検証ルート) 確定項目 6 / 7；対象ワークロードは D25 (対象ワークロードの優先順位) に帰属する。

<!-- doc-lint:registry name-col=2 -->

| 番号 | 略称 |
|---|---|
| T6 | 拒絶維持時に拒絶するか否かはプールの履歴に依存する |
| L8 | 並行線3レコード跨ぎクラッシュポイント再生ストリーム |

```

**出处 `.claude/kb/decisions/23-ジャーナルの役割と形式.ja.md:16-691`（整段抄，未转述）**

```markdown
### 確定項目

| # | 項目 | 確定内容 |
|---|---|---|
| 1 | **ジャーナルの役割自体** | 案Aを採用：毎回の fsync でダーティリーフ + 全祖先 + ルートスロット + 1件のレコードを書き込み、祖先は遅延させない；第1軸と第2軸は非直交 **状態：確定。** **進捗：実装済み。** |
| 2 | **ジャーナルは固定長リングかチェーン式か** | 固定長リングを採用；決定的な理由は、チェーン式がデッドロック3の判定基準を静的から動的へ変えてしまうこと **状態：確定。** **進捗：実装済み。** |
| 3 | **tail の配置場所・どれが権威か・いつ永続化するか** | システム構成スロットに配置、2以上のスロットでローテーション、毎回のパブリッシュに伴い1回書き込み；リカバリはtailを先に信用してはならず、再生は冪等でなければならず、リカバリは幾何を救済できない **状態：確定。** **進捗：実装済み。** |
| 4 | **レコードヘッダを単一のアトミックユニット内に収められるか** | 可能；ヘッダ487バイト = 10フィールド78 + 4件の増分21（トランザクション境界9、今回のパブリッシュ内シーケンス番号4、逆方向チェーン4、ペイロードチェックサム4）+ 新ルートセグメント364 + fsid 8 + MAC 16 **状態：確定。** **進捗：実装済み。** |
| 5 | **O2 (独立パーサー + checker) とは実際何であるか** | 単項述語；論理インテントログを却下した第2の理由は第1の理由の穴塞ぎとして書き直す；逆方向判定基準の主語はレコード照合器 **状態：確定。** **進捗：対象なし。** |
| 6 | **対象ワークロードの優先順位** | ジャーナルのラインから除外、D25 (対象ワークロードの優先順位) を参照；ジャーナルのどの項目もブロックしない **状態：確定。** **進捗：対象なし。** |
| 7 | **1つのトランザクションが正確に1件のジャーナルレコードを生成するか** | 複数レコードに跨がることが可能、レコードヘッダにトランザクション番号とコミットマーカーを保持；再生時はコミットマーカーが現れていない末尾を破棄 **状態：確定。** **進捗：実装済み。** |
| 8 | **レコード内でタイムラインを識別するフィールド** | レコードヘッダに逆方向チェーン（直前レコードのhash）を付与、幅32ビット **状態：確定。** **進捗：実装済み。** |
| 9 | **シーケンス番号とインスタンス番号のビット幅配分** | jsn = インスタンス番号32ビット + カウンタ48ビット、2フィールド合計10バイト；番号のロールバックは禁止 **状態：確定。** **進捗：実装済み。** |
| 10 | **`previous_hash` がカバーするフィールド** | 直前レコードのレコードヘッダ全体をカバーし、その中に jsn を含み、したがってインスタンス番号も含む **状態：確定。** **進捗：未実装。** |
| 11 | **逆方向チェーンのハッシュアルゴリズム** | 完全な32ビット CRC32C を採用、暗号文チェックサムと同一の関数 **状態：確定。** **進捗：実装済み。** |
| 12 | **レコードサイズ** | 4 KiB 固定長、システム構成に書き込み **状態：確定。** **進捗：実装済み。** |
| 13 | **ペイロードの完全性を何に依存するか** | `header_csum` がレコード全体 [0, 4096) をカバー；`payload_csum` を保持し、指名項目配列をカバー；いずれか一方が不合格なら当該レコードは無効 **状態：確定。** **進捗：実装済み。** |
| 14 | **再生の下限、管理者ロールバック、失敗処理およびインスタンス切り替え** | (インスタンス番号, checkpoint_txg) が選択されたルートより厳密に大きいレコードのみを適用；プレフィックス基準は5条；切り替えは1回のリカバリとして実行、管理者ロールバックはマウント中の1回の前方パブリッシュ **状態：確定。** **進捗：未実装。** |
| 15 | **レコード永続化後かつルートスロット永続化前にクラッシュした場合にリカバリが適用するもの** | レコードから当該パブリッシュのルートを再構築：レコードヘッダに364バイトの新ルートセグメントを保持、1レコードの適用 = 選択ルートの7フィールド置換；1回のパブリッシュの各レコードは同一の複製を保持 **状態：確定。** **進捗：実装済み。** |
| 16 | **mkfs および初回書き込み可能マウントのインスタンス番号** | mkfs は 0 を書き込み（有効インスタンスではない）、初回書き込み可能マウントは 1 を取得、すべてのシステム構成に書き込み完了したバリアを通過した後に初めてユニットを操作 **状態：確定。** **進捗：実装済み。** |
| 17 | **指名項目56バイトのフィールド表** | 1項目につき1ユニット：位置エントリ 14 × 2 + 種別 1 + 誕生ツリー 8 + 誕生 txg 8 + key末尾セグメント 10 + flags 1 **状態：確定。** **進捗：実装済み。** |
| 18 | **インフライトレコード数上限と tail のエンコーディング** | 上限 = リングスロット数 ÷ F、1回のリカバリ再生におけるプレフィックスは最大Nレコードを意味；不正なチェックサムは一律でチェーン切断して停止；tail はカウンタを格納 **状態：確定。** **進捗：実装済み。** |
| 19 | **空パブリッシュレコード、`previous_hash`、リング長** | トランザクション番号0は空パブリッシュ用に予約；チェーンは現インスタンス内の論理的直前レコードに基づき計算；リング長は mkfs パラメータ、デフォルト 768 MiB、≤ 容量 ÷ 4 **状態：確定。** **進捗：実装済み。** |
| 20 | **レコードが担持するもの、割り当てと記録の先行順序** | レコードはポインタ層の目標状態を担持し、論理インテントでも完全な物理REDOでもない；割り当てはジャーナルに先行する **状態：確定。** **進捗：実装済み。** |
| 21 | **レコードヘッダの2つのカウンタフィールド** | jsn はレコードごとにグローバル単調増加、プレフィックス判定はこれに依存；checkpoint_txg は各チェックポイントに1つ；両者は統合不可 **状態：確定。** **進捗：実装済み。** |
| 22 | **プレフィックス判定** | `jsn == expected`、不一致で即時停止；明示的なインフライトレコード数上限を別途設ける **状態：確定。** **進捗：実装済み。** |
| 23 | **3つのデッドロックをそれぞれどう打破するか** | デッドロック1は解釈案Aにより、デッドロック2はリザーブプールにより、デッドロック3は幾何的不変条件により打破：最悪占有量 ≤ リングサイズ / F、F ≥ 2 **状態：確定。** **進捗：実装済み。** |
| 24 | **デッドロック2と疑似ENOSPCは別個の障害** | 行き詰まりはリザーブプールで対処、reserve ≥ ckpt_cost で十分；疑似ENOSPCは受入のみが解決でき、リザーブプールはかえってそれを悪化させる **状態：確定。** **進捗：実装済み。** |
| 25 | **D16 (パブリッシュセマンティクス)「ジャーナルを迂回しない」の解釈案選択** | 解釈案Aを採用：セマンティクス上の効果はジャーナルを経由必須；ルート発行自体はジャーナルの追記ではなく、ルートリングの1スロット上書き **状態：確定。** **進捗：実装済み。** |
| 26 | **レコードヘッダの2つのパース規律** | チェックサムのカバー範囲は明示的なディスク上定数とし、sizeofから導出しない；自己記述長はチェックサム計算前に境界チェックを実施 **状態：確定。** **進捗：実装済み。** |

#### 確定項目 1：ジャーナルの役割自体

**確定内容**：**案Aを採用：毎回の fsync でダーティリーフ + 全祖先 + ルートスロット + 1件のレコードを書き込む。祖先は遅延させない。** ダーティリーフは fsync 対象ファイルのものにとどまらず、全ダーティリーフを指す：fsync = 1回のフル実行パブリッシュ。

- **2つの軸は非直交**：第1軸（毎回の fsync でルートを発行するか否か）で「発行する」を選択した後、第2軸（祖先のディスク書き込みを遅延させるか否か）には合法的な値が1つしか残らない。**祖先の遅延 ⇒ 新ルートが自己矛盾を起こす ⇒ 今回の fsync を反映したルートを発行不能。** コピーオンライトの下で、今回の fsync のリーフをルートから到達可能にするには、スパイン全体がすでにディスク上に存在しなければならない；そして「自己矛盾がないこと」は [invariants/02-根环.md](../../../crates/singlefs-checker/invariants/02-根环.md) I-7 (ルートリング) クラスの要件である（最大世代のルートから再帰的に検証して権威態を走破可能）。2×2の組み合わせのうち実現可能なのは3マスのみ：

  | | 祖先を遅延させない | 祖先を遅延させる |
  |---|---|---|
  | **毎回の fsync でルート発行** | **案A** | **存在しない** |
  | チェックポイントでのみルート発行 | `wal_full` | 案B / 案C |

  ⇒ **第1軸で確定済みの「毎回の fsync でルート発行」単独で、案Bと案Cの双方が同時に却下される**。かつ第1軸の4つの根拠（リングを切断できない / マウント時再生パスが1つ増える / ジャーナルが検証チェーンに参入する / バリアが1つも削減されない）は案Bに対しても**逐条そのまま成立**し、再論証を要しない。案C（E22 (レコードで指名した祖先の配置場所) の形態）も同様に祖先の内容を書き込まず、第1軸が再開された場合にのみ意味を持つ。
- **案A下で redo は空**であるが、「案A下でリカバリが無動作」を意味しない：リカバリ期間には依然として「未完了インテントの継続実行」という正規パスが存在する（[checks-owed.md](../checks-owed.md) C1）。
- **レコードヘッダの種別フィールドは必須であり、将来第2軸が再開されるか否かとは無関係である。** 種別フィールドの登録表（登録位 D22 (ユニット原子性の合成手法) 確定項目 17）：0 無効（破損と判定）、1 通常レコード；新規種別は追記のみでコードの再利用は不可、未知の種別はA条に従いマウントを拒絶。これを「第2軸のために残したオプション」と記述すると、コストを支払って第2軸の可逆性を購入したかのように誤認される —— **これによって可逆性は一切購入されていない。**
- **真に第2軸を可逆とするのは以下の4項目であり、1つも欠かせない**：

  | # | 予約内容 | 理由 |
  |---|---|---|
  | A | レコードヘッダの**種別フィールド** + **未知の種別 ⇒ マウント拒絶**、かつ本ポリシーを**システム構成 incompat ビット**が担持 | フィールドだけありポリシーが「認識不能レコードをスキップ」の場合、失敗モードは**パブリッシュ命令のサイレントな脱落**となり、フィールドがないより悪化 |
  | B | レコードの**自己記述長**、かつ**境界チェックを経てからチェックサム計算** | 種別フィールドは「これが何であるか」を示すのみで「長さ」は示さない。案Cのレコード長はツリー高に比例し、案Aのサイズで固定した予算に収まらない |
  | C | **ジャーナル幾何をシステム構成に含める**（リングサイズ、最大レコードバイト数、インフライトレコード数上限）、**コード上の定数とすることは禁止** | 第2軸の変更 ⇒ 最悪レコードサイズが変化 ⇒ デッドロック3の幾何的不変条件も追従変更が必要。コードに固定するとファイルシステム再作成しかなくなる |
  | D | 項目のサイズは**レコード種別によって決定** | D9 (暗号化) は MAC を128ビット満額で切り捨てないことを確定済み、案Cの項目はさらに広い |

  ⚠️ **AとBはもともと未済の検査であった**（Bは確定項目 26 の第2条）；これらを「第2軸のために支払ったコスト」として計上するのは二重記帳である。
- **第2軸を再開するための判定基準。** ⚠️ **これは設計時の判定基準であり、実行時のスイッチではない。** `.claude/rules/fs-design.md` ハード要件5「分岐変数は操作途中で変更してはならない」——「遅延させるか否か」をワークロードに応じて切り替えるランタイム分岐にすると、単一操作の途中でクラッシュセマンティクスが変化し、即座に規則違反となる。**第2軸は全体として一括切り替えしか許されず、incompat ビットを経由しなければならない。**

<!-- doc-lint:registry name-col=2 -->

| ゲート | 略称 | 判定対象 | 計測可能か |
|---|---|---|---|
| G23.0 | 轴一被重开 | 第1軸が再開されたか（否なら以降の条項は参照不要） | — |
| G23.1 | 祖先链占比达标 | 単一ストリーム同期書き込み（QD=1、単一スレッド）の fsync サービス時間において、「祖先チェーン書き込み + ルートスロット」の割合 ≥ X%、**X は実験登録時に宣言、事後決定は禁止** | **不可**、E16 (ジャーナルの役割：WAL vs インテントログ) の計測条件はブロック数のみを算定し時間は算定しないと明記。前提：トランザクション層 + 実機 harness |
| G23.2 | 甲丙差不收敛 | バッチサイズ拡大時、案Aと案Cの差が収束**しない** | **「差」は案A / 案Cの各バッチブロック数の比率を採用**（ユーザー決定 2026-09-28；案Cの代わりに案Bの数値を使用）：E16 (ジャーナルの役割：WAL vs インテントログ) 第5回実行により、2026-10-04時点の幅に基づく収束有無はツリー高に応じて変化し、「バッチ ≤ ストリーム数」の境界ではないことが判明 —— 実際のベースライン 1 TiB（4層、ルート直下11子）においてストリーム数 ≤ 16 は増加後に収束（ピークはバッチ ≈ ストリーム数）、ストリーム数 ≥ 24 はバッチ = 1 から即収束（`research/results/e16-fifth-run-peak-under-today-widths-2026-10-04-h487.out` のストリーム数64の行：`name=cell` 1882、1904行、バッチ1からバッチ64への比率が 3.500000 から 3.169231 へ低下、その `row_summary` は2405行）；ツリー高 ≤ 3 は全ストリーム数でバッチ = 1 から収束、ツリー高 ≥ 5 で走査された全ストリーム数は増加後に収束、ツリー高 4 時の閾値は約 2 × ルート直下の子数（サードパーティ攻撃側レプリカ上でのみ測定、メイン agent の再実行でバイト完全一致、リポジトリ装置での再作成なし、判定 `research/prompts/d23-e16r5-r1-main-verification.md` 第1節総括表）。「3.40× → 3.71×」の数値群は旧幅（ファンアウト恒常128、ツリー高恒常4）下のものである。新規記述への攻撃ラウンドは0巡。⚠️ 案Cは未測定 |
| G23.3 | 四项盘上齐备 | A–Dの4項目がディスク上に完備 | **可能、今すぐ確定すべき** |

- **逆方向判定基準（D23 (ジャーナルの役割と形式) に自前で具備必須）**：レコード照合器（D13 (検証ルート) 確定項目 7：第2入力を要する照合はこれに帰属）の運用開始後、累計N回のフル実行検証において「レコード照合器が捕捉したが checker + O1 (メモリモデル突き合わせ（HashMap 理想ファイルシステム）) + クラッシュポイント再生の3者がいずれも捕捉できなかった」イベント数が依然として **0**（Nは運用開始時に事前宣言）である場合、「案Bがレコード照合器のバイト単位突き合わせを破壊する」という根拠の増分価値は実測ゼロであり、第2軸の再開評価を実施すべきである。この条項がなければ、その根拠は反証不可能となる。

**射程**：毎回の fsync で何を書き込むか、第2軸を再開できるかを管轄；対象ワークロードは D25 (対象ワークロードの優先順位) に帰属。確定内容は「2つの軸が非直交である」という構造的根拠にのみ依拠しており、損益の高低とは無関係；確定時に提示された3つの根拠はいずれも降格され、荷重を受けない：「案Bがレコード照合器のバイト単位照合を破壊する」は当面非荷重、「案Bは『誤用がマウント不能に変わる』機能を購入できない」は適用射程誤りとして撤回、「案Bの収益形状」はボーナス上限が残るのみ。案Bの利益は単一ストリーム同期書き込みにおいてのみ存在し、それこそがデータベースや仮想マシンイメージ（D10 (ランダム書き込み / 断片化の真の回答) でコピーオンライトファイルシステムの共通の弱点として記録）である ⇒ 「案Bのボーナスは有界である」とは言えるが、「案Bの利益は取るに足らない」とは言えない。第1軸の書き込み増幅の前提はマイルストーン「上書き・解放・ロールバック・再利用」増補1のサードパーティ第1ラウンドで攻撃を受けたが、第1ラウンドの指摘は第2ラウンドで覆された（レコードヘッダ内の364バイト新ルートセグメントは第2ラウンド時点で既に存在し、適用は7フィールドの置換であるため、「wal_full は自己改修できない」は「毎回の fsync で別途ツリーテーブルユニットを書き込む」形態にのみ成立）、2ラウンドの判定完了後の結論は3つの分岐が案Aへ収束し本文は不変（`research/prompts/m2-s1-r1-main-verification.md`、`m2-s1-r2-main-verification.md`）。

**根拠**：

- E16 (ジャーナルの役割：WAL vs インテントログ)：祖先を遅延させるアームはチェックポイントでのみルートを発行し、fsync 時にルートを発行したことは一度もない；バッチ化によって祖先遅延の利益を相殺しきれない度合いはツリー高に応じて変化する（第5回実行：ツリー高 ≤ 3 では全ストリームでバッチ = 1 から収束、ツリー高 4 の閾値は約 2 × ルート直下子数、ツリー高 ≥ 5 で走査された全ストリームは増加後に収束）—— これは降格済み根拠「案Bの収益形状」の強度を変更するものであり、結論を変更するものではない。
- サードパーティ：2026-08-28 サードパーティ論証（資料 `_axis2-background.md`）、順方向推論により「第1軸 ⇒ 第2軸」という構造的根拠を提示、元根拠3件の降格経緯および推論状況は変更史に記載。

**未済の検査**：逆方向判定基準のNが未宣言、レコード照合器はクラッシュポイント再生に接続済み（`crates/singlefs-checker-tier/src/crash.rs` の `check_records`）、イベント数は未集計。

#### 確定項目 2：ジャーナルは固定長リングかチェーン式か

**確定内容**：**固定長リングを採用。** ⚠️ 候補理由「チェーン式はジャーナル書き込みパスを割り当てパスに変え、デッドロック2に突入する」は過剰に強いため不採用：チェーン式の増分は有界かつ算出可能であり、リザーブプールを引き上げれば遮断される —— それは真の循環ではない。チェーン式を真に却下したのは以下の3条であり、いずれもデッドロック2を経由しない：

1. **リザーブプールを1桁引き上げなければならず、リザーブプールの引き上げは疑似ENOSPCを悪化させる。** チェーン式下ではジャーナルブロック自体を割り当てる必要がある ⇒ リザーブプールは `ckpt_cost` から `ckpt_cost + 1チェックポイントウィンドウ内のジャーナルブロック` へ引き上げ必須；リザーブプールは通常の割り当てに対して純粋な税金であり、疑似ENOSPCは受入のみが解決できる（確定項目 24）⇒ **チェーン式はコストをデッドロックに変えるのではなく、疑似ENOSPCに変えてしまう。**
2. **プレフィックス判定と空間帰属が強固に結合される。** 固定長リング下では「有効プレフィックスがどこまで到達したか」はどのレコードを再生するかのみを決定する；チェーン式下では**アロケータがどのブロックを発行可能か**までも決定する —— プレフィックス規則と D21 (権威態と派生態の境界) の会計権威は2つの独立した判定でありながら、ブロックごとに完全一致しなければならず、不一致はサイレントなリークまたは二重割り当てとなる；番号途絶で停止した切断点以降のチェーンブロックはリークするか、再発行されたにもかかわらず再生処理が依然としてそれを参照し続ける。
3. **デッドロック3は解消されたのではなく、静的判定基準から動的判定基準へ置き換わったにすぎない —— この条項が安定的かつ決定的である。**

   | | 判定基準 | いつ算出可能か |
   |---|---|---|
   | 固定長リング | 最悪操作 ≤ リングサイズ / F、F ≥ 2 | **mkfs 時に計算、mount 時に検証**（XFS の `XFS_MIN_LOG_FACTOR` と同様） |
   | チェーン式 | 最悪操作 ≤ **現在の空き容量** | 算出不能 —— **現在の空き容量は操作途中で変化する** |

   `.claude/rules/fs-design.md` ハード要件5に逐字で明記：「**操作期間中に不変である入力に基づいてのみ分岐する**……「現在の充填率」は操作途中で閾値を超える可能性があり、**それは危険な分岐変数である**」。⇒ **チェーン式は、判定可能な幾何的不変条件を、本プロジェクトで明令禁止されているランタイム変数に置き換えてしまう。** これはトレードオフではなく、確定済みのハード要件への衝突である。

**射程**：ジャーナルの空間形態を管轄。確定項目 3 と結合：固定長リング下では tail の誤答は**単なる余分な仕事**（縮退可能）であるが、チェーン式下では**リークまたは二重割り当て**となり、tail は権威態でなければならず、tail-before-reuse の厳格順序を要し、FUA がクリティカルパスに載る；**インライン tail + チェーン式は最悪の組み合わせである**（インラインはレコードに便乗して前進するしかなく、チェーン式下でアイドル時にレコードが書き込まれないと ⇒ ジャーナルブロックが永久に回収されず、空転が空間リークに転化する）。⚠️ リング側の幾何的不変条件はすでに [invariants/03-journal.md](../../../crates/singlefs-checker/invariants/03-journal.md) の I-8.1 (リング幾何が十分大きい) として記述済みであり、その checker は実装済み（プールレベル checker の `judge_journal_ring_geometry_fields_agree`、core リカバリ時に3つの等式を同時に判定、`crates/singlefs-core/src/recovery.rs`）。

**根拠**：

- E25 (リングとチェーンで各々必要なリザーブプールサイズ)：チェックポイントの行き詰まりは予測された通りの段階でゼロに収束 —— リングは reserve = ckpt_cost、チェーンは ckpt_cost + 1ウィンドウ分のジャーナルブロック。
- E23 (ジャーナル幾何)：リングとチェーンの差は1項目のみ —— チェーン式は各ジャーナルブロックごとにアロケータへブロック要求を1回行うが、リングは0回；チェーン式にはデッドロック3がなく、リングは幾何的不変条件を要する。
- サードパーティ：2026-08-29 3アームが一致して「チェーン式はデッドロック2に突入する」は過剰と判定（資料 `_d23-background.md`）。

**未済の検査**：なし。

#### 確定項目 3：tail はシステム構成スロットに常駐、jbd2 を2箇所改修

**確定内容**：**システム構成形態を採用**（tail の権威複製は固定位置に常駐し、レコードヘッダにインライン化しない）、**かつ2箇所を改修**：

| # | 変更点 | 理由 |
|---|---|---|
| 1 | **単一複製のインプレース上書きを禁止**、2以上のスロットのローテーション + 世代番号 + 全候補を個別に検証してから最新を選択へと変更 | D22 (ユニット原子性の合成手法) がルートに対して定めた規律と同一：1回のテア（分断書き込み）が唯一の目撃者を抹消する |
| 2 | **毎回のパブリッシュに伴い1回書き込み、パブリッシュシーケンスの最終手順** | D16 (パブリッシュセマンティクス) 確定項目 7 の注記：ルートスロットの永続化後にシステム構成スロットを更新し、クラッシュポイント再生の列挙に入る；D16 (パブリッシュセマンティクス) 確定項目 6 の逐字「毎回のパブリッシュで `checkpoint_txg` を1加算 —— **fsync によりトリガーされたパブリッシュもまたパブリッシュである**」⇒「チェックポイントごとに1回」と「パブリッシュごとに1回」は同一の頻度であり、fsync によるパブリッシュも同様にシステム構成スロットの書き込みが必要 |

- インプレース上書きとコピーオンライトは矛盾しない：tail はルートスロット、ジャーナルレコードヘッダと同様に自己証明ユニット（ルート発行自体がルートリングの1スロットの上書き）に属し、D22 (ユニット原子性の合成手法) 確定項目 21 の管轄下にある；コピーオンライトは親を持たないユニットに対して何の利益ももたらさない。**tail はセマンティクス上派生態である**：権威判定は依然として `jsn` の連続プレフィックス + `checkpoint_txg` である。
- **これより導出される3つのハード要件**：
  1. **リカバリは tail を先に信じて前進してはならない：必ず全リング走査・1件ごとの検証を実施した後に、最長合法プレフィックスを選択しなければならない。** tail が陳腐化している場合、`[tail_stale, tail_actual)` の区間はすでに新周回のレコードによって上書きされている ⇒ 陳腐化した tail から前進すると、**最初の1件目で `jsn != expected` となる** ⇒ 番号途絶で即時停止 ⇒ `tail_actual` 以降の依然として必要なレコードすら再生されない。**それはデータ損失の方向であり、「余分な仕事」の方向ではない。** ⇒「陳腐化した tail はリカバリに余分な仕事をさせるだけである」という命題は、**全リング走査型リカバリの下でのみ成立する**。D22 (ユニット原子性の合成手法) がルートスロットに対して定めた「全候補を個別に検証した後に最新を選択」と同一の規律である。
  2. **再生は冪等でなければならない**：

     > **各ジャーナルレコードの再生は冪等でなければならない** —— 同一レコードを2回再生した結果は、1回再生した結果と同一である。

     「陳腐化した tail は単なる余分な仕事」という命題の前提はこれである：冪等性がなければ、陳腐化した tail は「余分な仕事」から「発効済みレコードの重複適用」へと転化する。確定項目 20「レコードはポインタ層の**目標状態**を担持する」と整合：目標状態は生来的に冪等である（ポインタを特定の値に書き込む操作は、2回書き込んでも結果が同じ）、論理インテントはそうではない（「カウンタに1加算」を2回実行すると結果が異なる）。
  3. **リカバリアルゴリズムは幾何を救済できない**：ウィンドウがリングを突き破った場合、いずれのリカバリ方式でもデータが失われる；リングサイズ ≥ ウィンドウは [checks-owed.md](../checks-owed.md) C28 の責務であり、**リカバリアルゴリズムの責務ではない**。両者が互いに代替することは許されない。
- ブロック再利用が重なった場合、「陳腐化した tail は単なる余分な仕事」は成立しなくなり、リカバリには再生下限（ルートの水位を採用、確定項目 14）が必要となる；これは確定項目 3 に対する絞り込みであり覆すものではない：「全リング走査・1件ごとの検証・tail を先に信じることの禁止」はそのまま成立し、変化するのは検証された合法プレフィックスのうちどの区間が適用されるかである。

**射程**：tail の配置場所、書き込み頻度、リカバリでの利用方法を管轄。バイト表 [layout/01-first-txn.md](../layout/01-first-txn.md) の第0節・第1節は「毎回のパブリッシュで1回」に従い記述：2回のウォームアップ空パブリッシュおよび新規プールでのファイル作成でそれぞれ tail を1回前進させる。⚠️ システム構成形態を採用した際に使用されたコスト表（E23 (ジャーナル幾何) の「2つのアームの差 21 : 500」）は失効済み：第1版では毎回のパブリッシュがチェックポイントであり、定常的な余分な書き込みは21回から毎回のパブリッシュで両ディスク各1スロットへと変化；再生の次元は存在しない（リカバリは tail を読まず、毎回のマウントで全リング走査）。「毎回のパブリッシュでシステム構成スロットをローテーションし、復帰前に永続化を待機する」根拠は判定基準 N-配置（確定項目 14「書き込み可能マウントで読み取った更新状態が読めない」、C554 案B、C577）：ローテーション回数を減らすか永続化を待たずに復帰すると、「確認済みパブリッシュ + ルートスロット一時読み取り不能 + 指名ユニット1点破損」時に確認済みパブリッシュが消失し、かつそのセルは「ルートスロット FUA 前クラッシュ + ユニット1点破損」とディスク面がバイト単位で同一となり、読み取り側で区別がつかない。この根拠は、その種の二重障害がフォールトトレランス射程内にあることに立脚する（ユーザー 2026-09-28 決定）；遅延ブランチ（D18 (ブロックに含める情報) 確定項目 11）が tail を読むことはこの根拠ではない。

**根拠**：

- E24 (リカバリアルゴリズム：tailを先に信じるとレコードを落とすか)：tail を先に信じることで欠落する件数は、ウィンドウ内で書き込まれた件数と正確に一致；全リング走査では欠落しないが、陳腐化した tail 以降のレコードをすべて再実行する（冪等性を強制）；ウィンドウがリングを突き破った場合は両アームともに欠落（リカバリは幾何を救済できない）。
- E23 (ジャーナル幾何)：jbd2 形態と XFS インライン形態は、定常時の追加書き込み量とアイドル後クラッシュ時の再生量において方向が逆 —— tail がチェックポイントごとに1回前進するという旧前提に基づく、射程を参照。
- サードパーティ：2026-08-29（資料 `_d23-background.md`）。
- サードパーティ判定 `research/prompts/c245-r3-main-verification.md`（C245 第3ラウンド）：毎回のパブリッシュローテーションの根拠を判定基準 N-配置に変更；攻撃側プローブにより、案B・案Cは「確認済みパブリッシュ + ルートスロット一時読み取り不能 + 指名ユニット1点破損」において確認済みパブリッシュを喪失し、案Aは4箇所の障害があって初めて喪失することが実測された。
- ユーザー確定 2026-09-28：その種の二重障害はフォールトトレランス射程内にある（原文は変更史）。

**未済の検査**：C77；C594。

#### 確定項目 4：レコードヘッダを単一アトミックユニット内に制約、ヘッダ487バイト

**確定内容**：**レコードヘッダは単一のアトミックユニット内に完全に収まる**（[invariants/03-journal.md](../../../crates/singlefs-checker/invariants/03-journal.md) I-8.2 (レコードヘッダがアトミックユニットを跨がない)）。ヘッダは **487 バイト**：

- **10フィールド 78 バイト**：magic 4 + 種別 2 + アルゴリズム種別 1 + レコードフラグ 1（ビット 0 = 今回のパブリッシュの末尾、確定項目 17；残余ビットは 0 を書き込み、0 以外を読み取った場合は破損と判定） + 自己記述長 4 + 指名項目数 4 + `jsn` **10** + `checkpoint_txg` 8 + nonce 12 + ヘッダチェックサム 32。`jsn` の 10 バイト = インスタンス番号 32 ビット + カウンタ 48 ビット（確定項目 9）。`tail_lsn` はヘッダに含まれない：確定項目 3 が逐字で「tail の権威複製は固定位置に常駐し、**レコードヘッダにインライン化しない**」と定めており、「各レコードヘッダにインライン化された `tail_lsn`」こそが却下された XFS 形態の定義である。
- **確定済みの増分3件 17 バイト、これら10フィールドの 78 には含まれない**：確定項目 7 のトランザクション番号 + コミットマーカー **9 バイト**、確定項目 8 の逆方向チェーン **4 バイト**、確定項目 13 のペイロードチェックサム **4 バイト**。
- **今回のパブリッシュ内シーケンス番号 4 バイト**（符号なし32ビット、1から開始：1回のパブリッシュがN件のレコードに分割される場合は順に 1..N、1件のみの場合は 1；空パブリッシュレコードも 1 を書き込み）：確定項目 14 注1において選択ルートのレコードが読み取れない場合にチェーン先頭を接続するために依拠。4バイト幅およびトランザクション番号・コミットマーカー直後という配置位置は、メイン agent が「空間節約のために自己完結性を犠牲にしない」方針に従い選定、覆すことは可能。ヘッダ内フィールドオフセット：トランザクション番号 78、コミットマーカー 86、パブリッシュ内シーケンス番号 87、逆方向チェーン 91、ペイロードチェックサム 95、新ルートセグメント 99、fsid 463、MAC 471、ヘッダ末尾 487。1回のパブリッシュのN件のレコードのシーケンス番号は順に 1..N で jsn と同期；読み取り側で以下の4ケースに遭遇した場合、いずれも当該レコード破損とみなしチェーン切断して即停止、そのパブリッシュ全体を不適用とする：シーケンス番号 0；パブリッシュ内での番号飛び；アンカーが読み取れる場合で次パブリッシュ先頭のシーケンス番号が 1 でない（このレコードで切断）；同一 (インスタンス番号, checkpoint_txg) 内で末尾フラグを持つレコードの後にさらにレコードが存在（フラグ付きレコードで切断；「後」はカウンタに基づき読み取り隣接を要求しない：フラグ付きレコードよりカウンタが大きく読み取り可能な同一 (インスタンス番号, checkpoint_txg) レコードをすべて対象とし中間の欠番では停止しない；同一インスタンス内でジャーナルレコードの `checkpoint_txg` がカウンタに伴い単調非減少であることが前提であり、この前提があれば全表走査読み取りと同一の適用結果を与える）。I-8.9 が判定していることと同一。
- **確定項目 15 の新ルートセグメント 364 バイト**（ツリーテーブルポインタ 86 + マッピングルートポインタ 86 + ツリーID水位 8 + F 8 + インスタンス表ポインタ 86 + 割り当てレコードツリーポインタ 86 + アンマウント記号 4）。
- **fsid 8 および MAC 16、いずれも新ルートセグメントの後方に配置**：
  - **fsid 8**（システム構成 fsid の下位8バイト、ユニットヘッダの8バイトと同一計測条件、D18 (ブロックに含める情報) 確定項目 7）。これを保持しない場合、I-1.4 はジャーナルレコードに免除規定を設ける必要があり、全リング走査で「前回のファイルシステムが本ディスクに残したレコード」を阻止できなくなる —— mkfs でリング全体をゼロ埋めすることは mkfs が完全に終了した場合にのみ成立し、mkfs 途中でクラッシュした場合は成立しない。8バイトを保持すれば免除は不要。
  - **MAC 16**（fsid の後方に配置；nonce 12 とアルゴリズム種別 1 は10フィールド内に既存）。ルートレコードの29バイトと同一の理由（D22 (ユニット原子性の合成手法) 確定項目 7）：ジャーナルレコードヘッダは自己証明構造であり、MAC を担持する親ポインタが存在しない ⇒ day-1 でスロットを予約しないと、暗号化を有効化した日に全体を平文免除とするか（`checkpoint_txg`、新ルートセグメント内の2つのポインタ、指名項目位置の漏洩）、レイアウト変更を余儀なくされる；D9 (暗号化) 確定項目 3 の原則は逐字で「暗号化のために予約したスロットは全ボリュームで確保し、他用途に転用しない」。I-6 には含めない。
- 登録値は 487（`JOURNAL_HEADER_BYTES`）；10フィールドの 78 は別途 `JOURNAL_HEADER_TEN_FIELD_BYTES` として登録。**78 と述べる際は10フィールドを指すことを明記し、この文とともに引用すること。** 487 は 512 セクタの 95% を占有し、後方に残余 **25 バイト**（56バイトの指名項目は収まらない）；4096 上では残余 **3609**（指名項目64個、⌊(4096 − 487) ÷ 56⌋ = ⌊3609 / 56⌋）。
<!-- format-const: JOURNAL_HEADER_BYTES = 487 stale=头 277 字节|的 277 字节头|合计 **277 字节**|JOURNAL_HEADER_BYTES: u64 = 277|JOURNAL_HEADER_BYTES: u64 = 84|JOURNAL_HEADER_BYTES: u64 = 86|JOURNAL_HEADER_BYTES: u64 = 78|512), 428|512), 426|512), 434|4096), 4012|4096), 4010|4096), 4018|512 上 428|512 上 426|头 86 字节|头 **307 字节**|登记值就是 307|⌊3789 / 56⌋|头 **311 字节**|余 **201 字节**|JOURNAL_HEADER_BYTES = 311|512 上余 201 -->
<!-- format-const: JOURNAL_HEADER_TEN_FIELD_BYTES = 78 -->

**射程**：⚠️ **そのアトミック幅は本プロジェクトでは決定しない**：D20 (荷重受け面: ユニットの原子性と自己完結性) は自己証明ユニットが**実行時にプローブされる** `physical_block_size` に依存することを確定済みであり、512 をフォーマットにハードコードすることは禁止。⚠️ **この値はフォーマット定数であり、変更時は3箇所を同期修正しなければならない**：kb のマークアップと本文、実験コードの `const` およびそれを固定する単体テスト、実験の再実行と成果物の更新；ゲート段階「フォーマット定数が kb と実験コード間で同期していること」（`.claude/gate.d/code-source-discipline.sh` の format-constants セル）により赤判定となる。レコードサイズを 4 KiB 固定長とした後、アライメントの浪費は恒常的に 0 となる（確定項目 12）；可変長レコードのアライメントコスト表は可変長レコードにのみ成立する。

**根拠**：

- E23 (ジャーナル幾何)：10フィールド（実行時は `tail_lsn` も保持）のヘッダは 512 ユニットに収まり、非アライメントで複数レコードが同一アトミックユニットに詰め込まれた場合、先頭以外はテア判定が不能 —— ヘッダは完全に単一アトミックユニット内に収まる必要がある。
- E75 (レコードサイズとリング幾何)：各ヘッダ解釈は 512 および 4096 の両 `physical_block_size` においていずれも1ユニットに収まる；2の冪乗固定長サイズ下ではヘッダがユニットを跨ぐことはない。
- fsid 8 および MAC 16：ユーザーモーダル確定、サードパーティ論証未実施、覆すことは可能、原文は変更史。
- パブリッシュ内シーケンス番号 4 バイト：ユーザー 2026-09-23 決定（`research/prompts/m2-presumed-clauses-r1-main-verification.md` K4 の Z6）、推論による、攻撃ラウンド0巡、原文は変更史。
- ユーザー確定 2026-09-24：ヘッダ 311 着地、末尾フラグを旧「パディング 1」バイトに配置、シーケンス番号の読み取り規則（原文は変更史）。
- ユーザー確定 2026-09-24：アンカーが読めるのに先頭シーケンスが 1 でない、末尾フラグの後に同 txg レコードが存在する、両ケースとも破損としてチェーン切断（実装20報告 `m2-lastflag-implementer-report.md` 第6節 2、3）、原文は変更史；実験なし：両ケースとも CRC-32C の偶然の一致または改ざんイメージでのみ到達。
- 第4ケースの「後」はカウンタに基づき読み取り欠番で停止しない、同一インスタンス内でジャーナルレコードの `checkpoint_txg` がカウンタに伴い単調非減少であることが前提：サードパーティ判定 `research/prompts/m3-formal-r2-main-verification.md` 第2節8行目および `research/prompts/m3-formal-r3-main-verification.md` 第2節第4ケース補足文、両アーム同方向；前提を別途不変条件とする判定は `research/prompts/m3-inv-b-r1-main-verification.md` 第2節9行目；ユーザー確定、原文は変更史；実験なし：読み取り規則の射程補足であり新規数値なし。
- 新ルートセグメントにインスタンス表ポインタ、割り当てレコードツリーポインタ、アンマウント記号の3項目を追加（ヘッダ 311 → 487、指名項目 67 → 64）：サードパーティ判定 `research/prompts/m3-ring-r1-main-verification.md` 第2節2行目および第4節第1項、`research/prompts/m3-ring-r2-main-verification.md` 第2節2行目（初ファイル、アンマウントの2パブリッシュを補完した後に元来書き込まれるはずのルートとバイト完全一致、checker 緑）、`research/prompts/m3-ring-r3-main-verification.md` 第2節1、2行目（フラグバイトに2ビットを配置した形態は専用クリーンアップパブリッシュで突破された；適用箇所で新ルートセグメント由来でないのはアンマウント記号、インスタンス表ポインタ、割り当てツリーポインタの3項目のみ）。ユーザー確定「案Aに戻る」（原文は変更史）。アンマウント記号が4バイトを占有するのは第1ラウンド順方向推論 H1-b のアルゴリズム（86 + 86 + 4）。実験なし：適用箇所のバイト補完であり数値はサードパーティレプリカで実測済み、実験番号なし。

**未済の検査**：なし（C94 は 2026-09-24 に清算済み、未済の検査清算済み表を参照）。

#### 確定項目 5：O2 (独立パーサー + checker) は単項述語、バイト単位照合はレコード照合器の所掌

**確定内容**：**O2 (独立パーサー + checker) は単項述語であり、単一イメージのみを判定する**。「バイト単位の突き合わせ」ではない —— それは二項関係であり第2引数を要する；第2入力を必要とする照合は**レコード照合器**の所掌であり、引数は `(クラッシュ前イメージ, レコードストリーム, クラッシュ後イメージ)`、権威記録は D13 (検証ルート) 確定項目 7。

- ⇒ D23 (ジャーナルの役割と形式) が論理インテントログを却下した第2の理由（「配置先の不確定性により照合が退化する」）は字面上空虚であり、**「照合が構文から意味論へ変化する」と書き直し、かつ独立した第2の理由ではなく第1の理由（再生に第2の実装を要する）の穴塞ぎである**：論理インテントレコードはいかなるディスク上オブジェクトも指名しない ⇒ 照合側がツリーセマンティクス + 会計 + 割り当て正当性を自前実装しなければならない ⇒ 第2実装のコストがそのまま照合側に転嫁される。
- ⇒ **「論理インテントログを却下する」結論は動かず**、第1の理由は独立して成立する（確定項目 20）。
- ⇒ D23 (ジャーナルの役割と形式) の逆方向判定基準の主语は**レコード照合器**である（確定項目 1；主語を O2 (独立パーサー + checker) と書くと字面通り恒常的に0となり —— O2 自体が checker であるため —— 反証不能となる）。

**射程**：D23 (ジャーナルの役割と形式) がこれら2つの検証ロールをどう呼称し、それぞれ何を却下するかのみを管轄；両者が何であるか、引数は何か、コードはどこにあるかの権威は D13 (検証ルート) 確定項目 6 / 7 に属する。

**根拠**：

- 実験なし：単項述語を二項関係に変えることはできず、これは型シグネチャの議論であって測定可能な量はない；2026-08-28 サードパーティ論証（資料 `_o2-background.md`）、逐条根拠は D13 (検証ルート) 確定項目 7 を参照。

**未済の検査**：なし。

#### 確定項目 6：対象ワークロード優先順位をジャーナルラインから除外

**確定内容**：対象ワークロードの優先順位は D23 (ジャーナルの役割と形式) から除外、D25 (対象ワークロードの優先順位) を参照。D8 (コアインデックス構造) / D10 (ランダム書き込み / 断片化の真の回答) / D23 (ジャーナルの役割と形式) を横断する問題であり、ジャーナル固有の問題ではない；ここに残すとジャーナルの詳細と誤認される。**D23 (ジャーナルの役割と形式) のいかなる項目もブロックしない**：第2軸は第1軸によって構造的に排除されており、対象ワークロードは確定項目 1 における「案Bの収益形状」という降格済み根拠の強度に影響を与えるのみで、結論には影響しない。

**射程**：単なる参照ポインタであり、確定内容は D25 (対象ワークロードの優先順位) を参照。

**根拠**：

- 実験なし：3つの決定に跨がるため配置場所を変更したものであり、ジャーナルの問題ではなく、測定可能な量はない。

**未済の検査**：なし。

#### 確定項目 7：1トランザクションは複数レコードに跨走可能、レコードヘッダにトランザクション境界フィールドを付与

**確定内容**：**複数レコードに跨走可能、レコードヘッダにトランザクション境界フィールドを付与**：トランザクション番号 8 + コミットマーカー 1 の計 9 バイト（確定項目 4 の3つの確定済み増分の1つ；確定項目 8 の逆方向チェーンとは**異なるフィールド**であり、合算評価は禁止）。

- トランザクション番号はインスタンスごとに1からカウント、同一トランザクションの全レコードが共有し、jsn とは別個のカウント；ユニットヘッダ書き込み順序はその下位48ビットを格納し上位16ビットは恒常的に0、新規フィールドは開設しない（C113 確定内容が本フィールドを再利用）。トランザクション番号0はトランザクションを担持しない空パブリッシュレコード用に予約（確定項目 19 ①）。
- レコードはトランザクション番号順に追記され、トランザクションがユニット書き込み発行後に失敗した場合はインスタンス切り替えとなり、より大きい番号のレコードは1件も追記されない —— これがインスタンス表の行の W が正確なプレフィックスとして機能する根拠である（第1版のシリアルコミット下で W のプレフィックス性は元来成立、D13 (検証戦略) 確定項目 2；この規律は並行コミット有効化後も成立し続けることを購入）。
- **リカバリアルゴリズム**：「jsn 厳格連続 + チェックサム + 逆方向チェーン」の後に**さらに1手順を追加** —— **コミットマーカーがまだ出現していないトランザクションの全レコードを破棄**。⚠️ **逆方向チェーンに代行させることは不可**：破棄される末尾はチェーン上で**連続**している（もともと順次書き込まれたため）、チェーン検査では一切阻止できない。**2つのゲートは別個の事象を管轄**：チェーンは「同一タイムラインか否か」を、コミットマーカーは「トランザクションが書き込み完了したか否か」を管轄する。
- ⚠️ **末尾破棄は日常動作でありエッジケースではない**：プレフィックスがトランザクション途中に落ちるクラッシュポイントが大半を占める ⇒ リカバリパスの当該手順は大多数のクラッシュポイントで通過するため、メインパスと同様にクラッシュポイント再生で網羅必須であり、例外分岐扱いしてはならない。
- 判定基準は I-8.8 (単一イメージ側) およびクラッシュポイント再生 (整数個トランザクション側) として記述済み；単一イメージ側はプールレベル checker で判定（`walk::check_pool_image` の `judge_commit_markers_per_transaction`）、クラッシュポイント再生側はクラッシュポイント再生ストリーム L8 (並行線3レコード跨ぎクラッシュポイント再生ストリーム) `crates/singlefs-checker-tier/tests/crash_enumeration_record_spill_over_stream.rs`（1トランザクションが2レコードに跨がり、全状態で I-8.8 を評価必須）で判定、C37 は清算済み、未済の検査清算済み表を参照。

**射程**：トランザクションとレコードの対応を管轄；レコードとタイムラインの対応は確定項目 8 であり、別問題 —— 確定項目 7 でいずれの回答を採用してもタイムラインのセルは漏れる。⚠️ D13 (検証戦略) 確定項目 2（シリアル / 並行）の系ではない —— シリアル下でも分割は生じる。I-8.2 (レコードヘッダがアトミックユニットを跨がない) は**レコードヘッダ**のみを制約し、大レコードを禁止する規則はない；1トランザクションで何ユニットのユーザーデータを書き込むかは D16 (パブリッシュセマンティクス) 確定項目 5 末尾の分割規律に帰属、レコード数計測条件との衝突は C310 に帰属。「1トランザクション1レコード」側には未モデル化のコストが2点存在：リングラップアラウンドに連続領域を要すること、レコード全体をメモリ上にバッファしてから書き込みを開始する必要があること。

**根拠**：

- E42 (1トランザクションあたりのレコード数)：1トランザクション1レコードと「複数レコード跨ぎ + トランザクション番号 + コミットマーカー」の双方が I-4.3 を充足；境界フィールドなしで複数跨ぎの場合、不正プレフィックス数は切断プレフィックス数と正確に一致し大多数のクラッシュポイントを占める —— 実験は正誤ではなくコストの所在を区別。
- E45 (複数レコードにまたがるトランザクションのリング占有)：測定された「複数跨ぎがリングを12倍肥大化」は I-8.2 (レコードヘッダがアトミックユニットを跨がない) より厳格なアライメント規則に起因；字面上の倍率は1.00倍であり、これに基づき確定項目 7 の根拠を撤回した措置は不成立（実験結論は失効）。
- ユーザー確定「複数跨ぎ + 境界フィールド」、理由「厳密により一般的」、攻撃ラウンド再検証でも維持；義務3件（フォーマット、リカバリアルゴリズム、不変条件）に対するユーザー計測条件は「アルゴリズムの強化が必要」、原文と経緯は変更史。

**未済の検査**：なし（C37 は 2026-10-11 に清算済み、未済の検査清算済み表を参照）。

#### 確定項目 8：レコードヘッダに32ビット逆方向チェーンを付与

**確定内容**：**レコードヘッダに逆方向チェーン（直前レコードのhash）を追加、幅32ビット。** カバー対象フィールドは確定項目 10、hash アルゴリズムは確定項目 11（完全な32ビットCRC32C）、書き込みパスでの値の計測条件は確定項目 19 ②。

- **補完する欠落**：D23 (ジャーナルの役割と形式) の確定済み3項目（固定長リング / 番号途絶で即停止 / 全リング走査）は**個別に成立するが、組み合わせると漏れが生じる**：固定長リングのスロットは `jsn` で決定 ⇒ 途絶以降の残留レコードは消滅せず上書き待ちとなる；リカバリ後に「プレフィックス末尾 + 1」から書き込みを再開すると、**残留シーケンスの直前に接続してしまう**（確定項目 22）。
- **同一フィールドに2つの責務、双方を記録**：タイムラインの識別；レコード照合器が順方向走査とは**完全に独立したトラバース**で同一プレフィックス境界を再検証可能とすること（`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「検証の2つのパスで同一コードを共有してはならない」に合致）。検証独立性のみを記録すると、「バイト節約」のトレードオフで最初に削減対象とされる。
- **hash がインスタンス番号を網羅（確定項目 10）した時点で、逆方向チェーンは「タイムライン識別」の責務に対して冗長となる** —— インスタンス番号ルートは確率的でなく厳密である；その時点でチェーンに残る責務はトラバース独立性となる。
- ⚠️ **1回の誤受入に対する第2防線は存在しない**：I-8.3、インフライトレコード数上限、`checkpoint_txg` の3ゲートはいずれもトリガーされない（同一チェックポイントウィンドウ内の2つのタイムラインで同フィールドが完全一致）⇒ 結果として**破棄されたタイムラインのポインタ層目標状態が現行状態に組み込まれ**、I-7.4 の保護ウィンドウ内にある場合完全合法となり、サイレントに旧データ化する。

**射程**：⚠️ **誤受入の分母は「空洞 → リカバリ → 継続書き込み → 再クラッシュ」という事故シーケンスの発生回数であり、時間ではない**；設定されたシナリオ点（毎日10回クラッシュ、10年間）は測定ではなく、クラッシュ頻度テレメトリは本リポジトリに未存在。ファイル拡大に伴いチェーン幅を拡張する必要はない：1回の事故でチェーンが試されるのは1箇所の継ぎ目のみ；ファイル拡大に伴い圧迫されるのはシーケンスカウンタ（確定項目 9）とリング幾何（I-8.1 (リング幾何が十分大きい)）である。⚠️ 継ぎ目数の計測条件は「1回の事故に1箇所の継ぎ目」という未モデル化の前提に依拠；1回のリカバリで複数の中断点が残る場合分母が変動 ⇒ **32ビットと確定項目 11 の CRC32C は同一の単一障害点であり**、計測条件崩壊時は両項目同時に崩壊、再開条件は両項目に同時発効（確定項目 11 再開条件第1条）。⚠️ コスト側（2 / 4 / 8バイトチェーンがディスク上で余分にユニットを消費するか否か、E49 (逆方向チェーン幅：32か64か) で6段階のヘッダ幅を走査）はレコードヘッダ総バイト mod 8 にのみ依存し、**構造的でなく偶発的である**ため、ヘッダ幅変更時は再計算必須；逆方向チェーンを含まない現行ヘッダは 483（487 − 4）であり走査対象外 ⇒ **再計算前に「16採択で節約なし」「64ビットも無償」「両側分岐なし」を結論として引用することは禁止。** ⚠️ XFS の `h_prev_block` は書き込み側でのみ代入されリカバリ側で比較されない；引用時は「フィールド形態」の引用に留め、「実運用での防御策」としての引用は禁止。

**根拠**：

- E32 (前回のタイムラインの残留)：固定長リング + 番号途絶で即停止 + プレフィックス末尾から継続書き込みの場合、残留レコードが次回リカバリで接続され再生される；2つの素朴な実装は相反する方向に判定基準を1条ずつ違反 —— 欠落は仕様にあり実装にはない。
- E36 (スロットマッピングの次元)：スロットとシーケンス番号の分離では救済不能、新旧レコードが同長の場合はシーケンス番号基準と同一挙動となり、レコード長が偶然異なる場合にのみ阻止。
- E39 (逆方向チェーンは残留レコードを阻止できるか)：チェーン有効化の全ケースで残留0件かつ自己レコード誤判定0件；誤受入率は各段階で 2⁻ⁿ に近似。
- E51 (逆方向チェーンの衝突機会数)：衝突機会の適正な分母は継ぎ目数（同源レコードのチェーン値は構造的に同等）、これに基づけば32ビットは全段階で十分；隣接比較をすべて機会とみなすトラバース分母は過大推計。
- ユーザー確定：逆方向チェーン追加は「バイトコストが小さく、同時に2つの利益を購入」；幅は「32ビットで十分なら32」、原文は変更史；サードパーティ 2026-08-30 3アーム一致で32ビットを指向。

**未済の検査**：C42；C200。

#### 確定項目 9：jsn = インスタンス番号32ビット + カウンタ48ビット

**確定内容**：**インスタンス番号32ビット + カウンタ48ビット、2フィールド合計10バイト、単一の64ビットに押し込めない。** 分離により追加の利益を獲得：**大小比較でビットフィールドのアンパックが不要**となり、リカバリパスで誤記述しやすいシフト演算を削減。

- ボトルネックはインスタンス番号側：インスタンス番号は fsync 頻度と無関係、毎日10回クラッシュ・10年 = 36,500 回、16ビット容量 65,535 ではマージン 1.8 倍のみ ⇒ 32ビットに拡張（容量 4.29×10⁹、使い切れない）；カウンタ48ビットは拡張不要。
- カウンタはプール全体で継続しインスタンスごとにゼロリセットしない（確定項目 14 第3項）、48ビットの寿命はボリューム全体で計算。
- ⚠️ **これに伴う義務：インスタンス番号のロールバック禁止、かつ保証の担保必須。** 番号がロールバックすると、番号未導入時の状態へと逆戻り（残留レコードの全数再生）。**D9 (暗号化) 確定項目 8 の nonce 水位要件と同型** —— 水位は「平文システム構成に格納してはならない……MAC でカバーされたフィールドに落とさなければならない」と逐字で要求。
- **番号の格納場所と保護主体**：ジャーナルレコード内への格納は循環参照（リカバリはレコードを読んで初めて番号を知るが、レコード読み取り自体に番号の判定が必要）；システム構成内への格納は「ロールバック禁止」要件に抵触 ⇒ D9 (暗号化) 確定項目 8 の nonce 水位と**同一のキャリア問題**であり、個別にではなく同時に確定させるべき。手法：max(システム構成, ルートリング) + 1 を取得し、各システム構成に書き込み（確定項目 16）。

**射程**：⚠️ **確定した2数値にはそれぞれ未測定事項が存在**：① カウンタ側は開発機でのみ測定、**開発機ディスクには電源喪失保護（PLP）が存在しない**、PLP付きエンタープライズディスクでは fsync 頻度が1〜2桁高くなり得るため開発機では未測定の仮説のまま；開発機測定も ext4 の fsync パスであり本プロジェクトの**上限プロキシ**。② 番号側の「毎日10回クラッシュ」は**シナリオ点であり測定ではない**。⇒ **再開条件は判定可能**：現場の fsync 頻度が入手され 10⁵/秒 を超えた場合、カウンタ側を再開。確定項目 14 以降プレフィックス規則は変更なし：選択ルートがカバーする最終レコード直後から再生を継続、選択ルートが最新インスタンスのルートである場合新規レコードは同一インスタンス内で厳格連続；選択ルートが旧インスタンスに後退した場合にのみインスタンス境界に衝突し、厳格プレフィックスに従い停止、コストは D16 (パブリッシュセマンティクス) 確定項目 7 の注記、補完要否は D16 (パブリッシュセマンティクス) 確定項目 8。

**根拠**：

- E44 (シーケンス番号ビット幅の実機測定とコスト)：実機 fsync 頻度下で48ビットカウンタは必要量を大幅に超過、「48ビット不足」の推論前提は実機で不成立；`jsn` を10 / 12バイトに拡張しても512および4096の各ユニット下で指名項目容量は不変、ユニット消費は皆無。
- E37 (ログインスタンス識別子)：インスタンス番号保持アームは残留および新規書き込みの2判定基準を同時に満たす、前提は番号のロールバック禁止 —— 1セルでもロールバックすれば残留全数再生へと逆戻り。
- ユーザー確定、原文は変更史。

**未済の検査**：C199；番号のキャリアおよびロールバック防止保護のID未採番、D9 (暗号化) 確定項目 8 の nonce 水位と同一のキャリア問題に連動。

#### 確定項目 10：previous_hash は直前レコードのヘッダ全体を網羅

**確定内容**：**直前レコードのレコードヘッダ全体を網羅し、その中に `jsn` を含む。** `jsn` は「インスタンス番号32ビット + カウンタ48ビット」（確定項目 9）である ⇒ **インスタンス番号がチェーン値の入力に含まれる** ⇒ 2つのタイムラインのチェーン値が構造的に異なり、誤受入率は 2⁻ⁿ 曲線に戻り、32ビットで十分とする判断が成立。

- ⚠️ **「jsn のみ」でなく「レコードヘッダ全体」を採用する理由**：網羅フィールドを増やしても**ディスク消費は1バイトも増えない**（hash の入力長はディスクに書き込まれない）、一方網羅フィールドを削ると「そのフィールドが改変されてもチェーンが一致する」脆弱性が生じる。
- ⚠️ **実装規律**：`header_csum` の32バイトは**hash 入力に含めてはならない**（ヘッダの一部でありながらヘッダに依存するため）—— ゼロとして関与するよう固定（確定項目 19 ②）。
- **チェーン値には「2つのタイムライン間で確実に異なる」入力が必須**であり、さもなくば誤受入率は**ビット幅にかかわらず100%**：`previous_hash` が直前の `header_csum` を参照した場合、競合する2レコードの magic / 種別 / `jsn`（いずれも expected と等しい）/ `checkpoint_txg`（同一ウィンドウ内で同一）は、指名項目数が同一で暗号化無効時に**構造的に同一**となる。確定項目 8 のビット幅確定とは別問題：幅は「衝突確率の大きさ」に答え、確定項目 10 は「衝突すべき差異が存在するか」に答える。

**射程**：hash の入力を管轄；関数は確定項目 11、幅は確定項目 8、現インスタンス先頭レコードは恒常的に0で書き込みパスでディスクを読まないことは確定項目 19 ②。

**根拠**：

- E37 (ログインスタンス識別子)：インスタンス番号付きリカバリは残留および新規書き込みの双方で0件、確率的でなく厳密 —— インスタンス番号をチェーン値入力に含めることで、チェーンに「2つのタイムライン間で確実に異なる」フィールドが備わる。
- ユーザー確定、原文は変更史；サードパーティ 2026-08-30 逆方向推論が実験コード精読時にチェーン値入力にタイムライン識別フィールドがないことを指摘、経緯は変更史。

**未済の検査**：C42。

#### 確定項目 11：逆方向チェーンのハッシュは完全な32ビットCRC32Cを採用

**確定内容**：**完全な32ビットCRC32Cを採用**、D19 (ブロックポインタ構造と幅予算) 確定項目 2 で確定済みの暗号文チェックサムと同一の関数。

- **主根拠は「この選択は荷重を受けない」ことであり、「CRC32Cがより優れている」ことではない**：32ビットで十分と判定済みであり、かつマージンは**3桁**存在する ⇒ 2⁻³² の曲線上において、2つの候補（完全CRC32C、32ビット切り捨て暗号ハッシュ）は**いずれも十分**であり、どちらを選択してもリスクの桁は変わらない；仮に完全な1ビットを喪失（衝突率倍増）してもマージンは依然十分。
- **「いずれも十分」の上に、2つの副次的理由により CRC32C を選択**：

  | 理由 | 何を獲得できるか |
  |---|---|
  | D19 (ブロックポインタ構造と幅予算) 確定項目 2 と同一の関数 | 実装が1つ減り、checker が1つ減り、クラッシュポイント再生の種類が1つ減る |
  | SSE4.2 の `crc32` 命令が存在 | コミットパスでの暗号計算が1回削減。⚠️ **本プロジェクトで1サイクルも測定していない**ため、これは二次的理由に留め、根拠としてはならない |

- **「切り捨て」でなく「完全」を採用**：切り捨てCRCは短いCRCではなく、チェーン値に切り捨てCRCを採用すると 2⁻ⁿ 曲線は適用不能。
- ⚠️ **撤回された理由**：「CRCの線形性 ⇒ 差分が32ビット以下のウィンドウに収まる場合衝突は証明可能に0、一方切り捨て暗号ハッシュは証明不能な 2⁻³² しか与えられない」。**これは不成立**：その「ウィンドウ限定」の前提はモデルによってハードコードされたもの —— 2つの競合レコードの `named_count` / `record_length` / `nonce` が構造的に同一とモデル化されていたが、それらは**2つの異なるトランザクション**であり、かつ I-6.1 により全ディスク一意の nonce が要求される。
- ⚠️ **これより記録すべき逆方向リスク**：**ウィンドウ条件が不成立の場合、線形アームのテール挙動は暗号ハッシュより劣る。** CRCの衝突差分集合は線形部分空間である —— 同一差分パターンは**毎回衝突する**；切り捨て暗号ハッシュは各ペアで独立した 2⁻³² となる。現有マージン下で結論は動かないが、**これは逆側を指し示す唯一の証拠であり、確定引用時に省略してはならない。**
- **再開条件（2条、いずれか成立で再開）**：
  1. 継ぎ目数という分母の計測条件が覆され、マージンが10倍未満に低下；
  2. 確定項目 10 の網羅範囲が変化し、「2つのタイムライン間で確実に異なる」前提が不成立となった場合 —— その時誤受入率は関数や幅と無関係に100%となる。

**射程**：逆方向チェーンの関数のみを管轄；`header_csum` のアルゴリズム（CRC32C 4バイト + 28バイトゼロ）は D18 (ブロックに含める情報) 確定項目 17、`payload_csum` のアルゴリズムは確定項目 13。

**根拠**：

- E51 (逆方向チェーンの衝突機会数)：継ぎ目数分母に基づき32ビットは全段階で十分、マージン3桁 —— 関数選択は非荷重。
- E61 (逆方向チェーンハッシュアルゴリズムの均一性)：尺度3は「有効ビット幅損失 ≥ 1ビット」のみを排除可能、両候補とも範囲内；尺度2失効に伴い「線形性により0証明可能」の理由も撤回。
- E40 (暗号文チェックサムのビット幅)：切り捨てCRCは「単一ビット反転を確実に検知」という構造的性質を全損 —— 切り捨てでなく完全を採用。
- ユーザー確定（修正後の尺度2は未実行）、原文は変更史。

**未済の検査**：なし。

#### 確定項目 12：レコードサイズは4 KiBを採用

**確定内容**：**レコードサイズは 4 KiB 固定長と決定、システム構成に書き込み**（XFS は同一パラメータをシステム構成の `h_size` に書き込み）。

- 確定項目 4 と整合：レコードヘッダ 487 バイトは単一の `physical_block_size` 内に完全に収まる必要があり、4096 は 512 と 4096 の両プローブ値で割り切れる。
- 固定長レコードを2の冪乗サイズとすることで、I-8.2 (レコードヘッダがアトミックユニットを跨がない) のアライメント浪費は**恒常的に 0** となる：レコードが背中合わせに配置され、開始オフセットは常にユニットの整数倍となる。
- 4 KiB レコードは指名項目 64 個を格納 <!-- format-const: JOURNAL_NAMED_ENTRIES_PER_RECORD = 64 stale== 67 项 -->（確定項目 4）；これ以上拡大しても I-8.1 (リング幾何が十分大きい) における「最悪トランザクション占有量」を引き上げ、リングサイズ下限を引き上げるだけであり、他の利益はない。
- **ユーザー計測条件**：まずこの数値で着地させ、コード実装時に再調整。

**射程**：⚠️ マージンセルは「1回の fsync に1トランザクション」の解釈（対象ワークロード12項目のトランザクションが1レコードに合致）に基づき、D16 (パブリッシュセマンティクス) 確定項目 5 末尾の分割規律（1トランザクションで最大1ユニットのユーザーデータを書き込む）とは同一計測条件にない：分割規律 + 確定項目 7 の1レコード1トランザクション番号によると、seq バッチ1は8レコードとなり各レコードは1〜5個の指名項目しか持たず、「12項目がちょうど1レコード」のセルは存在しない；マージン計測条件とリング幾何の関係は C310 に帰属、ユーザー決定待ち。対象ワークロード（D25 (対象ワークロードの優先順位)）が1桁拡大した場合、本数値は再検討を要し別途実験を設定。リングラップアラウンドおよび「1レコードに収めるためメモリに全レコードを事前バッファする」2つのコストは未モデル化。

**根拠**：

- E75 (レコードサイズとリング幾何)：4 KiB 上で対象ワークロードはちょうど1レコードを占有しレコード跨ぎは0、複数ヘッダ解釈による容量は同等；リング下限は既存バイナリ公表値と一致；2の冪乗固定長下でアライメント浪費は恒常的に0。
- ユーザー確定、原文は変更史。

**未済の検査**：C310。

#### 確定項目 13：ペイロード完全性、header_csum がレコード全体をカバー

**確定内容**：`header_csum` は**レコード全体 `[0, 4096)` をカバー**し、指名項目配列およびその後のパディングを含み、`header_csum` 自体の32バイトはゼロとして関与（32バイト内に CRC32C 4バイト + 28バイトゼロを配置、D18 (ブロックに含める情報) 確定項目 17）；`payload_csum` の4バイトフィールドは**保持**され、指名項目配列をカバーし、アルゴリズムは CRC32C を採用（確定項目 11 の逆方向チェーンと同一、第2の種別は導入しない）。**第1版では「ヘッダは落ちたがペイロードが半分しか落ちていない」事象を個別識別しない** —— いずれのチェックサム不合格でも当該レコード無効とし、リカバリは番号途絶で停止（確定項目 18 / I-8.3 と同一計測条件）。

- D18 (ブロックに含める情報) 確定項目 17 の逐字「自己証明構造（ルートレコード、システム構成スロット、ジャーナルレコードヘッダ）のチェックサムはパディングを含むスロット全体をカバー」と整合：ジャーナルレコードは自己証明構造でありスロットは4096バイト；ヘッダのみをカバーする場合、この汎用規則にジャーナルレコード用の例外を設ける必要が生じるが、ルートレコードやシステム構成スロットにそのような例外はない。
- **4候補のうち他の3候補はいずれも確定条項により排除されており、これは取捨選択ではなく排除である**：

  | 候補 | 却下理由 |
  |---|---|
  | 案A（単一チェックサムがレコード全体をカバー） | 「ヘッダは落ちたが末尾は落ちていない」の識別性が失われる。第1版ではまさに「レコード全体」というカバー範囲を採用しつつ `payload_csum` フィールドを保持 —— すなわち本行に記載されたコストを受容 |
  | 案C（各指名項目が指し示す対象のチェックサムを保持） | D4 (チェックサム位置) のチェックサムは「指し示されたユニットが無傷であること」を証明するが、**「この指名項目自体が完全にディスクに落ちたこと」は証明しない**。ペイロード完全性が問うのはこの指名項目自体の完全な永続化である |
  | 案D（案C + `named_count`） | 案Cの欠陥は解消されず、単に数量チェックが1つ増えるだけ |

- `payload_csum` はヘッダ内フィールドであり、**`header_csum` のカバー範囲内に含まれなければならない**。さもなくばそれ自体の改変がサイレントとなる（ペイロードとチェックサムを同時に改変すると双方通過）；`header_csum` が全体 `[0, 4096)` をカバーすることで本要件は自明に充足される。

**射程**：⚠️ **コストは明記されている**：「ヘッダは落ちたがペイロードが半分しか落ちていない」と「ペイロードの1バイト改変」の識別は元来2つのチェックサムの**差異**に依存していたが、全体カバーによりこれらは「全体が落ちきっていない」と第1版では同一の結末（レコード無効）として判定される；引き換えに「他の2つの自己証明構造と同一の規則」を獲得。「ヘッダのみカバー」を再開する場合、4マスの判定基準表はそのまま引用可能。

**根拠**：

- E76 (ペイロードチェックサムの判別力)：別途付加されたペイロードチェックサムは検査を行っておりヘッダの反響ではない（ヘッダが落ちペイロードが半分落ちた場合、ペイロードが1バイト改変された場合、いずれもこれのみが赤判定）；単一チェックサム形態は型として「ヘッダ落ち末尾未落ち」に回答不能；判定は2つのダイジェスト関数で同等、カバー範囲の産物。
- ユーザーモーダル確定で全体を採用（メイン agent はヘッダのみ `[0, 307)` を推奨）、サードパーティ論証未実施、覆すことは可能、原文は変更史。

**未済の検査**：なし。

#### 確定項目 14：再生の下限は選択ルートにより提示；管理者ロールバック、障害処理およびインスタンス切り替え

**確定内容**：**リカバリは `(インスタンス番号, checkpoint_txg)` が選択されたルートより厳密に大きいレコードのみを適用；陳腐化した tail は「どこからリングスキャンを開始するか」の最適化にすぎず、再生集合を決定しない。**

**プレフィックス判定の完全な基準は以下の5条であり、1つも欠かせない**（I-8.3 (厳格に連続する再生プレフィックス) を引用する際はこの文とともに引用すること）：
jsn 厳格連続（番号途絶で即停止）、**`(インスタンス番号, checkpoint_txg)` がルートの水位より大**、
コミットマーカー完備のトランザクションのみ適用（D23 (ジャーナルの役割と形式) 確定項目 7）、
適用前に指名ユニットを1点ずつ検証（D16 (パブリッシュセマンティクス) 確定項目 7、E77 (発行の永続順序) で荷重受けを証明）—— **指名ユニットの2複製（2つの位置エントリ）の双方が読み取れチェックサムが一致した場合にのみ適用**し、1複製でも読み取れないか不一致の場合は当該レコードおよびそれ以降の全レコードを適用しない。
⚠️ **第5条：適用の単位は1回のパブリッシュである —— 合法プレフィックスが1回のパブリッシュの2つのトランザクションの中間で途絶した場合、そのパブリッシュ全体を不適用とし、前4条で判定されたプレフィックスをパブリッシュ境界で再切断する；前4条を引用する際は本条をあわせて引用すること。**（D16 (パブリッシュセマンティクス) 確定項目 4）**パブリッシュ境界の識別方法**：1回のパブリッシュの末尾レコード = レコードフラグのビット0が1であるレコード（確定項目 4、17）；プレフィックス内でパブリッシュ末尾レコードに到達しない場合、そのパブリッシュ全体を不適用とする。

2つのミラーイメージにおいて「レコードが存在する」とみなす基準：**自己証明チェックサムがいずれか一方の複製で合格すれば存在とみなす**（クラッシュポイント再生での網羅的検証済み —— jsn 3 が1複製のみ永続化した2状態とも正常に読み戻された）。指名ユニットは異なる：**両複製を検証後にのみ適用**（プレフィックス第4条）。トレードオフ：2ディスクプールで1ディスクが完全に脱落し、レコードは残存ディスクにあるが指名ユニットの1複製が脱落ディスクにあった場合、そのレコードは適用されない —— リカバリはより古い版に後退し、直前に確認（fsync 復帰）された最新版が喪失する；D16 (パブリッシュセマンティクス) 確定項目 8 のウォームアップは本ケースをカバーできない。ユーザーは了解の上で受容（C519 セル：「いずれかの検証で合格」が E77 (発行の永続順序) の阻止対象である新旧複製の継ぎ接ぎを見逃すか否かは未測定）。

**インスタンス境界跨ぎ、チェーン先頭、比較順序、カウンタおよび切り替え時の W**（注1–4）：

1. **プレフィックス規則はインスタンス境界を跨がない**：チェーンは選択ルートがカバーする最終レコードの直後から接続し、次レコードのインスタンス番号が選択ルートと異なる場合は即停止（5条基準の第1条そのまま）。**選択ルートがカバーする最終レコード** = 選択ルートと同一インスタンス・同一 `checkpoint_txg` のレコードのうち jsn 最大のもの —— 1回のパブリッシュが複数レコードに分割された場合それらは単一の `checkpoint_txg` を共有し（D16 (パブリッシュセマンティクス) 確定項目 6）、そのパブリッシュの最終レコードこそがルートのカバー終端である。**そのレコードが読み取れない場合**、チェーン先頭は水位直上の最初の読み取り可能レコードに接続する。前提としてその「今回のパブリッシュ内シーケンス番号」が1（確定項目 4）、`checkpoint_txg` = 選択ルートの txg + 1 であること；シーケンスが1でない場合は次パブリッシュの先頭欠落を示し、番号途絶で即停止。**「そのレコード」は末尾フラグで識別** —— 選択ルートのパブリッシュで読み取れるレコードの中に末尾フラグ付きレコードが存在しない場合は「そのレコードが読み取れない」とみなし、「シーケンス1の最初の読み取り可能レコードをチェーン先頭とする」分岐に入る；読み取れる同一 txg レコードの中で jsn 最大のものを採用しない（ユーザー決定 2026-09-24、解釈案B；サードパーティ `research/prompts/m2-presumed-clauses-r1-main-verification.md` K4 分岐で4アーム同中、攻撃ラウンド0巡）。選択ルートが mkfs の第0世代ルートである場合はレコードを1件もカバーせず、以降のレコードは更新インスタンスに属する ⇒ 1件も適用しない。コストは D16 (パブリッシュセマンティクス) 確定項目 7 の注記、新インスタンスのウォームアップ要否は D16 (パブリッシュセマンティクス) 確定項目 8。
2. **レコード水位 `(インスタンス番号, checkpoint_txg)` はインスタンス番号を主順序として比較**、jsn の順序と合致；ルート選択は従来通り txg を主順序としインスタンス番号でタイブレーク（D22 (ユニット原子性の合成手法) 確定項目 7）、両箇所で比較対象が異なり各自の順序を遵守。txg 主順序の場合、新インスタンスの初ルートは「ルートリング最大 + 1」を取るが、破棄インスタンスのオープンチェックポイントでコミット済みのレコードはより大きい txg を持ち得て、水位直後として新インスタンスのルートに誤適用される。
3. **カウンタはプール全体で継続**：新インスタンスはプレフィックス末尾 + 1 から書き込みを継続し、ゼロリセットしない。プレフィックス末尾のカウンタはリング内で読み取り可能な最大値を取得：選択ルート自身のレコードが両複製とも読み取れない場合、新インスタンスの初レコードは読み取り可能最大番号 + 1 で採番され、読み取れなかったレコードの元のスロットに配置（上書き対象は元来読み取り不能）；カウンタはレコードヘッダから読み取り、txg から推論しない。固定長リングのスロットは jsn で決定（スロットと番号の分離は E36 (スロットマッピングの次元) で却下済み）、ゼロリセットすると新インスタンスのレコードが旧プレフィックスの必要スロットに衝突する；確定項目 9 の48ビットは元来全ボリューム寿命で計算。⚠️ **checkpoint_txg も同様**：新インスタンス（通常マウント、切り替え）の初パブリッシュの checkpoint_txg ≥ max(ルートリング内全ルートレコードの txg, リング内全自己証明通過レコードの checkpoint_txg) + 1；通常マウントは全リング走査であり、切り替えはこの下限算出のためにジャーナルリングを再走査しない（メモリ内 txg は今回のマウントで観測した全値以上；切り替え時の選択ルートはディスクから再読み取り、インスタンス切り替え条項参照）。C331 修復候補「レコード走査水位」と同一の量；C331 は清算済み（R2 に変更、未済の検査清算済み表を参照）。コスト：シーケンスの飛び番が現行実装より拡大（最大で破棄タイムライン長相当）；会計への影響なし：D5 (スナップショット / 空間会計メカニズム) 確定項目 2 各ルートの会計ツリーはその世代の行のみを格納し旧世代の点削除を行わないため、飛び番による未削除行の残留はない（C341 同様）。
4. **切り替え時の W は再発行されるチェックポイント内の最大トランザクション番号を採用**：オープンチェックポイント内の完了トランザクションはすべて > W となり、新書き込み順序でやり直す。第2条に基づき、それらのレコードは選択ルートの後方に位置しない；W が字面通りの「最終適用トランザクション番号」を取ると、それらのユニットは述語によりパブリッシュ済みと判定され誰も適用しなくなる（C287）。再発行チェックポイント内にトランザクション番号非0のレコードがない（空パブリッシュを再発行）場合 W = 0（確定項目 19 ①：トランザクション0は W の max に算入しない）；W は引き継がれたトランザクションの各書き込み順序インスタンス行に書き込み、切り替え選択ルートはインスタンス切り替え条項に従い取得。

**第1版にはクリーンシャットダウンフラグは存在しない**：毎回のマウントは一律でリカバリを実行（全リング走査、1件ごとの検証、ルート選択、5条基準および注1–4に従い適用）、前回がクリーンシャットダウンであったか否かによる分岐は行わない；受入、ウォームアップ、ロールバック候補集合、F の発効はいずれもそのようなフラグを参照しない。番号取得前のディスクごとの検査で、遅延分岐のユニット欠落側は本ディスクのリング内でそのバージョンの末尾レコードがあるスロットを読む（システム構成三方第 3 ラウンド判定 `research/prompts/m3-close-sysconf-r3-main-verification.md` 第 2 節 U3 の乙'、実装は `crates/singlefs-core/src/mount.rs` の `last_record_of_the_version_in_its_ring_slot_on_the_device`）；この読み出しはクリーンシャットダウンによる分岐とはみなさない（C499 (クリーンシャットダウンフラグの有無に関する条項なし) の読み方、ユーザー 2026-10-20 確定「認める、この読み出しを名指しする」）。

**初ファイルバージョンをどのレコードの直後に接続するか**：初ファイルバージョンを書き込む前に、接続対象版の直前の先行レコードがデコード可能であり、かつその `checkpoint_txg` がその版の txg と等しいことが必須 —— 新レコードのカウンタはそのカウンタ + 1、逆方向チェーンはそのレコードヘッダを採用（確定項目 8、9）；不充足の場合はいかなる書き込み前にも拒絶（`crates/singlefs-core/src/transaction.rs` の `publish_first_file` が `FirstFileVersionDoesNotFollowTheVersionItBuildsOn` を返却）、1バイトも書き込まない。

**管理者ロールバックはマウント中の1回の前方パブリッシュ**：管理者が書き込み可能マウント中にロールバック候補集合から旧ルート R_old を選択し、ファイルシステムは通常のパブリッシュを発行（D16 (パブリッシュセマンティクス) 確定項目 7 の永続順序）、新ルートのユーザー可視ツリー（inodeツリー、extentツリー）および `deleted_inodes` のルートポインタは R_old の版のものを採用；墓標ツリー、系譜ツリーは現行版に追従し R_old のものは採用せず、ロールバックパブリッシュ時に系譜レコード (T_rb, T_old) を1件書き込み（D18 (ブロックに含める情報) 確定項目 24）；退役ツリーも墓標・系譜ツリーと同様に現行版に追従し R_old を採用しない；ツリーテーブルユニット、割り当てレコードツリー、中央マッピングツリーおよび会計統計量（D5 (スナップショット / 空間会計メカニズム) 確定項目 4）は今回のパブリッシュ用に調整済みアロケータに従い現時計算、R_old の会計からはロードしない（inode番号とツリーID水位は表の「水位」セル参照）；インスタンス表は cur に追従；checkpoint_txg は通常通り1加算、**インスタンスの追加・採番・インスタンス表行の書き込み・レコードの適用や削除は一切行わない**。記法：cur = 現行版、new = ロールバックにより発行される新ルート、ユーザー可視ユニット = データユニットおよび extentツリー、inodeツリーのユニット。

| 項目 | 確定内容 |
|---|---|
| 候補集合 | ルートリング内で cur のインスタンス表に基づき有効と判定され ∧ txg ≥ F_有効（D16 (パブリッシュセマンティクス) 確定項目 1）∧ ファイル保持（その版でファイル書き込み実績あり、未書き込みの版ではない；全削除後にツリーが存在しファイル集合が空の版も実績ありとみなす）のルート：(i, T) が選択可能 ⟺ cur のインスタンス表に i の行が存在しない、または行 (i, Ti, Wi) が存在し T ≤ Ti —— クラッシュ復旧により放棄されたタイムライン上のルートは一切選択不可 |
| 解放 | cur の版の会計で割り当て済みでありながら new が参照しない配置先は、解放世代 = new の txg；隔離ユニット（D19 (ブロックポインタ構造と幅予算) 確定項目 5 ハードルール1：いずれかの複製で不一致照合時は両複製を割り当て済みに維持）を除き、割り当て済みに維持（ユーザー決定 2026-09-27） |
| 復活 | R_old が参照するユーザー可視ユニットのうち、cur の会計で解放済みと記録されたものを割り当て済みに戻し、割り当て世代は R_old の版の会計での割り当て世代を書き戻す |
| 水位 | inode番号水位 = max(cur の版のメモリ内水位, リング内で読み取れインスタンス表上クラッシュ復旧に放棄されていないルートの水位)；max 内の cur メモリ内項目は除外不可：リングのみ参照するとロールバック時に番号が再発行される（C143 の欠落）；cur のインスタンス表が読めない場合コードはロールバック全体をまず拒絶（`crates/singlefs-core/src/mount.rs` の `RollbackError::CurrentInstanceTableMalformed`）、ルートが読めない場合は最初から対象外。ツリーID水位は従来通り = max(cur の版のメモリ内水位, リング内で読み取れる全ルートの水位)。いずれの水位も R_old に追従して後退しない |
| レコード | ロールバック時のパブリッシュレコードのトランザクション番号は0とし、インスタンス内カウンタを進めない（空パブリッシュと同等：ユーザートランザクションを担持しない、D23 (ジャーナルの役割と形式) 確定項目 19 ①）；新ルートが担持する F は現行版ルートの値を引き継ぎ、F の引き上げとはみなさない；jsn は現行レコードから継続（メイン agent 2026-09-26 決定、推論による、攻撃ラウンド0巡） |
| 受入と会計 | ロールバックは従来通り受入不等式を判定しない；系譜コンテナは固定点ロールに帰属し ckpt_cost の Σ リストに算入；会計ラベル4・6は新ルートの版に基づき現時計算（inodeツリーと `deleted_inodes` は R_old を参照するため、連れ戻されたオブジェクトの墓標シェアは相応に上昇；ラベル6を新ルートに基づき現時計算する際、n_live (D3 (領域割り当て) 確定項目 2) = R_old の版のinodeツリー各リーフコンテナヘッダのレコード数合計から `deleted_inodes` の key 数を減算、第1版inodeツリーは単一ルートで最大135コンテナ）、不判定（`research/prompts/m3-delete-r2-main-verification.md` 第3節 V4；`research/prompts/m3-delete-r3-main-verification.md` 第3節 G3-ロールバック；ユーザー決定 K-W5「系譜レコード」、K-W5b） |
| いかなる書き込み前にも拒絶 | R_old が参照するユーザー可視ユニットが cur の会計に記録がない、または割り当て済みでありながら割り当て世代が R_old の版の会計と異なる；R_old の版または cur の版の会計のいずれかのノードが読み取れない；復活集合内の全ユニット（cur の会計で解放済みと記録され今回割り当て済みに戻されたもの）について全ディスク上の各複製が読み取れチェックサムが一致必須、1つでも不可なら拒絶 —— そのスロットは cur のタイムライン上で解放済みであり、解放後に未完了パブリッシュの書き込みによっていずれかのディスクの複製が上書きされた可能性があるため；R_old が参照し cur の会計で同一割り当て世代のまま割り当て済みであり隔離されたユニット（D19 (ブロックポインタ構造と幅予算) 確定項目 5 ハードルール1：ディスク照合で不一致があり両複製を割り当て済みに維持）も同様に全ディスク検証、1つでも読み取れないか不一致なら拒絶（ユーザー決定、原文は変更史）；cur の会計で割り当て済みの他のユニットはアロケータから発行されないため全ディスク検証の対象外（プレフィックス第4条「両複製を検証後にのみ適用」と同方向；メイン agent 2026-09-26 決定、推論による、攻撃ラウンド0巡） |

ロールバックパブリッシュのいかなる時点でクラッシュしても、通常の未完了パブリッシュと同一の結果にリカバリされる；ロールバックルートが永続化した後に初めて管理者に確認通知（D16 (パブリッシュセマンティクス) 確定項目 7）。ロールバック深度は候補集合により規定：通常時はルートリング全体、ディスク逼迫時は直近4状態に縮小可能、各ディスクの最新永続有効ルートは常に含む。**クラッシュ復旧により放棄されたタイムラインのルートがルートリングから離脱するまで、それらが参照するユニットの再割り当ておよびヘッダ消去は禁止 —— アロケータはリング内の全可読ルートの会計を照合し、放棄ルートにのみ参照されているスロットを隔離** —— 照合集合は狭義解釈を採用：「放棄タイムラインのルート」を主語とし、有効ルートに依然参照されているスロットは除外 —— 「有効ルート」はロールバック候補集合内のルートを指し（インスタンス表上有効 ∧ txg ≥ F_有効）、毎回のマウントおよび F 引き上げ時に現時再計算；放棄ルートの会計（ツリーテーブルまたは割り当てレコードツリー）が読み取れない場合もスキップしない：中央マッピングから当該ルートの参照配置先を現時計算し、メモリ上でのみ隔離してディスクには書き込まず、`abandoned_roots_unreadable` に計上して上位報告、マッピング現時計算も不可能な場合は1回再読み取り、依然不可能な場合は採番前に書き込み可能マウントを拒絶し読み取り専用マウントは通常通り（C393 は (b)-マッピングから現時計算を採用、ユーザー 2026-09-28 決定、攻撃ラウンド0巡、判定 `research/prompts/unreadable-at-mount-r2-main-verification.md` L3）；放棄ルートの会計が読めない場合でも通常アンマウント・セッション書き込み・再書き込み可能マウントは成功すべきであり、拒絶は証明がない場合にのみ保留：証明とはメモリ上の現行会計に当該ルートの txg より大きい世代を持ちツリーテーブルスロット（およびマッピングルートスロット）をカバーするレコードが存在することであり、会計のみを参照してスロット内容を読まず、証明があれば拒絶せず無ければ拒絶（C694 証明付き通過、ユーザー 2026-10-07 決定「CとBの双方を実施」、攻撃ラウンド0巡）；F 引き上げの一連の手順（受入とアンマウント）は本パスを通らない：放棄ルートはマウント時にルートリング表に記録された配置先に基づきシャドウ会計を計算、ディスクから会計を再読み取りせず読めなくても拒絶しない、この拒絶は再書き込み可能マウントのパスにのみ留める（D16 (パブリッシュセマンティクス) 確定項目 1「ルートスロットが今回読み取り不良」の行、ユーザー 2026-10-07 決定、攻撃ラウンド0巡）；広義解釈（リング内の任意の可読ルートに参照されたスロットをすべて隔離）は第1版24スロット下で F 引き上げが何も利益を得られなくなる（シャドウ会計；隔離量の受入不等式への算入は C318 参照）。同一確定内容（C113 確定内容 P3）がトランザクションをユニット単位で分割 ⇒ クラッシュ原子性の単位は1ユニットであり、書き込みリクエストではない（リポジトリ内のいかなる決定もリクエストレベルの原子性を公約しておらず、対外セマンティクスの縮小である）。

**書き込み可能マウント時の事前隔離**（C694 の C、ユーザー 2026-10-07 決定「CとBの双方を実施」、攻撃ラウンド0巡）：書き込み可能マウントの読み取り段階、アロケータ再構築後、水位直上の未適用ジャーナルレコードが指名する配置先をすべて隔離、そのルートがロールバック候補ルートであるか否かを問わず、現行会計で割り当て中のスロットのみを除外（元来割り当て済み；今回のマウントで解放・回収時にアロケータが隔離を追従補完、D28 (マウント時コミットメント量) 確定項目 1 第9項）；さらにそのパブリッシュの任意の可読レコードの新ルートセグメントを用いて会計を読み取り、会計内の配置先を一括隔離（`research/prompts/m3-batch33t-code-r1-main-verification.md` 第2節「t1 C の候補免除」「t1 C の盲点」、攻撃ラウンド0巡）；さらにルートリング内で選択版より新しく（ルート選択順）、最新ルートのインスタンス表に基づき有効で読み取り可能なルート自身の会計内の配置先を、そのルートの txg に対応するルートリングスロットごとにグループ化して結合（`C_NEWER_ROOTS`）。これら2つの会計が読み取れない場合：そのパブリッシュの可読レコードが揃っている（シーケンス 1..N 連続、末尾あり）なら追加せず、「レコード欠損時は書き込み可能を拒絶」に委ねる；揃っていない（シーケンスまたは末尾欠損）場合は放棄ルート会計読み取り不能と同様に中央マッピングから現時計算しメモリ上でのみ隔離、失敗時は1回再読み取り、依然失敗かつ現行会計に証明がない場合は採番前に書き込み可能マウントを拒絶、本拒絶はリストに含まれ拒絶原因は C393 と区別して報告（`C_MAPPING` 縮小；ユーザー 2026-10-08 決定「乙：core で別エラーバリアント新設」）；最新可読ルートのパブリッシュレコードが全数読めず会計も読めない場合：マッピング現時計算が可能なら隔離；不可能な場合は当該ルート自身のツリーテーブルと中央マッピングツリーの2ポインタを参照（C834、ユーザー 2026-10-15 確定「同一ポインタの両複製を更新」、判定 `research/prompts/c834-r3-main-verification.md` 第4節、攻撃ラウンド0巡；2026-10-08 決定 Y2 厳格化の置換）：いずれかのポインタの両複製が更新インスタンスのユニットで上書きされている場合は拒絶せず再読み取りもしない（テストケース `urJ`）；さもなくばレコード不揃い分岐と同様に処理（再読み取り1回、証明確認、依然失敗時は採番前に書き込み可能マウント拒絶）。`C_NEWER_ROOTS` と `C_MAPPING` は判定 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第2節 H1、H3 および第3節第1項に従い、攻撃側モデル上でのみ測定、攻撃ラウンド0巡。C の全配置先は本プロセスが当該レコードの所属 txg に対応するルートリングスロットを書き潰し配置先取得を完了した後にのみ解除され、D16 (パブリッシュセマンティクス) 確定項目 1 の全ゼロスロットのプロセス内再読み取り契機では解除されない。マウント時にルートスロットが読めずシステム構成にも目撃されていないルートが新規割り当てしたユニットを保護：シャドウ会計はリング内可読ルートの会計のみ走査し本ルートを見落とす；システム構成に目撃されたパブリッシュは乙-配置続（「書き込み可能マウントで読み取った更新状態が読めない」）が管轄。隔離スロットは受入不等式第9項に算入（D28 (マウント時コミットメント量) 確定項目 1）。**レコード欠損時は書き込み可能を拒絶**（ユーザー 2026-10-07 決定 F1「乙：レコード欠損時は書き込み可能拒絶」、攻撃ラウンド0巡）：水位直上の未適用パブリッシュで、可読レコードのシーケンス不連続または末尾欠損があり、かつ指名ユニットのうち1つでも両複製とも検証不合格の場合（`research/prompts/m3-batch33t-code-r1-main-verification.md` 第2節 t4行、攻撃ラウンド0巡）、1回再読み取りし、依然同様なら書き込み可能マウントを拒絶。書き込み可能マウントが「選択ルート読み取り失敗・システム構成未目撃時に目撃版へ後退」に従い後退した場合、後退先を新たな水位として再判定し、再判定の前にリングを再スキャンして再スキャンしたレコードを用い、読み出し段階でスキャンしたものは用いない（C754 (ロールバック後に再判定でレコード欠落時にリングを再スキャンしない)、ユーザー 2026-10-20 確定「再スキャン」）：その版の直上の未適用パブリッシュが同様の場合、1回再読み取りし依然同様なら同一原因で拒絶（`F1_AFTER_F2`、判定 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第2節 H2 および第3節第1項、攻撃側モデル測定、攻撃ラウンド0巡）。

**書き込み可能マウントで読み取った更新状態が読めない（C554 乙-配置続、ユーザー 2026-09-27 決定；判定基準 N-配置続 は `e158-r3-prereg.md`「5.1」および `e158-r4-prereg.md`「5.1」）**：読み取り段階（ルート選択、journal走査、再生）完了後、プール内各ディスクの2スロットのシステム構成を読み取り：判定不能 ⇒ 真とみなす（判定不能時は R2 を採用、権威記述は D22 (ユニット原子性の合成手法) 確定項目 8：スロットデバイスが読み取りエラーを報告し全ディスクで両スロット自証通過がない場合；ユーザー「R2に変更」決定；実装は R2：`crates/singlefs-core/src/recovery.rs` の `system_configuration_witness_is_undecidable`、読み取り段階と採番で共用；複数回マウントクラッシュポイント再生での注入は E165 (リカバリ再生の有界全数探索とミューテーション殺傷限界) r3 ルート選択と再生合同判定群により読み取り障害履歴下で KJ1T を判定、C331 は清算済み、未済の検査清算済み表を参照）；判定不能でない場合は自証通過全スロットの journal tail の最大値 c_目撃 を取得、0 ⇒ 不判定；選択版のパブリッシュで末尾フラグ付きレコードが読み取れる（いずれかの複製）⇒ c_目撃 > そのカウンタなら真；読み取れずカウンタ = c_目撃 のレコードが読める ⇒ その (インスタンス番号, checkpoint_txg) が選択版より大なら真；双方読めない場合は真。真の場合は今回のマウント内有効読み取りキャッシュ（読み取り成功かつ非ゼロの配置先のみ格納）上で読み取り段階を再実行（R = 1、D16 (パブリッシュセマンティクス) 確定項目 1 の「1回再読み取り」由来、2回読み取り間に待機なし）、再実行結果が偽ならそれを採用して続行、依然真なら採番前に書き込み可能マウントを拒絶しディスク上はバイト単位で不変、読み取り専用マウントは通常通り。アロケータ再構築で破棄判定に用いる最新ルートのインスタンス表が読めない場合も同様に1回再読み取り、依然読めなければ書き込み可能拒絶（コードレビュー第22条）。アロケータ再構築でルートリング2周走査を結合しても非破棄の自証有効ルートが皆無の場合も同様に採番前に書き込み可能マウントを拒絶し、`MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread` を報告（D16 (パブリッシュセマンティクス) 確定項目 1）、本拒絶はリスト掲載；マウント透明性（`crates/singlefs-harness/src/mount_transparency.rs`）はイメージ上で「最新ルートのインスタンス表に基づきリング内に非破棄の自証有効ルートが皆無」を再計算し、読み取り障害未注入時に再計算真なら不当と判定、障害注入時は不判定、インスタンス表読み取り不能時は表によるフィルタをバイパス。**「続」**：採番時の書き込み（`crates/singlefs-core/src/transaction.rs` の `write_acquired_instance`）は tail に 0 を書き込むのを廃止し、採番時に同一手法で読み取った目撃値 c_目撃 を書き込み（ユーザー 2026-09-27 決定「乙-配置続（推奨）」；実装マージ済み、`write_acquired_instance` 内の `highest_journal_tail_of`：各ディスク2スロットの自証通過・同一 fsid のシステム構成スロットの journal_tail の最大値を取得、全無時は0、第1バリア後・初採番書き込み前に1回読み取り）。採番失敗時のロールバック書き込み（D18 (ブロックに含める情報) 確定項目 11）も同一目撃値を保持し、0 に退行させない。mkfs は tail 0 を書き込み、mkfs 後の初採番は 0 を読み取って 0 を書き込み：新規プール新規ファイル作成のバイト列は不変。採番時も同一の判定不能（R2、D22 (ユニット原子性の合成手法) 確定項目 8 参照）で判定：判定不能なら初採番書き込み前に拒絶（実装 `self_verified_system_configurations_of_every_device`、R2採用；あるディスクで自証通過システム構成が皆無の場合も拒絶、ユーザー 2026-10-03 決定；複数回マウントクラッシュポイント再生注入も E165 (リカバリ再生の有界全数探索とミューテーション殺傷限界) r3 で判定、C331 清算済み）。システム構成に目撃されていない最新ルートは、「続」の着地後は次回マウントが自身の c_目撃 で観測；本形態の未済の検査は C554 の Q1。

**選択ルート読み取り失敗・システム構成未目撃時に目撃版へ後退**（ユーザー 2026-10-07 決定 F2「乙：3箇所で実施」、攻撃ラウンド0巡）：リカバリ、書き込み可能マウント読み取り段階、読み取り専用マウントルート選択の3箇所で同一ロジックを使用し、`[目撃版, 選択ルート)` 内で読み取れ選択ルートのインスタンス表により破棄されていない最新ルートへ後退（ルート選択順：checkpoint_txg 主、インスタンス番号副）；3 箇所とも後退するか否かは同じ和集合で判定する：選択された版の走査読み出し、オープン、前の版の構築という 3 つの読み手が読んだユニットとレコード（読めなかった末尾レコードも含む）のうち 1 つでも読めない、復号できない、または判定に通らず、かつシステム構成がそれを目撃していなければ後退する；和集合は後退の判定にのみ用い成否は判定しない、どの版に落ちた後もリカバリは走査読み出し、読み取り専用マウントはオープン、書き込み可能マウントは前の版の構築を見て、成否はそれぞれ自分の読み手に従う；後退できなければ選択された版に留まる。3 箇所で判定に渡すレコードは同じ出所である：トリガー後に journal リングを 1 回再スキャンし、再スキャンしたものを用いる；書き込み可能マウントでは F2、後退先の版の構築に要るレコード、および F1 の再判定（`F1_AFTER_F2`）が同じトリガー後のスキャンを用いる（C755 (リカバリと二種のマウントでロールバックを引き起こす失敗が異なる)、ユーザー 2026-10-20 確定「三文とも補う」、補った三文は攻撃ゼロ回の文言；実装は実装バッチで行う）；目撃版 = ルートリング内でカウンタ = c_目撃 のレコードの (txg, インスタンス) と一致する版。選択ルートのインスタンス表が読めない場合は目撃版と完全一致し読み取れるルートのみを承認し、目撃版と選択ルートのインスタンス表ポインタが同じ場合は区間内で選択ルートと同じインスタンスの読み取れるルートも追加で受け、破棄されていないものとして扱う（2 つのポインタはともにルートレコード内にあり、表は読まない；C751 (読み取り専用マウントがインスタンス表読めない時に確認済みバージョンを飛び越える)、ユーザー 2026-10-20 確定「甲：インスタンス表ポインタを共有する場合のみ受ける」、攻撃ゼロ回；実装は実装バッチで行う）；そのレコードが読めない、c_目撃 = 0、区間内に可読非破棄ルートが存在しない場合は後退しない；c_目撃 が選択パブリッシュ自身のレコード上にある場合は区間が空であり選択版は目撃済みとみなされ後退しない（`F2_NEWEST`、判定 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第2節 H4 および第3節第1項、攻撃側モデル測定、攻撃ラウンド0巡；レコード未読および c_目撃 = 0 による非後退は `research/prompts/m3-batch33t-code-r1-main-verification.md` 第2節 T5行）。選択版が「目撃済み」と判定される条件：末尾フラグ付きレコードが読み取れカウンタ ≤ c_目撃；末尾未読時はカウンタ = c_目撃 のレコードの (txg, インスタンス) と比較し選択版より古くなければ目撃済みと判定；双方未読時は目撃済みとみなし後退しない。F2 は「区間内に末尾レコードは読めるがルートが読めないパブリッシュがある」形には到達しない、論証は二つに分かれる：読み取り障害を注入しなければ F2 は呼ばれない；パブリッシュごとにシステム構成をローテーションするので c_目撃 は高々 1 パブリッシュしか遅れない（適用範囲：読み取り障害を注入しない、システム構成のローテーション書き込みが着地する；C758 (読み取り障害なしで末尾エントリが読めルートが読めないパブリッシュが未計測)、ユーザー 2026-10-20 確定「テストケースをもう 1 件追加して固定する」）。適用範囲外のシステム構成書き込みのサイレント消失の形（2 台のディスクのシステム構成書き込みが成功を報告するが着地せず、さらに 30 回上書きし、選択ルートのツリーテーブルが壊れる）はテストケースとして固定する：リカバリ、読み取り専用マウント、書き込み可能マウントはいずれも区間内の最新の読み取れるルートへ後退し、ルートスロットを上書きされたパブリッシュはその下にある（`crates/singlefs-harness/tests/system_configuration_write_lost_silently_falls_back_without_a_read_fault.rs` の 3 件のテストケース）。

**失敗処理は失敗の性質に基づき2分岐**（C113 確定内容 P1 失敗表）。**判別子は発生時点で判定可能な量であり回数ではない**：失敗発生時、まず**対象デバイスの固定配置先に対してプローブ書き込みを1回実行** —— 書き込み失敗 ⇒ 永続的障害、**次回マウントまで読み取り専用へ移行**；書き込み成功時、さらに**今回の失敗配置先に対して読み取り専用再検証を1回実行**：読み取り成功 ⇒ 一時的障害、**インスタンス切り替え**を実行；読み取り失敗 ⇒ 配置先レベルの永続的破損、同様に**次回マウントまで読み取り専用へ移行**。⚠️ 再検証手順は**読み取り専用で書き込みは行わない**：失敗配置先へのプローブ書き込みは権威データを破壊し（8つの失敗箇所のうち4箇所はルートスロットまたはシステム構成スロット）、固定配置先手順の変更は禁止 —— D2 (RAID ストライプポリシー) 確定項目 13 の逐字「D23 (ジャーナルの役割と形式) 確定項目 14 の2分岐判別子と同一動作」はこれに依拠（メイン agent 確認：`.claude/kb/decisions/02-RAID条带策略.md` 確定項目 13「縮退期間中読み取り専用」、項目15ではない）。**失敗箇所がバリアの場合は再検証すべき配置先が存在しない**ため、それらのケースは固定配置先プローブ書き込みのみを実行。書き込み可能デバイス数が w の下限を下回った場合、または切り替え予約が確保できない場合も直接読み取り専用分岐へ移行；連続切り替え回数が **N_switch = 3**（命名された調整可能パラメータ）を超過した場合も読み取り専用へ移行、これは最終防衛線であり真の判別子はプローブ書き込み + 配置先読み取り専用再検証である。**収束性は切り替えの第1歩が必ず失敗する書き込みであることに依存**：切り替えはまず採番を要し、採番は排他オープン成功の全システム構成への書き込み（全か無か）を要する ⇒ 「書き込めない」類の障害は切り替えを高々1手順しか進ませない（プローブ書き込みがまず阻止、全か無かも即座に失敗判定）。

- 切り替えに必要なブロックはマウント受入時に事前予約：**メモリ上の領域量**であり、毎回のマウントで再計算、クラッシュで消失、**ディスクには永続化しない** —— 割り当てレコードエントリは「割り当て済み」と「解放済み + 解放世代」の2状態のみを表現可能（D3 (領域割り当て) 確定項目 7）、ディスク上で第3の状態を表現不能であり、永続化自体がユニット書き込みを要し「書くために書く」の循環に戻る。予約量は **(N_switch + 1) × 1回切り替えの最悪量**（1マウントで N_switch 回の切り替えを許容、かつインスタンス表チェーンは切り替えごとに1行伸長し行書き込みはコピーオンライトで全チェーン再書き込みとなるため第k回は第1回より高コスト；余分な1回分は今回のマウントの行書き込みパブリッシュのメタデータ用）；**この量の計測条件および計算式の権威本文は D28 (マウント時コミットメント量) 確定項目 3**：1回切り替えの最悪量 = インスタンス表チェーン再書き込み（複製数 2 × 32768 × スライス数、デバイス別計算）+ ウォームアップ空パブリッシュ（最大 R 回 × 現時計算 c_max）、**固定点やり直しおよび番号 > W のデータユニット再書き込みは追加領域を要求しない** —— 切り替えはマウント内の1回のリカバリであり再構築状態で失敗時の割り当ては存在せず、やり直しはチェックポイントのリザーブプールを使用；本段落の引用時は D28 (マウント時コミットメント量) 確定項目 3 を引用。
- **予約が防ぐのは割り当て失敗の経路であり、書き込み失敗の経路ではない** —— 書き込み失敗はプローブ書き込みと採番の全か無かで防御。
- **「割り当て失敗」は発生箇所に応じて3分岐**（分割しないとプール満杯時に全ボリュームが読み取り専用となり、領域解放にディスク書き込みが必要で再マウントも予約の論理積で阻止され、`.claude/rules/fs-design.md`「領域解放操作自体は領域申請を要しない」に抵触）—— フォアグラウンドトランザクションのユーザーデータ割り当て失敗 ⇒ **呼び出し元に ENOSPC を返却**、チェックポイントは通常パブリッシュされマウント状態は不変；バックグラウンド・継続可能・非決定経路（リバランス、墓標回収、scrub、デフラグ）の割り当て失敗 ⇒ **そのタスクを一時停止しインテントに記録**、マウント状態は不変（`.claude/rules/fs-design.md` 第3セル「戒律なし」、D2 (RAID ストライプポリシー) 確定項目 4 c「フォアグラウンド無停止」）；**切り替え自身の予約が取得不能** ⇒ 初めて読み取り専用へ移行。
- ルートスロット書き込み失敗の再送時は checkpoint_txg を1加算して再送（ルートリング領域 = `txg mod R`、D22 (ユニット原子性の合成手法) 確定項目 2；加算しないと同一スロットを繰り返し再書き込みし R 個の障害ドメインを1つに縮退させてしまう）；⚠️ これにより「同一ルートスロットの連続失敗」は判定基準から除外 —— スロットは txg に伴いローテーションし、N_switch ≤ R 下では到達不能であり、残しても空虚なゲートとなる。
- **インスタンス切り替え** = マウント内で1回リカバリを実行：新インスタンス番号を取得、行書き込み、インフライトチェックポイントの再送 —— 選択ルートは**ディスクから再読み取りしてルートを選択**：オープン中のデバイスハンドルに対してリカバリのルート選択と同一規則で現時選択（`crates/singlefs-core/src/recovery.rs` の `choose_root`）、呼び出し元のメモリ内ルートは採用しない —— ルートが永続化済みでシステム構成スロットの手順が失敗した場合、メモリ内ルートはディスク上の現行ルートより古い（C458）；行は `[このルートのインスタンス, 新インスタンス)` にのみ存在し、引き継がれた各書き込み順序インスタンス k は (k, k の最終パブリッシュ txg（未パブリッシュ時は0）, 引き継がれ k に属する最大トランザクション番号) を書き込み、その他は D18 (ブロックに含める情報) 確定項目 11 に従う（旧インスタンスがルート発行済みで引き継ぎトランザクションがすべて自身のものである場合は (旧インスタンス, 最終発行 txg, 再送チェックポイント内最大トランザクション番号) の行）；番号 ≤ W のトランザクションは引き継ぎ（オープンチェックポイント内の完了トランザクションはすべて > W、C287 はこの W で封殺、注4）、番号 > W のものは新書き込み順序でやり直すか未復帰の呼び出し元にエラーを報告、固定点ユニットはすべて新インスタンスで再書き込み；読み取り専用へ移行せず、次回マウントを待たず、コスト = 1行 + 全インスタンス表チェーン再書き込み + 番号 > W のデータユニット再書き込み + 固定点やり直し1回、頻度は未測定。

**本版における失敗処理**：パブリッシュは失敗を許容しない。パブリッシュ失敗時はそれを凍結し、次回パブリッシュ前にバイト完全一致で原様再送（checkpoint_txg、カウンタ、パブリッシュ内シーケンス番号、レコードフラグ、ユニット位置とバイト列はいずれも不変）、再送成功後に初めて次回パブリッシュを構築；したがって合法的履歴において同一 (インスタンス番号, checkpoint_txg) で末尾フラグ付きレコードが2件存在することはなく、リカバリで遭遇した場合はディスク破損とみなし、いかなる書き込み前にも停止（`crates/singlefs-core/src/recovery.rs` の `RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided`）。原様再送は本版のシリアルコミットを管轄；プローブ書き込み、2分岐振り分け、インスタンス切り替えおよびルートスロット書き込み失敗時の txg 加算は依然本項目に属し、失敗処理コードとともに後続マイルストーンで実施。原様再送はシリアルコミット下で1回の失敗をパブリッシュストリーム全体の閉塞へと増幅する、C541 が未済の検査。

**射程**：リカバリがどの区間を適用するか、ロールバックと切り替えの実施手法、失敗の振り分け手法を管轄；インスタンス表の行形式と書き込み規則は D18 (ブロックに含める情報) 確定項目 11、切り替え予約量は D28 (マウント時コミットメント量) 確定項目 3、defer ウィンドウと F は D16 (パブリッシュセマンティクス) 確定項目 1。既知のエッジケース：

- ⚠️ 確定項目 3 に対する絞り込みであり覆すものではない：「全リング走査・1件ごとの検証・tail を先に信じることの禁止」はそのまま成立 —— 変化するのは検証された合法プレフィックスのうち**どの区間が適用されるか**である。
- ⚠️ **シャドウ会計およびインスタンス表による破棄判定（`abandoned_by_table`）の保持理由はクラッシュ復旧であり、管理者ロールバックではない**：管理者ロールバックが一切存在しない履歴においても、クラッシュ復旧が一時的読み取りエラーにより旧ルートへ後退し、インスタンス表がより新しいルートを破棄した場合、シャドウ会計を無効化すると破棄ルートの参照データユニットが再利用され、その後のリカバリでそのルートに着地した際に読み取り不能となる（第1ラウンド H6）。「クラッシュ復旧が旧ルートに着地した当該マウント内でのみ隔離」では阻止不能：再利用の書き込みは次回のクリーンマウントで発生する（第2ラウンド弁護側 G6、コード精読の判断、測定なし）。[checks-owed.md](../checks-owed.md) C314 に計上。C554 乙-配置続（ユーザー 2026-09-27 決定）以降、システム構成に目撃されたより新しいルートの一時的読み取り不能は破棄されなくなった：書き込み可能マウントは1回再読み取りし、依然不能なら書き込み可能を拒絶；シャドウ会計の保持理由はシステム構成に目撃されていない形態（そのパブリッシュのシステム構成ローテーションがディスク未永続化）に縮小；例外：破棄ルートのルートスロット、レコードおよび各ディスクのより新しいシステム構成スロットが同時に全ゼロを読み戻した場合、目撃済みの版も同様に破棄される（確定項目 14 エッジケース②）。
- ⚠️ **C331 判定不能時 R2 採用後のコスト（R2 採用の権威記述は D22 (ユニット原子性の合成手法) 確定項目 8；案A採用時の測定値は `research/prompts/unreadable-at-mount-r2-main-verification.md` L1）**：テアされたシステム構成ローテーション（通常の電源断で生成可能、テアされたパブリッシュは未確認）は判定不能による書き込み可能マウント拒絶を起こさなくなる；依然として判定不能となるのは、スロットデバイスが読み取りエラーを報告し全ディスクで両スロット自証通過がない形態（1スロットのシステム構成が持続的読み取りエラー）であり、書き込み可能マウントは毎回拒絶され、読み取り専用マウントはディスクに書き込まず、そのスロットが再書き込みされる機会がない —— プールは読み取り専用マウントのみ残存；「読み取り専用マウントはディスクに書き込まない」は2026-09-28のコードで実測（2016トレース）、条項上は未規定、C32、C33、C75 参照。書き込み可能マウントが1回も成功しない場合の脱出手段は条項未規定（C596）。C393 のマッピング現時計算も不可能な形態も同様に読み取り専用マウントのみ残存。
- ⚠️ **マウント中ロールバックの既知のエッジケース3種**：①「R_old が依然割り当て中の配置先が cur の会計で同一スロット・同一割り当て世代であること」の照合は確定内容に含まれず補完も不能 —— 解放済みレコードは解放世代を記録（`crates/singlefs-core/src/allocator.rs` の `AllocationRecord`）しており、「当該スロットが再利用された後に再度解放された」ことを検知不能；これが見逃す誤ロールバックは F 後退時にのみ到達可能（第3ラウンド判定第2節 K1：F 有効が最大値の最小値を取る場合は15マス中12マス、SysPre を取る場合は0マス）、F の非後退により防御。② クラッシュ復旧で破棄されたルートが再開時に全数一時読み取り不能となり、シャドウ会計がそれらの会計を読めず隔離0となり、その後に別インスタンスがそれらのスロットを再利用し、障害解除後に新インスタンスのルートを全数破損させると、リカバリが破棄ルートに着地して読み取り不能 —— ロールバック形態と無関係の多重障害、C554 参照 —— 乙-配置続が防御するのは目撃スロットが読め選択版以降のルートが読めないケース（1回再読み取り、依然不能なら書き込み可能拒絶）；破棄ルートのルートスロット、レコードおよび各ディスクの新しいシステム構成スロットが同時に全ゼロを読み戻した場合、目撃済み・確認済みの版も同様に破棄され、射程末尾の「F2 のコスト」と同類（サードパーティ判定 `research/prompts/m3-c554-c572-r1-main-verification.md` 第2節「破棄インスタンスのルートが再開時全数未読」の行）。③ 2つの状態間を行き来するロールバックがルートリング1周を超過した場合、退避可能な個別状態数が4未満となる（D16 (パブリッシュセマンティクス) 確定項目 1 のルートリング容量境界）。
- 実装のチェーン先頭は選択ルートがカバーする最終レコードにアンカー（`crates/singlefs-replay-prefix/src/lib.rs` の `counter_of_the_last_record_the_root_covers`、`crates/singlefs-core/src/recovery.rs` の `replay_journal` が `replay_core` を経由してプレフィックス取得）、それに対する赤判定テストは `crates/singlefs-harness/tests/writable_mount_takes_a_second_instance.rs` の `one_missing_record_right_after_the_chosen_root_stops_the_prefix_even_when_later_records_are_intact` 等のケース、および `crates/mutations.tsv` 内のアンカー1ビットずれ行（C365 清算済み、未済の検査清算済み表を参照）。
- ロールバック時のラベル6の n_live 現時計算解釈（「受入と会計」セル）の攻撃ラウンドは0巡。
- ⚠️ **読み取り専用再検証は配置先の持続的破損を買い取るものであり、コストとしてそれ自身の攻撃ラウンドが0巡。** 固定配置先プローブ書き込みを単独で使用した場合、配置先の持続的破損時に無駄な切り替えが反復（攻撃側がレプリカ装置で24の持続障害シナリオ中計46回を実測、本リポジトリ装置では未再実施のため kb 未収録）；読み取り専用再検証はこれを弁別可能。本改修は攻撃側アーム自身の提案であり、**攻撃ラウンド0巡**、ユーザーは2026-09-21に了解の上で受容。
- **切り替え時のディスク再読み取りによるルート選択に伴う3点**：ディスクから選択されたルートがメモリ内ルートより新しい場合、インフライトチェックポイントを再送せず、ディスク選択ルートを基準として継続；切り替えが書き込むインスタンス表行および W はディスク選択ルートに基づき記述；ロールバックパブリッシュにおいて、管理者が選択した R_old は「ディスク選択ルートを最新ルートとして算出されたロールバック候補集合」に含まれることが必須であり、含まれない場合は拒絶しディスク上はバイト単位で不変。マウント内インスタンス切り替えの実装はマイルストーン「上書き・解放・ロールバック・再利用」収束表第16、40行とともに後続マイルストーンへ（C458）。
- ⚠️ **ゼロユニットパブリッシュ分岐は凍結しない**：パブリッシュ末尾のシステム構成ローテーション後のバリアがエラーを報告した際、ユニット保持 / レコード保持のパブリッシュはローテーションエラーと同一処理（凍結、原様再送）；ゼロユニットパブリッシュ（`crates/singlefs-core/src/transaction.rs` の `publish_without_units`、末尾で `persist_the_root_then_rotate_the_system_configuration` を通るパス）は凍結せず、マウント全体がエラーを返却（`PublishError::BlockDevice` 経由）、次回パブリッシュは次回マウント時に新インスタンス番号で実行；条項未規定の分岐に対する設計選択（判定 `research/prompts/m2-closeout-code-r2-main-verification.md` 第1節「最新ルート未読、目撃とバリア」；`m2-impl-c577-barrier-implementer-report.md` 設計課題第2項）。これを固定するテストケースは `crates/singlefs-harness/tests/a_zero_unit_publish_whose_post_rotation_barrier_fails_returns_the_mount_error.rs` の `a_zero_unit_publish_whose_post_rotation_barrier_fails_is_not_frozen_and_the_whole_mount_returns_the_error`：mkfs のみのプール初回書き込み可能マウントにおいて、行書き込みゼロユニットパブリッシュがディスク1ローテーション後のバリアでエラー報告 ⇒ マウントは `MountError::Publish`（原因 `PublishError::BlockDevice`）を返却、永続化パブリッシュ0件、失敗1件；記録ストリームはディスク0のバリアまでで停止しディスク1は未記録；エラーのないデバイスに復帰して再マウントすると採番2、(インスタンス 1, txg 1) を選択。ファイルイメージ上の結果のみを固定：harness のファイルデバイスには書き込みキャッシュがなく、ディスク1のローテーション書き込みはバリアエラー前にイメージに到達しており、「バリアエラー時にローテーションが未永続化」というクラッシュ状態はカバーしない。
- ⚠️ **C366 確定内容は書き込み可能マウント層で到達可能なディスク面を持たない**：注3「プレフィックス末尾のカウンタはリング内で読み取れる最大値を取得」は、選択ルート自身のレコードが両複製とも読めない場合にのみ txg と分離可能；その形態は書き込み可能マウントの読み取り段階で C554 乙により「双方未読は真」と判定され、1回再読み取りしても真なら採番前に書き込み可能を拒絶（C579 ユーザーは乙の字面維持・書き込み可能拒絶を決定）、採番・行書き込み・ウォームアップへ到達しない。固定されている挙動：関数層 `crates/singlefs-harness/tests/warm_up_journal_counter.rs` の `warm_up_after_a_journal_counter_other_than_zero_counts_records_on_from_it_while_txg_stays_one_and_two`（リング内末尾 jsn 40 時ウォームアップは jsn 41、42 を書き込み、txg は 1、2 を維持）；同ファイル `c366_when_the_chosen_root_own_record_is_unreadable_the_writable_mount_is_refused_by_the_undecidable_witness_and_the_read_only_mount_opens_the_chosen_root`：書き込み可能マウントはいかなる書き込み前にも拒絶、両ディスクはバイト単位で不変、読み取り専用マウントは通常通り (インスタンス 1, txg 4) を選択し、リング内の読み取り可能 jsn 1–3 は1件も適用しない（`m2-investigate-three-reds-report.md` 第1節第2項、第7節）。
- インスタンス表の行削除スイープ（D18 (ブロックに含める情報) 確定項目 11「回収」）は放棄ルート参照ユニットを保護：読み取り成功の全ゼロルートスロットは読み取り不能と同等に扱い（「当該インスタンスのルートが存在する可能性あり」とみなす）、今回削除しない；その他のスロットは自証通過が必須であり、さもなくば1回再読み取りし依然不合格なら今回削除しない（推論による、ディスク面未作成）。
- ⚠️ **目撃版へ後退するコスト**（ユーザー 2026-10-07 F2 決定時に了解の上で受容）：確認済みの版が、新しいシステム構成スロットとそのユニットが同時に全ゼロを読み戻した際、リカバリ・書き込み可能マウント・読み取り専用マウントが前版をサイレントに読み戻す；書き込み可能マウントが前版上で採番した後にその版は破棄される（推論、未測定）（`research/prompts/m3-c694-r2-main-verification.md` 第2節「K3 リカバリ側補完法」行、第3節第7項、第4節 F2）。
- 第1版で生きたスナップショットが存在する場合管理者ロールバックを拒絶：現行版または対象版のツリーテーブルにコード14 / 15エントリが存在する場合は拒絶（ユーザー確定）。ロールバック開放時の形態（ツリーテーブルは対象版を丸ごと採用、解放集合・復活集合は生存ヘッド ∪ 生存スナップショット ∪ deadlist で計算、破棄進行中スナップショットは参照から除外）は攻撃側モデル上で違反0件の形態、判定 `research/prompts/m4-snapshot-r1-main-verification.md` 第3節 S8 に記録。

**根拠**：

- E78 (リプレイの開始点)：陳腐化した tail にブロック再利用が重なると、「全リング走査 + 番号途絶で即停止 + 冪等性」の3点セットが閉塞しない —— 不一致で即停止するリカバリは健全イメージ上で自己中断し、不一致をスキップするとテア判別力がゼロに縮退；水位アームと末尾削除アームの双方が全判定基準を満たすが、水位アームを採用したのは水位がルートレコード既存の `(インスタンス番号, checkpoint_txg)`（D22 (ユニット原子性の合成手法) 確定項目 7）と一致しフォーマットフィールドの新規追加がゼロであるためであり、末尾削除アームはリカバリパスに再試行ループを追加する必要がある。
- E77 (発行の永続順序)：適用前の指名ユニット検証を除去すると、ユニットとレコードが同セグメントのアームで即座にサイレントな継ぎ接ぎが発生 —— 第4条が荷重を受ける。
- E104 (スキャン再構築における現行バージョンの判定)：新ルートが T_old + 1 を取ると破棄ルートを圧倒できない —— 注2、注3の新インスタンス初パブリッシュがリング内最大 + 1 から開始するのはこれに依拠；もう半分の「ロールバック行を書かない場合ロールバック破棄ユニットが全規則アーム上で復活する」も依然荷重を受ける —— 設計はロールバック行を書かない（墓標第1ラウンド判定 `research/prompts/m3-tomb-r1-main-verification.md` 第2節、`research/prompts/m3-delete-r3-main-verification.md` 第3節）；ディスクスキャン再構築が「マウントされたルートがロールバックである」と認識するのは系譜レコードに依拠、読み取り側の利用法は未済の検査（K-W5b、C113 と同グループ）。
- E150 (放棄されたルートが参照するユニットのロールバック再利用)：シャドウ会計はモデル内で2マスとも違反0件、余分に隔離されたユニットは破棄された最新ルートがローテーション上書きされるまで維持。モデル内の破棄タイムラインは管理者ロールバックにより生成されたもの；シャドウ会計の保持理由はクラッシュ復旧が放棄したタイムライン（第1ラウンド H6）であり、メカニズムは同一、モデルはその形態を再実行していない。
- E142 (新規プール新規ファイルのドライラン)：ドライラン時に装置は「2つのミラーイメージのいずれか一方が合法であれば存在」を採用し、1複製のレコードを改変しても全合法レコードを読み取り ——「レコードが存在する」の計測条件がこれにより収束。
- サードパーティ：`research/prompts/c199-r1-main-verification.md`、`c199-r2-main-verification.md`（注1–4）；`c322-r3-main-verification.md`（採番バリアとスキップされた番号）；`c329-c330-r3-main-verification.md`（切り替え選択ルートと W の行）；`c143-r3-main-verification.md`（新ルート初 txg は CJ2 を採用）；`m2-r1-main-verification.md` 第3節および `m2-step45-code-r3-main-verification.md` 第6節5（シャドウ会計の対象、有効ルート）；C113 確定内容（`_c113-r9-final.md`）。
- ユーザー確定（再生下限、ロールバック、失敗表、切り替え予約計測条件、ミラー2複製の計測条件各1回）、原文は変更史。
- `research/prompts/c381-r3-main-verification.md` 第3節：候補案B（プローブ書き込みを失敗配置先への書き込みに変更）は3箇所で独立して指摘 —— バリア上に配置先がない、8箇所の失敗配置先のうち4箇所が権威構造である、ユーザー第2ラウンド第1問（配置先をアドレス空間表に含め「破損しても問題ない」を購入）と衝突、D2 (RAID ストライプポリシー) 確定項目 13 の記述が即座に偽となる。
- ユーザー確定 2026-09-21（改修案B）、原文は変更史。
- ユーザー確定 2026-09-23：切り替え選択ルートはディスクから再読み取りして選択（`research/prompts/c381-r3-main-verification.md` 第2節ゼロラウンド形態、第4節第3問）、原文は変更史；攻撃ラウンド0巡。
- ユーザー確定 2026-09-23：注3のプレフィックス末尾カウンタはリング内可読最大値を取得（C366 実装担当が選択ルート自身のレコードが未読時の条項欠落を指摘）、原文は変更史；本確定は書き込み可能マウント層で到達可能ディスク面を持たない（射程参照）、C579 ユーザー 2026-09-27 は乙の字面維持・書き込み可能拒絶を決定、原文は変更史。
- ユーザー確定 2026-09-23：第5条のパブリッシュ境界は確定項目 17 の取り決めに従い末尾を識別（C245 第2ラウンド `research/prompts/c245-r2-main-verification.md` 第4節3つの判別量の1つ、攻撃ラウンド0巡）、原文は変更史。末尾はレコードフラグのビット0で識別（確定項目 17）；パブリッシュ内レコードでフラグ付きが未読の場合、全体を不適用。
- ユーザー確定 2026-09-23：注1のアンカーは同一 txg 内で jsn 最大のものを採用、アンカー未読時はパブリッシュ内シーケンス番号によりチェーン先頭を接続、第1版にはクリーンシャットダウンフラグなし（サードパーティ第1ラウンド `research/prompts/m2-presumed-clauses-r1-main-verification.md` K3、K4；K4の2改修は攻撃ラウンド0巡、シーケンス番号は推論、現行実装済み：レコードヘッダの「パブリッシュ内シーケンス番号」フィールド、確定項目 4）、原文は変更史。
- ユーザー確定 2026-09-24：初ファイルバージョン書き込み前に先行レコードを検証、2026-09-24実装に基づき条項化（C495、サードパーティ `research/prompts/m2-refusals-presumed-r1-main-verification.md` 第3節 R5：拒絶前は0バイト書き込み、拒絶グループは0障害で到達不能）、原文は変更史。
- ユーザー確定 2026-09-24：切り替え時のディスク再読み取りに伴う3点はメイン agent 推奨に従い決定（判定 `research/prompts/m2-newq-r1-main-verification.md` 第3節 N5–N7、原文は変更史）；実験なし：3点ともディスク上フォーマットを変更せず実装は本マイルストーン外、差はセマンティクスのみ。
- ユーザー確定 2026-09-24：末尾レコードの複数跨ぎ、最終レコードへのフラグ付与（原文は変更史；サードパーティ `research/prompts/m2-treesplit-r1-main-verification.md` T6 (拒絶維持時に拒絶するか否かはプールの履歴に依存する)、確定前の字面で末尾を識別すると半端なパブリッシュが適用される、フラグ付きセル 0 / 0）。
- ユーザー確定 2026-09-24：末尾はフラグで識別、注1アンカーは解釈案B（原文は変更史；実装13報告 `m2-p6-header-implementer-report.md` 第4節①の2解釈）。
- ユーザー確定 2026-09-24：本版のパブリッシュ失敗時は原様再送（実装20報告 `m2-lastflag-implementer-report.md` 第6節1：同一インスタンス失敗後継続送信すると同一 (インスタンス番号, checkpoint_txg) の末尾フラグ付きレコードが2件残存）、原文は変更史；実験なし：失敗後の送信内容を規定するのみでフォーマット変更なし。
- `research/prompts/m2-final-code-r2-main-verification.md` 第3節 Z8（本版パブリッシュ失敗時原様再送、攻撃側未指摘）；Z9および第4節改修1–4はロールバック目撃に関するものであり、目撃削除に伴い非荷重。
- 管理者ロールバックをマウント中の前方パブリッシュへ変更（候補集合、解放、復活、水位、拒絶）、プレフィックス第5条およびロールバック目撃の削除、シャドウ会計根拠を H6 へ変更：実験なし、サードパーティプロトタイプ（`research/prompts/m2-rollback-forward-r1-opus-model/`、`m2-rollback-forward-r2-opus-model/`、`m2-rollback-forward-r3-opus-model/`）で実測、実験番号なし；3ラウンド判定 `research/prompts/m2-rollback-forward-r1-main-verification.md`、`m2-rollback-forward-r2-main-verification.md`、`m2-rollback-forward-r3-main-verification.md`。水位 max(メモリ, リング) の厳格化（第3ラウンド判定第2節 K1）は攻撃ラウンド0巡；ユーザー確定 2026-09-25（前方パブリッシュ、マウント中のみ）および 2026-09-26（第3ラウンド4問）、原文は変更史。
- メイン agent 2026-09-26 決定：ロールバックパブリッシュ前に復活集合の全ユニットをディスクごとに検証（「いかなる書き込み前にも拒絶」最終文）、ユーザー 2026-09-25 原文「1 is dropping one disk after fsync failure. Cannot accept, violates data safety commitment」（`records/2026-09-24-里程碑二收尾调度.md` 第3節）に従い、C519「両複製を検証後にのみ適用」と同方向；実験なし、推論、攻撃ラウンド0巡；読み取りオーバヘッドは R_old 以降に解放されたユーザー可視ユニット数に比例、未測定。
- ユーザー確定 2026-09-25：指名ユニットは両複製検証後にのみ適用、1ディスク完全喪失時に直前確認済み版を喪失することを受容（C519 実装1再現：`crates/singlefs-harness/tests/whole_device_loss_after_warm_up.rs`）、原文は変更史；実験なし：2026-09-25実装に基づき条項化、「いずれかの検証で合格」が E77 (発行の永続順序) の継ぎ接ぎを見逃すか否かは未測定。
- ユーザー確定 2026-09-27（C554 乙-配置続、乙を選択後に「配置続（推奨）」を補足選択）：書き込み可能マウントがシステム構成に目撃された更新状態を読み取れない場合、1回再読み取りし依然不能なら書き込み可能を拒絶；採番書き込みは tail = 0 を書き込まず、採番時に同一手法で読み取った目撃値を書き込み（実装マージ済み、`highest_journal_tail_of`）；判定基準 N-配置続 は `e158-r3-prereg.md`「5.1」および `e158-r4-prereg.md`「5.1」に定義、原文は変更史。E158 (ルート選択と修復の4分岐) 第4回実行：第3段階 H1g で乙-配置と乙-配置続を分離（16/32 対 0/32）、第2段階は4群で双方同一。
- ロールバック時ラベル6の n_live 現時計算解釈：`research/prompts/m3-blanks-r1-main-verification.md` 第4節「ロールバック時の n_live 現時計算」（0巡；メイン agent は順方向推論の解釈を採用）。
- サードパーティ判定 `research/prompts/m3-ring-r3-main-verification.md` 第2節9行目：スイープが全ゼロルートスロットをルートなしと誤認するとリング内放棄ルートの参照ユニットを消去し明文保護に違反；全ゼロスロットを「ルートなし」とみなさず自証通過または全ゼロを要求し再読み取りする2点の厳格化を採用（推論、未測定）。実験なし：保護対象は放棄タイムライン残余であり活データではない。
- 全削除後の版（ツリーあり、ファイル空）へのロールバックを候補と認定：メイン agent 2026-10-02 決定「Q-Kを採用」、候補集合原文の「ファイル書き込み実績あり」を解釈（`records/2026-10-01-删除与并发事务.md` 第8バッチ）；2026-10-03 メイン agent は第9実装担当の同一選択を採用。実験なし。
- アロケータ再構築でルートリング2周走査を結合しても有効ルートがない場合の拒絶を別エラーバリアントとしリスト追加、候補集合から単一周期ウィンドウの論理積を除去：ユーザー確定 2026-10-05（Q3「別エラーバリアント、リスト追加」およびスロット別2周結合）、原文は変更史；判定 `research/prompts/m3-batch10-code-r1-main-verification.md` 第2節および第6節、`research/prompts/m3-batch10-code-r2-main-verification.md` 第6節；再計算判定基準は `crates/singlefs-harness/src/mount_transparency.rs` 2026-10-05 再計算由来。候補集合の論理積は D16 (パブリッシュセマンティクス) 確定項目 1 のリング内最古有効ルート後退（ユーザー 2026-10-04 決定）後の残余。実験なし。
- 放棄ルート会計読み取り不能時の証明付き通過、書き込み可能マウント時の事前隔離、レコード欠損時の書き込み可能拒絶、選択ルート読み取り失敗時の目撃版後退：判定 `research/prompts/m3-c694-r1-main-verification.md` 第2節 K1、K2 および第4節、`research/prompts/m3-c694-r2-main-verification.md` 第2節 K1–K3 および第3節第1、2、6、7項、分岐リスト `m3-c694-r2-forks.md`；ユーザー確定 2026-10-07（C694「CとB双方実施」、F1「乙：レコード欠損時は書き込み可能拒絶」、F2「乙：3箇所で実施」）、原文は変更史。実験なし：サードパーティレプリカとモデルで測定、実験番号なし。
- 書き込み可能マウント事前隔離に新可読ルート会計を結合（`C_NEWER_ROOTS`）、会計未読かつレコード不揃い時はマッピングから現時計算（`C_MAPPING` 縮小）、F2 後退後に F1 を再判定（`F1_AFTER_F2`）、F2 は区間内最新ルートへ後退（`F2_NEWEST`）：判定 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第2節 K2 4行（H1–H4）、第3節第1、3項および第4節；ユーザー確定 2026-10-08「4形態すべて改修」、原文は変更史。C_MAPPING 拒絶をリスト掲載し C393 と区別報告：ユーザー確定 2026-10-08「乙：core で別エラーバリアント新設」、原文は変更史。新可読ルートのレコード全未読かつ会計未読時：マッピング計算可能なら保護、計算不能でも拒絶せず：メイン agent 2026-10-08 決定（`records/2026-10-01-删除与并发事务.md` 第39バッチ）。4改修は攻撃側モデル上でのみ測定、5箇所とも攻撃ラウンド0巡。実験なし。
- F2 は 3 つの読み手の和集合で後退を判定し、後退できなければ選択された版に留まり、判定に渡すレコードはトリガー後に再スキャンしたものを取る（C755 (リカバリと二種のマウントでロールバックを引き起こす失敗が異なる)）、F1 の再判定前にリングを再スキャンする（C754 (ロールバック後に再判定でレコード欠落時にリングを再スキャンしない)）、読み取り障害を注入せずシステム構成のローテーション書き込みが着地する場合、F2 は末尾レコードが読めルートが読めないパブリッシュに到達しない（C758 (読み取り障害なしで末尾エントリが読めルートが読めないパブリッシュが未計測)）、番号取得前のディスクごとの検査で本ディスクのリング内の末尾レコードを読むことはクリーンシャットダウンによる分岐とはみなさない（C499 (クリーンシャットダウンフラグの有無に関する条項なし)）：判定 `research/prompts/m3-close-rootring-r2-main-verification.md` 第2節 Y3、`research/prompts/m3-close-rootring-r3-main-verification.md` 第2節 C755 (リカバリと二種のマウントでロールバックを引き起こす失敗が異なる) のセル；ユーザー確定 2026-10-20（和集合の三文「三文とも補う」、再判定の再スキャン「再スキャン」、F2 の論証「テストケースをもう 1 件追加して固定する」、クリーンシャットダウンの読み出し「認める、この読み出しを名指しする」）、原文は変更履歴にある。実験なし：数値は三方レグのレプリカで計測、実験番号なし。
- 選択ルートのインスタンス表が読めない場合、目撃版と選択ルートのインスタンス表ポインタが同じときに限り同じインスタンスのルートを受ける（C751 (読み取り専用マウントがインスタンス表読めない時に確認済みバージョンを飛び越える)）：判定 `research/prompts/m3-close-rootring-r3-main-verification.md` 第2節 C751 (読み取り専用マウントがインスタンス表読めない時に確認済みバージョンを飛び越える) の縮小形態のセル（縮小形態は同一インスタンスのスイープパブリッシュが表を差し替える形で両レグにそれぞれ突かれ、再実行で一致、書き戻さない）と第3節のユーザーに渡した表の甲の行；ユーザー確定 2026-10-20「甲：インスタンス表ポインタを共有する場合のみ受ける」、原文は変更履歴にある。実験なし：数値は三方レグのモデルで計測しメイン agent の再実行で一致、甲は攻撃ゼロ回。
- ロールバック時 inode 水位はクラッシュ復旧で破棄されていないルートのみ採用（Opt2）、max 内に cur メモリ内項目を保持、ツリーID水位はリング内全ルートを採用（「水位」セル）：W3 3ラウンド判定 `research/prompts/m3-w3-r1-main-verification.md`、`m3-w3-r2-main-verification.md`（第2節攻撃側：全ルート採用時、破棄された可読高水位ルートによりロールバックが跳ね上がり W3 履歴照合が正常イメージ上で誤報）、`m3-w3-r3-main-verification.md`（第2節 Opt2 行：サンプリングと順方向推論で再発行や誤報なし、cur メモリ項目除去で再発行；コストは正当な跨ぎ再発行が11.6%増）；ユーザー確定 2026-10-08 Y1「Opt2：破棄されていないルートのみ採用（推奨）」、原文は変更史。実験なし。
- 退役ツリーは墓標・系譜ツリーと同様現行版に追従し R_old を採用しない：判定 `research/prompts/m3-retire-r1-main-verification.md` 第2節 K3 行（攻撃側サンプリング2回で不発）；ユーザー確定 X1 案A（原文は変更史）。
- 判定 `research/prompts/m4-snapshot-r1-main-verification.md` 第3節 S8（第1版で生存スナップショット存在時ロールバック拒絶；ロールバック開放時形態は0巡）および第7節ユーザー確定（分岐3「第1版でスナップショットありなら拒絶」、原文は変更史）。

**未済の検査**：C77；C314；C554；C334；C287；C126；C493；C381 —— サードパーティ第2、3ラウンドで失敗表の2判別子が指摘（プローブ書き込みはディスク上に配置先がない、バリア類の失敗箇所は「失敗配置先」すら未定義）、ユーザーは「配置先をアドレス空間表に含め、失敗箇所ごとに表を作成」を確定、第2、4問確定（D16 (パブリッシュセマンティクス) に新規確定項目を新設、失敗表は2段階判別子へ変更）、「インスタンス切り替えがメモリ内ルートを採用」は切り替え時にディスクから再読み取りして選択に決定、派生3点は審議中（`research/prompts/c381-r2-main-verification.md`、`c381-r3-main-verification.md`）；コード側（プローブ書き込み、読み取り専用再検証、インスタンス切り替え、読み取り専用移行）はマイルストーン「上書き・解放・ロールバック・再利用」収束表第16行とともに後続マイルストーンへ；C458：切り替え時のディスク再読み取りルート選択に対する赤判定テスト；C541；C558；C596；C866 (F2 のロールバックがバイト消失と内容拒否を区別しない)。

#### 確定項目 15：レコード永続化後・ルートスロット永続化前クラッシュ時、リカバリはレコードから当該パブリッシュのルートを再構築

**確定内容**：**レコードから当該パブリッシュのルートを再構築：レコードヘッダに「新ルートセグメント」を追加 = ツリーテーブルユニットポインタ 86 + 中央マッピングツリールートポインタ 86 + ツリーID水位 8 + ロールバック下限 F 8 + インスタンス表ユニットポインタ 86 + 割り当てレコードツリールートポインタ 86 + アンマウント記号 4 = 364 バイト <!-- format-const: JOURNAL_NEW_ROOT_SEGMENT_BYTES = 364 stale=新根段 188 字节|JOURNAL_NEW_ROOT_SEGMENT_BYTES = 188 -->、4096 レコードは指名項目 64 個を格納。1レコードの適用 = 選択ルートのこれら7フィールドをレコード内の値で置換（インスタンス表ポインタ、アンマウント記号、割り当てツリールートは新ルートセグメント由来とし選択ルートの踏襲を廃止）、ツリーセマンティクスは不要。1回のパブリッシュが複数レコードに分割される場合、各レコードは同一の新ルートセグメントを保持。確定項目 14 第2項「選択ルートより厳密に大なら適用」はそのまま成立。**

**射程**：「レコードが存在しルートスロットが存在しない」セルにおいてリカバリが何を適用するかを管轄；ポインタ幅は D19 (ブロックポインタ構造と幅予算) 確定項目 7 / 11 に追従。レコード適用時のポインタ層における残余の処理（アロケータ登録、オーバーレイ）は本セルの対象外であり、C284 および D28 (マウント時コミットメント量) 確定項目 2 に帰属。

**根拠**：

- E142 (新規プール新規ファイルのドライラン)：新ルートセグメントをレコードヘッダに追加後、「レコードの適用」がポインタ層で具体的定義を獲得し、ジャーナル無視とジャーナル参照のリカバリ結果が一連の状態で異なることが判明 —— ジャーナルの荷重受けが証明された。
- ユーザー確定、原文は変更史。
- ユーザー確定 2026-10-01：「複数レコードのパブリッシュでは各レコードが同一の新ルートセグメントを保持」を確定項目 15 に追加（`research/prompts/m3-inv-b-r1-main-verification.md` 第4節第3項）、原文は変更史。
- サードパーティ判定 `research/prompts/m3-ring-r3-main-verification.md` 第2節1、2行目：適用箇所のインスタンス表ポインタ、アンマウント記号、割り当てツリールートの3項目を新ルートセグメント由来に変更（初ファイル、アンマウントの2パブリッシュ補完後に生成されるルートが元来書き込まれるものとバイト完全一致、攻撃側測定；専用クリーンアップパブリッシュがレコード永続化後・ルート未永続化でクラッシュした際、インスタンス表ポインタが選択ルートを踏襲すると次回書き込み可能マウントが採番前に拒絶される、攻撃側レプリカ測定）；ユーザー確定 2026-10-02「案Aに戻る」、原文は変更史。実験なし：数値はサードパーティレプリカで実測、実験番号なし。

**未済の検査**：C284。

#### 確定項目 16：mkfs はインスタンス番号0を書き込み、初回書き込み可能マウントは1を取得

**確定内容**：**mkfs は 0 を書き込み、0 =「mkfs、インスタンス未存在」であり有効インスタンスではない；初回書き込み可能マウントは C113 確定内容 P2 に従い max(システム構成, ルートリング) + 1 = 1 を取得し、各システム構成に書き込み（システム構成スロット書き込み1回、世代番号 +1）完了した後に初めてユニットを操作。mkfs が書き出すユニット書き込み順序は (0, 0)、第0世代ルートのインスタンス番号は 0；D18 (ブロックに含める情報) 確定項目 11 の「1から開始、0は無効」は「インスタンスは1から開始、0は mkfs 用に予約」と解釈。**

- **採番手順におけるバリア**：採番時の2回のシステム構成書き込み後・本インスタンスの初非システム構成書き込み前に少なくとも1回の完了したバリアを通過、初回マウントパス上のウォームアップ開始バリアで充当可能とし別途発行は不要；そのバリア完了前に本インスタンスの全ライターはユニット・レコード・ルートの発行を禁止；いずれかのディスクで当該バリアがエラー報告 ⇒ 採番は全か無かで失敗、バリアのみ再発行して継続することは禁止。採番の max 読み取りおよびロールバック書き込みの番号は D18 (ブロックに含める情報) 確定項目 11、世代番号は D22 (ユニット原子性の合成手法) 確定項目 16 に従いディスクごとに +1。

**射程**：インスタンス番号の開始値および採番手順の順序を管轄；インスタンス表の行書き込みは D18 (ブロックに含める情報) 確定項目 11 に帰属。⚠️ 採番後、行書き込みやウォームアップパブリッシュがブロックデバイスエラーで失敗した場合、システム構成に書き込み済みの新世代番号は**ロールバックしない** —— ロールバックは採番自身のシステム構成書き込みのみを管轄（D18 (ブロックに含める情報) 確定項目 11）；この番号は破棄され、次回書き込み可能マウントは max + 1 で次を取得、破棄された番号は D18 (ブロックに含める情報) 確定項目 11 の行書き込み規則に従い行を得る。既知の挙動であり欠陥としては改修しない。

**根拠**：

- 実験なし：インスタンス番号の起点はエンコーディングの取り決め；採番バリアの判別力はクラッシュポイント再生にあり、カウント実験ではない。
- サードパーティ：`research/prompts/c322-r3-main-verification.md` 第5節（採番手順のバリア、3ラウンド論証後ユーザー確定）；インスタンス番号起点はユーザー確定。原文は変更史。
- ユーザー確定 2026-09-23：採番後の行書き込みやウォームアップ失敗時にロールバックしないことを受容、既知の挙動として記述（マイルストーン「上書き・解放・ロールバック・再利用」収束表第57行）、原文は変更史。

**未済の検査**：なし。

#### 確定項目 17：指名項目56バイトのフィールド表

**確定内容**：**1項目につき1ユニット（新規プール新規ファイル作成で12項目）：位置エントリ 14 × 2（オフセット 0）+ ユニット種別タグ 1（28）+ 誕生ツリー 8（29）+ 誕生 txg 8（37）+ key末尾セグメント 10（45：コード1は書き込みシーケンス；コード2 / コード3はインスタンス番号 4 + 誕生シーケンス 4 + ゼロパディング 2）+ flags 1（55）= 56 <!-- format-const: JOURNAL_NAMED_ENTRY_BYTES = 56 -->；ペイロードCRCは別途保持しない（位置エントリが各4バイトのチェックサムを保持）。再生処理は指名項目から直接マッピング key を組み立てる。**

- **レコードが指名する各項目は自前でチェックサムを保持**：D4 (チェックサム位置) はチェックサムを親ポインタにインライン化すると確定済みであるが、レコードが指名する対象はレコード永続化時点で親が未発行 ⇒ レコードが一時的に親の役割を代行、さもなくば再生時にユニットの完全性証明が存在しなくなる；リカバリ再生は適用前に指名ユニットを1点ずつ検証必須（確定項目 14 第4条）。
- **1回のパブリッシュがN件のトランザクション・N件のレコードに分割される際、パブリッシュ共有のコミット内生ブロック（extentツリーノード、inodeリーフコンテナおよびルート、割り当てツリー、会計ツリー、中央マッピングツリー、ツリーテーブルユニット）は最終レコードでのみ指名**、先行レコードで重複指名しない。
- **共有コミット内生ブロックが最終レコードの収容能力（4096レコードで64項目）を超過する場合、末尾レコードが複数レコードに跨走**：最終トランザクションレコードから順次追加レコードを書き込み、満杯にしてから次を開き、真の最終レコードのみが「今回のパブリッシュ末尾」フラグを保持；リカバリはこのフラグによりパブリッシュ境界を認識し、フラグ付きレコードが未読ならパブリッシュ全体を不適用（確定項目 14 第5条）。フラグはレコードヘッダ10フィールドの旧「パディング 1」バイトに配置（「レコードフラグ 1」に改名）、ビット0 = 今回のパブリッシュ末尾；1件のみのレコードで構成されるパブリッシュ（空パブリッシュ含む）もビット0に1を書き込み —— 新規プール新規ファイル作成のバイト列を変更（w1、w4、t13 の各レコードの同バイトおよびヘッダチェックサム [46, 50)；w4、t13 の逆方向チェーン [91, 95) も連動変更、E142 (新規プール新規ファイルのドライラン) G26 でバイト単位照合済み）。ユーザー 2026-09-24 決定。

**射程**：指名項目のバイト列を管轄；位置エントリの構成は D19 (ブロックポインタ構造と幅予算) 確定項目 4 / 11、マッピング key の形態は D19 (ブロックポインタ構造と幅予算) 確定項目 6 / 10。⚠️ 共有コミット内生ブロックを最終レコードでのみ指名する場合、最終レコード永続化前にクラッシュするとそれらのユニットはいかなるレコードにも指名されない —— 再生の回収パスが本ケースを認識可能でなければならず、クラッシュポイント再生の小ワークロードがこれをカバー必須（C491）。

**根拠**：

- E16 (ジャーナルの役割：WAL vs インテントログ)：レコードが指名する対象の親はその時点で未永続化、レコードが一時的に親を代行必須 ——「指名項目ごとのチェックサム保持」を要請。
- E77 (発行の永続順序)：指名項目がチェックサムを保持し適用前に1点ずつ検証することが、ユニットとレコード間のバリアを省略可能とする交換条件。
- ユーザー確定、原文は変更史。
- ユーザー確定 2026-09-23：共有コミット内生ブロックは最終レコードでのみ指名、原文は変更史。
- ユーザー確定 2026-09-24：末尾レコードの複数跨ぎ、最終レコードへのフラグ付与（原文は変更史；サードパーティ `research/prompts/m2-treesplit-r1-main-verification.md` T6 (拒絶維持時に拒絶するか否かはプールの履歴に依存する)、確定前の字面で末尾を識別すると半端なパブリッシュが適用される、フラグ付きセル 0 / 0）。

**未済の検査**：C376。

#### 確定項目 18：インフライトレコード数上限の値と意味論、tail フィールドのエンコーディング

**確定内容**：インフライトレコード数上限 = リングスロット数 ÷ F（`F` は I-8.1 (リング幾何が十分大きい) の安全係数）、デフォルトリング下で 196608 ÷ 3 = **65536**、システム構成に書き込み；tail 8バイトは jsn の48ビットカウンタを格納し、システム構成自身の2スロットローテーションにより冗長性を確保、専用の tail スロットは設けない；レコード n はリング内オフセット `(カウンタ − 1) mod スロット数 × 4096` に配置。

- **意味論**：この数値の意味は**1回のリカバリ再生におけるプレフィックスが最大Nレコードであること**。チェックサム不合格のレコードを読み取った場合、リカバリは**番号途絶で即停止** —— 直前レコードで停止し、「テア」か「破損」かを追及せず、読み取り専用マウントを拒絶することもない；書き込み可能マウントは別途 C554 乙-配置続に従い判定（確定項目 14）：システム構成が途絶箇所より新しいパブリッシュを目撃済みで再読み取り結果も真なら書き込み可能を拒絶。XFS の「直近Nレコード内の不正チェックサムはテアとみなし、それ以外は破損とみなしてマウント拒絶」とは異なる。

**射程**：⚠️ コスト：真の「ディスク中間破損」と「末尾テア」は第1版で同一の結末として判定され、プールは通常通りマウントされ破損レコード以降は全数不適用となる。第1版のシリアルコミット（D13 (検証戦略) 確定項目 2）下では任意の瞬間でインフライトレコードは恒常的に1件 ⇒ XFS の要求する「N = 並行書き込みレコード数」と「リングスロット数 ÷ F」は次元の異なる数値であり、同一フィールドに2つの意味論を共存させることは不能。

**根拠**：

- 実験なし：値は I-8.1 (リング幾何が十分大きい) の算術（リングスロット数 ÷ F）、意味論は計測条件の裁定 —— 番号途絶での停止は既に I-8.3 (厳格に連続する再生プレフィックス) の判定基準であり、本条は「チェックサム不合格」を同一の出口に統合したのみで新規分岐はなし。
- ユーザーモーダル確定、サードパーティ論証未実施、覆すことは可能、原文は変更史。

**未済の検査**：なし。

#### 確定項目 19：空パブリッシュレコードのトランザクション番号とコミットマーカー、previous_hash、リング長

**確定内容**：

- **① 空パブリッシュレコード**：トランザクション番号0はトランザクションを担持しないレコード用に予約、コミットマーカーは1を書き込み；インスタンス表 W の max に算入せず、「コミットマーカー未出現の末尾破棄」判定からも除外。
- **② `previous_hash`** = CRC32C(**本インスタンス内の論理的直前レコード**のヘッダ、`header_csum` の32バイトはゼロとして関与)；**本インスタンスで書き出される初レコードは恒常的に 0**；書き込みパスは**ディスク読み取りを行わない**。逆方向チェーンは**インスタンス別**である —— I-8.3 (厳格に連続する再生プレフィックス) の逐字「インスタンス境界跨ぎで即停止」に従い、再生は元来あるインスタンスのレコードから別インスタンスのレコードへ接続することはなく、インスタンス境界には別途インスタンス番号の防御が存在；「リング上の直前スロットのレコードヘッダ」を採用すると、書き込みパスでレコードを書き込むたびに直前スロットを読み戻す必要が生じ、「インスタンスを跨いでも接続可能」を獲得するだけであり、そのチェーンはどの判定にも使用されていない。バイト表で w1 の行に 0 を書き込む理由は「本インスタンス初レコード」であり、「リング上に先行レコードなし」ではない。
- **③ リング長**：リング長は **mkfs パラメータ**、**デフォルト 768 MiB**；制約 **リング ≤ デバイス容量 ÷ 4**、超過時は **mkfs 拒絶**。システム構成には依然として「リング長（バイト）」という単一フィールドのみを格納し、容量や比率は別途格納しない。**マウント時の有効 `T_dirty` = min(`T_dirty` フィールド, リング長 ÷ F)** —— システム構成の `T_dirty` には依然として 2 GiB を書き込み、デフォルト 768 MiB リング下での有効値 = min(2 GiB, 256 MiB) = 256 MiB。ユニット領域の開始スロット番号はリング長に連動、デフォルトリング下では 784 MiB（スロット 50176）。`T_dirty` のクランプは I-8.1 (リング幾何が十分大きい) の直接の帰結 —— リングが最悪占有量の `F ×` を収容できない場合、ダーティ許容バイト数を追従して引き下げなければパブリッシュ不能に陥る。

**射程**：③ の「パラメータ + 上限」により大容量ディスク上で 768 MiB は数パーセントを占めるのみとなり、小規模イメージでは縮小可能、`リング ≤ 容量 ÷ 4` によりリングが全ディスクを消費することを防止；⚠️ 連動事項：`.claude/kb/layout/01-first-txn.md` の想定イメージは **4 GiB**（1 GiB イメージでは本制約下で 768 MiB リングを収容不能）。実装：`crates/singlefs-core/src/make_filesystem.rs` がリング長パラメータを受け取り、最小ディスク容量 ÷ 4 に基づき超過を拒絶、デフォルト値は `crates/singlefs-format/src/lib.rs` の `JOURNAL_RING_DEFAULT_BYTES`。マウント読み取り側でもリング長を判定：インフライト上限（リングスロット数 ÷ F、確定項目 18）≥ 1、かつリング終端がコンパイル時ユニット領域起点（784 MiB、`crates/singlefs-format/src/lib.rs` の `UNIT_AREA_START_SLOT` と `JOURNAL_RING_START_SLOT`）を超過しないこと；超過時はプール全体でマウント拒絶（実装レビュー A3a 着地：`crates/singlefs-core/src/recovery.rs` の `SystemConfigurationValueOutsideWhatThisReaderAccepts::JournalRingBytesOutsideTheSupportedRange`）。

**根拠**：

- E143 (1トランザクション1ユニット下でのジャーナル記帳)：分割規律に基づき、`T_dirty` 満杯ウィンドウに必要なリングは当初想定 64 MiB リングの4倍となり、リング ≥ 768 MiB（F = 3）とするか `T_dirty` を圧縮する必要がある —— デフォルトリング長と `T_dirty` クランプの出所。
- ①、②、③ はいずれもユーザーモーダル確定、サードパーティ論証未実施、覆すことは可能；メイン agent 推奨値も同一、原文は変更史。

**未済の検査**：なし。

#### 確定項目 20：レコードはポインタ層の目標状態を担持、割り当てはジャーナルに先行

**確定内容**：**ジャーナルレコードは「ポインタ層の目標状態」を担持し、論理インテントでも、完全な物理REDOでもない。** ユニットはコピーオンライトによりそれぞれの最終配置先に書き込まれ自前でチェックサムを保持（D22 (ユニット原子性の合成手法) 確定項目 18 における「COW + D4 (チェックサム位置) で十分」のクラス）；ジャーナルは「どのサブツリールートがどのブロックへ移動したか + 新ルート」のみを記録する。

- **これより書き込みパスの厳格順序を導出：割り当てはジャーナルに先行する。** ジャーナルレコードは「トランザクションの全体」ではなく、「**ディスクに永続化済みの要素に対するパブリッシュ命令**」である。
- 判定基準：

  | 判定基準 | 論理インテント | 完全な物理REDO | **ポインタ層目標状態** |
  |---|---|---|---|
  | 冪等性（I-4.4 (インテントは目標状態のみ記述)「目標状態のみを記述」） | 目標状態として記述された場合のみ冪等 | 生来的に冪等 | **生来的に冪等** |
  | リプレイにアロケータが必要か | **必要** | 不要 | **不要** |
  | レコードサイズ（D16 (パブリッシュセマンティクス)「小さく、順次書き込み」） | 小 | **大**（全バイトを保持必須） | **小** |
  | レコード照合器（D13 (検証戦略) 確定項目 7）がレコード照合に要するもの | ツリーセマンティクス + 会計 + 割り当て正当性を独自実装（照合が構文から意味論へ変化） | バイト単位の突き合わせ | **フォーマット解析 + チェックサム + ルートリングでの最新選択** |

- **論理インテントを却下する第1の理由（独立して成立）**：O2 (独立パーサー + checker) は被テスト crate のいかなるコードの呼び出しも明令禁止されており（D13 (検証戦略)）、一方論理インテントの再生はアロケータ + ツリー操作 + チェックサム + 会計の再実装を要し、**実質的に第2のファイルシステムに等しい**。第2の理由（配置先の不確定性により照合が退化）は第1の理由の穴塞ぎであり、独立した理由ではない（確定項目 5）。
- レコード形態と D13 (検証戦略) の検証手法は相互に人質関係にある：検証手法の実現可能性がレコード形態を決定し、レコード形態が検証手法の産出等級を決定する；両者は同時に確定必須。

**射程**：レコードに何を格納するか、割り当てと記録の先行順序を管轄；レコード適用時にポインタ層でどのフィールドを置換するかは確定項目 15、残余は C284 に帰属；再生の冪等性というハード要件は確定項目 3。

**根拠**：

- 実験なし：方向性は2026-08-28サードパーティ論証が冪等性、再生時のアロケータ要否、レコードサイズ、照合要件の4判定基準に基づき導出（メイン agent 裁定）、4点とも構造的性質であり測定可能な対照はない；ZFS の ZIL が「主書き込みパスを再呼び出し」して独立実装コストを回避しているのは手がかりであり根拠ではない、変更史に記載。

**未済の検査**：C284。

#### 確定項目 21：世代番号：レコード粒度のグローバル単調番号であり、D5 (スナップショット / 空間会計メカニズム) の txg ではない

**確定内容**：**レコードヘッダは責務の異なる2つのカウンタフィールドを必要とし、統合は不可**：

| フィールド | 粒度 | 用途 |
|---|---|---|
| `jsn` | **レコードごと**、グローバル単調増加、10バイト（インスタンス番号32ビット + カウンタ48ビット、確定項目 9）| D22 (ユニット原子性の合成手法) 要件②の世代番号；プレフィックス判定がこれに依存 |
| `checkpoint_txg` | チェックポイントごと | どのチェックポイントに帰属するかを示し、D5 (スナップショット / 空間会計メカニズム) の birth/death 会計に使用 |

- 理由：D5 (スナップショット / 空間会計メカニズム) は「1チェックポイントに1つの txg 番号」⇒ 同一チェックポイントウィンドウ内の複数ジャーナルレコードは**同一の txg を共有**し相互に前後を識別不能となり、D22 (ユニット原子性の合成手法) が要求する「実際に検査される世代番号」が意義を失う。
- ⚠️ jsn がラップアラウンドしない幅を採用したことは、**「ラップアラウンド比較の免除」を購入したものであり、新能力ではない**：jbd2 の tid は32ビットでラップアラウンドし、`tid_gt` による符号付き差分比較に依存；ラップアラウンド比較こそ独立実装間で最も差異が生じやすい箇所である。
- ⚠️ **「txg を用いれば cycle は不要」は軸①に対してのみ成立**、D22 (ユニット原子性の合成手法) の直交2軸の表を参照。

**射程**：2つのカウンタの責務を管轄；水位とルート選択における両者の主副関係は確定項目 14 第2項に帰属；「毎回のパブリッシュで checkpoint_txg + 1」は D16 (パブリッシュセマンティクス) 確定項目 6 に帰属。

**根拠**：

- 実験なし：責務分担は D5 (スナップショット / 空間会計メカニズム) の粒度から直接導出され、定義間の演繹であり、測定可能な量はない。

**未済の検査**：なし（C85 は 2026-10-11 に清算済み、C88 に統合、未済の検査清算済み表を参照）。

#### 確定項目 22：プレフィックス判定：厳格連続、番号途絶で即時停止

**確定内容**：**プレフィックス判定基準は `jsn == expected` であり、不一致で即時停止。**「`jsn` > 前回適用済み」によるフィルタリングのみでは**不十分**：シーケンス内に空洞が存在する場合（5849成功、5850未書き込み、5851成功）、「>」は5851を5849の直後に誤接続し、**因果順序を破壊する**。さらに**明示的なインフライトレコード数上限**を必要とする（値と意味論は確定項目 18）。プレフィックス判定の完全な基準（5条）は確定項目 14。

**射程**：⚠️ 番号途絶での停止は**その回**のリカバリにおいて正しく（プレフィックスは5849で停止）、それ単独では**前回のタイムラインの残留**を防御不能：リカバリ後に継続書き込みを行う際、スロット5851には前回のタイムラインのレコードが依然として存在し、**チェックサムは良好で `jsn` は偶然 expected と一致** ⇒ 2回目のクラッシュ後のリカバリがそれを接続して再生。D23 (ジャーナルの役割と形式) で唯一測定されたスロットマッピング（`slot = jsn % スロット数`）下では構造的に必然発生し、配置と番号の分離時は偶然に退化するが分離では救済不能。既存の3ゲート（I-8.3 (厳格に連続する再生プレフィックス)、インフライト上限、同一ウィンドウで完全一致する `checkpoint_txg`）はいずれもトリガーされない；tail の前進は解決策とならない（tail はチェックポイント永続化後にのみ前進し残留レコードは適用されたことがない ⇒ `tail ≤ min(残留 jsn)` が恒常成立）；再生の冪等性も救済不能（自反的：「**同一レコード**を2回再生した結果は1回再生した結果と同一」、残留レコードは**別のレコード**であり、指名ユニットは破棄後に新タイムラインで再割り当て済みであり得る）。確定項目 8 / 9 / 10 がこれを防御：逆方向チェーン、シーケンス上位ビットのインスタンス番号、チェーン値のインスタンス番号網羅；リカバリ時に残留スロットを物理的に無効化するにはインプレース上書きを要し、zoned では不成立。

**根拠**：

- 実験なし：「>」が空洞後のレコードを空洞前に誤接続することはシーケンス定義上の反例であり、測定不要；残留セルの実験は確定項目 8 の根拠を参照。

**未済の検査**：C42。

#### 確定項目 23：3つのデッドロックの個別打破法

**確定内容**：ジャーナルに関連する3つのデッドロックのうち、2つはフォーマットに起因し事後補正不能：

| # | 循環 | 本プロジェクトにおける扱い |
|---|---|---|
| 1 | ジャーナル満杯 → チェックポイント → ルート発行 → **ルート発行時にジャーナル書き込みが必要か** | D16 (パブリッシュセマンティクス) の「ジャーナルを迂回しない」は解釈案Aを採用（確定項目 25）：ルート発行自体はジャーナルの追記ではない |
| 2 | **COW 固有**：チェックポイント → 新ノードをCOW書き込み → 割り当てが必要 → ディスク満杯 → ルートを発行できない | **リザーブプールがこれに対処**、`reserve ≥ ckpt_cost` で十分（確定項目 24）。⚠️ **D22 (ユニット原子性の合成手法) の K がこれを悪化させる**：ディスクがほぼ満杯のとき最も救命に必要な領域がまさに K により抑留されている ⇒ **K はランタイムポリシーでなければならず、フォーマット定数としてはならない** |
| 3 | 単一操作がリングより巨大 | 等価な幾何的不変条件が必要：**任意の操作の最悪ジャーナル占有量 ≤ リングサイズ / F、F ≥ 2**（[invariants/03-journal.md](../../../crates/singlefs-checker/invariants/03-journal.md) I-8.1 (リング幾何が十分大きい)）、mkfs 時に計算、mount 時に検証 |

- **デッドロック3から依存関係を導出**：**D8 (中核インデックス構造) のインテントバッチ分割機構は「巨大ファイル削除の便宜措置」にとどまらず、ジャーナル満杯デッドロック回避の必要条件の1つである** —— 予算に収まらない操作は個別合法な複数のトランザクションに分割可能でなければならない。

**射程**：これら3つの循環の遮断手法を管轄；リザーブプールと受入の分治は確定項目 24、解釈案Aは確定項目 25、リング幾何の具体的数値は確定項目 18 / 19。I-8.1 (リング幾何が十分大きい) の「占有量」は空間次元であり、1トランザクションの複数レコード跨ぎを許容（確定項目 7）。I-8.1 の checker は実装済み（プールレベル checker の `judge_journal_ring_geometry_fields_agree`、core リカバリ時に3等式を合同判定、`crates/singlefs-core/src/recovery.rs`）。デッドロック2はコピーオンライト固有：インプレース書き込みの jbd2 / XFS は帰巣に追加空間を要さず、本循環は存在しない。

**根拠**：

- E23 (ジャーナル幾何)：固定長リングにはデッドロック3が存在し幾何的不変条件を要する、リングが過小なら強制チェックポイント回数が急上昇；チェーン式にはデッドロック3がないがジャーナルブロックごとにアロケータ要求を要しデッドロック2の循環に突入。
- E19 (deferウィンドウ下の疑似ENOSPC)：リザーブプールが1回チェックポイントのコストと正確に一致することでチェックポイント行き詰まりが0に収束 —— デッドロック2はリザーブプールにより遮断。

**未済の検査**：なし。

#### 確定項目 24：デッドロック2と疑似ENOSPCは別個の障害であり個別対処を要する

**確定内容**：

| 障害 | 何が解決できるか |
|---|---|
| **チェックポイントの行き詰まり**（D23 (ジャーナルの役割と形式) デッドロック2） | **リザーブプール**、かつ `reserve ≥ ckpt_cost` で十分；本プロジェクトの ckpt_cost は D28 (マウント時コミットメント量) 確定項目 4 に従い現時計算（各記録ツリー高の合計 + 会計ノード）、定数は設定しない |
| **疑似ENOSPC** | **受入のみ**。リザーブプールはかえって悪化させる |

- **メカニズム**：リザーブプールはチェックポイント用に設けた下限であり、**通常の割り当て**に対しては純粋な税金である —— リザーブプールが行き詰まりを防ぐのに十分大きいことが前提；前提を満たさない場合は符号が反転し、行き詰まりにより領域回収が停止するため小さなリザーブプールの方がかえって拒絶を増加させる。疑似ENOSPCの成因は「defer ウィンドウとルートリング K が抑留する量が残余領域を超過すること」であり、**リザーブプールの射程外にある**。
- **疑似ENOSPCの解決策は算出可能な受入規則である**：

  > **残余領域 > チェックポイントごとの撹乱量 × (遅延世代数 + 1) + 1回チェックポイントのオーバーヘッド**

  ⇒ ディスクを `容量 − 撹乱量 × (遅延+1) − チェックポイントオーバーヘッド` 以上に充填してはならない。D3 (領域割り当て) 確定項目 15「空間予約は受入制御形態を採用」と同型 —— **「最悪状況」には defer ウィンドウと K が抑留する部分を含めなければならない。**

**射程**：リザーブプールが受入不等式にどう算入されるか、マウント時コミットメント量に算入されるかは D28 (マウント時コミットメント量) 確定項目 4、受入不等式の権威形式は D28 (マウント時コミットメント量) 確定項目 1。⚠️ **撹乱量は実負荷において変数である**が、モデル化時は定数として扱う；その見積もり手法は実機ストレステスト側の課題。

**根拠**：

- E19 (deferウィンドウ下の疑似ENOSPC)：予約が1回チェックポイントのコストと正確に一致することで行き詰まりを0に抑制；予約をさらに拡大すると疑似ENOSPCが一定ステップで上昇するのみ；受入規則の予測と実測値はマスごとに完全一致。
- E25 (リングとチェーンで各々必要なリザーブプールサイズ)：「リザーブプールは通常割り当てへの純税」はリザーブプールが行き詰まりを防ぐのに十分大きい場合にのみ成立、行き詰まり領域内では小さなリザーブプールの方がかえって拒絶を増加。

**未済の検査**：C83。

#### 確定項目 25：D16 (パブリッシュセマンティクス)「ジャーナルを迂回しない」は解釈案Aを採用

**確定内容**：D16 (パブリッシュセマンティクス) の

> 「いかなる操作もジャーナルを迂回して直接ルートを発行することはできない。」

という記述について、**解釈案Aを採用する**：

| 解釈案 | 意味 | 結果 |
|---|---|---|
| **案A（採用）** | 最終的にルートに反映されるいかなる**セマンティクス上の効果**も、あらかじめジャーナルを経由必須；**ルート発行という動作自体はジャーナルの追記ではない** | デッドロック1を打破 |
| 案B（字面通り） | ルート発行もジャーナルを経由 ⇒ ルート発行にレコード追記が必要 ⇒ ジャーナル満杯時にルートを発行不能 | **循環が閉塞し、デッドロック** |

⇒ D22 (ユニット原子性の合成手法) と整合：**ルートリングはジャーナル外の固定 K スロットであり、ルート発行 = ルートリングの1スロットの上書きであって追記ではない。** チェックポイントは「操作」ではなくジャーナル機構自体の構成要素；通常のユーザー操作は依然としてジャーナルを経由しなければならない。

**射程**：D16 (パブリッシュセマンティクス) の当該文言の解釈のみを管轄；パブリッシュの残余のセマンティクスは D16 (パブリッシュセマンティクス) に帰属。

**根拠**：

- 実験なし：文言の解釈裁定であり、2つの解釈のうち一方はデッドロック循環を直結させ、測定可能な量はない；jbd2 / XFS の解決策（空間解放パスはログ領域をゼロ消費）は解釈案Aと整合、手がかりとして変更史に記載。

**未済の検査**：なし。

#### 確定項目 26：レコードヘッダの2つのパース規律

**確定内容**：

1. **チェックサムがカバーするバイト範囲は明示的なディスク上定数でなければならず、言語の `sizeof(struct)` から導出してはならない。** 独立パーサーが同一の `sizeof` を偶然再現することはなく、これこそが checker が捕捉可能で単一実装が見落とすエラー種別である。
2. **レコードの自己記述長は必ず境界チェックを実施した後に、チェックサム計算に使用しなければならない。**

逆方向チェーンに関する規則（レコード照合器が順方向走査とは完全に独立したトラバースで同一プレフィックス境界を再検証可能とすること）は確定項目 8。

**射程**：ジャーナルレコードヘッダのパース順序および定数の出所を管轄；カバー範囲自体（全体 `[0, 4096)`）は確定項目 13；確定項目 1 のB条（自己記述長、境界チェック後にチェックサム計算）は第2条と同一。

**根拠**：

- 実験なし：2条とも実運用実装の欠陥から現時調査された規律（XFS は `sizeof` を使用したため i386 と他アーキテクチャで異なる CRC を算出し、`XLOG_REC_SIZE` / `XLOG_REC_SIZE_OTHER` の2定数と2周チェックという互換性の傷痕を負った；ZFS の ZILOG2 はブロックから `zc_nused` を読んでチェックサム範囲を決定し、読む前に `if (size < nused) return ECKSUM` を実行）、パース順序と定数の出所を制約するものであり測定可能な量はない。

**未済の検査**：なし。

```

**出处 `.claude/kb/decisions/23-ジャーナルの役割と形式.ja.md:692-695`（整段抄，未转述）**

```markdown
## 履歴

D23 (ジャーナルの役割と形式) の履歴条目は [decisions-history.md](../decisions-history.md) に集約。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:1-15`（整段抄，未转述）**

```markdown
## D23 journal 的角色与格式 —— 已定

**中文** · [English](23-journal-role-format.en.md) · [日本語](23-ジャーナルの役割と形式.ja.md)

<!-- doc-lint:not-numbers CRC-64 SSE4.2 CRC32C SHA-256 CJ2 -->

D23（journal 的角色与格式） 管 journal 是什么、写什么、崩溃之后怎么用它：记录承载什么与分配的先后（已定项 20）、每次 fsync 写什么（已定项 1）、环的几何、记录尺寸与在飞上限（已定项 2 / 12 / 18 / 19）、tail（已定项 3 / 18）、记录头与点名项的字节（已定项 4 / 7 / 8 / 9 / 10 / 11 / 13 / 15 / 17 / 21）、前缀判定与重放下界（已定项 14 / 22）、管理员回退、失败处置与实例切换（已定项 14 / 16）、与 journal 相关的三个死锁和保留池（已定项 23 / 24 / 25）、记录头的解析纪律（已定项 26）。不管的：发布的持久顺序、checkpoint 的触发与暖机归 D16（发布语义）；实例表的行格式与写行规则归 D18（块里携带什么信息） 已定项 11；根记录与根环归 D22（单元原子性怎么合成）；保留池与切换预留怎么进准入不等式归 D28（挂载期承诺量）；checker 与记录核对器各是什么归 D13（验证路线） 已定项 6 / 7；目标负载归 D25（目标负载优先级）。

<!-- doc-lint:registry name-col=2 -->

| 编号 | 简称 |
|---|---|
| T6 | 维持拒绝时拒不拒取决于池的历史 |
| L8 | 并行线三跨记录崩溃点重放流 |

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:47-92`（整段抄，未转述）**

```markdown
#### 已定项 1：journal 的角色本身

**定案**：**取甲：每次 fsync 写脏叶 + 全部祖先 + 根槽 + 一条记录。祖先不延后。** 脏叶是全部脏叶，不只被 fsync 的那个文件的：fsync = 一次全量发布。

- **两根轴不正交**：轴一（每次 fsync 发不发根）取「发根」之后，轴二（祖先落盘延不延后）只剩一个合法值。**延后祖先 ⇒ 新根不可能自洽 ⇒ 发不出一个反映本次 fsync 的根。** COW 下，要让本次 fsync 的叶从根可达，整条脊柱必须已在盘上；而「自洽」是 [invariants/02-根环.md](../../../crates/singlefs-checker/invariants/02-根环.md) I-7（根环） 类的要求（从代号最大的根递归校验能走完权威态）。2×2 只有三格可实现：

  | | 不延后祖先 | 延后祖先 |
  |---|---|---|
  | **每次 fsync 发根** | **甲** | **不存在** |
  | 只在 checkpoint 发根 | `wal_full` | 乙 / 丙 |

  ⇒ **轴一已定的「每次 fsync 发根」单独就把乙与丙一起判掉了**，且轴一那四条依据（环截断不了 / 多一条挂载期重放路径 / journal 进验证链 / 屏障一个没省）对乙**逐条原样成立**，不需要重新论证。丙（E22（记录点名祖先落点） 那个形态）同样不写祖先内容，只在轴一被重开时才有意义。
- **甲下 redo 为空**，不是「甲下恢复无动作」：恢复期仍有「续做未完成意图」这条常规路径（[checks-owed.md](../checks-owed.md) C1（意图续做做成常规路径））。
- **记录头的类型字段就是必需的，与轴二日后重不重开无关。** 类型字段的登记表（登记位 D22（单元原子性怎么合成） 已定项 17）：0 无效（判损坏）、1 普通记录；新类型只追加不重用码，未知类型按 A 条拒绝挂载。把它写成「为轴二留的期权」会让人以为付了钱、买到了轴二的可逆性——**它一分钱也没买到那个。**
- **真正让轴二可逆的是四项，缺一不可**：

  | # | 预留什么 | 为什么 |
  |---|---|---|
  | A | 记录头**类型字段** + **未知类型 ⇒ 拒绝挂载**，且该政策由**系统配置 incompat 位**承载 | 光有字段而政策是「跳过不认识的记录」，失败模式是**静默漏放一条发布指令**，比没有字段更糟 |
  | B | 记录**自述长度**，且**先边界检查再算校验和** | 类型字段只说「这是什么」，不说「这有多长」。丙的记录长度正比于树高，装不进按甲的尺寸钉死的预算 |
  | C | **journal 几何进系统配置**（环大小、最大记录字节、在飞记录数上限），**不许是代码常量** | 换轴二 ⇒ 最坏记录尺寸变 ⇒ 死锁 3 的几何不变量要跟着变。焊进代码就只能重建文件系统 |
  | D | 项的大小**由记录类型决定** | D9（加密）已定 MAC 满 128 位不截断，丙的项更宽 |

  ⚠️ **A 与 B 本来就欠着**（B 是已定项 26 的第二条），把它们记成「为轴二付的钱」是重复记账。
- **重开轴二的可判据。** ⚠️ **这是设计期判据，不是运行期开关。** `.claude/rules/fs-design.md` 硬要求 5「分支变量不许在操作中途改变」——把「延不延后」做成按负载切换的运行时分支，会让崩溃语义在一次操作中途改变，当场违规。**轴二只能整体切换，且走 incompat 位。**

<!-- doc-lint:registry name-col=2 -->

| 闸 | 简称 | 判什么 | 测得了吗 |
|---|---|---|---|
| G23.0 | 轴一被重开 | 轴一被重开（否则后面几条不必看） | — |
| G23.1 | 祖先链占比达标 | 单流同步写（QD=1、单线程）的 fsync 服务时间里，「写祖先链 + 根槽」占比 ≥ X%，**X 在实验注册时声明，不许事后定** | **不能**，E16（journal 的角色：WAL vs 意图日志） 口径明写只算块数不算时间。前置：事务层 + 真机 harness |
| G23.2 | 甲丙差不收敛 | 攒批变大时，甲与丙的差**不**收敛 | **「差」取甲 / 丙每批块数之比**（用户 2026-09-28 定；用乙的数代丙）：E16（journal 的角色：WAL vs 意图日志） 第五次跑显示，按 2026-10-04 的宽度收不收敛随树高变，不是「批 ≤ 流数」的界——真实基线 1 TiB（4 层、根下 11 个孩子）上流数 ≤ 16 先涨后收敛（峰值在批 ≈ 流数），流数 ≥ 24 从批 = 1 起就收敛（`research/results/e16-fifth-run-peak-under-today-widths-2026-10-04-h487.out` 里流数 64 那一行：`name=cell` 第 1882、1904 行，批 1 到批 64 比值从 3.500000 降到 3.169231，它的 `row_summary` 在第 2405 行）；树高 ≤ 3 全部流数从批 = 1 起收敛，树高 ≥ 5 扫到的流数全部先涨后收敛，树高 4 时门槛约为 2 × 根下孩子数（只在三方攻方副本上量过，主 agent 复跑逐字节相同，入库装置没重做，判决 `research/prompts/d23-e16r5-r1-main-verification.md` 第一节汇总表）。「3.40× → 3.71×」那组数是旧宽度（扇出恒 128、树高恒 4）下的。新写法被攻过零轮。⚠️ 丙尚未测 |
| G23.3 | 四项盘上齐备 | 第 A–D 四项在盘上齐备 | **能，现在就该定** |

- **反向判据（D23（journal 的角色与格式）必须自带）**：若记录核对器（D13（验证路线） 已定项 7：需要第二个输入的核对归它）上线后，累计 N 次全量验证里「记录核对器抓到而 checker + O1（内存模型对拍（HashMap 理想文件系统）） + 崩溃点重放三者都没抓到」的事件数仍为 **0**（N 在上线时先声明），则「乙毁掉记录核对器的逐字节比对」这条依据的增量价值实测为零，轴二应当重开评估。没有这一条，那条依据就是不可证伪的。

**射程**：管每次 fsync 写什么、轴二能不能重开；目标负载归 D25（目标负载优先级）。定案只压在「两根轴不正交」这一条结构性依据上，与收益多少无关；定案当时给出的三条依据都降了级、不承重：「乙毁掉记录核对器的逐字节比对」暂不承重，「乙买不到『用错了变成挂不上』」射程用错撤下，「乙的收益形状」只剩奖金上界。乙的收益只在单流同步写上存在，而那恰是数据库与虚拟机镜像（D10（随机写 / 碎片化的真答案） 记为 COW 文件系统的共同软肋）⇒ 只能说「乙的奖金有界」，不能说「乙的收益不值一提」。轴一的写放大前提在里程碑「覆盖写、释放、回退与复用」增补 1 的三方第一轮被打中，第一轮那处打中在第二轮被翻掉（记录头里 364 字节的新根段在第二轮时已经在，施加就是换七个字段，所以「wal_full 自己改不掉」只对「每次 fsync 另写一个树表单元」那一形成立），两轮判完的结论是三条岔路收成甲、正文不动（`research/prompts/m2-s1-r1-main-verification.md`、`m2-s1-r2-main-verification.md`）。

**依据**：

- E16（journal 的角色：WAL vs 意图日志）：延后祖先的臂只在 checkpoint 发根、从没在 fsync 时发过根；攒批吃不掉延后祖先的收益随树高变（第五次跑：树高 ≤ 3 全部流数从批 = 1 起收敛，树高 4 门槛约为 2 × 根下孩子数，树高 ≥ 5 扫到的流数全部先涨后收敛）——它改的是「乙的收益形状」那条已降级依据的强度，不是结论。
- 三方：2026-08-28 三方论证（材料 `_axis2-background.md`），正推腿给出「轴一 ⇒ 轴二」这条结构性依据，三条原依据怎么降级、腿况在变更史。

**欠**：反向判据的 N 没有声明，记录核对器已接在崩溃点重放上（`crates/singlefs-checker-tier/src/crash.rs` 的 `check_records`），事件数没人计。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:93-117`（整段抄，未转述）**

```markdown
#### 已定项 2：journal 是定长环还是链式

**定案**：**取定长环。** ⚠️ 候选理由「链式把 journal 写路径变成分配路径，于是进入死锁 2」过强，不采用：链式的增量是有界可算的，抬保留池就断开——它不是一个真环。真正判掉链式的是三条，都不经过死锁 2：

1. **保留池必须涨一个量级，而涨保留池会加重假性 ENOSPC。** 链式下 journal 块本身要分配 ⇒ 保留池必须从 `ckpt_cost` 涨到 `ckpt_cost + 一个 checkpoint 窗口内的 journal 块`；而保留池对普通分配是纯税，假性 ENOSPC 只有准入治得了（已定项 24）⇒ **链式不是把代价变成死锁，是把它变成假性 ENOSPC。**
2. **前缀判定与空间归属被绑死。** 定长环下「有效前缀到哪」只决定重放哪些记录；链式下还决定**哪些块分配器可以发出去**——前缀规则与 D21（权威态与派生态的分界）的记账权威是两个独立判定，却必须逐块一致，不一致就是静默泄漏或双分配；断号即止的截断点之后的链块要么泄漏、要么被重发而重放仍指着它。
3. **死锁 3 不是被消掉，是从静态判据换成动态判据——这条是决定性的。**

   | | 判据 | 什么时候算得出 |
   |---|---|---|
   | 定长环 | 最坏操作 ≤ 环大小 / F，F ≥ 2 | **mkfs 时算、mount 时校验**（同 XFS `XFS_MIN_LOG_FACTOR`） |
   | 链式 | 最坏操作 ≤ **当前空闲** | 算不出——**当前空闲在操作中途会变** |

   `.claude/rules/fs-design.md` 硬要求 5 逐字写着：「**只在操作期间不会变的输入上分支**……「当前填充率」会在操作中途越过阈值，**那是危险的分支变量**」。⇒ **链式把一条可判定的几何不变量，换成了本工程明令不许用的运行期变量。** 这不是权衡，是撞上已定的硬要求。

**射程**：管 journal 的空间形态。与已定项 3 耦合：定长环下 tail 答错**只是多做功**（可降级），链式下是**泄漏或双分配**，tail 必须是权威态、要 tail-before-reuse 硬序、FUA 落在关键路径；**内联 tail + 链式是最坏格**（内联只能搭记录便车推进，链式下空闲期不写记录 ⇒ journal 块一直回收不了，空转变成空间泄漏）。⚠️ 环这一侧的几何不变量已写成 [invariants/03-journal.md](../../../crates/singlefs-checker/invariants/03-journal.md) 的 I-8.1（环几何够大），它的 checker 已实现（池级 checker 的 `judge_journal_ring_geometry_fields_agree`，core 恢复时同判三条等式，`crates/singlefs-core/src/recovery.rs`）。

**依据**：

- E25（环与链各要多大保留池）：checkpoint 卡死恰好在预测的那一档归零——环在 reserve = ckpt_cost，链在 ckpt_cost 加一个窗口的 journal 块。
- E23（journal 几何）：环与链只差一项——链式每个 journal 块都要向分配器要一次块，环 0 次；链式没有死锁 3、环要几何不变量。
- 三方：2026-08-29 三条腿一致判「链式进入死锁 2」过强（材料 `_d23-background.md`）。

**欠**：无。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:118-149`（整段抄，未转述）**

```markdown
#### 已定项 3：tail 住系统配置槽，改 jbd2 两处

**定案**：**取系统配置形态**（tail 的权威副本住在一个固定位置，不内联在记录头），**并改两处**：

| # | 改动 | 为什么 |
|---|---|---|
| 1 | **不许单副本原地覆写**，改 ≥2 槽轮换 + 代号 + 先逐个验证全部候选再择新 | D22（单元原子性怎么合成）已对根定过同一条纪律：一次撕裂会抹掉唯一见证者 |
| 2 | **随每次发布写一次，是发布序列的最后一步** | D16（发布语义） 已定项 7 的注：根槽持久之后再更新系统配置槽、进崩溃点重放枚举；D16（发布语义） 已定项 6 逐字「每次发布把 `checkpoint_txg` 加一——**fsync 触发的发布也是发布**」⇒ 「每个 checkpoint 一次」与「每次发布一次」是同一个频率，fsync 触发的发布同样要写系统配置槽 |

- 原地覆写与 COW 不冲突：tail 与根槽、journal 记录头同属自证单元（发根本身就是覆写根环的一个槽），归 D22（单元原子性怎么合成）已定项 21 管；COW 对没有父的单元买不到东西。**tail 语义上是派生态**：权威判定仍是 `jsn` 连续前缀 + `checkpoint_txg`。
- **由此逼出三条硬要求**：
  1. **恢复不许先信 tail 再往前走：必须先全环扫描、逐条验证，再择最长合法前缀。** tail 陈旧时，`[tail_陈旧, tail_真实)` 这段已经被新一圈记录写掉了 ⇒ 从陈旧 tail 起往前走，**第一条就 `jsn != expected`** ⇒ 断号即止 ⇒ 连 `tail_真实` 之后仍然需要的记录也不重放。**那是数据丢失方向，不是「多做功」方向。** ⇒ 「陈旧 tail 只让恢复多做功」这句话**只在全环扫描式恢复下成立**。与 D22（单元原子性怎么合成）对根槽的「先逐个验证全部候选、再择新」是同一条纪律。
  2. **重放必须幂等**：

     > **每条 journal 记录的重放必须幂等**——同一条记录被重放两次与重放一次，结果相同。

     「陈旧 tail 只是多做功」这句话的前提是它：没有幂等，陈旧 tail 就从「多做功」变成「重复施加已生效的记录」。与已定项 20「记录承载指针层的**目标态**」相容：目标态天然幂等（把指针写成某个值，写两次结果一样），逻辑意图不是（「计数加一」执行两次结果不同）。
  3. **恢复算法救不了几何**：窗口撑爆环时两种恢复一起丢；环大小 ≥ 窗口是 [checks-owed.md](../checks-owed.md) C28（环几何不变量未实现）的职责，**不是恢复算法的**，两者不许互相顶替。
- 块重用叠上来之后「陈旧 tail 只是多做功」不成立，恢复需要重放下界，取根的水位（已定项 14）；这对已定项 3 是收窄不是推翻：「先全环扫描、逐条验证、不许先信 tail」原样成立，变的是验证出的合法前缀里哪一段被施加。

**射程**：管 tail 住哪、多久写一次、恢复怎么用它。字节表 [layout/01-first-txn.md](../layout/01-first-txn.md) 零与一两节按「每次发布一次」写：两次暖机空发布与新池新建文件各推进一次 tail。⚠️ 取系统配置形态时用过的代价表（E23（journal 几何） 的「两条臂差 21 : 500」）已作废：第一版每次发布都是 checkpoint，稳态多写从 21 次变成每次发布两盘各一槽；重放那一维不存在（恢复不读 tail、每次挂载全环扫描）。「每次发布轮换系统配置槽、返回前等它持久」的依据是判据 N-配置（已定项 14「可写挂载读到的更新状态读不出」，C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙、C577（系统配置没见证到的最新根，乙罩不到））：只减轮换次数或不等它持久就返回，「已确认的发布 + 根槽一时读不出 + 一份点名单元坏」时丢已确认的发布，而且那一格与「根槽 FUA 之前崩 + 一份单元坏」盘面逐字节相同、改读者分不开。这条依据成立在那一类双故障在容错射程里（用户 2026-09-28 定在）；落后支（D18（块里携带什么信息） 已定项 11）读 tail 不是这条依据。

**依据**：

- E24（恢复算法：先信 tail 会不会丢记录）：先信 tail 漏掉的条数恰等于窗口内写入的条数；全环扫描不丢，但把陈旧 tail 以来的记录全部重放一遍（逼出幂等）；窗口撑爆环时两条臂一起丢（恢复救不了几何）。
- E23（journal 几何）：jbd2 形态与 XFS 内联形态在稳态额外写与空闲崩溃后的重放量上方向相反——按的是 tail 每 checkpoint 推进一次的旧前提，见射程。
- 三方：2026-08-29（材料 `_d23-background.md`）。
- 三方判决 `research/prompts/c245-r3-main-verification.md`（C245（系统配置槽的写频率，两条条款说反话） 第三轮）：每次发布轮换的依据换成判据 N-配置；攻方探针量出乙、丙在「已确认的发布 + 根槽一时读不出 + 一份点名单元坏」上丢已确认的发布、甲要四处故障才丢。
- 用户定案 2026-09-28：那一类双故障在容错射程里（原话在变更史）。

**欠**：C77（重放起点未定义）；C594（每次轮换之前现算整池 F 的读开销没量）。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:150-179`（整段抄，未转述）**

```markdown
#### 已定项 4：记录头约束在单个原子单元内，头 487 字节

**定案**：**记录头完整落在一个原子单元内**（[invariants/03-journal.md](../../../crates/singlefs-checker/invariants/03-journal.md) I-8.2（记录头不跨原子单元））。头 **487 字节**：

- **十个字段 78 字节**：magic 4 + 类型 2 + 算法类型 1 + 记录标志 1（位 0 = 本次发布末条，已定项 17；其余位写 0，读到非 0 当损坏） + 自述长度 4 + 点名项数 4 + `jsn` **10** + `checkpoint_txg` 8 + nonce 12 + 头部校验和 32。`jsn` 的 10 字节 = 实例代号 32 位 + 计数器 48 位（已定项 9）。`tail_lsn` 不在头里：已定项 3 逐字定了「tail 的权威副本住在一个固定位置，**不内联在记录头**」，而「内联在每条记录头的 `tail_lsn`」正是被否掉的那个 XFS 形态的定义。
- **三笔已定增量 17 字节，不在那十个字段的 78 里**：已定项 7 的事务号 + 提交标记 **9 字节**、已定项 8 的反向链 **4 字节**、已定项 13 的载荷校验和 **4 字节**。
- **本次发布内序号 4 字节**（无符号 32 位，从 1 起：一次发布切成 N 条记录时依次是 1..N，只有一条时是 1；空发布记录也写 1）：已定项 14 注 1 在所选根那条记录读不出时靠它接链首。4 字节的宽度与紧跟事务号、提交标记的落点是主 agent 按「不为省空间牺牲自包含」取的，可推翻。字段在头里的偏移：事务号 78、提交标记 86、本次发布内序号 87、反向链 91、载荷校验和 95、新根段 99、fsid 463、MAC 471、头末 487。一次发布的 N 条记录序号依次 1..N、与 jsn 同步；读者遇到这四格，都当那条记录损坏、断链即止，那次发布整体不施加：序号 0；一次发布之内跳号；锚点读得出时，下一次发布的首条序号不是 1（断在这一条）；同一 (实例代号, checkpoint_txg) 里带末条标志的那条之后还有记录（断在带标志的那条；「之后」按计数器读、不要求紧接着：计数器大于带标志那条的、读得出的同 (实例代号, checkpoint_txg) 记录都算，中间缺号不停；前提是同一实例里 journal 记录的 `checkpoint_txg` 随计数器不减，有这个前提时与「表里任何位置」的全表读法给出同样的施加结果）。与 I-8.9（一次发布的记录序号连续且只有末条带标志） 判的是同一件事。
- **已定项 15 的新根段 364 字节**（树表指针 86 + 映射根指针 86 + 树 ID 水位 8 + F 8 + 实例表指针 86 + 分配记录树根指针 86 + 卸载记号 4）。
- **fsid 8 与 MAC 16，都排在新根段之后**：
  - **fsid 8**（系统配置 fsid 的低 8 字节，与单元头那 8 字节同口径，D18（块里携带什么信息） 已定项 7）。不带的话 I-1.4（块头 fsid 一致） 要给 journal 记录写一条豁免，而全环扫描就再也挡不住「上一个文件系统留在这块盘上的记录」——mkfs 整环写 0 只在 mkfs 真跑完时成立，重 mkfs 中途崩掉就不成立。带 8 字节之后不用豁免。
  - **MAC 16**（放在 fsid 之后；nonce 12 与算法类型 1 十个字段里已有）。理由与根记录那 29 字节同一条（D22（单元原子性怎么合成） 已定项 7）：journal 记录头是自证结构，没有父指针带 MAC ⇒ 不 day-1 留位，开加密那天要么整条走明文豁免（泄漏 `checkpoint_txg`、新根段里的两条指针、点名项位置），要么就是一次布局变更；D9（加密） 已定项 3 的原则逐字是「为加密留的位所有卷都留、不挪作他用」。I-6（加密不变量）不列它。
- 登记值就是 487（`JOURNAL_HEADER_BYTES`）；十个字段的 78 另登记为 `JOURNAL_HEADER_TEN_FIELD_BYTES`。**说 78 时要写明指的是十个字段，连同这一句一起引。** 487 占 512 扇区的 95%，其后还余 **25 字节**（装不下一个点名项，每项 56 字节）；4096 上余 **3609**（64 个点名项，⌊(4096 − 487) ÷ 56⌋ = ⌊3609 / 56⌋）。
   <!-- format-const: JOURNAL_HEADER_BYTES = 487 stale=头 277 字节|的 277 字节头|合计 **277 字节**|JOURNAL_HEADER_BYTES: u64 = 277|JOURNAL_HEADER_BYTES: u64 = 84|JOURNAL_HEADER_BYTES: u64 = 86|JOURNAL_HEADER_BYTES: u64 = 78|512), 428|512), 426|512), 434|4096), 4012|4096), 4010|4096), 4018|512 上 428|512 上 426|头 86 字节|头 **307 字节**|登记值就是 307|⌊3789 / 56⌋|头 **311 字节**|余 **201 字节**|JOURNAL_HEADER_BYTES = 311|512 上余 201 -->
   <!-- format-const: JOURNAL_HEADER_TEN_FIELD_BYTES = 78 -->

**射程**：⚠️ **那个原子宽度不由本工程定**：D20（承重面：单元的原子性与自包含）已定自证单元依赖**运行时探测**的 `physical_block_size`，不许把 512 焊进格式。⚠️ **这个值是格式常量，改它要三处一起动**：kb 标记与正文、实验源码的 `const` 与钉死它的单测、重跑实验并更新产物；由门禁阶段「格式常量在 kb 与实验源码之间同步」（`.claude/gate.d/code-source-discipline.sh` 的 format-constants 格）判红。记录尺寸取 4 KiB 定长之后对齐浪费恒为 0（已定项 12）；变长记录的对齐代价表只对变长记录成立。

**依据**：

- E23（journal 几何）：十个字段（跑时还带 `tail_lsn`）的头装得进一个 512 单元，不对齐、多条记录挤同一个原子单元时除第一条外都判不了撕裂——头要完整落在一个原子单元内。
- E75（记录尺寸与环几何）：各个头部读法在 512 与 4096 两档 `physical_block_size` 上都装得进一个单元；定长 2 的幂尺寸下头永远不跨单元。
- fsid 8 与 MAC 16：用户弹窗定案，没有走三方论证，可推翻，原话在变更史。
- 本次发布内序号 4 字节：用户 2026-09-23 定（`research/prompts/m2-presumed-clauses-r1-main-verification.md` K4 的 Z6），是推的、被攻过零轮，原话在变更史。
- 用户定案 2026-09-24：头 311 落地、末条标志放在原「填充 1」那个字节、序号的读者规则（原话在变更史）。
- 用户定案 2026-09-24：锚点读得出时首条序号不是 1、末条标志之后还有同 txg 的记录，两格都当损坏断链（实二十报告 `m2-lastflag-implementer-report.md` 第六节 2、3），原话在变更史；无实验：两格都要 CRC-32C 碰巧对上或改出来的镜像才走得到。
- 第四格的「之后」按计数器读、缺号不停，前提是同一实例里 journal 记录的 `checkpoint_txg` 随计数器不减：三方判决 `research/prompts/m3-formal-r2-main-verification.md` 第二节第 8 行与 `research/prompts/m3-formal-r3-main-verification.md` 第二节第四格补句一行，两条腿同向；前提另立不变量的判决是 `research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 9 行；用户定案（等前提判完一起写回），原话在变更史；无实验：补的是读者规则的射程，没有新的实验读数。
- 新根段加实例表指针、分配记录树根指针与卸载记号三项（头 311 → 487、点名项 67 → 64）：三方判决 `research/prompts/m3-ring-r1-main-verification.md` 第二节第 2 行（第一轮时的施加法在写行、首个文件、卸载三种发布上补出错根，推的）与第四节第 1 条，`research/prompts/m3-ring-r2-main-verification.md` 第二节第 2 行（首个文件、卸载两种发布补进去之后与当初会写的根逐字节相同、checker 绿，攻方在副本上量的），`research/prompts/m3-ring-r3-main-verification.md` 第二节第 1、2 行（两位放进记录标志字节的形态被专用清扫发布打穿；施加处不从新根段来的只有卸载记号、实例表指针、分配记录树根三项）。用户定案「回到甲」（原话在变更史）。卸载记号占 4 字节是第一轮正推腿 H1-b 的算法（86 + 86 + 4），判决没有单独定宽。无实验：补的是施加处的字节，数都在三方副本上量过，没立实验号。

**欠**：无（C94（登记的格式常量与后来的定案对不上） 2026-09-24 已还清，见欠账表已还清表）。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:180-195`（整段抄，未转述）**

```markdown
#### 已定项 5：O2（独立解析器 + checker） 是一元谓词，逐字节比对归记录核对器

**定案**：**O2（独立解析器 + checker） 是一元谓词，只判单个镜像**，不是「逐字节比对」——那是二元关系，需要第二个入参；需要第二个输入的核对归**记录核对器**，入参 `(崩溃前镜像, 记录流, 崩溃后镜像)`，权威记录在 D13（验证路线） 已定项 7。

- ⇒ D23（journal 的角色与格式）否掉逻辑意图日志的第二条理由（「落点不确定使核对退化」）按字面是空的，**改写为「核对从句法变成语义」，且它不是独立的第二条理由，是第一条（重放要第二个实现）的一条封堵**：逻辑意图记录不点名任何盘上对象 ⇒ 核对方要自行实现树语义 + 记账 + 分配合法性 ⇒ 第二实现的成本原样搬到核对侧。
- ⇒ **「否掉逻辑意图日志」这个结论不动**，第一条理由独立成立（已定项 20）。
- ⇒ D23（journal 的角色与格式）的反向判据主语是**记录核对器**（已定项 1；主语写成 O2（独立解析器 + checker） 时按字面恒为 0——O2（独立解析器 + checker） 就是 checker——不可证伪）。

**射程**：只管 D23（journal 的角色与格式） 怎么称呼这两个验证角色、它们各否掉什么；两者是什么、入参是什么、代码在哪，权威在 D13（验证路线） 已定项 6 / 7。

**依据**：

- 无实验：一元谓词不能变成二元关系，这是类型之争，没有可量的量；2026-08-29 三方论证（材料 `_o2-background.md`），逐条依据见 D13（验证路线） 已定项 7。

**欠**：无。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:196-207`（整段抄，未转述）**

```markdown
#### 已定项 6：目标负载优先级移出 journal 这条线

**定案**：目标负载优先级移出 D23（journal 的角色与格式），见 D25（目标负载优先级）。它跨 D8（核心索引结构） / D10（随机写 / 碎片化的真答案） / D23（journal 的角色与格式），不是 journal 的问题；留在这里会被当成 journal 细节。**它不阻塞 D23（journal 的角色与格式） 任何一项**：轴二是被轴一结构性排除的，目标负载只影响已定项 1 里「乙的收益形状」那条已降级依据的强度，不影响结论。

**射程**：只是一条指针，定了什么看 D25（目标负载优先级）。

**依据**：

- 无实验：这是一次挪位，理由是它跨三条决策、不是 journal 的问题，没有可量的量。

**欠**：无。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:208-227`（整段抄，未转述）**

```markdown
#### 已定项 7：一个事务可以跨多条记录，记录头带事务边界字段

**定案**：**可以跨多条，记录头带事务边界字段**：事务号 8 + 提交标记 1 共 9 字节（已定项 4 的三笔增量之一；与已定项 8 的反向链是**不同的字段**，不许合并计价）。

- 事务号按实例计数、从 1 起，同一事务的全部记录共享它，与 jsn 是两个计数；单元头写序存它的低 48 位、高 16 位恒 0，不新开字段（C113（扫描重建时多版单元的现行版本判定无输入） 定案复用这个字段）。事务号 0 留给不承载事务的空发布记录（已定项 19 ①）。
- 记录按事务号顺序追加，一个事务在发出单元写之后失败即实例切换、号更大的记录一条都不追加——这是实例表行的 W 能当精确前缀的依据（第一版串行提交下 W 的前缀性本来就成立，D13（验证路线） 已定项 2；这条纪律买的是并行提交打开之后它仍然成立）。
- **恢复算法**：在「jsn 严格连续 + 校验和 + 反向链」之后**再加一步**——**丢掉提交标记还没出现的那个事务的全部记录**。⚠️ **它不能靠反向链代劳**：被丢的尾巴在链上是**连续**的（它们本来就是顺序写的），链检查一条都拦不住。**两条闸管的是两件事**：链管「是不是同一条时间线」，提交标记管「这个事务写完了没有」。
- ⚠️ **丢尾巴是常态不是边角**：前缀落在事务中间的是绝大多数崩溃点 ⇒ 恢复路径里那一步会被绝大多数崩溃点走到，它必须和主路径一样被崩溃点重放覆盖，不许当异常分支。
- 判据已写成 I-8.8（前缀里的事务不被切开）（单张镜像那一半）与崩溃点重放（整数个事务那一半）；单张镜像那一半池级 checker 已判（`walk::check_pool_image` 的 `judge_commit_markers_per_transaction`），崩溃点重放那一半由崩溃点重放流 L8（并行线三跨记录崩溃点重放流） `crates/singlefs-checker-tier/tests/crash_enumeration_record_spill_over_stream.rs`（一个事务跨两条记录，每个状态必评估 I-8.8（前缀里的事务不被切开））判，C37（事务与记录的对应无定义） 已还清，见欠账表已还清表。

**射程**：管事务与记录的对应；记录与时间线的对应是已定项 8，两条不是同一条——已定项 7 取任一答案，时间线那一格照样漏。⚠️ 它不是 D13（验证路线） 已定项 2（串行 / 并行）的推论——串行下同样会切开。I-8.2（记录头不跨原子单元） 只约束**记录头**，没有任何规则禁止一条大记录；一个事务写几个单元的用户数据归 D16（发布语义） 已定项 5 末段的切分纪律，它与记录数口径的冲突归 C310（事务切分纪律与记录数口径打架）。「一事务一条」那一侧还有两条没建模的代价：环回绕要连续空间、整条记录要先在内存里缓冲齐才能开始写。

**依据**：

- E42（一事务几条记录）：一事务一条与「跨多条 + 事务号 + 提交标记」都满足 I-4.3（提交原子），跨多条而没有边界字段时非法前缀恰等于被切开的前缀数、占绝大多数崩溃点——实验分不出对错，只分得出代价落在哪。
- E45（事务跨多条记录的环占用）：它量出的「跨多条把环顶大 12 倍」是一条比 I-8.2（记录头不跨原子单元） 严的对齐规则造出来的，按字面倍数是 1.00×，拿它撤回已定项 7 依据的那一步不成立（实验结论作废）。
- 用户定案「跨多条 + 边界字段」，理由「它严格更一般」，攻击轮复核维持；定案带来的三条义务（格式、恢复算法、不变量）用户口径是「算法要增强」，原话与经过在变更史。

**欠**：无（C37（事务与记录的对应无定义） 2026-10-11 已还清，见欠账表已还清表）。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:228-248`（整段抄，未转述）**

```markdown
#### 已定项 8：记录头带一条 32 位反向链

**定案**：**记录头加一条反向链（前一条记录的 hash），宽度 32 位。** 覆盖哪些字段是已定项 10，hash 算法是已定项 11（完整 32 位 CRC32C），写路径上的取值口径是已定项 19 ②。

- **它补的缺口**：D23（journal 的角色与格式）三条已定项（定长环 / 断号即止 / 全环扫描）**各自成立，凑在一起漏**：定长环的槽位由 `jsn` 决定 ⇒ 断号之后的残留记录不会消失、只会等着被覆盖；而恢复之后从「前缀末 + 1」接着写，**那正好接在残留序列的前面**（已定项 22）。
- **同一个字段两个职责，两个都记**：分辨时间线；以及让记录核对器能用**与正向扫描完全不同的遍历**复核同一个前缀边界（正合 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「校验的两条路径不许共享同一段代码」）。只按验证独立性记，它会在「为了省字节」的权衡里第一个被砍掉。
- **一旦 hash 覆盖了实例代号（已定项 10），反向链对「分辨时间线」这个职责就是冗余的**——实例代号那条出路是精确的，不是概率的；那时链剩下的职责是遍历独立性。
- ⚠️ **一次误接受没有第二道防线**：I-8.3（重放前缀严格连续）、在飞记录数上限、`checkpoint_txg` 三条闸都不触发（两条时间线在同一个 checkpoint 窗口内，那个字段一模一样）⇒ 后果是**把一条已丢弃时间线的指针层目标态装进现行状态**，落在 I-7.4（近 K 代块未被复用） 保护窗口内时它完全合法、静默为旧数据。

**射程**：⚠️ **误接受的分母是「空洞 → 恢复 → 续写 → 再崩」这个事故序列发生的次数，不是时间**；取的场景点（每天崩 10 次、十年）不是测量，本仓还没有崩溃频率遥测。链宽不随文件变大而需要变宽：一次事故里链只在一个接缝上被考验，随文件变大受压的是序号计数器（已定项 9）与环的几何（I-8.1（环几何够大））。⚠️ 接缝数口径压在「一次事故一个接缝」这条没建模的假设上，一次恢复若留下多个断点，分母跟着变 ⇒ **32 位与已定项 11 的 CRC32C 是同一个单点**，那个口径一倒两条一起倒，重开条件对两项同时生效（已定项 11 重开条件第 1 条）。⚠️ 代价侧（2 / 4 / 8 字节链在盘上多不多占单元，E49（反向链宽度 32 还是 64） 扫过六档头宽）只取决于记录头总字节 mod 8，**是偶然不是结构**，头宽再动就要重算；现行不含反向链的头是 483（487 − 4），不在扫过的档位里 ⇒ **重算之前不许把「选 16 不省东西」「64 位也免费」「两侧都不分岔」当结论引用。** ⚠️ XFS 的 `h_prev_block` 只在写入侧赋值、恢复侧不比对它，引它时只能引「字段形态」，不能引「现役这么防」。

**依据**：

- E32（上一条时间线的残留）：定长环 + 断号即止 + 从前缀末接着写，残留记录会被下一次恢复接上去重放；两条自然实现各违反一条判据、方向相反——缺口在规范里，不在实现里。
- E36（槽位映射那一维）：槽位与序号解耦救不了，新旧记录等长时与按序号那一支逐格相同，只在记录长度恰好不同时碰巧挡住。
- E39（反向链挡不挡得住残留记录）：开链的全部格子残留为 0 且一条自己的记录都没误杀；误接受率逐档贴住 2⁻ⁿ。
- E51（反向链的碰撞机会有多少次）：碰撞机会的正确分母是接缝数（同源记录的链值构造性相等），按它 32 位全部档位够用；把每一次相邻比较都算一次机会的遍历分母是过计。
- 用户定案：加反向链「字节代价小，而它同时买到两样」；宽度「如果 32 位够用 那么就 32」，原话在变更史；三方 2026-08-30 三条腿一致指向 32 位。

**欠**：C42（残留记录冒充合法前缀）；C200（记录头改宽后的代价重算没写回条款）。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:249-267`（整段抄，未转述）**

```markdown
#### 已定项 9：jsn = 实例代号 32 位 + 计数器 48 位

**定案**：**实例代号 32 位 + 计数器 48 位，两个字段共 10 字节，不挤进一个 64 位。** 分开还多买一样：**比大小不必先解位域**，恢复路径少一处可以写错的移位。

- 短板在代号那一格：代号与 fsync 率无关，每天崩 10 次、十年 = 36 500 次，16 位容量 65 535 只有 1.8 倍余量 ⇒ 加宽到 32 位（容量 4.29×10⁹，用不完）；计数器 48 位不加宽。
- 计数器全池接着走、不按实例归零（已定项 14 第 3 条），48 位的寿命按全卷算。
- ⚠️ **随之而来的一条义务：代号不许回滚，且要有人保证。** 代号一回滚，这条出路逐格退回没有代号时的样子（残留全数重放）。**这与 D9（加密） 已定项 8 对 nonce 水位的要求同型**——那里逐字要求水位「不许住在裸明文系统配置里……必须落在一个被 MAC 覆盖的字段上」。
- **代号住哪儿、谁保护它**：住 journal 记录里是循环（恢复要先读记录才知道代号，而读记录正需要代号来判）；住系统配置则落进「不许回滚」那个要求 ⇒ 与 D9（加密） 已定项 8 的 nonce 水位**同一个载体问题**，两者应当一起定，不许各定各的。做法：取号取 max(系统配置, 根环) + 1、写进每一份系统配置（已定项 16）。

**射程**：⚠️ **敲定的两个数各有一处未测**：① 计数器那一档只在本机测过，**本机盘没有掉电保护（PLP）**，带 PLP 的企业盘 fsync 率可以高一到两个数量级，那一档本机测不到，仍是假设；本机测的还是 ext4 的 fsync 路径，是本工程 fsync 率的**上界代理**。② 代号那一档的「每天崩 10 次」是**取的场景点，不是测量**。⇒ **重开条件可判定**：拿到现场 fsync 率且它超过 10⁵/秒，计数器那一半重开。已定项 14 之后前缀规则不改：重放从所选根覆盖的最后一条记录之后接，所选根是最新实例的根时新记录都在同一实例里、严格连续；只有所选根退到更早实例时才碰到实例边界，那一格按严格前缀停，代价写在 D16（发布语义） 已定项 7 的注，要不要补是 D16（发布语义） 已定项 8。

**依据**：

- E44（序号位宽的本机实测与代价）：本机 fsync 率下 48 位计数器远超需要，「48 位不够」那条推理的前提在本机不成立；`jsn` 加宽到 10 / 12 字节在 512 与 4096 两种单元下点名项容量不变、一格都不多占单元。
- E37（日志实例代号）：带实例代号的那一臂两条判据同时满足，前提是代号不许回滚——回滚一格就逐格退回残留全数重放。
- 用户定案，原话在变更史。

**欠**：C199（实例代号递增与 jsn 断号即止互相矛盾）；代号的载体与防回滚保护没有编号，挂在 D9（加密） 已定项 8 的 nonce 水位同一个载体问题上。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:268-284`（整段抄，未转述）**

```markdown
#### 已定项 10：`previous_hash` 覆盖前一条记录的整个记录头

**定案**：**覆盖前一条记录的整个记录头，其中含 `jsn`。** `jsn` 是「实例代号 32 位 + 计数器 48 位」（已定项 9）⇒ **实例代号在链值的输入里** ⇒ 两条时间线的链值构造性不同，误接受率回到 2⁻ⁿ 那条曲线上，32 位够用的判断因此成立。

- ⚠️ **取「整个记录头」而不是「只取 jsn」的理由**：多覆盖的字段**一个字节都不多花**（hash 的输入长度不进盘），而少覆盖一个字段就多一条「这个字段被改了链还对得上」的缝。
- ⚠️ **一条实现纪律**：`header_csum` 那 32 字节**自己不能进 hash 输入**（它是头的一部分而又依赖头）——写死为按零参与（已定项 19 ②）。
- **链值必须有一个「两条时间线保证不同」的输入**，否则误接受率是 **100%，与位宽无关**：若 `previous_hash` 取前一条的 `header_csum`，两条竞争记录的 magic / 类型 / `jsn`（都等于 expected）/ `checkpoint_txg`（同一窗口内一模一样）在点名项数相同、加密关闭时**构造性相等**。它与已定项 8 的宽度定案是两件事：宽度答「碰撞概率多大」，已定项 10 答「有没有东西可碰」。

**射程**：管 hash 的输入；函数是已定项 11，宽度是已定项 8，本实例第一条恒 0、写路径不读盘是已定项 19 ②。

**依据**：

- E37（日志实例代号）：带实例代号的恢复在残留与新写两条判据上都是 0，精确而不是概率——把实例代号放进链值的输入，链就有了「两条时间线保证不同」的那个字段。
- 用户定案，原话在变更史；三方 2026-08-30 反推腿读实验源码时指出链值的输入里没有标识时间线的字段，经过在变更史。

**欠**：C42（残留记录冒充合法前缀）。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:285-314`（整段抄，未转述）**

```markdown
#### 已定项 11：反向链的 hash 取完整 32 位 CRC32C

**定案**：**取完整 32 位 CRC32C**，与 D19（块指针的结构与宽度预算） 已定项 2 已定的密文校验和同一个函数。

- **主依据是「这个选择不承重」，不是「CRC32C 更好」**：32 位已判够用，且余量是**三个数量级** ⇒ 在 2⁻³² 这条曲线上，两个候选（完整 CRC32C、32 位截断的密码学 hash）**都够用**，选谁不改变风险量级；即使真损失 1 整比特（碰撞率翻倍），余量仍够。
- **在「都够用」之上，按两条次要理由选 CRC32C**：

  | 理由 | 它买到什么 |
  |---|---|
  | 与 D19（块指针的结构与宽度预算） 已定项 2 同函数 | 少一份实现、少一份 checker、少一类崩溃点重放 |
  | 有 SSE4.2 的 `crc32` 指令 | 提交路径上少一次密码学计算。⚠️ **本工程一个周期都没测**，这条只作次要理由，不许当依据 |

- **取「完整」不取「截断」**：截断的 CRC 不是短一点的 CRC，若链值取截断 CRC，2⁻ⁿ 那条曲线不适用。
- ⚠️ **一条已撤回的理由**：「CRC 线性 ⇒ 差异落在 ≤32 比特窗口内时碰撞可证为 0，而截断的密码学 hash 只能给不可证的 2⁻³²」。**它不成立**：那个「窗口受限」的前提是被模型硬编码出来的——两条竞争记录的 `named_count` / `record_length` / `nonce` 被建成构造性相同，而它们是**两个不同的事务**，且 I-6.1（nonce 不重用） 要求 nonce 全盘唯一。
- ⚠️ **由此还得到一条要记的反向风险**：**窗口条件不成立时，线性臂的尾部比密码学 hash 差。** CRC 的碰撞差异集是一个线性子空间——同一个差异模式**每次都碰**；截断的密码学 hash 是每对独立的 2⁻³²。在现有余量下这不改变结论，**但它是唯一一条指向另一边的证据，不许在引用定案时略过。**
- **重开条件（两条，任一成立即重开）**：
  1. 接缝数那个分母口径被推翻，余量掉到 10 倍以内；
  2. 已定项 10 的覆盖范围变化，使「两条时间线保证不同」这个前件不再成立——那时误接受率 100%，与函数和位宽都无关。

**射程**：只管反向链的函数；`header_csum` 的算法（CRC32C 4 字节 + 28 字节零）归 D18（块里携带什么信息） 已定项 17，`payload_csum` 的算法在已定项 13。

**依据**：

- E51（反向链的碰撞机会有多少次）：32 位按接缝数的分母全部档位够用，余量三个数量级——函数选择不承重。
- E61（反向链 hash 算法的均匀性）：量三只排除得掉「有效位宽损失 ≥ 1 整比特」，两个候选都在其内；量二作废，连带撤回了「线性可证为 0」那条理由。
- E40（密文校验和位宽）：截断的 CRC 把「单比特翻转必检」这条结构性质整个丢掉——取完整、不取截断。
- 用户定案（改正后的量二不跑），原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:315-332`（整段抄，未转述）**

```markdown
#### 已定项 12：记录尺寸取 4 KiB

**定案**：**记录尺寸定为 4 KiB，定长，写进系统配置**（XFS 把同一个参数写进系统配置的 `h_size`）。

- 与已定项 4 相容：记录头 487 字节要完整落在一个 `physical_block_size` 内，而 4096 被 512 与 4096 两种探测值都整除。
- 定长记录取 2 的幂尺寸之后，I-8.2（记录头不跨原子单元） 的对齐浪费**恒为 0**：记录背靠背摆下去，起始偏移恒是单元的整数倍。
- 4 KiB 记录装 64 个点名项 <!-- format-const: JOURNAL_NAMED_ENTRIES_PER_RECORD = 64 stale== 67 项 -->（已定项 4）；再大只抬高 I-8.1（环几何够大） 里那个「最坏事务占用」，进而抬高环大小下界，买不到别的。
- **用户口径**：先按这个数落地，写代码时再调整。

**射程**：⚠️ 余量那一格按的是「一次 fsync 一个事务」的读法（目标负载 12 项事务恰占 1 条记录），与 D16（发布语义） 已定项 5 末段的切分纪律（一个事务最多写一个单元的用户数据）不在同一口径：按切分纪律加已定项 7 一条记录一个事务号，seq 批 1 是 8 条记录、每条只装 1 到 5 个点名项，「12 项恰占 1 条」那一格不存在；余量口径与环几何怎么改归 C310（事务切分纪律与记录数口径打架），要用户定。目标负载（D25（目标负载优先级））若长大一个量级，这个数要重推，那时另立实验。环回绕与「装成一条要先在内存里缓冲整条记录」两条代价没建模。

**依据**：

- E75（记录尺寸与环几何）：4 KiB 上目标负载恰占一条记录、跨记录数 0，几个头部读法给的容量相同；环下界与另一个二进制已发表的那一格逐格相等；定长 2 的幂尺寸下对齐浪费恒 0。
- 用户定案，原话在变更史。

**欠**：C310（事务切分纪律与记录数口径打架）。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:333-356`（整段抄，未转述）**

```markdown
#### 已定项 13：载荷完整性，`header_csum` 罩整条记录

**定案**：`header_csum` **覆盖整条记录 `[0, 4096)`**，含点名项数组与其后的补齐，`header_csum` 自身那 32 字节按零参与（32 字节里放 CRC32C 4 字节 + 28 字节零，D18（块里携带什么信息） 已定项 17）；`payload_csum` 那 4 字节字段**保留**，仍罩点名项数组，算法取 CRC32C（与已定项 11 的反向链同一个，不引入第二种）。**第一版不再单独分辨「头落了、载荷只落一半」**——任一校验和不过就是该记录无效，恢复断链即止（已定项 18 / I-8.3（重放前缀严格连续) 同一句口径）。

- 与 D18（块里携带什么信息） 已定项 17 逐字「自证结构（根记录、系统配置槽、journal 记录头）的校验和覆盖整个槽含补齐」一致：journal 记录是自证结构、槽就是那 4096 字节；只罩头的话，这条通用规则要为 journal 记录单开一个例外，而根记录与系统配置槽都没有这个例外。
- **四个候选里另外三个各被一条已定条款挡掉，这是排除不是取舍**：

  | 候选 | 挡它的 |
  |---|---|
  | 甲（一个校验和覆盖整条记录）| 会失去「头落尾没落」的可分辨性。第一版取的正是「整条」这个覆盖范围，同时保留 `payload_csum` 字段——也就是接受了这一行写的代价 |
  | 丙（每个点名项自带它所指目标的校验和）| D4（校验和位置） 的校验和证明「指向的那个单元完好」，**不证明「这条点名项本身完整落盘」**，而载荷完整性问的是这条点名项本身完整落盘 |
  | 丁（丙 + `named_count`）| 丙那个洞没补，只多一道数量核对 |

- `payload_csum` 是头里的字段，**必须落在 `header_csum` 的覆盖范围之内**，否则改掉它本身是静默的（载荷与校验和一起改，两个都过）；`header_csum` 罩整条 `[0, 4096)` 之后这条天然满足。

**射程**：⚠️ **代价是明写的**：分辨「头落了、载荷只落一半」与「载荷被改一个字节」原本靠两个校验和的**差异**，整条罩之后这两格与「整条没落全」在第一版判成同一种结局（记录无效）；买回来的是「与另外两个自证结构同一条规则」。要重开「只罩头」时，那张四格判据表原样可引。

**依据**：

- E76（载荷校验和的判别力）：另加的载荷校验和在检查、不是在回声头那一份（头落了载荷只落一半、载荷被改一个字节，都只有它红）；只有一个校验和的形态在类型上答不了「头落尾没落」；判决对两个摘要函数相同，是覆盖范围的产物。
- 用户弹窗定案取整条（主 agent 推荐只罩头 `[0, 307)`），没有走三方论证，可推翻，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:357-464`（整段抄，未转述）**

```markdown
#### 已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换

**定案**：**恢复只施加 `(实例代号, checkpoint_txg)` 严格大于所选根的记录；陈旧 tail 只是「从哪开始扫环」的优化，不再决定重放集合。**

**前缀判定的完整口径是五条，缺一不可**（引用 I-8.3（重放前缀严格连续）时连这句一起引）：
jsn 严格连续（断号即止）、**`(实例代号, checkpoint_txg)` 大于根的水位**、
提交标记齐全的事务才施加（D23（journal 的角色与格式）已定项 7）、
施加前逐项验证点名单元（D16（发布语义）已定项 7，E77（发布的持久顺序）证明承重）——**一个点名单元的两份（两条位置条目）都读得出、校验和都对才施加**，有一份读不出或对不上，这条记录与它之后的都不施加。
⚠️ **第五条：施加的单位是一次发布——合法前缀停在一次发布的两个事务之间时，那次发布整体不施加，前四条判出来的前缀再按发布边界截短一次；引用这四条时连这一条一起引。**（D16（发布语义） 已定项 4）**发布边界怎么认**：一次发布的末条 = 记录标志位 0 为 1 的那一条（已定项 4、17）；前缀里一次发布的末条没到，那次发布整体不施加。

两份镜像何时算「记录在」：**任一份自证校验和过即在**（崩溃点重放里的检查由崩溃点重放全量做出——jsn 3 只有一份持久的两个状态都读回文件）。点名单元不同：**两份都验过才施加**（前缀第四条）。代价：两块盘的池上一块盘整块掉了，记录在幸存盘上、点名单元有一份在掉了的盘上，那条记录不施加——恢复退到更早那一版，已确认（fsync 已返回）的最后一版丢掉；D16（发布语义） 已定项 8 的暖机管不到这一格。用户知情认下（C519（丢一整块盘时恢复丢掉刚确认的那一版） 那一格：「任一份验过」会不会放过 E77（发布的持久顺序） 要挡的一份新一份旧的嫁接没量过）。

**跨实例边界、链首、比较序、计数器与切换时的 W**（注 1–4）：

1. **前缀规则不跨实例边界**：链从所选根覆盖的最后一条记录之后接，下一条的实例代号与所选根不同即停（五条口径第一条原样）。**所选根覆盖的最后一条** = 与所选根同实例、同 `checkpoint_txg` 的记录里 jsn 最大的那条——一次发布切成多条记录时它们共享一个 `checkpoint_txg`（D16（发布语义） 已定项 6），那次发布的最后一条才是根覆盖到的末端。**那条读不出时**，链首接水位之上第一条可读记录，前提是它的「本次发布内序号」为 1（已定项 4）、`checkpoint_txg` = 所选根的 txg + 1；序号不是 1 说明下一次发布的开头缺了，断号即止。**「那条」按末条标志认**——所选根那次发布读得出的几条里没有一条带末条标志，就算「那条读不出」，走「序号为 1 的第一条可读记录当链首」那一支；不取读得出的同 txg 记录里 jsn 最大的那条（用户 2026-09-24 定，读法乙；三方 `research/prompts/m2-presumed-clauses-r1-main-verification.md` K4 那一支四条臂同中、没分出候选，这一读法被攻过零轮）。所选根是 mkfs 的第 0 代根时它一条记录都不覆盖，之后的记录全属于更新的实例 ⇒ 一条都不施加。代价写在 D16（发布语义） 已定项 7 的注，要不要让新实例先暖机是 D16（发布语义） 已定项 8。
2. **记录水位 `(实例代号, checkpoint_txg)` 按实例代号为主比**，与 jsn 同序；择根照旧 txg 为主、实例代号破平局（D22（单元原子性怎么合成） 已定项 7），两处比的东西不同，各守各的序。按 txg 为主时，新实例的第一个根取「根环最大 + 1」，而被抛弃实例开放 checkpoint 里已提交的记录 txg 可以更大，会被当成水位之后施加到新实例的根上。
3. **计数器全池接着走**：新实例从前缀末 + 1 接着写、不归零。前缀末的计数器取环里读得出的最大那一个：所选根自己那条记录两份都读不出时，新实例的第一条编在读得出的最大号 + 1，落回那条读不出的记录原来的槽（被重写的本来就读不出）；计数器从记录头读，不按 txg 推。定长环的槽位由 jsn 决定（槽位与序号解耦那条出路已被 E36（槽位映射那一维） 判掉），归零会让新实例的记录落到旧前缀还要用的槽上；已定项 9 的 48 位本来就按全卷寿命算。⚠️ **checkpoint_txg 也一样**：新实例（普通挂载、切换）的第一次发布的 checkpoint_txg ≥ max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1；普通挂载本来就全环扫描，切换不为算这个下界重扫 journal 环（内存里的 txg 已不小于这次挂载见过的一切；切换的所选根要读盘择，见实例切换那一句）。它与 C331（择根倒挂压过已确认的写） 的修法候选「记录扫描水位」是同一个量；C331（择根倒挂压过已确认的写） 已还清（改取 R2，见欠账表已还清表）。代价：跳号比现行实现大（最多到被抛弃时间线的长度）；记账不受影响：D5（快照 / 空间记账机制） 已定项 2 每条根的记账树只装这条根那一代的行，不点删旧代，跳号不留要删的行（C341（K 代点删在 txg 跳号时留下删不掉的行） 题面消失同理）。
4. **切换时的 W 取被重发的那个 checkpoint 里的最大事务号**：开放 checkpoint 里已完成的事务全部 > W、按新写序重做。按第 2 条，它们的记录不在所选根之后；W 若取字面的「最后施加的事务号」，它们的单元会被谓词判成已发布而谁都不施加它们（C287（切换收养开放 checkpoint 的事务后再崩））。被重发的 checkpoint 里没有事务号非 0 的记录（重发的是空发布）时 W = 0（已定项 19 ①：事务号 0 不进 W 的 max）；W 写在每个被照旧事务的写序实例那一行，切换的所选根按实例切换那一句取。

**第一版没有干净关闭标记**：每次挂载一律走恢复（全环扫描、逐条验证、择根，按五条口径与注 1–4 施加），不按上一次是不是干净关闭分支；准入、暖机、回退候选集、F 的生效都不读这样一个标记。取号之前逐盘核时，落后支缺单元那一半读本盘环里那一版末条记录所在的那一槽（系统配置三方第三轮判决 `research/prompts/m3-close-sysconf-r3-main-verification.md` 第二节 U3 的乙'，实现 `crates/singlefs-core/src/mount.rs` 的 `last_record_of_the_version_in_its_ring_slot_on_the_device`），这一读不算按干净关闭分支（C499（干净关闭标记有没有没有条款） 的读法，用户 2026-10-20 定「认，点名这一读」）。

**第一个文件版本接在哪条记录之后**：写第一个文件版本之前，要接的那一版之后的上一条记录必须解得出、且它的 `checkpoint_txg` 等于那一版的 txg——新记录的计数器取它的计数器 + 1、反向链取它的记录头（已定项 8、已定项 9）；不满足就在任何写之前拒绝（`crates/singlefs-core/src/transaction.rs` 的 `publish_first_file` 返回 `FirstFileVersionDoesNotFollowTheVersionItBuildsOn`），一个字节不写。

**管理员回退是挂着时的一次向前发布**：管理员在一次可写挂载里从回退候选集选一个旧根 R_old，文件系统发一次普通发布（D16（发布语义） 已定项 7 的持久顺序），新根的用户可见树（inode 树、extent 树）与 `deleted_inodes` 的根指针取 R_old 那一版的；墓碑树、谱系树随现行那一版，不取 R_old，回退那次发布写一条谱系记录 (T_rb, T_old)（D18（块里携带什么信息） 已定项 24）；退役树同墓碑树、谱系树，随现行那一版、不取 R_old；树表单元、分配记录树、中央映射树与记账统计量（D5（快照 / 空间记账机制） 已定项 4）按这次发布照调整之后的分配器现算，不从 R_old 那棵账载入（inode 号与树 ID 水位见表里「水位」一格）；实例表照 cur 的；checkpoint_txg 照常加一，**不新增实例、不取号、不写实例表行、不施加也不删除任何记录**。记号：cur = 现行那一版，new = 回退发出的新根，用户可见单元 = 数据单元与 extent 树、inode 树的单元。

| 项 | 定案 |
|---|---|
| 候选集 | 根环里按 cur 的实例表判仍然有效 ∧ txg ≥ F_生效（D16（发布语义） 已定项 1）∧ 带文件（那一版写过文件，不是还没写过文件的那一版；删完之后有树、文件集合为空的那一版写过文件，算带文件）的根：(i, T) 可选 ⟺ cur 的实例表无 i 的行，或有行 (i, Ti, Wi) 且 T ≤ Ti——被崩溃恢复抛弃的时间线上的根一条都选不中 |
| 释放 | cur 那一版的账里仍分配、而 new 不引用的落点，释放代 = new 的 txg；隔离单元（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1：任一份核出对不上、两份都留在已分配的）除外，留在已分配（用户 2026-09-27 定） |
| 复活 | R_old 引用的用户可见单元里，cur 的账记成已释放的，改回已分配，分配代写回 R_old 那一版账里的分配代 |
| 水位 | inode 号水位 = max(cur 那一版内存里的水位, 环里读得出、按实例表没被崩溃恢复抛弃的根的水位)；max 里 cur 那一版内存里的那一项不能去：只取环，回退那次就重发号（C143（inode 号水位在崩溃恢复落到旧根后退回去重发） 那一洞）；cur 的实例表读不出时代码先拒整次回退（`crates/singlefs-core/src/mount.rs` 的 `RollbackError::CurrentInstanceTableMalformed`），根读不出本来就不算。树 ID 水位照旧 = max(cur 那一版内存里的水位, 环里读得出的全部根的水位)。两个水位都不跟着 R_old 退回去 |
| 记录 | 回退那次发布的记录事务号取 0、不推进实例内计数（与空发布同：不承载用户事务，D23（journal 的角色与格式） 已定项 19 ①）；新根带的 F 取现行那一版根上的，不算抬 F；jsn 接现行那一条（主 agent 2026-09-26 定，推的，被攻过零轮） |
| 准入与记账 | 回退照旧不判准入式子；谱系容器归固定点角色、进 ckpt_cost 的 Σ 名单；记账标签 4、6 按新根这一版现算（inode 树与 `deleted_inodes` 取 R_old 的，所以被带回来的对象的墓碑份额照抬；标签 6 按新根现算时，n_live（D3（空间分配） 已定项 2）= R_old 那一版 inode 树各叶容器头的记录数之和减 `deleted_inodes` 的 key 数，第一版 inode 树一个根、至多 135 个容器），不判（`research/prompts/m3-delete-r2-main-verification.md` 第三节 V4；`research/prompts/m3-delete-r3-main-verification.md` 第三节 G3-回退；用户定案 K-W5「谱系记录」、K-W5b） |
| 在任何写之前拒 | R_old 引用的用户可见单元在 cur 的账里没有记录，或仍分配而分配代与 R_old 那一版账里的不同；R_old 那一版或 cur 那一版的账有一个节点读不出；复活集里的每个单元（cur 的账记成已释放、这一次改回已分配的那些）在每块盘上的那一份都要读得出、校验和对得上，有一份不行就拒——它们的槽在 cur 的时间线上释放过，释放之后可能被一次没发布成的写盖掉一块盘上的那一份；R_old 引用、cur 的账里仍分配而同分配代的单元里被隔离的那些（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1：读盘核出一份对不上、两份都留在已分配）同样逐盘验，有一份读不出或对不上就拒（用户定案，原话在变更史）；其余 cur 的账里仍分配的单元分配器不会发出去，不在逐盘验之内（与前缀第四条「两份都验过才施加」同一方向；主 agent 2026-09-26 定，推的，被攻过零轮） |

崩在回退那次发布的任何一点，恢复落到的与任何一次普通发布没成时相同；回退的根持久之后才向管理员确认（D16（发布语义） 已定项 7）。回退深度由候选集定：平时是整个根环，盘紧时可缩到最近 4 个可退到的状态，每块盘上最新的持久有效根都在内。**被崩溃恢复抛弃的时间线的根离开根环之前，它们引用的单元不许重新分配也不许抹头——分配器多查环里每一个可读根的账、只隔离其中**只被被抛弃根引用**的槽——查账的集合取窄读法：按主语「被抛弃时间线的根」读，仍被有效根引用的槽不在其内——「有效根」指回退候选集里的根（按实例表判仍然有效 ∧ txg ≥ F_生效），每次挂载与每次抬 F 按当时的候选集重算；被抛弃根的账（树表或分配记录树）读不出时不跳过：从中央映射现算这条根引用的落点，只在内存里隔离、不写盘，计进 `abandoned_roots_unreadable` 往上报，映射也现算不成就重读一次，仍不成在取号之前拒可写、只读照常（C393（被抛弃根的账读不出时修复没有条款） 取 (b)-从映射现算，用户 2026-09-28 定，被攻过零轮，判决 `research/prompts/unreadable-at-mount-r2-main-verification.md` L3）；被抛弃根的账读不出时，正常卸载、会话写与再可写挂载都该做得成，拒只在没有凭据时留着：凭据是内存里的现行账有一条代大于那条根的 txg、罩住它树表槽（与映射根槽）的记录，只看账、不读那一槽的内容，有它就不拒，没有它照拒（C694（被抛弃的根的账读不出时正常卸载被拒） 的凭据放行，用户 2026-10-07 定「C 与 B 都做」，被攻过零轮）；抬 F 那一串（准入与卸载）不走这一道：被抛弃根照挂载时记在根环表里的落点算影子账，不从盘上重读它的账、读不出也不拒，这一拒只留在再可写挂载那一道（D16（发布语义） 已定项 1「根槽这一次读坏」那一行，用户 2026-10-07 定，被攻过零轮）；宽读法（凡被环里任一可读根引用过的槽都隔离）会让抬 F 在第一版 24 槽里买不到任何东西（影子账；隔离量怎么进准入不等式见 C318（影子账隔离的单元没进准入不等式））。** 同一次定案（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P3）把事务按单元切分 ⇒ 崩溃原子性的单位是一个单元、不是一次写请求（仓里没有任何决策承诺过写请求级原子性，这是一次对外语义的收缩）。

**可写挂载时的预隔离**（C694（被抛弃的根的账读不出时正常卸载被拒） 的 C，用户 2026-10-07 定「C 与 B 都做」，被攻过零轮）：可写挂载读阶段、重建分配器之后，把水位之上、没被施加的全部 journal 记录点名的落点隔离，不论那条根是不是回退候选根，只除去现行账里仍分配着的槽（它们本来分配着；这次挂载里释放、回收时分配器照补隔离，D28（挂载期承诺量） 已定项 1 第九项）；还用那次发布任一条读得出的记录的新根段读它的账，账里的落点一并隔离（`research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节「t1 C 的候选豁免」「t1 C 的盲区」两行，被攻过零轮）；再并上根环里比所选那一版新（按择根的序）、按最新那条根的实例表有效、读得出的根自己的账里的落点，按那条根的 txg 对应的根环槽归组（`C_NEWER_ROOTS`）。这两份账读不出时：那次发布读得出的记录齐（本次发布内序号 1..N 连着、带末条）就不加，交「记录有缺口时拒可写」；不齐（缺序号或缺末条）就照被抛弃根的账读不出那一份办，从中央映射现算、只在内存里隔离，现算不成重读一次，仍不成而现行账里没有凭据就在取号之前拒可写，这一拒在单子上，拒因与 C393（被抛弃根的账读不出时修复没有条款） 那一拒分开报（`C_MAPPING` 收窄；用户 2026-10-08 定「乙：core 另起错误成员」）；较新可读根那次发布的记录一条都读不出、账也读不出时：映射现算得出就隔离；现算不成时看那条根自己的树表与中央映射树根两个指针（C834（择到的系统配置实例代号随盘序变），用户 2026-10-15 定案「同一指针两份都更新」，判决 `research/prompts/c834-r3-main-verification.md` 第四节，被攻过零轮；替换 2026-10-08 定的 Y2 收严）：有一个指针两份都被更新实例的单元盖掉时不拒、不重读（入库用例 `urJ` 那一形）；否则照记录不齐那一支办（重读一次、看凭据放行，仍不成在取号之前拒可写挂载）。`C_NEWER_ROOTS` 与 `C_MAPPING` 照判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 H1、H3 两行与第三节第 1 条，只在攻方模型上量过、被攻过零轮。C 的全部落点只在这个进程写掉那条记录所属 txg 对应的根环槽、那次发布取完落点之后解开，不随 D16（发布语义） 已定项 1「根槽这一次读坏」那一行给读回全零的槽的进程内重读时机解开。它护的是挂载那一刻根槽读不出、系统配置又没见证到的那条根新分配的单元：影子账只查环里读得出的根的账，看不见这条根；系统配置见证过的发布由乙-配置续（「可写挂载读到的更新状态读不出」那一段）管。隔离的槽算进准入不等式第九项（D28（挂载期承诺量） 已定项 1）。**记录有缺口时拒可写**（用户 2026-10-07 定 F1「乙：缺记录就拒可写」，被攻过零轮）：水位之上一次没施加的发布，读得出的那几条记录本次发布内序号不连续或缺末条、而它们点名的单元里有一个两份副本都验不过（`research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节 t4 行，被攻过零轮），重读一次，仍这样就拒可写挂载。可写挂载照「所选根走读失败、系统配置没见证它时退回见证到的那一版」退了之后，按退到的那一版当水位再判一次，再判之前重扫环、用重扫的那一份记录，不用读阶段扫出的那一份（C754（退回之后再判缺记录不重扫环），用户 2026-10-20 定「重扫」）：那一版之上一次没施加的发布这样，重读一次，仍这样就拒，拒因同一个（`F1_AFTER_F2`，判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 H2 那一行与第三节第 1 条，只在攻方模型上量过、被攻过零轮）。

**可写挂载读到的更新状态读不出（C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙-配置续，用户 2026-09-27 定；判据 N-配置续 照 `e158-r3-prereg.md`「5.1　判据 N 用哪一样」与 `e158-r4-prereg.md`「5.1　判据 N 用哪一样」）**：读阶段（择根、扫 journal、重放）读完，读池里每块盘两槽的系统配置：判不出 ⇒ 按真（判不出取 R2，权威写法在 D22（单元原子性怎么合成） 已定项 8：有槽设备报读错，且没有一块盘两槽都自证过；用户定「改取 R2」；实现取 R2：`crates/singlefs-core/src/recovery.rs` 的 `system_configuration_witness_is_undecidable`，读阶段与取号同用；多次挂载崩溃点重放里的注入由 E165（恢复重放有界穷举与变异杀伤界） r3 择根与重放合判一族在带读故障的多次挂载历史上判 KJ1T，C331（择根倒挂压过已确认的写） 已还清，见欠账表已还清表）；不是判不出时取全部自证过的槽 journal tail 的最大值 c_见证，0 ⇒ 不判；所选那一版那次发布带末条标志的记录读得出（任一份）⇒ c_见证 > 它的计数器即为真；读不出而计数器等于 c_见证 的记录读得出 ⇒ 它的 (实例代号, checkpoint_txg) 大于所选那一版即为真；两条都读不出按真。为真就在这次挂载内有效的读缓存（只收读成且不是全零的落点）上把读阶段重做一遍（R = 1，取自 D16（发布语义） 已定项 1「根槽这一次读坏」那一行的「重读一次」，两次读之间不等），重做那一遍为假就用它往下走，仍为真就在取号之前拒可写挂载、盘上逐字节不变，只读挂载照常。重建分配器判抛弃用的最新那条根的实例表读不出，同样重读一次、仍读不出拒可写（代码审阅第 22 条）。重建分配器读根环两遍并起来一条不被抛弃的自证有效根都没有，同样在取号之前拒可写挂载，报 `MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread`（D16（发布语义） 已定项 1「根槽这一次读坏」那一行），这一拒在单子上；挂载透明（`crates/singlefs-harness/src/mount_transparency.rs`）在镜像上复算「按最新那条根的实例表，环里一条不被抛弃的自证有效根都没有」，没注读故障时复算为真判不合法、注过读故障的态不判、实例表读不出时不按表滤。**「续」**：取号那一写（`crates/singlefs-core/src/transaction.rs` 的 `write_acquired_instance`）不再把 tail 写成 0，改写取号那一刻按同一取法读到的见证值 c_见证（用户 2026-09-27 定「乙-配置续（推荐）」；实现员 2026-09-27 交回并打上，落点是 `write_acquired_instance` 里的 `highest_journal_tail_of`：每块盘两槽里全部自证过、fsid 与本池相同的系统配置槽 journal_tail 取最大，一份都没有时 0，在第一道屏障之后、第一个取号写之前读一次）。取号失败的回卷写（D18（块里携带什么信息） 已定项 11）带同一个见证值，不退回 0。mkfs 写 tail 0，mkfs 之后第一次取号读到 0、写 0：新池新建文件的字节不变。取号那一刻按同一个判不出（R2，见 D22（单元原子性怎么合成） 已定项 8）判：判不出就在第一个取号写之前拒（实现落点 `crates/singlefs-core/src/transaction.rs` 的 `self_verified_system_configurations_of_every_device`，已取 R2；这一步在某块盘一份自证过的系统配置都没有时也拒（用户 2026-10-03 定留着），多次挂载崩溃点重放里的注入同样由 E165（恢复重放有界穷举与变异杀伤界） r3 择根与重放合判一族判，C331（择根倒挂压过已确认的写） 已还清，见欠账表已还清表）。系统配置没见证到的最新根，「续」落地之后由下一次挂载在自己的 c_见证 里看到；这一形记欠见 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 的 Q1。

**所选根走读失败、系统配置没见证它时退回见证到的那一版**（用户 2026-10-07 定 F2「乙：三处都做」，被攻过零轮）：恢复、可写挂载读阶段与只读挂载择根三处用同一套逻辑，退到 [见证到的那一版, 所选根) 里读得出、不被所选根的实例表抛弃的最新一条根（根按择根的序比：checkpoint_txg 为主、实例代号破平局）；三处判退不退看同一个并集：所选那一版的走读、打开、建上一版三样读者读过的单元与记录（末条记录读不出的那一条也算）里有一样读不出、解不开或判不过，且系统配置没见证它，就退；并集只用来判退、不判成败，落到哪一版之后恢复看走读、只读挂载看打开、可写挂载看建上一版，成不成各照自己的读者；退不成就留在所选的那一版。三处交给判法的记录是同一种来源：触发之后重扫一遍 journal 环、用重扫的那一份；可写挂载里 F2、建退到那一版要的那条记录与 F1 再判（`F1_AFTER_F2`）同用触发之后那一遍（C755（恢复与两种挂载触发退回的失败不同），用户 2026-10-20 定「三句都补」，补的三句零轮措辞；实现在实现批）；见证到的那一版 = 根环里与计数器 = c_见证 那条记录 (txg, 实例) 相等的那一版。所选根的实例表读不出时只认与见证到的那一版恰好相等、读得出的那条根，见证到的那一版与所选根的实例表指针相同时另收区间里与所选根同实例、读得出的根，当它不被抛弃（两个指针都在根记录里，不读表；C751（只读挂载在实例表读不出时越过已确认的版本），用户 2026-10-20 定「甲：共用实例表指针才收」，零轮；实现在实现批）；那条记录读不出、c_见证 = 0、那一段里没有读得出又不被抛弃的根都不退；c_见证 落在所选那次发布自己的某条记录上时那一段是空的，所选那一版算见证过、不退（`F2_NEWEST`，判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 H4 那一行与第三节第 1 条，只在攻方模型上量过、被攻过零轮；那条记录读不出与 c_见证 = 0 不退照 `research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节 T5 行）。所选那一版算「见证过」：它那次发布带末条标志的记录读得出、计数器 ≤ c_见证；末条读不出时拿计数器 = c_见证 的那条记录比 (txg, 实例)，不比所选那一版旧就算见证过；两条都读不出时当见证过、不退。F2 走不到「区间里有一次末条记录读得出、根读不出的发布」那一形，论证两半：不注读故障时 F2 不被调到；每次发布都轮换系统配置，c_见证 最多落后一次发布（射程：不注读故障、系统配置轮换写落盘；C758（无读故障时末条读得出根读不出的发布没量），用户 2026-10-20 定「再补一条用例钉住」）。射程之外系统配置写静默丢失那一形（两块盘的系统配置写报成功、不落盘，再覆盖写 30 次，所选根树表坏）钉成用例：恢复、只读挂载、可写挂载都退到区间里最新的可读根，根槽被盖掉的那几次发布在它之下（`crates/singlefs-harness/tests/system_configuration_write_lost_silently_falls_back_without_a_read_fault.rs` 的三条用例）。

**失败的处置按失败的性质分两支**（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P1 失败表）。**判别子是当下判得出的量，不是次数**：失败发生时先对**目标设备的固定落点做一次探针写**——写不进去 ⇒ 持续失败，走**转只读到下次挂载**；写得进去，再**对这次失败的那个落点做一次只读复核**：读得出 ⇒ 瞬时失败，走**实例切换**；读不出 ⇒ 落点级的持续坏，同样走**转只读到下次挂载**。⚠️ 复核这一步**只读不写**：写探针进失败落点会抹掉那里的权威数据（8 个失败点里 4 格的落点是根槽或系统配置槽），而固定落点这一步不许改——`D2`（RAID 条带策略） **已定项 13** 逐字「与 `D23`（journal 的角色与格式） 已定项 14 的两支判别子同一个动作」靠它成立（主 agent 现查：那句话在 `.claude/kb/decisions/02-RAID条带策略.md` 里，归属已定项 13「降级期间只读」，不是已定项 15）。**失败点是屏障时没有落点可复核**，那几格只有固定落点探针写这一步。可写设备数掉到 w 的下限以下、切换自己的预留拿不到，同样直接走只读支；连续切换次数越过 **N_switch = 3**（具名可调参数）也转只读，它是兜底的兜底，真正的判别子是探针写加落点只读复核。**收敛靠的是切换的第一步就是一次会失败的写**：切换要先取号，取号要写独占打开成功的每一份系统配置、全或无 ⇒「写不出去」这类故障最多让切换走一步（探针写先拦，侥幸过了全或无也立刻判失败）。

- 切换要用的块在挂载准入时预留：**它是内存里的一个空间量**，每次挂载重算、崩溃即丢、**不落盘**——分配记录条目只有「已分配」与「已释放 + 释放代」两个态（D3（空间分配） 已定项 7），盘上没法表达第三个态，而落盘表达它本身就要写单元、回到「要写才能写」。预留量按 **(N_switch + 1) × 一次切换的最坏量**（一次挂载允许 N_switch 次切换，且实例表链每切一次长一行、写行要 COW 重写整条链，第 k 次比第 1 次贵；多的一份给这次挂载写行那次发布的元数据）；**这个量的口径与式子的权威正文在 D28（挂载期承诺量） 已定项 3**：一次切换的最坏量 = 实例表链重写（副本数 2 × 32768 × 片数，按设备算）+ 暖机空发布（至多 R 次 × 现算 c_max），**固定点重做与号 > W 的数据单元重写不另要空间**——切换是挂载内一次恢复，重建态里失败那次的分配不存在，重做走 checkpoint 的保留池；引这一段时引 D28（挂载期承诺量） 已定项 3。
- **预留挡的是分配失败那一路，不挡写失败那一路**——写失败由探针写与取号全或无挡。
- **「分配失败」按它发生在哪里拆三支**（不拆的话池一满就整卷只读，而释放空间要写盘、重挂又被预留合取挡住，撞了 `.claude/rules/fs-design.md` 那句「释放空间这个操作本身不需要申请空间」）——前台事务的用户数据分配失败 ⇒ **向调用者返回 ENOSPC**，checkpoint 照常发布、不动挂载状态；后台、可续做、非决策路径（重平衡、墓碑回收、scrub、整理）的分配失败 ⇒ **暂停那个活并记进意图**，不动挂载状态（`.claude/rules/fs-design.md` 第三格「无戒律」、D2（RAID 条带策略） 已定项 4 c「前台不停」）；**切换自己的预留拿不到** ⇒ 才转只读。
- 根槽写失败重发时 checkpoint_txg 推进一格再发（根环区域 = `txg mod R`，D22（单元原子性怎么合成） 已定项 2，不推进就是反复重写同一个槽、把 R 个失败域用成一个）；⚠️ 由此「同一个根槽连续失败」不再是一条判据——槽位随 txg 轮转，N_switch ≤ R 时它永远够不着，留着它读起来像一道闸、实际是空的。
- **实例切换** = 挂载内做一次恢复：取新实例代号、写行、重发在飞 checkpoint——所选根**重新读盘择根**：对着仍打开的设备句柄按恢复择根的同一套规则现选（`crates/singlefs-core/src/recovery.rs` 的 `choose_root`），不取调用方内存里记的根——根已落盘而系统配置槽那一步失败时，内存里记的根比盘上的现行根旧（C458（实例切换取内存里的根，不重新读盘））；行只落在 [这个根的实例, 新实例)，每个被照旧事务的写序实例 k 写 (k, k 最后发布的 txg（没发布过根时 0）, 被照旧的、写序属于 k 的最大事务号)，其余按 D18（块里携带什么信息） 已定项 11（旧实例发布过根、被照旧的事务都是它自己的时候，就是 (旧实例, 最后发布的 txg, 被重发的那个 checkpoint 里的最大事务号) 这一行），号 ≤ W 的事务照旧（开放 checkpoint 里已完成的事务全部 > W，C287（切换收养开放 checkpoint 的事务后再崩） 按这个 W 封死，注 4）、号 > W 的按新写序重做或向还没返回的调用者报错，固定点单元全部按新实例重写；不转只读、不等下次挂载，代价 = 一条行 + 整条实例表链重写 + 号 > W 的数据单元重写 + 一次固定点重做，频率未测。

**这一版的失败处置**：发布不接受失败。一次发布失败之后把它冻结，下一次发布之前先逐字节原样重发它（checkpoint_txg、计数器、本次发布内序号、记录标志、单元的位置与字节都不变），重发成功才建下一次发布；于是合法历史里同一 (实例代号, checkpoint_txg) 不会有两条带末条标志的记录，恢复遇到它就是盘坏了，在任何写之前停下（`crates/singlefs-core/src/recovery.rs` 的 `RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided`）。原样重发管这一版的串行提交；探针写、两支分流、实例切换与根槽写失败推进 txg 仍在本项，随失败处置的代码在后面的里程碑做。原样重发在串行提交下会把一次失败放大成整条发布流阻塞，欠 C541（原样重发把一次失败放大成整条发布流阻塞）。

**射程**：管恢复施加哪一段、回退与切换怎么做、失败怎么分流；实例表的行格式与写行规则归 D18（块里携带什么信息） 已定项 11，切换预留的量归 D28（挂载期承诺量） 已定项 3，defer 窗口与 F 归 D16（发布语义） 已定项 1。已知边角：

- ⚠️ 它对已定项 3 是收窄不是推翻：「先全环扫描、逐条验证、不许先信 tail」原样成立——变的是验证出的合法前缀里**哪一段被施加**。
- ⚠️ **影子账与按实例表判抛弃（`abandoned_by_table`）留着，理由是崩溃恢复，不是管理员回退**：一段没有任何管理员回退的历史里，崩溃恢复因暂时读错落到旧根、实例表抛弃了较新的根，关掉影子账，被抛弃的根引用的数据单元被复用，之后恢复落在它上面读不出（第一轮 H6）。「只在崩溃恢复落到旧根的那次挂载里隔离」挡不住它：被复用的写发生在下一次干净的挂载里（第二轮辩方 G6，读代码的判断、没量）。落 [checks-owed.md](../checks-owed.md) C314（崩溃恢复落到旧根后可以复用被抛弃的根引用的单元）。 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙-配置续（用户 2026-09-27 定）之后，系统配置见证过的较新的根暂时读不出不再被抛弃：可写挂载重读一次，仍读不出就拒可写；影子账留着的理由收窄成系统配置没见证到的那一形（那次发布的系统配置轮换没落盘）；例外：被抛弃根的根槽、记录与每块盘较新的系统配置槽同时读回全零时，见证过的那一版同样被抛弃（已定项 14 边角②）。
- ⚠️ **C331（择根倒挂压过已确认的写） 判不出取 R2 之后的代价（取 R2 的权威写法在 D22（单元原子性怎么合成） 已定项 8；取甲时量的数在 `research/prompts/unreadable-at-mount-r2-main-verification.md` L1）**：撕裂的系统配置轮换（普通断电就造得出，被撕裂的那次发布没确认过）不再因判不出拒可写挂载；仍判不出的是有槽设备报读错、没有一块盘两槽都自证过那一形（一槽系统配置持续读错），可写挂载每次都拒，只读挂载不写盘，那一槽没有机会被重写——池只剩只读挂载；「只读挂载不写盘」对 2026-09-28 的代码量实（2016 条轨迹），条款上没有，见 C32（无密钥的一侧写不写得出字节没有闸）、C33（scrub 检出之后做什么没有条款）、C75（降级挂载这道闸自己不会红）。一次可写挂载都做不成时怎么走出来，没有条款（C596（见证判不出的系统配置盘面只剩只读挂载））。C393（被抛弃根的账读不出时修复没有条款） 从映射也现算不成的那一形同样只剩只读挂载。
- ⚠️ **挂着时回退的已知边角三样**：① 「R_old 仍分配的落点在 cur 的账里是同槽同分配代」这道核不在定案里，也补不上——已释放的记录记的是释放代（`crates/singlefs-core/src/allocator.rs` 的 `AllocationRecord`），核看不出「这个槽被复用过又释放了」；它放过的读不对的回退只在 F 回落时走得到（第三轮判决第二节 K1：F 生效取最大值的最小值时 15 格中 12 格，取 SysPre 时 0），靠 F 不回落挡。② 崩溃恢复抛弃的根在重开时全部暂时读不出，影子账读不到它们的账、隔离 0，之后别的实例复用了它们的槽，撤故障再把新实例的根全改坏，恢复落在被抛弃的根上读不出——与回退形态无关的多重故障，见 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到）——乙-配置续挡住的是见证槽读得出、所选那一版之后的根读不出的那一种（重读一次仍读不出就拒可写）；被抛弃根的根槽、记录与每块盘较新的系统配置槽同时读回全零时，见证过、已确认的那一版同样被抛弃，与射程末条「F2 的代价」同一类（三方判决 `research/prompts/m3-c554-c572-r1-main-verification.md` 第二节「被抛弃实例的根重开时全部读不出」那一行）。③ 两个状态来回回退超过根环一圈时，可退到的不同状态少于 4 个（D16（发布语义） 已定项 1 的根环容量边界）。
- 实现的链首锚在所选根覆盖的最后一条（`crates/singlefs-replay-prefix/src/lib.rs` 的 `counter_of_the_last_record_the_root_covers`，`crates/singlefs-core/src/recovery.rs` 的 `replay_journal` 经 `replay_core` 取前缀），对着它的会红检查是 `crates/singlefs-harness/tests/writable_mount_takes_a_second_instance.rs` 的 `one_missing_record_right_after_the_chosen_root_stops_the_prefix_even_when_later_records_are_intact` 等用例，与 `crates/mutations.tsv` 里名字带「链首锚点错一位」的行（C365（恢复路径链首的锚点没有会红的检查） 已还清，见欠账表已还清表）。
- 回退时标签 6 的 n_live 现算读法（「准入与记账」一格）被攻过零轮。
- ⚠️ **只读复核买的是落点持续坏那一类，代价是它自己零轮。** 固定落点探针写单独用时，落点持续坏那一类会反复白切换（攻方在副本装置上量到 24 个持续故障场景合计 46 次，那个数没在入库装置重做过、不进 kb）；只读复核能分辨它。这条改法是攻方腿自己提的，**被攻过零轮**，用户 2026-09-21 知情接受。
- **切换重新读盘择根带出的三件**：读盘选出的根比内存里记的新时，不重发在飞 checkpoint，以读盘选出的根为准接着走；切换写的实例表行与 W 按读盘选出的根写；回退那次发布里，管理员选的 R_old 必须在「以读盘选出的根为最新根」算出的回退候选集里，不在就拒绝、盘上逐字节不变。挂载内实例切换的实现随里程碑「覆盖写、释放、回退与复用」收口表第 16、40 行在后面的里程碑（C458（实例切换取内存里的根，不重新读盘））。
- ⚠️ **零单元发布这一支不冻结**：发布末尾系统配置轮换之后那道屏障报错时，带单元 / 带记录的发布与轮换报错同一处置（冻结、原样重发）；零单元发布（`crates/singlefs-core/src/transaction.rs` 的 `publish_without_units`，末尾走 `persist_the_root_then_rotate_the_system_configuration` 那一路）这一支不冻结、整个挂载返回错误（失败经 `PublishError::BlockDevice`），下一次发布落在下一次挂载、换了实例代号；这是替条款没写的一支做的选择（判决 `research/prompts/m2-closeout-code-r2-main-verification.md` 第一节「最新根读不出、见证与屏障」那一格；`m2-impl-c577-barrier-implementer-report.md`「停下交主 agent 的设计问题」第 2 条）。钉它的用例是 `crates/singlefs-harness/tests/a_zero_unit_publish_whose_post_rotation_barrier_fails_returns_the_mount_error.rs` 的 `a_zero_unit_publish_whose_post_rotation_barrier_fails_is_not_frozen_and_the_whole_mount_returns_the_error`：只做过 mkfs 的池第一次可写挂载，写行那次零单元发布在盘 1 轮换之后那道屏障报错一次 ⇒ 挂载报 `MountError::Publish`、原因 `PublishError::BlockDevice`，写账已落盘的发布 0 份、失败的 1 份；录制流落到盘 0 那道屏障为止，盘 1 那一道没进录制流；换回不报错的设备之后再可写挂载，取号 2、择到 (实例 1, txg 1)。它只钉文件镜像上的结局：harness 的文件设备没有写缓存，盘 1 的轮换写在屏障报错之前已经进了镜像，「屏障报错时轮换其实没持久」那一种崩溃状态它不罩（`m2-impl-r2-fixes-a-implementer-report.md`「结论」第 4 行、「停下交主 agent 的」第 5 条）。
- ⚠️ **C366（暖机路径把 txg 写进计数器与 tail） 那条定案在可写挂载这一层没有可达盘面**：注 3「前缀末的计数器取环里读得出的最大那一个」只在所选根自己那条记录两份都读不出时与 txg 分得开；那一形在可写挂载的读阶段被 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙判成「两条都读不出按真」，重读一次仍为真就在取号之前拒可写（C579（判据在单故障合法状态上也拒可写） 用户定维持乙字面），走不到取号、写行与暖机。钉住的是：函数层 `crates/singlefs-harness/tests/warm_up_journal_counter.rs` 的 `warm_up_after_a_journal_counter_other_than_zero_counts_records_on_from_it_while_txg_stays_one_and_two`（环里末条 jsn 40 时暖机写 jsn 41、42，txg 仍是 1、2）；同一份文件的 `c366_when_the_chosen_root_own_record_is_unreadable_the_writable_mount_is_refused_by_the_undecidable_witness_and_the_read_only_mount_opens_the_chosen_root`：可写挂载拒在任何写之前、两块盘逐字节不变，只读挂载照常择 (实例 1, txg 4)、环里读得出的 jsn 1–3 一条不施加（`m2-investigate-three-reds-report.md` 第一节第 2 条、第七节；`m2-impl-r2-fixes-b-implementer-report.md` 第三节 5c 行）。
- 实例表删行的清扫（D18（块里携带什么信息） 已定项 11 的「回收」一条）保护被抛弃根引用的单元：读成功的全零根槽与读不出同待遇（按「可能有该实例的根」算），这一次不删；其余的槽须自证过，否则重读一次、仍不过就这次不删（零轮，推的，没造盘面）。
- ⚠️ **退回见证到的那一版的代价**（用户 2026-10-07 定 F2 时知情接受）：已确认的版本在较新的系统配置槽与它的单元同时读回全 0 时，恢复、可写挂载与只读挂载悄悄读回上一版；可写挂载在上一版上取号之后，那一版被抛弃（推的，没量）（`research/prompts/m3-c694-r2-main-verification.md` 第二节「K3 恢复一侧补法」那一行、第三节第 7 条与第四节 F2）。
- 第一版有活快照时管理员回退拒：现行版或目标版树表里有码 14 / 15 条目就拒（用户定案）。放开回退时的形态（树表整张取目标版、释放集与复活集按活头 ∪ 活快照 ∪ deadlist 算、销毁中的快照不算引用）是攻方模型上 0 违例的零轮形态，记在判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S8。

**依据**：

- E78（重放的起点）：陈旧 tail 叠上块重用之后，「全环扫描 + 断号即止 + 幂等」三件套不闭合——失配即中止的恢复在健康镜像上自我中止，失配跳过的撕裂判别力归零；水位臂与尾删臂都过全部判据，取水位臂是因为水位就是根记录已有的 `(实例代号, checkpoint_txg)`（D22（单元原子性怎么合成） 已定项 7），零新增格式字段，而尾删臂要在恢复路径里加一个重试环。
- E77（发布的持久顺序）：拿掉施加前的点名单元验证，单元与记录同段那一臂当场出静默嫁接——第四条承重。
- E104（扫描重建的现行版本判定）：新根取 T_old + 1 时压不过被抛弃的根——注 2、注 3 新实例的第一次发布从环里最大 + 1 起靠它；它的另一半「不写回退行时被回退抛弃的单元在全规则臂上复活」仍承重——设计就是不写回退行（墓碑第一轮判决 `research/prompts/m3-tomb-r1-main-verification.md` 第二节、`research/prompts/m3-delete-r3-main-verification.md` 第三节「墓碑第一轮「一轮」各行」）；扫盘重建认「挂载根是一次回退」靠谱系记录，读侧怎么用记欠账（K-W5b，与 C113（扫描重建时多版单元的现行版本判定无输入） 同组）。
- E150（回退复用被抛弃的根引用的单元）：影子账在模型里两格零违例，多隔离的单元到被抛弃的最新根被轮转覆写为止。模型里的被抛弃时间线是管理员回退造出来的；影子账留着的理由是崩溃恢复抛弃的时间线（第一轮 H6），机理相同，模型没重跑那一形。
- E142（新池新建文件的干跑）：干跑时装置取「两份镜像任一份合法即在」，改坏一份记录仍读到全部合法记录——「记录在」的口径由此收口。
- 三方：`research/prompts/c199-r1-main-verification.md`、`research/prompts/c199-r2-main-verification.md`（注 1–4）；`research/prompts/c322-r3-main-verification.md`（取号屏障与跳过的号）；`research/prompts/c329-c330-r3-main-verification.md`（切换的所选根与 W 那一行）；`research/prompts/c143-r3-main-verification.md`（第一个新根的 txg 取 CJ2）；`research/prompts/m2-r1-main-verification.md` 第三节与 `research/prompts/m2-step45-code-r3-main-verification.md` 第六节 5（影子账查谁的账、有效根）；C113（扫描重建时多版单元的现行版本判定无输入） 定案（`_c113-r9-final.md`）。
- 用户定案（重放下界、回退、失败表、切换预留口径、两份镜像的口径各一次），原话在变更史。
- `research/prompts/c381-r3-main-verification.md` 第三节：候选乙（探针写改成写失败的那个落点）被三处独立打中——屏障上没有落点、8 个失败点里 4 格的落点是权威结构、与用户第二轮定的第 1 题（落点进地址空间表，买的就是「写坏了也不心疼」）对撞、`D2`（RAID 条带策略） 已定项 13 那句话会当场为假。
- 用户定案 2026-09-21（改法 B），原话在变更史。
- 用户定案 2026-09-23：切换的所选根重新读盘择根（`research/prompts/c381-r3-main-verification.md` 第二节记的零轮形态、第四节第 3 题），原话在变更史；被攻过零轮。
- 用户定案 2026-09-23：注 3 的前缀末计数器取环里读得出的最大那一个（C366（暖机路径把 txg 写进计数器与 tail） 实现员查出所选根自己那条记录读不出时条款没写），原话在变更史；这条定案在可写挂载这一层没有可达盘面（射程「C366（暖机路径把 txg 写进计数器与 tail） 那条定案在可写挂载这一层没有可达盘面」那一条），C579（判据在单故障合法状态上也拒可写） 用户 2026-09-27 定维持乙字面、拒可写（`m2-investigate-three-reds-report.md`），原话在变更史。
- 用户定案 2026-09-23：第五条的发布边界按已定项 17 那条约定认末条（C245（系统配置槽的写频率，两条条款说反话） 第二轮 `research/prompts/c245-r2-main-verification.md` 第四节列的三个判别量之一，被攻过零轮），原话在变更史。末条按记录标志位 0 认（已定项 17）；一次发布的记录里没读到带标志的那条，整次不施加。
- 用户定案 2026-09-23：注 1 的锚点取同 txg 里 jsn 最大的那条、锚点读不出时靠本次发布内序号接链首，第一版没有干净关闭标记（三方第一轮 `research/prompts/m2-presumed-clauses-r1-main-verification.md` K3、K4；K4 的两个改法都被攻过零轮，本次发布内序号那一个是推的，定案时没实现，现已实现：记录头的「本次发布内序号」字段，已定项 4），原话在变更史。
- 用户定案 2026-09-24：第一个文件版本写之前核上一条记录，照 2026-09-24 的实现写成条款（C495（第一个文件版本核上一版记录没有条款），三方 `research/prompts/m2-refusals-presumed-r1-main-verification.md` 第三节 R5：拒之前零字节写、被拒那一族零故障不可达），原话在变更史。
- 用户定案 2026-09-24：切换重新读盘择根带出的三件照主 agent 推荐定（判决 `research/prompts/m2-newq-r1-main-verification.md` 第三节 N5–N7，原话在变更史）；无实验：三件都不改盘上格式、实现不在这个里程碑，差别只在语义。
- 用户定案 2026-09-24：末条再跨记录、最后一条带标志位（原话在变更史；三方 `research/prompts/m2-treesplit-r1-main-verification.md` T6（维持拒绝时拒不拒取决于池的历史），按定案之前的字面认末条会施加半次发布，带标志位那一格 0 / 0）。
- 用户定案 2026-09-24：末条按标志认、注 1 的锚点读法乙（原话在变更史；实十三报告 `m2-p6-header-implementer-report.md` 第四节 ① 列的两种读法）。
- 用户定案 2026-09-24：这一版发布失败原样重发（实二十报告 `m2-lastflag-implementer-report.md` 第六节 1：同一实例失败后接着发，会留下两条同 (实例代号, checkpoint_txg) 都带末条标志的记录），原话在变更史；无实验：只定失败之后发什么，不改盘上格式。
- `research/prompts/m2-final-code-r2-main-verification.md` 第三节 Z8（这一版发布失败原样重发，攻方没打中）；那一份的 Z9 与第四节改法 1–4 是回退见证的，随见证删掉不再承重。
- 管理员回退改成挂着时的一次向前发布（候选集、释放、复活、水位、拒）、删前缀第五条与回退见证、影子账依据改指 H6：无实验，三方原型（`research/prompts/m2-rollback-forward-r1-opus-model/`、`m2-rollback-forward-r2-opus-model/`、`m2-rollback-forward-r3-opus-model/`）在冻结副本上量过，没立实验号，原型上的数不是入库装置上的数；三轮判决 `research/prompts/m2-rollback-forward-r1-main-verification.md`、`m2-rollback-forward-r2-main-verification.md`、`m2-rollback-forward-r3-main-verification.md`。水位取 max(内存, 环) 的收严（第三轮判决第二节 K1 的水位分句）被攻过零轮；用户定案 2026-09-25（向前发布、只在挂着的时候做）与 2026-09-26（第三轮交用户的四问），原话在变更史。
- 主 agent 2026-09-26 定：回退那次发布之前逐盘验复活集里的每个单元（「在任何写之前拒」那一格最后一句），照用户 2026-09-25 原话「1 就是fsync失败后 两个盘掉一个盘。这个不能认，这个违背我们数据安全的承诺」（`records/2026-09-24-里程碑二收尾调度.md` 第三节那一时刻的一行），与 C519（丢一整块盘时恢复丢掉刚确认的那一版） 那一格「两份都验过才施加」同一方向；无实验，推的，被攻过零轮；多读的量随 R_old 之后释放的用户可见单元数涨，没量。
- 用户定案 2026-09-25：点名单元两份都验过才施加、认下丢一整块盘时丢掉刚确认的那一版（C519（丢一整块盘时恢复丢掉刚确认的那一版） 实一复现：`crates/singlefs-harness/tests/whole_device_loss_after_warm_up.rs`），原话在变更史；无实验：照 2026-09-25 的实现写成条款，另一读法「任一份验过」会不会放过 E77（发布的持久顺序） 那种嫁接没量。
- 用户定案 2026-09-27（C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙-配置续，先选乙、再补选「配置续（推荐）」）：可写挂载读到系统配置见证过的更新状态读不出，重读一次仍读不出就拒可写；取号那一写不再写 tail = 0，改写取号那一刻按同一取法读到的见证值（实现已合入，`crates/singlefs-core/src/transaction.rs` 的 `highest_journal_tail_of`）；判据 N-配置续 定义在 `e158-r3-prereg.md`「5.1　判据 N 用哪一样」与 `e158-r4-prereg.md`「5.1　判据 N 用哪一样」，原话在变更史。E158（择根与修复四岔路） 第 4 次跑：第三段 H1g 把乙-配置与乙-配置续分开（16/32 对 0/32），第二段（以装置撞键修过的 seg2b 为准）四族两者逐族相同，页上「2026-09-27：第 4 次跑」一节的岔路表第 1、2 行。
- 回退时标签 6 的 n_live 现算读法：`research/prompts/m3-blanks-r1-main-verification.md` 第四节「回退那一刻 n_live 怎么现算」那一行（零轮；主 agent 采纳正推腿的读法）。
- 三方判决 `research/prompts/m3-ring-r3-main-verification.md` 第二节第 9 行：清扫把读成功的全零根槽当没有根，会抹掉仍在环里的被抛弃根引用的单元，违反这一项正文明文的保护；全零根槽不当「没有根」、每槽自证过或全零否则重读一次仍不过就不删，两处收严采纳（零轮，推的，没造盘面）。无实验：保护的对象是被抛弃时间线的残余，不是活数据。
- 回退到删完的一版（有树、文件集合为空）算候选：主 agent 2026-10-02 定「Q-K 照收」，候选集原文括注「那一版写过文件」的读法（调度记录 `records/2026-10-01-删除与并发事务.md`「十六　批 8」）；2026-10-03 主 agent 照收第九位实现员的同一选择（「回退到删完的一版照收，与 Q-K 一致」）。无实验。
- 重建分配器读根环两遍并起来没有有效根的拒另起拒因、单子加一行，与候选集去掉一圈窗口那一个合取：用户定案 2026-10-05（Q3「另起拒因、单子加一行」与按槽并两遍），原话在变更史；判决 `research/prompts/m3-batch10-code-r1-main-verification.md` 第二节（`Recovery(NoValidRoot)` 不在单子上那一行）与第六节、`research/prompts/m3-batch10-code-r2-main-verification.md` 第六节；复算判据取自 `crates/singlefs-harness/src/mount_transparency.rs` 2026-10-05 的复算。候选集那一个合取是 D16（发布语义） 已定项 1 的六处退回环里最旧有效根（用户 2026-10-04 定）之后漏改的残留，调度记录第 601 行点名。无实验：数在三方腿的副本上量，没立实验号。
- 被抛弃根的账读不出时有凭据才放、可写挂载时的预隔离、记录有缺口时拒可写、所选根走读失败时退回见证到的那一版：判决 `research/prompts/m3-c694-r1-main-verification.md` 第二节 K1、K2 各行与第四节，`research/prompts/m3-c694-r2-main-verification.md` 第二节 K1–K3 各行与第三节第 1、2、6、7 条，岔路单 `m3-c694-r2-forks.md`；用户定案 2026-10-07（C694（被抛弃的根的账读不出时正常卸载被拒）「C 与 B 都做」，F1「乙：缺记录就拒可写」，F2「乙：三处都做」），原话在变更史。无实验：数在三方腿的副本与模型上量，没立实验号。
- 可写挂载时的预隔离并上较新可读根的账（`C_NEWER_ROOTS`）、账读不出而记录不齐时从映射现算（`C_MAPPING` 收窄）、F2 退了之后再判一次 F1（`F1_AFTER_F2`）、F2 退到区间里最新的一条根（`F2_NEWEST`）：判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 K2 那四行（H1–H4）、第三节第 1、3 条与第四节；用户定案 2026-10-08「四形都修」，原话在变更史。C_MAPPING 那一拒列进单子、与 C393（被抛弃根的账读不出时修复没有条款） 那一拒分开报：用户定案 2026-10-08「乙：core 另起错误成员」，原话在变更史。较新可读根那次发布记录全读不出、账也读不出时映射算得出就护住、算不出不拒：主 agent 2026-10-08 定（`records/2026-10-01-删除与并发事务.md`「第 39 批（删除线）」，删除线第 38 批实现员停下的第一件）。四条改法只在攻方模型上量过，这五处都被攻过零轮。无实验：数在攻方腿的模型上量、主 agent 复跑，没立实验号。
- F2 判退看三样读者的并集、退不成留在所选那一版、交给判法的记录取触发之后重扫的那一份（C755（恢复与两种挂载触发退回的失败不同））、F1 再判之前重扫环（C754（退回之后再判缺记录不重扫环））、F2 在不注读故障、系统配置轮换写落盘时走不到末条读得出根读不出的发布（C758（无读故障时末条读得出根读不出的发布没量））、取号之前逐盘核读本盘环里末条记录那一读不算按干净关闭分支（C499（干净关闭标记有没有没有条款））：判决 `research/prompts/m3-close-rootring-r2-main-verification.md` 第二节 Y3、`research/prompts/m3-close-rootring-r3-main-verification.md` 第二节 C755（恢复与两种挂载触发退回的失败不同） 那一格；用户定案 2026-10-20（并集那三句「三句都补」、再判重扫「重扫」、F2 论证「再补一条用例钉住」、干净关闭那一读「认，点名这一读」），原话在变更史。无实验：数在三方腿的副本上量，没立实验号。
- 所选根的实例表读不出时、见证到的那一版与所选根实例表指针相同才收同实例的根（C751（只读挂载在实例表读不出时越过已确认的版本））：判决 `research/prompts/m3-close-rootring-r3-main-verification.md` 第二节 C751（只读挂载在实例表读不出时越过已确认的版本） 收窄形态那一格（收窄形态被两腿在同一实例清扫发布换表那一形各自打中、复跑对上，不写回）与第三节交用户表甲那一行；用户定案 2026-10-20「甲：共用实例表指针才收」，原话在变更史。无实验：数在三方腿的模型上量、主 agent 复跑对上，甲是零轮。
- 回退那次的 inode 号水位只取没被崩溃恢复抛弃的根（Opt2），max 里留着 cur 那一版内存里的，树 ID 水位照旧取环里全部根（「水位」那一格）：W3 三轮判决 `research/prompts/m3-w3-r1-main-verification.md`、`research/prompts/m3-w3-r2-main-verification.md`（第二节攻方：取环里全部根时，被崩溃恢复抛弃、读得出的高水位根让回退跳升，W3 的历史核对在合法镜像上误报）、`research/prompts/m3-w3-r3-main-verification.md`（第二节 Opt2 那几行：两次抽样与正推都没造出重发或误报，max 里去掉 cur 内存那一项就重发；代价是合法的跨抛弃重发多 11.6%）；用户定案 2026-10-08 Y1「Opt2：只取没被抛弃的根（推荐）」，原话在变更史。无实验：数在三方腿的模型与仓副本原型上量，没立实验号。
- 退役树同墓碑树、谱系树，随现行那一版、不取 R_old：判决 `research/prompts/m3-retire-r1-main-verification.md` 第二节 K3 那一行（两次攻方抽样都没打中）；用户定案 X1 甲（原话在变更史）。
- 判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S8（第一版有活快照时回退拒；放开回退时的形态是零轮）与第七节用户定案（岔路 3「第一版有快照就拒」，原话在变更史）。

**欠**：C77（重放起点未定义）；C314（崩溃恢复落到旧根后可以复用被抛弃的根引用的单元）；C554（崩溃恢复抛弃的根暂时读不出时影子账算不到）；C334（切换的所选根没有会红的检查）；C287（切换收养开放 checkpoint 的事务后再崩）；C126（切换预留的最坏量没有口径）；C493（回退候选集条文与实现说反话）；C381（根已落盘之后发布失败，分配器仍退回）——三方第二、三轮打中失败表那两支判别子（探针写在盘上没有落点、屏障类失败点连「失败的那个落点」都没有定义），用户已定「落点进地址空间表、射程逐个失败点列一张表」，第 2、4 题已定（`D16`（发布语义） 新立一条已定项、失败表改两步判别子），「实例切换取的是内存里的根」那一题定成切换重新读盘择根、带出的三件待定（`research/prompts/c381-r2-main-verification.md`、`c381-r3-main-verification.md`）；代码那一半（探针写、只读复核、实例切换、转只读）跟里程碑「覆盖写、释放、回退与复用」收口表第 16 行一起挪到后面的里程碑；C458（实例切换取内存里的根，不重新读盘）：切换重新读盘择根那条会红的检查；C541（原样重发把一次失败放大成整条发布流阻塞）；C558（回退目标是环里最旧的根时回退那次发布会写坏它）；C596（见证判不出的系统配置盘面只剩只读挂载）；C866（F2 退回不分字节丢了与内容拒收）。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:465-479`（整段抄，未转述）**

```markdown
#### 已定项 15：崩在记录持久之后、根槽持久之前，恢复由记录重建那次发布的根

**定案**：**由记录重建那次发布的根：记录头加「新根段」= 树表单元指针 86 + 中央映射树根指针 86 + 树 ID 水位 8 + 回退下界 F 8 + 实例表单元指针 86 + 分配记录树根指针 86 + 卸载记号 4 = 364 字节 <!-- format-const: JOURNAL_NEW_ROOT_SEGMENT_BYTES = 364 stale=新根段 188 字节|JOURNAL_NEW_ROOT_SEGMENT_BYTES = 188 -->，4096 记录装 64 个点名项。施加一条记录 = 把所选根的这七个字段换成记录里的（实例表指针、卸载记号、分配记录树根从新根段来，不再照抄所选根），不需要树语义。一次发布切成几条记录时，每条记录都带同一份新根段。已定项 14 条 2「严格大于所选根就施加」原样成立。**

**射程**：管「记录在、根槽不在」那一格恢复施加什么；指针的宽度随 D19（块指针的结构与宽度预算） 已定项 7 / 11 走。施加一条记录时指针层上其余的事（分配器登记、overlay）不在这一格，归 C284（施加一条记录在指针层上做什么没有定义） 与 D28（挂载期承诺量） 已定项 2。

**依据**：

- E142（新池新建文件的干跑）：新根段加进记录头之后，「施加一条记录」在指针层上有了定义，忽略 journal 与查 journal 的恢复结果在一批状态上不同——journal 承重。
- 用户定案，原话在变更史。
- 用户定案 2026-10-01：「多条记录的发布每条都带同一份新根段」补进已定项 15（`research/prompts/m3-inv-b-r1-main-verification.md` 第四节第 3 条），原话在变更史。
- 三方判决 `research/prompts/m3-ring-r3-main-verification.md` 第二节第 1、2 行：施加处的实例表指针、卸载记号、分配记录树根三项改从新根段来（首个文件、卸载两种发布补进去之后施加出来的根与当初会写的逐字节相同，攻方量的；专用清扫发布崩在记录已持久、根槽没持久时，实例表指针照抄所选根会让下一次可写挂载在取号之前被拒，攻方在副本上量的）；用户定案 2026-10-02「回到甲」，原话在变更史。无实验：数都在三方副本上，没立实验号。

**欠**：C284（施加一条记录在指针层上做什么没有定义）。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:480-495`（整段抄，未转述）**

```markdown
#### 已定项 16：mkfs 写实例代号 0，第一次可写挂载取 1

**定案**：**mkfs 写 0，0 = 「mkfs、尚无实例」，不是有效实例；第一次可写挂载按 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2 取 max(系统配置, 根环) + 1 = 1，并写进每一份系统配置（一次系统配置槽写，世代号 +1）之后才动单元。mkfs 写出的单元写序 (0, 0)、第 0 代根实例代号 0；D18（块里携带什么信息） 已定项 11 那句「从 1 起、0 无效」读成「实例从 1 起，0 保留给 mkfs」。**

- **取号那一步的屏障**：取号那两次系统配置写之后、本实例第一个非系统配置写之前至少一道完成了的屏障，首次挂载路径上暖机开场那道就算、不另发；那道屏障完成之前本实例的全部写者都不发单元、记录、根；那道屏障在任一块盘上报错 ⇒ 取号全或无失败，不许只重发屏障就继续。取号的 max 读法与回卷写带的号写在 D18（块里携带什么信息） 已定项 11，世代号按 D22（单元原子性怎么合成） 已定项 16 逐盘 + 1。

**射程**：管实例代号从几起、取号那一步的次序；实例表的行怎么写归 D18（块里携带什么信息） 已定项 11。⚠️ 取号之后，写行或暖机那几次发布因块设备报错失败时，已写进系统配置的新代号**不回卷**——回卷只管取号自己那几次系统配置写（D18（块里携带什么信息） 已定项 11）；这个号就此烧掉，下一次可写挂载按 max + 1 取下一个，被烧掉的号按 D18（块里携带什么信息） 已定项 11 的写行规则得一行。这是已知行为，不当缺陷修。

**依据**：

- 无实验：实例代号的起点是编码约定；取号屏障的判别力在崩溃点重放里，不是计数实验。
- 三方：`research/prompts/c322-r3-main-verification.md` 第五节（取号那一步的屏障，三轮三方之后用户定案）；实例代号的起点是用户定案。原话都在变更史。
- 用户定案 2026-09-23：取号之后写行或暖机失败不回卷，认了、写成已知行为（里程碑「覆盖写、释放、回退与复用」收口表第 57 行），原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:496-515`（整段抄，未转述）**

```markdown
#### 已定项 17：点名项 56 字节的字段表

**定案**：**一项一个单元（新池新建文件 12 项）：位置条目 14 × 2（偏移 0）+ 单元类型标签 1（28）+ 出生树 8（29）+ 出生 txg 8（37）+ key 尾段 10（45：码 1 写序；码 2 / 码 3 实例代号 4 + 出生序号 4 + 补零 2）+ flags 1（55）= 56 <!-- format-const: JOURNAL_NAMED_ENTRY_BYTES = 56 -->；不另带载荷 CRC（位置条目各带 4 字节校验和）。重放从点名项直接凑出映射 key。**

- **记录点名的每一项自带校验和**：D4（校验和位置） 已定校验和内联进父指针，而记录点名的东西在记录持久时它的父还没发出 ⇒ 记录临时充当父，否则重放时那些单元没有任何完整性凭据；恢复重放施加前必须逐项验证点名单元（已定项 14 第四条）。
- **一次发布切成 N 个事务、N 条记录时，这次发布共享的提交内生块（extent 树节点、inode 叶容器与根、分配记录树、记账树、中央映射树、树表单元）只在最后一条记录里点名**，前面几条不重复点名它们。
- **共享的提交内生块多于最后一条记录装得下（4096 记录 64 项）时，末条再跨记录**：从最后一个事务那条记录起依次多写几条，装满一条再开下一条，只有真正的最后一条带「本次发布末条」标志；恢复按这个标志认一次发布的边界，一次发布的记录里没读到带标志的那条，整次不施加（已定项 14 第五条）。标志放在记录头十个字段里「填充 1」那个字节（改名「记录标志 1」），位 0 = 本次发布末条；每一次只有一条记录的发布（含空发布记录），那一条的位 0 也写 1——改新池新建文件的字节（w1、w4、t13 三条记录的这一字节与头部校验和 [46, 50)；w4、t13 两条的反向链 [91, 95) 也跟着变，E142（新池新建文件的干跑） 空白 G26 逐字节核过）。用户 2026-09-24 定。

**射程**：管点名项的字节；位置条目的构成归 D19（块指针的结构与宽度预算） 已定项 4 / 11，映射 key 的形态归 D19（块指针的结构与宽度预算） 已定项 6 / 10。⚠️ 共享的提交内生块只在最后一条点名时，崩在最后一条持久之前，那些单元没有任何记录点名——重放的回收路径要认得出这一格，崩溃点重放的小负载要覆盖它（C491（多条记录时共享内生块在哪条点名没定））。

**依据**：

- E16（journal 的角色：WAL vs 意图日志）：记录点名的东西其父此刻还没写到盘上，记录必须临时充当父——逼出「点名的每一项自带校验和」。
- E77（发布的持久顺序）：点名项自带校验和、施加前逐项验证，是单元与记录之间那道屏障得以省掉的交换条件。
- 用户定案，原话在变更史。
- 用户定案 2026-09-23：共享的提交内生块只在最后一条记录里点名，原话在变更史。
- 用户定案 2026-09-24：末条再跨记录、最后一条带标志位（原话在变更史；三方 `research/prompts/m2-treesplit-r1-main-verification.md` T6（维持拒绝时拒不拒取决于池的历史），按定案之前的字面认末条会施加半次发布，带标志位那一格 0 / 0）。

**欠**：C376（发布边界在链尾没有输入）。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:516-530`（整段抄，未转述）**

```markdown
#### 已定项 18：在飞记录数上限的取值与语义，tail 字段的编码

**定案**：在飞记录数上限 = 环槽数 ÷ F（`F` 是 I-8.1（环几何够大） 的安全系数），默认环下 196608 ÷ 3 = **65536**，写进系统配置；tail 8 字节存 jsn 的 48 位计数器，靠系统配置自己的两槽轮换取冗余，不另设 tail 槽；记录 n 落在环内偏移 `(计数器 − 1) mod 槽数 × 4096`。

- **语义**：这个数的含义是**一次恢复重放的前缀最多 N 条**。读到一条校验和不过的记录时，恢复**断链即止**——停在它之前那一条，不追问它是「撕裂」还是「损坏」，也不因此拒绝只读挂载；可写挂载另按 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙-配置续判（已定项 14）：系统配置见证过比断链处那一版新的发布、重读一次仍判真，拒可写。不是 XFS 那种「最后 N 条内的坏校验和算撕裂、之外算损坏拒绝挂载」。

**射程**：⚠️ 代价：一条真正的「盘中段损坏」与一条「尾部撕裂」在第一版判成同一种结局，池照常挂上、损坏那一条之后的记录全部不施加。第一版串行提交（D13（验证路线） 已定项 2），任一时刻只有一条记录在飞 ⇒ XFS 那套在第一版里 N 恒 1，而它要的那个「N = 并发写出的记录数」与「环槽数 ÷ F」是两个量纲不同的数，同一个字段装不下两种语义。

**依据**：

- 无实验：取值是 I-8.1（环几何够大） 的算术（环槽数 ÷ F），语义是一次口径裁定——断链即止已经是 I-8.3（重放前缀严格连续) 的判据，这一句只是把「校验和不过」也并进同一条出口，不新增分支。
- 用户弹窗定案，没有走三方论证，可推翻，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:531-547`（整段抄，未转述）**

```markdown
#### 已定项 19：空发布记录的事务号与提交标记、`previous_hash`、环长

**定案**：

- **① 空发布记录**：事务号 0 保留给不承载事务的记录，提交标记写 1；不进实例表 W 的 max，不进「丢掉提交标记没出现的尾巴」判定。
- **② `previous_hash`** = CRC32C(**本实例内逻辑前一条**记录的头，其中 `header_csum` 那 32 字节按零参与)；**本实例写出的第一条记录恒 0**；写路径**不读盘**。反向链是**按实例**的——I-8.3（重放前缀严格连续) 逐字「跨实例边界即停」，重放本来就不会从一个实例的记录接到另一个实例的记录上，实例边界另有实例代号挡着；取「环上前一格的记录头」那一种，写路径每写一条记录就要先把环上那一格读回来，买到的只是「跨实例也能连上」，而那条链没有任何判定在用。字节表里 w1 那一行写 0 的理由是「本实例第一条」，不是「环上无前一条」。
- **③ 环长**：环长是 **mkfs 参数**，**默认 768 MiB**；约束 **环 ≤ 设备容量 ÷ 4**，越界**拒绝 mkfs**。系统配置里仍只存「环长度（字节）」这一个字段，不另存容量或比例。**挂载时有效 `T_dirty` = min(`T_dirty` 字段, 环长 ÷ F)**——系统配置里 `T_dirty` 仍写 2 GiB，默认 768 MiB 环下有效值 = min(2 GiB, 256 MiB) = 256 MiB。单元区起始槽号随环长走，默认环下是 784 MiB（槽 50176）。`T_dirty` 的夹取是 I-8.1（环几何够大） 的直接推论——环装不下 `F ×` 最坏占用时，能脏的字节数必须跟着降，否则一次发布发不出去。

**射程**：③ 的「参数 + 上界」让大盘上 768 MiB 只占容量的百分之几、小镜像按参数缩，`环 ≤ 容量 ÷ 4` 挡住「环吃掉整个盘」；⚠️ 连带：`.claude/kb/layout/01-first-txn.md` 的预想镜像是 **4 GiB**（1 GiB 的镜像在这条约束下装不下 768 MiB 的环）。实现：`crates/singlefs-core/src/make_filesystem.rs` 收环长参数、按最小那块盘的容量 ÷ 4 拒绝越界，默认值是 `crates/singlefs-format/src/lib.rs` 的 `JOURNAL_RING_DEFAULT_BYTES`。 挂载读者一侧另判环长：在飞上限（环槽数 ÷ F，已定项 18）≥ 1，且环末端不越过编译期单元区起点（768 MiB，`crates/singlefs-format/src/lib.rs` 的 `UNIT_AREA_START_SLOT` 与 `JOURNAL_RING_START_SLOT`）；越界整池拒绝挂载（实审 A3a 落地：`crates/singlefs-core/src/recovery.rs` 的 `SystemConfigurationValueOutsideWhatThisReaderAccepts::JournalRingBytesOutsideTheSupportedRange`）。

**依据**：

- E143（一事务一单元下的 journal 账）：按切分纪律，`T_dirty` 满窗要的环是预想 64 MiB 环的 4 倍，要么环 ≥ 768 MiB（F = 3）、要么 `T_dirty` 压小——默认环长与 `T_dirty` 夹取的来处。
- ①、②、③ 都是用户弹窗定案，没有走三方论证，可推翻；主 agent 推荐的也是这几档，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:548-572`（整段抄，未转述）**

```markdown
#### 已定项 20：记录承载指针层的目标态，分配先于 journal

**定案**：**journal 记录承载「指针层的目标态」，不是逻辑意图，也不是全量物理重做。** 单元由 COW 写到各自最终位置并自带校验和（D22（单元原子性怎么合成） 已定项 18 里「COW + D4（校验和位置） 够」那一类），journal 只记「哪些子树根换到了哪个块 + 新根」。

- **由此得到一条写路径的硬顺序：分配先于 journal。** journal 记录不是「事务的全部」，是「**已经写到盘上的东西的发布指令**」。
- 判据：

  | 判据 | 逻辑意图 | 全量物理重做 | **指针层目标态** |
  |---|---|---|---|
  | 幂等（I-4.4（意图只描述目标态）「只描述目标状态」） | 只有写成目标态才幂等 | 天然幂等 | **天然幂等** |
  | 重放要不要分配器 | **要** | 不要 | **不要** |
  | 记录大小（D16（发布语义）「小、顺序写」） | 小 | **大**（要带全部字节） | **小** |
  | 记录核对器（D13（验证路线） 已定项 7）核对一条记录要什么 | 自行实现树语义 + 记账 + 分配合法性（核对从句法变成语义） | 逐字节比对 | **格式解析 + 校验和 + 根环择新** |

- **否掉逻辑意图的第一条理由（独立成立）**：O2（独立解析器 + checker） 明令不许调用被测 crate 任何代码（D13（验证路线）），而逻辑意图的重放要重新实现分配器 + 树操作 + 校验和 + 记账，**约等于第二个文件系统**。第二条理由（落点不确定使核对退化）是第一条的一条封堵，不是独立理由（已定项 5）。
- 记录形态与 D13（验证路线） 的验证手段互相挟持：验证手段的可行性决定记录形态，记录形态决定验证手段的产出等级，两者要一起定。

**射程**：管记录里装什么、分配与记录谁先；施加一条记录在指针层上换哪几个字段是已定项 15，其余归 C284（施加一条记录在指针层上做什么没有定义）；重放幂等这条硬要求在已定项 3。

**依据**：

- 无实验：方向由 2026-08-28 三方论证按幂等、重放要不要分配器、记录大小、核对要什么四条判据推出（主 agent 裁决），四条都是结构性质，没有可量的对照；ZFS 的 ZIL 靠「把主写路径再调一遍」回避独立实现的成本，是线索，不是依据，在变更史。

**欠**：C284（施加一条记录在指针层上做什么没有定义）。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:573-593`（整段抄，未转述）**

```markdown
#### 已定项 21：世代号：记录粒度的全局单调号，不是 D5（快照 / 空间记账机制） 的 txg

**定案**：**记录头要两个计数字段，职责不同、不可合并**：

| 字段 | 粒度 | 用途 |
|---|---|---|
| `jsn` | **每条记录**，全局单调，10 字节（实例代号 32 位 + 计数器 48 位，已定项 9）| D22（单元原子性怎么合成） 要求②的世代号；前缀判定靠它 |
| `checkpoint_txg` | 每个 checkpoint | 归属哪个 checkpoint，供 D5（快照 / 空间记账机制） 的 birth/death 记账 |

- 理由：D5（快照 / 空间记账机制） 是「一个 checkpoint 一个 txg 号」⇒ 同一 checkpoint 窗口内的多条 journal 记录**共享同一个 txg**，彼此分不出先后，D22（单元原子性怎么合成） 要求的「被实际检查的世代号」就失去意义。
- ⚠️ jsn 取到不回绕的宽度，**买到的是「免掉回绕比较」，不是新能力**：jbd2 的 tid 是 32 位且会回绕，靠 `tid_gt` 的有符号差比较；回绕比较正是独立实现最容易实现得不一样的地方。
- ⚠️ **「用 txg 就不需要 cycle」只对轴①成立**，见 D22（单元原子性怎么合成） 那两个正交轴的表。

**射程**：管两个计数的职责；两者在水位与择根里谁为主比，归已定项 14 第 2 条；「每次发布 checkpoint_txg + 1」归 D16（发布语义） 已定项 6。

**依据**：

- 无实验：职责划分由 D5（快照 / 空间记账机制） 的粒度直接推出，是定义之间的推导，没有可量的量。

**欠**：无（C85（发布计数器的四个等号没写死） 2026-10-11 已还清，并入 C88（根环的时间线判别未实现），见欠账表已还清表）。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:594-605`（整段抄，未转述）**

```markdown
#### 已定项 22：前缀判定：严格连续、断号即止

**定案**：**前缀判据是 `jsn == expected`，不等即止。** 只用「`jsn` > 上次已应用」过滤**不够**：若序列里有空洞（5849 写成、5850 没写、5851 写成），「>」会把 5851 错接到 5849 后面，**破坏因果序**。还要一个**显式的在飞记录数上限**（取值与语义见已定项 18）。前缀判定的完整口径（五条）在已定项 14。

**射程**：⚠️ 断号即止在**那一次**恢复里是对的（前缀停在 5849），它单独挡不住**上一条时间线的残留**：恢复之后接着写时，5851 那个槽里还躺着上一条时间线的记录，**校验和是好的、`jsn` 恰好等于 expected** ⇒ 第二次崩溃后的恢复把它接上去重放。在 D23（journal 的角色与格式）唯一测过的那种槽位映射（`slot = jsn % 槽数`）下这由构造必然发生，位置与序号解耦时退化成巧合，而解耦救不了。现有的三道闸（I-8.3（重放前缀严格连续）、在飞记录数上限、同一窗口里一模一样的 `checkpoint_txg`）都不触发；tail 推进不是出路（tail 只在 checkpoint 持久之后前移、残留从未被应用 ⇒ `tail ≤ min(残留 jsn)` 恒成立）；重放幂等也救不了（它是自反的：「**同一条记录**被重放两次与重放一次，结果相同」，残留是**另一条**记录，它点名的那些单元在被丢弃之后可能已被新时间线重分配）。这一格由已定项 8 / 9 / 10 挡：反向链、序号高位带实例代号、链值覆盖实例代号；恢复时物理作废残留槽要求原地覆写，对 zoned 不成立。

**依据**：

- 无实验：「>」会把空洞之后的记录错接到空洞之前，这是序列定义上的反例，不需要量；残留那一格的实验在已定项 8 的依据里。

**欠**：C42（残留记录冒充合法前缀）。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:606-626`（整段抄，未转述）**

```markdown
#### 已定项 23：三个死锁各自怎么破

**定案**：journal 相关的三个死锁，两个落在格式上、事后补不进去：

| # | 环 | 对本工程 |
|---|---|---|
| 1 | journal 满 → checkpoint → 发根 → **发根要不要写 journal** | D16（发布语义）那句「不绕过 journal」取读法甲（已定项 25）：发根本身不是一条 journal 追加 |
| 2 | **COW 独有**：checkpoint → COW 写新节点 → 要分配 → 盘满 → 发不出根 | **保留池治这一条**，`reserve ≥ ckpt_cost` 即可（已定项 24）。⚠️ **D22（单元原子性怎么合成） 的 K 加重它**：盘快满时最该救命的空间正是被 K 扣住的 ⇒ **K 必须是运行时策略，不能是格式常量** |
| 3 | 单条操作大于环 | 需要等价的几何不变量：**任一操作的最坏 journal 占用 ≤ 环大小 / F，F ≥ 2**（[invariants/03-journal.md](../../../crates/singlefs-checker/invariants/03-journal.md) I-8.1（环几何够大）），mkfs 时算、mount 时校验 |

- **死锁 3 推出一条依赖关系**：**D8（核心索引结构） 的意图分批机制不是「删大文件的便利设施」，它是 journal 满死锁的必要条件之一**——算不下的操作必须能拆成若干个各自合法的事务。

**射程**：管这三个环怎么断；保留池与准入的分治在已定项 24，读法甲在已定项 25，环几何的具体取值在已定项 18 / 19。I-8.1（环几何够大） 的「占用」是空间量纲，允许一个事务跨多条记录（已定项 7）。I-8.1（环几何够大） 的 checker 已实现（池级 checker 的 `judge_journal_ring_geometry_fields_agree`，core 恢复时同判三条等式，`crates/singlefs-core/src/recovery.rs`）。死锁 2 是 COW 独有的：原地写的 jbd2 / XFS 回家不需要新空间，没有这个环。

**依据**：

- E23（journal 几何）：定长环有死锁 3、要几何不变量，环太小时强制 checkpoint 的次数断崖式上升；链式没有死锁 3，但每个 journal 块都要向分配器要块、进入死锁 2 的环。
- E19（defer 窗口下的假性 ENOSPC）：保留池恰等于一次 checkpoint 的开销就把 checkpoint 卡死打到 0——死锁 2 由保留池断开。

**欠**：无。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:627-651`（整段抄，未转述）**

```markdown
#### 已定项 24：死锁 2 与假性 ENOSPC 是两种故障，要分治

**定案**：

| 故障 | 谁治得了 |
|---|---|
| **checkpoint 卡死**（D23（journal 的角色与格式）死锁 2） | **保留池**，且 `reserve ≥ ckpt_cost` 就够；本工程的 ckpt_cost 按 D28（挂载期承诺量） 已定项 4 现算（各记录树高之和 + 记账节点），不设常数 |
| **假性 ENOSPC** | **只有准入**。保留池反而加重它 |

- **机制**：保留池是给 checkpoint 开的一道地板，对**普通分配**是纯税——前提是保留池已经大到不卡死；前提之外符号是反的，卡死之后空间根本不回收，小保留池反而拒绝得更多。而假性 ENOSPC 的成因是「defer 窗口与根环 K 扣住的量超过剩余空间」，**那不在保留池的射程内**。
- **假性 ENOSPC 的破法是一条可计算的准入规则**：

  > **剩余空间必须 > 每 checkpoint 搅动量 × (延迟代数 + 1) + 一次 checkpoint 的开销**

  ⇒ 不许把盘填到 `容量 − 搅动量 × (延迟+1) − checkpoint 开销` 以上。这与 D3（空间分配） 已定项 15「空间预留取准入控制形态」同构——**「最坏情况」里必须含 defer 窗口与 K 扣住的那一块。**

**射程**：保留池怎么进准入不等式、算不算挂载期承诺量归 D28（挂载期承诺量） 已定项 4，准入不等式的权威形式在 D28（挂载期承诺量） 已定项 1。⚠️ **搅动量在真实负载里是变量**，建模时当常数；怎么估它是真机压测那一半的问题。

**依据**：

- E19（defer 窗口下的假性 ENOSPC）：预留恰等于一次 checkpoint 的开销就把卡死打到 0；再加大预留只让假性 ENOSPC 按固定步长上涨；那条准入规则的预测与实测逐格一致。
- E25（环与链各要多大保留池）：「保留池对普通分配是纯税」只在保留池大到不卡死时成立，卡死区间里小保留池反而拒绝得更多。

**欠**：C83（提交固定点没人回答）。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:652-674`（整段抄，未转述）**

```markdown
#### 已定项 25：D16（发布语义）「不绕过 journal」取读法甲

**定案**：D16（发布语义）那句

> 「没有任何操作可以绕过 journal 直接发根。」

**取读法甲**：

| 读法 | 含义 | 结果 |
|---|---|---|
| **甲（采用）** | 任何最终反映到根里的**语义效果**必须先经由 journal；**发根这个动作本身不是一条 journal 追加** | 断开死锁 1 |
| 乙（字面） | 发根也要走 journal ⇒ 发根要追加记录 ⇒ journal 满时发不出根 | **环闭合，死锁** |

⇒ 它与 D22（单元原子性怎么合成） 兼容：**根环是 journal 之外的固定 K 槽，发根 = 覆写根环的一个槽，不是追加。** checkpoint 不是「操作」，是 journal 机制本身的一部分；普通用户操作仍然只能进 journal。

**射程**：只管 D16（发布语义）那一句怎么读；发布的其余语义归 D16（发布语义）。

**依据**：

- 无实验：这是一句措辞的读法裁定，两种读法里一种直接闭合死锁环，没有可量的量；jbd2 / XFS 的破法（释放空间的路径消耗零日志空间）与读法甲相容，是线索，在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:675-691`（整段抄，未转述）**

```markdown
#### 已定项 26：记录头的两条解析纪律

**定案**：

1. **校验和覆盖的字节范围必须是显式的盘上常量，绝不能由语言的 `sizeof(struct)` 推出。** 独立解析器不会碰巧复制同一个 `sizeof`，这正是 checker 能抓、单一实现抓不住的错误类型。
2. **记录自述长度必须先做边界检查，再用于校验和计算。**

反向链那一条（让记录核对器能用与正向扫描完全不同的遍历复核同一个前缀边界）是已定项 8。

**射程**：管解析 journal 记录头的次序与常量来源；覆盖范围本身（整条 `[0, 4096)`）在已定项 13；已定项 1 的 B 条（自述长度，先边界检查再算校验和）就是第 2 条。

**依据**：

- 无实验：两条都是从现役实现的缺陷现查出来的纪律（XFS 因为用了 `sizeof`，i386 与其他架构算出不同 CRC，背着 `XLOG_REC_SIZE` / `XLOG_REC_SIZE_OTHER` 两个常量和两遍校验的兼容疤；ZFS 的 ZILOG2 从块内读 `zc_nused` 决定校验范围，读之前先 `if (size < nused) return ECKSUM`），约束的是解析次序与常量来源，没有可量的量。

**欠**：无。

```

**出处 `.claude/kb/decisions/13-verification-route.en.md:1-6`（整段抄，未转述）**

```markdown
## D13 Verification Route —— Settled

[中文](13-验证路线.md) · **English** · [日本語](13-検証ルート.ja.md)

D13 (Verification Route) governs how this project verifies correctness: employing all three off-the-shelf approaches—writing a custom reference implementation for differential testing, bounded exhaustive crash testing, and crash refinement formal verification (survey in [prior-art.md](../prior-art.md) Section 7.4)—without slavishly copying them, designing bespoke verification methods tailored to four characteristics unique to this project (COW issuing a new root per transaction, Merkle self-verification, nonce uniqueness for full-volume AEAD, and lack of reference implementation). It settles how oracles are divided, which two items are custom-developed, how crash-point replay is tiered and sampled, how the crash state set is defined, and what is shared between checker and implementation. Out of scope: what each invariant specifies in [invariants/README.md](../../../crates/singlefs-checker/invariants/README.md); how the three verification approaches land and what blocks what in [verification-build.md](../verification-build.md); where commit steps are registered, how structural equivalence classes are partitioned, and how many exist belong to D17 (Implementation Layering and Third-Party Pipelines) settled item 2.

```

**出处 `.claude/kb/decisions/13-verification-route.en.md:7-443`（整段抄，未转述）**

```markdown
### Settled Items

| # | Item | Settlement |
|---|---|---|
| 1 | **Which part crash refinement verifies first** | Verifies the closed unit of "one publish + one crash + one recovery" first; first milestone is that any `jsn` continuous prefix does not split a publish; mount transparency is an independent fourth property; former candidate I-6.5 (Monotonic Nonce Watermark) withdrawn **Status: Settled.** **Progress: Implemented.** |
| 2 | **Whether transaction layer commits to serial commit** | Serial first, opening concurrency only past the intersection; future concurrent commits append records in transaction number order **Status: Settled.** **Progress: Implemented.** |
| 3 | **To what extent custom reference implementation (RefFS form) is pursued** | Taken to the extreme at the semantic level: operates in memory strictly via POSIX operations, defining persisted state and permissible post-crash states without on-disk format; written from scratch per kb, isolated crate, accepts whitelisted inputs only **Status: Settled.** **Progress: Implemented.** |
| 4 | **How crash state set of crash-point replay is defined** | Write stream segmented by device; segment closes only when every disk with writes in segment is released by its own barrier or FUA; each full write in segment takes states combined arbitrarily; in-place overwrites take third state "neither new nor old readable"; non-persisted takes old bytes generating real images; mirrored writes counted per device **Status: Settled.** **Progress: Implemented.** |
| 5 | **What is permitted to be shared between checker and tested implementation** | Shares only a constants module generated from kb, all else implemented separately; decision width probed by checker itself then compared against system configuration, reporting "declared value, unprobed" if unprobed **Status: Settled.** **Progress: Implemented.** |
| 6 | **Three mutually independent oracles** | In-memory model differential testing, independent parser + checker, and independent spec executor each verify distinct aspects; independence verified by "three-oracle collusion test", never self-proclaimed **Status: Settled.** **Progress: Implemented.** |
| 7 | **What O2 (Independent Parser + Checker) verifies** | Domain of O2 (Independent Parser + Checker) is a single image, judging set membership only; checks requiring a second input belong to record checker (inputs include persisted set from enumerator), unnumbered by design **Status: Settled.** **Progress: Implemented.** |
| 8 | **Two custom-developed mechanisms** | Merkle root chain diff (recursive comparison of adjacent generations of roots, skipping unchanged subtree pointers) and generational incremental scrub (traversing only blocks with birth generation above watermark) **Status: Settled.** **Progress: Unimplemented.** |
| 9 | **How crash-point replay is tiered** | Three tiers all exact full run; incomplete enumeration handled by exact pruning, temporarily unrunnable nodes marked ignore and opened gradually; milestone closeout and judge unit changes require main agent popup asking user to run full run, commits allowed on failure or skip with owed check; failure reproducibility prioritized **Status: Settled.** **Progress: Implemented.** |
| 10 | **What N truly must be covered exactly** | Number of equivalence classes of commit protocol structure, not number of layout schemes; implemented via closed enum plus exhaustive match **Status: Settled.** **Progress: Implemented.** |
| 11 | **When ignored nodes are enabled** | Ledger rotating by owed checks rejected; three rules retained for enabling: deterministic coverage records, `T1` (Regular Tier Wall-Clock Budget) as input, and separating discriminative power from coverage checks **Status: Settled.** **Progress: No code object.** |
| 12 | **Conflict 1: Checker-as-specification and verification paths sharing no tools** | Checker implements two directions: traversal and scan, independently judging the same invariant, sharing only format parsing **Status: Settled.** **Progress: Implemented.** |
| 13 | **Conflict 2: Refinement spec and differential model** | Spec and model written differently at representation level; common-origin observed: ≥ 5 inconsistencies with 0 spec errors triggers merge and budget transfer to exhaustive testing **Status: Settled.** **Progress: No code object.** |
| 14 | **Interlocks with other decisions** | Hash and AEAD treated as uninterpreted functions in formal verification; log structure three tiers burden symbolic execution; checker does not parse message semantics **Status: Settled.** **Progress: No code object.** |
| 15 | **Two tiers of verification code** | Harness tier (`singlefs-harness`) for unit and integration tests run anytime; checker tier (`singlefs-checker-tier`) runs quick tier by default at commit, full run upon user request **Status: Settled.** **Progress: Implemented.** |
| 16 | **Slice sizing and state count width for registered segments** | Segments whose domain exceeds u64 judged in slices, defaulting to 2^32 states per slice starting from library resume point; state count uniformly uses 1280-bit fixed-width integer with checked arithmetic; registry read only via environment variable **Status: Settled.** **Progress: Unimplemented.** |
| 17 | **Unit of verdict reuse is function** | Pruning unit is function: verdict reuse depends solely on de-commented fingerprints of dependent function units, toolchain, and resolved dependency versions; commits, tree diffs, full file text, file/crate fingerprints, manual versions, paths, and whole-stage marks forbidden **Status: Settled.** **Progress: Unimplemented.** |

#### Settled Item 1: Crash Refinement First Verifies "One Publish + One Crash + One Recovery"

**Settlement**: The first verification target of crash refinement is the relational property of recovery function `Recover`, whose minimal closed unit is "one publish + one crash + one recovery". The subject `Recover` across the four milestones refers to three call sites invoking `replay_journal` with `verify_named_units = true` in `crates/singlefs-core` (`Consult` strategy of `recover`, writable mount read phase, read-only mount), all three taking `records` from the output of `scan_journal`; `Ignore` and `ConsultWithoutNamedVerification` are arms for testing and control, placed in trusted computing base; `crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs` is not a production caller. The four milestones in sequence are:

| Milestone | Obligation | Placement |
|---|---|---|
| **M1 Totality** | `Recover` is defined, terminates, and outputs an image satisfying I-7.1 (Non-Empty Valid Root Slot Set) and I-8.3 (Strictly Continuous Replay Prefix) on **every** crash state enumerated in D13 (Verification Route) settled item 4 | Scaffold. **Completion criterion is not "proven", but "M2 can be stated on top of it"**—crash-point replay already enumerates full run ([verification-build.md](../verification-build.md) "crash-point replay does not sample, full run enumeration"), recovery hangs are immediately exposed, proving totality alone yields no marginal value. Prefix extraction, root selection, and ring scan are in Verus in `crates/singlefs-replay-prefix/src/lib.rs`: prefix core `replay_core` (termination and non-overflow proven inside core, loops have `decreases`; two `requires`—record table sorted ascending by (instance, counter) and counter has successor—guarded by assertions in glue `replay_journal`), root selection core `choose_root_slot` (takes max (`checkpoint_txg`, instance) among self-verified slots, tie-broken by earliest slot order), ring scan core `first_copy_of_each_record_key` (first come first served for same (instance, counter)), joint verdict written as corollary `corollary_recovery_replays_from_the_newest_self_verified_root`; gate checker-tier-formal-verification checks proof, pinned at 56 verified (`.claude/gate.d/verus-pins.tsv`). E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bound) serves as lightweight tool, its rerun 3 part 2 root-selection joint-verdict family calling real `choose_root`, `scan_journal`, `replay_journal` on ramdisk. Four buggy root-selection variants on legal history K family show only "take first valid" red, other three (ignoring instance, taking lower txg on tie, instance-primary) covered by Verus must-red worlds (`crates/singlefs-replay-prefix/verus-must-red-worlds.tsv` root selection section) and adversarial root ring KA (rerun 3 part 3). Outside core: system configuration selection (`choose_system_configuration`) and two glue sections (root selection slot reading, self-verification checksum, fsid, region-to-disk mapping, slot traversal order; ring scan offset enumeration, hint, parsing and CRC, disk traversal order) are outside Verus, covered by crash-point replay and crash amplification CPU judges invoking `recover` on every crash state (GPU judge rewrites `replay_journal` in WGSL, cannot verify core) |
| **M2 Prefix Does Not Split Publish** | **For any `jsn` continuous prefix, its replay result equals the result of an integer number of published checkpoints—no publish is split by prefix; when prefix stops mid-publish, that publish is wholly unapplied** (D16 (Publish Semantics) settled item 4 "a publish is wholly applied or wholly unapplied"). Two layers. **Syntactic layer** (for arbitrary self-verified record table, unconditional): applied records are integral publishes, in-flight bound and commit markers verified entry by entry, applied counters strictly consecutive from chain head (starting from anchor + 1 if readable; if unreadable, first entry `checkpoint_txg` = chosen root + 1, seq 1); back-pointer chain and named units are results of disk-reading callbacks, phrased as "for every applied record, callback returned true". **Semantic layer** (premise: every on-disk record originates from legal write side of this version, i.e. legal history family of E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bound); same (instance, counter) corresponds to unique content across all timelines): "result" is projection of root plus disk, not whole root—returned root's (instance, `checkpoint_txg`) and four new-root fields equal version j on chosen root's timeline (j not earlier than chosen root), remaining fields equal chosen root except unmount marker (reconstructed root marked "not written by unmount sequence"); on disk every version in (chosen root, j] has readable records, named units verified on both copies, sum of records not exceeding in-flight bound | Placement is I-8.3 (Strictly Continuous Replay Prefix)—its supplementary verdict clause is verbatim this rule ("prefix must not stop mid-publish—a publish is wholly applied or wholly unapplied"), do not create duplicate in [invariants/03-journal.md](../../../crates/singlefs-checker/invariants/03-journal.md). Syntactic layer in Verus core `replay_core` `ensures` (`replay_follows_the_prefix_rules`), must-red worlds in `crates/singlefs-replay-prefix/verus-must-red-worlds.tsv` where "modify verdict" lines alter executable verdict only without touching spec; semantic layer judged within bound by legal history family of E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bound), Verus does not prove. Outside core: glue (flattening tables, wrapping callbacks, assembling reports and new root) covered by E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bound) calling real `replay_journal`, harness tests, and mutation lines in `crates/mutations.tsv`; non-projected fields in root assembly (unmount marker, instance table pointer, allocation record tree root) lack mutation lines, generator of E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bound) writing fixed values; two disk-reading callbacks (back-pointer chain, named unit closure) covered by real callbacks in E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bound), ramdisk read counts, mutation lines in `crates/mutations.tsv`, and read-only mount read-count cases |
| **M3 Idempotence and Monotonicity** | **Idempotence**: `Recover(Recover(c)) = Recover(c)`, proven in two readings—record-level (applying same record twice vs once yields identical root, verbatim I-8.4 (Replay Idempotence)) and fixed-point (replaying with reconstructed root as chosen root, same record table, same reader, same switches returns identical root; premise that records above chosen root do not exceed in-flight bound, and record table satisfies I-8.12 (Non-Decreasing Journal Record TXG for Same Instance): journal records for same instance have non-decreasing txg with counter); second run returning `CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR` with 0 records applied when chosen root `checkpoint_txg` saturates counts as equal. "Is just I-8.4 (Replay Idempotence)" does not hold: I-8.4 (Replay Idempotence) is record-level, fixed-point is distinct property. **Monotonicity** (corollary of M2, not proven separately): under serial commit (settled item 2), persistence order of D16 (Publish Semantics) settled item 7, and chosen root being newest valid root in root ring, for two fault-free crash states `c ⊆ c'` in same run, `Recover(c)` lands on version not later than `Recover(c')`, compared by (`checkpoint_txg`, instance), identical order to D22 (How Unit Atomicity Is Synthesized) settled item 7; reopened if concurrent commit enabled or domain widened to include faults. "Applied transaction set is subset of" reading withdrawn: false even for ideal recovery | Verus core `replay_core` `ensures` proves both readings: record-level and fixed-point (fixed-point premise includes I-8.12 (Non-Decreasing Journal Record TXG for Same Instance)); monotonicity written as corollary `corollary_the_landed_version_does_not_go_back`, root selection and write-side premises in `requires`. Adversarial family of E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bound) judges fixed-point within bound, violations strictly on tables violating I-8.12 (Non-Decreasing Journal Record TXG for Same Instance) (rerun 3 part 1). Checker side (I-8.4 (Replay Idempotence)) implemented, judged on secondary crash in crash injection outside pool-level checker list: `REPLAY_IDEMPOTENCE_INVARIANT` and `judge_the_remount_after_a_second_crash` in `crates/singlefs-checker-tier/src/crash_injection.rs`. O2 (Independent Parser + Checker) cannot judge idempotence: I-4.5 (Deprecated: Requires Two Images) retirement reason verbatim "idempotence requires comparing two images, cannot be judged on single image" |
| **M4 Not Under-Applying** | Premise and target identical to M2 semantic layer. Root returned by `Recover` is precisely j\*: farthest version from chosen root where every version is fully on disk (record has at least one self-verified copy, named units verified on both copies), and sum of records does not exceed in-flight bound. Without this, proving M1–M3 cannot exclude "confirmed complete publish discarded by recovery": a recovery applying nothing passes all literal clauses of M1–M3. "Not under-applying" read as "spurious violations": variant must yield spurious violation to be blocked | Farthest-version criterion of E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bound) judged within bound; Verus must-red world line "stopping at first record above watermark and applying zero" goes red; M4 not proven separately |

**Mount Transparency** (independent fourth property, not third reading of M3): recovering read-only once on crash image yields R1; executing an action on same image—mount only (acquire number, write row, warm-up, mount continuation, and mount-initiated floor raising), or mount plus one publish—then recovering read-only again yields R2, comparing R1 and R2 field by field. Version compared: mount only compares to version recovered to, mount plus one publish compares to version immediately before post-mount publish. Also judged on secondary crash images img2 across three phases during mount: C(recover(img2)) = C(v1) (v1 is version landed by R1, C takes POSIX content and existence), judging both content and fields; in field half, falling back to v1 has fields identical to R1, landing on a version written by this mount judged per field-by-field table, scan report unjudged, floor raising F unjudged from first write onward, F prior to that (including warm-up) judged as "mount did not raise floor"; position of first floor raising write determined by location of floor-raising sequences in mount write table, unlocated (`NotLocatedInTheMountWrites`) marks whole-segment F unjudged. Field-by-field table (R2 against R1):

| Field | Judgment |
|---|---|
| POSIX content and existence | Must be identical |
| Pool identity and immutable config in system configuration (except local device number) | Must be identical; 37 immutable rows compared byte-for-byte except local device number |
| Node size in system configuration | Must be identical |
| Runtime config 36 bytes in system configuration (T_time, T_dirty, three watermarks) | R2 equals built-in default (`to_slot` writes defaults, lacking mount option channel) |
| Unmount marker | R2 constantly "not written by unmount", R1 marker excluded from comparison |
| F and effective F | Mount did not raise floor: R2 effective F and root F equal R1 effective F; raised floor: R2 effective F equals last raised floor from mount |
| Instance code | max(self-verified system config instances on both slots across disks, readable root instances in root ring) + 1, read from crash image |
| Instance table row | Written per D18 (Information Carried in a Block) settled item 11 recovery rows |
| `checkpoint_txg` | Lower bound only: ≥ max(root ring roots txg, ring self-verified records `checkpoint_txg`) + 1 (D23 (Role and Format of Journal) settled item 14 lower bound, allowing skipped numbers) |
| Root slot position | Not judged separately, determined by `checkpoint_txg` |
| Tree ID watermark | Identical when mount does not create first file |
| Root pointers of inode tree, extent tree, tombstones, lineage, deleted_inodes in tree table (latter three are tree table entries, kinds 9, 10, 11, not root record fields) | Identical; if mount resumed, judged by post-resumption version: equals closest step root pointers where txg ≤ R2; tree table unit landing point not compared |
| Root pointers, birth txg, head ID, tree ID, kind for tree table kinds 6, 7, 8, 12 | Must be identical (roots for kinds 6, 7, 8, 12 constantly all-zero); birth txg across roots judged by I-9.14 (Tree Table Entry Birth TXG Invariant Across Roots), head ID by I-9.27 (Tree Table Entry Head ID Value) on single image |
| Instance table and tree table unit pointers | Byte comparison omitted, same as tree table unit landing point |
| Scan report | 0 applied, 0 above watermark |
| Root slot offsets 428..457 (algorithm, nonce, MAC) | Must be 0; single-image property, judged by pool-level checker I-2.4 (Header Checksum Coverage), mount transparency does not compare |
| magic, flags remaining bits, self-verification checksum, whole-slot checksum | Not compared (structural, judged at slot read) |
| System configuration slot generation | Not compared, judged by I-7.16 (System Configuration Slot Generation and Slot Number Relation) on R2 image |
| System configuration journal_tail | R1 ≤ R2 ≤ max counter of self-verified records in R2 ring, equality holds when mount finishes |
| System configuration journal instance code | Equals R2 root instance code when mount finishes; secondary crash takes either R1 instance or new instance number |
| Allocation records (including root record `mapping_root`, `allocation_record_tree_root`, and tree table kinds 3, 4 roots) | Whole table comparison omitted: POSIX content identical and pool-level checker green on R2 image; `mapping_root`: R2 all-zero ⇔ R1 all-zero; `allocation_record_tree_root`: R1 has tree ⇒ R2 all-zero, R1 tree table 0 entries ⇒ (R2 non-zero ⇔ new instance number ≥ 2) |

System configuration runtime variables (slot generation, journal_tail, journal instance code) falling back to v1 are judged by their own row rules, not identical to v1.

**Scope**: Marginal value of refinement exists only in dimensions unattainable by crash-point replay enumeration—payload bytes, record counts / segment counts / txg counts (including ring wraparound), remnants of previous timeline, slot counts and device counts; crash-point dimension is already fully enumerated by crash-point replay. "Checksum passed" in model is boolean flag, **must not be read as refinement verifying integrity**—isomorphic to D4 (Checksum Placement) line in settled item 14 "must not read as verifying Merkle", and same informed acceptance regarding CRC32C collision probability as D13 (Verification Route) settled item 4. D13 (Verification Route) settled item 4 is `Crash` on left of refinement (enumeration domain of model layer), not `Spec` on right; treating it as spec makes refinement trivially true. Formalization of I-6.5 (Monotonic Nonce Watermark) withdrawn; reopened once D18 (Information Carried in a Block) settled item 14 settles and nonce allocation rules (monotonic counter + pre-allocated watermark, persisted before use) are established as settled item—when reopened, its sole true advantage is that nonces are invisible to POSIX, its spec structurally incapable of sharing origin with model of O1 (In-Memory Model Differential Testing (HashMap Ideal File System)), immune to Conflict 2 (D13 (Verification Route) settled item 13).

Premises and domains for M1–M4: semantic layer hinges solely on "same (instance, counter) corresponds to unique content across all timelines", guaranteed by verbatim re-issue in D23 (Role and Format of Journal) settled item 14 "failure handling of this version"; re-checked when root slot write failure advancing `checkpoint_txg` lands; premise domain takes legal history family of E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bound) (arbitrary landed subset of history written by legal writer, newer root slots may be unreadable). In M2 "remaining fields equal chosen root", root reconstructed from records has unmount marker `NotWrittenByTheUnmountSequence` (D22 (How Unit Atomicity Is Synthesized) settled item 7 "other roots write 0"), which may differ from chosen root marker, hence "except unmount marker". M3 monotonicity corollary holds only under serial commit, fault-free crash states. M4 together with root selection eliminates confirmed complete publishes discarded by recovery under complete premises: fault-free crash states (enumeration domain of settled item 4), chosen root per D22 (How Unit Atomicity Is Synthesized) settled item 7, chain not straddling instance boundaries (D23 (Role and Format of Journal) settled item 14 note 1), new instance roots written across two disks before any fsync returns (warm-up of D16 (Publish Semantics) settled item 8); not eliminated in faulty domain—in-flight bound truncates complete publishes, missing copy on dead disk prevents application (C519 (Losing Whole Disk Discards Just-Confirmed Version)).

Scope of mount transparency: applies only to fault-free crash states resulting in successful writable mount. Rejected writable mounts judged not for transparency, but legality of refusal: refusal legal ⇔ reason in D23 (Role and Format of Journal) settled item 14 list, recalculation holds on crash image, on-disk bytes identical, and read-only read in allowed set; unlisted reasons reported red per crash injection phase 2. Faulty states out of scope: unreadable slot at acquire time and N-config reported as same reason, separated before read-fault tier (C664 (Mount Refusal Legal Recalculation Conflates Two Refusal Reasons)). Tree ID watermark, two tree root pointers, `checkpoint_txg`, and scan report in field-by-field table are inferred; floor raising "raised" half unmeasured; `mapping_root`, `allocation_record_tree_root`, tombstones, lineage, deleted_inodes pointers, algorithm, nonce, MAC, slot generation, and journal_tail classified per C655 (Mount Transparency Field Table Has Unclassified Fields) and C687 (Placement Unjudged for Initial Tree Creation Version) verdicts Y1, Y2 in table.

**Basis**:
- E42 (How Many Records per Transaction): Payload pinned to make "split ⇒ illegal" an identity, structurally blind to "split yet coincidentally legal"—M2 symbolization of payload covers this gap.
- Three-party review: background `_d13-item1-background.md`, outputs `d13-item1-opus-forward-output.md`, `d13-item1-sonnet-reverse-output.md`, `d13-item1-local-attack-v2-output.md`; all three legs ranked former candidate last, cloud legs converging on "prefix judgment and commit atomicity", main agent combining root selection and publish atomicity into single unit (verdict and threshold in history).
- E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bound): Judges M2 semantic layer, M4, and M3 readings on real `replay_journal` within bound, measuring mutation kill bound; user ordered it as lightweight tool on 2026-10-01; C-domain crash state count divided into two cuts (cut A is barrier position of D16 (Publish Semantics) settled item 7); fixed-point violations in adversarial family strictly on record tables violating I-8.12 (Non-Decreasing Journal Record TXG for Same Instance), 0 violations when predicate holds (rerun 3 part 1); root-selection joint-verdict family calling real `choose_root`, `scan_journal`, `replay_journal` retains confirmed complete publishes within bound, legally and fault-free (rerun 3 part 2); four buggy root-selection variants on legal histories K, K9, KW show only "take first valid" red, other three red on adversarial root ring KA (rerun 3 part 3).
- Three-party verdicts (M1–M4 phrasing and placement): `research/prompts/m3-formal-r1-main-verification.md`, `research/prompts/m3-formal-r2-main-verification.md` Section 2, `research/prompts/m3-formal-r3-main-verification.md` Sections 2, 3; M3 monotonicity 4-step derivation in `m3-formal-r2-opus-output.md`.
- Three-party verdicts (M3 fixed-point premise adding I-8.12 (Non-Decreasing Journal Record TXG for Same Instance)): `research/prompts/m3-formal-code-r1-main-verification.md` Section 2 fixed-point precondition and Section 3 item 5; harness premise wider than Verus, reconciliation in scheduling record `records/2026-10-01-里程碑三形式化接入.md` batch 10.
- User decision (2026-10-01 popup, all per recommendation): unmount marker excluded, M4 scope premises stated fully, monotonic version order takes (`checkpoint_txg`, instance), txg saturation takes "report no_successor with zero applied counts as equal"; write-back deferred until K10 verdict, original wording in history.
- Mount transparency: no experiment: judged in crash injection (`crates/singlefs-harness/src/mount_transparency.rs`, called by checker tier crash injection), not experiment; attack numbers measured on replica harness across three verdict rounds, round 2 measured 147,806 initial crash states and 234,773 secondary crash states with 0 red on merged code without main agent rerun.
- Three-party verdicts (mount transparency): `research/prompts/m3-formal-r4-main-verification.md` Section 2 and Section 4 items 2, 3; `research/prompts/m3-formal-r5-main-verification.md` Section 2 U5, U6 and Section 4 items 5, 6; `research/prompts/m3-formal-r6-main-verification.md` Section 2 U5, U6 and untouched fields line; `research/prompts/m3-formal-code-r2-main-verification.md` Section 2 K2 lines.
- User decision (mount transparency plus secondary crash, 2026-10-02 popup, original wording in history): `research/prompts/m3-formal-r4-main-verification.md` user decision line.
- Root selection variant sentence: scheduling record `records/2026-10-01-里程碑三形式化接入.md` batch 12 E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bound) rerun 3 part 3 Q2 handback (main agent added this sentence).
- Mount resumption counted as part of mount, F bound located at end of write table: user decision 2026-10-03 "resumption is part of mount" (original wording in history); code round 3 verdict `research/prompts/m3-formal-code-r3-main-verification.md` Section 3 item 3; implementation `crates/singlefs-harness/src/mount_transparency.rs`.

**Owed**: C77 (Replay start undefined): at seam of root selection and prefix judgment; C649 (Two callback results unverified by Verus): paid off (M2 semantic layer back-pointers and named units remain postconditions unjudged by Verus, covered by mutation lines in `crates/mutations.tsv`, see settled owed table); C650 (Root selection and replay lack joint verdict): M4 scope sentence requires joint verdict to hold; C653 (Ring scan glue memory scales with disks); C654 (Must-red world expected lines drift with solver environment); C655 (Mount transparency field table has unclassified fields): paid off (table completed with padding per verdicts Y1, Y2, see settled owed table).

#### Settled Item 2: Serial Commit First

**Settlement**: Transaction layer commits to serial commit first—implementing `.claude/rules/fs-design.md` rule "when criteria cannot be written, implement only one path, and **implement the slow but correct path first**"; serialization is the explicit default of this project, not an arbitrary pick. Concurrency opens only past the intersection, never promised before it. Future concurrent commit constraint: records appended in transaction number order, fsync for transaction N must wait before return until transactions < N are all appended ∨ invalidated ∨ confirmed instance switch or read-only transition (invalidation is in-memory state of commit pipeline, visible to concurrent successors); this rule idles under serial commit.

**Scope**: Serial ceiling is ~11,000 entries per checkpoint (intersection where single-thread merge time catches up to one device sync, formula in C58 (Serial Commit Hitting Ceiling Lacks Alarm)): below intersection device sync is bottleneck and serialization suffers no penalty; above intersection single-thread merge becomes ceiling with zero safety margin. Settled item 2 does not decide record header field list or alter bytes written by new-pool file creation; it dictates performance ceiling and scope of crash refinement. Concurrent commit will affect crash semantics: "prefix" is total order concept; under concurrency, a prefix may contain complete transaction B alongside truncated transaction A; record header already carries transaction boundary fields (D23 (Role and Format of Journal) settled item 7: transaction number 8 + commit marker 1).

**Basis**:
- E17 (Write Buffer Merge Upper Bound): Single-thread sort-merge ceiling is half the intersection; lock-free parallelism per leaf partition is escape hatch past intersection, but assuming it sufficient is forbidden.
- E44 (Local Measurement and Cost of Sequence Width): Local fsync rate is other half of intersection.
- E42 (How Many Records per Transaction): Under serialization, multi-record transactions can still be split by prefix—record header field list dictated by whether transaction produces exactly one journal record, not serial vs concurrent.
- User decision, recorded in history; future concurrent constraint derived from C113 (Current Version Input Missing for Rebuild Multi-Version Units) settlement P1. Unreviewed by three-party: confirms explicit default; review required when switching to concurrency.

**Owed**: C58 (Serial commit hitting ceiling lacks alarm).

#### Settled Item 3: RefFS Taken to the Extreme, Handling Persistence and Crash Semantics Itself

**Settlement**: Refuses functional-only memory model, building a reference implementation (RefFS) handling persistence and crash semantics itself, elevating RefFS from "reserve candidate" to **active commitment**. "Taken to the extreme" means semantic level: RefFS operates in memory strictly via POSIX operations, defining what counts as persisted after each fsync and which POSIX states are permissible after crash, having no on-disk format; disk-writing and disk-recovering tier (effort approximating building another file system) is rejected. RefFS and implementation forbidden from sharing crash-point enumeration code: "three-oracle collusion test" (settled item 6) guards against shared misunderstanding, most critically "whether RefFS and implementation share identical flawed understanding of crash semantics".

Provenance and boundaries of RefFS:

| Item | Settlement |
|---|---|
| Provenance | Written from scratch per kb text only: does not copy code of O1 (In-Memory Model Differential Testing (HashMap Ideal File System)), does not read recording stream, does not invoke segmentation, state planner, or crash enumeration of implementation |
| Placement | Isolated crate `crates/singlefs-reference-filesystem`: dependency closure (including dev-dependencies) ⊆ {itself, `singlefs-format`}, preferring std where possible; source code does not reference `singlefs_core`, `singlefs_checker`, `singlefs_harness`, `singlefs_checker_tier` (including renamed aliases); neither `singlefs-core` nor `singlefs-checker` include it in dependency closure. Judged by gate checker-independence-and-sync cell checker-implementation-disjoint |
| Input whitelist | Provided by caller, accepts five items only: outcome of each operation (success, rejected before write, errored after write, re-issued as-is); operation index where crash occurs (hi) and its attribution rule; rollback target m; content read on each recovery; explicit publish points |
| First-version operations | First file, overwrite, publish call, fsync, rollback |
| Call sites | At call sites of crash injection and fault injection, RefFS verdict conjoins with all four criteria of O1 (In-Memory Model Differential Testing (HashMap Ideal File System)) crash judge; any failure flags red |

Crash semantics (serial commit, fault-free crash states):

1. **fsync**: Upon successful return of fsync, all operations carried by the publish awaited by this fsync must be present (under serialization, all operations with acquired numbers before call began, counted in units); read by state index, raising lower bound lo of allowed set to end of that publish. Errored fsync does not raise lo, operations of errored publish optionally present; subsequent operations judged against state read on next recovery.
2. **Drop rule: prefix by publish boundary**. When crash occurs during operation hi, recovery permits reading {S_j : j = lo, or j is end of an explicit publish; lo ≤ j ≤ hi}, where S_j is state resulting from applying first j operations after previous recovery, and S_0 is content read on previous recovery. Dropping arbitrary subsets is excluded under serial commit (incompatible with D16 (Publish Semantics) settled item 12), reopened under concurrency.
3. **Raising lo**: Successful publish calls, fsync, and rollback all raise lo (D16 (Publish Semantics) settled item 7: rollback and fsync share return condition); rollback counts as confirmed publish, its index marking publish end.
4. **Rollback**: Rollback is an operation in RefFS; post-rollback content equals content following m-th confirmed publish across entire history; m passed in by caller from root reported by implementation, RefFS does not judge whether target selection is correct. m covers every root-writing entry point (publish call, rollback, write row and warm-up, raise F, unmount), mapping to active version when written, active version including that read on recovery and restarted.
5. **Attribution of crash moment**: Operation index hi where crash occurs is attributed by "which step issued the barrier cutting current segment", not by first intercepted write; used only by RefFS, O1 (In-Memory Model Differential Testing (HashMap Ideal File System)) crash judge untouched.
6. **Post-recovery**: Following each recovery, RefFS first verifies that read state lies within allowed set, then advances from that state as starting point.

**Scope**: Settles ambition level and form of RefFS, not scheduling; must not be read as "building RefFS before implementation"—RefFS is verification vehicle, not milestone (`.claude/rules/format-evolution.md` "start from writing, not reading"). Explicit publish points take only publish calls and fsync in history: until automated batch accumulation lands, RefFS interfaces only with explicit publish points; automated publishes outside RefFS scope, belonging to settled item 1 M2 and record checker; recoveries discarding newest root cause RefFS to restart from read content without judging that step. Faulty crash states (lying devices, whole disk loss C519 (Losing Whole Disk Discards Just-Confirmed Version)) inapplicable. Independence: RefFS reads no recording streams, calls no segmentation or state planning, but its (lo, hi) shares origin with inputs to O1 (In-Memory Model Differential Testing (HashMap Ideal File System)) crash judge disk-level lower bound criterion, logged under C191 (Common-Origin Counter Covers Only Single Pair). D17 (Implementation Layering and Third-Party Pipelines) settled item 4 delegates executable definition of "entry" to RefFS operation surface, fsync persistence defined by RefFS crash semantics; O3 (Independent Spec Executor) remains independent executor of crash sets with unchanged role. First version of RefFS in `crates/singlefs-reference-filesystem`, connected to crash injection phases and fault injection: harness tier depends on it via translation layer `crates/singlefs-harness/src/reference_filesystem_translation.rs`, call sites taking votes from both RefFS and O1 (In-Memory Model Differential Testing (HashMap Ideal File System)) crash judge, flagging red if either fails.

**Basis**:
- No experiment: Ambition level, provenance, and inputs of RefFS are trade-offs between verification investment and boundaries; attack numbers from three review rounds measured on replica harness, unrepeated on main repo (`research/prompts/m3-formal-r5-main-verification.md` Section 5 item 3), uncited here.
- User decision, original wording in history: "take to the extreme" (original settlement); semantic level (2026-10-02, fork ticket `m3-formal-7-forks.md` RefFS level line, reinterpreting original settlement); prefix by publish boundary, call site conjunction, scratch rewrite, isolated crate (2026-10-02, `research/prompts/m3-formal-r4-main-verification.md`); conjunction tightened to all four criteria, rollback target as input (2026-10-02, `research/prompts/m3-formal-r5-main-verification.md`); rollback raises lo, crash moment attributed by barrier, m covers all root entries, whitelist adds re-issue (2026-10-02, `research/prompts/m3-formal-r6-main-verification.md`).
- Three-party verdicts: `research/prompts/m3-formal-r4-main-verification.md` (Round 1: fsync, drop rules, independence, six forks), `research/prompts/m3-formal-r5-main-verification.md` (Round 2: fsync synthesis sentence, publish boundary supplements, conjunction tightening, isolated crate, arbitrary subset rejected under serial), `research/prompts/m3-formal-r6-main-verification.md` (Round 3 final: rollback and lo, scope sentences, whitelist, fsync read by state index).
- D17 (Implementation Layering and Third-Party Pipelines) settled item 4: O1 (In-Memory Model Differential Testing (HashMap Ideal File System)) explicitly states "ignores crash", incapable of defining fsync persistence promise, whereas first runnable target is "correctly committing one transaction"—extreme RefFS fills this exact half.

**Owed**: C447 (Spec executor not connected to crash-point replay to compare against RefFS); C191 (Common-origin counter covers only single pair); C658 (Crash injection third phase permits resurrection of dropped step).

#### Settled Item 4: Crash-Point Replay Enumerates Full Write Subsets Only

**Settlement**: The crash state set to be enumerated by crash-point replay is defined as: recorded write request stream is segmented **by device**—a write is ordered before subsequent writes only by subsequent barriers on its own disk (or FUA writes on its disk); current segment closes only when every disk with writes in the segment is released by its own barrier or FUA; if any disk has unreleased writes, that barrier (or FUA) does not close segment, and writes before and after belong to same segment. A crash state is "preceding segments all persisted, each write in current segment takes one of its states in arbitrary combination, subsequent segments all non-persisted"; enumeration unit is **one full write**. FUA writes release only their own disk: grouped in same segment with preceding unseparated writes on that disk, writes on other disks not persisted thereby. A write takes two states (persisted / non-persisted); **in-place overwrites** (non-unit writes longer than a sector covering pre-existing content, such as system configuration slot writes and journal writes overwriting old records) take a third state "neither new nor old readable", enumerated in both full run and quick tier; third-state images cover entire write range, differing bytes split with first half new bytes and second half old bytes (torn by byte, not sector); torn states of other writes collapsed into "write non-persisted". In "non-persisted" positions, images retain **pre-crash old bytes at that position**, not zeroes, and these images (including third-state) are concretely generated and fed to checker and record checker, never treated as mere counters. Mirrored writes (w ≥ 2 of D2 (RAID Striping Strategy) settled item 6, journal mirroring of D22 (How Unit Atomicity Is Synthesized) settled item 8) counted as distinct writes per **device**, harness state counts computed via closed form per write stream. **Adjacent barriers on same disk collapsed into one**: when recording stream contains consecutive barriers on a disk without intervening writes (barriers on other disks do not count as writes), second barrier omitted (`push` in `crates/singlefs-harness/src/lib.rs`), independent model recording identically; lacking intervening writes, segment produces zero crash states, merging affecting only operation count and kind string without altering crash state set.

**Scope**: Defines enumeration domain at model layer. Whether FLUSH / FUA take effect on real devices as advertised belongs to C6 (Block Layer Semantic Assumptions Flawed), verified in QEMU with real devices. Segmenting by device yields states unreachable in practice (writes released on other disks remain combinable in segment when one disk unreleased), without missing reachable states. Tearing falls into two categories: writes outside in-place overwrites (unit writes, sub-sector writes, writes over empty ranges) differ from "non-persisted" solely by CRC32C collision probability (32-bit, D19 (Block Pointer Structure and Width Budget) settled item 2, D23 (Role and Format of Journal) settled item 11), an **informed acceptance**; tear granularity excluded from model, consistent with D20 (Bearing Surface: Unit Atomicity and Self-Containment) "units with parent pointers depend on no width"; in-place overwrite tears lose old content as well, whereas "non-persisted" retains complete old content, representing distinct states, hence enumerated separately. Crash-point replay sectors take 512 bytes; system configuration slot writes are sole source of third-state on both crash replay streams. Known corner case: third-state old bytes take version where "writes prior to this write fully landed", combinations where earlier overlapping writes in same segment failed to land contain extra bytes (such combination occurs zero times across both streams); multi-sector root slot writes over old content do not support third state in first version (harness assertion; pools built with 512-byte physical blocks, unreachable). Device tearing writes to match checksum collisions excluded from model. D13 (Verification Route) settled item 5 orthogonal: one defines enumeration domain, other defines crate boundaries.

**Basis**:
- E77 (Publish Persistence Order): State count of segment model across four barrier arrangements matches closed form per arm—enumeration domain counted correctly.
- E142 (File Creation Dry Run on New Pool): Rerun 17 part 1 geometry sampling point G7 (positive control on same stream treating only FUA as boundary) measured 4097 states, 2046 violations—lacking barriers, units, journal records, and root slots merge into single segment `[12]`; harness segmentation aligns with `crates/singlefs-harness/src/segments.rs:91` "FUA write closes segment containing it (FUA does not persist preceding writes, grouped together in arbitrary subset)", predicting 2050 / 0 contradicts "FUA forms own segment"—FUA text written per implementation.
- Three-party adversarial review and verdict: [verification-build.md](../verification-build.md) "Three-Party Adversarial Review (2026-09-03)" verdict item 3—every unit class has whole-unit validation (parent-pointed rely on parent checksum, D4 (Checksum Placement) settled item 1, D16 (Publish Semantics) settled item 7, D22 (How Unit Atomicity Is Synthesized) settled item 19; self-verified rely on whole-unit checksum plus checked generation, D22 (How Unit Atomicity Is Synthesized) settled item 21; journal records rely on header checksum and payload checksum, D23 (Role and Format of Journal) settled item 13); torn and non-persisted are identical to checksum.
- User decision (settled per E77 (Publish Persistence Order) and three-party review; FUA segment boundary sentence settled at C313 (Whether FUA Write Counts as Crash Segment Boundary) closeout, updated 2026-09-26 to match implementation "closes containing segment, grouped with preceding unseparated writes"), recorded in history.
- Device segmentation (unreleased disk keeps segment open), FUA releasing local disk only, in-place overwrites taking third state "neither new nor old readable": user decision (code review items 1, 3, 4; 2026-09-27 popup selecting "A merge, track barriers per device", "backfill, run full run", and "in-place overwrite adds third state"); third-state byte-tearing settled by main agent same day, 0 rounds attacked, recorded in history; implementation and discriminative power in `m2-rev-b3a2-implementer-report.md` Sections 1, 3.
- Collapsing adjacent barriers on same disk: user settled 2026-09-28 "collapse, follow today's recorder" (E142 (File Creation Dry Run on New Pool) rerun 19 hit registered stop S19-crates: independent model recorded both, recorder collapsed second, warm-up and full stream differing in op count and kind string but identical in state count, report `e142-r19-runner3-report.md`, prereg `e142-r19-prereg.md` R40), original wording in history.

**Owed**: C6 (Block layer semantic assumptions flawed): reconstructing crash states from VM device logs, where crash tests must turn red.

#### Settled Item 5: Sharing Only a Single Constants Module Generated from kb

**Settlement**: Checker and implementation share exactly one artifact: a constants module generated from kb field tables, where generator reads kb and both sides consume without manual modification (`.claude/singlefs-ai-sop/rules/machine-first.md`: generate repetition, do not copy manually). **All else strictly unshared**: address space newtypes declared separately, format parsing implemented independently, checksums use independent implementations, traversal and accounting code share zero intersection (criterion for C12 (Incremental Semantics Shared) defined by gate checker-independence-and-sync cell checker-implementation-disjoint, enforced as crate-level dependency closure disjointness rather than symbol level; exception "except format parsing and constants" exercised by this item).

- **Decision width must not be obtained elsewhere**: Criteria for I-7.5 (Root Slot Aligned to Decision Width) and I-8.2 (Record Header Does Not Straddle Atomic Unit) depend on `physical_block_size` probed at mount, which is not in image. Checker probes independently, comparing against value recorded in system configuration at mkfs (checker-side form of D22 (How Unit Atomicity Is Synthesized) settled item 2 "mkfs bounds by pool maximum, mount recomputes and compares per device"). **When unprobed, report "declared value, unprobed", never treating as probed**—images backed by files fall into this class, comprising majority of crash-point replay images, hence non-trivial. In heterogeneous pools decision width is a set rather than scalar (C17 (Device Geometry as Global Parameter)), checker recomputing mkfs maximum aggregation to avoid false reds on healthy heterogeneous pools.
- **Generator refuses to emit constants lacking kb placement**, emitting scalar values only: if it emits "offset functions computed from field tables", it becomes format parsing explicitly forbidden from sharing by D13 (Verification Route); criterion is presence of branches or arithmetic in emitted code.
- **Independent CRC32C implementations require pinned parameters** (polynomial, initial value, input/output reflection, XOR output): unpinned parameters yield divergent results on healthy images, producing false reds rather than independence.

**Scope**: Governs crate boundary between checker and implementation; cannot catch erroneous values written in kb itself—shared generated constants and separate manual copying both miss that class, requiring orthogonal gate (C94 (Registered Format Constants Conflict with Later Settlement)). Implementation gap: `crates/singlefs-format` is handwritten rather than generated (values bound by gate code-source-discipline format-constants cell per `format-const` markers in kb, binding marked ones only), generator absent; `crates/singlefs-checker` depends solely on `singlefs-format`, rewriting bitwise CRC-32C; checker reads `physical_block_size` and `io_min` declared in system configuration (`geometry_of` in `crates/singlefs-checker/src/image.rs`), "probed / declared" types defined as `DecisionWidth` in `crates/singlefs-checker/src/lib.rs`: declared form (`DeclaredOnly`) connected to I-7.5 (Root Slot Aligned to Decision Width) judgment (`decision_width_declared_by_the_chosen_system_configurations` taking max(`physical_block_size`, `io_min`) across chosen configs), probed form (`Probed`) exists only as type definition without constructors or connected judgments; I-8.2 (Record Header Does Not Straddle Atomic Unit) unimplemented. `physical_block_size` field at mkfs resides in geometry segment of D22 (How Unit Atomicity Is Synthesized) settled item 9 (4 bytes).

**Basis**:
- No experiment: Sharing boundary is verification independence policy, lacking measurable quantities; discriminative power demonstrated by separate implementations flagging red individually.
- Three-party review: background `_checker-sharing-vehicle-background.md`, outputs `research/prompts/checker-sharing-vehicle-*`, detailed in [verification-build.md](../verification-build.md)—forward leg derived "constants within exception, rest forbidden" from D13 (Verification Route) and C12 (Incremental Semantics Shared); alternate reverse leg and local leg independently constructed finding: errors written in kb missed by both sharing and separate copying.
- User decision, recorded in history.

**Owed**: C17 (Device geometry as global parameter); C387 (Constants module handwritten, not generated from kb); C388 (Checker decision width not self-probed).

#### Settled Item 6: Three Mutually Independent Oracles

**Settlement**: Three oracles each judge distinct aspects, their independence deriving from different sources:

<!-- doc-lint:registry name-col=2 -->

| # | Oracle | What It Judges | Source of Independence | Blind Spots |
|---|---|---|---|---|
| O1 | In-memory model differential testing (HashMap ideal file system) | POSIX visible semantics: content, size, directory structure; content expectation without crash belongs here, permissible post-crash states belong to RefFS (settled item 3) | Touches no disk, parses no disk format | On-disk structural self-consistency; crash semantics; **shared misunderstanding of same POSIX semantics between two implementations** |
| O2 | Independent parser + checker | On-disk bytes satisfy I-1 (Block Self-Description) ~ I-6 (Encryption Invariants) | **Checker calls no deserialization/traversal code of tested crate**, implementing independent parsing directly reading raw device | Pure structural self-consistency only: overwriting block with alternate **valid** content leaves Merkle chain and accounting intact |
| O3 | Independent spec executor | Truncated disk state at each crash point falls within specification-allowed set | **Written in another language** (non-Rust), derived solely from decisions and invariants text; first version in Python (E170 (First Version of O3 Spec Executor)) | Spec itself may be flawed—**meaningful only with positive control**; projected values and unprojected root fields, implementation's own recovery/reader code, system config witness values (journal_tail), write order within publish |

**Independence must be verified, never self-proclaimed**: Design a "three-oracle collusion test"—feeding an intentionally flawed implementation (e.g. constant zero nonce) through all three; if all pass, a shared blind spot is revealed, requiring oracle enhancement. Functional correctness does not rely solely on these three: O1 (In-Memory Model Differential Testing (HashMap Ideal File System)) delegates permissible post-crash states to RefFS (settled item 3), recovery function M2–M4 rests on Verus and E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bound) (settled item 1), and checks requiring second inputs belong to record checker (settled item 7); shared blind spots identified via "three-oracle collusion test".

**Scope**: Blind spots of each oracle tabulated in final column; domain of O2 (Independent Parser + Checker) belongs to settled item 7, dual-input checks to record checker. **Current implementation status**: O2 (Independent Parser + Checker) in `crates/singlefs-checker` (dependent only on format constants crate); O1 (In-Memory Model Differential Testing (HashMap Ideal File System)) implemented as ideal model in `crates/singlefs-harness/src/model.rs` (memory-only, uses `singlefs_format` constants, models no crashes), gate harness-model-differential-and-scenarios running differential testing on random history; O3 (Independent Spec Executor) is Python executor written per kb text, active version being `scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates_journal_header_487/` used in E171 (Three-Oracle Collusion Test) rerun 1 (E170 (First Version of O3 Spec Executor) rerun 2 copied `scripts/experiment-apparatus/e170_second_run_o3_spec_executor/` to `scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates/` adding predicates W, T, then updated 487-byte journal header; rerun 1 archive and original `scripts/experiment-apparatus/e170_o3_spec_executor/` retained): strict execution judges successfully (0 states hit "spec missing clause" on batches 1, 2), state-for-state consistent with implementation (all pass, not evidence of correctness; "selected root not max txg among self-verified roots" and "③ received ≥ 2 slots" paths have 0 states, C661 (Spec Executor Lacks States on Two Paths)), bad state table has discriminative power under corrected expectations (positive control and blind test correct), not yet connected to crash-point replay; judging a crash state: independently recovers version from image per spec recovery rules (system config selection, root selection, prefix per D23 (Role and Format of Journal) settled item 14 five rules), judging not in set if unrecoverable (pool rejection or no self-verified root); projection of recovered version (instance, checkpoint_txg, tree table pointer, central mapping root pointer, tree ID watermark, F) must match projection of a version in version table, with index in [confirmed, started]. Version table consists of active version recovered on base image plus each root slot write in recording stream, projections matching bytes written by implementation; confirmed takes latest version returned before latest landed write; started takes latest version of landed write. System config slots used only to choose slot, determine geometries, and reject out-of-bounds pool fields. Recovery rules written per kb text, allowed set drawn from implementation's own written roots; RefFS (settled item 3) in `crates/singlefs-reference-filesystem`, connected to crash injection and fault injection; "three-oracle collusion test" is E171 (Three-Oracle Collusion Test), read from frozen tree (snapshot summary on experiment page, differs from active `crates/`): O1 (In-Memory Model Differential Testing (HashMap Ideal File System)) column uses model differential plus model crash judge without RefFS (post-crash set half not using RefFS in E171 (Three-Oracle Collusion Test)); O3 (Independent Spec Executor) column uses E170 (First Version of O3 Spec Executor) rerun 1 frozen version; handcrafted bad images fed to O2 (Independent Parser + Checker) and O3 (Independent Spec Executor) only, expectations from E170 (First Version of O3 Spec Executor) designer, Q2 judging "present" per registration (all 4 are P8), judged per F3 as flawed expectation in E170 (First Version of O3 Spec Executor) rather than bad images, zero genuine bad images passed by both O2 (Independent Parser + Checker) and O3 (Independent Spec Executor); independence of origins (table col 4) measured only in Q2 between O2 (Independent Parser + Checker) and O3 (Independent Spec Executor), where patch and expectation derived from E170 (First Version of O3 Spec Executor) code, not treated as independent; O1 (In-Memory Model Differential Testing (HashMap Ideal File System)) excluded from Q2; unmeasured aspects not claimed verified.

**Two aspects unjudged by all three oracles** (first aspect now has judge in crash injection): first, on fault-free crash states, D23 (Role and Format of Journal) settled item 14 writable mount witness criterion holds (witness in system config slot precedes readable records on disk): crash injection phase 2 verifies refusal legality by recalculating criterion on crash image, judging legal if true (mount transparency applies to fault-free states, settled item 1); judge required to ask "crash state lacks injected read fault, why does criterion hold": crash injection phases 2, 3 recalculate criterion on fault-free crash states, judging illegal refusal if true (`RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults` in `crates/singlefs-checker-tier/src/crash_injection.rs`, C670 (No Judge for Writable Mount Witness Criterion True on Fault-Free Crash State) paid off, see settled table). Property "every disk has at least one self-verified system config on fault-free crash state" judged by G4 (crash injection phases 2, 3, connected alongside C670 (No Judge for Writable Mount Witness Criterion True on Fault-Free Crash State)): crash states before mkfs returns outside G4 scope, mkfs writing two slots per disk in single loop before disk barrier, crash injection candidate segments and E171 (Three-Oracle Collusion Test) base images chosen post-mkfs; G4 differs: when no system config selectable, recalculation yields `None`, G4 treats as true, read phase having rejected before reaching cell. Second, write order within publish: I-7.12 (System Configuration Floor Not Lower Than Root on Same Disk) of O2 (Independent Parser + Checker) judges step of writing system config first when raising floor; mutation cases test line 270 of E171 (Three-Oracle Collusion Test) (witness value at crash points post-publish return), line 242 case testing failure path freeze; `layout/01-first-txn.md` Section 8 registers root slot FUA and system config rotation as two segments; other write sequences unjudged by the three.

**Basis**:
- E170 (First Version of O3 Spec Executor): O3 (Independent Spec Executor) successfully implemented per "different language, derived solely from kb text, paired with positive control", judging single crash states fast enough for quick tier; rerun 1 (spec executor updated per 2026-10-03 kb) judged strictly, matching implementation on sample states with discriminative positive controls (unrunnable states and weak discriminative power of run 1 detailed on experiment page).
- G4 (every disk has at least one self-verified system config on fault-free crash state): Code round verdict `research/prompts/m3-formal-code-b26-r1-main-verification.md` Section 2 j2-b established, batch 29 implemented, scope and mkfs segment in `research/prompts/m3-formal-code-b29-r1-main-verification.md` Section 2 L3 and Section 3 item 3.
- Division of labor and independence sources of three oracles are architectural design of verification system; independence observed via "three-oracle collusion test" (E171 (Three-Oracle Collusion Test)).
- E171 (Three-Oracle Collusion Test): On frozen tree, flawed implementations table contains rows passed by all three oracles and rows caught by one oracle only, O3 (Independent Spec Executor) catching zero rows in flawed table; handcrafted forged roots and continuation images include cases passed by O2 (Independent Parser + Checker) but rejected by O3 (Independent Spec Executor) (readings on experiment page Q1, Q2, Q3).
- Three-party argumentation: New round establishing D13 (Verification Route) (local LLM / Sonnet / Opus, non-overlapping stances), background in history.
- Division of labor between O1 (In-Memory Model Differential Testing (HashMap Ideal File System)) and RefFS: three-party verdict `research/prompts/m3-formal-r5-main-verification.md` Section 2 isolated crate cell (both legs aligned).

**Owed**: C445 (Model differential sampling points do not scale with write paths); C446 (Unimplemented invariants lack precondition registry and expiry); C447 (Spec executor not connected to crash-point replay to compare against RefFS); C407 (Three-oracle collusion test judge not in gate): E171 (Three-Oracle Collusion Test) measured once, zero gate cells read output; C672 (Erroneous return value or memory state with unchanged write stream); C679 (Unmount marker excluded from model differential).

#### Settled Item 7: O2 (Independent Parser + Checker) Judges a Single Image

**Settlement**:

> **Settlement: The domain of O2 (Independent Parser + Checker) is a single image**—a unary predicate `P(image) → satisfied/unsatisfied`,
> judging set membership only, never identity. **Any check requiring a second input (another image or a recording stream) does not belong to it.**

**Checks requiring a second input belong to record checker**, with arguments `(pre-crash image, recording stream, post-crash image, persisted set)`—persisted set denotes writes landed in this crash state provided by enumerator (segment full write subset in crash-point replay); on-disk bytes alone cannot distinguish "write not landed, in-place content coincidentally identical" from "write landed", judging reuse and absence requiring persisted set; belongs to binary checks alongside mechanism 1 "Merkle root chain diff" (settled item 8). ⚠️ **Unnumbered by design**—confusion surrounding O2 (Independent Parser + Checker) was caused by using numbers as names; adding O4 merely provides another anchor for future drift. ⚠️ **"Post-crash image" consists of two copies, not one**: harness sequence executes implementation's recovery first, then record checker, recovery itself modifying disk (instance switch writing rows, re-issuing in-flight checkpoints, D23 (Role and Format of Journal) settled item 14) ⇒ root selection and prefix judgment inspect **crash-state image** ("recording stream" being journal records in its ring), compared against **implementation post-recovery image**.

**Scope**: In pointer layer target state, checking a record requires only format parsing + checksum + root ring selection; logical intent records name no on-disk objects, requiring checker to implement tree semantics + accounting + allocation validity—D23 (Role and Format of Journal) rejection of logical intent logs rewritten as guard on first reason in D23 (Role and Format of Journal) settled item 5. Record checker code in `crates/singlefs-checker-tier/src/crash.rs`, connected to each crash state in crash-point replay (gate checker-tier-crash-point-replay). Record checker judgment is thus not pure function of image: two states with identical post-crash bytes but different persisted sets may be judged differently, invalidating image-deduplication speedups (O2 (Independent Parser + Checker) and oracle inspect image only). Crash-point replay excludes writable mount: runs read-only recovery only (with and without journal) on each crash state; acquire number, write row, warm-up never run in crash-point replay. Writable mount covered by crash injection (`crates/singlefs-checker-tier/src/crash_injection.rs`): on each sampled crash state, following read-only recovery and judgment, mounts writable on same post-crash image, publishes once, runs pool-level checker; crashes again during mount, placing secondary crash state in acquire number, write row, warm-up phases, executing read-only recovery, querying model, pool-level checker, record checker: phase 3 write table passed to record checker switches from "prefix from mount segment to current segment" to full history stream + full mount stream + post-mount publish, persisted set corresponding segment by segment (history takes phase 1 persisted set, mount takes secondary crash subset, post-mount publish unlanded). "Compared against implementation post-recovery image": record checker accepts two images, crash-state image judging root presence and record presence (root selection and prefix), post-recovery image judging unit presence and reading landed instance table; phase 2 post-recovery image is pool after mount and publish, claimed version taking newest root written by mount and publish, falling back to version landed in phase 1 if mount wrote zero roots.

**Basis**:
- No experiment: **Basis is type, not cost**: "byte-by-byte comparison" is binary relation `R(image₁, image₂)`; **no amount of investment turns unary predicate into binary relation**; I-4.5 (Deprecated: Requires Two Images) retired on identical threshold.
- "Post-crash image" is two copies: settled by user during D16 (Publish Semantics) settled item 4 scope narrowing review (background `_d16-items134-background.md`), recorded in history.
- Persisted set as input: user decision, original wording in history; discriminator unobservability evidence in `research/prompts/m2-layer0-scale-r3-main-verification.md` Section 2 sector exemption line.
- Crash-point replay excludes writable mount, covered by crash injection: user decision (code review item 2, 2026-09-27 popup selecting "crash injection layer covers writable mount"), recorded in history; crash injection implementation in `m2-rev-b3b-implementer-report.md`.

**Owed**: None.

#### Settled Item 8: Two Custom-Developed Mechanisms: Merkle Root Chain Diff and Generational Incremental Scrub

**Settlement**: Two mechanisms unavailable off-the-shelf and custom-developed for this project:

**1. Merkle root chain diff —— Leverage unique to COW + new root per transaction.**
Historical roots physically exist prior to reclamation, forming replayable chain. Recursively comparing "subtree pointer + checksum/MAC" across adjacent root generations,
**directly skipping identical pointers without descending**, with complexity proportional strictly to actual modifications between generations.
Detects error class undetectable by traversal checker: **in-place writes self-consistent after synchronous tampering**—
a write bypassing COW modifying block in-place while synchronously updating checksum appears self-consistent in isolation (I-2.1 (Checksum Matches Content) all green),
whereas root chain diff exposes contradiction: **lacking COW, no new allocation should occur; parent pointer should remain unchanged, yet checksum altered**.
⚠️ **Must pair with negative control**: running same check post-normal COW commit must be all green, avoiding flagging normal updates as tampering.

**2. Generational incremental scrub.** Leveraging I-1.2 (Block Header Write Order Published) and D5 (Snapshot / Space Accounting Mechanism) birth txg
to maintain "generation watermark scanned in previous scrub", traversing only blocks with `birth generation > watermark`—
checksum chains of older subtrees remain self-consistent if unaltered. **Enables "verifying current change completely on every commit" within wall-clock constraints without sampling.**

**Scope**: Re-located replicas retain original `birth`, permanently falling below watermark, invisible to generational incremental scrub. Neither root chain diff (`grep -rn 根链 crates` yields 0 hits) nor generational scrub (`grep -rn scrub crates` yields 0 hits) exists in `crates/`.

**Basis**:
- No experiment: Both rely on structural properties of COW new root per transaction and birth txg; neither implemented, lacking measurable quantities.
- Three-party review: New round establishing D13 (Verification Route) (local LLM / Sonnet / Opus, non-overlapping stances), background in history.

**Owed**: C389 (Merkle root chain diff has zero code); C194 (Generational incremental scrub misses relocated replicas); C408 (Generational incremental scrub has zero code).

#### Settled Item 9: Crash-Point Replay Must Be Tiered to Avoid Wall-Clock Explosion

**Settlement**:

| Tier | Scope | Trigger |
|---|---|---|
| Smoke tier | Fixed seed, fixed script workloads and registered crash enumeration cases, unconstrained by write request count (scope below), exact full run | Checker tier full run (settled item 15): run upon user request; commit runs quick tier by default |
| Regular tier | Moderate workloads (thousands of write requests), exact full run, incomplete enumeration handled by exact pruning, unrunnable nodes marked ignore, coverage report and gate listing "not run this time" entry by entry | Every commit |
| Heavy tier | Regular tier workloads + large workloads + multi-device cross matrix, N≥5 rounds, exact full run | User request / pre-release |

**What can be claimed after pruning plus ignore**: can claim "enabled nodes passed across all crash states";
**cannot claim** "ignored nodes passed".

**Full run is not run by default at commit, unconstrained by time, run upon user request; pre-commit = quick tier all green (necessary condition).**
**At milestone closeouts or batches modifying crash-point replay or judges, main agent displays popup asking user to run full run; commits permitted on failure or skip, logging owed check for red or "milestone closeout skipped full run" until resolved.**
"Full run" refers to two items: checker-tier-crash-point-replay `--full` (gate checker-tier-crash-point-replay, settled item 15 full run) and crash amplification full run (`scripts/crash/crash-amplification.sh` without arguments).
"Modifying crash-point replay or judges" calculated dynamically from crash verdict library: triggered if changes cause mismatch in library verdict block unit keys, with mismatched keys containing judge units (function units in closure of judge entry fn); gate reports invalidated block count and changed judge unit list, main agent prompting popup accordingly, avoiding path-based lists.
"Previous full run" recorded by main agent in milestone log: date, conclusions of both runs.
**Full run tier must prioritize "failure reproducibility" over "maximal coverage"**—fixing a full run bug takes wall-clock ≈ `log2(N) × single full run duration`; an irreproducible full run failure has near-zero or negative value, training developers to ignore red flags.

**Scope**: Only smoke tier exists (gate checker-tier-crash-point-replay); smoke tier divided into two modes: regular quick tier takes arbitrary subset for in-place writes and all-or-nothing for COW writes (settled 2026-09-26, designated "variant 2", used after fixing C561 (Record Checker Reuse Exemption False Red on Partial COW)), unable to detect bugs manifesting only under partial COW landing, incapable of replacing full run; full run with resumable checkpoints (gate checker-tier-crash-point-replay `--full`), registered crash enumeration cases in `.claude/gate.d/stage-inputs.tsv` growing to 12,230,590,578 states on parallel line 1 stream, far exceeding smoke tier scale of "two-digit write requests", hence commit runs quick tier by default, full run executed upon user request per settled item 15; regular tier and heavy tier lack gate stages. Defining crash states belongs to settled item 4; exact pruning and enabling ignored nodes belong to settled items 10, 11.

**Basis**:
- No experiment: Tiering is design constraint of gate wall-clock; workloads and runtimes for regular tier and heavy tier absent, lacking measurable quantities.
- Three-party review: New round establishing D13 (Verification Route) (local LLM / Sonnet / Opus, non-overlapping stances), background in history.
- Regular quick tier uses variant 2: user decision, original wording in history; crash replay scale review `research/prompts/m2-layer0-scale-r3-main-verification.md`.
- User decision: "we do not sample, we prune plus initial ignore" (`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` Section 3); "enumeration can set ignore flag and open slowly" (`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` Section 3); "1. design complete enumeration, 2. incomplete enumeration is purpose of pruning" (`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` Section 3); Round 1 verdict `research/prompts/m3-prune-gpu-r1-main-verification.md`: settled item 9 overturned.
- User decision (2026-10-01): Full run unconstrained by time, runnable anytime upon user request, removing "nightly", original wording in history.
- User decision (2026-10-01): Timing for full run all-green is milestone closeout and pre-commit of batches modifying crash replay or judges, prompted via main agent popup, original wording in history; delegated to user per three-party verdict `research/prompts/test-script-align-r1-main-verification.md` Section 2 t1-3.
- Three-party verdict `research/prompts/test-script-align-r2-main-verification.md` Section 2 U3: two meanings of "full run", path judgment for "modifying crash replay or judges", logging "previous full run", synthesizing "admission condition" and "async" into single rule.

**Owed**: C409 (Regular tier and heavy tier lack stages in gate); C635 (Previous full run lacks machine-readable marker).

#### Settled Item 10: Truly Required Exact Coverage N Is "Number of Commit Protocol Equivalence Classes", Not "Layout Schemes"

**Settlement**: Regular tier exact coverage reduces the problem before addressing pruning. Domain of crash points is the **transaction layer state machine**, governed by two top-level rules in `.claude/rules/fs-design.md`:
"One transaction layer, shared by all structures" and "The only layer forbidden from diversifying is the transaction layer". Thus layout can alter only two categories:

| Difference | What It Changes | Judgment |
|---|---|---|
| Node size, stripe width | Bytes written per step, repetition of step | **Parameter**, introduces no new step kinds |
| Rotational vs SSD | Seek cost, IO scheduling | Resides at device layer, **outside transaction layer protocol** |
| **Zoned (ZNS)** | **Ability to overwrite root pointer in-place** | **Structural**: append-only, drifting root, introduces zone finish/reset |

**Implemented via closed enum + exhaustive match**: `CommitStep` in `crates/singlefs-core/src/transaction.rs`, step kinds registered in segment sequence registry of [layout/01-first-txn.md](../layout/01-first-txn.md) Section 8.
Any layout introducing new structural steps **must modify this shared definition ⇒ compiler enforces branches at all call sites ⇒
diff is grepped by gate**, automatically tagging "this commit introduced new structural equivalence class, requiring independent heavy tier replay",
**without relying on human memory** (diff-based gate scope owed under C8 (Gate Scope Indeterminable)). Adding newtype isolation (`RotationalPtr` / `ZonedPtr`),
turning "forbidden sharing" bucket in D12 (Target Media) settled item 2 (accounting fact judgment paths) into compile-time constraint.
⚠️ **Second duty (added by D17 (Implementation Layering and Third-Party Pipelines)): serves as integration point for third-party pipelines.**
Protocol owned by this project; external pipeline introducing new structural commit steps must submit diff to this repository.

**Scope**: Rotational and SSD are isomorphic at commit protocol level, established by D12 (Target Media) three-party review; zoned line indicates it introduces at least one additional equivalence class. Equivalence relation isomorphic to recording stream segment sequence, count belonging to D17 (Implementation Layering and Third-Party Pipelines) settled item 2. Whether kind string identifies nodes adequately hit in Round 1 verdict (suspended, awaiting Rounds 2, 3): `research/prompts/m3-prune-gpu-r1-main-verification.md`.
⚠️ **Pins "which steps exist", but cannot pin "failure atomicity width of each step"**—
persistent memory and object storage alter failure atomicity width of each step, unhandled by enum (gap and remediation in D17 (Implementation Layering and Third-Party Pipelines)).
`RotationalPtr` / `ZonedPtr` newtype isolation absent in `crates/` (`grep -rn "RotationalPtr\|ZonedPtr" crates` yields 0 hits).

**Basis**:
- No experiment: Identity of N derived from `.claude/rules/fs-design.md` top-level rules, lacking measurable quantities; criteria for equivalence relation and counts in D17 (Implementation Layering and Third-Party Pipelines) settled item 2.
- Three-party review: Regular tier sampling round ("three-party review + empirical verification"), background in history.

**Owed**: C8 (Gate scope indeterminable).

#### Settled Item 11: When Ignored Nodes Are Enabled: Rejecting Ledger Rotating by Owed Checks, Retaining Three Rules

**Settlement**:

**"Ledger rotating by owed checks" scheduling: Rejected.** Scheduling based on cells = `(layout, substage_bucket)`, taking head of queue in descending order of owed checks, claiming any cell covered within `K_max = ⌈N×M/C⌉` commits is rejected: fails to guarantee precisely the slow, rare branches it aims to protect (empirical calculations and four rejection reasons in history).

**Three retained rules**:

1. **Correct direction**: Requires **deterministic coverage records capable of turning gates red**, rather than probabilistic arguments—
   probabilistic guarantees equal no guarantee in long-term maintenance.

<!-- doc-lint:registry name-col=2 -->

| Symbol | Short Name | Meaning |
|---|---|---|
| T1 | Regular tier wall-clock budget | Local wall-clock budget for regular tier, an input rather than output |

2. **`T1` (Regular Tier Wall-Clock Budget) is input, not output** (`.claude/rules/fs-design.md`: gate runtime is itself a design constraint).
   Correct sequence is "declare budget first → constrain how many ignored nodes single commit can enable", not "measure runtime → infer how many can open".
   **This rule requires no measurement, settled immediately.**
3. **Discriminative power must be checked separately from coverage**: Commit enabling an ignored node must run its checker against a "known corrupted image",
   flagging red if no error reported—direct reuse of existing positive control discipline in D13 (Verification Route). See C13 (Checker Judgment Ineffective).

Gate lacks "regular tier crash-point replay" stage, unable to measure `T1` (Regular Tier Wall-Clock Budget) / `C` / per-cell runtime.
⚠️ **Therefore `C`, `K_max`, `T1` (Regular Tier Wall-Clock Budget) must not be written into kb at this time.**

**Scope**: Governs when ignored nodes are opened; smoke tier full run enumeration does not require `T1` (Regular Tier Wall-Clock Budget). Rule 2 "settled immediately" conflicts with ending sentence "must not be written into kb", recorded in [verification-build.md](../verification-build.md) "Pending Decisions" question 6, unadjudicated. Node registry carries status field (open / ignore / awaiting implementation), plus three rules—each ignore row names mutating object, turning red if object turns green while node remains ignored, coverage report lists ignored state count and ratio: attacked 0 rounds (`research/prompts/m3-prune-gpu-r1-main-verification.md`), unwritten as settlement pending Rounds 2, 3.

**Basis**:
- No experiment: Rejecting ledger based on simulation of scheduling rules (200 rounds, steady-state maximum of second half, strict prefix greedy, numbers in history), unnumbered as experiment; retained three rules are gate design principles, lacking measurable quantities.
- Three-party review: Regular tier sampling round ("three-party review + empirical verification"), background in history.
- User decision: "enumeration can set ignore flag and open slowly" (`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` Section 3); Round 1 verdict `research/prompts/m3-prune-gpu-r1-main-verification.md` overturned settled item 11, providing status field form and three supplementary conditions.

**Owed**: C8 (Gate scope indeterminable); C13 (Checker judgment ineffective).

#### Settled Item 12: Conflict 1: `Checker as Specification` vs `Verification Paths Sharing No Tools`

**Settlement**: **Conflict 1: Direct contradiction between `checker as specification` and `verification paths sharing no tools`.**
If active invariants (authoritative count in [invariants/README.md](../../../crates/singlefs-checker/invariants/README.md) header) serve as both checker source and formal verification spec, they become single point of failure.
**Viable compromise, paid for in this project**: Checker implements two directions—**traversal** (descending from system configuration)
and **scan** (full disk scan, reconstructing from block headers under "metadata blocks must be self-describing"), both judging the same invariant independently,
sharing only format parsing.

**Scope**: Governs sharing between the two internal directions of checker; sharing between checker and implementation governed by settled item 5. Pool-level checker implements both directions (`crates/singlefs-checker/src/walk.rs` header).

**Basis**:
- No experiment: Trade-off between "checker as spec" and verification path independence, lacking measurable quantities.
- Three-party review: New round establishing D13 (Verification Route) (local LLM / Sonnet / Opus, non-overlapping stances), background in history.

**Owed**: C410 (Checker two directions sharing lacks check): rule that two directions "share only format parsing" lacks checks turning red (no stage in `.claude/gate.d/`).

#### Settled Item 13: Conflict 2: Refinement Spec vs Differential Model

**Settlement**: **Conflict 2: Refinement spec and differential model are two expressions of the same entity.**
Merging into one → halves effort, but **directly violates independence requirements**; writing separately → nominal independence,
yet typically authored by same person in same week. Mitigation (reduces correlation without eliminating): ensure two versions **differ at representation level**—
spec phrased as "state predicates over images", model phrased as "op → state transition", facilitating mutual localization upon divergence.
**Common-origin must be observed**: Track counts of "spec and model diverge" events and which side was judged in error;
accumulating ≥ 5 divergences with 0 "spec error" verdicts indicates effective common origin, requiring merging paths and transferring budget to exhaustive testing.
Verdict has three categories: spec error / model error / clause missing sentence (each following valid interpretation where clause left ambiguity). Inconsistency re-evaluated within valid premise domain, recorded only if persisting; "clause missing sentence" recorded separately, excluded from "accumulating ≥ 5" and "spec error", re-evaluated and assigned to first two categories after clause updated.
Model here refers to O1 (In-Memory Model Differential Testing (HashMap Ideal File System)). Until O3 (Independent Spec Executor) delivers verdicts for same (history, crash point) (first version Python executor E170 (First Version of O3 Spec Executor) exists, unconnected to crash replay, C447 (Spec Executor Not Connected to Crash-Point Replay to Compare Against RefFS)), pair O3 (Independent Spec Executor) × RefFS registered as blank, logging zero events; O1 (In-Memory Model Differential Testing (HashMap Ideal File System)) crash judge does not substitute; unestablished pairs report "unestablished", never reporting green or entering counts.

**Scope**: Mitigation reduces correlation without eliminating it. Former candidate withdrawn in settled item 1 unaffected, reasons in settled item 1 scope. Crash refinement code consists of Verus core and spec in `crates/singlefs-replay-prefix/src/lib.rs` (prefix extraction, root selection, ring scan, and joint corollary, covering settled item 1 M1, M2 syntactic layer, M3 readings and monotonicity; M2 semantic layer and M4 judged within bound by E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bound)); spec cannot evaluate concrete states, provenance of divergence events undefined, counter lacks code (C191 (Common-Origin Counter Covers Only Single Pair)).

**Basis**:
- No experiment: Common origin observed via future divergence counts; spec and counter absent, lacking measurable quantities.
- Three-party review: New round establishing D13 (Verification Route) (local LLM / Sonnet / Opus, non-overlapping stances), background in history.
- Third category "clause missing sentence": three-party verdicts `research/prompts/m3-formal-r1-main-verification.md` Section 2 "F2 common origin", `research/prompts/m3-formal-r2-main-verification.md` Section 2 line 9, `research/prompts/m3-formal-r3-main-verification.md` Section 2 common-origin counter line; no experiment: counter lacks event source.
- "Model refers to O1 (In-Memory Model Differential Testing (HashMap Ideal File System))" and unestablished pairs registered as blank: three-party verdict `research/prompts/m3-formal-r5-main-verification.md` Section 2 common-origin cell; `research/prompts/m3-formal-r4-main-verification.md` user decision.

**Owed**: C191 (Common-origin counter covers only single pair).

#### Settled Item 14: Interlocks with Other Decisions

**Settlement**:

| Decision | Interlock Point |
|---|---|
| D4 (Checksum Placement) | Root chain diff flips Merkle from "necessary overhead" to verification lever; in formal verification hashes are uninterpreted functions, **collision resistance completely outside verification scope, must not be read as "verifying Merkle"** |
| D8 (Core Index Structure) | Log-structured nodes, write buffer, key cache burden symbolic execution (loop bounds, lag windows, cache coherence), rendering "Yggdrasil within one minute" measurement basis invalid in this project |
| D9 (Encryption) | AEAD intractable in SMT, modeled as uninterpreted function; **consequence "nonce reuse leaks plaintext" unprovable**, leaving pure combinatorial property "nonces never repeat"—which is precisely the most valuable output of this leg |
| D11 (Whether to Reserve Message Buffer in Index Nodes) | Settled item 1 selects "reserve"; once D8 (Core Index Structure) settled item 1 adopted idempotent full values, checker I-3.1 (Allocated Statistics Match) need not parse message semantics (D11 (Whether to Reserve Message Buffer in Index Nodes) judging Cost 1 invalid in this project), preventing Conflict 1 from becoming intractable |

**Scope**: Interlocks define boundaries of formal verification and checker when touching these decisions, leaving clauses of those decisions unaltered; root chain diff and Conflict 1 belong to settled items 8, 12.

**Basis**:
- No experiment: Interlocks are boundaries deduced from settled clauses of those decisions, lacking measurable quantities.
- Three-party review: New round establishing D13 (Verification Route) (local LLM / Sonnet / Opus, non-overlapping stances), background in history.

**Owed**: None.

#### Settled Item 15: Verification Code in Two Tiers —— Harness Tier Runs Anytime, Checker Tier Defaults to Commit

**Settlement**: Verification code under `crates/` divided into two tiers by "when it runs", each inhabiting a package, comprising three names alongside pool-level checker: **Harness tier** is `crates/singlefs-harness`, housing unit and integration tests plus scaffolding libraries (recorder, ramdisk, ideal model, random history, fault injection device, scenarios), run anytime code changes; internally split into light and heavy: debug runs taking ≥ 60s marked heavy with `#[ignore]`, measured during routine runs rather than isolated benchmarks, runnable anytime, uniformly wrapped in memory; one case per scenario, test files named by target rather than milestone. **Checker tier** is `crates/singlefs-checker-tier`, housing crash state enumeration engine, record checker, crash enumeration cases via verdict library across two phases and counter, discovery logs and toolchain identity, long-term verdict library (block keys determined by content, test types indexing needed blocks) and two-phase protocol, crash injection and bad-disk campaigns, real device log comparisons, experiment and real-device binaries, all corresponding cases, along with QEMU (gate checker-tier-qemu-device-streams), herd7 (checker-tier-lkmm), crates mutation table (checker-tier-crates-mutation-replay), and full experiment replays (checker-tier-research-build-and-replay experiment-replay cell): runs by default at commit only, runnable standalone (command carrying `SINGLEFS_HEAVY_TESTS=user-request`). Crash-point replay within checker tier (gate checker-tier-crash-point-replay) divided into two modes (others undivided): **Quick tier** consists of two passes: `cargo test --release -p singlefs-checker-tier --lib --tests` without `--ignored` (`--tests` including all `test = true` targets, including harness inline unit tests), plus pass with features touched by `cfg` in `tests/` (`--features <features> --lib --test <feature-gated-test>…`, features with `gpu` run via `scripts/crash/gpu-quiesce-run.sh`) (gate checker-tier-crash-point-replay runs both passes during full gate and commit, second pass being feature-gated-tests cell); **Full run** executes registered cases in `.claude/gate.d/stage-inputs.tsv` under key `crash-case:` entry by entry via `--include-ignored --exact` (checker-tier-crash-point-replay `--full`), skipped by default at commit, run upon user request per settled item 9; crash amplification judges migrated entirely to GPU, covering all pool-level checker invariants (`crates/singlefs-checker-tier/src/crash_judge_gpu.rs`). **Pool-level checker** is `crates/singlefs-checker`, the verdict library called by both tiers. Dependencies permitted in single direction only: checker tier depends on harness tier and pool-level checker, harness tier never depends on checker tier (gate checker-independence-and-sync cell checker-implementation-disjoint item ④, dev-dependencies included); shared snippets duplicated separately. Crash enumeration cases written strictly in `crates/singlefs-checker-tier/tests/` (judged by `scripts/crash/crash-case-check.py`); heavy test gate identified by package: tests reaching `singlefs-checker-tier` belong to checker tier.

**Scope**: Governs package placement, execution timing, gate recognition, and naming of code and tests under `crates/` (`.claude/rules/verification.md` "Definitions and Names"); does not govern what each invariant judges ([invariants/README.md](../../../crates/singlefs-checker/invariants/README.md)), enumeration domain definition (settled item 4), or sharing between pool-level checker and implementation (settled item 5: pool-level checker depends solely on `singlefs-format`). Upstream SOP no longer documents project verification methods: crash-point replay, model differential, checker-as-spec, and filesystem-specific deduction gaps consolidated into repository (`.claude/rules/verification.md`, `.claude/gate-not-implemented.tsv`), synced to replica without release. Tier splitting recorded in `records/2026-09-27-验证两档拆分.md`.

**Basis**:
- No experiment: Execution timing and package placement are process policies, lacking measurable quantities; discriminative power demonstrated by test cases and mutation tables.
- User decision (2026-09-27), original wording in history; three-party review `verification-split-r1` (`research/prompts/verification-split-r1-main-verification.md`) reviewed version prior to extracting `singlefs-checker-tier`, subsequent changes unattacked.
- User decision (2026-10-01): Full run unconstrained by time, runnable anytime upon user request, removing "nightly", original wording in history.

**Owed**: None.

#### Settled Item 16: Segments Exceeding u64 Judged in Slices, State Count Uses 1280-Bit Fixed-Width Integer

**Settlement**:
- Slicing: Segments registered by segment in crash amplification (`.claude/gate.d/crash-point-replay-segments.tsv`, five columns per row: item name, flow name, segment index, segment write count, segment write table digest) whose full domain state count exceeds u64 are judged sequentially in fixed-size slices; slice size determined by environment variable `SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES`, defaulting to 2^32 states; slice start is resumption point in verdict library (indexed by write table digest and judge unit key; item name, flow name, segment index for display only). Segments fitting in u64 judged unsliced in single pass. Code locations: `crates/singlefs-checker-tier/src/crash_amplification.rs:14800` `REGISTERED_SEGMENT_SLICE_STATES_ENVIRONMENT_VARIABLE`, `crates/singlefs-checker-tier/src/crash_amplification.rs:14947` `fn segment_slices_of_the_recording_from_the_environment_and_the_library`.
- Bit width: Total state count, slice start, slice end, and progress for registered segments uniformly use 1280-bit (10 × u128) fixed-width unsigned integer `StateCount1280Bits` (`crates/singlefs-checker-tier/src/crash.rs:1640`, limbs defined by `STATE_COUNT_LIMBS` in same file), arithmetic fully checked, non-saturating, non-truncating; driver lines `registered_segment_progress=` report judged states and total states in exact decimal, never powers.
- Passing registry to engine: Engine reads segment registry strictly via environment variable `SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY` (`crates/singlefs-checker-tier/src/crash_amplification.rs:14764` in `fn registered_segments_of_the_flow_from_the_environment`), treating unconfigured as zero segments registered without fallback. Stream extending 144-unit file to 145 units growing lower extent segment to two levels (`position_addressed_ExtentLowerSegmentGrowsToTwoLevels`) registers both unit write segment (322 unit writes) and record segment (290 in-place journal writes).

**Scope**: Governs slicing, integer representations, and registry reading in crash amplification engine; does not govern which segments should be registered (approved by user, recorded in registry header), how segment domain expands (verdict J10 of `research/prompts/m3-crash-enumeration-merge-r3-main-verification.md`), or how individual crash states are defined (settled item 4). Unregistered segments and whole-flow state sums must still fit in u64, rejecting execution before library write if exceeding; registered segments exceeding 1280 bits likewise rejected before library write. Slice size restricted to range 1 to 2^32: two GPU kernels add intra-slice offset as u32 to slice start (`crates/singlefs-checker-tier/src/crash_amplification.rs:14807` `LARGEST_REGISTERED_SEGMENT_SLICE_STATES`); light tier callers (`scripts/crash/crash-point-replay.sh --quick`) default slice size to `LIGHT_REGISTERED_SEGMENT_SLICE_STATES`. Segments exceeding expansion limit without registration fail amplification entry `scripts/crash/crash-amplification.sh`; crash-point replay, amplification, and item drivers verify table before passing to engine.

**Basis**:
- E161 (Deduplication and Segmentation Elapsed Time in Crash Amplification) "Rerun 2: Sampling Domain Reset to 11-Segment Stream (2026-10-05)": Full state counts match closed form, harness values, and crates plan segment by segment, growing exponentially with write counts, supporting premise that "long segment full domain exceeds u64"; slice size, bit width, and registry reading are engineering parameters settled by user, unsupported by E161.
- User decision, original wording in history: slicing and 2^32 states per slice (`records/2026-10-11-崩溃放量两段与脚本审计.md` line 96), 1280-bit width (lines 98, 99), registering both segments (line 92).
- Slicing and width supplement verdict J10 of `research/prompts/m3-crash-enumeration-merge-r3-main-verification.md`, unattacked by three-party review.

**Owed**: None.

#### Settled Item 17: Unit of Verdict Reuse Is Function, Key Composed Solely of Function Units, Toolchain, Dependency Versions

**Settlement**:
- Verdict reuse across heavy gates, crash verdict library, and experiment admission (crash state blocks, quick tier and model differential test functions, crates mutation lines, broad mutations, formal verification, QEMU tiers, litmus suites, research reruns, experiment mutation tables, experiment admission) depends strictly on its unit key: de-commented fingerprints of dependent function units, plus `#[cfg]`, `#[cfg_attr]`, and `cfg!` atoms genuinely referenced within closures (including mod headers) paired with boolean values under current target (unreferenced atoms excluded: builds with/without `gpu` feature share identical key and library if `gpu` unreferenced in closure), toolchain identity (including cargo build environment and external tool versions), dependency versions resolved from `Cargo.lock` genuinely referenced by closure, and content inputs of the verdict itself (segment write tables, base image digests, litmus content, mutation originals and replacements). Function units collected from entry point along call and reference paths, including referenced constants, types, statics, and macro expansions; WGSL segmented by fn.
- Strictly excluded from keys and reuse criteria: diff ranges computed by commit, HEAD, or tree; entire file text; file-level, crate-level, or table-level fingerprints/closures; manual version constants; paths (including repository root), names, seed strings, line numbers, resource caps, timeouts; whole-stage or whole-cell flags lacking per-verdict libraries. Keys too narrow (missing referenced constants, types, macros), placing libraries in ephemeral paths, losing libraries on rename, or duplicate judging across two libraries constitute violations.
- Incomplete collections (dynamic dispatch, macros, unparsed `include_str!(concat!…)`) reject key computation and run normally, never silently omitting dependencies; each key persists alongside its component unit list, reporting which units changed upon invalidation.
- One library per verdict class, fixed in persistent directory, never changing directory across runs or evaluation methods; repository root is not input: crash verdict library directory accepts absolute paths only via `SINGLEFS_CRASH_AMPLIFICATION_LIBRARY`, rejecting relative paths; full run switch writes without reading, updating newly judged results into same library; second host results merged into local copy.
- Invalidating old libraries upon key changes incurs no migration cost: no compatibility migration or backfilled signatures, rerunning after changes.

**Scope**: Governs reuse keys for heavy gates, crash verdict library, and experiment admission; does not govern diff ranges for static gates checking added lines only (`scripts/gate/changed-paths.sh`) or preflight `admission: inputs-changed`; does not govern enumeration domain (settled item 4), tiering, or user prompts for full runs (settled item 9). Image verdicts and given-state verdicts keyed by verdict definition group (closures of CPU reference judges `crash::judge_crash_image`, `crash_given_states::judge_the_given_states_on_the_cpu`), regardless of CPU or GPU evaluation: GPU judge is accelerated path, admitted via differential testing against definition group (quick tier GPU vs CPU differential cases); its units enter block attributes as provenance rather than keys, undivided by kind; GPU card drivers, card models, and multi-card partitioning remain block attributes or run parameters, excluded from keys.

**Basis**:
- User decision (2026-10-15), original wording in history and `records/2026-10-13-复用键与全仓改名.md` Section "2026-10-15 User Settles: Commits Unrelated to Reruns, Reruns Follow Code Changes Only", "2026-10-15 Batch 3".
- User decision (2026-10-17): Verdicts unbound to hardware, CPU vs GPU; reuse and pruning ignore devices; original wording in history and `records/2026-10-13-复用键与全仓改名.md` 2026-10-17 halting full run.
- Main agent decision (2026-10-15): Repository root excluded from keys and libraries, library path accepts absolute only, cfg components include genuinely referenced atoms only, recorded in same document decisions 1, 2.
- Counter-evidence: E164 (Small-Scope Functional Testing of New Crash Amplification Design) F4 demonstrated fingerprinting functions alone without referenced constants missed V5 and V11 (V11 modifying constant `DATA_POINTER_BYTES` only): function units must include referenced constants, types, statics, and macros.
- On-disk measurements: `research/prompts/pruning-survey-2026-10-15/ledger.md` Section 1.2—blocks differing only by name stored and judged separately across 4100 and 9679 blocks; altering single whole-library method digest rejudged 3,935,242,934 states, where 24 of 29 changed files in closure were verbatim identical after de-commenting: keys with names or file-level fingerprints trigger spurious re-judgments.
- Unattacked by three-party review; user settled opening three review rounds post-code changes, attacking strictly "whether pruning is atomic, whether keys comprise function units only".

**Owed**: C8 (Gate scope indeterminable), C489 (Broad mutation of singlefs-core lacks conclusion for current code), C528 (Two gate paths judge different change windows), C564 (Remaining 6 crash-point replay full runs unrun), C605 (Crash amplification reuse and pruning lack disabled control), C630 (Commit-time formal verification step 1 halt frequency unmeasured), C635 (Previous full run lacks machine-readable marker), C645 (Crash amplification full run lacks executor), C668 (Random history harness importer whitelist bypassable via alias), C777 (Five experiment keys lack registered lines in stage-inputs.tsv), C836 (Tree attached to staged-green not necessarily judged tree).

```

**出处 `.claude/kb/decisions/13-verification-route.en.md:444-447`（整段抄，未转述）**

```markdown
## Historical Versions

Historical items for D13 (Verification Route) are consolidated in [decisions-history.md](../decisions-history.md).

```

**出处 `.claude/kb/decisions/13-検証ルート.ja.md:1-6`（整段抄，未转述）**

```markdown
## D13 検証ルート —— 確定

[中文](13-验证路线.md) · [English](13-verification-route.en.md) · **日本語**

D13 (検証ルート) は本プロジェクトが正当性を何によって検証するかを司る：3種の既存手法をすべて実施し —— 独自参照実装の作成による差分突き合わせ、有界全数探索クラッシュテスト、crash refinement 形式検証（調査は [prior-art.md](../prior-art.md) 7.4 参照） —— その上で単なる模倣に留まらず、本プロジェクト固有の4点（トランザクションごとの新ルート発行を伴う COW、Merkle 自己検証、ボリューム全体の AEAD における nonce の一意性、参照実装の欠如）に応じた独自の検証手段を設計する。oracle をどう分割するか、独自開発する2点は何か、クラッシュポイント再生をどのように階層化およびサンプリングするか、クラッシュ状態集合をどう定義するか、checker と実装が何を共有するかを決定する。管轄外：各不変条件の規定内容は [invariants/README.md](../../../crates/singlefs-checker/invariants/README.md) に委ねる；3種の検証手段の具現化手順と依存関係は [verification-build.md](../verification-build.md) に委ねる；コミット手順の登録場所、構造同値類の分割方針とクラス数は D17 (実装階層とサードパーティパイプライン) 確定項目 2 に帰属する。

```

**出处 `.claude/kb/decisions/13-検証ルート.ja.md:7-444`（整段抄，未转述）**

```markdown
### 確定項目

| # | 項目 | 確定事項 |
|---|---|---|
| 1 | **crash refinement はどの部分を先に検証するか** | まず「1回のパブリッシュ + 1回のクラッシュ + 1回のリカバリ」という閉じたユニットを検証する。第1マイルストーンは任意の `jsn` 連続プレフィックスがパブリッシュを分断しないこと；マウント透明性は第4の独立した性質として確立；旧候補 I-6.5 (nonce 水位の単調性) は撤回 **状態：確定。** **進捗：実装済み。** |
| 2 | **トランザクション層が直列コミットを公約するか** | まず直列とし、交差点を越えた後に並行を開く；将来の並行コミット時はレコードをトランザクション番号順に追加する **状態：確定。** **進捗：実装済み。** |
| 3 | **独自参照実装（RefFS 形態）をどの程度まで突き詰めるか** | セマンティクスレベルを極限まで突き詰める：メモリ内でのみ POSIX 操作に従って動作し、永続化およびクラッシュ後に許容される状態を独自に定義し、ディスク上フォーマットは持たない；kb に基づいてゼロから独立パッケージとして作成し、ホワイトリストの入力のみを受理する **状態：確定。** **進捗：実装済み。** |
| 4 | **クラッシュポイント再生のクラッシュ状態集合をどう定義するか** | 書き込みストリームをデバイス単位でセグメントに分割し、セグメント内で書き込みのあった各ディスクが自らのバリアまたは FUA で解放されて初めてセグメントを閉じる；セグメント内の各完全書き込みは状態を任意に組み合わせ、インプレース上書きには「新旧いずれも読み出せない」という第3の状態を追加；未永続化部分には旧バイトを配置し実際にイメージを生成する；ミラー書き込みはデバイスごとに個別に計算する **状態：確定。** **進捗：実装済み。** |
| 5 | **checker とテスト対象実装の間で共有を許容するもの** | kb から生成された定数モジュール1つのみを共有し、それ以外は各々独自に実装する；判定幅は checker が独自に検出しシステム構成と照合し、検出不能時は「宣言値、未検出」と報告する **状態：確定。** **進捗：実装済み。** |
| 6 | **互いに独立な3つの oracle** | インメモリモデル突き合わせ、独立パーサ + checker、独立規約実行器の3つの oracle が各々異なる側面を判定する；独立性は「3 oracle 結託テスト」によって検証し、自己申告を禁ずる **状態：確定。** **進捗：実装済み。** |
| 7 | **O2 (独立パーサ + checker) は何を判定するか** | O2 (独立パーサ + checker) の定義域は単一イメージであり、集合の要素性のみを判定する；第2の入力を必要とする検査はレコード照合器（入力パラメータに列挙器が提供する永続化集合を含む）に帰属させ、意図的に番号を付与しない **状態：確定。** **進捗：実装済み。** |
| 8 | **独自開発の2点** | Merkle ルートチェーン差分（隣接する2世代のルートを再帰的に比較し、ポインタ不変箇所には掘り下げない）および世代間増分 scrub（birth generation が水位より高いブロックのみ走査） **状態：確定。** **進捗：未実装。** |
| 9 | **クラッシュポイント再生をどう階層化するか** | 3層とも正確なフル実行とし、列挙しきれない場合は正確な枝刈りに依拠し、直ちに実行できないノードには ignore を付与して順次開放する；マイルストーン完了および判定器ユニット変更を伴うコミット前にはポップアップでユーザーにフル実行層の実行を促し、失敗または未実行でもそのままコミット可能とし未済として記録；まず失敗再現性を確保する **状態：確定。** **進捗：実装済み。** |
| 10 | **真に正確に網羅すべき N は何か** | コミットプロトコル構造の同値類数であり、レイアウトのセット数ではない；具現化手段は閉じたコミット手順列挙と網羅的な match である **状態：確定。** **進捗：実装済み。** |
| 11 | **ignore ノードをいつ開放するか** | 未済の検査のローテーション台帳を否決；開放の契機を規定する3条を残す：決定論的なカバレッジ記録、`T1` (通常層 実時間予算) を入力とする、判別力とカバレッジを分離して検査する **状態：確定。** **進捗：対象なし。** |
| 12 | **衝突 1：checker 即仕様 と 検証の2経路間でツール共有を禁止する** | checker はトラバーサルとスキャンの2方向を実装し、同一の不変条件に対して各々独立に判定し、共有するのはフォーマット解析のみとする **状態：確定。** **進捗：実装済み。** |
| 13 | **衝突 2：refinement の spec と突き合わせの model** | spec と model は表現形式のレベルから異なって記述する；同源性を観測し、不一致が5回以上かつ spec の誤りが0回の場合は統合して予算を全数探索へ移管する **状態：確定。** **進捗：対象なし。** |
| 14 | **他の決定との連鎖** | ハッシュと AEAD は形式検証において uninterpreted function としてのみ扱える；ログ構造の3層がシンボリック実行を拘束する；checker はメッセージセマンティクスを解析する必要はない **状態：確定。** **進捗：対象なし。** |
| 15 | **検証コードを2層に分割** | harness 層（`singlefs-harness`）は単体および結合テストであり常時実行；checker 層（`singlefs-checker-tier`）は既定でコミット時にクイック層のみ実行し、フル実行はユーザー要求時に実行する **状態：確定。** **進捗：実装済み。** |
| 16 | **セグメント単位で登録されたセグメントの分割方法と状態数のビット幅** | 登録セグメントの全領域が u64 に収まらない場合はスライスごとに順次判定し、スライスサイズは既定で 2^32 状態、スライス起点はライブラリ内の再開点とする；状態数は一律に 1280 ビット固定幅整数を用い、演算はすべて checked とする；登録表は環境変数経由でのみ読み込む **状態：確定。** **進捗：未実装。** |
| 17 | **判定再利用のユニットは何か** | 枝刈りのユニットは関数である：ある判定を踏襲するか否かは、それが依存する関数ユニットの各コメント除去後指紋、ツールチェーン、解決された依存バージョンのみを参照する；コミットやツリーによる変更計算、ファイル全体の原文、ファイル単位やパッケージ単位の指紋、手書きバージョン番号、パス名のキーへの導入、工程全体のマークは一律に禁止する **状態：確定。** **進捗：未実装。** |

#### 確定項目 1：crash refinement はまず「1回のパブリッシュ + 1回のクラッシュ + 1回のリカバリ」を検証する

**確定事項**：crash refinement の最初の検証対象はリカバリ関数 `Recover` の関係的性質であり、最小の閉じたユニットは「1回のパブリッシュ + 1回のクラッシュ + 1回のリカバリ」である。4つのマイルストーンの主語 `Recover` は、`crates/singlefs-core` 内で `verify_named_units = true` を指定して `replay_journal` を呼び出している3箇所（`recover` の `Consult` 戦略、書き込み可能マウントの読み出しフェーズ、読み取り専用マウント）を指し、3箇所の `records` はすべて `scan_journal` の出力を取得する；`Ignore` および `ConsultWithoutNamedVerification` はテストおよび対照用のアームであり、トラステッドコンピューティングベースに配置する；`crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs` はプロダクションの呼び出し元ではない。4つのマイルストーンは順に以下の通りである：

| マイルストーン | 義務 | 配置先 |
|---|---|---|
| **M1 全域性** | `Recover` が D13 (検証ルート) 確定項目 4 で列挙された**すべての**クラッシュ状態において定義され、停止し、I-7.1 (ルートスロット有効集合の非空性) および I-8.3 (再生プレフィックスの厳密な連続性) を満たすイメージを出力すること | スキャフォールド。**完了の判定基準は「証明されたこと」ではなく「M2 をその上に記述できること」である** —— クラッシュポイント再生は元よりフル実行で列挙しており（[verification-build.md](../verification-build.md)「クラッシュポイント再生はサンプリングせず、フル実行で列挙」）、リカバリのハングはその場で露見するため、全域性単独を証明することに限界価値はない。プレフィックス取得、ルート選択、リングスキャンの3区間はすでに Verus に導入され、すべて `crates/singlefs-replay-prefix/src/lib.rs` にある：プレフィックス取得コア `replay_core`（停止性と非オーバーフローはコア内で証明済み、ループはすべて `decreases` を伴う；2つの `requires` —— レコード表が (インスタンス番号, カウンタ) 昇順、カウンタに後続が存在 —— はグルーコード `replay_journal` 内のアサーションで担保）、ルート選択コア `choose_root_slot`（自己検証済みのスロットのうち (`checkpoint_txg`, インスタンス番号) 最大のものを取得、同値時はスロット順最前を取得）、リングスキャンコア `first_copy_of_each_record_key`（同一 (インスタンス番号, カウンタ) は先着順）、ルート選択とプレフィックス取得の結合判定は系 `corollary_recovery_replays_from_the_newest_self_verified_root` として記述；ゲート checker-tier-formal-verification が証明を判定し、固定数は 56 verified（`.claude/gate.d/verus-pins.tsv`）。E165 (リカバリ再生の有界全数探索とミューテーション殺傷限界) を軽量ツールとして用い、その第3回再実行第2区間のルート選択結合判定ファミリが ramdisk 上で本物の `choose_root`、`scan_journal`、`replay_journal` を呼び出す。ルート選択順序の4つの改悪変種のうち、正当な履歴 K ファミリで赤となるのは「最初の有効なものを選択」のみであり、他の3種（比較時にインスタンス番号を無視、txg 同値時に低位を選択、インスタンス番号主導）は Verus 必赤世界（`crates/singlefs-replay-prefix/verus-must-red-worlds.tsv` のルート選択の節）および敵対的ルートリング KA でカバーされる（第3回再実行第3区間）。コア外：システム構成選択（`choose_system_configuration`）および2つのグルーコード（ルート選択のスロット読み出し、自己検証チェックサム、fsid、領域からディスクへのマッピング、スロットの走査順序；リングスキャンのオフセット列挙、ヒント、パースと CRC、ディスクの走査順序）は Verus の対象外であり、クラッシュポイント再生およびクラッシュ増幅の CPU 判定器が各クラッシュ状態上で `recover` を呼び出してこれらをカバーする（GPU 判定器は WGSL で `replay_journal` を別途記述しており、core は検証できない） |
| **M2 プレフィックスがパブリッシュを分断しない** | **任意の `jsn` 連続プレフィックスについて、その再生結果はある整数個のパブリッシュ済み checkpoint の結果と等しくなる —— プレフィックスによって分断されるパブリッシュは存在しない；プレフィックスがあるパブリッシュの途中で停止した場合、そのパブリッシュは全体として適用されない**（D16 (パブリッシュセマンティクス) 確定項目 4「1回のパブリッシュは全体として適用されるか全体として適用されないかのいずれかである」）。2層に分かれる。**構文層**（任意の自己検証済みレコード表に対して、前提なし）：適用されるレコードはいくつかの完全なパブリッシュであり、インフライト上限とコミットマーカーが1条ずつ判定され、適用されるカウンタはチェーン先頭から順に連続する（アンカーが読み出せる場合はアンカー + 1 から開始；読み出せない場合、先頭の `checkpoint_txg` = 選択されたルート + 1、シーケンス番号 1）；逆方向チェーンおよび指名ユニットはディスク読み出しコールバックの結果であり、「適用された各レコードについて、コールバックが真を返した」と記述される。**セマンティクス層**（前提：ディスク上の各レコードはこのバージョンの正当な書き込み側に由来する、すなわち E165 (リカバリ再生の有界全数探索とミューテーション殺傷限界) の正当な履歴ファミリに属する；同一の (インスタンス番号, カウンタ) はすべてのタイムラインを通じて唯一の内容に対応する）：「結果」はルートの射影 + ディスクであり、ルート全体ではない —— 返却されるルートの (インスタンス番号, `checkpoint_txg`) および新ルートセグメントの4フィールドは、選択されたルートのタイムライン上の第 j 版（j は選択されたルートより前ではない）と等しく、その他のフィールドは選択されたルートと等しい（アンマウント記号を除く；レコードから再構築されたルートは「アンマウントシーケンスで書き込まれたものではない」とする）；ディスク上の (選択されたルート, j] の各バージョンのレコードが読み出せ、指名ユニットの両方のコピーが検証済みであり、レコード件数の合計がインフライト上限を超えない | 配置先は I-8.3 (再生プレフィックスの厳密な連続性) —— その判定補足文は逐語的に本ルールそのもの（「プレフィックスがパブリッシュの途中で停止してはならない —— 1回のパブリッシュは全体として適用されるか全体として適用されないかのいずれかである」）であり、[invariants/03-journal.md](../../../crates/singlefs-checker/invariants/03-journal.md) 内に別途新設してはならない。構文層は Verus コア `replay_core` の `ensures`（`replay_follows_the_prefix_rules`）内にあり、必赤世界は `crates/singlefs-replay-prefix/verus-must-red-worlds.tsv` に登録され、そのうち「判定変更」行は実行可能判定のみを変更し spec には触れない；セマンティクス層は E165 (リカバリ再生の有界全数探索とミューテーション殺傷限界) の正当な履歴ファミリによって有界内で判定され、Verus では証明しない。コア外：グルーコード（テーブル展開、コールバックのラッピング、レポートと新ルートの組み立て）は E165 (リカバリ再生の有界全数探索とミューテーション殺傷限界) が直接本物の `replay_journal` を呼び出すこと、harness テストケース、および `crates/mutations.tsv` のミューテーション行でカバーする。新ルート組み立て時の射影外フィールド（アンマウント記号、インスタンス表ポインタ、割り当てレコードツリーのルート）にはミューテーション行がなく、E165 (リカバリ再生の有界全数探索とミューテーション殺傷限界) のジェネレータがこれらを固定値として記述する；2つのディスク読み出しコールバック（逆方向チェーン、指名ユニットクロージャ）は、E165 (リカバリ再生の有界全数探索とミューテーション殺傷限界) 内の実コールバック、ramdisk の読み出し回数、`crates/mutations.tsv` のミューテーション行、および読み取り専用マウントの幾つかのパターンの読み出し回数テストケースでカバーする |
| **M3 冪等性と単調性** | **冪等性**：`Recover(Recover(c)) = Recover(c)`、両方の解釈を証明する —— レコードレベル（同一レコードを2回適用した結果と1回適用した結果で得られるルートが同一であること、I-8.4 (再生の冪等性) の文面）および不動点（再構築されたルートを選択されたルートとし、同一のレコード表、同一のリーダー、同一のスイッチで再度再生した際、返却されるルートが不変であること；前提として選択されたルート以上のレコードがインフライト上限を超えず、かつレコード表が I-8.12 (同一インスタンスレコードの txg 非減少) を満たす：同一インスタンス内の journal レコードの txg はカウンタに伴って減少しない）；2回目で選択されたルートの `checkpoint_txg` が上限に達して `CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR` を報告し、1件も適用されなかった場合は同等とみなす。「すなわち I-8.4 (再生の冪等性) である」は成立しない：I-8.4 (再生の冪等性) はレコードレベルの1解釈であり、不動点は別の独立した性質である。**単調性**（M2 の系、個別には証明しない）：直列コミット（確定項目 2）、D16 (パブリッシュセマンティクス) 確定項目 7 の永続化順序、選択されたルートがルートリング内で最新の有効なルートであるという条件下で、同一実行内の D13 (検証ルート) 確定項目 4 の故障を含まない2つのクラッシュ状態 `c ⊆ c'` について、`Recover(c)` が到達するバージョンは `Recover(c')` が到達するバージョンより遅くならず、バージョンは (`checkpoint_txg`, インスタンス番号) の順で比較され、D22 (ユニットの原子性をどう合成するか) 確定項目 7 の最新選択順序と同一順序である；並行コミットを導入する場合、またはドメインを故障を含む状態に緩和する場合は再審議する。「適用されたトランザクション集合が包含される」という解釈は撤回：理想的なリカバリに対しても偽となるためである | Verus コア `replay_core` の ensures で両方の解釈を証明：レコードレベルおよび不動点（不動点の前提条件には I-8.12 (同一インスタンスレコードの txg 非減少) を含む）；単調性は系 `corollary_the_landed_version_does_not_go_back` として記述し、ルート選択と書き込み側の2つの前提はその requires に記述する。E165 (リカバリ再生の有界全数探索とミューテーション殺傷限界) の敵対的ファミリが有界内で不動点を判定し、違反はすべて I-8.12 (同一インスタンスレコードの txg 非減少) が成立しないレコード表においてのみ発生する（第3回再実行第1区間）。checker 側（I-8.4 (再生の冪等性)）は実装済みであり、クラッシュ注入の二次クラッシュ上で判定され、プールレベル checker のリストには含めない：`crates/singlefs-checker-tier/src/crash_injection.rs` の `REPLAY_IDEMPOTENCE_INVARIANT` および `judge_the_remount_after_a_second_crash`。O2 (独立パーサ + checker) は冪等性を判定できない：I-4.5 (廃止済み・2イメージ必要) の廃止理由は逐語的に「冪等性は2つのイメージを比較する必要があり、単一イメージでは判定できない」である |
| **M4 不足適用の防止** | 前提と対象は M2 のセマンティクス層と同一。`Recover` が返却するルートはまさに j\* である：選択されたルート以降の各バージョンがディスク上に完全に存在し（レコードのいずれかのコピーが自己検証済み、指名ユニットの両コピーが検証済み）、かつレコード件数の合計がインフライト上限を超えない最も遠いバージョンである。この条項を付加しない場合、M1–M3 をすべて証明したとしても「確認済みの完全なパブリッシュがリカバリによって破棄される」ことを排除できない：1件も適用しないリカバリであっても M1–M3 の文面をすべて通過してしまう。「不足適用の防止」は「追加の違反の発生」として解釈する：変種が追加の違反を生じさせて初めて、本条項によって阻止されたとみなす | E165 (リカバリ再生の有界全数探索とミューテーション殺傷限界) の最遠バージョン判定基準によって有界内で判定する；Verus の必赤世界において「水位を超える最初のレコードで停止し、1件も適用しない」行が赤となる；M4 自体は単独では証明しない |

**マウント透明性**（M3 の第3の解釈ではなく、独立した第4の性質）：クラッシュイメージ上で読み取り専用リカバリを1回実行して R1 を得る；同一のイメージ上でアクションを1つ実行 —— マウントのみ（番号取得、行書き込み、ウォームアップ、マウント継続実行、およびマウント自体の押し上げによる F 引き上げ）、またはマウント + 1回のパブリッシュ —— その後、再度読み取り専用リカバリを実行して R2 を得て、R1 と R2 をフィールドごとに比較する。比較対象のバージョン：マウントのみの場合はリカバリが到達したバージョンと比較し、マウント + 1回のパブリッシュの場合はマウント後のパブリッシュの直前のバージョンと比較する。マウント途中の3フェーズにおける二次クラッシュイメージ img2 上でも判定する：C(recover(img2)) = C(v1)（v1 は R1 が到達したバージョン、C は POSIX の内容および存在性を取得）、内容とフィールドの両半面を判定する；フィールド半面において、v1 のバージョンにフォールバックした場合は各フィールドが R1 と同一であり、今回のマウントによって書き込まれたいずれかのバージョンに到達した場合はフィールド別表に従って判定し、スキャンレポートは判定せず、F 引き上げの最初の書き込み以降は F を判定せず、それ以前（ウォームアップを含む）は「マウントが F 引き上げを押し上げなかった」セルに従って判定する；F 引き上げの最初の書き込み位置は、マウントが返却した F 引き上げシーケンス（受入で押し上げられたものおよび継続実行で空間のために押し上げられたものの双方）のマウント書き込み表末尾における位置に基づいて取得し、取得不能時（`NotLocatedInTheMountWrites`）は今回のマウントの二次クラッシュ区間全体の F を未判定とする。フィールド別表（R2 対 R1）：

| フィールド | 判定法 |
|---|---|
| POSIX 内容および存在性 | 一致必須 |
| プール識別情報およびシステム構成の不変構成（自ディスクデバイス番号を除く） | 一致必須；不変階層の37行は自ディスクデバイス番号を除き一律に生バイトで比較 |
| システム構成のノードサイズ | 一致必須 |
| システム構成の実行時構成 36 バイト（T_time、T_dirty、3水位） | R2 は組み込み既定値に等しい（マウントオプションチャネルがなく、`to_slot` は常に既定値を書き込む） |
| アンマウント記号 | R2 は常に「アンマウントによる書き込みではない」であり、R1 の記号は比較対象外 |
| F および F 有効値 | マウントが F 引き上げを押し上げなかった場合：R2 の F 有効値および R2 ルート内の F はいずれも R1 の F 有効値に等しい；押し上げた場合（受入によるものと継続実行で空間のために押し上げたものの双方を含む）：R2 の F 有効値はマウントが返却した最後のシーケンスが引き上げた F に等しい |
| インスタンス番号 | max(各ディスクの2スロットの自己検証済みシステム構成のインスタンス番号, ルートリングの全可読ルートのインスタンス番号) + 1、双方の量をクラッシュイメージから読み出す |
| インスタンス表の行 | D18 (ブロックに含める情報) 確定項目 11 に従いリカバリが書き込んだ行 |
| `checkpoint_txg` | 下限のみ判定：≥ max(ルートリングの全ルートの txg, リング内の全自己検証済みレコードの `checkpoint_txg`) + 1（D23 (journal の役割とフォーマット) 確定項目 14 の下限、番号飛びを許容） |
| ルートスロット位置 | 個別には判定せず、`checkpoint_txg` によって決定 |
| ツリー ID 水位 | マウントが最初のファイルを発行しない場合は同一 |
| ツリーテーブル内の inode ツリー、extent ツリー、墓碑、系譜、deleted_inodes のルートポインタ（後者3つはツリーテーブルエントリであり種別 9, 10, 11、ルートレコードのフィールドではない） | 同一；マウントが継続実行された場合は、継続実行後のバージョンに従って判定：マウントが返却した継続実行の各ステップのうち txg が R2 のバージョン以下の最新ステップの各ルートポインタに等しい；ツリーテーブルユニット自体の配置先は比較しない |
| ツリーテーブル種別 6, 7, 8, 12 のルート、誕生 txg、ヘッド ID、ツリー ID、種別 | 一致必須（種別 6, 7, 8, 12 のルートは常にオールゼロ）；誕生 txg のルート間比較は別途 I-9.14 (ツリーテーブルエントリの誕生 txg のルート間不変)、ヘッド ID は別途 I-9.27 (ツリーテーブルエントリのヘッド ID の値) が単一イメージ上で判定 |
| インスタンス表およびツリーテーブルのユニットポインタ | バイト比較は行わず、「ツリーテーブルユニット自体の配置先を比較しない」と同様とする |
| スキャンレポート | 適用 0 件、水位超過 0 件 |
| ルートスロットオフセット 428..457（アルゴリズム、nonce、MAC） | 0 であることが必須；単一イメージの性質であり、プールレベル checker の I-2.4 (ヘッダチェックサム対象範囲) が判定し、マウント透明性では比較しない |
| magic、flags の残りビット、自己検証チェックサム、スロット全体チェックサム | 比較しない（構造的性質であり、スロット読み出し時に判定） |
| システム構成スロット世代番号 | 比較せず、I-7.16 (システム構成スロット世代番号とスロット番号の関係) に委ねて R2 のイメージ上で判定 |
| システム構成の journal_tail | R1 ≤ R2 ≤ R2 リング内の自己検証済みレコードの最大カウンタ、マウント完了時は等号成立 |
| システム構成の journal インスタンス番号 | マウント完了時は R2 ルートのインスタンス番号と等しい；二次クラッシュ時は R1 のインスタンス番号または新インスタンス番号のいずれかを取る |
| 割り当てレコード（ルートレコードの `mapping_root`、`allocation_record_tree_root` およびツリーテーブル種別 3, 4 のルートを含む） | テーブル全体は比較しない：POSIX 内容が同一であり、かつプールレベル checker が R2 のイメージに対して緑と判定すること；`mapping_root`：R2 オールゼロ ⇔ R1 オールゼロ；`allocation_record_tree_root`：R1 がツリーを保持 ⇒ R2 オールゼロ、R1 のツリーテーブルが 0 件 ⇒（R2 非ゼロ ⇔ 新インスタンス番号 ≥ 2） |

システム構成の実行時変数（スロット世代番号、journal_tail、journal インスタンス番号）が v1 にフォールバックした場合、それら自身の行の規則に従って判定し、v1 と同一であることとしては判定しない。

**射程**：refinement の限界価値は、クラッシュポイント再生の列挙では網羅できない幾つかの次元 —— ペイロードバイト、レコード数 / セグメント数 / txg 数（リングラップアラウンドを含む）、直前のタイムラインの残留物、スロット数とデバイス数 —— にのみ存在する；クラッシュポイントの次元自体はクラッシュポイント再生によりすでにフル実行で列挙されている。「チェックサムを通過した」ことはモデル内ではブールフラグに過ぎず、**refinement が完全性を検証したと解釈してはならない** —— 確定項目 14 における D4 (チェックサムの位置) の「『Merkle を検証した』と解釈してはならない」と同形であり、D13 (検証ルート) 確定項目 4 の CRC32C 衝突確率に対する「認識した上での受容」と同一の判断である。D13 (検証ルート) 確定項目 4 は refinement の左側の `Crash`（モデル層の列挙ドメイン）であり、右側の `Spec` ではない；これを spec とみなすと refinement は自明に真となってしまう。I-6.5 (nonce 水位の単調性) の形式化は撤回された；D18 (ブロックに含める情報) 確定項目 14 が確定し、かつ nonce 割り当て規則（単調カウンタ + 前借り水位、水位の使用前永続化）が独立項目として確立された後にこのセルを再開する —— 再開時、その唯一の真の利点は nonce が POSIX から不可視である点であり、その spec は O1 (インメモリモデル突き合わせ (HashMap 理想ファイルシステム)) の model と構造的に同源になり得ず、衝突 2 (D13 (検証ルート) 確定項目 13) の影響を受けない。

M1–M4 の前提とドメイン：セマンティクス層は「同一の (インスタンス番号, カウンタ) はすべてのタイムラインを通じて唯一の内容に対応する」ことのみに依存し、D23 (journal の役割とフォーマット) 確定項目 14「このバージョンの失敗処理」の同一再送によって担保される。ルートスロットの書き込み失敗により `checkpoint_txg` を進めて再送するパスが実装された際に再検証する；前提ドメインは E165 (リカバリ再生の有界全数探索とミューテーション殺傷限界) の正当な履歴ファミリ（正当な書き込み側が書き出した履歴の任意のディスク永続化部分集合であり、新しいルートスロットは読み出せない場合がある）を取る。M2 の「その他のフィールドは選択されたルートと等しい」において、レコードから再構築されたルートのアンマウント記号は `NotWrittenByTheUnmountSequence`（D22 (ユニットの原子性をどう合成するか) 確定項目 7「他のルートは 0 を書き込む」）であり、選択されたルートの記号と異なる可能性があるため、「アンマウント記号を除く」と明記している。M3 の単調性の系は、直列コミットかつ故障を含まないクラッシュ状態においてのみ成立する。M4 はルート選択と連動して初めて「確認済みの」完全なパブリッシュがリカバリによって破棄されることを排除する。前提条件が揃っている必要がある：故障を含まないクラッシュ状態（確定項目 4 の列挙ドメイン）、D22 (ユニットの原子性をどう合成するか) 確定項目 7 のルート選択に従って選択されたルート、インスタンス境界を跨がないチェーン（D23 (journal の役割とフォーマット) 確定項目 14 注 1）、新インスタンスにおいて自インスタンスで書き込まれたルートが2枚のディスクをカバーするまでいかなる fsync も復帰させないこと（D16 (パブリッシュセマンティクス) 確定項目 8 のウォームアップ）；故障を含むドメインにおいては排除されない —— インフライト上限によりレコードが完全なパブリッシュが切り捨てられる場合があり、ディスク1台が丸ごと喪失した際に指名ユニットが1部欠損したバージョンは適用されない（C519 (ディスク丸ごと喪失時に直前に確認されたバージョンがリカバリで破棄される)）。

マウント透明性の射程：故障を含まず、書き込み可能マウントが成功したクラッシュ状態のみを管轄する。書き込み可能マウントが拒否された状態については透明性を判定せず、拒絶自体が正当であるか否かを判定する：拒絶が正当 ⇔ 拒絶理由が D23 (journal の役割とフォーマット) 確定項目 14 のリストに含まれ、かつその条項の判定基準に基づいてクラッシュイメージ上で自前で再計算して成立し、かつディスク上の生バイトが不変であり、かつ読み取り専用読み出しが許容集合内にあること；リストにない拒絶理由についてはクラッシュ注入第2区間の記述に従って赤を報告する。故障を含む状態は射程外である：番号取得の瞬間に1スロットが読み出せないことと N-構成が同一の拒絶理由として報告されるため、読み出し故障を含む階層に接続する前に分離する必要がある（C664 (マウント拒否の正当性再計算が2種類の拒絶理由を1つに統合して報告する)）。フィールド別表におけるツリー ID 水位、2つのツリーのルートポインタ、`checkpoint_txg`、スキャンレポートの4セルは推論されたものである（三者判定は弁護側の計算式を採用し、未測定）、F のセルの「押し上げた」半面は未測定である；ルートレコードの `mapping_root`、`allocation_record_tree_root`、墓碑、系譜、deleted_inodes の3つのルートポインタ、アルゴリズム、nonce、MAC、システム構成のスロット世代番号および journal_tail が各々どのカテゴリに帰属するかは、C655 (マウント透明性のフィールド別表に未分類フィールドが存在する)、C687 (mkfs 水位からツリー構築したバージョンの配置先で判定不能な数セル) の判定 Y1, Y2 に従ってフィールド別表内にパディングで補完して記述している。

**根拠**：
- E42 (1トランザクションあたりのレコード数)：「分断された ⇒ 不正」を恒等式とするためにペイロードを固定した結果、「分断されたが偶然に正当」であるクラスが構造的に不可視となった —— M2 がペイロードを記号化して補完したのはまさにこの点である。
- 三者レビュー：資料 `_d13-item1-background.md`、3つの経路の出力 `d13-item1-opus-forward-output.md`、`d13-item1-sonnet-reverse-output.md`、`d13-item1-local-attack-v2-output.md`；3経路一致して旧候補を最下位とし、2つのクラウド経路はいずれも「プレフィックス判定とコミット原子性」の接合部に収束し、メイン agent は閉鎖性に基づいて最新選択とパブリッシュ原子性を同一ユニットに統合した（判定および旧候補撤回の3つの閾値の原文は変更履歴にある）。
- E165 (リカバリ再生の有界全数探索とミューテーション殺傷限界)：有界内において本物の `replay_journal` に対し M2 のセマンティクス層、M4 および M3 の両解釈を判定し、ミューテーション殺傷限界を測定；ユーザーは 2026-10-01 にこれを軽量ツールと決定；C ドメインの正当なクラッシュ状態数は2種の分割法で記述（甲分割法は D16 (パブリッシュセマンティクス) 確定項目 7 のバリア位置）；敵対的ファミリにおける不動点の違反はすべて I-8.12 (同一インスタンスレコードの txg 非減少) が成立しないレコード表上にあり、述語が成立する側では違反 0（第3回再実行第1区間）；ルート選択結合判定ファミリが本物の `choose_root`、`scan_journal`、`replay_journal` を呼び出し、有界内・正当・故障なしの条件下で確認済みの完全なパブリッシュが失われない（第3回再実行第2区間）；ルート選択順序の4つの改悪変種のうち、正当な履歴 K, K9, KW ファミリの有界内において最初の有効なものを選択のみが赤となり、他の3種は敵対的ルートリング KA 上で赤となる（第3回再実行第3区間）。
- 三者判定（M1–M4 の文面と配置先）：`research/prompts/m3-formal-r1-main-verification.md`、`research/prompts/m3-formal-r2-main-verification.md` 第2節、`research/prompts/m3-formal-r3-main-verification.md` 第2, 3節；M3 単調性の系の4段階の導出は `m3-formal-r2-opus-output.md`「3.2 推論：直列コミット下で、j1 と j2 から到達バージョンが単調であることが導かれる」の節にある。
- 三者判定（M3 不動点の前提条件への I-8.12 (同一インスタンスレコードの txg 非減少) の追加と配置先の現状）：`research/prompts/m3-formal-code-r1-main-verification.md` 第2節不動点前提条件セル（条項に命中、コードには命中せず）および第3節第5項；ハーネス側の前提が Verus の前提条件より広く、違反がすべて当該述語不成立側に位置することの照合は、スケジューリング記録 `records/2026-10-01-里程碑三形式化接入.md` 第10バッチ「10-5 返却」にある。
- ユーザー決定（2026-10-01 ポップアップ、すべて推奨通り）：アンマウント記号の除外、M4 射程文の前提条件の完全記述、単調性のバージョン順序に (`checkpoint_txg`, インスタンス番号) を採用、txg 上限到達時は「no_successor を報告し1件も適用されなかった場合は同等とみなす」を採用；書き戻しは K10 判定完了後にまとめて実施、原文は変更履歴にある。
- マウント透明性：実験なし：判定手法はクラッシュ注入内（`crates/singlefs-harness/src/mount_transparency.rs`、checker 層のクラッシュ注入がこれを呼び出す）にあり、実験ではない；3回の判定において攻撃側の数値はレプリカハーネス上で測定され、コードラウンド第2回で攻撃側がマージ済みコード上で測定した第1次クラッシュ 147 806 状態、二次クラッシュ 234 773 状態 0 赤について、メイン agent は再実行していない。
- 三者判定（マウント透明性）：`research/prompts/m3-formal-r4-main-verification.md` 第2節「マウント後に再リカバリ：何を法とするか」「比較対象と射程」の2行および第4節第2, 3項（独立した第4条、比較対象のバージョン、射程）；`research/prompts/m3-formal-r5-main-verification.md` 第2節 U5, U6 の2行および第4節第5, 6項（フィールド別表、二次クラッシュの v1 比較、正当な拒絶の条件）；`research/prompts/m3-formal-r6-main-verification.md` 第2節 U5, U6 の2行（txg は下限のみ判定、拒絶の判定基準はクラッシュイメージ上で自前で再計算）および「フィールド別表で触れていないフィールド」の行；`research/prompts/m3-formal-code-r2-main-verification.md` 第2節 K2 の各行（実装は判定通り、成立；ウォームアップとマウントによる F 引き上げのセグメント分割）。
- ユーザー決定（マウント透明性に二次クラッシュを追加、2026-10-02 ポップアップ、原文は変更履歴にある）：`research/prompts/m3-formal-r4-main-verification.md`「ユーザー決定（2026-10-02）」二次クラッシュの行。
- ルート選択順序変種の一文：スケジューリング記録 `records/2026-10-01-里程碑三形式化接入.md` 第12バッチ E165 (リカバリ再生の有界全数探索とミューテーション殺傷限界) 第3回再実行第3区間判定 Q2 返却の項目（メイン agent がこの文を補完すると判定）。
- マウント継続実行をマウントの一部とみなす、F の境界はマウントが返却した F 引き上げの書き込み表末尾における位置とする：ユーザー決定 2026-10-03「継続実行はマウントの一部とみなす」（原文は変更履歴にある）；コードラウンド第3回判定 `research/prompts/m3-formal-code-r3-main-verification.md` 第3節第3項（境界は書き込みストリームの位置に基づき取得、見つからない場合も panic しない）；実装 `crates/singlefs-harness/src/mount_transparency.rs`。

**未済の検査**：C77 (再生起点の未定義)：最新選択とプレフィックス判定の接合部にある；C649 (Verus で判定できない2種のコールバック結果)：弁済済み（M2 セマンティクス層の逆方向チェーンと指名ユニットは依然として後置条件としてのみ記述でき、Verus 側では判定せず、2つのディスク読み出しコールバックは `crates/mutations.tsv` のミューテーション行でカバーする方針に変更、未済の検査弁済済み一覧を参照）；C650 (ルート選択と再生の結合判定がない)：M4 の射程文はルート選択と再生が結合して初めて成立する；C653 (リングスキャンのグルーコードのメモリがディスク数に応じて増加する)；C654 (必赤世界の期待行数がソルバ環境によって変動する)；C655 (マウント透明性のフィールド別表に未分類フィールドが存在する)：弁済済み（フィールド別表は判定 Y1, Y2 に従ってパディングで補完、未済の検査弁済済み一覧を参照）。

#### 確定項目 2：まず直列コミットとする

**確定事項**：トランザクション層はまず直列コミットを公約する —— `.claude/rules/fs-design.md` の「判定基準が書けない場合は、まず1つの経路のみを作成し、かつ**低速であっても正しい経路を先に作成する**」を具現化するものであり、直列化は本プロジェクトが明示する既定値であって、2つの選択肢から選んだ結果ではない。交差点を越えた後に並行を開くのであり、交差点の前に並行を公約するのではない。将来の並行コミットに対する事前の制約事項：レコードはトランザクション番号順に追加され、トランザクション番号 N の fsync は復帰前に < N のトランザクションがすべて追加済み ∨ 無効化済み ∨ インスタンス切り替え確定または読み取り専用遷移済みであることを待機しなければならない（無効化はコミットパイプラインのメモリ状態であり、並行する後続から可視）；直列コミット下ではこの規則は空転する。

**射程**：直列化の上限は1回の checkpoint あたり約 1.1 万エントリである（シングルスレッドのマージ所要時間が1回のデバイス同期に追いつく交差点、計算式は C58 (直列コミットの上限到達時の警報不在) にある）：交差点以下ではデバイス同期がボトルネックであり直列化に不利はないが、交差点を超えるとシングルスレッドのマージが上限となり安全マージンが消滅する。確定項目 2 はレコードヘッダのフィールド一覧を決定せず、新規プールでのファイル作成時に書き出されるバイト列を変更しない。性能上限と crash refinement の適用範囲を決定する。並行コミットは依然としてクラッシュセマンティクスに影響を与える：「プレフィックス」は全順序上の概念であり、並行下ではプレフィックスが完全なトランザクション B と不完全なトランザクション A を同時に含む可能性がある；レコードヘッダにはすでにトランザクション境界フィールドが含まれている（D23 (journal の役割とフォーマット) 確定項目 7：トランザクション番号 8 + コミットマーカー 1）。

**根拠**：
- E17 (write buffer マージ上限)：シングルスレッドの sort-merge の上限は交差点の半分である；リーフ分割によるロックフリー並行処理は交差点を越えた後の退避口であるが、それが十分であると既定で仮定することは許されない。
- E44 (シーケンス番号ビット幅の実測とコスト)：ローカル fsync レートは交差点のもう半分である。
- E42 (1トランザクションあたりのレコード数)：直列下であっても1つのトランザクションが複数レコードに跨がる場合は同様にプレフィックスによって分断され得る —— レコードヘッダのフィールド一覧を決定するのは「1つのトランザクションがちょうど1件の journal レコードを生成するか否か」であり、直列 / 並行ではない。
- ユーザー決定、変更履歴に記録；将来の並行コミットに関する事前制約は C113 (スキャン再構築時に複数バージョンユニットの現行バージョン判定に入力がない) 確定事項 P1 に由来する。三者検証は経ていない：これはすでに明記されている既定値を確認するものであり、真に三者検証を経るべきはいつ並行に変更するかという点であり、そのステップは実施されていない。

**未済の検査**：C58 (直列コミットの上限到達時の警報不在)。

#### 確定項目 3：RefFS を極限まで突き詰め、自前で永続化とクラッシュセマンティクスを処理する

**確定事項**：機能のみを扱うインメモリモデルにとどまらず、自前で永続化とクラッシュセマンティクスを処理する参照実装（RefFS）を作成し、RefFS を「予備候補」から**作成すべき対象**へと引き上げる。「極限まで突き詰める」はセマンティクスレベルを意味する：RefFS はメモリ内でのみ POSIX 操作に従って動作し、各 fsync 後に何が永続化されたとみなされるか、クラッシュ後にどのような POSIX 状態の観測が許容されるかを自前で定義し、ディスク上フォーマットは持たない；独自のディスク上フォーマットを持ち、自前でディスク書き込みとリカバリを行う層（工数はもう1つのファイルシステムを開発することに匹敵）は作成しない。RefFS と実装の間でクラッシュポイント列挙のコードを共用することを禁ずる：極限まで突き詰めた後、「3 oracle 結託テスト」（確定項目 6）が防ぐ共通の誤解の最も危険な落とし穴は、「RefFS と実装が同一のクラッシュセマンティクスに関する誤った理解を共有していないか」である。

RefFS の由来と境界：

| 項目 | 確定事項 |
|---|---|
| 由来 | kb の条文のみを参照して独自に作成：O1 (インメモリモデル突き合わせ (HashMap 理想ファイルシステム)) のコードをコピーせず、記録ストリームを読み込まず、セグメント分割・状態計画・実装のクラッシュポイント列挙を呼び出さない |
| 配置先 | 独立したパッケージ `crates/singlefs-reference-filesystem`：依存関係クロージャ（dev-dependencies を含む）⊆ {それ自身, `singlefs-format`}、std のみで済む場合は std のみを使用；ソースコードは `singlefs_core`、`singlefs_checker`、`singlefs_harness`、`singlefs_checker_tier` を参照しない（リネームされたエイリアスも同様）；`singlefs-core` と `singlefs-checker` の依存関係クロージャにこれは含まれない。3条はゲート checker-independence-and-sync の checker-implementation-disjoint セルが判定する |
| 入力ホワイトリスト | すべて呼び出し側から渡され、5点のみを受理：各操作の結末（成功、いかなる書き込みの前での拒絶、書き込み後のエラー、同一再送）；クラッシュが何番目の操作で発生したか（hi）およびその帰属ルール；ロールバック先 m；各リカバリで読み出された内容；明示的なパブリッシュポイント |
| 第1版の操作 | 最初のファイル、上書き、パブリッシュ呼び出し、fsync、ロールバック |
| 呼び出し箇所 | クラッシュ注入および故障注入の呼び出し箇所において、RefFS の判定と O1 (インメモリモデル突き合わせ (HashMap 理想ファイルシステム)) のクラッシュ判定官の全4条の判定基準の論理積を取り、いずれか1つでも赤であれば赤とする |

クラッシュセマンティクス（直列コミット、故障を含まないクラッシュ状態）：

1. **fsync**：fsync が成功復帰した後、今回の fsync が待機したパブリッシュが引き連れたすべての操作が存在しなければならない（直列下では呼び出し開始前に番号取得に成功した全操作であり、ユニット単位で計算）；状態インデックスとして読み替えると、許容集合の下限 lo がそのパブリッシュの末尾まで引き上げられる。fsync がエラーを返した場合は lo を引き上げず、エラーとなったパブリッシュが引き連れた操作は存在しても存在しなくてもよい；以降の操作は次回のリカバリで読み出された状態に接続して判定する。
2. **欠落方法：パブリッシュ境界によるプレフィックス**。クラッシュが第 hi 番目の操作内で発生した際、リカバリで読み出しが許容されるのは {S_j : j = lo、または j がある明示的パブリッシュの末尾；lo ≤ j ≤ hi} であり、S_j は前回のリカバリ以降の先頭 j 個の操作を適用して得られる状態、S_0 は前回のリカバリで読み出された内容である。任意の部分集合を欠落させるような欠落方法は直列コミット下では除外され（D16 (パブリッシュセマンティクス) 確定項目 12 と非互換）、並行コミットを導入する際に再審議する。
3. **lo の引き上げ**：成功復帰したパブリッシュ呼び出し、fsync、ロールバックはいずれも lo を引き上げる（D16 (パブリッシュセマンティクス) 確定項目 7：ロールバックと fsync は同一の復帰条件）；ロールバックは1回の確認済みパブリッシュとみなされ、そのインデックスはパブリッシュ末尾として計算される。
4. **ロールバック**：ロールバックは RefFS の1つの操作であり、ロールバック後の内容は履歴全体における第 m 回の確認済みパブリッシュの後の内容と等しい；m は呼び出し側が実装の報告したロールバックルートから逆算して渡し、RefFS は対象の選定が正しいか否かを判定しない。m の番号付けはルート書き込みのすべての入口（パブリッシュ呼び出し、ロールバック、行書き込みとウォームアップ、F 引き上げ、アンマウント）をカバーし、それを書き込んだ時点の現行バージョンにマッピングされ、現行バージョンには各リカバリで読み出され新たに開始されたバージョンを含む。
5. **クラッシュ時点の帰属**：クラッシュが何番目の操作で発生したかは、「現在のセグメントを区切ったバリアがどのステップから発行されたか」に基づいて帰属させ、最初に差し押さえられた書き込みには基づかない；このルールは RefFS のみに適用され、O1 (インメモリモデル突き合わせ (HashMap 理想ファイルシステム)) のクラッシュ判定官は変更しない。
6. **リカバリ後**：各リカバリの後、RefFS はまず読み出された状態が許容集合内にあるかを判定し、その状態を起点として処理を続行する。

**射程**：どの程度まで突き詰めるかと RefFS の形態のみを決定し、スケジュールは決定しない；「実装の前にまず RefFS を作成する」と解釈してはならない —— RefFS は検証手段でありマイルストーンではない（`.claude/rules/format-evolution.md`「書き込みから開始し、読み出しから開始しない」）。明示的なパブリッシュポイントは履歴内のパブリッシュ呼び出しと fsync のみを取得する：自動トリガーによるバッチ処理が具現化するまで、RefFS は明示的パブリッシュポイントのみを受け付け、自動トリガーのパブリッシュは RefFS の射程外であり、確定項目 1 の M2 およびレコード照合器に帰属する；クラッシュリカバリで最新ルートを破棄した際、RefFS は読み出された内容から新たに開始し、そのステップ自体は判定しない。故障を含むクラッシュ状態（虚偽のデバイス、ディスク丸ごと喪失 C519 (ディスク丸ごと喪失時に直前に確認されたバージョンがリカバリで破棄される)）には適用されない。独立性：RefFS は記録ストリームを読み込まず、セグメント分割や状態計画を呼び出さないが、その (lo, hi) と O1 (インメモリモデル突き合わせ (HashMap 理想ファイルシステム)) のクラッシュ判定官のディスクレベル下限の入力パラメータは同源であり、C191 (同源性カウンタが1ペアのみ対象) の同源会計として記録される。D17 (実装階層とサードパーティパイプライン) 確定項目 4 はこれに基づき「入口」の実行可能定義を RefFS の操作面に委ね、fsync の永続性公約は RefFS のクラッシュセマンティクスによって定義される；O3 (独立規約実行器) は依然としてクラッシュポイント集合の独立実行器であり、責務は不変である。RefFS の第1版は `crates/singlefs-reference-filesystem` にあり、クラッシュ注入の2区間および故障注入再開の呼び出し箇所に接続済みである：harness 層は変換層 `crates/singlefs-harness/src/reference_filesystem_translation.rs` を介してこれに依存し、呼び出し箇所では RefFS と O1 (インメモリモデル突き合わせ (HashMap 理想ファイルシステム)) のクラッシュ判定官の2票を判定し、いずれかが赤であれば赤とする。

**根拠**：
- 実験なし：どの程度まで突き詰めるか、RefFS の由来と入力の選定は検証投資と境界のトレードオフである；3回の判定における攻撃側の数値はレプリカ装置上で測定されたものであり、マージ先コード上では未再実行であるため（`research/prompts/m3-formal-r5-main-verification.md` 第5節第3項）、ここでは引用しない。
- ユーザー決定、原文は変更履歴にある：「極限まで突き詰める」（当初確定）；セマンティクスレベルとする（2026-10-02、分岐表 `m3-formal-7-forks.md` 内の RefFS をどの階層まで作成するかの行、当初確定事項の解釈変更）；パブリッシュ境界によるプレフィックス、呼び出し箇所の論理積、新規作成、独立パッケージ（2026-10-02、`research/prompts/m3-formal-r4-main-verification.md`「ユーザー決定」）；論理積を全4条へ厳格化、ロールバック先を入力とする（2026-10-02、`research/prompts/m3-formal-r5-main-verification.md`「ユーザー決定」）；ロールバックによる lo 引き上げ、クラッシュ時点のバリア帰属、m が全ルート書き込み入口をカバー、ホワイトリストに同一再送を追加（2026-10-02、`research/prompts/m3-formal-r6-main-verification.md`「ユーザー決定」）。
- 三者判定：`research/prompts/m3-formal-r4-main-verification.md`（第1ラウンド：fsync、欠落方法、独立性の3セルと6つの分岐）、`research/prompts/m3-formal-r5-main-verification.md`（第2ラウンド：fsync 合成文、パブリッシュ境界による補足3文、論理積厳格化、新規作成と独立パッケージ、任意部分集合除外の直列限定）、`research/prompts/m3-formal-r6-main-verification.md`（第3ラウンド最終ラウンド：ロールバックと lo、射程2文、ホワイトリスト、状態インデックスによる fsync 読み替え）。
- D17 (実装階層とサードパーティパイプライン) 確定項目 4：O1 (インメモリモデル突き合わせ (HashMap 理想ファイルシステム)) は「クラッシュは関知しない」と明記しており、fsync の永続性公約を定義できないが、最初の実行可能目標はまさに「1つのトランザクションを正しくコミットすること」である —— 極限まで突き詰めた RefFS が補完するのはまさにこの半分である。

**未済の検査**：C447 (規約実行器がクラッシュポイント再生および RefFS 比較に未接続)；C191 (同源性カウンタが1ペアのみ対象)；C658 (クラッシュ注入第3段階が失われたステップの復活を見逃す)。

#### 確定項目 4：クラッシュポイント再生は完全書き込みの部分集合のみを列挙する

**確定事項**：クラッシュポイント再生が列挙すべきクラッシュ状態集合は以下のように定義される：記録された書き込みリクエストストリームを**デバイス単位で**セグメントに分割する —— ある書き込みは、同一ディスク上の後続のバリア（またはそのディスク上の FUA 書き込み）によってのみ後続の書き込みより前に順序付けられる；現在のセグメント内で書き込みのあった各ディスクが自らのバリアまたは FUA で解放されたときに初めてそのセグメントが閉じられ、1枚でも解放されていないディスクがある場合、そのバリア（または FUA）はセグメントを閉じず、前後の書き込みは同一セグメントとなる。1つのクラッシュ状態は「先行するいくつかのセグメントがすべて永続化され、現在のセグメント内の各書き込みが各自の状態を任意に組み合わせて取得し、後続の各セグメントはすべて未永続化である」状態である；列挙の単位は**1回の完全書き込み**である。FUA を伴う書き込みは自ディスクのみを解放する：それはそのディスク上で先行するバリアで隔てられていない書き込みと同一セグメントとなり、他ディスク上の書き込みがそれによって永続化されることはない。1回の書き込みは2つの状態（永続化 / 未永続化）を取る；**インプレース上書き**（ユニット書き込みではなく、1セクタより長く、対象範囲に元々データが存在していた書き込みであり、システム構成スロットの書き込みや旧レコードを上書きする journal 書き込みがこれに該当）は第3の状態「新旧いずれも読み出せない」を追加で取り、フル実行およびクイック層の双方が列挙する；第3の状態のイメージはその書き込みの全範囲を覆い、新旧で異なる区間の前半は新しいバイト、後半は古いバイトとする（セクタ単位ではなくバイト単位で断裂）；その他の書き込みの断裂状態は一律に「今回の書き込みが未永続化」に統合する。「未永続化」の位置には、イメージ内に**クラッシュ前のその位置の古いバイト**を配置し、ゼロ埋めは行わない。また、これらのイメージ（第3の状態を含む）は実際に生成して checker およびレコード照合器に渡す必要があり、単なるカウンタとして扱うことは許されない。イメージのミラー書き込み（D2 (RAID ストライピング戦略) 確定項目 6 の w ≥ 2、D22 (ユニットの原子性をどう合成するか) 確定項目 8 の journal ミラー）は**デバイス単位で**各1回の書き込みとして計算し、harness の状態数は自らの書き込みストリームに従って別途閉じた式を計算する。**同一ディスク上で連続するバリアは1つに統合する**：記録ストリーム内で、あるディスク上のバリアと同一ディスク上の次のバリアとの間に書き込みが一切存在しない場合（他ディスクのバリアは書き込みとみなさない）、このバリアは記録せず（`crates/singlefs-harness/src/lib.rs` の `push`）、独立モデルも同一規則に従って記録する；中間に書き込みがなければそのセグメントはクラッシュ状態を生成せず、統合の有無はクラッシュ状態集合を変更せず、操作数と種類文字列のみを変更する。

**射程**：モデル層の列挙ドメインを定義する。実デバイスの FLUSH / FUA が公約通りに効力を発揮するか否かは C6 (ブロック層セマンティクス仮定の記述誤り) に帰属し、QEMU 内で実デバイスを用いて検証する必要がある。デバイス単位のセグメント分割により、実際には到達し得ない状態が余分に生成される（1台のディスクが解放されていないとき、他ディスクの解放済み書き込みもセグメント内に残って任意の組み合わせを取る）が、到達可能な状態の取りこぼしはない。断裂は2種類に分かれる：インプレース上書き以外の書き込み（ユニット書き込み、1セクタ以下の書き込み、対象範囲に元々データが存在しなかった書き込み）の断裂状態と「未永続化」の差異は CRC32C の衝突確率（32 ビット、D19 (ブロックポインタの構造と幅の予算) 確定項目 2、D23 (journal の役割とフォーマット) 確定項目 11）のみであり、**認識した上で受容**し、断裂粒度はモデルに含めない。これは D20 (荷重面：ユニットの原子性と自己完結性) の「親ポインタを持つユニットはいかなる幅にも依存しない」と整合する；インプレース上書きが断裂した場合、その箇所の旧データも失われるが、「未永続化」では完全な旧データが保持されるため、両者は同一の状態ではなく、個別に列挙する。クラッシュポイント再生のセクタは 512 バイトとする；2本のクラッシュポイント再生ストリームにおいて第3の状態を取るのはシステム構成スロットの書き込みのみである。既知のエッジケース：第3の状態の旧バイトは「今回の書き込み以前の書き込みがすべて永続化した」バージョンを採用し、同一セグメント内のより早期の重複する書き込みが永続化しなかった組み合わせでは、古い半分にその書き込みのバイトが多く含まれる（2本のクラッシュポイント再生ストリーム上ではこのような組み合わせは一度も発生していない）；ルートスロットの書き込みが1セクタを超えかつ旧データを上書きする場合、第1版では第3の状態をサポートしない（harness アサーション；クラッシュポイント再生のプールはすべて 512 バイトの物理ブロックで構築され、到達しない）。デバイスが1回の書き込みをチェックサム衝突にちょうど重なる形状に断裂させるケースはモデル外である。D13 (検証ルート) 確定項目 5 はこれと直交する：一方は列挙ドメインを決定し、他方は crate の境界を決定する。

**根拠**：
- E77 (パブリッシュの永続化順序)：セグメントモデルの4種のバリア配置における状態数は、アームごとに閉じた式と厳密に等しい —— この列挙ドメインの数え上げは正しい。
- E142 (新規プールでのファイル作成のドライラン)：第17回実行第1区間の幾何学的感度サンプリングポイント G7（同一書き込みストリームに対する陽性対照、FUA のみを境界とする）の実測で 4097 状態、2046 違反 —— バリアが存在しない場合、ユニット、journal レコード、ルートスロットが1つのセグメント `[12]` に統合される；装置の分割法は `crates/singlefs-harness/src/segments.rs:91` の「FUA 書き込みは自身が属するセグメントを閉じる（FUA は前方の通常書き込みを永続化しないため、それらは同一セグメントとなり任意の部分集合を取る）」と一致し、「FUA が単独でセグメントを形成する」という解釈が予言した 2050 / 0 と一致しない —— FUA の一文の字面は実装の分割法に従って記述されている。
- 三者対抗および判定：[verification-build.md](../verification-build.md)「三者対抗（2026-09-03）」判定第3項 —— すべての種類のユニットがユニット全体の検証を持つ（親ポインタを持つものは親ポインタ内のチェックサムに依拠、D4 (チェックサムの位置) 確定項目 1、D16 (パブリッシュセマンティクス) 確定項目 7、D22 (ユニットの原子性をどう合成するか) 確定項目 19；自己検証ユニットはユニット全体のチェックサムと実際に検査されるコードに依拠、D22 (ユニットの原子性をどう合成するか) 確定項目 21；journal レコードはヘッダチェックサムとヘッダ内のペイロードチェックサムに依拠、D23 (journal の役割とフォーマット) 確定項目 13）、断裂状態と「未永続化」はチェックサムの観点からは同一の事象である。
- ユーザー決定（E77 (パブリッシュの永続化順序) および三者対抗に従って決定；FUA 書き込みをセグメント境界とみなす一文は C313 (FUA 書き込みをクラッシュセグメント境界とみなすか否か) 完了時のユーザー決定であり、2026-09-26 にユーザーが実装に合わせて「自身が属するセグメントを閉じ、前方のバリアで隔てられていない書き込みと同一セグメントとする」に変更）、変更履歴に記録。
- デバイス単位のセグメント分割（1台でも解放されていないディスクがあればセグメントを閉じない）、FUA は自ディスクのみを解放、インプレース上書きに「新旧いずれも読み出せない」を追加：ユーザー決定（コードレビュー第1, 3, 4項；2026-09-27 ポップアップで「A 統合、デバイス単位でバリアを記録」「同様に補完、フル実行も走らせる」「インプレース上書きに第3の状態を追加」を選択）、第3の状態をバイト単位で断裂させることはメイン agent が同日に決定し、ゼロラウンド攻撃された、変更履歴に記録；実装と判別力（ディスク1のバリア1つのみを吸収、セグメント末尾 `26,2,1,2` → `26,5`、クラッシュポイント再生小ストリーム内でレコード照合器が報告）は `m2-rev-b3a2-implementer-report.md` 第1, 3節にある。
- 同一ディスク上で連続するバリアを1つに統合：ユーザーが 2026-09-28 に「統合し、本日のレコーダーに追従する」と決定（E142 (新規プールでのファイル作成のドライラン) 第19回実行で登録された停止 S19-crates に遭遇：独立モデルは双方を記録し、レコーダーは2つ目を統合、ウォームアップと全ストリームの4セルで操作数と種類文字列が異なるがクラッシュ状態数は同一、報告 `e142-r19-runner3-report.md` 結論段、登録 `e142-r19-prereg.md` の R40）、原文は変更履歴にある。

**未済の検査**：C6 (ブロック層セマンティクス仮定の記述誤り)：仮想マシンのデバイス側ログを用いてクラッシュ状態を再構築し、クラッシュテストが必ず赤になる側の半分。

#### 確定項目 5：kb から生成された定数モジュール1つのみを共有する

**確定事項**：checker と実装の間で共有するものは唯一つ：kb のフィールド表から生成された定数モジュールであり、ジェネレータが kb から読み取り、双方が消費するのみで、何人も手作業で改変してはならない（`.claude/singlefs-ai-sop/rules/machine-first.md`：重複は生成すべきであり、手作業で書き写してはならない）。**それ以外は一律に共有しない**：アドレス空間の newtype は各自で宣言し、フォーマット解析は各々独自に実装し、チェックサムは各々独立した実装を用い、トラバーサルおよび会計コードの積集合は空とする（C12 (増分セマンティクスの共用) はゲート checker-independence-and-sync の checker-implementation-disjoint セルが定めた判定基準であり、形態は crate 粒度の依存関係クロージャの積集合でありシンボルレベルではない、「フォーマット解析と定数を除く」という例外規定はまさに本項目が利用している）。

- **判定幅を他所から取得することを禁ずる**：I-7.5 (ルートスロットの判定幅アライメント) および I-8.2 (レコードヘッダが原子ユニットを跨がない) の判定基準はマウント時に検出された `physical_block_size` であり、イメージ内には存在しない。checker は自ら1回検出し、システム構成に記録された mkfs 時の値と照合する（D22 (ユニットの原子性をどう合成するか) 確定項目 2「mkfs 時にプール内最大値に基づいて区画を割り当て、マウント時にデバイスごとに再計算して照合する」の checker 側の形態）。**検出不能時は「宣言値、未検出」と報告し、検出されたものとみなすことを禁ずる** —— ファイルをデバイスとするイメージがこれに該当し、クラッシュポイント再生の大多数のイメージはファイルであるため、この規定はエッジケースではない。異種混在プールにおいて判定幅は単一の数値ではなく集合であり（C17 (デバイスジオメトリをグローバルパラメータとみなす誤り)）、checker は mkfs 時の最大値取得の集約を再計算しなければならない。さもなければ正常な異種混在プール上で誤って赤を報告することになる。
- **ジェネレータは kb に配置先のない定数の出力を拒否する**、またスカラー値のみを出力する：仮に「フィールド表に基づいて計算されたオフセット関数」を出力し始めた場合、それは D13 (検証ルート) が明示的に共有を禁止したフォーマット解析となってしまい、もはや定数ではない；判定基準は出力物に分岐と演算が含まれているか否かである。
- **CRC32C を各自で実装する前提はパラメータの固定である**（多項式、初期値、入力/出力の反転、排他的論理和出力）：パラメータを固定しなければ、2つの独立した実装が健全なイメージ上で異なる結果をもたらし、それは独立性ではなく誤判定の赤となる。

**射程**：checker と実装の crate 境界を管轄する；kb 内でそもそも誤って記述された値は管轄できない —— 生成された定数を1つ共有することと双方で個別に書き写すことのいずれもそのクラスの誤りを捕捉できず、直交するゲートを必要とする（C94 (登録されたフォーマット定数と後続の確定事項の不一致)）。実装と確定事項の乖離：`crates/singlefs-format` は手書きであり生成されたものではない（値はゲート code-source-discipline の format-constants セルが kb 内の `format-const` マークに従って固定しており、マークされたもののみを固定）、ジェネレータは存在しない；`crates/singlefs-checker` は `singlefs-format` のみに依存し、CRC-32C はビット単位のものを別途実装している；checker はシステム構成が宣言した `physical_block_size` と `io_min`（`crates/singlefs-checker/src/image.rs` の `geometry_of`）を取得し、「検出 / 宣言」の2種類の出所の型は `crates/singlefs-checker/src/lib.rs` の `DecisionWidth` である：宣言値の形式（`DeclaredOnly`）はすでに I-7.5 (ルートスロットの判定幅アライメント) の判定に接続されており（実装済み、`decision_width_declared_by_the_chosen_system_configurations` は各ディスクで選択されたシステム構成が宣言する max(`physical_block_size`, `io_min`) の最大値を取る）、検出値の形式（`Probed`）は型定義のみが存在し（`crates/singlefs-checker/src/lib.rs`）、リポジトリ全体にそれを構築するコードはなくいかなる判定にも接続されていない；I-8.2 (レコードヘッダが原子ユニットを跨がない) は未実装である。システム構成における mkfs 時の `physical_block_size` フィールドは D22 (ユニットの原子性をどう合成するか) 確定項目 9 のジオメトリセグメント（4バイト）にある。

**根拠**：
- 実験なし：共有の境界は検証の独立性を担保する規律であり、測定可能な量はない；その判別力は双方が個別に実装した検査が1条ずつ赤を報告して示すことに依拠する。
- 三者対抗：資料 `_checker-sharing-vehicle-background.md`、各経路の出力 `research/prompts/checker-sharing-vehicle-*`、状況は [verification-build.md](../verification-build.md) にある —— 順方向経路は D13 (検証ルート)、C12 (増分セマンティクスの共用) から「定数は例外内、他は禁止」を導出；代替逆方向経路とローカル経路は各々同一の結論を構築：kb 内で誤って記述された値は共有と個別記述のいずれでも捕捉できない。
- ユーザー決定、変更履歴に記録。

**未済の検査**：C17 (デバイスジオメトリをグローバルパラメータとみなす誤り)；C387 (定数モジュールが手書きで kb から生成されていない)；C388 (checker の判定幅を自前で検出しない)。

#### 確定項目 6：互いに独立な3つの oracle

**確定事項**：3つの oracle が各々異なる対象を判定し、独立性の根拠は各々異なる：

<!-- doc-lint:registry name-col=2 -->

| # | oracle | 判定対象 | 独立性の根拠 | 検証できないもの |
|---|---|---|---|---|
| O1 | インメモリモデル突き合わせ (HashMap 理想ファイルシステム) | POSIX 可視セマンティクス：内容、サイズ、ディレクトリ構造；クラッシュのない箇所での内容の期待値はこれに帰属し、クラッシュ後に許容される観測集合は RefFS（確定項目 3）に帰属する | ディスクに触れず、ディスクフォーマットを解析しない | ディスク構造が自己整合しているか；クラッシュセマンティクス；**同一の POSIX セマンティクスに対する2つの実装の共通の誤解** |
| O2 | 独立パーサ + checker | ディスクバイト列が I-1 (ブロックの自己記述) ～ I-6 (暗号化不変条件) を満たすか | **checker はテスト対象 crate のいかなるデシリアライズ / トラバーサルコードも呼び出してはならず**、解析処理を独自に再実装し、raw デバイスを直接読み出す必要がある | 構造の自己整合性のみを判定：あるブロックが別の**正当な**内容で上書きされた場合、Merkle チェーンは依然として自己整合し、会計も一致してしまう |
| O3 | 独立規約実行器 | クラッシュポイントで切断された後の各ディスク状態が規約の許容集合に含まれるか | **別の言語で実装**（Rust を使用しない）、decisions と invariants の記述のみから独自に再導出；第1版は Python で単独実装（E170 (O3仕様実行器第1版)） | 規約自体が誤って記述されている可能性 —— **陽性対照と組み合わせることで初めて意味を持つ**；射影される6種の要素の値および6種以外のルートレコードフィールド、実装自身のリカバリおよびリーダーのコード、システム構成スロットの証拠値（journal_tail）、1回のパブリッシュ内での各書き込みの前後関係 |

**独立性は検証されるべきであり、自己申告を禁ずる**：「3 oracle 結託テスト」を設計する —— 精密に構築された誤った実装（例えば nonce が常に 0）を用いて3つの関門を通し、3者すべてがパスさせた場合は共通の盲点が存在することを示しており、oracle の補完に立ち戻る。機能の正当性はこれら3つの oracle のみに依存するわけではない：O1 (インメモリモデル突き合わせ (HashMap 理想ファイルシステム)) の行はすでにクラッシュ後に許容される観測集合を RefFS（確定項目 3）に委ねており、リカバリ関数の M2–M4 は Verus および E165 (リカバリ再生の有界全数探索とミューテーション殺傷限界)（確定項目 1）に配置され、第2の入力を必要とする照合はレコード照合器（確定項目 7）に帰属する；3つの oracle 間の共通の盲点は「3 oracle 結託テスト」によって発見する。

**射程**：3つの oracle が各々何を検証できないかは表の最終列に記載されている；O2 (独立パーサ + checker) の定義域は確定項目 7 に帰属し、第2の入力を必要とする照合はレコード照合器に帰属する。**実装の現状**：O2 (独立パーサ + checker) は `crates/singlefs-checker`（フォーマット定数 crate にのみ依存）にある；O1 (インメモリモデル突き合わせ (HashMap 理想ファイルシステム)) の形態は `crates/singlefs-harness/src/model.rs` の理想モデル（メモリ上にのみ存在し、`singlefs_format` の定数のみを使用し、クラッシュは構築しない）であり、ゲート harness-model-differential-and-scenarios がランダム履歴を用いて段階的に突き合わせを行う；O3 (独立規約実行器) は kb の記述のみに基づいて書かれた Python 実行器であり、現行のものは E171 (3 oracle 結託テスト) 第1回再実行で使用された `scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates_journal_header_487/`（E170 (O3仕様実行器第1版) 第2回再実行が第1回再実行で凍結された `scripts/experiment-apparatus/e170_second_run_o3_spec_executor/` から `scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates/` をコピーし述語 W, T を追加、さらに 487 バイトの journal レコードヘッダに追従；第1回再実行で凍結された版と初回の版 `scripts/experiment-apparatus/e170_o3_spec_executor/` はアーカイブ済み）である：厳密な実行方法により判定可能であり（初回にエクスポートされたバッチ1、バッチ2において「規約の記述不足」に遭遇した状態は 0）、実装と状態ごとに一致し（両者とも完全にパス、これを正しく判定した証拠とはみなさない；「選択されたルートが自己検証済みルートの中で txg 最大のものではない」「③ 2スロット以上を受信」の2つのパスはバッチ内で 0 状態、C661 (規約実行器バッチ内で2つのパスに状態が存在しない)）、不正状態表を修正後の期待値に従って再判定すると判別力を有し（陽性対照およびブラインドテストの双方が正しく判定）、クラッシュポイント再生にはまだ接続されていない；あるクラッシュ状態を判定する手順：規約のリカバリ規則（システム構成選択、ルート選択、プレフィックスは D23 (journal の役割とフォーマット) 確定項目 14 の5条の計測条件に基づく）に従ってイメージから自前で1バージョンをリカバリし、リカバリ不能（プール全体の拒否または自己検証済みルート不在）の場合は集合外と判定；リカバリされたこのバージョンの射影（インスタンス番号、checkpoint_txg、ツリーテーブルユニットポインタ、中央マッピングツリールートポインタ、ツリー ID 水位、F）がバージョン表内のいずれかのバージョンの射影と等しく、かつそのバージョンのインデックスが [確認済み, 開始済み] の間にあること。バージョン表はベースイメージ上で自前でリカバリされた現行バージョンに加え、記録ストリーム内の各ルートスロット書き込みをストリーム順に各1バージョンとし、各バージョンの射影は実装がルートスロットに書き込んだバイト列を取得する；確認済みはクラッシュ状態で永続化した最新の書き込みより前に復帰済みの最新バージョンを取り（永続化した書き込みからのみ推論）、開始済みは永続化した書き込みが属する最新バージョンを取る。システム構成スロットはスロット選択、ルートリングおよび journal リングのジオメトリ取得、プールレベルフィールドの範囲外によるプール全体拒否の判定にのみ用いられる。したがってリカバリ規則の半分は kb の記述に基づいて別途実装されたものであり、許容集合の半分は実装自身が書き出したルートから取得される；RefFS（確定項目 3）は `crates/singlefs-reference-filesystem` にあり、クラッシュ注入および故障注入の呼び出し箇所に接続済みである；「3 oracle 結託テスト」は E171 (3 oracle 結託テスト) であり、測定値は凍結されたツリーから取得される（スナップショットダイジェストは実験ページ参照、現行の `crates/` とは同一ではない）：O1 (インメモリモデル突き合わせ (HashMap 理想ファイルシステム)) の列はモデル突き合わせ + モデルのクラッシュ判定官であり、RefFS には接続されておらず、表の O1 (インメモリモデル突き合わせ (HashMap 理想ファイルシステム)) の行とは異なる（クラッシュ後の集合の半分において E171 (3 oracle 結託テスト) は RefFS を使用していない）；O3 (独立規約実行器) の列は E170 (O3仕様実行器第1版) 第1回再実行で凍結された版を使用；手動構築された不正イメージファミリは O2 (独立パーサ + checker)、O3 (独立規約実行器) のみに供給され、期待値は E170 (O3仕様実行器第1版) の設計者によるものであり、Q2 は登録字面通りに「存在する」（4枚とも P8）と判定し、登録 F3 に基づいて E170 (O3仕様実行器第1版) の期待値推論の誤りでありこれら4枚は不正イメージではないと判定され、真の不正イメージの中で O2 (独立パーサ + checker)、O3 (独立規約実行器) の両者が完全にパスしたものは 0 枚である；3者の出所が独立であるか（表第4列）について E171 (3 oracle 結託テスト) は Q2 において O2 (独立パーサ + checker)、O3 (独立規約実行器) のペアのみを測定しており、そのペアのパッチと期待値は E170 (O3仕様実行器第1版) の改修コードに由来するため、独立した測定値とはみなさず、O1 (インメモリモデル突き合わせ (HashMap 理想ファイルシステム)) は Q2 に含まれていない；測定されていない項目を「互いに独立」が検証されたと記述してはならない。

**3つの oracle がいずれも判定しない2点**（第1点は現在クラッシュ注入側に判定器が存在する）：第1に、読み出し故障のないクラッシュ状態において、D23 (journal の役割とフォーマット) 確定項目 14 の書き込み可能マウントの証拠判定基準が真であること（システム構成スロット内の証拠値がディスク上から読み出せるレコードより先行している）：クラッシュ注入第2段階は拒絶が正当か否かを判定する際にクラッシュイメージ上でこの判定基準を再計算するのみであり、真であれば正当と判定する（マウント透明性の射程は故障を含まない状態のみを管轄する、確定項目 1）；「このクラッシュ状態には読み出し故障が注入されていないのに、判定基準が真である根拠は何か」を問う判定器が必要である：クラッシュ注入第2, 3段階は読み出し故障が注入されていないクラッシュ状態上でこの判定基準を再計算し、真であれば拒絶が不正であると判定する（`crates/singlefs-checker-tier/src/crash_injection.rs` の `RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults`、C670 (読み出し故障のないクラッシュ状態での書き込み可能マウント証拠判定が真であることの判定器不在) は弁済済み、未済の検査弁済済み一覧を参照）。「読み出し故障のないクラッシュ状態において各ディスクに少なくとも1部の自己検証済みシステム構成が存在する」という性質は G4 が判定する（クラッシュ注入第2, 3段階、C670 (読み出し故障のないクラッシュ状態での書き込み可能マウント証拠判定が真であることの判定器不在) と同一の接続方法）：mkfs 復帰前のクラッシュ状態は G4 の射程外であり、mkfs はディスクごとに2スロットを同一ループ内で書き込みその後に初めてディスクごとにバリアを発行するため、クラッシュ注入の候補セグメントと E171 (3 oracle 結託テスト) のベースイメージはいずれも mkfs の後に取得される；G4 とその条項にはもう1点形状の相違がある：システム構成を選択できない場合、その条項の再計算は `None` を返すが G4 は真とする。この時読み出しフェーズ自体がすでに拒絶しており、このセルには到達しない。第2に、1回のパブリッシュ内での各書き込みの前後関係：O2 (独立パーサ + checker) の I-7.12 (システム構成の F が同一ディスクルート上の F 以上) は、F 引き上げシーケンスにおいてまずシステム構成を書き込むステップを判定する；ミューテーションで指名されたテストケースは E171 (3 oracle 結託テスト) 270行目の種類（パブリッシュ復帰後の各クラッシュポイントにおける証拠値）のみを判定し、242行目で指名されたテストケースは失敗パスの凍結を判定する；`layout/01-first-txn.md` 第8節はルートスロットの FUA とシステム構成スロットのローテーションを2セグメントとして登録しているが、登録簿上に存在するに過ぎない；その他の書き込み順序は3者には含まれない。

**根拠**：
- E170 (O3仕様実行器第1版)：O3 (独立規約実行器) は「別の言語、kb の記述のみから再導出、陽性対照を配備」に従って第1版を記述可能であり、単一のクラッシュ状態を判定でき、1状態あたりの所要時間はクイック層に組み込み可能である；第1回再実行（2026-10-03 の kb に従って1回改修された実行器）は厳密な実行方法で判定可能であり、サンプリングされたバッチ状態上で実装と一致し、陽性対照は判別力を有する（初回実行で判定不能であった点および判別力不足であった2点は実験ページに記録）。
- G4（読み出し故障のないクラッシュ状態において各ディスクに少なくとも1部の自己検証済みシステム構成が存在する）：コードラウンド判定 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第2節 j2-b 成立、第29バッチで実装、射程と mkfs に関する部分は `research/prompts/m3-formal-code-b29-r1-main-verification.md` 第2節 L3 および第3節第3項を参照。
- 3つの oracle の分担とそれぞれの独立性の由来は検証体系の構造設計である；独立性は「3 oracle 結託テスト」によって観測される（E171 (3 oracle 結託テスト)）。
- E171 (3 oracle 結託テスト)：凍結されたツリー上において、不正実装表の中に3つの oracle すべてがパスさせた行があり、1つのみによって捉えられた行もあるが、O3 (独立規約実行器) は不正実装表上で1行も赤を出さなかった；手動作成の偽造ルート、偽造継続イメージの中に O2 (独立パーサ + checker) がパスさせ O3 (独立規約実行器) が集合外と判定したものがある（測定値と限定条件は実験ページ Q1, Q2, Q3 の3行にある）。
- 三者論証：D13 (検証ルート) 新設時のラウンド（ローカル LLM / Sonnet / Opus が各1経路、立場重複なし）、資料は保存されていない、経過は変更履歴にある。
- O1 (インメモリモデル突き合わせ (HashMap 理想ファイルシステム)) と RefFS の分担文：三者判定 `research/prompts/m3-formal-r5-main-verification.md` 第2節の新規作成および独立パッケージのセル（両経路が同一方向）。

**未済の検査**：C445 (モデル突き合わせのサンプリング点が書き込みパスに応じて増加しない)；C446 (未実装の不変条件に事前登録がなく期日もない)；C447 (規約実行器がクラッシュポイント再生および RefFS 比較に未接続)；C407 (3 oracle 結託テストの判定器がゲート未導入)：E171 (3 oracle 結託テスト) は1回測定されたが、その成果物を読み込むゲートセルは存在しない；C672 (戻り値やメモリ状態が誤っているが書き込みストリームが不変であるクラス)；C679 (アンマウント記号がモデル突き合わせに含まれていない)。

#### 確定項目 7：O2 は何を判定するか：単一イメージとレコード照合器

**確定事項**：

> **確定事項：O2 (独立パーサ + checker) の定義域は単一イメージである** —— 単項述語 `P(イメージ) → 成立/不成立` であり、
> 集合の要素性のみを判定し、決して identity を判定しない。**第2の入力（別のイメージ、または一連のレコードストリーム）を必要とする検査はいずれもこれに属さない。**

**第2の入力を必要とする照合はレコード照合器に帰属する**。入力パラメータは `(クラッシュ前イメージ, レコードストリーム, クラッシュ後イメージ, 永続化集合)` である —— 永続化集合は列挙器が提供する、このクラッシュ状態においてどの書き込みがディスクに永続化したかを示す情報である（クラッシュポイント再生のセグメント内完全書き込み部分集合）；ディスク上の生バイトを参照するだけでは「書き込みが未永続化であり、元の位置の内容が偶然同一であった」ことと「書き込みが永続化した」ことを区別できないため、再利用と欠損の判定には永続化集合を用いて判定しなければならない；これは独自開発第1項の「Merkle ルートチェーン差分」（確定項目 8）と同様に二項検査に属する。⚠️ **意図的にこれに番号を付与しない** —— O2 (独立パーサ + checker) をめぐる混乱は番号を呼称として用いたことに起因しており、さらに O4 を追加することは次なるドリフトの温床を提供するに過ぎない。⚠️ **「クラッシュ後イメージ」は1つではなく2つである**：harness の実行順序はまず実装自身のリカバリを実行し、その後にレコード照合器を実行するが、リカバリ自体がディスクを変更する（インスタンス切り替えによる行書き込み、インフライトな checkpoint の再送、D23 (journal の役割とフォーマット) 確定項目 14）⇒ ルート選択とプレフィックス判定は**クラッシュ状態イメージ**を参照し（「レコードストリーム」はそのリング内の journal レコードである）、比較対象は**実装がリカバリした後のイメージ**となる。

**射程**：ポインタ層の目標状態下において、1件のレコードを照合するにはフォーマット解析 + チェックサム + ルートリングの最新選択のみで足りる；論理意図レコードはいかなるディスク上オブジェクトも指名しないため、照合側はツリーセマンティクス + 会計 + 割り当て正当性を自前で実装しなければならない —— D23 (journal の役割とフォーマット) が論理意図ログを否決した第2の理由はこれに基づき第1の理由の1つの封じ込めとして書き直され、D23 (journal の役割とフォーマット) 確定項目 5 に記述されている。レコード照合器のコードは `crates/singlefs-checker-tier/src/crash.rs` にあり、クラッシュポイント再生の各クラッシュ状態に接続されている（ゲート checker-tier-crash-point-replay）。したがってレコード照合器の判定は単なるイメージの関数ではない：クラッシュ後イメージがバイト単位で同一であっても永続化集合が異なる2つの状態に対して、異なる判定を下すことが可能であり、イメージに基づく重複排除による高速化はこれには適用されない（O2 (独立パーサ + checker) および oracle は従来通りイメージのみを参照する）。クラッシュポイント再生は書き込み可能マウントをカバーしない：クラッシュポイント再生は各クラッシュ状態において読み取り専用リカバリのみを実行し（journal を参照する場合と参照しない場合を各1回）、番号取得、行書き込み、ウォームアップはクラッシュポイント再生のいかなるクラッシュ状態上でも実行しない。書き込み可能マウントはクラッシュ注入によってカバーされる（`crates/singlefs-checker-tier/src/crash_injection.rs`）：抽出された各クラッシュ状態において、読み取り専用リカバリと判定の後に同一のクラッシュ後イメージ上で書き込み可能マウントを1回起動し、再度パブリッシュを行い、プールレベル checker を実行する；マウント途中で再度クラッシュさせ、番号取得、行書き込み、ウォームアップの3区間に各1つの二次クラッシュ状態を配置し、各々において読み取り専用リカバリ、モデルへの問い合わせ、プールレベル checker、レコード照合器を実行する：第3段階においてレコード照合器に渡される書き込み表は「マウント区間から現在のセグメントまでのプレフィックス」から履歴全記録ストリーム + 全マウントストリーム + マウント後のパブリッシュへと置き換えられ、永続化集合はセグメントごとに対応する（履歴区間は第1次クラッシュの永続化集合を取り、マウント区間は二次クラッシュに従って部分集合を取り、その後のパブリッシュはすべて未永続化とする）。「比較対象は実装がリカバリした後のイメージである」の半分：レコード照合器は2つのイメージを受理するように改修され、クラッシュ状態イメージはルートが存在しレコードが存在するか否かを判定し（ルート選択とプレフィックス）、リカバリ後イメージはユニットが存在するかを判定し到達バージョンのインスタンス表を読み出す；第2段階で接続されるリカバリ後イメージはマウントとそのパブリッシュ後のプールであり、リカバリが自称するバージョンはマウントとそのパブリッシュが書き出した最新のルートを取得し、マウントが1つのルートも書き出さなかった場合は第1次クラッシュが到達したバージョンにフォールバックする。

**根拠**：
- 実験なし：**根拠は型であり、コストではない**：「バイト単位の比較」は二項関係 `R(イメージ₁, イメージ₂)` であり、**いかなる投資を行っても単項述語を二項関係に変換することはできない**；I-4.5 (廃止済み・2イメージ必要) は同一の基準によって廃止された。
- 「クラッシュ後イメージ」は2つである：D16 (パブリッシュセマンティクス) 確定項目 4 に伴い質問の焦点を絞り込んだラウンドの三者論証（資料 `_d16-items134-background.md`）においてユーザーが決定、変更履歴に記録。
- 永続化集合という入力パラメータ：ユーザー決定、原文は変更履歴にある；判定子が観測できない実証根拠は `research/prompts/m2-layer0-scale-r3-main-verification.md` 第2節のセクタ単位免除の行にある。
- クラッシュポイント再生は書き込み可能マウントをカバーせず、書き込み可能マウントと二次クラッシュはクラッシュ注入がカバーする：ユーザー決定（コードレビュー第2項、2026-09-27 ポップアップで「クラッシュ注入層で書き込み可能マウントを補完」を選択）、変更履歴に記録；クラッシュ注入側の実装は `m2-rev-b3b-implementer-report.md` にある。

**未済の検査**：なし。

#### 確定項目 8：独自開発の2点：Merkle ルートチェーン差分と世代間増分 scrub

**確定事項**：既存のツールには存在せず、本プロジェクトが独自に開発する2点：

**1. Merkle ルートチェーン差分 —— COW + トランザクションごとの新ルート発行に特有のてこ。**
履歴ルートは回収されるまで物理的に存在し、再生可能なチェーンを構成する。隣接する2世代のルートに対して再帰的に「部分ツリーポインタ + チェックサム / MAC」を比較し、
**ポインタが不変の箇所は直接スキップして掘り下げない**。計算量は2世代間で実際に変更された部分にのみ比例する。
これは従来の走査型 checker では検知できない種類の誤りを検出できる：**同期改ざん後に自己整合したインプレース書き込み** ——
ある書き込みが COW を迂回してインプレースでブロック内容を書き換え、同時にチェックサムを更新した場合、単一点で見ればそれは自己整合している（I-2.1 (チェックサムと内容の一致) は全緑）が、
ルートチェーン差分は矛盾を発見する：**COW を経ていない以上は新たな割り当てが存在するはずがなく、親ポインタは不変であるべきなのに、チェックサムが変化している**。
⚠️ **必ず陰性対照を設けること**：通常の COW コミット後に同一の検査を実行した際に完全に緑でなければならず、さもなければ通常更新のすべてを改ざんとみなしてしまう。

**2. 世代間増分 scrub。** I-1.2 (ブロックヘッダ書き込み順序パブリッシュ済み) および D5 (スナップショット / 空間会計メカニズム) の birth txg に依拠し、
「前回の scrub が走査した generation 水位」を維持し、`birth generation > 水位` のブロックのみを走査する ——
古い部分ツリーのチェックサムチェーンが変化していなければ、必然的に依然として自己整合している。**これにより、「毎コミット時に今回の変更をフル実行で検証する」ことが実時間上で実現可能となり、サンプリングが不要となる。**

**射程**：移動されたレプリカは `birth` が変化せず、永遠に水位の下に位置するため、世代間増分 scrub はそれらを検知できない。`crates/` 内にルートチェーン差分は存在せず（`grep -rn 根链 crates` でヒット 0）、世代間増分 scrub も存在しない（`grep -rn scrub crates` でヒット 0）。

**根拠**：
- 実験なし：2点とも COW のトランザクションごとの新ルート発行と birth txg の構造的性質に基づいて成立しており、いずれもまだ実装されておらず、測定可能な量はない。
- 三者論証：D13 (検証ルート) 新設時のラウンド（ローカル LLM / Sonnet / Opus が各1経路、立場重複なし）、資料は保存されていない、経過は変更履歴にある。

**未済の検査**：C389 (Merkle ルートチェーン差分のコード不在)；C194 (世代間増分 scrub が再配置されたレプリカを検知できない)；C408 (世代間増分 scrub のコード不在)。

#### 確定項目 9：クラッシュポイント再生は3層に分割する。さもなければ実時間が爆発する

**確定事項**：

| 層 | 範囲 | トリガー |
|---|---|---|
| スモーク層 | 固定シード、固定スクリプトの負荷と登録されたクラッシュ列挙ケース、書き込みリクエスト数で規模を制限しない（規模は射程参照）、正確なフル実行 | checker 層のフル実行（確定項目 15）：ユーザー要求時に実行；コミット時は既定でクイック層のみ実行 |
| 通常層 | 中規模負荷（書き込みリクエスト数千件以上）、正確なフル実行、列挙しきれない場合は正確な枝刈りに依拠し、直ちに実行できないノードには ignore を付与、カバレッジレポートおよびゲートで個別に「今回未実行」と列挙 | 毎コミット |
| 大負荷層 | 通常層の負荷 + 大規模負荷 + 複数デバイス階層の交差、N≥5 ラウンド、正確なフル実行 | ユーザー要求時 / リリース前 |

**枝刈りと ignore を導入した後に断言できること**：有効になっているノードがすべてのクラッシュ状態において通過したと断言できる；
**断言できないこと**：ignore されたノードが通過したとは断言できない。

**フル実行層はコミット時には既定で実行せず、時刻を制限せず、ユーザーが要求した際に実行する；コミット前 = クイック層が全緑（必要条件）。**
**マイルストーン完了時、クラッシュポイント再生または判定器を変更した一連のコミットの前には、メイン agent がポップアップでユーザーにフル実行層の実行を促す；失敗した場合やユーザーが実行しない場合でもコミット自体は受け入れ可能とし、修正または追試が完了するまで未結の赤または「この段階完了時にフル実行未実行」として記録する。**
「フル実行」は2点を指す：checker-tier-crash-point-replay `--full`（ゲート checker-tier-crash-point-replay、確定項目 15 のフル実行）およびクラッシュ増幅のフル実行（`scripts/crash/crash-amplification.sh` 引数なし）。
「クラッシュポイント再生または判定器を変更した」の判定はクラッシュ判定ライブラリに基づいて動的に計算する：今回の変更によりライブラリ内に判定ブロックを持つユニットキーが一致しなくなり、一致しないキーの中に判定器ユニット（判定器エントリ fn のユニットクロージャ内の関数ユニット）の変更が含まれている場合に該当とみなす；ステージは無効化されたブロック数と変更された判定器ユニット一覧を出力し、メイン agent はそれに基づいてポップアップを表示し、パス一覧に基づく判定は行わない。
「前回のフル実行」はメイン agent が現在のマイルストーン記録に1行で記述する：日付、2点それぞれの結論。
**フル実行層は「カバレッジの最大化」ではなく「失敗の再現性」を最優先で保証しなければならない** —— フル実行層のバグを1つ修正するための実時間 ≈ `log2(N) × フル実行1回の実時間` であり、再現不可能なフル実行層の失敗の価値はほぼゼロ、
あるいはマイナスであり、赤信号を無視する習慣を助長してしまう。

**射程**：スモーク層（ゲート checker-tier-crash-point-replay）のみが存在する；スモーク層は2層に分かれる：通常のクイック層ではセグメントごとのインプレース書き込みは任意の部分集合を取り、COW 書き込みは完全永続化または完全未永続化のみを取る（ユーザーが 2026-09-26 に決定、「甲二」と呼称、C561 (レコード照合器の再利用免除が再利用の半分のみ永続化された際に誤判定で赤となる) の修正後に使用）、これは COW 部分のみがディスクに永続化した際に初めて現れるクラスの誤りを検知できず、フル実行の代替にはならない；フル実行は中断からの再開をサポートし（ゲート checker-tier-crash-point-replay `--full`）、`.claude/gate.d/stage-inputs.tsv` に登録されたクラッシュ列挙ケースは並行線1のストリームで 12230590578 状態に達しており、もはや「書き込みリクエスト数2桁」のスモーク規模ではないため、コミット時には既定でクイック層のみを実行し、フル実行は確定項目 15 に従いユーザー要求時に実行する；通常層、大負荷層はゲート内にステージを持たない。各クラッシュ状態の定義方法は確定項目 4 に帰属する；ノードの正確な枝刈り方法、ignore ノードをいつ開放するかは確定項目 10、11 に帰属する。

**根拠**：
- 実験なし：階層化はゲートの実時間に関する設計制約であり、通常層、大負荷層の負荷と実時間はまだ存在せず、測定可能な量はない。
- 三者論証：D13 (検証ルート) 新設時のラウンド（ローカル LLM / Sonnet / Opus が各1経路、立場重複なし）、資料は保存されていない、経過は変更履歴にある。
- 通常のクイック層に甲二を使用：ユーザー決定、原文は変更履歴にある；クラッシュポイント再生の規模に関する3ラウンドの判定 `research/prompts/m2-layer0-scale-r3-main-verification.md`。
- ユーザー決定：「我々はサンプリングではなく、枝刈り + 初期 ignore である」（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 第3節「ユーザー決定：サンプリングせず、枝刈り＋初期 ignore」の行）、「列挙には ignore マークを設定し、徐々に開放できる」（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 第3節「ユーザー決定：ignore マークを徐々に開放」の行）、「1 列挙すべきものは設計し尽くす、全網羅で設計する」「2 列挙しきれないことこそが枝刈りの出番である」（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 第3節で資料担当がゲート doc-process-records の implementation-premise セルに阻止され、ユーザーが『全網羅で設計、列挙しきれない分は枝刈り、適切な粒度』と定めた行）；第1ラウンド判定 `research/prompts/m3-prune-gpu-r1-main-verification.md`「決定の振り返り」の節：確定項目 9 を破棄。
- ユーザー決定（2026-10-01）：フル実行層は時刻を制限せず、ユーザー要求時に随時実行、「夜間」を削除、原文は変更履歴にある。
- ユーザー決定（2026-10-01）：フル実行層が全緑であるべきタイミングは、マイルストーン完了時およびクラッシュポイント再生や判定器を変更した一連のコミットの前であり、メイン agent がポップアップでユーザーに実行を促す、原文は変更履歴にある；ユーザーに委ねるセルは三者判定 `research/prompts/test-script-align-r1-main-verification.md` 第2節 t1-3。
- 三者判定 `research/prompts/test-script-align-r2-main-verification.md` 第2節 U3（「U3 t1-3 の書き戻しは字面通りには実現不能」の行）：「フル実行」が指す2つの対象、「クラッシュポイント再生または判定器を変更した」のパス判定方法、「前回のフル実行」の記録方法、「受入条件」と「非同期」を1文に統合。

**未済の検査**：C409 (通常層・大負荷層がゲート内にステージを持たない)；C635 (前回のフル実行に機械可読なマーカーがない)。

#### 確定項目 10：真に正確に網羅すべき N は「レイアウトのセット数」ではなく「コミットプロトコル構造の同値類数」である

**確定事項**：通常層の正確な網羅は、まず問題を縮小してから枝刈りを議論する。クラッシュポイントの定義域は**トランザクション層ステートマシン**であり、`.claude/rules/fs-design.md` には2つのトップレベルルールがある：
「1つのトランザクション層をすべての構造が共用する」および「唯一多様化を許さないのがトランザクション層である」。したがってレイアウトが変更し得るのは2種類のみである：

| 差異 | 変更されるもの | 判定 |
|---|---|---|
| ノードサイズ、ストライプ幅 | 1ステップで書き込むバイト数、特定ステップの繰り返し回数 | **パラメータ**、ステップ種類の新規追加なし |
| 回転 vs SSD | seek コスト、IO スケジューリング | デバイス層に位置し、**トランザクション層プロトコルには位置しない** |
| **zoned（ZNS）** | **ルートポインタをインプレース上書きできるか否か** | **構造的**：追記のみ、ルート位置のドリフト、zone finish/reset の追加 |

**具現化手段は閉じた列挙 + 網羅的な match である**：`crates/singlefs-core/src/transaction.rs` の `CommitStep`、ステップ種類の登録場所は [layout/01-first-txn.md](../layout/01-first-txn.md) 8のセグメントシーケンス登録表である。
いかなるレイアウトであっても新たな構造的ステップを導入する場合、**この共有定義を変更しなければならない ⇒ コンパイラがすべての参照箇所で分岐の補完を強制する ⇒
この diff はゲートで grep 可能である**。これにより「今回新たな構造同値類が導入されたため、これに対して独立した大負荷層の再生を新たに開始する必要がある」と自動的にマークでき、
**人間が申告を覚えていることに依存しない**（diff によるゲート範囲の計算は C8 (ゲート適用範囲の判定不能) で未済）。さらに newtype による隔離（`RotationalPtr` / `ZonedPtr`）を追加し、
D12 (対象メディア) 確定項目 2 における「共用を禁ずる」バケット（会計事実の判定パス）をコンパイル時の制約とする。
⚠️ **これには第2の責務がある（D17 (実装階層とサードパーティパイプライン) で追加）：サードパーティパイプラインの接続ポイントである。**
プロトコルは本プロジェクトに帰属し、あるパイプラインが新たな構造的コミットステップを導入する場合、本リポジトリにこの diff を送信しなければならない。

**射程**：回転ディスクと SSD はコミットプロトコルのレベルで同型であり、D12 (対象メディア) の三者論証ラウンドがこれを担保する；zoned の行はそれが少なくとも1つの同値類を追加することを示すのみである。同値関係を記録ストリームのセグメントシーケンスの同型性に従って判定すること、および同値類が幾つ存在するかは D17 (実装階層とサードパーティパイプライン) 確定項目 2 に帰属する。種類文字列によってノードを識別することで十分か否かは、第1ラウンド判定で指摘された（保留、第2, 3ラウンドの攻撃待ち）：`research/prompts/m3-prune-gpu-r1-main-verification.md`「セルごとの判定」の節。
⚠️ **これが固定するのは「どのようなステップが存在するか」のみであり、「各ステップの失敗原子性の幅」を固定することはできない** ——
永続性メモリやオブジェクトストレージは各ステップの失敗原子性の幅を変更するが、列挙では対応できない（ギャップと補完策は D17 (実装階層とサードパーティパイプライン) 参照）。
`RotationalPtr` / `ZonedPtr` という newtype による隔離は `crates/` 内に存在しない（`grep -rn "RotationalPtr\|ZonedPtr" crates` でヒット 0）。

**根拠**：
- 実験なし：N が何であるかは `.claude/rules/fs-design.md` の2つのトップレベルルールから導出され、測定可能な量はない；同値関係をどう判定するか、幾つ存在するかに関する根拠は D17 (実装階層とサードパーティパイプライン) 確定項目 2 にある。
- 三者論証：通常層サンプリングのラウンド（「三者論証 + 実測検証」）、資料は保存されていない、経過は変更履歴にある。

**未済の検査**：C8 (ゲート適用範囲の判定不能)。

#### 確定項目 11：ignore ノードをいつ開放するか：未済の検査によるローテーション台帳を否決し、3条を残す

**確定事項**：

**「未済の検査によるローテーション台帳」スケジューリング：否決。** スロット = `(レイアウト, サブフェーズバケット)` とし、未済の検査の降順でキューの先頭を取得し、任意のスロットが高々 `K_max = ⌈N×M/C⌉` 回のコミットで必ずカバーされると主張したスケジューリングは否決された：それが保証できないものこそがまさに防ぐべき対象 —— 低速で稀な分岐 —— であったためである（実測検証および他の4つの否決理由は変更履歴にある）。

**残された3条**：

1. **方向性の正しさ**：確率的な論証ではなく、**決定論的であり、ゲートで赤と判定可能なカバレッジ記録**を求める ——
   長期的な保守において、確率的な保証は保証が存在しないことに等しい。

<!-- doc-lint:registry name-col=2 -->

| 記号 | 略称 | 意味 |
|---|---|---|
| T1 | 通常層 実時間予算 | 通常層のローカル実時間予算、出力ではなく入力 |

2. **`T1` (通常層 実時間予算) は入力であり、出力ではない**（`.claude/rules/fs-design.md`：ゲートの実行時間自体が設計制約である）。
   正しい順序は「まず予算を宣言する → それを用いて1回のコミットで開放可能な ignore ノード数を制約する」であり、「所要時間を測定する → 開放可能な数を導出する」ではない。
   **この条項はいかなる測定も必要とせず、今すぐ確定すべきである。**
3. **判別力はカバレッジと分離して検査しなければならない**：各 ignore ノードが開放されたコミットにおいて、「既知の壊れたイメージ」に対してそのノード自身の checker を追加で実行し、
   エラーを報告しなければ赤と判定する —— これは D13 (検証ルート) に既存の陽性対照規律の直接の再利用である。C13 (checker 判定の無効化) 参照。

ゲート内には「通常層 クラッシュポイント再生」というステージが存在せず、`T1` (通常層 実時間予算) / `C` / スロットごとの所要時間は測定できない。
⚠️ **したがって `C`、`K_max`、`T1` (通常層 実時間予算) の3つの数値はいずれも現時点で kb に書き込むことを禁ずる。**

**射程**：ignore ノードをいつ開放するかのみを管轄する；スモーク層の全数列挙には `T1` (通常層 実時間予算) は不要である。第2条の「今すぐ確定すべきである」と末尾の「現時点で kb に書き込むことを禁ずる」は矛盾しており、[verification-build.md](../verification-build.md)「未決確定事項」第6問に記録されているが未判定である。ノード登録表には状態フィールド（開放 / ignore / 実装待ち）を持たせ、さらに3条を追加する —— 各 ignore 行は変更対象となるオブジェクトを1つ指名し、そのオブジェクトが緑になったにもかかわらずノードが依然として ignore である場合に赤と判定、カバレッジレポートに ignore の状態数と割合を列挙：これら3条はゼロラウンド攻撃されており（`research/prompts/m3-prune-gpu-r1-main-verification.md`「セルごとの判定」の節）、確定事項とはせず第2, 3ラウンドの攻撃を待つ。

**根拠**：
- 実験なし：台帳の否決はスケジューリング規則のシミュレーション検証（200ラウンド、後半の定常状態最大値を取得、厳密なプレフィックス貪欲法、3行の数値は変更履歴にある）に基づき、実験としては登録されていない；残された3条はゲートの設計原則であり、測定可能な量はない。
- 三者論証：通常層サンプリングのラウンド（「三者論証 + 実測検証」）、資料は保存されていない、経過は変更履歴にある。
- ユーザー決定：「列挙には ignore マークを設定し、徐々に開放できる」（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 第3節「ユーザー決定：ignore マークを徐々に開放」の行）；第1ラウンド判定 `research/prompts/m3-prune-gpu-r1-main-verification.md`「決定の振り返り」の節：確定項目 11 を破棄、「セルごとの判定」の節でノード登録表に状態フィールドを持たせる形態と3条の攻撃待ち補足条件を提示。

**未済の検査**：C8 (ゲート適用範囲の判定不能)；C13 (checker 判定の無効化)。

#### 確定項目 12：衝突 1：checker 即仕様 と 検証の2経路間でツール共有を禁止する の衝突

**確定事項**：**衝突 1：`checker 即仕様` と `検証の2つの経路間でツール共有を禁止する` が直接衝突する。**
使用中の不変条件（件数の公式な登録場所は [invariants/README.md](../../../crates/singlefs-checker/invariants/README.md) のファイル先頭にある）が checker のソースコードであり形式検証の spec でもある場合、単一障害点となってしまう。
**実行可能な妥協点であり、本プロジェクトがすでに投資済みの解決策**：checker は2つの方向を実装する —— **トラバーサル型**（システム構成から順に辿る）
と**スキャン型**（ディスク全体を走査し、「メタデータブロックは自己記述的でなければならない」というブロックヘッダに基づいて再構築する）。双方は同一の不変条件に対して独立に判定を行い、
共有するのはフォーマット解析のみとする。

**射程**：checker 内部の2つの方向の間で何を共有するかを管轄する；checker と実装の間で何を共有するかは確定項目 5 に帰属する。プールレベル checker は双方の方向を備えている（`crates/singlefs-checker/src/walk.rs` のファイル先頭）。

**根拠**：
- 実験なし：これは「checker 即仕様」と検証経路の独立性の間のトレードオフであり、測定可能な量はない。
- 三者論証：D13 (検証ルート) 新設時のラウンド（ローカル LLM / Sonnet / Opus が各1経路、立場重複なし）、資料は保存されていない、経過は変更履歴にある。

**未済の検査**：C410 (checker の2方向での共有事項に関する検査不在)：2つの方向が「共有するのはフォーマット解析のみ」という条項に対して、赤になり得る検査が存在しない（`.claude/gate.d/` 内にこれを判定するステージがない）。

#### 確定項目 13：衝突 2：crash refinement の spec とモデル突き合わせの model

**確定事項**：**衝突 2：crash refinement の spec とモデル突き合わせの model は同一のものの2通りの記述である。**
1つに統合する → 工数を半減できるが、**独立性の要件に直接違反する**；各自で記述する → 名目上は独立だが、
通常は同一人物が同一週に書くことになる。緩和策（相関を低減するのみで排除はしない）：両者を**表現形式のレベルから異ならせる** ——
spec は「イメージに対する状態述語」として記述し、model は「op → 状態遷移」として記述する。不一致が生じた際に少なくとも相互の位置特定が可能となる。
**同源性は観測されなければならない**：「spec と model が不一致」となったイベント数および各判定でどちらが誤っていたかを統計する；
不一致が累計 ≥ 5 回で「spec の誤り」と判定された回数が 0 回であった場合、両者が実質的に同源であることを示しており、2本の経路を統合して予算を全数探索へ移管すべきである。
判定は3つの枠に分かれる：spec の誤り / model の誤り / 条項の記述不足（双方が条項のそれぞれ異なる解釈に従っており、条項にどちらが正しいかが書かれていない）。不一致が発生した場合、まず前提が成立するドメインで再判定し、依然として存在する不一致のみを記録する；「条項の記述不足」イベントはまず未済として記録し、「累計 ≥ 5 回」にも「spec の誤り」にも算入しない。条項に該当する記述が補完された後、補完された文に従って再度判定し、先の2つの枠のいずれかに帰属させる。
ここでの model は O1 (インメモリモデル突き合わせ (HashMap 理想ファイルシステム)) を指す。O3 (独立規約実行器) が同一の (履歴, クラッシュポイント) に対して判定行を出力できるようになるまで（Python 製実行器の第1版 E170 (O3仕様実行器第1版) は存在するがクラッシュポイント再生に未接続、C447 (規約実行器がクラッシュポイント再生および RefFS 比較に未接続)）、O3 (独立規約実行器) × RefFS のペアは空欄として登録し、イベントとしてカウントしない；O1 (インメモリモデル突き合わせ (HashMap 理想ファイルシステム)) のクラッシュ判定官を身代わりとしない；未確立のペアは「未確立」と報告し、緑とは報告せず、いかなる枠にも算入しない。

**射程**：緩和策は相関を低減するのみで排除はしない。確定項目 1 で撤回された旧候補はこの条項の影響を受けない、理由は確定項目 1 の射程にある。crash refinement のコードは `crates/singlefs-replay-prefix/src/lib.rs` 内の Verus コアとその spec（プレフィックス取得、ルート選択、リングスキャンの3区間と、ルート選択およびプレフィックス取得の結合判定の系であり、確定項目 1 の M1、M2 構文層および M3 の両解釈と単調性の系をカバーする；M2 セマンティクス層および M4 は E165 (リカバリ再生の有界全数探索とミューテーション殺傷限界) に依拠して有界内で判定する）；spec は具体的な状態に対して評価を行うことができず、「spec と model の不一致」イベントがどこから生じるかは未定であり、カウンタにはコードが存在しない（C191 (同源性カウンタが1ペアのみ対象)）。

**根拠**：
- 実験なし：同源性は将来の spec と model の不一致イベントのカウントによって観測されるべきであるが、spec もカウンタもまだ存在せず、測定可能な量はない。
- 三者論証：D13 (検証ルート) 新設時のラウンド（ローカル LLM / Sonnet / Opus が各1経路、立場重複なし）、資料は保存されていない、経過は変更履歴にある。
- 第3の枠「条項の記述不足」：三者判定 `research/prompts/m3-formal-r1-main-verification.md` 第2節「F2 同源性」の行（攻撃側が指摘、推論）、`research/prompts/m3-formal-r2-main-verification.md` 第2節第9行、`research/prompts/m3-formal-r3-main-verification.md` 第2節同源性カウンタの行、3ラウンド耐えた；実験なし：カウンタには依然としてイベントソースがない。
- 「model は O1 (インメモリモデル突き合わせ (HashMap 理想ファイルシステム)) を指す」および未確立のペアを空欄として登録：三者判定 `research/prompts/m3-formal-r5-main-verification.md` 第2節同源性カウンタのセル（両経路が同一方向を向いた部分）；`research/prompts/m3-formal-r4-main-verification.md`「ユーザー決定」内の身代わりペアの行（身代わりペアとはみなさない）。

**未済の検査**：C191 (同源性カウンタが1ペアのみ対象)。

#### 確定項目 14：他の決定との連鎖

**確定事項**：

| 決定 | 連鎖点 |
|---|---|
| D4 (チェックサムの位置) | ルートチェーン差分が Merkle を「支払うべき代償」から検証のてこへと反転させる；ただし形式検証においてハッシュは uninterpreted function であり、**衝突耐性は検証範囲の完全に外側にあり、「Merkle を検証した」と解釈してはならない** |
| D8 (コアインデックス構造) | ログ構造ノード、write buffer、key cache の3層はいずれもシンボリック実行を拘束し（ループ上限、遅延ウィンドウ、キャッシュ一貫性）、「Yggdrasil は1分以内」という計測条件を本プロジェクトにおいて成立させない |
| D9 (暗号化) | AEAD は SMT で解決不能であり、uninterpreted function としてのみモデル化できる；**「nonce の再利用による平文漏洩」という結果は検証できず**、残るのは「nonce が重複しない」という純粋な組合せ的性質のみである —— そしてこの純粋な組合せ的性質こそが、この検証経路の最も価値ある成果物である |
| D11 (インデックスノードにメッセージバッファを残すか) | 確定項目 1 で「残す」を選択；D8 (コアインデックス構造) 確定項目 1 が冪等完全値を選択した後は、checker の I-3.1 (割り当て済み統計の整合) はメッセージセマンティクスを解析する必要がなくなり（D11 (インデックスノードにメッセージバッファを残すか) はこれに基づきコスト 1 が本プロジェクトで成立しないと判定）、衝突 1 がこれによって「解決不能」になることはない |

**射程**：連鎖するのは形式検証と checker がこれら幾つかの決定に直面した際の境界であり、それらの決定の条項自体を変更するものではない；ルートチェーン差分および衝突 1 自体は確定項目 8、12 に帰属する。

**根拠**：
- 実験なし：連鎖はそれら幾つかの決定の確定条項から演繹された境界であり、測定可能な量はない。
- 三者論証：D13 (検証ルート) 新設時のラウンド（ローカル LLM / Sonnet / Opus が各1経路、立場重複なし）、資料は保存されていない、経過は変更履歴にある。

**未済の検査**：なし。

#### 確定項目 15：検証コードを2層に分割 —— harness 層は常時実行、checker 層は既定でコミット時のみ実行

**確定事項**：`crates/` 以下の検証コードは「いつ実行するか」に基づいて2層に分割し、各々1つのパッケージに配置する。プールレベル checker と合わせて計3つの名称となる：**harness 層**は `crates/singlefs-harness` であり、単体テストおよび結合テストに加えてそれらが使用するスキャフォールドライブラリ（レコーダー、インメモリプール、理想モデル、ランダム履歴、故障注入デバイス、シナリオ）を含み、コード変更時に随時実行する；パッケージ内はさらに軽量と重量に分かれる：debug ビルドで単一テストの実行が 60 秒以上のものは重量とし、`#[ignore]` を付与する。単一テストの所要時間は通常の実行時に測定されたものを参照し、別途個別に測定はしない。実行したいときは随時実行可能であり、一律にメモリラッパー経由とする；1テストケース1シナリオとし、テストファイル名はテスト対象に基づいて命名しマイルストーン名には基づかない。**checker 層**は `crates/singlefs-checker-tier` であり、クラッシュ状態列挙エンジンとレコード照合器、クラッシュ列挙ケースの判定ライブラリ経由の2段階とそのカウント、検出ログとツールチェーン識別情報、1つの長期判定ライブラリ（ブロックキーは内容のみによって決定され、各種テストが各々どのブロックを必要とするかはインデックスとしてのみ機能）と2段階プロトコル、クラッシュ注入と破損ディスク入力の2つの戦役、実デバイス側でのデバイスログ比較、実験および実デバイスのハーネスバイナリ、ならびにそれらの全テストケースを含み、QEMU（ゲート checker-tier-qemu-device-streams）、herd7（checker-tier-lkmm）、crates ミューテーション全表（checker-tier-crates-mutation-replay）、全実験再実行（checker-tier-research-build-and-replay の experiment-replay セル）を伴う：既定ではコミット時のみ実行し、単独での実行も可能（コマンドに `SINGLEFS_HEAVY_TESTS=user-request` を付与）。checker 層内のクラッシュポイント再生（ゲート checker-tier-crash-point-replay）は2層に分かれ、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-research-build-and-replay は分割しない：**クイック層**は2回実行する：`cargo test --release -p singlefs-checker-tier --lib --tests`（`--ignored` なし、`--tests` はすべての `test = true` のターゲットを含み、ハーネスバイナリのインライン単体テストもこれに含まれる）、およびもう1回は `tests/` 内の `cfg` で指定されたフィーチャを有効化して実行（`--features <フィーチャ> --lib --test <フィーチャ付きテストファイル>…`、フィーチャに `gpu` を含む実行は `scripts/crash/gpu-quiesce-run.sh` 経由で起動）（ゲート checker-tier-crash-point-replay はゲート全体の実行時およびコミット時の双方でこの2回を実行し、2回目はその feature-gated-tests セルである）、**フル実行**は `.claude/gate.d/stage-inputs.tsv` にキー `crash-case:` として登録されたテストケースを1件ずつ `--include-ignored --exact` で実行（checker-tier-crash-point-replay `--full`）、コミット時には既定で実行せずユーザー要求時に実行し、実行方法は確定項目 9 の階層化に従う；クラッシュ増幅判定器はシステム全体が GPU に移行しており、すべてのプールレベル checker 不変条件をカバーする（`crates/singlefs-checker-tier/src/crash_judge_gpu.rs`）。**プールレベル checker** は `crates/singlefs-checker` であり、判定器ライブラリとして機能し、上記2層から呼び出される。依存関係は一方向のみ許容される：checker 層は harness 層とプールレベル checker に依存し、harness 層は checker 層に依存しない（ゲート checker-independence-and-sync の checker-implementation-disjoint セル第 ④ 項が判定、dev-dependencies も含む）；双方の層が必要とする短いコードは各々個別に保持することを厭わない。クラッシュ列挙ケースは一律に `crates/singlefs-checker-tier/tests/` 内に記述する（`scripts/crash/crash-case-check.py` が判定）。ヘビー層テストの関門はパッケージ単位で判定する：`singlefs-checker-tier` に到達するテストが checker 層である。

**射程**：`crates/` 以下のコードとテストがどのパッケージに属するか、いつ実行するか、ゲートが何に基づいて識別するか、3つのパッケージをどう呼称するか（`.claude/rules/verification.md`「定義と名前」）を管轄する；各不変条件が何を判定するか（[invariants/README.md](../../../crates/singlefs-checker/invariants/README.md)）、列挙ドメインをどう定義するか（確定項目 4）、プールレベル checker と実装の間で何を共有するか（確定項目 5：プールレベル checker は依然として `singlefs-format` にのみ依存する）は関知しない。アップストリームの SOP には本プロジェクトの検証手段をこれ以上記載しない：クラッシュポイント再生、モデル突き合わせ、checker 即仕様、ファイルシステム特有の逆推論ギャップの4節および `gate.sh` の未実装リスト内のプロジェクトキーはすべて本リポジトリに集約された（`.claude/rules/verification.md`、`.claude/gate-not-implemented.tsv`）。アップストリーム側の変更はレプリカに同期済みであり、未リリースである。分割ラウンドの記録は `records/2026-09-27-验证两档拆分.md` にある。

**根拠**：
- 実験なし：いつ実行するか、どのパッケージに属するかはプロセスポリシーであり、測定可能な量はない；2層それぞれの判別力はそれぞれのテストケースおよび変異表によって証明される。
- ユーザー決定（2026-09-27）、原文は変更履歴にある；三者 `verification-split-r1`（判定 `research/prompts/verification-split-r1-main-verification.md`）は `singlefs-checker-tier` 分割前のバージョンを攻撃したものであり、パッケージ分割後の変更はゼロラウンド攻撃されている。
- ユーザー決定（2026-10-01）：フル実行は時刻を制限せず、ユーザー要求時に随時実行、「夜間」を削除、原文は変更履歴にある。

**未済の検査**：なし。

#### 確定項目 16：セグメント単位で登録されたセグメントの全領域が u64 に収まらなければスライス判定し、状態数には 1280 ビット固定幅整数を用いる

**確定事項**：
- スライス分割：クラッシュ増幅においてセグメント単位で登録されたセグメント（`.claude/gate.d/crash-point-replay-segments.tsv`、1行1セグメント5列：項目名、ストリーム名、セグメント番号、セグメント書き込み数、セグメント書き込み表ダイジェスト）について、全領域の状態数が u64 に収まらない場合は固定サイズのスライスごとに順次判定する；スライスサイズは環境変数 `SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES` で定義され、未設定時は 2^32 状態とする；スライスの起点は判定ライブラリ内の再開点とする（再開点はセグメント書き込み表ダイジェストと判定器ユニットキーによってアドレス指定され、項目名、ストリーム名、セグメント番号は表示用に過ぎない）。全領域が u64 に収まる登録セグメントはスライス分割せず、全領域を一度に判定する。コード配置先：`crates/singlefs-checker-tier/src/crash_amplification.rs:14800` `REGISTERED_SEGMENT_SLICE_STATES_ENVIRONMENT_VARIABLE`、`crates/singlefs-checker-tier/src/crash_amplification.rs:14947` `fn segment_slices_of_the_recording_from_the_environment_and_the_library`。
- ビット幅：登録セグメントの総状態数、スライス起点、スライス末尾、および進捗には一律に 1280 ビット（10 個の u128）固定幅符号なし整数 `StateCount1280Bits`（`crates/singlefs-checker-tier/src/crash.rs:1640`、手足の数は同ファイルの `STATE_COUNT_LIMBS`）を用い、演算はすべて checked とし、飽和や切り捨ては行わない；ドライバ行 `registered_segment_progress=` の判定済み状態数および総状態数はすべて正確な10進数で記述し、指数の形式では記述しない。
- 登録表をエンジンに渡す方法：エンジンは環境変数 `SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY` 経由でのみセグメント登録表を読み込み（`crates/singlefs-checker-tier/src/crash_amplification.rs:14764`、`fn registered_segments_of_the_flow_from_the_environment` 内）、未設定時は1セグメントも登録されていないものとして扱い、他の逃げ道は設けない。144 ユニットのファイル順次書き込みが 145 個となり extent 下段が2層に成長したストリーム（エンジンストリーム名 `position_addressed_ExtentLowerSegmentGrowsToTwoLevels`）のユニット書き込みセグメント（322 回のユニット書き込み）およびレコードセグメント（290 回の journal レコードインプレース書き込み）の双方が登録されている。

**射程**：クラッシュ増幅エンジンがセグメント登録されたセグメントをどうスライス分割するか、状態数をどの整数で記録するか、登録表をどこから読み込むかを管轄する；どのセグメントを登録すべきか（ユーザーが承認し登録表のファイル先頭に記述）、登録セグメントのドメイン自体をどう展開するか（判定 `research/prompts/m3-crash-enumeration-merge-r3-main-verification.md` の J10）、各クラッシュ状態をどう定義するか（確定項目 4）は関知しない。未登録のセグメントの状態数、およびストリーム全体を合算した状態数は依然として u64 に収まる必要があり、収まらない場合はライブラリ書き込み前に実行を拒絶する；登録セグメントの全領域が 1280 ビットを超えた場合も同様にライブラリ書き込み前に実行を拒絶する。スライスサイズは 1 から 2^32 までのみ許容される：2つの GPU カーネルがスライス内インデックスを u32 としてスライス起点に加算するためである（`crates/singlefs-checker-tier/src/crash_amplification.rs:14807` `LARGEST_REGISTERED_SEGMENT_SLICE_STATES`）；ライト層（`scripts/crash/crash-point-replay.sh --quick`）の呼び出し元がスライスサイズを設定していない場合は `LIGHT_REGISTERED_SEGMENT_SLICE_STATES` に設定する。展開上限でカバーできず未登録のセグメントがある場合、増幅エントリ `scripts/crash/crash-amplification.sh` は赤と判定する；クラッシュポイント再生エントリ、増幅エントリ、および項目ドライバは実行開始前に本テーブルを照合した上でエンジンに渡す。

**根拠**：
- E161 (クラッシュ増幅の重複排除とセグメント分割所要時間)「第2回再実行：サンプリングドメインを11セグメントのストリームに従って再定義（2026-10-05）」の節：セグメントのフル実行状態数と閉じた式、ハーネスによる個別の数え上げ、crates の計画がセグメントごとに等しく、状態数はセグメント書き込み数に伴い指数関数的に増加するが、これは「長いセグメントの全領域は u64 に収まらない」という前提を支持するに過ぎない；スライスサイズ、ビット幅の値、登録表の読み出し方法はエンジニアリングパラメータでありユーザーが決定したものであって、本実験が支持するものではない。
- ユーザー決定、原文は変更履歴にある：スライス分割と1スライスあたり 2^32 状態（`records/2026-10-11-崩溃放量两段与脚本审计.md` 96行目）、ビット幅 1280 ビット（同記録 98, 99行目）、両セグメントの登録（同記録 92行目）。
- スライス分割とビット幅は判定 `research/prompts/m3-crash-enumeration-merge-r3-main-verification.md` の J10（セグメントごとの登録）に対する補足であり、三者による攻撃は経ていない。

**未済の検査**：なし。

#### 確定項目 17：判定再利用のユニットは関数であり、キーは関数ユニット、ツールチェーン、依存バージョンのみで構成する

**確定事項**：
- 重型ゲート、クラッシュ判定ライブラリ、および実験受入における各判定（クラッシュ状態ブロック、クイック層およびモデル突き合わせの各テスト関数、crates ミューテーションの各行、広域ミューテーションの各行、形式検証の各条、QEMU の各階層、litmus の各項目、研究再実行の各行、実験ミューテーション表の各条、実験受入）を踏襲するか否かは、そのユニットキーのみを参照する：それが依存する関数ユニットの各コメント除去後指紋、クロージャ内の各ユニット（mod ヘッダを含む）の `#[cfg]`、`#[cfg_attr]`、`cfg!` が実際に参照する cfg アトムに今回のターゲット下での真理値を付加したもの（クロージャが参照しないアトムはキーに含まれない：`gpu` フィーチャの有無でコンパイルされた2回の実行において、クロージャが `gpu` に言及しない判定キーは同一となり、ライブラリを共有する）、ツールチェーン（cargo 解決後の有効ビルド環境および外部ツールバージョンを含む）、`Cargo.lock` から解決されたクロージャが実際に参照する依存バージョン、ならびにその判定自体の内容入力（セグメント書き込み表と開始イメージのダイジェスト、litmus の内容、ミューテーション原文と置換文など）を加える。関数ユニットはその判定のエントリから呼び出しと参照を辿って収集し、参照される定数、型、静的項目、マクロ展開を合わせて計算する；wgsl は fn 単位で区切る。
- 一律にキーに含めず、再利用の根拠とすることも禁ずるもの：コミット、HEAD、ツリーに基づく変更範囲；ファイル全体の原文；ファイル単位、パッケージ単位、テーブル全体の指紋またはクロージャ；手書きのバージョン定数；パス（リポジトリルートも含む）、名前、シード文字列、行番号、リソース上限、タイムアウト；全工程またはセル全体のマークのみで判定ごとのライブラリを持たないもの。キーが狭すぎる（参照される定数、型、マクロの収集漏れ）、ライブラリが失われやすい場所に置かれている、改名によりライブラリが消失する、2つのライブラリで同一バッチを重複判定することも同様に違反とみなす。
- 完全に収集できないもの（動的ディスパッチ、マクロ、`include_str!(concat!…)` などで認識不能なもの）はその判定のキー計算を拒否してそのまま実行し、暗黙のうちに収集を省略することはしない；各キーはそれを構成するユニット一覧とともにディスクに保存され、無効化された際にはどのユニットが変更されたかを報告できるようにする。
- 判定の種類ごとに1つのライブラリを永続ディレクトリに固定し、1回の実行ごとや判定手法ごとにディレクトリを変更しない；リポジトリルートもライブラリの入力ではない：クラッシュ判定ライブラリのディレクトリが環境変数 `SINGLEFS_CRASH_AMPLIFICATION_LIBRARY` で指定される際は絶対パスのみを受理し、相対パスは拒絶する；フル実行スイッチは書き込みのみで読み出さず、オンデマンドで判定されたものはキーに従って同一のライブラリに書き戻す；二台目で判定された結果はローカルのものにマージされる。
- キーの変更により古いライブラリが無効化されることはコストとはみなさない：互換マイグレーションは行わず、古い行への追記署名も行わず、変更後は再実行する。

**射程**：重型ゲート、クラッシュ判定ライブラリ、および実験受入の再利用キーを管轄する；静的ゲートの「新規追加行のみ判定」の変更範囲（`scripts/gate/changed-paths.sh`、これは判定基準のセマンティクスである）や preflight の `admission: inputs-changed` は関知しない；列挙ドメイン（確定項目 4）、階層化、およびいつユーザーにフル実行を促すか（確定項目 9）は関知しない。イメージ判定および特定状態判定のバージョンは、判定定義セット（CPU 参照判定器 `crash::judge_crash_image`、`crash_given_states::judge_the_given_states_on_the_cpu` のクロージャ）のキーであり、そのブロックが CPU 上で判定されたか GPU 上で判定されたかを問わない：GPU 判定器は加速経路であり、その受入は定義セットとの突き合わせ（クイック層の GPU と CPU の突き合わせテストケース）に依拠し、そのユニットはブロック属性に出典としてのみ入り、キーには入らず、種別も分けない；GPU カードのドライバ、カード型、複数枚のカードの分担もブロック属性や実行時パラメータとして残し、キーには含めない。

**根拠**：
- ユーザー決定（2026-10-15）、原文は変更履歴および `records/2026-10-13-复用键与全仓改名.md`「2026-10-15 ユーザー決定：コミットは再実行と無関係、再実行はコード変更のみに従う」の節、「2026-10-15 バッチ3」の各段にある。
- ユーザー決定（2026-10-17）：判定はハードウェアや CPU / GPU とバインドせず、再利用と枝刈りはデバイスを参照しない；原文は変更履歴および `records/2026-10-13-复用键与全仓改名.md` 2026-10-17 のフル実行停止に関する各段にある。
- メイン agent 決定（2026-10-15）：リポジトリルートはキーやライブラリに含めず、ライブラリディレクトリは絶対パスのみを受理、cfg 成分はクロージャが実際に参照するアトムのみを取得、出所は同記録「2026-10-15 検証フェーズ第1ステップ赤判定、2条の決定、4-3 4バッチ派遣」の決定第1, 2項。
- 反証：E164 (クラッシュ増幅新設計の小規模機能テスト) F4 において、関数のみに基づいて指紋を取得し参照される定数を収集しなかった結果、V5 と V11（V11 は定数 `DATA_POINTER_BYTES` のみを変更）の判定を漏洩したことが測定された：関数ユニットは参照される定数、型、静的項目、マクロとともに計算しなければならない。
- ディスク上の数値：`research/prompts/pruning-survey-2026-10-15/ledger.md`「1の2、ディスク上で測定されたもの」の節 —— 名前のみが異なるブロックが個別に保存・判定されたものが 4100 ブロックおよび 9679 ブロック；ライブラリ全体で一括の判定法ダイジェストを変更した際に累積で 3 935 242 934 状態の追加判定が発生したが、そのクロージャ内で変更された 29 ファイルのうち 24 ファイルはコメント除去後に逐語的に同一であった：キーに名前を含めたり、ファイル単位やバージョン全体で指紋を取得すると、同一の判定が重複して再実行されてしまう。
- 経路状況記録なし：三者による攻撃は経ていない；ユーザーはコード改修後に3ラウンドの攻撃側を開始し、「枝刈りがアトミックであるか、キーが関数ユニットのみで構成されているか」のみを攻撃対象と決定した。

**未済の検査**：C8 (ゲート適用範囲の判定不能)、C489 (singlefs-core の広域ミューテーションが今日のコードに対して結論を持たない)、C528 (2つのゲート経路で判定される変更ウィンドウが異なる)、C564 (残る6本のクラッシュポイント再生フル実行が誰も走らせていない)、C605 (クラッシュ増幅の再利用と枝刈りで無効化して再判定する対照がない)、C630 (コミット時の形式検証ステップ1で停止する頻度が未計測)、C635 (前回のフル実行に機械可読なマーカーがない)、C645 (クラッシュ増幅フル実行を誰もディスパッチできない)、C668 (ランダム履歴スキャフォールドの導入元ホワイトリストがエイリアス記法で迂回可能)、C777 (5つの実験キーが stage-inputs.tsv に登録行を持たない)、C836 (staged-green に紐づくツリーが判定されたツリーとは限らない)。

```

**出处 `.claude/kb/decisions/13-検証ルート.ja.md:445-448`（整段抄，未转述）**

```markdown
## 履歴バージョン

D13 (検証ルート) の履歴エントリは [decisions-history.md](../decisions-history.md) に集約されている。

```

**出处 `.claude/kb/decisions/13-验证路线.md:1-6`（整段抄，未转述）**

```markdown
## D13 验证路线 —— 已定

**中文** · [English](13-verification-route.en.md) · [日本語](13-検証ルート.ja.md)

D13（验证路线） 管本工程拿什么验正确性：三类现成办法全做——自写参照实现做差分对拍、有界穷举崩溃测试、crash refinement 形式验证（调查见 [prior-art.md](../prior-art.md) 7.4）——并且不照抄，针对本工程特有的四样（COW 每事务发新根、Merkle 自验证、整卷 AEAD 的 nonce 唯一性、没有参照实现）自己设计验证手段。它定 oracle 怎么分、自研哪两样、崩溃点重放怎么分层与抽样、崩溃状态集合怎么定义、checker 与实现共享什么。不管的：每条不变量写什么在 [invariants/README.md](../../../crates/singlefs-checker/invariants/README.md)；三样验证手段怎么落地、谁挡着谁在 [verification-build.md](../verification-build.md)；提交步骤登记在哪、结构等价类怎么分、有几个，归 D17（实现分层与第三方管道） 已定项 2。

```

**出处 `.claude/kb/decisions/13-验证路线.md:29-88`（整段抄，未转述）**

```markdown
#### 已定项 1：crash refinement 先验「一次发布 + 一次崩溃 + 一次恢复」

**定案**：crash refinement 的第一个验证对象是恢复函数 `Recover` 的关系性质，最小封闭单元是「一次发布 + 一次崩溃 + 一次恢复」。四个里程碑的主语 `Recover` 指 `crates/singlefs-core` 里以 `verify_named_units = true` 调 `replay_journal` 的三处（`recover` 的 `Consult` 策略、可写挂载读阶段、只读挂载），三处的 `records` 都取 `scan_journal` 的输出；`Ignore` 与 `ConsultWithoutNamedVerification` 是测试与对照用的臂，写进可信部分；`crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs` 不是产品调用方。四个里程碑依次是：

| 里程碑 | 义务 | 落点 |
|---|---|---|
| **M1 全性** | `Recover` 在 D13（验证路线） 已定项 4 枚举的**每一个**崩溃状态上有定义、终止、输出一个能过 I-7.1（根槽有效集非空） 与 I-8.3（重放前缀严格连续） 的镜像 | 脚手架。**完成判据不是「证出来了」，是「M2 能在它上面写出来」**——崩溃点重放本就全量枚举（[verification-build.md](../verification-build.md)「崩溃点重放不抽样、全量枚举」），恢复卡住当场暴露，单独证它没有边际价值。取前缀、择根、扫环三截已进 Verus，都在 `crates/singlefs-replay-prefix/src/lib.rs`：取前缀核 `replay_core`（终止与不溢出在核内证到，循环都带 `decreases`；`requires` 两条——记录表按 (实例代号, 计数器) 升序、计数器有后继——由胶水 `replay_journal` 里的断言兜），择根核 `choose_root_slot`（自证过的槽里取 (`checkpoint_txg`, 实例代号) 最大、键相同取槽序最前），扫环核 `first_copy_of_each_record_key`（同一 (实例代号, 计数器) 先到先得），择根与取前缀的合判写成推论 `corollary_recovery_replays_from_the_newest_self_verified_root`；门禁 checker-tier-formal-verification 判证明，登记数 56 verified（`.claude/gate.d/verus-pins.tsv`）。E165（恢复重放有界穷举与变异杀伤界） 当轻量工具，它第 3 次重跑第二段的择根合判族在内存盘上调真 `choose_root`、`scan_journal`、`replay_journal`。择根序的四个改坏变体在合法历史 K 族里只有「取第一个有效者」红，另三种（比较不看实例代号、txg 平局取低、实例代号为主）由 Verus 必红世界（`crates/singlefs-replay-prefix/verus-must-red-worlds.tsv` 择根那一节）与对抗根环 KA 罩（第 3 次重跑第三段）。核之外：择系统配置（`choose_system_configuration`）与两截胶水（择根的读槽、自证校验和、fsid、区域到盘的映射、槽的遍历次序；扫环的偏移枚举、提示、解析与 CRC、盘的遍历次序）不在 Verus 里，崩溃点重放与崩溃放量的 CPU 判器在每个崩溃状态上调 `recover` 罩它们（GPU 判器用 WGSL 另写一份 `replay_journal`，验不到 core） |
| **M2 前缀不切发布** | **任一 `jsn` 连续前缀，其重放结果等于某个整数个已发布的 checkpoint 的结果——不存在被前缀切开的发布；前缀停在一次发布中间时，那次发布整体不施加**（D16（发布语义） 已定项 4「一次发布整体施加或整体不施加」）。分两层。**句法层**（对任意自证过的记录表，无前提）：施加的记录是若干整次发布，在飞上限与提交标记逐条判过，施加的计数器从链首起逐一连续（锚点读得出时从锚点加一起；读不出时首条 `checkpoint_txg` = 所选根 + 1、序号 1）；反向链与点名单元是读盘回调的结果，写成「被施加的每条记录，回调交回了真」。**语义层**（前提：盘上每条记录出自这一版的合法写侧，即 E165（恢复重放有界穷举与变异杀伤界） 的合法历史一族；同一 (实例代号, 计数器) 在全部时间线上只对应一份内容）：「结果」是根的投影加盘，不是整条根——返回的根的 (实例代号, `checkpoint_txg`) 与新根段四个字段等于所选根那条时间线上的第 j 版（j 不早于所选根），其余字段等于所选根，卸载记号除外（由记录重建的根取「不是卸载那一串写下的」）；盘上 (所选根, j] 每一版的记录读得出、点名单元两份都验过、记录条数之和不超在飞上限 | 落点是 I-8.3（重放前缀严格连续）——它的判据补句逐字就是这一条（「前缀不得停在一次发布的中间——一次发布整体施加或整体不施加」），不要在 [invariants/03-journal.md](../../../crates/singlefs-checker/invariants/03-journal.md) 里另立一条。句法层在 Verus 核 `replay_core` 的 `ensures`（`replay_follows_the_prefix_rules`）里，必红世界登记在 `crates/singlefs-replay-prefix/verus-must-red-worlds.tsv`，其中「改判定」行只改可执行判定、不碰 spec；语义层靠 E165（恢复重放有界穷举与变异杀伤界） 的合法历史一族在界内判，Verus 不证。核之外：胶水（摊表、包回调、拼报告与新根）由 E165（恢复重放有界穷举与变异杀伤界） 直接调真 `replay_journal`、harness 用例与 `crates/mutations.tsv` 的变异行罩，拼根里投影之外的字段（卸载记号、实例表指针、分配记录树根）没有变异行、E165（恢复重放有界穷举与变异杀伤界） 的生成器把它们写成定值；两个读盘回调（反向链、点名单元闭包）由 E165（恢复重放有界穷举与变异杀伤界） 里真回调、内存盘的读数、`crates/mutations.tsv` 的变异行与只读挂载几种形状的读盘次数用例罩 |
| **M3 幂等与单调** | **幂等**：`Recover(Recover(c)) = Recover(c)`，两读都证——记录级（同一条记录施加两次与施加一次，得到的根相同，I-8.4（重放幂等） 的字面）与不动点（把重建出的根当所选根、同一张记录表、同一个读者、同一个开关再重放一次，返回的根不变；前提是所选根之上的记录不超在飞上限，且记录表满足 I-8.12（同实例记录的 txg 不减）：同一实例里 journal 记录的 txg 随计数器不减）；第二次因所选根 `checkpoint_txg` 顶格报 `CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR` 且一条没施加的，算相等。「就是 I-8.4（重放幂等）」不成立：I-8.4（重放幂等） 是记录级一读，不动点是另一条性质。**单调**（M2 的推论，不单独证）：串行提交（已定项 2）、D16（发布语义） 已定项 7 的持久顺序、所选根取根环里最新的有效根之下，同一次运行里 D13（验证路线） 已定项 4 的两个不带故障的崩溃状态 `c ⊆ c'`，`Recover(c)` 落到的版本不晚于 `Recover(c')` 落到的版本，版本按 (`checkpoint_txg`, 实例代号) 比，与 D22（单元原子性怎么合成） 已定项 7 的择新序同一个序；开并行提交或把域放宽到带故障时重开。「施加的事务集包含于」那一读法撤：它对理想恢复也为假 | Verus 核 `replay_core` 的 ensures 证两读：记录级与不动点（不动点的前件含 I-8.12（同实例记录的 txg 不减））；单调写成推论 `corollary_the_landed_version_does_not_go_back`，择根与写侧那两条前提写在它的 requires 里。E165（恢复重放有界穷举与变异杀伤界） 的对抗一族在界内判不动点，违例全在 I-8.12（同实例记录的 txg 不减） 不成立的记录表上（第 3 次重跑第一段）。checker 那一半（I-8.4（重放幂等））已实现，取在崩溃注入的二次崩溃上判、不进池级 checker 的清单：`crates/singlefs-checker-tier/src/crash_injection.rs` 的 `REPLAY_IDEMPOTENCE_INVARIANT` 与 `judge_the_remount_after_a_second_crash`。O2（独立解析器 + checker） 判不了幂等：I-4.5（已停用·需两镜像） 退役理由逐字「幂等要比对两个镜像，单镜像判不了」 |
| **M4 不少施加** | 前提与对象同 M2 的语义层。`Recover` 返回的根恰是 j\*：从所选根起每一版都完整在盘上（记录任一份自证过、点名单元两份都验过）、记录条数之和不超在飞上限的最远一版。不加这一条，M1–M3 全部证出来也排除不了「已确认的完整发布被恢复丢掉」：一条都不施加的恢复过 M1–M3 的全部字面。「不少施加」按「多出违例」读：变体多出违例才算被它挡住 | E165（恢复重放有界穷举与变异杀伤界） 的最远一版判据在界内判；Verus 的必红世界里「第一条水位之上的记录就停、一条都不施加」那一行红；M4 自己不单独证 |

**挂载透明**（独立的第四条性质，不是 M3 的第三读）：在崩溃镜像上只读恢复一次得 R1；在同一份镜像上做一个动作——只挂载（取号、写行、暖机、挂载续做与挂载自己推的抬 F），或挂载加一次发布——之后再只读恢复一次得 R2，逐字段比 R1 与 R2。比的那一版：只挂载比恢复落到的那一版，挂载加一次发布比挂载后那次发布之前的那一版。挂载途中三个相位的二次崩溃镜像 img2 上也判：C(recover(img2)) = C(v1)（v1 是 R1 落到的那一版，C 取 POSIX 内容与存在性），内容与字段两半都判；字段一半里，落回 v1 那一版自己的各字段与 R1 相同，落到这次挂载写的某一版的照逐字段表判，扫描报告不判，抬 F 第一个写起 F 不判、在它之前（含暖机）F 照「挂载没推抬 F」那一格判；抬 F 第一个写的位置按挂载交回的抬 F 那几串（准入推的与续做里推的）在挂载写表末尾的位置取，取不出（`NotLocatedInTheMountWrites`）时这一次挂载的二次崩溃整段 F 记不判。逐字段表（R2 对 R1）：

| 字段 | 判法 |
|---|---|
| POSIX 内容与存在性 | 必须相同 |
| 池身份与系统配置的不可变配置（本盘设备号除外） | 必须相同；不可变档 37 行除本盘设备号一律按原字节比 |
| 系统配置的节点大小 | 必须相同 |
| 系统配置的运行配置 36 字节（T_time、T_dirty、三水位） | R2 等于内置默认值（没有挂载选项通道，`to_slot` 总写默认） |
| 卸载记号 | R2 恒为「不是卸载写的」，R1 的记号不进比较 |
| F 与 F 生效值 | 挂载没推抬 F：R2 的 F 生效值与 R2 根里的 F 都等于 R1 的 F 生效值；推了（准入推的与续做里为空间推的都算）：R2 的 F 生效值等于挂载交回的最后一串抬到的 F |
| 实例代号 | max(各盘两槽自证系统配置的实例代号, 根环全部可读根的实例代号) + 1，两个量都从崩溃镜像读 |
| 实例表行 | 照 D18（块里携带什么信息） 已定项 11 恢复写的行 |
| `checkpoint_txg` | 只判下界：≥ max(根环全部根的 txg, 环里全部自证记录的 `checkpoint_txg`) + 1（D23（journal 的角色与格式） 已定项 14 的下界，允许跳号） |
| 根槽位置 | 不另判，由 `checkpoint_txg` 定 |
| 树 ID 水位 | 挂载不发第一个文件时相同 |
| 树表里 inode 树、extent 树、墓碑、谱系、deleted_inodes 的根指针（后三样是树表条目，种类 9、10、11，不是根记录字段） | 相同；挂载续做过时，按续做之后那一版判：等于挂载交回的续做各步里 txg 不大于 R2 那一版的最近一步的各根指针；树表单元自己的落点不比 |
| 树表种类 6、7、8、12 的根，诞生 txg、头 ID、树 ID、种类 | 必须相同（种类 6、7、8、12 的根恒全零）；诞生 txg 跨根另有 I-9.14（树表条目的诞生 txg 跨根不变），头 ID 另有 I-9.27（树表条目的头 ID 取值），单镜像判 |
| 实例表与树表的单元指针 | 不比字节，与「树表单元自己的落点不比」同 |
| 扫描报告 | 施加 0 条、水位之上 0 条 |
| 根槽偏移 428..457（算法、nonce、MAC） | 必须为 0；单镜像性质，归池级 checker 的 I-2.4（头校验和覆盖范围）判，挂载透明不比 |
| magic、flags 其余位、自证校验和、整槽校验和 | 不比（结构性，读槽时判） |
| 系统配置槽世代号 | 不比，交 I-7.16（系统配置槽世代号与槽号的关系） 在 R2 的镜像上判 |
| 系统配置的 journal_tail | R1 ≤ R2 ≤ R2 环里自证记录的最大计数器，挂载做完取等 |
| 系统配置的 journal 实例代号 | 挂载做完等于 R2 根的实例代号；二次崩溃时取 R1 的实例代号或新实例号两者之一 |
| 分配记录（含根记录的 `mapping_root`、`allocation_record_tree_root` 与树表种类 3、4 的根） | 不比整表：POSIX 内容相同，且池级 checker 对 R2 的镜像判绿；`mapping_root`：R2 全零 ⇔ R1 全零；`allocation_record_tree_root`：R1 带树 ⇒ R2 全零，R1 树表 0 条 ⇒ （R2 非零 ⇔ 新实例号 ≥ 2） |

系统配置的运行量（槽世代号、journal_tail、journal 实例代号）落回 v1 时照它们自己那一行判，不按 v1 相同判。

**射程**：refinement 的边际价值只在崩溃点重放枚举取不到的那几维——载荷字节、记录条数 / 段数 / txg 数（含环回绕）、上一条时间线的残留、槽数与设备数；崩溃点那一维崩溃点重放已全量枚举。「校验和过了」在模型里是一个布尔标志，**不许读成 refinement 验了完整性**——与已定项 14 里 D4（校验和位置） 那一行「不许读成『验了 Merkle』」同形，也与 D13（验证路线） 已定项 4 对 CRC32C 碰撞概率的「知情接受」是同一笔账。D13（验证路线） 已定项 4 是 refinement 左边的 `Crash`（模型层的枚举域），不是右边的 `Spec`，拿它当 spec 会让 refinement 恒真。I-6.5（nonce 水位单调） 的形式化撤下；D18（块里携带什么信息） 已定项 14 定了、且 nonce 分配规则（单调计数器 + 预支水位、水位在使用前持久）被立成一条分项之后，这一格重开——重开时它唯一的真优势是 nonce 对 POSIX 不可见，它的 spec 与 O1（内存模型对拍（HashMap 理想文件系统）） 的 model 结构上不可能同源，不受冲突 2（D13（验证路线） 已定项 13） 影响。

M1–M4 的前提与域：语义层只挂「同一 (实例代号, 计数器) 在全部时间线上只对应一份内容」，由 D23（journal 的角色与格式） 已定项 14「这一版的失败处置」的原样重发担保，根槽写失败推进 `checkpoint_txg` 再发那条路落地时重核；前提域取 E165（恢复重放有界穷举与变异杀伤界） 的合法历史一族（合法写侧写出的历史的任一落盘子集，较新的根槽可以读不出）。M2 的「其余字段等于所选根」里，由记录重建的根的卸载记号是 `NotWrittenByTheUnmountSequence`（D22（单元原子性怎么合成） 已定项 7「其余根写 0」），与所选根的记号可以不同，所以写「卸载记号除外」。M3 的单调推论只在串行提交、不带故障的崩溃状态里成立。M4 连同择根才排除「已确认的」完整发布被恢复丢掉，前提要全：不带故障的崩溃状态（已定项 4 的枚举域）、所选根按 D22（单元原子性怎么合成） 已定项 7 的择根取、链不跨实例边界（D23（journal 的角色与格式） 已定项 14 注 1）、新实例本实例写成的根覆盖两块盘之前不让任何 fsync 返回（D16（发布语义） 已定项 8 的暖机）；带故障的域里不排除——在飞上限会截掉记录完整的发布，一块盘整块掉了时点名单元少一份的那一版不施加（C519（丢一整块盘时恢复丢掉刚确认的那一版））。

挂载透明的射程：只管不带故障、可写挂载做成的崩溃状态。可写挂载被拒的状态不判透明，判拒绝本身合不合法：拒绝合法 ⇔ 拒因在 D23（journal 的角色与格式） 已定项 14 那张单子上，且照那一条的判据在崩溃镜像上自己复算成立，且盘上逐字节不变，且只读读出在允许集合里；拒因不在单子上的照崩溃注入第二截的写法报红。带故障的状态不在射程：取号那一刻有一槽读不出与 N-配置报成同一个拒因，接进带读故障的档之前要分开（C664（挂载拒绝合法复算把两种拒因报成一个））。逐字段表里树 ID 水位、两棵树的根指针、`checkpoint_txg`、扫描报告四格是推的（三方判决取辩方的式子，没量），F 那一格「推了」的一半没量；根记录的 `mapping_root`、`allocation_record_tree_root`，墓碑、谱系、deleted_inodes 三根指针，算法、nonce、MAC，系统配置的槽世代号与 journal_tail 各归哪一类，照 C655（挂载透明逐字段表有字段没归类）、C687（从 mkfs 水位建树那一版落点判不了的几格） 判决 Y1、Y2 写在逐字段表里。

**依据**：
- E42（一事务几条记录）：为了让「被切开 ⇒ 非法」成为恒等式而钉死了载荷，「被切开却碰巧合法」这一类它结构上看不见——M2 把载荷符号化补的正是这一格。
- 三方：材料 `_d13-item1-background.md`，三条腿的产出 `d13-item1-opus-forward-output.md`、`d13-item1-sonnet-reverse-output.md`、`d13-item1-local-attack-v2-output.md`；三腿一致把原候选排最后，两条云端腿各自收敛到「前缀判定与提交原子性」这道接缝，主 agent 按封闭性把择新与发布原子性收进同一个单元（判决与撤下原候选的三条门槛原文在变更史）。
- E165（恢复重放有界穷举与变异杀伤界）：在界内对真 `replay_journal` 判 M2 的语义层、M4 与 M3 两读，并量变异杀伤界；用户 2026-10-01 定它当轻量工具；C 域合法崩溃状态数按两种切法分写（甲切法是 D16（发布语义） 已定项 7 的屏障位置）；对抗一族里不动点的违例全在 I-8.12（同实例记录的 txg 不减） 不成立的记录表上，谓词成立那一边 0 违例（第 3 次重跑第一段）；择根合判族调真 `choose_root`、`scan_journal`、`replay_journal`，界内、合法、不带故障时已确认的完整发布不丢（第 3 次重跑第二段）；择根序的四个改坏变体在合法历史 K、K9、KW 三族界内只有取第一个有效者红，另三种在对抗根环 KA 上红（第 3 次重跑第三段）。
- 三方判决（M1–M4 的措辞与落点）：`research/prompts/m3-formal-r1-main-verification.md`、`research/prompts/m3-formal-r2-main-verification.md` 第二节、`research/prompts/m3-formal-r3-main-verification.md` 第二、三节；M3 单调推论的四步推导在 `m3-formal-r2-opus-output.md`「3.2 推的：串行提交下，j1 与 j2 推得出落到的版本单调」一节。
- 三方判决（M3 不动点前提补 I-8.12（同实例记录的 txg 不减） 与落点现状）：`research/prompts/m3-formal-code-r1-main-verification.md` 第二节不动点前件那一格（打中条款、不打代码）与第三节第 5 条；装置的前提比 Verus 的前件宽、违例全落在该谓词不成立一边的对账在调度记录 `records/2026-10-01-里程碑三形式化接入.md` 第 10 批「10-5 交回」那一条。
- 用户定案（2026-10-01 弹窗，都照推荐）：卸载记号除外、M4 射程句的前提写全、单调的版本序取 (`checkpoint_txg`, 实例代号)、txg 顶格取「报 no_successor 且一条没施加算相等」；写回等 K10 判完一起做，原话在变更史。
- 挂载透明：无实验：判法在崩溃注入里（`crates/singlefs-harness/src/mount_transparency.rs`，checker 档的崩溃注入调它），不是实验；三轮判决里攻方的数在副本装置上量，代码轮第二轮攻方在入库代码上量的第一次崩溃 147 806 态、二次崩溃 234 773 态 0 红主 agent 没复跑。
- 三方判决（挂载透明）：`research/prompts/m3-formal-r4-main-verification.md` 第二节「挂载后再恢复：模掉什么」「比的对象与射程」两行与第四节第 2、3 条（独立的第四条、比的那一版、射程）；`research/prompts/m3-formal-r5-main-verification.md` 第二节 U5、U6 两行与第四节第 5、6 条（逐字段表、二次崩溃比 v1、拒绝合法的条件）；`research/prompts/m3-formal-r6-main-verification.md` 第二节 U5、U6 两行（txg 只判下界、拒的判据在崩溃镜像上自己复算）与「逐字段表没碰的字段」一行；`research/prompts/m3-formal-code-r2-main-verification.md` 第二节 K2 各行（实现照判、站住；暖机与挂载推的抬 F 拆段）。
- 用户定案（挂载透明加二次崩溃，2026-10-02 弹窗，原话在变更史）：`research/prompts/m3-formal-r4-main-verification.md`「用户定案（2026-10-02）」二次崩溃那一行。
- 择根序变体那一句：调度记录 `records/2026-10-01-里程碑三形式化接入.md` 第 12 批 E165（恢复重放有界穷举与变异杀伤界） 第 3 次重跑第三段判 Q2 交回那一条（主 agent 判补这一句）。
- 挂载续做算挂载的一部分、F 的界按挂载交回的抬 F 在写表末尾的位置取：用户定案 2026-10-03「续做算挂载的一部分」（原话在变更史）；代码轮第三轮判决 `research/prompts/m3-formal-code-r3-main-verification.md` 第三节第 3 条（界按写流位置取，找不到不 panic）；实现 `crates/singlefs-harness/src/mount_transparency.rs`。

**欠**：C77（重放起点未定义）：在择新与前缀判定那道接缝上；C649（Verus 判不到的两样回调结果）：已还清（M2 语义层的反向链与点名单元仍只能写成后置条件，Verus 一侧不判，两个读盘回调改由 `crates/mutations.tsv` 的变异行罩着，见欠账表已还清表）；C650（择根与重放没有合判）：M4 的射程句要择根与重放合起来才成立；C653（扫环胶水内存随盘数涨）；C654（必红世界期望条数随求解环境漂）；C655（挂载透明逐字段表有字段没归类）：已还清（逐字段表照判决 Y1、Y2 补齐，见欠账表已还清表）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:89-102`（整段抄，未转述）**

```markdown
#### 已定项 2：先串行提交

**定案**：事务层先承诺串行提交——`.claude/rules/fs-design.md`「判据写不出来时，先只做一条，而且**先做慢而正确的那条**」落到实处，串行是本工程明令的默认值，不是在两条路里挑了一条。越过交点再开并行，而不是在交点之前就承诺并行。对将来并行提交的一条前置约束：记录按事务号顺序追加，事务号 N 的 fsync 在返回前必须等 < N 的事务全部已追加 ∨ 已作废 ∨ 已确定实例切换或转只读（作废是提交管道的内存状态，对并发后继可见）；串行提交下这条规则空转。

**射程**：串行的天花板在一次 checkpoint 约 1.1 万条条目（单线程合并耗时追上一次设备同步的交点，算式在 C58（串行提交撞天花板没人报警））：低于交点设备同步是瓶颈、串行不吃亏，高于交点单线程合并变成天花板，没有安全余量。已定项 2 不决定记录头的字段清单、不改变新池新建文件写出的字节，它决定性能天花板与 crash refinement 的范围。并行提交仍然会动崩溃语义：「前缀」是全序上的概念，并行下一个前缀可能同时含有完整的事务 B 与残缺的事务 A；记录头已带事务边界字段（D23（journal 的角色与格式） 已定项 7：事务号 8 + 提交标记 1）。

**依据**：
- E17（write buffer 合并上限）：单线程 sort-merge 的上限是交点的一半；按叶分区的无锁并行是越过交点之后的逃生口，但不许默认它够。
- E44（序号位宽的本机实测与代价）：本机 fsync 率是交点的另一半。
- E42（一事务几条记录）：串行下一个事务跨多条记录同样会被前缀切开——决定记录头字段清单的是「一个事务恰不恰好产生一条 journal 记录」，不是串行 / 并行。
- 用户定案，记在变更史；将来并行提交那条前置约束来自 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P1。没走三方论证：这是确认一条已经写明的默认值，真正要走三方的是何时改成并行，那一步没做。

**欠**：C58（串行提交撞天花板没人报警）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:103-135`（整段抄，未转述）**

```markdown
#### 已定项 3：RefFS 做到极致，自己处理持久化与崩溃语义

**定案**：不做只管功能的内存模型，做一个自己处理持久化与崩溃语义的参照实现（RefFS），把 RefFS 从「储备候选」升为**要做的东西**。「做到极致」取语义级：RefFS 只在内存里按 POSIX 操作走，自己定义每次 fsync 之后什么算持久、崩溃之后允许看到哪几种 POSIX 状态，没有盘上格式；带自己盘上格式、自己写盘与恢复的那一档（工程量接近再写一个文件系统）不做。RefFS 与实现不许共用崩溃点枚举的那段代码：做到极致之后「三 oracle 合谋测试」（已定项 6）防的那个共同误解，最危险的落点是「RefFS 与实现是否共享同一个崩溃语义的错误理解」。

RefFS 的来处与边界：

| 项 | 定案 |
|---|---|
| 来处 | 只照 kb 条文另写：不拷 O1（内存模型对拍（HashMap 理想文件系统）） 的代码，不读录制流，不调切段、状态计划与实现的崩溃点枚举 |
| 放哪 | 单独一个包 `crates/singlefs-reference-filesystem`：依赖闭包（dev-dependencies 也算）⊆ {它自己, `singlefs-format`}，能只用 std 就只用 std；源码不引 `singlefs_core`、`singlefs_checker`、`singlefs_harness`、`singlefs_checker_tier`（经改名的别名同样算）；`singlefs-core` 与 `singlefs-checker` 的依赖闭包里没有它。三条由门禁 checker-independence-and-sync 的 checker-implementation-disjoint 格判 |
| 输入白名单 | 都由调用方交入，只收五样：每个操作的结局（成、在任何写之前拒、写了之后报错、原样重发）；崩溃落在第几个操作里（hi）与它的归属规则；回退目标 m；每次恢复读出的内容；显式发布点 |
| 第一版的操作 | 第一个文件、覆盖写、发布调用、fsync、回退 |
| 调用点 | 崩溃注入与故障注入的调用点上，RefFS 的判定与 O1（内存模型对拍（HashMap 理想文件系统）） 崩溃判官的全部四条判据合取，任一红即红 |

崩溃语义（串行提交、不带故障的崩溃状态）：

1. **fsync**：fsync 返回成功之后，这次 fsync 等到的那次发布带走的全部操作必须在（串行下就是调用开始之前已取号成功的全部操作，按单元计）；按状态下标读，即允许集合的下界 lo 抬到那次发布的末尾。fsync 报错时不抬 lo，报错那次发布带的操作可在可不在；之后的操作接在下一次恢复读出的状态上判。
2. **丢法：按发布边界的前缀**。崩溃落在第 hi 个操作里时，恢复允许读出的是 {S_j : j = lo，或 j 恰是某次显式发布的末尾；lo ≤ j ≤ hi}，S_j 是上一次恢复之后前 j 个操作施加出来的状态，S_0 是上一次恢复读出的内容。丢掉任意子集的那种丢法在串行提交下出局（与 D16（发布语义） 已定项 12 不相容），开并行提交时重开。
3. **抬 lo**：返回成功的发布调用、fsync、回退都抬 lo（D16（发布语义） 已定项 7：回退与 fsync 同一个返回条件）；回退算一次已确认发布，它的下标算发布末尾。
4. **回退**：回退是 RefFS 的一个操作，回退之后的内容等于整段历史里第 m 次已确认发布之后的内容；m 由调用方从实现报的回退根翻出来交入，RefFS 不判目标选得对不对。m 的编号覆盖每一个写根的入口（发布调用、回退、写行与暖机、抬 F、卸载），映到写它时的现行版本，现行版本含每次恢复读出、重新起头的那一份。
5. **崩溃时刻的归属**：崩溃落在第几个操作里，按「切开当前段的那道屏障由哪一步发出」归属，不按第一个被扣下的写；这条只给 RefFS 用，O1（内存模型对拍（HashMap 理想文件系统）） 的崩溃判官不动。
6. **恢复之后**：每次恢复之后，RefFS 先判读出的状态在允许集合里，再以它为起点接着走。

**射程**：只定做到什么程度与 RefFS 的形态，不定排期；不许读成「先做 RefFS 再做实现」——RefFS 是验证手段不是里程碑（`.claude/rules/format-evolution.md`「从写开始，不从读开始」）。显式发布点只取历史里的发布调用与 fsync：自动触发的攒批落地之前 RefFS 只接显式发布点，自动触发的发布不在 RefFS 射程，归已定项 1 的 M2 与记录核对器；崩溃恢复抛弃最新根的那一次，RefFS 以读出内容重新起头、不判那一步。带故障的崩溃状态（说谎的设备、丢整块盘 C519（丢一整块盘时恢复丢掉刚确认的那一版））不适用。独立性：RefFS 不读录制流、不调切段与状态计划，但它的 (lo, hi) 与 O1（内存模型对拍（HashMap 理想文件系统）） 崩溃判官盘级下界那一条判据的入参输入同源，记在 C191（同源性计数器只盖了一对） 那一类同源账上。D17（实现分层与第三方管道） 已定项 4 据此把「入口」的可执行定义交给 RefFS 的操作面，fsync 的持久性承诺由 RefFS 的崩溃语义定义；O3（独立规约执行器） 仍是崩溃点集合的独立执行器，职责不变。RefFS 的第一版在 `crates/singlefs-reference-filesystem`，崩溃注入两截与故障注入重开的调用点已接：harness 档经翻译层 `crates/singlefs-harness/src/reference_filesystem_translation.rs` 依赖它，调用点判 RefFS 与 O1（内存模型对拍（HashMap 理想文件系统）） 崩溃判官两票、任一红即红。

**依据**：
- 无实验：做到什么程度、RefFS 怎么来与收什么是验证投入与边界的取舍；三轮判决里攻方的数都是副本装置上量的，入库装置上还没重做（`research/prompts/m3-formal-r5-main-verification.md` 第五节第 3 条），这里不引。
- 用户定案，原话在变更史：「做到极致」（原定案）；做到语义级（2026-10-02，岔路单 `m3-formal-7-forks.md` 里 RefFS 做到哪一档那一行，是对原定案换了读法）；按发布边界的前缀、调用点合取、另起、单独一个包（2026-10-02，`research/prompts/m3-formal-r4-main-verification.md`「用户定案」）；合取收严成全四条、回退目标作输入（2026-10-02，`research/prompts/m3-formal-r5-main-verification.md`「用户定案」）；回退抬 lo、崩溃时刻按屏障归属、m 覆盖每一个写根的入口、白名单加原样重发（2026-10-02，`research/prompts/m3-formal-r6-main-verification.md`「用户定案」）。
- 三方判决：`research/prompts/m3-formal-r4-main-verification.md`（第一轮：fsync、丢法、独立性三格与六条岔路）、`research/prompts/m3-formal-r5-main-verification.md`（第二轮：fsync 合成句、按发布边界补三句、合取收严、另起与单独一个包、任意子集出局限串行）、`research/prompts/m3-formal-r6-main-verification.md`（第三轮最终轮：回退与 lo、射程两句、白名单、fsync 按状态下标读）。
- D17（实现分层与第三方管道） 已定项 4：O1（内存模型对拍（HashMap 理想文件系统））明写「不管崩溃」，定义不了 fsync 的持久性承诺，而第一个可运行目标恰恰是「正确提交一个事务」——做到极致的 RefFS 补的正是这一半。

**欠**：C447（规约执行器没接进崩溃点重放与 RefFS 比对）；C191（同源性计数器只盖了一对）；C658（崩溃注入第三截放过丢掉的一步复活）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:136-151`（整段抄，未转述）**

```markdown
#### 已定项 4：崩溃点重放只枚举整写子集

**定案**：崩溃点重放要枚举的崩溃状态集合定义为：录下来的写请求流**按设备**切成段——一次写只被它自己那块盘上之后的屏障（或它那块盘上的 FUA 写）排在之后的写前面；当前段里每块有写的盘都被自己的屏障或 FUA 放行时这一段才关上，有一块盘还没放行，这道屏障（或 FUA）不关段、前后的写同段。一个崩溃状态是「前若干段全部持久，当前段每次写各取它的几态、任意组合，之后各段全部没持久」；枚举的单位是**一次整写**。带 FUA 的写只放行它自己那块盘：它与那块盘上前面没被屏障隔开的写同段，别的盘上的写不因它持久。一次写取两态（持久 / 没持久）；**原地覆写**（不是单元写、长于一个扇区、罩住的范围里原来有东西的写，系统配置槽写与覆盖旧记录的 journal 写属于这一类）多取第三态「新旧都读不出」，全量与快档都枚举；第三态的镜像罩那次写的整个范围，新旧不同的那一截前一半是新字节、后一半是旧字节（按字节撕，不按扇区）；其余写的撕裂态一律并进「这次写没持久」。「没持久」的位置上，镜像里放的是**崩溃前那个位置的旧字节**，不是零，而且这些镜像（第三态的在内）要真的生成出来喂给 checker 与记录核对器，不许只当计数。镜像双写（D2（RAID 条带策略） 已定项 6 的 w ≥ 2、D22（单元原子性怎么合成） 已定项 8 的 journal 镜像）按**设备**各算一次写，harness 的状态数按自己的写流另算闭式。**同一块盘上连着的屏障并成一道**：录制流里一道屏障与这块盘上一道屏障之间没有任何一次写（别的盘的屏障不算写）时，这一道不记（`crates/singlefs-harness/src/lib.rs` 的 `push`），独立模型照同一条规矩记；中间没有写，这一段不产生崩溃状态，合并与否不改变崩溃状态集合，只改变操作数与种类串。

**射程**：它定义的是模型层的枚举域。真实设备的 FLUSH / FUA 是否如宣称生效归 C6（块层语义假设写错），要在 QEMU 里用真设备验。按设备切段多出实际走不到的状态（一块盘没放行时，别的盘上已放行的写也留在段里取任意组合），不漏走得到的。撕裂分两类：原地覆写之外的写（单元写、不长于一个扇区的写、罩住的范围里原来没东西的写）撕裂态与「没持久」的差别只有 CRC32C 的碰撞概率（32 位，D19（块指针的结构与宽度预算） 已定项 2、D23（journal 的角色与格式） 已定项 11），**知情接受**，撕裂粒度不进模型，与 D20（承重面：单元的原子性与自包含）「有父指针的单元不依赖任何宽度」一致；原地覆写撕裂时那一处的旧内容也没了，「没持久」却留着完整的旧内容，两者不是同一个状态，所以单独枚举。崩溃点重放的扇区取 512 字节；两条崩溃点重放流上取第三态的只有系统配置槽写。已知边角：第三态的旧字节取「这次写之前的写全落了」那一版，同段更早、与它重叠的写没落的组合里旧的那一半多带那次写的字节（两条崩溃点重放流上这种组合一次都没有）；根槽写长于一个扇区又罩住旧内容时第一版不支持第三态（harness 断言；崩溃点重放的池都按 512 字节物理块建，走不到）。设备把一次写撕成恰好撞上校验和碰撞的形态不在模型内。D13（验证路线） 已定项 5 与它正交：一个定枚举域、一个定 crate 边界。

**依据**：
- E77（发布的持久顺序）：段模型在四种屏障摆法下的状态数逐臂等于闭式——这套枚举域数得对。
- E142（新池新建文件的干跑）：第十七次跑第一段的几何敏感性取样点 G7（阳性对照同一条写流、只把 FUA 当边界）实测 4097 个状态、2046 个违例——没有屏障时单元、journal 记录与根槽合成一段 `[12]`；装置的切法与 `crates/singlefs-harness/src/segments.rs:91`「FUA 写关掉自己所在的那一段（FUA 不替前面的普通写做持久，所以它们同段、任意子集）」一致，「FUA 自成一段」那一读法预言的 2050 / 0 对不上——FUA 那一句的字面照实现的切法写。
- 三方对抗与判决：[verification-build.md](../verification-build.md)「三方对抗（2026-09-03）」判决第 3 条——每一类单元都有整单元校验（有父指针的靠父指针里的校验和，D4（校验和位置） 已定项 1、D16（发布语义） 已定项 7、D22（单元原子性怎么合成） 已定项 19；自证单元靠整单元校验和加被实际检查的代号，D22（单元原子性怎么合成） 已定项 21；journal 记录靠头校验和与头内的载荷校验和，D23（journal 的角色与格式） 已定项 13），撕裂态与「没持久」在校验和眼里是同一件事。
- 用户定案（按 E77（发布的持久顺序） 与三方对抗定；FUA 写算段边界那一句是 C313（FUA 写算不算崩溃段的边界） 收口时的用户定案，2026-09-26 用户定照实现改成「关掉它所在的那一段，与前面没被屏障隔开的写同段」），记在变更史。
- 按设备切段（有一块盘没放行就不关段）、FUA 只放行它那块盘、原地覆写多取「新旧都读不出」：用户定案（代码审阅第 1、3、4 条；2026-09-27 弹窗选「A 合并，按设备记屏障」「照补，全量也跑」与「原地覆写补第三态」），第三态按字节撕是主 agent 同日定的、被攻过零轮，记在变更史；实现与判别力（只吞盘 1 一道屏障，段尾 `26,2,1,2` → `26,5`，崩溃点重放小流里记录核对器报出来）在 `m2-rev-b3a2-implementer-report.md` 第一、三节。
- 同一块盘上连着的屏障并成一道：用户 2026-09-28 定「并，跟今天的录制器」（E142（新池新建文件的干跑） 第十九次跑撞上登记的停机 S19-crates：独立模型两道都记、录制器并掉第二道，暖机与整条流四格的操作数与种类串不同、崩溃状态数相同，报告 `e142-r19-runner3-report.md` 结论段，登记 `e142-r19-prereg.md` 的 R40），原话在变更史。

**欠**：C6（块层语义假设写错）：拿虚机的设备侧日志重建崩溃状态、崩溃测试必须变红的那一半。

```

**出处 `.claude/kb/decisions/13-验证路线.md:152-168`（整段抄，未转述）**

```markdown
#### 已定项 5：只共享一份从 kb 生成的常量

**定案**：checker 与实现之间只共享一样东西：一份由 kb 的字段表生成的常量模块，生成器从 kb 读、两边都只消费、任何人不许手改（`.claude/singlefs-ai-sop/rules/machine-first.md`：重复要生成，不能手抄）。**其余一律不共享**：地址空间的 newtype 各自声明、格式解析各写一份、校验和各用一份独立实现、遍历与记账代码交集为空（C12（增量语义共用） 由门禁 checker-independence-and-sync 的 checker-implementation-disjoint 格定的判据，形态是 crate 粒度的依赖闭包交集、不是符号级，「格式解析与常量除外」那条例外就是这一项在用）。

- **判定宽度不许从别处拿**：I-7.5（根槽按判定宽度对齐） 与 I-8.2（记录头不跨原子单元） 的判据是挂载时探测的 `physical_block_size`，它不在镜像里。checker 自己探一次，再与系统配置记录的 mkfs 时取值比对（D22（单元原子性怎么合成） 已定项 2「mkfs 按池内最大值划、挂载时逐设备复算比对」的 checker 侧形态）。**探不到时报「声明值，未探测」，不许当成探到的**——文件当设备的镜像属于这一类，崩溃点重放的绝大多数镜像都是文件，所以这条不是边角。异构池里判定宽度是一个集合不是一个数（C17（设备几何当全局参数）），checker 要复算 mkfs 那步取最大值的聚合，不然健康的异构池上会假红。
- **生成器拒绝发射没有 kb 落点的常量**，也只发射标量值：它一旦开始发射「按字段表算出来的偏移函数」，那就是 D13（验证路线） 明令不许共享的格式解析，而不再是常量；判据是发射物里有没有分支与算术。
- **CRC32C 各写一份的前提是参数钉死**（多项式、初值、输入输出反转、异或输出）：参数不钉死，两份独立实现会在健康镜像上给出不同结果，那是假红而不是独立性。

**射程**：管 checker 与实现的 crate 边界；管不到 kb 里就写错的值——共享一份生成常量与两边各抄一份都抓不住那一类，它要一条正交的门禁（C94（登记的格式常量与后来的定案对不上））。实现与它的差距：`crates/singlefs-format` 是手写的，不是生成的（值由门禁 code-source-discipline 的 format-constants 格按 kb 里的 `format-const` 标记绑住，只绑标了的那些），生成器没有；`crates/singlefs-checker` 只依赖 `singlefs-format`，CRC-32C 另写了一份按位的；checker 取的是系统配置声明的 `physical_block_size` 与 `io_min`（`crates/singlefs-checker/src/image.rs` 的 `geometry_of`），「探到 / 声明」两种来源的类型是 `crates/singlefs-checker/src/lib.rs` 的 `DecisionWidth`：声明值那一种（`DeclaredOnly`）已接到 I-7.5（根槽按判定宽度对齐） 的判定上（已实现，`decision_width_declared_by_the_chosen_system_configurations` 取各盘所选系统配置声明的 max(`physical_block_size`, `io_min`) 里最大的），探到那一种（`Probed`）只有类型定义（`crates/singlefs-checker/src/lib.rs`），全仓没有构造它的代码、没有接到任何判定上；I-8.2（记录头不跨原子单元） 未实现。系统配置里 mkfs 时的 `physical_block_size` 字段在 D22（单元原子性怎么合成） 已定项 9 的几何段（4 字节）。

**依据**：
- 无实验：共享边界是保验证独立性的政策，没有可量的量；它的判别力靠两边各写一份的检查逐条红给人看。
- 三方对抗：材料 `_checker-sharing-vehicle-background.md`，腿的产出 `research/prompts/checker-sharing-vehicle-*`，腿况在 [verification-build.md](../verification-build.md)——正推腿从 D13（验证路线）、C12（增量语义共用） 推出「常量在例外内、其余禁止」；替代反推腿与本地腿各自构造出同一条：kb 里写错的值共享与各抄都抓不住。
- 用户定案，记在变更史。

**欠**：C17（设备几何当全局参数）；C387（常量模块手写，不是从 kb 生成）；C388（checker 的判定宽度不自己探测）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:169-198`（整段抄，未转述）**

```markdown
#### 已定项 6：三个互相独立的 oracle

**定案**：三个 oracle 各判一样东西，独立性各有来处：

<!-- doc-lint:registry name-col=2 -->

| # | oracle | 判什么 | 独立性来自哪里 | 验不到什么 |
|---|---|---|---|---|
| O1 | 内存模型对拍（HashMap 理想文件系统） | POSIX 可见语义：内容、大小、目录结构；无崩溃处的内容期望归它，崩溃之后允许看到的集合归 RefFS（已定项 3） | 不碰盘、不解析磁盘格式 | 磁盘结构是否自洽；崩溃语义；**两份实现对同一条 POSIX 语义的共同误解** |
| O2 | 独立解析器 + checker | 磁盘字节是否满足 I-1（块自描述）~I-6（加密不变量） | **checker 不得调用被测 crate 的任何反序列化/遍历代码**，必须重新实现一份解析，直接读裸设备 | 只判结构自洽：一个块被覆盖成另一份**合法**内容，Merkle 链照样自洽、记账照样对 |
| O3 | 独立规约执行器 | 崩溃点截断后的每个磁盘状态是否落在规约允许的集合里 | **用另一种语言写**（不用 Rust），只照 decisions 与 invariants 的文字重新推导；第一版用 Python 单写（E170（O3规约执行器第一版）） | 规约本身可能写错——**必须配阳性对照才有意义**；被投影的六样的取值与六样之外的根记录字段、实现自己的恢复与读者代码、系统配置槽的见证值（journal_tail）、一次发布内各写的先后 |

**独立性要被验证，不许自称**：设计一个「三 oracle 合谋测试」——拿一个精心构造的坏实现
（例如 nonce 永远为 0）去过三关，若三者全部放行，说明存在共同盲区，回头补 oracle。
功能正确性不只押在这三个 oracle 上：O1（内存模型对拍（HashMap 理想文件系统）） 那一行已把崩溃之后允许看到的集合交给 RefFS（已定项 3），恢复函数的 M2–M4 落在 Verus 与 E165（恢复重放有界穷举与变异杀伤界）（已定项 1），需要第二个输入的核对归记录核对器（已定项 7）；三个 oracle 之间的共同盲区靠「三 oracle 合谋测试」找。

**射程**：三个 oracle 各自验不到什么写在表的最后一列；O2（独立解析器 + checker） 的定义域归已定项 7，需要第二个输入的核对归记录核对器。**实现今天的样子**：O2（独立解析器 + checker） 在 `crates/singlefs-checker`（只依赖格式常量 crate）；O1（内存模型对拍（HashMap 理想文件系统）） 的形态是 `crates/singlefs-harness/src/model.rs` 的理想模型（只住内存、只用 `singlefs_format` 的常量、不建崩溃），门禁 harness-model-differential-and-scenarios 拿随机历史逐步对拍；O3（独立规约执行器） 是只照 kb 文字写的 Python 执行器，现行那一份是 E171（三oracle合谋测试） 第 1 次重跑用的 `scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates_journal_header_487/`（E170（O3规约执行器第一版） 第 2 次重跑从第 1 次重跑冻结的 `scripts/experiment-apparatus/e170_second_run_o3_spec_executor/` 拷出 `scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates/` 加谓词 W、T，再跟上 487 字节 journal 记录头；第 1 次重跑冻结的那一份与第一次那一份 `scripts/experiment-apparatus/e170_o3_spec_executor/` 留档）：严格跑法判得出（第一次导出的批 1、批 2 上 0 态撞「规约欠一句」），与实现逐态一致（两边全放行，不当它判得对的证据；「择到的根不是自证根里 txg 最大的那条」「③ 收到不少于 2 槽」两条路径批里 0 态，C661（规约执行器批里两条路径没有态）），坏状态表照改正的期望重判有判别力（阳性对照与盲测都判对），还没接进崩溃点重放；判一个崩溃态的做法：照规约的恢复规则（择系统配置、择根、前缀按 D23（journal 的角色与格式） 已定项 14 的五条口径）从镜像自己恢复出一版，恢复不出（整池拒或没有自证根）判不在；恢复出的这一版的投影（实例代号、checkpoint_txg、树表单元指针、中央映射树根指针、树 ID 水位、F）要等于版本表里某一版的投影，且这一版的下标在 [已确认, 已开始] 之间。版本表是基镜像上自己恢复出的现行那一版，加录制流里每一次根槽写按流序各一版，每版的投影取实现写进根槽的字节；已确认取崩溃态里落了的最晚一条写之前已返回的最晚一版（只从落了的写推），已开始取落了的写所属的最晚一版。系统配置槽只用来择槽、取根环与 journal 环的几何、判池级字段越界整池拒。所以恢复规则那一半是照 kb 文字另写的，允许集合那一半取自实现自己写出的根；RefFS（已定项 3）在 `crates/singlefs-reference-filesystem`，崩溃注入与故障注入的调用点已接；「三 oracle 合谋测试」是 E171（三oracle合谋测试），读数取自一棵冻住的树（快照摘要见实验页，与现行 `crates/` 不是同一份）：O1（内存模型对拍（HashMap 理想文件系统）） 一列是模型对拍加模型的崩溃判官，不接 RefFS，与表里 O1（内存模型对拍（HashMap 理想文件系统）） 那一行不同（崩溃之后的集合那一半 E171（三oracle合谋测试） 没用 RefFS）；O3（独立规约执行器） 一列用 E170（O3规约执行器第一版） 第 1 次重跑冻结的那一份；手造坏镜像那一族只喂 O2（独立解析器 + checker）、O3（独立规约执行器），期望出自 E170（O3规约执行器第一版） 的设计员，Q2 照登记字面判「有」（4 张全是 P8），按登记 F3 判成 E170（O3规约执行器第一版） 推期望推错、这四张不是坏镜像，真坏镜像里 O2（独立解析器 + checker）、O3（独立规约执行器） 两条全放行的 0 张；三条的源头是否独立（表第 4 列）E171（三oracle合谋测试） 只在 Q2 量过 O2（独立解析器 + checker）、O3（独立规约执行器） 一对，那一对的补丁与期望同出 E170（O3规约执行器第一版） 的改法代码，不当独立读数，O1（内存模型对拍（HashMap 理想文件系统）） 不在 Q2 里；没有量到的不写成「互相独立」被验证过。

**三条 oracle 都不判的两样**（第一样现在有崩溃注入这一侧的判官）：一是无读故障的崩溃态上，D23（journal 的角色与格式） 已定项 14 可写挂载那条见证判据为真（系统配置槽里的见证值超前于盘上读得出的记录）：崩溃注入第二截判拒绝合不合法时只在崩溃镜像上复算这条判据，为真就判合法（挂载透明的射程只管不带故障的状态，已定项 1）；要有判官问「这个崩溃态没注入读故障，判据凭什么为真」：崩溃注入第二、三截在没注入读故障的崩溃态上复算这条判据，为真即判拒得不合法（`crates/singlefs-checker-tier/src/crash_injection.rs` 的 `RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults`，C670（无读故障的崩溃态上可写挂载见证判据为真没有判官） 已还清，见欠账表已还清表）。「无读故障的崩溃态上每块盘至少一份自证过的系统配置」这一性质由 G4 判（崩溃注入第二、三截，与 C670（无读故障的崩溃态上可写挂载见证判据为真没有判官） 那条同接法）：mkfs 返回之前的崩溃态不在 G4 的射程，mkfs 逐盘两槽在同一个循环里写、之后才逐盘屏障，崩溃注入的候选段与 E171（三oracle合谋测试） 的基镜像都取在 mkfs 之后；G4 与那条还有一处不同形：择不到系统配置时那条的复算交 `None`、G4 按真，这时读阶段本身已拒、到不了这一格。二是一次发布内各写的先后：O2（独立解析器 + checker） 的 I-7.12（系统配置 F 不低于同盘根上的 F） 判抬 F 那一串先写系统配置这一步；变异点名的用例只判 E171（三oracle合谋测试） 第 270 行那一种（发布返回之后每个崩溃点的见证值），第 242 行点名的用例判的是失败路径冻结；`layout/01-first-txn.md` 第八节把根槽 FUA 与系统配置槽轮换登记成两段，只是登记在册；其余写序不在三条之内。

**依据**：
- E170（O3规约执行器第一版）：O3（独立规约执行器） 照「另一种语言、只照 kb 文字重新推导、配阳性对照」写得出第一版，判得动单个崩溃状态、一态够进快档；第 1 次重跑（照 2026-10-03 的 kb 改过一次的执行器）严格跑法判得出、与实现在取样的批态上一致、阳性对照有判别力（第一次跑判不出与判别力不足那两笔留在实验页）。
- G4（无读故障的崩溃态上每块盘至少一份自证过的系统配置）：代码轮判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 j2-b 立、第 29 批实现，射程与 mkfs 那一段见 `research/prompts/m3-formal-code-b29-r1-main-verification.md` 第二节 L3 与第三节第 3 条。
- 三个 oracle 的分工与各自独立性的来处是验证体系的结构设计；独立性靠「三 oracle 合谋测试」观测（E171（三oracle合谋测试））。
- E171（三oracle合谋测试）：在一棵冻住的树上，坏实现表里有三条 oracle 全部放行的行、有只靠一条才抓到的行，O3（独立规约执行器） 在坏实现表上一行没红；手造的伪造根、伪造续接镜像里有 O2（独立解析器 + checker） 放行而 O3（独立规约执行器） 判不在的（读数与限定在实验页 Q1、Q2、Q3 三行）。
- 三方论证：D13（验证路线） 新立那一轮（本地 LLM / Sonnet / Opus 各一条腿，立场不重复），材料没有留存，经过在变更史。
- O1（内存模型对拍（HashMap 理想文件系统）） 与 RefFS 的分工句：三方判决 `research/prompts/m3-formal-r5-main-verification.md` 第二节另起与单独一个包那一格（两腿同向）。

**欠**：C445（模型对拍的取样点不随写路径增长）；C446（未实现的不变量没有前置登记，也没有到期）；C447（规约执行器没接进崩溃点重放与 RefFS 比对）；C407（三 oracle 合谋测试的判官没进门禁）：E171（三oracle合谋测试） 已量过一次，没有一格门禁读它的产物；C672（交回值或内存状态错而写流不变的一类）；C679（卸载记号没进模型对拍）

```

**出处 `.claude/kb/decisions/13-验证路线.md:199-217`（整段抄，未转述）**

```markdown
#### 已定项 7：O2（独立解析器 + checker） 判的是**一个**镜像

**定案**：

> **定案：O2（独立解析器 + checker） 的定义域是单个镜像**——一个一元谓词 `P(镜像) → 成立/不成立`，
> 只判集合成员，永不判 identity。**任何需要第二个输入（另一个镜像、或一段记录流）的检查都不属于它。**

**需要第二个输入的核对归记录核对器**，入参 `(崩溃前镜像, 记录流, 崩溃后镜像, 持久集合)`——持久集合是枚举器给出的这个崩溃状态里哪些写落了盘（崩溃点重放的段内整写子集）；只看盘上字节分不开「写没落、而原地内容恰好相同」与「写落了」，判复用与缺席要拿持久集合判；它与自研第 1 项「Merkle 根链差分」（已定项 8）同属二元检查。⚠️ **故意不给它编号**——O2（独立解析器 + checker） 那场混乱就是编号当称呼造成的，再加一个 O4 只会给下一次漂移多一个落脚点。⚠️ **「崩溃后镜像」是两份，不是一份**：harness 的顺序是先跑实现自己的恢复、再跑记录核对器，而恢复本身会改盘（实例切换写行、重发在飞 checkpoint，D23（journal 的角色与格式） 已定项 14）⇒ 择根与前缀判定看**崩溃态镜像**（「记录流」就是它环里的 journal 记录），比对的对象是**实现恢复后的镜像**。

**射程**：指针层目标态下核对一条记录只需格式解析 + 校验和 + 根环择新；逻辑意图记录不点名任何盘上对象，核对方要自行实现树语义 + 记账 + 分配合法性——D23（journal 的角色与格式） 否掉逻辑意图日志的第二条理由据此改写成第一条的一条封堵，写在 D23（journal 的角色与格式） 已定项 5。记录核对器的代码在 `crates/singlefs-checker-tier/src/crash.rs`，接在崩溃点重放的每个崩溃状态上（门禁 checker-tier-crash-point-replay）。记录核对器的判定因此不只是镜像的函数：崩溃后镜像逐字节相同、持久集合不同的两个状态，它可以判得不同，按镜像去重的提速对它不适用（O2（独立解析器 + checker） 与 oracle 照旧只看镜像）。 崩溃点重放不覆盖可写挂载：崩溃点重放在每个崩溃状态上只跑只读恢复（看 journal 与不看各一遍），取号、写行、暖机在崩溃点重放的任何崩溃状态上都不跑。可写挂载由崩溃注入覆盖（`crates/singlefs-checker-tier/src/crash_injection.rs`）：每个抽到的崩溃状态上，只读恢复与判定之后在同一份崩溃后镜像上起一次可写挂载、再发一次布、跑池级 checker；挂载途中再崩一次，取号、写行、暖机三段各摆一个二次崩溃状态，每个上只读恢复、问模型、池级 checker、记录核对器：第三截交给记录核对器的写表从「挂载那一段到当前段为止的前缀」换成整条历史录制流 + 整条挂载流 + 挂载之后那次发布，持久集合逐段对应（历史那一段取第一次崩溃的持久集合，挂载那一段按二次崩溃取子集，之后那次发布全没落）。「比对的对象是实现恢复后的镜像」那一半：记录核对器改收两份镜像，崩溃态镜像判根在而记录在不在（择根与前缀），恢复后的镜像判单元在不在、读落到那一版的实例表；第二截接的恢复后镜像是挂载与那次发布之后的池，恢复自称的那一版取挂载与那次发布写出的最新那条根，挂载一条根都没写出时退回第一次崩溃落到的那一版。

**依据**：
- 无实验：**依据是类型，不是成本**：「逐字节比对」是二元关系 `R(镜像₁, 镜像₂)`，**没有任何投入能把一元谓词变成二元关系**；I-4.5（已停用·需两镜像） 因同一条门槛退役。
- 「崩溃后镜像」是两份：随 D16（发布语义） 已定项 4 收窄问法那一轮三方论证（材料 `_d16-items134-background.md`）由用户定案，记在变更史。
- 持久集合那一样入参：用户定案，原话在变更史；判别子观测不到的实据在 `research/prompts/m2-layer0-scale-r3-main-verification.md` 第二节按扇区豁免那一行。
- 崩溃点重放不覆盖可写挂载、可写挂载与二次崩溃由崩溃注入覆盖：用户定案（代码审阅第 2 条，2026-09-27 弹窗选「崩溃注入层补可写挂载」），记在变更史；崩溃注入那一侧的实现在 `m2-rev-b3b-implementer-report.md`。

**欠**：无。

```

**出处 `.claude/kb/decisions/13-验证路线.md:218-241`（整段抄，未转述）**

```markdown
#### 已定项 8：自研的两样：Merkle 根链差分与代际增量 scrub

**定案**：现成工具没有、本工程自研的两样：

**1. Merkle 根链差分 —— COW + 每事务发新根独有的杠杆。**
历史根在被回收前物理存在过，构成一条可回放的链。对相邻两代根递归比较「子树指针 + 校验和/MAC」，
**指针没变的地方直接跳过不下钻**，复杂度只正比于两代之间实际改动的部分。
它能检出常规遍历式 checker 检不出的一类错误：**同步篡改后自洽的原地写**——
某次写绕过 COW、原地改了块内容并同步更新了校验和，单点看它是自洽的（I-2.1（校验和与内容匹配） 全绿），
而根链差分会发现矛盾：**没走 COW 就不该有新分配，父指针理应没变，校验和却变了**。
⚠️ **必须配阴性对照**：正常 COW 提交后跑同一检查必须全绿，否则它会把每次正常更新都当成篡改。

**2. 代际增量 scrub。** 靠 I-1.2（块头写序已发布） 与 D5（快照 / 空间记账机制） 的 birth txg
维护一个「上次 scrub 扫到的 generation 水位」，只遍历 `birth generation > 水位` 的块——
老子树的校验和链没变就必然还自洽。**它让「每次提交都全量验证本次改动」在挂钟上可行，不必抽样。**

**射程**：被搬过的副本 `birth` 不变、会永远落在水位之下，代际增量 scrub 看不见它们。`crates/` 里没有根链差分（`grep -rn 根链 crates` 零命中），也没有代际增量 scrub（`grep -rn scrub crates` 零命中）。

**依据**：
- 无实验：两样都靠 COW 每事务发新根与 birth txg 的结构性质成立，都还没有实现，没有可量的量。
- 三方论证：D13（验证路线） 新立那一轮（本地 LLM / Sonnet / Opus 各一条腿，立场不重复），材料没有留存，经过在变更史。

**欠**：C389（Merkle 根链差分零代码）；C194（代际增量 scrub 看不见被搬过的副本）；C408（代际增量 scrub 零代码）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:242-275`（整段抄，未转述）**

```markdown
#### 已定项 9：崩溃点重放要分三层，否则挂钟必炸

**定案**：

| 层 | 范围 | 触发 |
|---|---|---|
| 冒烟层 | 固定种子、固定脚本的负载与登记的崩溃枚举用例，不按写请求数限规模（规模见射程），精确全量 | checker 档的全量（已定项 15）：用户要求时跑；提交时默认只跑快档 |
| 常规层 | 中等规模负载（写请求数上千），精确全量，枚举不完靠精确剪枝，一时跑不了的节点标 ignore，覆盖报告与门禁逐个列「本次未跑」 | 每次提交 |
| 大负载层 | 常规层 负载 + 大规模负载 + 多设备档位交叉，N≥5 轮，精确全量 | 用户要求时 / 发布前 |

**剪枝加 ignore 之后敢说什么**：敢说「开着的节点在全部崩溃状态上通过」；
**不敢说**「ignore 的节点通过」。

**全量档提交时不默认跑、不限时刻，由用户要求时跑；提交前 = 快档全绿（必要条件）。**
**里程碑收尾、动了崩溃点重放或判器的那一批提交之前，由主 agent 弹窗请用户跑一趟全量；跑红或用户不跑，提交照样可以进，记一笔未结的红或「这一阶段收尾没跑全量」，直到修掉或补跑。**
「全量」指两样：checker-tier-crash-point-replay `--full`（门禁 checker-tier-crash-point-replay，已定项 15 的全量）与崩溃放量全量（`scripts/crash/crash-amplification.sh` 不带参数）。
「动了崩溃点重放或判器」按崩溃判定库现算：这一批改动让库里有判定块的单元键对不上、对不上的那几把键里有判器单元（判器入口 fn 的单元闭包里的函数单元）变了，就算；阶段把作废块数与变了的判器单元清单打出来，主 agent 照它弹窗，不按路径清单判。
「上一次全量」由主 agent 写进当前里程碑记录一行：日期、两样各自的结论。
**全量档必须优先保证「失败可复现」而不是「覆盖率最大」**——修一个全量档 bug 的挂钟 ≈ `log2(N) × 一趟全量的挂钟`，一个不可复现的全量档失败价值接近 0，
甚至为负，它训练人忽略红灯。

**射程**：只有冒烟层（门禁 checker-tier-crash-point-replay）；冒烟层分两档：平时快档每段原地写取任意子集、COW 写只取全落或全不落（用户 2026-09-26 定，叫「甲二」，在 C561（记录核对器的复用豁免在复用只落一半时假红） 修好之后用），它看不见只在 COW 部分落盘时才显出来的那一类，替不了全量；全量带断点续跑（门禁 checker-tier-crash-point-replay `--full`），登记在 `.claude/gate.d/stage-inputs.tsv` 的崩溃枚举用例已经长到并行线一那条流 12230590578 个状态，不再是「写请求数两位数」的冒烟体量，所以提交时默认只跑快档，全量按已定项 15 由用户要求时跑；常规层、大负载层在门禁里没有阶段。每个崩溃状态怎么定义归已定项 4；节点怎么精确剪枝、ignore 节点什么时候开归已定项 10、11。

**依据**：
- 无实验：分层是门禁挂钟的设计约束，常规层、大负载层 的负载与挂钟都还没有，没有可量的量。
- 三方论证：D13（验证路线） 新立那一轮（本地 LLM / Sonnet / Opus 各一条腿，立场不重复），材料没有留存，经过在变更史。
- 平时快档用甲二：用户定案，原话在变更史；崩溃点重放规模三轮判决 `research/prompts/m2-layer0-scale-r3-main-verification.md`。
- 用户定案：「我们不是抽样 我们是剪枝 加 初期ignore」（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 第三节「用户定案：不抽样，剪枝加初期 ignore」那一行）、「枚举可以设置 ignore 标记 然后慢慢开启」（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 第三节「用户定：ignore 标记慢慢开」那一行）、「1 该枚举该设计的设计完 设计全」「2 枚举不完 正是剪枝的用处」（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 第三节里材料员被门禁 doc-process-records 的 implementation-premise 格挡下、用户定『设计全、枚举不完靠剪枝、粒度合适』的那一行）；第一轮判决 `research/prompts/m3-prune-gpu-r1-main-verification.md`「回看决策」一节：已定项 9 推翻。
- 用户定案（2026-10-01）：全量档不限时刻，用户要求时随时跑，删掉「夜间」，原话在变更史。
- 用户定案（2026-10-01）：全量档全绿的时点是里程碑收尾与动了崩溃点重放或判器的那一批提交之前，由主 agent 弹窗请用户跑，原话在变更史；交用户那一格是三方判决 `research/prompts/test-script-align-r1-main-verification.md` 第二节 t1-3。
- 三方判决 `research/prompts/test-script-align-r2-main-verification.md` 第二节 U3（「U3 t1-3 写回照字面做不了」那一行）：「全量」的两个所指、「动了崩溃点重放或判器」的路径判法、「上一次全量」的记法，「准入条件」与「异步」合成一句。

**欠**：C409（常规层、大负载层在门禁里没有阶段）；C635（上一次全量没有机器可读的标记）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:276-305`（整段抄，未转述）**

```markdown
#### 已定项 10：真正必须精确覆盖的 N 不是「布局套数」，是「提交协议结构的等价类数」

**定案**：常规层 精确覆盖先缩问题，再谈剪枝。崩溃点的定义域是**事务层状态机**，而 `.claude/rules/fs-design.md` 有两句顶层规则：
「一个事务层，所有结构共用」与「唯一不许多样化的是事务层」。所以布局能改的只有两类：

| 差异 | 改变的是 | 判定 |
|---|---|---|
| 节点大小、条带宽度 | 一步写多少字节、某步重复几次 | **参数**，不新增步骤种类 |
| 旋转 vs SSD | seek 代价、IO 调度 | 落在设备层，**不落在事务层协议里** |
| **zoned（ZNS）** | **能不能原地覆盖根指针** | **结构性**：只能追加、根位置漂移，且多出 zone finish/reset |

**落地手段是一个封闭枚举 + 穷尽 match**：`crates/singlefs-core/src/transaction.rs` 的 `CommitStep`，步骤种类的登记位是 [layout/01-first-txn.md](../layout/01-first-txn.md) 八的段序列登记表。
任何布局要引入新的结构性步骤**必须改这个共享定义 ⇒ 编译器强制所有引用处补分支 ⇒
这个 diff 门禁 grep 得到**，可以自动打标「本次引入了新的结构等价类，需为它新开一轮独立大负载层 重放」，
**不靠人记得申报**（按 diff 算门禁范围欠在 C8（门禁范围判不出来））。再加 newtype 隔离（`RotationalPtr` / `ZonedPtr`），
把 D12（目标介质） 已定项 2 里「不许共用」那一桶（记账事实的判定路径）变成编译期约束。
⚠️ **它有第二个职责（D17（实现分层与第三方管道） 加的）：它是第三方管道的接入点。**
协议归本工程所有，一条管道要引入新的结构性提交步骤必须往本仓发这个 diff。

**射程**：旋转与 SSD 在提交协议层面同构，由 D12（目标介质） 那一轮三方论证承担；zoned 那一行只说明它至少多出一个等价类。等价关系按录制流的段序列同构、有几个，归 D17（实现分层与第三方管道） 已定项 2。按种类串认节点够不够，第一轮判决打中（挂起，待第二、三轮攻）：`research/prompts/m3-prune-gpu-r1-main-verification.md`「逐格判决」一节。
⚠️ **它钉住的只是「有哪些步骤」，钉不住「每一步的失败原子性宽度」**——
持久内存与对象存储改变的是每一步的失败原子性宽度，枚举接不住（缺口与补法见 D17（实现分层与第三方管道））。
`RotationalPtr` / `ZonedPtr` 这层 newtype 隔离 `crates/` 里没有（`grep -rn "RotationalPtr\|ZonedPtr" crates` 零命中）。

**依据**：
- 无实验：N 是什么由 `.claude/rules/fs-design.md` 那两句顶层规则推出，没有可量的量；等价关系怎么判、有几个的依据在 D17（实现分层与第三方管道） 已定项 2。
- 三方论证：常规层 抽样那一轮（「三方论证 + 实测验算」），材料没有留存，经过在变更史。

**欠**：C8（门禁范围判不出来）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:306-340`（整段抄，未转述）**

```markdown
#### 已定项 11：ignore 的节点什么时候开：否决按欠账轮转的账本，留下三条

**定案**：

**「按欠账轮转的账本」调度：否决。** 格子 = `(布局, 子阶段桶)`、按欠账降序取队首、声称任一格子最多 `K_max = ⌈N×M/C⌉` 次提交必被覆盖的那个调度被否决：它保证不了的恰恰是它要防的那一类——慢的、罕见的分支（实测验算与另外四条否决理由在变更史）。

**留下的三条**：

1. **方向对**：要一个**确定性的、能被门禁判红的覆盖记录**，而不是概率论证——
   概率保证在长期维护里等于没有保证。

<!-- doc-lint:registry name-col=2 -->

| 符号 | 简称 | 含义 |
|---|---|---|
| T1 | 常规层 挂钟预算 | 常规层 本地挂钟预算，是输入不是输出 |

2. **`T1`（常规层 挂钟预算） 是输入，不是输出**（`.claude/rules/fs-design.md`：门禁运行时长本身是设计约束）。
   正确顺序是「先声明预算 → 用它约束一次提交能开几个 ignore 节点」，不是「量出耗时 → 推出能开多少」。
   **这一条不需要任何测量，现在就该定。**
3. **判别力必须与覆盖分开检查**：每个 ignore 节点被打开的那次提交里，额外对一份「已知坏的镜像」跑它自己的 checker，
   不报错即判红——这是 D13（验证路线）已有的阳性对照纪律的直接复用。见 C13（checker 判定失效）。

门禁里没有「常规层 崩溃点重放」这个阶段，量不出 `T1`（常规层 挂钟预算） / `C` / 每格耗时。
⚠️ **因此 `C`、`K_max`、`T1`（常规层 挂钟预算） 这三个数一个都不许现在写进 kb。**

**射程**：只管 ignore 的节点什么时候开；冒烟层全量枚举不需要 `T1`（常规层 挂钟预算）。第 2 条的「现在就该定」与末句「一个都不许现在写进 kb」说反话，记在 [verification-build.md](../verification-build.md)「待定案」第 6 问，没判。节点登记表带状态字段（开着 / ignore / 等实现），另加三条——每个 ignore 行点名一个会变的对象、那个对象变绿而节点仍 ignore 时判红、覆盖报告列 ignore 的状态数与占比：这三条被攻过零轮（`research/prompts/m3-prune-gpu-r1-main-verification.md`「逐格判决」一节），不写成定案，等第二、三轮再攻。

**依据**：
- 无实验：否决账本靠的是一次调度规则的模拟验算（200 轮、取后半程稳态最大值、严格前缀贪心，三行数在变更史），没有登记成实验；留下的三条是门禁设计原则，没有可量的量。
- 三方论证：常规层 抽样那一轮（「三方论证 + 实测验算」），材料没有留存，经过在变更史。
- 用户定案：「枚举可以设置 ignore 标记 然后慢慢开启」（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 第三节「用户定：ignore 标记慢慢开」那一行）；第一轮判决 `research/prompts/m3-prune-gpu-r1-main-verification.md`「回看决策」一节：已定项 11 推翻，「逐格判决」一节给节点登记表带状态字段的形态与三条待攻的补充条件。

**欠**：C8（门禁范围判不出来）；C13（checker 判定失效）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:341-356`（整段抄，未转述）**

```markdown
#### 已定项 12：冲突 1：`checker 即规范` 与 `校验的两条路径不许共享工具`

**定案**：**冲突 1：`checker 即规范` 与 `校验的两条路径不许共享工具` 直接打架。**
在用的不变量（条数的权威登记位在 [invariants/README.md](../../../crates/singlefs-checker/invariants/README.md) 文件头）若既是 checker 源码又是形式验证的 spec，就成了单点。
**可行的折中，而且本工程已经付过这笔钱**：checker 做两个方向——**遍历式**（从系统配置走下去）
与**扫描式**（全盘扫，靠「元数据块必须自描述」的块头重建），两者对同一条不变量独立判断，
共享的只有格式解析。

**射程**：管 checker 内部两个方向之间共享什么；checker 与实现之间共享什么归已定项 5。池级 checker 两个方向都有（`crates/singlefs-checker/src/walk.rs` 文件头）。

**依据**：
- 无实验：这是「checker 即规范」与校验路径独立之间的取舍，没有可量的量。
- 三方论证：D13（验证路线） 新立那一轮（本地 LLM / Sonnet / Opus 各一条腿，立场不重复），材料没有留存，经过在变更史。

**欠**：C410（checker 两个方向共享什么没有检查）：两个方向「共享的只有格式解析」这一条没有会红的检查（`.claude/gate.d/` 里没有判它的阶段）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:357-377`（整段抄，未转述）**

```markdown
#### 已定项 13：冲突 2：crash refinement 的 spec 与模型对拍的 model

**定案**：**冲突 2：crash refinement 的 spec 与模型对拍的 model 是同一个东西的两种写法。**
合并成一份 → 省一半工作量，但**直接违反独立性要求**；各写一份 → 名义独立，
但通常是同一个人同一周写的。缓解（只降低相关性、不消除）：让两份**在表述层面就不同**——
spec 写成「对镜像的状态谓词」，model 写成「op → 状态转移」，不一致时至少能互相定位。
**同源性要被观测**：统计「spec 与 model 不一致」的事件数及每次判定哪一方错；
若累计 ≥ 5 次不一致而判定「spec 错」的次数为 0，说明两者实际同源，应把两条腿合并、预算移给穷举。
判定分三格：spec 错 / model 错 / 条款欠一句（两边各照条款的一种读法，条款没写谁对）。一次不一致先在前提成立的域里重判，还在的才记；「条款欠一句」的事件先记账，不计入「累计 ≥ 5 次」，也不计入「spec 错」，条款补了那一句之后按补的句子再判一次、归到前两格。
这里的 model 指 O1（内存模型对拍（HashMap 理想文件系统））。O3（独立规约执行器） 能对同一（历史、崩溃点）交出判定行之前（第一版 Python 执行器 E170（O3规约执行器第一版） 已有，没接进崩溃点重放，C447（规约执行器没接进崩溃点重放与 RefFS 比对）），O3（独立规约执行器） × RefFS 这一对登记为空格、不计事件；O1（内存模型对拍（HashMap 理想文件系统）） 的崩溃判官不当替身；没立的对报「未立」，不报绿、不计入任何一格。

**射程**：缓解只降低相关性、不消除。已定项 1 撤下的那条原候选不受这条影响，理由在已定项 1 的射程。crash refinement 的代码是 `crates/singlefs-replay-prefix/src/lib.rs` 里的 Verus 核与它的 spec（取前缀、择根、扫环三截与择根和取前缀的合判推论，覆盖已定项 1 的 M1、M2 句法层与 M3 的两读和单调推论；M2 语义层与 M4 靠 E165（恢复重放有界穷举与变异杀伤界） 在界内判）；spec 不能对具体状态求值，「spec 与 model 不一致」的事件从哪来没定，计数器没有代码（C191（同源性计数器只盖了一对））。

**依据**：
- 无实验：同源性要靠将来 spec 与 model 的不一致事件计数观测，spec 与计数器都还没有，没有可量的量。
- 三方论证：D13（验证路线） 新立那一轮（本地 LLM / Sonnet / Opus 各一条腿，立场不重复），材料没有留存，经过在变更史。
- 第三格「条款欠一句」：三方判决 `research/prompts/m3-formal-r1-main-verification.md` 第二节「F2 同源性」一行（攻方打中，推的）、`research/prompts/m3-formal-r2-main-verification.md` 第二节第 9 行、`research/prompts/m3-formal-r3-main-verification.md` 第二节同源性计数器一行，三轮站住；无实验：计数器仍没有事件来源。
- 「model 指 O1（内存模型对拍（HashMap 理想文件系统））」与没立的对登记为空格：三方判决 `research/prompts/m3-formal-r5-main-verification.md` 第二节同源性计数器那一格（两腿同向的部分）；`research/prompts/m3-formal-r4-main-verification.md`「用户定案」里替身对那一行（不算替身对）。

**欠**：C191（同源性计数器只盖了一对）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:378-396`（整段抄，未转述）**

```markdown
#### 已定项 14：与其他决策的连锁

**定案**：

| 决策 | 连锁点 |
|---|---|
| D4（校验和位置） | 根链差分把 Merkle 从「要付的代价」翻转成验证杠杆；但形式验证里哈希是 uninterpreted function，**碰撞抗性完全在验证范围之外，不许读成「验了 Merkle」** |
| D8（核心索引结构） | 日志结构节点、write buffer、key cache 三层都会绑架符号执行（循环上界、滞后窗口、缓存一致性），使「Yggdrasil 一分钟内」的口径在本工程不成立 |
| D9（加密） | AEAD 在 SMT 里不可解，只能建模成 uninterpreted function，**「nonce 重用泄漏明文」这个后果验不出来**，只剩「nonce 不重复」这条纯组合性质——而这条纯组合性质恰是这条腿最值钱的产出 |
| D11（索引节点要不要留消息缓冲区） | 已定项 1 取「留」；D8（核心索引结构） 已定项 1 取幂等完整值之后 checker 的 I-3.1（已分配统计对得上） 不必解析消息语义（D11（索引节点要不要留消息缓冲区） 据此判它的代价 1 在本工程不成立），冲突 1 不因此变成「无解」 |

**射程**：连锁的是形式验证与 checker 碰到这几条决策时的边界，不改那几条决策的条款；根链差分与冲突 1 本身归已定项 8、12。

**依据**：
- 无实验：连锁是从那几条决策的已定条款推出来的边界，没有可量的量。
- 三方论证：D13（验证路线） 新立那一轮（本地 LLM / Sonnet / Opus 各一条腿，立场不重复），材料没有留存，经过在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/13-验证路线.md:397-409`（整段抄，未转述）**

```markdown
#### 已定项 15：验证代码分两档——harness 档随时跑，checker 档默认只在提交时跑

**定案**：`crates/` 下的验证代码按「什么时候跑」分两档，各住一个包，与池级 checker 一共三个名字：**harness 档**是 `crates/singlefs-harness`，单元测试与集成测试加上它们用的脚手架库（录制器、内存池、理想模型、随机历史、故障注入设备、场景），改了代码随时跑；包内再分轻重：debug 下单条跑到 60 秒及以上的是重，标 `#[ignore]`，单条用时看平常跑的时候量到的、不另外单独量，要跑随时跑，一律经内存包装；一条用例一个场景，测试文件按测什么起名、不按里程碑起名。**checker 档**是 `crates/singlefs-checker-tier`，崩溃态枚举引擎与记录核对器、崩溃枚举用例经判定库的两段与它的计数、发现日志与工具链身份、一份长期判定库（块键只由内容定，几类测试各自要哪些块只当索引）与两段协议、崩溃注入与坏盘输入两场战役、真设备一侧的设备日志比对、实验与真设备装置二进制，以及它们的全部用例，连同 QEMU（门禁 checker-tier-qemu-device-streams）、herd7（checker-tier-lkmm）、crates 变异整表（checker-tier-crates-mutation-replay）、全部实验复跑（checker-tier-research-build-and-replay 的 experiment-replay 格）：默认只在提交时跑，能单独跑（命令带 `SINGLEFS_HEAVY_TESTS=user-request`）。checker 档里的崩溃点重放（门禁 checker-tier-crash-point-replay）分两档，checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-research-build-and-replay 不分：**快档**是两趟：`cargo test --release -p singlefs-checker-tier --lib --tests` 不带 `--ignored`（`--tests` 带上全部 `test = true` 的目标，装置二进制的内联单测也在里面），再一趟带 `tests/` 里 `cfg` 点到的特性（`--features <特性> --lib --test <挂特性的测试文件>…`，特性里有 `gpu` 的这一趟经 `scripts/crash/gpu-quiesce-run.sh` 起）（门禁 checker-tier-crash-point-replay 在整轮门禁与提交时都跑这两趟，第二趟是它的 feature-gated-tests 格），**全量**是登记在 `.claude/gate.d/stage-inputs.tsv` 键为 `crash-case:` 的用例逐条 `--include-ignored --exact`（checker-tier-crash-point-replay `--full`），提交时不默认跑，由用户要求时跑，跑法按已定项 9 分层；崩溃放量判器整套上了 GPU，罩全部池级 checker 不变量（`crates/singlefs-checker-tier/src/crash_judge_gpu.rs`）。**池级 checker** 是 `crates/singlefs-checker`，判决器库，被两档调用。依赖只许一个方向：checker 档依赖 harness 档与池级 checker，harness 档不依赖 checker 档（门禁 checker-independence-and-sync 的 checker-implementation-disjoint 格第 ④ 条判，dev-dependencies 也算）；两档都要的一小段代码宁可各留一份。崩溃枚举用例一律写在 `crates/singlefs-checker-tier/tests/` 里（`scripts/crash/crash-case-check.py` 判），重型测试闸按包判：跑到 `singlefs-checker-tier` 的测试就是 checker 档。

**射程**：管 `crates/` 下代码与测试住哪个包、什么时候跑、闸按什么认、三个包怎么称呼（`.claude/rules/verification.md`「定义与名字」）；不管每条不变量判什么（[invariants/README.md](../../../crates/singlefs-checker/invariants/README.md)）、不管枚举域怎么定（已定项 4）、不管池级 checker 与实现共享什么（已定项 5：池级 checker 仍只依赖 `singlefs-format`）。上游 SOP 不再写这个项目的验证手段：崩溃点重放、模型对拍、checker 即规范、文件系统特有的反推缺口四节与 `gate.sh` 未实现清单里的项目键都收进本仓（`.claude/rules/verification.md`、`.claude/gate-not-implemented.tsv`），上游那一批已同步进副本、未发版。拆分那一轮的记录在 `records/2026-09-27-验证两档拆分.md`。

**依据**：
- 无实验：什么时候跑、住哪个包是流程政策，没有可量的量；两档各自的判别力由各自的用例与变异表证。
- 用户定案（2026-09-27），原话在变更史；三方 `verification-split-r1`（判决 `research/prompts/verification-split-r1-main-verification.md`）攻的是拆出 `singlefs-checker-tier` 之前的那一版，拆包之后的改动被攻过零轮。
- 用户定案（2026-10-01）：全量不限时刻，用户要求时随时跑，删掉「夜间」，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/13-验证路线.md:410-425`（整段抄，未转述）**

```markdown
#### 已定项 16：按段登记的段整域装不进 u64 就分片判，态数用 1280 位定宽整数

**定案**：
- 分片：崩溃放量里按段登记的段（`.claude/gate.d/crash-point-replay-segments.tsv`，一行一段五列：项名、流名、段号、段写数、段写表摘要），整域态数装不进 u64 时按固定大小的片依次判；片大小由环境变量 `SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES` 定，没设取 2^32 态；片起点是判定库里的续点（续点按段写表摘要与判器单元键寻址，项名、流名、段号只作显示）。整域装得进 u64 的登记段不分片，整域一次判。代码落点：`crates/singlefs-checker-tier/src/crash_amplification.rs:14800` `REGISTERED_SEGMENT_SLICE_STATES_ENVIRONMENT_VARIABLE`、`crates/singlefs-checker-tier/src/crash_amplification.rs:14947` `fn segment_slices_of_the_recording_from_the_environment_and_the_library`。
- 位宽：登记段的总态数、片起点、片尾与进度一律用 1280 位（10 个 u128）定宽无符号整数 `StateCount1280Bits`（`crates/singlefs-checker-tier/src/crash.rs:1640`，肢数是同一文件的 `STATE_COUNT_LIMBS`），算术全部 checked，不饱和、不截断；驱动行 `registered_segment_progress=` 的已判态数与总态数都写精确十进制，不写幂的形态。
- 登记表怎么交给引擎：引擎只经环境变量 `SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY` 读按段登记表（`crates/singlefs-checker-tier/src/crash_amplification.rs:14764`，在 `fn registered_segments_of_the_flow_from_the_environment` 里），没设就当一段都没登，没有别的退路。144 个单元的文件顺序写成 145 个、extent 下段长成两层的那条流（引擎流名 `position_addressed_ExtentLowerSegmentGrowsToTwoLevels`）的单元写段（322 次单元写）与记录段（290 次 journal 记录原地写）两段都登。

**射程**：管崩溃放量引擎对按段登记的段怎么分片、态数用什么整数记、登记表从哪里读；不管哪几段该登（由用户点头，写在登记表文件头）、不管登记段的域本身怎么展开（判决 `research/prompts/m3-crash-enumeration-merge-r3-main-verification.md` 的 J10）、不管每个崩溃状态怎么定义（已定项 4）。没登记的段的态数、与整条流合起来的态数仍要装得进 u64，装不下在写库之前拒跑；登记段整域越过 1280 位同样在写库之前拒跑。片大小只许 1 到 2^32：两个 GPU 内核把片内序号当 u32 加到片起点上（`crates/singlefs-checker-tier/src/crash_amplification.rs:14807` `LARGEST_REGISTERED_SEGMENT_SLICE_STATES`）；轻型（`scripts/crash/crash-point-replay.sh --quick`）调用方没设片大小时设成 `LIGHT_REGISTERED_SEGMENT_SLICE_STATES`。展开上限盖不住又没登记的段，放量入口 `scripts/crash/crash-amplification.sh` 判红；崩溃点重放入口、放量入口与分项驱动开跑前核这份表再送给引擎。

**依据**：
- E161（崩溃放量的去重与分段耗时）「第 2 次重跑：取样域按 11 段的流重定（2026-10-05）」一节：段的全量态数与闭式、装置逐个数、crates 计划逐段相等，态数随段写数按幂长，撑的只是「长段整域装不进 u64」这个前提；片大小、位宽取值与登记表的读法是工程参数，用户定，它不撑。
- 用户定案，原话在变更史：分片与每片 2^32 态（`records/2026-10-11-崩溃放量两段与脚本审计.md` 第 96 行）、位宽 1280 位（同一份记录第 98、99 行）、两段都登（同一份记录第 92 行）。
- 分片与位宽是对判决 `research/prompts/m3-crash-enumeration-merge-r3-main-verification.md` 的 J10（按段登记）的补充，没经三方攻过。

**欠**：无。

```

**出处 `.claude/kb/decisions/13-验证路线.md:426-446`（整段抄，未转述）**

```markdown
#### 已定项 17：判定复用的单元是函数，键只由函数单元、工具链、依赖版本组成

**定案**：
- 重型门禁、崩溃判定库与实验准入的每一份判定（崩溃状态块、快档与模型对拍的每条测试函数、crates 变异每行、广谱变异每条、形式化每条、QEMU 每档、litmus 每份、研究复跑每行、实验变异表每条、实验准入）沿不沿用，只看它的单元键：它依赖的函数单元各自去注释后的指纹，加闭包里各单元（含 mod 头）的 `#[cfg]`、`#[cfg_attr]`、`cfg!` 真引到的 cfg 原子、各带这一次目标下的真值（闭包没引到的原子不进键：带不带 `gpu` 特性编的两趟，闭包不提 `gpu` 的判定键相同、共用一份库），加工具链（含 cargo 解析后的生效构建环境与外部工具版本）、从 `Cargo.lock` 解析出的闭包真引到的依赖版本，加这份判定自己的内容输入（段写表与起步镜像摘要、litmus 内容、变异原文与替换文这类）。函数单元从这份判定的入口沿调用与引用收，连同引用的常量、类型、静态项与宏展开一起算；wgsl 按 fn 切。
- 一律不进键、也不许当复用依据：按提交、HEAD 或树算的改动范围；整文件原文；文件级、包级、整表级的指纹或闭包；手写的版本常量；路径（仓根也算）、名字、种子串、行号、资源上限、限时；只有整道或整格的标记而没有按判定的库。键过窄（引用的常量、类型、宏漏收）、库放在会丢的位置、改名丢库、两份库重复判同一批，同样算违反。
- 收不全（动态分派、宏、`include_str!(concat!…)` 认不出）的那一份拒算键、照跑，不静默少收；每把键连同组成它的单元清单落盘，作废时报得出是哪几个单元变了。
- 每类判定一份库，固定在持久目录，不随一趟、不随判法换目录；仓根也不是库的输入：崩溃判定库的目录由环境变量 `SINGLEFS_CRASH_AMPLIFICATION_LIBRARY` 给时只收绝对路径，相对路径拒绝；全量开关只写不读、现判的照键写回同一份库；第二台判出的并进本机那一份。
- 改键让旧库作废不算代价：不做兼容迁移、不为旧行补签，改完重跑。

**射程**：管重型门禁、崩溃判定库与实验准入的复用键；不管静态门禁「只判新加行」的改动范围（`scripts/gate/changed-paths.sh`，那是判据语义）与 preflight 的 `admission: inputs-changed`；不管枚举域（已定项 4）、分层与什么时候请用户跑全量（已定项 9）。镜像判定与给定状态判定的版本是判定定义那一组（CPU 参照判器 `crash::judge_crash_image`、`crash_given_states::judge_the_given_states_on_the_cpu` 的闭包）的键，不论这一块在 CPU 还是 GPU 上判的：GPU 判器是加速路径，它的准入靠与定义那一组的对拍（快档的 GPU 与 CPU 对拍用例），它的单元只进块属性当出处，不进键、不分种类；显卡驱动、卡型、几张卡怎么分活同样留作块的属性或跑的参数，不进键。

**依据**：
- 用户定案（2026-10-15），原话在变更史与 `records/2026-10-13-复用键与全仓改名.md`「2026-10-15 用户定：提交与重跑无关，重跑只跟代码变化走」一节、「2026-10-15 批3」那几段。
- 用户定案（2026-10-17）：判定不与硬件、不与 CPU 还是 GPU 绑定，复用与剪枝不看设备；原话在变更史与 `records/2026-10-13-复用键与全仓改名.md` 2026-10-17 叫停全量那几段。
- 主 agent 定案（2026-10-15）：仓根不进键与库、库目录只收绝对路径、cfg 成分只取闭包真引到的原子，出处同一份记录「2026-10-15 验证段第一步判红、两条定案、4-3 四批派出」定案第 1、2 条。
- 反证：E164（崩溃放量新设计的小范围功能测试） F4 量到只按函数取指纹、不连引用的常量一起收，漏判 V5 与 V11（V11 只改常量 `DATA_POINTER_BYTES`）：函数单元要连同引用的常量、类型、静态项与宏一起算。
- 盘上的数：`research/prompts/pruning-survey-2026-10-15/ledger.md`「一之二、盘上量到的」一节——只差名字的块各存各判 4100 块与 9679 块；整库一把的判法摘要一换累计多判 3 935 242 934 态，那次闭包里改动的 29 份文件有 24 份去注释后逐字相同：键带名字、按文件或整版取指纹，同一份判定会重判。
- 没有腿况记录：未经三方攻过；用户定改完代码开三轮攻方，只攻「剪枝是不是原子化的、键是不是只由函数单元组成」。

**欠**：C8（门禁范围判不出来）、C489（singlefs-core 的广谱变异对今天的代码没有结论）、C528（两条门禁路判的改动窗口不同）、C564（另外 6 条崩溃点重放全量没人跑）、C605（崩溃放量的复用与剪枝没有关掉再判的对照）、C630（提交时形式化那一道第 1 步停下的频率没量）、C635（上一次全量没有机器可读的标记）、C645（崩溃放量全量没有人能派）、C668（随机历史脚手架的引入者白名单能被别名写法绕过）、C777（五个实验键在 stage-inputs.tsv 没有登记行）、C836（staged-green 挂的树不一定是被判的树）。

```

**出处 `.claude/kb/decisions/15-format-freeze-policy.en.md:1-6`（整段抄，未转述）**

```markdown
## D15 (Format Freeze Policy) — Settled

[中文](15-格式冻结政策.md) · **English** · [日本語](15-フォーマット凍結ポリシー.ja.md)

D15 (Format Freeze Policy) governs two questions: **when it is permissible to state "format v1 frozen"**, and **what channel newly added items follow after freezing**. It is the common prerequisite for D11 (Index Nodes Message Buffers) / D12 (Target Media) / D14 (Dual-Track File Sizes and Lifetime) — the current value of the cost "format branch permanent" is zero, but infinite after freezing, and the true cost of those decisions is defined by it. Out of scope: what value each individual field should settle on belongs to its respective decision; before the first external user emerges, the format is soft, and that policy resides in `.claude/rules/format-evolution.md`; the semantics of each bit in `feature-bits.md` belongs to the decision claiming that bit, while D15 (Format Freeze Policy) only governs how bit numbers are allocated and whether they can be reclaimed.

```

**出处 `.claude/kb/decisions/15-format-freeze-policy.en.md:7-233`（整段抄，未转述）**

````markdown
### Settled Items

Items in D15 (Format Freeze Policy) have only a single set of numbers without reordering.

| # | Item | Settlement |
|---|---|---|
| 1 | **Specific handling of item 7 in pre-freeze checklist** | Item 7 does not write a list of decisions, but is written as a single predicate — each unsettled item across the repository declares whether it "touches the format"; if a freeze request intersects with it, judge red; default padding width is 0 **Status: Settled.** **Progress: No code object.** |
| 2 | **Whether the tree table is an open list** | Yes, made an open list from day-1 **Status: Settled.** **Progress: Implemented.** |
| 3 | **Source of `spec_hash`** | No switching. Invariably computed against `format-spec/<component>.toml`; that toml file is a complete projection of the kb field table, including row ordering and non-scalar on-disk semantic rules **Status: Settled.** **Progress: Unimplemented.** |
| 4 | **Bit allocation registry for feature bits** | Registry resides in settled item 4; incompat bit 1 = first pure-SSD layout line (the version with F and unmount mark), set upon mkfs, bit 0 retired; compat_ro and compat have zero bits allocated **Status: Settled.** **Progress: Implemented.** |
| 5 | **What the action of freezing constitutes** | Freezing = committing a file `.claude/kb/format-frozen/<component>.toml`; humans only decide whether to initiate it, while the machine decides whether the request holds **Status: Settled.** **Progress: Unimplemented.** |
| 6 | **Checklist that must be completed before freeze** | Seven prerequisites, re-judged upon each freeze; values for each item are taken fresh from the settlement line of the corresponding item, no manual copying allowed **Status: Settled.** **Progress: No code object.** |
| 7 | **How many layers freezing is split into, and what lies outside the four layers** | The four layers have strict dependency ordering (system configuration and block header, pointer field layout, key encoding, index node internal layout), plus three independent freeze components **Status: Settled.** **Progress: No code object.** |
| 8 | **What freedoms are lost after freezing** | Width, existence, and meaning of frozen fields can no longer change, only additions permitted; checker judgments on frozen portions also become compatibility contracts **Status: Settled.** **Progress: No code object.** |
| 9 | **Post-freeze change judgment table** | Nine categories of changes judged for compat / compat_ro / incompat and whether to bump, with the criterion being what old readers will do **Status: Settled.** **Progress: No code object.** |
| 10 | **Bit count, accounting, and allocation rights of feature bits** | Three bitmaps of 256 bits each stored in system configuration; allocation right is "first merged, first claimed", bit numbers strictly monotonic; once used, bits may never be reclaimed **Status: Settled.** **Progress: Implemented.** |
| 11 | **Two gates that can go red paired with freezing** | Format freeze contract (`spec_hash` comparison) and backward compatibility regression (golden images); image bodies do not enter git, checklist is append-only **Status: Settled.** **Progress: Unimplemented.** |

#### Settled Item 1: Target of Item 7 is Unsettled Items, Not a Decision List

**Settlement**: Three rules.

1. **The target is every still unsettled item across the whole repository, not a decision list.** Each unsettled item must carry a verdict statement — **touches the format or not**: whether its answer will modify any on-disk bytes or alter the **interpretation measurement basis** of existing bytes (the measurement basis aspect reflects the pattern of C97 (changing field semantics without altering width produces no red gate anywhere in repository): width does not change by a single byte, but semantics change). Three possible values: **touches / does not touch / boundary unsettled (treated as touches)**; **unwritten is treated as touches**.
2. **Judgment**: When a freeze request is initiated, any item judged as "touches" or not yet judged triggers a red gate. To freeze in batches, the request must **list each item by name and explicitly accept the risk** (pattern following settled item 7 "risks of batch freezing must be explicitly accepted"); **omitting any item triggers a red gate**.
3. **Default padding width is 0**, with the burden of proof on the party proposing non-zero width; it must pass the format branching criterion in `.claude/rules/fs-design.md` (verbatim: "whether a format branch is worthwhile depends on whether it can turn 'used incorrectly' into 'fails to mount'").

Framing determines interpretation (`.claude/singlefs-ai-sop/skills/decide/SKILL.md` hard requirement 7: phrasing of tentative items must have only one reading): instead of "pinning values across six cells item by item" (content of each cell lives in its respective decision; duplicating it creates dual records of the same fact), we adopt "defining the judgment rules for item 7"; item 7 is **re-judged upon each freeze**, because the checklist is a pre-commit check triggered by commits and runs with every freeze request.

**Scope**: Default padding width 0 governs **reserving placeholders for features settled as not done**; it does not govern **general reserved areas** within structures — general reserved areas are governed by the first row of settled item 9 judgment table (new field in reserved area / TLV tail => compat without bump) and C102 (inode header lacks reserved space making field additions non-compat), which have different criteria; conflating the two would deduce "reserve nothing anywhere", whereas C102 (inode header lacks reserved space making field additions non-compat) is repaying an owed check in the opposite direction. What elevates to a rule is solely the procedure of "default 0 + inverted burden of proof"; any entry reaching the branch "v1 does not include this feature" must pass the format branching criterion again on its own. Predicates are not partitioned by layer: exiting freeze is judged by structure, not by layer (scope of settled item 7), and membership is incomplete — the three independent freeze components in settled item 7 do not belong to any of the four layers => layer-partitioned predicates are undefined on them; partitioning by the binary predicate "touches format or not" avoids both defects and assumes no independent adjudicability between layers. The cell for D26 (Background Compaction and Placement Reclamation) settled item 4 remains owned by D26 (Background Compaction and Placement Reclamation) itself; D15 (Format Freeze Policy) has no authority to answer on its behalf. Judgment inputs are always fetched fresh and never cached here: `grep -c '未定\*\*' .claude/kb/decisions.md` counts how many unsettled items remain across the repository (item list line tails are currently formatted as "—— **未定** · 未实现", no longer ending with "未定**"), and `bash .claude/gate.d/doc-decisions.sh --check decision-items-sync` judges whether that list is a faithful projection of the body.

**Evidence**: No experiment: what is settled is the target and default values of gate judgment rules, with no quantifiable metric. Supporting it are two facts — list patterns leak entries (D22 (Synthesizing Unit Atomicity) settled item 9 is the **field table of system configuration**, and system configuration is Layer 1; D19 (Block Pointer Structure & Width Budget) settled item 6 is the **field table of central mapping entries**, residing in Layer 2 — neither was on that six-decision list, so the list had zero say when freezing those two layers), and relying on a "format-level" tag fails to catch anything (registry location of IDs is in the title line of each decision body, not on the index page; `grep -rn 格式级 .claude/gate.d/ .claude/scripts/` yields zero hits => zero objects, zero implementation). Settled by main agent based on three-way verification; an invalid argument is recorded in decisions history to avoid reusing it.

**Owed checks**: C62 (settlement values of items branch in kb); C102 (inode header lacks reserved space making field additions non-compat).

#### Settled Item 2: Tree Table Made an Open List from Day-1

**Settlement**: The tree table is made an open list from day-1. Evidence lies in the judgment table of settled item 9: under an open list, "adding a keyspace" means old readers merely **do not see the new tree** (compat grade, graceful degradation); hardcoding tree counts means old readers **cannot read the tree table** (incompat, immediate breakage). **Open lists strictly dominate**, with the only cost being an extra table length field plus a variable-length table section in the tree table unit — identical to the rationale in settled item 10 "reserves should lean generous". => Adding keyspaces is an explicitly stated expansion objective of this project, not an accident, so that row in the judgment table is pinned as **compat**.

**Scope**: What is settled is "tree table is an open list", regardless of where tree tables reside — tree tables reside in the root record indirection layer (D22 (Synthesizing Unit Atomicity) settled item 7 verbatim "tree roots invariably reside in tree table units, never flattened into root slots", settled item 12 settles "system configuration does not carry its own tree table unit pointer"; D21 (Boundary Between Authoritative and Derived States) groups root record + tree table unit + instance table unit together into "root"). It **must be incorporated when Layer 1 freezes**, and cannot wait until adding a tree reveals it was omitted; freeze layer membership for the tree table unit itself remains owed.

**Evidence**:

- E142 (new-pool file creation dry run): The tree table unit written during new-pool file creation declares entries by count (0 entries in mkfs version, 11 entries in transaction version, representing values before adding retired trees; 12 entries in transaction version after adding retired trees), table length is data rather than compile-time constant — the open list form holds on written bytes.
- User settlement, verbatim text in decisions history.

**Owed checks**: C246 (tree table unit lacks freeze layer membership).

#### Settled Item 3: `spec_hash` Invariably Computed from Complete Projection of kb Field Tables, Never Switched to Rust Structs

**Settlement**: `spec_hash` is invariably computed against `format-spec/<component>.toml`. That toml file is a **complete projection of the kb field table** — containing **row ordering** (on-disk sequence) and **non-scalar on-disk semantic rules** (merge rules for component 2 in settled item 7 are encoded as data), not merely the collection of registered scalar constants. What is added once code exists is **not changing the hash source**, but two gates: (1) **equality check** between spec and code (comparing both width and ordering); (2) golden image regression (settled item 11 rule 2).

Four reasons:

1. **Directionality**. D13 (Verification Route) settled item 5 verbatim: "**checker and implementation share only one artifact: a constants module generated from kb field tables**, with the generator reading from kb and both sides only consuming, no manual edits permitted by anyone" => authority resides in kb, while Rust structs are **downstream consumers**; computing hashes against structs anchors the fingerprint of a permanent contract on a derivative artifact.
2. **Hash is a change detector, not an equality check between two artifacts.** Computing hashes on structs cannot catch "structs inconsistent with kb"; it alarms only when structs **change**; what genuinely catches that class of error is an equality check, which requires the spec side to exist => switching across not only gains nothing extra, but tears down the opposite side required for comparison.
3. **Language binding**. D17 (Implementation Layering & Third-Party Pipeline) settled item 3 verbatim: "Verdicts must be independently verifiable by third parties, otherwise 'conformance' degrades into 'we claim you conform'". Anchoring fingerprints on Rust source text prevents third-party pipelines from recomputing them.
4. **Feasibility**. Spec files for the three components can be drafted immediately, allowing hash mechanisms to take effect long before the first freeze; whereas the freeze moment for component 1 in settled item 7 is the first image writing a Type 2 container => switching would be forced into the identical commit as the first real freeze, with zero burn-in period.

**Scope**: Projecting only scalars into the spec is insufficient; row ordering must be included, and equality checks must compare sequence — the unit header in [layout/01-first-txn.md](../layout/01-first-txn.md) lists six adjacent fields of width 8 in row order (the 5-tuple's tree ID / object ID / object birth txg / anchor offset, plus birth generation and fsid); transposing any two during serialization leaves all scalar constants and `DATA_UNIT_HEADER_BYTES` unchanged while on-disk bytes become completely misaligned, falling into the "changing field semantics" row of settled item 9 judgment table. The boundary in D13 (Verification Route) settled item 5 stating "generator emits only scalar values" governs shared artifacts between checker and implementation, whereas `format-spec/<component>.toml` is a **gate artifact** with zero runtime consumers => outside that boundary's scope, and a field table with row ordering contains neither branching nor arithmetic. **Explicit statement of what is uncaught**: hashes do not establish binding between spec and actually written bytes — if the first golden image was itself written by faulty code, regression will treat that faulty version as baseline; what stops that is checker as an independent parser and the conformance suite judge, not `spec_hash`. Specs can only be written manually, with no existing mechanism projecting kb field tables to toml. The **magnitude of risk** in batch freezing where "early frozen layers omit reserved fields" cannot be quantified; it depends on yet non-existent concrete layouts in D8 (Core Index Structure) / D11 (Index Nodes Message Buffers), so only the stop-loss rule in settled item 7 can be provided, while residual risk remains unquantifiable.

**Evidence**:

- E142 (new-pool file creation dry run): Six adjacent fields of width 8 in the unit header are arranged sequentially in written bytes; swapping any two while preserving all scalars is constructible — projecting only scalars in spec cannot safeguard field ordering.
- Three-way verification: the reverse deduction leg eliminated the branch "spec projects only scalars" according to pre-execution failed clauses; settlement adopts the tightened form (spec must include row ordering), which demands one extra guarantee and subtracts nothing. Settled by main agent.

**Owed checks**: C227 (golden image baseline written by its own code version); C226 (spec manually copied without generation from field tables); C119 (freeze components lack spec files).

#### Settled Item 4: Bit Allocation Registry for Feature Bits

**Settlement**: Three bitmaps of 256 bits each (three bitmaps of settled item 10); the sole registry for bit numbers is the table in this section. "Refuse mount on unrecognized bits" must apply even to the default layout, so the first layout line claims a bit from mkfs onward — avoiding the alternative "first line claims no bit" (which would allow readers unfamiliar with SSD layout to mount).

| bitmap | bit | meaning | provenance |
|---|---|---|---|
| incompat | 0 | First pure-SSD layout line | D12 (Target Media) settled item 5; D12 (Target Media) settled item 1 "one incompat bit per layout"; retired: the version with rollback witness and system configuration lacking F, readers reject as unrecognized bit (user 2026-09-26) |
| incompat | 1 | First pure-SSD layout line (system configuration carries rollback floor F, root record carries unmount mark, lacks rollback witness and rollback row) | D12 (Target Media) settled item 5; user 2026-09-26 settled "switch incompat bit, cannot mount" |
| incompat | 2..255 | Unallocated; append one bit for each new layout line | D12 (Target Media) settled item 1 |
| compat_ro | 0..255 | Unallocated | — |
| compat | 0..255 | Unallocated | — |

**Scope**: Governs only how bit numbers are split and the assignment of the first bit; semantic definitions of each bit belong to the decision claiming that bit. Bytes written in new-pool file creation: the 96 bytes of feature bits in system configuration change from all zeros to incompat bit 1 set to 1; bitmap is little-endian, bit 1 is the 2nd bit counting from the least significant bit of the first byte (D22 (Synthesizing Unit Atomicity) settled item 13). This item did not go through three-way verification; both main agent recommendation and user choice can be overturned.

**Evidence**:

- E142 (new-pool file creation dry run): The 96 bytes of feature bits in system configuration are written per this item, closing its blank G12, and [layout/01-first-txn.md](../layout/01-first-txn.md) updated synchronously.
- User settlement, verbatim text in decisions history; pre-settlement cell text also in decisions history, including its same-day entry "alters bytes: no" — that statement has been overturned by this settlement.
- User settlement 2026-09-26: bit 0 retired, layout identity switched to bit 1, old images (with rollback witness, system configuration without F) cannot mount (three-way verdict `research/prompts/m2-rollback-forward-r3-main-verification.md` section 2 K7: "reject writable mount if witness table non-empty" eliminated; read-only recovery of 13 corruption depths had 4 falling into rolled-back versions; altering system configuration format version number cannot block it as current readers do not verify version numbers), verbatim in decisions history; no experiment: old readers failing to mount on new bit measured on prototype, new readers rejecting old bit is derived; "altering system configuration format version number cannot block it as current readers do not verify version numbers" is obsolete: starting 2026-09-27, readers reject any system configuration format version other than 1 (D22 (Synthesizing Unit Atomicity) settled item 9).

**Owed checks**: C183 (one incompat bit per layout while on-disk bytes show no difference).

#### Settled Item 5: Freezing is a Commit Arbitrated by Gates, Not a Verbal Statement

**Settlement**: The **action** is committing `.claude/kb/format-frozen/<component>.toml` to the repository (containing `version`, `frozen_at_commit`, `spec_hash`). **Who judges**: not a human "feeling ready" — once the gate detects that the current diff touches this directory, it switches automatically to a stricter suite of pre-checks (checklist in settled item 6), permitting the commit to pass only if all are satisfied. **Humans only decide "whether to initiate a freeze now", while machines decide "whether this freeze request holds"**, isomorphic with `.claude/singlefs-ai-sop/rules/show-me-test.md` "do not differentiate by source, differentiate only by evidence".

**Scope**: That stage does not run ordinarily (running only when the diff touches that directory), so attaching it never slows daily commits; **it is destined to trigger 0 times for long periods, which is not a flaw but correct behavior** — sharing the same honesty as gates explicitly listing unimplemented stages. ⚠️ It is commit-triggered, whereas the freeze moment for component 1 in settled item 7 is "the first image writing a Type 2 container", which is an execution rather than a commit => pre-freeze checklist has no execution path for component 1, and the gate will never ignite for it.

**Evidence**: No experiment: settles gate trigger timing and arbitration ownership, with no quantifiable metric; format aligns with existing "project local stage" mounting points in shared `gate.sh`. Settled by main agent.

**Owed checks**: C225 (component freeze moment bypasses gate triggers).

#### Settled Item 6: Checklist That Must Be Completed Before Freeze

**Settlement**: Seven prerequisites, judged item by item upon freeze request initiation, re-judged upon each freeze.

| # | Item | What happens if freeze proceeds without it |
|---|---|---|
| 1 | For the component to freeze, its field table in kb has no row marked "unsettled", "envisioned", "semi-settled", or "placeholder" | Freezing an evolving design; subsequent answers to open items will be retroactively claimed as part of v1. ⚠️ This and item 7 are two distinct rulers: item 1 measures completeness of **already written tables**, while item 7 measures whether **unwritten items** touch it |
| 2 | I-1 (block self-describing) and I-2 (checksum) category checkers are implemented and pass green | "Checker is specification" — without checkers, "what v1 is" exists only in documents, and subsequent modifications cannot be mechanically adjudicated |
| 3 | Four reserved fields of D9 (Encryption) settled item 12 can be asserted in checkers as "present with width matching reservation" | Omitting even one forces incompat patching post-freeze, and existing old images permanently lack this field |
| 4 | I-3.1 (allocated accounting matches) is implemented and passed N>=5 rounds all green | The sole verification method for accounting correctness is this comparison; without it, whether frozen ledger fields are correctly designed remains completely untested |
| 5 | Model differential and crash-point replay each pass at least one round, **running against the exact format fingerprint about to be frozen** | Otherwise v1 describes a format never touched by automated verification |
| 6 | C4 (project rule checklists inconsistent) and C8 (gate scope cannot be evaluated) in [checks-owed.md](../checks-owed.md) are implemented | Post-freeze rule divergence or gates slowing down without measurement leads to incidents in higher-risk phases |
| 7 | Every still unsettled item across repository carries the verdict "touches format or not", and none judges "touches" or remains unjudged — unless the freeze request explicitly enumerates them and accepts the risk | This chain mutually defines itself; silently freezing with "unsettled" items equates to pretending decisions are already made. Judgment rules in settled item 1 |

⚠️ **Reviewing every item in the checklist requires taking values fresh from the settlement line of corresponding items; manual copying is forbidden.** Freezing is irreversible, while copying an erroneous number looks nowhere awkward in prose => **open the settlement line of that item verbatim during review**; machine-checked forms and prerequisites are tracked in C62 (settlement values of items branch in kb); until that check is implemented, this step relies on humans.

**Executable forms of items 1 and 7**: Item 1 scans the field table of the component to freeze — the table in [layout/01-first-txn.md](../layout/01-first-txn.md) listing fields sequentially by on-disk order is its ready form; scanning its **"width" and "status" columns**, encountering **any** of `未定` / `预想` / `半定` / `占位` triggers a red gate. Item 7 scans the "touches format or not" verdict in unsettled items across the repository. It splits into two halves: the **presence check** is the second ruler of the blocking-verdict cell in `.claude/gate.d/doc-decisions.sh`, sharing parsing, authoritative lists, independent reporting, and distinct remedies with the first ruler (modifying new-pool file creation bytes), running regularly; the **rejection check** (judging "touches" or unwritten triggers red gate unless listed explicitly with accepted risk) happens upon freeze request initiation, whose prerequisite is C225 (component freeze moment bypasses gate triggers) — freeze requests have no execution path for component 1. Neither item **creates a duplicate checklist** — `.claude/singlefs-ai-sop/rules/kb-discipline.md`: only one authoritative record permitted per fact.

**Scope**: Wordlist for item 1 is written into clauses without leaving script authors to guess — terms used in byte tables today are "envisioned" and "semi-settled" (the extent tree node row verbatim "envisioned 162 = 86 + 2 x 24 + 28", status column "semi-settled"; whether it carries key ranges is unsettled, 162 with, 114 without, a 48-byte difference shifting the entry start offset), precisely the evolving design item 1 intends to intercept; a check scanning only "unsettled" would pass it, allowing a Layer 4 freeze request to retroactively affirm a 48-byte unresolved measurement basis into v1. The checklist validates whether prerequisites are complete, not whether the content of any item is correct.

**Evidence**:

- E142 (new-pool file creation dry run): The field table item 1 scans is the very table written and checked row by row; the words "envisioned" and "semi-settled" in "width" and "status" columns possess concrete objects.
- Settled by main agent (narrowing of item 1 vocabulary in 2026 detailed in decisions history); judgment rules for item 7 reside in settled item 1.

**Owed checks**: C62 (settlement values of items branch in kb); C8 (gate scope cannot be evaluated) (the other half of checklist item 6, C4 (project rule checklists inconsistent), has been repaid).

#### Settled Item 7: Freezing Split into Four Layers, Plus Three Independent Freeze Components Outside Layers

**Settlement**: The four layers have strict dependency ordering.

```
Layer 1: System configuration + block header (self address / generation / tree ID / fsid / magic)
    ↓
Layer 2: Pointer field layout (D4 checksum + D9 MAC / nonce / ciphertext checksum)
    ↓
Layer 3: Key encoding for each keyspace (D8 locality_id / inode / offset)
    ↓
Layer 4: Index node internal layout (node size rules, bset format, D11 message buffers)
```

Rationale for dependency direction: Layer 1 is the physical premise that "metadata blocks must be self-describing", upon which all subsequent layers rest; Layer 2 is pinned by D9 (Encryption) MAC / nonce / ciphertext checksum being "impossible to insert after the fact", and must precede tree structures relying on it; once key encodings in Layer 3 are settled, organizing these keys within nodes has a foundation for discussion.

**Risks of batch freezing must be explicitly accepted**: If an early frozen layer omits fields, subsequent work must patch via incompat, equivalent to burning design margins prematurely. There is only one stop-loss rule: **Layers 1 and 2 must include all four D9 (Encryption) reserved fields prior to freeze**, as they happen to be fields known to be needed eventually, which cannot be deferred to Layers 3 or 4.

**Independent freeze components outside the four layers**: Three classes of permanent bytes do not belong to any of the four layers; each forms an independent component, with `spec_hash` in C9 (covertly modifying format after freeze) recorded per component, keeping the four layers untouched:

- 1. **Payload record format for packed record units**, registered by packed record type: Type 2's 140-byte inode record table (D8 (Core Index Structure) settled item 6), freeze moment being the first image writing a Type 2 container; Type 1 (width 40, D18 (Information Carried in a Block) settled item 24) is mutually independent, not frozen together with Type 2.
- 2. **Statistics registry** (tag value + dimension + merge rules, D5 (Snapshot and Space Accounting Mechanism) settled item 3 / 4) — merge rules are on-disk semantics; two readers applying different rules to the same bytes will compute divergent values; C71 (silently ignoring unrecognized statistics tags) shields only unrecognized tags, not recognizing tags while applying wrong rules.
- 3. **journal record format** (record header, named items, payload checksum coverage, transaction boundary): pre-freeze requirements no earlier than Layer 2 and placement granularity; freeze moment is the commit submitting its toml file; new record formats follow "new record type + system configuration log-incompat bit", full text in D16 (Publish Semantics) settled item 3.

**Scope**: The four-layer diagram is not rearranged: D21 (Boundary Between Authoritative and Derived States) settled item 10 changed status classification to per-structure judgments; in Layer 4, Code 2 nodes of the accounting tree are authoritative state while internal nodes of other trees are derived state, so no layer is pure; thus the unit of exiting freeze is a structure rather than a layer (placement of each structure, its state category, and whether it exits freeze are registered in `.claude/kb/freeze-layer-membership.md`, checked by gate doc-decisions freeze-layer-membership cell); spec files for the three components do not exist; freeze moment for component 1 is an execution rather than a commit, so the gate in settled item 5 does not ignite for it. No clauses yet exist for which layer tree table units and instance table units belong to.

**Evidence**: No experiment: layering and dependency ordering are derived from sorting "which elements cannot be retrofitted later", with no quantifiable metric; target of the stop-loss rule (four reserved fields of D9 (Encryption)) resides in D9 (Encryption) settled item 12. Settled by main agent and user, verbatim in decisions history.

**Owed checks**: C119 (freeze components lack spec files); C225 (component freeze moment bypasses gate triggers); C246 (tree table unit lacks freeze layer membership).

#### Settled Item 8: What Freedoms Are Lost After Freezing

**Settlement**:

| Freedoms during soft format phase | What they become after freezing |
|---|---|
| Tearing down and rebuilding structures | Width / existence / meaning of frozen fields can no longer change; additions only, retired fields follow "retain read support, cease writing" |
| Correcting text directly upon finding flawed designs without compatibility | Can still be overturned at decision record level, but **on-disk byte compatibility transitions from zero cost to non-zero** |
| mkfs automatic selection rules can switch algorithms anytime | Selection rules for frozen components are also pinned; changing algorithms causes default discrepancies between old and new images, requiring a new format version number |
| Modifying formats directly upon disappointing experiment outcomes | New policies must be introduced via new feature bits, retaining interpretability of old values |
| Checker freely modified alongside formats | **For frozen portions, checker judgments become de facto compatibility contracts** — silently altering checker interpretations of frozen fields is as perilous as silently altering formats |

**Scope**: Only states which actions cease to be valid post-freeze, without adjudicating specific change grades — which belongs to settled item 9. Before the first external user emerges, format remains soft, and none of this table takes effect prior to that point (`.claude/rules/format-evolution.md`).

**Evidence**: No experiment: itemized expansion of the constraint "external permanent commitments cannot be altered", with no quantifiable metric. The final row (checker judgments become contracts) shares provenance with "checker is specification" in D13 (Verification Route). Settled by main agent.

**Owed checks**: None.

#### Settled Item 9: Post-Freeze Change Judgment Table

**Settlement**:

| Change type | Old reader behavior | Grade | Bump | Rationale |
|---|---|---|---|---|
| Adding fields in reserved space / TLV tail | Ignored, continues normal operation | compat | No | Pure addition without semantic dependency; safety net of "ignoring unfamiliar fields" suffices |
| Adding fields whose omission causes old readers to miscompute **another existing field** | **Miscomputes, potentially corrupting data** | incompat | Yes | Once semantics couple, safety nets fail |
| Expanding existing field width | Parsed with old width, out-of-bounds read / misalignment | incompat | Yes | Width is part of layout; offset calculations in old code break completely |
| Changing existing field meaning | Interpreted with old meaning, producing erroneous behavior **without error** | incompat, with **old position abandoned without reuse** | Yes | Reusing positions leaves old and new readers unable to distinguish, the most dangerous class. Two examples (D18 (Information Carried in a Block) settled item 11): adding a unit class (registry codes 4–15) alters the interpretation measurement basis of class identity segments; giving meaning to the constant-0 flag bit at common prefix offset 7 (old readers reject non-zero bits) |
| Adding record types (new key types) | Read-only traversals may misjudge; compaction / write may delete misunderstood records | compat_ro | Yes | **"Read-only safety" and "writable safety" are independent guarantees** that must be judged separately. ⚠️ Packed record types inside packed record units **do not follow this row** (D18 (Information Carried in a Block) settled item 11): skipping a tombstone or inode record alters read answers, defaulting to incompat; only types proven at registration that "skipping changes no read answer" demote to compat_ro, and compat_ro readers must not execute scan-rebuilds |
| Adding a keyspace | If tree table is open list day-1: simply does not see new tree; if tree count hardcoded: fails to read tree table | **compat** (settled item 2 makes tree table an open list day-1). Reserving open tree table -> compat; omitted -> incompat | Yes | **"Whether tree table is expandable" must be settled at freeze**, cannot wait until adding trees reveals omission |
| Changing sorting order | Binary search / traversal results incorrect **without error** | incompat | Yes | Sort order is index identity; reordering log-structured nodes directly relies on its invariance |
| Changing checksum / MAC algorithm | D9 (Encryption) algorithm type field reserved -> unfamiliar types fail explicitly | Reserved -> compat_ro; unreserved -> incompat | Yes | Item 4 of D9 (Encryption) settled item 12 is precisely the channel prepared for this change |
| Retiring a field | Unaffected, old readers continue interpreting old way | compat, with **parsing code preserved permanently** | No | Any field committed to disk must remain interpretable as long as a single old image exists |

**Scope**: Judges "what old readers will do", not "whether this change ought to be made". The first row (additions in reserved space are compat) holds only where real reserved space exists; inode headers carry only 2 bytes of reserved space, accommodating few fields.

**Evidence**:

- E59 (can buffered messages be recomputed solely from units): Skipping a tombstone record causes deleted keys to fully resurrect, resurrection count precisely matching deletion count — the quantified basis for "packed record types do not follow compat_ro row, defaulting to incompat".
- Settled by main agent, row-by-row rationales documented in table.

**Owed checks**: C102 (inode header lacks reserved space making field additions non-compat); C97 (changing field semantics without altering width produces no red gate anywhere in repository).

#### Settled Item 10: Bit Count, Accounting, and Allocation Rights of Feature Bits

**Settlement**:

- **Bit count**: Three bitmaps of **256 bits** each (`u64[4]`), stored in system configuration. Bits are far cheaper than field widths (3x32 bytes negligible for system configuration), **reserves should lean generous**.
- **Accounting**: `.claude/kb/feature-bits.md` lists `bit | category | name | introduced version | introduced commit | status | one-line semantics`; the sole registry for bit numbers is the table in settled item 4.
- **Allocation right is "first merged, first claimed", not an approval hierarchy**. Gate enforcement: bit numbers strictly monotonic, no gap-skipping reservations, no code referencing non-existent bit numbers — mechanical arbitration.
- **Once used, bits may never be reclaimed.** Rationale is **stricter** than non-recycling of numbers in [invariants/README.md](../../../crates/singlefs-checker/invariants/README.md): numbers are retrieval keys whose reuse misleads humans or models; **feature bits are physical bits that existed on disk**; reallocating them to a new feature means an old image mounted on new code misinterprets the bit as "new feature X enabled" — **the file system would perform real incorrect actions** (e.g., treating non-existent encryption characteristics as present and searching for non-existent keys). Retired bits are marked "meaning permanently locked to semantics of last use prior to retirement", and never assigned to new features.

**Scope**: Governs how bits are divided, recorded, and whether they can be reclaimed; does not govern semantics of individual bits (belonging to decisions claiming them).

**Evidence**: No experiment: bit counts and non-reclamation are deduced from the physical fact that "old images will be read by new code", with no quantifiable metric; 256-bit space overhead is negligible for system configuration, so this cell does not optimize for frugality. Settled by main agent.

**Owed checks**: C183 (one incompat bit per layout while on-disk bytes show no difference).

#### Settled Item 11: Two Gates That Can Go Red Paired with Freezing

**Settlement**:

1. **Format freeze contract**: Record `spec_hash` for each frozen component; gate recomputes and compares; divergence requires current diff to **simultaneously** satisfy "`feature-bits.md` has new rows or `version` is explicitly altered" and "corresponding entry in `decisions.md` modified in same commit". Missing either triggers a red gate.
2. **Backward compatibility regression (golden images)**: Whenever a component version changes, generate and archive an image using **code of that era**; gate reads all historical images using **current code**, executes checker, and compares expected content digests. Image bodies **do not enter git** (`.claude/singlefs-ai-sop/rules/command-safety.md`: test images must not reside in repository); repository commits only the manifest `.claude/kb/golden-images.toml`; images are placed in `${SINGLEFS_GOLDEN_DIR:-$HOME/.cache/singlefs-golden}`. The manifest is **append-only** — deleting regression for any version equates to silently abandoning compatibility commitments, requiring formal decision records.

**Scope**: Neither is yet implemented; numbers are in checks-owed table. Rule 2 compares only "whether today's code understands historical images", not "whether the original image was written correctly"; Rule 1 anchors on declarations rather than bytes; neither shields against "the initial golden image itself being produced by erroneous code".

**Evidence**: No experiment: settles what two gate checks should judge; discrimination self-proof awaits implementation; failure modes they guard against (covertly modifying formats, quietly abandoning compatibility commitments) are tracked in C9 (covertly modifying format after freeze) and C227 (golden image baseline written by its own code version). Settled by main agent.

**Owed checks**: C9 (covertly modifying format after freeze); C227 (golden image baseline written by its own code version).

````

**出处 `.claude/kb/decisions/15-format-freeze-policy.en.md:234-237`（整段抄，未转述）**

```markdown
## Historical Versions

Historical entries for D15 (Format Freeze Policy) are centralized in [decisions-history.md](../decisions-history.md).

```

**出处 `.claude/kb/decisions/15-フォーマット凍結ポリシー.ja.md:1-6`（整段抄，未转述）**

```markdown
## D15 (フォーマット凍結ポリシー) — 確定

[中文](15-格式冻结政策.md) · [English](15-format-freeze-policy.en.md) · **日本語**

D15 (フォーマット凍結ポリシー) は二つの問題を所管する：**いつ「format v1 frozen」と宣言できるか**、および**凍結後に新要素を追加する際にどの流路を経由するか**。本決定は D11 (インデックスノード内のメッセージバッファ要否) / D12 (対象メディア) / D14 (大小ファイル・永続/一時デュアルトラック) に共通する前提である——「フォーマット分岐が恒久化する」というコストの現在価値はゼロであるが、凍結後は無限大となるため、これら各決定の真の代償は本決定により定義される。対象外とする事項：各フィールド自体に定めるべき具体的な値は各決定の所管に属する；最初の外部ユーザーが現れるまでフォーマットは流動的であり、その運用方針は `.claude/rules/format-evolution.md` にある；`feature-bits.md` における各ビットのセマンティクスは当該ビットを要求する決定に属し、D15 (フォーマット凍結ポリシー) はビット番号の割り振り方針と回収可否のみを所管する。

```

**出处 `.claude/kb/decisions/15-フォーマット凍結ポリシー.ja.md:7-233`（整段抄，未转述）**

````markdown
### 確定項目

D15 (フォーマット凍結ポリシー) の項目番号体系は単一であり、番号の振り直しは行わない。

| # | 項目 | 確定内容 |
|---|---|---|
| 1 | **凍結前チェックリスト第 7 項の具体的処置** | 第 7 項は決定リストを列挙せず、単一の判定式とする——リポジトリ内の未確定項目ごとに「フォーマット変更要否」を自ら宣言し、凍結要求がそれと交差すれば赤判定；デフォルトパディング幅は 0 **状態：確定。** **進捗：対象なし。** |
| 2 | **ツリーテーブルは開放リストか否か** | 開放リストとする、day-1 から開放リストとして実装 **状態：確定。** **進捗：実装済み。** |
| 3 | **`spec_hash` の算出元** | 切り替えない。常に `format-spec/<コンポーネント>.toml` に対して算出し、当該 toml は行順および非スカラーのディスク上セマンティクス規則を含む kb フィールド表の完全射影とする **状態：確定。** **進捗：未実装。** |
| 4 | **feature bit のビット割り当て登録表** | 登録表は確定項目 4 に配置；incompat ビット 1 = 最初の純 SSD レイアウト系統（F およびアンマウント記号を含む版）、mkfs 時点から設定、ビット 0 は廃止；compat_ro、compat は 1 ビットも割り当てない **状態：確定。** **進捗：実装済み。** |
| 5 | **凍結という動作の定義** | 凍結 = リポジトリに `.claude/kb/format-frozen/<コンポーネント>.toml` をコミットすること；人間は開始するか否かのみを決定し、機械が要求の成立可否を判定する **状態：確定。** **進捗：未実装。** |
| 6 | **凍結前に完了必須のチェックリスト** | 7 項目の前提条件とし、凍結ごとに再判定；各項目の値は対応する小項目の確定行からその都度取得し、手作業の転記を禁止する **状態：確定。** **進捗：対象なし。** |
| 7 | **凍結階層の分割と4層外の独立要素** | 4層には厳格な依存関係（システム構成とブロックヘッダ、ポインタフィールド配置、key 符号化、インデックスノード内部レイアウト）が存在し、別途3つの独立凍結コンポーネントを設ける **状態：確定。** **進捗：対象なし。** |
| 8 | **凍結後に失われる自由** | 凍結済みフィールドの幅・存在・意味は変更不可となり追加のみ許容；凍結箇所に対する checker の判定も互換性規約となる **状態：確定。** **進捗：対象なし。** |
| 9 | **凍結後の変更判定表** | 9 分類の変更それぞれについて旧リーダーの挙動を基準に compat / compat_ro / incompat および bump の要否を判定 **状態：確定。** **進捗：対象なし。** |
| 10 | **feature bit のビット数・会計・割り当て権** | 3 つのビットマップ（各 256 ビット）をシステム構成に格納；割り当て権は「先行マージ優先」、ビット番号は厳格に単調増加；一度使用したビットは再利用を禁止 **状態：確定。** **進捗：実装済み。** |
| 11 | **凍結に伴い整備すべき赤判定検査** | フォーマット凍結規約（`spec_hash` 照合）および後方互換性リグレッション（ゴールデンイメージ）；イメージ本体は git に含めず、リストは追加のみ許容 **状態：確定。** **進捗：未実装。** |

#### 確定項目 1：第 7 項の判定対象は未確定項目であり決定リストではない

**確定内容**：3 条。

1. **対象はリポジトリ内の未だ未確定であるすべての項目であり、決定リストではない。** 未確定項目ごとに判定文言——**フォーマット変更要否**を付与する：その結論がディスク上のバイト列を変更するか、あるいは既存バイト列の**解釈計測条件**を変更するか否か（計測条件の側面は C97 (改字段含义而不改宽度，全仓没有一处会红) の形態を指す：幅は 1 バイトも変わらないが意味が変化する場合）。取り得る値は 3 つ：**変更あり / 変更なし / 境界未確定 (変更ありとみなす)**；**未記載は変更ありとみなす**。
2. **判定**：凍結要求の発行時、「変更あり」または未判定の項目があれば赤判定とする。段階的凍結を行う場合は、要求内で**全項目を個別に列挙してリスクを明示的に受容**しなければならず（確定項目 7 の「段階的凍結のリスクは明示的に受容されなければならない」を踏襲）、**列挙漏れがあれば赤判定**とする。
3. **パディング幅のデフォルトは 0** とし、非ゼロの幅を確保しようとする側に立証責任を課し、`.claude/rules/fs-design.md` のフォーマット分岐判定基準（原文ママ：「フォーマット分岐に価値があるかは、『誤用』を『マウント不可』に変えられるか否かによる」）を通過させなければならない。

問いの立て方が読み方を規定する (`.claude/singlefs-ai-sop/skills/decide/SKILL.md` 必須要件 7：未確定項目の問いは一意に読めなければならない)：「6 つのスロットの値を個別に固定する」方式（各スロットの内容は各決定内に存在するため、複製は同一事実の二重記録となる）は採らず、「第 7 項の判定規則を定める」方式を採る；チェックリストはコミット契機の前置検査であり凍結要求ごとに実行されるため、第 7 項は**凍結ごとに再判定**される。

**射程**：デフォルトパディング幅 0 の規定が所管するのは**実装しないことが確定した機能のためのプレースホルダ確保**であり、構造体内の**汎用予約領域**は所管しない——汎用予約領域は確定項目 9 判定表の第 1 行（予約領域 / TLV 末尾へのフィールド追加 ⇒ compat かつ bump なし）および C102 (索引节点头没有预留区，加字段不是 compat) が所管し、判定基準が異なる；両者を混同すると「どこにも予約を残さない」という結論に至るが、C102 (索引节点头没有预留区，加字段不是 compat) は現在その逆方向の未済の検査を清算中である。規則として昇格するのは「デフォルト 0 + 立証責任の転換」という手続きのみであり、「v1 にはこの機能を含めない」という分岐に達した項目は、各自でフォーマット分岐基準を再度通過する必要がある。述語は層ごとに分割しない：凍結からの除外は構造単位で判定され、層単位ではないためであり（確定項目 7 の射程）、かつ構成要素が揃っていない——確定項目 7 における 3 つの独立凍結コンポーネントは 4 層のいずれにも属さない ⇒ 層単位の述語はそれらに対して定義できない；「フォーマット変更要否」という二値述語で分割することでこれら双方の破綻を回避でき、層間の独立判定可能性を前提としない。D26 (バックグラウンド整理と配置回収) 確定項目 4 のセルは従来どおり D26 (バックグラウンド整理と配置回収) 自らが保持し、D15 (フォーマット凍結ポリシー) が代行して回答する権限はない。判定入力は常に現行状態から取得し、ここに複製を保持しない：`grep -c '未定\*\*' .claude/kb/decisions.md` によりリポジトリ内の未確定項目残数を数え（項目リスト行末は現在「—— **未定** · 未实现」の形式であり「未定**」で終わらない）、`bash .claude/gate.d/doc-decisions.sh --check decision-items-sync` で当該リストが本文の忠実な射影であることを判定する。

**根拠**：実験なし：ゲート判定規則の対象とデフォルト値を定めるものであり、定量可能な数値は存在しない。これを支えるのは 2 つの事実である——リスト形式には漏れが生じ得ること（D22 (ユニット原子性の合成方法) 確定項目 9 は**システム構成のフィールド表**であり、システム構成は第 1 層である；D19 (ブロックポインタの構造と幅予算) 確定項目 6 は**中央マッピングエントリのフィールド表**であり、第 2 層に位置する——いずれも 6 決定のリストには含まれず、これら 2 層を凍結する際にリストは一切関与できない）、および「フォーマットレベル」タグに依存した網羅は不可能であること（番号の登録場所は各決定本文のタイトル行であり索引ページではないため、`grep -rn 格式级 .claude/gate.d/ .claude/scripts/` はヒット 0 件 ⇒ 対象 0、実装 0）。メイン agent による確定、三方検証に基づく；成立しなかった論拠は再利用防止のため変更履歴に記録している。

**未済の検査**：C62 (分项定案值在 kb 里分叉)；C102 (索引节点头没有预留区，加字段不是 compat)。

#### 確定項目 2：ツリーテーブルを day-1 から開放リストとする

**確定内容**：ツリーテーブルは day-1 から開放リストとして構成する。根拠は確定項目 9 の判定表にある：開放リスト下では「新規 keyspace の追加」は旧リーダーにとって単に**新規ツリーが見えない**だけであり（compat グレード、グレースフルデグラデーション）；ツリー数をハードコードすると**ツリーテーブルを読み出せない**状態となる（incompat、即時破損）。**開放リストが厳格に優位**であり、その代償はツリーテーブルユニット内のテーブル長フィールドと可変長テーブル部分の追加にとどまる——これは確定項目 10 の「予約は広めに確保する」と同一の論理である。⇒ keyspace の追加は本プロジェクトにおいて明記された拡張目標であり偶発的事象ではないため、判定表の当該行のグレードは **compat** として固定される。

**射程**：定めるのは「ツリーテーブルが開放リストであること」であり、ツリーテーブルの配置先とは無関係である——ツリーテーブルはルートレコードの間接参照層に位置する（D22 (ユニット原子性の合成方法) 確定項目 7 原文ママ「ツリーのルートは常にツリーテーブルユニットに格納し、ルートスロットにフラット展開しない」、確定項目 12「システム構成は自前のツリーテーブルユニットポインタを持たない」；D21 (権威態と派生態の境界) はルートレコード + ツリーテーブルユニット + インスタンス表ユニットを一括して「ルート」に分類）。これは**第 1 層の凍結時に組み込まれなければならず**、ツリーを追加する段になって未確保であることが判明する事態は許容されない；なおツリーテーブルユニット自体の凍結層帰属は依然として未済である。

**根拠**：

- E142 (新規プールでのファイル作成のドライラン)：新規プールでのファイル作成で書き出されるツリーテーブルユニットはエントリ数で宣言され（mkfs 版は 0 エントリ、トランザクション版は 11 エントリであり、退役ツリー追加前の現行値；退役ツリー追加後のトランザクション版は 12 エントリ）、テーブル長はコンパイル時定数ではなくデータである——開放リストの形態は書き出されたバイト列において成立している。
- ユーザーによる確定、発言原文は変更履歴にある。

**未済の検査**：C246 (树表单元没有冻结层归属)。

#### 確定項目 3：`spec_hash` は kb フィールド表の完全射影から常に算出し、Rust 構造体へ切り替えない

**確定内容**：`spec_hash` は常に `format-spec/<コンポーネント>.toml` に対して算出する。当該 toml は **kb フィールド表の完全射影**であり——**行順**（ディスク上の順序）および**非スカラーのディスク上セマンティクス規則**（確定項目 7 のコンポーネント ② のマージ規則をデータ形式で記述）を含み、登録済みのスカラー定数の単なる集合ではない。コード実装後に新たに追加されるのは**ハッシュ元の変更ではなく**、2 つの検査である：① spec とコード間の**同等性検査**（幅と順序の双方を比較）；② ゴールデンイメージリグレッション（確定項目 11 第 2 条）。

4 つの理由：

1. **方向性**。D13 (検証ルート) 確定項目 5 原文ママ：「**checker と実装の間で共有されるのは唯一つ：kb のフィールド表から生成された定数モジュールのみであり**、ジェネレータが kb から読み取り双方は消費のみ行い、何人による手作業の変更も禁止する」⇒ 権威は kb にあり、Rust 構造体は**下流の消費者**である；構造体に対してハッシュを算出することは、恒久的な規約の指紋を派生品に固定することに他ならない。
2. **ハッシュは変更検知器であり、二つの成果物間の同等性検査ではない。** 構造体に対してハッシュを算出しても「構造体と kb の不一致」は捕捉できず、構造体が**変更された**際に警告を発するのみである；その種の誤りを真に捕捉するのは同等性検査であり、同等性検査は spec 側の存在を前提とする ⇒ 切り替えを行っても付加価値が得られないばかりか、照合すべき対向側を解体してしまう。
3. **言語バインディング**。D17 (実装階層とサードパーティパイプライン) 確定項目 3 原文ママ：「判定は第三者が独自に再検証可能でなければならず、さもなければ『適合』は『当方が適合と主張している』状態に劣化する」。指紋を Rust ソースコードのテキストに固定すると、サードパーティパイプラインによる再計算が不可能となる。
4. **実現性**。3 つのコンポーネントの spec ファイルは直ちに記述可能であり、ハッシュ機構を最初の凍結より遥か以前から有効化できる；一方で確定項目 7 のコンポーネント ① の凍結契機はタイプ 2 コンテナを書き出す最初のイメージである ⇒ 切り替え作業が最初の実凍結と同一コミットに強制され、試行運用期間がゼロとなる。

**射程**：spec がスカラーのみを射影するのでは不十分であり、行順を含めなければならず、同等性検査も順序を比較しなければならない——[layout/01-first-txn.md](../layout/01-first-txn.md) のユニットヘッダには幅 8 の隣接フィールドが 6 つ行順に並んでおり（5つ組のツリー ID / オブジェクト ID / オブジェクト誕生代 / アンカーオフセット、ならびに誕生代番号、fsid）、シリアライズ時に任意の 2 つを入れ替えても全スカラーおよび `DATA_UNIT_HEADER_BYTES` は一切不変のままディスク上のバイト列が完全に狂い、確定項目 9 判定表の「既存フィールドの意味変更」の行に該当することになる。D13 (検証ルート) 確定項目 5 の「ジェネレータはスカラー値のみを出力する」という境界規定は checker と実装の間の共有物を所管しており、`format-spec/<コンポーネント>.toml` は**ゲート成果物**であり実行時の消費者は存在しない ⇒ 当該境界の射程外であり、行順を持つフィールド表には分岐も算術も存在しない。**検知不能な事項の明記**：spec と実際に書き出されたバイト列との間において、ハッシュは直接の拘束力を持たない——最初のゴールデンイメージ自体が誤ったコードで出力された場合、リグレッションは誤った版を基準として扱うことになる；これを阻止するのは独立したパーサである checker および適合性スイートの判定プログラムであり、`spec_hash` ではない。spec は手書きするほかなく、kb フィールド表から toml へ射影する自動機構は存在しない。段階的凍結における「早期凍結層でのフィールド確保漏れ」の**リスク規模**を定量化することはできず、未だ存在しない D8 (コアインデックス構造) / D11 (インデックスノード内のメッセージバッファ要否) の具体的レイアウトに依存するため、確定項目 7 の損切り規則を提示できるのみであり、残留リスクは計測不能である。

**根拠**：

- E142 (新規プールでのファイル作成のドライラン)：ユニットヘッダにおける幅 8 の隣接する 6 フィールドは書き出されたバイト列上で行ごとに整列しており、任意の 2 つを入れ替えても全スカラーを不変に保つ状態は構成可能である——spec がスカラーのみを射影する形態ではフィールド順序を担保できない。
- 三方論証：事前定義された失敗条項に基づき、逆推論レグが「spec はスカラーのみを射影する」という方針を除外した。確定内容は厳格化された形態（spec は行順を含まなければならない）を採用しており、新形態は旧形態よりも追加要件を課すのみで要件を一切削減していない。メイン agent による確定。

**未済の検査**：C227 (黄金镜像的基准是它自己那份代码写的)；C226 (spec 是手抄的，没有从字段表生成)；C119 (冻结组件没有 spec 文件)。

#### 確定項目 4：feature bit のビット割り当て登録表

**確定内容**：3 つのビットマップ（各 256 ビット、確定項目 10 の 3 種のビットマップ）；ビット番号の唯一の登録場所は本節の登録表とする。「認識不能なビットでのマウント拒絶」はデフォルトレイアウトに対しても機能させる必要があるため、第 1 の系統が独自に 1 ビットを占有し mkfs 時点から設定する——「第 1 系統はビットを消費しない」方式は採らない（さもなければ SSD レイアウトを認識しないリーダーでもマウント可能となってしまうため）。

| bitmap | bit | 意味 | 出典 |
|---|---|---|---|
| incompat | 0 | 最初の純 SSD レイアウト系統 | D12 (対象メディア) 確定項目 5；D12 (対象メディア) 確定項目 1「レイアウトごとに 1 つの incompat ビット」；廃止：ロールバック証人を含みシステム構成に F を欠く版であり、リーダーは一律に未認識ビットとして拒絶する（ユーザー 2026-09-26） |
| incompat | 1 | 最初の純 SSD レイアウト系統（システム構成にロールバック下限 F を含み、ルートレコードにアンマウント記号を含み、ロールバック証人およびロールバック行を持たない） | D12 (対象メディア) 確定項目 5；ユーザー 2026-09-26 決定「incompat ビットを変更し、マウント不可とする」 |
| incompat | 2..255 | 未割り当て；レイアウト系統が新設されるごとに追加 | D12 (対象メディア) 確定項目 1 |
| compat_ro | 0..255 | 未割り当て | — |
| compat | 0..255 | 未割り当て | — |

**射程**：ビット番号の分割方法と最初のビットの帰属のみを定め、各ビットのセマンティクスは当該ビットを要求する決定に属する。新規プールでのファイル作成におけるバイト列：システム構成の feature bits 領域（96 バイト）は全ゼロから incompat ビット 1 のセット（1）へと変化する；ビットマップはリトルエンディアンであり、ビット 1 は第 1 バイトの最下位から数えて第 2 ビットである（D22 (ユニット原子性の合成方法) 確定項目 13）。本項目は三方論証を経ておらず、メイン agent の推奨およびユーザーの選択はいずれも覆され得る。

**根拠**：

- E142 (新規プールでのファイル作成のドライラン)：システム構成の feature bits 領域（96 バイト）が本項目に従って書き出され、空白領域 G12 が解消され、[layout/01-first-txn.md](../layout/01-first-txn.md) が同期更新された。
- ユーザーによる確定、発言原文は変更履歴にある；確定前の当該セルの原文も変更履歴にあり、当時の記述「バイト変更：否」を含んでいたが、その記述は本項目の確定により覆された。
- ユーザー確定 2026-09-26：ビット 0 は廃止されレイアウト識別はビット 1 に切り替わり、旧イメージ（ロールバック証人を含みシステム構成に F を欠く）はマウント不可となる（三方判定 `research/prompts/m2-rollback-forward-r3-main-verification.md` 第 2 節 K7：「証人テーブルが空でなければ書き込み可能マウントを拒絶する」方針は除外、読み出し専用リカバリの破損深度 13 パターンのうち 4 パターンがロールバックされた版に該当；システム構成フォーマットバージョン番号の変更では阻止できず、現行リーダーはバージョン番号を検証しないため）、原文は変更履歴にある；実験なし：新ビットに対する旧リーダーのマウント不可はプロトタイプで計測済み、旧ビットに対する新リーダーの拒絶は論理的導出；「システム構成フォーマットバージョン番号の変更では阻止できず、現行リーダーはバージョン番号を検証しない」との記述は陳腐化：2026-09-27 以降、リーダーはシステム構成フォーマットバージョンが 1 でない場合一律に拒絶する（D22 (ユニット原子性の合成方法) 確定項目 9）。

**未済の検査**：C183 (每套布局一个 incompat 位而盘上差不出字节)。

#### 確定項目 5：凍結はゲート仲裁を受けるコミットであり言葉の表明ではない

**確定内容**：**アクション**はリポジトリに `.claude/kb/format-frozen/<コンポーネント>.toml`（内容：`version`、`frozen_at_commit`、`spec_hash`）をコミットすることである。**誰が判定するか**：人間が「準備ができたと感じた」から発効するのではない——ゲートは今回の差分が当該ディレクトリに触れたことを検知すると、より厳格な前置検査群（確定項目 6 のチェックリスト）へと自動的に切り替わり、すべてを満たした場合にのみ当該コミットの通過を許可する。**「今凍結を開始するか否か」のみを人間が決定し、「今回の凍結要求が成立するか否か」は機械が決定する**のであり、`.claude/singlefs-ai-sop/rules/show-me-test.md` の「出所によって区別せず、根拠によってのみ区別する」と同型である。

**射程**：当該ステージは通常時には実行されず（差分が当該ディレクトリに触れた時のみ実行）、日常のコミットを遅延させない；**長期間にわたりトリガー回数が 0 回となる宿命にあるが、これは欠陥ではなく正しい状態である**——ゲートが未実装ステージを明示的に列挙することと同種の誠実さである。⚠️ コミット契機であるのに対し、確定項目 7 のコンポーネント ① の凍結契機は「タイプ 2 コンテナを書き出す最初のイメージ」であり、これは実行であってコミットではない ⇒ 凍結前チェックリストはコンポーネント ① に対して実行経路を持たず、ゲートがそれに対して発火することはない。

**根拠**：実験なし：ゲートトリガーの契機と仲裁権の帰属を定めるものであり、定量可能な数値は存在しない；共有 `gate.sh` に既存の「プロジェクトローカルステージ」マウントポイントと整合させた。メイン agent による確定。

**未済の検査**：C225 (组件的冻结时刻绕开门禁触发)。

#### 確定項目 6：凍結前に完了必須のチェックリスト

**確定内容**：7 項目の前提条件であり、凍結要求の発行時に項目ごとに判定し、凍結ごとに再判定する。

| # | 項目 | 実施せずに凍結した場合の帰結 |
|---|---|---|
| 1 | 今回凍結対象となるコンポーネントのフィールド表において、kb 内のいかなる行にも「未定」「構想」「半定」「プレースホルダ」の記載がないこと | 変更途中の設計を凍結することになり、以降未確定項目の回答が補填されるたびに v1 の一部として追認されてしまう。⚠️ 本項目と第 7 項は異なる 2 つの尺度である：第 1 項は**既に書き出された表**の充足度を測り、第 7 項は**未だ書かれていない項目群**がそれに抵触するかを測る |
| 2 | I-1 (ブロック自己記述)、I-2 (チェックサム) 系（ブロック自己記述、チェックサム）の checker が実装済みかつ緑通過していること | 「checker こそが仕様」——checker がなければ「v1 とは何か」は文書内にのみ存在することになり、後続の変更を機械的に判定できなくなる |
| 3 | D9 (暗号化) 確定項目 12 の 4 つの予約フィールドについて、checker 上で「存在し幅が予約値に合致している」とアサート可能であること | 1 つでも確保を怠ると凍結後に incompat でパッチを当てるほかなく、既存の旧イメージは恒久的に当該フィールドを欠損する |
| 4 | I-3.1 (割り当て済み統計の合致) が実装済みであり、N>=5 回のフル実行ですべて緑通過していること | 会計の正当性を検証する唯一の手段がこの対比である；通過していなければ、凍結された台帳フィールドの設計が正しかったか否か一切検証されていないことになる |
| 5 | モデル突き合わせおよびクラッシュポイント再生がそれぞれ最低 1 回 PASS しており、**かつ実行対象が凍結直前のフォーマット指紋そのものであること** | さもなければ v1 が記述するフォーマットはいかなる自動検証も経ていないことになる |
| 6 | [checks-owed.md](../checks-owed.md) の C4 (项目规则清单不一致) および C8 (门禁范围判不出来) が清算済みであること | 凍結後に規則の乖離やゲートの遅延が放置されれば、よりリスクの高いフェーズにおいて障害へと直結する |
| 7 | リポジトリ内の未だ未確定であるすべての項目に「フォーマット変更要否」の判定が付与されており、「変更あり」または未判定の項目が 1 つも存在しないこと——今回の凍結要求において全件を個別に列挙しリスクを明示的に受容した場合を除く | この連鎖は相互に定義されており、「未確定」を残したまま暗黙裏に凍結することは既に決定したと詐称することと同義である。判定規則は確定項目 1 を参照 |

⚠️ **チェックリスト内の各項目の再確認において、値は常に対応項目の確定行からその都度取得しなければならず、手作業の転記を禁止する。** 凍結は不可逆であり、数値を誤記してもテキスト上いかなる違和感も生じないためである ⇒ **再確認時は当該項目の確定行を逐字確認すること**；機械検査の形態と前提条件は C62 (分项定案值在 kb 里分叉) に記録されており、それが実装されるまでは本工程は人手に依存する。

**第 1 項および第 7 項の実行可能形態**：第 1 項は今回凍結対象のフィールド表を走査する——[layout/01-first-txn.md](../layout/01-first-txn.md) のディスク上順序で行ごとに列挙された表がその既定形態であり、その**「幅」列および「状態」列**を走査し、`未定` / `预想` / `半定` / `占位` の**いずれか**が現れれば直ちに赤判定とする。第 7 項はリポジトリ内の未確定項目における「フォーマット変更要否」判定を走査する。これは 2 つに分割される：**記載の有無**についてはゲートステージ `.claude/gate.d/doc-decisions.sh` の blocking-verdict セルの第 2 の尺度であり、第 1 の尺度（新規プールでのファイル作成バイトの変更有無）と同一の解析・同一の権威リストを共有し、それぞれ数値を報告し個別の解決策を提示し常時稼働する；**赤判定**の側面（「変更あり」または未記載なら赤判定、全件列挙の上リスク明示受容の場合を除く）は凍結要求発行時の処理であり、前提は C225 (组件的冻结时刻绕开门禁触发) である——凍结要求という契機はコンポーネント ① に対して実行経路を持たない。いずれの項目も**第 2 のリストを別途作成しない**——`.claude/singlefs-ai-sop/rules/kb-discipline.md`：同一事実に対する権威ある記録は 1 箇所のみ許容される。

**射程**：第 1 項の用語集は条文内に規定し、スクリプト作成者の推測に委ねない——バイト表で現行使用されている語は「**預想**」および「**半定**」である（extent ツリーノードの行は原文ママ「**預想 162** = 86 + 2 × 24 + 28」、状態列「半定」、key 区間を含めるか否かは未定であり、含めれば 162、含めなければ 114 であり 48 バイトの差が生じエントリ領域の開始位置もずれる）、それこそが第 1 項が阻止すべき「変更途中の設計」に他ならず、「未定」のみを走査する方式では誤って通過させてしまい、第 4 層の凍結要求が 48 バイトの未確定計測条件を v1 として追認してしまうことになる。チェックリストが判定するのは前提条件の充足有無であり、いかなる項目の内容の当否も判定しない。

**根拠**：

- E142 (新規プールでのファイル作成のドライラン)：第 1 項が走査すべきフィールド表とは、それが行ごとに出力し行ごとに検証した表そのものであり、「幅」列および「状態」列における「預想」「半定」という 2 語は実在の対象を有している。
- メイン agent による確定（第 1 項の語彙が 2026 年に絞り込まれた経緯は変更履歴にある）；第 7 項の判定規則は確定項目 1 にある。

**未済の検査**：C62 (分项定案值在 kb 里分叉)；C8 (门禁范围判不出来)（チェックリスト第 6 項のもう半分である C4 (项目规则清单不一致) は清算済み）。

#### 確定項目 7：凍結は 4 層に分割し、4 層外に独立した 3 つの凍結コンポーネントを設ける

**確定内容**：4 層には厳格な依存順序が存在する。

```
第 1 層：システム構成 + ブロックヘッダ（自アドレス / generation / ツリー ID / fsid / magic）
    ↓
第 2 層：ポインタフィールドレイアウト（D4 チェックサム + D9 の MAC / nonce / 暗号文チェックサム）
    ↓
第 3 層：各 keyspace の key 符号化（D8 の locality_id / inode / offset）
    ↓
第 4 層：インデックスノード内部レイアウト（ノードサイズ規則、bset 形式、D11 のメッセージバッファ）
```

依存順序の理由：第 1 層は「メタデータブロックは自己記述でなければならない」という物理的前提であり、後続のすべての層はその上に構築される；第 2 層は D9 (暗号化) の MAC / nonce / 暗号文チェックサムが「事後追加不能」であることにより固定され、これに依存するツリー構造に先行しなければならない；第 3 層の key 符号化が定まって初めて、ノード内部でこれら key をどう編制するかの議論基盤が成立する。

**段階的凍結のリスクは明示的に受容されなければならない**：早期に凍結された層でフィールド確保を怠った場合、後続は incompat によるパッチ適用を余儀なくされ、全体設計時に確保すべきであった余力を前倒しで喪失することに等しい。損切り規則は唯一つである：**第 1 層および第 2 層の凍結前に D9 (暗号化) の全 4 つの予約フィールドを内包していなければならない**。これらは「いずれ必要となることが明白な」フィールドであり、第 3 層・第 4 層での補填を期待することはできないためである。

**4 層外の独立凍結コンポーネント**：4 層のいずれにも属さない恒久的なバイト列が 3 つ存在し、それぞれを独立したコンポーネントとして立脚させ、C9 (冻结后偷改格式) の `spec_hash` はコンポーネント単位で記録し、4 層側には影響を与えない：

- ① **パック記録ユニットのペイロードレコード形式**、パック記録種別ごとに登録：種別 2 の 140 バイト inode レコード表（D8 (コアインデックス構造) 確定項目 6）、凍結契機は種別 2 コンテナを書き出す最初のイメージ；種別 1 のもの（幅 40、D18 (ブロックに含める情報) 確定項目 24）は相互に独立しており、種別 2 と一括凍結は行わない。
- ② **統計量登録表**（タグ値 + 次元 + マージ規則、D5 (スナップショット・領域会計メカニズム) 確定項目 3 / 4）——マージ規則はディスク上セマンティクスであり、2 つのリーダーが同一のバイト列に対し異なる規則を適用すれば異なる値を算出してしまう；C71 (不认识的统计量标签静默忽略) は「未認識タグ」のみを保護し、「タグは認識するが誤った規則を適用する」事態は防げない。
- ③ **journal 記録形式**（レコードヘッダ、指定項目、ペイロードチェックサムの適用範囲、トランザクション境界）：凍結前提条件は第 2 層および配置先粒度セルより先行せず、凍結契機は当該 toml を提出するコミットとし、新規レコード形式は「新規レコード種別 + システム構成 log-incompat ビット」を採用する。全文は D16 (パブリッシュセマンティクス) 確定項目 3 にある。

**射程**：4 層の図は再編しない：D21 (権威態と派生態の境界) 確定項目 10 は状態区分を構造単位の判定へと変更した；第 4 層において会計ツリーのコード 2 ノードは権威態、他のツリーの内部ノードは派生態であり、純粋な層は存在しないため、凍結除外の単位は構造体であり層ではない（各構造体がどの層に属し、状態区分が何であり、凍結から除外されるか否かは `.claude/kb/freeze-layer-membership.md` に登録され、ゲート doc-decisions の freeze-layer-membership セルが判定する）；3 つのコンポーネントの spec ファイルは存在しない；コンポーネント ① の凍結契機は実行でありコミットではないため、確定項目 5 の遮断ゲートはそれに対して発火しない。ツリーテーブルユニットおよびインスタンス表ユニットがどの層に属するかは未だ条項が存在しない。

**根拠**：実験なし：層の分割と依存順序は「後から追加できない要素はどれか」から導出した順序付けであり、定量可能な数値は存在しない；損切り規則の対象（D9 (暗号化) の 4 つの予約フィールド）は D9 (暗号化) 確定項目 12 にある。メイン agent およびユーザーによる確定、原文は変更履歴にある。

**未済の検査**：C119 (冻结组件没有 spec 文件)；C225 (组件的冻结时刻绕开门禁触发)；C246 (树表单元没有冻结层归属)。

#### 確定項目 8：凍結後に失われる自由

**確定内容**：

| 流動フォーマット期の自由 | 凍結後に何へと変化するか |
|---|---|
| 構造を「解体して再構築」 | 凍結済みフィールドの幅 / 存在 / 意味は変更不可；追加のみ許容され、廃止フィールドは「読み出しサポート維持、書き込み停止」とする |
| 設計の誤りを発見時に互換性を残さず本文を直接修正 | 決定記録レベルでは依然として覆すことが可能であるが、**ディスク上バイト列レベルでは互換性コストがゼロから非ゼロへと転換する** |
| mkfs の自動選択規則でアルゴリズムを随時変更 | 凍結済みコンポーネントの選択規則自体も固定される；アルゴリズム変更により新旧イメージのデフォルト値が乖離するため、新規フォーマットバージョン番号の下で表現されなければならない |
| 実験結果が芳しくない場合にフォーマットを直接改訂 | 新規 feature bit を通じて新規ポリシーを導入し、旧値の解釈能力を保持しなければならない |
| フォーマットに合わせて checker を自由に変更 | **凍結部分について、checker の判定は事実上の互換性規約となる**——凍結フィールドに対する checker の解釈を密かに改変することは、フォーマットを密かに改変することと同等に危険である |

**射程**：凍結後にいかなる行為が非正当となるかを述べるにとどまり、個別の具体的変更がいずれのグレードに属するかは判定しない——それは確定項目 9 の所管である。最初の外部ユーザーが現れるまでフォーマットは流動的であり、本表はそれ以前には一切効力を持たない (`.claude/rules/format-evolution.md`)。

**根拠**：実験なし：「対外的な恒久公約は変更できない」という制約を項目ごとに展開したものであり、定量可能な数値は存在しない。最終行の規定（checker の判定も規約となる）は D13 (検証ルート) の「checker こそが仕様」と同源である。メイン agent による確定。

**未済の検査**：なし。

#### 確定項目 9：凍結後の変更判定表

**確定内容**：

| 変更分類 | 旧リーダーはどう振る舞うか | グレード | bump | 理由 |
|---|---|---|---|---|
| 予約領域 / TLV 末尾へのフィールド追加 | 無視し、正常動作を継続 | compat | 否 | セマンティクス依存のない純粋な追加であり、「未知のフィールドを無視する」セーフティネットで十分であるため |
| フィールド追加であり、その欠落により旧リーダーが**他の既存フィールド**を誤計算する場合 | **誤計算し、データを破損する恐れがある** | incompat | 是 | セマンティクスが結合した場合、セーフティネットは破綻するため |
| 既存フィールド幅の拡張 | 旧幅で解析し、範囲外読み出し / 位置ずれ | incompat | 是 | 幅はレイアウトの一部であり、旧コードのオフセット計算が完全に狂うため |
| 既存フィールドの意味の変更 | 旧解釈で処理し、**エラーを出さずに**不正な挙動を発生 | incompat、かつ**旧位置を廃棄し再利用禁止** | 是 | 位置の再利用は新旧リーダーの識別を不能とし、最も危険な分類であるため。実例 2 件（D18 (ブロックに含める情報) 確定項目 11）：ユニット種別の追加（登録表コード 4–15）は種別識別セグメントの解釈計測条件を変更する；共通プレフィックスオフセット 7 の常に 0 である flag ビットへの意味付与（旧リーダーは非ゼロビットを一律拒絶） |
| レコード種別の追加（新規 key 種別） | 読み出し専用トラバースが誤判定の恐れ；整理 / 書き込みが理解不能なレコードを誤削除する恐れ | compat_ro | 是 | **「読み出し専用の安全性」と「書き込み可能の安全性」は独立した二重の保証**であり、個別に判定しなければならないため。⚠️ パック記録ユニット内のパック記録種別は**本行に従わない**（D18 (ブロックに含める情報) 確定項目 11）：墓石系または inode 系のレコードのスキップは読み出し専用の回答を変化させるためデフォルトは incompat とする；登録時に「スキップしても読み出し専用の回答が一切不変である」と証明された種別のみ compat_ro へ降格し、compat_ro リーダーはスキャン再構築の実行を禁止する |
| keyspace の追加 | ツリーテーブルが day-1 から開放リストの場合：新規ツリーが見えないのみ；ツリー数がハードコードの場合：ツリーテーブルを読み出せない | **compat**（確定項目 2 によりツリーテーブルは day-1 から開放リスト化）。開放ツリーテーブルを確保済み → compat；未確保 → incompat | 是 | **「ツリーテーブルが拡張可能か否か」は凍結時に決定されなければならず**、ツリー追加時に未確保であることが判明する事態は許容されないため |
| ソート順序の変更 | 二分探索 / トラバース結果の誤り、**エラーなし** | incompat | 是 | ソート順序はインデックスの本質であり、ログ構造ノードの並べ替えは順序の不変性に直結するため |
| チェックサム / MAC アルゴリズムの変更 | D9 (暗号化) のアルゴリズム種別フィールドが確保済み → 未認識の種別に遭遇時は明示的に失敗 | 確保済み → compat_ro；未確保 → incompat | 是 | D9 (暗号化) 確定項目 12 第 4 項はこの種の変更のために用意された流路であるため |
| フィールドの廃止 | 影響を受けず、旧リーダーは旧方式で解釈を継続 | compat、かつ**パース用コードを恒久保持** | 否 | 一度ディスクに書き込まれたフィールドは、旧イメージが 1 つでも存在する限り恒久的に解釈可能でなければならないため |

**射程**：「旧リーダーがどう振る舞うか」を判定するものであり、「その変更を行うべきか否か」は判定しない。第 1 行の規定（予約領域への追加は compat）は真に予約領域が存在する場合にのみ成立し、インデックスノードヘッダには 2 バイトの予約領域しかなく、格納可能なフィールドは極めて限られる。

**根拠**：

- E59 (缓冲消息能不能只从单元重算)：墓石系レコードを 1 つスキップすると削除済み key がすべて復活し、復活数は削除回数に正確に一致する——「パック記録種別は compat_ro の行に従わずデフォルトは incompat である」という文言の定量的根拠。
- メイン agent による確定、行ごとの理由は表内に記載。

**未済の検査**：C102 (索引节点头没有预留区，加字段不是 compat)；C97 (改字段含义而不改宽度，全仓没有一处会红)。

#### 確定項目 10：feature bit のビット数・会計・割り当て権

**確定内容**：

- **ビット数**：3 つのビットマップ（各 **256 ビット**、`u64[4]`）をシステム構成に格納。ビットはフィールド幅より遥かに低コストであり（3×32 バイトはシステム構成にとって無視可能）、**予約は広めに確保する**。
- **会計**：`.claude/kb/feature-bits.md` に `ビット番号 | カテゴリ | 名称 | 導入バージョン | 導入コミット | 状態 | セマンティクス一言` を列挙；ビット番号の唯一の登録場所は確定項目 4 の表とする。
- **割り当て権は「先行マージ優先」とし、承認制としない**。ゲートによる強制：ビット番号は厳格に単調増加し、番号の飛び越し予約を禁止し、表に存在しないビット番号を参照するコードを禁止——機械的仲裁を行う。
- **一度使用したビットは再利用を禁止する。** 根拠は [invariants/README.md](../../../crates/singlefs-checker/invariants/README.md) の番号非回収規定より**さらに硬い**：番号は検索キーであり、回収による誤導は「文書を読む人間またはモデル」にとどまる；**feature bit はディスク上に物理的に実在したビットであり**、回収して新機能に再割り当てした場合、旧イメージを新コードでマウントした際に当該ビットが「新機能 X が有効」と誤読され——**ファイルシステムが実際に誤作動を起こす**（例：存在しない暗号化特徴を存在すると誤認し、存在しない鍵を探索する）。廃止されたビットは「廃止直前の最終使用セマンティクスに恒久固定」と表記し、新機能への付与を禁止する。

**射程**：ビットの分割・記録・回収可否を所管し、各ビットのセマンティクス（要求した決定に属する）は所管しない。

**根拠**：実験なし：ビット数および再利用禁止は「旧イメージが新コードにより読み出される」という物理的事実から導出したものであり、定量可能な数値は存在しない；256 ビットの空間コストはシステム構成にとって無視可能であり、本セルは節約競争を行わない。メイン agent による確定。

**未済の検査**：C183 (每套布局一个 incompat 位而盘上差不出字节)。

#### 確定項目 11：凍結に伴い整備すべき赤判定検査

**確定内容**：

1. **フォーマット凍結規約**：凍結済みコンポーネントごとに `spec_hash` を記録；ゲートは再計算して照合し、不一致時は今回の差分が「`feature-bits.md` に新規行が存在するか、または `version` が明示的に変更されている」ことと「`decisions.md` の対応条目が同一コミット内で変更されている」ことの**双方**を満たすことを要求する。いずれか一方でも欠落すれば赤判定とする。
2. **後方互換性リグレッション（ゴールデンイメージ）**：あるコンポーネントのバージョン変更ごとに**当時のコード**を用いてイメージを生成・アーカイブし、ゲートは**現行コード**を用いて全履歴イメージを読み出し、checker を実行し、期待されるコンテンツダイジェストを照合する。イメージ本体は **git に含めない** (`.claude/singlefs-ai-sop/rules/command-safety.md`：テストイメージのリポジトリ内格納を禁止)；リポジトリにはマニフェスト `.claude/kb/golden-images.toml` のみをコミットする；イメージは `${SINGLEFS_GOLDEN_DIR:-$HOME/.cache/singlefs-golden}` に配置する。マニフェストは**追加のみ許容**される——特定バージョンのリグレッション削除はその互換性公約を密かに放棄することに等しく、正式な決定記録を経なければならない。

**射程**：双方が未実装であり、番号は未済の検査表にある。第 2 項は「現行コードが過去のイメージを理解できるか」のみを比較し、「当時のイメージが正しく書き出されていたか」は比較しない；第 1 項は宣言に固定されバイト列には固定されない；双方は「最初のゴールデンイメージ自体が誤ったコードで出力された」事態を防ぐことはできない。

**根拠**：実験なし：2 つのゲート検査がいかなる判定を行うべきかを定めるものであり、判別力の自己証明は実装を待つ必要がある；これらが防ぐべき障害形態（フォーマットの不正改変、互換性公約の密かな放棄）は C9 (冻结后偷改格式) および C227 (黄金镜像的基准是它自己那份代码写的) にそれぞれ 1 条存在する。メイン agent による確定。

**未済の検査**：C9 (冻结后偷改格式)；C227 (黄金镜像的基准是它自己那份代码写的)。

````

**出处 `.claude/kb/decisions/15-フォーマット凍結ポリシー.ja.md:234-237`（整段抄，未转述）**

```markdown
## 履歴バージョン

D15 (フォーマット凍結ポリシー) の履歴項目は [decisions-history.md](../decisions-history.md) に集約されている。

```

**出处 `.claude/kb/decisions/15-格式冻结政策.md:1-6`（整段抄，未转述）**

```markdown
## D15 格式冻结政策 —— 已定

**中文** · [English](15-format-freeze-policy.en.md) · [日本語](15-フォーマット凍結ポリシー.ja.md)

D15（格式冻结政策） 管两个问题：**什么时候允许说「format v1 frozen」**，以及**冻结之后新增东西走什么通道**。它是 D11（索引节点要不要留消息缓冲区） / D12（目标介质） / D14（双轨（大小文件 / 持久临时）） 的共同前置——「格式分支永久」这条成本的当期值为零、冻结后为无穷，那几条决策的真实代价由它定义。不管的：每一条字段本身该定什么值归各自那条决策；第一个外部用户出现之前格式是软的，那条政策在 `.claude/rules/format-evolution.md`；`feature-bits.md` 里每一位的语义归申请那一位的决策，D15（格式冻结政策） 只管位号怎么分、能不能回收。

```

**出处 `.claude/kb/decisions/15-格式冻结政策.md:25-40`（整段抄，未转述）**

```markdown
#### 已定项 1：第 7 项的对象是未定分项，不是决策名单

**定案**：三条。

1. **对象是全仓每一条仍未定的分项，不是一份决策名单。** 每条未定分项要带一句判定——**动不动格式**：它的答案会不会改动任何盘上字节，或改动已有字节的**解释口径**（口径这半是 C97（改字段含义而不改宽度，全仓没有一处会红） 的形态：宽度一个字节不变而含义变了）。三个取值：**动 / 不动 / 界不定（视同动）**；**没写过视同动**。
2. **判定**：冻结请求发起时，凡判「动」或没判过的，判红。要分批冻结就在请求里**逐条列名并显式接受风险**（形态照已定项 7 那条「分批冻结的风险必须显式接受」），**列漏一条判红**。
3. **留白宽度默认 0**，举证责任在要留非零宽度的一方，必须过 `.claude/rules/fs-design.md` 的格式分支判据（逐字：「一条格式分支值不值得，看它能不能把『用错了』变成『挂不上』」）。

问法先定读法（`.claude/singlefs-ai-sop/skills/decide/SKILL.md` 硬要求 7：待定项的问法必须只有一种读法）：不取「逐条钉死六格的取值」（每一格的内容住在各自那条决策里，复制一份就是同一事实两处记录），取「定第 7 项的判定规则」；第 7 项是**逐次冻结重判**，因为清单是 commit 触发的前置检查，每一次冻结请求都跑一遍。

**射程**：留白宽度默认 0 这一条管的是**给一个已定不做的特性占位**，不管结构里的**通用预留区**——通用预留区由已定项 9 判定表第一行（新增字段在预留区 / TLV 尾部 ⇒ compat 不 bump）与 C102（索引节点头没有预留区，加字段不是 compat） 管，判据不同；把两者混成一条会推出「哪儿都别留」，而 C102（索引节点头没有预留区，加字段不是 compat） 正在往相反方向还账。升成规则的只有「默认 0 + 举证责任倒置」这套程序，任何一条走到「v1 不含此特性」那一支的条目，都要自己再过一遍格式分支判据。谓词不按层切：退出冻结按结构判、不按层判（已定项 7 的射程），且成员不齐——已定项 7 里那三个独立冻结组件不属于四层里的任何一层 ⇒ 按层切的谓词在它上面没有定义；按「动不动格式」这个二值谓词切，这两条毛病都碰不到，而且不假设层与层之间可以独立判定。D26（后台整理与放置回收） 已定项 4 那一格照旧由 D26（后台整理与放置回收） 自己拥有，D15（格式冻结政策） 无权替它答。判定输入一律现取，不在这里存副本：`grep -c '未定\*\*' .claude/kb/decisions.md` 数全仓还有几条未定分项（分项清单行末现为「—— **未定** · 未实现」这一形，不再以「未定**」收尾），`bash .claude/gate.d/doc-decisions.sh --check decision-items-sync` 判那张清单是不是正文的忠实投影。

**依据**：无实验：定的是门禁判定规则的对象与默认值，没有可量的量。撑它的是两条事实——名单形态漏得到（D22（单元原子性怎么合成） 已定项 9 是**系统配置的字段表**，而系统配置就是第 1 层；D19（块指针的结构与宽度预算）已定项 6 是**中央映射条目的字段表**，落在第 2 层——两条都不在那份六条名单里，冻这两层时名单一个字都说不上话），以及靠一个「格式级」标签去兜是兜不住的（编号的登记位在各决策正文的标题行、不在索引页，`grep -rn 格式级 .claude/gate.d/ .claude/scripts/` 零命中 ⇒ 零对象、零实现）。主 agent 定案，按三方论证；一条不成立的论据写在变更史里，免得再用。

**欠**：C62（分项定案值在 kb 里分叉）；C102（索引节点头没有预留区，加字段不是 compat）。

```

**出处 `.claude/kb/decisions/15-格式冻结政策.md:41-53`（整段抄，未转述）**

```markdown
#### 已定项 2：树表 day-1 做成开放列表

**定案**：树表 day-1 就做成开放列表。依据在已定项 9 那张判定表里：开放列表下「新增一棵 keyspace」对旧读者只是**看不到新树**（档次 compat，优雅降级）；树数量硬编码则是**读不出树表**（incompat，直接坏）。**开放列表严格占优**，代价只是树表单元里多一个表长字段加一段可变长表——与已定项 10「预留要往宽了留」是同一条理由。⇒ 加 keyspace 是本工程明写的扩展目标，不是意外，判定表里那一行的档次因此钉死为 **compat**。

**射程**：定的是「树表是开放列表」，与树表住哪无关——树表住根记录的间接层（D22（单元原子性怎么合成） 已定项 7 逐字「树根一律住树表单元，不平铺进根槽」、已定项 12 定「系统配置不带自己的树表单元指针」；D21（权威态与派生态的分界） 把根记录 + 树表单元 + 实例表单元一并划进「根」）。它**必须在第 1 层冻结时就做进去**，不能等到要加树才发现没留；而树表单元自己的冻结层归属仍欠。

**依据**：

- E142（新池新建文件的干跑）：新池新建文件写出的树表单元按条数声明（mkfs 那版 0 条、事务那版 11 条，是退役树加进之前的现值；退役树加进之后事务那版 12 条），表长是数据不是编译期常数——开放列表这个形态在写出来的字节上成立。
- 用户定案，原话在变更史。

**欠**：C246（树表单元没有冻结层归属）。

```

**出处 `.claude/kb/decisions/15-格式冻结政策.md:54-73`（整段抄，未转述）**

```markdown
#### 已定项 3：`spec_hash` 恒对 kb 字段表的完整投影求，不切到 Rust 结构体

**定案**：`spec_hash` 恒对 `format-spec/<组件>.toml` 求。那份 toml 是 **kb 字段表的完整投影**——含**行序**（盘上顺序）与**非标量的盘上语义规则**（已定项 7 里组件 ② 的合并规则以数据形式写进去），不是已登记的那批标量常量的集合。有代码之后新增的**不是换哈希源**，是两道检查：① spec 与代码之间的**相等性检查**（宽度与顺序都比）；② 黄金镜像回归（已定项 11 第 2 条）。

四条理由：

1. **方向**。D13（验证路线） 已定项 5 逐字：「**checker 与实现之间只共享一样东西：一份由 kb 的字段表生成的常量模块**，生成器从 kb 读、两边都只消费，任何人不许手改」⇒ 权威在 kb，Rust 结构体是**下游消费者**；对结构体求哈希，是把永久契约的指纹锚在派生物上。
2. **哈希是变更探测器，不是两份工件的相等性检查。** 对结构体求哈希抓不到「结构体与 kb 不一致」，它只在结构体**变化**时报警；真正抓那一类错的是相等性检查，而相等性检查要求 spec 侧存在 ⇒ 切过去不但没多买到东西，还把要比对的另一侧拆掉了。
3. **语言绑定**。D17（实现分层与第三方管道） 已定项 3 逐字：「判定必须能被第三方自己复核，否则『合规』就退化成『我们说你合规』」。指纹锚在 Rust 源码文本上，第三方管道复算不了它。
4. **落地**。三个组件的 spec 文件写得出来，哈希机制可以远早于第一次冻结开始生效；而已定项 7 里组件 ① 的冻结时刻是第一个写出类型 2 容器的镜像 ⇒ 切换会被逼到与第一次真实冻结同一个提交里完成，零试跑期。

**射程**：spec 只投影标量是不够的，必须含行序，相等性检查也要比顺序——[layout/01-first-txn.md](../layout/01-first-txn.md) 的单元头逐行列着六个宽度都是 8 的相邻字段（五元组的树 ID / 对象 ID / 对象出生代 / 锚点偏移，以及诞生代号、fsid），把序列化时任意两个对调，所有标量与 `DATA_UNIT_HEADER_BYTES` 一个都不变、盘上字节全错位，落进已定项 9 判定表「改变已有字段含义」那一行。D13（验证路线） 已定项 5 那条「生成器只发射标量值」的边界管的是 checker 与实现之间的共享物，而 `format-spec/<组件>.toml` 是**门禁工件**、运行期一个消费者都没有 ⇒ 不在那条边界的射程里，而且一张带行序的字段表既没有分支也没有算术。**抓不到的明说**：spec 与实际写出的字节之间，哈希不构成绑定——第一份黄金镜像若本身就是错代码写出来的，回归会把错的那份当基准；堵它的是 checker 这个独立解析器与一致性套件的判定程序，不是 `spec_hash`。spec 只能手写，没有任何从 kb 字段表投影到 toml 的机制。分批冻结「早冻层漏留字段」的**风险大小**给不出量化形式，它依赖尚不存在的 D8（核心索引结构） / D11（索引节点要不要留消息缓冲区） 具体布局，只能给出已定项 7 那条止损规则，残余风险无法量。

**依据**：

- E142（新池新建文件的干跑）：单元头那六个宽度都是 8 的相邻字段在写出来的字节里逐行排着，对调任意两个而全部标量不变这件事是可构造的——spec 只投影标量罩不住字段顺序。
- 三方论证：反推腿按跑前写死的失败条款判「spec 只投影标量」那条臂出局，定案取的是收严后的形态（spec 必须含行序），新形态比旧形态多要求一样东西、没少要求任何东西。主 agent 定案。

**欠**：C227（黄金镜像的基准是它自己那份代码写的）；C226（spec 是手抄的，没有从字段表生成）；C119（冻结组件没有 spec 文件）。

```

**出处 `.claude/kb/decisions/15-格式冻结政策.md:74-95`（整段抄，未转述）**

```markdown
#### 已定项 4：feature bit 的位分配登记表

**定案**：三个 bitmap 各 256 位（已定项 10 的三张位图），位号的唯一登记位是这一节的登记表。「不认识不许挂」要对默认布局也生效，所以第一条线自己占一位、mkfs 起就置上——不取「第一条线不占位」那条（那样不认识 SSD 布局的读者照样能挂）。

| bitmap | 位 | 含义 | 出处 |
|---|---|---|---|
| incompat | 0 | 第一条纯 SSD 布局线 | D12（目标介质） 已定项 5；D12（目标介质） 已定项 1「每套布局一个 incompat 位」；退役：带回退见证、系统配置不带 F 的那一版，读者一律按不认识的位拒（用户 2026-09-26） |
| incompat | 1 | 第一条纯 SSD 布局线（系统配置带回退下界 F、根记录带卸载记号、没有回退见证与回退行） | D12（目标介质） 已定项 5；用户 2026-09-26 定「换 incompat 位，挂不上」 |
| incompat | 2..255 | 未分配；每开一条布局线追加一位 | D12（目标介质） 已定项 1 |
| compat_ro | 0..255 | 未分配 | — |
| compat | 0..255 | 未分配 | — |

**射程**：只定位号怎么分与第一位的归属，每一位的语义归申请那一位的决策。新池新建文件的字节：系统配置 feature bits 那 96 字节从全 0 变成 incompat 位 1 置 1，位图小端、位 1 是第一个字节从最低位数起的第 2 位（D22（单元原子性怎么合成） 已定项 13）。这一项没有走三方论证，主 agent 的推荐与用户的选择都可推翻。

**依据**：

- E142（新池新建文件的干跑）：系统配置 feature bits 那 96 字节按这一项写出来，它的空白 G12 随之收口，[layout/01-first-txn.md](../layout/01-first-txn.md) 同步改。
- 用户定案，原话在变更史；定案之前那一格的原文也在变更史，含它当天写的「改字节：否」——那句已被这一项的定案推翻。
- 用户定案 2026-09-26：位 0 退役、布局身份换成位 1，旧镜像（带回退见证、系统配置不带 F）挂不上（三方判决 `research/prompts/m2-rollback-forward-r3-main-verification.md` 第二节 K7：「见证表非空就拒可写挂载」出局，只读恢复 13 个改坏深度里 4 个落到被回退掉的那一版；改系统配置格式版本号挡不住，今天的读者不核版本号），原话在变更史；无实验：旧读者对新位挂不上在原型上量过，新读者对旧位拒是推的；「改系统配置格式版本号挡不住，今天的读者不核版本号」这一句已过时：2026-09-27 起读者遇系统配置格式版本不是 1 一律拒收（D22（单元原子性怎么合成） 已定项 9）。

**欠**：C183（每套布局一个 incompat 位而盘上差不出字节）。

```

**出处 `.claude/kb/decisions/15-格式冻结政策.md:96-105`（整段抄，未转述）**

```markdown
#### 已定项 5：冻结是一次受门禁仲裁的提交，不是一句话

**定案**：**动作**是向仓库提交 `.claude/kb/format-frozen/<组件>.toml`（内容含 `version`、`frozen_at_commit`、`spec_hash`）。**谁判**：不是人「觉得可以了」就生效——门禁一旦发现本次 diff 触碰了这个目录，自动切换到一套更严的前置检查（清单在已定项 6），全部满足才允许该 commit 通过。**人只决定「现在要不要发起冻结」，机器决定「这次冻结请求成不成立」**，与 `.claude/singlefs-ai-sop/rules/show-me-test.md`「不按来源区分，只按证据区分」同构。

**射程**：那个阶段平时不跑（只在本次 diff 触碰这个目录时才跑），因此挂上去也不拖慢日常提交；**它注定常年触发 0 次，这不是缺陷，是正确状态**——与门禁显式列出未实现阶段是同一种诚实。⚠️ 它是 commit 触发的，而已定项 7 里组件 ① 的冻结时刻是「第一个写出类型 2 容器的镜像」，那是一次运行、不是一次提交 ⇒ 冻结前清单对组件 ① 没有执行路径，门禁永远不会为它点火。

**依据**：无实验：定的是门禁触发时机与仲裁归属，没有可量的量；形态与共享 `gate.sh` 已有的「项目本地阶段」挂载点对齐。主 agent 定案。

**欠**：C225（组件的冻结时刻绕开门禁触发）。

```

**出处 `.claude/kb/decisions/15-格式冻结政策.md:106-132`（整段抄，未转述）**

```markdown
#### 已定项 6：冻结前必须完成的清单

**定案**：七项前置，冻结请求发起时逐项判，逐次冻结重判。

| # | 项 | 不做它冻结会怎样 |
|---|---|---|
| 1 | 本次要冻的组件，它的字段表在 kb 里没有一行写着「未定」「预想」「半定」「占位」 | 冻结的是一个还在动的设计；此后每次补答一个待定项都会被追认成 v1 的一部分。⚠️ 它与第 7 项是两把尺子：第 1 项量**已经写下来的那张表**齐不齐，第 7 项量**还没写下来的那些分项**动不动它 |
| 2 | I-1（块自描述）、I-2（校验和） 类（块自描述、校验和）checker 已实现且跑绿 | 「checker 即规范」——没有 checker，「v1 是什么」只存在于文档里，后续改动无法被机械判定 |
| 3 | D9（加密） 已定项 12 的四项预留字段在 checker 里可断言「存在且宽度符合预留」 | 漏留一项，冻结后只能靠 incompat 打补丁，且已有旧镜像永远缺这个字段 |
| 4 | I-3.1（已分配统计对得上） 已实现且跑过 N≥5 轮全绿 | 记账正确性唯一的验证手段就是这条对照；没跑过，冻结的账本字段是不是设计对了完全没被检验 |
| 5 | 模型对拍与崩溃点重放 各至少跑过一轮 PASS，**且跑的正是即将冻结的那份格式指纹** | 否则 v1 描述的是一份从未被任何自动化验证碰过的格式 |
| 6 | [checks-owed.md](../checks-owed.md) 的 C4（项目规则清单不一致） 与 C8（门禁范围判不出来） 已实现 | 冻结后规则脱节、门禁越跑越慢却没人量化，会在风险更高的阶段变成事故 |
| 7 | 全仓每一条仍未定的分项都带着「动不动格式」那句判定，且没有一条判「动」或没判过——除非本次冻结请求把它们逐条列名并显式接受风险 | 这条链互相定义，沉默地带着「待定」冻结等同于假装它已经决定了。判定规则见已定项 1 |

⚠️ **清单里每一项的复核，取值一律从对应分项的定案行现取，不许手抄。** 冻结是不可逆的，而抄错一个数在文本上没有任何地方看起来别扭 ⇒ **复核时逐字打开那条分项的定案行**；机检形态与它的前置记在 C62（分项定案值在 kb 里分叉），在那条实现之前这一步靠人。

**第 1 项与第 7 项的可执行形态**：第 1 项扫本次要冻组件的字段表——[layout/01-first-txn.md](../layout/01-first-txn.md) 那张按盘上顺序逐行列字段的表就是它的现成形态，扫它的**「宽度」列与「状态」列**，出现 `未定` / `预想` / `半定` / `占位` **任一**即判红。第 7 项扫全仓未定分项里那句「动不动格式」的判定。它拆成两半：**写没写**那一半是门禁阶段 `.claude/gate.d/doc-decisions.sh` 的 blocking-verdict 格的第二把尺，与那一道的第一把尺（改不改新池新建文件的字节）共用同一次解析、同一份权威清单，各报各的数、各带各的出路，常跑；**判红**那一半（凡判「动」或没写过即判红，除非逐条列名并显式接受风险）是冻结请求发起时的事，前置是 C225（组件的冻结时刻绕开门禁触发）——冻结请求这个触发对组件 ① 没有执行路径。两项都**不另建第二份清单**——`.claude/singlefs-ai-sop/rules/kb-discipline.md`：同一事实只许一处权威记录。

**射程**：第 1 项的词表写进条款、不留给写脚本的人猜——字节表今天用的词是「**预想**」与「**半定**」（extent 树节点那一行逐字「**预想 162** = 86 + 2 × 24 + 28」、状态列「半定」，带不带 key 区间还没定，带 162、不带 114，差 48 字节，条目区起点跟着挪），那正是第 1 项要拦的「还在动的设计」，而按只扫「未定」的形态它一定放行，第 4 层的冻结请求就能把一个 48 字节的未决口径追认成 v1。清单判的是前置齐不齐，判不了任何一条分项的内容对不对。

**依据**：

- E142（新池新建文件的干跑）：第 1 项要扫的那张字段表就是它逐行写出来又逐行核过的那张，「宽度」列与「状态」列上「预想」「半定」这两个词有真实对象。
- 主 agent 定案（第 1 项词表 2026 年那次收窄的经过在变更史），第 7 项的判定规则在已定项 1。

**欠**：C62（分项定案值在 kb 里分叉）；C8（门禁范围判不出来）（清单第 6 项的另一半 C4（项目规则清单不一致） 已还清）。

```

**出处 `.claude/kb/decisions/15-格式冻结政策.md:133-162`（整段抄，未转述）**

````markdown
#### 已定项 7：冻结分四层，四层之外另有三个独立冻结组件

**定案**：四层有严格依赖顺序。

```
第 1 层：系统配置 + 块头（自身地址 / generation / 树 ID / fsid / magic）
    ↓
第 2 层：指针字段布局（D4 校验和 + D9 的 MAC / nonce / 密文校验和）
    ↓
第 3 层：各 keyspace 的 key 编码（D8 的 locality_id / inode / offset）
    ↓
第 4 层：索引节点内部布局（节点大小规则、bset 形态、D11 的消息缓冲区）
```

依赖方向的理由：第 1 层是「元数据块必须自描述」的物理前提，后面所有层都建在它之上；第 2 层被 D9（加密） 的 MAC / nonce / 密文校验和「事后加不进去」钉死，必须先于依赖它的树结构；第 3 层的 key 编码定了，节点内部怎么组织这些 key 才有讨论基础。

**分批冻结的风险必须显式接受**：早冻的层若漏留字段，后面只能走 incompat 打补丁，等价于提前烧掉一次本该整体设计时留的余量。止损规则只有一条：**第 1、2 层冻结前必须已包含 D9（加密） 全部四项预留**，因为它们恰好是「已知迟早要用」的字段，不能指望第 3、4 层再补。

**四层之外的独立冻结组件**：有三样永久字节不属于四层里的任何一层，各立成一个组件，C9（冻结后偷改格式） 的 `spec_hash` 按组件记，四层不动——

- ① **打包记录单元的载荷记录格式**，按打包记录类型登记：类型 2 的 140 字节 inode 记录表（D8（核心索引结构） 已定项 6），冻结时刻是第一个写出类型 2 容器的镜像；类型 1 的（宽 40，D18（块里携带什么信息） 已定项 24）各自独立，不因类型 2 一起冻结。
- ② **统计量登记表**（标签值 + 维度 + 合并规则，D5（快照 / 空间记账机制） 已定项 3 / 4）——合并规则是盘上语义，两个读者对同一批字节用不同规则会算出不同的值，C71（不认识的统计量标签静默忽略） 只罩「不认识的标签」，罩不到「认识标签但用错规则」。
- ③ **journal 记录格式**（记录头、点名项、载荷校验和的覆盖范围、事务边界）：冻结前置不早于第 2 层与落点粒度那一格，冻结时刻是提交它那份 toml 的 commit，新记录格式走「新记录类型 + 系统配置 log-incompat 位」，全文在 D16（发布语义） 已定项 3。

**射程**：四层图不重排：D21（权威态与派生态的分界） 已定项 10 把态别改成逐结构判，第 4 层里记账树的码 2 节点是权威态、别的树的内部节点是派生态，没有一层是纯的，所以退出冻结的单位是一个结构、不是一层（每个结构落哪一层、态别是什么、退不退出冻结，登记在 `.claude/kb/freeze-layer-membership.md`，门禁 doc-decisions 的 freeze-layer-membership 格判）；三个组件的 spec 文件都不存在；组件 ① 的冻结时刻是一次运行不是一次提交，已定项 5 那道闸对它不点火。树表单元与实例表单元落哪一层还没有条款。

**依据**：无实验：分层与依赖顺序是从「哪一样事后加不进去」推出来的排序，没有可量的量；止损规则那一条的对象（D9（加密） 四项预留）在 D9（加密） 已定项 12。主 agent 与用户定案，原话在变更史。

**欠**：C119（冻结组件没有 spec 文件）；C225（组件的冻结时刻绕开门禁触发）；C246（树表单元没有冻结层归属）。

````

**出处 `.claude/kb/decisions/15-格式冻结政策.md:163-180`（整段抄，未转述）**

```markdown
#### 已定项 8：冻结之后失去哪些自由

**定案**：

| 软格式期的自由 | 冻结后变成什么 |
|---|---|
| 结构「拆了重做」 | 已冻结字段的宽度 / 存在性 / 含义不能再改；只能新增，退役字段走「保留读支持、停止写」 |
| 发现设计错了直接改正文不留兼容 | 决策记录层面仍可推翻，但**盘上字节层面从 0 兼容成本变成非零** |
| mkfs 自动选取规则可随时换算法 | 已冻结组件的选取规则本身也被钉死；换算法会让新旧镜像默认值不同，必须体现在新格式版本号下 |
| 实验结果不理想就直接改格式重定 | 只能通过新 feature bit 引入新策略，并保留旧值的解释能力 |
| checker 可以跟着格式自由改 | **对已冻结部分，checker 的判断变成事实上的兼容性契约**——悄悄改 checker 对冻结字段的解释，和悄悄改格式一样危险 |

**射程**：只说冻结之后哪些动作不再合法，不说某一次具体改动该判哪一档——那归已定项 9。第一个外部用户出现之前格式是软的，这张表在那之前一条都不生效（`.claude/rules/format-evolution.md`）。

**依据**：无实验：它是「对外的永久承诺改不掉」这条约束的逐条展开，没有可量的量。最后一行那条（checker 的判断也成契约）与 D13（验证路线） 的「checker 即规范」同源。主 agent 定案。

**欠**：无。

```

**出处 `.claude/kb/decisions/15-格式冻结政策.md:181-205`（整段抄，未转述）**

```markdown
#### 已定项 9：冻结后的改动判定表

**定案**：

| 改动类型 | 旧读者会怎样 | 哪档 | bump | 理由 |
|---|---|---|---|---|
| 新增字段在预留区 / TLV 尾部 | 忽略，继续正常工作 | compat | 否 | 没有语义依赖的纯加法，「忽略陌生字段」这条安全网够用 |
| 新增字段，缺了会让旧读者对**另一个已有字段**算错 | **算错，可能损坏数据** | incompat | 是 | 语义一旦耦合，安全网失效 |
| 扩大已有字段宽度 | 按旧宽度解析，越界读 / 错位 | incompat | 是 | 宽度是布局的一部分，旧代码的偏移计算全错 |
| 改变已有字段含义 | 按旧含义解读，**不报错**地产生错误行为 | incompat，且**弃用旧位置不复用** | 是 | 位置复用会让新旧读者无法区分，是最危险的一类。两个实例（D18（块里携带什么信息） 已定项 11）：新增一个单元类（登记表码 4–15）改的是类身份段的解析口径；给共同前缀偏移 7 里恒 0 的 flag 位赋义（旧读者对非 0 位一律拒收） |
| 新增记录类型（新 key 类型） | 只读遍历可能误判；整理 / 写可能误删看不懂的记录 | compat_ro | 是 | **「只读安全」与「可写安全」是两条独立保证**，必须分开判。⚠️ 打包记录单元内的打包记录类型**不走这一行**（D18（块里携带什么信息） 已定项 11）：跳过一个墓碑类或 inode 类记录会改变只读答案，默认档是 incompat；只有登记时能证明「跳过它不改变任何只读答案」的类型才降到 compat_ro，且 compat_ro 读者不许跑扫描重建 |
| 新增一棵 keyspace | 若树表 day-1 就是开放列表：只是看不到新树；若树数量硬编码：读不出树表 | **compat**（已定项 2 定树表 day-1 做成开放列表）。留了开放树表 → compat；没留 → incompat | 是 | **「树表是否可扩展」必须在冻结时决定**，不能等要加树才发现没留 |
| 改变排序规则 | 二分查找 / 遍历结果错，**不报错** | incompat | 是 | 排序规则是索引本体，日志结构节点的重排直接依赖它不变 |
| 改变校验和 / MAC 算法 | D9（加密） 的算法类型字段已留 → 遇到不认识的类型只能显式失败 | 已留 → compat_ro；未留 → incompat | 是 | D9（加密） 已定项 12 第 4 项正是为这类改动准备的通道 |
| 退役一个字段 | 不受影响，旧读者继续按旧方式解释 | compat，且**永久保留解析代码** | 否 | 落过盘的字段，只要还有一份旧镜像存在就必须一直能被解释 |

**射程**：判的是「旧读者会怎样」，不判「这个改动该不该做」。第一行那条（预留区里的新增是 compat）只在真有预留区时成立，索引节点头只有 2 字节预留，装得下的字段很少。

**依据**：

- E59（缓冲消息能不能只从单元重算）：跳过一个墓碑类记录会让已删的 key 全数复活、复活数恰等于删除次数——「打包记录类型不走 compat_ro 那一行、默认档是 incompat」这半句的量。
- 主 agent 定案，逐行的理由写在表里。

**欠**：C102（索引节点头没有预留区，加字段不是 compat）；C97（改字段含义而不改宽度，全仓没有一处会红）。

```

**出处 `.claude/kb/decisions/15-格式冻结政策.md:206-220`（整段抄，未转述）**

```markdown
#### 已定项 10：feature bit 的位数、记账与分配权

**定案**：

- **位数**：三个 bitmap 各 **256 位**（`u64[4]`），存系统配置。位比字段宽度便宜得多（3×32 字节对系统配置可忽略），**预留要往宽了留**。
- **记账**：`.claude/kb/feature-bits.md` 列 `位号 | 类别 | 名称 | 引入版本 | 引入 commit | 状态 | 语义一句话`；位号的唯一登记位是已定项 4 那张表。
- **分配权是「谁先合并谁先占」，不是审批制**。门禁强制：位号严格递增、不许跳号占位、不许有代码引用了表里不存在的位号——机械仲裁。
- **位一旦用过不许回收。** 理由比 [invariants/README.md](../../../crates/singlefs-checker/invariants/README.md) 编号不回收**更硬**：编号是检索键，回收误导的是「读文档的人或模型」；**feature bit 是磁盘上真实存在过的物理位**，回收后重新分配给新特性，一个旧镜像挂到新代码上这一位会被误读成「新特性 X 已启用」——**文件系统会真的做错事**（例如把不存在的加密特征当存在，去找不存在的密钥）。退役的位标为「含义永久锁定为退役前最后一次使用的语义」，不许赋予新特性。

**射程**：管位怎么分、怎么记、能不能回收，不管每一位的语义（归申请那一位的决策）。

**依据**：无实验：位数与不回收都是从「旧镜像会被新代码读到」这条物理事实推出来的，没有可量的量；256 位的空间代价对系统配置可忽略，这一格不比谁省。主 agent 定案。

**欠**：C183（每套布局一个 incompat 位而盘上差不出字节）。

```

**出处 `.claude/kb/decisions/15-格式冻结政策.md:221-233`（整段抄，未转述）**

```markdown
#### 已定项 11：冻结要配的两条会红的检查

**定案**：

1. **格式冻结契约**：每个已冻结组件记 `spec_hash`；门禁重算并比对，不同则要求本次 diff **同时**满足「`feature-bits.md` 有新增行或 `version` 被显式改动」与「`decisions.md` 对应条目在同一 commit 里被改动」。缺一判红。
2. **向后兼容回归（黄金镜像）**：每次某组件版本变化时用**当时的代码**生成一份镜像并归档，门禁用**当前代码**去读全部历史镜像、跑 checker、比对期望内容摘要。镜像本体**不进 git**（`.claude/singlefs-ai-sop/rules/command-safety.md`：测试镜像不许放仓库内），仓库只提交清单 `.claude/kb/golden-images.toml`；镜像放 `${SINGLEFS_GOLDEN_DIR:-$HOME/.cache/singlefs-golden}`。清单**只增不删**——删掉某版本的回归等于悄悄放弃对它的兼容承诺，必须走决策记录。

**射程**：两条都还没实现，编号在欠账表里。第 2 条只比「今天的代码读不读得懂历史镜像」，不比「当初那份镜像写对没有」；第 1 条锚在声明上、不锚在字节上，两条都罩不到「第一份黄金镜像本身就是错代码写出来的」。

**依据**：无实验：定的是两道门禁检查该判什么，判别力自证要等它们有实现；它们要罩的失败形态（偷改格式、悄悄放弃兼容承诺）在 C9（冻结后偷改格式） 与 C227（黄金镜像的基准是它自己那份代码写的） 里各有一条。主 agent 定案。

**欠**：C9（冻结后偷改格式）；C227（黄金镜像的基准是它自己那份代码写的）。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:15-33`（整段抄，未转述）**

```markdown

| # | 分项 | 定案 |
|---|---|---|
| 1 | **位置条目带不带设备身份** | 带：每个物理位置各带一份 dev id，不引一层 chunk 映射表 **状态：已定。** **进度：已实现。** |
| 2 | **密文校验和的位宽** | 32 位；取完整 CRC32C，不许由更宽的 CRC 截断而来 **状态：已定。** **进度：已实现。** |
| 3 | **定宽还是变长** | 定宽，不加密时留位：MAC 与 nonce 恒定占位，留成空位、不挪作他用 **状态：已定。** **进度：已实现。** |
| 4 | **位置条目各段的宽度** | 设备 ID 4 字节、物理偏移 6 字节（16 KiB 槽号）、密文校验和 4 字节 ⇒ 位置条目 14 字节 **状态：已定。** **进度：已实现。** |
| 5 | **位置的权威住哪** | 位置条目降为提示，中央映射是解引用与释放判定的唯一入口 **状态：已定。** **进度：未实现。** |
| 6 | **中央映射条目的字段表** | key 取按类复用写序键（收严后）：码 1 为 27 字节、码 2 / 码 3 为 25 字节；value 照已定项 4 / 5 **状态：已定。** **进度：已实现。** |
| 7 | **指针头部带哪几段** | 头部 50：MAC 16、nonce 12、算法类型 1、压缩算法码 1、压后长度 2、extent 偏移 2、出生树 8、出生 txg 8 **状态：已定。** **进度：已实现。** |
| 8 | **引用侧携带多少出生身份，映射装哪几类** | 全部指针进映射，指向码 1 的指针 88、指向码 2 / 码 3 的 86；映射树节点、树表单元、实例表单元自举豁免 **状态：已定。** **进度：已实现。** |
| 9 | **出生序号怎么发、按什么次序发** | 树内从 0 起、码 2 与码 3 共用一个计数、换 checkpoint 清零、重写换号；次序同提交内生块的 bump 次序 **状态：已定。** **进度：已实现。** |
| 10 | **映射 key 定宽还是按类变长** | 一律 27 字节，码 2 / 码 3 末尾补零；中央映射条目一律 55 字节 **状态：已定。** **进度：已实现。** |
| 11 | **指针的偏移表、位置条目的顺序、映射树的根住哪** | 段按偏移表排，位置条目按设备身份升序；映射树根住根记录，根记录合计 457 **状态：已定。** **进度：已实现。** |
| 12 | **码 2 / 码 3 的映射条目 day-1 写不写，固定点写几次** | day-1 写出；一个实例在一个 checkpoint 里每个码 2 节点与码 3 容器只写出一次 **状态：已定。** **进度：已实现。** |
| 13 | **块指针分几段** | 分两段：逻辑属性住头部、每逻辑 extent 一份，物理属性住位置条目、每物理位置一份 **状态：已定。** **进度：已实现。** |
| 14 | **位置条目能不能用几何公式从单一起点推算** | 不能，逐个列出 **状态：已定。** **进度：已实现。** |
| 15 | **extent 距起点偏移的值域与第一版取值** | 值域 [0, 32768)、15 位够；第一版恒 0，位照留 **状态：已定。** **进度：已实现。** |

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:17-46`（整段抄，未转述）**

```markdown

| # | 分项 | 定案 |
|---|---|---|
| 1 | **journal 的角色本身** | 取甲：每次 fsync 写脏叶 + 全部祖先 + 根槽 + 一条记录，祖先不延后；轴一与轴二不正交 **状态：已定。** **进度：已实现。** |
| 2 | **journal 是定长环还是链式** | 取定长环；决定性理由是链式把死锁 3 的判据从静态换成动态 **状态：已定。** **进度：已实现。** |
| 3 | **tail 住哪、谁是权威、什么时候持久** | 住系统配置槽，≥2 槽轮换，随每次发布写一次；恢复不许先信 tail，重放必须幂等，恢复救不了几何 **状态：已定。** **进度：已实现。** |
| 4 | **记录头能不能约束在单个原子单元内** | 能；头 487 字节 = 十个字段 78 + 四笔增量 21（事务边界 9、本次发布内序号 4、反向链 4、载荷校验和 4）+ 新根段 364 + fsid 8 + MAC 16 **状态：已定。** **进度：已实现。** |
| 5 | **O2（独立解析器 + checker） 到底是什么** | 一元谓词；否掉逻辑意图日志的第二条理由改写成第一条的封堵；反向判据的主语是记录核对器 **状态：已定。** **进度：无对象。** |
| 6 | **目标负载优先级** | 移出 journal 这条线，见 D25（目标负载优先级）；它不阻塞 journal 的任何一项 **状态：已定。** **进度：无对象。** |
| 7 | **一个事务恰不恰好产生一条 journal 记录** | 可以跨多条，记录头带事务号与提交标记；重放丢掉提交标记没出现的尾巴 **状态：已定。** **进度：已实现。** |
| 8 | **记录里分辨时间线的字段** | 记录头带一条反向链（前一条记录的 hash），宽 32 位 **状态：已定。** **进度：已实现。** |
| 9 | **序号与实例代号的位宽划分** | jsn = 实例代号 32 位 + 计数器 48 位，两个字段共 10 字节；代号不许回滚 **状态：已定。** **进度：已实现。** |
| 10 | **`previous_hash` 覆盖哪些字段** | 覆盖前一条记录的整个记录头，其中含 jsn，因而含实例代号 **状态：已定。** **进度：未实现。** |
| 11 | **反向链的 hash 算法** | 取完整 32 位 CRC32C，与密文校验和同一个函数 **状态：已定。** **进度：已实现。** |
| 12 | **记录尺寸** | 4 KiB 定长，写进系统配置 **状态：已定。** **进度：已实现。** |
| 13 | **载荷的完整性靠什么** | `header_csum` 罩整条记录 [0, 4096)；`payload_csum` 保留、罩点名项数组；任一不过即该记录无效 **状态：已定。** **进度：已实现。** |
| 14 | **重放的下界、管理员回退、失败处置与实例切换** | 只施加 (实例代号, checkpoint_txg) 严格大于所选根的记录；前缀口径五条；切换按一次恢复做，管理员回退是挂着时的一次向前发布 **状态：已定。** **进度：未实现。** |
| 15 | **崩在记录持久之后、根槽持久之前，恢复施加什么** | 由记录重建那次发布的根：记录头带 364 字节新根段，施加一条记录 = 换掉所选根的七个字段；一次发布的每条记录带同一份 **状态：已定。** **进度：已实现。** |
| 16 | **mkfs 与第一次可写挂载的实例代号** | mkfs 写 0（不是有效实例），第一次可写挂载取 1、写进每一份系统配置并过一道完成的屏障之后才动单元 **状态：已定。** **进度：已实现。** |
| 17 | **点名项 56 字节的字段表** | 一项一个单元：位置条目 14 × 2 + 类型 1 + 出生树 8 + 出生 txg 8 + key 尾段 10 + flags 1 **状态：已定。** **进度：已实现。** |
| 18 | **在飞记录数上限与 tail 的编码** | 上限 = 环槽数 ÷ F，含义是一次恢复重放的前缀最多 N 条；坏校验和一律断链即止；tail 存计数器 **状态：已定。** **进度：已实现。** |
| 19 | **空发布记录、`previous_hash`、环长** | 事务号 0 留给空发布；链按本实例逻辑前一条算；环长是 mkfs 参数，默认 768 MiB、≤ 容量 ÷ 4 **状态：已定。** **进度：已实现。** |
| 20 | **记录承载什么、分配与记录谁先** | 记录承载指针层的目标态，不是逻辑意图、也不是全量物理重做；分配先于 journal **状态：已定。** **进度：已实现。** |
| 21 | **记录头的两个计数字段** | jsn 每条记录全局单调、前缀判定靠它；checkpoint_txg 每个 checkpoint 一个；两者不可合并 **状态：已定。** **进度：已实现。** |
| 22 | **前缀判定** | `jsn == expected`，不等即止；另有一个显式的在飞记录数上限 **状态：已定。** **进度：已实现。** |
| 23 | **三个死锁各自怎么破** | 死锁 1 靠读法甲，死锁 2 靠保留池，死锁 3 靠几何不变量：最坏占用 ≤ 环大小 / F，F ≥ 2 **状态：已定。** **进度：已实现。** |
| 24 | **死锁 2 与假性 ENOSPC 是两种故障** | 卡死靠保留池，reserve ≥ ckpt_cost 就够；假性 ENOSPC 只有准入治得了，保留池反而加重它 **状态：已定。** **进度：已实现。** |
| 25 | **D16（发布语义）「不绕过 journal」那句取哪个读法** | 取读法甲：语义效果必须经 journal，发根不是一条 journal 追加，是覆写根环的一个槽 **状态：已定。** **进度：已实现。** |
| 26 | **记录头的两条解析纪律** | 校验和覆盖范围是显式盘上常量，不由 sizeof 推；自述长度先做边界检查再用于校验和 **状态：已定。** **进度：已实现。** |

```

**出处 `.claude/kb/decisions/13-验证路线.md:8-28`（整段抄，未转述）**

```markdown

| # | 分项 | 定案 |
|---|---|---|
| 1 | **crash refinement 先验哪一块** | 先验「一次发布 + 一次崩溃 + 一次恢复」这个封闭单元，第一个里程碑是任一 `jsn` 连续前缀不切开发布；挂载透明另立第四条性质；原候选 I-6.5（nonce 水位单调） 撤下 **状态：已定。** **进度：已实现。** |
| 2 | **事务层承不承诺串行提交** | 先串行，越过交点再开并行；将来并行提交时记录按事务号顺序追加 **状态：已定。** **进度：已实现。** |
| 3 | **自写参照实现（RefFS 形态）做到什么程度** | 做到极致取语义级：只在内存里按 POSIX 操作走，自己定义持久与崩溃后允许的状态，没有盘上格式；照 kb 另起、单独一个包、只收白名单输入 **状态：已定。** **进度：已实现。** |
| 4 | **崩溃点重放的崩溃状态集合怎么定义** | 写流按设备切段，段里每块有写的盘都被自己的屏障或 FUA 放行才关段；段内每次整写各取几态、任意组合，原地覆写多一态「新旧都读不出」；没持久放旧字节且真生成镜像；双写按设备各算 **状态：已定。** **进度：已实现。** |
| 5 | **checker 与被测实现允许共享什么** | 只共享一份从 kb 生成的常量模块，其余各写一份；判定宽度由 checker 自己探测再与系统配置比对，探不到报「声明值，未探测」 **状态：已定。** **进度：已实现。** |
| 6 | **三个互相独立的 oracle** | 内存模型对拍、独立解析器 + checker、独立规约执行器三个 oracle 各判一样；独立性要靠「三 oracle 合谋测试」验证，不许自称 **状态：已定。** **进度：已实现。** |
| 7 | **O2（独立解析器 + checker） 判什么** | O2（独立解析器 + checker） 的定义域是单个镜像，只判集合成员；需要第二个输入的检查归记录核对器（入参含枚举器给的持久集合），故意不给编号 **状态：已定。** **进度：已实现。** |
| 8 | **自研的两样** | Merkle 根链差分（相邻两代根递归比较，指针没变处不下钻）与代际增量 scrub（只遍历 birth generation 高于水位的块） **状态：已定。** **进度：未实现。** |
| 9 | **崩溃点重放怎么分层** | 三层都精确全量，枚举不完靠精确剪枝，一时跑不了的节点标 ignore 慢慢开；里程碑收尾与判器单元有变的那批提交前弹窗请用户跑全量档，红或不跑照样提交、记未结；先保失败可复现 **状态：已定。** **进度：已实现。** |
| 10 | **真正必须精确覆盖的 N 是什么** | 提交协议结构的等价类数，不是布局套数；落地是一个封闭的提交步骤枚举加穷尽 match **状态：已定。** **进度：已实现。** |
| 11 | **ignore 的节点什么时候开** | 否决按欠账轮转的账本；留下三条改管开的时机：确定性的覆盖记录、`T1`（常规层 挂钟预算） 是输入、判别力与覆盖分开检查 **状态：已定。** **进度：无对象。** |
| 12 | **冲突 1：checker 即规范与校验路径不许共享工具** | checker 做遍历与扫描两个方向，对同一条不变量各自独立判，共享的只有格式解析 **状态：已定。** **进度：已实现。** |
| 13 | **冲突 2：refinement 的 spec 与对拍的 model** | spec 与 model 在表述层面就写得不同；同源性要观测，≥ 5 次不一致而判 spec 错 0 次就合并、预算移给穷举 **状态：已定。** **进度：无对象。** |
| 14 | **与其他决策的连锁** | 哈希与 AEAD 在形式验证里只能当 uninterpreted function；日志结构三层绑架符号执行；checker 不必解析消息语义 **状态：已定。** **进度：无对象。** |
| 15 | **验证代码分两档** | harness 档（`singlefs-harness`）是单元与集成测试、随时跑；checker 档（`singlefs-checker-tier`）默认只在提交时跑快档，全量用户要求时跑 **状态：已定。** **进度：已实现。** |
| 16 | **按段登记的段怎么分片、态数用多宽** | 登记段整域装不进 u64 就按片依次判，片大小默认 2^32 态、片起点是库里的续点；态数一律用 1280 位定宽整数、算术全 checked；登记表只经环境变量读 **状态：已定。** **进度：未实现。** |
| 17 | **判定复用的单元是什么** | 剪枝的单元是函数：一份判定沿不沿用只看它依赖的函数单元各自去注释后的指纹、工具链、解析出的依赖版本；按提交或树算改动、整文件原文、文件级与包级指纹、手写版本号、路径名字进键、整道标记一律不许 **状态：已定。** **进度：未实现。** |

```

**出处 `.claude/kb/decisions/15-格式冻结政策.md:8-24`（整段抄，未转述）**

```markdown

D15（格式冻结政策）的分项只有一套编号，不重排。

| # | 分项 | 定案 |
|---|---|---|
| 1 | **冻结前清单第 7 项的具体处置** | 第 7 项不写决策名单，写成一句判定——全仓每条未定分项自己声明「动不动格式」，冻结请求与它相交即判红；留白宽度默认 0 **状态：已定。** **进度：无对象。** |
| 2 | **树表是不是开放列表** | 是，day-1 就做成开放列表 **状态：已定。** **进度：已实现。** |
| 3 | **`spec_hash` 的来源** | 不切换。恒对 `format-spec/<组件>.toml` 求，那份 toml 是 kb 字段表的完整投影，含行序与非标量的盘上语义规则 **状态：已定。** **进度：未实现。** |
| 4 | **feature bit 的位分配登记表** | 登记表住已定项 4；incompat 位 1 = 第一条纯 SSD 布局线（带 F 与卸载记号那一版），mkfs 起就置上，位 0 退役；compat_ro、compat 一位都没分 **状态：已定。** **进度：已实现。** |
| 5 | **冻结这个动作是什么** | 冻结 = 提交一份 `.claude/kb/format-frozen/<组件>.toml`，人只决定发不发起，机器决定这次请求成不成立 **状态：已定。** **进度：未实现。** |
| 6 | **冻结前必须完成的清单** | 七项前置，逐次冻结重判；每一项的取值从对应分项的定案行现取，不许手抄 **状态：已定。** **进度：无对象。** |
| 7 | **冻结分几层，四层之外还有什么** | 四层有严格依赖顺序（系统配置与块头、指针字段布局、key 编码、索引节点内部布局），另有三个独立冻结组件 **状态：已定。** **进度：无对象。** |
| 8 | **冻结之后失去哪些自由** | 已冻结字段的宽度、存在性、含义不能再改，只能新增；对已冻结部分 checker 的判断也成了兼容性契约 **状态：已定。** **进度：无对象。** |
| 9 | **冻结后的改动判定表** | 九类改动各自判 compat / compat_ro / incompat 与 bump 与否，判据是旧读者会怎样 **状态：已定。** **进度：无对象。** |
| 10 | **feature bit 的位数、记账与分配权** | 三个 bitmap 各 256 位存系统配置；分配权是「谁先合并谁先占」，位号严格递增；位一旦用过不许回收 **状态：已定。** **进度：已实现。** |
| 11 | **冻结要配的两条会红的检查** | 格式冻结契约（`spec_hash` 比对）与向后兼容回归（黄金镜像），镜像本体不进 git、清单只增不删 **状态：已定。** **进度：未实现。** |

```
