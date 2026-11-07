"""审核复跑：E200 造树的既有单测与 M40–M47，仅操作临时副本。经 task-server 起。"""

from pathlib import Path
import os
import subprocess
import sys
import tempfile


ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "scripts/experiment-apparatus/e200_revision_6_segment_one_on_an_approximate_tree"
TABLE = ROOT / "scripts/mutation/tables/e200_revision_6_segment_one_on_an_approximate_tree.tsv"
environment = dict(os.environ, PYTHONDONTWRITEBYTECODE="1")
baseline = subprocess.run(
    [sys.executable, "-B", "-m", "unittest", "-v", "test_build_approximate_tree"],
    cwd=SOURCE,
    env=environment,
    check=False,
)
if baseline.returncode:
    sys.exit(baseline.returncode)
with tempfile.TemporaryDirectory(prefix="singlefs-document-audit-e200-") as directory:
    product = Path(directory) / "mutations.out"
    result = subprocess.run(
        [sys.executable, "-B", "mutation_driver.py", "--table", str(TABLE),
         "--scratch", str(Path(directory) / "copies"), "--output", str(product)],
        cwd=SOURCE,
        env=environment,
        check=False,
    )
    if product.exists():
        print(product.read_text(), end="", flush=True)
    sys.exit(result.returncode)
