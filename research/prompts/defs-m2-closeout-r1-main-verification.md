# 定义收尾第一轮判决（defs-m2-closeout-r1，2026-09-26）

<!-- doc-lint:not-numbers D1 D2 D3 D4 O1 O2 O3 O4 O5 O6 O7 O8 O9 O10 O11 O12 O13 O14 O15 O16 O17 O18 F1 F2 F3 F4 F5 F6 F7 F8 F9 F10 F11 F12 F13 F14 F15 F16 -->

## 一、这一轮

- 正文 `research/prompts/_defs-m2-closeout-r1-body.md`，背景材料 `_defs-m2-closeout-r1-background.md`，附录二 `_defs-m2-closeout-r1-diff.md`，开工快照 `research/prompts/defs-m2-closeout-r1-snapshot/sha256sums.txt`（核查员 `sha256sum -c` 22 个全 OK）。
- 腿：正推 `defs-m2-closeout-r1-sonnet-output.md`；云端攻方 `defs-m2-closeout-r1-opus-output.md`（模型 `defs-m2-closeout-r1-opus-model/`）；本地攻方 `defs-m2-closeout-r1-local-attack-output-s1.md`、`-s3.md` 干净，`-s2.md` 带损坏（UNTESTED 27 处全拼成 UNTESFED，两道自动闸判绿）。
- 核查员 `defs-m2-closeout-r1-verifier-output.md`：96 处 ✓83 ✗12 分不清 2。云端攻方 50 处 0 个 ✗，`rerun.sh` 整份复跑 61/61 字段与留存一致；正推 11 个 ✗（10 个行号错位，1 个把现行定义的「一层」接进了记录原文的引号，抹平了 D2 要核的那处差异，那一格正推的「一致」不采）；本地攻方核对表开头「59–70 行不在本轮 diff 里」是错的（那十行是这一轮新加的），它逐格核探针行为的结果不受影响。

## 二、逐格判

主 agent 按核查员的 ✓ 采信云端攻方引的原文与探针；推论逐条判：

