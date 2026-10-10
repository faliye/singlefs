# m3-batch56-code-r1 正推腿（Sonnet）模型目录（2026-10-10）

- `scripts/`：变异驱动与差分比对。`mut.sh`（单对变异）、`mutm.sh` / `mutm2.sh`（多对合并变异，后者给 cargo 加 200 秒限时）、`runrow.sh` / `runprefix.sh`（按 `crates/mutations.tsv` 的行号 / 名字前缀取一行跑）、`tsvrow.py` / `tsvrow2.py`（取行）、`fuzzrun.sh` + `fuzzdiff.py`（K1 差分）、`queue2.sh` / `queue3.sh`（我排过的批次）。路径里写死了我的草稿目录 `~/.local/share/singlefs/work/m3-batch56-code-r1-sonnet/`，换地方跑要改 `W=` 和 `F=`（`F` 是冻结副本）。
- `mutdefs/`：自造变异的原文 / 替换文（`ka`–`ki`、`c*`、`t*`、`wa`、`wb`、`kb2`、`ca`）与取自登记表的 `row*` 展开。
- `probes/`：不入仓的探针。`zz_probe_k2_abandon.rs`（崩溃前缀镜像 + 可写挂载 + 带旋钮 checker）、`zz_probe_k1_diff.rs`（记账条目差分，是 `corrupt_on_disk_content_is_refused_instead_of_panicking.rs` 的拷贝加末尾一条用例）、`zz_probe_kernel_size.rs`（量 GPU 内核展开量，示例程序）、`probe_k4.rs.txt`（K4 等价扩族探针，追加进 `inode_tree.rs` 的 tests 模块）。
- `logs/`：全部日志，`row<N>.log` 是 37 行里第 N 行的原样输出。
- 这些探针与变异都只在 `copy/`（冻结副本的拷贝）里用过，冻结副本没动。
- 注意：`copy/`（冻结副本的拷贝，含 `.cargo/`）与 `target/` 已在交回前删掉；重跑先 `cp -r <冻结副本>/. copy/ && cp -r <仓根>/.cargo copy/`，再把 `probes/` 里的文件放回 `copy/crates/…/tests/` 或 `examples/`。
