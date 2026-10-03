# e171-conclusion-r3 正推腿（Sonnet）报告：格 K1、K4

轮名 e171-conclusion-r3（第三轮，最终轮），立场：正推。日期 2026-10-03。kb 引文写 kb 文件自己的行号（全部现查）；代码引文写是哪一棵树：「s4 树」= E171 跑的 `/tmp/claude-1000/e171-s4/frozen-tree/`，「今天的树」= 本轮冻结副本 `/tmp/claude-1000/e171-conclusion-r3/tree/`（与仓里 `crates/` 逐字节同，下面第 0.1 节有命令）。模型目录 `research/prompts/e171-conclusion-r3-sonnet-model/`（`recount_k4.py`、`rerun-on-r3-tree.sh`），草稿 `/tmp/claude-1000/e171-conclusion-r3-sonnet/`。

## 0 先报五件会改写回文字或改数的事

1. **第 15 行的归属依据，kb 里有两句互相冲突，第二轮「站住两轮」没有对着它们查过**：D3 已定项 8 的射程说用户数据落点是分配器政策、违反它的镜像完全合法、不改新池新建文件的字节；`layout/01-first-txn.md` 与 D3 已定项 10 的字节表却逐字写着 t1 落在 50180。第 34 行靠已定项 8 射程归「分配政策」，第 15 行靠 layout 的字节表归「真盲区」，两行违反的是同一条已定项 8 定案 1。两句并排抄在第 1.7 节；按 `.claude/rules/format-evolution.md`「挪的时候不许丢内容」一条，两条已定条款互相矛盾的不在写回时判，记进 checks-owed 交用户。
2. **第二轮写回五条里，除第 4 条只给了依据没给句子外，其余四条各有写宽或写窄处**（第 1 节逐条）：句 1 的「没测到」对 Q2 写宽；句 2 的 29 张数不在实验页、「O1 崩溃一列不是 RefFS」放错格；句 3 漏版本 0、没定义已确认与已开始、「系统配置槽除择槽与几何外不判」漏整池拒、结尾「D13 第 110 行」是判决语；句 5 的「点名测试判一部分」对第 242 行不成立。
3. **K4：五个数我都从 `out/` 与产物重数同数（第 2 节原样输出），但其中「实现一侧」两组（61 个见证拒，RefFS 27 / 27 与 36 态）的「今天的树」是第一轮副本，不是今天**：今天的 core 有 `Undecidable` 一支的新定义（`recovery.rs` 第 3301 行），61 里有 6 个态落在这一支，今天的树上没重做。
4. **入库装置上一个都不能原样重做**：E171 装置没有挂载模式、没有 RefFS 列、panic 不导出；`replay.sh` 的 E171 复跑读「今天的 `crates/`」（`replay.sh` 第 947 行），产物是 s4 树的，逐字比对会因树不同而变。29 张（Q2）是唯一已经在入库产物里的数，但那几行复跑用今天的树另出一份，不是还原 s4 的读数。
5. **第二轮引的产物行号（4525、4759、4760）在 `…-s4-f4-substitutes-comparator.out` 里，这份产物没有 `replay.sh` 行，实验页状态还写「补位待跑」**；同一批 Q2 行在有复跑登记的 `…-s4-f4-comparator.out` 第 4662、4663 行。

## 0.1 树与快照（命令原样输出）
```
$ sha256sum -c --quiet research/prompts/e171-conclusion-r3-snapshot/kb-sha256.txt; echo kb_rc=$?
kb_rc=0
$ sha256sum -c --quiet research/prompts/e171-conclusion-r3-snapshot/crates-sha256.txt; echo crates_rc=$?
crates_rc=0
$ diff -rq e171-s4/frozen-tree/crates e171-conclusion-r1/tree/crates | wc -l
83
$ diff -rq e171-s4/frozen-tree/crates e171-conclusion-r3/tree/crates | wc -l
105
$ diff -rq e171-conclusion-r1/tree/crates e171-conclusion-r3/tree/crates | wc -l
32
$ diff -rq /tmp/claude-1000/e171-conclusion-r3/tree/crates /home/fy5090/code/singlefs/crates | wc -l
0
```

## 1 K1：第二轮判决第三节的写回句与 C407 改写，逐句对读数与 kb 原文

第二轮判决的五条写回在 `research/prompts/e171-conclusion-r2-main-verification.md` 第 42–46 行，C407 改写在第 48 行。每一句下面给：核到的、写宽 / 写窄 / 放错格、该怎么写（零轮，没被攻过）、什么现象会推翻我。完整可写回的文字集中在第 3 节。

### 1.1 句 1　关系

原句：「三条是否源头独立（表第 4 列）E171 没测到，不写支撑也不写推翻」（`research/prompts/e171-conclusion-r2-main-verification.md:42`）。

- 核到的：Q1 触发那一支成立。产物 `E7RESULT name=q1_alpha counted_rows=37 all_quiet_rows=12` 一行（`research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator.out:4529`），实验页 Q1 行（`.claude/kb/experiments/171-三oracle合谋测试.md:11`）。D13 条件句是「若三者全部放行，说明存在共同盲区，回头补 oracle。」（`.claude/kb/decisions/13-验证路线.md:166`），读数直接撑得住。
- **写宽一处**：「没测到」对 Q2 不成立。Q2 就是量 O2、O3 一对有没有共同放过的坏镜像（`.claude/kb/experiments/171-三oracle合谋测试.md:5` 写「Q2（手造坏镜像喂 O2、O3，有没有一张两条全放行）」），产物 `name=q2_all_quiet … all_quiet_rows=4`（`research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator.out:4763`），4 张全是 P8、按登记 F3 判成期望推错，D13 射程已经写着「真坏镜像里 O2（独立解析器 + checker）、O3（独立规约执行器） 两条全放行的 0 张」（`.claude/kb/decisions/13-验证路线.md:169`）。所以没量到的是 O1 与另两条的源头独立，不是 O2、O3 这一对；而这一对的 0 张也不能当独立的证据：补丁与期望出自同一份改法代码，「两边一起错时逐行相等发现不了」（`.claude/kb/experiments/170-O3规约执行器第一版.md:527`）。
- 放错格：没有。关系写在实验页「影响的决策」那一行，不进 D13 正文，对。`.claude/rules/format-evolution.md@决策正文只写现状，依据写成指针；决策与实验双向登记` 要求「支撑」连着分项依据引回实验：句 2 做这件事，两句必须一起落（今天依据段写着「这一条依据今天还不引它」，`.claude/kb/decisions/13-验证路线.md:173`）。
- 该怎么写：关系格写「支撑」，指「独立性要被验证，不许自称」（`.claude/kb/decisions/13-验证路线.md:165`）与回头补 oracle 那一句；同一格写明表第 4 列只在 Q2 量过 O2、O3 一对，不支撑也不推翻「三个互相独立」。
- 推翻它的现象：Q2 的补丁与期望若改成由 O3、O2 都不读的第三方各自独立产生（E170 页 527 行那条限定消失），这一对的 0 张才算独立的读数，「没测到」才该改成「测到了」。

### 1.2 句 2　依据

原句：「引存在性读数——有三者全部放行的坏实现（E171 s4 树）」（`research/prompts/e171-conclusion-r2-main-verification.md:43`），后接「不引比例与真盲区行数」（同一行）。

- 核到的：全放行 `row_numbers=15,23,27,34,42,92,112,242,269,270,271,277`（产物第 4529 行）；只靠一条 `q3_alpha caught_rows=25`（第 4571 行）；O3 独有的坏镜像 P 族 12、B 族 17（第 4759、4760 行，我逐行重数得 12 + 17 = 29，第 2 节）。
- **三处要改**：
  1. **数不在实验页**：`grep -n 'o3_only' .claude/kb/experiments/171-三oracle合谋测试.md` 零命中（页里只有「O3 在坏镜像那一族（Q2）上红得多」一句，第 114 行）。`.claude/rules/format-evolution.md@决策正文只写现状，依据写成指针；决策与实验双向登记` 说依据「实验页上已有的数不抄」：先把这 29 与它的限定补进页的 Q2 行，D13 依据只指过去、不写 12 + 17。
  2. **29 是实例数，不是种类数，也不是「坏实现」**：29 张取自 `name=q2_row` 里 O2 绿、O3 判不在的行，P 族是补丁种类 P2、P3、P4、P5、P9，B 族是盲测的 P2、P3、P5 三型的随机实例（第 2 节脚本输出 `patch kinds among them`）；这些补丁是结构自洽的伪造根与伪造续接记录（E170 登记 `research/prompts/e170-preregistration.md:493` 起：P2 伪根 txg 更大，P3 伪续接一条，P5 根的实例代号改小），正是 D13 O2 行「验不到什么」自己写的「一个块被覆盖成另一份合法内容」那一类（`.claude/kb/decisions/13-验证路线.md:162`）。所以这 29 说明 O3 判得到 O2 声明不判的那一类，不是 O2 的意外盲区。其中流一的 P2、P5 两种补丁 O2 一列随底态翻面（`q2_geometry_unstable_kinds=2`，`.claude/kb/experiments/171-三oracle合谋测试.md:12`）。
  3. **只写 O3 独有、不写 O3 在坏实现表上 0 红，读起来是选择性引用**：同一份产物 `O3甲=0 O3乙=0`（第 4525 行）。依据一句里两件都写（不带数）。
