# m3-close-reuse-closure-r1 攻方腿（Opus）报告（2026-10-22）

立场：假设「丙加甲」站得住是错的，找反例。只攻正文「四、分工」云端攻方那一行的四格。被判脚本六份的 sha256 开工核过，与 `research/prompts/m3-close-reuse-closure-r1-snapshot/judged-sha256.txt` 逐份对上（模型的 `out/judged-sha256.check` 是原样输出）。一切实验都在草稿目录的仓副本上跑，副本上的数注明是副本。

## 复跑

```
cd <仓根>
bash scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/m3-close-reuse-closure-r1-opus-model/tools/rerun.sh \
  --repo <仓根> --frozen <被判脚本 sha256 对得上的树（仓根即可）> --work <新的草稿目录>
```

rerun.sh 拷一棵树（有 .git 时按 `git ls-files -co --exclude-standard` 取 scripts、.claude、crates 与 Cargo 清单，另拷 `.claude/singlefs-ai-sop`），核六份被判脚本的 sha256，跑 `run-all.sh`（闭包四个入口两条臂、包装 shell 单元两条臂、四个夹具、包装的六个世界、两个 import 世界），最后把 `out/summary.txt` 与模型里那一份逐行比。约 100 秒，不编 Rust、不起 Verus。本腿交回前在仓根上这样跑过一次，末行原样：

```
name=rerun_compare status=same lines=28
```

模型目录每个文件的 sha256（`find … -type f | sort | xargs sha256sum`）：

```
    6c73735f49481b5306b115c2f203b1525182ccde8ca9500be039308997ec6370  research/prompts/m3-close-reuse-closure-r1-opus-model/fixture-root/scripts/hosts/lib.sh
    076cd415020e10e3242ae511225a89eb359be8a89edf32ad141f18c1a5e891c2  research/prompts/m3-close-reuse-closure-r1-opus-model/fixture-root/scripts/hosts/local-cd-literal.sh
    751f42990d5e529f6fc58b75573845ab8b8d1807875e4d95368f479d17ed332c  research/prompts/m3-close-reuse-closure-r1-opus-model/fixture-root/scripts/hosts/local-dir-prefix.sh
    8c68445e4f1ce28570566c8bbf57111dc749e6b40ff0d8226370626b5143e58f  research/prompts/m3-close-reuse-closure-r1-opus-model/fixture-root/scripts/hosts/probe.py
    918f8e5c87221420a37cea3644a65f521a8eddb7f5b48fab72bc2991bd8608d4  research/prompts/m3-close-reuse-closure-r1-opus-model/fixture-root/scripts/hosts/ssh-only.sh
    2a8e6bc211e608c6c287f1a4d3f3c8929a769cbed21a6e1fd478a7be35a4aee6  research/prompts/m3-close-reuse-closure-r1-opus-model/fixture-root/scripts/hosts/through-function.sh
    b541929ced0ff7461a362c5a6dff6485ef3441eb75986c0361b4577be2314d03  research/prompts/m3-close-reuse-closure-r1-opus-model/judged-sha256.txt
    5b01ffb0271f3d7490788a4e990d1f722b8c3bce689dcd6198019bae5c9b6475  research/prompts/m3-close-reuse-closure-r1-opus-model/make-variants.py
    1ea64b3276789b7a5d9cf83338802a58cba7db900f3b205f0acc4b4ebdedb987  research/prompts/m3-close-reuse-closure-r1-opus-model/out/dropped-by-edge-cut-scripts_gate_verus-proof-check_py__check_must_red_worlds.txt
    1ea64b3276789b7a5d9cf83338802a58cba7db900f3b205f0acc4b4ebdedb987  research/prompts/m3-close-reuse-closure-r1-opus-model/out/dropped-by-edge-cut-scripts_gate_verus-proof-check_py__check_proof.txt
    1ea64b3276789b7a5d9cf83338802a58cba7db900f3b205f0acc4b4ebdedb987  research/prompts/m3-close-reuse-closure-r1-opus-model/out/dropped-by-edge-cut-scripts_gate_verus-proof-check_py__check_vacuity.txt
    7a0b6c8d57109c9544659a1e1406187908b5cddc1bc52cad132b2d089d3cb060  research/prompts/m3-close-reuse-closure-r1-opus-model/out/dropped-by-edge-cut-scripts_mutation_broad-mutation_py__Runner_escalate_step.txt
    4bc601aa660211a91040ba6a2789b9e3e6799f73d5089a5e79c43878350cfef3  research/prompts/m3-close-reuse-closure-r1-opus-model/out/fixture-local-cd-literal.txt
    eeba60baa3b2f4c2069e8ee9bd951db43bd06fdd57c0dd3a15d7f5d01f0705f7  research/prompts/m3-close-reuse-closure-r1-opus-model/out/fixture-local-dir-prefix.txt
    bd4e5b4efb9d935142550225e7adaa471786e0e4817a9c91032d1d2b6114ed78  research/prompts/m3-close-reuse-closure-r1-opus-model/out/fixture-ssh-only.txt
    3fd882ce61d8dd0fbb42a69949bdf33a95bb06600fc45232a2e6ea052736c181  research/prompts/m3-close-reuse-closure-r1-opus-model/out/fixture-through-function.txt
    b9e766e603e5bf28a72754be14c5707e6c6cf3bdc939a4930339ae6f3c384f27  research/prompts/m3-close-reuse-closure-r1-opus-model/out/judged-sha256.check
    a8c694f36a9f6141013aa02b9c3b1559fd00251ca84cc5de7a7d407421325b3f  research/prompts/m3-close-reuse-closure-r1-opus-model/out/summary.txt
    5bf0e88e7f4ca171dd6ac60edd660fb6242ed360854911ebc61cf130fdcb361b  research/prompts/m3-close-reuse-closure-r1-opus-model/out/v1-admitted-peer-fits.txt
    8c8f3c65456c3d2ffc82a69ca261122dc146aa0636cd5c2548b4d5fffa500005  research/prompts/m3-close-reuse-closure-r1-opus-model/out/v1-admitted-peer-tail-rlimit.txt
    1ac93a54e12f593659c0f6738da468b41e9bfa99c94b5bc8942422a2a66f7bbd  research/prompts/m3-close-reuse-closure-r1-opus-model/out/v1-tmp-kept.txt
    9df58f22bb3d87e6213928cc2291778807746aa0f21e20e0b58a5a361860192e  research/prompts/m3-close-reuse-closure-r1-opus-model/out/v1-tmp-removed-lookup-off.txt
    a3e9d9b03a0855374f3a1d7c4250ee3ecb7b1e94618bc7e746cb1b1632885fd0  research/prompts/m3-close-reuse-closure-r1-opus-model/out/v1-tmp-removed-nohostlookup.txt
    99c64df17c6440e01f826234c5a841c5bf1e2d93a338651d9aa7a94757b6c265  research/prompts/m3-close-reuse-closure-r1-opus-model/out/v1-tmp-removed.txt
    de2b50ff5fa775ee086f939d1ef0bd872472cce2f04a9573b61b5efef458246e  research/prompts/m3-close-reuse-closure-r1-opus-model/out/v2-import-broken.txt
    be5bbdd17e9f64a043d536944aaa0c4dc29658f18b5871d618f4714b4fb449e0  research/prompts/m3-close-reuse-closure-r1-opus-model/out/v2-import-intact.txt
    0e3a430ee61b34c2bcf4ada17d20cfbd1f6febd8b50ac40b52a3c81ba15f99fe  research/prompts/m3-close-reuse-closure-r1-opus-model/out/variants.txt
    6d83880193fd711689d0847614f07dcdfce9d84f8ac482262ab8021e02d22cd6  research/prompts/m3-close-reuse-closure-r1-opus-model/out/worlds/world-admitted-peer-fits/stderr
    3ae998e08361b0eeb44c306a1bb84735f8d7c57b4127d2269d864eb45d60412e  research/prompts/m3-close-reuse-closure-r1-opus-model/out/worlds/world-admitted-peer-tail-rlimit/stderr
    476d318de9d413909467a2e0431dc4f0dcbdb79df4053bd1bd7245a09de737dd  research/prompts/m3-close-reuse-closure-r1-opus-model/out/worlds/world-import-broken/stderr
    e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  research/prompts/m3-close-reuse-closure-r1-opus-model/out/worlds/world-import-intact/stderr
    774ade8b27aedc2096f0784e0eca054a197320537c40c5b47dd6d13389320f73  research/prompts/m3-close-reuse-closure-r1-opus-model/out/worlds/world-tmp-kept/stderr
    0b4b39ed36afe1cc9fcb1e6954d179e8724b0a7346a5684cbaa4154162a13ace  research/prompts/m3-close-reuse-closure-r1-opus-model/out/worlds/world-tmp-removed-lookup-off/stderr
    684d7205831db1b24fabd040cb16c8a4b7226c0f20195cc1e951dca890c95a74  research/prompts/m3-close-reuse-closure-r1-opus-model/out/worlds/world-tmp-removed-nohostlookup/stderr
    a12342f15aa0ab3d1a5cffd1636632fa720e51f13d7916a7683874ba7d46c347  research/prompts/m3-close-reuse-closure-r1-opus-model/out/worlds/world-tmp-removed/stderr
    be1329255b9754f850804dfe646917b9d77b4bc93c639a5a987c641637c308f5  research/prompts/m3-close-reuse-closure-r1-opus-model/run-all.sh
    7372b3a6c85e943ebb5d6dc75c1c0adb09d237b943a8d5c94b7899e5032a0b20  research/prompts/m3-close-reuse-closure-r1-opus-model/tools/rerun.sh
    f2cd40d50839f38583d298d3703a9946b48be3f94b58f6611b88fdb84792bd7f  research/prompts/m3-close-reuse-closure-r1-opus-model/v1-admitted-then-judged.sh
    6495823340579d2f09006277120617694268c50285966ecd30131829d382a6aa  research/prompts/m3-close-reuse-closure-r1-opus-model/v1-judge-diagnostics.py
    406e208996b4dc011bd61ef24627b0cb7c02612c2f81c7493265a2d7780433f0  research/prompts/m3-close-reuse-closure-r1-opus-model/v1-queue-world.sh
    93d15ec54171c6529bca8fa3cb68937c2924fff70ff933d9b43f099ef2c1e4b9  research/prompts/m3-close-reuse-closure-r1-opus-model/v1-tmpdir-world.sh
    d71c0a13d368c57537ff716e4b6cd96b4d7d56d49631b09771823a27094c0d4a  research/prompts/m3-close-reuse-closure-r1-opus-model/v2-import-side-effect.sh
```

