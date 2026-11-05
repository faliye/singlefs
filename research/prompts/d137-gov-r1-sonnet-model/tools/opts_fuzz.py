#!/usr/bin/env python3
"""bash/sh 选项写法的差分：每种写法在真 bash 里是否真起 cargo 垫片，钩子判不判。"""
import sys, os, itertools, warnings
warnings.filterwarnings("ignore")
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import feed, truth, sandbox_reset
P = "'cargo test --all'"
forms = []
for sh in ("bash", "sh", "dash"):
    for opts in ["-c", "-ec", "-ce", "-uc", "-xc", "-lc", "-euc", "-euo pipefail -c", "-eo pipefail -c", "-euxo pipefail -c", "-o pipefail -c", "-o errexit -c", "+o history -c",
                 "-O extglob -c", "-e -u -o pipefail -c", "-eco pipefail", "-ceo pipefail", "-c -e", "-c -o pipefail", "--norc -c", "--noprofile --norc -c", "--posix -c", "-eu -c",
                 "-Eeuo pipefail -c", "-Eeo pipefail -c", "-eE -o pipefail -c", "-vc", "-ec -o pipefail", "-o pipefail -ec", "-ouc pipefail"]:
        forms.append(f"{sh} {opts} {P}")
forms += [f"bash -c {P} arg0 arg1", f"bash -c {P} _", f"env bash -euo pipefail -c {P}", f"nice bash -euo pipefail -c {P}", f"timeout 5 bash -euo pipefail -c {P}",
          f"bash -euo pipefail heavy.sh", f"bash -eo pipefail heavy.sh", f"bash -o pipefail heavy.sh", f"bash -eux heavy.sh", f"sh -eu heavy.sh", f"bash -euo pipefail ./heavy.sh",
          f"bash -euo pipefail -s < heavy.sh", f"bash --rcfile benign.sh -c {P}", f"bash --init-file benign.sh -c {P}", f"bash --rcfile benign.sh heavy.sh"]
rows = []
for f in forms:
    sandbox_reset.reset()
    rc, err, notes = feed.feed("heavy-test-guard.sh", f, agent="implementation-writer")
    sandbox_reset.reset()
    trc, calls, out = truth.truth(f)
    ran = any(x.startswith("CARGO test") for x in calls)
    sandbox_reset.reset()
    rows.append((rc, "真起" if ran else "没起", "漏拒" if (rc == 0 and ran) else ("误拒" if rc == 2 and not ran else "符合"), len(notes), f))
for r in rows: print("\t".join(map(str, r)))
