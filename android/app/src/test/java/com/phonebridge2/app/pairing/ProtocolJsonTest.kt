package com.phonebridge2.app.pairing

import kotlin.test.Test
import kotlin.test.assertEquals

class ProtocolJsonTest {
    @Test
    fun pairChallengeMatchesRustSerdeShape() {
        val message = Message.PairChallenge(
            Message.PairChallengeData(
                device_id = "pb2-phone",
                fingerprint = "AABBCCDD",
                short_code = "AABB-CCDD",
            )
        )

        assertEquals(
            "{\"type\":\"PairChallenge\",\"data\":{\"device_id\":\"pb2-phone\",\"fingerprint\":\"AABBCCDD\",\"short_code\":\"AABB-CCDD\"}}",
            ProtocolJson.encode(message),
        )
    }

    @Test
    fun pingHasNoDataField() {
        assertEquals("{\"type\":\"Ping\"}", ProtocolJson.encode(Message.Ping))
    }
}