## 各格判定一览

| 格 | 判定 | 一句话 | 量的还是推的 |
|---|---|---|---|
| V1 | 打中，两处 | ① 排队中 TMPDIR 被删，HostLookup 退 252 之前在前台那一趟抛异常，包装从 252 变成 251；关掉查机器的两条臂同一状态都是 252。② HostLookup 打的那一行（含第二台 stderr 的最后一行）进了判法的 diagnostics，必红世界写进判定库的「一句判定」随它变 | ① 量过（副本，真包装）；② 量过（副本，真包装加 must_red_row_problems 函数级） |
| V2 | 打中，两处；另有一处写法走不通 | ① 按调用边排（HostLookup.query → host-resources.py）：四个入口各顺带排掉 lib_long_run.py 40 个单元等，而这一份在 need_of 的 predicted_bytes 里随 host-dispatch.py 的模块体被 import，它坏了包装从起跑变成退 252。② 按函数排写不成：HostLookup 不是闭包单元，它的原文在 shell 函数 admission 的指纹里，要排就把整个排队、记账一起排掉。③ 套现成的 excluded_units 排不掉那条拒算 | ①② 量过（副本）；③ 推的（按代码） |
| V3 | 打中，三处 | ① 第二台上没有「同版本的仓」：PEER_REPOSITORY_DIRECTORY 是每一趟建、跑完删的树副本目录。② ssh 那一支本来就不进闭包（ssh 后面的命令串既不连边也不拒算），今天的拒算只来自本机替身那一支；甲把它折开之后闭包看着完整，第二台真跑的仍是闭包外的代码。③「$目录/<相对路径>」按本机根里那一份认，所依的「拷进去的是它」对常驻目录不成立 | ① 量过（读配置模板）；②③ 量过（夹具） |
| V4 | 打中 | 候选句若写成「只打提示的路不进闭包」，与已定项 17「不静默少收」、规则第 31 行「依赖的函数单元」打架；V1 已量出那条路会改退出码，判据只能收窄到「不改写进判定库的绿」才自洽，而这一分句仍被 V1② 打中 | 推的（条文对读），依据是 V1、V2 量过的数 |

