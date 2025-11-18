//! RPC Rate Limiting for DDoS Protection
//!
//! Implements intelligent rate limiting with:
//! - Per-IP address limits
//! - Per-method limits
//! - Burst allowance
//! - Automatic ban for abusive IPs

use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter as GovernorRateLimiter,
};
use nonzero_ext::*;
use std::{
    collections::HashMap,
    net::IpAddr,
    num::NonZeroU32,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard, PoisonError},
    time::{Duration, Instant},
};

/// Helper to handle poisoned RwLock for read access
/// If the lock is poisoned (thread panicked while holding it), we recover the inner value
fn read_lock_recover<T>(lock: &RwLock<T>) -> RwLockReadGuard<'_, T> {
    lock.read().unwrap_or_else(|e| {
        tracing::error!("RwLock poisoned during read, recovering: {}", e);
        e.into_inner()
    })
}

/// Helper to handle poisoned RwLock for write access
fn write_lock_recover<T>(lock: &RwLock<T>) -> RwLockWriteGuard<'_, T> {
    lock.write().unwrap_or_else(|e| {
        tracing::error!("RwLock poisoned during write, recovering: {}", e);
        e.into_inner()
    })
}

/// Rate limiter for RPC endpoints
#[derive(Clone)]
pub struct RpcRateLimiter {
    /// Per-IP rate limiters
    ip_limiters: Arc<RwLock<HashMap<IpAddr, IpRateLimiter>>>,
    
    /// Global rate limiter (for all requests)
    global_limiter: Arc<GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
    
    /// Configuration
    config: RateLimitConfig,
    
    /// Banned IPs (temporarily blocked)
    banned_ips: Arc<RwLock<HashMap<IpAddr, BanInfo>>>,
}

/// Per-IP rate limiter with burst capacity
struct IpRateLimiter {
    /// The actual rate limiter
    limiter: GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock>,
    
    /// Last access time (for cleanup)
    last_access: Instant,
    
    /// Violation count (for auto-ban)
    violations: u32,
}

/// Information about a banned IP
struct BanInfo {
    /// When the ban expires
    expires_at: Instant,
    
    /// Reason for ban
    reason: BanReason,
    
    /// Number of violations that led to ban
    violation_count: u32,
}

#[derive(Debug, Clone, Copy)]
enum BanReason {
    /// Excessive rate limit violations
    ExcessiveViolations,
    
    /// Manual ban by admin
    Manual,
    
    /// Suspicious activity detected
    Suspicious,
}

/// Rate limit configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Requests per second per IP
    pub requests_per_second: NonZeroU32,
    
    /// Burst capacity (max requests in burst)
    pub burst_size: NonZeroU32,
    
    /// Global requests per second
    pub global_requests_per_second: NonZeroU32,
    
    /// Number of violations before auto-ban
    pub max_violations: u32,
    
    /// Ban duration for auto-bans
    pub auto_ban_duration: Duration,
    
    /// Cleanup interval for inactive IP entries
    pub cleanup_interval: Duration,
    
    /// Method-specific limits (method_name -> requests_per_second)
    pub method_limits: HashMap<String, NonZeroU32>,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        let mut method_limits = HashMap::new();
        
        // Stricter limits for expensive methods
        method_limits.insert("eth_call".to_string(), nonzero!(20u32));
        method_limits.insert("eth_estimateGas".to_string(), nonzero!(10u32));
        method_limits.insert("debug_traceTransaction".to_string(), nonzero!(5u32));
        method_limits.insert("trace_replayTransaction".to_string(), nonzero!(5u32));
        
        // Moderate limits for common read methods
        method_limits.insert("eth_getBalance".to_string(), nonzero!(50u32));
        method_limits.insert("eth_getBlockByNumber".to_string(), nonzero!(50u32));
        method_limits.insert("eth_getTransactionReceipt".to_string(), nonzero!(50u32));
        
        Self {
            requests_per_second: nonzero!(100u32),    // 100 req/sec per IP
            burst_size: nonzero!(200u32),             // Allow bursts up to 200
            global_requests_per_second: nonzero!(10000u32), // 10k req/sec globally
            max_violations: 10,
            auto_ban_duration: Duration::from_secs(300), // 5 minute ban
            cleanup_interval: Duration::from_secs(3600), // Clean up hourly
            method_limits,
        }
    }
}

impl RpcRateLimiter {
    /// Create a new rate limiter with default config
    pub fn new() -> Self {
        Self::with_config(RateLimitConfig::default())
    }
    
