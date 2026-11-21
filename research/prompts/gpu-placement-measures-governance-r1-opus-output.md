# 攻方腿报告：gpu-placement-measures-governance-r1（治理文件轮第一轮）

攻击面：正文判据格「误伤」「判法本身」两格与「已经量的」一节逐句核；第一轮三个方向（过度外推与越位、内部矛盾与循环依赖、哪一条最脆弱）。这条腿没有往任务服务交任何东西，下面凡写「待主 agent 跑」的预期退出码都是推的，结局以主 agent 复跑的 `task wait` 末行与日志为准。

## 复跑命令与模型文件

复跑（主 agent 交，在新 shell 里起；命令原样在末尾「要主 agent 跑的命令」一节）：

`bash research/prompts/gpu-placement-measures-governance-r1-opus-model/tools/rerun-from-a-clean-work-directory.sh <仓根> <开工快照目录> <运行目录，工作根下、以 -run 收尾> <模型目录>`

开工快照目录给 `research/prompts/gpu-placement-measures-governance-r1-snapshot`（rerun.sh 拿它的 judged-sha256.txt 先核被判文件）。复跑造 48 份样本、逐份起被判的判法，每份打一行 `name=case case=<样本> predicted_exit=<预期> observed_exit=<实测> verdict=match|mismatch`，末行 `name=summary cases=48 mismatches=<数>`；mismatches 不为 0 时退 1（说明这份报告的某条预期没坐实）。

| 文件 | sha256 |
|---|---|
| tools/rerun.sh | f4e3e5d1932b0d361a2444b41fb2cf5ea40da4787a03aceb5d0583f844272656 |
| tools/rerun-from-a-clean-work-directory.sh | e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b |
| tools/build_samples.py | 4e58c20c31206558f2f687cf40257d0c381eaf446b57a3440aae229163fae47e |
| tools/build_doc_process_samples.py | 84e5069a15eee18119c2e4624eb6cf4938810e24a028a8a634584445dbd8d29d |
| tools/list_targets.py | 3c6c3270376dc22e0c88fbeb151fee6b7043e4842b5ba2aeafac0bfd25dbf3d8 |

rerun.sh 由 `scripts/agents/three-way-rerun-template.sh` 拷来，只换了 `rerun_steps`（第 1–79 行与末 80 行与模板逐行相同，`diff` 核过）。探针的内存与时间没量过：推的是 10 次 `cargo build --tests`（每次两个不带外部依赖的小包）加 11 次 crate-roles 判法、13 次门禁阶段、8 次喂钩子、6 次 `cargo metadata`（不编），合计几分钟、常驻内存在 rustc 的量级（几百 MiB）。

## 各格判定一览

| 格 | 判定 | 打中的东西（详见各节） | 坐实程度 |
|---|---|---|---|
| 误伤：放置类钩子词表 | 打中 | 九个放置类词里「测试包」「装置包」「实验包」「新开包」自带「包」，另要「包」那一条对它们是空条件；「包」又命中「包括」「包含」：问实验包起名、问带「归位」的轮名、问「放进 kb 吗」而说明写「包括」的弹窗都被拒 | 读码核过；样本 3 份待主 agent 跑 |
| 误伤：钩子放过换说法的放置类弹窗 | 打中 | 「放在哪个包」「搬到」、偏离写在选项 label 里（原来那一次的「整份搬进测试包」就是选项名）、问「哪个目录」、问「哪个 crate」都放行 | 读码核过；样本 4 份待主 agent 跑 |
| 误伤：规则与钩子对不上 | 打中 | 规则写放置类包括「哪个目录」，钩子另要「包」或 `crates/` | 读码核过 |
| 误伤：crate-roles 整包滞后一行 | 打中 | 理由是「拆包进行中」，放行的却是那个包里此后新加的一切（新的 e 装置、新的测试目标） | 读码核过；样本 1 份待跑 |
| 误伤：只判八格的名单与截止日 | 打中 | 名单不核行指的文件还在不在，正文归档后永远不缩；截止日比规则生效晚一天；判几格看标题里「第一个」日期，写正文的人引一个更早的日期就只判八格或整份不查 | 读码核过；样本 3 份待跑 |
| 误伤：crate-roles 放行表 e165 一行、⑨ 两张表 6 行 | 没打中 | 理由与用户定案、源码逐行对得上，引用不在了会判红、能缩 | 读码核过 |
| 判法：「e<数字>_ 文件名」 | 打中 | 真树上验证包 `src/bin/` 里有两份不带号的实验装置（E172、E142 起的），用户定案归实验包，判法看不见也不进滞后表；另有连字符名、`[[bin]]` 指到 `src/bin/` 外、产品包 `examples/`、`src/main.rs` 四种写法 | 真树现查；样本 4 份待跑 |
| 判法：「库为空」与「被依赖者数」 | 打中 | 测试包的库里写一行 `pub use p_gpu;` 就不算空；登成测试支撑、两个依赖者写在 `cfg(any())` 下（永远不编）就够数；包嵌一层（`crates/<包>/<子目录>/Cargo.toml`）整个看不见 | 读码核过；样本 3 份待跑 |
| 判法：⑨ 的路径解析 | 打中（最脆弱） | 同一处 `#[path]` 跨包引用换九种写法都判绿：`cfg_attr`、`use std::include_str as …`、方括号定界、原始字符串、行内模块里的 `#[path]`、绝对路径、经一份非 `.rs` 的中转文件、放进名叫 `target` 的模块目录、放进点起头的目录 | 读码核过；判法样本 10 份、编译样本 10 份待跑 |
| 判法：写闸第十一道认「已有门禁」 | 只打中已知的那一处 | 认法本身（解开符号链接、取 fixtures 下第一层名、核 `.claude/gate.d/<名>` 是文件）没造出绕过；绕得过的是 Bash 一路（`cp`、`set -o noclobber` 排他新建），钩子文件头自己写了、调度记录已排给下一件 | 读码核过 |
| 已经量的 | 有对不上与写不出出处的 | 「放置类钩子 10 条」「写闸」开关复跑写不出条数出处；code-round-checklist「绿」在真树上第九格判了 0 份；code-tooling 的样本档（crate-roles 的五份样本在里面）没结局、正文没写；`git-hooks-install.py:133` 行号已漂；被判文件之一 `break-switch-replay-files.tsv` 已与开工快照对不上 | 逐句对调度记录与现查 |
| 越位：产品包一律不许有二进制 | 打中（推的） | D17（实现分层与第三方管道）定的 FUSE 薄适配一出来，产品二进制在四种角色里没有合法位置 | 读 kb 核过，没造样本 |
| 矛盾：单元图登成测试支撑 | 打中（推的） | 拆包按 F4 的读法让产品 GPU 包自己 `[build-dependencies]` 依赖单元图，与「测试支撑（只给测试与装置用的库）」相反，判法不判产品包依赖测试支撑包 | 读码与记录核过 |