## V1：丙排掉的那条路会不会改判定结局

### V1①：排队中 TMPDIR 被删，退出码从 252 变 251（量过，副本）

机器状态：本机内存排队（账上一笔挡路的要 250M，总上限 256M，这一条要 200M），第二台放得下（假第二台，写法照包装自证的 host_lookup_world）。排队开始那一刻 HostLookup 在后台查过一趟，用过 tempfile，TMPDIR 已被 tempfile 记下；等满之前 TMPDIR 被删（会话暂存目录被清掉是一种来源；盘满时同样卡在这一处，盘满没量）。退 252 之前 finish 在前台再查一趟：

- 「with tempfile.TemporaryFile(mode="w+", encoding="utf-8") as output, tempfile.TemporaryFile(mode="w+", encoding="utf-8") as errors:」（`scripts/hosts/run-with-memory-cap.sh:1007`）在 query 的 try 之外，抛 FileNotFoundError；
- 异常沿 run、finish 抛出 admit（「lookup.finish({"need": need, "slice_occupied": occupied, "slice_total": total_bytes})」（`scripts/hosts/run-with-memory-cap.sh:1162`）），finally 里照旧 cancel，python 退 1；
- 「if [[ $admission_status -ne 0 ]]; then」（`scripts/hosts/run-with-memory-cap.sh:1534`）接着 「report_unavailable "排队那一步退 $admission_status（上面是它的报错），命令一行都没跑" "照上面那几行修；不许绕开排队去跑"」（`scripts/hosts/run-with-memory-cap.sh:1535`），包装退 251。

同一状态四条臂（`out/summary.txt` 原样，`run-all.sh` 里的 v1-tmpdir-world.sh 四次）：

```
name=tmp-kept status=252 ran=no
name=tmp-removed-lookup-off status=252 ran=no
name=tmp-removed-nohostlookup status=252 ran=no
name=tmp-removed status=251 ran=no
name=tmp-removed traceback=FileNotFoundError
```

lookup-off（`RUN_WITH_MEMORY_CAP_HOST_LOOKUP=off`）与 nohostlookup（弄坏开关，排队时不起查机器那一趟，相当于「这条路不存在」）两条臂在同一状态下都是 252；只有查机器那条路照常走时是 251。

判法那一侧随之变的（按代码，推的）：

- 形式化：「if state is not None and exit_code == 252:」（`scripts/gate/verus-proof-check.py:736`）只在 252 时记 QueueRefusal；拿到 251 的那一格不记，同一道后面几格照样起包装、照样排队，出路也换成 251 那一句。
- crates 变异：「touch "$STOP_FLAG"」（`scripts/mutation/mutate.sh:2191`）——251 让整张表停下，后面的变异不判；252 只把这一行记成排不上、接着判下一行。

没量到的同类一处（推的）：排队开始那一刻「self.thread.start()」（`scripts/hosts/run-with-memory-cap.sh:940`）在主线程里起线程，没有 try；任务数撞了 pids.max 或 RLIMIT_NPROC 时 Thread.start 抛 RuntimeError，同样落到 251。`RUN_WITH_MEMORY_CAP_HOST_LOOKUP=off` 管不到它：开关是在 query 里判的（「if switch:」（`scripts/hosts/run-with-memory-cap.sh:991`）），线程照起。我用 `systemd-run --user --scope -p TasksMax=N` 试了 N=2、3、4、5、6、8：N=2 时包装的 preflight 先 fork 失败（退 1），N≥3 时线程起得来、都是 252，所以没造出来。装置是模型里的 v1-queue-world.sh，不在 run-all.sh 里。

### V1②：HostLookup 打的字进了判法的 diagnostics，写进判定库的一句判定随它变（量过，副本）

链条（每一步都是今天的代码）：

- 判法把包装的整份 stderr 当 Verus 的诊断：「"json": None, "diagnostics": stderr + "\n" + stdout, "stderr": stderr}」（`scripts/gate/verus-proof-check.py:731`）。
- HostLookup 往同一份 stderr 打：「say(f"run-with-memory-cap: 没查到别的机器：{problem}")」（`scripts/hosts/run-with-memory-cap.sh:1049`）；problem 里带第二台 stderr 的最后一行：「tail = stderr.strip().splitlines()[-1][-200:].replace("第二台", host_name(machine.number)) if stderr.strip() else ""」（`scripts/hosts/host-resources.py:334`）。
- 必红世界按子串找 rlimit：「rlimit_exhausted = RLIMIT_EXHAUSTED in outcome["diagnostics"]」（`scripts/gate/verus-proof-check.py:1761`），并写进一句判定：「summary = f"落点 {'、'.join(landing) or '无'}；证明错 {len(counted)} 条" + ("，rlimit 用尽夹证明错" if rlimit_exhausted else "")」（`scripts/gate/verus-proof-check.py:1771`）。
- 判绿的那一句原样进库：「failed_record = self.store.record(item, components, summary, run=self.run, details=details)」（`scripts/gate/verus-proof-check.py:1471`），库里那一行是「"verdict": "green", "summary": summary, "details": dict(details or {}), "run": run,」（`scripts/crash/lib_verdict_library.py:475`）；成功句再按它数：「rlimit_with_proof_error = sum(1 for summary in judging.summaries.values() if "rlimit 用尽夹证明错" in summary)」（`scripts/gate/verus-proof-check.py:1865`）。

