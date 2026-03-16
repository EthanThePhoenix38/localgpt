/// WebSocket client for connecting to the Bevy World Inspector server.

package com.localgpt.inspector

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.Response
import okhttp3.WebSocket
import okhttp3.WebSocketListener

enum class ConnectionState {
    DISCONNECTED, CONNECTING, CONNECTED, ERROR
}

class InspectorClient {
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main)
    private val okhttp = OkHttpClient()
    private var webSocket: WebSocket? = null

    // State flows
    private val _connectionState = MutableStateFlow(ConnectionState.DISCONNECTED)
    val connectionState: StateFlow<ConnectionState> = _connectionState.asStateFlow()

    private val _sceneTree = MutableStateFlow<List<TreeEntity>>(emptyList())
    val sceneTree: StateFlow<List<TreeEntity>> = _sceneTree.asStateFlow()

    private val _selectedEntityId = MutableStateFlow<ULong?>(null)
    val selectedEntityId: StateFlow<ULong?> = _selectedEntityId.asStateFlow()

    private val _selectedEntityDetail = MutableStateFlow<EntityDetailData?>(null)
    val selectedEntityDetail: StateFlow<EntityDetailData?> = _selectedEntityDetail.asStateFlow()

    private val _worldInfo = MutableStateFlow<WorldInfoData?>(null)
    val worldInfo: StateFlow<WorldInfoData?> = _worldInfo.asStateFlow()

    private val _selectedTransform = MutableStateFlow<Pair<List<Float>, List<Float>>?>(null)
    val selectedTransform: StateFlow<Pair<List<Float>, List<Float>>?> = _selectedTransform.asStateFlow()

    // Connection

    fun connect(host: String = "localhost", port: Int = 9877) {
        if (_connectionState.value == ConnectionState.CONNECTING ||
            _connectionState.value == ConnectionState.CONNECTED
        ) return

        _connectionState.value = ConnectionState.CONNECTING

        val request = Request.Builder()
            .url("ws://$host:$port/ws")
            .build()

        webSocket = okhttp.newWebSocket(request, object : WebSocketListener() {
            override fun onOpen(webSocket: WebSocket, response: Response) {
                scope.launch { _connectionState.value = ConnectionState.CONNECTED }
            }

            override fun onMessage(webSocket: WebSocket, text: String) {
                scope.launch { handleMessage(text) }
            }

            override fun onFailure(webSocket: WebSocket, t: Throwable, response: Response?) {
                scope.launch { _connectionState.value = ConnectionState.ERROR }
            }

            override fun onClosed(webSocket: WebSocket, code: Int, reason: String) {
                scope.launch { _connectionState.value = ConnectionState.DISCONNECTED }
            }
        })
    }

    fun disconnect() {
        webSocket?.close(1000, "Client disconnect")
        webSocket = null
        _connectionState.value = ConnectionState.DISCONNECTED
        _sceneTree.value = emptyList()
        _selectedEntityId.value = null
        _selectedEntityDetail.value = null
        _worldInfo.value = null
        _selectedTransform.value = null
    }

    // Commands

    fun selectEntity(entityId: ULong) {
        send(ClientMessage.SelectEntity(entityId))
        _selectedEntityId.value = entityId
        _selectedEntityDetail.value = null
        send(ClientMessage.RequestEntityDetail(entityId))
    }

    fun deselect() {
        send(ClientMessage.Deselect())
        _selectedEntityId.value = null
        _selectedEntityDetail.value = null
        _selectedTransform.value = null
    }

    fun toggleVisibility(entityId: ULong) {
        send(ClientMessage.ToggleVisibility(entityId))
    }

    fun focusEntity(entityId: ULong) {
        send(ClientMessage.FocusEntity(entityId))
    }

    fun refreshSceneTree() {
        send(ClientMessage.RequestSceneTree())
    }

    fun refreshWorldInfo() {
        send(ClientMessage.RequestWorldInfo())
    }

    // Private

    private fun send(message: ClientMessage) {
        val json = InspectorJson.encodeToString(ClientMessage.serializer(), message)
        webSocket?.send(json)
    }

    private fun handleMessage(text: String) {
        val message = try {
            InspectorJson.decodeFromString(ServerMessage.serializer(), text)
        } catch (e: Exception) {
            return
        }

        when (message) {
            is ServerMessage.SceneTree -> {
                _sceneTree.value = message.entities
            }
            is ServerMessage.EntityDetail -> {
                if (_selectedEntityId.value == message.entityId) {
                    _selectedEntityDetail.value = message.data
                }
            }
            is ServerMessage.WorldInfo -> {
                _worldInfo.value = message.data
            }
            is ServerMessage.SelectionChanged -> {
                _selectedEntityId.value = message.entityId
                _selectedEntityDetail.value = null
                _selectedTransform.value = null
                send(ClientMessage.RequestEntityDetail(message.entityId))
            }
            is ServerMessage.SelectionCleared -> {
                _selectedEntityId.value = null
                _selectedEntityDetail.value = null
                _selectedTransform.value = null
            }
            is ServerMessage.SceneChanged -> {
                send(ClientMessage.RequestSceneTree())
            }
            is ServerMessage.EntityTransformUpdated -> {
                if (_selectedEntityId.value == message.entityId) {
                    _selectedTransform.value = Pair(message.position, message.rotationDegrees)
                }
            }
        }
    }
}
