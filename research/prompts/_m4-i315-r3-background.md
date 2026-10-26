# m4-i315-r3：「没有位置项可比」那一格两个收严的自证与归类器按（盘, 槽）比（推论轮第三轮，最终轮）正文（2026-10-21）

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 -->

## 一、题面

第二轮判决 `research/prompts/m4-i315-r2-main-verification.md`，用户在「用户定案」一节定了三件事：
- 第 1 行开这一轮，专攻两个收严的自证与两者叠用，当隔离豁免不再进候选；
- 第 2 行把甲二按（盘, 槽）比并进这一轮；
- 第 3 行「盘上完好的隔离」那一形先建丙（隔离落盘）的计数模型，不在这一轮判。

岔路单是 `research/prompts/m4-i315-r2-forks.md`。这是第三轮，也是最终轮：这一轮里新冒出来的零轮形态不再为它开一轮，写进判决的交用户表、标「零轮」。

被判那一格：一个已分配、没有根引用的单元，checker 要判它是不是写者隔离的。走过的版本与根环里别的根都没有一条位置项指着它的某一份时，隔离判据今天退回自证（「None => check_unit(&unit).is_ok(),」，`crates/singlefs-checker/src/walk.rs` 第 8461 行与第 16239 行）。第一轮攻方打中、第二轮两腿复核过的那一形 K2-1：种子 7463871032432356395、注入点 0，被吞的写落在复用槽上，旧住户的单元自证照过，于是 I-3.1、I-3.11、I-3.15 一起红。

这一轮被判的候选，都是第二轮腿提的、被攻过零轮：
- **自证加代号核对**（第二轮辩方提的）：这一份自证过、头可用，而且头里诞生代号等于罩住它的未释放记录的分配代，才算对得上。借的是 I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号）那个等式。辩方模型 `research/prompts/m4-i315-r2-sonnet-model/`，臂名 `gen`。
- **丁 两份互比**（第二轮攻方提的）：这一份自证过了，还要与另一块盘同槽那一份相同才算对得上；另一份读不出或不同算对不上。攻方副本补丁 `research/prompts/m4-i315-r2-opus-model/tools/apply_compare_arm.py` 是逐字节比，入库写法要与 GPU 一致的话改比整单元 CRC，那一版没跑过。
- **两者叠用**：两样都过才算对得上。
- **甲二按（盘, 槽）比**（第二轮攻方提的收严）：归类器在插回去那张盘上，把实现交出、与被吞的写有重叠的单元当隔离单元再判一遍；只有每条违例点名的（盘, 槽）在插回去那张盘上都不红，才归说谎的设备。实现的隔离清单要跨重挂累积，并收回退那一条路交回的字段。第二轮攻方量到，按不变量名比的甲二把一个无关泄漏归成了说谎的设备。

这一轮要的是：上面四样各在合法盘、说谎设备的盘与坏盘上判什么；第二轮判决与两腿报的数在入库装置上核不核得出；候选清单有没有漏；各要改 `crates/` 哪几处、代价多少。要交用户的岔路写成岔路单的形态。

被测条款（材料附录整节抄）：
- 不变量 I-3.1（已分配统计对得上）、I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号）、I-3.11（已分配减 defer 等于最新根走读）、I-3.15（未释放记录罩住的单元由映射指在这一槽）四行（`crates/singlefs-checker/invariants/05-空间记账.md`）；
- D19（块指针的结构与宽度预算） 已定项 5；
- D3（空间分配） 已定项 3 与已定项 7（分配代、释放代）；
- D13（验证路线） 里故障注入、说谎设备那几句；
- 第一、二轮判决与岔路单。

## 二、实现今天的样子（主 agent 的观测，2026-10-21，行号按主工作区此刻的 `crates/`）

- 隔离判据那一格两处同写法：
  - `crates/singlefs-checker/src/walk.rs` 的 `quarantined_slots_exempted_per_device`（第 8395 行起，I-3.1 / I-3.11 那一份，第 8461 行 `None =>` 臂）；
  - `some_copy_of_the_unit_fails_its_location_entries`（第 16223 行起，I-3.15 那一份，第 16239 行）。
  - 位置项校验和取自走过的各版：`LocationEntryChecksumsForQuarantine`（第 8192 行起）。