## 判据格「误伤」

### 放置类钩子：词表自带「包」，「包」又命中别的词（打中）

- 判法：「PLACEMENT_WORDS = ("放哪", "放进", "住进", "搬进", "归位", "新开包", "测试包", "装置包", "实验包")」（`.claude/hooks/ask-user-placement-guard.sh:70`），另要「PACKAGE_WORDS = ("包", "crates/")」（`.claude/hooks/ask-user-placement-guard.sh:71`），都按子串找。九个词里四个本身含「包」，问句里出现这四个词之一，「另有包」那一条自动成立；单字「包」还命中「包括」「包含」「打包」「包装」。
- 自证只钉了一种不误伤的形状：「("必放：有放置类的词而没有「包」与 crates/（这一行放进 kb 吗）"」（`.claude/hooks/ask-user-placement-guard.sh:203`），说明里写「包括」就不成立。
- 反例（样本在 `build_samples.py` 的 `HOOK_CASES`，预期退 2，待主 agent 跑）：
  - `misfire-experiment-package-name`：问「实验包起什么名字？」，两个选项是两个包名。调度记录把这一问列成下一轮要问的事：「实验包怎么起名与接法」（`records/2026-11-19-产品GPU包测试与E202装置归位.md:96`）。问的是名字，不是东西放哪。
  - `misfire-round-title`：问句里带这一轮调度记录的标题「产品GPU包测试与E202装置归位」（「归位」加「包」），问的是规则改动先不先提交。
  - `misfire-contains-word`：问「这条欠账放进 .claude/kb/checks-owed.md 吗？」，选项说明写「包括出处与判据两列」。与自证里那一例只差说明里一个「包括」。
- 四句：分辨得出（同一问去掉那个词就放行）；钩子当时看得到判别它的东西（问句与说明原文都在 tool_input 里）；满足的是判据格「误伤」的「会不会拦下与放置无关的弹窗」那一分句；正文没有跑前给的改法，零轮改法见末尾表。

### 放置类钩子：换个说法就放过（打中）

- 钩子只扫问句与选项说明：「# 判法：扫 tool_input.questions 里每一问的 question 与每个 options[].description（header、label 不扫）：」（`.claude/hooks/ask-user-placement-guard.sh:11`）。文件头自认的盲区只写了「# 管不到的：换了说法的放置类问题（词表外的「挪到」「落在哪个包」）认不出」（`.claude/hooks/ask-user-placement-guard.sh:52`）。
- 反例（预期退 0 即放行，待主 agent 跑）：
  - `bypass-placed-where`：「E202 的装置放在哪个包？」，两个选项一个「搬到」测试包、一个留在产品包的 `src/bin/`，两个都偏离。「放在哪」不含子串「放哪」，「搬到」不是「搬进」。
  - `bypass-label-only`：问「E202 的装置怎么办？」，选项 label 就是当时那一次的原话「整份搬进测试包」，说明写「跟十个测试目标放一起」。当时用户选的正是这个名字：「用户定案（E202 放哪，弹窗）：选「整份搬进测试包」」（`records/2026-11-11-GPU两段与按需领卡核查.md:276`）。偏离标准的话只写在 label 里，钩子不看。
  - `bypass-directory`：「E202 的装置放进哪个目录？」，选项是 `scripts/experiment-apparatus/` 与 `research/`。有「放进」，没有「包」与 `crates/`。
  - `bypass-crate-word`：「这十个测试目标放进哪个 crate？」，选项「新开一个 -tests crate」「放回产品 GPU crate」——正是原因第 3 条那一次的形状，写成 crate 不写「包」就放行。
- 对照 `refuse-control`（「E202 的装置放哪？」、说明「整份搬进测试包」）预期退 2，证这批样本是被判的。

### 规则与钩子对不上（打中，内部矛盾）

- 规则：「- 弹窗问放置类问题（东西放哪个包、哪个目录，要不要新开包）」（`.claude/rules/directory-layout.md:25`）。钩子另要「包」或 `crates/`，问「哪个目录」而不提包的不算放置类（上面 `bypass-directory`）。调度记录写了要把规则那一条收窄成与钩子一样：「`.claude/rules/directory-layout.md` 里弹窗那一条没写「另有「包」或 `crates/`」，并进同一次落文档」（`records/2026-11-19-产品GPU包测试与E202装置归位.md:142`）。收窄之后规则就不管 F2 乙那种「装置放 `scripts/experiment-apparatus/` 还是别处」的问题，两头只能选一头。

### crate-roles 整包滞后一行（打中）

- 「crates/singlefs-background-gpu-tests	# 拆包进行中」（`.claude/gate.d/crate-roles-lag.tsv:4`）。判法对整包那一行直接跳过：「if directory in lag_paths:」（`scripts/gate/crate-roles-check.py:377`），这个包里的空库带测试、`e<数字>_` 装置、此后新加的一切都不报。
- 会缩：包删掉之后「if not os.path.lexists(os.path.join(root, path)):」（`scripts/gate/crate-roles-check.py:395`）判红，逼着删行。不会缩的情形是拆包一直不落：调度记录里拆包排在别的会话的补丁之后，没有截止。
- 反例 `tests-package-whole-lag`（预期退 0，待跑）：样本里一个已登在整包滞后行里的空库测试包，再加一份 `src/bin/e203_gc_new_apparatus.rs` 与一个新测试目标，基准滞后表与此刻相同。真树上对应的事：拆包落地之前，E202 那一份装置在测试包里还在被别的会话修（调度记录第 82–83、123 行），往这个包里再加装置或测试，crate-roles 不报。原因第 3 条（测试搬进新包）正是这个包里的事。
- 四句：分辨得出（同一份新装置放进没登整包的包就红）；判法看得到（文件在树上），是滞后表的粒度把它挡掉；满足「会不会永远不缩」与「误伤」格里「靠的是滞后表」那一分句；零轮改法：整包一行换成逐条登现有的测试目标与 E202 那一份，新加的照判。

### 只判八格的名单与截止日（打中）

- 名单只比文件名：「[[ "$1" == "$listed" ]] && return 1」（`.claude/gate.d/doc-process-records.sh:3094`），不核名单里的文件还在不在。八份旧正文在下一次归档时删出工作区，名单的八行留着、没有一格逼它删，永远不缩。
- 截止日「CODE_ROUND_CHECKLIST_PLACEMENT_CUTOFF="2026-11-20"」（`.claude/gate.d/doc-process-records.sh:738`），规则从今天（2026-11-19）起就要九格：「每一轮都逐格点名九格」（`.claude/rules/implementation-workflow.md:43`）。今天写的代码轮正文只判八格；拆包第一簇的代码轮若今天起正文，搬包的那一轮恰好不判「放的位置」。真树上两份标题日期是 2026-11-19 的正文都只有八格（`grep -c '### 检查单：'` 各 8）。
- 判几格看标题里第一个日期：「date="$(grep -oE '[0-9]{4}-[0-9]{2}-[0-9]{2}' <<<"$heading" | head -n 1)"」（`.claude/gate.d/doc-process-records.sh:3106`）。代码轮正文的标题常写它攻的那份判决（真树上「攻第一轮判决的修补（2026-11-18）」这类）；把判决日期写在成稿日期前面，就按判决日期判。
- 反例（`build_doc_process_samples.py`，待跑）：`title-date-control` 标题只有 2026-12-19、八格齐，预期退 1；`title-date-cited-earlier` 同一份正文，标题先写「攻 2026-11-18 判决的修补」，预期退 0；`title-date-before-checklist` 一份标题先写 2026-11-05、一格没写，另一份九格齐，预期退 0（前一份整份不查）。

### 没打中的放行行

- `crate-roles-allowed.tsv` 的 e165 一行：理由是门禁 formal-verification 的红例钉表起它，与用户定案「留验证包」（`records/2026-11-19-产品GPU包测试与E202装置归位.md:112`）对得上；放行表本来就不缩，条目不在了判红。
- `crate-cross-references-allowed.tsv` 4 行、`crate-cross-references-lag.tsv` 2 行：逐行对了源码（`grep -rnoE` 列出全部带 `../` 的 `#[path]` 与 `include*`），6 处都在；滞后的 2 行都随测试包删掉而引用消失或编不过，`⑨` 对用不着的行判红，会缩。

## 判据格「判法本身」

### crate-roles：「e<数字>_ 文件名」（打中）

- 判法只看 `src/bin/` 下条目名：「binaries = os.path.join(directory, "src", "bin")」（`scripts/gate/crate-roles-check.py:229`），名字照「EXPERIMENT_BINARY_NAME = re.compile(r"^e[0-9]+_")」（`scripts/gate/crate-roles-check.py:105`）；`[[bin]]` 只在产品包上看：「if role == PRODUCT_ROLE and (binaries or declared_binaries):」（`scripts/gate/crate-roles-check.py:356`）。文件头自认的只有「管不到的：角色登得对不对；名字不以 e<数字>_ 起头的实验装置二进制；src/main.rs 起的二进制」（`scripts/gate/crate-roles-check.py:41`）。
- 真树上的反例（现查）：验证包 `src/bin/` 下不带号的五份里，`deletion_write_dump.rs` 与 `new_pool_file_creation_region_bytes.rs` 是实验装置起的：「`deletion_write_dump`、`new_pool_file_creation_region_bytes` 由 E172、E142 的装置与 `replay.sh` 起」（`records/2026-11-19-产品GPU包测试与E202装置归位.md:102`），`.claude/gate.d/stage-inputs.tsv:48` 那一行也写它由 `replay.sh` 的 driver_e172 起。用户定案是「留验证包，其余进实验包」（`records/2026-11-19-产品GPU包测试与E202装置归位.md:112`），这两份属于「其余」。规则写「- 验证包只装测试、测试的脚手架，与门禁、崩溃入口自己起的工具二进制。」（`.claude/rules/directory-layout.md:22`）。判法不认它们、滞后表里也没有它们：22 份 `e` 起头的条目搬走之后 crate-roles 照绿，这两份留在验证包没有一格逼它们走。这是「名字不以 e<数字>_ 起头」那条自认盲区在真树上的两例，不是假想。
- 造的反例（`build_samples.py` 的 `crate_roles_cases`，待跑）：
  - `product-examples`：产品包 `examples/e202_gpu_cpu_gc_comparison.rs`（`cargo run --example` 起），预期退 0。原因第 1 条（产品包里放装置）换一个目录就过；规则「- 产品包不放二进制（`src/bin/`、`[[bin]]`），装置二进制尤其不放。」（`.claude/rules/directory-layout.md:23`）只点了两种写法。
  - `product-main-rs`：产品包 `src/main.rs`，预期退 0（文件头自认）。
  - `verification-bin-outside-src-bin`：验证包 `[[bin]] name = "e202_gpu_cpu_gc_comparison"`、`path = "src/apparatus/…"`，预期退 0。
  - `verification-hyphen-name`：验证包 `src/bin/e202-gpu-cpu-gc-comparison.rs`（连字符），预期退 0。
  - 对照 `product-src-bin-control`（产品包 `src/bin/e202_….rs`）预期退 1；`cargo metadata` 那 6 条把每个样本里 cargo 认出的目标（种类 bin、example、test）逐个打出来，证这几种写法 cargo 都当二进制。
- 四句：分辨得出；「是不是实验装置」这一点判法看不到——文件名不带号时，判别它的东西在 `stage-inputs.tsv`、`replay.sh` 里谁起它（判别子在仓里，不在判法读的范围里）；满足「判法本身」格「`e<数字>_` 文件名……有没有绕得过去的写法」；零轮改法见末尾表（按 `cargo metadata` 的目标判、验证包二进制改成放行名单）。

### crate-roles：「库为空」与「被依赖者数」（打中）

- 「库为空」：去掉注释与内层属性之后「return not text.strip()」（`scripts/gate/crate-roles-check.py:210`）。反例 `tests-package-one-line-library`：原因第 2 条那种测试包，库里只写 `pub use p_gpu;`，预期退 0；对照 `tests-package-control`（同一个包、库只有注释与 `#![deny(unsafe_code)]`）预期退 1。
- 「被依赖者数」：测试支撑包跳过库为空那一条（「if role != SUPPORT_ROLE and targets and library_is_empty(absolute, manifest):」，`scripts/gate/crate-roles-check.py:366`），只要「if len(users) < SUPPORT_DEPENDENTS_MINIMUM:」（`scripts/gate/crate-roles-check.py:371`）不成立；依赖表连 `target.<条件>` 下的也算：「tables += [condition.get(name) for name in DEPENDENCY_TABLES]」（`scripts/gate/crate-roles-check.py:139`）。反例 `tests-package-as-support`：同一个空库测试包登成测试支撑，验证包与仓根 `exclude` 点名的一个工程各在 `[target.'cfg(any())'.dev-dependencies]` 下写它（`cfg(any())` 永远为假，永远不编），预期退 0。
- 包的发现：「if os.path.isfile(os.path.join(crates, name, "Cargo.toml")))」（`scripts/gate/crate-roles-check.py:278`）只认 `crates/` 下一层；别的工程又「if isinstance(entry, str) and not entry.startswith("crates/")」（`scripts/gate/crate-roles-check.py:266`）把 `crates/` 起头的成员排掉。反例 `tests-package-nested`：测试包放在 `crates/p-gpu/gpu-tests/`、登进 workspace members、不登角色表，预期退 0，包数那一格也对得上（它只数 `crates/*/Cargo.toml`）。嵌在产品包目录下，⑨ 还把它与产品包当成同一个包（`own = parts[1]`），它 `#[path]` 引产品包 `tests/` 下的模块不算跨包。
- 四句：分辨得出；判法看得到（库文件内容、依赖表、目录都在），是判据写成「空不空」「数清单」；满足「库为空」「被依赖者数」两处「绕得过去的写法」。

