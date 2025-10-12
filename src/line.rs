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

        let measure = (line[1..4]).parse::<u32>().expect("Bad measure from line.");
        let channel = (line[4..6]).parse::<u32>().expect("Bad channel from line.");

        let mut keysounds = vec![];

        if let Some(pos) = line.find(':') {
            // let prefix = line[..=pos]; // includes ':'
            let body = line[pos + 1..].to_string();

            for i in (0..body.len()).step_by(2) {
                let chunk = &body[i..i + 2];

                keysounds.push(as_id(chunk).unwrap());
            }
        }

        Some(Self {
            measure,
            channel,
            keysounds,
        })
    }

    pub(crate) fn replace_keysounds(&mut self, old_id: u64, new_id: u64) -> Option<()> {
        /*
                if !self.is_note() {
                    return;
                }

         if !self.is_note() || (channel != "01" && channel.chars().nth(1).unwrap() != '1') {
                    return keysounds;
                }
        */

        // If its not a regular p1 note, return false
        if self.channel < as_id("10".to_string()).unwrap() as u32 && self.channel != 0 {
            return None;
        }

        self.keysounds = self
            .keysounds
            .iter()
            .map(|keysound| match keysound {
                old_id => new_id,
                _ => *keysound,
            })
            .collect();

        Some(())

        /*
        let old_id_str = as_str(old_id);
        let new_id_str = as_str(new_id);

        if let Some(pos) = self.line().find(':') {
            // let prefix = &self.line()[..=pos]; // includes ':'
            let mut new_body = String::with_capacity(self.len() - pos - 1);

            let body = &self.line()[pos + 1..];

            for i in (0..body.len()).step_by(2) {
                let chunk = &body[i..i + 2];
                if chunk == old_id_str {
                    new_body.push_str(&new_id_str);
                } else {
                    new_body.push_str(chunk);
                }
            }

            self.line.replace_range(pos + 1.., &new_body);
        }
        */
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

impl ToString for Note {
    fn to_string(&self) -> String {
        let keysounds_string: String = self
            .keysounds
            .iter()
            .map(|keysound| format!("{:2}", as_str(*keysound)))
            .collect::<Vec<String>>()
            .join("");

        format!(
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
}
