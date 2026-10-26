# m4-exit5-relocation-proof-r2 判决：出口 5 照「core 直验，连胶水一起证」的做法与代价（主 agent，2026-10-26，第二轮）

<!-- doc-lint:not-numbers Y1 Y2 Y3 Y4 Y5 Y6 G7 P4 -->

同一题的第二轮。第一轮判决 `research/prompts/m4-exit5-relocation-proof-r1-main-verification.md`。用户 2026-10-26 先定层「core 直验，连胶水一起证」，看过这一轮正推量出的规模后再定「里程碑四全验」（A、B、C、D 四类都在里程碑四做完）。

两条腿：正推 `research/prompts/m4-exit5-relocation-proof-r2-sonnet-output.md`（sha256 ced14ddef3f375d0723086056ba5954acb6f847938cc2c7fced01db4446a9098，模型目录 `research/prompts/m4-exit5-relocation-proof-r2-sonnet-model/`），攻方 `research/prompts/m4-exit5-relocation-proof-r2-opus-output.md`（sha256 5082ff2ca50acb43909ab43849cb5b78179b0c2645fc1d513c2779e6f02c47aa，模型目录 `research/prompts/m4-exit5-relocation-proof-r2-opus-model/`）。

## 一、核查

- 开工快照：`python3 scripts/agents/snapshot-tree.py m4-exit5-relocation-proof-r2 --out /tmp/claude-1000/-home-fy5090-code-singlefs/361bb28f-a369-4461-967b-925319d2cf5d/scratchpad/x5r2/tree`，「清单点名 1 份，倒推出 1 份，sha256 对上 1 份」。
- cite-check：`python3 scripts/agents/cite-check.py research/prompts/m4-exit5-relocation-proof-r2-sonnet-output.md research/prompts/m4-exit5-relocation-proof-r2-opus-output.md --root <上一条的树> --root /home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen --root /home/fy5090/code/singlefs --background research/prompts/_m4-exit5-relocation-proof-r2-background.md`，末行「✓ 核了 8 处引文、71 处裸行号，对上 7 处，没判 72 处（其中裸行号没有引文可比 45 处；逐处列在上面）」，没有对不上的。
- 两条腿模型复跑（第二台没有 Verus，在本机仓外目录跑，`RUN_WITH_MEMORY_CAP_WAIT_SECONDS=7200 bash scripts/hosts/run-with-memory-cap.sh 8G bash ~/.local/share/singlefs/work/m4-exit5-relocation-proof-r2-verify/<腿>-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen --work …/<腿>-rerun`）：攻方末行「name=rerun-compare out/summary.txt same=yes」；正推末行「name=rerun_result mismatches=0」（y1–y5 各段 `result=same`）。两份报告里的数都复出。

## 二、逐格判定

