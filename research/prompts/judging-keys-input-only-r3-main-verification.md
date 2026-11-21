# judging-keys-input-only-r3：判决（2026-11-20）

正文 `research/prompts/_judging-keys-input-only-r3-body.md`，开工快照 `research/prompts/judging-keys-input-only-r3-snapshot/`。这是这一题的第三轮（设计推论轮，最终轮）；前两轮判决 `research/prompts/judging-keys-input-only-r1-main-verification.md`、`research/prompts/judging-keys-input-only-r2-main-verification.md`。三轮里每一轮都有腿打中「键只有输入数据、判法改了靠对拍与复用哨兵抓」这一套在原样下站不住，按多数判打穿；第三轮之后不开第四轮。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 辩方（Sonnet） | `research/prompts/judging-keys-input-only-r3-sonnet-output.md` | 「已定项 17 加用户后定三格：站得住一半」；未定项清单「够不着」，漏 3 条 |
| 攻方（Opus） | `research/prompts/judging-keys-input-only-r3-opus-output.md` | 「问 1（已定项 17 作为整体）：打中，不能原样当「已定」交」；问 2「漏两条」；问 3「两处对不上，其余逐个对上」 |

核查：

- `task-server/task submit --cwd ~/.local/share/singlefs/work/judging-keys-r1-verdict -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py judging-keys-input-only-r3 --out ~/.local/share/singlefs/work/judging-keys-r1-verdict/judging-keys-input-only-r3-tree --repository /home/fy5090/code/singlefs`：type=succeeded exit=0，「清单点名 1 份，倒推出 1 份，sha256 对上 1 份」。
- `python3 scripts/agents/cite-check.py research/prompts/judging-keys-input-only-r3-opus-output.md research/prompts/judging-keys-input-only-r3-sonnet-output.md --root /home/fy5090/code/singlefs --background research/prompts/_judging-keys-input-only-r3-background.md`：「✓ 核了 65 处引文、40 处裸行号，对上 65 处，没判 40 处」。
- 主 agent 交的读数（`~/.local/share/singlefs/work/judging-keys-r3-readings/summary.md` 与各 `task-<号>.log`）：六个判器入口的单元图闭包，产品单元并集 1770 个、与集成副本键表那一组相同；两个第 ② 段核对入口的产品单元不在镜像判定两入口并集里的 0 个；观察者盘上那一入口只引到 367 个产品单元；`crash_library_reader keys` 的 `CRASH_LIBRARY_STATES recorded_states=93953107474 states_judged_under_the_current_key=0 states_judged_only_under_another_key=6027463981 states_never_judged=87925643493`；两趟计时编译沿用增量编译目录、构建脚本没重跑，读数无效。
- 攻方腿点名的那条单测 `crash_amplification::reuse_sentinels::tests::reuse_sentinel_candidates_leave_out_a_block_whose_observer_conclusion_is_missing`（集成副本，`--exact`）：type=succeeded exit=0，「1 passed」。坐实「候选块不多于哨兵数时没有强制现判那一块」，第二轮判决里「每趟强制 9 块」改成「每趟至多 7 块」，改动均匀落在 1% 的块上一趟抓到约 6.79%（攻方算的）。
- 主 agent 现读：两台工具链在多机起跑前比一次（`scripts/hosts/multi-host-run.sh` 第 1367 行 `peer_host_compare_toolchains`）。

## 二、判决

1. D13（验证路线） 已定项 17 现行正文不能原样当「已定」：两腿逐句判下来，要改的都是把已有的用户定案与集成副本现状写回，逐行清单以攻方报告「一、问 1」那张表为准（432 标题、435、436 加判定项标识的例外、438 两处、444 三处、450、458、459、460、461 拆两行、补观察者结论的键一行、466 射程、依据、欠）。
2. 交用户的六条岔路，用户已在弹窗里全部定了（`records/2026-11-19-判定键只算产品与测试代码.md` 第 118–124 行）：崩溃判定与录入键里工具链与依赖版本也去掉；录入键只留写流与场景；判器改了就整族删重判；测试服务代码一律不进任何键（连它们自己的单测与变异行）；观察者判法意思变了同判器、整族删重判；判段问 GPU 服务收不收临时库那一问删掉。前两轮挂起的格据此去向：
   - 判器改了谁发现、漏判类、判法换代无作废手段（第一轮格 1、2、4）：按用户「整族删重判」收，平时强制现判只当抽检；罩不到的几类登欠账。
   - 观察者（第一轮格 3）：同上。
   - 录入键收宽、录入漂移（第二轮用户格 3）：录入键只留写流摘要与场景，产品单元不进；录全不全的检查今天没有，登欠账。
   - 工具链与依赖（第二轮用户格 4、5）：去掉。
   - 产品代码进键对 GPU 路线盲判（第二轮用户格 1）：产品代码照用户定进崩溃判定的键；GPU 判的块随产品改动重判而判不出产品恢复的差别，登欠账（GPU 路线看不到产品改动），不另交用户。
   - 产品代码那一组收宽（第二轮用户格 2）：构造一在读数下不成立（0 个）；构造二（同一棵树两种构建两把键，差在 `singlefs_core::transaction::lookup_work` 那 5 个只给测试计数的单元）照已定项 17「两份库重复判同一批，同样算违反」判得出，主 agent 定改法（「三、改法」）。
   - 删旧库（第一轮格 6）：布局号升 7，删库与写下新布局行在同一步做。
   - 枚举与状态物化（第一轮格 5）、字面冲突（第一轮格 7）：前一格缩成「共用辅助一起改才逃得掉」，登欠账；后一格判定项标识的例外写进 436 行，测试服务代码与单测键那一处按用户「一律不进」写。
