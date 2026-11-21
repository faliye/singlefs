# gpu-placement-measures-governance-r1：判决（2026-11-19）

正文 `research/prompts/_gpu-placement-measures-governance-r1-body.md`，开工快照 `research/prompts/gpu-placement-measures-governance-r1-snapshot/`。这一题的第一轮。调度记录 `records/2026-11-19-产品GPU包测试与E202装置归位.md`。被判文件快照 20 份派腿之前记；写判决前 `sha256sum -c` 有 2 份对不上：`.claude/gate.d/stage-owners.tsv`、`.claude/gate.d/break-switch-replay-files.tsv`，都是别的会话快照之后往登记表里加了行，这一轮判的只是这一路自己加的那几行，那几行没变。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 正推（sonnet） | `research/prompts/gpu-placement-measures-governance-r1-sonnet-output.md`（sha256 c7abd8d9…） | 产品包放 E202 二进制、测试 `#[path]` 引验证包拦得住；空库只拦空库形态；「搬得超出定案、包名泛称」没有检查；钩子认不出假「标准：」与换说法；写闸拦不住 Bash；`directory-layout.md:42` 自称；新一节第 20 行没给验证包留口子；`verification.md:9` 与 D13（验证路线） 已定项 15 冲突；不以 `e<数字>_` 起头的装置看不见 |
| 攻方（opus） | `research/prompts/gpu-placement-measures-governance-r1-opus-output.md`（sha256 99427b7c…） | 放置类钩子误伤与漏放；整包滞后行与只判八格名单不缩；crate-roles 三种认法与 ⑨ 各有绕法（⑨ 九种写法判绿）；「已经量的」几句出处不全；产品包一律不许二进制越位（将来的 FUSE 薄适配）；单元图登测试支撑却被产品包构建依赖 |

- 攻方腿的 48 份样本主 agent 复跑：把它的模型目录拷进 `~/.local/share/singlefs/work/gpu-placement-measures-governance-r1-verdict/model`（5 份 sha256 与腿报告逐个相同），交 `bash <副本>/tools/rerun-from-a-clean-work-directory.sh <仓根> <快照目录> ~/.local/share/singlefs/work/gpu-placement-measures-governance-r1-verdict-run <副本>`，结局 type=succeeded exit=0，日志「name=summary cases=48 mismatches=0」：攻方腿预言的 48 个结局全部坐实；「name=judged_files_check exit=1 lines_not_ok=3」是快照之后被改的登记表（见上一段）。
- 正推腿报告里喂钩子的那几条命令没有另跑：它们判的形态被攻方 48 份样本罩住（放置类钩子红绿、写闸 Write 与 Bash 两路、crate-roles 对真仓）。
- 两条腿的 cite-check 末行：正推「核了 24 处引文……对上 24 处」、攻方「核了 51 处引文……对上 51 处」，引文没有对不上的。

## 二、判决

打中的都坐实（攻方 48 份样本逐个对上）。逐条去向：

1. **放置类钩子**（攻方误伤格，坐实）：词表里四个词自带「包」、「包括」也算「包」，问实验包起名、问句带轮名「装置归位」、说明写「包括」的弹窗被误拒；「放在哪个包」「搬到」、偏离只写在选项标题里、问「哪个目录」「哪个 crate」被放过。`directory-layout.md:25` 写放置类包括「哪个目录」，钩子不认。改：判法照规则那一句重写，认「哪个包 / 哪个目录 / 哪个 crate / 搬到 / 放在」与选项标题，「包」按整词认、不认「包括」，钩子与规则用同一份词表。
2. **crate-roles 判法**（攻方判法格，坐实）：库里一行 `pub use` 就不算空库、`cfg(any())` 下的依赖凑得够被依赖者数、嵌在 `crates/<包>/<子目录>/` 的包看不见、不以 `e<数字>_` 起头的装置（真树上 `deletion_write_dump`、`new_pool_file_creation_region_bytes`，E172、E142 起的）看不见，`examples/`、`src/main.rs`、指到 `src/bin/` 外的 `[[bin]]`、连字符名字绕得过。改：验证包与产品包的每个二进制目标都登记（验证包只许门禁工具那几份），没登记的判红，不按名字形状认；空库按「去掉 `pub use` 与注释后没有条目」认；被依赖者数只数无条件依赖；递归找 `crates/` 下全部 `Cargo.toml`。
3. **⑨**（攻方判法格，这一轮最脆弱的一条，坐实）：同一处跨包引用换九种写法都判绿（`cfg_attr`、宏改名导入、方括号、原始字符串、行内模块里的 `#[path]`、绝对路径、经非 `.rs` 中转、`target` 目录、点开头目录）。改：不在源码字面上认写法，改判编译产物：对每个包取 `cargo` 的 dep-info（`.d` 文件）列出的源文件，落在别的包目录里的判红；做不到就逐一把九种写法补进判法与样本。改法由工具员先给两种做法的代价，主 agent 定。
4. **滞后表与名单**（攻方误伤格，坐实）：`crate-roles-lag.tsv` 测试包整包一行连之后新加的东西也放过，拆包没有截止日；只判八格的名单不核文件在不在、永远不缩；截止日 2026-11-20 比规则生效晚一天，标题里先写一个更早的日期就能少判。改：整包行换成逐个目标的行；名单里的文件不在了判红；截止日改成换上那一天，那一天已有的只有八格的正文登进名单；「标题第一个日期」认法的漏洞写进 `.claude/rules/directory-layout.md`「门禁管哪一半」靠 review 那一栏。
5. **写闸 Bash 一路**（两条腿都报，坐实）：检出钩子 ⑭ 加同一半（调度记录里已定）。
6. **「搬得超出定案、包名泛称」没有检查**（正推第 1 格，推不出、坐实没有）：包名泛称由 crate-roles 的「测试支撑包被依赖者不到 2 个」与「空库加测试目标」间接罩住；「执行超出定案」没有机器判得了的形态，写进规则「门禁管哪一半」靠 review 那一栏，不立检查。
7. **规则正文**（正推第 5 格，坐实）：`directory-layout.md:42`「这份规则」是自称，改成名字；新一节「一个包的集成测试放它自己的 `tests/`」加一句验证包的跨包验证与崩溃枚举用例住验证包 `tests/`（与 `scripts/crash/crash-case-check.py:26` 一致）；`verification.md:9` 与 D13（验证路线） 已定项 15 的旧话随拆包第二簇的写回一起改（调度记录已定）。
8. **越位**（攻方，推的，主 agent 采）：「产品包不放装置二进制」写成「产品包不放实验装置与测试二进制」，不禁产品自己的可执行入口；单元图的角色登成「测试支撑」而产品包构建依赖它，角色加一种「构建支撑」或把单元图登成验证一侧的工具，交工具员给出两种的改动面，主 agent 定。
9. **「已经量的」出处**（攻方，坐实）：放置类钩子 10 条与写闸开关复跑的条数，调度记录里只写了结局、没抄成功句；code-round-checklist 的「绿」在真树上第九格判的是 0 份（截止日之后还没有代码轮正文）；`git-hooks-install.py:133` 的行号已漂到 `:154`。这几句在调度记录里补出处与限定，不改结论。

