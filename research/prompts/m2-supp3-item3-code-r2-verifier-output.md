# 核查员报告：m2-supp3-item3-code-r2

你交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

## 〇、判别力自证

挑的引用：Opus 报告「crash_injection.rs:869」（`fn draw_a_proper_subset` 的签名行）。
在草稿目录 `/tmp/claude-1000/m2s3i3-r2-verifier/self-test/crash_injection.rs`（工作区该文件的副本）里把行号
人为 +1 → 870，按第二步核（取该行内容与引用要证明的原文比对）：

```
待核内容（引用要证明的原文）：
fn draw_a_proper_subset(source: &mut SeededRandomSource, segment_length: usize) -> Vec<bool> {

副本第 870 行实际内容：
    if segment_length <= WRITES_A_SUBSET_MASK_HOLDS {
```

两者不同字 ⇒ **判 ✗**。核查方法分辨得出行号错位，自证通过。

## 一、输入核对

- 报告 sha256：`m2-supp3-item3-code-r2-opus-output.md` 现算
  `2fc17008cda5410d0faf3385142bbaf0e62df1843940cf2513a2893b9b449fff`，与交回时给的**逐字相同**。
  `m2-supp3-item3-code-r2-sonnet-output.md` 现算
  `00f4af6c99a02edba8a344022a729e404764340d67fd910e98ee4bad92dd9c9c`，与交回时给的**逐字相同**。
  两份都不落入「报告文件现在的 sha256 与交回里给的对不上」那一条，正常按内容核。
- 开工快照 16 项：`crates/` 相关 10 项现算 sha256 与快照**逐字相同**（工作区这一轮没被改过）；
  `.claude/kb/checks-owed.md` 现算 sha256 与快照不同——与主 agent 派发时说明的一致（暂存区被别的会话重排）。
  独立复核 C284、C397 两行：现在的工作区行号分别是 252、349（快照记的是 257、354），
  取这两行现算 md5 得 `46e4bf46cad8e8a38c2459f13316a8d0`、`6f4b28f4a38cc8efced85c62af748f48`，
  与主 agent 给出的 `46e4bf46cad8`、`6f4b28f4a38c` 前缀**逐字相同**。这两行按内容核，不按行号核。
  `.claude/kb/invariants.md` 不在快照的 16 项里；三条腿的正文与核对表都没有引用它的具体行号，未触发「没有快照可核」的情形。

## 二、主 agent 已知的两处要求复核

### 2.1 `crates/mutations.tsv` 第 189、191、192 行

独立现查（不看本地腿或主 agent 的结论，直接 `awk` 现取）：

| 行 | 第 1 列（增补条款） | 第 2 列（文件） |
|---|---|---|
| 189 | 增补 3 第 3 件（用户 2026-09-20 定案第 7 条，同日定的范围）：这个测试周期的种子基换成别的数 | `crates/singlefs-harness/src/crash_injection.rs`（`SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE`）|
| 191 | 增补 3 第 3 件（用户 2026-09-20 定案第 3 条的配套口径）：记录核对器第二条判据不认「被流里更晚的写盖过」，合法复用被判成单元缺席 | `crates/singlefs-harness/src/crash.rs` |
| 192 | 增补 3 第 3 件（用户 2026-09-20 定案第 7 条，同日定的范围）：随机历史快档的种子基改回写死的别的数 | `crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs`（`FAST_TIER_FIRST_SEED`）|

**判 ✓**：第 189、192 行钉的确实是种子基换成别的数就红；第 191 行钉的确实是记录核对器第二条判据（与种子基无关），
是用户定案第 3 条的配套口径，不是第 7 条。本地腿转述核对表「第 3 节」的更正（派发提示写的「191、192」应为
「189、192」）与主 agent 现查的结论**独立复核后一致**。

### 2.2 Opus 报告第六节第 7 条：受污染数据有没有漏进正文

Opus 报告 6.7 自陈：`mv 文件.orig 文件` 还原变异后 cargo 未重编，一度测得「段长 {1,2,4,12,20}、段内有洞 39、
崩溃状态 93」这组受污染的数，后来用全新副本复现出干净的「段长 {1,2,4,10,18}、段内有洞 40、候选段 1432、
镜像对 2782」，前一组已作废。

在报告全文 398 行范围内独立 grep：

