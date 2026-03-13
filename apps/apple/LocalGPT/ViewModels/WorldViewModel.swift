import Foundation
import RealityKit
import Combine
import ARKit

/// ViewModel managing the RealityKit scene for world generation.
@MainActor
class WorldViewModel: ObservableObject {
    // MARK: - Published State

    @Published var entities: [String: WorldEntity] = [:]
    @Published var entityNames: [String] = []
    @Published var worldState = WorldState()
    @Published var isProcessing = false
    @Published var lastError: String?
    @Published var showError = false

    // MARK: - RealityKit Components

    var arView: ARView?
    let rootEntity = AnchorEntity(world: .zero)
    private var realityEntities: [String: Entity] = [:]
    private var cameraEntity: Entity?
    private var sunLight: DirectionalLight?

    // MARK: - Services

    let behaviorService = WorldBehaviorService()
    let audioService = WorldAudioService()

    // MARK: - Behavior State

    private var activeBehaviors: [String: [ActiveBehavior]] = [:]
    private var updateTimer: Timer?

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
        startBehaviorUpdateLoop()
    }

    private func startBehaviorUpdateLoop() {
        // Run behavior updates at 60fps
        updateTimer = Timer.scheduledTimer(withTimeInterval: 1.0 / 60.0, repeats: true) { [weak self] _ in
            Task { @MainActor [weak self] in
                self?.updateBehaviorsInternal()
            }
        }
    }

    func setupARView(_ arView: ARView) {
        self.arView = arView

        // Configure for virtual world (no AR tracking)
        let config = ARWorldTrackingConfiguration()
        config.planeDetection = []
        arView.session.run(config, options: [.resetTracking, .removeExistingAnchors])

        // Add root entity to scene
        arView.scene.addAnchor(rootEntity)

        // Setup camera
        setupCamera()

        // Add grid floor for reference
        addGridFloor()

        // Add sun light to scene
        if let sun = sunLight {
            rootEntity.addChild(sun)
        }
    }

    private func setupDefaultLighting() {
        // Create directional sun light
        let sun = DirectionalLight()
        sun.light.intensity = 2000
        sun.light.color = .white
        sun.look(at: [0, -1, -0.5], from: [5, 10, 5], relativeTo: nil)
        sunLight = sun
    }

    private func setupCamera() {
        guard arView != nil else { return }

        let camera = PerspectiveCamera()
        camera.position = worldState.camera.position
        camera.look(at: worldState.camera.lookAt, from: worldState.camera.position, relativeTo: nil)
        cameraEntity = camera

        rootEntity.addChild(camera)
    }

    private func addGridFloor() {
        // Add a simple floor plane
        let floorMesh = MeshResource.generatePlane(width: 100, depth: 100)
        let floorMaterial = SimpleMaterial(color: .init(white: 0.2, alpha: 0.5), isMetallic: false)
        let floor = ModelEntity(mesh: floorMesh, materials: [floorMaterial])
        floor.position = [0, -0.01, 0]
        rootEntity.addChild(floor)
    }

    // MARK: - Entity Management Tools

    /// Spawn a primitive shape in the world
    /// - Parameters:
    ///   - shape: Shape type (cube, sphere, cylinder, cone, plane, capsule, pyramid, torus)
    ///   - position: [x, y, z] position
    ///   - color: Color name or hex code
    ///   - scale: Optional [x, y, z] scale (default: [1, 1, 1])
    ///   - rotation: Optional [x, y, z] rotation in degrees (default: [0, 0, 0])
    ///   - name: Optional entity name (auto-generated if not provided)
    /// - Returns: Entity ID
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
            handleError("Unknown shape type: \(shape). Available: \(PrimitiveShape.allCases.map { $0.rawValue }.joined(separator: ", "))")
            return ""
        }

        let entityId = name ?? generateEntityName(for: shapeType)
        let pos = simd_float3(position.count >= 3 ? SIMD3(position[0], position[1], position[2]) : .zero)
        let scl = scale != nil && scale!.count >= 3 ? SIMD3(scale![0], scale![1], scale![2]) : SIMD3<Float>(1, 1, 1)
        let rot = rotation != nil && rotation!.count >= 3 ? SIMD3(rotation![0], rotation![1], rotation![2]) : SIMD3<Float>(0, 0, 0)

        // Create mesh based on shape
        let mesh: MeshResource
        switch shapeType {
        case .cube:
            mesh = MeshResource.generateBox(size: 1)
        case .sphere:
            mesh = MeshResource.generateSphere(radius: 0.5)
        case .cylinder:
            mesh = MeshResource.generateCylinder(height: 1, radius: 0.5)
        case .cone:
            mesh = MeshResource.generateCone(height: 1, radius: 0.5)
        case .plane:
            mesh = MeshResource.generatePlane(width: 1, depth: 1)
        case .capsule:
            // RealityKit doesn't have generateCapsule, use cylinder as approximation
            mesh = MeshResource.generateCylinder(height: 1, radius: 0.25)
        case .pyramid:
            mesh = MeshResource.generatePyramid(baseSize: 1, height: 1)
        case .torus:
            mesh = MeshResource.generateTorus(ringRadius: 0.5, pipeRadius: 0.2)
        }

        // Create material with color
        let colorData = parseColor(color)
        let material = SimpleMaterial(color: colorData.toUIColor(), isMetallic: false)

        // Create entity
        let modelEntity = ModelEntity(mesh: mesh, materials: [material])
        modelEntity.position = pos
        modelEntity.scale = scl
        modelEntity.orientation = simd_quatf(
            angle: rot.x * .pi / 180, axis: [1, 0, 0]
        ) * simd_quatf(
            angle: rot.y * .pi / 180, axis: [0, 1, 0]
        ) * simd_quatf(
            angle: rot.z * .pi / 180, axis: [0, 0, 1]
        )
        modelEntity.name = entityId

        // Add to scene
        rootEntity.addChild(modelEntity)
        realityEntities[entityId] = modelEntity

        // Track in data model
        let worldEntity = WorldEntity(
            id: entityId,
            name: entityId,
            shape: shapeType,
            position: SIMD3(pos.x, pos.y, pos.z),
            scale: SIMD3(scl.x, scl.y, scl.z),
            rotation: SIMD3(rot.x, rot.y, rot.z),
            color: colorData
        )
        entities[entityId] = worldEntity
        entityNames.append(entityId)
        worldState.entities.append(worldEntity)
        worldState.modifiedAt = Date()

        return entityId
    }

    /// Load a 3D model from a URL (supports USDZ, usdc, reality)
    /// - Parameters:
    ///   - url: URL to the model file
    ///   - position: [x, y, z] position
    ///   - scale: Optional [x, y, z] scale
    ///   - name: Optional entity name
    /// - Returns: Entity ID or error message
    @discardableResult
    func loadModel(
        from url: URL,
        position: [Float],
        scale: [Float]? = nil,
        name: String? = nil
    ) async -> String {
        let entityId = name ?? "model_\(entityCounter + 1)"
        let pos = simd_float3(position.count >= 3 ? SIMD3(position[0], position[1], position[2]) : .zero)
        let scl = scale != nil && scale!.count >= 3 ? SIMD3(scale![0], scale![1], scale![2]) : SIMD3<Float>(1, 1, 1)

        isProcessing = true
        defer { isProcessing = false }

        do {
            // Load entity from file
            let loadedEntity = try await Entity(contentsOf: url)
            loadedEntity.position = pos
            loadedEntity.scale = scl
            loadedEntity.name = entityId

            // Add to scene
            rootEntity.addChild(loadedEntity)
            realityEntities[entityId] = loadedEntity

            // Track in data model
            let worldEntity = WorldEntity(
                id: entityId,
                name: entityId,
                shape: nil, // Model, not primitive
                position: SIMD3(pos.x, pos.y, pos.z),
                scale: SIMD3(scl.x, scl.y, scl.z),
                rotation: .zero,
                color: ColorData(r: 0.5, g: 0.5, b: 0.5),
                isModelEntity: true,
                modelPath: url.path
            )
            entities[entityId] = worldEntity
            entityNames.append(entityId)
            worldState.entities.append(worldEntity)
            worldState.modifiedAt = Date()

            entityCounter += 1
            return "Loaded model '\(entityId)' from \(url.lastPathComponent)"
        } catch {
            handleError("Failed to load model: \(error.localizedDescription)")
            return "Error: Failed to load model - \(error.localizedDescription)"
        }
    }

    /// Load a model from app bundle
    /// - Parameters:
    ///   - named: Resource name in bundle
    ///   - position: [x, y, z] position
    ///   - scale: Optional [x, y, z] scale
    ///   - name: Optional entity name
    /// - Returns: Entity ID or error message
    @discardableResult
    func loadBundleModel(
        named resourceName: String,
        position: [Float],
        scale: [Float]? = nil,
        name: String? = nil
    ) async -> String {
        guard let url = Bundle.main.url(forResource: resourceName, withExtension: nil) else {
            return "Error: Model '\(resourceName)' not found in bundle"
        }
        return await loadModel(from: url, position: position, scale: scale, name: name)
    }

    /// Modify an existing entity
    /// - Parameters:
    ///   - entityId: Entity to modify
    ///   - position: Optional new [x, y, z] position
    ///   - scale: Optional new [x, y, z] scale
    ///   - rotation: Optional new [x, y, z] rotation in degrees
    ///   - color: Optional new color
    /// - Returns: Success message
    @discardableResult
    func modifyEntity(
        entityId: String,
        position: [Float]? = nil,
        scale: [Float]? = nil,
        rotation: [Float]? = nil,
        color: String? = nil
    ) -> String {
        guard var worldEntity = entities[entityId],
              let realityEntity = realityEntities[entityId] as? ModelEntity else {
            handleError("Entity '\(entityId)' not found")
            return "Error: Entity '\(entityId)' not found"
        }

        // Update position
        if let pos = position, pos.count >= 3 {
            let newPos = simd_float3(pos[0], pos[1], pos[2])
            realityEntity.position = newPos
            worldEntity.position = SIMD3(pos[0], pos[1], pos[2])
        }

        // Update scale
        if let scl = scale, scl.count >= 3 {
            let newScale = simd_float3(scl[0], scl[1], scl[2])
            realityEntity.scale = newScale
            worldEntity.scale = SIMD3(scl[0], scl[1], scl[2])
        }

        // Update rotation
        if let rot = rotation, rot.count >= 3 {
            realityEntity.orientation = simd_quatf(
                angle: rot[0] * .pi / 180, axis: [1, 0, 0]
            ) * simd_quatf(
                angle: rot[1] * .pi / 180, axis: [0, 1, 0]
            ) * simd_quatf(
                angle: rot[2] * .pi / 180, axis: [0, 0, 1]
            )
            worldEntity.rotation = SIMD3(rot[0], rot[1], rot[2])
        }

        // Update color
        if let col = color {
            let colorData = parseColor(col)
            let material = SimpleMaterial(color: colorData.toUIColor(), isMetallic: false)
            realityEntity.model?.materials = [material]
            worldEntity.color = colorData
        }

        // Update stored entity
        entities[entityId] = worldEntity
        if let idx = worldState.entities.firstIndex(where: { $0.id == entityId }) {
            worldState.entities[idx] = worldEntity
        }
        worldState.modifiedAt = Date()

        return "Modified entity '\(entityId)'"
    }

    /// Delete an entity from the scene
    /// - Parameter entityId: Entity to delete
    func deleteEntity(_ entityId: String) -> String {
        guard let realityEntity = realityEntities[entityId] else {
            return "Error: Entity '\(entityId)' not found"
        }

        // Remove behaviors
        activeBehaviors.removeValue(forKey: entityId)

        // Remove audio emitter
        audioService.removeAudioEmitter(id: entityId)

        realityEntity.removeFromParent()
        realityEntities.removeValue(forKey: entityId)
        entities.removeValue(forKey: entityId)
        entityNames.removeAll { $0 == entityId }
        worldState.entities.removeAll { $0.id == entityId }
        worldState.modifiedAt = Date()

        return "Deleted entity '\(entityId)'"
    }

    // MARK: - Camera Tools

    /// Set camera position and orientation
    func setCamera(position: [Float], lookAt: [Float]? = nil) -> String {
        guard let camera = cameraEntity as? PerspectiveCamera else {
            return "Error: Camera not initialized"
        }

        let pos = position.count >= 3 ? simd_float3(position[0], position[1], position[2]) : simd_float3(0, 5, 10)
        camera.position = pos

        if let look = lookAt, look.count >= 3 {
            let target = simd_float3(look[0], look[1], look[2])
            camera.look(at: target, from: pos, relativeTo: nil)
            worldState.camera.lookAt = SIMD3(look[0], look[1], look[2])
        }

        worldState.camera.position = SIMD3(pos.x, pos.y, pos.z)

        // Update audio listener position
        audioService.setListenerPosition(pos, orientation: camera.orientation)

        return "Camera positioned at [\(pos.x), \(pos.y), \(pos.z)]"
    }

    // MARK: - Lighting Tools

    /// Set lighting intensity and color
    func setLight(intensity: Float? = nil, color: String? = nil) -> String {
        guard let sun = sunLight else {
            return "Error: Light not initialized"
        }

        if let intensity = intensity {
            sun.light.intensity = intensity
            worldState.lighting.intensity = intensity
        }

        if let col = color {
            let colorData = parseColor(col)
            sun.light.color = colorData.toUIColor()
            worldState.lighting.color = colorData
        }

        return "Light updated"
    }

    // MARK: - Scene Tools

    /// Get scene information
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
        Ambience: \(worldState.ambience.type.rawValue)
        """
    }

    /// Clear all entities from the scene
    func clearScene() -> String {
        for (_, entity) in realityEntities {
            entity.removeFromParent()
        }
        realityEntities.removeAll()
        entities.removeAll()
        entityNames.removeAll()
        worldState.entities.removeAll()
        activeBehaviors.removeAll()
        worldState.modifiedAt = Date()

        return "Scene cleared"
    }

    // MARK: - Behavior Tools

    /// Add a behavior to an entity
    func addBehavior(to entityId: String, type: BehaviorType, parameters: [String: Double] = [:]) -> String {
        guard entities[entityId] != nil,
              let realityEntity = realityEntities[entityId] else {
            return "Error: Entity '\(entityId)' not found"
        }

        // Store behavior in entity data model
        if var entity = entities[entityId] {
            let behavior = EntityBehavior(type: type, parameters: parameters)
            entity.behaviors.append(behavior)
            entities[entityId] = entity

            // Update world state
            if let idx = worldState.entities.firstIndex(where: { $0.id == entityId }) {
                worldState.entities[idx].behaviors.append(behavior)
            }
        }

        // Create active behavior for animation loop
        var activeBehavior = ActiveBehavior(type: type, parameters: parameters)
        activeBehavior.baseY = realityEntity.position.y
        if activeBehaviors[entityId] == nil {
            activeBehaviors[entityId] = []
        }
        activeBehaviors[entityId]?.append(activeBehavior)

        return "Added \(type.rawValue) behavior to '\(entityId)'"
    }

    /// Remove a behavior from an entity
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

    /// List behaviors for an entity
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

    // MARK: - Behavior Update Loop

    private func updateBehaviorsInternal() {
        let time = Float(Date().timeIntervalSinceReferenceDate)

        for (entityId, behaviors) in activeBehaviors {
            guard let entity = realityEntities[entityId] else { continue }

            for i in 0..<behaviors.count {
                var behavior = behaviors[i]
                let dt: Float = 1.0 / 60.0

                switch behavior.type {
                case .orbit:
                    updateOrbit(entity: entity, behavior: &behavior, time: time)

                case .spin:
                    updateSpin(entity: entity, behavior: &behavior, dt: dt)

                case .bob:
                    updateBob(entity: entity, behavior: &behavior, time: time)

                case .lookAt:
                    updateLookAt(entity: entity, behavior: &behavior)

                case .pulse:
                    updatePulse(entity: entity, behavior: &behavior, time: time)

                case .pathFollow:
                    updatePathFollow(entity: entity, behavior: &behavior, time: time)

                case .bounce:
                    updateBounce(entity: entity, behavior: &behavior, dt: dt)
                }

                activeBehaviors[entityId]?[i] = behavior
            }
        }

        // Update audio listener position
        if let camera = cameraEntity {
            audioService.setListenerPosition(camera.position, orientation: camera.orientation)
        }
    }

    private func updateOrbit(entity: Entity, behavior: inout ActiveBehavior, time: Float) {
        let radius = Float(behavior.parameters["radius"] ?? 3.0)
        let speed = Float(behavior.parameters["speed"] ?? 1.0)

        let angle = time * speed
        let x = cos(angle) * radius
        let z = sin(angle) * radius

        entity.position = [x, entity.position.y, z]
    }

    private func updateSpin(entity: Entity, behavior: inout ActiveBehavior, dt: Float) {
        let speed = Float(behavior.parameters["speed"] ?? 90.0) // degrees per second
        let axis = behavior.parameters["axis"] ?? 1 // 0=x, 1=y, 2=z

        let rotationAxis: SIMD3<Float>
        if axis == 0 {
            rotationAxis = [1, 0, 0]
        } else if axis == 2 {
            rotationAxis = [0, 0, 1]
        } else {
            rotationAxis = [0, 1, 0]
        }

        let deltaRotation = simd_quatf(angle: speed * dt * .pi / 180, axis: rotationAxis)
        entity.orientation = entity.orientation * deltaRotation
    }

    private func updateBob(entity: Entity, behavior: inout ActiveBehavior, time: Float) {
        let amplitude = Float(behavior.parameters["amplitude"] ?? 0.5)
        let speed = Float(behavior.parameters["speed"] ?? 2.0)

        let offset = sin(time * speed) * amplitude
        entity.position.y = behavior.baseY + offset
    }

    private func updateLookAt(entity: Entity, behavior: inout ActiveBehavior) {
        // Look at would require another entity reference
        // For now, this is a placeholder
    }

    private func updatePulse(entity: Entity, behavior: inout ActiveBehavior, time: Float) {
        let minScale = Float(behavior.parameters["min_scale"] ?? 0.8)
        let maxScale = Float(behavior.parameters["max_scale"] ?? 1.2)
        let speed = Float(behavior.parameters["speed"] ?? 2.0)

        let t = (sin(time * speed) + 1) / 2 // 0 to 1
        let scale = minScale + (maxScale - minScale) * t

        entity.scale = SIMD3<Float>(repeating: scale)
    }

    private func updatePathFollow(entity: Entity, behavior: inout ActiveBehavior, time: Float) {
        let speed = Float(behavior.parameters["speed"] ?? 1.0)
        behavior.phase += speed / 60.0
        if behavior.phase > 1 {
            behavior.phase = 0
        }
    }

    private func updateBounce(entity: Entity, behavior: inout ActiveBehavior, dt: Float) {
        let gravity = Float(behavior.parameters["gravity"] ?? -9.8)
        let bounceFactor = Float(behavior.parameters["bounce"] ?? 0.7)
        let floorY = Float(behavior.parameters["floor_y"] ?? 0.0)

        behavior.velocity += gravity * dt
        entity.position.y += behavior.velocity * dt

        if entity.position.y < floorY {
            entity.position.y = floorY
            behavior.velocity = -behavior.velocity * bounceFactor

            // Stop if velocity is too small
            if abs(behavior.velocity) < 0.1 {
                behavior.velocity = 0
            }
        }
    }

    // MARK: - Audio Tools

    /// Set ambient sound
    func setAmbience(_ type: AmbienceType, volume: Float = 0.5) -> String {
        worldState.ambience = AmbienceState(type: type, volume: volume)
        audioService.setAmbience(type, volume: volume)
        return "Set ambience to \(type.rawValue)"
    }

    // MARK: - World Save/Load

    /// Save current world state
    func saveWorld(name: String) -> String {
        do {
            _ = try worldState.save(name: name)
            return "World '\(name)' saved successfully"
        } catch {
            handleError("Failed to save world: \(error.localizedDescription)")
            return "Error: Failed to save world"
        }
    }

    /// Load a saved world
    func loadWorld(name: String, clearCurrent: Bool = true) -> String {
        do {
            let loadedState = try WorldState.load(name: name)

            if clearCurrent {
                _ = clearScene()
            }

            worldState = loadedState

            // Recreate entities
            for entity in loadedState.entities {
                spawnPrimitive(
                    shape: entity.shape?.rawValue ?? "cube",
                    position: [entity.position.x, entity.position.y, entity.position.z],
                    color: "gray", // Will be updated
                    scale: [entity.scale.x, entity.scale.y, entity.scale.z],
                    rotation: [entity.rotation.x, entity.rotation.y, entity.rotation.z],
                    name: entity.name
                )

                // Update color separately
                if let modelEntity = realityEntities[entity.name] as? ModelEntity {
                    let material = SimpleMaterial(color: entity.color.toUIColor(), isMetallic: false)
                    modelEntity.model?.materials = [material]
                }

                // Restore behaviors
                for behavior in entity.behaviors {
                    _ = addBehavior(to: entity.name, type: behavior.type, parameters: behavior.parameters)
                }
            }

            // Restore camera
            _ = setCamera(
                position: [loadedState.camera.position.x, loadedState.camera.position.y, loadedState.camera.position.z],
                lookAt: [loadedState.camera.lookAt.x, loadedState.camera.lookAt.y, loadedState.camera.lookAt.z]
            )

            // Restore ambience
            _ = setAmbience(loadedState.ambience.type, volume: loadedState.ambience.volume)

            return "World '\(name)' loaded successfully with \(loadedState.entities.count) entities"
        } catch {
            handleError("Failed to load world: \(error.localizedDescription)")
            return "Error: Failed to load world"
        }
    }

    /// List all saved worlds
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
        updateTimer?.invalidate()
    }
}
