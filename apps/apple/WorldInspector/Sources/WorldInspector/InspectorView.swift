/// Main inspector view — combines outliner sidebar, detail panel, and world info bar.

import SwiftUI

/// The top-level inspector view with split navigation.
public struct InspectorView: View {
    @State private var client = InspectorClient()
    @State private var showConnectionSheet = false

    public init() {}

    public var body: some View {
        NavigationSplitView {
            OutlinerView(client: client)
                .navigationTitle("Outliner")
                #if os(macOS)
                .navigationSplitViewColumnWidth(min: 200, ideal: 280, max: 400)
                #endif
        } detail: {
            DetailView(client: client)
                .navigationTitle(selectedEntityName)
        }
        .toolbar {
            ToolbarItemGroup(placement: .primaryAction) {
                Button {
                    if client.connectionState == .connected {
                        client.disconnect()
                    } else {
                        client.connect()
                    }
                } label: {
                    Image(systemName: client.connectionState == .connected
                          ? "wifi" : "wifi.slash")
                }
                .help(client.connectionState == .connected ? "Disconnect" : "Connect")

                if client.connectionState == .connected {
                    Button {
                        client.refreshSceneTree()
                    } label: {
                        Image(systemName: "arrow.clockwise")
                    }
                    .help("Refresh scene tree")
                }
            }
        }
        .safeAreaInset(edge: .bottom) {
            worldInfoBar
        }
        .onAppear {
            client.connect()
        }
        .onDisappear {
            client.disconnect()
        }
    }

    private var selectedEntityName: String {
        if let id = client.selectedEntityId,
           let entity = client.sceneTree.first(where: { $0.id == id }) {
            return entity.name
        }
        return "Inspector"
    }

    @ViewBuilder
    private var worldInfoBar: some View {
        if let info = client.worldInfo {
            HStack(spacing: 16) {
                if let name = info.name {
                    Label(name, systemImage: "globe")
                }

                Label("\(info.entityCount) entities", systemImage: "cube.fill")

                Label(
                    info.behaviorState.paused ? "Paused" : String(format: "%.1fs", info.behaviorState.elapsed),
                    systemImage: info.behaviorState.paused ? "pause.circle" : "play.circle"
                )

                if let audio = info.audio {
                    Label(
                        audio.active ? "\(audio.emitterCount) emitters" : "Off",
                        systemImage: audio.active ? "speaker.wave.2.fill" : "speaker.slash"
                    )
                }

                Spacer()
            }
            .font(.caption)
            .foregroundStyle(.secondary)
            .padding(.horizontal, 12)
            .padding(.vertical, 6)
            .background(.bar)
        }
    }
}
