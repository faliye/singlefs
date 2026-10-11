# m3-batch27-code-r1 正推腿报告（Sonnet，K1 与 K3）

日期 2026-10-06。立场：正推。引代码一律写冻结副本里的全路径:行号（行号都是在冻结副本 `/tmp/claude-1000/m3-batch27-code-r1/tree/` 里现查的；主工作区这几份之后被形式化线改过，行号对不上主工作区）；引 kb 写 kb 文件自己的行号。
方案按 `crates/` 今天的实现来谈：下文每格都是读冻结副本里的代码得出，没有从背景材料里数行号。

## 0 前置：被判文件与改动原样

被判 5 份 sha256 在冻结副本上复核（命令 `cd /tmp/claude-1000/m3-batch27-code-r1/tree; sha256sum -c`，输入是 `research/prompts/m3-batch27-code-r1-snapshot/crates-sha256.txt`），原样输出：

```
crates/singlefs-checker-tier/src/crash_judge_checker.wgsl: OK
crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs: OK
crates/singlefs-checker/src/walk.rs: OK
crates/singlefs-harness/tests/checker_known_bad_images.rs: OK
crates/mutations.tsv: OK
```

改动原样 `research/prompts/_m3-batch27-code-r1-diff.md` 是不是全部改动：把它的 crates.patch 段套在主工作区暂存区（`git show :<路径>`，第 27 批合入之前的样子）那 4 份文件上，逐份与冻结副本 `cmp`，原样输出：

```
same: crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
same: crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
same: crates/singlefs-checker/src/walk.rs
same: crates/singlefs-harness/tests/checker_known_bad_images.rs
```

变异表同法：暂存区的 `crates/mutations.tsv`（2936 行）与冻结副本只差第 2934、2935 两行与追加的第 2937–2942 行，且这 8 行与 diff 文件里 `mutations-replacements.tsv`、`mutations-append.tsv` 两段逐行相同（python 比对，两项都输出 True）。⇒ 这一轮的改动面就是 diff 文件写的，没有夹带。
推翻它的现象：冻结副本里某份文件与「暂存区版 + diff」不同。

模型与脚本：`research/prompts/m3-batch27-code-r1-sonnet-model/`（`rerun.sh`、`chain_model.py`、`apply_row.py`、`expected.txt`）。口径与结果在第 6 节。

## 1 K1-J1：GPU 实例表链上限

判定：**一致**（256 片、257 片两点上「判了的同判、判不了的回落 CPU 后同判」，站点号不撞）。另附一处同族观察 O-1（第 1.6 条），不算 J1 改动本身的错。

### 1.1 常量与出口怎么走

