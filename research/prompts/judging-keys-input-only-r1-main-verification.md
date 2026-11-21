# judging-keys-input-only-r1：判决（2026-11-20）

正文 `research/prompts/_judging-keys-input-only-r1-body.md`，开工快照 `research/prompts/judging-keys-input-only-r1-snapshot/`。这是这一题的第一轮（设计推论轮），之前没有判决。两条腿都打中，照 `.claude/rules/three-way-inference.md`「开不开下一轮（设计推论轮）」开第二、三轮；打中的先挂起，不直接写进 D13（验证路线） 正文。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 正推（Sonnet） | `research/prompts/judging-keys-input-only-r1-sonnet-output.md` | 「三样（对拍、哨兵、强制现判）按今天的写法，有四类改动完全或几乎抓不到…判法变弱（红变绿）、观察者函数自己的判法、给定状态与核对器、枚举与状态物化」 |
| 攻方（Opus） | `research/prompts/judging-keys-input-only-r1-opus-output.md` | 「补丁里对拍结论…的键只有输入摘要，判器改了对拍照复用、不重拍」；「最脆弱的是「漏判」类改动（把红判成绿）」 |

核查：

- `task-server/task submit --cwd ~/.local/share/singlefs/work/judging-keys-r1-verdict -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py judging-keys-input-only-r1 --out ~/.local/share/singlefs/work/judging-keys-r1-verdict/judging-keys-input-only-r1-tree --repository /home/fy5090/code/singlefs`：type=command-failed exit=1，「.claude/kb/decisions/13-验证路线.md  倒推不出：diff 里没有它，此刻 HEAD 那一份 sha256 9d42827d55ff 不是快照的 72e6b72b8812」。快照里只点名这一份；它在快照之后的工作区里又被改过、没有提交，倒推不出。腿引它的行（436、440、441、443、444、450、452、458、460、461、464、466、480）拿腿抄的原文在今天的文件里逐处找，人工核。
- `python3 scripts/agents/cite-check.py research/prompts/judging-keys-input-only-r1-opus-output.md research/prompts/judging-keys-input-only-r1-sonnet-output.md --root /home/fy5090/code/singlefs --background research/prompts/_judging-keys-input-only-r1-background.md`：退 1。对不上两处，都是 `.claude/rules/verification.md:41`，原文在今天的第 43 行（规则在快照之后改过，句子没变）；其余是裸行号、没判，坐实结论用到的几处由主 agent 逐处现读。
- 主 agent 现读坐实的（主工作区；补丁副本是 `~/.local/share/singlefs/work/judging-keys-input-data-only/copy`）：
  - 强制现判每趟每流一块：`crates/singlefs-verification/src/crash_amplification/reuse_sentinels.rs:430`「强制现判照同一串数接着从剩下的里抽一块」；哨兵缺省 3 块（同一份第 16 行）。
  - 给定状态表判段写死不强制：`crates/singlefs-verification/src/crash_amplification/given_states_engine.rs:1074` `crate::verdict_store::ForcedRejudging::NotForced`。
  - 绿状态存空向量：`crates/singlefs-verification/src/crash_judge_conclusions.rs:36`「绿的状态是空向量」。
  - 同键两份不同的绿拒存：`crates/singlefs-verification/src/verdict_store/conclusion_blocks/tests.rs:324`，拒绝那一支 `crates/singlefs-verification/src/verdict_store/conclusion_blocks.rs:819` `DifferentConclusionUnderAStoredKeyNotDecidedInTheFirstVersion`。
  - 强制现判那一块，观察者录入块那一行在库里就算录过、不重跑观察者：`crates/singlefs-verification/src/crash_amplification/observer_recording.rs:1016-1019`。
  - 补丁副本：`crash_library/` 目录里 `sentinel|哨兵` 每份 0 处；待判行一行读不出整扫失败（`verdict_store/gpu_pending_rows.rs:340` `from_value_bytes(&value_bytes).ok_or_else(not_in_the_layout)?`）；布局号仍是 6（`verdict_store/library_opening_and_layout.rs:7`）。
  - 补丁副本在两条腿读过之后改过对拍键：`crash_library/pairing_inputs_and_conclusions.rs:371`「改产品恢复，对拍重拍；两边判器与比法自己的代码不进」、`crash_library/pairing_blocks.rs:12`「块的输入数据与判器引到的产品代码相同的下一趟复用」。攻方 A1 里「产品恢复改了对拍不重拍」那一半已不成立；「判器（WGSL、CPU 判器）改了对拍不重拍」那一半照旧成立。
  - 池级 checker 判一块时调产品恢复（攻方 A2）、录入组入口就是事实抽取与打包判表（攻方 B1）：两腿交回当天主 agent 已现查坐实，`records/2026-11-19-判定键只算产品与测试代码.md` 第 98 行。

## 二、判决

两条腿都打中，这一题开第二轮。用户在腿交回之后已定的三格不再挂起：

