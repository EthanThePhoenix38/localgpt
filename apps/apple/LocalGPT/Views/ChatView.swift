import SwiftUI
import LocalGPTWrapper

struct ChatView: View {
    @StateObject private var viewModel = ChatViewModel()
    @State private var inputText = ""

    var body: some View {
        NavigationStack {
            VStack {
                // Message List
                ScrollViewReader { proxy in
                    ScrollView {
                        LazyVStack(spacing: 12) {
                            ForEach(viewModel.messages) { message in
                                MessageBubble(message: message)
                                    .id(message.id)
                            }

                            if viewModel.isThinking {
                                ThinkingIndicator()
                                    .id("thinking")
                            }
                        }
                        .padding()
                    }
                    .onChange(of: viewModel.messages) { oldMessages, newMessages in
                        withAnimation {
                            proxy.scrollTo(newMessages.last?.id, anchor: .bottom)
                        }
                    }
                    .onChange(of: viewModel.isThinking) { oldThinking, newThinking in
                        if newThinking {
                            withAnimation {
                                proxy.scrollTo("thinking", anchor: .bottom)
                            }
                        }
                    }
                }

                // Input Area
                HStack(spacing: 12) {
                    TextField("Ask LocalGPT...", text: $inputText, axis: .vertical)
                        .padding(10)
                        .background(Color(.systemGray6))
                        .cornerRadius(20)
                        .lineLimit(1...5)

                    Button(action: sendMessage) {
                        Image(systemName: "arrow.up.circle.fill")
                            .font(.system(size: 32))
                            .foregroundColor(inputText.isEmpty ? .gray : .teal)
                    }
                    .disabled(inputText.isEmpty || viewModel.isThinking)
                }
                .padding()
                .background(Color(.systemBackground))
            }
            .navigationTitle("LocalGPT")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button(action: viewModel.resetSession) {
                        Image(systemName: "trash")
                    }
                }
            }
            .alert("Error", isPresented: $viewModel.showError) {
                Button("OK", role: .cancel) { }
            } message: {
                Text(viewModel.lastError ?? "Unknown error")
            }
        }
    }

    private func sendMessage() {
        let text = inputText.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !text.isEmpty else { return }

        inputText = ""
        viewModel.send(text: text)
    }
}

#Preview {
    ChatView()
}
