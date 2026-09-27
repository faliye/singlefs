# 定义收尾第二轮判决（defs-m2-closeout-r2，2026-09-26）

<!-- doc-lint:not-numbers E1 E2 E3 F1 F2 F3 F4 F5 F6 F7 F8 F9 F10 F11 F12 F13 F14 F15 F16 G1 G2 G3 G4 G5 G6 G7 G8 G9 H1 H2 H3 H4 H5 H6 H7 H8 H9 O3 O5 O8 O9 O11 O12 -->

## 一、这一轮

- 正文 `research/prompts/_defs-m2-closeout-r2-body.md`，背景材料 `_defs-m2-closeout-r2-background.md`，附录二 `_defs-m2-closeout-r2-diff.md`，开工快照 `research/prompts/defs-m2-closeout-r2-snapshot/sha256sums.txt`（核查员 `sha256sum -c` 24 个全 OK）。
- 腿：辩方 `defs-m2-closeout-r2-sonnet-output.md`；云端攻方 `defs-m2-closeout-r2-opus-output.md`（模型 `defs-m2-closeout-r2-opus-model/`）；本地攻方 `defs-m2-closeout-r2-local-attack-output-s1.md`、`-s2.md` 两份干净。
- 核查员 `defs-m2-closeout-r2-verifier-output.md`：150 处 ✓147 ✗2 核不动 1。攻方 76 处 0 个 ✗，`rerun.sh` 整份复跑承重的探针与演示逐条一致；辩方 1 处行号错位（`:15` 应为 `:25`，文字抄对）；本地攻方运行记录把 s1 的生词数写成 1、应为 2。
- **第一轮判决的两处更正**（原判决照留，更正写在这里）：第一轮判决第一节「96 处 ✓83」应为 ✓82（核查员复核第一轮核查员表三：本地攻方分项 6 个 ✓，不是 7 个）；第一轮把 O9 与 O3、O8 同评「部分」评轻了——O9 的代价（`check.sh`、15 号、74 号起的 cargo 不在任何内存上限里）是现查坐实的现状（辩方 E3）。

## 二、逐格判

| 攻方编号 | 被攻的改法 | 判 | 改法 |
|---|---|---|---|
| H1 | F9（`gate.sh` 整条经内存包装） | **打中，F9 这个方向错**：整轮放进一个 scope，任一阶段撞上限 systemd 就停掉整个 scope、后面的阶段不跑（私有 slice 上缩小版 6 次都退 250）；外层按要的量占账，里层 59 / 87 的包装排不上（252）；上限只能落在一个很窄的窗口里，32 线程时多半是空的（推的） | G1 |
| H2 | F7（输出逐字节不变就删新文件） | 打中：与 69 号绕成死圈 | G2 |
| H3 | F4（原型跑小流不算重型） | 打中：把层 0 流拷成不带 `layer0` 的名字 hook 就放行；上限只限「每次」、分几次能拼回整套；派发闸把主 agent 转述 3b 的自然写法误拒（D1–D3） | G3 |
| H4 | F13（弹窗闸认中文文件名） | 打中：6 句没有出处的断言改后放过；攻方的收严在副本上自证 30 种全过、这 6 句照拒 | G4 |
| H5 | F5（按名字里四个词认违例计数） | 打中：漏 `journal_differing_states` 这类，又把 `*_violations_ok=true`、`layer0_violations=zero` 这类当成「不为 0」要点名 | G5 |
| H6 | F1 × F2（并行收退出码） | 打中：分批时每批复用同一组 `.rc`，某一件在写 `.rc` 之前被停，读到上一批的 0；也没带「派出去多少项就收回多少项」 | G6 |
| H7 | F16（84 号挪到写实验页之后） | 打中：40、86 号同形，只挪了 84 号 | G7 |
| H8 | F10（实现员照跑登记给它的 74 号） | 打中：74 号里面的 `cargo test --release` 不经包装 | G1 一并 |
| H9 | F6（本地辩方改名） | 小：攻方的样本前缀由派发给，辩方写死 | G8 |
| — | F8 的 55 号下限 | 没打中（推的）；真正缺的是 54、57 号的上限算法 | G9 |
| — | F3、F10 的 8G、F11、F12、F14、F15 | 没打中 | — |
| E3 | 第一轮「部分」「小」与延后两件 | O3、O8、O11、O12 判得够；O9 评轻了（第一节更正）；延后两件站得住：提交时都走带 `SINGLEFS_HEAVY_TESTS=commit` 前缀的路径，碰不到这两处 | — |

