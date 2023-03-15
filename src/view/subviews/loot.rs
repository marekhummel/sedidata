use crate::{
    model::champion::Champion,
    service::{data_manager::DataManager, lookup::LookupService, util::UtilService},
    view::ViewResult,
};

pub struct LootView<'a, 'b: 'a> {
    manager: &'a DataManager,
    lookup: &'a LookupService<'b>,
    util: &'a UtilService<'b>,
}

impl<'a, 'b> LootView<'a, 'b> {
    pub fn new(manager: &'b DataManager, lookup: &'b LookupService, util: &'b UtilService) -> Self {
        Self {
            manager,
            lookup,
            util,
        }
    }

    pub fn level_four_champs(&self) -> ViewResult {
        let mut masteries = self.util.get_masteries_with_level(4)?;
        masteries.sort_by_key(|m| m.points_to_next_level);

        let champions = masteries
            .iter()
            .map(|m| self.lookup.get_champion(m.champ_id.clone()))
            .collect::<Result<Vec<&Champion>, _>>()?;

        for (champ, mastery) in champions.iter().zip(masteries) {
            println!(
                "{:<15} ({} pts missing)",
                champ.name, mastery.points_to_next_level
            );
        }
        Ok(())
    }

    pub fn mastery_tokens(&self) -> ViewResult {
        let loot = self.manager.get_loot()?;
        let tokens = &loot.mastery_tokens;
        let champ_shards_set = self.util.get_champ_shard_set()?;

        let mut full_info = tokens
            .into_iter()
            .map(|t| {
                (
                    t,
                    self.lookup
                        .get_champion(t.champ_id.clone())
                        .map(|c| c.name.to_string()),
                    champ_shards_set.contains(&t.champ_id),
                )
            })
            .collect::<Vec<_>>();

        full_info.sort_by_key(|(t, champ_name, upgradable)| {
            (
                -(t.level as i16),
                -(t.count as i16),
                upgradable.clone(),
                champ_name.as_ref().map_or("".to_string(), |s| s.clone()),
            )
        });

        let mut output = String::new();
        for (t, champ_name, upgradable) in full_info {
            if let Err(err) = champ_name {
                return Err(err.into());
            }

            output.push_str(
                format!(
                    "{:<15} (Level {}): {}/{} tokens{}\n",
                    champ_name.unwrap(),
                    t.level,
                    t.count,
                    t.level - 4,
                    match (t.count == t.level - 4, upgradable) {
                        (true, true) => " - READY FOR UPGRADE",
                        (true, false) => " - MISSING SHARD",
                        _ => "",
                    }
                )
                .as_str(),
            );
        }
        print!("{}", output);

        Ok(())
    }

    pub fn unplayed_champs(&self) -> ViewResult {
        Ok(())
    }

    pub fn blue_essence_overview(&self) -> ViewResult {
        Ok(())
    }

    pub fn missing_champ_shards(&self) -> ViewResult {
        Ok(())
    }

    pub fn interesting_skins(&self) -> ViewResult {
        Ok(())
    }

    pub fn skin_shards_first_skin(&self) -> ViewResult {
        Ok(())
    }

    pub fn skin_shards_disenchantable(&self) -> ViewResult {
        Ok(())
    }
}