### ⑨ 的路径解析（打中，这一轮最脆弱的一条）

判法要点：只扫「for path in sorted(glob.glob("crates/**/*.rs", recursive=True)):」（`.claude/gate.d/checker-independence-and-sync.sh:888`），且「if len(parts) < 3 or "target" in parts[2:-1]:」（`.claude/gate.d/checker-independence-and-sync.sh:890`）就跳过；相对路径一律从「base = os.path.dirname(path)」（`.claude/gate.d/checker-independence-and-sync.sh:906`）解；落点「if target_parts[0] != "crates" or len(target_parts) < 3:」（`.claude/gate.d/checker-independence-and-sync.sh:927`）算 crates/ 之外、不判；认不出的只计数：「cross_counts["unrecognized"] += sum(1 for found in ANY_INCLUDE.finditer(code) if found.start() not in recognized)」（`.claude/gate.d/checker-independence-and-sync.sh:935`）。文件头自认的只有「⑨ 认不出路径不是字面量的跨包引用（常量、变量、宏里嵌别的宏）与运行时读的文件」（`.claude/gate.d/checker-independence-and-sync.sh:1012`）。

同一处引用（产品 GPU 包 `tests/cross.rs` 引验证包 `tests/c/mod.rs` 或 `src/k.wgsl`）的九种写法，每种一份门禁样本根（拷 `checker-implementation-disjoint-cross-package-source-green` 再加文件）与一份编译样本（同样的相对布局，两个不带外部依赖的包，`cargo build --offline --tests`，测试里调 `c::shared_value()` 或读 `KERNEL`，编得过就是跨包那一份真编进来了）：

