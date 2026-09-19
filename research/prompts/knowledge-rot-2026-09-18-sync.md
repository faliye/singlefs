<!-- knowledge-sync -->
# knowledge-rot-2026-09-18 阶段同步

触发文件：research/scripts/vm-geom.sh、research/scripts/vm-bench.sh、research/scripts/quote-kb.py、research/scripts/verify-citations.sh、research/scripts/replay.sh、research/scripts/e6-units.sh、research/scripts/vm-kernel.sh、research/scripts/oov-check.py、research/scripts/fetch-refs.sh、research/scripts/pdf-text.py、research/scripts/replace-once.py、research/scripts/stale-candidates.py、.claude/hooks/write-guard.sh、.claude/agents/sweep.md、.claude/gate.d/73-research-gate-lint.sh、.claude/gate.d/47-research-script-selftests.sh、.claude/gate.d/stage-owners.tsv、research/scripts/fixtures/stale-candidates-benchmark-facts.tsv（第二段：C382 补出路与门禁 73 号、回扫员可靠性改造；`.claude/main-agent.md` 那一行调度也改了，它与 47 号、stage-owners.tsv 同时有别的会话的改动）

范围：`b1c8cef~1..HEAD` 十六个提交加工作区里让现状句变成假话的地方。回扫员四份报告 `research/prompts/knowledge-rot-2026-09-18-sweep-a.md` 到 `-sweep-d.md`；A、C 报零处要改，B 报三处，D 报两处（其中一处判错）。欠账表前置列那一类是主 agent 自己按「层 0 只有 / 多次挂载的录制流」定向搜出来的，四份报告都没报。

## 搜索

- 门禁 kb 快阶段 44 个与 doc-lint 在工作区上各跑一遍：40 绿，红的 66、68、69、72 号只指向别的会话在飞的 bg-notify-r1、m2-supp3-item1、E152 装置与 main-agent.md / agent-common.md；doc-lint 查 433 项全绿
- `grep -rn -F` 逐个搜「层 0 只有」「层 0 的负载只有」「层 0 没有」「多次挂载的录制流」「非首次挂载」「只有一次挂载」于 CLAUDE.md、.claude/kb、.claude/rules、.claude/skills、records（变更史与历史版本节除外）→ 25 处，其中 checks-owed.md 20 处、decisions/05 一处、records/2026-09-13-总审核.md 三处
- `awk` 取欠账表前置列含 checker / 可写挂载 / 回退 / 层 0 的行 → 45 行，逐行对里程碑「第二个事务」增补 2 收口表与 `research/prompts/m2-closeout-debts-check.md`
- `grep -rn -E` 搜「crates/ 里没有」「还没实现」「尚未实现」类现状句于 kb → 24 处，逐处判
- `grep -oE 'I-[0-9]+\.[0-9]+' crates/singlefs-harness/tests/checker_known_bad_images.rs | sort -u` → 29 个不变量号
- `bash .claude/scripts/gate-lint.sh`（整仓）→ 90 处红，353 个脚本、498 条拒绝

## 命中处置

