# 第十二批任务 53：E171 多机入口删掉按此刻空闲现算线程（tooling-writer，2026-10-24）

## 结论

- 第 1 条做完：`rows_orchestrator.py` 的每台线程与 `segment_one_units_revision_8.py` 的本机线程额度（连同干跑「本机空着时」那一份，原取核数）都改成登记的每台默认重活线程数（`scripts/hosts/heavy_threads.py` 的 `default_heavy_threads`），`HOST_THREADS_CEILING = 31` 删了。目录里再 grep `available_threads`，只剩三处假资源 JSON 样本的键（自证与单测喂的输入，代码不读它），没有拿它定线程的写法。
- 第 2 条做了一半：六份入口的 `# long-run-hosts:` 已改成规范句。**两张豁免表在我的写范围外，写闸拒了**（`✗ tooling-writer 的写范围不含 /home/fy5090/code/singlefs/.claude/long-run-live-hosts-exempt`）。要主 agent 删的行写在「要主 agent 做的」一节。表没删之前，主工作区里六份 `long-run-check` 都退 5，红在「登记的 sha256 对不上」。在模拟树里删掉这几行之后，六份都退 0，exempt-check 也退 0。门禁两格被重型测试闸拒了（tooling-writer 不跑门禁阶段），命令交主 agent。
- 第 3 条做完：重钉了 `reuse_library.py` 的 `ORCHESTRATOR_SHA256` 和 `segment_one_units_revision_11.py` 的 `UNITS_R8_SHA256`，别的钉值都没动，理由见「钉值」一节。复用键不含编排器与单元编排的 sha256，所以已判过的装置、E170、执行器、驱动行这几类读数单元的键不变；变的只有 `research-mutations`（mutation-driver，它的 [research] 节是整个研究目录）。
- 第 4 条做完：`test_default_threads.py` 加了两条测试，都喂此刻空闲只报 1 线程的假资源。改回现算之后分别红在第 63、80、82 行的断言；八份单测和十份自证的结果与改前基线相同，只有 `segment_one_units_revision_8.py` 的 e2e-no-room 一格因为语义变了要改样本，改完转绿。

推翻条件：主工作区删掉两张表里那几行之后，`long-run-check` 还有一份不退 0；或者 heavy-thread-defaults 格点名本批改过的五份 .py 里的新行；或者 `rows_orchestrator_revision_11.py` / `segment_one_units_revision_11.py` / `rows_orchestrator_segment_two_revision_11.py` 的自证不再是「红 0」。

## 改过的文件（11 份，都在 `scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/`）

`git diff --stat` 原样：

```
 .../reuse_library.py                               |  3 +-
 .../rows_orchestrator.py                           | 71 ++++++++++++++--------
 ...egment_one_on_the_revision_6_tree_multi_host.sh |  5 +-
 ...egment_one_on_the_revision_8_tree_multi_host.sh |  3 +-
 .../run_segment_one_revision_11_multi_host.sh      |  3 +-
 ...egment_two_on_the_revision_6_tree_multi_host.sh |  3 +-
 .../run_segment_two_revision_10_multi_host.sh      |  3 +-
 .../run_segment_two_revision_11_multi_host.sh      |  3 +-
 .../segment_one_units_revision_11.py               |  5 +-
 .../segment_one_units_revision_8.py                | 37 +++++++----
 .../test_default_threads.py                        | 50 +++++++++++++++
 11 files changed, 141 insertions(+), 45 deletions(-)
```

开工时核过基线：目录在暂存区是整份新加（`A`），没有未暂存的改动；改前的 sha256 记在草稿目录 `baseline-sha256.txt`。改完之后 `git status` 里这个目录带 `AM` 的正好是这 11 份。

## 第 1 条：逐处改了什么

开工时 `grep -rn 'available_threads\|host-resources'` 的命中原样贴过（rows_orchestrator.py 第 25、26、236、249、252、280、306、1114、1319、1363 行；segment_one_units_revision_8.py 第 19、330、340、343、349、766 行；segment_one_units_revision_11.py 第 409 行；六份入口的 long-run-hosts 行）。

