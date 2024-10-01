#!/usr/bin/env bash

python3 -c "$(curl -fsSL -o get-platformio.py https://raw.githubusercontent.com/platformio/platformio-core-installer/5f852c87e5647a3cfa9ef470322aa2c788179e2c/get-platformio.py)"
export PATH=$PATH:/home/iop/.platformio/penv/bin
whoami
cp server server-running
./server-running 2>&1 | tee -a ./monitor.log
