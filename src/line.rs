use std::fmt::Display;

use regex::Regex;

use crate::bms::{as_id, as_str};

#[derive(Debug, Clone)]
pub enum Line {
    Generic(GenericLine),
    Note(Note),
}

impl Line {
    pub fn new(line: &str) -> Self {
        let note_regex =
            Regex::new(r"#[A-Za-z0-9][A-Za-z0-9][A-Za-z0-9][A-Za-z0-9][A-Za-z0-9]:").unwrap();

        if note_regex.is_match(line) {
            if let Some(new_note) = Note::new(line) {
                return Line::Note(new_note);
            }
        }

        Self::Generic(GenericLine::new(line.to_string()))
    }

    pub fn as_note(&self) -> Option<&Note> {
        match &self {
            Self::Note(n) => Some(n),
            _ => None,
        }
    }
}

impl Display for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = match self {
            Line::Generic(generic_line) => generic_line.line().trim_end().to_string(),
            Line::Note(note) => note.to_string(),
        };

        write!(f, "{}", val)
    }
}

#[derive(Debug, Clone)]
pub struct Note {
    measure: u32,
    channel: u32,
    keysounds: Vec<u64>,
}

impl Note {
    pub fn new(line: &str) -> Option<Self> {
        if !Self::line_is_note(line) {
            return None;
        }

        let measure = match (line[1..4]).parse::<u32>() {
            Ok(v) => v,
            Err(e) => {
                // eprintln!("Error parsing measure: {}", e);
                return None;
            }
        };

        let channel = match (line[4..6]).parse::<u32>() {
            Ok(v) => v,
            Err(e) => {
                // eprintln!("Error parsing channel: {}", e);
                return None;
            }
        };

        let mut keysounds = vec![];

        if let Some(pos) = line.find(':') {
            // let prefix = line[..=pos]; // includes ':'
            let body = line[pos + 1..].to_string();

            for i in (0..body.len()).step_by(2) {
                let chunk = &body[i..i + 2];

                let keysound_id = match as_id(chunk) {
                    Ok(v) => v,
                    Err(e) => {
                        // eprintln!("Error parsing keysound ID: {}", e);
                        return None;
                    }
                };

                keysounds.push(keysound_id);
            }
        }

        Some(Self {
            measure,
            channel,
            keysounds,
        })
    }

    pub(crate) fn replace_keysounds(&mut self, old_id: u64, new_id: u64) -> Option<()> {
        // If its not a regular p1 note, return false
        if self.channel < as_id("10").unwrap() as u32 && self.channel % 36 != 1 {
            eprintln!("Refusing to replace keysounds.");
            return None;
        }

        self.keysounds = self
            .keysounds
            .iter()
            .map(|keysound| {
                if *keysound == old_id {
                    new_id
                } else {
                    *keysound
                }
            })
            .collect();

        Some(())
    }

    pub fn channel(&self) -> u32 {
        self.channel
    }

    pub fn keysounds(&self) -> &[u64] {
        &self.keysounds
    }

    pub fn keysounds_used(&self) -> Vec<u64> {
        let mut keysounds = self.keysounds.clone();

        keysounds.sort();
        keysounds.dedup();
        keysounds
    }

    pub fn uses_keysound(&self, keysound_id: u64) -> bool {
        self.keysounds.contains(&keysound_id)
    }

    pub fn line_is_note(line: &str) -> bool {
        let note_regex =
            Regex::new(r"#[A-Za-z0-9][A-Za-z0-9][A-Za-z0-9][A-Za-z0-9][A-Za-z0-9]:").unwrap();

        note_regex.is_match(line)
    }
}

impl Display for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let keysounds_string: String = self
            .keysounds
            .iter()
            .map(|keysound| format!("{:2}", as_str(*keysound)))
            .collect::<Vec<String>>()
            .join("");

        write!(
            f,
            "#{:03}{:02}:{}",
            self.measure, self.channel, keysounds_string
        )
    }
}

#[derive(Debug, Clone)]
pub struct GenericLine {
    line: String,
}

impl GenericLine {
    pub fn new(line: String) -> Self {
        Self { line }
    }

    pub fn get_channel(&self) -> &str {
        (self.line.get(4..6).unwrap()) as _
    }

    /*
    pub fn get_keysounds(&self) -> Vec<u64> {
        let mut keysounds = vec![];

        let channel = self.get_channel();

        if let Some(pos) = self.line().find(':') {
            // let prefix = &self.line()[..=pos]; // includes ':'
            let body = &self.line()[pos + 1..];

            for i in (0..body.len()).step_by(2) {
                let chunk = &body[i..i + 2];

                keysounds.push(as_id(chunk).unwrap());
            }
        }

        keysounds.sort();
        keysounds.dedup();
        keysounds
    }
    */

    pub fn line(&self) -> &str {
        &self.line
    }

    pub fn len(&self) -> usize {
        self.line.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() > 0
    }

    pub fn line_mut(&mut self) -> &mut String {
        &mut self.line
    }
}

#[cfg(test)]
mod tests {
    use crate::bms::as_id;

    use super::*;

    #[test]
    fn test_get_keysounds() {
        let line = Line::new("#09501:S2S3S4S5S2S3S4S5S2S3S4S5S2S3S4S5");

        let keysound_ids = ["S2", "S3", "S4", "S5"]
            .map(|keysound_str| {
                let error_msg = format!("Unable to convert keysound {} to an id.", keysound_str);

                as_id(keysound_str).expect(&error_msg)
            })
            .into_iter()
            .collect::<Vec<u64>>();

        let note = line.as_note().expect("Failed to get note");

        let keysounds = note.keysounds_used();

        assert_eq!(keysounds.len(), 4, "The line should have 5 keysounds.");

        for id in keysound_ids {
            assert!(note.uses_keysound(id));
        }

        assert!(!note.uses_keysound(as_id("S1").unwrap()));
        assert!(!note.uses_keysound(as_id("S6").unwrap()));

        // assert!(keysound_ids.iter().all(|id| line.uses_keysound(*id)));
    }

    #[test]
    fn test_note_serialisation() {
        let note = Note {
            measure: 50,
            channel: 14,
            keysounds: [
                "7H", "7I", "7P", "7H", "7I", "7P", "7K", "7I", "7P", "7H", "7I", "7P", "7H", "7I",
                "7P", "7H",
            ]
            .map(|s| as_id(s).expect("Failed to create ID from strings."))
            .to_vec(),
        };

        assert_eq!(note.to_string(), "#05014:7H7I7P7H7I7P7K7I7P7H7I7P7H7I7P7H");
    }

    #[test]
    fn test_note_ser_deser() {
        let line = "#05014:7H7I7P7H7I7P7K7I7P7H7I7P7H7I7P7H";

        assert_eq!(
            Note::new(line)
                .expect("Failed to initialise note from line.")
                .to_string(),
            line
        );
    }

    #[test]
    fn test_replace_keysounds() {
        let mut note = Note {
            measure: 1,
            channel: as_id("11").unwrap() as u32,
            keysounds: vec![18, 19, 20],
        };

        assert!(note.replace_keysounds(18, 19).is_some());

        assert_eq!(note.keysounds, vec![19, 19, 20]);
    }

    #[test]
    fn test_replace_keysounds_2() {
        let mut note = Note::new("#05201:0000SU0000SV0000").unwrap();

        note.replace_keysounds(as_id("SU").unwrap(), as_id("SV").unwrap());

        assert_eq!(note.to_string(), "#05201:0000SV0000SV0000");
    }
}
