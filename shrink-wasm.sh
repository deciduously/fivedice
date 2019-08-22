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

# $1 = target $2 = focus $3 = level
function shrink {
    ARG='-O'
    if [ "$2" = "size" ]; then
        if [ "$3" = "aggro" ]; then
            ARG="${ARG}z"
        else
            ARG="${ARG}s"
        fi
    else
        if [ "$3" = "aggro" ]; then
            ARG="${ARG}3"
        fi
    fi
    COMMAND="wasm-opt $ARG -o $1 $WASM"
    echo $COMMAND
    eval $COMMAND
}

function choose_smaller {
    NORMAL='_normal'
    AGGRO='_aggressve'
    NORMAL_TARGET="${PKGDIR}/${BINARY}${NORMAL}.wasm"
    AGGRO_TARGET="${PKGDIR}/${BINARY}${AGGRO}.wasm"
    shrink $NORMAL_TARGET $2 $3
    NORMAL_SIZE="$(eval extract_size $NORMAL_TARGET)"
    shrink $AGGRO_TARGET $2 $3 
    AGGRO_SIZE="$(eval extract_size $AGGRO_TARGET)"
    if [ $NORMAL_SIZE -lt $AGGRO_SIZE ]; then
        echo "Normal settings smaller, saving..."; mv $NORMAL_TARGET $WASM; rm $AGGRO_TARGET;
    else
        echo "Aggressive settings smaller, saving..."; mv $AGGRO_TARGET $WASM; rm $NORMAL_TARGET;
    fi
}

# parse args
for i in "$@"
do
case $i in
    -f=*|--focus=*)
    FOCUS="${i#*=}"
    shift
    ;;
    -h=*|--help=*)
    SHOWHELP=true
    shift;;
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
if [ -n $1 ]; then
    TARGET=$1
fi

# If help requested, print it
if [ -n $SHOWHELP ]; then
    echo "Usage: $ ./shrink-wasm.sh {-f/--focus}={speed|size} {-l/--level}={normal|aggro} target"
    echo ""
    echo "Defaults if either not found or not spelled correctly are \"speed\" and \"normal\""
    echo ""
fi

echo_size $WASM
if [ -z $FOCUS ]; then
    FOCUS_STR='speed'
else
    FOCUS_STR=$FOCUS
fi
echo "Shrinking, optimizing for ${FOCUS_STR}."
if [ "$LEVEL" = "aggro" ]; then
    echo "Using aggressive optimizations."
fi
if [ "$FOCUS" = "size" ]; then
    choose_smaller $1
else
    shrink $WASM $FOCUS $LEVEL
fi
echo_size $WASM

exit