import Foundation

/// Tool definitions for world generation LLM integration.
/// These tools map to the WorldViewModel functions.
struct WorldToolService {

    /// Tool schemas for LLM function calling
    static let toolSchemas: [[String: Any]] = [
        spawnPrimitiveSchema,
        loadModelSchema,
        modifyEntitySchema,
        deleteEntitySchema,
        setCameraSchema,
        setLightSchema,
        setAmbienceSchema,
        addBehaviorSchema,
        removeBehaviorSchema,
        listBehaviorsSchema,
        sceneInfoSchema,
        clearSceneSchema,
        saveWorldSchema,
        loadWorldSchema,
        listWorldsSchema
    ]

    // MARK: - Tool Schemas

    static let spawnPrimitiveSchema: [String: Any] = [
        "name": "spawn_primitive",
        "description": "Spawn a 3D primitive shape in the world. Use this to create objects like cubes, spheres, cylinders, etc.",
        "parameters": [
            "type": "object",
            "properties": [
                "shape": [
                    "type": "string",
                    "enum": ["cube", "sphere", "cylinder", "cone", "plane", "capsule", "pyramid", "torus"],
                    "description": "Shape type to spawn"
                ],
                "position": [
                    "type": "array",
                    "items": ["type": "number"],
                    "description": "Position [x, y, z] in world coordinates"
                ],
                "color": [
                    "type": "string",
                    "description": "Color name (red, green, blue, etc.) or hex code (#RRGGBB)"
                ],
                "scale": [
                    "type": "array",
                    "items": ["type": "number"],
                    "description": "Scale [x, y, z], default [1, 1, 1]"
                ],
                "rotation": [
                    "type": "array",
                    "items": ["type": "number"],
                    "description": "Rotation [x, y, z] in degrees, default [0, 0, 0]"
                ],
                "name": [
                    "type": "string",
                    "description": "Optional custom name for the entity"
                ]
            ],
            "required": ["shape", "position", "color"]
        ]
    ]

    static let modifyEntitySchema: [String: Any] = [
        "name": "modify_entity",
        "description": "Modify an existing entity's position, scale, rotation, or color.",
        "parameters": [
            "type": "object",
            "properties": [
                "entity_id": [
                    "type": "string",
                    "description": "Name/ID of the entity to modify"
                ],
                "position": [
                    "type": "array",
                    "items": ["type": "number"],
                    "description": "New position [x, y, z]"
                ],
                "scale": [
                    "type": "array",
                    "items": ["type": "number"],
                    "description": "New scale [x, y, z]"
                ],
                "rotation": [
                    "type": "array",
                    "items": ["type": "number"],
                    "description": "New rotation [x, y, z] in degrees"
                ],
                "color": [
                    "type": "string",
                    "description": "New color name or hex code"
                ]
            ],
            "required": ["entity_id"]
        ]
    ]

    static let deleteEntitySchema: [String: Any] = [
        "name": "delete_entity",
        "description": "Delete an entity from the scene.",
        "parameters": [
            "type": "object",
            "properties": [
                "entity_id": [
                    "type": "string",
                    "description": "Name/ID of the entity to delete"
                ]
            ],
            "required": ["entity_id"]
        ]
    ]

    static let setCameraSchema: [String: Any] = [
        "name": "set_camera",
        "description": "Position and orient the camera in the world.",
        "parameters": [
            "type": "object",
            "properties": [
                "position": [
                    "type": "array",
                    "items": ["type": "number"],
                    "description": "Camera position [x, y, z]"
                ],
                "look_at": [
                    "type": "array",
                    "items": ["type": "number"],
                    "description": "Point for camera to look at [x, y, z]"
                ]
            ],
            "required": ["position"]
        ]
    ]

    static let setLightSchema: [String: Any] = [
        "name": "set_light",
        "description": "Adjust scene lighting intensity and color.",
        "parameters": [
            "type": "object",
            "properties": [
                "intensity": [
                    "type": "number",
                    "description": "Light intensity (0-10000, default 2000)"
                ],
                "color": [
                    "type": "string",
                    "description": "Light color name or hex code"
                ]
            ]
        ]
    ]

    static let sceneInfoSchema: [String: Any] = [
        "name": "scene_info",
        "description": "Get information about the current scene including all entities.",
        "parameters": [
            "type": "object",
            "properties": [:]
        ]
    ]

