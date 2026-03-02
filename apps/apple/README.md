# LocalGPT iOS App

This directory contains the iOS integration for LocalGPT using Rust, UniFFI, and SwiftUI.

## Project Structure

```
apps/ios/
├── LocalGPT.xcodeproj/      # Xcode project
├── LocalGPT/                # Main app target
│   ├── LocalGPTApp.swift    # App entry point
│   ├── Views/
│   │   ├── ChatView.swift   # Main chat interface
│   │   └── Components/
│   │       ├── MessageBubble.swift
│   │       └── ThinkingIndicator.swift
│   ├── ViewModels/
│   │   └── ChatViewModel.swift
│   ├── Models/
│   │   └── Message.swift
│   └── Assets.xcassets/
├── LocalGPTTests/           # Unit tests
├── LocalGPTUITests/         # UI tests
├── LocalGPTWrapper/         # Swift Package (Rust XCFramework + bindings)
└── scripts/
    └── build_ios.sh         # Build script for Rust core
```

## Getting Started

### 1. Build the Rust Library

Ensure you have the iOS targets installed:
```bash
rustup target add aarch64-apple-ios aarch64-apple-ios-sim
```

Run the build script from the repository root:
```bash
bash apps/ios/scripts/build_ios.sh
```

This will create `LocalGPTWrapper/LocalGPTCore.xcframework`.

### 2. Open in Xcode

Open `LocalGPT.xcodeproj` in Xcode. The project is already configured with:
- LocalGPTWrapper package dependency
- Multi-platform support (iOS, macOS, visionOS)
- SwiftUI lifecycle

### 3. Build and Run

Select a target (iOS Simulator or your device) and press ⌘R to build and run.

## Features

- **Local-first**: Agent logic runs entirely on-device
- **Async**: UI remains responsive while the Rust core is thinking
- **UniFFI**: Modern type-safe bindings between Swift and Rust
- **XCFramework**: Easy distribution and integration into Xcode

## Architecture

The app follows the MVVM pattern:
- **Model**: `Message.swift` defines the message data structure
- **ViewModel**: `ChatViewModel.swift` manages state and communicates with the Rust core
- **View**: `ChatView.swift` and its components render the chat interface
