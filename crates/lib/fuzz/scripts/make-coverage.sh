#!/usr/bin/env bash

set -euo pipefail

TARGETS="$(cargo fuzz list)"

TARGET="${1:-}"

if ! grep -qx "${TARGET}" <<< "${TARGETS}"; then
    echo "Usage: $0 <fuzz_target>"
    [ -z "${TARGET}" ] || echo "Target not found: ${TARGET}"
    echo
    echo "Available targets:"
    echo "${TARGETS}"
    exit 1
fi

DEMANGLER="rustfilt"

if which ${DEMANGLER}; then
    DEMANGLER_OPTION="--Xdemangler=${DEMANGLER}"
else
    echo "Demangler 'rustfilt' not found. Generating coverage without it."
    echo "Install with 'cargo install rustfilt'"
    DEMANGLER_OPTION=""
fi

RUSTROOT="$(rustc --print sysroot)"
HOST="$(rustc -vV | sed -n 's/^host: //p')"

LLVM_COV="${RUSTROOT}/lib/rustlib/${HOST}/bin/llvm-cov"

if ![ -f ${LLVM_COV} ]; then
    echo "llvm-cov not found. Have you installed the 'llvm-tools-preview' component with rustup?"
    echo "Looked in ${LLVM_COV}"
    exit 1
fi

echo "Running cargo fuzz cov"
cargo fuzz cov "${TARGET}"

echo "Running llvm-cov"
${LLVM_COV} show \
    "target/${HOST}/coverage/${HOST}/release/${TARGET}" \
    --instr-profile=./coverage/"${TARGET}"/coverage.profdata \
    --sources="$(find .. -name "*.rs")" \
    --format=html \
    --output-dir="./coverage/${TARGET}/html" \
    --show-line-counts-or-regions "${DEMANGLER_OPTION}"
