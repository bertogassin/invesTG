use std::collections::BTreeMap;

pub type WorldMap = BTreeMap<&'static str, BTreeMap<&'static str, Vec<&'static str>>>;

pub fn world() -> WorldMap {
    let mut world = BTreeMap::new();
    let mut europe = BTreeMap::new();

    europe.insert(
        "Франция",
        vec!["Париж", "Марсель", "Лион", "Тулуза", "Ницца"],
    );

    europe.insert("Германия", vec!["Берлин", "Гамбург", "Мюнхен", "Кёльн"]);

    europe.insert("Италия", vec!["Рим", "Милан", "Неаполь", "Турин"]);

    world.insert("Европа", europe);
    world
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn production_geography_is_stable() {
        let world = world();

        assert_eq!(world.len(), 1);

        let europe = world.get("Европа").expect("Europe");

        assert_eq!(europe.len(), 3);

        let cities: usize = europe.values().map(Vec::len).sum();

        assert_eq!(cities, 13);
        assert!(europe.get("Франция").expect("France").contains(&"Ницца"));
    }
}