装置：真包装先排队、查机器开着，3 秒后挡路的假账删掉，这一条放行；放行后的「命令」照 Verus 的样子打一条证明错与 verification-results JSON（不起 Verus）；包装的 stdout 与 stderr 原样交 v1-judge-diagnostics.py，照 run_verus 的拼法调副本里 verus-proof-check.py 的 must_red_row_problems（只 import，不走 __main__）。两次只差假第二台 stderr 的最后一行，Verus 那一半逐字相同（`out/summary.txt` 原样）：

```
name=admitted-peer-fits host_lookup_lines_in_diagnostics=1 problems=[] summary=落点 无；证明错 1 条 wrapper_status=0
name=admitted-peer-tail-rlimit host_lookup_lines_in_diagnostics=1 problems=[] summary=落点 无；证明错 1 条，rlimit 用尽夹证明错 wrapper_status=0
```

两次都判绿、都会记进库，记下的一句判定不同。今天这一格还进不了库：判法单元拒算时库不写：「有拒算的不写（交原因）。」（`scripts/crash/lib_verdict_library.py:456`），丙让拒算消失之后这条通道才通。

假第二台最后一行写成 rlimit 那句话是我造的；真第二台的 stderr 是 ssh 与第二台上那份程序打的，什么样的字会出现没量过。要紧的是通道：判法分不出哪几行是包装打的。今天 HostLookup 打的行以 `run-with-memory-cap:` 或 `  →` 起头，碰不到「if not line.startswith("error"):」（`scripts/gate/verus-proof-check.py:762`）那一条，只碰得到 rlimit 那种按子串的判法；以后改了 host-resources.py、host-dispatch.py 里给人看的字（judge_host 的 why、local_only_because），在丙之下都不进键。

### 「只打提示」用什么自证钉得住（推的，被攻过零轮）

只在包装那一侧证不够，要两侧各一格：

1. 包装：查机器那条路上的任何异常（TemporaryFile、Thread.start、导入 core_usage）都接住，退出码与 lookup-off 臂相同。自证照 V1① 的世界造「排队中删 TMPDIR」，核退出码等于 252；弄坏开关是让 HostLookup 把异常抛出去（例 `RUN_WITH_MEMORY_CAP_BREAK=hostlookupraises`），这一格应当红在 251。
2. 判法：verus-proof-check 拼 diagnostics 之前去掉包装自己打的行（`run-with-memory-cap:` 起头的行与查机器打的 `  → …此刻放得下` 行），或者 HostLookup 改写到告警记录、不写 stderr。自证往包装 stderr 里塞一行带 RLIMIT_EXHAUSTED 的查机器提示，核一句判定不变；弄坏开关是不去掉，这一格应当红。

## V2：排除规则三种写法，各顺带排掉了什么

### 先看今天闭包里那条路长什么样（量过，副本）

- 包装里的排队代码是喂给 `python3 /dev/fd/3` 的内联脚本。照规则「python 的切成单元（不进闭包，穿过它连下去」（`scripts/gate/lib_function_units.py:135`），HostLookup.query、admit、need_of 都不是闭包成员，只在 `--explain` 的 via 行里当来源出现；它们的原文算在 shell 函数 `admission` 的指纹里。把 HostLookup.run 打的那一句改几个字，`admission` 的指纹就变（`out/summary.txt` 原样）：

```
name=wrapper_shell_units arm=full admission=9c0d1ca99464b9a1c4eb0f76908f91a7c2bd5a2394ddc4d195305a827c6530c5 hostlookup_units=0
name=wrapper_shell_units arm=hostlookup-text admission=4e4b872a9c9c0903f79cc4017b7e35b0d11fc1fa5437201c48bfb650e2a81093 hostlookup_units=0
```

- 经包装起、判法单元闭包碰得到包装的入口，我逐个收了闭包：verus-proof-check.py 的三个判法入口，加上 broad-mutation.py 的 `Runner.escalate_step`（它的闭包同样到包装、同样拒在 `scripts/hosts/peer-host-lib.sh:76`，正文没列这一个）。broad-mutation.py 的 `process_item`、`gpu_pass_command`，crates-mutation-rows.py 的 `verdict_of_run`，e165-pin-red-examples.py 的 `replay` 我也收了，闭包都碰不到包装（各 93、27、51、939 个单元，含 run-with-memory-cap.sh 的 0 个）：丙管不到它们。这四个数量过一次，没进 rerun.sh。

### 写法一：按调用边排（HostLookup.query → host-resources.py 那一条）——顺带排掉了 need_of 真会执行的代码（量过，副本）

在副本里只把那一行起 host-resources.py 的命令换成一条字面的内联 python（make-variants.py 的 edge-cut），别的边不动，再收四个入口的闭包（`out/summary.txt` 原样，check_proof 与 escalate_step 两个入口；vacuity 与 must-red 同形，见 summary）：

```
name=closure arm=full entry=scripts/gate/verus-proof-check.py::check_proof rc=4 units=1577 lib_long_run_units=40 host_dispatch_predict=1 refusals=inline-script-not-literal scripts/hosts/peer-host-lib.sh:76,
name=closure arm=edge-cut entry=scripts/gate/verus-proof-check.py::check_proof rc=0 units=1230 lib_long_run_units=0 host_dispatch_predict=1 refusals=none
name=dropped_by_edge_cut entry=scripts/gate/verus-proof-check.py::check_proof total=347 by_file=.claude/hooks/lib_heavy_tests.py:18,.claude/hooks/lib_long_run.py:40,.claude/hooks/lib_shell_words.py:5,.claude/hooks/lib_work_root.py:1,.claude/scripts/project_preflight.py:1,.claude/singlefs-ai-sop/scripts/preflight.py:3,.claude/singlefs-ai-sop/scripts/proc.py:1,scripts/crash/crash-amplification-lib.sh:26,scripts/crash/crash-point-replay-shard-configuration-check.sh:20,scripts/gate/lib_function_units.py:1,scripts/hosts/host-dispatch.py:41,scripts/hosts/host-resources.py:75,scripts/hosts/memory_peaks_lib.py:9,scripts/hosts/peer-host-lib.sh:5,scripts/hosts/rocksdb-prebuilt.py:101,
name=closure arm=full entry=scripts/mutation/broad-mutation.py::Runner.escalate_step rc=4 units=790 lib_long_run_units=40 host_dispatch_predict=1 refusals=inline-script-not-literal scripts/hosts/peer-host-lib.sh:76,
name=closure arm=edge-cut entry=scripts/mutation/broad-mutation.py::Runner.escalate_step rc=0 units=434 lib_long_run_units=0 host_dispatch_predict=1 refusals=none
```

