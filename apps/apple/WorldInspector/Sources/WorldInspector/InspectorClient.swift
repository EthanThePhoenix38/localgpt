/// WebSocket client for connecting to the Bevy World Inspector server.

import Foundation
import Observation

/// Connection state for the inspector client.
public enum ConnectionState: Equatable {
    case disconnected
    case connecting
    case connected
    case error(String)
}

/// Observable client that manages the WebSocket connection and world state.
@Observable
public final class InspectorClient {
    // MARK: - Published State

    /// Current connection state.
    public private(set) var connectionState: ConnectionState = .disconnected

    /// The entity tree from the server.
    public private(set) var sceneTree: [TreeEntity] = []

    /// Currently selected entity ID.
    public private(set) var selectedEntityId: UInt64?

    /// Detail data for the selected entity.
    public private(set) var selectedEntityDetail: EntityDetailData?

    /// Global world info.
    public private(set) var worldInfo: WorldInfoData?

    /// Live transform of the selected entity (streamed at 10Hz).
    public private(set) var selectedTransform: (position: SIMD3<Float>, rotation: SIMD3<Float>)?

    // MARK: - Private

    private var webSocketTask: URLSessionWebSocketTask?
    private let session = URLSession(configuration: .default)
    private let encoder = JSONEncoder()
    private let decoder = JSONDecoder()

    public init() {}

    // MARK: - Connection

    /// Connect to the inspector WebSocket server.
    public func connect(host: String = "localhost", port: Int = 9877) {
        guard connectionState != .connecting && connectionState != .connected else { return }

        connectionState = .connecting
        let url = URL(string: "ws://\(host):\(port)/ws")!
        webSocketTask = session.webSocketTask(with: url)
        webSocketTask?.resume()
        connectionState = .connected
        receiveMessages()
    }

    /// Disconnect from the server.
    public func disconnect() {
        webSocketTask?.cancel(with: .normalClosure, reason: nil)
        webSocketTask = nil
        connectionState = .disconnected
        sceneTree = []
        selectedEntityId = nil
        selectedEntityDetail = nil
        worldInfo = nil
        selectedTransform = nil
    }

    // MARK: - Commands

    /// Select an entity by ID. Syncs across all connected clients.
    public func selectEntity(_ entityId: UInt64) {
        send(.selectEntity(entityId: entityId))
        selectedEntityId = entityId
        selectedEntityDetail = nil
        send(.requestEntityDetail(entityId: entityId))
    }

    /// Clear the selection.
    public func deselect() {
        send(.deselect)
        selectedEntityId = nil
        selectedEntityDetail = nil
        selectedTransform = nil
    }

    /// Toggle visibility of an entity.
    public func toggleVisibility(_ entityId: UInt64) {
        send(.toggleVisibility(entityId: entityId))
    }

    /// Focus the Bevy camera on an entity.
    public func focusEntity(_ entityId: UInt64) {
        send(.focusEntity(entityId: entityId))
    }

    /// Request a fresh scene tree.
    public func refreshSceneTree() {
        send(.requestSceneTree)
    }

    /// Request fresh world info.
    public func refreshWorldInfo() {
        send(.requestWorldInfo)
    }

    // MARK: - Private

    private func send(_ message: ClientMessage) {
        guard let data = try? encoder.encode(message),
              let text = String(data: data, encoding: .utf8)
        else { return }

        webSocketTask?.send(.string(text)) { [weak self] error in
            if let error {
                self?.connectionState = .error(error.localizedDescription)
            }
        }
    }

    private func receiveMessages() {
        webSocketTask?.receive { [weak self] result in
            guard let self else { return }

            switch result {
            case .success(let message):
                switch message {
                case .string(let text):
                    self.handleTextMessage(text)
                case .data:
                    break // Binary frames (GLB snapshots) — not yet handled
                @unknown default:
                    break
                }
                // Continue receiving
                self.receiveMessages()

            case .failure(let error):
                self.connectionState = .error(error.localizedDescription)
            }
        }
    }

    private func handleTextMessage(_ text: String) {
        guard let data = text.data(using: .utf8),
              let message = try? decoder.decode(ServerMessage.self, from: data)
        else { return }

        Task { @MainActor in
            switch message {
            case .sceneTree(let entities):
                self.sceneTree = entities

            case .entityDetail(let entityId, let detail):
                if self.selectedEntityId == entityId {
                    self.selectedEntityDetail = detail
                }

            case .worldInfo(let info):
                self.worldInfo = info

            case .selectionChanged(let entityId):
                self.selectedEntityId = entityId
                self.selectedEntityDetail = nil
                self.selectedTransform = nil
                self.send(.requestEntityDetail(entityId: entityId))

            case .selectionCleared:
                self.selectedEntityId = nil
                self.selectedEntityDetail = nil
                self.selectedTransform = nil

            case .sceneChanged:
                self.send(.requestSceneTree)

            case .entityTransformUpdated(let entityId, let pos, let rot):
                if self.selectedEntityId == entityId {
                    self.selectedTransform = (position: pos, rotation: rot)
                }
            }
        }
    }
}