rows_orchestrator.py（改后 sha256 e6f3a370…）：
- 第 94 行 `HOSTS_DIRECTORY`（scripts/hosts）取代 `HOST_THREADS_CEILING = 31`。
- 第 275 行新函数 `registered_host_threads`：调 `heavy_threads.default_heavy_threads(os.environ)`，登记读不出时抛 RunCondition（退 78）。每台都取这一个数。
- 第 286、289 行 `allocate(hosts, resources, threads=None)`：不给 threads 就现读登记值，原来的 `min(found.available_threads, HOST_THREADS_CEILING)` 删掉了。签名仍兼容冻结层 `ORIGINAL["allocate"](hosts, resources)` 那种两参调用。
- `HostResources` 去掉 `available_threads` 字段，`resources_from_json` 只要求有 memory 的两项。
- 第 308 行 `single_lane_host`：每台线程都一样，挑「有槽的台里编号最小的」。
- 第 320 行日志行改成「每台线程 N（登记值）」。
- 文件头「资源与分配」与第 13 步那句、`phase_timeout_reruns` 的文档串跟着改了。
- 自证：第 1374 行 `SAMPLE_HOST_THREADS = 31` 给只判内存的三格（today、memory-bound、wrapper-account）用。第 1403 行 allocation-caps 格改成喂此刻空闲 1 线程，核两台都取登记值；allocation-single-lane 改成要本机取登记值。自证项数仍是 34（冻结层 `rows_orchestrator_revision_8.py:56`、`rows_orchestrator_segment_two.py:93` 写死了「自证 34 项，红 0」，所以没有加格）。

segment_one_units_revision_8.py（改后 sha256 be014e33…）：
- 第 331 行新函数 `registered_threads`（同上取法，登记读不出退 78）。
- 第 363 行 `local_budget` 的线程改成 `registered_threads()`。
- 第 350 行 `idle_budget`（干跑「本机空着时」）的线程原取 `cpu.cores`，现在也取登记值，不再要求 cores 字段。
- 文件头「次序与资源」、干跑标题与开跑那行日志跟着改了。
- 第 922 行 e2e-no-room 格：原来靠「此刻空闲 5 线程」放不下，现在线程是登记值，改成内存额度 3 GiB 放不下（单元最少要 4G）。

segment_one_units_revision_11.py 第 410 行只是自证的假资源（`available_threads: 40`），不定线程，没改。

留下的内存护栏，以及为什么留：
- rows_orchestrator.py 第 296、297 行的 `⌊slice_room/包装账⌋`、`⌊available/10G⌋`，加上配置上限那一项：这几项按内存定槽数，防止几道同时冷编译把 slice 内外的余量吃光（文件头「包装账」「实占」两段）。它们不看 CPU 忙闲。
- segment_one_units_revision_8.py 第 362 行 `min(memory["slice_room"], memory["available"])`，以及第 599、605 行逐行编排在跑时再现查的那份 fresh：现在只有内存那一半还起作用。
- `LANE_THREADS_MINIMUM = 5`（每道至少 5 线程，原跑法的限时基线是在这个量级上量的）与 `THREADS = 10`（每个单元的线程，修订 6 跑法第 53–58 行）没动：它们是跑法口径，不是每台默认。

验收 grep（改后，输出不截断）：

