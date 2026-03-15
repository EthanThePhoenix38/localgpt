#if os(macOS)
import Foundation
import SceneKit
import Combine

/// ViewModel managing the SceneKit scene for world generation on macOS.
@MainActor
class WorldViewModelSceneKit: ObservableObject {
    // MARK: - Published State

    @Published var entities: [String: WorldEntity] = [:]
    @Published var entityNames: [String] = []
    @Published var worldState = WorldState()
    @Published var isProcessing = false
    @Published var lastError: String?
    @Published var showError = false

    // MARK: - SceneKit Components

    var scnView: SCNView?
    let scene = SCNScene()
    let rootNode = SCNNode()
    private var sceneEntities: [String: SCNNode] = [:]
    private var cameraNode: SCNNode?
    private var sunLight: SCNLight?

    // MARK: - Behavior State

    private var activeBehaviors: [String: [ActiveBehavior]] = [:]
    private var displayLink: CVDisplayLink?
    private var lastBehaviorTime: TimeInterval = CACurrentMediaTime()

    struct ActiveBehavior {
        let type: BehaviorType
        var parameters: [String: Double]
        var phase: Float = 0
        var baseY: Float = 0
        var velocity: Float = 0
    }

    // MARK: - Counters

    private var entityCounter = 0

    // MARK: - Initialization

    init() {
        setupDefaultLighting()
        setupScene()
        startBehaviorUpdateLoop()
    }

    private func setupScene() {
        scene.rootNode.addChildNode(rootNode)

        // Add grid floor
        let floorGeometry = SCNFloor()
        floorGeometry.width = 100
        floorGeometry.length = 100
        floorGeometry.reflectivity = 0.1
        let floorNode = SCNNode(geometry: floorGeometry)
        floorNode.position = SCNVector3(0, -0.01, 0)
        rootNode.addChildNode(floorNode)

        // Setup camera
        setupCamera()
    }

    private func setupDefaultLighting() {
        // Create directional sun light
        let sunNode = SCNNode()
        sunLight = SCNLight()
        sunLight!.type = .directional
        sunLight!.intensity = 2000
        sunLight!.color = NSColor.white
        sunNode.light = sunLight
        sunNode.position = SCNVector3(5, 10, 5)
        sunNode.look(at: SCNVector3(0, 0, 0))
        rootNode.addChildNode(sunNode)

        // Add ambient light
        let ambientNode = SCNNode()
        let ambientLight = SCNLight()
        ambientLight.type = .ambient
        ambientLight.intensity = 200
        ambientLight.color = NSColor.white
        ambientNode.light = ambientLight
        rootNode.addChildNode(ambientNode)
    }

    private func setupCamera() {
        let camera = SCNCamera()
        camera.fieldOfView = 60
        camera.zNear = 0.1
        camera.zFar = 1000

        cameraNode = SCNNode()
        cameraNode!.camera = camera
        cameraNode!.position = SCNVector3(worldState.camera.position.x, worldState.camera.position.y, worldState.camera.position.z)
        cameraNode!.look(at: SCNVector3(worldState.camera.lookAt.x, worldState.camera.lookAt.y, worldState.camera.lookAt.z))
        rootNode.addChildNode(cameraNode!)
    }

    func setupSCNView(_ view: SCNView) {
        self.scnView = view
        view.scene = scene
        view.pointOfView = cameraNode
        view.allowsCameraControl = true
        view.autoenablesDefaultLighting = false
        view.backgroundColor = NSColor.windowBackgroundColor
        view.antialiasingMode = .multisampling4X
    }

    private func startBehaviorUpdateLoop() {
        lastBehaviorTime = CACurrentMediaTime()

        CVDisplayLinkCreateWithActiveCGDisplays(&displayLink)
        CVDisplayLinkSetOutputCallback(displayLink!, { _, inNow, _, _, _, userInfo in
            guard let userInfo = userInfo else { return kCVReturnSuccess }
            let viewModel = Unmanaged<WorldViewModelSceneKit>.fromOpaque(userInfo).takeUnretainedValue()
            let currentTime = CACurrentMediaTime()
            let dt = Float(currentTime - viewModel.lastBehaviorTime)
            viewModel.lastBehaviorTime = currentTime

            DispatchQueue.main.async {
                viewModel.updateBehaviors(dt: dt)
            }
            return kCVReturnSuccess
        }, Unmanaged.passUnretained(self).toOpaque())
        CVDisplayLinkStart(displayLink!)
    }

