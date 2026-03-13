#if os(iOS) || os(visionOS)
import RealityKit
import ARKit
import SwiftUI

// MARK: - Entity Extensions

extension Entity {
    /// Look at a target position from a source position
    func look(at target: SIMD3<Float>, from source: SIMD3<Float>, relativeTo referenceEntity: Entity?) {
        let direction = normalize(target - source)
        let rotation = simd_quatf(from: [0, 0, -1], to: direction)
        setOrientation(rotation, relativeTo: referenceEntity)
    }
}

extension ModelEntity {
    /// Apply a simple color material
    func setColor(_ color: UIColor) {
        let material = SimpleMaterial(color: color, isMetallic: false)
        self.model?.materials = [material]
    }

    /// Apply a metallic material
    func setMetallicColor(_ color: UIColor, roughness: Float = 0.5) {
        let material = SimpleMaterial(color: color, roughness: .init(floatLiteral: roughness), isMetallic: true)
        self.model?.materials = [material]
    }
}

// MARK: - Mesh Generation Helpers

extension MeshResource {
    /// Generate a pyramid mesh (RealityKit doesn't have built-in pyramid)
    static func generatePyramid(baseSize: Float = 1, height: Float = 1) -> MeshResource {
        let half = baseSize / 2
        let h = height / 2

        let vertices: [SIMD3<Float>] = [
            // Base vertices
            [-half, -h, -half],
            [half, -h, -half],
            [half, -h, half],
            [-half, -h, half],
            // Apex
            [0, h, 0]
        ]

        let indices: [UInt32] = [
            // Base (2 triangles)
            0, 2, 1,
            0, 3, 2,
            // Front face
            0, 1, 4,
            // Right face
            1, 2, 4,
            // Back face
            2, 3, 4,
            // Left face
            3, 0, 4
        ]

        var descriptor = MeshDescriptor()
        descriptor.positions = MeshBuffer(vertices)
        descriptor.primitives = .triangles(indices)

        do {
            return try MeshResource.generate(from: [descriptor])
        } catch {
            // Fallback to box
            return MeshResource.generateBox(size: baseSize)
        }
    }

    /// Generate a torus mesh (RealityKit doesn't have built-in torus in all versions)
    static func generateTorus(ringRadius: Float = 0.5, pipeRadius: Float = 0.2, radialSegments: Int = 24, pipeSegments: Int = 12) -> MeshResource {
        var vertices: [SIMD3<Float>] = []
        var indices: [UInt32] = []

        for i in 0..<radialSegments {
            let theta = Float(i) * 2 * .pi / Float(radialSegments)
            let cosTheta = cos(theta)
            let sinTheta = sin(theta)

            for j in 0..<pipeSegments {
                let phi = Float(j) * 2 * .pi / Float(pipeSegments)
                let cosPhi = cos(phi)
                let sinPhi = sin(phi)

                let x = (ringRadius + pipeRadius * cosPhi) * cosTheta
                let y = pipeRadius * sinPhi
                let z = (ringRadius + pipeRadius * cosPhi) * sinTheta

                vertices.append([x, y, z])

                // Create faces
                let current = UInt32(i * pipeSegments + j)
                let nextI = UInt32(((i + 1) % radialSegments) * pipeSegments + j)
                let nextJ = UInt32(i * pipeSegments + (j + 1) % pipeSegments)
                let nextIJ = UInt32(((i + 1) % radialSegments) * pipeSegments + (j + 1) % pipeSegments)

                indices.append(contentsOf: [
                    current, nextI, nextJ,
                    nextI, nextIJ, nextJ
                ])
            }
        }

        var descriptor = MeshDescriptor()
        descriptor.positions = MeshBuffer(vertices)
        descriptor.primitives = .triangles(indices)

        do {
            return try MeshResource.generate(from: [descriptor])
        } catch {
            // Fallback to sphere
            return MeshResource.generateSphere(radius: ringRadius)
        }
    }
}

// MARK: - Scene Setup Helpers

extension ARView {
    /// Configure for virtual world mode (no AR camera tracking)
    func configureForVirtualWorld() {
        let config = ARWorldTrackingConfiguration()
        config.planeDetection = []
        config.environmentTexturing = .none
        session.run(config, options: [.resetTracking, .removeExistingAnchors])

        // Disable scene understanding
        environment.sceneUnderstanding.options = []
    }

    /// Add default lighting
    func addDefaultLighting() -> DirectionalLight {
        let light = DirectionalLight()
        light.light.intensity = 2000
        light.light.color = .white
        light.position = [5, 10, 5]
        light.look(at: [0, 0, 0], from: [5, 10, 5], relativeTo: nil)

        let anchor = AnchorEntity(world: .zero)
        anchor.addChild(light)
        scene.addAnchor(anchor)

        return light
    }
}

// MARK: - Transform Helpers

extension simd_quatf {
    /// Create from Euler angles (degrees)
    init(eulerAngles degrees: SIMD3<Float>) {
        let radians = degrees * .pi / 180
        self = simd_quatf(angle: radians.x, axis: [1, 0, 0])
            * simd_quatf(angle: radians.y, axis: [0, 1, 0])
            * simd_quatf(angle: radians.z, axis: [0, 0, 1])
    }

    /// Convert to Euler angles (degrees)
    var eulerAngles: SIMD3<Float> {
        // Extract Euler angles from quaternion (ZYX order)
        let q = self
        let sinr_cosp = 2 * (q.real * q.vector.x + q.vector.y * q.vector.z)
        let cosr_cosp = 1 - 2 * (q.vector.x * q.vector.x + q.vector.y * q.vector.y)
        let roll = atan2(sinr_cosp, cosr_cosp)

        let sinp = 2 * (q.real * q.vector.y - q.vector.z * q.vector.x)
        let pitch: Float
        if abs(sinp) >= 1 {
            pitch = copysign(.pi / 2, sinp)
        } else {
            pitch = asin(sinp)
        }

        let siny_cosp = 2 * (q.real * q.vector.z + q.vector.x * q.vector.y)
        let cosy_cosp = 1 - 2 * (q.vector.y * q.vector.y + q.vector.z * q.vector.z)
        let yaw = atan2(siny_cosp, cosy_cosp)

        return SIMD3<Float>(roll, yaw, pitch) * 180 / .pi
    }
}

// MARK: - Color Conversion

extension Color {
    /// Create Color from SIMD3 (RGB 0-1 range)
    init(simd3: SIMD3<Float>) {
        self.init(red: Double(simd3.x), green: Double(simd3.y), blue: Double(simd3.z))
    }
}

extension SIMD3 where Scalar == Float {
    /// Create SIMD3 from Color
    init(color: Color) {
        let uiColor = UIColor(color)
        var red: CGFloat = 0
        var green: CGFloat = 0
        var blue: CGFloat = 0
        var alpha: CGFloat = 0
        uiColor.getRed(&red, green: &green, blue: &blue, alpha: &alpha)
        self.init(Float(red), Float(green), Float(blue))
    }
}
#endif