```
$ grep -rn --exclude-dir=__pycache__ 'available_threads\|host-resources\|HOST_THREADS_CEILING' .   （cut -c1-260）
segment_one_units_revision_11.py:410:    resources = judge.write_text(os.path.join(root, "resources.json"), json.dumps({"hosts": [{"number": 1, "read": True, "cpu": {"available_threads": 40},
run_segment_one_on_the_revision_8_tree_multi_host.sh:9:# 多机的说明：每台槽数按每台线程与 scripts/hosts/host-resources.py --json 现查的内存额度算（…）
run_segment_two_revision_11_multi_host.sh:13:# 多机的说明：每台槽数按每台线程与 scripts/hosts/host-resources.py --json 现查的内存额度算（…）
run_segment_two_on_the_revision_6_tree_multi_host.sh:10:# 多机的说明：每台槽数按每台线程与 scripts/hosts/host-resources.py --json 现查的内存额度算（…）
test_default_threads.py:72:                json.dump({"hosts": [{"number": 1, "read": True, "cpu": {"available_threads": 1, "cores": 2},
run_segment_one_on_the_revision_6_tree_multi_host.sh:8:# 多机的说明：每台槽数按每台线程与 scripts/hosts/host-resources.py --json 现查的内存额度算（rows_orchestrator.py 文件头「资源与分配」）
rows_orchestrator.py:27:  内存额度由 scripts/hosts/host-resources.py --json 逐台现查（内存护栏），读不到一台退 78。
rows_orchestrator.py:1128:        raise RunCondition(f"host-resources.py --json 退 {completed.returncode}")
rows_orchestrator.py:1333:        resources_command=environment.get("ROWS_ORCHESTRATOR_RESOURCES_COMMAND") or "python3 scripts/hosts/host-resources.py --json",
rows_orchestrator.py:1378:    """假的 host-resources.py --json 一台：threads 只写进 cpu 那一项当此刻空闲线程的样本，编排器不读它（allocation-caps 格核这一条）。"""
rows_orchestrator.py:1381:        entry.update(cpu={"available_threads": threads}, memory={"available": int(available_gib * GIB), "slice_room": int(room_gib * GIB)})
run_segment_two_revision_10_multi_host.sh:13:# 多机的说明：每台槽数按每台线程与 scripts/hosts/host-resources.py --json 现查的内存额度算（…）
run_segment_one_revision_11_multi_host.sh:10:# 多机的说明：每台槽数按每台线程与 scripts/hosts/host-resources.py --json 现查的内存额度算（…）
segment_one_units_revision_8.py:20:  SINGLEFS_THREAD_CAP 设了照它），不看此刻忙闲，登记读不出退 78；内存额度（内存护栏）每次起跑、续跑现查 scripts/hosts/host-resources.py --json：
segment_one_units_revision_8.py:344:    command = override("UNITS_R8_RESOURCES_COMMAND", "python3 scripts/hosts/host-resources.py --json")
segment_one_units_revision_8.py:354:    command = override("UNITS_R8_RESOURCES_COMMAND", "python3 scripts/hosts/host-resources.py --json")
segment_one_units_revision_8.py:357:        raise RunCondition(f"host-resources.py --json 退 {completed.returncode}")
segment_one_units_revision_8.py:780:    resources = judge.write_text(os.path.join(root, "resources.json"), json.dumps({"hosts": [{"number": 1, "read": True, "cpu": {"available_threads": threads},
```
（几行入口注释太长，括号里的部分用「…」代替，完整的在入口文件里。）`available_threads` 还剩四处，在 segment_one_units_revision_11.py:410、test_default_threads.py:72、rows_orchestrator.py:1381、segment_one_units_revision_8.py:780，都是假资源 JSON 的键。`host-resources` 剩下的命中都是读内存额度。

## 第 2 条：入口文件头

六份入口的 `# long-run-hosts:` 一行都改成了规范句，逐字照 `.claude/hooks/lib_long_run.py:103` 的 `CANONICAL_HOSTS_SENTENCE`：

```
# long-run-hosts: 台数从 crash-point-replay-shard-configuration-check.sh --emit-hosts 读，每台线程取 .claude/heavy-thread-default 的登记值，不现算、不写常量
```
原来写在那一行里的说明另起了一行 `# 多机的说明：…`（按内存现查槽数、几秒重查加槽、复用库只在本机）。修订 6 第一段那份入口第 3 行「第 1 行到第 350 行……只改了这几行注释」改成了 351，因为文件头多了一行。改后的 sha256：修订 6 第一段 732473a1…，修订 6 第二段 fbf446e2…，修订 8 第一段 7623b047…，修订 10 第二段 4af1ac4b…，修订 11 第一段 b79c8170…，修订 11 第二段 0c2fe4eb…。

主工作区（两张表还没删）：`bash scripts/hosts/detached-run.sh long-run-check -- bash <入口>` 六份都退 5，原样输出在草稿目录 `long-run-check-main-tree.txt`。每份都只红在一条上，原文开头是「…在 .claude/long-run-live-hosts-exempt 里登记的 sha256 是 aae881ff…」。判法库 `python3 .claude/hooks/lib_long_run.py exempt-check /home/fy5090/code/singlefs` 退 1，原样（每行截到 330 字）：

```
  ✗ .claude/long-run-live-hosts-exempt 有 6 处不对（共 6 行）：
     第 12 行的 …run_segment_one_on_the_revision_6_tree_multi_host.sh 现在的 sha256 是 732473a1026b6e1456d44328a13680a6fc69373a23e51758bafa11cc91abdcf2，表里登记的是 aae881ff…（脚本改过了…
     第 13 行 …（同上，fbf446e2… / d18eef80…） 第 14 行（7623b047… / 7407f211…） 第 15 行（4af1ac4b… / 2bb604ca…） 第 16 行（b79c8170… / 3fa96f9f…） 第 17 行（0c2fe4eb… / 8e10231f…）
     → 怎么办：这份脚本不该再改：把改动撤回去；确要改它，文件头按登记值读法写（多机一行用规范句）并删掉这一行，或交主 agent 问用户再改表里的 sha256
rc=1
```