- 放错格：「写明 E171 的 O1 崩溃一列是模型的崩溃判官、不是 RefFS」（`research/prompts/e171-conclusion-r2-main-verification.md:43`）不是依据，是射程（E171 量的是什么）；实验页也没有这句（`grep -n RefFS .claude/kb/experiments/171-三oracle合谋测试.md` 零命中），E171 登记明写不用它（`research/prompts/e171-preregistration.md:193`）。放进 D13 射程，同时补进页的「它答不了的」。
- 推翻它的现象：若补进页的 Q2 行时发现 `q2_row` 里 O2 绿、O3 不在的 29 张有 1 张在 E170 的期望表里期望「在」（即 O3 判错了、不是 O2 漏了），「O3 有只靠它判出的坏镜像」对那几张不成立；我没逐张核期望（只核了数与补丁种类）。

### 1.3 句 3　O3 行三个格

原句：「要落在录制流里写过根槽、流序在 [已确认, 已开始] 之间的某一版上」（`research/prompts/e171-conclusion-r2-main-verification.md:44`），「每一版的内容取实现写的根，系统配置槽除择槽与几何外不判」（同一行），「验不到什么」加「实现写出的根的内容、实现自己的恢复与读者代码、系统配置槽的见证值」（同一行）。读码：`research/e7-index-bench/src/bin/e170_second_run_o3_spec_executor/` 下的文件，今天仓里与第一轮副本里那一份逐字节同（`diff -rq` 无输出，第 2 节）。

- 「判什么」留设计句：对。`.claude/kb/decisions/13-验证路线.md:163` 的「崩溃点截断后的每个磁盘状态是否落在规约允许的集合里」是设计，今天的做法写进射程「实现今天的样子」（那一段已经存在，`.claude/kb/decisions/13-验证路线.md:169`）。
- **写窄**（四处）：
  1. 版本 0 漏了：版本表 = 「基镜像上的现行那一版（版本 0，执行器在基镜像上自己恢复得出）加录制流里每一次根槽写按流序各一版」（`research/e7-index-bench/src/bin/e170_second_run_o3_spec_executor/version_table.py:3`），已确认的下标可以是 0。「录制流里写过根槽的某一版」不含它。
  2. 「已确认」「已开始」没有定义。已确认只从崩溃态里落了的写推：「if version.return_point is not None and version.return_point <= largest_index:」（`research/e7-index-bench/src/bin/e170_second_run_o3_spec_executor/version_table.py:97`），崩在返回之后而之后的写一条没落的态推不到。第二轮攻方自己给的写法里有「已确认按崩溃态里落了的最晚一条写定」（`research/prompts/e171-conclusion-r2-opus-output.md:229`），判决转写时丢了。
  3. 「系统配置槽除择槽与几何外不判」漏了整池拒：`pool_level_refusal_of_slot` 判池级字段越界、任一自证过的槽越界整池拒（`research/e7-index-bench/src/bin/e170_second_run_o3_spec_executor/system_configuration.py:165`），整池拒时恢复不出版本（`research/e7-index-bench/src/bin/e170_second_run_o3_spec_executor/executor_core.py:37`），恢复不出的态判不在（同目录 `executor_core.py` 第 126 行）。journal_tail 全目录只有一处读：全持久态的锚点断言 A4（`research/e7-index-bench/src/bin/e170_second_run_o3_spec_executor/executor_main.py:180`），判定不用它。
  4. 「每一版的内容取实现写的根」太宽：投影只有六样（`research/e7-index-bench/src/bin/e170_second_run_o3_spec_executor/root_ring.py:22` 写「一版 = (实例代号, checkpoint_txg, 树表单元指针 86, 中央映射树根指针 86, 树 ID 水位, 回退下界 F)」），根记录其余字段不进投影。「验不到什么」里的「实现写出的根的内容」同样宽，该写「被投影的六样的取值，与六样之外的字段」。
- 「前缀五条」对：`research/e7-index-bench/src/bin/e170_second_run_o3_spec_executor/replay_prefix.py:3` 写「前缀口径五条」。
- **结尾「D13 第 110 行「职责不变」与之一致」是判决语，不进 kb**：kb 正文引别的分项写「已定项 3」不写行号；那句原文是「仍是崩溃点集合的独立执行器，职责不变」（`.claude/kb/decisions/13-验证路线.md:110`），与 O3 取录制流并不矛盾（RefFS 才「不读录制流」），但这是判决的结论，不是射程该写的现状。
- **放错格 / 重复**：机制（版本表、窗口、投影、系统配置槽怎么用）放射程，后果（看不到什么）放「验不到什么」，第二轮原句把「每版内容取实现写的根」与「实现写出的根的内容」各写了一遍。「独立性来自哪里」一格没动：取录制流意味着允许集合那一半不是照 kb 推出来的，只有恢复规则那一半是（第一轮 G3 点过「判什么」「独立性来自哪里」都写宽，第二轮的写回只改了前者）；射程补一句，不改表格那一格。
- 推翻它的现象：O3 的版本表若在今天的执行器里改成不含版本 0（`version_table.py` 第 3 行那一句变），或 `journal_tail` 进了判定，上面的写窄就变了；我读的是仓里这一份（与第一轮副本逐字节同），没在新态上跑。

### 1.4 句 4　「全部故事」改写

原句：「依据 D13 自己的表（O1 行把崩溃之后的集合交给 RefFS）与 M4 的落点（E165、Verus）」（`research/prompts/e171-conclusion-r2-main-verification.md:45`）。被改的原句：「本工程功能正确性的全部故事就是这三个 oracle。」（`.claude/kb/decisions/13-验证路线.md:167`）。

- 依据都核到：表 O1 行「无崩溃处的内容期望归它，崩溃之后允许看到的集合归 RefFS（已定项 3）」（`.claude/kb/decisions/13-验证路线.md:161`）；M4 行「M4 自己不单独证」，落点是 E165 的最远一版判据与 Verus 的必红世界（`.claude/kb/decisions/13-验证路线.md:34`）；已定项 7「需要第二个输入的核对归记录核对器」（`.claude/kb/decisions/13-验证路线.md:186`）。不依据「每行有点名测试」是对的：那张表取自 `crates/mutations.tsv`，每行本来就带点名的测试，不是 E171 的读数。
- **写窄一处**：判决只给了依据，没给要写进去的句子；依据里漏了记录核对器（已定项 7，E171 页自己就给它留了附带列）。**同一句在 C407 行里还有一份**：「功能正确性的全部故事就押在这三条上」（`.claude/kb/checks-owed.md:327`），改 D13 时 C407 行同改，否则全仓还有谁在用这句（`.claude/rules/format-evolution.md@撤回一个数或一条结论，同样记进欠账表`）答不全。
- 位置：这是定案块里的一句（第 167 行），不是表格某一格；该写成「不只押在这三条上」的否定加三处指针，不要把别处的内容抄进来。
- 推翻它的现象：若「O1 行把崩溃之后的集合交给 RefFS」被改回（RefFS 的调用点撤掉），或 M4 的落点不再是 E165 与 Verus，这一句的依据就变。

### 1.5 句 5　写序与见证判据

原句：「一次发布内各写的先后（写序）只有 I-7.12、段序列钉与点名测试判一部分；无读故障的崩溃态上见证判据为真没有判官」（`research/prompts/e171-conclusion-r2-main-verification.md:46`）。

