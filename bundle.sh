#!/bin/bash

# HOW TO USE:
# run the script with the path of the directory you want to sort, e.g.
# bash sort.bash /tmp    --- Will sort /tmp
# bash sort.bash .       --- Will sort current directory
# bash sort.bash ../src  --- Will sort the src directory in the directory above the current one

# Get the list of files
files=($(ls $1 | grep wasm\.gz))

p=$(pwd)

if [[ -nz $(echo ${files[0]} | grep -o aarch64) ]]; then
  # has aarch64
  declare -a contracts=()
  declare -a names=()
  for i in "${files[@]}"; do
    contracts+=($(echo $i | grep .*\.wasm\.gz*))
  done
  for e in "${contracts[@]}"; do
    names+=(${e::${#e}-16})
  done
  cd $1
  for contract in "${names[@]}"; do
    mkdir $contract
    mv $contract-* $contract/$contract.wasm.gz
    mv $contract.json $contract/$contract.json
  done
else
  # no aarch64
  declare -a contracts=()
  declare -a names=()
  for i in "${files[@]}"; do
    contracts+=($(echo $i | grep .*\.wasm\.gz*))
  done
  for e in "${contracts[@]}"; do
    names+=(${e::${#e}-8})
  done
  for contract in "${names[@]}"; do
    cd $1
    mkdir $contract
    mv $contract.* $contract/
  done
fi

cd $p