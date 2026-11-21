# E201 第 1 次重跑登记：变异表与装置单测在今天的树上

写于 2026-11-18，重跑登记，不占新实验号。调度记录 `records/2026-11-17-subagent账本清欠调度记录.md` 第 6 批。
原登记 `research/prompts/e201-preregistration.md`（sha256 `a78819ef899f8ad68cd0a91048f9c9ac94d33ff2f245df35939ef0dceb625fe8`，755 行，含修订 1 至 8；本登记引它的地方都按它写入时的样子读）。
问题单 `research/prompts/e201-r1-questions.md`（sha256 `c8314c7f516b859a2afb7bb24aa30113c700ea5173350d281321feada48bbe1e`，三行，状态都是开着）。

<!-- doc-lint:not-numbers MT UT DR F1 F2 F3 F4 F5 F6 Z1 Z2 Z3 Z4 Z5 Z6 S1 S2 S3 S4 U1 U2 U3 R1 O3 F7 T0 T1 T2 -->

## 一、问题

英文名：three_oracle_collusion_full_table_on_the_current_tree

**为什么重跑，不取新号**：E201（三oracle合谋今树整表）的变异表在原产物之后改过（原产物 `table_rows=50`，今天的表 57 个数据行），而 `research/results/` 里没有一份不比它旧的产物，门禁 doc-experiments 的 evidence-in-repo 格判红。臂与判据没变，变的是输入，所以是重跑。准入：E201 没接 `.claude/gate.d/stage-inputs.tsv` 的实验键，不走 `scripts/gate/admission.py` 的 77，派发提示写明「准入已判」。

**与原登记的关系**：这一次只跑变异这一条，即变异驱动 `mutation_driver.py` 加装置目录的单测；原登记第九节（V1 至 V14 与后补各行）是被跑的表，原登记 5.2 第二张表（逐格比与归因这条管子的单测）是被跑的单测。整表两段（`full_table_run.py run`、`run --stream-three`）、`full_table_run.py --selftest` 与弄坏开关不在这一次里，原登记 Q1、Q2（`research/prompts/e171-current-tree-full-table-questions.md`）也不在这一次里；这一次的三行只问装置自己此刻健不健康，对原登记是备料。

主 agent 给的问题（派发提示原文）：

> 重跑已有实验 E201（三oracle合谋今树整表）第 1 次，不占新实验号，写重跑登记 `research/prompts/e201-r1-prereg.md`。要回答的问题（一种读法）：问题单三行——变异表此刻每一行在今天的树上各是抓到、无效还是没红；装置的六个单测模块在今天的树上是否全过；变异驱动此刻能不能出产物。范围只到变异这一条（`mutation_driver.py` 加单测），整表两段不在这一次里。

岔路单（重跑的问题单 `research/prompts/e201-r1-questions.md` 第 5 至 9 行，`sed -n '5,9p'` 原样；下面「行名」是本登记给三行起的短名，每个量都挂在其中一行上）：

| 问题 | 候选 | 翻面观测 | 够判条件 | 状态 |
|---|---|---|---|---|
| 变异表 `scripts/mutation/tables/e201_three_oracle_collusion_full_table_on_the_current_tree.tsv` 此刻的每一行，在今天的树上各是抓到、无效还是没红 | 全部抓到；有无效或没红的行 | 任何一行不是抓到 | 变异驱动 `mutation_driver.py` 对整张表出一份新产物，产物头报的行数等于表里此刻的数据行数，三个数（抓到、无效、没红）各自落定 | 开着 |
| 装置的单测在今天的树上是否全过 | 全过；有红 | 任何一条红（此刻任务 19166 红一条：`test_every_repository_root_path_the_crates_name_outside_test_code_is_copied_and_synchronized`，`['.'] != []`） | 六个单测模块在今天的树上全过；红的那一条定出是装置的同步清单要跟 `crates/` 的改动，还是 `crates/` 那一侧新写的路径不该出现 | 开着 |
| 变异驱动此刻起得来吗 | 起得来；起不来 | 任务 19167 那一次退 2、日志里没有输出 | 定出退 2 的原因并修到驱动出产物 | 开着 |

### 读法写死

1. **行名**：MT 是岔路单第一行（变异表逐行）；UT 是第二行（装置单测）；DR 是第三行（变异驱动起得来）。第六节每个量写它挂在哪一行上，三行各判各的，不合取。
2. **「此刻的变异表」**：`scripts/mutation/tables/e201_three_oracle_collusion_full_table_on_the_current_tree.tsv`，登记时 sha256 `7ff739ee43965a8fda2af2b75737105327020ab422da2b8b4104f924706304a1`；数据行照驱动 `read_table` 的读法，即不以 `#` 起头、非空的行，登记时现数 57 行（`grep -v '^#' <表> | grep -vc '^$'`）。跑前、跑后各核一次表的 sha256，不同走作废 Z3。
3. **「抓到、无效、没红」**照驱动源码 `scripts/experiment-apparatus/e201_three_oracle_collusion_full_table_on_the_current_tree/mutation_driver.py` 的 `run_unit_tests` 与 `main`：✅ 抓到 = 替换后 `py_compile` 过、单测进程退出码非 0；⏭ 无效 = 替换后 `py_compile` 抛 `PyCompileError`；❌ 没红 = 替换后单测进程退出 0；⏱ 超时 = 单测超过 `TIMEOUT_SECONDS`（300 秒）。三个数是 ✅、⏭、❌ 的条数；⏱ 单列，不进三个数，有一条 ⏱ 就是三个数没落定。⏭ 与 ❌ 的处置照 `.claude/rules/mutation-sampling.md`「没红的三类，判据不同」与第八类，不直接记盲区或等价。
4. **「六个单测模块」**：驱动常量 `TEST_MODULES` 的六个，即 `test_contrast_against_r1`、`test_q2_misses`、`test_attribution`、`test_whole_table_matrix`、`test_current_tree_row_driver`、`test_full_table_run`，与任务 19166 起的是同一组。装置目录里另有第七个测试模块 `test_row_selection_and_reuse.py`（7 条用例，不在 `TEST_MODULES` 里、不在 19166 里）：UT 的判定只看六个，第七个附带报一次结局（第六节 UT-附带），不进 UT，也不并进 `TEST_MODULES`（并进去是改装置，不在这一次里）。**「全过」** = 六个模块一起起，`-v` 输出末尾是 `OK`，且 `Ran` 行的条数等于六个文件里 `def test_` 的行数、failures、errors、skipped 都是 0。
5. **「今天的树」**：跑那一刻的主工作区（含别的会话没提交的改动）。六个模块里有几条读真仓：扫 `crates/` 非测试代码里的字面路径（`test_full_table_run.py` 的 `repository_root_paths_named_outside_test_code`）、读仓根脚本、读 `scripts/hosts/lib_repository_copy.py` 的闭包、读 `.claude/gate.d/path-moves.tsv`；驱动起的单测经环境变量 `E201_REPOSITORY_ROOT_FOR_A_COPY` 指回真仓根，所以变异副本里的单测读的也是真仓。结论只对「那一刻的树」成立，每一次起任务前后各记一份树摘要（5.2 节命令）。
6. **DR 的「起得来」**：驱动退出码 0 或 1，且 `--output` 文件里有 `name=verdict` 与 `name=done` 两行（退 1 是有没红、超时或还原后基线不绿，产物是完整的，读产物判 MT，不读退出码判 MT）。退 2 是基线单测红（`main` 里 `baseline_outcome != SURVIVED` 那一支，产物只有 `config`、各条 `anchor`、`baseline unit_tests=red`、`done`）；退 3 是锚点不是恰好一次（产物末尾 `name=verdict anchors_not_exactly_once=<N> mutations_run=0`）；别的退出码、或日志里有 Python 回溯，都算起不来、另写原因。
7. **产物名与头**：新产物 `research/results/e201-three-oracle-collusion-full-table-on-the-current-tree-<跑那一天>-r1-mutations.out`，日期用 `TZ=Asia/Tokyo date +%F` 取；不用 `…-2026-11-18-mutations.out` 那一个名字（见第四节第 2 条，它是任务 19167 退 2 留下的、没进 git 的残留，不整份覆盖）。产物头第一行 `E7RESULT name=config table_rows=<N> unit_test_modules=<六个模块名逗号连> timeout_seconds=300`，N 必须等于读法 2 的数据行数（登记时 57）；成功的产物行数 = 1（config）+ N（anchor）+ 1（baseline）+ N（mutation）+ 1（verdict）+ 1（done）= 2N + 4，登记时 118 行，末行 `E7RESULT name=done emitted=118`。这个式子拿原产物核过（第四节第 4 条）：`2×50+4 = 104`。
8. **装置一侧的修**：UT 红的那一条若定为装置要跟 `crates/`（第十一节 U1），修只许收严或补臂、不许放宽（原登记第十二节的规矩）：写进第十二节修订，先于任何一份产物，列每个改动文件的改前改后 sha256 与改了哪几行；扫描对「非仓根自身」的路径照旧要求罩住。

未定项：无。三行都能按上面的读法与第六节的门槛判；两处要主 agent 知道、不阻塞开跑的事写在报告「登记里待主 agent 认的项」里（第七个测试模块不在 `TEST_MODULES`；`.claude/gate.d/replay-comparison-units.tsv` 没有 E201 的行）。

## 二、被测条款与它引的定义

被测条款 `.claude/kb/decisions/13-验证路线.md` 的「已定项 6：三个互相独立的 oracle」（E201 实验页「影响的决策」表里它的关系是备料；这一次的三行只问装置此刻健不健康，不引条款、不改它的关系）。下面是 `scripts/kb/quote-kb.py` 整段抄出来的，回读逐字节一致：

**出处 `.claude/kb/decisions/13-验证路线.md:176-206`（整段抄，未转述）**