- 核到：I-7.12 判写序的一部分：「所以任何合法状态上一条根带的 F 都已经在它所在那块盘的系统配置里」（`.claude/kb/invariants.md:66`），E171 只靠 O2 的第 86、102、265 行就是它。第二截判拒绝：`judge_writable_mount_refusal` 对见证类拒绝先在崩溃镜像上复算判据，为真、盘上没变、只读读出在允许集合里就判合法（今天的树 `/tmp/claude-1000/e171-conclusion-r3/tree/crates/singlefs-harness/src/mount_transparency.rs:1195` 调 `newer_publish_witnessed_on_the_crash_image`；这个文件与 s4 树逐字节同，`diff -q` 为 `mt_same`）。它与 D13 自己写的射程一致：「可写挂载被拒的状态不判透明，判拒绝本身合不合法」「带故障的状态不在射程」（`.claude/kb/decisions/13-验证路线.md:57`）。所以「无判官」的准确说法是：判拒绝的那一道只复算「判据在崩溃镜像上是不是为真」，不问「这个崩溃态没注入读故障，判据凭什么为真」。
- **写宽一处**：「点名测试判一部分」对第 242 行不成立。第 242 行变异（s4 树 `crates/mutations.tsv` 第 172 行）点名的是 `publish_that_fails_on_the_system_configuration_slot_is_frozen_so_the_next_publish_cannot_overwrite_the_units_of_its_root`（`/tmp/claude-1000/e171-s4/frozen-tree/crates/mutations.tsv:172`），判的是失败路径的冻结，不是写序；对第 270 行点名的是 `every_crash_point_after_a_publish_on_a_version_without_file_returns_keeps_the_witness`（`/tmp/claude-1000/e171-s4/frozen-tree/crates/mutations.tsv:1312`），那条钉返回之后的见证值，成立。
- **「段序列钉」只能写「登记在册」，不能写「判得出」**：登记表逐字比对（`.claude/kb/layout/01-first-txn.md:406`），普通发布一行的段序列把根槽 FUA 与系统配置槽轮换分成两段（`.claude/kb/layout/01-first-txn.md:411`），用例 `fn recorded_paths_match_the_registered_segment_sequences`（`crates/singlefs-harness/tests/new_pool_file_creation_publish.rs:315`）存在。第 242 行变异把这两段并成一段（页 168 行写「基线里根槽 FUA 自成一段，变异把它与系统配置写切进同一段」），所以按推理这个用例会红；没编译、没跑（要编译 harness 档，不归我），是推的。
- 放错格：这句跨 oracle（O2 的 I-7.12、段序列钉、点名测试不是 O1–O3 里的哪一条），不进表最后一列；射程一句，欠一行。第二轮写「验不到什么或欠账」没选，这里选：射程「三条都不判的」一句 + 新欠账一行（第 3 节）。
- 今天的 core 上的限定：句里的「见证判据为真」在 s4 树与第一轮副本上数过；今天的 core 对「判不出」重定义了，撕裂的轮换不算判不出（`/tmp/claude-1000/e171-conclusion-r3/tree/crates/singlefs-core/src/recovery.rs:3301` 写「撕裂的系统配置轮换（读得出、自证不过，设备没报错）不算判不出」）。判据 W 本身不依赖 core，但第二截那一道在今天的 core 上给什么要重做（第 2 节 K4）。
- 推翻它的现象：段序列用例若在第 242 行变异上不红，「登记在册」就是全部，射程里「写序有判官」那半句要删；第二截若在今天的 core 上对第 242、269 行的拒不再判 legal，「没有判官」那半句要改。

### 1.6 C407 改写

原句：「读已入库的 E171 比对器产物（不重跑，重跑归 experiment-replay）」，「每个登记行挂一个补 oracle 的欠账号」，「每一列至少有一样（坏实现行或坏镜像）只有它红，O3 那一列用 Q2 的 O3 独有坏镜像」（`research/prompts/e171-conclusion-r2-main-verification.md:48`）。被改的是 C407 行的「怎么拦」与「前置」两格（`.claude/kb/checks-owed.md:327`）。

- 核到：D13 判定量是「有没有三者全部放行」，钉全放行集合合字面；今天没有一格门禁读这份产物：`grep -rn 合谋 .claude/gate.d` 只命中 `checker-independence-and-sync.sh` 第 40 行一句注释（命令输出在第 2.4 节）。「前置」格的「O3（独立规约执行器） 零代码」（`.claude/kb/checks-owed.md:327`）已过期：O3 已有 Python 执行器。
- **写宽三处，写窄两处**：
  1. 宽：「每个登记行挂一个补 oracle 的欠账号」。第二轮自己的归属里，第 34 行是分配政策（镜像合法）、第 92 行是决策正文欠一句、第 27、112、277 行是三条按设计不判实现恢复的那一段（判决第 38 行），这几行的欠账不是「补 oracle」。该写「挂一个欠账号，类别取：补 oracle / 写条款 / 登记按设计三条都不判」。
  2. 宽：「每一列」没定义。E171 有四列（O1-对拍、O1-崩溃判官、O2、O3），Q3 的 `only_O1`、`only_O2` 是三列口径（`research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator.out:4571`）；O1-崩溃判官单独红的行只有第 233 行，批 2 不红（实验页 `.claude/kb/experiments/171-三oracle合谋测试.md:174` 记「不稳定」）。取三列口径（O1 = 对拍并崩溃判官）。
  3. 宽：O3 那一列取 `name=q2_family` 的 `o3_only_strict` 之和，来源是手造的伪造根 / 伪造续接镜像（上面 1.2 第二点），不是坏实现；句子里要写明，否则读起来像 O3 在坏实现上有独有的红，而产物里 O3 在 37 行坏实现上 0 红。
  4. 窄：「登记集合」没钉哪一种读法。Q1 三种读法甲、乙 12 行、丙 13 行（多第 233 行），第 102、233、265 行批 1 批 2 翻面（产物第 4529、4542、4555 行）。该钉甲乙的 12 行，丙另登记；登记集合要有一个文件（形态先例：`.claude/gate.d/e165-pinned-red-examples.tsv`，行由脚本追加不手改）。
  5. 窄：没说读哪一份产物。第二轮引的行号在 `…-s4-f4-substitutes-comparator.out`，它没有 `replay.sh` 行（`research/scripts/replay.sh:1012` 写「补位产物入库之后另登一行」），页 463 行仍写补位那一趟待跑；有复跑登记的是 `…-s4-f4-comparator.out`（`research/scripts/replay.sh:262`）。判官读哪一份、两份差什么（补位 6 行全是未触达，`name=substitution_summary … still_not_compiled=0`，第 4631 行；分母仍 37，第 4521 行）要先写进页。
- 判官怎么骗它是 K3（攻方腿的格），这里只给文字落点；第 3 节的 C407 文字把阴性、阳性对照那几个字段（`positive_controls_not_red=0`、`a3_columns_with_red_on_the_baseline=0`、`negative_bases_with_o3_red=0`、`known_bad_rows_with_o2_quiet=0`，产物第 4794 行）列为判官要同读的字段，K3 腿若证明它们不够，这一段再改。
- 推翻它的现象：补位那一趟已入库的产物把全放行集合改了（今天读到仍是 12 行）；或页把 Q1 的甲乙读法改成不同的集合。

### 1.7 真盲区候选名单与另几类的归属，逐行对 kb 原文与产物整行

第二轮归属见 `research/prompts/e171-conclusion-r2-main-verification.md:38`。「产物行」都是 `research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator.out` 里该行 `name=q1_quiet_row reading=q1_alpha` 的行号；「kb 原文」是我现查的 kb 行。判定一栏：一致 = 归属与原文、读数对得上；冲突 = kb 里还有另一句按相反方向读；没说 = 原文没覆盖。