```
$ grep -n "段内有洞\|{1, 2, 4" research/prompts/m2-supp3-item3-code-r2-opus-output.md
12:| K1-① | ... | 快档 93 个崩溃状态里 40 个段内有洞（43%）...
70:崩溃状态 93 个；段内有洞（后发的写先持久）40 个
87: ...（`崩溃状态 93 个：…段内有洞（后发的写先持久）的 40 个`）
379:   这条腿因此一度量到一组与干净构建不同的数（段长 {1, 2, 4, **12**, **20**}、段内有洞 39、崩溃状态 93）。
381:   （段长 {1, 2, 4, **10**, **18**}、段内有洞 40、候选段 1432、镜像对 2782）...
```

「段内有洞」只在正文另外两处（12、70、87 行）出现，取值均为干净的 **40**，不是受污染的 39；
「39」这个数字只出现在第 379 行（6.7 节自陈受污染那一句里），别处零命中。
「段长 12」在第 317 行另有一次出现，但那是变异 C 深枚举一段合并后的段长（context 见报告 309-319 行：
写死历史里那个 12 个写的段被整段枚举到「段长 12」「段长 14」两档），与 6.7 节作废的段长分布 `{1,2,4,12,20}`
是两件不同的事，不构成污染数据泄漏。

**判 ✓**：那组受污染的数（39、{1,2,4,12,20}）确实只出现在 6.7 节自陈的那一句里，没有流进报告正文别处。

## 三、云端攻方（Opus）报告核对表

模型目录 11 个文件 sha256 现算，与报告「一之二」节列的 11 个逐字相同（全 ✓，不逐行贴，命令：
`sha256sum research/prompts/m2-supp3-item3-code-r2-opus-model/*` 与报告表格逐行比对）。

| 引用 | 核的结果 | 命令 / 依据 |
|---|---|---|
| `crash_injection.rs:697,698,706`（`check_records_against` 调用、计数、判据合取） | ✓ 三行内容逐字对应 | `grep -n "check_records_against(&image\|tally.record_checks += 1\|record_check == RecordCheck::default()" crates/singlefs-harness/src/crash_injection.rs` |
| `crash_injection.rs:1314-1317`（`record_checks == crash_points` 断言） | ✓ 范围精确（`assert_eq!(` 至 `);`） | `sed -n '1310,1320p'`，函数 `crash_points_are_reproducible_proper_subsets_sorted_by_segment` 起于 1286 行 |
| `crates/mutations.tsv:183`（K3-⑤ 计数器变异） | ✓ 六列内容与报告描述逐字一致 | `awk -F'\t' 'NR==183'` |
| `crates/mutations.tsv:67`（K3-⑥ 屏障变异） | ✓ 与报告描述逐字一致 | `awk -F'\t' 'NR==67'` |
| `crates/mutations.tsv:191`（旁证，见 2.1） | ✓ | 同上 |
| `crash_injection.rs:869`（`draw_a_proper_subset` 签名）、`:871`（掩码行）、`WRITES_A_SUBSET_MASK_HOLDS = 63` | ✓ 三处行号精确 | `grep -n "fn draw_a_proper_subset\|let mask = source.below\|WRITES_A_SUBSET_MASK_HOLDS"` |
| 2.1 节量具原样输出块（`run-fresh-copy-probe1.txt`） | ✓ 产物文件逐字含有该块；**并且本核查员在独立草稿副本上重跑同一个测试，输出逐字重现**（含 2830/2782/1432/45577958 等全部数字） | 见下方「复跑」 |
| 3.1 节表格（68/93/104/121 次，`root_without_record`/`claimed_state_missing_unit` 均 0） | ✓ 四段数字与 `run-baseline-crash-injection.txt` 逐字对应 | `grep -n "记录核对器跑了" research/prompts/m2-supp3-item3-code-r2-opus-model/run-baseline-crash-injection.txt` |
| 3.2 节代码块「7 passed…15.63s」「5 passed…16.36s」 | **✗ 产物里找不到这两行原样**：11 个模型文件里没有一个文件含 `15.63s` 或 `16.36s`；最接近的 `run-fresh-copy-mutation-A.txt` 是同一次判定（7 passed/0 failed/1 ignored、5 passed/0 failed/39 filtered out 完全一致）但秒数是 `15.76s`/`16.56s`，与正文引的秒数不同 | `grep -rn "15.63s\|16.36s" research/prompts/m2-supp3-item3-code-r2-opus-model/ research/prompts/m2-supp3-item3-code-r2-opus-output.md`；`grep -n "test result:" .../run-fresh-copy-mutation-A.txt` |
| 3.4 节判红原样输出块（`segment_index: 6`、`persisted_within_the_segment: [false,false,true]` 等） | ✓ 逐字见于 `run-mutation-67-crash-injection.txt`，**本核查员独立重跑变异 67 也逐字重现同一块** | 见下方「复跑」 |
| 3.5 节「6 passed;1 failed…」与 `:344:5` 断言 | ✓ **本核查员独立重跑「变异 67+A」重现同一失败**，失败行确系 `crash_injection.rs:344` 的 `assert_eq!(injection.tally.record_root_without_record, 0, …)` | 见下方「复跑」 |
| `crash.rs:531-540`（`written_over_later`）、`:544-549`（`root_without_record` 判据）、`:550-563`（`claimed_state_missing_unit` 判据） | ✓ 三处范围精确 | `grep -n "let written_over_later\|if !publish.records.is_empty()\|check.root_without_record = true"`，`sed -n '550,565p'` |

