//! Shared, solver-independent business and optimization data types.
//!
//! The domain crate deliberately contains no solver or UI dependencies. This
//! keeps the model reviewable, serializable, and reusable by a future desktop
//! application, CLI, or remote execution service.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BakeryPlan {
    pub bakery_name: String,
    pub horizon_days: u32,
    pub products: Vec<Product>,
    pub resources: Vec<Resource>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub unit: String,
    pub price_cents: u64,
    pub unit_cost_cents: u64,
    pub demand_limit: u64,
    pub recipe: Vec<RecipeLine>,
}

impl Product {
    pub fn profit_cents(&self) -> i64 {
        self.price_cents as i64 - self.unit_cost_cents as i64
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecipeLine {
    pub resource_id: String,
    pub quantity_per_unit: u64,
    pub unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Resource {
    pub id: String,
    pub name: String,
    pub unit: String,
    pub available_quantity: u64,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ValidationError {
    #[error("bakery name must not be empty")]
    EmptyBakeryName,
    #[error("planning horizon must be between 1 and 365 days")]
    InvalidHorizon,
    #[error("at least one product is required")]
    NoProducts,
    #[error("at least one resource is required")]
    NoResources,
    #[error("duplicate product id: {0}")]
    DuplicateProductId(String),
    #[error("duplicate resource id: {0}")]
    DuplicateResourceId(String),
    #[error("product {product_id} has an empty field: {field}")]
    EmptyProductField {
        product_id: String,
        field: &'static str,
    },
    #[error("resource {resource_id} has an empty field: {field}")]
    EmptyResourceField {
        resource_id: String,
        field: &'static str,
    },
    #[error("product {product_id} has no recipe lines")]
    EmptyRecipe { product_id: String },
    #[error("product {product_id} references unknown resource {resource_id}")]
    UnknownResource {
        product_id: String,
        resource_id: String,
    },
    #[error("product {product_id} has a zero recipe quantity for resource {resource_id}")]
    ZeroRecipeQuantity {
        product_id: String,
        resource_id: String,
    },
    #[error(
        "product {product_id} uses unit {recipe_unit} for resource {resource_id}, but the resource uses {resource_unit}"
    )]
    UnitMismatch {
        product_id: String,
        resource_id: String,
        recipe_unit: String,
        resource_unit: String,
    },
    #[error("product {product_id} has duplicate recipe resource {resource_id}")]
    DuplicateRecipeResource {
        product_id: String,
        resource_id: String,
    },
}

impl BakeryPlan {
    /// Validate the business model before any solver receives it.
    ///
    /// Quantities are intentionally integer and unit-labelled in this first
    /// vertical slice. The solver can therefore operate on deterministic,
    /// scaled units such as grams, minutes, and cents without floating-point
    /// input ambiguity.
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.bakery_name.trim().is_empty() {
            return Err(ValidationError::EmptyBakeryName);
        }
        if !(1..=365).contains(&self.horizon_days) {
            return Err(ValidationError::InvalidHorizon);
        }
        if self.products.is_empty() {
            return Err(ValidationError::NoProducts);
        }
        if self.resources.is_empty() {
            return Err(ValidationError::NoResources);
        }

        let resource_ids: BTreeSet<&str> = self
            .resources
            .iter()
            .map(|resource| resource.id.as_str())
            .collect();
        if resource_ids.len() != self.resources.len() {
            let mut seen = BTreeSet::new();
            if let Some(duplicate) = self
                .resources
                .iter()
                .map(|resource| resource.id.as_str())
                .find(|id| !seen.insert(*id))
            {
                return Err(ValidationError::DuplicateResourceId(duplicate.to_owned()));
            }
        }

        for resource in &self.resources {
            if resource.id.trim().is_empty() {
                return Err(ValidationError::EmptyResourceField {
                    resource_id: resource.id.clone(),
                    field: "id",
                });
            }
            if resource.name.trim().is_empty() {
                return Err(ValidationError::EmptyResourceField {
                    resource_id: resource.id.clone(),
                    field: "name",
                });
            }
            if resource.unit.trim().is_empty() {
                return Err(ValidationError::EmptyResourceField {
                    resource_id: resource.id.clone(),
                    field: "unit",
                });
            }
        }

        let mut product_ids = BTreeSet::new();
        for product in &self.products {
            if !product_ids.insert(product.id.as_str()) {
                return Err(ValidationError::DuplicateProductId(product.id.clone()));
            }
            if product.id.trim().is_empty() {
                return Err(ValidationError::EmptyProductField {
                    product_id: product.id.clone(),
                    field: "id",
                });
            }
            if product.name.trim().is_empty() {
                return Err(ValidationError::EmptyProductField {
                    product_id: product.id.clone(),
                    field: "name",
                });
            }
            if product.unit.trim().is_empty() {
                return Err(ValidationError::EmptyProductField {
                    product_id: product.id.clone(),
                    field: "unit",
                });
            }
            if product.recipe.is_empty() {
                return Err(ValidationError::EmptyRecipe {
                    product_id: product.id.clone(),
                });
            }

            let mut recipe_resources = BTreeSet::new();
            for line in &product.recipe {
                if !resource_ids.contains(line.resource_id.as_str()) {
                    return Err(ValidationError::UnknownResource {
                        product_id: product.id.clone(),
                        resource_id: line.resource_id.clone(),
                    });
                }
                if line.quantity_per_unit == 0 {
                    return Err(ValidationError::ZeroRecipeQuantity {
                        product_id: product.id.clone(),
                        resource_id: line.resource_id.clone(),
                    });
                }
                if line.unit.trim().is_empty() {
                    return Err(ValidationError::EmptyProductField {
                        product_id: product.id.clone(),
                        field: "recipe unit",
                    });
                }
                let resource = self
                    .resources
                    .iter()
                    .find(|resource| resource.id == line.resource_id)
                    .expect("resource id was validated above");
                if line.unit != resource.unit {
                    return Err(ValidationError::UnitMismatch {
                        product_id: product.id.clone(),
                        resource_id: line.resource_id.clone(),
                        recipe_unit: line.unit.clone(),
                        resource_unit: resource.unit.clone(),
                    });
                }
                if !recipe_resources.insert(line.resource_id.as_str()) {
                    return Err(ValidationError::DuplicateRecipeResource {
                        product_id: product.id.clone(),
                        resource_id: line.resource_id.clone(),
                    });
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_plan() -> BakeryPlan {
        BakeryPlan {
            bakery_name: "Test bakery".into(),
            horizon_days: 1,
            products: vec![Product {
                id: "bread".into(),
                name: "Bread".into(),
                unit: "loaf".into(),
                price_cents: 300,
                unit_cost_cents: 100,
                demand_limit: 10,
                recipe: vec![RecipeLine {
                    resource_id: "flour".into(),
                    quantity_per_unit: 200,
                    unit: "g".into(),
                }],
            }],
            resources: vec![Resource {
                id: "flour".into(),
                name: "Flour".into(),
                unit: "g".into(),
                available_quantity: 2_000,
            }],
        }
    }

    #[test]
    fn accepts_a_valid_plan() {
        assert_eq!(valid_plan().validate(), Ok(()));
    }

    #[test]
    fn rejects_unknown_recipe_resources() {
        let mut plan = valid_plan();
        plan.products[0].recipe[0].resource_id = "water".into();

        assert_eq!(
            plan.validate(),
            Err(ValidationError::UnknownResource {
                product_id: "bread".into(),
                resource_id: "water".into(),
            })
        );
    }

    #[test]
    fn rejects_duplicate_resource_ids() {
        let mut plan = valid_plan();
        plan.resources.push(plan.resources[0].clone());

        assert_eq!(
            plan.validate(),
            Err(ValidationError::DuplicateResourceId("flour".into()))
        );
    }

    #[test]
    fn rejects_recipe_unit_mismatches() {
        let mut plan = valid_plan();
        plan.products[0].recipe[0].unit = "kg".into();

        assert_eq!(
            plan.validate(),
            Err(ValidationError::UnitMismatch {
                product_id: "bread".into(),
                resource_id: "flour".into(),
                recipe_unit: "kg".into(),
                resource_unit: "g".into(),
            })
        );
    }
}
