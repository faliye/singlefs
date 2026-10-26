# m3-close-reuse-closure-r2 云端正推（Sonnet）报告（2026-10-22）

正推腿：从 `crates/`、`scripts/` 今天的实现与已定条款推「应该是什么」，再与被判形态（正文三、「被判的形态」）逐格比。立场照正文五：W1、W2 写改法落在哪个函数、自证格怎么造。模型目录 `research/prompts/m3-close-reuse-closure-r2-sonnet-model/`。

## 〇、开工校验、复跑、偏差

开工快照 `sha256sum -c research/prompts/m3-close-reuse-closure-r2-snapshot/judged-sha256.txt`：8 行全部 OK（`scripts/hosts/run-with-memory-cap.sh`、`host-resources.py`、`host-dispatch.py`、`peer-host-lib.sh`、`source-stamp.py`、`scripts/gate/lib_function_units.py`、`verus-proof-check.py`、`scripts/crash/lib_verdict_library.py`）。

复跑（模型目录整份拷走可跑；`--frozen` 要一棵含 `scripts/`、`.claude/{gate.d,hooks,scripts,singlefs-ai-sop}` 的树，脚本自己只拷这几处、不拷 `scripts/experiment-apparatus`；整条应经内存包装，我跑的时候本机 slice 被别的会话占满，包装排了约 5 分钟（总上限 40.1 GiB、已占 40.3 GiB）没排上，我停了它（先停它的 admit 子进程，再核整棵子进程都不在了），改用 `capped.sh 31 nice -n 19` 直接跑——脚本只起 python 与 bash，不起编出来的代码）：

    bash scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/m3-close-reuse-closure-r2-sonnet-model/tools/rerun.sh --repo <仓根> --frozen <冻结树> --work <新目录>

输出 `research/prompts/m3-close-reuse-closure-r2-sonnet-model/out/rerun-1.out`（下面写作「rerun 输出」，引它的行号）。其中 JOIN 一节的「用了 不到 5 秒 / 5 到 12 秒」随机器忙闲在两档之间跳（前一次跑是「不到 5 秒」），其余逐行确定。模板的 `--selftest` 校的是模板自带的 `rerun_steps`，我换了那一段，没跑它。

模型目录每个文件的 sha256（`cd <模型目录> && find . -type f | sort | xargs sha256sum`）：

79af6fb7e7c46f89cffde15ce72955c2a741c9d213015cdf810fb3931aba443b  ./fixtures/hd/a/scripts/helper.py
46dd518982356ab9744dd3c9f757ee0b548de1d7ab5cfc1dbf364e92a8dd8cbe  ./fixtures/hd/a/scripts/w.sh
79af6fb7e7c46f89cffde15ce72955c2a741c9d213015cdf810fb3931aba443b  ./fixtures/hd/b/scripts/helper.py
e85b7eca61b18e9f353e637553ab987600af11d389f2d73301c910ff8a801a4e  ./fixtures/hd/b/scripts/w.sh
d0244536496a9df8dd957372ee7c9b07303d8c2b89339472490f52eb4cd63475  ./fixtures/imp/scripts/a.py
a76b180810f41f80142df12b03743618b9509160d8b27d57316ea6809b328b13  ./fixtures/imp/scripts/loaded.py
37c5afe7dc66d7077097e0e9815534946b50f233c192acb87fd6ce36c746412b  ./fixtures/imp/scripts/side_effect_lib.py
073b1b3e50b77fa0c368970fb86bc42a01b1d2c90cb084c6aef34387c7fc8d90  ./fixtures/imp/scripts/unused_dep.py
85fd28c07606b50df4309b0cd50b770e4a5df5b19c2b912e17f1544340caf324  ./fixtures/launch/scripts/a.py
69666f9bd4286b04f71c4e0075e53622fcefa10d39d6a3cdaec676b8a721b8de  ./fixtures/launch/scripts/child_mod.py
ba76cd47158f901a0f324fc0c10cfe0d9ddc2fdf382dd288a5db97896822bfe9  ./fixtures/launch/scripts/child_run.py
6688905bfcbd61fedcb571dea13a2a718547d496fdec257776d3e96548a8ea7e  ./fixtures/launch/scripts/child_sub.py
37c5afe7dc66d7077097e0e9815534946b50f233c192acb87fd6ce36c746412b  ./fixtures/launch/scripts/dep_child_mod.py
37c5afe7dc66d7077097e0e9815534946b50f233c192acb87fd6ce36c746412b  ./fixtures/launch/scripts/dep_child_run.py
37c5afe7dc66d7077097e0e9815534946b50f233c192acb87fd6ce36c746412b  ./fixtures/launch/scripts/dep_child_sub.py
0c95bf64422855de56aa7526e03965c401a2b2375dc6f4f389a608bc047a8d33  ./out/closure-current-via-host-dispatch.txt
f3fd9ae966713a38bb9cfdc21490ee935999ccdd1750d41080b56902add631e1  ./out/rerun-1.out
918a6fd6bab79f4f0e8a05b39a1afc1999322fb57dc0ffd41dfb4cff650019ea  ./out/scan-current.out
dfa8b6a4fff4d2de91cd58dd85bda9e1bfa58920b8cada4cd0d5f2ceacad9f2e  ./out/scan-cut-lookup.out
05603dcaaf88b1211f52319b1c39c318634b9971539996a344f47d7175ecc201  ./out/scan-exec-all-draft.out
3e8baeb02f25c76cb56d12ce27ef24897144a0178130c477982bc3d4551a53f9  ./out/scan-import-edge-draft.out
ca9874a6d713e7e4d45de750d2cf8ee49744faa7221284e3d86dab2530d9b458  ./out/scan-load-edge.out
c0c0dc3a908abd7333f31c9e374825b49ce4bf8078d38c8bd5f01cbdabc13cb4  ./tools/edge_cells.sh
521f30fa023f3782b5db00695bf0f8f095d6f9585478c6698d0c1a5739a911f1  ./tools/entries35.txt
ad5911354584a089bbeaa01f92d7bcda870f032d3fc00e0e29104f04e8126064  ./tools/import_trace.py
87f3ca9f2b8ca04600a07b38034748c91ed007d115704270cf984bbac2983a10  ./tools/make_trees.py
717c4a90942f6ae967dee20272b2c850fef68af9b22879c36c8bfb07d8a9e34c  ./tools/make_wide_trees.py
35863825c497062d52ee0d82c84a2440ca6f45dbedcede0271f06288d29182e7  ./tools/patch_try.py
63c5b6520394aebcb66d0da2c39d8b955fecf6713f209e5d7d2030ba835dd174  ./tools/rerun.sh
be513afc47e6afe39097a255ff2f995fda1107b0cddeb078ab2aa00707276d39  ./tools/scan_compare.py
bb931a368bd95934f46e3272669bae19bf7bd8d65bf91ce8472e913d8a060812  ./tools/scan_entries.sh
a7358017fe7928fe2c0191f154bfa8eb8efe878d271a8a1c4fda4234b998f689  ./tools/strip_demo.py
c757e1cedd3466ec0e8c59d46783b219d03c61851bad7716e3ae30cfec34d593  ./tools/unit_diff.py
28add916672e4b872ab2ce4dbce057594d75b65e8c9011851b8a93983a32d19d  ./tools/wrapper_probe.py