| 行 | 第二轮归属 | kb 原文（kb 文件自己的行号） | 产物行 | 判定 |
|---|---|---|---|---|
| 15 | 真盲区候选 | D3 已定项 10 ③「数据单元取最低的、起点 32768 对齐、两槽都空、整跨度都过合取的起点」（`.claude/kb/decisions/03-空间分配.md:204`）；`layout/01-first-txn.md` 第 67、309 行写 t1 落 50180（下面并排）。变异把用户数据从 slot 50180 挪到 slot 50240：822149120 ÷ 16384 = 50180，823132160 ÷ 16384 = 50240（页 `.claude/kb/experiments/171-三oracle合谋测试.md:86` 的两个偏移），而 50240 正是开放段 [50240, 50304) 的起点 | 4530 | **冲突**：D3 已定项 8 射程说这类违反写出的镜像完全合法、不改新池新建文件的字节（`.claude/kb/decisions/03-空间分配.md:171`），第 34 行就靠它归「分配政策」；同一句对第 15 行同样适用。判决的「站住两轮」引 ⑤ 与 layout 第 309 行；⑤ 的表只有 t2–t8（`.claude/kb/decisions/03-空间分配.md:206`），t1 只在 layout |
| 23 | 真盲区候选 | D8 已定项 6「一个码 3 叶（类型 2、容器号 1、出生代 3、记录数 1、记录宽 140）」（`.claude/kb/decisions/08-核心索引结构.md:203`）；射程「每个容器号都是某条记录首次插入时的 inode 号」（`.claude/kb/decisions/08-核心索引结构.md:205`） | 4531 | 一致 |
| 42 | 真盲区候选 | D16 已定项 1「推一串空发布把 F 抬到现行那一版的 txg」（`.claude/kb/decisions/16-发布语义.md:42`）；「卸载之后 F = 现行那一版的 txg，回退候选集只剩现行那一版」（`.claude/kb/decisions/16-发布语义.md:57`） | 4534 | 一致 |
| 242 | 真盲区候选 | D16 已定项 7「系统配置槽在根槽持久之后再更新」，段序列因此唯一（`.claude/kb/decisions/16-发布语义.md:181`）；类比 I-7.12（`.claude/kb/invariants.md:66`）。写序条款本身是明文；「态不合法」要靠 D23 已定项 14 的见证判据（第 2 节 W 为真 15 态）才成立 | 4537 | 一致，但「不合法」的第二块依据（见证判据）读的是 D23 已定项 14 那一段，不是 D16（`.claude/kb/decisions/23-journal的角色与格式.md:392`） |
| 269 | 真盲区候选 | D23 已定项 14「c_见证 > 它的计数器即为真」（`.claude/kb/decisions/23-journal的角色与格式.md:392`）；「mkfs 之后第一次取号读到 0、写 0：新池新建文件的字节不变」（同一行） | 4538 | 一致 |
| 270 | 真盲区候选，只有带 ack 的判官判得到 | D16 已定项 7「fsync 等系统配置轮换持久之后才返回」（`.claude/kb/decisions/16-发布语义.md:181`） | 4539 | 一致；今天没有任何一条 oracle 判它（O3 不读 tail，第 1.3 节） |
| 271 | 真盲区候选 | D8 已定项 8「`deleted_inodes` 与墓碑树写 inode 树的树 ID（65537）」（`.claude/kb/decisions/08-核心索引结构.md:250`） | 4540 | 一致 |
| 27 | 三条不判实现恢复里靠记录重建的那一段 | D13 M4「M4 自己不单独证」，落点 E165 与 Verus（`.claude/kb/decisions/13-验证路线.md:34`）；D23 已定项 15「由记录重建那次发布的根」（`.claude/kb/decisions/23-journal的角色与格式.md:450`） | 4532 | 一致 |
| 277 | 同 27 | 同上；D23 已定项 14「恢复只施加 `(实例代号, checkpoint_txg)` 严格大于所选根的记录」（`.claude/kb/decisions/23-journal的角色与格式.md:357`） | 4541 | 一致 |
| 112 | 同 27，多施加，依据只有注 1 | 「前缀规则不跨实例边界」（`.claude/kb/decisions/23-journal的角色与格式.md:369`） | 4536 | 一致 |
| 34 | 分配政策，镜像合法 | D3 已定项 8 射程「违反它的实现写出的镜像完全合法」（`.claude/kb/decisions/03-空间分配.md:171`） | 4533 | 一致（但见第 15 行：同一句也覆盖第 15 行） |
| 92 | 条款在字节表与里程碑验收，决策正文欠一句 | 里程碑页「inode 记录的写入时间留成 A 的 ⇒ 判红」（`.claude/kb/milestone/02-second-txn.md:113`）；字节表「写入时刻（mkfs 参数，可复现）」（`.claude/kb/layout/01-first-txn.md:267`） | 4535 | 一致，窄一点：字节表那一行说的是新池新建文件那一次（mkfs 参数），第二次发布（覆盖写）的写入时间取值只在里程碑页，页自己写「它不要求正确」（`.claude/kb/milestone/02-second-txn.md:7`）。「决策正文欠一句」成立 |

名单里没有的两行：第 233 行只在丙读法全放行（产物第 4563 行），第二轮已按「丙里真盲区候选、甲乙里 O1-崩溃判官抓到」处理，我没找到与之冲突的原文；第 155、161 行是未触达却红，不在 Q1 名单（产物第 4528 行）。完整性（该进而没进的）是 K2，不在我这一格。

#### 第 15 行与第 34 行的两句并排（`sed -n` 原样）

D3 已定项 8 射程（`.claude/kb/decisions/03-空间分配.md` 第 171 行整行）：

```
**射程**：它是**分配器政策不是格式位**，与已定项 5 同一类——盘上没有字段声明它，违反它的实现写出的镜像完全合法；拦它只能靠运行时读数（D26（后台整理与放置回收） 已定项 6 的回落计数与全空聚簇段数）。⚠️ 第 2 条混放的代价：装着用户数据的旧段要等段里的用户数据也释放才能整段空出来，提交内生块的聚簇随挂载次数退化；现象是全空聚簇段数随挂载次数单调下降、回落次数上升。**改新池新建文件的字节：否**——新池新建文件写出的字段与宽度一个都不变，变的只是分配记录条目里那个落点的取值，而 [layout/01-first-txn.md](../layout/01-first-txn.md) 登记的是字段与宽度不是取值。两句待办：w ≥ 2 时各设备上的聚簇段要不要对齐归已定项 5 的实现；「同一次请求切出的事务按 key 升序取号」已写进 D16（发布语义） 已定项 5，运行时检查仍欠。runs 这个量今天没有条款消费它的绝对值（D26（后台整理与放置回收） 已定项 1 逐字「阈值数值不进格式：等 E10（老化：结构抗老化 vs 事后整理）」）。
```

D3 已定项 10 射程（`.claude/kb/decisions/03-空间分配.md` 第 220 行整行）：

```
**射程**：③ 里「索引节点取最低的空槽」只在开放段之内说——按字面读「单元区里最低的空槽」会与已定项 5 / 已定项 8「聚簇段只给提交内生块」给出两套写清单，字节表与 E142（新池新建文件的干跑） 装置取的都是开放段内那一套。⑤ 取「树 ID 升序 + 树内先叶后根」的理由：树 ID 是全仓到处在用的那个键（D8（核心索引结构） 已定项 8 逐字「条目按树 ID 升序排」），先叶后根是 COW 本来的顺序（父指针要等孩子的落点定了才写得出来）；中央映射树倒数第二是因为它要装别人的条目，树表单元最末是因为它装全部树的根指针。码 3 容器取「数据单元」那一档，依据是 D18（块里携带什么信息） 已定项 9 的对齐要求按单元大小分档、与它是不是用户数据无关，两种读法在开放段内给同一个落点，写死为条款把 C324（码 3 打包容器要不要 32768 对齐没写） 收口。**改新池新建文件的字节：是**——七个提交内生块的落点整体重排，分配记录的 key、位置条目的槽号、中央映射条目的 value 全跟着变。这一项没有走三方论证，主 agent 的推荐与用户的选择都可推翻。
```

`layout/01-first-txn.md` 第 67 行与第 309 行整行：

```
| t1 | 事务 | 数据单元（文件内容）——不在开放段 [50240, 50304) 里的最低 32768 对齐空槽对 | 1 | 32768 | 50180–50181 | 65536 | 二 |
| 用户数据落点政策 | — | — | 已选设备内槽号最小、32768 对齐、两槽都空、不在开放段与 `R` 内的空槽（新池新建文件 t1@50180） | D3（空间分配） 已定项 8 / 已定项 10 | 已定（2026-09-13） |
```

- 读法：已定项 8 射程说「违反它的实现写出的镜像完全合法」「改新池新建文件的字节：否」「登记的是字段与宽度不是取值」（`.claude/kb/decisions/03-空间分配.md:171`）；已定项 10 射程说「改新池新建文件的字节：是」「七个提交内生块的落点整体重排」（`.claude/kb/decisions/03-空间分配.md:220`），layout 逐字写 t1 落 50180 并把它标成「已定」。两处说的对象不同（t1 用户数据 vs t2–t8 提交内生块）但都落在新池新建文件那一份字节上，kb 没有一句说清「用户数据落点的取值算不算 layout 登记的字节」。
- 对名单的影响：按已定项 8 射程读，第 15 行同第 34 行，归「分配政策、镜像合法」，真盲区候选剩 6 行（23、42、242、269、270、271）；按 layout 读，第 15 行留在名单。我不判（format-evolution 说两条已定条款互相矛盾的不在写回时判），建议：名单写回时第 15 行标「依据有冲突，见欠账 C新」，欠账一行写这两句（第 3 节）。
- 推翻它的现象：若 `layout/01-first-txn.md` 第 67、174、192、210 行的「50180」被一句明文标成「示例取值、不是登记的字节」，冲突消失，第 15 行按 layout 读不成立、归「分配政策」；反过来若 D3 已定项 8 射程那句「改新池新建文件的字节：否」被改成「是」，第 34 行也要重判。