**复跑（草稿目录 `/tmp/claude-1000/m2s3i3-r2-verifier/opus-rerun/`，`rsync -a --exclude target --exclude .git` 拷自工作区，全程 `nice -n 19`，不在原目录跑，实测前后 `ps` 未见 qemu/vm-bench/fio/e152 在跑）**：

| 步骤 | 命令 | 结果 |
|---|---|---|
| probe1（干净构建量具） | `cargo test -q -p singlefs-harness --test zz_opus_r2_probe -- --nocapture` | 输出与 `run-fresh-copy-probe1.txt` **逐字相同**（含全部直方图数字）；`test result: ok. 1 passed…finished in 54.72s`（对照文件是 55.70s，秒数天然不同，判定不变） |
| 变异 A（单独） | 施加 `:706` 判据丢弃，`cargo test --test second_transaction_supplement_three_crash_injection` 与 `--lib crash_injection` | `7 passed; 0 failed; 1 ignored`／`5 passed; 0 failed; 0 measured; 39 filtered out`，与报告及 `run-fresh-copy-mutation-A.txt` 判定一致（秒数 15.53s/16.02s，自然波动） |
| 变异 67（单独） | 施加 `mutations.tsv:67`，`cargo test --test second_transaction_supplement_three_crash_injection -- --nocapture` | `4 passed; 3 failed`，失败用例名与 `run-mutation-67-crash-injection.txt` **逐个相同**（`crash_states_land_inside_rollbacks_and_floor_raises`、`every_crash_state_of_a_written_out_history_recovers_into_a_committed_version`、`crash_states_after_raising_the_floor_into_the_gap_match_the_second_known_red_form`），判红崩溃状态块（`segment_index:8`／`segment_index:6` 等）与产物逐字重现 |
| 变异 67 + 变异 A（叠加） | 在变异 67 之上再施加 `:706` | `6 passed; 1 failed`，唯一失败是 `every_crash_state_of_a_written_out_history_recovers_into_a_committed_version`，panic 于 `crash_injection.rs:344:5`，断言消息「今天的代码上没有『根在而它那次发布的记录一条都不在』的状态」，与报告 3.5 节描述逐字一致 |

三次独立重跑（Opus 自己的副本、`run-fresh-copy-*` 第二副本、本核查员的第三副本）在全部可比字段上一致，唯一
对不上的是 3.2 节那两行未落盘的秒数文本（见上表 ✗ 一行），判定本身（全绿/失败用例名/失败计数）三次全部相同。

**补充**：Opus 报告不含
一份「转述英文/引 kb」核对表（它引的是代码与产物，不是 kb 条款或英文转述），本节按第 2、3、4 步逐条核完，
没有第 5 步适用的对象。

**Opus 报告计数**：核了 12 处引用/产物 + 4 段复跑命令 = 16 项；✓ 15 项；✗ 1 项（3.2 节两行秒数文本未落盘存档，
判定本身经三次独立复现不受影响）。

## 四、云端正推（Sonnet）报告核对表

