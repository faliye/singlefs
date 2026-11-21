# after-split-governance-r1：判决（2026-11-20）

正文 `research/prompts/_after-split-governance-r1-body.md`，开工快照 `research/prompts/after-split-governance-r1-snapshot/`。这是这一题的第一轮。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 正推（sonnet） | `research/prompts/after-split-governance-r1-sonnet-output.md` | 格 1 主体成立，另打中两处（只判八格的名单「只缩不长」没有会红的检查；构建脚本运行时读别的包不在判法射程、判法文件头说法像判得了）；格 4 成立，三处边界交主 agent 定；格 5 不成立，规则与判法对不上五处 |
| 攻方（opus） | `research/prompts/after-split-governance-r1-opus-output.md` | 第 2 格放置类词表不成立、前像成立、相抵部分成立、dep-info 后缀认法不成立；第 3 格单元键收相对路径按字面不成立（与 D13（验证路线） 已定项 17 相抵）、HOME 改按配置进键之后不成立（最脆弱：生效构建环境大多没进键）、自证不够当换台证据；「已经量的」22 句里 7 句量在被判文件更早的一版、1 句核不到 |

核查（照 `.claude/rules/three-way-inference.md`「核查与判决由主 agent 做，不派子 agent 去核子 agent」）：

- 倒推开工快照的树：`task-server/task submit --cwd ~/.local/share/singlefs/work/after-split-governance-r1-verdict -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py after-split-governance-r1 --out ~/.local/share/singlefs/work/after-split-governance-r1-verdict/after-split-governance-r1-tree --repository /home/fy5090/code/singlefs`，type=command-failed exit=1：34 份里 30 份对上；`.claude/hooks/bash-command-detector.sh`、`.claude/hooks/write-guard.sh`、`.claude/hooks/write-guard-decisions.py`、`.claude/hooks/write-guard-selftest.py` 四份倒推不出（快照之后 mainagent(A)（梳理子 agent 定义）把这几份钩子改成部分文件、入口 import，`write-guard-decisions.py` 改了名，HEAD 也有新提交改过前两份）。腿引到这四份的行，主 agent 在主工作区现版逐处人工核，判决只用核得上的那几处。
- 引文：`python3 scripts/agents/cite-check.py research/prompts/after-split-governance-r1-sonnet-output.md research/prompts/after-split-governance-r1-opus-output.md --root ~/.local/share/singlefs/work/after-split-governance-r1-verdict/after-split-governance-r1-tree --background research/prompts/_after-split-governance-r1-background.md`，「✓ 核了 84 处引文、60 处裸行号，对上 78 处，没判 66 处（其中裸行号没有引文可比 46 处；逐处列在上面）」。没判的 20 处引文都是快照之外的文件（拆出的库、`crates/` 下的构建工具、规则），判决用到的几处在主工作区现核：D13（验证路线） 已定项 17 原文在 `.claude/kb/decisions/13-验证路线.md:436`「一律不进键……路径（仓根也算）、名字、种子串、行号、资源上限、限时……只有一个例外：观察者结论的键里用判定项标识」，第 435 行「加工具链（含 cargo 解析后的生效构建环境与外部工具版本）」；`.claude/rules/verification.md:41`「路径、名字、行号、资源上限、限时、提交与树不进键」。
- 两条腿的模型都拷到 `~/.local/share/singlefs/work/after-split-governance-r1-verdict/` 下的副本里复跑（拷完 `diff -rq` 与原目录相同）：
  - 正推：`task-server/task submit -- bash <副本>/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs <副本>/frozen-stub <运行目录> <副本>` type=succeeded exit=0，「MODEL_RESULT identical」；c12「build-script-path-attribute provider-listed=yes」，c14「build-script-runtime-read provider-listed=no rerun-if-changed-in-build-output=yes」，c15「compile-failure cargo-exit-nonzero=yes」。模板自证 type=succeeded exit=0。放置类钩子五条喂法退出码依次 0、2、0、2、0，与腿写的期望相同。rules-lint type=succeeded exit=0。
  - 攻方：`task-server/task submit --cwd <核查目录> -- bash <副本>/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs <运行目录> <副本>` type=succeeded exit=0。读数：`name=judged_files_check lines=4`；相抵六行：同一行只改路径、号挪到另一句、会话描述改指另一个会话、描述挪行改写四样 `new_hits=0`，删旧号加不同号、同一号重复两样 `new_hits=1`；放置类十二问：误拦四问（时间义「放在」带「包」、调度记录标题没加「」的「归位」、结果文件放进草稿目录、日志目录在哪）exit=2、漏拦六问（放到包、新开一个包、单开包还是并进、哪一个包、搬进 crate 路径、哪个子目录）exit=0，两问对照判对；五种环境 `compile_*` 都 `cargo_exit=0 problems=0`；键二十行：plain 四包都 sound-reuse，RUSTFLAGS 环境、上层目录 cargo 配置、家目录 cargo 配置、关 debug-assertions 四种环境下 alpha 都 false-reuse、gamma 在前两种 false-reuse；`classify_delta`：`crates/beta/src/code.md`、`shared/snippet.inc`、`OUT_DIR:delta/copied.rs` 都 red=no，`crates/beta/data/sample.json` red=yes。
