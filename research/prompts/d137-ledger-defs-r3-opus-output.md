# d137-ledger-defs-r3 攻方腿报告（格 A：只攻第二轮修补之后才有的东西，2026-11-02）

立场：假设第二轮修补之后的形态是错的，造可达的历史与输入去打穿它。只攻第二轮修补之后才有的东西；第一轮 A1–A13、第二轮 B1–B11 不原样再提（与它们相近的几条在各节写明差在哪）。
被判文件 19 份此刻的 sha256 与快照 `research/prompts/d137-ledger-defs-r3-snapshot/files-sha256.txt` 逐份相同（`sha256sum -c` 没有一行不是 OK，量的）。
快照之外引到的文件此刻 sha256 前 8 位：`task-server/task` dfed1f94、`task-server/task_server_lib.py` 3349c5be（开工时 4d3f51a3，写报告期间别的会话改了它，下面引的三行内容没变、行号下移，照此刻的写）、`crates/singlefs-checker-tier/src/gpu_card_configuration.rs` ff98c3ce、`.claude/rules/path-moves.md` cd5fd4c6、`.claude/rules/timer-only-remind.md` 14534cc4、`.claude/singlefs-ai-sop/rules/session-wrapup.md` a4d80b9d。
被判文件此刻 sha256 前 8 位（与快照同）：`.claude/gate.d/doc-experiments.sh` 7e0f7a6b、`scripts/hosts/compile-then-swap.py` 6328769d、`scripts/gate/merge-and-verify.sh` cc7ae95d、`.claude/agents/experiment-runner.md` 83d9a62d、`.claude/agents/experiment-designer.md` 2a2fa725、`.claude/agents/tooling-writer.md` f0028544、`.claude/agents/kb-scribe.md` a7f6c098、`.claude/agent-common.md` 54ff4c79。

## 模型目录、复跑命令与每个文件的 sha256

模型目录 `research/prompts/d137-ledger-defs-r3-opus-model/`。`tools/rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh`（5e273544）拷，只改了 `rerun_steps` 那一段（只造世界根与变异体、只拷文件、只起 git 与 bash）；单跑门禁两格与探针另交（下面「要主 agent 跑的命令」第 2–5 条）。
一行都没跑过：下表与各节的「门禁会判成什么」全是照代码逐行推的，判决要引的数等主 agent 复跑。

复跑（详见「要主 agent 跑的命令」）：
```
task-server/task submit -- bash research/prompts/d137-ledger-defs-r3-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs --work /home/fy5090/.local/share/singlefs/work/d137-ledger-defs-r3-main/rerun-1
task-server/task submit -- bash research/prompts/d137-ledger-defs-r3-opus-model/tools/judge-worlds.sh /home/fy5090/.local/share/singlefs/work/d137-ledger-defs-r3-main/rerun-1/worlds
```

```
b3c39c15246bad4acd075f5b99469b84b9e2be65528360c5cf67bd14a95c25ec  out/expected.txt
275c8f4cd200d3eea45600f32feaac4c695937d94ebf1f1e07297bb38c30b444  tools/judge-worlds.sh
4730742a27dcc596e83de327bca51b398e4cdcf709a16ec15c524108e10a00d1  tools/make-mutants.sh
4cbc75bd876c8f580244c8ab14d5db70759c43fbbb46303536ba08e6ef24ee6a  tools/make-worlds.sh
011bfa9af44eddf863ccd2a49ed876b0e27aa2afa5d32078601cc75574f91744  tools/release-features-probe.py
6452a42bda9e41020e286ead40e70c605b3bf7c77cff0c6f952ba9998d97ab68  tools/rerun.sh
75d8d417f3344f291f08d50baa4dc23cc45892432653fd06a4c6eb203cc56aa4  worlds/chain-array-parameter-unjudged/after/research/prompts/e990-preregistration.md
aa3063f567ab1d4f5041e662f389daabf98c4bb1c563c4a1cc1575af8ffbbec8  worlds/chain-array-parameter-unjudged/before/crates/singlefs-checker-tier/src/bin/e990_judge_sample.rs
b707199eae2363a154133c464d773174143c583239df88384cb28c842c298c64  worlds/chain-array-parameter-unjudged/before/crates/singlefs-checker-tier/src/crash_state_enumeration.rs
151feab1eada80a834300ebf6dc0cc0cfcc5092fe8d6fc8f832e3c63c1ce7e05  worlds/prereg-fence-trailing-comment-control/after/research/prompts/e990-preregistration.md
49025886cfef5579b8ab996f3e7c7b5f2c9eb89342bac6705417f24648ebd430  worlds/prereg-fence-trailing-comment-control/before/crates/singlefs-checker-tier/src/bin/e990_judge_sample.rs
5368b980ef66205bb22a472d3685787fa1e72b4e2c96730ea48706397622a9b2  worlds/prereg-list-item-trailing-comment/after/research/prompts/e990-preregistration.md
49025886cfef5579b8ab996f3e7c7b5f2c9eb89342bac6705417f24648ebd430  worlds/prereg-list-item-trailing-comment/before/crates/singlefs-checker-tier/src/bin/e990_judge_sample.rs
8629649c9f899cbc1e5a4936988b6203a0200a11d46094ad4fc76fef849cbd75  worlds/prereg-table-cell-trailing-comment/after/research/prompts/e990-preregistration.md
49025886cfef5579b8ab996f3e7c7b5f2c9eb89342bac6705417f24648ebd430  worlds/prereg-table-cell-trailing-comment/before/crates/singlefs-checker-tier/src/bin/e990_judge_sample.rs
a553a644682e22c919fb1c125b5bf4e23b964b1021f9086a4d29ae9fac58e5da  worlds-shared/judge-sources/crates/singlefs-checker-tier/Cargo.toml
85081d55b152e6d3999da310da80a83625d130225017e717f349884b1ef73092  worlds-shared/judge-sources/crates/singlefs-checker-tier/src/crash_amplification.rs
a0f549efa2472fc09521af17d0e62c30ccc07c781ff7cbb963513713d80fefe2  worlds-shared/judge-sources/research/prompts/README.md
b707199eae2363a154133c464d773174143c583239df88384cb28c842c298c64  worlds/sync-array-parameter-new-entry/before/crates/singlefs-checker-tier/src/crash_state_enumeration.rs
6f8f190bb7ed2cc22158f747c70e6bb3632faca895ebb3a0be1b3881d3c6e12a  worlds/sync-array-parameter-on-the-path/before/crates/singlefs-checker-tier/src/crash_state_enumeration.rs
f29ecd6aa51aec7ae750c45ef1202802b32455d437f28a6f9ab4cf8c9e22fa1b  worlds/sync-cycle-new-entry-after/before/crates/singlefs-checker-tier/src/crash_state_enumeration.rs
4b7d24d786161b7f9115ab7b68a36f2915fb5d4fae867852642c81b717899829  worlds/sync-cycle-new-entry-before-control/before/crates/singlefs-checker-tier/src/crash_state_enumeration.rs
bb52baa7205a3b5975978cbb30cc8db4248706b0b4c8d3a49bad77d57638ce34  worlds/sync-slice-parameter-new-entry-control/before/crates/singlefs-checker-tier/src/crash_state_enumeration.rs
c677985e0e74e9b7b42fd77d99a32054c904f3d94ddb94855e2044928b9a2b05  worlds/sync-slice-parameter-on-the-path-control/before/crates/singlefs-checker-tier/src/crash_state_enumeration.rs
```

