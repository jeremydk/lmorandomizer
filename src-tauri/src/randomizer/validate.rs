use crate::dataset::storage::Storage;

pub fn validate(storage: &Storage) -> bool {
    let mut current_item_names: Vec<String> = vec![];
    let mut playing = storage.clone();
    for _ in 0..100 {
        let sacred_orb_count = current_item_names
            .iter()
            .filter(|x| x.starts_with("sacredOrb:"))
            .count() as u8;
        let result = playing.split_reachables_unreachables(&current_item_names, sacred_orb_count);
        let mut reached = result.0;
        if reached.is_empty() {
            return false;
        }
        playing = result.1;

        if playing.all_items().is_empty() {
            log::trace!("true");
            return true;
        }
        current_item_names.append(&mut reached);
    }
    unreachable!();
}
