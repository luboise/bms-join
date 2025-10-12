use regex::Regex;

use crate::bms::as_str;

#[derive(Debug, Clone)]
pub struct Line {
    line: String,
}

impl Line {
    pub fn new(line: String) -> Self {
        Self { line }
    }

    pub fn is_note(&self) -> bool {
        let note_regex =
            Regex::new(r"#[A-Za-z0-9][A-Za-z0-9][A-Za-z0-9][A-Za-z0-9][A-Za-z0-9]:").unwrap();
        return note_regex.is_match(&self.line);
    }

    pub fn get_channel(&self) -> &str {
        let channel = self.line.get(4..6).unwrap();
        channel
    }

    pub fn uses_keysound(&self, keysound_id: u64) -> bool {
        false
    }

    pub fn get_keysounds(&self) -> Vec<u64> {
        vec![]
    }

    pub fn replace_keysounds(&mut self, old_id: u64, new_id: u64) {
        if !self.is_note() {
            return;
        }

        let channel = self.get_channel();

        if channel != "01" && channel.chars().nth(1).unwrap() != '1' {
            return;
        }

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
    }

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
        let line = Line::new("#09501:S2S3S4S5S2S3S4S5S2S3S4S5S2S3S4S5".to_string());

        let keysound_ids = ["S2", "S3", "S4", "S5"]
            .map(|keysound_str| {
                let error_msg = format!("Unable to convert keysound {} to an id.", keysound_str);

                as_id(keysound_str).expect(&error_msg)
            })
            .into_iter()
            .collect::<Vec<u64>>();

        for id in keysound_ids {
            assert!(line.uses_keysound(id));
        }

        // assert!(keysound_ids.iter().all(|id| line.uses_keysound(*id)));
    }
}