## 各格判定一览

| 号 | 打在哪（路径:行，sha256 前 8 位） | 照做会怎样 | 轻重 | 量的 / 推的 |
|---|---|---|---|---|
| C1 | 新格 judge-entries-in-sync 切函数单元：`.claude/gate.d/doc-experiments.sh:3367`（7e0f7a6b）表头从上一个分号或右花括号之后算起 | 签名里带数组类型（`[u8; DIGEST_BYTES]`，两份判段源文件里这样的签名 40 处以上）的函数整个不进单元：新加的公开判段入口签名里带数组就漏报（格绿、跑前登记那一格认不出、装置照登记不带 gpu、判段只打 NOT_RUN）；判段路径上的私有函数带数组就把七个入口判成「认不出」，照这一格的出路从表里删掉，之后跑前登记那一格不再要它们带 gpu | 中 | 推的；世界 sync-array-parameter-*、chain-array-parameter-unjudged 待跑 |
| C2 | 同一格认法 ④ 的走图：`.claude/gate.d/doc-experiments.sh:3432`、`:3437`（7e0f7a6b） | 环上被截断时算出的「走不到」照样记忆化：新公开入口与表里的入口互相调、写在它后面时漏报；写在前面就报。结局看函数在文件里的先后 | 低 | 推的；世界 sync-cycle-* 待跑 |
| C3 | 跑前登记那一格的新注释认法：`.claude/gate.d/doc-experiments.sh:3186`（7e0f7a6b）只在围栏里截命令行尾的注释 | 同一格把围栏外行首是命令的行、表格整格的命令当命令位置，续行也照 shell 注释认（`:3067`），行尾注释却不截：「# 以后加 --features gpu」算带了 gpu、判绿，bash 跑的命令没带 | 中低 | 推的；世界 prereg-*-trailing-comment 待跑 |
| C4 | `compile-then-swap.py` 新自证：`scripts/hosts/compile-then-swap.py:46`、`:897`（6328769d）声称命令行上的参数怎么交到 cargo 手里一并在判；23 项没有一项给 `--release` | 执行员定义写死的用法是 `--release` 加特性（`.claude/agents/experiment-runner.md:30`），在这个组合上丢特性或丢 `--release` 的变异体自证照样全过 | 中 | 推的；变异体与探针待跑 |
| C5 | 执行员第 4c 步新句：`.claude/agents/experiment-runner.md:40`（83d9a62d）「任务日志用 `task log <号> --all` 数」 | `--all` 只是最后一次尝试的整份（`task-server/task:25`）；同一任务几次尝试（服务撞上限放大重排、卡住重起、第 1c 步要求的续跑脚本）时，前几次打的 NOT_RUN 数不到，合计 0、当判段跑了 | 中低 | 推的（客户端用法行与定义逐字比） |
| C6 | 统一任务服务按同名源文件头认要卡与四份声明：`task-server/task_server_lib.py:247`、`:1970`、`:1984`（3349c5be）；定义里 0 处写这一行 | 只有四份装置写了声明，定义与门禁都不要求新的走判段装置写：下一个入库装置照设计员、执行员定义带 gpu 编、交裸二进制，服务不认要卡，二进制退 78（运行条件不满足），一段产物白跑、要再走一次 compile-then-swap；E171 两份此刻就没写。服务文件头说也认 `scripts/experiment-apparatus/` 下的同名源，那棵树没有 `src/bin/`、bin 名照定义写连字符，一份也认不到 | 中低 | 推的（grep 与读代码） |
| C7 | 三份规则按小节点名：`.claude/agents/tooling-writer.md:16`（f0028544）点名 `.claude/rules/path-moves.md`「怎么做」 | 那一节第 2 步是 `git mv`、搬前搬后起仓里的脚本、等进程退出、给在跑的子 agent 发消息；共用约束不许子 agent 做 git 写操作，工具员手里没有发消息的工具，共用约束又写「与定义冲突时以定义为准」：工具员照点名的小节 `git mv`，改名直接进暂存区 | 低到中 | 推的（逐字比） |
| C8 | 同上：`.claude/agents/kb-scribe.md:16`（a7f6c098）点名「改一个全仓术语：正文之外还有五处会红」 | 那一节要求的跟改（文件名照「怎么做」搬、`.claude/term-rename-exempt` 登记、kb 之外手工全仓搜）在写回员写范围之外，与它自己的「规格里没写的句子一个字不加」相冲；点名之后照做不了，只能停下 | 低 | 推的（逐字比） |

