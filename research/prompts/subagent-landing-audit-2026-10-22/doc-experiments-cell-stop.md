# doc-experiments 加格：登记里的正式跑在单测单元表与 replay.sh 都找得到（停在出口 1，没改任何文件）

日期：2026-10-22。结论：这一格现在不能按字面判，停下交回。

## 现查的形态（命令在仓根）

- 「重跑登记的比对对象表」：`grep -n '比对对象' research/prompts/e165-r{1..5}-prereg.md e165-preregistration.md` 只在 E165 的五份重跑登记里命中，别的实验没有这张表。五份的写法各不相同：
  - r1 只在统计量表里有一行「R1 的比对对象份数」，指「第一节比对象表」。
  - r2 第 565 行起是散文加表，点名 `replay.sh:222–224`。
  - r3 第 633 行是一段行内散文，三份产物的 sha256 写在句子里。
  - r4 第 845 行起是四列表（产物、sha256、行数、参数），参数列写 `replay.sh:222`、`--bounds segment1 --family adversarial --profile r2` 之类。
  - r5 第 662 行起是七列表（#、比对对象、sha256、行数、`replay.sh` 行 / 跑法、重跑产物名、备注），「replay.sh 行」一列写行号加驱动函数名、「无复跑行（config 行）」、或一句带参数的跑法。
  - 没有固定的列头、固定的列位置，也没有机器可读的登记键。表里写的 `replay.sh` 行号随该脚本改动漂移（r4 写 222–224、r5 写 224–226，指的是同样三行）。
- 「单测的单元表」：`grep -rln 'formal_run_units' crates scripts` 只命中 `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs` 一处（测试 `every_criterion_has_a_positive_control_and_every_control_is_red`，`let formal_run_units: [(&str, Family, RunProfile, &[Criterion]); 19]`，19 个单元）。它是 Rust 数组字面，单元与登记、`replay.sh` 的对应只写在每个元组上面的一行注释里（「`scripts/experiment-apparatus/replay.sh` 的 `driver_e165`……；产物 `…out`」或「`research/prompts/e165-r5-prereg.md` 比对对象表第 4 行（`replay.sh` 里没有复跑行）」）。别的实验没有这种单元表。
- `replay.sh` 复跑表：固定形态（`E165|@driver_xxx||<产物名>|exact`，E165 共 26 行，`grep -c '^E165|@driver' scripts/experiment-apparatus/replay.sh` 可核）。三处里只有这一处有固定形态。

## 判不了的原因

1. 「每份重跑登记的比对对象表」只有 E165 有，五份五种写法；按字面写成跨实验的一格，实际只对一个实验、一份登记（r5）的七列散文表做列位置解析，r5 之后再有登记换一种写法就静默漏判（扫到 0 项）。
2. 「单测的单元表」是一个 Rust 测试里的数组，不是文本表；格要解析 Rust 源码里的元组与注释才能取到「单元对应哪条正式跑」，注释是自由文本。
3. 已知的 4 个没有复跑行的单元（r5 比对对象表第 4–7 行）在单元表里都在（注释写明「`replay.sh` 里没有复跑行」），在 `replay.sh` 里没有。要判「单元表和 `replay.sh` 都找得到」，这 4 行当前就是红，滞后表登记它们没问题；但格本身没有固定输入可读。

## 缺什么固定形态、建议怎么定

建议先立一张机读登记，再立格，不去猜解析：

- 新增 `.claude/gate.d/replay-comparison-units.tsv`（或并进 `scripts/experiment-apparatus/replay.sh` 复跑表的新列），一行一个比对对象：实验号、登记文件与表内序号、留存产物名、复跑行的驱动函数名或 `none:<理由>`、单元表里的单元名（单测里 `formal_run_units` 的第一项）。
- 单元表那一侧：让单测数组旁边再导出一份文本清单（例如装置 `--list-formal-run-units` 打单元名与产物名，或在数组上方写一块 `// formal-run-unit: <单元名> <留存产物名>` 的结构化注释），格读这份清单而不解析元组。这是 `crates/` 改动，不归 tooling-writer 写范围，要交 implementation-writer。
- 格的三项判据：登记表每行的留存产物名在 `replay.sh` 复跑表某行第 4 列里出现，或该行写了 `none:<理由>` 且在滞后表里；登记表每行的单元名在单元清单里出现；两边都没有没登记的单元或复跑行（反向）。登记行数为 0 退 77。
- 滞后表形态照 `multipath-registry-lag.tsv`：现存 E165 的 4 行（r5 比对对象表第 4–7 行，产物名 `e165-replay-bounded-enumeration-2026-10-02-r2-segment1b-{legal-family-2026-10-02,adversarial-family-2026-10-02,odd-marker-legal-family,odd-marker-adversarial-family}.out`）只缩不涨。
- 弄坏开关与红绿样本在上面三项落定后才写得出，现在写等于猜。

## 对同表第 4 行（某一档判了某条判据、这一档的正式跑里没有那条判据的对照读数）

这一格就算立了也不罩它：第 4 行的对象是「判据 → 对照读数」，单测里已有 `controlled == registered` 的逐单元断言和 `controlled_in_some_unit` 罩全表的断言（E165 内部），不属于「登记 ↔ 单元表 ↔ replay.sh」三处对账；不扩成第二格。

## 没做什么

没改 `.claude/gate.d/doc-experiments.sh`（已暂存的改动原样，`git status --short` 仍为 `M `）、没加样本、没加弄坏开关、没建滞后表、没在真仓单跑新格（格不存在）；没读 m5-dev 工作树；没跑任何门禁、编译与重型测试。

推翻本结论的现象：另有一处我没 grep 到的、跨实验统一写法的「比对对象」登记（例如放在 `research/results/` 或 `.claude/kb/experiments/` 下的机读表）；`grep -rn '比对对象' .` 在仓里其余位置我只查了 `research/prompts/`、`.claude/rules`、`.claude/gate.d`、`.claude/singlefs-ai-sop/rules`、`crates`、`scripts`，kb 与 records 里命中的是对登记的引用，没逐份核。
