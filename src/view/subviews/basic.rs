use crate::{service::data_manager::DataManager, view::ViewResult};

pub struct BasicView<'a> {
    manager: &'a DataManager,
}

impl<'a> BasicView<'a> {
    pub fn new<'b: 'a>(manager: &'b DataManager) -> Self {
        Self { manager }
    }

    pub fn print_summoner(&self) -> ViewResult {
        let summoner = self.manager.get_summoner();
        println!("{:?}", summoner);
        Ok(())
    }
}
