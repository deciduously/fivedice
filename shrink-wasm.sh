#!/bin/bash

PKGDIR='pkg'
BINARY='fivedice_bg' # TODO parameterize?
WASM="$PKGDIR/$BINARY.wasm"

function wasm_size {
    wc -c $1
}

function echo_size {
    echo "$(eval wasm_size $1)"
}

function extract_size {
    wasm_size $1 | sed 's/^\([0-9]\+\).*/\1/'
}

# shink_speed TARGETNAME -f/--focus="size|speed", -l/--level="aggressive|normal", if either isnt understood defaults are speed | normal
function shrink {
    # parse args
    for i in "$@"
    do
    case $i in
        -f=*|--focus=*)
        FOCUS="${i#*=}"
        shift
        ;;
        -l=*|--level=*)
        LEVEL="${i#*=}"
        shift
        ;;
        *)
        # unknown option
        ;;
    esac
    done
    # last line is target, non-opt, no equals sign
    if [[ -n $1 ]]; then
        TARGET=$1
    fi
    ARG='-O'
    if [ "$FOCUS" = "size" ]; then
        if [ "$LEVEL" = "aggressive" ]; then
            ARG="${ARG}z"
        else
            ARG="${ARG}s"
        fi
    else
        if [ "$LEVEL" = "aggressive" ]; then
            ARG="${ARG}3"
        fi
    fi
    COMMAND="wasm-opt $ARG -o $TARGET $WASM"
    echo $COMMAND
    eval $COMMAND
}

function choose_smaller {
    NORMAL='_normal'
    AGGRO='_aggressve'
    NORMAL_TARGET="${PKGDIR}/${BINARY}${NORMAL}.wasm"
    AGGRO_TARGET="${PKGDIR}/${BINARY}${AGGRO}.wasm"
    eval shrink "-f=$1" $NORMAL_TARGET
    NORMAL_SIZE="$(eval extract_size $NORMAL_TARGET)"
    eval shrink "-f=$1" -l=aggressive $AGGRO_TARGET
    AGGRO_SIZE="$(eval extract_size $AGGRO_TARGET)"
    if [ $NORMAL_SIZE -lt $AGGRO_SIZE ]; then
        echo "Normal settings smaller, saving..."; mv $NORMAL_TARGET $WASM; rm $AGGRO_TARGET;
    else
        echo "Aggressive settings smaller, saving..."; mv $AGGRO_TARGET $WASM; rm $NORMAL_TARGET;
    fi
}

echo_size $WASM
echo "Shrinking, optimizing for ${1}"
if [ "$1" = "size" ]; then
    choose_smaller $1
else
    shrink "-f=$1" -l=aggressive $WASM
fi
echo_size $WASM

exit