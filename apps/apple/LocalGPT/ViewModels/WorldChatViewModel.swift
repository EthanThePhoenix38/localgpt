#if os(iOS) || os(visionOS)
import Foundation
import Combine
import LocalGPTWrapper

/// Chat ViewModel with world generation tool support.
/// Routes tool calls to WorldViewModel for 3D scene manipulation.
@MainActor
class WorldChatViewModel: ObservableObject {
    @Published var messages: [Message] = []
    @Published var isThinking = false
    @Published var showError = false
    @Published var lastError: String?
    @Published var isUsingOnDevice = false

    private var client: LocalGptClient?
    private var appleService = AppleFoundationModelsService()
    private weak var worldViewModel: WorldViewModel?

    // System prompt for world generation
    private let worldSystemPrompt = """
    You are a 3D world creation assistant. Help users build virtual worlds by calling tools.

    Available tools:
    - spawn_primitive: Create 3D shapes (cube, sphere, cylinder, cone, plane, capsule)
    - modify_entity: Move, scale, rotate, or recolor entities
    - delete_entity: Remove entities
    - set_camera: Position the camera
    - set_light: Adjust lighting
    - scene_info: Get current scene state
    - clear_scene: Reset the world

    Tips:
    - Position entities at reasonable Y values (e.g., Y=0.5 for a cube on the ground)
    - Use descriptive entity names
    - Create interesting compositions with multiple shapes
    - Consider scale relationships between objects

    When the user asks to create something, call the appropriate tool directly.
    """

    init(worldViewModel: WorldViewModel? = nil) {
        self.worldViewModel = worldViewModel
        setupClient()
    }

    func setWorldViewModel(_ viewModel: WorldViewModel) {
        self.worldViewModel = viewModel
    }

