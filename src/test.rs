use once_cell::sync::OnceCell;

struct Cache {
    data: OnceCell<i32>,
}

impl Cache {
    fn get(&self) -> Result<&i32, String> {
        self.data.get_or_try_init(request)
    }

    fn refresh(&mut self) {
        println!("Refresh");
        self.data = OnceCell::new();
    }
}

fn request() -> Result<i32, String> {
    println!("Request");
    Ok(32)
}

pub fn main() {
    let mut c = Cache {
        data: OnceCell::new(),
    };

    let mut i = 0;
    let mut t = 0;
    while i < 10 && t < 5 {
        {
            let data1 = c.get();
            let data2 = c.get();
            println!("{i} {t}: {:?} {:?}", data1, data2);
            i += 1;

            if i < 9 {
                continue;
            }
        }

        c.refresh();
        t += 1;
        i = 0;
    }
}
