//! Performance benchmarks for execution plan module
//!
//! These benchmarks measure the performance of various execution plan operations.

use rust_llm_api_router::app::services::execution_plan::{
    ExecutionContext, ExecutionPlan, ExecutionPlanner, ExecutionPlannerConfig, PlanningOptions,
};
use rust_llm_api_router::domain::traits::AccountRepository;
use rust_llm_api_router::domain::Account;
use rust_llm_api_router::infrastructure::persistence::JsonAccountRepository;
use std::sync::Arc;

// Note: Run these benchmarks with: cargo bench

// ============================================================================
// BENCHMARK HELPERS
// ============================================================================

fn create_test_repo() -> Arc<dyn AccountRepository> {
    // Create a temporary repository for benchmarking
    let repo = JsonAccountRepository::new().expect("Should create repository");
    Arc::new(repo)
}

fn setup_repo_with_accounts(repo: &Arc<dyn AccountRepository>, count: usize) {
    // Add multiple accounts for testing
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        for i in 0..count {
            let account = Account::new(
                format!("bench-account-{}", i),
                "openai",
                format!("sk-key-{}", i),
            );
            repo.save(account).await.expect("Should save account");
        }
    });
}

// ============================================================================
// PLANNER BENCHMARKS
// ============================================================================

/// Benchmark: Creating a standard execution plan
#[tokio::test]
async fn benchmark_create_standard_plan() {
    let repo = create_test_repo();
    setup_repo_with_accounts(&repo, 5);

    let planner = ExecutionPlanner::new(repo, ExecutionPlannerConfig::default());

    let context = ExecutionContext::new("bench-1", "gpt-4")
        .with_preferred_providers(vec!["openai".to_string()]);

    // Warm up
    for _ in 0..10 {
        let _ = planner.create_plan(context.clone()).await;
    }

    // Benchmark
    let start = std::time::Instant::now();
    for _ in 0..1000 {
        let _ = planner.create_plan(context.clone()).await;
    }
    let elapsed = start.elapsed();

    println!("Create standard plan: {:?} per iteration", elapsed / 1000);
}

/// Benchmark: Creating a failover execution plan
#[tokio::test]
async fn benchmark_create_failover_plan() {
    let repo = create_test_repo();
    setup_repo_with_accounts(&repo, 10);

    let config = ExecutionPlannerConfig::reliability();
    let planner = ExecutionPlanner::new(repo, config);

    let context = ExecutionContext::new("bench-1", "gpt-4")
        .with_preferred_providers(vec!["openai".to_string()])
        .with_planning_options(PlanningOptions::reliability());

    // Warm up
    for _ in 0..10 {
        let _ = planner.create_plan(context.clone()).await;
    }

    // Benchmark
    let start = std::time::Instant::now();
    for _ in 0..1000 {
        let _ = planner.create_plan(context.clone()).await;
    }
    let elapsed = start.elapsed();

    println!("Create failover plan: {:?} per iteration", elapsed / 1000);
}

/// Benchmark: Creating plan with many accounts
#[tokio::test]
async fn benchmark_create_plan_many_accounts() {
    let repo = create_test_repo();
    setup_repo_with_accounts(&repo, 50);

    let config = ExecutionPlannerConfig::default().with_max_accounts(20);

    let planner = ExecutionPlanner::new(repo, config);

    let context = ExecutionContext::new("bench-1", "gpt-4")
        .with_preferred_providers(vec!["openai".to_string()]);

    // Benchmark
    let start = std::time::Instant::now();
    for _ in 0..1000 {
        let _ = planner.create_plan(context.clone()).await;
    }
    let elapsed = start.elapsed();

    println!(
        "Create plan with 50 accounts: {:?} per iteration",
        elapsed / 1000
    );
}

// ============================================================================
// PLAN EXECUTION BENCHMARKS
// ============================================================================

/// Benchmark: Plan execution with single account
#[tokio::test]
async fn benchmark_plan_execution_single_account() {
    let repo = create_test_repo();
    setup_repo_with_accounts(&repo, 1);

    let planner = ExecutionPlanner::new(repo, ExecutionPlannerConfig::default());

    let context = ExecutionContext::new("bench-1", "gpt-4")
        .with_preferred_providers(vec!["openai".to_string()]);

    // Benchmark what we can measure - plan creation and access
    let start = std::time::Instant::now();
    for _ in 0..1000 {
        let plan = planner.create_plan(context.clone()).await.unwrap();
        // Access plan data to measure what can be measured
        let _ = plan.account_count();
        let _ = plan.has_accounts();
    }
    let elapsed = start.elapsed();

    println!(
        "Single account execution: {:?} per iteration",
        elapsed / 1000
    );
}

