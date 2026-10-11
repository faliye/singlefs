# tooling-broad-mutation-prune-r3 攻方模型

- `ws/`：小模型工作区（`mcore` 被变异，`mharness` 放测试与一个 bin）。X1–X7 是 C2 的场景，Y1–Y9 是 C1 的场景，各场景的注释在文件头。
- `run.py`：跑 C2（插桩覆盖 → 按覆盖挑测试 → 挑出来的与全部测试各跑一遍比结局）与 C1（第二版只动一处 → 三种读法判「改没改」→ 两版各施加同一条变异真跑全部测试）。原样输出 `run-output.txt`。
- `evidence/`：
  - `c4_cpu_time.py` / `c4-cpu-time.txt`：同一条 harness 测试的墙钟与 CPU 时间，按线程数、超订、SMT 兄弟核被占分格（真 crates 副本上的测试二进制）。
  - `c1_literals_in_crates.py` / `c1-literals-in-crates.txt`：core、checker 的函数体里，正则去注释与空白折叠会看错的串字面量有几处。
  - `c1_commit_classes.py` / `c1-commit-classes.txt`：最近动了导出范围的提交里，只改测试或数据、不改被变异源码的有几个。
  - `crates_c2_selection.py` / `crates-*.txt`：真 crates 副本第一段每条测试插桩，按覆盖给清单 5 091 条挑测试，与库目录这一版的结局比。

复跑（仓根下；llvm-tools 1.98.0 解到任意目录，取它的 `lib/rustlib/x86_64-unknown-linux-gnu/bin`）：

```
bash scripts/hosts/capped.sh 12 bash scripts/hosts/run-with-memory-cap.sh 8G python3 -B research/prompts/tooling-broad-mutation-prune-r3-opus-model/run.py <工作目录> <llvm bin 目录>
```
