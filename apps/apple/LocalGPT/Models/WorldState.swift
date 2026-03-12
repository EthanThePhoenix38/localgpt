import Foundation
import RealityKit

/// Complete world state for save/load
struct WorldState: Codable {
    var name: String
    var entities: [WorldEntity]
    var camera: CameraState
    var lighting: LightingState
    var ambience: AmbienceState
    var createdAt: Date
    var modifiedAt: Date

    init(name: String = "Untitled World") {
        self.name = name
        self.entities = []
        self.camera = CameraState()
        self.lighting = LightingState()
        self.ambience = AmbienceState()
        self.createdAt = Date()
        self.modifiedAt = Date()
    }
}

/// Camera state
struct CameraState: Codable {
    var position: SIMD3<Float>
    var lookAt: SIMD3<Float>
    var fieldOfView: Float

    init(
        position: SIMD3<Float> = SIMD3<Float>(0, 5, 10),
        lookAt: SIMD3<Float> = .zero,
        fieldOfView: Float = 60
    ) {
        self.position = position
        self.lookAt = lookAt
        self.fieldOfView = fieldOfView
    }
}

/// Lighting state
struct LightingState: Codable {
    var intensity: Float
    var color: ColorData
    var isHDREnabled: Bool

    init(
        intensity: Float = 1.0,
        color: ColorData = ColorData(r: 1, g: 1, b: 1),
        isHDREnabled: Bool = true
    ) {
        self.intensity = intensity
        self.color = color
        self.isHDREnabled = isHDREnabled
    }
}

/// Ambient audio state
struct AmbienceState: Codable {
    var type: AmbienceType
    var volume: Float

    init(type: AmbienceType = .silence, volume: Float = 0.5) {
        self.type = type
        self.volume = volume
    }
}

/// Ambient sound types
enum AmbienceType: String, Codable, CaseIterable {
    case silence
    case wind
    case rain
    case forest
    case ocean
    case cave
    case stream
    case night
    case city
}

// MARK: - SIMD3 Codable Extensions

extension SIMD3: Codable where Scalar: Codable {
    public func encode(to encoder: Encoder) throws {
        var container = encoder.unkeyedContainer()
        try container.encode(x)
        try container.encode(y)
        try container.encode(z)
    }

    public init(from decoder: Decoder) throws {
        var container = try decoder.unkeyedContainer()
        let x = try container.decode(Scalar.self)
        let y = try container.decode(Scalar.self)
        let z = try container.decode(Scalar.self)
        self.init(x, y, z)
    }
}

// MARK: - World Persistence

extension WorldState {
    /// Save world to documents directory
    func save(name: String) throws -> URL {
        let docs = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask).first!
        let worldsDir = docs.appendingPathComponent("Worlds", isDirectory: true)

        // Create worlds directory if needed
        if !FileManager.default.fileExists(atPath: worldsDir.path) {
            try FileManager.default.createDirectory(at: worldsDir, withIntermediateDirectories: true)
        }

        let worldDir = worldsDir.appendingPathComponent(name, isDirectory: true)
        if !FileManager.default.fileExists(atPath: worldDir.path) {
            try FileManager.default.createDirectory(at: worldDir, withIntermediateDirectories: true)
        }

        // Save JSON state
        let jsonURL = worldDir.appendingPathComponent("world.json")
        let encoder = JSONEncoder()
        encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
        encoder.dateEncodingStrategy = .iso8601
        let data = try encoder.encode(self)
        try data.write(to: jsonURL)

        return jsonURL
    }

    /// Load world from documents directory
    static func load(name: String) throws -> WorldState {
        let docs = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask).first!
        let jsonURL = docs.appendingPathComponent("Worlds/\(name)/world.json")

        let data = try Data(contentsOf: jsonURL)
        let decoder = JSONDecoder()
        decoder.dateDecodingStrategy = .iso8601
        return try decoder.decode(WorldState.self, from: data)
    }

    /// List all saved worlds
    static func listAll() throws -> [String] {
        let docs = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask).first!
        let worldsDir = docs.appendingPathComponent("Worlds", isDirectory: true)

        guard FileManager.default.fileExists(atPath: worldsDir.path) else {
            return []
        }

        let contents = try FileManager.default.contentsOfDirectory(
            at: worldsDir,
            includingPropertiesForKeys: [.isDirectoryKey],
            options: [.skipsHiddenFiles]
        )

        return contents
            .filter { (try? $0.resourceValues(forKeys: [.isDirectoryKey]).isDirectory) ?? false }
            .map { $0.lastPathComponent }
            .sorted()
    }

    /// Delete a saved world
    static func delete(name: String) throws {
        let docs = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask).first!
        let worldDir = docs.appendingPathComponent("Worlds/\(name)", isDirectory: true)
        try FileManager.default.removeItem(at: worldDir)
    }
}
