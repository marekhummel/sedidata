use std::collections::HashMap;

use once_cell::sync::OnceCell;

#[derive(PartialEq, Eq, Hash)]
enum RequestType {
    One,
    Two,
    Three,
}

struct Cache {
    data: HashMap<RequestType, OnceCell<i32>>,
}

impl Cache {
    fn get(&self, rt: RequestType) -> &i32 {
        self.data
            .get(&rt)
            .unwrap()
            .get_or_try_init(|| request(rt))
            .unwrap()
    }
}

fn request(rt: RequestType) -> Result<i32, String> {
    Ok(match rt {
        RequestType::One => 32,
        RequestType::Two => 129,
        RequestType::Three => -3,
    })
}

pub fn main() {
    let c = Cache {
        data: vec![
            (RequestType::One, OnceCell::new()),
            (RequestType::Two, OnceCell::new()),
            (RequestType::Three, OnceCell::new()),
        ]
        .into_iter()
        .collect(),
    };

    let data1 = c.get(RequestType::One);
    let data2 = c.get(RequestType::Two);
    // let data3 = c.get(RequestType::Two);
    println!("{} {}", data1, data2);
}
