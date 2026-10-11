# m3-batch37-43-code-r1 正推腿（Sonnet）的模型目录

仓路径从环境变量取：`SINGLEFS_REPO` = 冻结副本根（`~/.local/share/singlefs/work/m3-batch37-43-code-r1/tree`），`SINGLEFS_MAIN_REPO` = 主仓根（只取 `scripts/` 里的内存包装与线程上限脚本）。可选 `SONNET_TARGET` = 编译目录（默认 `<输出目录>/work/target`）。

| 文件 | 做什么 |
|---|---|
| `rerun.sh <输出目录> [阶段…]` | 分段跑（每段前台 ≤ 4 分钟，同一时刻只起一条 cargo，内存包装 6G / 8G，线程 15）。阶段：`static` sha256 23 份 + 变异表全表锚点 + 各前缀行数；`f2diff` 差分模型；`baseline` 编译 + core lib + 10 个 harness 目标；`mut <从> <到>` 变异行区间（行序号 0–47，追加写 `mutations.txt`）；`adhoc` 窄版 A1–A3。第一次调用输出目录必须为空 |
| `compare.sh <目录甲> <目录乙>` | 用法写在文件头第一行。两边都有的 `static/f2diff/baseline/mutations/adhoc.txt` 排序、去耗时与 `ROWS` 行后逐行比 |
| `f2_differential.py` | F2_NEWEST 的 CPU 与 GPU 两份实现各自逐支转写（读码转写，不调原代码），随机 400000 个小状态比，并对 GPU 转写打 4 个弄坏开关证会红。不依赖仓 |
| `mutrun.py` / `static_anchor.py` / `adhoc.py` | 变异行逐条现跑（checker 档行 `SKIPPED`）/ 变异表锚点静态判 / 窄版变异（用例在 `adhoc-cases.tsv`） |
| `rows.txt` | 这一轮要跑的变异行名前缀（48 行：第 37、38、39、41、42、43 批新行 37 + 改过锚点或替换文的 11 行） |
| `test-args.txt` | 基线编译与跑的 10 个 harness 目标 |
| `crates-sha256.txt` | 冻结副本 23 份文件的 sha256（抄自 `research/prompts/m3-batch37-43-code-r1-snapshot/`） |
| `expected/` | 交回前在空目录从头跑出的输出（`rerun.sh` 分段跑，`compare.sh` 比出全同） |

分段跑法（原样，交回前跑通的那一串）：

```
export SINGLEFS_REPO=… SINGLEFS_MAIN_REPO=…
bash rerun.sh OUT static f2diff
bash rerun.sh OUT baseline                 # 约 1.5 分钟（含从头编译）
bash rerun.sh OUT mut 0 11 mut 12 48       # 约 3 分钟
bash rerun.sh OUT mut 11 12                # 随机历史慢档那一行，单独跑
bash rerun.sh OUT adhoc                    # 约 1 分钟
bash compare.sh OUT expected
```

没经这里跑的：checker 档与 GPU（对拍文件、`crash_judge_*.wgsl` 的变异行、恢复内核钉值 328970、崩溃注入第三截）——子 agent 不跑，见报告「要主 agent 跑的命令」。

`prove-red.sh` 的跑法与结果（harness 档 39 行，分三段，每段前台 < 4 分钟；`names.txt` 是 39 个变异名，一行一个）：

```
export CARGO_TARGET_DIR=<编译目录> PROVE_RED_LOG_DIRECTORY=<日志目录>
cd <仓副本>   # 副本里改过的文件要 touch 一下再跑：cp -p 回原样的源文件 mtime 比编译产物旧，cargo 不重编（我踩过一次）
mapfile -t N < <(sed -n '2,12p' names.txt)
bash <主仓>/scripts/prove-red.sh --copy "$PWD" --memory 6G singlefs-harness "${N[@]}"
```

三段的输出在 `prove-red-run/chunk-*.txt`：第 2–12 行 11 条、第 1 行与 13–26 行 15 条、第 27–39 行 13 条，合计 39 条，「跑的都抓到了」。
