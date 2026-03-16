package com.localgpt.inspector.ui

import androidx.compose.foundation.layout.*
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.localgpt.inspector.ConnectionState
import com.localgpt.inspector.InspectorClient

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun InspectorScreen(
    client: InspectorClient = remember { InspectorClient() },
    modifier: Modifier = Modifier,
) {
    val connectionState by client.connectionState.collectAsState()
    val worldInfo by client.worldInfo.collectAsState()
    val selectedId by client.selectedEntityId.collectAsState()

    var showOutliner by remember { mutableStateOf(true) }

    DisposableEffect(Unit) {
        client.connect()
        onDispose { client.disconnect() }
    }

    Scaffold(
        modifier = modifier,
        topBar = {
            TopAppBar(
                title = { Text("World Inspector") },
                actions = {
                    // Toggle outliner / detail
                    IconButton(onClick = { showOutliner = !showOutliner }) {
                        Icon(
                            if (showOutliner) Icons.Default.ViewSidebar else Icons.Default.List,
                            contentDescription = "Toggle panel",
                        )
                    }
                    // Refresh
                    if (connectionState == ConnectionState.CONNECTED) {
                        IconButton(onClick = { client.refreshSceneTree() }) {
                            Icon(Icons.Default.Refresh, contentDescription = "Refresh")
                        }
                    }
                    // Connect / disconnect
                    IconButton(onClick = {
                        if (connectionState == ConnectionState.CONNECTED) {
                            client.disconnect()
                        } else {
                            client.connect()
                        }
                    }) {
                        Icon(
                            when (connectionState) {
                                ConnectionState.CONNECTED -> Icons.Default.Wifi
                                ConnectionState.CONNECTING -> Icons.Default.Sync
                                ConnectionState.DISCONNECTED -> Icons.Default.WifiOff
                                ConnectionState.ERROR -> Icons.Default.ErrorOutline
                            },
                            contentDescription = "Connection",
                        )
                    }
                },
            )
        },
        bottomBar = {
            worldInfo?.let { info ->
                WorldInfoBar(info)
            }
        },
    ) { padding ->
        Row(modifier = Modifier.padding(padding).fillMaxSize()) {
            if (showOutliner) {
                OutlinerScreen(
                    client = client,
                    modifier = Modifier.weight(1f),
                )
                VerticalDivider()
            }
            DetailScreen(
                client = client,
                modifier = Modifier.weight(if (showOutliner) 1.5f else 1f),
            )
        }
    }
}

@Composable
private fun WorldInfoBar(info: com.localgpt.inspector.WorldInfoData) {
    Surface(tonalElevation = 2.dp) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 12.dp, vertical = 6.dp),
            horizontalArrangement = Arrangement.spacedBy(16.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            info.name?.let { name ->
                InfoChip(Icons.Default.Public, name)
            }
            InfoChip(Icons.Default.ViewInAr, "${info.entityCount} entities")
            InfoChip(
                if (info.behaviorState.paused) Icons.Default.PauseCircle else Icons.Default.PlayCircle,
                if (info.behaviorState.paused) "Paused" else "%.1fs".format(info.behaviorState.elapsed),
            )
            info.audio?.let { audio ->
                InfoChip(
                    if (audio.active) Icons.Default.VolumeUp else Icons.Default.VolumeOff,
                    if (audio.active) "${audio.emitterCount} emitters" else "Off",
                )
            }
        }
    }
}

@Composable
private fun InfoChip(icon: androidx.compose.ui.graphics.vector.ImageVector, label: String) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(4.dp),
    ) {
        Icon(
            icon,
            contentDescription = null,
            modifier = Modifier.size(14.dp),
            tint = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        Text(
            label,
            style = MaterialTheme.typography.labelSmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}