| 样本 | 写法 | ⑨ 为什么看不见 | 成功句里留不留痕 | 预期（门禁 / 编译） |
|---|---|---|---|---|
| plain-control | `#[path = "../../singlefs-verification/tests/c/mod.rs"]` | 对照 | 判红 | 1 / 0 |
| cfg-attr-path | `#[cfg_attr(all(), path = "…")]` | 两个正则都要 `#[path` 紧挨 | 不留 | 0 / 0 |
| renamed-include-macro | `use std::include_str as grab;` 后 `grab!("…k.wgsl")` | `ANY_INCLUDE` 要 `include_str!` | 不留 | 0 / 0 |
| bracket-delimiters | `include_str!["…k.wgsl"]` | `INCLUDE_FORM` 只认圆括号 | 计进「认不出」 | 0 / 0 |
| raw-string-path | `#[path = r"…"]` | `STRING_LITERAL` 要 `"` 起头 | 计进「认不出」 | 0 / 0 |
| inline-module-base | `mod common_shared { #[path = "../../../singlefs-verification/…"] pub mod c; }` | 行内模块里的 `#[path]` 按 Rust 规则从 `tests/common_shared/` 起算，⑨ 从 `tests/` 起算，多出的 `../` 把落点算到 `crates/` 之外 | 计进「crates/ 之外」 | 0 / 0 |
| absolute-path | `#[path = "<样本根绝对路径>/crates/singlefs-verification/tests/c/mod.rs"]` | `os.path.join` 遇绝对路径丢掉 base，落点首段不是 `crates` | 计进「crates/ 之外」 | 0 / 0 |
| non-rs-intermediate | `#[path = "shim.inc"] mod shim;`，`shim.inc` 里再 `#[path = "../../…/c/mod.rs"]` | 只扫 `*.rs`，中转文件是同包 | 计进「同包」 | 0 / 0 |
| target-directory | `mod target;`，`tests/target/mod.rs` 里引 | 路径段里有 `target` 整份跳过 | 不留 | 0 / 0 |
| hidden-directory | `#[path = ".shared/mod.rs"]`，里面引 | `glob` 的 `**` 不进点起头的目录 | 不留 | 0 / 0 |

