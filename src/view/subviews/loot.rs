use std::{
    cmp::max,
    collections::{HashMap, HashSet},
};

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
        println!("Champions that are mastery level 4:\n");

        let mut masteries = self.util.get_masteries_with_level(vec![4])?;
        masteries.sort_by_key(|m| m.points_to_next_level);

        let champions = masteries
            .iter()
            .map(|m| self.lookup.get_champion(&m.champ_id))
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
        println!("Mastery tokens and if they can be upgraded:\n");
        let maxed_masteries = self.util.get_masteries_with_level(vec![5, 6])?;
        let champ_shards_set = self.util.get_champ_shard_set()?;

        let mut full_info = maxed_masteries
            .into_iter()
            .map(|m| {
                (
                    m.level,
                    m.tokens.unwrap_or(0),
                    self.lookup
                        .get_champion(&m.champ_id)
                        .map(|c| c.name.to_string()),
                    champ_shards_set.contains(&m.champ_id),
                )
            })
            .collect::<Vec<_>>();

        full_info.sort_by_key(|(level, tokens, champ_name, upgradable)| {
            (
                -(*level as i16),
                -(*tokens as i16),
                upgradable.clone(),
                champ_name.as_ref().map_or("".to_string(), |s| s.clone()),
            )
        });

        let mut output = String::new();
        for (level, tokens, champ_name, upgradable) in full_info {
            if let Err(err) = champ_name {
                return Err(err.into());
            }

            output.push_str(
                format!(
                    "{:<15} (Level {}): {}/{} tokens{}\n",
                    champ_name.unwrap(),
                    level,
                    tokens,
                    level - 3,
                    match (tokens == level - 3, upgradable) {
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
        println!("Champions with 0 mastery points:\n");

        let champs = self.manager.get_champions()?;
        let played_champs = self.util.get_played_champions_set()?;

        let mut unplayed = champs
            .iter()
            .filter(|c| !played_champs.contains(&c.id))
            .collect::<Vec<_>>();
        unplayed.sort_by_key(|c| c.name.as_str());

        for c in &unplayed {
            println!("{}", c.name);
        }

        println!("\n{} champ(s) total", &unplayed.len());
        Ok(())
    }

    pub fn blue_essence_overview(&self) -> ViewResult {
        let loot = self.manager.get_loot()?;
        let be = loot.credits.blue_essence;
        let champ_shards = &loot.champion_shards;

        let convertable = champ_shards
            .iter()
            .map(|cs| cs.count as u32 * cs.disenchant_value as u32)
            .sum::<u32>();

        let keep1 = champ_shards
            .iter()
            .map(|cs| max(cs.count as i8 - 1, 0) as u32 * cs.disenchant_value as u32)
            .sum::<u32>();

        let keep2 = champ_shards
            .iter()
            .map(|cs| max(cs.count as i8 - 2, 0) as u32 * cs.disenchant_value as u32)
            .sum::<u32>();

        println!("Current BE: {}", be);
        println!("Convertable BE: {}", convertable);
        println!("Convertable BE (Keep one shard per champ): {}", keep1);
        println!("Convertable BE (Keep two shards per champ): {}", keep2);

        Ok(())
    }

    pub fn missing_champ_shards(&self) -> ViewResult {
        println!("Champions for which no champ shard is owned:\n");

        let champs = self.manager.get_champions()?;
        let loot = self.manager.get_loot()?;
        let owned_champ_shards = loot
            .champion_shards
            .iter()
            .map(|cs| cs.champ_id.clone())
            .collect::<HashSet<_>>();

        let mut missing_cs = champs
            .iter()
            .filter(|c| !owned_champ_shards.contains(&c.id))
            .collect::<Vec<_>>();
        missing_cs.sort_by_key(|c| c.name.as_str());

        for c in &missing_cs {
            println!("{}", c.name);
        }
        println!("\n{} champ(s) total", &missing_cs.len());
        Ok(())
    }

    pub fn interesting_skins(&self) -> ViewResult {
        println!("Owned skin shards for champs with 10k or more mastery points (sorted by mastery points):\n");

        let sorted_champs = self
            .util
            .get_champions_sorted_by_mastery(None, Some(10_000))?;
        let skin_shards = &self.manager.get_loot()?.skin_shards;

        for c in sorted_champs {
            let shards = skin_shards
                .iter()
                .filter(|ss| self.lookup.get_skin(&ss.skin_id).unwrap().champ_id == c);

            let mut prefix = self.lookup.get_champion(&c)?.name.to_string();
            prefix.push_str(":");
            for shard in shards {
                let skin_name = self.lookup.get_skin(&shard.skin_id)?.name.as_str();
                println!("{:<16}  {}", prefix, skin_name);

                prefix = "".to_string();
            }
        }
        Ok(())
    }

    pub fn skin_shards_first_skin(&self) -> ViewResult {
        println!("Shows skin shards which would be the first skin for the champ (sorted by mastery points):\n");

        let skin_shards = &self.manager.get_loot()?.skin_shards;
        let skins = self.util.get_owned_nobase_skins()?;
        let champs_with_skin = skins
            .iter()
            .map(|s| s.champ_id.clone())
            .collect::<HashSet<_>>();

        let sorted_champs = self.util.get_champions_sorted_by_mastery(None, None)?;
        let champs_no_skin = sorted_champs
            .into_iter()
            .filter(|cid| !champs_with_skin.contains(&cid));

        for c in champs_no_skin {
            let shards = skin_shards
                .iter()
                .filter(|ss| self.lookup.get_skin(&ss.skin_id).unwrap().champ_id == c);

            let mut prefix = self.lookup.get_champion(&c)?.name.to_string();
            prefix.push_str(":");
            for shard in shards {
                let skin_name = self.lookup.get_skin(&shard.skin_id)?.name.as_str();
                println!("{:<16}  {}", prefix, skin_name);

                prefix = "".to_string();
            }
        }
        Ok(())
    }

    pub fn skin_shards_disenchantable(&self) -> ViewResult {
        println!("Shows skin shards for champs with less than 12000 mastery points and for which a skin is already owned (amount in parenthesis):\n");

        let skin_shards = &self.manager.get_loot()?.skin_shards;
        let skins = self.util.get_owned_nobase_skins()?;
        let skins_per_champ = skins.iter().fold(HashMap::new(), |mut map, skin| {
            *map.entry(skin.champ_id.clone()).or_insert(0u8) += 1;
            map
        });

        let champs_by_mastery = self
            .util
            .get_champions_sorted_by_mastery(Some(12_000), None)?;
        let mut sorted_champs_with_skins = champs_by_mastery
            .iter()
            .filter(|cid| skins_per_champ.contains_key(cid))
            .collect::<Vec<_>>();
        sorted_champs_with_skins.reverse();

        for c in sorted_champs_with_skins {
            let shards = skin_shards
                .iter()
                .filter(|ss| self.lookup.get_skin(&ss.skin_id).unwrap().champ_id == c.clone());

            let mut champ_prefix = self.lookup.get_champion(&c)?.name.to_string();
            champ_prefix.push_str(format!(" ({})", skins_per_champ.get(c).unwrap_or(&0)).as_str());
            champ_prefix.push_str(":");
            for shard in shards {
                let skin_name = self.lookup.get_skin(&shard.skin_id)?.name.as_str();
                println!("{:<19}  {}", champ_prefix, skin_name);

                champ_prefix = "".to_string();
            }
        }
        Ok(())
    }
}