拒算没了，host-dispatch.py 的 predict 留着（走 need_of 那一路），这两样符合丙的本意。但 lib_long_run.py 的 40 个单元（连同它 import 的 lib_heavy_tests.py、lib_shell_words.py）全排掉了，而它在不查机器的那一路上照样执行：

- need_of 在峰值表里没有这条命令时调 predicted_bytes，按文件路径整份执行 host-dispatch.py：「spec.loader.exec_module(dispatch)」（`scripts/hosts/run-with-memory-cap.sh:857`）；
- host-dispatch.py 的模块体里：「import lib_long_run as long_run_lib  # noqa: E402   长活声明的判法（文件头 ⓪），与 detached-run.sh 共用」（`scripts/hosts/host-dispatch.py:141`）；
- 闭包规则只在「那个模块有单元进闭包时」连模块级语句：「模块级语句只在那个模块有单元进闭包时连进来（import 时的副作用照算），模块被别的进程起时连的是 main-guard 与 module-body。」（`scripts/gate/lib_function_units.py:262`）。predict 用不到 long_run_lib 这个名字，所以 import 那一条单元只经 HostLookup 那一路的 long_run_check 进闭包；边一剪，它就没了。

在副本里让 lib_long_run.py 一 import 就抛异常（make-variants.py 的 broken-import），同一条包装命令（总上限 9G、上限 10G、峰值表里没有它、不查机器）：

```
name=import-broken status=252 ran=no
name=import-intact status=0 ran=yes
```

预测算不出时按上限 10G 记账，比总上限大，当场退 252；算得出时按 8G 记账、起跑。退出码与「这一处判没判」都变了，而写法一之下 lib_long_run.py 不在这几个入口的键里。病根是闭包规则按名字连 import（今天它在键里也是碰巧经 HostLookup 那一路进来的），写法一把它从「碰巧在」变成「不在」。memory_peaks_lib.py 被排掉的 9 个是模块级赋值与 import 单元，包装 import 这一份时同样会执行（推的，没量）。

### 写法二：按函数排（排 HostLookup）——写不成（量过，副本）

HostLookup 不是闭包单元（上面 `hostlookup_units=0`），可点名排除的最小单元是 shell 函数 `admission`。排它就是把 admit、need_of、排队判法、记账、峰值表这些全排出键：V2 写法一里那个 252 与 0 的分岔、V1① 里 252 与 251 的分岔全落在这一个单元里。要按函数排，得先把内联脚本切成单元（今天的规则是穿过它连下去、它自己不进闭包），那是改闭包算法，不是写一条排除规则。

### 写法三：套现成的 excluded_units / excluded_files——前者排不掉拒算，后者是按文件排（推的，按代码）

lib_verdict_library.py 的 key 已有两样排除。单元那一样只挡成员：「if identity in seen or unit.file in excluded_files or f"{unit.file}::{unit.name}" in excluded_units:」（`scripts/crash/lib_verdict_library.py:200`）；拒算只按文件滤：「merged.refusals.update(refusal for refusal in closure.refusals if refusal.file not in excluded_files)」（`scripts/crash/lib_verdict_library.py:205`）。点名 `run-with-memory-cap.sh::HostLookup.query` 既点不中（不是单元），也挡不住落在 peer-host-lib.sh 的那条拒算；点名 peer-host-lib.sh 整份文件，拒算是没了，host-resources.py、host-dispatch.py 的 judge_host 照旧在键里，而且是按文件排。

### 写法四：按包装的开关排（判法给包装设 `RUN_WITH_MEMORY_CAP_HOST_LOOKUP=off`）

V1① 的世界里 lookup-off 那条臂是 252，运行期确实不走 TemporaryFile 那一处（量过）。但三处要另算：闭包算法不读环境变量，「设了开关就不算这条路」要由排除规则声明，算法核不了每个入口真的设了；开关只在 query 里判，begin 照样起线程（V1① 最后一段，推的）；判法关掉查机器，包装文件头那一段记的用户 2026-10-20 要的排队时找别的机器（`scripts/hosts/run-with-memory-cap.sh:103`），在门禁排队时就没了（那一行指的调度记录这条腿没去读）。

### V2 小结

| 写法 | 拒算消不消 | 顺带排掉的、真会改结局的 | 依据 |
|---|---|---|---|
| 按调用边 | 消 | lib_long_run.py 等（包装退 0 还是 252） | 量过（副本） |
| 按函数 | 写不成 | 要排就得排 admission 整个单元 | 量过（副本） |
| excluded_units | 不消 | — | 推的 |
| excluded_files=peer-host-lib.sh | 消 | 不排 host-resources，是按文件排 | 推的 |
| 按开关 | 闭包里不消，要另立声明 | Thread.start 那一处仍在 | 推的 |

## V3：甲之后键漏掉第二台版本差、或错误折开的地方

### V3①：第二台上没有「同版本的仓」这个东西（量过，读配置模板）

甲要求「第二台有同版本的仓」。今天配置里第二台唯一的目录键是这样定义的：「# 第二台上放这一趟树副本的专用目录（绝对路径）：每一趟在它下面建一个子目录放树与编译目录、跑完删掉；」（`multi-host.env.example:11`）。它不是常驻的仓，是每一趟建、跑完删的副本根。已有的跨机判定都是现拷现核：crates 变异的双机驱动把被判的树拷过去，再在两台各算一遍判法单元指纹，对不上就停——「peer_base="$(peer_host_run "$peer_run_directory" "python3 scripts/mutation/crates-mutation-rows.py judging-units .")" \」（`scripts/mutation/mutation-shard-run.sh:371`），「[[ "$local_base" == "$peer_base" ]] \」（`scripts/mutation/mutation-shard-run.sh:375`）。

