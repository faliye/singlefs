# m3-close-b19-r3fix-code-r1 判决：批 19 第三轮零轮改法合入之后的代码轮（2026-11-05）

被判的产品代码：`crates/singlefs-core/src/mount.rs`（这一批合入里唯一的产品代码；池级 checker、harness 档、checker 档的改动是测试代码，不进这一轮）。正文 `research/prompts/_m3-close-b19-r3fix-code-r1-body.md`，三份腿报告：正推 `research/prompts/m3-close-b19-r3fix-code-r1-sonnet-output.md`、攻方第一样本 `research/prompts/m3-close-b19-r3fix-code-r1-opus-output.md`（959a21ea…）、攻方第二样本 `research/prompts/m3-close-b19-r3fix-code-r1-opus-s2-output.md`（6ac9e297…）。

## 一、核查

- 开工快照 `research/prompts/m3-close-b19-r3fix-code-r1-snapshot/` 的六份（`crates-sha256.txt` 三份、`kb-sha256.txt` 三份）写判决时 `sha256sum -c` 全 OK（主 agent 2026-11-05 现跑），快照之后这几份没被改过，主工作区就是开工那一刻的树，没另跑 `snapshot-tree.py` 倒推。
- 引文：`python3 scripts/agents/cite-check.py research/prompts/m3-close-b19-r3fix-code-r1-sonnet-output.md research/prompts/m3-close-b19-r3fix-code-r1-opus-output.md research/prompts/m3-close-b19-r3fix-code-r1-opus-s2-output.md --root /home/fy5090/code/singlefs --background research/prompts/_m3-close-b19-r3fix-code-r1-background.md` 末行「✓ 核了 76 处引文、64 处裸行号，对上 76 处，没判 64 处（其中裸行号没有引文可比 60 处；逐处列在上面）」。三份共同依靠的裸行号主 agent 逐处现读：`crates/singlefs-core/src/recovery.rs:1867`（读根时核落点公式 `root_sits_in_the_ring_slot_the_formula_gives`）、`crates/singlefs-core/src/mount.rs:2808`（重建分配器那一遍同核，注释写明 C743）、`crates/singlefs-core/src/recovery.rs:2230`（被抛弃判 `checkpoint_txg > row.selected_root_txg`）、`crates/singlefs-core/src/root_ring.rs:188`（第 0 代根单独认三个区域的槽 0），都与腿报告的说法相符。
- 模型复跑（腿交回的模型拷到主 agent 草稿目录或照腿给的复跑脚本，经服务跑）：
  - 攻方第一样本 6714：`MODELRESULT name=premises_hold S=4 cases=4515840 pin_maps_differ=737280 oldest_differs=0 … unmount_released_up_to_differs=0 differences_without_a_genesis_root_read=0`，S = 8 同数；对照臂 `root_read_off_its_formula_slot S=8 … unmount_released_up_to_differs=317760`。
  - 正推 6715–6717（拷到 `~/.local/share/singlefs/work/m3-close-b19-r3fix-code-r1-main/sonnet-model`）：「S=1 TMAX=6：核了 10648 个状态，按槽与按 txg 的最旧有效根相同、卸载放开的那一截相同」「S=2 TMAX=6：核了 2744000 个状态…相同」；`--selftest` 找到反例（会红）。
  - 攻方第二样本 6755（拷到 `…/m3-close-b19-r3fix-code-r1-main/opus-s2-model`）：`E1RESULT name=genesis_slot_attribution configurations=8847360 pinned_sets_differ=3420800 oldest_differs=0 reclaim_floor_differs=0 newest_on_every_device_differs=483840 immediate_reclaim_differs=0`；自证 `release_generation_from_zero immediate_reclaim_differs=483840`、`genesis_counted_abandoned oldest_differs=1778272`。
  三条路径不共用代码（Rust 模型拼进 core 副本、独立 Python 枚举、另一份 Rust 模型），各有会红的对照。

## 二、逐格判决