| 攻方编号 | 格 | 判 | 改法（交修定义的 agent 照做，编号见第三节） |
|---|---|---|---|
| O1 | D1×D2 | 打中：攻方第 3c 步「逐个 `wait "$pid"`」在 run_in_background 里多个作业时每次写检出、叫醒主 agent；共用约束 ④ 只认单独的 `wait`、会吞退出码 | F1 |
| O2 | D1 | 打中：「随机跑批限时」只能靠包装的限时变量，撞限退 253，而共用约束说这一次不算结果 | F2 |
| O3 | D1 | 部分：「崩溃点逐点穷举」读成前缀截断，「checker 判结束状态」读成比层 0 弱；与正推同一处 | F3 |
| O4 | D1×D3 | 打中：3b 让攻方用层 0 那套枚举，共用约束第 46 行无限定地写子 agent 不跑「层 0」 | F4 |
| O5 | D1 | 打中：执行员第 4c 步只点名 false / not_run，判决行里的违例计数（`layer0_violations`、`width_mismatches`、`today_ambiguous` 这类）非 0 照样漏报；门禁 84 号同一个盲区 | F5；84 号那一半记欠账（第四节） |
| O6 | D1 | 打中：本地辩方第 19 行把改名限在「写范围」「产出」两节，攻方「做什么」第 2 步里的核对表名按字面仍是 `-local-attack-…`；辩方被要求替换「输入」一节的项却不读那一节 | F6 |
| O7 | D2 | 打中：执行员第 4 步「跑之前删掉旧输出」与 ⑤「已存在又没进 git 的产物留着」冲突，拆成两条命令能绕过闸删掉未跟踪产物 | F7 |
| O8 | D2 | 部分：崩溃验证员的「输入」没有内存上限，默认落到 8G；55 号同时起 6 台 2048 MiB 的虚机（推的，没量过），峰值表里 54/55/57 一行都没有；正推同报 | F8 |
| O9 | D2 | 部分：门禁分诊员不包 `gate.sh` 的理由（嵌套）与包装脚本第 72 行「嵌套支持」相反，代价是 `check.sh`、15、74 号起的 cargo 不在任何上限里 | F9 |
| O10 | D2×D3 | 打中：阶段归属表把 74 号登记给实现员，实现员第 4 步写「不跑门禁阶段」，三份定义对 74 号各说各的；74 号自己给的建议命令被 hook 拒（H23） | F10 |
| O11 | D2 | 小：共用约束第 60 行「④ 之外前台与 run_in_background 一样拒」，而 ② 在 run_in_background 里只记检出不拒 | F11 |
| O12 | D2×D3 | 部分：拒绝清单漏了 `handback-scratch-check.sh`、`ask-user-claim-guard.sh` 等，而主 agent 定义说拒绝的种类都列在那里 | F12 |
| O13 | D3 | 打中：弹窗断言 hook 只认 ASCII 路径，引中文文件名的 `文件:行号` 被拒，出路又叫人写 `文件:行号`；`.claude/kb` 与 `records` 下已跟踪 273 个文件里 252 个是中文名 | F13 |
| O14 | D3 | 打中：主 agent 被指去当全单的那张重型清单比 hook 窄（裸 `herd7`、`qemu-system-*`、直接执行层 0 二进制、工作区根裸跑 `cargo test`）；57 号自己的 `fetch-deps.sh --check` 对主 agent 不带前缀被拒 | F14 |
| O15 | D3 | 打中：变异分诊第 3 步要 59 号的输出路径，「输入」一节没列 | F15 |
| O16、O17、O18 | D4 | 分析采信（甲、乙各自的误放误拒命令、丙一 / 丙二两条第三条路；只取版本号的裸调用四种判法都要同一张小白名单）；**这一轮不定**：提交时崩溃验证员与门禁分诊员都带 `SINGLEFS_HEAVY_TESTS=commit` 前缀跑，这道闸的过度拒绝只挡子 agent 平时跑静态样本，不挡里程碑二的提交 | 去向见第四节 |
| 正推 D2-e | D2 | 打中：84 号登记给执行员，而执行员第 6 步跑登记阶段在第 7 步写实验页之前，新跑出带 false 字段的实验会在第 6 步被 84 号判「没点名」 | F16 |
| 正推 D3 | D3 | 打中：主 agent 定义两处重型清单改成「以那张清单为准」时没删掉只列六类的括注；「全单以」措辞 | F14 一并改 |

## 三、改法（被攻过零轮，第二轮攻）

