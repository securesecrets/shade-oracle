#!/bin/bash

for f in artifacts/*.wasm;
    do
        echo $f
        # echo $(md5sum $f | cut -f 1 -d " ") >> ${CHECKSUM_DIR}/$f.txt
        # cat ./$f.wasm | gzip -n -9 > ${COMPILED_DIR}/$f.wasm.gz
        # rm ./$f.wasm
    done;