偏差两处，照实写：
1. 我在 `grep -l` 全仓找一句提示用语时，输出里列出了 `research/prompts/m3-close-reuse-closure-r2-opus-model/` 下三个文件名（`w1-strip-judges.py`、`w1-queue-world.sh`、`make-variants.py`）。只看到名字，没打开任何一个，下面的推导与量都不用它们。
2. 参考了第一轮两条腿的产出（`m3-close-reuse-closure-r1-sonnet-output.md`、`m3-close-reuse-closure-r1-opus-output.md`）与第一轮判决；这些不在禁读清单里。第一轮攻过的角度（TMPDIR 被删退 251、stderr 末行进判定句子、`lib_long_run` 经那条边进闭包、第二台没有常驻仓、ssh 串不连边）本身不重复，只在需要时引用它们的结果。

## 一、W1：丙第 1、2 件之后，HostLookup 那条路还剩什么

**结论**：第 1 件方向对、但「起线程、前台建临时文件、读输出」三处不够，要兜的是 `HostLookup` 的四个方法加 `run` 里 `query()` 之后那一段；第 2 件「先去掉以 `run-with-memory-cap:` 起头的行」按字面写不成，两个方向都错；两件做完，还剩排队锁与 `join` 的时限一处、判法读到的行要分「判定用」与「给人看」两份。

### 1.1 第 1 件的 try 落在哪几个函数（读代码 + 探针）

读：`admit` 在排队循环里调 `lookup.begin`（`scripts/hosts/run-with-memory-cap.sh:1170` 「lookup.begin({"need": need, "slice_occupied": occupied, "slice_total": total_bytes})」），在三处退 252 之前调 `lookup.finish`（同文件 1093、1151、1162）。`begin` 起线程（`scripts/hosts/run-with-memory-cap.sh:939` 「self.thread = threading.Thread(target=self.run, args=("queued", local), daemon=True)」），`finish` 先 `join` 再前台再查一趟（`scripts/hosts/run-with-memory-cap.sh:949` 「self.run("refused", local)」），`query` 里前台建两份临时文件（`scripts/hosts/run-with-memory-cap.sh:1007` 「with tempfile.TemporaryFile(mode="w+", encoding="utf-8") as output, tempfile.TemporaryFile(mode="w+", encoding="utf-8") as errors:」）。异常冒出 `admit` 之后，包装按「排队那一步退了非 0 非 252」处理：`scripts/hosts/run-with-memory-cap.sh:1535` 「report_unavailable "排队那一步退 $admission_status（上面是它的报错），命令一行都没跑"」，它退 251（`scripts/hosts/run-with-memory-cap.sh:505` 「exit "$MEMORY_CAP_UNAVAILABLE_EXIT"」）。内嵌 python 未捕获异常的退出码是 1 属推（Python 的行为，没单独起整个包装量）；第一轮判决那次整包装的量是 TMPDIR 中途被删。

要紧的一点：`begin` 在排队循环里，它抛异常把「再等一会儿就能起跑」的命令直接变成 251，不只是把 252 变 251。

探针 `tools/wrapper_probe.py faults`（把 `admission()` 内嵌 python 原样载入，往 `HostLookup` 注入故障，只看异常往不往主线程外冒；注入的是条件，不是真把资源用光）：现状 6 格里 5 格异常冒出来（rerun 输出 133-137：起不了线程、`finish` 前台建不了临时文件、第二台那一份的 `memory_cap` 不是字符串、`human` 抛、`cancel` 里 `poll` 抛），第 6 格（后台线程里建不了临时文件）异常不冒出主线程、但往 stderr 打了 11 行 traceback（rerun 输出 138）。比第一轮多出三个冒出点：`run` 里 `query()` 之后的 `size_bytes(peer["memory_cap"] or "")`（`scripts/hosts/run-with-memory-cap.sh:1055`）、`human`、`cancel`。`query` 自己的 try 只收 `(ValueError, KeyError, TypeError, AttributeError, StopIteration)` 且只罩读 JSON 那一段（1040），罩不到后面。

改法（`tools/patch_try.py`，只在副本里，不是实现）：
- `run` 改名 `run_unguarded`，新 `run` 一个 `except Exception` 兜住、`say` 一行说明；`begin` 里起线程两行包 try；`cancel` 里 `process.kill` 包 `contextlib.suppress(Exception)`。线程的目标就是 `run`，线程里的异常同样兜住，不再打 traceback。`except Exception` 不吞 `SystemExit`：排队时包装被 TERM 停，处理器 `sys.exit(128+n)` 照旧往外走（`scripts/hosts/run-with-memory-cap.sh:1353`）。
- 改后同样 6 格：0 格异常冒出，每格一行说明（rerun 输出 140-145，「打了 1 行」）。

