# 逐行判 S32：全仓知识腐烂治理 2026-10-17（批4），组 F831-F838、F840-F845、F848-F849

范围：基准 ce1457c3，结束在工作区（读文件）。候选表 `/home/fy5090/.local/share/singlefs/work/rot-2026-10-17/candidates.tsv`，我的 16 组共 299 行（取法：`awk -F'\t' '$1=="F831"||…||$1=="F849"' candidates.tsv`，对应草稿 `mine.tsv`，`wc -l` 得 299）。
没建副本，没起编译，没跑重型测试，没改任何被判的文件。

做法：每行先按原文在文件里定位（299 行原文全部还在，没有「原文已不在」），再取检索词命中处前后的上下文判；数字类现数：
- 不变量已实现：`grep -ch '^| I-.*已实现' crates/singlefs-checker/invariants/0*.md | awk '{s+=$1} END{print s}'` 得 92，`IMPLEMENTED_INVARIANTS: [&str; 92]`（`crates/singlefs-checker/src/image.rs` 第 112 行）。
- 崩溃枚举用例登记行：`grep -c '^crash-case:' .claude/gate.d/stage-inputs.tsv` 得 14。
- 新池新建文件流 `FULL_STATES` = 16_777_260（`crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs` 第 705 行）；固定脚本流到正常卸载全量 1 678 377 204（`scripts/crash/crash-point-replay-shard-run.sh` 文件头登记与 `crash_enumeration_fixed_script_stream.rs`）。
- `.gitignore` 现为 32 行，没有 `/research/`；`rustup target list --installed` 有 musl 与 gnu 两个目标。

判法：带日期的批次记录、改名登记、三方记录里的数与做法判「事件句不改」；已经写成新说法的现状句判「不相干」；undated 的现状句、还没做的做法列、指错路径的判「要改」并给改后的句子。

## 逐行判定

