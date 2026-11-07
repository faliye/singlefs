#!/usr/bin/env python3
"""Audit-only reproduction: real E200 wrapper, fake cargo/cp, temporary files only."""
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts/experiment-apparatus/e200_revision_6_segment_one_on_an_approximate_tree/compile_in_the_approximate_tree.sh"
BINARIES = ("e171_three_oracle_collusion_after_the_oracle_fixes", "e170_o3_spec_executor_export")


def executable(path, body):
    path.write_text(body)
    path.chmod(0o755)


def scenario(fail_copy):
    with tempfile.TemporaryDirectory(prefix="singlefs-doc-copy-audit-") as temporary:
        root = Path(temporary)
        tree, build, fake = root / "tree", root / "build", root / "bin"
        for directory in (tree / "target/release", build / "release", fake):
            directory.mkdir(parents=True)
        (tree / "Cargo.toml").write_text('[workspace]\nmembers = []\n')
        (root / "manifest.tsv").write_text("place\tpath\tclass\tstatus\tpinned\tmarker\tsource\tsha256\n")
        for binary in BINARIES:
            (tree / "target/release" / binary).write_text("old-binary\n")
            (build / "release" / binary).write_text("new-binary\n")
        executable(fake / "cargo", "#!/bin/sh\nexit 0\n")
        if fail_copy:
            executable(fake / "cp", "#!/bin/sh\necho audit-injected-copy-failure >&2\nexit 1\n")
        environment = dict(os.environ, PATH=str(fake) + os.pathsep + os.environ["PATH"],
                           CARGO_TARGET_DIR=str(build), PYTHONDONTWRITEBYTECODE="1")
        product = root / "product.out"
        result = subprocess.run(["bash", str(SCRIPT), str(product)], cwd=tree,
                                env=environment, capture_output=True, text=True)
        verdict = next(line for line in product.read_text().splitlines() if "name=verdict " in line)
        contents = [(tree / "target/release" / binary).read_text() for binary in BINARIES]
        print(json.dumps({"case": "copy_failure_with_old_binaries" if fail_copy else "normal_copy",
                          "exit": result.returncode, "verdict": verdict,
                          "contents": contents, "stderr": result.stderr}, ensure_ascii=False))
        assert result.returncode == 0 and "compiled=true" in verdict and "tree_unchanged=true" in verdict
        assert contents == (["old-binary\n"] * 2 if fail_copy else ["new-binary\n"] * 2)
        if fail_copy:
            assert result.stderr.count("audit-injected-copy-failure") == 2


if __name__ == "__main__":
    scenario(False)
    scenario(True)
    print("REPRODUCED: E200 wrapper reports success with stale binaries after both copy commands fail")
