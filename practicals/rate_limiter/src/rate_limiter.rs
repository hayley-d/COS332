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

#[derive(Debug)]
pub struct State {
    endpoints: HashMap<String, Arc<Mutex<HashMap<IpAddr, Arc<Mutex<VecDeque<DateTime<Utc>>>>>>>>,
    lamport_timestamp: u64,
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
            lamport_timestamp: 0,
        }));
    }

    pub fn increment_time(&mut self) -> u64 {
        let temp = self.lamport_timestamp;
        self.lamport_timestamp += 1;
        return temp;
    }

    pub async fn clear_ips(&mut self) {
        let now = Utc::now();
        let mut removable_endpoints: Vec<(String, IpAddr)> = Vec::new();
        for (endpoint, ip_map) in &mut self.endpoints {
            let ip_map_guard = ip_map.lock().await;
            for (ip, request_window) in ip_map_guard.iter() {
                let mut window_guard = request_window.lock().await;

                while let Some(&oldest) = window_guard.front() {
                    if oldest + WINDOW_SIZE < now {
                        window_guard.pop_front();
                    } else {
                        break;
                    }
                }

                if window_guard.is_empty() {
                    removable_endpoints.push((endpoint.clone(), *ip));
                }
            }
        }

        // Remove stale IPs
        for (endpoint, ip) in removable_endpoints {
            if let Some(ip_map) = self.endpoints.get_mut(&endpoint) {
                let mut ip_map_guard = ip_map.lock().await;
                ip_map_guard.remove(&ip);
            }
        }
    }
}
pub async fn rate_limit(
    state: Arc<Mutex<State>>,
    ip_address: IpAddr,
    end_point: &str,
    timestamp: DateTime<Utc>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut state = state.lock().await;
    if state.increment_time() % 100 == 0 {
        state.clear_ips().await;
    }

    let end_point_map: Arc<Mutex<HashMap<IpAddr, Arc<Mutex<VecDeque<DateTime<Utc>>>>>>> =
        match state.endpoints.get(end_point) {
            Some(map) => map.clone(),
            None => {
                return Err(Box::new(io::Error::new(
                    ErrorKind::Interrupted,
                    "Invalid endpoint",
                )))
            }
        };

    let mut end_point_map = end_point_map.lock().await;

    let request_window: Arc<Mutex<VecDeque<DateTime<Utc>>>> = match end_point_map.get(&ip_address) {
        Some(list) => list.clone(),
        None => {
            // IP not yet listed -> add IP to the map
            end_point_map.insert(ip_address.clone(), Arc::new(Mutex::new(VecDeque::new())));

            end_point_map
                .get(&ip_address)
                .unwrap()
                .lock()
                .await
                .push_back(timestamp);

            return Ok(());
        }
    };
    let mut gaurd = request_window.lock().await;

    let mut index: usize = 0;

    loop {
        let old: &DateTime<Utc> = match gaurd.get(index) {
            Some(t) => t,
            None => break,
        };

        let time_diff = timestamp.signed_duration_since(old);

        if time_diff >= WINDOW_SIZE {
            gaurd.pop_front();
        } else {
            break;
        }

        index += 1;
    }

    let result = gaurd.len() >= LIMIT;

    if !result {
        gaurd.push_back(timestamp);
        return Ok(());
    } else {
        return Err(Box::new(io::Error::new(
            ErrorKind::Interrupted,
            "Limit exeeded",
        )));
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use chrono::Duration;
    use tokio::task::JoinSet;

    use super::*;

    async fn setup_state() -> Arc<Mutex<State>> {
        let mut request_map: HashMap<
            String,
            Arc<Mutex<HashMap<IpAddr, Arc<Mutex<VecDeque<DateTime<Utc>>>>>>>,
        > = HashMap::new();
        let ip = IpAddr::from_str("127.0.0.1").unwrap();
        let endpoint = "/test".to_string();

        let inner_map: Arc<Mutex<HashMap<IpAddr, Arc<Mutex<VecDeque<DateTime<Utc>>>>>>> =
            Arc::new(Mutex::new(HashMap::new()));

        inner_map
            .lock()
            .await
            .insert(ip, Arc::new(Mutex::new(VecDeque::new())));

        request_map.insert(endpoint, inner_map);

        Arc::new(Mutex::new(State {
            endpoints: request_map,
            lamport_timestamp: 0,
        }))
    }

    #[tokio::test]
    async fn test_valid_request() {
        let state = setup_state().await;
        let ip = IpAddr::from_str("127.0.0.1").unwrap();
        let endpoint = "/test";
        let timestamp = Utc::now();

        let result = rate_limit(state.clone(), ip, endpoint, timestamp).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_exceed_rate_limit() {
        let state = setup_state().await;
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

    // not done
    #[tokio::test]
    async fn test_window_expires() {
        let state = setup_state().await;
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
    async fn test_exact_window_boundary() {
        let state = setup_state().await;
        let ip = IpAddr::from_str("127.0.0.1").unwrap();
        let now = Utc::now();
        let end_point = "/test";

        for _ in 0..LIMIT {
            let result = rate_limit(state.clone(), ip, end_point, now).await;
            assert!(result.is_ok());
        }

        let later = now + WINDOW_SIZE;
        assert!(rate_limit(state.clone(), ip, "/test", later).await.is_ok());
    }

    #[tokio::test]
    async fn test_burst_traffic() {
        let state = setup_state().await;
        let ip = IpAddr::from_str("127.0.0.1").unwrap();
        let ip2 = IpAddr::from_str("127.0.0.2").unwrap();

        let now = Utc::now();

        for _ in 0..LIMIT {
            assert!(rate_limit(state.clone(), ip, "/test", now).await.is_ok());
            assert!(rate_limit(state.clone(), ip2, "/test", now).await.is_ok());
        }

        assert!(rate_limit(state.clone(), ip, "/test", now).await.is_err());
        assert!(rate_limit(state.clone(), ip, "/test", now).await.is_err());
        assert!(rate_limit(state.clone(), ip2, "/test", now).await.is_err());
        assert!(rate_limit(state.clone(), ip2, "/test", now).await.is_err());
    }

    #[tokio::test]
    async fn test_invalid_endpoint() {
        let state = setup_state().await;
        let ip = IpAddr::from_str("127.0.0.1").unwrap();
        let endpoint = "/invalid";
        let timestamp = Utc::now();

        let result = rate_limit(state.clone(), ip, endpoint, timestamp).await;
        assert!(result.is_err());
    }

    // not done
    #[tokio::test]
    async fn test_invalid_ip() {
        let state = setup_state().await;
        let ip = IpAddr::from_str("192.168.0.1").unwrap();
        let endpoint = "/test";
        let timestamp = Utc::now();

        let result = rate_limit(state.clone(), ip, endpoint, timestamp).await;
        assert!(result.is_ok());
    }

    // not done
    #[tokio::test]
    async fn test_mixed_ips() {
        let state = setup_state().await;

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
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_under_limit() {
        let state = setup_state().await;
        let ip = IpAddr::from_str("127.0.0.1").unwrap();
        let now = Utc::now();

        for _ in 0..(LIMIT - 1) {
            assert!(rate_limit(state.clone(), ip, "/test", now).await.is_ok());
        }
    }

    #[tokio::test]
    async fn test_multiple_ips_endpoints() {
        let state = setup_state().await;
        let ip1 = IpAddr::from_str("127.0.0.1").unwrap();
        let ip2 = IpAddr::from_str("192.168.0.1").unwrap();
        let now = Utc::now();

        for _ in 0..LIMIT {
            assert!(rate_limit(state.clone(), ip1, "/test", now).await.is_ok());
            assert!(rate_limit(state.clone(), ip2, "/test", now).await.is_ok());
        }

        assert!(rate_limit(state.clone(), ip1, "/test", now).await.is_err());
        assert!(rate_limit(state.clone(), ip2, "/test", now).await.is_err());
    }

    #[tokio::test]
    async fn test_sliding_window_precision() {
        let state = setup_state().await;
        let ip = IpAddr::from_str("127.0.0.1").unwrap();
        let now = Utc::now();

        for i in 0..LIMIT {
            let timestamp = now + chrono::Duration::milliseconds(i as i64 * 1000);
            assert!(rate_limit(state.clone(), ip, "/test", timestamp)
                .await
                .is_ok());
        }

        let later = now + WINDOW_SIZE - chrono::Duration::milliseconds(1000);
        assert!(rate_limit(state.clone(), ip, "/test", later).await.is_ok());
    }

    #[tokio::test]
    async fn test_large_volume_stress() {
        let state = setup_state().await;
        let ip = IpAddr::from_str("127.0.0.1").unwrap();
        let now = Utc::now();

        for _ in 0..10000 {
            let result = rate_limit(state.clone(), ip, "/test", now).await;
            if result.is_err() {
                break;
            }
        }
    }

    #[tokio::test]
    async fn test_concurrent_requests() {
        let state = setup_state().await;
        let ip = IpAddr::from_str("127.0.0.1").unwrap();

        let mut tasks = JoinSet::new();

        for _ in 0..LIMIT {
            let clone_state = state.clone();
            tasks.spawn(async move {
                let _ = rate_limit(clone_state.clone(), ip.clone(), "/test", Utc::now()).await;
            });
        }

        while let Some(res) = tasks.join_next().await {
            assert!(res.is_ok());
        }

        let result = rate_limit(state, ip, "/test", Utc::now()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_clear_ips_removes_stale_ips() {
        let state = setup_state().await;
        let ip1 = IpAddr::from_str("127.0.0.1").unwrap();
        let ip2 = IpAddr::from_str("192.168.0.1").unwrap();
        let now = Utc::now();

        // Add stale IP (no requests in the last 5 seconds)
        state
            .lock()
            .await
            .endpoints
            .entry("/test".to_string())
            .or_default()
            .lock()
            .await
            .insert(
                ip1,
                Arc::new(Mutex::new(VecDeque::from(vec![
                    now - WINDOW_SIZE - Duration::seconds(1),
                ]))),
            );

        // Add active IP (requests within the last 5 seconds)
        state
            .lock()
            .await
            .endpoints
            .entry("/test".to_string())
            .or_default()
            .lock()
            .await
            .insert(
                ip2,
                Arc::new(Mutex::new(VecDeque::from(vec![now - Duration::seconds(3)]))),
            );

        // Run clear_ips
        state.lock().await.clear_ips().await;

        // Verify results
        let state = state.lock().await;
        let endpoints = state.endpoints.get("/test").unwrap().lock().await;
        assert!(!endpoints.contains_key(&ip1));
        assert!(endpoints.contains_key(&ip2));
    }

    #[tokio::test]
    async fn test_clear_ips_no_stale_ips() {
        let state = setup_state().await;
        let ip = IpAddr::from_str("127.0.0.1").unwrap();
        let now = Utc::now();

        // Add an active IP
        state
            .lock()
            .await
            .endpoints
            .entry("/test".to_string())
            .or_default()
            .lock()
            .await
            .insert(
                ip,
                Arc::new(Mutex::new(VecDeque::from(vec![now - Duration::seconds(1)]))),
            );

        // Run clear_ips
        state.lock().await.clear_ips().await;

        // Verify no changes
        let state = state.lock().await;
        let endpoints = state.endpoints.get("/test").unwrap().lock().await;
        assert!(endpoints.contains_key(&ip));
    }

    #[tokio::test]
    async fn test_clear_ips_all_stale_ips() {
        let state = setup_state().await;
        let ip1 = IpAddr::from_str("127.0.0.1").unwrap();
        let ip2 = IpAddr::from_str("192.168.0.1").unwrap();
        let now = Utc::now();

        // Add stale IPs
        state
            .lock()
            .await
            .endpoints
            .entry("/test".to_string())
            .or_default()
            .lock()
            .await
            .insert(
                ip1,
                Arc::new(Mutex::new(VecDeque::from(vec![
                    now - WINDOW_SIZE - Duration::seconds(1),
                ]))),
            );
        state
            .lock()
            .await
            .endpoints
            .entry("/test".to_string())
            .or_default()
            .lock()
            .await
            .insert(ip2, Arc::new(Mutex::new(VecDeque::new()))); // Empty queue

        // Run clear_ips
        state.lock().await.clear_ips().await;

        // Verify all stale IPs are removed
        let state = state.lock().await;
        let endpoints = state.endpoints.get("/test").unwrap().lock().await;
        assert!(!endpoints.contains_key(&ip1));
        assert!(!endpoints.contains_key(&ip2));
    }

    #[tokio::test]
    async fn test_clear_ips_concurrent_access() {
        let state = setup_state().await;
        let ip = IpAddr::from_str("127.0.0.1").unwrap();
        let now = Utc::now();

        // Add an active IP
        state
            .lock()
            .await
            .endpoints
            .entry("/test".to_string())
            .or_default()
            .lock()
            .await
            .insert(
                ip,
                Arc::new(Mutex::new(VecDeque::from(vec![now - Duration::seconds(1)]))),
            );

        // Spawn concurrent tasks
        let state_clone = state.clone();
        let clear_task = tokio::spawn(async move {
            state_clone.lock().await.clear_ips().await;
        });

        let state_clone = state.clone();
        let rate_limit_task = tokio::spawn(async move {
            rate_limit(state_clone, ip, "/test", Utc::now())
                .await
                .unwrap();
        });

        clear_task.await.unwrap();
        rate_limit_task.await.unwrap();

        // Verify the IP still exists after concurrent operations
        let state = state.lock().await;
        let endpoints = state.endpoints.get("/test").unwrap().lock().await;
        assert!(endpoints.contains_key(&ip));
    }
}
