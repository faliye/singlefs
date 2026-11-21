# gpu-placement-measures-governance-r1 正推腿报告（Sonnet）

日期 2026-11-19。立场：正推腿，只管判据格 1、4、5；不起任何命令（自证、样本档、门禁、cargo 都没起），要核的写在末尾「要主 agent 跑的命令」。草稿目录 `/home/fy5090/.local/share/singlefs/work/gpu-placement-measures-governance-r1-forward`（四份喂钩子用的弹窗 JSON、`gate-overlap.py --list` 的输出 `overlap-list.txt`）。

## 开工核

- 被判 20 份的开工快照 `sha256sum -c research/prompts/gpu-placement-measures-governance-r1-snapshot/judged-sha256.txt`：19 份 OK，`.claude/gate.d/break-switch-replay-files.tsv` 一份 FAILED（别的会话在开工快照之后改了它的 19 行，文件改于 2026-11-19，现 sha256 `9918657593afa2c13ba17703becbd41fe2354cc90f25a85a4b9ac6c538513f14`）。这份表不在三格的直接射程里，但表里放置类钩子那一行写「九条开关」，钩子文件头现数是十条（`ASK_USER_PLACEMENT_GUARD_BREAK=` 十行），记录里已知这一处（`records/2026-11-19-产品GPU包测试与E202装置归位.md` 第 142 行），不另算打中。
- 三格的被判对象都在 OK 的 19 份里：`directory-layout.md`、`crate-roles-check.py`、`crate-roles*.tsv`、`crate-cross-references-*.tsv`、`checker-independence-and-sync.sh`、`code-tooling.sh`、`ask-user-placement-guard.sh`、`write-guard.sh`、`settings.json`。
- 派发提示的「五条」与记录「原因」表不是同一份五条：记录「原因」表是 ① E202 放进产品包 ② 另开 `-tests` 包 ③ 执行时搬得超出定案、泛称包名 ④ E202 放哪的弹窗没有位置标准一项 ⑤ 测试里 `#[path]` 跨包；派发提示把 ②③ 并成一条，另加「工具员先把样本改进主工作区」——它出自记录「新发现的缺口（治理，不延后）」那一条（第 125 行），不在「原因」表里。下面按派发提示的五条走，原因 ③ 与「没能避免的原因」第 4 条另列一小节。

## 格 1：拦不拦得住原来那几次

结论：**成立，带三处打中**（打中 = 今天的判法判不红）：（一）原因 ② 只在「库为空」这一半形态上红，库里放一行代码就不红；（二）弹窗那条只判格式，「标准：」选项的内容与换了说法的问句不判；（三）样本先进主工作区那条只拦 Write / Edit / MultiEdit，Bash 里写不拦。原因 ③ 的「搬得超出定案」没有检查能判。原因 ⑤ 今天判不红，靠的是滞后表里的一行，不是判法漏了。

每条先写走哪一道、哪一格、哪一行，再写今天的树上它是红还是绿。

### 1.1 产品包里放 `src/bin/e202_…rs`：拦住

- 走法（转述）：提交时的整轮门禁起 `.claude/gate.d/code-tooling.sh`，它的格 crate-roles（`stage_cell crate-roles cell_crate_roles …`，`.claude/gate.d/code-tooling.sh:963`）调 `scripts/gate/crate-roles-check.py`，基准那一版滞后表有就走 `--lag-baseline`（`table) python3 "$STAGE_REPOSITORY/scripts/gate/crate-roles-check.py" "$ROOT" --lag-baseline "$crate_roles_base" ;;`，`.claude/gate.d/code-tooling.sh:5758`）。
- 产品包 `singlefs-background-gpu` 登的是「产品」（`.claude/gate.d/crate-roles.tsv:9`，转述）。判红的行：「if role == PRODUCT_ROLE and (binaries or declared_binaries):」（`scripts/gate/crate-roles-check.py:356`）报 product-binary；「if role != EXPERIMENT_ROLE:」（`scripts/gate/crate-roles-check.py:359`）里 `e<数字>_` 起头的 `src/bin/` 条目报 experiment-binary。两条都进 `violations`，只有包目录在滞后表里（`scripts/gate/crate-roles-check.py:377` 的「if directory in lag_paths:」）或条目在放行表 / 滞后表里才不报；产品包不在任何一张表里，所以红。
- 今天的树（现查，`ls crates/singlefs-background-gpu/src/bin` 不存在）：产品包没有 `src/bin/`，格是绿的；E202 现住 `crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/main.rs`，那个包整包在滞后表里（1.2 写）。
- 样本：`.claude/gate.d/fixtures/code-tooling.sh/crate-roles-product-experiment-binary-red/expect` 的 want 两行是「✗ product-binary：crates/p-gpu（产品）有二进制：src/bin/e1_x.rs」与「✗ experiment-binary：…」，与读码一致（样本档由主 agent 跑，背景里已量「判得都对」）。
- 漏的（判法自己写明）：「管不到的：角色登得对不对；名字不以 e<数字>_ 起头的实验装置二进制；src/main.rs 起的二进制；包里放的测试测的是不是这个包。」（`scripts/gate/crate-roles-check.py:41`）。产品包的 `src/bin/` 与 `[[bin]]` 不论名字一概红，只有 `src/main.rs` 逃得过；E202 形态（`e<数字>_`、`src/bin/`）两条都红。
- 什么现象会推翻「拦住」：在草稿副本里给产品包加一份 `src/bin/e202_x.rs` 且不动三张表，门禁 code-tooling 的 crate-roles 格仍退 0。

### 1.2 另开一个空库加十个测试目标的 `-tests` 包：只拦「库为空」形态