所以甲要么新加一份常驻在第二台的仓（谁更新、更新落后时怎么办，今天没有任何一处管），要么每次查机器都现拷。版本差靠什么挡住，甲的题面没写。核的代价不大：在副本上 `lib_function_units.py key . scripts/hosts/host-resources.py::probe_this_host` 两次各 0.15 秒、7 个单元（量过，本机，副本）。查机器给第二台的时限是 8 秒（「HOST_LOOKUP_PEER_TIMEOUT_SECONDS = 8    # 交给 host-resources.py 的每台时限（HOST_RESOURCES_HOST_TIMEOUT_SECONDS）」（`scripts/hosts/run-with-memory-cap.sh:776`）），放得下一次两台各算一把键再比（推的：第二台上的耗时没量）。

### V3②：ssh 那一支本来就不进闭包，甲折开替身那一支之后闭包看着完整（量过，夹具）

peer_host_run 有两支：本机替身「PATH="$PEER_CARGO_BIN_DIRECTORY:$PATH" bash -c "$2") 2> >(peer_host_redact >&2) | cat」（`scripts/hosts/peer-host-lib.sh:81`），真第二台「ssh -o BatchMode=yes "$PEER_SSH_HOST" "cd $(printf '%q' "$1") && PATH=$(printf '%q' "$PEER_CARGO_BIN_DIRECTORY"):\$PATH bash -c $(printf '%q' "$2")" \」（`scripts/hosts/peer-host-lib.sh:84`）。夹具（模型的 fixture-root，四份小脚本，一个 probe.py）照这两支写，用今天的 lib_function_units.py 收闭包（`out/summary.txt` 原样）：

```
name=fixture case=through-function rc=4 probe_units=0 refusals=inline-script-not-literal scripts/hosts/lib.sh:4,
name=fixture case=ssh-only rc=0 probe_units=0 refusals=
name=fixture case=local-cd-literal rc=4 probe_units=1 refusals=shell-path-unfolded scripts/hosts/local-cd-literal.sh:2,
name=fixture case=local-dir-prefix rc=0 probe_units=4 refusals=
```

- through-function（照 peer_host_run 两支写的函数，调用处给字面串）：拒在替身那一支的 `bash -c "$2"`，就是今天那条拒算。
- ssh-only（ssh 后面跟字面命令串，串里起 probe.py）：退 0，既不连到 probe.py，也不拒算。ssh 的参数在算法眼里只是一个参数。

所以今天那条拒算其实是替身那一支「替」真第二台挨的。甲教算法把 `bash -c "$N"` 在调用处折开以后，替身那一支连到本机根里的脚本，拒算消失；ssh 那一支照旧什么都不收。闭包从「拒算、照跑」变成「键算得出」，第二台上真执行的那份脚本仍在键外，且看不出来。这是错误折开里最直接的一种：被折开的那一支，不是运行期真走的那一支。

### V3③：「$目录/<相对路径>」按本机根里那一份认，理由对第二台不成立（量过，夹具；后果是推的）

- local-dir-prefix（`python3 "$PEER_REPOSITORY_DIRECTORY/scripts/hosts/probe.py" 0.2`）：退 0，连到根里的 probe.py（4 个单元）。依据是那一条折叠规则，第 250 行写「临时树、仓副本、worktree 里的…按根里同一相对路径那一份认」（`scripts/gate/lib_function_units.py:250`，行里有引号，不整句抄），理由在下一行：「拷进去的是它，替身只会让闭包多收、不会少收；尾巴在根里不在的照旧拒算。」（`scripts/gate/lib_function_units.py:251`）。第二台的目录不是这一趟拷进去的，这个理由不成立：键里是本机那一版，第二台跑的是它自己那一版。
- local-cd-literal（先 cd 到第二台目录，再用相对路径起）：今天拒 shell-path-unfolded。甲的调用处若这样写（`peer_host_run "$PEER_REPOSITORY_DIRECTORY" "python3 scripts/hosts/host-resources.py …"`，与 multi-host-run.sh 第 310 行同形），折开 `$2` 之后还要过 `cd "$1"` 这一关，拒算不消；若写成「$1/scripts/…」那一种，拒算消了，又落进上一条。

两种写法：一种甲拿不掉拒算，一种拿掉了但键是错的。这一点只在甲的实现里才分得清，我没有甲的实现，只量了今天的规则在这两种写法上怎么判。

### V3 里「键怎么算第二台那一侧」的反例（推的）

丙成立的前提下，HostLookup 这一路整个不进键，第二台的版本差改不了写进库的内容，V3 对形式化那三格没有判定上的后果。后果落在甲的折叠规则上，它对每个调用 peer_host_run 的入口都生效：mutation-shard-run.sh、crash-point-replay-shard-run.sh、multi-host-run.sh 以后进了哪条判定的闭包，第二台跑的被判命令都会按 V3③ 那条规则连到本机根里的那一份，版本一致只靠运行期那一步现拷加两边比键（mutation-shard-run.sh ④ 那种）。那一步不在闭包里，算法核不了它在不在。

## V4：候选句与已定条款打架的地方（推的，条文对读；依据是 V1–V3 量过的数）

下面引已定项 17 与规则第 31 行时，第 432、433 行与规则第 31 行整行超过 cite-check 一处引文 400 字的上限，只抄相关那一句（行号是那份文件自己的）；第 434、435 行整行抄。