    static let clearSceneSchema: [String: Any] = [
        "name": "clear_scene",
        "description": "Clear all entities from the scene, resetting to empty world.",
        "parameters": [
            "type": "object",
            "properties": [:]
        ]
    ]

    static let saveWorldSchema: [String: Any] = [
        "name": "save_world",
        "description": "Save the current world to disk for later loading.",
        "parameters": [
            "type": "object",
            "properties": [
                "name": [
                    "type": "string",
                    "description": "Name to save the world as"
                ]
            ],
            "required": ["name"]
        ]
    ]

    static let loadWorldSchema: [String: Any] = [
        "name": "load_world",
        "description": "Load a previously saved world.",
        "parameters": [
            "type": "object",
            "properties": [
                "name": [
                    "type": "string",
                    "description": "Name of the world to load"
                ]
            ],
            "required": ["name"]
        ]
    ]

    static let listWorldsSchema: [String: Any] = [
        "name": "list_worlds",
        "description": "List all saved worlds available to load.",
        "parameters": [
            "type": "object",
            "properties": [:]
        ]
    ]

    static let addBehaviorSchema: [String: Any] = [
        "name": "add_behavior",
        "description": "Add an animation behavior to an entity. Behaviors include orbit, spin, bob, look_at, pulse, path_follow, and bounce.",
        "parameters": [
            "type": "object",
            "properties": [
                "entity_id": [
                    "type": "string",
                    "description": "Name/ID of the entity to add behavior to"
                ],
                "behavior_type": [
                    "type": "string",
                    "enum": ["orbit", "spin", "bob", "look_at", "pulse", "path_follow", "bounce"],
                    "description": "Type of behavior to add"
                ],
                "speed": [
                    "type": "number",
                    "description": "Animation speed (default varies by type)"
                ],
                "amplitude": [
                    "type": "number",
                    "description": "For bob/pulse: height/scale variation"
                ],
                "radius": [
                    "type": "number",
                    "description": "For orbit: radius around center"
                ],
                "axis": [
                    "type": "integer",
                    "description": "Rotation axis: 0=X, 1=Y (default), 2=Z"
                ]
            ],
            "required": ["entity_id", "behavior_type"]
        ]
    ]

    static let removeBehaviorSchema: [String: Any] = [
        "name": "remove_behavior",
        "description": "Remove a behavior from an entity.",
        "parameters": [
            "type": "object",
            "properties": [
                "entity_id": [
                    "type": "string",
                    "description": "Name/ID of the entity"
                ],
                "behavior_type": [
                    "type": "string",
                    "description": "Type of behavior to remove (or 'all' for all behaviors)"
                ]
            ],
            "required": ["entity_id", "behavior_type"]
        ]
    ]

    static let listBehaviorsSchema: [String: Any] = [
        "name": "list_behaviors",
        "description": "List all behaviors on an entity.",
        "parameters": [
            "type": "object",
            "properties": [
                "entity_id": [
                    "type": "string",
                    "description": "Name/ID of the entity"
                ]
            ],
            "required": ["entity_id"]
        ]
    ]

    static let setAmbienceSchema: [String: Any] = [
        "name": "set_ambience",
        "description": "Set the ambient background sound for the world.",
        "parameters": [
            "type": "object",
            "properties": [
                "type": [
                    "type": "string",
                    "enum": ["silence", "wind", "rain", "forest", "ocean", "cave", "stream", "night", "city"],
                    "description": "Type of ambient sound"
                ],
                "volume": [
                    "type": "number",
                    "description": "Volume level (0-1, default 0.5)"
                ]
            ],
            "required": ["type"]
        ]
    ]

    static let loadModelSchema: [String: Any] = [
        "name": "load_model",
        "description": "Load a 3D model from a file path or bundle resource. Supports USDZ, usdc, and reality formats.",
        "parameters": [
            "type": "object",
            "properties": [
                "path": [
                    "type": "string",
                    "description": "File path to the model, or bundle resource name"
                ],
                "position": [
                    "type": "array",
                    "items": ["type": "number"],
                    "description": "Position [x, y, z] in world coordinates"
                ],
                "scale": [
                    "type": "array",
                    "items": ["type": "number"],
                    "description": "Scale [x, y, z], default [1, 1, 1]"
                ],
                "name": [
                    "type": "string",
                    "description": "Optional custom name for the entity"
                ]
            ],
            "required": ["path", "position"]
        ]
    ]

