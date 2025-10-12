use std::{
    env, fs,
    io::{self, BufRead, BufReader, Write},
    num::ParseIntError,
    path::PathBuf,
};

pub mod bms;
pub mod line;

use line::Line;

use crate::bms::{as_id, as_str};

#[derive(Debug, Clone)]
struct Keysound {
    keysound_id: u64,
    keysound_file: String,
}

impl Keysound {
    pub fn from_line(line: &str) -> Result<Self, ParseIntError> {
        let keysound_id = line[4..6].to_string();
        let keysound_file = line[7..].to_string();

        Ok(Keysound {
            keysound_id: as_id(&keysound_id)?,
            keysound_file,
        })
    }

    pub fn to_string(&self) -> String {
        format!("#WAV{} {}", as_str(self.keysound_id), self.keysound_file)
    }
}

#[derive(Debug, Clone)]
struct BMSFile {
    path: PathBuf,

    head: Vec<Line>,
    keysounds: Vec<Keysound>,
    tail: Vec<Line>,
}

impl BMSFile {
    pub fn from_path(path: &PathBuf) -> Result<Self, std::io::Error> {
        let mut head = Vec::new();
        let mut keysounds: Vec<Keysound> = Default::default();
        let mut tail = Vec::new();

        BufReader::new(fs::File::open(path).expect("Unable to open file"))
            .lines()
            .filter(|line| line.is_ok())
            .map(|line| line.unwrap())
            .for_each(|line| {
                if line.starts_with("#WAV") {
                    keysounds.push(Keysound::from_line(&line).expect("Can't parse line."));
                } else if keysounds.is_empty() {
                    head.push(Line::new(line));
                } else {
                    tail.push(Line::new(line));
                }
            });

        Ok(BMSFile {
            path: path.clone(),
            head,
            keysounds,
            tail,
        })
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut strings: Vec<String> = Vec::new();

        for line in &self.head {
            strings.push(line.line().trim_end().to_string())
        }

        for keysound in &self.keysounds {
            strings.push(keysound.to_string())
        }

        for line in &self.tail {
            strings.push(line.line().trim_end().to_string())
        }

        strings.join("\n").as_bytes().to_vec()
    }

    fn has_keysound(&self, keysound_id: u64) -> bool {
        self.get_keysound(keysound_id).is_some()
    }

    fn uses_keysound(&self, keysound_id: u64) -> bool {
        // Make sure the keysound actually exists
        if !self.get_keysound(keysound_id).is_some() {
            return false;
        }

        return self.tail.iter().any(|line| {
            line.is_note()
                && line
                    .get_keysounds()
                    .iter()
                    .any(|keysound| *keysound == keysound_id)
        });
    }

    fn get_keysound(&self, id: u64) -> Option<&Keysound> {
        self.keysounds.iter().find(|ks| ks.keysound_id == id)
    }

    fn get_keysound_mut(&mut self, id: u64) -> Option<&mut Keysound> {
        self.keysounds.iter_mut().find(|ks| ks.keysound_id == id)
    }

    fn keysounds(&self) -> &[Keysound] {
        &self.keysounds
    }

    fn reload(&mut self) -> Result<(), std::io::Error> {
        println!("Reloading {}", self.path.display());

        match Self::from_path(&self.path) {
            Ok(new_bms) => {
                self.head = new_bms.head;
                self.keysounds = new_bms.keysounds;
                self.tail = new_bms.tail;

                Ok(())
            }
            Err(e) => {
                eprintln!("Error reloading the BMS file. Check that it still exists.");
                self.head.clear();
                self.keysounds.clear();
                self.tail.clear();

                Err(e)
            }
        }
    }

    fn save(&self) -> Result<(), std::io::Error> {
        println!("Saving {}", self.path.display());
        fs::write(&self.path, self.to_bytes())
    }
}

pub enum Command {
    Replace,
    Merge,
    HandleUnused,
    Quit,
    Unknown(char),
    Empty,
}

fn get_next_command() -> Command {
    println!(
        "\nWhat would you like to do:
        r - Replace one or more keysounds with another one
        u - View unused keysounds.
        q - Quit the program\n\n"
    );

    /*
    println!(
        "\nWhat would you like to do:
        r - Replace one or more keysounds with another one
        m - Merge multiple keysounds into a single keysound
        q - Quit the program\n\n"
    );
    */

    let input = get_string();

    if input.is_empty() {
        return Command::Empty;
    }

    match input.chars().next().unwrap() {
        'r' => Command::Replace,
        'm' => Command::Merge,
        'u' => Command::HandleUnused,
        'q' => Command::Quit,
        val => Command::Unknown(val),
    }
}

