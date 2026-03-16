/// World Inspector Protocol — Kotlin types matching the Bevy WebSocket protocol.

package com.localgpt.inspector

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonContentPolymorphicSerializer
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive

// ---------------------------------------------------------------------------
// JSON configuration
// ---------------------------------------------------------------------------

val InspectorJson = Json {
    ignoreUnknownKeys = true
    encodeDefaults = true
}

// ---------------------------------------------------------------------------
// Client → Server messages
// ---------------------------------------------------------------------------

@Serializable
sealed class ClientMessage {
    abstract val type: String

    @Serializable
    data class Subscribe(val topics: List<String>) : ClientMessage() {
        override val type = "subscribe"
    }

    @Serializable
    class RequestSceneTree : ClientMessage() {
        override val type = "request_scene_tree"
    }

    @Serializable
    data class RequestEntityDetail(
        @SerialName("entity_id") val entityId: ULong
    ) : ClientMessage() {
        override val type = "request_entity_detail"
    }

    @Serializable
    class RequestWorldInfo : ClientMessage() {
        override val type = "request_world_info"
    }

    @Serializable
    data class SelectEntity(
        @SerialName("entity_id") val entityId: ULong
    ) : ClientMessage() {
        override val type = "select_entity"
    }

    @Serializable
    class Deselect : ClientMessage() {
        override val type = "deselect"
    }

    @Serializable
    data class ToggleVisibility(
        @SerialName("entity_id") val entityId: ULong
    ) : ClientMessage() {
        override val type = "toggle_visibility"
    }

    @Serializable
    data class FocusEntity(
        @SerialName("entity_id") val entityId: ULong
    ) : ClientMessage() {
        override val type = "focus_entity"
    }

    @Serializable
    class RequestSceneSnapshot : ClientMessage() {
        override val type = "request_scene_snapshot"
    }
}

// ---------------------------------------------------------------------------
// Server → Client messages
// ---------------------------------------------------------------------------

@Serializable(with = ServerMessageSerializer::class)
sealed class ServerMessage {

    @Serializable
    data class SceneTree(val entities: List<TreeEntity>) : ServerMessage()

    @Serializable
    data class EntityDetail(
        @SerialName("entity_id") val entityId: ULong,
        val data: EntityDetailData
    ) : ServerMessage()

    @Serializable
    data class WorldInfo(val data: WorldInfoData) : ServerMessage()

    @Serializable
    data class SelectionChanged(
        @SerialName("entity_id") val entityId: ULong
    ) : ServerMessage()

    @Serializable
    object SelectionCleared : ServerMessage()

    @Serializable
    object SceneChanged : ServerMessage()

    @Serializable
    data class EntityTransformUpdated(
        @SerialName("entity_id") val entityId: ULong,
        val position: List<Float>,
        @SerialName("rotation_degrees") val rotationDegrees: List<Float>
    ) : ServerMessage()
}

object ServerMessageSerializer : JsonContentPolymorphicSerializer<ServerMessage>(ServerMessage::class) {
    override fun selectDeserializer(element: JsonElement) = when (
        element.jsonObject["type"]?.jsonPrimitive?.content
    ) {
        "scene_tree" -> ServerMessage.SceneTree.serializer()
        "entity_detail" -> ServerMessage.EntityDetail.serializer()
        "world_info" -> ServerMessage.WorldInfo.serializer()
        "selection_changed" -> ServerMessage.SelectionChanged.serializer()
        "selection_cleared" -> ServerMessage.SelectionCleared.serializer()
        "scene_changed" -> ServerMessage.SceneChanged.serializer()
        "entity_transform_updated" -> ServerMessage.EntityTransformUpdated.serializer()
        else -> throw IllegalArgumentException("Unknown type: ${element.jsonObject["type"]}")
    }
}

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

@Serializable
data class TreeEntity(
    val id: ULong,
    val name: String,
    @SerialName("entity_type") val entityType: String,
    @SerialName("parent_id") val parentId: ULong? = null,
    val visible: Boolean,
    val children: List<ULong>
)

@Serializable
data class EntityDetailData(
    val identity: IdentitySection,
    val transform: TransformSection? = null,
    val shape: String? = null,
    val material: MaterialSection? = null,
    val light: LightSection? = null,
    val behaviors: List<BehaviorSection> = emptyList(),
    val audio: AudioSection? = null,
    @SerialName("mesh_asset") val meshAsset: String? = null,
    val hierarchy: HierarchySection
)

@Serializable
data class IdentitySection(
    val name: String,
    val id: ULong,
    @SerialName("entity_type") val entityType: String
)

@Serializable
data class TransformSection(
    val position: List<Float>,
    @SerialName("rotation_degrees") val rotationDegrees: List<Float>,
    val scale: List<Float>,
    val visible: Boolean
)

@Serializable
data class MaterialSection(
    @SerialName("base_color") val baseColor: List<Float>,
    val metallic: Float,
    val roughness: Float,
    val reflectance: Float,
    val emissive: List<Float>,
    @SerialName("alpha_mode") val alphaMode: String,
    @SerialName("double_sided") val doubleSided: Boolean,
    val unlit: Boolean
)

@Serializable
data class LightSection(
    @SerialName("light_type") val lightType: String,
    val color: List<Float>,
    val intensity: Float,
    val range: Float? = null,
    @SerialName("shadows_enabled") val shadowsEnabled: Boolean,
    @SerialName("inner_angle") val innerAngle: Float? = null,
    @SerialName("outer_angle") val outerAngle: Float? = null
)

@Serializable
data class BehaviorSection(
    val id: String,
    @SerialName("behavior_type") val behaviorType: String,
    @SerialName("base_position") val basePosition: List<Float>,
    @SerialName("base_scale") val baseScale: List<Float>
)

@Serializable
data class AudioSection(
    @SerialName("sound_type") val soundType: String,
    val volume: Float,
    val radius: Float,
    @SerialName("attached_to") val attachedTo: String? = null,
    val position: List<Float>? = null
)

@Serializable
data class HierarchySection(
    val parent: String? = null,
    val children: List<String> = emptyList()
)

@Serializable
data class WorldInfoData(
    val name: String? = null,
    @SerialName("entity_count") val entityCount: Int,
    @SerialName("behavior_state") val behaviorState: BehaviorStateInfo,
    val audio: AudioStateInfo? = null
)

@Serializable
data class BehaviorStateInfo(
    val paused: Boolean,
    val elapsed: Double
)

@Serializable
data class AudioStateInfo(
    val active: Boolean,
    @SerialName("emitter_count") val emitterCount: Int,
    @SerialName("ambience_layers") val ambienceLayers: List<String>
)
