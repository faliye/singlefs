#!/usr/bin/env bash
# 样本现场（情形 marker-unstaged）：在样本的临时副本里造暂存树（git init 再 git add -A），改动之前的逐文件 sha256 记进 .broad-sample/manifest.tsv、导出范围记进 .broad-sample/export-paths。
# 广谱变异库目录里的那一版不在这里造：阶段在样本模式下先起 .broad-sample/prepare.py（把驱动的路径给它），它载入今天的驱动、调 fingerprint_of_manifest、
# global_key_lines、version_key 造那一版，样本不自己抄驱动的指纹公式。导出范围是登记表里 checker-tier-crates-mutation-replay.sh 那一行（去掉 research/）加 .cargo。
set -euo pipefail
variant="marker-unstaged"
mkdir -p crates/sample/src litmus .cargo .claude/gate.d .broad-sample/home
printf '[workspace]\nmembers = ["crates/sample"]\nresolver = "2"\n' > Cargo.toml
printf '# 样本的锁文件\nversion = 4\n' > Cargo.lock
printf '[package]\nname = "sample"\nversion = "0.1.0"\nedition = "2021"\n' > crates/sample/Cargo.toml
printf 'pub fn add(left: u32, right: u32) -> u32 {\n    left + right\n}\n' > crates/sample/src/lib.rs
printf 'C sample\n{}\nP0(int *x) { WRITE_ONCE(*x, 1); }\nexists (x=1)\n' > litmus/sample.litmus
printf '[build]\njobs = 1\n' > .cargo/config.toml
printf 'checker-tier-crates-mutation-replay.sh\tcrates/ Cargo.toml Cargo.lock litmus/\t# 样本：广谱变异的导出范围取这一行\n' > .claude/gate.d/stage-inputs.tsv
case "$variant" in
  marker-*)
    printf 'checker-tier-broad-mutation.sh\tcrates/ Cargo.toml Cargo.lock litmus/ .cargo/\t# 样本：写全绿标记之前核这几条路径上工作区等于暂存区\n' >> .claude/gate.d/stage-inputs.tsv
    : > .broad-sample/write-marker ;;
esac
printf 'rustc 1.0.0-sample (0000000 样本)\nhost: x86_64-unknown-linux-gnu\n' > .broad-sample/rustc-vv.txt
printf 'cargo-nextest 0.9.0-sample\ncargo-mutants 25.0.0-sample\nLLVM version 19.0.0-sample\n' > .broad-sample/tool-versions.txt
printf '# 样本的忽略用例排除表（一行都没登记）\n' > .broad-sample/exclusions.tsv
printf '%s\n' "$variant" > .broad-sample/variant
python3 - "$variant" <<'PYEOF'
import hashlib, os, sys
variant = sys.argv[1]
export_paths = ["crates", "Cargo.toml", "Cargo.lock", "litmus", ".cargo"]
if variant == "export-range":
    export_paths.remove(".cargo")
manifest = {}
for export_path in export_paths:
    if os.path.isfile(export_path):
        manifest[export_path] = hashlib.sha256(open(export_path, "rb").read()).hexdigest()
        continue
    for directory, _directories, files in os.walk(export_path):
        for name in files:
            path = os.path.join(directory, name)
            manifest[path] = hashlib.sha256(open(path, "rb").read()).hexdigest()
with open(".broad-sample/manifest.tsv", "w", encoding="utf-8") as handle:
    handle.write("".join(f"{manifest[path]}\t{path}\n" for path in sorted(manifest)))
with open(".broad-sample/export-paths", "w", encoding="utf-8") as handle:
    handle.write(" ".join(export_paths) + "\n")
PYEOF
cat > .broad-sample/prepare.py <<'PYEOF'
#!/usr/bin/env python3
"""样本现场的后半：阶段在样本模式下起它（python3 .broad-sample/prepare.py <驱动>），载入今天的驱动造广谱变异库目录里的那一版。
情形（.broad-sample/variant）：toolchain 那一版记旧工具链；pending 留着 tree.source.pending；version-key 版本指纹照全局键进指纹之前的式子（全局键记得对）；
source-fingerprint tree.source 的源码指纹与 manifest 不是同一次写的；global-key 记全局键时排除表不在（此刻在）；global-key-missing 全局键进指纹之前的驱动换的那一版。"""
import hashlib, importlib.util, json, os, sys, types
sys.dont_write_bytecode = True
specification = importlib.util.spec_from_file_location("broad_mutation_driver_for_sample", sys.argv[1])
driver = importlib.util.module_from_spec(specification)
specification.loader.exec_module(driver)
sample = ".broad-sample"
with open(os.path.join(sample, "variant"), encoding="utf-8") as handle:
    variant = handle.read().strip()
manifest = driver.read_manifest(os.path.join(sample, "manifest.tsv"))
with open(os.path.join(sample, "export-paths"), encoding="utf-8") as handle:
    export_paths = handle.read().split()
source_fingerprint = driver.fingerprint_of_manifest(manifest)
with open(os.path.join(sample, "rustc-vv.txt"), encoding="utf-8") as handle:
    toolchain = handle.read().strip() + "\n"
if variant == "toolchain":
    toolchain = "rustc 0.9.0-sample (1111111 样本：那一版是旧工具链)\nhost: x86_64-unknown-linux-gnu\n"
with open(os.path.join(sample, "tool-versions.txt"), encoding="utf-8") as handle:
    tools = [line.strip() for line in handle if line.strip()]
exclusions = os.path.join(sample, "no-such-exclusions.tsv" if variant == "global-key" else "exclusions.tsv")
limits = driver.limits_line_of(types.SimpleNamespace(cpu_multiplier=3.0, cpu_floor=30.0, wall_multiplier=10.0, wall_floor=300.0))
global_text = driver.global_key_lines(driver.JUDGE_RULES_VERSION, toolchain, manifest, driver.build_environment_of(os.getcwd(), os.environ), tools, driver.exclusions_digest_of(exclusions), limits)
global_digest = hashlib.sha256(global_text.encode()).hexdigest()
home = os.path.join(sample, "home")
if variant in ("version-key", "global-key-missing"):
    fingerprint = hashlib.sha256(f"{source_fingerprint}\n{toolchain}judge {driver.JUDGE_RULES_VERSION}\n".encode()).hexdigest()
else:
    fingerprint = driver.version_key(home, source_fingerprint, toolchain, global_digest)
version = os.path.join(home, "versions", fingerprint)
os.makedirs(version, exist_ok=True)   # 同一个样本根重跑阶段时照造
with open(os.path.join(version, "manifest.tsv"), "w", encoding="utf-8") as handle:
    handle.write("".join(f"{manifest[path]}\t{path}\n" for path in sorted(manifest)))
with open(os.path.join(version, "toolchain.txt"), "w", encoding="utf-8") as handle:
    handle.write(toolchain)
pointer = {"commit": "0123456789abcdef0123456789abcdef01234567", "revision": "HEAD", "fingerprint": fingerprint,
           "source_fingerprint": "0" * 64 if variant == "source-fingerprint" else source_fingerprint, "files": len(manifest), "export_paths": export_paths}
if variant != "global-key-missing":
    with open(os.path.join(version, "global-key.txt"), "w", encoding="utf-8") as handle:
        handle.write(global_text)
    pointer["global_key"] = global_digest
with open(os.path.join(home, "tree.source"), "w", encoding="utf-8") as handle:
    json.dump(pointer, handle, ensure_ascii=False, indent=1)
if variant == "pending":
    with open(os.path.join(home, "tree.source.pending"), "w", encoding="utf-8") as handle:
        json.dump(pointer, handle, ensure_ascii=False, indent=1)
PYEOF
git init -q .
git add -A
case "$variant" in
  unstaged-change|marker-unstaged) printf '// 工作区里改了、没暂存\n' >> crates/sample/src/lib.rs ;;
  file-differs) printf '// 暂存了、那一版没有的改动\n' >> crates/sample/src/lib.rs; git add crates/sample/src/lib.rs ;;
  new-file) printf 'pub fn sub(left: u32, right: u32) -> u32 {\n    left - right\n}\n' > crates/sample/src/extra.rs; git add crates/sample/src/extra.rs ;;
  merge-conflict)
    # 索引里 lib.rs 在合并冲突：第 1、2 段是另一份，第 3 段与那一版相同（工作区不动）
    original="$(git rev-parse :crates/sample/src/lib.rs)"
    other="$(printf 'pub fn add(left: u32, right: u32) -> u32 {\n    left - right\n}\n' | git hash-object -w --stdin)"
    printf '0 %s\tcrates/sample/src/lib.rs\n100644 %s 1\tcrates/sample/src/lib.rs\n100644 %s 2\tcrates/sample/src/lib.rs\n100644 %s 3\tcrates/sample/src/lib.rs\n' \
      0000000000000000000000000000000000000000 "$other" "$other" "$original" | git update-index --index-info ;;
esac
