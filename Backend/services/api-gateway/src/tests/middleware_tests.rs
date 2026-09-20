#[cfg(test)]
mod test_mod {
    use crate::middleware::RateLimitState;

    #[test]
    fn test_rate_limit_state_new() {
        let state = RateLimitState::new(100, 60, 20);
        assert_eq!(state.max_requests, 100);
        assert_eq!(state.window_secs, 60);
        assert_eq!(state.remaining("test_key" ), 100);
    }

    #[test]
    fn test_rate_limit_allow_first_request() {
        let state = RateLimitState::new(10, 60, 2);
        assert!(state.check_rate_limit("client1" ));
    }

    #[test]
    fn test_rate_limit_remaining_after_request() {
        let state = RateLimitState::new(10, 60, 2);
        state.check_rate_limit("client1" );
        assert_eq!(state.remaining("client1" ), 9);
    }

    #[test]
    fn test_rate_limit_exceed_limit() {
        let state = RateLimitState::new(2, 60, 0);
        assert!(state.check_rate_limit("client1" ));
        assert!(state.check_rate_limit("client1" ));
        // 第三次请求应该被拒绝
        assert!(!state.check_rate_limit("client1" ));
    }

    #[test]
    fn test_rate_limit_different_clients() {
        let state = RateLimitState::new(1, 60, 0);
        assert!(state.check_rate_limit("client1" ));
        // client1 被限流
        assert!(!state.check_rate_limit("client1" ));
        // client2 应该不受影响
        assert!(state.check_rate_limit("client2" ));
    }

    #[test]
    fn test_rate_limit_remaining_unknown_key() {
        let state = RateLimitState::new(100, 60, 20);
        // 未知 key 应该返回最大请求数
        assert_eq!(state.remaining("unknown_key" ), 100);
    }

    #[test]
    fn test_rate_limit_state_clone() {
        let state1 = RateLimitState::new(100, 60, 20);
        let state2 = state1.clone();

        assert_eq!(state1.max_requests, state2.max_requests);
        assert_eq!(state1.window_secs, state2.window_secs);
        assert_eq!(state1.burst, state2.burst);
    }

    #[test]
    fn test_rate_limit_state_with_burst() {
        let state = RateLimitState::new(100, 60, 50);
        assert_eq!(state.max_requests, 100);
        assert_eq!(state.window_secs, 60);
        assert_eq!(state.burst, 50);
    }

    #[test]
    fn test_rate_limit_state_default_burst() {
        let state = RateLimitState::new(100, 60, 20);
        // 默认 burst 为 20
        assert_eq!(state.burst, 20);
    }
}
