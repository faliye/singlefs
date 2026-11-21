# replay-prefix-backfill-code-r3：判决（2026-11-19）

正文 `research/prompts/_replay-prefix-backfill-code-r3-body.md`，开工快照 `research/prompts/replay-prefix-backfill-code-r3-snapshot/`。这一题的第三轮，也是最后一轮；第一轮判决 `research/prompts/replay-prefix-backfill-code-r1-main-verification.md`，第二轮判决 `research/prompts/replay-prefix-backfill-code-r2-main-verification.md`，第二轮第三节第 1 到 5 条的改法经 `scripts/gate/apply-writer-patch.py` 合进主工作区、`--read-outcome` 退 0。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 攻方 | `research/prompts/replay-prefix-backfill-code-r3-opus-output.md` | 打中 3 条（README 别的数对不上表、`GPU_GC_PROOFS.md:203` 没拿到卡的两种情形、`:83`「三条 witness」），推的 1 条（G10 照图例也该标）；19 成立、复跑命令全名与范围数对、新加 13 行与第二轮没看过的 45 行没打中；检查单「带 --item 的命令与项」推不出 |
| 正推 | `research/prompts/replay-prefix-backfill-code-r3-sonnet-output.md` | 第二轮第三节第 1、3、4 条做到，第 5 条宽读做到，第 2 条两处出入（`GPU_DEVICE_LOST` 在两个真卡测试包里零命中、`:198`「没有可交的复跑命令」）；13 行落点与第六段相同；另报 README 同类遗留四处、G10 与 G11 同类、`verus-pins.tsv` 第 40–46 行带日期的注释 |

主 agent 的核查（2026-11-19 现读）：

- 开工快照 `sha256sum -c` 两份清单（`crates-sha256.txt` 七份、`governance-sha256.txt` 一份）都 OK。
- 两条腿交回时各自跑了 `scripts/agents/cite-check.py`：攻方「核了 35 处引文、1 处裸行号，对上 35 处」，正推「核了 77 处引文、7 处裸行号，对上 77 处」。判决引到的原文由主 agent 在主工作区逐处现读：`crates/singlefs-replay-prefix/src/README.md:119`、`:126`、`:138`、`:148`、`:165`，`src/GPU_GC_PROOFS.md:28`、`:41`、`:83`、`:198`、`:203`，`src/gpu_gc_proofs.rs` 里 G10 那一段（证明体只有 `reveal_with_fuel(record_slice_sum, 3)`），`.claude/gate.d/crash-point-replay.sh:12`（`# gate-cell: product-gpu-real-card-tests` 那一行）。
- 攻方要的那一条主 agent 交了：`bash .claude/gate.d/formal-verification.sh --list-items .`，结局行「TASK_SERVER_RESULT … type=succeeded exit=0」；13 个新 `must-red:` 名按词逐个 `grep -cP` 都是 1。检查单「带 --item 的命令与项」因此成立。
- 正推要的模型复量与说明文件六条命令实交，没交：落点在合入前的快照上已由 `formal-verification.sh` 点 21 项判过（「每行改坏之后 Verus 都报错，证明错落点与第六段 landing= 相同」），条数门禁只报不判；复跑命令的全名两条腿都在源码里逐个核过，真卡那两组任务不拿卡、交不了。

## 二、判决

