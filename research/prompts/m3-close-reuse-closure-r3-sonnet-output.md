# m3-close-reuse-closure-r3 辩方腿报告（Sonnet，三方论证第三轮，2026-10-23）

辩的是前两轮判决里被判出局或改掉的写法，复核判打中够不够得着、是不是同样打中替代写法（判决：`research/prompts/m3-close-reuse-closure-r1-main-verification.md`、`research/prompts/m3-close-reuse-closure-r2-main-verification.md`）。格 X1–X3 的结论在第三、四、五节；第一节是判定一览表，第二节是逐条辩护 W1–W6，六、七节是零轮形态与没做什么。没读禁读清单里的任何文件。

## 〇、复跑与哈希

- 开工快照：`sha256sum -c research/prompts/m3-close-reuse-closure-r3-snapshot/judged-sha256.txt` 开工时 9 行全 OK；交回前再核，`scripts/hosts/peer-host-lib.sh` 变了（别的会话 2026-10-22 在这一轮开工之后改的，header 与第二台共用编译目录的守卫那一段，peer_host_run 一段没动，函数仍在第 69 行），其余 8 行 OK。我的量全在开工时拷出的冻结副本上做，副本里 9 份与快照同哈希。
- 冻结副本：`~/.local/share/singlefs/work/m3-close-reuse-closure-r3-sonnet/repo`，51M，按交回闸的要求交回前已删（只拷了 `scripts/`（不含 `experiments/`，另拷 `replay.sh` 与 `e165-pin-red-examples.py`）、`.claude/` 的 `gate.d`、`hooks`、`scripts`、`singlefs-ai-sop`）。树在漂：`.claude/hooks/lib_heavy_tests.py` 2026-10-22 被改过（在第二轮判决写完之后），`scripts/mutation/crates-mutation-rows.py` 我做的几个钟头里还有别的会话在改（行号漂了好几次），所以下面带行号的引文以交回时的主工作区为准，量的数以 `out/` 里留的原样输出为准；副本删了之后复跑把主仓当 `--frozen`，主树在漂，数可能跟着变，对不上先看树漂移（第五节末）。
- 复跑（整条经包装；`--frozen` 给主仓，脚本自己只拷要的；八棵树各收 41 个入口的闭包，另三个算键的作业，实测约 10 分钟，退 0）：

```
bash scripts/hosts/run-with-memory-cap.sh 6G bash research/prompts/m3-close-reuse-closure-r3-sonnet-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs --work <新的草稿目录>
python3 research/prompts/m3-close-reuse-closure-r3-sonnet-model/tools/hostlookup_inject.py scripts/hosts/run-with-memory-cap.sh
```

  复跑原样输出在 `out/rerun-1.out`（第一行起的两行排队提示是包装的，其中那行「→ 第二台此刻放得下…」是这一趟排队时 HostLookup 真打出来的）。`rerun.sh --selftest` 查的是模板自带的 rerun_steps，换了步骤的副本上不适用，没跑。
- 模型目录 `research/prompts/m3-close-reuse-closure-r3-sonnet-model/` 每个文件的 sha256：

```
8f864b37f339d9a4da54d91aeed6d830b240478bbad82964298604eaaac56953  out/arg-limit.out
fa76cec75f381aa1e852101c5fe0c0d147ae76609f21c9a5cf1a8d68b67d5dfa  out/compare-leg-scan.out
149de1d941e2af125f8b8e6ddf19bfc523b0b892859f11a515637247f68f1517  out/cut-narrow.out
69da1215cb6504df6049483ba5e2f541d11cc90d34719c3d843e78631505d5eb  out/diff-class-heads.txt
d98f0212b7bbacd73661eb05a84336e4c57907671db3563731b9e2674b0c7d82  out/diff-cut-modlevel.txt
722f9de1c2172ff16594d159b57877c8d3a8384251a6a70419e8a84719be64f8  out/diff-vs-current.txt
ebef60d3d7483eacb43d4fde7b49b6684ae28be4d4cafc3151ad0b75db550067  out/e165-load-edge.out
23bb75152ecf2b81b31a8f18ecfb087403c28b2196bc2217593bf33acec9d7ad  out/entries-registry.gen.txt
6315d5c849d236b94b6b84dd62a36f449aa81f8546d751e3a42f83b875fcbaf8  out/fileexcl-current.txt
5ce874ecfc61edef58bff54c83c836085e188da574bd3bc6f06fb4ea18aa2177  out/fileexcl-mut-a.txt
492eae79796f9eb2fc0742e5b9f662c6a62c8403a386486066700d2155d1c0a1  out/fileexcl-mut-b.txt
338afa55657a61210fc52f2cf6f381ce3471a0a0f94b4a0f56a3d000e241506e  out/hostlookup-inject.out
c048822b71d0176c387d47604c83123b6abe0f443cad6b3ead61c45faeb6e64e  out/membership.out
a11ed3a0aae59d0d2f8894e7f62d956cee7c7463f082dc4730980bd6434bdc84  out/module-census-current.out
7ca1e110908bfa20edfa830889ff8a2b34ffa89f97640c349599e8671547b249  out/module-census.out
27c340e546ae3eb8240e9bf112a34bcb43cb2b7bef4b320d8cd88b9019e10387  out/not-judging-files.out
3ba30f76fa135e6ff175ceb63b2c53b7fe8296ca683fe59d7e02e3cedf6b124c  out/opus-w3-needed-files.out
f45d71fbb855ce97d18916f956c79cd79ada96e4004290625ac3ed1696c8ceac  out/rerun-1.out
c14f45c122952caf44d835976f455e0639ca165bf8365ff758abde8044443a94  out/scan-current.tsv
2926938b57b424996e4c4f2bc5362e22e5e5f44e928ee64ab3b3ca8f6b5c735a  out/scan-cut-algo.tsv
5fed68f340bde8b6c84ee534adfe6dda411942d66a2e552f50dc851c3f265fa1  out/scan-cut-modlevel-class.tsv
e8f2bbdaa38f9a68830c7ae24955c6b4dcf63758541cf71e868c77b0cf65e6ba  out/scan-cut-modlevel.tsv
b19fed7c616c53a77b9a5d3a061424dc06d6a2edac6682610c293f76fd599816  out/scan-cut-src.tsv
18fd0b94a4b78f45b578f508accf75f27224e689f7bab055eab5a8ece95a5bce  out/scan-modlevel.tsv
1e1b307b6f53e44572ef0d7475811759fc6a327c05d16048f3b9b71343fbeb89  out/scan-narrow.tsv
0bf789a00696449c1c4f253a93f514ef4da35149573feee4c8259a0dcca6e419  out/scan-stub-hr.tsv
5f59f5fc50690aa0fde676bc204455ff77fb1e2a3016d16a44a6bdc440e7bb0e  tools/arg_limit_check.sh
0ed0d5257a6fe79c017d9486301e8361bd7011bcb4b4aac4834470c89ffd2cac  tools/closure_set.py
e375b72eb79d2c1dbe597f268c1121cffd42eac6555e2c25ae5c87b703733aed  tools/compare_leg_scan.py
5e88b118953bb515f8fb414b9c56ac81421f97d2e57218174e1c604be86712e4  tools/cut_narrow_check.sh
521f30fa023f3782b5db00695bf0f8f095d6f9585478c6698d0c1a5739a911f1  tools/entries35-from-r2.txt
23bb75152ecf2b81b31a8f18ecfb087403c28b2196bc2217593bf33acec9d7ad  tools/entries-registry.txt
a9545f509038a4f40fee095e70d00917f32162bad2c2e30d8f4067678159664b  tools/entries-two.txt
7108508285cd35f4875a33450f1eb8d7bd8e92ab29ef2c12d05f6d94fa597381  tools/fileexcl_key.py
a1f25bee77c2f7cb0c333dcf48b8cf919b961b831c9f46cf3f7ec63473afdeaa  tools/gen_entries_registry.py
75c7a849018f73bb953c13f1ada42368c382ecefaf79358ca1fbd5d24254bf9f  tools/hostlookup_inject.py
549fa5a33af560dc16f0e2b7cd77134b0a5e64400ca306b93868ec54e5ddd481  tools/make_variants_all.py
52590d85fbf5117c7eceece13aa474ee0932d9bb266fc3a9577792da04083f33  tools/membership_check.py
34c9a703e02741ebcad0ba0e70cc26a744b817efc7fc68ead54735bd9c1ee477  tools/module_census.py
b24d5af272c7d29a28d60e776299e010ed822102aef933e3289d2cb4498eb7ae  tools/needed_files_strace.sh
ae4e5a9128e6b7a7ae895b3e856429173f027bd927044fec91534efb31140aa2  tools/remote_program_size.py
ac815849b69678e28f9e101a835a3c39088be90bf847ff7a92432d431a564b04  tools/rerun.sh
c797c86558e9b5ba417b05cf242ef13b34d859fb64bab14c6f2e2565ba272ac6  tools/scan_diff2.py
c8a926aa2dba6b0e8be3d757c075eef7455f3cee889f5666554cd5bb8523214a  tools/scan.py
```