没打中的形状（merge-and-verify 读不出与不是成员两支、按名字认的入口表与源码的其余差、四份声明逐份核模式）见末尾「没打中的形状」。

## 一、新格 judge-entries-in-sync 的认法（C1、C2）

### C1 签名里带分号的函数整个不进单元

代码：`.claude/gate.d/doc-experiments.sh:3367`：「header_from = max(position, code.rfind(";", position, brace) + 1, code.rfind("}", position, brace) + 1)」，接着 `.claude/gate.d/doc-experiments.sh:3370`：「function = re.search(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)", header)」。
函数签名里有数组类型时（`fn f(digest: [u8; 32]) -> bool {`），`rfind(";")` 落在方括号里那个分号上，表头只剩「 32]) -> bool 」，认不出 `fn`，也不是 impl、mod，`position = closing + 1` 把整个函数体跳过：这个函数不是单元、不是调用边的终点。格上方「管不到的」只列了经别的模块选去路、宏、函数指针三种（`.claude/gate.d/doc-experiments.sh:3265`：「这几种 ③ 连不上边，漏判。认出的多了（.<名> 连到同名方法）只会多报、不漏。」），没有这一种，而且这一种同时会漏报与多报。

两份判段源文件里签名带数组类型的函数，awk 照签名拼行再配 `\[[^]]*;[^]]*\]` 数到 crash_amplification.rs 49 处、crash_state_enumeration.rs 2 处（量的，含测试模块里的；例 `crates/singlefs-checker-tier/src/crash_amplification.rs:5006` 的 `compare_judge_and_verifier`、`:5136` 的 `record_the_full_run_of_the_flow`，签名里都是 `&[u8; DIGEST_BYTES]`）。这份仓里摘要一律是 `[u8; DIGEST_BYTES]`，新入口带摘要参数是常态，不是凑出来的写法。

两种打法（世界都在模型目录，门禁照它推的退出码在 `out/expected.txt` 第二节）：
- 漏报（sync-array-parameter-new-entry）：门禁样本 judge-entries-in-sync-green 那两份源文件，crash_state_enumeration.rs 末尾加 `pub fn enumerate_crash_states_keyed_by_the_plan(plan_digest: [u8; 32]) -> bool { … enumerate_crash_states() }`。照认法 ④ 它是公开、走得到记号的新入口，应当红；格判绿，成功句与 green 样本一字不差（20 个函数单元、公开 17、带记号 4）。对照（sync-slice-parameter-new-entry-control）只把参数换成 `&[u8]`，判红点名它。
- 一路到装置（chain-array-parameter-unjudged）：同一棵树再放一个装置 bin 调这个新入口、一份跑前登记起它而没带 gpu。judge-entries-in-sync 绿；preregistration-judge-features 照 `JUDGE_ENTRIES` 整词认，`enumerate_crash_states_keyed_by_the_plan` 里的 `enumerate_crash_states` 后面接着 `_`，整词配不上，算「不走判段」、判绿。执行员照登记编、照第 4c 步数 NOT_RUN 才发现，一段产物白跑。
- 多报再顺着出路漏判（sync-array-parameter-on-the-path）：路径上的私有函数 `enumerate_crash_states_through_the_library` 签名里带数组，它一断，`enumerate_crash_states` 一族七个入口全报「表里有、源码里认不出」。这一格的出路（`.claude/gate.d/doc-experiments.sh:3458`：「删了或不再走判段的从表里删」）照字面就是把七个名字删出表；删完两格都绿，跑前登记那一格从此不要调这七个入口的装置带 gpu。对照（参数写成 `&[u8]`）判绿。

