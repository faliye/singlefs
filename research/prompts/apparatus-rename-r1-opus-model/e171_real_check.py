#!/usr/bin/env python3
"""只读：用一棵树里的 frozen_o3 / reuse_library，对仓根（--repo）里此刻的 E171 冻结执行器与两个钉值各判一次。用法：python3 e171_real_check.py <树> <仓根> <标签>"""
import os
import sys

sys.dont_write_bytecode = True
tree, repo, label = sys.argv[1:4]
D = "scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes"
sys.path.insert(0, os.path.join(tree, D))
import frozen_o3  # noqa: E402
import reuse_library  # noqa: E402
print(f"name=e171_real.{label} freeze_differences={frozen_o3.freeze_differences(repo)!r}")
for name, pinned in reuse_library.PINNED_SHA256.items():
    path = os.path.join(repo, D, os.path.basename(reuse_library.PINNED_PATHS[name]))
    print(f"name=e171_real.{label} pin={name} accepted={reuse_library.pinned_sha256_now(path, pinned, repo) is not None}")
