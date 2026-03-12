import Foundation
import RealityKit
import SwiftUI

/// Shape types for primitives
enum PrimitiveShape: String, Codable, CaseIterable {
    case cube
    case sphere
    case cylinder
    case cone
    case plane
    case capsule
    case pyramid
    case torus
}

/// Audio emitter types
enum AudioEmitterType: String, Codable, CaseIterable {
    case water
    case fire
    case hum
    case wind
    case birds
    case custom
}

/// Behavior types for entities
enum BehaviorType: String, Codable, CaseIterable {
    case orbit
    case spin
    case bob
    case lookAt
    case pulse
    case pathFollow
    case bounce
}

/// Data model for a world entity
struct WorldEntity: Identifiable, Codable {
    let id: String
    var name: String
    var shape: PrimitiveShape?
    var position: SIMD3<Float>
    var scale: SIMD3<Float>
    var rotation: SIMD3<Float>
    var color: ColorData
    var behaviors: [EntityBehavior]
    var audioEmitter: AudioEmitterConfig?
    var isModelEntity: Bool
    var modelPath: String?

    init(
        id: String = UUID().uuidString,
        name: String,
        shape: PrimitiveShape? = nil,
        position: SIMD3<Float> = .zero,
        scale: SIMD3<Float> = SIMD3<Float>(1, 1, 1),
        rotation: SIMD3<Float> = .zero,
        color: ColorData = ColorData(r: 0.5, g: 0.5, b: 0.5),
        behaviors: [EntityBehavior] = [],
        isModelEntity: Bool = false,
        modelPath: String? = nil
    ) {
        self.id = id
        self.name = name
        self.shape = shape
        self.position = position
        self.scale = scale
        self.rotation = rotation
        self.color = color
        self.behaviors = behaviors
        self.isModelEntity = isModelEntity
        self.modelPath = modelPath
    }
}

/// Color data for serialization
struct ColorData: Codable {
    let r: Float
    let g: Float
    let b: Float
    let a: Float

    init(r: Float, g: Float, b: Float, a: Float = 1.0) {
        self.r = r
        self.g = g
        self.b = b
        self.a = a
    }

    init(from color: Color) {
        // SwiftUI Color to RGB components
        // Note: This is a simplified conversion
        let uiColor = UIColor(color)
        var red: CGFloat = 0
        var green: CGFloat = 0
        var blue: CGFloat = 0
        var alpha: CGFloat = 0
        uiColor.getRed(&red, green: &green, blue: &blue, alpha: &alpha)
        self.r = Float(red)
        self.g = Float(green)
        self.b = Float(blue)
        self.a = Float(alpha)
    }

    func toUIColor() -> UIColor {
        UIColor(red: CGFloat(r), green: CGFloat(g), blue: CGFloat(b), alpha: CGFloat(a))
    }

    func toColor() -> Color {
        Color(uiColor: toUIColor())
    }
}

/// Behavior configuration for an entity
struct EntityBehavior: Codable {
    let type: BehaviorType
    var parameters: [String: Double]

    init(type: BehaviorType, parameters: [String: Double] = [:]) {
        self.type = type
        self.parameters = parameters
    }
}

/// Audio emitter configuration
struct AudioEmitterConfig: Codable {
    let type: AudioEmitterType
    var volume: Float
    var loop: Bool

    init(type: AudioEmitterType, volume: Float = 1.0, loop: Bool = true) {
        self.type = type
        self.volume = volume
        self.loop = loop
    }
}

/// Parse color from string (hex or name)
func parseColor(_ colorString: String) -> ColorData {
    let lowercased = colorString.lowercased()

    // Named colors
    switch lowercased {
    case "red": return ColorData(r: 1, g: 0, b: 0)
    case "green": return ColorData(r: 0, g: 0.8, b: 0)
    case "blue": return ColorData(r: 0, g: 0, b: 1)
    case "yellow": return ColorData(r: 1, g: 1, b: 0)
    case "cyan": return ColorData(r: 0, g: 1, b: 1)
    case "magenta": return ColorData(r: 1, g: 0, b: 1)
    case "white": return ColorData(r: 1, g: 1, b: 1)
    case "black": return ColorData(r: 0, g: 0, b: 0)
    case "gray", "grey": return ColorData(r: 0.5, g: 0.5, b: 0.5)
    case "orange": return ColorData(r: 1, g: 0.5, b: 0)
    case "purple": return ColorData(r: 0.5, g: 0, b: 0.5)
    case "pink": return ColorData(r: 1, g: 0.75, b: 0.8)
    case "brown": return ColorData(r: 0.6, g: 0.3, b: 0.1)
    case "teal": return ColorData(r: 0, g: 0.8, b: 0.8)
    default:
        // Try hex parsing
        if let hex = parseHexColor(colorString) {
            return hex
        }
        // Default gray
        return ColorData(r: 0.5, g: 0.5, b: 0.5)
    }
}

private func parseHexColor(_ hex: String) -> ColorData? {
    var hexSanitized = hex.trimmingCharacters(in: .whitespacesAndNewlines)
    hexSanitized = hexSanitized.replacingOccurrences(of: "#", with: "")

    var rgb: UInt64 = 0
    guard Scanner(string: hexSanitized).scanHexInt64(&rgb) else { return nil }

    let length = hexSanitized.count
    if length == 6 {
        return ColorData(
            r: Float((rgb & 0xFF0000) >> 16) / 255.0,
            g: Float((rgb & 0x00FF00) >> 8) / 255.0,
            b: Float(rgb & 0x0000FF) / 255.0
        )
    } else if length == 8 {
        return ColorData(
            r: Float((rgb & 0xFF000000) >> 24) / 255.0,
            g: Float((rgb & 0x00FF0000) >> 16) / 255.0,
            b: Float((rgb & 0x0000FF00) >> 8) / 255.0,
            a: Float(rgb & 0x000000FF) / 255.0
        )
    }
    return nil
}
