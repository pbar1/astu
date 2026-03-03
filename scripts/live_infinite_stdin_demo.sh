#!/usr/bin/env bash
set -euo pipefail

TMP_DIR="${ASTU_DATA_DIR:-/tmp/astu-live-flex}"
FIFO_PATH="${TMP_DIR}/producer.fifo"

mkdir -p "${TMP_DIR}"
rm -f "${FIFO_PATH}"
mkfifo "${FIFO_PATH}"

cleanup() {
  if [[ -n "${PROD_PID:-}" ]]; then
    kill "${PROD_PID}" 2>/dev/null || true
    wait "${PROD_PID}" 2>/dev/null || true
  fi
  rm -f "${FIFO_PATH}"
}
trap cleanup EXIT INT TERM

(
  i=1
  while :; do
    printf 'line-%08d\n' "${i}"
    i=$((i + 1))
  done > "${FIFO_PATH}"
) &
PROD_PID=$!

echo "ASTU_DATA_DIR=${TMP_DIR}"
echo "Producer PID=${PROD_PID}"
echo "Press Ctrl-C once for graceful stop, twice to force exit."

ASTU_DATA_DIR="${TMP_DIR}" ASTU_NO_PAGER=1 target/release/astu run \
  -T local://alpha -T local://beta -T local://gamma \
  --confirm=3 --concurrency=3 --live \
  'while IFS= read -r line; do echo {host}:$line; sleep 0.003; done' < "${FIFO_PATH}"

echo
echo "Post-run checks:"
ASTU_DATA_DIR="${TMP_DIR}" astu jobs --output=json
ASTU_DATA_DIR="${TMP_DIR}" astu freq stdout --output=json
ASTU_DATA_DIR="${TMP_DIR}" astu freq error --output=json
ls -lah "${TMP_DIR}/spool"
