# 根环实现第 10z 批第十六至十九块：代码轮第一轮判决（2026-10-09）

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 M1 M2 M7 M12 R C P Q F A -->

正文 `research/prompts/_m3-batch10z-pqrs-code-r1-body.md`。两腿报告：
- 攻方：`research/prompts/m3-batch10z-pqrs-code-r1-opus-output.md`，sha256 9073ba38…dcfc，337 行；模型 `research/prompts/m3-batch10z-pqrs-code-r1-opus-model/`，七份文件 sha256 与报告开头逐个相同；
- 正推：`research/prompts/m3-batch10z-pqrs-code-r1-sonnet-output.md`，sha256 a03f3a72…a519，537 行；模型 `research/prompts/m3-batch10z-pqrs-code-r1-sonnet-model/`。

## 一、核查

| 项 | 结论 |
|---|---|
| 开工快照 crates | 冻结副本 `~/.local/share/singlefs/work/m3-batch10z-pqrs-code-r1/tree/` 对 `research/prompts/m3-batch10z-pqrs-code-r1-snapshot/crates-sha256.txt` `sha256sum -c --quiet` 全过 |
| 开工快照 kb | `kb-sha256.txt` 六份里 D16、D23、E171 页 OK；D28、invariants、checks-owed 对不上，都是删除线在腿跑着时写的：D28 照删除线交的原串新串清单 `~/.local/share/singlefs/work/kb-writeback-retire/for-root-ring.md`（144 行，sha256 前 12 位 daabb9a44623）把 7 处新串换回原串，副本 sha256 前 16 位 f7d8de0c4bcb7503 与快照相同；invariants 对 HEAD 只改第 12、16、19、155 行与第 317 行后插一行，腿引的第 58 行不在其中、之前行数不变；checks-owed 删除线改了第 477 行（C617 补句）、第 615–621 行（新插 C766–C772），两腿没有按行号引 checks-owed |
| 腿引 D28 第 27 行三句 | 倒推出的快照副本第 27 行三句各命中 1 次 |
| 引文 | `cite-check.py` 攻方 `--root` 冻结副本：11 处对上 9 处、2 处 D16 路径没判，主 agent 现核 `.claude/kb/decisions/16-发布语义.md` 第 29 行「推空与抬 F 产生的空发布根不算」、第 36 行「非空有效根不足 4 个时取最旧的有效根」各在；正推 `--root` 冻结副本加仓根：17 / 17 |
| 复跑攻方模型 | 主 agent 在新副本上整份跑 `rerun.sh`（`~/.local/share/singlefs/work/m3-batch10z-pqrs-code-r1-main-rerun/runs/`，rc=0）：23 份输出里 `^ATK` 行 22 份与腿交回的逐行相同，`k1-small.log` 只差腿那一份少了 `small=overwrite` 标签、数逐个相同（报告没引它）；`k1w-run3.log` 计数 `cells=203 stuck=203 push_ok=21 push_fail=182 way1_ok=203 ctrl_refused=27 ctrl_admitted_push_fail=155`，与报告第一节逐字相同。复跑的副本与编译目录（约 1.3G）已删 |
| 复核正推 K4 行 1524 | 主 agent 在冻结副本拷贝上照 `crates/mutations.tsv` 那一行（变异名「第10z批第九块 K3 loop-span1 …」，冻结副本第 3380 行）原文恰好命中一次改成替换文，跑那份 harness 测试 `--include-ignored`：`test result: ok. 36 passed; 0 failed`，点名的 `unmount_after_the_root_ring_slot_of_an_abandoned_root_turns_unreadable_takes_its_recorded_placements_and_succeeds ... ok`，坐实这一行已不红。副本与编译目录（约 1.2G）已删 |
| 主 agent 原型（乙，为交用户的代价数） | 冻结副本拷贝上给抬 F 那一串加一个推环之后用的入口（不判上限、不带卸载记号、回收的槽扣到生效），推环做成之后把 F 抬到推出来的最后一版，跑攻方五件（`~/.local/share/singlefs/work/m3-batch10z-pqrs-code-r1-main-proto-b/runs/`）：`cells=203 stuck=203 push_ok=203 push_fail=0`；实现员三形长到 2–80 个单元 24 格 `push_then=ok@1`；两槽 C 与被抛弃根 A 的历史 `violating_prefixes=0`。副本上的数、零轮，只用于弹窗，不进 kb；用户没选这一条 |
| 腿的数 | 都是腿的模型或副本上的数，判决只据它们定下一批，不引进 kb |

