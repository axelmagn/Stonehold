#!/bin/bash
# BROKEN
set -exuo pipefail

WASM_SERVER_RUNNER_CUSTOM_INDEX_HTML="index-wsr.html" wasm-server-runner "$@"