import os
import subprocess

threads = int(os.environ["SAMPLE_OWN_JOBS"])
subprocess.run(["bash", "scripts/hosts/capped.sh", "default", "true"], check=True)
