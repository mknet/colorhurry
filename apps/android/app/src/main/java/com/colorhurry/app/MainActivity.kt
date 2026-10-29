package com.colorhurry.app

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.viewModels
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.Surface
import androidx.compose.ui.Modifier
import com.colorhurry.app.shell.GameController
import com.colorhurry.app.ui.AppColors
import com.colorhurry.app.ui.HomeScreen

class MainActivity : ComponentActivity() {
    private val game: GameController by viewModels()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent {
            Surface(modifier = Modifier.fillMaxSize(), color = AppColors.Bg) {
                HomeScreen(game)
            }
        }
    }
}
