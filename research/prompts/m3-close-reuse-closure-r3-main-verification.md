# m3-close-reuse-closure-r3 判决（推论轮第三轮，最终轮，2026-10-23）

被判对象：第二轮判决 `research/prompts/m3-close-reuse-closure-r2-main-verification.md` 第三节的形态。丙有三件：HostLookup 整体兜底；源头打标记、判法只去标记行；闭包收模块级 import 与赋值，再只排一条边。甲：按值打包，把本机同一份 `host-resources.py` 连同它在仓内 import 的文件送到第二台。条文：文件级与边级两种排除并列。正文 `research/prompts/_m3-close-reuse-closure-r3-body.md`。腿：辩方（Sonnet）`research/prompts/m3-close-reuse-closure-r3-sonnet-output.md`（ddb12a48…），攻方（Opus）`research/prompts/m3-close-reuse-closure-r3-opus-output.md`（bbdcf07c…）。这一轮是最终轮，不开第四轮；这一轮新冒出的改法都算零轮，进第四节的交用户表。

## 一、核查

1. **开工快照**：`judged-sha256.txt` 9 行。写判决时 `sha256sum -c`，`scripts/hosts/peer-host-lib.sh` FAILED，其余 8 行 OK。
   - 改的是别的会话：头注释少了「第二台共用编译目录」两段，函数 `peer_host_shared_target_guard` 整段删掉（主树比快照少 81 行），`peer_host_run` 那一段没动。
   - 两腿的量都在开工时拷的冻结副本上做。攻方留下的副本 `~/.local/share/singlefs/work/m3-close-reuse-closure-r3-opus/repo` 里 9 份逐份过快照（9 行 OK），其中 `peer-host-lib.sh` 与 HEAD 相同。复跑用它当冻结树。
   - 腿引的行没有落在被删的那一段：辩方引 `peer-host-lib.sh:77` 的 ssh 一行，按的是交回时主树的行号，冻结副本里在第 84 行，是同一行。
2. **cite-check**：`python3 scripts/agents/cite-check.py <两份腿报告> --root . --background research/prompts/_m3-close-reuse-closure-r3-background.md` 的末行是「✓ 核了 74 处引文、14 处裸行号，对上 74 处，没判 14 处（其中裸行号没有引文可比 14 处；逐处列在上面）」。
   - 改拿冻结副本当 `--root`，有 7 处对不上。这 7 处引的都是快照以外、开工后被别处改过的文件（`crates-mutation-rows.py`、`merge-and-verify.sh`、`lib_heavy_tests.py`、`code-tooling.sh`、`peer-host-lib.sh`），两腿都写明按交回时的主树取行号。
3. **主 agent 现查**：
   - `lib_function_units.py:7201` 是类被点名时连方法的那一句「if method.container is head and (method.short in self.attributes_seen or (method.short.startswith("__") and method.short.endswith("__"))):」。
   - 包装的第 813 行是「fcntl.flock(handle, fcntl.LOCK_EX)」，第 947、959 行是不带时限的「self.thread.join()」，第 776 行是「HOST_LOOKUP_PEER_TIMEOUT_SECONDS = 8」。
   - 本机 `/usr/include/linux/binfmts.h` 写着「#define MAX_ARG_STRLEN (PAGE_SIZE * 32)」，与辩方量的「单个参数 131,071 字节装得下」相容。
   - `grep -rn NOT_JUDGING_FILES .claude/kb` 0 行。
   - `grep -n allow_refused .claude/kb/checks-owed.md` 只命中 C888（广谱变异指纹带拒算也出键）。
4. **复跑**：见第六节。

## 二、逐格判定