```markdown
#### 已定项 6：三个互相独立的 oracle

**定案**：三个 oracle 各判一样东西，独立性各有来处：

<!-- doc-lint:registry name-col=2 -->

| # | oracle | 判什么 | 独立性来自哪里 | 验不到什么 |
|---|---|---|---|---|
| O1 | 内存模型对拍（HashMap 理想文件系统） | POSIX 可见语义：内容、大小、目录结构；无崩溃处的内容期望归它，崩溃之后允许看到的集合归 RefFS（已定项 3） | 不碰盘、不解析磁盘格式 | 磁盘结构是否自洽；崩溃语义；**两份实现对同一条 POSIX 语义的共同误解** |
| O2 | 独立解析器 + checker | 磁盘字节是否满足 I-1（块自描述）~I-6（加密不变量） | **checker 不得调用被测 crate 的任何反序列化/遍历代码**，必须重新实现一份解析，直接读裸设备 | 只判结构自洽：一个块被覆盖成另一份**合法**内容，Merkle 链照样自洽、记账照样对 |
| O3 | 独立规约执行器 | 崩溃点截断后的每个磁盘状态是否落在规约允许的集合里 | **用另一种语言写**（不用 Rust），只照 decisions 与 invariants 的文字重新推导；第一版用 Python 单写（E170（O3规约执行器第一版）） | 规约本身可能写错——**必须配阳性对照才有意义**；被投影的六样的取值与六样之外的根记录字段、实现自己的恢复与读者代码、系统配置槽的见证值（journal_tail）、一次发布内各写的先后 |

**独立性要被验证，不许自称**：设计一个「三 oracle 合谋测试」——拿一个精心构造的坏实现
（例如 nonce 永远为 0）去过三关，若三者全部放行，说明存在共同盲区，回头补 oracle。
功能正确性不只押在这三个 oracle 上：O1（内存模型对拍（HashMap 理想文件系统）） 那一行已把崩溃之后允许看到的集合交给 RefFS（已定项 3），恢复函数的 M2–M4 落在 Verus 与 E165（恢复重放有界穷举与变异杀伤界）（已定项 1），需要第二个输入的核对归记录核对器（已定项 7）；三个 oracle 之间的共同盲区靠「三 oracle 合谋测试」找。

**射程**：三个 oracle 各自验不到什么写在表的最后一列；O2（独立解析器 + checker） 的定义域归已定项 7，需要第二个输入的核对归记录核对器。**实现今天的样子**：O2（独立解析器 + checker） 在 `crates/singlefs-checker`（只依赖格式常量 crate）；O1（内存模型对拍（HashMap 理想文件系统）） 的形态是 `crates/singlefs-verification/src/model.rs` 的理想模型（只住内存、只用 `singlefs_format` 的常量、不建崩溃），门禁 model-differential-and-scenarios 拿随机历史逐步对拍；O3（独立规约执行器） 是只照 kb 文字写的 Python 执行器，现行那一份是 E171（三oracle合谋测试） 第 1 次重跑用的 `scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates_journal_header_487/`（E170（O3规约执行器第一版） 第 2 次重跑从第 1 次重跑冻结的 `scripts/experiment-apparatus/e170_second_run_o3_spec_executor/` 拷出 `scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates/` 加谓词 W、T，再跟上 487 字节 journal 记录头；第 1 次重跑冻结的那一份与第一次那一份 `scripts/experiment-apparatus/e170_o3_spec_executor/` 留档）：严格跑法判得出（第一次导出的批 1、批 2 上 0 态撞「规约欠一句」），与实现逐态一致（两边全放行，不当它判得对的证据；「择到的根不是自证根里 txg 最大的那条」「③ 收到不少于 2 槽」两条路径批里 0 态，C661（规约执行器批里两条路径没有态）），坏状态表照改正的期望重判有判别力（阳性对照与盲测都判对），还没接进崩溃点重放；判一个崩溃态的做法：照规约的恢复规则（择系统配置、择根、前缀按 D23（journal 的角色与格式） 已定项 14 的五条口径）从镜像自己恢复出一版，恢复不出（整池拒或没有自证根）判不在；恢复出的这一版的投影（实例代号、checkpoint_txg、树表单元指针、中央映射树根指针、树 ID 水位、F）要等于版本表里某一版的投影，且这一版的下标在 [已确认, 已开始] 之间。版本表是基镜像上自己恢复出的现行那一版，加录制流里每一次根槽写按流序各一版，每版的投影取实现写进根槽的字节；已确认取崩溃态里落了的最晚一条写之前已返回的最晚一版（只从落了的写推），已开始取落了的写所属的最晚一版。系统配置槽只用来择槽、取根环与 journal 环的几何、判池级字段越界整池拒。所以恢复规则那一半是照 kb 文字另写的，允许集合那一半取自实现自己写出的根；RefFS（已定项 3）在 `crates/singlefs-reference-filesystem`，崩溃注入与故障注入的调用点已接；「三 oracle 合谋测试」是 E171（三oracle合谋测试），读数取自一棵冻住的树（快照摘要见实验页，与现行 `crates/` 不是同一份）：O1（内存模型对拍（HashMap 理想文件系统）） 一列是模型对拍加模型的崩溃判官，不接 RefFS，与表里 O1（内存模型对拍（HashMap 理想文件系统）） 那一行不同（崩溃之后的集合那一半 E171（三oracle合谋测试） 没用 RefFS）；O3（独立规约执行器） 一列用 E170（O3规约执行器第一版） 第 1 次重跑冻结的那一份；手造坏镜像那一族只喂 O2（独立解析器 + checker）、O3（独立规约执行器），期望出自 E170（O3规约执行器第一版） 的设计员，Q2 照登记字面判「有」（4 张全是 P8），按登记 F3 判成 E170（O3规约执行器第一版） 推期望推错、这四张不是坏镜像，真坏镜像里 O2（独立解析器 + checker）、O3（独立规约执行器） 两条全放行的 0 张；三条的源头是否独立（表第 4 列）E171（三oracle合谋测试） 只在 Q2 量过 O2（独立解析器 + checker）、O3（独立规约执行器） 一对，那一对的补丁与期望同出 E170（O3规约执行器第一版） 的改法代码，不当独立读数，O1（内存模型对拍（HashMap 理想文件系统）） 不在 Q2 里；没有量到的不写成「互相独立」被验证过。

**三条 oracle 都不判的两样**（第一样由崩溃注入与崩溃点重放两侧判，崩溃点重放那一侧的观察者在实现批）：一是无读故障的崩溃态上，D23（journal 的角色与格式） 已定项 14 可写挂载那条见证判据为真（系统配置槽里的见证值超前于盘上读得出的记录）：崩溃注入第二截判拒绝合不合法时只在崩溃镜像上复算这条判据，为真就判合法（挂载透明的射程只管不带故障的状态，已定项 1）；要有判官问「这个崩溃态没注入读故障，判据凭什么为真」：崩溃注入第二、三截在没注入读故障的崩溃态上复算这条判据，为真即判拒得不合法（`crates/singlefs-verification/src/crash_injection.rs` 的 `RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults`，C670（无读故障的崩溃态上可写挂载见证判据为真没有判官） 已还清，见欠账表已还清表）。「无读故障的崩溃态上每块盘至少一份自证过的系统配置」这一性质由崩溃点重放 mkfs 返回之后各条流上的观察者判：观察者按流登记，记录 mkfs 的流不挂，只记相对基镜像新出现的、一份自证过的系统配置都没有的盘，撕裂域在内，判槽自证读池级 checker 的逐盘数（`crates/singlefs-checker/src/image.rs` 的 `system_configuration_redundancy_report`），另有一条登记完整性用例判含系统配置槽写段的流都挂了它（实现在实现批）。随机历史里补写闸（D22（单元原子性怎么合成） 已定项 8）不过的那一形，由崩溃注入第二、三截的 G4 拒绝那一臂判；补写闸过了的那一形，随机历史里没人判，正确实现留不下这种盘面（两次抽样都没打中）。mkfs 返回之前的崩溃态不在观察者与 G4 拒绝那一臂的射程：mkfs 逐盘两槽在同一个循环里写、之后才逐盘屏障，崩溃注入的候选段与 E171（三oracle合谋测试） 的基镜像都取在 mkfs 之后；G4 拒绝那一臂与 C670（无读故障的崩溃态上可写挂载见证判据为真没有判官） 那条还有一处不同形：择不到系统配置时那条的复算交 `None`、G4 按真，这时读阶段本身已拒、到不了这一格。二是一次发布内各写的先后：O2（独立解析器 + checker） 的 I-7.12（系统配置 F 不低于同盘根上的 F） 判抬 F 那一串先写系统配置这一步；变异点名的用例只判 E171（三oracle合谋测试） 第 270 行那一种（发布返回之后每个崩溃点的见证值），第 242 行点名的用例判的是失败路径冻结；`layout/01-first-txn.md` 第八节把根槽 FUA 与系统配置槽轮换登记成两段，只是登记在册；其余写序不在三条之内。

**依据**：
- E170（O3规约执行器第一版）：O3（独立规约执行器） 照「另一种语言、只照 kb 文字重新推导、配阳性对照」写得出第一版，判得动单个崩溃状态、一态够快、每趟都能跑；第 1 次重跑（照 2026-10-03 的 kb 改过一次的执行器）严格跑法判得出、与实现在取样的批态上一致、阳性对照有判别力（第一次跑判不出与判别力不足那两笔留在实验页）。
- G4（无读故障的崩溃态上每块盘至少一份自证过的系统配置）：代码轮判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 j2-b 立、第 29 批实现，射程与 mkfs 那一段见 `research/prompts/m3-formal-code-b29-r1-main-verification.md` 第二节 L3 与第三节第 3 条。
- 两槽全废盘面改由崩溃点重放 mkfs 返回之后各条流上的观察者判、G4 拒绝那一臂留着：三方判决 `research/prompts/m3-close-g4-rewrite-r3-main-verification.md` 第二节 Y2 各行与第三节（前两轮 `research/prompts/m3-close-g4-rewrite-r1-main-verification.md`、`research/prompts/m3-close-g4-rewrite-r2-main-verification.md`）；观察者的接法（按流登记、相对基镜像、登记完整性用例）是用户定案 2026-10-22「按流登记+相对基镜像」，零轮，原话在变更史。数在三方腿的模型与副本原型上量，没立实验号。
- 三个 oracle 的分工与各自独立性的来处是验证体系的结构设计；独立性靠「三 oracle 合谋测试」观测（E171（三oracle合谋测试））。
- E171（三oracle合谋测试）：在一棵冻住的树上，坏实现表里有三条 oracle 全部放行的行、有只靠一条才抓到的行，O3（独立规约执行器） 在坏实现表上一行没红；手造的伪造根、伪造续接镜像里有 O2（独立解析器 + checker） 放行而 O3（独立规约执行器） 判不在的（读数与限定在实验页 Q1、Q2、Q3 三行）。
- 三方论证：D13（验证路线） 新立那一轮（本地 LLM / Sonnet / Opus 各一条腿，立场不重复），材料没有留存，经过在变更史。
- O1（内存模型对拍（HashMap 理想文件系统）） 与 RefFS 的分工句：三方判决 `research/prompts/m3-formal-r5-main-verification.md` 第二节另起与单独一个包那一格（两腿同向）。

**欠**：C445（模型对拍的取样点不随写路径增长）；C447（规约执行器没接进崩溃点重放与 RefFS 比对）；C407（三 oracle 合谋测试的判官没进门禁）：E171（三oracle合谋测试） 已量过一次，没有一格门禁读它的产物；C672（交回值或内存状态错而写流不变的一类）；C679（卸载记号没进模型对拍）

```

## 三、实现今天的样子

2026-11-18 现查，只读，没起任何项目代码。`git rev-parse HEAD` = `4319aa2b035b2d8238d64aeb68b3ceb7674cc723`。执行员开工时逐项再查，对不上的写进第十二节修订，不改判据。

- **变异驱动** `scripts/experiment-apparatus/e201_three_oracle_collusion_full_table_on_the_current_tree/mutation_driver.py`（150 行，sha256 `03763fedb26f7f1909134e566a4026d259d3f8776ec287f50b0bc23af79655e6`，git 里无改动）：第 32 行 `TEST_MODULES`（六个）、第 34 行 `TIMEOUT_SECONDS = 300`、第 38 行 `read_table`（跳 `#` 与空行）、第 82 行 `run_unit_tests`（子进程 `python -m unittest <模块…>`，环境多给 `E201_REPOSITORY_ROOT_FOR_A_COPY` 指真仓根）、第 94 行 `main`；第 105 行 `name=config`；第 113–115 行锚点不是恰好一次则写 `verdict anchors_not_exactly_once` 退 3；第 118–121 行写 `name=baseline unit_tests=<green|red>`、红则 `lines.finish()` 退 2；第 143 行 `name=verdict`。驱动不往标准输出写结果行，只写 `--output` 文件，所以任务日志是空的不说明它没跑。
- **装置目录的文件与 sha256 前 16 位**（`sha256sum *.py *.tsv`，git 里无改动）：`attribution.py` `8958eb366860f599`、`contrast_against_r1.py` `374020bf5aa29be0`、`current_tree_row_driver.py` `d39d11b0de45ec6b`、`device_steps_on_leased_cards.py` `8a0da01f750070f7`、`e171_research_side.py` `e2739c1199e9f01e`、`full_table_run.py` `900b1ecafa64b821`、`mutation_driver.py` `03763fedb26f7f19`、`q2_misses.py` `f49f7fc556cf19e3`、`whole_table_matrix.py` `cff9b3a00fc150e4`；测试 `test_attribution.py` `bb43dd182ca767d7`、`test_contrast_against_r1.py` `984fe2e845b49c4c`、`test_current_tree_row_driver.py` `148b1aa83b5583db`、`test_full_table_run.py` `9ddcf2fef9313ecd`、`test_q2_misses.py` `08ff76417f635f34`、`test_row_selection_and_reuse.py` `f82456253679cebe`、`test_whole_table_matrix.py` `4cae664fd431eb2b`；三份表 `cell_classes.tsv` `4c053fa2a7325ec1`、`device_function_map.tsv` `98aec2229ca37c8e`、`research_function_map.tsv` `9761c84f03d4295f`。
- **读真仓的那一条红单测** `test_full_table_run.py`：第 334 行 `repository_root_paths_named_outside_test_code`（扫 `crates/<包>/` 下非测试 `.rs` 里 `../` 或 `/../` 起头的字符串字面量，解到 `crates/` 之外的相对仓根路径，交回 `{路径: [出处]}`）、第 396 行 `paths_not_covered_by`（不等于、也不在某一项目录底下的路径）、第 401 行起 `RepositoryRootReferenceTests`，其中第 405 行起那一条先断言 `.claude/heavy-thread-default` 与 `scripts/crash/crash-point-replay-shard-configuration-check.sh` 在扫描结果里（判例，防扫法空过），再断言两份 `SYNCED_ENTRIES`（`full_table_run.py` 第 146 行、`current_tree_row_driver.py` 第 87 行，都是 `scripts/hosts/lib_repository_copy.py` 的 `copy_closure(...).entries` 现算）罩住扫到的每一条；第 414 行起另一条核那些 shell 脚本传递地 source、起的仓内文件也被罩住。
- **`crates/` 一侧**：`crates/singlefs-implementation-experiments/src/bin/e185_gc_batch_wall_clock_split/main.rs` 第 823 行 `const REPOSITORY_ROOT_OF_THE_BUILD: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");`（`#[cfg(feature = "gpu")]`，第 821 行文档注释写「编这份二进制的那棵树的仓根」）；第 840 行用法：`caller.multi_host_configuration_file(Path::new(REPOSITORY_ROOT_OF_THE_BUILD))?`。该函数在 `crates/singlefs-verification/src/gpu_card_configuration.rs` 第 822 至 859 行：只读环境变量 `SINGLEFS_MULTI_HOST_CONFIG` 指的文件，或 git 主工作树根下的 `multi-host.env`（第 813 行常量；git 忽略、每台机器一份）。`crates/singlefs-verification/Cargo.toml` 第 158 至 161 行 e185 是该包的另一个 bin，第 113 至 116 行才是 E201 装置编的 `e171_three_oracle_collusion_after_the_oracle_fixes`；后者源文件里 `CARGO_MANIFEST_DIR`、`multi_host_configuration_file` 零命中（`grep -n`，第四节第 6 条）。
- **变异表**：`scripts/mutation/tables/e201_three_oracle_collusion_full_table_on_the_current_tree.tsv` 65 行（8 行注释、57 个数据行，`awk` 数）；第 51 至 57 个数据行是原产物之后加的（名字以「复用」起头六条、「判法」一条），原产物只有 50 行。
- **统一任务服务**：单测与驱动都是单进程 python，不分片、不要卡、不编 `crates/`；`.claude/heavy-thread-default` 登记值 31（这两样都不并行）。

## 四、跑之前已经存在的数

设计时读到或自己算出的，照实列，不删。标「量的」的是我用只读命令（`ls`、`wc`、`grep`、`sha256sum`、`awk`、读文件）当场取的，标「推的」的是由量的推出的、没跑过。

1. **任务 19166**（六个单测模块，在装置目录里；调度给的，我读了整份日志，2658 字节）：量的，末尾 `Ran 121 tests in 12.753s`、`FAILED (failures=1)`；红的是 `test_full_table_run.RepositoryRootReferenceTests.test_every_repository_root_path_the_crates_name_outside_test_code_is_copied_and_synchronized`，断言 `paths_not_covered_by(named, run.SYNCED_ENTRIES)` 得 `['.']`、期望 `[]`，`named` 里 `'.'` 的出处只有一条 `crates/singlefs-implementation-experiments/src/bin/e185_gc_batch_wall_clock_split/main.rs:872`；另外三个名字（`.claude/heavy-thread-default`、`scripts/crash/crash-point-replay-shard-configuration-check.sh`、`.claude/singlefs-ai-sop/scripts/preflight.py`）都在、都被罩住。六个文件里 `def test_` 共 121 行（量的，`grep -c` 各文件 3、13、8、62、16、19 相加），与 `Ran 121` 相同，所以别的 120 条过（推的，只有一条 F）。日志里没有 `-v` 逐条名，「`test_the_two_function_maps_are_the_registered_ones` 过」是推的，要这一次的 `-v` 输出核（第六节 UT-钉值）。
   影响：UT 此刻的读数是「红一条」，但这个读数在登记写之前就存在；UT 的正式读数是登记之后执行员那一次（第五节次序第 1 步），19166 不算数，只当线索。
2. **任务 19167**（变异驱动）：量的，退 2，日志 0 字节（`ls -l`）。驱动只写 `--output` 文件，不往标准输出写结果行（第三节），所以空日志不说明没跑。`research/results/e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-18-mutations.out` 在，没进 git（`git status --short` 报 `??`），60 行（量的 `wc -l`）。我读了它的头 5 行：`name=config table_rows=57 unit_test_modules=<六个> timeout_seconds=300`，与四条 `name=anchor … hits=1`（V1 至 V4）；其余行没读，也没读它末行。
   推的：60 = 1（config）+ 57（anchor）+ 1（baseline）+ 1（done），而驱动只有「基线红」这一支（第 119–121 行）会在写完 anchor 之后不写 mutation 行就收尾，所以 19167 退 2 的原因是基线单测红、且 57 条锚点都恰好命中一次（命中数不是 1 走第 113 行退 3，不会走到基线）。这条推论在 DR 那一行里用命令核（第六节 DR-原因）。