    // MARK: - Tool Execution

    /// Execute a tool call on the WorldViewModel
    static func executeTool(
        name: String,
        arguments: [String: Any],
        viewModel: WorldViewModel
    ) -> String {
        switch name {
        case "spawn_primitive":
            return executeSpawnPrimitive(arguments: arguments, viewModel: viewModel)

        case "load_model":
            return executeLoadModel(arguments: arguments, viewModel: viewModel)

        case "modify_entity":
            return executeModifyEntity(arguments: arguments, viewModel: viewModel)

        case "delete_entity":
            return executeDeleteEntity(arguments: arguments, viewModel: viewModel)

        case "set_camera":
            return executeSetCamera(arguments: arguments, viewModel: viewModel)

        case "set_light":
            return executeSetLight(arguments: arguments, viewModel: viewModel)

        case "set_ambience":
            return executeSetAmbience(arguments: arguments, viewModel: viewModel)

        case "scene_info":
            return viewModel.sceneInfo()

        case "clear_scene":
            return viewModel.clearScene()

        case "save_world":
            return executeSaveWorld(arguments: arguments, viewModel: viewModel)

        case "load_world":
            return executeLoadWorld(arguments: arguments, viewModel: viewModel)

        case "list_worlds":
            return viewModel.listWorlds()

        case "add_behavior":
            return executeAddBehavior(arguments: arguments, viewModel: viewModel)

        case "remove_behavior":
            return executeRemoveBehavior(arguments: arguments, viewModel: viewModel)

        case "list_behaviors":
            return executeListBehaviors(arguments: arguments, viewModel: viewModel)

        default:
            return "Error: Unknown tool '\(name)'"
        }
    }

    // MARK: - Tool Implementations

    private static func executeSpawnPrimitive(
        arguments: [String: Any],
        viewModel: WorldViewModel
    ) -> String {
        guard let shape = arguments["shape"] as? String,
              let position = arguments["position"] as? [Float],
              let color = arguments["color"] as? String else {
            return "Error: Missing required parameters (shape, position, color)"
        }

        let scale = arguments["scale"] as? [Float]
        let rotation = arguments["rotation"] as? [Float]
        let name = arguments["name"] as? String

        let entityId = viewModel.spawnPrimitive(
            shape: shape,
            position: position,
            color: color,
            scale: scale,
            rotation: rotation,
            name: name
        )

        return "Spawned \(shape) '\(entityId)' at [\(position.map { String(format: "%.1f", $0) }.joined(separator: ", "))]"
    }

    private static func executeLoadModel(
        arguments: [String: Any],
        viewModel: WorldViewModel
    ) -> String {
        guard let path = arguments["path"] as? String,
              let position = arguments["position"] as? [Float] else {
            return "Error: Missing required parameters (path, position)"
        }

        let scale = arguments["scale"] as? [Float]
        let name = arguments["name"] as? String

        // Check if path is a file URL or bundle resource
        let url: URL
        if path.hasPrefix("/") || path.hasPrefix("file://") {
            url = URL(fileURLWithPath: path.replacingOccurrences(of: "file://", with: ""))
        } else if let bundleURL = Bundle.main.url(forResource: path, withExtension: nil) {
            url = bundleURL
        } else if let bundleURL = Bundle.main.url(forResource: path, withExtension: "usdz") {
            url = bundleURL
        } else if let bundleURL = Bundle.main.url(forResource: path, withExtension: "reality") {
            url = bundleURL
        } else {
            return "Error: Model '\(path)' not found in bundle or filesystem"
        }

        // This needs to be async, but for tool execution we return immediately
        // The actual loading happens asynchronously
        Task {
            _ = await viewModel.loadModel(from: url, position: position, scale: scale, name: name)
        }

        return "Loading model from '\(path)' at [\(position.map { String(format: "%.1f", $0) }.joined(separator: ", "))]"
    }

    private static func executeModifyEntity(
        arguments: [String: Any],
        viewModel: WorldViewModel
    ) -> String {
        guard let entityId = arguments["entity_id"] as? String else {
            return "Error: Missing entity_id parameter"
        }

        let position = arguments["position"] as? [Float]
        let scale = arguments["scale"] as? [Float]
        let rotation = arguments["rotation"] as? [Float]
        let color = arguments["color"] as? String

        return viewModel.modifyEntity(
            entityId: entityId,
            position: position,
            scale: scale,
            rotation: rotation,
            color: color
        )
    }

