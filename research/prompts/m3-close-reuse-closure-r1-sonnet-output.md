# m3-close-reuse-closure-r1 云端正推腿（Sonnet）报告（2026-10-22）

立场：正推。从 D13 已定项 17 的条款与 `scripts/`、`crates/` 今天的实现推「丙加甲应该是什么」，再与题面的改法逐格比。方案按 `crates/` 与 `scripts/` 今天的实现谈；Rust 那一份（`crates/singlefs-unit-graph`）这一轮没读也没动，所有闭包数都是 `scripts/gate/lib_function_units.py` 量的。
标记：「量的」= 这一轮在冻结副本上跑出来、原样输出落盘；「读的」= 读代码得出；「推的」= 由前两者推出、没有直接跑过。

## 0 开工核对、复跑、文件

被判脚本 sha256 与快照核对（`sha256sum -c research/prompts/m3-close-reuse-closure-r1-snapshot/judged-sha256.txt`，原样输出）：

```
scripts/gate/lib_function_units.py: OK
scripts/hosts/run-with-memory-cap.sh: OK
scripts/hosts/host-resources.py: OK
scripts/hosts/host-dispatch.py: OK
scripts/hosts/peer-host-lib.sh: OK
scripts/gate/verus-proof-check.py: OK
```

三份 kb（`.md`、`.en.md`、`.ja.md`）对 `kb-sha256.txt` 也是 OK。

冻结副本：`~/.local/share/singlefs/work/m3-close-reuse-closure-r1-sonnet/repo/`（主工作区 HEAD 841b3beab 加未提交改动，`rsync -a --max-size=2M` 拷出 `scripts`、`.claude`、`crates`、`Cargo.*`、`.cargo`，不拷 `target`；副本里六份被判脚本的 sha256 与快照相同）。副本与闭包原始输出（`closure-*.txt`、`dump-*.txt`）都在草稿目录，没入库：每份都是几百 KB 到 140 MB 的中间物，报告里引的数由下面的模型目录复跑得出。切边副本（量数时建在草稿目录里的 `variants/cut-lookup/` 与复跑建的 `rerun-work/`，各 140 MB）已删，复跑时 `rerun.sh` 在 `--work` 下重建；冻结副本留着（`~/.local/share/singlefs/work/m3-close-reuse-closure-r1-sonnet/repo/`，140 MB），因为主 agent 复跑的 `--frozen` 要指它，判决核完删。

模型目录 `research/prompts/m3-close-reuse-closure-r1-sonnet-model/`，复跑（整条经内存包装，工作目录随意；草稿目录给新的）：

```
bash scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/m3-close-reuse-closure-r1-sonnet-model/tools/rerun.sh \
  --repo <仓根> --frozen ~/.local/share/singlefs/work/m3-close-reuse-closure-r1-sonnet/repo --work <新的草稿目录> \
  --model <仓根>/research/prompts/m3-close-reuse-closure-r1-sonnet-model
python3 research/prompts/m3-close-reuse-closure-r1-sonnet-model/tools/unit_diff.py <冻结副本> <切边副本>
bash research/prompts/m3-close-reuse-closure-r1-sonnet-model/tools/fixture_closure.sh <冻结副本> <仓根>/research/prompts/m3-close-reuse-closure-r1-sonnet-model
bash research/prompts/m3-close-reuse-closure-r1-sonnet-model/tools/inline_refusals.sh <冻结副本>
bash research/prompts/m3-close-reuse-closure-r1-sonnet-model/tools/entries_scan.sh <冻结副本> research/prompts/m3-close-reuse-closure-r1-sonnet-model/tools/entries.txt
```

切边副本由 `rerun.sh` 在 `--work` 下的 `trees/cut-lookup/` 里自己建（`tools/cut_lookup_edge.py`）。各文件 sha256（`sha256sum` 原样）：

```
5ecc501be5c85f8ce2e20a54ddb05d149ccf790e01be7302f4bd05db45813c03  tools/cut_lookup_edge.py
70ba9aa58382a8f9c5249b24a53c1721ef933b6e63ff5ff473541d19a9e61485  tools/exception_probe.py
339ac38dd6376d6951cb4a108d36ba1364c8958025f27335cbcc4a9fbc0d1ac8  tools/rerun.sh
e740587ce68e6d86ad8fd85019abac2c3e18eba11125adfa043bff4603f2c1ab  tools/unit_diff.py
09bbe3c48d21c764eb0ebddd2a9eeb40f0207d61b3344774c608cc8ba26047aa  tools/fixture_closure.sh
3d7b08314282e049a52e966bb2b59da430db1661e6430c0e135ea3ebb5c0f99c  out/rerun-1.out
6188dc9ecd9a1730e9d5629e3e7854741760217357a1afc57fd815eccf75af25  out/unit-diff.txt
b26403964234b4207c8d5eacde770ac946fe9fcd612ef1e024ce3c8a361fbcae  out/fixture-closure.out
8c0a1173f37fc72346974577dac72eb701a8b2eaadd6ca363595efefe8b6bc3b  fixtures/scripts/inner.sh
570a6fa858a2c8df8b152d20ba62cc886e0f7a9715df3680a8f4de57d017fd06  fixtures/scripts/main_callback.sh
7d82f3b12969a2a958d9b551a4f159831ee355ac4833573520a69ca1c8106cc2  fixtures/scripts/main.sh
e69709f73e986ff83f520f42511322d1f596ba9c723779506c71f77c93b64b74  fixtures/scripts/main_text.sh
dd7b6975e2089027720d8a2d92712d26919359b847d1b814c0bfca5e37118124  fixtures/scripts/main_after_wrapper.sh
1394eda5f6bcc2ae62a747efbc02b513e2a1a285507ba397bad04035eed80129  fixtures/scripts/wrap.sh
521f30fa023f3782b5db00695bf0f8f095d6f9585478c6698d0c1a5739a911f1  tools/entries.txt
d8e261c9841348f37b560c08fa6fb1c721a87df7944fd940c40b330e4ae77c62  tools/entries_scan.sh
16899d9dfbcfdd030b6d915c606417ceaebf6620e2db91fc694dbb1ffaf2e831  out/entries-scan-current.out
2e0b307eae84a96d6bf42d188b5b3e06d4b4ebb21f908dae107920280aa4a233  tools/inline_refusals.sh
18dc48d64db0dd6a56d95cc9d055a305ad7baa3d482bb06251bfc0a7393db660  out/inline-refusals.out
```