模拟树：把 `.claude/hooks/lib_long_run.py`、`lib_shell_words.py`、`lib_heavy_tests.py`、`.claude/heavy-thread-default`、六份改后的入口拷到一棵临时树（做法同 `detached-run.sh` 自证「长活⑩」第 2047–2057 行），豁免表只留文件头第 1–9 行，然后逐份跑 `python3 <树>/.claude/hooks/lib_long_run.py script <树> -- bash <树>/<入口>`。六份都退 0，每份都报「✓ 长活声明齐了（…，判了 3 个键）：分片：…；续跑：…；多机：台数从 crash-point-replay-shard-configuration-check.sh --emit-hosts 读，每台线程取 .claude/heavy-thread-default 的登记值，不现算、不写常量」。exempt-check 退 0，原样：

```
  ✓ .claude/long-run-live-hosts-exempt 0 行：每行的脚本都在、登记的 sha256 都对得上，文件头都得靠现算读法才齐（按登记值读不齐）；没给基准（基准那一版里还没有这张表，或不在 git 仓里），只缩不涨这一条没比
exempt-check rc=0
```
模拟树（744K）用完删了。

门禁两格：我照提示起了 `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 heavy-thread-defaults、long-run-header-exempt 两格; …' bash .claude/gate.d/code-tooling.sh --check heavy-thread-defaults,long-run-header-exempt <仓根>`，被 `.claude/hooks/heavy-test-guard.sh` 拒了（原话「✗ 提交时才跑的检查被拒：门禁阶段 code-tooling.sh … tooling-writer 不跑「提交时才跑的检查」」）。我没有换写法绕过去，命令交主 agent。

## 第 3 条：钉值

目录里每一处钉值都按 `reuse_library.py` 的 `PIN_SITES`（第 1866–1871 行一带）逐个核过，另外全仓（不含 research/、target/）grep 了改前那几份的 sha256：

重钉了两处：
- `reuse_library.py:424` `ORCHESTRATOR_SHA256`：b48bf322… → e6f3a370ca6f66577e5cd9f3d44a30dd315660958ebb50922df3fc53ec6ef8ff（rows_orchestrator.py 改后内容）。上面一行注释写了这一次重钉的出处（records/2026-10-22-子agent结果落地审计.md 第十二批）。修订 11 载入冻结层时经 `orchestrator_repin` 和 `rows_revision_8_repins` 把冻结层里的旧钉值换成这个新值，所以冻结层文件本身一个字节不用改。
- `segment_one_units_revision_11.py:56` `UNITS_R8_SHA256`：cc52fcfc…（搬目录之前那一版，靠倒换搬迁来认）→ be014e3334d494e4f947ef3de367461a5d2c48fe5fdb01a4be0facefe3e74537。文件头第 5、6 行「一个字节不改（…cc52fcfc…）」跟着改成了说明重钉。

没动的，以及为什么：
- `reuse_library.py:420` `ORCHESTRATOR_SHA256_BEFORE_MOVE`（1dcfb8d0…）：搬目录之前那一版，冻结层正文里写的就是它，载入时才换。照原样留。
- `SEGMENT_TWO_ORCHESTRATOR_SHA256`（7dda75bc…）、`CROSS_HOST_CHECK_SHA256`（4ab95241…）、`rows_orchestrator_revision_11.py` 的 `ROWS_R8_SHA256`（25d77b2b…）、`rows_orchestrator_segment_two_revision_11.py` 的 `SEGMENT_TWO_R10_SHA256`（627e48c6…）：被钉的那几份这一批没改。
- 冻结层自己写的 1dcfb8d0…（`cross_host_o3_check.py:48`、`rows_orchestrator_revision_8.py:53`、`rows_orchestrator_segment_two.py:92`）和 `rows_orchestrator_segment_two_revision_10.py:79` 的 7d7b458b…：都是搬目录之前那一版，按提示照原样留。这四层直接跑本来就退 3（基线也是这样，见下表），只能经修订 11 换钉值后载入。
- `revision_11_placeholders.json` 的 H31（九个新文件 → sha256，「执行员写完新文件之后照它们现在的 sha256 填、主 agent 认」，三层编排 run 之前都核）：**这一批之前九项就已经全对不上了**（逐个比过，全是 DIFF，比如 reuse_library.py 登记的是 c8b9080e…，现在是 bb53e843…），是搬目录那时起就过期的。按定义它要主 agent 认，我没填。下一次起修订 11 之前，要主 agent 照现在的内容重填并认可，不然三层编排都会退 78。
- `revision_8_research_side_of_the_revision_6_run.json`：记的是修订 6 那一趟研究一侧的 sha256，是历史记录，没动。`revision_8_placeholders.json` 的 H17 是 `files: null`，没有钉值。

