package com.phonebridge2.app.connection

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch

/** Keeps trying known routes without requiring the user to reopen pairing. */
class AutoReconnect(
    private val scope: CoroutineScope,
    private val connectRoute: (ConnectionRoute) -> Unit,
) {
    private var job: Job? = null

    fun start(routes: RouteSet) {
        stop()
        job = scope.launch {
            var attempt = 0
            while (isActive) {
                val route = routes.routes.getOrNull(attempt % routes.routes.size) ?: break
                connectRoute(route)
                attempt++
                delay((2000L shl minOf(attempt, 4)).coerceAtMost(30_000L))
            }
        }
    }

    fun stop() { job?.cancel(); job = null }
}
