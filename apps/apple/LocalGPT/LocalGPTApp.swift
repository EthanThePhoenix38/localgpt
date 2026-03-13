import SwiftUI
import LocalGPTWrapper

@main
struct LocalGPTApp: App {
    init() {
        #if DEBUG
        // Suppress UIKit's internal Auto Layout constraint warnings on iPad
        // These are known Apple bugs in _UIRemoteKeyboardPlaceholderView
        UserDefaults.standard.set(false, forKey: "_UIConstraintBasedLayoutLogUnsatisfiable")
        #endif
    }

    var body: some Scene {
        WindowGroup {
            TabView {
                ChatView()
                    .tabItem {
                        Label("Chat", systemImage: "message.fill")
                    }

                #if os(iOS) || os(visionOS)
                WorldChatView()
                    .tabItem {
                        Label("World", systemImage: "cube.transparent")
                    }
                #elseif os(macOS)
                // macOS uses SceneKit for 3D world generation
                WorldChatViewMac()
                    .tabItem {
                        Label("World", systemImage: "cube.transparent")
                    }
                #endif

                WorkspaceEditorView()
                    .tabItem {
                        Label("Workspace", systemImage: "doc.text.fill")
                    }
            }
            .tint(.teal)
        }

        #if os(visionOS)
        // Vision Pro immersive world space
        ImmersiveSpace(id: "world") {
            WorldImmersiveView()
        }
        #endif
    }
}

#if os(visionOS)
/// Immersive space view for Vision Pro
struct WorldImmersiveView: View {
    @StateObject private var viewModel = WorldViewModel()

    var body: some View {
        RealityView { content in
            // Add root entity
            content.add(viewModel.rootEntity)

            // Add default lighting
            let light = DirectionalLight()
            light.light.intensity = 2000
            light.position = [5, 10, 5]
            viewModel.rootEntity.addChild(light)
        } update: { content in
            // Updates handled through viewModel
        }
        .onAppear {
            // Configure for immersive mode
        }
    }
}
#endif
