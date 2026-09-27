# kb 第七批前半写回规格：实审 A3-checker-2 → I-7.13 立号

规格文件：
- `/tmp/claude-1000/kb-batch7a-drafter/spec.json`
- `/tmp/claude-1000/kb-batch7a-drafter/spec.md`

`python3 research/scripts/kb-spec-check.py <文件>` 末行（两份一致）：

```
✓ 规格 9 条：旧串各恰好 1 次，编号引用、指代、依据配对与变更史快查都过（查了 9 条）
```

sha256sum：
- spec.json：`03982423701d2d9039b7b9dbc92cbc2c6362a8314077eacb755f1fb13444b578`
- spec.md：`03b3543e77c8354e1ebc0dac47c0d1ac1fcc0bd6482247d1fd1c80419bc57d4c`

规格 9 条对应判决给的 7 个任务项：任务 1 → 规格第 1 条（新增 I-7.13 行）；任务 2 → 第 2 条（第 12 行条数）；任务 3 → 第 3 条（I-2.4 状态列）；任务 4 → 第 4 条（I-1.10 状态列）；任务 5 → 第 5 条（09-加密.md:231）；任务 6 → 第 6、7 条（22-单元原子性怎么合成.md 第 206、72 行两处）；任务 7 → 第 8、9 条（decisions-history/2026-09.md 与 invariants.md「## 历史版本」两处插入）。

## 逐条源码核实（文件:行号、函数名，报告与判决里的行号只当线索，均现查过）

- `crates/singlefs-checker/src/image.rs:46` 常量 `SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS = "I-7.13"`；`:54` `IMPLEMENTED_INVARIANTS`（47 条）；`:454` `judge_system_configuration_values_the_reader_accepts`；`:619` `judge_pointer_mac_and_nonce_are_zero`；`:339` `value_refused_by_this_reader`（判定哪些 `Verdict` 归 I-7.13：`FormatVersionNotRecognized` / `EncryptionTypeNotOff` / `FixedStructureSlotSpacingOutsideTheFormatRange` / `PhysicalBlockSizeOutsideTheRootSlotBounds` / `JournalRingBytesOutsideTheSupportedRange`，R、S 越界的两个成员在 `false` 分支，仍归「不可择」）。
- I-7.13 判据里的具体界：`crates/singlefs-checker/src/lib.rs:209` 格式版本常量 = 1；`:213` 加密类型常量 = 0；`image.rs:284` 固定结构槽距 ≥ `FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES`（`crates/singlefs-format/src/lib.rs:231` = 4096）且槽 1 落在根环基址之前；`image.rs:294` `physical_block_size` ∈ [`ROOT_RECORD_BYTES`（`singlefs-format/src/lib.rs:188` = 457）, 槽距]；`image.rs:306-315` journal 环长 ÷ 4096 ÷ F ≥ 1 且环末端不越过单元区起始槽号。
- `crates/singlefs-checker/src/walk.rs:649` `judge_unit_header`；`:695` 判格式版本；`:730` 判 29 字节预留位；四个跟随指针判 MAC/nonce 的入口：`:790`（码 2 节点指针）、`:1440`（inode 内部条目子指针）、`:1766`（实例表链指针）、`:1916`（extent 数据指针），均调用 `judge_pointer_mac_and_nonce_are_zero`，落的是 I-2.4，不是 I-7.13。
- `walk.rs:747` `index_node_view_judging_a_zero_entry_width`（I-1.10：条目宽 0 而条目数非 0，且这棵树登记了条目字段表时报违例）。
- `walk.rs:5554` `check_pool_image` 入口；`:5573-5578` 调 `judge_system_configuration_values_the_reader_accepts`，任一槽越界即对该池全部 `IMPLEMENTED_INVARIANTS` 报「不适用」并提前返回。
- 坏镜像现查：`crates/singlefs-harness/tests/checker_known_bad_images.rs` 里 `grep -n "I-7.13"` 零命中——**没有一份改坏触发 I-7.13 的镜像**，`the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target`（`:2296` 起）因此仍会判红。规格第 1 条与第 8/9 条的现状句照实写「补不补交主 agent 定」，没有替主 agent 派实现员补这份坏镜像（那是判决第三栏「实现员『补 I-7.13 的坏镜像』」那一半，不归写回规格）。

## 修正了判决文字里一处与源码不符的地方（未改判决原意，只按源码写现状）

