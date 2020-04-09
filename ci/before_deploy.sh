# This script takes care of building your crate and packaging it for release

set -ex

main() {
    src=$(pwd)
    stage=$(mktemp -d)

    test -f Cargo.lock || cargo generate-lockfile

    if [ "$TRAVIS_OS_NAME" = "windows" ]; then
        export PATH="/c/PythonForO2:/c/PythonForO2/Scripts:$PATH"
    else
        source /home/travis/virtualenv/python$PYTHON_VERSION/bin/activate
    fi
    
    cd origen
    cargo build --target $TARGET --release --workspace --bins
    cp target/$TARGET/release/origen $stage/
    cd ../

    cd $stage
    tar czf $src/$CRATE_NAME-$TRAVIS_TAG-$TARGET.tar.gz *
    cd $src

    rm -rf $stage
}

main