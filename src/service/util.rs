use std::collections::HashSet;

use crate::model::{
    champion::{Champion, Chroma, Skin},
    ids::SkinId,
};

use super::data_manager::{DataManager, DataRetrievalError};

pub fn get_owned_champions(manager: &DataManager) -> Result<Vec<&Champion>, DataRetrievalError> {
    let champs = manager.get_champions()?;
    Ok(champs.iter().filter(|c| c.owned).collect())
}

pub fn get_owned_skins(manager: &DataManager) -> Result<Vec<&Skin>, DataRetrievalError> {
    let champs = manager.get_skins()?;
    Ok(champs.iter().filter(|s| s.owned).collect())
}

pub fn get_owned_nobase_skins(manager: &DataManager) -> Result<Vec<&Skin>, DataRetrievalError> {
    let champs = manager.get_skins()?;
    Ok(champs.iter().filter(|s| s.owned && !s.is_base).collect())
}

pub fn get_owned_chromas(manager: &DataManager) -> Result<Vec<&Chroma>, DataRetrievalError> {
    let champs = manager.get_chromas()?;
    Ok(champs.iter().filter(|s| s.owned).collect())
}

pub fn get_owned_skins_lookup(
    manager: &DataManager,
) -> Result<HashSet<SkinId>, DataRetrievalError> {
    let owned_skins = get_owned_skins(manager)?;
    Ok(owned_skins.iter().map(|s| s.id.clone()).collect())
}