- 走法：同一格 crate-roles。新包没登记：`for name in sorted(set(packages) - set(rows_by_name)):`（`scripts/gate/crate-roles-check.py:323`）报 registered。登记成验证或产品：「if role != SUPPORT_ROLE and targets and library_is_empty(absolute, manifest):」（`scripts/gate/crate-roles-check.py:366`）报 empty-library-tests；`library_is_empty` 去掉注释与内层属性后看有没有字（`return not text.strip()`，`scripts/gate/crate-roles-check.py:210`）。登记成测试支撑：「if len(users) < SUPPORT_DEPENDENTS_MINIMUM:」（`scripts/gate/crate-roles-check.py:371`）要至少 2 个依赖者，`-tests` 包没人依赖，报 support-dependents。三条路都红。
- 今天的树（现查）：`crates/singlefs-background-gpu-tests/src/lib.rs` 4 行，两行 `//!`、一行 `#![deny(unsafe_code)]`、一行空行，去掉后为空；`tests/` 下 13 个 `.rs` 目标、`Cargo.toml` 另有 5 条 `[[test]]`（`test_targets` 两处都数，转述：同一目标会数两遍，只影响红行里写的个数）。包登的角色是验证（`.claude/gate.d/crate-roles.tsv:13`「singlefs-background-gpu-tests	验证」），所以若不是整包滞后，empty-library-tests 与 experiment-binary（E202）都会红。今天绿靠 `.claude/gate.d/crate-roles-lag.tsv` 第 4 行整包一行，判法在 `scripts/gate/crate-roles-check.py:377` 整包跳过。
- 滞后行不会永远留着：包按角色一条都不犯了，滞后表那一行判红（`scripts/gate/crate-roles-check.py:397-399` 的 lag-table 分支，转述），拆包落地、包删掉时同一次改动要删。
- **打中（变体，推的，读码）**：规则写的是一般句「不为测试另开包」，判的只是「库为空」。在 `src/lib.rs` 里放一行非注释的代码，`library_is_empty` 交 False，empty-library-tests 不判；该包登验证、同时依赖验证包与产品 GPU 包，门禁 ⑧ 也不看它（⑧ 只判两个点名的包：「⑧ 验证包 singlefs-verification 与产品 GPU 包 singlefs-background-gpu：两边的传递闭包」，`.claude/gate.d/checker-independence-and-sync.sh:38`）。所以「另开一个包装测试」的原因 ② 在一行代码的伪装下不红。判法文件头自己的射程也只写到「包里放的测试测的是不是这个包」不管（`scripts/gate/crate-roles-check.py:41`）。要核：在草稿副本里给 `-tests` 型样本包的 `lib.rs` 加一行 `pub fn marker() {}`，crate-roles 格退 0。这一条归攻方腿造样本，这里只报形态。
- 什么现象会推翻「库为空形态拦住」：现有样本 `code-tooling.sh/crate-roles-empty-library-tests-red` 在关着开关时不红在 empty-library-tests（它的 expect 要求「✗ empty-library-tests：crates/p-x-tests（验证）库是空的、却有 2 个测试目标」）。

### 1.3 原因 ③：执行时搬得超出用户定案、包名起成泛称 `-tests`：没有检查能判，推不出「拦住」

- 「搬的范围超出定案」是执行与定案之间的差，定案在弹窗答复与记录里，仓里没有机读的位置，没有一道门禁、钩子读它（读码：三张新表与 ⑨ 的两张表只登包与路径，`crate-roles-check.py` 的 `read_table` 只核四列与理由字数，`scripts/gate/crate-roles-check.py:103` 的 `REASON_MINIMUM = 4`）。泛称包名同样没有格判名字。
- 它的结构后果（十个目标搬进一个没有库的新包）与 1.2 是同一个形态，空库那一半由 empty-library-tests 判；非空库那一半 1.2 已列为打中。
- 判：**推不出**（不是检查能管的事）。归记录里的措施 M4 检查单第九格「放的位置」那一路（审核 `crates/` 代码轮的人看放置），那一格只判正文里有没有第九格，不判内容对不对（背景材料：「截止日 2026-11-20 之前的正文只判八格」，转述）。

### 1.4 测试目标 `#[path]` 引验证包 `tests/` 下的模块：拦住，今天绿靠一行滞后表

- 走法：门禁 `.claude/gate.d/checker-independence-and-sync.sh` 的格 checker-implementation-disjoint 的 ⑨。`INCLUDE_FORM` 认 `#[path = "…"]`、`include!`、`include_str!`、`include_bytes!` 与它们的 `concat!`（`.claude/gate.d/checker-independence-and-sync.sh:794` 起）。字面路径按所在文件的目录解析，规整后落在另一个 `crates/<包>/` 下、后缀 `.rs` 或 `.wgsl` 的记进 `cross_found`（「cross_found.setdefault((path, target), []).append(line_number)」，`.claude/gate.d/checker-independence-and-sync.sh:934`）；不在放行表也不在滞后表的列进 `cross_unlisted`（「cross_unlisted = sorted(key for key in cross_found if key not in cross_allowed_keys and key not in cross_lag_keys)」，`.claude/gate.d/checker-independence-and-sync.sh:941`），「if cross_unlisted:」（`.claude/gate.d/checker-independence-and-sync.sh:943`）起红行。
- 原事件的那一行：「#[path = "../../singlefs-verification/tests/common_background_gpu_compute/mod.rs"]」（`crates/singlefs-background-gpu-tests/tests/background_gpu_compute_agrees_with_the_cpu_backed_test_double.rs:11`）。手算解析：所在目录 `crates/singlefs-background-gpu-tests/tests`，两个 `..` 之后是 `crates/`，目标 `crates/singlefs-verification/tests/common_background_gpu_compute/mod.rs`，所在包 `singlefs-background-gpu-tests`、目标包 `singlefs-verification`，不同、后缀 `.rs`，所以进 `cross_found`。
- 今天不红：`.claude/gate.d/crate-cross-references-lag.tsv` 第 3 行就是这一对（所在文件与目标与上面逐字相同，转述；表里是制表符分隔，不引原文）。这是有意的存量放行，不是判法漏了；那一行在引用改掉之后要删（`cross_table_problems` 判「已经没有字面路径引…」，`.claude/gate.d/checker-independence-and-sync.sh` 的 ⑨ 段，转述）。
- 滞后表只缩不涨的比较：`HEAD` 里还没有这张表（`git cat-file -e HEAD:.claude/gate.d/crate-cross-references-lag.tsv` 报「exists on disk, but not in 'HEAD'」；`crate-roles-lag.tsv`、`crate-roles.tsv` 同样），所以第一次提交时基准是「没有这张表」，「if cross_lag_state == "table":」（`.claude/gate.d/checker-independence-and-sync.sh:961`）不进，涨没比；提交之后成基准才管。crate-roles 那边同理：基准没有时走 `--lag-baseline-absent`（`*) python3 "$STAGE_REPOSITORY/scripts/gate/crate-roles-check.py" "$ROOT" --lag-baseline-absent "基准里还没有这张表，或不在 git 仓里" ;;`，`.claude/gate.d/code-tooling.sh:5760`），成功句写明没比。这一次提交里放进两张滞后表的 24 行（crate-roles-lag 22 行、cross-references-lag 2 行，量的：`grep -c`/`wc -l` 的行数见末尾命令）不受涨检查约束，这一点是不可避免的首次提交形态，但也意味着这 24 行是否都该进表，只有这一轮的人审得了。
- 样本：`checker-implementation-disjoint-cross-package-source-red` 的 expect 要求「crates/singlefs-background-gpu/tests/x.rs:1  引 crates/singlefs-verification/tests/c/mod.rs」，形态与原事件相同（所在包方向反过来，判法对称）。
- 什么现象会推翻「⑨ 拦住」：把 lag 表第 3 行临时删掉（草稿副本）后，checker-independence-and-sync 的 checker-implementation-disjoint 仍退 0。

