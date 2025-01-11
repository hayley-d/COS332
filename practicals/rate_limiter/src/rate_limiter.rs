use chrono::prelude::*;
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
    endpoints: HashMap<String, Arc<Mutex<HashMap<IpAddr, Arc<Mutex<VecDeque<DateTime<Utc>>>>>>>>, //HashMap of the end_points that contains a
                                                                                                  //hashMap of the IpAddresses that contains the VecDeque
}

impl State {
    pub fn new() -> Arc<Mutex<Self>> {
        let mut end_points: HashMap<
            String,
            Arc<Mutex<HashMap<IpAddr, Arc<Mutex<VecDeque<DateTime<Utc>>>>>>>,
        > = HashMap::new();

        end_points.insert(String::from("/"), Arc::new(Mutex::new(HashMap::new())));
        end_points.insert(String::from("/home"), Arc::new(Mutex::new(HashMap::new())));
        end_points.insert(String::from("/fib"), Arc::new(Mutex::new(HashMap::new())));
        end_points.insert(String::from("other"), Arc::new(Mutex::new(HashMap::new())));
        end_points.insert(
            String::from("/signup"),
            Arc::new(Mutex::new(HashMap::new())),
        );
        end_points.insert(String::from("/login"), Arc::new(Mutex::new(HashMap::new())));

        end_points.insert(
            String::from("/coffee"),
            Arc::new(Mutex::new(HashMap::new())),
        );

        return Arc::new(Mutex::new(State {
            endpoints: end_points,
        }));
    }
}
pub async fn rate_limit(
    state: Arc<Mutex<State>>,
    ip_address: IpAddr,
    end_point: &str,
    timestamp: DateTime<Utc>,
) -> Result<(), Box<dyn std::error::Error>> {
    let request_window: Arc<Mutex<VecDeque<DateTime<Utc>>>> =
        match (match state.lock().await.endpoints.get(end_point) {
            Some(map) => map.lock().await,
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
                // IP not yet listed
                // add IP
                state
                    .lock()
                    .await
                    .endpoints
                    .get(end_point)
                    .unwrap()
                    .lock()
                    .await
                    .insert(ip_address.clone(), Arc::new(Mutex::new(VecDeque::new())));
                return Err(Box::new(io::Error::new(
                    ErrorKind::Interrupted,
                    "Invalid endpoint",
                )));
            }
        };

    let index: usize = 0;
    loop {
        let gaurd = request_window.lock().await;
        let old: &DateTime<Utc> = match gaurd.get(index) {
            Some(t) => t,
            None => break,
        };

        let time_diff = timestamp.signed_duration_since(old);

        if time_diff >= WINDOW_SIZE {
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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use chrono::Duration;

    use super::*;

    async fn setup_state() -> Arc<Mutex<State>> {
        let mut request_map: HashMap<String, HashMap<IpAddr, Arc<Mutex<VecDeque<DateTime<Utc>>>>>> =
            HashMap::new();
        let ip = IpAddr::from_str("127.0.0.1").unwrap();
        let endpoint = "/test".to_string();

        let mut inner_map: Arc<Mutex<HashMap<IpAddr, Arc<Mutex<VecDeque<DateTime<Utc>>>>>>> =
            Arc::new(Mutex::new(HashMap::new()));

        inner_map
            .lock()
            .await
            .insert(ip, Arc::new(Mutex::new(VecDeque::new())));

        request_map.insert(endpoint, inner_map);

        Arc::new(Mutex::new(State {
            endpoints: request_map,
        }))
    }

    #[tokio::test]
    async fn test_valid_request() {
        let state = setup_state();
        let ip = IpAddr::from_str("127.0.0.1").unwrap();
        let endpoint = "/test";
        let timestamp = Utc::now();

        let result = rate_limit(state.clone(), ip, endpoint, timestamp).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_exceed_rate_limit() {
        let state = setup_state();
        let ip = IpAddr::from_str("127.0.0.1").unwrap();
        let endpoint = "/test";
        let timestamp = Utc::now();

        for _ in 0..LIMIT {
            rate_limit(state.clone(), ip, endpoint, timestamp)
                .await
                .unwrap();
        }

        let result = rate_limit(state.clone(), ip, endpoint, timestamp).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_window_expires() {
        let state = setup_state();
        let ip = IpAddr::from_str("127.0.0.1").unwrap();
        let endpoint = "/test";

        let mut timestamp = Utc::now();
        for _ in 0..LIMIT {
            rate_limit(state.clone(), ip, endpoint, timestamp)
                .await
                .unwrap();
            timestamp = timestamp + Duration::seconds(1);
        }

        // Advance the timestamp beyond the window size
        timestamp = timestamp + Duration::seconds(6);

        let result = rate_limit(state.clone(), ip, endpoint, timestamp).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_invalid_endpoint() {
        let state = setup_state();
        let ip = IpAddr::from_str("127.0.0.1").unwrap();
        let endpoint = "/invalid";
        let timestamp = Utc::now();

        let result = rate_limit(state.clone(), ip, endpoint, timestamp).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_invalid_ip() {
        let state = setup_state();
        let ip = IpAddr::from_str("192.168.0.1").unwrap();
        let endpoint = "/test";
        let timestamp = Utc::now();

        let result = rate_limit(state.clone(), ip, endpoint, timestamp).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mixed_ips() {
        let state = setup_state();
        let ip1 = IpAddr::from_str("127.0.0.1").unwrap();
        let ip2 = IpAddr::from_str("192.168.0.1").unwrap();
        let endpoint = "/test";
        let timestamp = Utc::now();

        for _ in 0..LIMIT {
            rate_limit(state.clone(), ip1, endpoint, timestamp)
                .await
                .unwrap();
        }

        // Another IP should not affect the rate limit for the first IP
        let result = rate_limit(state.clone(), ip2, endpoint, timestamp).await;
        assert!(result.is_err());
    }
}