| 格 | 两条腿 | 判定 |
|---|---|---|
| Y1 怎么接 | 正推：`verus!` 宏与属性模式都能接上 core 的小函数，产品构建 0 警告，5557 个 MIR 函数规范化后 0 行差；vstd 本来就在 core 的产品依赖闭包里。攻方打中：同一个 `verus!` 里用 `cfg(verus_keep_ghost)` 写两份函数体，Verus 带 `--no-cheating` 验过、产品二进制违 ensures，门禁作弊正则不认；Verus 的 rustc 1.98.1 对产品 1.98.0 | **接法采正推，攻方两处收成必须的检查**：被验区域里不许出现按 `verus_keep_ghost` 分叉的函数体（判法加一格、配必红样本）；产品构建与 Verus 看到的 MIR 逐函数相同那一格进门禁（正推 Y1 的比对做成会红的检查）；Verus 与产品的 rustc 版本差写进工具链钉表，对不上判红 |
| Y2 std 规格与作弊标记 | 两腿一致：`assume_specification`、`external_body`、`uninterp spec fn` 在 `--no-cheating` 下被 Verus 自己拒，core 直验必须去掉这个旗子。正推：闭包 456 个 std 调用键，按名字配上 vstd 规格 49%（调用点 82%）；登记式（集合相等、差分测试、必红世界）判得出写错的规格。攻方打中：去掉旗子后 `axiom fn` 证出 false 也过、门禁正则不认 `axiom`；看着对而与 std 不符的规格必红世界照红、产品违 ensures；vstd 自带 485 行 `assume_specification` 等不登记 | **采登记式信任清单，攻方三处收成清单的判法**：去掉 `--no-cheating`；被验区域里每一处 `assume_specification`、`external_body`、`axiom`、`uninterp` 都要在信任清单里有一行（点名的 std 函数、规格原文 sha256、配一条对 std 实际行为的差分测试），没登记的判红；vstd 自带的那批按 vstd 版本整体登一行（钉 vstd 版本与那批的 sha256）；`axiom fn` 一律不许（证出 false 那一形的必红样本） |
| Y3 射程与分阶段 | 正推：两个闭包并起来 1073 个 fn、24970 行，分 A 472 / B 190 / C 291 / D 120；第一阶段原型 13 verified；总代价推的约 11 万–12.5 万行。攻方打中：崩溃之后读映射那一路 30 个 fn、1521 行在射程外（`locations_of_key`、`read_mapped_tree_root`、`replay_journal`）；闭包里 5 种只供测试的运行时开关，pub 的 `set_reuse_window` 在闭包外，拨到 `ForcedToZero` 就绕过 F 当场回收 | **射程加崩溃后读映射那一路**（发布、回退、崩溃后读三个闭包的并集）；只供测试的开关在被验区域里一律当前提写进 requires 或挪进默认关的 cargo 特性（与 m4-publish-trigger-r1 判决 P6 同一个做法），`set_reuse_window` 那一条进第三轮必攻；用户定全验，分阶段只排次序（A、B、C、D），不减射程 |
| Y4 判法与门禁 | 正推：`verus-proof-check.py` 与登记表要改十一条（多 crate 登记、`cargo verus focus` 替直接起 `verus`、信任清单替作弊标记），cargo 指纹不看 `--verify-*`，换参数再验会拿到上一趟的结果（实测）。攻方打中：1071 个 fn 里 598 个在 impl / trait 里、82 个名字被 274 个 fn 共用，按裸名逐项验走不通；只加 ensures 或 proof 块就让判器闭包键变，549 个 fn 同在判器闭包里，改一次 core 证明崩溃判定库整库重判 | **采正推的十一条，攻方两处收成条款**：按函数单元的全路径（含 impl 与 trait）逐项验；判器闭包键在算指纹时去掉 Verus 的规格与证明块（只留会编进产品的代码），把「只改证明、产品代码不变」做成键不变（单元图那一侧的改法，配三问自证），否则整库重判的代价写进交用户表 |
| Y5 第一轮挂起的三处 | 正推：新协议 13 verified、区间版 P4 9 verified，并入 replay-prefix 副本 88 → 110，第一轮三个反例被排除；G7 在含回退与抬 F 的历史上不是恒等，是「回退候选窗口移一格」，已证成引理。攻方（没读到正推的改后协议，攻的是第一轮判决字面那两处修补的最直接写法）打中：区间版 P4 只看现行映射，搬迁能写进回退目标还在读的槽（33 verified、三份必红变体都红）；补了回退视图的协议仍只按起点槽号避让，两槽单元的第二槽照样可写 | **采正推的改后协议与 G7 改写，攻方两处进第三轮必攻**：第三轮在正推那份协议上复跑攻方两个反例的变体（写进回退目标的槽、两槽单元的第二槽），排除了才算这一格关上 |
| Y6 罩不到的 | 正推列十五项、各写由什么罩；攻方同 | **采**，进出口 5 的射程句 |

## 三、交用户

| 项 | 形态 | 被攻过几轮 | 去向 |
|---|---|---|---|
| 全验：发布、回退、崩溃后读三个闭包 | 用户定 | 一轮（这一轮攻了射程，补进崩溃后读） | 第三轮攻整形 |
| 信任清单替 `--no-cheating`、axiom 一律不许、cfg 分叉判红、MIR 比对进门禁 | 两腿合 | 一轮 | 第三轮 |
| 按全路径逐项验、判器键去掉规格与证明块 | 两腿合 | 零轮 | 第三轮 |
| 改后协议、区间 P4、G7「回退候选窗口移一格」 | 正推提 | 零轮（攻方攻的是字面修补） | 第三轮 |

## 四、写回与后续

- 第三轮（`m4-exit5-relocation-proof-r3`，最后一轮）攻第二节判定的整形；判完一起写回出口 5、G7、D13（新立一项或补已定项 1）、`.claude/rules/verification.md` 两句。
- 实现在第三轮判完之后按 A、B、C、D 分批派。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 1 | 备料 | 2026-10-26 不受影响：第三轮判完再回看 |
| D13（验证路线） 已定项 17 | 备料 | 2026-10-26 不受影响：判器键去掉规格与证明块那一条第三轮攻，判完再回看 |