| 格 | 辩方 | 攻方 | 主 agent 判 |
|---|---|---|---|
| X1 丙第 1 件（整体兜底） | 站得住；在 HostLookup 原文上注入五个点，起线程、建临时文件、格式化三点是异常、冒出调用方，try 关得了；排队锁不放与 join 无时限是**卡住**，try 关不了，只能靠 LOCK_NB 与 join 时限，自证要造「锁被别的进程占着」「线程里的 query 睡着」两种世界，用抛异常去假注入，那两格会假绿 | 没攻（前两轮攻过） | **站住**。三轮里第一、二轮打的是 try 的范围，第三轮把范围补全之后没被打中。自证的造法要照辩方那一句写：两格造卡住，不造异常 |
| X1 丙第 2 件（源头打标记） | 去多：这一击打不到窄正则版（仓里已有 `WRAPPER_QUEUE_LINE` 只去排队两种行）。去少：这一击对源头标记版同样打中，`say()` 共用，「! 告警记录写不进」两处各打一次，标记要逐个调用点打。另一个代价：标记会让 `scripts/gate/merge-and-verify.sh:517` 那个锚在行首的 grep 不再命中 | X2① 打中：读包装输出、又决定判定的还有 e165 `replay`（`scripts/experiment-apparatus/e165-pin-red-examples.py:1269`、`:1272`）与 crates 变异 `verdict_of_run`（`scripts/mutation/crates-mutation-rows.py:1479`）；这两处的闭包里包装是 0 个单元；丙第 2 件的自证只钉住 Verus 那一处 | **打中，挂起**。两腿从两侧打中同一个缺口：被判形态只点了 Verus 一处，读包装输出的判法与读者至少还有 e165 replay、crates 变异、`merge-and-verify.sh:517` 与看门狗。去标记行，还是像辩方说的用窄正则去四种行首，是零轮选项，进交用户表 |
| X1 丙第 3 件（收模块级语句） | 打中：不收 class 头，与 `.claude/kb/decisions/13-验证路线.md:434`「不静默少收」打架。辩方的收法让类头进来时触发 `class_entered`，连进方法，21 个入口新冒出拒算 | 打中（量过，副本）：import 时会执行的除了 import 与赋值，还有类头与带装饰器、缺省值的 def 头。改坏一个类头，四把键都不变，包装在 predict 世界里 0 变 252；改坏一个没人调的 def 的缺省值，0 变 251。「连 class 头会把拒算带回来」带回来的其实是方法：只收类头本身（mlh），35 个入口拒算 0、类头那一改键会变，def 头那一改仍不变 | **打中，挂起**。两腿一致：照字面做，闭包仍静默少收 import 时执行的单元。两腿对「收类头带不带回拒算」说法不一，是因为收法不同：辩方的变体连了方法，攻方的 mlh 不连。两份都是腿自己的补丁，都只被攻过零轮。改法（另收类头本身与 def 头，或把它们另登记成一种排除）进交用户表 |
| X1 键变多宽 | 采纳形态在登记表现取的 41 个入口里变 34 个，每个多 52 到 523 个单元；第二轮判决引的「25 个、多 18 到 45」是另一个补丁的数 | 35 个入口里变 32 个，每个多 86 到 523 个单元；没人读的常量 `LONG_RUN_SECONDS`、`DEFAULT_RESERVE_TEXT` 进了键，与第 433 行「资源上限、限时」不进键字面相碰（攻方自己记成相碰、不记违反）；`lib_heavy_tests.py` 168 个单元进了 check_proof 的键 | 两腿入口清单不同（35 个与 41 个），数各自成立、方向一致。采：第二轮判决引的「25 个、多 18 到 45」不能当被判形态的代价，改正见第五节。按 `.claude/kb/decisions/13-验证路线.md:436`，键变多少不挡这一改 |
| X1 甲（「送过去的文件已在本机闭包里」） | 打中：切边之后 `host-resources.py` 在两个闭包里 0 个单元 | 打中（量过）：在四个闭包里都是 0；打包要送的两份 shell 也是 0 | 两腿一致，**打中**：那半句理由是假的，送过去的文件是被丙排出键的。在丙之下这不改判定（那条路只出提示），但条文不能照这半句写 |
| X2 甲（按值打包） | 打中：整棵进程树打开 13 份文件、951,447 字节，`peer_host_run` 把命令文本当 ssh 的一个参数，单个参数只装得下 131,071 字节；今天送的程序 23,800 字节。另外要 Python 3.11 以上（`tomllib`），探测每台只有 8 秒 | 打中（量过）：要带 13 份（8 份 python、5 份 shell），9 份不在 `scripts/hosts/`。3.11 那一条在本机模拟过；host-resources 的结论今天读本机的峰值表、派发账与多机配置，空 HOME 下预测从历史峰值 1 MiB 掉到缺省 8 GiB；`host-dispatch.py` 那一路甲没管 | 两腿一致，**打中**：照被判形态的字面，甲送不过去、版本不一定够、送过去也算不出本机那个结论。两腿各自提出的替代（辩方：保持今天的做法，送本机 `memory_peaks_lib.py` 源码加 `inspect.getsource(probe_this_host)`；攻方：第二台只跑读数那一段）落在同一个形态上，零轮，进交用户表 |
| X2 其余落点 | 读包装输出的调用点还有 `verus-proof-check.py:1339`、`e165-pin-red-examples.py:1184`、`merge-and-verify.sh:517`、`agent-watch.py`；LOCK_NB 取不到锁要多一种原因行；`selftest-registry.tsv`、`verdict-library-scripts.tsv` 三行的 `reuse-three-questions` 要逐格重核；文件头、三语条文与两处复述要一起改；入口清单该从登记表机器生成，不手抄 | X2⑥ 没打中：复用三问的「闭包外改动」都不碰 python 模块级语句 | 采：辩方列的落点进实现时的验收清单（第四节）。两腿对 reuse-three-questions 的判法不冲突：攻方判的是今天的格不会被打红，辩方说的是改了之后要重核造法 |
| X3 前两轮的数 | 核上：1579→1232、792→436、4 个入口变（三种切法）、164/290（切边树、含 class 头）、1232→1655、776（现状树 777）、10 份/7 份（下界）；核不上：「25 个、多 18 到 45」在今天的树上是 26 个，多出 e165 `replay` | 核上 7 个；口径不对 2 个：164/290 是切边树上的数，含类头，今天的树上是 348/473，「随丙第 3 件关」关不掉；「25 / 18 到 45」不是被判形态的数 | 两腿用互不共用的办法（攻方：被调方换成空文件、自己写的 ast 普查、strace；辩方：三种切法、从登记表取入口、自己的解析器）核出同一批数。采两腿共同的结论，改正见第五节 |
| 辩方零轮：按文件排 `host-resources.py` 与 `peer-host-lib.sh` | 够不着：用现成的 `excluded_files` 排这两份，三个形式化入口拒算 0，两个模块级 import 的自证照过，不改闭包算法，只动 4 个入口的键；缺点是键里多留 267 个只经那条边进来的单元，`Runner.escalate_step` 用自己的 `closure_key`，没有排除文件的参数 | — | 零轮，进交用户表 |
| 条文 | 边排除在已定项 17 里没有授权；`NOT_JUDGING_FILES` 是按角色排（理由在 `scripts/crash/lib_verdict_library.py:20`），边排除是按实测排，两条理由不同，要分开写；kb 里零命中 | 「两种排除并列」与第 433 行「文件级…的指纹或闭包」字面不相碰（攻方对读） | 采：条文分两句写，各写各的理由；`NOT_JUDGING_FILES` 进条文。改法见第四节 |

