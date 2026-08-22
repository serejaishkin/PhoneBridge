package com.phonebridge.app.media

import android.service.notification.NotificationListenerService
import android.service.notification.StatusBarNotification

/**
 * Grants MediaSessionManager access to active media sessions.
 * Android requires Notification Access for getActiveSessions().
 */
class MediaNotificationListenerService : NotificationListenerService() {

    override fun onListenerConnected() {
        super.onListenerConnected()
        MediaControllerBridge.init(this)
    }

    override fun onNotificationPosted(sbn: StatusBarNotification?) {
        MediaControllerBridge.refresh()
    }

    override fun onNotificationRemoved(sbn: StatusBarNotification?) {
        MediaControllerBridge.refresh()
    }
}
