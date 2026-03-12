import SwiftUI

/// Entity inspector view for editing entity properties
struct EntityInspectorView: View {
    @ObservedObject var viewModel: WorldViewModel
    let entityId: String
    @Environment(\.dismiss) var dismiss

    @State private var positionX: String = ""
    @State private var positionY: String = ""
    @State private var positionZ: String = ""
    @State private var scaleX: String = ""
    @State private var scaleY: String = ""
    @State private var scaleZ: String = ""
    @State private var rotationX: String = ""
    @State private var rotationY: String = ""
    @State private var rotationZ: String = ""
    @State private var selectedColor: Color = .gray
    @State private var behaviors: [BehaviorType] = []

    var body: some View {
        NavigationStack {
            Form {
                if let entity = viewModel.entities[entityId] {
                    // Entity Info Section
                    Section("Entity Info") {
                        HStack {
                            Text("Name")
                            Spacer()
                            Text(entity.name)
                                .foregroundColor(.secondary)
                        }
                        HStack {
                            Text("Type")
                            Spacer()
                            Text(entity.shape?.rawValue ?? "Model")
                                .foregroundColor(.secondary)
                        }
                    }

                    // Position Section
                    Section("Position") {
                        HStack {
                            Text("X")
                            TextField("0", text: $positionX)
                                .keyboardType(.decimalPad)
                                .multilineTextAlignment(.trailing)
                        }
                        HStack {
                            Text("Y")
                            TextField("0", text: $positionY)
                                .keyboardType(.decimalPad)
                                .multilineTextAlignment(.trailing)
                        }
                        HStack {
                            Text("Z")
                            TextField("0", text: $positionZ)
                                .keyboardType(.decimalPad)
                                .multilineTextAlignment(.trailing)
                        }
                    }

                    // Scale Section
                    Section("Scale") {
                        HStack {
                            Text("X")
                            TextField("1", text: $scaleX)
                                .keyboardType(.decimalPad)
                                .multilineTextAlignment(.trailing)
                        }
                        HStack {
                            Text("Y")
                            TextField("1", text: $scaleY)
                                .keyboardType(.decimalPad)
                                .multilineTextAlignment(.trailing)
                        }
                        HStack {
                            Text("Z")
                            TextField("1", text: $scaleZ)
                                .keyboardType(.decimalPad)
                                .multilineTextAlignment(.trailing)
                        }
                    }

                    // Rotation Section
                    Section("Rotation (degrees)") {
                        HStack {
                            Text("X")
                            TextField("0", text: $rotationX)
                                .keyboardType(.decimalPad)
                                .multilineTextAlignment(.trailing)
                        }
                        HStack {
                            Text("Y")
                            TextField("0", text: $rotationY)
                                .keyboardType(.decimalPad)
                                .multilineTextAlignment(.trailing)
                        }
                        HStack {
                            Text("Z")
                            TextField("0", text: $rotationZ)
                                .keyboardType(.decimalPad)
                                .multilineTextAlignment(.trailing)
                        }
                    }

                    // Color Section
                    Section("Color") {
                        ColorPicker("Color", selection: $selectedColor)
                    }

                    // Behaviors Section
                    Section("Behaviors") {
                        if behaviors.isEmpty {
                            Text("No behaviors")
                                .foregroundColor(.secondary)
                        } else {
                            ForEach(behaviors, id: \.self) { behavior in
                                HStack {
                                    Text(behavior.rawValue.capitalized)
                                    Spacer()
                                    Button(action: {
                                        _ = viewModel.removeBehavior(from: entityId, type: behavior)
                                        loadEntityData()
                                    }) {
                                        Image(systemName: "trash")
                                            .foregroundColor(.red)
                                    }
                                }
                            }
                        }

                        Menu("Add Behavior") {
                            ForEach(BehaviorType.allCases, id: \.self) { behavior in
                                if !behaviors.contains(behavior) {
                                    Button(action: {
                                        _ = viewModel.addBehavior(to: entityId, type: behavior, parameters: [:])
                                        loadEntityData()
                                    }) {
                                        Text(behavior.rawValue.capitalized)
                                    }
                                }
                            }
                        }
                    }

                    // Actions Section
                    Section {
                        Button(action: applyChanges) {
                            HStack {
                                Spacer()
                                Text("Apply Changes")
                                    .fontWeight(.semibold)
                                Spacer()
                            }
                        }

                        Button(role: .destructive, action: {
                            _ = viewModel.deleteEntity(entityId)
                            dismiss()
                        }) {
                            HStack {
                                Spacer()
                                Text("Delete Entity")
                                Spacer()
                            }
                        }
                    }
                } else {
                    Text("Entity not found")
                        .foregroundColor(.secondary)
                }
            }
            .navigationTitle("Edit \(entityId)")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .navigationBarLeading) {
                    Button("Cancel") {
                        dismiss()
                    }
                }
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button("Done") {
                        applyChanges()
                        dismiss()
                    }
                }
            }
            .onAppear {
                loadEntityData()
            }
        }
        .presentationDetents([.large])
    }

    private func loadEntityData() {
        guard let entity = viewModel.entities[entityId] else { return }

        positionX = String(format: "%.2f", entity.position.x)
        positionY = String(format: "%.2f", entity.position.y)
        positionZ = String(format: "%.2f", entity.position.z)

        scaleX = String(format: "%.2f", entity.scale.x)
        scaleY = String(format: "%.2f", entity.scale.y)
        scaleZ = String(format: "%.2f", entity.scale.z)

        rotationX = String(format: "%.1f", entity.rotation.x)
        rotationY = String(format: "%.1f", entity.rotation.y)
        rotationZ = String(format: "%.1f", entity.rotation.z)

        selectedColor = entity.color.toColor()
        behaviors = entity.behaviors.map { $0.type }
    }

    private func applyChanges() {
        let position: [Float]? = [
            Float(positionX) ?? 0,
            Float(positionY) ?? 0,
            Float(positionZ) ?? 0
        ]

        let scale: [Float]? = [
            Float(scaleX) ?? 1,
            Float(scaleY) ?? 1,
            Float(scaleZ) ?? 1
        ]

        let rotation: [Float]? = [
            Float(rotationX) ?? 0,
            Float(rotationY) ?? 0,
            Float(rotationZ) ?? 0
        ]

        let color = colorToName(selectedColor)

        _ = viewModel.modifyEntity(
            entityId: entityId,
            position: position,
            scale: scale,
            rotation: rotation,
            color: color
        )
    }

    private func colorToName(_ color: Color) -> String {
        // Convert SwiftUI Color to name for the modify API
        let components = UIColor(color).rgba
        if components.red > 0.9 && components.green < 0.1 && components.blue < 0.1 {
            return "red"
        } else if components.red < 0.1 && components.green > 0.9 && components.blue < 0.1 {
            return "green"
        } else if components.red < 0.1 && components.green < 0.1 && components.blue > 0.9 {
            return "blue"
        } else if components.red > 0.9 && components.green > 0.9 && components.blue < 0.1 {
            return "yellow"
        } else if components.red < 0.1 && components.green > 0.9 && components.blue > 0.9 {
            return "cyan"
        } else if components.red > 0.9 && components.green < 0.1 && components.blue > 0.9 {
            return "magenta"
        } else if components.red > 0.9 && components.green > 0.5 && components.blue < 0.1 {
            return "orange"
        } else if components.red > 0.9 && components.green > 0.7 && components.blue > 0.7 {
            return "pink"
        } else if components.red > 0.9 && components.green > 0.9 && components.blue > 0.9 {
            return "white"
        } else if components.red < 0.1 && components.green < 0.1 && components.blue < 0.1 {
            return "black"
        } else {
            return "gray"
        }
    }
}

// UIColor extension to get RGBA components
extension UIColor {
    var rgba: (red: CGFloat, green: CGFloat, blue: CGFloat, alpha: CGFloat) {
        var red: CGFloat = 0
        var green: CGFloat = 0
        var blue: CGFloat = 0
        var alpha: CGFloat = 0
        getRed(&red, green: &green, blue: &blue, alpha: &alpha)
        return (red, green, blue, alpha)
    }
}

#Preview {
    EntityInspectorView(viewModel: WorldViewModel(), entityId: "test_cube")
}
