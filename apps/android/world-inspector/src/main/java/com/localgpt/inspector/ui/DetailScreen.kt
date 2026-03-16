package com.localgpt.inspector.ui

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.dp
import com.localgpt.inspector.EntityDetailData
import com.localgpt.inspector.InspectorClient

@Composable
fun DetailScreen(
    client: InspectorClient,
    modifier: Modifier = Modifier,
) {
    val detail by client.selectedEntityDetail.collectAsState()
    val selectedId by client.selectedEntityId.collectAsState()
    val liveTransform by client.selectedTransform.collectAsState()

    Box(modifier = modifier.fillMaxSize()) {
        when {
            detail != null -> {
                val d = detail!!
                Column(
                    modifier = Modifier
                        .fillMaxSize()
                        .verticalScroll(rememberScrollState())
                        .padding(12.dp),
                    verticalArrangement = Arrangement.spacedBy(12.dp),
                ) {
                    IdentityCard(d.identity.name, d.identity.entityType, d.identity.id)
                    TransformCard(d.transform, liveTransform)
                    d.shape?.let { ShapeCard(it) }
                    d.material?.let { MaterialCard(it) }
                    d.light?.let { LightCard(it) }
                    if (d.behaviors.isNotEmpty()) BehaviorsCard(d.behaviors)
                    d.audio?.let { AudioCard(it) }
                    d.meshAsset?.let { MeshCard(it) }
                    HierarchyCard(d.hierarchy)
                }
            }
            selectedId != null -> {
                Column(
                    modifier = Modifier.align(Alignment.Center),
                    horizontalAlignment = Alignment.CenterHorizontally,
                ) {
                    CircularProgressIndicator()
                    Spacer(Modifier.height(8.dp))
                    Text("Loading...", color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
            }
            else -> {
                Column(
                    modifier = Modifier.align(Alignment.Center),
                    horizontalAlignment = Alignment.CenterHorizontally,
                ) {
                    Icon(
                        Icons.Default.ViewInAr,
                        contentDescription = null,
                        modifier = Modifier.size(48.dp),
                        tint = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.2f),
                    )
                    Spacer(Modifier.height(8.dp))
                    Text(
                        "Select an entity to inspect",
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Section cards
// ---------------------------------------------------------------------------

@Composable
private fun IdentityCard(name: String, entityType: String, id: ULong) {
    SectionCard("Identity", Icons.Default.Info) {
        PropertyRow("Name", name)
        PropertyRow("Type", entityType)
        PropertyRow("ID", id.toString())
    }
}

@Composable
private fun TransformCard(
    transform: com.localgpt.inspector.TransformSection?,
    live: Pair<List<Float>, List<Float>>?,
) {
    if (transform == null) return
    val pos = live?.first ?: transform.position
    val rot = live?.second ?: transform.rotationDegrees

    SectionCard("Transform", Icons.Default.OpenWith) {
        PropertyRow("Position", formatVec3(pos))
        PropertyRow("Rotation", formatVec3(rot))
        PropertyRow("Scale", formatVec3(transform.scale))
        PropertyRow("Visible", if (transform.visible) "Yes" else "No")
    }
}

@Composable
private fun ShapeCard(shape: String) {
    SectionCard("Shape", Icons.Default.Category) {
        PropertyRow("Variant", shape)
    }
}

@Composable
private fun MaterialCard(mat: com.localgpt.inspector.MaterialSection) {
    SectionCard("Material", Icons.Default.Palette) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Text(
                "Base Color",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            Spacer(Modifier.weight(1f))
            ColorSwatch(mat.baseColor)
            Spacer(Modifier.width(6.dp))
            Text(
                formatVec4(mat.baseColor),
                style = MaterialTheme.typography.bodySmall,
                fontFamily = FontFamily.Monospace,
            )
        }
        PropertyRow("Metallic", "%.3f".format(mat.metallic))
        PropertyRow("Roughness", "%.3f".format(mat.roughness))
        PropertyRow("Reflectance", "%.3f".format(mat.reflectance))
        PropertyRow("Alpha Mode", mat.alphaMode)
        PropertyRow("Double Sided", if (mat.doubleSided) "Yes" else "No")
        PropertyRow("Unlit", if (mat.unlit) "Yes" else "No")
    }
}

@Composable
private fun LightCard(light: com.localgpt.inspector.LightSection) {
    SectionCard("Light (${light.lightType})", Icons.Default.LightMode) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Text(
                "Color",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            Spacer(Modifier.weight(1f))
            ColorSwatch(light.color + listOf(1f))
            Spacer(Modifier.width(6.dp))
            Text(
                formatVec3(light.color),
                style = MaterialTheme.typography.bodySmall,
                fontFamily = FontFamily.Monospace,
            )
        }
        PropertyRow("Intensity", "%.1f".format(light.intensity))
        light.range?.let { PropertyRow("Range", "%.2f".format(it)) }
        PropertyRow("Shadows", if (light.shadowsEnabled) "Yes" else "No")
        light.innerAngle?.let {
            PropertyRow("Inner Angle", "%.1f°".format(it * 180f / Math.PI.toFloat()))
        }
        light.outerAngle?.let {
            PropertyRow("Outer Angle", "%.1f°".format(it * 180f / Math.PI.toFloat()))
        }
    }
}

@Composable
private fun BehaviorsCard(behaviors: List<com.localgpt.inspector.BehaviorSection>) {
    SectionCard("Behaviors (${behaviors.size})", Icons.Default.Loop) {
        behaviors.forEach { beh ->
            Column(modifier = Modifier.padding(vertical = 2.dp)) {
                Text(
                    "${beh.id}: ${beh.behaviorType}",
                    style = MaterialTheme.typography.bodySmall,
                    fontFamily = FontFamily.Monospace,
                )
                Text(
                    "Base: ${formatVec3(beh.basePosition)}",
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
    }
}

@Composable
private fun AudioCard(audio: com.localgpt.inspector.AudioSection) {
    SectionCard("Audio", Icons.Default.VolumeUp) {
        PropertyRow("Sound", audio.soundType)
        PropertyRow("Volume", "%.2f".format(audio.volume))
        PropertyRow("Radius", "%.1fm".format(audio.radius))
        audio.attachedTo?.let { PropertyRow("Attached To", it) }
    }
}

@Composable
private fun MeshCard(path: String) {
    SectionCard("Mesh Asset", Icons.Default.Description) {
        PropertyRow("Path", path)
    }
}

@Composable
private fun HierarchyCard(hierarchy: com.localgpt.inspector.HierarchySection) {
    if (hierarchy.parent == null && hierarchy.children.isEmpty()) return
    SectionCard("Hierarchy", Icons.Default.AccountTree) {
        hierarchy.parent?.let { PropertyRow("Parent", it) }
        if (hierarchy.children.isNotEmpty()) {
            Text(
                "Children",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            hierarchy.children.forEach { child ->
                Text(
                    child,
                    style = MaterialTheme.typography.bodySmall,
                    fontFamily = FontFamily.Monospace,
                    modifier = Modifier.padding(start = 8.dp),
                )
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Reusable components
// ---------------------------------------------------------------------------

@Composable
private fun SectionCard(
    title: String,
    icon: ImageVector,
    content: @Composable ColumnScope.() -> Unit,
) {
    var expanded by remember { mutableStateOf(true) }

    Surface(
        shape = RoundedCornerShape(8.dp),
        tonalElevation = 1.dp,
    ) {
        Column(modifier = Modifier.padding(12.dp)) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Icon(
                    icon,
                    contentDescription = null,
                    modifier = Modifier.size(18.dp),
                    tint = MaterialTheme.colorScheme.primary,
                )
                Spacer(Modifier.width(8.dp))
                Text(title, style = MaterialTheme.typography.titleSmall)
                Spacer(Modifier.weight(1f))
                IconButton(
                    onClick = { expanded = !expanded },
                    modifier = Modifier.size(24.dp),
                ) {
                    Icon(
                        if (expanded) Icons.Default.ExpandLess else Icons.Default.ExpandMore,
                        contentDescription = "Toggle section",
                        modifier = Modifier.size(16.dp),
                    )
                }
            }

            AnimatedVisibility(visible = expanded) {
                Column(
                    modifier = Modifier.padding(top = 8.dp),
                    verticalArrangement = Arrangement.spacedBy(4.dp),
                    content = content,
                )
            }
        }
    }
}

@Composable
private fun PropertyRow(key: String, value: String) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            key,
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        Spacer(Modifier.weight(1f))
        Text(
            value,
            style = MaterialTheme.typography.bodySmall,
            fontFamily = FontFamily.Monospace,
        )
    }
}

@Composable
private fun ColorSwatch(rgba: List<Float>) {
    val r = (rgba.getOrElse(0) { 0f }).coerceIn(0f, 1f)
    val g = (rgba.getOrElse(1) { 0f }).coerceIn(0f, 1f)
    val b = (rgba.getOrElse(2) { 0f }).coerceIn(0f, 1f)
    Box(
        modifier = Modifier
            .size(14.dp)
            .clip(RoundedCornerShape(3.dp))
            .background(Color(r, g, b)),
    )
}

// ---------------------------------------------------------------------------
// Formatting
// ---------------------------------------------------------------------------

private fun formatVec3(v: List<Float>): String {
    if (v.size < 3) return "[]"
    return "[%.3f, %.3f, %.3f]".format(v[0], v[1], v[2])
}

private fun formatVec4(v: List<Float>): String {
    if (v.size < 4) return "[]"
    return "[%.2f, %.2f, %.2f, %.2f]".format(v[0], v[1], v[2], v[3])
}
