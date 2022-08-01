use std::fs;
use std::io;
use std::io::Write;
use thiserror::Error;

type Word = [char; 5];

#[derive(Error, Debug)]
enum Error {
    #[error("No more words can be suggested")]
    NoWord,
    #[error("IO error")]
    IO(#[from] std::io::Error),
}

#[derive(Debug)]
enum Letter {
    Not(char),
    NotAt(char, usize),
    Yes(char),
    YesAt(char, usize),
}

fn word_from_str(word: &str) -> Word {
    let mut result: Word = ['\0'; 5];

    for (i, c) in word.chars().take(5).enumerate() {
        result[i] = c;
    }

    result
}

fn word_to_str(word: &Word) -> String {
    word.map(|v| v.to_string()).join("")
}

fn get_result(guess: &Word, word: &Word) -> Vec<Letter> {
    let mut result = Vec::with_capacity(guess.len());

    for (i, g) in guess.iter().enumerate() {
        if word.contains(g) {
            result.push(Letter::Yes(*g));

            if word[i] != *g {
                result.push(Letter::NotAt(*g, i))
            }
        } else {
            result.push(Letter::Not(*g));
        }
    }

    return result;
}

fn word_valid(word: &Word, result: &Vec<Letter>) -> bool {
    for letter in result.iter() {
        match letter {
            Letter::YesAt(c, i) => {
                if word[*i] != *c {
                    return false;
                }
            }
            Letter::Yes(c) => {
                if !word.contains(c) {
                    return false;
                }
            }
            Letter::NotAt(c, i) => {
                if word[*i] == *c {
                    return false;
                }
            }
            Letter::Not(c) => {
                if word.contains(c) {
                    return false;
                }
            }
        }
    }

    true
}

fn next_guess(dic: &Vec<Word>, result: &Vec<Letter>) -> Option<Word> {
    if result.is_empty() {
        return Some(word_from_str("tares"));
    }

    let dic_valid = || dic.iter().filter(|v| word_valid(v, result));

    let mut current_word: Option<(Word, usize)> = None;
    for guess in dic {
        let mut score = 0;

        for word in dic_valid() {
            let new_result = get_result(guess, word);

            score += dic_valid().filter(|w| !word_valid(w, &new_result)).count();
        }

        match current_word {
            None => current_word = Some((*guess, score)),
            Some((_, s)) if s < score => current_word = Some((*guess, score)),
            _ => {}
        }
    }

    current_word.map(|v| v.0)
}

fn read_result(word: &Word) -> Result<Vec<Letter>, Error> {
    let mut vec = Vec::new();

    print!("Enter result letters (c = correct, w = wrong, x = wrong position): ");
    io::stdout().flush()?;

    let mut line = String::new();
    io::stdin().read_line(&mut line)?;

    for (i, c) in line.chars().enumerate() {
        match c {
            'c' => vec.push(Letter::YesAt(word[i], i)),
            'x' => {
                vec.push(Letter::NotAt(word[i], i));
                vec.push(Letter::Yes(word[i]));
            }
            'w' => vec.push(Letter::Not(word[i])),
            _ => {}
        }
    }

    Ok(vec)
}

fn main() -> Result<(), Error> {
    let content = fs::read_to_string("dic")?;
    let dic: Vec<Word> = content.lines().map(word_from_str).collect();
    let mut result: Vec<Letter> = Vec::new();

    loop {
        let dic_valid: Vec<&Word> = dic.iter().filter(|v| word_valid(v, &result)).collect();

        if dic_valid.len() == 1 {
            println!("Result: {}", word_to_str(dic_valid.first().unwrap()));
            break;
        } else {
            println!("Words left: {}", dic_valid.len());
        }

        if let Some(word) = next_guess(&dic, &result) {
            println!("Next word: {}", word_to_str(&word));

            for r in read_result(&word)? {
                result.push(r);
            }
        } else {
            return Err(Error::NoWord);
        }
    }

    Ok(())
}
