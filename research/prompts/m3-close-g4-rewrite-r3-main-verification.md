# m3-close-g4-rewrite-r3 判决（推论轮第三轮，最终轮，2026-10-22）

正文 `research/prompts/_m3-close-g4-rewrite-r3-body.md`，背景材料 `research/prompts/_m3-close-g4-rewrite-r3-background.md`（sha256 d9f3a650063e837a657caeb0472d93c5759f17cde729928036d38fa01308c9eb）。开工快照 `research/prompts/m3-close-g4-rewrite-r3-snapshot/`（`code-sha256.txt` 七份、`kb-sha256.txt` 十二份）。前两轮判决 `research/prompts/m3-close-g4-rewrite-r1-main-verification.md`、`research/prompts/m3-close-g4-rewrite-r2-main-verification.md`。

两条腿：
- 正推（Sonnet）：`research/prompts/m3-close-g4-rewrite-r3-sonnet-output.md`，sha256 15e9a63428997f2f5c46b85ef029db79bf3338a282757333073afc0a5b8e5670。模型目录 `research/prompts/m3-close-g4-rewrite-r3-sonnet-model/`。
- 攻方（Opus）：`research/prompts/m3-close-g4-rewrite-r3-opus-output.md`，sha256 b29a35b7481a8b80ecdbb912e1b937a34560ffd6c742195b78779e0b07bce05c。模型目录 `research/prompts/m3-close-g4-rewrite-r3-opus-model/`。

## 一、核查

1. 快照：两腿交齐之后 `sha256sum -c` 两份清单，三份对不上，倒推出快照时的原样再核。倒推材料存在 `~/.local/share/singlefs/work/m3-close-g4-rewrite-r3-main-checks/`。
   - `crates/singlefs-checker/src/image.rs`、`crates/singlefs-checker-tier/src/crash_injection.rs`：开工之后被里程碑四预开发会话的 m4-i315-r3 补丁改过（它交来的 unified diff `for-3c-snapshot.diff`，sha256 741c80778b741d2289a0cf48f5b00e17916b7541d487acc202c220cb23cb9330，7 个 hunk）。它交来的改前整份（`before-main-apply/`）sha256 与快照逐份相同（9db82fab…、8b37f0f1…）。攻方腿的冻结树 `~/.local/share/singlefs/work/m3-close-g4-rewrite-r3-opus/frozen` 对 `code-sha256.txt` 七份全 OK，腿引 `crates/` 的行按它核。
   - `.claude/kb/checks-owed.md`：开工之后被本会话批 15 的 kb 写回改过两处（`~/.local/share/singlefs/work/m3-close-w15b-kb-writeback/replace.json` 里欠账表那一处，与 `spec-k1.json` 新立 C874 那一行）。两处都反着做一遍，得到的副本 sha256 59b68208… 与快照相同（`checks-owed-snapshot-copy.md`）。两腿引的欠账表行（C461 第 732 行、C874 第 579 行）是工作区此刻的行号：C461 那一句「另报哪块盘少于两槽自证过、可写挂载会拒」在快照原样的第 731 行逐字找到；C874 那一行快照里还没有，是快照之后本会话新立的，腿引它只用来说「已经立了」，与工作区此刻相符。
