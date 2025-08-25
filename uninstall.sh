#!/bin/sh

rm /usr/local/bin/asus-px-keyboard-tool
rm /etc/systemd/system/asus-px-keyboard-tool.service
rm /etc/systemd/system/asus-px-keyboard-tool-restore.service
rm /etc/asus-px-keyboard-tool.conf
rm -rf /var/lib/asus-px-kb-tool/ || true