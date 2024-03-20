use lazy_static::lazy_static;
use rand::Rng;
use serde::{de::DeserializeOwned, Deserialize};

enum JsonFile {
    Pizza,
    Roulette,
}

pub enum SpinMode {
    Vegan,
    Vegetarian,
    Any,
}

trait Deserializable: DeserializeOwned {}

#[derive(Debug, Clone, Deserialize)]
pub struct PizzaDetail {
    pub name: String,
    pub extra: String,
    pub description: String,
    pub vegan: bool,
    pub vegetarian: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RouletteMessage {
    phrase: String,
}

impl Deserializable for PizzaDetail {}
impl Deserializable for RouletteMessage {}

lazy_static! {
    static ref PIZZA_OPTIONS: Vec<PizzaDetail> = load_from_json(JsonFile::Pizza);
    static ref ROULETTE_MESSAGES: Vec<RouletteMessage> = load_from_json(JsonFile::Roulette);
}

fn load_from_json<T: Deserializable>(file: JsonFile) -> Vec<T> {
    let json = match file {
        JsonFile::Pizza => include_str!("../config/pizza.json"),
        JsonFile::Roulette => include_str!("../config/roulette.json"),
    };
    let result: Vec<T> = serde_json::from_str(json).expect("Failed to parse JSON configuration");
    result
}

pub fn get_random_pizza(mode: SpinMode) -> PizzaDetail {
    let mut rng = rand::thread_rng();
    let mut filtered_pizzas: Vec<&PizzaDetail> = PIZZA_OPTIONS
        .iter()
        .filter(|p| match mode {
            SpinMode::Vegan => p.vegan,
            SpinMode::Vegetarian => p.vegetarian,
            SpinMode::Any => true,
        })
        .collect();
    let random_index = rng.gen_range(0..filtered_pizzas.len());
    filtered_pizzas.remove(random_index).clone()
}

pub fn get_random_roulette_message() -> String {
    let mut rng = rand::thread_rng();
    let phrases: Vec<RouletteMessage> = ROULETTE_MESSAGES.to_vec();
    let random_index = rng.gen_range(0..phrases.len());
    phrases.get(random_index).unwrap().phrase.clone()
}