自证格怎么造：在 `--selftest` 的 `host_lookup_cells`（1776 起）里加注入点而不是靠环境造——`TMPDIR=<不存在的目录>` 造不出「建不了临时文件」：Python 起头就坏的 TMPDIR 会退到 `/tmp`，只有中途删才坏（命令行一次手跑：`TMPDIR=/nonexistent bash -c 'python3 /dev/fd/3 3<<"PY" …'` 里 `tempfile.TemporaryFile()` 照建成，except 分支那行没打，输出只有「heredoc fine」；一次性手跑，不在 rerun 里）。所以加一个只给自证的注入变量（文件头已有同类 `RUN_WITH_MEMORY_CAP_SELFTEST_ADMIT_PAUSE`，1087 行读它），取值 `thread`、`tempfile`、`peer-json`、`human`：每个故障两格——（一）挡着路、要退 252 的世界：退出码与同一世界设 `RUN_WITH_MEMORY_CAP_HOST_LOOKUP=off` 的那一臂逐个相同（252、命令没跑）；（二）不挡路、排一会儿就放行的世界：退 0、命令跑了。弄坏开关 `hostlookupunguarded`（`run` 不兜、`begin` 不包）→ 这几格红。

### 1.2 try 之外还剩：排队锁与 join 的时限、告警记录、节流文件

- **时限（实测 + 推）**。`take_turn` 在后台线程里阻塞地拿排队锁（`scripts/hosts/run-with-memory-cap.sh:969` 「with admission_lock():」），`finish` 与 `cancel` 的 `join()` 没有时限（947、959 行）。探针 `join`：另一个进程持排队锁 40 秒时，现状与「只加 try」两臂的 `finish` 12 秒还没回来（rerun 输出 166-169）；`take_turn` 改成 `LOCK_NB` 加 2 秒上限、`join` 加 `HOST_LOOKUP_SECONDS + HOST_LOOKUP_PEER_TIMEOUT_SECONDS + 5` 的上限后回来了（rerun 输出 170-171）。「只给 join 加上限」不够：`finish` 随后在主线程里再 `run` 一趟，它的 `take_turn` 也卡在同一把锁上，我先试过只改 `join`，仍不回。现实触发弱（持锁方都是毫秒级；而且 `admit` 自己 1088、1109 行也是阻塞拿这把锁，持锁方真卡死时排队本来就卡），所以这一条只改「退 252 的路」会不会因它拖住，属推；便宜，建议顺手加。常量在 `scripts/hosts/run-with-memory-cap.sh:775` 「HOST_LOOKUP_SECONDS = 15.0」与 777 行上下。
- **第二台挂死**：`query` 里 `process.wait(timeout=limit)` 到点杀 host-resources.py，留下它起的 ssh 自己收尾（957 行注释）；退 252 之前最多等两趟（先等排队开始那一趟，再自己一趟）。只拖时间，不改退出码。
- **告警记录**：`core_usage.append_line` 是普通追加（`scripts/hosts/core_usage.py:274` 「with open(path, "a", encoding="utf-8") as handle:」），`record` 已有 `except Exception`（1077）；它的作用是叫醒看门狗，不碰退出码与判定。
- **节流文件** `host-lookup.json`（`take_turn`，967-985）：只决定几条一起排队时哪个包装打提示，`except OSError: pass`。它不改结局，但让「哪个包装的 stderr 里有提示」随并发变——这正是第 2 件要处理的东西。
- 一处刻意不兜：`SystemExit`（TERM 处理器）、`KeyboardInterrupt` 不是 `Exception`，照旧外冒，是对的。

### 1.3 第 2 件「先去掉以 `run-with-memory-cap:` 起头的行再读」：去多了，也去少了

**去多了（实测）**：广谱变异升档的证据要在日志里读到包装打的撞顶句——`scripts/mutation/broad-mutation.py:2219` 「if wrapper_hit_text(first_exit, first_text) not in first_log_text:」；那句是 `scripts/hosts/run-with-memory-cap.sh:1624` 「echo "run-with-memory-cap: 撞了内存上限 $cap（scope $unit 的 Result=oom-kill，命令退出码 $exit_code）" >&2」，超时那句是 1630 行「echo "run-with-memory-cap: 超过限时 ${TIME_LIMIT_SECONDS} 秒」，都以 `run-with-memory-cap:` 起头。步的 stdout 与 stderr 并进同一份日志（`scripts/mutation/broad-mutation.py:2553` 「stderr=subprocess.STDOUT, pass_fds=(self.library.lock_descriptor,))」）。探针 `tools/strip_demo.py` 载入真的 `escalation_evidence_problems`：原样读证据缺 0 样，先去掉前缀行再读缺 2 样（rerun 输出 173-174）。也就是第 2 件按字面做，被判入口 `Runner.escalate_step` 每次升档都判「证据不齐」。

**去少了（实测）**：HostLookup 打的行里不带前缀的有：放得下的提示行（`scripts/hosts/run-with-memory-cap.sh:1060` 「line = f"  → {peer['name']}此刻放得下（可用」，带第二台此刻的可用量与预测量，随第二台状态变）、告警记录写不进（1078 「say(f"  ! 告警记录写不进（」）、后台线程异常没兜住时 threading 默认钩子打的整段 traceback（探针 `lines`：现状这四类里 12 行 `prefixed=NO`，rerun 输出 147-164）。形式化入口收「包装自己的 ✗ 与 → 行」：`scripts/gate/verus-proof-check.py:803` 「return [line.strip() for line in outcome.get("stderr", "").split("\n") if line.strip().startswith(("✗", "→"))]」，提示行去掉行首空白后以 → 起头，所以会被当成包装自己的结局句，成为 `QueueRefusal.record` 的「包装说」（689）与「没跑成」那条的 `carried`；前缀去掉与否它都在（rerun 输出 175-176，头一行仍是提示行）。