被判的源文件：`crates/singlefs-core/src/mounted_session.rs`、`crates/singlefs-core/src/mount.rs`、`crates/singlefs-core/src/allocator.rs`、`crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs`、`crates/singlefs-checker-tier/src/gpu_unit_checks.rs`、`crates/singlefs-checker-tier/src/crash_amplification.rs`。

## 二、逐格判决

| 格 | 两腿 | 判 |
|---|---|---|
| K1-1 判据 | 攻方：同一盘面去掉 C 照样被拒的 27 格也报 `StuckBehindTheWideProtection`；正推：`ff` 例同形（384 槽盘 39 行） | **打中（窄）**：判据只看「F 在上限、C 罩着的槽有隔离着的」，不看放开之后写不写得下。改法并进用户定的甲（第三节）：出路按预演定，不再把走不通的出路报给用户；字段名「被宽护卡住」在那 27 格上仍不准，下一批在文档注释里写明宽护是挡着的之一、不是唯一 |
| K1-1 三个量 | 两腿：次数、R 的 txg、槽数单槽 C 各形都对（正推 365 次报告 0 次不符） | 没打中；正推注入级两形（C 与被抛弃根落点重叠时「放出来」的数偏大、C 有落点而无隔离槽时推环入口照推）进下一批 |
| K1-2、K2-3 出路二走不走得通 | 攻方：384 槽盘 9 形 203 格推环本身都做成、隔离回 0，推完同一个写再试 182 格照旧被拒、其中 155 格对照臂放得下；实现员三形长到 60、80 个单元 6/24 格推完仍拒；不推环只建 3–4 个 inode 就放得下。正推：自己的历史上 265 次推环 226 次写得下、39 次仍拒，256 槽盘 21/21 仍拒，判「没打穿条款」；它写的推翻条件「384 槽盘上推环、重试被拒，而同一历史无 C 的对照写得下」正是攻方 155 格的形 | **打中**：主 agent 复跑坐实攻方的数。机理是读码推的（推出来的全是同一状态的空发布根，环里非空状态不足 4 个，`.claude/kb/decisions/16-发布语义.md` 第 36 行上限退到环里最旧的根，defer 窗口变成整环）。交用户弹窗（四条路各带代价，乙带主 agent 原型的数），**用户 2026-10-09 定「甲：预演推环与准入，推不通就不列」** |
| K1-2 出路一 | 两腿：卸载再挂 203 格与实现员三形全放得下。攻方：R 没有根时出路一那句「R 按新实例的实例表判成被抛弃」不对，实际是再挂时 C 那一圈没了。正推：R 的账永久读不出时卸载先做成、再挂被拒，会话已断，出路一排第一对这一成因偏险 | **打中（窄，措辞）**：出路一那句按 R 有没有根分两种说法；写明「R 的账读不出若是盘上永久坏，卸载之后再挂会被拒、会话已断开」 |
| K2-1 一致性 | 攻方：568 个块设备错注入点、478 个写前缀，崩了再挂池级 checker 0 违例。正推：四个注入点与 322 个推环中镜像 0 红，256 槽盘阳性对照 27/38 红 | 没打中（两腿各一次，攻方带对照臂、正推带阳性对照） |
| K2-1 报法 | 攻方：真发失败之后再推一次，预演在冻结着的分配器拷贝上照样过、报成 `PublishFailedAfterTheRehearsal`；`mounted_session.rs` 第 348 行那句「预演与真发只在读盘核换下的那一份对不上时分叉」在这一形不成立 | **打中（窄）**：推环入口第一道先看 `allocator.frozen_publish()`，冻结着就在预演之前拒成点名冻结的 `RootRingPushRefused` 成员 |
| K2-2 提前盖掉 | 两腿：两槽 C、环里另有被抛弃根，逐写前缀 I-7.4 / I-4.8 0 违例，离环次序对 | 没打中 |
| K2-3 出路二措辞 | 正推：「根环里更早的回退候选会被这一串整环写掉」对次数少于一环的推法偏重（偏移 3 时只推 3 次，环长 24） | **打中（窄，措辞）**：写成推几次、写掉环里几槽 |
| K3-1 | 攻方 480 段 × 16 步、正推 1260 段，0 漏、0 双扣；正推漏补对照 39/96 红 | 没打中 |
| K3-2 | 正推：表里没记着那一格改成真，入库 35 条全绿、192 段历史不红 | 没打中（危害没证）；这一支就是 C762（抬 F 给根环表没记着的被抛弃根补的隔离一直不放），不补用例，C762 题面已有 |
| K4 core 判别力 | 攻方 M1（真发失败不挪 `current`）、M2（推环入口不核盘表）、M7（挡着的槽数取全部隔离）、M12（R 那一槽取最先轮到的）对那份 harness 测试 35 条全绿；正推自套 25 个变异活 11 个，另坐实实现员替换的第 10z 批第九块 loop-span1 那一行已不红（主 agent 复核同） | **打中（窄）**：补用例到会拦 |
| K4 checker 档扫源码 | 正推：崩溃放量那条名单外三种写法拦不住（参数不独占一行的新层、换参数名的新层、`let` 解构或多空格重绑）；GPU 卡池那条名单外没找到静默绕过 | **打中（窄）**：能拦的补到会拦，拦不了的写进名单 |
| K5 | 正推：第十八块 5 个 hunk 都在 `mod tests` 里、产品代码没动；卡池模块以后合法的 8 类写法会被判红，其中 `const X: bool = A != B;` 是把 `A !` 当成宏调用的误判；`brace_depth_at_the_start_of_each_line` 遇到原始字符串直接 `assert!` | **打中（窄）**：误判要改（`!=` 不当宏），别的合法写法判红的在用例旁写明是故意的还是误判 |

