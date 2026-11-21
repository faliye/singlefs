#!/usr/bin/env bash
python3 - <<'PY'
import os
import subprocess


class OtherLauncher:
    def run(self, command):
        return subprocess.run(command, check=False)


OtherLauncher().run(os.environ["PROBE_COMMAND"].split())
PY
