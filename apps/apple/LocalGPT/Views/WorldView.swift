import SwiftUI
import RealityKit
import ARKit

/// RealityKit view for 3D world generation
struct WorldView: View {
    @StateObject private var viewModel = WorldViewModel()
    @State private var showEntityList = false
    @State private var showSaveSheet = false
    @State private var showLoadSheet = false
    @State private var worldName = ""

    var body: some View {
        NavigationStack {
            ZStack {
                // RealityKit Scene
                RealityKitView(viewModel: viewModel)
                    .edgesIgnoringSafeArea(.all)

                // Overlay Controls
                VStack {
                    // Camera control hints
                    HStack {
                        Spacer()
                        VStack(alignment: .trailing, spacing: 4) {
                            Text("Pinch to zoom")
                            Text("Drag to orbit")
                        }
                        .font(.caption2)
                        .foregroundColor(.secondary)
                        .padding(8)
                        .background(.ultraThinMaterial)
                        .cornerRadius(8)
                    }
                    .padding()

                    Spacer()

                    // Bottom toolbar
                    HStack(spacing: 20) {
                        // Entity list button
                        Button(action: { showEntityList = true }) {
                            VStack(spacing: 4) {
                                Image(systemName: "list.bullet")
                                Text("\(viewModel.entities.count)")
                                    .font(.caption2)
                            }
                            .padding()
                            .background(.ultraThinMaterial)
                            .cornerRadius(12)
                        }

                        Spacer()

                        // Camera reset button
                        Button(action: {
                            _ = viewModel.setCamera(position: [0, 5, 10], lookAt: [0, 0, 0])
                        }) {
                            Image(systemName: "viewfinder")
                                .padding()
                                .background(.ultraThinMaterial)
                                .cornerRadius(12)
                        }

                        // Clear button
                        Button(action: {
                            _ = viewModel.clearScene()
                        }) {
                            Image(systemName: "trash")
                                .padding()
                                .background(.ultraThinMaterial)
                                .cornerRadius(12)
                        }
                        .disabled(viewModel.entities.isEmpty)
                    }
                    .padding()
                }

                // Processing indicator
                if viewModel.isProcessing {
                    ProgressView("Processing...")
                        .padding()
                        .background(.ultraThinMaterial)
                        .cornerRadius(12)
                }
            }
            .navigationTitle("World")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .navigationBarLeading) {
                    Button(action: { showLoadSheet = true }) {
                        Image(systemName: "folder")
                    }
                }

                ToolbarItem(placement: .navigationBarTrailing) {
                    Button(action: { showSaveSheet = true }) {
                        Image(systemName: "square.and.arrow.down")
                    }
                    .disabled(viewModel.entities.isEmpty)
                }
            }
            .sheet(isPresented: $showEntityList) {
                EntityListView(viewModel: viewModel)
            }
            .sheet(isPresented: $showSaveSheet) {
                SaveWorldSheet(viewModel: viewModel, worldName: $worldName)
            }
            .sheet(isPresented: $showLoadSheet) {
                LoadWorldSheet(viewModel: viewModel)
            }
            .alert("Error", isPresented: $viewModel.showError) {
                Button("OK", role: .cancel) { }
            } message: {
                Text(viewModel.lastError ?? "Unknown error")
            }
        }
    }
}

/// UIViewRepresentable wrapper for ARView with gesture support
struct RealityKitView: UIViewRepresentable {
    @ObservedObject var viewModel: WorldViewModel

    func makeUIView(context: Context) -> ARView {
        let arView = ARView(frame: .zero)

        // Configure for virtual world (non-AR mode)
        let config = ARWorldTrackingConfiguration()
        config.worldAlignment = .gravity
        arView.session.run(config, options: [.resetTracking, .removeExistingAnchors])

        // Enable camera controls for non-AR mode
        arView.environment.sceneUnderstanding.options = []

        // Add gesture recognizers
        addGestures(to: arView, context: context)

        // Setup scene
        viewModel.setupARView(arView)

        return arView
    }

    func updateUIView(_ uiView: ARView, context: Context) {
        // Updates handled through viewModel bindings
    }