3. **原产物** `research/results/e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-11-mutations.out`：量的，sha256 `1342dab8ba56f52b75c0cf89c294b9708410b9c148d497c0ddab4269f5c0233d`，104 行；我读了末两行：`E7RESULT name=verdict mutations=50 caught=50 invalid=0 survived=0 timed_out=0 restored_and_baseline_green=true`、`E7RESULT name=done emitted=104`（与实验页第 41 行同一句）；实验页第 41 行还写「任务 13144 `Ran 115 tests`、`OK`」。其余行没读。
   影响：这是表还只有 50 行时的读数，今天表 57 行、单测 121 条，树也动过；门槛写的是「57 行全 ✅」，不是「与原产物同」。原产物只在第 4 条里当产物行数式子的校核，和第六节 MT-附带里按变异名逐行并排的另一边。
4. **产物行数式子**：量的，原产物 1+50+1+50+1+1 = 104，与 `wc -l` 的 104 相同；今天 N = 57 代入得 118（推的）。
5. **钉值**：量的，`cell_classes.tsv` 今天 sha256 `4c053fa2a7325ec16a7983121231ade70ca5269547baff922654162aca93d39a`、`research_function_map.tsv` `9761c84f03d4295f28cf157279bf9e09010e0f0aee84049141d8e8f017d56a6a`，与代码里钉的（`contrast_against_r1.py:31`、`attribution.py:48`）逐字相同；`device_function_map.tsv` 今天 sha256 `98aec2229ca37c8e…`（前 16 位），代码钉的是 `95b93eb4bf7be327215bbfcdf80bdaf890d6a36e3db1ea9be2edcc955cbce0b9`（`attribution.py:47`），**不相同**。`attribution.py:116–128` 的 `load_function_map` 认两种：此刻 sha256 等于钉值，或钉值在「照 `.claude/gate.d/path-moves.tsv` 把此刻内容逐次倒换回去」的各版 sha256 里。量的：`path-moves.tsv` 里含 `singlefs-harness` 的行 445 条（例：第 193 至 195 行，`singlefs-harness` 改成 `singlefs-verification`，2026-11-05）；`device_function_map.tsv` 里含 `singlefs-verification` 的行 1 条、含 `singlefs-harness` 的行 0 条。推的：今天的表是搬迁之后的内容，倒换回去才是钉值那一版，所以不等是预期的、不是被改过；这是推的，没算过 sha256，不当「对得上」。真读是 UT 里 `test_attribution.py:162` 的 `test_the_two_function_maps_are_the_registered_ones`（对真表调 `load_function_map`，再核错的钉值会红）与 `:168` 那一条手造的搬迁用例。
6. **设计时读到的 `.` 的来历**（读代码、没跑，是这一次 UT 红的另一半答案，如实记）：量的，`e185_gc_batch_wall_clock_split.rs:823` 是 `concat!(env!("CARGO_MANIFEST_DIR"), "/../..")`，字面解出来恰是仓根自身，后面没有再接字面子路径；第 840 行只把它交给 `multi_host_configuration_file`，那个函数只读 `SINGLEFS_MULTI_HOST_CONFIG` 或 `multi-host.env`（第三节），两样都是 git 忽略、每台一份、不拷进副本的（E201 实验页「口径」：`multi-host.env` 不拷，副本里起的命令显式带主工作区那一份的路径）；E201 装置编的 `e171_three_oracle_collusion_after_the_oracle_fixes` 源文件里 `CARGO_MANIFEST_DIR`、`multi_host_configuration_file` 都是零命中（`grep -n`，`ugrep` 没报错）；`git log -S'REPOSITORY_ROOT_OF_THE_BUILD'` 点名提交 `490168e59`（2026-11-17）。推的：`.` 是「仓根自身」，不是某个要拷的文件，罩住的应是它下面读的文件，而它下面读的 `multi-host.env` 本来就按设计不拷。
   影响：这把 UT 红那一条的归类往「装置一侧的扫描要跟」（U1）推。判据不由它来：U1、U2、U3 写成可核的条件（第十一节），执行员逐条现查；任何一条查出来与这一条推的不同，按查出来的办。
7. **判别力（第 3a 步 ②）**：三行各自分辨的差都是整条数或一个退出码：MT 在「全部抓到」那一臂上 `caught=57` 其余 0，在「有行不是抓到」那一臂上 `caught<57`；UT 在「全过」臂上 failures=0 且 errors=0，在「有红」臂上其和 ≥ 1；DR 在「起得来」臂上退 0 或 1 且有 `done` 行，在「起不来」臂上退 2、3 或别的码。分辨率是一条用例、一个整数退出码，不用比例、比值，在任何一臂的基线上都不退化。
8. **可跑性（第 3a 步 ①）**：单测 1 次 = 12.753 秒（量的，19166）；驱动单测次数 = 1（基线）+ 57（变异）+ 1（收尾基线）= 59，推的 59 × 12.753 ≈ 752 秒，加每次拷装置目录与 `py_compile`（目录 896K，量的 `du -sh`），整趟推 13 至 15 分钟；每条变异的副本用完即删，同一时刻磁盘上只有 1 份副本，峰值占用小于 10 MB（推的）；内存：单个 python 进程，没量过，推不超过 1 GB。
9. **要触发的事件（第 3a 步 ③）**：不适用，这一次没有以搬空、回收、淘汰、写满为前提的量。

## 五、臂、阳性对照、真实基线

### 5.1 臂（岔路单每一行的候选，对面那条写成支持它的人认的样子）

| 行 | 候选 | 支持它的人会要求（怎么做 → 做完会怎样） |
|---|---|---|
| MT | **全部抓到** | 驱动对表里每一行都把替换文写进一份新副本、编得过、单测退出码非 0；读产物，57 行每行都是 ✅，不拿 ⏭ 当抓到，不拿「基线绿」代替逐行结局，收尾要还原并核基线仍绿、装置目录跑前跑后逐字节相同（`restored_and_baseline_green=true`）。做完 `caught=57`、其余三栏 0 |
| MT | **有行不是抓到** | 逐行列出 ⏭、❌、⏱ 的行与行名，每条 ❌ 照 `mutation-sampling.md`「没红的三类」找一个换了就不同值的取样点后再分类，每条 ⏭ 照第八类记成因（替换文编不过），不直接写「盲区」「等价」；退 3（锚点失效）时一行都没跑，记「没跑」，不算进任何一栏 |
| UT | **全过** | 六个模块在今天的树上一起起，`OK`，条数等于 `def test_` 行数；红过的那一条要么修好再起一次、再 `OK`，要么写明为什么停 |
| UT | **有红** | 逐条列红的用例名与断言原文，不把「红」写成「装置坏了」：红的那一条先分清是装置的扫描要跟 `crates/`、还是 `crates/` 一侧新写的路径不该出现（第十一节 U1、U2、U3），分不出的写「分不出」 |
| DR | **起得来** | 驱动在装置目录里起，退 0 或 1，`--output` 文件里有 `verdict` 与 `done` 行，头一行 `table_rows` 等于表的数据行数 |
| DR | **起不来** | 退 2、3 或别的码；写出原因（基线红、锚点失效、起不了 python、写不了产物），原因要由命令核出来，不由退出码猜 |

### 5.2 阳性对照（每一条臂都跑）与阴性对照

对照表（控制表）写在草稿目录 `<草稿目录>/control.tsv`，三列同真表（变异名、原文、替换文，制表符分隔，`\n` 还原成换行），三行，原文都是 `contrast_against_r1.py` 第 31 行那一句 `CLASSES_SHA256 = "4c053fa2a7325ec16a7983121231ade70ca5269547baff922654162aca93d39a"`（量的：在装置目录非测试、非驱动的 `.py` 里恰好命中一次，另外七个源文件零命中）：

| 控制行 | 替换文 | 必须看到 | 臂 |
|---|---|---|---|
| 控制·抓到 | 把钉值首位 `4` 换成 `0`（`CLASSES_SHA256 = "0c053fa2…"`，其余 63 位不动） | ✅ ，`unit_test_exit` 非 0（`test_contrast_against_r1.py` 第 120 行 `test_the_classes_table_is_the_registered_one` 的 `assertEqual` 红） | MT 全部抓到；UT 有红（单测在被弄坏的副本上会红） |
| 控制·没红 | 替换文就是原文 | ❌，`unit_test_exit=0` | MT 有行不是抓到（没红） |
| 控制·无效 | `CLASSES_SHA256 = (`（`py_compile` 编不过） | ⏭，`unit_test_exit=-1` | MT 有行不是抓到（无效） |

- 控制行跑在真驱动上：`mutation_driver.py --table <草稿目录>/control.tsv --scratch <草稿目录>/scratch-control --output <新文件>`。控制产物必须是：头一行 `config table_rows=3 …`，三条 `anchor … hits=1`，`baseline unit_tests=green`，三条 `mutation` 行依次 ✅、❌、⏭，`verdict mutations=3 caught=1 invalid=1 survived=1 timed_out=0 restored_and_baseline_green=true`，末行 `done emitted=10`（1+3+1+3+1+1），驱动退 1（有没红）。这是 DR 起得来、MT 两臂、UT 有红三条的阳性对照，跑在基线绿之后（UT 修好之后，第 5.6 节次序）。
- **DR 起不来那一臂**的阳性对照：同一张控制表再跑一次，多给 `--unit-test-module test_no_such_module_for_the_control`（基线单测起不来、红），必须看到：驱动退 2，产物 `config table_rows=3`、三条 `anchor`、`baseline unit_tests=red`、`done emitted=6`，没有 `mutation` 与 `verdict` 行。它证明读 19167 那一类产物的读法（行数、末几行）分得出「基线红」，也是 UT 有红臂的又一个红例。
- **UT 全过那一臂**的阳性对照：控制·没红那一行与控制·抓到那一行共用的 `baseline unit_tests=green`、`restored_and_baseline_green=true`（驱动在原样装置上两次基线都绿），加 UT 正式读数的 `OK`。
- **阴性对照**：没动装置、没动表的两次基线（控制产物与正式产物的 `baseline` 行与收尾还原基线）都绿；控制·没红的 `unit_test_exit=0` 也是同一条。阴性对照上有任一处红：不是结果，是装置或树不稳，走 F2。
- 两份控制产物进 `research/results/`（名字照 5.6），不进判据，只给读正式产物的人看管子是通的。

### 5.3 真实基线

MT 的真实基线是原产物 `…-2026-11-11-mutations.out`（50 行表，全抓；第四节第 3 条）：正式产物按变异名与它逐行并排，只作第六节 MT-附带，门槛不看它；DR 的真实基线是任务 19167 那一次（退 2）；UT 的真实基线是任务 19166 那一次（红一条）。三条真实基线都是「上一次是什么样」，这一次不按它们判，只并列。

### 5.4 装置写在哪（写死）

**研究一侧**，即 `scripts/experiment-apparatus/e201_three_oracle_collusion_full_table_on_the_current_tree/` 里已有的 Python 单测与 `mutation_driver.py`。这三行问的是装置自己的单测与变异驱动健不健康，另写一份独立模型答不了「驱动此刻起不起得来」，但也不是在问 `crates/` 这份代码的性质，所以不用入库装置、不改 `crates/` 一行。不编 Rust，不经判段入口，不起 GPU：登记里没有 `cargo`，`--features gpu`、`NOT_RUN` 计数与卡的约束都不适用（不走崩溃判段）。执行员唯一可能改的文件是 UT 红那一条的出处 `test_full_table_run.py`（只在第十一节 U1 成立时，写进第十二节），别的装置文件、表与 `crates/` 一概不改；改了的话产物前先记新的 sha256。

### 5.5 复用什么、不复用什么

原样用：驱动、六个单测模块、变异表（读法 2 记的 sha256）、`e171_research_side.py`（驱动 import）。不复用 E171（三oracle合谋测试）的修订 8 那一层，也不用整表两段的任何入口。控制表是这一次新写的，只在草稿目录与产物里，不进 `scripts/mutation/tables/`。

### 5.6 谁跑什么、次序、实现量、分段

命令都由主 agent 经统一任务服务交（执行员写进「要主 agent 跑的命令」一节）；`<装置目录>` 是 `scripts/experiment-apparatus/e201_three_oracle_collusion_full_table_on_the_current_tree` 的绝对路径，`<草稿目录>` 在共享工作根下。三条命令都是单进程 python，不套包装、不写线程与内存参数；范围声明闸若拒（它认 `replay.sh` 等入口，没认这几条的形状），照拒绝说明补四段声明（种类 `target`），不换写法。

| 步 | 谁 | 做什么 | 补哪一行 |
|---|---|---|---|
| 0 | 执行员，只读 | 核第三节每个 sha256 与行数（装置文件、表、`path-moves.tsv`、`lib_repository_copy.py`），不同的写进第十二节；记树摘要 T0（命令见第八节）；`ls research/results/ \| grep e201` 记已有产物 | 全部 |
| 1 | 主 agent 交 | `task-server/task submit --cwd <装置目录> -- python3 -B -m unittest -v test_contrast_against_r1 test_q2_misses test_attribution test_whole_table_matrix test_current_tree_row_driver test_full_table_run`；另一条 `… -- python3 -B -m unittest -v test_row_selection_and_reuse`（附带） | UT |
| 2 | 执行员 | 把步 1 的日志（`task-server/task log <号> --all`）落进 `research/results/e201-…-<日期>-r1-unit-tests-before.out`；全过则跳到步 4；有红则照第十一节 U1 至 U3 分类 | UT |
| 3 | 执行员 | U1：改 `test_full_table_run.py`、写第十二节修订、补手造用例，然后再交步 1 那一条，落 `…-r1-unit-tests-after.out`，必须 `OK`；U2 或 U3：停，交主 agent，步 4 至 6 不跑 | UT、DR |
| 4 | 主 agent 交 | 两条控制：`… --cwd <装置目录> -- python3 mutation_driver.py --table <草稿目录>/control.tsv --scratch <草稿目录>/scratch-control --output <仓根>/research/results/e201-…-<日期>-r1-control-mutations.out` 与同一条加 `--unit-test-module test_no_such_module_for_the_control`、`--output …-r1-control-red-baseline.out` | DR、MT 管道 |
| 5 | 主 agent 交 | 正式：`task-server/task submit --cwd <装置目录> -- python3 mutation_driver.py --table ../../mutation/tables/e201_three_oracle_collusion_full_table_on_the_current_tree.tsv --scratch <草稿目录>/scratch-formal --output <仓根>/research/results/e201-three-oracle-collusion-full-table-on-the-current-tree-<日期>-r1-mutations.out`（`<日期>` 用 `TZ=Asia/Tokyo date +%F`） | MT、DR |
| 6 | 执行员 | 树摘要 T1；读产物（第六节 MT 各量）；MT-附带的并排；没红与无效逐条分类；回对实验页里引到的数 | MT |

