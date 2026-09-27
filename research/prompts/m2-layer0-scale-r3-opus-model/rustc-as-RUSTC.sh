#!/bin/sh
# 当作 RUSTC 用：转给 PATH 里的 rustc，编 crate 时加一个 cfg。
case " $* " in *" --crate-name "*) exec rustc "$@" --cfg opus_probe ;; *) exec rustc "$@" ;; esac