2. 引文：`python3 scripts/agents/cite-check.py <两份报告> --root <冻结树> --root . --background <背景材料>` 末行「✓ 核了 72 处引文、73 处裸行号，对上 72 处，没判 73 处（其中裸行号没有引文可比 71 处；逐处列在上面）」（输出 `~/.local/share/singlefs/work/m3-close-g4-rewrite-r3-main-checks/cite-check.txt`）。判决用到的行主 agent 逐处读过：
   - `crates/singlefs-core/src/make_filesystem.rs:52`：「pub const SYSTEM_CONFIGURATION_GENERATION_AT_MKFS: u64 = 1;」。正推腿 Y2-b 第 2 条写「mkfs 写的槽世代号全是 0」，不对；它那一条的结论（世代号判别子在取号窗口里漏报）与攻方量的方向一致，数见第二节。
   - `crates/singlefs-checker-tier/src/crash_injection.rs:18`（冻结树）：「崩溃点重放不跑可写挂载，这两截只在这里有。」；`crates/singlefs-checker-tier/tests/common_crash_point_observers/mod.rs:194`：「.unwrap_or_else(|error| panic!("每个崩溃状态上可写挂载都做得成：{error:?}"));」。攻方撤回它第二轮「崩溃点重放不跑可写挂载」那一句，撤得对；崩溃注入文件头那一句同样说过了头。
   - `crates/singlefs-checker-tier/src/crash_given_states.rs` 里 `grep -c observer` 为 0：给定状态表那一路（崩溃注入、读故障两项随机历史）没有观察者。
   - `.claude/kb/decisions/23-journal的角色与格式.md:396` 的判不出要「有槽设备报读错」；`crates/singlefs-checker/invariants/03-journal.md:12`：「单盘全废不判违例，走旁路报告：另报哪块盘少于两槽自证过、可写挂载会拒。对着只读恢复判。」
   - `research/prompts/m3-inv-a-r1-main-verification.md:58`：「3. I-8.5 对着只读恢复，可写会拒的走旁路报告。」（用户定案；丙-I 要反转其中一半）。
3. 复跑：两腿的模型目录拷到 `~/.local/share/singlefs/work/m3-close-g4-rewrite-r3-main-checks/`（`sonnet-model-copy`、`opus-model-copy`），经内存包装各跑它自己的 `tools/rerun.sh`，结果见第五节。
4. 主 agent 换路子核的一个数：第二轮攻方 W1 那一句「四组合计 172 366 态」。拿第二轮输出（攻方这一轮的 `out-r2/`）逐份加「states count」：`remount` 20 519、`zeros_mount` 20 519、`cold_start_then_remount` 20 519、八个基线 6 × 20 519 + 2 × 24 626 = 172 366，四组合计 233 923。172 366 是八个基线那一组的和，标签错；正推腿报的 233 923 对。

## 二、逐格判定

