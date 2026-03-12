import Foundation
import RealityKit
import Combine

/// Service for managing entity behaviors and animations.
/// This service provides behavior definitions and parameter validation.
/// Actual behavior execution is handled by WorldViewModel's update loop.
@MainActor
class WorldBehaviorService: ObservableObject {

    // MARK: - Behavior Definitions

    /// Default parameters for each behavior type
    static let defaultParameters: [BehaviorType: [String: Double]] = [
        .orbit: ["radius": 3.0, "speed": 1.0, "axis": 1],
        .spin: ["speed": 90.0, "axis": 1],
        .bob: ["amplitude": 0.5, "speed": 2.0],
        .lookAt: [:],
        .pulse: ["min_scale": 0.8, "max_scale": 1.2, "speed": 2.0],
        .pathFollow: ["speed": 1.0],
        .bounce: ["gravity": -9.8, "bounce": 0.7, "floor_y": 0.0]
    ]

    /// Get default parameters for a behavior type
    static func getDefaultParameters(for type: BehaviorType) -> [String: Double] {
        return defaultParameters[type] ?? [:]
    }

    /// Validate and merge parameters with defaults
    static func validateParameters(
        type: BehaviorType,
        customParams: [String: Double]
    ) -> [String: Double] {
        var params = getDefaultParameters(for: type)
        for (key, value) in customParams {
            params[key] = value
        }
        return params
    }

    /// Get description for a behavior type
    static func getDescription(for type: BehaviorType) -> String {
        switch type {
        case .orbit:
            return "Orbits around a center point in the XZ plane"
        case .spin:
            return "Rotates continuously around an axis"
        case .bob:
            return "Oscillates up and down sinusoidally"
        case .lookAt:
            return "Always faces a target entity or point"
        case .pulse:
            return "Scales rhythmically between min and max"
        case .pathFollow:
            return "Follows a series of waypoints"
        case .bounce:
            return "Bounces with gravity and damping"
        }
    }

    /// Get parameter descriptions for a behavior type
    static func getParameterDescriptions(for type: BehaviorType) -> [String: String] {
        switch type {
        case .orbit:
            return [
                "radius": "Distance from center point",
                "speed": "Angular velocity (radians/second)",
                "axis": "Rotation axis (0=X, 1=Y, 2=Z)"
            ]
        case .spin:
            return [
                "speed": "Rotation speed (degrees/second)",
                "axis": "Rotation axis (0=X, 1=Y, 2=Z)"
            ]
        case .bob:
            return [
                "amplitude": "Height variation",
                "speed": "Oscillation frequency"
            ]
        case .lookAt:
            return [
                "target": "Target entity ID or position"
            ]
        case .pulse:
            return [
                "min_scale": "Minimum scale factor",
                "max_scale": "Maximum scale factor",
                "speed": "Pulse frequency"
            ]
        case .pathFollow:
            return [
                "speed": "Movement speed",
                "loop": "Whether to loop (1) or stop (0)"
            ]
        case .bounce:
            return [
                "gravity": "Gravitational acceleration",
                "bounce": "Bounce coefficient (0-1)",
                "floor_y": "Floor level"
            ]
        }
    }
}