| 组 | 载体 | 判定 | 改后的句子或理由 |
|---|---|---|---|
| F831 | .claude/kb/checks-owed.md:675 | 事件句不改 | 已还清表里 C328 的当日记录：「2026-09-14 用户收尾弹窗定」「（2026-09-14 实测）」，262165 是当天崩溃点重放的状态数，说的是那一次做出的事。 |
| F831 | .claude/kb/checks-owed.md:677 | 事件句不改 | 已还清表里 C322 的当日定案记录：「2026-09-14 三轮三方后用户收尾弹窗定案」，「262165 个状态违例 2 → 0」是当天的前后对比，不是今天的状态数。 |
| F831 | .claude/kb/experiments.md:164 | 事件句不改 | 说的是旧布局：「旧 8 单元布局下的判决（宽度 28 格全对、21 条写、崩溃点重放的 262165 个状态零违例等）见实验页历史版本」，262165 本来就标成旧布局的数。（本行开头原话：「E142（新池新建文件的干跑）   部分已跑（202」） |
| F831 | .claude/kb/experiments/142-新池新建文件的干跑.md:21 | 事件句不改 | E142 页这一段是第七次跑的判决行：「第七次跑收口了三处」「判决行的七格照旧全过」，262165 是第七次跑时主臂的状态数，不是今天的流。 |
| F831 | .claude/kb/experiments/142-新池新建文件的干跑.md:189 | 事件句不改 | 这一格标明读的是旧流：「这一格量的是第十七次跑那条流」「旧几何，8 单元、33 次写、262165 个状态」，是 2026-09-25 对账时的读数，说的是那一次的事。 |
| F831 | .claude/kb/experiments/142-新池新建文件的干跑.md:193 | 事件句不改 | 「第十七次跑第二段 2026-09-25 的读数」，「262165 个状态那一批里数出来的 3 个」只是在交代这个数的来历，说的是那一批。 |
| F831 | .claude/kb/experiments/142-新池新建文件的干跑.md:309 | 要改 | 「它答不了的」一节是现状话，262165 已不是自然状态总数：崩溃点重放全量枚举的自然状态（第十四次跑当时是 262165 个，现新池新建文件流全部 16 777 260 个）里这两格恒为 0（`name=header311_reader_branch_counts`，第十四次跑的读数）；`crates/` 的崩溃点重放流 L8（并行线三跨记录崩溃点重放流） 管这一形态，这个装置答不了。 |
| F831 | .claude/kb/milestone/01-first-txn.md:159 | 事件句不改 | 「现状（2026-09-14 开工，验收六条全过）」下交代 C322 当天怎么落的：「录制流与 262165 个状态不变」是那一次改动前后的对比。 |
| F831 | .claude/kb/milestone/01-first-txn.md:217 | 事件句不改 | 「现状（2026-09-14 开工，验收两条全过…）」报的是当天验收跑的读数：「全量 262165 个状态 = 闭式，本机 release 下 22.7 s」，是那一次跑出来的，不是今天的状态数。 |
| F831 | .claude/kb/milestone/01-first-txn.md:224 | 事件句不改 | 「E142 第六次跑崩溃点重放是 262165 个状态」是第六次跑的读数，带产物名 `e142-first-txn-dry-run-2026-09-13-settled6.out`，说的是那一次。 |
| F831 | .claude/kb/milestone/02-second-txn.md:57 | 事件句不改 | 这是第二步当时验收要过的门禁清单：「崩溃点重放全量状态数仍 262165，计数与新产物逐字比」，「仍」是对那一步改动前后的比较，里程碑二早已收尾。 |
| F831 | .claude/kb/milestone/02-second-txn.md:64 | 事件句不改 | 「复核（主 agent，提交 d2aeb7d 之后按新基准重审）」：第八次跑产物 `states=262165` 是那次复核逐字读到的。 |
| F831 | .claude/kb/milestone/02-second-txn.md:68 | 事件句不改 | 「现状（2026-09-17 步 5 落地之后）」：「闭式 2104413 个状态」是步 5 落地当天的数，说的是那一步做出来的样子，之后的变化记在崩溃放量那几段。 |
| F831 | .claude/kb/milestone/02-second-txn.md:198 | 事件句不改 | 「现状（2026-09-17 落地）」下列这一步落了什么：「固定脚本到 E 54 段、2104413 个状态（快的那条 108 个）」是步 5 当天的读数，不是今天的流。 |
| F831 | .claude/kb/milestone/02-second-txn.md:227 | 要改 | 「崩溃点重放的负载是两条流（新池新建文件；固定脚本到 E，54 段、2104413 个状态）」是现状句，只有新池新建文件那一半后补了 2026-10-11 注。改为：「崩溃点重放的负载是两条流（新池新建文件，全部 16 777 260 个崩溃状态；固定脚本到正常卸载，共 1 678 377 204 个，2026-09-17 当时是到 E 的 54 段、2104413 个状态）」，其后的 2026-09-17 零违例读数照旧是事件句。 |
| F831 | records/2026-09-13-总审核.md:375 | 事件句不改 | 总审核发现表的一行：「262165 个崩溃状态怎么物化成镜像，全仓没有条款」，是 2026-09-13 审核当时的发现，记录是建设过程。 |
| F831 | records/2026-09-13-总审核.md:401 | 事件句不改 | 总审核记录里 E142 第七次跑的结果：「崩溃点重放 262165 个状态零违例，25 单测、变异表全抓」，说的是那一次跑。 |
| F831 | records/2026-09-13-总审核.md:421 | 事件句不改 | 总审核记录的步 7 落地叙述：「全量 262165 个状态零违例…本机 release 22.7 s」是当天的读数。 |
| F831 | records/2026-09-13-总审核.md:449 | 事件句不改 | 总审核记录：「崩溃点重放全量（262165 个状态，release 下 170.9 s）」是那一次跑的结果，「C322 还开着」也是当时的状态。 |
| F831 | records/2026-09-13-总审核.md:454 | 事件句不改 | 总审核记录里 I-7.8 的发现：「按字面，崩溃点重放全量 262165 个状态里 261891 个判红」是当时算出来的数。 |
| F831 | records/2026-09-13-总审核.md:467 | 事件句不改 | 总审核记录的验证：「崩溃点重放全量 262165 个状态 checker 违例 0（此前 I-7.7 两个）」是那一次修完后的读数。 |
| F831 | records/2026-09-21-增补3第4件与checker补三条.md:31 | 事件句不改 | 增补3记录：「崩溃点重放全量（release）262165 状态零违例」是补三条不变量那一次跑的结果。 |
| F831 | records/2026-09-21-增补3第4件与checker补三条.md:40 | 事件句不改 | 增补3记录：「补 I-1.8 之前，崩溃点重放全量 262165 个状态零违例，这个 bug 一直看不见」，说的是补之前那个时点的事。 |
| F831 | records/2026-09-24-里程碑二收尾调度.md:161 | 事件句不改 | 里程碑二收尾调度记录（批次快照）：「仍是 18 与 262165」是 2026-09-25 当时 `crates/` 用例钉的旧数，去向表交代的是那天的事。 |
| F831 | records/2026-09-24-里程碑二收尾调度.md:163 | 事件句不改 | 里程碑二收尾调度记录（批次快照）：「今天的段序列…钉的旧数组是 2104413」是 2026-09-25 量到的读数。 |
| F832 | .claude/kb/milestone/03-third-txn.md:61 | 不相干 | 说的数是对的：「checker 判 92 条不变量」，与现数 `grep -ch '^｜ I-.*已实现'` 求和 92 和 `IMPLEMENTED_INVARIANTS: [&str; 92]` 一致，不受 29 条这件事实影响。 |
| F832 | .claude/kb/verification-build.md:138 | 不相干 | 这一句已经是新说法：「92 条，`IMPLEMENTED_INVARIANTS` 的长度」，「最初的 29 条是」明说是最初的数，现数 92 对得上。 |
| F832 | CLAUDE.md:112 | 不相干 | 「checker 判 92 条不变量」与现数 92 一致，「与代码清单 `IMPLEMENTED_INVARIANTS` 同数」，不是旧的 29。 |
| F832 | records/2026-09-16-subagent拆分提案.md:1096 | 不相干 | 这一条只是提议往 `IMPLEMENTED_INVARIANTS` 追加时要拦的检查，没有写条数，说的不是池级 checker 判几条。 |
| F832 | records/2026-09-16-subagent拆分提案.md:1097 | 不相干 | 「第二次（同一天，删除与并发那条线第 22 批）」只提到 `IMPLEMENTED_INVARIANTS` 这个名字，没有写条数，说的是慢档用例红的事。 |
| F832 | records/2026-09-24-里程碑二收尾调度.md:177 | 事件句不改 | 里程碑二收尾调度记录（批次快照）：「`IMPLEMENTED_INVARIANTS` 46 → 47」是当时那一批加一条的前后对比。 |
| F832 | records/2026-09-24-里程碑二收尾调度.md:301 | 事件句不改 | 里程碑二收尾调度记录（批次快照）：「`IMPLEMENTED_INVARIANTS` 48 → 49」是当时新立 I-7.14 那一批的前后对比。 |
| F832 | records/2026-09-30-里程碑三开工第一轮.md:369 | 事件句不改 | 里程碑三开工第一轮记录（批次快照）：「已实现 65 = `IMPLEMENTED_INVARIANTS` 65」是当时写回员甲交回时的数。 |
| F832 | records/2026-09-30-里程碑三开工第一轮.md:514 | 事件句不改 | 里程碑三开工第一轮记录（批次快照）：「对方报 `IMPLEMENTED_INVARIANTS` 68 条」是当时那一轮的数。 |
| F832 | records/2026-09-30-里程碑三开工第一轮.md:539 | 事件句不改 | 里程碑三开工第一轮记录（批次快照）：「第 3 条 I-9.28（`IMPLEMENTED_INVARIANTS` 69 条）」是当时那一批做完的数。 |
| F832 | records/2026-09-30-里程碑三开工第一轮.md:704 | 事件句不改 | 里程碑三开工第一轮记录（批次快照）：「`IMPLEMENTED_INVARIANTS` 改 84 条」是当时那一批改成的数。 |
| F832 | records/2026-10-01-删除与并发事务.md:729 | 不相干 | 「F6（`IMPLEMENTED_INVARIANTS` 追加次序）主 agent 照报告建议定」说的是追加次序，没有写条数。 |
| F832 | records/2026-10-01-删除与并发事务.md:788 | 事件句不改 | 删除与并发事务记录（批次快照）：「`IMPLEMENTED_INVARIANTS` 末尾追加五条（下标 69–73）」是当时第 18 批的追加。 |
| F832 | records/2026-10-01-删除与并发事务.md:789 | 事件句不改 | 删除与并发事务记录（批次快照）：「`IMPLEMENTED_INVARIANTS` 仍是 69 条」是当时主 agent 现查的数。 |
| F832 | records/2026-10-01-删除与并发事务.md:814 | 事件句不改 | 删除与并发事务记录（批次快照）：「= 74 = `IMPLEMENTED_INVARIANTS` 长度」是当时那一批写后的数。 |
| F832 | records/2026-10-01-删除与并发事务.md:924 | 不相干 | 这一条是教训：「加 `IMPLEMENTED_INVARIANTS` 的实现员要连同 checker 档里按不变量名单计数的用例一起改」，没有写条数。 |
| F832 | records/2026-10-01-删除与并发事务.md:935 | 事件句不改 | 删除与并发事务记录（批次快照）：「`IMPLEMENTED_INVARIANTS` 74 → 83（下标 74–82…）」是当时第 22 批的前后对比。 |
| F832 | records/2026-10-01-删除与并发事务.md:955 | 事件句不改 | 删除与并发事务记录（批次快照）：「= 83 = `IMPLEMENTED_INVARIANTS` 长度」是第 23 批合入时的数。 |
| F832 | records/2026-10-01-删除与并发事务.md:1098 | 事件句不改 | 删除与并发事务记录（批次快照）：「`IMPLEMENTED_INVARIANTS` 82 → 83」是 10o 加 I-3.14 那一次的前后对比。 |
| F832 | records/2026-10-01-删除与并发事务.md:1424 | 事件句不改 | 删除与并发事务记录里当天简报的数：「不变量 kb 现数 90、`IMPLEMENTED_INVARIANTS` 92（I-9.36、I-9.37 待写回），README 还写 49」是 2026-10-01 的读数，批次快照。 |
| F832 | records/2026-10-01-里程碑三形式化接入.md:698 | 不相干 | 三方形式化接入记录里交代约束：「`judge` 断言 ID 在 `IMPLEMENTED_INVARIANTS` 里，表长等于 `invariants.md`「已实现」行数」，没有写条数，说的是机制。 |
| F832 | records/2026-10-01-里程碑三形式化接入.md:769 | 事件句不改 | 形式化接入记录（批次快照）：「`IMPLEMENTED_INVARIANTS` 68 条、下标 65 是 I-3.5」是当时主 agent 判交回点时的数。 |
| F832 | records/2026-10-01-里程碑三形式化接入.md:817 | 事件句不改 | 形式化接入记录（批次快照）：「`IMPLEMENTED_INVARIANTS` 变 69 条」是当时立 I-9.28 那一批的结果。 |
| F832 | records/2026-10-01-里程碑三形式化接入.md:891 | 不相干 | 形式化接入记录里的一句：「I-9.5 池级 checker 没实现（`IMPLEMENTED_INVARIANTS` 里没有）」说的是当时某条没在表里，没有写条数，也是当时的事。 |
| F832 | records/2026-10-01-里程碑三形式化接入.md:990 | 事件句不改 | 形式化接入记录（批次快照）：「IMPLEMENTED_INVARIANTS 末尾追加 69–73」是当时派发的约定，不是今天的状态。 |
| F832 | records/2026-10-01-里程碑三形式化接入.md:1144 | 事件句不改 | 形式化接入记录（批次快照）：「第一次红来自 `deee47f4`（把 I-5.5 加进 `IMPLEMENTED_INVARIANTS`…）」是调查某一次红的经过。 |
| F832 | records/2026-10-11-里程碑三知识腐烂治理.md:30 | 事件句不改 | 2026-10-11 知识腐烂治理记录（批次快照）：「主 agent 现数（课题 3）…`[&str; 92]`」是当天的读数，现数仍是 92，也没有变。 |
| F833 | .claude/kb/checks-owed.md:547 | 事件句不改 | C793 一行叙述 2026-10-12 全量里连红三次的经过：「第三次（2026-10-12 全量 n）录入段写进了默认库 `library-by-conclusion-kind`」，说的是那一次的事，驱动已改。 |
| F833 | .claude/kb/milestone/02-second-txn.md:484 | 事件句不改 | 里程碑二放量「现状（落地）」一条：「三段流第 ①②③ 段落地（`crash_facts.rs`、`crash_verify_gpu.rs`…）」说的是 2026-09-28 那一批落了什么，文件现在都还在，没有说两段的事。 |
| F833 | .claude/kb/term-renames.md:24 | 事件句不改 | 改名登记里的叙述：「2026-10-13 那一批改了，登在文末「第一条流 / 第二条流、两段流水线、三层表的档 → 按内容起的名字（2026-10-13 起）」一节」，是改名过程。 |
| F833 | .claude/kb/term-renames.md:27 | 事件句不改 | 这是改名登记节的标题，节名里的「两段流水线」是被改掉的旧叫法，登记它就是为了留旧名对照。 |
| F833 | .claude/kb/term-renames.md:31 | 事件句不改 | 改名登记：「崩溃放量两段流水线：第一段 → 录入段（流先入库），第二段 → 判定段」，写的是旧名到新名，新名已是现在的用词。 |
| F833 | .claude/kb/verification-build.md:265 | 不相干 | 引的是代码里仍在的名字：`crash_amplification.rs` 文件头第 2 条「先录后核」与「录入段」那一节，现查文件头仍是「2. **先录后核**」，没有过时。 |
| F833 | .claude/rules/verification.md:13 | 不相干 | 这是 checker 档各模块的分工：「先录后核的流水线 `crash_amplification`」「三段流第 ① 段的事实表…`crash_facts`」，这几个文件现都还在（`ls crates/singlefs-checker-tier/src`），同一行前面已写了两段与长期判定库。 |
| F833 | records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md:54 | 事件句不改 | 里程碑三崩溃放量调度记录（批次快照）：「接着落三段流那一批进主仓」说的是 2026-09-28 那天的经过。 |
| F833 | records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md:55 | 事件句不改 | 里程碑三崩溃放量调度记录（批次快照）：「落三段流 → 主仓验证、证红 5 条 → 冻树」是那天的计划链。（本行开头原话：「到 D 全域收口：51,773,558 态、0 红 」） |
| F833 | records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md:56 | 事件句不改 | 里程碑三崩溃放量调度记录（批次快照）：「用户定 GPU 判器现在做；三段流第 ①②③ 段落代码」是那天的经过。 |
| F833 | records/2026-09-30-里程碑三开工第一轮.md:983 | 事件句不改 | 开工第一轮记录（批次快照）：「默认库换成 `<家目录>/library-by-conclusion-kind`」是 2026-09-30 那一批的改动，之后改成一份 crash-state-verdicts。 |
| F833 | records/2026-10-11-崩溃放量两段与脚本审计.md:51 | 事件句不改 | 崩溃放量两段与脚本审计记录里 2026-10-11 的进度条：「课题 3、4、5、6 做完…判定库按测试类各一份长期库」是那一天的形态，之后已改成一份长期库。 |
| F833 | records/2026-10-11-崩溃放量两段与脚本审计.md:75 | 事件句不改 | 同一份记录里 2026-10-12 的一条：「n 的崩溃点重放 步退 1…`record.log` 第 15 行 `library=…/library-by-conclusion-kind` 坐实」，是那一次红的来路。 |
| F833 | records/2026-10-11-崩溃放量两段与脚本审计.md:138 | 事件句不改 | 记录里的用户定名弹窗：「用户选「按提议的名字改」——第一条流 / 第二条流 → 新池新建文件流 / 固定脚本流」，说的是那一次定案。 |
| F833 | records/2026-10-13-复用键与全仓改名.md:6 | 事件句不改 | 复用键与全仓改名记录（批次快照）开头的课题原话，指到改名登记，说的是 2026-10-13 那一批的事。 |
| F833 | records/2026-10-13-复用键与全仓改名.md:95 | 事件句不改 | 复用键与全仓改名记录（批次快照）：「轻型放量改用长期库 `library-by-conclusion-kind`（与重型同一份）」是那一批当时的出口，之后库名改成 crash-state-verdicts。 |
| F833 | records/2026-10-13-复用键与全仓改名.md:637 | 事件句不改 | 复用键与全仓改名记录：「清盘（用户 2026-10-15 定「空库跑…」）」是那一天清掉旧库的操作，库名只是被清的对象。 |
| F834 | .claude/kb/checks-owed.md:735 | 不相干 | 这一行在 checks-owed 的已还清表里（`### 已还清` 在第 576 行，C626 在第 735 行），表里就是已还清的条目，没有说开着。 |
| F834 | .claude/kb/verification-build.md:294 | 不相干 | 这一句已是新说法：「C626（走 GPU 的计算类没有登记） 已还清，用户定这一项不得欠账」，登记表 `.claude/gate.d/gpu-exemptions.tsv` 现在也存在。 |
| F835 | records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md:71 | 事件句不改 | 崩溃放量调度记录（批次快照）：「kb 决策正文（D13 已定项 15「GPU 只接校验和那一截…」那句）等这一批代码进了再写回」是当天的待办，之后写回已做（decisions-history 6184 行）。（本行开头原话：「用户定：崩溃点网络模型（口述，主 agent 整理，」） |
| F835 | records/2026-10-01-删除与并发事务.md:358 | 事件句不改 | 删除与并发事务记录（批次快照）：「下一批写回…D13 已定项 15「GPU 只接校验和那一截」改现状」是当时的待办，之后已写回。 |
| F835 | records/2026-10-01-对接上游测试脚本规范.md:370 | 事件句不改 | 2026-10-01 记录里抄的当时 D13 定案正文：「GPU 只接校验和那一截（E163）、要不要接归 D24」是那天的原文，记录是建设过程。（本行开头原话：「**定案**：`crates/` 下的验证代码按「什」） |
| F835 | records/2026-10-01-对接上游测试脚本规范.md:374 | 事件句不改 | 与上一行同一段正文：「GPU 只接校验和那一截…要不要接归 D24」是 2026-10-01 对接规范那一批抄的当时定案。（本行开头原话：「**定案**：`crates/` 下的验证代码按「什」） |
| F836 | .claude/kb/pitfalls.md:25 | 不相干 | 这一行已是新说法：「⏸ 没有条款管它（D9（加密） 已全部定案，密文复制 / raw send 这一格没立…」，不再指向 D9 未定。 |
| F836 | .claude/kb/prior-art.md:288 | 不相干 | 「scrub / resilver / 重命名 / 删除 / raw send」是上游文献对无 key 能做什么的比较，与本工程 pitfalls 第 13 条的状态无关。 |
| F836 | .claude/kb/prior-art.md:293 | 不相干 | 「MAC 树之所以不做成 Merkle 树，是因为 raw send 不传间接块」是对 ZFS 机制的引述，不是本工程的状态。 |
| F836 | .claude/kb/prior-art.md:295 | 不相干 | 引文：「going up the bp tree. However, this presents a problem for raw sends.」是上游原文。 |
| F836 | .claude/kb/prior-art.md:342 | 不相干 | 上游 issue 的事实：「该 issue 下 2025-05-31 称修掉两个非 raw send 的加密 bug」，是文献证据，不是本工程现状。 |
| F837 | .claude/kb/decisions/17-实现分层与第三方管道.md:86 | 要改 | 改为：「QEMU 档跑的是静态二进制吃 `/dev/vda`：实验二进制是 musl 静态的，checker 档带 RocksDB 的真设备二进制用 `x86_64-unknown-linux-gnu` 加 `+crt-static` 编成 static-pie（[vm-harness.md](../vm-harness.md)）。」现状见 `.claude/kb/vm-harness.md` 第 36 行与 `.claude/kb/tooling.md` 第 106 行，原句「QEMU 档跑的就是静态 musl 二进制」对 checker 档不成立。 |
| F837 | .claude/kb/experiments/07-离线索引harness.md:166 | 不相干 | 实验二进制的编法或复跑命令，实验二进制本来就用 musl，与 checker 档二进制另用 gnu 加 +crt-static 不冲突：「/vm-bench.sh <musl 二进制> 11 none 」 |
| F837 | .claude/kb/experiments/07-离线索引harness.md:189 | 不相干 | 实验二进制的编法或复跑命令，实验二进制本来就用 musl，与 checker 档二进制另用 gnu 加 +crt-static 不冲突：「m-harness.md)：musl 静态编译 + `scrip」 |
| F837 | .claude/kb/experiments/07-离线索引harness.md:280 | 不相干 | 实验二进制的编法或复跑命令，实验二进制本来就用 musl，与 checker 档二进制另用 gnu 加 +crt-static 不冲突：「**：`e7-index`，musl 静态，xorshift64」 |
| F837 | .claude/kb/experiments/12-攒批的顺序追加vs不攒批的随机页读改写.md:269 | 不相干 | 实验二进制的编法或复跑命令，实验二进制本来就用 musl，与 checker 档二进制另用 gnu 加 +crt-static 不冲突：「12-lifecycle`，musl 静态，xorshift64」 |
| F837 | .claude/kb/experiments/159-fsync等待时间随并发数组提交与wal两臂.md:56 | 不相干 | 实验二进制的编法或复跑命令，实验二进制本来就用 musl，与 checker 档二进制另用 gnu 加 +crt-static 不冲突：「unknown-linux-musl --bin e159-fs」 |
| F837 | .claude/kb/experiments/159-fsync等待时间随并发数组提交与wal两臂.md:57 | 不相干 | 实验二进制的编法或复跑命令，实验二进制本来就用 musl，与 checker 档二进制另用 gnu 加 +crt-static 不冲突：「/vm-bench.sh <musl 二进制路径> probe-」 |
| F837 | .claude/kb/experiments/159-fsync等待时间随并发数组提交与wal两臂.md:58 | 不相干 | 实验二进制的编法或复跑命令，实验二进制本来就用 musl，与 checker 档二进制另用 gnu 加 +crt-static 不冲突：「/vm-bench.sh <musl 二进制路径> cell -」 |
| F837 | .claude/kb/experiments/53-丢一整块盘之后根环还挂不挂得上.md:47 | 不相干 | 实验二进制的编法或复跑命令，实验二进制本来就用 musl，与 checker 档二进制另用 gnu 加 +crt-static 不冲突：「- 二进制必须是静态的（musl 目标）。」 |
| F837 | .claude/kb/experiments/53-丢一整块盘之后根环还挂不挂得上.md:56 | 不相干 | 实验二进制的编法或复跑命令，实验二进制本来就用 musl，与 checker 档二进制另用 gnu 加 +crt-static 不冲突：「unknown-linux-musl --bin e53_rin」 |
| F837 | .claude/kb/experiments/53-丢一整块盘之后根环还挂不挂得上.md:57 | 不相干 | 实验二进制的编法或复跑命令，实验二进制本来就用 musl，与 checker 档二进制另用 gnu 加 +crt-static 不冲突：「unknown-linux-musl/release/e53_r」 |
| F837 | .claude/kb/experiments/53-丢一整块盘之后根环还挂不挂得上.md:103 | 不相干 | 实验二进制的编法或复跑命令，实验二进制本来就用 musl，与 checker 档二进制另用 gnu 加 +crt-static 不冲突：「unknown-linux-musl --bin e53_rin」 |
| F837 | .claude/kb/experiments/53-丢一整块盘之后根环还挂不挂得上.md:104 | 不相干 | 实验二进制的编法或复跑命令，实验二进制本来就用 musl，与 checker 档二进制另用 gnu 加 +crt-static 不冲突：「unknown-linux-musl/release/e53_r」 |
| F837 | .claude/kb/experiments/56-消息缓冲的收益vsε.md:330 | 不相干 | 实验二进制的编法或复跑命令，实验二进制本来就用 musl，与 checker 档二进制另用 gnu 加 +crt-static 不冲突：「m-harness.md)：musl 静态编译 + `scrip」 |
| F837 | .claude/kb/experiments/56-消息缓冲的收益vsε.md:344 | 不相干 | 实验二进制的编法或复跑命令，实验二进制本来就用 musl，与 checker 档二进制另用 gnu 加 +crt-static 不冲突：「unknown-linux-musl/release/e56_e」 |
| F837 | .claude/kb/tooling.md:106 | 不相干 | 这一行已是新说法：「checker 档的真设备二进制 `new_pool_file_creation_on_device` 不走它…改用 `--target x86_64-unknown-linux-gnu` 加 `+crt-static` 编成 static-pie」。 |
| F837 | .claude/kb/verification-build.md:199 | 不相干 | 环境表写的是「musl 目标已装」，现查 `rustup target list --installed` 有 x86_64-unknown-linux-musl 与 x86_64-unknown-linux-gnu，说的是装了什么，仍是真话。 |
| F837 | .claude/kb/vm-harness.md:11 | 不相干 | 实验二进制的编法或复跑命令，实验二进制本来就用 musl，与 checker 档二进制另用 gnu 加 +crt-static 不冲突：「unknown-linux-musl --bin <实验二进制名」 |
| F837 | .claude/kb/vm-harness.md:12 | 不相干 | 实验二进制的编法或复跑命令，实验二进制本来就用 musl，与 checker 档二进制另用 gnu 加 +crt-static 不冲突：「unknown-linux-musl/release/<名> /」 |
| F837 | .claude/kb/vm-harness.md:36 | 不相干 | 这一行已是新说法：实验二进制用 musl 编，「带 RocksDB 的 checker 档二进制用 `--target x86_64…」另编。 |
| F837 | records/2026-09-24-里程碑二收尾调度.md:286 | 事件句不改 | 里程碑二收尾调度记录（批次快照）：「`e162_verdict_store_sender` musl 静态」是 2026-09-24 那一批写的要点，说的是实验二进制。 |
| F837 | records/2026-10-01-里程碑三形式化接入.md:1099 | 事件句不改 | 里程碑三形式化接入记录（批次快照）：「merge-and-verify 加…musl 虚机二进制编译在后台跑」是当时合入验证的做法。 |
| F837 | records/2026-10-01-里程碑三形式化接入.md:1101 | 事件句不改 | 里程碑三形式化接入记录（批次快照）：「musl release `new_pool_file_creation_on_device` 都退 0」是 2026-10-01 当时的结果，之后该二进制改成 gnu 加 crt-static。 |
| F837 | records/2026-10-01-门禁注释精简.md:88 | 事件句不改 | 门禁注释精简记录：「6 台并行同跑 direct 两轮（同一个 musl 二进制…）」是当时的并行实验。 |
| F837 | records/2026-10-11-scripts审计.md:17 | 事件句不改 | scripts 审计记录（批次快照）：「报「musl 构建失败：rustup target add…」，病因也写错」是审计当时发现的脚本毛病。 |
| F837 | records/2026-10-11-崩溃放量两段与脚本审计.md:70 | 事件句不改 | 崩溃放量两段与脚本审计记录里 2026-10-12 的条目：「虚机二进制…编不过」，随后改成 gnu 加 crt-static，是这次改法的来路。 |
| F838 | .claude/kb/experiments/139-按盘回退下界的收严形态.md:296 | 不相干 | 这一行提到的是 `research/results` 下的产物路径，不是说 `research/` 不进 git：「T name=verdict_f_' ../../research/results/e139-tightened-f」 |
| F838 | .claude/kb/experiments/168-删除借保留量的环窗口.md:48 | 不相干 | 「被 .gitignore 第 32 行忽略」现查 `.gitignore` 第 32 行正是 `/research/results/e168-delete-ring-window-borrow-segment1-2026-10-03.out`，说的是单个超大产物，仍是真话。 |
| F838 | .claude/kb/experiments/170-O3规约执行器第一版.md:790 | 不相干 | 这一行提到的是 `research/results` 下的产物路径，不是说 `research/` 不进 git：PYTHONDONTWRITEBYTECODE=1 python3 comparator_q2_ |
| F838 | .claude/kb/experiments/170-O3规约执行器第一版.md:791 | 不相干 | 这一行提到的是 `research/results` 下的产物路径，不是说 `research/` 不进 git：「 --freeze-reference <仓根>/research/results/e170-r1-o3-spec-」 |
| F838 | .claude/kb/experiments/174-借据丢掉时等待与显存回还.md:79 | 不相干 | 这一行提到的是 `research/results` 下的产物路径，不是说 `research/` 不进 git：「ome/fy5090/code/singlefs/research/results」 |
| F838 | .claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md:34 | 不相干 | 这一行提到的是 `research/results` 下的产物路径，不是说 `research/` 不进 git：「en-crates-2/repo; R=$PWD/research/results/e175-product-pat」 |
| F838 | .claude/kb/tooling.md:74 | 不相干 | 这一行已是新说法：「`scripts/` ｜ 实验代码与脚本 ｜ 跑的人 ｜ 进（`.gitignore` 只排除 `target/` 与个别超大文件）」。 |
| F838 | records/2026-08-29-审计轮-外部引用复核与阻塞集.md:207 | 事件句不改 | 2026-08-29 审计轮记录的发现：「`.gitignore` 第 9 行：`/research/`」是那天审计时仓里的样子，现查 `.gitignore` 里已没有 /research/ 这一行（第 10 行注释写 research/ 入库），记录是过程。 |
| F838 | records/2026-08-29-审计轮-外部引用复核与阻塞集.md:211 | 事件句不改 | 同一审计记录的发现：「这不是疏漏，是写进 `.gitignore` 的一次选择」，说的是 2026-08-29 当时，现在 `.gitignore` 注释已写「research/ 入库（2026-08-29 用户定案）」。 |
| F838 | records/2026-09-16-subagent拆分提案.md:847 | 不相干 | 这一行提到的是 `research/results` 下的产物路径，不是说 `research/` 不进 git：「lts/` 底下的纯文件名 ⇒ `results/research/results/…` ｜ `replay.sh`」 |
| F838 | records/2026-09-16-subagent拆分提案.md:941 | 事件句不改 | subagent 拆分提案记录里 59 号门禁的事：「这张表不进 git（`.gitignore`）」是当时的设计，之后 `memory-peaks.tsv` 已挪出仓（见 2026-09-30 记录）。 |
| F838 | records/2026-09-16-subagent拆分提案.md:965 | 事件句不改 | 拆分提案记录里的一次故障经过：「`.claude/singlefs-ai-sop/` 在 `.gitignore` 第 6 行…」是当时的行号，不是现状话。 |
| F838 | records/2026-09-16-subagent拆分提案.md:1222 | 不相干 | 这一行提到的是 `research/results` 下的产物路径，不是说 `research/` 不进 git：「-segment-four-1791467406/research/e178-segment-four-peer-i」 |
| F838 | records/2026-09-24-里程碑二收尾调度.md:312 | 事件句不改 | 里程碑二收尾调度记录（批次快照）：「规范副本被 `.gitignore` 第 6 行忽略」是 2026-09-25 一次双机真跑的故障经过。 |
| F838 | records/2026-09-28-agent体系用量时间与调度分析.md:52 | 事件句不改 | agent 体系用量记录：「副本目录整个被 `.gitignore` 挡着，这份文件不会进提交」是 2026-09-29 同步 0.0.62 时的观察。 |
| F838 | records/2026-09-28-里程碑二收尾接手.md:42 | 事件句不改 | 里程碑二收尾接手记录：「2026-09-28：仓根 git 忽略的配置与模板改名…`.gitignore` 改忽略新名」是那天的操作。 |
| F838 | records/2026-09-30-里程碑三开工第一轮.md:68 | 事件句不改 | 里程碑三开工第一轮记录：「`.claude/singlefs-ai-sop/` 在 `.gitignore` 里，worktree 没有它」是合入验证时遇到的毛病。 |
| F838 | records/2026-09-30-里程碑三开工第一轮.md:775 | 事件句不改 | 里程碑三开工第一轮记录：「进下一批工具活：sync 的排除表认…（或读 `.gitignore`）」是当时的待办。 |
| F838 | records/2026-09-30-里程碑三开工第一轮.md:1072 | 事件句不改 | 里程碑三开工第一轮记录：「`sync` 仍拷 `multi-host.env` 与 `.gitignore` 里别的大文件，暂认」是当时对工具批交回的判。 |
| F838 | records/2026-09-30-里程碑三开工第一轮.md:1076 | 不相干 | 这一行提到的是 `research/results` 下的产物路径，不是说 `research/` 不进 git：「去之后 `include_str!("../../research/results/…")` 那 4 个装置编不过。」 |
| F838 | records/2026-09-30-里程碑三开工第一轮.md:1080 | 事件句不改 | 里程碑三开工第一轮记录：「`memory-peaks.tsv` 挪出仓…`.gitignore` 那一行删了」是那一批的改动。 |
| F838 | records/2026-10-01-门禁注释精简.md:103 | 事件句不改 | 门禁注释精简记录：「`.gitignore` 一行…由主 agent 逐字节拷进仓」是那一批补 QEMU 补丁时的事。 |
| F838 | records/2026-10-11-scripts审计-第二轮.md:28 | 不相干 | 这一行提到的是 `research/results` 下的产物路径，不是说 `research/` 不进 git：「录写死 `$HOME/code/singlefs/research/results/corruption` ｜ 加用」 |
| F838 | records/2026-10-11-崩溃放量两段与脚本审计.md:62 | 事件句不改 | 崩溃放量两段与脚本审计记录：「worktree 没拷被 `.gitignore` 忽略的 `.claude/singlefs-ai-sop/` 副本」是 2026-10-11 遇到的故障。 |
| F838 | records/2026-10-13-复用键与全仓改名.md:50 | 事件句不改 | 复用键与全仓改名记录：「副本在 `.gitignore` 第 8 行，git 不跟踪」是 2026-10-13 同步副本时的观察，现查 `.gitignore` 第 8 行仍是该路径。 |
| F838 | records/2026-10-13-复用键与全仓改名.md:828 | 事件句不改 | 复用键与全仓改名记录：「规范副本 `.claude/singlefs-ai-sop/` 被 `.gitignore` 排除，照门禁出路那三行建的 worktree 里没有它」是一次验证中的故障经过。 |
| F838 | records/2026-10-13-欠账还五条.md:392 | 事件句不改 | 欠账还五条记录：「E168:48 在被 `.gitignore` 忽略的 segment1 产物（115 MB）里」是当时核对 kb 引用时的一条。 |
| F840 | .claude/kb/milestone/03-third-txn.md:196 | 事件句不改 | 里程碑三收尾项的做法列：「超阈值的检出 2026-10-12 做了：看门狗报告里列出草稿根超阈值…的编译目录与仓副本，只报不删」，说的是 2026-10-12 做成的事，与新事实一致。 |
| F840 | .claude/main-agent.md:58 | 不相干 | 这一句已是新说法：「同一件事连报几次（阈值在 `scripts/agents/watch.conf`）主 agent 都没处置、也没 `--ack` 的，看门狗可以终止那个进程」，与 watch.conf 现有的 terminate-after-unacknowledged-reports=3 一致。 |
| F840 | .claude/main-agent.md:60 | 不相干 | 这一段已列全新增的告警：「交回之后后台还在跑（被停的同样算）」「进程常驻内存过线（默认 8 GiB）」「进程过长（默认 120 分钟）」，也写了「重启前旧进程留下的子 agent 记录标「状态=已失效」」与只报不删的编译目录与仓副本。 |
| F840 | records/2026-09-16-subagent拆分提案.md:913 | 事件句不改 | 拆分提案记录里一次改动的叙述：「阈值与 `watch.conf` 没动」「在等上游」，说的是那一次改 agent-watch.py 时的事。 |
| F840 | records/2026-09-16-subagent拆分提案.md:914 | 事件句不改 | 拆分提案记录里那一次加检出的叙述：「加一类检出「交回之后后台还在跑」…`watch.conf` 的 `leftover-background-grace-minutes`，默认 2 分钟」，说的是那一次做成的事，该检出现在仍在。 |
| F840 | records/2026-09-16-subagent拆分提案.md:919 | 事件句不改 | 拆分提案记录的那一次改动：「`scripts/agents/watch.conf` 加 `ask-every-minutes=60`」，之后 watch.conf 改成 ask-every-minutes=0，记录说的是当时。 |
| F840 | records/2026-09-16-subagent拆分提案.md:939 | 事件句不改 | 拆分提案记录里加进程常驻内存检出的叙述：「默认 8…写进 `scripts/agents/watch.conf` 头的可写 key」，说的是那一次加的事。 |
| F840 | records/2026-09-16-subagent拆分提案.md:1011 | 事件句不改 | 拆分提案记录里的改动表一行：「`scripts/agents/watch.conf` 设 `context-tokens=0`」，是那一次的处置。 |
| F840 | records/2026-09-27-同步远端冲突块.md:1159 | 事件句不改 | 同步远端冲突块记录里抄的当时那一段主 agent 文字，只写「告警有哪几种、各自阈值与什么时候报，见 `scripts/agents/agent-watch.py` 文件头与 `scripts/agents/watch.conf`」，是指路，且是冲突块快照。 |
| F840 | records/2026-09-27-同步远端冲突块.md:1165 | 事件句不改 | 与上一行同段（同步远端冲突块记录的另一个冲突块）：「见 `scripts/agents/agent-watch.py` 文件头与 `scripts/agents/watch.conf`」，是指路，且是冲突块快照。 |
| F840 | records/2026-09-28-agent体系用量时间与调度分析.md:81 | 事件句不改 | agent 体系用量记录的措施跟踪表：「`scripts/agents/watch.conf` 加 `context-tokens=400000` ｜ 已做」是 2026-09-28 当时的完成情况，批次快照。 |
| F840 | records/2026-09-28-agent体系用量时间与调度分析.md:229 | 事件句不改 | agent 体系用量记录的措施跟踪表：「`watch.conf`：`ask-every-minutes=0`，`context-tokens=400000` ｜ 已做」是当时的完成情况，批次快照。 |
| F840 | records/2026-09-28-agent体系用量时间与调度分析.md:249 | 事件句不改 | agent 体系用量记录的措施跟踪表：「子 agent 预算 40 万（共用约束与 `watch.conf`） ｜ 已做」是当时的完成情况，批次快照。 |
| F840 | records/2026-09-28-agent体系用量时间与调度分析.md:325 | 事件句不改 | agent 体系用量记录的措施跟踪表：「3.7 的 `watch.conf` 与主 agent 文字」「已做」是当时的完成情况，批次快照。 |
| F840 | records/2026-10-12-里程碑三非代码收尾.md:54 | 事件句不改 | 里程碑三非代码收尾记录（批次快照）：「批 2 工具活甲…改 `scripts/agents/agent-watch.py`、`watch.sh`、`watch.conf`」是 2026-10-12 那一批的交回，与新事实一致。 |
| F840 | records/2026-10-13-复用键与全仓改名.md:41 | 事件句不改 | 复用键与全仓改名记录（批次快照）：「`scripts/agents/agent-watch.py` 文件头第 19 行「只看、只报，不停任何子 agent、不杀任何进程」」是 2026-10-13 用户提问当天现查的状态，批 3 随后改了。 |
| F840 | records/2026-10-13-复用键与全仓改名.md:54 | 事件句不改 | 复用键与全仓改名记录（批次快照）：「看门狗：`watch.conf` 新键 `terminate-after-unacknowledged-reports=3`」是 2026-10-13 那一批做成的事。 |
| F841 | .claude/kb/checks-owed.md:60 | 事件句不改 | 说的是 2026-09-09 那次立账与收严的经过：「这是…「新立一条判据，当场拿它回扫已有的条目」的形状」「收严之后当场回扫过一次」；规则小节后来改了名，引文是当时的小节名。 |
| F841 | .claude/kb/checks-owed.md:135 | 事件句不改 | 2026-09-06 与 2026-09-09 的追记：「这正是上游 SOP 2026-09-06 新增的那条要拦的形态」，引的是当时那条规则的原话「同样要当场回扫谁在引它」，说的是那一次回扫漏了什么。 |
| F841 | .claude/kb/checks-owed.md:146 | 要改 | 改为：「补完后把 D12（目标介质） 已定项 2 三个桶的全部成员记进回扫清单，随下一次阶段同步一批扫（`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「新立一条判据，记下要回扫的条目，随下一次阶段同步一批扫」）。」原句「补完当场按…回扫」引的小节名已改，做法也已改成记下清单、按批扫。 |
| F841 | .claude/kb/checks-owed.md:158 | 事件句不改 | 说的是 2026-09-07 这一条为什么算漏网：「这一条是同一族里漏网的那个」，引的是当时的小节名，后来改了名。 |
| F841 | .claude/kb/checks-owed.md:178 | 要改 | 改为：「二选一并写下依据：① 给「`P > devs`」补一条新理由（若有），或 ② 判它随理由一起作废、把正文里那半句删掉，并把引它的对象记进回扫清单、随下一次阶段同步一批扫（至少 E115 的 `devs` 那一项）。」这一格是还没做的做法，原句「并当场回扫谁在引它」与现行规则（记下要回扫的条目，随下一次阶段同步一批扫）不一致。 |
| F841 | .claude/kb/checks-owed.md:180 | 要改 | 改为：「写下判据并把要回扫的对象记进回扫清单、随下一次阶段同步一批扫（`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「新立一条判据，记下要回扫的条目，随下一次阶段同步一批扫」）：至少要判 `physical_block_size`…」原句「写下判据并当场回扫」是还没做的做法，引的小节名也已改。 |
| F841 | .claude/kb/checks-owed.md:202 | 事件句不改 | 说的是 2026-09-10 当天的事：「给阶段 22 加「指名而不存在」判据时当场回扫得到」，引的是当时的小节名。 |
| F841 | .claude/kb/decisions/02-RAID条带策略.md:256 | 事件句不改 | 决策正文里交代那次回扫的经过：「回扫打出的消费方远不止两个」，说的是当时做的事，引的是当时的小节名。 |
| F841 | records/2026-09-05-SOP剥离轮.md:29 | 事件句不改 | SOP 剥离轮记录（批次快照）：「新立判据当场回扫已有条目」是那一轮对 SOP 规则条目的盘点。 |
| F841 | records/2026-09-06-D1回扫重判.md:6 | 事件句不改 | D1 回扫重判记录（2026-09-06）：「新立一条判据，当场拿它回扫已有的条目」的下半场，是当时对规则的引用与那一轮的经过。 |
| F841 | records/2026-09-06-树ID水位臂比较重做.md:68 | 事件句不改 | 树 ID 水位臂比较记录（2026-09-06）：「立完要当场回扫 C143」是当时的做法与经过。 |
| F841 | records/2026-09-10-D6项2与E130.md:85 | 事件句不改 | D6 项 2 与 E130 记录（2026-09-10）：「`evidence-discipline.md`「新立一条判据，当场拿它回扫已有的条目」要防的」是当时的引用。 |
| F841 | records/2026-09-11-D19项6对抗第三轮.md:35 | 事件句不改 | D19 项 6 对抗第三轮记录（2026-09-11）：「与「新立一条判据，当场拿它回扫已有的条目」是同一个漏」是那一轮的自查。 |
| F841 | records/2026-09-28-agent体系用量时间与调度分析.md:263 | 事件句不改 | agent 体系用量记录（批次快照）：「「新立判据当场回扫」「撤回当场回扫」改成按批」是那次分析提出的改法，之后已改进 evidence-discipline.md。 |
| F842 | .claude/agents/three-way-attack.md:48 | 不相干 | 这一句已是新说法：「`<模型目录>/tools/rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷，只改 `rerun_steps` 那一段；仓根与冻结副本用 `--repo`、…」。 |
| F842 | .claude/agents/three-way-defense.md:36 | 不相干 | 这一句已是新说法：「`<模型目录>/tools/rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷，只改 `rerun_steps` 那一段」。 |
| F842 | .claude/agents/three-way-forward.md:37 | 不相干 | 这一句已是新说法：「`<模型目录>/tools/rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷，只改 `rerun_steps` 那一段」。 |
| F842 | .claude/kb/checks-owed.md:454 | 不相干 | 这一行命令里是已经存在的一份复跑脚本：「`research/prompts/m3-formal-code-b25-r1-opus-model/tools/rerun.sh`」，说的是怎么复跑那一轮，不是三方腿怎么写 rerun.sh。 |
| F842 | .claude/kb/experiments/171-三oracle合谋测试.md:231 | 不相干 | E171 页里的一句复跑叙述：「主 agent 复跑（`tools/rerun.sh … all`）这 7 行逐字同」，说的是那一次复跑，不涉及 rerun.sh 怎么写。 |
| F842 | .claude/kb/milestone/03-third-txn.md:190 | 要改 | 把检查列的「没拦：仓里没有复跑脚本模板；」改为「没拦（2026-10-11 之前仓里没有复跑脚本模板，现有 `scripts/agents/three-way-rerun-template.sh`）；」，其余照旧。原句「没拦：仓里没有复跑脚本模板」在同一格后面的「2026-10-11 下游：模板 `scripts/agents/three-way-rerun-template.sh`」已落地之后不再是真话。 |
| F842 | records/2026-09-16-subagent拆分提案.md:1076 | 事件句不改 | 拆分提案记录里的一次现象叙述：「三方攻方腿写的复跑脚本（`research/prompts/e171-conclusion-r1-opus-model/tools/rerun.sh` 第 7 步）…」，是当时某一份脚本的毛病。 |
| F842 | records/2026-09-30-里程碑三开工第一轮.md:699 | 事件句不改 | 里程碑三开工第一轮记录（批次快照）：「装置 `research/prompts/m3-formal-code-b34-r1-opus-model/`、`tools/rerun.sh build` 起」是当时某一轮攻方腿的复跑入口。 |
| F842 | records/2026-10-01-里程碑三形式化接入.md:619 | 事件句不改 | 里程碑三形式化接入记录（批次快照）：「攻方 `tools/rerun.sh` 在会话草稿 `r1-rerun/` 复跑」是那一轮主 agent 复核的经过。 |
| F842 | records/2026-10-01-里程碑三形式化接入.md:623 | 事件句不改 | 里程碑三形式化接入记录（批次快照）：「攻方 `tools/rerun.sh` 在会话草稿 `r2-rerun/` 从头跑，14 份输出逐个 `cmp` 无差」是那一轮的复核。 |
| F842 | records/2026-10-01-里程碑三形式化接入.md:713 | 事件句不改 | 里程碑三形式化接入记录（批次快照）：「攻方 `tools/rerun.sh` 在会话草稿 `c673-r1-rerun/` 复跑 `RERUN_RC=0`」是那一轮的复核。 |
| F842 | records/2026-10-01-里程碑三形式化接入.md:739 | 事件句不改 | 里程碑三形式化接入记录（批次快照）：「攻方 `tools/rerun.sh` 在会话草稿 `b16-rerun/` 复跑坐实」是那一轮的复核。 |
| F842 | records/2026-10-01-里程碑三形式化接入.md:776 | 事件句不改 | 里程碑三形式化接入记录（批次快照）：「主 agent 在会话草稿 `b17-rerun/` 复跑它的 `tools/rerun.sh`」是那一轮的复核。 |
| F842 | records/2026-10-01-里程碑三形式化接入.md:818 | 事件句不改 | 里程碑三形式化接入记录（批次快照）：「攻方 `tools/rerun.sh` 在会话草稿 `b19-opus-rerun/` 复跑中」是那一轮的复核。 |
| F842 | records/2026-10-01-里程碑三形式化接入.md:839 | 事件句不改 | 里程碑三形式化接入记录（批次快照）：「攻方 `tools/rerun.sh` 在会话草稿 `b20-opus-rerun/` 复跑」是那一轮的复核。 |
| F842 | records/2026-10-01-里程碑三形式化接入.md:944 | 事件句不改 | 里程碑三形式化接入记录（批次快照）：「攻方 `tools/rerun.sh all` 在会话草稿 `b24-opus-rerun/` 后台复跑中」是那一轮的复核。 |
| F842 | records/2026-10-01-里程碑三形式化接入.md:961 | 事件句不改 | 里程碑三形式化接入记录（批次快照）：「攻方 `tools/rerun.sh` 全件在会话草稿 `c655-opus-rerun/` 后台复跑中」是那一轮的复核。 |
| F842 | records/2026-10-01-里程碑三形式化接入.md:1120 | 事件句不改 | 里程碑三形式化接入记录（批次快照）：「主 agent 在草稿目录照 `tools/rerun.sh` 五段重跑」是那一轮的复核。 |
| F842 | records/2026-10-11-scripts审计-第二轮.md:40 | 事件句不改 | scripts 审计第二轮记录（批次快照）：审计发现表里「`scripts/agents/three-way-rerun-template.sh:76`」处 `rsync -a` 的毛病，是当时的行号与发现。 |
| F842 | records/2026-10-11-scripts审计.md:29 | 事件句不改 | scripts 审计记录（批次快照）：「只有 `hosts/capped.sh` 与 `agents/three-way-rerun-template.sh` 开 `set -e`」是当时审计的发现。 |
| F842 | records/2026-10-11-里程碑三待优化上下游协调.md:64 | 事件句不改 | 里程碑三待优化上下游协调记录（批次快照）：「新建 `scripts/agents/three-way-rerun-template.sh`、改 `scripts/agents/dispatch-prompt.py`」是 2026-10-11 那一批的交回。 |
| F842 | records/2026-10-11-里程碑三待优化上下游协调.md:72 | 事件句不改 | 里程碑三待优化上下游协调记录（批次快照）：「在 `.claude/gate.d/code-tooling.sh` 的 `SELFTEST_RUNNERS` 加 `bash scripts/agents/three-way-rerun-template.sh --selftest`」是那一批的做法。 |
| F842 | records/2026-10-12-里程碑三工具与治理批.md:117 | 事件句不改 | 里程碑三工具与治理批记录（批次快照）：「经 `run-with-memory-cap.sh 4G` 跑 `tools/rerun.sh` 八行全 same」是那一轮的复核。 |
| F843 | .claude/agents/experiment-runner.md:32 | 不相干 | 这一句已是新说法：「同一会话里几条包装命令照常排队，不用设并行理由 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON`」，与 `scripts/hosts/run-with-memory-cap.sh` 第 120、121 行「日常一条（2026-10-06 撤了…）」一致。 |
| F843 | .claude/kb/experiments/169-并行提交的追加纪律与前缀恢复.md:19 | 事件句不改 | E169 页历史叙述某一次运行：「第二段第一次运行没有产出：内存包装退出码 252（同一会话下已有别的执行员的一条在跑，没给并行理由）」，说的是当时的包装规则下发生的事。 |
| F843 | records/2026-09-16-subagent拆分提案.md:1082 | 事件句不改 | 拆分提案记录（批次快照）：「主 agent 设 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON` 带 `--skip-apply` 重跑测试」是当时一次合入验证的经过。 |
| F843 | records/2026-09-16-subagent拆分提案.md:1133 | 事件句不改 | 拆分提案记录（批次快照）：「每条都带 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON` 过了内存包装」是当时一次并行跑变异的经过。 |
| F843 | records/2026-09-30-里程碑三开工第一轮.md:234 | 事件句不改 | 里程碑三开工第一轮记录（批次快照）：「实现员设了内存包装的并行理由与超峰值理由…追认」是当时的处置。 |
| F843 | records/2026-09-30-里程碑三开工第一轮.md:236 | 事件句不改 | 里程碑三开工第一轮记录（批次快照）：命令里的 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON=<理由>` 是当时交给下一步的重跑命令。 |
| F843 | records/2026-09-30-里程碑三开工第一轮.md:659 | 事件句不改 | 里程碑三开工第一轮记录（批次快照）：「fmt、clippy 两步被内存包装拒…主 agent 设并行理由补跑」是当时的经过。 |
| F843 | records/2026-10-01-删除与并发事务.md:150 | 事件句不改 | 删除与并发事务记录里的收拢表一行（批次快照）：「要 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON` ｜ 开着 ｜ 下轮开（治理批）」是 2026-10-01 当天的收拢状态，之后 2026-10-06 撤了这条规则。 |
| F843 | records/2026-10-01-删除与并发事务.md:517 | 事件句不改 | 删除与并发事务记录（批次快照）：「它给内存包装设了 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON`，照包装文件头的出路设的，收下」是当时的处置。 |
| F843 | records/2026-10-01-删除与并发事务.md:793 | 事件句不改 | 删除与并发事务记录（批次快照）：「实现员证红时因内存包装退 252 自设了 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON`…主 agent 认」是当时的处置。 |
| F843 | records/2026-10-01-删除与并发事务.md:806 | 事件句不改 | 删除与并发事务记录（批次快照）：「`--skip-apply` 带并行理由重跑」是当时一次合入验证的经过。 |
| F843 | records/2026-10-01-对接上游测试脚本规范.md:504 | 事件句不改 | 对接上游测试脚本规范记录（批次快照）：「`mutate.sh` 只在自证第 141 行设 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON`」是当时一次回归的现查。 |
| F843 | records/2026-10-01-对接上游测试脚本规范.md:509 | 事件句不改 | 对接上游测试脚本规范记录（批次快照）：「`mutate.sh` 并行理由那一件交回」是那一批工具活的交回。 |
| F843 | records/2026-10-01-对接上游测试脚本规范.md:583 | 事件句不改 | 对接上游测试脚本规范记录（批次快照）：「`crates-mutation-rows.py` 的 `run_under_wrapper` 多工作进程不带并行理由、撞「日常一条」」是当时发现的毛病。 |
| F843 | records/2026-10-01-对接上游测试脚本规范.md:592 | 事件句不改 | 对接上游测试脚本规范记录（批次快照）：「`crates-mutation-rows.py` 每次起包装带工作进程自己的并行理由」是当时那一批做成的事。 |
| F843 | records/2026-10-01-对接上游测试脚本规范.md:603 | 事件句不改 | 对接上游测试脚本规范记录（批次快照）：「`mutation-shard-run.sh` 第二台外层包装的并行理由」是当时留给下一件的待办。 |
| F843 | records/2026-10-01-对接上游测试脚本规范.md:607 | 事件句不改 | 对接上游测试脚本规范记录（批次快照）：「mutation-shard-run 外层包装并行理由」是那一批 shell 工具活的交回项。 |
| F843 | records/2026-10-01-里程碑三形式化接入.md:144 | 事件句不改 | 里程碑三形式化接入记录（批次快照）：「第一次起被包装拒（同会话已有 3-1 的一条在跑），带并行理由重起」是当时的经过。 |
| F843 | records/2026-10-01-里程碑三形式化接入.md:444 | 事件句不改 | 里程碑三形式化接入记录（批次快照）：「带并行理由 `--skip-apply` 重跑」是当时一次合入验证的经过。 |
| F843 | records/2026-10-01-里程碑三形式化接入.md:739 | 事件句不改 | 里程碑三形式化接入记录（批次快照）：「第一次被内存包装同会话只许一条拒、带并行理由重跑」是当时一次复跑的经过。 |
| F843 | records/2026-10-01-里程碑三形式化接入.md:777 | 事件句不改 | 里程碑三形式化接入记录（批次快照）：「fmt 第一次被「同会话只许一条」拒（252）、带并行理由重跑退 0」是当时的经过。 |
| F843 | records/2026-10-01-里程碑三形式化接入.md:890 | 事件句不改 | 里程碑三形式化接入记录（批次快照）：命令里的 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON=…` 是当时重起 e170 导出的做法。 |
| F843 | records/2026-10-01-里程碑三形式化接入.md:1074 | 事件句不改 | 里程碑三形式化接入记录（批次快照）：「它自己设了 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON`…以后的派发提示写明这一句」是当时的处置，之后该规则已撤。 |
| F843 | records/2026-10-01-里程碑三形式化接入.md:1165 | 事件句不改 | 里程碑三形式化接入记录（批次快照）：「它自设 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON`（包装给的出路），认」是当时的处置。 |
| F844 | .claude/kb/milestone/03-third-txn.md:191 | 不相干 | 说的是内存上限的默认值，不是线程分配：「没有历史峰值时，上限默认取 `scripts/hosts/host-resources.py` 的预测值」「2026-10-11 下游已改」，host-resources.py 的预测值仍用于峰值表缺项，与线程取登记值无关。 |
| F844 | records/2026-09-16-subagent拆分提案.md:1154 | 事件句不改 | 拆分提案记录（批次快照）：「新建 `scripts/hosts/host-resources.py`（逐台核数、占用、可用内存…）」是那一批交回的新脚本，说的是当时做成的事。 |
| F844 | records/2026-09-16-subagent拆分提案.md:1162 | 事件句不改 | 拆分提案记录（批次快照）：「出路写「改成按…`--emit-hosts` 与 `host-resources.py --json` 现算分配…」」是当时对判据落成字面的提议，后来线程改取登记值。 |
| F844 | records/2026-09-16-subagent拆分提案.md:1166 | 事件句不改 | 拆分提案记录（批次快照）：「`scripts/hosts/host-resources.py` 带命令时，这条命令的历史用时超过 2 小时而只派到一台，报一行提醒」是当时第 5 条的描述。 |
| F844 | records/2026-09-16-subagent拆分提案.md:1168 | 事件句不改 | 拆分提案记录（批次快照）：脚本文件头 `# long-run-hosts:`「要点名 `--emit-hosts` 与 `host-resources.py`」是当时的判法，之后 2026-10-13 改成逐字等于规范句。 |
| F844 | records/2026-09-30-里程碑三开工第一轮.md:754 | 事件句不改 | 里程碑三开工第一轮记录（批次快照）：「`host-resources.py` 提醒、`host-dispatch.py` 拒「历史 > 2 小时只派一台」」是当时工具批的五条。 |
| F844 | records/2026-09-30-里程碑三开工第一轮.md:758 | 事件句不改 | 里程碑三开工第一轮记录（批次快照）：「改成按 `host-resources.py` 现算（清单同目录 `quota-kept.tsv`…）」是当时的待办与做法。 |
| F844 | records/2026-09-30-里程碑三开工第一轮.md:1060 | 事件句不改 | 里程碑三开工第一轮记录（批次快照）：「另查出 `host-resources.py` 只采 1 秒 `/proc/stat`」是当时的现查。 |
| F844 | records/2026-09-30-里程碑三开工第一轮.md:1066 | 事件句不改 | 里程碑三开工第一轮记录（批次快照）：「`host-dispatch.py` 的 `long_run_check` 一份判法、`host-resources.py` 调它」是 t1 工具批的交回。 |
| F844 | records/2026-09-30-里程碑三开工第一轮.md:1068 | 事件句不改 | 里程碑三开工第一轮记录（批次快照）：「三处 `capped.sh 6/8` 改按 `host-resources.py --json` 现算」是当时的做法，之后 2026-10-13 改成取登记值。 |
| F844 | records/2026-09-30-里程碑三开工第一轮.md:1069 | 事件句不改 | 里程碑三开工第一轮记录（批次快照）：「第二台跑测试的线程改按 `host-resources.py` 现算」是当时的做法，之后 2026-10-13 改成取登记值。 |
| F844 | records/2026-10-01-删除与并发事务.md:1309 | 事件句不改 | 删除与并发事务记录（批次快照）：「合入验证照 `host-resources.py` 现查的余量给」是当时的安排。 |
| F844 | records/2026-10-01-里程碑三形式化接入.md:1281 | 事件句不改 | 里程碑三形式化接入记录（批次快照）：「每次续跑时经 `host-resources.py --json` 现查、按单行线道…算各台线道数」是当时派发的约定。 |
| F844 | records/2026-10-01-里程碑三形式化接入.md:1283 | 事件句不改 | 里程碑三形式化接入记录（批次快照）：「它要改 `host-resources.py`、`host-dispatch.py`、`multi-host-run.sh`、`merge-and-verify.sh`，本线不碰」是当时的分工。 |
| F844 | records/2026-10-01-里程碑三形式化接入.md:1292 | 事件句不改 | 里程碑三形式化接入记录（批次快照）：「三处 `capped.sh 6/8` 改按 `host-resources.py` 现算」是当时的改动，之后改取登记值。 |
| F844 | records/2026-10-01-里程碑三形式化接入.md:1294 | 事件句不改 | 里程碑三形式化接入记录（批次快照）：「E171 跑完、两台全空（`host-resources.py`：本机 32 线程 55.5 GiB…）」是当时的现查读数。 |
| F844 | records/2026-10-12-里程碑三工具与治理批.md:109 | 事件句不改 | 里程碑三工具与治理批记录（批次快照）：「experiment-runner.md 第 32 行、experiment-designer.md 第 34 行线程按 `host-resources.py` 现算，要改登记值」是当时的待办，现行定义已改取登记值（`grep host-resources.py .claude/agents` 零命中）。 |
| F844 | records/2026-10-12-里程碑三工具与治理批.md:114 | 事件句不改 | 里程碑三工具与治理批记录（批次快照）：「两份实验定义多机线程取登记值，…（`lib_long_run.judge` 判脚本头仍要点名 `host-resources.py`）」是当时的做法，之后 2026-10-13 判脚本头改为规范句。 |
| F844 | records/2026-10-12-里程碑三工具与治理批.md:125 | 事件句不改 | 里程碑三工具与治理批记录（批次快照）：「`lib_long_run.py` 文件头判法、`detached-run.sh long-run-check`、`host-dispatch.py` ⓪ 仍要点名 host-resources.py」是当时的证红叙述。 |
| F844 | records/2026-10-13-复用键与全仓改名.md:49 | 事件句不改 | 复用键与全仓改名记录（批次快照）：「驱动照 `crash-amplification-lib.sh` 文件头「现算线程」读 `scripts/hosts/host-resources.py --json`」是 2026-10-13 排查一次真跑的经过。 |
| F844 | records/2026-10-13-复用键与全仓改名.md:372 | 事件句不改 | 复用键与全仓改名记录（批次快照）：「8 格从「按 host-resources.py 现算」改成「取登记值」」是那一批做成的事，旧说法在这里是被改掉的对象。 |
| F844 | records/2026-10-13-复用键与全仓改名.md:378 | 事件句不改 | 复用键与全仓改名记录（批次快照）：「host-resources.py 自证 long-history-reminder 的期望串改成登记位」是门禁第一趟红的逐处修法。 |
| F845 | .claude/kb/milestone/02-second-txn.md:516 | 不相干 | 说的是放量里一块状态数的设计：「一块正好是 GPU 的一个批、双机分片的一个单位」，不是双机分片怎么跑。 |
| F845 | .claude/kb/term-renames.md:100 | 不相干 | 改名对照表一行：「crash-point-replay-shard-run ｜ 正则 ｜ 崩溃点重放双机分片包装的脚本名」，登的是旧名到新名。 |
| F845 | .claude/kb/term-renames.md:103 | 不相干 | 改名对照表一行：「singlefs-crash-point-replay-shard ｜ 正则 ｜ 双机分片包装的临时目录前缀」，登的是旧名到新名。 |
| F845 | .claude/rules/implementation-workflow.md:66 | 不相干 | 这一条已是新说法：「`scripts/crash/crash-point-replay-shard-run.sh --merged-log`：它经崩溃放量的双机驱动 `scripts/crash/crash-amplification-two-hosts.sh` 跑…第二台判出的块取回、按块键并进本机那一份崩溃判定库（第二台那一趟的树连同库跑完就删，不留长期库）」。 |
| F845 | .claude/rules/mutation-sampling.md:58 | 不相干 | 这一句说选行的口径：「选行（`--item`、`--quick`）与双机分片同样按行」，没有写分片驱动怎么跑。 |
| F845 | records/2026-09-16-subagent拆分提案.md:965 | 事件句不改 | 拆分提案记录（批次快照）：登记表第 54 行「双机分片驱动与它的自检 `scripts/crash/layer0-shard-run-selftest.sh`」是 2026-09-28 当时的脚本名。 |
| F845 | records/2026-09-24-里程碑二收尾调度.md:200 | 事件句不改 | 里程碑二收尾调度记录（批次快照）：「双机分片提前开工、分析员派出（2026-09-26）」是当天的经过。 |
| F845 | records/2026-09-24-里程碑二收尾调度.md:271 | 事件句不改 | 里程碑二收尾调度记录（批次快照）：「配置检查现判「双机分片不能用」，交用户补」是当时本机私有配置缺键时的状态。（本行开头原话：「G3 交回（2026-09-27）   接续的工具实」） |
| F845 | records/2026-09-24-里程碑二收尾调度.md:296 | 事件句不改 | 里程碑二收尾调度记录（批次快照）：「`--full` 从此按双机分片跑 ｜ 等交回」是当时的待办格。（本行开头原话：「kb 第九批起草员与 E142 第十九次执行员派出；」） |
| F845 | records/2026-09-24-里程碑二收尾调度.md:311 | 事件句不改 | 里程碑二收尾调度记录（批次快照）：「主 agent 越权 ssh 两次（2026-09-28）」是当天的违规经过。 |
| F845 | records/2026-09-24-里程碑二收尾调度.md:318 | 事件句不改 | 里程碑二收尾调度记录（批次快照）：「54 快档 + `--full` 双机分片」是当时的跑批安排。（本行开头原话：「分片续跑验过、全部暂存、第五步开跑（2026-09-」） |
| F845 | records/2026-09-24-里程碑二收尾调度.md:321 | 事件句不改 | 里程碑二收尾调度记录（批次快照）：「崩溃验证员（54 快档 + `--full` 双机分片、55、57、59）」是当时第五步的安排。 |
| F845 | records/2026-09-27-代码审阅38条去向.md:13 | 事件句不改 | 代码审阅 38 条去向记录（批次快照）：「实分一（崩溃点重放双机分片）在改 `crates/singlefs-checker-tier/src/crash.rs`」是当时的分批。 |
| F845 | records/2026-09-27-同步远端冲突块.md:1211 | 事件句不改 | 同步远端冲突块记录（冲突块快照）：「双机分片的驱动脚本 `scripts/crash/layer0-shard-run.sh` 带什么参数都算」抄的是当时规则的旧文，驱动脚本已改名。 |
| F845 | records/2026-09-28-agent体系用量时间与调度分析.md:204 | 事件句不改 | agent 体系用量记录（批次快照）：指到「`records/2026-09-28-门禁59号提速与双机分片.md` Q15」，是记录标题里的名字。 |
| F845 | records/2026-09-28-agent体系用量时间与调度分析.md:297 | 事件句不改 | agent 体系用量记录（批次快照）：指到「`records/2026-09-28-门禁59号提速与双机分片.md` Q9」，是记录标题里的名字。 |
| F845 | records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md:96 | 事件句不改 | 里程碑三崩溃放量调度记录（批次快照）：「`checker-tier-crash-point-replay.sh` 第 481–487 行本地配置判得过就自动走双机分片」是当时现查到的状态与要改的点。 |
| F845 | records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md:107 | 事件句不改 | 里程碑三崩溃放量调度记录（批次快照）：「复用标记、KV、GPU 批、双机分片、ignore 都不能让单跑做不到」是那一轮对正文 A7 的要求。 |
| F845 | records/2026-09-28-里程碑二剩余七项的非重型部分.md:20 | 事件句不改 | 里程碑二剩余七项记录（批次快照）：指到「`records/2026-09-28-门禁59号提速与双机分片.md` 第 39 行」，是记录标题里的名字。 |
| F845 | records/2026-09-28-门禁59号提速与双机分片.md:1 | 事件句不改 | 这是记录自己的标题「门禁 59 号提速与双机分片：调度记录」，批次记录，说的是 2026-09-28 那一轮。 |
| F845 | records/2026-09-28-门禁59号提速与双机分片.md:7 | 事件句不改 | 这是记录开头的课题：「这一轮修判错、补按条复用与续跑、接双机分片」，说的是 2026-09-28 那一轮要做的事。 |
| F845 | records/2026-09-28-门禁59号提速与双机分片.md:31 | 事件句不改 | 门禁 59 号提速与双机分片记录（批次快照）：「规则里「双机分片」那一条」是那一轮分工表里的一格。 |
| F845 | records/2026-09-28-门禁59号提速与双机分片.md:127 | 事件句不改 | 门禁 59 号提速与双机分片记录（批次快照）：「不做：…双机分片、按输入复用与全绿标记…重设计」是那一轮当时的处置。 |
| F845 | records/2026-09-30-里程碑三开工第一轮.md:464 | 事件句不改 | 里程碑三开工第一轮记录（批次快照）：指到「`records/2026-09-28-门禁59号提速与双机分片.md` 第 12 行 Q2」，是记录名。 |
| F845 | records/2026-09-30-里程碑三开工第一轮.md:991 | 事件句不改 | 里程碑三开工第一轮记录（批次快照）：「给定状态引擎缺双机分片（H5）…交 E4」是当时那一轮的判决项。 |
| F845 | records/2026-10-01-对接上游测试脚本规范.md:10 | 事件句不改 | 对接上游测试脚本规范记录（批次快照）：「只留本仓的阈值、命令、登记表…双机分片驱动」是当时规则去重时留下的项。 |
| F845 | records/2026-10-01-对接上游测试脚本规范.md:47 | 事件句不改 | 对接上游测试脚本规范记录（批次快照）：「本仓留 `std::thread::scope`、不能并行写明为什么、双机分片」是当时规则合拍表的一格。 |
| F845 | records/2026-10-01-对接上游测试脚本规范.md:370 | 事件句不改 | 对接上游测试脚本规范记录（批次快照）：「**checker 档**是…断点续跑与双机分片、崩溃注入与坏盘输入两场战役」抄的是 2026-10-01 当时的定案原文。 |
| F845 | records/2026-10-01-对接上游测试脚本规范.md:374 | 事件句不改 | 对接上游测试脚本规范记录（批次快照）：「断点续跑与双机分片、崩溃注入与坏盘输入两场战役」抄的是 2026-10-01 当时的定案原文。 |
| F845 | records/2026-10-11-崩溃放量两段与脚本审计.md:85 | 事件句不改 | 崩溃放量两段与脚本审计记录（批次快照）：「这一趟双机分片实际是单机跑，而双机配置写着 ENABLE_ACROSS_MACHINES=1」是 2026-10-11 一次变异趟的观察。 |
| F845 | records/2026-10-13-复用键与全仓改名.md:48 | 事件句不改 | 复用键与全仓改名记录（批次快照）：「双机分片包装的自证（已改名 `scripts/crash/crash-point-replay-shard-run.sh`，`--selftest`）退 77」是 2026-10-13 排查的经过。 |
| F848 | .claude/kb/decisions/13-验证路线.md:33 | 不相干 | 这一格已是新说法：「取前缀、择根、扫环三截已进 Verus，都在 `crates/singlefs-replay-prefix/src/lib.rs`」。 |
| F848 | .claude/kb/decisions/13-验证路线.md:366 | 不相干 | 这一句已是新说法：「Verus 核与它的 spec（取前缀、择根、扫环三截与择根和取前缀的合判推论，…）」。 |
| F848 | .claude/kb/verification-build.md:236 | 不相干 | 这一句已是新说法：「现状：第 7 步 crash refinement 的取前缀、择根、扫环三截进了 Verus（`crates/singlefs-replay-prefix`）」。 |
| F848 | .claude/rules/verification.md:125 | 不相干 | 这一格已是新说法：「形式化验证：恢复重放取前缀、择根、扫环三截的核与择根取前缀的合判推论经 Verus 验过」。 |
| F848 | .claude/skills/crash-test/SKILL.md:25 | 不相干 | 这一格已是新说法：「恢复重放取前缀、择根、扫环三截的核与择根取前缀的合判推论经 Verus 验过」。 |
| F848 | CLAUDE.md:8 | 要改 | 把「恢复重放取前缀的核 `singlefs-replay-prefix`」改为「恢复重放取前缀、择根、扫环三截的核 `singlefs-replay-prefix`」，其余照旧。现行覆盖见 `.claude/rules/verification.md` 形式化验证一格与 `crates/singlefs-replay-prefix/src/lib.rs`，原写法只点了取前缀一截。 |
| F848 | records/2026-10-13-Verus证明审计.md:3 | 不相干 | 这一句已写全三个核：「检测 `crates/singlefs-replay-prefix/src/lib.rs`（取前缀、择根、扫环三个核与它们的 spec、引理、推论）」，是 2026-10-13 的审计记录，与「只覆盖取前缀」无关。 |
| F849 | .claude/agent-common.md:82 | 不相干 | 这一段就是新说法：「工作根…底下仓副本里的 `--selftest` 只在三条同时满足时放行：…登记为 light 且副本里这一行与主仓逐字相同、命令是单独一个 `--selftest`、副本里那份脚本与主工作区那一份 sha256 相同；改过的脚本的自证不在副本里起…交主 agent」。 |
| F849 | records/2026-09-30-里程碑三开工第一轮.md:407 | 事件句不改 | 里程碑三开工第一轮记录（批次快照）：「第三轮攻方在冻结副本里起 `admission.py --selftest` 被重型闸拒（副本那一份不按登记表放行）」是当时看门狗的检出。 |
| F849 | records/2026-09-30-里程碑三开工第一轮.md:842 | 事件句不改 | 里程碑三开工第一轮记录（批次快照）：「副本里跑不了 `--selftest`（重型闸只认本仓那一份）记进下一批工具欠账」是当时的处置，之后闸已按登记表放行（agent-common.md 现行规则）。 |
| F849 | records/2026-09-30-里程碑三开工第一轮.md:1050 | 事件句不改 | 里程碑三开工第一轮记录（批次快照）：「`--selftest` 第一次退 1，开关 `check-every-card` 红在…」是当时一份清场补丁的复核经过，说的与副本里跑自证无关。 |
| F849 | records/2026-10-01-里程碑三形式化接入.md:887 | 事件句不改 | 里程碑三形式化接入记录（批次快照）：「它在副本里跑驱动 `--selftest` 被拒（路径不是登记表那一份）」是当时看门狗的检出。 |
| F849 | records/2026-10-01-里程碑三形式化接入.md:1298 | 不相干 | 这一条说的是自证新增一格：「`crates-mutation-rows.py --selftest` 加副本缺 `.cargo/config.toml` 的样本格」，写的是变异副本缺配置的样本，不是在副本里跑自证的办法；且是 2026-10-01 记录。 |
| F849 | records/2026-10-11-里程碑三知识腐烂治理.md:126 | 事件句不改 | 知识腐烂治理记录（批次快照）：「乙组在副本里跑 `crates-mutation-rows.py --selftest`…被 heavy-test 闸拒，没执行」是批 9 看门狗的检出。 |
| F849 | records/2026-10-13-复用键与全仓改名.md:56 | 不相干 | 这里的「副本」是同步下来的上游规则副本：「`lib_source_digest.py --selftest` 对着新副本 47 格都对」，与仓副本里怎么起自证无关，且是 2026-10-13 记录。 |
| F831 | crates/singlefs-checker/invariants/02-根环.md:14 | 事件句不改 | 不变量表 I-7.7 一行里交代 C322 定案的验证：「崩溃点重放全量 262165 个状态 I-7.7（系统配置实例代号不低于根环） 违例 0（旧定义…2 个状态判红）」是 2026-09-14 当时改定义前后的对比。 |
| F831 | crates/singlefs-checker/invariants/02-根环.md:15 | 事件句不改 | 不变量表 I-7.8 一行里交代旧字面为什么不取：「按字面数孤儿，崩溃点重放全量 262165 个状态里 261891 个判红」是总审核当时算出来的数，已指到 records。 |
| F832 | crates/singlefs-checker/invariants/02-根环.md:20 | 不相干 | 这一行只写「列进 `IMPLEMENTED_INVARIANTS`」，没有写条数，说的不是 29 条这件事实。（本行开头原话：「I-7.13   系统配置池级字段在读者收的范围里 」） |
| F832 | crates/singlefs-checker/invariants/02-根环.md:21 | 不相干 | 这一行只写「列进 `IMPLEMENTED_INVARIANTS`」，没有写条数。 |
| F832 | crates/singlefs-checker/invariants/05-空间记账.md:11 | 不相干 | 这一行写的是「`IMPLEMENTED_INVARIANTS` 下标 65」（现查第 65 项是 REFERENCE_INTERVALS_ARE_EXACT）与「`IMPLEMENTED_INVARIANTS` 加一条」，没有写总条数。（本行开头原话：「I-3.5   引用区间的精确性   **引用区间的」） |
| F832 | crates/singlefs-checker/invariants/07-分配器.md:10 | 不相干 | 这一行写的是「`IMPLEMENTED_INVARIANTS` 下标 67」（现查第 67 项是 FIRST_DATA_UNIT_OF_THE_VERSION…），没有写总条数。（本行开头原话：「I-5.5   从 mkfs 水位建树那一版的 t1」） |
| F832 | crates/singlefs-checker/invariants/09-inode树结构.md:28 | 不相干 | 这一行只写「列进 `image.rs` 的 `IMPLEMENTED_INVARIANTS`」，没有写条数。（本行开头原话：「I-9.16   树表条目按树 ID 严格升序且合发」） |
| F832 | crates/singlefs-checker/invariants/09-inode树结构.md:39 | 不相干 | 这一行写的是「`IMPLEMENTED_INVARIANTS` 下标 66」（现查第 66 项是 HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES），没有写总条数。（本行开头原话：「I-9.27   树表条目的头 ID 取值   走读」） |
| F832 | crates/singlefs-checker/invariants/09-inode树结构.md:48 | 不相干 | 这一行写的是「`IMPLEMENTED_INVARIANTS` 下标 90」（现查第 90 项是 NON_RIGHTMOST_LINEAGE…），没有写总条数。 |
| F832 | crates/singlefs-checker/invariants/09-inode树结构.md:49 | 不相干 | 这一行写的是「`IMPLEMENTED_INVARIANTS` 下标 91」（现查第 91 项是 LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN…），没有写总条数。 |
| F832 | crates/singlefs-checker/invariants/README.md:29 | 不相干 | 已经是新说法：「池级 checker（`walk::check_pool_image`）判 92 条」，与现数 92 一致。 |
| F832 | crates/singlefs-checker/invariants/README.md:33 | 不相干 | 「用了 92 位」与 `IMPLEMENTED_INVARIANTS` 长度 92 一致，容量常量 `POOL_CHECKER_VIOLATED_BIT_CAPACITY` 不大于 128，数是对的。 |
| F840 | scripts/agents/README.md:7 | 不相干 | 这一行只写看门狗的文件清单：「`watch.sh`、`watch.conf`、`agent-watch.py` ｜ 看门狗：复检子 agent 的会话记录与本实例底下的长进程，阈值在 `watch.conf`」，没有列告警种类。 |
| F842 | scripts/agents/README.md:13 | 不相干 | 这一行已是新说法：「三方腿模型目录 `tools/rerun.sh` 的模板：仓根与冻结副本从参数接，拷到哪都能跑；腿拷过去只改 `rerun_steps`」。 |
| F844 | scripts/hosts/README.md:10 | 不相干 | 这一行只写脚本分工：「`host-resources.py` ｜ …量各台此刻能用多少」，没有写线程怎么取。 |
| F845 | .claude/skills/crash-test/commit-stages.md:21 | 要改 | 把「本机那一片与 merge 由第 1b 步的包装管」改为「本机分份那一趟与带导入再跑的那一趟由第 1b 步的包装管」，其余照旧。现在的驱动 `scripts/crash/crash-point-replay-shard-run.sh` 文件头写的是「本机再带 `SINGLEFS_CRASH_AMPLIFICATION_IMPORT=<取回的库>` 跑一趟 0/2」，没有 merge 这一步了。 |
| F845 | .claude/skills/crash-test/commit-stages.md:30 | 要改 | 把「双机分片跑的那几条另抄驱动打的 ①（两台工具链相同）、③（两台输入指纹相同）、④（两片开跑那一行…）、⑤（账本拷回）、⑥（三份发现日志）各一行；判绿要这几行都在、`checker-tier-crash-point-replay` 判 merge 那一趟的日志绿」改为：「双机跑的那几条另抄 `<库的家目录>/runs/<标签>/` 下导入那一趟日志（import.log）里的三样原话：`CRASH_POINT_REPLAY_PARALLEL_START` 带 `imported_blocks=<正数> shard=0/2`、一行不带 `blocks_left_to_other_hosts=` 的 `CRASH_POINT_REPLAY_PARALLEL_FINISHED`、计数行 `states` 等于 `closed_form` 且 `violations=0`、`exhaustive=true`；判绿要这几样都在、`checker-tier-crash-point-replay` 判导入那一趟的日志绿」，后半「第二台那一片退 245 或 250–254…」照旧。依据：驱动文件头与自证「判：」一段（`scripts/crash/crash-point-replay-shard-run.sh`）；现查两个驱动脚本里已没有「两台工具链相同」「输入指纹相同」「账本」这几句。 |
| F845 | scripts/README.md:8 | 不相干 | 这一行只是目录说明：「`crash/` ｜ 崩溃点重放、崩溃放量、崩溃点重放双机分片、GPU 清场」，没有写分片怎么跑。 |
| F845 | scripts/crash/README.md:3 | 不相干 | 这一句只是目录说明：「崩溃点重放、崩溃放量、崩溃点重放双机分片与 GPU 清场」，没有写分片怎么跑。 |
| F845 | scripts/mutation/README.md:9 | 不相干 | 这一行已是新说法：「第二台判出的当场导入、不留长期库；变异复跑起的 cargo 写自己的崩溃判定库」，写的是变异双机驱动，与崩溃点重放分片驱动的改法无冲突。 |

