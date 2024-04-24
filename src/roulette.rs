use lazy_static::lazy_static;
use rand::seq::SliceRandom;
use serde::{de::DeserializeOwned, Deserialize};

pub enum SpinMode {
    Any,
    Vegan,
    Vegetarian,
}

impl SpinMode {
    pub fn from_command(s: &str) -> Option<Self> {
        match s {
            "/spin" => Some(SpinMode::Any),
            "/spin-vegan" => Some(SpinMode::Vegan),
            "/spin-vegetarian" => Some(SpinMode::Vegetarian),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PizzaEntry {
    pub name: String,
    pub extra: String,
    pub description: String,
    /// Indicates that the pizza contains only vegan-friendly ingredients
    #[serde(default)]
    /// Indicates that the pizza contains only vegetarian-friendly ingredients
    pub vegan: bool,
    #[serde(default)]
    pub vegetarian: bool,
    /// Indicates that the pizza is a personal size
    #[serde(default)]
    pub personal: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FortunePhraseEntry {
    pub phrase: String,
}

lazy_static! {
    static ref PIZZAS: Vec<PizzaEntry> = load_json(include_str!("../config/pizzas.json"));
    static ref FORTUNE_PHRASES: Vec<FortunePhraseEntry> =
        load_json(include_str!("../config/fortune_phrases.json"));
}

fn load_json<T: DeserializeOwned>(include_str: &str) -> Vec<T> {
    serde_json::from_str(include_str).expect("Failed to parse JSON configuration")
}

fn get_random_element<T>(source: &[T]) -> Option<&T> {
    let mut rng = rand::thread_rng();
    source.choose(&mut rng)
}

/// Get a random pizza from the registry given a filter
///
/// If the filter is [SpinMode::Any], then you cannot get a personal size pizza.
/// If the filter is [SpinMode::Vegan], then you can only get a vegan pizza, which also implies that
/// the pizza is personal size (see the menu)
/// If the filter is [SpinMode::Vegetarian], then you can only get a vegetarian pizza, and you
/// cannot get a personal size pizza.
pub fn get_random_pizza(filter: SpinMode) -> &'static PizzaEntry {
    let filtered_pizzas: Vec<&PizzaEntry> = PIZZAS
        .iter()
        .filter(|pizza_entry| match filter {
            SpinMode::Any => !pizza_entry.personal,
            SpinMode::Vegan => pizza_entry.vegan,
            SpinMode::Vegetarian => {
                !pizza_entry.personal && pizza_entry.vegetarian && pizza_entry.vegan
            }
        })
        .collect();

    get_random_element(&filtered_pizzas).expect("PizzaEntry vector is empty")
}

pub fn get_random_fortune_phrase() -> &'static FortunePhraseEntry {
    get_random_element(&FORTUNE_PHRASES).expect("FortunePhrases vector is empty")
}
