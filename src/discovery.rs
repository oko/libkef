use bytes::Buf;
use futures::prelude::*;
use log::{debug, info};
use ssdp_client::SearchTarget;
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use url::Url;
use xmltree::Element;

fn get_serial(device: &Element) -> Option<String> {
    match device.get_child("manufacturer")?.get_text()?.as_ref() {
        // we only care about serial numbers from KEF manufactured devices
        "KEF" => {
            // TODO: maybe return "missing" serial numbers when this fails?
            let serial = device.get_child("serialNumber")?.get_text()?.to_string();
            Some(serial)
        }
        _ => None,
    }
}

async fn check_device(url: &Url) -> Option<String> {
    let resp = match reqwest::get(url.as_ref()).await {
        Ok(r) => r,
        Err(_) => return None,
    };
    let content = match resp.bytes().await {
        Ok(c) => c,
        Err(_) => return None,
    };
    let root = match Element::parse(content.clone().reader()) {
        Ok(r) => r,
        Err(_) => return None,
    };
    let dev = root.get_child("device")?;
    match get_serial(dev) {
        Some(serial) => Some(serial),
        None => None,
    }
}

/// Discover
pub async fn discover(timeout: Duration) -> Option<HashMap<Url, String>> {
    let search_target = SearchTarget::RootDevice;

    let mut devices: HashSet<Url> = HashSet::new();
    let mut speakers: HashMap<Url, String> = HashMap::new();

    // discover potential devices and check them out
    // 1. iterate ssdp devices discovered
    // 2. check if they've already been searched in `devices` set
    // 3. if not, poll their `Location` URL and get UPnP metadata
    // 4. if UPnP manufacturer is KEF, assume it's a speaker and try to fetch serial number
    match ssdp_client::search(&search_target, timeout, 2).await {
        Ok(mut resp) => {
            while let Some(response) = resp.next().await {
                match response {
                    Ok(resp) => {
                        let loc = resp.location();
                        debug!("got SSDP response pointing to {}", &loc);
                        match Url::parse(resp.location()) {
                            Ok(u) => {
                                if devices.contains(&u) {
                                    debug!("already crawled {}, skipping", &loc);
                                    continue;
                                };
                                info!("checking {}", resp.location());
                                match check_device(&u).await {
                                    Some(sn) => {
                                        debug!("found KEF device {} at {}", &sn, &loc);
                                        speakers.insert(u.clone(), sn);
                                    }
                                    None => (),
                                };
                                devices.insert(u);
                            }
                            Err(_) => continue,
                        };
                    }
                    Err(_) => continue,
                };
            }

            Some(speakers)
        }
        Err(_) => None,
    }
}