## 2 K4：第二轮判决引的数，出处、树、入库装置上能不能重做、写进 kb 标什么

第二轮判决里这些数的出处是第二轮攻方（Opus）的模型目录 `research/prompts/e171-conclusion-r2-opus-model/out/`（判决第 4 节：主 agent 在会话草稿照 `tools/rerun.sh` 跑，14 份输出逐个 `cmp` 无不同）。下面这张表是我对每个数做的：出处、哪棵树、我重数 / 重跑的结果、入库装置上能不能重做、kb 里要标什么。「入库装置」指仓里入库的 `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs`、`research/e7-index-bench/src/bin/e171_three_oracle_collusion/` 的比对器与 `research/scripts/replay.sh` 的 E171 登记行。

### 2.1 逐个数的出处、树与重做能力

| 数 | 出处（文件） | 用的哪棵树 | 我的重数 | 入库装置上能重做吗 | 写进 kb 要标什么 |
|---|---|---|---|---|---|
| W 为真：第 242 行 15 态、第 269 行 63 态；基线 0 / 576；76 条流 19 361 态里只这四条流非零 | `…-opus-model/out/witness-consistency-all.out`（76 行）、`witness_consistency.py` | 不读任何 `crates/` 树：读 E171 导出态（s4 树写出的 `research/target/e171-dumps/`，不入 git）与仓里 O3 Python（与第一轮副本逐字节同） | `recount_k4.py` 从 out 重数同数；我另在今天仓里重跑 `witness_consistency.py`（8 条流）同数（2.2） | **不能**：W 只在攻方模型目录里，入库的比对器与装置没有这一谓词；要入库得把 W 并进研究一侧比对器（零轮）再复跑 | 「副本脚本在 E171 导出态上数（Python，读不入 git 的导出目录），入库装置没有 W」；不写成入库读数 |
| 61 个见证拒：第 242 行 9、第 269 行 52；W 为真的 78 态里另有 17 态先被撕裂系统配置那一种拒（C331 已知形） | `out/refusal-judge-summary.out`（`core=s4` 行）、`out/witness-vs-mount.out`（28 行） | **s4 树的 core**；第一轮副本的 core 同数（`refusal-judge-summary.out` 里 `core=today` 的第 242、269 行 16 行与 `core=s4` 的逐行同，我比过）；**今天的树没重做** | 重数得 9 + 52 = 61，撕裂在先 17，W 为假的 2 320 态里 0 个见证类拒（2.2） | **不能**：入库装置没有挂载模式；崩溃注入第二截（checker 档）在它自己抽的态上判，不吃 E171 的态，也没有第 242、269 行的变异 | 「s4 树 core 上两次可写挂载（副本探针）」；**今天的 core 在 `Undecidable` 一支上改过（`recovery.rs:3301`，今天的树），这 52 里 6 个态（`Undecidable`）落在这一支，今天的数不保证是 61**，不写成今天的实现 |
| 第 125 行 D = 38（连同第 155、161 行改触达定义 D = 40） | `out/drive-s4-m125.tsv`（第 4–13 次调用 `o2_full_persisted=I-3.11`，第 14 次 `panic`）、产物 `name=q1_denominator`（第 4521 行 `counted_D=37 … self_crash=6`） | s4 树，加副本里打第 125 行变异的 core | 重数：O2 红的调用 4–13 共 10 个，37 + 1 = 38，37 + 3 = 40（2.2） | **不能**：装置对 panic 整条流不导出（`/tmp/claude-1000/e171-s4/frozen-tree/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs:1116` 写「某一步 panic（实现自崩，这条流不导出）」）；38 是改了口径（panic 照报错截断照判）之后的数，不是登记读法 5 的数 | 「D = 37 是登记读法 5 的口径；第 125 行在 panic 前有 13 次调用，副本探针在 s4 树上驱动、O2 在全持久态上红，改口径后 D = 38；这是改法的预演，不是装置读数」；两个 D 都写，不删 37 |
| O3 独有的坏镜像 29 张（Q2 的 P 族 12 + B 族 17） | 产物 `…-s4-f4-substitutes-comparator.out` 第 4759、4760 行；同一批在有复跑登记的 `…-s4-f4-comparator.out` 第 4662、4663 行（`research/results/` 下 6 份产物都有 `o3_only_strict=12`） | 基线流是 s4 树的（E171 装置 `baseline` 模式 + E170 导出装置 `bad-states`） | `name=q2_row` 117 行逐行重数得 P 12、B 17（2.2）；补丁种类 P2、P3、P4、P5、P9，B 族 P2、P3、P5 | **能，但不是还原**：`replay.sh` 第 261、262 行等 E171 登记行重跑，读「今天的 `crates/`」（`research/scripts/replay.sh:947`），基线流与 s4 树不同（今天的树第一轮就第 40 条 op 起不同），数不保证仍是 12 / 17；约 4 小时，重型，归主 agent | 「s4 树基线流上的读数；实例数（5 种补丁 + 盲测 3 型的随机实例），期望出自 E170 设计员」；页 Q2 行先补这个数 |
| RefFS 27 / 27 与 36 态 | `out/judge-summary.out`（`judge_s4_v2` 行）、`out/shapes.out`（`o3_strict_not_in=27`） | s4 树 core；第一轮副本逐态同；**今天的树没重做** | 重数：warmup 形 O3 不在 27、其中 RefFS 绿 27；content 形 O3 不在 27、RefFS 红 36（= 27 + 另 9 态 O3 判在）（2.2） | **不能**：E171 没有 RefFS 一列；两种写流是攻方 `o3_ack_shapes.py` 造的、RefFS 直调 crate、崩溃落点（hi）手给、没经 `reference_filesystem_translation`（攻方报告「这条腿自己的限度」第 3、4 条） | 只进实验页「它答不了的」，写「副本探针，RefFS 直调 crate，崩溃落点手给，没经翻译层」；那「另 9 态」是两边归属崩溃时刻的规则不同，推的，不写进 D13 |
| 「与它 83 个文件不同」（正文第二节） | 第二轮判决「树」一段；`diff -rq` | 83 是 s4 树对**第一轮冻结副本** | s4 对今天（本轮冻结副本，与仓逐字节同）是 105，第一轮副本对今天 32（0.1 节） | — | 不写数；写「与今天的 `crates/` 不是同一份（快照摘要见页）」。s4 树不是任何一次提交的样子（139 个文件与 `HEAD` 不同、37 个不在 `HEAD`，2.4），只在 `/tmp` 里，kb 不能把 `/tmp` 路径当依据，只写页里已有的摘要 `d15864653cd8…` |

### 2.2 我复核的命令与原样输出
```
$ python3 research/prompts/e171-conclusion-r3-sonnet-model/recount_k4.py

== 1 W（witness-consistency-all.out）
labels=76 states=19361
w_true_nonzero={'r242new': 5, 'r242fix': 10, 'r269new': 8, 'r269fix': 55}
row242 w_true=15 row269 w_true=63
baseline states=576 w_true=0

== 2 见证拒（refusal-judge-summary.out，core=s4）
witness_class_refusals_by_row={'r242': 9, 'r269': 52} total=61
torn_system_configuration_refusals_by_row={'base': 135, 'r242': 245, 'r269': 135, 'r270': 250}
second_cut_verdicts_over_all_s4_refusals={'legal:NewerPublishWitnessedBySystemConfiguration': 826}

== 3 RefFS（judge-summary.out）
('content-judge_s4_v2', 'in', 'green') 252
('content-judge_s4_v2', 'in', 'red') 9
('content-judge_s4_v2', 'not_in', 'red') 27
('warmup-judge_s4_v2', 'in', 'green') 261
('warmup-judge_s4_v2', 'not_in', 'green') 27
warmup: o3_not_in=27 of which reffs_green=27
content: o3_not_in=27 reffs_red_total=36 reffs_red_while_o3_in=9

== 4 第 125 行（drive-s4-m125.tsv 与产物）
calls with o2_full_persisted=I-3.11: [4, 5, 6, 7, 8, 9, 10, 11, 12, 13]
last call: (14, 'unmount', 'o2_full_persisted=not_judged')
4521 E7RESULT name=q1_denominator rows_run=281 counted_D=37 untouched=238 only_in_the_ignored_pass=0 compile_failed=0 anchor_not_unique=0 not_found=0 self_crash=6 timeout=0 device_stop=0 wrapper_ended=0 enough_to_judge_none_side=true
4528 E7RESULT name=untouched_but_red rows=2 row_numbers=155,161
counted_D=37 self_crash=6 -> D with row 125 = 38; with 155,161 as well = 40

== 5 Q2 O3 独有（产物 q2_row 逐行重数）
o3_only_strict by family (recount from q2_row): {'P': 12, 'B': 17} total 29
patch kinds among them: {'P': ['P2', 'P3', 'P4', 'P5', 'P9'], 'B': ['P2', 'P3', 'P5']}
4759 family=P rows=26 o3_only_strict=12
4760 family=B rows=40 o3_only_strict=17
```