复用键受影响的层（`reuse_library.py` 文件头键的十节与 `reading_unit_sections` / `driver_research`）：
- 编排器与单元编排的 sha256 都不进键；`[args]` 去掉了 `--threads`、`--lanes`、`--memory-cap`；线程数在键外（第 11.2 节「键外的几样」）。所以装置（device）、E170、执行器（o3-executor）、驱动行（driver-row）的键不因为这一批改动而变：它们的 [research] 只认 device_timing.py、row_driver.py 与它 import 的四份、执行器那几份，这一批都没碰。
- 会变键的只有 `research-mutations`（mutation-driver）：它的 [research] 是整个研究目录（`research_lines(repo, [RESEARCH_RELATIVE, …])`），目录里任何一个字节变了，键就变。下一次跑时这一单元要现跑、重判。
- 实际起跑另外还会被这几样拦住：H31 过期（三层编排退 78，见上）；修订 6 第二段、修订 10 第二段起跑时写的输入清单（`<导出根>/inputs.sha256`，里面有 rows_orchestrator.py、segment_one_units_revision_8.py 等）跟以前那一趟对不上，所以续跑以前那一趟会被清单拦停，要另起一趟；修订 8 单元编排续跑时清单里的 sha256 也一样。

## 第 4 条：单测与自证

新加两条测试（`test_default_threads.py` 的 `RegisteredHostThreadsTests`），跑之前都去掉 `SINGLEFS_THREAD_CAP`：
- `test_every_host_of_the_rows_orchestrator_gets_the_registered_threads`：两台此刻空闲都只报 1 线程，内存 200 GiB。核两台的 `allocation.threads` 都等于登记值（第 63 行），每台至少一槽，`single_lane_host` 挑出的那台也是登记值。
- `test_the_local_budget_of_the_revision_8_units_takes_the_registered_threads`：本机此刻空闲 1 线程、核数 2。核 `local_budget` 的线程等于登记值（第 80 行），内存是 min(slice_room, available) = 20 GiB；`idle_budget` 的线程也是登记值（第 82 行），内存是 min(slice_total, mem_total) = 60 GiB。

先证红（在草稿目录的临时副本里改回，主工作区没碰）：
- rows_orchestrator.py 换回暂存区那一版（现算写法）：红在第 63 行，原样是 `AssertionError: Lists differ: [1, 1] != [31, 31]`，`FAILED (failures=1)`（草稿目录 `red-rows-orchestrator-2.txt`）。
- segment_one_units_revision_8.py 换回暂存区那一版：红在第 80 行，`AssertionError: 1 != 31`（`red-units-r8.txt`）。
- 只把 `idle_budget` 那一行改回取 `cpu.cores`：红在第 82 行，`AssertionError: 2 != 31`（`red-idle.txt`）。
- 换回现在这一版：`Ran 5 tests … OK`。临时副本（1.6M）用完删了。

改前与改后（主工作区，`timeout … nice -n 19 python3 -B …`，在这个目录里跑），末行原样：

