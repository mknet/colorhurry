package com.colorhurry.app.ui

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.drawscope.Stroke
import com.colorhurry.shared.Rgb
import kotlin.math.PI
import kotlin.math.cos
import kotlin.math.min
import kotlin.math.sin

@Composable
fun LedRing(leds: List<Rgb>, modifier: Modifier = Modifier) {
    Canvas(modifier = modifier.aspectRatio(1f).fillMaxSize()) {
        val sizePx = min(size.width, size.height)
        val center = Offset(size.width / 2f, size.height / 2f)
        val radius = sizePx / 2f

        drawCircle(
            brush = Brush.radialGradient(
                colors = listOf(
                    Color(0xFF2A3542),
                    Color(0xFF151C24),
                    Color(0xFF0C1016),
                ),
                center = center,
                radius = radius,
            ),
            radius = radius,
            center = center,
        )
        drawCircle(
            color = AppColors.RingBorder,
            radius = radius - 1.5f,
            center = center,
            style = Stroke(width = 3f),
        )
        drawCircle(color = Color(0xFF2A3542), radius = radius * 0.18f, center = center)

        val ledR = sizePx * 0.06f
        val orbit = radius * 0.72f
        leds.forEachIndexed { i, c ->
            val angle = -PI / 2 + i * (2 * PI / leds.size)
            val pos = Offset(
                center.x + cos(angle).toFloat() * orbit,
                center.y + sin(angle).toFloat() * orbit,
            )
            val fill = Color(c.r.toInt(), c.g.toInt(), c.b.toInt())
            drawCircle(color = fill, radius = ledR, center = pos)
            drawCircle(
                color = Color(0xFF314050),
                radius = ledR,
                center = pos,
                style = Stroke(width = 1f),
            )
        }
    }
}
