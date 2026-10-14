import subprocess
SAMPLE = "bash scripts/hosts/capped.sh 4 cargo build"
subprocess.run(["bash", "scripts/hosts/capped.sh", "16", "cargo", "build"], check=False)