| 打中的格 | 去向 |
|---|---|
| 攻方 A2：判器闭包里的产品恢复不在崩溃判定的键里 | 用户定「产品代码进键」（记录第 99 行）：崩溃判定的键 = 块的输入数据 + 判器引到的被检测产品代码的指纹；补丁已照做，对拍键同样带它 |
| 攻方 B1、B2，正推「录入漂移」：录入键排出录制管线，事实抽取改了不重录；按包分不开场景与管线 | 用户定录入只是管道、录得全不全是录入检测的课题，录入键由产品代码的屏障决定（记录第 100、102 行）；第二轮只攻这一口径本身判不判得出「录全了」 |
| 正推问 3：录入键留不留工具链与依赖版本 | 用户定编译档与线程数不进键（记录第 105 行）；工具链与依赖版本也不进键是 mainagent(B) 消息里转述的用户补定，第二轮材料里补原话出处 |

挂起、交第二轮攻的（现查坐实；判法照用户「改判器不作废已判的块」不变，缺的是判器改了之后谁发现）：

1. 判器改动的发现手段只剩每流每趟一块强制现判：哨兵 3 块只验复用没坏、不比结论；候选只取不红的块；给定状态表与第 ② 段核对零抽样；对拍按输入复用，判器改了不重拍（攻方 A1、A4，正推 3.1、3.2）。
2. 漏判类（红变绿）三样都看不见：绿状态是空向量，漏判前后逐字相同；红块现判成绿直接换上（攻方 C1，正推「判法变弱」）。
3. 观察者：强制现判不重跑观察者函数；判定项标识不随判法意思变，换号没有能机械判的判据，又撞已定项 17「名字、手写的版本常量不进键」（攻方 B3，正推问 4）。
4. 判法换代没有合规的作废手段：代码指纹与按树算的改动范围都被已定项 17 禁了；判器改动让绿结论内容变时，强制现判那一块存不进、每趟都红、走不出去（攻方 B4、B5，正推问 6 第 4 条）。
5. 枚举与状态物化：块键里是态号，号到持久集合的映射由枚举代码定、不在任何键里（正推 3.1 末条）。
6. 删旧库：旧格式待判行一行就让 GPU 服务整扫失败；布局号没升，没重编的二进制照旧往库里写旧格式；判定向量两张表（`02`、`03`）删不掉（正推 7.2）。
7. 已定项 17 内部的字面冲突：结论种类定值版本与判定项标识算不算「手写的版本常量」；测试服务代码「不进任何键」与单测键「照旧」（正推问 6 第 2、3 条）。

## 三、改法

腿提的改法都只在各自的推理上过，标零轮，第二轮一并攻；都落在验证包、`scripts/` 与 kb，不写进产品代码，不做 A/B。

| 改法 | 修哪一格 | 出处 | 标 |
|---|---|---|---|
| 强制现判从一块改成按比例或按块键哈希轮转取一批，同一个旋钮罩对拍库、给定状态表、核对行与观察者录入；p=1 即「不复用的全量对照」 | 1、4 | 正推 R1–R4，攻方 F1、F5 | 零轮 |
| 每趟两边现判一张阳性样本集（每条不变量至少一张违例镜像），不入库复用 | 2 | 攻方 F2 | 零轮 |
| 判法换代由人登记、经 prune-server 客户端整族删那一种结论 | 4 | 攻方 F7 | 零轮 |
| 计数向量里不变量名与当前清单不同的块答不复用 | 4（B5 那一支） | 正推 R6 | 零轮 |
| 观察者标识带修订号，钉表核观察者指纹（指纹不进键）逼出换号 | 3 | 正推问 4 | 零轮，与「判器那几组入口登记整批删」有张力 |
| 布局号升到 7；删库次序写成「停判段 → 删待判行与崩溃判定各表 → GPU 服务照新代码重起」，`02`、`03` 标保留 | 6 | 正推 7.2、7.3 | 零轮，照用户「删掉」的定案做，不另交用户 |

## 交用户的

第二、三轮攻完再交。要交用户定的岔路到时写进 `research/prompts/judging-keys-input-only-r1-forks.md`：判器改了之后靠哪一样发现（强制现判按比例、阳性样本集、人登记整族删，或几样合用），与观察者换号由谁判。

## 这一轮被判过的文件

- .claude/kb/decisions/13-验证路线.md：已定项 17 的新定案（键只有输入数据、改判器不作废已判的块、靠对拍与复用哨兵抓、录入键只取产品代码加场景、观察者用判定项标识分开），判它在判器、观察者、录制管线、枚举改动下有哪些假绿；打中的七格挂起，见「二、判决」。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 17 | 备料 | 2026-11-20 不受影响：第一轮打中的七格挂起，第二、三轮之后才改正文；用户在腿交回后已定的三格（产品代码进键、录入按屏障、编译档与线程不进键）另由写回改正文 |
