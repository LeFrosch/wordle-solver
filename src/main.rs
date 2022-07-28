use std::env;
use std::fs;
use thiserror::Error;

const LETTER_SCORE: [i32; 26] = [
    25, // A
    9,  // B
    17, // C
    14, // D
    26, // E
    8,  // F
    10, // G
    11, // H
    23, // I
    1,  // J
    5,  // K
    18, // L
    12, // M
    20, // N
    22, // O
    13, // P
    0,  // Q
    24, // R
    19, // S
    21, // T
    15, // U
    4,  // V
    6,  // W
    3,  // X
    7,  // Y
    2,  // Z
];

#[derive(Error, Debug)]
enum Error {
    #[error("The argument `{0}` is not valid")]
    Argument(String),
    #[error("Could not access the dictionary")]
    Dictionary(#[from] std::io::Error),
}

enum Letter {
    Yes { v: char, i: [bool; 5] },
    Not { v: char },
}

fn parse_index(arg: &[char]) -> [bool; 5] {
    let mut buffer: [bool; 5] = [false; 5];

    for (i, c) in arg.iter().take(5).enumerate() {
        if *c == 'x' {
            buffer[i] = true;
        }
    }

    buffer
}

fn parse_letter(arg: &[char]) -> Option<Letter> {
    match arg {
        ['-', v] if v.is_ascii_lowercase() => Some(Letter::Not { v: *v }),
        [v, ':', r @ ..] if v.is_ascii_lowercase() => Some(Letter::Yes {
            v: *v,
            i: parse_index(r),
        }),
        _ => None,
    }
}

fn parse_letters() -> Result<Vec<Letter>, Error> {
    let mut vec = Vec::new();

    for arg in env::args().skip(1) {
        match parse_letter(&arg.chars().collect::<Vec<char>>()[..]) {
            Some(l) => vec.push(l),
            None => return Err(Error::Argument(arg)),
        }
    }

    Ok(vec)
}

fn word_score_old(word: &str, letters: &Vec<Letter>) -> i32 {
    assert!(
        !word.contains(|v: char| (v as usize) < 97),
        "Not a valid word: {}",
        word
    );

    let mut score = 0;

    for letter in letters {
        match letter {
            Letter::Not { v } => {
                if word.contains(*v) {
                    score -= 100;
                }
            }
            Letter::Yes { v, i } => {
                for (k, c) in word.chars().enumerate() {
                    if i[k] {
                        if c == *v {
                            score += 50;
                        }
                    } else {
                        if c == *v {
                            score -= 100;
                        }
                    }
                }
            }
        }
    }

    let mut encountered = ['\0'; 5];
    for (i, c) in word.chars().enumerate() {
        if encountered.contains(&c) {
            continue;
        }

        score += LETTER_SCORE[c as usize - 97];
        encountered[i] = c;
    }

    score
}

fn insert_into_buffer<'a>(word: &'a str, score: i32, buffer: &mut [Option<(i32, &'a str)>]) {
    let mut current_word = word;
    let mut current_score = score;

    for i in 0..buffer.len() {
        match buffer[i] {
            None => {
                buffer[i] = Some((current_score, current_word));
                return;
            }
            Some((score, word)) if score < current_score => {
                buffer[i] = Some((current_score, current_word));
                current_word = word;
                current_score = score;
            }
            _ => {}
        }
    }
}

fn filter_dic<'a>(dic: &[&'a str], letters: &Vec<Letter>) -> [Option<(i32, &'a str)>; 5] {
    let mut buffer: [Option<(i32, &str)>; 5] = [None; 5];

    for word in dic {
        let score = word_score_old(word, letters);

        insert_into_buffer(&word, score, &mut buffer);
    }

    buffer
}

fn invert_letters(letters: &Vec<Letter>) -> Vec<Letter> {
    let mut vec = Vec::new();

    for letter in letters {
        match letter {
            Letter::Yes { v, i } => {
                vec.push(Letter::Not { v: *v });
                vec.push(Letter::Yes { v: *v, i: *i });
                vec.push(Letter::Yes { v: *v, i: *i });
            }
            Letter::Not { v } => vec.push(Letter::Not { v: *v }),
        }
    }

    vec
}

fn print_result(result: &[Option<(i32, &str)>]) {
    for word in result {
        if let Some((s, w)) = word {
            println!("{}: {}", w, s);
        }
    }
}

fn count_amount(dic: &[&str]) {
    let mut amounts: [[u32; 5]; 26] = [[0; 5]; 26];

    for word in dic {
        for (i, c) in word.chars().enumerate() {
            amounts[c as usize - 97][i] += 1;
        }
    }
}

fn main() -> Result<(), Error> {
    let letters = parse_letters()?;

    let content = fs::read_to_string("dic")?;
    let dic: Vec<&str> = content.lines().collect();

    let result = filter_dic(&dic, &letters);

    if let Some((score, _)) = result[0] {
        print_result(&result);
        println!();
        print_result(&filter_dic(&dic, &invert_letters(&letters)))
    }

    Ok(())
}
