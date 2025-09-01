use crate::game::EntityAction;
use crate::map;
use crate::map::MapItem;

pub fn get_enemy_action<const DIMX: usize, const DIMY: usize>(
    [pos_x, pos_y]: [usize; 2],
    map: &map::Map<DIMX, DIMY>,
) -> EntityAction {
    if let Ok([hero_x, hero_y]) = find_hero_on_map(map) {
        let [diff_x, diff_y]: [i32; 2] =
            [hero_x as i32 - pos_x as i32, hero_y as i32 - pos_y as i32];
        if diff_x.abs() > diff_y.abs() {
            if diff_x < 0 {
                return EntityAction::MoveLeft;
            } else {
                return EntityAction::MoveRight;
            }
        } else {
            if diff_y < 0 {
                return EntityAction::MoveDown;
            } else {
                return EntityAction::MoveUp;
            }
        }
    } else {
        return EntityAction::None;
    }
}

fn find_hero_on_map<const DIMX: usize, const DIMY: usize>(
    map: &map::Map<DIMX, DIMY>,
) -> Result<[usize; 2], ()> {
    let working_map = map.map.expect("Map should exist!");
    for rowidx in 0..DIMX {
        for colidx in 0..DIMY {
            if working_map[rowidx][colidx] == MapItem::HeroEntity {
                return Ok([rowidx, colidx]);
            }
        }
    }
    return Err(());
}