| 项 | 改前基线 | 改后 |
|---|---|---|
| rows_orchestrator.py --selftest | 退 0「自证 34 项，红 0」 | 退 0「自证 34 项，红 0」（7 个弄坏开关都红在点名的格） |
| segment_one_units_revision_8.py --selftest | 退 0「自证 25 项，红 0」 | 中间一次退 1「红 1」（e2e-no-room，「本机只有 5 线程：退 0」）；改了样本之后退 0「自证 25 项，红 0」（budget-ignored 等 11 个弄坏开关都红在点名的格） |
| segment_one_units_revision_11.py --selftest | 退 0「自证 10 项，红 0」 | 退 0「自证 10 项，红 0」 |
| rows_orchestrator_revision_11.py --selftest | 退 0「自证 10 项，红 0」 | 退 0「自证 10 项，红 0」 |
| rows_orchestrator_segment_two_revision_11.py --selftest | 退 0「自证 11 项，红 0」 | 退 0「自证 11 项，红 0」 |
| reuse_library.py --selftest | 退 0「自证 48 项，红 0」 | 退 0「自证 48 项，红 0」（pin-moves-real-files「6 处钉值、6 份被钉文件…」绿） |
| rows_orchestrator_segment_two.py --selftest | 退 3（rows_orchestrator.py 不是 1dcfb8d0…） | 退 3（同一原因，基线就这样） |
| rows_orchestrator_revision_8.py --selftest | 退 3（同上） | 退 3（同上） |
| rows_orchestrator_segment_two_revision_10.py --selftest | 退 3（rows_orchestrator_segment_two.py 不是 7d7b458b…） | 退 3（同上） |
| cross_host_o3_check.py --selftest | 退 3（被钉那份内容变了） | 退 3（同上） |
| test_default_threads | OK，3 项 | OK，5 项 |
| test_segment_one | FAILED (failures=1)，33 项 | FAILED (failures=1)，33 项（同一条：`test_a_product_of_another_run_is_refused_before_anything_is_written` 第 256 行 `77 != 2`，不是这一批的改动） |
| test_segment_two | OK，17 项 | OK，17 项 |
| test_revision_8 | 退 3（导入时核钉值） | 退 3（同上） |
| test_revision_10 | 退 3（同上） | 退 3（同上） |
| test_revision_11 | OK，20 项 | OK，20 项 |
| test_frozen_o3 | OK，16 项 | OK，16 项 |
| test_device_timing | OK，8 项 | OK，8 项 |

原样输出在草稿目录的 `baseline/` 和 `final/` 下（同名 .txt 与 .rc）。

弄坏开关：没有新加。只判「每台线程等于登记值」的反面，就得在代码里留一条读 `available_threads` 的路，这样验收 grep 就清不掉。所以红是在临时副本里改回现算证的（见上）。新写的 `# gate-similar:` / `# hook-events:` 行：没有，这一批没加门禁也没加钉子。

语法检查：六份入口 `bash -n` 都过了；五份改过的 .py 用 `py_compile`（cfile 写到草稿目录）都过了。命名两条（照共用约束「命名检查交回前跑」，这一批没改 .rs，属于顺手核一遍）：`code-source-discipline.sh --check vague-names,test-file-names,ordinal-names,data-names` 退 1，1009 行 ✗，按本批 11 份文件过滤命中 0 行（汇总「4 格里 3 格红（vague-names ordinal-names data-names）」，命中都在别的文件，比如 `.claude/kb/tooling.md:120`）；`naming-lint.sh` 退 1，「3493 处名字不合命名纪律（查了 777 个 .rs 文件…）」，只查 .rs，本批没有 .rs。

## 发现（不在这一批的范围里，交主 agent 判）

1. 修订 8 逐行编排的「加槽」（`rows_orchestrator_revision_8.py:178` `try_regrow`，第 186 行对每台调 `one.allocate`）会给已经有槽的台再加槽。以前每台线程取此刻空闲，在跑的槽会把空闲压下去，所以加出来的槽线程不多。现在每台线程固定是登记值，每一次加槽都按满额 31 再分一遍，这台总共起的线程可能超过登记值，只剩内存那几项在挡。第二段的加槽（`rows_orchestrator_segment_two.py` 的 `make_regrow`）只给起跑时一槽都没有的台加，没有这个问题。要修，就得在修订 11 那一层给加槽扣掉这台已经分出去的线程，或者改冻结的修订 8 再重钉 `ROWS_R8_SHA256`。推翻条件：加槽那一条路上另有地方扣掉了已有槽的线程（我只读了第 160–205 行）。
2. 同一台本机上，第 0–10 步的单元（线程额度是登记值）和第 11–13 步的逐行编排（每台线程也是登记值）在修订 8 里会有一段时间同时跑。以前两边都看此刻空闲，会互相让；现在线程这一半不再互相让，只有内存还在挡（`segment_one_units_revision_8.py:599`、`605` 那份 fresh 现查）。
3. `revision_8_placeholders.py` 的 RUNTIME_PLACEHOLDERS H19 和 `revision_10_placeholders.json` 的 H29 写的是「编排器每次起跑与续跑现查现算（…槽…）」。槽数仍然按内存现算，说法没错，所以没改。