### 1.5 弹窗问「E202 放哪」只列偏离标准的选项：原话形态拦住，两个残余打中

- 走法：钩子 `.claude/hooks/ask-user-placement-guard.sh`，在 `.claude/settings.json` 里挂在 `AskUserQuestion` 的 matcher 下（「"matcher": "AskUserQuestion",」，`.claude/settings.json:113`；命令行在 117、121 行，转述）。
- 原弹窗原文不在仓里：只有答复「选「整份搬进测试包」」与标题「E202 放哪」（「用户定案（E202 放哪，弹窗）」，`records/2026-11-11-GPU两段与按需领卡核查.md:276`；答复选项是整份搬进测试包）。所以按问句「E202 放哪？」、选项说明含「测试包」推：放置类词表有「放哪」「测试包」（「PLACEMENT_WORDS = ("放哪", "放进", "住进", "搬进", "归位", "新开包", "测试包", "装置包", "实验包")」，`.claude/hooks/ask-user-placement-guard.sh:70`），另要的「包」在选项说明里（「PACKAGE_WORDS = ("包", "crates/")」，`.claude/hooks/ask-user-placement-guard.sh:71`；「return has_placement_word and has_package_word」，`.claude/hooks/ask-user-placement-guard.sh:116`）。判红的行三处：问句没点名规则文件与小节「if not (RULE_FILE_NAME in text and SECTION_TITLE in text) and not broken("rule-citation-unchecked"):」（`.claude/hooks/ask-user-placement-guard.sh:128`，种类 citation）；选项说明没以「标准：」「偏离：」起头「elif not broken("option-prefix-unchecked"):」（`.claude/hooks/ask-user-placement-guard.sh:140`，种类 prefix）；没有「标准：」选项「if standard_count == 0 and not broken("standard-option-unchecked"):」（`.claude/hooks/ask-user-placement-guard.sh:142`，种类 standard）。有一条就退 2（`decide`，`.claude/hooks/ask-user-placement-guard.sh:156-169`，转述）。
- 「只列偏离标准的选项」本身：全带「偏离：」写了规则原话、问句也点名了，仍红在 standard（142 行）。这一形态拦住，是这条措施的核心。预测：`popup-deviation-only-formatted.json` 退 2、只记 standard。
- **打中 A（假「标准：」）**：选项以「标准：」起头、内容其实是偏离（例：「标准：整份搬到测试包的 src/bin/」），三项格式都过，退 0。钩子头自述「「标准：」那个选项是不是真照标准、「偏离：」引的那一条是不是它真偏离的那一条，靠人」（`.claude/hooks/ask-user-placement-guard.sh:52`）。预测：`popup-fake-standard.json` 退 0。
- **打中 B（换说法）**：问句与选项说明里没有词表九个词、也没有「包」的放置类问题不认（例：「E202 的装置应该落在哪个目录？」，选项「整份挪到测试子目录」）。头注自述「管不到的：换了说法的放置类问题」（`.claude/hooks/ask-user-placement-guard.sh:52`）。预测：`popup-reworded-no-placement-word.json` 退 0。原话「E202 放哪」本身在词表内，所以 B 是对今后的漏，不是对原事件的漏。
- 什么现象会推翻「原话形态拦住」：`popup-original-e202.json` 喂钩子不退 2。

### 1.6 工具员先把样本改进主工作区：Write / Edit / MultiEdit 拦住，Bash 没拦，原事件用哪种工具推不出

- 走法：写闸 `.claude/hooks/write-guard.sh` 第十一道。写闸挂在 `Write|Edit|MultiEdit`（「"matcher": "Write|Edit|MultiEdit",」，`.claude/settings.json:104`）。只管 `tooling-writer`（「SWAP_ONLY_AGENTS = ("tooling-writer",)」，`.claude/hooks/write-guard.sh:853`）。`existing_gate_of_fixture`（`.claude/hooks/write-guard.sh:907` 起）认「`.claude/gate.d/fixtures/<门禁>/` 下、且 `.claude/gate.d/<门禁>` 是已存在的普通文件」，改已有、新建都算；`decide_script_swap_only` 在它命中时退 2（「return 2, gate_fixture_refusal(agent_type, relative, gate)」，`.claude/hooks/write-guard.sh:972`）。
- 原事件：措施工具员在主工作区改了 `.claude/gate.d/fixtures/doc-process-records.sh/` 下 code-round-checklist-green、-red、-empty-red 三份已有样本，新建了 -placement-red、-placement-undated-red 两份（`records/2026-11-19-产品GPU包测试与E202装置归位.md:124` 转述）；`doc-process-records.sh` 是已存在的门禁（`ls .claude/gate.d/doc-process-records.sh` 在），所以经 Write / Edit / MultiEdit 写这五份，今天都退 2。
- **打中（Bash）**：写闸头注自述「Bash 里往样本档目录写（cp、重定向）这道闸看不见」（`.claude/hooks/write-guard.sh:110`）；检出钩子 `bash-command-detector.sh` 里没有这一半（`grep -n 'fixtures' .claude/hooks/bash-command-detector.sh` 只命中一个自证用例，`.claude/hooks/bash-command-detector.sh:9131`）。记录把「检出钩子 ⑭ 加同一半」排给下一件工具员（第 142 行），今天没落。原事件记录没写用的是 Write / Edit 还是 Bash，所以「原来那一次今天会不会红」推不出：Write / Edit 会红，Bash 不会。
- 旁：`tooling-writer` 的定义第 3 步仍写「`fixtures/` 下的样本照常在主工作区改」（`.claude/agents/tooling-writer.md:31`），与写闸现状相反，记录已知待落（第 142 行），不另算。
- 什么现象会推翻「Write / Edit 拦住」：喂写闸一条 `agent_type=tooling-writer`、`tool_name=Edit`、目标 `.claude/gate.d/fixtures/doc-process-records.sh/code-round-checklist-green/expect` 的 JSON 不退 2（写闸的 `--selftest` 已有「整份换上:工具员 Write 已有门禁的一份样本拒」一格，头注 `.claude/hooks/write-guard.sh:297`，由主 agent 跑）。

### 1.7 「没能避免的原因」第 4 条（检查单没有放置那一格）：不在五条里，顺带

- 门禁 doc-process-records 的 code-round-checklist 格加第九格「放的位置」，截止日之前的正文只判八格（背景材料转述）。它判正文里有没有第九格，不判放得对不对（审核放置靠人看）。判：**推不出**它会不会拦住原事件，那两轮代码轮当时的正文是八格检查单。

### 1.8 格 1 判定

