package com.phonebridge2.app.discovery

import android.content.Context

/** Stores the user's selected PC independently from ephemeral discovery results. */
class PeerConnectionStore(context: Context) {
    private val prefs = context.getSharedPreferences("phonebridge_connection", Context.MODE_PRIVATE)

    fun select(deviceId: String) {
        prefs.edit().putString("selectedDeviceId", deviceId).apply()
    }

    fun selectedDeviceId(): String? = prefs.getString("selectedDeviceId", null)

    fun clear() {
        prefs.edit().remove("selectedDeviceId").apply()
    }
}
