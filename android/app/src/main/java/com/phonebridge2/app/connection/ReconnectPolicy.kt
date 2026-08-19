package com.phonebridge2.app.connection

import kotlin.math.min
import kotlin.time.Duration
import kotlin.time.Duration.Companion.seconds

/** Exponential reconnect policy with a bounded delay. */
class ReconnectPolicy(
    private val initialDelay: Duration = 1.seconds,
    private val maxDelay: Duration = 30.seconds,
) {
    private var attempt = 0

    fun nextDelay(): Duration {
        val multiplier = 1L shl min(attempt, 5)
        attempt++
        return minOf(initialDelay * multiplier.toDouble(), maxDelay)
    }

    fun reset() {
        attempt = 0
    }
}