fn get_string() -> String {
    let mut line = String::new();

    match io::stdin().read_line(&mut line) {
        Ok(_) => line.trim_end().to_string(),
        Err(_) => {
            eprintln!("{}", line);
            "".to_string()
        }
    }
}

fn get_strings(separating_char: char) -> Vec<String> {
    let line = get_string();

    line.split(separating_char)
        .map(|slice| slice.to_string())
        .collect()
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let bms_path: PathBuf = (*args[1]).into();
    // let replacements_path: PathBuf = (*args[2]).into();

    /*
    let modifcations: Modifications = serde_json::from_reader(BufReader::new(
        fs::File::open(replacements_path).expect("Unabel to open novas pussy for reading"),
    ));
    */

    fs::copy(
        &bms_path,
        bms_path.parent().unwrap().join(format!(
            "{}_backup.bms",
            bms_path.file_stem().unwrap().to_str().unwrap()
        )),
    )
    .expect("Unable to backup file.");

    let mut bms = BMSFile::from_path(&bms_path).expect("Unable to read bms file.");

    let mut quit = false;

    loop {
        if quit {
            break;
        }

        match get_next_command() {
            Command::Replace => {
                print!(
                    "Enter the ID (eg. 0A) of the keysound which you would like to replace with: "
                );

                io::stdout().flush().expect("Unable to flush stdout.");

                let new_id_line = get_string();

                println!();

                if let Ok(id) = as_id(&new_id_line) {
                    // Reload after getting user input
                    if let Err(e) = bms.reload() {
                        eprintln!("Error details: {}", e);
                        continue;
                    }
                    if !bms.has_keysound(id) {
                        eprintln!("No keysound exists with id {}", as_str(id));
                        continue;
                    }

                    print!("Enter the ID's which you would like replaced (eg. 0B,0C,0D,0E): ");
                    io::stdout().flush().expect("Unable to flush stdout.");

                    let id_list = get_strings(',');

                    println!();

                    let res_ids: Result<Vec<u64>, ParseIntError> =
                        id_list.iter().map(as_id).collect();

                    // Recheck the id in case the user edited the file in their own editor
                    if let Err(e) = bms.reload() {
                        eprintln!("Error details: {}", e);
                        continue;
                    }
                    if !bms.has_keysound(id) {
                        eprintln!("No keysound exists with id {}", as_str(id));
                        continue;
                    }

                    match res_ids {
                        Ok(ids) => {
                            let old_ids: Vec<u64> = ids
                                .iter()
                                .filter(|old_id| !bms.has_keysound(**old_id))
                                .copied()
                                .collect();

                            if !old_ids.is_empty() {
                                old_ids.iter().for_each(|id| {
                                    eprintln!("ID {} doesn't exist in the bms file.", as_str(*id));
                                });

                                continue;
                            }

                            // Skip invalid keysounds
                            if ids.iter().any(|old_id| !bms.has_keysound(*old_id)) {
                                continue;
                            }

                            let new_id_upper = &new_id_line.to_uppercase();

                            if let Ok(new_id) = as_id(new_id_upper) {
                                old_ids.iter().for_each(|old_id| {
                                    println!("Replacing {} with {}", as_str(*old_id), as_str(id));

                                    // Delete the old keysound
                                    bms.keysounds.retain(|ks| ks.keysound_id != *old_id);

                                    for line in &mut bms.tail {
                                        line.replace_keysounds(*old_id, new_id)
                                    }
                                });

                                if let Err(e) = bms.save() {
                                    eprintln!("Error details: {}", e);
                                }
                            } else {
                                eprintln!("Error converting id {}.", new_id_upper);
                            }
                        }
                        Err(e) => eprintln!("Error getting input ids: {}", e),
                    }
                } else {
                    eprintln!("Unable to convert line to id.");
                    continue;
                }
            }
            Command::HandleUnused => {
                let unused_keysounds: Vec<&Keysound> = bms
                    .keysounds
                    .iter()
                    .filter(|keysound| !bms.has_keysound(keysound.keysound_id))
                    .collect();

                println!(
                    "The following keysounds are unused: {:?}",
                    unused_keysounds
                        .to_owned()
                        .iter()
                        .map(|keysound| keysound.keysound_id)
                        .collect::<Vec<u64>>()
                );
            }

            Command::Merge => {
                //
                continue;
            }
            Command::Unknown(c) => eprintln!("Unknown command: {}", c),
            Command::Empty => continue,
            Command::Quit => quit = true,
        }
    }
}