- `/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1580`：「const INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED: u32 = 256u;」
- `/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1601`：「for (; page < INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED; page = page + 1u) {」
- `/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1686`：「if (page == INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED) {」
- `/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1687`：「undecidable = UNDECIDABLE_INSTANCE_TABLE_CHAIN_LONGER_THAN_THE_PAGES_WALKED;」

循环体里的提前出口全是 `break`（读不出、`w_visit` 判走过、容器不 ok、身份不对、`if (chain != 1u) {` 都在 `page` 自增之前离开循环；`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1680`：「if (chain != 1u) {」），所以 `page` 只有在 256 轮都走到链指针记录说「有下一片」时才会等于 256。逐点：

| 链 | GPU 内核（第 1589 行起的 `w_walk_instance_table_chain`） | CPU（`walk_instance_table_chain`，没有页数上限） | 同不同 |
|---|---|---|---|
| 255 片 | 第 255 轮（page = 254）`chain == 0u` 置门、break，page = 254 | 第 255 片 `LastPage` 置门 | 同 |
| 256 片 | 第 256 轮（page = 255）`chain == 0u` 置门、break，page = 255 ≠ 256，不记判不了 | 第 256 片 `LastPage` 置门 | 同（对拍用例两边都比） |
| 257 片 | 第 256 轮（page = 255）`chain == 1u`，自增到 256，循环条件落空，`page == 256`，记判不了（原因字 3 \| 61 << 8）；门没置 | 第 257 片 `LastPage` 置门 | GPU 判不了、交 CPU：回落后同判 |
| 256 轮里某片读不出 / 走过 / 身份不对 / 链记录不合法 | 在该轮 break，page < 256，不记判不了，门不置 | 同一处 break，门不置 | 同 |
| 257 片、第 257 片读不出 | page = 256，记判不了（保守） | 第 257 片读不出，门不置 | GPU 判不了、交 CPU 后同判 |

CPU 一侧的依据：`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker/src/walk.rs:2877`：「fn walk_instance_table_chain(」 的循环是 `/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker/src/walk.rs:2887`：「loop {」，出口只有读不出、走过、容器不合法、身份不对、链记录 `Malformed`、`LastPage`；置门在 `/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker/src/walk.rs:2965`：「AbandonedRootsFilteringOfTheCandidateSet::FilteredByTheNewestRootsInstanceTable;」，仅 `LastPage` 且是挂载根那一张时。GPU 的置门 `/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1676`：「if (chain == 0u) {」 在 `judge_rows` 块里，同条件。

纸面模型 `research/prompts/m3-batch27-code-r1-sonnet-model/chain_model.py`：把上面 GPU 循环的出口逐支抄成 Python、再写一份 CPU 版，在 32 种链（长 1、2、255、256、257、258、1000；末片记录不合法；第 1、254–257 片读不出；第 1、255、256 片已走过）上比。末行原样：`cases=32 violations=0`（「违例」= GPU 判了而与 CPU 不同）。注意它是我对 wgsl 控制流的手抄，与我读 wgsl 共用同一份理解，不是独立路径；独立的一条是下面第 1.3 条的 harness 变异与第 1.5 条的容量核对。
推翻它的现象：GPU 对拍里 256 片那条流上出现任何一个判不了或不一致状态，或 257 片那条流上没有一个状态记原因字 `0x3d03`。**GPU 对拍我不跑（复核不了：checker 档包与 GPU 卡）**。

### 1.2 站点号不撞

命令（冻结副本 `crates/singlefs-checker-tier/src/` 下全部 `*.wgsl`）：`grep -ohE '\(([0-9]+)u << 8u\)' *.wgsl | sed -E 's/\(([0-9]+)u.*/\1/' | sort -n | uniq -c | awk '$1>1{print "dup",$2}'`，输出为空（没有重复）。同一批站点号排序后：`1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32 33 34 35 36 40 41 42 43 44 45 46 47 48 49 52 53 54 57 58 59 60 61`，61 只出现一次：`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1585`：「const UNDECIDABLE_INSTANCE_TABLE_CHAIN_LONGER_THAN_THE_PAGES_WALKED: u32 = UNDECIDABLE_TOO_MANY_ENTRIES | (61u << 8u);」。「条目太多」类的低 8 位是 3：`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:80`：「const UNDECIDABLE_TOO_MANY_ENTRIES: u32 = 3u;」，与测试里的 `3 | (61 << 8)` 同值（`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:6538`：「const GPU_UNDECIDABLE_INSTANCE_TABLE_CHAIN_LONGER_THAN_THE_PAGES_WALKED: u32 = 3 | (61 << 8);」）。
推翻它的现象：有第二处 `(61u << 8u)`，或测试里的 3 \| (61 << 8) 与 wgsl 的常量算出不同的数。

### 1.3 对拍用例与变异行

- 256 片：`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:7492`：「fn the_gpu_judge_agrees_with_the_cpu_on_a_newest_instance_table_chain_of_as_many_pages_as_the_gpu_walks(」。流造对了先在 CPU 一侧钉（整条落盘的镜像上 I-9.29 红，`record_..._spread_over_pages` 里的断言），再逐状态比；`compare_on_every_state_of_these_segments` 把「判不了」算进不一致（`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:2117`：「if undecidable」），所以这条流上任何一个判不了的状态都会让它红。
- 257 片：`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:7526`：「fn the_gpu_judge_leaves_to_the_cpu_a_newest_instance_table_chain_longer_than_the_pages_it_walks() {」。逐状态：原因字为 0 的比违例位；原因字等于常量 `/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:7555`：「GPU_UNDECIDABLE_INSTANCE_TABLE_CHAIN_LONGER_THAN_THE_PAGES_WALKED => {」 的记数，并要求至少一个这样的状态上 CPU 红 I-9.29；别的原因字一律算不一致；再用 `gpu_verdict_items`（判不了交回 `None`）模拟流水线回落 CPU，比回落后的红项与 CPU 的 `red_items`。我读了 `crates/singlefs-checker-tier/src/crash_amplification.rs` 里 `gpu_verdict_items` 的开头（原因字非 0 就交回 `None`），与测试注释一致。
- 变异：`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/mutations.tsv:2938`：「undecidable = UNDECIDABLE_INSTANCE_TABLE_CHAIN_LONGER_THAN_THE_PAGES_WALKED;」 把记判不了那一行换成注释；点名的测试就是 257 片那条（第 6 段与第 5 段筛都对得上，见第 4 节核对）。变异之后 257 片的链走满上限时门不置、原因字 0，CPU 红 I-9.29 而 GPU 位不红，会撞 `0 =>` 那一支的违例位比较与「至少一个判不了」两处断言。**GPU 那几行我不跑，复核不了。**

### 1.4 链造得对不对（读 `rebuild_the_newest_instance_table_as_a_chain`）

`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:6685`：「fn rebuild_the_newest_instance_table_as_a_chain(」 把单片表改成 `rows_of_each_page.len()` 片：第 k 片的容器号写成 k、末片链记录说没有下一片、其余指下一片（位置条目的整单元 CRC 按各片重封后的字节算）；新片写在第 200 000 槽起每片两槽（`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:6573`：「const FIRST_SLOT_OF_THE_INSTANCE_TABLE_PAGES_AFTER_THE_FIRST: u64 = 200_000;」），写之前断言流里没写过、基镜像里全零；`pages.count()` 是 256 / 257，所以 256 流的末片在第 256 轮、257 流的第 256 片仍指下一片。⇒ 两条流恰好站在 `page` 等于 255 / 256 的两侧。
一处弱点：测试里的 `GPU_INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED = 256`（`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:6534`：「const GPU_INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED: usize = 256;」）与 wgsl 第 1580 行的 256 是两份手写的数，没有编译期或运行期的连接：改一边忘了另一边，两条流不再站在上限两侧，但 256 那条仍只比「同判」、257 那条会因没有判不了的状态而红，所以只有一边会红。不是错，是提醒。

### 1.5 容量核对（256 片流上 GPU 会不会因别的表满而判不了）

每片走读记两条位置的引用与落点：`w_note_pointer_references`（`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1251`：「fn w_note_pointer_references(p: Ptr, span: u32) {」）每片记 ≤ 2 个引用、2 个版内落点、2 个「下面」条目、≤ 2 个校验和、≤ 2 个外观落点；256 片 ≤ 512 条。对照上限：访问表 `/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:312`：「const C_MAX_VISITED: u32 = 2048u;」、引用表 `/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:306`：「const C_MAX_REFERENCES: u32 = 2048u;」、「下面」表 `/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:326`：「const C_MAX_BELOW: u32 = 4096u;」、外观落点 `/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:751`：「const C_APPEARANCE_MAX_PLACEMENTS: u32 = 2048u;」。512 加这条流里别的单元（新池新建文件，几十个）都在上限之内。存量没逐项数，GPU 不跑，所以只记「读码推得过」。
推翻它的现象：256 片那条流在 GPU 上出现原因字低 8 位为 3、站点号是 1、3、4、7、40 之一（引用表、访问表、版内落点、「下面」表、外观落点满了）的状态。

### 1.6 O-1（观察，不算 J1 改动的错）：同一个 256 在扫描内核里还有两处硬写、走满没有出口字

J1 只改了走读内核里这一处。同族的写法还在：

- `/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:477`：「fn r_instance_pages_after_first(first: Ptr) {」 与 `/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1093`：「fn r_instance_rows(record_base: u32) -> bool {」 各一个 `/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:480`：「for (var step = 0u; step < 256u; step = step + 1u) {」（第二个在 1097 行），走满之后前者置 `r_complete = false`、后者 `return false`，都没有 `undecidable`。
- 对照：恢复内核 `/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl:229`：「for (var page = 0u; page < 256u; page = page + 1u) {」 与公共段 `/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:2777`：「for (var page = 0u; page < 256u; page = page + 1u) {」 走满后记 `UNDECIDABLE_TOO_DEEP`；CPU 的 `instance_table_rows_of_root_without_judging`（`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker/src/walk.rs:5794`：「fn instance_table_rows_of_root_without_judging(」）是无上限的 `loop`（`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker/src/walk.rs:5801`：「loop {」）。
- 推出的差异（读码推，没跑）：根环里某条自证过的根 R 的实例表链 ≥ 257 片，且 R 不在回退候选集里（所以走读内核不走它的链、拿不到 J1 的 61 号），表里有一行 T 不小于 R 的 checkpoint_txg：CPU `/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker/src/walk.rs:8070`：「let Some(rows) = instance_table_rows_of_root_without_judging(reader, &root.record_bytes)」 读全链、判 I-3.12 红；GPU `judge_instance_table_rows_against_the_roots_referencing_them`（`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1910`：「if (!r_instance_rows(root.base)) {」）走满 256 步 `return false` 后 `continue`，不置 I-3.12。I-7.9 那一处同形：`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1329`：「if (!r_instance_rows(raising.base)) {」，在对 `root_count` 条根循环的 `judge_rollback_floor_raises_against_their_ceilings` 里。候选集里的根若也被走读内核走过链，则被 61 号兜住；R 在候选集外时没人兜。
- 这一轮的两条 257 片流只改最新根的表，碰不到这一形。是否算打中、要不要改，交主 agent / 攻方腿现查。
推翻它的现象：走读内核对根环里每一条自证过的根都走一遍它的实例表链（那样 R 一定经过 61 号），或 I-3.12 / I-7.9 只对候选集里的根判。

## 2 K1-J2：关门那一侧两条流与 `chain == 0u` 变异

判定：**一致**。两条流都走到关门那一支；`chain == 0u` 那行有变异，由「链指针记录不合法」那条流抓；「第 1 片读不出」那条流没有自己的变异（缺口，见 2.3）。

### 2.1 门怎么开、两条流怎么关

门只在 `chain == 0u` 时置：`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1676`：「if (chain == 0u) {」，下一行 `/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1677` 是置 `w_newest_instance_table_chain_reached_its_last_page = true`。`chain` 由 `w_chain_record_kind` 算（`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:603`：「fn w_chain_record_kind(base: u32, o: u32, width: u32) -> u32 {」）：宽不是 88 或种类不是 1 → 2；有无下一片 0 且指针全零 → 0；有无下一片 1 且指针不全零 → 1；其余 → 2。

- 「链指针记录不合法」流（`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:7442`：「fn the_gpu_judge_agrees_with_the_cpu_that_the_one_birth_invariant_is_not_compared_once_the_chain_record_of_the_newest_instance_table_is_invalid(」）：唯一一片里去掉实例 2 那一行、链记录写「有下一片」而指针全零（`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:6639`：「fn chain_record_saying_a_next_page_without_a_pointer() -> Vec<u8> {」）。`chain_record_kind` 对这条记录交回 2，`chain != 1u` break，门不置。CPU：`chain_record_view` 交回 `Malformed`，`break`，`abandoned_roots_filtering` 保持默认，I-9.29 报「最新根的实例表读不出」那句部分没判（`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker/src/walk.rs:9748` 起的 `match`）。
- 「第 1 片读不出」流（`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:7453`：「fn the_gpu_judge_agrees_with_the_cpu_that_the_one_birth_invariant_is_not_compared_once_the_second_page_of_the_newest_instance_table_does_not_read_back(」）：`[other_rows, rows_of_the_abandoned_instance]` 两片，第 1 片两份各翻一个载荷字节不重封（`PagesAfterTheFirstInTheFlow::BothCopiesFlippedWithoutResealing`）。GPU 第 0 轮 `chain == 1u` 继续，第 1 轮 `w_read_referenced_unit` 读不出走 `s.version == NONE` 的 break，门不置；CPU 同处 break。

共用的断言函数 `/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:7392`：「fn assert_the_gpu_judge_agrees_with_the_cpu_that_the_one_birth_invariant_is_not_compared_once_the_newest_instance_table_stops_before_its_last_page(」 先在 CPU 一侧钉两件事：改之前整条落盘的镜像上 I-9.29 评估过且成立（`Holds`）；改之后报 `PartiallyJudged` 且说明以「最新根的实例表读不出」开头；再要求 CPU 红 I-9.29 的状态存在、逐状态 GPU 与 CPU 不一致数为 0。门误开时：候选集里没有实例 2 那一行，被抛弃的 C 与现行线同号不同出生代，GPU 会红 I-9.29 而 CPU 不红，不一致数 > 0，用例红。

### 2.2 变异

`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/mutations.tsv:2939`：「if (chain == 0u) {\n                w_newest_instance_table_chain_reached_its_last_page = true;」 换成 `if (chain != 1u) {\n ... = true;`，点名「链指针记录不合法」那条流。变异后这条流上 `chain == 2u` 也置门，门开，GPU 红 I-9.29 而 CPU 不红 ⇒ 点名的流会红。锚点在 `crash_judge_checker.wgsl` 里恰好命中一次，参数筛（第 5 段的测试名过滤）筛得到第 6 段点名的测试（`apply_row.py --check` 的输出见第 6 节）。

### 2.3 缺口（观察）

「第 1 片读不出」那条流没有变异行点名它。变异 2939 对它无效（第 0 片 `chain == 1u`，`chain != 1u` 不成立；第 1 片读不出就 break，走不到置门那行）。能让这条流红的改法是「读不出那一支也置门」之类，表里没有。不影响这一轮的正确性，影响的是这条流抓不抓得住以后的回退。
推翻 2.1 的现象：两条流之一在 GPU 上门开了而对拍不红；推翻 2.2：把锚点改成 `!= 1u` 后点名的流仍绿（我跑不了 GPU，只能读码推）。**GPU 对拍与 GPU 那几行变异我不跑，复核不了。**

## 3 K1-J3：类型段不认得时不读孩子

判定：**大体一致，有一处不逐项相同**：读不读（按长度读）、登记几槽（层级 1 两槽、≥ 2 一槽）、判什么（I-9.2 红）、记不记走读失败（不记）与墓碑树、谱系树那一支都相同；不同的一处是 inode 树在这一支之前仍先调一遍「孩子自证出的类」那个只读探测，墓碑树、谱系树不调。该不同不是第 27 批引入的（探测是第 24–26 批的老写法），且它让 inode 树这一侧登记的槽数更贴近孩子真占的槽。

### 3.1 逐项对表

| 项 | 墓碑树、谱系树（`walk_the_entries_of_a_container_tree_node`） | inode 树（`walk_inode_internal_node`，J3） | GPU（inode / 墓碑、谱系） |
|---|---|---|---|
| 认出类型段不认得 | `/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker/src/walk.rs:2199`：「ContainerTreeEntryType::Unregistered(_) => None,」 | `/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker/src/walk.rs:2505`：「InodeInternalEntryType::Unregistered(_) => None,」 | `/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2646`：「if (class_the_inode_entry_type_names == NONE) {」 / `/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3088`：「if (entry_type != record_type) {」 |
| 先探测孩子自证的类？ | 不探测：`class_the_entry_type_names` 为 `None` 时 `/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker/src/walk.rs:2201`：「if let Some(class_the_entry_type_names) = class_the_entry_type_names {」 不进 | 探测：`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker/src/walk.rs:2507`：「if let Some(child_class) = child_unit_class_other_than_the_entry_type_names(」 对 `None` 也调；自证出类就按那一类占的槽登记、I-9.2 红、`continue` | inode：`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2637`：「let class_of_the_inode_child = w_child_unit_class_other_than_the_entry_type_names(child, class_the_inode_entry_type_names);」，对 `NONE` 也调；墓碑、谱系不调 |
| 没自证出类：按长度读孩子？ | 不读 | 不读：`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker/src/walk.rs:2534`：「InodeInternalEntryType::Unregistered(unregistered_type) => {」 起 `continue`，在 `read_referenced_unit` 之前 | 不读 |
| 登记几槽 | `/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker/src/walk.rs:2257`：「let span_in_slots = if node.level >= 2 { 1 } else { 2 };」 | `/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker/src/walk.rs:2539`：「let child_span_in_slots_by_the_level = if node.level >= 2 { 1 } else { 2 };」 | inode：`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2648`：「if (root.level >= 2u) {」；墓碑、谱系 `/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3090` 的 `span_of_the_unregistered_child` |
| 判什么 | `identity_invariant` 红 | I-9.2 红 | `/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2652`：「w_judge(INVARIANT_I_9_2, false);」 / `/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3095`：「w_judge(identity_invariant, false);」 |
| 记走读失败 | 不记 | 不记 | 不记（`continue`，没有 `w_failures`） |

不同的那一格（第 2 行）的后果：inode 树类型段不认得而孩子头里的类标签登记过、按那一类的长度自证得过时，登记的槽数按孩子自己的类（码 2 一槽、码 1 / 3 两槽）、判 I-9.2 红；墓碑、谱系同一形登记按层级定的槽数。孩子真占的槽与层级定的不符时（层级 1 下是码 2 节点），墓碑、谱系多登记一槽。I-9.2 的 kb 条款只说「其余判该条目损坏」（`.claude/kb/invariants.md:289`：「其余判该条目损坏」），没有定槽数；I-9.17、I-9.23 两行写「与 I-9.2（条目身份与子头相符） 同形」。

### 3.2 GPU 同判

走读内核 inode 根 `/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2596`：「fn w_walk_inode_root(root: Node, tree: vec2<u32>, root_device: u32, root_slot: vec2<u32>, fsid_low: vec2<u32>) {」 里，探测（`NONE` 也传进去）先于新分支；新分支登记引用后 `w_judge(INVARIANT_I_9_2, false)`、`continue`，不碰 `w_failures`，之后「读不出」那一支只剩类型段 0 判身份三段为 0（`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2661`：「w_judge(INVARIANT_I_9_2, is_zero64(identity_birth_tree) && is_zero64(identity_container) && is_zero64(identity_birth));」），与 CPU `judge_an_inode_entry_whose_child_does_not_read_back`（`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker/src/walk.rs:4171`：「fn judge_an_inode_entry_whose_child_does_not_read_back(」，参数改成不含 `Unregistered` 的 `RegisteredInodeInternalEntryType`，`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker/src/walk.rs:141`：「enum RegisteredInodeInternalEntryType {」）同。类型段 0 读得出的那一支原来的 `else if (record_type == 0u)` 缩成 `else`，只剩类型段 0 会走到（类型段 2 在上面，类型段不认得的在前面 `continue` 了），仍 `UNDECIDABLE_TOO_DEEP`（GPU 一侧没有往下一层 inode 内部节点走的形状，CPU 会往下走；这是第 27 批之前就有的老限制，J3 没动）。
推翻它的现象：对一形「类型段不认得 + 孩子头里是登记过的类且自证得过」，CPU 与 GPU 登记的槽数不同（读两边的探测函数 `walk.rs:4136`、`wgsl:2981`：我读出的是逐支对应）；或墓碑树、谱系树那一支也被改成先探测（那样第 3.1 表第 2 行的差异就没了）。

## 4 K1-J4 与测试：期望、变异行、替换的两行

判定：**一致**；三处弱点列在 4.3。

### 4.1 坏镜像与对拍流的期望

harness 坏镜像（`crates/singlefs-harness/tests/checker_known_bad_images.rs`）：

- 类型段 0 带身份、孩子读不出：`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-harness/tests/checker_known_bad_images.rs:10914`：「fn an_inode_tree_entry_of_type_zero_with_an_identity_over_a_child_that_does_not_read_back_reddens_the_inode_entry_invariant(」。只断言 I-9.2 的说明含「孩子两份都读不出」与「（要三段为 0）」，没有钉整张红表（第 26 批老写法，J4 没改它，只把辅助函数 `image_whose_inode_root_entry_names_a_child_that_does_not_read_back`（`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-harness/tests/checker_known_bad_images.rs:10890`：「fn image_whose_inode_root_entry_names_a_child_that_does_not_read_back(」）改成收镜像而不收标签）。
- 类型段 7、孩子读不出：`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-harness/tests/checker_known_bad_images.rs:10944`：「fn an_inode_tree_entry_with_an_unregistered_type_over_a_child_that_does_not_read_back_reddens_only_the_inode_entry_invariant(」。期望写全了：同一张新池镜像上先造「类型段 7、孩子完好」的对照，钉红表恰为 `["I-9.2"]`（`...:10964`：「violated_invariants(&verdicts_with_a_readable_child),」）；孩子读不出那张钉红表恰为「I-1.11（叶容器头里的类标签改成 9 自己带的）+ 对照所红（I-9.2）」（`...:10976`：「MAPPING_KEY_CLASS_TAG_MATCHES_THE_UNIT_HEADER,」），因此 I-2.1、I-4.8、I-7.2、I-7.4（走读失败连带）与 I-3.1（槽数少登记连带）都被「恰为」排除；再各钉 I-1.11 与 I-9.2 的说明文字。
  路径都是 `crates/singlefs-harness/tests/checker_known_bad_images.rs`。

GPU 对拍流（`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`）：

- 类型段 0 带身份：`...:7358`：「fn the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_type_zero_with_an_identity_names_a_child_that_does_not_read_back(」，期望 `...:7367`：「&["I-9.2", "I-2.1"],」。CPU 侧要红的两条写了；走读失败带出的 I-4.8、I-7.2、I-7.4 没单独钉红，靠逐位比较覆盖。
- 类型段 7：`...:7375`：「fn the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_an_unregistered_type_names_a_child_that_does_not_read_back(」，走 `assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_the_identity_without_a_walk_failure`：CPU 红 I-9.2，I-2.1 / I-4.8 / I-7.2 / I-7.4 一个状态都不红，逐位相同。CPU 侧「I-3.1 不红」「I-1.11 红」没在这条流里钉，由 harness 静态镜像那张「恰为」表钉；流上 GPU 若少登记槽，I-3.1 会比 CPU 多红，逐位比较会抓（变异 2942 点名这条流）。
- 辅助：`corrupt_units_and_carry_their_whole_unit_checksums_up_to_the_root`（`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:582`：「fn corrupt_units_and_carry_their_whole_unit_checksums_up_to_the_root(」）的 `LeftAsBeforeTheChange`：被改单元自己不重封（头里类标签 9，两道校验和不动），指着它的位置条目换成改后的整单元 CRC 并一路重封到根。这样「孩子读不出」成立而其余引用链自洽，坏法只剩这一处。

### 4.2 变异行点名的测试与参数筛、替换的两行

命令与输出（`apply_row.py --check` 数锚点在文件里的命中次数、看第 5 段参数筛里有没有一条是第 6 段测试名的子串）：

```
row=2934 anchor_hits=1 named_test_matched_by_a_filter=1 test_target=checker_known_bad_images
row=2935 anchor_hits=1 named_test_matched_by_a_filter=1 test_target=checker_known_bad_images
row=2937 anchor_hits=1 named_test_matched_by_a_filter=1 test_target=checker_known_bad_images
row=2938 anchor_hits=1 named_test_matched_by_a_filter=1 test_target=crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state
row=2939 anchor_hits=1 named_test_matched_by_a_filter=1 test_target=crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state
row=2940 anchor_hits=1 named_test_matched_by_a_filter=1 test_target=crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state
row=2941 anchor_hits=1 named_test_matched_by_a_filter=1 test_target=crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state
row=2942 anchor_hits=1 named_test_matched_by_a_filter=1 test_target=crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state
```

另：全表无重名行（`awk` 取第 1 段 `sort | uniq -d` 输出为空）、每行恰六段（`NF != 6` 的行数 0）。

harness 档三行（2934、2935、2937）我在冻结副本的拷贝上逐行改坏并跑点名的测试（编译 6G、跑 4G、线程 15，经 `scripts/hosts/run-with-memory-cap.sh` 与 `capped.sh`），原样输出（`expected.txt` 里取）：

```
harness_run: baseline test an_inode_tree_entry_of_type_zero_with_an_identity_over_a_child_that_does_not_read_back_reddens_the_inode_entry_invariant ... ok
harness_run: baseline test an_inode_tree_entry_with_an_unregistered_type_over_a_child_that_does_not_read_back_reddens_only_the_inode_entry_invariant ... ok
harness_run: mutated-row-2934 test an_inode_tree_entry_of_type_zero_with_an_identity_over_a_child_that_does_not_read_back_reddens_the_inode_entry_invariant ... FAILED
harness_run: mutated-row-2935 test an_inode_tree_entry_with_an_unregistered_type_over_a_child_that_does_not_read_back_reddens_only_the_inode_entry_invariant ... FAILED
harness_run: mutated-row-2937 test an_inode_tree_entry_with_an_unregistered_type_over_a_child_that_does_not_read_back_reddens_only_the_inode_entry_invariant ... FAILED
```

即对照两条绿；三行各被各自点名的那条测试抓（每行改坏时另一条仍是 ok，没有串）。2937 红在 I-3.1：我取了失败信息里那一条 `("I-3.1", Violated("盘 0：记账的已分配 Some(278528)…`，与变异行标签说的「I-3.1 连带红」一致。2935 红在红表变成 `["I-1.11", "I-2.1", "I-4.8", "I-7.2", "I-7.4"]`（左）对 `["I-1.11", "I-9.2"]`（右）。

替换的两行（第 26 批旧行取自暂存区 `git show :crates/mutations.tsv`，逐段对新旧）：

- `/tmp/claude-1000/m3-batch27-code-r1/tree/crates/mutations.tsv:2934`：只换了第 3 段（原文）里的一个词：`entry_type,` → `registered_entry_type,`；第 1、2、4、5、6 段与旧行相同。**是只换锚点。**
- `/tmp/claude-1000/m3-batch27-code-r1/tree/crates/mutations.tsv:2935`：**不只是换锚点**。第 3 段原文从「调用 `judge_an_inode_entry_whose_child_does_not_read_back` 那一块」换成了新分支里 `self.judgements.judge("I-9.2", false, ...)` 那一块；第 4 段替换文从「注释」换成了「读孩子（`read_referenced_unit(... node_bytes() ...)`）并 `walk_failures.push`」，同时把 I-9.2 的判整块拿掉；第 6 段测试名也随改名换了：`...reddens_the_inode_entry_invariant` → `...reddens_only_the_inode_entry_invariant`（第 1 段标签文字没动，仍是「类型段不认得的条目只记 I-2.1 与走读失败」，与新替换文的语义「按 16 KiB 读、记走读失败」相符）。它现在是一行新变异，不是旧行换锚点。旧名在 `crates/` 里再无引用（`grep -rn` 冻结副本 `crates` 零命中；仓里 `*.md`、`*.tsv`、`*.rs`、`*.sh` 只在三份冻结证据里：`_m3-batch26-code-r1-diff.md`、`m3-batch26-code-r1-sonnet-output.md`、本轮 `_m3-batch27-code-r1-diff.md`）。

### 4.3 弱点（不是错）

1. `/tmp/claude-1000/m3-batch27-code-r1/tree/crates/mutations.tsv:2935` 同时删 I-9.2 的判与加读孩子两件事，红表里同时少了 I-9.2、多了 I-2.1 等；它抓得住，但分不清到底是哪一半被测试抓的，不是逐点变异。
2. 第 1.4 条的两份 256；第 2.3 条「第 1 片读不出」流没有自己的变异。
3. 类型段 0 带身份那条 harness 镜像只钉 I-9.2 的说明文字，不钉整张红表（旧写法，J4 没动）。

## 5 K3：被判的 5 份文件，每份一格

| 文件（冻结副本里） | 判到了什么 | 判定 |
|---|---|---|
| `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` | sha256 对上；改动 5 处全读：J1 的常量（1579–1585 行）、循环头（1599–1601 行）、循环后记判不了（1686–1688 行），J3 的新分支（2646–2655 行）与「读不出」那支删掉 `else if`（`w_judge(INVARIANT_I_9_2, false)` 一句）、类型段 0 的 `else if` 缩成 `else`。逐支对 CPU（第 1、3 节）；站点号去重（1.2）。另读了未改的邻居找同族（O-1） | 一致；O-1 观察 |
| `crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` | sha256 对上；读了新增的辅助（`OwnChecksumsOfTheChangedUnits`、`rebuild_the_newest_instance_table_as_a_chain`、`Comparison` 新增两个字段）、四条 J1/J2 流、两条 J4 流与比较处 | 一致；弱点 1.4、2.3、4.1 |
| `crates/singlefs-checker/src/walk.rs` | sha256 对上；读了 `RegisteredInodeInternalEntryType`、`walk_inode_internal_node` 的新分支与签名改动、`judge_an_inode_entry_whose_child_does_not_read_back`，对表墓碑树、谱系树（3.1） | 大体一致；探测一格不同（3.1 第 2 行，老写法） |
| `crates/singlefs-harness/tests/checker_known_bad_images.rs` | sha256 对上；两条测试与辅助都读了，并在拷贝上跑绿（对照）与三行变异（4.2） | 一致 |
| `crates/mutations.tsv` | sha256 对上；与 diff 两段逐行相同；8 行锚点各命中 1 次、参数筛筛得到、无重名、均六段；替换的两行一行只换锚点一行是新变异（4.2） | 一致；2935 行不只换锚点 |

## 6 跑过的命令与原样输出、模型

负载：开工前 `ps -o pid,args -u "$(id -u)"` 看到别的会话在跑 `cargo test`、`cargo-clippy`、`cargo check` 与 rocksdb 的 c++ 编译；没有 `qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`，所以照常跑，全部加 `nice -n 19`，没有等锁。

- 编译：`nice -n 19 bash scripts/hosts/run-with-memory-cap.sh 6G bash scripts/hosts/capped.sh 15 cargo test --offline --locked -p singlefs-harness --test checker_known_bad_images --no-run`（`RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON` 设了理由：包装按已记的实测峰值 1446M 拒了 6G，派发提示规定显式 6G）。跑：同形 `4G`，参数 `-- stale_copy_of_the_child child_that_does_not_read_back`。编译目录是我自己的 `CARGO_TARGET_DIR`。
- 链模型：`python3 chain_model.py` 末行 `cases=32 violations=0`。
- `rerun.sh` 在空目录从头跑（把模型目录拷到新位置、新输出目录、新 target）：

```
SINGLEFS_REPO=/home/fy5090/code/singlefs M3B27_OUT=<空目录> bash model/rerun.sh
COMPARE-OK (56 行)
```

`compare` 是脚本末尾把 `results.txt` 与 `expected.txt` 各自 `sort` 之后 `diff`；脚本自己不依赖 `SINGLEFS_REPO` 之外的仓路径，冻结副本路径用 `M3B27_TREE`（默认 `/tmp/claude-1000/m3-batch27-code-r1/tree`）。文件 sha256：`rerun.sh` 3040b41688affe6df9e0b5609419be45f75cb1eca423db6cf1a2c0c484deb357，`chain_model.py` 02e41178deff697d6c41017ae944073095ca72cb3e426f65c48eb472c2e12c9c，`apply_row.py` 1710b649f28861f71d791147f221f7093ddfe89e6be98ceb215aa5cebeb3fea1，`expected.txt` c6b2fd48ed01636428e09907a43cc50de130146c5343b42e846f4f0f6f5180b5。

## 7 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1 J1 链上限 256 / 257 | 一致 | 恰 256 片两边都判、257 片 GPU 记 `0x3d03` 交 CPU 后同判；站点号 61 全 wgsl 唯一；GPU 对拍复核不了 |
| K1 J1 观察 O-1 | 观察（读码推，没跑） | 扫描内核 `r_instance_pages_after_first`、`r_instance_rows` 两处同样的 256 走满不记判不了，CPU 无上限；根在候选集外时 I-3.12 / I-7.9 可能 CPU 红而 GPU 不红 |
| K1 J2 两条关门流 | 一致 | 两条都走到门不置那一支；门误开时 I-9.29 GPU 红 CPU 不红，对拍会红 |
| K1 J2 `chain == 0u` 变异 | 一致 | 2939 行点名「链指针记录不合法」流，锚点恰好命中 1 次 |
| K1 J2 缺口 | 观察 | 「第 1 片读不出」流没有变异行点名 |
| K1 J3 类型段不认得 | 大体一致 | 不读孩子、两槽 / 一槽按层级、I-9.2 红、不记走读失败，与墓碑、谱系同；GPU 同改 |
| K1 J3 不同的一格 | 不一致（一处） | inode 树这一支前先探测孩子自证的类，墓碑、谱系不探测；老写法，不是 J3 引入 |
| K1 J4 期望 | 一致 | harness 类型段 7 镜像钉「恰为」表；GPU 两条流 CPU 侧期望较松（4.1），靠逐位比较补 |
| K1 测试 变异行点名与参数筛 | 一致 | 8 行锚点各 1 次、筛得到；harness 三行实跑被各自点名的测试抓 |
| K1 测试 替换的两行 | 一行是、一行不是 | 2934 只换锚点；2935 换了锚点、替换文与测试名，是新变异 |
| K3 wgsl | 一致 + O-1 | 见第 5 节 |
| K3 GPU 对拍 .rs | 一致 | 见第 5 节 |
| K3 walk.rs | 大体一致 | 见第 5 节 |
| K3 harness 测试 .rs | 一致 | 见第 5 节 |
| K3 mutations.tsv | 一致 | 见第 5 节 |

## 8 没做什么

- 不判 K2（哪一格会错）：那是攻方腿的格；O-1 只是读码顺手看到的同族写法，不替主 agent 判它算不算打中。
- GPU 对拍与 GPU 那几行变异（2938–2942）不跑：这是 checker 档包 `singlefs-checker-tier` 的测试，且要 GPU 卡。提交时主 agent 跑：`SINGLEFS_HEAVY_TESTS=commit cargo test -p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- <变异表第 6 段点名的测试名>`，经内存包装。看哪几行：256 片流绿、257 片流绿且打印 `CRASH_JUDGE_GPU_CHAIN_LIMIT` 的 `left_to_the_cpu_by_the_chain_limit` 大于 0。
- 没有核 wgsl 的 naga 解析与校验（`for (; …; …)` 空初始化写法、模块级 `const` 引用另一常量）：库单测 `checker_walk_kernel_source_parses_and_validates` 在 checker 档包里，我不能跑；主 agent 的观测是合入验证 170 个测试二进制全绿，我复核不了。
- 没有核扫描内核展开大小（`every_kernel_stays_within_the_inlined_size_budget`）：本轮正文划出去了。
- 没有判 m3-batch26-code-r1 判决第三节、实例表走读「走过」的键：本轮正文划出去了。
- 链模型是我对 wgsl 控制流的手抄，只能说明「我读出的控制流」的结论，不能当 GPU 实测。
- 临时目录 `/tmp/claude-1000/m3-batch27-code-r1-sonnet/` 下我自己建的拷贝与编译目录（`work`、`target`、`run1`、`fresh`、`patchcheck`）交回前都删掉；证据都在本报告与模型目录里，需要时 `rerun.sh` 重生。