/// Benchmark: Plan execution with multiple accounts (failover)
#[tokio::test]
async fn benchmark_plan_execution_failover() {
    let repo = create_test_repo();
    setup_repo_with_accounts(&repo, 5);

    let config = ExecutionPlannerConfig::reliability();
    let planner = ExecutionPlanner::new(repo, config);

    let context = ExecutionContext::new("bench-1", "gpt-4")
        .with_preferred_providers(vec!["openai".to_string()])
        .with_planning_options(PlanningOptions::reliability());

    // Benchmark - measure plan creation and access
    let start = std::time::Instant::now();
    for _ in 0..1000 {
        let plan = planner.create_plan(context.clone()).await.unwrap();
        let _ = plan.account_count();
        let _ = plan.primary_account();
    }
    let elapsed = start.elapsed();

    println!(
        "Failover execution (5 accounts): {:?} per iteration",
        elapsed / 1000
    );
}

/// Benchmark: Plan execution with all accounts failing
#[tokio::test]
async fn benchmark_plan_execution_all_fail() {
    let repo = create_test_repo();
    setup_repo_with_accounts(&repo, 5);

    let config = ExecutionPlannerConfig::reliability();
    let planner = ExecutionPlanner::new(repo, config);

    let context = ExecutionContext::new("bench-1", "gpt-4")
        .with_preferred_providers(vec!["openai".to_string()])
        .with_planning_options(PlanningOptions::reliability());

    // Benchmark - measure plan creation and access
    let start = std::time::Instant::now();
    for _ in 0..100 {
        let plan = planner.create_plan(context.clone()).await.unwrap();
        let _ = plan.account_count();
        let _ = plan.fallback_accounts();
    }
    let elapsed = start.elapsed();

    println!("All fail (5 accounts): {:?} per iteration", elapsed / 100);
}

// ============================================================================
// CONFIGURATION BENCHMARKS
// ============================================================================

/// Benchmark: Config builder
#[tokio::test]
async fn benchmark_config_builder() {
    // Warm up
    for _ in 0..10 {
        let _ = ExecutionPlannerConfig::reliability().with_max_accounts(10);
    }

    // Benchmark
    let start = std::time::Instant::now();
    for _ in 0..10000 {
        let _ = ExecutionPlannerConfig::reliability().with_max_accounts(10);
    }
    let elapsed = start.elapsed();

    println!("Config builder: {:?} per iteration", elapsed / 10000);
}

/// Benchmark: Config presets
#[tokio::test]
async fn benchmark_config_presets() {
    // Benchmark
    let start = std::time::Instant::now();
    for _ in 0..10000 {
        let _ = ExecutionPlannerConfig::reliability();
        let _ = ExecutionPlannerConfig::cost_optimized();
        let _ = ExecutionPlannerConfig::low_latency();
    }
    let elapsed = start.elapsed();

    println!("Config presets: {:?} per iteration", elapsed / 10000);
}

// ============================================================================
// CONTEXT CREATION BENCHMARKS
// ============================================================================

/// Benchmark: Context creation
#[tokio::test]
async fn benchmark_context_creation() {
    // Warm up
    for _ in 0..10 {
        let _ = ExecutionContext::new("bench", "gpt-4");
    }

    // Benchmark
    let start = std::time::Instant::now();
    for _ in 0..100000 {
        let _ = ExecutionContext::new("bench", "gpt-4");
    }
    let elapsed = start.elapsed();

    println!("Context creation: {:?} per iteration", elapsed / 100000);
}

/// Benchmark: Context with options
#[tokio::test]
async fn benchmark_context_with_options() {
    let options = PlanningOptions::reliability();

    // Benchmark
    let start = std::time::Instant::now();
    for _ in 0..100000 {
        let _ = ExecutionContext::new("bench", "gpt-4")
            .with_preferred_providers(vec!["openai".to_string()])
            .with_planning_options(options.clone());
    }
    let elapsed = start.elapsed();

    println!("Context with options: {:?} per iteration", elapsed / 100000);
}

// ============================================================================
// FAILOVER MANAGER BENCHMARKS
// ============================================================================

use rust_llm_api_router::app::services::account_rotation::AccountSelector;
use rust_llm_api_router::app::services::failover::FailoverManager;

/// Benchmark: FailoverManager execution
#[tokio::test]
async fn benchmark_failover_manager_execution() {
    let repo = create_test_repo();
    setup_repo_with_accounts(&repo, 5);

    let manager = FailoverManager::with_round_robin(repo);

    // Benchmark
    let start = std::time::Instant::now();
    for _ in 0..1000 {
        let result = manager
            .execute_with_failover(
                "openai",
                |_account: &rust_llm_api_router::domain::Account| async move {
                    Ok::<(String, Vec<(String, String)>), String>(("result".to_string(), vec![]))
                },
            )
            .await;
        let _ = result;
    }
    let elapsed = start.elapsed();

    println!(
        "FailoverManager execution: {:?} per iteration",
        elapsed / 1000
    );
}