## 要主 agent 现查

没有判「要人看」的行。下面三件是我按规则放过或判得较松的，主 agent 抽查复判时可先看：

- 对 `.claude/singlefs-ai-sop/rules/evidence-discipline.md` 旧小节名「新立一条判据，当场拿它回扫已有的条目」的引用（F841 组 `.claude/kb/checks-owed.md` 第 60、135、158、202 行，`.claude/kb/decisions/02-RAID条带策略.md` 第 256 行等）我按事件句放过了，因为它们交代的是当时那一次做了什么；该小节现名「新立一条判据，记下要回扫的条目，随下一次阶段同步一批扫」，这些引文现在指到一个不存在的标题。要消灭悬空引用，可统一把引文标题换成现名；还没做的做法列（`.claude/kb/checks-owed.md` 第 146、178、180 行）我判了要改。
- F831 组 `.claude/kb/experiments/142-新池新建文件的干跑.md` 第 189 行（问题单第 4 行）写「这条流还没枚举过（第十九次跑第二段，重型，未跑）」：我按「这一格量的是第十七次跑那条流」判事件句。那一句写的是 E142 装置主臂是否在新几何上枚举过，我没查有没有后来的主臂产物（`research/results` 里 e142 的最新产物只到第二十五次跑的比对），若主 agent 知道已跑过则这一行要改。
- F831 组 `.claude/kb/milestone/02-second-txn.md` 第 68、198 行（固定脚本做到发布 E、闭式 2104413）按「步 5 落地之后」「2026-09-17 落地」判事件句；第 227 行同类的一句因为已经有「2026-10-11 注」只补了一半，我判了要改。判得是否一致请抽查。

