package com.phonebridge2.app.discovery

import android.content.Context

/** Stores the last successful transport for fast reconnect after a link drop. */
class PreferredRouteStore(context: Context) {
    private val prefs = context.getSharedPreferences("phonebridge_routes", Context.MODE_PRIVATE)

    fun save(deviceId: String, route: TransportKind) {
        prefs.edit().putString(deviceId, route.name).apply()
    }

    fun load(deviceId: String): TransportKind? = prefs.getString(deviceId, null)?.let {
        runCatching { TransportKind.valueOf(it) }.getOrNull()
    }

    fun clear(deviceId: String) { prefs.edit().remove(deviceId).apply() }
}
