use lazy_static::lazy_static;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct PizzaDetail {
    pub name: String,
    pub extra: String,
    pub description: String
}

lazy_static! {
    static ref PIZZA_OPTIONS: Vec<PizzaDetail> = get_pizzas_from_configuration();
}

fn get_pizzas_from_configuration() -> Vec<PizzaDetail> {
    let json = include_str!("../config/pizza.json");
let pizzas: Vec<PizzaDetail> = serde_json::from_str(json)
    .expect("Failed to parse pizza configuration");
pizzas
}

pub fn get_random_pizza() -> PizzaDetail {
    let random_index = rand::random::<usize>() % PIZZA_OPTIONS.len();
    PIZZA_OPTIONS[random_index].clone()
}