四句：
- 分不分辨臂：分辨「第二轮修补之后」与「修补之前」。修补前没有这一格，表靠人维护；修补后这一格宣称「源码里新加的公开入口」都会报，签名带数组的那一类正好报不出，另一类误报还配着「从表里删」的出路。我提的改法（下面）在三个世界上都判对（推的）。
- 被判的系统当时看不看得到判别它的东西：看得到，源文件全文就在格手里，分号在方括号里还是在语句末尾，配一下方括号与圆括号的深度就分得出。
- 满足的是判据字面的哪一个分句：判据「④ 认出的名字集合与 JUDGE_ENTRIES 逐个相同。源码里认出而表里没有的（新加的入口）判红」——认法 ① 漏切单元，使「新加的入口」这一分句在签名带数组时永不成立；另一半「表里有而源码里认不出的（改了名、删了、不再公开、走不到记号）判红」被误触发。
- 跑前条款给的改法在打中的格上还中不中：第二轮判决给的改法是「入口表与源码同步那一格」本身（`research/prompts/d137-ledger-defs-r2-main-verification.md:34`：「入口表与源码同步那一格（另立，同一件里加）」），它就是被打中的对象，对这几格不起作用。

我提的改法（只在我的模型上推过、被攻过零轮）：找表头起点时只认方括号、圆括号、尖括号深度为 0 的分号（或直接在 `[position, brace)` 里找最后一个深度 0 的 `fn <名>`）；加一份红样本「新公开入口签名里带 `[u8; 32]`」与一份「路径上的私有函数带数组、判绿」，各配一个弄坏开关。推翻条件：主 agent 跑 judge-worlds.sh，sync-array-parameter-new-entry 退 1、或 sync-array-parameter-on-the-path 退 0，C1 不成立。

### C2 环上截断的「走不到」被记忆化

代码：`.claude/gate.d/doc-experiments.sh:3429`：「认法 ④：顺调用边走得到带记号的单元；成环的那一支按走不到算（环上别的边照走）。」；`.claude/gate.d/doc-experiments.sh:3432`：「if id(unit) in visiting:」交回 False；`.claude/gate.d/doc-experiments.sh:3437`：「reaches[id(unit)] = result」不分这个结果是不是在环被截断时算出的，一律记下。

打法（sync-cycle-new-entry-after）：green 样本里把表里的 `enumerate_crash_states` 改成先调新入口 `enumerate_crash_states_one_more_round(0)`、再调 `enumerate_crash_states_with_every_kind_of_observer()`；新入口（公开、写在文件末尾）调回 `enumerate_crash_states()`。主循环照文件次序先算 `enumerate_crash_states`：访问中 {它}，先走到新入口，新入口唯一的出边回到正在访问的 `enumerate_crash_states`，按环截断算假、记进 `reaches`；轮到新入口时直接取这个假，不报。按认法 ④ 它经 `enumerate_crash_states` 走得到记号，应当红。对照（sync-cycle-new-entry-before-control）把新入口挪到 `enumerate_crash_states` 前面，判红点名它。同一对函数、同一组边，结局只随文件里的先后翻。

四句：分辨修补前后（修补前没有这一格）；看得到（整张图在格手里）；打的是「源码里认出而表里没有的（新加的入口）判红」那一分句；判决给的改法就是这一格，不起作用。轻重低：要两个公开入口互相调，真源里此刻没找到这种对（没核全，推的）。

我提的改法（被攻过零轮）：不递归记忆化，改成从带记号的单元沿反向边做一次广度遍历，走到的都是「走得到」；或只记环外算出的结果。推翻条件：sync-cycle-new-entry-after 退 1。

## 二、跑前登记那一格的续行与注释新认法（C3）

代码：续行并接对围栏外行首是命令的行同样做（`.claude/gate.d/doc-experiments.sh:3181`：「joinable = in_fence or (not line.lstrip().startswith("|") and bool(command_spans(line, False)))」），续行判断照 shell 注释认（`.claude/gate.d/doc-experiments.sh:3067`：「return shell_comment_start(line) is None and (len(line) - len(line.rstrip("\\"))) % 2 == 1」，不分围栏里外）；可截掉行尾注释只在围栏里做（`.claude/gate.d/doc-experiments.sh:3186`：「comment = shell_comment_start(line) if in_fence else None」）。
同一行在围栏外：反斜杠落在 `#` 之后时，格照 shell 认它是注释、不续行；`#` 之后写的 `--features gpu` 却照样切词、算进特性。表格整格的命令（反引号里的那一截）也一样不截。

打法（out/expected.txt 第二节）：
- prereg-list-item-trailing-comment：列表项下缩进的一行 `task-server/task submit -- cargo run --release -p singlefs-checker-tier --bin e990_judge_sample  # 以后加 --features gpu`（门禁自己认的「行首是命令的行」）。格判绿「走判段的 1 条」；bash 不把注释交给 cargo，装置不带 gpu 编、判段只打 NOT_RUN。
- prereg-table-cell-trailing-comment：同一条命令写进表格整格的反引号里，同样判绿。
- 对照 prereg-fence-trailing-comment-control：同一条命令写在围栏里，判红（与门禁样本 preregistration-judge-features-trailing-comment-red 同形）。三份只差命令写在哪。

与第二轮 B4 的差别：B4 是注释行以反斜杠收尾把下一行吞进注释（并接那一步）；这里是行尾注释里的特性被当成参数（切词那一步），第二轮修补只在围栏里截。

四句：分辨修补前后与围栏里外（修补后围栏里判对、围栏外照旧判错）；看得到（`#` 就在那一行）；打的是判据 ④「特性……里没有 gpu（或 <包>/gpu）判红」——被注释里的字满足了「有 gpu」；判决给的改法「注释行不并」在这几格上不起作用（这里没有续行）。
我提的改法（被攻过零轮）：截注释不看在不在围栏，凡是 `command_spans` 认出的那一段（行首是命令的行、表格整格反引号里）都照 `shell_comment_start` 截；加两份红样本。推翻条件：两个世界有一个退 1。