**实现量**：不写新装置；至多改 `test_full_table_run.py` 一处与补两三条手造用例（U1 时）；控制表三行。一个执行员一次做得完，不分执行员。

**分段**（每段交回之后主 agent 对着岔路单判续不续）：

| 段 | 做什么 | 补哪一行 | 补完够不够判 |
|---|---|---|---|
| 第一段 | 步 0 至 3 | UT 全部；DR 的「原因」那一半 | UT 够判（全过，或红已分类停下）；DR 要等第二段出产物才够；MT 不够 |
| 第二段 | 步 4 至 6 | MT 全部；DR 的「出产物」那一半 | MT 够判（57 行结局与三个数落定）；DR 够判 |

第一段落在 U2 或 U3 的，第二段不跑：UT 的答案是「有红，且是 `crates/` 一侧的事」，MT 不够判，DR 的原因已定出（基线红），标「够判后未跑」不适用、标「主 agent 定不跑」。

## 六、报哪些量与各自的判据

每个量占一行、各报各的判定，不合取（`test-discipline.md`「判据不写成合取」）。MT 那几行没有数值门槛：问的是「每一行各是什么」，答案就是 57 个结局符号，候选读符号定，门槛要是写成「`caught=57`」就是臂的定义本身（`test-discipline.md`「门槛不能是被测臂的定义」），所以 MT 只设有效性检查（行数、锚点、基线、还原、两种数法一致），这些检查不是从臂的定义推出来的。「命令」一栏都在仓根起，取数不跑项目代码的直接跑，起装置与驱动的由主 agent 经服务交（第五节 5.6）。

| 量 | 怎么算 | 门槛或核对 | 判定 | 挂哪一行、取什么值翻面 |
|---|---|---|---|---|
| UT-结局 | 步 1 的 `-v` 日志末尾：`Ran N tests`、`OK` 或 `FAILED (failures=a, errors=b, skipped=c)` | 末行是 `OK` | 是：全过；不是：有红 | UT。翻面：任何一条红（failures 加 errors ≥ 1）；登记时已读到 19166 的 1 条，但那一次在登记之前，这一次重读 |
| UT-条数 | `Ran` 的 N 与 `grep -c 'def test_'` 六个文件的和比（登记时 121 = 3+13+8+62+16+19，每个文件的数在第四节第 1 条） | N 等于和；U1 补了手造用例的，和加上补的条数 | 相等：没有漏起；不等：UT 的读数作废，查为什么（模块没起、用例被重名覆盖） | UT 的有效性检查，对应「六个模块全过」要先确认六个模块真起了 |
| UT-跳过 | 日志里 `skipped=` 或 `... skipped` 行数 | 0 | 非 0：该条不算过，列名字 | UT；一条跳过的用例没判，不算「全过」 |
| UT-红的清单 | 红的用例全名与断言原文（`AssertionError` 那几行原样贴） | 无 | 逐条列 | UT「有红」那一臂要的东西 |
| UT-分类 | 对每一条红，照第十一节 U1、U2、U3、新红 判，命令输出整段贴进第十二节 | 每条红恰落一类 | 落 U1：装置的扫描要跟；落 U2：`crates/` 一侧或拷贝范围要动，停；落 U3：分不出，停；新红：不是 19166 那一条，停 | UT 的够判条件「定出是装置要跟还是 `crates/` 一侧新写的路径不该出现」 |
| UT-钉值 | 步 1 日志里这三条各自的结局：`test_the_two_function_maps_are_the_registered_ones`、`test_a_map_that_differs_only_by_registered_path_moves_matches_the_pin_after_reversing_them`、`test_the_classes_table_is_the_registered_one` | 三条都 `ok` | 都 ok：三份表的钉值此刻对得上（含对应表经搬迁登记倒换那一条）；有一条不是 ok：走停机 S2 | UT；派发提示里「钉值对不上怎么办」那一问 |
| UT-附带 | 步 1 另一条 `-v` 日志：`test_row_selection_and_reuse` 七条用例的结局 | 无门槛，只报 | 附带，够判后不跑：这一次起一次，不进 UT 的判定，不进 `TEST_MODULES` | 不挂岔路单行；顺带量，因为它在装置目录里却不在 `TEST_MODULES` 里 |
| DR-原因 | 读 19167 留下的 `research/results/e201-…-2026-11-18-mutations.out`（60 行，没进 git）：`grep -c 'name=anchor'`、`grep -c 'hits=1 '`、`grep -c 'name=baseline unit_tests=red'`、`grep -c 'name=mutation '`、末行；再 `task-server/task status 19167` 与 `task-server/task log 19167` | 期待值（推的，第四节第 2 条）：57、57、1、0、`done emitted=60`；这几个数是推出来的预期，不是门槛，对不上就是原因另有 | 全对上：原因是基线单测红，与 UT 同一件事；对不上：原因另查（读产物末几行与 `task status`，不凭退出码猜），写出查到的 | DR。翻面取值：原因不是「基线单测红」 |
| DR-原因同源 | 步 5 正式产物的 `name=baseline unit_tests=` 一行 | `green` | green：UT 修好之后驱动的基线也绿，退 2 的原因就是 UT 红（去掉原因，症状没了）；仍是 red 而装置目录里 UT 已绿：副本里的基线红，另一个原因，走 F5 | DR；够判条件「修到驱动出产物」 |
| DR-产物 | 步 5 任务的 `TASK_SERVER_RESULT` 行 `exit=`，产物是否存在，`wc -l`，头一行，末两行 | `exit` 为 0 或 1；产物头一行 `table_rows=57` 与读法 2 的数据行数相同；行数 = 2×57+4 = 118；末行 `done emitted=118` | 全满足：起得来；退 2 或 3、或行数不对：起不来或产物不完整，走 Z4 | DR。翻面：退 2、3 或别的码 |
| DR-控制 | 步 4 两份控制产物 | 5.2 写的样子（10 行 / 6 行，退 1 / 退 2，三个符号依次 ✅ ❌ ⏭） | 对：管子通；不对：F1 | DR、MT 的阳性对照，两条臂都有 |
| MT-行数 | 产物头一行 `table_rows` 与 `grep -v '^#' <表> \| grep -vc '^$'` | 两数相等（登记时 57） | 相等：产物说的是这张表；不等：Z1 | MT 的有效性检查 |
| MT-锚点 | `grep -c 'name=anchor'` 与 `grep -c 'hits=1 '` | 都等于 MT-行数 | 都等：每一行的原文在今天的装置里恰好命中一次；不等：驱动退 3、没有一行变异跑过，MT 记「没跑」，F3 | MT；锚点失效的行不进任何一栏（`mutation-sampling.md` 第七类） |
| MT-基线 | `name=baseline unit_tests=` 与 verdict 的 `restored_and_baseline_green` | `green`、`true` | 都过：副本里的基线与收尾还原都绿；不过：F2 | MT 的有效性检查（阴性对照） |
| MT-逐行 | 57 条 `name=mutation number=<k> outcome=<符号> mutation=<变异名> unit_test_exit=<码>` 全部列出，按表里的次序 | 变异名按序等于表第一列（`awk -F'\t' '!/^#/ && NF {gsub(/ /,"_",$1); print $1}'`，驱动的 `field()` 把空白换成下划线；与产物里 `name=mutation` 行的 `mutation=` 字段逐行对，57 行、无重名无缺行；登记时表里无重名，量的 `uniq -d` 得 0） | 对上：行集对；对不上：F3 | MT。每一行各是 ✅、⏭、❌、⏱ 之一，这就是答案；任何一行不是 ✅ 即翻面 |
| MT-三个数 | verdict 行的 `caught`、`invalid`、`survived`、`timed_out`；另用 `grep -c '✅'` 之类对 57 条 `mutation` 行各数一次 | 四栏之和等于 57；两种数法逐栏相等（校验：两条路子不共用同一段代码） | 相等：三个数落定（`timed_out` 为 0）；不等：产物自相矛盾，Z4；`timed_out` 非 0：那几行没落定，F4 | MT 的够判条件「三个数各自落定」 |
| MT-没红分类 | 每条 ❌：改的是哪个文件哪一处、原文与替换文，哪些测试盯着这一处（`grep` 点名），换一个取样点（能让这一处输出不同的输入）找到没有 | 无门槛，每条写出一个具体的取样点或写「没找到」 | 按 `mutation-sampling.md` 三分：真盲区、等价、取样点不敏感；不确定的写「不确定」，不记盲区或等价 | MT「有行不是抓到」那一臂；没有 ❌ 时这一行不出 |
| MT-无效成因 | 每条 ⏭：替换文为什么编不过（`py_compile` 的报错原文） | 无 | 逐条写；无效不算抓到也不算没红，不进「全抓」 | 同上；没有 ⏭ 时不出 |
| MT-附带 | 正式产物的前 50 条按变异名与原产物 `…-2026-11-11-mutations.out` 的 `mutation` 行并排，列出结局不同的名字；后 7 条标「原产物没有这一行」 | 无 | 附带，够判后不跑：只并列、不进 MT 的判定 | 不挂岔路单行 |

## 七、钉绝对值的断言

**出自被测条款本身的**：无。被测条款（`13-验证路线.md` 已定项 6）讲三个 oracle 的结构，不含这一次的任何数；这一次的三行问的是装置健不健康，不是条款。这一类不适用，没有「不符走条款可能错」的锚点。

**独立算出、用命令核过的**（不符才作废，对应第十一节的作废条款）：

| 锚点 | 值 | 怎么独立算、核过的命令 | 不符时 |
|---|---|---|---|
| 变异表数据行数 | 57 | `grep -vc '^#' <表>` 得 57，`awk -F'\t' '!/^#/ && NF {n++} END {print n}'` 也得 57（两条路子，2026-11-18 现跑）；表文件共 65 行（`wc -l`），8 行注释 | 产物头 `table_rows` 不是它：Z1 |
| 六个单测模块的条数 | 121 | 每个文件 `grep -c 'def test_'`：`test_whole_table_matrix.py` 3、`test_contrast_against_r1.py` 13、`test_q2_misses.py` 8、`test_full_table_run.py` 62、`test_attribution.py` 16、`test_current_tree_row_driver.py` 19，相加 121；与 19166 的 `Ran 121 tests` 独立相符（一边数源文件、一边数 unittest 实际起的） | `Ran` 的 N 不等于它（U1 补过用例的加上补的条数）：UT-条数作废 |
| 成功产物的行数 | 2N+4，N = 57 时 118，`done emitted=118` | 原产物 N = 50 时 104 行，`wc -l` 得 104，`emitted=104`（量的）；式子 1+N+1+N+1+1 对着驱动 `main` 的写行次序数（`mutation_driver.py` 第 105 行 `config`、第 110 行每条 `anchor`、第 118 行 `baseline`、第 136 行每条 `mutation`、第 143 行 `verdict`、第 145 行 `finish` 写 `done`；第 113 行的 `verdict anchors_not_exactly_once` 与第 120 行的收尾是退 3、退 2 的两支） | 退 0 或 1 的产物行数不是它：Z4 |
| 控制产物的行数 | 10（抓到、没红、无效三行），6（基线红的那一份） | 同一个式子，三行表：1+3+1+3+1+1 = 10；基线红：1+3+1+1 = 6 | 不是：F1 |
| 两份表的钉值 | `cell_classes.tsv` `4c053fa2a7325ec16a7983121231ade70ca5269547baff922654162aca93d39a`、`research_function_map.tsv` `9761c84f03d4295f28cf157279bf9e09010e0f0aee84049141d8e8f017d56a6a` | `sha256sum` 现算，与 `contrast_against_r1.py:31`、`attribution.py:48` 钉的逐字相同（2026-11-18 现跑） | 不同：停机 S2 |
| 对应表的钉值 | 代码钉 `95b93eb4bf7be327215bbfcdf80bdaf890d6a36e3db1ea9be2edcc955cbce0b9`，文件今天 `98aec2229ca37c8e…` | **没独立算过**：靠 UT-钉值那三条单测对真表跑 `load_function_map`；不把「倒换后对得上」当已核的锚点 | UT-钉值有一条不是 ok：停机 S2 |

互比断言旁的绝对值：MT 的「两种数法逐栏相等」（`verdict` 行对 `grep -c` 各符号）旁钉着「四栏之和 = 57」，57 是表的行数，不是从产物里取的。

## 八、轨迹与几何敏感性

**轨迹**：这一次没有被某条条款当停机、起跑、准入谓词输入的数量轨迹；终局计数（三个数）不是被谓词消费的轨迹量，逐行结局 57 条全报。唯一被谓词消费的量是「树摘要」：作废 Z3 比跑前跑后的两个值，所以报三个值——步 1 之前的 T0、步 5 之前的 T1、步 5 之后的 T2（步 3 起了单测的，步 3 前后各一个）。命令（仓根，只读）：

```
{ find crates -type f \( -name '*.rs' -o -name Cargo.toml \) -not -path '*/target/*' -exec sha256sum {} + ; sha256sum Cargo.toml Cargo.lock .claude/heavy-thread-default .claude/gate.d/path-moves.tsv scripts/crash/*.sh scripts/hosts/*.sh scripts/hosts/*.py ; } | LC_ALL=C sort -k2 | sha256sum
```

执行员步 0 先核这条命令点名的每个文件都在（`sha256sum` 不报「没有那个文件」），不在的从命令里去掉、写进第十二节，之后三个值用同一条。六个模块读真仓的东西还有 `.claude/scripts/preflight.sh` 与 `.claude/singlefs-ai-sop/scripts/preflight.*`（19166 日志 `named` 里点名）：执行员把它们也加进命令，记进第十二节。

**几何敏感性**：这个实验的「几何」是两个旋钮，都不是被测条款给的：驱动的单测集合（`TEST_MODULES` 六个）与超时（`TIMEOUT_SECONDS` 300 秒）。正式判定取在「六个模块、300 秒、今天的树」这一个点上；方向相反的取样点两个，只在出现对应结局时才跑：

