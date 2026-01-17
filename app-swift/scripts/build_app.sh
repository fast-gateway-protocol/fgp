#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUILD_DIR="${ROOT_DIR}/build"

xcodebuild \
  -project "${ROOT_DIR}/FGPManager.xcodeproj" \
  -scheme FGPManager \
  -configuration Debug \
  -destination 'platform=macOS' \
  -derivedDataPath "${BUILD_DIR}" \
  build

APP_PATH="${BUILD_DIR}/Build/Products/Debug/FGP Manager.app"
if [ -d "${APP_PATH}" ]; then
  echo "Built app at ${APP_PATH}"
else
  echo "Build finished but app not found at ${APP_PATH}" >&2
  exit 1
fi
