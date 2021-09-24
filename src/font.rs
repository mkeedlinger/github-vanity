use lazy_static::lazy_static;
use std::collections::HashMap;
use toml::Value as Toml;

const FONT_TOML: &str = include_str!("./font.toml");

lazy_static! {
    pub static ref SYMBOLS: HashMap<char, Vec<bool>> = {
        let mut map = HashMap::new();

        let toml = FONT_TOML.parse::<Toml>().unwrap();
        let toml = match toml {
            Toml::Table(table) => table,
            _ => panic!(),
        };

        for (key, val) in toml {
            let val = match val {
                Toml::String(val) => val,
                _ => panic!(),
            };

            let mut bools: Vec<bool> = vec![];

            let lines: Vec<&str> = val.lines().collect();

            for i in 0..lines[0].len() {
                for j in 0..7 {
                    let designation = lines[j]
                        .chars()
                        .nth(i)
                        .expect(&format!("Font char `{}`", key));
                    let designation = match designation {
                        '-' => false,
                        'X' | 'x' => true,
                        _ => panic!(),
                    };
                    bools.push(designation);
                }
            }

            map.insert(key.chars().next().unwrap(), bools);
        }

        map
    };
}