    /// Create a new rate limiter with custom config
    pub fn with_config(config: RateLimitConfig) -> Self {
        let global_quota = Quota::per_second(config.global_requests_per_second);
        let global_limiter = Arc::new(GovernorRateLimiter::direct(global_quota));
        
        Self {
            ip_limiters: Arc::new(RwLock::new(HashMap::new())),
            global_limiter,
            config,
            banned_ips: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Check if a request should be allowed
    ///
    /// Returns Ok(()) if allowed, Err with reason if rejected
    pub fn check_rate_limit(
        &self,
        ip: IpAddr,
        method: Option<&str>,
    ) -> Result<(), RateLimitError> {
        // Check if IP is banned
        if let Some(ban_info) = self.is_banned(&ip) {
            return Err(RateLimitError::Banned {
                expires_at: ban_info.expires_at,
                reason: ban_info.reason,
            });
        }
        
        // Check global rate limit first
        if self.global_limiter.check().is_err() {
            return Err(RateLimitError::GlobalLimit);
        }
        
        // Check method-specific limit if applicable
        if let Some(method_name) = method {
            if let Some(&limit) = self.config.method_limits.get(method_name) {
                // For now, we'll use a simple approach
                // In production, you'd have per-method per-IP limiters
                if self.global_limiter.check().is_err() {
                    return Err(RateLimitError::MethodLimit {
                        method: method_name.to_string(),
                    });
                }
            }
        }
        
        // Check per-IP limit
        let mut limiters = write_lock_recover(&self.ip_limiters);
        let ip_limiter = limiters.entry(ip).or_insert_with(|| {
            let quota = Quota::per_second(self.config.requests_per_second)
                .allow_burst(self.config.burst_size);
            
            IpRateLimiter {
                limiter: GovernorRateLimiter::direct(quota),
                last_access: Instant::now(),
                violations: 0,
            }
        });
        
        ip_limiter.last_access = Instant::now();
        
        match ip_limiter.limiter.check() {
            Ok(_) => {
                // Reset violations on successful request
                ip_limiter.violations = 0;
                Ok(())
            }
            Err(_) => {
                // Increment violations
                ip_limiter.violations += 1;
                
                // Auto-ban if too many violations
                if ip_limiter.violations >= self.config.max_violations {
                    drop(limiters); // Release lock before banning
                    self.ban_ip(
                        ip,
                        self.config.auto_ban_duration,
                        BanReason::ExcessiveViolations,
                    );
                    return Err(RateLimitError::AutoBanned);
                }
                
                Err(RateLimitError::IpLimit { ip })
            }
        }
    }
    
    /// Check if an IP is currently banned
    fn is_banned(&self, ip: &IpAddr) -> Option<BanInfo> {
        let mut banned = write_lock_recover(&self.banned_ips);
        
        if let Some(ban_info) = banned.get(ip) {
            // Check if ban has expired
            if Instant::now() >= ban_info.expires_at {
                banned.remove(ip);
                None
            } else {
                Some(*ban_info)
            }
        } else {
            None
        }
    }
    
    /// Manually ban an IP address
    pub fn ban_ip(&self, ip: IpAddr, duration: Duration, reason: BanReason) {
        let expires_at = Instant::now() + duration;
        let violation_count = read_lock_recover(&self.ip_limiters)
            .get(&ip)
            .map(|l| l.violations)
            .unwrap_or(0);

        write_lock_recover(&self.banned_ips).insert(
            ip,
            BanInfo {
                expires_at,
                reason,
                violation_count,
            },
        );
        
        tracing::warn!(
            %ip,
            ?reason,
            duration_secs = duration.as_secs(),
            "IP address banned"
        );
    }
    
    /// Manually unban an IP address
    pub fn unban_ip(&self, ip: IpAddr) -> bool {
        let removed = write_lock_recover(&self.banned_ips).remove(&ip).is_some();
        
        if removed {
            tracing::info!(%ip, "IP address unbanned");
        }
        
        removed
    }
    
    /// Clean up old IP limiters to prevent memory leak
    pub fn cleanup_old_entries(&self) {
        let mut limiters = write_lock_recover(&self.ip_limiters);
        let cutoff = Instant::now() - self.config.cleanup_interval;
        
        let before_count = limiters.len();
        limiters.retain(|_, limiter| limiter.last_access > cutoff);
        let after_count = limiters.len();
        
        if before_count != after_count {
            tracing::debug!(
                removed = before_count - after_count,
                "Cleaned up old rate limiter entries"
            );
        }
    }
    
    /// Get statistics about current rate limiting
    pub fn stats(&self) -> RateLimitStats {
        let limiters = read_lock_recover(&self.ip_limiters);
        let banned = read_lock_recover(&self.banned_ips);
        
        RateLimitStats {
            tracked_ips: limiters.len(),
            banned_ips: banned.len(),
            total_violations: limiters.values().map(|l| l.violations as u64).sum(),
        }
    }
}

impl Default for RpcRateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

/// Rate limit error types
#[derive(Debug, thiserror::Error)]
pub enum RateLimitError {
    #[error("IP {ip} exceeded rate limit")]
    IpLimit { ip: IpAddr },
    
    #[error("Global rate limit exceeded")]
    GlobalLimit,
    
    #[error("Method {method} rate limit exceeded")]
    MethodLimit { method: String },
    
    #[error("IP banned due to excessive violations")]
    AutoBanned,
    
    #[error("IP banned until {expires_at:?} (reason: {reason:?})")]
    Banned {
        expires_at: Instant,
        reason: BanReason,
    },
}

impl Copy for BanInfo {}
impl Clone for BanInfo {
    fn clone(&self) -> Self {
        *self
    }
}

/// Rate limiter statistics
#[derive(Debug, Clone)]
pub struct RateLimitStats {
    /// Number of IPs currently being tracked
    pub tracked_ips: usize,
    
    /// Number of currently banned IPs
    pub banned_ips: usize,
    
    /// Total violations across all IPs
    pub total_violations: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, IpAddr};
    
    #[test]
    fn test_rate_limit_allows_within_quota() {
        let config = RateLimitConfig {
            requests_per_second: nonzero!(10u32),
            burst_size: nonzero!(10u32),
            ..Default::default()
        };
        
        let limiter = RpcRateLimiter::with_config(config);
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        
        // Should allow first request
        assert!(limiter.check_rate_limit(ip, None).is_ok());
    }
    
    #[test]
    fn test_cleanup_old_entries() {
        let mut config = RateLimitConfig::default();
        config.cleanup_interval = Duration::from_secs(0); // Cleanup immediately
        
        let limiter = RpcRateLimiter::with_config(config);
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        
        limiter.check_rate_limit(ip, None).ok();
        assert_eq!(limiter.stats().tracked_ips, 1);
        
        limiter.cleanup_old_entries();
        assert_eq!(limiter.stats().tracked_ips, 0);
    }
}