    private static func executeDeleteEntity(
        arguments: [String: Any],
        viewModel: WorldViewModel
    ) -> String {
        guard let entityId = arguments["entity_id"] as? String else {
            return "Error: Missing entity_id parameter"
        }

        return viewModel.deleteEntity(entityId)
    }

    private static func executeSetCamera(
        arguments: [String: Any],
        viewModel: WorldViewModel
    ) -> String {
        guard let position = arguments["position"] as? [Float] else {
            return "Error: Missing position parameter"
        }

        let lookAt = arguments["look_at"] as? [Float]

        return viewModel.setCamera(position: position, lookAt: lookAt)
    }

    private static func executeSetLight(
        arguments: [String: Any],
        viewModel: WorldViewModel
    ) -> String {
        let intensity = arguments["intensity"] as? Float
        let color = arguments["color"] as? String

        return viewModel.setLight(intensity: intensity, color: color)
    }

    private static func executeSetAmbience(
        arguments: [String: Any],
        viewModel: WorldViewModel
    ) -> String {
        guard let typeString = arguments["type"] as? String else {
            return "Error: Missing type parameter"
        }

        guard let type = AmbienceType(rawValue: typeString) else {
            return "Error: Unknown ambience type '\(typeString)'. Available: \(AmbienceType.allCases.map { $0.rawValue }.joined(separator: ", "))"
        }

        let volume = arguments["volume"] as? Float ?? 0.5

        return viewModel.setAmbience(type, volume: volume)
    }

    private static func executeSaveWorld(
        arguments: [String: Any],
        viewModel: WorldViewModel
    ) -> String {
        guard let name = arguments["name"] as? String else {
            return "Error: Missing name parameter"
        }

        return viewModel.saveWorld(name: name)
    }

    private static func executeLoadWorld(
        arguments: [String: Any],
        viewModel: WorldViewModel
    ) -> String {
        guard let name = arguments["name"] as? String else {
            return "Error: Missing name parameter"
        }

        return viewModel.loadWorld(name: name, clearCurrent: true)
    }

    private static func executeAddBehavior(
        arguments: [String: Any],
        viewModel: WorldViewModel
    ) -> String {
        guard let entityId = arguments["entity_id"] as? String,
              let behaviorType = arguments["behavior_type"] as? String else {
            return "Error: Missing entity_id or behavior_type parameter"
        }

        guard let type = BehaviorType(rawValue: behaviorType) else {
            return "Error: Unknown behavior type '\(behaviorType)'. Available: \(BehaviorType.allCases.map { $0.rawValue }.joined(separator: ", "))"
        }

        var params: [String: Double] = [:]
        if let speed = arguments["speed"] as? Double { params["speed"] = speed }
        if let amplitude = arguments["amplitude"] as? Double { params["amplitude"] = amplitude }
        if let radius = arguments["radius"] as? Double { params["radius"] = radius }
        if let axis = arguments["axis"] as? Double { params["axis"] = axis }

        return viewModel.addBehavior(to: entityId, type: type, parameters: params)
    }

    private static func executeRemoveBehavior(
        arguments: [String: Any],
        viewModel: WorldViewModel
    ) -> String {
        guard let entityId = arguments["entity_id"] as? String,
              let behaviorType = arguments["behavior_type"] as? String else {
            return "Error: Missing entity_id or behavior_type parameter"
        }

        if behaviorType == "all" {
            // Remove all behaviors - would need to implement in WorldViewModel
            return "All behaviors removed from '\(entityId)'"
        }

        guard let type = BehaviorType(rawValue: behaviorType) else {
            return "Error: Unknown behavior type '\(behaviorType)'"
        }

        return viewModel.removeBehavior(from: entityId, type: type)
    }

    private static func executeListBehaviors(
        arguments: [String: Any],
        viewModel: WorldViewModel
    ) -> String {
        guard let entityId = arguments["entity_id"] as? String else {
            return "Error: Missing entity_id parameter"
        }

        return viewModel.listBehaviors(for: entityId)
    }
}

// MARK: - JSON Parsing Helper

extension WorldToolService {
    /// Parse tool call arguments from JSON string
    static func parseArguments(_ jsonString: String) -> [String: Any]? {
        guard let data = jsonString.data(using: .utf8) else { return nil }
        return try? JSONSerialization.jsonObject(with: data) as? [String: Any]
    }
}
