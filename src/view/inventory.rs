use std::collections::HashSet;

use crate::service::{data_manager::DataManager, dictionary::Dictionary, util};

use super::ViewError;

pub struct InventoryView<'a, 'b: 'a> {
    manager: &'a DataManager,
    dictionary: &'a Dictionary<'b>,
}

impl<'a, 'b> InventoryView<'a, 'b> {
    pub fn new(manager: &'b DataManager, dictionary: &'b Dictionary) -> Self {
        Self {
            manager,
            dictionary,
        }
    }

    pub fn champions_without_skin(&self) -> Result<(), ViewError> {
        let champs = util::get_owned_champions(&self.manager)?;
        let skins = util::get_owned_nobase_skins(&self.manager)?;

        let champs_with_skin = skins
            .iter()
            .map(|s| s.champ.clone())
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

    pub fn chromas_without_skin(&self) -> Result<(), ViewError> {
        let skins = util::get_owned_skins_lookup(&self.manager)?;
        let chromas = util::get_owned_chromas(&self.manager)?;

        let chromas_no_skin = chromas.iter().filter(|ch| !skins.contains(&ch.skin));
        for chroma in chromas_no_skin {
            let skin = self.dictionary.get_skin(chroma.skin.clone())?;
            let champ = self.dictionary.get_champion(skin.champ.clone())?;
            println!("{} ({}): {:?}", skin.name, champ.name, chroma.id);
        }
        Ok(())
    }
}
