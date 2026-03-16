/// World Inspector Protocol — Swift types matching the Bevy WebSocket protocol.
///
/// These Codable types mirror the Rust `protocol.rs` definitions exactly,
/// enabling JSON serialization/deserialization over WebSocket.

import Foundation

// MARK: - Client → Server Messages

/// Messages sent from the inspector client to the Bevy server.
public enum ClientMessage: Encodable {
    case subscribe(topics: [String])
    case requestSceneTree
    case requestEntityDetail(entityId: UInt64)
    case requestWorldInfo
    case selectEntity(entityId: UInt64)
    case deselect
    case toggleVisibility(entityId: UInt64)
    case focusEntity(entityId: UInt64)
    case requestSceneSnapshot

    public func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .subscribe(let topics):
            try container.encode("subscribe", forKey: .type)
            try container.encode(topics, forKey: .topics)
        case .requestSceneTree:
            try container.encode("request_scene_tree", forKey: .type)
        case .requestEntityDetail(let id):
            try container.encode("request_entity_detail", forKey: .type)
            try container.encode(id, forKey: .entityId)
        case .requestWorldInfo:
            try container.encode("request_world_info", forKey: .type)
        case .selectEntity(let id):
            try container.encode("select_entity", forKey: .type)
            try container.encode(id, forKey: .entityId)
        case .deselect:
            try container.encode("deselect", forKey: .type)
        case .toggleVisibility(let id):
            try container.encode("toggle_visibility", forKey: .type)
            try container.encode(id, forKey: .entityId)
        case .focusEntity(let id):
            try container.encode("focus_entity", forKey: .type)
            try container.encode(id, forKey: .entityId)
        case .requestSceneSnapshot:
            try container.encode("request_scene_snapshot", forKey: .type)
        }
    }

    private enum CodingKeys: String, CodingKey {
        case type
        case topics
        case entityId = "entity_id"
    }
}

// MARK: - Server → Client Messages

/// Messages received from the Bevy server.
public enum ServerMessage: Decodable {
    case sceneTree(entities: [TreeEntity])
    case entityDetail(entityId: UInt64, data: EntityDetailData)
    case worldInfo(data: WorldInfoData)
    case selectionChanged(entityId: UInt64)
    case selectionCleared
    case sceneChanged
    case entityTransformUpdated(entityId: UInt64, position: SIMD3<Float>, rotationDegrees: SIMD3<Float>)

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let type = try container.decode(String.self, forKey: .type)

        switch type {
        case "scene_tree":
            let entities = try container.decode([TreeEntity].self, forKey: .entities)
            self = .sceneTree(entities: entities)
        case "entity_detail":
            let id = try container.decode(UInt64.self, forKey: .entityId)
            let data = try container.decode(EntityDetailData.self, forKey: .data)
            self = .entityDetail(entityId: id, data: data)
        case "world_info":
            let data = try container.decode(WorldInfoData.self, forKey: .data)
            self = .worldInfo(data: data)
        case "selection_changed":
            let id = try container.decode(UInt64.self, forKey: .entityId)
            self = .selectionChanged(entityId: id)
        case "selection_cleared":
            self = .selectionCleared
        case "scene_changed":
            self = .sceneChanged
        case "entity_transform_updated":
            let id = try container.decode(UInt64.self, forKey: .entityId)
            let pos = try container.decode([Float].self, forKey: .position)
            let rot = try container.decode([Float].self, forKey: .rotationDegrees)
            self = .entityTransformUpdated(
                entityId: id,
                position: SIMD3(pos[0], pos[1], pos[2]),
                rotationDegrees: SIMD3(rot[0], rot[1], rot[2])
            )
        default:
            throw DecodingError.dataCorruptedError(
                forKey: .type, in: container,
                debugDescription: "Unknown message type: \(type)"
            )
        }
    }

    private enum CodingKeys: String, CodingKey {
        case type
        case entities
        case entityId = "entity_id"
        case data
        case position
        case rotationDegrees = "rotation_degrees"
    }
}