编译那一栏是推的（Rust 参考手册「path 属性」一节的行内模块规则、`cfg_attr` 与宏改名导入的写法；本项目没编过），由编译样本坐实。原因第 5 条（测试目标 `#[path]` 引验证包 `tests/`）加一个 `cfg_attr(all(), …)` 就重演。四句：分辨得出；九种写法 ⑨ 读的源码里都看得到；满足「判法本身」格点名的「`#[path]` 写法」「相对路径、`..`」两分句；零轮改法见末尾表。

### 写闸第十一道认「已有门禁」（认法本身没打中；绕得过的是已知的 Bash 一路）

- 认法：目标解开符号链接、取 `.claude/gate.d/fixtures/` 下第一层目录名，「return gate if gate and os.path.isfile(os.path.join(root, GATE_STAGE_DIRECTORY, gate)) else None」（`.claude/hooks/write-guard.sh:916`）；只管「SWAP_ONLY_AGENTS = ("tooling-writer",)」（`.claude/hooks/write-guard.sh:853`）的 Write、Edit、MultiEdit。
- 试过、没绕过的形状：经符号链接或 `./`、`..` 拼的路径（先解开再比）；样本根下直接放的文件（不算哪一道的样本，也不是样本档）；`setup.sh` 这类样本里的脚本（在第一层目录里，照拒）；别的 agent 类型（写范围表里能写 `.claude/gate.d/**` 的只有工具员一行，实现员、调查员写不进去，由写闸第二道拒）；新形态的阶段目录 `.claude/gate.d/<阶段>/<用例>.case.sh`（仓里此刻一个都没有，`ls -d .claude/gate.d/*/` 只有 `fixtures/`、`lib/`、`__pycache__/`，没造样本）。
- 绕得过的：Bash 里往样本档目录写。钩子文件头自己写了「还没有门禁阶段的样本目录（新门禁的样本，与新门禁一起写）不拦；Bash 里往样本档目录写（cp、重定向）这道闸看不见。」（`.claude/hooks/write-guard.sh:110`），检出钩子 ⑫ 照写范围表放行工具员写 `.claude/gate.d/**`。其中 `set -o noclobber` 后 `cat >` 排他新建是共用约束点名许可的写法，新建一份样本正走这一路。调度记录已定「检出钩子 ⑭ 加同一半」（`records/2026-11-19-产品GPU包测试与E202装置归位.md:142`），这里不另造样本，只记它在开工快照这一刻还开着。