| 载体 | 原句 | 处置 |
|---|---|---|
| .claude/kb/checks-owed.md:24 | C13 前置：池级 checker 判的 23 条每条一份坏镜像 | 改了：写成 2026-09-18 起 29 条，步 6 与增补 2 加的六条各配一份（`checker_known_bad_images.rs` 现数 29 个不变量号） |
| .claude/kb/checks-owed.md:33 | C22 前置：层 0 的负载只有第一个事务，没有释放与重用 | 改了：第二条流带释放与重用；欠的是点名 I-4.8 的断言（收口表第 22 行） |
| .claude/kb/checks-owed.md:42 | C29 前置：层 0 只有一次挂载的第一个事务，这一格要的场景还不在负载里 | 改了：多次挂载已有，同一实例内的陈旧 tail 造不出、「先信 tail」没有会红的检查（收口表第 22、23 行） |
| .claude/kb/checks-owed.md:52 | C42 前置：层 0 没有「恢复 → 续写 → 再崩」，残留记录还种不进基镜像 | 改了：能种残留记录、只有正例；反例随 C124 打回重议 |
| .claude/kb/checks-owed.md:129 | C124 前置：层 0 没有回退 | 改了：第二条流有回退（发布 D）；缺的形态打回重议（收口表第 ① 行） |
| .claude/kb/checks-owed.md:146 | C143 前置：多次挂载的录制流 | 改了：注明录制流已有、还要连续根读不出的故障注入，另一半交用户（收口表第 10 行） |
| .claude/kb/checks-owed.md:149 | C147 前置：要 checker 存在 | 改了：I-7.8 已实现（2026-09-14），坏镜像只罩判别力第 ① 格，「根轮出环」那一格没有坏镜像 |
| .claude/kb/checks-owed.md:255 | C274 前置：层 0 只有一次挂载的第一个事务 | 改了：三次可写挂载都是正常退出后重开，没有重发在飞 checkpoint 的切换 |
| .claude/kb/checks-owed.md:262 | C281 前置：检查仍欠：事务层、管理员回退实现、崩溃点重放 harness | 改了：三样都有了，逐句核差 I-7.2 本身的判别力断言（`m2-closeout-debts-check.md` 第 76 行） |
| .claude/kb/checks-owed.md:267 | C286 前置：层 0 只有一次挂载的第一个事务 | 改了：没有只读之后 remount 成可写 |
| .claude/kb/checks-owed.md:268 | C287 前置：层 0 只有一次挂载的第一个事务 | 改了：没有切换收养开放 checkpoint 之后再崩 |
| .claude/kb/checks-owed.md:292 | C314 前置：检查仍欠：崩溃点重放 harness（多次挂载的录制流） | 改了：录制流已有，格 1 没有专门断言（`m2-closeout-debts-check.md` 第 103 行） |
| .claude/kb/checks-owed.md:296 | C318 两列：被抛弃根的已分配统计量 − R_old 的 | 改了：照 D28 已定项 1 第九项 2026-09-17 口径写只被被抛弃根引用的槽数，点名 `mount.rs` 的隔离函数 |
| .claude/kb/checks-owed.md:300 | C329 前置：层 0 之外要有非首次挂载路径的录制流 | 改了：录制流已有，写行那次准入充裕，要另造准入不够的负载 |
| .claude/kb/checks-owed.md:301 | C330 前置：多次挂载的崩溃点重放 | 改了：已有，回退那次写中间实例行 (2, 0, 0)，会红的检查仍欠 |
| .claude/kb/checks-owed.md:302 | C331 前置：多次挂载的录制流 | 改了：已有；修法条款空白（收口表第 6、7 行） |
| .claude/kb/checks-owed.md:303 | C332 前置：多次挂载的录制流 | 改了：已有；缺根槽读失败的开关（收口表第 7、24 行） |
| .claude/kb/checks-owed.md:304 | C333 前置：多次挂载的录制流 | 改了：已有；删行 `crates/` 里没有（`mount.rs:106-107`「第一版都不做」） |
| .claude/kb/checks-owed.md:305 | C334 前置：多次挂载的录制流 | 改了：已有；定义没被攻过（收口表第 16 行） |
| .claude/kb/checks-owed.md:306 | C335 前置：多次挂载的录制流 | 改了：已有；缺根槽读失败的开关 |
| .claude/kb/checks-owed.md:325 | C353 前置：层 0 的固定脚本要多于一次发布——步 0 | 改了：已满足，用例 ③ 断言差异态 1；「拿掉新根段归 0」的判别力自证没有 |
| .claude/kb/checks-owed.md:356 | （新增） | 补了：C382 研究脚本与 hook 的拒绝不在门禁自检射程里（单跑 gate-lint 整仓红 90 处，d2aeb7d 记过 64 处未入账） |
| .claude/kb/checks-owed.md:98 | C88 前置：时间线判别还没写进 checker | 不改：现查 `crates/singlefs-checker/src` 没有同 (实例代号, checkpoint_txg) 双根的判定，仍是真话 |
| .claude/kb/decisions/05-快照-空间记账机制.md:351 | 回退选中 R_old 那一刻按「被抛弃根的已分配统计量 − R_old 的已分配统计量」算 | 改了：照 D28 已定项 1 第九项 2026-09-17 口径；变更史 2026-09-18 |
| .claude/kb/decisions/05-快照-空间记账机制.md:220 | 层 0 的负载只有第一个事务、一条 deadlist 条目都不写 | 改了：写两条流都不写 deadlist 条目（`crates/` 只注册 deadlist 树、不写条目），零覆盖结论不变 |
| .claude/kb/decisions/22-单元原子性怎么合成.md:492 | 恢复后生效值 = 各幸存盘所带 F 最大值的最小值 | 改了：句末加「生效」2026-09-17 打回重议、定案前照字面 |
| .claude/kb/decisions/16-发布语义.md:207 | **检查**那一半仍欠（崩溃点重放 harness 的多次挂载录制流） | 改了：录制流已有，C314 逐句核仍差格 1 的专门断言 |
| .claude/handover/qemu-herd7/README.md:28 | ## singlefs 下次同步 SOP 之前要改的（五条待办） | 改了：改成现状表，四条已做；`install-owned` 两行 litmus 仍欠（回扫员 D 判成已做，现查 `install.sh` 第 215–219 行仍会判红） |
| .claude/kb/invariants.md:57 | I-7.4 / I-4.8 按 F_生效 定候选集，没复述「生效」打回重议 | 不改：打回之后定案前照字面，两条按今天的字面读法实现；打回的说明在 D16 已定项 1 与 I-3.1 那一行 |
| research/scripts/path-moves.tsv:1 | logs/ → briefs/ 改名没登记 | 不改：改名（2026-09-16）早于搬迁登记规则（2026-09-17），全仓旧路径零命中，登记表只为改写服务 |
| records/2026-09-13-总审核.md:494 | 还没做：多次挂载的录制流 | 不改：十一·九是 2026-09-14 那一轮的记录，说的是当时 |
| .claude/kb/milestone/02-second-txn.md:547 | 清单 26 条 | 不改：2026-09-17 步 6 落地当时的数，事件句；文件正被别的会话写 |
| .claude/agents/sweep.md:35 | 7. 阶段同步的：从改动范围与做成的事取关键词…… | 改了：阶段同步改成写事实表（过 `stale-candidates.py --check-facts`）加逐行判（过 `--check-report`），判据写成判分句、带日期的现状句当现状判；经过与实测见 `records/2026-09-18-回扫员可靠性.md` |
| .claude/main-agent.md:43 | `sweep`（阶段同步；输入给改动范围与做成的事……） | 改了：先派一个 sweep 写事实表，候选表按组切段逐行判，交回后核对器与抽 20 行复判 |
| .claude/gate.d/73-research-gate-lint.sh:24 | （三方第一轮打中：样本目录下也扫真仓 SOP 包） | 改了：设 GATE_LINT_DIR 指到第一个目标目录，判决 `research/prompts/sweep-rel-r1-main-verification.md` |
| .claude/gate.d/73-research-gate-lint.sh:1 | （新增） | 补了：研究脚本与 hook 的拒绝进门禁自检射程，C382（研究脚本与 hook 的拒绝不在门禁自检的射程里） 挪进已还清 |
| .claude/kb/decisions/08-核心索引结构.md:54 | 单线程 30.9–31.8M 条目/秒（707–727 MiB/s，四轮……），42.2 GB/s | 改了：照 E17 八轮区间 29.9–31.8M、40.8 GB/s、并行 5.49–6.38×；D9、D13、D25、E17 页与实验索引同改，变更史 2026-09-18（其二） |
| .claude/kb/decisions/06-快照实现模型.md:236 | 树表第二层门槛 66 头（每头一棵树是 44 头） | 改了：39 / 27 头（E132 按条目 200 重跑）；同文件两处 148 字节改 200，变更史 2026-09-18（其三） |
| .claude/kb/decisions/16-发布语义.md:211 | C314 还清之后落到字节表……在那之前字节表暂按 1 | 改了：2026-09-13 起已落、已生效；同文件第 291 行与 milestone/01-first-txn.md:184 同改 |
| .claude/kb/decisions/05-快照-空间记账机制.md:252 | defer 队列待释放（第 5 项）每盘一行值 0 | 改了：16384（2026-09-17 起） |
| .claude/kb/checks-owed.md:150 | C148 行首：条目从 67 变 121、每层 134 棵 | 改了：行首补 2026-09-16 加宽到 200、每层 81 棵，原文照录；C157、C272、C352 同批改 |
| .claude/kb/verification-build.md:157 | 层 0 的固定负载要含第二个事务…… | 改了：句末补第二条流已含覆盖写、释放、重开、回退、抬 F 与复用；第 155、163、237 行同样补现状 |
| .claude/kb/decisions/26-后台整理与放置回收.md:234 | deadlist 条目内容随位置权威一起定 | 不改：那一轮论证的判决行，说的是那一次 |
| .claude/kb/decisions/08-核心索引结构.md:483 | 每条目 18 字节（长度 2 + 预留 16） | 不改：明写按 2026-09-06 的条目宽算的例子 |
| .claude/kb/decisions/16-发布语义.md:209 | 崩溃点重放 harness 那条检查仍欠 | 不改：「前置的进展（2026-09-13）」那一天的记录 |
