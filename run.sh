#!/bin/bash
export LD_LIBRARY_PATH=$(pwd)/bass-sys/linux/bass24/x64
echo $LD_LIBRARY_PATH
exec mold -run cargo run --release -- "$@"