    // MARK: - Entity Management

    @discardableResult
    func spawnPrimitive(
        shape: String,
        position: [Float],
        color: String,
        scale: [Float]? = nil,
        rotation: [Float]? = nil,
        name: String? = nil
    ) -> String {
        guard let shapeType = PrimitiveShape(rawValue: shape.lowercased()) else {
            handleError("Unknown shape type: \(shape)")
            return ""
        }

        let entityId = name ?? generateEntityName(for: shapeType)
        let pos = position.count >= 3 ? SIMD3<Float>(position[0], position[1], position[2]) : .zero
        let scl = scale != nil && scale!.count >= 3 ? SIMD3<Float>(scale![0], scale![1], scale![2]) : SIMD3<Float>(1, 1, 1)
        let rot = rotation != nil && rotation!.count >= 3 ? SIMD3<Float>(rotation![0], rotation![1], rotation![2]) : .zero

        // Create geometry based on shape
        let geometry: SCNGeometry
        switch shapeType {
        case .cube:
            geometry = SCNBox(width: 1, height: 1, length: 1, chamferRadius: 0)
        case .sphere:
            geometry = SCNSphere(radius: 0.5)
        case .cylinder:
            geometry = SCNCylinder(radius: 0.5, height: 1)
        case .cone:
            geometry = SCNCone(topRadius: 0, bottomRadius: 0.5, height: 1)
        case .plane:
            geometry = SCNPlane(width: 1, height: 1)
        case .capsule:
            geometry = SCNCapsule(capRadius: 0.25, height: 1)
        case .pyramid:
            geometry = SCNPyramid(width: 1, height: 1, length: 1)
        case .torus:
            geometry = SCNTorus(ringRadius: 0.5, pipeRadius: 0.2)
        }

        // Apply color
        let colorData = parseColor(color)
        let material = SCNMaterial()
        material.diffuse.contents = colorData.toNSColor()
        material.lightingModel = .physicallyBased
        geometry.materials = [material]

        // Create node
        let node = SCNNode(geometry: geometry)
        node.position = SCNVector3(pos.x, pos.y, pos.z)
        node.scale = SCNVector3(scl.x, scl.y, scl.z)
        node.eulerAngles = SCNVector3(
            rot.x * .pi / 180,
            rot.y * .pi / 180,
            rot.z * .pi / 180
        )
        node.name = entityId

        rootNode.addChildNode(node)
        sceneEntities[entityId] = node

        // Track in data model
        let worldEntity = WorldEntity(
            id: entityId,
            name: entityId,
            shape: shapeType,
            position: pos,
            scale: scl,
            rotation: rot,
            color: colorData
        )
        entities[entityId] = worldEntity
        entityNames.append(entityId)
        worldState.entities.append(worldEntity)
        worldState.modifiedAt = Date()

        return entityId
    }

    @discardableResult
    func modifyEntity(
        entityId: String,
        position: [Float]? = nil,
        scale: [Float]? = nil,
        rotation: [Float]? = nil,
        color: String? = nil
    ) -> String {
        guard var worldEntity = entities[entityId],
              let node = sceneEntities[entityId] else {
            handleError("Entity '\(entityId)' not found")
            return "Error: Entity '\(entityId)' not found"
        }

        if let pos = position, pos.count >= 3 {
            node.position = SCNVector3(pos[0], pos[1], pos[2])
            worldEntity.position = SIMD3(pos[0], pos[1], pos[2])
        }

        if let scl = scale, scl.count >= 3 {
            node.scale = SCNVector3(scl[0], scl[1], scl[2])
            worldEntity.scale = SIMD3(scl[0], scl[1], scl[2])
        }

        if let rot = rotation, rot.count >= 3 {
            node.eulerAngles = SCNVector3(
                rot[0] * .pi / 180,
                rot[1] * .pi / 180,
                rot[2] * .pi / 180
            )
            worldEntity.rotation = SIMD3(rot[0], rot[1], rot[2])
        }

        if let col = color {
            let colorData = parseColor(col)
            let material = SCNMaterial()
            material.diffuse.contents = colorData.toNSColor()
            material.lightingModel = .physicallyBased
            node.geometry?.materials = [material]
            worldEntity.color = colorData
        }

        entities[entityId] = worldEntity
        if let idx = worldState.entities.firstIndex(where: { $0.id == entityId }) {
            worldState.entities[idx] = worldEntity
        }
        worldState.modifiedAt = Date()

        return "Modified entity '\(entityId)'"
    }