## 汇总

| 判定 | 行数 |
|---|---|
| 要改 | 10 |
| 要补 | 0 |
| 事件句不改 | 202 |
| 不相干 | 87 |
| 要人看 | 0 |
| 合计 | 299 |

数行命令：`grep -c '^| F8' <本报告>` 得 299；各判定行数 `awk -F'|' 'NR>17 && $4 ~ "^ 要改 $"' <本报告> | wc -l` 得 10，其余同法。

要改的 10 行（组 / 载体 / 要点）：
- F831 `.claude/kb/experiments/142-新池新建文件的干跑.md:309`：262165 个自然状态是第十四次跑的数，加上现数 16 777 260。
- F831 `.claude/kb/milestone/02-second-txn.md:227`：两条流的负载句只补了新池新建文件一半，固定脚本流补成到正常卸载共 1 678 377 204。
- F837 `.claude/kb/decisions/17-实现分层与第三方管道.md:86`：「QEMU 档跑的就是静态 musl 二进制」对 checker 档不成立。
- F841 `.claude/kb/checks-owed.md:146`、`:178`、`:180`：未做的做法列还写「当场回扫」，改成记进回扫清单、随下一次阶段同步一批扫。
- F842 `.claude/kb/milestone/03-third-txn.md:190`：「没拦：仓里没有复跑脚本模板」，模板已在 `scripts/agents/three-way-rerun-template.sh`。
- F845 `.claude/skills/crash-test/commit-stages.md:21`、`:30`：双机驱动的 merge 与「①③④⑤⑥」旧输出写法。
- F848 `CLAUDE.md:8`：「恢复重放取前缀的核」应写取前缀、择根、扫环三截。

## 没做什么

- 没改任何文件，没建副本，没起编译，没跑重型测试与门禁。
- 第 10 步的反向核按派发提示不做。
- 候选表行号是出表时的；我按原文逐行重新定位，299 行都在，载体列照候选表原样写。
- 事件句与批次快照的判定靠读上下文，没有对每条记录逐一回到当时的提交核；推翻条件：若某条「事件句不改」的记录行其实在说今天仓里的路径、数、门禁名并且已不真，应改判要改。

## --check-report 输出末行

命令：`python3 scripts/kb/stale-candidates.py --check-report /home/fy5090/.local/share/singlefs/work/rot-2026-10-17/candidates.tsv <本报告> --groups F831,F832,F833,F834,F835,F836,F837,F838,F840,F841,F842,F843,F844,F845,F848,F849`，退出码 0，末行：

  ✓ 报告判全了：299 行候选都有逐行判定