## 三、`compile-then-swap.py` 经命令行的自证（C4）

第二轮修补把自证改成每一项都经 `run_command_line()` 起，文件头宣称（`scripts/hosts/compile-then-swap.py:46`：「命令行上的参数怎么交到 cargo 手里一并在判。」），成功句写（`scripts/hosts/compile-then-swap.py:897`：「每一项都经命令行那一层起，命令行上的特性（--features 写几次取并集、--features=、-F、-F<值>、引号里空格隔开的几个、--all-features）」）。23 项里没有一项给 `--release`（`grep -n release` 只命中用法、文件头、`build()` 与 `parse_command_line()`，量的）。
执行员定义写死的用法正是 `--release` 加特性：`.claude/agents/experiment-runner.md:30`：「主 agent 经 `compile-then-swap.py … --scratch <草稿目录> --release` 编过换上」，二进制路径也写死在 release 下（同一行：「二进制在 `<草稿目录>/compile-then-swap/target/release/e<号>_<英文名>`」）。

两个变异体，各只改 `build()` 一行（`tools/make-mutants.sh`）：
- release-drops-features：`scripts/hosts/compile-then-swap.py:334`：「if features and mode != "features-dropped":」后面加 `and not release`。带 `--release` 时特性不交给 cargo。
- release-ignored：`scripts/hosts/compile-then-swap.py:333`：「profile = ["--release"] if release else []」换成 `profile = []`。只编 debug；同一个草稿目录上一次编过的 `target/release/<名>` 还在时，执行员照续派提示跑的是那一份旧二进制。
照代码推：两个变异体的自证都「查了 23 项」全过；探针在三种 `--release` 加特性的写法上，前一个 `gpu_in_every_call=no`、后一个 `release_in_every_call=no`（out/expected.txt 第四、五节）。

与第二轮 B6 的差别：B6 是自证绕过命令行那一层、直接调 `swap()`；修补后命令行那一层进了自证，没进的是真用的参数组合。
四句：分辨修补前后吗——不分，修补前后的自证都不给 `--release`，这一格打的是修补后新加的那句「一并在判」的射程；看得到（参数就在命令行）；打的是第二轮判决 B6 那一行的验收「丢掉命令行特性的变异体自证判红」只在不带 `--release` 时成立；判决给的改法（自证经命令行起）在 `--release` 这一格上不起作用。
我提的改法（被攻过零轮）：加一项 release-features-passed-through（`--release --features gpu`：两条 cargo 都带 `--release` 与 gpu），配弄坏开关 release-drops-features、release-ignored 各一条。推翻条件：探针照任一变异体 `selftest_in_process rc=1`。

## 四、执行员数 NOT_RUN 的新句（C5）

第二轮修补加的句子：`.claude/agents/experiment-runner.md:40`：「任务日志用 `task-server/task log <号> --all | grep -c NOT_RUN`（不带 `--all` 默认只给最后 200 行，数不全）」。
客户端的用法行：`task-server/task:25`：「打那一次尝试（默认最后一次）的日志：默认最后 LOG_TAIL_LINES 行，--all 整份」。`--all` 补上的是行数，不是尝试：一个任务跑了几次尝试，`task log <号> --all` 只给最后一次。
可达的历史：同一份定义第 1c 步要求挂钟超过 2 小时的产物写成续跑脚本（`.claude/agents/experiment-runner.md:31`：「同一条命令重起只补没做完的」）；服务撞到内存上限会放大重排、卡住会重起（共用约束「子 agent 不交任务」那一条：内存上限、卡住、撞上限放大重排都由服务定），重起是同一任务的下一次尝试。第一次尝试跑完一段、判段那一段打了 NOT_RUN（装置这样起：带 gpu 的配置、二进制没编进 gpu），第二次尝试照续跑只补后面不走判段的那一截；执行员照这一句数，最后一次尝试里 0 行，「合计得 0」成立，按判段跑了报。NOT_RUN 只打在标准输出、不进产物文件时（定义自己把「装置的标准输出（任务日志）」与「它写出的每份产物」分开数，正是这一种），产物文件那一半也补不回来。
四句：分辨修补前后（修补前用不带 `--all` 的写法，同样只看最后一次；修补后的句子宣称补全了）；看得到（`task-server/task status <号>` 列出每一次尝试，`task log <号> --attempt <第几次> --all` 逐次取得到）；打的是第 4c 步「合计得 0」那一分句；判决给的改法（加 `--all`）在多次尝试这一格上不起作用。
我提的改法（被攻过零轮）：句子改成「`task-server/task status <号>` 列出的每一次尝试各 `task-server/task log <号> --attempt <第几次> --all | grep -c NOT_RUN`，逐次相加」。没造世界：要服务真跑出多次尝试，只能主 agent 在服务上造，我没写那条命令（推的）。

## 五、统一任务服务按源文件头认要卡与四份声明（C6）

