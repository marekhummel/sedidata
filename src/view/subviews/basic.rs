use crate::{
    service::{
        data_manager::DataManager,
        lookup::{self, LookupService},
        util::UtilService,
    },
    view::ViewResult,
};

pub struct BasicView<'a, 'b: 'a> {
    manager: &'a DataManager,
    lookup: &'a LookupService<'b>,
    util: &'a UtilService<'b>,
}

impl<'a, 'b> BasicView<'a, 'b> {
    pub fn new(manager: &'b DataManager, lookup: &'b LookupService, util: &'b UtilService) -> Self {
        Self {
            manager,
            lookup,
            util,
        }
    }

    pub fn print_summoner(&self) -> ViewResult {
        let summoner = self.manager.get_summoner();
        println!("{:?}\n", summoner);
        Ok(())
    }
}
