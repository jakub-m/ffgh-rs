#!/bin/bash
set -eu
set -o pipefail
bin=ffgh-bin
while [ 1 ]; do
  $bin -v sync || true
  echo "RESTART"
  sleep 1
done