这一轮有打中，照 `.claude/rules/implementation-workflow.md`「代码轮与治理文件轮怎么收」改完再攻一轮；第二轮派一条辩方腿复核这份判决。

## 三、改法

- 「二、判决」第 1–5、7–8 条的改法交下一件工具员，都是零轮，改完进第二轮攻；不改产品代码，不做 A/B。第 3 条的两种做法与第 8 条的两种做法，工具员先报改动面，主 agent 定。
- 第 6、9 条不改检查：第 6 条写进规则靠 review 一栏，第 9 条补调度记录。
A/B 读数：任务 <号>

## 交用户的

这一轮没有要交用户定的。

## 这一轮被判过的文件

- .claude/rules/directory-layout.md：判了新一节的放置类定义与钩子不一致、第 42 行自称、集成测试那一条没给验证包留口子、门禁管哪一半要补靠 review 的两样
- .claude/rules/implementation-workflow.md：判了检查单第九格的格名与说明，与 doc-process-records 的判法一致
- .claude/rules/verification-reference.md：判了「门禁管哪一半」新加三行与三格判法一致
- .claude/skills/three-way/SKILL.md：判了检查单表新加的「放的位置」一行与第九格判法一致
- .claude/gate.d/code-tooling.sh：判了 crate-roles 格的接线与 e173 自证接进 runner 表
- .claude/gate.d/checker-independence-and-sync.sh：判了 ⑧ 允许共用的新增与 ⑨ 的字面认法有九种绕法
- .claude/gate.d/doc-process-records.sh：判了第九格截止日晚一天、只判八格名单不核文件在不在、标题第一个日期的认法
- .claude/gate.d/stage-owners.tsv：判了 crate-roles 那一行的归属
- .claude/gate.d/crate-roles.tsv：判了 11 个包的角色登记，单元图登测试支撑与产品包构建依赖它相悖
- .claude/gate.d/crate-roles-lag.tsv：判了测试包整包一行放行得过宽、没有截止
- .claude/gate.d/crate-roles-allowed.tsv：判了 e165 一行理由站得住
- .claude/gate.d/crate-cross-references-allowed.tsv：判了 4 对放行理由站得住
- .claude/gate.d/crate-cross-references-lag.tsv：判了 2 对滞后理由站得住、能缩
- .claude/gate.d/break-switch-replay-files.tsv：判了新登的两行，开关数随钩子改动要更新
- scripts/gate/crate-roles-check.py：判了空库、被依赖者数、二进制认法与包的发现各有绕法
- scripts/gate/README.md：判了新脚本那一行的说明
- .claude/hooks/ask-user-placement-guard.sh：判了词表误伤与漏放、与规则的放置类定义对不上
- .claude/hooks/write-guard.sh：判了第十一道认已有门禁的办法站得住，Bash 一路由检出钩子 ⑭ 补
- .claude/hooks/selftest-registry.tsv：判了新钩子与新脚本的自证登记
- .claude/settings.json：判了新钩子挂在 PreToolUse 的 AskUserQuestion 上

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 15 | 不影响 | 2026-11-19 不受影响：这一轮判的是检查与规则，D13 已定项 15 的改写归拆包那一题的写回 |