## 「已经量的」逐句核

正文这一节没有给任务命令，也没有贴日志行；这条腿手里没有主 agent 给的任务号，`task-server/task log` 读不到。下面逐句对调度记录与现查，标「有出处」「写不出出处」「对不上」。

| 正文那一句 | 出处 | 核的结果 |
|---|---|---|
| `crate-roles-check.py --selftest` exit=0 | 「自证：crate-roles 判法、新钩子都 exit=0」（`records/2026-11-19-产品GPU包测试与E202装置归位.md:129`） | 有出处。那一趟在措施工具员换上时跑；被判那一份的 sha256 7cb3b91d… 与那件工具员草稿里的新版逐字相同（`sha256sum` 量的），之后没再改过，结论能搬过来 |
| `ask-user-placement-guard.sh --selftest` exit=0（「查了 25 种」） | 「头注补点名的 3 条、放置类钩子、写闸三条开关复跑都 exit=0」同一行前半「放置类钩子、写闸两份自证 exit=0」（`records/2026-11-19-产品GPU包测试与E202装置归位.md:143`） | exit=0 有出处；「25 种」调度记录里没有。按 `selftest()` 数是 14 例样本 + 规则文件 1 + stdin 4 + 判不了 3 + 抛异常 1 + 那一节读不出 1 + 平常跑设开关 1 = 25（推的，读码数的），要引就引那一趟日志的成功句 |
| `write-guard.sh --selftest` exit=0 | 同上一行 | 有出处 |
| 样本档两道都「判得都对」 | 「checker-independence-and-sync、doc-process-records 两道样本档「判得都对」」（`records/2026-11-19-产品GPU包测试与E202装置归位.md:143`） | 有出处。漏写的：crate-roles 的五份样本住在 code-tooling 的样本档里，同一行写「code-tooling 的样本档还在跑」（`records/2026-11-19-产品GPU包测试与E202装置归位.md:143`），正文「已经量的」与「没跑成的」都没提它，crate-roles 格的样本档在被判的版本上没有结局 |
| 开关复跑 crate-roles-check.py 14 条 | 「复跑了 14 条开关……逐格相同 14 条」（`records/2026-11-19-产品GPU包测试与E202装置归位.md:129`） | 有出处，判法之后没改过 |
| 开关复跑 放置类钩子 10 条 | 记录里写的是第一版的「「复跑了 9 条开关……逐格相同 9 条」」（`records/2026-11-19-产品GPU包测试与E202装置归位.md:129`）；第二版只写「复跑都 exit=0」，没写条数 | 写不出「10 条」的出处。文件头现数 10 条（`grep -c '^#   ASK_USER_PLACEMENT_GUARD_BREAK='` 量的 10），复跑登记表那一行还写「九条开关（文件头 ASK_USER_PLACEMENT_GUARD_BREAK= 现数）」（`.claude/gate.d/break-switch-replay-files.tsv:96`），记录也认了「放置类钩子九条变十条、写闸五条变六条」（`records/2026-11-19-产品GPU包测试与E202装置归位.md:142`）还没改 |
| 开关复跑 写闸 | 同上，记录没写条数 | 写不出条数出处。登记表那一行写「五条开关（照 switch_entries 的读法现数」（`.claude/gate.d/break-switch-replay-files.tsv:61`），文件头现在 `WRITE_GUARD_BREAK=` 五条加 `LIB_WRITER_LOCK_BREAK` 一条，共六条 |
| 三条补点名的头注 | `records/2026-11-19-产品GPU包测试与E202装置归位.md:143` | 有出处 |
| 真仓单格 checker-implementation-disjoint、code-round-checklist 绿 | 「真仓单格：checker-implementation-disjoint、code-round-checklist、rules-lint、命名检查都 exit=0」（`records/2026-11-19-产品GPU包测试与E202装置归位.md:129`） | 那一趟跑在第二件工具员换上之前；两道阶段前后两版的差只在 `#` 注释行（`diff` 量的），判法同一份，推的能搬。另一处：真树上代码轮正文的标题日期都不晚于 2026-11-19、没日期的 8 份都在名单里，第九格判到的是 0 份（推的，读码与 `head -n1` 量的标题），「code-round-checklist 绿」对「放的位置」一格什么都没证 |
| 真仓单格 crate-roles 绿 | 「code-tooling 九格里 crate-roles、agent-write-scope、gate-structure、break-switch-headers、selftest-parallelism 绿」（`records/2026-11-19-产品GPU包测试与E202装置归位.md:130`） | 有出处 |
| 真仓单格 research-script-selftest-coverage 绿 | 「research-script-selftest-coverage 格转绿」（`records/2026-11-19-产品GPU包测试与E202装置归位.md:143`） | 有出处 |
| 没跑成的：拒算点 `git-hooks-install.py:133` | 「拒算点 `scripts/gate/git-hooks-install.py:133`」（`records/2026-11-19-产品GPU包测试与E202装置归位.md:131`） | 记录有，今天对不上：那份文件此刻第 133 行不是那条调用，`subprocess.run(["bash", "-c", PEER_RUNNER` 在 `scripts/gate/git-hooks-install.py:154`（`grep -n` 量的）。那份文件不在被判的 20 份里、别的会话在改，行号漂了 |