服务这一侧：`task-server/task_server_lib.py:247`：「（crates/ 或 scripts/experiment-apparatus/ 下任一层的 src/bin/<名字>.rs 或 src/bin/<名字>/main.rs）前同样行数里的」；找源文件只认 `src/bin` 目录（`task-server/task_server_lib.py:1984`：「if os.path.basename(directory) == "bin" and os.path.basename(os.path.dirname(directory)) == "src":」），根是 `task-server/task_server_lib.py:1970`：「BINARY_SOURCE_ROOTS = ("crates", "scripts/experiment-apparatus")」。
二进制这一侧：没经服务给卡、配置开了 GPU 时退出、出路写声明（`crates/singlefs-checker-tier/src/gpu_card_configuration.rs:397`：「出路：整条经 task-server/task submit 交，服务清场、分卡；要卡的入库装置二进制在 src/bin/<名字>.rs 文件头写 `// task-server-gpu-arguments: …`」）。

打中的三处：
1. 定义与门禁都不要求写这一行。`.claude/agents/`、`.claude/rules/`、`.claude/main-agent.md`、`.claude/agent-common.md` 里 `task-server-gpu-arguments` 0 处（`grep -cF`，量的）。设计员定义要走判段的装置带 gpu、清场由服务做；执行员定义把入库装置的产物命令写成裸二进制（`.claude/agents/experiment-runner.md:30`：「执行员交回的产物命令只执行这一份（写成 `task-server/task submit -- <二进制…>`）」）。两份都没说二进制要在源文件头声明要卡的模式。下一个入库装置照这两份写、照第二轮修补的流程走：编时带 gpu、交裸二进制，服务认不出要卡、不清场不分卡，二进制在配置开了 GPU 时退 78；执行员按本次未跑报，要改源文件头就得再走一次 compile-then-swap 与单测（入库装置的单测是重型）。没有门禁核「源文件调了 `JUDGE_ENTRIES` 的 `src/bin` 装置有这一行」。
2. 此刻就有两份：调判段入口的 `src/bin` 装置 6 份，没写声明的是 `e171_three_oracle_collusion.rs`、`e171_three_oracle_collusion_after_the_oracle_fixes.rs`（`grep -LF`，量的；背景材料说归做 E171 的会话，去向由辩方那一格判，这里只记它与第 1 条同一个缺口）。
3. 服务文件头说也认 `scripts/experiment-apparatus/` 下的同名源，那棵树里 `src/bin` 目录 0 个（`find … -type d -name bin`，量的），源文件在根上、经 `[[bin]] path` 登记，bin 名照执行员定义写连字符（`.claude/agents/experiment-runner.md:32`：「`[[bin]]` 的 `name` 全写连字符」），与蛇形的源文件名永远对不上：这一半一份也认不到。那棵树里的 GPU 装置（E163、E174）此刻经 `replay.sh`（文件头写了 `*`）或 python 入口起，不靠这一条，所以此刻不出事；文件头的话说宽了。
四句：分辨修补前后（修补前四份也不认，修补后四份认了，第五份照旧不认）；看得到（服务读源文件头，二进制自己读配置）；打的是第二轮判决「裸二进制要卡」那一行的去向「服务认得出入库装置二进制要卡」——只对写了声明的那几份成立；判决给的改法（服务按声明认）在没声明的装置上不起作用。
我提的改法（被攻过零轮）：设计员与执行员定义各补一句「入库装置的源文件调了 `JUDGE_ENTRIES` 里的入口的，文件头前 60 行写 `// task-server-gpu-arguments: <走判段的模式…>`（不分模式写 `*`）」；门禁 doc-experiments 加一格：`crates/*/src/bin/` 下源文件调了判段入口而没有这一行的判红；服务文件头删掉 `scripts/experiment-apparatus/` 那一半，或照 `[[bin]]` 的 name 与 path 认。

## 六、三份规则按小节点名之后，定义之间说的话（C7、C8）

读法：`.claude/agent-common.md:3`：「与定义冲突时以定义为准。」；`.claude/agent-common.md:21`：「点名的小节是这件活要照做的那几条」；`.claude/agent-common.md:61`：「- 不做 git 写操作（`add`、`commit`、`stash`、`checkout`、`restore`、`reset`、`worktree`……），不提交，不推送。」

C7：`.claude/agents/tooling-writer.md:16`：「`.claude/rules/path-moves.md`「怎么做」（搬文件、改路径时）」。那一节第 2 步：`.claude/rules/path-moves.md:13`：「2. `git mv` 搬。」，整目录搬时先后起 `python3 scripts/kb/move-directory-in-use-check.py`，`.claude/rules/path-moves.md:14`：「退 1 时等它列出的进程退出、子 agent 交回再搬，等不了的给在跑的子 agent 发消息改路径」。
- `git mv` 是 git 写操作，共用约束不许；工具员定义点名这一节「要照做」，冲突时又以定义为准。照定义读，工具员搬文件就 `git mv`：改名直接进暂存区，主 agent 暂存提交时以为暂存区里只有自己暂存的（`.claude/singlefs-ai-sop/rules/session-wrapup.md`「4. 同一个仓里有没有别的会话在飞？」那一节「暂存区里只有我的」不再成立）。钩子不拦 `git mv`（Bash 检出钩子 ⑮ 只拦清整个索引的写法，⑫ 认的是命令位置上的 `mv`，量的：`grep` 钩子里 `git mv` 0 处）。
- 仓里的脚本 `move-directory-in-use-check.py` 子 agent 不能直接起（不在 `.claude/hooks/lib_heavy_tests.py` 的 `TEXT_TOOLS_RUN_DIRECTLY` 名单里，量的），「等进程退出」「给在跑的子 agent 发消息」工具员手里没有对应的工具。这一节点名之后照做不了，定义里没写哪几步交主 agent。
- 四句：分辨修补前后（修补前工具员不点名这一节）；看得到（三份文字都在工具员开工先读的范围里）；打的是第二轮判决之后同一批「三份规则由定义按小节点名」的句子；那一批的改法就是点名，不起作用。
我提的改法（被攻过零轮）：点名改成「`.claude/rules/path-moves.md`「怎么做」第 1、3、4、5、7 步（第 2 步的 `git mv` 与登记、搬前搬后的在用检查写进报告交主 agent）」，或在工具员定义里补一句同义的。

