# 门禁修复留下的岔路 第二轮正文（2026-09-24）

<!-- doc-lint:not-numbers T1 T3 T4 T5 T6 T7 T8 T9 T10 T11 -->

## 一、这一轮要判什么

第一轮判决（`research/prompts/gate-fix-forks-r1-main-verification.md`）给七格定了暂定形态，并按 `.claude/rules/implementation-workflow.md` 先写成了代码与判别力样本（工作区，未暂存）。这一轮攻**写好的代码与样本**：每格的暂定形态有没有具体可复现的失效；第一轮判决对落选候选的判法站不站得住。第一轮选定的 T9、T10（维持现状）只由辩方复核，交用户的 T2 与 T7 的 kb 目录、git 两类不在这一轮。

共用问句不变：

> **这一格的暂定形态，门禁才既不放过它该拦的、也不留下一道改工作区里任何文件都消不掉的红？**

每格只落一种结论：**站住**（给出这一轮试过、没打中的具体形状）/ **打中**（一个具体可复现的失效，或一句逐字抄得出、与它直接冲突的条款原文）/ **打中但改法在手**（打中，并给出一个改法与它修的是哪一格）。

## 二、实现今天的样子（主 agent 的观测，2026-09-24 现查）

**这一轮不改 `crates/`**：被判的全在 `.claude/gate.d/`、`.claude/scripts/`、`research/scripts/`、`.claude/rules/`；`crates/` 下的 73 个未提交路径全是别的会话的。与 `crates/` 有关的现查只有 T4 一条：`crates/singlefs-harness/src/bin/` 下 4 份：`e156_allocation_basis_counts.rs`、`first_transaction_device_log_check.rs`、`first_transaction_on_device.rs`、`first_transaction_region_bytes.rs`。

门禁的调用契约（`.claude/singlefs-ai-sop/scripts/gate.sh`，上游副本，本仓不许改）：退出码 0 记通过、77 记「本次未跑」、其余记失败；`gate.sh:115–123` 算一次 `GATE_DIFF_BASE`（`lib.sh` 的 `diff_base`）导给每道阶段；样本自检（`stage-selftest.sh`）在清掉 `GATE_BASE`、`GATE_STAGED_FROM` 的环境里跑，**不清** `GATE_DIFF_BASE`。

被判的文件（改动都在工作区；未跟踪的整份都是新写的）：

| 格 | 文件 |
|---|---|
| T1 | `.claude/gate.d/10-kb-rot.sh`、`fixtures/10-kb-rot.sh/`；对照 `.claude/gate.d/75-decision-experiment-links.sh`（没改） |
| T3 | `.claude/rules/implementation-workflow.md`「改 agent 定义与共用约束，走同一条三步」那一节 |
| T4 | `.claude/gate.d/80-absolute-assertions.sh`、`fixtures/80-absolute-assertions.sh/` |
| T5 | `research/scripts/changed-paths.sh`（未跟踪）；`.claude/gate.d/64-change-range-single-source.sh` 与 `fixtures/64-change-range-single-source.sh/`（未跟踪）；`.claude/gate.d/40-results-cited.sh`、`61-settled-same-file.sh`、`88-quoted-result-lines.sh`、`92-layout-checker-sync.sh` 与前三份的 `fixtures/`；`30-decision-history.sh`、`66-abandoned-rounds.sh` 各一行 |
| T6、T7 | `.claude/gate.d/52-segment-registry.sh`、`fixtures/52-segment-registry.sh/`（未跟踪）；`.claude/gate.d/31-blocking-verdict.sh`；被调的 `research/scripts/check-segment-registry.py`（别的会话的未提交改动，这一轮不改它） |
| T8 | `.claude/scripts/gen-decision-items.py` 的 `clip` |
| T11 | `research/scripts/changed-paths.sh` 的 `gate_diff_base`；上游 `lib.sh` 的 `diff_base` |

## 三、各格的暂定形态与这一轮要攻的问句

### T1　撤掉 10 号 §3，射程交 75 号 ⑤（戊）

形态：10 号不再判「实验改成已跑、引用它的决策有没有同批改」；75 号 ⑤ 把实验页标题行的变动算作正文改了，要求同一批回看影响的决策表至少一行、写「改了」的那条决策文件在同一批里。第一轮判决记下戊比「丙b*（只判这一批、要求引用它的那份决策在范围里）」少判一样：⑤ 改过影响表**任一行**就放行。

问：找一段**这一批里的**改动，丙b* 会红、75 号 ⑤ 放行，而那正是 10 号 howto 要拦的（「把这个实验的结论写回它支撑的那条决策」）；或者证明这样的改动在 75 号其余几条（① 每条提到的决策都要有一行、③ 双向）下也会红。另问：10 号剩下的「已跑实验要有决策引用」与 75 号 ⑨「实验必须对应决策」是不是同一件事；按子串认「已跑」把「已跑但结论作废」也算进去，与 75 号 ⑨ 排除作废、退役的做法冲不冲突。

### T3　implementation-workflow.md 补「三步在定义上各取什么」

形态：第 1 步不适用（定义是文档，`show-me-test.md`「没有测试的 patch 一律不收」只管 `crates/*/src/`，只改文档和脚本除外）；第 2 步就是那一轮三方；第 3 步照 `show-me-test.md`「新增的测试必须先证明它会红」里「这条同样管设计论证」那一句。

问：这段话与哪一条现有条款原文冲突；「第 1 步不适用」会不会放过一类改定义时本该带会红检查的改动（给出一次具体的改定义）。

