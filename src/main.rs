use std::fs;
use std::io;
use std::io::Write;
use thiserror::Error;

type Word = [char; 5];
type Amounts = [[u32; 5]; 26];

struct Dic {
    results: Vec<Letter>,
    known: [bool; 5],
    words: Vec<Word>,
}

#[derive(Error, Debug)]
enum Error {
    #[error("No more words can be suggested")]
    NoWord,
    #[error("IO error")]
    IO(#[from] std::io::Error),
}

#[derive(Debug)]
enum Letter {
    YesAt { c: char, i: usize },
    NotAt { c: char, i: usize },
    Not { c: char },
}

impl Dic {
    fn word_from_str(word: &str) -> Word {
        let mut result: Word = ['\0'; 5];

        for (i, c) in word.chars().take(5).enumerate() {
            result[i] = c;
        }

        result
    }

    fn new(content: &str) -> Self {
        Dic {
            results: Vec::new(),
            known: [false; 5],
            words: content.lines().map(Self::word_from_str).collect(),
        }
    }

    fn word_valid(&self, word: &Word) -> bool {
        for letter in self.results.iter() {
            match letter {
                Letter::YesAt { c, i } => {
                    if word[*i] != *c {
                        return false;
                    }
                }
                Letter::NotAt { c, i } => {
                    if word[*i] == *c {
                        return false;
                    }
                    if !word.contains(c) {
                        return false;
                    }
                }
                Letter::Not { c } => {
                    if word.contains(c) {
                        return false;
                    }
                }
            }
        }

        true
    }

    fn count_amounts(valid_words: &Vec<&Word>) -> Amounts {
        let mut amounts: Amounts = [[0; 5]; 26];

        for word in valid_words {
            for (i, c) in word.iter().enumerate() {
                amounts[*c as usize - 97][i] += 1;
            }
        }

        amounts
    }

    fn word_score(&self, word: &Word, amounts: &Amounts) -> u32 {
        let mut score = 0;

        let mut found = Vec::with_capacity(5);
        for (i, c) in word.iter().enumerate() {
            if found.contains(&c) {
                continue;
            }
            found.push(c);

            let k = *c as usize - 97;
            for l in 0..5 {
                if !self.known[l] {
                    score += amounts[k][l];
                }
            }
        }

        score
    }

    fn next_word(&self) -> Option<&Word> {
        let valid_words: Vec<&Word> = self
            .words
            .iter()
            .filter(|word| self.word_valid(word))
            .collect();

        if valid_words.len() < 3 {
            return valid_words.first().map(|word| *word);
        }
        let amounts = Self::count_amounts(&valid_words);

        /*
        for (i, a) in amounts.iter().enumerate() {
            println!(
                "{}: {:3} | {:3} | {:3} | {:3} | {:3}",
                char::from_u32((i + 97) as u32).unwrap(),
                a[0],
                a[1],
                a[2],
                a[3],
                a[4]
            );
        }
        */

        let mut current: Option<(u32, &Word)> = None;
        for word in self.words.iter() {
            let score = self.word_score(word, &amounts);

            match current {
                Some((current_score, ..)) if current_score >= score => {}
                _ => current = Some((score, word)),
            }
        }

        current.map(|v| v.1)
    }

    fn apply_result(&mut self, results: Vec<Letter>) {
        for letter in results {
            if let Letter::YesAt { c: _, i } = letter {
                self.known[i] = true;
            }

            self.results.push(letter);
        }
    }
}

fn read_result(word: &Word) -> Result<Vec<Letter>, Error> {
    let mut vec = Vec::new();

    print!("Enter result letters (c = correct, w = wrong, x = wrong position): ");
    io::stdout().flush()?;

    let mut line = String::new();
    io::stdin().read_line(&mut line)?;

    for (i, c) in line.chars().enumerate() {
        match c {
            'c' => vec.push(Letter::YesAt { c: word[i], i: i }),
            'x' => vec.push(Letter::NotAt { c: word[i], i: i }),
            'w' => vec.push(Letter::Not { c: word[i] }),
            _ => {}
        }
    }

    Ok(vec)
}

fn main() -> Result<(), Error> {
    let content = fs::read_to_string("dic")?;
    let mut dic = Dic::new(&content);

    loop {
        if let Some(word) = dic.next_word() {
            println!("Next word: {}", word.map(|v| v.to_string()).join(""));

            dic.apply_result(read_result(word)?);
        } else {
            return Err(Error::NoWord);
        }
    }
}
