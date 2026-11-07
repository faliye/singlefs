#!/usr/bin/env bash
rsync -a "$STAGE_REPOSITORY/.claude/gate.d/lib/" "$1/lib/"
