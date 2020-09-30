#!/usr/bin/env bash

CHECK_FILES=(
  "target/x86_64-unknown-linux-musl/release/modbusgw"
  "target/arm-unknown-linux-musleabihf/release/modbusgw"
  "target/x86_64-pc-windows-gnu/release/modbusgw.exe"
)

for f in ${CHECK_FILES[@]}; do
  echo -n "$f "
  if [[ $f == *"target/arm-"* ]]; then
    file $f | grep "statically linked, stripped$" > /dev/null || exit 1
  elif [[ $f == *"target/x86_64-pc-windows-"* ]]; then
    file $f | grep "(stripped to external PDB)" > /dev/null || exit 2
  else
    ldd $f | grep -E "statically linked|not a dynamic executable" > /dev/null || exit 3
    file $f | grep ", stripped$" > /dev/null || exit 4
  fi
  echo "OK"
done

echo "FILES CHECKED"
