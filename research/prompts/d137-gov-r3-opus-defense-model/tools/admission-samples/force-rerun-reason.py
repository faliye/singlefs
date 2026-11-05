import os, sys
if os.environ.get("SINGLEFS_EXPERIMENT_RERUN_REASON", ""):
    print("  ✓ 放行（SINGLEFS_EXPERIMENT_RERUN_REASON 给了理由）", file=sys.stderr)
    sys.exit(0)
print("  ✗ 拒绝开跑 E142：实验键没变", file=sys.stderr)
sys.exit(77)
