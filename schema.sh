#!/bin/bash

for d in contracts/*;
    do
        [ -d $d ] && cd "$d" && echo Entering into $d and generating schema. && cargo schema
        cd ..
    done;