## 三、用户定案与下一批（第 10z 批第二十、二十一块）

用户 2026-10-09 弹窗原话「甲：预演推环与准入，推不通就不列」：推环入口不改；报 `StuckBehindTheWideProtection` 之前在分配器拷贝上预演推环，再按推完的根环表算上限、预演一次这次改动的准入（连同它会推的抬 F）；放得下才在出路里列推环，放不下只列卸载再挂。弹窗里交代了：乙（推完把 F 抬到最后一版）在主 agent 原型上 203/203 放得下、越过 D16 第 36 行；丁（去掉推环入口）会话要断开、R 的账永久读不出时再挂被拒；丙只改措辞。

- 第二十块（core）：`mounted_session.rs` 甲；推环入口先判冻结；出路一按 R 有没有根分两种说法并写明永久读不出的后果；出路二写推几次、写掉几槽；「被宽护卡住」的文档注释写明宽护是挡着的之一；正推注入级两形（重叠时放出来的数、C 有落点而无隔离槽时推环入口的结局）定成会红的用例；用例补到拦住 M1、M2、M7、M12、loop-span1 那一行与正推活下来的变异。落在 `crates/singlefs-core/src/mounted_session.rs` 与那份 harness 测试，不碰 `mount.rs`。
- 第二十一块（checker 档测试）：卡池扫源码把 `!=` 当宏的误判改掉，另几类合法写法判红的写明；崩溃放量扫源码名单外三种能拦的补、拦不了的写进名单；原始字符串那一处 `assert!` 改成认得原始字符串或写进名单。落在 `crates/singlefs-checker-tier/src/gpu_unit_checks.rs`、`crates/singlefs-checker-tier/src/crash_amplification.rs` 的测试模块。
- 两块合入后合成一批再走一轮代码轮。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D16（发布语义） 已定项 1 | 备料 | 2026-10-09 不受影响：用户选甲，不越过「抬 F 的上限」那一行；第 37 行写回时带上「出路按预演定」 |
| D28（挂载期承诺量） 已定项 1 | 备料 | 2026-10-09 不受影响：第九项判的两条补隔离路没打中；依据段指针等第二十、二十一块判完一起写 |
| D23（journal 的角色与格式） 已定项 14 | 备料 | 2026-10-09 不受影响：冻结那一道照旧，推环入口先判冻结只是提前拒 |
