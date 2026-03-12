# iPad World Generation

This directory contains the RealityKit-based 3D world generation feature for the iPad app. Users can chat to create virtual worlds with Apple's native 3D and audio stack.

## Features

### Core 3D
- **Primitives**: cube, sphere, cylinder, cone, plane, capsule, pyramid, torus
- **Transforms**: position, scale, rotation, color
- **Behaviors**: orbit, spin, bob, pulse, bounce (real-time at60fps)
- **Camera**: orbit gestures (pinch zoom, drag orbit)
- **Audio**: ambient sounds, spatial emitters

### World Management
- **Save/Load**: JSON serialization to Documents/Worlds/
- **Scene info**: Entity listing with transforms

### LLM Integration
- **Apple Foundation Models** (on-device, iOS 18.1+)
- **Cloud APIs** via Rust FFI
- **Slash commands**: `/spawn`, `/camera`, `/behavior`, `/ambience`, `/help`

### visionOS Support
- ImmersiveSpace for Vision Pro worlds

## Usage

1. Open the app and select the "World" tab
2. Type commands like "create a red cube" or use slash commands like `/spawn sphere blue`
3. Save worlds with `/save my-world`
4. Load worlds from the saved list

## Architecture

```
WorldViewModel (manages RealityKit scene)
├── WorldToolService (provides LLM tool definitions)
├── WorldChatViewModel (routes tool calls to scene)
├── WorldAudioService (spatial audio)
└── WorldBehaviorService (animation parameters)
```

## Files Created

- `Models/WorldEntity.swift` - Entity data model
- `Models/WorldState.swift` - Scene state
- `ViewModels/WorldViewModel.swift` - Scene management
- `ViewModels/WorldChatViewModel.swift` - Chat integration
- `Views/WorldView.swift` - RealityKit view
- `Views/WorldChatView.swift` - Combined view
- `Services/WorldToolService.swift` - Tool definitions
- `Services/WorldAudioService.swift` - Audio service
- `Services/WorldBehaviorService.swift` - Behavior parameters
- `Extensions/RealityKit+Extensions.swift` - Helper methods

## Future Enhancements

- Add bundled audio files for ambient sounds
- Implement touch entity selection
- Add material editor (metallic, roughness)
- Add particle effects
