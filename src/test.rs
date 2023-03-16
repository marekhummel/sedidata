use std::{
    cell::RefCell,
    collections::{hash_map::Entry, HashMap},
    rc::Rc,
};

#[derive(PartialEq, Eq, Hash)]
enum RequestType {
    One,
    Two,
    Three,
}

struct Cache {
    data: RefCell<HashMap<RequestType, Rc<i32>>>,
}

impl Cache {
    fn get(&self, rt: RequestType) -> Rc<i32> {
        match self.data.borrow_mut().entry(rt) {
            Entry::Occupied(oe) => oe.get().clone(),
            Entry::Vacant(ve) => {
                let new_value = Rc::new(request(ve.key()).unwrap());
                ve.insert(new_value.clone());
                new_value
            }
        }
    }
}

fn request(rt: &RequestType) -> Result<i32, String> {
    Ok(match rt {
        RequestType::One => 32,
        RequestType::Two => 129,
        RequestType::Three => -3,
    })
}

pub fn main() {
    let c = Cache {
        data: RefCell::from(HashMap::new()),
    };

    let data1 = c.get(RequestType::One);
    let data2 = c.get(RequestType::Two);
    let data3 = c.get(RequestType::One);
    println!("{:?} {:?} {:?}", data1, data2, data3);
}
