package com.localgpt.inspector.ui

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.unit.dp
import com.localgpt.inspector.InspectorClient
import com.localgpt.inspector.TreeEntity

@Composable
fun OutlinerScreen(
    client: InspectorClient,
    modifier: Modifier = Modifier,
) {
    val sceneTree by client.sceneTree.collectAsState()
    val selectedId by client.selectedEntityId.collectAsState()

    var searchText by remember { mutableStateOf("") }
    val expandedIds = remember { mutableStateSetOf<ULong>() }

    val rootEntities = remember(sceneTree, searchText) {
        val filtered = if (searchText.isBlank()) sceneTree
        else sceneTree.filter { it.name.contains(searchText, ignoreCase = true) }
        filtered.filter { it.parentId == null }
    }

    Column(modifier = modifier.fillMaxSize()) {
        // Search bar
        OutlinedTextField(
            value = searchText,
            onValueChange = { searchText = it },
            modifier = Modifier
                .fillMaxWidth()
                .padding(8.dp),
            placeholder = { Text("Search entities...") },
            leadingIcon = { Icon(Icons.Default.Search, contentDescription = "Search") },
            trailingIcon = {
                if (searchText.isNotEmpty()) {
                    IconButton(onClick = { searchText = "" }) {
                        Icon(Icons.Default.Clear, contentDescription = "Clear")
                    }
                }
            },
            singleLine = true,
        )

        HorizontalDivider()

        // Entity tree
        LazyColumn(
            modifier = Modifier.weight(1f),
            contentPadding = PaddingValues(vertical = 4.dp),
        ) {
            items(rootEntities, key = { it.id.toLong() }) { entity ->
                EntityTreeItem(
                    entity = entity,
                    allEntities = sceneTree,
                    selectedId = selectedId,
                    expandedIds = expandedIds,
                    depth = 0,
                    onSelect = { client.selectEntity(it) },
                    onToggleVisibility = { client.toggleVisibility(it) },
                    onToggleExpand = { id ->
                        if (id in expandedIds) expandedIds.remove(id)
                        else expandedIds.add(id)
                    },
                )
            }
        }

        HorizontalDivider()

        // Status bar
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 12.dp, vertical = 6.dp),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(
                "${sceneTree.size} entities",
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    }
}

@Composable
private fun EntityTreeItem(
    entity: TreeEntity,
    allEntities: List<TreeEntity>,
    selectedId: ULong?,
    expandedIds: Set<ULong>,
    depth: Int,
    onSelect: (ULong) -> Unit,
    onToggleVisibility: (ULong) -> Unit,
    onToggleExpand: (ULong) -> Unit,
) {
    val children = remember(allEntities, entity.id) {
        allEntities.filter { it.parentId == entity.id }
    }
    val isExpanded = entity.id in expandedIds
    val isSelected = entity.id == selectedId

    Column {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .clickable { onSelect(entity.id) }
                .padding(start = (16 + depth * 20).dp, end = 8.dp, top = 4.dp, bottom = 4.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            // Expand/collapse
            if (children.isNotEmpty()) {
                IconButton(
                    onClick = { onToggleExpand(entity.id) },
                    modifier = Modifier.size(24.dp),
                ) {
                    Icon(
                        if (isExpanded) Icons.Default.ExpandMore else Icons.Default.ChevronRight,
                        contentDescription = "Toggle expand",
                        modifier = Modifier.size(16.dp),
                    )
                }
            } else {
                Spacer(Modifier.size(24.dp))
            }

            Spacer(Modifier.width(4.dp))

            // Entity type icon
            Icon(
                imageVector = entityTypeIcon(entity.entityType),
                contentDescription = entity.entityType,
                modifier = Modifier.size(16.dp),
                tint = if (entity.visible)
                    MaterialTheme.colorScheme.onSurface
                else
                    MaterialTheme.colorScheme.onSurface.copy(alpha = 0.3f),
            )

            Spacer(Modifier.width(8.dp))

            // Name
            Text(
                text = entity.name,
                style = MaterialTheme.typography.bodyMedium.let {
                    if (!entity.visible) it.copy(textDecoration = TextDecoration.LineThrough)
                    else it
                },
                color = when {
                    isSelected -> MaterialTheme.colorScheme.primary
                    !entity.visible -> MaterialTheme.colorScheme.onSurface.copy(alpha = 0.4f)
                    else -> MaterialTheme.colorScheme.onSurface
                },
                modifier = Modifier.weight(1f),
            )

            // Visibility toggle
            IconButton(
                onClick = { onToggleVisibility(entity.id) },
                modifier = Modifier.size(24.dp),
            ) {
                Icon(
                    if (entity.visible) Icons.Default.Visibility else Icons.Default.VisibilityOff,
                    contentDescription = "Toggle visibility",
                    modifier = Modifier.size(14.dp),
                    tint = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }

        // Selected highlight
        if (isSelected) {
            HorizontalDivider(
                modifier = Modifier.padding(start = (16 + depth * 20).dp),
                color = MaterialTheme.colorScheme.primary,
                thickness = 2.dp,
            )
        }

        // Children (recursive)
        if (isExpanded) {
            children.forEach { child ->
                EntityTreeItem(
                    entity = child,
                    allEntities = allEntities,
                    selectedId = selectedId,
                    expandedIds = expandedIds,
                    depth = depth + 1,
                    onSelect = onSelect,
                    onToggleVisibility = onToggleVisibility,
                    onToggleExpand = onToggleExpand,
                )
            }
        }
    }
}

private fun entityTypeIcon(entityType: String) = when (entityType) {
    "Primitive" -> Icons.Default.Cube
    "Light" -> Icons.Default.LightMode
    "Camera" -> Icons.Default.CameraAlt
    "Mesh" -> Icons.Default.ViewInAr
    "Group" -> Icons.Default.Folder
    "AudioEmitter" -> Icons.Default.VolumeUp
    else -> Icons.Default.HelpOutline
}

@Composable
private fun mutableStateSetOf(vararg elements: ULong): MutableSet<ULong> {
    return remember { mutableStateSetOf<ULong>().apply { addAll(elements.toList()) } }
}