- 头里诞生代号的读法 `birth_txg_in_a_usable_unit_header`（`walk.rs` 第 7362 行）。分配记录的 `generation` 字段（第 5288 行，「仍分配时是分配代；已释放时是释放代」）。
- GPU 扫描内核 `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl` 的 `c_copy_matches_its_location_entry`（第 1908 行起），一个函数供 I-3.1 / I-3.11 那一组与 I-3.15 那一组。
- 故障归类器：
  - `crates/singlefs-harness/src/fault_injection.rs` 的 `left_by_the_lying_device`（第 2566 行起），第 2576 行「let every_violation_is_explained = observation.violations.iter().all(|(invariant, _)| {」按不变量名比；
  - 池级 checker 每条不变量只留第一处违例说明（`crates/singlefs-checker/src/image.rs` 第 494 行一带）。
- 实现的隔离清单：
  - `TransactionOutput::quarantined_after_release_checksum_mismatch` 在重挂之后为空（`crates/singlefs-core/src/recovery.rs` 第 4130 行「quarantined_after_release_checksum_mismatch: Vec::new(),」）；
  - 回退那条路交回另一个字段 `quarantined_user_visible_copies`（`crates/singlefs-core/src/mount.rs` 第 8645 行）。`mount.rs` 别的会话在改，不进这一轮的开工快照；引它写 HEAD 的行号，写明是 HEAD。
- 第二轮两腿的模型可以拿来当底，它们的底是乙二之后的主工作区，打之前先核：
  - 辩方 `research/prompts/m4-i315-r2-sonnet-model/`（固定盘面 `defense_worlds.rs`、随机历史探针 `defense_probe.rs`、副本补丁 `apply_arms*.py`）；
  - 攻方 `research/prompts/m4-i315-r2-opus-model/`（`attack_probe.rs`、缺陷开关 `ATTACK_DEFECT`、`apply_*.py`）。
- 方案按 `crates/` 今天的实现（上面这几处）来谈。

## 三、格

### K1　候选的定义与清单完整性
四样候选各写清判什么、要的输入从哪来。候选清单有没有漏：「没有位置项可比」时，还有没有盘上已有、不动格式的比对物，第二轮两腿都没提。

### K2　合法盘与说谎设备的盘上会不会误红
每个候选在每一形上判什么，各形如下：
- K2-1 那一形（复用槽、位置项转出根环）；
- 第二轮辩方固定盘面五个世界（`defense_worlds.rs`，覆盖写 24 次与 30 次）；
- 两块盘同时丢写、两份都是同一个旧住户（丁的推的盲区）；
- 旧住户诞生代号恰等于记录分配代（自证加代号核对的推的盲区：辩方读码推为造不出，要造一次或证明造不出）；
- 隔离之后管理员回退复活；
- 整理搬迁与隔离单元相邻；
- 快照保住的旧版；
- 重挂之后。
放大档（60 步 × 300 段、种子偏移 1000，第一轮攻方的口径）里各候选剩几点新发现。

### K3　坏盘上会不会放过
每个候选在坏盘上判什么：
- 泄漏、两份完好、位置项已转出根环（第二轮当隔离豁免放过的那一类）；
- 泄漏之后槽被别的住户占了；
- 泄漏加一份头坏；
- 实现隔离路径上的缺陷：只留一块盘的记录、多留一个无关单元、与隔离同一次发布的无关泄漏（攻方 `extra_leak`、`leak_once`）。
甲二按（盘, 槽）比会不会把哪种实现缺陷归成说谎的设备。

### K4　条款文意与第二轮的数能不能核
两个收严的自证与 I-3.1、I-3.11 定义行「每一份都读得出且对得上，照旧判违例（用户 2026-09-25 定「核出坏的才豁免」」那一句是否相容；补进定义行的那一句该怎么写（推的措辞）。第二轮判决第一、二节引的数，挑判决依赖的那几格在入库的 harness 档上复做，能复做的复做、对上没有：快档 24 段、K2-1 种子、`leak_once` 放大档、固定盘面 `LeakWithBothCopiesIntact`。

### K5　改法与代价
按候选列：`walk.rs` 两处、GPU 内核对应几处（对拍逐位比）、不变量定义行、故障归类器（`fault_injection.rs`、checker 交回违例点名的（盘, 槽）的入口、清单跨重挂累积）、必绿与必红世界、变异行。每处给改法一句与估的行数，推的写明推的。

## 四、分工

已攻过的角度不重复：
- 第一轮两条腿的 K1–K5：乙一、乙二在快档与固定盘面上的判法；甲-按份；乙二依据「隔离之后没人指着」一般不成立。
- 第二轮两条腿的 K1–K5：当隔离豁免放过一整类；甲二按名比把无关泄漏归成说谎设备；「只留一块盘的记录」各候选都放过；「盘上完好的隔离」只有丙分得开。

| 腿 | 格 | 立场 |
|---|---|---|
| 云端辩方（Sonnet） | K1–K5 | 复核第二轮判决够不够得着：判决里淘汰当隔离豁免、保留两个收严的那几句，引的数在入库的 harness 档上核不核得出（K4 后一半，能复做的复做）；候选清单完整不完整；给自证加代号核对、丁、两者叠用、甲二按（盘, 槽）比各造最强版本、替它辩护；要交用户的岔路写成候选、翻面观测、够判条件三列，不带倾向 |
| 云端攻方（Opus） | K1–K5 | 找反例：哪一形合法历史让自证加代号核对、丁、两者叠用各自误红（尤其两个推的盲区，造一次，造不出就写明为什么），哪一形坏盘让它们放过而退回自证不放过，甲二按（盘, 槽）比会把哪种实现缺陷放成说谎设备；能在主工作区的拷贝上写小模型、造镜像、跑 harness 档量的就量（拷到自己的草稿目录改，不在主工作区里改） |

### 小节清单：`.claude/kb/experiments/112-legacy-writer-encountering-unknown-trees.en.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## E112 Legacy Writer Encountering Unknown Trees — Run (2026-09-06, pure arithmetic, 11 unit tests / 9 mutants caught): **Among the four mountable cells, `Skip + Carry` has zero silent outcomes, with cost of 2 extra rows written per generation; ⚠️ `Refuse` two cells also have zero silent outcomes, eliminated from balance by `mount_ok` filter — its cost did not enter the silence criterion** | 抄 | 顶上标题，与开篇正文合成行区间 |
| （E112 标题之下、第一个下级标题之前的正文：第 2-9 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### Cited Items (Copied Verbatim into Source Comments) | 抄 | 默认全抄（脚本生成） |
| ### Model (Locked Before Run) | 抄 | 默认全抄（脚本生成） |
| ### Criteria (Locked Before Run, Unchanged After Run) | 抄 | 默认全抄（脚本生成） |
| ### Failure Clauses (Locked Before Run) | 抄 | 默认全抄（脚本生成） |
| ### Critical Model Premise (Checked on 2026-09-06) | 抄 | 默认全抄（脚本生成） |
| ### Measurements (2026-09-06, pure arithmetic, 11 unit tests / 9 mutants caught) | 抄 | 默认全抄（脚本生成） |
| ### Conclusion (Cross-Checked Against Acceptance Clause Locked Before Run) | 抄 | 默认全抄（脚本生成） |
| ### Measurement Basis | 抄 | 默认全抄（脚本生成） |
| ### Decisions Affected | 抄 | 默认全抄（脚本生成） |
| ## Historical Versions | 抄 | 顶上标题，与开篇正文合成行区间 |
| ### 2026-09-06 | 抄 | 默认全抄（脚本生成） |
| #### (Part 1) | 抄 | 随父节带到 |
| #### (Part 2) | 抄 | 随父节带到 |

### 小节清单：`.claude/kb/experiments/112-旧ライターが未知のツリーに遭遇した場合.ja.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## E112 旧ライターが未知のツリーに遭遇した場合 —— 実行済み (2026-09-06、純粋算術、11単体テスト / 9変異全検出)：**マウント可能な 4 つのセルにおいて `Skip + Carry` はサイレントな結末がゼロ、代償は世代ごとに 2 行の追加書き込み；⚠️ `Refuse` の 2 セルも同様にサイレントゼロであるが、`mount_ok` フィルタによって天秤から除外されたものである —— その代償はサイレントという判定基準には入っていない** | 抄 | 顶上标题，与开篇正文合成行区间 |
| （E112 标题之下、第一个下级标题之前的正文：第 2-9 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 引用された条項 (ソースコードのコメントに逐語転記) | 抄 | 默认全抄（脚本生成） |
| ### モデル (実行前に確定) | 抄 | 默认全抄（脚本生成） |
| ### 判定基準 (実行前に確定、実行後の変更不可) | 抄 | 默认全抄（脚本生成） |
| ### 失敗条項 (実行前に確定) | 抄 | 默认全抄（脚本生成） |
| ### 重要なモデル前提 (2026-09-06 現状確認) | 抄 | 默认全抄（脚本生成） |
| ### 実測 (2026-09-06、純粋算術、11単体テスト / 9変異全検出) | 抄 | 默认全抄（脚本生成） |
| ### 結論 (実行前に確定した受容条項との照合) | 抄 | 默认全抄（脚本生成） |
| ### 計測条件 | 抄 | 默认全抄（脚本生成） |
| ### 影響を受ける意思決定 | 抄 | 默认全抄（脚本生成） |
| ## 履歴バージョン | 抄 | 顶上标题，与开篇正文合成行区间 |
| ### 2026-09-06 | 抄 | 默认全抄（脚本生成） |
| #### (その一) | 抄 | 随父节带到 |
| #### (その二) | 抄 | 随父节带到 |

### 小节清单：`crates/singlefs-checker/invariants/05-空间记账.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## I-3 空间记账 —— 类 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （I-3 标题之下、第一个下级标题之前的正文：第 2-39 行） | 抄 | 开篇正文，与顶上标题合成行区间 |

### 小节清单：`.claude/kb/decisions/19-block-pointer-structure-width-budget.en.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D19 (Block Pointer Structure & Width Budget) — Settled | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D19 标题之下、第一个下级标题之前的正文：第 2-13 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### Settled Items | 抄 | 默认全抄（脚本生成） |
| #### Settled Item 1: Position Entries Carry Device Identity | 抄 | 随父节带到 |
| #### Settled Item 2: Ciphertext Checksum Bit-Width Takes 32 Bits | 抄 | 随父节带到 |
| #### Settled Item 3: Fixed-Width, Slots Reserved When Unencrypted | 抄 | 随父节带到 |
| #### Settled Item 4: Location Entry Width Takes 14 Bytes | 抄 | 随父节带到 |
| #### Settled Item 5: Location Authority Is Hybrid, Central Mapping Is Unique Entrance | 抄 | 随父节带到 |
| #### Settled Item 6: Central Mapping Key Reuses Write Sequence Key by Class (Post-Tightening) | 抄 | 随父节带到 |
| #### Settled Item 7: Pointer Header Carries Birth (Tree, TXG), 8 Bytes Each | 抄 | 随父节带到 |
| #### Settled Item 8: All Pointers Enter Mapping, Reference Side Carries Mapping Key Segments by Class | 抄 | 随父节带到 |
| #### Settled Item 9: How Birth Sequence Numbers Are Allocated and in What Order | 抄 | 随父节带到 |
| #### Settled Item 10: Mapping Key Strictly 27 Bytes, Entries Strictly 55 Bytes | 抄 | 随父节带到 |
| #### Settled Item 11: Pointer Offset Table, Position Entry Ordering, Where Mapping Tree Root Resides | 抄 | 随父节带到 |
| #### Settled Item 12: Code 2 / Code 3 Mapping Entries Written Day-1, Fixed-Point Writes Once per Checkpoint | 抄 | 随父节带到 |
| #### Settled Item 13: Block Pointer Split into Two Sections | 抄 | 随父节带到 |
| #### Settled Item 14: Position Entries Listed Individually, Cannot Be Computed from Single Origin via Geometric Formula | 抄 | 随父节带到 |
| #### Settled Item 15: Value Range of Extent Offset from Origin and Version 1 Value | 抄 | 随父节带到 |
| ### Chain Reactions with Other Decisions | 抄 | 默认全抄（脚本生成） |
| ## Historical Versions | 抄 | 顶上标题，与开篇正文合成行区间 |

### 小节清单：`.claude/kb/decisions/19-ブロックポインタ構造と幅予算.ja.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D19 (ブロックポインタ構造と幅予算) — 確定 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D19 标题之下、第一个下级标题之前的正文：第 2-13 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 確定項目 | 抄 | 默认全抄（脚本生成） |
| #### 確定項目 1：位置エントリにデバイス識別子を含める | 抄 | 随父节带到 |
| #### 確定項目 2：暗号文チェックサムは 32 ビットを採用 | 抄 | 随父节带到 |
| #### 確定項目 3：固定幅、非暗号化時もスロットを確保 | 抄 | 随父节带到 |
| #### 確定項目 4：位置エントリは 14 バイト | 抄 | 随父节带到 |
| #### 確定項目 5：位置の権威はハイブリッド、中央マッピングが唯一の入口 | 抄 | 随父节带到 |
| #### 確定項目 6：中央マッピング key は種別ごと書き込みシーケンスキー再利用（厳格化後）を採用 | 抄 | 随父节带到 |
| #### 確定項目 7：ポインタヘッダは誕生 (ツリー, txg) を保持、各 8 バイト | 抄 | 随父节带到 |
| #### 確定項目 8：全ポインタがマッピング経由、参照側は種別ごとにマッピング key セグメントを担持 | 抄 | 随父节带到 |
| #### 確定項目 9：誕生シーケンス番号の発行方法と順序 | 抄 | 随父节带到 |
| #### 確定項目 10：マッピング key は一律 27、エントリは一律 55 | 抄 | 随父节带到 |
| #### 確定項目 11：ポインタのオフセット表、位置エントリの順序、マッピングツリー根の配置場所 | 抄 | 随父节带到 |
| #### 確定項目 12：Code 2 / Code 3 のマッピングエントリは day-1 で書き出し、固定点はチェックポイントあたり1回書き出し | 抄 | 随父节带到 |
| #### 確定項目 13：ブロックポインタは2セグメントに分割 | 抄 | 随父节带到 |
| #### 確定項目 14：位置エントリは個別に列挙、単一起点から幾何学的計算で導出しない | 抄 | 随父节带到 |
| #### 確定項目 15：起点からの extent オフセットの値域と第1版の値 | 抄 | 随父节带到 |
| ### 他の決定との連鎖 | 抄 | 默认全抄（脚本生成） |
| ## 過去の版 | 抄 | 顶上标题，与开篇正文合成行区间 |

### 小节清单：`.claude/kb/decisions/19-块指针的结构与宽度预算.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D19 块指针的结构与宽度预算 —— 已定 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D19 标题之下、第一个下级标题之前的正文：第 2-13 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 已定项 | 不抄 | 分项索引表，用 --extra 按行区间取 |
| #### 已定项 1：位置条目带设备身份 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 2：密文校验和取 32 位 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 3：定宽，不加密时留位 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 4：位置条目 14 字节 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 5：位置条目降为提示，中央映射是唯一入口 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 6：中央映射 key 取按类复用写序键（收严后） | 抄 | 默认全抄（脚本生成） |
| #### 已定项 7：指针头部带出生 (树, txg)，各 8 字节 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 8：全部指针进映射，引用侧按类携带映射 key 的各段 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 9：出生序号怎么发，按什么次序发 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 10：映射 key 一律 27，条目一律 55 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 11：指针的偏移表、位置条目的顺序、映射树的根住哪 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 12：码 2 / 码 3 的映射条目 day-1 写出，固定点一个 checkpoint 只写一次 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 13：块指针分两段 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 14：位置条目逐个列出，不用几何公式从单一起点推算 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 15：extent 距起点偏移的值域与第一版取值 | 抄 | 默认全抄（脚本生成） |
| ### 与其他决策的连锁 | 抄 | 默认全抄（脚本生成） |
| ## 历史版本 | 不抄 | 历史节，正文只引现状 |

### 小节清单：`.claude/kb/decisions/03-丙.en.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D3: C — settled | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D3: 标题之下、第一个下级标题之前的正文：第 2-4 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ## History | 抄 | 顶上标题，与开篇正文合成行区间 |

### 小节清单：`.claude/kb/decisions/03-space-allocation.en.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D3 Space Allocation — Settled | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D3 标题之下、第一个下级标题之前的正文：第 2-6 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### Settled Items | 抄 | 默认全抄（脚本生成） |
| #### Settled Item 1: Allocation Records Answer "Belongs to Active Version" Per-Placement | 抄 | 随父节带到 |
| #### Settled Item 2: Tombstones and livelist FREE Entries Counted into Worst Case by Acceptance Control | 抄 | 随父节带到 |
| #### Settled Item 3: Allocation Records Reside in an Independent Keyspace B-Tree | 抄 | 随父节带到 |
| #### Settled Item 4: Authoritative Form of Acceptance Inequality | 抄 | 随父节带到 |
| #### Settled Item 5: Commit-Endogenous Blocks Allocated from Clustered Segments | 抄 | 随父节带到 |
| #### Settled Item 6: In-Flight Synthesis of Acceptance | 抄 | 随父节带到 |
| #### Settled Item 7: On-Disk Encoding of Allocation Record Entries and Registry of Allocation Acceptance Conjunctions | 抄 | 随父节带到 |
| #### Settled Item 8: User Data Blocks Do Not Enter Clustered Segments; Placement Is Smallest Starting Free Slot in Selected Device | 抄 | 随父节带到 |
| #### Settled Item 9: False ENOSPC Prohibited; Deleted Space Usable Within Bounded Steps | 抄 | 随父节带到 |
| #### Settled Item 10: Definition of Clustered Segments, Four Placement Cells for User Data, Bump Order of Commit-Endogenous Blocks | 抄 | 随父节带到 |
| #### Settled Item 11: Span Segment in 20-Byte Allocation Record Placed in Value | 抄 | 随父节带到 |
| #### Settled Item 12: Unique Registry for Allocation Acceptance Conjunctions and Relationship Between the Two Acceptance Formulas | 抄 | 随父节带到 |
| #### Settled Item 13: No Split into Data and Metadata Space Categories | 抄 | 随父节带到 |
| #### Settled Item 14: Collateral Commitment of Non-Classification — Background Compaction Resident, Automatic, and Transparent to Users | 抄 | 随父节带到 |
| #### Settled Item 15: Space Reservation Takes Acceptance Control Form | 抄 | 随父节带到 |
| #### Settled Item 16: Keyless Side Reads and Probes Only When Encryption Is Enabled | 抄 | 随父节带到 |
| #### Settled Item 17: Space Release Operation Itself Does Not Require Space Allocation | 抄 | 随父节带到 |
| ## Revision History | 抄 | 顶上标题，与开篇正文合成行区间 |

### 小节清单：`.claude/kb/decisions/03-領域割り当て.ja.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D3 領域割り当て —— 確定 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D3 标题之下、第一个下级标题之前的正文：第 2-6 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 確定項目 | 抄 | 默认全抄（脚本生成） |
| #### 確定項目 1：割り当てレコードが配置先ごとに「現行バージョンに属するか」を回答可能 | 抄 | 随父节带到 |
| #### 確定項目 2：墓石および livelist の FREE エントリを受入制御により最悪ケースに算入 | 抄 | 随父节带到 |
| #### 確定項目 3：割り当てレコードは独立 keyspace の B-Tree に常駐 | 抄 | 随父节带到 |
| #### 確定項目 4：受入不等式の権威形式 | 抄 | 随父节带到 |
| #### 確定項目 5：コミット内生ブロックはクラスタセグメントから割り当て | 抄 | 随父节带到 |
| #### 確定項目 6：受入のインフライト合成 | 抄 | 随父节带到 |
| #### 確定項目 7：割り当てレコードエントリのディスク上エンコーディングおよび割り当て受入連言の登録表 | 抄 | 随父节带到 |
| #### 確定項目 8：ユーザーデータブロックはクラスタセグメントに入らず、配置先は選択デバイス内で開始スロット番号最小の空きスロット | 抄 | 随父节带到 |
| #### 確定項目 9：偽の ENOSPC を禁止、削除された領域は有界ステップ数内に利用可能 | 抄 | 随父节带到 |
| #### 確定項目 10：クラスタセグメントの定義、ユーザーデータ配置先の 4 格、コミット内生ブロックの bump 順序 | 抄 | 随父节带到 |
| #### 確定項目 11：割り当てレコード 20 バイト中のスパンセグメントは value に格納 | 抄 | 随父节带到 |
| #### 確定項目 12：割り当て受入連言の唯一の登録場所、および 2 本の受入式の関係 | 抄 | 随父节带到 |
| #### 確定項目 13：data / metadata の 2 つの領域種別に分類しない | 抄 | 随父节带到 |
| #### 確定項目 14：非分類に伴う付帯コミットメント——バックグラウンド整理は常駐・自動・ユーザー透過 | 抄 | 随父节带到 |
| #### 確定項目 15：領域予約は受入制御の形態を採用 | 抄 | 随父节带到 |
| #### 確定項目 16：暗号化有効時に鍵のない側は読み取りと検出のみ実施 | 抄 | 随父节带到 |
| #### 確定項目 17：領域解放操作自体は領域の申請を不要とする | 抄 | 随父节带到 |
| ## 過去バージョン | 抄 | 顶上标题，与开篇正文合成行区间 |

### 小节清单：`.claude/kb/decisions/03-空间分配.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D3 空间分配 —— 已定 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D3 标题之下、第一个下级标题之前的正文：第 2-6 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 已定项 | 不抄 | 分项索引表，用 --extra 按行区间取 |
| #### 已定项 1：分配记录逐落点答得出「属不属于现行版本」 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 2：墓碑与 livelist 的 FREE 条目由准入控制算进最坏情况 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 3：分配记录住一棵独立 keyspace 的 btree | 抄 | 默认全抄（脚本生成） |
| #### 已定项 4：准入不等式的权威形式 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 5：提交内生块从聚簇段分配 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 6：准入的在飞合成 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 7：分配记录条目的盘上编码，与分配准入合取的登记表 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 8：用户数据块不进聚簇段，落点是已选设备内起点最小的空槽 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 9：不许有假性 ENOSPC，删掉的空间在有界步数内可用 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 10：聚簇段的定义、用户数据落点的四格、提交内生块的 bump 次序 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 11：分配记录 20 字节里跨度段进 value | 抄 | 默认全抄（脚本生成） |
| #### 已定项 12：分配准入合取的唯一登记位，与两条准入式的关系 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 13：不分 data / metadata 两类空间 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 14：不分类的连带承诺——后台整理常驻、自动、用户无需知道 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 15：空间预留取准入控制形态 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 16：加密开启时无密钥的一侧只读只检测 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 17：释放空间这个操作本身不需要申请空间 | 抄 | 默认全抄（脚本生成） |
| ## 历史版本 | 不抄 | 历史节，正文只引现状 |

### 小节清单：`.claude/kb/decisions/13-verification-route.en.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D13 Verification Route —— Settled | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D13 标题之下、第一个下级标题之前的正文：第 2-6 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### Settled Items | 抄 | 默认全抄（脚本生成） |
| #### Settled Item 1: Crash Refinement First Verifies "One Publish + One Crash + One Recovery" | 抄 | 随父节带到 |
| #### Settled Item 2: Serial Commit First | 抄 | 随父节带到 |
| #### Settled Item 3: RefFS Taken to the Extreme, Handling Persistence and Crash Semantics Itself | 抄 | 随父节带到 |
| #### Settled Item 4: Crash-Point Replay Enumerates Full Write Subsets Only | 抄 | 随父节带到 |
| #### Settled Item 5: Sharing Only a Single Constants Module Generated from kb | 抄 | 随父节带到 |
| #### Settled Item 6: Three Mutually Independent Oracles | 抄 | 随父节带到 |
| #### Settled Item 7: O2 (Independent Parser + Checker) Judges a Single Image | 抄 | 随父节带到 |
| #### Settled Item 8: Two Custom-Developed Mechanisms: Merkle Root Chain Diff and Generational Incremental Scrub | 抄 | 随父节带到 |
| #### Settled Item 9: Crash-Point Replay Must Be Tiered to Avoid Wall-Clock Explosion | 抄 | 随父节带到 |
| #### Settled Item 10: Truly Required Exact Coverage N Is "Number of Commit Protocol Equivalence Classes", Not "Layout Schemes" | 抄 | 随父节带到 |
| #### Settled Item 11: When Ignored Nodes Are Enabled: Rejecting Ledger Rotating by Owed Checks, Retaining Three Rules | 抄 | 随父节带到 |
| #### Settled Item 12: Conflict 1: `Checker as Specification` vs `Verification Paths Sharing No Tools` | 抄 | 随父节带到 |
| #### Settled Item 13: Conflict 2: Refinement Spec vs Differential Model | 抄 | 随父节带到 |
| #### Settled Item 14: Interlocks with Other Decisions | 抄 | 随父节带到 |
| #### Settled Item 15: Verification Code in Two Tiers —— Harness Tier Runs Anytime, Checker Tier Defaults to Commit | 抄 | 随父节带到 |
| #### Settled Item 16: Segments Exceeding u64 Judged in Slices, State Count Uses 1280-Bit Fixed-Width Integer | 抄 | 随父节带到 |
| #### Settled Item 17: Unit of Verdict Reuse Is Function, Key Composed Solely of Function Units, Toolchain, Dependency Versions | 抄 | 随父节带到 |
| ## Historical Versions | 抄 | 顶上标题，与开篇正文合成行区间 |

### 小节清单：`.claude/kb/decisions/13-検証ルート.ja.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D13 検証ルート —— 確定 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D13 标题之下、第一个下级标题之前的正文：第 2-6 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 確定項目 | 抄 | 默认全抄（脚本生成） |
| #### 確定項目 1：crash refinement はまず「1回のパブリッシュ + 1回のクラッシュ + 1回のリカバリ」を検証する | 抄 | 随父节带到 |
| #### 確定項目 2：まず直列コミットとする | 抄 | 随父节带到 |
| #### 確定項目 3：RefFS を極限まで突き詰め、自前で永続化とクラッシュセマンティクスを処理する | 抄 | 随父节带到 |
| #### 確定項目 4：クラッシュポイント再生は完全書き込みの部分集合のみを列挙する | 抄 | 随父节带到 |
| #### 確定項目 5：kb から生成された定数モジュール1つのみを共有する | 抄 | 随父节带到 |
| #### 確定項目 6：互いに独立な3つの oracle | 抄 | 随父节带到 |
| #### 確定項目 7：O2 は何を判定するか：単一イメージとレコード照合器 | 抄 | 随父节带到 |
| #### 確定項目 8：独自開発の2点：Merkle ルートチェーン差分と世代間増分 scrub | 抄 | 随父节带到 |
| #### 確定項目 9：クラッシュポイント再生は3層に分割する。さもなければ実時間が爆発する | 抄 | 随父节带到 |
| #### 確定項目 10：真に正確に網羅すべき N は「レイアウトのセット数」ではなく「コミットプロトコル構造の同値類数」である | 抄 | 随父节带到 |
| #### 確定項目 11：ignore ノードをいつ開放するか：未済の検査によるローテーション台帳を否決し、3条を残す | 抄 | 随父节带到 |
| #### 確定項目 12：衝突 1：checker 即仕様 と 検証の2経路間でツール共有を禁止する の衝突 | 抄 | 随父节带到 |
| #### 確定項目 13：衝突 2：crash refinement の spec とモデル突き合わせの model | 抄 | 随父节带到 |
| #### 確定項目 14：他の決定との連鎖 | 抄 | 随父节带到 |
| #### 確定項目 15：検証コードを2層に分割 —— harness 層は常時実行、checker 層は既定でコミット時のみ実行 | 抄 | 随父节带到 |
| #### 確定項目 16：セグメント単位で登録されたセグメントの全領域が u64 に収まらなければスライス判定し、状態数には 1280 ビット固定幅整数を用いる | 抄 | 随父节带到 |
| #### 確定項目 17：判定再利用のユニットは関数であり、キーは関数ユニット、ツールチェーン、依存バージョンのみで構成する | 抄 | 随父节带到 |
| ## 履歴バージョン | 抄 | 顶上标题，与开篇正文合成行区间 |

### 小节清单：`.claude/kb/decisions/13-验证路线.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D13 验证路线 —— 已定 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D13 标题之下、第一个下级标题之前的正文：第 2-6 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 已定项 | 不抄 | 分项索引表，用 --extra 按行区间取 |
| #### 已定项 1：crash refinement 先验「一次发布 + 一次崩溃 + 一次恢复」 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 2：先串行提交 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 3：RefFS 做到极致，自己处理持久化与崩溃语义 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 4：崩溃点重放只枚举整写子集 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 5：只共享一份从 kb 生成的常量 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 6：三个互相独立的 oracle | 抄 | 默认全抄（脚本生成） |
| #### 已定项 7：O2（独立解析器 + checker） 判的是**一个**镜像 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 8：自研的两样：Merkle 根链差分与代际增量 scrub | 抄 | 默认全抄（脚本生成） |
| #### 已定项 9：崩溃点重放要分三层，否则挂钟必炸 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 10：真正必须精确覆盖的 N 不是「布局套数」，是「提交协议结构的等价类数」 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 11：ignore 的节点什么时候开：否决按欠账轮转的账本，留下三条 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 12：冲突 1：`checker 即规范` 与 `校验的两条路径不许共享工具` | 抄 | 默认全抄（脚本生成） |
| #### 已定项 13：冲突 2：crash refinement 的 spec 与模型对拍的 model | 抄 | 默认全抄（脚本生成） |
| #### 已定项 14：与其他决策的连锁 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 15：验证代码分两档——harness 档随时跑，checker 档默认只在提交时跑 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 16：按段登记的段整域装不进 u64 就分片判，态数用 1280 位定宽整数 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 17：判定复用的单元是函数，键只由函数单元、工具链、依赖版本组成 | 抄 | 默认全抄（脚本生成） |
| ## 历史版本 | 不抄 | 历史节，正文只引现状 |

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
| I-3.1 | 已分配统计对得上 | 已分配空间统计 == 实际遍历所有引用得到的和 | 已实现（池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；崩溃点重放每个崩溃状态都判） ⚠️ **引用按映射位置记**（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3 与第七节用户定案「重构」，D19（块指针的结构与宽度预算） 已定项 5 / 8）：豁免三类（映射树自己的节点、树表单元、实例表单元）之外的码 1 / 2 / 3，走读按那一版的中央映射认位置，指针里的位置条目只当提示——引用集合记映射位置，提示过期（单元被搬走、源槽已释放）不进引用；提示与映射条目在同一序号上带的整单元校验和不同的那条指针不当过期的提示，照它自己的位置条目记（`walk::CentralMappingLocationsOfAVersion::locate`，实现员 2026-10-15 推的读法，交主 agent 认）；合法搬迁过的镜像在 `crates/singlefs-harness/tests/checker_relocated_images_judged_by_the_mapping_position.rs`。⚠️ 「源记录还在册、目的槽没有记录」那一形（映射换位、分配记录一条不动）按槽数加总对得上，这一条看不见，由 G3 的新判 N2 判红：I-3.15（未释放记录罩住的单元由映射指在这一槽），2026-10-15 实现。 ⚠️ **checker 读法（2026-09-14 用户收尾弹窗定甲）**：「实际遍历所有引用」按根环里全部有效根的引用取并集——新池新建文件之后第 0 版树表单元只被更早的根引用、仍占着空间（已释放、在 defer 队列里），只按最新根算会少 1 槽。**「有效根」= 回退候选集里的根**：按最新根指着的实例表有效（无那个实例的行，或有行 (i, Ti, Wi) 且 T ≤ Ti）且 txg ≥ F_生效（D16（发布语义） 已定项 1「生效」）。遍历的是最新根与不被抛弃、不低于 F 的有效根（RGall：候选集不再滤一圈窗口，环上有洞时洞那一槽里留着的上一圈的根只要不被抛弃、不低于 F 也走；代码 `crates/singlefs-checker/src/walk.rs` 的 `check_pool_image_given_the_container_capacity_knobs` 里 `candidate_indexes` 那一段）。F_生效 由 checker 自己算：取各块盘上最新的有效根所带 F 的最大值，再与池里自证过、fsid 与本池相同的系统配置槽带的 F 取大（`walk::check_pool_image` 里的 `effective_rollback_floor`，C556（checker 与崩溃点重放不读系统配置里的 F） 已还清）——被抛弃时间线的根引用的单元由影子账隔离、F 之下的根引用的已释放单元已可再分配，都不在当前账里（里程碑「覆盖写、释放、回退与复用」步 4 / 步 5，`crates/singlefs-checker/src/walk.rs`）。⚠️ **最新根带的 F 低于 F_生效（两者不等）时这一格不判、报「不适用」（实二的读法，主 agent 2026-09-26 暂认，最终代码三方专攻这一格）**：先把新 F 写进系统配置之后、带新 F 的根落盘之前，或带新 F 的根全读不出，最新根的「已分配」是按它自己那个较低的 F 记的，还算着 F_生效 之下那些根引用的槽，与按 F_生效 取的候选集并集不可比，照判会在合法状态上红；另一种读法（并集按最新根自己的 F 取、别的几条按 F_生效 取）条款没选。坏镜像 `checker_known_bad_images.rs` 的 `every_root_carrying_the_raised_floor_unreadable_keeps_the_candidate_invariants_green_because_the_checker_reads_the_system_configuration_floor` 钉住这一格报不适用，`crates/mutations.tsv` 里把这道例外拿掉的那一行（实二）让它红。F_生效 之下仍在 defer 的槽：F_生效 之下、没被抛弃的根引用着，候选集那一遍没走到，而最新根的账里罩住它的记录是已释放、释放代高于回收门槛的（还在 defer 里，分配器还不许回收；回收门槛 = max(F_生效, 环里最旧有效根)，D16（发布语义） 已定项 1「可再分配」那一行），并进遍历得到的和（收口表第 43 行取丁-defer，用户 2026-09-28 定，被攻过一轮，判决 `research/prompts/abandoned-floor-r1-main-verification.md` M1；`walk::deferred_slots_referenced_only_below_the_floor_per_device`，用例 `raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_the_allocated_statistic.rs`）；⚠️ 它会不会盖住「该回收而没回收」那一类错，没拿变异试过。隔离的记录：已分配而没有任何根引用的记录，按单元豁免——它罩住的那个单元只要有任何一份副本 checker 自己读出来读不出或校验和对不上，这个单元在每块盘上的那几条记录都豁免；每一份都读得出且对得上，照旧判违例（用户 2026-09-25 定「核出坏的才豁免」，按单元的读法是主 agent 同日定的，D19（块指针的结构与宽度预算） 已定项 5 的隔离）。 中央映射条目指的物理范围算进遍历得到的引用，与树里指向同一单元的引用同 (设备, 起点槽, 跨度) 时算一个（2026-09-27，`m2-rev-b1-implementer-report.md` 第六节第 2 条）。由记录施加出来的那一版照 D23（journal 的角色与格式） 已定项 15，走那条记录新根段里的实例表指针与分配记录树根（`walk::walk_version_applied_only_by_records`；用户 2026-10-02 定新根段回到三字段，实例表指针与分配记录树根随新根段换成那次发布的）。⚠️ **没量的前提**：「隔一条也是同一张实例表」是按「一个实例写行之后每一版照抄同一张实例表」推的，没量（`m2-rev-b2-implementer-report.md` 第七节第 2 条）；量它是 C699（取号跳过改认出生代号） 的第一步。 ⚠️ **钉最旧 txg 的代价（用户认的）**：读坏槽与读回全零的槽把回收门槛钉在它可能住过的最旧 txg（D16（发布语义） 已定项 1「根槽这一次读坏」），这样的合法盘面上 I-3.1（已分配统计对得上） 红在记账多的一边，到钉解开为止（这个进程写掉那一槽，或进程内记根时重读读出有效根、读出被抛弃的根，`crates/singlefs-core/src/allocator.rs` 的 `RootRingOccupant` 文档注释）；`crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs` 三条慢档用例在第 10h 批改成只许红在记账多的一边（用户 2026-10-04 定钉最旧 txg、2026-10-05 定全零槽钉门槛而不并进上限，`records/2026-09-30-里程碑三开工第一轮.md` 第 10b、10c 批）。 ⚠️ **钉落盘之后的并法（C773（读坏与全零根槽的回收门槛钉只在内存里） 定案，用户 2026-10-20 认；checker 未实现，随 C773（读坏与全零根槽的回收门槛钉只在内存里） 实现批，GPU 判器那一侧没有现成件）**：根记录带回收门槛 t 与钉槽 p 之后（I-7.18（根记录的回收门槛等于现算的门槛）、I-7.19（钉着的槽不住本实例写的根）），这一格只把「钉带出来的那一截」不看引用并进遍历得到的和，被抛弃根引用着的槽不并（C684（双故障形下已分配统计该不该并被抛弃根的槽） 保持「不并」，那一形照旧红在记账多的一边）；最新根下面没有记账树时照旧报不适用（`walk.rs` 今天已是）。「钉带出来的那一截」两腿的算例都取释放代落在 (t, 不带钉现算的门槛] 的已释放槽（推的）；checker 要走被抛弃根才分得出「被抛弃根引用着」，底子是 `walk::judge_blocks_referenced_by_abandoned_roots`。出处：根环三方第二轮判决 `research/prompts/m3-close-rootring-r2-main-verification.md` 第二节 Y4 那一行，第三轮判决 `research/prompts/m3-close-rootring-r3-main-verification.md` 第二节 C773（读坏与全零根槽的回收门槛钉只在内存里） 那一行之后的格式件补漏段。 |
| I-3.2 | 已停用·方向反了 | **此编号不再使用**——它写成「≤ 引用它的**最新**快照的代号」，方向反了：紧的约束是最**旧**的那个。旧写法被 I-3.5（引用区间的精确性） 的子判据 ② 严格蕴含，且检不出任何一类真实错误 | 不适用 |
| I-3.3 | 已停用·归属未定义 | **此编号不再使用**——「所属快照」未定义，而两种归属约定给出不同的比较符。由 I-3.6（deadlist 紧界） 取代 | 不适用 |
| I-3.5 | 引用区间的精确性 | **引用区间的精确性**。**内容**：对任一块 b 与任一根 R（快照根或活头根），**「b 从 R 可达」这个可达性谓词，必须等于记账所用的那个谓词**——两者是同一个定义，不是两份。**当前形态下该谓词的表达式**是 `birth(b) ≤ R.txg < death(b)`（[decisions.md](../../../.claude/kb/decisions.md) D5（快照 / 空间记账机制） 形态）；**换快照模型时表达式要重写，内容不变**。可判定形式（与 I-3.1（已分配统计对得上） 共用同一趟全盘遍历）：按 txg 升序枚举全部快照根与活头得 `Reach(b)`，则 ①② 合成一条**出现规则**：取回退候选集（I-7.4（近 K 代块未被复用） 用的那一份，加由记录施加出来、根槽从没落盘的那几版）按 txg 升序，同 txg 的几条并成一条（R⁻ 侧引用集合取并，R 侧逐条各判一次），相邻两条 (R⁻, R) 为一对，逐对判。R 引用而 R⁻ 不引用的每个落点 b（第一次出现与再次出现都判，`Reach(b)` 在 txg 序上是否连续不再单判）满足 (a) `birth(b) > R⁻.txg`；或 (b) b 落在 R 树表里某棵用户可见树（今天 extent 树、inode 树，`deleted_inodes` 树根非全零之后加它）的子树里，那棵树的根指针非全零、出生 txg 严格小于 R⁻.txg，且那棵子树里每个读得出的单元头的诞生代号都不大于那个出生 txg（读不出头的单元不计入）。`birth(b)` 取 b 的单元头里的诞生代号；b 自己的头读不出时这一个落点不判，读不出归 I-1.1（块头自述逻辑地址） 与 I-2.1（校验和与内容匹配）。**缺口**：R.txg ≠ R⁻.txg + 1 时这一对之间有洞，分两种处置——最新根的实例表里有一行 (R⁻ 的实例, T = R⁻.txg)、R 的实例比 R⁻ 的大、两者之间每个实例的行都是 (j, 0, 0)，即实例表行证明 R 所在的实例直接接在 R⁻ 那一版上的，这一对照常判（gap-row）；不是的，用户可见树子树里的落点不判，别的落点（树表、记账树、分配记录树、中央映射树的单元与实例表单元）仍照 (a) 判 `birth(b) > R⁻.txg`。R 或 R⁻ 的引用集合数不全（走读读不出，或走到没有条款的形状）时整对报不适用；没有一对判到任何落点时整条报不适用并带理由（同 I-4.9（非最新候选根引用的单元不晚于它） 候选集只剩一条根的写法）；③ 若 b 不被活头引用，则 b **恰好**出现在一个 deadlist 中，且该 deadlist 的归属区间 = `(Reach(b) 中最晚者.txg, 其下一个根.txg]`；④ 若 b 被活头引用，则 b 不出现在任何 deadlist 中；③ 的逆向（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S5 ①）：deadlist 中每一条 b 都不被活头引用，且 `Reach(b)` 非空（某个活快照引用它）或 b 在未完成销毁意图的待放集里（意图 (头, lo = S.txg, hi, P = prev(S).txg) 下纪元落在 [lo, hi) 里、birth > P 的条目，D5（快照 / 空间记账机制） 已定项 12）。③ ④ 与逆向里的「快照」只算活快照：带销毁中记号（码 14 条目 flags 位 0，同一判决 S4）的快照从写意图那一刻起不再钉住块（D5（快照 / 空间记账机制） 已定项 20 的例外句），不进 `Reach`；「归属区间」按纪元认：快照 X 那一份 = 纪元落在 [prev(X).txg, X.txg)、活头那一份 = [prev(头).txg, ∞)，b 该在的那一份是引用它的最晚那个活快照之后的那一侧（纪元落在 [那个快照的 S.txg, 链上下一个活快照的 S.txg 或 ∞)）。b 按单元认、写成中央映射 key（活快照与活头两棵用户可见树下面每个单元的 key 从指针现算，deadlist 条目里就是 key）。⚠️ **射程**：缺口的来源是新实例第一条根的 txg 取根环与 journal 里见过的最大 txg 加一（`crates/singlefs-core/src/mount.rs` 取号那一步），所以写行那次发布的记录已落而根槽没落、多记录发布只落一部分之后再可写挂载，必出现缺口；另有恢复丢掉尾巴记录、根槽写失败重发跳一格（今天的代码走不到，随失败处置在后面的里程碑做）、根槽读不出、根槽被环盖掉而记录也被盖掉（短 journal 环）。**盲区**：缺口右边那一对里，落在用户可见树子树里的新写单元（新叶）的诞生代号被改小到 R⁻.txg 以下，判不出；整条用户可见链（树根指针出生 txg 与树里单元的诞生代号）一起改小到 R⁻.txg 以下的坏镜像同样判不出：盘上与一次合法回退逐项同形，判别子是回退那次发布写的谱系记录（D18（块里携带什么信息） 已定项 24），`crates/` 今天没有。**谱系记录落地之后**，(b) 收成「R 的谱系树里有一条记录 (T_rb, T_old)，R⁻.txg < T_rb ≤ R.txg，且那棵用户可见树的根指针出生 txg ≤ T_old；谱系树读不全时该对不适用」，这一形与缺口条款并存（`research/prompts/m3-birth-floor-r2-main-verification.md` 第二节第 9 行）。**判别力**：诞生代号改小到恰好等于 R⁻.txg 的新写单元必须红；最新根的 inode 树指回已丢的旧叶、分配记录与映射条目一起改平必须红；回退根槽被环盖掉、记录也盖掉之后的合法镜像不许红。**推翻条件**：任一合法镜像（池级 checker 与崩溃点重放其余判定都不红）上红。 | 部分实现：出现规则（TF、β、gap、gap-row、n3）已实现（落点折成单元先按中央映射 key 折：搬迁前后两个落点是同一个单元，不算后一版新出现的，判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3；`walk::Walk::note_references_of_a_pointer_followed_by_the_walk`），池级 checker `crates/singlefs-checker/src/walk.rs` 的 `judge_the_appearance_rule_of_adjacent_candidate_versions`（`IMPLEMENTED_INVARIANTS` 下标 65；坏镜像在 `crates/singlefs-harness/tests/checker_judges_reference_intervals_and_every_new_root_segment_field.rs`；变异在 `crates/mutations.tsv` 里「第8批 I-3.5（引用区间的精确性）」那几行）；③ ④（deadlist 那两条）连同逆向已实现（2026-10-15，里程碑四只读快照 checker 侧，判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S5 ①）：并在下标 65 这一条下，池级 checker 的 `walk::judge_snapshots_and_deadlists_of_the_versions` 在回退候选集每一版（加由记录施加出来的那几版，树表单元相同的只判一次）上判：④ 每一条钉的单元活头不引用，逆向每一条钉的单元有活快照引用或在待放集里，③ 活快照引用而活头不引用的每个单元恰好一条、头是活头、纪元落在它该在的那一侧；一版里一个快照都没有、deadlist 一条都没有时这三格没有对象、不判；树表、活头两棵树、deadlist 树或活快照的两棵树读不全时报射程里判不了。纯判法的单测与必红世界在 `crates/singlefs-checker/src/walk.rs` 的测试模块（④、逆向、③ 缺一条与挂错一侧）与 `crates/singlefs-harness/tests/checker_snapshot_space_accounting_must_red_worlds.rs`（写路径建两个快照之后往 deadlist 里放一个活头还引用的块、放一个谁都不钉的块）；变异在 `crates/mutations.tsv` 里「只读快照 checker 侧」那几行。GPU 判器内核那一位判出现规则，也判 ③ ④ 与逆向：扫描内核的引用与快照那一截（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl` 的 `judge_snapshots_and_deadlists_of_the_versions`）照 `singlefs_checker::walk` 的同名函数判、置位，对拍逐位比，下标 65 不在对拍不比的表里（`crates/singlefs-checker-tier/src/crash_judge_conclusions.rs` 的 `POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS` 是空表）。出现规则取按树继承下界加 β 出现规则（用户 2026-10-02 定 ① 取 β）加缺口处置；缺口处置用户 2026-10-03 定 gap 加 n3（判决里的局部标号，只放过用户可见树里的单元）加 gap-row 两样都加。**被攻过几轮**：按树继承下界加 β 加整对不适用的缺口，在合法镜像上两轮没红（各一次抽样）；gap 加 n3 加 gap-row 的合成形、同 txg 几条并成一条、`deleted_inodes` 非全零才算继承、读不出头的单元不计入这三处措辞、谱系记录落地之后的区间形，都是零轮，终轮不再开轮，实现时再攻。实现挂在池级 checker 的主走读上（每版落点集合快照、用户可见树标签、压得住的判、新判定函数、位置键折叠、`IMPLEMENTED_INVARIANTS` 加一条、用例与变异），I-3.1（已分配统计对得上） 在短环上洞里的误红归下一批调查，与这一条共用同一个「候选集有洞」判法（`research/prompts/m3-birth-floor-r3-main-verification.md` 第二节第 4、5 行与第四节）。② 的上界半（`birth(b) ≤ R_first.txg`）另立 I-4.9（非最新候选根引用的单元不晚于它）（`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 1 行）。⚠️ **射程注（2026-09-13）**：跨头共享的块（稀疏旁表计数 > 1，D5（快照 / 空间记账机制） 已定项 6）不由 deadlist 单独管，判据 ③ / ④ 对它们不适用；它们的生死由旁表计数与各头的可达性合起来判，那条不变量还没立。 |
| I-3.6 | deadlist 紧界 | 任一 deadlist(S) 中的块满足 `birth ≤ prev(S).txg`（约定 B 下的紧界）。判别力：抓「kill 时该走直接释放却进了 deadlist」这一类（泄漏方向）。S 是活快照或活头，一条条目归哪一侧按纪元认（快照 X 那一份 = [prev(X).txg, X.txg)、活头那一份 = [prev(头).txg, ∞)，D5（快照 / 空间记账机制） 已定项 12）；`birth` 取条目里映射 key 的出生 txg 段。**豁免**（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S5 ②，攻方 H4-2）：落在未完成销毁意图 [lo, hi) 里、birth > P 的条目（写意图之后、它被放掉之前，前驱已传给下一侧）不判；「续做点之前不许再有这样的条目」那一半没有输入（续做点不存、条目被放掉就是进度，判决第七节岔路 2 的用户定案），不判。 | 已实现（2026-10-15，`crates/singlefs-checker` 的 `walk::judge_snapshots_and_deadlists_of_the_versions` 里每一版判；`IMPLEMENTED_INVARIANTS` 下标 93；写路径写出的 deadlist 上的必红世界「生在快照之后、该直接释放的单元挂进 deadlist」与 I-3.5（引用区间的精确性） ③「只被快照引用的单元在 deadlist 里少一条」在 `crates/singlefs-harness/tests/checker_snapshot_space_accounting_must_red_worlds.rs`）：一版里 deadlist 一条条目都没有时判一次、成立（同 I-9.24（删除队列至多一条 key） 空队列报成立）；条目的头不是这一版的活头、纪元落不进恰好一侧（前驱链断了，归 I-9.25（树表条目的 previous_snapshot_txg 取值））的那几条报射程里判不了；树表、deadlist 树读不全报射程里判不了。必红世界与豁免去掉要红的合法态在 `crates/singlefs-checker/src/walk.rs` 的测试模块（birth 晚于活头前驱的块进了 deadlist；写销毁意图与最后一批之间的待放条目）；变异在 `crates/mutations.tsv`「只读快照 checker 侧」那几行。盘上的必红世界要等 deadlist 写者落地（里程碑四丙 2）之后补。GPU 判器内核不置位、对拍不比。 |
| I-3.4 | 可用空间扣待删占用 | 待删中 inode 没释放的单元计在记账「已分配」里（释放只进 defer），记账标签 4（待删除但意图未完成占用）恒 0，可用空间统计因此已扣除待删占用；准入式子另扣环窗口常驻项（D28（挂载期承诺量） 已定项 1）。来源 `research/prompts/m3-delete-r3-main-verification.md` 第三节判标签 4 那一格，被攻过零轮 | 已实现（池级 checker `walk::check_pool_image`，函数 `judge_pending_deletion_occupancy_of_a_version`；坏镜像两份：标签 4 那一行的值非 0、整行缺失，登记在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `.chain`，改法在 `crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs`；GPU 判器下标 70，扫描内核判）。池级 checker 只判「标签 4 恒 0」：判的每一版里，走全了的记账树都要有标签 4 那一行且值为 0，缺行判红；另两句不在这一行判：计在「已分配」里归 I-3.1（已分配统计对得上）、I-3.11（已分配减 defer 等于最新根走读），可用空间已扣待删占用归 I-5.2（空闲统计对得上），即空闲加已分配等于容量；判哪几版、配对与读不全的报法照 I-9.5（删除队列里的号仍在树里） 的共用判法；出处 `research/prompts/m3-delete-checker-r1-main-verification.md` 第二节） |
| I-3.7 | 活快照可枚举 | 盘上存在一个可枚举的持久结构，其元素恰为当前全部活快照（既不多也不少）。这个结构是树表里不带销毁中记号的码 14 条目集合（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S1；带销毁中记号的照同一判决 S4 算销毁中、不算活快照，最后一批同一次发布删掉它的码 14、15）。可判定形式：① 每一版里每一个活快照都是这一版之前的一次捕获：诞生 txg（S.txg）不大于这一版的 txg，两条根指针的出生 txg 不大于 S.txg；② 回退候选集里 txg 相接的两版之间，活快照只经建快照那次发布多出来（诞生 txg 就是后一版的 txg）、只经写销毁意图（带上记号）与最后一批（带着记号的那一条没了）少下去，同一个快照的诞生 txg 不变；txg 不相接的两版之间不判。第二来源前驱链（码 14 的前驱 = 排在它前面那条的诞生 txg，活头 inode 条目的前驱 = 最后一条的）由 I-9.25（树表条目的 previous_snapshot_txg 取值） 判，这一条不重判。字段落点：码 14 条目住 D8（核心索引结构） 已定项 8 的树表条目字段表（种类码在 D8（核心索引结构） 已定项 9 的登记表），销毁中记号是 D5（快照 / 空间记账机制） 已定项 12 的 flags 位 0。 | 已实现（2026-10-15，`crates/singlefs-checker` 的 `walk::judge_snapshots_and_deadlists_of_the_versions` 每一版判 ①、走完判 ②；`IMPLEMENTED_INVARIANTS` 下标 94；② 依赖写意图与最后一批各是一次独立发布：core 的 `transaction::publish_snapshot_destruction_intent` 只写记号、`publish_snapshot_destruction_step` 能放的条目不多于一批时（零条也是）才是最后一批、同一次删那一对，两者从不并成一次——2026-10-15 核过）：一个快照都没有的一版 ① 判一次、成立；树表读不全报射程里判不了。必红世界在 `crates/singlefs-harness/tests/checker_snapshot_space_accounting_must_red_worlds.rs`（写路径建三个快照之后最新那一版的树表去掉中间那一对：少一条；加一对诞生 txg 是旧 txg 的：多一条）与 `crates/singlefs-checker/src/walk.rs` 的测试模块；变异在 `crates/mutations.tsv`「只读快照 checker 侧」那几行。GPU 判器内核不置位、对拍不比。 |
| I-3.8 | 实例表行唯一且低于挂载根 | 实例表 kind 0 行按实例代号唯一；行只在回收条件成立后删（整轮清扫对池中每一个落点都得出判定、其中没有该实例的未发布单元、那次清扫里没有读失败 ∧ 全部设备在线 ∧ 根环里没有该实例发布的根、且根环每个槽都读成功）；行的实例代号 < 挂载根实例（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2） | 已实现（`crates/singlefs-checker/src/walk.rs` 的 `judge_instance_table_rows`：行按实例代号唯一、每行实例代号低于最新根的实例（记录形态归 I-3.14（实例表每片记录形态合法））；最新根的实例表第 0 片解不开时报不适用，理由点名这一片解不开、行没收全；回收那一半没有输入：删行由整轮清扫做（`crates/singlefs-core/src/instance_table_sweep.rs`，可写挂载末尾一支与后台一支），checker 不判删行的条件） |
| I-3.9 | 释放代落在停止引用它的那一格区间里 | 任一带已释放标志的分配记录，它的释放代必须落在区间 `(最后一条仍引用这个落点的有效根的 checkpoint_txg, 最早不再引用它的有效根的 checkpoint_txg]` 内；根环没有洞时这个区间只有一个值，等同于「等于最早不再引用它的那条根的 txg」。若每一条有效根都还引用它，它不许带已释放标志（D3（空间分配） 已定项 7（释放时条目不删、改写 value：跨度段最高位置 1、后 8 字节从分配代换成释放代）；C374（释放代与树表诞生 txg 只有验收断言盯着））。环上有洞时（被施加的 journal 记录做的释放写着一个根槽从没写过的 txg，实测 2026-09-18 残留记录那条崩溃点重放流的 12 个状态）按区间判，按「等于」判会在合法镜像上判红；已经回收、还没被复用的已释放记录跳过不判（见证它释放的根已不在候选集里，定位不了那个区间），全是这一类时整条报不适用并带理由。判别力：把发布 B 那次的释放代从 4 写成 3 必须红（最早不再引用那十个落点的有效根是 B、txg 4）。 | 已实现（2026-09-18，`crates/singlefs-checker` 的 `walk::judge_release_generation_and_tree_table_birth`；一条根引用的落点按它自己的中央映射认（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3：搬走之后源槽在搬迁那一版起不再被引用，释放代落在搬迁那一格；`walk::RootReferences::note_located_by_the_central_mapping`）；从 deadlist 放掉的块的必红世界（释放代改成写意图那次的 txg）在 `crates/singlefs-harness/tests/checker_snapshot_space_accounting_must_red_worlds.rs`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `each_release_generation_and_tree_table_birth_bad_image_reddens_only_its_own_invariant_after_the_overwrite`；崩溃点重放每个崩溃状态都判）。**判法收窄在实现途中**：今天任一候选根走不全就整条报不适用；收窄成只在比最后引用它的那条根晚的候选根都走得全时才判这一条已释放记录，攻方造出的对照格镜像（抬 F 的空发布之后再覆盖写，把最后一次空发布那条根的树表翻坏）钉进用例（`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 10 行）。**从 deadlist 放掉的块**（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S5 ③，攻方 H5-3）：释放代落在（写意图 txg, 最后一批 txg]。落法（2026-10-15）：「一条根引用的落点」算进它活快照的两棵树（码 14 照 inode 树、码 15 照 extent 树往下数）与它中央映射树叶里每条映射条目指的单元（被钉单元的映射条目留在映射里，放掉它的那一批同一次发布删条目，同一判决 S3、S4），带销毁中记号的快照的两棵树不算（D5（快照 / 空间记账机制） 已定项 20 的例外句）；于是一个从 deadlist 放掉的块，最后还引用它的是放它那一批之前的那条根、最早不再引用它的是那一批的根，取到的区间落在（写意图 txg, 最后一批 txg] 里，比判决那一句更紧。照数销毁中快照的树，那几个块在最后一批之前的每条根上都还算引用，合法态红；不数映射条目，写意图那条根之后就不再算引用、释放代晚于区间，合法态红。变异在 `crates/mutations.tsv`「只读快照 checker 侧」那几行（单测钉「销毁中快照的两棵树不记引用」）；盘上的必红世界（从 deadlist 放掉的块释放代写成写意图那一次的 txg）要等 deadlist 写者与删快照落地（里程碑四丙 2）之后补。 |
| I-3.10 | 已分配记录的分配代等于它罩住的单元的诞生代号 | 任一**未带已释放标志**的分配记录，它的分配代（D3（空间分配） 已定项 3：`value = 分配代`；盘上编码见 D3（空间分配） 已定项 7）等于它罩住的那个单元头里的**诞生代号**（D18（块里携带什么信息） 已定项 7 的类身份段）——记录与单元是同一次发布写出来的，两个数因此同源。⚠️ **射程**：① 只管未释放的记录，已释放的那一半归 I-3.9（释放代落在停止引用它的那一格区间里）；② 记录罩住多个槽时（数据单元跨两槽）取起点槽那个单元头；③ **那个槽上读不出可用的单元头时这一条没有对象**（报不适用，不报违例）——读不出本身由 I-1.1（块头自述逻辑地址） 与 I-2.1（校验和与内容匹配） 管；④ **只读回退候选集里那几版的分配记录**（候选根的分配记录树、树表 0 条那一版根记录直接持有的那一片、由记录施加出来的那几版），**再加下一次挂载会先施加的那一版**：下一次挂载的恢复照 D23（journal 的角色与格式） 已定项 14 的重放规则施加的**第一次**发布（与 I-4.1（重放后树可遍历） 同一份重放，所选根取最新根），取它末条那一份新根段里树表指着的分配记录树；一次都不施加、锚点认不定、所选根的 txg 没有后继时没有这一版，前缀停在后面某次发布的点名单元上时仍取第一次（`walk::version_the_next_mount_applies_first` 与 `walk::replay_the_next_mount_would_do`；认法依据 `research/prompts/m3-formal-code-b36-r1-main-verification.md` 第二节 P1 与第三节第 1 条，取末条那一份依据 `research/prompts/m3-formal-code-b35-r1-main-verification.md` 第二节 P1；主 agent 2026-09-24 按代码三方 `research/prompts/m2-wave3-code-r1-main-verification.md` 第四节第 2 条定接这一步，被攻过零轮；只接这一步——恢复连着施加几次发布时第二次及以后那几版读不到，欠在欠账表）；不读被实例表判抛弃的根与低于 F 的根：低于 F 的根那棵账里未释放的记录罩住的槽，在 F 抬过之后可以被合法回收、换成诞生代号不同的单元，照「任一」去判会在合法镜像上红（主 agent 2026-09-23 定，被攻过零轮）。⚠️ 「再加下一次挂载会先施加的那一版」那一段的同源前提，靠 D16（发布语义） 已定项 7 持久顺序里单元与记录之间那道屏障撑着：拿掉那道屏障、又复用回收槽时，记录已持久而复用槽上仍是旧单元，恢复按点名验证拒施加、结局正确，这一条却在那个槽上红（2026-09-28 主 agent 复跑 floorreuse4：删屏障 271 个状态红 48、带屏障 73 个状态 0，`research/prompts/k3-9-barrier-r1-main-verification.md` 第一节）；用户 2026-09-28 定屏障维持现状、checker 不改，欠在 C592（单独拿掉单元写与记录间那道屏障今天没判据会红）。**它拦的是什么**：复用一个已回收的落点时改写了那条分配记录、却把上一次的代留着（2026-09-22 增补 3 第 1 件的代码三方第一轮攻方腿造出这条变异，快档与大档都判不出，因为全仓没有一条不变量管已分配记录的分配代） | 已实现（2026-09-23，待代码三方与崩溃验证；池级 checker `walk::judge_allocation_generations_against_unit_births`，读法照射程 ④，同一条记录（盘、槽、代）只判一次，射程 ③ 按记录逐条跳过、一条都没判到才整条报不适用；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；崩溃点重放每个崩溃状态都判） 下一次挂载会先施加的那一版由 `walk::tree_table_pointer_of_the_version_the_next_mount_applies_first` 并进读集；它的坏镜像（发布 B 的记录已落、根槽没落，B 那棵分配记录树里一条未释放记录的分配代改成诞生代号 + 1）是 `checker_known_bad_images.rs` 的 `an_allocation_generation_past_its_unit_birth_in_the_version_only_its_journal_record_carries_reddens_the_allocation_generation_invariant_before_the_mount` |
| I-3.11 | 已分配减 defer 等于最新根走读 | 逐盘：被判镜像上记账树里「已分配」那一行（记账第 1 项）减「defer 待释放」那一行（第 5 项），等于从最新有效根走读到的、这块盘上被引用的槽数。统计量的标签与行形态是 D5（快照 / 空间记账机制） 已定项 4 与已定项 10 的那几行，读的是镜像里的记账行、不读内存里的分配器；走读与 I-3.1（已分配统计对得上） 同一个走法，只取最新有效根。⚠️ 它补的是 I-3.1（已分配统计对得上） 与 I-5.2（空闲统计对得上） 都够不着的那一种：写者把活单元记成已释放、放进 defer，已分配与空闲两个和照样对得上（alloc-basis 第三轮判决 `research/prompts/alloc-basis-r3-main-verification.md` 的 Z10）。⚠️ **判别力自证在造出来的基底上做**：E156（alloc-basis 四条岔路的代价数） 第 4 次重跑在造出来的基底 β_syn（第 5 项为 0）上再记 1 槽 defer，I-3.11（已分配减 defer 等于最新根走读） 判红、去掉「减 defer」判绿（`e156-alloc-basis-counts-2026-09-26-r2.out` 的 `E7RESULT name=q7c1_not_subtracting_defer basis=beta_syn k=1 real_check_is_red=true without_subtracting_defer_is_red=false flips_red_to_green=true`）。可达状态里第 5 项能不能为 0 没量到：挂载时回退（回退写行那次发布）没有了，同一份产物里挂着回退那一版第 5 项是 356 槽（`E7RESULT name=u13a_rollback_to_oldest_candidate overwrites=72 target_txg=52 txg_before=75 txg_after=76 instance_before=1 instance_after=1 instance_table_rows_before=0 instance_table_rows_after=0 item1_slots=374 item5_slots=356 i311=holds structure_holds=true`）。隔离的记录：已分配而没有任何根引用的记录，按单元豁免——它罩住的那个单元只要有任何一份副本 checker 自己读出来读不出或校验和对不上，这个单元在每块盘上的那几条记录都豁免；每一份都读得出且对得上，照旧判违例（用户 2026-09-25 定「核出坏的才豁免」，按单元的读法是主 agent 同日定的，D19（块指针的结构与宽度预算） 已定项 5 的隔离）。 | 已实现（池级 checker `walk::check_pool_image`：逐盘判「defer 待释放 + 最新根走读 == 已分配」，最新根走读取 `walk_root(最新根)` 刚走完那一刻、与 I-3.1（已分配统计对得上） 同一个走法；最新根下面没有记账树时报不适用。坏镜像 `live_unit_recorded_as_deferred_reddens_only_the_allocated_minus_deferred_invariant`（盘 0 defer 行 +1 / −1 槽，只红它，I-3.1（已分配统计对得上） 与 I-5.2（空闲统计对得上） 照旧真被评估过且成立）；判别力自证在造出来的基底上 `on_a_synthetic_base_with_an_empty_defer_queue_one_deferred_slot_reddens_the_allocated_minus_deferred_invariant`（两盘 defer 挪回 0 之后再记一槽）；`crates/mutations.tsv` 5 条变异（判定恒真、不减第 5 项、`==` 放宽成 `≥` / `≤`、拿全部候选版本那一份走读比）；用户 2026-09-23 定采纳：alloc-basis 岔路单第 12 行（岔路 7）） |
| I-3.12 | 实例表行 T 低于引用它的根的 txg | 根环里任一自证过的根 R，它指着的实例表的每一行 (i, T, W) 满足 T < R 的 `checkpoint_txg`；链上某一片读不出的根不判。依据 D18（块里携带什么信息） 已定项 11（恢复写的行，所选根那个实例的 T 是重放之后那个根的 txg，中间实例写 0）与 D23（journal 的角色与格式） 已定项 14（新实例第一次发布的 `checkpoint_txg` 大于环里全部根与记录的号）。射程是每条根各自指着的那张表，不是只有挂载根那一张（那一张归 I-3.8（实例表行唯一且低于挂载根））；同一实例的行在两张表里是否逐字节相同不判，条款没有这一句。 | 已实现（池级 checker `walk::judge_instance_table_rows_against_the_roots_referencing_them`：每条自证过的根各沿自己的实例表链判，链上有一片读不出的根跳过，一行都没判到时报不适用；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 6 行） |
| I-3.13 | 记账行代段等于这一版的 txg | 走读到的任一版（候选根，以及由记录施加出来的那几版）的记账树里，每一行的代段（记账 key 偏移 14，8 字节）等于这一版的 `checkpoint_txg`，只判走全了的树；inode 号水位那一统计量的台阶行不判（D5（快照 / 空间记账机制） 已定项 2 射程）。依据 D5（快照 / 空间记账机制） 与 D16（发布语义） 已定项 6 的「记账代 = checkpoint 号」（每条根的记账树只装这条根那一代的行）。 | 已实现（池级 checker 走读里的 `walk_tree_table_entry` 对每一版的记账树判，版本含由记录施加出来的，只判走全了的树；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 5 行） |
| I-3.14 | 实例表每片记录形态合法 | 走读到的实例表每一片里：每条记录的 `kind` 只有 0（行）或 1（链指针记录）；链指针记录恰好是这一片的最后一条、片中间不出现，且自身合法（有无下一片 0 或 1、无下一片时位置指针全零、有下一片时不全零，且位置指针头部 MAC / nonce、算法类型、压缩算法码与压后长度、extent 偏移都是第一版的取值：有一段不是第一版的取值，这一片也解不开）；`kind` 0 行的 flags 字节为 0。依据 D18（块里携带什么信息） 已定项 11（`kind` 不是 0 或 1 ⇒ 这一片解不开；flags「第一版恒 0、非 0 拒收」；链指针记录「恒为一片的最后一条」，最后一条不是合法的链指针记录同样解不开，位置指针头部的第一版取值见 D19（块指针的结构与宽度预算） 已定项 3），用户 2026-10-06 定另立；头部那一形用户 2026-10-06 定算第五种形态（岔路单 `research/prompts/m3-batch10o-code-r1-forks.md` K2-1 那一行） | 已实现（`crates/singlefs-checker/src/walk.rs` 的 `judge_instance_table_page_record_forms` 判前三件、`walk_instance_table_chain` 里链指针记录那一格判最后一件，走到的每一片都判；不成立的一片解不开：行不收、链不往下走、候选集滤不全（I-9.29（同号只许一个出生代） 的门关）；只读不判的读法与引用集合那一遍同一个读法；GPU 走读内核同判，扫描内核的引用集合那一段与只读读法、恢复内核读实例表同判 flags；有下一片而位置指针头部有一段不是第一版取值的那一形：CPU 三处读法（主走读、只读不判的读法、引用集合那一遍）经 `crates/singlefs-checker/src/walk.rs` 的 `chain_record_view` 判，GPU 走读与扫描经 `crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl` 的 `w_chain_record_kind` 判，恢复内核读实例表同判；坏镜像两张（位置指针 MAC 不全零、extent 偏移非 0），在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`） |
| I-3.15 | 未释放记录罩住的单元由映射指在这一槽 | 回退候选集里每一版（连同由记录施加出来的那几版）的分配记录里每条未释放的记录：罩住的那一槽是根记录或父指针直接指着的（豁免三类——映射树各节点、树表、实例表各片——与根记录持有的那几样，含树表 0 条那一版由根记录持有、树 ID 0、不进映射的那棵分配记录树的各节点），成立；否则读那一槽上的单元头，头里的出生身份（类标签、诞生代号、写序实例、尾段；码 1 / 码 3 再加偏移 43 的出生树，码 2 的出生树随 C289（码 2 的出生树取哪个字段没写） 不比，同 I-1.11（映射 key 与单元头相符））与同一版中央映射里某把 key 相符、且那把 key 的位置条目指着这一槽（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3 的 N2 反查、G9 的「已死」「不一致」两类判红）。头用不了的那一格没有对象（读不出本身归 I-1.1（块头自述逻辑地址）、I-2.1（校验和与内容匹配））；隔离的单元（有一份副本读盘核出与位置项对不上，D19（块指针的结构与宽度预算） 已定项 5 硬规则 1，G9 单列的「隔离」类）不判，隔离判据读哪几份按被判这一版的事实分两支（判决 `research/prompts/m4-i315-r1-main-verification.md` 第三节 C1「乙二」，用户 2026-10-21 定案）：这一版中央映射里没有一条位置条目指着这一槽（这一槽有记录的每块盘上都没有）时，读这一版这一单元（起点槽、跨度）每块盘上未释放记录的那一份，头用不了的那一份也在里面、照同一个判据核（它对不上位置项）；有位置条目指着这一槽时只读上面头用得了、对不上映射的那几份。只看被判这一版的映射，不推别的版本里有没有人指着这一槽；一版的树表、分配记录树、中央映射树或实例表链读不全时那一版不判（射程里有一部分判不了） | 已实现（2026-10-15，里程碑四己 3：池级 checker `walk::judge_unreleased_allocation_records_against_the_central_mapping`；三形必红世界——只改映射不写副本、只写副本不改映射（只供测试的发布次序开关）、有记录没人引用（G9「已死」）——与合法搬迁之后成立的世界在 `crates/singlefs-harness/tests/checker_compaction_intent_and_relocation_must_red_worlds.rs`；「映射条目改指一份没人分配的副本、分配记录不动」那一形（按槽数加总 I-3.1（已分配统计对得上） / I-3.11（已分配减 defer 等于最新根走读） 看不见）在 `crates/singlefs-harness/tests/checker_cross_links_malformed_nodes_and_mapping_entries.rs`，主 agent 2026-10-15 定加 N2（`records/2026-10-13-里程碑四预开发.md`「己 2 交回」那一条）。映射 key 与单元头不符的几份坏镜像上与 I-1.11（映射 key 与单元头相符） 一起红。隔离那一格照 I-3.1（已分配统计对得上） 那一份隔离判据（`walk::some_copy_of_the_unit_fails_its_location_entries`），实现员 2026-10-15 推的读法，交主 agent 认；读哪几份的两支 2026-10-21 照判决 m4-i315-r1 C1 改，必绿世界「一份头用不了、映射里没有 key、两盘记录留在已分配」在 `checker_compaction_intent_and_relocation_must_red_worlds.rs`，有 key 指着的那一形照红由 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `a_shared_leaf_container_born_after_the_rollback_target_with_a_garbled_copy_reddens_the_birth_and_appearance_rules_on_either_device` 钉着。GPU 判器下标 95：扫描内核 `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl` 的 `g_judge_the_unreleased_records_of_the_version_against_its_central_mapping` 同判、置位，对拍逐位比（`crash_judge_conclusions::POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS` 是空表）） |
| I-3.16 | 碎片度与全空段统计对得上 | 最新根记账里每块盘的 runs（D5（快照 / 空间记账机制） 已定项 4 第 10 项：单元区空闲槽的连续段数）与全空聚簇段数（第 11 项；D3（空间分配） 已定项 10 ①：段内 64 槽都没有未释放分配记录的整段数，C372（全空聚簇段数两份定义相反） 按记录口径收口）等于全扫重算的值，全扫不与分配器的增量维护共享代码（C413（碎片度与全空段统计没有不变量））。占着的槽取 I-3.1（已分配统计对得上） 那一份并集（全部走过的版本的引用、隔离豁免、F 之下仍在 defer 的）；那块盘上它的槽数乘 16 KiB 不等于记账的已分配（I-3.1（已分配统计对得上） 在这块盘上不成立）、最新根带的 F 低于 F 生效值、或最新根下面没有记账树时不判 | 已实现（2026-10-15，里程碑四己 3：池级 checker `walk::judge_fragmentation_and_empty_segment_statistics`，runs 由 `walk::free_runs_between_the_occupied_slots` 全扫重算；判别力自证「增量少加一次」（runs、全空段数各少一，只红这一条）与合法搬迁之后成立的世界在 `crates/singlefs-harness/tests/checker_compaction_intent_and_relocation_must_red_worlds.rs`）。全空段那一半在两种口径分得开的盘上照判（2026-10-15 起，里程碑四写段收尾）：分配器 `allocator::DeviceFreeMap::empty_segments` 改成按记录口径增量维护（defer 槽不算占用、只数整段），C372（全空聚簇段数两份定义相反） 代码那一半收口；崩溃恢复抛弃一条根之后那份干净镜像上两种口径分得开，`checker_known_bad_images.rs` 钉住它在那里判成立。GPU 判器下标 97：内核不置位、对拍不比，主 agent 合入时补登 |

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

**出处 `.claude/kb/decisions/03-丙.en.md:1-4`（整段抄，未转述）**

```markdown
## D3: C — settled

Scope: a pure-policy sample decision.

```

**出处 `.claude/kb/decisions/03-丙.en.md:5-6`（整段抄，未转述）**

```markdown
## History

```

**出处 `.claude/kb/decisions/03-space-allocation.en.md:1-6`（整段抄，未转述）**

```markdown
## D3 Space Allocation — Settled

[中文](03-空间分配.md) · **English** · [日本語](03-領域割り当て.ja.md)

D3 (Space Allocation) governs the allocator aspect: whether space is classified, what form reservations take, granularity and vehicle and on-disk encoding of allocation records, placement of user data and commit-endogenous blocks, geometry of clustered segments, the unique registry position of allocation acceptance conjunctions, and the hard requirement that "false ENOSPC is prohibited, and deleted space must be usable within bounded steps." What it does not govern: authoritative form of the acceptance inequality and in-flight synthesis of acceptance are moved to D28 (Mount-Time Committed Allowance) settled items 1 and 2, with settled items 4 and 6 keeping only pointers without renumbering; trigger criteria, termination predicates, and resident floor for compaction are in D26 (Background Compaction and Placement Reclamation); which devices a single write lands on is in D2 (RAID Stripe Policy) settled item 8; what `df` reports and the gate for advancing F are in D16 (Publish Semantics) settled item 1; snapshot-side deadlist discipline is in D5 (Snapshot / Space Accounting Mechanism).

```

**出处 `.claude/kb/decisions/03-space-allocation.en.md:7-334`（整段抄，未转述）**

````markdown
### Settled Items

Items in D3 (Space Allocation) use a single numbering sequence, continuous with the unsettled items section, without renumbering.

| # | Item | Settlement |
|---|---|---|
| 1 | **Granularity of allocation records** | Answers whether a placement belongs to the active version per-placement, not an aggregated quantity **Status: Settled.** **Progress: Implemented.** |
| 2 | **Coexistence of tombstone units and "releasing space does not require allocating space"** | Tombstones prepaid upon writing live objects; deletion batches handled by resident ring window in acceptance formula; deletion-class publishes do not evaluate formula **Status: Settled.** **Progress: Implemented.** |
| 3 | **Carrier form of allocation records** | Resides in an independent keyspace B-tree, key is placement (device identity + in-device offset), value is allocation generation **Status: Settled.** **Progress: Implemented.** |
| 4 | **Authoritative form of acceptance inequality** | Authoritative text is in D28 (Mount-Time Committed Allowance) settled item 1; this cell retains only conclusion sentence and pointer **Status: Settled.** **Progress: Implemented.** |
| 5 | **Placement of commit-endogenous blocks** | Blocks generated by commit itself allocated from clustered segments; placement of user data blocks not subject to this constraint **Status: Settled.** **Progress: Implemented.** |
| 6 | **In-flight synthesis of acceptance** | Authoritative text is in D28 (Mount-Time Committed Allowance) settled item 2; this cell retains only conclusion sentence and pointer **Status: Settled.** **Progress: Implemented.** |
| 7 | **On-disk encoding and width of allocation record entries** | Placement granularity 16384; key = (device identity 4, slot number 6); value = (span segment 2, generation 8); rewrite on release without point deletion **Status: Settled.** **Progress: Implemented.** |
| 8 | **Default placement policy for user data blocks and fallback upon segment exhaustion** | Version 1 does not enter clustered segments; takes empty slot with smallest starting offset in selected device passing full-span allocation acceptance conjunctions; segment exhaustion falls back to same rule **Status: Settled.** **Progress: Implemented.** |
| 9 | **False ENOSPC prohibited; deleted space usable within bounded steps** | If `df` reports s bytes free, writing s bytes must succeed; write of identical size succeeds within bounded steps after deleting s bytes **Status: Settled.** **Progress: Unimplemented.** |
| 10 | **Definition of clustered segments, four placement cells for user data, bump order of commit-endogenous blocks** | Segment length 64 slots, starting point is lowest 64-slot-aligned fully empty segment; bump order by ascending tree ID, post-order within tree (leaves before root) **Status: Settled.** **Progress: Implemented.** |
| 11 | **Whether span segment in 20-byte allocation record belongs to key or value** | Enters value: key 10 bytes, value 10 bytes; released flag borrows highest bit of span segment **Status: Settled.** **Progress: Implemented.** |
| 12 | **Unique registry for allocation acceptance conjunctions and relationship between two acceptance formulas** | Conjunction formula registered solely in settled item 7 with seven clauses; two acceptance formulas are two sequential gates **Status: Settled.** **Progress: Implemented.** |
| 13 | **Whether to divide data and metadata spaces** | No division, globally unified allocator **Status: Settled.** **Progress: Implemented.** |
| 14 | **Collateral commitment of non-classification** | Background compaction must be resident, automatic, and transparent to users; undertaken by D26 (Background Compaction and Placement Reclamation) **Status: Settled.** **Progress: Unimplemented.** |
| 15 | **Form of space reservation** | Takes acceptance control: operations increasing space occupation calculate worst-case requirements to unwind themselves before entry, rejected if failing **Status: Settled.** **Progress: Implemented.** |
| 16 | **What keyless side can do when encryption is enabled** | Read-only and probe-only, neither moving nor reclaiming; resident automatic compaction commitment does not hold when encryption is enabled **Status: Settled.** **Progress: Unimplemented.** |
| 17 | **Space release operation itself does not require space allocation** | COW system must guarantee this; space needed by release path prepaid at time of write **Status: Settled.** **Progress: Implemented.** |

#### Settled Item 1: Allocation Records Answer "Belongs to Active Version" Per-Placement

**Settlement**: Allocation records are the authoritative recording point for the predicate "whether a placement belongs to the active version at this moment," not an aggregate quantity—the accounting tree in D5 (Snapshot / Space Accounting Mechanism) settled item 3 holds "(statistic, dimension tuple, generation) → full value" and cannot answer for an individual placement. The predicate is synthesized from two halves, both settled:

| Half | Provided by | Meaning |
|---|---|---|
| Has this placement been allocated | Allocation records of **D3 (Space Allocation)** | Free placement belongs to no version |
| Does the allocated placement belong to the **active** version | Deadlist of **D5 (Snapshot / Space Accounting Mechanism)** | Placement entering deadlist is referenced only by snapshots, not in live tree |

"Authoritative recording point" is a runtime measurement basis, not an ontological state category: allocation records **are derived state**, and the entire tree can be reconstructed from units + accounting + roots; authoritative state remains those three without expansion (authoritative records in D21 (Boundary Between Authoritative and Derived State) settled item 5); runtime evaluation of this predicate may only query it without ad-hoc scanning, while if lost it can be reconstructed.

**Scope**: Governs runtime per-placement evaluation of active version. Does not govern active version evaluation of multi-version units during scan reconstruction—that cell is settled per C113 (Multi-Version Unit Active Version Judgment Lacks Input in Scan Reconstruction) as "unit write sequence + instance table published predicate + ancestor table", without reading this predicate; recalculation source in D11 (Whether Index Nodes Reserve Message Buffers) settled item 4 therefore excludes allocation records. Compaction **policy** itself is outside scope (items in D26 (Background Compaction and Placement Reclamation)); what was measured is how much overhead the carrier forcibly incurs.

**Basis**:

- E59 (Can Buffered Messages Be Recalculated Solely from Units): Scanning only units cannot distinguish active values across multiple readable copies of the same key (the indistinguishable count exactly equals overwrite count); adding this predicate eliminates divergences across all four message categories—this is what the per-placement half buys.
- E74 (Allocation Record Entry Count and Compaction Cost): Per-placement entry count scales with capacity, differing in dimension from the accounting tree; neither tree was built incorrectly. Compaction path has no terms scaling with disk capacity, while COW rewrites entire nodes, creating hundreds-fold difference between contiguous and scattered keys—key in settled item 3 places compaction on favorable end.
- User settlement, original wording in history.

**Owed check**: C122 (Entry count measurement basis after non-point-deletion in allocation records); C412 (Wall-clock cost of compaction unmeasured).

#### Settled Item 2: Tombstones and livelist FREE Entries Counted into Worst Case by Acceptance Control

**Settlement**: Deletion operation itself requires disk writes—D18 (Information Carried in a Block) settled item 8 settles that "deletion requires writing tombstone units", and deleting blocks in a writable head appends a FREE entry (36 bytes, D6 (Snapshot Implementation Model) settled item 2) to livelist, while settled item 17 principle verbatim states "COW file system must guarantee that 'releasing space' operation itself does not require requesting space". Solution is not carving exceptions for deletion path, but collecting this cost in advance at write time: acceptance control in settled item 15 already mandates that "any operation increasing space usage must calculate **how much is needed to unwind itself in worst case** before admission"—**tombstones are part of 'unwinding itself'** and counted into that amount; livelist FREE entries are likewise part of "unwinding itself", counted and prepaid at write time. Tombstones are accounted per object: amortized share of one tombstone record per live object (nlink ≥ 1, including internal nodes of tombstone tree) counted into committed allowance (D5 (Snapshot / Space Accounting Mechanism) settled item 4 item 6, accounting label 6), plus resident slots pool-wide. Exact formula: label 6 = replica_count × ((2 + H_tombstone + 1 + 2 + H_retire + 1) × 16384 + 41 × n_live + 35 × n_records), in bytes, recorded as sum across replicas (C375 (Whether pending delete usage and committed reservation are per-copy or sum lacks clause)); H_tombstone is internal height of active tombstone tree, H_retire is internal height of retire tree (both 0 for null root or root-only), read live from code 2 headers of respective tree roots (precedent: ckpt_cost reading in D28 (Mount-Time Committed Allowance) settled item 4); 41 = ⌈2 × 16384 ÷ 815⌉, rounded up per replica per object then multiplied by n_live; n_live = count of records with nlink ≥ 1 in active inode tree; 35 = ⌈32768 ÷ 959⌉, n_records = total record count in active inode tree (including nlink = 0): retirement entries n_records does not count n_live because retirement records are written in final batch, where object between intention and final batch is no longer in n_live, and attaching retirement share to n_live would release it before use; for versions without `deleted_inodes` entry in tree table, label 6 is 0. New writes in deletion batches covered by resident deduction in acceptance formula "ring window = (root ring slot count 3 × S + 1) × batch upper bound W" (D28 (Mount-Time Committed Allowance) settled item 1, S is system configuration slots per region), not entering accounting rows. Deletion-class publishes (intention, batches, continuation batches in mount) skip formula check while still evaluating placement check; write-class publishes evaluated against post-publish label; ring window counted from first file version publish. ⇒ Deletion path still does not request space: settled item 17 principle unchanged verbatim; what changed is algorithm for "how much needed to unwind itself".

**Scope**: Settles who pays this charge (acceptance control, paid at write time), not how much—version 1 writes one 40-byte full object tombstone per unlink (D18 (Information Carried in a Block) settled item 24); whether batch amortization is viable and whether deletion works under full disk unquantified; deletion path implemented (`crates/singlefs-core/src/unlink.rs`), awaiting stage 2 of E168 (Deletion Borrowing Reserved Allowance Ring Window) on real deletion batches (stage 1 ran on patched replica 2026-10-03, stage 2 unrun). Tightens settled item 15 "reservation dynamically tracks real demand without idle holding": reservation bound to "whether it will be deleted in future"; tombstone term is pure waste for never-deleted objects, with waste unmeasured. Deletion-class publish skipping formula survived one attack round; write-class publish checking post-publish label and ring window from first file version survived zero attack rounds; resident slot formula 2 + H_tombstone + 1 read live survived zero attack rounds; retire term (2 + H_retire + 1, 35 × n_records) combined with retired container term in D28 (Mount-Time Committed Allowance) settled item 4 survived zero attack rounds; under "delete → rollback" cycle their combination still oversells, recorded in C617 (Tombstones Left by Post-Delete Rollback Lack Prepaid Cover).

**Basis**:

- E82 (In-Flight Synthesis of Acceptance): Forgetting tombstone charge deadlocks all deletions on full disk—"unwinding itself" containing tombstones is executable, not rhetoric.
- E130 (Does One livelist Per Head Pass Bounded Destruction): Extra bytes written per released block elevate livelist append to part of "unwinding itself"; failing to prepay conflicts with settled item 17 principle.
- User settlement (tombstone half and later livelist half), original wording in history.
- Tombstone amortized per live object plus pool-wide resident copy, deletion-class publish skipping formula, write-class publish checking post-publish label, ring window starting from first file version: `research/prompts/m3-delete-r3-main-verification.md` section 3 rows G1-tombstone prepayment, G1-deletion-class publish judgment, G1-write-class publish, G1-window starting point.
- Ring window is resident deduction in acceptance formula, not entering committed allowance: user settlements K-U4 "pool-wide single ring window" and K-R3-2 "resident deduction (recommended)", K-R3-2 rewording K-U4, original wording in history.
- Exact formula for label 6 (resident slots 2 + H_tombstone + 1 read live, 41 × n_live, sum across replicas): `research/prompts/m3-blanks-r1-main-verification.md` section 2 tombstone prepayment row; user settlement 2026-10-01 "Option B: 2 + height + 1, read live (recommended)", original wording in history.
- Label 6 adding retire term: verdict `research/prompts/m3-retire-r1-main-verification.md` section 2 K4 rows P-a, P-b (single item in ckpt_cost per genealogy still oversells without rollback, verified in first sampling, reproduced by main agent; tombstone-style prepayment adopted) and section 3 item 6; user settlement "implement both (recommended)", original wording of two modals in history.
- Ring window root count reads root ring slots 3 × S: same verdict section 2 root count in root ring row (one round).

**Owed check**: C260 (Prepayment of livelist entries not counted into unwinding amount).

#### Settled Item 3: Allocation Records Reside in an Independent Keyspace B-Tree

**Settlement**: Carrier is an independent keyspace B-tree, key = placement (device identity + in-device offset), value = allocation generation. Form follows accounting tree in D5 (Snapshot / Space Accounting Mechanism) settled item 3 with identical B-tree mechanics, avoiding a second distinct structure; key using placement is direct consequence of settled item 1 "per-placement", sharing coordinate system `(dev, offset)` with position entry in D19 (Block Pointer Structure and Width Budget) settled item 1, making placement query a point lookup rather than scan; adding keyspace is not incompat change—D15 (Format Freeze Policy) settled item 2 establishes tree table as day-1 open list, verdict table pinning "adding a keyspace" as compat. Implemented in this form first, adjusted during coding.

**Scope**: Costs after settling carrier (entry count, tree height, compaction amplification split by key space continuity) measured by E74 (Allocation Record Entry Count and Compaction Cost), belonging to settled item 1. Carrier is settled, not granularity: if measurements prove unacceptable, settled item 1 granularity reopens, not this item's carrier. On-disk encoding and width of entries belong to settled item 7; span segment placement in key or value belongs to settled item 11.

**Basis**:

- E97 (Entry Encoding of Accounting and Allocation Records): Evaluates encoding and entry width under same coordinate system; illegal placements drop to zero when encoding placement as 16 KiB slot number, coordinate system must be 16 KiB slot number.
- User settlement, original wording in history.

**Owed check**: C122 (Entry count measurement basis after non-point-deletion in allocation records).

#### Settled Item 4: Authoritative Form of Acceptance Inequality

**Settlement**: Authoritative text is in D28 (Mount-Time Committed Allowance) settled item 1; cite that location, not here. Conclusion sentence:

```
Available(d) = Capacity(d) − Allocated(d) − Unreclaimable(d) − Mount-Time Committed Allowance(d) − Abandoned Root Exclusive Allowance(d) − Pending Delete Usage ÷ Replica Count − Committed Allowance ÷ Replica Count − Checkpoint Reserved Pool ÷ Replica Count − Ring Window(d)
```

**Scope**: Cell retains only pointer and conclusion sentence; itemized measurement basis of formula, whether summed by device, why each item exists in formula, all in D28 (Mount-Time Committed Allowance) settled item 1. Numbering preserved without reordering, keeping this section.

**Basis**: No experiment: cell is pointer; authoritative text and all bases are in D28 (Mount-Time Committed Allowance) settled item 1.

**Owed check**: None.

#### Settled Item 5: Commit-Endogenous Blocks Allocated from Clustered Segments

**Settlement**: Blocks generated by commit itself—COW new nodes of allocation record tree, accounting tree, and other structures—must be allocated from clustered segments (bump segments) contiguous in key space; user data block placement not subject to this constraint. Segment geometry and bump order in settled item 10; fallback on exhaustion in settled item 8 item 2. Commit-endogenous blocks failing to find empty slots during fixed-point allocation (`NoFreeSlotOnAnyDevice`) reject this publish prior to any disk write, leaving disk unchanged; cell covers splits and clustered segment tiers uncovered by acceptance formula (C545 (Space Acceptance Fails to Cover Splits and Clustered Segment Tiers)).

**Scope**: Represents **allocator policy not format bit**—clustered segment position and length are runtime choices, undeclared by any on-disk field; implementations violating it produce fully legal images, albeit hundreds of times costlier per publish. ⇒ Restrained solely by runtime readings: commit path records "fixed-point convergence iterations for current publish", failing red if exceeding declared bound (`.claude/rules/fs-design.md` hard requirement 4: wrong branch is performance cliff, cliffs do not return errors). Whether clustered segments across devices align when w ≥ 2 (each unit's w copies landing in single column ⇒ w clustered segments advancing independently) lacks repo clauses, left to implementation of this item.

**Basis**:

- E148 (Commit Fixed-Point Recalculated with Two Record Trees): Clustered commit-endogenous blocks under two record trees yield constant fixed point; fully scattered exhibits no bound—constant bound exists solely under clustered policy.
- E81 (Fixed Point of Commit): All accounting structures are COW trees, writing nodes generates allocations, and fixed-point amplification depends on whether commit-endogenous blocks cluster—fully scattered fsync drags metadata blocks leaving I-8.1 (Ring Geometry Sufficiently Large) "worst transaction usage" without usable upper bound; clustering commit-endogenous blocks breaks chain, unaffected by fully random user data. Yields lower bound (assuming one-shot COW at publish end); interleave-as-you-insert implementations costlier.
- User settlement, original wording in history.
- `research/prompts/c363b-r1-main-verification.md` section 4 item 1, settled by main agent, survived zero attack rounds.

**Owed check**: C83 (Commit fixed-point unanswered).

#### Settled Item 6: In-Flight Synthesis of Acceptance

**Settlement**: Authoritative text is in D28 (Mount-Time Committed Allowance) settled item 2; cite that location, not here. Conclusion sentence: acceptance reading = published statistics − in-flight approved reservations; releases generated within window issue no credits; "worst-case requirement to unwind itself" includes tombstone charge.

**Scope**: Cell retains only pointer and conclusion sentence; ontological category of in-flight overlay, how overlay is recalculated when instance switch reconstructs memory state, all in D28 (Mount-Time Committed Allowance) settled item 2. Numbering preserved without reordering, keeping this section.

**Basis**: No experiment: cell is pointer; authoritative text and all bases are in D28 (Mount-Time Committed Allowance) settled item 2.

**Owed check**: None.

#### Settled Item 7: On-Disk Encoding of Allocation Record Entries and Registry of Allocation Acceptance Conjunctions

**Settlement**: Placement granularity takes 16384 bytes (one 16 KiB slot); single record covers one storage unit. key = (device identity 4, 16 KiB slot number 6) totaling 10 bytes; value = (span segment 2 containing released flag, allocation generation or release generation 8) totaling 10 bytes, generation storing full checkpoint number.

| Segment | Width | Belonging | Basis |
|---|---|---|---|
| Device identity | **4** | key | D19 (Block Pointer Structure and Width Budget) settled item 4. **Same coordinate system**, distinct definitions prohibited (settled item 3 basis item 2 verbatim) |
| Placement | **6**, encoded as **16 KiB slot number** | key | Granularity forced by D18 (Information Carried in a Block) settled item 9 (index nodes aligned to 16384, size exactly 16 KiB); E97 (Entry Encoding of Accounting and Allocation Records) criterion 7 empirically shows encoding by 32 KiB unit number fails to represent half of slots, while encoding by 16384 represents all. 6-byte slot number addresses 4 EiB / device |
| Span segment (including released flag bit) | **2 (initial value)** | value (settled item 11) | Single record covers one unit, span segment indicates how many slots it covers. Width follows D19 (Block Pointer Structure and Width Budget) settled item 3 "fixed width reserved" policy, **reason is policy not current usage**—version 1 is constant 1 or 2; **highest bit borrowed as released flag** (0 = still allocated, 1 = released), not consuming span value bits |
| Allocation generation / release generation | **8** | value | Shares width and value range with `checkpoint_txg` in D22 (Unit Atomicity Synthesis) settled item 7, semantics determined by span segment flag bit. E97 (Entry Encoding of Accounting and Allocation Records) criteria 3 / 8 empirical measurement: finite modulus merely postpones key order breakdown until wrap-around |

**Entries are not deleted, not point-deleted upon placement release** (C113 (Multi-Version Unit Active Version Judgment Lacks Input in Scan Reconstruction) settlement P4): rewritten as "released + release generation"—release generation written into those 8 bytes, released flag borrows highest bit of span segment, entry retained until overwritten upon placement reallocation. Reason against point deletion: tree sorted by placement with generation in value; point deletion by generation requires full-tree scan, while generation list needed for point deletion is precisely the memory state lost on crash; entry rewrite and COW producing this release take effect atomically in same publish—allocation records are COW tree under root (D22 (Unit Atomicity Synthesis) settled item 3), entries published with root, release generation = checkpoint_txg of that publish; crash discards unpublished checkpoint, leaving zero traces of releases therein while referenced older versions remain referenced by selected root. Memory state of defer queue lost on crash, handled by gate "release generation ≤ max(F_effective, oldest valid root in ring)" (I-7.4 (Recent K Generations Blocks Not Reused) note; predicate supplied by D16 (Publish Semantics) settled item 1, validity evaluated by instance table; mechanism to advance F depends on this predicate). "Allocation generation" measurement basis: relocation destination record writes unit birth generation, not relocation publish txg (D26 (Background Compaction and Placement Reclamation) settled item 5).

**"Release" measurement basis** = instant allocator places placement into defer queue = referenced by no root (including snapshots); placements referenced by snapshots enter deadlist rather than defer per D5 (Snapshot / Space Accounting Mechanism) deadlist discipline, released only when snapshot destroyed ⇒ release generation is later than publish where last referencing root was destroyed. Conjunction shared by both branches of **sweep acceptance** = not in allocator in-flight overlay ∧ header readable; entries require released ∧ release generation ≤ max(F_effective, oldest valid root in ring) (D16 (Publish Semantics) settled item 1 predicate; by release measurement basis, no root in ring references it; unit instance membership irrelevant, placements released by current instance must also be sweepable); orphans (no entry) require write sequence instance in header < current instance ∧ uncommitted per instance table published predicate (current instance has no row, its orphans can only be in-flight writes, left to overlay clause).

**Unique registry of allocation acceptance conjunction formula is this table** (settled item 12 points here), seven clauses total, allocation permitted only if all evaluate true:

| # | Conjunction | Origin and Scope |
|---|---|---|
| 1 | `Available ≥ Demand` | Nine-term formula of D28 (Mount-Time Committed Allowance) settled item 1 |
| 2 | Placement predicate | Settled item 1 "belongs to active version", inputs are allocation records + deadlist |
| 3 | Candidate placement not in any live concession area | D9 (Encryption) settled item 7 reopen changed `\|A\|` from "allocated" to "committed allowance", and that quantity lacks device dimension and tree dimension (D5 (Snapshot / Space Accounting Mechanism) settled item 4 item 6), slots of A having no bit in allocation record tree ⇒ key-side allocator could reallocate them; proposition 5(b) verbatim "shares single full-disk traversal with I-3.1 (Allocated Statistics Match) / I-5.1 (Physical Ranges Do Not Overlap)" = checker, not runtime gate. Form follows D26 (Background Compaction and Placement Reclamation) settled item 1 item 3 for `R`: derived from reading bounded live object set, not extra disk bit; at most one live concession area simultaneously ⇒ O(1) point check not traversal. ⚠️ D9 (Encryption) settled item 7 withdrawn following settled item 5 switch to "scrub only" (concession area A has no code object) ⇒ conjunction holds universally true, clause preserved as-is |
| 4 | Stripe table[candidate placement] contains no live members | Operates only for w ≥ 3 stripes: D2 (RAID Stripe Policy) settled item 11 rule 1 verbatim "after member of w ≥ 3 stripe released, that slot and entire stripe (including parity) must not be reused before repair completes"; w = 2 images have no parity, releasing both copies together, clause universally true. Cost is one point check per allocation (stripe table ordered by placement, sharing coordinate system with allocation record tree, D2 (RAID Stripe Policy) settled item 14), execution path resting on stripe member table carrier (D2 (RAID Stripe Policy) settled item 12) |
| 5 | Not in `R` under protection window of D26 (Background Compaction and Placement Reclamation) settled item 1 | Slots in compaction intention window prohibited as allocation targets |
| 6 | Candidate placement not in any open clustered segment (user data only) | Settled item 10 ②; commit-endogenous blocks bumped from open segments, exempt from this clause |
| 7 | `Fully empty clustered segment count ≥ N₀` | Second conjunction formula of D26 (Background Compaction and Placement Reclamation) settled item 3; `N₀` takes 1 in version 1, formula `⌈(3S − 1) × c ÷ 64⌉ + 1` (S = root slots per region, c = commit-endogenous blocks replaced per publish, calculated once at mount) finalized after count model verification (E10 (Aging: Structural Anti-Aging vs Post-Hoc Compaction)); **intercepts only foreground write-class publishes**: deletion-class, compaction batches, mount and rollback publishes exempt (user 2026-10-13 settled "intercept only foreground write-class publishes") |

**Single record exactly covering one unit is allocator invariant, not format-enforced**: span 1 must remain permanently legal (index nodes are single 16 KiB slot), so `(even slot, span 1)` record can always be written. Implemented in this form first, adjusted during coding.

**Scope**: Capacity multiplier is not criterion of this item—`.claude/rules/fs-design.md` explicitly mandates "efficiency comparison constitutes no criterion"; E97 (Entry Encoding of Accounting and Allocation Records) narrowing two candidates to only this difference could not decide superior option, cell settled by user. Upper bound of entry count shifts from "allocated placement count" to "ever-allocated, unoverwritten placement count" due to "no point deletion", still bounded by capacity / 16 KiB; E74 (Allocation Record Entry Count and Compaction Cost) criterion 1 "entry count exactly equals allocated placement count" is false by definition under new measurement basis, requiring re-run with E97 (Entry Encoding of Accounting and Allocation Records) for absolute values, unrun. Citations elsewhere describing entry form as "one per unit with span segment" reflect version prior to settled item 11—"single record covers one storage unit" remains unchanged, with span segment residing in value. Placement granularity enters two freeze tiers simultaneously (position entries in D15 (Format Freeze Policy) settled item 7 tier 2, allocation keyspace key in tier 3, tier 2 freezing before tier 3) ⇒ must be treated as tier 2 problem; heterogeneous pools (D12 (Target Media) settled item 3) prevent tying it to incompat bit of single layout line—position entry in layout A node may point to layout B device, handling still owed. Conjunctions 3 and 4 do not alter bytes written by new-pool file creation (version 1 unencrypted, two disks w = 2, both clauses universally true).

**Basis**:

- E97 (Entry Encoding of Accounting and Allocation Records): Encoding placement as 16 KiB slot number represents all legal starting points, illegal placements drop to zero; finite modulus for generation merely delays key order breakdown until wrap-around; entry count, tree height, and capacity share linear scaling with entry width on same tier, narrowing two candidates to capacity difference alone.
- E44 (Empirical Measurement and Cost of Sequence Bit Width on Host): Host publish rate yields "how long 1-byte generation wraps round", 8-byte generation is not arbitrary.
- User settlement (encoding format, third conjunction, fourth conjunction), original wording in history; C113 (Multi-Version Unit Active Version Judgment Lacks Input in Scan Reconstruction) settlement P4 provides "no point deletion".
- Verdict `research/prompts/m4-gc-r1-main-verification.md` section 3 G4 (conjunction 7 intercepts only foreground write-class publishes: attacker model freezes 0 states under foreground-only vs 2480 states under all-intercept; `N₀` formula verified via count model first, taking 1 prior), G3 (relocation destination record writes unit birth generation) and section 7 user settlement "intercept only foreground write-class publishes", original wording in history.

**Owed check**: C122 (Entry count measurement basis after non-point-deletion in allocation records); C127 (Pinned state lacks placement in allocation records); C411 (Allocation record span segment mismatch with unit category uninspected).

#### Settled Item 8: User Data Blocks Do Not Enter Clustered Segments; Placement Is Smallest Starting Free Slot in Selected Device

**Settlement**: Five rules.

1. **Version 1 user data blocks do not enter clustered segments. Placement**: After device set selected by D2 (RAID Stripe Policy) settled item 8, each selected device takes the **placement with smallest starting slot number, 32768-aligned start, both slots empty in span, not in any open clustered segment, and full span passing allocation acceptance conjunctions (seven clauses, registered solely in settled item 7)**; **no device selection, no device round-robin** (device dimension belongs to D2 (RAID Stripe Policy) settled item 8); `R` in protection window (D26 (Background Compaction and Placement Reclamation) settled item 1 item 3) is already one of the seven conjunctions, not separately listed.
2. **Clustered segments closed to user data during mount**: Clustered segments opened during this mount (currently open segment and segments opened prior to fallback) are closed to user data (settled item 10 ② "any open clustered segment" refers to these); after reopen (bump pointer and "segments opened during this mount" reside solely in memory, opening fresh segment upon mount, settled item 10 ①), segments opened in previous mount holding commit-endogenous blocks open to user data, mixed placement permitted. Settled item 5 "placement of user data blocks not subject to this constraint" holds as-is without generalization; commit-endogenous blocks falling back on segment exhaustion land in smallest slot number free slot on device, likewise excluding `R`—C146 (Fallback Policy on Segment Exhaustion Undefined in Repo) item ② (where fallback lands) receives policy, while item ① (compaction next destination falling back to segment currently being drained) belongs to D26 (Background Compaction and Placement Reclamation) settled item 1.
3. **Placements for ≤ 4 KiB tier prior to packing follow rule 1**; containers written by D27 (Small-Data Packing Container) settled item 6 packing also follow rule 1.
4. **Restricted to version 1**. Whether allocator hints are implemented and their form settled in D14 (Dual Track (Large/Small Files / Persistent/Temporary)) settled item 2, this rule being its negative arm; since rule 1 uniquely determines version 1 placement, any version 1 hint cannot hold true simultaneously—D14 (Dual Track (Large/Small Files / Persistent/Temporary)) settled item 2 reduces to "No" for version 1, reopening only if rule 1 reopens.
5. **Resident floor cites D26 (Background Compaction and Placement Reclamation) settled item 3 without separate numbers**; compaction start and stop follow D26 (Background Compaction and Placement Reclamation) settled item 1.

When disks are unequal in size in version 1, placement answers across disks must be identical; when small disk lacks fully empty segment while large disk has one, reject prior to mutating state (publish layer uniformly reporting false ENOSPC). **State explicitly that this is a version 1 limitation, not permanent criterion**; future relaxations must resolve D2 (RAID Stripe Policy) settled item 2 half (device set selection after small disk full) and settled item 8 pending item ① (whether clustered segments align across disks when w ≥ 2).

**Scope**: Represents **allocator policy**, sharing category with settled item 5—undeclared by on-disk fields; exception is **version building tree from mkfs water level for first time**, recognized on single image as: ① tree table non-empty, and extent tree entry tree ID is 65536 (water level seeded by mkfs); ② birth txg of each entry in tree table equals `checkpoint_txg` of this version. New-pool file creation falls into this class, as does the first file of instances 1, 2, 3 remounted after mkfs building from mkfs water level. User data placement for that instance must match pair calculated by settled item 8 rule 1 policy in this version allocation records (smallest start, 32768-aligned, both slots empty, not in open segment), treated as format-level, deviations producing illegal images, evaluated by I-5.5 (t1 Placement for Version Building Tree from mkfs Water Level). Value for new-pool file creation is t1@50180, i.e., unit area start slot U plus 4 (mkfs instance table occupies U, U + 1; tree table occupies U + 2; first 32768-aligned empty slot pair starts at U + 4; default 768 MiB ring U = 50176; when ring length non-default, U calculated per settled item 10 ④, t1 remaining U + 4), registered in byte table of [layout/01-first-txn.md](../layout/01-first-txn.md); after reclamation (e.g., two slots of mkfs instance table reclaimed), placement is no longer U + 4. Known limitation: checker cannot recompute reclamation threshold at moment this version was written, treating released records as occupied; implementations with reclamation threshold too strict (hardcoding scan from U + 4) or too loose (relaxing one txg) undetectable in this class; this is final, and the reclamation threshold is not recomputed (user settled 2026-10-20 "the limitation is final", original wording in history; C689 (Tree Creation Placement Criterion Fails to Catch Bad Implementation Hardcoding Scan from U + 4) paid off); the too-loose kind is judged by I-7.4 (Blocks in Recent K Generations Not Reused) on crash states before the root lands in crash-point replay; other approximations (open segments, quarantine, hold bits) itemized in `crates/singlefs-checker/invariants/07-分配器.md` row I-5.5 (t1 Placement for Version Building Tree from mkfs Water Level). For other writes (overwrites, subsequent publishes, remounted versions outside this class), violating implementations produce legal images, restrained solely by runtime readings (fallback counts and fully empty clustered segment counts in D26 (Background Compaction and Placement Reclamation) settled item 6). ⚠️ Cost of rule 2 mixed placement: old segments holding user data become fully empty only after user data in segment is released, commit-endogenous clustering degrading with mount count; manifests as monotonic decrease in fully empty clustered segment count and increase in fallbacks with mount count. **Modifies new-pool file creation bytes: Yes**—fields and widths written by new-pool file creation unchanged; placement value of user data (t1@50180, along with pointers to it and values in allocation record entries) settled by rule 1, registered as settled bytes in [layout/01-first-txn.md](../layout/01-first-txn.md) sections "0. Disk Map and Write Manifest for New-Pool File Creation" and "5. Space Accounting and Allocation". Two pending items: whether clustered segments align across disks when w ≥ 2 belongs to settled item 5 implementation; "transactions carved from same request take IDs in ascending key order" written into D16 (Publish Semantics) settled item 5, runtime check owed. Quantity runs lacks clauses consuming its absolute value (D26 (Background Compaction and Placement Reclamation) settled item 1 verbatim "threshold numbers do not enter format: awaiting E10 (Aging: Structural Anti-Aging vs Post-Hoc Compaction)").

**Basis**:

- E151 (Arrival Order of User Data Placements and Container Arm): Distinguishing quantity among candidates is **whether commit-endogenous blocks enter clustered segments**—when user data enters bump segment, vast majority fail to enter; when user data does not enter segment, value is 0 (commit-endogenous block lifetime is one checkpoint, 0 is lower bound); required by settled item 5 user settlement, upstream of quantity read by D26 (Background Compaction and Placement Reclamation) settled item 3 second conjunction. Same page provides runs overhead of this policy on block-rewrite workloads and container tiers, tying on uniform random workloads.
- E93 (Placement and Fragmentation Under Aging): Lowest free slot priority hits ceiling within hundreds of checkpoints on uniform random workloads, clustered segment arm fallback exceeding 90%—"clustered segment allocation" policy rides on compaction; this item not chosen for runs.
- Verdicts of two rounds of three-way verification in `research/prompts/d3-r2-main-verification.md`, history of rounds 1 and 2 in [records/2026-09-10-D3项8第一轮.md](../../../records/2026-09-10-D3项8第一轮.md) and [records/2026-09-13-D3项8第二轮.md](../../../records/2026-09-13-D3项8第二轮.md).
- User settlement (selecting Option G among three paths), original wording in history.
- 2026-09-23 three-way verdict `m2-placement-falsepositive-r1-main-verification.md` evaluating unequal disk placement—three erroneous members at that time reflected blank version 1 defaults; D2 (RAID Stripe Policy) settled item 2 states "not covered by settled item 2", settled item 8 has no executable object under version 1 geometry, settled item 10 permits differing slot numbers across replicas.
- User settlement 2026-09-23: when clustered segment ceases counting—codified per day's implementation: clustered segments opened during this mount remain closed, mixed placement permitted after remount; code unchanged (row 59 in milestone "overwrite, release, rollback and reuse" closing table), original wording in history; warning noted in `.claude/warnings/2026-09-23.md`.
- User settlement 2026-10-03: user data placement values count as layout-registered bytes on new-pool file creation, original wording in history; question raised in `research/prompts/e171-conclusion-r3-main-verification.md` section 5 row 1 (contradiction between D3 (Space Allocation) settled item 8 scope and `layout/01-first-txn.md` sections "0. Disk Map and Write Manifest for New-Pool File Creation" and "5. Space Accounting and Allocation").
- User settlement 2026-10-04: placement constraint governs version building tree from mkfs water level for first time (candidate A), original wording in history; recognition and criteria provided by `research/prompts/c673-clauses-r1-main-verification.md` section 2 K2 and section 5 row 1 (rules ①②, t1 = U + 4 evaluated across 6 geometries × 5 lengths, all 24 cells green, measured on attacker replica).
- User settlement 2026-10-04: placement criterion for this class switched to live calculation from current allocation records (option "f2: live calculation from allocation records") rather than hardcoded U + 4, original wording in history; verdict `research/prompts/m3-formal-code-b17-r1-main-verification.md` section 2 K2 (t1 = U + 4 and settled item 8 rule 1 incompatible after reclamation: writing first file after k ≥ 10 writable mounts post-mkfs, or pushing > 22 zero-unit publishes in single mount, falsely trips I-5.5 (t1 Placement for Version Building Tree from mkfs Water Level); measured on attacker replica, reproduced by main agent).
- User settlement 2026-10-20: the known limitation of this class's placement criterion is final (option "the limitation is final"), original wording in history; `records/2026-10-18-里程碑三收尾与近期欠账.md` line 323.

**Owed check**: C146 (Fallback Policy on Segment Exhaustion Undefined in Repo) item ①; C243 (Quantity in Termination Predicate Lacks Trajectory).

#### Settled Item 9: False ENOSPC Prohibited; Deleted Space Usable Within Bounded Steps

**Settlement**: Two hard requirements.

1. **False ENOSPC prohibited**: As long as free space reported by `df` ≥ s, writing s bytes must succeed. File system may perform bounded steps of internal work (pushing publishes, advancing rollback floor, etc.) prior to allocation; ENOSPC permitted only when disk is genuinely full (`df` free < s).
2. **Deleted space usable within bounded steps**: After deleting an object of s bytes, a write of identical size must succeed within bounded steps. Bound must be formulated as computable number, never "eventually". **Bound is 3 user-visible state-changing publishes**: deleted blocks pinned by older 3 of recent 4 states, returning to allocatable set only after 3 subsequent such publishes; prior to that `df` does not count them as free, writes encountering genuine ENOSPC per `df`, rule 1 unbroken. Empty publishes pushed by file system itself do not count toward the 3 (empty publishes do not alter user-visible state).

Decidable form is invariant I-5.3 (Reported Free Space Guaranteed Deliverable). **What it immediately changed**: what `df` reports is other half of settlement: `df` measurement basis settled in D16 (Publish Semantics) settled item 1 excludes deducted portions from free space, fulfilling rule 1 at cost of conservative reporting—deleting s bytes may increase `df` by zero bytes. Per literal wording of D16 (Publish Semantics) settled item 1, this half overreports today (verdict `research/prompts/m3-admission-r1-main-verification.md` section 3 row "`df` formulation": D16 (Publish Semantics) overreports 73/73 verbatim); `df` formulation unsettled, see C370 (Demand, Available, and df Lack Shared Units). "Bounded steps" of rule 2 calculated per arm selected in D16 (Publish Semantics) settled item 1: when acceptance insufficient, push empty publishes to advance F, at most B = 4 + 2 k_tol publishes per acceptance, reporting ENOSPC only after exhausting steps.

**Scope**: Governs solely false ENOSPC and post-deletion availability, not genuine ENOSPC; quotas excluded (version 1 excludes per-snapshot quotas). Unverified on real devices. **Bound 3 judged literally**: arm selected in D16 (Publish Semantics) settled item 1 deleting and rewriting after fill-up encounters genuine ENOSPC per its own `df` across attempts, failing literal requirements of rule 2; scope limited to k_tol = 2 root slot write failures. Reserved pool term is non-constant—calculated live per structure at each publish per D28 (Mount-Time Committed Allowance) settled item 4, delivery conditions of rule 1 varying with tree height.

**Basis**:

- E139 (Tightening Form of Per-Disk Rollback Floor): Failure windows for delete-then-rewrite after fill-up enumerated window by window, identifying windows exceeding "3 times" with artifacts; publishes per acceptance empirically within B, no deadlock under pre-run reserved pool.
- User settlement (verbatim "our goal is to avoid btrfs ENOSPC, we absolutely cannot accept ENOSPC", choosing both over rule 1 alone), original wording in history. Directly addresses two halves of symptom in [pitfalls.md](../pitfalls.md) item 1, whereas "non-classification" of settled item 13 addresses only one root cause.

**Owed check**: None (C283 (Reporting ENOSPC on Acceptance Failure Without Prior Publish Push) resolved 2026-09-26, see settled debt register).

#### Settled Item 10: Definition of Clustered Segments, Four Placement Cells for User Data, Bump Order of Commit-Endogenous Blocks

**Settlement**: ① Clustered segment length 64 slots of 16 KiB; segment start = lowest 64-slot-aligned fully empty segment in unit area; bump pointer resides solely in memory, opening fresh segment after mount; "fully empty clustered segment count" = count of segments whose 64 slots contain no unreleased allocation records (recorded by accounting row; current code `empty_segments` in `crates/singlefs-core/src/allocator.rs` uses occupancy measurement basis, to be changed to record measurement basis, C372 (Opposing Definitions of Fully Empty Clustered Segment Count)); allocator opening segments additionally checks quarantine bits (shadow accounting, D19 (Block Pointer Structure and Width Budget) settled item 5 hard rule 1) and advance-F hold bits, termed "openable clustered segments" (`lowest_empty_segment` finds segment start via this)—two quantities recorded separately, interchanging prohibited (C379 (Two Measurement Bases for Accounting Fully Empty Clustered Segments and Allocator Openable Segments) adopting Option B, settled by user 2026-09-28, survived one attack round, verdict `research/prompts/abandoned-floor-r1-main-verification.md` M3: partially quarantined segment in natural history causes 1-segment difference). ② Candidate set for user data adds conjunction "candidate placement not in any open clustered segment". ③ Data units take lowest starting slot, 32768-aligned, both slots empty, full span passing conjunctions; index nodes are commit-endogenous blocks bumped from open clustered segments (settled item 5), "lowest free slot" scoped strictly within open segment without descending outside. ④ System configuration geometry section adds "unit area start slot number" 8 bytes (version 1 = slot following journal ring end: mkfs calculates from ring length and writes into system configuration offset 417, rejecting prior to any write if start not on 64-slot segment boundary; readers selecting system configuration check 417 = computed start, rejecting whole pool on mismatch; default 768 MiB ring is 50176, audit A3b report `m2-rev-a3b-implementer-report.md`), allocator not descending below; fixed structures write no allocation records. ⑤ **Bump order of commit-endogenous blocks from open segment**: when publish rewrites instance table units (row-write publish), **instance table units come first**, trailing slice first when multiple slices exist (slice k acts as parent of slice k+1 per "leaves before root"); remainder ordered by **ascending tree ID**, post-order within tree (**leaves before root**), ascending key within layer, **central mapping tree second to last**, **tree table unit last**; code 3 packing containers take placement per "data unit" tier (32768-aligned start, both slots empty). This sequence simultaneously serves as birth sequence issuing order (D19 (Block Pointer Structure and Width Budget) settled item 9).

⑤ Placements on new-pool file creation (open segment [50240, 50304), bump cursor starting from 50240):

| Order | Unit | Tree ID | Placement | Birth Sequence |
|---|---|---|---|---|
| t2 | extent tree root and leaf (code 2, 16 KiB) | 65536 | 50240 | 0 |
| t3 | inode tree leaf container (code 3, 32 KiB, start requires 32768 alignment ⇒ skips 50241) | 65537 | 50242–50243 | 0 |
| t4 | inode tree root (code 2) | 65537 | 50244 | 1 |
| t5 | allocation record tree leaf, device 0 (code 2) | 65538 | 50245 | 0 |
| t6 | allocation record tree leaf, device 1 (code 2) | 65538 | 50246 | 1 |
| t7 | allocation record tree layer 1 node, device 0 (code 2) | 65538 | 50247 | 2 |
| t8 | allocation record tree layer 1 node, device 1 (code 2) | 65538 | 50248 | 3 |
| t9 | allocation record tree root, layer 2 (code 2) | 65538 | 50249 | 4 |
| t10 | accounting tree root and leaf (code 2) | 65539 | 50250 | 0 |
| t11 | central mapping tree root and leaf (code 2) | 65540 | 50251 | 0 |
| t12 | tree table unit (code 2) | — | 50252 | 0 |

Slot 50241 is a gap left by this sequence: t2 occupying 50240 leaves cursor at 50241, while t3 requires two 32768-aligned slots ⇒ cursor advances to 50242, leaving 50241 empty indefinitely. Increases free run count of new-pool file creation from 3 to 4 (accounting item 10, D5 (Snapshot / Space Accounting Mechanism) settled item 8).

**Scope**: In ③, "index nodes take lowest free slot" scoped strictly within open segment—reading "lowest free slot in unit area" literally would conflict with settled items 5 and 8 "clustered segments for commit-endogenous blocks only", yielding two conflicting write manifests; byte table and E142 (Dry Run of New-Pool File Creation) harness both take open segment variant. Reason for ⑤ selecting "ascending tree ID + leaves before root": tree ID is key used throughout repo (D8 (Core Index Structure) settled item 8 verbatim "entries sorted by ascending tree ID"), leaves before root is native COW order (parent pointer writable only after child placement settled); central mapping tree second to last because it holds entries of others, tree table unit last because it holds root pointers of all trees. Code 3 containers take "data unit" tier per alignment requirements of D18 (Information Carried in a Block) settled item 9 tiered by unit size regardless of user data status; both readings give identical placement in open segment, codified as clause closing C324 (Whether Code 3 Packing Containers Require 32768 Alignment Unspecified). **Modifies new-pool file creation bytes: Yes**—eleven commit-endogenous blocks (default geometry) placements reordered as a whole, allocation record keys, position entry slot numbers, central mapping entry values updating accordingly. Item bypassed three-way verification, main agent recommendation and user choice both overturnable. ⚠️ Placement table in ⑤ reflects default geometry (768 MiB ring, two 4 GiB disks): commit-endogenous block count varies with geometry (13 blocks at 1024 MiB ring length); whether images deviating from t2–t12 in this table are illegal is unclaused and unevaluated in this version (only t1 evaluated by settled item 8 scope).

**Basis**:

- E151 (Arrival Order of User Data Placements and Container Arm): Segment length 64 slots is geometry used in harness modeling, not arbitrary.
- E142 (Dry Run of New-Pool File Creation): Placement table in ⑤ and gap at 50241 are bytes written by it, byte table matching verbatim.
- User settlement (① through ④ settled per master audit options, ⑤ settled in separate modal; sentence placing instance table first in ⑤ settled 2026-09-23), original wording in history.
- `research/prompts/m2-presumed-clauses-r1-main-verification.md` K1: When clause omits instance table, among three candidates only "first" avoids conflict with ⑤ "central mapping tree second to last, tree table unit last" (local attackers consistent twice). Prior to K1, this order existed solely as row order in byte table without clauses—two implementations bumping per respective orders produced distinct placements, birth sequences, mapping keys, with nothing alerting.
- User settlement 2026-09-24: trailing slice first when instance table has multiple slices (one of two clause gaps returned by milestone 4, original wording in history); no experiment: two orderings differ only in placement and birth sequence order without measurable cost difference.
- Verdict `research/prompts/c673-clauses-r1-main-verification.md` section 2 K2: Table in ⑤ modified per implementation values (11 units, tree IDs starting from 65536, placements matching [layout/01-first-txn.md](../layout/01-first-txn.md) "0. Disk Map and Write Manifest for New-Pool File Creation"; attacker replica reading commit-endogenous blocks relative to unit area start [64, 66, 68, …, 76]), commit-endogenous block count varying with geometry (13 blocks at 1024 MiB ring length on same replica); birth sequences counted per tree starting from 0 per D19 (Block Pointer Structure and Width Budget) settled item 9.
- Verdict `research/prompts/m4-gc-r1-main-verification.md` section 3 G4: C372 (Opposing Definitions of Fully Empty Clustered Segment Count) resolved via record measurement basis, parenthesis in ① and D26 (Background Compaction and Placement Reclamation) settled item 1 item 1 consolidated into identical phrasing.

**Owed check**: None (C324 (Whether Code 3 Packing Containers Require 32768 Alignment Unspecified) resolved by ⑤, see settled debt register).

#### Settled Item 11: Span Segment in 20-Byte Allocation Record Placed in Value

**Settlement**: Placed in value: key = (device identity 4, 16 KiB slot number 6) totaling 10 bytes, value = (span segment 2, allocation generation or release generation 8) totaling 10 bytes; released flag remains in highest bit of span segment. Code 2 header of allocation record tree calculated with key width 10.

**Scope**: Governs solely which side span segment belongs to; width and semantics of field itself belong to settled item 7. **Modifies new-pool file creation bytes: Yes**—entry width field of t5, declared length, and layout of those records. Item bypassed three-way verification, main agent recommendation and user choice both overturnable.

**Basis**:

- E142 (Dry Run of New-Pool File Creation): Entry width of t5, declared length, and record layout written per width of this item, byte table matching artifacts verbatim.
- Settled item 7 verbatim "entries not deleted, rewritten as released + release generation" is an **in-place value rewrite**; if span segment entered key, flipping flag would alter key, leaving two records for same placement in tree.
- User settlement, original wording in history.

**Owed check**: None.

#### Settled Item 12: Unique Registry for Allocation Acceptance Conjunctions and Relationship Between the Two Acceptance Formulas

**Settlement**: Allocation acceptance conjunction formula registered solely in table under settled item 7 with seven clauses, all other locations pointing there. D16 (Publish Semantics) settled item 1 formula `Allocatable = min(Reallocatable + Live Metadata − Reserved Pool, df)` and D28 (Mount-Time Committed Allowance) settled item 1 nine-term formula form two sequential gates: first passing D28 (Mount-Time Committed Allowance) "Available ≥ Demand", then passing D16 (Publish Semantics) F-advancement logic. "Demand" calculated in disk bytes, identical per disk (matching measurement basis of "Allocated" per-disk physical bytes in D5 (Snapshot / Space Accounting Mechanism) settled item 7).

**Scope**: Settles registry location and sequence of two formulas, without altering content of any conjunction clause. **Modifies new-pool file creation bytes: No**—new-pool file creation always passes acceptance, item governing acceptance evaluation itself. Item bypassed three-way verification, main agent recommendation and user choice both overturnable.

**Basis**: No experiment: single registry is direct requirement of `.claude/singlefs-ai-sop/rules/kb-discipline.md` rule 4 (prior to consolidation conjunction formula had four mutually conflicting formulations), lacking measurable quantities; two-gate interpretation matches D16 (Publish Semantics) settled item 1 wording "when acceptance insufficient, push empty publish to advance F". User settlement, original wording in history; "Demand" in disk bytes identical per disk: user settlement (original wording in history).

**Owed check**: None.

#### Settled Item 13: No Split into Data and Metadata Space Categories

**Settlement**: No split into data and metadata space categories; globally unified allocator.

**Scope**: Governs whether allocator has second classification axis, irrespective of placement policies (settled items 8 and 10), device dimension (D2 (RAID Stripe Policy) settled item 8), or whether nodes and leaves of single tree are separated. Directly yields collateral commitment in settled item 14: non-classification concentrates fragmentation pressure on single allocator. Measurement basis of D26 (Background Compaction and Placement Reclamation) settled item 3 floor "the smaller the better" also built upon it—with non-classification, "releasing requiring allocation" is not fatal as in btrfs.

**Basis**: No experiment: cell settles not opening second classification axis, lacking measurable quantity. Root cause is symptom in [pitfalls.md](../pitfalls.md) item 1—btrfs two-tier allocation and rigid chunk classification causing ENOSPC when `df` reports tens of gigabytes free, failing even file deletion; per `.claude/singlefs-ai-sop/rules/evidence-discipline.md` "practices of other projects are clues, not evidence", serves as counter-evidence and mechanism, not positive proof. User settlement, original wording in history.

**Owed check**: None.

#### Settled Item 14: Collateral Commitment of Non-Classification — Background Compaction Resident, Automatic, and Transparent to Users

**Settlement**: Choosing non-classification in settled item 13 commits background compaction to being **resident, automatic, and transparent to users**—btrfs relying on manual `balance` is one of its greatest operational stigmas. Undertaking party for this commitment is D26 (Background Compaction and Placement Reclamation): mechanisms in its settled items 1, 2, 3, with trigger criteria, termination predicates, and resident floor located there, along with unsettled items.

**Scope**: Commits to "not troubling users", without committing to specific algorithms or thresholds for compaction (numbers await E10 (Aging: Structural Anti-Aging vs Post-Hoc Compaction)). Two exceptions must be cross-cited: commitment does not hold when encryption enabled (settled item 16); after D26 (Background Compaction and Placement Reclamation) settled item 7 splits reclamation into two paths, which tasks halt in leg 1 when choosing handover-to-disk remains unsettled (D26 (Background Compaction and Placement Reclamation) unsettled item 9), to be answered together with settled item 5.

**Basis**:

- E93 (Placement and Fragmentation Under Aging): Arm performing no placement hits ceiling within hundreds of checkpoints on uniform random workloads, degrading abruptly rather than slowly; clustered segment arm fallback exceeds 90%, "clustered segment allocation" policy riding on compaction ⇒ without resident compaction feeding empty segments, placement policy itself collapses.
- User settlement, original wording in history.

**Owed check**: C243 (Quantity in Termination Predicate Lacks Trajectory).

#### Settled Item 15: Space Reservation Takes Acceptance Control Form

**Settlement**: Space reservation takes acceptance control form, keeping amounts small. Reservation is not "fencing off territory as unusable", but rather: any operation increasing space occupation calculates "how much is needed to unwind itself in worst case" before admission, rejected if failing. Reservation thus dynamically tracks real demand without idle holding; amounts must be small. Executable form is nine-term formula of D28 (Mount-Time Committed Allowance) settled item 1, readings synthesized per D28 (Mount-Time Committed Allowance) settled item 2, registered in seven-clause conjunction table under settled item 7.

**Scope**: Settles form and direction (small amounts, tracking demand), settling no concrete numbers—per-item calculations in D28 (Mount-Time Committed Allowance), compaction resident floor in D26 (Background Compaction and Placement Reclamation) settled item 3. Items constituting "how much needed to unwind itself" expanded by settled item 2 (tombstones, livelist FREE entries), each expansion tightening "no idle holding".

**Basis**:

- E82 (In-Flight Synthesis of Acceptance): Three intuitive formulations each produce countable violations—reading published values alone causes intra-window overselling; recording intra-window releases as available consumes blocks defer should hold; forgetting tombstones deadlocks all deletions on full disk ⇒ acceptance control form viable only when paired with in-flight synthesis, rather than fencing off territory.
- User settlement, original wording in history.

**Owed check**: C87 (Synthetic Value Read by Acceptance Undefined).

#### Settled Item 16: Keyless Side Reads and Probes Only When Encryption Is Enabled

**Settlement**: When encryption is enabled, the keyless side **reads and probes only, neither moving nor reclaiming**.

- **Cannot move**: Four settled clauses cannot hold true simultaneously—moving a unit issues a new stripe (D2 (RAID Stripe Policy) settled item 6 verbatim "`w ≥ 2` is hard lower bound (zero-redundancy stripes prohibited)"), each slot of new stripe requires member record (D2 (RAID Stripe Policy) settled item 12), and member table verbatim "**must reside in unit payload**", payload being ciphertext (D9 (Encryption) settled item 11 "all encrypted except system configuration, metadata retaining minimal plaintext header") ⇒ keyless side cannot write member table ⇒ relocation impossible. `w ≥ 2` is hard lower bound ⇒ holds for **all** pools, not merely `w ≥ 3` pools.
- **Cannot reclaim**: Accounting is authoritative state (D21 (Boundary Between Authoritative and Derived State)) and encrypted, unwriteable by keyless side ⇒ "marking space allocatable" impossible during disk-lock period.
- **Probing always possible** (pointers carry keyless ciphertext checksums)—sole remaining capability of keyless side.

**Scope**: Costs must be stated explicitly and cross-cited—settled item 14 commitment of "resident, automatic, user-transparent background compaction" **does not hold** when encryption enabled; compaction and evacuation halt entirely during disk lock, disk replacement and volume shrink on near-full volumes likewise failing. Accepted with awareness, not an oversight. Encryption excluded from first runnable version (D9 (Encryption) settled item 10) ⇒ version 1 has no execution path.

**Basis**: No experiment: not preference shift in trade-offs, but "movable" side proved unreachable by four settled clauses, lacking measurable costs to compare. User settlement (revoking former "movable"), original wording in history; authoritative record in D9 (Encryption) settled item 5.

**Owed check**: C167 (Repo-Wide Rescan of Plaintext Mapping Layer References Unperformed).

#### Settled Item 17: Space Release Operation Itself Does Not Require Space Allocation

**Settlement**: COW system must guarantee that "releasing space" operation itself does not require space allocation. btrfs failed to guarantee this at design level, relying on heuristics patched for over a decade with ongoing issues. Any bytes required by release path (tombstone units, livelist FREE entries) prepaid at write time via acceptance control of settled item 15 (settled item 2).

**Scope**: Most rigid rule in D3 (Space Allocation), all other items conforming to it: settled item 2 settles accommodating "deletion requires disk writes" without violating it; settled item 9 item 2 settles how soon deleted space returns. Does not promise "release succeeds immediately"—generations pinned by defer window and rollback floor excluded from free space per D16 (Publish Semantics) settled item 1 measurement basis, constituting genuine ENOSPC rather than rule violation. Deletion-class publishes skipping formula evaluation (settled item 2) represents version 1 implementation of "releasing space does not require space allocation"; tombstone tree grows height when root full (D8 (Core Index Structure) settled item 6), not rejecting deletions due to structural limits.

**Basis**:

- E130 (Does One livelist Per Head Pass Bounded Destruction): Candidate A extra bytes written per released block directly collide with this principle, compelling executable interpretation of criterion "worst-case space demand per batch bounded and reservable pre-batch".
- E82 (In-Flight Synthesis of Acceptance): Arm omitting tombstone charge deadlocks all deletions upon full disk—manifestations of violating this principle are countable.
- Deletion-class publishes skipping formula: `research/prompts/m3-delete-r3-main-verification.md` section 3 row G1-deletion-class publish judgment; tombstone tree height growth when root full: same verdict section 3 row G6-tombstone root full and user settlement K-R3-3 "grow root height (recommended)", original wording in history.
- User settlement, original wording in history.

**Owed check**: C260 (Prepayment of livelist entries not counted into unwinding amount).

````

**出处 `.claude/kb/decisions/03-space-allocation.en.md:335-338`（整段抄，未转述）**

```markdown
## Revision History

Historical entries for D3 (Space Allocation) are consolidated in [decisions-history.md](../decisions-history.md).

```

**出处 `.claude/kb/decisions/03-領域割り当て.ja.md:1-6`（整段抄，未转述）**

```markdown
## D3 領域割り当て —— 確定

[中文](03-空间分配.md) · [English](03-space-allocation.en.md) · **日本語**

D3 (領域割り当て) はアロケータの枠を管轄する：領域を分類するか否か、予約がどのような形態を取るか、割り当てレコードの粒度・キャリア・ディスク上エンコーディング、ユーザーデータおよびコミット内生ブロックの配置先、クラスタセグメントの幾何、割り当て受入連言の唯一の登録場所、ならびに「偽の ENOSPC を禁止、削除された領域は有界ステップ数内に利用可能」とする硬性要件。管轄外：受入不等式の権威形式および受入のインフライト合成は D28 (マウント時コミット量) 確定項目 1 / 2 に移管され、確定項目 4 / 6 はポインタのみ残し番号は変更しない；整理のトリガー判定、停止述語、常駐最低割当額は D26 (バックグラウンド整理と配置回収)；1 回の書き込みがどのデバイスに配置されるかは D2 (RAID ストライプポリシー) 確定項目 8；`df` が何を報告するかおよび F 引き上げゲートは D16 (パブリッシュセマンティクス) 確定項目 1；スナップショット側の deadlist 規律は D5 (スナップショット / 領域会計メカニズム)。

```

**出处 `.claude/kb/decisions/03-領域割り当て.ja.md:7-334`（整段抄，未转述）**

````markdown
### 確定項目

D3 (領域割り当て) の項目番号は一連の体系のみを用い、「未確定項目」節と連番であり、番号の振り直しは行わない。

| # | 項目 | 決定事項 |
|---|---|---|
| 1 | **割り当てレコードの粒度** | 配置先ごとに「この配置先が現行バージョンに属するか」を回答可能であり、集約量ではない **状態：確定。** **進捗：実装済み。** |
| 2 | **墓石ユニットと「領域解放は領域申請を不要とする」の共存** | 墓石は生存オブジェクトの書き込み時に前払い；削除の各バッチは受入式内の常駐リングウィンドウで吸収し、削除系パブリッシュは式を判定しない **状態：確定。** **進捗：実装済み。** |
| 3 | **割り当てレコードのキャリア形態** | 独立 keyspace の B-Tree に常駐、key は配置先（デバイス識別子 + デバイス内オフセット）、value は割り当て世代 **状態：確定。** **進捗：実装済み。** |
| 4 | **受入不等式の権威形式** | 権威本文は D28 (マウント時コミット量) 確定項目 1 にあり、本セルには結論文とポインタのみを残す **状態：確定。** **進捗：実装済み。** |
| 5 | **コミット内生ブロックの配置先** | コミット自身が生成するブロックはクラスタセグメントから割り当て；ユーザーデータブロックの配置先はこの制約を受けない **状態：確定。** **進捗：実装済み。** |
| 6 | **受入のインフライト合成** | 権威本文は D28 (マウント時コミット量) 確定項目 2 にあり、本セルには結論文とポインタのみを残す **状態：確定。** **進捗：実装済み。** |
| 7 | **割り当てレコードエントリのディスク上エンコーディングと幅** | 配置先粒度 16384；key =（デバイス識別子 4, スロット番号 6）；value =（スパンセグメント 2, 世代 8）；解放時は上書きとしポイント削除は行わない **状態：確定。** **進捗：実装済み。** |
| 8 | **ユーザーデータブロックのデフォルト配置先ポリシー、およびセグメント枯渇時のフォールバック** | バージョン 1 ではクラスタセグメントに入らず、選択デバイス内で開始オフセット最小かつフルスパンで受入連言を通過する空きスロットを取得；セグメント枯渇時は同一ルールにフォールバック **状態：確定。** **進捗：実装済み。** |
| 9 | **偽の ENOSPC を禁止、削除された領域は有界ステップ数内に利用可能** | `df` が s バイト空きを報告すれば s バイトの書き込みは成功しなければならない；s バイト削除後は同一サイズの書き込みが有界ステップ数内に成功する **状態：確定。** **進捗：未実装。** |
| 10 | **クラスタセグメントの定義、ユーザーデータ配置先の 4 格、コミット内生ブロックの bump 順序** | セグメント長 64 スロット、開始点は最低位の 64 スロット境界整列された完全空きセグメント；bump 順序はツリー ID 昇順、ツリー内は後順（葉が先、根が後） **状態：確定。** **進捗：実装済み。** |
| 11 | **割り当てレコード 20 バイト中のスパンセグメントが key と value のいずれに帰属するか** | value に格納：key 10 バイト、value 10 バイト；解放済みフラグはスパンセグメントの最上位ビットを借用 **状態：確定。** **進捗：実装済み。** |
| 12 | **割り当て受入連言の唯一の登録場所、および 2 本の受入式の関係** | 連言式は確定項目 7 の 1 箇所のみに登録し全 7 条；2 本の受入式は直列の 2 つのゲートを構成 **状態：確定。** **進捗：実装済み。** |
| 13 | **data / metadata の 2 つの領域種別に分類するか否か** | 分類せず、グローバルに統一されたアロケータを使用 **状態：確定。** **進捗：実装済み。** |
| 14 | **非分類に伴う付帯コミットメント** | バックグラウンド整理は常駐・自動・ユーザー透過でなければならない；引き受け側は D26 (バックグラウンド整理と配置回収) **状態：確定。** **進捗：未実装。** |
| 15 | **領域予約が取る形態** | 受入制御を採用：領域専有を増加させるいかなる操作も、進入前に最悪ケースで自身を巻き戻すために必要な量を計算し、不成立なら拒絶 **状態：確定。** **進捗：実装済み。** |
| 16 | **暗号化有効時に鍵のない側が実施可能な操作** | 読み取りと検出のみ実施し、移動も回収も行わない；常駐自動整理のコミットメントは暗号化有効時には成立しない **状態：確定。** **進捗：未実装。** |
| 17 | **領域解放操作自体は領域の申請を不要とする** | COW システムはこれを保証しなければならない；解放パスで使用する領域は書き込みの瞬間に前払い **状態：確定。** **進捗：実装済み。** |

#### 確定項目 1：割り当てレコードが配置先ごとに「現行バージョンに属するか」を回答可能

**決定事項**：割り当てレコードは「ある配置先が現在現行バージョンに属するか否か」という述語の権威記録点であり、集約量ではない——D5 (スナップショット / 領域会計メカニズム) 確定項目 3 の会計ツリーが保持するのは「(統計量, 次元タプル, 世代) → 完全値」であり、個々の配置先には回答できない。述語は 2 つの半分から合成され、双方とも確定済みである：

| 半分 | 提供元 | 意味 |
|---|---|---|
| この配置先は割り当て済みか | **D3 (領域割り当て)** の割り当てレコード | 空き配置先はいかなるバージョンにも属さない |
| 割り当て済みのものは**現行**バージョンに属するか | **D5 (スナップショット / 領域会計メカニズム)** の deadlist | deadlist に入った配置先はスナップショットからのみ参照され、生存ツリーには存在しない |

「権威記録点」はランタイムの計測条件であり、状態分類ではない：割り当てレコード**は派生状態であり**、ツリー全体はユニット + 会計 + ルートから再構築可能であり、権威状態は依然としてその 3 点のままで項目拡張はしない（権威記録は D21 (権威状態と派生状態の境界) 確定項目 5 にある）；ランタイムにおいてこの述語を判定する際は割り当てレコードへの問い合わせのみが許容され、その場でのスキャンは禁止されるが、自身が失われても再構築可能である。

**適用範囲**：ランタイムにおいて配置先ごとに現行バージョンを判定する処理を管轄する。スキャン再構築時における複数バージョンユニットの現行バージョン判定は管轄外——その枠は C113 (スキャン再構築時における複数バージョンユニットの現行バージョン判定に入力がない) の決定事項に従い「ユニット書き込み順序 + インスタンス表のパブリッシュ済み述語 + 祖先表」に集約され、この述語は読み取らない；D11 (インデックスノードにメッセージバッファを残すか) 確定項目 4 の再計算ソースにも割り当てレコードは含まれない。整理の**ポリシー**自体は適用範囲外であり（D26 (バックグラウンド整理と配置回収) の項目）、測定されたのはキャリアが強制的に支払うコストである。

**根拠**：

- E59 (バッファメッセージはユニットのみから再計算可能か)：ユニットのみをスキャンする場合、同一 key の複数の読み取り可能コピーから現行値を判別できない（判別不能数は上書き回数に正確に一致する）。この述語を加えることで 4 種のメッセージの不一致はすべてゼロに帰着する——配置先ごとの半分が買い取った価値はこれである。
- E74 (割り当てレコードのエントリ数と整理コスト)：配置先ごとのエントリ数は容量に比例し、会計ツリーとはディメンションが異なり、いずれのツリーも構築に誤りはない。整理パスにはディスク容量に依存する項が存在しない一方、COW が書き換えるのはノード全体であり、配置先が key 空間内で連続か分散かによって数百倍の差異が生じる——確定項目 3 の key は整理を有利な側に位置付ける。
- ユーザー決定事項、原文は変更履歴にある。

**未済の検査**：C122 (割り当てレコードの非ポイント削除後のエントリ数計測条件)；C412 (整理の壁時計コストが未測定)。

#### 確定項目 2：墓石および livelist の FREE エントリを受入制御により最悪ケースに算入

**決定事項**：削除操作自体がディスク書き込みを要する——D18 (ブロックに含める情報) 確定項目 8 は「削除には墓石ユニットの書き込みが必要」と定め、書き込み可能ヘッドでブロックを削除する際はさらに livelist へ FREE エントリ（36 バイト、D6 (スナップショット実装モデル) 確定項目 2）を追加しなければならず、一方で確定項目 17 の原則には「COW システムは『領域解放』操作自体が領域申請を不要とすることを保証しなければならない」と明記されている。解決策は削除パスに例外を設けることではなく、その支払いを書き込みの瞬間に前倒しして回収することである：確定項目 15 の受入制御はもともと「領域専有を増加させるいかなる操作も、進入前に**最悪ケースで自身を巻き戻すために必要な量**を計算する」ことを要求している——**墓石は「自身を巻き戻す」ことの一部であり**、その量に計上する；livelist の FREE エントリも同様に「自身を巻き戻す」ことの一部であり、その量に計上して書き込み時に前払いする。墓石の分はオブジェクト単位で計上する：生存オブジェクト（nlink ≥ 1）1 個あたり墓石レコード 1 件分の按分（墓石ツリーの内部ノードの按分を含む）をコミット済み予約に計上し（D5 (スナップショット / 領域会計メカニズム) 確定項目 4 第 6 項、会計ラベル 6）、さらにプール全体で常駐するスロットを加える。正確な算式：ラベル 6 = レプリカ数 × ((2 + H_墓石 + 1 + 2 + H_退役 + 1) × 16384 + 41 × n_live + 35 × n_レコード)、単位はバイト、レプリカの合計として計上（C375 (削除保留専有およびコミット済み予約をコピー単位で計上するか合計で計上するかの条項がない)）；H_墓石 は現行版の墓石ツリーの内部層数、H_退役 は退役ツリーの内部層数（いずれもルートなしまたはルートのみの場合は 0）、各ツリーのルートノードのコード 2 ヘッダから直接読み取る（先例：D28 (マウント時コミット量) 確定項目 4 の ckpt_cost 読み取り方法）；41 = ⌈2 × 16384 ÷ 815⌉、各レプリカ・各オブジェクトごとに切り上げてから n_live を乗じる；n_live = この版の inode ツリーにおける nlink ≥ 1 のレコード数；35 = ⌈32768 ÷ 959⌉、n_レコード = この版の inode ツリーにおける全レコード数（nlink = 0 を含む）：退役項目数 n_レコード は n_live を数えない。なぜなら退役レコードは最終バッチで書き込まれ、意図から最終バッチまでの間にそのオブジェクトは既に n_live に含まれず、退役枠を n_live に紐付けると使用前に解放されてしまうためである；ツリーテーブルに `deleted_inodes` エントリが存在しないバージョンでは、ラベル 6 は 0 とする。削除各バッチの新規書き込みは受入式内の常駐控除項「リングウィンドウ = (ルートリングスロット数 3 × S + 1) × 各バッチ上限 W」で吸収し（D28 (マウント時コミット量) 確定項目 1、S はシステム構成の各区スロット数）、会計行には入れない。削除系パブリッシュ（意図、各バッチ、マウント時の再開バッチ）は式の判定をスキップし、配置先の判定のみ通常通り実施；書き込み系パブリッシュはパブリッシュ後のラベルに基づき判定；リングウィンドウは最初のファイルバージョンのパブリッシュから起算する。⇒ 削除パスは依然として領域を申請しない：確定項目 17 の原則は一字も変更せず、変更されたのは「自身を巻き戻すために必要な量」のアルゴリズムである。

**適用範囲**：この支払いを誰が負担するか（受入制御、書き込み時に前払い）を決定し、その金額を決定するものではない——バージョン 1 では 1 回の unlink あたり 40 バイトのオブジェクト全体の墓石を 1 件書き込む（D18 (ブロックに含める情報) 確定項目 24）；バッチ按分が可能か、ディスク満杯時に削除が動作するかは未計測である；削除パスは実装済みであり（`crates/singlefs-core/src/unlink.rs`）、E168 (削除が予約量を借用するリングウィンドウ) の第 2 段階が実実装の削除バッチ上で計測値を算出するのを待つ必要がある（第 1 段階は 2026-10-03 にパッチ適用済みレプリカ上で実行、第 2 段階は未実行）。確定項目 15 の「予約量は実際の需要に動的に追従し無駄に占有しない」を厳格化した：予約量は「将来削除されるか否か」と結合され、墓石の項は永久に削除されないオブジェクトにとっては純粋な浪費であり、浪費量も未測定である。削除系パブリッシュが式判定をスキップする仕様は 1 巡の攻撃を通過；書き込み系パブリッシュがパブリッシュ後のラベルに基づき判定しリングウィンドウが最初のファイルバージョンから起算する仕様は 0 巡の攻撃を通過；ラベル 6 の算式において常駐スロット数を 2 + H_墓石 + 1 の直接読み取りとする仕様は 0 巡の攻撃を通過；退役の項（2 + H_退役 + 1、35 × n_レコード）と D28 (マウント時コミット量) 確定項目 4 の退役コンテナ項の合算形態は 0 巡の攻撃を通過し、「削除 → ロールバック」環においてこれらを合算してもなお過剰販売となる問題は C617 (削除後ロールバックで残された墓石に前払いのカバーがない) に記録されている。

**根拠**：

- E82 (受入のインフライト合成)：墓石の支払いを忘れるとディスク満杯後の削除が全件デッドロックする——「自身を巻き戻す」に墓石を含めることは実行可能な仕様であり、単なる修辞ではない。
- E130 (ヘッドあたり 1 部の livelist は有界破棄を通過するか)：ブロック解放ごとに書き増す数バイトにより、livelist の追記は「自身を巻き戻す」の一部に引き上げられ、前払いを行わなければ確定項目 17 の原則と衝突する。
- ユーザー決定事項（墓石の半分および後から追加された livelist の半分）、原文は変更履歴にある。
- 墓石を生存オブジェクトごとに按分しプール全体で常駐 1 部を加える、削除系パブリッシュは式判定をスキップ、書き込み系パブリッシュはパブリッシュ後ラベルで判定、リングウィンドウは最初のファイルバージョンから起算：`research/prompts/m3-delete-r3-main-verification.md` 第 3 節 G1-墓石前払い、G1-削除系パブリッシュ判定法、G1-書き込み系パブリッシュ、G1-ウィンドウ起算の 4 行。
- リングウィンドウは受入式内の常駐控除項でありコミット済み予約には入らない：ユーザー決定事項 K-U4「プール全体で 1 つのリングウィンドウ」および K-R3-2「常駐控除項（推奨）」、K-R3-2 は K-U4 の文面を修正、2 件の原文は変更履歴にある。
- ラベル 6 の正確な算式（常駐スロット数 2 + H_墓石 + 1 直接読み取り、41 × n_live、レプリカ合計で計上）：`research/prompts/m3-blanks-r1-main-verification.md` 第 2 節の墓石前払いの行；ユーザー決定事項 2026-10-01「乙 2 + 層数 + 1、直接読み取り（推奨）」、原文は変更履歴にある。
- ラベル 6 に退役の項を追加：判定 `research/prompts/m3-retire-r1-main-verification.md` 第 2 節 K4 の P-a、P-b の 2 行（系譜に従い ckpt_cost 内に単独計上してもロールバックなしでも過剰販売となり、初回サンプリングで確定、メイン agent が再実行して再現；墓石方式の前払いを採用）および第 3 節第 6 条；ユーザー決定事項「双方を実施（推奨）」、2 回のモーダルの原文は変更履歴にある。
- リングウィンドウのルート数はルートリングスロット数 3 × S を読み取る：同判定第 2 節ルートリング内のルート数の行（1 巡）。

**未済の検査**：C260 (livelist エントリの前払いが自身を巻き戻す量に未算入)。

#### 確定項目 3：割り当てレコードは独立 keyspace の B-Tree に常駐

**決定事項**：キャリアは独立 keyspace の B-Tree であり、key = 配置先（デバイス識別子 + デバイス内オフセット）、value = 割り当て世代。形態は D5 (スナップショット / 領域会計メカニズム) 確定項目 3 の会計ツリーに準拠し、同一の B-Tree メカニズムを用い、第 2 の構造は新設しない；key に配置先を採用することは確定項目 1 の「配置先ごと」の直接の帰結であり、`(dev, オフセット)` は D19 (ブロックポインタの構造と幅の予算) 確定項目 1 の位置エントリと同一の座標系を共有し、配置先の検索はスキャンではなくポイント参照となる；keyspace の追加は非互換変更ではない——D15 (フォーマット凍結ポリシー) 確定項目 2 はツリーテーブルを初日からオープンリストと規定し、判定表は「keyspace の追加」を互換変更と確定している。まずはこの形態で着地させ、コーディング時に調整する。

**適用範囲**：キャリア確定後のコスト（エントリ数、ツリー高、整理増幅が key 空間の連続性により二分される点）は E74 (割り当てレコードのエントリ数と整理コスト) で測定され、確定項目 1 に属する。確定したのはキャリアであり、粒度ではない：測定値が受け入れ不能な場合、再審議されるのは確定項目 1 の粒度であり、本項のキャリアではない。エントリのディスク上エンコーディングと幅は確定項目 7 に属し、スパンセグメントが key と value のいずれに帰属するかは確定項目 11 に属する。

**根拠**：

- E97 (会計と割り当てレコードのエントリエンコーディング)：同一座標系下のエンコーディングとエントリ幅を判定し、配置先を 16 KiB スロット番号でエンコードすると不正配置先はゼロに帰着するため、座標系は 16 KiB スロット番号でなければならない。
- ユーザー決定事項、原文は変更履歴にある。

**未済の検査**：C122 (割り当てレコードの非ポイント削除後のエントリ数計測条件)。

#### 確定項目 4：受入不等式の権威形式

**決定事項**：権威本文は D28 (マウント時コミット量) 確定項目 1 にあり、本項を引用する際はそちらを引用し、ここを引用してはならない。結論文：

```
利用可能(d) = 容量(d) − 割り当て済み(d) − 回収不能(d) − マウント時コミット量(d) − 放棄ルート排他量(d) − 削除保留専有 ÷ レプリカ数 − コミット済み予約 ÷ レプリカ数 − チェックポイント保留プール ÷ レプリカ数 − リングウィンドウ(d)
```

**適用範囲**：本セルにはポインタと結論文のみを残す；算式の各項の計測条件、デバイスごとに合算するか否か、各項がなぜ式に存在するのかはすべて D28 (マウント時コミット量) 確定項目 1 にある。番号の振り直しは行わないため、本節を保持する。

**根拠**：実験なし：本セルはポインタに過ぎず、権威本文とすべての根拠は D28 (マウント時コミット量) 確定項目 1 にある。

**未済の検査**：なし。

#### 確定項目 5：コミット内生ブロックはクラスタセグメントから割り当て

**決定事項**：コミット自身が生成するブロック——割り当てレコードツリー、会計ツリー等の構造の COW 新ノード——は、key 空間上で連続するクラスタセグメント（bump セグメント）から割り当てられなければならない；ユーザーデータブロックの配置先はこの制約を受けない。セグメントの幾何および bump 順序は確定項目 10、セグメント枯渇時のフォールバックは確定項目 8 第 2 条にある。コミット内生ブロックが不動点割り当て時に空きスロットを見つけられない場合（`NoFreeSlotOnAnyDevice`）、いかなる書き込みよりも前に今回のパブリッシュを拒絶し、ディスク上は変更しない；本セルは受入式がカバーできない分割およびクラスタセグメント層を吸収する（C545 (領域受入が分割とクラスタセグメント層をカバーできない)）。

**適用範囲**：これは**アロケータポリシーでありフォーマットビットではない**——クラスタセグメントの位置と長さはランタイムの選択であり、ディスク上のいかなるフィールドもこれを宣言しない；これに違反する実装が書き出したイメージも完全に正当であるが、パブリッシュあたりのコストが数百倍高くなる。⇒ これを阻止するにはランタイムの読み取り値に頼るほかない：コミットパスは「今回のパブリッシュの不動点収束ラウンド数」を記録し、宣言上限を超えた場合は不合格と判定する（`.claude/rules/fs-design.md` 硬性要件 4：分岐の誤選択は性能の崖であり、崖はエラーを返さない）。w ≥ 2 の場合に各デバイス上のクラスタセグメントを整列させるか否か（各ユニットの w 個のレプリカが 1 列に整然と配置 ⇒ w 個のクラスタセグメントが個別に進行）についてはリポジトリ全体に条項がなく、本項の実装に委ねられる。

**根拠**：

- E148 (コミット不動点を 2 本のレコードツリーで再計算)：2 本のレコードツリー下でコミット内生ブロックをクラスタ化すれば定数の不動点が得られるが、完全散布時には上限が存在しない——定数上限はクラスタポリシー下でのみ存在する。
- E81 (コミットの不動点)：すべての会計構造は COW ツリーであり、それらにノードを書き込むと再び割り当てが発生し、不動点ループの増幅はコミット内生ブロックがクラスタ化されているか否かによって決まる——完全散布時には 1 回の fsync が引き連れるメタデータブロック数により I-8.1 (リング幾何が十分に大きい) の「最悪トランザクション専有」に使用可能な上限が存在しなくなるが、コミット内生ブロックのみをクラスタ化すれば連鎖が断ち切られ、ユーザーデータが完全にランダムであっても影響しない。提示されたのは下限であり（パブリッシュ末尾で一括 COW と仮定）、挿入しながら COW を行う実装はさらに高コストとなる。
- ユーザー決定事項、原文は変更履歴にある。
- `research/prompts/c363b-r1-main-verification.md` 第 4 節第 1 条、メイン agent が決定、0 巡の攻撃を通過。

**未済の検査**：C83 (コミット不動点に回答がない)。

#### 確定項目 6：受入のインフライト合成

**決定事項**：権威本文は D28 (マウント時コミット量) 確定項目 2 にあり、本項を引用する際はそちらを引用し、ここを引用してはならない。結論文：受入読み取り値 = パブリッシュ済み統計量 − インフライト承認済み予約；ウィンドウ内で発生した解放はクレジットを発行しない；「最悪ケースで自身を巻き戻すために必要な量」には墓石の分を含める。

**適用範囲**：本セルにはポインタと結論文のみを残す；インフライト overlay の状態分類、インスタンス切り替えによるメモリ状態再構築時の overlay 再計算方法はすべて D28 (マウント時コミット量) 確定項目 2 にある。番号の振り直しは行わないため、本節を保持する。

**根拠**：実験なし：本セルはポインタに過ぎず、権威本文とすべての根拠は D28 (マウント時コミット量) 確定項目 2 にある。

**未済の検査**：なし。

#### 確定項目 7：割り当てレコードエントリのディスク上エンコーディングおよび割り当て受入連言の登録表

**決定事項**：配置先粒度は 16384 バイト（16 KiB スロット 1 個）を採用；1 件のレコードが 1 個のストレージユニットを記録する。key = (デバイス識別子 4, 16 KiB スロット番号 6) の計 10 バイト；value = (解放済みフラグを含むスパンセグメント 2, 割り当て世代または解放世代 8) の計 10 バイト、世代には完全なチェックポイント番号を格納する。

| セグメント | 幅 | 帰属 | 根拠 |
|---|---|---|---|
| デバイス識別子 | **4** | key | D19 (ブロックポインタの構造と幅の予算) 確定項目 4。**同一の座標系**であり、個別定義は禁止（確定項目 3 根拠第 2 条の原文ママ） |
| 配置先 | **6**、**16 KiB スロット番号**としてエンコード | key | 粒度は D18 (ブロックに含める情報) 確定項目 9 により要請（インデックスノードは 16384 に整列、サイズは正確に 16 KiB）；E97 (会計と割り当てレコードのエントリエンコーディング) 判定基準 7 の実測により、32 KiB ユニット番号でエンコードするとスロットの半分が表現不能となり、16384 でエンコードすると全スロット表現可能。6 バイトのスロット番号で 4 EiB / デバイスをアドレッシング |
| スパンセグメント（解放済みフラグビットを含む） | **2（初期値）** | value（確定項目 11） | 1 件のレコードが 1 ユニットを記録し、スパンセグメントはそれが何スロットをカバーするかを示す。幅は D19 (ブロックポインタの構造と幅の予算) 確定項目 3 の「固定幅・将来予約」ポリシーに従い、**根拠はポリシーであり現在の使用量ではない**——バージョン 1 では常に 1 または 2；**最上位ビットを解放済みフラグとして借用**（0 = 割り当て中、1 = 解放済み）、スパン値を表現するビットは消費しない |
| 割り当て世代 / 解放世代 | **8** | value | D22 (ユニット原子性の合成) 確定項目 7 の `checkpoint_txg` と同幅・同一値域であり、セマンティクスはスパンセグメントのフラグビットによりいずれかを選択。E97 (会計と割り当てレコードのエントリエンコーディング) 判定基準 3 / 8 の実測：いかなる有限剰余系もラップアラウンドの瞬間まで key 順序の破綻を先送りするに過ぎない |

**配置先解放時にエントリは削除せず、ポイント削除も行わない**（C113 (スキャン再構築時における複数バージョンユニットの現行バージョン判定に入力がない) 決定事項 P4）：「解放済み + 解放世代」へと書き換える——解放世代をその 8 バイトに書き込み、解放済みフラグビットはスパンセグメントの最上位ビットを借用し、エントリはその配置先が再割り当てされる際に上書きされるまで保持される。ポイント削除を行わない理由：ツリーは配置先順でソートされ世代は value に常駐するため、世代によるポイント削除はツリー全体のスキャンを要し、ポイント削除に必要な世代別リストこそがクラッシュで失われるメモリ状態そのものである；エントリ書き換えと今回の解放を生み出した COW は同一パブリッシュ内でアトミックに有効化される——割り当てレコードはルート配下の COW ツリーであり（D22 (ユニット原子性の合成) 確定項目 3）、エントリとルートは同一パブリッシュでコミットされ、解放世代 = そのパブリッシュの checkpoint_txg となる；クラッシュで失われるのは未パブリッシュのチェックポイントであり、その中の解放は痕跡を残さず、対応する旧バージョンは選択されたルートから引き続き参照される。クラッシュ後に defer キューのメモリ状態が失われた後は、「解放世代 ≤ max(F_有効, リング内最古有効ルート)」ゲートが引き継ぐ（I-7.4 (直近 K 世代のブロックは再利用されない) の注；述語は D16 (パブリッシュセマンティクス) 確定項目 1 が提供、有効性はインスタンス表に基づき判定；F 引き上げのメカニズムはこの述語を担保とする）。「割り当て世代」の計測条件：移動先のレコードにはユニットの誕生世代を書き込み、移動時のパブリッシュ txg は書き込まない（D26 (バックグラウンド整理と配置回収) 確定項目 5）。

**「解放」の計測条件** = アロケータが配置先を defer キューに入れた瞬間 = ルート（スナップショットを含む）から一切参照されなくなった時点；スナップショットから参照されている配置先は D5 (スナップショット / 領域会計メカニズム) の deadlist 規律に従い defer ではなく deadlist に入り、スナップショット破棄時に初めて解放される ⇒ 解放世代はそれを参照する最後のルートが破棄されたパブリッシュよりも後になる。**清掃受入**の双方の分岐に共通する連言 = アロケータのインフライト overlay に存在しない ∧ ヘッダが読み取り可能；エントリが存在するものはさらに解放済み ∧ 解放世代 ≤ max(F_有効, リング内最古有効ルート) を要求（D16 (パブリッシュセマンティクス) 確定項目 1 の述語；解放の計測条件より、この時点でリング内にそれを参照するルートは存在しない；ユニットがどのインスタンスに属するかは無関係であり、現行インスタンス自身が解放した配置先も清掃可能でなければならない）；孤児（エントリなし）はヘッダ内の書き込みシーケンスインスタンス < 現行インスタンス ∧ インスタンス表のパブリッシュ済み述語により未パブリッシュと判定されることを要求（現行インスタンスには行がなく、その孤児はインフライトの書き込みにしかなり得ず、overlay の条項に委ねられる）。

**割り当て受入の連言式は、本テーブルが唯一の登録場所である**（確定項目 12 がここを指示）、全 7 条、すべて真の場合にのみ割り当てを許可：

| # | 連言 | 出典と適用範囲 |
|---|---|---|
| 1 | `利用可能 ≥ 需要` | D28 (マウント時コミット量) 確定項目 1 の 9 項式 |
| 2 | 配置先述語 | 確定項目 1 の「現行バージョンに属するか」、入力は割り当てレコード + deadlist |
| 3 | 候補配置先がいかなるアクティブな譲渡区にも属さない | D9 (暗号化) 確定項目 7 の再審議により `\|A\|` は「割り当て済み」から「コミット済み予約」へと改められ、その量はデバイス次元およびツリー次元を持たないため（D5 (スナップショット / 領域会計メカニズム) 確定項目 4 第 6 項）、A のスロットは割り当てレコードツリー内で一切のビット表現を持たなくなる ⇒ 鍵側の割り当て機構はそれらを再割り当て可能となる；命題 5(b) 原文ママ「I-3.1 (割り当て済み統計の整合) / I-5.1 (物理範囲の非重複) と同一のフルディスク走査を共有する」= チェッカーであり、ランタイムゲートではない。形態は D26 (バックグラウンド整理と配置回収) 確定項目 1 第 3 条の `R` の記述に準拠：有界な生存オブジェクト集合を読み取ることで演繹し、ディスク上にビットを追加することには依存しない；アクティブな譲渡区は同時に最大 1 個 ⇒ O(1) ポイント参照であり走査ではない。⚠️ D9 (暗号化) 確定項目 7 は確定項目 5 が「スクラブのみ」へと改められたことに伴い撤回（譲渡区 A は対象なし）⇒ 本連言は常に真、条項はそのまま保持 |
| 4 | ストライプ表[候補配置先] に生存メンバーが存在しない | w ≥ 3 のストライプにのみ作用：D2 (RAID ストライプポリシー) 確定項目 11 ルール 1 原文ママ「w ≥ 3 ストライプのメンバーが解放された後、当該スロットおよびストライプ全体（パリティを含む）は修復完了まで再利用してはならない」；w = 2 のイメージにはパリティがなく 2 コピー同時に解放されるため、本条は常に真。コストは割り当てごとに 1 回のポイント参照が追加されること（ストライプ表は配置先順にソートされ、割り当てレコードツリーと同一座標系を共有、D2 (RAID ストライプポリシー) 確定項目 14）、ポイント参照の実行パスはストライプメンバー表のキャリアに依存（D2 (RAID ストライプポリシー) 確定項目 12）|
| 5 | D26 (バックグラウンド整理と配置回収) 確定項目 1 の保護期間内の `R` に含まれない | 整理意図期間内のそれらスロットは割り当て先として禁止 |
| 6 | 候補配置先がいかなるオープンなクラスタセグメントにも属さない（ユーザーデータのみ対象）| 確定項目 10 ②；コミット内生ブロックはまさにオープンセグメントから bump されるため、本条の制約を受けない |
| 7 | `完全空きクラスタセグメント数 ≥ N₀` | D26 (バックグラウンド整理と配置回収) 確定項目 3 の第 2 連言式；`N₀` はバージョン 1 では 1、算式 `⌈(3S − 1) × c ÷ 64⌉ + 1`（S = 各区ルートスロット数、c = 1 パブリッシュで交換されるコミット内生ブロック数、マウント時に 1 回計算）は計数モデルの検証を経て確定（E10 (エージング：構造の耐エージング性 vs 事後整理)）；**フォアグラウンドの書き込み系パブリッシュのみを阻止**：削除系、整理バッチ、マウントおよびロールバックのパブリッシュは本条を免除（ユーザーが 2026-10-13 に「フォアグラウンド書き込み系パブリッシュのみを阻止」と決定）|

**1 件のレコードが正確に 1 ユニットをカバーすることはアロケータの不変条件であり、エンコーディングにより強制されるものではない**：スパン 1 は永久に正当でなければならず（インデックスノードは単一の 16 KiB スロット）、したがって `(偶数スロット, スパン 1)` というレコードは常に書き出し可能である。まずはこの形態で着地させ、コーディング時に調整する。

**適用範囲**：容量専有倍率は本項の判定基準ではない——`.claude/rules/fs-design.md` は「効率の比較はいかなる判定基準も構成しない」と明記しており、E97 (会計と割り当てレコードのエントリエンコーディング) が 2 つの候補をこの差のみに絞り込んだ後に優劣を判定できず、本セルはユーザーが決定した。エントリ数の上限は「非ポイント削除」に伴い「割り当て済み配置先数」から「過去に割り当てられ、まだ上書きされていない配置先数」へと変化し、依然として容量 / 16 KiB が境界となる；E74 (割り当てレコードのエントリ数と整理コスト) 判定基準 1「エントリ数は正確に割り当て済み配置先数に一致する」はこの新計測条件のもとで定義上偽となり、計測条件変更後は E97 (会計と割り当てレコードのエントリエンコーディング) と共に再実行して新たな絶対値を取得する必要があるが、未実行。他所で本エントリ形態を「1 ユニットを 1 件で記録し別途スパンセグメントを保持」と引用している箇所は確定項目 11 以前の版の記述を採用している——「1 件のレコードが 1 ストレージユニットを記録」という半分は従前のままであり、スパンセグメントは value に格納される。配置先粒度の数値は同時に 2 つの凍結層に入り（位置エントリは D15 (フォーマット凍結ポリシー) 確定項目 7 の第 2 層、割り当て keyspace の key は第 3 層にあり、第 2 層は第 3 層より先に凍結される）⇒ 第 2 層の問題として扱う必要がある；異種プール（D12 (ターゲットメディア) 確定項目 3）により特定のレイアウト線の非互換ビットに紐付けることができず——レイアウト A のノード内の位置エントリがレイアウト B のデバイスを指す場合があり、この枠は依然として対応が未済である。連言 3 および連言 4 はいずれも新規プールでのファイル作成が書き出すバイト列を変更しない（バージョン 1 は非暗号化、ディスク 2 台で w = 2 であり、双方とも恒真）。

**根拠**：

- E97 (会計と割り当てレコードのエントリエンコーディング)：配置先を 16 KiB スロット番号でエンコードすると正当な開始点はすべて表現可能となり不正配置先はゼロに帰着する；世代を有限剰余系でエンコードすることはラップアラウンドまで key 順序の破綻を先送りするに過ぎない；同ティアにおけるエントリ数、ツリー高、容量専有はエントリ幅に比例し、2 つの候補の差異は容量専有のみに絞り込まれた。
- E44 (シーケンスビット幅の実機実測とコスト)：実機のパブリッシュレートから「1 バイトの世代が何時間で一巡するか」を算出し、世代に 8 バイトを採用した根拠は恣意的なものではない。
- ユーザー決定事項（エンコーディング形態、第 3 連言、第 4 連言）、原文は変更履歴にある；C113 (スキャン再構築時における複数バージョンユニットの現行バージョン判定に入力がない) 決定事項 P4 が「非ポイント削除」を提供。
- 判定 `research/prompts/m4-gc-r1-main-verification.md` 第 3 節 G4（連言 7 はフォアグラウンド書き込み系パブリッシュのみを阻止：攻撃側モデルにおいてフォアグラウンドのみ阻止のアームは凍結 0、全阻止のアームは凍結 2480 状態；`N₀` の算式は計数モデルで先に検証し、検証前は 1 とする）、G3（移動先レコードにはユニットの誕生世代を書き込む）および第 7 節のユーザー決定事項「フォアグラウンド書き込み系パブリッシュのみ阻止」、原文は変更履歴にある。

**未済の検査**：C122 (割り当てレコードの非ポイント削除後のエントリ数計測条件)；C127 (固定状態が割り当てレコード内に配置先を持たない)；C411 (割り当てレコードのスパンセグメントとユニット種別の不一致が未検査)。

#### 確定項目 8：ユーザーデータブロックはクラスタセグメントに入らず、配置先は選択デバイス内で開始スロット番号最小の空きスロット

**決定事項**：5 条。

1. **バージョン 1 のユーザーデータブロックはクラスタセグメントに入らない。配置先**：D2 (RAID ストライプポリシー) 確定項目 8 によりデバイス集合が選択された後、選択された各デバイス上でそれぞれ**開始スロット番号最小、開始位置が 32768 整列、スパン内の両スロットが空き、いかなるオープンなクラスタセグメントにも属さず、フルスパンで割り当て受入連言（全 7 条、唯一の登録場所は確定項目 7）を通過する配置先**を取得；**デバイス選択・ラウンドロビンは行わない**（デバイス次元は D2 (RAID ストライプポリシー) 確定項目 8 に属する）；保護期間内の `R`（D26 (バックグラウンド整理と配置回収) 確定項目 1 第 3 条）は既にその 7 条の連言の 1 つに含まれており、別途列挙しない。
2. **クラスタセグメントはマウント期間中ユーザーデータに対して閉鎖**：今回のマウントでオープンされたクラスタセグメント（現在オープン中のセグメント、およびフォールバック前にオープンされたセグメント）はいずれもユーザーデータに提供しない（確定項目 10 ② の「いかなるオープンなクラスタセグメント」はこれらセグメントを指す）；再オープン後（bump ポインタおよび「今回のマウントでどのセグメントをオープンしたか」はメモリのみに保持され、マウント後に新たにセグメントをオープン、確定項目 10 ①）、前回のマウントでオープンされコミット内生ブロックを格納していたセグメントはユーザーデータに開放され、混在配置が許容される。確定項目 5「ユーザーデータブロックの配置先はこの制約を受けない」はそのまま成立し、一般化しない；コミット内生ブロックのセグメント枯渇時は当該デバイス内でスロット番号最小の空きスロットにフォールバックし、同様に `R` を除外する——C146 (空きセグメント枯渇時のフォールバックポリシーがリポジトリ全体で未定義) 第 ② 条（フォールバック先）によりポリシーが定まり、第 ① 条（整理の次の移動先が自身が解放中のセグメントに戻る問題）は本項ではなく D26 (バックグラウンド整理と配置回収) 確定項目 1 に属する。
3. **≤ 4 KiB ティアのパッキング前の配置先は第 1 条と同一**；D27 (小データパッキングコンテナ) 確定項目 6 のパッキングで書き出されるコンテナも第 1 条に従う。
4. **バージョン 1 に限定**。アロケータヒントを導入するか否か、ヒントの記述形式は D14 (二重トラック (大小ファイル / 永続一時)) 確定項目 2 で判定され、本条はその減数アームとなる；第 1 条がバージョン 1 の配置先を一意に確定している以上、いかなるバージョン 1 のヒントもそれと両立し得ない——D14 (二重トラック (大小ファイル / 永続一時)) 確定項目 2 はバージョン 1 において「否」のみが残り、当該項目の再審議条件の 1 つは第 1 条の再審議である。
5. **最低割当額は D26 (バックグラウンド整理と配置回収) 確定項目 3 を引用し、数値を別途記載しない**；整理の起動と停止は D26 (バックグラウンド整理と配置回収) 確定項目 1 に従う。

バージョン 1 でディスクサイズが不揃いの場合、各ディスクの配置先の解は同一でなければならない；小容量ディスクに完全空きセグメントがなく大容量ディスクにある場合、いかなる状態変更の前にも拒絶する（パブリッシュ層が一律に偽の ENOSPC を報告）。**これがバージョン 1 の制限であり恒久的な判定基準ではないことを明記する**。将来解放する際に回答すべき課題は D2 (RAID ストライプポリシー) 確定項目 2 の半分（小容量ディスク満杯後のデバイス集合選択）および確定項目 8 保留事項 ①（w ≥ 2 時の各ディスクのクラスタセグメント整列要否）である。

**適用範囲**：これは**アロケータポリシー**であり、確定項目 5 と同種——ディスク上にこれを宣言するフィールドは存在しない；例外は**mkfs 水位から初めてツリーを構築したバージョン**であり、単一イメージ上での判定法は ① ツリーテーブルが非空であり、かつ extent ツリーエントリのツリー ID が 65536（mkfs が設定した水位）；② ツリーテーブル内の各エントリの誕生 txg がこのバージョンの `checkpoint_txg` に等しいこと。新規プールでのファイル作成はこの種別に属し、mkfs 後の再マウント、インスタンス 1、2、3 の最初のファイルも mkfs 水位からツリーを構築する限り同様である。この種別においてその回のユーザーデータ配置先は、この版の割り当てレコード内で確定項目 8 第 1 条のポリシーにより算出されたペア（開始位置最小、32768 整列、両スロット空き、オープンセグメント外）に等しくなければならず、フォーマットレベルとみなされ、それから逸脱して書き出されたイメージは不正と判定され、判定を行うのは I-5.5 (mkfs 水位からツリー構築したバージョンの t1 配置先) である。新規プールでのファイル作成における値は t1@50180、すなわちユニット領域開始スロット番号 U に 4 を加えたもの（mkfs のインスタンス表が U, U + 1 を占有、ツリーテーブルが U + 2 を占有、最初の 32768 整列の空きスロットペアは U + 4 から開始；デフォルトの 768 MiB リングでは U = 50176、リング長がデフォルトでない場合は U は確定項目 10 ④ に従い直接計算、t1 は依然として U + 4）、[layout/01-first-txn.md](../layout/01-first-txn.md) のバイト表に登録；回収後（例：mkfs のインスタンス表の 2 スロットが回収された後）の配置先はもはや U + 4 ではない。既知の限界：チェッカーはこの版のディスク面からこの版を書き込んだ瞬間の回収閾値を再計算できず、解放済みレコードは一律占有中として扱うため、回収閾値が厳しすぎる（U + 4 からのスキャンをハードコード）または緩すぎる（1 txg 緩和）不良な実装はこの種別において検出不能である；これは確定であり、回収閾値は再計算しない（ユーザーが 2026-10-20 に「限界で確定」と決定、原文は変更履歴にある；C689 (ツリー構築配置先判定基準が U + 4 からのスキャンをハードコードした不良実装を捕捉できない) は解消済み）、緩すぎる種別はクラッシュポイント再生でルートが永続化される前のクラッシュ状態上で I-7.4 (直近 K 代ブロック非再利用) が判定する；その他の近似（オープンセグメント、隔離および保留ビット）は `crates/singlefs-checker/invariants/07-分配器.md` の I-5.5 (mkfs 水位からツリー構築したバージョンの t1 配置先) の行に逐一記載。それ以外の書き込み（上書き、その後のパブリッシュ、再マウント後にこの種別に属さないバージョン）においてこれに違反する実装が書き出したイメージは正当であり、これを阻止するにはランタイムの読み取り値に頼るほかない（D26 (バックグラウンド整理と配置回収) 確定項目 6 のフォールバック回数と完全空きクラスタセグメント数）。⚠️ 第 2 条の混在配置のコスト：ユーザーデータを格納した旧セグメントはセグメント内のユーザーデータも解放されて初めてセグメント全体が空くため、コミット内生ブロックのクラスタ化はマウント回数に伴い劣化する；現象として完全空きクラスタセグメント数がマウント回数に伴い単調減少し、フォールバック回数が増加する。**新規プールでのファイル作成のバイト列を変更するか：変更する**——新規プールでのファイル作成が書き出すフィールドと幅は一つも変わらないが、ユーザーデータ分の配置先の値（t1@50180、ならびにそれを指すポインタおよび割り当てレコードエントリ内の値）は第 1 条によって確定され、[layout/01-first-txn.md](../layout/01-first-txn.md)「0. ディスクマップと新規プールでのファイル作成の書き込みマニフェスト」「5. 領域会計と割り当て」の 2 節に確定済みバイトとして登録。保留 2 点：w ≥ 2 時に各デバイス上のクラスタセグメントを整列させるか否かは確定項目 5 の実装に属する；「同一リクエストから切り出されたトランザクションは key 昇順に採番」は D16 (パブリッシュセマンティクス) 確定項目 5 に記載済みであり、ランタイムチェックは依然未済である。runs の絶対値を消費する条項は存在しない（D26 (バックグラウンド整理と配置回収) 確定項目 1 原文ママ「閾値の数値はフォーマットに入れない：E10 (エージング：構造の耐エージング性 vs 事後整理) を待つ」）。

**根拠**：

- E151 (ユーザーデータ配置先の到着順序とコンテナアーム)：候補を切り分ける量は**コミット内生ブロックがクラスタセグメントに入るか否か**である——ユーザーデータが bump セグメントに入る場合、それらの大多数はセグメントに入れず、入らない場合は 0（装置内でコミット内生ブロックの寿命は 1 チェックポイントであり 0 は下限）；確定項目 5 のユーザー決定事項の「必須」が要求した量であり、D26 (バックグラウンド整理と配置回収) 確定項目 3 の第 2 連言式が読み取る量の上流でもある。同ページはこのポリシーがブロック再書き込み負荷およびコンテナティアにおいて生じる runs コストを提示し、一様ランダム負荷では同等。
- E93 (エージング下の配置と断片化度)：最低空きスロット優先アームは一様ランダム負荷で数百チェックポイント以内に天井に達し、クラスタセグメントアームのフォールバック率は 9 割を超える——「クラスタセグメント割り当て」ポリシーは整理の上に成立する；本条は runs のために選択されたものではない。
- 2 巡の三者検証の判定は `research/prompts/d3-r2-main-verification.md`、第 1 巡および第 2 巡の経緯は [records/2026-09-10-D3项8第一轮.md](../../../records/2026-09-10-D3项8第一轮.md) および [records/2026-09-13-D3项8第二轮.md](../../../records/2026-09-13-D3项8第二轮.md) にある。
- ユーザー決定事項（3 つの選択肢の中から庚を決定）、原文は変更履歴にある。
- 2026-09-23 三者判定 `m2-placement-falsepositive-r1-main-verification.md` におけるディスク不揃い時の配置先判定枠——当時の 3 つの誤ったメンバーは空白のバージョン 1 デフォルトに対するものであり、D2 (RAID ストライプポリシー) 確定項目 2 は「確定項目 2 でカバーしない」と明記、確定項目 8 はバージョン 1 の幾何において実行可能な対象がなく、確定項目 10 は各レプリカのスロット番号が異なることを許容している。
- ユーザー決定事項 2026-09-23：クラスタセグメントがいつ無効化されるか——当日の実装に従い条項化：今回のマウントでオープンされたクラスタセグメントは閉鎖、再オープン後は混在配置許容；コードは変更せず（マイルストーン「上書き・解放・ロールバック・再利用」クローズ表第 59 行）、原文は変更履歴にある；警告あり、`.claude/warnings/2026-09-23.md` 参照。
- ユーザー決定事項 2026-10-03：ユーザーデータ配置先の値は新規プールでのファイル作成において layout 登録のバイトとして扱う、原文は変更履歴にある；問題は `research/prompts/e171-conclusion-r3-main-verification.md` 第 5 節第 1 行で提起（D3 (領域割り当て) 確定項目 8 適用範囲と `layout/01-first-txn.md`「0. ディスクマップと新規プールでのファイル作成の書き込みマニフェスト」「5. 領域会計と割り当て」の 2 節の記述が相互に矛盾）。
- ユーザー決定事項 2026-10-04：配置先制約は mkfs 水位から初めてツリーを構築したバージョンを管轄（候補甲）、原文は変更履歴にある；判定法と判定基準は `research/prompts/c673-clauses-r1-main-verification.md` 第 2 節 K2 および第 5 節第 1 行により提示（判定法 ①②、t1 = U + 4 は 6 種の幾何 × 5 種の長さにおいて 24 格全判定・全緑、攻撃側レプリカで実測）。
- ユーザー決定事項 2026-10-04：この種別の配置先判定基準をこの版の割り当てレコードから直接計算する方式（選択肢「f2：割り当てレコードから直接計算」）に変更し、U + 4 をハードコードしない、原文は変更履歴にある；判定 `research/prompts/m3-formal-code-b17-r1-main-verification.md` 第 2 節 K2（t1 = U + 4 と確定項目 8 第 1 条は回収後に両立不能：mkfs 後に k ≥ 10 回の書き込み可能マウントを行ってから最初のファイルを書き込む、または同一マウント内で 22 回以上のゼロユニットパブリッシュを実行すると、I-5.5 (mkfs 水位からツリー構築したバージョンの t1 配置先) が誤って赤判定；攻撃側レプリカで実測、メイン agent が再実行して同一結果）。
- ユーザー決定事項 2026-10-20：この種別の配置先判定基準の既知の限界は確定とする（選択肢「限界で確定」）、原文は変更履歴にある；`records/2026-10-18-里程碑三收尾与近期欠账.md` 第 323 行。

**未済の検査**：C146 (空きセグメント枯渇時のフォールバックポリシーがリポジトリ全体で未定義) 第 ① 条；C243 (停止述語の量が軌跡を持たない)。

#### 確定項目 9：偽の ENOSPC を禁止、削除された領域は有界ステップ数内に利用可能

**決定事項**：2 つの硬性要件。

1. **偽の ENOSPC を禁止**：`df` が報告する空き容量 ≥ s である限り、s バイトの書き込みは必ず成功しなければならない。ファイルシステムは内部で有界ステップ数の処理（パブリッシュの推進、ロールバック下限の引き上げ等）を行ってから割り当てを行うことができる；ディスクが真に満杯の場合（`df` 空き容量 < s）にのみ ENOSPC の報告が許容される。
2. **削除された領域は有界ステップ数内に利用可能**：s バイトのオブジェクトを削除した後、同一サイズの書き込みは有界ステップ数内に必ず成功しなければならない。この限界は計算可能な数値として記述されなければならず、「最終的には」は禁止される。**限界はユーザー可視状態を変更するパブリッシュ 3 回である**：削除されたブロックは直近 4 つの状態のうち古い 3 つによって固定されており、そのようなパブリッシュがさらに 3 回発生して初めて再割り当て可能集合に戻る；それ以前は `df` はそれを空き容量として計上せず、その状態での書き込みは `df` 上真の ENOSPC となり、第 1 条は破綻しない。ファイルシステム自身が推進する空パブリッシュはこの 3 回にはカウントしない（空パブリッシュはユーザー可視状態を変更しないため）。

判定可能な形式は不変条件 I-5.3 (報告された空き領域はすべて確実に引き渡し可能) である。**これがその場で何を変更したか**：`df` が何を報告するかはこの決定事項のもう半分である：D16 (パブリッシュセマンティクス) 確定項目 1 が定めた `df` 計測条件は控除された部分を空き容量から除外しており、第 1 条はこれによって保証されるが、代償として `df` の報告値は控えめとなり、s バイトを削除しても `df` が 1 バイトも増加しない場合がある。D16 (パブリッシュセマンティクス) 確定項目 1 の文面通りであればこの半分は現在過大報告している（判定 `research/prompts/m3-admission-r1-main-verification.md` 第 3 節「`df` の記述法」の行：D16 (パブリッシュセマンティクス) 文面通りで 73/73 過大報告）、`df` 記述法は未確定、C370 (需要・利用可能量・df に共通の単位がない) 参照。第 2 条の「有界ステップ数」は D16 (パブリッシュセマンティクス) 確定項目 1 で選択されたアームに従い計算：受入不足時は空パブリッシュを推進して F を引き上げ、1 回の受入につき最大 B = 4 + 2 k_tol 回のパブリッシュを行い、すべて実行してもなお不足する場合に初めて ENOSPC を報告する。

**適用範囲**：偽の ENOSPC および削除後の利用可能性のみを管轄し、真の ENOSPC は管轄外；クォータは含まれない（バージョン 1 ではスナップショットごとのクォータは作成しない）。実機での検証は未実施。**限界 3 は文面通り判定**：D16 (パブリッシュセマンティクス) 確定項目 1 で選択されたアームを満杯後に削除・再書き込みすると、自身の `df` 上それらは真の ENOSPC となり、本条第 2 条の文面通りには未達となる；適用範囲は k_tol = 2 回のルートスロット書き込み失敗まで。保留プールの項は定数ではない——D28 (マウント時コミット量) 確定項目 4 に従いパブリッシュごとに当時の構造から直接計算され、第 1 条の履行条件はツリー高に伴い変動する。

**根拠**：

- E139 (ディスク別ロールバック下限の厳格化形態)：満杯後に削除・再書き込みを行う失敗ウィンドウをウィンドウごとに数え上げ、「3 回」を超過した枠がどのウィンドウで成果物を持つかを特定；1 受入あたりのパブリッシュ数は実測で B を超えず、実行前保留プール下でデッドロックしない。
- ユーザー決定事項（原文ママ「我々の目標は btrfs ENOSPC の回避であり、ENOSPC は絶対に容認できない」、「第 1 条のみ」と「双方必要」の間で双方必要を選択）、原文は変更履歴にある。[pitfalls.md](../pitfalls.md) 第 1 条の症状の 2 つの半分そのものであり、確定項目 13 の「非分類」はその病根の一方のみを治癒したに過ぎない。

**未済の検査**：なし（C283 (受入失敗時にパブリッシュを先に推進せず ENOSPC を報告する) は 2026-09-26 に清算済み、未済の検査表の清算済み表参照）。

#### 確定項目 10：クラスタセグメントの定義、ユーザーデータ配置先の 4 格、コミット内生ブロックの bump 順序

**決定事項**：① クラスタセグメント長は 64 個の 16 KiB スロット；セグメント開始点 = ユニット領域内で最低位の 64 スロット整列された完全空きセグメント；bump ポインタはメモリのみに保持され、マウント後に新たにセグメントをオープン；「完全空きクラスタセグメント数」= セグメント内の 64 スロットすべてに未解放の割り当てレコードが存在しないセグメント数（会計行が記録するのはこれである；コードの現状の `empty_segments`（`crates/singlefs-core/src/allocator.rs`）は専有計測条件であり、レコード計測条件に変更する必要がある、C372 (完全空きクラスタセグメント数の 2 つの定義が相反する)）；アロケータがセグメントをオープンする際は別途隔離ビット（シャドウ会計、D19 (ブロックポインタの構造と幅の予算) 確定項目 5 硬性ルール 1）および F 引き上げ保留ビットを確認し、その量を「オープン可能クラスタセグメント」と呼ぶ（`lowest_empty_segment` はこれに基づきセグメント開始点を探索）——2 つの量は別個に記録し、相互代替は禁止（C379 (会計の完全空きクラスタセグメント数とアロケータがオープン可能なセグメントの 2 つの計測条件) で B を採用、ユーザーが 2026-09-28 に決定、1 巡の攻撃を通過、判定 `research/prompts/abandoned-floor-r1-main-verification.md` M3：自然履歴における一部隔離セグメントにより両者に 1 セグメントの差が生じる）。② ユーザーデータの候補集合に「候補配置先がいかなるオープンなクラスタセグメントにも属さない」という連言を追加。③ データユニットは最低位、開始位置が 32768 整列、両スロット空き、フルスパンで連言を通過する開始点を取得；インデックスノードはコミット内生ブロックであり、オープンクラスタセグメントから bump され（確定項目 5）、「最低位の空きスロット」はオープンセグメント内でのみ言及され、セグメント外へ探索しない。④ システム構成幾何セクションに「ユニット領域開始スロット番号」8 バイトを追加（バージョン 1 = ジャーナルリング末尾の次のスロット：mkfs がリング長から直接計算してシステム構成オフセット 417 に書き込み、開始点が 64 スロットセグメント境界上にない場合はいかなる書き込みよりも前に拒絶；リーダーがシステム構成を選択する際に 417 = 直接計算された開始点を判定し、不一致時はプール全体を拒絶；デフォルト 768 MiB リングでは 50176、実査 A3b 報告 `m2-rev-a3b-implementer-report.md`）、アロケータは下位を探索しない；固定構造は割り当てレコードを書き込まない。⑤ **コミット内生ブロックをオープンセグメントから bump する順序**：1 回のパブリッシュでインスタンス表ユニットを書き換える際（行書き込みのパブリッシュ）、**インスタンス表ユニットが最前**、複数スライス存在する場合は末尾スライスが先（第 k スライスが第 k+1 スライスの親となる、「葉が先、根が後」に従う）；残りは**ツリー ID 昇順**、ツリー内は**葉が先、根が後**、同層内は key 昇順、**中央マッピングツリーは最後から 2 番目**、**ツリーテーブルユニットが最後**；コード 3 パッキングコンテナは「データユニット」ティアに従い配置先を取得（開始位置 32768 整列、両スロット空き）。この順序は同時に誕生シーケンスの発行順序でもある（D19 (ブロックポインタの構造と幅の予算) 確定項目 9）。

⑤ 新規プールでのファイル作成における配置先（オープンセグメント [50240, 50304)、bump カーソルは 50240 から開始）：

| 順序 | ユニット | ツリー ID | 配置先 | 誕生シーケンス |
|---|---|---|---|---|
| t2 | extent ツリー根兼葉（コード 2, 16 KiB） | 65536 | 50240 | 0 |
| t3 | inode ツリー葉コンテナ（コード 3, 32 KiB, 開始位置は 32768 整列が必要 ⇒ 50241 をスキップ） | 65537 | 50242–50243 | 0 |
| t4 | inode ツリー根（コード 2） | 65537 | 50244 | 1 |
| t5 | 割り当てレコードツリー葉, デバイス 0（コード 2） | 65538 | 50245 | 0 |
| t6 | 割り当てレコードツリー葉, デバイス 1（コード 2） | 65538 | 50246 | 1 |
| t7 | 割り当てレコードツリー第 1 層ノード, デバイス 0（コード 2） | 65538 | 50247 | 2 |
| t8 | 割り当てレコードツリー第 1 層ノード, デバイス 1（コード 2） | 65538 | 50248 | 3 |
| t9 | 割り当てレコードツリー根, 第 2 層（コード 2） | 65538 | 50249 | 4 |
| t10 | 会計ツリー根兼葉（コード 2） | 65539 | 50250 | 0 |
| t11 | 中央マッピングツリー根兼葉（コード 2） | 65540 | 50251 | 0 |
| t12 | ツリーテーブルユニット（コード 2） | — | 50252 | 0 |

スロット 50241 はこの順序が残した空洞である：t2 が 50240 を占有した後カーソルは 50241 に留まるが、t3 は 32768 整列の 2 スロットを要求 ⇒ カーソルは 50242 に進み、50241 は空いたままとなる。これにより新規プールでのファイル作成の空き run 数は 3 から 4 に変化する（会計第 10 項、D5 (スナップショット / 領域会計メカニズム) 確定項目 8）。

**適用範囲**：③ において「インデックスノードは最低位の空きスロットを取得」はオープンセグメント内でのみ言及される——「ユニット領域内の最低位の空きスロット」と文字通り読むと、確定項目 5 / 確定項目 8「クラスタセグメントはコミット内生ブロックにのみ提供」と矛盾し 2 組の書き込みマニフェストが生じてしまう；バイト表および E142 (新規プールでのファイル作成のドライラン) 装置はいずれもオープンセグメント内の解を採用している。⑤ が「ツリー ID 昇順 + ツリー内は葉が先、根が後」を採用した理由：ツリー ID はリポジトリ全体で使用されているキーであり（D8 (コアインデックス構造) 確定項目 8 原文ママ「エントリはツリー ID 昇順でソート」）、葉が先、根が後は COW 本来の順序である（親ポインタは子の配置先が確定して初めて書き出し可能）；中央マッピングツリーが最後から 2 番目である理由は他者のエントリを格納するためであり、ツリーテーブルユニットが最後である理由は全ツリーのルートポインタを格納するためである。コード 3 コンテナが「データユニット」ティアを採用する根拠は、D18 (ブロックに含める情報) 確定項目 9 の整列要件がユニットサイズごとにティア分けされておりユーザーデータか否かとは無関係であるため；双方の解釈はオープンセグメント内で同一の配置先を与え、条項として確定することで C324 (コード 3 パッキングコンテナが 32768 整列を要するか否かが未記述) を解決。**新規プールでのファイル作成のバイト列を変更するか：変更する**——11 個のコミット内生ブロック（デフォルト幾何）の配置先が全体として再配置され、割り当てレコードの key、位置エントリのスロット番号、中央マッピングエントリの value がすべて連動して変化する。本項目は三者論証を経ておらず、メイン agent の推奨およびユーザーの選択はいずれも覆し可能である。⚠️ ⑤ の配置先テーブルはデフォルト幾何（768 MiB リング、4 GiB ディスク 2 台）での値である：コミット内生ブロック数は幾何に伴い変動し（リング長 1024 MiB 時は 13 個）、本テーブルの t2–t12 から逸脱して書き出されたイメージが不正となるか否かは現行バージョンでは条項がなく、判定しない（t1 のみ確定項目 8 適用範囲により判定）。

**根拠**：

- E151 (ユーザーデータ配置先の到着順序とコンテナアーム)：セグメント長 64 スロットは装置モデリング時に使用した幾何であり、恣意的な数値ではない。
- E142 (新規プールでのファイル作成のドライラン)：⑤ の配置先テーブルおよび 50241 の空洞はそれが書き出したバイト列であり、バイト表と一字一句一致する。
- ユーザー決定事項（① から ④ はマスター監査の選択肢に従い個別に決定、⑤ は別のモーダルで決定；⑤ 内のインスタンス表最前の一文は 2026-09-23 に決定）、原文は変更履歴にある。
- `research/prompts/m2-presumed-clauses-r1-main-verification.md` K1：条項がインスタンス表を名指ししない場合、3 つの候補のうち「最前」のみが ⑤ の「中央マッピングツリー最後から 2 番目、ツリーテーブルユニット最後」と衝突しない（ローカル攻撃側が 2 回一致）。K1 以前はこの順序はバイト表内の行順としてのみ存在し、条項は皆無であった——2 つの実装が各自の順序で bump すると、配置先、誕生シーケンス、マッピング key がすべて異なって書き出され、何ら警告を発しなかった。
- ユーザー決定事項 2026-09-24：インスタンス表が複数スライス存在する場合は末尾スライスが先（マイルストーン 4 が差し戻した 2 つの条項空白の 1 つ、原文は変更履歴にある）；実験なし：2 つの順序は配置先と誕生シーケンスの順序のみ異なり、測定可能なコスト差はない。
- 判定 `research/prompts/c673-clauses-r1-main-verification.md` 第 2 節 K2：⑤ のテーブルを実装現行値に従い修正（11 ユニット、ツリー ID は 65536 から開始、配置先は [layout/01-first-txn.md](../layout/01-first-txn.md)「0. ディスクマップと新規プールでのファイル作成の書き込みマニフェスト」と一致；攻撃側レプリカでコミット内生ブロックのユニット領域開始点からの相対位置 [64, 66, 68, …, 76] を読み取り）、コミット内生ブロック数は幾何に伴い変動（同一レプリカでリング長 1024 MiB 時は 13 個）；誕生シーケンスは D19 (ブロックポインタの構造と幅の予算) 確定項目 9 に従いツリーごとに 0 から起算。
- 判定 `research/prompts/m4-gc-r1-main-verification.md` 第 3 節 G4：C372 (完全空きクラスタセグメント数の 2 つの定義が相反する) をレコード計測条件で解決、① の括弧内と D26 (バックグラウンド整理と配置回収) 確定項目 1 第 1 条を同一文言に統合。

**未済の検査**：なし（C324 (コード 3 パッキングコンテナが 32768 整列を要するか否かが未記述) は ⑤ により解決済み、未済の検査表の清算済み表参照）。

#### 確定項目 11：割り当てレコード 20 バイト中のスパンセグメントは value に格納

**決定事項**：value に格納：key = (デバイス識別子 4, 16 KiB スロット番号 6) の計 10 バイト、value = (スパンセグメント 2, 割り当て世代または解放世代 8) の計 10 バイト；解放済みフラグは依然としてスパンセグメントの最上位ビットに置く。割り当てレコードツリーのコード 2 ヘッダは key 幅 10 として計算。

**適用範囲**：スパンセグメントがどちら側に属するかのみを決定し、フィールド自体の幅とセマンティクスは確定項目 7 に属する。**新規プールでのファイル作成のバイト列を変更するか：変更する**——t5 のエントリ幅フィールド、宣言長、それらレコードのレイアウト。本項目は三者論証を経ておらず、メイン agent の推奨およびユーザーの選択はいずれも覆し可能である。

**根拠**：

- E142 (新規プールでのファイル作成のドライラン)：t5 のエントリ幅、宣言長、レコードレイアウトが本項の幅に従い書き出され、バイト表と成果物が一字一句一致する。
- 確定項目 7 原文ママ「エントリは削除せず、解放済み + 解放世代に書き換える」は**インプレースでの value 書き換え**である；スパンセグメントが key に入るとフラグ反転により key が変化し、ツリー内の同一配置先に 2 件のレコードが残ってしまう。
- ユーザー決定事項、原文は変更履歴にある。

**未済の検査**：なし。

#### 確定項目 12：割り当て受入連言の唯一の登録場所、および 2 本の受入式の関係

**決定事項**：割り当て受入の連言式は確定項目 7 のテーブル 1 箇所のみに登録し全 7 条、他所はすべてそこを参照する。D16 (パブリッシュセマンティクス) 確定項目 1 の `再割り当て可能 = min(再割り当て可能 + 生存メタデータ − 保留プール, df)` と D28 (マウント時コミット量) 確定項目 1 の 9 項式は直列の 2 つのゲートを構成する：まず D28 (マウント時コミット量) の「利用可能 ≥ 需要」を通過し、次いで D16 (パブリッシュセマンティクス) の F 引き上げロジックを通過する。「需要」はディスクバイト単位で計算し各ディスク同一（D5 (スナップショット / 領域会計メカニズム) 確定項目 7 の「割り当て済み」ディスク別物理バイトと同一計測条件）。

**適用範囲**：登録場所と 2 式の順序を決定し、いかなる連言条項の内容も変更しない。**新規プールでのファイル作成のバイト列を変更するか：変更しない**——新規プールでのファイル作成は常に受入を通過し、本項は受入判定自体を制御する。本項目は三者論証を経ておらず、メイン agent の推奨およびユーザーの選択はいずれも覆し可能である。

**根拠**：実験なし：1 箇所への集約は `.claude/singlefs-ai-sop/rules/kb-discipline.md` 第 4 条の直接の要求であり（集約前は連言式に 4 箇所の相互に異なる記述が存在）、測定可能な量は存在しない；2 つのゲートの解釈は D16 (パブリッシュセマンティクス) 確定項目 1「受入不足時は空パブリッシュを推進して F を引き上げる」の文言と整合。ユーザー決定事項、原文は変更履歴にある；「需要」はディスクバイト単位で各ディスク同一：ユーザー決定事項（原文は変更履歴にある）。

**未済の検査**：なし。

#### 確定項目 13：data / metadata の 2 つの領域種別に分類しない

**決定事項**：data / metadata の 2 つの領域種別に分類せず、グローバルに統一されたアロケータを使用する。

**適用範囲**：アロケータが第 2 の分類軸を持つか否かを管轄し、配置先ポリシー（確定項目 8 / 10）、デバイス次元（D2 (RAID ストライプポリシー) 確定項目 8）、ツリーのノードと葉を分離して配置するか否かは管轄外。確定項目 14 の付帯コミットメントを直接導出する：非分類は断片化圧力が単一のアロケータに集中することを意味する。D26 (バックグラウンド整理と配置回収) 確定項目 3 の最低割当額「小さいほどよい」という計測条件もその上に構築されている——非分類後は「解放に領域申請が必要」であることが btrfs ほど致命的ではなくなる。

**根拠**：実験なし：本セルは第 2 の分類軸を設けないことを決定しており、測定可能な量は存在しない。病根は [pitfalls.md](../pitfalls.md) 第 1 条の症状である——btrfs の 2 段階割り当てと chunk 分類の硬直化により、`df` に数十 GB の空きがあっても領域不足を報告し、ファイル削除すら領域不足となる；`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「他プロジェクトの手法は手がかりであり証拠ではない」に従い、反証およびメカニズムとしてのみ扱い、正の証拠とはしない。ユーザー決定事項、原文は変更履歴にある。

**未済の検査**：なし。

#### 確定項目 14：非分類に伴う付帯コミットメント——バックグラウンド整理は常駐・自動・ユーザー透過

**決定事項**：確定項目 13 の非分類を選択したことは、バックグラウンド整理が**常駐・自動・ユーザー透過**であることをコミットしたことを意味する——btrfs が手動で `balance` を実行して回復を試みる運用は最大の悪名の 1 つである。このコミットメントの引き受け先は D26 (バックグラウンド整理と配置回収) である：メカニズムは同項目の確定項目 1 / 2 / 3 にあり、トリガー判定、停止述語、常駐最低割当額はそこに置かれ、未確定項目もそこに紐付けられている。

**適用範囲**：「ユーザーの手を煩わせない」ことをコミットし、整理がどのアルゴリズムを使用するか、閾値をいくらにするかはコミットしない（数値は E10 (エージング：構造の耐エージング性 vs 事後整理) を待つ）。2 つの例外を連動して引用する必要がある：暗号化有効時は本コミットメントは成立しない（確定項目 16）；D26 (バックグラウンド整理と配置回収) 確定項目 7 が回収を 2 本の線に分割した後、ディスクへの引き渡し線を選択した場合に脚 1 でどのタスクが停止するかは未確定であり（D26 (バックグラウンド整理と配置回収) 未確定項目 9）、その問いは確定項目 5 と共に回答される必要がある。

**根拠**：

- E93 (エージング下の配置と断片化度)：いかなる配置も行わないアームは一様ランダム負荷で数百チェックポイント以内に天井に達し、緩やかな劣化ではない；クラスタセグメントアームのフォールバック率は 9 割を超え、「クラスタセグメント割り当て」ポリシーは整理の上に成立する ⇒ 常駐整理が空きセグメントを供給しなければ、配置先ポリシー自体が成立しない。
- ユーザー決定事項、原文は変更履歴にある。

**未済の検査**：C243 (停止述語の量が軌跡を持たない)。

#### 確定項目 15：領域予約は受入制御の形態を採用

**決定事項**：領域予約は受入制御の形態を採用し、量を小さく抑える。予約は「ある区画を囲い込んで使用不可にする」ことではなく：領域専有を増加させるいかなる操作も、進入前に「**最悪ケースで自身を巻き戻すために必要な量**」を計算し、成立しなければ拒絶する。これにより予約量は実際の需要に動的に追従し、無駄な占有を防止する；量は小さくなければならない。実行可能な形式は D28 (マウント時コミット量) 確定項目 1 の 9 項式であり、読み取り値の合成法は D28 (マウント時コミット量) 確定項目 2、登録場所は確定項目 7 の 7 条連言テーブルである。

**適用範囲**：形態と方向性（小容量、需要追従）を決定し、いかなる具体的数値も決定しない——各項の計算量は D28 (マウント時コミット量)、整理の常駐最低割当額は D26 (バックグラウンド整理と配置回収) 確定項目 3 にある。「自身を巻き戻すために必要な量」に含まれる内訳は確定項目 2 により拡充され（墓石、livelist の FREE エントリ）、1 項目拡充されるごとに「無駄に占有しない」制約が一段厳格化される。

**根拠**：

- E82 (受入のインフライト合成)：3 種の直感的記述はそれぞれ数え上げ可能な違反を生じる——パブリッシュ済み値のみを読み取ると同ウィンドウで過剰販売となり、ウィンドウ内の解放を利用可能に計上すると defer が保留すべきブロックを消費し、墓石を忘れるとディスク満杯後の削除が全件デッドロックする ⇒ 受入制御という形態はインフライト合成と組み合わせて初めて成立し、領域の囲い込みでは成立しない。
- ユーザー決定事項、原文は変更履歴にある。

**未済の検査**：C87 (受入が読み取る合成値が未定義)。

#### 確定項目 16：暗号化有効時に鍵のない側は読み取りと検出のみ実施

**決定事項**：暗号化有効時、鍵のない側は**読み取りと検出のみ実施し、移動も回収も行わない**。

- **移動不能**：4 つの確定条項が同時に真になり得ない——ユニットを移動することは新たなストライプを発行することであり（D2 (RAID ストライプポリシー) 確定項目 6 原文ママ「`w ≥ 2` は硬い下限（冗長度ゼロのストライプは発行禁止）」）、新たなストライプの各スロットはメンバーレコードを必要とし（D2 (RAID ストライプポリシー) 確定項目 12）、メンバー表は原文ママ「**ユニットのペイロード内に常駐しなければならない**」とされ、ペイロードは暗号文である（D9 (暗号化) 確定項目 11「システム構成を除きすべて暗号化、メタデータは最小限の平文ヘッダを保持」）⇒ 鍵のない側はメンバー表を書き込めない ⇒ 移動は成立しない。`w ≥ 2` は硬い下限 ⇒ 本条は `w ≥ 3` のプールのみならず**すべての**プールに成立する。
- **回収不能**：会計は権威状態であり（D21 (権威状態と派生状態の境界)）かつ暗号化されており、鍵のない側は書き込み不能 ⇒ ディスクロック期間中に「領域を割り当て可能とマークする」ことは不可能。
- **検出は常に実施可能**（ポインタ内に鍵不要の暗号文チェックサムを保持）——これが鍵のない側に残された唯一の能力である。

**適用範囲**：代償を明記し連動引用する必要がある——確定項目 14 の「常駐・自動・ユーザー透過のバックグラウンド整理」は暗号化有効時には**成立せず**、ディスクロック期間中は整理と疎開がすべて停止し、ほぼ満杯のボリュームにおけるディスク交換および縮小も同様に実行不能となる。これは承知の上での受容であり、見落としではない。暗号化は最初の実行可能バージョンには含めない（D9 (暗号化) 确定項目 10）⇒ バージョン 1 には実行パスが存在しない。

**根拠**：実験なし：トレードオフの選好変更ではなく、「移動可能」側が 4 つの確定条項により到達不能と証明されたためであり、比較すべき測定可能なコストは存在しない。ユーザー決定事項（従前の「移動可能」を撤回）、原文は変更履歴にある；権威記録は D9 (暗号化) 確定項目 5 にある。

**未済の検査**：C167 (平文マッピング層参照の全リポジトリリスキャンが未実施)。

#### 確定項目 17：領域解放操作自体は領域の申請を不要とする

**決定事項**：COW システムは「領域解放」操作自体が領域の申請を不要とすることを保証しなければならない。btrfs は設計レベルでこれを保証せず、多数のヒューリスティクスで十数年間パッチを当て続けてもなお問題を抱えている。解放パスで書き込みを要するバイト（墓石ユニット、livelist の FREE エントリ）は、書き込みの瞬間に確定項目 15 の受入制御に従い領域が前払いされる（確定項目 2）。

**適用範囲**：D3 (領域割り当て) において最も強固な条項であり、他の全項目がこれに準拠する：確定項目 2 はこれを破綻させずに「削除がディスク書き込みを要する」仕様をどう収容するかを定め、確定項目 9 第 2 条は削除された領域がどれだけの時間で復帰するかを定める。「解放が必ず即座に成功する」ことをコミットするものではない——defer ウィンドウおよびロールバック下限によって固定された世代は D16 (パブリッシュセマンティクス) 確定項目 1 の計測条件に基づき空き容量に計上されず、それは真の ENOSPC であり例外破綻ではない。削除系パブリッシュが式判定をスキップする仕様（確定項目 2）はバージョン 1 における「領域解放は領域申請を不要とする」の具現化である；墓石ツリーのルート満杯時の層拡張（D8 (コアインデックス構造) 確定項目 6）により、構造起因で削除が拒絶されることはない。

**根拠**：

- E130 (ヘッドあたり 1 部の livelist は有界破棄を通過するか)：候補甲においてブロック解放ごとに書き増す数バイトが本原則に真っ向から衝突し、「バッチごとの最悪領域需要が有界でありバッチ前に予約可能」という判定基準の実行可能な解釈を要請した。
- E82 (受入のインフライト合成)：墓石の支払いを忘れたアームはディスク満杯後の削除が全件デッドロックする——本原則が破綻した際の現象は数え上げ可能である。
- 削除系パブリッシュが式判定をスキップ：`research/prompts/m3-delete-r3-main-verification.md` 第 3 節 G1-削除系パブリッシュ判定法の行；墓石ツリーのルート満杯時の層拡張：同判定第 3 節 G6-墓石ツリールート満杯の行およびユーザー決定事項 K-R3-3「ルート拡張書き込み（推奨）」、原文は変更履歴にある。
- ユーザー決定事項、原文は変更履歴にある。

**未済の検査**：C260 (livelist エントリの前払いが自身を巻き戻す量に未算入)。

````

**出处 `.claude/kb/decisions/03-領域割り当て.ja.md:335-338`（整段抄，未转述）**

```markdown
## 過去バージョン

D3 (領域割り当て) の履歴エントリは [decisions-history.md](../decisions-history.md) に集約されている。

```

**出处 `.claude/kb/decisions/03-空间分配.md:1-6`（整段抄，未转述）**

```markdown
## D3 空间分配 —— 已定

**中文** · [English](03-space-allocation.en.md) · [日本語](03-領域割り当て.ja.md)

D3（空间分配） 管分配器这一格：空间分不分类、预留取什么形态、分配记录的粒度与载体与盘上编码、用户数据与提交内生块的落点、聚簇段的几何、分配准入合取的唯一登记位，以及「不许有假性 ENOSPC、删掉的空间在有界步数内可用」这条硬要求。不管的：准入不等式的权威形式与准入的在飞合成挪去了 D28（挂载期承诺量） 已定项 1 / 2，已定项 4 / 6 只留指针、编号不动；整理的触发判据、停机谓词与常驻底额在 D26（后台整理与放置回收）；一次写落哪几块设备在 D2（RAID 条带策略） 已定项 8；`df` 报什么与抬 F 那道闸在 D16（发布语义） 已定项 1；快照侧的 deadlist 纪律在 D5（快照 / 空间记账机制）。

```

**出处 `.claude/kb/decisions/03-空间分配.md:31-51`（整段抄，未转述）**

```markdown
#### 已定项 1：分配记录逐落点答得出「属不属于现行版本」

**定案**：分配记录是「某个落点此刻属不属于现行版本」这个谓词的权威记录点，不是聚合量——D5（快照 / 空间记账机制） 已定项 3 那棵记账树装的是「(统计量, 维度元组, 代) → 完整值」，它答不了单个落点。谓词由两半合成，两半都已定：

| 半 | 谁给 | 说的是 |
|---|---|---|
| 这个落点分配出去了吗 | **D3（空间分配）** 的分配记录 | 空闲的落点不属于任何版本 |
| 分出去的属不属于**现行**版本 | **D5（快照 / 空间记账机制）** 的 deadlist | 进了 deadlist 的落点只被快照引用，不在活树里 |

「权威记录点」是运行时口径，不是态别：分配记录**是派生态**，整棵树可从单元 + 记账 + 根重建，权威态仍是那三样、不扩项（权威记录在 D21（权威态与派生态的分界） 已定项 5）；运行时判这个谓词只许问它、不许现扫，而它自己丢了能重建。

**射程**：管运行时逐落点判现行版本。不管扫描重建时多版单元的现行版本判定——那一格按 C113（扫描重建时多版单元的现行版本判定无输入） 定案收成「单元写序 + 实例表的已发布谓词 + 祖先表」，读不到这个谓词；D11（索引节点要不要留消息缓冲区） 已定项 4 的重算源因此也不含分配记录。整理**策略**本身不在射程里（D26（后台整理与放置回收） 的分项），量出来的是载体强制付出多少。

**依据**：

- E59（缓冲消息能不能只从单元重算）：只扫单元时同一个 key 的多份可读副本分不出现行值（分不出的数恰好等于覆写次数），加上这个谓词之后四类消息的分歧全部归零——逐落点这一半买到的就是这个。
- E74（分配记录的条目数与整理开销）：逐落点的条目数随容量走、与记账树的量纲不同，两棵树都没建错；整理路径里没有随盘容量走的项，而 COW 重写的是整个节点、落点在 key 空间里连续还是打散差几百倍——已定项 3 那条 key 把整理放在好的那一端。
- 用户定案，原话在变更史。

**欠**：C122（分配记录不点删之后的条目数口径）；C412（整理的挂钟代价没量过）。

```

**出处 `.claude/kb/decisions/03-空间分配.md:52-70`（整段抄，未转述）**

```markdown
#### 已定项 2：墓碑与 livelist 的 FREE 条目由准入控制算进最坏情况

**定案**：删除这个操作本身要写盘——D18（块里携带什么信息） 已定项 8 定「删除要写墓碑单元」，可写头里删块时还要往 livelist 追加一条 FREE 条目（36 字节，D6（快照实现模型） 已定项 2），而已定项 17 的原则逐字写着「COW 系统必须保证『释放空间』这个操作本身不需要申请空间」。解法不是给删除路径开口子，是把那一笔提前到写入那一刻收掉：已定项 15 的准入控制本来就要求「任何增加空间占用的操作，进门前先算**最坏情况下解开自己需要多少**」——**墓碑就是「解开自己」的一部分**，计入那个量；livelist 的 FREE 条目同样是「解开自己」的一部分，计入那个量、写入时预付。墓碑那一笔按对象记：每个活对象（nlink ≥ 1）一份墓碑记录的摊分（含墓碑树内部节点的摊分）记进已承诺预留（D5（快照 / 空间记账机制） 已定项 4 第 6 项，记账标签 6），另加全池常驻的槽。精确式子：标签 6 = 副本数 × ((2 + H_墓碑 + 1 + 2 + H_退役 + 1) × 16384 + 41 × n_live + 35 × n_记录)，单位字节、按副本之和记（C375（待删占用与已承诺预留按份还是按和记没有条款））；H_墓碑 是现行那一版墓碑树的内部层数、H_退役 是退役树的（都是零根与只有根时 0），各从自己的树根节点码 2 头现读（先例：D28（挂载期承诺量） 已定项 4 的 ckpt_cost 读法）；41 = ⌈2 × 16384 ÷ 815⌉，每份副本、每个对象先取整再乘 n_live；n_live = 这一版 inode 树里 nlink ≥ 1 的记录数；35 = ⌈32768 ÷ 959⌉，n_记录 = 这一版 inode 树里全部记录数（含 nlink = 0 的）：退役项数 n_记录 不数 n_live，因为退役记录写在最后一批，意图到最后一批之间那个对象已不在 n_live 里，退役份额挂在 n_live 上就在用到之前释放了；树表里没有 `deleted_inodes` 条目的版本，标签 6 取 0。删除各批的新写由准入式子里的常驻扣项「环窗口 = (根环槽数 3 × S + 1) × 每批上界 W」兜（D28（挂载期承诺量） 已定项 1，S 是系统配置每区槽数），不进记账行。删除类发布（意图、各批、挂载里的续做批）不判式子那一判，落点那一判照判；写入类发布按发布之后的标签判；环窗口从第一个文件版本那次发布起计。⇒ 删除路径仍然不申请空间：已定项 17 的原则一个字不改，改的是「解开自己需要多少」这个量的算法。

**射程**：定的是这笔账归谁付（准入控制，写入时付），不是这笔账是多少——第一版一次 unlink 写一条 40 字节整对象墓碑（D18（块里携带什么信息） 已定项 24）；能不能批量摊、盘满时删得动删不动还没有数；删除路径已有实现（`crates/singlefs-core/src/unlink.rs`），要等 E168（删除借保留量的环窗口） 第二段在真实现的删除批上出量才算得出来（第一段 2026-10-03 在套补丁表的副本上跑过，第二段还没跑）。它把已定项 15「预留量动态贴着实际需求，不白占」变紧了：预留量与「将来会不会被删」绑在一起，墓碑那一项对永不删除的对象是纯浪费，浪费多少同样没量。删除类发布不判式子被攻过一轮；写入类发布按发布之后的标签判、环窗口从第一个文件版本起计被攻过零轮；标签 6 的式子里常驻槽数取 2 + H_墓碑 + 1 现读这一写法被攻过零轮；退役那一项（2 + H_退役 + 1、35 × n_记录）与 D28（挂载期承诺量） 已定项 4 的退役容器一项合起来这一形被攻过零轮，「删 → 回退」环下它们合起来仍超，记在 C617（删后回退留下的墓碑没有预付罩着）。

**依据**：

- E82（准入的在飞合成）：忘掉墓碑那一笔时盘满后删除全数死锁——「解开自己」含墓碑是可执行的，不是修辞。
- E130（每头一份 livelist 过不过有界销毁）：每释放一个块要多写的那几个字节把 livelist 追加抬成「解开自己」的一部分，不预付就与已定项 17 的原则冲突。
- 用户定案（墓碑那一半与后加的 livelist 那一半），原话在变更史。
- 墓碑按活对象摊分加全池常驻一份、删除类发布不判式子、写入类发布按发布之后的标签判、环窗口从第一个文件版本起计：`research/prompts/m3-delete-r3-main-verification.md` 第三节 G1-墓碑预付、G1-删除类发布怎么判、G1-写入类发布、G1-窗口起计四行。
- 环窗口是准入式子里的常驻扣项、不进已承诺预留：用户定案 K-U4「全池一份环窗口」与 K-R3-2「常驻扣项（推荐）」，K-R3-2 改了 K-U4 的字面，两条原话在变更史。
- 标签 6 的精确式子（常驻槽数 2 + H_墓碑 + 1 现读、41 × n_live、按副本之和记）：`research/prompts/m3-blanks-r1-main-verification.md` 第二节墓碑预付那一行；用户定案 2026-10-01「乙 2 + 层数 + 1，现读（推荐）」，原话在变更史。
- 标签 6 加退役那一项：判决 `research/prompts/m3-retire-r1-main-verification.md` 第二节 K4 的 P-a、P-b 两行（照谱系在 ckpt_cost 里单列一项，单做不回退也超卖，第一次抽样坐实、主 agent 复跑重现；墓碑式预付采纳）与第三节第 6 条；用户定案「两样都做（推荐）」，两次弹窗的原话在变更史。
- 环窗口的根数读根环槽数 3 × S：同一份判决第二节根环里的根数那一行（一轮）。

**欠**：C260（livelist 条目的预付没算进解开自己那个量）。

```

**出处 `.claude/kb/decisions/03-空间分配.md:71-83`（整段抄，未转述）**

```markdown
#### 已定项 3：分配记录住一棵独立 keyspace 的 btree

**定案**：载体是独立 keyspace 的 btree，key = 落点（设备身份 + 设备内偏移），value = 分配代。形态照 D5（快照 / 空间记账机制） 已定项 3 的记账树，同一套 btree 机制，不新开第二种结构；key 用落点是已定项 1「逐落点」的直接后果，`(dev, 偏移)` 与 D19（块指针的结构与宽度预算） 已定项 1 的位置条目同一个坐标系，查一个落点是点查不是扫描；加一棵 keyspace 不是 incompat 改动——D15（格式冻结政策） 已定项 2 定树表 day-1 做成开放列表，判定表把「新增一棵 keyspace」钉死为 compat。先按这个形态落地，写代码时再调整。

**射程**：载体定了之后的代价（条目数、树高、整理放大按 key 空间连不连续分出两端）由 E74（分配记录的条目数与整理开销） 量，归已定项 1。定的是载体，不是粒度：量出来若不可接受，重开的是已定项 1 的粒度，不是这一项的载体。条目的盘上编码与宽度归已定项 7，跨度段归 key 还是 value 归已定项 11。

**依据**：

- E97（记账与分配记录的条目编码）：同一坐标系下的编码与条目宽由它判，落点按 16 KiB 槽号编码时非法落点归零，坐标系必须是 16 KiB 槽号。
- 用户定案，原话在变更史。

**欠**：C122（分配记录不点删之后的条目数口径）。

```

**出处 `.claude/kb/decisions/03-空间分配.md:84-97`（整段抄，未转述）**

````markdown
#### 已定项 4：准入不等式的权威形式

**定案**：权威正文在 D28（挂载期承诺量） 已定项 1，引这一项时引那一处，别引这里。结论句：

```
可用(d) = 容量(d) − 已分配(d) − 不可回收(d) − 挂载期承诺量(d) − 被抛弃根独占量(d) − 待删占用 ÷ 副本数 − 已承诺预留 ÷ 副本数 − checkpoint 保留池 ÷ 副本数 − 环窗口(d)
```

**射程**：这一格只留指针与结论句；式子逐项的口径、按不按设备求和、每一项为什么在式子里，全在 D28（挂载期承诺量） 已定项 1。编号不重排，所以这一节留着。

**依据**：无实验：这一格只是指针，权威正文与全部依据都在 D28（挂载期承诺量） 已定项 1。

**欠**：无。

````

**出处 `.claude/kb/decisions/03-空间分配.md:98-112`（整段抄，未转述）**

```markdown
#### 已定项 5：提交内生块从聚簇段分配

**定案**：提交自身产生的块——分配记录树、记账树等结构的 COW 新节点——必须从 key 空间上连续的聚簇段（bump 段）分配；用户数据块的落点不受此约束。段的几何与 bump 次序在已定项 10，段耗尽时的回落在已定项 8 第 2 条。提交内生块在固定点分配时找不到空槽（`NoFreeSlotOnAnyDevice`），在任何写之前拒绝这次发布，盘上不变；这一格兜住准入式子罩不住的分裂与聚簇段层（C545（空间准入罩不住分裂与聚簇段层））。

**射程**：它是**分配器政策不是格式位**——聚簇段的位置与长度是运行时选择，盘上没有任何字段声明它；违反它的实现写出的镜像完全合法，只是每次发布贵几百倍。⇒ 拦它只能靠运行时读数：提交路径记「本次发布的固定点收敛轮数」，超过声明上界判红（`.claude/rules/fs-design.md` 硬要求 4：走错分支是性能悬崖，悬崖不报错）。w ≥ 2 时各设备上的聚簇段要不要对齐（每个单元 w 份副本各整落一列 ⇒ w 个聚簇段各自推进）全仓没有条款，归这一项的实现。

**依据**：

- E148（提交固定点按两棵记录树重算）：两棵记录树下聚簇提交内生块仍给常数固定点，全散布时没有上界——常数上界只在聚簇政策下存在。
- E81（提交的固定点）：全部记账结构都是 COW 树，给它们写节点又产生分配，固定点循环的放大由提交内生块聚不聚簇决定——全散布时一次 fsync 拖出的元数据块数让 I-8.1（环几何够大） 的「最坏事务占用」没有可用上界，只聚簇提交内生块就断链，用户数据全随机也不影响。它给的是下界（假设发布末尾一次性 COW），边插边 COW 的实现更贵。
- 用户定案，原话在变更史。
- `research/prompts/c363b-r1-main-verification.md` 第四节第 1 条，主 agent 定，被攻过零轮。

**欠**：C83（提交固定点没人回答）。

```

**出处 `.claude/kb/decisions/03-空间分配.md:113-122`（整段抄，未转述）**

```markdown
#### 已定项 6：准入的在飞合成

**定案**：权威正文在 D28（挂载期承诺量） 已定项 2，引这一项时引那一处，别引这里。结论句：准入读数 = 已发布统计量 − 在飞已批准的预留；窗口内产生的释放不发信用；「最坏情况下解开自己需要多少」含墓碑那一笔。

**射程**：这一格只留指针与结论句；在飞 overlay 的态别、实例切换重建内存态时 overlay 怎么重算，都在 D28（挂载期承诺量） 已定项 2。编号不重排，所以这一节留着。

**依据**：无实验：这一格只是指针，权威正文与全部依据都在 D28（挂载期承诺量） 已定项 2。

**欠**：无。

```

**出处 `.claude/kb/decisions/03-空间分配.md:123-162`（整段抄，未转述）**

```markdown
#### 已定项 7：分配记录条目的盘上编码，与分配准入合取的登记表

**定案**：落点粒度取 16384 字节（一个 16 KiB 槽）；一条记录记一个存储单元。key = (设备身份 4, 16 KiB 槽号 6) 共 10 字节；value = (跨度段 2 含已释放标志, 分配代或释放代 8) 共 10 字节，代存完整 checkpoint 号。

| 段 | 宽 | 归哪边 | 依据 |
|---|---|---|---|
| 设备身份 | **4** | key | D19（块指针的结构与宽度预算） 已定项 4。**同一个坐标系**，不许各定各的（已定项 3 依据第 2 条逐字）|
| 落点 | **6**，编码为 **16 KiB 槽号** | key | 粒度由 D18（块里携带什么信息） 已定项 9 逼出来（索引节点对齐 16384、大小就是 16 KiB）；E97（记账与分配记录的条目编码） 判据 7 实测按 32 KiB 单元号编码时一半的槽表达不了、按 16384 编码时零个。6 字节槽号寻址 4 EiB / 设备 |
| 跨度段（含已释放标志位） | **2（初值）** | value（已定项 11） | 一条记录记一个单元，跨度段说它覆盖几个槽。宽度按 D19（块指针的结构与宽度预算） 已定项 3 的「定宽留位」政策取，**理由是政策不是用量**——第一版恒为 1 或 2；**最高位借作已释放标志**（0 = 仍分配、1 = 已释放），不占表达跨度值的位 |
| 分配代 / 释放代 | **8** | value | 与 D22（单元原子性怎么合成） 已定项 7 的 `checkpoint_txg` 同宽同值域，语义按跨度段的标志位取其一。E97（记账与分配记录的条目编码） 判据 3 / 8 实测：任何有限模都只是把破 key 序推迟到回绕那一刻 |

**落点释放时条目不删、不点删**（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P4）：改写成「已释放 + 释放代」——释放代写进那 8 字节，已释放标志位借跨度段的最高位，条目留到该落点被重新分配时覆盖。不点删的理由：树按落点排序、代住 value，按代点删要扫全树，而点删所需的按代清单正是崩溃会丢的那份内存态；条目改写与产生这次释放的 COW 在同一次发布里原子生效——分配记录是根下的 COW 树（D22（单元原子性怎么合成） 已定项 3），条目与根同一次发布，释放代 = 那次发布的 checkpoint_txg；崩溃丢掉的是没发布的那个 checkpoint，它里面的释放一条都不留痕、对应的旧版本仍被所选根引用。崩溃后 defer 队列的内存态丢失，接手的是「释放代 ≤ max(F_生效, 环里最旧有效根)」这条闸（I-7.4（近 K 代块未被复用） 的注；谓词由 D16（发布语义） 已定项 1 给，有效性按实例表判；抬 F 的机制押在这个谓词上）。「分配代」的口径：搬迁的目的地那条记录写单元的诞生代，不写搬迁那次发布的 txg（D26（后台整理与放置回收） 已定项 5）。

**「释放」的口径** = 分配器把落点放进 defer 队列那一刻 = 没有任何根（含快照）引用它；被快照引用的落点按 D5（快照 / 空间记账机制） 的 deadlist 纪律进 deadlist 不进 defer、快照销毁时才释放 ⇒ 释放代晚于最后一个引用它的根被销毁的那次发布。**清扫准入**两支共有的合取 = 不在分配器在飞 overlay 里 ∧ 头可读；有条目的再要求已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)（D16（发布语义） 已定项 1 的谓词；由释放的口径，此时环里没有根还引用它；单元属于哪个实例无关，当前实例自己释放的落点也要能清）；孤儿（没有条目）要求头里的写序实例 < 当前实例 ∧ 按实例表的已发布谓词判未发布（当前实例没有行，它的孤儿只能是在飞的写，交给 overlay 那一条）。

**分配准入的合取式，唯一登记位就是这张表**（已定项 12 指到这里），共七条，全部为真才许分配：

| # | 合取 | 出处与射程 |
|---|---|---|
| 1 | `可用 ≥ 需求` | D28（挂载期承诺量） 已定项 1 的九项式子 |
| 2 | 落点谓词 | 已定项 1 的「属不属于现行版本」，输入是分配记录 + deadlist |
| 3 | 候选落点不在任何活让渡区里 | D9（加密） 已定项 7 那次重开把 `\|A\|` 从「已分配」改记「已承诺预留」，而那个量不带设备维、不带树维（D5（快照 / 空间记账机制） 已定项 4 第 6 项），A 的槽在分配记录树里不再有任何一位表达 ⇒ 密钥侧分配器可以把它们再分配一次；命题 5(b) 逐字「与 I-3.1（已分配统计对得上） / I-5.1（物理范围不重叠） 共用同一趟全盘遍历」= checker，不是运行时闸。形态照 D26（后台整理与放置回收） 已定项 1 第 3 条对 `R` 的写法：靠读一个有界的活对象集合推出来，不靠盘上多一位；活让渡区同时至多一个 ⇒ O(1) 点查不是遍历。⚠️ D9（加密） 已定项 7 随已定项 5 改取「只做 scrub」而撤回（让渡区 A 无对象）⇒ 这个合取恒真，条款原样保留 |
| 4 | 条带表[候选落点] 里没有活成员 | 只对 w ≥ 3 的条带起作用：D2（RAID 条带策略） 已定项 11 规则 1 逐字「w ≥ 3 条带的成员被释放后，该格与整条带（含 parity）在修复完成前不得复用」；w = 2 的镜像没有 parity、两份一起释放，这一条恒真。代价是每次分配多一次点查（条带表按落点排，与分配记录树同一个坐标系，D2（RAID 条带策略） 已定项 14），点查的执行路径押在条带成员表的载体上（D2（RAID 条带策略） 已定项 12）|
| 5 | 不在 D26（后台整理与放置回收） 已定项 1 保护期内的 `R` | 整理意图期内那批槽不许当分配目的地 |
| 6 | 候选落点不在任何开放的聚簇段里（只管用户数据）| 已定项 10 ②；提交内生块正是从开放段 bump，不受这一条约束 |
| 7 | `全空聚簇段数 ≥ N₀` | D26（后台整理与放置回收） 已定项 3 的第二合取式；`N₀` 第一版取 1，算式 `⌈(3S − 1) × c ÷ 64⌉ + 1`（S = 每区根槽数，c = 一次发布换下的提交内生块数，挂载时算一次）等计数模型验过再定（E10（老化：结构抗老化 vs 事后整理））；**只拦前台写入类发布**：删除类、整理批、挂载与回退的发布免这一条（用户 2026-10-13 定「只拦前台写入类发布」）|

**一条记录恰好覆盖一个单元是分配器的不变量，不是编码强制的**：跨度 1 必须永远合法（索引节点就是一个 16 KiB 槽），所以 `(偶数槽, 跨度 1)` 这条记录永远写得出来。先按这个形态落地，写代码时再调整。

**射程**：占容量那个倍数不是这一项的判据——`.claude/rules/fs-design.md` 明令「比谁省不构成任何判据」，E97（记账与分配记录的条目编码） 把两条候选压到只剩这一个差别之后自己判不出高下，这一格是用户定的。条目数上界随「不点删」从「已分配落点数」变成「曾分配过、尚未被覆盖的落点数」，仍以容量 / 16 KiB 为界；E74（分配记录的条目数与整理开销） 判据 1「条目数恰好等于已分配落点数」在这个新口径下按定义为假，改口径之后要与 E97（记账与分配记录的条目编码） 一起重跑取新的绝对值，未跑。别处把这条条目形态引成「一条记一个单元、另带跨度段」的，取的是已定项 11 之前那版写法——「一条记录记一个存储单元」这一半照旧，跨度段则住 value。落点粒度这个数同时进两层冻结（位置条目在 D15（格式冻结政策） 已定项 7 的第 2 层、分配 keyspace 的 key 在第 3 层，而第 2 层要先于第 3 层冻）⇒ 它要当第 2 层的问题看；异构池（D12（目标介质） 已定项 3）让它不能挂在某一条布局线的 incompat 位上——一个布局 A 的节点里的位置条目会指向布局 B 的设备，这一格仍欠处置。合取 3 与合取 4 都不改新池新建文件写出的字节（第一版不加密、两块盘 w = 2，两条恒真）。

**依据**：

- E97（记账与分配记录的条目编码）：落点按 16 KiB 槽号编码时全部合法起点都表达得了、非法落点归零，代按有限模编码只是把破 key 序推迟到回绕；同一档上的条目数、树高与占容量逐条按条目宽线性，两条候选压到只剩占容量一个差别。
- E44（序号位宽的本机实测与代价）：本机的发布速率给出「1 字节的代多久绕完一圈」，代取 8 字节不是拍的。
- 用户定案（编码形态、第三个合取、第四个合取），原话在变更史；C113（扫描重建时多版单元的现行版本判定无输入） 定案 P4 给「不点删」。
- 判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G4（合取 7 只拦前台写入类发布：攻方模型上只拦前台的臂冻结 0、全拦的臂冻结 2480 态；`N₀` 的算式先建计数模型验，验之前取 1）、G3（搬迁目的地记录写单元的诞生代）与第七节用户定案「只拦前台写入类发布」，原话在变更史。

**欠**：C122（分配记录不点删之后的条目数口径）；C127（钉住状态在分配记录里没有落点）；C411（分配记录的跨度段与单元类别不符没检查）。

```

**出处 `.claude/kb/decisions/03-空间分配.md:163-191`（整段抄，未转述）**

```markdown
#### 已定项 8：用户数据块不进聚簇段，落点是已选设备内起点最小的空槽

**定案**：五条。

1. **第一版用户数据块不进聚簇段。落点**：设备集合由 D2（RAID 条带策略） 已定项 8 选定之后，在每一块被选中的设备上各自取该设备内**起点槽号最小、起点 32768 对齐、跨度内两槽都空、不在任何开放的聚簇段里、整跨度都过分配准入合取（七条，唯一登记位是已定项 7）的落点**；**不选设备、不轮转设备**（设备维归 D2（RAID 条带策略） 已定项 8）；保护期内的 `R`（D26（后台整理与放置回收） 已定项 1 第 3 条）已经是那七条合取里的一条，不另列。
2. **聚簇段在挂载期间对用户数据关着**：这次挂载开过的聚簇段（当前开放的那一段，以及回落之前开过的段）都不给用户数据（已定项 10 ② 的「任何开放的聚簇段」指的就是这些段）；重开之后（bump 指针与「这次挂载开过哪些段」都只住内存，挂载后新开一段，已定项 10 ①），上一次挂载开过、装着提交内生块的段对用户数据开放，混放允许。已定项 5「用户数据块的落点不受此约束」原样成立、不推广；提交内生块的段耗尽时回落到该设备内槽号最小的空槽，同样排除 `R`——C146（无空段时的回落政策全仓无定义） 第 ② 条（回落落到哪）由此有了政策，第 ① 条（整理的下一个目的地落回自己正在清空的段）不归这一项，归 D26（后台整理与放置回收） 已定项 1。
3. **≤ 4 KiB 档打包前的落点同第 1 条**；D27（小数据打包容器） 已定项 6 打包写出的容器也走第 1 条。
4. **只限第一版**。做不做分配器提示、提示写成什么在 D14（双轨（大小文件 / 持久临时）） 已定项 2 判，这一条是它的减数臂；第 1 条既然把第一版的落点唯一确定，任何第一版的提示都与它不能同真——D14（双轨（大小文件 / 持久临时）） 已定项 2 在第一版这一格上因此只剩「否」，那一项的重开条件之一就是重开第 1 条。
5. **底额引 D26（后台整理与放置回收） 已定项 3、不另写数**；整理的起跑与停机照 D26（后台整理与放置回收） 已定项 1。

第一版盘不等大时，各盘落点答案必须相同；小盘没有全空段而大盘有时，在动任何状态之前拒绝（发布层统一报假性 ENOSPC）。**写明这是第一版的限制、不是永久判据**，以后放开时要答的是 D2（RAID 条带策略） 已定项 2 那一半（小盘写满之后设备集合怎么选）与已定项 8 待办 ①（w ≥ 2 时各盘聚簇段要不要对齐）。

**射程**：它是**分配器政策**，与已定项 5 同一类——盘上没有字段声明它；例外是**从 mkfs 水位第一次建树的那一版**，单镜像上的认法是 ① 树表非空，且 extent 树条目的树 ID 是 65536（mkfs 种下的水位）；② 树表里每条条目的诞生 txg 都等于这一版的 `checkpoint_txg`。新池新建文件是这一类，mkfs 之后重挂、实例 1、2、3 的第一个文件只要从 mkfs 水位建树也是。这一类上那一次用户数据的落点要等于这一版分配记录里已定项 8 第 1 条那条政策算出的那一对（起点最小、32768 对齐、两槽都空、不在开放段里），算格式级，偏离它写出的这一类镜像不合法，判它的是 I-5.5（从 mkfs 水位建树那一版的 t1 落点）。新池新建文件那一次的取值是 t1@50180，即单元区起始槽号 U 加 4（mkfs 的实例表占 U、U + 1，树表占 U + 2，第一个 32768 对齐的空槽对从 U + 4 起；默认 768 MiB 环 U = 50176，环长不是默认值时 U 按已定项 10 ④ 现算、t1 仍是 U + 4），登记在 [layout/01-first-txn.md](../layout/01-first-txn.md) 字节表里；回收之后（例如 mkfs 的实例表那两槽被回收）落点不再是 U + 4。已知限度：checker 在这一版盘面上复算不出写这一版那一刻的回收门槛，已释放的记录一律当占着，回收门槛太严（写死从 U + 4 起扫）与太松（放宽一个 txg）的坏实现在这一类上都判不出；这是定局，不复算回收门槛（用户 2026-10-20 定「限度即终局」，原话在变更史；C689（建树落点判据抓不到写死从 U + 4 起扫的坏实现） 已还清），太松那一类由崩溃点重放里根落盘之前的崩溃态上 I-7.4（近 K 代块未被复用） 判；另几样近似（开放段、隔离与扣住位）逐条写在 `crates/singlefs-checker/invariants/07-分配器.md` I-5.5（从 mkfs 水位建树那一版的 t1 落点） 那一行。其余的写（覆盖写、之后的发布、重挂之后不属于这一类的版本）上违反它的实现写出的镜像合法，拦它只能靠运行时读数（D26（后台整理与放置回收） 已定项 6 的回落计数与全空聚簇段数）。⚠️ 第 2 条混放的代价：装着用户数据的旧段要等段里的用户数据也释放才能整段空出来，提交内生块的聚簇随挂载次数退化；现象是全空聚簇段数随挂载次数单调下降、回落次数上升。**改新池新建文件的字节：是**——新池新建文件写出的字段与宽度一个都不变，用户数据那一份的落点取值（t1@50180，连同指向它的指针与分配记录条目里的取值）由第 1 条定，[layout/01-first-txn.md](../layout/01-first-txn.md)「零、盘面地图与新池新建文件的写清单」「五、空间记账与分配」两节把它登记成已定的字节。两句待办：w ≥ 2 时各设备上的聚簇段要不要对齐归已定项 5 的实现；「同一次请求切出的事务按 key 升序取号」已写进 D16（发布语义） 已定项 5，运行时检查仍欠。runs 这个量没有条款消费它的绝对值（D26（后台整理与放置回收） 已定项 1 逐字「阈值数值不进格式：等 E10（老化：结构抗老化 vs 事后整理）」）。

**依据**：

- E151（用户数据落点的到达序与容器臂）：分得开候选的量是**提交内生块进不进聚簇段**——用户数据进 bump 段时它们绝大多数落不进，不进段时是 0（装置里提交内生块寿命一个 checkpoint，0 是下界）；它是已定项 5 用户定案的「必须」要的量，也是 D26（后台整理与放置回收） 已定项 3 第二合取式读的量的上游。同一页给出这条政策在成段重写负载与容器档上的 runs 代价，均匀随机负载打平。
- E93（老化下的放置与碎片度）：最低空槽优先这条臂在均匀随机负载上几百个 checkpoint 内就冲顶，聚簇段那条臂的回落率九成以上——「聚簇段分配」这个政策骑在整理身上；这一条不是为 runs 选的。
- 两轮三方论证的判决在 `research/prompts/d3-r2-main-verification.md`，第一轮与第二轮的经过在 [records/2026-09-10-D3项8第一轮.md](../../../records/2026-09-10-D3项8第一轮.md) 与 [records/2026-09-13-D3项8第二轮.md](../../../records/2026-09-13-D3项8第二轮.md)。
- 用户定案（在三条出路里定了庚），原话在变更史。
- 2026-09-23 三方判决 `m2-placement-falsepositive-r1-main-verification.md` 里判盘不等大落点那一格——那时的三个错误成员是对空白的第一版默认，D2（RAID 条带策略） 已定项 2 明写「不由已定项 2 覆盖」、已定项 8 在第一版几何下没有对象可执行、已定项 10 允许各副本槽号不同。
- 用户定案 2026-09-23：一个聚簇段什么时候不再算——照当天的实现写成条款：这次挂载开过的聚簇段都关着，重开之后混放允许；代码不动（里程碑「覆盖写、释放、回退与复用」收口表第 59 行），原话在变更史；有警告，见 `.claude/warnings/2026-09-23.md`。
- 用户定案 2026-10-03：用户数据落点的取值在新池新建文件上算 layout 登记的字节，原话在变更史；问题由 `research/prompts/e171-conclusion-r3-main-verification.md` 第五节第 1 行提出（D3（空间分配） 已定项 8 射程与 `layout/01-first-txn.md`「零、盘面地图与新池新建文件的写清单」「五、空间记账与分配」两节对读互相矛盾）。
- 用户定案 2026-10-04：落点约束管从 mkfs 水位第一次建树的那一版（候选甲），原话在变更史；认法与判据由 `research/prompts/c673-clauses-r1-main-verification.md` 第二节 K2 与第五节第 1 行给出（认法 ①②、t1 = U + 4 在 6 种几何 × 5 种长度上 24 格全判、全绿，攻方副本上量过）。
- 用户定案 2026-10-04：这一类的落点判据改成按这一版分配记录现算（选项「f2：按分配记录现算」），不写死 U + 4，原话在变更史；判决 `research/prompts/m3-formal-code-b17-r1-main-verification.md` 第二节 K2（t1 = U + 4 与已定项 8 第 1 条在回收之后不能同真：mkfs 之后只做 k ≥ 10 次可写挂载再写第一个文件，或同一次挂载里推 22 次以上零单元发布，I-5.5（从 mkfs 水位建树那一版的 t1 落点） 误红；攻方副本上量过，主 agent 复跑同）。
- 用户定案 2026-10-20：这一类落点判据的已知限度是定局（选项「限度即终局」），原话在变更史；`records/2026-10-18-里程碑三收尾与近期欠账.md` 第 323 行。

**欠**：C146（无空段时的回落政策全仓无定义） 第 ① 条；C243（停机谓词那个量从来没有过轨迹）。

```

**出处 `.claude/kb/decisions/03-空间分配.md:192-209`（整段抄，未转述）**

```markdown
#### 已定项 9：不许有假性 ENOSPC，删掉的空间在有界步数内可用

**定案**：两条硬要求。

1. **不许有假性 ENOSPC**：只要 `df` 报出的空闲 ≥ s，写 s 字节就必须成功。文件系统可以先在内部做有界步数的工作（推发布、抬回退下界之类）再分配；只有盘真的写满（`df` 空闲 < s）才许报 ENOSPC。
2. **删掉的空间在有界步数内可用**：删掉一个 s 字节的对象之后，同样大小的写必须在有界步数内成功。这个界要写成算得出的数，不许是「最终会」。**界是 3 次改变用户可见状态的发布**：删掉的块被最近 4 个状态里更旧的 3 个钉着，要再发生 3 次这样的发布才回可分配集合；在那之前 `df` 不把它算空闲，写它按 `df` 是真 ENOSPC，第 1 条不破。文件系统自己推的空发布不算这 3 次（空发布不改用户可见状态）。

可判定形式是不变量 I-5.3（报出的空闲都兑现得了）。**它当场改变了什么**：`df` 报什么是这条定案的另一半：D16（发布语义） 已定项 1 定的 `df` 口径把被扣掉的那几截排除在空闲之外，第 1 条靠它兑现，代价是 `df` 少报、删掉 s 字节时 `df` 可能一个字节都不涨。照 D16（发布语义） 已定项 1 字面这一半今天多报（判决 `research/prompts/m3-admission-r1-main-verification.md` 第三节「`df` 的写法」那一行：D16（发布语义） 字面 73/73 多报），`df` 写法未定，见 C370（需求、可用与 df 没有共同单位）。第 2 条的「有界步数」按 D16（发布语义） 已定项 1 选定的臂计：准入不够时先推空发布抬 F，一次准入最多 B = 4 + 2 k_tol 次发布，做满仍不够才报 ENOSPC。

**射程**：只管假性 ENOSPC 与删除后的可用性，不管真 ENOSPC；配额不在内（第一版不做每快照配额）。未在真设备上验证。**界 3 是按字面判的**：D16（发布语义） 已定项 1 选定那条臂在填满之后删了再写，按它自己的 `df` 那几次都是真 ENOSPC，而按这一条第 2 条的字面它没达标；射程只到 k_tol = 2 次根槽写失败。保留池那一项不是常数——它按 D28（挂载期承诺量） 已定项 4 每次发布按当时的结构现算，第 1 条的兑现条件随树高变。

**依据**：

- E139（按盘回退下界的收严形态）：填满之后删了再写的失败窗口逐窗数出来，越过「3 次」的那几格是哪几窗有产物；一次准入的发布数实测不超过 B，跑前保留池下不卡死。
- 用户定案（逐字「我们的目标是 避免 btrfs ENOSPC， 我们绝对不能接受 ENOSPC」，在「只要第 1 条」与「两条都要」之间选了两条都要），原话在变更史。它正是 [pitfalls.md](../pitfalls.md) 第 1 条那个症状的两半，而已定项 13 的「不分类」只治了其中一个病根。

**欠**：无（C283（准入失败时不先推发布就报 ENOSPC） 2026-09-26 已还清，见欠账表已还清表）。

```

**出处 `.claude/kb/decisions/03-空间分配.md:210-245`（整段抄，未转述）**

```markdown
#### 已定项 10：聚簇段的定义、用户数据落点的四格、提交内生块的 bump 次序

**定案**：① 聚簇段段长 64 个 16 KiB 槽；段起点 = 单元区内最低的、64 槽对齐的全空段；bump 指针只住内存，挂载后新开一段；「全空聚簇段数」= 段内 64 槽都没有未释放分配记录的段数（记账行记的就是它；代码今天的 `empty_segments`（`crates/singlefs-core/src/allocator.rs`）是占用口径、要改成记录口径，C372（全空聚簇段数两份定义相反））；分配器开段另看隔离位（影子账、D19（块指针的结构与宽度预算） 已定项 5 硬规则 1）与抬 F 扣住位，那一个量叫「可开聚簇段」（`lowest_empty_segment` 按它找段起点）——两个量分开登记，不许互相顶替（C379（记账的全空聚簇段数与分配器能开的段两个口径） 取 B，用户 2026-09-28 定，被攻过一轮，判决 `research/prompts/abandoned-floor-r1-main-verification.md` M3：自然历史里部分隔离的一段让两者差 1 段）。② 用户数据的候选集加一条合取「候选落点不在任何开放的聚簇段里」。③ 数据单元取最低的、起点 32768 对齐、两槽都空、整跨度都过合取的起点；索引节点是提交内生块，从开放聚簇段 bump（已定项 5），「最低的空槽」只在开放段之内说，不下探到段外。④ 系统配置几何段加「单元区起始槽号」8 字节（第一版 = journal 环末尾的下一个槽：mkfs 按环长现算写进系统配置偏移 417，起点不在 64 槽段边界上在任何写之前拒；读者择系统配置时判 417 = 现算起点，对不上整池拒；默认 768 MiB 环是 50176，实审 A3b 报告 `m2-rev-a3b-implementer-report.md`），分配器不下探；固定结构不写分配记录。⑤ **提交内生块从开放段 bump 的次序**：一次发布重写实例表单元时（写行那一次），**实例表单元最前**，多于一片时尾片先（第 k 片当第 k+1 片的父，照「先叶后根」）；其余按**树 ID 升序**，树内**先叶后根**、同层按 key 升序，**中央映射树倒数第二**，**树表单元最末**；码 3 打包容器按「数据单元」那一档取落点（起点 32768 对齐、两槽都空）。这个次序同时就是出生序号的发号次序（D19（块指针的结构与宽度预算） 已定项 9）。

⑤ 在新池新建文件上的落点（开放段 [50240, 50304)，bump 游标从 50240 起）：

| 序 | 单元 | 树 ID | 落点 | 出生序号 |
|---|---|---|---|---|
| t2 | extent 树根兼叶（码 2，16 KiB） | 65536 | 50240 | 0 |
| t3 | inode 树叶容器（码 3，32 KiB，起点要 32768 对齐 ⇒ 跳过 50241） | 65537 | 50242–50243 | 0 |
| t4 | inode 树根（码 2） | 65537 | 50244 | 1 |
| t5 | 分配记录树叶，设备 0（码 2） | 65538 | 50245 | 0 |
| t6 | 分配记录树叶，设备 1（码 2） | 65538 | 50246 | 1 |
| t7 | 分配记录树层级 1 节点，设备 0（码 2） | 65538 | 50247 | 2 |
| t8 | 分配记录树层级 1 节点，设备 1（码 2） | 65538 | 50248 | 3 |
| t9 | 分配记录树根，层级 2（码 2） | 65538 | 50249 | 4 |
| t10 | 记账树根兼叶（码 2） | 65539 | 50250 | 0 |
| t11 | 中央映射树根兼叶（码 2） | 65540 | 50251 | 0 |
| t12 | 树表单元（码 2） | — | 50252 | 0 |

槽 50241 是这个次序留下的一个空洞：t2 占掉 50240 之后游标停在 50241，而 t3 要 32768 对齐的两槽 ⇒ 游标进到 50242，50241 一直空着。它让新池新建文件的空闲 run 数从 3 变 4（记账第 10 项，D5（快照 / 空间记账机制） 已定项 8）。

**射程**：③ 里「索引节点取最低的空槽」只在开放段之内说——按字面读「单元区里最低的空槽」会与已定项 5 / 已定项 8「聚簇段只给提交内生块」给出两套写清单，字节表与 E142（新池新建文件的干跑） 装置取的都是开放段内那一套。⑤ 取「树 ID 升序 + 树内先叶后根」的理由：树 ID 是全仓到处在用的那个键（D8（核心索引结构） 已定项 8 逐字「条目按树 ID 升序排」），先叶后根是 COW 本来的顺序（父指针要等孩子的落点定了才写得出来）；中央映射树倒数第二是因为它要装别人的条目，树表单元最末是因为它装全部树的根指针。码 3 容器取「数据单元」那一档，依据是 D18（块里携带什么信息） 已定项 9 的对齐要求按单元大小分档、与它是不是用户数据无关，两种读法在开放段内给同一个落点，写死为条款把 C324（码 3 打包容器要不要 32768 对齐没写） 收口。**改新池新建文件的字节：是**——十一个提交内生块（默认几何）的落点整体重排，分配记录的 key、位置条目的槽号、中央映射条目的 value 全跟着变。这一项没有走三方论证，主 agent 的推荐与用户的选择都可推翻。 ⚠️ ⑤ 那张落点表是默认几何（768 MiB 环、两块 4 GiB 盘）上的值：提交内生块的个数随几何变（环长 1024 MiB 时 13 个），t2–t12 偏离这张表写出的镜像算不算不合法，这一版没有条款、不判（只有 t1 由已定项 8 射程判）。

**依据**：

- E151（用户数据落点的到达序与容器臂）：段长 64 槽是它的装置建模时用的几何，段长这个数不是这一项现拍的。
- E142（新池新建文件的干跑）：⑤ 那张落点表与 50241 那个空洞是它写出来的字节，字节表与它逐字对得上。
- 用户定案（① 到 ④ 按总审核的选项逐条定，⑤ 另一次弹窗定案；⑤ 里实例表最前那一句 2026-09-23 定），原话在变更史。
- `research/prompts/m2-presumed-clauses-r1-main-verification.md` K1：条款没点名实例表时，三个候选里只有「最前」不与 ⑤ 的「中央映射树倒数第二，树表单元最末」冲突（本地攻方两次一致）。K1 之前这个次序只以字节表里那一行的行序存在，一条条款都没有——两个实现按各自顺序 bump，写出来的落点、出生序号、映射 key 全都不同，而没有任何东西会说话。
- 用户定案 2026-09-24：实例表多于一片时尾片先（实四交回的两处条款空白之一，原话在变更史）；无实验：两种排法只差落点与出生序号的次序，没有可量的代价差。
- 判决 `research/prompts/c673-clauses-r1-main-verification.md` 第二节 K2：⑤ 那张表按实现现值改（11 个单元、树 ID 从 65536 起，落点与 [layout/01-first-txn.md](../layout/01-first-txn.md)「零、盘面地图与新池新建文件的写清单」同；攻方副本上读出提交内生块相对单元区起点 [64, 66, 68, …, 76]），提交内生块个数随几何变（同一份副本上环长 1024 MiB 时 13 个）；出生序号照 D19（块指针的结构与宽度预算） 已定项 9 逐树从 0 起数。
- 判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G4：C372（全空聚簇段数两份定义相反） 按记录口径收口，① 的括号与 D26（后台整理与放置回收） 已定项 1 第 1 条收成同一句。

**欠**：无（C324（码 3 打包容器要不要 32768 对齐没写） 已由 ⑤ 收口，已还清，见欠账表已还清表）。

```

**出处 `.claude/kb/decisions/03-空间分配.md:246-259`（整段抄，未转述）**

```markdown
#### 已定项 11：分配记录 20 字节里跨度段进 value

**定案**：进 value：key = (设备身份 4, 16 KiB 槽号 6) 共 10 字节，value = (跨度段 2, 分配代或释放代 8) 共 10 字节；已释放标志仍在跨度段最高位。分配记录树的码 2 头按 key 宽 10 算。

**射程**：只定跨度段归哪一边，字段本身的宽度与语义在已定项 7。**改新池新建文件的字节：是**——t5 的条目宽字段、声明长度与那批记录的排布。这一项没有走三方论证，主 agent 的推荐与用户的选择都可推翻。

**依据**：

- E142（新池新建文件的干跑）：t5 的条目宽、声明长度与记录排布按这一项的宽度写出来，字节表与产物逐字对得上。
- 已定项 7 逐字「条目不删、改写成已释放 + 释放代」是**原地改 value**；跨度段进 key 时标志一翻 key 就变，树里同一个落点会留下两条记录。
- 用户定案，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/03-空间分配.md:260-269`（整段抄，未转述）**

```markdown
#### 已定项 12：分配准入合取的唯一登记位，与两条准入式的关系

**定案**：分配准入的合取式只登记在已定项 7 那张表一处，共七条，别处一律指过去。D16（发布语义） 已定项 1 的 `可分配 = min(可再分配 + 活元数据 − 保留池, df)` 与 D28（挂载期承诺量） 已定项 1 的九项式子是两道串联的闸：先过 D28（挂载期承诺量） 的「可用 ≥ 需求」，再过 D16（发布语义） 的抬 F 逻辑。「需求」按盘字节计、每盘相同（与 D5（快照 / 空间记账机制） 已定项 7 的「已分配」逐盘物理字节同口径）。

**射程**：定的是登记位与两式的先后，不改任何一条合取的内容。**改新池新建文件的字节：否**——新池新建文件恒过准入，这一项挡的是准入判定本身。这一项没有走三方论证，主 agent 的推荐与用户的选择都可推翻。

**依据**：无实验：收到一处是 `.claude/singlefs-ai-sop/rules/kb-discipline.md` 第 4 条的直接要求（收到一处之前合取式有四处互不相同的写法），没有可量的量；两道闸的读法与 D16（发布语义） 已定项 1「准入不够时先推空发布抬 F」的措辞一致。用户定案，原话在变更史；「需求」按盘字节、每盘相同：用户定案（原话在变更史）。

**欠**：无。

```

**出处 `.claude/kb/decisions/03-空间分配.md:270-279`（整段抄，未转述）**

```markdown
#### 已定项 13：不分 data / metadata 两类空间

**定案**：不分 data / metadata 两类空间，全局统一分配器。

**射程**：管的是分配器有没有第二个分类轴，不管落点政策（已定项 8 / 10）、不管设备维（D2（RAID 条带策略） 已定项 8）、也不管一棵树的节点与它的叶要不要分开放。它直接推出已定项 14 那条连带承诺：不分类意味着碎片化压力全压在一个分配器上。D26（后台整理与放置回收） 已定项 3 的底额「越小越好」这条口径也建在它上面——不分类之后「释放要申请空间」不像 btrfs 那样致命。

**依据**：无实验：这一格定的是不开第二个分类轴，没有可量的量。病根是 [pitfalls.md](../pitfalls.md) 第 1 条那个症状——btrfs 的两级分配加 chunk 分类僵化，`df` 还有几十 G 却报没空间、删文件也报没空间；按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「别的项目怎么做，是线索不是证据」，它只当反证与机制，不当正证。用户定案，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/03-空间分配.md:280-292`（整段抄，未转述）**

```markdown
#### 已定项 14：不分类的连带承诺——后台整理常驻、自动、用户无需知道

**定案**：选了已定项 13 的不分类，就等于承诺后台整理是**常驻的、自动的、用户无需知道的**——btrfs 靠人手动跑 `balance` 来救，那是它最大的运维恶名之一。这个承诺的承接方是 D26（后台整理与放置回收）：机制在它的已定项 1 / 2 / 3，触发判据、停机谓词与常驻底额都在那里，未定的那几项也挂在那里。

**射程**：承诺的是「不要用户操心」，不承诺整理用的是哪套算法、阈值取多少（数值等 E10（老化：结构抗老化 vs 事后整理））。两处例外要连着引：加密开启时这条承诺不成立（已定项 16）；D26（后台整理与放置回收） 已定项 7 把回收分成两条线之后，走交给盘那条线时腿一里哪几件活停下还未定（D26（后台整理与放置回收） 未定项 9），那一问要连同已定项 5 一起答。

**依据**：

- E93（老化下的放置与碎片度）：不做任何放置的那条臂在均匀随机负载上几百个 checkpoint 内就冲顶，不是缓慢退化；聚簇段那条臂的回落率九成以上，「聚簇段分配」这个政策骑在整理身上 ⇒ 没有常驻整理喂空段，落点政策本身立不住。
- 用户定案，原话在变更史。

**欠**：C243（停机谓词那个量从来没有过轨迹）。

```

**出处 `.claude/kb/decisions/03-空间分配.md:293-305`（整段抄，未转述）**

```markdown
#### 已定项 15：空间预留取准入控制形态

**定案**：空间预留用准入控制形态，量要小。预留不是「划一块地不给用」，而是：任何增加空间占用的操作，进门前先算「**最坏情况下解开自己需要多少**」，算不过就拒绝。这样预留量动态贴着实际需求，不白占；量要小。它的可执行形式是 D28（挂载期承诺量） 已定项 1 的九项式子，读数怎么合成在 D28（挂载期承诺量） 已定项 2，登记位是已定项 7 那张七条合取表。

**射程**：定的是形态与方向（量要小、贴着需求），不定任何一个具体的数——每一项算多少在 D28（挂载期承诺量），整理的常驻底额在 D26（后台整理与放置回收） 已定项 3。「解开自己需要多少」这个量含哪几笔由已定项 2 扩写（墓碑、livelist 的 FREE 条目），扩一笔就把「不白占」紧一分。

**依据**：

- E82（准入的在飞合成）：三种直觉写法各出一类可数违例——只读已发布值同窗超卖、把窗口内释放记成可用会吃掉 defer 该扣住的块、忘了墓碑盘满后删除全数死锁 ⇒ 准入控制这个形态要配在飞合成才立得住，而不是划一块地。
- 用户定案，原话在变更史。

**欠**：C87（准入读的合成值无定义）。

```

**出处 `.claude/kb/decisions/03-空间分配.md:306-319`（整段抄，未转述）**

```markdown
#### 已定项 16：加密开启时无密钥的一侧只读只检测

**定案**：加密开启时，无密钥的一侧**只读只检测，既不搬也不回收**。

- **不能搬**：四条已定条款不能同时为真——搬一个单元就是发出一条新条带（D2（RAID 条带策略） 已定项 6 逐字「`w ≥ 2` 是硬下界（零冗余的条带不许发出）」），新条带的每一格都要一条成员记录（D2（RAID 条带策略） 已定项 12），而成员表逐字「**必须住在单元的载荷里**」，载荷是密文（D9（加密） 已定项 11「除系统配置外全部加密，元数据留最小明文头」）⇒ 无密钥侧写不了成员表 ⇒ 搬不成。`w ≥ 2` 是硬下界 ⇒ 这条对**任何**池成立，不只是 `w ≥ 3` 的池。
- **不能回收**：记账是权威态（D21（权威态与派生态的分界））且加密，无密钥侧写不了它 ⇒「把空间标记为可分配」在锁盘期做不到。
- **检测始终可做**（指针里有不要密钥的密文校验和）——这是无密钥侧唯一保留的能力。

**射程**：代价要写明并连着引——已定项 14 那条「常驻、自动、用户无需知道的后台整理」在加密开启时**不成立**，锁盘期整理与疏散全部停摆，近满卷上的换盘与缩容同样做不成。这是知情接受的，不是遗漏。加密不进第一个可运行版本（D9（加密） 已定项 10）⇒ 第一版没有执行路径。

**依据**：无实验：不是取舍改了偏好，是「能搬」这一侧被四条已定条款证明不可达，没有可量的代价要比。用户定案（撤销原来的「能搬」），原话在变更史；权威记录在 D9（加密） 已定项 5。

**欠**：C167（明文映射层的引用全仓回扫没做）。

```

**出处 `.claude/kb/decisions/03-空间分配.md:320-334`（整段抄，未转述）**

```markdown
#### 已定项 17：释放空间这个操作本身不需要申请空间

**定案**：COW 系统必须保证「释放空间」这个操作本身不需要申请空间。btrfs 没在设计层保证这点，靠一堆启发式补了十几年还在出问题。凡是释放路径要写的字节（墓碑单元、livelist 的 FREE 条目），空间在写入那一刻就按已定项 15 的准入控制预付掉（已定项 2）。

**射程**：它是 D3（空间分配） 里最硬的一条，别的分项都要过它：已定项 2 定的是怎么在不破它的前提下容纳「删除要写盘」，已定项 9 第 2 条定的是删掉的空间多久回来。它不承诺「释放一定立刻成功」——被 defer 窗口与回退下界钉住的那几代按 D16（发布语义） 已定项 1 的口径不算空闲，那是真 ENOSPC 不是破例。删除类发布不判式子（已定项 2）是「释放空间不需要申请空间」在第一版的落法；墓碑树根满时长高（D8（核心索引结构） 已定项 6），不因结构拒删除。

**依据**：

- E130（每头一份 livelist 过不过有界销毁）：候选甲每释放一个块要多写的那几个字节逐字撞上这条原则，逼出「每批最坏空间需求有界且可在批前预留」这条判据的可执行读法。
- E82（准入的在飞合成）：忘掉墓碑那一笔的臂在盘满后删除全数死锁——这条原则被破掉时的现象是可数的。
- 删除类发布不判式子：`research/prompts/m3-delete-r3-main-verification.md` 第三节 G1-删除类发布怎么判那一行；墓碑树根满时长高：同一份判决第三节 G6-墓碑树根满那一行与用户定案 K-R3-3「写根长高（推荐）」，原话在变更史。
- 用户定案，原话在变更史。

**欠**：C260（livelist 条目的预付没算进解开自己那个量）。

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

**出处 `.claude/kb/decisions/03-空间分配.md:8-30`（整段抄，未转述）**

```markdown

D3（空间分配）的分项只有一套编号，与「未定项」一节连号，不重排。

| # | 分项 | 定案 |
|---|---|---|
| 1 | **分配记录的粒度** | 逐落点答得出「这个落点属不属于现行版本」，不是聚合量 **状态：已定。** **进度：已实现。** |
| 2 | **墓碑单元与「释放空间不需要申请空间」怎么共存** | 墓碑按活对象写入时预付；删除各批由准入式子里的常驻环窗口兜，删除类发布不判式子 **状态：已定。** **进度：已实现。** |
| 3 | **分配记录的载体形态** | 住一棵独立 keyspace 的 btree，key 是落点（设备身份 + 设备内偏移），value 是分配代 **状态：已定。** **进度：已实现。** |
| 4 | **准入不等式的权威形式** | 权威正文在 D28（挂载期承诺量） 已定项 1，这一格只留结论句与指针 **状态：已定。** **进度：已实现。** |
| 5 | **提交内生块的落点** | 提交自身产生的块从聚簇段分配；用户数据块的落点不受此约束 **状态：已定。** **进度：已实现。** |
| 6 | **准入的在飞合成** | 权威正文在 D28（挂载期承诺量） 已定项 2，这一格只留结论句与指针 **状态：已定。** **进度：已实现。** |
| 7 | **分配记录条目的盘上编码与宽度** | 落点粒度 16384；key =（设备身份 4, 槽号 6）；value =（跨度段 2, 代 8）；释放时改写不点删 **状态：已定。** **进度：已实现。** |
| 8 | **用户数据块的默认落点政策，以及段耗尽时的回落** | 第一版不进聚簇段，取已选设备内起点最小、整跨度过准入合取的空槽；段耗尽回落同一规则 **状态：已定。** **进度：已实现。** |
| 9 | **不许有假性 ENOSPC，删掉的空间在有界步数内可用** | `df` 报出 s 字节空闲就写得进 s 字节；删掉 s 字节之后同样大小的写在有界步数内成功 **状态：已定。** **进度：未实现。** |
| 10 | **聚簇段的定义、用户数据落点的四格、提交内生块的 bump 次序** | 段长 64 槽、起点是最低的 64 槽对齐全空段；bump 次序按树 ID 升序、树内先叶后根 **状态：已定。** **进度：已实现。** |
| 11 | **分配记录 20 字节里跨度段归 key 还是 value** | 进 value：key 10 字节、value 10 字节；已释放标志仍借跨度段最高位 **状态：已定。** **进度：已实现。** |
| 12 | **分配准入合取的唯一登记位，与两条准入式的关系** | 合取式只登记在已定项 7 一处、共七条；两条准入式是串联的两道闸 **状态：已定。** **进度：已实现。** |
| 13 | **分不分 data / metadata 两类空间** | 不分，全局统一分配器 **状态：已定。** **进度：已实现。** |
| 14 | **不分类带出来的连带承诺** | 后台整理必须常驻、自动、用户无需知道；承接方是 D26（后台整理与放置回收） **状态：已定。** **进度：未实现。** |
| 15 | **空间预留取什么形态** | 取准入控制：任何增加空间占用的操作进门前先算最坏情况下解开自己需要多少，算不过就拒绝 **状态：已定。** **进度：已实现。** |
| 16 | **加密开启时无密钥的一侧能做什么** | 只读只检测，既不搬也不回收；常驻自动整理那条承诺在加密开启时不成立 **状态：已定。** **进度：未实现。** |
| 17 | **释放空间这个操作本身不需要申请空间** | COW 系统必须保证它；释放路径要用的空间在写入那一刻预付掉 **状态：已定。** **进度：已实现。** |

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
