//! Deterministic bakery production planning.
//!
//! This crate is the first vertical slice of OptiAI. It takes a reviewed,
//! typed business model and sends it to HiGHS through `good_lp`. No language
//! model is involved in the calculation.

use good_lp::{Expression, Solution, SolverModel, constraint, default_solver, variable, variables};
use opti_domain::{BakeryPlan, Product, Resource, ValidationError};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductionDecision {
    pub product_id: String,
    pub product_name: String,
    pub units: u64,
    pub profit_cents: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceUsage {
    pub resource_id: String,
    pub resource_name: String,
    pub unit: String,
    pub available_quantity: u64,
    pub used_quantity: u64,
    pub remaining_quantity: u64,
    pub is_bottleneck: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolveResult {
    pub bakery_name: String,
    pub objective_profit_cents: i64,
    pub decisions: Vec<ProductionDecision>,
    pub resources: Vec<ResourceUsage>,
}

#[derive(Debug, Error)]
pub enum SolveError {
    #[error("invalid bakery plan: {0}")]
    Validation(#[from] ValidationError),
    #[error("HiGHS could not solve the model: {0}")]
    Solver(String),
    #[error("solver returned a non-integral production quantity for {product_id}: {value}")]
    NonIntegralResult { product_id: String, value: f64 },
}

/// Solve a bakery plan as a mixed-integer linear program.
pub fn solve_plan(plan: &BakeryPlan) -> Result<SolveResult, SolveError> {
    plan.validate()?;

    let mut model = variables!();
    let decision_variables = plan
        .products
        .iter()
        .map(|product| {
            model.add(
                variable()
                    .integer()
                    .min(0.0)
                    .max(product.demand_limit as f64),
            )
        })
        .collect::<Vec<_>>();

    let objective = plan.products.iter().enumerate().fold(
        Expression::from(0.0),
        |expression, (index, product)| {
            expression + (product.profit_cents() as f64) * decision_variables[index]
        },
    );

    let mut problem = model.maximise(objective).using(default_solver);
    for resource in &plan.resources {
        let resource_expression = plan.products.iter().enumerate().fold(
            Expression::from(0.0),
            |expression, (index, product)| {
                let quantity = product
                    .recipe
                    .iter()
                    .find(|line| line.resource_id == resource.id)
                    .map_or(0.0, |line| line.quantity_per_unit as f64);
                expression + quantity * decision_variables[index]
            },
        );
        problem = problem.with(constraint!(
            resource_expression <= resource.available_quantity as f64
        ));
    }

    let solution = problem
        .solve()
        .map_err(|error| SolveError::Solver(error.to_string()))?;

    let mut decisions = Vec::with_capacity(plan.products.len());
    for (index, product) in plan.products.iter().enumerate() {
        let raw_value = solution.value(decision_variables[index]);
        let rounded_value = raw_value.round();
        if (raw_value - rounded_value).abs() > 1e-6 {
            return Err(SolveError::NonIntegralResult {
                product_id: product.id.clone(),
                value: raw_value,
            });
        }
        let units = rounded_value as u64;
        decisions.push(ProductionDecision {
            product_id: product.id.clone(),
            product_name: product.name.clone(),
            units,
            profit_cents: product.profit_cents() * units as i64,
        });
    }

    let objective_profit_cents = decisions.iter().map(|decision| decision.profit_cents).sum();
    let resources = resource_usage(plan, &decisions);

    Ok(SolveResult {
        bakery_name: plan.bakery_name.clone(),
        objective_profit_cents,
        decisions,
        resources,
    })
}

fn resource_usage(plan: &BakeryPlan, decisions: &[ProductionDecision]) -> Vec<ResourceUsage> {
    let quantities_by_product = decisions
        .iter()
        .map(|decision| (decision.product_id.as_str(), decision.units))
        .collect::<BTreeMap<_, _>>();

    plan.resources
        .iter()
        .map(|resource| {
            let used_quantity = plan
                .products
                .iter()
                .map(|product| {
                    let units = quantities_by_product
                        .get(product.id.as_str())
                        .copied()
                        .unwrap_or_default();
                    let quantity_per_unit = product
                        .recipe
                        .iter()
                        .find(|line| line.resource_id == resource.id)
                        .map_or(0, |line| line.quantity_per_unit);
                    units.saturating_mul(quantity_per_unit)
                })
                .sum();
            let remaining_quantity = resource.available_quantity.saturating_sub(used_quantity);
            ResourceUsage {
                resource_id: resource.id.clone(),
                resource_name: resource.name.clone(),
                unit: resource.unit.clone(),
                available_quantity: resource.available_quantity,
                used_quantity,
                remaining_quantity,
                is_bottleneck: remaining_quantity == 0,
            }
        })
        .collect()
}

/// Produce a deterministic, non-AI explanation from solver facts.
pub fn explain_result(result: &SolveResult) -> String {
    let production = result
        .decisions
        .iter()
        .filter(|decision| decision.units > 0)
        .map(|decision| format!("{} {}", decision.units, decision.product_name))
        .collect::<Vec<_>>();
    let bottlenecks = result
        .resources
        .iter()
        .filter(|resource| resource.is_bottleneck)
        .map(|resource| resource.resource_name.as_str())
        .collect::<Vec<_>>();

    let production_summary = if production.is_empty() {
        "no products".to_owned()
    } else {
        production.join(", ")
    };
    let bottleneck_summary = if bottlenecks.is_empty() {
        "No resource is fully consumed under this plan.".to_owned()
    } else {
        format!("The limiting resource(s) are {}.", bottlenecks.join(", "))
    };

    format!(
        "For {}, produce {}. Estimated contribution profit is {}. {}",
        result.bakery_name,
        production_summary,
        format_cents(result.objective_profit_cents),
        bottleneck_summary
    )
}

pub fn format_cents(cents: i64) -> String {
    let sign = if cents < 0 { "-" } else { "" };
    let absolute = cents.unsigned_abs();
    format!("{sign}${}.{:02}", absolute / 100, absolute % 100)
}

/// Small, deterministic fixture used by the CLI and integration tests.
pub fn demo_plan() -> BakeryPlan {
    BakeryPlan {
        bakery_name: "Sunrise Bakery".into(),
        horizon_days: 1,
        products: vec![
            Product {
                id: "croissant".into(),
                name: "croissants".into(),
                unit: "piece".into(),
                price_cents: 350,
                unit_cost_cents: 140,
                demand_limit: 100,
                recipe: vec![
                    opti_domain::RecipeLine {
                        resource_id: "flour".into(),
                        quantity_per_unit: 80,
                        unit: "g".into(),
                    },
                    opti_domain::RecipeLine {
                        resource_id: "sugar".into(),
                        quantity_per_unit: 15,
                        unit: "g".into(),
                    },
                    opti_domain::RecipeLine {
                        resource_id: "oven_minutes".into(),
                        quantity_per_unit: 2,
                        unit: "minute".into(),
                    },
                ],
            },
            Product {
                id: "bread".into(),
                name: "loaves of bread".into(),
                unit: "loaf".into(),
                price_cents: 280,
                unit_cost_cents: 90,
                demand_limit: 80,
                recipe: vec![
                    opti_domain::RecipeLine {
                        resource_id: "flour".into(),
                        quantity_per_unit: 300,
                        unit: "g".into(),
                    },
                    opti_domain::RecipeLine {
                        resource_id: "sugar".into(),
                        quantity_per_unit: 5,
                        unit: "g".into(),
                    },
                    opti_domain::RecipeLine {
                        resource_id: "oven_minutes".into(),
                        quantity_per_unit: 4,
                        unit: "minute".into(),
                    },
                ],
            },
            Product {
                id: "cake".into(),
                name: "cakes".into(),
                unit: "cake".into(),
                price_cents: 900,
                unit_cost_cents: 400,
                demand_limit: 40,
                recipe: vec![
                    opti_domain::RecipeLine {
                        resource_id: "flour".into(),
                        quantity_per_unit: 500,
                        unit: "g".into(),
                    },
                    opti_domain::RecipeLine {
                        resource_id: "sugar".into(),
                        quantity_per_unit: 200,
                        unit: "g".into(),
                    },
                    opti_domain::RecipeLine {
                        resource_id: "oven_minutes".into(),
                        quantity_per_unit: 15,
                        unit: "minute".into(),
                    },
                ],
            },
        ],
        resources: vec![
            Resource {
                id: "flour".into(),
                name: "flour".into(),
                unit: "g".into(),
                available_quantity: 12_000,
            },
            Resource {
                id: "sugar".into(),
                name: "sugar".into(),
                unit: "g".into(),
                available_quantity: 5_000,
            },
            Resource {
                id: "oven_minutes".into(),
                name: "oven time".into(),
                unit: "minute".into(),
                available_quantity: 900,
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_plan_has_a_repeatable_optimal_result() {
        let result = solve_plan(&demo_plan()).expect("demo plan should solve");

        assert_eq!(result.objective_profit_cents, 25_000);
        assert_eq!(
            result
                .decisions
                .iter()
                .map(|decision| (decision.product_id.as_str(), decision.units))
                .collect::<Vec<_>>(),
            vec![("croissant", 100), ("bread", 0), ("cake", 8)]
        );
        assert!(
            result
                .resources
                .iter()
                .any(|resource| resource.resource_id == "flour" && resource.is_bottleneck)
        );
    }

    #[test]
    fn explanation_uses_solver_facts() {
        let result = solve_plan(&demo_plan()).expect("demo plan should solve");
        let explanation = explain_result(&result);

        assert!(explanation.contains("100 croissants"));
        assert!(explanation.contains("8 cakes"));
        assert!(explanation.contains("$250.00"));
        assert!(explanation.contains("flour"));
    }

    #[test]
    fn formats_negative_cents() {
        assert_eq!(format_cents(-5), "-$0.05");
        assert_eq!(format_cents(1_234), "$12.34");
    }
}