**怎么改（推）**：判法侧不能靠前缀猜，要在源头给 HostLookup 的每一行输出同一个固定标记——`say` 之上加一个 `lookup_say`，四个分支与「告警记录写不进」都走它，行首标记如 `run-with-memory-cap[查别的机器]:`；`run` 兜住异常之后线程不再打 traceback。判法侧（形式化三个入口的 `run_verus`、广谱变异读日志、crates 变异）只去掉带这个标记的行；去掉的行改为转打到判法自己的 stderr 给人看（`scripts/gate/verus-proof-check.py:631` 「WRAPPER_QUEUE_LINE = re.compile(」那条转打的做法），不进判定用的输入——否则 252 时用户在「没跑成」里看不到「改用 --peer」的提示。要同改的消费者：`scripts/gate/merge-and-verify.sh:505` 的 `^  → 第二台此刻放得下` 锚在行首。
自证格：（a）HostLookup 四种结局加线程异常，抓全部 stderr，每行必须带标记或属包装已有的固定句（`wrapper_probe.py lines` 是这一格的原型，现状红）；（b）判法侧：同一份 Verus 输出配不同的第二台末行，写进库的判定逐字相同，且 `escalation_evidence_problems` 仍读得到撞顶句；弄坏开关：判法不去标记行 → （b）第一半红；去掉所有 `run-with-memory-cap:` 行 → （b）第二半红。
另一条更省的路（推）：四个判法入口起包装时设 `RUN_WITH_MEMORY_CAP_HOST_LOOKUP`（`scripts/hosts/run-with-memory-cap.sh:990` 「switch = os.environ.get("RUN_WITH_MEMORY_CAP_HOST_LOOKUP", "")」），这一支整个不出现在被判路径上，只剩一句固定文字，第 1、2 件也就不是被判入口的前提；代价是判法跑出的 252 不再带「改去第二台」的提示。用户 2026-10-20 要的就是这个提示（包装文件头 103 行起），所以我推标记那条；主 agent 定。

**什么现象会推翻 W1 的结论**：（一）能造出一个 HostLookup 的故障，在补了 `run`/`begin`/`cancel` 的 try 之后包装退出码仍与关掉 HostLookup 的那一臂不同（我的探针只注入了 6 个点，不是穷尽）；（二）广谱变异、crates 变异另有读包装前缀行的地方我没读到（我只读了 `escalation_evidence_problems`、verus 的 `wrapper_own_lines` 与 `diagnostics` 三处，crates 变异的 `scripts/mutation/crates-mutation-rows.py` 的 `wrapper_outcome` 只看退出码，没读它对 stderr 文字的用法）。

## 二、W2：丙第 3 件「只排一条边」加「exec_module 连边」

**结论**：切 `HostLookup.query` 起 `host-resources.py` 那一条边做对了一半，且只动 4 个入口；「`exec_module` 要连成边」这一半不对症——那条边今天就连着，漏的是载入时一并执行的**模块级 import**，切边之后 `lib_long_run` 一类单元因此掉出闭包，一行语法错就能让「真实载入会抛、键不变」。要补的是闭包算法里「按路径载入的模块，它的模块级 import 连同引来的模块一起收」，落在 `ScriptClosure.python_edges`。

### 2.1 只切一条边：量到的

`tools/make_trees.py` 的 `cut-lookup` 树只把 `HostLookup.query` 里 `Popen` 那一处换成 `None`。`check_proof` 的闭包 1579 → 1232 个单元，拒算 1 → 0（rerun 输出 2-3：「RESULT closure tree=current rc=4 units=1579」「RESULT closure tree=cut-lookup rc=0 units=1232」）。35 个登记入口（`tools/entries35.txt`，与第一轮同一份）里闭包变了的只有 4 个，3 个形式化入口加 `Runner.escalate_step`，拒算变多的 0 个，其余 31 个单元摘要逐个相同（rerun 输出 88-92）。所以「按边排除写进算法会不会让别的入口变」：按（调用方单元，被调文件）键的排除，不会。

少掉的 348 个单元（含改动自身造成的 1 进 1 出）按文件分（rerun 输出 9-27）：
- 227 个在 `predicted_bytes` 路径上根本不会 import 的文件里：`rocksdb-prebuilt.py` 101、`host-resources.py` 75、`crash-amplification-lib.sh` 26、`crash-point-replay-shard-configuration-check.sh` 20、`peer-host-lib.sh` 5。它们只经查别的机器那一支进闭包；W1 兜住之后那一支只改提示、不改结局，所以这 227 个不改结局。
- 119 个在载入 `host-dispatch.py` 时要 import 的文件里：`host-dispatch.py` 41、`lib_long_run.py` 40、`lib_heavy_tests.py` 18、`memory_peaks_lib.py` 9、`lib_shell_words.py` 5、规范副本 `preflight.py` 3，另三个各 1。其中真被 `predict` 走到的函数仍在闭包里；掉出去的 `lib_long_run`、`lib_heavy_tests`、`lib_shell_words` 是载入时执行的，不是 `predict` 调到的。

### 2.2 「exec_module 连边」今天就连着，漏的是模块级 import

读：`call_facts` 认 `spec_from_file_location` 的字面路径（`scripts/gate/lib_function_units.py:4384` 「if canonical == "importlib.util.spec_from_file_location" and "python-spec-paths-ignored" not in breaks:」），`python_edges` 对载入的文件只推它的模块体（`scripts/gate/lib_function_units.py:7256` 「self.push(loaded.module_body, unit, "load")」）和已被名字引到的单元。`closure --explain` 里 `host-dispatch.py:<module-body>` 就是经 `predicted_bytes` 的 `load` 规则进来的（`closure --explain` 输出里 `grep -n 'host-dispatch.py:<module-body>'` 的结果存在模型目录 `out/closure-current-via-host-dispatch.txt`，第 3 行是 「host-dispatch.py:<module-body>」经 「run-with-memory-cap.sh:predicted_bytes」 的 `load`；行首的 2786 是整份 explain 输出里的行号）。载入 `host-dispatch.py` 要执行它的 `import lib_long_run as long_run_lib`（`scripts/hosts/host-dispatch.py:141` 「import lib_long_run as long_run_lib」），这个名字只在 `header_hosts` 里被引，而 `header_hosts` 只经 host-resources 那一支可达。

实测（独立两条路，互不共用代码：Python 自己 import 与静态闭包）：`tools/import_trace.py` 真载入 `host-dispatch.py`，列实际 import 的仓内文件 6 份，再与闭包里出现的文件比：现状树缺 0 份、只切边的树缺 3 份（`lib_heavy_tests.py`、`lib_long_run.py`、`lib_shell_words.py`）、切边加下面 2.3 的补丁缺 0 份（rerun 输出 81、84、87）。再往切边树的 `lib_long_run.py` 末尾加一行 `def broken(:`：闭包 rc=0、仍 1232 个单元、无拒算，键不变（rerun 输出 6，「cut-lookup-syn rc=0 units=1232」）；而真实 `exec_module` 在这一行上会抛，`predicted_bytes` 吞掉异常回 `(None, …)`（`scripts/hosts/run-with-memory-cap.sh:861` 「except Exception as error:   # 导入或调用出错：不当成 0，按上限算」），`need_of` 改按上限排队（883 行），排队结局变。这就是第一轮「`lib_long_run` 的一个单元改了，形式化入口的键变」那格的缺口，也说明那格自证要挑模块体或模块级 import，挑一个只在函数体里的单元，补丁之后键也不变。

**别的起 python 文件的地方是不是同样不连**：是，且不限于 `exec_module`。模型目录 `fixtures/launch`、`fixtures/imp` 里每个被起文件有一条 import 时必抛的依赖（`int("not a number")`、`raise SystemExit`），名字没被引：按路径载入、`runpy.run_path`、`subprocess` 起 `sys.executable` 加脚本、`importlib.import_module`、普通 `import` 五种写法，闭包里都只有被起文件自己，依赖文件都不在（rerun 输出 126-129）。所以这不是切边新造的毛病，是算法对模块级 import 一贯按「名字被引到才收」，切边把它从「被另一条边碰巧罩住」变成裸露。

### 2.3 改法落点与它的代价（量）

落点：`ScriptClosure.python_edges`（`scripts/gate/lib_function_units.py:7218`）。载入规则那一段把被载入的文件记进 `executing_modules`；模块体单元出队、且本模块在这个集合里时，把本模块每个 `import` 单元推进去；`import` 单元出队时把它引来的仓内模块也记进集合、推它的模块体。原型在 `tools/make_trees.py` 的 `load_edge`。
- 切边 + 这个补丁：`lib_long_run` 等回到闭包，语法错那棵树拒算 `python-syntax`（rerun 输出 5、7：「cut-load-edge rc=0 units=1289」「cut-load-edge-syn rc=4 units=1257 refused=[      1 python-syntax;]」）。
- 代价：35 个入口里闭包变的 25 个，每个多 +18 到 +45 个单元，拒算变多的 0 个（rerun 输出 93-118）。已定项 17 写了「改键让旧库作废不算代价」（`.claude/kb/decisions/13-验证路线.md:436` 「改键让旧库作废不算代价：不做兼容迁移、不为旧行补签，改完重跑。」），所以这是允许的，但要在判决里写明「这一改让 25 个入口的键变」。
- 更宽的写法我量过（模型目录 `out/scan-import-edge-draft.out`、`out/scan-exec-all-draft.out`，单次产物；`tools/make_wide_trees.py` 重建这两棵树，`check_proof` 的单元数与拒算数重跑对得上，摘要不同是因为当时的草稿补丁多一个环境开关、算法文件自己的单元不同；rerun 没带这两棵树）：「每个模块体都连它的模块级 import」31/35 个入口的闭包变；再连模块级赋值和类头，31/35 变且 17 个入口（`broad-mutation.py` 的 14 个、`crates-mutation-rows.py` 的 3 个）由 0 拒算变成 4 到 6 条拒算（闭包涨到 1844 到 2285 个单元），那一写法不可取。
- 备选，不改算法：把 `host-dispatch.py` 载入时 import 的几份并进这 4 个入口的键（手写清单，会漂）。自证格就是上面的 `import_trace.py`：它把「真 import 到的仓内文件」与「闭包里的文件」对一遍，漂了会红，两条路不共用代码。

### 2.4 按边排除落在哪、自证格怎么造

落点：同一个函数的 `for file in facts.program_links:`（`scripts/gate/lib_function_units.py:7257` 「for file in facts.program_links:」）前加一个跳过：（调用方单元名，被调文件）在声明里就不 `push_program`；声明读自调用方源文件里的 `# closure-excluded-edge: <调用方单元> -> <被调文件>：<理由>` 注释；闭包收完后，声明没被用上的拒算 `closure-exclusion-stale`。`HostLookup.query` 是包装内嵌 python 的单元，`unit.file` 是 `run-with-memory-cap.sh`、`unit.name` 是 `HostLookup.query`，对得上。声明写在那段 heredoc 里：实测内嵌 python 的注释算在 shell 函数 `admission` 的指纹里（rerun 输出 130-131：两个只差声明理由一个字的变体，`admission` 指纹不同），所以声明进键，改了或删了键就变。解析必须锚在注释行首（`^\s*#\s*closure-excluded-edge:`）：不锚的话 `lib_function_units.py` 自己的源码里那条正则字面量会被当成声明（它是形式化入口闭包的成员），首版原型在草稿里量到 2 处 `closure-exclusion-stale`，已改锚定。
五格（`tools/edge_cells.sh`，rerun 输出 119-125）：改被调方 `host-resources.py` 的常量键不变；改调用方包装的常量键变；改声明理由键变；声明指到不存在的边拒算（同时因为真边不再被排，`inline-script-not-literal` 回来）；入口自己另有一条直接起 `host-resources.py` 的边时，它的 75 个单元照进。弄坏开关：声明不生效 → 第一格红；声明不进键 → 第三格红；不核声明用没用上 → 第四格红。

**什么现象会推翻 W2 的结论**：（一）切边树上有一个会改 `predicted_bytes` 结局的、既不在 `import_trace` 的 6 份里、也不是模块级赋值的东西（我没穷尽 `predict` 往下的调用，只对了载入时的 import 集合）；（二）补丁之后 25 个入口里有哪一个出现新拒算（我量了 0）；（三）模块级赋值在 import 时抛（`HARD_CODED_QUOTA = re.compile(…)` 一类）仍不在闭包里——补丁没收，更宽的写法不可取，这是留下的缺口，只能靠 `import_trace` 那一格的「真载入要成功」这一半兜（现在那一格只比文件集，不比成败）。

## 三、W3：甲（第二台跑同一份仓里的脚本，取自每一趟同步过去的树副本）

**结论**：甲按正文的写法站不住，原因有四：（一）今天送 base64 程序的做法本身就保证「第二台跑的是本机这一份的文本」，甲把它换成「靠同步保证」，保证变弱；（二）`source-stamp.py sync` 不是跨机的同步，探测那一刻第二台上也没有这一次的树副本；（三）「`scripts/hosts/` 下用到的几份」点不全；（四）「导入时照已有做法按本机现算的键逐条核」在广谱变异的导入里不是已有做法。键里不算第二台那一侧本身站得住，但站的理由不是甲给的那条。

### 3.1 今天的做法：程序按值送，第二台上不需要仓

`remote_probe_program` 把本机 `memory_peaks_lib.py` 的全文加 `probe_this_host` 的源码拼成程序（`scripts/hosts/host-resources.py:230` 「return "\n".join([library, inspect.getsource(probe_this_host), PROBE_DRIVER])」），base64 编码后经 `peer_host_run /` 在第二台的根目录起（`scripts/hosts/host-resources.py:314` 「'peer_host_run / "$5"')」；`scripts/hosts/host-dispatch.py:316` 同样）。所以第二台上那一份与本机那一份是同一份文本，是构造出来的，不靠任何同步。一次实测（只此一次，复核不了的写在第七节）：`host-resources.py --json -- true` 全程 1.6 秒，其中 `ssh true` 单趟 0.20 秒（连测 3 次都是 0.20）；HostLookup 给整次查询的时限是 15 秒（`scripts/hosts/run-with-memory-cap.sh:775`）。