`rerun.sh` 的自证（`--selftest`）没跑：模板自证核的是模板自带的 `rerun_steps`，这里换了。

## 1 V1：丙排掉的那条路上，有没有会改判定结局的东西

**结论**：分两层答。
内容层（Verus 判出什么、判绿判红、写进判定库什么）：那条路上没有任何一处能改。读的，有出处，见下表。
结局层（这一次判定产不产出）：正常机器状态下不改；有两处在主线程里没兜住的异常，线程数或临时空间耗尽时，会把一次本来会排上队跑起来的判定变成包装退 251，或把应当退 252 的变成 251。再加上退 252 之前最多多等约两个查询时限。这两样都是「这一次判定没产出」，不是「判出另一个结局」，不会让库里多出一条错的绿；但题面和包装文件头那句「只打提示、不改结局」在这两处不成立。

**那条路是什么**（读的）：`HostLookup.query` 在包装的内嵌 python 里，用 `subprocess.Popen` 起 `host-resources.py`，输出落临时文件，到点只杀自己起的那一个进程：

- 「process = self.process = subprocess.Popen([sys.executable, "-B", script, "--json", "--", *self.command], stdout=output, stderr=errors,」（`scripts/hosts/run-with-memory-cap.sh:1012`）
- 包装文件头自己的定位：「不改排队本身：不自动改去第二台（两台结论口径不一样时混用分不清），退出码照旧。」（`scripts/hosts/run-with-memory-cap.sh:115`）

**这条路的全部出口**（读的）：

| 出口 | 去哪 | 能不能碰判定内容 | 出处 |
|---|---|---|---|
| 提示行 | 包装的标准错误；`say` 只写 `sys.stderr` | 不能：判法读 Verus 的 JSON 取自标准输出 | 「outcome["json"] = json.loads(stdout) if stdout.strip().startswith("{") else None」（`scripts/gate/verus-proof-check.py:733`）；「def say(text):」（`scripts/hosts/run-with-memory-cap.sh:915`） |
| 告警记录 | `core_usage.append_line` 追加到告警文件 | 不能：不是判定库，只有看门狗读 | `HostLookup.record`（`scripts/hosts/run-with-memory-cap.sh:1065` 起） |
| 节流文件 `host-lookup.json` | 包装状态目录 | 不能：只有 `take_turn` 读写 | 「def take_turn(self):」（`scripts/hosts/run-with-memory-cap.sh:963`） |
| 时间 | `finish` 在退 252 之前前台查：先等排队开始那一趟查完，再查一趟，每趟至多 `HOST_LOOKUP_SECONDS` | 不能：退出码仍是 252 | 「HOST_LOOKUP_SECONDS = 15.0」（`scripts/hosts/run-with-memory-cap.sh:775`）；「lookup.finish(refused)」（`scripts/hosts/run-with-memory-cap.sh:1151`） |
| 本机排队锁 | `take_turn` 在线程里拿 `admission_lock()` 的 flock | 不能：只是和准入抢同一把锁，延迟 | `scripts/hosts/run-with-memory-cap.sh:808-817` |
| 被起的 `host-resources.py` 本身 | 只读 /proc、ledger、配置，经 ssh 读第二台 | 不能：非自证路径里没有写文件（`open(…, "w")` 的命中行全在 `SelftestWorld`（625 行起）与自证格（748 行起）里） | `grep -nE 'open\(.*"(w|a)"' scripts/hosts/host-resources.py` 命中 665、667、669、679、681、685、691、702、706、790、799、837 行，`main` 在 594-624 行、没有写（读的） |

**和 Verus 的标准错误会不会串行**（读的，推的一半）：放行后 `cancel` 先停查询、`join` 线程，再返回；shell 侧同步调 `admission admit` 拿到状态才起 scope：「admit_status = admit(unit, int(cap_bytes), cap_text, int(total_bytes), slice_directory, key, int(wrapper_pid), host_lookup)」（`scripts/hosts/run-with-memory-cap.sh:1356`）、「host_lookup.cancel(admitted=(admit_status == 0))」（`scripts/hosts/run-with-memory-cap.sh:1358`）。所以提示行不会插进 Verus 一行输出的中间；判法按行认的「error」起头（`scripts/gate/verus-proof-check.py:745`、`scripts/gate/verus-proof-check.py:762`）碰不到提示行（提示行以 `run-with-memory-cap:`、`  →`、`  !` 起头）。

**提示行会改的只有红句的文字**（读的）：判法从标准错误里挑以 ✗ 或 → 起头的行带进「没跑成」的红句：「return [line.strip() for line in outcome.get("stderr", "").split("\n") if line.strip().startswith(("✗", "→"))]」（`scripts/gate/verus-proof-check.py:803`）。查别的机器放得下时打的那一行恰是 `  → 第二台此刻放得下…`：「line = f"  → {peer['name']}此刻放得下（可用 {human(peer['available'])}、要 {human(peer['need'])}）：改用 {command}{cap_note}"」（`scripts/hosts/run-with-memory-cap.sh:1060`）。它排在包装自己的 ✗ 行前面时，`QueueRefusal.record` 取 `own[0]` 写进红句：「own = wrapper_own_lines(outcome)」（`scripts/gate/verus-proof-check.py:689`），「包装说：{own[0][:200]}」（`scripts/gate/verus-proof-check.py:690`）。只改红句文字，不改红绿。

**判定库里存什么**（读的）：「"verdict": "green", "summary": summary, "details": dict(details or {}), "run": run,」（`scripts/crash/lib_verdict_library.py:475`），summary 与 details 来自 Verus 的 JSON：「return [], f"{results['verified']} verified、0 errors", {"verified": str(results["verified"])}」（`scripts/gate/verus-proof-check.py:1353`）。标准错误一个字不进库；只记绿，没跑成的不记。

**结局层里没兜住的两处**（读的加注入量的）：

- 排队开始那一刻 `lookup.begin` 起线程，没有 try：「self.thread.start()」（`scripts/hosts/run-with-memory-cap.sh:940`），调用处在排队主循环里：「lookup.begin({"need": need, "slice_occupied": occupied, "slice_total": total_bytes})」（`scripts/hosts/run-with-memory-cap.sh:1170`）。
- 退 252 之前 `lookup.finish` 在主线程里前台查，查的路径上建临时文件也没有 try：「with tempfile.TemporaryFile(mode="w+", encoding="utf-8") as output, tempfile.TemporaryFile(mode="w+", encoding="utf-8") as errors:」（`scripts/hosts/run-with-memory-cap.sh:1007`）。
- 异常从 `admit` 冒出去，`finally` 里 `cancel` 收拾完，python 退 1；shell 侧不是 0 也不是 252 的一律按起不了 scope 处理：「report_unavailable "排队那一步退 $admission_status（上面是它的报错），命令一行都没跑" "照上面那几行修；不许绕开排队去跑"」（`scripts/hosts/run-with-memory-cap.sh:1535`），退出码是「MEMORY_CAP_UNAVAILABLE_EXIT=251」（`scripts/hosts/run-with-memory-cap.sh:432`）。251 与 252 都在判法认的包装自己的结局里：「WRAPPER_OWN_EXITS = {245: "显存不够（起止之间内核报了 NV_ERR_NO_MEMORY）", 250: "撞了这一条自己的内存上限", 251: "包装用法错或起不了 scope", 252: "内存不够排不上",」（`scripts/gate/verus-proof-check.py:344`），判法记「没跑成」、不记库。
- 探针 `tools/exception_probe.py` 把包装里 `admission()` 的内嵌 python 原样载入，往这两处注入「起不了线程」「临时文件写不下」（`out/rerun-1.out` 末三行，原样）：

```
begin（起线程失败，主线程里）: 异常冒出来 RuntimeError: can't start new thread
finish（临时文件建不了，退 252 之前前台查，主线程里）: 异常冒出来 OSError: [Errno 28] No space left on device
begin（临时文件建不了，后台线程里查）: 没有异常冒出来
```

末一行是对照：同一个故障落在排队开始那一趟的后台线程里，线程自己死掉，不冒到主线程（Python 打回溯到标准错误）。注入的是条件，不是真把线程或磁盘用光；从异常到 251 的那一段是读代码推的，没有拿真包装端到端跑（真包装要起 systemd scope 与本机内存排队，这一轮没起）。自然触发有多容易，没量。

**哪种机器状态会改结局**（上面三样的归拢）：

| 机器状态 | 改不改包装的结局 | 依据 |
|---|---|---|
| 第二台连得上、连不上、放得下、放不下、查超时 | 不改：都是 252 或照常放行，命令不自动改去第二台 | 包装自证里现成的六格 `host_lookup_cells`（`scripts/hosts/run-with-memory-cap.sh:1776` 起）六格都断言退出码，其中放得下与不拖慢准入两格另断言命令跑没跑（`scripts/hosts/run-with-memory-cap.sh:1784` 起，`scripts/hosts/run-with-memory-cap.sh:1850` 起）；读的，没跑 |
| 本机起不了新线程（进程数或线程数撞上限），发生在第一次报排队那一刻 | 改：本来排队等到放行再跑，变成退 251 | 读加注入 |
| 本机临时目录写满或文件描述符用尽，发生在要退 252 的那一刻 | 改：252 变 251 | 读加注入 |
| 本机排队锁被别的包装占着 | 只延迟：线程里的 `take_turn` 与准入抢同一把 flock | `scripts/hosts/run-with-memory-cap.sh:808-817` |
| 第二台 ssh 慢、本机 nvidia-smi 慢 | 只多等：每趟至多 15 秒，退 252 前最多两趟 | `scripts/hosts/run-with-memory-cap.sh:775`、`scripts/hosts/run-with-memory-cap.sh:949` |

**「只打提示」今天钉得住多少**：包装自证有六格（`RUN_WITH_MEMORY_CAP_SELFTEST_ONLY=host-lookup` 只跑它们）和五个弄坏开关（`nohostlookup`、`hostlookupnorecord`、`hostlookupblocking`、`hostlookupnotimeout`、`hostlookupunthrottled`），断言的是：放得下时打两行提示、写两条告警、命令不自动改去第二台、不拖慢准入、几条一起排队时 60 秒只查一次。没钉的：标准输出一个字不多；查别的机器那一支自己出异常时退出码不变；判法一侧读到提示行判定不变。弄坏开关怎么设、补哪几格见 V4。

**推翻 V1 的现象**：
- 在同一个假的第二台世界里，同一条命令分别设 `RUN_WITH_MEMORY_CAP_HOST_LOOKUP=off` 与不设，退出码、标准输出字节或「命令跑没跑」有一样不同（这是现成自证世界就能做的差分，这一轮没做）；
- 找到 `say` 的一条提示里带换行、且换行后的一行以 `error` 起头（`problem` 来自 `host-resources.py` 的单行文字与异常的 `str`，读下来没有换行）；
- 找到查别的机器那一支写标准输出，或写判定库目录的一处。

## 2 V2：排除规则怎么写才不越界

**结论**：按「调用边」排，且只排一条：调用方单元 `HostLookup.query` 起 `scripts/hosts/host-resources.py` 这一条边。调用方自己的单元（`HostLookup` 各方法、`admit`、`predicted_bytes` 与包装其余部分）照进键，被调方只要经别的边进闭包也照进。不按文件排，不按函数整个排，不按包装的开关排。

**量的**（`out/rerun-1.out` 与 `out/unit-diff.txt`，冻结副本上闭包原样）：

```
RESULT tree=current entry=scripts/gate/verus-proof-check.py::check_proof units=1579 refused=[inline-script-not-literal scripts/hosts/peer-host-lib.sh:76;]
RESULT tree=current entry=scripts/gate/verus-proof-check.py::check_vacuity units=1567 refused=[inline-script-not-literal scripts/hosts/peer-host-lib.sh:76;]
RESULT tree=current entry=scripts/gate/verus-proof-check.py::check_must_red_worlds units=1575 refused=[inline-script-not-literal scripts/hosts/peer-host-lib.sh:76;]
RESULT tree=cut-lookup entry=scripts/gate/verus-proof-check.py::check_proof units=1232 refused=[]
RESULT tree=cut-lookup entry=scripts/gate/verus-proof-check.py::check_vacuity units=1220 refused=[]
RESULT tree=cut-lookup entry=scripts/gate/verus-proof-check.py::check_must_red_worlds units=1228 refused=[]
```

切边只换了一处（`HostLookup.query` 里那个 `Popen` 换成 `None`，`tools/cut_lookup_edge.py`）。切完三个入口都不再拒算，单元少 347 个，没有新增一个单元（`--explain` 对照，`added=[]`）。少的单元按文件：

```
单元合计 现状 1579、切边之后 1232
scripts/hosts/host-resources.py: 现状 75、切边之后 0、少 75
scripts/hosts/host-dispatch.py: 现状 67、切边之后 26、少 41
scripts/hosts/memory_peaks_lib.py: 现状 40、切边之后 31、少 9
scripts/hosts/peer-host-lib.sh: 现状 5、切边之后 0、少 5
scripts/crash/crash-amplification-lib.sh: 现状 26、切边之后 0、少 26
scripts/crash/crash-point-replay-shard-configuration-check.sh: 现状 20、切边之后 0、少 20
scripts/hosts/rocksdb-prebuilt.py: 现状 101、切边之后 0、少 101
.claude/hooks/lib_long_run.py: 现状 40、切边之后 0、少 40
.claude/hooks/lib_heavy_tests.py: 现状 18、切边之后 0、少 18
（另有 lib_shell_words.py 5、preflight.py 3、lib_work_root.py 1、project_preflight.py 1、proc.py 1、lib_function_units.py 1，同在 out/unit-diff.txt）
```

读法：整文件离开闭包的都是只经 `host-resources.py` 进来的；`host-dispatch.py` 与 `memory_peaks_lib.py` 只少一部分，留下的是包装准入自己用的。`predicted_bytes` 在内嵌 python 里按路径载入 `host-dispatch.py` 的 `predict` 与 `settings_from_environment` 算这一条要多少内存（`scripts/hosts/run-with-memory-cap.sh:851` 起，闭包里的 `via` 行 `host-dispatch.py:predict … loaded predict`），这决定排得上还是 252，必须留在键里。

**三种写法逐个看**：

- 按文件排：做不到不越界。`host-dispatch.py` 要么整份排（连 `predict` 一起丢，准入的「要多少」不再进键），要么不排（`judge_host` 等 41 个单元留着，拒算虽然没了但键里带着只给提示用的东西）。`memory_peaks_lib.py` 同理（31 个准入用的单元）。今天已有的文件级排除更糟：`scripts/crash/lib_verdict_library.py` 的 `FunctionUnitIndex.key` 排掉文件里的单元时连拒算也一起丢：「merged.refusals.update(refusal for refusal in closure.refusals if refusal.file not in excluded_files)」（`scripts/crash/lib_verdict_library.py:205`），被排文件里的拒算被静默抹掉，与已定项 17 的「不静默少收」相冲（见下）。这个文件级排除今天只用在两份算键的库上：「excluded = () if "judging-files-included" in BREAK else NOT_JUDGING_FILES」（`scripts/crash/lib_verdict_library.py:223`）。
- 按函数排（排掉 `HostLookup` 整个类）：越界。能把一次判定变成 251 的恰是 `begin`、`finish`、`cancel` 三个方法体（上节），排掉它们，包装的退出码语义改了键不变。
- 按包装的开关排（`RUN_WITH_MEMORY_CAP_HOST_LOOKUP` 非空就不查）：闭包不带路径条件，这个开关在 `query` 里是运行期读环境：「switch = os.environ.get("RUN_WITH_MEMORY_CAP_HOST_LOOKUP", "")」（`scripts/hosts/run-with-memory-cap.sh:990`），闭包算法静态判不了；要靠调用方（`run_verus`，`scripts/gate/verus-proof-check.py:720` 起的命令与环境）每次设上开关才成立，那就等于关掉提示，与丙「保留提示」相反。

已定项 17 的相关两句：

- 「收不全（动态分派、宏、`include_str!(concat!…)` 认不出）的那一份拒算键、照跑，不静默少收；每把键连同组成它的单元清单落盘，作废时报得出是哪几个单元变了。」（`.claude/kb/decisions/13-验证路线.md:434`）
- 「键过窄（引用的常量、类型、宏漏收）、库放在会丢的位置、改名丢库、两份库重复判同一批，同样算违反。」（`.claude/kb/decisions/13-验证路线.md:433`）

按边排的写法与这两句的关系：被排的边不是「认不出」，是认出来了、判定它不依赖；所以它不属「拒算」，但属「键过窄」的反面，要有一个写下来的依据，才不是静默少收。依据就是 V1（内容层不碰、结局层至多变成没跑成）加每条排除各有一格会红的自证（V4）。

**别的经内存包装起的判定入口，逐个看**（量的，`tools/entries_scan.sh`、`out/entries-scan-current.out`）：登记的 python / shell 判法入口取了 35 个（广谱变异 `JUDGING_ENTRIES` 与 `GPU_JUDGING_ENTRIES`、变异表驱动 `JUDGING_ENTRIES`、复跑判法函数三个、`e165-pin-red-examples.py` 的 `replay`、形式化三个；清单 `tools/entries.txt`），在冻结副本上各收一次闭包：

- 闭包里没有内存包装的单元：31 个（`wrapper_units=0`，其中含 `replay.sh` 三个判法函数、变异表驱动六个、广谱变异 GPU 七个）。
- 带着内存包装 88 个单元、`host-resources.py` 75 个单元、各拒算一处 `peer-host-lib.sh:76` 的：4 个——形式化三个，和广谱变异的 `Runner.escalate_step`（「("scripts/mutation/broad-mutation.py", "Runner.escalate_step")」，`scripts/mutation/broad-mutation.py:530`）。
- `Runner.escalate_step` 照广谱变异自己的扫描范围收（三份文件，`rerun.sh` 里带 `--under`）：现状 792 个单元、拒算 `peer-host-lib.sh:76`；切边之后 436 个、不拒算（`out/rerun-1.out`）。
- 登记表以外的判法（崩溃状态块、QEMU 每档、litmus、研究复跑每行里走 Rust 闭包的那些）这一轮没逐个收：Rust 闭包不经 shell 包装；`vm-bench.sh`、`replay_judge_fresh_product`、QEMU 的 `judge_mode_*` 单独收过，闭包里没有包装；`lkmm.sh` 拒算 5 处、与包装无关。

广谱变异的判法单元指纹带拒算也出键：「key, refusals = library.closure_key(closure, allow_refused=True)」（`scripts/mutation/broad-mutation.py:1465`）。所以丙对它的后果是键变一次（792 变 436 个单元），旧库按已定项 17 「改键让旧库作废不算代价」（`.claude/kb/decisions/13-验证路线.md:436`）整库作废一次；它是不是算得上已定项 17 要的「拒算键、照跑」（今天它带着拒算照样出键），这一轮不判，写进判定交主 agent。`broad-mutation.py --selftest` 与 `judging_units_key` 本体这一轮没跑，只用 `closure` 命令行量了闭包。

**排除声明写在哪**（推的）：键里的注释数据单元把 shell 文件里任何「# <词>:」的注释都收进去：「KEPT_SHELL_COMMENT = re.compile(r"(?:^|[\s;&|()])(#\s*(?:[A-Za-z][A-Za-z0-9_-]*:|shellcheck\b).*)$")」（`scripts/gate/lib_function_units.py:5215`）。所以把声明写成包装文件里 `admission()` 上方的一行 `# closure-excluded-edge: <调用方单元> -> <被调文件>：<为什么>`，声明自己就进键：改了、删了键就变，不会出现「排除表悄悄加一行、键不变」。边按（调用方单元限定名，被调文件）认，只切这一条边；声明指到不存在的边时要拒算（另起一个拒算形态名），免得边没了声明还留着。这一段闭包算法没有现成的，是新加的；怎么实现不在这一轮。

**推翻 V2 的现象**：
- 找到一个登记的判法入口，它的闭包里有只经 `HostLookup.query` 这一条边才进来、而又确实影响判定内容的单元（`out/unit-diff.txt` 列的少掉的单元，逐个查有没有别的引用方；闭包算法按名字过近似，别的引用方没有就是没有，动态分派它是拒算不是静默少收）；
- 找到 `host-resources.py` 非自证路径上写出会被准入读的状态（峰值表、账）：那样它改的就是「要多少内存」，进的是准入的结局，边就不能整条排。

## 3 V3：甲之后键怎么算第二台那一侧

**结论**：
- 键里不放第二台那一侧，也放不了：键是本机此刻的源码、工具链与依赖算出来的，第二台的仓版本本机算不出来；同一份脚本的单元，本机闭包里到得了的已经在键里。第二台那头版本不同怎么办，是用它的那个消费者的事，不是键的事：只打提示的消费者（`HostLookup`）版本对不上就打「没查到别的机器」并停，不回退到送程序；把第二台判出的记录并回本机库的消费者照现有做法——两台各算判法单元指纹比相等、导入时按本机此刻源码现算的键逐条核。
- 甲「要求第二台有同版本的仓」与第二台今天的样子不合：第二台没有常驻的仓。
- 判法学会「函数里 `bash -c "$N"` 的串算调用处给的」：今天仓里这种函数有两处（`peer_host_run` 与 `multi-host-run.sh` 自证里的 `start_shared_driver`，grep 命中，见下），已量的 35 个登记判法入口里，丙一做就没有一个再拒算，甲这一条不是它们的前提；它做对的条件有五条（下面），其中三条今天的引擎里没有现成的机制（或只有本机树副本那一种）。

**第二台的仓**（读的）：
- 「# 第二台上放这一趟树副本的专用目录（绝对路径）：每一趟在它下面建一个子目录放树与编译目录、跑完删掉；」（`multi-host.env.example:11`）
- 「④ 树拷到第二台 <PEER_REPOSITORY_DIRECTORY>/runs/<标签>/（新建目录，标签已在就拒、退 2）：拷的是 git 列得出的文件」（`scripts/hosts/multi-host-run.sh:18`）
- 「第二台那一趟的树连同库跑完就删，不留长期库」（`.claude/rules/implementation-workflow.md:66`）

所以「第二台有同版本的仓」今天不存在：要么新立一份常驻副本（版本漂移是常态，要维护同步），要么每次查别的机器前先拷树（排队等内存的那一刻要在 15 秒里出结果，`scripts/hosts/run-with-memory-cap.sh:775`）。现在的 base64 送程序恰是为了第二台不必有仓：「# ── 每台上跑的那一段：本机直接调；别的台把这个函数的源码连同 memory_peaks_lib.py 送过去调（所以只用标准库、不引用本文件别处的名字）──」（`scripts/hosts/host-resources.py:133`）。

**键与第二台的现有关系**（读的）：
- 「第二台判出的并进本机那一份」（`.claude/kb/decisions/13-验证路线.md:435`）：第二台判出的记录按本机的键存。
- 「按本机此刻源码现算的行键与判法单元指纹逐条核过才并进本机那一份判定库」（`.claude/rules/implementation-workflow.md:66`）。
- 两台各算判法单元指纹、不等就停：`peer_base="$(peer_host_run "$peer_run_directory" "python3 scripts/mutation/crates-mutation-rows.py judging-units .")" \`（`scripts/mutation/mutation-shard-run.sh:371`）、`[[ "$local_base" == "$peer_base" ]] \`（`scripts/mutation/mutation-shard-run.sh:375`）。这里第二台跑的就是「同一份脚本、参数选机器」的形态，版本由「每趟现拷的树」与「比相等」两样管，不进键。

**折法要做对的条件**（读的加今天引擎的量）：

今天引擎处理函数里位置参数的两种情形（`tools/fixture_closure.sh`、`fixtures/scripts/`，量的，`out/fixture-closure.out`）：

```
== scripts/main.sh            run_step() { "$@"; } 调用处给命令词      → inner.sh 进闭包
== scripts/main_callback.sh   同上，调用处第一个词不是命令            → inner.sh 进闭包
== scripts/main_text.sh       run_text() { bash -c "$2"; } 调用处给整串字面 → refused inline-script-not-literal scripts/main_text.sh:3
== scripts/main_after_wrapper.sh  start_driver() { bash scripts/wrap.sh --peer -- bash -c "$3"; } 调用处给整串字面 → refused inline-script-not-literal scripts/main_after_wrapper.sh:3
```

即 `"$@"` 当命令那条靠「命令就是调用处那几个词」（`scripts/gate/lib_function_units.py:140`，实现 `scripts/gate/lib_function_units.py:6772` 一带），调用处的词由调用处按命令认；`bash -c "$2"` 要的是调用处给的整串当内联脚本认，引擎今天没有这一条。python 一侧已有同类的折法：「函数参数（本模块每一处调用给的实参）」（`scripts/gate/lib_function_units.py:142`）。

仓里每一处 `peer_host_run` 调用的串长什么样（人工逐行归类，`grep -rnE '(^|[^_a-z])peer_host_run +' scripts --include=*.sh --include=*.py` 命中 21 处真调用加 2 处文档行；归类按读到的那一行）：

| 形状 | 处数 | 位置 |
|---|---|---|
| 整串字面（或字面赋值的局部变量） | 6 | `peer-host-lib.sh:107`、`multi-host-run.sh:449`、`mutation-shard-run.sh:364`、`mutation-shard-run.sh:371`、`mutation-shard-run.sh:381`、`mutation-shard-run.sh:468` |
| 字面加 `$(printf '%q' 变量)` 当数据词 | 10 | `multi-host-run.sh:215`、`multi-host-run.sh:271`、`multi-host-run.sh:273`、`multi-host-run.sh:310`、`multi-host-run.sh:396`、`multi-host-run.sh:399`、`mutation-shard-run.sh:215`、`mutation-shard-run.sh:216`、`mutation-shard-run.sh:357`、`peer-host-lib.sh:295` |
| 串里套 `declare -f` 的函数正文、函数输出、再套一层 `bash -c` | 3 | `peer-host-lib.sh:287`、`peer-host-lib.sh:318`、`mutation-shard-run.sh:326` |
| python 经 `bash -c` 转进来的（跨两种语言、两跳位置参数） | 2 | `host-resources.py:314`、`host-dispatch.py:316` |

所以只认「整串字面」的折法覆盖 6 处；要覆盖带 `$(printf '%q' …)` 的 10 处就得认「数据词位置」（变量值只落在参数位、不落在命令位或 `bash -c` 的串里），这一条引擎今天也没有；剩下 5 处不该折，应当继续拒算。

甲要折对，五个条件缺一个都会错折，逐条配仓里的例子：

- 只认所在函数的位置参数 `$1`…`$9`：不认 `$变量`、`$*`、数组下标。`peer-host-lib.sh` 里 `bash -c "$QUIESCE_STOP_COMMAND"` 的值来自运行期读的多机配置，折不出，必须继续拒算：「bash -c "$QUIESCE_STOP_COMMAND" >/dev/null 2>&1」（`scripts/hosts/peer-host-lib.sh:135`）、「if bash -c "$QUIESCE_STOPPED_CHECK_COMMAND" >/dev/null 2>&1; then」（`scripts/hosts/peer-host-lib.sh:136`），今天就是拒算（`out/inline-refusals.out`，shell 一侧 37 处）。`grep -rnE '(bash|sh|python3?) +(-[A-Za-z]+ +)*-c +"\$[0-9@*{]'` 在 `scripts`、`.claude/gate.d`、`.claude/hooks`、`.claude/scripts` 下命中的函数位置参数写法只有两处：「PATH="$PEER_CARGO_BIN_DIRECTORY:$PATH" bash -c "$2") 2> >(peer_host_redact >&2) | cat」（`scripts/hosts/peer-host-lib.sh:81`）与 `multi-host-run.sh` 自证里 `start_shared_driver` 的 `bash -c "$3"`（`scripts/hosts/multi-host-run.sh:999`，四处调用都是字面加数据词）。shell 拒算行的行号是回到原文按函数头找的索引，不是 `bash -c` 那一行本身（`out/inline-refusals.out` 里 `scripts/experiment-apparatus/replay.sh:1220` 是一行 `echo`）：「起止行是回到原文按函数头找的，只当索引。」（`scripts/gate/lib_function_units.py:66`）。
- 函数在闭包里的每一处引用都得是直接调用、且调用处的串折得出：有一处不是（`declare -f` 把函数正文带走、按名字传给别的命令），或折不出，就在函数处拒算。引擎已认按名字引用函数：「"前缀_$1"（按名字前缀连看得见的函数）」（`scripts/gate/lib_function_units.py:140`），所以「还有没有别的引用」今天问得出，但折法要在这上面加一条「全是直接调用」的判。今天没有现成的。
- 数据词只许落在参数位：`$(printf '%q' 变量)` 落在 `bash -c`、`eval` 或命令位时变量值是代码，不是数据。`peer-host-lib.sh:287` 的 `setsid nohup bash -c $(printf '%q' "$detached_text")` 就是这一种，必须不折。今天没有现成的。
- 串里相对脚本路径按调用处给的目录认：`peer_host_run / "…"` 的目录是 `/`，`peer_host_run "$peer_run_directory" "python3 scripts/mutation/crates-mutation-rows.py judging-units ."` 的目录是第二台上这一趟现拷的树。引擎今天只认本机的树副本（转述：折不出的目录加根里存在的相对路径，临时树、仓副本里的同一份，按根里那一份认；弄坏开关 `tree-copy-tails-ignored` 关掉的就是这一条，`scripts/gate/lib_function_units.py:173`）。把第二台那棵树认成「本机被判的根」是一个假定，不是核过的事实；认不成的（目录是 `/` 或折不出）相对路径应当拒算。
- 跨语言几跳都要折得出：`host-resources.py` 经 python 的 `bash -c runner` 传 `$5`、再进 `peer_host_run` 的 `$2`：「'peer_host_run / "$5"')」（`scripts/hosts/host-resources.py:314`）；`host-dispatch.py` 同理：「runner = 'source "$1" || exit 70; eval "$2"; peer_host_run / "$3"'」（`scripts/hosts/host-dispatch.py:316`）。python 一侧的 `text` 是函数参数，引擎认「函数参数（本模块每一处调用给的实参）」（`scripts/gate/lib_function_units.py:142`），但 `probe_machine` 给的是含 base64 的 f 串：「program = base64.b64encode(remote_probe_program().encode("utf-8")).decode("ascii")」（`scripts/hosts/host-resources.py:329`），折不出。甲改成字面的 `python3 scripts/hosts/host-resources.py …` 之后这一跳才折得开；`host-dispatch.py:316` 的 `$2`（`self.declarations`，`declare -p` 的输出）与 `$3` 是运行期值，不会折开，应当继续拒算（`Peer.run` 不在形式化三个入口的闭包里：冻结副本上 `closure --explain` 的输出里没有它，输出在草稿目录、没入库）。

**哪些会被折开、哪些会被错折**：条件全满足时，今天被折开的函数位置参数 `-c` 只有 `peer_host_run` 与 `start_shared_driver` 两个函数（`peer_host_run` 的 21 处调用里，6 处整串字面的能折，10 处字面加数据词的按第三条判能折，5 处要继续拒算）；条件缺「全是直接调用」或「数据词位置」任一条，`peer-host-lib.sh:287`、`318` 与 `mutation-shard-run.sh:326` 会被错折成绿。python 一侧 16 处拒算（`out/inline-refusals.out`）不受这条影响：python 一侧是 `["bash", "-c", 变量]` 的参数折法，规则在 `scripts/gate/lib_function_units.py:142`。

**推翻 V3 的现象**：
- 找到第二台上常驻着同版本仓的现成做法（配置里、脚本里、记录里任一处），那「同版本的仓不存在」这一句就不成立；
- 找到一个已登记的判法入口，闭包里 `peer_host_run` 的调用处折开后才不拒算、而丙不能代它解决（这一轮量的 35 个里没有）；
- 找到一处消费第二台判定又不核版本的：那说明「版本归消费者管」少了一个消费者，不是键的事。

## 4 V4：条文与自证

**D13 已定项 17 要加一条**（逐字候选，插在 `.claude/kb/decisions/13-验证路线.md:434` 那一条「收不全……」之后；三语文件同一条各加：`13-verification-route.en.md` 的 Settled Item 17、`13-検証ルート.ja.md` 的 確定項目 17，同一位置）：

> - 沿调用与引用收闭包时，一条调用边同时满足下面三样才许排除：被调方是调用方单元另起的一个程序，它的输出只进调用方的提示行与告警记录，不进判定的退出码、标准输出与判定库的内容，调用方对它的失败、超时与读不出的输出各有兜底；排除写在调用方源码的注释里（`# closure-excluded-edge: <调用方单元> -> <被调文件>：<为什么>`），声明进键，改了或删了键就变，指到不存在的边拒算；这条边有一格自证钉住：改被调方键不变、改调用方自己的单元键变、被调方经别的边进闭包时照进。排除只落在这一条边上：调用方单元、被调方经别的边进来的单元与闭包里别处的拒算照旧进键、照旧拒算。不按文件排、不按函数整个排、不按调用方的环境开关排。

该条「依据」一节（`.claude/kb/decisions/13-验证路线.md:446` 「没有腿况记录：未经三方攻过」那一条旁）要另写：排除条款是这一轮正推一条腿推出的、被攻过零轮。

与已定项 17 现有条文的关系：不冲突，是在「收不全拒算」（434）与「键过窄算违反」（433）之间补一条「认出来了、判定不依赖」的出路，并要求它自带依据与自证。已有的文件级排除（`NOT_JUDGING_FILES`，两份算键的库）条文里没写，这一轮不动它，但新条文「不按文件排」与它字面上冲突，要么新条文加一句「算键的库自己除外」，要么另立一题；这是主 agent 要选的岔路。

**`.claude/rules/implementation-workflow.md` 那一节**（标题「复用重型门禁的判定：按每条判定的单元键判，不按「改了哪个目录」或「哪棵树绿过」判」，`.claude/rules/implementation-workflow.md:29`）在第 33 行那一段之后加一段（逐字候选）。两份候选取决于 `HostLookup` 的 `begin`、`finish` 补不补兜底：

> 候选一（不补兜底）：闭包里有一类调用边按 D13（验证路线） 已定项 17 的排除条款排在闭包之外：内存包装 `scripts/hosts/run-with-memory-cap.sh` 排队时查别的机器那一支（`HostLookup.query` 起 `scripts/hosts/host-resources.py`）。排它是因为那一支只往标准错误打提示、往告警记录追加一行，不碰标准输出与判定库，判定内容不受它影响；它自己出异常（起不了线程、临时文件写不下）时包装至多退 251、这一次判定没跑成，不会判出另一个结局。包装自己的单元（含 `HostLookup` 各方法）照进键。排除登记在调用方源码的 `# closure-excluded-edge:` 注释里，不另立表。

> 候选二（`begin`、`finish` 补了兜底）：同上，把「它自己出异常……不会判出另一个结局」换成「它自己出异常只打一行说明、包装的退出码照旧」。

要求（推的）：候选二的话，那句「退出码照旧」才与 `scripts/hosts/run-with-memory-cap.sh:115` 的文件头一致；候选一不改代码，但文件头那句「退出码照旧」与 251 的路径要在同一次改里改成「查别的机器那一支自己出异常时除外」。

**丙之外，甲的条文**：折法不改已定项 17 与这一节（它仍在「收不全拒算」之内，只是更多写法折得出）；要写的是 `scripts/gate/lib_function_units.py` 文件头「起代码的写法逐种认」一段的一条（逐字候选）：

> -c 的串是所在函数的位置参数 $N（N 为 1 到 9）时：函数在闭包里的每一处引用都是直接调用、且每一处调用给的串折得出，就在各调用处当内联脚本认，相对路径按调用处给的目录认；有一处不是、折不出、或目录折不成一棵已知的树，在函数处拒算 inline-script-not-literal。$变量、$*、数组下标不在此列。

按这一轮量到的，丙一做，登记的判法入口不再需要它，所以甲的折法不是丙的前提；写不写、什么时候写，由主 agent 定，这里只给条件。

**自证要加什么**（推的；下面的名字都是提议，仓里没有；四份脚本的自证今天都登记为 light，`.claude/hooks/selftest-registry.tsv` 的 32、127、146、148 行）：

丙：
- `scripts/gate/lib_function_units.py --selftest`：五格 fixture（一份调用方加一份被调程序）——改被调方键不变；改调用方自己的单元键变；被调方另有一条边进闭包，改它键变；声明改了或删了键变；声明指到不存在的边拒算。弄坏开关：`closure-excluded-edge-ignored`（声明不生效，第一格红）、`closure-excluded-edge-drops-file`（按文件排，第三格红）、`closure-exclusion-declaration-outside-key`（声明不进键，第四格红）、`closure-exclusion-stale-allowed`（第五格红）。
- `scripts/hosts/run-with-memory-cap.sh --selftest`（`host-lookup` 那一组，现有六格在 `scripts/hosts/run-with-memory-cap.sh:1776` 起）：加「查别的机器开着与关着差分」一格——假的第二台世界放得下、放不下、连不上、超时四种，每种开关两趟，退出码、标准输出字节、命令跑没跑逐对相等；弄坏开关 `hostlookupstdout`（提示改写标准输出，红）。候选二才加「查别的机器那一支出异常退出码不变」一格，需要一个只给自证的注入变量（例如 `RUN_WITH_MEMORY_CAP_SELFTEST_LOOKUP_FAULT=thread|tempfile`），弄坏开关 `hostlookupunguarded`（去掉兜底，红）；候选一把这一格写成「退 251、命令没跑」。
- `scripts/gate/verus-proof-check.py --selftest`：（一）「假包装先打查别的机器会打的两种行再起假 Verus」：判定与库里那一份记录的字节同没有提示行的一趟，弄坏开关 `wrapper-hint-as-proof-error`（把以 → 起头的行当证明错，红）；（二）真改动的键格：拷树改真单元，改 `host-resources.py` 里一个函数三个入口的键不变，改 `HostLookup.query` 或 `run_verus` 键变。今天 `reuse-judging-units-changed` 靠传一个造出来的判法单元指纹：「got, output, _verdicts = run_items("must-red-worlds", reuse_root, verdict_library=store_directory, judging_units="自证造的另一份判法单元")」（`scripts/gate/verus-proof-check.py:2803`），看不出真改动，一个被错排的单元不会让它红。
- `scripts/mutation/broad-mutation.py --selftest`：同样的真改动键格给 `Runner.escalate_step`。理由：它带拒算也出键（`scripts/mutation/broad-mutation.py:1465`），今天没有一格会在它的键变化时红。

门禁 code-tooling 的 reuse-selftests 格（`cell_reuse_selftests() {`，`.claude/gate.d/code-tooling.sh:3560`）：
- 格函数只把 `lib_reuse_checks.py audit` 交出去，audit 认的是每份登记脚本自证末尾那一行规范输出（`scripts/gate/lib_reuse_checks.py` 文件头「一」「四」）。丙的钉子要放进各脚本自证三问的驱动里——第二问「闭包外改动」加「改被排除的被调方」，第三问「闭包内改一个单元」加「改调用方单元」；规范输出那一行不变，格不必改。
- 想让「写了排除声明却没钉」自动红：audit 加一条——射程里每一处 `# closure-excluded-edge:` 声明，所在脚本（或登记表第三列点名的判法脚本）的规范输出必须带 `exclusions=<n>` 且 n 等于声明数；弄坏开关 `audit-exclusions-unchecked`；样本 `reuse-selftests-red` 加一份写了声明、规范输出没带 `exclusions=` 的，`reuse-selftests-green` 加一份带的。这一条是提议，不是前提；被攻过零轮。
- 每条新弄坏开关在所在脚本文件头的开关表里写全三样（`break-switch-headers` 格）：「# 登记的脚本要：代码里读弄坏开关、文件头开关表每条三样写全（break-switch-headers 格）、在 .claude/hooks/selftest-registry.tsv 里登记成 light。」（`.claude/gate.d/break-switch-replay-files.tsv:2`）。

**推翻 V4 的现象**：排除条款写进去之后，把一条真会改判定内容的边（例如形式化入口里 `run_verus` 起 Verus 的那一条）误写成排除，上面的钉子格若不红，这些候选就钉不住它；这一点这一轮没做：没有实现，只是推。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| V1 内容层 | 一致 | 那条路只出标准错误提示、告警记录、节流文件，判法读 Verus 标准输出的 JSON，库只存 Verus 判出的数，不碰判定内容 |
| V1 结局层 | 冲突 | 包装文件头「退出码照旧」（`scripts/hosts/run-with-memory-cap.sh:115`）与 `self.thread.start()`（`scripts/hosts/run-with-memory-cap.sh:940`）、`tempfile.TemporaryFile`（`scripts/hosts/run-with-memory-cap.sh:1007`）没兜住异常并存：起不了线程或写不下临时文件时退 251 / 252 变 251，判定没跑成；注入量过，真机没量 |
| V2 按调用边排 | 一致 | 只切 `HostLookup.query` 起 `host-resources.py` 一条边：三个入口拒算 1 处变 0 处，单元 1579 变 1232，没有新增单元；广谱变异 `Runner.escalate_step` 同样 792 变 436、拒算 1 变 0 |
| V2 按文件 / 按函数 / 按开关排 | 冲突 | 按文件会丢 `predict` 等准入用的单元并抹掉被排文件里的拒算；按函数会丢能让包装退 251 的 `begin`、`finish`、`cancel`；按开关闭包静态判不了、而且等于关掉提示 |
| V3 键里算第二台那一侧 | 规则没说 | 已定项 17 只写到「第二台判出的并进本机那一份」；推出的是键里不放第二台，版本归消费者核（两台比判法单元指纹相等、导入时按本机现算的键逐条核） |
| V3 甲「第二台有同版本的仓」 | 冲突 | 题面要求同版本的仓，而 `multi-host.env.example:11` 写第二台只放每一趟的树副本、跑完删 |
| V3 `bash -c "$N"` 折法 | 规则没说 | 条文不用改；折对要五个条件，三个今天引擎里没有现成机制；已量的 35 个入口里丙一做就不再需要它 |
| V4 条文 | 规则没说 | 已定项 17 补一条按边排除的条款（逐字候选在上）；`implementation-workflow.md` 补一段，两份候选取决于 `HostLookup` 补不补兜底；三语各改一次 |
| V4 自证与 reuse-selftests | 规则没说 | 钉子放进四份脚本自证的真改动格与差分格；`reuse-selftests` 格本身可以不改，想自动逮「声明了没钉」就给 audit 加一问 |

## 没做什么

- 没替主 agent 采纳或判决；候选条文、自证名字、弄坏开关名都是提议，没落地，被攻过零轮。
- 没读被禁的文件：`m3-close-reuse-closure-r1-opus-output.md`、`m3-close-reuse-closure-r1-opus-model/`、`m3-close-reuse-closure-r1-main-verification.md`。
- 没跑任何重型测试，没编 Rust；没起真包装端到端（V1 的 251 路径是读代码加往 `HostLookup` 注入故障推的）；没跑 `verus-proof-check.py --selftest`（题面说的「119 格里 22 格 `reuse-*` 红」这一轮复核不了：脚本改过的自证不在副本里起）、没跑 `run-with-memory-cap.sh --selftest` 的 `host-lookup` 组、没跑 `broad-mutation.py --selftest` 与 `judging_units_key` 本体。
- 没读 `crates/singlefs-unit-graph`；Rust 一侧的闭包规则这一轮没碰；登记判法里走 Rust 闭包的（崩溃状态块、QEMU 每档、litmus、crates 变异每行）没逐个收。
- 丙的实现（切边）只在副本里用换掉一行代码模拟，没有实现「声明在注释里、按边排除」的闭包算法；甲的折法一行没写，只用今天引擎对三份 fixture 的行为和仓里 21 处 `peer_host_run` 调用的形状做了推。
- 21 处调用的四类归类是人工逐行归的，没有脚本核；第二台那一侧的真机行为（真 ssh、真第二台）没碰，也没读本机的 `multi-host.env`。
- 草稿目录里的闭包原始输出和冻结副本没入库（体量大，且都可由模型目录复跑得出）：`~/.local/share/singlefs/work/m3-close-reuse-closure-r1-sonnet/`；其中冻结副本 `repo/`（140 MB，仓副本）不删，理由是主 agent 复跑要指它，判决核完再删。