| 格 | 判决 | 依据 |
|---|---|---|
| Q1　三处与 D16（发布语义） 已定项 1 字面 | **对得上（三份一致）**：「环里最旧有效根」「正常卸载」两句按槽认盘之后与字面相符 | 三份腿报告 Q1 |
| Q2.1　写者与 checker 结局不同 | **造不出（三份一致，两份攻方样本）**：checker 对第 0 代根也按读出它的那一槽归盘（攻方第二样本点 `crates/singlefs-checker/src/image.rs:1760`、`walk.rs:21115`、`9065`、`9421`），写者改后算出的数没变 | 三份腿报告；第一节模型 |
| Q2.2　补读那几槽带的槽与真住的槽不同 | **今天走不到（三份一致）**：重读、读坏不是全零、读回全零三种读数带的都是观测到根的那一槽；只有第 0 代根可能错位读而被照收（`root_ring.rs:188`），带错时三处只会少放、不会多放（攻方第二样本，读码） | 三份腿报告 Q2.2 |
| Q2.3　「两种认法算出的数相同」 | **结论成立，理由要换（三份一致，模型上量的）**：撑住它的是 P1 读出的非第 0 代根都住公式那一槽（`recovery.rs:1867`、`mount.rs:2808`，读码坐实）、P2 第 0 代根不被抛弃（`recovery.rs:2230`，读码坐实）、P4 没有释放代为 0 的已释放记录（推的；攻方第二样本说 mkfs 那一次没逐行核），实现员写的「暖机已在每块盘落了根」不是必要前提，在被吞的根槽写下不成立而两边仍相等（两份攻方样本的反例历史，第一节模型 `unmount_released_up_to_differs=0` / `immediate_reclaim_differs=0`） | 三份腿报告；第一节模型 |
| Q2.4　单测与变异 | **测到了改的那三处，组装那一步没罩着（三份一致）**：三行变异在整池上是等价变异，只有函数级单测分得开（读码推的，红在哪三份说法一致）；第 4994、4997 行补读时给根带槽与第 5041、5311 行调用的实参没有变异也没有单测罩着；变异「抬 F 那一遍按 txg 认读出根的槽」的替换串取 `AT_MAKE_FILESYSTEM` 的 S，不是逐字改回改前（改前取池的 S）。**定**：补一条 `readable_roots_for_raising_the_floor` 的函数级用例与罩住带槽那两行的变异，那一行变异改成取池的 S；只改测试代码，随下一批 | 三份腿报告 Q2.4 |
| Q3　别处按 txg 推槽或推盘 | **都不该改（三份一致）**：推的是将来要写的 txg，或碰不到 txg 0；正推另点第 2251 行与第 5257、5263 行一组按槽也同数，不改 | 三份腿报告 Q3 |
| Q1 外带　卸载那一格「两遍只有一遍自证过、这一遍重读仍坏」的槽 | **条款内部有一处没对齐（正推与攻方第二样本都点出，补丁之前就在，与按槽无关）**：代码照 `.claude/kb/decisions/16-发布语义.md`「根槽这一次读坏」一行里「重建分配器两遍里只有一遍自证过的槽」那一句（照挂载时记着的根算），而「正常卸载」那一行末句说这一遍读不出、自证不过的槽当没有根。走到它要两类故障叠在一起，没造盘。两条已定条款字面相抵，不在这一轮判，登 `.claude/kb/checks-owed.md` 交用户 | 正推 Q1；攻方第二样本 Q1 |

## 三、交用户的

无岔路。条款相抵那一处登欠账，由用户在排它的时候定哪一句为准。

## 四、被攻过零轮的

无新形态：mount.rs 那三处改法是第三轮判决定的零轮改法，这一轮三份样本（两份攻方）都没打中，第三轮之后不再为它开轮。

## 五、去向

| 事 | 去向 |
|---|---|
| mount.rs 三处按槽认盘 | 代码轮过：不改 |
| 实现员报告里「暖机」那句理由 | 不改代码；理由以本判决第二节 Q2.3 为准 |
| 函数级用例与组装那一步的变异、`AT_MAKE_FILESYSTEM` 那一行变异改取池的 S | 下一批实现员（测试代码，不开代码轮） |
| 「正常卸载」末句与「重建分配器两遍里只有一遍自证过的槽」一句相抵 | 新登 `.claude/kb/checks-owed.md`，交用户 |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D16（发布语义） 已定项 1 | 备料 | 2026-11-05 不受影响：代码轮核按槽认盘与条款相符；条款内部那一处相抵登欠账交用户 |