    func deleteEntity(_ entityId: String) -> String {
        guard let node = sceneEntities[entityId] else {
            return "Error: Entity '\(entityId)' not found"
        }

        activeBehaviors.removeValue(forKey: entityId)
        node.removeFromParentNode()
        sceneEntities.removeValue(forKey: entityId)
        entities.removeValue(forKey: entityId)
        entityNames.removeAll { $0 == entityId }
        worldState.entities.removeAll { $0.id == entityId }
        worldState.modifiedAt = Date()

        return "Deleted entity '\(entityId)'"
    }

    // MARK: - Camera

    func setCamera(position: [Float], lookAt: [Float]? = nil) -> String {
        guard let cameraNode = cameraNode else {
            return "Error: Camera not initialized"
        }

        let pos = position.count >= 3 ? SIMD3<Float>(position[0], position[1], position[2]) : SIMD3<Float>(0, 5, 10)
        cameraNode.position = SCNVector3(pos.x, pos.y, pos.z)

        if let look = lookAt, look.count >= 3 {
            cameraNode.look(at: SCNVector3(look[0], look[1], look[2]))
            worldState.camera.lookAt = SIMD3(look[0], look[1], look[2])
        }

        worldState.camera.position = pos
        return "Camera positioned at [\(pos.x), \(pos.y), \(pos.z)]"
    }

    // MARK: - Lighting

    func setLight(intensity: Float? = nil, color: String? = nil) -> String {
        guard let sun = sunLight else {
            return "Error: Light not initialized"
        }

        if let intensity = intensity {
            sun.intensity = CGFloat(intensity)
            worldState.lighting.intensity = intensity
        }

        if let col = color {
            let colorData = parseColor(col)
            sun.color = colorData.toNSColor()
            worldState.lighting.color = colorData
        }

        return "Light updated"
    }

    // MARK: - Scene Info

    func sceneInfo() -> String {
        let entityList = entityNames.map { name in
            if let entity = entities[name] {
                let behaviors = activeBehaviors[name]?.map { $0.type.rawValue } ?? []
                let behaviorStr = behaviors.isEmpty ? "" : " [\(behaviors.joined(separator: ", "))]"
                return "- \(name): \(entity.shape?.rawValue ?? "model") at [\(String(format: "%.1f", entity.position.x)), \(String(format: "%.1f", entity.position.y)), \(String(format: "%.1f", entity.position.z))]\(behaviorStr)"
            }
            return "- \(name)"
        }.joined(separator: "\n")

        return """
        Scene contains \(entities.count) entities:
        \(entityList.isEmpty ? "(empty)" : entityList)

        Camera: [\(String(format: "%.1f", worldState.camera.position.x)), \(String(format: "%.1f", worldState.camera.position.y)), \(String(format: "%.1f", worldState.camera.position.z))]
        """
    }

    func clearScene() -> String {
        for (_, node) in sceneEntities {
            node.removeFromParentNode()
        }
        sceneEntities.removeAll()
        entities.removeAll()
        entityNames.removeAll()
        worldState.entities.removeAll()
        activeBehaviors.removeAll()
        worldState.modifiedAt = Date()
        return "Scene cleared"
    }

    // MARK: - Behaviors