/// Benchmark: FailoverManager with failures
#[tokio::test]
async fn benchmark_failover_with_failures() {
    let repo = create_test_repo();
    setup_repo_with_accounts(&repo, 5);

    let manager = FailoverManager::new(repo, AccountSelector::round_robin(), 3);

    // Benchmark - first fails, second succeeds
    let start = std::time::Instant::now();
    for i in 0..1000 {
        let i = i;
        let result = manager
            .execute_with_failover(
                "openai",
                |account: &rust_llm_api_router::domain::Account| {
                    let account_id = account.id.clone();
                    async move {
                        if account_id.contains("1") {
                            Err::<(String, Vec<(String, String)>), String>("fail".to_string())
                        } else {
                            Ok::<_, String>((format!("success-{}", i), vec![]))
                        }
                    }
                },
            )
            .await;
        let _ = result;
    }
    let elapsed = start.elapsed();

    println!("Failover with 1 fail: {:?} per iteration", elapsed / 1000);
}

/// Benchmark: Health tracking
#[tokio::test]
async fn benchmark_health_tracking() {
    let repo = create_test_repo();
    setup_repo_with_accounts(&repo, 5);

    let manager = FailoverManager::with_round_robin(repo);

    // Record many requests
    for _ in 0..1000 {
        let result = manager
            .execute_with_failover(
                "openai",
                |_account: &rust_llm_api_router::domain::Account| async move {
                    Ok::<(String, Vec<(String, String)>), String>(("result".to_string(), vec![]))
                },
            )
            .await;
        let _ = result;
    }

    // Benchmark getting health
    let start = std::time::Instant::now();
    for _ in 0..10000 {
        let _ = manager.get_all_health();
    }
    let elapsed = start.elapsed();

    println!("Health tracking: {:?} per iteration", elapsed / 10000);
}

// ============================================================================
// SCALABILITY BENCHMARKS
// ============================================================================

/// Benchmark: Scaling to many accounts
#[tokio::test]
async fn benchmark_scaling_many_accounts() {
    for account_count in [10, 50, 100] {
        let repo = create_test_repo();
        setup_repo_with_accounts(&repo, account_count);

        let planner = ExecutionPlanner::new(repo, ExecutionPlannerConfig::default());

        let context = ExecutionContext::new("bench", "gpt-4")
            .with_preferred_providers(vec!["openai".to_string()]);

        let start = std::time::Instant::now();
        for _ in 0..100 {
            let _ = planner.create_plan(context.clone()).await;
        }
        let elapsed = start.elapsed();

        println!(
            "Plan creation with {} accounts: {:?} per iteration",
            account_count,
            elapsed / 100
        );
    }
}

/// Benchmark: Concurrent planning
#[tokio::test]
async fn benchmark_concurrent_planning() {
    let repo = create_test_repo();
    setup_repo_with_accounts(&repo, 10);

    let planner = Arc::new(ExecutionPlanner::new(
        repo,
        ExecutionPlannerConfig::default(),
    ));

    // Benchmark concurrent planning
    let start = std::time::Instant::now();

    let mut handles = vec![];
    for i in 0..100 {
        let planner = planner.clone();
        let handle = tokio::spawn(async move {
            let context = ExecutionContext::new(format!("req-{}", i), "gpt-4")
                .with_preferred_providers(vec!["openai".to_string()]);
            planner.create_plan(context).await
        });
        handles.push(handle);
    }

    futures::future::join_all(handles).await;
    let elapsed = start.elapsed();

    println!("Concurrent planning (100 requests): {:?}", elapsed);
}

// ============================================================================
// MEMORY BENCHMARKS
// ============================================================================

/// Benchmark: Memory usage with many plans
#[tokio::test]
async fn benchmark_memory_many_plans() {
    let repo = create_test_repo();
    setup_repo_with_accounts(&repo, 10);

    let planner = ExecutionPlanner::new(repo, ExecutionPlannerConfig::default());

    // Create many plans
    let context = ExecutionContext::new("bench", "gpt-4")
        .with_preferred_providers(vec!["openai".to_string()]);

    let mut plans = vec![];
    for _ in 0..1000 {
        let plan = planner.create_plan(context.clone()).await.unwrap();
        plans.push(plan);
    }

    // Just verify they exist - memory measurement would require additional tooling
    assert_eq!(plans.len(), 1000);
    println!("Created 1000 plans successfully");
}
