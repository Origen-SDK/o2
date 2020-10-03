# This script is responsible for executing the Rust and Python tests,
# it is skipped when running a deploy build (on a tag)
#
# PWD on entry and exit is o2/rust/

set -ex

main() {
    if [ "$TRAVIS_OS_NAME" = "windows" ]; then
        export PATH="/c/PythonForO2:/c/PythonForO2/Scripts:$PATH"
    else
        source /home/travis/virtualenv/python$PYTHON_VERSION/bin/activate
    fi

    #if [ ! -z $DISABLE_TESTS ]; then
    #    return
    #fi

    if [ "$O2_REGRESSION" = "BACKEND" ]; then
        cd origen
        # don't know why this isn't set by cargo in the ci env
        export CARGO_BIN_EXE_ORIGEN="../target/$TARGET/debug/origen"
        #cargo test --target $TARGET --workspace --release
        cargo test --target $TARGET --workspace
        cd ../
    else
        # Build the CLI
        cd origen
        #cargo build --target $TARGET --release --workspace --bins
        cargo build --target $TARGET --workspace --bins
        cd ../

        # pass the path for the CLI tests to work
        export TRAVIS_ORIGEN_CLI="../../rust/origen/target/$TARGET/debug/origen"
        cd ../test_apps/python_app
        ../../rust/origen/target/$TARGET/debug/origen build --target $TARGET
        ../../rust/origen/target/$TARGET/debug/origen env setup
        ../../rust/origen/target/$TARGET/debug/origen -v
        ../../rust/origen/target/$TARGET/debug/origen exec pytest -vv
        cd ../../rust
    fi
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi
