# 工具与治理批的零轮形态：第二轮判决（2026-10-13）

<!-- doc-lint:not-numbers X1 X2 X3 X4 X5 X6 X7 L1 L2 L3 L4 L5 L6 L7 M1 M2 M3 M4 M5 M6 M7 O-1 O-2 O-3 V1 V3 V5 V7 V8 -->

正文 `research/prompts/_m3-defs-b6-r2-body.md`；腿：辩方 `research/prompts/m3-defs-b6-r2-sonnet-output.md`（sha256 badae1cd…0efc，模型 `research/prompts/m3-defs-b6-r2-sonnet-model/`）、攻方 `research/prompts/m3-defs-b6-r2-opus-output.md`（sha256 96e8e4b0…7bec，模型 `research/prompts/m3-defs-b6-r2-opus-model/`）。第一轮判决 `research/prompts/m3-defs-b6-r1-main-verification.md`。

被判的定义与规则文件（按路径）：`.claude/main-agent.md`、`.claude/agent-common.md`、`.claude/rules/three-way-inference.md`、`.claude/agents/clerk.md`、`.claude/agents/experiment-designer.md`、`.claude/agents/experiment-runner.md`、`.claude/agents/implementation-writer.md`、`.claude/agents/investigator.md`、`.claude/agents/mutation-triage.md`、`.claude/agents/three-way-attack.md`、`.claude/agents/three-way-defense.md`、`.claude/agents/three-way-forward.md`、`.claude/agents/tooling-writer.md`。

## 核查记录（主 agent 做）

- 开工快照：第二轮、第一轮两份冻结副本对各自快照非 OK 0 行；辩方报它在两份冻结副本的 `.claude/gate.d/` 下留过 `__pycache__` 又删掉，主 agent 现查两份里比正文新的 `.pyc` 0 份。
- `cite-check.py <两份报告> --root <第二轮冻结副本>`：85 处对上 56、没判 29（都是引主工作区 `research/prompts/` 下的判决与模型输出）；换 `--root` 主工作区：「✓ 核了 85 处引文，对上 85 处，没判 0 处」。
- 攻方模型经 `run-with-memory-cap.sh 2G` 复跑（要设 `R1_FROZEN`）：25 份二十三 same，两份 x6 只差长说明按字节截断的位置，判定与退出码列逐行相同。辩方模型同样复跑：45 份全 same。
- 门禁格主 agent 现跑（`SINGLEFS_HEAVY_TESTS=commit`）：攻方四组（两份冻结副本）结局与腿推的逐条相同——X2 样本第二轮多点名 `or_literal_default.sh`、两版都不点名 `$var` / `$((…))` 回退与 f 串名等；把一行新读插进已被「代码批」行放行的 `random_histories.rs` 两版都绿，同一行写进新文件第二轮红；X5 第二轮 private-values 打码、同一道的 clock-times 红行与 prime-marks、session-names 的「二进制：」行仍把（编的）私有值原样打出 3 处；X7 改脚本正文同时重钉 sha256，long-run-header-exempt 绿且写「没涨」。辩方四组：v3 九种写法第二轮全点名（第一轮只点名 3 处）；x2 的 8 份误判样本（字符串比较、名字元组、字典实参、未用常量、argparse、rust 比较与 match、`&&` 夹下限）第二轮全点名、第一轮 0 处；exempt-table 树红在缺路径、已写齐、sha256 对不上、旧格式四处；abandoned-rounds 六棵树五棵与推的相同（legacy-sourceless 红、legacy-listed 绿、other-name-unrecognised 绿、other-name-recognised 红、other-form-nomaterial 绿），symlink-source 那棵缺腿文件、门禁格退 77 无对象（样本缺件，不是判错；⑥ 那一侧以腿的库读数为准）；pv-tar-mirror-red 第二轮红且不漏值。
- O-3（排除表比第一轮多 2 行）：主工作区 HEAD 里没有 `.claude/heavy-thread-exempt`（`git cat-file -e HEAD:…` 不在），提交那一次基准里没这张表，只缩不涨不比，不会红；不算改过头。

## 逐格判决

| 格 | 判 | 依据 | 去向 |
|---|---|---|---|
| X1 ⑮ 只认整数 | 部分站不住 | 括注一概拒、「线程上限说明：」不被当第二个线程上限都修到（辩方）；第一轮 V1 乙那几句挪进「线程上限说明：」一行照放、照样送到子 agent 手里（攻方 X1 甲、辩方，改前也中）；代码块里的示例行也被拒（攻方 X1 乙，多拒）；`dispatch-prompt.py` 第 20 行仍把 clerk 叫「书记员」（辩方） | 改法 M1 |
| X2 heavy-thread-defaults 宽判法 | 部分站不住 | 第一轮 V3 九种写法与 `lib.rs:8` 修到（辩方，主 agent 跑样本坐实）；代价：8 种不读环境的写法回来成误判（辩方 x2 样本，主 agent 坐实）；已被「代码批」行放行的文件里新加的读被那一行吞掉（攻方 X2 乙，主 agent 坐实，这一批带进来的）；`|| X=$var`、`|| X=$((…))` 不点名（攻方 X2 甲）；`printenv` 读法两版都漏 | 改法 M2 |
| X3 重型测试闸 | 部分站不住 | `env -iC`、`env -S` 与 48 例 env 矩阵修到（辩方）；拒绝说明让主 agent 直接带 `user-request` 前缀跑改过的自证，与 `.claude/rules/implementation-workflow.md` 第 53 行「提交之外要跑先弹窗问用户」相抵（攻方 X3 丙，这一批带进来的）；`env -S` 里 `${…}` 展开、`-p` 的值写在变量里放行（攻方 X3 甲乙，改前也中） | 改法 M3；X3 甲乙另立账 |
| X4 叫停轮 | 部分站不住 | 日期根修到、门禁与 ⑥ 同读一份旧行名单（辩方、主 agent 跑树坐实）；「别的轮名」只认仓里有材料的名字，形如 `other-r2` 而仓里没有材料的第一轮拒、第二轮收（辩方 O-1，改过头），规则文字写「不只认 -r<数字> 形」而代码不再认这一形 | 改法 M4 |
| X5 private-values | 站得住（这一格） | 整串打码修到、6 种路径形状无误报（辩方，主 agent 坐实不漏值）；同一道 doc-text 的 clock-times 红行、prime-marks / session-names 的「二进制：」行原样打出含私有值的路径（攻方 X5 甲，改前也中） | X5 甲并进改法 M5 |
| X6 ⑭ 规范句 | 站不住 | 定义原话被拒的问题修到（辩方）；规范句后接「；按此刻空闲现算」「；机器忙时按负载降」「；除以此刻同时在跑的重活数」与 host-resources、available_parallelism，⑭ 与 long-run-check 都放、第一轮都拒（攻方 X6 甲、辩方 O-2，改过头）；说明里写 E171、layer0、半角分号或规范句折成两行都被拒（攻方 X6 乙，多拒） | 改法 M6 |
| X7 豁免表 sha256 | 部分站不住 | sha256 入表并核、CRLF 判红、执行位不影响（攻方、辩方）；改脚本正文同时重钉 sha256，exempt-check 绿且写「没涨」，表是自证（攻方 X7 甲、辩方）；规范句文件头加现算正文三道全过（V8 甲，判别子观测不到，定义已写由写的人保证） | 改法 M7；V8 甲另立账 |

## 改法（全部被攻过零轮；进批 8，改完开这一题第三轮（最终轮））

| 改法 | 改什么 | 修的格 |
|---|---|---|
| M1 | 子 agent 只照「线程上限：」那一个整数起重活，提示里别处写的线程数、倍数、折算一概不照（写进共用约束与九份定义「输入」）；「线程上限说明：」那一行删掉这一说法，说明照普通正文写；代码块（三个反引号围起来的）里的行 ⑮ 不判；`dispatch-prompt.py` 第 20 行改叫 clerk 的定义名 | X1 |
| M2 | 排除表按被判行匹配：锚串要落在那一条被判行里才放那一行，不按文件并组；`|| X=$var`、`|| X=$((…))` 认成钉数；`printenv NAME` 认成读；第二轮新回来的误判逐行判，不读环境变量的进排除表并写理由（宽判法的代价由排除表兜，判法不再收窄） | X2 |
| M3 | 重型测试闸仓副本那一支的拒绝说明不直接写「带前缀跑」，改成指到 `.claude/rules/implementation-workflow.md` 重型测试那一条的做法 | X3 |
| M4 | 「别的轮名」= 仓里认得的轮名全集 ∪ 形如 `<名>-r<数字>` 的写法，两样都拒；规则文字照这个写 | X4 |
| M5 | doc-text 各格打印路径都经 private-values 那一份打码（抽成共用函数），clock-times 红行、「二进制：」行不再原样打私有值 | X5 甲 |
| M6 | 规范句后面不许接任何说明：`# long-run-hosts:` 与「多机：」那一行逐字等于规范句（可带句末句号），说明另起一行写；定义写明 | X6 |
| M7 | exempt-check 的只缩不涨按（路径、sha256）成对比：基准那一版里同一路径的 sha256 变了也算涨，判红；E171 下一次修订的新入口不进表 | X7 |

另立一笔账（改前也中，不拿来否这批改动）：X3 甲（`env -S` 里 `${…}` 展开）、X3 乙（`-p` 的值在变量里）、V8 甲（文件头读法与正文行为不一致静态看不出）、X1 乙（代码块示例行，M1 一并改）。前两轮判决的另立账照旧。

## 回看决策

不涉及决策：这一轮判的是 agent 定义、共用约束、项目规则的文字与钩子、门禁格的行为，没有一处引用或改动 `.claude/kb/decisions/` 里的分项。

## 更正（2026-10-13，第三轮正推腿 Y6-30、Y6-34 坐实，见 `research/prompts/m3-defs-b6-r3-main-verification.md`）

- 核查记录 O-3 那一条「排除表比第一轮多 2 行」：数据行两边都是 66（总行数 81 对 83，多的是 1 行注释与 1 行空行）；变的是键（路径:锚串），第二轮新 2 个、同时少 2 个。
- 逐格判决 X3 那一行括注引的「提交之外要跑先弹窗问用户」不是 `.claude/rules/implementation-workflow.md` 第 53 行的原话（那一行写「任务确实要跑时主 agent 先弹窗问用户」），那句出自 `.claude/hooks/heavy-test-guard.sh` 的拒绝说明。判决方向不变。
