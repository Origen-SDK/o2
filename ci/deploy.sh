# This script takes care of building your crate and packaging it for release

set -ex

main() {
    if [ "$TRAVIS_OS_NAME" = "windows" ]; then
        export PATH="/c/PythonForO2:/c/PythonForO2/Scripts:$PATH"
        export RUSTFLAGS ="-C target-feature=+crt-static"
    else
        source /home/travis/virtualenv/python$PYTHON_VERSION/bin/activate
    fi
        
    # This publishes the CLI to the Github releases page
    if [ "$O2_REGRESSION" = "BACKEND" ]; then
        src=$(pwd)
        stage=$(mktemp -d)

        cd origen

        test -f Cargo.lock || cargo generate-lockfile
        
        cargo build --target $TARGET --release --workspace --bins
        cp target/$TARGET/release/origen $stage/

        cd $stage
        tar czf $src/$CRATE_NAME-$TRAVIS_TAG-$TARGET.tar.gz *
        cd $src

        rm -rf $stage

    # This publishes the Python package (for the current Python version and platform) to PyPI
    else
        pip3 install maturin
        pip3 install poetry
        pip3 install twine

        # Make the CLI available
        cd origen
        cargo build --target $TARGET --workspace --bins

        target/$TARGET/debug/origen build --release --publish

        if [ $BUILD_ORIGEN_PYTHON ]; then
            target/$TARGET/debug/origen build --release --publish --python
        fi
    fi
}

main
