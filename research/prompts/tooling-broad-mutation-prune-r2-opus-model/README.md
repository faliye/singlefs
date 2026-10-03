# tooling-broad-mutation-prune-r2 攻方腿的模型与副本证据（2026-10-01）

## 模型（ws/、v2/、scenarios.tsv、run.py、run-output.txt）

`ws/` 是第一版树：`core`（被改的 crate，每个模块对应一个场景）、`harness`（集成测试，一个场景一个测试文件）。
`scenarios.tsv` 每行一个场景：第一版 = ws/（`v1_file` 不是 `-` 时换成 `v1_source`），第二版再换 `v2_file`（`v2_source` 写 `ws` 就换回 ws/ 里那一份）；
变异 = 在 `mutated_fn` 那个 fn 里把 `old` 换成 `new`（带 cargo-mutants 的标记注释）。`mutated_fn` 写成「impl 头里的一段::函数名」或只写函数名。
`run.py` 照正文第三节判「可复用」，读法甲、乙内、乙外照字面与照意图两种读，读法丙、丁是攻方提的改法（被攻过零轮），判法写在 `run.py` 文件头。
P3 与 B8 按名字过近似（起点文件里出现的任何标识符，只要工作区里有同名的 const / static / const fn / macro_rules! / 带 derive 的类型，就并进定义它的文件），比任何只凭语法解析名字的工具收得多。

复跑（经内存包装，llvm 工具取 rust 1.98.0 的 llvm-tools 组件，下载到草稿目录，不动 ~/.rustup）：

```
S=/tmp/claude-1000/<草稿目录>; mkdir -p $S/tools && cd $S/tools
curl -sSfO https://static.rust-lang.org/dist/2026-08-20/llvm-tools-1.98.0-x86_64-unknown-linux-gnu.tar.xz && tar -xJf llvm-tools-1.98.0-x86_64-unknown-linux-gnu.tar.xz
L=$S/tools/llvm-tools-1.98.0-x86_64-unknown-linux-gnu/llvm-tools-preview/lib/rustlib/x86_64-unknown-linux-gnu/bin
bash research/scripts/run-with-memory-cap.sh 6G bash research/scripts/capped.sh 12 nice -n 19 python3 -B research/prompts/tooling-broad-mutation-prune-r2-opus-model/run.py $S/model-work $L
```

`run-output.txt` 是这一次的原样输出。

## 库目录产物与真 crates 副本上的证据（evidence/）

- `evidence/v2b-*`：只读库目录这一版的 nextest 日志，算每条测试 PASS 的耗时与基线的比（V2②）；`wall_vs_cpu.py` 在副本上把同一条 harness 测试挤到两个核上并发跑，比墙钟与 CPU 时间。
- `evidence/crates-*`：副本 = `git archive HEAD crates Cargo.toml Cargo.lock litmus .cargo`，218 个文件逐个 sha256 与库目录这一版的 manifest.tsv 相同；
  施加变异用第一轮攻方模型的 `apply.py`。`mutant_path.py`（core 库测试抓到的）与 `mutant_path_harness.py`（harness 测试抓到的）比 T 在没改过的树上与在变异体上执行到的文件与函数（V2③）。
- `evidence/v3-*`：`parser_check.py` 拿 run.py 的语法切函数对清单 5091 条找包住它的 fn；`fn_changes.py` 与 `outside_diff.py` 数最近 15 个提交按函数、按文件各要重判多少条；`instr_cost.py` 量插桩倍数与逐条覆盖的开销（V3）。
- 各输出文件头一行或文件名写了是哪个脚本的输出；脚本的用法写在各自文件头。
