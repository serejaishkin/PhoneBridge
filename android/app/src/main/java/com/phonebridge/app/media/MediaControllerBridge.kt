package com.phonebridge.app.media

import android.content.Context
import android.media.MediaMetadata
import android.media.session.MediaController
import android.media.session.MediaSessionManager
import android.os.Bundle

/**
 * Phone-side media control only.
 *
 * The PC sends one of: media_play, media_pause, media_play_pause,
 * media_next, media_previous. We operate on the most relevant active
 * MediaSession exposed by Android.
 *
 * Access to active sessions requires the user to enable PhoneBridge in
 * Notification access. No media audio is routed through this class.
 */
object MediaControllerBridge {

    private var manager: MediaSessionManager? = null
    private var controller: MediaController? = null

    fun init(context: Context) {
        manager = context.getSystemService(Context.MEDIA_SESSION_SERVICE) as MediaSessionManager
        refresh()
    }

    fun refresh(): Boolean {
        val sessionManager = manager ?: return false
        return try {
            val sessions = sessionManager.getActiveSessions(null)
            controller = sessions.firstOrNull { it.playbackState != null }
                ?: sessions.firstOrNull()
            controller != null
        } catch (_: SecurityException) {
            controller = null
            false
        }
    }

    fun handleCommand(command: String): Boolean {
        if (!refresh()) return false
        val c = controller ?: return false
        val transport = c.transportControls

        return when (command) {
            "media_play" -> {
                transport.play()
                true
            }
            "media_pause" -> {
                transport.pause()
                true
            }
            "media_play_pause" -> {
                val playing = c.playbackState?.state == android.media.session.PlaybackState.STATE_PLAYING
                if (playing) transport.pause() else transport.play()
                true
            }
            "media_next" -> {
                transport.skipToNext()
                true
            }
            "media_previous" -> {
                transport.skipToPrevious()
                true
            }
            else -> false
        }
    }

    fun snapshot(): Map<String, String> {
        if (!refresh()) return emptyMap()
        val c = controller ?: return emptyMap()
        val metadata = c.metadata
        val state = c.playbackState?.state

        return buildMap {
            put("package", c.packageName)
            put("state", playbackStateName(state))
            metadata?.getString(MediaMetadata.METADATA_KEY_TITLE)?.let { put("title", it) }
            metadata?.getString(MediaMetadata.METADATA_KEY_ARTIST)?.let { put("artist", it) }
            metadata?.getString(MediaMetadata.METADATA_KEY_ALBUM)?.let { put("album", it) }
        }
    }

    private fun playbackStateName(state: Int?): String = when (state) {
        android.media.session.PlaybackState.STATE_PLAYING -> "playing"
        android.media.session.PlaybackState.STATE_PAUSED -> "paused"
        android.media.session.PlaybackState.STATE_BUFFERING -> "buffering"
        android.media.session.PlaybackState.STATE_NONE -> "none"
        else -> "unknown"
    }
}
