# tooling-broad-mutation-prune-r1 攻方腿的模型与副本证据（2026-10-01）

## 模型（ws/、v2/、scenarios.tsv、run.py）

`ws/` 是第一版树：`fmtc`（只放常量，照 singlefs-format）、`core`（被改的 crate）、`harness`（集成测试）、`tier`（经 `#[path]` 指回 harness 的 common，照 checker 档）。
`scenarios.tsv` 每行一个场景：第二版只换 `changed_file` 一个文件（换成 `v2/` 下那一份），变异是对 `mutated_file` 的一处字面替换（带 cargo-mutants 的标记注释）。
`run.py` 对每个场景：两版各跑没改过的全部测试、施加变异后的全部测试；第一版与第二版各逐条测试单独起进程量覆盖（`-C instrument-coverage`）；
照正文第三节 P2「抓到的」那一条算关联集（读法 A 用第一版的覆盖，读法 B 用第二版的覆盖），加 P3「被改文件引用到的常量的定义文件」（scenarios.tsv 最后一列，按读被改文件手填）。

复跑（经内存包装，llvm 工具取 rust 1.98.0 的 llvm-tools 组件，下载到草稿目录，不动 ~/.rustup）：

```
S=/tmp/claude-1000/<草稿目录>; mkdir -p $S/tools && cd $S/tools
curl -sSfO https://static.rust-lang.org/dist/2026-08-20/llvm-tools-1.98.0-x86_64-unknown-linux-gnu.tar.xz && tar -xJf llvm-tools-1.98.0-x86_64-unknown-linux-gnu.tar.xz
L=$S/tools/llvm-tools-1.98.0-x86_64-unknown-linux-gnu/llvm-tools-preview/lib/rustlib/x86_64-unknown-linux-gnu/bin
bash research/scripts/run-with-memory-cap.sh 1G bash research/scripts/capped.sh 12 nice -n 19 python3 research/prompts/tooling-broad-mutation-prune-r1-opus-model/run.py $S/model-work $L
```

产出 `run-output.txt`（这一次的原样输出）。

## 真 crates 副本上的证据（crates-copy-evidence/）

副本 = `git archive HEAD crates Cargo.toml Cargo.lock litmus .cargo`，218 个文件逐个 sha256 与库目录这一版的 manifest.tsv 相同。
`apply.py` 照驱动 `replace_span` 的写法把清单（库目录这一版的 list.json）里的一条变异写进副本，`apply.py restore` 拷回原件。
`outside_fn.py <副本> <库目录版本目录>` 列出不在任何函数里的运算符变异（常量初始化式里的），配这一版结果表的结局 → `crates-copy-evidence/const-initializer-mutations.txt`。
其余文件头一行写了是哪条命令的输出。