    func addBehavior(to entityId: String, type: BehaviorType, parameters: [String: Double] = [:]) -> String {
        guard entities[entityId] != nil,
              let node = sceneEntities[entityId] else {
            return "Error: Entity '\(entityId)' not found"
        }

        if var entity = entities[entityId] {
            let behavior = EntityBehavior(type: type, parameters: parameters)
            entity.behaviors.append(behavior)
            entities[entityId] = entity

            if let idx = worldState.entities.firstIndex(where: { $0.id == entityId }) {
                worldState.entities[idx].behaviors.append(behavior)
            }
        }

        var activeBehavior = ActiveBehavior(type: type, parameters: parameters)
        activeBehavior.baseY = Float(node.position.y)
        if activeBehaviors[entityId] == nil {
            activeBehaviors[entityId] = []
        }
        activeBehaviors[entityId]?.append(activeBehavior)

        return "Added \(type.rawValue) behavior to '\(entityId)'"
    }

    func removeBehavior(from entityId: String, type: BehaviorType) -> String {
        guard var entity = entities[entityId] else {
            return "Error: Entity '\(entityId)' not found"
        }

        entity.behaviors.removeAll { $0.type == type }
        entities[entityId] = entity
        activeBehaviors[entityId]?.removeAll { $0.type == type }

        if let idx = worldState.entities.firstIndex(where: { $0.id == entityId }) {
            worldState.entities[idx].behaviors.removeAll { $0.type == type }
        }

        return "Removed \(type.rawValue) behavior from '\(entityId)'"
    }

    func listBehaviors(for entityId: String) -> String {
        guard let entity = entities[entityId] else {
            return "Error: Entity '\(entityId)' not found"
        }

        if entity.behaviors.isEmpty {
            return "Entity '\(entityId)' has no behaviors"
        }

        let behaviorList = entity.behaviors.map { "- \($0.type.rawValue)" }.joined(separator: "\n")
        return "Behaviors for '\(entityId)':\n\(behaviorList)"
    }

    // MARK: - Behavior Updates

    private func updateBehaviors(dt: Float) {
        let time = Float(CACurrentMediaTime())

        for (entityId, behaviors) in activeBehaviors {
            guard let node = sceneEntities[entityId] else { continue }

            for i in 0..<behaviors.count {
                var behavior = behaviors[i]

                switch behavior.type {
                case .orbit:
                    updateOrbit(node: node, behavior: &behavior, time: time)
                case .spin:
                    updateSpin(node: node, behavior: &behavior, dt: dt)
                case .bob:
                    updateBob(node: node, behavior: &behavior, time: time)
                case .lookAt:
                    break // Placeholder
                case .pulse:
                    updatePulse(node: node, behavior: &behavior, time: time)
                case .pathFollow:
                    break // Placeholder
                case .bounce:
                    updateBounce(node: node, behavior: &behavior, dt: dt)
                }

                activeBehaviors[entityId]?[i] = behavior
            }
        }
    }

    private func updateOrbit(node: SCNNode, behavior: inout ActiveBehavior, time: Float) {
        let radius = Float(behavior.parameters["radius"] ?? 3.0)
        let speed = Float(behavior.parameters["speed"] ?? 1.0)

        let angle = time * speed
        let x = cos(angle) * radius
        let z = sin(angle) * radius

        let currentY = node.position.y
        node.position = SCNVector3(CGFloat(x), currentY, CGFloat(z))
    }

    private func updateSpin(node: SCNNode, behavior: inout ActiveBehavior, dt: Float) {
        let speed = Float(behavior.parameters["speed"] ?? 90.0)
        let axis = behavior.parameters["axis"] ?? 1

        let rotationAxis: SCNVector3
        if axis == 0 {
            rotationAxis = SCNVector3(1, 0, 0)
        } else if axis == 2 {
            rotationAxis = SCNVector3(0, 0, 1)
        } else {
            rotationAxis = SCNVector3(0, 1, 0)
        }

        let deltaRotation = SCNQuaternion.fromAxisAngle(axis: rotationAxis, angle: (speed * dt * Float.pi) / 180.0)
        node.localRotate(by: deltaRotation)
    }

    private func updateBob(node: SCNNode, behavior: inout ActiveBehavior, time: Float) {
        let amplitude = Float(behavior.parameters["amplitude"] ?? 0.5)
        let speed = Float(behavior.parameters["speed"] ?? 2.0)

        let offset: Float = sin(time * speed) * amplitude
        node.position.y = CGFloat(behavior.baseY + offset)
    }

