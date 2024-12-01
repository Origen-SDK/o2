#!/bin/bash

echo "========================================"
echo "=====         Install Rust         ====="
echo "========================================"
curl https://sh.rustup.rs -sSf | sh -s -- -y
source ${HOME}/.cargo/env

echo "========================================"
echo "=====       Set Rust Version       ====="
echo "========================================"
rustup install ${RUST_VERSION}
rustup default ${RUST_VERSION}

echo "========================================"
echo "=====      Check Rust Version      ====="
echo "========================================"
rustc --version
cargo --version
read -p "Pausing for 5 seconds" -t 5

echo "========================================"
echo "=====    Install Newer OpenSSL     ====="
echo "========================================"
curl -O -L https://www.openssl.org/source/openssl-1.1.1w.tar.gz
ls -al openssl-1.1.1w.tar.gz
tar zxf openssl-1.1.1w.tar.gz
cd openssl-1.1.1w
./config
make
make install
cd $ROOT_DIR

echo "========================================"
echo "=====  Save Minor Python Version   ====="
echo "========================================"
IFS='.' read -r -a SPLIT_VER <<< ${PYTHON_VERSION}
PY_M_VER=${SPLIT_VER[0]}.${SPLIT_VER[1]}
echo $PY_M_VER

LIBFFI_VER="3.12"
if [[ $PY_M_VER == $LIBFFI_VER ]]; then
    echo "========================================"
    echo "=====        Install libffi        ====="
    echo "========================================"
    yum install libffi-devel -y
    ldconfig
else
    LOW=$(echo -e "$PY_M_VER\n$LIBFFI_VER" | sort --version-sort | head --lines=1)
    if [[ $LOW != $PY_M_VER ]]; then
        echo "========================================"
        echo "=====        Install libffi        ====="
        echo "========================================"
        yum install libffi-devel -y
        ldconfig
    fi
fi

echo "========================================"
echo "=====     Install Perl-IPC-cmd     ====="
echo "========================================"
yum install perl-IPC-Cmd -y

echo "========================================"
echo "=====        Install Python        ====="
echo "========================================"
ls $ROOT_DIR/openssl-1.1.1w
curl -O https://www.python.org/ftp/python/${PYTHON_VERSION}/Python-${PYTHON_VERSION}.tgz
tar zxf Python-${PYTHON_VERSION}.tgz
cd Python-${PYTHON_VERSION}
./configure --with-openssl=$ROOT_DIR/openssl-1.1.1w --prefix=/root/python --enable-optimizations --enable-shared
make altinstall

if [[ $PYTHON_VERSION == "3.7.17" ]]; then
    echo "========================================"
    echo "Copy Python Shared Library (Python 3.7) "
    echo "========================================"
    echo $PY_M_VER
    cd $ROOT_DIR/Python-${PYTHON_VERSION}
    ls
    cp libpython${PY_M_VER}\m.so.1.0 /usr/local/lib64/
    cd $ROOT_DIR
else
    echo "========================================"
    echo "Copy Python Shared Library (Python 3.8+)"
    echo "========================================"
    echo $PY_M_VER
    cd $ROOT_DIR/Python-${PYTHON_VERSION}
    ls
    cp libpython${PY_M_VER}.so.1.0 /usr/local/lib64/
    cd $ROOT_DIR
fi

echo "========================================"
echo "=====     Check LD_LIBRARY_PATH    ====="
echo "========================================"
echo $LD_LIBRARY_PATH

echo "========================================"
echo "===== Alias Python and Pip binaries ===="
echo "========================================"
echo $PY_M_VER
ls /root/python/bin
ln -s /root/python/bin/python${PY_M_VER} /root/python/bin/python
ln -s /root/python/bin/pip${PY_M_VER} /root/python/bin/pip
ls /root/python/bin

echo "========================================"
echo "=====          Update $PATH         ===="
echo "========================================"
export PATH=/root/python/bin:$PATH

echo "========================================"
echo "=====    Display Python Version     ===="
echo "========================================"
which python
which pip
python --version
pip --version

echo "========================================"
echo "=====        Install Poetry         ===="
echo "========================================"
pip install poetry==1.3.2
poetry --version

echo "========================================"
echo "=====      Install Auditwheel       ===="
echo "========================================"
pip install setuptools
pip install auditwheel
auditwheel --version
