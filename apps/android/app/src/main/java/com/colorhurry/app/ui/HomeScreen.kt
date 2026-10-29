package com.colorhurry.app.ui

import android.Manifest
import android.os.Build
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.colorhurry.shared.Screen
import com.colorhurry.app.shell.GameController
import com.google.accompanist.permissions.ExperimentalPermissionsApi
import com.google.accompanist.permissions.rememberMultiplePermissionsState

@OptIn(ExperimentalPermissionsApi::class)
@Composable
fun HomeScreen(game: GameController) {
    val permissions = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
        listOf(
            Manifest.permission.BLUETOOTH_SCAN,
            Manifest.permission.BLUETOOTH_CONNECT,
            Manifest.permission.BLUETOOTH_ADVERTISE,
        )
    } else {
        listOf(Manifest.permission.ACCESS_FINE_LOCATION)
    }
    val permissionState = rememberMultiplePermissionsState(permissions)

    LaunchedEffect(Unit) {
        if (!permissionState.allPermissionsGranted) {
            permissionState.launchMultiplePermissionRequest()
        }
        game.boot()
    }

    val view by game.view.collectAsState()
    val bleStatus by game.bleStatus.collectAsState()

    val title = when (view.screen) {
        Screen.MODESELECT -> "Mode select"
        Screen.PICKER -> "Picker"
        Screen.RECEIVER -> "Receiver"
    }

    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(
                Brush.radialGradient(
                    colors = listOf(Color(0xFF1B2A3A), AppColors.Bg),
                    center = androidx.compose.ui.geometry.Offset(0.2f, 0f),
                    radius = 1200f,
                ),
            ),
    ) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(16.dp),
        ) {
            Text(
                text = "Color Hurry",
                color = AppColors.Text,
                fontSize = 22.sp,
                fontWeight = FontWeight.SemiBold,
            )
            Spacer(Modifier.height(4.dp))
            Text(
                text = "Same entry as the board: Skip cycles role/channel, Apply starts. Match a CPB over BLE (protocol v4).",
                color = AppColors.Muted,
                fontSize = 13.sp,
            )
            Spacer(Modifier.height(10.dp))
            Text(text = bleStatus, color = AppColors.Muted, fontSize = 12.sp)
            Spacer(Modifier.height(12.dp))

            Column(
                modifier = Modifier
                    .fillMaxSize()
                    .clip(RoundedCornerShape(20.dp))
                    .background(AppColors.Panel)
                    .border(1.dp, AppColors.Border, RoundedCornerShape(20.dp))
                    .padding(14.dp),
            ) {
                Text(text = title, color = AppColors.Text, fontSize = 17.sp, fontWeight = FontWeight.SemiBold)
                Spacer(Modifier.height(4.dp))
                Text(text = game.modeHint(view), color = AppColors.Muted, fontSize = 13.sp)
                Spacer(Modifier.height(8.dp))
                Box(
                    modifier = Modifier
                        .weight(1f)
                        .fillMaxWidth(),
                    contentAlignment = Alignment.Center,
                ) {
                    LedRing(leds = view.leds, modifier = Modifier.fillMaxSize())
                }
                Spacer(Modifier.height(12.dp))
                Row(horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                    ActionButton(
                        label = "Button A (skip)",
                        secondary = true,
                        onClick = game::skip,
                        modifier = Modifier.weight(1f),
                    )
                    ActionButton(
                        label = "Button B (apply)",
                        secondary = false,
                        onClick = game::activate,
                        modifier = Modifier.weight(1f),
                    )
                }
            }
        }
    }
}

@Composable
private fun ActionButton(
    label: String,
    secondary: Boolean,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Box(
        modifier = modifier
            .clip(RoundedCornerShape(10.dp))
            .background(if (secondary) AppColors.Secondary else AppColors.Primary)
            .clickable(onClick = onClick)
            .padding(vertical = 14.dp, horizontal = 12.dp),
        contentAlignment = Alignment.Center,
    ) {
        Text(text = label, color = Color.White, fontWeight = FontWeight.SemiBold, fontSize = 14.sp)
    }
}