另：`sha256sum -c research/prompts/gpu-placement-measures-governance-r1-snapshot/judged-sha256.txt` 在仓根跑，20 份里 `.claude/gate.d/break-switch-replay-files.tsv` 一份 FAILED；暂存区、HEAD、工作区三版的 sha256 都不是快照登的 da89c2fe…（量的）。开工之后别的会话改了它，被判的那一版此刻在树上找不回来，主 agent 核这一份要从快照之外找。

## 第一轮三个方向

### 过度外推与越位

- 产品包一律不许有二进制（「- 产品包不放二进制（`src/bin/`、`[[bin]]`），装置二进制尤其不放。」，`.claude/rules/directory-layout.md:23`），四种角色里也没有「产品二进制」。原因第 1 条要拦的是装置二进制；规则与判法把产品自己的二进制一并禁了。D17（实现分层与第三方管道）定的是「第一版没有挂载：FUSE 是后来的一层薄适配」（`.claude/kb/decisions/17-实现分层与第三方管道.md:87`）——那一层要出一个挂载程序，按今天的规则它放进任何产品包都判红，只能走文件头自认判不到的 `src/main.rs`。推的：今天没有这样的二进制，红的是将来；改法是给产品二进制一个登记位（放行表登名字），而不是靠盲区。
- 测试支撑「至少两个依赖者」把「依赖」当成计数：`cfg(any())` 下的依赖也算（见判法一节）。越位在于把「两个包都用」的语义换成了「两份清单里写了名字」。

### 内部矛盾与循环依赖

- 规则与钩子对放置类的射程不一致（见「误伤」一节）。
- 单元图的角色：「函数单元图，验证包与产品 GPU 包的测试一侧都依赖」（`.claude/gate.d/crate-roles.tsv:14`）登成测试支撑，规则定义「测试支撑（只给测试与装置用的库）」（`.claude/rules/directory-layout.md:19`）。拆包按 F4 的读法是「产品包只是 `Cargo.toml` 多写一行 `[build-dependencies]`」（`records/2026-11-19-产品GPU包测试与E202装置归位.md:91`）：产品包的构建依赖一个只给测试用的包，与角色定义相反，crate-roles 不判「产品包依赖测试支撑包」。另一头的次序依赖：单元图今天的两个依赖者是验证包与要删掉的测试包；测试包删掉的那一次改动里，产品 GPU 包的构建依赖不同时落，support-dependents 就红。两件事被判法绑成必须同一次改动落，正文与调度记录都没写这一条约束（推的，读码与依赖表）。
- 「已经量的」里 code-round-checklist「绿」与第九格：真树上判九格的 0 份，绿与第九格无关（见上一节），正文把它列在措施的证据里。

### 哪一条最脆弱

⑨。九种写法里六种在成功句里不留一处痕迹或只是「crates/ 之外」「同包」的计数涨一点，没有基准比计数；原来那一次的 `#[path]` 加一个 `cfg_attr(all(), …)` 就重演。其次是放置类钩子：label 不扫，而原来那一次的偏离选项就是以 label 出现的。

## 零轮改法（只在我的推理上，没在模型上量过、被攻过零轮）

| 改法 | 修哪几格 | 量过还是推的 |
|---|---|---|
| ⑨ 改判 rustc 的 dep-info：编测试目标时 cargo 在编译目录写 `.d` 文件，列出每个目标真读进来的全部源文件（`mod`、`#[path]`、`include*!` 都在里面），逐个核落点在不在本包目录、在不在放行表 | ⑨ 表里九种写法全部 | 推的；要一次编译，比扫文本重 |
| ⑨ 只做文本时：扫 `crates/` 下全部文件（不只 `*.rs`、进点起头的目录、只跳过 `crates/<包>/target/`），认 `cfg_attr(…, path = …)`、三种定界、原始字符串，行内模块按 Rust 规则算起点，绝对路径与落到 `crates/` 外的判红，`include*` 改名导入判红而不是只计数 | cfg-attr-path、bracket-delimiters、raw-string-path、inline-module-base、absolute-path、non-rs-intermediate、target-directory、hidden-directory；renamed-include-macro 要另认 `use … include_str as` | 推的 |
| crate-roles 按 `cargo metadata` 的目标判：产品包有 bin 或 example 种类的目标就红；验证包的二进制改成放行名单（今天该留的四份），名单外的一律红 | product-examples、product-main-rs、verification-bin-outside-src-bin、verification-hyphen-name、真树上那两份不带号的实验装置 | 推的；对产品二进制的越位要另给登记位 |
| 包的发现改成 `crates/` 下任意深度的 `Cargo.toml` 都要登角色；「验证」角色只许登记表里点名的几个包，新包登验证要另加放行行 | tests-package-nested、tests-package-one-line-library | 推的 |
| 测试支撑的依赖者只数 `[dependencies]`、`[dev-dependencies]`、`[build-dependencies]` 本表，不数 `target.<条件>` 下的 | tests-package-as-support | 推的；真树上 `crates/*/Cargo.toml` 与仓根点名的三个工程的清单里一张 `[target.…]` 表都没有（`grep -n` 退 1，零命中），这一改今天不误伤 |
| 整包滞后一行换成逐条登现有的测试目标与 E202 那一份 | tests-package-whole-lag | 推的 |
| 钩子扫 label；「另有包」改成另认一组与放置有关的说法（目录、`crates/`、「哪个包」「新开」），四个含「包」的放置词不再自带这一条；词表补「放在」「放到」「搬到」「挪到」「留在」「落在」「目录」「crate」 | 三份 misfire、四份 bypass | 推的；补词会多拦，要先在近几轮真弹窗上量多拦几问 |
| 只判八格的名单加一格「名单里的文件都在」；判几格改看提交进仓的日期或标题末尾那个成稿日期；截止日改成规则生效那一天，今天那两份正文进名单 | title-date-cited-earlier、title-date-before-checklist；名单不缩 | 推的 |