## 要主 agent 做的

1. 删两张豁免表里的行（我的写范围外，写闸拒了）：
   - `.claude/long-run-live-hosts-exempt` 第 10–17 行：空行、第 11 行注释「# E171 的六份多机入口被 sha256 钉住不能改，…」和第 12–17 行六份入口。删完只剩第 1–9 行文件头，与我在模拟树里核过 exempt-check 退 0 的那一版逐字相同（`head -n 9`）。
   - `.claude/heavy-thread-exempt` 第 53–58 行：空行、第 54–55 行注释「# E171 修订 6–11 的第一段编排器：…」与第 56–58 行三行 `rows_orchestrator.py:` 锚串（`available_threads: int`、`min(found.available_threads, HOST_THREADS_CEILING)`、`可用线程 {found.available_threads}`）。这三处锚串现在一处都命中不到，不删的话 heavy-thread-defaults 会判「锚串一处被判行都不命中」红。
2. 删完再跑：
   - 在仓根逐份跑 `bash scripts/hosts/detached-run.sh long-run-check -- bash scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/<入口>`，六份都应退 0。
   - `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 heavy-thread-defaults、long-run-header-exempt 两格; 范围=2 格; 为什么=核 E171 多机编排改成每台取登记线程之后线程默认与长活豁免表的判法' bash .claude/gate.d/code-tooling.sh --check heavy-thread-defaults,long-run-header-exempt /home/fy5090/code/singlefs`：看两格都绿、报的行数是多少。heavy-thread-defaults 只要点名本批改过的五份 .py 里的新行，就推翻第 1 条的结论。
3. 下一次起 E171 修订 11 之前：照现在的内容重填 `revision_11_placeholders.json` 的 H31，并认可。

## 给实验页与变更史用的一段

E171 多机编排的每台线程不再按此刻空闲现算（原来是 `min(cpu.available_threads, 31)`），改成取登记的每台默认重活线程数（`.claude/heavy-thread-default`，经 `scripts/hosts/heavy_threads.py`）。第一段编排器 `rows_orchestrator.py`（修订 6、8、10、11 共用）和修订 8 单元编排 `segment_one_units_revision_8.py`（修订 11 共用）的本机线程额度都改了；槽数仍按内存现查（slice_room、MemAvailable、配置上限）。依据是用户原话「peer-run.sh 那套「核数减负载」 删掉 。直接改。豁免 。 检查还有没有类似的 有就删掉」，主 agent 全仓查过，只剩 E171 这一处。重钉了 `reuse_library.py` 的 `ORCHESTRATOR_SHA256`（→ e6f3a370…）和 `segment_one_units_revision_11.py` 的 `UNITS_R8_SHA256`（→ be014e33…）；搬目录之前那一版的钉值照留。复用键不含编排器与线程数，所以装置、E170、执行器、驱动行这几类读数单元的已判结果照样复用；变键要重判的只有 `research-mutations`。另外 H31 早就过期，起跑前要重填、重认；以前那几趟的输入清单与新内容对不上，要另起一趟，不能续跑。

## 没做什么

- 没跑重型测试（checker-tier-crash-point-replay、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification、checker-tier-research-build-and-replay、checker-tier-broad-mutation、gate.sh、全量 cargo test、check.sh）；没走定义三方；没提交。
- 门禁 code-tooling 的两格没跑（被闸拒，交主 agent）；两张豁免表没改（写范围外，交主 agent）。
- 没起 E171 的入口，没跑它的产物；没改 `research/` 下的冻结证据；H31 没填。
- 冻结的修订 8、修订 10、第二段编排器与 cross_host_o3_check.py 没改（也就没修「发现」第 1 条的加槽超额）。
- 做 py_compile 的时候在目录的 `__pycache__/` 里写了一份 rows_orchestrator 的 .pyc，当场删了（`__pycache__` 不进 git）。
- 草稿目录里留着报告、基线与改后的输出、`edits/`（替换表与 .new 文件，几百 KB 的文本）；临时副本与模拟树都删了（删之前 1.6M、744K）。