- 「已经量的」那一节在被判那一版上重量（攻方第四节点名量在更早一版的几句）：checker-independence-and-sync 样本档 type=succeeded exit=0「点名的 1 道阶段判得都对（45 个用例）」；它的样本开关 type=succeeded exit=0「按条复用：22 条开关（1 道阶段），现判 22、复用 0、拒算 0」；真仓 crate-roles 格 type=command-failed exit=1，`✗` 只有 support-dependents 那一行；写闸开关复跑 type=command-failed exit=1「8 条里拒算 8 条」，拒算点 `.claude/hooks/write_guard_decisions.py:230`，是 mainagent(A) 正在改造的那几份钩子，等它改完重交。正文里「旧判法同样红」那一句没有一趟用旧判法跑过真仓，出自工具员报告里标「推的」那一段，撤回。
- 换台复用：真仓 crate-cross-references 格又交两趟，都落在本机：前一趟「单元键：现判 13、复用 0」，13 包都报「键变了：judge」（三趟第二台之后，mainagent(B)（算键闭包）换上了算键库修复版、mainagent(A) 在改钩子，判法闭包变了，重判是对的）；后一趟「现判 0、复用 12、强制现判 1、拒算 0」。一台写库、另一台读库的复用这一轮没量到，任务服务不钉台。

## 二、判决

