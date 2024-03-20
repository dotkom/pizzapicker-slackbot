use lazy_static::lazy_static;
use rand::seq::SliceRandom;
use serde::{de::DeserializeOwned, Deserialize};

enum JsonFile {
    FortunePhrase,
    Pizza,
}

pub enum RouletteFilter {
    All,
    Vegan,
    Vegetarian,
}

trait Deserializable: DeserializeOwned {}

#[derive(Debug, Clone, Deserialize)]
pub struct PizzaEntry {
    pub name: String,
    pub extra: String,
    pub description: String,
    pub vegan: bool,
    pub vegetarian: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FortunePhraseEntry {
    pub phrase: String,
}

impl Deserializable for PizzaEntry {}
impl Deserializable for FortunePhraseEntry {}

lazy_static! {
    static ref PIZZAS: Vec<PizzaEntry> = load_from_json(JsonFile::Pizza);
    static ref FORTUNE_PHRASES: Vec<FortunePhraseEntry> = load_from_json(JsonFile::FortunePhrase);
}

fn load_from_json<T: Deserializable>(file: JsonFile) -> Vec<T> {
    let json = match file {
        JsonFile::FortunePhrase => include_str!("../config/fortune_phrases.json"),
        JsonFile::Pizza => include_str!("../config/pizzas.json"),
    };

    serde_json::from_str(json).expect("Failed to parse JSON configuration")
}

fn get_random_element<T>(source: Vec<T>) -> Option<T>
where
    T: Clone,
{
    let mut rng = rand::thread_rng();
    source.choose(&mut rng).cloned()
}

pub fn get_random_pizza(filter: RouletteFilter) -> PizzaEntry {
    let filtered_pizzas: Vec<PizzaEntry> = PIZZAS
        .iter()
        .filter(|pizza_entry| match filter {
            RouletteFilter::All => true,
            RouletteFilter::Vegan => pizza_entry.vegan,
            RouletteFilter::Vegetarian => pizza_entry.vegetarian,
        })
        .cloned()
        .collect();

    get_random_element(filtered_pizzas).expect("PizzaEntry vector is empty")
}

pub fn get_random_fortune_phrase() -> FortunePhraseEntry {
    let phrases = FORTUNE_PHRASES.to_vec();
    get_random_element(phrases).expect("FortunePhrases vector is empty")
}