## 没打中的形状

- 放置类钩子的「标准：」「偏离：」与引原话：只判前缀与「」里有没有那一节的原话，前缀可以贴在偏离的选项上、原话可以引不相干的一条——文件头已自认「靠人」，不算新打中，没造样本。
- crate-roles 的登记、目录、角色、包数、放行表与滞后表的写法与用不着的行：按 `selftest_cases()` 的 19 格逐格读了判法，没造出漏判；整包行在包删掉后、条目行在条目搬走后都判红。
- crate-roles 的 `experiment-dependency`：今天没有登成实验装置的包，这一格对真树是空判；试了改名依赖（`package =`）与 `workspace = true` 两种写法，判法都按真名认（第 134–149 行），没打中。
- ⑨ 的 `concat!` 拼法：头一段 `env!("CARGO_MANIFEST_DIR")`、全字面量两种都照解；`env!` 带第二个参数、`concat!` 里嵌 `concat!` 会落进「认不出」计数——与表里方括号那一种同类，没另造样本。
- 符号链接：⑨ 不解开符号链接，`crates/<包>/tests/x.rs` 链到别的包就看不见；仓里不放软链接由门禁「软链接」一道判（`.claude/singlefs-ai-sop/rules/command-safety.md`「仓里不放软链接」），不算这一道的漏。
- 写闸第十一道：见判法一节列的五种形状，取样范围是钩子里 `existing_gate_of_fixture`、`landing_target` 两段与写范围表里能写 `.claude/gate.d/**` 的那一行。

## 这条腿自己的限度

- 48 份样本的结局一份都没跑：每个「打中」都只读码坐实到判法那一行，预期退出码是推的。最可能不中的是编译样本里的两种写法：`use std::include_str as grab;`（宏改名导入）与行内模块的 `#[path]` 起点，我按 Rust 参考手册的规则推、没编过；它们编不过，⑨ 表里那两行改判「门禁看不见、但也编不成」，不算绕过。
- 门禁样本根是拷现成的绿样本再加文件，没在真仓上造输入；真仓上的反例只有验证包里两份不带号的实验装置与真树的八格正文，那两条是现查的。
- `cargo metadata` 想先在草稿里试一次，被权限层拒了，没有绕；它的结果只在复跑里出。
- 只读了被判的 20 份里与这两格有关的 11 份（三张 crate-roles 表、两张跨包引用表、crate-roles-check.py、checker-independence-and-sync.sh 的 ⑨ 一段、doc-process-records.sh 的检查单一段、两份钩子、directory-layout.md 那一节）；settings.json、stage-owners.tsv、selftest-registry.tsv、README、verification-reference.md、SKILL.md 只按需 grep。
- 禁读清单里的正推腿产出一份没读。

## 要主 agent 跑的命令

1. `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/gpu-placement-measures-governance-r1-attack -- bash /home/fy5090/code/singlefs/research/prompts/gpu-placement-measures-governance-r1-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs/research/prompts/gpu-placement-measures-governance-r1-snapshot /home/fy5090/.local/share/singlefs/work/gpu-placement-measures-governance-r1-attack-run /home/fy5090/code/singlefs/research/prompts/gpu-placement-measures-governance-r1-opus-model`
   - 在哪起：`--cwd` 是这条腿的草稿目录；运行目录 `…-attack-run` 由包装脚本先删再建，只删这一个。
   - 看什么：末行 `TASK_SERVER_RESULT` 的 `type=` 与 `exit=`；`exit=0` 是 48 份预期全部坐实，`exit=1` 是有对不上的，`task-server/task log <号> --all` 里找 `verdict=mismatch` 的 `name=case` 行与它下面 `    | ` 起头的输出摘录；`name=summary cases=48 mismatches=<数>` 一行给总数；`name=judged_files_check` 一行预期 `lines_not_ok=1`（就是 `break-switch-replay-files.tsv` 那一份 FAILED，别的 19 份 OK）。`exit=2` 是用法或目录不合，看 stderr。
   - 判决要引的数由主 agent 从这一趟日志里取，这份报告里的预期不算读数。

## 没做什么

- 不判正推那几格（拦不拦得住原来那几次、与已有检查的重叠、规则正文）；不替主 agent 采纳；没往任务服务交任务，判决要引的数没复跑（命令在上一节）。
- 没造写闸 Bash 一路与检出钩子的样本（已知、已排下一件）；没量零轮改法。
- 草稿目录里只有一份拼 rerun.sh 中段用的文本 `rerun-middle.txt`，没有仓副本与编译目录，没有要删的。