| 格 | 判定 | 依据 |
|---|---|---|
| Y1 甲-二 不做 | **站得住**，三轮定 | 两腿一致：判官不读补写闸，两张盘面判官结论逐字段相同（攻方复跑第二轮那两份 `byte_same=yes`）；两个候选都只看两槽、不读闸 |
| Y1 G4-丙 原形态出局三处理由 | ① 整写子集：**站得住**，并罩到丙-I 在随机历史里的那一半；② 一件事记两条：**对丙-置 不成立、对丙-I 原样成立**；③ 共用核心层算法：**两个候选都改读池级 checker 的逐盘数就解了**，余下的口径差（池级 checker 的「自证过」不分 fsid）只落在两盘全废、今天已红的那几态上 | 两腿一致；② 的依据是给定状态表那一路没有观察者（主 agent 现查 0 处），丙-置 与 G4 拒绝那一臂不在同一趟判同一张盘面；③ 攻方 19 趟流里两种读法单盘那一格 0 处不同（量的，线索） |
| Y1 第一轮的甲 | 跟 G4 拒绝那一臂走：臂留它留 | 两腿一致（读码）：那条测试直接调第三截判法，是 `crates/mutations.tsv` 第 2408 行唯一的杀手 |
| Y2 正确实现在 mkfs 返回之后留不留得下单盘全废态 | **没打中，两次抽样都没打中，记「没打中」** | 第二轮 W1 是第一次；这一轮两腿各一次、历史不重：攻方另 8 段历史 168 947 态（撕裂 250 态）0 态，正推腿可达盘面闭包 25 张 0 张死盘、穷举 729 条小流得「单盘全废态可达 ⟺ 某段里一块盘的系统配置槽写不止一个」。取样都是两块盘、小几何；覆盖写、删除、清扫几条大流与三块盘没取，推翻条件照攻方写的 |
| Y2 分清 mkfs 中途与之后 | **盘面上分不清**（判别子观测不到），两腿一致、量的 | 正推：mkfs 返回之后丢一块盘的 8 张盘面逐槽等于 mkfs 末段里的某个状态；攻方：丙-I 照字面在 mkfs 自己 6 态、补写流 3 态误报，加世代号判别子在 mkfs 之后第一次取号漏 MUT-B1 4 态、MUT-B2 6 态，补写流误报 1 态（原型与手算两路一致） |
| Y2 丙-I | **不取** | 判别子观测不到（上一行）；改池级 checker 让判镜像、GPU 判器两组判法单元的键全换（两腿读码一致；正推腿量了闭包大小 2487、2567、WGSL 1600 个单元）；G4 拒绝那一臂必须撤（否则同一张盘面记两条）；反转用户定案（`m3-inv-a-r1-main-verification.md:58`）的一半。不取就不动那条定案 |
| Y2 丙-置 | **取**：判两槽全废盘面的判官挂成崩溃点重放的观察者，判槽自证读池级 checker 的逐盘数（`system_configuration_redundancy_report`），撕裂域在内 | 两腿一致取它。它的具体接法有两腿各自的收严（第三节），被攻过零轮 |
| Y2 丙-择 | **不做** | 两腿一致：两个候选都改读池级 checker 的逐盘数，它在两盘全废那几态上照样点名两块盘；那几态今天就红 I-8.5 与 oracle 两处 |
| Y2 G4-戊 | **不另做** | 正推：丙-置 相对基镜像判，「只判新增的无槽盘」白送；攻方：丙-置 不在崩溃注入里，这一问对它不适用；丙-I 做不了（二次崩溃那条流 6 态盘 0 仍全废，量的，线索） |
| Y2 G4 拒绝那一臂撤与留 | **留**（随丙-置） | 两腿一致：撤留随候选翻面，丙-置 下留、丙-I 下撤；前两轮辩方「撤不得」、攻方「撤掉」各对一半。留臂：E171 装置那一臂、`crates/mutations.tsv` 第 2406–2408 行照旧 |
| Y3 未定项与条文 | 条文候选只在丙-置 下写；**新打中一处**：「少于两槽自证过、可写挂载会拒」在没有读故障的盘面上不成立，与补写例外无关 | 攻方量的：正确实现 250 态加两张手造盘面，旁路报告点名「会拒」的盘全部挂上；正推读码同向（D23 已定项 14 的判不出要有槽设备报读错，主 agent 现查第 396 行）。同一句另在 `crates/singlefs-checker/src/image.rs` 两处注释、两条测试的文档注释、欠账表 C461 一行、字段名 `devices_whose_writable_mount_would_be_refused` |
| Y4 第二轮攻方的数 | **核上，两处标签要改** | 攻方在这一轮冻结树上复跑第二轮 18 条流 21 份计数全对上，另用不共用代码的手算核上 W1、MUT-B2 16 + 1、MUT-B1 8 / 34；正推腿用自己的模型推出同样的数。标签：「W1 四组合计 172 366」是八个基线一组之和，四组合计 233 923（主 agent 复算，第一节第 4 条）；MUT-B1 的整写 8 / 撕裂 34 依赖原型把坏实现开关设在整个进程（基镜像也带坏实现），基镜像干净时是 6 / 36，总数同为 42（两腿一致） |

## 三、去向

### 这一轮定的（三轮定）

- 甲-二 不做。
- 两槽全废盘面由崩溃点重放的观察者判（丙-置），判槽自证读池级 checker 的逐盘数，撕裂域在内；丙-I 不取，I-8.5 照用户定案仍对着只读恢复判。
- G4 拒绝那一臂留；第一轮的甲、E171 装置那一臂、`crates/mutations.tsv` 第 2406–2408 行照旧。
- 丙-择 不做；G4-戊 不另做。
- 第二轮攻方两处标签改正：W1 172 366 是八个基线一组之和（四组 233 923）；MUT-B1 8 / 34 是基镜像也带坏实现时的分法（基镜像干净 6 / 36）。

