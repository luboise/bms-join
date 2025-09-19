use std::{
    env, fs,
    io::{self, BufRead, BufReader, Write},
    num::ParseIntError,
    path::PathBuf,
};

use radix_fmt::radix_36;
use regex::Regex;

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

    head: Vec<String>,
    keysounds: Vec<Keysound>,
    tail: Vec<String>,
}

fn as_id<T: AsRef<str>>(chars: T) -> Result<u64, ParseIntError> {
    u64::from_str_radix(&chars.as_ref().to_string().to_uppercase(), 36)
}

fn as_str(id: u64) -> String {
    let ret = format!("{:0>2}", radix_36(id)).to_uppercase();
    if ret.len() == 1 {
        "0".to_owned() + &ret
    } else {
        ret
    }
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
                    head.push(line);
                } else {
                    tail.push(line);
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

        for string in &self.head {
            strings.push(string.trim_end().to_string())
        }

        for keysound in &self.keysounds {
            strings.push(keysound.to_string())
        }

        for string in &self.tail {
            strings.push(string.trim_end().to_string())
        }

        strings.join("\n").as_bytes().to_vec()
    }

    fn has_keysound(&self, keysound_id: u64) -> bool {
        self.get_keysound(keysound_id).is_some()
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
    Quit,
    Unknown(char),
    Empty,
}

fn get_next_command() -> Command {
    println!(
        "\nWhat would you like to do:
        r - Replace one or more keysounds with another one
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

                let id_line = get_string();

                println!();

                let note_regex =
                    Regex::new(r"#[A-Za-z0-9][A-Za-z0-9][A-Za-z0-9][A-Za-z0-9][A-Za-z0-9]:")
                        .unwrap();

                if let Ok(id) = as_id(&id_line) {
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
                            let bad_ids: Vec<u64> = ids
                                .iter()
                                .filter(|old_id| !bms.has_keysound(**old_id))
                                .copied()
                                .collect();

                            if !bad_ids.is_empty() {
                                bad_ids.iter().for_each(|id| {
                                    eprintln!("ID {} doesn't exist in the bms file.", as_str(*id));
                                });

                                continue;
                            }

                            if ids.iter().any(|old_id| !bms.has_keysound(*old_id)) {
                                continue;
                            }

                            ids.iter().for_each(|old_id| {
                                println!("Replacing {} with {}", as_str(*old_id), as_str(id));

                                // Delete the old keysound
                                bms.keysounds.retain(|ks| ks.keysound_id != *old_id);

                                for line in &mut bms.tail {
                                    if !note_regex.is_match(line) {
                                        continue;
                                    }

                                    let channel = line.get(4..6).unwrap();

                                    if channel != "01" && channel.chars().nth(1).unwrap() != '1' {
                                        continue;
                                    }

                                    let old_id_str = as_str(*old_id);

                                    if let Some(pos) = line.find(':') {
                                        let prefix = &line[..=pos]; // includes ':'
                                        let mut new_body =
                                            String::with_capacity(line.len() - pos - 1);

                                        let body = &line[pos + 1..];

                                        for i in (0..body.len()).step_by(2) {
                                            let chunk = &body[i..i + 2];
                                            if chunk == old_id_str {
                                                new_body.push_str(&id_line.to_uppercase());
                                            } else {
                                                new_body.push_str(chunk);
                                            }
                                        }

                                        line.replace_range(pos + 1.., &new_body);
                                    }
                                }
                            });

                            if let Err(e) = bms.save() {
                                eprintln!("Error details: {}", e);
                            }
                        }
                        Err(e) => eprintln!("Error getting input ids: {}", e),
                    }
                } else {
                    eprintln!("Unable to convert line to id.");
                    continue;
                }
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
