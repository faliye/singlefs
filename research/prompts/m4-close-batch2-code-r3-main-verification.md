# m4-close-batch2-code-r3 判决：里程碑四收口第二批，代码轮第三轮（最终轮，2026-11-10）

<!-- doc-lint:not-numbers R -->

## 一、腿与核查

- 腿：辩方 `research/prompts/m4-close-batch2-code-r3-sonnet-output.md`（复核第二轮判决 `research/prompts/m4-close-batch2-code-r2-main-verification.md`），攻方 `research/prompts/m4-close-batch2-code-r3-opus-output.md`，模型 `research/prompts/m4-close-batch2-code-r3-opus-model/`。各抽一次，「没打中」不支撑结论。
- 开工快照 `research/prompts/m4-close-batch2-code-r3-snapshot/crates-sha256.txt` 两份；腿跑着时主 agent 没改这两份文件。
- 引文：辩方报「✓ 核了 49 处引文、14 处裸行号，对上 49 处」，攻方报「✓ 核了 24 处引文、2 处裸行号，对上 24 处」。主 agent 现查：`grep -nE '回退门|门开|门关' .claude/kb/decisions/26-后台整理与放置回收.md` 零命中；第 104 行原文「例外是只删整理意图、不搬单元的那一次：它归删除类，不判式子，落点那一判照判」；第 254 行原文「一次整理发布要写的根槽连同它之后的余量里的根一走、根环里不同用户状态就少于 4 个时，这次整理发布不发」。
- 复跑：攻方探针任务 12569（模型拷到 `~/.local/share/singlefs/work/m4-close-batch2-code-r3-main-rerun/model`），`name=m4_close_batch2_r3_attack_snapshot rc=0 expected=0`；汇总 `name=m4_close_batch2_r3_attack_summary product_probe_all_ok=yes exception_drops_through_the_closed_gate=yes explicit_entry_drops_with_the_budget_spent=yes mutant_caught_by_the_probe=yes mutant_survives_the_existing_targets=yes`。
- 第二轮那一批的验证（量的）：变异表任务 12522「计数：抓到 9 条、没红 0 条、无效 0 条、锚点失效 0 行（共 9 条）」；崩溃点重放 2 项任务 12525「2 项都不红…崩溃枚举用例 3 条复用 3 条…缩过域」；命名四格任务 12523 退 0；build 任务 12481 第 4 次尝试退 0（前三次撞内存上限，编译规模所致）。

## 二、判决

| 格 | 判 |
|---|---|
| T1（攻方）＝ 辩方对第二轮交用户表的更正：「停着删意图要过回退门」kb 字面没写 | 打中，主 agent 现查坐实（第一节 grep 零命中）。第二轮判决交用户表那句「条款字面已写明那一次照样受第 5 条约束」不成立，是读法。交用户 2026-11-10，用户定「过回退门（推荐）」：代码不改，kb 第 254 行补一句写明停着那一次删意图也过这道门 |
| T2（攻方）：显式入口停着、预算扣完照删 | 打中，量的（`explicit_entry_drops_with_the_budget_spent=yes`）。显式入口预算 `.max(1)`（`crates/singlefs-core/src/compaction_plan.rs` 第 1577 行一带，辩方同报），没有产品调用方。用户 2026-11-10 定的「照预算，扣完就等」问的是后台入口；kb 写回把两个入口都写成「扣完就等」，与代码对不上。改 kb 字面限在后台入口，不改代码 |
| T5（攻方）＝ F2（辩方）：停着删意图那一支的破例分支没有用例与变异 | 打中，量的（`mutant_survives_the_existing_targets=yes`、`mutant_caught_by_the_probe=yes`）。补用例与变异行，只改测试代码，不开代码轮 |
| H3「换得到东西」那一半（辩方、攻方 T6） | 推不出：A/B 的 2 对 16 只证明读到了，两臂 `started=false`、`publishes_of_the_wake=0`，H3 会改变起跑的那一形没造到。次序本身有 kb 第 254 行「挂载不再给新预算」撑着，不撤。登欠账 |
| F1（辩方）：停着且门关着的那一趟没有备忘，运行库一拍最多连叫 64 次 | 推不出，读码，不写盘。登欠账 |
| T3（攻方）：R 是唯一全空段、R 之外写满时，前台写不进的破例不触发，门等不来新改动 | 推不出，没造盘面；这一形在没停着的循环里早就有，B2 改法把它带进了停着那一支。登欠账 |
| T4（攻方）：H2 新加的文档串「这一趟不跨调用接着推」比代码宽 | 推不出，轻。文档串写实 |
| F4（辩方）：H3 前移后读根环读不通按预算扣完算，停着删意图压到下一次用户改动 | 推不出，边角。登欠账 |
| F5（辩方）、`mounted_read.rs` 第 1030 行注释（辩方）、两处文档串没写破例那一档不过门（攻方） | 文档串写实 |
| 没打中（各一次抽样） | H2 对显式入口与可写挂载续做（两腿）；B2 与攒批会话、放锁入口的交叉（攻方）；「停下都删意图」与门关时留着冲不冲突（攻方） |
| 四·1、四·3 至四·6 | 两腿都判成立 |
| 四·2 | 攻方推不出（T4 与文档串），并进上一行 |
| 四·7 | 变异那一半成立（任务 12522）；崩溃点重放那一半任务 12525 有了结局：2 项不红，缩过域 |
| 四·8 | 辩方判成立（推的，与暂停流同一个发布函数）；攻方推不出（删意图那一版落盘前后崩、恢复先走删除续做那一段没有枚举）。登欠账 |

## 三、改法

1. kb：`.claude/kb/decisions/26-后台整理与放置回收.md` 第 254 行（已定项 13 第 5 条）补一句：停着时那一次只删意图的发布同样过这道门，门关就不发、意图与 R 排他留到门开（用户 2026-11-10 定）；第 251 行一带（已定项 13 第 4 条）「扣完就等」限在后台入口，显式入口预算至少 1（T2）。变更史同步。
2. 测试（只改测试代码，不开代码轮）：攻方探针 `research/prompts/m4-close-batch2-code-r3-opus-model/probes/attack_stopped_drop_exception_and_the_explicit_entry.rs` 缩成入库用例，配变异行「破例那一档也判门」（T5）。
3. 文档串写实（只改注释）：T4、F5、`mounted_read.rs` 第 1030 行、两处文档串补「破例那一档不过门」。
4. 欠账：H3 改变起跑那一形没量、F1、T3、F4、四·8 删意图落盘前后崩恢复先走删除续做那一段。

## 交用户的

| 项 | 状态 |
|---|---|
| 停着时那一次删意图过不过回退门 | 用户 2026-11-10 定「过回退门（推荐）」 |

## 这一轮被判过的文件

- crates/singlefs-core/src/mounted_session.rs：B2 改法（停着删意图先读挂载预算、过回退门）、H3 次序、H2 解除停下时重判触发、破例那一档、相关文档串
- crates/singlefs-core/src/mounted_read.rs：第 1030 行一带读与计数的注释

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D26（后台整理与放置回收） 已定项 13 | 推翻 | 2026-11-10 改了：第 5 条补「停着删意图也过回退门」，第 4 条「扣完就等」限在后台入口 |
| D26（后台整理与放置回收） 已定项 3 | 不影响 | 2026-11-10 不受影响：「归删除类，不判式子」管式子，不管回退门，字面不改 |