W 在今天仓里重跑（Python，经 `capped.sh 15` 与 `nice -n 19`；`o3copy` 是第一轮副本里 O3 执行器目录的拷贝，与仓里那一份逐字节同（2.4））：

```
$ bash research/scripts/capped.sh 15 nice -n 19 python3 research/prompts/e171-conclusion-r2-opus-model/witness_consistency.py o3copy base1=… r242new=… r269new=… r270new=…
name=witness_consistency label=base1 states=60 w_true=0 no_landing=0 first_states=-
name=witness_consistency label=r242new states=81 w_true=5 no_landing=0 first_states=13,16,17,18,62
name=witness_consistency label=r269new states=60 w_true=8 no_landing=0 first_states=2,5,6,7,8,9,10,11
name=witness_consistency label=r270new states=56 w_true=0 no_landing=0 first_states=-
$ （同上，换 base2 / r242fix / r269fix / r270fix）
name=witness_consistency label=base2 states=516 w_true=0 no_landing=0 first_states=-
name=witness_consistency label=r242fix states=649 w_true=10 no_landing=0 first_states=13,16,17,18,105,108,109,110,454,491
name=witness_consistency label=r269fix states=516 w_true=55 no_landing=0 first_states=2,5,6,7,8,9,10,11,74,77,78,79
name=witness_consistency label=r270fix states=460 w_true=0 no_landing=0 first_states=-
$ grep -E "base1|base2|r242|r269|r270" …-opus-model/out/witness-consistency-all.out
name=witness_consistency label=base1 states=60 w_true=0 no_landing=0 first_states=-
name=witness_consistency label=base2 states=516 w_true=0 no_landing=0 first_states=-
name=witness_consistency label=r242new states=81 w_true=5 no_landing=0 first_states=13,16,17,18,62
name=witness_consistency label=r242fix states=649 w_true=10 no_landing=0 first_states=13,16,17,18,105,108,109,110,454,491
name=witness_consistency label=r269new states=60 w_true=8 no_landing=0 first_states=2,5,6,7,8,9,10,11
name=witness_consistency label=r269fix states=516 w_true=55 no_landing=0 first_states=2,5,6,7,8,9,10,11,74,77,78,79
name=witness_consistency label=r270new states=56 w_true=0 no_landing=0 first_states=-
name=witness_consistency label=r270fix states=460 w_true=0 no_landing=0 first_states=-
```

### 2.4 其余核对命令（原样输出）

```
$ grep -rn 合谋 .claude/gate.d
.claude/gate.d/checker-independence-and-sync.sh:40:# 归门禁 checker-tier-crates-mutation-replay；也判不了 RefFS 的语义对不对、它与实现是不是落在同一个崩溃语义的误解
$ grep -rln 合谋 crates | head
crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs
$ diff -q s4/…/mount_transparency.rs r3/…/mount_transparency.rs && echo mt_same
mt_same
$ diff -rq <第一轮副本>/research/e7-index-bench/src/bin/e170_second_run_o3_spec_executor research/e7-index-bench/src/bin/e170_second_run_o3_spec_executor; echo rc=$?
rc=0
$ snapshot_digest(s4 树)（row_driver.py 的函数，页写的摘要见 .claude/kb/experiments/171-三oracle合谋测试.md 第 321 行）
('d15864653cd83cedddc4902b6598d5db9cf590a62e6cbb8a54bed6661e4700cb', 238)
$ s4 树 crates 与 HEAD 逐文件比（git hash-object）
same_as_HEAD 67 differ_from_HEAD 139 not_in_HEAD 37
$ grep -nF 的三处 replay.sh 行
262:E171|@driver_e171_s4_f4_comparator||e171-three-oracle-collusion-2026-10-03-s4-f4-comparator.out|exact
947:# 推的挂钟约两小时（s4 整表 5 569.6 秒、Q3-核三趟各约两分钟）。117 行的读数与 A4–A7 
1012:# 编不过的行补位那一趟（第八段起驱动在编不过之后照登记 5.2 补位）还没有入库产物
$ grep -c "^E171.@driver" research/scripts/replay.sh; grep -c substitutes research/scripts/replay.sh
10
0
$ git ls-files research/results | grep -c e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes
2
```

## 3 可直接写回的文字（全部零轮：我给的收严与改写，没被攻过；kb 里的行号写回时现查）

写回只由 kb-scribe 做，我不写 kb。下面每一段是「换哪一处、换成什么」；没有一句引 `/tmp` 路径，E171 的读数只指到实验页与 `research/results/` 下的产物。

### 3.1 D13 已定项 6（`.claude/kb/decisions/13-验证路线.md`）

**a. 定案块「本工程功能正确性的全部故事就是这三个 oracle。」（第 167 行）换成：**

```
功能正确性不只押在这三个 oracle 上：O1 行已把崩溃之后允许看到的集合交给 RefFS（已定项 3），恢复函数的 M2–M4 落在 Verus 与 E165（已定项 1），需要第二个输入的核对归记录核对器（已定项 7）；三个 oracle 之间的共同盲区靠上面的合谋测试找。
```

**b. 表 O3 行「验不到什么」格，原文「规约本身可能写错——必须配阳性对照才有意义」，后面接：**

```
；被投影的六样的取值与六样之外的根记录字段、实现自己的恢复与读者代码、系统配置槽的见证值（journal_tail）、一次发布内各写的先后
```

**c. 射程「实现今天的样子」：**

c1. O3 那一句「……还没接进崩溃点重放」之后、「RefFS（已定项 3）在……」之前加：

```
；判一个崩溃态的做法：照规约的恢复规则（择系统配置、择根、前缀按 D23 已定项 14 的五条口径）从镜像自己恢复出一版，恢复不出（整池拒或没有自证根）判不在；恢复出的这一版的投影（实例代号、checkpoint_txg、树表单元指针、中央映射树根指针、树 ID 水位、F）要等于版本表里某一版的投影，且这一版的下标在 [已确认, 已开始] 之间。版本表是基镜像上自己恢复出的现行那一版，加录制流里每一次根槽写按流序各一版，每版的投影取实现写进根槽的字节；已确认取崩溃态里落了的最晚一条写之前已返回的最晚一版（只从落了的写推），已开始取落了的写所属的最晚一版。系统配置槽只用来择槽、取根环与 journal 环的几何、判池级字段越界整池拒。所以恢复规则那一半是照 kb 文字另写的，允许集合那一半取自实现自己写出的根
```

c2. 「「三 oracle 合谋测试」是 E171（三oracle合谋测试）：……这处缺陷在修。」整句换成（装置取号、暖机出错 O1 不判那一处已修，页「历史版本」写「装置补了取号、暖机出错记红」，原句的「这处缺陷在修」是腐烂句）：

```
「三 oracle 合谋测试」是 E171（三oracle合谋测试），读数取自一棵冻住的树（快照摘要见实验页，与今天的 `crates/` 不是同一份）：O1 一列是模型对拍加模型的崩溃判官，不接 RefFS，与上表 O1 行不同（崩溃之后的集合那一半 E171 没用 RefFS）；O3 一列用 E170（O3规约执行器第一版） 第 1 次重跑冻结的那一份；手造坏镜像那一族只喂 O2、O3，期望出自 E170 的设计员，Q2 照登记字面判「有」（4 张全是 P8），按登记 F3 判成 E170（O3规约执行器第一版） 推期望推错、这四张不是坏镜像，真坏镜像里 O2、O3 两条全放行的 0 张；三条的源头是否独立（表第 4 列）E171 只在 Q2 量过 O2、O3 一对，O1 不在 Q2 里，没有量到的不写成「互相独立」被验证过。
```

c3. 射程末尾加一段：

