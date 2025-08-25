#!/bin/sh
set -e

# check common binary locations
if [ -f asus-px-keyboard-tool ]; then
  install -v asus-px-keyboard-tool /usr/local/bin/
elif [ -f target/debug/asus-px-keyboard-tool ]; then
  install -v target/debug/asus-px-keyboard-tool /usr/local/bin/asus-px-keyboard-tool
elif [ -f target/release/asus-px-keyboard-tool ]; then
  install -v target/release/asus-px-keyboard-tool /usr/local/bin/asus-px-keyboard-tool
else
  echo "Error: could not find asus-px-keyboard-tool binary!"
  exit 1
fi

install -dv /var/lib/asus-px-kb-tool/
install -v asus-px-keyboard-tool.service /etc/systemd/system/
install -v asus-px-keyboard-tool-restore.service /etc/systemd/system/

if [ -f /etc/asus-px-keyboard-tool.conf ]; then
  echo "Warning: /etc/asus-px-keyboard-tool.conf already exists, not overwriting!"
else
  install -v asus-px-keyboard-tool.conf /etc/
fi