## 三、改法（被攻过零轮，第三轮攻）

- **G1** 撤回 F9：门禁分诊员照旧直接起 `gate.sh`，不整条包；**起 cargo 的门禁阶段各自在阶段里面经内存包装**：74 号、15 号把阶段里的 `cargo test` 改成经 `research/scripts/run-with-memory-cap.sh`（上限取派发提示或 `REPLAY_MEMORY_CAP` 的默认值），59、87 号今天已经各自逐条包着，照旧。实现员与崩溃验证员跑 74 号时照同一份阶段脚本，里面已经包了，定义里只写「照跑」。`check.sh` 是上游 SOP 的共享脚本，本仓不改，它的 `cargo test --all` 不在内存上限里这一件报给 SOP 会话（第四节）。
- **G2** 撤回 F7 的「逐字节一致就删新文件」：执行员不删任何产物；这一次的输出写新文件名，`research/scripts/replay.sh` 的登记行改指新文件，旧的留着。
- **G3** 攻方第 3b 步：原型里的流只许自己在原型里造，不许从名字带 `layer0` 的用例里拷流；一轮里全部原型跑的全量合起来不超过约 10⁷ 个状态，超过的交主 agent。派发闸对「在原型里调枚举函数跑小流」这类否定或转述写法的误拒，记进第四节。
- **G4** 弹窗闸照攻方的收严改（攻方模型目录里的 `f13fix` 改法），自证照它的 30 种，外加这一轮那 6 句没有出处的断言各一格必拒。
- **G5** 执行员第 4c 步：点名的对象是判决行里表示「没过」的字段——布尔值为 false、取值为 `not_run`、以及数值计数里名字表示违例、不匹配、歧义、失败且值大于 0 的；布尔值为 true、取值为 `zero`、以及名字不表示「没过」的计数（例如 `journal_differing_states`）不点名。拿不准的点名并写「拿不准」。
- **G6** 攻方第 3c 步与共用约束 ④：退出码文件名带批号与件号，每批开跑前清空这一批的文件；收的时候数一遍，文件数与派出去的件数对不上整批作废。
- **G7** 执行员：登记给它的阶段里凡是读实验页的（84、40、86 号）都挪到第 7 步写完实验页之后跑；这一次不写实验页的在第 6 步最后跑，红了照写。
- **G8** 本地辩方的样本前缀与攻方同，取派发给的前缀，不写死。
- **G9** 崩溃验证员「输入」的内存上限：54、57 号各给一个数（没量过的写「推的」），55 号照 F8；主 agent 派发时照给。

## 四、延后与报给别处的

- **上游**：`check.sh`（SOP 共享脚本）里的 `cargo test --all` 不经内存包装，本仓改不了；报给 SOP 会话，并记进 `records/2026-09-16-subagent拆分提案.md` 第四十节。
- **派发闸的误拒**（H3 的 D1–D3：主 agent 用自然写法转述 3b 被拒）：记进第十步「总结与归档」的素材。
- 第一轮第四节延后的两件（重型闸静态分支与版本号白名单、84 号只认 false）照旧延后。

## 五、按路径点名被判的定义与文件（门禁 72 号）

`.claude/agents/three-way-attack.md`、`.claude/agents/three-way-local-defense.md`、`.claude/agents/experiment-runner.md`、`.claude/agents/crash-verifier.md`、`.claude/agents/gate-triage.md`、`.claude/agents/implementation-writer.md`、`.claude/agents/mutation-triage.md`、`.claude/agent-common.md`、`.claude/main-agent.md`，另有 `.claude/rules/implementation-workflow.md`、`.claude/hooks/ask-user-claim-guard.sh`、`.claude/gate.d/74-model-differential.sh`：第二轮判过，打中的按第三节改，改完第三轮攻；`.claude/gate.d/15-*.sh` 随 G1 改、第三轮一起判。

## 六、第三轮（最后一轮）

只攻 G1–G9 的改后字面；第三轮之后停，第三轮里新冒出来的零轮改法写进判决的交用户表、标「零轮」。

## 回看决策

不涉及决策：被判的是 agent 定义、共用约束、主 agent 定义、规则、hook 与门禁，不改 `.claude/kb/decisions/` 里任何一条决策。
