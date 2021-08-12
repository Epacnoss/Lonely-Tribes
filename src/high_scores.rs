use crate::states::game_state::{get_levels, LEVELS};
use ron::{from_str, to_string};
use serde::{Deserialize, Serialize};
use std::fs::{create_dir, read_to_string, write};

///The path to the high scores
const HIGH_SCORES_PATH: &str = "assets/data/high_scores.ron";
///The data dir
pub const DATA_DIR: &str = "assets/data";

///Struct to score High Scores
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HighScores {
    ///HashMap of high scores, with the key being the level index, and the i32 being the number of moves
    pub scores: Vec<i32>,
}
impl Default for HighScores {
    fn default() -> Self {
        Self {
            scores: Vec::with_capacity(get_levels().len()),
        }
    }
}
impl HighScores {
    pub fn new() -> Self {
        let file = read_to_string(HIGH_SCORES_PATH).unwrap_or_else(|_| "".to_string());

        let scores: Vec<i32> = from_str(file.as_str()).unwrap_or_default();

        Self { scores }
    }

    ///Function to add a score, check whether it is better than the written down one, and if so write it to a file
    ///
    /// Returns an option
    /// If it is None, then the high score was beaten
    /// If Some, then the i32 is the old high score
    pub fn add_score_and_write(&mut self, index: usize, score: i32) -> Option<i32> {
        let mut new_high_score = false;
        let current = {
            if self.scores.len() > index {
                //to avoid panicking on overflow
                self.scores.remove(index)
            } else {
                i32::MAX
            }
        };
        if score < current {
            new_high_score = true;
            self.scores.insert(index, score);

            self.write_self_to_file();
        } else {
            self.scores.insert(index, current);
        }

        if new_high_score {
            None
        } else {
            Some(current)
        }
    }

    ///Function to check whether or not a level has been beaten yet
    ///
    /// Simple function, but better for reading code later
    pub fn get_high_score(&self, level: usize) -> Option<i32> {
        self.scores.get(level).copied()
    }

    ///Function to find what the next level to be played is
    pub fn find_next_level(&self) -> usize {
        let mut i = 0;
        let mut an_unfinished_level_exists = false;
        for level in 0..LEVELS.len() {
            if self.get_high_score(level).is_some() {
                i = level + 1;
            } else {
                an_unfinished_level_exists = true;
            }
        }
        if an_unfinished_level_exists {
            i
        } else {
            i - 1
        }
    }

    ///Function to serialise the scores to a file
    fn write_self_to_file(&self) {
        let text = to_string(&self.scores);
        if let Ok(text) = text {
            write(HIGH_SCORES_PATH, &text).unwrap_or_else(|_| {
                create_dir(DATA_DIR)
                    .unwrap_or_else(|err| log::error!("Unable to create data directory: {}", err));
                write(HIGH_SCORES_PATH, &text)
                    .unwrap_or_else(|err| log::error!("Unable to write high scores: {}", err));
            });
        }
    }
}
