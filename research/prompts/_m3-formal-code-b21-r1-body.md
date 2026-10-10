# 第 21 批代码轮第一轮正文（2026-10-04）

<!-- doc-lint:not-numbers K1 K2 K3 F3 H1 H2 -->

## 一、题面

第 21 批（`records/2026-10-01-里程碑三形式化接入.md#21`）合入了两份补丁，改动全文 `research/prompts/_m3-formal-code-b21-r1-diff.md`：

- 21-1（判决 `research/prompts/m3-formal-code-b19-r1-main-verification.md` 第三节第 3 条）：
  - H2：`crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs` 新用例 `the_first_file_after_nine_warm_up_only_mounts_lands_past_the_mkfs_instance_table_that_is_one_txg_short_of_reclaim`：mkfs、只暖机的可写挂载 9 次、第 9 次挂载上写第一个文件，断言每块盘的数据单元落 U + 4、池级 checker 全绿；变异「回收谓词 `generation.0 <= floor.0 + 1`」（`crates/singlefs-core/src/allocator.rs`）。
  - I-9.27 写 0 的六种：`crates/singlefs-harness/tests/checker_known_bad_images.rs` 给种类 4、6、7、8、11、12 各一张坏镜像（第 22–27 张），原用例改成每张判完再一起断言；六行变异「种类 X 不归 Zero 那一臂」（`crates/singlefs-checker/src/walk.rs`）。
  - `crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs` 文件头与一条用例的文档注释照 `crates/singlefs-harness/src/mount_transparency.rs` 现行判法（R2）改写。
- 21-2（判决 `research/prompts/m3-formal-code-b20-r1-main-verification.md` 第三节第 2 条，连同第 18 批判决的 F3）：
  - F3：`draw_second_crash_points` 的种子再异或第一次崩溃段内子集的 FNV-1a 指纹（`fingerprint_of_the_persisted_subset`），只动按种子摆那一支；内联用例与变异「指纹换成空子集的」。
  - 改法 a：固定历史用例加断言「发布之后读回的 inode 号比崩溃那一版多恰好一个」；变异「那一支建 0 个」。
  - 改法 b：新测试文件 `crates/singlefs-checker-tier/tests/crash_injection_publish_after_a_mount_whose_deletion_continuation_stays_short.rs`，短续做盘面（单元区 128 槽、覆盖写 3 次、14 个单元、意图之后 1 批）断言被单元区墙拒（`Explained(UnitAreaWall)`）；变异「判法只看 `inode_record` 是不是 Some」。
  - 第四种接错（第三截「所选那一版」交成第一次崩溃那一版）：找到判官结论只变在 C670 成立那一支的态，合法与否与计数分不开；用例与变异照「支」这一层写。主 agent 判这样算还清（读码推：判据对所选那一版单调，换成更旧那一版只会真到真）。

条款：`.claude/kb/invariants.md` 的 I-5.5、I-9.27 两行；`.claude/kb/decisions/03-空间分配.md`「#### 已定项 7」「#### 已定项 8」；`.claude/kb/decisions/08-核心索引结构.md`「#### 已定项 16」；`.claude/kb/checks-owed.md` C670（无读故障的崩溃态上可写挂载见证判据为真没有判官）、C689（建树落点判据抓不到写死从 U + 4 起扫的坏实现）、C696（崩溃注入的读故障一支在随机历史里从没触发）。

## 二、实现今天的样子（主 agent 的观测，2026-10-04）

- 合入验证：harness 档 150 个测试二进制全绿；fmt、`clippy --all-targets --all-features` 退 0。21-1 的 7 行变异 prove-red 全抓到（种类 4、12 由实现员证，6、7、8、11 由主 agent 补证，各只红自己那一张）。
- 21-2 的 checker 档用例与 4 行变异只编过、没跑；实现员在 harness 档写探针量过：混子集时两个子集摆法不同、不混时相同（长度 0–12 的子集同长度内指纹 0 处相撞）；建 1 个 inode 读回 [(2,1),(3,1)]、建 0 个读回 [(2,1)]；只看 Some 时得 `Unexplained`（`WriteToAnInodeBeingDeleted`）。它读码判断 F3 之后变异 2195、2283 的红绿可能变，归提交时重跑。
- 主仓里 `crash_injection.rs` 在这一批之前已合进根环那条线第 9 批的改钉簇（部分没判计数）与实例表清扫两段。

方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　21-1 的用例证不证得到点名的东西
H2 用例在正确实现下绿、在放宽一个 txg 下红，有没有别的原因让它在别的历史长度、别的环几何上该绿而红；六张 I-9.27 坏镜像各只红 I-9.27、改动只落在点名的那一种条目上；原用例改成「判完再一起断言」有没有放宽判定；注释与 `mount_transparency.rs` 现行代码逐句对不对得上。

### K2　21-2 的 F3 与改法 a、b
F3 的指纹会不会让某些第一次崩溃态在不同段号或不同子集上撞到同一串随机数、让二次崩溃的取样反而变窄；改法 a 的断言在建出一个 inode 之外的哪些坏发布下仍然绿；改法 b 的盘面是不是真走到 nlink = 0 那一支（不是别的原因被单元区墙拒），只看 Some 那一臂在这个盘面上是不是真报 Unexplained。

### K3　第四种接错按「支」算还清对不对
「判据对所选那一版单调」这句推论成不成立；有没有一个态让第三截「所选那一版」接错改变合法与否或计数。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1、K3 | 逐句核代码做的是不是条款与判决说的，每格给判定与引文 |
| 云端攻方（Opus） | K2、K3 | 攻哪一格会错：用例判不出的坏改法、取样变窄、单调推论的反例；量得了的在副本上写小模型或 harness 用例量，checker 档不跑、写推的 |

## 五、跑前写死的判据

- 两腿同判且主 agent 现查坐实 ⇒ 按那一判；打中而坐实 ⇒ 改法并进下一批实现员；分歧 ⇒ 开第二轮。
- checker 档、GPU 没跑的格，腿的结论只能是「推的」，判决里照实标。

交付：腿的报告写 `research/prompts/m3-formal-code-b21-r1-<腿名>-output.md`，分段落盘（每次写进文件不超过 150 行，第一段排他新建，回复只写短句）；模型与脚本放 `research/prompts/m3-formal-code-b21-r1-<腿名>-model/`；草稿放 `/tmp/claude-1000/<腿名>/`。引 kb 条款写 kb 文件自己的行号（去 kb 文件里现查，不从背景材料里数）。方案按 `crates/` 今天的实现来谈。
