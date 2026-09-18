#!/usr/bin/env bash
# gate-stage: red
if grep -rq BAD kb/; then echo "  ✗ kb 里有 BAD"; echo "    → 删掉"; exit 1; fi
exit 0
