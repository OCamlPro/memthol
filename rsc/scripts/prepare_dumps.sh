#!/bin/bash

# This is meant to run from the root of the repository.

my_name=`basename "$0"`

init_name="init.memthol"
diff_tail=".memthol.diff"

rsc_dir="./rsc"
test_dirs=(
    "$rsc_dir/ackermann_with_sets"
)

if [ ! -d "$rsc_dir" ] ; then
    >&2 echo "Error, script $my_name must run from the root of the memthol-ui repository."
    exit 2
fi

for test_dir in $test_dirs ; do
    printf "preparing $test_dir ..."

    if [ ! -d "$test_dir" ] ; then
        echo
        >&2 echo "Error, could not find test-directory $test_dir"
        exit 2
    fi

    touch "$test_dir/$init_name"

    for dump_file in $test_dir/*$diff_tail ; do
        sleep 0.005
        touch $dump_file
    done

    echo " done"
done