- 入口清单不抄原腿的：`tools/entries-registry.txt` 从源码里的登记表现取（`ast` 读 `JUDGING_ENTRIES`、`GPU_JUDGING_ENTRIES`、`REPLAY_JUDGING_FUNCTIONS` 等），41 个，比两轮用的 35 个多 6 个（第四节 X2）。

## 一、判定一览

判定只用三个词：站得住（判决的打中成立，被判出局的写法辩不回来）、够不着（判决的打中碰不到这个写法，或碰到的只是它的一个变体）、同样打中别的臂（判决拿来打这个写法的那一击，换成判决采纳的写法或别的替代写法也一样中）。同一行可能分几种，分开写。

| 判决条目 | 辩方判定 | 证据 |
|---|---|---|
| 第二轮 W1：HostLookup 只包起线程、建临时文件、读输出三处（第一轮 V1 结局那一击）出局，改成整体兜底 | 站得住（出局对）。采纳形态的方向对，但有两个注入点 try 关不了 | 用包装里 class HostLookup 的原文在假世界里注入五个点（`out/hostlookup-inject.out`）：起线程失败、建临时文件失败、格式化抛 TypeError 三个都是异常冒出调用方，try 能关；排队锁不放、join 无时限两个是卡住，try 关不了，只能靠 LOCK_NB 与 join 时限（见第二节 W1） |
| 第二轮 W1：按 `run-with-memory-cap:` 前缀去行出局，改成源头打标记 | 出局对「凡这个前缀一律去」站得住；「窄正则」版够不着去多这一击；去少这一击同样打中源头标记版 | 去多：升档证据两句确实带同一前缀。但仓里已有窄正则先例 `WRAPPER_QUEUE_LINE`（形式化与 crates 变异各一份），只去排队两种行，不碰证据两句。去少：`say()` 被 HostLookup 与别的包装输出共用，同一句「! 告警记录写不进」两处各打一次，源头标记要逐个调用点打，覆盖义务和窄正则一样；标记还会弄坏 `merge-and-verify.sh` 里 `^  → 第二台此刻放得下` 的锚（第二节 W2） |
| 第一轮 V2 / 第二轮 W2：「认 `exec_module` 字面路径」让 `predicted_bytes` 连边 | 站得住（打中成立，那条 load 边切边前后都在，照字面做等于没改）。它的窄读法替代被 memory_peaks_lib 的自证打掉，宽读法站得住 | 切边树里 `host-dispatch.py:<module-body>` 经 load 进来，`lib_long_run.py` 掉出去是因为模块级 import 不跟；窄读法（只收按路径载入的模块与它们 import 引来的模块）切边后 `memory_peaks_lib.py` 的 `import subprocess` 不在闭包，宽读法在（第二节 W3） |
| 第一轮 V2：按文件排、按函数排、按开关排，只有按调用边可行 | 按函数、按开关：站得住（沿用判决的读法，我没另量：`excluded_units` 只挡闭包成员，`HostLookup.query` 是内联单元不是成员；开关是运行期环境变量，静态判不了）。按文件：够不着（两个文件的版本没被打到），同样打中的只剩「只打提示」那个前提 | 用 `lib_verdict_library` 现成的 `excluded_files` 把 `host-resources.py` 与 `peer-host-lib.sh` 排掉：三个形式化入口的判法键算得出、拒算 0、不改闭包算法，且 `lib_long_run.py` 与 `memory_peaks_lib.py` 的模块级 import 各改一处键都变（`out/fileexcl-*.txt`）。原判决只推了单排 `peer-host-lib.sh` 一份（第一轮攻方报告标「推的」）。这是我造的零轮形态（第二节 W4） |
| 第二轮 W3：甲靠 `source-stamp.py sync` 把 `scripts/hosts/` 拷到第二台 | 站得住（出局对）；改写成按值打包同样打中别的臂，还多三个没列的问题 | `sync` 的目的地是本机目录（`os.makedirs(copy…)`）；只拷 `scripts/hosts/` 起不来，原腿的脚本在现在这棵树上复现退 1。按值打包：python 层的 10 份就有 932,103 字节（`lib_heavy_tests.py` 一份 495,943），整棵进程树打开 13 份 951,447 字节，base64 后 1.2 MB 以上，远超单个参数 131,071 字节；`peer_host_run` 把命令文本当 ssh 的一个参数；`lib_heavy_tests.py` 模块级 `import … tomllib` 要 Python 3.11 以上（第二节 W5） |
| 第二轮 W2：只排一条边，35 个入口只 4 个变 | 站得住 | 登记表现取 41 个也是 4 个变（`out/diff-vs-current.txt`）。35 不是登记表全集：少 `REPLAY_JUDGING_FUNCTIONS` 里 3 个与 3 个 `judge:` 入口，这 6 个都不碰包装，结论不变 |
| 第二轮 W2：闭包收模块级 import 与赋值、不收 class 头（采纳） | 键变的范围按采纳形态量是 34/41，不是判决引的「25 个、多 18 到 45」；不收 class 头与 434 行「不静默少收」打架。这两击我替判决方量实了，不是辩护 | 宽读法（采纳形态）34 个入口全变，多 52 到 523 个单元；窄读法 26 个、多 11 到 45；连 class 头 21 个入口新冒出拒算（第三节 X1） |
| 第二轮 W3/W4：键里不算第二台；条文把文件级排除与边级排除并列 | 站得住；条文还缺一种（class 头不收）、`NOT_JUDGING_FILES` 在 kb 里零命中、三语与三处复述要一起改 | 第三节 X1、第四节 X2 |

## 二、逐条辩护

### W1 兜底范围：只包三处出局，整体兜底采纳

