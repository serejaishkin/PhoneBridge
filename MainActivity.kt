package com.phonebridge.app

import android.Manifest
import android.content.Context
import android.content.Intent
import android.media.projection.MediaProjectionManager
import android.net.wifi.WifiManager
import android.os.Build
import android.os.Bundle
import android.text.format.Formatter
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import androidx.core.content.ContextCompat

class MainActivity : ComponentActivity() {
    private val requestPermissions = registerForActivityResult(
        ActivityResultContracts.RequestMultiplePermissions()
    ) { }

    private val mediaProjectionLauncher = registerForActivityResult(
        ActivityResultContracts.StartActivityForResult()
    ) { result ->
        if (result.resultCode == RESULT_OK && result.data != null) {
            val pcIp = getSharedPreferences("prefs", Context.MODE_PRIVATE)
                .getString("pc_ip", "192.168.137.1") ?: "192.168.137.1"

            val serviceIntent = Intent(this, AudioCaptureService::class.java).apply {
                putExtra("code", result.resultCode)
                @Suppress("DEPRECATION")
                putExtra("data", result.data)
                putExtra("pc_ip", pcIp)
            }
            ContextCompat.startForegroundService(this, serviceIntent)
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        val perms = mutableListOf(
            Manifest.permission.RECORD_AUDIO,
            Manifest.permission.READ_PHONE_STATE,
            Manifest.permission.CALL_PHONE
        )
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            perms.add(Manifest.permission.ANSWER_PHONE_CALLS)
        }
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            perms.add(Manifest.permission.POST_NOTIFICATIONS)
        }
        requestPermissions.launch(perms.toTypedArray())

        setContent {
            PhoneBridgeUI(
                onStart = { ip -> startCapture(ip) },
                onStop = { stopCapture() }
            )
        }
    }

    private fun startCapture(pcIp: String) {
        getSharedPreferences("prefs", Context.MODE_PRIVATE).edit()
            .putString("pc_ip", pcIp).apply()

        val mgr = getSystemService(Context.MEDIA_PROJECTION_SERVICE) as MediaProjectionManager
        mediaProjectionLauncher.launch(mgr.createScreenCaptureIntent())
    }

    private fun stopCapture() {
        stopService(Intent(this, AudioCaptureService::class.java))
    }
}

@Composable
fun PhoneBridgeUI(onStart: (String) -> Unit, onStop: () -> Unit) {
    val ctx = LocalContext.current
    var pcIp by remember {
        mutableStateOf(
            ctx.getSharedPreferences("prefs", Context.MODE_PRIVATE)
                .getString("pc_ip", "192.168.137.1") ?: "192.168.137.1"
        )
    }
    var running by remember { mutableStateOf(false) }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        Text("PhoneBridge V1", style = MaterialTheme.typography.headlineMedium)

        OutlinedTextField(
            value = pcIp,
            onValueChange = { pcIp = it },
            label = { Text("IP адрес ПК") },
            modifier = Modifier.fillMaxWidth()
        )

        val wifiManager = ctx.applicationContext.getSystemService(Context.WIFI_SERVICE) as WifiManager
        val ip = Formatter.formatIpAddress(wifiManager.connectionInfo.ipAddress)
        Text("Твой IP: $ip", style = MaterialTheme.typography.bodySmall)

        Button(
            onClick = {
                if (!running) onStart(pcIp) else onStop()
                running = !running
            },
            modifier = Modifier.fillMaxWidth()
        ) {
            Text(if (running) "Остановить" else "Запустить трансляцию")
        }

        Text(
            "ПК и телефон должны быть в одной сети",
            style = MaterialTheme.typography.bodySmall
        )
    }
}