```
**三条 oracle 都不判的两样**：一是无读故障的崩溃态上，D23 已定项 14 可写挂载那条见证判据为真（系统配置槽里的见证值超前于盘上读得出的记录）：崩溃注入第二截判拒绝合不合法时只在崩溃镜像上复算这条判据，为真就判合法（挂载透明的射程只管不带故障的状态，已定项 1），没有判官问「这个崩溃态没注入读故障，判据凭什么为真」（欠 C新甲）。二是一次发布内各写的先后：O2 的 I-7.12 判抬 F 那一串先写系统配置这一步；`layout/01-first-txn.md` 第八节的登记表把根槽 FUA 与系统配置槽轮换登记成两段，由钉住录制流的用例与门禁 doc-registries 比；其余写序不在三条之内。
```

**d. 依据：**第 173 行那条换成，再加一条 E171：

```
- 三个 oracle 的分工与各自独立性的来处是验证体系的结构设计；独立性靠「三 oracle 合谋测试」观测（E171）。
- E171（三oracle合谋测试）：在一棵冻住的树上，坏实现表里有三条 oracle 全部放行的行、有只靠一条才抓到的行，O3 在坏实现表上一行没红；手造的伪造根、伪造续接镜像里有 O2 不红而 O3 判不在的（读数与限定在实验页 Q1、Q2、Q3 三行）。
```

**e. 欠：**末尾那条 C407 换成，再加两条：

```
；C407（三 oracle 合谋测试的判官没进门禁）：E171（三oracle合谋测试） 已量过一次，没有一格门禁读它的产物；C新甲（无读故障的崩溃态上可写挂载见证判据为真没有判官）；C新乙（用户数据落点的取值算不算 layout 登记的字节没有一句说清）：已定项 8 射程与 layout 字节表互相矛盾，第 15 行归属悬着
```

### 3.2 实验页（`.claude/kb/experiments/171-三oracle合谋测试.md`）

**a. 「影响的决策」D13 已定项 6 那一行（原「备料」）换成：**

```
| D13（验证路线） 已定项 6 | 支撑 | 2026-10-03 改了：支撑定案里「独立性要被验证，不许自称」与「若三者全部放行，说明存在共同盲区，回头补 oracle」两句（Q1 在坏实现表上读到三者全部放行的行）；表第 4 列「独立性来自哪里」E171 只在 Q2 量过 O2、O3 一对，不支撑也不推翻「三个互相独立」；该分项依据段已引回本页 |
```

**b. 「它答不了的」加三条：**

```
- E171 的 O1 一列是模型对拍加模型的崩溃判官，不接 RefFS（登记第 36 行把崩溃判官算进 O1，第 193 行写本实验不用 RefFS），与 D13 表 O1 行把崩溃之后的集合归 RefFS 不同；两者的下界也不同（盘上已持久的最新根 vs 最后一次返回成功的发布）。读码读出来的：「已确认而丢了」那一类在 E171 的四列里只有 O3 判，没有在 37 行上跑过 RefFS。
- Q2 的 O3 独有坏镜像（`name=q2_family` 的 P 族与 B 族 `o3_only_strict` 之和）是 5 种补丁（P2、P3、P4、P5、P9）与盲测 3 型（P2、P3、P5）的实例，全是结构自洽的伪造根、伪造续接记录，正是 D13 表 O2 行「验不到什么」写的那一类；补丁与期望出自 E170 的同一份改法代码（E170 页「它答不了的」）；流一 P2、P5 两种补丁的 O2 一列随底态翻面。数写在这里，D13 依据段只指到本页。
- 这一页的行级读数都是快照摘要那棵树上的；今天的 `crates/` 与它不是同一份，页里的「全放行」「只靠一条」名单不能直接当今天实现的名单。
```

**c. 补位已跑（页要同改的几处，由 kb-scribe 照产物改）：**产物 `…-s4-f4-substitutes-comparator.out` 第 4631 行 `name=substitution_summary rows_that_did_not_compile=6 substituted_rows=6 still_not_compiled=0`，6 行换上的全是未触达（第 4625–4630 行 `category=untouched`），分母第 4521 行仍 `counted_D=37`，Q1 名单（第 4529 行）与 Q3 名单（第 4571 行）与补位之前逐字同。页标题状态、「现状」第 7 行、「编不过的 6 行补位（待跑）」一节、「它答不了的」第 460、463 行、`replay.sh` 第 1012 行的「另登一行」要一起改；补位产物没有 `replay.sh` 登记行。

### 3.3 欠账表（`.claude/kb/checks-owed.md`）

**a. C407 一行：**简称、「要拦什么」「怎么拦」「前置」四格换成：

```
简称：三 oracle 合谋测试的判官没进门禁
要拦什么：D13（验证路线） 已定项 6 逐字要求「独立性要被验证，不许自称」：设计一个「三 oracle 合谋测试」，拿一个精心构造的坏实现（例如 nonce 永远为 0）去过三关，若三者全部放行，说明存在共同盲区，回头补 oracle。测试本身已出（E171（三oracle合谋测试），比对器产物在 `research/results/`），但没有一格门禁读它：`grep -rn 合谋 .claude/gate.d` 只命中一句注释 ⇒ 产物里「三者全部放行」的行集合变了、某一条 oracle 失灵，门禁照绿
怎么拦：判官只读已入库的 E171 比对器产物，不重跑（重跑归 experiment-replay 格）：① `name=q1_alpha` 的 `row_numbers`（甲乙两种读法同，丙读法 `q1_gamma` 多第 233 行、丙另登记）逐字等于登记集合，登记集合里每一行挂一个欠账号，类别取补 oracle、写条款、登记按设计三条都不判；多一行少一行都红；② 三列（O1 = 对拍并崩溃判官、O2、O3）每列至少有一样东西只有它红：O1、O2 取 `name=q3_alpha` 的 `only_O1`、`only_O2` 非空，O3 取 `name=q2_family` 各族 `o3_only_strict` 之和大于 0（今天由手造的伪造根、伪造续接镜像供，不是坏实现）；③ 同一份产物 `name=verdict` 里 `positive_controls_not_red=0`、`a3_columns_with_red_on_the_baseline=0`、`negative_bases_with_o3_red=0`、`known_bad_rows_with_o2_quiet=0`（对照不失灵）。「关一条仍有别的腿红」不进判据，降为测量（Q3 在量）。判别力自证：把产物里 `only_O2` 清空、`o3_only_strict` 改成 0、多加一个全放行行号、把 `positive_controls_not_red` 改成 1，判官必须各红一次
前置：判官读的那份产物先要登记：补位那一趟的产物（`…-s4-f4-substitutes-comparator.out`）没有 `replay.sh` 行，实验页状态还写补位待跑；登记集合要一个文件（形态先例 `.claude/gate.d/e165-pinned-red-examples.tsv`，行由脚本追加不手改）；三条 oracle 的代码都在（O3 是 E170 的 Python 执行器），原前置「O3 零代码」已过期
```

**b. 新增两行**（号待 kb-scribe 取下一个没用过的，今天表里最大 C669）：

```
C新甲 | 无读故障的崩溃态上可写挂载见证判据为真没有判官 | 没有任何读故障的崩溃态，不许让 D23（journal 的角色与格式） 已定项 14 可写挂载那条见证判据为真：为真时正确实现的可写挂载在取号之前永久拒，一次普通断电之后池只剩只读挂载。E171（三oracle合谋测试） 第 242、269 行取到的崩溃态上，单镜像谓词 W 为真的态各 15、63 个（攻方副本脚本，入库装置没有 W），崩溃注入第二截 `judge_writable_mount_refusal` 把这类拒绝全判合法（它只在崩溃镜像上复算判据）。数是快照那棵树上的，今天的 core 在「判不出」一支上改过，没重做 | 崩溃注入第二截在没注入读故障的崩溃镜像上复算见证判据，为真即判拒得不合法；撕裂系统配置那一支（C664（挂载拒绝合法复算把两种拒因报成一个）、C331（择根倒挂压过已确认的写））先分开，否则基线上同样有的撕裂拒一起红。判别力自证：第 242、269 行变异在崩溃注入里必须由绿转红、基线 0 红 | 第二截要知道这个崩溃态注没注入读故障（D13 已定项 1 挂载透明的射程只管不带故障的状态）；先在今天的 core 上重数 W 与 61 个见证拒 | e171-conclusion-r3 判决（攻方在副本装置上量，入库装置没重做）
C新乙 | 用户数据落点的取值算不算 layout 登记的字节没有一句说清 | D3（空间分配） 已定项 8 射程写「改新池新建文件的字节：否」「登记的是字段与宽度不是取值」「违反它的实现写出的镜像完全合法」，已定项 10 射程写「改新池新建文件的字节：是」，`layout/01-first-txn.md` 第 67、174、192、210、309 行逐字写 t1 落 50180 并标已定 ⇒ E171（三oracle合谋测试） 第 15 行（用户数据挪到开放段起点）按 layout 读是真盲区、按已定项 8 读与第 34 行同是分配政策；两条已定条款互相矛盾，不在写回时判 | 用户判一次：用户数据落点的取值算不算 layout 登记的字节。算 ⇒ 已定项 8 射程那几句改、第 15 行留真盲区；不算 ⇒ layout 那几行 t1@50180 标成示例、第 15 行归分配政策。判完同改两处；门禁 doc-registries 不拦这种互相矛盾 | 无，要用户判 | e171-conclusion-r3 判决
```