任务 5、6、7 的判决原文把「三种码的格式版本与 29 字节、指针头部 MAC / nonce、系统配置加密类型越界」都写成「报 I-7.13」。现查 `walk.rs:695/730` 与 `judge_pointer_mac_and_nonce_are_zero`，单元头格式版本 / 29 字节预留位 / 指针头部 MAC・nonce 三样报的都是 **I-2.4（头校验和覆盖范围）**，只有系统配置本身的格式版本、加密类型（连同槽距、pbs、journal 环长）越界才报 **I-7.13**——这与实现员报告第三节表格逐字一致（该表第 1、2 行落 I-2.4，第 3 行落 I-7.13）。规格第 5、6a、8、9 条据此写成「单元头/指针头部归 I-2.4，系统配置归 I-7.13」两分开的现状句，没有按判决字面把全部四样都写成 I-7.13。这不是要主 agent 判断的分歧点（源码只有一种读法），列在这里供核对。

## 全 kb 搜索「A3-checker-2」「判 46 条」「46 条」，命中而不在 7 个任务项里的

`grep -rn "A3-checker-2" .claude/kb/` 命中 12 处；其中 `.claude/kb/decisions-history.md`（49 号生成物）与 `.claude/kb/decisions-history/2026-09.md` 里带日期标题的历史条目（4 处）按 kb-discipline 第 8 条是「历史」，不算过时现状句，没进规格。**唯一一处不在 7 个任务项、且是现状句的命中**：

- `.claude/kb/decisions/22-单元原子性怎么合成.md:336`（已定项 16「系统配置槽轮换规则；根环区域公式的地址空间与区域内槽位」内）：「读者判固定结构槽距：≥ 4096，且槽 1 整槽落在同一槽自述的根环起点之前；`physical_block_size` ∈ [根记录宽 457, 槽距]。越界即拒：checker 对越界槽报违例、实现整池拒挂载（用户 2026-09-27 JST 14:0x 定「整池拒」，**A3-checker-2 在做**）。」——A3-checker-2 已交回落地，这句「在做」已过时，该改成「checker 报 I-7.13（系统配置池级字段在读者收的范围里）」。这条判决没点名，没写进规格，留给主 agent 判要不要并入下一批或单独补一条。

`grep -rn "46 条"` 命中：`.claude/kb/invariants.md:12`（任务 2，已处理）；`CLAUDE.md:109`（判决明写「由主 agent 改，不进规格」，未写规格）；`.claude/kb/experiments.md:165` 与 `.claude/kb/experiments/156-alloc-basis四条岔路的代价数.md` 两处「checker 46 条不变量零违例」——都是带日期的历史观测记录（某次跑的产物读数），不是「现共 N 条」这类当前状态声明，不算过时，没进规格。

## 另一处观察：invariants.md 第 16—17 行「现共 79 条在用」没有进规格

新立 I-7.13 会让「在用」总数从 79 变 80（同类先例：`invariants.md:297` 记 2026-09-27 立 I-1.11 时「在用条数 78 → 79」）。判决的 7 个任务项没有点名这一行，规格因此没有改它——这行既不含「A3-checker-2」也不含「46 条」，不在「写之前全 kb 搜一遍」的两个搜索词范围内，是核对第 1 条新增行时顺带看到的。是否要在这一批或下一批里把 79 改成 80，交主 agent 判。

## 要主 agent 判的点

1. `.claude/kb/decisions/22-单元原子性怎么合成.md:336` 那句「A3-checker-2 在做」已过时，判决没点名，没进规格——要不要在下一批写回。
2. `.claude/kb/invariants.md` 第 16—17 行「现共 79 条在用」没随 I-7.13 立号改成 80，判决没点名，没进规格——要不要一起改。
3. I-7.13 目前在 `checker_known_bad_images.rs` 里没有坏镜像，规格第 1、8、9 条的现状句已照实写「补不补交主 agent 定」；补这份坏镜像归实现员，不归这次写回。

## 没做什么

- 没写 kb（没有改 `.claude/kb/` 下任何文件），没跑 kb 门禁阶段（10、20、31、48、75 等——那些在 kb-scribe 写完之后由主 agent 或门禁跑）。
- 没有另做推论：规格里的判断以判决三栏原文、实现员报告第三/四节与现查的源码为准，没有替判决没写清楚的地方做选择（唯一一处文字与源码不符已在上面「修正」一节说明，是照源码写现状，不是判断）。
- 没有把「要主 agent 判的点」三条自己决定，也没有扩大范围去顺手改 `decisions/22-....md:336` 或 `invariants.md:16-17`。
- 没跑重型测试（任务写明「重型测试：不跑」）。
- 没删草稿目录：`/tmp/claude-1000/kb-batch7a-drafter/` 下的文件都是本轮自己写的规格与报告，没有编译目录、没有仓副本，无需清理。