| 取样点 | 什么时候跑 | 怎么跑 | 报什么 |
|---|---|---|---|
| 测试集合更大（六个加 `test_row_selection_and_reuse`） | MT 里有 ❌ 的行 | 草稿目录里一张只含那一行（原文与替换文原样）的表，驱动加 `--unit-test-module` 七次给七个模块，其余同 5.6 步 5 | 这一行换成七个模块之后是 ✅ 还是仍是 ❌；翻面记「对测试集合敏感」，并列写，不改正式判定 |
| 超时放宽（草稿副本里把 `TIMEOUT_SECONDS` 改成 1200，只给这一次用，不动仓里的驱动） | MT 里有 ⏱ 的行 | 同上，一张只含那一行的表，驱动跑在草稿副本里 | 放宽后是 ✅、❌ 还是仍 ⏱ |

没有 ❌、⏱ 的，这两行写「没有触发，不跑」，不是欠账。单测一次 12.753 秒对 300 秒的余量是 23 倍多（推的，12.753 × 23.5 ≈ 300），正常不会 ⏱；出现 ⏱ 多半是机器忙，重交同一条命令一次，两次都 ⏱ 才记 ⏱（F4）。

MT 与 UT、DR 都是确定性的：驱动与 unittest 没有随机源、没有时钟判据，没有隐藏状态，唯一的环境依赖是真仓的树（读法 5），所以不按「至少 5 轮」重复，证据强度靠控制行与钉绝对值；不稳定的唯一来源（别的会话改了树）由 Z3 判。

## 九、变异

被跑的变异就是变异表的 57 行（不增不减，不加新行）。每行在哪个取样点上改变输出，记在两处：V1 至 V14 在原登记第九节（`research/prompts/e201-preregistration.md` 第 513–532 行）；第 15 至 41 行（以「落地」起头）在原登记修订 3 的落地清单与变异表自带的注释里，每行红在哪个单测写在那里。下面只列原登记里没有逐行写取样点的：第 42 至 57 行。「取样点」一栏是我从源码读出来的预期（推的，没跑），产物才是判定：任何一行与预期不同按 MT 的候选读，不回头改预期。

| # | 变异名 | 改了什么 | 在哪个取样点上改变输出（预期，推的） |
|---|---|---|---|
| 42 | 落地 计划拷树不走共用库的闭包 | `full_table_run.py` 的 `SYNCED_ENTRIES` 由 `copy_closure(REPOSITORY_ROOT).entries` 改成 `lib_repository_copy.COMMON_ENTRIES` | 闭包比共用清单多的那几项：`test_full_table_run.py` 第 265 行断言 `"prune-server" in run.SYNCED_ENTRIES`，变异下 `prune-server` 不在 |
| 43 | 落地 线道树同步不走共用库的闭包 | `current_tree_row_driver.py` 第 87 行同样改成 `COMMON_ENTRIES` | `test_current_tree_row_driver.py` 第 45 行断言 `"prune-server" in driver.SYNCED_ENTRIES`，取样点同上 |
| 44 | 落地 入口不交多机配置 | `full_table_run.py` 里 `os.environ.update(lib_repository_copy.environment_for_commands_in_the_copy(REPOSITORY_ROOT, os.environ))` 改成 `pass` | 在副本里起的命令的环境：该带上的多机配置环境变量，变异下没有（假驱动记下的环境） |
| 45 | 落地 逐行驱动不交多机配置 | `current_tree_row_driver.py` 里同一句改成 `pass` | 同上，逐行驱动那一层 |
| 46、47 | 落地 逐行驱动线道数默认写死 1、不给线道数时只开一条 | `--lanes` 的默认值 `None` 改成 `1`；`lanes = max(1, registered_threads // max(1, …))` 改成 `lanes = 1` | 不给 `--lanes` 时线道数应是登记线程数（31）除以每线道线程数，大于 1；写死成 1 时变小 |
| 48 | 落地 O3 执行器目录换回 487 | `O3_EXECUTOR_DIRECTORY_RELATIVE` 的值换回 487 那一份目录 | 断言该常量是 496 目录的那一条 |
| 49 | 落地 O3 一侧不把 frozen_o3 换到 496 | `point_frozen_o3_at_the_496_directory(frozen_o3)` 改成 `pass` | `FrozenO3Is496Tests`：frozen_o3 的三个常量应是 496 版 |
| 50 | 落地 对应表 sha256 不照搬迁登记倒换 | `attribution.py` 的 `load_function_map` 里去掉「倒换后各版也不是钉值」那半句 | 真表：`device_function_map.tsv` 今天 sha256 不是钉值（第四节第 5 条），原式靠倒换通过、变异抛 `ValueError`，所以红在 `test_the_two_function_maps_are_the_registered_ones`；手造那一条在 `test_attribution.py` 第 168 行起 |
| 51 | 复用 拒算的行照进请求 | `full_table_run.py` 第 243 行 `if row.refusal and not broken(...)` 改成 `if False:` | 请求里有一行要补位（`row.refusal` 真）：原式不进请求，变异进 |
| 52 | 复用 碰到模块集合不拒算 | 第 237 行 `return any(token in …)` 改成 `return False` | 改坏文本里有 `mod `、`#[path`、`include_str!` 等 `MODULE_SET_TOKENS` 之一的那一行：原式拒算，变异不拒 |
| 53 | 复用 合并不核库里产物的 sha256 | 第 404 行去掉 `sha256_bytes(output) == record.get("output_sha256")` | 库里一条记录的产物被改坏一个字节：原式不取回，变异取回 |
| 54 | 复用 收尾红不进复用账 | 第 475 行 `if code != 0:` 改成 `if False:` | 判定库收尾退 1（`test_full_table_run.py` 第 186 行断言 `["item_verdicts_finish_exit_1"]`） |
| 55 | 复用 现判的行不记进库 | 第 386 行 `if plan.recorded(number):` 改成 `if False:` | 一行刚现判完要记进库：原式记，变异不记 |
| 56 | 复用 不入库的那一包不留 | 第 384 行 `if not plan.in_the_library(number):` 改成 `if False:` | 一行不在库里：原式走不入库的那一支，变异不走 |
| 57 | 判法 E171 目录只在 e171_research_side 里插 | `current_tree_row_driver.py` 第 64 行 `sys.path.insert(1, e171_research_side.E171_DIRECTORY)` 改成 `pass` | 运行期这一句的判断为假（`e171_research_side` 已把目录插进 `sys.path`），运行期输出不变；改变输出的取样点是 `scripts/gate/lib_function_units.py` 静态解这份模块的 import：原式解得出 `device_timing`、`frozen_o3`、`run_directory`、`bad_implementation_table`，变异下四份都 `import-unresolved`（`test_current_tree_row_driver.py` 的 `JudgeImportTests` 钉它，源文件第 60–62 行注释自述）。这一行在运行期取样点上与原式同值，只在静态解析这一个取样点上不同值，不是等价变异 |

上表的「改了什么」是我读变异表那一行的原文与替换文、对着源码行号写的（2026-11-18）；执行员核对不上的在产物之前写进第十二节修订，不改判据。

**装置一侧的修（只在 U1 成立时有）自己的三条手造变异**：修的是 `test_full_table_run.py` 里的扫描与断言，驱动的锚点不收测试文件，所以不进变异表；执行员在草稿副本里手改、交主 agent 起 `python3 -B -m unittest -v test_full_table_run`，每一条必须红在新补的那条用例上，红的用例名与断言原文进第十二节：

| 手造变异 | 改成什么 | 在哪个取样点上改变输出 |
|---|---|---|
| 放行一切 | 「仓根自身」的放行条件改成对所有路径都真 | 假 `crates/` 文件里写 `"../../scripts/not_copied.sh"`（解出来是 `scripts/not_copied.sh`）：原式仍报它没罩住，变异放过 |
| 放行以点起头的 | 条件改成 `resolved.startswith(".")` | 假文件里写 `"../../.claude/not_copied"`：原式仍报，变异放过 |
| 不放行仓根自身 | 去掉放行 | 假文件里写 `concat!(env!("CARGO_MANIFEST_DIR"), "/../..")`：原式不报，变异报 `.` |

## 十、失败条款

「没测出效应」不一律写成装置有 bug：MT 里有行不是抓到、UT 里有红、DR 起不来，都是正当结果，如实记；下面只写让一次观测不算数或让管子不可信的情形。

| # | 条款 | 什么观测会让它触发 |
|---|---|---|
| F1 | 控制产物不是 5.2 写的样子：这条管子（驱动与读产物的读法）分不出抓到、没红、无效与基线红，那一类结局在正式产物里的读数这一趟不可信；不记成「表有盲区」，修好重跑 | 控制·抓到、没红、无效三行的结局符号不是 ✅、❌、⏭，或 `unit_test_exit` 不是「非 0、0、-1」，或行数不是 10，或退出码不是 1；红基线那一份退出码不是 2、行数不是 6、出现了 `mutation` 或 `verdict` 行 |
| F2 | 阴性对照红：基线不绿、还原后基线不绿、或控制·没红的单测红，不是结果，是装置与树对不上，两边都查（`.claude/rules/implementation-first.md` 第 4 条），既不作废也不当结果，交主 agent | 正式产物或控制产物里 `name=baseline unit_tests=red`（UT 已 `OK` 的前提下），或 `restored_and_baseline_green=false` 而 Z2、Z3 都没触发 |
| F3 | 表的行没跑齐：MT 不够判，没跑的行记「没跑」，不记成「抓到」也不记成「没红」 | 产物里 `name=mutation` 行数不是 57，或行名与表里的次序对不上（MT-逐行），或锚点不是恰好一次（MT-锚点，驱动退 3） |
| F4 | 有 ⏱：那几行不进三个数，单列；先重交同一条命令一次，两次都 ⏱ 才记 ⏱，并跑第八节「超时放宽」那一格 | verdict 的 `timed_out` 大于 0 |
| F5 | 副本里的基线红而装置目录里 UT 已绿：退 2 的原因不止一个，DR-原因只定出了其中一个 | 步 5 产物 `baseline unit_tests=red`，而步 3（或步 2）装置目录里同一组模块 `OK`。查法：`<草稿目录>` 里建一份与驱动拷法相同的副本（`copytree` 同款，不带 `__pycache__`），在副本里带 `E201_REPOSITORY_ROOT_FOR_A_COPY=<仓根>` 起同一条 `unittest`，看红的是哪一条；修只许落在「驱动怎么准备与起副本」，不许碰什么算抓到，写进第十二节 |
| F6 | DR-原因的预期对不上：退 2 的原因不能只写成「基线单测红」 | 19167 残留产物里 `name=baseline unit_tests=red` 的行数不是 1，或有 `name=mutation` 行，或 `name=anchor` 行数不是 57，或 `task-server/task log 19167` 里有 Python 回溯 |
| F7 | UT 分类落不进 U1、U2、U3：红了而分不出是装置要跟还是 `crates/` 一侧的事，UT 记「有红，原因没定出」，停 | 红的那一条或几条里有不是 19166 那一条的（新红），或 U1 的五个条件里有查不出的又没有 U2 的条件中 |

## 十一、作废条款与停机条款

### 读真仓那条红单测怎么分类（UT-分类的判法）

问法：红的那一条是装置要跟 `crates/` 的改动，还是 `crates/` 一侧新写的路径不该出现。执行员在步 2 现查下面各条，命令与原样输出整段贴进第十二节；先看 U2 的四个条件，再看 U1，两个都没中是 U3，不先有结论再找依据（`.claude/rules/evidence-discipline.md`「不许先有结论再建模型」）。第四节第 6 条是设计时读代码得到的预期，只是线索，这里的条件才是判据。

| 类 | 条件 | 处置 |
|---|---|---|
| U2 | 下面任一条成立：①`grep -rn 'REPOSITORY_ROOT_OF_THE_BUILD' crates` 的用法行里，有在它上面拼出的、落在 `SYNCED_ENTRIES` 之外、又不是 `multi-host.env` 的字面子路径（`.join("…")`、`format!`）；②被它调的函数读 `SYNCED_ENTRIES` 之外的仓内文件（读 `gpu_card_configuration.rs` 第 822–859 行与 `git_common_directory_of`，不止 `SINGLEFS_MULTI_HOST_CONFIG` 指的文件和 `multi-host.env`）；③步 1 日志里 `AssertionError` 的 `paths_not_covered_by` 交出的不止 `['.']`，或 `test_every_file_the_named_shell_scripts_source_or_run_is_copied_and_synchronized` 红；④E201 装置编的 bin（`e171_three_oracle_collusion_after_the_oracle_fixes.rs`）或它依赖的包里引用了该常量或它的用法 | **停**，不改 `crates/`、不改测试，步 4 至 6 不跑；交主 agent，写明落在哪个分支：路径在 E201 装置编译与运行的闭包上，是「同步清单（`scripts/hosts/lib_repository_copy.py` 的 `copy_closure`，不在执行员写范围）要加这一项」；路径不在闭包上，是「`crates/` 一侧新写的路径不该出现」，附 `grep` 命令与整行输出 |
| U1 | U2 一条都没中，且全部成立：①`paths_not_covered_by` 交出的恰是 `['.']`，`named['.']` 的出处只有 `crates/singlefs-implementation-experiments/src/bin/e185_gc_batch_wall_clock_split/main.rs:872` 一条，该行字面是 `concat!(env!("CARGO_MANIFEST_DIR"), "/../..")`，后面没再接字面子路径；②`REPOSITORY_ROOT_OF_THE_BUILD` 在 `crates/` 里每一处用法都只作为参数交给 `multi_host_configuration_file`（登记时只有 e185 第 840 行）；③`multi_host_configuration_file` 与它调的函数读的仓内文件只有 `SINGLEFS_MULTI_HOST_CONFIG` 指的和 `multi-host.env`，而 `scripts/hosts/lib_repository_copy.py` 的 `environment_for_commands_in_the_copy` 的说明写的是把主工作区那一份多机配置交给副本里起的命令；④`test_every_file_the_named_shell_scripts_source_or_run_is_copied_and_synchronized` 在同一份日志里是 `ok`；⑤`e171_three_oracle_collusion_after_the_oracle_fixes.rs` 里 `CARGO_MANIFEST_DIR`、`multi_host_configuration_file` 零命中（`grep -n`） | 装置的扫描要跟：只改 `test_full_table_run.py`。扫描交回的 `named` 不变（`.` 与它的出处仍可见）；解出来恰是 `.` 的条目单列成「仓根自身」，对它另断言：每条出处所在行的文本含 `CARGO_MANIFEST_DIR`（cargo 给的包目录往上数的写法）；非 `.` 的路径仍照旧要求被两份 `SYNCED_ENTRIES` 罩住。补三条手造用例（临时假 `crates/` 目录）：仓根自身被放行、`../../scripts/not_copied.sh` 仍被报、仓根自身仍被扫到而不是被吞掉；三条手造变异（第九节）各红一次。修订写进第十二节，列改动文件的改前改后 sha256；步 3 再交步 1 那一条命令，必须 `OK` |
| U3 | U1 的五个条件里有一条因读不到文件、命令报错或输出与上面写的形态对不上而查不出，而 U2 的四个条件一条都没中 | **停**，记「分不出」，列查不出的是哪一条与它的原样输出；交主 agent，步 4 至 6 不跑 |
| 新红 | 步 1 或步 3 里红的不止 19166 那一条，或红的不是它（例如 `test_attribution` 的钉值那几条） | 每条红各自走上面三类；不是读真仓那一条的写「新红」，停，交主 agent。钉值那几条红走停机 S2 |