| 原因 | 今天走哪一格 | 判 | 红 / 绿 | 打中 |
|---|---|---|---|---|
| ① 产品包放 `src/bin/e202_…rs` | code-tooling 格 crate-roles：`crate-roles-check.py:356`、`:359` | 拦住 | 产品包红；今天树上绿（产品包无 `src/bin/`） | `src/main.rs` 起的二进制管不到（`:41`） |
| ② 另开空库加测试目标的包 | 同格：`crate-roles-check.py:366`（空库）、`:371`（测试支撑依赖者） | 只拦空库形态 | 无滞后行就红；今天靠整包滞后行绿 | 库里放一行代码就不红（推的） |
| ③ 搬得超出定案、泛称包名 | 无 | 推不出 | — | 没有检查能判 |
| ⑤ 测试 `#[path]` 引验证包 | 门禁 checker-independence-and-sync ⑨：`:794`、`:934`、`:941`、`:943` | 拦住 | 无滞后行就红；今天靠 `crate-cross-references-lag.tsv` 第 3 行绿 | 无（动态拼出的路径、宏里嵌宏认不出，归攻方腿） |
| ④ 弹窗只列偏离选项 | 钩子 `ask-user-placement-guard.sh`：`:128`、`:140`、`:142` | 原话形态拦住 | 退 2（待主 agent 喂） | 假「标准：」、换说法（`:52` 自述） |
| 样本先进主工作区 | 写闸第十一道：`write-guard.sh:972` | Write / Edit 拦住，Bash 没拦 | tooling-writer 红 | Bash 里 `cp`、重定向（`:110` 自述） |

「什么现象会推翻整格」：上表任何一行的「红」被主 agent 现跑的样本或喂入推翻（命令见末尾）。

## 格 4：与已有检查的重叠

输入：`python3 .claude/singlefs-ai-sop/scripts/gate-overlap.py --list` 的输出（只读列表，退出 0，存在草稿目录 `overlap-list.txt`，末行「共 69 份：门禁 44、钩子 25」，量的）。

结论：**部分成立**。（一）没有已有的门禁或钩子判同一条判据；（二）新钩子的 `# gate-similar:` 两条与一条 `gate-overlap:copy-kept` 写得对；（三）有三处「同输入、不同判」的重叠，没有一处够并进已有那份，但其中一处该在文件头点名；（四）新判法脚本 `scripts/gate/crate-roles-check.py` 不在 `--list` 的清单里，不要求写 gate-similar。

### 4.1 `--list` 的范围（读列表得到）