**判定**：站得住。出局这一击我在包装原文上独立复现了；采纳形态里 try 管得了的和管不了的要分开。

推导。`tools/hostlookup_inject.py` 把包装内嵌 python 里 `class HostLookup`（从 `class HostLookup:` 到 `def admit(`）的原文摘出来，换上假 `host-resources.py`（打一份「第二台放得下」的 JSON）、假状态目录，直接调 `begin()` 与 `finish()`，各注入一个点。原样输出（`out/hostlookup-inject.out`）：

```
name=inject point=起线程失败 where=begin() escaped=True exception=RuntimeError
name=inject point=建临时文件失败 where=finish()（退 252 之前，主线程） escaped=True exception=FileNotFoundError
name=inject point=格式化抛 TypeError where=finish()（退 252 之前，主线程） escaped=True exception=TypeError
name=inject point=排队锁不放 where=finish() escaped=True exception=卡住（3 秒后调用还没返回）
name=inject point=等线程的 join 无时限 where=finish() escaped=True exception=卡住（3 秒后调用还没返回）
```

对应原文：`scripts/hosts/run-with-memory-cap.sh:940`：「self.thread.start()」在主线程、没有 try；`scripts/hosts/run-with-memory-cap.sh:1007`：「with tempfile.TemporaryFile(mode="w+", encoding="utf-8") as output, tempfile.TemporaryFile(mode="w+", encoding="utf-8") as errors:」在 `query()` 里、`self.guard` 与 try 之外，而 `finish()` 在 `admit()` 退 252 之前从主线程调它；`scripts/hosts/run-with-memory-cap.sh:947`：「self.thread.join()」与 `scripts/hosts/run-with-memory-cap.sh:959`：「self.thread.join()」都不带时限；排队锁是 `scripts/hosts/run-with-memory-cap.sh:813`：「fcntl.flock(handle, fcntl.LOCK_EX)」的阻塞锁。

所以「只包三处」撑不住，整体兜底方向对。但排队锁不放与 join 无时限这两个注入点是卡住不是抛异常：整段 try 管不了，判决里「排队锁取 LOCK_NB、join 加上限」这半句是这两点的唯一关法，自证的六个注入点要在这两点上造「锁被别的进程占着」与「线程里的 query 睡着」的世界，造成抛异常的假注入会让这两格假绿。

**什么现象会推翻**：真包装在 systemd 世界里，起线程失败或临时文件建不了时退出码并不从 252 变 251（我是把 class 原文摘出来在假世界里调，没起真包装；两轮腿在真包装上量过同一现象，我没重做）。

### W2 前缀去行与源头打标记

**判定**：「凡 `run-with-memory-cap:` 起头一律去」站得住（出局对）；去多这一击够不着「窄正则」版；去少这一击同样打中源头标记版。

去多。升档证据两句都带这个前缀：`scripts/hosts/run-with-memory-cap.sh:1624`：「echo "run-with-memory-cap: 撞了内存上限 $cap（scope $unit 的 Result=oom-kill，命令退出码 $exit_code）" >&2」，`scripts/hosts/run-with-memory-cap.sh:1630`：「echo "run-with-memory-cap: 超过限时 ${TIME_LIMIT_SECONDS} 秒（scope $unit 的 Result=timeout」，广谱变异按这两句认限时与上限（`scripts/mutation/broad-mutation.py:2204`：「return f"超过限时 {limit_text} 秒" if exit_code == TIME_LIMIT_HIT_EXIT else f"撞了内存上限 {limit_text}"」）。宽前缀会把它们一起去掉，这一击成立。