### 2.5 补位之前与之后的两份产物，Q1、Q3 名单逐行比（原样输出）

```
$ for k in q1_alpha q1_beta q1_gamma q3_alpha q3_beta q1_column_red_rows: cmp <(grep …$A) <(grep …$B)
same: name=q1_alpha 
same: name=q1_beta 
same: name=q1_gamma 
same: name=q3_alpha 
same: name=q3_beta 
same: name=q1_column_red_rows
$ grep "name=q1_denominator " 两份产物
E7RESULT name=q1_denominator rows_run=281 counted_D=37 untouched=232 only_in_the_ignored_pass=0 compile_failed=6 anchor_not_unique=0 not_found=0 self_crash=6 timeout=0 device_stop=0 wrapper_ended=0 enough_to_judge_none_side=true
E7RESULT name=q1_denominator rows_run=281 counted_D=37 untouched=238 only_in_the_ignored_pass=0 compile_failed=0 anchor_not_unique=0 not_found=0 self_crash=6 timeout=0 device_stop=0 wrapper_ended=0 enough_to_judge_none_side=true
```

## 4 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1 句 1　关系 | 写宽一处 | 「E171 没测到源头独立」对 Q2 不成立（Q2 量了 O2、O3 一对，0 张真坏镜像两条全放行），但这一对的期望与补丁同源，不当独立的读数；关系「支撑」要与依据句同时落 |
| K1 句 2　依据 | 写宽两处、放错格一处 | 29 张数不在实验页（先补页）；29 是 5 种补丁加 3 型盲测的实例、全是伪造根与伪造续接，不是坏实现；要连「O3 在坏实现表 0 红」一起写；「O1 崩溃一列不是 RefFS」是射程，不是依据 |
| K1 句 3　O3 行 | 写窄四处、判决语一处、重复一处 | 漏版本 0；已确认、已开始没定义；系统配置槽漏整池拒；「每版内容」投影只有六样；「D13 第 110 行……与之一致」不进 kb；机制与后果两处各写一遍 |
| K1 句 4　全部故事 | 写窄 | 判决只给了依据、没给句子，漏记录核对器，C407 行里还有一份同句 |
| K1 句 5　写序与见证 | 写宽一处、放错格一处 | 「点名测试判一部分」对第 242 行不成立（点名的是失败路径冻结）；「段序列钉」只能写登记在册、判不判得出第 242 行没核；跨 oracle 的句子进射程加欠账，不进表最后一列 |
| K1 C407 改写 | 写宽三、写窄二 | 「补 oracle 的欠账号」「每一列」「O3 独有」写宽；「登记集合」「读哪份产物」写窄；补位产物没有复跑登记 |
| K1 名单与另几类归属 | 11 行一致，第 15 行冲突 | D3 已定项 8 射程与 layout 字节表对用户数据落点互相矛盾；第 15 行与第 34 行违反同一条定案、归属却相反；判决引的 ⑤ 表里没有 t1 |
| K1 句 1–5 与实验页「影响的决策」 | 对得上，须同落 | 今天「备料」与依据段「今天还不引它」互相一致；改「支撑」必须同次改依据，反过来依据引实验必须是支撑或推翻 |
| K4　W 的 15 / 63 态 | 可核，入库装置不能重做 | 我从 out 重数、在仓里重跑 8 条流同数；W 只在攻方目录里，读不入 git 的导出目录；标「副本脚本」 |
| K4　61 个见证拒 | 可核（s4 core），今天的 core 没重做，入库装置不能 | 9 + 52 重数同；52 里 6 个 `Undecidable` 落在今天 core 改过的一支；不写成今天的实现 |
| K4　第 125 行 D = 38 | 可核，入库装置不能 | 38 是改口径的预演；37 与 38 都写；装置对 panic 不导出 |
| K4　O3 独有 29 张 | 可核，入库产物里有，复跑不还原 | 12 + 17 逐行重数同；6 份产物都有；`replay.sh` 复跑读今天的树，不保证仍是 29 |
| K4　RefFS 27 / 27 与 36 态 | 可核，入库装置不能 | 27 绿 / 27 红 / 36 红重数同；RefFS 直调、崩溃落点手给；只进页，不进 D13 |
| K4　「83 个文件不同」 | 对象写错 | 83 是 s4 对第一轮副本；对今天是 105；不写数 |
| 产物与登记 | 缺一行登记 | 第二轮引的产物没有 `replay.sh` 行，页状态还写补位待跑 |

## 5 没做什么，要主 agent 跑的命令

没做什么（不归我的、没验的、跑不动的，照实列）：

- K2（名单完整性）、K3（判官能不能骗）是攻方腿的格：1.6 节只给文字落点，1.7 节只核第二轮已给的归属，没找「该进而没进」的行，没造骗判官的输入。不替主 agent 采纳。
- **没在今天的树上重做 61 个见证拒与 RefFS 27 / 27、36 态**：要编译 harness 档探针，`agent-common.md` 不让子 agent 编 Rust。脚本写好了：`research/prompts/e171-conclusion-r3-sonnet-model/rerun-on-r3-tree.sh`，`bash -n` 过，**没跑**。
- **没核「段序列用例在第 242 行变异上会不会红」**：要编译，是推的（1.5 节）。
- 没逐张核 29 张 O3 独有坏镜像在 E170 期望表里的期望（只核了数与补丁种类）。
- 没跑 `replay.sh` 的 E171 登记行（重型、约四小时），没起 E171 装置。
- 没读禁读清单里的文件（`research/prompts/e171-conclusion-r3-opus-output.md`、`research/prompts/e171-conclusion-r3-opus-model/`、`records/2026-10-01-里程碑三形式化接入.md`）。
- 草稿目录里我建的 O3 目录拷贝 `/tmp/claude-1000/e171-conclusion-r3-sonnet/o3copy`（`witness_consistency.py` 要它）：已在交回前删掉，W 的输出已原样贴在 2.2；`/tmp/claude-1000/e171-conclusion-r3-sonnet/recount-k4.out` 与 `w/` 留着，不是编译目录也不是工作树。

要主 agent 跑的命令（一条）：

```
工作目录：/home/fy5090/code/singlefs
命令：bash research/prompts/e171-conclusion-r3-sonnet-model/rerun-on-r3-tree.sh /tmp/claude-1000/e171-conclusion-r3-rerun 15
环境变量：脚本里已设 RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON、PYTHONDONTWRITEBYTECODE；不用另设
内存上限与线程数：编译 6G，批次 8G，线程 15（脚本写死，沿用第二轮攻方 tools/rerun.sh 的数）
跑完看哪几行：out/judge-summary-r3.out 对 out/expected-judge-summary-r3.out 的 diff（脚本末尾第 7 步已打）；
  out/refusal-judge-summary-r3.out 里 core=r3 的 r242、r269 行：见证类拒（AgainstTheRecordAtTheWitnessedCounter、
  AgainstTheSelectedVersionsLastRecord、Undecidable）三种理由的计数之和，第二轮 s4 core 上是 9 与 52
结论怎么用：和第二轮同数 ⇒ 第 2 节表里「今天的 core 没重做」那一条可去掉，kb 可以写成「s4 树与今天的树同数」；
  不同 ⇒ 不同的行单列，kb 只写 s4 树的数
```

脚本编译用的是第一轮攻方的 `workspace-Cargo.toml` 与 `workspace-Cargo.lock`（今天的树与第一轮副本的 crate 清单与各 `Cargo.toml` 逐字节同，`cat */Cargo.toml | sha256sum` 两边相同）。

## 6 cite-check 末行

```
$ python3 research/scripts/cite-check.py research/prompts/e171-conclusion-r3-sonnet-output.md --root /tmp/claude-1000/e171-conclusion-r3/tree --root /tmp/claude-1000/e171-s4/frozen-tree --root . --background research/prompts/_e171-conclusion-r3-background.md
  ✓ 核了 41 处引文，对上 41 处，没判 0 处（逐处列在上面）
```