    private func updatePulse(node: SCNNode, behavior: inout ActiveBehavior, time: Float) {
        let minScale = Float(behavior.parameters["min_scale"] ?? 0.8)
        let maxScale = Float(behavior.parameters["max_scale"] ?? 1.2)
        let speed = Float(behavior.parameters["speed"] ?? 2.0)

        let t: Float = (sin(time * speed) + 1) / 2
        let scale: Float = minScale + (maxScale - minScale) * t

        let cg = CGFloat(scale)
        node.scale = SCNVector3(cg, cg, cg)
    }

    private func updateBounce(node: SCNNode, behavior: inout ActiveBehavior, dt: Float) {
        let gravity = Float(behavior.parameters["gravity"] ?? -9.8)
        let bounceFactor = Float(behavior.parameters["bounce"] ?? 0.7)
        let floorY = Float(behavior.parameters["floor_y"] ?? 0.0)

        behavior.velocity += gravity * dt
        let newYFloat: Float = Float(node.position.y) + behavior.velocity * dt
        node.position.y = CGFloat(newYFloat)

        if newYFloat < floorY {
            node.position.y = CGFloat(floorY)
            behavior.velocity = -behavior.velocity * bounceFactor

            if abs(behavior.velocity) < 0.1 {
                behavior.velocity = 0
            }
        }
    }

    // MARK: - Save/Load

    func saveWorld(name: String) -> String {
        do {
            _ = try worldState.save(name: name)
            return "World '\(name)' saved successfully"
        } catch {
            handleError("Failed to save world: \(error.localizedDescription)")
            return "Error: Failed to save world"
        }
    }

    func loadWorld(name: String, clearCurrent: Bool = true) -> String {
        do {
            let loadedState = try WorldState.load(name: name)

            if clearCurrent {
                _ = clearScene()
            }

            worldState = loadedState

            for entity in loadedState.entities {
                spawnPrimitive(
                    shape: entity.shape?.rawValue ?? "cube",
                    position: [entity.position.x, entity.position.y, entity.position.z],
                    color: "gray",
                    scale: [entity.scale.x, entity.scale.y, entity.scale.z],
                    rotation: [entity.rotation.x, entity.rotation.y, entity.rotation.z],
                    name: entity.name
                )

                if let node = sceneEntities[entity.name] {
                    let material = SCNMaterial()
                    material.diffuse.contents = entity.color.toNSColor()
                    material.lightingModel = .physicallyBased
                    node.geometry?.materials = [material]
                }

                for behavior in entity.behaviors {
                    _ = addBehavior(to: entity.name, type: behavior.type, parameters: behavior.parameters)
                }
            }

            _ = setCamera(
                position: [loadedState.camera.position.x, loadedState.camera.position.y, loadedState.camera.position.z],
                lookAt: [loadedState.camera.lookAt.x, loadedState.camera.lookAt.y, loadedState.camera.lookAt.z]
            )

            return "World '\(name)' loaded successfully with \(loadedState.entities.count) entities"
        } catch {
            handleError("Failed to load world: \(error.localizedDescription)")
            return "Error: Failed to load world"
        }
    }

    func listWorlds() -> String {
        do {
            let worlds = try WorldState.listAll()
            if worlds.isEmpty {
                return "No saved worlds found"
            }
            return "Saved worlds:\n" + worlds.map { "- \($0)" }.joined(separator: "\n")
        } catch {
            return "Error listing worlds: \(error.localizedDescription)"
        }
    }

    // MARK: - Helpers

    private func generateEntityName(for shape: PrimitiveShape) -> String {
        entityCounter += 1
        return "\(shape.rawValue)_\(entityCounter)"
    }

    private func handleError(_ message: String) {
        lastError = message
        showError = true
    }

    deinit {
        if let link = displayLink {
            CVDisplayLinkStop(link)
        }
    }
}

// MARK: - SCNQuaternion Extension

extension SCNQuaternion {
    static func fromAxisAngle(axis: SCNVector3, angle: Float) -> SCNQuaternion {
        let halfAngle: Float = angle / 2
        let s: Float = sin(halfAngle)
        let x: Float = Float(axis.x) * s
        let y: Float = Float(axis.y) * s
        let z: Float = Float(axis.z) * s
        let w: Float = cos(halfAngle)
        return SCNQuaternion(x, y, z, w)
    }
}

#endif

