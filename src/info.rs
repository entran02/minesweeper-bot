use std::collections::HashMap;

pub const LOG_FLAG: &str = "FLAGGING";
pub const LOG_REVEAL: &str = "REVEALING";
pub const LOG_REVEAL_RANDOM: &str = "REVEALING RANDOM";
pub const LOG_GAME_RESET: &str = "\n---------------------RESETTING GAME------------------------------\n\n\n";
pub const LOG_GAME_COMPLETE: &str = "GAME SHOULD BE COMPLETE";

pub fn get_reps() -> HashMap<&'static str, char> {
    let mut reps = HashMap::new();
    reps.insert("square blank", '_'.into());
    reps.insert("square bombflagged", 'f'.into());
    reps.insert("square open0", '0');
    reps.insert("square open1", '1');
    reps.insert("square open2", '2');
    reps.insert("square open3", '3');
    reps.insert("square open4", '4');
    reps.insert("square open5", '5');
    reps.insert("square open6", '6');
    reps.insert("square open7", '7');
    reps.insert("square open8", '8');
    reps
}