| 引用 | 核的结果 | 命令 / 依据 |
|---|---|---|
| `crash_injection.rs:848-850`（`operation_kind` match 块） | ✓（引用内容三行精确对应；报告代码块多带了闭合的 `};` 一行，那一行实际在 851，不影响引用的三行本身对不对） | `grep -n "let operation_kind = match step\|StepPosition::StartingPoint => None,\|StepPosition::Operation"` |
| `crash_injection.rs:792-798`（`candidate_segments` 定义） | ✓（同上，代码块多带的 `.collect();` 实际在 799） | `sed -n '790,799p'` |
| `crates/mutations.tsv:184` | ✓ 六列与报告描述逐字一致 | `awk -F'\t' 'NR==184'` |
| `second_transaction_supplement_three_crash_injection.rs:197-200` | ✓ 范围精确、断言文字逐字相同 | `sed -n '197,200p'` |
| `crash_injection.rs:1377-1380` | ✓ 范围精确、断言文字逐字相同 | `sed -n '1377,1380p'` |
| `crash_injection.rs:1383-1384`（注释）、`:1385-1414`（整条测试） | ✓ 两处范围精确（1385 起于 `#[test]`，1414 是该函数闭合 `}`） | `sed -n '1383,1414p'`；`awk 'NR==1414'` |
| `crash_injection.rs:315`（字段定义）、`:651`（计数递增） | ✓ 精确；字段的注释实际在 314（报告只引 315 作字段定义行，未误标注释行号） | `sed -n '313,317p'`；`sed -n '649,660p'` |
| `crash_injection.rs:654-659`（`if let Some(kind)`） | ✓ 范围精确 | 同上 |
| `second_transaction_supplement_three_crash_injection.rs:140-236` 及表内 17 处子行号（143-147/148-151/152-155/156-159/160-177/178-192/193-196/197-200/201-204/205-208/209/210-213/214-217/218-221/222-225/226-229/230-235） | ✓ 抽查 143-147、160-177、178-192、209、230-235 五处，范围全部精确到闭合括号；函数整体 140-236 也精确 | `grep -n "fn assert_every_crash_injection_path_was_exercised"`；逐段 `sed -n` |
| `second_transaction_supplement_three_crash_injection.rs:25`（`FAST_TIER_SEEDS = 24`） | ✓ | `grep -n "FAST_TIER_SEEDS"` |
| 背景材料 `_m2-supp3-item3-code-r2-background.md:37`（K6 题面引文） | ✓ 逐字子串匹配 | `sed -n '37p'` |
| diff `_m2-supp3-item3-code-r2-diff.md:729-735` | ✓ `-`/`+` 两行范围精确落在引用区间内 | `grep -n "marks.after_the_starting_point\|marks.after_make_filesystem"` |
| `crash_injection.rs:308`（`histories_without_any_crash_point` 字段） | ✓ 行号精确；引的注释是原注释的前半句，未带括注「（流里一个候选段都没有）」——不影响该节论证 | `sed -n '306,309p'` |
| 「`histories_without_any_crash_point` 在测试文件里零命中」 | ✓ 独立复跑同一条 grep，退出码 1（零命中） | `grep -n histories_without_any_crash_point crates/singlefs-harness/tests/second_transaction_supplement_three_crash_injection.rs` |
| `crash_injection.rs:845-847`（`step_containing` 调用） | ✓ | `sed -n '843,848p'` |
| `history.rs:2615-2639`（`apply_cold_start_recover`）、`:2619`（`recover(...)` 调用） | ✓ 两处精确，闭合 `}` 确在 2639 | `grep -n "fn apply_cold_start_recover"`；`sed -n '2615,2620p'`、`sed -n '2635,2640p'` |
| `recovery.rs:1355`（`pub fn recover`）、`:840`（`replay_journal` 的 `reader: &dyn PoolReader`） | ✓ 两处精确 | `sed -n '1353,1356p'`；`sed -n '838,841p'` |

**复跑（同一个草稿副本 `/tmp/claude-1000/m2s3i3-r2-verifier/opus-rerun/`，与 Opus 那节共用同一份 rsync 副本，
每次改动前后都核对 `diff -q` 确认副本已还原干净；Sonnet 报告本身是直接在工作区上现改现跑再原样复原，
这里改用独立副本重跑，不复用它对工作区的操作）**：

