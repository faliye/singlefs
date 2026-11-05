# m4-gpugc-unsafe-gate-r1 攻方腿（Opus，重派）报告：未完成

## 各格判定一览

| 格 | 结论 |
|---|---|
| Q1 判法：放过与误拒 | 未判 |
| Q2 登记理由 | 未判 |
| Q3 SAFETY 前提 | 未判 |

这条腿停在半途。读源码时有一次回复被安全分类器拦下，之后没有接着做。模型目录 research/prompts/m4-gpugc-unsafe-gate-r1-opus-model/ 没有建，没有交出源码样本。

## 做完的

- 认领草稿目录 /home/fy5090/.local/share/singlefs/work/m4-gpugc-unsafe-gate-r1-opus-2。目录里只有 .owner 与 .writer-lock，没有仓副本或编译目录。
- 读了共用约束、四份规则文件、正文、清单、登记表 .claude/gate.d/unsafe-allowances.tsv，以及门禁 .claude/gate.d/code-source-discipline.sh 里 unsafe-allowances 那一格（cell_unsafe_allowances 的判法）。
- 开工快照 judged-sha256.txt 列的 201 份文件逐份比过 sha256：sha256sum -c 报 201 行 OK，工作区与快照一致（量的）。

## 读判法时记下的线索（推的，没造样本、没跑）

- 判法跳过后面紧跟 assume_specification 的 allow(unsafe_code)；singlefs-core 的 checksum.rs 有两处靠这条豁免。Q1 可以查这条豁免能不能被拿来放过别的写法。
- bin 根只认 src/main.rs、src/bin/<名>.rs、src/bin/<名>/main.rs 三种；Cargo.toml 里 [[bin]] 另指路径的、build.rs、examples/ 下的文件都归库根那一份。该不该这样归，要另核。
- 判法写明逗号结尾不算续行；match 分支那一形没有核。

## 要主 agent 跑的命令

无。

## 没打中的形状

没造任何形状。

## 这条腿自己的限度 / 没做什么

- Q1、Q2、Q3 一格都没判，上面的线索也都没验。
- 建议新派一条攻方腿接着做：从 E185 余下几处 unsafe 读起，按行号一次只读一处的几行。
- 没往统一任务服务交任务；没跑判法、没跑样本。