1. **第 1 格（拦不拦得住）**：第 1 组放置类钩子对两个点名输入判对；第 2 组前像比对拦得住「别的会话改了同名行」；第 3 组旧号只改路径放行、同行新加一个号拒；第 4 组编不过的写法判红、九种写法里 c01 到 c13 读数都对。成立。另有两处坐实打中：① 只判八格的名单 `.claude/gate.d/code-round-checklist-eight-cells.tsv` 写「只缩不长」，判法只核每行指的正文在（`.claude/gate.d/lib/doc-process-records-rounds-and-ledger-cells.sh` 的名单那一段），没有拿基准比，往名单加一行新正文不红；② 构建脚本运行时读别的包的文件不在判法射程（c14 读数坐实），判法文件头那一句读起来像判得了。
2. **第 2 格（误伤与绕过）**：放置类词表不成立（模型十二问里误拦四问、漏拦六问；主 agent 自己这一轮的弹窗「每个文件的归属包」前一版写成「落在哪个包」也被拦，是同一类）。前像比对成立：没有别的生成六段替换的生成方；攻方点名的 `scripts/gate/lib_crates_split_tables.py:226` 整表 `--out` 换进变异表不过前像，是 crates 代码拆分那一侧的工具，交 mainagent(C)（crates 代码拆分）认领。相抵部分成立：同一字面的旧号挪到另一句、会话描述改指另一个会话两样不拒（模型读数 `new_hits=0`）。dep-info 后缀认法不成立：`.md` 后缀当模块编进来、不属任何包的 `.inc`、构建脚本把别的包源码拷进 OUT_DIR 再 `include!` 三样放过（`classify_delta` 读数），判法文件头、放行表表头与规则对数据文件的说法互相对不上，放行表第 7 行的理由与代码不符。
3. **第 3 格（复用键与多机）**：单元键收相对路径与 D13（验证路线） 已定项 17 字面相抵（`.claude/kb/decisions/13-验证路线.md:436`），主 agent 开轮时判它是「判定自己的内容输入」不成立；去掉路径又会让文件换了归属包而键不变。交用户定，用户已定「D13 开第二个例外」（见「交用户的」）。HOME 改按 cargo 配置内容进键之后，键里没再找到随编译目录变的成分，但生效构建环境大多没进键（RUSTFLAGS 一类环境变量、上层目录的 cargo 配置、`CARGO_PROFILE_*`、关 debug-assertions），模型四种环境下都有 false-reuse。这一点不算违反已定项：工作区 D13（验证路线） 同一节已有用户定案「所有判定的键都不带工具链、外部工具版本与依赖版本」（`.claude/kb/decisions/13-验证路线.md:483`），换构建环境错复用是这条定案的代价；反过来判法键里现收的 `toolchain` 与按 cargo 配置内容进键的那一样违反这条定案，要拿掉。本仓今天挂在 `#[path]` 前的 cfg 只有 `feature = "gpu"`，这几样环境翻不动它。自证 key-independent-of-compile-directory 那一格是假 cargo 在两个根下写同一份录好的 dep-info，两份事实相同是喂法造出来的，不够当换台证据。
4. **第 4 格（不兼容旧格式）**：成立。正推点名的三处边界：`scripts/gate/apply_writer_patch_plan.py` 弄坏开关里读六段、读只写名字的分支，与 `.claude/gate.d/lib-forbidden-notations.py` 里留作弄坏开关的旧相抵算法，都只在自证与开关下走得到，是弄坏开关证红要的「改坏的那一处」，平常路径不认旧格式，不算兼容分支；crate-cross-references 滞后表通路此刻 0 行，留着。
5. **第 5 格（规则与定义正文）**：不成立。正推点名的五处坐实：①「偏离：」要用「」照抄那一节原话至少 4 个字（钩子 `QUOTE_MINIMUM`），`.claude/rules/directory-layout.md` 没写；② `crate-roles-check.py` 把「库文件不在」也算空库，规则没写；③ `.claude/rules/verification-reference.md` 写「标题日期更早的只判前八格」，实际早于某个日期的一格不查；④ `.claude/agent-common.md` ⑭ 与 `.claude/agents/tooling-writer.md` 第 3 步的样本补丁出路没写「照仓里的相对路径」，`git apply -p2` 会指错位置；⑤ 规则「测试支撑包至少两个依赖者」与判法数依赖者的口径不同（判法把 `crates/` 之外的 cargo 工程也算进去）。另 `.claude/agents/implementation-writer.md` 补丁目录那一段把前像规矩插在文件列表中间、列表读断；`.claude/gate.d/break-switch-replay-files.tsv` 这一批新加的行末注释带日期与「此前……那一版」，违反「代码与脚本的注释只写现状」。
6. **「已经量的」**：7 句量在更早一版，主 agent 在被判那一版上重量了能量的四条（见核查），样本档与样本开关、crate-roles 真仓格结局不变；写闸开关复跑拒算，等 mainagent(A) 改完重交；「旧判法同样红」撤回。门禁 code-tooling 的 crate-roles 样本档缺「产品包依赖测试支撑包」那一格的样本（正推点名），坐实。

## 三、改法

都交下一件工具员，改完开第二轮（第二轮派辩方腿复核这份判决，照 `.claude/rules/implementation-workflow.md`「代码轮与治理文件轮怎么收」）。都是零轮，不碰产品代码，不做 A/B。

