#if os(macOS)
import SwiftUI
import SceneKit

/// macOS World view using SceneKit for 3D rendering.
struct WorldViewMac: NSViewRepresentable {
    @ObservedObject var viewModel: WorldViewModelSceneKit

    func makeNSView(context: Context) -> SCNView {
        let scnView = SCNView()
        viewModel.setupSCNView(scnView)
        return scnView
    }

    func updateNSView(_ scnView: SCNView, context: Context) {
        // Updates handled through viewModel
    }
}

/// Container view for the macOS world generation feature.
struct WorldChatViewMac: View {
    @StateObject private var viewModel = WorldViewModelSceneKit()
    @State private var inputText = ""
    @State private var showingSceneInfo = false

    var body: some View {
        HSplitView {
            // 3D View (left side)
            WorldViewMac(viewModel: viewModel)
                .frame(minWidth: 400)

            // Chat Panel (right side)
            VStack(spacing: 0) {
                // Message area showing recent actions
                ScrollViewReader { proxy in
                    ScrollView {
                        LazyVStack(alignment: .leading, spacing: 8) {
                            ForEach(viewModel.entityNames, id: \.self) { name in
                                if let entity = viewModel.entities[name] {
                                    EntityRowView(entity: entity, entityId: name)
                                }
                            }
                        }
                        .padding()
                    }
                }

                Divider()

                // Input area
                HStack(spacing: 12) {
                    TextField("Describe what to create...", text: $inputText, axis: .vertical)
                        .padding(8)
                        .background(Color.secondary.opacity(0.1))
                        .cornerRadius(8)
                        .lineLimit(1...3)
                        .onSubmit {
                            processInput()
                        }

                    Button(action: processInput) {
                        Image(systemName: "arrow.up.circle.fill")
                            .font(.system(size: 24))
                            .foregroundColor(inputText.isEmpty ? .gray : .teal)
                    }
                    .disabled(inputText.isEmpty)
                    .keyboardShortcut(.return, modifiers: [])
                }
                .padding()
            }
            .frame(minWidth: 300, maxWidth: 400)
        }
        .navigationTitle("3D World")
        .toolbar {
            ToolbarItem(placement: .automatic) {
                Button(action: { showingSceneInfo = true }) {
                    Image(systemName: "info.circle")
                }
            }
            ToolbarItem(placement: .automatic) {
                Button(action: clearScene) {
                    Image(systemName: "trash")
                }
            }
            ToolbarItem(placement: .automatic) {
                Menu("Save/Load") {
                    Button("Save World") {
                        // TODO: Show save dialog
                    }
                    Button("Load World") {
                        // TODO: Show load dialog
                    }
                    Divider()
                    Button("List Saved Worlds") {
                        // TODO: Show worlds list
                    }
                }
            }
        }
        .alert("Scene Info", isPresented: $showingSceneInfo) {
            Button("OK", role: .cancel) { }
        } message: {
            Text(viewModel.sceneInfo())
        }
        .alert("Error", isPresented: $viewModel.showError) {
            Button("OK", role: .cancel) { }
        } message: {
            Text(viewModel.lastError ?? "Unknown error")
        }
    }

    private func processInput() {
        let text = inputText.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !text.isEmpty else { return }

        inputText = ""

        // Parse natural language commands
        parseAndExecute(text)
    }

    private func parseAndExecute(_ text: String) {
        let lowercased = text.lowercased()

        // Simple command parsing
        if lowercased.contains("cube") || lowercased.contains("box") {
            let color = extractColor(from: text)
            let pos = extractPosition(from: text)
            _ = viewModel.spawnPrimitive(shape: "cube", position: pos, color: color)
        } else if lowercased.contains("sphere") || lowercased.contains("ball") {
            let color = extractColor(from: text)
            let pos = extractPosition(from: text)
            _ = viewModel.spawnPrimitive(shape: "sphere", position: pos, color: color)
        } else if lowercased.contains("cylinder") {
            let color = extractColor(from: text)
            let pos = extractPosition(from: text)
            _ = viewModel.spawnPrimitive(shape: "cylinder", position: pos, color: color)
        } else if lowercased.contains("cone") {
            let color = extractColor(from: text)
            let pos = extractPosition(from: text)
            _ = viewModel.spawnPrimitive(shape: "cone", position: pos, color: color)
        } else if lowercased.contains("pyramid") {
            let color = extractColor(from: text)
            let pos = extractPosition(from: text)
            _ = viewModel.spawnPrimitive(shape: "pyramid", position: pos, color: color)
        } else if lowercased.contains("torus") || lowercased.contains("donut") || lowercased.contains("ring") {
            let color = extractColor(from: text)
            let pos = extractPosition(from: text)
            _ = viewModel.spawnPrimitive(shape: "torus", position: pos, color: color)
        } else if lowercased.contains("clear") || lowercased.contains("reset") {
            _ = viewModel.clearScene()
        } else if lowercased.contains("info") || lowercased.contains("list") {
            showingSceneInfo = true
        } else {
            // Default: create a cube
            let color = extractColor(from: text)
            _ = viewModel.spawnPrimitive(shape: "cube", position: [0, 0.5, 0], color: color)
        }
    }

    private func extractColor(from text: String) -> String {
        let lowercased = text.lowercased()
        if lowercased.contains("red") { return "red" }
        if lowercased.contains("blue") { return "blue" }
        if lowercased.contains("green") { return "green" }
        if lowercased.contains("yellow") { return "yellow" }
        if lowercased.contains("orange") { return "orange" }
        if lowercased.contains("purple") { return "purple" }
        if lowercased.contains("pink") { return "pink" }
        if lowercased.contains("white") { return "white" }
        if lowercased.contains("black") { return "black" }
        if lowercased.contains("cyan") || lowercased.contains("teal") { return "cyan" }
        return "gray"
    }

    private func extractPosition(from text: String) -> [Float] {
        // Default position at center, slightly elevated
        return [0, 0.5, 0]
    }

    private func clearScene() {
        _ = viewModel.clearScene()
    }
}

/// Row view displaying entity information.
struct EntityRowView: View {
    let entity: WorldEntity
    let entityId: String

    var body: some View {
        HStack(spacing: 8) {
            Image(systemName: "cube.box.fill")
                .foregroundColor(.teal)

            VStack(alignment: .leading, spacing: 2) {
                Text(entityId)
                    .font(.headline)
                Text(entity.shape?.rawValue ?? "primitive")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }

            Spacer()

            Text("[\(String(format: "%.1f", entity.position.x)), \(String(format: "%.1f", entity.position.y)), \(String(format: "%.1f", entity.position.z))]")
                .font(.caption)
                .foregroundColor(.secondary)
        }
        .padding(.vertical, 4)
    }
}

#Preview {
    WorldChatViewMac()
}
#endif