C8：`.claude/agents/kb-scribe.md:16`：「`.claude/rules/path-moves.md`「改一个全仓术语：正文之外还有五处会红」（规格是改术语的）」。那一节的表要求文件名含旧名的照「怎么做」搬（`.claude/rules/path-moves.md:41`：「文件名含旧名的，按本规则「怎么做」那一节搬」）、留旧名的登记进 `.claude/term-rename-exempt`、kb 之外的 C / I 引用手工全仓搜；写回员写范围只有 `.claude/kb/**`、不变量清单与工作根，定义又写 `.claude/agents/kb-scribe.md:15`：「规格里没写的句子一个字不加」。点名的小节要它做的几样它都不能做，只能停下；照定义的优先级不会写错东西，轻重低。改法：点名写成「（规格是改术语的，用来核规格有没有漏这五处；漏了停下报告）」。

## 没打中的形状

- `merge-and-verify.sh` 读不出与不是成员照跑：试了包名行写成不带引号、带 `$`、空串、两个名字、同一份里写两行、名字是成员而目录名不同、产品 GPU 包的 path 依赖不是工作区成员（checker 档包对 `prune-server` 有 path 依赖、它在 `exclude` 里）。前四种都落进「读不出 → 照跑」，方向对；写两行时取头一行，与那一道里 bash 取后一行可能不同，但那一道此刻只有一行（`grep -n PRODUCT_GPU_PACKAGE_NAME` 只有第 600 行一处赋值，量的）；非成员 path 依赖要补丁改到 `crates/` 之外，crates.patch 只收 `crates/` 与 `litmus/`（推的），没造成世界。
- judge-entries-in-sync 的其余认法：字面量与生命周期的切法（`'_`、`'a'`、`b'x'`、带花括号的字符串）、`impl … for` 整块不算、`pub(crate)` 不算、`#[cfg(feature = "gpu")]` 成对函数、带 `for<'a>` 的固有 impl 被当成 trait 实现。前几种照代码走对；成对函数在真源里都是私有、路径上另有 `cfg!` 记号（量的：`crash_state_enumeration.rs` 第 925、941 行一对，`crash_amplification.rs` 第 1540、1559 行一对）；`for<'a>` 那种真源固有 impl 头里 0 处（量的），只是潜在的。
- 判段入口写在第三份模块里（`crash_judge_dispatch.rs` 这类）：格上方已列为管不到的，不算打中；真源里判段去路的记号除两份判段源文件外只在 `crash_identity.rs` 测试里与 e185 装置自己里出现（`grep`，量的）。
- `compile-then-swap.py` 的特性写法：`<包>/gpu` 照原样交给 cargo、`--features` 与 `--all-features` 同给、`-F` 开头的位置参数，都走得通（推的）；`main()` 那一层没进自证，但它只有六行、没有分支能丢特性。
- 四份声明逐份核模式：e158 不给参数取 `all`、`first_non_option` 对 e161 的 `--force`、e168 的 `--geometry`，都与声明的词对得上（读代码，推的）；声明漏的模式会让二进制退 78，不会静默挑卡（`gpu_card_configuration.rs` 那一支，读代码）。
- 三份规则的另几处点名（实现员、调查员、攻方腿点名 session-wrapup 第 5 节，实现员、调查员点名 timer-only-remind「不用」）：与共用约束只是叠加（多写一行「没删 <全路径>：<为什么>」），删服务编译目录由钩子拦（`.claude/hooks/lib_heavy_tests.py` 的 build_directory_removal），没找到会做错的读法。

## 这条腿自己的限度

- 一条命令都没跑：子 agent 不往服务交任务，世界、变异体、门禁两格与探针的结局全是照代码推的，写在 `out/expected.txt`；推错的话以主 agent 复跑的为准。量的只有 `grep`、`find`、`awk`、`sha256sum` 这几样只读命令的计数。
- C1 的世界都是小样本：真源上这一格此刻判什么没跑（第二轮修补报告的 D14 那条命令，背景材料里没见到任务号）；真源里 51 个签名带数组的函数不进单元，此刻 13 个名字对得上的话是碰巧没落在路径上，我没逐个核这 51 个在不在某条判段路径上。
- C5 没造世界：要服务真跑出同一任务的几次尝试才看得到，我造不出；只核了客户端用法行与定义逐字对不上。
- C6 的「下一个装置退 78」是读 `gpu_card_configuration.rs` 那一支推的，那份文件不在快照里、是别的会话改的，此刻工作区那一份（ff98c3ce）。
- C7、C8 是字句冲突，没有可跑的世界；会不会真有工具员照点名的小节 `git mv`，是推的。
- `tools/rerun.sh` 只改了 `rerun_steps`；模板自带的 `--selftest` 在与仓无关的临时目录里起拷出去的那一份，我的 `rerun_steps` 要模型目录里的 `worlds/` 与 `tools/`，那一份自证会退非 0，不要拿它判模型。
- 没读禁读清单里的辩方报告；没判 R 格。