## 三、多轮小结

- 第一轮：打中「只打提示」在退出码与写进库的句子两层都不成立，按边排除会丢掉预测路径上的单元，甲的前提不存在。
- 第二轮：打中 try 的范围不够、按前缀去行去多又去少、闭包不收模块级语句（今天就有的盲区）、甲的同步写法。站住的有两条：只排一条边，键里不算第二台。
- 第三轮：
  - 站住：整体兜底（自证造法照辩方那一句）。
  - 打中：源头标记只钉住了一处，闭包照字面仍然少收类头与 def 头，甲的按值打包送不过去，「送过去的文件已在本机闭包里」这半句是假的。

三轮里被多数打穿的：甲的三种写法（常驻仓、同步、按值打包）各被一轮打穿；闭包怎么收，第二、三轮都打中。三轮都站住的只有「只排一条边」（第三轮两腿用登记表的 41 个入口复核，变的仍只有那 4 个）。

## 四、交用户表

这几条都只被攻过零轮，是两条腿自己量的补丁或主 agent 的组合，都没在入库装置上量过。

| 题 | 选项 | 各自已量到的 | 没量的 |
|---|---|---|---|
| ① 闭包怎么收 import 时执行的东西 | 甲一：模块级 import、赋值之外，另收类头本身（不连方法）与「带装饰器或缺省值」的 def 头（攻方 mlh 加 def 头那一样）。甲二：类头与 def 头另登记成一种排除，写进条文、配一格自证。甲三：照第二轮形态，只收 import 与赋值 | mlh：35 个入口拒算 0，类头那一改键会变；def 头那一样没实现。甲二：辩方量过，连了方法的收法会冒出 21 个入口的拒算（不是甲二本身的数）。甲三：类头、def 头一改，键不变、包装 0 变 252/251 | 甲一的 def 头那一半；三种在入库装置上的数 |
| ② 键要不要不收只被定义、没人读的字面常量（`LONG_RUN_SECONDS` 一类） | 收（照第二轮形态）；或赋值右边是字面常量的，只在被读时进键 | 收：32/35 个入口的键变，每个多 86 到 523 个单元 | 后一种没实现 |
| ③ 判法怎么不读 HostLookup 的行 | 源头打标记，覆盖 Verus、e165 replay、crates 变异与 `merge-and-verify.sh:517`；或窄正则列出 HostLookup 的四种行首（仓里已有 `WRAPPER_QUEUE_LINE` 先例） | 两种都要一格「新增一个 `say` 调用点而没登记就红」的自证；标记版会让 `merge-and-verify.sh:517` 那个锚在行首的 grep 不再命中 | 两种都没实现，没起真包装逐路径跑 |
| ④ 甲改成什么 | 甲改一：保持今天的做法，第二台只跑读数那一段，送过去的是本机 `memory_peaks_lib.py` 源码加 `inspect.getsource(probe_this_host)`，结论在本机算；`host-dispatch.py` 那一路照它今天的同一种写法。甲改二：放弃甲，只做丙 | 今天送的程序 23,800 字节，装得下，只用标准库，在模拟的低版本下照跑 | 第二台上的任何数（传输用时、Python 版本、8 秒够不够） |
| ⑤ 丙第 3 件的切法 | 照第二轮形态：改闭包算法，再切一条边；或辩方的按文件排：排 `host-resources.py` 与 `peer-host-lib.sh`，不改闭包算法 | 按文件排：三个形式化入口拒算 0，两个 import 自证照过，键里多留 267 个单元，`Runner.escalate_step` 要在驱动里另写过滤 | 按文件排对广谱变异那一把的数 |

