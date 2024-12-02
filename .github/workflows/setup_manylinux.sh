#!/bin/bash

echo -e "\nInstall Rust"
echo "========================================"
curl https://sh.rustup.rs -sSf | sh -s -- -y
source ${HOME}/.cargo/env

echo -e "\nSet Rust Version"
echo "========================================"
rustup install ${RUST_VERSION}
rustup default ${RUST_VERSION}

echo -e "\nCheck Rust Version"
echo "========================================"
rustc --version
cargo --version

echo -e "\nInstall Newer OpenSSL"
echo "========================================"
curl -O -L https://www.openssl.org/source/openssl-1.1.1w.tar.gz
ls -al openssl-1.1.1w.tar.gz
tar zxf openssl-1.1.1w.tar.gz
cd openssl-1.1.1w
./config
make
make install
cd $ROOT_DIR

echo -e "\nSave Minor Python Version"
echo "========================================"
IFS='.' read -r -a SPLIT_VER <<< ${PYTHON_VERSION}
PY_M_VER=${SPLIT_VER[0]}.${SPLIT_VER[1]}
echo $PY_M_VER

LIBFFI_VER="3.12"
if [[ $PY_M_VER == $LIBFFI_VER ]]; then
    echo -e "\nInstall libffi"
    echo "========================================"
    yum install libffi-devel -y
    ldconfig
else
    LOW=$(echo -e "$PY_M_VER\n$LIBFFI_VER" | sort --version-sort | head --lines=1)
    if [[ $LOW != $PY_M_VER ]]; then
        echo -e "\nInstall libffi"
        echo "========================================"
        yum install libffi-devel -y
        ldconfig
    fi
fi

echo -e "\nInstall Perl-IPC-cmd"
echo "========================================"
yum install perl-IPC-Cmd -y

echo -e "\nInstall Python"
echo "========================================"
ls $ROOT_DIR/openssl-1.1.1w
curl -O https://www.python.org/ftp/python/${PYTHON_VERSION}/Python-${PYTHON_VERSION}.tgz
tar zxf Python-${PYTHON_VERSION}.tgz
cd Python-${PYTHON_VERSION}
./configure --with-openssl=$ROOT_DIR/openssl-1.1.1w --prefix=/root/python --enable-optimizations --enable-shared
make altinstall

if [[ $PYTHON_VERSION == "3.7.17" ]]; then
    echo -e "\nCopy Python Shared Library (Python 3.7)"
    echo "========================================"
    echo $PY_M_VER
    cd $ROOT_DIR/Python-${PYTHON_VERSION}
    ls
    cp libpython${PY_M_VER}\m.so.1.0 /usr/local/lib64/
    cd $ROOT_DIR
else
    echo -e "\nCopy Python Shared Library (Python 3.8+)"
    echo "========================================"
    echo $PY_M_VER
    cd $ROOT_DIR/Python-${PYTHON_VERSION}
    ls
    cp libpython${PY_M_VER}.so.1.0 /usr/local/lib64/
    cd $ROOT_DIR
fi

echo -e "\nCheck LD_LIBRARY_PATH"
echo "========================================"
echo $LD_LIBRARY_PATH

echo -e "\nAlias Python and Pip binaries"
echo "========================================"
echo $PY_M_VER
ls /root/python/bin
ln -s /root/python/bin/python${PY_M_VER} /root/python/bin/python
ln -s /root/python/bin/pip${PY_M_VER} /root/python/bin/pip
ls /root/python/bin

echo -e "\nUpdate PATH"
echo "========================================"
export PATH=/root/python/bin:$PATH

echo -e "\nDisplay Python Version"
echo "========================================"
which python
which pip
python --version
pip --version

echo -e "\nInstall Poetry"
echo "========================================"
pip install poetry==1.3.2
poetry --version

echo -e "\nInstall Auditwheel"
echo "========================================"
pip install setuptools
pip install auditwheel
auditwheel --version

echo -e "\nBuild Origen Metal Python Package"
echo "========================================"
cd ${GIT_DIR}/python/origen_metal
poetry build --format wheel

echo -e "\nDisplay OM Dist Directory="
echo "========================================"
cd ${GIT_DIR}/python/origen_metal
ls dist

echo -e "\nRepair OM Wheel"
echo "========================================"
cd ${GIT_DIR}/python/origen_metal
auditwheel show dist/*
auditwheel repair dist/*

echo -e "\nDisplay OM Wheelhouse Directory"
echo "========================================"
cd ${GIT_DIR}/python/origen_metal
ls wheelhouse
OM_WHEEL=$( ls wheelhouse | head -1 )

echo -e "\nDisplay OM Wheel Name"
echo "========================================"
echo $OM_WHEEL

echo -e "\nGet OM Python Package Version"
echo "========================================"
cd ${GIT_DIR}/python/origen_metal
poetry version -s > $OM_VER_FILE

echo -e "\nBuild Origen Python Package"
echo "========================================"
cd ${GIT_DIR}/python/origen
poetry build --format wheel

echo -e "\nDisplay Origen Dist Directory"
echo "========================================"
cd ${GIT_DIR}/python/origen
ls dist

echo -e "\nRepair Origen Wheel"
echo "========================================"
cd ${GIT_DIR}/python/origen
auditwheel show dist/*
auditwheel repair dist/*

echo -e "\nDisplay Origen Wheelhouse Directory"
echo "========================================"
cd ${GIT_DIR}/python/origen
ls wheelhouse
ORIGEN_WHEEL=$( ls wheelhouse | head -1 )

echo -e "\nDisplay Origen Wheelhouse Directory"
echo "========================================"
cd ${GIT_DIR}
ls -al python/origen/origen/__bin__/bin
ls -al rust/pyapi/target/release
echo $ORIGEN_WHEEL

echo -e "\nGet Origen Python Package Version"
echo "========================================"
cd ${GIT_DIR}/python/origen
poetry version -s > $ORIGEN_VER_FILE


