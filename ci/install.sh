set -ex

main() {

    if [ "$TRAVIS_OS_NAME" = "linux" ]; then
        rustup target install $TARGET
    fi

    # Install Python
    if [ "$TRAVIS_OS_NAME" = "windows" ]; then
        choco install python3 --version $PYTHON_VERSION --override --installarguments '/quiet InstallAllUsers=1 PrependPath=1 "TargetDir=C:\PythonForO2"'
    else
        archive_url=https://storage.googleapis.com/travis-ci-language-archives/python/binaries/ubuntu/16.04/x86_64/python-$PYTHON_VERSION.tar.bz2
        curl -sSf --retry 5 -o python-$PYTHON_VERSION.tar.bz2 $archive_url
        sudo tar xjf python-$PYTHON_VERSION.tar.bz2 --directory /
        mkdir -p /home/travis/bin
        # Not sure how to dynamically update the PATH from here (can't export vars to the next section), but
        # this will suffice (~/bin is at the top of the PATH)
        ln -s /home/travis/virtualenv/python$PYTHON_VERSION/bin/python /home/travis/bin/python
        ln -s /home/travis/virtualenv/python$PYTHON_VERSION/bin/python3 /home/travis/bin/python3
        ln -s /home/travis/virtualenv/python$PYTHON_VERSION/bin/pip /home/travis/bin/pip
        ln -s /home/travis/virtualenv/python$PYTHON_VERSION/bin/pip3 /home/travis/bin/pip3
    fi
    if [ "$TRAVIS_OS_NAME" = "windows" ]; then
        export PATH="/c/PythonForO2:/c/PythonForO2/Scripts:$PATH"
        # pip >= 19.1 required to fix a bug that impacts installing our python app
        C:\\PythonForO2\\python.exe -m pip install --upgrade pip
    else
        source /home/travis/virtualenv/python$PYTHON_VERSION/bin/activate
        # pip >= 19.1 required to fix a bug that impacts installing our python app
        pip3 install --upgrade pip
    fi
    #python3 --version
}

main