| 步骤 | 命令 | 结果 |
|---|---|---|
| K2 基线（快档量具） | `cargo test -p singlefs-harness --test second_transaction_supplement_three_crash_injection crash_injection_fast_tier_recovers_only_into_versions_the_model_committed -- --nocapture` | 「崩溃状态 93 个：…落在起点那一段里的 3 个」与全部 7 类操作计数（1/21/2/53/9/4/0）**逐字重现**报告 94-101 行引用块 |
| 变异 184（施加） | 按 `mutations.tsv:184` 第 3/4 列定点替换 `crash_injection.rs:795`，`cargo test -p singlefs-harness --lib -- crash_injection::tests::crash_points_fall_inside_the_starting_point` | `test result: FAILED. 0 passed; 1 failed…43 filtered out`，panic 于 `crash_injection.rs:1406:9`，与报告 132-135 行引用**逐字重现**（清单更长，报告用「…」省略，未失真） |
| 变异 184（复原） | 还原替换并 `touch` | `test result: ok. 1 passed; 0 failed…43 filtered out`，与报告 141-142 行**逐字重现** |
| K6 大档（500 段） | `SINGLEFS_CRASH_INJECTION_SEEDS=500 SINGLEFS_CRASH_INJECTION_OPERATIONS=24 SINGLEFS_CRASH_INJECTION_POINTS=4 cargo test -p singlefs-harness --test second_transaction_supplement_three_crash_injection crash_injection_large_tier_from_the_environment -- --ignored --nocapture` | 「历史 500 段：…一个崩溃状态都摆不出的 4 段」「崩溃状态 1926 个：…」与全部 7 类操作计数（17/423/59/985/223/96/0）**逐字重现**报告 237-246 行引用块 |

四条命令全部一次性精确重现（该测试族按写死种子基跑，属于确定性观测，不需要跑 5 轮）。

**Sonnet 报告计数**：核了 17 处引用/产物 + 4 段复跑命令 = 21 项；✓ 21 项；✗ 0 项（`848-850`/`792-798`
两处代码块比引用区间多带一行闭合符号，判定为不影响引用本身正确性的边界瑕疵，未计入 ✗）。

## 五、本地攻方转述核对表核对（`m2-supp3-item3-code-r2-local-attack-translation-audit.md`）

行号全部对工作区当前文件现查（快照里含 `crash_injection.rs`、两份测试文件、`mutations.tsv`，10 项 sha256 全 OK，
按第 2 步对快照核；快照未含 `.claude/gate.d/59-crates-mutation-replay.sh`，K4 Fact 6 那一行不算「文件:行」引用，
是核对表自陈的检索命令与结果，另见下表说明）。

### K4（种子基）

| 英文项 | 核的结果 | 命令 |
|---|---|---|
| Fact 1（`crash_injection.rs:61-64`） | ✓ 四行内容逐字对应 | `grep -n "开一个新周期要做三样\|旧的数据盘..."` |
| Fact 2（`crash_injection.rs:66-67`） | ✓ 两行内容逐字对应 | 同上 |
| Fact 3（`second_transaction_supplement_three_crash_injection.rs:74-79`，约 :80-100） | ✓ 74-79 精确；「约 80-100」核对表自己标了「约」，未冒充精确值 | `sed -n '70,105p'` |
| Fact 4（`second_transaction_supplement_three_random_history.rs:62-78`，`:32,37,42,47,52`） | ✓ 六处行号全部精确（62=fn，78=闭合，32/37/42/47/52=五个常量声明） | `grep -n "FIRST_SEED: u64 = SEED_BASE"`；`grep -n "fn the_five_sampling_tiers"` |
| Fact 5（门禁脚本 + 全仓检索，零命中） | ✓ 独立重跑同样的两条 `grep -rniE`，均零命中，与核对表描述一致 | 见下方「独立重跑」 |
| Fact 6（`.claude/gate.d/59-crates-mutation-replay.sh:4-9`） | **核不动（快照未含此文件）**：内容现读与核对表引文一致，但无法排除该脚本在腿交回之后被别的会话改过——它不在给定的 16 项快照里 | `sed -n '4,9p' .claude/gate.d/59-crates-mutation-replay.sh`（核了内容，未核「腿跑时是否就是这一版」） |
| Fact 7（`mutations.tsv:189,192`） | ✓ 与本报告第 2.1 节独立复核结果一致 | 同 2.1 |
| 「事实核查先于翻译核查」节的 189/191/192 三行 | ✓ 与本报告 2.1 节独立复核一致 | 同 2.1 |

### K5（已知红判别力）

| 英文项 | 核的结果 | 命令 |
|---|---|---|
| Fact 1（`history.rs:1241-1262`，结构体） | ✓ 范围精确（1241 空行起，含两行注释、derive、struct 定义至闭合 `}`） | `sed -n '1241,1262p'` |
| Fact 2（`history.rs:1230,1231-1239`，`leading_number_after`） | **✗ 行号错位**：实际注释在 **1231**（1230 是空行），函数体（含签名到 `.ok()`）在 **1232-1239**；核对表把注释错标成 1230。全仓（背景材料、附录、diff）检索这两句注释均零命中，不属于「误写成背景材料行号」，是单纯的行号数错 | `grep -n "label. 之后紧跟着"`；`awk 'NR==1230,1232'` |
| Fact 3（`history.rs:1279,1281-1291`，`allocation_statistic_mechanism`） | **✗ 行号错位**：实际注释在 **1280**（1279 是空行），核对表标成 1279；函数体的字段构造实际在 1283-1291（`Some(AllocationStatisticMechanism {` 起），核对表给的 1281-1291 把 `#[must_use]`（1281）与 `fn` 签名（1282）也划进「函数体」范围，比字段构造多出两行、又不含 `Some(...)` 那一行 | `grep -n "从 I-3.1 的说明文字里读机理标识"`；`awk 'NR>=1278 && NR<=1294'` |
| Fact 4（`history.rs:1265-1266,1268-1271` 与 `:1273,1275-1277`） | ✓ 四段范围全部精确（跳过中间的 `#[must_use]` 属性行，与实际代码结构一致） | `sed -n '1263,1278p'` |
| Fact 5（`walk.rs:1503-1504` 注释，`:1517` 格式串） | **✗ 引文范围偏窄**：核对表原文引了「…两者的签名…一模一样，不带这一段就只能按签名认」，这句实际横跨 **1503-1506**（1505-1506 两行），但引用只给了 1503-1504；`:1517` 精确 | `sed -n '1500,1520p' crates/singlefs-checker/src/walk.rs` |
| Fact 6/7/8（`mutations.tsv:185/186/187`，Gamma/Delta/Epsilon） | ✓ 三行六列内容与核对表描述逐字对应；185/186 必须红的测试名逐字相同（`known_red_forms_are_matched_by_mechanism_not_by_signature`），核对表「同 Fact 6」的说法坐实 | `awk -F'\t' 'NR==185||NR==186||NR==187'` |

### 运行记录与产物核对（`m2-supp3-item3-code-r2-local-attack-runlog.md`）

