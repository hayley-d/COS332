use chrono::prelude::*;
use chrono::TimeDelta;
use chrono::Utc;
use std::collections::{HashMap, VecDeque};
use std::io::ErrorKind;
use std::net::IpAddr;
use std::sync::Arc;
use tokio::io;
use tokio::sync::Mutex;

const WINDOW_SIZE: chrono::Duration = chrono::Duration::seconds(5);
const LIMIT: usize = 15;

pub struct State {
    request: HashMap<String, HashMap<IpAddr, Arc<Mutex<VecDeque<Utc>>>>>, //HashMap of the end_points that contains a
                                                                          //hashMap of the IpAddresses that contains the VecDeque
}
pub async fn rate_limit(
    state: Arc<Mutex<State>>,
    ip_address: IpAddr,
    end_point: &str,
    timestamp: Utc,
) -> Result<(), Box<dyn std::error::Error>> {
    let request_window: Arc<Mutex<VecDeque<Utc>>> =
        match (match state.lock().await.request.get(end_point) {
            Some(map) => map,
            None => {
                return Err(Box::new(io::Error::new(
                    ErrorKind::Interrupted,
                    "Invalid endpoint",
                )))
            }
        })
        .get(&ip_address)
        {
            Some(list) => list.clone(),
            None => {
                return Err(Box::new(io::Error::new(
                    ErrorKind::Interrupted,
                    "Invalid endpoint",
                )))
            }
        };

    let index: usize = 0;
    loop {
        let old: &Utc = match request_window.lock().await.get(index) {
            Some(t) => t,
            None => break,
        };

        let time_diff = timestamp.signed_duration_since(old); // for some reason
                                                              // singned_duration_since is "no method named found for struct Utc in current scope"

        if time_diff >= TimeDelta::try_seconds(-5) {
            request_window.lock().await.pop_front();
        } else {
            break;
        }
    }

    let result = request_window.lock().await.len() >= LIMIT;

    if !result {
        request_window.lock().await.push_back(timestamp);
        return Ok(());
    } else {
        return Err(Box::new(io::Error::new(
            ErrorKind::Interrupted,
            "Invalid endpoint",
        )));
    }
}