- `--list` 的门禁只列 `.claude/gate.d/*.sh` 与上游包 `scripts/` 下的检查（44 份），钩子 25 份；`scripts/gate/*.py` 这类被门禁格调用的判法脚本不在清单里（列表里没有 `crate-roles-check.py`、`directory-layout-check.py`、`crate-sources-reachable.py`）。所以 crate-roles 判法没有「新文件要写 gate-similar」的要求，它挂在已有门禁 code-tooling 的新格上；⑨ 是已有门禁 checker-independence-and-sync 里的新条，同样不是新文件。新文件里要写 gate-similar 的只有 `.claude/hooks/ask-user-placement-guard.sh` 一份（以及 `.claude/gate.d/` 下没有新增门禁阶段，`git status` 里新增的是表与样本）。
- 判同一条判据的已有检查，逐类查：
  - 放置类弹窗：列表里挂 `PreToolUse[AskUserQuestion]` 的只有 ask-user-claim-guard.sh 与新钩子两份（`overlap-list.txt` 第 56、57 行）。它的判据是断言词有没有出处（转述；「词表只认这九个」，`.claude/agent-common.md:114`），与放置词表、选项前缀不相同。
  - 工具员写主工作区的路径：写闸第十一道本来就在 write-guard.sh 里，新一半是同一道里加一个目录条件（`existing_gate_of_fixture`），不是新钩子；`.claude/hooks/agent-write-scope.tsv` 给 tooling-writer 的写范围是静态路径（第 34 行 `.claude/gate.d/**`，含 `fixtures`），表达不了「门禁阶段已存在才拦」，所以不能并进那张表（转述）。
  - 产品包二进制 / `e<数字>_` 位置：没有别的格判位置。`.claude/gate.d/code-source-discipline.sh` 也用「crates/*/src/bin 下以 e<数字>_ 开头的」认实验二进制（`.claude/gate.d/code-source-discipline.sh:41`），但它判的是变异表与绝对值断言纪律，认法与 crate-roles 的 `EXPERIMENT_BINARY_NAME` 一致，是对口径的一致而不是重叠；装置搬进实验包之后这一格仍判它们（不受搬迁影响）。
  - 崩溃枚举用例放哪个包：`scripts/crash/crash-case-check.py` 的 placement 判「写在别的包（singlefs-core 这类）里判红」（`scripts/crash/crash-case-check.py:26`）。它判的是一类测试住验证包，方向与 crate-roles「一个包的测试放它自己的 `tests/`」相反，见格 5 的冲突一条；不是同一条判据。
  - 跨包源码引用：`scripts/gate/crate-sources-reachable.py`（门禁 code-source-discipline 的一格）也解析 `#[path]`、`include!`、`include_str!`、`include_bytes!`，但判的是「每份 `.rs` 有着落」，说明里还写「引到的文件在给的目录之外也照着读（跨 crate 的共用模块）」（`scripts/gate/crate-sources-reachable.py:20`），不判跨包。与 ⑨ 同输入不同判。

### 4.2 三处同输入、不同判的重叠

1. ⑨ 与 `crate-sources-reachable.py`：两处各写了一份认 `#[path]` 与 `include*!` 字面路径的解析（⑨ 在 `.claude/gate.d/checker-independence-and-sync.sh:794`，另一份在 `scripts/gate/crate-sources-reachable.py` 的 ③）。判不同（跨包 vs 有着落），不并。建议：`checker-independence-and-sync.sh` 文件头 gate-similar 已有七条（第 7 至 13 行），没有一条点名 code-source-discipline 的 crate-sources-reachable 格；那一格解析同一批宏，该补一句「它判有着落、这里判跨包」。这是建议，不是门禁红的事（该文件不是新文件）。
2. `crate-roles-check.py` 与 ⑧ 各自建一份「依赖谁」的表：前者 `def workspace_renames(root_manifest):`（`scripts/gate/crate-roles-check.py:128`），后者 `def workspace_renames():`（`.claude/gate.d/checker-independence-and-sync.sh:316`），读同一处根 `Cargo.toml` 的 `[workspace.dependencies]` 的 `package =`，一个用 `tomllib`、一个用正则。判不同（角色与依赖者数、实验包依赖 vs 验证包与产品 GPU 包两边闭包），不并；风险是两份认法将来分叉（一边认 `[target.…]`、`workspace = true` 的写法，另一边没跟上）。写进「推翻条件」：给一个 `workspace = true` 带 `package =` 的依赖喂两边，两边认出的真名不同则成立。
3. crate-roles 的 `without_comments`（`scripts/gate/crate-roles-check.py:152`）与 `scripts/crash/crash-flow-coverage.py:451` 的 `def without_comments(text):`、⑨ 的 `blanked_rust`（`.claude/gate.d/checker-independence-and-sync.sh:803`）同类：Rust 去注释函数仓里已有三份。门禁 gate-overlap 的整段抄检测对 `scripts/gate/*.py` 不扫（不在 `--list`），这一条只记事实，不判。

### 4.3 新钩子 `# gate-similar:` 写得对不对

- 第 6 行点名 ask-user-claim-guard.sh（「ask-user-claim-guard.sh 同挂 PreToolUse[AskUserQuestion]、同读每一问的问句与选项说明」，`.claude/hooks/ask-user-placement-guard.sh:6`）：名字在 `--list`（第 56 行），触发点对；「词表、判据与出路没有一条相同」对（它的词表是九个断言词）。理由说了为什么不并（要同时管两份规则、两套弄坏开关与自证）。对。
- 第 7 行点名 runner-dispatch-guard.sh（`.claude/hooks/ask-user-placement-guard.sh:7`）：在 `--list` 里触发点是 `PreToolUse[Agent|Task]`（`overlap-list.txt` 第 66 行），与写的「挂 Agent|Task」一致。对。
- 第 9 行 `gate-overlap:copy-kept ask-user-claim-guard.sh`（`.claude/hooks/ask-user-placement-guard.sh:9`）：被点名的两段确实在两份里各有一份——`unjudged_pass`（`.claude/hooks/ask-user-claim-guard.sh:330`、本钩子 `:301`）与 `main`。理由写了「不带判据」与「抽成共用模块要改 ask-user-claim-guard.sh」。对。
- 缺的：没有。write-guard.sh 判写入路径与内容、不判给人看的弹窗文字，不点名它不算漏。
- 钩子头另一处与登记：`# hook-events: PreToolUse:AskUserQuestion`（`.claude/hooks/ask-user-placement-guard.sh:5`）与 `.claude/settings.json:113` 的 matcher 一致；`.claude/hooks/selftest-registry.tsv:203` 登了自证（`.claude/hooks/ask-user-placement-guard.sh	selftest`）。`.claude/agent-common.md:114` 的「只管主 agent 用的工具的三道」还没提这一道，记录里已列待落（第 127 行），不算本格打中。

### 4.4 格 4 判定

「成立」的部分：没有判同一条判据的已有门禁或钩子（上面逐类查过，范围限于 `--list` 的 69 份与我读过的 `scripts/gate/`、`scripts/crash/` 里的几份判法脚本，没有扫全仓）；gate-similar 三行对。「不成立」的部分：⑨ 所在门禁的 gate-similar 缺 crate-sources-reachable 一句（建议补，非红）。
什么现象会推翻：主 agent 跑 `gate-overlap.py`（不带 `--list`）对这次改动判出 ask-user-placement-guard.sh 有「整段抄」或「标识符重合超 0.5」而 gate-similar 没点名的那一份。

## 格 5：规则正文与 rules-discipline、与 verification.md 验证包那一行

两问分开答：5a 新一节合不合 `rules-discipline.md`——**成立（合）**，带两个小项；5b 与 `verification.md`「定义与名字」验证包那一行冲不冲突——**冲突成立**；另有两处同类的不一致（5.3、5.4）一并列出，其中 5.3 成立、5.4 里「实验装置包在哪」推不出。

### 5.1 5a：与 `.claude/singlefs-ai-sop/rules/rules-discipline.md` 合不合

读法：规则文件整份读过。新一节是 `.claude/rules/directory-layout.md` 第 17 至 25 行「crates/：各包放什么」，与第 32 至 44 行「门禁管哪一半」里新加的表与「靠 review 的」。对着 `rules-discipline.md` 逐条：

- `rules-discipline.md@1. 正文只写四样`：写「怎么做：步骤、次序、判据、阈值」，不写「为什么这么定」「这条怎么来的」「当时是什么样」。新一节七条都是做什么与不做什么（第 19 至 25 行）。现查：对第 17 至 45 行 grep「因为|所以|为了|原因|此前|原先|后来|曾经|已改|2026|records/|CHANGELOG|本节|本表|本条|上述|如下|下表|见上」，`rc=1`，一处都没命中（量的，输出见末尾命令一节同名命令）。合。
- `rules-discipline.md@4. 指向历史的链接要带劝阻句`：新一节没有指向 `records/`、`CHANGELOG.md` 或 kb 历史的链接（同一次 grep，`records/`、`CHANGELOG` 零命中）。合。
- `rules-discipline.md@7. 正文不许用位置指代，也不许自称`：这一条判「自称只判指文档结构的那几个词」。新一节没有「本节」「本表」「本条」「上述」之类。**小项一**：第 42 行写「放置类弹窗的问句点名这份规则与小节」（`.claude/rules/directory-layout.md:42`），「这份规则」是指本文件的自称，字面不在那一条列的五个词里，门禁 doc-lint 判不判不确定（推的）；改成「点名 `directory-layout.md`「crates/：各包放什么」」就没有自称，也与钩子判的字面（`RULE_FILE_NAME`、`SECTION_TITLE`）一致。
- `rules-discipline.md@6. 立一条新规则之前，先问它能不能变成会失败的检查`：新一节把能按字面判的列成表（第 38 至 42 行），判不了的列进「靠 review 的：」（`.claude/rules/directory-layout.md:44`）。**小项二**：第 20 行有两句没有任何格判、也没进「靠 review 的」那一段：「只给一个包的测试用的支撑代码写成那个包的 `tests/common_<名>/mod.rs`」，与「不为测试另开包」在库不为空时的那一半（1.2 的打中）。表第 40 行把空库那一半写成「库为空而有测试目标的包只许是测试支撑」，是准确的；缺的是「靠 review」那一段没写这两句。
- `rules-discipline.md@8. 正文不写某一个使用者的东西`：门禁表头写明只判本包自己的规则（「只判本包自己的规则」，同一份文件「门禁管哪一半」），项目本地的 `.claude/rules/directory-layout.md` 可以写项目路径。不判。
- 其余（第 3、5 条：不留搬出的论证、不留历史节）：新一节没有历史节、没有「原为」痕迹，合。

什么现象会推翻 5a：主 agent 跑门禁 rules-lint（`.claude/singlefs-ai-sop/scripts/rules-lint.sh` 的项目本地扫，`.claude/rules/*.md` 在扫的范围里；`.claude/rules-lint-exclude` 文件不存在，`ls` 报 No such file，没有排除项，量的）对这份文件判红，或 doc-lint 判第 42 行自称（推翻小项一的「不确定」）。

### 5.2 5b：与 `.claude/rules/verification.md`「定义与名字」验证包那一行冲突（整行并排）

```
.claude/rules/verification.md:9
| **验证包** | `crates/singlefs-verification` | 单元测试、集成测试、它们用的脚手架库与实验、真设备装置二进制（`src/bin/`），连同各道门禁判的东西 | 照「一个门禁：两种跑法」那张表 |

.claude/rules/directory-layout.md:22
- 验证包只装测试、测试的脚手架，与门禁、崩溃入口自己起的工具二进制。

.claude/rules/directory-layout.md:21
- 实验装置二进制只放实验装置包；没有包依赖实验装置包。

.claude/gate.d/crate-roles.tsv:12
singlefs-verification	验证	crates/singlefs-verification	验证包：测试、测试的脚手架与门禁、崩溃入口自己起的工具
```

- 冲突：`verification.md` 第 9 行说验证包装「实验」与「真设备装置二进制（`src/bin/`）」；新规则第 21、22 行说装置二进制只放实验装置包、验证包只装门禁与崩溃入口自己起的工具二进制；登记表第 12 行的理由与新规则一致。一个读者按 `verification.md`「定义与名字」办，会往验证包 `src/bin/` 里再放装置，随后 crate-roles 对 `e<数字>_` 条目判红（推的）。
- 同一句话在 kb 里更重：`.claude/kb/decisions/13-验证路线.md:405` 的 D13（验证路线）已定项 15 定案写「实验与真设备装置二进制，以及它们的全部用例」住在验证包；用户第二轮弹窗定了「改 D13，门禁工具留验证包」（`records/2026-11-19-产品GPU包测试与E202装置归位.md:112`，转述），今天 D13 工作区里那一句还在（上面 `grep -nF` 命中第 405 行）。
- 记录里的安排是验证包那一行「随着改」（`records/2026-11-19-产品GPU包测试与E202装置归位.md:94`），落在拆包实现一簇；没有任何检查把「这一轮已经在规则里写了新位置」与「`verification.md` 同名一行还是旧话」绑在一起，今天树上两句话同时在。
- 过渡期放行有记录可查：验证包 `src/bin/` 的 21 个 `e<数字>_` 条目登在滞后表（`.claude/gate.d/crate-roles-lag.tsv` 第 5 至 25 行，`grep -vc '^#'` 得 22 行，其中一行是整包），`e165_replay_bounded_enumeration.rs` 登在放行表。这解决门禁判红，不解决规则正文互相矛盾。
- 另有一处不在门禁视线内：`src/bin/` 里不以 `e<数字>_` 起头的 5 份（`crash_library_reader`、`deletion_write_dump`、`new_pool_file_creation_device_log_check`、`new_pool_file_creation_on_device`、`new_pool_file_creation_region_bytes`，`ls crates/singlefs-verification/src/bin | grep -v '^e[0-9]'` 量的）。记录第 112 行定 4 份留验证包（含 `e165`）、其余进实验包，但 `deletion_write_dump` 与 `new_pool_file_creation_region_bytes` 是 E172、E142 装置与 `replay.sh` 起的（记录第 102 行，转述）、名字不是 `e<数字>_`，`crate-roles-check.py` 看不见（「管不到的：…名字不以 e<数字>_ 起头的实验装置二进制…」，`scripts/gate/crate-roles-check.py:41`），新规则第 22 行又不许验证包装它们：搬迁之后若漏搬，没有任何格会红。
- 什么现象会推翻「冲突成立」：`verification.md` 第 9 行在这一轮的树上已改成不含「真设备装置二进制（`src/bin/`）」「实验」（现查它此刻还在，`.claude/rules/verification.md:9`）；或主 agent 判这是预期的过渡态并另有登记位。

### 5.3 同类不一致一：第 20 行「一个包的集成测试放它自己的 `tests/`」与验证包、崩溃枚举用例的放法

```
.claude/rules/directory-layout.md:20
- 一个包的集成测试放它自己的 `tests/`，单元测试放 `src/` 里的 `#[cfg(test)]`；不为测试另开包。只给一个包的测试用的支撑代码写成那个包的 `tests/common_<名>/mod.rs`；至少两个包都依赖的才单开测试支撑包。

scripts/crash/crash-case-check.py:26
     写在别的包（singlefs-core 这类）里判红，那会让 cargo test 那个包的人跑到全量枚举；
```

- 冲突点（读文本）：第 20 行没有给验证角色留例外。验证包本身就是「为测试另开的包」，`crates/singlefs-core`、`crates/singlefs-format`、`crates/singlefs-checker`、`crates/singlefs-reference-filesystem` 四个包都没有 `tests/`（量的：逐包 `ls` 的 `tests=0`），它们的集成测试住在 `crates/singlefs-verification/tests/`（518 项，量的）；崩溃枚举用例的判法更进一步，在别的包里写了判红（上面 `crash-case-check.py:26`，同一条登在 `.claude/rules/verification-reference.md:151` 的表里）。按第 20 行字面，这些测试都该搬回各包的 `tests/`；按 D13 已定项 15 与 crash-case-check，它们该留在验证包。
- 门禁这边不红（empty-library-tests 只判库为空的包，`scripts/gate/crate-roles-check.py:366` 对验证包的非空库不触发），所以是文本不一致，不是门禁误伤；但原因 ② 的实现者读的正是这种句子。
- 判：**成立**（文本冲突，推的，没有现跑可核）。出路：第 20 行加一句例外，写明验证包的 `tests/` 装崩溃枚举用例与对别的包的验证测试，指到 `verification.md`「定义与名字」，只写怎么做。
- 什么现象会推翻：规则里别处（`directory-layout.md` 或 `verification.md`）已有一句把「验证角色的包」从第 20 行里摘出去（现查第 17 至 25 行没有，上面已贴全文）。

### 5.4 同类不一致二：登记表与角色定义、实验装置包在哪

- 登记表第 13 行把测产品 GPU 包的 `-tests` 包登成验证（「singlefs-background-gpu-tests	验证」，`.claude/gate.d/crate-roles.tsv:13`），而同一张表头与规则第 19 行都把验证角色定义成「验证（验证包、池级 checker、RefFS）」（`.claude/gate.d/crate-roles.tsv:3`）：它不是这三个里的任何一个。四个角色里没有一个对得上它（测试支撑要 ≥2 个依赖者，它没有）。今天靠整包滞后行遮住（`scripts/gate/crate-roles-check.py:377`），拆包删掉它就没有这个问题。判：登记与角色定义不一致，但是过渡态、有滞后行与记录两处指向，**不算打中**。
- 实验装置角色今天一个包都没有：`awk -F'\t' '!/^#/ && $2=="实验装置"' .claude/gate.d/crate-roles.tsv | wc -l` 得 0（量的），所以「没有包依赖实验装置包」的判法在今天的树上是空判（`experiment_packages = {name for name, role in roles.items() if role == EXPERIMENT_ROLE}`，`scripts/gate/crate-roles-check.py:385`，集合为空，后面的循环一次都不进）；样本里它能红，真仓上要等实验包建出来才有对象。
- **推不出**：规则第 21 行「实验装置二进制只放实验装置包」没说实验装置包在哪里。`scripts/experiment-apparatus/Cargo.toml` 是仓里已有的一个 cargo 工程，里面 178 条 `[[bin]]`（`grep -c '^\[\[bin\]\]'` 量的），在根 `Cargo.toml` 的 `exclude` 里（「exclude = ["scripts/experiment-apparatus", "prune-server"]」，`Cargo.toml:20`），`crate-roles-check.py` 只读 `crates/*/Cargo.toml`（`scripts/gate/crate-roles-check.py:277`）、不登记它，也不扫它的 `e<数字>_`。它算不算「实验装置包」规则没说；用户第二个定案是在 `crates/` 下新开实验包（`records/2026-11-19-产品GPU包测试与E202装置归位.md:90` 转述用户原话「按照和 E185 这些装置放在一起」），两处会并存。要主 agent 定：规则第 21 行写明实验装置包是 `crates/` 下登的那一个，还是也包括 `scripts/experiment-apparatus`。
- 什么现象会推翻：登记表里有一行实验装置角色，或规则第 17 至 25 行写了 `scripts/experiment-apparatus` 的位置。

## 要主 agent 跑的命令

草稿目录 `/home/fy5090/.local/share/singlefs/work/gpu-placement-measures-governance-r1-forward` 里放了六份喂钩子用的 JSON（`popup-original-e202.json`、`popup-deviation-only-formatted.json`、`popup-fake-standard.json`、`popup-reworded-no-placement-word.json`、`write-guard-tooling-writer-edits-existing-gate-fixture.json`、`bash-detector-tooling-writer-copies-into-gate-fixture.json`）。我一条都没起；下面每条的预测是读码推的，跑出来不同就是我读错了，以实测为准。

1. 原弹窗形态喂放置类钩子（预测退 2，stderr 点名问句没点名规则、两个选项没起头、没有「标准：」选项）
   命令：`task-server/task submit -- bash -c 'bash .claude/hooks/ask-user-placement-guard.sh < /home/fy5090/.local/share/singlefs/work/gpu-placement-measures-governance-r1-forward/popup-original-e202.json'`
   在哪起：仓根 `/home/fy5090/code/singlefs`。看：`TASK_SERVER_RESULT` 的 `type=command-failed`、`exit=2`；`task-server/task log <号>` 里有「问句没点名」「选项说明没以「标准：」或「偏离：」起头」「一个以「标准：」起头的选项都没有」。
2. 格式全对但只列偏离的选项（预测退 2，只缺「标准：」那一样）
   命令：`task-server/task submit -- bash -c 'bash .claude/hooks/ask-user-placement-guard.sh < /home/fy5090/.local/share/singlefs/work/gpu-placement-measures-governance-r1-forward/popup-deviation-only-formatted.json'`
   在哪起：仓根。看：`exit=2`；日志里只有「一个以「标准：」起头的选项都没有」一类，没有「问句没点名」。
3. 假「标准：」选项（预测退 0：格式三项都过，打中 A）
   命令：`task-server/task submit -- bash -c 'bash .claude/hooks/ask-user-placement-guard.sh < /home/fy5090/.local/share/singlefs/work/gpu-placement-measures-governance-r1-forward/popup-fake-standard.json'`
   在哪起：仓根。看：`type=succeeded`、`exit=0`。
4. 换了说法的放置类问句（预测退 0，打中 B）
   命令：`task-server/task submit -- bash -c 'bash .claude/hooks/ask-user-placement-guard.sh < /home/fy5090/.local/share/singlefs/work/gpu-placement-measures-governance-r1-forward/popup-reworded-no-placement-word.json'`
   在哪起：仓根。看：`type=succeeded`、`exit=0`。
5. 写闸喂「工具员 Edit 已有门禁的样本档」（预测退 2，stderr「不在主工作区里写已有门禁 doc-process-records.sh 的样本档目录」）
   命令：`task-server/task submit -- bash -c 'CLAUDE_PROJECT_DIR=/home/fy5090/code/singlefs bash .claude/hooks/write-guard.sh < /home/fy5090/.local/share/singlefs/work/gpu-placement-measures-governance-r1-forward/write-guard-tooling-writer-edits-existing-gate-fixture.json'`
   在哪起：仓根。看：`exit=2`，日志里有「已有门禁 doc-process-records.sh 的样本档目录」。
6. 检出钩子喂「工具员 Bash 里 cp 进已有门禁的样本档目录」（预测退 0、不拒，打中 1.6 的 Bash 一半）
   命令：`task-server/task submit -- bash -c 'AGENT_HOOK_DETECTIONS=/home/fy5090/.local/share/singlefs/work/gpu-placement-measures-governance-r1-forward/bash-detections.txt bash .claude/hooks/bash-command-detector.sh < /home/fy5090/.local/share/singlefs/work/gpu-placement-measures-governance-r1-forward/bash-detector-tooling-writer-copies-into-gate-fixture.json'`
   在哪起：仓根。看：`type=succeeded`、`exit=0`（退 2 就是我读错，⑭ 已经有这一半）。
7. crate-roles 判法对真仓（预测退 0，成功句写「滞后表 22 行（整包 1、条目 21）」，单跑、不带基准当「没给基准」）
   命令：`task-server/task submit -- python3 scripts/gate/crate-roles-check.py /home/fy5090/code/singlefs`
   在哪起：仓根。看：`exit=0`，日志里「✓ 判了 crates/ 下 11 个包」那一行。
8. gate-overlap 对这一次改动（读 ask-user-placement-guard.sh 有没有被判整段抄或标识符重合而没点名）
   命令：`task-server/task submit -- python3 .claude/singlefs-ai-sop/scripts/gate-overlap.py`
   在哪起：仓根。看：`exit=0`；红的话看点名的文件是不是 `ask-user-placement-guard.sh`。
9. rules-lint 与 doc-lint 看 `directory-layout.md`（5a；doc-lint 全仓有存量，只看有没有行点名 `directory-layout.md`、尤其第 42 行的「这份规则」）
   命令：`task-server/task submit -- bash .claude/singlefs-ai-sop/scripts/rules-lint.sh`
   在哪起：仓根。看：`exit=0`，日志无点名 `.claude/rules/directory-layout.md` 的 ✗ 行。
   命令：`task-server/task submit -- bash .claude/singlefs-ai-sop/scripts/doc-lint.sh`
   在哪起：仓根。看：日志里没有点名 `.claude/rules/directory-layout.md` 的 ✗ 行（别的文件的存量违规不算）。
10. 「今天绿靠滞后行」的两处（1.2、1.4）要在有副本的树上删行再跑：复核不了（命令写不成一条：要先建副本、删表行，再交 `bash .claude/gate.d/checker-independence-and-sync.sh` 与 `bash .claude/gate.d/code-tooling.sh --check crate-roles`；用现有样本代替：`code-tooling.sh/crate-roles-empty-library-tests-red` 与 `checker-implementation-disjoint-cross-package-source-red` 的 expect 已写明无滞后行时红在哪几行，样本档由主 agent 跑，背景里已量「判得都对」）。

## 量的

- 开工快照核：`sha256sum -c research/prompts/gpu-placement-measures-governance-r1-snapshot/judged-sha256.txt`，19 行 OK、1 行 FAILED（`.claude/gate.d/break-switch-replay-files.tsv`）、末行「sha256sum: WARNING: 1 computed checksum did NOT match」。
- 三张滞后 / 放行表的行数：`grep -vc '^#' .claude/gate.d/crate-roles-lag.tsv` 得 22，`grep -vc '^#' .claude/gate.d/crate-cross-references-lag.tsv` 得 2；`crate-roles.tsv` 非注释行 `grep -c '^[^#]'` 得 11。
- 角色登记里实验装置角色的包数：`awk -F'\t' '!/^#/ && $2=="实验装置"' .claude/gate.d/crate-roles.tsv | wc -l` 得 0。
- `crates/` 各包 `tests/` 项数与库大小（`ls` 与 `wc -c` 量的）：`singlefs-core`、`singlefs-format`、`singlefs-checker`、`singlefs-reference-filesystem`、`singlefs-device-library` 的 `tests/` 都是 0 项；`singlefs-verification` 518 项；`singlefs-background-gpu-tests` 的 `src/lib.rs` 227 字节（去掉注释与 `#![deny(unsafe_code)]` 后为空）。
- 对 `.claude/rules/directory-layout.md` 第 17 至 45 行 grep 历史与论证用词：`rc=1`（零命中）；`.claude/rules-lint-exclude`：`ls` 报 No such file。
- `scripts/experiment-apparatus/Cargo.toml` 的 `[[bin]]` 数：178（`grep -c '^\[\[bin\]\]'`）。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 1：产品包放 `src/bin/e202_…rs` | 成立（拦住） | code-tooling 的 crate-roles 格 `crate-roles-check.py:356`、`:359` 红；今天产品包没有 `src/bin/`，绿 |
| 1：另开空库加测试目标的包 | 成立（只拦空库形态）；打中：库里放一行代码 | `crate-roles-check.py:366` 红；今天靠整包滞后行绿；一行代码的伪装不红（推的，归攻方腿造样本） |
| 1：搬得超出定案、泛称包名（原因 ③） | 推不出 | 没有检查读定案，没有格判包名 |
| 1：测试 `#[path]` 引验证包 | 成立（拦住） | ⑨ `checker-independence-and-sync.sh:934`、`:941`、`:943` 红；今天靠 `crate-cross-references-lag.tsv` 第 3 行绿；首次提交时两张滞后表无基准，涨没比 |
| 1：弹窗只列偏离选项 | 成立（原话形态拦住）；打中：假「标准：」、换说法 | `ask-user-placement-guard.sh:128`、`:140`、`:142` 红（待主 agent 喂）；`:52` 自述两个残余 |
| 1：工具员先把样本改进主工作区 | 成立（Write / Edit 拦住）；打中：Bash | `write-guard.sh:972`；`:110` 自述 Bash 看不见，检出钩子没有这一半，原事件用哪种工具推不出 |
| 4：与已有检查重叠 | 部分成立 | 没有判同一条判据的已有检查；gate-similar 三行对；⑨ 所在门禁缺 crate-sources-reachable 一句（建议，非红）；依赖解析与去注释各有重复实现（只记事实） |
| 5a：与 `rules-discipline.md` | 成立（合） | 零论证零历史；小项一第 42 行「这份规则」自称；小项二第 20 行两句没进「靠 review 的」 |
| 5b：与 `verification.md:9` | 冲突成立 | 验证包装「实验」「真设备装置二进制（`src/bin/`）」对「验证包只装测试…与门禁、崩溃入口自己起的工具二进制」；D13 已定项 15 同样还是旧话；非 `e<数字>_` 的两份装置漏搬无格会红 |
| 5：第 20 行与验证包、崩溃枚举用例放法 | 成立（文本冲突，推的） | 「一个包的集成测试放它自己的 `tests/`」没给验证角色留例外，与 `crash-case-check.py:26` 方向相反 |
| 5：登记表与实验装置包 | 登记与角色定义不一致算过渡态、不算打中；实验装置包在哪推不出 | `singlefs-background-gpu-tests` 登成验证不在角色定义里；实验装置角色 0 个包，「没有包依赖实验装置包」今天空判；`scripts/experiment-apparatus` 算不算实验装置包规则没说 |

## 没做什么

- 不判别的腿的格：格 2、3（误伤与判法本身）归攻方腿，这里没判；报告里出现的「打中」变体（库里放一行代码、假「标准：」、换说法、Bash 写样本档）只列形态，样本与能不能绕过由攻方腿与主 agent 核。
- 不替主 agent 采纳或出判决：上面「成立 / 不成立 / 推不出」是正推腿的读法，每条的推翻现象已写。
- 没往统一任务服务交任务，没起自证、样本档、门禁、cargo、`gate-overlap.py` 的非 `--list` 判，没喂任何钩子；判决要引的数没复跑（命令在「要主 agent 跑的命令」一节，预测都是读码推的）。
- 没写模型与探针，没占内存与时间；草稿目录里只有六份喂钩子的 JSON、`overlap-list.txt` 与一份截出的规则正文。
- 没扫全仓：「没有判同一条判据的已有检查」的范围是 `--list` 的 69 份加我读过的 `scripts/gate/`、`scripts/crash/` 几份判法脚本；`.claude/agent-common.md` 与 `.claude/agents/tooling-writer.md` 第 3 步 5 处的落地没判（背景写明这一轮不判）。
- 没读禁读清单里的任何文件（攻方腿这一轮的产出与草稿目录）。