    private func addGestures(to arView: ARView, context: Context) {
        // Pinch gesture for zoom
        let pinchGesture = UIPinchGestureRecognizer(target: context.coordinator, action: #selector(Coordinator.handlePinch(_:)))
        arView.addGestureRecognizer(pinchGesture)

        // Pan gesture for orbit
        let panGesture = UIPanGestureRecognizer(target: context.coordinator, action: #selector(Coordinator.handlePan(_:)))
        arView.addGestureRecognizer(panGesture)

        // Rotation gesture
        let rotationGesture = UIRotationGestureRecognizer(target: context.coordinator, action: #selector(Coordinator.handleRotation(_:)))
        arView.addGestureRecognizer(rotationGesture)
    }

    func makeCoordinator() -> Coordinator {
        Coordinator(viewModel: viewModel)
    }

    class Coordinator: NSObject {
        weak var viewModel: WorldViewModel?
        private var lastPanLocation: CGPoint = .zero
        private var orbitAngle: Float = 0
        private var orbitElevation: Float = 0.5
        private var orbitRadius: Float = 10

        init(viewModel: WorldViewModel) {
            self.viewModel = viewModel
        }

        @objc func handlePinch(_ gesture: UIPinchGestureRecognizer) {
            guard let viewModel = viewModel else { return }

            switch gesture.state {
            case .changed:
                let scale = Float(gesture.scale)
                orbitRadius = max(2, min(50, orbitRadius / scale))
                updateCameraPosition()
                gesture.scale = 1.0
            default:
                break
            }
        }

        @objc func handlePan(_ gesture: UIPanGestureRecognizer) {
            guard let viewModel = viewModel else { return }

            let translation = gesture.translation(in: gesture.view)

            switch gesture.state {
            case .changed:
                let sensitivity: Float = 0.01
                orbitAngle -= Float(translation.x) * sensitivity
                orbitElevation = max(-1.5, min(1.5, orbitElevation - Float(translation.y) * sensitivity))
                updateCameraPosition()
                gesture.setTranslation(.zero, in: gesture.view)
            default:
                break
            }
        }

        @objc func handleRotation(_ gesture: UIRotationGestureRecognizer) {
            guard let viewModel = viewModel else { return }

            switch gesture.state {
            case .changed:
                orbitAngle -= Float(gesture.rotation)
                updateCameraPosition()
                gesture.rotation = 0
            default:
                break
            }
        }

        private func updateCameraPosition() {
            guard let viewModel = viewModel else { return }

            let x = orbitRadius * cos(orbitElevation) * sin(orbitAngle)
            let y = orbitRadius * sin(orbitElevation)
            let z = orbitRadius * cos(orbitElevation) * cos(orbitAngle)

            _ = viewModel.setCamera(position: [x, y + 2, z], lookAt: [0, 0, 0])
        }
    }
}

/// Entity list view
struct EntityListView: View {
    @ObservedObject var viewModel: WorldViewModel
    @Environment(\.dismiss) var dismiss

    var body: some View {
        NavigationStack {
            List {
                if viewModel.entityNames.isEmpty {
                    Text("No entities in scene")
                        .foregroundColor(.secondary)
                } else {
                    ForEach(viewModel.entityNames, id: \.self) { name in
                        if let entity = viewModel.entities[name] {
                            HStack {
                                VStack(alignment: .leading) {
                                    Text(name)
                                        .font(.headline)
                                    Text("\(entity.shape?.rawValue ?? "model") at [\(String(format: "%.1f", entity.position.x)), \(String(format: "%.1f", entity.position.y)), \(String(format: "%.1f", entity.position.z))]")
                                        .font(.caption)
                                        .foregroundColor(.secondary)
                                }
                                Spacer()
                            }
                            .swipeActions(edge: .trailing, allowsFullSwipe: true) {
                                Button(role: .destructive) {
                                    _ = viewModel.deleteEntity(name)
                                } label: {
                                    Label("Delete", systemImage: "trash")
                                }
                            }
                        }
                    }
                }
            }
            .navigationTitle("Entities")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button("Done") {
                        dismiss()
                    }
                }
            }
        }
        .presentationDetents([.medium, .large])
    }
}

/// Save world sheet
struct SaveWorldSheet: View {
    @ObservedObject var viewModel: WorldViewModel
    @Binding var worldName: String
    @Environment(\.dismiss) var dismiss
    @State private var saveResult: String?

    var body: some View {
        NavigationStack {
            Form {
                Section("World Name") {
                    TextField("Enter world name", text: $worldName)
                        .textContentType(.name)
                }

                if let result = saveResult {
                    Section("Result") {
                        Text(result)
                            .foregroundColor(result.contains("Error") ? .red : .green)
                    }
                }
            }
            .navigationTitle("Save World")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .navigationBarLeading) {
                    Button("Cancel") {
                        dismiss()
                    }
                }
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button("Save") {
                        if !worldName.isEmpty {
                            saveResult = viewModel.saveWorld(name: worldName)
                            if saveResult?.contains("successfully") == true {
                                DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) {
                                    worldName = ""
                                    dismiss()
                                }
                            }
                        }
                    }
                    .disabled(worldName.isEmpty)
                }
            }
        }
        .presentationDetents([.height(250)])
    }
}

/// Load world sheet
struct LoadWorldSheet: View {
    @ObservedObject var viewModel: WorldViewModel
    @Environment(\.dismiss) var dismiss
    @State private var savedWorlds: [String] = []
    @State private var isLoading = false

    var body: some View {
        NavigationStack {
            Group {
                if isLoading {
                    ProgressView("Loading...")
                } else if savedWorlds.isEmpty {
                    VStack(spacing: 16) {
                        Image(systemName: "folder.badge.questionmark")
                            .font(.system(size: 48))
                            .foregroundColor(.secondary)
                        Text("No saved worlds")
                            .font(.headline)
                        Text("Create a world and save it to see it here")
                            .font(.caption)
                            .foregroundColor(.secondary)
                            .multilineTextAlignment(.center)
                    }
                    .padding()
                } else {
                    List(savedWorlds, id: \.self) { name in
                        Button(action: {
                            _ = viewModel.loadWorld(name: name, clearCurrent: true)
                            dismiss()
                        }) {
                            HStack {
                                Image(systemName: "globe")
                                    .foregroundColor(.teal)
                                Text(name)
                                    .foregroundColor(.primary)
                                Spacer()
                                Image(systemName: "chevron.right")
                                    .foregroundColor(.secondary)
                            }
                        }
                        .swipeActions(edge: .trailing, allowsFullSwipe: true) {
                            Button(role: .destructive) {
                                try? WorldState.delete(name: name)
                                loadWorlds()
                            } label: {
                                Label("Delete", systemImage: "trash")
                            }
                        }
                    }
                }
            }
            .navigationTitle("Load World")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button("Cancel") {
                        dismiss()
                    }
                }
            }
        }
        .presentationDetents([.medium, .large])
        .onAppear {
            loadWorlds()
        }
    }

    private func loadWorlds() {
        savedWorlds = (try? WorldState.listAll()) ?? []
    }
}

#Preview {
    WorldView()
}
