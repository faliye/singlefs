# m3-close-i718-oldslot-r3 辩方腿（Sonnet）模型目录

做法与前两轮四条腿不同：不改池级 checker、不往 `walk.rs` 里装开关。池级 checker 原样判每一张镜像，取它对 I-7.18 的判定；
各候选（丙、乙字面、乙加闸、丁乙、丁乙同实例、甲、丁甲）从镜像上自己读出的字段（最新根的 t、F、p，环上读得出的根，下一槽，journal 里点名 N+1 的记录）
在原型里后置分类。分类用的现算式（max(F, min(环上读得出的根, 钉值))）先拿池级 checker 的原判核对：四个 S、共 15 千多行镜像上 `calib=BAD` 0 行。

- `tests/i718_r3_independent.rs`：后置分类原型。格 A（题面形，连事后读坏最旧根 BR / Y）、B（环洞落在下一槽）、F（挂载时钉住的槽轮到下一槽，含诚实 / t 写成 F / p 写成 255 三份写者）、
  G（下一次发布是新实例的写行）；第三轮新增 Z2 四格：两个洞、洞加抬 F、多个短命实例交替、p 的全部取值。
- `tests/i718_r3_given_states.rs`：C560 那几个场景交给池级 checker 的镜像，在 harness 档里收下来、用池级 checker 原样判。
- `tools/`：`rerun.sh`（复跑）、`job-list.sh`（各 S 的作业）、`run-jobs.sh`、`peer-batch.sh`（并行起作业；第二台上整条放在外层内存包装里跑）、
  `summary.py`、`tally.py`、`invariance.py`、`rows-diff-check.py`、`slots-from-env.py`。
- `out/final/`：`all-S*.txt` 是 ROW / SCAN 原样行（只有标准输出，没有内存包装的提示行），`summary-S*.txt` 是汇总，`given-states.txt` 是给定状态，
  `invariance.txt` 是各 S 的分类键比对。S = 4、5、8 是全量（S = 8 不含「p 全部取值」那一格，超时）；S = 7 只有格 A、G；S = 16 是抽样（格 A 只到 N = 95、不扫描；格 B、F 各抽 4 片，其中 B 的 36/48 片与 F 的 4 片被限时截断）。
- 复跑：`bash scripts/hosts/run-with-memory-cap.sh 24G bash <模型目录>/tools/rerun.sh --repo <仓根> --frozen <仓根或冻结树> --work <新的草稿目录> --model <模型目录>`；
  末尾每个 S 一行 `name=summary-S<S> same_as=…` 或 `DIFFERS`，再一行 `name=given-states …`。`RERUN_QUICK=1` 只跑 S = 4 的格 A、G 与给定状态。
