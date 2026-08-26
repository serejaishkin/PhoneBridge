//! Linux implementation of HFP client-role support detection via BlueZ D-Bus.
//!
//! BlueZ exposes each adapter under `/org/bluez/hciX` with an `Adapter1`
//! interface whose `UUIDs` property lists the profiles the stack can serve.
//! Evidence-based decision, consistent with NEXT_STEPS.md:
//!
//! 1. System bus unreachable, or BlueZ service missing -> Unsupported: there is
//!    no Bluetooth stack at all, so call audio cannot work.
//! 2. An adapter advertises the Hands-Free profile UUIDs (111E HF / 111F AG)
//!    -> Supported.
//! 3. Adapters exist but none advertises Hands-Free -> Unknown: depends on the
//!    PulseAudio/PipeWire/oFono setup; UI must show "needs manual verification".

use crate::protocol::HfpSupport;
use anyhow::Context;
use std::collections::HashMap;
use zbus::zvariant::{ObjectPath, OwnedObjectPath, Value};

const BLUEZ_SERVICE: &str = "org.bluez";
const ADAPTER_IFACE: &str = "org.bluez.Adapter1";
const PROPS_IFACE: &str = "org.freedesktop.DBus.Properties";
/// Standard Hands-Free UUIDs: 111E = HF unit role, 111F = Audio Gateway role.
const HANDS_FREE_UUIDS: [&str; 2] = [
    "0000111e-0000-1000-8000-00805f9b34fb",
    "0000111f-0000-1000-8000-00805f9b34fb",
];

pub async fn detect() -> HfpSupport {
    match detect_inner().await {
        Ok(support) => {
            log::info!("Linux HFP support detection result: {support:?}");
            support
        }
        Err(e) => {
            log::warn!("Linux HFP detection failed, reporting Unsupported: {e}");
            HfpSupport::Unsupported
        }
    }
}

async fn detect_inner() -> anyhow::Result<HfpSupport> {
    let connection = zbus::Connection::system().await.context("connecting to the system D-Bus bus")?;

    // Enumerate managed objects; we only need adapter paths here, property
    // values are fetched per adapter so no deep parsing is required.
    let reply = connection
        .call_method(
            Some(BLUEZ_SERVICE),
            ObjectPath::try_from("/").context("building root object path")?,
            Some("org.freedesktop.DBus.ObjectManager"),
            "GetManagedObjects",
            &(),
        )
        .await
        .context("calling BlueZ ObjectManager (is the bluetooth service running?)")?;
    let objects: HashMap<OwnedObjectPath, HashMap<String, HashMap<String, zbus::zvariant::OwnedValue>>> =
        reply.body().deserialize().context("parsing GetManagedObjects reply")?;

    let adapter_paths: Vec<String> = objects
        .iter()
        .filter(|(_path, interfaces)| interfaces.contains_key(ADAPTER_IFACE))
        .map(|(path, _)| path.as_str().to_string())
        .collect();

    if adapter_paths.is_empty() {
        return Ok(HfpSupport::Unsupported);
    }

    for path in adapter_paths {
        match adapter_supports_hands_free(&connection, &path).await {
            Ok(true) => return Ok(HfpSupport::Supported),
            Ok(false) => continue,
            // One unreadable adapter should not fail the whole detection.
            Err(e) => log::debug!("skipping adapter {path}: {e}"),
        }
    }
    Ok(HfpSupport::Unknown)
}

async fn adapter_supports_hands_free(connection: &zbus::Connection, path: &str) -> anyhow::Result<bool> {
    let reply = connection
        .call_method(
            Some(BLUEZ_SERVICE),
            ObjectPath::try_from(path).with_context(|| format!("invalid adapter path {path}"))?,
            Some(PROPS_IFACE),
            "Get",
            &(ADAPTER_IFACE, "UUIDs"),
        )
        .await
        .context("reading Adapter1.UUIDs")?;

    // Properties.Get returns a boxed variant ('v'); tolerate a bare array too.
    let body = reply.body();
    let wrapped: Value = body.deserialize().context("parsing UUIDs property")?;
    let array = match wrapped {
        Value::Value(inner) => match *inner {
            Value::Array(array) => Some(array),
            _ => None,
        },
        Value::Array(array) => Some(array),
        _ => None,
    };

    let Some(array) = array else { return Ok(false) };
    for item in array.iter() {
        if let Value::Str(uuid) = item {
            let uuid = uuid.as_str().to_lowercase();
            if HANDS_FREE_UUIDS.contains(&uuid.as_str()) {
                return Ok(true);
            }
        }
    }
    Ok(false)
}
