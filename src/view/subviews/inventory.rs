use std::collections::HashSet;

use crate::{
    service::{data_manager::DataManager, lookup::LookupService, util::UtilService},
    view::ViewResult,
};

pub struct InventoryView<'a, 'b: 'a> {
    manager: &'a DataManager,
    lookup: &'a LookupService<'b>,
    util: &'a UtilService<'b>,
}

impl<'a, 'b> InventoryView<'a, 'b> {
    pub fn new(manager: &'b DataManager, lookup: &'b LookupService, util: &'b UtilService) -> Self {
        Self {
            manager,
            lookup,
            util,
        }
    }

    pub fn champions_without_skin(&self) -> ViewResult {
        let champs = self.util.get_owned_champions()?;
        let skins = self.util.get_owned_nobase_skins()?;

        let champs_with_skin = skins
            .iter()
            .map(|s| s.champ_id.clone())
            .collect::<HashSet<_>>();
        let mut champs_no_skin = champs
            .iter()
            .filter(|c| !champs_with_skin.contains(&c.id))
            .collect::<Vec<_>>();
        champs_no_skin.sort_by_key(|c| c.name.clone());
        for champ in champs_no_skin {
            println!("{}", champ.name);
        }
        Ok(())
    }

    pub fn chromas_without_skin(&self) -> ViewResult {
        let skins = self.util.get_owned_skins_set()?;
        let chromas = self.util.get_owned_chromas()?;

        let chromas_no_skin = chromas.iter().filter(|ch| !skins.contains(&ch.skin_id));
        for chroma in chromas_no_skin {
            let skin = self.lookup.get_skin(chroma.skin_id.clone())?;
            let champ = self.lookup.get_champion(skin.champ_id.clone())?;
            println!("{} ({}): {:?}", skin.name, champ.name, chroma.id);
        }
        Ok(())
    }
}
