# C834 第三轮正推腿的模型目录

这一份不另写原型：第二轮两腿的补丁、原型、驱动脚本直接读仓里 `research/prompts/c834-r2-opus-model/` 与 `research/prompts/c834-r2-sonnet-model/`（只读），这里只有挑格、分件、比对、点数的脚本和一轮复跑的汇总。

- `tools/rerun.sh`：模板拷来，只换 `rerun_steps`（读 `tools/steps.sh` 并调它）。用法：
  `bash scripts/hosts/run-with-memory-cap.sh 24G bash research/prompts/c834-r3-sonnet-model/tools/rerun.sh --repo <仓根> --frozen <冻结副本> --work <新的草稿目录> [--model research/prompts/c834-r3-sonnet-model]`，
  末几行看 `name=rerun_compare … same`。整条的挂钟看机器忙闲：这一轮攻方原型 168 件在负载 100 以上时跑了约 45 分钟，辩方原型 280 件在负载 40 到 65 时约 15 分钟，编译共约 13 分钟；前台命令限 4 分钟，
  所以这一轮是分步跑的同一份代码，见 `steps.sh` 头部的 `RERUN_STEPS`。
- `tools/steps.sh`：步骤本体。`prepare`（冻结副本拷两份、各装第二轮的补丁）、`build_opus`/`build_sonnet`（编译，带 `BINDGEN_EXTRA_CLANG_ARGS`，取 `scripts/crash/crash-amplification-lib.sh` 的 `CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS`）、
  `opus_nine`、`opus_batch:<0..5>`、`sonnet_batch:<0..9>`、`summarize`。
- `tools/pieces.py`：件表。攻方模型：丙、丁、丁加 y2、丁只藏记录、三次挂载两段，共 168 件；辩方模型：映射根变体与指针槽读法各在四个前缀上全量（含乙那一类中间约 16000 个状态），共 280 件；每件至多 600 个状态。
- `tools/compare.py`：这一轮汇总与第二轮入库汇总逐行比（只比这一轮跑过的格；辩方汇总的例子序号两边都去掉，那一字段在第二轮是先到先得、不确定）。
- `tools/tail-compare.py`：映射根变体全量逐状态输出里挑出第二轮跑过的头尾两段，重汇总后和第二轮 `aggregate-mapping-tail.txt` 比。
- `tools/middle.py`：映射根变体全量里第二轮没跑的中间一段（序号 20–15999）单独汇总。
- `tools/flags.py`：每个状态那一行里映射根凭据与指针槽凭据逐状态相同不相同；指针槽读法的判由凭据推出，和第二轮 `aggregate-all.txt` 里 `pointer_slots_fixed` 的同一格逐格比。
- `tools/tally.py`：放过的状态在被判那次挂载之前是不是已经红；攻方腿另数两个凭据的四种组合。
- `tools/cells.py`：把汇总压成一格一行，便于抄进报告。
- `out/`：这一轮复跑的汇总，`steps.sh` 的 `summarize` 步把新跑出的和这里逐行比（`sonnet-aggregate.txt` 去掉例子序号再比）。
这些数都是冻结副本拷贝上的数，不是入库装置上的数。