3. 前两轮报的数，两腿逐个对过：「每趟强制 9 块」改成至多 7 块；R1 稳态代价「p ×（84,710 + 32,629）秒」只取了下端，改写成 p × 约 10 万到 150 万秒（攻方算的），「修不了换代」那句结论不变；库里态数以第三轮读数为准（只在别的键下 6.42%，从没判过 93.58%）；变异表点名池级 checker 948 行是集成副本的数、主工作区是 946。其余逐个对上。

## 三、改法

都落在验证包、`scripts/`、两个服务与 kb，不写进产品代码，不做 A/B。用户在弹窗里定的六条不算零轮；下面主 agent 定的两条标零轮，并进写段这一批，由代码轮三方攻。

| 改法 | 落点 | 依据 | 标 |
|---|---|---|---|
| D13（验证路线） 已定项 17 照用户定案与集成副本现状写回，逐行照攻方报告「一、问 1」那张表，岔路那几格照用户六条填 | kb（写回员） | 用户定案（记录第 96、99–105、118–124 行，`records/2026-11-20-拆分带出的门禁与变异红.md` 第 248 行起） | 用户定 |
| 录入键 = 录下的写流摘要 + 场景标识；`RECORDING_SCENARIOS` 那一套从场景入口收闭包、按包过滤的收法删掉；崩溃判定、对拍、观察者与录入各键里的工具链行与 `Cargo.lock` 行删掉 | 验证包构建脚本、`crates/singlefs-unit-graph/src/entry_groups.rs`、录入那一路 | 用户定 | 用户定 |
| 判段问 GPU 服务收不收临时库那一问删掉，连 `gpu-verification-server/control` 的 `serves` 子命令、它的自证格与弄坏开关；服务里装登记写法键的 `version_key` 一类名字跟着改 | 验证包 `crash_amplification/gpu_verification_server.rs` 与调用处、`gpu-verification-server/` | 用户定 | 用户定 |
| 整族删重判：prune-server 客户端加按结论种类删一族的子命令，停判段、删完在同一步写下新布局行，删前列两台持认领的持有者；布局号升到 7 | `scripts/prune-server/prune_server_client.py`、验证包 `verdict_store/library_opening_and_layout.rs` | 用户定（整族删）；布局行谁写下照第二轮攻方 R6 | 用户定，删法细节零轮 |
| 判器引到的产品代码那一组只按一组固定的 cfg 算（验证包判段那一趟编的特性），不让带不带 `lookup-work-counters` 的两种构建算出两把键 | 验证包构建脚本 | 已定项 17「两份库重复判同一批，同样算违反」 | 零轮 |
| 测试服务代码不进按条复用的测试函数键与变异行的点名测试闭包 | 算键库与变异判法（别的会话的文件，问过 mainagent(A)、mainagent(B) 归谁） | 用户定 | 用户定 |

## 交用户的

六条已由用户定（记录第 118–124 行），岔路单 `research/prompts/judging-keys-input-only-r1-forks.md` 不再另写。

## 这一轮被判过的文件

- .claude/kb/decisions/13-验证路线.md：已定项 17 现行正文加用户后定三格作为整体能不能当「已定」，逐句判照留、要改、要用户定（要改的逐行清单在攻方报告「一、问 1」）；前两轮交用户的岔路清单补全成六条并由用户定完；前两轮报的数逐个对过读数。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 17 | 推翻 | 2026-11-20 改了：「判法改了对不对，靠对拍与复用哨兵抓」「录入键只取产品代码加场景」「其余键由函数单元、工具链、依赖版本组成」三句被推翻，照用户六条与攻方报告「一、问 1」那张表写回 |
