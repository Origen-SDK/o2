#!/bin/bash
set -euo pipefail

export ORIGEN_PUBLISH_STEP=1
ROOT_DIR="${ROOT_DIR:-$PWD}"

: "${GIT_DIR:?GIT_DIR is required}"
: "${PYTHON_VERSION:?PYTHON_VERSION is required}"
: "${PACKAGE_TO_BUILD:?PACKAGE_TO_BUILD is required}"

if [[ "${PACKAGE_TO_BUILD}" != "origen" && "${PACKAGE_TO_BUILD}" != "origen_metal" ]]; then
    echo "PACKAGE_TO_BUILD must be either 'origen' or 'origen_metal'"
    exit 1
fi
shopt -s nullglob

single_wheel() {
    local directory="$1"
    local wheels=("${directory}"/*.whl)
    if [[ ${#wheels[@]} -ne 1 ]]; then
        echo "Expected exactly one wheel in ${directory}, found ${#wheels[@]}" >&2
        ls -al "${directory}" >&2 || true
        return 1
    fi
    basename "${wheels[0]}"
}

echo -e "\nInstall Rust"
echo "========================================"
curl https://sh.rustup.rs -sSf | sh -s -- -y
source ${HOME}/.cargo/env

echo -e "\nSet Rust Version"
echo "========================================"
rustup install stable
rustup default stable

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

echo -e "\nInstall libffi"
echo "========================================"
yum install libffi-devel -y
ldconfig

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

echo -e "\nInstall UV"
echo "========================================"
curl -LsSf https://astral.sh/uv/0.12.5/install.sh | sh
export PATH="$HOME/.local/bin:$PATH"
uv --version

echo -e "\nInstall Auditwheel"
echo "========================================"
pip install setuptools
pip install auditwheel
auditwheel --version

if [[ "${PACKAGE_TO_BUILD}" == "origen_metal" ]]; then
echo -e "\nBuild Origen Metal Python Package"
echo "========================================"
cd ${GIT_DIR}/python/origen_metal
uv build --wheel --clear --no-create-gitignore

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
OM_WHEEL=$(single_wheel wheelhouse)

echo -e "\nDisplay OM Wheel Name"
echo "========================================"
echo $OM_WHEEL

echo -e "\nGet OM Python Package Version"
echo "========================================"
cd ${GIT_DIR}/python/origen_metal
python -c 'import pathlib,re; print(re.search(r"^version\s*=\s*\"([^\"]+)\"", pathlib.Path("pyproject.toml").read_text(), re.M).group(1))' > $OM_VER_FILE

elif [[ "${PACKAGE_TO_BUILD}" == "origen" ]]; then
echo -e "\nBuild Origen Python Package"
echo "========================================"
cd ${GIT_DIR}
cargo build --manifest-path rust/origen/cli/Cargo.toml --release --bin origen
cp rust/origen/target/release/origen python/origen/origen/__bin__/bin/origen
cd ${GIT_DIR}/python/origen
uv build --wheel --clear --no-create-gitignore

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
ORIGEN_WHEEL=$(single_wheel wheelhouse)

echo -e "\nDisplay Origen Wheelhouse Directory"
echo "========================================"
cd ${GIT_DIR}
ls -al python/origen/origen/__bin__/bin
ls -al rust/pyapi/target/release
echo $ORIGEN_WHEEL

echo -e "\nGet Origen Python Package Version"
echo "========================================"
cd ${GIT_DIR}/python/origen
python -c 'import pathlib,re; print(re.search(r"^version\s*=\s*\"([^\"]+)\"", pathlib.Path("pyproject.toml").read_text(), re.M).group(1))' > $ORIGEN_VER_FILE
fi
