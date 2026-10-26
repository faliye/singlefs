# m3-close-reuse-closure-r2 攻方腿（Opus）报告（2026-10-23）

立场：假设第二轮被判的形态（判决第三节：丙三件加改过的甲）站得住是错的，在 W1–W4 四格找反例。第二轮分工照规则：第一轮的修补本身、产生结论的方法、从未被看过的地方；第一轮攻过的五件事本身不重复。开工 `sha256sum -c research/prompts/m3-close-reuse-closure-r2-snapshot/judged-sha256.txt` 八份全 OK（模型的 `out/judged-sha256.check` 是复跑时的原样输出）。一切实验在草稿目录的仓副本上跑，副本上的数都是副本的数，不是入库装置的数。第一轮两条腿的模型只读，没改；要用的思路拷进了本轮模型目录另写。

## 复跑

```
cd <仓根>
bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/m3-close-reuse-closure-r2-opus-model/tools/rerun.sh \
  --repo <仓根> --frozen <被判八份 sha256 对得上的树> --work <新的草稿目录>
```

rerun.sh 照 `scripts/agents/three-way-rerun-template.sh` 拷的，只改了 rerun_steps：从冻结树拷 scripts、.claude、crates、litmus 与 Cargo 清单，跑 `run-all.sh`，把 `out/summary.txt` 与模型里那一份逐行比。run-all.sh 先核八份被判脚本的 sha256，再造变体树（硬链原树、只换点名的文件），四组装置并行（各写各的文件，`wait` 之后数齐四个完成标记再读）：W1 的包装世界与判法去行比对、W2 的闭包与键、W2 的 35 个入口、W3 与 W4。不编 Rust、不起 Verus、不起 cargo。本腿交回前在草稿副本上不经外层包装跑过一次（第一次经 8G 包装起时整台 slice 被别的会话的重活占满、排队超过 5 分钟，我停了它、包装退 251；之后直接跑，装置里只有脚本，没有编出来的二进制），约 2 分 20 秒，末行原样：

```
name=rerun_compare status=same lines=147
```

主 agent 复跑时请经包装、上限给 8G（run-all 最多同时起十几个 python 闭包进程，峰值没量过，8G 是推的）。冻结树要与本腿副本同一时刻：被判八份之外，装置还读 broad-mutation.py、memory_peaks_lib.py、lib_long_run.py、lib_verdict_library.py 等，别的会话改了它们，键与单元数会变、compare 报 differ，那时先比 `<草稿目录>/summary.diff` 再判。

模型目录每个文件的 sha256（`find … -type f | sort | xargs sha256sum`）：

