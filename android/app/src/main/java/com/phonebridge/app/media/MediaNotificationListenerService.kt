package com.phonebridge.app.media

import android.service.notification.NotificationListenerService
import android.service.notification.StatusBarNotification
import com.phonebridge.app.network.SignalingClient

/** Keeps the PC-side media state synchronized with Android MediaSession changes. */
class MediaNotificationListenerService : NotificationListenerService() {
    override fun onListenerConnected() {
        super.onListenerConnected()
        MediaControllerBridge.init(this)
        publishState()
    }

    override fun onNotificationPosted(sbn: StatusBarNotification?) {
        MediaControllerBridge.refresh()
        publishState()
    }

    override fun onNotificationRemoved(sbn: StatusBarNotification?) {
        MediaControllerBridge.refresh()
        publishState()
    }

    private fun publishState() {
        val snapshot = MediaControllerBridge.snapshot()
        if (snapshot.isEmpty()) return
        val client = SignalingClient()
        Thread {
            if (client.connectBlocking(SignalingClient.DEFAULT_URL, 1500)) {
                client.sendEvent("media_state", snapshot)
                Thread.sleep(50)
                client.disconnect()
            }
        }.apply { isDaemon = true; start() }
    }
}