| 声明 | 核的结果 | 命令 |
|---|---|---|
| 四次调用退出码全 0，无作废副本 | ✓ | `ls research/prompts/ | grep -i "local-attack" | grep -i void`（零命中） |
| `wc -w` 词数 579/305/461/313 | ✓ 四份输出文件现数逐字相同 | `wc -w` 四份 `-output-s*.md` |
| K5-s1 `oov-check.py` 判「生词=1（epsilon's）」，其余三份判「生词=0」 | ✓ **本核查员把四份输出文件拷到草稿目录独立重跑** `corruption-check.py`／`oov-check.py`，判定与词数逐字重现（K5-s1 生词=1、词=epsilon's；其余三份生词=0，四份 corruption 判定全绿） | 见下方命令 |
| 输出样本未出现文件名/行号，未用 markdown 强调 | ✓ 通读四份输出样本，只出现「fact N」「row Alpha/Beta/Gamma/Delta/Epsilon/Zeta」等提示要求的编号，未见文件名、行号、`**`/`` ` ``/`*` | 直接读取四份文件 |

```
$ cd /tmp/claude-1000/m2s3i3-r2-verifier/rerun && for f in *.md; do python3 .../corruption-check.py "$f"; python3 .../oov-check.py "$f"; done
绿 …k4-output-s1.md  …words=552…          绿 …k4-output-s1.md  生词=0 拼接=0
绿 …k4-output-s2.md  …words=281…          绿 …k4-output-s2.md  生词=0 拼接=0
绿 …k5-output-s1.md  …words=440…          绿 …k5-output-s1.md  生词=1 拼接=0   生词: epsilon's
绿 …k5-output-s2.md  …words=290…          绿 …k5-output-s2.md  生词=0 拼接=0
```
（`corruption-check.py` 的分词口径与 `wc -w` 不同，故 words 数字不同，但两个工具各自的判定与 runlog 声明一致）

**本地攻方计数**：K4/K5 核对表 + 运行记录合计核了 21 处引用/声明；✓ 17 处；✗ 3 处（K5 Fact 2、Fact 3 行号错位，
K5 Fact 5 引文范围偏窄）；核不动 1 处（K4 Fact 6 引的门禁脚本不在快照里）。

## 六、汇总计数

| 腿 | 核了 | ✓ | ✗ | 核不动 / 分不清 |
|---|---|---|---|---|
| 云端攻方 Opus | 16（12 处引用/产物 + 4 段复跑） | 15 | 1（3.2 节两行秒数文本未落盘存档，判定本身不受影响） | 0 |
| 云端正推 Sonnet | 21（17 处引用/产物 + 4 段复跑） | 21 | 0（两处代码块比引用区间多带一行闭合符号，判为边界瑕疵不计入 ✗） | 0 |
| 本地攻方（K4/K5 转述核对表 + 运行记录） | 25（21 处核对表/声明 + 4 份 corruption/oov 复核） | 21 | 3（K5 Fact 2、Fact 3 行号错位；K5 Fact 5 引文范围偏窄） | 1（K4 Fact 6 引的门禁脚本不在快照里） |
| 附：主 agent 派发的两处指定复核（2.1、2.2） | 2 | 2 | 0 | 0 |
| **合计** | **64** | **59** | **4** | **1** |

## 七、没做什么

- 不判一条打中成不成立、该不该采纳；不核推理本身（K1/K2/K3/K6 的结论、三条改法该不该收严、K4/K5 问答里的判断），
  只核引用、产物与复跑。
- Opus 报告 2.2 节表格里的组合数学（如「9 / 262143」「171 / 262143 ≈ 0.065%」）与「期望命中 ≈ 4×10⁻⁴」这类算术，
  按分工属于推理本身，未重算；只核了它引用的源码行号与产物数字（`crash_injection.rs:869/871`、直方图原始输出）。
- Opus 报告未跑的三条改法（甲乙丙）与「没打中」的形状（第五节）未实现、未复跑——报告自己也写明未实现，
  这条腿本轮没有产出可核的产物。
- Sonnet 报告未逐条复跑 `mutations.tsv` 里本轮新增的另外 36 条变异（报告自陈只复核了与 K2 直接相关的第 184 行），
  本核查员同样只复核了这一行。
- Opus 报告变异 C 的「段长 14 全枚举」（1454 秒那一次）与大档（500 段×40 步×8 点）未复跑：前者挂钟代价过高、
  后者报告自己也没跑；`.claude/kb/checks-owed.md` 因暂存区被别的会话重排，未对它做超出 C284/C397 两行之外的
  逐行核对。
- 未判断 K5 Fact 2/3/5 三处行号问题该不该、怎么改（是核查员的观测，改不改由主 agent 定）。
- 未核 K4/K5 提示文件本身是否符合「本地腿的提示按格数拆」等派发规则，只核了提示与转述核对表引的源码内容对不对。
- `.claude/kb/invariants.md` 不在快照的 16 项里，且三条腿正文均未引用它的具体行号，未触发额外核查。

## 八、复跑留存

草稿目录 `/tmp/claude-1000/m2s3i3-r2-verifier/`：
- `self-test/crash_injection.rs`：判别力自证用的行号偏移副本。
- `opus-rerun/`：`rsync` 自工作区的整仓副本，Opus/Sonnet 两节的全部复跑都在这个副本上做，收尾时已用
  `diff -q` 核对 `crash_injection.rs`、`transaction.rs` 两个被改动过的文件都已还原到与工作区逐字相同。
- `rerun/`：本地攻方四份输出样本的副本及 `corruption-check.py`/`oov-check.py` 复跑。
- `*.log`：各段复跑的原样输出（`probe1-rerun.log`、`mutA-test1.log`、`mutA-test2.log`、`mut67-test.log`、
  `mut67plusA-test.log`、`sonnet-k2-baseline.log`、`sonnet-mut184.log`、`sonnet-mut184-restored.log`、
  `sonnet-k6-large.log`）。
- 工作区本身：`git status --porcelain` 对 `crash_injection.rs`、`transaction.rs` 两文件核对，均无本核查员留下的改动
  （复跑全部在 `/tmp` 副本里做，未碰工作区）。
