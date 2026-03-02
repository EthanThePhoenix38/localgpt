#!/bin/bash
set -e

# Get the directory where the script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
# Root of the repo (one level deeper now: apps/ios/scripts -> apps/ios -> apps -> root)
ROOT_DIR="$SCRIPT_DIR/../../../"

# Configuration
CRATE_NAME="localgpt_mobile"
RELEASE_MODE="--release"
TARGET_DIR="$ROOT_DIR/target"
IOS_WRAPPER_DIR="$SCRIPT_DIR/../LocalGPTWrapper"
LIB_NAME="lib$CRATE_NAME.a"

# Targets (Apple Silicon Host)
DEVICE_TARGET="aarch64-apple-ios"
SIM_TARGET="aarch64-apple-ios-sim"

echo "Building for iOS Device ($DEVICE_TARGET)..."
cargo build -p localgpt-mobile-ffi --lib --target $DEVICE_TARGET $RELEASE_MODE

echo "Building for iOS Simulator ($SIM_TARGET)..."
cargo build -p localgpt-mobile-ffi --lib --target $SIM_TARGET $RELEASE_MODE

# Output directories
mkdir -p "$IOS_WRAPPER_DIR/Sources/LocalGPTWrapper/include"
mkdir -p "$IOS_WRAPPER_DIR/Sources/LocalGPTWrapper/Resources"

# Generate Swift Bindings & C Header
echo "Generating UniFFI Bindings..."
LIBRARY_PATH="$TARGET_DIR/$DEVICE_TARGET/release/$LIB_NAME"

cargo run --bin uniffi-bindgen -p localgpt-mobile-ffi -- generate \
    --library "$LIBRARY_PATH" \
    --language swift \
    --out-dir "$IOS_WRAPPER_DIR/Sources/LocalGPTWrapper"

# Modern UniFFI generates <crate>FFI.h and <crate>FFI.modulemap
FFI_HEADER="${CRATE_NAME}FFI.h"
FFI_MODULEMAP="${CRATE_NAME}FFI.modulemap"

# Move generated header to include/
if [ -f "$IOS_WRAPPER_DIR/Sources/LocalGPTWrapper/$FFI_HEADER" ]; then
    mv "$IOS_WRAPPER_DIR/Sources/LocalGPTWrapper/$FFI_HEADER" "$IOS_WRAPPER_DIR/Sources/LocalGPTWrapper/include/"
fi

# Modern UniFFI modulemap is already good, but let's make sure it's in include/
if [ -f "$IOS_WRAPPER_DIR/Sources/LocalGPTWrapper/$FFI_MODULEMAP" ]; then
    mv "$IOS_WRAPPER_DIR/Sources/LocalGPTWrapper/$FFI_MODULEMAP" "$IOS_WRAPPER_DIR/Sources/LocalGPTWrapper/include/module.modulemap"
fi

# Create XCFramework
echo "Creating XCFramework..."
rm -rf "$IOS_WRAPPER_DIR/LocalGPTCore.xcframework"

xcodebuild -create-xcframework \
    -library "$TARGET_DIR/$DEVICE_TARGET/release/$LIB_NAME" \
    -headers "$IOS_WRAPPER_DIR/Sources/LocalGPTWrapper/include" \
    -library "$TARGET_DIR/$SIM_TARGET/release/$LIB_NAME" \
    -headers "$IOS_WRAPPER_DIR/Sources/LocalGPTWrapper/include" \
    -output "$IOS_WRAPPER_DIR/LocalGPTCore.xcframework"

echo "Build complete! Wrapper updated at $IOS_WRAPPER_DIR"