1. **「只打提示的路不进闭包」与「不静默少收」。** 「- 收不全（动态分派、宏、`include_str!(concat!…)` 认不出）的那一份拒算键、照跑，不静默少收；每把键连同组成它的单元清单落盘，作废时报得出是哪几个单元变了。」（`.claude/kb/decisions/13-验证路线.md:434`）。丙不是收不全，是看得见而不收。候选句不加「排除的边连同理由随键落盘、作废时一起报」，就是静默少收；V2① 那 347 个单元（含 lib_long_run.py）就是这样不声不响掉出去的。
2. **「沿调用收」开了例外，例外的判据要能被核。** 「函数单元从这份判定的入口沿调用与引用收，连同引用的常量、类型、静态项与宏展开一起算；wgsl 按 fn 切。」（`.claude/kb/decisions/13-验证路线.md:432`）。候选句若写「不改判定结局的路不收」，V1① 已量出那条路改退出码（252 变 251）；收窄到「不改写进判定库的内容」，V1② 又量出写进库的一句判定会变。只有先让判法不读包装自己打的行（V1 最后一节第 2 格），这句判据才自洽。判据要写成能被自证核的那一种（包装退出码与关掉查机器那条臂相同、判法 diagnostics 不含包装行），不写「只打提示」这类核不了的话。
3. **「依赖的函数单元」的「依赖」。** 规则第 31 行：「键只由这条判定依赖的函数单元各自去注释后的指纹、闭包真引到的 cfg 原子与它们的真值、工具链、解析出的依赖版本与它自己的内容输入组成」（`.claude/rules/implementation-workflow.md:31`）。查机器那条路跑在判法起的包装进程里，按字面算「依赖」。丙要么改这一句，要么给「依赖」下定义（只算能改写进库内容的）；不改就是规则与实现两样说法。
4. **按文件排与「文件级」。** 「键过窄（引用的常量、类型、宏漏收）、库放在会丢的位置、改名丢库、两份库重复判同一批，同样算违反。」（`.claude/kb/decisions/13-验证路线.md:433`）同一行的前半句把「文件级、包级、整表级的指纹或闭包」列在不许之列。V2 写法三用 excluded_files 排 peer-host-lib.sh 是文件级；写法一漏掉 lib_long_run.py 是「键过窄」。候选句要写清排的是哪一条调用边，并要求被排那一侧的模块在别的路上被 import 时照收（V2① 的修法）。
5. **甲的「核版本」不能拿提交或整文件当依据。** 第 433 行开头：「一律不进键、也不许当复用依据：按提交、HEAD 或树算的改动范围；整文件原文；文件级、包级、整表级的指纹或闭包」（`.claude/kb/decisions/13-验证路线.md:433`）。若候选句写「两台仓的 HEAD 相同才信第二台」或「比 host-resources.py 整份 sha256」，就撞这一条（丙之下它不是复用依据，只是信不信提示，可以争；但写进条文时要写明它不进键）。已有的样子是两边各算判法单元键再比（V3①）。
6. **「第二台判出的并进本机那一份」与甲。** 「- 每类判定一份库，固定在持久目录，不随一趟、不随判法换目录；仓根也不是库的输入：崩溃判定库的目录由环境变量 `SINGLEFS_CRASH_AMPLIFICATION_LIBRARY` 给时只收绝对路径，相对路径拒绝；全量开关只写不读、现判的照键写回同一份库；第二台判出的并进本机那一份。」（`.claude/kb/decisions/13-验证路线.md:435`）。甲若把「第二台跑同一份脚本，键按本机算」写成条文，V3②③ 下第二台的版本不进键，合并进本机库的那几条判定就带着别的版本的判法；条文要连着写运行期的两边比键（mutation-shard-run.sh ④），并写明这一步不在闭包里、闭包核不了它。

### 自证与门禁 code-tooling 的 reuse-selftests 格要加什么（推的，被攻过零轮）

reuse-selftests 格认的是登记表里每份带判定库的脚本的复用三问（`.claude/gate.d/code-tooling.sh` 第 24 行的格说明）。包装不在那张表里，包装那一侧的自证它看不见；判法那一侧加进 verus-proof-check.py 的 `--selftest` 才会被这一格带上。

| 加的格 | 在哪份脚本的自证里 | 绿的条件 | 弄坏开关（让它红） | 今天 / 丙之后 |
|---|---|---|---|---|
| 查机器那条路抛异常不改退出码 | run-with-memory-cap.sh | V1① 的世界（排队中删 TMPDIR）退 252，与关掉查机器那条臂相同 | HostLookup 把异常抛出去 | 今天红（量过：251） |
| 判法不读包装自己打的行 | verus-proof-check.py | 往包装 stderr 塞一行带 RLIMIT_EXHAUSTED 的查机器提示，必红世界的一句判定不变 | 不去掉包装行 | 今天红（量过：多了「rlimit 用尽夹证明错」） |
| 被排的那一路改了，照复用 | verus-proof-check.py | 只改 host-resources.py 的 judge_host 给人看的字，三格都复用 | 键照收那一路 | 丙之后应绿（没实现，没跑） |
| 别的路 import 的模块改了，照现判 | verus-proof-check.py | 只改 lib_long_run.py 模块级一行，三格都现判 | 按名字连 import（今天的规则） | 写法一之下红（量过：边一剪，lib_long_run.py 0 个单元进闭包） |

## 打中之后的四句

| 打中 | 分不分辨臂 | 被判的系统当时看不看得到 | 满足判据字面的哪一个分句 | 跑前给的改法在这几格上还中不中 |
|---|---|---|---|---|
| V1①（252 变 251） | 分辨「丙那条路只打提示」成不成立；不分辨甲（甲不碰这一处） | 看得到：包装自己接得住异常，判法拿到的是不同的退出码 | 正文 V1 格「会改变判法入口的判定结局（退出码、判红判绿、写进判定库的内容）」的「退出码」；不满足「写进判定库的内容」 | 丙（排除规则）碰不到：它只改键，不改运行期；甲也碰不到。要另加 HostLookup 接住异常（推的） |
| V1②（一句判定随第二台 stderr 变） | 分辨丙的前提；不分辨甲 | 判法看不到：拼起来的 diagnostics 里分不出哪几行是包装打的（判别子观测不到，要改判法让它看得到） | 「写进判定库的内容」 | 丙碰不到（今天这条路因拒算不进库，丙让它进库）；甲碰不到 |
| V2①（边剪掉 lib_long_run.py） | 分辨丙的三种写法：按边的写法中，按函数的写法写不成，按开关的写法不中这一格 | 闭包算法看不到：它按名字连 import，运行期的模块体执行它不认 | V2 格「写法会不会顺带把真会改结局的代码也排出闭包」 | 按边：中；先补「被 import 的模块整份照收」再剪边：不中（推的） |
| V3②③（ssh 那一支不进闭包、按本机根认） | 分辨甲的调用处两种写法；对丙不分辨 | 闭包算法看不到 ssh 的命令串，也分不出常驻目录与这一趟的副本 | V3 格「第二台的仓版本不同怎么办」「会不会让别的现在正确拒算的地方被错误折开」 | 甲照题面做：中；甲加运行期两边比键：版本差挡得住，键仍不含第二台（推的） |