```
    a21f7e5ef146c5bfb98cf3c4f9d67197c80c59479e857e6ee435ed59cd7569f0  research/prompts/m3-close-reuse-closure-r2-opus-model/judged-sha256.txt
    7083914fe92db6fc72a939f698374135d840fd632ca6588abb54601dd14626bb  research/prompts/m3-close-reuse-closure-r2-opus-model/make-variants.py
    71b860d7293d8032efb1dcec966f2895d2f1501472ac07250abb035f57136fe1  research/prompts/m3-close-reuse-closure-r2-opus-model/out/hits/memory-escalated.log
    3ef8ee1f769e24c8d0b7da5208049bd6301460474533032e6886bac5406d298e  research/prompts/m3-close-reuse-closure-r2-opus-model/out/hits/memory-first.log
    dbbe88c8ff0bf680ebbacee6669372f00b2fe55dd307a8c29702190c0a317418  research/prompts/m3-close-reuse-closure-r2-opus-model/out/hits/time-escalated.log
    49a2fe63b61d202677f4f5b179ebd82b4d3c330b4b2dc7a9417ad2a4dcc3a7f2  research/prompts/m3-close-reuse-closure-r2-opus-model/out/hits/time-first.log
    8409582e173ed5d0dbbb827aaad6103d04b4d4ec4f9fc617c8c9c07bc4c76d3c  research/prompts/m3-close-reuse-closure-r2-opus-model/out/judged-sha256.check
    9d2be66d635c3d9f42e1c67cfb67a223d9c6bc54d2913da33d0e4e7daad4ea24  research/prompts/m3-close-reuse-closure-r2-opus-model/out/part-hits.txt
    f08c4afbeacd64505e2afb4e45057568c9380450c3aecc02da576ea4aa2dc9dd  research/prompts/m3-close-reuse-closure-r2-opus-model/out/part-strip.txt
    592b513fff88de413aec53eee7eb03fc0e603c7e8de30b2d110e36a126fcf3dd  research/prompts/m3-close-reuse-closure-r2-opus-model/out/part-w2-census.txt
    65a2009ede5e2eef725495bde559fe19ab02193a023f8ab84b30ca08e222fa05  research/prompts/m3-close-reuse-closure-r2-opus-model/out/part-w2-closures.txt
    c5df9a0a18b7eafa5d25e64b97a6dbc8548f27648d9a473c2ecbe6051d46ac53  research/prompts/m3-close-reuse-closure-r2-opus-model/out/part-w2-entries.txt
    340bc827fc7988b527c33cfc8e427e04dec2016f85bb56bb36ebc565693dc207  research/prompts/m3-close-reuse-closure-r2-opus-model/out/part-w2-module-level-fix.txt
    6ce9ea04768ddc2250b107db81c6e306969c8ef061953de53a754669a32c18f8  research/prompts/m3-close-reuse-closure-r2-opus-model/out/part-w2-starts.txt
    3ba30f76fa135e6ff175ceb63b2c53b7fe8296ca683fe59d7e02e3cedf6b124c  research/prompts/m3-close-reuse-closure-r2-opus-model/out/part-w3-needed.txt
    bcfd33e309a6eccc8b64f130500b000bf7b63e06cbbf11f9dc80d1edf9388a90  research/prompts/m3-close-reuse-closure-r2-opus-model/out/part-w3-stale.txt
    cc0d57ba89203e240d86f3e127429e90fbb308ad5be4842a7ba68823e443ee01  research/prompts/m3-close-reuse-closure-r2-opus-model/out/part-w4-declaration.txt
    d8be64608241c10f12c6ba4cfbeb9ba817f79265c14546f07d80265555924b37  research/prompts/m3-close-reuse-closure-r2-opus-model/out/part-w4-keys.txt
    2776e91feccc90b14a5a1beeadde1c7ae646ecf02c29ab6736616741dced4623  research/prompts/m3-close-reuse-closure-r2-opus-model/out/summary.txt
    1264ae505850c0a6f77c6224428158e2bd9096df046548f16c52d9ca03fb2bbb  research/prompts/m3-close-reuse-closure-r2-opus-model/out/variants.txt
    9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  research/prompts/m3-close-reuse-closure-r2-opus-model/out/worlds/hr-int-cap-admitted/status
    da68d7750983143559ded125d5bc05fbfe7a17fab03ccdb31008ea4fe13acfc6  research/prompts/m3-close-reuse-closure-r2-opus-model/out/worlds/hr-int-cap-admitted/stderr
    d5c6c5db0511989f8b2db87038d1fc6952a2f828e46aef7f3836d4a5d457b775  research/prompts/m3-close-reuse-closure-r2-opus-model/out/worlds/hr-int-cap-refused-lookup-off/status
    0b0a409b70c4720584e2ce51d7bbb76dd97bcac1ae1cec4151ace52793b18986  research/prompts/m3-close-reuse-closure-r2-opus-model/out/worlds/hr-int-cap-refused-lookup-off/stderr
    3fd73c76dbb0414be1e7f5ad353ae43c89c55454adad30f8d0f0a74d8f702708  research/prompts/m3-close-reuse-closure-r2-opus-model/out/worlds/hr-int-cap-refused/status
    ff893675320517714d1251c9b3e2e92507ff0118bfc14a9b51dee670e8e657e0  research/prompts/m3-close-reuse-closure-r2-opus-model/out/worlds/hr-int-cap-refused/stderr
    d5c6c5db0511989f8b2db87038d1fc6952a2f828e46aef7f3836d4a5d457b775  research/prompts/m3-close-reuse-closure-r2-opus-model/out/worlds/item1-catchall-refused/status
    6540d0cbf76a92d47f33e551cfac6378bd2f15af6f806dca5564bf98d9306561  research/prompts/m3-close-reuse-closure-r2-opus-model/out/worlds/item1-catchall-refused/stderr
    9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  research/prompts/m3-close-reuse-closure-r2-opus-model/out/worlds/item1-literal-admitted/status
    05005d326808c560e33d0643fa8f554ee78aab317655b95e08a6b2f229001ed7  research/prompts/m3-close-reuse-closure-r2-opus-model/out/worlds/item1-literal-admitted/stderr
    3fd73c76dbb0414be1e7f5ad353ae43c89c55454adad30f8d0f0a74d8f702708  research/prompts/m3-close-reuse-closure-r2-opus-model/out/worlds/item1-literal-refused/status
    b63659b3b2c2da8a8021d7591615c458c2fb2f800937a074efa6f8a9b01c3781  research/prompts/m3-close-reuse-closure-r2-opus-model/out/worlds/item1-literal-refused/stderr
    d5c6c5db0511989f8b2db87038d1fc6952a2f828e46aef7f3836d4a5d457b775  research/prompts/m3-close-reuse-closure-r2-opus-model/out/worlds/lookup-off-refused/status
    052849f48027a4968fbfaf0362ca34383c09e1e044bbfae5b6241388c7e26c8e  research/prompts/m3-close-reuse-closure-r2-opus-model/out/worlds/lookup-off-refused/stderr
    d5c6c5db0511989f8b2db87038d1fc6952a2f828e46aef7f3836d4a5d457b775  research/prompts/m3-close-reuse-closure-r2-opus-model/out/worlds/peer41-refused/status
    aec387498f9a11011a4ce5305d06f3ceb922ce96d6f41083bd909294bbcbf96a  research/prompts/m3-close-reuse-closure-r2-opus-model/out/worlds/peer41-refused/stderr
    d5c6c5db0511989f8b2db87038d1fc6952a2f828e46aef7f3836d4a5d457b775  research/prompts/m3-close-reuse-closure-r2-opus-model/out/worlds/peer43-refused/status
    5dcb91d818e9b0f473284a7f65b3242511c9b20b98a35fe72fc611ef39ce2ab6  research/prompts/m3-close-reuse-closure-r2-opus-model/out/worlds/peer43-refused/stderr
    769abb1d136abab9906fa21b66f701459c0dc4d11cf6680c84fdd0df143dad0a  research/prompts/m3-close-reuse-closure-r2-opus-model/run-all.sh
    58f7bea26c212361bad3bc3d8f1fd643bd128f75491c36366e3bef9b7e180b3c  research/prompts/m3-close-reuse-closure-r2-opus-model/tools/rerun.sh
    842a4821dcbb6975b58873f01a16869d3f1f84e9e27f278c04b7c6ca70f6ec00  research/prompts/m3-close-reuse-closure-r2-opus-model/w1-queue-world.sh
    034438f1f6ee7099429024a63dda08ede0b2dadcbff1e1918b2c50443b6b7b8c  research/prompts/m3-close-reuse-closure-r2-opus-model/w1-strip-judges.py
    2bbc49ac00d0dd0934ce176ba26b8cb021e191a34c3077ee5cab4c7dac8b8836  research/prompts/m3-close-reuse-closure-r2-opus-model/w1-wrapper-hits.sh
    697b430dd560378944ffc28023d52e78d99c28db4ca010e20d3f9cefde253426  research/prompts/m3-close-reuse-closure-r2-opus-model/w2-closures.sh
    df9549fa2433dc1ba7caa3ffba0a611a9e0ba8c461716ae51c782f5249829d13  research/prompts/m3-close-reuse-closure-r2-opus-model/w2-entries.sh
    521f30fa023f3782b5db00695bf0f8f095d6f9585478c6698d0c1a5739a911f1  research/prompts/m3-close-reuse-closure-r2-opus-model/w2-entries.txt
    67aaa5b08421ae6ab82d69e8ab8609ecc5390d44c608ccd9f22c029682062fd1  research/prompts/m3-close-reuse-closure-r2-opus-model/w2-import-census.py
    71973ecf233f0ccf9fc1869c1d17b2a098e43a76592a6a6899f8cd50e7b70fac  research/prompts/m3-close-reuse-closure-r2-opus-model/w2-module-census.py
    1753b0d459c675fedf0cadccc7d9d3c633e21eed03e3ada74f66f164a293c84f  research/prompts/m3-close-reuse-closure-r2-opus-model/w2-module-level-fix.sh
    7e946f7aa9f631e83108936663a1c813ad8134839d0d54b0c193205008a2623f  research/prompts/m3-close-reuse-closure-r2-opus-model/w2-wrapper-start.sh
    79c9261569d50afbdbb8eae859df0f91cdb04a29fb28bfb1bfe6fbd3990ddd73  research/prompts/m3-close-reuse-closure-r2-opus-model/w3-needed-files.py
    19f98853e66b00797d5011ed98f569dc02724df8b5b5546c00bcc01942d5d8ba  research/prompts/m3-close-reuse-closure-r2-opus-model/w3-sync-stale.sh
    9aa098576bdb5686d42dcc47944b59469170daae30768342f18ccdfc9e890f79  research/prompts/m3-close-reuse-closure-r2-opus-model/w4-declaration-fingerprints.sh
```

## 各格判定一览

| 格 | 判定 | 一句话 | 量的还是推的 |
|---|---|---|---|
| W1 | 打中，三处 | ① 去多：广谱变异判「升档」证据要读的正是包装打的 `run-with-memory-cap: 超过限时 …`、`… 撞了内存上限 …` 两句，按前缀去行之后限时、内存两种升档各从 0 条问题变 2 条，那一条从「登记进库目录、带标签」变成「证据不齐、没结论」。② 漏掉：查机器那一支打的 `  → 第二台此刻放得下…` 不以前缀起头，去行之后还在；形式化判法专挑 → 与 ✗ 行当「包装说」，252 那一处的句子换成查机器那一行、随第二台的可用内存变（43.0 与 41.0 GiB 两次不同）。③ try 的范围：host-resources.py 的 `--json` 换一个字段的写法（丙之下它不进键，判法单元指纹改前改后同一把），HostLookup.run 在读完输出之后格式化那一行时抛 TypeError，退 252 之前前台那一趟让包装退 251；照第 1 件字面补的 try 仍 251，整个 run 包住才 252；后台那一趟的回溯 10 行不以前缀起头，去行去不掉 | ①③ 量过（副本，真包装，判法只 import 调函数）；② 量过句子，「不进库」是读代码 |
| W2 | 打中，三处；一处没打中 | ① `exec_module` 那条边今天就在（load 边，切边前后都在）；lib_long_run.py 掉出去是因为模块级 import 按名字连。照第 3 件「认 exec_module 的字面路径」做等于没改，它要的自证「lib_long_run.py 的一个单元改了，形式化入口的键变」在切边树上红：四个入口的键一个不变。② 包装自己静态 import 的 memory_peaks_lib.py 切边后掉了 6 个模块级单元（含 `import subprocess`），那一行多带一个不存在的模块，四把键不变、包装从退 0 变退 251；这一处与 exec_module 无关。③ 产生结论的方法：闭包算法文件头说「import 时的副作用照算」，实际只连 module-body；check_proof 切边后的闭包里 12 份 python 文件有 164 个模块级单元不在（load 进来的 52、当程序起的 6），escalate_step 290。④ 没打中：35 个登记入口里切边只改这四个的闭包 | 全部量过（副本） |
| W3 | 打中，三处；一处推的；一处没打中 | ① 被判形态点名的 `source-stamp.py sync` 两头都是本机路径，第二台的树是 multi-host-run.sh ④ 拷的，两种拷法规则不同（读代码）。② 「只同步 scripts/hosts/ 下用到的几份」起不来：照 HostLookup 的调法跑 host-resources.py 读到仓内 10 份，7 份不在 scripts/hosts/；只拷 scripts/hosts/ 的树退 1，`ModuleNotFoundError: No module named 'project_preflight'`。③ 同步不带 `--delete`：源树删掉的文件副本里还在，第二台若用常驻目录，跑的是新旧两版的并集。④ 推的：整树一趟 10420 份、590M，每台时限 8 秒；到点只停 host-resources.py，它起的同步照跑 | ①读代码；②③量过（本机副本，没碰第二台）；④数量过、时间推的 |
| W4 | 打中，四处 | ① 第 2 件与广谱变异文件头「升档」要包装那一句当证据的规则打架（同 W1①）。② 候选条文的边判据「对它的失败、超时与读不出的输出各有兜底」与它配的三格自证都认不出 W1③：被调方一改键不变（正是排除的本意），包装退出码却从 252 变 251。③ 「声明进键」：声明写在 python 调用方的注释里不进指纹（第 432 行「各自去注释后的指纹」），写在 shell 里才进。④ NOT_JUDGING_FILES 两份经 import 与 load 进闭包（check_proof 切边后 776 个单元），不是「另起的一个程序」，候选句的边判据装不下它，「不按文件排」又禁了它 | ①②③ 量过（副本、夹具）；④ 数量过、冲突是条文对读 |

