#if os(iOS) || os(visionOS)
import SwiftUI
import RealityKit

/// Combined view with 3D world and chat interface
struct WorldChatView: View {
    @StateObject private var worldViewModel = WorldViewModel()
    @StateObject private var chatViewModel = WorldChatViewModel()
    @State private var inputText = ""
    @State private var showWorldOnly = false

    var body: some View {
        NavigationStack {
            VStack(spacing: 0) {
                // 3D World View
                ZStack(alignment: .topTrailing) {
                    RealityKitView(viewModel: worldViewModel)

                    // Toggle button
                    Button(action: { showWorldOnly.toggle() }) {
                        Image(systemName: showWorldOnly ? "rectangle.split.3x3" : "rectangle")
                            .padding(8)
                            .background(.ultraThinMaterial)
                            .cornerRadius(8)
                    }
                    .padding(8)
                }
                .frame(height: showWorldOnly ? nil : 250)
                .frame(maxHeight: showWorldOnly ? .infinity : 250)

                if !showWorldOnly {
                    Divider()

                    // Chat View
                    VStack(spacing: 0) {
                        // Message List
                        ScrollViewReader { proxy in
                            ScrollView {
                                LazyVStack(spacing: 12) {
                                    ForEach(chatViewModel.messages) { message in
                                        MessageBubble(message: message)
                                            .id(message.id)
                                    }

                                    if chatViewModel.isThinking {
                                        ThinkingIndicator()
                                            .id("thinking")
                                    }
                                }
                                .padding()
                            }
                            .onChange(of: chatViewModel.messages) { oldMessages, newMessages in
                                withAnimation {
                                    proxy.scrollTo(newMessages.last?.id, anchor: .bottom)
                                }
                            }
                            .onChange(of: chatViewModel.isThinking) { oldThinking, newThinking in
                                if newThinking {
                                    withAnimation {
                                        proxy.scrollTo("thinking", anchor: .bottom)
                                    }
                                }
                            }
                        }

                        Divider()

                        // Input Area
                        HStack(spacing: 12) {
                            TextField("Describe what to build...", text: $inputText, axis: .vertical)
                                .padding(10)
                                .background(Color(.systemGray6))
                                .cornerRadius(20)
                                .lineLimit(1...3)

                            Button(action: sendMessage) {
                                Image(systemName: "arrow.up.circle.fill")
                                    .font(.system(size: 32))
                                    .foregroundColor(inputText.isEmpty ? .gray : .teal)
                            }
                            .disabled(inputText.isEmpty || chatViewModel.isThinking)
                        }
                        .padding()
                        .background(Color(.systemBackground))
                    }
                    .frame(maxHeight: .infinity)
                }
            }
            .navigationTitle("World Builder")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .navigationBarLeading) {
                    Menu {
                        Button(action: { showWorldOnly.toggle() }) {
                            Label(
                                showWorldOnly ? "Show Chat" : "Full Screen World",
                                systemImage: showWorldOnly ? "rectangle.split.3x3" : "rectangle"
                            )
                        }

                        Divider()

                        Button(action: { _ = worldViewModel.clearScene() }) {
                            Label("Clear Scene", systemImage: "trash")
                        }

                        Button(action: { _ = worldViewModel.listWorlds() }) {
                            Label("List Worlds", systemImage: "folder")
                        }
                    } label: {
                        Image(systemName: "ellipsis.circle")
                    }
                }

                ToolbarItem(placement: .navigationBarTrailing) {
                    HStack(spacing: 12) {
                        Button(action: { chatViewModel.resetSession() }) {
                            Image(systemName: "arrow.counterclockwise")
                        }

                        Menu {
                            Button(action: {
                                // Show save dialog
                            }) {
                                Label("Save World", systemImage: "square.and.arrow.down")
                            }

                            Button(action: {
                                // Show load dialog
                            }) {
                                Label("Load World", systemImage: "folder.badge.plus")
                            }
                        } label: {
                            Image(systemName: "square.and.arrow.down")
                        }
                    }
                }
            }
            .alert("Error", isPresented: $chatViewModel.showError) {
                Button("OK", role: .cancel) { }
            } message: {
                Text(chatViewModel.lastError ?? "Unknown error")
            }
        }
        .onAppear {
            chatViewModel.setWorldViewModel(worldViewModel)
        }
    }

    private func sendMessage() {
        let text = inputText.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !text.isEmpty else { return }

        inputText = ""
        chatViewModel.send(text: text)
    }
}

#Preview {
    WorldChatView()
}
#endif