- **F1** 攻方第 3c 步与共用约束 ④：并行作业收退出码只写一种——每项把退出码写进自己的文件，最后单独一个 `wait`，按派活的次序逐个读；两处都写这一种，④ 的出路里也写它。
- **F2** 攻方第 3b 步：删掉「限时」；每批的规模按先跑一小段估的时长定，不设包装的限时变量。
- **F3** 攻方第 3b 步：崩溃点照层 0 的枚举域（段内写的整写子集逐个枚举，不是只截前缀），每个崩溃状态都跑 checker。
- **F4** 共用约束第 46 行的「层 0」写成「名字含 `layer0` 的测试二进制与 54 号」；攻方第 3b 步写明：在自己的原型里调 `crates/singlefs-harness/src/crash.rs` 的枚举函数跑小流（每次全量不超过约 10⁶ 个状态）不算重型。
- **F5** 执行员第 4c 步：点名的对象加上判决行里名字表示违例、不匹配、歧义、失败的计数非 0 的（`*_violations`、`*_mismatches`、`*ambiguous*`、`*_failed` 这一类），写清哪一行、哪个字段、值、是不是预期内的。
- **F6** 本地辩方：改名覆盖攻方定义里出现的全部 `-local-attack` 文件名；辩方开工读攻方的「输入」「做什么」「写范围」「产出」四节。
- **F7** 执行员第 4 步：不删旧输出；这一次的输出写新文件名（日期或 `rN`），`research/results/` 下已有的一个都不删、不覆盖；完成标记按这一次的文件名认。
- **F8** 崩溃验证员「输入」加一项必给的：每道的内存上限（55 号按同时起的虚机数与每台的内存算）；主 agent 派发提示那一节写明给崩溃验证员时带上它。
- **F9** 门禁分诊员：`gate.sh` 整条经内存包装（包装脚本第 72 行支持嵌套），上限取派发提示给的；删掉「外面不再包」对 `gate.sh` 的那一句。
- **F10** 74 号：实现员定义第 4 步写明阶段归属表登记给它的阶段照跑（74 号在内），其余门禁阶段不跑；74 号文件里给的建议命令改成 hook 放行的写法（经内存包装），用探针喂一次核它放行。
- **F11** 共用约束第 60 行：写明 ② 在 run_in_background 里只记检出、不拒，④ 只在 run_in_background 里拒，其余两处一样拒。
- **F12** 拒绝清单补上 `handback-scratch-check.sh`（交回前拦没删的编译目录与仓副本）与 `ask-user-claim-guard.sh`（弹窗问句里的断言要带出处），以及修定义的 agent 现查到的、在 `.claude/settings.json` 里注册着会退 2 的其余 hook。
- **F13** `ask-user-claim-guard.sh`：认 `文件:行号` 时路径允许非 ASCII 字符；补一格自证（中文文件名的出处放行、没出处照拒），先证明会红。
- **F14** `.claude/rules/implementation-workflow.md`「重型测试只在提交时跑」开头那张清单补全到与 `heavy-test-guard.sh` 实际拒的一致（裸 `herd7`、`qemu-system-*`、直接执行层 0 二进制、工作区根裸跑 `cargo test`）；主 agent 定义两处只留指过去的那一句，删掉六类的括注，「全单以」改成「完整清单以」。57 号 `fetch-deps.sh --check` 被拒那一格随 D4 延后（第四节）。
- **F15** 变异分诊「输入」加 59 号的输出路径。
- **F16** 执行员：第 6 步的门禁里 84 号挪到第 7 步写完实验页之后跑。

## 四、不在这一轮改的，去向

- **D4（重型闸的静态分支）与 O18 的版本号白名单**：延到里程碑二收尾之后，记进 `records/2026-09-16-subagent拆分提案.md` 第四十节第 40 行的现状（附这一轮的分析：甲、乙、丙一、丙二各自的误放误拒命令，四种判法都要同一张小白名单）。
- **O5 的门禁那一半**：84 号只认 `字段=false`，违例计数非 0 不判；记欠账，延到收尾之后。
- **工具的两处坑**（第十步「总结与归档」的素材，不在这一轮改）：本地腿样本里单字母系统性替换（UNTESFED）两道损坏闸都抓不到；损坏闸连提示文件一起查，提示里带 Rust 路径 `::` 会误红（`m2-layer0-scale-r1` 本地攻方第一次调用作废）。

## 五、按路径点名被判的定义（门禁 72 号）

`.claude/agents/three-way-local-attack.md`、`.claude/agents/three-way-local-defense.md`、`.claude/agents/three-way-attack.md`、`.claude/agents/experiment-runner.md`、`.claude/agents/crash-verifier.md`、`.claude/agents/gate-triage.md`、`.claude/agents/implementation-writer.md`、`.claude/agent-common.md`、`.claude/main-agent.md`，以及 `.claude/gate.d/stage-owners.tsv`：第一轮判过，打中的按第三节改，改完第二轮攻改法。`.claude/agents/mutation-triage.md`（97f5904 那一块只被通查扫过，这一轮 O15 打中）随 F15 改、第二轮一起判。

## 六、第二轮

修完之后开第二轮：只攻第三节 F1–F16 的改后字面（改法本身），攻击面不重复第一轮；派一条辩方腿复核这份判决里「部分」「小」的几格与第四节延后的去向够不够得着。

## 回看决策

不涉及决策：这一轮被判的是 agent 定义、共用约束、主 agent 定义、阶段归属表与两份 hook，不改 `.claude/kb/decisions/` 里任何一条决策的定案、射程或依据。