够不着的版本：仓里已经有按窄正则去行的先例，只去排队两种行，不碰证据两句：`scripts/gate/verus-proof-check.py:631`：「WRAPPER_QUEUE_LINE = re.compile(r"^run-with-memory-cap: 排(队（|了 )")」，`scripts/mutation/crates-mutation-rows.py:2378`：「WRAPPER_QUEUE_LINE = re.compile(r"^run-with-memory-cap: 排(队（|了 )")」（同一条正则两份；形式化那份只转打，不从判定输入里去，见 `scripts/gate/verus-proof-check.py:731`：「"json": None, "diagnostics": stderr + "\n" + stdout, "stderr": stderr}」）。HostLookup 打到 stderr 的只有四种行首：`scripts/hosts/run-with-memory-cap.sh:1049`：「say(f"run-with-memory-cap: 没查到别的机器：{problem}")」、`scripts/hosts/run-with-memory-cap.sh:1052`：「say(f"run-with-memory-cap: 现查别的机器：」、`scripts/hosts/run-with-memory-cap.sh:1060`：「line = f"  → {peer['name']}此刻放得下」（不带前缀）、`scripts/hosts/run-with-memory-cap.sh:1078`：「say(f"  ! 告警记录写不进（{type(error).__name__}: {error}）」（不带前缀）；线程里的 traceback 在整体兜底之后不存在。窄正则列这四种就够，不碰证据两句。

去少这一击对两种写法一样中：`say()` 是共用的（`scripts/hosts/run-with-memory-cap.sh:915`：「def say(text):」），HostLookup 之外 `outcome_alerts` 也走它（`scripts/hosts/run-with-memory-cap.sh:1220`：「line = (f"run-with-memory-cap: 这一次峰值 {measured_text}，是记账 {booked_text} 的 {ratio:.1f} 倍：工作量可能变了"」），同一句「! 告警记录写不进」还在 `scripts/hosts/run-with-memory-cap.sh:1200`：「say(f"  ! 告警记录写不进（{type(error).__name__}: {error}）」从别的函数再打一遍。所以「HostLookup 打的每一行」只能逐个调用点打标记，覆盖义务和窄正则列四种一样大，两者都要一格「新增一个 say 调用而没登记就红」的自证。峰值增长那一行随量到的峰值变、也不是证据两句，标记版管不到它；它是否进过库里的内容我没量。

源头标记版自己的代价（判决没列）：`scripts/gate/merge-and-verify.sh:517`：「hint="$(grep -E '^  → 第二台此刻放得下|没查到别的机器：|现查别的机器：' "$logs/$step.log" 2>/dev/null | tail -n 1)"」的头一个备选串锚在行首，给那一行加标记前缀它就不再命中；包装自证里 `host_lookup_cells` 的断言多是子串 grep，不受影响。

**什么现象会推翻**：窄正则版：有一种 HostLookup 行首窄正则列不到、又在整体兜底之后仍会打到 stderr 上（我数的四种来自 `say(` 的调用点，没起真包装逐路径跑）。

### W3 「认 exec_module 字面路径」与模块级语句

**判定**：站得住（打中成立）。替代它的两种读法里，窄读法被打掉，宽读法站得住。

`scripts/gate/lib_function_units.py:7256`：「self.push(loaded.module_body, unit, "load")」按路径载入的模块只推模块体；`scripts/gate/lib_function_units.py:7226`：「if unit.kind != "module-body":」那一段是「任何单元进来就带上本模块的模块体」。我在切边树上看到 `host-dispatch.py:<module-body>` 经 `predicted_bytes` 的 load 进闭包（切边前后都在），而 `lib_long_run.py` 40 个单元与 `memory_peaks_lib.py` 的模块级 import 都掉了：照第一轮判决「让 `exec_module` 连成边」那一件字面做，没有新边可连，等于没改。

两种补法（`tools/make_variants_all.py` 造的两棵树，各对 41 个入口量过，`out/scan-*.tsv`、`out/diff-*.txt`）：

- 窄读法 `narrow`：只有按路径载入的模块、与它们 import 引来的仓内模块，才收模块级 import。切边加这个补丁后 `lib_long_run.py` 的 5 个 import 回来，但 `memory_peaks_lib.py` 的 `import subprocess` 仍不在闭包（check_proof 与 escalate_step 都是 0 个），第二轮判决要的自证「改 `memory_peaks_lib.py` 的 `import subprocess`，包装入口的键变」过不了。
- 宽读法 `modlevel`（任何模块的模块体进闭包，它的模块级 import 与赋值跟着进）：两个自证都过（该 `import subprocess` 单元 1 个、`lib_long_run.py` 41 个单元）。它还和算法文件头自己写的口径一致：`scripts/gate/lib_function_units.py:262`：「模块级语句只在那个模块有单元进闭包时连进来（import 时的副作用照算）」。

**什么现象会推翻**：把 `memory_peaks_lib.py` 那一自证从判决里去掉，窄读法就够用（窄读法只收 import 时 `lib_long_run.py` 在闭包里是 6 个单元，import 与赋值都收是 41 个，见 `out/cut-narrow.out`）。

### W4 按文件排：两个文件的版本

**判定**：够不着。判决拿来打「按文件排」的有两击：会丢掉 `predict` 这类准入单元（对排 `host-dispatch.py` 成立），和 13-验证路线.md 的「文件级」一句。排的若是 `scripts/hosts/host-resources.py` 与 `scripts/hosts/peer-host-lib.sh` 这两份，丢准入单元这一击碰不到：切边之后这两份的单元一个都不在闭包里（`out/scan-cut-src.tsv` 里 host-resources 单元 0），说明现状里它们只经查别的机器那一条边进来。「文件级」那一击：`.claude/kb/decisions/13-验证路线.md:433`：「文件级、包级、整表级的指纹或闭包」写在「一律不进键」那一句里，管的是拿文件级指纹或闭包当键的输入；排除一个文件不让文件指纹进键，和判决同意并列的 `NOT_JUDGING_FILES`（`scripts/crash/lib_verdict_library.py:88`：「NOT_JUDGING_FILES = ("scripts/crash/lib_verdict_library.py", "scripts/gate/lib_function_units.py")」）是同一类，判决自己也不拿这一句去打边排除。

量过（`tools/fileexcl_key.py`，用 `scripts/crash/lib_verdict_library.py:205`：「merged.refusals.update(refusal for refusal in closure.refusals if refusal.file not in excluded_files)」那个现成的文件级参数，在 `NOT_JUDGING_FILES` 之外再排那两份）。原样输出，`out/fileexcl-current.txt`：

```
check_proof	键=d5ea05474e11246c	单元=722	拒算=0
check_vacuity	键=789d99fd65b8dd39	单元=711	拒算=0
check_must_red_worlds	键=16b2fb91efcbb2c4	单元=719	拒算=0
```

改一处再算，`lib_long_run.py` 的 `import re` 写成 `import re as re`（`out/fileexcl-mut-a.txt`）三个键变成 `1b09f8419e8d5340`、`5342dc53c6d1d66c`、`c149539f8f4edc6a`；`memory_peaks_lib.py` 的 `import subprocess` 写成 `import subprocess as subprocess`（`out/fileexcl-mut-b.txt`）变成 `f65fe571a8a498d9`、`a612223ddaf86845`、`5988bc6f079e8dc3`。即：不改闭包算法，三个形式化入口拒算消失，第二轮判决给的两个键自证照过，四个入口（三个形式化加 `Runner.escalate_step`）以外的键一个不变。

它同样打中的（别的臂也一样）：「只打提示、不改结局」那个前提（第一轮 V1 内容与结局），要靠整体兜底、去行与自证来立；它也不修模块级盲区（但保留了现状里经那条宽边带进来的单元，所以这个盲区在这一臂上不暴露）。它的缺点是键里留着 267 个只经那条边进来的单元（切边树比它少），改它们会白白让形式化判定作废；以及广谱变异的 `Runner.escalate_step` 用的是自己的 `closure_key`（`scripts/mutation/broad-mutation.py:1466`：「key, refusals = library.closure_key(closure, allow_refused=True)」，GPU 那一把在 3417 行同形），没有排除文件参数，要在驱动里另写一个过滤。

零轮形态（没被任何一轮攻过），只是替「被判出局」的一方找站得住的理由。**什么现象会推翻**：有一个登记入口经别的路径也真走到 `host-resources.py` 或 `peer-host-lib.sh` 里的判结局代码（41 个入口里只有那 4 个到得了，我按切边前后单元差量的，没起真包装逐路径跑）。

### W5 甲靠同步，以及按值打包

**判定**：站得住（出局对）；按值打包同样打中别的臂，还多三个问题。

出局对：`scripts/hosts/source-stamp.py:186`：「os.makedirs(copy, exist_ok=True)」`sync` 的目的地是本机目录；第二台只放每一趟的树副本，`multi-host.env.example:11`：「每一趟在它下面建一个子目录放树与编译目录、跑完删掉」，探测排队时没有副本。第二轮腿的脚本在现在这棵树上复现（只拷 `scripts/hosts/` 的树起 host-resources.py）：

```
name=needed_files full_tree_rc=0 repo_files=10 outside_scripts_hosts=7
name=only_scripts_hosts rc=1 stdout_is_json=False last_stderr_line=ModuleNotFoundError: No module named 'project_preflight'
```

按值打包（第二轮判决的甲）的问题，量过：

- 要送的不止 10 份。我用 `strace -f` 抓整棵进程树打开的仓内文件（`tools/needed_files_strace.sh`，含 bash source 进来的）：13 份、9 份在 `scripts/hosts/` 之外，共 951,447 字节（`out/rerun-1.out`：「name=needed_files_strace files=13 outside_scripts_hosts=9 bytes=951447 base64_bytes=1268596 single_argument_limit=131071」）。第二轮腿的 10/7 是它自己脚本声明的下界（只记 python 层打开与子进程参数点名的）；那 10 份的字节数加起来是 932,103。
- 「probe 一台自己」的模式要带的可能少于 13 份（13 份是整份 `host-resources.py` 带多机配置跑一趟打开的）。但只要载 `host-dispatch.py` 就要 import `lib_long_run.py`，后者按路径载 `lib_heavy_tests.py`（495,943 字节）；判决写的「连同它在仓内 import 的文件」这个口径本身就超限，缩小要先改口径、写明哪个模式。
- 送不过去：`peer_host_run` 把整条命令当 ssh 的一个参数（`scripts/hosts/peer-host-lib.sh:77`：「ssh -o BatchMode=yes "$PEER_SSH_HOST"」），Linux 单个参数 131,071 字节装得下、131,072 起 E2BIG（我在本机 `/bin/true` 上试过：131071 通过、131072 与 200000 失败）。今天送的探测程序只有 23,800 字节、base64 31,736 字节（`scripts/hosts/host-resources.py:230`：「return "\n".join([library, inspect.getsource(probe_this_host), PROBE_DRIVER])」，`out/rerun-1.out`：「name=remote_program_today bytes=23800 base64_bytes=31736 single_argument_limit=131071 fits=True」），装得下。
- 版本：`.claude/hooks/lib_heavy_tests.py:623`：「itertools, os, re, shlex, shutil, stat, subprocess, sys, tempfile, tomllib」模块级 import `tomllib`（Python 3.11 起），`scripts/hosts/core_usage.py` 要 `zoneinfo`（3.9 起）；今天送的那段只用 `memory_peaks_lib.py` 的标准库。全部是标准库，没有要 pip 装的包，所以「缺包」不会发生，「版本低」会：第二台 Python 低于 3.11 时按值打包的整份在第二台上 import 就失败。第二台的 Python 版本我没法量。
- 判决写「键里不算第二台（送过去的文件已在本机闭包里）」：切边之后 `host-resources.py` 的单元在 check_proof 与 escalate_step 的闭包里是 0 个（`out/membership.out`：`tree=cut-src` 与 `tree=cut-modlevel` 两行 `host_resources_units=0`），送过去的文件是被排除在键外，不是「已在闭包里」。两者落在同一个前提上（第二台那一支只打提示、不改结局），理由要照这一句改写，不然读的人会以为按值打包的文件进了键。
- 今天已经是「两台跑同一段文本」：送过去的是本机 `memory_peaks_lib.py` 的源码加 `probe_this_host` 的 `inspect.getsource`，不是第二台自己的一份。如果甲要的只是这一点，现状加切边就满足，键里本来就不用算第二台。

同步（rsync 经 `peer_host_copy`）这一臂不碰 E2BIG，但要一个常驻的探测目录，与「跑完删掉」的约定冲突，第二台版本落后时也没有人管。

**什么现象会推翻**：第二台上有同版本常驻仓，或甲的本意只是现状（那么 W5 的按值打包整段不需要）。

### W6 只排一条边与入口清单

**判定**：站得住。登记表现取的 41 个入口也是 4 个变，原样见第五节 X3。35 个清单不是登记表全集：`scripts/gate/admission.py:372`：「REPLAY_JUDGING_FUNCTIONS = ("replay_judge_fresh_product", "strip_admission_header", "strip_timing", "strip_load_fields", "timing_load_reading_raw", "timing_load_decide")」六个，清单里只有三个；另有 `judge:` 入口 `lib_libtest_result_lines.py::result_line`、`::recordable_results` 与 `harness-model-differential-and-scenarios.sh::model_section_verdict`（`.claude/gate.d/checker-tier-research-build-and-replay.sh`、`checker-tier-crash-point-replay.sh`、`harness-model-differential-and-scenarios.sh` 各自带的 `judge:` 令牌）。这 6 个量过都不碰包装（`scan-current.tsv` 里包装单元 0）。**什么现象会推翻**：登记表以外还有别的入口经 python 或 shell 到得了包装。

## 三、X1：「已定」项撑不撑得住

**结论**：已定项 17 撑得住「键只由函数单元组成」；被判形态里有与它的字面打架的有 class 头不收；它没有授权的有排除一条认得出的调用边；判决引的代价数（25 个）不是采纳形态的数（实际 34 个）。三处都不是辩护，是我替判决方量实的。

逐句读已定项 17（`.claude/kb/decisions/13-验证路线.md` 第 429–436 行那一节，从头读到 `## 历史版本` 之前）：

- 收的口径。`.claude/kb/decisions/13-验证路线.md:432`：「函数单元从这份判定的入口沿调用与引用收，连同引用的常量、类型、静态项与宏展开一起算」。python 的模块级 import 与赋值在这句里就是「引用的常量」和 `use` 一类。`.claude/kb/decisions/13-验证路线.md:433`：「键过窄（引用的常量、类型、宏漏收）」算违反，所以今天闭包不收模块级语句就是现存的违反：现状树里 check_proof 闭包的 17 份 `.py` 缺 348 个 import、赋值、class 单元（`out/module-census-current.out`）；切边又把 `memory_peaks_lib.py` 的 `import subprocess` 这类单元挤出闭包（`out/membership.out`：现状树该单元在闭包里，切边树不在）。宽读法是在还债，不是新增规则；算法文件头早就这么写：`scripts/gate/lib_function_units.py:262`：「模块级语句只在那个模块有单元进闭包时连进来（import 时的副作用照算）」。这一条支持宽读法，不支持窄读法。
- 不静默少收。`.claude/kb/decisions/13-验证路线.md:434`：「的那一份拒算键、照跑，不静默少收」。被判形态「不收 class 头」是一个认得出、因为一收就拒算才不收的形态，等于静默少收：class 体里的模块级语句（类属性、装饰器、基类表达式）改了键不变。我量了收进来会怎样（`tools/make_variants_all.py` 的 `cut-modlevel-class`，`out/diff-class-heads.txt`）：

```
DIFF cut-modlevel-class: 入口 41 个，集合或退出码变了 34 个，不计算法文件自己之后真变的 34 个，拒算与基准不同的 21 个
```

  21 个入口（广谱变异 15、crates 变异 3、形式化 3）由 rc 0 变 rc 4，拒算形态是 `shell-path-unfolded @ host-dispatch.py:301/318` 与 `subprocess-command-unfolded @ broad-mutation.py` 等（原因推的：收进 class 头就触发 `class_entered`，把 `Peer.__init__` 一类带 `bash -c` 的方法拉进来；没跑 `--explain` 核）。两条出路：收进来并接受 21 个入口每趟现判（等于这批复用全丢），或把「class 体里的模块级语句不进键」另登记成一种排除、写进条文、配一格自证（第二轮判决的条文只列了文件级与边级两种）。判决现在的写法两样都不是。
- 拒算的照跑。`scripts/mutation/broad-mutation.py:1466`：「key, refusals = library.closure_key(closure, allow_refused=True)」（`:3417` 同形）。今天 `Runner.escalate_step` 带着拒算照样出键并沿用，与 434 行不符；切边之后这一个入口的拒算消失（rc 4 变 0），违反只是睡着，`allow_refused=True` 还在。这条判决已记欠账，我补一句：它不随这次改动关闭。
- 作废不算代价。`.claude/kb/decisions/13-验证路线.md:436`：「改键让旧库作废不算代价：不做兼容迁移、不为旧行补签，改完重跑。」，所以键变的规模不挡这一改，但判决要报对数：采纳形态（宽读法、不收 class 头）下登记入口 34/41 全变（只剩 7 个 shell 入口不变），多 52 到 523 个单元，新拒算 0。判决引的「25 个入口的键变、多 18 到 45 个单元」是窄读法、不切边的数（第五节 X3 核过，能复现），不是采纳形态的数。
- 边排除没有条文授权。432 行是「沿调用与引用收」；434 行只讲认不出的拒算。一条认得出、因为不影响结局而排掉的调用边，已定项没说可不可以。`.claude/kb/decisions/13-验证路线.md:446`：「用户定改完代码开三轮攻方，只攻」`.claude/kb/decisions/13-验证路线.md:446`：「剪枝是不是原子化的、键是不是只由函数单元组成」——边排除后键仍只由函数单元组成，站得住；「剪枝是不是原子化」靠的是实测自证（只打提示、不改结局），不是键的组成，已定项的字面没法替它担保。并列的文件级排除也只在代码里：`NOT_JUDGING_FILES` 在 `.claude/kb/` 下零命中（`grep -rln NOT_JUDGING_FILES .claude` 只出 `scripts/crash/lib_verdict_library.py` 与 `scripts/experiment-apparatus/e165-pin-red-examples.py`），它的理由写在 `scripts/crash/lib_verdict_library.py:20`：「它们管怎么存、怎么算键，不管判对错，改它们不让任何判定作废。」——是按角色排（不管判对错），边排除是按实测排（不改结局），两条理由不同，条文要分开写，不能写成「同一类」。

与别处条文的字面关系：`.claude/rules/implementation-workflow.md:31`：「键只由这条判定依赖的函数单元各自去注释后的指纹」与 `.claude/rules/verification.md:50`：「单元键只由判定依赖的函数单元各自去注释后的指纹」是已定项 17 的复述；排除之后键里剩下的仍都是函数单元，字面不打架，但条文改动要三处同改（再加 kb 的英文、日文两份）。

**什么现象会推翻**：class 头不收这一条：有人证明 class 体里的模块级语句不可能影响任一登记判定的结局（那就是「过宽」而非「静默少收」）；边排除一条：条文改成明文授权「认得出且自证不改结局的调用边可排」并经三轮攻。

## 四、X2：未定项清单的完整性

判决第二轮「另登欠账」只有两行，被判形态要随之动的地方比它多。逐项现查：

- 读包装输出的判法调用点，不止「形式化三个、广谱变异、crates 变异」：形式化 `scripts/gate/verus-proof-check.py:731`、报错尾 `scripts/gate/verus-proof-check.py:1339`：「tail = " | ".join(line for line in outcome.get("stderr", "").strip().split("\n")[-6:] if line.strip())」；crates 变异 `scripts/mutation/crates-mutation-rows.py:130`：「这里收包装的 stderr 判结局」；广谱变异 `scripts/mutation/broad-mutation.py:2204`（升档证据）；形式化钉红例 `scripts/experiment-apparatus/e165-pin-red-examples.py:1184`：「tail = " | ".join(line for line in completed.stderr.strip().split("\n")[-4:] if line.strip())」（`:1193` 同形，只进红句，不入库）；以及不是判法的读者 `scripts/gate/merge-and-verify.sh:517`（行首锚，见 W2）、`scripts/agents/agent-watch.py`（读告警记录里的 `line` 字段，给 `line` 加标记会进看门狗的话；那个文件别的会话仍在改，行号待查）。
- `say()` 共用与同句两处（W2）：标记要逐调用点打，另配「新增 say 而没登记就红」的自证。
- 整体兜底的另一半：LOCK_NB 取不到锁等于「这一趟不查」，`take_turn`（`scripts/hosts/run-with-memory-cap.sh:963`：「def take_turn(self):」）现有的「刚查过」原因在 `scripts/hosts/run-with-memory-cap.sh:978`：「return (f"{now - last['at']:.0f} 秒前包装 {last['pid']} 刚查过」，要加一种「锁被占着」的原因行，对应自证（`scripts/hosts/run-with-memory-cap.sh:1776`：「host_lookup_cells() {」起）要造被占着的世界；join 加时限后线程可能还活着，`finish()`（`scripts/hosts/run-with-memory-cap.sh:942`：「def finish(self, local):」）与 `cancel()`（`scripts/hosts/run-with-memory-cap.sh:951`：「def cancel(self, admitted):」）里不带时限的两处 `join`（`:947`、`:959`）要跟着改。
- 按值打包要带的文件：13 份，951,447 字节，逐份见 `out/rerun-1.out`（`.claude/hooks/lib_heavy_tests.py` 一份就 495,943 字节）；传输要换：`peer_host_run` 一个参数装不下（W5）；第二台 Python 要 3.11 以上（`tomllib`）；探测每台时限只有 8 秒（`scripts/hosts/run-with-memory-cap.sh:776`：「HOST_LOOKUP_PEER_TIMEOUT_SECONDS = 8」），传 1 MB 再起 Python 装不装得进 8 秒我没法量（要第二台）。判决里甲「不依赖第二台有仓、也不依赖同步」成立，但留下传输、版本、时限三个没写的。
- `host-dispatch.py` 一路：`scripts/hosts/host-dispatch.py:316`：「runner = 'source "$1" || exit 70; eval "$2"; peer_host_run / "$3"'」，它自己的探测程序也是 base64 经同一条通道（`scripts/hosts/host-dispatch.py:346`：「program = base64.b64encode(peer_probe_program().encode("utf-8")).decode("ascii")」），装置显存那一路的程序大小我没量；`Peer.__init__` 的 `bash -c`（`scripts/hosts/host-dispatch.py:295`）是 class 头一收就拒算的那两处。
- 自证与登记：`.claude/hooks/selftest-registry.tsv` 里 `scripts/hosts/run-with-memory-cap.sh` 一行是 light，改过的脚本的自证不在副本里起，要写进「要主 agent 跑的命令」；`.claude/gate.d/verdict-library-scripts.tsv` 里 `verus-proof-check.py`、`crates-mutation-rows.py`、`broad-mutation.py` 三行各有一格 `reuse-three-questions`，宽读法让它们闭包里多了模块级单元，「闭包外改动之后现判 0」那一问的造法要逐格重核；门禁 code-tooling 的 reuse-selftests 格是 `.claude/gate.d/code-tooling.sh:3567`：「cell_reuse_selftests() {」；`scripts/gate/lib_function_units.py` 的 `CONTROL_ENTRIES`（第 7643 行）与 `--selftest`、`scripts/crash/lib_verdict_library.py --selftest` 都要重跑。
- 文件头与条文落点：`scripts/hosts/run-with-memory-cap.sh` 文件头「排队时去找别的机器」、`scripts/hosts/host-resources.py` 文件头、`scripts/gate/lib_function_units.py:262` 的口径句、`scripts/gate/verus-proof-check.py:94` 与 `scripts/mutation/broad-mutation.py:91` 的「判法单元指纹」段、`scripts/crash/lib_verdict_library.py:20` 的 `NOT_JUDGING_FILES` 理由；条文：`.claude/kb/decisions/13-验证路线.md`、`13-verification-route.en.md`、`13-検証ルート.ja.md` 三份，加 `implementation-workflow.md:31` 与 `verification.md:50` 两处复述。
- 入口清单本身：判决用的 35 个是手抄清单；登记表现取 41 个（W6）。清单应当由登记表机器生成，不手抄（取法见 `tools/gen_entries_registry.py`，输出与 `tools/entries-registry.txt` 逐行相同，`out/entries-registry.gen.txt`）。

**什么现象会推翻**：上面任一项在主工作区现查不存在（我引的行号是交回时现查的，别的会话仍在改 `scripts/mutation/crates-mutation-rows.py`、`scripts/hosts/peer-host-lib.sh` 等）。

## 五、X3：前两轮报出的数，逐个核

核法与原腿不同的地方：入口清单由登记表现取（41 个，`tools/gen_entries_registry.py`，与手存的 `tools/entries-registry.txt` 逐行相同）；切边用三种互不共用的做法——源码层（把 `HostLookup.query` 里起 `host-resources.py` 的 `Popen` 换成 `None`，与原腿同法，作对照）、算法层（`python_edges` 里跳过 `HostLookup.query` 连到 `host-resources.py` 的那一条 program 边）、桩层（`host-resources.py` 换成一行注释，边还在、被连的文件没内容）；单元数我自己解析 `lib_function_units.py closure` 的输出，不用原腿的 awk；模块级缺口另用 `ast` 数了顶层 import 与赋值（`out/module-census.out` 每份文件一行）。树漂移说明见第〇节。

### 核上的

**「单元 1579 → 1232」「792 → 436」「35 个入口只 4 个变」——核上。** 三种切法都得同一个数：源码层 1232、算法层 1232、桩层 1233（桩多一个 `<module-body>`）。`out/diff-vs-current.txt` 原样（`DIFF cut-src` 一段）：
    DIFF cut-src: 入口 41 个，集合或退出码变了 4 个，不计算法文件自己之后真变的 4 个，拒算与基准不同的 0 个
      scripts/mutation/broad-mutation.py::Runner.escalate_step rc 4→0 原始行 792→436 去重 791→435 文件 24→15 拒算[inline-script-not-literal@scripts/hosts/peer-host-lib.sh:76]→[] 掉357 多1（不计算法文件：掉357 多1）
      scripts/gate/verus-proof-check.py::check_proof rc 4→0 原始行 1579→1232 去重 1578→1231 文件 25→17 拒算[inline-script-not-literal@scripts/hosts/peer-host-lib.sh:76]→[] 掉348 多1（不计算法文件：掉347 多1）
      scripts/gate/verus-proof-check.py::check_vacuity rc 4→0 原始行 1567→1220 去重 1566→1219 文件 25→17 拒算[inline-script-not-literal@scripts/hosts/peer-host-lib.sh:76]→[] 掉348 多1（不计算法文件：掉347 多1）
      scripts/gate/verus-proof-check.py::check_must_red_worlds rc 4→0 原始行 1575→1228 去重 1574→1227 文件 25→17 拒算[inline-script-not-literal@scripts/hosts/peer-host-lib.sh:76]→[] 掉348 多1（不计算法文件：掉347 多1）
原始行数里 check_proof 1579、check_vacuity 1567、check_must_red_worlds 1575，切边后各少 347；去重后（种类、限定名、文件、指纹）少 1，是 `run-with-memory-cap.sh` 里 `PEAK_TABLE` 赋了两次、指纹相同。其余 37 个入口一个单元没动（`DIFF cut-src` 头行写「变了 4 个」）。算法层切法让另外 6 个入口各掉 1 多 1（`DIFF cut-algo` 头行写变了 10 个、不计算法文件自己真变 4 个），是我补丁改了 `lib_function_units.py` 自己的一个单元，不是闭包变了。

**「check_proof 缺 164 个、escalate_step 缺 290 个模块级单元」——核上，但要补三点：树、口径、扫描范围。** 原样（`out/module-census.out`，在切边树上）：
    check_proof py文件=12 单元缺口(按 dump) assign=106 class=8 import=50 main-guard=9 合计(import+assign+class)=164 合计(import+assign)=156
    Runner.escalate_step py文件=10 单元缺口(按 dump) assign=213 class=22 import=55 main-guard=8 合计(import+assign+class)=290 合计(import+assign)=268
树：164 与 290 是切边树上的数，现状树上是另外的数（`out/module-census-current.out`）：
    check_proof py文件=17 单元缺口(按 dump) assign=271 class=17 import=60 main-guard=11 合计(import+assign+class)=348 合计(import+assign)=331
    Runner.escalate_step py文件=16 单元缺口(按 dump) assign=378 class=31 import=64 main-guard=10 合计(import+assign+class)=473 合计(import+assign)=442
口径：164 与 290 把 class 头算进去了（8 个、22 个）、没算 `main-guard`；判决要收的是 import 与赋值，直接缺口是 156 与 268。扫描范围：原腿的 dump 只扫 `scripts`、`.claude/hooks` 等几个目录，我的 dump 扫了全仓 python，切边树上两边得同一个数，说明闭包里的 12 份、10 份 `.py` 都在它的扫描范围里。ast 独立数（每份文件顶层 import 名、赋值名、class 数）在同一份输出里逐文件列了。

**「1232 → 1655」——核上（宽读法，切边再收模块级 import 与赋值）。** 原样（`out/diff-cut-modlevel.txt`）：
      scripts/mutation/broad-mutation.py::Runner.escalate_step rc 0→0 原始行 436→959 去重 435→958 文件 15→19 拒算[]→[] 掉0 多523（不计算法文件：掉0 多523）
      scripts/gate/verus-proof-check.py::check_proof rc 0→0 原始行 1232→1656 去重 1231→1655 文件 17→20 拒算[]→[] 掉1 多425（不计算法文件：掉0 多421）
去重后 check_proof 是 1231 变 1655；原始行 1656 比 1655 多的 1 个是我的补丁给 `lib_function_units.py` 自己加的一个模块级赋值。`Runner.escalate_step` 436 变 959 也对上。这个数是宽读法的数，不是下一条「25 个入口」那个补丁的数，两条在判决里挨着写，前后不是同一种改法。

**「776 个单元」——核上（切边树）。** 原样（`out/not-judging-files.out`）：
    name=not_judging_files tree=current entry=check_proof lib_verdict_library_units=62 lib_function_units_units=715 total_distinct=1578
    name=not_judging_files tree=cut-src entry=check_proof lib_verdict_library_units=62 lib_function_units_units=714 total_distinct=1231
    name=judging_units_lines tree=cut-src entry=check_proof lines=456
切边树上这两份合计 62 加 714 等于 776；现状树上是 777。`lib_verdict_library.py judging-units` 在切边树上打出 456 行单元，等于 1232 减 776。

**「仓内 10 份、7 份在外」——核上，是下界。** 用原腿的脚本在现在这棵树上重跑（`out/opus-w3-needed-files.out`）：
    name=needed_files full_tree_rc=0 repo_files=10 outside_scripts_hosts=7
    name=only_scripts_hosts rc=1 stdout_is_json=False last_stderr_line=ModuleNotFoundError: No module named 'project_preflight'
原腿脚本自己写明只记 python 层打开的文件与子进程参数里点名的脚本。我用 `strace -f` 抓整棵进程树（`tools/needed_files_strace.sh`，含 bash source 进来的），得 13 份、9 份在外，多出的三份是 `.claude/scripts/preflight.sh`、`.claude/singlefs-ai-sop/scripts/preflight.sh`、`scripts/hosts/heavy-threads.sh`（`out/rerun-1.out` 末段，清单逐份带字节数；该文件里有几处被复跑脚本里的 `cut` 切断的多字节字符，读它用 `grep -a`）：
    name=needed_files_strace files=13 outside_scripts_hosts=9 bytes=951447 base64_bytes=1268596 single_argument_limit=131071

### 核不上的

**「25 个入口的键变、多 18 到 45 个单元」——核不上：现在这棵树上是 26 个。** 原样（`out/compare-leg-scan.out`，第二轮正推腿 `SCAN load-edge` 那一段对我实现的同一种补丁）：
    name=compare_leg_scan 现在这棵树上变的入口=26 腿报的=25
      仅现在有： ['scripts/experiment-apparatus/e165-pin-red-examples.py::replay']
      仅腿有： []
      两边都有的 25 个：增量范围 18~45；单元数不同的：check_must_red_worlds 现在(1575, 1607) 腿(1575, 1606)；check_proof 现在(1579, 1611) 腿(1579, 1610)；check_vacuity 现在(1567, 1599) 腿(1567, 1598)；GpuJudging.row_holds 现在(888, 921) 腿(888, 920)；Judgement.row_holds 现在(948, 990) 腿(948, 989)；Judgement.selection 现在(943, 985) 腿(943, 984)；Judgement.state 现在(963, 1005) 腿(963, 1004)；UnitTree.closure_key 现在(863, 885) 腿(863, 884)
25 个入口的名字与增量范围 18 到 45 对上；8 个入口单元数多 1，是我的补丁给算法文件自己加了一个赋值。多出来的 e165 `replay`：用原腿自己的 `make_trees.py` 在现在这棵树上造 load-edge 树再收，也是变的（`out/e165-load-edge.out`）：
    # 原腿 make_trees.py 的 load-edge 补丁在现在这棵树上对 e165 replay 入口的结果（入口,rc,原始行,去重）与现状
    his-load-edge:
    scripts/experiment-apparatus/e165-pin-red-examples.py::replay 0 949 949
    current:
    scripts/experiment-apparatus/e165-pin-red-examples.py::replay 0 939 939
所以不是我补丁的差别，是原腿量时那棵冻结副本里 `e165` 入口不经 `lib_function_units.py` 的载入、现在经（原因我没追，推的）。更要紧的是这个数属于另一个补丁：窄读法、不切边；采纳形态（宽读法、切边）是 34 个入口、多 52 到 523 个单元，原样（`out/diff-cut-modlevel.txt` 头行与 `out/diff-vs-current.txt` 的 `DIFF narrow` 头行）：
    DIFF cut-modlevel: 入口 41 个，集合或退出码变了 34 个，不计算法文件自己之后真变的 34 个，拒算与基准不同的 0 个
    DIFF modlevel: 入口 41 个，集合或退出码变了 34 个，不计算法文件自己之后真变的 34 个，拒算与基准不同的 0 个
    DIFF narrow: 入口 41 个，集合或退出码变了 26 个，不计算法文件自己之后真变的 26 个，拒算与基准不同的 0 个
判决第三节写的「25 个入口的键变」与「1232 → 1655」不是同一种改法的数，交用户或写进条文时要分开报。

**「仓内 10 份、7 份在外」——作为完整清单核不上**，是下界（13 份、9 份，见上）。作为「只拷 `scripts/hosts/` 起不来」的证据，核上：`last_stderr_line=ModuleNotFoundError: No module named 'project_preflight'`（`out/opus-w3-needed-files.out` 末行）。

### 没核

- 第一轮「改坏 `memory_peaks_lib.py` 的 import，包装从退 0 变 251」「`lib_long_run.py` 让 import 失败，包装从退 0 变 252」：要真包装加 systemd scope，我没起；键那一半核上（切边树里该 import 单元不在闭包：`out/membership.out`、`out/cut-narrow.out`）。
- 第二轮「TypeError 让包装 252 变 251」：我只在 `HostLookup` 原文上复现了异常冒到调用方（W1），没起真包装看退出码。
- 第二台上的任何数（传输用时、Python 版本、8 秒时限够不够）：没第二台可量。

### 数字一览

| 数 | 判 | 出处 |
|---|---|---|
| 1579 → 1232（check_proof）、1567 → 1220、1575 → 1228 | 核上 | `out/diff-vs-current.txt` 的 `DIFF cut-src` |
| 792 → 436（escalate_step） | 核上 | 同上 |
| 41 个入口（原 35 个）只 4 个变 | 核上 | 同上头行 |
| 164 / 290（切边树，含 class 头） | 核上；现状树是 348 / 473 | `out/module-census.out`、`out/module-census-current.out` |
| 1232 → 1655、436 → 959 | 核上（宽读法） | `out/diff-cut-modlevel.txt` |
| 776 | 核上（切边树；现状树 777） | `out/not-judging-files.out` |
| 25 个入口、多 18 到 45 | 核不上：26 个（窄读法，另一个补丁） | `out/compare-leg-scan.out`、`out/e165-load-edge.out` |
| 仓内 10 份、7 份在外 | 下界核上，完整清单是 13 份、9 份 | `out/opus-w3-needed-files.out`、`out/rerun-1.out` |

**什么现象会推翻第五节的核结论**：别的会话在我量完之后又改了这些文件里闭包会走到的东西（`.claude/hooks/lib_heavy_tests.py` 已经在第二轮腿跑完之后动过一次）。复跑原样输出与 `out/` 里留的比；不一样先看树漂移，再看补丁。

## 六、新冒出的零轮形态（要交用户表时标「零轮」）

这几条是我在替被判出局的一方找站得住的理由时造出来的，没被任何一轮攻过，不是我的推荐，只是判决写交用户表时要有的选项：

| 形态 | 它回应判决里哪一击 | 我量到的 | 没量的 |
|---|---|---|---|
| 文件级排除 `host-resources.py` 与 `peer-host-lib.sh`（用 `lib_verdict_library` 现成的 `excluded_files`） | 第一轮 V2 按文件排出局；第二轮闭包算法要改、键变 34 个入口 | 三个形式化入口拒算 0、键算得出，两个模块级 import 自证照过（`out/fileexcl-*.txt`）；只动 4 个入口的键 | 广谱变异 `Runner.escalate_step` 要在驱动里另写过滤；键里多留 267 个单元；「只打提示」前提仍要整体兜底与去行来立 |
| 去行用窄正则（仓里已有 `WRAPPER_QUEUE_LINE` 先例），列 HostLookup 的四种行首 | 第二轮 W1 按前缀去行出局 | 四种行首来自 `say(` 调用点（W2） | 没起真包装逐路径跑；不改 `merge-and-verify.sh:517` |
| class 体里的模块级语句不进键，另登记成一种排除并写进条文 | 采纳形态与 434 行「不静默少收」打架 | 收进 class 头就有 21 个入口新拒算（`out/diff-class-heads.txt`） | 条文怎么写、自证怎么造 |
| 甲保持现状：送过去的是本机源码的 `inspect.getsource(probe_this_host)` 加 `memory_peaks_lib.py` | 第二轮 W3 甲打穿；按值打包的传输、版本、时限三个没写的问题 | 今天送的程序 23,800 字节，装得下；全是标准库 | 用户要的「同一份仓里的脚本、由参数选机器」是不是这一点我没法判 |

## 七、没做什么

- 不替主 agent 撤销或维持判决，只交复核结果。
- 没起真包装（systemd scope）：第一、二轮的退出码现象（252 变 251、0 变 251/252）只在 `HostLookup` 原文上复现了异常冒到调用方，没看真退出码。
- 没连第二台：传输用时、Python 版本、8 秒时限都没量；按值打包的 E2BIG 只在本机 `/bin/true` 上试了 execve 的单参数上限（`out/arg-limit.out`）。
- 没跑任何自证（`run-with-memory-cap.sh --selftest`、`lib_function_units.py --selftest`、各 `reuse-three-questions` 格）：它们归主 agent 在提交那一趟跑；改过的脚本的自证不在副本里起。
- 没跑重型测试、编译、门禁阶段。
- 没改主工作区：所有补丁树都建在 `~/.local/share/singlefs/work/m3-close-reuse-closure-r3-sonnet/` 下、交回前已删，变体树是用 `tools/make_variants_all.py` 现造的，只用来量，不是实现；`research/prompts/m3-close-reuse-closure-r3-sonnet-model/` 与本报告是写进仓里的仅有两处。
- 没查「峰值增长那一行」（`outcome_alerts`）是否进过库里的内容。
- 没追 e165 `replay` 入口在原腿冻结副本里不变、现在变的原因。
- 没读禁读清单里的任何文件。
