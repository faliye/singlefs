# m3-close-b19-core-r1 判决（第一轮，2026-10-29）

正文 `research/prompts/_m3-close-b19-core-r1-body.md`；腿：正推 `research/prompts/m3-close-b19-core-r1-sonnet-output.md`（706b4b68…）、攻方 `research/prompts/m3-close-b19-core-r1-opus-output.md`（cca20134…）。

## 一、核查

- 开工快照倒推：`python3 scripts/agents/snapshot-tree.py m3-close-b19-core-r1 --out ~/.local/share/singlefs/work/m3-close-b19-core-r1-judge/tree`：清单 9 份，倒推出 8 份、sha256 对上 8 份；倒推不出 1 份：`.claude/kb/checks-owed.md`（快照之后被这一批改去向那几次写过，HEAD 那份不是快照那份）。两条腿引欠账表的只有各格登记行的编号与前置格，主 agent 对着此刻的表逐处人工核过，C773、C724、C745、C874 四行的条款字句开工之后没被改。
- 引文：`python3 scripts/agents/cite-check.py research/prompts/m3-close-b19-core-r1-sonnet-output.md research/prompts/m3-close-b19-core-r1-opus-output.md --root ~/.local/share/singlefs/work/m3-close-b19-core-r1-judge/tree --background research/prompts/_m3-close-b19-core-r1-background.md`：「核了 60 处引文、80 处裸行号，对上 40 处，没判 100 处」，判了的 0 处对不上。没判的多是快照之外的文件（`transaction.rs`、`mounted_session.rs`、`crash_judge_checker.wgsl` 等），判决要用的几处主 agent 在主工作区现读：`mount.rs:3944`、`:3987` 两处 `.ok_or(RecoveryFailure::NoValidRoot)?`；`mounted_session.rs:2257` 起 `run_compaction_with_the_region_check_and_a_hook_in_the_middle_of_each_batch` 先核盘表与版本，不看 `allocator.frozen_publish()`，随后照常 `discard_the_segments_reusable_as_a_whole`（`mounted_session.rs:2913`），后者同样不看冻结；同文件 `:2792` 推环入口才判冻结。
- 复跑：正推腿模型拷进 `~/.local/share/singlefs/work/m3-close-b19-core-r1-judge/sonnet-model`，经 `run-with-memory-cap.sh 4G` 跑 `tools/rerun.sh --frozen <倒推的树>`，退 0，`model-output.txt` 与腿交的逐字节相同（sha256 75e15679…）：「current total 15958404 violations 829794 violations_with_first_txg_le_3S 0 slot_indexes_holding_genesis_in_violations [0]」「genesis_zero total 15958404 violations 0」。攻方腿没写模型。
- 核不动的：攻方 K3-1 第 2 步（冻结后整理发 discard 时，段里真是冻结那次已落盘的根引用的单元）两条腿都没量，主 agent 只核了「入口与 discard 都不判冻结」这一半。

## 二、逐格判决

| 格 | 判决 | 依据 |
|---|---|---|
| R1 C773 | **采乙**：第 0 代根所在槽（槽号 0）的钉值取 0，p 照存公式槽 `3·槽 + 区域`；写者把（钉值, 槽）一起往下带到写根那一刻、平局按（钉值, 槽）破；池级 checker `walk.rs` 的 `pinned_ring_slot_of_the_root` 与 GPU 走读内核（攻方指出 `crash_judge_checker.wgsl` 两处）各改一处；D16（发布语义） 已定项 1 补一句「槽号 0 的钉值是第 0 代根的 0」。甲（改存槽号）与乙信息量相同而多动格式，不采；丙（认已知结局）两腿都判回收门槛会越过第 0 代根、不只是误红，不采 | 两腿一致推荐乙；正推腿模型复跑：今天取法 829794 违例全在坏槽号 0、乙 0 违例（模型宽于真实历史，只证乙充分）；正推腿另报变异「把钉值 0 直接当 p 写出」I-7.18 仍绿、要 first_txg = 3S 的用例才现形，补这一条 |
| R2 C724 | **采「写明走不到」**：`mount.rs:3987` 结构上被 `:3944` 先拦（`valid` 非空时 `newest_per_device` 非空），改成 `expect` 并写理由；`:3944` 在本进程准入路径上走不到的两条前提写进代码注释与 D23（journal 的角色与格式） 已定项 14 射程；共享函数保留这个错（有外部调用方）；不加进拒因单子 | 两腿一致；正推腿另指「加进单子」是类别错（这两处在写行之后跑，判据在崩溃镜像上复算不了）；前提 b「根带的 F 不高于它自己的 txg」正推腿没找到条款原文，是推的，实现时补一条断言钉住 |
| R3 C745 | **题面两句没打中**（攻方读码：写根四条路径都经记根或整次挂载报错、分配器随错丢掉；「旧记录被拿去顶上」只在直接调公开的准入抬 F 上限函数时走得到、不写盘）；没打中是一次抽样，照规则不当结论，C745 留着、下一轮再攻一次。**新线索 K3-1 采为欠账**：冻结着时整理入口照跑、照发 discard，主 agent 现查入口与 discard 都不判冻结属实；后果（discard 掉冻结那次已落盘的根引用的单元、重发前断电则恢复择到的根单元已没了）没量，先写会红的用例（系统配置槽轮换写注入 `WriteFails`、冻结后调 `run_compaction`、断言 discard 不含冻结那次落点所在的段），红了再改（改法候选：discard 开头判冻结、或整理入口第一道判冻结，零轮） | 攻方腿；主 agent 现读 `mounted_session.rs:2257`、`:2913`、`:2792` |
| R4 C874 | **采可观测那一半**：补写闸没过的原因进挂载错误成员的载荷（新的公开封闭枚举：闸前读错加五件闸，只报第一件、第一块盘，条文写明），成功一支在挂载输出加补写过的盘；core 没有日志设施，不做观测记录行。崩溃流罩取号写两轮的两段（各 8 个真子集态），回卷那一支用 `WriteFails` 用例罩。**「只判整写不判撕裂」那行变异分歧**：正推腿放枚举侧（`crash.rs:2325`）让撕裂退化成新写、点名新流测试；攻方腿判真实补写流两轮之间有屏障、撕裂造不出新死盘、这行杀不掉，要合成态用例而那与 C874「不手造写表」字面冲突——分歧点是撕裂态在真实流上造不造得出，交第二轮 | 两腿报告 R4 一节 |

## 三、交用户的

无（R4 变异那一点进第二轮；K3-1 进欠账、先写会红的用例）。

## 四、被攻过零轮的

R1 乙的 GPU 内核改法细节、R2 的两条前提断言、K3-1 的两个改法、R4 错误成员的枚举形态——实现交回后并进这一批的代码轮三方攻。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D16（发布语义） 已定项 1 | 备料 | 2026-10-29 改了：R1 采乙，补「槽号 0 的钉值是第 0 代根的 0」一句，随实现那一批写回 |
| D23（journal 的角色与格式） 已定项 14 | 备料 | 2026-10-29 改了：R2 采写明走不到，射程补两条前提，随实现那一批写回 |