不论选哪一项，实现时都要照做的（两腿列的落点，不是选项）：

- 自证：HostLookup 的六个注入点里，锁与 join 两格造卡住的世界；`reuse-three-questions` 那三行逐格重核；`lib_function_units.py`、`lib_verdict_library.py`、`run-with-memory-cap.sh` 的 `--selftest` 重跑。
- 条文：文件级排除与边级排除分两句写，各写理由（文件级按角色排，边级按实测「不改结局」排）；`NOT_JUDGING_FILES` 写进条文；改 `.claude/kb/decisions/13-验证路线.md` 的同时，改它的英文、日文两份，以及 `.claude/rules/implementation-workflow.md`、`.claude/rules/verification.md` 两处复述。
- 入口清单由登记表机器生成（辩方 `tools/gen_entries_registry.py` 的取法），不手抄 35 个。

## 五、另登欠账与改正

| 欠账或改正 | 去向 |
|---|---|
| 判法单元闭包不收 import 时执行的模块级语句。今天的树上，check_proof 缺 348 个、escalate_step 缺 473 个（import、赋值与类头合计；两腿各自核出）；只收 import 与赋值关不掉类头与 def 头那一部分。第二轮判决写成「随丙第 3 件关」，当时没登进欠账表 | 登 `.claude/kb/checks-owed.md` C890 |
| 广谱变异 `import_results` 只核键是不是 64 位十六进制，不按本机现算的键重算（第二轮正推 W3）。当时没登进欠账表 | 登 `.claude/kb/checks-owed.md` C891 |
| 第二轮判决 W2 一格写的「25 个入口的键变（多 18 到 45 个单元）」，是第二轮正推腿自己那一版补丁（窄读法、不切边）的数，不是被判形态的代价。被判形态的代价：35 个入口里变 32 个、多 86 到 523（攻方）；登记表 41 个入口里变 34 个、多 52 到 523（辩方） | 改正写在这里，第二轮判决不回改（冻结证据） |
| 广谱变异 `closure_key(…, allow_refused=True)` 带拒算也出键 | 已有 C888，切边之后不随之关掉（辩方 X1） |

## 六、复跑

两份模型都拷进副本 `~/.local/share/singlefs/work/m3-close-b16-verify/closure-r3-rerun/` 再跑，`--frozen` 都给攻方留下的冻结副本（9 份逐份过快照）。

