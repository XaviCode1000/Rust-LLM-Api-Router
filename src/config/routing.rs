//! Routing configuration module

use std::env;

/// Routing strategy for request execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RoutingStrategy {
    #[default]
    Auto,
    CostOptimized,
    Cascading,
    Failover,
    LoadBalanced,
}

impl std::fmt::Display for RoutingStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RoutingStrategy::Auto => write!(f, "auto"),
            RoutingStrategy::CostOptimized => write!(f, "cost-optimized"),
            RoutingStrategy::Cascading => write!(f, "cascading"),
            RoutingStrategy::Failover => write!(f, "failover"),
            RoutingStrategy::LoadBalanced => write!(f, "load-balanced"),
        }
    }
}

impl std::str::FromStr for RoutingStrategy {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "auto" => Ok(RoutingStrategy::Auto),
            "cost-optimized" | "cost_optimized" => Ok(RoutingStrategy::CostOptimized),
            "cascading" => Ok(RoutingStrategy::Cascading),
            "failover" => Ok(RoutingStrategy::Failover),
            "load-balanced" | "load_balanced" => Ok(RoutingStrategy::LoadBalanced),
            _ => Err(format!(
                "Invalid routing strategy '{}'. Valid values: auto, cost-optimized, cascading, failover, load-balanced",
                s
            )),
        }
    }
}

/// Configuration for routing strategies
#[derive(Debug, Clone)]
pub struct RoutingConfig {
    /// Selected routing strategy
    pub strategy: RoutingStrategy,
    /// Whether cascading is enabled
    pub cascading_enabled: bool,
    /// Minimum quality score for cascading (0.0-1.0)
    pub cascading_min_quality: f64,
    /// Maximum tiers to try in cascading
    pub cascading_max_tiers: u32,
    /// Timeout per tier in cascading (milliseconds)
    pub cascading_per_tier_timeout_ms: u64,
    /// Whether budget mode is enabled
    pub budget_mode: bool,
    /// Maximum cost per million tokens (if budget mode is enabled)
    pub max_cost_per_million: Option<f64>,
    /// Maximum retries per request
    pub max_retries: u32,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
}

impl Default for RoutingConfig {
    fn default() -> Self {
        Self {
            strategy: RoutingStrategy::Auto,
            cascading_enabled: false,
            cascading_min_quality: 0.75,
            cascading_max_tiers: 3,
            cascading_per_tier_timeout_ms: 5000,
            budget_mode: false,
            max_cost_per_million: None,
            max_retries: 3,
            timeout_seconds: 60,
        }
    }
}

impl RoutingConfig {
    /// Create routing config from CLI arguments and environment variables
    pub fn from_cli_and_env(
        routing_strategy: &str,
        cascading: bool,
        quality_threshold: f64,
        budget_mode: bool,
        max_retries: u32,
        timeout: u64,
    ) -> Result<Self, String> {
        // Validate quality threshold
        if !(0.0..=1.0).contains(&quality_threshold) {
            return Err("Quality threshold must be between 0.0 and 1.0".to_string());
        }

        // Parse routing strategy
        let strategy = routing_strategy.parse::<RoutingStrategy>()?;

        // Cascading enabled: CLI flag or env var
        let cascading_enabled = cascading
            || env::var("CASCADING_ENABLED")
                .map(|v| v == "true")
                .unwrap_or(false);

        // Quality threshold: CLI flag or env var (default 0.75)
        let cascading_min_quality = if quality_threshold != 0.75 {
            quality_threshold
        } else {
            env::var("CASCADING_MIN_QUALITY")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.75)
        };

        // Max tiers: env var (default 3)
        let cascading_max_tiers = env::var("CASCADING_MAX_TIERS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(3);

        // Per tier timeout: env var (default 5000ms)
        let cascading_per_tier_timeout_ms = env::var("CASCADING_PER_TIER_TIMEOUT_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(5000);

        // Budget mode: CLI flag or env var
        let budget_mode = budget_mode
            || env::var("BUDGET_MODE")
                .map(|v| v == "true")
                .unwrap_or(false);

        // Max retries: CLI flag or env var (default 3)
        let max_retries = if max_retries != 3 {
            max_retries
        } else {
            env::var("MAX_RETRIES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3)
        };

        // Timeout: CLI flag or env var (default 60s)
        let timeout_seconds = if timeout != 60 {
            timeout
        } else {
            env::var("REQUEST_TIMEOUT_SECONDS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(60)
        };

        Ok(Self {
            strategy,
            cascading_enabled,
            cascading_min_quality,
            cascading_max_tiers,
            cascading_per_tier_timeout_ms,
            budget_mode,
            max_cost_per_million: None,
            max_retries,
            timeout_seconds,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_routing_strategy_display() {
        assert_eq!(format!("{}", RoutingStrategy::Auto), "auto");
        assert_eq!(format!("{}", RoutingStrategy::CostOptimized), "cost-optimized");
        assert_eq!(format!("{}", RoutingStrategy::Cascading), "cascading");
        assert_eq!(format!("{}", RoutingStrategy::Failover), "failover");
        assert_eq!(format!("{}", RoutingStrategy::LoadBalanced), "load-balanced");
    }

    #[test]
    fn test_routing_strategy_from_str() {
        assert_eq!("auto".parse::<RoutingStrategy>().unwrap(), RoutingStrategy::Auto);
        assert_eq!(
            "cost-optimized".parse::<RoutingStrategy>().unwrap(),
            RoutingStrategy::CostOptimized
        );
        assert_eq!(
            "cost_optimized".parse::<RoutingStrategy>().unwrap(),
            RoutingStrategy::CostOptimized
        );
        assert_eq!("cascading".parse::<RoutingStrategy>().unwrap(), RoutingStrategy::Cascading);
        assert_eq!("failover".parse::<RoutingStrategy>().unwrap(), RoutingStrategy::Failover);
        assert_eq!(
            "load-balanced".parse::<RoutingStrategy>().unwrap(),
            RoutingStrategy::LoadBalanced
        );
        assert_eq!(
            "load_balanced".parse::<RoutingStrategy>().unwrap(),
            RoutingStrategy::LoadBalanced
        );

        assert!("invalid".parse::<RoutingStrategy>().is_err());
    }

    #[test]
    fn test_routing_config_default_values() {
        let config = RoutingConfig::from_cli_and_env("auto", false, 0.75, false, 3, 60).unwrap();

        assert_eq!(config.strategy, RoutingStrategy::Auto);
        assert!(!config.cascading_enabled);
        assert_eq!(config.cascading_min_quality, 0.75);
        assert_eq!(config.cascading_max_tiers, 3);
        assert_eq!(config.cascading_per_tier_timeout_ms, 5000);
        assert!(!config.budget_mode);
        assert!(config.max_cost_per_million.is_none());
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.timeout_seconds, 60);
    }

    #[test]
    fn test_routing_config_cascading_enabled() {
        let config =
            RoutingConfig::from_cli_and_env("cascading", true, 0.85, false, 5, 30).unwrap();

        assert_eq!(config.strategy, RoutingStrategy::Cascading);
        assert!(config.cascading_enabled);
        assert_eq!(config.cascading_min_quality, 0.85);
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.timeout_seconds, 30);
    }

    #[test]
    fn test_routing_config_invalid_quality_threshold() {
        let result = RoutingConfig::from_cli_and_env("auto", false, 1.5, false, 3, 60);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Quality threshold must be between 0.0 and 1.0"));

        let result = RoutingConfig::from_cli_and_env("auto", false, -0.1, false, 3, 60);
        assert!(result.is_err());
    }
}
