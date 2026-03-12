import Foundation
import Combine
import FoundationModels

/// Service for Apple's on-device Foundation Models (Apple Intelligence).
/// Provides zero-cost, private, offline-capable AI responses.
@MainActor
class AppleFoundationModelsService: ObservableObject {
    @Published var isAvailable = false
    @Published var isProcessing = false

    private var session: LanguageModelSession?

    init() {
        checkAvailability()
    }

    /// Check if Apple Intelligence is available on this device.
    func checkAvailability() {
        isAvailable = SystemLanguageModel.default.isAvailable
        if isAvailable {
            // Create session with LocalGPT persona
            session = LanguageModelSession {
                """
                You are LocalGPT, a helpful AI assistant with persistent memory.
                You have access to the user's notes, daily logs, and knowledge base.
                Be concise, helpful, and respect the user's privacy.
                """
            }
            // Prewarm for faster first response
            session?.prewarm()
        }
    }

    /// Send a message and get a response.
    /// Returns the response text, or nil if unavailable.
    func chat(message: String) async throws -> String? {
        guard isAvailable, let session = session else {
            return nil
        }

        isProcessing = true
        defer { isProcessing = false }

        do {
            let response = try await session.respond(to: message)
            return response.content
        } catch {
            // If Apple Intelligence fails, return nil to trigger fallback
            print("Apple Foundation Models error: \(error)")
            return nil
        }
    }

    /// Send a message and get a complete response (non-streaming).
    func chatComplete(message: String) async throws -> String? {
        guard isAvailable, let session = session else {
            return nil
        }

        isProcessing = true
        defer { isProcessing = false }

        do {
            let response = try await session.respond(to: message)
            return response.content
        } catch {
            print("Apple Foundation Models error: \(error)")
            return nil
        }
    }

    /// Send a message with world generation context.
    /// Returns the response text with any tool calls processed.
    func chatWithWorldTools(
        message: String,
        worldViewModel: WorldViewModel
    ) async throws -> String? {
        guard isAvailable, let session = session else {
            return nil
        }

        isProcessing = true
        defer { isProcessing = false }

        let worldPrompt = """
        You are a 3D world creation assistant. Help users build virtual worlds by calling tools.

        When the user asks to create something, respond with a JSON tool call in this exact format:
        {"tool": "<tool_name>", "arguments": {<arguments>}}

        Available tools:
        - spawn_primitive: {"tool": "spawn_primitive", "arguments": {"shape": "cube|sphere|cylinder|cone|plane|capsule", "position": [x, y, z], "color": "red|green|blue|etc"}}
        - modify_entity: {"tool": "modify_entity", "arguments": {"entity_id": "name", "position": [x, y, z], "color": "color"}}
        - delete_entity: {"tool": "delete_entity", "arguments": {"entity_id": "name"}}
        - set_camera: {"tool": "set_camera", "arguments": {"position": [x, y, z], "look_at": [x, y, z]}}
        - set_light: {"tool": "set_light", "arguments": {"intensity": 2000, "color": "white"}}
        - add_behavior: {"tool": "add_behavior", "arguments": {"entity_id": "name", "behavior_type": "orbit|spin|bob|pulse|bounce"}}
        - scene_info: {"tool": "scene_info", "arguments": {}}
        - clear_scene: {"tool": "clear_scene", "arguments": {}}

        Tips:
        - Position entities at reasonable Y values (e.g., Y=0.5 for a cube on the ground)
        - Use descriptive entity names
        - After creating entities, briefly describe what you made

        User request: \(message)
        """

        do {
            let response = try await session.respond(to: worldPrompt)
            var content = response.content

            // Process any tool calls in the response
            content = processToolCalls(in: content, worldViewModel: worldViewModel)

            return content
        } catch {
            print("Apple Foundation Models error: \(error)")
            return nil
        }
    }

    /// Process tool calls in LLM response text
    private func processToolCalls(in text: String, worldViewModel: WorldViewModel) -> String {
        var result = text

        // Pattern for JSON tool calls
        let toolPattern = #"\{"tool":\s*"([^"]+)",\s*"arguments":\s*(\{[^}]*\})\}"#

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

                if let args = WorldToolService.parseArguments(argsString) {
                    let toolResult = WorldToolService.executeTool(
                        name: toolName,
                        arguments: args,
                        viewModel: worldViewModel
                    )
                    result.replaceSubrange(fullRange, with: "✓ \(toolResult)")
                }
            }
        }

        return result
    }
}