### 3.2 第二台上有没有树副本、`source-stamp.py sync` 是不是跨机

- 树副本只在 `multi-host-run.sh --peer` 跑命令那一趟存在：`multi-host.env.example:11` 「第二台上放这一趟树副本的专用目录（绝对路径）：每一趟在它下面建一个子目录放树与编译目录、跑完删掉；」；子目录是 `scripts/hosts/multi-host-run.sh:391` 「peer_run_directory="$peer_runs/$run_label"」，跑完 `rm -rf`（214 行）。HostLookup 在本机排队时探测（`begin`/`finish`），那一刻没有「这一条命令」的树副本。第二台 `runs/` 里此刻别人的目录碰运气：一次实测 6 个目录，3 个带 `host-resources.py` 且摘要与本机相同，3 个没有这份脚本（只此一次的观测；不能当可用的树）。
- `source-stamp.py sync <源树> <副本>` 是本机到本机：`scripts/hosts/source-stamp.py:186` 「os.makedirs(copy, exist_ok=True)」先在本地建目录，再 `rsync` 到同一个本地路径。把目的地写成「主机:路径」，它在当前目录下建了一个名叫 `fakehost:` 的本地目录并报成功（rerun 输出 177：「SOURCE_STAMP_SYNC rc=0 本地多出的目录: fakehost:」）。跨机拷树的是 `multi-host-run.sh` 的 ④：只拷 git 列得出的文件，排除 `.git`、`target` 与规范副本（`scripts/hosts/multi-host-run.sh:437` 「{ { peer_host_copy --exclude target --exclude .git --exclude /.claude/singlefs-ai-sop --from0 --files-from="$files_list" -r」），规范副本另拷（447 行），是 `rsync -a`，不是 `--no-times --checksum`。
- 所以甲要的「每一趟同步」得为探测另做一套：每次查询新建目录、拷、探测、删，且拷前先有第二台的 ssh 往返。节流（每 60 秒一次，`HOST_LOOKUP_EVERY_SECONDS`）压得住次数，压不住每次的成本和失败面。

### 3.3 「`scripts/hosts/` 下用到的几份」点不全（实测，独立路径）

第二台上要把 `host-resources.py` 当程序跑（甲的「由参数选机器」）：`tools/import_trace` 一类的真 import 列出，载入 `host-resources.py` 并调 `dispatcher()` 之后仓内被 import 的文件有 `memory_peaks_lib.py`、`.claude/scripts/project_preflight.py`，再加载 `host-dispatch.py` 带来的 `.claude/hooks/lib_long_run.py`、`lib_heavy_tests.py`、`lib_shell_words.py`（草稿里一次手跑的输出，与 rerun 输出 80、83 里 `host-dispatch.py` 的载入集合同一批；`host-resources.py` 那一份没进 rerun）。当程序跑时它还调 `preflight(__file__)`（`scripts/hosts/host-resources.py:984` 「preflight(__file__)」），要规范副本 `preflight.py`，而规范副本被 `.gitignore` 挡着（`.claude/scripts/project_preflight.py` 文件头「规范副本 .claude/singlefs-ai-sop/ 被 .gitignore 挡着」），`multi-host-run.sh` 是另拷的。清单里 `scripts/hosts/` 之外就有 5 处。漏拷的结局是第二台那一支 ImportError 或 preflight 红，探测失败——安全方向，不会判错。清单要从真 import 机械生成，不手写。

### 3.4 同步失败 / 时机不对时第二台跑的是不是同一份

- 探测路径：失败 = 没有提示（W1 兜住之后不改任何退出码与判定）。`rsync -a` 不带 `--delete`，复用同一个目录会留下旧文件；每次新目录则没有这个问题。拷到一半本机文件又被改，第二台跑的是混合版：最坏是提示里的可用量或「放得下」错，不进判定。
- 判定路径（`multi-host-run.sh` ④ 已有的）：`rsync` 非 0 一律失败、命令不起（`scripts/hosts/multi-host-run.sh:444` 「multi_host_fail "树 rsync 到第二台失败」），唯一放行的是退 24 且消失的全是没跟踪的文件（441 行）。
- 要让「同一份」成为证据而不是信任：第二台在同步之后对清单里每个文件打 `sha256sum`，本机对同一批比，不等就判探测无效；这与程序按值送相比只多出这一步。若目的只是让两台跑同一个探测函数，按值送已经做到，不改。

### 3.5 「键里不算第二台那一侧」

探测不产生键，没有这个问题。判定路径上它站得住，理由是：复用只认键相等，第二台算键用的是它自己跑的那棵树（④ 每趟新拷、跑的过程中不变），键相等就是单元指纹多重集相等；树版本不对的行本机现算的键对不上，永远用不上（失效方向安全）。但「导入时按本机现算的键逐条核」不是广谱变异导入的做法：`import_results` 只核键是 64 位十六进制（`scripts/mutation/broad-mutation.py:4157` 「if not re.fullmatch(r"[0-9a-f]{64}", row["key"]):」）和两边结局有没有冲突，没有重算（4137 起的函数里没有 `closure_key`/`judging_unit_key` 一类的调用，读到的）。要么条文改成「导入不核键，复用时键相等才用」，要么给导入加重算（要给每行重建它的入口与输入，成本不小）。crates 变异并入本机库的路径我没读（`scripts/mutation/mutation-shard-run.sh` 文件头说经 `crates-mutation-rows.py` 并入），不下结论。

**什么现象会推翻 W3 的结论**：（一）第二台上有一份常驻、被别的机制保证同版本的仓（`multi-host.env.example` 的专用目录不是，若以后改成常驻则（二）（三）的前提变）；（二）`import_results` 之外有一条导入路径在重算键；（三）按值送的程序在第二台上不是本机那一份文本（如被 ssh 的 shell 吃掉了字符）——可以让第二台回显程序文本的 sha256 来证；我没做。

## 四、W4：丙加甲合起来的条文，与 `NOT_JUDGING_FILES`、已定项 17 第 432-435 行

**结论**：条文可以写成与已定项 17 自洽，但要以 W1-W3 的改法为前提；按正文第三节的原样（前缀、只排一条边、甲的「已有做法」）写，有三处写不成或与第 432-435 行、实现打架。已定项 17 现在没提 `NOT_JUDGING_FILES`，那是一条没写进条文的文件级排除，新条文一并写进去才不留第二条沉默的例外。

### 4.1 逐行对读（原文整行抄，行号现查）

- `.claude/kb/decisions/13-验证路线.md:432`：「函数单元从这份判定的入口沿调用与引用收，连同引用的常量、类型、静态项与宏展开一起算；wgsl 按 fn 切。」——按边排除是这句的例外，必须写成例外，不能靠「不改结局」这类判据；W1 量到「不改结局」要靠兜住异常才成立，「不改写进库的内容」要靠分离标记行才成立。
- `.claude/kb/decisions/13-验证路线.md:433`：「一律不进键、也不许当复用依据：按提交、HEAD 或树算的改动范围；整文件原文；文件级、包级、整表级的指纹或闭包」——边排除不是文件级指纹，不冲突；但 `scripts/crash/lib_verdict_library.py:88` 「NOT_JUDGING_FILES = ("scripts/crash/lib_verdict_library.py", "scripts/gate/lib_function_units.py")」按文件把两份库的单元整份拿出键（该文件 200 行 「if identity in seen or unit.file in excluded_files or f"{unit.file}::{unit.name}" in excluded_units:」），这是文件级排除，条文没写，只在 `lib_verdict_library.py` 文件头 19-20 行有一句理由。新条文把它和边排除并列写成「收得到而不进键的两样」，才不留沉默的例外。
- `.claude/kb/decisions/13-验证路线.md:434`：「收不全（动态分派、宏、`include_str!(concat!…)` 认不出）的那一份拒算键、照跑，不静默少收；每把键连同组成它的单元清单落盘，作废时报得出是哪几个单元变了。」——边排除是「认出来了、判定不依赖」，不是「收不全」；要求声明进键（实测声明随调用方单元的指纹进键，rerun 输出 130-131）、没用上的声明拒算（rerun 输出 123）。清单（`lib_verdict_library.py:217` 「return UnitKey(key, tuple(sorted(unit.text() for unit in merged.order)), tuple(refusal.text() for refusal in refusals))」）只列单元，不列被排除的边；作废时看得到调用方单元变了，看不到「这把键排了哪条边」，建议清单另列每条用上的声明。
- `.claude/kb/decisions/13-验证路线.md:435`：「每类判定一份库，固定在持久目录，不随一趟、不随判法换目录；仓根也不是库的输入：崩溃判定库的目录由环境变量 `SINGLEFS_CRASH_AMPLIFICATION_LIBRARY` 给时只收绝对路径，相对路径拒绝；全量开关只写不读、现判的照键写回同一份库；第二台判出的并进本机那一份。」——与「键里不算第二台那一侧」相容，但「按本机现算的键逐条核」不能写成已有做法（W3.5）。
- 第 436 行「改键让旧库作废不算代价」：2.3 的补丁让 25 个入口的键变，在这一行的允许范围内，判决里写明即可。
- `.claude/rules/implementation-workflow.md:31`：「改动之后一条判定要不要重判，判据是**它的单元键在判定库里有没有键相同、作数的判定**」那一段没有任何排除，补一段（见 4.2）。

三处（按正文原样写时）：（一）第 2 件的前缀写法与升档证据冲突，条文里「判法读标准错误时去掉以 `run-with-memory-cap:` 起头的行」一句写出来就是错的（rerun 输出 174）；（二）「`exec_module` 连边」写进条文，条文说的是今天已有的 `load` 规则，真缺的模块级 import 没写；（三）甲的「照已有做法按本机现算的键逐条核」与 `import_results` 不符。

### 4.2 逐字候选（被攻过零轮，是这一条腿推的）

已定项 17，插在 `.claude/kb/decisions/13-验证路线.md:434` 那一条之后（`.en.md` 的 Settled Item 17、`.ja.md` 的 確定項目 17 各加同一条）：

> - 沿调用与引用收闭包时，收得到而不进键的只有两样：算键、存库的两份库文件（`scripts/crash/lib_verdict_library.py` 的 `NOT_JUDGING_FILES`：它们管怎么存、怎么算键，不管判对错），和调用方源码里用 `# closure-excluded-edge: <调用方单元> -> <被调文件>：<理由>` 声明过的调用边。边排除只落在这一条边上：被调文件经别的边进闭包时它的单元照进，调用方单元与闭包里别处的拒算照旧。声明要同时满足：被调方是调用方另起的程序，它的输出只进调用方的提示行与告警记录，不进判定读的退出码、标准输出与写进库的内容；调用方对被调方的异常、失败、超时与读不出的输出各自兜住，退出码照旧；判法读调用方标准错误当诊断时，只读不带该调用方提示标记的行。声明写在调用方单元的源码里、随该单元的指纹进键，声明没被用上的拒算；每条声明有一格自证：改被调方键不变，改调用方键变，被调方经别的边进来时照进，改了或删了声明键变，指到不存在的边拒算。按路径载入（`exec_module`）、`runpy`、子进程起的 python 文件，载入时执行的模块级 import 连同它们引来的模块一并收。

「依据」一节另写一行：排除条款是云端正推腿推出的，被攻过零轮。

`.claude/rules/implementation-workflow.md`，插在第 31 行那一段之后：

> 闭包里收得到而不进键的只有两样（D13（验证路线） 已定项 17 的排除条款）：算键、存库的两份库文件，和声明过的调用边。声明过的边目前只有一条：内存包装 `scripts/hosts/run-with-memory-cap.sh` 排队时查别的机器那一支（`HostLookup.query` 起 `scripts/hosts/host-resources.py`），它只往带固定标记的提示行与告警记录写，出任何异常都只打一行说明、不改包装的退出码；判法读包装标准错误当诊断时只读不带标记的行，带标记的行转打给人看。广谱变异导入第二台判出的行时只核键的形态与两边结局有没有冲突，不重算键；复用靠本机现算的键与行里的键相等。

两点保留：（一）第 433 行说路径与名字不进键，声明里写了被调文件的路径——它是调用方单元源码文本的一部分，和 shell 函数指纹里本来就有的字面路径同一性质，但字面读有争议，判决时点一句；（二）候选里的「提示标记」要等 W1 的标记写法定了才有具体名字，现在是占位描述。

**什么现象会推翻 W4 的结论**：（一）攻方给出一条已定项 17 第 432-436 行里我没对的句子，与候选直接冲突；（二）`NOT_JUDGING_FILES` 另有条文（如 `.claude/kb/` 别处）已写明，使「沉默的例外」不成立——我只读了 `13-验证路线.md`、`implementation-workflow.md` 与 `lib_verdict_library.py`。

## 五、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| W1 第 1 件（try） | 冲突 | 方向对但三处不够：`begin`/`finish`/`run` 后半/`cancel` 都要兜，现状 5 个注入点异常冒出，兜完 0 个；`begin` 在排队循环里，抛了会把能起跑的命令变 251 |
| W1 第 2 件（去前缀行） | 冲突 | 去多了（升档证据缺 2 样）也去少了（12 行不带前缀）；改成源头加标记、判法只去标记行并转打给人看 |
| W1 残留 | 规则没说 | 排队锁与 `join` 的时限（现状 12 秒不回，改后回）；告警记录与节流文件不改结局 |
| W2 只排一条边 | 一致 | 1579→1232，拒算清零，35 个入口只变 4 个，其余摘要逐个相同 |
| W2 exec_module 连边 | 冲突 | 边今天就有（`load` 规则）；漏的是模块级 import，切边树 + 语法错键不变；补丁后 25 个入口键变、0 新拒算 |
| W2 别的起 python 的地方 | 一致（同样不连） | 五种写法都只带被起文件自己，不带它的模块级 import |
| W3 同一份脚本 | 冲突 | 按值送本来就保证同一份；`source-stamp.py sync` 本机到本机；探测那刻没有树副本；清单点不全 |
| W3 键里不算第二台 | 一致 | 理由是复用靠键相等、第二台算键用的是它执行的树；「已有做法逐条核」不成立 |
| W4 条文自洽 | 冲突（改写后自洽） | 按正文原样有三处与 432-435 行打架；`NOT_JUDGING_FILES` 要一并写进条文；候选句见 4.2 |

## 六、没做什么

- 没判别、没替主 agent 采纳或出判决；没改仓里任何文件（改动都在草稿副本与模型目录里）。
- 没跑任何重型测试、没起 cargo、没跑 `gate.sh` 与任何 `--selftest`；补丁（`patch_try.py`、`make_trees.py`）只是量用的原型，没写进实现，包装与闭包算法的真改动与它们的自证格是实现员的活。
- 没用整条包装验证退出码（未捕获异常 → 251 这一段是读代码加进程内探针的推；第一轮整包装的量不重做）。
- 没读 crates 变异并入第二台判定的路径、没读 `crates-mutation-rows.py` 对 stderr 文字的用法，W1、W3 里相关处都标了。
- 没量真正的 TMPDIR 中途被删、盘满；注入的是条件。
- 禁读清单里的三样没读；偏差见第〇节。

## 七、要主 agent 跑的命令 / 复核不了的

- 复跑：第〇节那条命令，工作目录随意，内存上限 4G；跑完与 `out/rerun-1.out` 逐行比（JOIN 一节「用了…秒」的档位可跳）。
- 复核不了（要第二台）：第 3.1 节的 1.6 秒与 `ssh true` 0.20 秒、3.2 节第二台 `runs/` 里 6 个目录的现状、3.3 节的 host-resources 载入集合（草稿里一次手跑，没进 rerun）。
- 实现之后才能跑的：`RUN_WITH_MEMORY_CAP_SELFTEST_ONLY=host-lookup bash scripts/hosts/run-with-memory-cap.sh --selftest`（登记为 light，改过的包装的自证不在副本里起，交主 agent）。

草稿与副本：`/home/fy5090/.local/share/singlefs/work/m3-close-reuse-closure-r2-sonnet/` 里我建的仓副本（约 44M）、试验树、探针目录、复跑工作目录（合计约 1.5G）已全部删掉，目录里只剩写者锁与所有者标记；能留的都在模型目录里。
