const PIZZA_OPTIONS: [&str; 3] = [
    "**1 DEN ENKLE**: Ost & tomatsaus - og bare det!",
    "**2 KVESS**: Ost, tomatsaus, skinke og sjapinjong",
    "**3 DRØMMEN**: Ost, tomatsaus, kjøttdeig og rød paprika"
];

pub fn get_random_pizza() -> String {
    let random_index = rand::random::<usize>() % PIZZA_OPTIONS.len();
    PIZZA_OPTIONS[random_index].to_string()
}
