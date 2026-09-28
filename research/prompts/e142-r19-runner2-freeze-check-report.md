# E142 第十九次跑续做：执行员报告（停在「核冻结值」，差异里有代码行，照派发提示停下交回）

日期 2026-09-28。这一段只做了派发提示「续做的第一步：核冻结值」，没跑任何产物、没写登记修订、没改 `replay.sh`、没写实验页。

## 结论

- 冻结那一版找到了，在 git 里：上一段证据 `research/prompts/e142-r19-runner-evidence/model-vs-head.diff` 第 2 行是 `index dd3a1887..47002dc0 100644`，`git cat-file -p 47002dc0 | sha256sum` 得 `41ef20b50ffc32adece04054d6ffdffce1d0f6ca8d127ad8680953b37e4b9dd1`（= R19E-2 冻结值），9319 行。
- 与今天的 `research/e7-index-bench/src/bin/e142_new_pool_file_creation_dry_run.rs`（`5f882741ae96687b9abce9f30830ec32da1c9f411091d4fe3cac4248a5fb5a43`，9319 行，= `bc57af7a` 里的那一版）逐行比：**154 行不同**，没有增删行。其中 60 行是注释或 `#[allow(…, reason = …)]`，**94 行是代码**。
- 这 154 行全是 `bc57af7a` 那一批按内容改名的机械替换。证据：把今天的文件按 8 对替换换回旧名（脚本 `/tmp/claude-1000/e142-r19-runner2/freeze/reverse_rename.py`：`新池新建文件`→`第一个事务`、`NEW_POOL_FILE_CREATION`→`FIRST_TRANSACTION`、`NewPoolFileCreation`→`FirstTransaction`、`new_pool_file_creation`→`first_transaction`，外加 `new_pool_file_creation_allocation_nodes`→`first_txn_allocation_nodes`、`eight_byte_flip_probes_recover_as_the_e142_product_records`→`probes_behave_as_milestone_step_six_expects`、`覆盖写、释放、回退与复用`→`第二个事务`、`新池新建文件的干跑`→`第一个事务的干跑`），sha256 恰好是 `41ef20b5…`。变异表 `research/mutations/e142_new_pool_file_creation_dry_run.tsv`（今天 `0d91aafd…`，180 行）用同一个脚本换回，sha256 恰好是冻结值 `15a4fe6da352e15236f267b38e6735295d7d0c7659900de6dedd80867d2e9c77`。
- 94 行代码是常量、函数、结构体、测试函数的改名（`FIRST_TRANSACTION_TXG`→`NEW_POOL_FILE_CREATION_TXG`、`record_first_transaction_stream`→`record_new_pool_file_creation_stream`、`first_transaction_write_list_matches`→`new_pool_file_creation_write_list_matches`、测试 `probes_behave_as_milestone_step_six_expects`→`eight_byte_flip_probes_recover_as_the_e142_product_records` 等），加 5 行 expect / assert 消息。**其中 3 行改了产物里的字段名**（冻结版行号，今天同号）：
  - 第 6294 行 `name=warm_up … first_transaction_txg=` → `new_pool_file_creation_txg=`
  - 第 6758 行 `name=g3_shape … first_txn_allocation_nodes=` → `new_pool_file_creation_allocation_nodes=`
  - 第 6990 行 `name=mapping_key_collision … mapping_entries_first_transaction=` → `mapping_entries_new_pool_file_creation=`
  （第 6537 行 `name=change_count_run_pair` 只换了格式串里内联的常量名，打出的文本不变。）
- 派发提示原句：「差异里有一行代码，停下交回，不跑产物」。照做，停在这里。

什么现象会推翻「只差改名」：`python3 /tmp/claude-1000/e142-r19-runner2/freeze/reverse_rename.py <今天的模型> | sha256sum` 不是 `41ef20b5…`，或同一脚本作用在变异表上不是 `15a4fe6d…`。

## 续做前要主 agent 定的（都碰到登记第六、十一节或臂 N18 的取法，执行员不能自己修订）