### 交用户（零轮，第三轮之后不再开轮）

| 项 | 内容 | 出处 |
|---|---|---|
| 丙-置 的接法 | 两腿各提一样收严：正推「相对基镜像新出现的、一份自证过的槽都没有的盘才记，全池皆废的态不重记」，攻方「每条枚举流报自己的来历（mkfs 返回之后 / mkfs 自己 / mkfs 中途崩溃之后的挂载），观察者照来历判，不按段认」。攻方量到按段认在补写流 3 态上误报；正推量到不取基镜像在同一形 3 态上误报。主 agent 的组合：观察者按流登记（记录 mkfs 的流不挂）且相对基镜像判，加一条登记完整性用例（含系统配置槽写段的节点都挂了它）。三样都只在腿的模型上量过 | 正推 Y2-c；攻方 Y2 第 3 节 |
| G4 拒绝那一臂的判槽自证 | 臂今天用核心层的 `parse_slot` 与 `choose_system_configuration` 复算，改读池级 checker 的逐盘数 | 第二轮判决「第三轮再攻」第 4 条；两腿 Y1 理由 ③ |
| 「可写挂载会拒」这一句 | I-8.5 一行、`image.rs` 两处注释、两条测试的文档注释、欠账表 C461 一行改成「带读故障时可能判不出而拒；没有读故障时挂得上，取号写两轮补回那一槽」；字段 `devices_whose_writable_mount_would_be_refused` 改名或改成只报「少一槽」 | 攻方 Y3-①；正推 Y3 |
| 崩溃注入文件头那一句 | `crash_injection.rs` 文件头「崩溃点重放不跑可写挂载」改掉 | 攻方 Y1 撤回第 1 条 |
| 条文 | 照丙-置 改：D13（验证路线） 第 187 行（这一性质由崩溃点重放 mkfs 返回之后各条流上的观察者判，随机历史里补写闸不过的那一形由崩溃注入第二截的 G4 拒绝那一臂判，补写闸过了的那一形随机历史里没人判、正确实现留不下）、删「与 C670 那条同接法」；D13 第 72 行留臂之下只改措辞；D23（journal 的角色与格式） 第 396 行带上补写例外（D22（单元原子性怎么合成） 已定项 8），「自证过」写明是核心层口径；I-8.5 不变量本身不动，旁路报告那半句照上一行改 | 正推 Y3；攻方 Y3 条文表（「由观察者判」漏了随机历史，主 agent 照它补进括注） |

### 记下、不在这一轮改

| 项 | 去向 |
|---|---|
| 丙-置 的实现（观察者、登记、登记完整性用例、变异行至少四条：不判、只判整写不判撕裂、不取基镜像、判槽读法恒为「有」） | 用户定了接法之后进实现批；不碰核心层、池级 checker、`mount_transparency.rs` 的判官本体与 WGSL |
| C874（补写那一支进不了崩溃测试、运行时看不出闸哪一件没过） | 正推腿列了还缺的五样（相对基镜像判、闸五件不判的明说、补写流造法、登记完整性与共用 `parse_slot` 两行、字段名），随丙-置 那一批补进 C874 那一行 |
| 补写中途崩溃的枚举流 | 并进 C874；那条流落地时观察者按流登记、不按段认 |
| 第二轮攻方 `tools/rerun.sh` 的比法（比老几版输出会出 8 个 `same=no`） | 冻结证据不改；这一轮攻方的 `tools/rerun.sh` 去三种行再比，复跑以它为准 |

## 四、多轮小结（三轮）

