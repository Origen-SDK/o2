# This script takes care of testing your crate

set -ex

# TODO This is the "test phase", tweak it as you see fit
main() {
    if [ "$TRAVIS_OS_NAME" = "windows" ]; then
        export PATH="/c/PythonForO2:/c/PythonForO2/Scripts:$PATH"
    else
        source /home/travis/virtualenv/python$PYTHON_VERSION/bin/activate
    fi

    cd origen
    #cargo build --target $TARGET --release --workspace --bins
    cargo build --target $TARGET --workspace --bins
    cd ../

    if [ "$O2_REGRESSION" = "FRONTEND" ]; then
        cd pyapi
        #cargo build --release
        cargo build --target $TARGET
        cd ../
        if [ "$TRAVIS_OS_NAME" = "windows" ]; then
            cp pyapi/target/$TARGET/debug/_origen.dll ../python/_origen.pyd
        else
            rm -f ../python/_origen.so || true
            cp pyapi/target/$TARGET/debug/lib_origen.so ../python/_origen.so
        fi
    fi

    #if [ ! -z $DISABLE_TESTS ]; then
    #    return
    #fi

    if [ "$O2_REGRESSION" = "BACKEND" ]; then
        cd origen
        #cargo test --target $TARGET --release
        cargo test --target $TARGET
        # cli tests were skipped above
        cd cli
        # don't know why this isn't set by cargo in the ci env
        export CARGO_BIN_EXE_ORIGEN="../target/$TARGET/debug/origen"
        cargo test --target $TARGET
        cd ../../
    else
        # pass the path for the CLI tests to work
        export TRAVIS_ORIGEN_CLI="../../rust/origen/target/$TARGET/debug/origen"
        cd ../test_apps/python_app
        ../../rust/origen/target/$TARGET/debug/origen -v
        ../../rust/origen/target/$TARGET/debug/origen env setup
        ../../rust/origen/target/$TARGET/debug/origen exec pytest -vv
        cd ../../rust
    fi
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi
