# 探针（只读、不编、不起装置、不起真包装）：钉住的一行表 + 点名的变异在 + 退 FAKE_EXIT 的假包装，调 e165-pin-red-examples.py 的 main --replay，打它的 ✗ 与 → 与退出码。
# 用法：python3 e3-e165-exit-codes.py <仓根> <草稿目录> <退出码>
import importlib.util, os, sys, tempfile
sys.dont_write_bytecode = True
root_repo, scratch, code = sys.argv[1], sys.argv[2], sys.argv[3]
sys.path.insert(0, os.path.join(root_repo, "research/scripts"))
spec = importlib.util.spec_from_file_location("e165_pin", os.path.join(root_repo, "research/scripts/e165-pin-red-examples.py"))
module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)
with tempfile.TemporaryDirectory(prefix="e165-exit-", dir=scratch) as root:
    os.makedirs(os.path.join(root, ".claude/gate.d")); os.makedirs(os.path.join(root, "crates"))
    with open(os.path.join(root, "crates/mutations.tsv"), "w") as handle:
        handle.write("# 表头\n红的一条\tcrates/x.rs\ta\tb\t-p x\tt\n")
    original = "E7RESULT name=state domain=L ring_slots=64 n=2 ordinal=3 counters=states state=shape=1c0;first=returned:a"
    row = [module.input_fingerprint("L", "2", original), "L", "2", "3", "probe:2", "crates/mutations.tsv#红的一条", original, original.replace(":a", ":b")]
    with open(os.path.join(root, module.TABLE), "w") as handle:
        handle.write("# 探针用\n" + "\t".join(row) + "\n")
    wrapper = os.path.join(root, "fake-wrapper.sh")
    with open(wrapper, "w") as handle:
        handle.write(f"#!/usr/bin/env bash\nexit {code}\n")
    module.MEMORY_CAP_RUNNER = wrapper
    status = module.main(["--replay", "--root", root, "--binary", "/bin/true", "--memory-cap", "1G"])
    print(f"=> 退出码 {status}")