- 攻方：`bash scripts/hosts/run-with-memory-cap.sh 8G bash <副本>/opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen <冻结副本> --work <副本>/opus-work`。退出码 0（排了 1669 秒本机内存），末行 `name=rerun_compare status=same lines=246 diff_lines=0`。
- 辩方：`bash scripts/hosts/run-with-memory-cap.sh 6G bash <副本>/sonnet-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen <冻结副本> --work <副本>/sonnet-work`。退出码 0，末三行：
  - `name=remote_program_today bytes=23800 base64_bytes=31736 single_argument_limit=131071 fits=True`
  - `name=python_version file=.claude/hooks/lib_heavy_tests.py syntax_min=3.7 newer_stdlib={'tomllib': '3.11'}`
  - `name=python_version file=.claude/hooks/lib_long_run.py syntax_min=3.7 newer_stdlib={}`

  复跑产物与腿留存的 `research/prompts/m3-close-reuse-closure-r3-sonnet-model/out/` 里同名的 15 份逐字节相同（`cmp`）：`diff-class-heads.txt`、`diff-cut-modlevel.txt`、`diff-vs-current.txt`、三份 `fileexcl-*.txt`、九份 `scan-*.tsv`。其余是复跑自己的中间文件，腿那边没有留。

攻方那一趟里 `wrapper-world.sh` 起的是真内存包装（start、predict 两种世界），X1① 的「0 变 252、0 变 251」在复跑里逐行相同；辩方没起真包装，W1 的注入只在 HostLookup 原文摘出来的假世界里做。没核的：第二台上的任何数（传输用时、Python 版本、8 秒时限够不够），两腿都没碰第二台。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 17 | 支撑 | 2026-10-23 不受影响：第三轮打中的都交用户定（第四节），条文改动等用户定案后随写回一起改 |

## 用户定案（2026-10-23，弹窗）

| 题 | 用户选的 |
|---|---|
| ① 闭包收哪些 import 时执行的模块级语句 | 「另收类头与函数头（推荐）」：import、赋值之外，另收类头本身（不连它的方法）与带装饰器或缺省值的 def 头；字面常量照收 |
| ③ 判法怎么不读 HostLookup 打的行 | 「窄正则列四种行首（推荐）」：照 `WRAPPER_QUEUE_LINE` 的写法只去 HostLookup 的四种行首，覆盖 Verus、e165 钉表重放、crates 变异判定句三处判法；不碰升档证据两句，不动 `merge-and-verify.sh:517` 的锚 |
| ④ 甲改成什么 | 「保持今天的做法（推荐）」：第二台只跑读数那一段（本机 `memory_peaks_lib.py` 源码加 `inspect.getsource(probe_this_host)`），结论在本机算；`host-dispatch.py` 那一路同一写法 |
| ⑤ 怎么把查机器那条路排出判法键 | 「改闭包算法再切一条边（推荐）」 |

第 ② 题（没人读的字面常量）随第 ① 题的选项定：照收。

**最终形态（写回与实现照这一份）**：

1. **HostLookup 整体兜底**：照第三节第 1 件（第二轮判决第三节）；排队锁取 `LOCK_NB`、join 加上限，自证里锁与 join 两格造卡住的世界，不造异常。
2. **判法去行**：一条窄正则列 HostLookup 的四种行首（「run-with-memory-cap: 没查到别的机器：」「run-with-memory-cap: 现查别的机器：」「  → <机器>此刻放得下」「  ! 告警记录写不进」），verus-proof-check.py、e165-pin-red-examples.py 的 replay、crates-mutation-rows.py 的 verdict_of_run 三处判法读包装输出前去掉这几行；包装每新增一个 `say` 调用点没登记就红的一格自证；同一份 Verus 输出、这几行内容不同，存进库的判定逐字相同的一格自证。
3. **闭包收 import 时执行的模块级语句**：被载入、被 import 的模块的模块级 import、赋值、类头本身（不当成类被点名、不连方法）与带装饰器或缺省值的 def 头进闭包；再切 `HostLookup.query` 起 `host-resources.py` 那一条子进程边。自证：改 `lib_long_run.py` 一个模块级 import、改 `memory_peaks_lib.py` 的 `import subprocess`、改 `host-dispatch.py` 一个类头、改一个没人调的 def 的缺省值，各让对应入口的键变；切边前后少掉的单元只经那条边可达。关 C890。
4. **甲**：不改第二台那一路；条文里写「第二台那一路只出提示、不改结局，按第 3 件排出键」，不写「送过去的文件已在本机闭包里」。
5. **条文**：D13（验证路线） 已定项 17 写两种排除、分两句、各写理由：文件级（`NOT_JUDGING_FILES`，算键的库自己，按角色排：管怎么存、怎么算键，不管判对错）与边级（一条只打提示、不改结局的子进程边，按实测排，配第 1–3 件的自证）；自证另加一格「键不变而退出码变」必红。三语三份一起改；`.claude/rules/implementation-workflow.md`、`.claude/rules/verification.md` 两处复述随下一批治理文件改。
6. **入口清单**由登记表机器生成，不手抄。
