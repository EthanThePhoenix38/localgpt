/// Entity detail panel — shows all component data for the selected entity.

import SwiftUI

/// Displays comprehensive entity data in collapsible sections.
public struct DetailView: View {
    let client: InspectorClient

    public init(client: InspectorClient) {
        self.client = client
    }

    public var body: some View {
        Group {
            if let detail = client.selectedEntityDetail {
                ScrollView {
                    VStack(alignment: .leading, spacing: 12) {
                        identitySection(detail.identity)
                        transformSection(detail.transform, live: client.selectedTransform)
                        shapeSection(detail.shape)
                        materialSection(detail.material)
                        lightSection(detail.light)
                        behaviorsSection(detail.behaviors)
                        audioSection(detail.audio)
                        meshSection(detail.meshAsset)
                        hierarchySection(detail.hierarchy)
                    }
                    .padding()
                }
            } else if client.selectedEntityId != nil {
                VStack {
                    ProgressView()
                    Text("Loading...")
                        .foregroundStyle(.secondary)
                }
            } else {
                VStack {
                    Image(systemName: "cube.transparent")
                        .font(.largeTitle)
                        .foregroundStyle(.tertiary)
                    Text("Select an entity to inspect")
                        .foregroundStyle(.secondary)
                }
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    // MARK: - Sections

    @ViewBuilder
    private func identitySection(_ identity: IdentitySection) -> some View {
        SectionCard(title: "Identity", systemImage: "info.circle") {
            PropertyRow("Name", identity.name)
            PropertyRow("Type", identity.entityType)
            PropertyRow("ID", "\(identity.id)")
        }
    }

    @ViewBuilder
    private func transformSection(_ transform: TransformSection?, live: (position: SIMD3<Float>, rotation: SIMD3<Float>)?) -> some View {
        if let t = transform {
            let pos = live?.position ?? SIMD3(t.position[0], t.position[1], t.position[2])
            let rot = live?.rotation ?? SIMD3(t.rotationDegrees[0], t.rotationDegrees[1], t.rotationDegrees[2])

            SectionCard(title: "Transform", systemImage: "move.3d") {
                PropertyRow("Position", formatVec3(pos))
                PropertyRow("Rotation", formatVec3(rot))
                PropertyRow("Scale", formatArray3(t.scale))
                PropertyRow("Visible", t.visible ? "Yes" : "No")
            }
        }
    }

    @ViewBuilder
    private func shapeSection(_ shape: String?) -> some View {
        if let shape {
            SectionCard(title: "Shape", systemImage: "cube") {
                PropertyRow("Variant", shape)
            }
        }
    }

    @ViewBuilder
    private func materialSection(_ material: MaterialSection?) -> some View {
        if let mat = material {
            SectionCard(title: "Material", systemImage: "paintpalette") {
                HStack {
                    Text("Base Color")
                        .foregroundStyle(.secondary)
                    Spacer()
                    colorSwatch(mat.baseColor)
                    Text(formatArray4(mat.baseColor))
                }
                PropertyRow("Metallic", String(format: "%.3f", mat.metallic))
                PropertyRow("Roughness", String(format: "%.3f", mat.roughness))
                PropertyRow("Reflectance", String(format: "%.3f", mat.reflectance))
                PropertyRow("Alpha Mode", mat.alphaMode)
                PropertyRow("Double Sided", mat.doubleSided ? "Yes" : "No")
                PropertyRow("Unlit", mat.unlit ? "Yes" : "No")
            }
        }
    }

    @ViewBuilder
    private func lightSection(_ light: LightSection?) -> some View {
        if let light {
            SectionCard(title: "Light (\(light.lightType))", systemImage: "sun.max") {
                HStack {
                    Text("Color")
                        .foregroundStyle(.secondary)
                    Spacer()
                    colorSwatch(light.color + [1.0])
                    Text(formatArray3(light.color))
                }
                PropertyRow("Intensity", String(format: "%.1f", light.intensity))
                if let range = light.range {
                    PropertyRow("Range", String(format: "%.2f", range))
                }
                PropertyRow("Shadows", light.shadowsEnabled ? "Yes" : "No")
                if let inner = light.innerAngle {
                    PropertyRow("Inner Angle", String(format: "%.1f\u{00B0}", inner * 180 / .pi))
                }
                if let outer = light.outerAngle {
                    PropertyRow("Outer Angle", String(format: "%.1f\u{00B0}", outer * 180 / .pi))
                }
            }
        }
    }

    @ViewBuilder
    private func behaviorsSection(_ behaviors: [BehaviorSection]) -> some View {
        if !behaviors.isEmpty {
            SectionCard(title: "Behaviors (\(behaviors.count))", systemImage: "arrow.triangle.2.circlepath") {
                ForEach(behaviors) { beh in
                    VStack(alignment: .leading, spacing: 2) {
                        Text("\(beh.id): \(beh.behaviorType)")
                            .font(.caption.monospaced())
                        Text("Base: \(formatArray3(beh.basePosition))")
                            .font(.caption2)
                            .foregroundStyle(.secondary)
                    }
                    .padding(.vertical, 2)
                }
            }
        }
    }

    @ViewBuilder
    private func audioSection(_ audio: AudioSection?) -> some View {
        if let audio {
            SectionCard(title: "Audio", systemImage: "speaker.wave.2") {
                PropertyRow("Sound", audio.soundType)
                PropertyRow("Volume", String(format: "%.2f", audio.volume))
                PropertyRow("Radius", String(format: "%.1fm", audio.radius))
                if let attached = audio.attachedTo {
                    PropertyRow("Attached To", attached)
                }
            }
        }
    }

    @ViewBuilder
    private func meshSection(_ path: String?) -> some View {
        if let path {
            SectionCard(title: "Mesh Asset", systemImage: "doc.richtext") {
                PropertyRow("Path", path)
            }
        }
    }

    @ViewBuilder
    private func hierarchySection(_ hierarchy: HierarchySection) -> some View {
        if hierarchy.parent != nil || !hierarchy.children.isEmpty {
            SectionCard(title: "Hierarchy", systemImage: "list.bullet.indent") {
                if let parent = hierarchy.parent {
                    PropertyRow("Parent", parent)
                }
                if !hierarchy.children.isEmpty {
                    VStack(alignment: .leading, spacing: 2) {
                        Text("Children")
                            .foregroundStyle(.secondary)
                        ForEach(hierarchy.children, id: \.self) { child in
                            Text(child)
                                .font(.caption.monospaced())
                        }
                    }
                }
            }
        }
    }

    // MARK: - Formatting Helpers

    private func formatVec3(_ v: SIMD3<Float>) -> String {
        String(format: "[%.3f, %.3f, %.3f]", v.x, v.y, v.z)
    }

    private func formatArray3(_ a: [Float]) -> String {
        guard a.count >= 3 else { return "[]" }
        return String(format: "[%.3f, %.3f, %.3f]", a[0], a[1], a[2])
    }

    private func formatArray4(_ a: [Float]) -> String {
        guard a.count >= 4 else { return "[]" }
        return String(format: "[%.2f, %.2f, %.2f, %.2f]", a[0], a[1], a[2], a[3])
    }

    @ViewBuilder
    private func colorSwatch(_ rgba: [Float]) -> some View {
        let r = Double(rgba.count > 0 ? rgba[0].clamped(to: 0...1) : 0)
        let g = Double(rgba.count > 1 ? rgba[1].clamped(to: 0...1) : 0)
        let b = Double(rgba.count > 2 ? rgba[2].clamped(to: 0...1) : 0)
        RoundedRectangle(cornerRadius: 3)
            .fill(Color(red: r, green: g, blue: b))
            .frame(width: 14, height: 14)
            .overlay(
                RoundedRectangle(cornerRadius: 3)
                    .strokeBorder(.secondary.opacity(0.3), lineWidth: 0.5)
            )
    }
}

// MARK: - Reusable Components

/// A collapsible card section with title and icon.
struct SectionCard<Content: View>: View {
    let title: String
    let systemImage: String
    @ViewBuilder let content: () -> Content
    @State private var isExpanded = true

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Button {
                withAnimation(.easeInOut(duration: 0.2)) {
                    isExpanded.toggle()
                }
            } label: {
                HStack {
                    Image(systemName: systemImage)
                        .foregroundStyle(.blue)
                    Text(title)
                        .font(.headline)
                    Spacer()
                    Image(systemName: isExpanded ? "chevron.down" : "chevron.right")
                        .foregroundStyle(.secondary)
                        .font(.caption)
                }
            }
            .buttonStyle(.plain)

            if isExpanded {
                content()
                    .font(.caption)
            }
        }
        .padding(12)
        .background(.regularMaterial, in: RoundedRectangle(cornerRadius: 8))
    }
}

/// A key-value row for property display.
struct PropertyRow: View {
    let key: String
    let value: String

    init(_ key: String, _ value: String) {
        self.key = key
        self.value = value
    }

    var body: some View {
        HStack {
            Text(key)
                .foregroundStyle(.secondary)
            Spacer()
            Text(value)
                .monospaced()
        }
    }
}

// MARK: - Float Extension

private extension Float {
    func clamped(to range: ClosedRange<Float>) -> Float {
        min(max(self, range.lowerBound), range.upperBound)
    }
}