### 作废（那一次的输出不算结果；修好之后重交同一条命令，不改判据）

| # | 条款 | 触发观测 |
|---|---|---|
| Z1 | 表不是登记的那一张 | 步 5 前后 `sha256sum <表>` 不是读法 2 的 `7ff739ee…`（改过表的，在第十二节写修订并换新 sha256 之后才跑），或产物头 `table_rows` 不等于 `grep -v '^#' <表> \| grep -vc '^$'` |
| Z2 | 装置在跑的时候变了 | 步 4 之前与步 5 之后，装置目录里 `*.py`、`*.tsv` 的 `sha256sum` 不同（第十二节记过的 U1 改动除外）；驱动自己的 `restored_and_baseline_green` 里也有「跑前跑后逐字节相同」这一半，但它与「还原后基线绿」并成一个值，分不开，所以另用这条 |
| Z3 | 树在跑的时候动了（六个模块与驱动的副本读的都是真仓） | 一次任务前后的树摘要（第八节命令）不同：步 1、步 3、步 4 的每一条与步 5 都各取前后两个值；不同的那一次不算，重交 |
| Z4 | 产物不完整或自相矛盾 | 驱动退 0 或 1 而产物里没有 `name=verdict mutations=` 行或没有 `name=done` 行（中途退出，`mutation-sampling.md` 第七类判据：收尾没有「已还原，基线仍全绿」就是中途退出，那一轮的「全抓」不算数）；或行数不是 2×57+4 = 118；或四栏之和不是 57；或 `grep -c` 数出的各栏与 verdict 行不等 |
| Z5 | 服务结局不是跑成 | 步 4、步 5 任务的 `TASK_SERVER_RESULT` 行 `exit=` 不是预期值（步 4 控制 `exit=1` 与 `exit=2`，步 5 `exit=0` 或 `exit=1`，步 1 与步 3 的 `exit=0` 或 `exit=1`），或带 `shard=`，或 `type=` 不是跑完：那一次的产物不进判定 |

### 停机（装置与 `crates/` 对不上，或钉值对不上：两边都查，既不作废也不当结果，交主 agent）

| # | 条款 | 触发观测 |
|---|---|---|
| S1 | UT 红落在 U2、U3、新红 | 见上面分类表；不改 `crates/`、不改装置的钉值、不改测试 |
| S2 | 钉值对不上 | 三份表中有一份：`sha256sum` 不是读法钉的值而 `load_function_map`（对应表、研究一侧对应表）与 `load_cell_classes` 又不认；表现为 UT-钉值那三条有一条不是 `ok`，或 `cell_classes.tsv`、`research_function_map.tsv` 的 sha256 与第四节第 5 条不同。两边都查：`git log -p -- <那份表>` 看表有没有被改、`git log -- .claude/gate.d/path-moves.tsv` 看搬迁登记有没有变；**不重钉、不改表**，交主 agent。对应表的 sha256 与代码钉值本来就不同（`98aec222…` 对 `95b93eb4…`，第四节第 5 条）本身不触发，只有 `load_function_map` 抛 `ValueError` 才触发 |

装置文件的 sha256 与第三节不同（别的会话改了装置目录）不是停机：记进第十二节，之后的 Z2 以记下的那一版为准；改的是 `mutation_driver.py`、变异表或测试文件的，同时报主 agent。

### 够判停机

每段交回附岔路单三行的表（已够判 / 还差什么 / 剩下的量能不能让它翻面）。第一段交回后：UT 应够判（`OK`，或红已分类并停），DR 的原因那一半够判，MT 不够。第二段交回后三行都应够判：MT 够判 = 产物过了 Z1 至 Z5、三个数落定（`timed_out` 为 0）、没红与无效逐条分了类；DR 够判 = 产物过了第六节 DR-产物。UT 落在 S1 的，第二段标「主 agent 定不跑」，MT 的量逐条标「未跑，等 UT 的停机处置」。附带量（UT-附带、MT-附带、第八节两个取样点）一律够判后不跑。

### 比对对象（重跑登记固定形态）

| # | 比对对象 | sha256 | 行数 | `replay.sh` 行 / 跑法 | 重跑产物名 | 备注 |
|---|---|---|---|---|---|---|
| 1 | `e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-11-mutations.out` | `1342dab8ba56f52b75c0cf89c294b9708410b9c148d497c0ddab4269f5c0233d` | 104 | none:变异驱动不在 `replay.sh` 的复跑表里，跑法是 `mutation_driver.py`（5.6 步 5） | `e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-18-r1-mutations-2026-11-18.out` | 只在 MT-附带按变异名并排；日期按跑那一天，跑在别的日子的在第十二节改写这一格；这一格改指第 57 行换上之后的产物，改前的名字与依据在第十二节修订 3 |

这一行在 `.claude/gate.d/replay-comparison-units.tsv` 里没有登记行（E201 没有任何一行），门禁 doc-experiments 的 replay-comparison-units 格在执行员登记之前判红：「复跑」写 `none:` 的行只缩不涨，所以要么在 `replay.sh` 补驱动行，要么按那张表文件头的「不登记的比对对象」写一行、点名开着的欠账号；这两样不在我的写范围里，交主 agent。

## 十二、修订

（留空；装置写之后、产物之前由执行员写，只许收严或补臂。步 0 核出的 sha256 与行数差异、树摘要命令的最终形态、U1 或 F5 的改动与它们的改前改后 sha256、步 2 现查的命令与原样输出、落产物那一天的产物名，抄在这里。）

修订 1 的正文由执行员起草（它写这份文件时被权限分类器拒），主 agent 抽核过其中的 sha256、行数与 grep 输出之后原样接进来，时点在控制与正式产物之前（用户定由主 agent 写）。

### 修订 1（2026-11-18，产物之前；单测红的分类与装置一侧的修）

依据：派发提示（主 agent）写的「照登记第十一节给的可核条件，把那条单测红分到 U1、U2、U3 或新红里，给现查的证据」「是 U1 的：在装置目录里改 `test_full_table_run.py` 与登记要求的手造用例、手造变异」。判据原样保留，下面只收严、补判据，没有放宽。

**步 0 核对（现查，只读，`git rev-parse HEAD` 仍是 `4319aa2b035b2d8238d64aeb68b3ceb7674cc723`）**：

- 装置目录里第三节列的每个 `*.py`、`*.tsv` 的 sha256 前 16 位与第三节逐个相同；表 `scripts/mutation/tables/e201_three_oracle_collusion_full_table_on_the_current_tree.tsv` 的 sha256 是 `7ff739ee43965a8fda2af2b75737105327020ab422da2b8b4104f924706304a1`，数据行 57（`grep -v '^#' <表> | grep -vc '^$'`），共 65 行；登记文件与问题单的 sha256 与登记头一致（登记文件在这一条修订写入之前是 `9aa12368f9d2073ec59478ed53a6f2b61cf688f6de7b90a9eeeeedf07a286bbb`）。
- 三份表现算：`cell_classes.tsv` `4c053fa2a7325ec16a7983121231ade70ca5269547baff922654162aca93d39a`（`contrast_against_r1.py:31` 钉的同一串）、`research_function_map.tsv` `9761c84f03d4295f28cf157279bf9e09010e0f0aee84049141d8e8f017d56a6a`（`attribution.py:48`）、`device_function_map.tsv` `98aec2229ca37c8ee8675722929b0fb5d36676115fdeb421406269a5e9ff5b90`（`attribution.py:47` 钉的是 `95b93eb4…`，与第四节第 5 条一致：不触发 S2）。`device_function_map.tsv` 里 `singlefs-verification` 1 行、`singlefs-harness` 0 行；`path-moves.tsv` 673 行，含 `singlefs-harness` 的 445 行（与第四节相同）。
- 树摘要命令点名的文件都在（`ls` 无报错：`Cargo.toml`、`Cargo.lock`、`.claude/heavy-thread-default`、`.claude/gate.d/path-moves.tsv`、`scripts/crash/*.sh` 8 个、`scripts/hosts/*.sh` 与 `*.py` 共 26 个）。最终形态在第八节命令后面加三个文件：`.claude/scripts/preflight.sh .claude/singlefs-ai-sop/scripts/preflight.*`（`ls` 得 `preflight.py`、`preflight.sh` 两个），全文：

```
{ find crates -type f \( -name '*.rs' -o -name Cargo.toml \) -not -path '*/target/*' -exec sha256sum {} + ; sha256sum Cargo.toml Cargo.lock .claude/heavy-thread-default .claude/gate.d/path-moves.tsv scripts/crash/*.sh scripts/hosts/*.sh scripts/hosts/*.py .claude/scripts/preflight.sh .claude/singlefs-ai-sop/scripts/preflight.* ; } | LC_ALL=C sort -k2 | sha256sum
```

- T0（装置改动之前，最终形态那条命令）：`092d27a9f9fb46eac63e4a2ffd7e3b02a02c791eb638dc51d6bdaf4fd076406e`（第八节原命令不含 preflight 三个文件的：`5e7357e86b768d272e1970afdc260f5fc959248298b4080f9174b794e366b67a`）。装置目录与 `scripts/mutation/tables/` 不在这条命令里，装置一侧的变化由 Z2 判。
- `research/results/` 里 e201 的已有文件（`ls research/results | grep e201`）：`…-2026-11-11-mutations.out`（原产物）、`…-2026-11-18-mutations.out`（任务 19167 退 2 的残留，没进 git）；这两份不动。

**UT 红的分类（第十一节 U2 四条、U1 五条逐条现查）**。原样输出：

```
$ grep -rn 'REPOSITORY_ROOT_OF_THE_BUILD' crates
crates/singlefs-verification/src/bin/e185_gc_batch_wall_clock_split.rs:823:    const REPOSITORY_ROOT_OF_THE_BUILD: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
crates/singlefs-verification/src/bin/e185_gc_batch_wall_clock_split.rs:840:            caller.multi_host_configuration_file(Path::new(REPOSITORY_ROOT_OF_THE_BUILD))?;
$ grep -n 'CARGO_MANIFEST_DIR\|multi_host_configuration_file' crates/singlefs-verification/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs; echo "grep_rc=$?"
grep_rc=1
$ grep -n 'name = "e185\|name = "e171' crates/singlefs-verification/Cargo.toml
109:name = "e171_three_oracle_collusion"
114:name = "e171_three_oracle_collusion_after_the_oracle_fixes"
159:name = "e185_gc_batch_wall_clock_split"
```

- U2 ①：常量只有 823（定义）与 840（作为参数交给 `multi_host_configuration_file`）两处，没有 `.join`、`format!` 拼子路径：不成立。
- U2 ②：被调的 `GpuConfigurationOfTheCaller::multi_host_configuration_file`（`crates/singlefs-verification/src/gpu_card_configuration.rs:723–732`）把环境变量 `SINGLEFS_MULTI_HOST_CONFIG` 原样记下的值、`repository_root`、`git_common_directory_of(repository_root)`（第 861–874 行，起 `git rev-parse --path-format=absolute --git-common-dir`）交给 `multi_host_configuration_file_given`（第 834–858 行）；它只读 `configured` 点名的文件，或 `git common-dir` 的上一层下的 `multi-host.env`（第 813 行常量），没有则退到 `repository_root.join("multi-host.env")`：读的仓内文件只有那两样：不成立。
- U2 ③：19166 日志第 18–19 行 `- ['.']`、`+ [] : {…}`，第 12 行 `AssertionError: Lists differ: ['.'] != []`，`paths_not_covered_by` 交出的恰是 `['.']`；`grep -c '^FAIL:\|^ERROR:'` 数该日志得 1，所以 `test_every_file_the_named_shell_scripts_source_or_run_is_copied_and_synchronized` 没红：不成立。
- U2 ④：`REPOSITORY_ROOT_OF_THE_BUILD` 只在 e185 那一个 bin 里（上面第一条 `grep -rn`，`crates` 全树只有这两行）；E201 装置编的 `e171_three_oracle_collusion_after_the_oracle_fixes.rs` 里 `CARGO_MANIFEST_DIR`、`multi_host_configuration_file` 零命中（`grep_rc=1`）：不成立。
- U1 ①：`named['.']` 的出处只有 `crates/singlefs-implementation-experiments/src/bin/e185_gc_batch_wall_clock_split/main.rs:872`（19166 日志第 19 行），823 行字面是 `concat!(env!("CARGO_MANIFEST_DIR"), "/../..");`，没再接字面子路径：成立。
- U1 ②：成立（同 U2 ① 的 `grep -rn`，每一处用法都只是 `multi_host_configuration_file` 的参数）。
- U1 ③：成立（同 U2 ②；`scripts/hosts/lib_repository_copy.py:916–921` `environment_for_commands_in_the_copy` 的说明写「SINGLEFS_MULTI_HOST_CONFIG 设成主工作区那一份配置的绝对路径」，`multi-host.env` 本来就不进 `SYNCED_ENTRIES`，`test_full_table_run.py` 的 `test_every_synced_entry_is_in_the_repository_and_the_plan_and_the_lanes_copy_the_shared_closure` 断言了这一点）。
- U1 ④：成立，由日志推：19166 是不带 `-v` 的一次，只有一条 `FAIL:`、没有 `ERROR:`，那一条是读真仓扫描的那条，所以另一条 shell 脚本传递闭包的用例没红；`-v` 的 `ok` 行在步 3 之后的新日志里读。
- U1 ⑤：成立（`grep_rc=1`）。