1. **臂 N18 用哪份源码、Q142.56 怎么比**：仓里第十八次 `research/results/e142-new-pool-file-creation-dry-run-2026-09-26-r18-main-2.out` 已被 `bc57af7a` 就地改成新字段名：`grep -c 'first_transaction_txg=\|first_txn_allocation_nodes=\|mapping_entries_first_transaction='` 得 0；新字段名的行有 7 行（第 33 行 `name=warm_up`，第 621–625 行 `name=g3_shape` 五行，第 664 行 `name=mapping_key_collision`），都在第一行 `name=done`（第 715 行）之前、都是模型那一侧。照派发提示「另编冻结前的 `0676ed9b…` 模型重跑臂 N18」，那份源码打的是旧字段名 ⇒ Q142.56 至少这 7 行不同 ⇒ 作废 V19a（推的，没跑）。出路两条，由主 agent 定：N18 源码取 `0676ed9b` 加同一批改名（与冻结版同一种替换），或 Q142.56 / D_model 的归一化另加字段名换回（后者改的是 R37 的比法）。
2. **D_model 归因**：N18 若用旧名，D_model（N18 → N19C）会多出这三种行组，归不到 ①–⑪ ⇒ F19e。
3. **臂 N18 产物也被就地改写**：`research/results/e142-new-pool-file-creation-dry-run-2026-09-27-r19-arm-n18.out` 今天 sha256 `f5eaeeddddd711d2d58f5fabfbe8639fe02f1f2a64a384d05a72185256c2b804`，上一段报告记的是 `2bbacde1…`；它与 D_crates 文件 `e142-r19-diff-r18-to-arm-n18-2026-09-27.txt`（今天 `fe0b2161…`）都是上一段在快照 A 那一版 `crates/` 上量的，续做本来就要重取（上一段报告「续做要备的」）。
4. **S19-clause 会在步 ③(1) 停**：这一次照 R19C-3(b) 的七个取法重抄（`quote-kb.py`，「✓ 7 段整抄进 …，回读逐字节一致」），189 行，sha256 `4d98ee794050933c8686abd47131212790878d7f35add64bbef400aae3590ddb`，不等 R19E-1 的 `e1bda65b…`。换回旧名之后与 R19E-1 原件（`/tmp/claude-1000/e142-r19-runner/step0/baseline-clauses-step0.md`，仍是 `e1bda65b…`）逐字符比，剩下的差只有三类：`layout/01` 第八节里的用例路径（抄件第 34、40、44–48、52 行，例：`crates/singlefs-harness/tests/first_transaction_step_one_mkfs.rs` → `…/mkfs_bytes_judged_by_the_checker.rs`，`crates/singlefs-checker/tests/second_transaction_step_zero_layer0.rs` → `crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs`）；第 36 行产物文件名 `e142-first-txn-dry-run-…` → `e142-new-pool-file-creation-dry-run-…`；D16 已定项 7（抄件第 179 行）删了一处钟点。段序列、写数、种类串、闭式一个字没变。换不换基线（与 R19E-1 同一种收严）由主 agent 认。另：仓里上一段证据的基线件 `research/prompts/e142-r19-runner-evidence/step0/baseline-clauses-step0.md` 已被改名扫改写，今天 `7350ca81…`，不再是 `e1bda65b…`。
5. **登记里点名的路径改了名**（路径不在冻结范围内，照实列）：step_five 用例今天在 `crates/singlefs-harness/tests/new_pool_file_creation_publish.rs`（按函数名 `grep -rl 'fn recorded_paths_match_the_registered_segment_sequences' crates/` 找到，内容没读）；`crash_segments_per_device_and_torn_in_place_overwrites.rs` 同名还在；导出 bin `crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs`；`driver_e142`（`research/scripts/replay.sh` 第 451 行起）今天跑 `cargo run -q -p singlefs-checker-tier --bin e142_new_pool_file_creation_write_dump` 与 `./target/release/e142-new-pool-file-creation-dry-run`；`replay.sh` 第 171 行 `E142|@driver_e142||e142-new-pool-file-creation-dry-run-2026-09-26-r18-main-2.out|exact`；实验页 `.claude/kb/experiments/142-新池新建文件的干跑.md`。
6. **独立比对 bin**：今天 `311ce77c9c73148059f4d8a3425ab8902413c34e7ca45ddc04cc582de92fae51`，与 `1ca9b2f9` 里的 `f4f26386…` 只差第 3 行文档注释里的模型文件名（路径）；它的变异表 `8978e80a…` 没变。

