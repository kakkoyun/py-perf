#!/usr/bin/env bash

# Copyright (c) 2022 The rbperf authors
#
# This source code is licensed under the MIT license found in the
# LICENSE file in the root directory of this source tree.
set -o errexit nounset pipefail

NPROC=$(nproc --all)
ELFUTILS_VERSION="0.188"
ELFUTILS_SHA_512="585551b2d937d19d1becfc2f28935db1dd1a3d25571a62f322b70ac8da98c1a741a55d070327705df6c3e2ee026652e0b9a3c733b050a0b0ec5f2fc75d5b74b5"

ZLIB_VERSION="1.2.13"
ZLIB_SHA256="b3a24de97a8fdbc835b9833169501030b8977031bcb54b3b3ac13740f846ab30"

run() {
    "$@" 2>&1
}

mkdir -p target/static-libs
mkdir -p target/static-libs/libz
mkdir -p target/static-libs/elfutils
STATIC_LIBS_OUT_PATH="${PWD}/target/static-libs"

run pushd "${STATIC_LIBS_OUT_PATH}"

# Notes:
# * -fpic is not the same as -FPIC
# https://gcc.gnu.org/onlinedocs/gcc/Code-Gen-Options.html
#
# * cflags required for clang to compile elfutils
export CFLAGS="-fno-omit-frame-pointer -fpic -Wno-gnu-variable-sized-type-not-at-end -Wno-unused-but-set-parameter"
export CC=clang

echo "=> Building elfutils"
run curl -L -O "https://sourceware.org/pub/elfutils/${ELFUTILS_VERSION}/elfutils-${ELFUTILS_VERSION}.tar.bz2"
if ! sha512sum "elfutils-${ELFUTILS_VERSION}.tar.bz2" | grep -q "$ELFUTILS_SHA_512"; then
    echo "Checksum for elfutils doesn't match"
    exit 1
fi

run tar xjf "elfutils-${ELFUTILS_VERSION}.tar.bz2"

run pushd "elfutils-${ELFUTILS_VERSION}"
run ./configure --prefix="${STATIC_LIBS_OUT_PATH}/elfutils" --disable-debuginfod --disable-libdebuginfod

run make "-j${NPROC}"
run make install
cp "${STATIC_LIBS_OUT_PATH}/elfutils/lib/libelf.a" "${STATIC_LIBS_OUT_PATH}"
run popd

echo "=> Building zlib"
run curl -L -O "https://zlib.net/zlib-${ZLIB_VERSION}.tar.gz"
if ! sha256sum "zlib-${ZLIB_VERSION}.tar.gz" | grep -q "$ZLIB_SHA256"; then
    echo "Checksum for zlib doesn't match"
    exit 1
fi
run tar xzf zlib-${ZLIB_VERSION}.tar.gz

run pushd "zlib-${ZLIB_VERSION}"
run ./configure --prefix="${STATIC_LIBS_OUT_PATH}/libz" >/dev/null
run make "-j${NPROC}" >/dev/null
run make install >/dev/null
cp "${STATIC_LIBS_OUT_PATH}/libz/lib/libz.a" "${STATIC_LIBS_OUT_PATH}"
run popd

run popd
