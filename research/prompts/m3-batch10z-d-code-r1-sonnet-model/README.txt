m3-batch10z-d-code-r1 正推腿（Sonnet）的模型目录。报告：research/prompts/m3-batch10z-d-code-r1-sonnet-output.md
所有编译与测试都在草稿目录 ~/.local/share/singlefs/work/m3-batch10z-d-code-r1-sonnet/ 的仓副本（repo，取自冻结副本 tree/）里做，仓内文件没改。

probe/  K2 / K1-1 探测（不入库装置）
  k2d_probe.rs            合成好的探测测试文件；放进副本 crates/singlefs-harness/tests/ 下，与 common_unwitnessed_publish/、common_admission_pool/ 同级。
                          = head.rs（抄新用例文件 raise_floor_takes_… 的第 22–481 行搭建，加了「读回全零」模式与合并故障）+ tail1.rs（h1_run：K2-1 / K2-3，被抛弃根 C）
                          + tail2.rs（h2_run：K2-2，R 来自 common_unwitnessed_publish 的盘，逐组读回全零 × 账故障 / 根槽故障 / 账与记录读回全零）+ tail3.rs（K1-1：管理员回退、公开上限函数）。
  build_probe.py          从新用例文件生成 head.rs。
  sumh2.py                把 H2 扫描日志里的 RESULT 行压成一行一例。
  复跑：CARGO_TARGET_DIR=<编译目录> bash scripts/hosts/run-with-memory-cap.sh 12G bash scripts/hosts/capped.sh 6 cargo test --offline -p singlefs-harness --test k2d_probe -- --nocapture --test-threads 6 <过滤串>
        过滤串：k23_ / h1_ring / h1_account / k11_ / h2_<ow|sq>_<none|acct|root|az|rz|bz>_g<00..13>
k3/  K3 自造变异
  mut.py                  变异表（名 -> 文件、原文、替换文；原文须恰好命中一次）与施加 / 还原：python3 mut.py <名> [restore]；路径写死在副本 repo 下。
                          revert-h2-or-else 的替换文是空串，restore 的断言认不出（count("") 恒大于 1），那一次是手工从冻结副本 cp 回 mount.rs 还原的（还原后与冻结副本 cmp 相同）。
  run.sh                  施加 -> core lib -> 两份新 harness 档用例 -> （WIDE=1 时）另五个相关 harness 目标 -> 还原（trap）。out/<名>.txt 是每条变异的结果摘要。
logs/                     K2 / K1 / 基线的原始输出。probe-h1-a.log 是探针第一版（基线行的故障没关，故基线行与 H1 行相同，不引）；probe-h1-b.log 是第二版（200 步，无写满）；
                          probe-k23-a.log 是现行探针的 K2-3 各形；probe-pre.log 是撤掉 10z-d 的副本（repo-pre：对冻结副本 mount.rs / allocator.rs 反向打 ~/.local/share/singlefs/work/m3-r1/batch10z-d/patch/hunks/ 下两份 diff，15 个 hunk 全成）上同一探针的输出。
追加：probe/tail3.rs 里 print_keys_of_p_and_r、print_instance_table_after_m1_bz_and_az、diag_bz_refusal 是排查用：bz 一批会话写第 13 步拒，是我的读故障盖到了已被重新分配给新写入的 R 的账单元（logs/probe-diag.log：拒的有效根是实例 2 的 txg 10，不是 R），不是产品行为。
logs/probe-h1-c.log 是现行探针（60 步 + 写满，带险步统计）的 H1 五形；K2-1 的表引它。