    private func setupClient() {
        isUsingOnDevice = appleService.isAvailable

        do {
            let docs = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask).first!
            let dataDir = docs.appendingPathComponent("LocalGPT", isDirectory: true).path
            self.client = try LocalGptClient(dataDir: dataDir)

            if client?.isBrandNew() ?? false {
                let modeInfo: String
                if appleService.isAvailable {
                    modeInfo = "\n\n✅ Using on-device Apple Intelligence (free, private)"
                } else {
                    modeInfo = "\n\n☁️ Cloud API mode"
                }
                messages.append(Message(
                    text: "I'm your 3D world creation assistant! Tell me what to build and I'll create it in the World tab. Try saying \"create a red cube\" or \"build a simple house\"." + modeInfo,
                    isUser: false
                ))
            }
        } catch {
            handleError(error)
        }
    }

    func send(text: String) {
        let userMsg = Message(text: text, isUser: true)
        messages.append(userMsg)
        isThinking = true

        Task(priority: .userInitiated) {
            var response: String?
            var usedOnDevice = false

            // First, check if this is a direct tool command
            if let toolResponse = tryDirectToolCall(text) {
                response = toolResponse
            } else {
                // Try Apple Foundation Models with world tools
                if appleService.isAvailable, let worldVM = worldViewModel {
                    response = try? await appleService.chatWithWorldTools(
                        message: text,
                        worldViewModel: worldVM
                    )
                    if response != nil {
                        usedOnDevice = true
                    }
                }

                // Fallback to Rust client
                if response == nil {
                    isUsingOnDevice = false
                    response = await sendViaRustClient(text: text)
                    response = processToolCalls(in: response ?? "")
                }
            }

            await MainActor.run {
                self.isThinking = false
                if let response = response, !response.isEmpty {
                    self.isUsingOnDevice = usedOnDevice
                    self.messages.append(Message(text: response, isUser: false))
                } else {
                    self.showError = true
                    self.lastError = "No AI provider available"
                }
            }
        }
    }

    /// Try to parse and execute direct tool commands like "/spawn cube"
    private func tryDirectToolCall(_ text: String) -> String? {
        let trimmed = text.trimmingCharacters(in: .whitespacesAndNewlines)

        // Check for slash commands
        guard trimmed.hasPrefix("/") else { return nil }

        let parts = trimmed.dropFirst().split(separator: " ", omittingEmptySubsequences: true)
        guard !parts.isEmpty else { return nil }

        let command = String(parts[0]).lowercased()
        let args = parts.dropFirst().map(String.init)

        switch command {
        case "spawn", "create", "add":
            return handleSpawnCommand(args)

        case "delete", "remove":
            return handleDeleteCommand(args)

        case "camera":
            return handleCameraCommand(args)

        case "behavior", "behaviour":
            return handleBehaviorCommand(args)

        case "ambience":
            return handleAmbienceCommand(args)

        case "clear":
            _ = worldViewModel?.clearScene()
            return "Scene cleared"

        case "info", "list":
            return worldViewModel?.sceneInfo()

        case "save":
            let name = args.first ?? "untitled"
            return worldViewModel?.saveWorld(name: name)

        case "load":
            guard let name = args.first else {
                return worldViewModel?.listWorlds()
            }
            return worldViewModel?.loadWorld(name: name, clearCurrent: true)

        case "worlds":
            return worldViewModel?.listWorlds()

        case "help":
            return getHelpText()

        default:
            return nil
        }
    }

    private func getHelpText() -> String {
        return """
        **World Builder Commands**

        **Creating Objects:**
        `/spawn <shape> [color] [name]` - Create a shape (cube, sphere, cylinder, cone, plane, capsule)
        `/delete <name>` - Delete an entity

        **Camera:**
        `/camera` - Reset camera to default
        `/camera <x> <y> <z>` - Position camera
        `/camera <x> <y> <z> <lookX> <lookY> <lookZ>` - Position and aim camera

        **Behaviors:**
        `/behavior <entity> <type>` - Add behavior (orbit, spin, bob, pulse, bounce)
        `/behavior <entity> <type> <speed>` - Add behavior with speed

        **Ambience:**
        `/ambience <type>` - Set ambience (silence, wind, rain, forest, ocean, cave, stream)

        **World Management:**
        `/save <name>` - Save current world
        `/load <name>` - Load a saved world
        `/worlds` - List saved worlds
        `/clear` - Clear all entities
        `/info` - Show scene info

        **Natural Language:**
        Just describe what you want to build! Try:
        - "create a red cube"
        - "build a tower with blue blocks"
        - "make the cube spin"
        """
    }

    private func handleBehaviorCommand(_ args: [String]) -> String? {
        guard let worldVM = worldViewModel, args.count >= 2 else { return nil }

        let entityId = args[0]
        let behaviorType = args[1].lowercased()

        var params: [String: Double] = [:]
        if args.count > 2, let speed = Double(args[2]) {
            params["speed"] = speed
        }

        guard let type = BehaviorType(rawValue: behaviorType) else {
            return "Unknown behavior: \(behaviorType). Available: orbit, spin, bob, lookAt, pulse, bounce, pathFollow"
        }

        return worldVM.addBehavior(to: entityId, type: type, parameters: params)
    }

    private func handleAmbienceCommand(_ args: [String]) -> String? {
        guard let worldVM = worldViewModel, let typeString = args.first else { return nil }

        guard let type = AmbienceType(rawValue: typeString.lowercased()) else {
            return "Unknown ambience: \(typeString). Available: \(AmbienceType.allCases.map { $0.rawValue }.joined(separator: ", "))"
        }

        return worldVM.setAmbience(type)
    }

    private func handleSpawnCommand(_ args: [String]) -> String? {
        guard let worldVM = worldViewModel, !args.isEmpty else { return nil }

        // Parse: /spawn [shape] [color] [name]
        let shape = args[0].lowercased()
        let color = args.count > 1 ? args[1] : "gray"
        let name = args.count > 2 ? args[2] : nil

        // Default position
        let position: [Float] = [Float.random(in: -3...3), 0.5, Float.random(in: -3...3)]

        let entityId = worldVM.spawnPrimitive(
            shape: shape,
            position: position,
            color: color,
            name: name
        )

        return "Created \(shape) '\(entityId)' at [\(String(format: "%.1f", position[0])), \(String(format: "%.1f", position[1])), \(String(format: "%.1f", position[2]))]"
    }

    private func handleDeleteCommand(_ args: [String]) -> String? {
        guard let worldVM = worldViewModel, let name = args.first else { return nil }
        return worldVM.deleteEntity(name)
    }

    private func handleCameraCommand(_ args: [String]) -> String? {
        guard let worldVM = worldViewModel else { return nil }

        if args.isEmpty {
            return worldVM.setCamera(position: [0, 5, 10], lookAt: [0, 0, 0])
        }

        // Parse: /camera x y z [lookX lookY lookZ]
        let nums = args.compactMap { Float($0) }
        if nums.count >= 3 {
            let lookAt = nums.count >= 6 ? [nums[3], nums[4], nums[5]] : nil
            return worldVM.setCamera(position: [nums[0], nums[1], nums[2]], lookAt: lookAt)
        }

        return nil
    }

    /// Process tool calls in LLM response text
    private func processToolCalls(in text: String) -> String {
        // Look for JSON tool call patterns like: {"tool": "spawn_primitive", "arguments": {...}}
        var result = text

        // Simple pattern matching for tool calls
        let toolPattern = #"\{"tool":\s*"([^"]+)",\s*"arguments":\s*(\{[^}]+\})\}"#

        guard let regex = try? NSRegularExpression(pattern: toolPattern, options: []) else {
            return text
        }

        let range = NSRange(result.startIndex..., in: result)
        let matches = regex.matches(in: result, options: [], range: range)

        // Process matches in reverse to maintain indices
        for match in matches.reversed() {
            if let toolRange = Range(match.range(at: 1), in: result),
               let argsRange = Range(match.range(at: 2), in: result),
               let fullRange = Range(match.range, in: result) {
                let toolName = String(result[toolRange])
                let argsString = String(result[argsRange])

                if let args = WorldToolService.parseArguments(argsString),
                   let worldVM = worldViewModel {
                    let toolResult = WorldToolService.executeTool(
                        name: toolName,
                        arguments: args,
                        viewModel: worldVM
                    )
                    result.replaceSubrange(fullRange, with: "[\(toolResult)]")
                }
            }
        }

        return result
    }

    private func sendViaRustClient(text: String) async -> String? {
        guard let client = client else { return nil }

        return await withCheckedContinuation { continuation in
            Task.detached(priority: .userInitiated) {
                do {
                    let response = try client.chat(message: text)
                    continuation.resume(returning: response)
                } catch {
                    print("Rust client error: \(error)")
                    continuation.resume(returning: nil)
                }
            }
        }
    }

    func resetSession() {
        do {
            try client?.newSession()
            messages.removeAll()
            if client?.isBrandNew() ?? false {
                messages.append(Message(
                    text: "Ready to create 3D worlds! What would you like to build?",
                    isUser: false
                ))
            }
        } catch {
            handleError(error)
        }
    }

    private func handleError(_ error: Error) {
        self.lastError = error.localizedDescription
        self.showError = true
    }
}
#endif