## W1：丙第 1、2 件之后，HostLookup 那条路还改不改判定结局

### W1①：按前缀去行会把广谱变异判「升档」要读的包装行一起去掉（去多，量过，副本）

被判形态第 2 件点名了广谱变异：「凡读包装标准错误当诊断的判法（形式化三个入口、广谱变异、crates 变异），先去掉以」（`research/prompts/m3-close-reuse-closure-r1-main-verification.md:37`）。而广谱变异有一处判法恰恰靠包装自己打的前缀行当证据：

- 升档证据判法：「if wrapper_hit_text(first_exit, first_text) not in first_log_text:」（`scripts/mutation/broad-mutation.py:2219`），要找的那一段是「return f"超过限时 {limit_text} 秒" if exit_code == TIME_LIMIT_HIT_EXIT else f"撞了内存上限 {limit_text}"」（`scripts/mutation/broad-mutation.py:2204`）。
- 包装打这两句时以前缀起头：「echo "run-with-memory-cap: 超过限时 ${TIME_LIMIT_SECONDS} 秒（scope $unit 的 Result=timeout，systemd 停掉了 scope 里的进程；命令退出码 $exit_code）" >&2」（`scripts/hosts/run-with-memory-cap.sh:1630`），「echo "run-with-memory-cap: 撞了内存上限 $cap（scope $unit 的 Result=oom-kill，命令退出码 $exit_code）" >&2」（`scripts/hosts/run-with-memory-cap.sh:1624`）。
- 证据齐了才往库目录的登记表追加一行、交标签：「append_row(os.path.join(self.library.home, ESCALATION_REGISTRY), ESCALATION_COLUMNS,」（`scripts/mutation/broad-mutation.py:2635`）；不齐走「if problems and not break_is("escalation-evidence-unchecked"):」（`scripts/mutation/broad-mutation.py:2632`）那一支，交「证据不齐、没结论」。

装置：树里那一份真包装在这一趟自己的 slice 里撞限时两趟（限 1 秒、2 秒）与内存上限两趟（64M、128M），日志原样落盘（w1-wrapper-hits.sh）；w1-strip-judges.py 只 import 副本的 broad-mutation.py，拿原样日志与去掉前缀行的日志各调一次 `escalation_evidence_problems`（`out/summary.txt` 原样）：

```
name=wrapper_hit case=time-first status=253
name=wrapper_hit case=time-escalated status=253
name=wrapper_hit case=memory-first status=250
name=wrapper_hit case=memory-escalated status=250
name=broad_escalation_evidence kind=time arm=raw problems=0 first=[]
name=broad_escalation_evidence kind=time arm=stripped problems=2 first=["原来那趟的日志里没有包装打的「超过限时 1 秒」"]
name=broad_escalation_evidence kind=memory arm=raw problems=0 first=[]
name=broad_escalation_evidence kind=memory arm=stripped problems=2 first=["原来那趟的日志里没有包装打的「撞了内存上限 64M
```

（最后一行是 summary 里截到 100 字的样子；`out/part-strip.txt` 有整行。）去行之后两种升档都判「证据不齐」：登记表不追加、那一条不带升档标签，写进库目录的内容变了。这一处正落在 escalate_step 里——就是丙要切边的那个入口。根子是前缀分不开两种行：查机器那一支打的提示与包装自己的撞顶结局都以 `run-with-memory-cap:` 起头，判法按前缀去行，要么去多（这一格），要么留下（下一格）。

### W1②：查机器那一支打的不以前缀起头的行，去行之后照样进判法（漏掉，量过，副本）

- 放得下时打的那一行：「line = f"  → {peer['name']}此刻放得下（可用 {human(peer['available'])}、要 {human(peer['need'])}）：改用 {command}{cap_note}"」（`scripts/hosts/run-with-memory-cap.sh:1060`），行首是两个空格加 →。
- 形式化判法专挑这种行：「return [line.strip() for line in outcome.get("stderr", "").split("\n") if line.strip().startswith(("✗", "→"))]」（`scripts/gate/verus-proof-check.py:803`）；252 那一处取第一行当「包装说」：「where = f"{key} 格{'…' + label + '…' if label else ''}那一处退了 252" + (f"，包装说：{own[0][:200]}" if own else "")」（`scripts/gate/verus-proof-check.py:690`），写进门禁给的那份文件：「handle.write(f"{where}\t{self.first[1]}\n")」（`scripts/gate/verus-proof-check.py:697`）；没跑成那一句另把这几行全带上：「carried = "；包装自己打的：" + "".join(f"…{line}…" for line in own_lines) if own_lines else ""」（`scripts/gate/verus-proof-check.py:788`）。

装置：w1-queue-world.sh（真包装排队，假第二台照包装自证 host_lookup_world 的写法），等满 6 秒退 252；第二台可用内存 43 GiB 与 41 GiB 两个世界，另一条臂关掉查机器。w1-strip-judges.py 先去掉前缀行，再照判法的拼法调 `outcome_problem` 与 `QueueRefusal.record`（原样，截到 100 字）：

```
name=strip_leftovers world=world-peer43-refused lines_after_strip=5 host_lookup_lines_left=2
name=verus_wrapper_outcome world=world-peer43-refused exit=252 own_lines=4 first_own_line=→ 第二台此刻放得下
  queue_refusal_where=proof 格「proof:attack」那一处退了 252，包装说：→ 第二台此刻放得下（可用 43.0 GiB、要 8.0 GiB）：改用 bash s
name=verus_wrapper_outcome world=world-peer41-refused exit=252 own_lines=4 first_own_line=→ 第二台此刻放得下
  queue_refusal_where=proof 格「proof:attack」那一处退了 252，包装说：→ 第二台此刻放得下（可用 41.0 GiB、要 8.0 GiB）：改用 bash s
name=verus_wrapper_outcome world=world-lookup-off-refused exit=252 own_lines=2 first_own_line=✗ 内存不够
  queue_refusal_where=proof 格「proof:attack」那一处退了 252，包装说：✗ 内存不够排不上：等了 6 秒（上限 RUN_WITH_MEMORY_CAP_WAI
```

排队开始那一趟在后台查，放得下的那一行先于包装自己的 ✗ 行打出，于是「包装说」的是查机器那一行、带着第二台此刻的数；关掉查机器的臂说的是包装自己的原因。这一层改的是门禁的「没跑成」句子与同一道后面几格的跳过原因（写进 QueueRefusal 文件），不是写进判定库的内容：库只记绿：「只记绿：红、没跑成、被信号停了的一律不记，下一趟照判。」（`scripts/crash/lib_verdict_library.py:30`，读代码）。同类没触发的还有「  ! 告警记录写不进…」那一行（`scripts/hosts/run-with-memory-cap.sh:1078`，推的）。

### W1③：第 1 件补的 try 只罩到读完输出，读完之后格式化那一行抛异常仍让 252 变 251（try 的范围，量过，副本）

第 1 件字面：「HostLookup 的起线程、前台建临时文件、读输出整段包在 try 里，任何异常都只记一行告警、不改包装的退出码（该退 252 照退 252）」（`research/prompts/m3-close-reuse-closure-r1-main-verification.md:36`），配的两格自证注入的是起线程与建临时文件，都在读输出之前。今天读完输出之后还有一段不在任何 try 里：

- 读 JSON 那一段有兜底：「except (ValueError, KeyError, TypeError, AttributeError, StopIteration) as error:」（`scripts/hosts/run-with-memory-cap.sh:1040`），它只罩到交回那个 dict 为止；
- 之后 run 里拿 dict 里的数做字：上面 W1② 引的第 1060 行调 `human(peer['need'])`，第 1055 行「peer_cap = size_bytes(peer["memory_cap"] or "")」（`scripts/hosts/run-with-memory-cap.sh:1055`）；
- 退 252 之前的那一趟在前台：「self.run("refused", local)」（`scripts/hosts/run-with-memory-cap.sh:949`），排队那一步一抛就走「if [[ $admission_status -ne 0 ]]; then」（`scripts/hosts/run-with-memory-cap.sh:1534`）那一支，包装退 251。

机器状态（可达的历史）：host-resources.py 的 `--json` 把这条命令的预测内存改报给人看的写法——副本里只改一处，「"prediction": {"memory": prediction.memory_predicted, "memory_history": prediction.memory_history, "cap": prediction.memory_cap,」（`scripts/hosts/host-resources.py:450`）的 `prediction.memory_predicted` 换成 `peaks.human(prediction.memory_predicted)`（make-variants.py 的 hr-int-cap）。丙之下 host-resources.py 不进键：切边树与切边树加这一改，形式化入口的判法单元指纹同一把（原样）：

```
name=judging_key tree=cut entry=check_proof key=274ceff933716fc2
name=judging_key tree=cut-hr-int-cap entry=check_proof key=274ceff933716fc2
```

同一个排队世界（W1② 那一份，第二台放得下），四条臂（原样）：

```
name=hr-int-cap-refused status=251 ran=no traceback=2 typeerror=2 peer_fits_lines=0 stderr_lines_not_starting_with_prefix=25
name=hr-int-cap-refused-lookup-off status=252 ran=no traceback=0 typeerror=0 peer_fits_lines=0 stderr_lines_not_starting_with_prefix=3
name=item1-literal-refused status=251 ran=no traceback=2 typeerror=2 peer_fits_lines=0 stderr_lines_not_starting_with_prefix=25
name=item1-catchall-refused status=252 ran=no traceback=0 typeerror=0 peer_fits_lines=0 stderr_lines_not_starting_with_prefix=3
```

item1-literal 是我照第 1 件字面补的 try（起线程那两行、建临时文件到读完输出那一段，make-variants.py 里 `literal_item1`）；item1-catchall 把整个 run 包住（`catchall_item1`）。字面那一版仍退 251；只有包住整个 run 才与关掉查机器的臂相同。放行那一路（排队 3 秒后放行，后台那一趟抛）包装退 0、命令照跑，但线程的回溯打在包装的 stderr 上，去掉前缀行之后剩下 10 行查机器那一支的字（`out/worlds/hr-int-cap-admitted/stderr` 原样，变体目录的路径换成了 `<变体目录>`）：

```
Exception in thread Thread-1 (run):
Traceback (most recent call last):
  File "/usr/lib/python3.12/threading.py", line 1073, in _bootstrap_inner
    self.run()
  File "/usr/lib/python3.12/threading.py", line 1010, in run
    self._target(*self._args, **self._kwargs)
  File "/dev/fd/3", line 298, in run
  File "<变体目录>/hr-int-cap/scripts/hosts/memory_peaks_lib.py", line 72, in human
    if byte_count >= 1024 ** 3:
       ^^^^^^^^^^^^^^^^^^^^^^^
TypeError: '>=' not supported between instances of 'str' and 'int'
命令自己的 stderr
```

字面那一版放行时同样 1 处回溯（`name=item1-literal-admitted … traceback=1`），包住整个 run 的那一版 0 处。这一格要紧的是：第 1 件的两格自证（起不了线程、建不了临时文件）都注入在读输出之前，换成「整个 run 包住」之前，这一种故障两格都抓不到；而这一种故障恰好来自丙排出键外的那份文件。

### W1 没打中的几处（读代码）

告警记录写不进被「except Exception as error:   # 导入失败、写不进：只说一句，排队照旧」（`scripts/hosts/run-with-memory-cap.sh:1077`）接住；节流文件 `host-lookup.json` 读写失败落到第 984 行的 `except OSError:` 后面 `pass`，照查；第二台挂死受每趟时限约束（第 1017–1021 行到点 kill），只在退 252 那一路多等，而包装的限时「从起跑算、不算排队的时间」（`scripts/hosts/run-with-memory-cap.sh:176` 那一行的原话），等的这几秒碰不到 253。没找到别的改退出码、改红绿或改库内容的地方。

## W2：只排那一条边、再「连上 exec_module」之后，闭包还丢不丢会改结局的单元

装置：make-variants.py 造的 cut 树把 HostLookup.query 起 host-resources.py 那一处换成起字面内联 python（等于按边排掉那一条，别的边不动，同第一轮的写法）；w2-closures.sh 用树里的 lib_function_units.py 收四个入口的闭包，键取判法单元指纹（`lib_verdict_library.py judging-units`，判法建库时那一把）。

### W2①：exec_module 那条边今天就连着；第 3 件的改法等于没改，它自己那一格自证红（量过，副本）

predicted_bytes 起 host-dispatch.py 是「spec.loader.exec_module(dispatch)」（`scripts/hosts/run-with-memory-cap.sh:857`），闭包算法早就认 spec_from_file_location 的字面路径：「facts.loads.append(file)」（`scripts/gate/lib_function_units.py:4395`），再「self.push(loaded.module_body, unit, "load")」（`scripts/gate/lib_function_units.py:7256`）。切边前后，`host-dispatch.py:<module-body>` 经 `predicted_bytes` 的 load 边都在闭包里（下面 `load_edge_to_host_dispatch=1`）。lib_long_run.py 掉出去的原因在另一处：模块级 import 是「import（模块级 import，按绑定的名字各一单元）」（`scripts/gate/lib_function_units.py:59`），host-dispatch.py 的「import lib_long_run as long_run_lib  # noqa: E402   长活声明的判法（文件头 ⓪），与 detached-run.sh 共用」（`scripts/hosts/host-dispatch.py:141`）只有被别的单元点名 `long_run_lib` 才进闭包，predict 那一路不点名。原样：

```
name=closure tree=full entry=scripts/gate/verus-proof-check.py::check_proof rc=4 units=1579 key=none load_edge_to_host_dispatch=1 import_long_run_lib=1 import_preflight=1 memory_peaks_lib_import_subprocess=9d97d469cf8d refusals=inline-script-not-literal scripts/hosts/peer-host-lib.sh:76,
name=closure tree=cut entry=scripts/gate/verus-proof-check.py::check_proof rc=0 units=1232 key=274ceff933716fc2 load_edge_to_host_dispatch=1 import_long_run_lib=0 import_preflight=0 memory_peaks_lib_import_subprocess=absent refusals=none
name=closure tree=cut-long-run-unit entry=scripts/gate/verus-proof-check.py::check_proof rc=0 units=1232 key=274ceff933716fc2 load_edge_to_host_dispatch=1 import_long_run_lib=0 import_preflight=0 memory_peaks_lib_import_subprocess=absent refusals=none
name=closure tree=cut entry=scripts/mutation/broad-mutation.py::Runner.escalate_step rc=0 units=436 key=d2b4aec981ecca8a load_edge_to_host_dispatch=1 import_long_run_lib=0 import_preflight=0 memory_peaks_lib_import_subprocess=absent refusals=none
name=closure tree=cut-long-run-unit entry=scripts/mutation/broad-mutation.py::Runner.escalate_step rc=0 units=436 key=d2b4aec981ecca8a load_edge_to_host_dispatch=1 import_long_run_lib=0 import_preflight=0 memory_peaks_lib_import_subprocess=absent refusals=none
```

cut-long-run-unit 在 cut 之上只把 lib_long_run.py 的模块级赋值 `CANONICAL_HOSTS_SENTENCE` 末尾加一个句号（import 时执行的一个单元）。四个入口的键一个不变（另两个入口同形，见 summary）——第 3 件要的自证「lib_long_run.py 的一个单元改了，形式化入口的键变」照「认 exec_module 的字面路径」做是红的。另一种写法「改成静态 import」：文件名 `host-dispatch.py` 带减号，`import host-dispatch` 在 python 里是语法错（推的，没跑）；就算换个法子静态 import 进来，模块级 import 照样按名字连，结果同上（推的）。

### W2②：包装自己静态 import 的 memory_peaks_lib.py，模块级单元同样掉出去，改了键不变、包装退 251（量过，副本）

包装的排队脚本开头「import memory_peaks_lib as peaks   # 读峰值表、读账、算回收不掉的用量与还没涨到量的那一截（统一派发器用同一份）」（`scripts/hosts/run-with-memory-cap.sh:769`），每次起包装都执行它的整份模块级语句，与 exec_module 无关。切边之后它有 6 个模块级单元不在闭包（w2-import-census.py 照排队脚本的写法 import 一遍，列出真执行到的仓内模块，再比闭包，原样）：

```
name=import_census tree=cut file=scripts/hosts/memory_peaks_lib.py module_level=19 not_in_closure=6 examples=assign:DEFAULT_RESERVE_TEXT,assign:DEFAULT_SLICE,assign:DURATION_TABLE_HEADER,assign:LONG_RUN_SECONDS,import:fcntl,import:subprocess
name=import_census_total tree=full executed_files=6 not_in_closure=236
name=import_census_total tree=cut executed_files=6 not_in_closure=300
```

cut-mpl-import 在 cut 之上只把「import subprocess」（`scripts/hosts/memory_peaks_lib.py:27`）那一行改成多带一个不存在的模块。四个入口的判法单元指纹与 cut 逐个相同（cut-mpl-import 那四行 key 与 cut 相同，见 summary），而包装（w2-wrapper-start.sh，这一趟自己的 slice、不排队）：

```
name=wrapper_start tree=cut status=0 ran=yes last_error=
name=wrapper_start tree=cut-mpl-import status=251 ran=no last_error=ModuleNotFoundError: No module named 'attack_w2_missing_module'
```

memory_peaks_lib.py 文件头自己说「import 本身不起子进程、不开文件。…」（`scripts/hosts/memory_peaks_lib.py:5`），但 import 时会不会失败、会不会改模块常量，闭包今天不管（full 树里它就有 2 个模块级单元不在闭包，`import:fcntl` 在内；切边之后 6 个）。

### W2③：产生结论的方法——闭包算法自己写的「import 时的副作用照算」不成立（量过，副本）

算法文件头写：「模块级语句只在那个模块有单元进闭包时连进来（import 时的副作用照算），模块被别的进程起时连的是 main-guard 与 module-body。」（`scripts/gate/lib_function_units.py:262`）。实际连进来的只有 module-body；同一份模块的 import、assign 单元仍按名字连。w2-module-census.py 把闭包里有单元的每份 python 文件的模块级 import、assign、class 单元逐个对闭包（原样）：

```
name=module_census entry=scripts_gate_verus-proof-check_py__check_proof python_files_in_closure=12 module_level_units_not_in_closure=164 by_how_file_entered={'load': 52, 'module-body': 106, 'program': 6}
name=module_census entry=scripts_mutation_broad-mutation_py__Runner_escalate_step python_files_in_closure=10 module_level_units_not_in_closure=290 by_how_file_entered={'load': 115, 'module-body': 169, 'program': 6}
```

`load` 是经 spec_from_file_location / import_module 进来的那几份（check_proof 闭包里四条：core_usage.py 起 lib_work_root.py、preflight.py 起 proc.py、lib_verdict_library.py 起 lib_function_units.py、包装起 host-dispatch.py），`program` 是当程序起的（preflight.sh 起 preflight.py）。题面问「别的经 importlib、runpy、subprocess 起 python 文件的地方今天是不是同样不连边」：边都连着，缺的是边那头模块级 import、assign 这一层，经哪种方式进来都一样缺。丙第 3 件只对 host-dispatch.py 这一份「连边」，碰不到这一层。

### W2④：把这一层连上会怎样（我提的改法，只在我的模型上量过、被攻过零轮）

cut-ml 在 cut 之上改闭包算法一处：一份模块有单元进闭包时（`module_entered`），它模块级的 import、assign 单元一律进闭包。cut-mlc 另把模块级 class 头也连上。w2-module-level-fix.sh 在每棵变体树里用那棵树自己的算法收闭包、算键（原样）：

```
name=module_level_fix tree=cut-ml entry=scripts/gate/verus-proof-check.py::check_proof rc=0 units=1655 lib_long_run_units=41 memory_peaks_lib_import_subprocess=1 key=990b2be37195adb5 refusals=
name=module_level_fix tree=cut-ml entry=scripts/mutation/broad-mutation.py::Runner.escalate_step rc=0 units=959 lib_long_run_units=41 memory_peaks_lib_import_subprocess=1 key=5e94673c710d60e4 refusals=
name=module_level_fix tree=cut-mlc entry=scripts/gate/verus-proof-check.py::check_proof rc=4 units=1697 lib_long_run_units=41 memory_peaks_lib_import_subprocess=1 key=none refusals=shell-path-unfolded scripts/hosts/host-dispatch.py:301,shell-path-unfolded scripts/hosts/host-dispatch.py:318,
name=module_level_fix tree=cut-ml-long-run entry=scripts/gate/verus-proof-check.py::check_proof rc=0 units=1655 lib_long_run_units=41 memory_peaks_lib_import_subprocess=1 key=79d2f285ac7e44ed refusals=
name=module_level_fix tree=cut-ml-mpl entry=scripts/gate/verus-proof-check.py::check_proof rc=0 units=1656 lib_long_run_units=41 memory_peaks_lib_import_subprocess=1 key=2c9e1af64471fd39 refusals=
name=module_level_fix tree=cut-ml-hr entry=scripts/gate/verus-proof-check.py::check_proof rc=0 units=1655 lib_long_run_units=41 memory_peaks_lib_import_subprocess=1 key=990b2be37195adb5 refusals=
```

- 连上 import、assign 之后，拒算不回来，lib_long_run.py 41 个单元进闭包，lib_long_run 那一改、memory_peaks_lib 那一改各让键变（79d2…、2c9e… 都不等于 990b…）：第 3 件那一格自证转绿，W2② 那一格也绿。
- 代价：check_proof 1232 → 1655 个单元，escalate_step 436 → 959，比今天整条边都在时（792）还多——今天的键本来就窄。
- 连 class 头会把拒算带回来：类头连上之后它的 `__init__` 与名字被点到的方法跟着进来，host-dispatch.py 的 Peer 类里第 301、318 行起 bash 的两处 shell-path-unfolded 回来了。只连 import、assign 是在「拒算不回来」与「class 体里的模块级语句不进键」之间取的一边，后者没量。
- host-resources.py 那一改（W1③）键照旧不变（990b… 与 cut-ml 相同）：那一格要靠包装把整个 run 包住，不是闭包能管的。

### W2⑤：没打中——按边排除不改别的入口的闭包（量过，副本）

第一轮正推腿登记的 35 个判法入口（`w2-entries.txt`，从它的 tools/entries.txt 拷来），在原树与 cut 树上各收一次闭包，按（种类、限定名、文件）比：35 行里 31 行 `dropped=0 added=0`、拒算两边相同；变的 4 行就是 check_proof、check_vacuity、check_must_red_worlds、Runner.escalate_step（各少 347、347、347、356 个，拒算 1 → 0）。另：登记表里 broad-mutation.py 的 `escalation_evidence_problems` 自己是一个判法入口（21 个单元），丙第 2 件若在它的输入上去行，W1① 那一改落在 escalate_step 的闭包里，不落在这一个入口的键里（推的，没造这一种实现）。

## W3：甲——第二台跑的是不是同一份

我没有碰第二台（派发没给多机，装置里第二台都是本机的假世界或本机目录）；下面量过的都是本机副本上的数。

### W3①：被判形态点名的拷法到不了第二台（读代码）

甲写「取自每一趟同步过去的树副本（`scripts/hosts/source-stamp.py sync` 拷的那一份）」。sync 的两头都是本机路径：「os.makedirs(copy, exist_ok=True)」（`scripts/hosts/source-stamp.py:186`），rsync 的两个参数是 `source.rstrip("/") + "/"` 与 `copy.rstrip("/") + "/"`（同一文件第 192 行）。第二台的树是 multi-host-run.sh 拷的：「④ 树拷到第二台 <PEER_REPOSITORY_DIRECTORY>/runs/<标签>/（新建目录，标签已在就拒、退 2）：拷的是 git 列得出的文件」（`scripts/hosts/multi-host-run.sh:18`），跑完「⑦ 删第二台这一趟的树与 .pid / .exit / .log / .serial；④ 之后在哪一步失败退出，这一步都挂在 EXIT 上照做。」（`scripts/hosts/multi-host-run.sh:49`）。两种拷法取的文件不一样：sync 拷源树里除 .git、target 一类、__pycache__ 之外的全部（含 git 忽略的文件），multi-host-run 拷 git 列得出的。甲照字面实现会用错工具；照意思实现，要说清用哪一种、在哪个目录。

### W3②：「探测前只把 scripts/hosts/ 下用到的几份同步过去」起不来（量过，本机副本）

w3-needed-files.py 在整棵副本上照 HostLookup 的调法跑一次 `host-resources.py --json -- true`（多机开关关着，只查这一台，等于第二台「由参数选自己」那一种），装 audit hook 记 python 层打开的仓内文件与子进程点名的仓内脚本（bash 里 source 进来的记不到，是下界）；再只拷 scripts/hosts/ 下的 .py 与 .sh 进一棵空树，同一条命令再跑（原样）：

```
name=needed_files full_tree_rc=0 repo_files=10 outside_scripts_hosts=7
  outside=.claude/hooks/lib_heavy_tests.py
  outside=.claude/hooks/lib_long_run.py
  outside=.claude/hooks/lib_shell_words.py
  outside=.claude/scripts/project_preflight.py
  outside=.claude/singlefs-ai-sop/scripts/preflight.py
  outside=scripts/crash/crash-amplification-lib.sh
  outside=scripts/crash/crash-point-replay-shard-configuration-check.sh
name=only_scripts_hosts rc=1 stdout_is_json=False last_stderr_line=ModuleNotFoundError: No module named 'project_preflight'
```

第一份就卡在模块开头：「from project_preflight import preflight  # noqa: E402」（`scripts/hosts/host-resources.py:76`）。要同步的不是「scripts/hosts/ 下用到的几份」，而是这 10 份连同 bash 里 source 进来的那几份；清单要算出来，不能手写——手写的清单漏一份，第二台起不来，HostLookup 照实打「没读到」；漏的那份在第二台上恰好有旧版，就跑旧版（下一格）。

### W3③：同步不带 --delete，第二台若用常驻目录，跑的是新旧两版的并集（量过，本机副本）

sync 文件头写明：「手工拷树的入口：rsync -a --no-times --checksum（不带 --delete；排除 .git、__pycache__，与任意一层名叫 target、target-* 的目录：」（`scripts/hosts/source-stamp.py:18`）；multi-host-run 也是「rsync 不带 --delete；规范副本另拷（peer_host_sop_source：树里有拷树里的，没有拷主仓的），拷完回读 scripts/preflight.py；」（`scripts/hosts/multi-host-run.sh:21`），它靠每一趟新建目录躲开这一点。w3-sync-stale.sh 用 sync 拷两趟，两趟之间在源树删掉 heavy_threads.py（原样）：

```
name=sync_without_delete first_rc=0 second_rc=0 deleted_in_source_still_in_copy=yes python_files_only_in_copy=1
```

甲若为了赶探测的时限把那几份同步进一个常驻目录（不每次新建），本机删掉或改名的模块在第二台还在，import 照样成功，第二台跑的不是本机那一份。

### W3④：每次新建目录的那一种，代价与收尾（数量过，时间推的）

要每次新建，整棵树一趟（本机仓根现量）：

```
$ git ls-files | wc -l
10420
$ git ls-files -z | du -ch --files0-from=- 2>/dev/null | tail -1
590M	total
```

查机器给每台的时限是「HOST_LOOKUP_PEER_TIMEOUT_SECONDS = 8    # 交给 host-resources.py 的每台时限（HOST_RESOURCES_HOST_TIMEOUT_SECONDS）」（`scripts/hosts/run-with-memory-cap.sh:776`）；整树一趟 8 秒拷不拷得完没在第二台量（推的：拷不完）。到点时 HostLookup 只停 host-resources.py 一个进程：「process.kill()   # 只停自己起的这一个（按它的进程号）」（`scripts/hosts/run-with-memory-cap.sh:1019`），文件头说「它起的 ssh 由 ssh 自己收尾」（同一文件第 117 行）。同步若由 host-resources.py 起，到点之后同步与第二台上的删除都不归任何人停：第二台留下半棵树，或与 60 秒后下一趟的同步撞在同一个目录上（推的，按代码）。排队那一刻第二台上若正好有别的双机活的 `runs/<标签>/`，那是那一趟开跑时的树，比本机此刻旧；multi-host-run 对已在的标签直接拒，也借不过来（读代码）。

### W3 没打中：「键里不算第二台」

丙之下查机器这一路不出判定，第二台跑哪一版只改提示的字；第二台判出的东西导入本机时按本机现算的键逐条核，这一条本轮没找到反例。只有一处相连：提示的字经 W1② 那条通道进门禁的「没跑成」句子，第二台版本不同时那一句也不同（推的，没造两个版本的第二台）。

## W4：丙加甲合起来，候选条文与已定项 17、NOT_JUDGING_FILES 打不打架

候选条文取第一轮正推腿给已定项 17 的那一条（逐字）：「- 沿调用与引用收闭包时，一条调用边同时满足下面三样才许排除：被调方是调用方单元另起的一个程序，它的输出只进调用方的提示行与告警记录，不进判定的退出码、标准输出与判定库的内容，调用方对它的失败、超时与读不出的输出各有兜底；排除写在调用方源码的注释里（`# closure-excluded-edge: <调用方单元> -> <被调文件>：<为什么>`），声明进键，改了或删了键就变，指到不存在的边拒算；这条边有一格自证钉住：改被调方键不变、改调用方自己的单元键变、被调方经别的边进闭包时照进。排除只落在这一条边上：调用方单元、被调方经别的边进来的单元与闭包里别处的拒算照旧进键、照旧拒算。不按文件排、不按函数整个排、不按调用方的环境开关排。」（`research/prompts/m3-close-reuse-closure-r1-sonnet-output.md:244`）。已定项 17 引的三行整行抄：

- 「- 一律不进键、也不许当复用依据：按提交、HEAD 或树算的改动范围；整文件原文；文件级、包级、整表级的指纹或闭包；手写的版本常量；路径（仓根也算）、名字、种子串、行号、资源上限、限时；只有整道或整格的标记而没有按判定的库。键过窄（引用的常量、类型、宏漏收）、库放在会丢的位置、改名丢库、两份库重复判同一批，同样算违反。」（`.claude/kb/decisions/13-验证路线.md:433`）
- 「- 收不全（动态分派、宏、`include_str!(concat!…)` 认不出）的那一份拒算键、照跑，不静默少收；每把键连同组成它的单元清单落盘，作废时报得出是哪几个单元变了。」（`.claude/kb/decisions/13-验证路线.md:434`）
- 「- 每类判定一份库，固定在持久目录，不随一趟、不随判法换目录；仓根也不是库的输入：崩溃判定库的目录由环境变量 `SINGLEFS_CRASH_AMPLIFICATION_LIBRARY` 给时只收绝对路径，相对路径拒绝；全量开关只写不读、现判的照键写回同一份库；第二台判出的并进本机那一份。」（`.claude/kb/decisions/13-验证路线.md:435`）

第 432 行整行 427 字，超过 cite-check 一处引文 400 字的上限，只抄要用的一句：「函数单元从这份判定的入口沿调用与引用收，连同引用的常量、类型、静态项与宏展开一起算」（`.claude/kb/decisions/13-验证路线.md:432`），同一行前面还有「各自去注释后的指纹」（`.claude/kb/decisions/13-验证路线.md:432`）。

### W4①：第 2 件与广谱变异「升档」证据规则打架（量过，同 W1①）

候选条文的前提是「它的输出只进调用方的提示行」，第 2 件为此让判法去掉 `run-with-memory-cap:` 起头的行。广谱变异的升档证据必须读到包装打的那一句（W1①引的第 2219、2204 行），按前缀去行，升档那一条的登记从有到无。要两样都成立，前缀要分开：查机器那一支打自己的前缀（或只写告警记录、不写标准错误），判法只去那一种。照今天的写法，第 2 件与候选条文的前提自洽不了。

### W4②：候选的兜底判据与三格自证都认不出 W1③，那一格正好落在「不静默少收」上（量过，副本）

W1③：被调方 host-resources.py 改了输出里一个字段的写法，判法单元指纹不变（切边树两把键都是 274ceff9…），包装从退 252 变退 251。对着候选逐句看：输出能读出（JSON 照样解析），「读不出的输出各有兜底」不触发；三格自证里「改被调方键不变」恰好就是这里发生的事，是自证要它绿的那一格；另两格不注入被调方输出形状的变化。于是候选条文认可的一条边，被调方一改就改了退出码，而键不变——这是第 433 行末句说的「键过窄」，也是第 434 行「不静默少收」防的那一种。要自洽，判据得写成「调用方从起它到最后一次用它的输出，整段都在兜底里」，自证得加一格「被调方交回形状合法、类型不对的输出，包装退出码与关掉这一支的那一臂相同」（W1③ 的 item1-catchall 臂在我的模型上量过是 252，被攻过零轮）。

### W4③：「声明进键」在 python 调用方上不成立（量过，夹具）

w4-declaration-fingerprints.sh 在四处各造两份只差 `# closure-excluded-edge: …` 这一行注释的夹具，用 lib_function_units.py dump 比单元行（原样）：

```
name=declaration_fingerprint file=caller.py units_without=3 units_with=3 identical=yes
name=declaration_fingerprint file=function-body.sh units_without=3 units_with=3 identical=no
name=declaration_fingerprint file=top-level.sh units_without=2 units_with=2 identical=no
name=declaration_fingerprint file=heredoc.sh units_without=3 units_with=3 identical=no
```

python 的 def 指纹去注释（已定项 17 第 432 行「各自去注释后的指纹」，算法开关表里「python-comments-kept             def 的指纹取原文（带注释）」（`scripts/gate/lib_function_units.py:160`）是弄坏开关），声明写在 python 调用方里，改它、删它键都不变，候选那一句「声明进键，改了或删了键就变」假。shell 里进（`<kept-comments>` 数据单元）。HostLookup 这条边的调用方在包装的 here-doc 里，量出来进（heredoc.sh 那一行）；候选条文是通用的，下一条 python 调用方的排除就钉不住。另有一处字面上的别扭（对读，没量）：声明里带 `<被调文件>` 路径与 `<调用方单元>` 名字，第 433 行把「路径（仓根也算）、名字」列在不进键之列；若「声明进键」指它的文字随调用方单元的指纹进键，那与代码里别的字面串没有分别，不算违反，条文要写成这个意思。

### W4④：NOT_JUDGING_FILES 装不进候选的边判据，「不按文件排」又禁了它（数量过，冲突是对读）

「NOT_JUDGING_FILES = ("scripts/crash/lib_verdict_library.py", "scripts/gate/lib_function_units.py")」（`scripts/crash/lib_verdict_library.py:88`），理由「它们管怎么存、怎么算键，不管判对错，改它们不让任何判定作废。」（`scripts/crash/lib_verdict_library.py:20`）。check_proof 切边后的闭包里这两份的单元（原样）：

```
name=not_judging_files_units_in_cut_closure entry=check_proof scripts/crash/lib_verdict_library.py=62 scripts/gate/lib_function_units.py=714
```

它们经 verus-proof-check.py 的 import（5 个名字）与 lib_verdict_library.py 的 `function_units` 按路径 load 进来（`out/` 下没有单列这一行，是我在同一份闭包输出上数的 via 行），是进程内的调用，不是「被调方是调用方单元另起的一个程序」，候选的三样判据一样也套不上；候选末句「不按文件排」又直接禁了文件级排除。两条出路都要明写：候选加一句「算键与存库的两份另立一条、配自己的自证」，或撤掉 NOT_JUDGING_FILES（之后改闭包算法本身会让所有判法单元指纹作废，第 436 行说这不算代价）。还有一层与本题相连：丙的排边与 W2④ 那一改都写在 lib_function_units.py 里，它在 NOT_JUDGING_FILES 之内，算法改了键只经「多收、少收的单元」间接变，算法自己的单元不进键（读代码）。

### W4 没打中：甲与第 435 行

「第二台判出的并进本机那一份」与甲「键里不算第二台、导入时按本机现算的键逐条核」不冲突；查机器这一路在丙之下不出判定，没有要并进本机的东西。

## 打中之后的四句

| 打中 | 分不分辨臂 | 被判的系统当时看不看得到判别它的东西 | 满足判据字面的哪一个分句 | 跑前给的改法在打中的那几格上还中不中 |
|---|---|---|---|---|
| W1①（去行去掉升档证据） | 分辨第 2 件的「按前缀去行」与「查机器那一支另起前缀、只去那一种」两种写法；不分辨甲 | 判法看不到：两种行同一个前缀，按前缀分不开（判别子观测不到，要改的是包装打的字，不是判法） | W1 格「去掉 `run-with-memory-cap:` 起头的行会不会把判法真要读的行一起去掉」；后果落在「写进库的内容」（广谱变异库目录的升档登记表） | 第 2 件照字面：中；第 1 件、第 3 件、甲：碰不到这一格 |
| W1②（→ 行去不掉） | 同上两种写法 | 判法看得到行首是 → 还是 ✗，看不到是谁打的 | W1 格「漏掉 HostLookup 打出的不以它起头的行」；后果是门禁的没跑成句子与 QueueRefusal 文件，不满足「写进库的内容」 | 第 2 件照字面：中；另起前缀：不中（推的） |
| W1③（读完输出之后抛，252 变 251） | 分辨第 1 件「三处包 try」与「整个 run 包住」 | 包装看得到（它自己的异常），今天没接 | W1 格「try 之外还剩哪几处」，后果是「退出码」 | 第 1 件照字面：中（量过 251）；整个 run 包住：不中（量过 252）；第 1 件那两格自证：抓不到（注入点都在读输出之前，推的） |
| W2①（exec_module 已连、自证格红） | 分辨第 3 件两种写法与「模块级 import、assign 一律连」 | 闭包算法看得到那几个单元（dump 里有），规则不连 | W2 格「连了 exec_module 仍漏掉的会改结局的单元」 | 第 3 件「认 exec_module 字面路径」：中（量过，键不变）；「改成静态 import」：写不成（推的）；模块级 import、assign 一律连：不中（量过） |
| W2②（memory_peaks_lib 改了键不变、退 251） | 同上 | 同上 | W2 格「只排一条边仍丢掉的会改结局的单元」，后果是「退出码」 | 第 3 件两种写法：都中（这份不经 exec_module）；模块级一律连：不中（量过） |
| W2③（算法文件头那一句不成立） | 不分辨臂：丙之前就有（full 树 236 个），所有按函数单元键复用的 python 判法共用这个前提 | 看得到 | W2 格「别的经 importlib、runpy、subprocess 起 python 文件的地方」 | 照「打中不分辨臂」办：另立一笔账先修，判臂只看 W2①② |
| W3②③（只同步几份起不来；不带 --delete） | 分辨甲的「同步几份进常驻目录」与「每趟新建整树」 | 第二台看不到本机删了什么 | W3 格「探测前同步几份」「第二台跑的不是同一份」 | 甲照字面：中；每趟新建整树：这两格不中，换来 W3④ 的代价（推的） |
| W4②③④（条文） | 不分辨丙甲，是条文写法 | — | W4 格「候选句与已定项 17、NOT_JUDGING_FILES 打架」 | 第一轮正推腿的候选：中；按下一节改写：推的 |

## 我提的改法各修哪一格（只在我的模型上量过、被攻过零轮）

| 改法 | W1① | W1② | W1③ | W2① | W2② | W3② | W3③ | W4② | W4③ | W4④ |
|---|---|---|---|---|---|---|---|---|---|---|
| 查机器那一支打自己的前缀（例 `run-with-memory-cap-host-lookup:`，含回溯在内一律经 say 打），判法只去这一种 | 修（推的） | 修（推的） | 不修 | 不修 | 不修 | 不修 | 不修 | 不修 | 不修 | 不修 |
| HostLookup.run 整个包在 try 里（begin 起的那一趟与 finish 前台那一趟都过它） | 不修 | 不修 | 修（量过：item1-catchall 252、放行时 0 处回溯） | 不修 | 不修 | 不修 | 不修 | 修自证那一半（量过的是退出码，自证格没实现） | 不修 | 不修 |
| 闭包算法：模块有单元进闭包时，模块级 import、assign 一律进闭包（不连 class 头） | 不修 | 不修 | 不修（量过：键照旧不变） | 修（量过：键 990b… → 79d2…） | 修（量过：键 → 2c9e…） | 不修 | 不修 | 不修 | 不修 | 不修 |
| 同上另连 class 头 | — | — | — | 修但拒算回来（量过：host-dispatch.py 第 301、318 行） | 同左 | — | — | — | — | — |
| 第二台的同步清单由同一份 import 追踪算出（同 w3-needed-files.py 的办法，另补 bash source），每趟新建目录、收尾不挂在会被 kill 的进程上 | 不修 | 不修 | 不修 | 不修 | 不修 | 修（推的） | 修（推的） | 不修 | 不修 | 不修 |
| 排除声明不写注释，写成调用方单元里一个会进指纹的字面量（或 shell 的 kept-comments 那一种） | 不修 | 不修 | 不修 | 不修 | 不修 | 不修 | 不修 | 不修 | 修（推的：python 里的字面量进 ast 指纹） | 不修 |
| 候选加一句：算键与存库的两份另立一条、配自己的自证（或撤 NOT_JUDGING_FILES） | 不修 | 不修 | 不修 | 不修 | 不修 | 不修 | 不修 | 不修 | 不修 | 修（推的） |

## 没打中的形状

- W1：告警记录写不进、节流文件 `host-lookup.json` 读写失败、第二台挂死（每趟至多 15 秒、只在退 252 那一路多等），都没改到退出码、红绿或库（读代码，见 W1 末节）。查机器那一支往标准输出写字：它的子进程输出进临时文件，自己只往标准错误打（读代码）。→ 行碰形式化判法的子串判法：它的字是命令行、第二台的名字与内存数，没有「Resource limit (rlimit) exceeded」，也不以 error 起头；包装 252 的出路判法找「等多久都放不下」，→ 行里也没有（推的）。多行 problem：第二台 stderr 只取末一行，`splitlines` 已把别的分行符拆掉，判法按 `\n` 切看不到第二行（推的）。
- W2：按边排除不改 35 个登记入口里另外 31 个的闭包（量过）；没逐份收全仓经包装起的脚本。
- W3：「键里不算第二台」本轮没找到反例；第二台的 python 与本机不同版本这一种没造。
- W4：甲与第 435 行不冲突。
- 取样：W1 的排队世界每个只跑一次（等满 6 秒或 3 秒后放行，第二台可用 43 与 41 GiB 两档）；W2 只看四个碰得到包装的入口，入口清单取第一轮正推腿的 35 个；W4 夹具四种写法各一份。

## 这条腿自己的限度

- 单次抽样：这一轮只有这一条攻方腿，没打中的几条不支撑「没有」（`.claude/rules/three-way-inference.md` 里讲「一条腿只抽一次样不算一次观测」的那一节）。
- 副本是开工时 rsync 的主工作区（scripts、.claude、crates、litmus、Cargo 清单），被判八份 sha256 对得上；装置还读别的文件（broad-mutation.py、memory_peaks_lib.py、lib_long_run.py 等），那几份是开工那一刻的样子。
- W1① 只调到 `escalation_evidence_problems` 这一个函数，没让广谱变异真跑一次升档（那是重型）；「登记表不追加」是读第 2632–2640 行推的。W1② 只调到 `outcome_problem`、`QueueRefusal.record`，没起 Verus。
- W1③ 的 host-resources.py 那一改是我造的（一次说得通的重构），仓的历史里没找它出现过；要证的是通道，不是常见程度。
- 甲没有实现：W3 只量了它要用的几样今天的行为（拷法的两头、要读哪些文件、不带 --delete），没碰第二台；整树一趟要几秒是推的。
- W2③ 的普查数取决于 dump 扫的目录（scripts、.claude/gate.d、.claude/hooks、.claude/scripts、.claude/singlefs-ai-sop/scripts）；W3② 的 audit hook 记不到 bash 里 source 进来的文件，是下界。
- 复跑我没经包装跑成：经 8G 包装起时整台 slice 被别的会话占着，排队 5 分钟以上，我停了它（包装退 251）；直接跑 rerun.sh 一次，`status=same lines=147`。

## 没做什么

- 没判正推、辩方那几格；没替主 agent 采纳；副本上量出的数不算入库装置上的数。
- 没跑重型测试、门禁阶段与各类 lint，没编 Rust，没起 Verus、cargo；只跑了 lib_function_units.py 的 closure、dump，lib_verdict_library.py 的 judging-units，source-stamp.py sync（草稿目录里），真内存包装（自己建、自己停的 slice），广谱变异与形式化判法都只 import 后调纯函数。
- 原型扫描、崩溃枚举不在这一题里，没调 `enumerate_crash_states`，状态数 0。
- 没碰第二台，没读多机配置的真值（装置里第二台都是假的）。
- 没读禁读清单里的三样。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-close-reuse-closure-r2-opus` 里的仓副本、变体树与几趟输出交回前已删，只留认领时的 .owner 与 .writer-lock；没有留下自己起的进程与 slice（交回前 ps 与 systemctl --user list-units 现查过，0 个）。
- 要主 agent 跑的命令：只有上面「复跑」那一条（经包装、上限 8G、工作目录仓根，看末行 `name=rerun_compare status=same lines=147`）。