### T4　80 号射程加各 `src/bin` 下的 `e<数字>_*.rs`（庚）

形态：判 `research/e7-index-bench/src/bin` 下全部，加上 `crates/*/src/bin`、`research/*/src/bin` 下文件名以 `e<数字>_` 开头的；其余只在成功行逐个列名。

问：按文件名认实验，会漏掉哪一类实验、误罩哪一类工具（逐份判今天那 4 份，再给出以后会出现的形状）。

### T5　改动范围只许一份取法

形态：共用脚本 `changed-paths.sh` 三个函数——`gate_diff_base`、`gate_changed_paths`（三次 git 调用逐条判退出码，失败退非 0）、新加的 `gate_added_lines <基准> <路径…>`（输出「路径<TAB>行文」：跟踪文件取 `git diff --unified=0` 的 + 行，未跟踪文件整份算新增）；40、61、88、92 改用它（40 号改成三道都判完再退出）；新立 64 号：① `.claude/gate.d/` 顶层的 `.sh`、`.py` 代码行里出现 `merge-base`、`@{upstream}`、`gate-ok`、读 `GATE_BASE` 或 `GATE_DIFF_BASE` 的判红；② 列路径的 git 调用（`--name-only`、`--name-status`、`ls-files`）不带 `core.quotepath=false` 的判红，认同一行带、同文件里带着它定义的变量或列表、同文件里带着它的包装函数、带 `-z` 四种写法；「代码行」不含以 `#`、`echo`、`printf`、`print(`、`bad`、`howto`、`die`、引号开头的行。

问：`gate_added_lines` 在哪种输入上给错（改名、删除、二进制文件、路径带空格或制表符、没有结尾换行、CRLF、子模块、`diff.noprefix` 之类的配置）；40、61、88、92 改写之后有没有哪个判定变了（给出新旧两版判得不同的输入）；64 号的两条判据漏判与误判的具体写法。

### T6　52 号按阶段自己的位置调仓里的脚本，样本只放三样合成输入（乙）

第一轮攻方没打中乙，只抽了一次样。问：找一个让仓里 `check-segment-registry.py` 退化、而乙形态的样本仍判对的变异；或者一个样本本身判错的输入。

### T7　52 号找不到脚本、31 号找不到生成器退 1

问：列出今天会走到这两个分支的全部场合（真仓整轮、样本自检、`gate.sh --staged` 的临时 worktree、单独跑），哪一个场合下退 1 是误红。31 号去掉了「按脚本位置取不到就退回按 cwd 取」那条后路：有没有哪个场合原来靠那条后路才取得到生成器。

### T8　剥尾正则与 doc-lint 认编号同形（丙a）

形态：`clip` 只在真截断过时剥尾，剥的编号是 `(?<![A-Za-z0-9._-])[A-Z]+-?\d+(?:\.\d+)*\s*$`，与 doc-lint 认编号的形状一致（整词、前一个字符不是字母数字 `.` `_` `-`）。截断正好停在领域词（`SHA256`、`RAID5`）之后时整词剥掉。

问：按这个正则逐格算一张输入表，找出剥成半截的、剥掉了不该剥的、该剥没剥的（剩下裸引用、doc-lint 会判红的）。

### T11　共用脚本认不认 `GATE_DIFF_BASE`

第一轮判决的现查：今天 `diff_base` → HEAD~1，`gate_diff_base gate` → HEAD；两者只在「全推出去了」时不同。11 道依赖改动范围的阶段在两个基准下，10 道输出逐字相同，68 号不同：HEAD~1 把上一次提交里的同步记录算进这一批，替这一批又改了的 7 个文件顶了账。

问：在「全推出去了」与「本地有没推的提交」两种仓状态下，两个候选各让哪一道阶段判错（放过该拦的或拦了不该拦的）；认 `GATE_DIFF_BASE` 时，样本自检不清它（见第二节），共用脚本要怎么防真仓的提交号漏进样本的临时仓。

## 四、三条腿的分工（互不重叠）

| 腿 | 只做哪几格 | 不许碰 |
|---|---|---|
| **云端攻方（Opus）** | T1、T5、T6、T11：给具体可复现的失效（一段历史、一个输入、一个变异），跑出来 | T3、T4、T7、T8，辩方的复核 |
| **本地攻方** | T4、T7、T8：按提示里写死的事实表逐格判、逐格算 | T1、T3、T5、T6、T11 |
| **云端辩方（Sonnet）** | 复核第一轮判决：T1（替丙b* 辩：戊少判的那一格是不是真缺口）、T3（替正推的「①③ 暂缺」辩）、T8（替乙「读 not-numbers 当词表」辩）、T9 与 T10（第一轮选现状，够不够得着）；T3 的形态本身由辩方顺带看有没有与条款原文冲突 | 不造新反例攻代码，那是攻方的格 |

## 五、跑前写死的判据

- 一个**具体可复现**的失效（给得出文件、提交或输入写法，现跑确实如此）⇒ 打中。
- 一句**逐字抄得出**、与形态直接冲突的现有条款原文 ⇒ 打中。
- 只说「可能有风险」而给不出具体对象、写法或原文的 ⇒ 没打中。
- 「没打中」按 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」要有第二次抽样才采信：T6 这一轮就是第二次。
- 腿自己提的改法算「被攻过零轮」，写明。

## 六、不许做的

- 不许改仓里任何文件，也不许改 `.claude/singlefs-ai-sop/`。
- 不许跑 54、55、57、59、87 号。
- 上游 `gate.sh` / `lib.sh` 的改法可以写成「要等上游」的前提，不当这一轮的改法交。
