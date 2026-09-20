# m2-supp3-item2-code-r1 攻方腿（Opus）模型目录：复跑

全部在仓副本上跑，主工作区不改。草稿根 `D=/tmp/claude-1000/m2-supp3-item2-code-r1-opus`（脚本里写死；换目录改 `run-mutant.sh`、`run-geometry.sh` 第一行的 `D=`）。

```bash
D=/tmp/claude-1000/m2-supp3-item2-code-r1-opus; M=research/prompts/m2-supp3-item2-code-r1-opus-model
mkdir -p $D/copies $D/logs $D/model && cp $M/*.py $M/*.sh $M/mutants.tsv $D/model/
rsync -a --exclude target --exclude .git ./ $D/copies/base/          # 仓根执行；开工快照 sha256sum -c 应全 OK（D28 除外，见报告）
python3 $D/model/attack-patch.py $D/copies/base                      # 只加三个环境变量开关与一行探针，不设时语义不变
bash $D/model/run-mutant.sh base                                     # 基线三段 × 三组套件
for id in B1 B2 B2s B3 B5 B6 N2 A1 A2 A3 A4 A8 A9 A10 N1 N3 N4 N5 W1 W2 R1; do bash $D/model/run-mutant.sh $id; done
for id in B1 B2 B2s B3 B5 B6 N2; do bash $D/model/run-geometry.sh $id; done   # 然后 (cd $D && python3 model/geo-table.py B1 B2 B2s B3 B5 B6 N2)
```

长历史（容量墙区间，M2）：`ATTACK_NO_CHECKER=1 ATTACK_PROBE=1 SINGLEFS_RANDOM_HISTORY_SEEDS=96 SINGLEFS_RANDOM_HISTORY_OPERATIONS=200 SINGLEFS_RANDOM_HISTORY_WEIGHTS=broad|reuse SINGLEFS_RANDOM_HISTORY_SHRINK=none <bin> --ignored --exact random_histories_large_tier_seeds_and_length_from_the_environment --nocapture > log; python3 wall-summary.py log`（`<bin>` 取 `$D/logs/<id>/bin.txt`）。
改法：`wall-fix-patch.py <副本>`（墙）、`reason-fix-patch.py <副本>`（回退理由），施加在 W1 / R1 / base 的副本上再建再跑。

产物：`matrix.log`、`matrix2.log`（变异 × 三组套件）、`geometry-table.txt`（M5）、`B2s-windows.txt`、`wall-interval-reach.txt`、`wall-long-runs.txt`、`reason-fix-runs.txt`。都是副本上的数，不是入库装置上的数。