结论：U2 四条都不成立，U1 五条都成立，分类是 **U1（装置的扫描要跟）**，没有新红。19166 日志里 `current_tree_row_driver.SYNCED_ENTRIES` 那一句断言（原第 412 行）没有走到（第 411 行先红）；它与 `run.SYNCED_ENTRIES` 是同一份闭包（`test_every_synced_entry…` 断言二者排序后相等，那条在 19166 里没红），所以不影响分类，新用例里两份照旧都断言。

**步 2 的落盘**：派发提示叫「用已有的输出分类」，步 1 的 `-v` 日志没有交；改装置之前的读数就是任务 19166 的整份日志，已原样存进 `research/results/e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-18-r1-unit-tests-before.out`（24 行，`task-server/task log 19166 --all` 的原样输出，sha256 `7c23b234cb976f0c993f36a830ad84948b9a751fde52dadaa44e1e0c6e241dcd`）；它不带 `-v`，所以 UT-钉值那三条 `ok` 只能在步 3 的新日志里读。

**U1 的改动（只改 `test_full_table_run.py`，装置的别的文件、表、驱动与 `crates/` 一概没动）**：

- 改前 sha256 `9ddcf2fef9313ecd4b26a02bc2b9e8438a765d4bee990a6bfacaee0bcf43c217`，改后 `ef5ac9d9f8a95967a8fd5deb2ae1ca45f979bd9f7c8a15ab879a7062d5d3da87`（`git diff --stat`：62 行增、3 行减）。
- 新增三个函数（第 401–423 行一带）：`is_the_repository_root_itself`（`resolved == "."`，放行条件只在这一处）、`paths_to_synchronize`（扫到的路径去掉仓根自身，其余一条不少）、`origins_of_the_repository_root_without_the_manifest_directory`（仓根自身的每条出处所在行的文本要含 `CARGO_MANIFEST_DIR`，不含的交回）。`repository_root_paths_named_outside_test_code` 本身没改，`named` 里 `.` 与它的出处照旧可见。
- `RepositoryRootReferenceTests.test_every_repository_root_path_the_crates_name_outside_test_code_is_copied_and_synchronized`：两处 `paths_not_covered_by(named, …)` 改成 `paths_not_covered_by(paths_to_synchronize(named), …)`，加第三条断言 `origins_of_the_repository_root_without_the_manifest_directory(run.REPOSITORY_ROOT, named) == []`；两条 `assertIn` 判例照旧。非仓根自身的路径仍要被两份 `SYNCED_ENTRIES` 罩住，没有放宽。
- 三条手造用例（临时假 `crates/` 目录，同 `test_the_crates_scan_leaves_out_test_code_…` 的写法）：`test_the_repository_root_itself_is_let_through_and_needs_no_copy`（仓根自身扫得到、不要求被罩住）、`test_a_file_below_the_repository_root_is_still_reported_next_to_the_root_itself`（`../../scripts/not_copied.sh` 与 `../../.claude/not_copied` 仍被报）、`test_the_repository_root_itself_stays_in_what_the_scan_returns_and_needs_the_manifest_directory_on_its_line`（仓根自身仍在 `named` 里、出处可见；一处不带 `CARGO_MANIFEST_DIR` 的 `"../.."` 被另报）。`def test_` 行数：`test_full_table_run.py` 62 变 65，六个模块合计 121 变 124。

**补的判据（只收严）**：

- UT-条数：U1 补了三条，步 3 之后日志的 `Ran` 必须是 124，`-v` 日志末尾 `OK`，三条新用例与 UT-钉值那三条都 `ok`。
- 三条手造变异（第九节末表）各在装置目录的一份整目录副本里只改 `is_the_repository_root_itself` 的返回行：放行一切（`return True`）、放行以点起头的（`return resolved.startswith(".")`）、不放行仓根自身（`return False`）。起 `python3 -B -m unittest -v test_full_table_run`（环境变量 `E201_REPOSITORY_ROOT_FOR_A_COPY` 指真仓根），每一条必须红，且红的清单里必有：放行一切、放行以点起头的红在 `test_a_file_below_the_repository_root_is_still_reported_next_to_the_root_itself`；不放行仓根自身红在 `test_the_repository_root_itself_is_let_through_and_needs_no_copy`（它同时会让读真仓那一条红，这不算多余）。红的用例名与断言原文由承接的执行员抄回这一节。
- 产物名：新产物日期用 `TZ=Asia/Tokyo date +%F` 取，是 2026-11-18；正式产物 `research/results/e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-18-r1-mutations.out`，与第十一节比对对象表第 1 行的「重跑产物名」一致；两份控制产物与改后日志的名字见报告「要主 agent 跑的命令」一节。

**修订 1 之后、产物之前的读数（主 agent 经任务服务交，日志 `task-server/task log <号> --all`）**：

- 步 3 改后六个模块：任务 19424 退 0，「Ran 124 tests in 20.026s」「OK」。
- 手造变异「放行一切」：任务 19427 退 1，「FAILED (failures=1)」，红的是 `test_a_file_below_the_repository_root_is_still_reported_next_to_the_root_itself`。
- 手造变异「放行以点起头的」：任务 19428 退 1，「FAILED (failures=1)」，红的是同一条。
- 手造变异「不放行仓根自身」：任务 19429 退 1，「FAILED (failures=3)」，红的是 `test_the_repository_root_itself_is_let_through_and_needs_no_copy`、读真仓那一条 `test_every_repository_root_path_the_crates_name_outside_test_code_is_copied_and_synchronized`，另有 `test_a_file_below_the_repository_root_is_still_reported_next_to_the_root_itself`（修订里没预告这一条也红；不放宽判据，必红的那一条在清单里）。
- 附带 `test_row_selection_and_reuse`：任务 19430 退 0，「Ran 7 tests」「OK」。

修订 2 的正文由第二段执行员起草、主 agent 接进来。它开头说「`research/prompts/` 不在这一段执行员的写范围」是照主 agent 派发提示里的话写的，那句话不对：执行员的写范围有 `research/prompts/e*-r*-prereg.md`（`.claude/hooks/agent-write-scope.tsv:31`）；第一段执行员写这份登记时是被工具层的权限分类器拒的。

### 修订 2（2026-11-18，产物之后；只记读数与派发提示定的处置，不改判据）

执行员起草，交主 agent 接在修订 1 之后（`research/prompts/` 不在这一段执行员的写范围）。产物已跑，原判据与修订 1 一字不动；这一条只记产物之后的读数、Z2 与 Z3 的判法、第 57 行的分类，和派发提示定的处置。处置的依据是派发提示（主 agent）第 3 条那一句：「是等价的照规矩留档并换一条真会改行为的」。

**树摘要与装置摘要（主 agent 取的，命令是修订 1 那一条的最终形态与 `cd <装置目录> && sha256sum *.py *.tsv | sha256sum`）**：控制之前、控制之后、正式之后三次树摘要都是 `f44b458d8cb589976a607fe388f0a8948ef4d394e318905380d57390a4ed6722`；装置摘要前后都是 `196a8d7ecad5612fb54f0a44043fc855b65fe291e0b3b1f36b4c90675cc774f8`（执行员第二段开工时现算两样，同值）。树摘要与 T0（`092d27a9…`）不同：T0 之后别的会话往 `crates/` 打过补丁。Z3 按「一次任务前后」判：任务 19438、19439 夹在「控制之前」「控制之后」之间，任务 19440 夹在「控制之后」「正式之后」之间，前后值相同，三趟都不触发 Z3，读数都是树 `f44b458d…` 上的。任务 19424（改后单测）与 19427–19430 前后没有取树摘要，Z3 对它们判不了；UT 的「全过」另由控制与正式产物里四次基线（同六个模块、同样读真仓、Z3 判过的窗口里）`baseline unit_tests=green` 与 `restored_and_baseline_green=true` 印证。Z2：装置摘要前后相同，`test_full_table_run.py` 现算 `ef5ac9d9…` 等于修订 1 记的改后值，不触发。

**表**：`sha256sum` 现算 `7ff739ee…`（与读法 2 同），`git status` 无改动；产物头 `table_rows=57`，等于 `grep -v '^#' <表> | grep -vc '^$'` 的 57。Z1 不触发。

**读数**：

- 改后单测：任务 19424 `type=succeeded exit=0`，整份日志落 `research/results/e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-18-r1-unit-tests-after.out`（233 行，sha256 `0dcc12f95ae362c89d720e880e7b8890082a00c2cba411266865477d9f60092c`）：第 217 行 `Ran 124 tests in 20.026s`、第 219 行 `OK`；`skipped`、`FAIL:`、`ERROR:` 零命中；UT-钉值三条（第 32、60、70 行）与三条新用例（第 153、173、175 行）都以 `ok` 收尾。
- 手造变异 19427、19428：各 `FAILED (failures=1)`，红的是 `test_a_file_below_the_repository_root_is_still_reported_next_to_the_root_itself`，断言原文 `AssertionError: Lists differ: [] != ['.claude/not_copied', 'scripts/not_copied.sh']`（放行一切）与 `AssertionError: Lists differ: ['scripts/not_copied.sh'] != ['.claude/not_copied', 'scripts/not_copied.sh']`（放行以点起头的）。
- 手造变异 19429（不放行仓根自身）：`FAILED (failures=3)`：`test_the_repository_root_itself_is_let_through_and_needs_no_copy`（`AssertionError: Lists differ: ['.'] != []`）、读真仓那一条（`AssertionError: Lists differ: ['.'] != []`）、`test_a_file_below_the_repository_root_is_still_reported_next_to_the_root_itself`（`AssertionError: Lists differ: ['.', '.claude/not_copied', 'scripts/not_copied.sh'] != ['.claude/not_copied', 'scripts/not_copied.sh']`）。多红的这一条的原因：它的假 `crates/` 文件第一行就是仓根自身那一句 `concat!(env!("CARGO_MANIFEST_DIR"), "/../..")`（用例名里的「next to the root itself」），断言 `paths_not_covered_by(paths_to_synchronize(named), ("Cargo.toml",))` 恰等于两条子路径；不放行仓根自身时 `paths_to_synchronize` 不去掉 `.`，`.` 进了没罩住的清单。这是同一处改坏在另一条也取到仓根自身的用例上红，不是新红，判据不放宽。
- 附带：任务 19430 `Ran 7 tests`、`OK`。
- 控制一：任务 19438 退 1，产物 `…-2026-11-18-r1-control-mutations.out` 10 行，三条 `mutation` 依次 ✅（`unit_test_exit=1`）、❌（`0`）、⏭（`-1`），`verdict mutations=3 caught=1 invalid=1 survived=1 timed_out=0 restored_and_baseline_green=true`，`done emitted=10`：与 5.2 逐项相同，F1 不触发。
- 控制二：任务 19439 退 2，产物 `…-2026-11-18-r1-control-red-baseline.out` 6 行：`config table_rows=3 unit_test_modules=test_no_such_module_for_the_control …`、三条 `anchor … hits=1`、`baseline unit_tests=red`、`done emitted=6`，没有 `mutation`、`verdict` 行：与 5.2 相同，F1 不触发。
- 正式：任务 19440 退 1，产物 `…-2026-11-18-r1-mutations.out` 118 行（sha256 `ade983ea79fcbce3a05e579ea1e16f0498aaf738219c0ceaf6eb6db40f41d68b`）：57 条 `anchor`、57 条 `hits=1`、`baseline unit_tests=green`、57 条 `mutation`（变异名按序与表第一列 `diff` 无差，无重名）、`verdict mutations=57 caught=56 invalid=0 survived=1 timed_out=0 restored_and_baseline_green=true`、`done emitted=118`；按符号数 ✅ 56、⏭ 0、❌ 1、⏱ 0，与 verdict 逐栏相同。Z4、Z5、F2、F3、F4、F5 都不触发。

**第 57 行（「判法 E171 目录只在 e171_research_side 里插」，`unit_test_exit=0`）的分类：等价**。第九节第 57 行写的预期取样点是算键的库静态解这份模块的 import；今天这个取样点上它也与原式同值：

- 运行期：`e171_research_side.py:27–28` 无条件保证 `E171_DIRECTORY` 在 `sys.path` 里，`current_tree_row_driver.py:59` 与 `:63` 之间不动 `sys.path`，`:63` 的判断恒为假，替换前后运行期同值。
- 静态解析：`scripts/gate/lib_function_units.py:7369–7389` 的 `PythonModule.search_directories` 把模块级 import 进来的仓内模块加的 `sys.path` 目录算在引它的模块上（弄坏开关 `python-imported-sys-path-ignored` 关掉这一条，`:698`）；逐行驱动 `:59` 的 `import e171_research_side` 解到同目录的仓内文件，那一份 `:28` 加的目录就进了逐行驱动的搜索目录，所以去掉逐行驱动自己那一句，四份照样解得出。正式产物里这一行 `unit_test_exit=0`（`JudgeImportTests` 没红）是这一点的量的读数。
- 唯一让两式不同值的设置是算键的库开着那个弄坏开关，那是换了判法、不是逐行驱动的一个输入；逐行驱动 `:60–62` 的注释写的「不写，四份解不出」在今天的树上不成立。`git log` 里这一行（`f09a3c464`）、那一句驱动代码（`6736cbb38`）与算键的库那一段（`f09a3c464`）同一批进仓，看不出谁先写。
- 原产物 `…-2026-11-11-mutations.out` 只有 50 行，没有这一行；那 50 行里与今天同名的 49 行两边都 ✅，`V13_键不含改坏用的变异行` 今天叫 `V13_请求不含改坏用的变异行`（也 ✅），另 7 行（6 条「复用」、这一条「判法」）原产物没有。

**处置（派发提示定的，执行员只备好、没写进装置与表）**：留档一条测试、换一条会改行为的变异。改法备在执行员草稿目录 `swap/`：`test_current_tree_row_driver.py` 的 `JudgeImportTests` 加一条 `test_without_the_drivers_own_insertion_the_key_library_still_resolves_the_e171_modules_through_the_research_side`（拷进小树的逐行驱动去掉自己那一句，四份照样解得出；弄坏开关下红）；`current_tree_row_driver.py` 第 60–62 行注释改成今天的样子（代码不动）；表第 57 行换成「判法 E171 目录不进 import 路径」（`e171_research_side.py` 那一句 `    sys.path.insert(1, E171_DIRECTORY)` 换成 `    pass`，锚点在非测试源文件里恰好一次）。换上之后：`def test_` 合计 125，UT-条数的和照加 1；表的 sha256 变，Z1 按换上之后的新值判；正式整表重跑一次，产物新名（同一天跑写 `e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-18-r1-mutations-2026-11-18.out`），这一份 r1 产物的读数照留、不改；第十一节比对对象表第 1 行的「重跑产物名」与 `replay.sh` 复跑行、`.claude/gate.d/replay-comparison-units.tsv` 那一行要一起改指新产物。主 agent 不换的，第 57 行照这一条的分类记「等价、没换」，r1 的读数就是 MT 的答案。