| 结论 | 第一轮 | 第二轮 | 第三轮 | 定 |
|---|---|---|---|---|
| 红的测试走甲 | 定 | 站住 | 跟臂走、臂留 | 定 |
| 补写例外放过的两槽全废盘面挂上了没人判 | 打中 | 两腿一致 | 两腿一致，在崩溃点重放里也在 | 定，由丙-置 补 |
| 甲-二 | — | 不做 | 站住 | 定 |
| G4-丙 原形态 | 提出 | 三处站不住 | 三处理由站住（② 对丙-置 不成立） | 原形态出局 |
| 判官形态 | — | 交第三轮 | 丙-置 取、丙-I 不取 | 定；接法零轮交用户 |
| 拒绝臂撤与留 | — | 两腿相反 | 随候选翻面，丙-置 下留 | 定 |

## 五、复跑

两腿的模型目录拷进 `~/.local/share/singlefs/work/m3-close-g4-rewrite-r3-main-checks/`，经 `scripts/hosts/run-with-memory-cap.sh` 各跑它自己的 `tools/rerun.sh`，`--frozen` 都给攻方留的冻结树。日志两份经 `scripts/hosts/two-logs.sh` 写：`rerun-sonnet.log`、`rerun-opus.log`。

- 正推（2G）：「name=rerun_compare file=run_all.out sha256=6633604cd9c49bb583ae47c4954f407dca2aee50e9376dc554d7cd6b36e80487 same=yes」「name=rerun_compare file=second_crash_bases.out sha256=0bf245c8d771484b46f0800858cb2770e0c1c19abee8eb0b38866ed00f3a2f0e same=yes」。`y3_greps.out` 是 `informational=yes same=no`：它读仓此刻的样子，腿报告里写明只比不判。
- 攻方（16G，`nice -n 19`）：`rerun_compare` 60 行全部 `same=yes`，其中 `how=byte` 38 行、`how=stripped` 21 行、`tag=independent_count` 1 行；`rerun_batch` 38 件 `rc=0`、非 0 的 0 件。与腿报告「38 how=byte same=yes / 21 how=stripped same=yes / 1 tag=independent_count same=yes」逐项相同。

判决引的腿的数（Y2 第 1 节 168 947 态 0 张单盘全废、Y2 第 2 节丙-I 的误报与漏报、Y3 的 250 态全挂上、Y4 的第二轮各数）都出在这两份复跑核上的输出里。它们是副本原型上的数，不是入库装置上的数，判决只拿来判臂，不写进 kb 当量值。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 1 | 支撑 | 2026-10-22 不受影响：挂载透明的射程照旧；第 72 行措辞随交用户表改 |
| D13（验证路线） 已定项 6 | 支撑 | 2026-10-22 改了：第 187 行由 G4 判改成由崩溃点重放的观察者判（丙-置），条文稿交用户 |
| D22（单元原子性怎么合成） 已定项 8 | 支撑 | 2026-10-22 不受影响：补写例外与它的闸照用户定案，这一轮只判由谁判两槽全废 |
| D23（journal 的角色与格式） 已定项 14 | 支撑 | 2026-10-22 改了：第 396 行拒因单子带上补写例外、写明「自证过」的口径，条文稿交用户 |

## 用户定案（2026-10-22，弹窗）

| 项 | 用户选的 |
|---|---|
| 丙-置 的接法 | 「按流登记+相对基镜像」：观察者只挂 mkfs 返回之后的流（记录 mkfs 的流不挂），只记相对基镜像新出现的死盘，另加一条登记完整性用例 |
| 「可写挂载会拒」这一句 | 「改措辞并改字段名」：改成「带读故障时可能判不出而拒；没有读故障时挂得上，取号写两轮补回那一槽」，`devices_whose_writable_mount_would_be_refused` 改名，代码与 kb 一批改 |
| G4 拒绝那一臂的判槽自证 | 「改读池级 checker」：随丙-置 那一批实现一起改，变异行改指新读法 |