1. 放置类钩子与 `.claude/rules/directory-layout.md`「放置类问题怎么认」一起改：误拦的四问与本轮主 agent 那一问放行、漏拦的六问拦下；词表与规则仍由自证逐组比对；模型十二问进喂法。
2. 只判八格的名单接只缩不涨的基准比对（照 crate-roles 滞后表用的 `.claude/gate.d/lib-lag-baseline.py`），配样本与开关。
3. 相抵：同一字面的旧号离开原来那一行（挪到另一句、另一行）照拒；会话描述改指另一个会话的，写闸判不出指的是谁，写进判法文件头「判不到的」，不另立判法。
4. dep-info 判法：`.md` 等数据后缀的文件当模块编进来（出现在编译单元的源码里而不是 `include_str!` 一类读进来）照源码判；不属任何包的文件不论后缀都判红（除非登放行表）；构建脚本报的 `rerun-if-changed` 落在别的包目录、与拷进 OUT_DIR 再编进来的照判；判法文件头、放行表表头与规则三处数据文件的说法统一，放行表第 7 行理由照代码重写。
5. 单元键（更正：主 agent 写这一条时引的是 HEAD 那一版 D13（验证路线） 已定项 17 第 435 行；工作区里同一节已有用户定案「所有判定的键都不带工具链、外部工具版本与依赖版本」（`.claude/kb/decisions/13-验证路线.md:483`，原话「全部都不留  工具链和依赖版本」，`records/2026-11-19-判定键只算产品与测试代码.md:156`），没有例外。所以改成：工具链与生效构建环境一样都不进键，判法键里现收的 `toolchain` 与按 cargo 家目录 config 内容进键的那一样一起拿掉；攻方量到的换构建环境错复用是这条定案的代价，不另立判法）；换台复用补一格「一台写库、另一台读库」（两个临时根、两份事实各自由假 cargo 现录、库共用），并在真仓上量到一次再算数。路径进键照用户定案等 D13 写回之后保留，写回之前不改。
6. 规则与定义正文五处与另两处照第 5 条改；`.claude/gate.d/break-switch-replay-files.tsv` 这一批行末注释去掉日期与「此前」，连同用户还没答的两行开关条数说明一起问用户。
7. crate-roles 样本档补「产品包依赖测试支撑包」红样本。

## 交用户的

- D13（验证路线） 已定项 17 与 crate-cross-references 格的单元键收路径相抵：用户定「D13 开第二个例外」——判文件归属包的这一格，键里收包目录与相对仓根的路径，只用来判归属。改已定决策照 `.claude/rules/three-way-inference.md` 先开一轮设计推论三方、再由写回员写回 kb 与变更史；写回之前判法照现在的样子跑。
- `.claude/gate.d/break-switch-replay-files.tsv` 两行开关条数说明（replace-once 6 改 11、replace-batch 11 改 16）与判法登进开关复跑表那一行：工具员己的 Edit 被权限判定拒过，主 agent 不代做，等用户答。

## 这一轮被判过的文件

