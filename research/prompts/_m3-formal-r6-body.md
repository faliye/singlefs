# RefFS 语义级的崩溃语义与「挂载后再恢复」模掉什么：第三轮正文（最终轮，2026-10-02）

<!-- doc-lint:not-numbers O1 O2 O3 Y2 Y7 T1 T5 T6 U1 U2 U3 U4 U5 U6 U7 F1 F2 F3 F4 F5 F6 G1 G2 P Q R1 R2 M2 M3 V1 V2 V3 W1 -->

## 一、题面

前两轮判决：`research/prompts/m3-formal-r4-main-verification.md`（第一轮，含「用户定案」F1–F6）、`research/prompts/m3-formal-r5-main-verification.md`（第二轮，含「用户定案」G1 收严成全四条、G2 回退目标作输入）。前两轮四条腿的报告：`research/prompts/m3-formal-r4-sonnet-output.md`、`m3-formal-r4-opus-output.md`、`m3-formal-r5-sonnet-output.md`、`m3-formal-r5-opus-output.md`。

这是第三轮、最终轮（`.claude/rules/three-way-inference.md`「第三轮之后停」）。只攻前两轮站住的形态：第二轮判决第四节八条（连同用户定的 G1、G2）与第一轮判决第四节三条（T1、T5、T6，已被第二轮改写的照第二轮）。这一轮里新冒出的零轮形态不再开轮，写进判决的交用户表、标「零轮」。

同批还有一件要攻：全池没有可用槽 0 时槽 1 在哪（C651（全池没有可用的系统配置槽 0 时 checker 与实现找槽 1 的读法不同）），第一轮判决 `research/prompts/m3-c651-r1-main-verification.md`（V1 逐档、V2 世代号最大同号取偏移小、V3 两张坏镜像与一条 io_min 8192 的层 0 流；用户定 W1 mkfs 把每块盘 [0, 根环基址) 整段写 0）。那一题是第二轮，攻它第四节三条与 W1。

## 二、实现今天的样子（主 agent 的观测，2026-10-02）

- `crates/` 里没有 RefFS（`grep -rn 'RefFs\|ref_fs\|reffs\|reference_filesystem' crates/ --include=*.rs` 零命中）；O1 在 `crates/singlefs-harness/src/model.rs`，判官 `crash_recovery_disagreement` 在 `crates/singlefs-checker-tier/src/crash_injection.rs` 第 902、1828 行与 `crates/singlefs-harness/src/fault_injection.rs` 第 2353、2360 行调。
- mkfs 今天只清根环三个区域与 journal 环（`crates/singlefs-core/src/make_filesystem.rs` 第 525–554 行，两处 `write_zeroes_at`）；checker 找槽 1 的 ③ 取 4096（`crates/singlefs-checker/src/image.rs` 第 567 行起 `system_configuration_slot_readings`），实现 ③ 逐档（`crates/singlefs-core/src/recovery.rs` 的 `slot_one_found_by_trying_every_slot_spacing_the_format_allows`）。
- 前两轮攻方的原型在 `research/prompts/m3-formal-r5-opus-model/`、`research/prompts/m3-c651-r1-opus-model/`，主 agent 都复跑坐实过（两份判决第一节）。
- 冻结副本 `/tmp/claude-1000/m3-formal-r6/tree/`，清单 `research/prompts/m3-formal-r6-snapshot/crates-sha256.txt`。方案按 `crates/` 今天的实现来谈。

## 三、格（第三轮的三个攻击面：「已定」撑不撑得住 / 清单完整性 / 报出的数能不能核）

### X1　站住的条款句撑不撑得住
第二轮判决第四节八条与 G1、G2、C651 判决第四节三条与 W1，逐条：写成条款句之后有没有一段历史让它把合法状态判坏或放过坏状态；G2 的「第 m 次已确认发布」在回退被拒、回退之后又崩溃、回退目标是被抛弃时间线的根这几格上定义得出吗；W1 写零在 mkfs 中途崩溃时留下什么（旧池残留一半清了一半没清），读者照 V1 逐档还择得对吗。

### X2　清单完整性
RefFS 要从调用方收的输入白名单（操作结局、回退目标 m、每次恢复读出的内容、显式发布点）全了吗，有没有第五样实际要用而没列的；挂载透明逐字段表还有没有前两轮都没碰的字段（根记录与系统配置的全部字段逐个过一遍）；C651 那一题：除了槽 1 的位置，「全池槽 0 都不可用」还影响哪一步（tail 从哪读、F 的系统配置那一份从哪读）。

### X3　报出的数能不能核
前两轮判决引的数（第二轮 U3 的 6 态、U5 的 16 390 / 24 598 / 12 309 / 57 446、U6 的 3 态；C651 的 C 组 12、S 组 240 各格）的取法本身：取样的历史、盘宽、替身（RefFS-Q 取实现的只读恢复当窗前窗后内容）会不会把数推向一边；在入库装置上重做时，哪一条历史、哪一档盘宽才算同一个量。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端辩方（Sonnet） | X1–X3 | 复核第二轮两份判决：判决收严的地方有没有收过头（把合法状态判红）、交用户的定案写成条款句时哪一句站不住；替第二轮被判出局的写法（照字面只取 ③、回退弃权、V1-常量、W2 不清）辩护，看它在哪一格其实不输 |
| 云端攻方（Opus） | X1–X3，重点 X1 | 找反例：造历史或坏实现让站住的条款句判错；不重复前两轮攻方报告各节攻过的角度；在 `crates/` 的拷贝上量，可以复用前两轮的原型（拷出去再改，不改原目录） |

## 五、跑前写死的判据

- 一条站住的条款句两腿都没打中 ⇒ 写回（一次抽样的「没打中」照 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」只算这一次）。
- 打中 ⇒ 主 agent 现查坐实；三轮里多数打穿才算打穿（这一题前两轮都站住的，这一轮打中只算一次，写进判决、交用户）。
- 腿提的新改法算零轮，不再开轮。

交付：腿的报告写 `research/prompts/m3-formal-r6-<腿名>-output.md`，分段落盘（每次写进文件不超过 150 行，第一段排他新建，回复只写短句）；模型与脚本放 `research/prompts/m3-formal-r6-<腿名>-model/`；草稿放 `/tmp/claude-1000/<腿名>/`。引 kb 条款写 kb 文件自己的行号（去 kb 文件里现查，不从背景材料里数）。方案按 `crates/` 今天的实现来谈。