// MARK: - Data Types

public struct TreeEntity: Codable, Identifiable, Hashable {
    public let id: UInt64
    public let name: String
    public let entityType: String
    public let parentId: UInt64?
    public let visible: Bool
    public let children: [UInt64]

    enum CodingKeys: String, CodingKey {
        case id, name, visible, children
        case entityType = "entity_type"
        case parentId = "parent_id"
    }
}

public struct EntityDetailData: Codable {
    public let identity: IdentitySection
    public let transform: TransformSection?
    public let shape: String?
    public let material: MaterialSection?
    public let light: LightSection?
    public let behaviors: [BehaviorSection]
    public let audio: AudioSection?
    public let meshAsset: String?
    public let hierarchy: HierarchySection

    enum CodingKeys: String, CodingKey {
        case identity, transform, shape, material, light, behaviors, audio, hierarchy
        case meshAsset = "mesh_asset"
    }
}

public struct IdentitySection: Codable {
    public let name: String
    public let id: UInt64
    public let entityType: String

    enum CodingKeys: String, CodingKey {
        case name, id
        case entityType = "entity_type"
    }
}

public struct TransformSection: Codable {
    public let position: [Float]
    public let rotationDegrees: [Float]
    public let scale: [Float]
    public let visible: Bool

    enum CodingKeys: String, CodingKey {
        case position, scale, visible
        case rotationDegrees = "rotation_degrees"
    }
}

public struct MaterialSection: Codable {
    public let baseColor: [Float]
    public let metallic: Float
    public let roughness: Float
    public let reflectance: Float
    public let emissive: [Float]
    public let alphaMode: String
    public let doubleSided: Bool
    public let unlit: Bool

    enum CodingKeys: String, CodingKey {
        case metallic, roughness, reflectance, emissive, unlit
        case baseColor = "base_color"
        case alphaMode = "alpha_mode"
        case doubleSided = "double_sided"
    }
}

public struct LightSection: Codable {
    public let lightType: String
    public let color: [Float]
    public let intensity: Float
    public let range: Float?
    public let shadowsEnabled: Bool
    public let innerAngle: Float?
    public let outerAngle: Float?

    enum CodingKeys: String, CodingKey {
        case color, intensity, range
        case lightType = "light_type"
        case shadowsEnabled = "shadows_enabled"
        case innerAngle = "inner_angle"
        case outerAngle = "outer_angle"
    }
}

public struct BehaviorSection: Codable, Identifiable {
    public let id: String
    public let behaviorType: String
    public let basePosition: [Float]
    public let baseScale: [Float]

    enum CodingKeys: String, CodingKey {
        case id
        case behaviorType = "behavior_type"
        case basePosition = "base_position"
        case baseScale = "base_scale"
    }
}

public struct AudioSection: Codable {
    public let soundType: String
    public let volume: Float
    public let radius: Float
    public let attachedTo: String?
    public let position: [Float]?

    enum CodingKeys: String, CodingKey {
        case volume, radius, position
        case soundType = "sound_type"
        case attachedTo = "attached_to"
    }
}

public struct HierarchySection: Codable {
    public let parent: String?
    public let children: [String]
}

public struct WorldInfoData: Codable {
    public let name: String?
    public let entityCount: Int
    public let behaviorState: BehaviorStateInfo
    public let audio: AudioStateInfo?

    enum CodingKeys: String, CodingKey {
        case name, audio
        case entityCount = "entity_count"
        case behaviorState = "behavior_state"
    }
}

public struct BehaviorStateInfo: Codable {
    public let paused: Bool
    public let elapsed: Double
}

public struct AudioStateInfo: Codable {
    public let active: Bool
    public let emitterCount: Int
    public let ambienceLayers: [String]

    enum CodingKeys: String, CodingKey {
        case active
        case emitterCount = "emitter_count"
        case ambienceLayers = "ambience_layers"
    }
}