1. **README 别处的数与写法（攻方 1-b、正推另发现）**：打中。`src/README.md:138`「必红世界表另登记 7 个变异」与表里 unlink 那一组的行数对不上；`:148`「11 个必红变异」同样；`:165` 覆盖写那一段列举漏了第二轮新加的两行；「义务」一词在 `:109` 是成项数、在 `:119`、`:148` 是 verified 条数，同一份文件两种单位；`:119`「同日继续补齐」、`:126`「本轮新增」是讲经过的写法。第二轮改法第 1 条只改了 `:109`、`:111`，这几处没碰到。
2. **`GPU_GC_PROOFS.md:203` 没拿到卡的情形写窄了（攻方 2-c）**：打中。那一句只写了打 `NOT_RUN_NO_GPU_ADAPTER` 一种；判定库已有两边一致的结论时测试打 `PAIRING_CONCLUSION_REUSED`、不开卡照样通过，只录那一段时打 `RECORDED_ONLY`（攻方读共用模块 `common_pairing_input_through_the_library/mod.rs` 推的，主 agent 没跑）。正推另指 `GPU_DEVICE_LOST` 在两个真卡测试包里零命中，主 agent `grep -rln GPU_DEVICE_LOST crates/singlefs-background-gpu-tests crates/singlefs-background-gpu` 零命中坐实；说明文件里讲它的那几句与这两个包对不上。
3. **`GPU_GC_PROOFS.md:83`「三条 `gpu-gc-*witness*`」（攻方 3-d）**：打中。`grep -c 'gpu-gc-.*witness' crates/singlefs-replay-prefix/verus-must-red-worlds.tsv` 是 5；第二轮新加的两行改的是见证里的值，不是注入不可能断言。
4. **`GPU_GC_PROOFS.md:198`「接口定下之前这两组没有可交的复跑命令」（正推）**：打中，写宽了。门禁 crash-point-replay 的 `product-gpu-real-card-tests` 格（`.claude/gate.d/crash-point-replay.sh:12`）起的就是这两个目标；那一句要写成「任务不拿卡，由那一格交 GPU 服务代跑」。
5. **G10 没标「定义自检」（攻方推的、主 agent 现查坐实）**：打中。`GPU_GC_PROOFS.md:28` 图例「`ensures` 是同一文件里 spec 函数定义的展开」；G10 的证明体只有 `reveal_with_fuel(record_slice_sum, 3)`，ensures 是 `record_slice_sum` 展开两层；`:41` 只标「模型，只证重数 2」。
6. **`.claude/gate.d/verus-pins.tsv` 第 40–46 行带日期的注释（正推）**：成立，不在第二轮判决第三节字面里；与第二轮判决第二节第 6 条那一笔欠账（comment-history 格不读 `.tsv`）同一类，并进那一笔。攻方另指 `:49` 又写了 214、与 `:48`「见 crate 行」的做法相反（推的隐患），并进同一条改法。
7. **没打中的**：19 这个数；复跑命令的全名与范围数（4、5、82、2，12 个真卡用例名）；新加 13 行必红改坏的都是名字说的那条、落点对；第二轮没看过的 45 行必红（攻方全看）；7 行条数与实测逐行相同；第二轮第三节第 3、4 条。两条腿都没打中，记「没打中」。正推报 505、506 两行条数推不出（求解器定），条数门禁只报不判，不改。

## 三、改法

都是说明文字与登记表注释，不动证明、必红行与产品代码。第三轮之后不再开轮，这几条修补**零轮**；不进产品代码，不做 A/B。

1. `src/README.md`：`:138`、`:148` 的必红数照表现数改，`:165` 补第二轮新加的两行；「义务」一律用成项数（照门禁 formal-verification 文件头 verus-proof 格的成项规则），`:119`、`:148` 照改；`:119`、`:126` 讲经过的写法改成只写现状。
2. `src/GPU_GC_PROOFS.md`：`:203` 补 `PAIRING_CONCLUSION_REUSED`、`RECORDED_ONLY` 两种情形；讲 `GPU_DEVICE_LOST` 的那几句照两个真卡测试包今天的样子改或删；`:83` 的条数改成照表现数；`:198` 改成由门禁那一格交 GPU 服务代跑；`:41` G10 补「定义自检」。
3. `.claude/gate.d/verus-pins.tsv` 第 40–46 行与 `:49` 的注释：去掉日期，义务数一律指 crate 行。主 agent 用 `scripts/kb/replace-once.py` 改。
4. 欠账：第二轮判决第三节第 6 条那一笔（comment-history 格不读 `.tsv`），连同这一类的存量（第 40–46 行）一起登 `.claude/kb/checks-owed.md`（三语），交下一批写回。

## 交用户的

无。改法都是说明与登记的对齐，条款现成。

## 这一轮被判过的文件

- crates/singlefs-replay-prefix/src/README.md：判了第二轮改法第 1 条（`:109`、`:111` 改成 19）与别处的义务数、必红数，打中 `:119`、`:126`、`:138`、`:148`、`:165`。
- crates/singlefs-replay-prefix/src/GPU_GC_PROOFS.md：判了第二轮改法第 2、5 条（卡的说法、复跑命令、`NOT_RUN_NO_GPU_ADAPTER` 的现状句、G11 的标），打中 `:83`、`:198`、`:203`、G10 少标、`GPU_DEVICE_LOST` 那几句与两个真卡测试包对不上。
- crates/singlefs-replay-prefix/verus-must-red-worlds.tsv：判了新加 13 行与第二轮没看过的 45 行、7 行条数、第 365 行与两段注释行，没打中。
- crates/singlefs-replay-prefix/src/file_range_proofs.rs：判了两个 `None` 支的新必红罩住的 ensures（`:171`、`:221`），没打中。
- crates/singlefs-replay-prefix/src/gpu_gc_scoring_proofs.rs：判了新必红罩住的候选窗口与排序二分那几条 ensures，没打中。
- crates/singlefs-replay-prefix/src/gpu_gc_proofs.rs：判了两条见证值的新必红与 G10、G11 的证明体，打中 G10 该标定义自检（落在说明文件）。
- crates/singlefs-replay-prefix/src/partial_write_proofs.rs：判了覆盖写两条新必红罩住的 ensures，没打中。

## 回看决策

不涉及决策：改法只对齐说明文件与登记表注释，不动证明、必红行、产品代码与任何决策分项。
