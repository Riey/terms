use ansi_term::Color;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::Deserialize;
use std::collections::HashSet;
use std::io::{self, Write};

#[derive(Deserialize)]
struct MultipleChoiceQuestion {
    question: String,
    options: [String; 4],
    answer: char,
}

#[derive(Deserialize)]
struct MatchingPair {
    term: String,
    definition: String,
}

#[derive(Deserialize)]
struct MatchingQuestion {
    question: String,
    pairs: Vec<MatchingPair>,
}

#[derive(Deserialize)]
struct FillInTheBlankQuestion {
    question: String,
    answer: String,
}

#[derive(Deserialize)]
struct SpellingQuestion {
    question: String,
    options: [String; 3],
    answer: String,
}

#[derive(Deserialize)]
struct Chapter {
    chapter: u32,
    #[serde(default)]
    multiple_choice: Vec<MultipleChoiceQuestion>,
    #[serde(default)]
    matching: Vec<MatchingQuestion>,
    #[serde(default)]
    fill_in_the_blanks: Vec<FillInTheBlankQuestion>,
    #[serde(default)]
    spelling: Vec<SpellingQuestion>,
}

#[derive(Deserialize)]
struct Questions {
    chapters: Vec<Chapter>,
}

trait Askable {
    fn ask(&self) -> bool;
}

impl Askable for MultipleChoiceQuestion {
    fn ask(&self) -> bool {
        println!("{}", self.question);
        for (i, option) in self.options.iter().enumerate() {
            println!("{}. {}", (b'a' + i as u8) as char, option);
        }

        let answer = get_user_input("당신의 답변: ");
        let is_correct = answer == self.answer.to_string();

        print_result(is_correct, &self.answer.to_string());
        is_correct
    }
}

struct SingleMatchingQuestion {
    term: String,
    definition: Vec<String>,
    correct_answer: String,
}

impl SingleMatchingQuestion {
    fn new(term: String, pairs: &[MatchingPair]) -> Self {
        let correct_pair = pairs.iter().find(|p| p.term == term).unwrap();
        let definition = pairs.iter().map(|p| p.definition.clone()).collect();
        Self {
            term: correct_pair.term.clone(),
            definition,
            correct_answer: correct_pair.definition.clone(),
        }
    }
}

impl Askable for SingleMatchingQuestion {
    fn ask(&self) -> bool {
        println!("다음 용어에 맞는 정의를 고르세요: {}", self.term);
        for (i, definition) in self.definition.iter().enumerate() {
            println!("{}. {}", (i + 1), definition);
        }

        let answer: usize = get_user_input("당신의 답변 (정답 번호를 입력하세요): ")
            .trim()
            .parse()
            .unwrap_or(0);

        let is_correct = self.definition.get(answer - 1) == Some(&self.correct_answer);
        print_result(is_correct, &self.correct_answer);
        is_correct
    }
}

impl Askable for FillInTheBlankQuestion {
    fn ask(&self) -> bool {
        println!("{}", self.question);

        let answer = get_user_input("당신의 답변: ");
        let is_correct = answer.eq_ignore_ascii_case(&self.answer);

        print_result(is_correct, &self.answer);
        is_correct
    }
}

impl Askable for SpellingQuestion {
    fn ask(&self) -> bool {
        println!("{}", self.question);
        for option in &self.options {
            println!("{}", option);
        }

        let answer = get_user_input("당신의 답변: ");
        let is_correct = answer.eq_ignore_ascii_case(&self.answer);

        print_result(is_correct, &self.answer);
        is_correct
    }
}

fn get_user_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn print_result(is_correct: bool, correct_answer: &str) {
    if is_correct {
        println!("{}", Color::Green.paint("정답!\n"));
    } else {
        println!(
            "{} 정답은 {}\n",
            Color::Red.paint("오답입니다!"),
            Color::Green.paint(correct_answer)
        );
    }
}

fn main() {
    ansi_term::enable_ansi_support().ok();
    // let data = fs::read_to_string("questions.yaml").expect("파일을 읽을 수 없습니다");
    let data = include_str!("../data.yaml");
    let questions: Questions = serde_yaml::from_str(&data).expect("YAML 파싱 실패");

    let available_chapters: HashSet<u32> = questions.chapters.iter().map(|c| c.chapter).collect();

    let selected_chapters = loop {
        println!("다음 챕터 목록에서 하나 이상의 챕터를 선택하세요 (콤마로 구분, a를 입력하면 전부 선택):");
        for chapter in &questions.chapters {
            println!("챕터 {}", chapter.chapter);
        }

        let input = get_user_input("선택한 챕터: ");

        if input == "a" {
            break available_chapters;
        }

        let selected_chapters: HashSet<u32> = input
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();

        if !selected_chapters.is_subset(&available_chapters) {
            println!("잘못된 챕터가 포함되어 있습니다. 다시 선택하세요.\n");
            continue;
        };
        break selected_chapters;
    };

    println!("풀 문제의 개수를 입력하세요(a를 입력하면 모든 문제를 선택합니다):");
    let input = get_user_input("");

    let mut all_questions = Vec::new();

    for chapter in questions.chapters {
        if !selected_chapters.contains(&chapter.chapter) {
            continue;
        }

        all_questions.extend(
            chapter
                .multiple_choice
                .into_iter()
                .map(|q| (Box::new(q) as Box<dyn Askable>, chapter.chapter)),
        );
        for matching in &chapter.matching {
            all_questions.extend(matching.pairs.iter().map(|pair| {
                (
                    Box::new(SingleMatchingQuestion::new(
                        pair.term.clone(),
                        &matching.pairs,
                    )) as Box<dyn Askable>,
                    chapter.chapter,
                )
            }));
        }
        all_questions.extend(
            chapter
                .fill_in_the_blanks
                .into_iter()
                .map(|q| (Box::new(q) as Box<dyn Askable>, chapter.chapter)),
        );
        all_questions.extend(
            chapter
                .spelling
                .into_iter()
                .map(|q| (Box::new(q) as Box<dyn Askable>, chapter.chapter)),
        );
    }

    let range = if input == "a" {
        0..all_questions.len()
    } else {
        let mut rng = thread_rng();
        all_questions.shuffle(&mut rng);
        let num_questions: usize = input.trim().parse().unwrap_or(5);
        0..num_questions
    };

    let mut score = 0;
    let mut question_count = 0;
    for question in all_questions[range.clone()].iter() {
        question_count += 1;
        println!(
            "챕터 {} ({}/{})",
            Color::Yellow.bold().paint(question.1.to_string()),
            Color::Yellow.paint(question_count.to_string()),
            Color::Yellow.paint(range.len().to_string())
        );
        if question.0.ask() {
            score += 1;
        }
    }

    println!(
        "총 {} 문제 중 {} 개 맞췄습니다!",
        Color::Yellow.paint(question_count.to_string()),
        Color::Yellow.paint(score.to_string())
    );
    println!("나가려면 아무 키나 누르세요...");
    get_user_input("");
}