## 核对用的命令（原样输出摘在「结论」一节）

```
git cat-file -p 47002dc0 > freeze/model-frozen-41ef20b5.rs              # sha256 41ef20b5…
diff freeze/model-frozen-41ef20b5.rs <今天的模型> > freeze/model.diff   # 退 1；560 行；只有 c 型块
python3 分类脚本：changed 154 comment_or_allow_reason 60 code 94 code_with_name= 4
python3 freeze/reverse_rename.py <今天的模型> | sha256sum                # 41ef20b5…（= 冻结值）
python3 freeze/reverse_rename.py <今天的变异表> | sha256sum              # 15a4fe6d…（= 冻结值）
git show 1ca9b2f9:research/e7-index-bench/src/bin/e142_region_diff_independent.rs | diff - <今天的独立 bin>   # 只差第 3 行
quote-kb.py freeze/requote-now.md <七个取法>                             # 4d98ee79…，189 行
```

草稿都在 `/tmp/claude-1000/e142-r19-runner2/freeze/`：`model.diff`、`model-frozen-41ef20b5.rs`、`model-today-reversed.rs`、`table-today-reversed.tsv`、`independent.diff`、`requote-now.md`、`requote-now-reversed.md`、`requote-now-vs-e1bda65b.diff`、`reverse_rename.py`。另有一份 `table-reconstructed.tsv`（拿上一段证据里的变异行草稿拼的，sha256 `c188d49d…`，不等冻结值——那份草稿行不是最终版，这份拼件不作证据）。都是单个文件，不是编译目录、工作树或仓副本。

## 判决行点名（第 4c 步）

这一段没有产物，没有判决行可列。

## 岔路表（问题单 `research/prompts/e142-r19-questions.md` 三行）

| # | 状态 | 还差什么 | 剩下的量能不能让它翻面 |
|---|---|---|---|
| 1 段数、写数、两态与三态闭式 | 还差 | 与上一段相同：步 ③ 全部（主产物、Q142.46–Q142.50、Q142.57、Q142.60、Q142.61、Q142.48 新格、PC-a、PC-d、判别力自证 ①–④、R41 行数）。这一段一样都没量 | 能 |
| 2 六个判决字段 | 还差 | Q142.51 的 N19C 那一列要主产物；N18 那一列要按上面第 1 条定下的源码重跑（上一段那份产物是快照 A 那一版 `crates/` 上的，且已被就地改名） | 能 |
| 3 与第十八次逐行比、各因哪一批 | 还差 | D_crates 要重取快照 A、重跑臂 N18 后重做；D_model、D_total、Q142.52–Q142.55、PC-b、PC-c 要主产物；改名带来的三种行组怎么归因要先定（上面第 1、2 条） | 能 |
| 第二段（R19C-8 层 0 主臂） | 未跑 | 重型，等主 agent 问用户 | — |

## 登记修订

没有。派发提示只在「差异只有注释、路径与改名」时让写修订；这一次差异里有代码行（含 3 个产物字段名），没写。

## 没做什么

- 步 ③④ 全部没做：没取快照、没编模型、没重跑臂 N18、没跑主产物、没跑独立 bin、没做 diff 与比对、没读 `crates/` 用例的钉值；`replay.sh` 第 171 行没改；实验页、实验索引、`experiments-history.md` 没写；`layout/01` 第八节 mkfs 行与第二条流那一行该怎么写（派发提示要的）没有新产物，给不出。
- 门禁阶段一道没跑：这一段仓里一个文件都没写（只写了 `/tmp/claude-1000/e142-r19-runner2/` 下），52 号的红是今天已有的状态，没重跑。
- 没编译、没跑 cargo，所以没看负载。
- 没判这个实验的结论能不能推翻或确立决策；没跑门禁全量；没提交。
