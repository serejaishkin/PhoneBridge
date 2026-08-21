plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    id("org.jetbrains.kotlin.plugin.serialization")
}

android {
    namespace = "com.phonebridge.app"
    compileSdk = 34

    defaultConfig {
        applicationId = "com.phonebridge.app"
        // minSdk 31, потому что control-plane опирается на TelephonyCallback
        // (замена deprecated PhoneStateListener) — это Android 12 (API 31)+.
        // См. AI_HANDOFF_GUI.md, п.4.1 — реальная поддержка звонков всё равно
        // зависит от Bluetooth HFP на стороне PC, так что более старые Android
        // здесь не приоритет.
        minSdk = 31
        targetSdk = 34
        versionCode = 1
        versionName = "0.1.0-skeleton"
    }

    buildFeatures {
        compose = true
    }
    composeOptions {
        kotlinCompilerExtensionVersion = "1.5.14"
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    kotlinOptions {
        jvmTarget = "17"
    }

    packaging {
        resources {
            excludes += "META-INF/versions/9/OSGI-INF/MANIFEST.MF"
        }
    }
}

dependencies {
    implementation("androidx.core:core-ktx:1.13.1")
    implementation("androidx.activity:activity-compose:1.9.0")
    implementation(platform("androidx.compose:compose-bom:2024.06.00"))
    implementation("androidx.compose.ui:ui")
    implementation("androidx.compose.material3:material3")
    implementation("androidx.lifecycle:lifecycle-runtime-ktx:2.8.2")
    implementation("androidx.lifecycle:lifecycle-viewmodel-compose:2.8.2")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.8.1")
    implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.6.3")
    implementation("org.java-websocket:Java-WebSocket:1.5.6")
    // Генерация самоподписанного X.509-сертификата для TLS-пейринга.
    implementation("org.bouncycastle:bcpkix-jdk18on:1.78.1")
}