**够判停机**：三行都够判（报告的岔路表）；附带量里 UT-附带、MT-附带已出，第八节「测试集合更大」那一格（MT 里有 ❌ 才跑）够判后未跑，「超时放宽」没有触发、不跑。

**修订 2 之后的读数与处置的落地（主 agent）**：

- 等价的校验（在执行员草稿目录的装置拷贝里，换上留档用例与新注释）：任务 19578 退 0，「Ran 125 tests」「OK」；任务 19582（算键的库开着弄坏开关 `python-imported-sys-path-ignored`，只跑 `JudgeImportTests`）退 1，「Ran 2 tests」「FAILED (failures=1)」，红的恰是留档的那一条 `test_without_the_drivers_own_insertion_the_key_library_still_resolves_the_e171_modules_through_the_research_side`：第 57 行分作等价的校验过了。换上的那一行预跑任务 19584 退 0（驱动退 0 即这一行被抓到）。
- 处置落地：`swap/` 里三份改名换进仓（换上之前核过草稿里的原件与仓里那一刻逐字节相同，换上之后回读一致）。换上之后：`test_current_tree_row_driver.py` 的 sha256 `0d837ff6c15be7a1843391e934050c4a70128b19ed0c89544ef0770643b0f064`、`current_tree_row_driver.py` `3856ce73681a3f4f981d0e7d27698d400d3269b96012fb64432eb6191dd139e8`、变异表 `2ab53e47a439110964849a414e352a4183eeb7c97e9cddb1edeaa1c9b66eb69b`（数据行仍 57）；六个模块 `def test_` 合计 125；装置摘要（`cd <装置目录> && sha256sum *.py *.tsv | sha256sum`）`59f018b854bb13606bb438e833ccc29ea18020a207072f2d61fb91cbf48f222e`。Z1、Z2 自此按这几个新值判。正式整表照修订 2 写的新产物名重跑，读数接在这一条后面。
- 换上之后的读数：改后六个模块任务 19631 退 0，整份日志落 `research/results/e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-18-r1-unit-tests-after-2026-11-18.out`（184 行），「Ran 125 tests in 13.567s」「OK」。正式整表任务 19632 退 0，产物 `research/results/e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-18-r1-mutations-2026-11-18.out` 118 行：「E7RESULT name=config table_rows=57 …」「E7RESULT name=baseline unit_tests=green」「E7RESULT name=verdict mutations=57 caught=57 invalid=0 survived=0 timed_out=0 restored_and_baseline_green=true」「E7RESULT name=done emitted=118」。树摘要在改后单测之前、之后与正式之后三次都是 `c6105f91ff0aa72be325b1d67c33ee923252a90402de8954e9831e874c061ed8`（与修订 2 记的 `f44b458d…` 不同：其间别的会话往 `crates/` 合过东西；Z3 只比一次任务前后，三次相同、不触发）；装置摘要前后都是 `59f018b854bb13606bb438e833ccc29ea18020a207072f2d61fb91cbf48f222e`。

### 修订 3（2026-11-18，产物之后；改指新产物与复跑驱动的形态，不改判据）

执行员（第三段）写。依据是派发提示（主 agent）第 1、2、5 条：「改指新产物：`scripts/experiment-apparatus/replay.sh` 复跑表 E201 那一行、`.claude/gate.d/replay-comparison-units.tsv` 里 E201 那一行、登记第十一节比对对象表第 1 行的重跑产物名，都改指换上之后的那份新产物」「把 E201 的驱动函数改成它认得的形态；改的是你加的那个函数，不改那一格的判法」「登记第十二节：这一段做的事记一条」。原判据、修订 1、修订 2 一字不动。

**改指的三处**（都从 `…-2026-11-18-r1-mutations.out` 改成 `e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-18-r1-mutations-2026-11-18.out`；新产物 118 行，sha256 `5f8f77648bb6700968ba07837800f9d8ad8f4bff2d76ca862ecee5c37f64338c`）：

- `scripts/experiment-apparatus/replay.sh` 复跑表第 2915 行：`E201|@driver_e201_mutations||e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-18-r1-mutations-2026-11-18.out|exact`。
- `.claude/gate.d/replay-comparison-units.tsv` 第 113 行第 5 列（留存产物名），其余五列不变；改后这份表 sha256 `e3bfc6d4b7977a4171b16b659ea2ff12d3dce2a602a69d764d472cc74707c3e2`。
- 第十一节比对对象表第 1 行「重跑产物名」一格，备注加一句指到这一条。同一行「`replay.sh` 行 / 跑法」一格写的 `none:变异驱动不在 replay.sh 的复跑表里` 是登记写下时的样子，原样留着；此刻这一条的复跑是 `replay.sh` 的 `driver_e201_mutations`（`.claude/gate.d/replay-comparison-units.tsv` 第 113 行第 6 列）。

r1 那份产物 `…-2026-11-18-r1-mutations.out`（118 行，sha256 `ade983ea79fcbce3a05e579ea1e16f0498aaf738219c0ceaf6eb6db40f41d68b`，56 抓 1 没红）留着，读数不改：它是第 57 行还是「判法 E171 目录只在 e171_research_side 里插」那一版表（sha256 `7ff739ee…`）的读数；新产物是第 57 行换成「判法 E171 目录不进 import 路径」那一版表（sha256 `2ab53e47…`）的读数。

**复跑驱动的形态**：任务 19579（`replay.sh --selftest`）红在「静态核 mutation-driver-tables」，日志第 310 行真仓那一趟是 `ast.literal_eval` 的回溯。那一格的判法（`replay.sh` 的 `replay_mutation_driver_table_problems`）从驱动函数正文取 `--unit-test-module <模块>`，正文里一个都没有时退到 `mutation_driver.py` 里 `^TEST_MODULES = (.*)$` 那一行的 `ast.literal_eval`；E201 那份驱动的 `TEST_MODULES` 跨两行写（第 32–33 行），按行取到的是没收口的半个元组。`replay.sh` 里调 `mutation_driver.py` 的 12 个驱动函数中，只有 `driver_e201_mutations` 走的那个目录的 `TEST_MODULES` 不在一行里收口（`awk` 列驱动函数、`grep -n '^TEST_MODULES = '` 逐份看，2026-11-18）。改法：`driver_e201_mutations` 的驱动命令里把六个单测模块逐个写成 `--unit-test-module test_contrast_against_r1 … --unit-test-module test_full_table_run`，名字与次序同 `TEST_MODULES`；驱动 `main` 的 `config` 行照 `test_modules` 的次序拼 `unit_test_modules=`，与不给时逐字节相同，跑的单测也相同。函数上方加两行注释写这一点。判法没改。改后 `replay.sh` sha256 `6b47e1e9d13fcadf62a2c392de9b0c6f753c81de7c0bf2f9a0917c58e3af6372`（含别的会话同一份文件里没提交的改动；这一段只改了第 2915 行、第 4028–4030 行注释与第 4037–4038 行两行参数），`bash -n` 过。推的、没跑：六个模块的 import 闭包罩住装置目录全部八份非测试源文件，表头没写 `--unit-test-module`，所以那一格对 E201 不报问题；自证由主 agent 交、等结局。

## 十三、读过的文件与跑过的命令

读过的文件（行区间；只读，2026-11-18）：

- `.claude/agent-common.md` 1–125；`.claude/rules/test-discipline.md`、`mutation-sampling.md`、`implementation-first.md`、`three-way-inference.md`、`evidence-discipline.md`、`test-script-discipline.md` 整份。
- `research/prompts/e201-r1-questions.md` 整份；`research/prompts/e201-preregistration.md` 1–82、316–450、513–582（另 `grep -n '^#'` 列了全部标题，没读第 83–315 行与第 583–755 行的修订正文）。
- `.claude/kb/experiments/201-三oracle合谋今树整表.md` 整份，**含它的结果句**：第 20 至 22、41 行的数（单测 107、115 条，变异 14、38、45、50 条全抓，任务 6325、13144 等）。读到了，列在这里：它们是表只有 14 至 50 行时的读数，与 MT 的候选读法无关，这一次的判据不从它们来（第四节第 3 条）。
- `research/results/e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-11-mutations.out`：`tail -3`（末三行，含 `verdict`、`done`）与 `wc -l`、`sha256sum`；`research/results/e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-18-mutations.out`：`head -5`、`wc -l`（`ls -la research/results/e201*` 只列了名字与大小）。其余行没读。
- `task-server` 日志：任务 19166 整份（2658 字节）、任务 19167（0 字节），经 `cat` 读 `~/.local/state/singlefs/task-server/logs/` 下两份文件。
- 装置目录：`mutation_driver.py` 1–150（整份）；`test_full_table_run.py` 330–427；`attribution.py` 30–75、116–128；`test_attribution.py` 156–176；`test_contrast_against_r1.py` 119–124；`current_tree_row_driver.py` 55–66；`test_row_selection_and_reuse.py` 的类与用例名（`grep -n`）；`full_table_run.py` 237–243、382–387、404、474–476（`sed` 与 `grep -n`）；`e171_three_oracle_collusion_after_the_oracle_fixes/result_lines.py` 30–50；`e171_research_side.py` 15–45（`grep -n`）。
- `scripts/mutation/tables/e201_three_oracle_collusion_full_table_on_the_current_tree.tsv`：第一列全列，第 42 至 57 个数据行的原文与替换文。
- `crates/singlefs-implementation-experiments/src/bin/e185_gc_batch_wall_clock_split/main.rs` 815–830 与 `grep -n REPOSITORY_ROOT_OF_THE_BUILD`；`crates/singlefs-verification/src/gpu_card_configuration.rs` 50–58、805–880、`grep -n multi_host_configuration_file`；`crates/singlefs-verification/Cargo.toml` `grep -n 'name = "e185\|name = "e171'`；`crates/singlefs-implementation-experiments/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes/main.rs` `grep -n 'CARGO_MANIFEST_DIR\|multi_host_configuration_file\|GpuConfigurationOfTheCaller'`（零命中）。
- `.claude/gate.d/replay-comparison-units.tsv` 1–40；`.claude/gate.d/path-moves.tsv` `grep -c` 与 `grep -n 'singlefs-harness'`；`.claude/hooks/run-scope-guard.sh` 1–60；`scripts/kb/quote-kb.py` 1–40；`.claude/heavy-thread-default`。

跑过的命令（原样，仓根起，都是只读）：

```
bash scripts/hosts/work-directory-ownership.sh claim <工作根>/ledger-topdown-e201-rerun-design --purpose 'E201 变异表在今树上的重跑登记'
sha256sum research/prompts/e201-preregistration.md research/prompts/e201-r1-questions.md
sha256sum *.py *.tsv   # 在装置目录
sha256sum research/results/e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-11-mutations.out; wc -l <同一个文件>
grep -vc '^#' scripts/mutation/tables/e201_three_oracle_collusion_full_table_on_the_current_tree.tsv   # 得 57
awk -F'\t' '!/^#/ && NF {n++; print n": "$1}' scripts/mutation/tables/e201_three_oracle_collusion_full_table_on_the_current_tree.tsv   # 数到 57
grep -c 'def test_' test_{contrast_against_r1,q2_misses,attribution,whole_table_matrix,current_tree_row_driver,full_table_run}.py   # 13、8、16、3、19、62，和 121
grep -c 'singlefs-harness' .claude/gate.d/path-moves.tsv   # 445
grep -c 'singlefs-verification' device_function_map.tsv; grep -c 'singlefs-harness' device_function_map.tsv   # 1、0
grep -c '4c053fa2a7325ec16a7983121231ade70ca5269547baff922654162aca93d39a' <装置目录下八个源文件>   # 只有 contrast_against_r1.py 得 1
python3 scripts/kb/quote-kb.py <草稿目录>/quote.md '.claude/kb/decisions/13-验证路线.md@#### 已定项 6：三个互相独立的 oracle'   # 回读逐字节一致
```

## 十四、并行与挂钟

独立单位：驱动的 57 条变异各自在一份新副本里跑单测、互相不共用文件，原理上可以并行；但驱动的写法是逐条串行并按表的次序写产物行，这一次不改驱动（改驱动是改装置）。步 1、步 3 的单测是一次 unittest 进程里的 121 条用例，用例互不依赖但不拆开并行。控制两条（步 4）互相独立，可以同时交。

线程：单进程单线程，不分线程；`.claude/heavy-thread-default` 登记值 31 是重活默认，驱动与 unittest 都不读 `SINGLEFS_THREAD_CAP`，也不起线程池，所以这一次不会用到它。

串行：步 0 至 5 按次序串行：步 1 的读数决定步 3 要不要改装置，步 3 改了装置才轮到步 4、5 的产物（产物不许比装置旧），步 5 的基线必须绿才有逐行结局；步 4 的两条控制要在基线绿之后。驱动内部逐条串行：每条变异的副本、单测、删副本依次做，前一条还原后下一条才开始。

预计挂钟：步 1 一次单测约 13 秒（量的，任务 19166 的 `Ran 121 tests in 12.753s`），附带的第七个模块几秒；步 4 第一条控制 5 次单测（基线、三条变异、收尾基线）、第二条 1 次（基线红即退），合计 6 次，约 1.5 分钟（推的，6 × 12.753 秒加拷贝）；步 5 正式 59 次单测约 752 秒加拷贝与 `py_compile`，推 13 至 15 分钟；U1 改装置与补手造用例的人工另算，不在挂钟里；整趟任务时间合计推 20 分钟以内，没有超过 2 小时的段，不需要分片单元与续跑入口（驱动没有续跑；中途退出的整趟重交，每趟 15 分钟，重做的代价小于写续跑的）。

预计在用核数：各段都是 1 个核：单个 python 解释器跑 unittest，驱动子进程串行，没有并行；峰值内存推不超过 1 GB（没量过，依据：纯 Python、装置目录 896K、不编 Rust、不开设备）。台数、挑哪台与内存上限不写进登记，由统一任务服务定；这几条任务读仓里的文件、在共享根下写副本，两台都能跑（`.claude/agent-common.md`「所有任务都要多机可用」）。