## 要主 agent 跑的命令

都不是重型。前三条在仓根 `/home/fy5090/code/singlefs` 起（submit 不给 `--cwd`）。`--work` 与探针的 `--work` 我写成主 agent 自己的草稿目录 `/home/fy5090/.local/share/singlefs/work/d137-ledger-defs-r3-main/`，要换就整串换。

1. 造世界与变异体：
   `task-server/task submit -- bash research/prompts/d137-ledger-defs-r3-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs --work /home/fy5090/.local/share/singlefs/work/d137-ledger-defs-r3-main/rerun-1`
   看：`TASK_SERVER_RESULT` 的 `exit=0`；`task-server/task log <号> --all` 里十行 `name=world_built`、两行 `name=mutant_built … changed_lines=2`（`source_sha256=6328769d`）、一行 `name=rerun_roots`。
2. 单跑门禁两格（脚本里每条门禁命令各自带范围声明；要第 1 条先跑完）：
   `task-server/task submit -- bash research/prompts/d137-ledger-defs-r3-opus-model/tools/judge-worlds.sh /home/fy5090/.local/share/singlefs/work/d137-ledger-defs-r3-main/rerun-1/worlds`
   看：`exit=0`（脚本不判对错）；`task log <号> --all` 里一行 `name=judge_inputs gate_sha256=7e0f7a6b`、十一行 `name=judge_world`，每行的 `rc=` 与 `out/expected.txt` 第二节第三列逐行比。对照四行（sync-slice-parameter-new-entry-control 1、sync-slice-parameter-on-the-path-control 0、sync-cycle-new-entry-before-control 1、prereg-fence-trailing-comment-control 1）要先对上，这一批才作数；每个世界的门禁原样输出在 `rerun-1/worlds/<世界名>.<格名>.out`。
3. 探针照真脚本：
   `task-server/task submit -- python3 research/prompts/d137-ledger-defs-r3-opus-model/tools/release-features-probe.py --script scripts/hosts/compile-then-swap.py --work /home/fy5090/.local/share/singlefs/work/d137-ledger-defs-r3-main/probe-real --selftest`
   看：`exit=0`；四行 `name=cli_release_features` 与 `out/expected.txt` 第三节逐行比；自证打「✓ compile-then-swap 自检通过（查了 23 项…」，末行 `name=selftest_in_process rc=0`。
4. 探针照变异体 release-drops-features（要第 1 条先跑完）：
   `task-server/task submit -- python3 research/prompts/d137-ledger-defs-r3-opus-model/tools/release-features-probe.py --script /home/fy5090/.local/share/singlefs/work/d137-ledger-defs-r3-main/rerun-1/mutants/release-drops-features/scripts/hosts/compile-then-swap.py --work /home/fy5090/.local/share/singlefs/work/d137-ledger-defs-r3-main/probe-release-drops-features --selftest`
   看：与 `out/expected.txt` 第四节逐行比；C4 成立看末行 `selftest_in_process rc=0` 且三行 `--release` 写法 `gpu_in_every_call=no`。
5. 探针照变异体 release-ignored：
   `task-server/task submit -- python3 research/prompts/d137-ledger-defs-r3-opus-model/tools/release-features-probe.py --script /home/fy5090/.local/share/singlefs/work/d137-ledger-defs-r3-main/rerun-1/mutants/release-ignored/scripts/hosts/compile-then-swap.py --work /home/fy5090/.local/share/singlefs/work/d137-ledger-defs-r3-main/probe-release-ignored --selftest`
   看：与第五节逐行比；C4 成立看 `selftest_in_process rc=0` 且三行 `release_in_every_call=no`。
6. （C1 旁证，可选）真源上这一格此刻判什么：
   `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-experiments.sh 的 judge-entries-in-sync 格; 范围=1 格; 为什么=核判段入口表同步格在真源两份判段源文件上的判定与扫到的函数单元数' task-server/task submit -- bash .claude/gate.d/doc-experiments.sh --check judge-entries-in-sync`
   看：`exit=` 与成功句或红句里的「扫了 N 个函数单元」；扫到的单元里没有签名带数组的那 51 个函数（含测试模块里的），绿了也不说明它们同步。

跑完这几条都不留仓副本与编译目录；探针与世界的临时仓在各自 `--work` 下，判决写完照第三步删。

## 没做什么

- 不判 R 格（辩方那几格），不替主 agent 采纳；没读禁读清单里的辩方报告。
- 没往统一任务服务交任务，判决要引的数一个都没复跑（命令在上一节）；模型上的数全是推的。
- 没改任何被判文件；我提的改法都只在我的模型上推过、被攻过零轮。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/d137-ledger-defs-r3-opus` 里没有仓副本、编译目录或库拷贝（只放过一份模板拷贝与一段文本，用完已删）。

## 引文核对

`python3 scripts/agents/cite-check.py research/prompts/d137-ledger-defs-r3-opus-output.md` 末行（交回前现跑）：
```
  ✓ 核了 30 处引文、13 处裸行号，对上 30 处，没判 13 处（其中裸行号没有引文可比 13 处；逐处列在上面）
```
