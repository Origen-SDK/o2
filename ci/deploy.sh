# This script takes care of building your crate and packaging it for release

set -ex

main() {
    # This publishes the CLI to the Github releases page
    if [ "$O2_REGRESSION" = "BACKEND" ]; then
        if [ "$TRAVIS_OS_NAME" = "windows" ]; then
            export PATH="/c/PythonForO2:/c/PythonForO2/Scripts:$PATH"
        else
            source /home/travis/virtualenv/python$PYTHON_VERSION/bin/activate
        fi
        
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

    # This publishes the Python package (for the current Python version) to PyPI
    else


    fi
}

main
