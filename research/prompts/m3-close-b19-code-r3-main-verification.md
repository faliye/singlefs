# m3-close-b19-code-r3 判决（代码轮第三轮，最终轮，2026-10-29）

正文 `research/prompts/_m3-close-b19-code-r3-body.md`；本轮改动 `research/prompts/_m3-close-b19-code-r3-diff.md`；前两轮判决 `research/prompts/m3-close-b19-code-r1-main-verification.md`、`research/prompts/m3-close-b19-code-r2-main-verification.md`。腿：正推 `research/prompts/m3-close-b19-code-r3-sonnet-output.md`（3cd77df5…，模型 `research/prompts/m3-close-b19-code-r3-sonnet-model/`）、攻方 `research/prompts/m3-close-b19-code-r3-opus-output.md`（56d05657…，模型 `research/prompts/m3-close-b19-code-r3-opus-model/`）。

被判的产品代码：`crates/singlefs-core/src/mount.rs`（本轮修补），连同前两轮的 `crates/singlefs-core/src/admission.rs`、`crates/singlefs-core/src/allocator.rs`、`crates/singlefs-core/src/instance_table_sweep.rs`、`crates/singlefs-core/src/mounted_session.rs`、`crates/singlefs-core/src/root_record.rs`。

## 一、核查

- 开工快照倒推：`python3 scripts/agents/snapshot-tree.py m3-close-b19-code-r3 --out ~/.local/share/singlefs/work/m3-close-b19-code-r3-judge/tree`：「清单点名 13 份，倒推出 13 份，sha256 对上 13 份」。
- 引文：`python3 scripts/agents/cite-check.py research/prompts/m3-close-b19-code-r3-sonnet-output.md research/prompts/m3-close-b19-code-r3-opus-output.md --root ~/.local/share/singlefs/work/m3-close-b19-code-r3-judge/tree --background research/prompts/_m3-close-b19-code-r3-background.md`：「核了 68 处引文、52 处裸行号，1 处对不上，74 处没判」。对不上的一处：正推腿第 66 行引 `mount.rs:6133`「一个读错的槽可能是更新的见证，不能当没发生」，那一行原文是注释「设备报读错的槽里装的是什么看不见、可能是一份更新的见证」（`mount.rs:6133`），是转述不是原文，论点与原文一致，判决照原文用。
- 复跑：两腿模型 `tools/rerun.sh`，本机内存排不上，各经 `scripts/hosts/multi-host-run.sh --peer --memory-cap 16G` 拿主工作区（与快照 13/13 相同）的拷贝当冻结树跑，都退 0。攻方：`name=diffs_against_this_leg count=0`，`SAME_AS_EARLIER_ROUND name=v5-r1-opus-probe`、`SAME_AS_EARLIER_ROUND name=v5-r2-u6`，`DIFF_FROM_EARLIER_ROUND name=v5-r2-u2 … changed_lines=2`（修补要改的那一行，预期）。正推：`SAME name=v5-r2-u2-with-the-filter-off`、三行认法恒真变异 `3 ok, 0 FAILED`、恒不剔 `2 ok, 2 FAILED`；`DIFF name=v5-r2-u2`：它拿第二轮存的修补前输出比今天的树，差的正是拒因从「盘 1 偏移 0 长 4096」变成「盘 0 偏移 25067520 长 16384」，与腿自报一致（日志 `~/.local/share/singlefs/work/m3-close-b19-code-r3-judge/rerun-{sonnet,opus}.log`）。
- 现查岔路单：`git show 59ce5a865^:research/prompts/m3-b44-code-r3-forks.md` 第 7 行「也记读坏」定义「取号之前那一判照读错账与读坏账两本判」，第 9 行「两样都做」状态「用户已定（2026-10-18 选这一条）」。
- 核不动的：两块以上补写盘的形（窄盘镜像只有两块盘，`make_filesystem.rs:379`/`384`）；「两种拒同时成立报逐盘核那一拒」在号到顶、空间准入、写行暖机三处预演上（推的）。

## 二、逐格判决

| 格 | 判决 | 依据 |
|---|---|---|
| V1 剔多 | **没打中**（两腿一致：读长 4096 加偏移 0 或槽距只可能是系统配置槽） | 两腿 V1 |
| V1、V2 剔少 | **打中（两腿各自独立、都量过）**：剔除只按「盘号在补写盘表里」判；补写盘之外的成员盘一个系统配置槽撕裂或每次读报错，只进内存那一趟取号必读到、记进账，叠树表每次读坏时拒因点名那块成员盘的槽而不是树表（攻方副本 6 格、正推 4 形）；第二轮新加的回归断言在「补写盘与读坏盘不同一块」那一形下红。结局都是取号之前拒、盘不变，错的是拒因。第三轮是最终轮，改法丙、改法丁都零轮且都收窄用户定的岔路，交用户 | 两腿 V1、V2；复跑 |
| V1 读错账不剔 | **维持，理由改写**：逐盘核收下的盘两槽都读得出，之后那两槽报读错是新事件；理由句照 `mount.rs:6133` 的口径写成「读错的槽里装的看不见，可能是更新的见证」，不写「一定是新故障」；只对补写盘那两槽成立 | 两腿 V1 |
| V2 先后 | **维持，措辞收窄**：「两种拒同时成立报逐盘核那一拒」只对续做预演成立，条文写成「准入与写行暖机预演之后、续做预演之前」 | 两腿 V2 |
| V3 C929 | **维持**（两腿一致没打中）；条文按内容写（不写闸编号），并罩住「补写第一轮落了之后崩溃」那一态 | 两腿 V3 |
| V4 写回清单 | **补全**：前两轮判决的写回清单之外另加：U2 修补改了用户定的岔路一、二的判法（交用户，第三节）；`mount.rs:5638` 文档「两本账同一判」与 5639–5641 自相矛盾，随第三节定案一起改；逐盘核先后的落点放 D18 那两行（正推腿建议，主 agent 写回时现查）；D23（journal 的角色与格式） 已定项 14 的两条前提代码已在引（`mount.rs:4415`、`:4678`）而 kb 里还没有；C929 一句罩补写中途崩溃 | 两腿 V4 |
| V5 | **核得上**：第一轮攻方五行、第二轮 T5 的 420 次挂载第 366 次首扫、U2 修补前后的拒因，在今天的树上都复现（两腿各跑，主 agent 复跑对上） | 两腿 V5；复跑 |

## 三、交用户的

| 岔路 | 候选 | 被攻过几轮 |
|---|---|---|
| 续做预演里只进内存那一趟取号读到坏的系统配置槽，拒因被盖（结局不变、盘不变、错的是拒因）；第二轮修补已剔补写盘两槽，收窄了用户 2026-10-18 定的「两样都做」里「也记读坏…两本判」 | ① 保留第二轮修补（只剔补写盘两槽），确认这一收窄，成员盘坏槽盖拒因登欠账以后再定；② 改法丙：逐盘核那一刻已坏、已报读错的槽两本账都剔（成员盘那一形也修掉，收窄更多）；③ 改法丁：只进内存那一趟取号那几读穿到盘上、不进账（多出的读盘代价没量）；④ 撤回第二轮修补，照原岔路两本账照判，拒因在补写盘那一形照旧报补写盘槽 | ① 两轮（第二轮采、第三轮攻出剔少）；②③ 零轮；④ 是今天之前的形态 |

## 四、被攻过零轮的

改法丙、改法丁；V2 先后措辞收窄后的条文句。照规则第三轮之后不再开轮。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D18（块里携带什么信息） | 备料 | 2026-10-29 不受影响：逐盘核与续做预演的先后一句随写回落到 D18，不改已定条款 |
| D23（journal 的角色与格式） 已定项 14 | 备料 | 2026-10-29 不受影响：两条前提随批末写回 |
