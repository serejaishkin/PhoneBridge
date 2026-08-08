package com.phonebridge.app.call

import android.telecom.Call
import android.telecom.InCallService

class CallService : InCallService() {

    override fun onCallAdded(call: Call) {
        super.onCallAdded(call)
        // Handle call state changes
    }

    override fun onCallRemoved(call: Call) {
        super.onCallRemoved(call)
    }
}
