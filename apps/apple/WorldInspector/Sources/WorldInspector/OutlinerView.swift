/// Entity outliner sidebar — hierarchical tree of all world entities.

import SwiftUI

/// Displays the entity tree as a hierarchical list with search.
public struct OutlinerView: View {
    let client: InspectorClient

    @State private var searchText = ""
    @State private var expandedIds: Set<UInt64> = []

    public init(client: InspectorClient) {
        self.client = client
    }

    public var body: some View {
        VStack(spacing: 0) {
            // Search bar
            HStack {
                Image(systemName: "magnifyingglass")
                    .foregroundStyle(.secondary)
                TextField("Search entities...", text: $searchText)
                    .textFieldStyle(.plain)
                if !searchText.isEmpty {
                    Button { searchText = "" } label: {
                        Image(systemName: "xmark.circle.fill")
                            .foregroundStyle(.secondary)
                    }
                    .buttonStyle(.plain)
                }
            }
            .padding(8)

            Divider()

            // Entity tree
            List(selection: Binding(
                get: { client.selectedEntityId },
                set: { newId in
                    if let id = newId {
                        client.selectEntity(id)
                    } else {
                        client.deselect()
                    }
                }
            )) {
                ForEach(rootEntities, id: \.id) { entity in
                    entityRow(entity)
                }
            }
            .listStyle(.sidebar)

            Divider()

            // Status bar
            HStack {
                Text("\(client.sceneTree.count) entities")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                Spacer()
                connectionIndicator
            }
            .padding(.horizontal, 8)
            .padding(.vertical, 4)
        }
    }

    // MARK: - Entity Row

    private func entityRow(_ entity: TreeEntity) -> AnyView {
        let children = childEntities(of: entity.id)

        if children.isEmpty {
            return AnyView(entityLabel(entity))
        } else {
            return AnyView(
                DisclosureGroup(
                    isExpanded: Binding(
                        get: { expandedIds.contains(entity.id) },
                        set: { expanded in
                            if expanded {
                                expandedIds.insert(entity.id)
                            } else {
                                expandedIds.remove(entity.id)
                            }
                        }
                    )
                ) {
                    ForEach(children, id: \.id) { child in
                        entityRow(child)
                    }
                } label: {
                    entityLabel(entity)
                }
            )
        }
    }

    @ViewBuilder
    private func entityLabel(_ entity: TreeEntity) -> some View {
        HStack(spacing: 6) {
            Image(systemName: iconName(for: entity.entityType))
                .foregroundStyle(entity.visible ? .primary : .tertiary)
                .frame(width: 16)

            Text(entity.name)
                .strikethrough(!entity.visible)
                .foregroundStyle(entity.visible ? .primary : .secondary)

            Spacer()

            Button {
                client.toggleVisibility(entity.id)
            } label: {
                Image(systemName: entity.visible ? "eye" : "eye.slash")
                    .foregroundStyle(.secondary)
            }
            .buttonStyle(.plain)
        }
        .tag(entity.id)
    }

    // MARK: - Helpers

    private var rootEntities: [TreeEntity] {
        let filtered = searchText.isEmpty
            ? client.sceneTree
            : client.sceneTree.filter { $0.name.localizedCaseInsensitiveContains(searchText) }

        return filtered.filter { $0.parentId == nil }
    }

    private func childEntities(of parentId: UInt64) -> [TreeEntity] {
        client.sceneTree.filter { $0.parentId == parentId }
    }

    private func iconName(for entityType: String) -> String {
        switch entityType {
        case "Primitive": return "cube.fill"
        case "Light": return "sun.max.fill"
        case "Camera": return "camera.fill"
        case "Mesh": return "pyramid.fill"
        case "Group": return "folder.fill"
        case "AudioEmitter": return "speaker.wave.2.fill"
        default: return "questionmark.circle"
        }
    }

    @ViewBuilder
    private var connectionIndicator: some View {
        switch client.connectionState {
        case .connected:
            Circle()
                .fill(.green)
                .frame(width: 8, height: 8)
        case .connecting:
            ProgressView()
                .controlSize(.mini)
        case .error:
            Circle()
                .fill(.red)
                .frame(width: 8, height: 8)
        case .disconnected:
            Circle()
                .fill(.gray)
                .frame(width: 8, height: 8)
        }
    }
}