## 我提的改法各修哪一格（只在我的模型上量过、被攻过零轮）

| 改法 | V1① | V1② | V2① | V3② | V3③ |
|---|---|---|---|---|---|
| HostLookup 的 begin、finish、run 外面接住一切异常（SystemExit 除外），只打一行原因 | 修（推的：没实现；对照臂 lookup-off 量过是 252） | 不修 | 不修 | 不修 | 不修 |
| 判法拼 diagnostics 前去掉包装自己打的行，或 HostLookup 只写告警记录、不写 stderr | 不修 | 修（推的：没实现；问题本身量过） | 不修 | 不修 | 不修 |
| 闭包算法：模块被 import（含 exec_module 整份执行）时，模块体与它 import 的模块整份照收，不按名字挑 | 不修 | 不修 | 修（推的：没实现；问题本身量过） | 不修 | 不修 |
| 丙写成按调用边排，被排的边与理由随键落盘 | 不修 | 不修 | 只有与上一行一起才不中 | 不修 | 不修 |
| 甲的调用处：ssh 那一支与替身那一支一起认，ssh 命令串里起的仓内脚本照拒算；第二台跑之前现拷、两边比判法单元键，对不上就当「没读到」 | 不修 | 不修 | 不修 | 修（推的） | 修（推的） |

## 没打中的形状

- **HostLookup 把命令改送第二台**：没有。文件头写「#   不改排队本身：不自动改去第二台（两台结论口径不一样时混用分不清），退出码照旧。」（`scripts/hosts/run-with-memory-cap.sh:115`），代码里也只打一行、写告警记录；放行之后跑的是本机那一条。
- **HostLookup 往判法的 stdout 写字（让 JSON 解析失败）**：没有。它的子进程 stdout、stderr 进临时文件，自己只往 stderr 打；Verus 的 JSON 从 stdout 读。
- **HostLookup 打出以 `error` 起头的行**：今天的四种写法（没查到、放不下、放得下、告警写不进）都以 `run-with-memory-cap:` 或 `  →`、`  !` 起头；第二台 stderr 的最后一行只取一行（host-resources.py 第 334 行），带不进换行。只碰得到按子串判的 rlimit 那一条（V1②）。
- **HostLookup 让一个本该红的项判绿**：没造出来。证明格判绿只看 Verus 的 JSON；必红世界判绿要 JSON 里 errors 至少 1，包装打的字改不了 JSON；空真那一格同理（按代码，推的）。V1② 改的是绿项记下的那一句，不是红绿。
- **cancel 与排队锁死锁**：admit 放行时 return 已经出了 admission_lock 的 with，cancel 在 finally 里、锁外；后台那一趟的 take_turn 拿锁只读写一份小 JSON（按代码，推的）。
- **用任务数上限造 Thread.start 失败**：TasksMax 2 到 8 试了六档，见 V1① 末段，没造出来。
- **甲错误折开别的 `bash -c "$N"`**：全仓（scripts、.claude 下的 .sh、.py，fixtures 除外）`bash -c "$<数字或 @ *>"` 共 6 处：peer-host-lib.sh 第 81 行、multi-host-run.sh 第 999 行（自证里的场景函数，调用处都给字面串，折开是对的），其余 4 处在假 ssh 的字面正文里或 lib_function_units.py 的夹具里。没找到第二个会被错误折开的函数；被错误折开的是 peer_host_run 自己的 ssh 那一支（V3②）。
- **判定入口里另有碰得到包装、而正文没列的**：找到一个，broad-mutation.py 的 `Runner.escalate_step`（V2 开头），已算进 V2 的量；别的四个碰不到。全仓经包装起的脚本有 76 份（`grep -rlF run-with-memory-cap.sh scripts .claude/gate.d`，去掉夹具），我只收了各判法登记的判法入口的闭包，没逐份收那 76 份。

## 这条腿自己的限度

- 副本是开工时按 `git ls-files -co --exclude-standard` 拷的（另拷 `.claude/singlefs-ai-sop`），被判六份的 sha256 对得上；没核的那几份（host-dispatch.py 之外的 lib_long_run.py 等）是开工那一刻主工作区的样子。rsync 有两个登记了却不在盘上的文件没拷到（`.claude/gate.d/replay-comparison-units-lag.tsv` 与它的夹具副本），与本题无关。
- V1① 的「TMPDIR 被删」是我挑的一种让 TemporaryFile 失败的状态；盘满、句柄用尽没量。V1② 第二台 stderr 那一行是我写的。两者证的是通道存在，不是常见程度。
- V1②、V2① 的判法一侧只调到 must_red_row_problems 这一个函数，没起 Verus、没跑 verus-proof-check.py 的任何一种用法（形式化是重型，按定义不跑）。
- 甲没有实现，V3 只量了今天的规则在甲会用到的几种写法上怎么判。
- 单次抽样：这一轮只有这一条攻方腿，没打中的那几条不支撑「没有」（`.claude/rules/three-way-inference.md` 那一节「一条腿只抽一次样不算一次观测——否定结论尤其不算」）。

## 没做什么

- 没判正推、辩方那几格；没替主 agent 采纳；副本上量出的数不算入库装置上的数。
- 没跑任何重型测试、门禁阶段与 lint；没编 Rust。跑的只有 lib_function_units.py 的 closure、key、dump（只读根），真内存包装（自己建、自己删的 slice），verus-proof-check.py 只 import 后调一个纯函数。
- 原型扫描、崩溃枚举不在这一题里，没有调 `enumerate_crash_states`，状态数为 0。
- 没读禁读清单里的三样。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-close-reuse-closure-r1-opus` 里的仓副本、变体树与复跑目录交回前删掉；在副本里做过一次 `git init`（在草稿目录，不是本仓）。