- .claude/hooks/ask-user-placement-guard.sh：判了新词表拦不拦得住点名的两问与误拦、漏拦（第 1、2 格），坐实误拦四问漏拦六问
- .claude/hooks/bash-command-detector.sh：判了 ⑭ 往已有门禁样本档目录写的那一半（第 1 格），快照后被别的会话改造、按现版核
- .claude/hooks/write-guard.sh：判了第十一道样本档那一半与任务号相抵的点名（第 1、2 格），快照后被别的会话改造、按现版核
- .claude/hooks/write-guard-decisions.py：判了文档串与相抵说法（第 5 格），快照后被别的会话改名为 write_guard_decisions.py、按现版核
- .claude/hooks/write-guard-selftest.py：判了「改了旧号那一行、号没动放行」与「新加第二个号照判」两格（第 1 格），快照后被别的会话挪动、按现版核
- .claude/hooks/lib_write_scope.py：判了已有门禁样本档目录的认法（第 1 格）
- scripts/gate/crate-roles-check.py：判了按目标逐个登记与产品包依赖测试支撑包一格（第 1、5 格），坐实「库文件不在也算空」与依赖者口径和规则对不上
- .claude/gate.d/crate-roles.tsv：判了拆后现算的角色行（第 1 格）
- .claude/gate.d/crate-roles-lag.tsv：判了只缩不涨与逐目标行（第 1 格）
- .claude/gate.d/crate-roles-allowed.tsv：判了放行的门禁工具与 example（第 2 格）
- .claude/gate.d/doc-process-records.sh：判了 code-round-checklist 截止日与名单（第 1 格），坐实名单只缩不长没有会红的检查
- .claude/gate.d/code-tooling.sh：判了 crate-roles 格说明（第 5 格），坐实样本档缺产品包依赖测试支撑包那一份
- .claude/gate.d/code-round-checklist-eight-cells.tsv：判了名单十行与它的增长（第 1 格）
- scripts/gate/apply-writer-patch.py：判了前像比对与段数拒（第 1、4 格）
- scripts/gate/apply_writer_patch_plan.py：判了前像逐行比对与弄坏开关里的旧形态分支（第 1、4 格）
- scripts/gate/merge-and-verify.sh：判了合补丁那一支写前像（第 2 格）
- scripts/gate/merge-and-verify-selftest-lib.sh：判了合补丁前像那一格自证（第 2 格）
- scripts/gate/patch-of-the-current-tree.py：判了写替换行带前像（第 2 格）
- .claude/gate.d/lib-forbidden-notations.py：判了按违例本身相抵（第 1、2、4 格），坐实挪行与改指会话两样不拒
- scripts/kb/lib_notation_check.py：判了五格共用自证（第 1 格）
- scripts/kb/replace-once.py：判了它调的相抵与开关点名（第 1 格）
- scripts/kb/replace-batch.py：判了它调的相抵与开关点名（第 1 格）
- scripts/kb/insert-row.py：判了它调的相抵（第 1 格）
- scripts/gate/crate-cross-references-check.py：判了 dep-info 分类、构建脚本单元、单元键与换台复用（第 1、2、3 格），坐实三样后缀漏判、构建脚本运行时读的不判、生效构建环境没进键、路径进键与 D13 相抵
- .claude/gate.d/checker-independence-and-sync.sh：判了 crate-cross-references 格接入与样本开关点名（第 1 格）
- .claude/gate.d/crate-cross-references-allowed.tsv：判了放行 5 行的理由（第 2 格），坐实第 7 行理由与代码不符
- .claude/gate.d/crate-cross-references-lag.tsv：判了 0 行的滞后表通路（第 4 格）
- scripts/prune-server/prune_server_client.py：判了「键空间」补的 crate-cross-references 那一句（第 5 格）
- .claude/gate.d/break-switch-replay-files.tsv：判了这一批改的几行（第 5 格），坐实行末注释带日期与「此前」
- .claude/agent-common.md：判了 ⑭ 与写闸两处相抵说法（第 5 格），坐实样本补丁出路缺相对路径
- .claude/agents/tooling-writer.md：判了第 3 步样本补丁那一句（第 5 格），坐实缺相对路径
- .claude/agents/implementation-writer.md：判了补丁目录那一段前像说法（第 5 格），坐实插在文件列表中间读断
- .claude/rules/verification-reference.md：判了门禁管哪一半里 crate-roles、code-round-checklist、crate-cross-references 几行（第 5 格），坐实「更早的只判前八格」与判法不符
- .claude/rules/directory-layout.md：判了放置类问题认法、后缀说法与门禁管哪一半（第 5 格），坐实「偏离：」要引原话那一条没写

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 17 | 备料 | 2026-11-20 不受影响：这一轮只判治理文件，判出 crate-cross-references 格的键收路径与已定项字面相抵、生效构建环境没进键；用户定开第二个例外，另开设计推论轮写回，这一轮不改决策正文 |
